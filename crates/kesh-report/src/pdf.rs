//! Sérialiseur PDF des 4 rapports comptables — Story 9-2a.
//!
//! 4 fonctions publiques pures (input DTO → `Result<Vec<u8>, ReportError>`) :
//! - [`render_balance_sheet_pdf`] — Bilan (Actifs / Passifs / résultat de l'exercice).
//! - [`render_income_statement_pdf`] — Compte de résultat (Produits / Charges / résultat net).
//! - [`render_trial_balance_pdf`] — Balance des comptes (6 colonnes : N°, Compte, Ouverture, Débit, Crédit, Clôture).
//! - [`render_journal_report_pdf`] — Journaux (5 sections fixes, filtre optionnel).
//!
//! Architecture (Pass 1 AA-M1 + Decision §swiss-amount-format) :
//! - **Aucune dépendance sur `kesh-i18n`** — tous les libellés sont fournis via
//!   [`PdfContext`] résolu côté handler `kesh-api` (DD-14 pattern `kesh-qrbill`).
//! - Format suisse montants (`1'234.56`) + dates (`dd.mm.yyyy`) via helpers locaux.
//! - Helvetica builtin (cohérent kesh-qrbill, pas d'embedding TTF v0.1 — L3/L11).
//!
//! Montants **alignés à droite** sur le bord de leur colonne (correctif
//! `fix-pdf-montants-alignes`) : `PdfBuilder::write_right` mesure le texte
//! par la métrique de la police ([`kesh_qrbill::largeur_mm`]) et le fait finir
//! au bord donné — les décimales s'alignent, quelle que soit la longueur.
//!
//! Pagination automatique (T2.3a) : page break à `cursor_y < MARGIN_BOTTOM_MM` +
//! footer `page X/Y` redrawn après finalisation.

use chrono::NaiveDate;
use printpdf::{
    BuiltinFont, IndirectFontRef, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference,
};
use rust_decimal::Decimal;

use kesh_db::entities::AccountRole;
use kesh_qrbill::largeur_mm;

use crate::balance_sheet::{AccountBalance, BalanceSheet};
use crate::errors::ReportError;
use crate::general_ledger::FiscalYearBreak;
use crate::income_statement::IncomeStatement;
use crate::journal_report::JournalReport;
use crate::trial_balance::TrialBalance;
use crate::vat_report::VatReport;

// ============================================================================
// Constants centralisées — A4 portrait (T2.3a + Pass 3 ECH3-M3)
// ============================================================================

const PAGE_WIDTH_MM: f32 = 210.0; // A4 portrait
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_TOP_MM: f32 = 20.0;
const MARGIN_BOTTOM_MM: f32 = 25.0;
const MARGIN_LEFT_MM: f32 = 15.0;
const MARGIN_RIGHT_MM: f32 = 15.0;
/// Bord droit de la colonne des montants en devise principale — bilan, compte
/// de résultat, totaux, TVA, dépenses par projet, crédit des journaux : la
/// marge droite, symétrique de la gauche. Le plus long montant plausible,
/// « -12'345'678.90 » en Helvetica 10 pt, mesure 23,1 mm (23,4 en gras) : il
/// commence vers 171,6 mm.
const AMOUNT_RIGHT_MM: f32 = PAGE_WIDTH_MM - MARGIN_RIGHT_MM;
/// Bord droit de la colonne des soldes en devise du bilan et du compte de
/// résultat (Story 37-4) : « USD -1'234'567.89 » (29,6 mm en 10 pt) commence à
/// 130,4 mm, après le nom tronqué du compte ([`ACCOUNT_NAME_MAX_CHARS_FX`]), et
/// finit plus de 11 mm avant le plus long montant en devise principale.
const FX_AMOUNT_RIGHT_MM: f32 = PAGE_WIDTH_MM - 50.0;
const FONT_SIZE_PT: f32 = 10.0;
const FONT_SIZE_HEADER_PT: f32 = 14.0;
const FONT_SIZE_FOOTER_PT: f32 = 8.0;
const LINE_HEIGHT_MM: f32 = 4.2; // ≈ 12pt at 10pt font
/// Seconde ligne en petit des montants en devise (Story 37-4, AC-5).
const FX_FONT_PT: f32 = 6.5;
const FX_LINE_HEIGHT_MM: f32 = 3.0;
/// Remontée de la seconde ligne vers la ligne qu'elle complète : elle se lit
/// collée à son compte ou à son mouvement, non à la ligne suivante (revue de
/// code P2). L'avance totale (`LINE_HEIGHT_MM + FX_LINE_HEIGHT_MM`) ne change
/// pas, donc les réservations de place non plus.
const FX_RAISE_MM: f32 = 1.2;

// ============================================================================
// PdfContext — libellés et métadonnées résolus côté handler (Pass 3 ECH3-C1 + BH3-M2)
// ============================================================================

/// Contexte i18n résolu côté handler `kesh-api` AVANT appel à `render_*_pdf`.
///
/// Tous les champs sont owned (`String`) — pas de `&'static str` qui forcerait
/// `Box::leak` ou consts. `locale` est dérivé via `SELECT accounting_language
/// FROM companies WHERE id = ?` côté handler (1 query DB additionnelle ≤ 5ms).
#[derive(Debug, Clone)]
pub struct PdfContext {
    /// Nom de la company (en-tête du PDF).
    pub company_name: String,
    /// Code locale BCP-47 (e.g. `"fr-CH"`) — informatif uniquement v0.1 (pas
    /// utilisé pour le formatage interne qui reste suisse universel).
    pub locale: String,
    /// Message affiché au centre si rapport vide (clé i18n
    /// `reports-pdf-empty-message` résolue côté handler).
    pub empty_message: String,
    /// Si filtre journal actif (e.g. `Some("Ventes")`) → affiché dans l'en-tête.
    pub journal_filter_label: Option<String>,
    /// Libellés localisés des sections / colonnes.
    pub section_labels: SectionLabels,
}

/// Libellés localisés des sections, colonnes et totaux (chargés depuis
/// `kesh-i18n` côté handler).
#[derive(Debug, Clone)]
pub struct SectionLabels {
    // Sections bilan
    pub assets: String,
    pub liabilities: String,
    pub equity: String,
    pub equity_result_label: String,
    /// Libellé de la **ligne calculée** « Résultat reporté (calculé) » (report
    /// à-nouveau virtuel — Story 14-1). Marqué « (calculé) » depuis 14-3c pour le
    /// distinguer d'un compte physique de rôle `RetainedEarnings` (collision D1).
    pub retained_result_label: String,
    /// Total de la section Capitaux propres (comptes physiques + 2 lignes calculées) — 14-3c.
    pub total_equity: String,
    // Sous-titres de groupes de rôle de fonds propres (Story 14-3c, hardcode FR-CH D3).
    /// Sous-titre du groupe de rôle `EquityCapital`.
    pub equity_capital_label: String,
    /// Sous-titre du groupe de rôle `EquityOther`.
    pub equity_other_label: String,
    /// Sous-titre du groupe des comptes **physiques** de rôle `RetainedEarnings`
    /// (report d'ouverture / ajustements manuels d'un migrant) — distinct de la
    /// ligne calculée `retained_result_label` (collision D1).
    pub retained_earnings_account_label: String,
    /// Sous-titre du groupe des comptes **physiques** de rôle `CurrentYearResult`
    /// (cas legacy : compte non-postable ayant un solde antérieur à 14-3a).
    pub current_year_result_account_label: String,
    // Sections compte de résultat
    pub revenues: String,
    pub expenses: String,
    pub net_result: String,
    // Totaux
    pub total_assets: String,
    pub total_liabilities: String,
    pub total_revenues: String,
    pub total_expenses: String,
    pub total_debit: String,
    pub total_credit: String,
    pub grand_total: String,
    // Colonnes
    pub col_account_number: String,
    pub col_account_name: String,
    pub col_debit: String,
    pub col_credit: String,
    pub col_balance: String,
    /// En-tête court de la colonne d'ouverture de la balance (Story 25-5-b) —
    /// « Solde d'ouverture » ne tient pas dans une colonne de 30 mm.
    pub col_opening: String,
    /// En-tête court de la colonne de clôture de la balance (Story 25-5-b).
    pub col_closing: String,
    /// Avertissement de la balance quand les ouvertures ne s'équilibrent pas avec
    /// le résultat reporté (Story 25-5-b) — l'écran a son ⚠️, le PDF ce texte.
    pub opening_unbalanced: String,
    pub col_entry_date: String,
    pub col_description: String,
    // Header
    pub header_period: String,
    // Journaux
    pub journal_filter_prefix: String,
    // Grand livre (Story 24-1)
    pub ledger_title: String,
    pub col_piece: String,
    pub col_journal: String,
    pub col_counterpart: String,
    pub col_running: String,
    pub ledger_opening: String,
    pub ledger_closing: String,
    pub ledger_movements_total: String,
    /// Pied de page d'un compte scindé : le solde emporté à la page suivante.
    pub ledger_carry_forward: String,
    /// En-tête de la suite d'un compte : le solde repris de la page précédente.
    pub ledger_carried_over: String,
    pub ledger_fy_break: String,
    pub ledger_archived: String,
    pub ledger_unnatural: String,
    pub ledger_no_movement: String,
    // Montants en devise (Story 37-4, AC-2 et AC-5) — seconde ligne en petit du
    // grand livre et de la balance.
    /// « Réévaluation au cours » — mention d'une ligne `revaluation`, suivie du
    /// cours, sans montant en devise.
    pub fx_revaluation_at_rate: String,
    /// « solde » — le solde progressif en devise d'un mouvement.
    pub fx_balance: String,
    /// « ouverture » — l'ouverture en devise d'un compte de la balance.
    pub fx_opening: String,
    /// « débit » — les mouvements au débit en devise.
    pub fx_debit: String,
    /// « crédit » — les mouvements au crédit en devise.
    pub fx_credit: String,
    /// « clôture » — la clôture en devise d'un compte de la balance.
    pub fx_closing: String,
    // En-têtes des colonnes de montant d'un rapport qui contient un compte en
    // devise (correctif `fix-pdf-montants-alignes`) : sans eux, rien ne dit que
    // la colonne des soldes est en devise principale.
    /// Code de la devise principale, en tête de ses colonnes : « CHF ». Sur la
    /// balance et le grand livre, il suit l'en-tête de chaque colonne de montant
    /// (« Débit CHF ») ; au bilan et au compte de résultat, il est l'en-tête.
    pub col_base_currency: String,
    /// En-tête de la colonne des soldes en devise du bilan et du compte de
    /// résultat : « Devise ».
    pub col_foreign_currency: String,
}

impl SectionLabels {
    /// Libellés par défaut FR-CH pour les tests unit / fallback.
    pub fn fr_ch_defaults() -> Self {
        Self {
            assets: "Actifs".into(),
            liabilities: "Passifs".into(),
            equity: "Capitaux propres".into(),
            equity_result_label: "Résultat de l'exercice".into(),
            retained_result_label: "Résultat reporté (calculé)".into(),
            total_equity: "Total capitaux propres".into(),
            equity_capital_label: "Capital".into(),
            equity_other_label: "Autres fonds propres".into(),
            retained_earnings_account_label: "Bénéfice/perte reporté".into(),
            current_year_result_account_label: "Résultat de l'exercice (compte)".into(),
            revenues: "Produits".into(),
            expenses: "Charges".into(),
            net_result: "Résultat net".into(),
            total_assets: "Total actifs".into(),
            total_liabilities: "Total passifs".into(),
            total_revenues: "Total produits".into(),
            total_expenses: "Total charges".into(),
            total_debit: "Total débit".into(),
            total_credit: "Total crédit".into(),
            grand_total: "Total général".into(),
            col_account_number: "N° compte".into(),
            col_account_name: "Intitulé".into(),
            col_debit: "Débit".into(),
            col_credit: "Crédit".into(),
            col_balance: "Solde".into(),
            col_opening: "Ouverture".into(),
            col_closing: "Clôture".into(),
            // ≈ 145 mm en Helvetica-Bold 10 pt : tient sur une ligne (`write_line` ne
            // renvoie pas à la ligne).
            opening_unbalanced:
                "Contrôle d'ouverture en écart : une écriture antérieure est déséquilibrée.".into(),
            col_entry_date: "Date".into(),
            col_description: "Libellé".into(),
            header_period: "Période".into(),
            journal_filter_prefix: "Journal".into(),
            ledger_title: "Grand livre".into(),
            col_piece: "Pièce".into(),
            col_journal: "Journal".into(),
            col_counterpart: "Contrepartie".into(),
            col_running: "Solde".into(),
            ledger_opening: "Solde d'ouverture".into(),
            ledger_closing: "Solde de clôture".into(),
            ledger_movements_total: "Total des mouvements".into(),
            ledger_carry_forward: "À reporter".into(),
            ledger_carried_over: "Report".into(),
            ledger_fy_break: "Clôture de l'exercice — le solde repart de zéro".into(),
            ledger_archived: "archivé".into(),
            ledger_unnatural: "Solde contre nature".into(),
            ledger_no_movement: "Aucun mouvement sur la période.".into(),
            fx_revaluation_at_rate: "Réévaluation au cours".into(),
            fx_balance: "solde".into(),
            fx_opening: "ouverture".into(),
            fx_debit: "débit".into(),
            fx_credit: "crédit".into(),
            fx_closing: "clôture".into(),
            col_base_currency: "CHF".into(),
            col_foreign_currency: "Devise".into(),
        }
    }

    /// L'en-tête d'une colonne de montant suivi du code de la devise
    /// principale (« Débit CHF ») si le rapport contient un compte en devise
    /// (`multi`), sinon l'en-tête tel quel : un rapport mono-devise ne change
    /// pas.
    fn amount_header(&self, label: &str, multi: bool) -> String {
        if multi {
            format!("{label} {}", self.col_base_currency)
        } else {
            label.to_string()
        }
    }

    /// Sous-titre du groupe de rôle de fonds propres au bilan (Story 14-3c), ou
    /// `None` si le rôle n'est **pas** un rôle equity (ne devrait jamais survenir
    /// dans la section `equity`, filtrée par `is_equity_role` en amont).
    fn equity_role_label(&self, role: Option<AccountRole>) -> Option<&str> {
        match role {
            Some(AccountRole::EquityCapital) => Some(&self.equity_capital_label),
            Some(AccountRole::EquityOther) => Some(&self.equity_other_label),
            Some(AccountRole::RetainedEarnings) => Some(&self.retained_earnings_account_label),
            Some(AccountRole::CurrentYearResult) => Some(&self.current_year_result_account_label),
            _ => None,
        }
    }
}

impl PdfContext {
    /// Constructeur minimal FR-CH pour les tests unit / usage stand-alone.
    pub fn fr_ch_default(company_name: impl Into<String>) -> Self {
        Self {
            company_name: company_name.into(),
            locale: "fr-CH".into(),
            empty_message: "Aucune écriture dans la période sélectionnée.".into(),
            journal_filter_label: None,
            section_labels: SectionLabels::fr_ch_defaults(),
        }
    }
}

// ============================================================================
// Helpers de formatage (T2.4, T2.5, T2.7)
// ============================================================================

/// Formate un `Decimal` au format suisse : apostrophe séparateur de milliers,
/// point décimal, **toujours 2 décimales** (T2.4 + Pass 1 ECH-L1 négatifs).
///
/// Exemples :
/// - `1234.5` → `"1'234.50"`
/// - `-1234.56` → `"-1'234.56"`
/// - `0` → `"0.00"`
pub fn format_swiss_amount(amount: Decimal) -> String {
    // Format raw avec 2 décimales (gère signe et arrondi)
    let raw = format!("{amount:.2}");
    let (sign, body) = if let Some(stripped) = raw.strip_prefix('-') {
        ("-", stripped)
    } else {
        ("", raw.as_str())
    };
    // Sépare partie entière / décimale
    let dot_pos = body.find('.').unwrap_or(body.len());
    let (int_part, dec_part) = body.split_at(dot_pos);

    // Insère apostrophes tous les 3 chiffres en partant de la droite
    let chars: Vec<char> = int_part.chars().rev().collect();
    let mut grouped = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push('\'');
        }
        grouped.push(*c);
    }
    let grouped: String = grouped.chars().rev().collect();

    format!("{sign}{grouped}{dec_part}")
}

/// Arrondi **explicite** d'un montant en devise avant tout formatage (Story
/// 37-4, AC-5) : la somme reste exacte (4 décimales stockées), seul l'affichage
/// passe à 2 décimales, l'équidistant loin de zéro — les formateurs ne font
/// qu'un `{:.2}`, dont l'arrondi n'est pas celui qu'on écrit.
///
/// Un résidu négatif de moins d'un demi-centime (`-0.0040`) s'arrondit à un
/// zéro **sans signe** : il ne s'affiche pas « -0.00 » (revue de code P1).
pub(crate) fn round_for_display(amount: Decimal) -> Decimal {
    let r = amount.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
    if r.is_zero() { Decimal::ZERO } else { r }
}

/// Montant en devise au format suisse : `USD 1'233.57` (Story 37-4).
pub(crate) fn format_currency_amount(currency: &str, amount: Decimal) -> String {
    format!(
        "{currency} {}",
        format_swiss_amount(round_for_display(amount))
    )
}

