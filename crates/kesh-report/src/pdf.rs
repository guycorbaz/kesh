//! Sérialiseur PDF des 4 rapports comptables — Story 9-2a.
//!
//! 4 fonctions publiques pures (input DTO → `Result<Vec<u8>, ReportError>`) :
//! - [`render_balance_sheet_pdf`] — Bilan (Actifs / Passifs / résultat de l'exercice).
//! - [`render_income_statement_pdf`] — Compte de résultat (Produits / Charges / résultat net).
//! - [`render_trial_balance_pdf`] — Balance des comptes (5 colonnes : N°, Compte, Débit, Crédit, Solde).
//! - [`render_journal_report_pdf`] — Journaux (5 sections fixes, filtre optionnel).
//!
//! Architecture (Pass 1 AA-M1 + Decision §swiss-amount-format) :
//! - **Aucune dépendance sur `kesh-i18n`** — tous les libellés sont fournis via
//!   [`PdfContext`] résolu côté handler `kesh-api` (DD-14 pattern `kesh-qrbill`).
//! - Format suisse montants (`1'234.56`) + dates (`dd.mm.yyyy`) via helpers locaux.
//! - Helvetica builtin (cohérent kesh-qrbill, pas d'embedding TTF v0.1 — L3/L11).
//!
//! Pagination automatique (T2.3a) : page break à `cursor_y < MARGIN_BOTTOM_MM` +
//! footer `page X/Y` redrawn après finalisation.

use chrono::NaiveDate;
use printpdf::{
    BuiltinFont, IndirectFontRef, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference,
};
use rust_decimal::Decimal;

use crate::balance_sheet::{AccountBalance, BalanceSheet};
use crate::errors::ReportError;
use crate::income_statement::IncomeStatement;
use crate::journal_report::JournalReport;
use crate::trial_balance::TrialBalance;

// ============================================================================
// Constants centralisées — A4 portrait (T2.3a + Pass 3 ECH3-M3)
// ============================================================================

const PAGE_WIDTH_MM: f32 = 210.0; // A4 portrait
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_TOP_MM: f32 = 20.0;
const MARGIN_BOTTOM_MM: f32 = 25.0;
const MARGIN_LEFT_MM: f32 = 15.0;
const FONT_SIZE_PT: f32 = 10.0;
const FONT_SIZE_HEADER_PT: f32 = 14.0;
const FONT_SIZE_FOOTER_PT: f32 = 8.0;
const LINE_HEIGHT_MM: f32 = 4.2; // ≈ 12pt at 10pt font

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
    pub col_entry_date: String,
    pub col_description: String,
    // Header
    pub header_period: String,
    // Journaux
    pub journal_filter_prefix: String,
}

