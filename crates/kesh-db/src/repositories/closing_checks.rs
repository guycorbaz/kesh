//! Préparer la clôture d'un exercice — les **contrôles préalables**, en lecture
//! seule (Story 29-1a, #232).
//!
//! [`for_fiscal_year`] rend, pour un exercice `Y`, **exactement** sept
//! contrôles, dans cet ordre (l'ensemble clos de la fiche, AC1) :
//!
//! | Code | Sévérité | `attention` quand… |
//! |---|---|---|
//! | K1 `EARLIER_FISCAL_YEAR_OPEN` | `blocking` | un exercice antérieur est ouvert (le plus ancien est rendu) |
//! | K2 `PENDING_BANK_TRANSACTIONS` | `warning` | une transaction bancaire `pending` est datée (`booking_date`) dans Y |
//! | K3 `BANK_STATEMENT_AT_YEAR_END` | `warning` | un compte bancaire examiné n'a pas de relevé arrêté au dernier jour de Y, ou un écart avec les livres |
//! | K4 `DRAFT_INVOICES` | `info` | une facture client en brouillon est datée dans Y |
//! | K5 `IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE` | `info` | une pièce fournisseur importée attend d'être complétée |
//! | K6 `INCONSISTENT_ENTRIES` | `anomaly` | une écriture incohérente (déséquilibrée, à moins de deux lignes, hors de son exercice, ou d'un autre exercice datée dans Y) |
//! | K7 `NEXT_FISCAL_YEAR` | `info` | aucun exercice ne suit Y |
//!
//! `count` vaut le nombre de points relevés : transactions (K2), comptes en
//! défaut (K3), factures (K4), pièces (K5), écritures distinctes (K6) ; pour
//! K1 et K7, `1` quand le contrôle est en `attention`, `0` sinon.
//!
//! Rien ne **bloque** : la clôture garde son unique refus existant (K1,
//! `fiscal_years::close`), qui reste l'autorité.
//!
//! # Une seule vue, aucune écriture, aucun verrou (AC2)
//!
//! La fonction s'exécute sur la connexion d'un
//! [`crate::snapshot::ReadSnapshot`] que la route ouvre et clôt : toutes les
//! lectures voient la base **au même instant**, l'exercice Y compris (lu ici,
//! par [`FISCAL_YEAR_SQL`], non sur le pool) ; l'instantané est `READ ONLY`.
//! Aucune lecture ne verrouille. La garde statique T-a2 (tests du module)
//! vérifie que tout appel `query`, `query_as` ou `query_scalar` de ce module —
//! suivi de `(` ou d'un turbofish `::<…>(` — a pour premier argument une
//! constante de [`ALL_SQL`] (volet a), et qu'aucun mot de verrou n'est écrit en
//! littéral dans le source décommenté (volet b).
//!
//! ⚠️ **Formes que T-a2 ne voit pas** (revue de code P1, B-2 = F7 ; la garde
//! n'est pas élargie, sa forme ayant été réduite à deux volets en validation,
//! C-29-1-27) : `query_with`, `query_as_with`, `query_scalar_with`,
//! `query_unchecked` et toute forme dont le mot est suivi d'un caractère
//! d'identifiant ; les macros `query!`, `query_as!`, `query_scalar!` ;
//! `QueryBuilder`, `sqlx::raw_sql` ; les méthodes d'`Executor` appelées sur une
//! chaîne (`execute("…")`, `fetch_*("…")`) ; un alias (`use sqlx::query as q`) ;
//! et, au volet (b), un mot de verrou coupé par une continuation de chaîne
//! (`"… FOR \` puis `UPDATE"` à la ligne suivante). Aucune n'est employée dans
//! ce module ; leur absence se vérifie **en revue de code**, comme celle des
//! verrous des fonctions externes ci-dessous.
//!
//! ⚠️ **Angle mort assumé de T-a2** : les fonctions de dépôt **externes**
//! appelées ici — et le SQL qu'elles exécutent — ne sont pas lues par la
//! garde. Elles se vérifient **en revue de code** (chacune sans verrou, sur
//! l'exécuteur reçu) :
//!
//! - [`super::bank_accounts::balances_by_bank_account`] (texte `BALANCES_SQL`) ;
//! - [`super::bank_accounts::balances_from_row`], [`super::bank_accounts::is_shared`],
//!   [`super::bank_accounts::statement_gap`] (sans SQL) ;
//! - [`super::fiscal_years::find_earliest_earlier_open`] (texte `EARLIER_OPEN_SQL`) ;
//! - [`super::fiscal_years::find_next_after`] (texte `NEXT_AFTER_SQL`).
//!
//! Le panneau lit **à un instant** : un contrôle lu `ok` peut être faux une
//! seconde plus tard (une transaction importée entre-temps). C'est un
//! instrument de revue, non une garantie.
//!
//! # Ce que les contrôles ne disent pas
//!
//! - **K5 n'est pas borné à l'exercice** : `imported_supplier_invoices` n'a pas
//!   de date métier, seulement `created_at`.
//! - **K3** ne compare que les relevés qui portent un solde (CAMT ; un CSV n'en
//!   a pas). Un compte sans compte lié, à compte lié partagé, ou alimenté par
//!   CSV seulement est **non vérifiable** (`notCheckable`), sans effet sur le
//!   statut. Le relevé de référence est celui qui **se termine au plus tard le
//!   dernier jour de Y** : un relevé à cheval sur la fin d'année ne compte pas.
//! - **K4** ne compte que les factures **client** (`invoices`) : un avoir
//!   (`credit_notes`) en brouillon n'est pas examiné. Kesh émet un avoir en une
//!   seule étape, au statut `issued` ; un avoir `draft` ne peut venir que de
//!   données posées hors de l'application (revue de code P1, B-1).
//! - **K4 et K6** comptent **tous** les points (`count`), mais n'en listent que
//!   les [`MAX_LISTED`] premiers, par date puis par `id` ; le manuel le dit, et
//!   seul K4 a un lien vers la liste complète.
//! - **K6** ne peut signaler que des données héritées ou restaurées : aucun
//!   chemin d'écriture actuel n'en produit (l'équilibre est vérifié après
//!   l'INSERT) ; il ne couvre pas une ligne dont le compte appartient à une
//!   autre société que l'écriture.
//! - **Coût de K6** (revue de code P1, B-3) : la table dérivée des totaux de
//!   lignes agrège **tout** le grand livre de la société, non les seules
//!   écritures de la population ; le coût croît avec l'historique, à chaque
//!   ouverture du panneau et du dialogue de clôture (deux lectures). Sans effet
//!   fonctionnel ; une borne ne s'ajouterait que sur mesure.
//! - **K7** rend le **premier** exercice qui commence après la fin de Y, même
//!   après un trou : un exercice 2028 sans 2027 « suit » 2026, et K7 est `ok`.
//! - L'écart de K3 est celui de [`bank_accounts::statement_gap`] : les deux
//!   soldes arrondis au centime, demi loin de zéro — un reste de moins d'un
//!   demi-centime est nul, un reste d'un demi-centime ou plus compte.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlConnection;