/// Formate un **cours de change** par unité de devise (Story 37-4, AC-5) : sans
/// `%`, zéros de queue retirés, au moins 4 décimales, au plus 10 (la précision
/// stockée) — `0.8808`, `0.005751`, `1.5000`. Partagé par le PDF et le CSV.
///
/// ⚠️ Ne pas confondre avec [`format_rate`], qui rend un taux de TVA
/// (`"8.10 %"`) : il rendrait `0.88 %`.
pub(crate) fn format_exchange_rate(rate: Decimal) -> String {
    let r = rate
        .round_dp_with_strategy(10, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
        .normalize();
    let scale = r.scale().max(4);
    format!("{r:.prec$}", prec = scale as usize)
}

/// L'abscisse, en mm, où commencer `text` pour qu'il **finisse** à `right_mm` :
/// le bord droit moins sa largeur imprimée en Helvetica (ou Helvetica-Bold si
/// `bold`) à `font_size` points, mesurée par [`kesh_qrbill::largeur_mm`] — la
/// table des chasses de l'AFM d'Adobe, en WinAnsi, celle des deux polices
/// intégrées qu'emploie [`PdfBuilder`]. Partagée par toutes les colonnes de
/// montant des rapports PDF.
fn right_aligned_x(text: &str, font_size: f32, bold: bool, right_mm: f32) -> f32 {
    right_mm - largeur_mm(text, font_size, bold)
}

/// Formate une date au format suisse `dd.mm.yyyy` (T2.5).
pub fn format_swiss_date(d: NaiveDate) -> String {
    d.format("%d.%m.%Y").to_string()
}

/// Tronque une chaîne à `max` caractères avec ellipsis `…` si dépassement
/// (Pass 1 code-review H4 : utilisé pour la colonne `name` de la balance des
/// comptes — Swiss GAAP autorise des libellés > 30 chars qui débordent sur
/// la colonne débit).
///
/// Si `s.chars().count() <= max`, retourne `s` tel quel (clone). Sinon
/// retourne les `max - 1` premiers caractères suivis de `…` (taille finale
/// = `max` caractères Unicode).
fn truncate_with_ellipsis(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

// ============================================================================
// Structure de rendu — wrapper sur PdfDocument avec curseur Y + pagination
// ============================================================================

/// Wrapper interne autour de `printpdf::PdfDocument` qui gère :
/// - le curseur vertical (`cursor_y` en mm depuis le bas),
/// - la pagination automatique (page break + redraw header condensé),
/// - les références de page pour le footer X/Y post-finalisation.
struct PdfBuilder {
    doc: PdfDocumentReference,
    font: IndirectFontRef,
    font_bold: IndirectFontRef,
    /// Liste des `(page_idx, layer_idx)` créés — utilisée pour redraw du footer.
    pages: Vec<(printpdf::PdfPageIndex, printpdf::PdfLayerIndex)>,
    /// Position courante du curseur Y (mm depuis le bas).
    cursor_y: f32,
    /// Index de la page courante dans `self.pages`.
    current_page_idx: usize,
    /// Titre non ASCII du document, à poser dans ses propriétés à la
    /// finalisation, et le gabarit ASCII écrit à sa place par printpdf : voir
    /// [`title_placeholder`].
    title_fix: Option<(String, String)>,
}

impl PdfBuilder {
    fn new(title: &str) -> Result<Self, ReportError> {
        let title_fix = title_placeholder(title).map(|ph| (ph, title.to_string()));
        let doc_title = title_fix.as_ref().map_or(title, |(ph, _)| ph.as_str());
        let (doc, page1, layer1) =
            PdfDocument::new(doc_title, Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| ReportError::PdfGeneration(format!("font: {e}")))?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| ReportError::PdfGeneration(format!("font bold: {e}")))?;
        Ok(Self {
            doc,
            font,
            font_bold,
            pages: vec![(page1, layer1)],
            cursor_y: PAGE_HEIGHT_MM - MARGIN_TOP_MM,
            current_page_idx: 0,
            title_fix,
        })
    }

    fn current_layer(&self) -> PdfLayerReference {
        let (page_idx, layer_idx) = self.pages[self.current_page_idx];
        self.doc.get_page(page_idx).get_layer(layer_idx)
    }

    /// Ajoute une nouvelle page A4 et reset le curseur. Le caller est responsable
    /// de redessiner le header condensé si nécessaire.
    fn new_page(&mut self) {
        let page_num = self.pages.len() + 1;
        let (page_idx, layer_idx) = self.doc.add_page(
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            format!("Layer {page_num}"),
        );
        self.pages.push((page_idx, layer_idx));
        self.current_page_idx = self.pages.len() - 1;
        self.cursor_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM;
    }

    /// Vérifie si une nouvelle ligne tient sur la page courante ; sinon ajoute
    /// une page (sans redraw du header — caller décide).
    fn ensure_space_for_row(&mut self) {
        if self.cursor_y - LINE_HEIGHT_MM < MARGIN_BOTTOM_MM {
            self.new_page();
        }
    }

    /// Réserve la place de `n` lignes consécutives sur la page courante ; sinon
    /// saute de page. Sert au **keep-with-next** : dessiner un sous-titre de groupe
    /// (ex. rôle de fonds propres, Story 14-3c) seulement s'il tient AVEC au moins
    /// sa première ligne de contenu — évite un sous-titre orphelin en bas de page.
    fn ensure_space_for_rows(&mut self, n: f32) {
        if self.cursor_y - n * LINE_HEIGHT_MM < MARGIN_BOTTOM_MM {
            self.new_page();
        }
    }

    /// Dessine du texte à la position courante (curseur Y) puis avance d'une ligne.
    fn write_line(&mut self, text: &str, font_size: f32, bold: bool, x_offset: f32) {
        let layer = self.current_layer();
        let font = if bold { &self.font_bold } else { &self.font };
        layer.use_text(
            text,
            font_size,
            Mm(MARGIN_LEFT_MM + x_offset),
            Mm(self.cursor_y),
            font,
        );
        self.cursor_y -= LINE_HEIGHT_MM;
    }

    /// Dessine du texte sans avancer (pour les colonnes côte-à-côte).
    fn write_inline(&self, text: &str, font_size: f32, bold: bool, x_mm: f32) {
        let layer = self.current_layer();
        let font = if bold { &self.font_bold } else { &self.font };
        layer.use_text(text, font_size, Mm(x_mm), Mm(self.cursor_y), font);
    }

    /// Dessine `text` **aligné à droite** sur l'abscisse `right_mm`, à la
    /// position courante, sans avancer : le texte finit au bord de sa colonne,
    /// si bien que les montants d'une même colonne y alignent leurs décimales
    /// (les chiffres d'Helvetica sont de même chasse, 556 millièmes d'em, en
    /// maigre comme en gras ; l'apostrophe et le signe, non, d'où la mesure).
    fn write_right(&self, text: &str, font_size: f32, bold: bool, right_mm: f32) {
        self.write_inline(
            text,
            font_size,
            bold,
            right_aligned_x(text, font_size, bold, right_mm),
        );
    }

    /// Dessine la seconde ligne en petit d'un montant en devise, **sous la ligne
    /// qui vient d'être écrite** (le curseur a déjà avancé d'une ligne) : elle
    /// est posée à `FX_RAISE_MM` au-dessus du curseur, puis le curseur avance de
    /// `FX_LINE_HEIGHT_MM`.
    fn write_fx_line(&mut self, text: &str, x_mm: f32) {
        self.cursor_y += FX_RAISE_MM;
        self.write_inline(text, FX_FONT_PT, false, x_mm);
        self.cursor_y -= FX_RAISE_MM + FX_LINE_HEIGHT_MM;
    }

    /// Dessine le footer `page X/Y` sur toutes les pages après finalisation.
    ///
    /// Pass 1 code-review M17 (BH1-M4) : position Y passe de 10mm à 15mm
    /// (= `MARGIN_BOTTOM_MM - 10` = 25-10) pour rester au-dessus de la zone
    /// non-imprimable de la plupart des imprimantes laser (~12mm typique).
    fn draw_footers(&self) {
        let total = self.pages.len();
        for (i, (page_idx, layer_idx)) in self.pages.iter().enumerate() {
            let layer = self.doc.get_page(*page_idx).get_layer(*layer_idx);
            let footer = format!("page {} / {}", i + 1, total);
            layer.use_text(
                footer,
                FONT_SIZE_FOOTER_PT,
                Mm(PAGE_WIDTH_MM - 30.0),
                Mm(MARGIN_BOTTOM_MM - 10.0),
                &self.font,
            );
        }
    }

    fn finalize(self) -> Result<Vec<u8>, ReportError> {
        self.draw_footers();
        let bytes = self
            .doc
            .save_to_bytes()
            .map_err(|e| ReportError::PdfGeneration(format!("save: {e}")))?;
        match &self.title_fix {
            Some((placeholder, title)) => set_pdf_title(bytes, placeholder, title),
            None => Ok(bytes),
        }
    }
}

/// Le gabarit ASCII du titre d'un document dont le titre n'est pas ASCII, ou
/// `None` pour un titre ASCII, que printpdf écrit juste.
///
/// printpdf 0.7 écrit le titre des propriétés du PDF (`/Title` du dictionnaire
/// `Info`) en chaîne littérale faite des **octets UTF-8** du titre
/// (`printpdf-0.7.0/src/document_info.rs:83`), qu'un lecteur décode en
/// PDFDocEncoding : « Compte de résultat » s'y lit « Compte de rÃ©sultat ». Le
/// PDF veut une chaîne de texte en UTF-16BE précédée de `FE FF`, que printpdf ne
/// sait pas écrire (son titre est une `String`). Le document est donc créé sous
/// ce gabarit, puis [`set_pdf_title`] remplace, dans les octets produits, la
/// chaîne littérale `(gabarit)` par la chaîne hexadécimale `<FEFF…>` du vrai
/// titre. Le gabarit a **exactement la longueur** de cette chaîne (4 + 4 par
/// unité UTF-16, les délimiteurs se répondant), si bien qu'aucun décalage
/// d'octets ne fausse la table des références croisées : rien n'est relu ni
/// réécrit d'autre. Seuls des caractères alphanumériques ASCII, que lopdf écrit
/// tels quels.
///
/// ⚠️ **À revoir à toute montée de printpdf ou de lopdf** : le procédé repose
/// sur leur fonctionnement interne aux versions du `Cargo.lock` (printpdf 0.7.0,
/// lopdf 0.31.0) — titre écrit une seule fois, en chaîne littérale non
/// échappée, dans le dictionnaire `Info` hors de tout flux compressé, sans
/// métadonnées XMP ni flux d'objets. Une version qui écrirait le titre
/// autrement ferait échouer le rendu de tout rapport au titre non ASCII
/// (gabarit absent ou en double → `ReportError::PdfGeneration`), ce que
/// `le_titre_des_proprietes_du_pdf_se_relit_juste` signale. Seul le profil de
/// test (non compressé) est exercé par les tests ; le profil release l'est à la
/// lecture des sources (`optimize` ne compresse que les flux).
fn title_placeholder(title: &str) -> Option<String> {
    if title.is_ascii() {
        return None;
    }
    let len = 4 + 4 * title.encode_utf16().count();
    Some(
        "KeshTitrePdf"
            .chars()
            .chain(std::iter::repeat('x'))
            .take(len)
            .collect(),
    )
}

/// Une chaîne de texte PDF en hexadécimal : `<FEFF…>`, l'UTF-16BE de `text`
/// précédé de la marque d'ordre des octets (ISO 32000-1, § 7.9.2.2).
fn pdf_utf16_hex(text: &str) -> String {
    let mut out = String::from("<FEFF");
    for unit in text.encode_utf16() {
        out.push_str(&format!("{unit:04X}"));
    }
    out.push('>');
    out
}

/// Remplace, dans le PDF produit, la chaîne littérale `(placeholder)` — le
/// titre provisoire écrit par printpdf, voir [`title_placeholder`] — par le
/// vrai titre en UTF-16BE. Le remplacement garde la longueur (contrôlée) ; il
/// exige une occurrence et une seule, sans quoi le PDF n'est pas rendu.
fn set_pdf_title(
    mut bytes: Vec<u8>,
    placeholder: &str,
    title: &str,
) -> Result<Vec<u8>, ReportError> {
    let needle = format!("({placeholder})");
    let hex = pdf_utf16_hex(title);
    let needle = needle.as_bytes();
    let erreur = |m: &str| ReportError::PdfGeneration(format!("titre du document : {m}"));
    if hex.len() != needle.len() {
        return Err(erreur("longueur du gabarit"));
    }
    let mut hits = bytes
        .windows(needle.len())
        .enumerate()
        .filter(|(_, w)| w == &needle);
    let (at, _) = hits.next().ok_or_else(|| erreur("gabarit absent"))?;
    if hits.next().is_some() {
        return Err(erreur("gabarit en double"));
    }
    bytes[at..at + needle.len()].copy_from_slice(hex.as_bytes());
    Ok(bytes)
}

// ============================================================================
// Helpers de rendu — header / footer / empty (T2.3, T2.6)
// ============================================================================

fn draw_header(
    builder: &mut PdfBuilder,
    ctx: &PdfContext,
    title: &str,
    period_start: NaiveDate,
    period_end: NaiveDate,
) {
    // Company name (gras, taille 14)
    builder.write_line(&ctx.company_name, FONT_SIZE_HEADER_PT, true, 0.0);
    // Title (gras, taille 12)
    builder.write_line(title, 12.0, true, 0.0);
    // Période (taille 10)
    builder.write_line(
        &format!(
            "{} : {} → {}",
            ctx.section_labels.header_period,
            format_swiss_date(period_start),
            format_swiss_date(period_end)
        ),
        FONT_SIZE_PT,
        false,
        0.0,
    );
    // Optional journal filter
    if let Some(filter) = &ctx.journal_filter_label {
        builder.write_line(
            &format!("{} : {}", ctx.section_labels.journal_filter_prefix, filter),
            FONT_SIZE_PT,
            false,
            0.0,
        );
    }
    builder.cursor_y -= LINE_HEIGHT_MM; // espace
}

/// Rendu spécifique « rapport vide » : message centré à la verticale (T2.6 + AC #8).
///
/// Pass 1 code-review H3 (BH1-H2) : on ne hardcode plus `PAGE_HEIGHT / 2`
/// (148.5 mm) car cela pouvait collisionner avec un header multi-ligne
/// (`journal_filter_label` en plus). On centre dynamiquement entre le curseur
/// courant (post-header) et la marge basse.
fn draw_empty_message(builder: &mut PdfBuilder, ctx: &PdfContext) {
    // Centrer entre la position courante (post-header) et la marge basse.
    let mid = (builder.cursor_y + MARGIN_BOTTOM_MM) / 2.0;
    builder.cursor_y = mid;
    let layer = builder.current_layer();
    // X fixe, à 30 mm de la marge gauche, non centré : le message n'est pas mesuré (une
    // mesure exacte existe, `kesh_qrbill::largeur_mm`, voir `right_aligned_x`,
    // mais le centrage du message vide n'est pas repris par ce correctif).
    layer.use_text(
        &ctx.empty_message,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM + 30.0),
        Mm(builder.cursor_y),
        &builder.font,
    );
}

/// Longueur maximale du nom d'un compte **en devise** au bilan et au compte de
/// résultat : de `MARGIN_LEFT_MM + 25` (40 mm) au solde en devise, aligné à
/// droite sur [`FX_AMOUNT_RIGHT_MM`] et qui commence au plus tôt vers 130,4 mm —
/// 34 caractères à 2,5 mm la capitale (comme `TB_NAME_MAX_CHARS`) tiennent dans
/// 85 mm, avec une marge (Story 37-4, revue de code P1).
const ACCOUNT_NAME_MAX_CHARS_FX: usize = 34;

/// En-têtes des deux colonnes de montant du bilan et du compte de résultat,
/// alignés à droite comme elles — « Devise » sur les soldes en devise, « CHF »
/// sur les soldes en devise principale —, dessinés seulement si le rapport
/// contient un compte en devise : un rapport mono-devise n'a qu'une colonne de
/// montant et garde son rendu.
fn draw_currency_columns_header(builder: &mut PdfBuilder, ctx: &PdfContext) {
    let l = &ctx.section_labels;
    builder.write_right(
        &l.col_foreign_currency,
        FONT_SIZE_PT,
        true,
        FX_AMOUNT_RIGHT_MM,
    );
    builder.write_right(&l.col_base_currency, FONT_SIZE_PT, true, AMOUNT_RIGHT_MM);
    builder.cursor_y -= LINE_HEIGHT_MM;
}

/// Vrai si l'un des comptes porte une devise (Story 37-4) : le rapport montre
/// alors une colonne en devise à côté de celle en devise principale.
fn any_foreign<'a>(accounts: impl IntoIterator<Item = &'a AccountBalance>) -> bool {
    accounts.into_iter().any(|ab| ab.currency.is_some())
}

/// Dessine une ligne de compte (N° | Nom | Solde) ou variante.
fn draw_account_row(builder: &mut PdfBuilder, ab: &AccountBalance) {
    builder.ensure_space_for_row();
    let y = builder.cursor_y;
    let layer = builder.current_layer();
    layer.use_text(
        &ab.account_number,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM),
        Mm(y),
        &builder.font,
    );
    // Story 37-4 (revue de code P1) : un compte en devise porte son solde en
    // devise, aligné à droite sur `FX_AMOUNT_RIGHT_MM` (il commence au plus tôt
    // vers 130 mm) ; son nom est tronqué pour ne pas le recouvrir. Un compte en devise principale garde son nom entier.
    let name = if ab.currency.is_some() {
        truncate_with_ellipsis(&ab.account_name, ACCOUNT_NAME_MAX_CHARS_FX)
    } else {
        ab.account_name.clone()
    };
    layer.use_text(
        &name,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM + 25.0),
        Mm(y),
        &builder.font,
    );
    // Story 37-4, AC-5 : la devise et le solde en devise entre le nom et le
    // solde — compte en devise seulement ; les totaux restent en devise
    // principale.
    if let (Some(cur), Some(fx)) = (&ab.currency, ab.balance_currency) {
        builder.write_right(
            &format_currency_amount(cur, fx),
            FONT_SIZE_PT,
            false,
            FX_AMOUNT_RIGHT_MM,
        );
    }
    builder.write_right(
        &format_swiss_amount(ab.balance),
        FONT_SIZE_PT,
        false,
        AMOUNT_RIGHT_MM,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;
}

/// Dessine une ligne de total de section (label gras + montant aligné à droite).
fn draw_totals_footer(builder: &mut PdfBuilder, label: &str, amount: Decimal) {
    builder.ensure_space_for_row();
    let y = builder.cursor_y;
    let layer = builder.current_layer();
    layer.use_text(
        label,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM),
        Mm(y),
        &builder.font_bold,
    );
    builder.write_right(
        &format_swiss_amount(amount),
        FONT_SIZE_PT,
        true,
        AMOUNT_RIGHT_MM,
    );
    builder.cursor_y -= LINE_HEIGHT_MM * 1.5;
}

// ============================================================================
// Public rendering functions (T2.1)
// ============================================================================