impl SectionLabels {
    /// Libellés par défaut FR-CH pour les tests unit / fallback.
    pub fn fr_ch_defaults() -> Self {
        Self {
            assets: "Actifs".into(),
            liabilities: "Passifs".into(),
            equity: "Capitaux propres".into(),
            equity_result_label: "Résultat de l'exercice".into(),
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
            col_entry_date: "Date".into(),
            col_description: "Libellé".into(),
            header_period: "Période".into(),
            journal_filter_prefix: "Journal".into(),
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

/// Formate une date au format suisse `dd.mm.yyyy` (T2.5).
pub fn format_swiss_date(d: NaiveDate) -> String {
    d.format("%d.%m.%Y").to_string()
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
}

impl PdfBuilder {
    fn new(title: &str) -> Result<Self, ReportError> {
        let (doc, page1, layer1) =
            PdfDocument::new(title, Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");
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

    /// Dessine le footer `page X/Y` sur toutes les pages après finalisation.
    fn draw_footers(&self) {
        let total = self.pages.len();
        for (i, (page_idx, layer_idx)) in self.pages.iter().enumerate() {
            let layer = self.doc.get_page(*page_idx).get_layer(*layer_idx);
            let footer = format!("page {} / {}", i + 1, total);
            layer.use_text(
                footer,
                FONT_SIZE_FOOTER_PT,
                Mm(PAGE_WIDTH_MM - 30.0),
                Mm(10.0),
                &self.font,
            );
        }
    }

    fn finalize(self) -> Result<Vec<u8>, ReportError> {
        self.draw_footers();
        self.doc
            .save_to_bytes()
            .map_err(|e| ReportError::PdfGeneration(format!("save: {e}")))
    }
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
fn draw_empty_message(builder: &mut PdfBuilder, ctx: &PdfContext) {
    // Position approximative au centre vertical
    builder.cursor_y = PAGE_HEIGHT_MM / 2.0;
    let layer = builder.current_layer();
    // X approximatif centré (Helvetica builtin n'expose pas de mesure de largeur,
    // donc on utilise une heuristique conservatrice).
    layer.use_text(
        &ctx.empty_message,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM + 30.0),
        Mm(builder.cursor_y),
        &builder.font,
    );
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
    layer.use_text(
        &ab.account_name,
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM + 25.0),
        Mm(y),
        &builder.font,
    );
    layer.use_text(
        format_swiss_amount(ab.balance),
        FONT_SIZE_PT,
        Mm(PAGE_WIDTH_MM - 45.0),
        Mm(y),
        &builder.font,
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
    layer.use_text(
        format_swiss_amount(amount),
        FONT_SIZE_PT,
        Mm(PAGE_WIDTH_MM - 45.0),
        Mm(y),
        &builder.font_bold,
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

    let is_empty = bs.assets.is_empty() && bs.liabilities.is_empty();
    if is_empty {
        draw_empty_message(&mut builder, ctx);
        return builder.finalize();
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

    // Section Capitaux propres (1 seule ligne synthétique — Pass 3 BH3-H2)
    builder.write_line(&ctx.section_labels.equity, FONT_SIZE_PT, true, 0.0);
    builder.ensure_space_for_row();
    {
        let y = builder.cursor_y;
        let layer = builder.current_layer();
        layer.use_text(
            &ctx.section_labels.equity_result_label,
            FONT_SIZE_PT,
            Mm(MARGIN_LEFT_MM + 25.0),
            Mm(y),
            &builder.font,
        );
        layer.use_text(
            format_swiss_amount(bs.equity_result),
            FONT_SIZE_PT,
            Mm(PAGE_WIDTH_MM - 45.0),
            Mm(y),
            &builder.font,
        );
    }
    builder.cursor_y -= LINE_HEIGHT_MM * 1.5;

    // Pied de page : équation bilan (AC #3)
    draw_totals_footer(
        &mut builder,
        &ctx.section_labels.total_assets,
        bs.total_assets,
    );
    let total_liab_eq = bs.total_liabilities + bs.equity_result;
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

/// Génère le PDF de la balance des comptes — 5 colonnes (AC #5).
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

    // En-têtes de colonnes (5 colonnes : N°, Compte, Débit, Crédit, Solde)
    let col_num_x = MARGIN_LEFT_MM;
    let col_name_x = MARGIN_LEFT_MM + 20.0;
    let col_debit_x = MARGIN_LEFT_MM + 90.0;
    let col_credit_x = MARGIN_LEFT_MM + 125.0;
    let col_balance_x = MARGIN_LEFT_MM + 160.0;

    {
        let y = builder.cursor_y;
        let layer = builder.current_layer();
        layer.use_text(
            &ctx.section_labels.col_account_number,
            FONT_SIZE_PT,
            Mm(col_num_x),
            Mm(y),
            &builder.font_bold,
        );
        layer.use_text(
            &ctx.section_labels.col_account_name,
            FONT_SIZE_PT,
            Mm(col_name_x),
            Mm(y),
            &builder.font_bold,
        );
        layer.use_text(
            &ctx.section_labels.col_debit,
            FONT_SIZE_PT,
            Mm(col_debit_x),
            Mm(y),
            &builder.font_bold,
        );
        layer.use_text(
            &ctx.section_labels.col_credit,
            FONT_SIZE_PT,
            Mm(col_credit_x),
            Mm(y),
            &builder.font_bold,
        );
        layer.use_text(
            &ctx.section_labels.col_balance,
            FONT_SIZE_PT,
            Mm(col_balance_x),
            Mm(y),
            &builder.font_bold,
        );
    }
    builder.cursor_y -= LINE_HEIGHT_MM * 1.5;

    // Lignes
    for row in &tb.rows {
        builder.ensure_space_for_row();
        let y = builder.cursor_y;
        let layer = builder.current_layer();
        layer.use_text(
            &row.account_number,
            FONT_SIZE_PT,
            Mm(col_num_x),
            Mm(y),
            &builder.font,
        );
        layer.use_text(
            &row.account_name,
            FONT_SIZE_PT,
            Mm(col_name_x),
            Mm(y),
            &builder.font,
        );
        layer.use_text(
            format_swiss_amount(row.total_debit),
            FONT_SIZE_PT,
            Mm(col_debit_x),
            Mm(y),
            &builder.font,
        );
        layer.use_text(
            format_swiss_amount(row.total_credit),
            FONT_SIZE_PT,
            Mm(col_credit_x),
            Mm(y),
            &builder.font,
        );
        layer.use_text(
            format_swiss_amount(row.balance),
            FONT_SIZE_PT,
            Mm(col_balance_x),
            Mm(y),
            &builder.font,
        );
        builder.cursor_y -= LINE_HEIGHT_MM;
    }

    // Totaux
    builder.cursor_y -= LINE_HEIGHT_MM * 0.5;
    builder.ensure_space_for_row();
    {
        let y = builder.cursor_y;
        let layer = builder.current_layer();
        layer.use_text(
            &ctx.section_labels.grand_total,
            FONT_SIZE_PT,
            Mm(col_num_x),
            Mm(y),
            &builder.font_bold,
        );
        layer.use_text(
            format_swiss_amount(tb.total_debit),
            FONT_SIZE_PT,
            Mm(col_debit_x),
            Mm(y),
            &builder.font_bold,
        );
        layer.use_text(
            format_swiss_amount(tb.total_credit),
            FONT_SIZE_PT,
            Mm(col_credit_x),
            Mm(y),
            &builder.font_bold,
        );
    }
    builder.cursor_y -= LINE_HEIGHT_MM;

    builder.finalize()
}

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
                layer.use_text(
                    format_swiss_amount(line.debit),
                    FONT_SIZE_PT,
                    Mm(MARGIN_LEFT_MM + 110.0),
                    Mm(y),
                    &builder.font,
                );
                layer.use_text(
                    format_swiss_amount(line.credit),
                    FONT_SIZE_PT,
                    Mm(MARGIN_LEFT_MM + 145.0),
                    Mm(y),
                    &builder.font,
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
        layer.use_text(
            format_swiss_amount(section.section_total_debit),
            FONT_SIZE_PT,
            Mm(MARGIN_LEFT_MM + 110.0),
            Mm(y),
            &builder.font_bold,
        );
        layer.use_text(
            format_swiss_amount(section.section_total_credit),
            FONT_SIZE_PT,
            Mm(MARGIN_LEFT_MM + 145.0),
            Mm(y),
            &builder.font_bold,
        );
        builder.cursor_y -= LINE_HEIGHT_MM * 1.5;
    }

    // Grand total
    builder.write_inline(
        &ctx.section_labels.grand_total,
        FONT_SIZE_PT,
        true,
        MARGIN_LEFT_MM,
    );
    let y = builder.cursor_y;
    let layer = builder.current_layer();
    layer.use_text(
        format_swiss_amount(jr.grand_total_debit),
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM + 110.0),
        Mm(y),
        &builder.font_bold,
    );
    layer.use_text(
        format_swiss_amount(jr.grand_total_credit),
        FONT_SIZE_PT,
        Mm(MARGIN_LEFT_MM + 145.0),
        Mm(y),
        &builder.font_bold,
    );
    builder.cursor_y -= LINE_HEIGHT_MM;

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
                }]
            },
            total_assets: if empty { Decimal::ZERO } else { dec!(1234.56) },
            total_liabilities: if empty { Decimal::ZERO } else { dec!(500.00) },
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
                    total_debit: dec!(2000),
                    total_credit: dec!(1000),
                    balance: dec!(1000),
                }]
            },
            total_debit: if empty { Decimal::ZERO } else { dec!(2000) },
            total_credit: if empty { Decimal::ZERO } else { dec!(1000) },
            balanced: true,
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

    #[test]
    fn trial_balance_pdf_starts_with_pdf_signature() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_trial_balance_pdf(&fixture_tb(false), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    #[test]
    fn journal_report_pdf_starts_with_pdf_signature() {
        let ctx = PdfContext::fr_ch_default("CI Test Company");
        let bytes = render_journal_report_pdf(&fixture_jr(false), &ctx).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
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
}