use crate::entities::{FiscalYear, FiscalYearStatus, bank_account::bank_account_display_name};
use crate::errors::{DbError, map_db_error};
use crate::repositories::{bank_accounts, fiscal_years};

/// Les contrôles d'un exercice — la réponse de
/// `GET /api/v1/fiscal-years/{id}/closing-checks`, sérialisée en camelCase.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosingChecks {
    /// Identifiant de l'exercice examiné (Y).
    pub fiscal_year_id: i64,
    /// Nom de Y.
    pub fiscal_year_name: String,
    /// Premier jour de Y.
    pub start_date: NaiveDate,
    /// Dernier jour de Y.
    pub end_date: NaiveDate,
    /// **Toujours sept** contrôles, dans l'ordre de l'AC1 (table de la doc du
    /// module).
    pub checks: Vec<ClosingCheck>,
}

/// Un contrôle : son code, sa sévérité, son statut, le nombre de points relevés
/// et ses détails.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosingCheck {
    /// Code stable — c'est **lui** qui dit au client comment lire `details`.
    pub code: CheckCode,
    /// Sévérité, fixe par code.
    pub severity: Severity,
    /// `ok` ou `attention`.
    pub status: CheckStatus,
    /// Nombre de points relevés (voir la doc du module) ; pour K1 et K7, `1` en
    /// `attention`, `0` sinon. Jamais borné, même quand la liste l'est.
    pub count: i64,
    /// Détails propres au code.
    pub details: CheckDetails,
}

/// Code stable d'un contrôle, sérialisé en SCREAMING_SNAKE_CASE ; un code neuf
/// serait un changement de contrat de l'API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckCode {
    /// K1 — un exercice antérieur est ouvert.
    EarlierFiscalYearOpen,
    /// K2 — transactions bancaires en attente, datées dans Y.
    PendingBankTransactions,
    /// K3 — relevé bancaire de fin d'exercice.
    BankStatementAtYearEnd,
    /// K4 — factures client en brouillon, datées dans Y.
    DraftInvoices,
    /// K5 — pièces fournisseur importées à compléter (non borné à Y).
    ImportedSupplierInvoicesToComplete,
    /// K6 — écritures incohérentes.
    InconsistentEntries,
    /// K7 — l'exercice qui suit Y.
    NextFiscalYear,
}

/// Sévérité d'un contrôle, sérialisée en minuscules. Seul `blocking` (K1)
/// correspond à un refus réel, celui de la clôture ; rien ne bloque ici.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// La clôture refuse tant que le point demeure (K1).
    Blocking,
    /// À vérifier avant de clôturer (K2, K3).
    Warning,
    /// Pour information (K4, K5, K7).
    Info,
    /// Données incohérentes, héritées ou restaurées (K6).
    Anomaly,
}

/// Statut d'un contrôle, sérialisé en minuscules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    /// Rien à signaler (pour K7 : un exercice suit).
    Ok,
    /// Au moins un point relevé.
    Attention,
}

/// Détails d'un contrôle, sérialisés **sans étiquette** (`untagged`) : le JSON
/// ne porte que le contenu de la variante. ⚠️ K1 et K7 ont la **même forme**
/// (`{ "fiscalYear": … | null }`) : le client les distingue par
/// [`ClosingCheck::code`], jamais par la forme des détails.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum CheckDetails {
    /// K1.
    EarlierFiscalYearOpen(EarlierOpenDetails),
    /// K2.
    PendingBankTransactions(PendingDetails),
    /// K3.
    BankStatementAtYearEnd(StatementDetails),
    /// K4.
    DraftInvoices(DraftDetails),
    /// K5 (objet vide).
    ImportedSupplierInvoicesToComplete(ImportedDetails),
    /// K6.
    InconsistentEntries(InconsistentDetails),
    /// K7.
    NextFiscalYear(NextFiscalYearDetails),
}

/// Un exercice nommé (K1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalYearRef {
    /// Identifiant de l'exercice.
    pub id: i64,
    /// Nom de l'exercice.
    pub name: String,
}

/// Détails de K1.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EarlierOpenDetails {
    /// Le **plus ancien** exercice antérieur ouvert, ou `None` (JSON `null`).
    pub fiscal_year: Option<FiscalYearRef>,
}