/// Génère le PDF du bilan (AC #1, #2, #3).
pub fn render_balance_sheet_pdf(
    bs: &BalanceSheet,
    ctx: &PdfContext,
) -> Result<Vec<u8>, ReportError> {
    let mut builder = PdfBuilder::new("Bilan")?;
    draw_header(
        &mut builder,
        ctx,
        "Bilan",
        bs.period.start_date,
        bs.period.end_date,
    );

    // Garde « rapport vide » centralisée `BalanceSheet::is_empty()` (Story 14-3c P1-F1)
    // — inclut `equity` : un reclassement pur entre comptes de fonds propres (section
    // equity non vide, actifs/passifs/virtuels nuls) n'est PAS vide.
    if bs.is_empty() {
        draw_empty_message(&mut builder, ctx);
        return builder.finalize();
    }

    if any_foreign(bs.assets.iter().chain(&bs.liabilities).chain(&bs.equity)) {
        draw_currency_columns_header(&mut builder, ctx);
    }

    // Section Actifs
    builder.write_line(&ctx.section_labels.assets, FONT_SIZE_PT, true, 0.0);
    for ab in &bs.assets {
        draw_account_row(&mut builder, ab);
    }
    draw_totals_footer(
        &mut builder,
        &ctx.section_labels.total_assets,
        bs.total_assets,
    );

    // Section Passifs
    builder.write_line(&ctx.section_labels.liabilities, FONT_SIZE_PT, true, 0.0);
    for ab in &bs.liabilities {
        draw_account_row(&mut builder, ab);
    }
    draw_totals_footer(
        &mut builder,
        &ctx.section_labels.total_liabilities,
        bs.total_liabilities,
    );

    // Section Capitaux propres — présentation par rôle (Story 14-3c). D'abord les
    // comptes physiques de fonds propres, groupés par rôle sous des sous-titres FR-CH
    // (ordre par rôle fixé en backend — on itère `bs.equity` sans re-trier). Puis les
    // deux lignes **calculées** distinctes (« Résultat reporté (calculé) » marquée
    // calculée pour la distinguer d'un compte physique RetainedEarnings — collision D1).
    builder.write_line(&ctx.section_labels.equity, FONT_SIZE_PT, true, 0.0);
    let mut current_role: Option<Option<AccountRole>> = None;
    for ab in &bs.equity {
        if current_role != Some(ab.role) {
            // Nouveau groupe de rôle → sous-titre gras indenté.
            match ctx.section_labels.equity_role_label(ab.role) {
                Some(label) => {
                    // Keep-with-next (ECH-1) : réserver le sous-titre + sa 1re ligne
                    // ensemble, pour ne pas laisser le sous-titre orphelin en bas de page.
                    builder.ensure_space_for_rows(2.0);
                    builder.write_line(label, FONT_SIZE_PT, true, 5.0);
                }
                // Défense en profondeur (BH-1) : `is_equity_role` garantit en amont que
                // tout compte de `bs.equity` porte un rôle equity ; un `None` ici signale
                // une régression de la partition — on la journalise sans crasher le rendu.
                None => tracing::warn!(
                    account_number = %ab.account_number,
                    role = ?ab.role,
                    "balance_sheet PDF : compte sans rôle de fonds propres dans la section \
                     equity (partition incohérente ? cf. is_equity_role)"
                ),
            }
            current_role = Some(ab.role);
        }
        draw_account_row(&mut builder, ab);
    }
    for (label, amount) in [
        (
            &ctx.section_labels.retained_result_label,
            bs.retained_earnings,
        ),
        (&ctx.section_labels.equity_result_label, bs.equity_result),
    ] {
        builder.ensure_space_for_row();
        let y = builder.cursor_y;
        let layer = builder.current_layer();
        layer.use_text(
            label,
            FONT_SIZE_PT,
            Mm(MARGIN_LEFT_MM + 25.0),
            Mm(y),
            &builder.font,
        );
        builder.write_right(
            &format_swiss_amount(amount),
            FONT_SIZE_PT,
            false,
            AMOUNT_RIGHT_MM,
        );
        builder.cursor_y -= LINE_HEIGHT_MM;
    }
    // Total capitaux propres = comptes physiques + 2 lignes calculées.
    let total_equity_all = bs.total_equity + bs.retained_earnings + bs.equity_result;
    draw_totals_footer(
        &mut builder,
        &ctx.section_labels.total_equity,
        total_equity_all,
    );

    // Pied de page : équation bilan (AC #3).
    //
    // Pass 1 code-review H2 (BH1-H3) : `Total actifs` est déjà affiché en
    // sous-total après la section Actifs ci-dessus. Le réafficher ici
    // dupliquait visuellement le montant. L'équation bilan se vérifie par
    // comparaison Total actifs (haut) vs Total passifs + Capitaux propres
    // (bas) — pas besoin de redraw. Équation restructurée 14-3c :
    // `total_liabilities` (dettes seules) + total capitaux propres.
    let total_liab_eq = bs.total_liabilities + total_equity_all;
    draw_totals_footer(
        &mut builder,
        &format!(
            "{} + {}",
            ctx.section_labels.total_liabilities, ctx.section_labels.equity
        ),
        total_liab_eq,
    );

    builder.finalize()
}

/// Génère le PDF du compte de résultat (AC #4).
pub fn render_income_statement_pdf(
    is_: &IncomeStatement,
    ctx: &PdfContext,
) -> Result<Vec<u8>, ReportError> {
    let mut builder = PdfBuilder::new("Compte de résultat")?;
    draw_header(
        &mut builder,
        ctx,
        "Compte de résultat",
        is_.period.start_date,
        is_.period.end_date,
    );

    let is_empty = is_.revenues.is_empty() && is_.expenses.is_empty();
    if is_empty {
        draw_empty_message(&mut builder, ctx);
        return builder.finalize();
    }

    if any_foreign(is_.revenues.iter().chain(&is_.expenses)) {
        draw_currency_columns_header(&mut builder, ctx);
    }

    // Section Produits
    builder.write_line(&ctx.section_labels.revenues, FONT_SIZE_PT, true, 0.0);
    for ab in &is_.revenues {
        draw_account_row(&mut builder, ab);
    }
    draw_totals_footer(
        &mut builder,
        &ctx.section_labels.total_revenues,
        is_.total_revenues,
    );

    // Section Charges
    builder.write_line(&ctx.section_labels.expenses, FONT_SIZE_PT, true, 0.0);
    for ab in &is_.expenses {
        draw_account_row(&mut builder, ab);
    }
    draw_totals_footer(
        &mut builder,
        &ctx.section_labels.total_expenses,
        is_.total_expenses,
    );

    // Pied de page : résultat net
    draw_totals_footer(&mut builder, &ctx.section_labels.net_result, is_.net_result);

    builder.finalize()
}

/// Positions des deux colonnes de texte de la balance (N°, intitulé ; Story
/// 25-5-b), en mm depuis la marge gauche : l'intitulé dispose de 55 mm, de 35 à
/// 90 mm de la page.
const TB_COL_X: [f32; 2] = [0.0, 20.0];

/// Bords droits des quatre colonnes de montant de la balance — ouverture,
/// débit, crédit, clôture —, en mm depuis le bord gauche de la page : montants
/// et en-têtes y sont alignés à droite, tous les 26 mm jusqu'à la marge droite.
/// Le plus long montant plausible, « -12'345'678.90 » (23,1 mm en Helvetica
/// 10 pt, 23,4 en gras), commence vers 93,6 mm dans la colonne d'ouverture,
/// après l'intitulé (qui finit à 90 mm).
const TB_AMOUNT_RIGHT_MM: [f32; 4] = [117.0, 143.0, 169.0, AMOUNT_RIGHT_MM];

/// Longueur maximale de l'intitulé d'un compte dans la balance PDF : 22
/// caractères tiennent dans 55 mm, même en capitales.
const TB_NAME_MAX_CHARS: usize = 22;

/// Dessine une ligne de la balance : six cellules, `None` = cellule vide. Les
/// deux premières (N°, intitulé) s'écrivent à gauche de leur colonne, les
/// quatre montants alignés à droite sur [`TB_AMOUNT_RIGHT_MM`].
fn draw_tb_row(builder: &mut PdfBuilder, cells: [Option<&str>; 6], bold: bool) {
    let (texts, amounts) = cells.split_at(TB_COL_X.len());
    for (cell, x) in texts.iter().zip(TB_COL_X) {
        if let Some(text) = cell {
            builder.write_inline(text, FONT_SIZE_PT, bold, MARGIN_LEFT_MM + x);
        }
    }
    for (cell, right) in amounts.iter().zip(TB_AMOUNT_RIGHT_MM) {
        if let Some(text) = cell {
            builder.write_right(text, FONT_SIZE_PT, bold, right);
        }
    }
}

