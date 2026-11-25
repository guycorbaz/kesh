//! Sérialiseur CSV des 4 rapports comptables — Story 9-2a.
//!
//! Format RFC 4180 + extensions Excel suisse :
//! - UTF-8 + BOM (`\u{FEFF}` en tête) — pour Excel CH/DE qui détecte l'encodage via BOM.
//! - Séparateur `;` (PAS la virgule — convention Excel CH/DE).
//! - Terminator `\r\n` (CRLF).
//! - Format décimal ISO point + **toujours 2 décimales**, **arrondies au centime**
//!   (`kesh_core::types::format_two_decimals` : l'équidistant loin de zéro, comme à l'écran).
//! - Dates ISO 8601 `YYYY-MM-DD` (machine-readable).
//! - Escaping automatique (RFC 4180) géré par `csv 1.3`.
//!
//! Cas rapport vide (T3.5 + Pass 1 ECH-M1) : **écrit uniquement la ligne d'en-tête**
//! sans data rows. Le frontend détecte le cas vide via le nombre de lignes du blob.

use std::io::Write;

use kesh_core::csv::csv_sanitize;
use kesh_core::types::format_two_decimals;
use rust_decimal::Decimal;

use crate::aged_receivables::AgedReceivables;
use crate::balance_sheet::{AccountBalance, BalanceSheet};
use crate::errors::ReportError;
use crate::general_ledger::{FiscalYearBreak, GeneralLedger};
use crate::groupes::{
    AvecGroupes, GenreLigne, Grouping, LigneBalanceGroupee, LigneGroupeeAffichee, LigneRapport,
    PositionSousTotal, SectionsBilan, SectionsResultat,
};
use crate::income_statement::IncomeStatement;
use crate::journal_report::JournalReport;
use crate::pdf::format_exchange_rate;
use crate::trial_balance::TrialBalance;
use crate::vat_report::VatReport;

/// Octets UTF-8 du BOM (`\u{FEFF}`).
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Une cellule **texte** passée par l'échappement commun des CSV
/// ([`kesh_core::csv::csv_sanitize`], Story 37-5) : libellés d'écriture, noms
/// et numéros de compte, noms de contact et de projet, nom d'exercice. Jamais
/// un montant : `-12.50` doit rester un nombre pour le tableur.
fn txt(s: &str) -> String {
    csv_sanitize(s.to_owned())
}

/// Helper interne — construit un `csv::Writer` configuré (séparateur `;`, terminator CRLF).
fn make_writer<W: Write>(writer: W) -> csv::Writer<W> {
    csv::WriterBuilder::new()
        .delimiter(b';')
        .terminator(csv::Terminator::CRLF)
        .from_writer(writer)
}

/// Formate un Decimal au format ISO décimal point, **toujours 2 décimales** (T3.6 +
/// Pass 1 ECH-H5 — sinon Excel auto-type colonne mixte casse), **arrondies au
/// centime** loin de zéro (`10.0050` → `10.01`, Story 48-11, #1245).
fn format_amount_iso(amount: Decimal) -> String {
    format_two_decimals(amount)
}

/// Montant **en devise** (Story 37-4, AC-5) : la somme exacte (4 décimales
/// stockées), au format des colonnes CHF (point décimal, sans séparateur de
/// milliers : un tableur ne relit pas `1'233.57`) ; l'arrondi est celui de
/// [`format_amount_iso`].
fn format_currency_amount_iso(amount: Decimal) -> String {
    format_amount_iso(amount)
}

/// Écrit un enregistrement : les colonnes de base, puis, si le rapport contient
/// un compte en devise, ses colonnes en devise (Story 37-4, AC-5). Sans
/// colonnes en devise, l'enregistrement est celui d'avant, octet pour octet.
fn write_fx_record<W: Write>(
    wtr: &mut csv::Writer<W>,
    base: &[&str],
    extra: Option<Vec<String>>,
) -> Result<(), ReportError> {
    let mut rec: Vec<&str> = base.to_vec();
    if let Some(extra) = &extra {
        rec.extend(extra.iter().map(String::as_str));
    }
    wtr.write_record(&rec).map_err(map_csv_err)
}

/// En-têtes en devise du bilan et du compte de résultat (`None` sans compte en
/// devise dans le rapport).
fn balance_header_extra(fx: bool) -> Option<Vec<String>> {
    fx.then(|| vec!["Devise".to_string(), "SoldeDevise".to_string()])
}

/// Colonnes en devise d'une ligne du bilan ou du compte de résultat : la devise
/// et le solde d'un compte en devise, vides pour un compte CHF ou une ligne de
/// total (`ab = None`), absentes sans compte en devise dans le rapport.
fn balance_row_extra(ab: Option<&AccountBalance>, fx: bool) -> Option<Vec<String>> {
    fx.then(
        || match ab.and_then(|a| a.currency.as_ref().map(|c| (a, c))) {
            Some((a, cur)) => vec![
                cur.clone(),
                a.balance_currency
                    .map(format_currency_amount_iso)
                    .unwrap_or_default(),
            ],
            None => vec![String::new(); 2],
        },
    )
}

/// Une ligne de compte du bilan ou du compte de résultat.
fn write_account_balance<W: Write>(
    wtr: &mut csv::Writer<W>,
    section: &str,
    ab: &AccountBalance,
    fx: bool,
) -> Result<(), ReportError> {
    write_fx_record(
        wtr,
        &[
            section,
            &txt(&ab.account_number),
            &txt(&ab.account_name),
            &format_amount_iso(ab.balance),
        ],
        balance_row_extra(Some(ab), fx),
    )
}

/// Helper : écrit le BOM en tête puis renvoie le writer prêt à recevoir des records.
///
/// Pass 1 code-review H1 : mappe vers `CsvGeneration` (pas `PdfGeneration` —
/// sinon un échec d'écriture du BOM lors d'un export CSV apparaissait à tort
/// comme un « Échec génération PDF » côté UI).
fn write_bom<W: Write>(writer: &mut W) -> Result<(), ReportError> {
    writer
        .write_all(UTF8_BOM)
        .map_err(|e| ReportError::CsvGeneration(format!("csv BOM write: {e}")))
}

/// Génère le CSV du bilan (AC #12).
///
/// Colonnes : `Section;NumeroCompte;NomCompte;Solde` où
/// `Section ∈ {Actifs, Passifs, CapitauxPropres}`. La section `CapitauxPropres`
/// (Story 14-3c) liste d'abord les **comptes physiques de fonds propres**, dans
/// l'ordre par rôle fixé en backend (**sans en-têtes de groupe** — le format linéaire
/// une-ligne-par-compte porte le regroupement par l'ordre), puis **deux lignes
/// calculées** distinctes (Story 14-1) : « Résultat reporté (calculé) »
/// (`retained_earnings`, **omise quand elle est nulle** — correctif
/// fix-bilan-resultat-reporte) + « Résultat de l'exercice » (`equity_result`), enfin
/// `Total capitaux propres`. Les comptes de rôle equity **ne figurent plus** dans
/// `Passifs` (partition D2) — `Total passifs` = dettes réelles seules. Libellés
/// **hardcodés FR-CH** (i18n sérialiseurs déférée, limitation L1/L4/L11 — D3).
pub fn render_balance_sheet_csv<W: Write>(bs: &BalanceSheet, writer: W) -> Result<(), ReportError> {
    rendre_bilan_csv(bs, None, writer)
}

/// Génère le CSV du bilan **tel que l'écran le groupe** (Story 46-3, AC-5) :
/// avec `groupedRows` (46-1), deux colonnes après `Section` — `Niveau` (la
/// profondeur) et `TypeLigne` (`groupe`, `compte`, `propre`, `soustotal`) —
/// et une ligne par ligne groupée ; montants arrondis au centime
/// ([`kesh_core::types::format_two_decimals`], AC-6). Sans `groupedRows`, c'est
/// [`render_balance_sheet_csv`], à l'octet (AC-2).
pub fn render_grouped_balance_sheet_csv<W: Write>(
    report: &AvecGroupes<BalanceSheet, SectionsBilan>,
    writer: W,
) -> Result<(), ReportError> {
    rendre_bilan_csv(
        &report.rapport,
        report.grouping.as_ref().zip(report.grouped_rows.as_ref()),
        writer,
    )
}

/// La disposition des colonnes d'un CSV du bilan ou du compte de résultat :
/// groupé ou non (colonnes `Niveau;TypeLigne`), en devise ou non (colonnes
/// `Devise;SoldeDevise`), et le formateur de ses montants.
struct Disposition {
    /// Colonnes `Niveau;TypeLigne` (Story 46-3).
    groupe: bool,
    /// Colonnes `Devise;SoldeDevise` (Story 37-4).
    fx: bool,
}

impl Disposition {
    /// La ligne d'en-tête.
    fn entete<W: Write>(&self, wtr: &mut csv::Writer<W>) -> Result<(), ReportError> {
        self.ecrire(
            wtr,
            "Section",
            Some(["Niveau", "TypeLigne"]),
            ["NumeroCompte", "NomCompte", "Solde"],
            balance_header_extra(self.fx),
        )
    }

    /// Un enregistrement : `section`, puis, en mode groupé, les deux cellules
    /// `groupe` (vides si `None`), puis `reste`, puis les colonnes en devise.
    fn ecrire<W: Write>(
        &self,
        wtr: &mut csv::Writer<W>,
        section: &str,
        groupe: Option<[&str; 2]>,
        reste: [&str; 3],
        extra: Option<Vec<String>>,
    ) -> Result<(), ReportError> {
        let mut base: Vec<&str> = vec![section];
        if self.groupe {
            base.extend(groupe.unwrap_or(["", ""]));
        }
        base.extend(reste);
        write_fx_record(wtr, &base, extra)
    }

    /// Une ligne de total, ou une ligne calculée : `nom` dans `NomCompte`.
    fn total<W: Write>(
        &self,
        wtr: &mut csv::Writer<W>,
        section: &str,
        nom: &str,
        montant: Decimal,
    ) -> Result<(), ReportError> {
        self.ecrire(
            wtr,
            section,
            None,
            ["", nom, &format_amount_iso(montant)],
            balance_row_extra(None, self.fx),
        )
    }
}

/// `TypeLigne` d'une ligne groupée (Story 46-3, AC-5).
fn type_ligne(kind: GenreLigne) -> &'static str {
    match kind {
        GenreLigne::Groupe => "groupe",
        GenreLigne::Compte => "compte",
        GenreLigne::Propre => "propre",
        GenreLigne::SousTotal => "soustotal",
    }
}

/// Les cellules d'une ligne groupée, communes au bilan, au compte de résultat
/// (Story 46-3, AC-5) et à la balance (Story 46-6b).
struct CellulesGroupees {
    /// `Niveau` (la profondeur) et `TypeLigne`.
    groupe: [String; 2],
    /// Numéro et nom, neutralisés par `txt`.
    texte: [String; 2],
    /// Les montants arrondis, ou vides sur le titre d'un groupe ouvert dont le
    /// sous-total est en pied (sa ligne `soustotal` les porte).
    montants: Vec<String>,
    /// Les colonnes en devise, si le rapport en a (`fx`) : la devise et les
    /// montants en devise de la ligne qui en porte et écrit ses montants,
    /// vides sinon.
    devise: Option<Vec<String>>,
}

/// Les cellules de la ligne groupée `l` (règle de rendu de la 46-1,
/// [`LigneGroupeeAffichee::porte_montant`]), montants écrits par
/// [`format_amount_iso`] (arrondis au centime, comme à plat).
fn cellules_groupees<L: LigneGroupeeAffichee>(
    l: &L,
    position: PositionSousTotal,
    fx: bool,
) -> CellulesGroupees {
    let montant = l.porte_montant(position);
    let montants: Vec<String> = l
        .montants()
        .into_iter()
        .map(|v| {
            if montant {
                format_amount_iso(v)
            } else {
                String::new()
            }
        })
        .collect();
    let devise = fx.then(|| match l.devise() {
        Some((cur, en_devise)) if montant => std::iter::once(cur.to_string())
            .chain(en_devise.into_iter().map(format_amount_iso))
            .collect(),
        _ => vec![String::new(); montants.len() + 1],
    });
    CellulesGroupees {
        groupe: [
            l.profondeur().to_string(),
            type_ligne(l.genre()).to_string(),
        ],
        texte: [txt(l.numero()), txt(l.nom())],
        montants,
        devise,
    }
}

/// Les lignes groupées d'une section du bilan ou du compte de résultat
/// ([`cellules_groupees`]) : niveau, type, numéro, nom, solde, puis la devise
/// et le solde en devise.
fn write_grouped_rows<W: Write>(
    wtr: &mut csv::Writer<W>,
    d: &Disposition,
    section: &str,
    lignes: &[LigneRapport],
    position: PositionSousTotal,
) -> Result<(), ReportError> {
    for l in lignes {
        let c = cellules_groupees(l, position, d.fx);
        let [niveau, type_l] = &c.groupe;
        let [numero, nom] = &c.texte;
        d.ecrire(
            wtr,
            section,
            Some([niveau, type_l]),
            [numero, nom, &c.montants[0]],
            c.devise,
        )?;
    }
    Ok(())
}