/// K2 : les transactions en attente d'un compte bancaire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingByAccount {
    /// Identifiant du compte bancaire.
    pub bank_account_id: i64,
    /// Nom affiché du compte (banque et description).
    pub name: String,
    /// Transactions `pending` datées dans Y.
    pub count: i64,
    /// Parmi elles, celles que l'utilisateur a écartées des propositions.
    pub rejected_count: i64,
}

/// Détails de K2.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingDetails {
    /// Un élément par compte qui a au moins une transaction en attente.
    pub accounts: Vec<PendingByAccount>,
}

/// K3 : motif d'un compte en défaut (motifs 3 à 5 de la table de précédence).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatementReason {
    /// Le relevé à solde retenu — celui qui se termine au plus tard le dernier
    /// jour de Y — manque, ou se termine avant Y.
    Missing,
    /// Relevé arrêté avant `MIN(Y.end_date, closed_on)`.
    EndsBeforeYearEnd,
    /// Écart entre le relevé et les livres à sa date.
    Gap,
}

/// K3 : motif d'un compte examiné mais **non vérifiable** (motifs 1 et 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotCheckableReason {
    /// Sans compte de grand livre lié, ou compte lié partagé.
    NotComparable,
    /// Aucun relevé à solde, toutes dates (CSV seulement).
    NoBalanceStatement,
}

/// K3 : un compte en défaut, avec les montants qui le montrent (chaînes
/// décimales en JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementIssue {
    /// Identifiant du compte bancaire.
    pub bank_account_id: i64,
    /// Nom affiché du compte.
    pub name: String,
    /// Le premier motif qui s'applique.
    pub reason: StatementReason,
    /// Date de fin du relevé retenu, s'il y en a un.
    pub statement_date: Option<NaiveDate>,
    /// Solde final de ce relevé.
    pub statement_closing_balance: Option<Decimal>,
    /// Solde comptable du compte lié à cette date.
    pub ledger_balance_at_statement: Option<Decimal>,
    /// Écart arrondi au centime ([`bank_accounts::statement_gap`]), `None` s'il
    /// est nul ou ne se calcule pas.
    pub gap: Option<Decimal>,
}

/// K3 : un compte examiné mais non vérifiable — sans effet sur le statut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotCheckable {
    /// Identifiant du compte bancaire.
    pub bank_account_id: i64,
    /// Nom affiché du compte.
    pub name: String,
    /// Pourquoi il n'est pas vérifiable.
    pub reason: NotCheckableReason,
}

/// Détails de K3.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementDetails {
    /// Les comptes en défaut — `count` en est le nombre.
    pub accounts: Vec<StatementIssue>,
    /// Les comptes examinés mais non vérifiables.
    pub not_checkable: Vec<NotCheckable>,
}

/// K4 : une facture en brouillon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftInvoiceRef {
    /// Identifiant de la facture.
    pub id: i64,
    /// Libellé : le nom du contact (un brouillon n'a pas de numéro).
    pub label: String,
    /// Date de la facture.
    pub date: NaiveDate,
}

/// Détails de K4.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftDetails {
    /// Au plus [`MAX_LISTED`] brouillons, par date puis par `id`.
    pub invoices: Vec<DraftInvoiceRef>,
}

/// Détails de K5 : objet vide, le compte suffit.
#[derive(Debug, Clone, Serialize)]
pub struct ImportedDetails {}

/// K6 : motif d'une écriture incohérente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntryReason {
    /// Écriture de Y dont les totaux débit et crédit diffèrent.
    Unbalanced,
    /// Écriture de Y à moins de deux lignes (zéro compris).
    TooFewLines,
    /// Écriture de Y datée hors de Y.
    DateOutsideItsYear,
    /// Écriture d'un autre exercice datée dans Y.
    OtherYearDatedInside,
}

/// K6 : nombre d'écritures par motif, sérialisé en SCREAMING_SNAKE_CASE
/// (`UNBALANCED`, …) — les clés sont les codes de [`EntryReason`]. Une
/// écriture peut compter sous deux motifs : la somme peut dépasser `count`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct CountByReason {
    /// [`EntryReason::Unbalanced`].
    pub unbalanced: i64,
    /// [`EntryReason::TooFewLines`].
    pub too_few_lines: i64,
    /// [`EntryReason::DateOutsideItsYear`].
    pub date_outside_its_year: i64,
    /// [`EntryReason::OtherYearDatedInside`].
    pub other_year_dated_inside: i64,
}

/// K6 : une écriture incohérente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InconsistentEntry {
    /// Identifiant de l'écriture.
    pub id: i64,
    /// Numéro de l'écriture dans son exercice.
    pub entry_number: i64,
    /// Date de l'écriture.
    pub entry_date: NaiveDate,
    /// Nom de l'exercice **de l'écriture** (qui peut ne pas être Y).
    pub fiscal_year_name: String,
    /// Ses motifs, dans l'ordre de [`EntryReason`].
    pub reasons: Vec<EntryReason>,
}

/// Détails de K6.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InconsistentDetails {
    /// Décompte par motif, sur **toutes** les écritures relevées.
    pub count_by_reason: CountByReason,
    /// Au plus [`MAX_LISTED`] écritures, par date puis par `id`.
    pub entries: Vec<InconsistentEntry>,
}

/// K7 : l'exercice qui suit Y.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextFiscalYearRef {
    /// Identifiant de l'exercice.
    pub id: i64,
    /// Nom de l'exercice.
    pub name: String,
    /// Son premier jour.
    pub start_date: NaiveDate,
    /// Son statut (ouvert ou clos).
    pub status: FiscalYearStatus,
}