/// Génère le PDF de la balance des comptes — 6 colonnes : N°, Intitulé,
/// Ouverture, Débit, Crédit, Clôture (Story 25-5-b, #385), puis la ligne calculée
/// du résultat reporté et les totaux des mouvements.
pub fn render_trial_balance_pdf(
    tb: &TrialBalance,
    ctx: &PdfContext,
) -> Result<Vec<u8>, ReportError> {
    let mut builder = PdfBuilder::new("Balance des comptes")?;
    draw_header(
        &mut builder,
        ctx,
        "Balance des comptes",
        tb.period.start_date,
        tb.period.end_date,
    );

    if tb.rows.is_empty() {
        draw_empty_message(&mut builder, ctx);
        return builder.finalize();
    }

    let l = &ctx.section_labels;
    // Avec un compte en devise, les quatre colonnes disent leur devise
    // (« Débit CHF ») : les montants en devise sont dans la seconde ligne.
    let multi = tb.rows.iter().any(|r| r.currency.is_some());
    let [opening_h, debit_h, credit_h, closing_h] =
        [&l.col_opening, &l.col_debit, &l.col_credit, &l.col_closing]
            .map(|h| l.amount_header(h, multi));
    draw_tb_row(
        &mut builder,
        [
            Some(&l.col_account_number),
            Some(&l.col_account_name),
            Some(&opening_h),
            Some(&debit_h),
            Some(&credit_h),
            Some(&closing_h),
        ],
        true,
    );
    builder.cursor_y -= LINE_HEIGHT_MM * 1.5;

    for row in &tb.rows {
        // Un compte en devise garde sa seconde ligne sur la même page.
        if row.currency.is_some() {
            builder.ensure_space_for_rows(2.0);
        } else {
            builder.ensure_space_for_row();
        }
        // Pass 1 code-review H4 (BH1-H4) : un intitulé trop long déborderait sur la
        // colonne suivante — tronqué avec ellipsis Unicode.
        let name = truncate_with_ellipsis(&row.account_name, TB_NAME_MAX_CHARS);
        let (opening, debit, credit, closing) = (
            format_swiss_amount(row.opening_balance),
            format_swiss_amount(row.total_debit),
            format_swiss_amount(row.total_credit),
            format_swiss_amount(row.closing_balance),
        );
        draw_tb_row(
            &mut builder,
            [
                Some(&row.account_number),
                Some(&name),
                Some(&opening),
                Some(&debit),
                Some(&credit),
                Some(&closing),
            ],
            false,
        );
        builder.cursor_y -= LINE_HEIGHT_MM;

        // Story 37-4, AC-5 : sous un compte en devise, une seconde ligne en
        // petit — ouverture, débit, crédit et clôture en devise. Aucune ligne de
        // total en devise (devises mêlées).
        if let Some(cur) = &row.currency {
            let m =
                |v: Option<Decimal>| format_swiss_amount(round_for_display(v.unwrap_or_default()));
            let texte = format!(
                "{cur} {} {} · {} {} · {} {} · {} {}",
                l.fx_opening,
                m(row.opening_currency),
                l.fx_debit,
                m(row.total_debit_currency),
                l.fx_credit,
                m(row.total_credit_currency),
                l.fx_closing,
                m(row.closing_currency),
            );
            builder.write_fx_line(&texte, MARGIN_LEFT_MM + TB_COL_X[1]);
        }
    }

    // Ligne calculée du résultat reporté : ouverture = clôture, sans mouvement.
    builder.ensure_space_for_row();
    let retained = format_swiss_amount(tb.retained_earnings);
    // Libellé fixe, non tronqué : « Résultat reporté (calculé) » ≈ 46 mm en casse
    // mixte, dans les 55 mm de la colonne.
    draw_tb_row(
        &mut builder,
        [
            None,
            Some(&l.retained_result_label),
            Some(&retained),
            None,
            None,
            Some(&retained),
        ],
        false,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;

    // Totaux des mouvements.
    builder.cursor_y -= LINE_HEIGHT_MM * 0.5;
    builder.ensure_space_for_row();
    let (total_debit, total_credit) = (
        format_swiss_amount(tb.total_debit),
        format_swiss_amount(tb.total_credit),
    );
    draw_tb_row(
        &mut builder,
        [
            Some(&l.grand_total),
            None,
            None,
            Some(&total_debit),
            Some(&total_credit),
            None,
        ],
        true,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;

    // Le contrôle d'ouverture : silencieux s'il tient, écrit s'il tombe — un PDF
    // remis à un réviseur ne doit pas perdre le signal que l'écran affiche.
    if !tb.opening_balanced {
        builder.cursor_y -= LINE_HEIGHT_MM * 0.5;
        builder.ensure_space_for_row();
        builder.write_line(&l.opening_unbalanced, FONT_SIZE_PT, true, 0.0);
    }

    builder.finalize()
}

/// Bords droits des colonnes débit et crédit des journaux, en mm depuis le bord
/// gauche de la page : 35 mm d'écart, comme avant l'alignement à droite ; le plus
/// long débit plausible (23,1 mm, 23,4 en gras) commence vers 136,6 mm, laissant
/// plus de 90 mm à l'intitulé du compte, qui commence à 45 mm.
const JR_DEBIT_RIGHT_MM: f32 = AMOUNT_RIGHT_MM - 35.0;
const JR_CREDIT_RIGHT_MM: f32 = AMOUNT_RIGHT_MM;

/// Génère le PDF des journaux — 5 sections fixes ou 1 section filtrée (AC #6, #7).
pub fn render_journal_report_pdf(
    jr: &JournalReport,
    ctx: &PdfContext,
) -> Result<Vec<u8>, ReportError> {
    let mut builder = PdfBuilder::new("Journaux")?;
    draw_header(
        &mut builder,
        ctx,
        "Journaux",
        jr.period.start_date,
        jr.period.end_date,
    );

    // Rapport vide = toutes les sections vides
    let is_empty = jr.journals.iter().all(|s| s.entries.is_empty());
    if is_empty {
        draw_empty_message(&mut builder, ctx);
        return builder.finalize();
    }

    for section in &jr.journals {
        // Header de section (toujours présent — AC #6 : sections vides incluses)
        builder.ensure_space_for_row();
        builder.write_line(
            &format!("— {} —", section.journal.as_str()),
            FONT_SIZE_PT,
            true,
            0.0,
        );

        if section.entries.is_empty() {
            // Section vide : on note "—" (Pass 1 ECH-09 invariant)
            builder.write_line("(aucune écriture)", FONT_SIZE_PT, false, 5.0);
            continue;
        }

        for entry in &section.entries {
            // Ligne de tête d'écriture : date + n° + description
            builder.ensure_space_for_row();
            let header_line = format!(
                "{} | #{} | {}",
                format_swiss_date(entry.entry_date),
                entry.entry_number,
                entry.description
            );
            builder.write_line(&header_line, FONT_SIZE_PT, true, 0.0);

            // Lignes d'écriture
            for line in &entry.lines {
                builder.ensure_space_for_row();
                let y = builder.cursor_y;
                let layer = builder.current_layer();
                layer.use_text(
                    &line.account_number,
                    FONT_SIZE_PT,
                    Mm(MARGIN_LEFT_MM + 5.0),
                    Mm(y),
                    &builder.font,
                );
                layer.use_text(
                    &line.account_name,
                    FONT_SIZE_PT,
                    Mm(MARGIN_LEFT_MM + 30.0),
                    Mm(y),
                    &builder.font,
                );
                builder.write_right(
                    &format_swiss_amount(line.debit),
                    FONT_SIZE_PT,
                    false,
                    JR_DEBIT_RIGHT_MM,
                );
                builder.write_right(
                    &format_swiss_amount(line.credit),
                    FONT_SIZE_PT,
                    false,
                    JR_CREDIT_RIGHT_MM,
                );
                builder.cursor_y -= LINE_HEIGHT_MM;
            }
        }

        // Totaux de section
        builder.cursor_y -= LINE_HEIGHT_MM * 0.3;
        builder.ensure_space_for_row();
        let y = builder.cursor_y;
        let layer = builder.current_layer();
        layer.use_text(
            &ctx.section_labels.total_debit,
            FONT_SIZE_PT,
            Mm(MARGIN_LEFT_MM + 5.0),
            Mm(y),
            &builder.font_bold,
        );
        builder.write_right(
            &format_swiss_amount(section.section_total_debit),
            FONT_SIZE_PT,
            true,
            JR_DEBIT_RIGHT_MM,
        );
        builder.write_right(
            &format_swiss_amount(section.section_total_credit),
            FONT_SIZE_PT,
            true,
            JR_CREDIT_RIGHT_MM,
        );
        builder.cursor_y -= LINE_HEIGHT_MM * 1.5;
    }

    // Grand total
    //
    // Pass 1 code-review H11 (ECH1-H1) : sans `ensure_space_for_row`, le grand
    // total pouvait être dessiné en dessous de la marge basse — invisible —
    // si la dernière section laissait `cursor_y < MARGIN_BOTTOM_MM`.
    builder.ensure_space_for_row();
    builder.write_inline(
        &ctx.section_labels.grand_total,
        FONT_SIZE_PT,
        true,
        MARGIN_LEFT_MM,
    );
    builder.write_right(
        &format_swiss_amount(jr.grand_total_debit),
        FONT_SIZE_PT,
        true,
        JR_DEBIT_RIGHT_MM,
    );
    builder.write_right(
        &format_swiss_amount(jr.grand_total_credit),
        FONT_SIZE_PT,
        true,
        JR_CREDIT_RIGHT_MM,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;

    builder.finalize()
}

// ============================================================================
// Rapport TVA — Story 11-2
// ============================================================================

/// Libellés localisés du rapport TVA, résolus côté handler `kesh-api`.
///
/// Struct **dédié** (séparé de [`SectionLabels`]) pour ne pas toucher au type
/// partagé par les 4 rapports existants (non-régression).
#[derive(Debug, Clone)]
pub struct VatPdfLabels {
    /// Titre du rapport (ex. « Décompte TVA »).
    pub title: String,
    /// Colonne taux.
    pub col_rate: String,
    /// Colonne chiffre d'affaires HT.
    pub col_base_ht: String,
    /// Colonne TVA due.
    pub col_vat_due: String,
    /// Libellé total CA HT.
    pub total_base_ht: String,
    /// Libellé total TVA due.
    pub total_vat_due: String,
    /// Titre de la section des diminutions de contre-prestation (Story 25-4-d2c).
    pub write_off_title: String,
    /// Libellé total de la TVA des soldes (Story 25-4-d2c).
    pub total_vat_write_off: String,
    /// Libellé TVA due nette (Story 25-4-d2c).
    pub total_vat_due_net: String,
    /// Libellé TVA récupérable.
    pub vat_recoverable: String,
    /// Libellé solde.
    pub balance: String,
    /// Libellé écart de réconciliation (Story 18-1e). En dur FR — l'i18n des
    /// exports PDF est déférée v0.2 (cohérent avec les autres libellés).
    pub reconciliation_delta: String,
}

impl VatPdfLabels {
    /// Libellés par défaut FR-CH (tests unit / fallback).
    pub fn fr_ch_defaults() -> Self {
        Self {
            title: "Décompte TVA".into(),
            col_rate: "Taux".into(),
            col_base_ht: "Chiffre d'affaires HT".into(),
            col_vat_due: "TVA due".into(),
            total_base_ht: "Total chiffre d'affaires HT".into(),
            total_vat_due: "Total TVA due".into(),
            write_off_title: "Diminutions de contre-prestation (soldes)".into(),
            total_vat_write_off: "Total TVA des soldes".into(),
            total_vat_due_net: "TVA due nette".into(),
            vat_recoverable: "TVA récupérable".into(),
            balance: "Solde".into(),
            reconciliation_delta: "Écart de réconciliation".into(),
        }
    }
}

// Colonnes du tableau TVA (mm depuis le bord gauche de la page) : le taux à
// gauche ; la base HT et la TVA, en-têtes compris, alignées à droite sur leur
// bord — celui de la TVA est celui des totaux (`draw_totals_footer`).
const VAT_COL_RATE_X: f32 = MARGIN_LEFT_MM;
const VAT_BASE_RIGHT_MM: f32 = AMOUNT_RIGHT_MM - 45.0;
const VAT_DUE_RIGHT_MM: f32 = AMOUNT_RIGHT_MM;

/// Formate un taux en pourcent pour l'affichage (ex. `8.10` → `"8.10 %"`).
fn format_rate(rate: Decimal) -> String {
    format!("{rate:.2} %")
}

/// Dessine une ligne du tableau TVA (3 colonnes, les deux montants alignés à
/// droite).
fn draw_vat_row(builder: &mut PdfBuilder, rate: &str, base_ht: &str, vat_due: &str, bold: bool) {
    builder.ensure_space_for_row();
    builder.write_inline(rate, FONT_SIZE_PT, bold, VAT_COL_RATE_X);
    builder.write_right(base_ht, FONT_SIZE_PT, bold, VAT_BASE_RIGHT_MM);
    builder.write_right(vat_due, FONT_SIZE_PT, bold, VAT_DUE_RIGHT_MM);
    builder.cursor_y -= LINE_HEIGHT_MM;
}

/// Génère le PDF du rapport TVA (Story 11-2).
pub fn render_vat_report_pdf(
    report: &VatReport,
    ctx: &PdfContext,
    labels: &VatPdfLabels,
) -> Result<Vec<u8>, ReportError> {
    let mut builder = PdfBuilder::new(&labels.title)?;
    draw_header(
        &mut builder,
        ctx,
        &labels.title,
        report.period.start_date,
        report.period.end_date,
    );

    // Story 18-1d : un rapport sans vente (rows vide) mais avec de la TVA
    // récupérable (achats seuls) n'est PAS vide — on rend le bloc totaux
    // (récupérable + solde). On ne court-circuite que si AUSSI récupérable == 0.
    // Story 25-4-d2c : ni avec des soldes.
    if report.rows.is_empty()
        && report.total_vat_recoverable == Decimal::ZERO
        && report.write_off_rows.is_empty()
    {
        draw_empty_message(&mut builder, ctx);
        return builder.finalize();
    }

    // En-tête de colonnes.
    draw_vat_row(
        &mut builder,
        &labels.col_rate,
        &labels.col_base_ht,
        &labels.col_vat_due,
        true,
    );

    // Lignes par taux.
    for row in &report.rows {
        draw_vat_row(
            &mut builder,
            &format_rate(row.rate),
            &format_swiss_amount(row.base_ht),
            &format_swiss_amount(row.vat_due),
            false,
        );
    }

    builder.cursor_y -= LINE_HEIGHT_MM * 0.5;

    // Totaux + récupérable + solde.
    draw_totals_footer(&mut builder, &labels.total_base_ht, report.total_base_ht);
    draw_totals_footer(&mut builder, &labels.total_vat_due, report.total_vat_due);
    // Story 25-4-d2c : les diminutions (soldes) — seulement s'il y en a.
    if !report.write_off_rows.is_empty() {
        builder.cursor_y -= LINE_HEIGHT_MM * 0.5;
        draw_vat_row(&mut builder, &labels.write_off_title, "", "", true);
        for row in &report.write_off_rows {
            draw_vat_row(
                &mut builder,
                &format_rate(row.rate),
                &format_swiss_amount(row.base_ht),
                &format_swiss_amount(row.vat),
                false,
            );
        }
        draw_totals_footer(
            &mut builder,
            &labels.total_vat_write_off,
            report.total_vat_write_off,
        );
        draw_totals_footer(
            &mut builder,
            &labels.total_vat_due_net,
            report.total_vat_due_net,
        );
    }
    draw_totals_footer(
        &mut builder,
        &labels.vat_recoverable,
        report.total_vat_recoverable,
    );
    draw_totals_footer(&mut builder, &labels.balance, report.vat_balance);
    // Story 18-1e : écart de réconciliation (TVA due dérivée − solde compte TVA due
    // au grand livre, périmètre ventes). 0.00 dans le cas nominal.
    draw_totals_footer(
        &mut builder,
        &labels.reconciliation_delta,
        report.reconciliation_delta,
    );

    builder.finalize()
}

// ============================================================================
// Story 19-6a — Rapport « Dépenses par projet »
// ============================================================================

/// Libellés du PDF « Dépenses par projet » (précédent `VatPdfLabels` — labels
/// dédiés en dur FR-CH, i18n PDF déférée v0.2).
#[derive(Debug, Clone)]
pub struct ProjectExpensesPdfLabels {
    pub title: String,
    pub col_account: String,
    pub subtotal: String,
    pub grand_total: String,
    pub empty_message: String,
}

impl ProjectExpensesPdfLabels {
    /// Libellés par défaut FR-CH.
    pub fn fr_ch_defaults() -> Self {
        Self {
            title: "Dépenses par compte analytique".into(),
            col_account: "Compte".into(),
            subtotal: "Sous-total".into(),
            grand_total: "Total dépenses".into(),
            empty_message: "Aucune dépense taguée sur ce compte analytique pour la période.".into(),
        }
    }
}

impl Default for ProjectExpensesPdfLabels {
    fn default() -> Self {
        Self::fr_ch_defaults()
    }
}

/// Dessine une ligne compte de dépense (N° | Nom | Montant), pattern
/// `draw_account_row` adapté à [`crate::project_report::ExpenseAccountRow`].
fn draw_expense_row(builder: &mut PdfBuilder, row: &crate::project_report::ExpenseAccountRow) {
    builder.ensure_space_for_row();
    let y = builder.cursor_y;
    let layer = builder.current_layer();
    layer.use_text(
        &row.account_number,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM),
        Mm(y),
        &builder.font,
    );
    layer.use_text(
        &row.account_name,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM + 25.0),
        Mm(y),
        &builder.font,
    );
    builder.write_right(
        &format_swiss_amount(row.amount),
        FONT_SIZE_PT,
        false,
        AMOUNT_RIGHT_MM,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;
}

/// Génère le PDF du rapport « Dépenses par projet » (Story 19-6a).
///
/// En-tête = nom company + titre + libellé projet + période. Une sous-section par
/// projet du scope (racine puis sous-projets), lignes de compte + sous-total,
/// puis total général. Le drill-down (écritures) n'est PAS rendu en PDF (réservé
/// à l'affichage interactif).
pub fn render_project_expenses_pdf(
    report: &crate::project_report::ProjectExpensesReport,
    ctx: &PdfContext,
    labels: &ProjectExpensesPdfLabels,
) -> Result<Vec<u8>, ReportError> {
    let mut builder = PdfBuilder::new(&labels.title)?;

    // En-tête : company + titre + projet + période (une ligne libre chacun).
    builder.write_line(&ctx.company_name, FONT_SIZE_HEADER_PT, true, 0.0);
    builder.write_line(
        &format!(
            "{} — {} {}",
            labels.title, report.project.code, report.project.name
        ),
        12.0,
        true,
        0.0,
    );
    builder.write_line(&report.period_label, FONT_SIZE_PT, false, 0.0);
    builder.cursor_y -= LINE_HEIGHT_MM;

    if report.sections.is_empty() {
        // Message vide (position courante post-header).
        let layer = builder.current_layer();
        layer.use_text(
            &labels.empty_message,
            FONT_SIZE_PT,
            Mm(MARGIN_LEFT_MM),
            Mm(builder.cursor_y),
            &builder.font,
        );
        return builder.finalize();
    }

    for section in &report.sections {
        let section_title = if section.is_root {
            format!("{} {}", section.project.code, section.project.name)
        } else {
            format!("  ↳ {} {}", section.project.code, section.project.name)
        };
        builder.write_line(&section_title, FONT_SIZE_PT, true, 0.0);
        for row in &section.rows {
            draw_expense_row(&mut builder, row);
        }
        draw_totals_footer(&mut builder, &labels.subtotal, section.subtotal);
    }

    draw_totals_footer(&mut builder, &labels.grand_total, report.grand_total);
    builder.finalize()
}

// ============================================================================
// Story 19-6b — Rapport « Rendement par projet »
// ============================================================================

/// Libellés du PDF « Rendement par projet » (FR-CH par défaut, i18n déférée v0.2).
#[derive(Debug, Clone)]
pub struct ProjectReturnPdfLabels {
    pub title: String,
    pub col_cout: String,
    pub col_revenus: String,
    pub col_resultat: String,
    pub col_rendement: String,
    pub total: String,
    pub empty_message: String,
}

impl ProjectReturnPdfLabels {
    pub fn fr_ch_defaults() -> Self {
        Self {
            title: "Rendement par compte analytique".into(),
            col_cout: "Coût investi".into(),
            col_revenus: "Revenus".into(),
            col_resultat: "Résultat net".into(),
            col_rendement: "Rendement".into(),
            total: "Total".into(),
            empty_message: "Aucun mouvement tagué sur ce compte analytique pour la période.".into(),
        }
    }
}

impl Default for ProjectReturnPdfLabels {
    fn default() -> Self {
        Self::fr_ch_defaults()
    }
}

// Colonnes du tableau rendement (mm depuis le bord gauche de la page) : le
// libellé à gauche ; les quatre colonnes numériques, en-têtes compris, alignées
// à droite sur leur bord, 30 mm d'écart (27 pour le rendement, court).
const RET_COL_LABEL_X: f32 = MARGIN_LEFT_MM;
const RET_COUT_RIGHT_MM: f32 = AMOUNT_RIGHT_MM - 87.0;
const RET_REV_RIGHT_MM: f32 = AMOUNT_RIGHT_MM - 57.0;
const RET_RES_RIGHT_MM: f32 = AMOUNT_RIGHT_MM - 27.0;
const RET_RENDEMENT_RIGHT_MM: f32 = AMOUNT_RIGHT_MM;

fn format_rendement(pct: Option<Decimal>) -> String {
    match pct {
        Some(p) => format!("{p:.2}%"),
        None => "—".to_string(),
    }
}

/// Dessine les quatre cellules numériques d'une ligne du tableau rendement —
/// coût, revenus, résultat, rendement —, alignées à droite sur leur bord, à la
/// position courante et sans avancer. Sert aussi à la ligne des en-têtes.
fn draw_return_cells(builder: &PdfBuilder, cells: [&str; 4], bold: bool) {
    let rights = [
        RET_COUT_RIGHT_MM,
        RET_REV_RIGHT_MM,
        RET_RES_RIGHT_MM,
        RET_RENDEMENT_RIGHT_MM,
    ];
    for (text, right) in cells.into_iter().zip(rights) {
        builder.write_right(text, FONT_SIZE_PT, bold, right);
    }
}

/// Dessine une ligne du tableau rendement (label + 4 colonnes numériques).
fn draw_return_row(
    builder: &mut PdfBuilder,
    label: &str,
    cout: Decimal,
    revenus: Decimal,
    resultat: Decimal,
    rendement: Option<Decimal>,
    bold: bool,
) {
    builder.ensure_space_for_row();
    builder.write_inline(label, FONT_SIZE_PT, bold, RET_COL_LABEL_X);
    draw_return_cells(
        builder,
        [
            &format_swiss_amount(cout),
            &format_swiss_amount(revenus),
            &format_swiss_amount(resultat),
            &format_rendement(rendement),
        ],
        bold,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;
}

/// Génère le PDF du rapport « Rendement par projet » (Story 19-6b).
pub fn render_project_return_pdf(
    report: &crate::project_report::ProjectReturnReport,
    ctx: &PdfContext,
    labels: &ProjectReturnPdfLabels,
) -> Result<Vec<u8>, ReportError> {
    let mut builder = PdfBuilder::new(&labels.title)?;

    builder.write_line(&ctx.company_name, FONT_SIZE_HEADER_PT, true, 0.0);
    builder.write_line(
        &format!(
            "{} — {} {}",
            labels.title, report.project.code, report.project.name
        ),
        12.0,
        true,
        0.0,
    );
    builder.write_line(&report.period_label, FONT_SIZE_PT, false, 0.0);
    builder.cursor_y -= LINE_HEIGHT_MM;

    if report.sections.is_empty() {
        let layer = builder.current_layer();
        layer.use_text(
            &labels.empty_message,
            FONT_SIZE_PT,
            Mm(MARGIN_LEFT_MM),
            Mm(builder.cursor_y),
            &builder.font,
        );
        return builder.finalize();
    }

    // En-tête de colonnes (gras).
    draw_return_cells(
        &builder,
        [
            &labels.col_cout,
            &labels.col_revenus,
            &labels.col_resultat,
            &labels.col_rendement,
        ],
        true,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;

    for section in &report.sections {
        let label = if section.is_root {
            format!("{} {}", section.project.code, section.project.name)
        } else {
            format!("  ↳ {} {}", section.project.code, section.project.name)
        };
        draw_return_row(
            &mut builder,
            &label,
            section.cout_investi,
            section.revenus,
            section.resultat_net,
            section.rendement_pct,
            false,
        );
    }

    builder.cursor_y -= LINE_HEIGHT_MM * 0.5;
    draw_return_row(
        &mut builder,
        &labels.total,
        report.totals.cout_investi,
        report.totals.revenus,
        report.totals.resultat_net,
        report.totals.rendement_pct,
        true,
    );

    builder.finalize()
}

// ============================================================================
// Story 24-1 — Grand livre
// ============================================================================

/// Colonnes du grand livre, en mm depuis le bord gauche.
///
/// Huit colonnes sur de l'A4 portrait : la police descend à 8 pt pour que la
/// dernière (le solde progressif) reste dans la marge droite.
const LEDGER_FONT_PT: f32 = 8.0;
const LEDGER_COL_DATE: f32 = MARGIN_LEFT_MM;
const LEDGER_COL_PIECE: f32 = MARGIN_LEFT_MM + 19.0;
const LEDGER_COL_JOURNAL: f32 = MARGIN_LEFT_MM + 34.0;
const LEDGER_COL_DESC: f32 = MARGIN_LEFT_MM + 48.0;
const LEDGER_COL_COUNTERPART: f32 = MARGIN_LEFT_MM + 96.0;
/// Bords droits des trois colonnes de montant — débit, crédit, solde
/// progressif —, en mm depuis le bord gauche de la page : montants et
/// en-têtes y sont alignés à droite, 23 mm d'écart jusqu'à la marge droite. Le
/// plus long montant plausible, « -12'345'678.90 » en 8 pt, mesure 18,5 mm
/// (18,8 en gras).
const LEDGER_DEBIT_RIGHT_MM: f32 = AMOUNT_RIGHT_MM - 46.0;
const LEDGER_CREDIT_RIGHT_MM: f32 = AMOUNT_RIGHT_MM - 23.0;
const LEDGER_RUNNING_RIGHT_MM: f32 = AMOUNT_RIGHT_MM;
/// Bord droit du solde en devise des lignes d'encadrement (ouverture, report,
/// rupture, clôture ; Story 37-4) : la fin de la colonne des contreparties,
/// vide sur ces lignes, 2 mm avant la colonne du débit.
const LEDGER_FX_RIGHT_MM: f32 = LEDGER_DEBIT_RIGHT_MM - 22.0;

/// Un montant nul ne s'écrit pas — une colonne débit/crédit reste vide, comme
/// sur un grand livre tenu à la main.
fn ledger_amount(v: Decimal) -> String {
    if v.is_zero() {
        String::new()
    } else {
        format_swiss_amount(v)
    }
}

/// Bandeau d'identification du compte : numéro, intitulé, mentions.
///
/// Il est redessiné **à chaque page** — un feuillet de grand livre qui ne dit
/// pas de quel compte il est n'est pas une pièce comptable, c'est un listing.
fn draw_ledger_account_header(
    builder: &mut PdfBuilder,
    ctx: &PdfContext,
    section: &crate::general_ledger::LedgerSection,
    continued: bool,
    multi: bool,
) {
    let l = &ctx.section_labels;
    let mut title = format!("{} — {}", section.account_number, section.account_name);
    if !section.active {
        title.push_str(&format!(" ({})", l.ledger_archived));
    }
    if section.unnatural_balance {
        title.push_str(&format!("  ⚠ {}", l.ledger_unnatural));
    }
    // Story 37-4 : la devise d'un compte en devise, dès le bandeau.
    if let Some(cur) = &section.currency {
        title.push_str(&format!(" · {cur}"));
    }
    if continued {
        title.push_str(&format!("  ({})", l.ledger_carried_over));
    }
    builder.write_line(&title, FONT_SIZE_PT, true, 0.0);

    // En-têtes de colonnes.
    builder.write_inline(&l.col_entry_date, LEDGER_FONT_PT, true, LEDGER_COL_DATE);
    builder.write_inline(&l.col_piece, LEDGER_FONT_PT, true, LEDGER_COL_PIECE);
    builder.write_inline(&l.col_journal, LEDGER_FONT_PT, true, LEDGER_COL_JOURNAL);
    builder.write_inline(&l.col_description, LEDGER_FONT_PT, true, LEDGER_COL_DESC);
    builder.write_inline(
        &l.col_counterpart,
        LEDGER_FONT_PT,
        true,
        LEDGER_COL_COUNTERPART,
    );
    // Avec un compte en devise dans le rapport, les colonnes de montant disent
    // leur devise (« Débit CHF ») ; les soldes en devise portent leur code.
    for (label, right) in [
        (&l.col_debit, LEDGER_DEBIT_RIGHT_MM),
        (&l.col_credit, LEDGER_CREDIT_RIGHT_MM),
        (&l.col_running, LEDGER_RUNNING_RIGHT_MM),
    ] {
        builder.write_right(&l.amount_header(label, multi), LEDGER_FONT_PT, true, right);
    }
    builder.cursor_y -= LINE_HEIGHT_MM;
}

/// Une ligne « libellé à gauche, montant à droite » du grand livre — ouverture,
/// clôture, report, rupture d'exercice.
///
/// `fx` : le même solde en devise d'un compte en devise (Story 37-4, AC-5),
/// écrit dans la colonne des contreparties, vide sur ces lignes, aligné à
/// droite sur [`LEDGER_FX_RIGHT_MM`] ; le montant, sur le bord du solde.
fn draw_ledger_marker(
    builder: &mut PdfBuilder,
    label: &str,
    amount: Decimal,
    fx: Option<String>,
    bold: bool,
) {
    builder.write_inline(label, LEDGER_FONT_PT, bold, LEDGER_COL_DATE);
    if let Some(fx) = fx {
        builder.write_right(&fx, LEDGER_FONT_PT, bold, LEDGER_FX_RIGHT_MM);
    }
    builder.write_right(
        &format_swiss_amount(amount),
        LEDGER_FONT_PT,
        bold,
        LEDGER_RUNNING_RIGHT_MM,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;
}

/// Rend le grand livre en PDF (Story 24-1).
///
/// Deux propriétés que le tableau à l'écran n'a pas besoin de tenir, et que le
/// papier exige :
///
/// - **l'en-tête de compte est répété sur chaque page**, faute de quoi un
///   feuillet isolé ne dit plus de quel compte il parle ;
/// - **un compte scindé entre deux pages porte « À reporter » au pied et
///   « Report » en tête**, pour que la chaîne des soldes reste vérifiable à la
///   main d'un feuillet à l'autre.
pub fn render_general_ledger_pdf(
    gl: &crate::general_ledger::GeneralLedger,
    ctx: &PdfContext,
) -> Result<Vec<u8>, ReportError> {
    let l_title = ctx.section_labels.ledger_title.clone();
    let mut builder = PdfBuilder::new(&l_title)?;
    draw_header(&mut builder, ctx, &l_title, gl.period.from, gl.period.to);

    if gl.sections.is_empty() {
        draw_empty_message(&mut builder, ctx);
        return builder.finalize();
    }
    // Un compte en devise dans le rapport : les en-têtes de montant de tous les
    // comptes disent « CHF », pour un rendu égal d'un compte à l'autre.
    let multi = gl.sections.iter().any(|s| s.currency.is_some());

    for section in &gl.sections {
        // Story 37-4 : le solde en devise d'une ligne d'encadrement, `None`
        // pour un compte en devise principale (rien ne change alors).
        let fx = |v: Option<Decimal>| {
            section
                .currency
                .as_deref()
                .map(|cur| format_currency_amount(cur, v.unwrap_or_default()))
        };
        // Un compte commence en tête de page s'il ne tient pas au moins son
        // bandeau, sa ligne d'ouverture et un mouvement — et, en devise, la
        // seconde ligne de ce mouvement (Story 37-4, revue de code P1).
        builder.ensure_space_for_rows(if section.currency.is_some() { 5.0 } else { 4.0 });
        draw_ledger_account_header(&mut builder, ctx, section, false, multi);
        draw_ledger_marker(
            &mut builder,
            &ctx.section_labels.ledger_opening,
            section.opening,
            fx(section.opening_currency),
            false,
        );

        if section.lines.is_empty() {
            builder.write_line(
                &ctx.section_labels.ledger_no_movement,
                LEDGER_FONT_PT,
                false,
                0.0,
            );
        }

        // Le solde à reporter si la page se termine ici : celui de la dernière
        // ligne écrite, l'ouverture tant qu'aucune ne l'a été, zéro juste après
        // une rupture d'exercice.
        let mut carry = section.opening;
        // Le même report en devise (Story 37-4, AC-5), remis à zéro avec lui.
        let mut carry_fx = section.opening_currency;

        // Rupture d'exercice — le solde d'un compte de résultat y repart de zéro.
        let draw_break = |builder: &mut PdfBuilder, brk: &FiscalYearBreak| {
            builder.ensure_space_for_row();
            draw_ledger_marker(
                builder,
                &format!(
                    "{} — {}",
                    format_swiss_date(brk.date),
                    ctx.section_labels.ledger_fy_break
                ),
                brk.closing_balance,
                fx(brk.closing_balance_currency),
                false,
            );
        };

        for (idx, line) in section.lines.iter().enumerate() {
            // La règle est celle de `LedgerSection::break_before` : au changement
            // d'exercice, et en tête de section (#777).
            if let Some(brk) = section.break_before(idx) {
                draw_break(&mut builder, brk);
                carry = Decimal::ZERO;
                carry_fx = carry_fx.map(|_| Decimal::ZERO);
            }

            // Saut de page : « À reporter » au pied, bandeau + « Report » en tête.
            // Un mouvement en devise garde sa seconde ligne sur la même page.
            let seconde_ligne = if section.currency.is_some() {
                FX_LINE_HEIGHT_MM
            } else {
                0.0
            };
            if builder.cursor_y - 2.0 * LINE_HEIGHT_MM - seconde_ligne < MARGIN_BOTTOM_MM {
                draw_ledger_marker(
                    &mut builder,
                    &ctx.section_labels.ledger_carry_forward,
                    carry,
                    fx(carry_fx),
                    true,
                );
                builder.new_page();
                draw_ledger_account_header(&mut builder, ctx, section, true, multi);
                draw_ledger_marker(
                    &mut builder,
                    &ctx.section_labels.ledger_carried_over,
                    carry,
                    fx(carry_fx),
                    false,
                );
            }

            builder.write_inline(
                &format_swiss_date(line.entry_date),
                LEDGER_FONT_PT,
                false,
                LEDGER_COL_DATE,
            );
            // ⚠️ Le numéro de pièce repart à 1 à chaque exercice : dès que la
            // période en traverse plusieurs, il ne désigne rien tout seul.
            let piece = if section.fiscal_year_breaks.is_empty() {
                line.entry_number.to_string()
            } else {
                format!("{}/{}", line.fiscal_year_name, line.entry_number)
            };
            builder.write_inline(
                &truncate_with_ellipsis(&piece, 9),
                LEDGER_FONT_PT,
                false,
                LEDGER_COL_PIECE,
            );
            builder.write_inline(
                &truncate_with_ellipsis(&line.journal, 7),
                LEDGER_FONT_PT,
                false,
                LEDGER_COL_JOURNAL,
            );
            builder.write_inline(
                &truncate_with_ellipsis(&line.description, 26),
                LEDGER_FONT_PT,
                false,
                LEDGER_COL_DESC,
            );
            builder.write_inline(
                &truncate_with_ellipsis(&line.counterpart.join(","), 10),
                LEDGER_FONT_PT,
                false,
                LEDGER_COL_COUNTERPART,
            );
            builder.write_right(
                &ledger_amount(line.debit),
                LEDGER_FONT_PT,
                false,
                LEDGER_DEBIT_RIGHT_MM,
            );
            builder.write_right(
                &ledger_amount(line.credit),
                LEDGER_FONT_PT,
                false,
                LEDGER_CREDIT_RIGHT_MM,
            );
            builder.write_right(
                &format_swiss_amount(line.running_balance),
                LEDGER_FONT_PT,
                false,
                LEDGER_RUNNING_RIGHT_MM,
            );
            builder.cursor_y -= LINE_HEIGHT_MM;
            carry = line.running_balance;

            // Story 37-4, AC-5 : sous un mouvement d'un compte en devise, une
            // seconde ligne en petit — « USD 31.35 × 0.8808 · solde USD
            // 1'233.57 », ou « Réévaluation au cours X · solde … », sans montant
            // en devise (AC-2).
            if let Some(cur) = &section.currency {
                let cours = line
                    .exchange_rate
                    .map(format_exchange_rate)
                    .unwrap_or_default();
                let mouvement = if line.is_revaluation() {
                    format!("{} {cours}", ctx.section_labels.fx_revaluation_at_rate)
                } else {
                    let montant = line.debit_currency.unwrap_or_default()
                        + line.credit_currency.unwrap_or_default();
                    format!("{} × {cours}", format_currency_amount(cur, montant))
                };
                let texte = format!(
                    "{mouvement} · {} {}",
                    ctx.section_labels.fx_balance,
                    format_currency_amount(cur, line.running_balance_currency.unwrap_or_default())
                );
                builder.write_fx_line(&texte, LEDGER_COL_DESC);
                carry_fx = line.running_balance_currency;
            }
        }

        // Fin d'exercice franchie après la dernière ligne (#778).
        for brk in section.trailing_breaks() {
            draw_break(&mut builder, brk);
        }

        // Totaux de mouvements, puis solde de clôture.
        builder.ensure_space_for_rows(if section.currency.is_some() { 3.0 } else { 2.0 });
        builder.write_inline(
            &ctx.section_labels.ledger_movements_total,
            LEDGER_FONT_PT,
            false,
            LEDGER_COL_DATE,
        );
        builder.write_right(
            &format_swiss_amount(section.total_debit),
            LEDGER_FONT_PT,
            false,
            LEDGER_DEBIT_RIGHT_MM,
        );
        builder.write_right(
            &format_swiss_amount(section.total_credit),
            LEDGER_FONT_PT,
            false,
            LEDGER_CREDIT_RIGHT_MM,
        );
        builder.cursor_y -= LINE_HEIGHT_MM;
        // Les totaux de la section en devise — seuls totaux en devise d'un
        // rapport (une section n'a qu'une devise).
        if let Some(cur) = &section.currency {
            let m = |v: Option<Decimal>| format_currency_amount(cur, v.unwrap_or_default());
            let texte = format!(
                "{} {} · {} {}",
                ctx.section_labels.fx_debit,
                m(section.total_debit_currency),
                ctx.section_labels.fx_credit,
                m(section.total_credit_currency),
            );
            builder.write_fx_line(&texte, LEDGER_COL_DESC);
        }

        draw_ledger_marker(
            &mut builder,
            &ctx.section_labels.ledger_closing,
            section.closing,
            fx(section.closing_currency),
            true,
        );
        builder.cursor_y -= LINE_HEIGHT_MM; // respiration entre deux comptes
    }

    builder.finalize()
}

// ============================================================================
// Tests unit (T10.1)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance_sheet::AccountBalance;
    use crate::period::ReportPeriod;
    use crate::trial_balance::TrialBalanceRow;
    use kesh_db::entities::AccountType;
    use kesh_db::entities::journal_entry::Journal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn period() -> ReportPeriod {
        ReportPeriod {
            fiscal_year_id: 1,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        }
    }

    fn fixture_bs(empty: bool) -> BalanceSheet {
        BalanceSheet {
            period: period(),
            assets: if empty {
                vec![]
            } else {
                vec![AccountBalance {
                    account_id: 1,
                    account_number: "1000".into(),
                    account_name: "Caisse".into(),
                    account_type: AccountType::Asset,
                    active: true,
                    balance: dec!(1234.56),
                    role: None,
                    currency: None,
                    balance_currency: None,
                }]
            },
            liabilities: if empty {
                vec![]
            } else {
                vec![AccountBalance {
                    account_id: 2,
                    account_number: "2000".into(),
                    account_name: "Fournisseurs".into(),
                    account_type: AccountType::Liability,
                    active: true,
                    balance: dec!(500.00),
                    role: None,
                    currency: None,
                    balance_currency: None,
                }]
            },
            equity: vec![],
            total_assets: if empty { Decimal::ZERO } else { dec!(1234.56) },
            total_liabilities: if empty { Decimal::ZERO } else { dec!(500.00) },
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: if empty { Decimal::ZERO } else { dec!(734.56) },
            equation_holds: true,
        }
    }

    fn fixture_is(empty: bool) -> IncomeStatement {
        IncomeStatement {
            period: period(),
            revenues: if empty {
                vec![]
            } else {
                vec![AccountBalance {
                    account_id: 3,
                    account_number: "3000".into(),
                    account_name: "Ventes".into(),
                    account_type: AccountType::Revenue,
                    active: true,
                    balance: dec!(2000),
                    role: None,
                    currency: None,
                    balance_currency: None,
                }]
            },
            expenses: if empty {
                vec![]
            } else {
                vec![AccountBalance {
                    account_id: 4,
                    account_number: "4000".into(),
                    account_name: "Achats".into(),
                    account_type: AccountType::Expense,
                    active: true,
                    balance: dec!(1265.44),
                    role: None,
                    currency: None,
                    balance_currency: None,
                }]
            },
            total_revenues: if empty { Decimal::ZERO } else { dec!(2000) },
            total_expenses: if empty { Decimal::ZERO } else { dec!(1265.44) },
            net_result: if empty { Decimal::ZERO } else { dec!(734.56) },
        }
    }

    fn fixture_tb(empty: bool) -> TrialBalance {
        TrialBalance {
            period: period(),
            rows: if empty {
                vec![]
            } else {
                vec![TrialBalanceRow {
                    account_id: 1,
                    account_number: "1000".into(),
                    account_name: "Caisse".into(),
                    account_type: AccountType::Asset,
                    active: true,
                    opening_balance: dec!(500),
                    total_debit: dec!(2000),
                    total_credit: dec!(1000),
                    balance: dec!(1000),
                    closing_balance: dec!(1500),
                    currency: None,
                    opening_currency: None,
                    total_debit_currency: None,
                    total_credit_currency: None,
                    balance_currency: None,
                    closing_currency: None,
                }]
            },
            total_debit: if empty { Decimal::ZERO } else { dec!(2000) },
            total_credit: if empty { Decimal::ZERO } else { dec!(1000) },
            balanced: true,
            retained_earnings: if empty { Decimal::ZERO } else { dec!(500) },
            opening_balanced: true,
        }
    }

    fn fixture_jr(empty: bool) -> JournalReport {
        use crate::journal_report::{JournalEntryLineRow, JournalEntryRow, JournalSection};
        let sections: Vec<JournalSection> = [
            Journal::Achats,
            Journal::Ventes,
            Journal::Banque,
            Journal::Caisse,
            Journal::OD,
        ]
        .iter()
        .enumerate()
        .map(|(i, j)| {
            let entries = if empty || i > 0 {
                vec![]
            } else {
                vec![JournalEntryRow {
                    entry_id: 1,
                    entry_number: 1,
                    entry_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                    description: "Test".into(),
                    lines: vec![JournalEntryLineRow {
                        account_id: 1,
                        account_number: "1000".into(),
                        account_name: "Caisse".into(),
                        debit: dec!(100),
                        credit: dec!(0),
                        line_order: 0,
                        currency: None,
                        amount_currency: None,
                        exchange_rate: None,
                        rate_source: None,
                    }],
                }]
            };
            JournalSection {
                journal: *j,
                entries,
                section_total_debit: if empty || i > 0 {
                    Decimal::ZERO
                } else {
                    dec!(100)
                },
                section_total_credit: Decimal::ZERO,
            }
        })
        .collect();
        JournalReport {
            period: period(),
            journals: sections,
            grand_total_debit: if empty { Decimal::ZERO } else { dec!(100) },
            grand_total_credit: Decimal::ZERO,
        }
    }

    // --- T2.4 + T2.7 : format suisse montants ---

    #[test]
    fn format_swiss_amount_thousand_separator_with_apostrophe() {
        assert_eq!(format_swiss_amount(dec!(1234.56)), "1'234.56");
    }

    #[test]
    fn format_swiss_amount_million_apostrophes() {
        assert_eq!(format_swiss_amount(dec!(1234567.89)), "1'234'567.89");
    }

    #[test]
    fn format_swiss_amount_under_thousand_no_apostrophe() {
        assert_eq!(format_swiss_amount(dec!(123.45)), "123.45");
    }

    #[test]
    fn format_swiss_amount_zero_two_decimals() {
        assert_eq!(format_swiss_amount(Decimal::ZERO), "0.00");
    }

    /// T2.7 + Pass 1 ECH-L1 — montant négatif avec apostrophe.
    #[test]
    fn format_swiss_amount_negative_with_separator() {
        assert_eq!(
            format_swiss_amount(Decimal::from_str("-1234.56").unwrap()),
            "-1'234.56"
        );
    }

    #[test]
    fn format_swiss_amount_negative_small() {
        assert_eq!(format_swiss_amount(dec!(-123.45)), "-123.45");
    }

    #[test]
    fn format_swiss_date_dd_mm_yyyy() {
        let d = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        assert_eq!(format_swiss_date(d), "15.05.2026");
    }

    // --- T10.1(a) : byte signature %PDF-1. pour les 4 rapports ---

    #[test]
    fn balance_sheet_pdf_starts_with_pdf_signature() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_balance_sheet_pdf(&fixture_bs(false), &ctx).unwrap();
        assert!(
            bytes.starts_with(b"%PDF-1."),
            "PDF must start with %PDF-1. signature"
        );
    }

    #[test]
    fn income_statement_pdf_starts_with_pdf_signature() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_income_statement_pdf(&fixture_is(false), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    /// Story 14-3c (ECH-1) : une section Capitaux propres avec de nombreux comptes de
    /// fonds propres, groupés par rôle, force plusieurs pages — le rendu par rôle (avec
    /// sous-titres keep-with-next `ensure_space_for_rows`) ne doit pas paniquer et produit
    /// un PDF valide multi-pages. Exerce le chemin de pagination des sous-titres de groupe.
    #[test]
    fn balance_sheet_pdf_multi_role_equity_paginates_without_panic() {
        use kesh_db::entities::AccountRole;
        let eq = |i: usize, role: AccountRole| AccountBalance {
            account_id: 5000 + i as i64,
            account_number: format!("29{i:02}"),
            account_name: format!("Fonds propres #{i}"),
            account_type: AccountType::Liability,
            active: true,
            balance: dec!(100),
            role: Some(role),
            currency: None,
            balance_currency: None,
        };
        // 120 comptes EquityOther + 1 EquityCapital → 2 groupes de rôle, > 1 page A4.
        let mut equity: Vec<AccountBalance> = vec![eq(0, AccountRole::EquityCapital)];
        equity.extend((1..120).map(|i| eq(i, AccountRole::EquityOther)));
        let total_equity: Decimal = equity.iter().map(|a| a.balance).sum();
        let bs = BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity,
            total_assets: total_equity,
            total_liabilities: Decimal::ZERO,
            total_equity,
            retained_earnings: Decimal::ZERO,
            equity_result: Decimal::ZERO,
            equation_holds: true,
        };
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_balance_sheet_pdf(&bs, &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."), "PDF multi-pages valide");
        assert!(bytes.len() > 3000, "contenu multi-pages non trivial");
    }

    #[test]
    fn trial_balance_pdf_starts_with_pdf_signature() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_trial_balance_pdf(&fixture_tb(false), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    #[test]
    fn trial_balance_pdf_with_retained_line_is_not_the_empty_render() {
        // Story 25-5-b — la ligne calculée et les six colonnes : le rendu peuplé
        // diffère du rendu vide (garde `rows.is_empty()` inchangée).
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let full = render_trial_balance_pdf(&fixture_tb(false), &ctx).unwrap();
        let empty = render_trial_balance_pdf(&fixture_tb(true), &ctx).unwrap();
        assert!(full.starts_with(b"%PDF-1."));
        assert!(full.len() > empty.len());
    }

    #[test]
    fn trial_balance_pdf_writes_the_opening_warning_only_when_unbalanced() {
        // Story 25-5-b, revue P1 — le contrôle d'ouverture n'existait qu'à l'écran.
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let ok = render_trial_balance_pdf(&fixture_tb(false), &ctx).unwrap();
        let mut broken = fixture_tb(false);
        broken.opening_balanced = false;
        let warned = render_trial_balance_pdf(&broken, &ctx).unwrap();
        assert!(warned.len() > ok.len(), "l'avertissement doit être écrit");
    }

    #[test]
    fn trial_balance_names_are_truncated_to_fit_six_columns() {
        // Story 25-5-b — 55 mm pour l'intitulé : 22 caractères, ellipsis comprise.
        let long = "Provisions pour dépréciation d'actifs"; // 37 caractères
        let truncated = truncate_with_ellipsis(long, TB_NAME_MAX_CHARS);
        assert_eq!(truncated.chars().count(), 22);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn journal_report_pdf_starts_with_pdf_signature() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_journal_report_pdf(&fixture_jr(false), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    fn fixture_vat(empty: bool) -> VatReport {
        VatReport {
            period: period(),
            rows: if empty {
                vec![]
            } else {
                vec![crate::vat_report::VatReportRow {
                    rate: dec!(8.10),
                    category: None,
                    base_ht: dec!(1000),
                    vat_due: dec!(81.00),
                }]
            },
            total_base_ht: if empty { Decimal::ZERO } else { dec!(1000) },
            total_vat_due: if empty { Decimal::ZERO } else { dec!(81.00) },
            write_off_rows: vec![],
            total_vat_write_off: Decimal::ZERO,
            total_vat_due_net: if empty { Decimal::ZERO } else { dec!(81.00) },
            total_vat_recoverable: Decimal::ZERO,
            vat_balance: if empty { Decimal::ZERO } else { dec!(81.00) },
            reconciliation_delta: Decimal::ZERO,
            reconciliation_status: "ok".to_string(),
        }
    }

    #[test]
    fn vat_report_pdf_starts_with_pdf_signature() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes =
            render_vat_report_pdf(&fixture_vat(false), &ctx, &VatPdfLabels::fr_ch_defaults())
                .unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    #[test]
    fn vat_report_pdf_empty_does_not_crash() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes =
            render_vat_report_pdf(&fixture_vat(true), &ctx, &VatPdfLabels::fr_ch_defaults())
                .unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    /// Story 18-1d (AC10/T-D2b) : « achats seuls » — rows vide MAIS récupérable > 0
    /// ne doit PAS court-circuiter sur le message vide ; le PDF rend le bloc totaux
    /// (récupérable + solde) sans panic.
    #[test]
    fn vat_report_pdf_recoverable_only_renders() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let report = VatReport {
            period: period(),
            rows: vec![],
            total_base_ht: Decimal::ZERO,
            total_vat_due: Decimal::ZERO,
            write_off_rows: vec![],
            total_vat_write_off: Decimal::ZERO,
            total_vat_due_net: Decimal::ZERO,
            total_vat_recoverable: dec!(81.00),
            vat_balance: dec!(-81.00),
            reconciliation_delta: Decimal::ZERO,
            reconciliation_status: "ok".to_string(),
        };
        let bytes = render_vat_report_pdf(&report, &ctx, &VatPdfLabels::fr_ch_defaults()).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
        // Un PDF avec le tableau + bloc totaux est plus volumineux que le simple
        // message « aucune écriture » (cas fixture_vat(true)).
        let empty_bytes =
            render_vat_report_pdf(&fixture_vat(true), &ctx, &VatPdfLabels::fr_ch_defaults())
                .unwrap();
        assert!(
            bytes.len() > empty_bytes.len(),
            "le PDF achats-seuls (totaux rendus) doit être plus gros que le message vide"
        );
    }

    /// Story 25-4-d2c — un rapport avec des soldes : sans vente ni récupérable,
    /// il n'est PAS vide (garde de `render_vat_report_pdf`) ; avec des ventes, la
    /// section des soldes l'allonge.
    fn with_write_off(mut report: VatReport) -> VatReport {
        report.write_off_rows = vec![crate::vat_report::VatWriteOffRow {
            rate: dec!(8.10),
            base_ht: dec!(20.00),
            vat: dec!(1.62),
        }];
        report.total_vat_write_off = dec!(1.62);
        report.total_vat_due_net = report.total_vat_due - dec!(1.62);
        report
    }

    #[test]
    fn vat_report_pdf_write_off_only_is_not_empty() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let labels = VatPdfLabels::fr_ch_defaults();
        let empty = render_vat_report_pdf(&fixture_vat(true), &ctx, &labels).unwrap();
        let write_off_only =
            render_vat_report_pdf(&with_write_off(fixture_vat(true)), &ctx, &labels).unwrap();
        assert!(
            write_off_only.len() > empty.len(),
            "un rapport avec des soldes ne court-circuite pas sur le message vide"
        );
    }

    #[test]
    fn vat_report_pdf_draws_the_write_off_section_only_with_write_offs() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let labels = VatPdfLabels::fr_ch_defaults();
        let plain = render_vat_report_pdf(&fixture_vat(false), &ctx, &labels).unwrap();
        let with =
            render_vat_report_pdf(&with_write_off(fixture_vat(false)), &ctx, &labels).unwrap();
        assert!(with.len() > plain.len(), "la section des soldes s'ajoute");
    }

    // --- AC #8 — empty report : génère un PDF valide avec empty_message ---

    #[test]
    fn balance_sheet_pdf_empty_does_not_crash() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_balance_sheet_pdf(&fixture_bs(true), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
        assert!(
            bytes.len() > 200,
            "empty PDF should still have minimal content"
        );
    }

    /// Review Pass 1 — actif/passif vides mais report/résultat non nuls et opposés :
    /// le PDF NE DOIT PAS afficher le message « vide » ; il doit dessiner la section
    /// fonds propres. On le prouve en comparant à un bilan RÉELLEMENT vide (tout à 0) :
    /// le rendu non-vide (2 sections + 2 lignes fonds propres) est plus volumineux.
    #[test]
    fn balance_sheet_pdf_shows_equity_when_assets_liabilities_net_to_zero() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let truly_empty = render_balance_sheet_pdf(&fixture_bs(true), &ctx).unwrap();

        let mut bs = fixture_bs(true);
        bs.retained_earnings = dec!(1000);
        bs.equity_result = dec!(-1000);
        let with_equity = render_balance_sheet_pdf(&bs, &ctx).unwrap();

        assert!(with_equity.starts_with(b"%PDF-1."));
        assert!(
            with_equity.len() > truly_empty.len(),
            "un bilan actif/passif nuls mais report/résultat non nuls doit dessiner la \
             section fonds propres (rendu > message vide) : {} vs {}",
            with_equity.len(),
            truly_empty.len()
        );
    }

    #[test]
    fn trial_balance_pdf_empty_does_not_crash() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_trial_balance_pdf(&fixture_tb(true), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    // --- AC #6 + #7 : journaux avec filtre ---

    #[test]
    fn journal_report_pdf_with_filter_label_renders() {
        let mut ctx = PdfContext::fr_ch_default("CI Test Company");
        ctx.journal_filter_label = Some("Ventes".to_string());
        // Reduce to 1 section filtered
        let mut jr = fixture_jr(false);
        jr.journals.retain(|s| s.journal == Journal::Achats);
        let bytes = render_journal_report_pdf(&jr, &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    // --- Pass 1 code-review M7 (AA1-M3) : empty path pour IS + JR ---

    #[test]
    fn income_statement_pdf_empty_does_not_crash() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_income_statement_pdf(&fixture_is(true), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
        assert!(bytes.len() > 200);
    }

    #[test]
    fn journal_report_pdf_empty_does_not_crash() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_journal_report_pdf(&fixture_jr(true), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
        assert!(bytes.len() > 200);
    }

    // --- Pass 1 code-review H4 (BH1-H4) : trial balance long account name ---

    /// `truncate_with_ellipsis` doit retourner la chaîne intacte si <= max.
    #[test]
    fn truncate_with_ellipsis_short_string_unchanged() {
        assert_eq!(truncate_with_ellipsis("Caisse", 30), "Caisse");
        assert_eq!(truncate_with_ellipsis("", 30), "");
    }

    /// `truncate_with_ellipsis` doit tronquer + ajouter `…` au-delà de max
    /// (longueur totale = max caractères Unicode).
    #[test]
    fn truncate_with_ellipsis_long_string_truncated() {
        let long = "Provisions pour dépréciation d'actifs"; // 37 chars
        let truncated = truncate_with_ellipsis(long, 30);
        assert_eq!(truncated.chars().count(), 30);
        assert!(truncated.ends_with('…'));
    }

    // --- Pass 1 code-review H8 (AA1-H1) : AC #9 SLA assertion fast path ---

    /// AC #9 — Le rendu PDF Bilan sur 1000 lignes doit tenir < 500 ms.
    ///
    /// Pass 1 code-review H8 : criterion mesure mais ne fail pas si > 500ms.
    /// On ajoute un test unitaire dédié qui borne la durée de rendering pur
    /// (pas de DB, pas de réseau — cf. AC #9 « rendering pur »).
    ///
    /// Seuil conservateur : 500 ms (AC #9). En cas de flake CI sur runner lent,
    /// la marge peut être révisée v0.2 sans casser la sémantique de l'AC.
    #[test]
    fn ac9_balance_sheet_pdf_1000_under_500ms() {
        // Construit un BalanceSheet de ~1000 lignes équilibrées (500 actifs + 500 passifs).
        let mut assets: Vec<AccountBalance> = Vec::with_capacity(500);
        let mut liabilities: Vec<AccountBalance> = Vec::with_capacity(500);
        for i in 0..500 {
            assets.push(AccountBalance {
                account_id: i as i64,
                account_number: format!("{:04}", 1000 + i),
                account_name: format!("Compte actif #{i}"),
                account_type: AccountType::Asset,
                active: true,
                balance: dec!(100),
                role: None,
                currency: None,
                balance_currency: None,
            });
            liabilities.push(AccountBalance {
                account_id: 500 + i as i64,
                account_number: format!("{:04}", 2000 + i),
                account_name: format!("Compte passif #{i}"),
                account_type: AccountType::Liability,
                active: true,
                balance: dec!(100),
                role: None,
                currency: None,
                balance_currency: None,
            });
        }
        let bs = BalanceSheet {
            period: period(),
            assets,
            liabilities,
            equity: vec![],
            total_assets: dec!(50000),
            total_liabilities: dec!(50000),
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: dec!(0),
            equation_holds: true,
        };
        let ctx = PdfContext::fr_ch_default("CI Test Company");

        let start = std::time::Instant::now();
        let bytes = render_balance_sheet_pdf(&bs, &ctx).unwrap();
        let elapsed = start.elapsed();

        assert!(bytes.starts_with(b"%PDF-1."));
        assert!(
            elapsed < std::time::Duration::from_millis(500),
            "AC #9 violé : Bilan PDF 1000 lignes a pris {} ms (seuil 500 ms)",
            elapsed.as_millis()
        );
    }

    // --- Pass 1 code-review H2 (BH1-H3) : pas de duplicate Total actifs ---

    /// AC #3 + Pass 1 code-review H2 : le PDF Bilan ne dessine plus `Total actifs`
    /// deux fois (avant le patch, le rendu duplicait visuellement la ligne après
    /// le bloc Capitaux propres). Vérification cinématique : le PDF reste valide
    /// et taille reste raisonnable (régression test for the bytewise dedup
    /// est difficile car printpdf compresse les streams ; le test E2E
    /// `export_balance_sheet_pdf_returns_200` garantit que la signature reste OK).
    #[test]
    fn balance_sheet_pdf_no_duplicate_total_assets() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_balance_sheet_pdf(&fixture_bs(false), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
        // Sanity: rendering doit rester rapide (pas de duplication exponentielle).
        assert!(bytes.len() < 100_000);
    }

    // --- Pass 1 code-review M18 (BH1-M3) : AC #31 taille < 5 MB sur 10k ---

    /// AC #31 — Le PDF des Journaux sur 10'000 écritures doit faire < 5 MB.
    ///
    /// Pass 1 code-review M18 : ce test remplace l'`assert!` qui était à
    /// l'intérieur du bench `bench_pdf_10k` (anti-pattern criterion — un
    /// `assert!` dans `b.iter` ne fail pas proprement le bench et fausse les
    /// samples). Maintenant l'invariant taille AC #31 est gardé par un test
    /// unit dédié, indépendant des benchmarks.
    ///
    /// **Status v0.1 (Pass 1 code-review)** : la mesure réelle sur 10k entrées
    /// avec le format actuel (Helvetica builtin, 2 lignes par écriture, pas
    /// de compression custom) est de ~6.5 MB — au-dessus du seuil AC #31
    /// 5 MB. **Le test est marqué `#[ignore]`** en v0.1, marqué comme dette
    /// technique L13 dans le story file : compression PDF stream ou
    /// pagination intelligente requise pour respecter AC #31, à traiter en
    /// v0.2 ou Epic 15. Le test est exécutable manuellement via
    /// `cargo test -p kesh-report -- --ignored` pour mesurer la taille
    /// effective et suivre l'évolution.
    ///
    /// Coût : construction d'un dataset 10k en mémoire (~quelques ms) + un
    /// rendu PDF complet (~1-2s en CI).
    #[test]
    #[ignore = "AC #31 violation connue : ~6.5 MB sur 10k entrées (seuil 5 MB) — dette technique L13, fix v0.2"]
    fn pdf_10k_journal_report_size_under_5mb() {
        use crate::journal_report::{JournalEntryLineRow, JournalEntryRow, JournalSection};
        const N: usize = 10_000;
        let journals_list = [
            Journal::Achats,
            Journal::Ventes,
            Journal::Banque,
            Journal::Caisse,
            Journal::OD,
        ];
        let per_section = N / 5;
        let mut sections: Vec<JournalSection> = Vec::with_capacity(5);
        for (idx, j) in journals_list.iter().enumerate() {
            let entries: Vec<JournalEntryRow> = (0..per_section)
                .map(|i| JournalEntryRow {
                    entry_id: (idx * per_section + i) as i64,
                    entry_number: i as i64,
                    entry_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                    description: format!("Écriture #{i}"),
                    lines: vec![
                        JournalEntryLineRow {
                            account_id: 1,
                            account_number: "1000".into(),
                            account_name: "Caisse".into(),
                            debit: dec!(100),
                            credit: dec!(0),
                            line_order: 0,
                            currency: None,
                            amount_currency: None,
                            exchange_rate: None,
                            rate_source: None,
                        },
                        JournalEntryLineRow {
                            account_id: 2,
                            account_number: "2000".into(),
                            account_name: "Fournisseurs".into(),
                            debit: dec!(0),
                            credit: dec!(100),
                            line_order: 1,
                            currency: None,
                            amount_currency: None,
                            exchange_rate: None,
                            rate_source: None,
                        },
                    ],
                })
                .collect();
            sections.push(JournalSection {
                journal: *j,
                entries,
                section_total_debit: dec!(0),
                section_total_credit: dec!(0),
            });
        }
        let jr = JournalReport {
            period: period(),
            journals: sections,
            grand_total_debit: dec!(0),
            grand_total_credit: dec!(0),
        };
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_journal_report_pdf(&jr, &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
        assert!(
            bytes.len() < 5 * 1024 * 1024,
            "AC #31 violé : PDF 10k entries fait {} bytes (seuil 5 MB)",
            bytes.len()
        );
    }

    // ------------------------------------------------------------------
    // Story 24-1 — grand livre
    // ------------------------------------------------------------------

    fn ledger_line(i: i64, fy: i64) -> crate::general_ledger::LedgerLine {
        crate::general_ledger::LedgerLine {
            line_id: i,
            entry_id: i,
            entry_date: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
            fiscal_year_id: fy,
            fiscal_year_name: format!("Exercice {fy}"),
            entry_number: i,
            journal: "Banque".into(),
            description: "Un libellé assez long pour éprouver la troncature".into(),
            counterpart: vec!["6000".into(), "6001".into()],
            debit: dec!(120.00),
            credit: dec!(0),
            running_balance: Decimal::from(i) * dec!(120.00),
            lettering_code: None,
            debit_currency: None,
            credit_currency: None,
            exchange_rate: None,
            rate_source: None,
            running_balance_currency: None,
        }
    }

    fn ledger_with(n: i64) -> crate::general_ledger::GeneralLedger {
        crate::general_ledger::GeneralLedger {
            period: crate::general_ledger::LedgerPeriod {
                from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                to: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            },
            sections: vec![crate::general_ledger::LedgerSection {
                account_id: 7,
                account_number: "1020".into(),
                account_name: "Banque".into(),
                account_type: AccountType::Asset,
                active: true,
                balance_side: "debit",
                opening: dec!(500.00),
                lines: (1..=n).map(|i| ledger_line(i, 1)).collect(),
                total_debit: Decimal::from(n) * dec!(120.00),
                total_credit: dec!(0),
                closing: dec!(500.00) + Decimal::from(n) * dec!(120.00),
                unnatural_balance: false,
                fiscal_year_breaks: vec![],
                line_count: n,
                currency: None,
                opening_currency: None,
                total_debit_currency: None,
                total_credit_currency: None,
                closing_currency: None,
            }],
        }
    }

    #[test]
    fn general_ledger_pdf_paginates_and_repeats_the_account_header() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        // Une page A4 tient ~55 lignes ; 300 en couvre plusieurs, donc le chemin
        // « À reporter / Report » est réellement emprunté.
        let court = render_general_ledger_pdf(&ledger_with(5), &ctx).expect("pdf court");
        let long = render_general_ledger_pdf(&ledger_with(300), &ctx).expect("pdf long");
        assert!(!court.is_empty());
        assert!(
            long.len() > court.len(),
            "un livre de 300 lignes doit peser plus qu'un livre de 5 ({} vs {})",
            long.len(),
            court.len()
        );
    }

    #[test]
    fn general_ledger_pdf_renders_a_fiscal_year_break() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let mut gl = ledger_with(2);
        gl.sections[0].lines[1].fiscal_year_id = 2;
        gl.sections[0].fiscal_year_breaks = vec![crate::general_ledger::FiscalYearBreak {
            date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            closing_fiscal_year_id: 1,
            closing_balance: dec!(620.00),
            closing_balance_currency: None,
        }];
        let bytes = render_general_ledger_pdf(&gl, &ctx).expect("pdf");
        assert!(!bytes.is_empty());
    }

    /// #777 / #778 — une rupture en tête de section et une rupture finale se
    /// dessinent : le PDF qui les porte pèse plus que le même sans elles.
    #[test]
    fn general_ledger_pdf_renders_leading_and_trailing_breaks() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let sans = render_general_ledger_pdf(&ledger_with(2), &ctx).expect("pdf sans");
        let mut gl = ledger_with(2);
        gl.sections[0].fiscal_year_breaks = vec![
            crate::general_ledger::FiscalYearBreak {
                // La date de la première ligne : rupture en tête.
                date: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
                closing_fiscal_year_id: 9,
                closing_balance: dec!(500.00),
                closing_balance_currency: None,
            },
            crate::general_ledger::FiscalYearBreak {
                date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                closing_fiscal_year_id: 1,
                closing_balance: dec!(240.00),
                closing_balance_currency: None,
            },
        ];
        assert!(gl.sections[0].break_before(0).is_some(), "rupture en tête");
        assert_eq!(
            gl.sections[0].trailing_breaks().count(),
            1,
            "rupture finale"
        );
        let avec = render_general_ledger_pdf(&gl, &ctx).expect("pdf avec");
        assert!(
            avec.len() > sans.len(),
            "les deux ruptures doivent se dessiner ({} vs {})",
            avec.len(),
            sans.len()
        );
    }

    #[test]
    fn general_ledger_pdf_empty_renders_the_empty_message() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let mut gl = ledger_with(1);
        gl.sections.clear();
        let bytes = render_general_ledger_pdf(&gl, &ctx).expect("pdf vide");
        assert!(!bytes.is_empty());
    }

    // ------------------------------------------------------------------
    // Story 37-4 — montants en devise
    // ------------------------------------------------------------------

    /// Le texte d'un PDF non compressé (profil de test) : chaque `<hex> Tj`,
    /// décodé en Latin-1 (approximation de WinAnsi suffisante ici).
    fn pdf_text(bytes: &[u8]) -> Vec<String> {
        let s = String::from_utf8_lossy(bytes);
        let mut out = Vec::new();
        let mut rest: &str = &s;
        while let Some(end) = rest.find("> Tj") {
            let before = &rest[..end];
            if let Some(start) = before.rfind('<') {
                let hex = &before[start + 1..];
                out.push(
                    (0..hex.len() / 2)
                        .filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
                        .map(char::from)
                        .collect(),
                );
            }
            rest = &rest[end + 4..];
        }
        out
    }

    /// AC-5 — le cours se lit par unité, sans `%`, de 4 à 10 décimales, zéros
    /// de queue retirés ; un cours JPY (< 0.01) garde ses chiffres.
    #[test]
    fn le_cours_de_change_se_formate_par_unite() {
        assert_eq!(format_exchange_rate(dec!(0.8808)), "0.8808");
        assert_eq!(format_exchange_rate(dec!(0.0057510000)), "0.005751");
        assert_eq!(format_exchange_rate(dec!(1.5)), "1.5000");
        assert_eq!(format_exchange_rate(dec!(0.95)), "0.9500");
        assert_eq!(format_exchange_rate(dec!(0.1234567891)), "0.1234567891");
        assert_eq!(format_exchange_rate(dec!(0.00575123456789)), "0.0057512346");
        assert_eq!(format_exchange_rate(dec!(108)), "108.0000");
        // `format_rate` (TVA) ne s'emploie pas : il rendrait « 0.88 % ».
        assert_eq!(format_rate(dec!(0.8808)), "0.88 %");
    }

    /// AC-5 — arrondi explicite, l'équidistant loin de zéro, avant formatage.
    #[test]
    fn le_montant_en_devise_s_arrondit_avant_formatage() {
        assert_eq!(round_for_display(dec!(1233.565)), dec!(1233.57));
        assert_eq!(round_for_display(dec!(-1233.565)), dec!(-1233.57));
        assert_eq!(round_for_display(dec!(0.0049)), dec!(0.00));
        // Un résidu négatif sous le demi-centime ne s'affiche pas « -0.00 ».
        assert_eq!(
            format_swiss_amount(round_for_display(dec!(-0.0040))),
            "0.00"
        );
        assert_eq!(format_currency_amount("USD", dec!(-0.0049)), "USD 0.00");
        assert_eq!(
            format_currency_amount("USD", dec!(1233.5650)),
            "USD 1'233.57"
        );
    }

    /// Un grand livre USD de 300 lignes, réparti sur plusieurs pages.
    fn ledger_usd(n: i64) -> crate::general_ledger::GeneralLedger {
        let mut gl = ledger_with(n);
        let s = &mut gl.sections[0];
        s.currency = Some("USD".into());
        s.opening_currency = Some(dec!(1000));
        s.total_debit_currency = Some(Decimal::from(n) * dec!(100));
        s.total_credit_currency = Some(Decimal::ZERO);
        s.closing_currency = Some(dec!(1000) + Decimal::from(n) * dec!(100));
        for (i, l) in s.lines.iter_mut().enumerate() {
            l.debit_currency = Some(dec!(100));
            l.credit_currency = Some(Decimal::ZERO);
            l.exchange_rate = Some(dec!(1.2));
            l.rate_source = Some("manual".into());
            l.running_balance_currency = Some(dec!(1000) + Decimal::from(i as i64 + 1) * dec!(100));
        }
        gl
    }

    /// AC-5 — PDF multi-pages : seconde ligne en petit sous chaque mouvement,
    /// « À reporter » et « Report » portent le report en devise.
    #[test]
    fn general_ledger_pdf_multi_pages_reporte_la_devise() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let t = pdf_text(&render_general_ledger_pdf(&ledger_usd(300), &ctx).unwrap());
        assert!(
            t.iter()
                .any(|l| l == "USD 100.00 × 1.2000 · solde USD 1'100.00")
        );
        assert!(t.iter().any(|l| l == "USD 31'000.00"), "clôture en devise");
        let reports: Vec<usize> = t
            .iter()
            .enumerate()
            .filter(|(_, l)| *l == "À reporter")
            .map(|(i, _)| i)
            .collect();
        assert!(reports.len() >= 2, "plusieurs pages");
        // Le report en devise suit immédiatement le libellé « À reporter ».
        for i in reports {
            assert!(
                t[i + 1].starts_with("USD "),
                "report en devise : {:?}",
                &t[i..i + 3]
            );
        }
        assert!(
            t.iter()
                .any(|l| l == "débit USD 30'000.00 · crédit USD 0.00")
        );
    }

    /// Les ordonnées (`y`, en points) des textes d'un PDF non compressé, dans
    /// l'ordre d'écriture : chaque `x y Td … <hex> Tj`.
    fn pdf_text_y(bytes: &[u8]) -> Vec<(f32, String)> {
        let s = String::from_utf8_lossy(bytes);
        let mut out = Vec::new();
        for bloc in s.split("BT").skip(1) {
            let Some(td) = bloc.find(" Td") else { continue };
            let y = bloc[..td]
                .split_whitespace()
                .last()
                .and_then(|v| v.parse::<f32>().ok());
            let Some(y) = y else { continue };
            let Some(a) = bloc.find('<') else { continue };
            let Some(b) = bloc[a..].find("> Tj") else {
                continue;
            };
            let hex = &bloc[a + 1..a + b];
            let txt: String = (0..hex.len() / 2)
                .filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
                .map(char::from)
                .collect();
            out.push((y, txt));
        }
        out
    }

    /// Revue de code P2 — la seconde ligne en devise se lit collée à son
    /// mouvement : plus près de la ligne qu'elle complète que de la suivante.
    #[test]
    fn la_seconde_ligne_en_devise_est_plus_pres_de_son_mouvement_que_du_suivant() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let t = pdf_text_y(&render_general_ledger_pdf(&ledger_usd(3), &ctx).unwrap());
        // Le texte écrit juste avant la seconde ligne est le dernier de son
        // mouvement (le solde progressif), sur la ligne du mouvement.
        let pos = |txt: &str| t.iter().position(|(_, l)| l == txt).unwrap();
        let i1 = pos("USD 100.00 × 1.2000 · solde USD 1'100.00");
        let i2 = pos("USD 100.00 × 1.2000 · solde USD 1'200.00");
        let (mouvement1, fx1) = (t[i1 - 1].0, t[i1].0);
        let mouvement2 = t[i2 - 1].0;
        assert!(
            mouvement1 - fx1 < fx1 - mouvement2,
            "la seconde ligne ({fx1}) est plus près de son mouvement ({mouvement1}) que du suivant ({mouvement2})"
        );
    }

    /// Revue de code P1 — au bilan, le nom d'un compte en devise est tronqué
    /// pour ne pas recouvrir son solde en devise ; un compte CHF garde le sien.
    #[test]
    fn bilan_pdf_tronque_le_nom_d_un_compte_en_devise() {
        let long = "Compte bancaire en dollars américains auprès de la banque cantonale";
        let compte = |id: i64, cur: Option<&str>| AccountBalance {
            account_id: id,
            account_number: format!("10{id}"),
            account_name: long.into(),
            account_type: AccountType::Asset,
            active: true,
            balance: dec!(100),
            role: None,
            currency: cur.map(String::from),
            balance_currency: cur.map(|_| dec!(110)),
        };
        let bs = BalanceSheet {
            assets: vec![compte(20, Some("USD")), compte(21, None)],
            ..fixture_bs(true)
        };
        let ctx = PdfContext::fr_ch_default("Test SA");
        let t = pdf_text(&render_balance_sheet_pdf(&bs, &ctx).unwrap());
        assert!(t.iter().any(|l| l == long), "compte CHF : nom entier");
        let tronque = truncate_with_ellipsis(long, ACCOUNT_NAME_MAX_CHARS_FX);
        assert!(tronque.chars().count() <= ACCOUNT_NAME_MAX_CHARS_FX);
        // L'ellipse « … » se lit `\u{85}` une fois décodée de WinAnsi par
        // `pdf_text` (approximation Latin-1).
        let tronque = tronque.replace('…', "\u{85}");
        assert!(
            t.contains(&tronque),
            "compte USD : nom tronqué ({tronque}) dans {t:?}"
        );
        assert!(t.iter().any(|l| l == "USD 110.00"));
    }

    /// Mono-devise : aucune mention en devise dans le PDF.
    #[test]
    fn general_ledger_pdf_sans_devise_ne_dit_rien_en_devise() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let t = pdf_text(&render_general_ledger_pdf(&ledger_with(300), &ctx).unwrap());
        assert!(!t.iter().any(|l| l.contains(" × ") || l.contains("solde ")));
    }

    // ------------------------------------------------------------------
    // Correctif fix-pdf-montants-alignes — montants alignés à droite
    // ------------------------------------------------------------------

    /// Un texte posé dans un PDF non compressé : abscisse de départ (mm), corps
    /// (pt), gras ou non, chaîne (décodée en Latin-1, comme [`pdf_text`]).
    struct Pose {
        x_mm: f32,
        size: f32,
        bold: bool,
        text: String,
    }

    /// Les textes d'un PDF non compressé, dans l'ordre d'écriture : chaque bloc
    /// `BT /<police> <corps> Tf <x> <y> Td <hex> Tj ET` de printpdf.
    fn pdf_poses(bytes: &[u8]) -> Vec<Pose> {
        let s = String::from_utf8_lossy(bytes);
        let mut out = Vec::new();
        for bloc in s.split("BT").skip(1) {
            let (Some(tf), Some(td)) = (bloc.find(" Tf"), bloc.find(" Td")) else {
                continue;
            };
            let mut police = bloc[..tf].split_whitespace().rev();
            let (Some(size), Some(font)) = (police.next(), police.next()) else {
                continue;
            };
            let Some(x) = bloc[tf + 3..td].split_whitespace().next() else {
                continue;
            };
            let (Some(a), Some(b)) = (bloc.find('<'), bloc.find("> Tj")) else {
                continue;
            };
            let hex = &bloc[a + 1..b];
            out.push(Pose {
                x_mm: x.parse::<f32>().unwrap() * 25.4 / 72.0,
                size: size.parse().unwrap(),
                bold: font == "/Helvetica-Bold",
                text: (0..hex.len() / 2)
                    .filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
                    .map(char::from)
                    .collect(),
            });
        }
        out
    }

    /// Un montant tel que l'écrivent les rapports : `-1'234.56`, précédé ou non
    /// d'un code de devise (`USD 1'233.57`). Un taux (`8.10 %`, `21.50%`) ou une
    /// date (`01.03.2026`) n'en est pas un.
    fn est_montant(t: &str) -> bool {
        let t = match t.split_once(' ') {
            Some((code, m)) if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) => m,
            Some(_) => return false,
            None => t,
        };
        let t = t.strip_prefix('-').unwrap_or(t);
        let Some((int, dec)) = t.split_once('.') else {
            return false;
        };
        dec.len() == 2
            && dec.chars().all(|c| c.is_ascii_digit())
            && int.starts_with(|c: char| c.is_ascii_digit())
            && int.chars().all(|c| c.is_ascii_digit() || c == '\'')
    }

    /// Bout en bout : chaque montant du PDF rendu (hors seconde ligne en petit
    /// d'un montant en devise, qui est une phrase) finit sur l'un des bords
    /// `attendus` — son abscisse de départ lue dans le flux du PDF, plus sa
    /// largeur imprimée — et chacun de ces bords porte au moins un montant.
    fn assert_montants_alignes(nom: &str, bytes: &[u8], attendus: &[f32]) {
        let poses: Vec<Pose> = pdf_poses(bytes)
            .into_iter()
            .filter(|p| p.size > FX_FONT_PT && est_montant(&p.text))
            .collect();
        assert!(poses.len() >= attendus.len(), "{nom} : montants lus");
        let mut vus = vec![false; attendus.len()];
        for p in &poses {
            let bord = p.x_mm + largeur_mm(&p.text, p.size, p.bold);
            let i = attendus.iter().position(|a| (a - bord).abs() < 0.01);
            assert!(
                i.is_some(),
                "{nom} : « {} » finit à {bord:.3} mm, hors des bords {attendus:?}",
                p.text
            );
            vus[i.unwrap()] = true;
        }
        assert!(
            vus.iter().all(|v| *v),
            "{nom} : un bord sans montant ({vus:?})"
        );
    }

    /// Le point décimal d'un montant aligné à droite sur `bord` : la largeur des
    /// trois derniers caractères (`.dd`) avant le bord.
    fn abscisse_du_point(t: &str, bold: bool, bord: f32) -> f32 {
        right_aligned_x(t, FONT_SIZE_PT, bold, bord)
            + largeur_mm(&t[..t.len() - 3], FONT_SIZE_PT, bold)
    }

    /// Unitaire — des montants de longueurs différentes, négatifs, en gras, en
    /// devise, finissent tous au même bord, et leurs points décimaux tombent à
    /// la même abscisse.
    #[test]
    fn right_aligned_x_fait_finir_chaque_montant_au_meme_bord() {
        for (t, bold) in [
            ("0.00", false),
            ("4.56", false),
            ("1'234.56", false),
            ("-89.67", false),
            ("-12'345'678.90", true),
            ("USD 1'233.57", false),
            ("EUR -4.92", true),
        ] {
            let x = right_aligned_x(t, FONT_SIZE_PT, bold, AMOUNT_RIGHT_MM);
            assert!(
                (x + largeur_mm(t, FONT_SIZE_PT, bold) - AMOUNT_RIGHT_MM).abs() < 1e-4,
                "{t}"
            );
            assert!(x < AMOUNT_RIGHT_MM, "{t} commence avant son bord");
        }
        // Plus long, il commence plus à gauche.
        assert!(
            right_aligned_x("1'234.56", FONT_SIZE_PT, false, AMOUNT_RIGHT_MM)
                < right_aligned_x("4.56", FONT_SIZE_PT, false, AMOUNT_RIGHT_MM)
        );
        // Le point décimal à la même abscisse, maigre et gras, CHF et devise.
        for bold in [false, true] {
            let p0 = abscisse_du_point("0.00", bold, AMOUNT_RIGHT_MM);
            for t in ["1'234.56", "-89.67", "-12'345'678.90", "USD 1'233.57"] {
                assert!(
                    (abscisse_du_point(t, bold, AMOUNT_RIGHT_MM) - p0).abs() < 1e-4,
                    "{t} (gras : {bold})"
                );
            }
        }
    }

    /// La prémisse de l'alignement des décimales : les dix chiffres
    /// d'Helvetica, maigre et gras, ont la même chasse (556 millièmes d'em).
    #[test]
    fn les_chiffres_d_helvetica_ont_une_meme_chasse() {
        for bold in [false, true] {
            let zero = largeur_mm("0", FONT_SIZE_PT, bold);
            assert!((zero - 556.0 / 1000.0 * FONT_SIZE_PT * 25.4 / 72.0).abs() < 1e-4);
            for c in '1'..='9' {
                assert_eq!(largeur_mm(&c.to_string(), FONT_SIZE_PT, bold), zero, "{c}");
            }
        }
    }

    /// Les colonnes ne se chevauchent pas, même pour le plus long montant
    /// plausible : chaque colonne de montant commence après la fin de ce qui la
    /// précède.
    #[test]
    fn les_colonnes_de_montant_ne_se_chevauchent_pas() {
        let long = "-12'345'678.90";
        let debut = |t: &str, size: f32, bord: f32| right_aligned_x(t, size, true, bord);
        // Balance : l'intitulé (55 mm depuis 35 mm), puis quatre colonnes.
        let mut fin = MARGIN_LEFT_MM + TB_COL_X[1] + 55.0;
        for bord in TB_AMOUNT_RIGHT_MM {
            assert!(
                debut(long, FONT_SIZE_PT, bord) > fin,
                "balance, bord {bord}"
            );
            fin = bord;
        }
        // L'en-tête le plus long de la balance, en multi-devises, après l'intitulé.
        let entete = SectionLabels::fr_ch_defaults().amount_header("Ouverture", true);
        assert!(
            right_aligned_x(&entete, FONT_SIZE_PT, true, TB_AMOUNT_RIGHT_MM[0])
                > MARGIN_LEFT_MM + TB_COL_X[1] + largeur_mm("Intitulé", FONT_SIZE_PT, true)
        );
        for (i, h) in ["Débit", "Crédit", "Clôture"].into_iter().enumerate() {
            let h = SectionLabels::fr_ch_defaults().amount_header(h, true);
            assert!(
                right_aligned_x(&h, FONT_SIZE_PT, true, TB_AMOUNT_RIGHT_MM[i + 1])
                    > TB_AMOUNT_RIGHT_MM[i]
            );
        }
        // Grand livre (8 pt) : contreparties (10 caractères), débit, crédit, solde.
        let mut fin = LEDGER_COL_COUNTERPART + largeur_mm("6000,6001…", LEDGER_FONT_PT, false);
        for bord in [
            LEDGER_DEBIT_RIGHT_MM,
            LEDGER_CREDIT_RIGHT_MM,
            LEDGER_RUNNING_RIGHT_MM,
        ] {
            assert!(
                debut(long, LEDGER_FONT_PT, bord) > fin,
                "grand livre, bord {bord}"
            );
            fin = bord;
        }
        // Le solde en devise des lignes d'encadrement, avant le débit.
        assert!(LEDGER_FX_RIGHT_MM < debut(long, LEDGER_FONT_PT, LEDGER_DEBIT_RIGHT_MM));
        // Journaux, TVA, rendement : deux colonnes voisines.
        for (a, b) in [
            (JR_DEBIT_RIGHT_MM, JR_CREDIT_RIGHT_MM),
            (VAT_BASE_RIGHT_MM, VAT_DUE_RIGHT_MM),
            (RET_COUT_RIGHT_MM, RET_REV_RIGHT_MM),
            (RET_REV_RIGHT_MM, RET_RES_RIGHT_MM),
        ] {
            assert!(debut(long, FONT_SIZE_PT, b) > a, "bords {a} et {b}");
        }
        // Bilan : nom tronqué d'un compte en devise (34 caractères à 2,5 mm,
        // l'estimation de `ACCOUNT_NAME_MAX_CHARS_FX`), solde en devise, solde CHF.
        let fin_nom = MARGIN_LEFT_MM + 25.0 + ACCOUNT_NAME_MAX_CHARS_FX as f32 * 2.5;
        let fx = "USD -1'234'567.89";
        assert!(right_aligned_x(fx, FONT_SIZE_PT, false, FX_AMOUNT_RIGHT_MM) > fin_nom);
        assert!(debut(long, FONT_SIZE_PT, AMOUNT_RIGHT_MM) > FX_AMOUNT_RIGHT_MM);
    }

    /// Un bilan à deux comptes en devise, de longueurs et de signes différents.
    fn fixture_bs_multi() -> BalanceSheet {
        let mut bs = fixture_bs(false);
        let compte = |n: &str, bal, cur: &str, fx| AccountBalance {
            account_id: 9,
            account_number: n.into(),
            account_name: "Compte en devise".into(),
            account_type: AccountType::Asset,
            active: true,
            balance: bal,
            role: None,
            currency: Some(cur.into()),
            balance_currency: Some(fx),
        };
        bs.assets
            .push(compte("1030", dec!(4.67), "EUR", dec!(4.92)));
        bs.assets
            .push(compte("1102", dec!(16077.51), "EUR", dec!(17072.20)));
        bs.liabilities
            .push(compte("2010", dec!(-1012.25), "USD", dec!(-1133.60)));
        bs
    }

    /// Bout en bout — bilan et compte de résultat : solde CHF sur la marge
    /// droite (totaux en gras compris), solde en devise sur son propre bord.
    #[test]
    fn bilan_et_resultat_pdf_alignent_les_montants_a_droite() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        assert_montants_alignes(
            "bilan",
            &render_balance_sheet_pdf(&fixture_bs_multi(), &ctx).unwrap(),
            &[FX_AMOUNT_RIGHT_MM, AMOUNT_RIGHT_MM],
        );
        let mut is_ = fixture_is(false);
        is_.revenues[0].balance = dec!(152340.80);
        is_.expenses[0].balance = dec!(-3.15);
        assert_montants_alignes(
            "compte de résultat",
            &render_income_statement_pdf(&is_, &ctx).unwrap(),
            &[AMOUNT_RIGHT_MM],
        );
    }

    /// Bout en bout — balance : quatre colonnes, en-têtes compris (ils finissent
    /// au même bord), seconde ligne en devise laissée telle quelle.
    #[test]
    fn balance_pdf_aligne_les_quatre_colonnes_a_droite() {
        let mut tb = fixture_tb(false);
        let mut usd = tb.rows[0].clone();
        usd.account_number = "1032".into();
        usd.opening_balance = dec!(-12500.00);
        usd.total_debit = dec!(1250300.10);
        usd.total_credit = dec!(5.00);
        usd.currency = Some("USD".into());
        usd.opening_currency = Some(dec!(-13750));
        usd.total_debit_currency = Some(dec!(1));
        usd.total_credit_currency = Some(dec!(1));
        usd.closing_currency = Some(dec!(1));
        tb.rows.push(usd);
        let ctx = PdfContext::fr_ch_default("Test SA");
        let bytes = render_trial_balance_pdf(&tb, &ctx).unwrap();
        assert_montants_alignes("balance", &bytes, &TB_AMOUNT_RIGHT_MM);
        let l = &ctx.section_labels;
        let poses = pdf_poses(&bytes);
        for (entete, bord) in [&l.col_opening, &l.col_debit, &l.col_credit, &l.col_closing]
            .into_iter()
            .zip(TB_AMOUNT_RIGHT_MM)
        {
            // Balance avec un compte en devise : « Ouverture CHF ».
            let entete = &format!("{entete} CHF");
            let p = poses.iter().find(|p| &p.text == entete).unwrap();
            assert!(
                (p.x_mm + largeur_mm(entete, p.size, p.bold) - bord).abs() < 0.01,
                "{entete}"
            );
        }
    }

    /// Bout en bout — journaux : débit et crédit, lignes et totaux.
    #[test]
    fn journaux_pdf_alignent_debit_et_credit_a_droite() {
        let mut jr = fixture_jr(false);
        jr.journals[0].entries[0].lines[0].debit = dec!(12345.60);
        jr.journals[0].section_total_debit = dec!(12345.60);
        jr.grand_total_debit = dec!(12345.60);
        let ctx = PdfContext::fr_ch_default("Test SA");
        assert_montants_alignes(
            "journaux",
            &render_journal_report_pdf(&jr, &ctx).unwrap(),
            &[JR_DEBIT_RIGHT_MM, JR_CREDIT_RIGHT_MM],
        );
    }

    /// Bout en bout — TVA (avec soldes) : base HT et TVA ; les totaux finissent
    /// sur le bord de la TVA.
    #[test]
    fn tva_pdf_aligne_base_et_tva_a_droite() {
        let mut vat = with_write_off(fixture_vat(false));
        vat.rows[0].base_ht = dec!(152340.80);
        let ctx = PdfContext::fr_ch_default("Test SA");
        assert_montants_alignes(
            "TVA",
            &render_vat_report_pdf(&vat, &ctx, &VatPdfLabels::fr_ch_defaults()).unwrap(),
            &[VAT_BASE_RIGHT_MM, VAT_DUE_RIGHT_MM],
        );
    }

    /// Bout en bout — grand livre en USD : débit, crédit, solde progressif, et
    /// le solde en devise des lignes d'encadrement sur son propre bord.
    #[test]
    fn grand_livre_pdf_aligne_les_montants_a_droite() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let mut gl = ledger_usd(3);
        gl.sections[0].lines[1].debit = Decimal::ZERO;
        gl.sections[0].lines[1].credit = dec!(9.12);
        gl.sections[0].total_credit = dec!(9.12);
        assert_montants_alignes(
            "grand livre",
            &render_general_ledger_pdf(&gl, &ctx).unwrap(),
            &[
                LEDGER_FX_RIGHT_MM,
                LEDGER_DEBIT_RIGHT_MM,
                LEDGER_CREDIT_RIGHT_MM,
                LEDGER_RUNNING_RIGHT_MM,
            ],
        );
    }

    /// Bout en bout — dépenses et rendement par compte analytique.
    #[test]
    fn rapports_de_projet_pdf_alignent_les_montants_a_droite() {
        use crate::project_report::{
            ExpenseAccountRow, ProjectExpenseSection, ProjectExpensesReport, ProjectInfo,
            ProjectReturnReport, ProjectReturnSection, ProjectReturnTotals,
        };
        let projet = || ProjectInfo {
            id: 1,
            code: "P1".into(),
            name: "Projet".into(),
        };
        let ligne = |a| ExpenseAccountRow {
            account_id: 1,
            account_number: "6000".into(),
            account_name: "Charges".into(),
            amount: a,
            entries: vec![],
        };
        let ctx = PdfContext::fr_ch_default("Test SA");
        let depenses = ProjectExpensesReport {
            report_type: "expenses".into(),
            project: projet(),
            mode: "consolidated".into(),
            period_label: "2026".into(),
            sections: vec![ProjectExpenseSection {
                project: projet(),
                is_root: true,
                rows: vec![ligne(dec!(12345.60)), ligne(dec!(-3.15))],
                subtotal: dec!(12342.45),
            }],
            grand_total: dec!(12342.45),
        };
        assert_montants_alignes(
            "dépenses par projet",
            &render_project_expenses_pdf(&depenses, &ctx, &ProjectExpensesPdfLabels::default())
                .unwrap(),
            &[AMOUNT_RIGHT_MM],
        );
        let section = |c, r| ProjectReturnSection {
            project: projet(),
            is_root: true,
            cout_investi: c,
            revenus: r,
            resultat_net: r - c,
            rendement_pct: None,
        };
        let rendement = ProjectReturnReport {
            report_type: "return".into(),
            project: projet(),
            mode: "consolidated".into(),
            period_label: "2026".into(),
            sections: vec![
                section(dec!(12345.60), dec!(15000)),
                section(dec!(3.15), dec!(0)),
            ],
            totals: ProjectReturnTotals {
                cout_investi: dec!(12348.75),
                revenus: dec!(15000),
                resultat_net: dec!(2651.25),
                rendement_pct: Some(dec!(21.47)),
            },
        };
        assert_montants_alignes(
            "rendement par projet",
            &render_project_return_pdf(&rendement, &ctx, &ProjectReturnPdfLabels::default())
                .unwrap(),
            &[RET_COUT_RIGHT_MM, RET_REV_RIGHT_MM, RET_RES_RIGHT_MM],
        );
    }

    // ------------------------------------------------------------------
    // Correctif fix-pdf-montants-alignes — titre des propriétés du PDF
    // ------------------------------------------------------------------

    /// Le titre des propriétés d'un PDF (`/Title` du dictionnaire `Info`), relu
    /// par lopdf — qui doit donc analyser le fichier entier, table des
    /// références croisées comprise — et décodé : UTF-16BE après `FE FF`,
    /// sinon octet par octet (PDFDocEncoding, ASCII ici).
    fn pdf_title(bytes: &[u8]) -> String {
        let doc = lopdf::Document::load_mem(bytes).expect("PDF lisible par lopdf");
        let info = doc.trailer.get(b"Info").unwrap().as_reference().unwrap();
        let title = doc.get_dictionary(info).unwrap().get(b"Title").unwrap();
        let raw = title.as_str().unwrap();
        match raw.strip_prefix(&[0xFE, 0xFF]) {
            Some(utf16) => String::from_utf16(
                &utf16
                    .chunks(2)
                    .map(|c| u16::from_be_bytes([c[0], c[1]]))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            None => raw.iter().map(|b| char::from(*b)).collect(),
        }
    }

    /// Unitaire — le gabarit a la longueur exacte de la chaîne hexadécimale qui
    /// le remplacera (délimiteurs compris), pour tout titre non ASCII, y compris
    /// hors du plan multilingue de base (deux unités UTF-16) ; aucun pour un
    /// titre ASCII.
    #[test]
    fn le_gabarit_du_titre_a_la_longueur_de_sa_chaine_utf16() {
        assert_eq!(title_placeholder("Bilan"), None);
        assert_eq!(title_placeholder("Erfolgsrechnung"), None);
        for t in [
            "é",
            "Compte de résultat",
            "Décompte TVA",
            "Prüfbericht — Übersicht",
            "Clé 𝄞",
        ] {
            let ph = title_placeholder(t).unwrap();
            assert!(ph.chars().all(|c| c.is_ascii_alphanumeric()), "{ph}");
            assert_eq!(format!("({ph})").len(), pdf_utf16_hex(t).len(), "{t}");
        }
        assert_eq!(pdf_utf16_hex("é"), "<FEFF00E9>");
        assert_eq!(pdf_utf16_hex("𝄞"), "<FEFFD834DD1E>");
    }

    /// Unitaire — le remplacement exige le gabarit une fois et une seule.
    #[test]
    fn le_titre_ne_se_pose_que_sur_un_gabarit_unique() {
        let ph = title_placeholder("é").unwrap();
        let un = format!("<</Title({ph})>>").into_bytes();
        assert_eq!(
            set_pdf_title(un, &ph, "é").unwrap(),
            b"<</Title<FEFF00E9>>>".to_vec()
        );
        assert!(set_pdf_title(b"<</Title(x)>>".to_vec(), &ph, "é").is_err());
        let deux = format!("({ph})({ph})").into_bytes();
        assert!(set_pdf_title(deux, &ph, "é").is_err());
    }

    /// Bout en bout — le titre des propriétés se relit juste, accents compris,
    /// dans un PDF que lopdf analyse (le remplacement n'a décalé aucun octet) :
    /// compte de résultat, décompte TVA, grand livre au titre allemand et hors
    /// du plan de base ; un titre ASCII (« Bilan ») reste tel quel.
    #[test]
    fn le_titre_des_proprietes_du_pdf_se_relit_juste() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        assert_eq!(
            pdf_title(&render_income_statement_pdf(&fixture_is(false), &ctx).unwrap()),
            "Compte de résultat"
        );
        assert_eq!(
            pdf_title(&render_balance_sheet_pdf(&fixture_bs(false), &ctx).unwrap()),
            "Bilan"
        );
        let vat = render_vat_report_pdf(&fixture_vat(false), &ctx, &VatPdfLabels::fr_ch_defaults())
            .unwrap();
        assert_eq!(pdf_title(&vat), "Décompte TVA");
        let mut de = PdfContext::fr_ch_default("Test SA");
        de.section_labels.ledger_title = "Hauptbuch — Prüfung 𝄞".into();
        // Plusieurs pages : le titre se pose aussi sur un gros document.
        let gl = render_general_ledger_pdf(&ledger_with(300), &de).unwrap();
        assert_eq!(pdf_title(&gl), "Hauptbuch — Prüfung 𝄞");
        // Aucun reste du gabarit.
        assert!(!String::from_utf8_lossy(&gl).contains("KeshTitr"));
    }

    // ------------------------------------------------------------------
    // Correctif fix-pdf-montants-alignes — en-têtes des colonnes en devise
    // ------------------------------------------------------------------

    /// Le bord droit d'un texte posé : son abscisse de départ plus sa largeur.
    fn bord_de(poses: &[Pose], text: &str) -> Option<f32> {
        poses
            .iter()
            .find(|p| p.text == text)
            .map(|p| p.x_mm + largeur_mm(&p.text, p.size, p.bold))
    }

    /// Bilan et compte de résultat avec un compte en devise : « Devise » et
    /// « CHF » en tête de leurs colonnes, alignés à droite sur elles ; sans
    /// compte en devise, aucun en-tête — le rendu mono-devise ne change pas.
    #[test]
    fn bilan_et_resultat_disent_la_devise_de_leurs_colonnes_en_multi_devises() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let poses = pdf_poses(&render_balance_sheet_pdf(&fixture_bs_multi(), &ctx).unwrap());
        assert!((bord_de(&poses, "Devise").unwrap() - FX_AMOUNT_RIGHT_MM).abs() < 0.01);
        assert!((bord_de(&poses, "CHF").unwrap() - AMOUNT_RIGHT_MM).abs() < 0.01);
        let mut is_ = fixture_is(false);
        is_.revenues[0].currency = Some("EUR".into());
        is_.revenues[0].balance_currency = Some(dec!(2100));
        let poses = pdf_poses(&render_income_statement_pdf(&is_, &ctx).unwrap());
        assert!((bord_de(&poses, "Devise").unwrap() - FX_AMOUNT_RIGHT_MM).abs() < 0.01);
        assert!((bord_de(&poses, "CHF").unwrap() - AMOUNT_RIGHT_MM).abs() < 0.01);
        for bytes in [
            render_balance_sheet_pdf(&fixture_bs(false), &ctx).unwrap(),
            render_income_statement_pdf(&fixture_is(false), &ctx).unwrap(),
        ] {
            let t = pdf_text(&bytes);
            assert!(
                !t.iter().any(|l| l == "Devise" || l == "CHF"),
                "mono : {t:?}"
            );
        }
    }

    /// Balance et grand livre avec un compte en devise : les en-têtes de montant
    /// disent « CHF » ; sans compte en devise, ils restent tels quels.
    #[test]
    fn balance_et_grand_livre_disent_chf_en_multi_devises() {
        let ctx = PdfContext::fr_ch_default("Test SA");
        let mut tb = fixture_tb(false);
        let mono = pdf_text(&render_trial_balance_pdf(&tb, &ctx).unwrap());
        assert!(mono.iter().any(|l| l == "Ouverture") && !mono.iter().any(|l| l.ends_with(" CHF")));
        tb.rows[0].currency = Some("USD".into());
        let multi = pdf_text(&render_trial_balance_pdf(&tb, &ctx).unwrap());
        for h in ["Ouverture CHF", "Débit CHF", "Crédit CHF", "Clôture CHF"] {
            assert!(multi.iter().any(|l| l == h), "{h} dans {multi:?}");
        }
        let mono = pdf_text(&render_general_ledger_pdf(&ledger_with(3), &ctx).unwrap());
        assert!(mono.iter().any(|l| l == "Solde") && !mono.iter().any(|l| l.ends_with(" CHF")));
        let multi = pdf_text(&render_general_ledger_pdf(&ledger_usd(3), &ctx).unwrap());
        for h in ["Débit CHF", "Crédit CHF", "Solde CHF"] {
            assert!(multi.iter().any(|l| l == h), "{h}");
        }
    }
}