/// La disposition d'un CSV du bilan ou du compte de résultat, à plat ou
/// groupé ; `fx` : une ligne (un compte à plat, une ligne groupée) porte une
/// devise.
fn disposition(groupe: bool, fx: bool) -> Disposition {
    Disposition { groupe, fx }
}

/// Le corps d'une section : à plat, une ligne par compte ; groupé, les lignes
/// groupées.
fn write_section_body<W: Write>(
    wtr: &mut csv::Writer<W>,
    d: &Disposition,
    section: &str,
    comptes: &[AccountBalance],
    groupes: Option<(&[LigneRapport], PositionSousTotal)>,
) -> Result<(), ReportError> {
    match groupes {
        None => {
            for ab in comptes {
                write_account_balance(wtr, section, ab, d.fx)?;
            }
            Ok(())
        }
        Some((lignes, position)) => write_grouped_rows(wtr, d, section, lignes, position),
    }
}

/// Le bilan, à plat (`groupes = None`) ou par groupes : un seul rendu, dont
/// seul le corps des sections diffère (Story 46-3, T4).
fn rendre_bilan_csv<W: Write>(
    bs: &BalanceSheet,
    groupes: Option<(&Grouping, &SectionsBilan)>,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    // Story 37-4, AC-5 : `Devise;SoldeDevise` seulement si un compte en devise
    // figure dans le rapport (groupé : si une ligne groupée en porte une) ; les
    // totaux restent en devise principale.
    let fx = match groupes {
        None => bs
            .assets
            .iter()
            .chain(&bs.liabilities)
            .chain(&bs.equity)
            .any(|a| a.currency.is_some()),
        Some((_, s)) => s
            .assets
            .iter()
            .chain(&s.liabilities)
            .chain(&s.equity)
            .any(|l| l.currency.is_some()),
    };
    let d = disposition(groupes.is_some(), fx);
    let section = |f: fn(&SectionsBilan) -> &[LigneRapport]| {
        groupes.map(|(g, s)| (f(s), g.subtotal_position))
    };
    d.entete(&mut wtr)?;

    // Cas rapport vide : header seul, pas de data rows (Pass 1 ECH-M1). Garde
    // centralisée `BalanceSheet::is_empty()` (Story 14-3c P1-F1) — inclut `equity` :
    // un reclassement pur entre comptes de fonds propres (section equity non vide,
    // actifs/passifs/virtuels nuls) n'est PAS vide, sinon on masquerait la section
    // Capitaux propres réellement peuplée.
    if bs.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    // Pass 1 code-review M6 (ECH1-M1) : guard chaque section indépendamment
    // — sinon un BS partiel (actifs vides mais passifs présents, ou inverse)
    // affiche un « Total actifs;;;0.00 » phantom alors qu'aucune ligne actif
    // n'a été émise. Idem section Passifs. Une section groupée est vide si et
    // seulement si sa section à plat l'est.
    if !bs.assets.is_empty() {
        write_section_body(&mut wtr, &d, "Actifs", &bs.assets, section(|s| &s.assets))?;
        d.total(&mut wtr, "Total actifs", "", bs.total_assets)?;
    }

    if !bs.liabilities.is_empty() {
        write_section_body(
            &mut wtr,
            &d,
            "Passifs",
            &bs.liabilities,
            section(|s| &s.liabilities),
        )?;
        d.total(&mut wtr, "Total passifs", "", bs.total_liabilities)?;
    }

    // Section Capitaux propres — présentation par rôle (Story 14-3c). D'abord les
    // comptes physiques de fonds propres, dans l'ordre par rôle fixé en backend
    // (une ligne par compte, l'ordre porte le regroupement — pas d'en-tête de groupe
    // dans le format linéaire CSV, AC-C) ; groupé, l'ordre de l'arbre (Story 46-3).
    // Puis les deux lignes **calculées** distinctes
    // (« Résultat reporté (calculé) » marquée calculée pour la distinguer d'un compte
    // physique de rôle RetainedEarnings — collision D1). Le report à-nouveau DOIT figurer
    // dès qu'il est non nul, sinon le bilan exporté est déséquilibré ; nul, il est omis
    // comme au PDF et à l'écran (`shows_retained_earnings_line`, correctif
    // fix-bilan-resultat-reporte) — sans effet sur le total ni l'équation.
    write_section_body(
        &mut wtr,
        &d,
        "CapitauxPropres",
        &bs.equity,
        section(|s| &s.equity),
    )?;
    if bs.shows_retained_earnings_line() {
        // Arrondie comme la règle qui la montre (revue P1, LOW-1).
        d.total(
            &mut wtr,
            "CapitauxPropres",
            "Résultat reporté (calculé)",
            bs.retained_earnings,
        )?;
    }
    d.total(
        &mut wtr,
        "CapitauxPropres",
        "Résultat de l'exercice",
        bs.equity_result,
    )?;

    // Total capitaux propres = comptes physiques + 2 lignes calculées.
    let total_equity_all = bs.total_equity + bs.retained_earnings + bs.equity_result;
    d.total(&mut wtr, "Total capitaux propres", "", total_equity_all)?;

    // Ligne finale : somme passifs (dettes) + capitaux propres — invariant restructuré
    // (14-3c) : `total_liabilities + total_equity + retained_earnings + equity_result`
    // = ancien total passifs + fonds propres.
    let total_liab_eq = bs.total_liabilities + total_equity_all;
    d.total(
        &mut wtr,
        "Total passifs + capitaux propres",
        "",
        total_liab_eq,
    )?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV du compte de résultat (AC #13).
///
/// Colonnes : `Section;NumeroCompte;NomCompte;Solde` où
/// `Section ∈ {Produits, Charges}`. Ligne finale `ResultatNet;;;<somme>`.
pub fn render_income_statement_csv<W: Write>(
    is_: &IncomeStatement,
    writer: W,
) -> Result<(), ReportError> {
    rendre_resultat_csv(is_, None, writer)
}

/// Génère le CSV du compte de résultat tel que l'écran le groupe (Story 46-3,
/// AC-5) : même régime que [`render_grouped_balance_sheet_csv`].
pub fn render_grouped_income_statement_csv<W: Write>(
    report: &AvecGroupes<IncomeStatement, SectionsResultat>,
    writer: W,
) -> Result<(), ReportError> {
    rendre_resultat_csv(
        &report.rapport,
        report.grouping.as_ref().zip(report.grouped_rows.as_ref()),
        writer,
    )
}

/// Le compte de résultat, à plat ou par groupes (Story 46-3, T4).
fn rendre_resultat_csv<W: Write>(
    is_: &IncomeStatement,
    groupes: Option<(&Grouping, &SectionsResultat)>,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    // Story 37-4, AC-5 : même règle de colonnes que le bilan.
    let fx = match groupes {
        None => is_
            .revenues
            .iter()
            .chain(&is_.expenses)
            .any(|a| a.currency.is_some()),
        Some((_, s)) => s
            .revenues
            .iter()
            .chain(&s.expenses)
            .any(|l| l.currency.is_some()),
    };
    let d = disposition(groupes.is_some(), fx);
    let section = |f: fn(&SectionsResultat) -> &[LigneRapport]| {
        groupes.map(|(g, s)| (f(s), g.subtotal_position))
    };
    d.entete(&mut wtr)?;

    if is_.revenues.is_empty() && is_.expenses.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    // Pass 1 code-review M6 (ECH1-M1) : guard chaque section indépendamment
    // pour éviter un « Total produits;;;0.00 » / « Total charges;;;0.00 »
    // phantom quand une seule des deux sections est non-vide.
    if !is_.revenues.is_empty() {
        write_section_body(
            &mut wtr,
            &d,
            "Produits",
            &is_.revenues,
            section(|s| &s.revenues),
        )?;
        d.total(&mut wtr, "Total produits", "", is_.total_revenues)?;
    }

    if !is_.expenses.is_empty() {
        write_section_body(
            &mut wtr,
            &d,
            "Charges",
            &is_.expenses,
            section(|s| &s.expenses),
        )?;
        d.total(&mut wtr, "Total charges", "", is_.total_expenses)?;
    }

    // Résultat net : utile même si une section est vide (= total_revenues -
    // total_expenses, donc reflète le rapport réel).
    d.total(&mut wtr, "ResultatNet", "", is_.net_result)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV de la balance des comptes (AC #14).
///
/// Colonnes : `NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture`
/// — Story 25-5-b (#385) : la colonne `Solde` (net des mouvements, pris pour un
/// solde de compte) est **remplacée** par l'ouverture et la clôture. Après les
/// comptes, la ligne calculée du résultat reporté (numéro vide, comme au bilan),
/// puis la ligne `Total` (mouvements ; ouverture et clôture vides).
pub fn render_trial_balance_csv<W: Write>(tb: &TrialBalance, writer: W) -> Result<(), ReportError> {
    rendre_balance_csv(tb, None, writer)
}

/// Génère le CSV de la balance **telle que l'écran la groupe** (Story 46-6b,
/// AC-2) : avec `groupedRows` (46-6a), deux colonnes en **tête** — `Niveau` et
/// `TypeLigne`, là où se placerait une colonne `Section`, que la balance n'a
/// pas — et une ligne par ligne groupée, aux mêmes valeurs et à la même règle
/// de rendu que le bilan ([`cellules_groupees`]) ; montants arrondis au
/// centime ([`kesh_core::types::format_two_decimals`]). Les colonnes en devise
/// paraissent si une ligne groupée porte une devise ; un groupe en devise
/// unique les remplit comme un compte. Les lignes du résultat reporté et
/// `Total` restent ce qu'elles sont, `Niveau;TypeLigne` vides. Sans
/// `groupedRows`, c'est [`render_trial_balance_csv`], à l'octet (AC-3).
pub fn render_grouped_trial_balance_csv<W: Write>(
    report: &AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>,
    writer: W,
) -> Result<(), ReportError> {
    rendre_balance_csv(
        &report.rapport,
        report.grouping.as_ref().zip(report.grouped_rows.as_deref()),
        writer,
    )
}

/// Un enregistrement de la balance : en mode groupé (`groupe`), les deux
/// cellules `tete` (`Niveau`, `TypeLigne`) d'abord ; puis `base` et les
/// colonnes en devise. À plat, l'enregistrement d'avant la Story 46-6b.
fn write_tb_record<W: Write>(
    wtr: &mut csv::Writer<W>,
    groupe: bool,
    tete: [&str; 2],
    base: &[&str],
    extra: Option<Vec<String>>,
) -> Result<(), ReportError> {
    let mut rec: Vec<&str> = Vec::with_capacity(base.len() + 2);
    if groupe {
        rec.extend(tete);
    }
    rec.extend_from_slice(base);
    write_fx_record(wtr, &rec, extra)
}

/// La balance, à plat (`groupes = None`) ou par groupes : un seul rendu, dont
/// seuls les colonnes de tête, le corps et l'arrondi diffèrent (Story 46-6b).
fn rendre_balance_csv<W: Write>(
    tb: &TrialBalance,
    groupes: Option<(&Grouping, &[LigneBalanceGroupee])>,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);
    let groupe = groupes.is_some();

    // Story 37-4, AC-5 : colonnes en devise seulement si un compte en devise
    // figure dans le rapport (groupé : si une ligne groupée en porte une, comme
    // au bilan) ; aucune ligne de total en devise (devises mêlées).
    let fx = match groupes {
        None => tb.rows.iter().any(|r| r.currency.is_some()),
        Some((_, lignes)) => lignes.iter().any(|l| l.currency.is_some()),
    };
    let vides = || fx.then(|| vec![String::new(); 5]);
    write_tb_record(
        &mut wtr,
        groupe,
        ["Niveau", "TypeLigne"],
        &[
            "NumeroCompte",
            "NomCompte",
            "SoldeOuverture",
            "TotalDebit",
            "TotalCredit",
            "SoldeCloture",
        ],
        fx.then(|| {
            [
                "Devise",
                "SoldeOuvertureDevise",
                "TotalDebitDevise",
                "TotalCreditDevise",
                "SoldeClotureDevise",
            ]
            .map(String::from)
            .to_vec()
        }),
    )?;

    // Cas rapport vide : en-tête seul, à plat comme groupé.
    if tb.rows.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    match groupes {
        None => {
            for row in &tb.rows {
                let extra = match &row.currency {
                    Some(cur) if fx => Some(vec![
                        cur.clone(),
                        row.opening_currency
                            .map(format_currency_amount_iso)
                            .unwrap_or_default(),
                        row.total_debit_currency
                            .map(format_currency_amount_iso)
                            .unwrap_or_default(),
                        row.total_credit_currency
                            .map(format_currency_amount_iso)
                            .unwrap_or_default(),
                        row.closing_currency
                            .map(format_currency_amount_iso)
                            .unwrap_or_default(),
                    ]),
                    _ => vides(),
                };
                write_fx_record(
                    &mut wtr,
                    &[
                        &txt(&row.account_number),
                        &txt(&row.account_name),
                        &format_amount_iso(row.opening_balance),
                        &format_amount_iso(row.total_debit),
                        &format_amount_iso(row.total_credit),
                        &format_amount_iso(row.closing_balance),
                    ],
                    extra,
                )?;
            }
        }
        Some((g, lignes)) => {
            for l in lignes {
                let c = cellules_groupees(l, g.subtotal_position, fx);
                let mut base: Vec<&str> = c.groupe.iter().map(String::as_str).collect();
                base.extend(c.texte.iter().map(String::as_str));
                base.extend(c.montants.iter().map(String::as_str));
                write_fx_record(&mut wtr, &base, c.devise)?;
            }
        }
    }

    // Ligne calculée : libellé fixe, comme au CSV du bilan.
    let retained = format_amount_iso(tb.retained_earnings);
    write_tb_record(
        &mut wtr,
        groupe,
        ["", ""],
        &[
            "",
            "Résultat reporté (calculé)",
            &retained,
            "",
            "",
            &retained,
        ],
        vides(),
    )?;

    write_tb_record(
        &mut wtr,
        groupe,
        ["", ""],
        &[
            "Total",
            "",
            "",
            &format_amount_iso(tb.total_debit),
            &format_amount_iso(tb.total_credit),
            "",
        ],
        vides(),
    )?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV des journaux (AC #15, #16).
///
/// Colonnes : `Journal;DateEcriture;NumeroEcriture;Description;NumeroCompte;NomCompte;Debit;Credit`.
/// Une ligne par `journal_entry_line`. Ordre déterminé par
/// `Journal ASC, entry_date ASC, journal_entry_id ASC, line_order ASC`
/// (déjà appliqué côté SQL Story 9-1).
pub fn render_journal_report_csv<W: Write>(
    jr: &JournalReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Journal",
        "DateEcriture",
        "NumeroEcriture",
        "Description",
        "NumeroCompte",
        "NomCompte",
        "Debit",
        "Credit",
    ])
    .map_err(map_csv_err)?;

    let is_empty = jr.journals.iter().all(|s| s.entries.is_empty());
    if is_empty {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    for section in &jr.journals {
        for entry in &section.entries {
            for line in &entry.lines {
                wtr.write_record([
                    section.journal.as_str(),
                    &entry.entry_date.format("%Y-%m-%d").to_string(),
                    &entry.entry_number.to_string(),
                    &txt(&entry.description),
                    &txt(&line.account_number),
                    &txt(&line.account_name),
                    &format_amount_iso(line.debit),
                    &format_amount_iso(line.credit),
                ])
                .map_err(map_csv_err)?;
            }
        }
    }

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Mappers d'erreur internes
// ---------------------------------------------------------------------------

/// Génère le CSV du rapport TVA (Story 11-2).
///
/// Colonnes : `Taux;ChiffreAffairesHT;TVADue` (une ligne par taux), puis lignes
/// récapitulatives (label en 1re colonne, montant dans la colonne naturelle) :
/// total CA HT, total TVA due, TVA récupérable (solde du compte impôt préalable,
/// Story 18-1d), solde. Rapport vide = en-tête seul, **sauf** si la TVA récupérable
/// est non nulle (cas « achats seuls » : on rend le récapitulatif).
pub fn render_vat_report_csv<W: Write>(
    report: &VatReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record(["Taux", "ChiffreAffairesHT", "TVADue"])
        .map_err(map_csv_err)?;

    // Cas rapport vide : header seul (Pass 1 ECH-M1, pattern par renderer).
    // Story 18-1d : un rapport sans vente (rows vide) mais avec de la TVA
    // récupérable (achats seuls) n'est PAS vide — il faut écrire le récapitulatif.
    // On ne court-circuite que si AUSSI récupérable == 0.
    // Story 25-4-d2c : un rapport sans vente ni récupérable mais avec des soldes
    // n'est pas vide non plus.
    if report.rows.is_empty()
        && report.total_vat_recoverable == Decimal::ZERO
        && report.write_off_rows.is_empty()
    {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    for row in &report.rows {
        wtr.write_record([
            &format_amount_iso(row.rate),
            &format_amount_iso(row.base_ht),
            &format_amount_iso(row.vat_due),
        ])
        .map_err(map_csv_err)?;
    }

    // Récapitulatif (montant dans sa colonne naturelle).
    wtr.write_record([
        "Total chiffre d'affaires HT",
        &format_amount_iso(report.total_base_ht),
        "",
    ])
    .map_err(map_csv_err)?;
    wtr.write_record([
        "Total TVA due",
        "",
        &format_amount_iso(report.total_vat_due),
    ])
    .map_err(map_csv_err)?;
    // Story 25-4-d2c : les diminutions de contre-prestation (soldes) — seulement
    // s'il y en a dans la période. Une ligne de TITRE (première colonne seule) les
    // distingue des lignes de vente, de même forme.
    if !report.write_off_rows.is_empty() {
        wtr.write_record(["Diminutions de contre-prestation (soldes)", "", ""])
            .map_err(map_csv_err)?;
        for row in &report.write_off_rows {
            wtr.write_record([
                &format_amount_iso(row.rate),
                &format_amount_iso(row.base_ht),
                &format_amount_iso(row.vat),
            ])
            .map_err(map_csv_err)?;
        }
        wtr.write_record([
            "Total TVA des soldes",
            "",
            &format_amount_iso(report.total_vat_write_off),
        ])
        .map_err(map_csv_err)?;
        wtr.write_record([
            "TVA due nette",
            "",
            &format_amount_iso(report.total_vat_due_net),
        ])
        .map_err(map_csv_err)?;
    }
    wtr.write_record([
        "TVA récupérable",
        "",
        &format_amount_iso(report.total_vat_recoverable),
    ])
    .map_err(map_csv_err)?;
    wtr.write_record(["Solde", "", &format_amount_iso(report.vat_balance)])
        .map_err(map_csv_err)?;
    // Story 18-1e : écart de réconciliation (TVA due dérivée − solde compte TVA due
    // au grand livre, périmètre ventes). Libellé en dur FR (i18n exports déférée v0.2).
    wtr.write_record([
        "Écart de réconciliation",
        "",
        &format_amount_iso(report.reconciliation_delta),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV de la balance âgée des créances clients (Story 21-7, AC 6).
///
/// Colonnes : `Contact;Non échu;1-30;31-60;61-90;90+;Total`. Une ligne par
/// contact, puis une ligne « Total général » (les `totals`). En-têtes **français
/// en dur** (patron `render_vat_report_csv` — les `render_*_csv` n'ont pas de
/// paramètre locale, i18n exports déférée v0.2). Rapport vide → en-tête seul.
pub fn render_aged_receivables_csv<W: Write>(
    report: &AgedReceivables,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Contact",
        "Non échu",
        "1-30",
        "31-60",
        "61-90",
        "90+",
        "Total",
    ])
    .map_err(map_csv_err)?;

    // Cas rapport vide : header seul, pas de data rows (pattern par renderer).
    if report.rows.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    for row in &report.rows {
        wtr.write_record([
            &txt(&row.contact_name),
            &format_amount_iso(row.buckets.not_due),
            &format_amount_iso(row.buckets.days_1_to_30),
            &format_amount_iso(row.buckets.days_31_to_60),
            &format_amount_iso(row.buckets.days_61_to_90),
            &format_amount_iso(row.buckets.days_over_90),
            &format_amount_iso(row.buckets.total),
        ])
        .map_err(map_csv_err)?;
    }

    // Ligne « Total général » (les totaux sommés en Rust).
    wtr.write_record([
        "Total général",
        &format_amount_iso(report.totals.not_due),
        &format_amount_iso(report.totals.days_1_to_30),
        &format_amount_iso(report.totals.days_31_to_60),
        &format_amount_iso(report.totals.days_61_to_90),
        &format_amount_iso(report.totals.days_over_90),
        &format_amount_iso(report.totals.total),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV du rapport « Dépenses par projet » (Story 19-6a).
///
/// Colonnes : `Projet;SousProjet;NumeroCompte;NomCompte;Montant`. Une ligne par
/// (section, compte), un sous-total par section, un total général. Le drill-down
/// (écritures) n'est PAS exporté en CSV (réservé à l'affichage/JSON).
pub fn render_project_expenses_csv<W: Write>(
    report: &crate::project_report::ProjectExpensesReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Projet",
        "SousProjet",
        "NumeroCompte",
        "NomCompte",
        "Montant",
    ])
    .map_err(map_csv_err)?;

    let root_label = txt(&format!(
        "{} — {}",
        report.project.code, report.project.name
    ));

    for section in &report.sections {
        let sub_label = if section.is_root {
            String::new()
        } else {
            txt(&format!(
                "{} — {}",
                section.project.code, section.project.name
            ))
        };
        for row in &section.rows {
            wtr.write_record([
                &root_label,
                &sub_label,
                &txt(&row.account_number),
                &txt(&row.account_name),
                &format_amount_iso(row.amount),
            ])
            .map_err(map_csv_err)?;
        }
        wtr.write_record([
            &root_label,
            &sub_label,
            "",
            "Sous-total",
            &format_amount_iso(section.subtotal),
        ])
        .map_err(map_csv_err)?;
    }

    wtr.write_record([
        &root_label,
        "",
        "",
        "Total dépenses",
        &format_amount_iso(report.grand_total),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Formate un rendement `Option<Decimal>` en `"xx.xx%"` ou `"—"` (Story 19-6b).
fn format_rendement(pct: Option<Decimal>) -> String {
    match pct {
        Some(p) => format!("{}%", format_two_decimals(p)),
        None => "—".to_string(),
    }
}

/// Génère le CSV du rapport « Rendement par projet » (Story 19-6b).
///
/// Colonnes : `Projet;SousProjet;CoutInvesti;Revenus;ResultatNet;RendementPct`.
/// Une ligne par section + une ligne total.
pub fn render_project_return_csv<W: Write>(
    report: &crate::project_report::ProjectReturnReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Projet",
        "SousProjet",
        "CoutInvesti",
        "Revenus",
        "ResultatNet",
        "RendementPct",
    ])
    .map_err(map_csv_err)?;

    let root_label = txt(&format!(
        "{} — {}",
        report.project.code, report.project.name
    ));
    for section in &report.sections {
        let sub_label = if section.is_root {
            String::new()
        } else {
            txt(&format!(
                "{} — {}",
                section.project.code, section.project.name
            ))
        };
        wtr.write_record([
            &root_label,
            &sub_label,
            &format_amount_iso(section.cout_investi),
            &format_amount_iso(section.revenus),
            &format_amount_iso(section.resultat_net),
            &format_rendement(section.rendement_pct),
        ])
        .map_err(map_csv_err)?;
    }
    wtr.write_record([
        &root_label,
        "Total",
        &format_amount_iso(report.totals.cout_investi),
        &format_amount_iso(report.totals.revenus),
        &format_amount_iso(report.totals.resultat_net),
        &format_rendement(report.totals.rendement_pct),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Pass 1 code-review H1 : mappent vers `CsvGeneration` (pas `PdfGeneration`).
/// Cf. commentaire `write_bom` ci-dessus pour le contexte.
fn map_csv_err(e: csv::Error) -> ReportError {
    ReportError::CsvGeneration(format!("csv write: {e}"))
}

fn map_io_err(e: std::io::Error) -> ReportError {
    ReportError::CsvGeneration(format!("csv io flush: {e}"))
}

/// Rend le grand livre en CSV.
///
/// ⚠️ **L'ouverture et la clôture sont des LIGNES, pas des métadonnées hors
/// tableau** : un CSV doit rester réconciliable dans un tableur, où l'on
/// additionne une colonne. Une ouverture reléguée en en-tête obligerait à la
/// ressaisir à la main. Les **ruptures d'exercice** aussi, sans débit ni
/// crédit : sur un compte de résultat dont la période franchit une fin
/// d'exercice, la clôture est l'ouverture plus les mouvements **moins** le solde
/// de chaque exercice clos (#778).
///
/// Une section par compte, séparée par une ligne vide — le compte est répété sur
/// chaque ligne pour que le fichier reste filtrable et triable.
pub fn render_general_ledger_csv<W: Write>(
    ledger: &GeneralLedger,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    // Story 37-4, AC-5 : six colonnes en devise, seulement si le rapport contient
    // un compte en devise — sinon la sortie reste celle d'avant, octet pour octet.
    let fx = ledger.sections.iter().any(|s| s.currency.is_some());
    write_fx_record(
        &mut wtr,
        &[
            "NumeroCompte",
            "NomCompte",
            "Date",
            "Piece",
            "Exercice",
            "Journal",
            "Libelle",
            "Contrepartie",
            "Debit",
            "Credit",
            "SoldeProgressif",
        ],
        fx.then(|| {
            [
                "Devise",
                "DebitDevise",
                "CreditDevise",
                "Cours",
                "SoldeProgressifDevise",
                "MentionDevise",
            ]
            .map(String::from)
            .to_vec()
        }),
    )?;

    for section in &ledger.sections {
        // Colonnes en devise d'une ligne de la section : vides pour un compte
        // CHF (dans un rapport qui en contient un en devise), absentes s'il n'y
        // en a aucun. `cols` = [débit, crédit, cours, solde, mention].
        let extra = |cols: [String; 5]| -> Option<Vec<String>> {
            fx.then(|| match &section.currency {
                Some(cur) => std::iter::once(cur.clone()).chain(cols).collect(),
                None => vec![String::new(); 6],
            })
        };
        let solde_fx = |v: Option<Decimal>| v.map(format_currency_amount_iso).unwrap_or_default();
        let vide = String::new;

        // Ligne d'ouverture — le report d'entrée de période.
        write_fx_record(
            &mut wtr,
            &[
                &txt(&section.account_number),
                &txt(&section.account_name),
                &ledger.period.from.format("%Y-%m-%d").to_string(),
                "",
                "",
                "",
                "Solde d'ouverture",
                "",
                "",
                "",
                &format_amount_iso(section.opening),
            ],
            extra([
                vide(),
                vide(),
                vide(),
                solde_fx(section.opening_currency),
                vide(),
            ]),
        )?;

        // Rupture d'exercice : une ligne sans débit ni crédit, qui porte le solde
        // de l'exercice clos. Sans elle, le solde progressif d'un compte de
        // résultat retomberait à zéro sans explication, et la clôture ne
        // s'expliquerait plus par l'ouverture et les mouvements (#777, #778).
        let write_break = |wtr: &mut csv::Writer<W>, brk: &FiscalYearBreak| {
            write_fx_record(
                wtr,
                &[
                    &txt(&section.account_number),
                    &txt(&section.account_name),
                    &brk.date.format("%Y-%m-%d").to_string(),
                    "",
                    "",
                    "",
                    "Clôture de l'exercice — le solde repart de zéro",
                    "",
                    "",
                    "",
                    &format_amount_iso(brk.closing_balance),
                ],
                extra([
                    vide(),
                    vide(),
                    vide(),
                    solde_fx(brk.closing_balance_currency),
                    vide(),
                ]),
            )
        };

        for (idx, line) in section.lines.iter().enumerate() {
            if let Some(brk) = section.break_before(idx) {
                write_break(&mut wtr, brk)?;
            }
            let cours = line.exchange_rate.map(format_exchange_rate);
            // Une réévaluation se lit « Réévaluation au cours X », sans montant
            // en devise (AC-2) — libellé littéral, comme les autres du CSV.
            let (debit_fx, credit_fx, mention) = if line.is_revaluation() {
                (
                    vide(),
                    vide(),
                    format!(
                        "Réévaluation au cours {}",
                        cours.clone().unwrap_or_default()
                    ),
                )
            } else {
                (
                    solde_fx(line.debit_currency),
                    solde_fx(line.credit_currency),
                    vide(),
                )
            };
            write_fx_record(
                &mut wtr,
                &[
                    &txt(&section.account_number),
                    &txt(&section.account_name),
                    &line.entry_date.format("%Y-%m-%d").to_string(),
                    &line.entry_number.to_string(),
                    &txt(&line.fiscal_year_name),
                    &line.journal,
                    &txt(&line.description),
                    &txt(&line.counterpart.join(" ")),
                    &format_amount_iso(line.debit),
                    &format_amount_iso(line.credit),
                    &format_amount_iso(line.running_balance),
                ],
                extra([
                    debit_fx,
                    credit_fx,
                    cours.unwrap_or_default(),
                    solde_fx(line.running_balance_currency),
                    mention,
                ]),
            )?;
        }
        for brk in section.trailing_breaks() {
            write_break(&mut wtr, brk)?;
        }

        // Total des mouvements, puis clôture.
        write_fx_record(
            &mut wtr,
            &[
                &txt(&section.account_number),
                &txt(&section.account_name),
                "",
                "",
                "",
                "",
                "Total des mouvements",
                "",
                &format_amount_iso(section.total_debit),
                &format_amount_iso(section.total_credit),
                "",
            ],
            extra([
                solde_fx(section.total_debit_currency),
                solde_fx(section.total_credit_currency),
                vide(),
                vide(),
                vide(),
            ]),
        )?;

        write_fx_record(
            &mut wtr,
            &[
                &txt(&section.account_number),
                &txt(&section.account_name),
                &ledger.period.to.format("%Y-%m-%d").to_string(),
                "",
                "",
                "",
                "Solde de clôture",
                "",
                "",
                "",
                &format_amount_iso(section.closing),
            ],
            extra([
                vide(),
                vide(),
                vide(),
                solde_fx(section.closing_currency),
                vide(),
            ]),
        )?;
    }

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

// ============================================================================
// Tests unit (T10.1)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance_sheet::AccountBalance;
    use crate::journal_report::{JournalEntryLineRow, JournalEntryRow, JournalSection};
    use crate::period::ReportPeriod;
    use crate::trial_balance::TrialBalanceRow;
    use chrono::NaiveDate;
    use kesh_db::entities::AccountType;
    use kesh_db::entities::journal_entry::Journal;
    use rust_decimal_macros::dec;

    fn period() -> ReportPeriod {
        ReportPeriod {
            fiscal_year_id: 1,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        }
    }

    // --- T10.1(b) : BOM + ; + CRLF pour les 4 CSV ---

    #[test]
    fn balance_sheet_csv_starts_with_bom() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity: vec![],
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: Decimal::ZERO,
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        assert_eq!(&buf[..3], UTF8_BOM, "CSV must start with UTF-8 BOM");
    }

    #[test]
    fn balance_sheet_csv_uses_semicolon_delimiter() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![AccountBalance {
                account_id: 1,
                account_number: "1000".into(),
                account_name: "Caisse".into(),
                account_type: AccountType::Asset,
                active: true,
                balance: dec!(100),
                role: None,
                currency: None,
                balance_currency: None,
            }],
            liabilities: vec![],
            equity: vec![],
            total_assets: dec!(100),
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: dec!(100),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let header = body.lines().next().unwrap();
        assert!(
            header.contains(';'),
            "CSV header must use `;` delimiter, got: {header}"
        );
        assert!(
            !header.contains(','),
            "CSV header must NOT use `,` (Excel CH/DE convention), got: {header}"
        );
    }

    #[test]
    fn income_statement_csv_uses_crlf_terminator() {
        let is_ = IncomeStatement {
            period: period(),
            revenues: vec![],
            expenses: vec![],
            total_revenues: Decimal::ZERO,
            total_expenses: Decimal::ZERO,
            net_result: Decimal::ZERO,
        };
        let mut buf = Vec::new();
        render_income_statement_csv(&is_, &mut buf).unwrap();
        // Vérifier qu'il existe au moins une ligne se terminant par \r\n
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(
            s.contains("\r\n"),
            "CSV must use CRLF terminator (RFC 4180), got first 50 bytes: {:?}",
            s.chars().take(50).collect::<String>()
        );
    }

    #[test]
    fn trial_balance_csv_bom_and_delimiters() {
        let tb = TrialBalance {
            period: period(),
            rows: vec![],
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
            balanced: true,
            retained_earnings: Decimal::ZERO,
            opening_balanced: true,
        };
        let mut buf = Vec::new();
        render_trial_balance_csv(&tb, &mut buf).unwrap();
        assert_eq!(&buf[..3], UTF8_BOM);
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(body.contains(';'));
        assert!(body.contains("\r\n"));
    }

    // --- T10.1(c) : escaping RFC 4180 — séparateur, quote, retour-ligne ---

    /// Pass 1 ECH-L4 (a) : nom contenant `;` doit être quoté.
    /// Pass 1 ECH-L4 (b) : nom contenant `"` doit doubler la quote → `""`.
    /// Pass 1 ECH-L4 (c) : nom contenant `\n` doit être quoté.
    #[test]
    fn csv_escapes_special_characters_per_rfc_4180() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![
                AccountBalance {
                    account_id: 1,
                    account_number: "1000".into(),
                    account_name: "Caisse; intérimaire".into(),
                    account_type: AccountType::Asset,
                    active: true,
                    balance: dec!(100),
                    role: None,
                    currency: None,
                    balance_currency: None,
                },
                AccountBalance {
                    account_id: 2,
                    account_number: "1001".into(),
                    account_name: r#"Nom avec "guillemets""#.into(),
                    account_type: AccountType::Asset,
                    active: true,
                    balance: dec!(50),
                    role: None,
                    currency: None,
                    balance_currency: None,
                },
                AccountBalance {
                    account_id: 3,
                    account_number: "1002".into(),
                    account_name: "Ligne 1\nLigne 2".into(),
                    account_type: AccountType::Asset,
                    active: true,
                    balance: dec!(25),
                    role: None,
                    currency: None,
                    balance_currency: None,
                },
            ],
            liabilities: vec![],
            equity: vec![],
            total_assets: dec!(175),
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: dec!(175),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        // (a) ; → quoté
        assert!(
            body.contains(r#""Caisse; intérimaire""#),
            "name containing `;` must be quoted, body:\n{body}"
        );
        // (b) " → doublé
        assert!(
            body.contains(r#""Nom avec ""guillemets""""#),
            "name containing `\"` must double-escape, body:\n{body}"
        );
        // (c) \n → une espace : l'échappement commun des cellules texte
        // (`kesh_core::csv::csv_sanitize`, Story 37-5) remplace CR, LF et TAB,
        // comme dans les autres exports CSV du produit.
        assert!(
            body.contains(";Ligne 1 Ligne 2;"),
            "name containing newline must come out on one line, body:\n{body}"
        );
    }

    // --- AC #17 + T3.5 : rapport vide = uniquement ligne d'en-tête ---

    #[test]
    fn empty_balance_sheet_csv_has_header_only() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity: vec![],
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: Decimal::ZERO,
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        // 1 ligne header + 0 data rows = exactement 1 CRLF terminé → 1 ligne effective
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(
            lines.len(),
            1,
            "empty CSV must have only header line, got: {lines:?}"
        );
        assert_eq!(lines[0], "Section;NumeroCompte;NomCompte;Solde");
    }

    /// Review Pass 1 — actif/passif vides mais report/résultat non nuls et opposés
    /// (bénéfice N entièrement dépensé en N+1) : la section fonds propres NE DOIT PAS
    /// être masquée (sinon bilan exporté déséquilibré). Régression du finding HIGH.
    #[test]
    fn balance_sheet_csv_shows_equity_when_assets_liabilities_net_to_zero() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity: vec![],
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: dec!(1000),
            equity_result: dec!(-1000),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(
            body.contains("Résultat reporté"),
            "la ligne « Résultat reporté » doit figurer, body:\n{body}"
        );
        assert!(
            body.contains("Résultat de l'exercice"),
            "la ligne « Résultat de l'exercice » doit figurer, body:\n{body}"
        );
        // Total passifs + capitaux propres = 0 + 1000 + (-1000) = 0 (équilibré, présent)
        assert!(
            body.contains("Total passifs + capitaux propres"),
            "le total fonds propres doit figurer, body:\n{body}"
        );
    }

    /// Story 14-3c — la section CapitauxPropres itemise les comptes physiques de fonds
    /// propres (dans l'ordre reçu), la ligne calculée est marquée « (calculé) » (D1), et
    /// les comptes de fonds propres ne figurent PAS dans les lignes Passifs (partition D2).
    #[test]
    fn balance_sheet_csv_equity_section_by_role() {
        let equity = |number: &str, name: &str, bal, role| AccountBalance {
            account_id: 1,
            account_number: number.into(),
            account_name: name.into(),
            account_type: AccountType::Liability,
            active: true,
            balance: bal,
            role: Some(role),
            currency: None,
            balance_currency: None,
        };
        let bs = BalanceSheet {
            period: period(),
            assets: vec![AccountBalance {
                account_id: 9,
                account_number: "1000".into(),
                account_name: "Banque".into(),
                account_type: AccountType::Asset,
                active: true,
                balance: dec!(3000),
                role: None,
                currency: None,
                balance_currency: None,
            }],
            liabilities: vec![AccountBalance {
                account_id: 8,
                account_number: "2000".into(),
                account_name: "Fournisseurs".into(),
                account_type: AccountType::Liability,
                active: true,
                balance: dec!(500),
                role: Some(kesh_db::entities::AccountRole::Payable),
                currency: None,
                balance_currency: None,
            }],
            equity: vec![
                equity(
                    "2800",
                    "Capital",
                    dec!(2000),
                    kesh_db::entities::AccountRole::EquityCapital,
                ),
                equity(
                    "2970",
                    "Bénéfice reporté",
                    dec!(500),
                    kesh_db::entities::AccountRole::RetainedEarnings,
                ),
            ],
            total_assets: dec!(3000),
            total_liabilities: dec!(500),
            total_equity: dec!(2500),
            retained_earnings: dec!(0),
            equity_result: dec!(0),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        // Comptes physiques itemisés dans la section CapitauxPropres.
        assert!(
            body.contains("CapitauxPropres;2800;Capital;2000.00"),
            "{body}"
        );
        assert!(
            body.contains("CapitauxPropres;2970;Bénéfice reporté;500.00"),
            "{body}"
        );
        // Ligne calculée nulle : omise (correctif fix-bilan-resultat-reporte) — seul le
        // compte physique 2970 porte le report, sans ligne « (calculé) » à 0.00 à côté.
        assert!(!body.contains("Résultat reporté (calculé)"), "{body}");
        assert!(body.contains("Total capitaux propres;;;2500.00"), "{body}");
        // Le capital NE figure PAS dans une ligne Passifs (partition D2).
        assert!(
            !body.contains("Passifs;2800"),
            "capital absent des passifs\n{body}"
        );
        // La dette réelle reste bien dans Passifs.
        assert!(body.contains("Passifs;2000;Fournisseurs;500.00"), "{body}");
        assert!(body.contains("Total passifs;;;500.00"), "{body}");

        // Même bilan, report calculé NON nul (collision D1 active) : la ligne calculée
        // est marquée « (calculé) », distincte du compte physique 2970, juste après les
        // comptes et avant « Résultat de l'exercice », et comptée au total.
        let mut bs = bs;
        bs.retained_earnings = dec!(300);
        bs.total_assets = dec!(3300);
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(
            body.contains(
                "CapitauxPropres;2970;Bénéfice reporté;500.00\r\n\
                 CapitauxPropres;;Résultat reporté (calculé);300.00\r\n\
                 CapitauxPropres;;Résultat de l'exercice;0.00\r\n"
            ),
            "{body}"
        );
        assert!(body.contains("Total capitaux propres;;;2800.00"), "{body}");
        assert!(
            body.contains("Total passifs + capitaux propres;;;3300.00"),
            "{body}"
        );

        // Revue P1, LOW-1 : la règle juge la valeur arrondie au centime. Sous le
        // demi-centime, la ligne est omise (ni « 0.00 » ni « -0.00 ») ; au
        // demi-centime, elle paraît, arrondie loin de zéro.
        for (retained, line) in [
            (dec!(0.004), None),
            (dec!(-0.004), None),
            (
                dec!(0.005),
                Some("CapitauxPropres;;Résultat reporté (calculé);0.01\r\n"),
            ),
            (
                dec!(-0.005),
                Some("CapitauxPropres;;Résultat reporté (calculé);-0.01\r\n"),
            ),
        ] {
            let mut bs = bs.clone();
            bs.retained_earnings = retained;
            let mut buf = Vec::new();
            render_balance_sheet_csv(&bs, &mut buf).unwrap();
            let body = String::from_utf8(buf[3..].to_vec()).unwrap();
            match line {
                Some(l) => assert!(body.contains(l), "{retained} : {body}"),
                None => assert!(!body.contains("Résultat reporté"), "{retained} : {body}"),
            }
            assert!(!body.contains(";-0.00"), "{retained} : {body}");
        }
    }

    #[test]
    fn empty_journal_report_csv_has_header_only() {
        let jr = JournalReport {
            period: period(),
            journals: vec![JournalSection {
                journal: Journal::Achats,
                entries: vec![],
                section_total_debit: Decimal::ZERO,
                section_total_credit: Decimal::ZERO,
            }],
            grand_total_debit: Decimal::ZERO,
            grand_total_credit: Decimal::ZERO,
        };
        let mut buf = Vec::new();
        render_journal_report_csv(&jr, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "empty journals CSV must have only header");
    }

    // --- T3.6 : format décimal toujours 2 décimales ---

    #[test]
    fn format_amount_iso_always_two_decimals() {
        assert_eq!(format_amount_iso(Decimal::ZERO), "0.00");
        assert_eq!(format_amount_iso(dec!(1234.5)), "1234.50");
        assert_eq!(format_amount_iso(dec!(-1234.56)), "-1234.56");
        assert_eq!(format_amount_iso(dec!(1000000)), "1000000.00");
    }

    // --- AC #15 : journals CSV contient toutes les colonnes attendues ---

    #[test]
    fn journal_report_csv_has_8_columns() {
        let jr = JournalReport {
            period: period(),
            journals: vec![JournalSection {
                journal: Journal::Ventes,
                entries: vec![JournalEntryRow {
                    entry_id: 1,
                    entry_number: 42,
                    entry_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
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
                }],
                section_total_debit: dec!(100),
                section_total_credit: Decimal::ZERO,
            }],
            grand_total_debit: dec!(100),
            grand_total_credit: Decimal::ZERO,
        };
        let mut buf = Vec::new();
        render_journal_report_csv(&jr, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert!(lines.len() >= 2);
        // Header : 8 colonnes
        let header_cols: Vec<&str> = lines[0].split(';').collect();
        assert_eq!(
            header_cols.len(),
            8,
            "journals CSV header must have 8 columns"
        );
        // Data row : 8 colonnes, date ISO, journal name
        let data_cols: Vec<&str> = lines[1].split(';').collect();
        assert_eq!(data_cols.len(), 8);
        assert_eq!(data_cols[0], "Ventes");
        assert_eq!(data_cols[1], "2026-05-15");
        assert_eq!(data_cols[2], "42");
    }

    // --- AC #14 : trial balance CSV header + total row ---

    #[test]
    fn trial_balance_csv_header_and_total_row() {
        let tb = TrialBalance {
            period: period(),
            rows: vec![TrialBalanceRow {
                account_id: 1,
                account_number: "1000".into(),
                account_name: "Caisse".into(),
                account_type: AccountType::Asset,
                active: true,
                opening_balance: dec!(1000),
                total_debit: dec!(200),
                total_credit: dec!(50),
                balance: dec!(150),
                closing_balance: dec!(1150),
                currency: None,
                opening_currency: None,
                total_debit_currency: None,
                total_credit_currency: None,
                balance_currency: None,
                closing_currency: None,
            }],
            total_debit: dec!(200),
            total_credit: dec!(50),
            balanced: true,
            retained_earnings: dec!(-600),
            opening_balanced: true,
        };
        let mut buf = Vec::new();
        render_trial_balance_csv(&tb, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(
            lines.len(),
            4,
            "1 header + 1 row + résultat reporté + 1 total = 4 lines"
        );
        assert_eq!(
            lines[0],
            "NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture"
        );
        // Story 25-5-b — l'ouverture et la clôture dans l'ordre des colonnes.
        assert_eq!(lines[1], "1000;Caisse;1000.00;200.00;50.00;1150.00");
        assert_eq!(lines[2], ";Résultat reporté (calculé);-600.00;;;-600.00");
        assert_eq!(lines[3], "Total;;;200.00;50.00;");
    }

    // Story 21-7 — CSV balance âgée.
    use crate::aged_receivables::{AgedBucket, AgedReceivables, AgedReceivablesRow};

    fn aged_bucket() -> AgedBucket {
        AgedBucket {
            not_due: dec!(110),
            days_1_to_30: dec!(205),
            days_31_to_60: dec!(307),
            days_61_to_90: dec!(409),
            days_over_90: dec!(1581),
            total: dec!(2612),
        }
    }

    #[test]
    fn aged_receivables_csv_bom_header_rows_total() {
        let report = AgedReceivables {
            as_of: chrono::NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
            rows: vec![AgedReceivablesRow {
                contact_id: 1,
                contact_name: "Alpha SA".into(),
                buckets: aged_bucket(),
            }],
            totals: aged_bucket(),
        };
        let mut buf = Vec::new();
        render_aged_receivables_csv(&report, &mut buf).unwrap();

        // BOM UTF-8 en tête.
        assert_eq!(&buf[..3], &[0xEF, 0xBB, 0xBF]);
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 3, "1 header + 1 contact + 1 total");
        assert_eq!(lines[0], "Contact;Non échu;1-30;31-60;61-90;90+;Total");
        assert_eq!(
            lines[1],
            "Alpha SA;110.00;205.00;307.00;409.00;1581.00;2612.00"
        );
        assert!(lines[2].starts_with("Total général;"));
        assert!(lines[2].ends_with(";2612.00"));
    }

    #[test]
    fn aged_receivables_csv_empty_is_header_only() {
        let report = AgedReceivables {
            as_of: chrono::NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
            rows: vec![],
            totals: AgedBucket {
                not_due: Decimal::ZERO,
                days_1_to_30: Decimal::ZERO,
                days_31_to_60: Decimal::ZERO,
                days_61_to_90: Decimal::ZERO,
                days_over_90: Decimal::ZERO,
                total: Decimal::ZERO,
            },
        };
        let mut buf = Vec::new();
        render_aged_receivables_csv(&report, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "rapport vide = en-tête seul");
        assert_eq!(lines[0], "Contact;Non échu;1-30;31-60;61-90;90+;Total");
    }

    // ------------------------------------------------------------------
    // Story 24-1 — grand livre
    // ------------------------------------------------------------------

    fn ledger_fixture() -> GeneralLedger {
        use crate::general_ledger::{GeneralLedger, LedgerLine, LedgerPeriod, LedgerSection};
        GeneralLedger {
            period: LedgerPeriod {
                from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                to: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            },
            sections: vec![LedgerSection {
                account_id: 7,
                account_number: "1020".into(),
                account_name: "Banque".into(),
                account_type: AccountType::Asset,
                active: true,
                balance_side: "debit",
                opening: dec!(500.00),
                lines: vec![LedgerLine {
                    line_id: 1,
                    entry_id: 1,
                    entry_date: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
                    fiscal_year_id: 3,
                    fiscal_year_name: "Exercice 2026".into(),
                    entry_number: 12,
                    journal: "Banque".into(),
                    description: "Loyer mars".into(),
                    counterpart: vec!["6000".into()],
                    debit: dec!(1200.00),
                    credit: dec!(0),
                    running_balance: dec!(1700.00),
                    // Lettrée : l'export CSV n'en écrit pas la colonne (AC6 de
                    // la 15-1b) — les colonnes restent celles d'avant.
                    lettering_code: Some("AB".into()),
                    debit_currency: None,
                    credit_currency: None,
                    exchange_rate: None,
                    rate_source: None,
                    running_balance_currency: None,
                }],
                total_debit: dec!(1200.00),
                total_credit: dec!(0),
                closing: dec!(1700.00),
                unnatural_balance: false,
                fiscal_year_breaks: vec![],
                line_count: 1,
                currency: None,
                opening_currency: None,
                total_debit_currency: None,
                total_credit_currency: None,
                closing_currency: None,
            }],
        }
    }

    /// ⚠️ Ouverture et clôture sont des **lignes du tableau**, pas des
    /// métadonnées d'en-tête : c'est ce qui rend le fichier réconciliable dans
    /// un tableur, où l'on additionne une colonne.
    #[test]
    fn general_ledger_csv_writes_opening_and_closing_as_rows() {
        let mut buf = Vec::new();
        render_general_ledger_csv(&ledger_fixture(), &mut buf).expect("csv");
        let out = String::from_utf8(buf).expect("utf-8");
        let lines: Vec<&str> = out.lines().collect();

        assert!(lines[0].starts_with('\u{feff}') || lines[0].contains("NumeroCompte"));
        let corps = out.replace('\u{feff}', "");
        let l: Vec<&str> = corps.lines().collect();
        assert!(l[1].contains("Solde d'ouverture"), "ligne 1 = {}", l[1]);
        assert!(l[1].ends_with("500.00"));
        assert!(l[2].contains("Loyer mars"));
        assert!(l[3].contains("Total des mouvements"));
        assert!(l[4].contains("Solde de clôture"));
        assert!(l[4].ends_with("1700.00"));
    }

    /// #777 / #778 — les ruptures d'exercice sont des lignes du tableau : celle
    /// qui précède la première ligne (l'ouverture appartient à l'exercice
    /// précédent), et celle qu'aucune ligne ne suit. Données synthétiques.
    #[test]
    fn general_ledger_csv_writes_fiscal_year_breaks_as_rows() {
        use crate::general_ledger::FiscalYearBreak;
        let mut gl = ledger_fixture();
        let s = &mut gl.sections[0];
        s.fiscal_year_breaks = vec![
            FiscalYearBreak {
                date: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
                closing_fiscal_year_id: 2,
                closing_balance: dec!(500.00),
                closing_balance_currency: None,
            },
            FiscalYearBreak {
                date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                closing_fiscal_year_id: 3,
                closing_balance: dec!(1200.00),
                closing_balance_currency: None,
            },
        ];
        s.lines[0].running_balance = dec!(1200.00);
        s.closing = dec!(0);
        let mut buf = Vec::new();
        render_general_ledger_csv(&gl, &mut buf).expect("csv");
        let corps = String::from_utf8(buf)
            .expect("utf-8")
            .replace('\u{feff}', "");
        let l: Vec<&str> = corps.lines().collect();
        assert!(l[1].contains("Solde d'ouverture"), "{}", l[1]);
        assert!(
            l[2].contains("2026-03-04"),
            "rupture avant la ligne : {}",
            l[2]
        );
        assert!(l[2].contains("Clôture de l'exercice"), "{}", l[2]);
        assert!(l[2].ends_with("500.00"), "{}", l[2]);
        assert!(l[3].contains("Loyer mars"), "{}", l[3]);
        assert!(l[4].contains("2027-01-01"), "rupture finale : {}", l[4]);
        assert!(l[4].ends_with("1200.00"), "{}", l[4]);
        assert!(l[5].contains("Total des mouvements"), "{}", l[5]);
        assert!(l[6].contains("Solde de clôture"), "{}", l[6]);
        assert!(l[6].ends_with(";0.00"), "{}", l[6]);
    }

    /// La colonne « Exercice » porte le **nom** de l'exercice, jamais son id —
    /// un identifiant de base de données ne dit rien à qui lit le fichier, et
    /// c'est pourtant ce qui désambiguïse le numéro de pièce.
    #[test]
    fn general_ledger_csv_names_the_fiscal_year_instead_of_its_id() {
        let mut buf = Vec::new();
        render_general_ledger_csv(&ledger_fixture(), &mut buf).expect("csv");
        let out = String::from_utf8(buf).expect("utf-8");
        assert!(out.contains("Exercice 2026"), "le nom manque : {out}");
        // L'id (3) ne doit pas s'être glissé dans la colonne Exercice.
        assert!(!out.contains(";12;3;"), "l'id d'exercice est rendu : {out}");
    }

    // --- Story 25-4-d2c — les soldes dans le CSV du rapport TVA ---

    fn vat_report(rows: bool, write_offs: bool) -> crate::vat_report::VatReport {
        use crate::vat_report::{VatReport, VatReportRow, VatWriteOffRow};
        let rows = if rows {
            vec![VatReportRow {
                rate: dec!(8.10),
                category: None,
                base_ht: dec!(1000.00),
                vat_due: dec!(81.00),
            }]
        } else {
            vec![]
        };
        let total_vat_due: Decimal = rows.iter().map(|r| r.vat_due).sum();
        let write_off_rows = if write_offs {
            vec![VatWriteOffRow {
                rate: dec!(8.10),
                base_ht: dec!(20.00),
                vat: dec!(1.62),
            }]
        } else {
            vec![]
        };
        let total_vat_write_off: Decimal = write_off_rows.iter().map(|r| r.vat).sum();
        VatReport {
            period: period(),
            total_base_ht: rows.iter().map(|r| r.base_ht).sum(),
            rows,
            total_vat_due,
            write_off_rows,
            total_vat_write_off,
            total_vat_due_net: total_vat_due - total_vat_write_off,
            total_vat_recoverable: Decimal::ZERO,
            vat_balance: total_vat_due - total_vat_write_off,
            reconciliation_delta: Decimal::ZERO,
            reconciliation_status: "ok".to_string(),
        }
    }

    fn csv_of(report: &crate::vat_report::VatReport) -> String {
        let mut buf = Vec::new();
        render_vat_report_csv(report, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn vat_csv_writes_the_write_off_section_with_a_title_row() {
        let csv = csv_of(&vat_report(true, true));
        assert!(
            csv.contains("Diminutions de contre-prestation (soldes);;"),
            "ligne de titre, première colonne seule : {csv}"
        );
        assert!(csv.contains("8.10;20.00;1.62"), "ligne par taux : {csv}");
        assert!(csv.contains("Total TVA des soldes;;1.62"), "{csv}");
        assert!(csv.contains("TVA due nette;;79.38"), "{csv}");
        assert!(
            csv.contains("Solde;;79.38"),
            "le solde est sur le net : {csv}"
        );
    }

    #[test]
    fn vat_csv_without_write_offs_is_unchanged() {
        let csv = csv_of(&vat_report(true, false));
        assert!(!csv.contains("Diminutions"), "{csv}");
        assert!(!csv.contains("TVA due nette"), "{csv}");
    }

    #[test]
    fn vat_csv_with_write_offs_only_is_not_empty() {
        // Garde « vide » (`render_vat_report_csv`) : ni vente ni récupérable,
        // mais un solde → pas le seul en-tête.
        let csv = csv_of(&vat_report(false, true));
        assert!(csv.contains("Total TVA des soldes;;1.62"), "{csv}");
    }

    // ------------------------------------------------------------------
    // Story 37-5 — échappement commun des cellules texte
    // ------------------------------------------------------------------

    /// Les cellules texte du journal (libellé, numéro et nom de compte)
    /// sortent échappées ; les montants, même négatifs, restent des nombres.
    #[test]
    fn journal_csv_echappe_les_cellules_texte_pas_les_montants() {
        let ligne = |id: i64, num: &str, nom: &str, debit, credit| JournalEntryLineRow {
            account_id: id,
            account_number: num.into(),
            account_name: nom.into(),
            debit,
            credit,
            line_order: 0,
            currency: None,
            amount_currency: None,
            exchange_rate: None,
            rate_source: None,
        };
        let jr = JournalReport {
            period: period(),
            journals: vec![JournalSection {
                journal: Journal::OD,
                entries: vec![JournalEntryRow {
                    entry_id: 1,
                    entry_number: 7,
                    entry_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
                    description: "=HYPERLINK(\"x\")".into(),
                    lines: vec![
                        ligne(1, "1000", "+Caisse", dec!(-12.50), Decimal::ZERO),
                        ligne(2, "2000", "@Fournisseur", Decimal::ZERO, dec!(-12.50)),
                        ligne(3, "3000", "\tTab", Decimal::ZERO, Decimal::ZERO),
                        ligne(4, "4000", "\rRetour", Decimal::ZERO, Decimal::ZERO),
                        ligne(5, "5000", "-Moins", Decimal::ZERO, Decimal::ZERO),
                    ],
                }],
                section_total_debit: dec!(-12.50),
                section_total_credit: dec!(-12.50),
            }],
            grand_total_debit: dec!(-12.50),
            grand_total_credit: dec!(-12.50),
        };
        let mut buf = Vec::new();
        render_journal_report_csv(&jr, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lignes: Vec<Vec<&str>> = body
            .split("\r\n")
            .filter(|l| !l.is_empty())
            .skip(1)
            .map(|l| l.split(';').collect())
            .collect();
        assert_eq!(lignes.len(), 5, "{body}");
        assert_eq!(lignes[0][3], "\"'=HYPERLINK(\"\"x\"\")\"", "{body}");
        assert_eq!(lignes[0][5], "'+Caisse");
        assert_eq!(lignes[1][5], "'@Fournisseur");
        assert_eq!(lignes[2][5], " Tab", "la tabulation de tête tombe");
        assert_eq!(lignes[3][5], " Retour", "le retour chariot de tête tombe");
        assert_eq!(lignes[4][5], "'-Moins");
        // Montants négatifs : des nombres, sans apostrophe.
        assert_eq!(lignes[0][6], "-12.50");
        assert_eq!(lignes[1][7], "-12.50");
    }

    /// Bilan, balance et grand livre : noms de compte, libellé, exercice et
    /// contrepartie échappés ; soldes négatifs intacts.
    #[test]
    fn bilan_balance_et_grand_livre_echappent_les_cellules_texte() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![AccountBalance {
                account_id: 1,
                account_number: "1000".into(),
                account_name: "=Caisse".into(),
                account_type: AccountType::Asset,
                active: true,
                balance: dec!(-5.00),
                role: None,
                currency: None,
                balance_currency: None,
            }],
            liabilities: vec![],
            equity: vec![],
            total_assets: dec!(-5.00),
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: dec!(-5.00),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(body.contains(";1000;'=Caisse;-5.00"), "{body}");

        let tb = TrialBalance {
            period: period(),
            rows: vec![TrialBalanceRow {
                account_id: 1,
                account_number: "1000".into(),
                account_name: "-Caisse".into(),
                account_type: AccountType::Asset,
                active: true,
                opening_balance: dec!(-10),
                total_debit: dec!(0),
                total_credit: dec!(0),
                balance: dec!(0),
                closing_balance: dec!(-10),
                currency: None,
                opening_currency: None,
                total_debit_currency: None,
                total_credit_currency: None,
                balance_currency: None,
                closing_currency: None,
            }],
            total_debit: dec!(0),
            total_credit: dec!(0),
            balanced: true,
            retained_earnings: Decimal::ZERO,
            opening_balanced: true,
        };
        let mut buf = Vec::new();
        render_trial_balance_csv(&tb, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(
            body.contains("1000;'-Caisse;-10.00;0.00;0.00;-10.00"),
            "{body}"
        );

        let mut gl = ledger_fixture();
        gl.sections[0].account_name = "+Banque".into();
        gl.sections[0].lines[0].description = "@Loyer".into();
        gl.sections[0].lines[0].fiscal_year_name = "-2026".into();
        gl.sections[0].lines[0].counterpart = vec!["=6000".into()];
        gl.sections[0].lines[0].running_balance = dec!(-1.00);
        let mut buf = Vec::new();
        render_general_ledger_csv(&gl, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap().replace('\u{feff}', "");
        let l: Vec<&str> = out.lines().collect();
        assert!(l[2].starts_with("1020;'+Banque;"), "{}", l[2]);
        assert!(l[2].contains(";'-2026;"), "{}", l[2]);
        assert!(l[2].contains(";'@Loyer;'=6000;"), "{}", l[2]);
        assert!(l[2].ends_with(";-1.00"), "{}", l[2]);
    }

    /// Balance âgée : le nom du contact sort échappé (formule neutralisée,
    /// retour à la ligne et tabulation remplacés par une espace) ; les
    /// montants, même négatifs, restent des nombres.
    #[test]
    fn balance_agee_echappe_le_nom_du_contact_pas_les_montants() {
        let negatif = AgedBucket {
            not_due: dec!(-12.50),
            days_1_to_30: Decimal::ZERO,
            days_31_to_60: Decimal::ZERO,
            days_61_to_90: Decimal::ZERO,
            days_over_90: Decimal::ZERO,
            total: dec!(-12.50),
        };
        let contact = |id: i64, nom: &str| AgedReceivablesRow {
            contact_id: id,
            contact_name: nom.into(),
            buckets: negatif.clone(),
        };
        let report = AgedReceivables {
            as_of: NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
            rows: vec![
                contact(1, "=HYPERLINK(\"x\")"),
                contact(2, "+Alpha"),
                contact(3, "-Beta"),
                contact(4, "@Gamma"),
                contact(5, "Delta\nSA\tSàrl"),
                contact(6, " =Espace"),
            ],
            totals: negatif.clone(),
        };
        let mut buf = Vec::new();
        render_aged_receivables_csv(&report, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lignes: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lignes.len(), 8, "en-tête + 6 contacts + total : {body}");
        assert!(
            lignes[1].starts_with("\"'=HYPERLINK(\"\"x\"\")\";"),
            "{}",
            lignes[1]
        );
        assert!(lignes[2].starts_with("'+Alpha;"), "{}", lignes[2]);
        assert!(lignes[3].starts_with("'-Beta;"), "{}", lignes[3]);
        assert!(lignes[4].starts_with("'@Gamma;"), "{}", lignes[4]);
        assert!(lignes[5].starts_with("Delta SA Sàrl;"), "{}", lignes[5]);
        assert!(lignes[6].starts_with("' =Espace;"), "{}", lignes[6]);
        for l in &lignes[1..] {
            assert!(l.ends_with(";-12.50"), "montant négatif intact : {l}");
        }
    }

    fn projet(id: i64, code: &str, nom: &str) -> crate::project_report::ProjectInfo {
        crate::project_report::ProjectInfo {
            id,
            code: code.into(),
            name: nom.into(),
        }
    }

    /// Dépenses par projet : libellés de projet et de sous-projet, numéro et
    /// nom de compte échappés ; montants négatifs intacts.
    #[test]
    fn depenses_par_projet_echappent_les_cellules_texte() {
        use crate::project_report::{
            ExpenseAccountRow, ProjectExpenseSection, ProjectExpensesReport,
        };
        let ligne = |id: i64, num: &str, nom: &str| ExpenseAccountRow {
            account_id: id,
            account_number: num.into(),
            account_name: nom.into(),
            amount: dec!(-7.25),
            entries: vec![],
        };
        let report = ProjectExpensesReport {
            report_type: "expenses".into(),
            project: projet(1, "=P1", "Racine"),
            mode: "fiscal_year".into(),
            period_label: "2026".into(),
            sections: vec![
                ProjectExpenseSection {
                    project: projet(1, "=P1", "Racine"),
                    is_root: true,
                    rows: vec![ligne(10, "+6000", "Loyer\nbureau")],
                    subtotal: dec!(-7.25),
                },
                ProjectExpenseSection {
                    project: projet(2, "@S1", "Sous\tprojet"),
                    is_root: false,
                    rows: vec![ligne(11, "6100", "-Frais")],
                    subtotal: dec!(-7.25),
                },
            ],
            grand_total: dec!(-14.50),
        };
        let mut buf = Vec::new();
        render_project_expenses_csv(&report, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lignes: Vec<Vec<&str>> = body
            .split("\r\n")
            .filter(|l| !l.is_empty())
            .skip(1)
            .map(|l| l.split(';').collect())
            .collect();
        assert_eq!(
            lignes.len(),
            5,
            "2 comptes, 2 sous-totaux, 1 total : {body}"
        );
        assert_eq!(
            lignes[0],
            ["'=P1 — Racine", "", "'+6000", "Loyer bureau", "-7.25"]
        );
        assert_eq!(lignes[1][3], "Sous-total");
        assert_eq!(
            lignes[2],
            [
                "'=P1 — Racine",
                "'@S1 — Sous projet",
                "6100",
                "'-Frais",
                "-7.25"
            ]
        );
        assert_eq!(
            lignes[4],
            ["'=P1 — Racine", "", "", "Total dépenses", "-14.50"]
        );
    }

    /// Rendement par projet : libellés échappés ; montants négatifs et
    /// pourcentages intacts.
    #[test]
    fn rendement_par_projet_echappe_les_libelles() {
        use crate::project_report::{
            ProjectReturnReport, ProjectReturnSection, ProjectReturnTotals,
        };
        let section = |p, is_root| ProjectReturnSection {
            project: p,
            is_root,
            cout_investi: dec!(100),
            revenus: dec!(40),
            resultat_net: dec!(-60),
            rendement_pct: Some(dec!(-60)),
        };
        let report = ProjectReturnReport {
            report_type: "return".into(),
            project: projet(1, "-P1", "Racine\r\nlongue"),
            mode: "fiscal_year".into(),
            period_label: "2026".into(),
            sections: vec![
                section(projet(1, "-P1", "Racine\r\nlongue"), true),
                section(projet(2, "+S1", "Sous-projet"), false),
            ],
            totals: ProjectReturnTotals {
                cout_investi: dec!(200),
                revenus: dec!(80),
                resultat_net: dec!(-120),
                rendement_pct: Some(dec!(-60)),
            },
        };
        let mut buf = Vec::new();
        render_project_return_csv(&report, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lignes: Vec<Vec<&str>> = body
            .split("\r\n")
            .filter(|l| !l.is_empty())
            .skip(1)
            .map(|l| l.split(';').collect())
            .collect();
        assert_eq!(lignes.len(), 3, "{body}");
        assert_eq!(
            lignes[0],
            [
                "'-P1 — Racine  longue",
                "",
                "100.00",
                "40.00",
                "-60.00",
                "-60.00%"
            ]
        );
        assert_eq!(lignes[1][1], "'+S1 — Sous-projet");
        assert_eq!(
            lignes[2],
            [
                "'-P1 — Racine  longue",
                "Total",
                "200.00",
                "80.00",
                "-120.00",
                "-60.00%"
            ]
        );
    }

    // ------------------------------------------------------------------
    // Story 46-3 — CSV par groupes
    // ------------------------------------------------------------------

    use crate::essai_groupes::{
        ab, bilan, bilan_essai, bilan_essai_groupe, noeud, options, resultat, resultat_essai, usd,
    };
    use crate::groupes::{ArbreComptes, AvecGroupes, NiveauDetail, PositionSousTotal};

    /// Le CSV groupé du bilan d'essai, sans BOM, en lignes.
    fn csv_bilan_groupe(o: &crate::groupes::OptionsGroupes) -> Vec<String> {
        let mut buf = Vec::new();
        render_grouped_balance_sheet_csv(&bilan_essai_groupe(o), &mut buf).unwrap();
        String::from_utf8(buf[3..].to_vec())
            .unwrap()
            .split("\r\n")
            .map(String::from)
            .collect()
    }

    /// AC-5, AC-6 — en `header` : `Niveau;TypeLigne` après `Section`, une ligne
    /// par ligne groupée, montants arrondis au centime (sous-total 16.011 →
    /// 16.01) ; totaux et lignes calculées avec deux cellules vides de plus.
    #[test]
    fn bilan_csv_groupe_header() {
        let l = csv_bilan_groupe(&options(
            NiveauDetail::Tous,
            PositionSousTotal::Titre,
            &[],
            &[],
        ));
        assert_eq!(
            l[..9],
            [
                "Section;Niveau;TypeLigne;NumeroCompte;NomCompte;Solde",
                "Actifs;1;groupe;10;Actifs circulants;16.01",
                "Actifs;2;groupe;100;Liquidités;16.01",
                "Actifs;3;propre;100;Liquidités;1.00",
                "Actifs;3;compte;1000;Caisse;10.01",
                "Actifs;3;compte;1020;Banque;5.01",
                "Actifs;1;groupe;14;Immobilisations;300.00",
                "Actifs;2;compte;1500;Machines;300.00",
                "Total actifs;;;;;316.01",
            ]
        );
        assert!(l.contains(&"CapitauxPropres;;;;Résultat de l'exercice;0.00".to_string()));
        assert!(l.contains(&"Total passifs + capitaux propres;;;;;316.01".to_string()));
    }

    /// AC-6 — au CSV groupé, totaux et lignes calculées arrondis au centime
    /// (10.005 → 10.01) ; le CSV à plat aussi, depuis la Story 48-11 (#1245).
    #[test]
    fn bilan_csv_groupe_totaux_arrondis_au_centime() {
        let mut bs = bilan(vec![crate::essai_groupes::ab(
            1000,
            "Caisse",
            dec!(10.005),
            AccountType::Asset,
        )]);
        bs.equity_result = dec!(0.005);
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let env = AvecGroupes::bilan(bs.clone(), &crate::essai_groupes::arbre(), &o);
        let mut buf = Vec::new();
        render_grouped_balance_sheet_csv(&env, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        for attendu in [
            "\r\nTotal actifs;;;;;10.01\r\n",
            "\r\nCapitauxPropres;;;;Résultat de l'exercice;0.01\r\n",
            "\r\nTotal passifs + capitaux propres;;;;;10.01\r\n",
        ] {
            assert!(s.contains(attendu), "{attendu:?} absent de {s:?}");
        }
        let mut plat = Vec::new();
        render_balance_sheet_csv(&bs, &mut plat).unwrap();
        assert!(
            String::from_utf8(plat)
                .unwrap()
                .contains("\r\nTotal actifs;;;10.01\r\n")
        );
    }

    /// AC-5 — en `footer` : titre d'un groupe ouvert sans montant, sa ligne
    /// `soustotal` après sa descendance ; un groupe fermé à la main porte son
    /// montant et n'a pas de comptes.
    #[test]
    fn bilan_csv_groupe_footer_et_ferme() {
        let l = csv_bilan_groupe(&options(
            NiveauDetail::Niveau(2),
            PositionSousTotal::Pied,
            &[],
            &[14],
        ));
        assert_eq!(
            l[1..5],
            [
                "Actifs;1;groupe;10;Actifs circulants;",
                "Actifs;2;groupe;100;Liquidités;16.01",
                "Actifs;1;soustotal;10;Actifs circulants;16.01",
                "Actifs;1;groupe;14;Immobilisations;300.00",
            ]
        );
    }

    /// AC-5 — colonnes en devise si une ligne groupée en porte : la devise et le
    /// solde en devise arrondi, sur le groupe comme sur ses comptes.
    #[test]
    fn bilan_csv_groupe_devise() {
        let arbre = ArbreComptes::depuis_noeuds(vec![
            noeud(102, "Banques USD", None),
            noeud(1025, "Banque USD", Some(102)),
        ]);
        let bs = bilan(vec![usd(1025, "Banque USD", dec!(9.005), dec!(10.005))]);
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let mut buf = Vec::new();
        render_grouped_balance_sheet_csv(&AvecGroupes::bilan(bs, &arbre, &o), &mut buf).unwrap();
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        assert_eq!(
            l[..4],
            [
                "Section;Niveau;TypeLigne;NumeroCompte;NomCompte;Solde;Devise;SoldeDevise",
                "Actifs;1;groupe;102;Banques USD;9.01;USD;10.01",
                "Actifs;2;compte;1025;Banque USD;9.01;USD;10.01",
                "Total actifs;;;;;9.01;;",
            ]
        );
    }

    /// AC-5 — en `footer`, le titre ouvert d'un groupe en devise laisse vides
    /// le solde, la devise et le solde en devise ; son sous-total les porte.
    #[test]
    fn bilan_csv_groupe_devise_footer() {
        let arbre = ArbreComptes::depuis_noeuds(vec![
            noeud(102, "Banques USD", None),
            noeud(1025, "Banque USD", Some(102)),
        ]);
        let bs = bilan(vec![usd(1025, "Banque USD", dec!(9.005), dec!(10.005))]);
        let o = options(NiveauDetail::Tous, PositionSousTotal::Pied, &[], &[]);
        let mut buf = Vec::new();
        render_grouped_balance_sheet_csv(&AvecGroupes::bilan(bs, &arbre, &o), &mut buf).unwrap();
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        assert_eq!(
            l[1..4],
            [
                "Actifs;1;groupe;102;Banques USD;;;",
                "Actifs;2;compte;1025;Banque USD;9.01;USD;10.01",
                "Actifs;1;soustotal;102;Banques USD;9.01;USD;10.01",
            ]
        );
    }

    /// AC-5 — les cellules texte du CSV groupé (numéro et nom, d'un groupe
    /// comme d'un compte) sont neutralisées comme à plat : une cellule qui
    /// commence par `=`, `+`, `-` ou `@` prend une apostrophe ; le montant, non.
    #[test]
    fn bilan_csv_groupe_echappe_les_cellules_texte() {
        let arbre = ArbreComptes::depuis_noeuds(vec![
            crate::groupes::NoeudCompte {
                numero: "-10".into(),
                ..noeud(10, "=1+1", None)
            },
            crate::groupes::NoeudCompte {
                numero: "+1000".into(),
                ..noeud(1000, "@x", Some(10))
            },
        ]);
        let mut compte = crate::essai_groupes::ab(1000, "@x", dec!(-5), AccountType::Asset);
        compte.account_number = "+1000".into();
        let bs = bilan(vec![compte]);
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let mut buf = Vec::new();
        render_grouped_balance_sheet_csv(&AvecGroupes::bilan(bs, &arbre, &o), &mut buf).unwrap();
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        assert_eq!(
            l[1..3],
            [
                "Actifs;1;groupe;'-10;'=1+1;-5.00",
                "Actifs;2;compte;'+1000;'@x;-5.00",
            ]
        );
    }

    /// AC-5 (revue P2, L1) — chaque section du CSV groupé du bilan porte ses
    /// **propres** lignes : actifs, passifs et capitaux propres de soldes
    /// distincts, chacun sous son groupe ; une section remplacée par une autre
    /// (même nombre de lignes) ou une ligne fausse rougit ici, ce que ne voit
    /// pas le décompte de lignes de `export_pdf_groupe_fidele_au_json`.
    #[test]
    fn bilan_csv_groupe_chaque_section_porte_ses_lignes() {
        let arbre = ArbreComptes::depuis_noeuds(vec![
            noeud(10, "Actifs circulants", None),
            noeud(1000, "Caisse", Some(10)),
            noeud(20, "Dettes", None),
            noeud(2000, "Fournisseurs", Some(20)),
            noeud(28, "Fonds propres", None),
            noeud(2800, "Capital", Some(28)),
        ]);
        let mut bs = bilan(vec![ab(1000, "Caisse", dec!(10), AccountType::Asset)]);
        bs.liabilities = vec![ab(2000, "Fournisseurs", dec!(4), AccountType::Liability)];
        bs.equity = vec![ab(2800, "Capital", dec!(6), AccountType::Liability)];
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let mut buf = Vec::new();
        render_grouped_balance_sheet_csv(&AvecGroupes::bilan(bs, &arbre, &o), &mut buf).unwrap();
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        for attendu in [
            "Actifs;1;groupe;10;Actifs circulants;10.00",
            "Actifs;2;compte;1000;Caisse;10.00",
            "Passifs;1;groupe;20;Dettes;4.00",
            "Passifs;2;compte;2000;Fournisseurs;4.00",
            "CapitauxPropres;1;groupe;28;Fonds propres;6.00",
            "CapitauxPropres;2;compte;2800;Capital;6.00",
        ] {
            assert!(l.contains(&attendu), "{attendu:?} absent de {l:?}");
        }
        // Et rien d'autre dans les sections : une ligne par ligne groupée.
        for (section, n) in [("Actifs;", 2), ("Passifs;", 2), ("CapitauxPropres;", 3)] {
            let vues = l.iter().filter(|x| x.starts_with(section)).count();
            assert_eq!(vues, n, "{section} dans {l:?}");
        }
    }

    /// AC-5 (revue P2, L1) — le compte de résultat groupé en CSV : produits et
    /// charges sous leurs groupes, `Total produits`, `Total charges` et
    /// `ResultatNet` (arrondi au centime) avec deux cellules vides de plus.
    #[test]
    fn compte_de_resultat_csv_groupe() {
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let env = AvecGroupes::compte_de_resultat(
            resultat_essai(),
            &crate::essai_groupes::arbre_resultat(),
            &o,
        );
        let mut buf = Vec::new();
        render_grouped_income_statement_csv(&env, &mut buf).unwrap();
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        assert_eq!(
            l[..9],
            [
                "Section;Niveau;TypeLigne;NumeroCompte;NomCompte;Solde",
                "Produits;1;groupe;3;Produits d'exploitation;150.00",
                "Produits;2;compte;3000;Ventes;100.00",
                "Produits;2;compte;3200;Services;50.00",
                "Total produits;;;;;150.00",
                "Charges;1;compte;4000;Achats;40.00",
                "Total charges;;;;;40.00",
                "ResultatNet;;;;;110.00",
                "",
            ]
        );
    }

    /// AC-5 (revue P2, L2) — la colonne en devise du CSV groupé suit les
    /// **lignes groupées**, non les comptes : un compte en USD replié dans un
    /// groupe fermé de devises mélangées ne la fait pas paraître.
    #[test]
    fn bilan_csv_groupe_ferme_en_devises_melangees_sans_colonne_en_devise() {
        let arbre = ArbreComptes::depuis_noeuds(vec![
            noeud(10, "Liquidités", None),
            noeud(1000, "Caisse", Some(10)),
            noeud(1025, "Banque USD", Some(10)),
        ]);
        let bs = bilan(vec![
            ab(1000, "Caisse", dec!(10), AccountType::Asset),
            usd(1025, "Banque USD", dec!(9), dec!(10)),
        ]);
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[10]);
        let mut buf = Vec::new();
        render_grouped_balance_sheet_csv(&AvecGroupes::bilan(bs, &arbre, &o), &mut buf).unwrap();
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        let l: Vec<&str> = s.split("\r\n").collect();
        assert_eq!(
            l[..3],
            [
                "Section;Niveau;TypeLigne;NumeroCompte;NomCompte;Solde",
                "Actifs;1;groupe;10;Liquidités;19.00",
                "Total actifs;;;;;19.00",
            ]
        );
    }

    /// AC-2 — sans groupes, le CSV « groupé » est le CSV à plat, à l'octet.
    #[test]
    fn csv_groupe_sans_groupes_est_le_csv_a_plat() {
        let bs = bilan_essai();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        render_grouped_balance_sheet_csv(&AvecGroupes::sans_groupes(bs.clone()), &mut a).unwrap();
        render_balance_sheet_csv(&bs, &mut b).unwrap();
        assert_eq!(a, b);
        let is_ = resultat_essai();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        render_grouped_income_statement_csv(&AvecGroupes::sans_groupes(is_.clone()), &mut a)
            .unwrap();
        render_income_statement_csv(&is_, &mut b).unwrap();
        assert_eq!(a, b);
    }

    /// AC-3 — rapport vide groupé : la ligne d'en-tête seule.
    #[test]
    fn csv_groupe_vide_n_a_que_l_en_tete() {
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let vide = ArbreComptes::depuis_noeuds(vec![]);
        let mut buf = Vec::new();
        render_grouped_income_statement_csv(
            &AvecGroupes::compte_de_resultat(resultat(vec![], vec![]), &vide, &o),
            &mut buf,
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(buf[3..].to_vec()).unwrap(),
            "Section;Niveau;TypeLigne;NumeroCompte;NomCompte;Solde\r\n"
        );
    }

    // ------------------------------------------------------------------
    // Story 46-6b — CSV de la balance par groupes
    // ------------------------------------------------------------------

    use crate::essai_groupes::{
        balance, balance_essai, balance_essai_groupee, balance_usd_groupee,
    };

    /// Les lignes du CSV d'une balance groupée (sans le BOM).
    fn csv_balance(
        env: &AvecGroupes<TrialBalance, Vec<crate::LigneBalanceGroupee>>,
    ) -> Vec<String> {
        let mut buf = Vec::new();
        render_grouped_trial_balance_csv(env, &mut buf).unwrap();
        assert_eq!(&buf[..3], UTF8_BOM);
        String::from_utf8(buf[3..].to_vec())
            .unwrap()
            .split("\r\n")
            .map(String::from)
            .collect()
    }

    /// AC-2, AC-4 — `Niveau;TypeLigne` en tête, une ligne par ligne groupée,
    /// quatre montants arrondis au centime (débit exact de `10` : 10.011 →
    /// 10.01) ; résultat reporté et `Total` avec deux cellules vides en tête.
    #[test]
    fn balance_csv_groupee_header() {
        let l = csv_balance(&balance_essai_groupee(&options(
            NiveauDetail::Tous,
            PositionSousTotal::Titre,
            &[],
            &[],
        )));
        assert_eq!(
            l,
            vec![
                "Niveau;TypeLigne;NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture",
                "1;groupe;10;Actifs circulants;5.00;10.01;0.00;15.01",
                "2;groupe;100;Liquidités;5.00;10.01;0.00;15.01",
                "3;compte;1000;Caisse;0.00;10.01;0.00;10.01",
                "3;compte;1020;Banque;5.00;0.01;0.00;5.01",
                "1;groupe;14;Immobilisations;300.00;0.00;0.00;300.00",
                "2;compte;1500;Machines;300.00;0.00;0.00;300.00",
                "1;compte;2000;Fournisseurs;305.00;0.00;10.01;315.01",
                ";;;Résultat reporté (calculé);0.00;;;0.00",
                ";;Total;;;10.01;10.01;",
                "",
            ]
        );
    }

    /// AC-2 — en pied : le titre d'un groupe ouvert a ses montants vides, sa
    /// ligne `soustotal` les porte ; un groupe fermé les porte sur son titre.
    #[test]
    fn balance_csv_groupee_footer_et_groupe_ferme() {
        let l = csv_balance(&balance_essai_groupee(&options(
            NiveauDetail::Niveau(2),
            PositionSousTotal::Pied,
            &[],
            &[14],
        )));
        for attendu in [
            "1;groupe;10;Actifs circulants;;;;",
            "2;groupe;100;Liquidités;5.00;10.01;0.00;15.01",
            "1;soustotal;10;Actifs circulants;5.00;10.01;0.00;15.01",
            "1;groupe;14;Immobilisations;300.00;0.00;0.00;300.00",
        ] {
            assert!(
                l.contains(&attendu.to_string()),
                "{attendu:?} absent de {l:?}"
            );
        }
        assert!(!l.iter().any(|x| x.contains(";1500;")), "{l:?}");
    }

    /// AC-2 — colonnes en devise : un groupe en devise unique les remplit
    /// comme un compte (montants en devise arrondis), une ligne sans devise et
    /// les lignes calculées les laissent vides.
    #[test]
    fn balance_csv_groupee_devise() {
        let l = csv_balance(&balance_usd_groupee(&options(
            NiveauDetail::Tous,
            PositionSousTotal::Titre,
            &[],
            &[],
        )));
        assert_eq!(
            l[0],
            "Niveau;TypeLigne;NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture;Devise;SoldeOuvertureDevise;TotalDebitDevise;TotalCreditDevise;SoldeClotureDevise"
        );
        for attendu in [
            "1;groupe;10;Devises;90.00;9.45;0.00;99.45;USD;100.00;10.51;0.00;110.51",
            "2;compte;1000;Caisse USD;0.00;0.45;0.00;0.45;USD;0.00;0.51;0.00;0.51",
            "1;compte;2000;Fournisseurs;90.00;0.00;9.45;99.45;;;;;",
            ";;Total;;;9.45;9.45;;;;;;",
        ] {
            assert!(
                l.contains(&attendu.to_string()),
                "{attendu:?} absent de {l:?}"
            );
        }
        // En pied, le titre ouvert laisse aussi ses colonnes en devise vides.
        let l = csv_balance(&balance_usd_groupee(&options(
            NiveauDetail::Tous,
            PositionSousTotal::Pied,
            &[],
            &[],
        )));
        assert!(
            l.contains(&"1;groupe;10;Devises;;;;;;;;;".to_string()),
            "{l:?}"
        );
        assert!(
            l.contains(
                &"1;soustotal;10;Devises;90.00;9.45;0.00;99.45;USD;100.00;10.51;0.00;110.51"
                    .to_string()
            ),
            "{l:?}"
        );
    }

    /// AC-2 — balance vide groupée : l'en-tête seul, comme à plat.
    #[test]
    fn balance_csv_groupee_vide() {
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let env = AvecGroupes::balance(balance(vec![]), &crate::essai_groupes::arbre(), &o);
        assert_eq!(
            csv_balance(&env),
            vec![
                "Niveau;TypeLigne;NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture",
                "",
            ]
        );
    }

    /// AC-3 — sans `grouping`, le CSV groupé est celui d'avant, à l'octet.
    #[test]
    fn balance_csv_sans_groupes_est_le_rendu_a_plat() {
        let tb = balance_essai();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        render_trial_balance_csv(&tb, &mut a).unwrap();
        render_grouped_trial_balance_csv(&AvecGroupes::sans_groupes(tb), &mut b).unwrap();
        assert_eq!(a, b);
    }

    /// Une cellule de texte d'une ligne groupée de la balance est neutralisée
    /// comme à plat (Story 37-5) ; un montant négatif reste un nombre.
    #[test]
    fn balance_csv_groupee_neutralise_le_texte() {
        let arbre = ArbreComptes::depuis_noeuds(vec![
            noeud(10, "=Groupe", None),
            noeud(1000, "@Caisse", Some(10)),
        ]);
        let tb = balance(vec![crate::essai_groupes::tb_ligne(
            1000,
            "@Caisse",
            AccountType::Asset,
            [dec!(-1), dec!(0), dec!(0), dec!(-1)],
        )]);
        let o = options(NiveauDetail::Tous, PositionSousTotal::Titre, &[], &[]);
        let l = csv_balance(&AvecGroupes::balance(tb, &arbre, &o));
        assert!(
            l.contains(&"1;groupe;10;'=Groupe;-1.00;0.00;0.00;-1.00".to_string()),
            "{l:?}"
        );
        assert!(
            l.contains(&"2;compte;1000;'@Caisse;-1.00;0.00;0.00;-1.00".to_string()),
            "{l:?}"
        );
    }
    // --- Story 48-11 (#1245) : les montants s'arrondissent au centime ---

    #[test]
    fn format_amount_iso_arrondit_au_centime() {
        assert_eq!(format_amount_iso(dec!(10.0050)), "10.01");
        assert_eq!(format_amount_iso(dec!(5.0060)), "5.01");
        assert_eq!(format_amount_iso(dec!(-0.0040)), "0.00");
        assert_eq!(format_amount_iso(dec!(-10.0050)), "-10.01");
    }

    /// Prouve le **formateur** seul. En production, `project_report.rs` arrondit
    /// déjà le rendement **au pair** (`round_dp(2)`) avant l'affichage : un
    /// rendement exact de 12.345 s'affiche donc `12.34%` (#1442).
    #[test]
    fn format_rendement_arrondit() {
        assert_eq!(format_rendement(Some(dec!(12.345))), "12.35%");
    }

    #[test]
    fn la_balance_csv_arrondit_la_fraction_de_centime() {
        let tb = TrialBalance {
            period: period(),
            rows: vec![TrialBalanceRow {
                account_id: 1,
                account_number: "1000".into(),
                account_name: "Caisse".into(),
                account_type: AccountType::Asset,
                active: true,
                opening_balance: dec!(0),
                total_debit: dec!(10.0050),
                total_credit: dec!(0),
                balance: dec!(10.0050),
                closing_balance: dec!(10.0050),
                currency: None,
                opening_currency: None,
                total_debit_currency: None,
                total_credit_currency: None,
                balance_currency: None,
                closing_currency: None,
            }],
            total_debit: dec!(10.0050),
            total_credit: dec!(0),
            balanced: true,
            retained_earnings: dec!(0),
            opening_balanced: true,
        };
        let mut buf = Vec::new();
        render_trial_balance_csv(&tb, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(body.contains("10.01"), "{body}");
        assert!(!body.contains("10.00"), "{body}");
    }
}