/// Détails de K7.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextFiscalYearDetails {
    /// Le premier exercice qui commence après la fin de Y — même après un
    /// trou —, ou `None` (JSON `null`).
    pub fiscal_year: Option<NextFiscalYearRef>,
}

// ---------------------------------------------------------------------------
// Les requêtes — l'inventaire clos de T-a2
// ---------------------------------------------------------------------------

/// Nombre maximal d'éléments listés par contrôle (K4, K6) ; le compte, lui,
/// n'est pas borné.
const MAX_LISTED: usize = 20;

/// L'exercice Y, lu **dans l'instantané** et scopé à la société (AC1) : absent
/// ou d'une autre société → [`DbError::NotFound`] (anti-énumération).
const FISCAL_YEAR_SQL: &str = "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
     FROM fiscal_years WHERE id = ? AND company_id = ?";

/// K2 : transactions `pending` datées dans Y, par compte bancaire ; les
/// rejetées par l'utilisateur (`auto_match_rejected_at`) comptées à part.
const PENDING_SQL: &str = "SELECT bt.bank_account_id, ba.bank_name, ba.description, \
            CAST(COUNT(*) AS SIGNED) AS n, \
            CAST(SUM(CASE WHEN bt.auto_match_rejected_at IS NOT NULL THEN 1 ELSE 0 END) AS SIGNED) AS rejected \
     FROM bank_transactions bt \
     INNER JOIN bank_accounts ba ON ba.id = bt.bank_account_id AND ba.company_id = bt.company_id \
     WHERE bt.company_id = ? AND bt.status = 'pending' AND bt.booking_date BETWEEN ? AND ? \
     GROUP BY bt.bank_account_id, ba.bank_name, ba.description \
     ORDER BY bt.bank_account_id ASC";

/// K3 : les comptes bancaires **examinés** — ni archivés, ni clos avant Y, et
/// actifs dans Y par l'un de trois `EXISTS` séparés (jamais `bank_transactions`
/// dans le même `FROM` que les lignes du grand livre) : une transaction datée
/// dans Y, un relevé à solde arrêté dans Y, une ligne d'écriture de Y sur le
/// compte lié. Le quatrième `EXISTS`, **sans borne de date**, n'entre pas dans
/// le critère : il dit si le compte a jamais reçu un relevé à solde (motif
/// `NO_BALANCE_STATEMENT`). Pas de critère sur `created_at` : une reprise de
/// comptabilité crée le compte en N+1 et y importe les relevés de N.
const EXAMINED_ACCOUNTS_SQL: &str = "SELECT ba.id, ba.bank_name, ba.description, ba.journal_account_id, ba.closed_on, \
            CAST(EXISTS (SELECT 1 FROM bank_imports bi \
                         WHERE bi.bank_account_id = ba.id AND bi.company_id = ba.company_id \
                           AND bi.closing_balance IS NOT NULL) AS SIGNED) AS has_balance_statement \
     FROM bank_accounts ba \
     WHERE ba.company_id = ? AND ba.archived = FALSE \
       AND (ba.closed_on IS NULL OR ba.closed_on >= ?) \
       AND (EXISTS (SELECT 1 FROM bank_transactions bt \
                    WHERE bt.bank_account_id = ba.id AND bt.company_id = ba.company_id \
                      AND bt.booking_date BETWEEN ? AND ?) \
            OR EXISTS (SELECT 1 FROM bank_imports bi \
                       WHERE bi.bank_account_id = ba.id AND bi.company_id = ba.company_id \
                         AND bi.closing_balance IS NOT NULL AND bi.period_to BETWEEN ? AND ?) \
            OR EXISTS (SELECT 1 FROM journal_entry_lines jel \
                       INNER JOIN journal_entries je ON je.id = jel.entry_id \
                       WHERE jel.account_id = ba.journal_account_id AND je.company_id = ba.company_id \
                         AND je.entry_date BETWEEN ? AND ?)) \
     ORDER BY ba.id ASC";

/// K4 : nombre de factures client en brouillon datées dans Y.
const DRAFT_COUNT_SQL: &str = "SELECT CAST(COUNT(*) AS SIGNED) FROM invoices \
     WHERE company_id = ? AND status = 'draft' AND date BETWEEN ? AND ?";

/// K4 : les premières, avec le nom du contact — le libellé de la liste des
/// factures pour un brouillon, qui n'a pas de numéro.
const DRAFT_LIST_SQL: &str = "SELECT i.id, c.name, i.date FROM invoices i \
     INNER JOIN contacts c ON c.id = i.contact_id \
     WHERE i.company_id = ? AND i.status = 'draft' AND i.date BETWEEN ? AND ? \
     ORDER BY i.date ASC, i.id ASC LIMIT ?";

/// K5 : pièces fournisseur importées à compléter — **non borné à l'exercice**.
const IMPORTED_SQL: &str = "SELECT CAST(COUNT(*) AS SIGNED) FROM imported_supplier_invoices \
     WHERE company_id = ? AND status = 'to_complete'";

/// K6 : les écritures incohérentes, une ligne par écriture, un drapeau par
/// motif. Population : les écritures **de Y** et celles **datées dans Y**. Les
/// totaux des lignes sont agrégés dans une table dérivée bornée à la société.
///
/// Le `WHERE` de population n'est qu'un **filtre de lignes** : chaque drapeau
/// porte déjà sa propre condition sur Y (exercice ou date), si bien qu'une
/// écriture hors population a ses quatre drapeaux à zéro et serait écartée par
/// le `HAVING`. L'élargir ne change donc pas le résultat — mutation M-a33,
/// équivalente (revue de code P1).
const INCONSISTENT_SQL: &str = "SELECT je.id, je.entry_number, je.entry_date, fy.name AS fiscal_year_name, \
            CAST(je.fiscal_year_id = ? AND COALESCE(l.total_debit, 0) <> COALESCE(l.total_credit, 0) AS SIGNED) \
              AS unbalanced, \
            CAST(je.fiscal_year_id = ? AND COALESCE(l.n, 0) < 2 AS SIGNED) AS too_few_lines, \
            CAST(je.fiscal_year_id = ? AND (je.entry_date < ? OR je.entry_date > ?) AS SIGNED) \
              AS date_outside_its_year, \
            CAST(je.fiscal_year_id <> ? AND je.entry_date BETWEEN ? AND ? AS SIGNED) \
              AS other_year_dated_inside \
     FROM journal_entries je \
     INNER JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
     LEFT JOIN (SELECT jel.entry_id, COUNT(*) AS n, \
                       SUM(jel.debit) AS total_debit, SUM(jel.credit) AS total_credit \
                FROM journal_entry_lines jel \
                INNER JOIN journal_entries j2 ON j2.id = jel.entry_id \
                WHERE j2.company_id = ? \
                GROUP BY jel.entry_id) l ON l.entry_id = je.id \
     WHERE je.company_id = ? AND (je.fiscal_year_id = ? OR je.entry_date BETWEEN ? AND ?) \
     HAVING unbalanced = 1 OR too_few_lines = 1 OR date_outside_its_year = 1 \
         OR other_year_dated_inside = 1 \
     ORDER BY je.entry_date ASC, je.id ASC";

/// **Toutes** les requêtes du module : la garde T-a2 exige que chaque site
/// d'appel `query*` ait pour premier argument l'une d'elles, et que leur nombre
/// égale celui des `const …_SQL` du module.
#[allow(dead_code)] // lu par la garde statique T-a2, pas par le code
const ALL_SQL: &[&str] = &[
    FISCAL_YEAR_SQL,
    PENDING_SQL,
    EXAMINED_ACCOUNTS_SQL,
    DRAFT_COUNT_SQL,
    DRAFT_LIST_SQL,
    IMPORTED_SQL,
    INCONSISTENT_SQL,
];

// ---------------------------------------------------------------------------
// La lecture
// ---------------------------------------------------------------------------

/// Les sept contrôles de l'exercice `fiscal_year_id` de la société, dans
/// l'ordre de l'AC1, lus sur `conn` — la connexion d'un
/// [`crate::snapshot::ReadSnapshot`] (voir la doc du module).
///
/// Retourne [`DbError::NotFound`] si l'exercice n'existe pas ou appartient à
/// une autre société.
pub async fn for_fiscal_year(
    conn: &mut MySqlConnection,
    company_id: i64,
    fiscal_year_id: i64,
) -> Result<ClosingChecks, DbError> {
    let y = sqlx::query_as::<_, FiscalYear>(FISCAL_YEAR_SQL)
        .bind(fiscal_year_id)
        .bind(company_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;

    let checks = vec![
        earlier_fiscal_year_open(conn, company_id, &y).await?,
        pending_bank_transactions(conn, company_id, &y).await?,
        bank_statement_at_year_end(conn, company_id, &y).await?,
        draft_invoices(conn, company_id, &y).await?,
        imported_supplier_invoices_to_complete(conn, company_id).await?,
        inconsistent_entries(conn, company_id, &y).await?,
        next_fiscal_year(conn, company_id, &y).await?,
    ];
    Ok(ClosingChecks {
        fiscal_year_id: y.id,
        fiscal_year_name: y.name,
        start_date: y.start_date,
        end_date: y.end_date,
        checks,
    })
}

fn status_of(attention: bool) -> CheckStatus {
    if attention {
        CheckStatus::Attention
    } else {
        CheckStatus::Ok
    }
}

/// K1 — le plus ancien exercice antérieur ouvert.
async fn earlier_fiscal_year_open(
    conn: &mut MySqlConnection,
    company_id: i64,
    y: &FiscalYear,
) -> Result<ClosingCheck, DbError> {
    let earlier = fiscal_years::find_earliest_earlier_open(&mut *conn, company_id, y.start_date)
        .await?
        .map(|fy| FiscalYearRef {
            id: fy.id,
            name: fy.name,
        });
    Ok(ClosingCheck {
        code: CheckCode::EarlierFiscalYearOpen,
        severity: Severity::Blocking,
        status: status_of(earlier.is_some()),
        count: i64::from(earlier.is_some()),
        details: CheckDetails::EarlierFiscalYearOpen(EarlierOpenDetails {
            fiscal_year: earlier,
        }),
    })
}

/// K2 — transactions bancaires en attente, datées dans Y.
async fn pending_bank_transactions(
    conn: &mut MySqlConnection,
    company_id: i64,
    y: &FiscalYear,
) -> Result<ClosingCheck, DbError> {
    let rows: Vec<(i64, String, Option<String>, i64, i64)> = sqlx::query_as(PENDING_SQL)
        .bind(company_id)
        .bind(y.start_date)
        .bind(y.end_date)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    let accounts: Vec<PendingByAccount> = rows
        .into_iter()
        .map(
            |(id, bank_name, description, n, rejected)| PendingByAccount {
                bank_account_id: id,
                name: bank_account_display_name(&bank_name, description.as_deref()),
                count: n,
                rejected_count: rejected,
            },
        )
        .collect();
    let total: i64 = accounts.iter().map(|a| a.count).sum();
    Ok(ClosingCheck {
        code: CheckCode::PendingBankTransactions,
        severity: Severity::Warning,
        status: status_of(total > 0),
        count: total,
        details: CheckDetails::PendingBankTransactions(PendingDetails { accounts }),
    })
}

/// Un compte bancaire examiné par K3 ([`EXAMINED_ACCOUNTS_SQL`]).
#[derive(sqlx::FromRow)]
struct ExaminedAccount {
    id: i64,
    bank_name: String,
    description: Option<String>,
    journal_account_id: Option<i64>,
    closed_on: Option<NaiveDate>,
    has_balance_statement: i64,
}

/// K3 — le relevé de fin d'exercice, compte par compte, avec sa table de
/// précédence (le **premier** motif qui s'applique) :
///
/// 1. `NOT_COMPARABLE` — sans compte lié, ou compte lié partagé → `notCheckable` ;
/// 2. `NO_BALANCE_STATEMENT` — aucun relevé à solde, toutes dates → `notCheckable` ;
/// 3. `MISSING` — pas de relevé arrêté au plus tard le dernier jour de Y, ou
///    arrêté avant Y ;
/// 4. `ENDS_BEFORE_YEAR_END` — relevé arrêté avant `MIN(Y.end_date, closed_on)` ;
/// 5. `GAP` — [`bank_accounts::statement_gap`] n'est pas nul.
///
/// La borne SQL du relevé est `Y.end_date` pour tous les comptes ; la clôture
/// d'un compte n'intervient qu'ici, en Rust, au motif 4.
async fn bank_statement_at_year_end(
    conn: &mut MySqlConnection,
    company_id: i64,
    y: &FiscalYear,
) -> Result<ClosingCheck, DbError> {
    let examined: Vec<ExaminedAccount> = sqlx::query_as(EXAMINED_ACCOUNTS_SQL)
        .bind(company_id)
        .bind(y.start_date) // closed_on >= Y.start
        .bind(y.start_date) // transactions
        .bind(y.end_date)
        .bind(y.start_date) // relevés
        .bind(y.end_date)
        .bind(y.start_date) // grand livre
        .bind(y.end_date)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;

    let mut accounts = Vec::new();
    let mut not_checkable = Vec::new();
    if !examined.is_empty() {
        let rows =
            bank_accounts::balances_by_bank_account(&mut *conn, company_id, Some(y.end_date))
                .await?;
        for acc in examined {
            let row = rows.get(&acc.id);
            let name = bank_account_display_name(&acc.bank_name, acc.description.as_deref());
            if acc.journal_account_id.is_none() || bank_accounts::is_shared(row) {
                not_checkable.push(NotCheckable {
                    bank_account_id: acc.id,
                    name,
                    reason: NotCheckableReason::NotComparable,
                });
                continue;
            }
            if acc.has_balance_statement == 0 {
                not_checkable.push(NotCheckable {
                    bank_account_id: acc.id,
                    name,
                    reason: NotCheckableReason::NoBalanceStatement,
                });
                continue;
            }
            let balances = bank_accounts::balances_from_row(acc.journal_account_id, row);
            let gap = bank_accounts::statement_gap(&balances);
            let measured_through = acc.closed_on.map_or(y.end_date, |c| c.min(y.end_date));
            let reason = match balances.statement_date {
                None => StatementReason::Missing,
                Some(d) if d < y.start_date => StatementReason::Missing,
                Some(d) if d < measured_through => StatementReason::EndsBeforeYearEnd,
                Some(_) if gap.is_some() => StatementReason::Gap,
                Some(_) => continue,
            };
            accounts.push(StatementIssue {
                bank_account_id: acc.id,
                name,
                reason,
                statement_date: balances.statement_date,
                statement_closing_balance: balances.statement_closing_balance,
                ledger_balance_at_statement: balances.ledger_balance_at_statement,
                gap,
            });
        }
    }
    let count = accounts.len() as i64;
    Ok(ClosingCheck {
        code: CheckCode::BankStatementAtYearEnd,
        severity: Severity::Warning,
        status: status_of(count > 0),
        count,
        details: CheckDetails::BankStatementAtYearEnd(StatementDetails {
            accounts,
            not_checkable,
        }),
    })
}

/// K4 — factures client en brouillon, datées dans Y.
async fn draft_invoices(
    conn: &mut MySqlConnection,
    company_id: i64,
    y: &FiscalYear,
) -> Result<ClosingCheck, DbError> {
    let count: i64 = sqlx::query_scalar(DRAFT_COUNT_SQL)
        .bind(company_id)
        .bind(y.start_date)
        .bind(y.end_date)
        .fetch_one(&mut *conn)
        .await
        .map_err(map_db_error)?;
    let invoices = if count == 0 {
        Vec::new()
    } else {
        let rows: Vec<(i64, String, NaiveDate)> = sqlx::query_as(DRAFT_LIST_SQL)
            .bind(company_id)
            .bind(y.start_date)
            .bind(y.end_date)
            .bind(MAX_LISTED as i64)
            .fetch_all(&mut *conn)
            .await
            .map_err(map_db_error)?;
        rows.into_iter()
            .map(|(id, label, date)| DraftInvoiceRef { id, label, date })
            .collect()
    };
    Ok(ClosingCheck {
        code: CheckCode::DraftInvoices,
        severity: Severity::Info,
        status: status_of(count > 0),
        count,
        details: CheckDetails::DraftInvoices(DraftDetails { invoices }),
    })
}

/// K5 — pièces fournisseur importées à compléter (non borné à l'exercice).
async fn imported_supplier_invoices_to_complete(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<ClosingCheck, DbError> {
    let count: i64 = sqlx::query_scalar(IMPORTED_SQL)
        .bind(company_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(map_db_error)?;
    Ok(ClosingCheck {
        code: CheckCode::ImportedSupplierInvoicesToComplete,
        severity: Severity::Info,
        status: status_of(count > 0),
        count,
        details: CheckDetails::ImportedSupplierInvoicesToComplete(ImportedDetails {}),
    })
}

/// Une écriture relevée par [`INCONSISTENT_SQL`].
#[derive(sqlx::FromRow)]
struct InconsistentRow {
    id: i64,
    entry_number: i64,
    entry_date: NaiveDate,
    fiscal_year_name: String,
    unbalanced: i64,
    too_few_lines: i64,
    date_outside_its_year: i64,
    other_year_dated_inside: i64,
}

/// K6 — écritures incohérentes : `count` les compte **distinctes**,
/// `countByReason` par motif (une écriture peut compter sous deux), la liste
/// est bornée à [`MAX_LISTED`], dans l'ordre `entry_date, id`.
async fn inconsistent_entries(
    conn: &mut MySqlConnection,
    company_id: i64,
    y: &FiscalYear,
) -> Result<ClosingCheck, DbError> {
    let rows: Vec<InconsistentRow> = sqlx::query_as(INCONSISTENT_SQL)
        .bind(y.id) // unbalanced
        .bind(y.id) // too_few_lines
        .bind(y.id) // date_outside_its_year
        .bind(y.start_date)
        .bind(y.end_date)
        .bind(y.id) // other_year_dated_inside
        .bind(y.start_date)
        .bind(y.end_date)
        .bind(company_id) // table dérivée des lignes
        .bind(company_id) // je.company_id
        .bind(y.id) // population : écritures de Y…
        .bind(y.start_date) // … ou datées dans Y
        .bind(y.end_date)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;

    let mut by_reason = CountByReason::default();
    let mut entries = Vec::new();
    let count = rows.len() as i64;
    for r in rows {
        let mut reasons = Vec::new();
        for (flag, reason, counter) in [
            (
                r.unbalanced,
                EntryReason::Unbalanced,
                &mut by_reason.unbalanced,
            ),
            (
                r.too_few_lines,
                EntryReason::TooFewLines,
                &mut by_reason.too_few_lines,
            ),
            (
                r.date_outside_its_year,
                EntryReason::DateOutsideItsYear,
                &mut by_reason.date_outside_its_year,
            ),
            (
                r.other_year_dated_inside,
                EntryReason::OtherYearDatedInside,
                &mut by_reason.other_year_dated_inside,
            ),
        ] {
            if flag != 0 {
                reasons.push(reason);
                *counter += 1;
            }
        }
        if entries.len() < MAX_LISTED {
            entries.push(InconsistentEntry {
                id: r.id,
                entry_number: r.entry_number,
                entry_date: r.entry_date,
                fiscal_year_name: r.fiscal_year_name,
                reasons,
            });
        }
    }
    Ok(ClosingCheck {
        code: CheckCode::InconsistentEntries,
        severity: Severity::Anomaly,
        status: status_of(count > 0),
        count,
        details: CheckDetails::InconsistentEntries(InconsistentDetails {
            count_by_reason: by_reason,
            entries,
        }),
    })
}

/// K7 — l'exercice qui suit Y (le premier dont `start_date > Y.end_date`).
async fn next_fiscal_year(
    conn: &mut MySqlConnection,
    company_id: i64,
    y: &FiscalYear,
) -> Result<ClosingCheck, DbError> {
    let next = fiscal_years::find_next_after(&mut *conn, company_id, y.end_date)
        .await?
        .map(|fy| NextFiscalYearRef {
            id: fy.id,
            name: fy.name,
            start_date: fy.start_date,
            status: fy.status,
        });
    let missing = next.is_none();
    Ok(ClosingCheck {
        code: CheckCode::NextFiscalYear,
        severity: Severity::Info,
        status: status_of(missing),
        count: i64::from(missing),
        details: CheckDetails::NextFiscalYear(NextFiscalYearDetails { fiscal_year: next }),
    })
}

#[cfg(test)]
mod tests {
    //! T-a2 (AC2, C-29-1-26) — garde statique **inversée** : au lieu
    //! d'énumérer les formes interdites, elle inventorie les sites de requête
    //! qui ne résolvent **pas** en une constante de `ALL_SQL`. Deux volets
    //! seulement ; les fonctions de dépôt externes sont un angle mort assumé
    //! (doc du module).

    /// Le source du module **hors tests** : tout ce qui précède le premier
    /// `#[cfg(test)]`.
    fn hors_tests(src: &str) -> &str {
        src.split("#[cfg(test)]").next().unwrap_or(src)
    }

    /// Les lignes dont le premier non-blanc est `//` (donc `///`, `//!`) retirées.
    fn decommente(src: &str) -> String {
        src.lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn est_ident(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    /// Les noms énumérés par `const ALL_SQL: &[&str] = &[ … ];`.
    fn noms_de_all_sql(src: &str) -> Vec<String> {
        let debut = src
            .find("const ALL_SQL: &[&str] = &[")
            .expect("le module déclare `const ALL_SQL: &[&str] = &[…]`");
        let reste = &src[debut + "const ALL_SQL: &[&str] = &[".len()..];
        let fin = reste.find("];").expect("fin de ALL_SQL");
        reste[..fin]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Nombre de `const …_SQL: &str` du source.
    fn nb_constantes_sql(src: &str) -> usize {
        let mut n = 0;
        let mut reste = src;
        while let Some(i) = reste.find("const ") {
            let apres = &reste[i + "const ".len()..];
            let nom: String = apres.chars().take_while(|c| est_ident(*c)).collect();
            let suite = &apres[nom.len()..];
            if nom.ends_with("_SQL") && suite.trim_start().starts_with(": &str") {
                n += 1;
            }
            reste = apres;
        }
        n
    }

    /// Volet (a) : chaque site `query`, `query_as`, `query_scalar` suivi de
    /// `::<` ou de `(` — source mis sur une ligne, turbofish sauté en
    /// équilibrant `<` et `>` — dont le premier argument n'est pas une
    /// constante de `ALL_SQL`. Rend les non résolus, nommés par un extrait.
    fn non_resolus(src: &str, noms: &[String]) -> Vec<String> {
        let plat: Vec<char> = src.replace('\n', " ").chars().collect();
        let texte: String = plat.iter().collect();
        let mut sites = Vec::new();
        let mut i = 0;
        while i < plat.len() {
            let precedent_ok = i == 0 || !est_ident(plat[i - 1]);
            let mot = ["query_scalar", "query_as", "query"]
                .into_iter()
                .find(|w| texte[char_offset(&plat, i)..].starts_with(w));
            let Some(mot) = mot.filter(|_| precedent_ok) else {
                i += 1;
                continue;
            };
            let mut j = i + mot.len();
            if j < plat.len() && est_ident(plat[j]) {
                i += 1;
                continue;
            }
            while j < plat.len() && plat[j] == ' ' {
                j += 1;
            }
            // Turbofish : `::<…>`, équilibré.
            if plat[j..].starts_with(&[':', ':', '<']) {
                j += 2;
                let mut profondeur = 0i32;
                while j < plat.len() {
                    match plat[j] {
                        '<' => profondeur += 1,
                        '>' => {
                            profondeur -= 1;
                            if profondeur == 0 {
                                j += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                while j < plat.len() && plat[j] == ' ' {
                    j += 1;
                }
            }
            if j >= plat.len() || plat[j] != '(' {
                i += mot.len();
                continue; // le mot sans appel (prose, chemin d'import)
            }
            j += 1;
            while j < plat.len() && plat[j] == ' ' {
                j += 1;
            }
            let arg: String = plat[j..].iter().take_while(|c| est_ident(**c)).collect();
            let resolu = arg.ends_with("_SQL")
                && arg.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && arg
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                && noms.contains(&arg);
            if !resolu {
                let extrait: String = plat[i..plat.len().min(i + 80)].iter().collect();
                sites.push(extrait);
            }
            i = j;
        }
        sites
    }

    fn char_offset(plat: &[char], i: usize) -> usize {
        plat[..i].iter().map(|c| c.len_utf8()).sum()
    }

    /// Volet (b) : les mots de verrou du source décommenté, en majuscules.
    fn mots_de_verrou(src: &str) -> Vec<&'static str> {
        let haut = src.to_uppercase();
        ["FOR UPDATE", "LOCK IN SHARE MODE", "FOR SHARE"]
            .into_iter()
            .filter(|m| haut.contains(m))
            .collect()
    }

    /// T-a2 — appliquée au source réel du module.
    #[test]
    fn t_a2_aucune_requete_hors_all_sql_et_aucun_verrou() {
        let src = include_str!("closing_checks.rs");
        let code = decommente(hors_tests(src));
        let noms = noms_de_all_sql(&code);
        assert!(!noms.is_empty(), "ALL_SQL est vide");
        let nr = non_resolus(&code, &noms);
        assert!(nr.is_empty(), "sites de requête non résolus : {nr:#?}");
        assert_eq!(
            nb_constantes_sql(&code),
            noms.len(),
            "toute `const …_SQL: &str` du module figure dans ALL_SQL (inventaire clos)"
        );
        let v = mots_de_verrou(&code);
        assert!(v.is_empty(), "mots de verrou dans le source : {v:?}");
    }

    /// T-a2 — cas de contrôle du détecteur : la forme dominante du dépôt
    /// (turbofish à signature imbriquée, appel coupé par rustfmt, littéral)
    /// est vue par les deux volets ; une constante énumérée est résolue.
    #[test]
    fn t_a2_le_detecteur_voit_la_forme_dominante() {
        let synthetique = "const A_SQL: &str = \"SELECT 1\";\n\
                           const ALL_SQL: &[&str] = &[A_SQL];\n\
                           let r = sqlx::query_as::<_, (i64, Option<String>)>(\n    \"SELECT 1 FOR UPDATE\")\n\
                           .fetch_one(conn);\n\
                           let s = sqlx::query_scalar(A_SQL).fetch_one(conn);\n\
                           let t = sqlx::query(&format!(\"{A_SQL} x\"));\n";
        let noms = noms_de_all_sql(synthetique);
        assert_eq!(noms, vec!["A_SQL".to_string()]);
        let nr = non_resolus(synthetique, &noms);
        assert_eq!(
            nr.len(),
            2,
            "le turbofish coupé et le format! sont non résolus : {nr:#?}"
        );
        assert!(
            nr[0].starts_with("query_as::<_, (i64, Option<String>)>("),
            "{nr:#?}"
        );
        assert!(nr[1].starts_with("query(&format!"), "{nr:#?}");
        assert_eq!(mots_de_verrou(synthetique), vec!["FOR UPDATE"]);
        // Un commentaire qui cite le mot de verrou ne fait pas rougir.
        assert!(mots_de_verrou(&decommente("/// aucun FOR UPDATE ici\nlet x = 1;")).is_empty());
        assert_eq!(nb_constantes_sql(synthetique), 1);
    }
}
