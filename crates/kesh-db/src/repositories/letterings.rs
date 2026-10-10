//! Lettrage — la marque, posée et retirée par un seul chemin (Story 15-1a-i, #518).
//!
//! Un **groupe de lettrage** est un ensemble d'au moins deux lignes
//! d'écriture d'une même société, sur un même compte **lettrable**, dont la
//! somme `Σ(débit − crédit)` est exactement nulle. La marque est portée par
//! chaque ligne (`journal_entry_lines.lettering_key`, `lettering_origin`, R1) ;
//! la clé du groupe est le **plus petit `id`** de ses lignes, son code affiché
//! cette clé écrite en lettres ([`kesh_core::lettering::code_from_key`], R2).
//!
//! ⛔ **La marque s'écrit dans UNE fonction — `create_group_inner` — et se
//! retire dans UNE autre — `dissolve_group_inner`** (R3), atteintes par les
//! deux primitives ([`create_group_in_tx`], [`dissolve_group_in_tx`]) et par la
//! synchronisation des pièces ([`sync_invoice_in_tx`],
//! [`dissolve_invoice_document_group_in_tx`], Story 15-1a2-i ;
//! [`sync_supplier_invoice_in_tx`],
//! [`dissolve_supplier_invoice_document_group_in_tx`], Story 15-1a2-ii)
//! **seules** — la synchronisation **appelle** ces corps, elle n'écrit pas la
//! marque. Aucun autre `UPDATE` ni `INSERT` du code de production ne nomme
//! `lettering_key` ou `lettering_origin` : le test lexical
//! `crates/kesh-db/tests/letterings_lexical.rs` le vérifie. Exceptions,
//! nommées : les **deux migrations de rattrapage** de la 15-1a2-ii
//! (`20261010000001_lettering_documents_backfill.sql`,
//! `20261010000002_lettering_reversal_pairs_backfill.sql` — SQL de migration) ;
//! le **rejeu à l'import** de la première (`post_restore::replay_post_restore_backfills`,
//! entrée de classe A de `POST_RESTORE_BACKFILLS`) — un écrivain de production
//! de la marque, hors des deux corps et **sans audit**, à chaque import ; la
//! **restauration d'une sauvegarde** (`backup.rs`, colonnes lues
//! dynamiquement — un groupe restauré est celui qui existait) ; et les
//! suppressions en bloc qui emportent les lignes **avec** leurs marques
//! (`journal_entries::delete_all_by_company`, `reset_demo` de `kesh-seed`).
//!
//! # Séquence des deux primitives (R7)
//!
//! 1. **Premier acte : une lecture VERROUILLANTE qui découvre et verrouille à
//!    la fois** — lignes **et** en-têtes, par une jointure `FOR UPDATE` :
//!    [`LOCK_LINES_BY_ID_SQL`] (création, parcours de la clé primaire des
//!    lignes) ou [`LOCK_LINES_BY_KEY_SQL`] (dissolution, parcours
//!    d'`idx_jel_lettering`). Aucune lecture ordinaire avant elle. Moins de
//!    lignes que demandées (création) ou aucune (dissolution) →
//!    [`DbError::NotFound`], indiscernable d'une ligne d'une autre société.
//!    ⚠️ Limite écrite (F-11) : le filtre `je.company_id` s'applique après le
//!    parcours, si bien qu'un identifiant d'une autre société est verrouillé
//!    le temps de la transaction — le **contenu** du 404 est indiscernable,
//!    sa **durée** ne l'est pas.
//! 2. **Mode `Manual` seulement — les exercices du groupe, un par un, par clé
//!    primaire, dans l'ordre chronologique** : (a) lecture **non
//!    verrouillante** de leurs `id`, `start_date` et `name`, triés par
//!    `start_date` ([`FISCAL_YEARS_OF_GROUP_SQL`] — `start_date` est immuable) ;
//!    (b) verrou de chacun, dans cet ordre, par une boucle Rust
//!    ([`LOCK_LETTERING_FISCAL_YEAR_SQL`]) — un `ORDER BY` ne fixe pas l'ordre
//!    d'acquisition des verrous, la boucle si ; le `status` est lu sous ce
//!    verrou ; (c) « un exercice postérieur est clos », lu **sans verrou**
//!    (`fiscal_years::find_later_closed`), pour chaque exercice ouvert du
//!    groupe — sûr par l'argument (α)-(β) de la Story 15-12b : sous la clôture
//!    dans l'ordre (15-12a), aucune transition ne crée « X ouvert, postérieur
//!    clos », et la clôture d'un postérieur attend l'exercice que tient (b).
//!    ⚠️ (a) est la **première lecture ordinaire** : elle ouvre la vue
//!    `REPEATABLE READ` avant l'attente des verrous de (b). Ce qui se lit
//!    ensuite ordinairement (lettrabilité, appartenance à une pièce, borne
//!    `books_locked_through`) lit l'état d'avant l'attente — chaque cas est
//!    tolérable, la fiche de la story le démontre ; la doctrine « le verrou
//!    d'abord » est tenue pour ce que le lettrage **écrit** ou **juge sous
//!    verrou** (lignes, en-têtes, `status` des exercices).
//! 3. **Mode `System { held_open_fiscal_year_id }`** : la primitive ne
//!    verrouille AUCUN exercice — l'appelant en tient déjà un, ouvert, `FOR
//!    UPDATE` ; elle vérifie, sans requête, qu'une ligne lue à l'acte 1 le
//!    porte (sinon [`DbError::Invariant`], défaut de l'appelant), puis lit le
//!    **nom** des exercices du groupe par une lecture **ordinaire, non
//!    verrouillante** ([`LETTERING_FISCAL_YEAR_NAMES_SQL`]) pour la réponse et
//!    l'audit. ⛔ Pas de jointure de `fiscal_years` dans l'acte 1 : sous `FOR
//!    UPDATE`, elle verrouillerait les exercices hors de l'ordre de la clôture.
//! 4. L'`UPDATE` final vise des lignes **tenues** ; son compte de lignes
//!    trouvées passe par [`kesh_core::lettering::check_rows_affected`] — un
//!    écart rend [`DbError::LetteringConcurrentChange`] (409), jamais un
//!    succès partiel ni un 500.
//!
//! ⚠️ **Ces requêtes sont reconnues à leur texte par les tests**
//! (`attendre_une_requete_en_cours`) : acte 1 de la création `["jel.id IN",
//! "FOR UPDATE"]`, de la dissolution `["jel.lettering_key = ", "FOR UPDATE"]`,
//! verrou d'exercice `["SELECT id, start_date, status FROM fiscal_years WHERE
//! id = ", "FOR UPDATE"]` — sa liste de colonnes l'oppose à
//! `LOCK_EARLIER_BY_ID_SQL` (`SELECT id FROM …`) et à l'étape (c) de la clôture
//! (`SELECT id, company_id …`). Les changer, c'est changer leurs motifs.
//!
//! # La règle des périodes (R7, mode `Manual`)
//!
//! Une ligne est **« en période ouverte »** si son exercice est `Open`, si
//! aucun exercice postérieur n'est `Closed`, et si sa date est strictement
//! postérieure à `companies.books_locked_through` (seuil inclusif). **Lettrer
//! comme délettrer** exige qu'au moins UNE ligne du groupe le soit — un groupe
//! tout entier en période close ne se crée ni ne se défait
//! ([`DbError::LetteringAllLinesInClosedPeriods`]). La borne se lit
//! ordinairement, sans verrou sur `companies` : une pose de borne commitée
//! pendant l'attente des verrous n'est pas vue (même tolérance qu'à la
//! création d'écriture).
//!
//! # Interblocages résiduels
//!
//! Nommés à la fiche (R7) : avec `update_in_tx`/`delete_in_tx` (en-tête puis
//! lignes, quand l'acte 1 prend une ligne avant son en-tête), avec la
//! contre-passation, et — mesuré en T0 — avec **toute insertion de ligne**
//! pendant la dissolution du groupe de plus petite clé (verrou du trou
//! d'`idx_jel_lettering` où s'insèrent les lignes ouvertes, toutes sociétés).
//! Les routes qui les portent sont toutes rejouées sur interblocage (AC12) :
//! l'ordre réduit la fréquence, le rejeu est la défense (Pattern 5).
//!
//! La **synchronisation des pièces** (Story 15-1a2-i, [`sync_invoice_in_tx`])
//! s'exécute après les verrous du geste (facture, écriture, exercice) et
//! prend ensuite, par sa découverte, `invoice_settlements` et `credit_notes` de
//! la facture, puis les lignes et en-têtes des écritures de la pièce. Ce
//! qu'elle ajoute aux verrous des gestes (revue P2, B2-1 = E2-4 ; non mesuré) :
//! - **toutes** les lignes des écritures parcourues par `idx_jel_entry` — produit,
//!   TVA, banque compris, non seulement celles sur la créance —, et le verrou de
//!   clé suivante de cet index non unique, qui fait attendre jusqu'au `COMMIT`
//!   l'insertion de lignes d'une écriture d'identifiant voisin (toutes sociétés) ;
//! - les verrous d'intervalle des lectures `FOR UPDATE` d'`invoice_settlements`
//!   et de `credit_notes` par leurs index de facture, qui peuvent faire attendre
//!   l'insertion d'un règlement ou d'un avoir d'une facture **voisine** (revue
//!   P1, B-2 = E-1).
//!
//! Ce sont des **attentes** ; aucun cycle neuf n'a été établi. Les gestes d'une
//! même facture restent sérialisés par la facture : règlement, solde, avoir et
//! annulation la verrouillent d'abord ; le rapprochement y pose un verrou
//! partagé dès l'`INSERT` d'`invoice_settlements` (clé étrangère), monté en
//! exclusif à l'`UPDATE … AND version = ?` — montée et interblocage antérieurs
//! à cette story. Aucune absence de cycle n'est affirmée pour autant : toutes
//! les routes appelantes sont rejouées (`Rejouee`, et `retry_with` pour
//! `accept_batch`).

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{MySql, MySqlConnection, Transaction};

use kesh_core::lettering::{self as core_lettering, LetteringRefusal};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::errors::{DbError, map_db_error};
use crate::repositories::journal_entries::DocumentKind;
use crate::repositories::{audit_log, fiscal_years, journal_entries};

/// La vue des postes ouverts et les propositions (Story 15-1b) — lecture
/// seule, sur les fonctions de ce module (lettrabilité, règle des périodes,
/// code d'un groupe).
mod open_items;
pub use open_items::{
    DocumentState, LetteringProposal, LetteringProposals, OPEN_ITEMS_BALANCE_SQL,
    OPEN_ITEMS_PAGE_SQL, OPEN_ITEMS_TOTALS_SQL, OpenItem, OpenItems, OpenReason, ProposalLine,
    lettering_proposals, open_items, open_items_invoice_states_sql,
};

/// Origine d'un groupe de lettrage (colonne `lettering_origin`, contrainte
/// `chk_jel_lettering_origin`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Posé par Kesh quand une **pièce** est soldée (Story 15-1a2).
    Document,
    /// Posé par Kesh entre une ligne et sa contre-passation (Story 15-1a-ii).
    Reversal,
    /// Choisi par l'utilisateur (`POST /api/v1/letterings`).
    Manual,
}

impl Origin {
    /// Valeur stockée en base et exposée par l'API.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Reversal => "reversal",
            Self::Manual => "manual",
        }
    }

    /// Lit la valeur stockée ; `None` pour une valeur inconnue.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "document" => Some(Self::Document),
            "reversal" => Some(Self::Reversal),
            "manual" => Some(Self::Manual),
            _ => None,
        }
    }
}

/// Mode d'appel d'une primitive (R3, R7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Les routes : la primitive verrouille les exercices du groupe et applique
    /// la règle des périodes et le refus des lignes d'une pièce (R5).
    Manual,
    /// La contre-passation (15-1a-ii) et les pièces (15-1a2) : l'appelant tient
    /// déjà, `FOR UPDATE`, un exercice **ouvert** qui couvre une ligne du groupe.
    System { held_open_fiscal_year_id: i64 },
}

/// L'auteur d'un geste de lettrage, audité par `NewAuditLogEntry::for_actor`.
///
/// ⚠️ La contre-passation ne porte que l'utilisateur (`reverse_in_tx`) : le
/// lettrage `reversal` passe `api_key_id: None` — écart nommé (R3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Actor {
    pub user_id: i64,
    pub api_key_id: Option<i64>,
}

/// Une ligne d'un groupe, telle que la réponse et l'audit la décrivent.
///
/// ⚠️ `fiscal_year_id` et `fiscal_year_name` par ligne : le numéro d'écriture
/// repart à 1 à chaque exercice — dans un groupe à cheval, deux lignes peuvent
/// porter « écriture n° 12 » de deux exercices (C127).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetteringLine {
    pub id: i64,
    pub entry_id: i64,
    pub entry_number: i64,
    pub fiscal_year_id: i64,
    pub fiscal_year_name: String,
    pub date: NaiveDate,
    pub debit: Decimal,
    pub credit: Decimal,
}

/// Un groupe de lettrage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetteringGroup {
    pub key: i64,
    pub code: String,
    pub origin: Origin,
    pub account_id: i64,
    pub account_number: String,
    /// Triées par `id` croissant.
    pub lines: Vec<LetteringLine>,
}

/// Acte 1 de la création : lignes **et** en-têtes, par la clé primaire des
/// lignes. `{ids}` est remplacé par autant de `?` que d'identifiants.
const LOCK_LINES_BY_ID_SQL: &str = "SELECT jel.id, jel.entry_id, jel.account_id, jel.debit, \
     jel.credit, jel.lettering_key, jel.lettering_origin, je.fiscal_year_id, je.entry_date, \
     je.entry_number \
     FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
     WHERE jel.id IN ({ids}) AND je.company_id = ? ORDER BY jel.id FOR UPDATE";

/// Acte 1 de la dissolution : lignes **et** en-têtes d'un groupe, par
/// `idx_jel_lettering` (le verrou de clé suivante empêche aussi une
/// reformation concurrente sous la même clé).
const LOCK_LINES_BY_KEY_SQL: &str = "SELECT jel.id, jel.entry_id, jel.account_id, jel.debit, \
     jel.credit, jel.lettering_key, jel.lettering_origin, je.fiscal_year_id, je.entry_date, \
     je.entry_number \
     FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
     WHERE jel.lettering_key = ? AND je.company_id = ? ORDER BY jel.id FOR UPDATE";

/// Lecture d'un groupe **sans verrou** (`GET`).
const FIND_GROUP_SQL: &str = "SELECT jel.id, jel.entry_id, jel.account_id, jel.debit, \
     jel.credit, jel.lettering_key, jel.lettering_origin, je.fiscal_year_id, je.entry_date, \
     je.entry_number \
     FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
     WHERE jel.lettering_key = ? AND je.company_id = ? ORDER BY jel.id";

/// R7 point 2 (a) — les exercices du groupe, **sans verrou**, triés par
/// `start_date` (immuable). `{ids}` : autant de `?` que d'exercices distincts.
const FISCAL_YEARS_OF_GROUP_SQL: &str = "SELECT id, start_date, name FROM fiscal_years \
     WHERE id IN ({ids}) AND company_id = ? ORDER BY start_date ASC";

/// R7 point 2 (b) — verrou d'un exercice du groupe **par clé primaire**. Le
/// `status` lu ici, sous le verrou, est celui que juge la règle des périodes.
///
/// ⚠️ Texte reconnu par les tests (motifs `"SELECT id, start_date, status FROM
/// fiscal_years WHERE id = "`, `"FOR UPDATE"`) : sa liste de colonnes l'oppose
/// aux verrous de la clôture.
const LOCK_LETTERING_FISCAL_YEAR_SQL: &str =
    "SELECT id, start_date, status FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE";

/// [`open_period_rule`] — statut, date de début et nom des exercices nommés,
/// **sans verrou**. `{ids}` : autant de `?` que d'exercices distincts.
const OPEN_PERIOD_RULE_FISCAL_YEARS_SQL: &str = "SELECT id, start_date, status, name \
     FROM fiscal_years WHERE id IN ({ids}) AND company_id = ?";

/// [`document_group_frozen_by_periods`] — les clés des groupes d'origine
/// `document` qui contiennent une ligne de l'écriture, **sans verrou**, par
/// ordre croissant.
const DOCUMENT_KEYS_OF_ENTRY_SQL: &str = "SELECT DISTINCT jel.lettering_key \
     FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
     WHERE jel.entry_id = ? AND je.company_id = ? AND jel.lettering_origin = 'document' \
     AND jel.lettering_key IS NOT NULL ORDER BY jel.lettering_key";

/// [`document_group_frozen_by_periods`] — `(fiscal_year_id, entry_date)` de
/// chaque ligne d'un groupe, **sans verrou**.
const GROUP_LINE_PERIODS_SQL: &str = "SELECT je.fiscal_year_id, je.entry_date \
     FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
     WHERE jel.lettering_key = ? AND je.company_id = ?";

/// R7 point 3 — le **nom** des exercices du groupe en mode `System`, par une
/// lecture **ordinaire, non verrouillante** (C128). Ne juge rien.
const LETTERING_FISCAL_YEAR_NAMES_SQL: &str =
    "SELECT id, name FROM fiscal_years WHERE company_id = ? AND id IN ({ids})";

/// Une ligne lue par l'acte 1 (ou par [`FIND_GROUP_SQL`]).
#[derive(Debug, Clone, sqlx::FromRow)]
struct LineRow {
    id: i64,
    entry_id: i64,
    account_id: i64,
    debit: Decimal,
    credit: Decimal,
    lettering_key: Option<i64>,
    lettering_origin: Option<String>,
    fiscal_year_id: i64,
    entry_date: NaiveDate,
    entry_number: i64,
}

/// Remplace `{ids}` par `n` marqueurs `?`.
fn with_placeholders(sql: &str, n: usize) -> String {
    sql.replace("{ids}", &vec!["?"; n].join(", "))
}

/// Convertit un refus pur de `kesh-core` en erreur de persistance.
fn refusal(r: LetteringRefusal) -> DbError {
    match r {
        LetteringRefusal::TooFewLines => DbError::LetteringTooFewLines,
        LetteringRefusal::AccountsDiffer => DbError::LetteringAccountsDiffer,
        LetteringRefusal::LineAlreadyLettered { key } => {
            DbError::LetteringLineAlreadyLettered { code: code_of(key) }
        }
        LetteringRefusal::Unbalanced { difference } => DbError::LetteringUnbalanced { difference },
        LetteringRefusal::ConcurrentChange => DbError::LetteringConcurrentChange,
    }
}

/// Code affiché d'une clé (une clé est un identifiant de ligne, donc ≥ 1).
pub fn code_of(key: i64) -> String {
    u64::try_from(key)
        .map(core_lettering::code_from_key)
        .unwrap_or_default()
}

/// Plafond de lignes d'un groupe **manuel** (AC6) — refus de forme, à placer
/// avant toute lecture de la base.
pub fn check_manual_line_count(line_ids: &[i64]) -> Result<(), DbError> {
    if line_ids.len() > core_lettering::MAX_LINES_PER_GROUP {
        return Err(DbError::LetteringTooManyLines {
            max: core_lettering::MAX_LINES_PER_GROUP,
        });
    }
    Ok(())
}

/// R4, la condition « un compte bancaire le désigne » — **archivé ou non**
/// (C127) — écrite UNE fois, sur l'alias `a` de `accounts` : lue par
/// [`letterable_account`] (une ligne) et [`letterable_account_ids`] (en lot).
const BANK_LINKED_SQL: &str =
    "EXISTS (SELECT 1 FROM bank_accounts b WHERE b.journal_account_id = a.id)";

/// R4, le prédicat **pur** (Story 15-1b, AC11 ; C-15-1b-7) : un compte est
/// lettrable s'il est de type `Asset` ou `Liability` (`account_type` tel que la
/// base le lit — l'entité passe `AccountType::as_str()`) et qu'aucun compte
/// bancaire ne le désigne. La seule écriture de la règle, appelée par
/// [`letterable_account`] et [`letterable_account_ids`].
pub fn is_letterable(account_type: &str, bank_linked: bool) -> bool {
    matches!(account_type, "Asset" | "Liability") && !bank_linked
}

/// Un compte est **lettrable** (R4) s'il appartient à la société, est de type
/// `Asset` ou `Liability`, et qu'aucun `bank_accounts.journal_account_id` ne le
/// désigne — **archivé ou non** (C127 : un compte qui a été celui d'un compte
/// bancaire relève de la réconciliation, et ne redevient pas lettrable à
/// l'archivage). Un compte archivé (`accounts.active = FALSE`) reste lettrable :
/// ses lignes existent et se soldent. La règle est [`is_letterable`].
///
/// Rend aussi le **numéro** du compte (pour l'audit), `None` si le compte est
/// introuvable dans la société.
///
/// Lecture ordinaire. Fonction partagée avec la vue des postes ouverts (15-1b).
pub async fn letterable_account(
    conn: &mut MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<(bool, String)>, DbError> {
    let sql = format!(
        "SELECT a.account_type, a.number, {BANK_LINKED_SQL} \
         FROM accounts a WHERE a.id = ? AND a.company_id = ?"
    );
    let row: Option<(String, String, bool)> = sqlx::query_as(&sql)
        .bind(account_id)
        .bind(company_id)
        .fetch_optional(conn)
        .await
        .map_err(map_db_error)?;
    Ok(row.map(|(account_type, number, banque)| (is_letterable(&account_type, banque), number)))
}

/// Les comptes **lettrables** de la société, archivés compris, en **une**
/// requête (Story 15-1b, AC11) — pour `GET /accounts`, sans N+1 sur le plan
/// comptable. Même prédicat ([`is_letterable`]) et même condition SQL
/// ([`BANK_LINKED_SQL`]) que [`letterable_account`].
pub async fn letterable_account_ids(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<BTreeSet<i64>, DbError> {
    let sql = format!(
        "SELECT a.id, a.account_type, {BANK_LINKED_SQL} FROM accounts a WHERE a.company_id = ?"
    );
    let rows: Vec<(i64, String, bool)> = sqlx::query_as(&sql)
        .bind(company_id)
        .fetch_all(conn)
        .await
        .map_err(map_db_error)?;
    Ok(rows
        .into_iter()
        .filter(|(_, account_type, banque)| is_letterable(account_type, *banque))
        .map(|(id, _, _)| id)
        .collect())
}

/// Raccourci booléen de [`letterable_account`] : `false` aussi pour un compte
/// introuvable dans la société.
pub async fn is_letterable_account(
    conn: &mut MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<bool, DbError> {
    Ok(letterable_account(conn, company_id, account_id)
        .await?
        .is_some_and(|(lettrable, _)| lettrable))
}

/// Rang 4 : refuse un compte non lettrable ; rend son numéro (pour l'audit).
async fn require_letterable(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    account_id: i64,
) -> Result<String, DbError> {
    let (lettrable, number) = letterable_account(tx, company_id, account_id)
        .await?
        .ok_or_else(|| {
            DbError::Invariant(format!(
                "lettrage : le compte {account_id} d'une ligne tenue est hors de la société"
            ))
        })?;
    if !lettrable {
        return Err(DbError::LetteringAccountNotLetterable);
    }
    Ok(number)
}

/// Exercice d'une ligne, tel que la règle des périodes le juge.
#[derive(Debug, Clone)]
struct FiscalYearState {
    name: String,
    open: bool,
    later_closed: bool,
}

/// R7 point 2 — mode `Manual` : lit les exercices du groupe sans verrou, les
/// verrouille un par un dans l'ordre chronologique, puis lit « un postérieur
/// est clos » sans verrou pour chaque exercice ouvert.
async fn lock_fiscal_years_of_group(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    fiscal_year_ids: &BTreeSet<i64>,
) -> Result<BTreeMap<i64, FiscalYearState>, DbError> {
    // (a) Lecture non verrouillante, triée par `start_date` (immuable).
    let sql = with_placeholders(FISCAL_YEARS_OF_GROUP_SQL, fiscal_year_ids.len());
    let mut q = sqlx::query_as::<_, (i64, NaiveDate, String)>(&sql);
    for id in fiscal_year_ids {
        q = q.bind(*id);
    }
    let ordonnes = q
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
    if ordonnes.len() != fiscal_year_ids.len() {
        // L'exercice d'une écriture tenue par l'acte 1 ne peut disparaître
        // (`fk_journal_entries_fiscal_year`, sans cascade).
        return Err(DbError::Invariant(format!(
            "lettrage : {} exercice(s) attendu(s) pour le groupe, {} lu(s)",
            fiscal_year_ids.len(),
            ordonnes.len()
        )));
    }

    // (b) Verrou de chacun, UN PAR UN, dans l'ordre chronologique — c'est la
    // boucle, et non un `ORDER BY`, qui fixe l'ordre d'acquisition.
    let mut etats = BTreeMap::new();
    for (id, _start, name) in ordonnes {
        let verrouille: Option<(i64, NaiveDate, String)> =
            sqlx::query_as(LOCK_LETTERING_FISCAL_YEAR_SQL)
                .bind(id)
                .bind(company_id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_db_error)?;
        let (_, start_date, status) = verrouille.ok_or_else(|| {
            DbError::Invariant(format!(
                "lettrage : l'exercice {id} d'une ligne tenue a disparu"
            ))
        })?;
        let open = status == "Open";
        etats.insert(
            id,
            (
                FiscalYearState {
                    name,
                    open,
                    later_closed: false,
                },
                start_date,
            ),
        );
    }

    // (c) « Un exercice postérieur est clos », SANS verrou, pour chaque
    // exercice ouvert (un exercice clos rend ses lignes closes sans autre
    // lecture).
    let mut resultat = BTreeMap::new();
    for (id, (mut etat, start_date)) in etats {
        if etat.open {
            etat.later_closed = fiscal_years::find_later_closed(tx, company_id, start_date)
                .await?
                .is_some();
        }
        resultat.insert(id, etat);
    }
    Ok(resultat)
}

/// Le nom des exercices du groupe, lu sans verrou — mode `System` (R7 point 3)
/// et lecture d'un groupe ([`find_group`]).
///
/// Un exercice manquant est un [`DbError::Invariant`], comme en mode `Manual`
/// ([`lock_fiscal_years_of_group`]) : l'exercice d'une ligne lue ne peut
/// disparaître (`fk_journal_entries_fiscal_year`, sans cascade). Le taire
/// écrirait un nom vide dans la réponse et dans l'audit (revue P1, E-2).
async fn fiscal_year_names(
    conn: &mut MySqlConnection,
    company_id: i64,
    fiscal_year_ids: &BTreeSet<i64>,
) -> Result<BTreeMap<i64, String>, DbError> {
    let sql = with_placeholders(LETTERING_FISCAL_YEAR_NAMES_SQL, fiscal_year_ids.len());
    let mut q = sqlx::query_as::<_, (i64, String)>(&sql).bind(company_id);
    for id in fiscal_year_ids {
        q = q.bind(*id);
    }
    let names: BTreeMap<i64, String> = q
        .fetch_all(conn)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .collect();
    if names.len() != fiscal_year_ids.len() {
        return Err(DbError::Invariant(format!(
            "lettrage : {} exercice(s) attendu(s) pour le groupe, {} lu(s)",
            fiscal_year_ids.len(),
            names.len()
        )));
    }
    Ok(names)
}

/// Le numéro du compte d'un groupe (réponse et audit), sans exiger qu'il soit
/// encore lettrable (C104). Compte hors de la société → [`DbError::Invariant`] :
/// le compte d'une ligne lue ne peut disparaître (FK sans cascade), et le taire
/// écrirait un numéro vide (revue P1, E-2).
async fn group_account_number(
    conn: &mut MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<String, DbError> {
    letterable_account(conn, company_id, account_id)
        .await?
        .map(|(_, number)| number)
        .ok_or_else(|| {
            DbError::Invariant(format!(
                "lettrage : le compte {account_id} d'un groupe est hors de la société"
            ))
        })
}

/// La borne du verrou de période, lue ordinairement.
async fn books_locked_through(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<Option<NaiveDate>, DbError> {
    sqlx::query_scalar("SELECT books_locked_through FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_one(conn)
        .await
        .map_err(map_db_error)
}

/// Le prédicat **par ligne** de la règle des périodes, sur l'état **trouvé**
/// de son exercice — le seul texte de la règle, partagé par le mode `Manual`
/// ([`any_line_in_open_period`]) et par [`OpenPeriodRule::line_in_open_period`]
/// (Story 15-1a2-0, D1 ; C-15-1a2-18) : jamais recopié.
///
/// ⚠️ **Borne stricte** : `books_locked_through` est le **dernier jour clos**
/// (seuil inclusif) — une ligne datée du jour de la borne est close, celle du
/// lendemain ouverte.
fn line_open_in_period(
    exercice: &FiscalYearState,
    locked_through: Option<NaiveDate>,
    entry_date: NaiveDate,
) -> bool {
    exercice.open && !exercice.later_closed && locked_through.is_none_or(|borne| entry_date > borne)
}

/// Règle des périodes (R7) : au moins une ligne « en période ouverte ».
///
/// ⚠️ Un exercice absent de `exercices` rend la ligne close (`is_some_and`) :
/// le cas ne se produit pas ici, `exercices` étant lu sous verrou sur les
/// exercices des lignes mêmes ([`lock_fiscal_years_of_group`], qui rend
/// [`DbError::Invariant`] s'il en manque un). [`OpenPeriodRule`] le traite, lui,
/// en `Invariant` (C-15-1a2-25).
fn any_line_in_open_period(
    lines: &[LineRow],
    exercices: &BTreeMap<i64, FiscalYearState>,
    locked_through: Option<NaiveDate>,
) -> bool {
    lines.iter().any(|l| {
        exercices
            .get(&l.fiscal_year_id)
            .is_some_and(|e| line_open_in_period(e, locked_through, l.entry_date))
    })
}

/// La règle des périodes **hors du mode `Manual`** : l'état des exercices
/// NOMMÉS et la borne du verrou de période, lus **SANS verrou** (Story 15-1a2-0,
/// D1 ; C-15-1a2-18). Employée par le rang 2 bis de la file des annulations
/// ([`document_group_frozen_by_periods`]), et — telle quelle — par la
/// synchronisation des pièces (15-1a2-i) et la vue des postes ouverts (15-1b).
///
/// Une ligne est **« en période ouverte »** si son exercice est `Open`, si aucun
/// exercice postérieur n'est `Closed` (`fiscal_years::find_later_closed`, lu
/// sur la **date de début** de l'exercice), et si sa date est **strictement**
/// postérieure à `companies.books_locked_through` — le prédicat par ligne est
/// celui du mode `Manual`, partagé ([`line_open_in_period`]).
///
/// ⚠️ **Lecture sans verrou — tolérance nommée** : une pose de borne, ou la
/// clôture d'un exercice, validée entre cette lecture et le `COMMIT` de
/// l'appelant n'est pas vue (la tolérance qu'a déjà une écriture créée pendant
/// la pose du verrou ; D4 de la Story 15-1a2-0). Aucun verrou n'est pris ici.
#[derive(Debug, Clone)]
pub struct OpenPeriodRule {
    exercices: BTreeMap<i64, FiscalYearState>,
    locked_through: Option<NaiveDate>,
}

/// Lit, **sans verrou**, le statut et la date de début des exercices NOMMÉS
/// (pour `fiscal_years::find_later_closed`, appelé sur chaque exercice
/// ouvert), puis la borne du verrou de période. Un identifiant d'exercice
/// introuvable dans la société n'est pas une erreur ici : il reste **inconnu**
/// de la règle, et [`OpenPeriodRule::line_in_open_period`] le refuse.
pub async fn open_period_rule(
    conn: &mut MySqlConnection,
    company_id: i64,
    fiscal_year_ids: &[i64],
) -> Result<OpenPeriodRule, DbError> {
    let ids: BTreeSet<i64> = fiscal_year_ids.iter().copied().collect();
    let mut exercices = BTreeMap::new();
    if !ids.is_empty() {
        let sql = with_placeholders(OPEN_PERIOD_RULE_FISCAL_YEARS_SQL, ids.len());
        let mut q = sqlx::query_as::<_, (i64, NaiveDate, String, String)>(&sql);
        for id in &ids {
            q = q.bind(*id);
        }
        let lus = q
            .bind(company_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(map_db_error)?;
        for (id, start_date, status, name) in lus {
            let open = status == "Open";
            let later_closed = open
                && fiscal_years::find_later_closed(&mut *conn, company_id, start_date)
                    .await?
                    .is_some();
            exercices.insert(
                id,
                FiscalYearState {
                    name,
                    open,
                    later_closed,
                },
            );
        }
    }
    let locked_through = books_locked_through(conn, company_id).await?;
    Ok(OpenPeriodRule {
        exercices,
        locked_through,
    })
}

impl OpenPeriodRule {
    /// La ligne `(fiscal_year_id, entry_date)` est-elle « en période ouverte » ?
    ///
    /// Exercice inconnu de la règle (non nommé à [`open_period_rule`], ou d'une
    /// autre société) → [`DbError::Invariant`] : l'appelant nomme les exercices
    /// des lignes qu'il interroge ; un exercice absent est un défaut de
    /// l'appelant, **jamais une réponse** (C-15-1a2-25). Un `false` serait muet
    /// pour la vue des postes ouverts, qui filtrerait la ligne sans le dire.
    pub fn line_in_open_period(
        &self,
        fiscal_year_id: i64,
        entry_date: NaiveDate,
    ) -> Result<bool, DbError> {
        let exercice = self.exercices.get(&fiscal_year_id).ok_or_else(|| {
            DbError::Invariant(format!(
                "règle des périodes : l'exercice {fiscal_year_id} n'a pas été nommé à \
                 open_period_rule"
            ))
        })?;
        Ok(line_open_in_period(
            exercice,
            self.locked_through,
            entry_date,
        ))
    }
}

/// [`open_period_rule`] sur les exercices des lignes, puis « au moins une ligne
/// en période ouverte ». `lines` : couples `(fiscal_year_id, entry_date)` —
/// l'identifiant d'**exercice**, pas d'écriture. Aucune ligne → `false`.
pub async fn lines_in_open_period(
    conn: &mut MySqlConnection,
    company_id: i64,
    lines: &[(i64, NaiveDate)],
) -> Result<bool, DbError> {
    let ids: Vec<i64> = lines.iter().map(|(fy, _)| *fy).collect();
    let regle = open_period_rule(conn, company_id, &ids).await?;
    for (fiscal_year_id, entry_date) in lines {
        if regle.line_in_open_period(*fiscal_year_id, *entry_date)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Le rang **2 bis** de la file commune des annulations (Story 15-1a2-0, D2) :
/// la clé du premier (plus petite clé) groupe d'origine `document` qui contient
/// une ligne de l'écriture `entry_id` et dont **AUCUNE** ligne n'est en période
/// ouverte ([`OpenPeriodRule`]) ; `None` sinon.
///
/// Un tel groupe est **figé** : l'annulation qui le dissoudrait est refusée
/// ([`crate::errors::SettlementCancelBlocker::DocumentLetteringInClosedPeriods`]).
/// Il redevient dissoluble dès qu'**une** de ses lignes repasse en période
/// ouverte — la plus récente se libère la première.
///
/// Lecture **sans verrou** des lignes du groupe, des exercices et de la borne
/// (tolérance de [`OpenPeriodRule`]). Les lignes sont lues par
/// `idx_jel_lettering` ; le filtre de société porte sur l'en-tête.
///
/// ⚠️ **Dormant avant la 15-1a2-i** : aucun chemin de production ne pose
/// encore d'origine `document` — seuls les tests en posent, en SQL brut.
pub async fn document_group_frozen_by_periods(
    conn: &mut MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Option<i64>, DbError> {
    let cles: Vec<i64> = sqlx::query_scalar(DOCUMENT_KEYS_OF_ENTRY_SQL)
        .bind(entry_id)
        .bind(company_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    for key in cles {
        let lignes: Vec<(i64, NaiveDate)> = sqlx::query_as(GROUP_LINE_PERIODS_SQL)
            .bind(key)
            .bind(company_id)
            .fetch_all(&mut *conn)
            .await
            .map_err(map_db_error)?;
        // Un groupe dissous entre les deux lectures (deux instantanés distincts
        // hors transaction) n'a plus de lignes : il n'est pas figé — `false`
        // pour une liste vide ne veut pas dire « tout est clos » (revue P1, E-1 =
        // A-2). ⚠️ Branche non montable en isolation : les clés sont lues SUR
        // les lignes, seule une dissolution concurrente entre les deux lectures
        // y mène — dite ici plutôt que forcée par un entrelacement.
        if lignes.is_empty() {
            continue;
        }
        if !lines_in_open_period(&mut *conn, company_id, &lignes).await? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}

/// R5 — la première ligne (dans l'ordre des lignes) dont l'écriture appartient
/// à une **pièce** : un propriétaire dont le type
/// [`DocumentKind::blocks_manual_lettering`] — la liste R5 écrite une fois, sur
/// [`DocumentKind`], jamais une seconde ici —, rendu avec le motif
/// [`DocumentKind::reversal_blocker`], l'identifiant et le numéro de la pièce.
///
/// **Un** appel à [`journal_entries::document_owners`] pour tout le groupe
/// (Story 15-1b-0, D3). Une écriture absente ou d'une autre société n'a pas de
/// propriétaire — inatteignable : les lignes viennent de l'acte 1, borné par
/// société.
async fn first_document_owner(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    lines: &[LineRow],
) -> Result<Option<DbError>, DbError> {
    let entries: Vec<i64> = lines.iter().map(|l| l.entry_id).collect();
    let owners = journal_entries::document_owners(tx, company_id, &entries).await?;
    Ok(lines.iter().find_map(|line| {
        owners
            .get(&line.entry_id)?
            .iter()
            .find(|o| o.kind.blocks_manual_lettering())
            .map(|o| DbError::LetteringLineOwnedByDocument {
                blocker: o.kind.reversal_blocker(),
                document_id: Some(o.id),
                document_label: o.number.clone(),
            })
    }))
}

/// Vérifie qu'une ligne du groupe porte l'exercice que l'appelant tient (mode
/// `System`, R7 point 3) — sans requête, sur les lignes de l'acte 1.
fn check_held_fiscal_year(lines: &[LineRow], held_open_fiscal_year_id: i64) -> Result<(), DbError> {
    if lines
        .iter()
        .any(|l| l.fiscal_year_id == held_open_fiscal_year_id)
    {
        Ok(())
    } else {
        Err(DbError::Invariant(format!(
            "lettrage système : l'exercice tenu {held_open_fiscal_year_id} ne couvre aucune ligne \
             du groupe"
        )))
    }
}

/// Construit le groupe décrit (réponse et audit).
///
/// Un exercice de ligne absent de `names` → [`DbError::Invariant`]. Chaque
/// lecture actuelle des noms (`lock_fiscal_years_of_group`,
/// `fiscal_year_names`) le refuse déjà (revue P1, E-2) ; la garde est
/// répétée ici pour qu'un futur appelant ne retrouve pas en silence le nom
/// vide qu'E-2 a proscrit (revue de code P2, B2-2).
fn build_group(
    key: i64,
    origin: Origin,
    account_id: i64,
    account_number: String,
    lines: &[LineRow],
    names: &BTreeMap<i64, String>,
) -> Result<LetteringGroup, DbError> {
    let lines = lines
        .iter()
        .map(|l| {
            let fiscal_year_name = names.get(&l.fiscal_year_id).cloned().ok_or_else(|| {
                DbError::Invariant(format!(
                    "lettrage : nom de l'exercice {} de la ligne {} non lu",
                    l.fiscal_year_id, l.id
                ))
            })?;
            Ok(LetteringLine {
                id: l.id,
                entry_id: l.entry_id,
                entry_number: l.entry_number,
                fiscal_year_id: l.fiscal_year_id,
                fiscal_year_name,
                date: l.entry_date,
                debit: l.debit,
                credit: l.credit,
            })
        })
        .collect::<Result<Vec<_>, DbError>>()?;
    Ok(LetteringGroup {
        key,
        code: code_of(key),
        origin,
        account_id,
        account_number,
        lines,
    })
}

/// La pièce dont une synchronisation pose ou défait le groupe `document` — pour
/// l'audit (Story 15-1a2-i, P3 ; C-15-1a2-22). Défini **ici, une fois** : la
/// 15-1a2-ii (factures fournisseurs) le consomme tel quel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRef {
    /// [`DocumentKind::Invoice`] (facture client) ou
    /// [`DocumentKind::SupplierInvoice`] (facture fournisseur, Story
    /// 15-1a2-ii), sérialisé par [`DocumentKind::as_str`] — la valeur même de
    /// `document.type` de la vue des postes ouverts (15-1b). Typé (Story
    /// 15-1b-0, D1 bis ; C-15-1b-0-3) : aucun littéral ne peut plus y entrer.
    pub document_type: DocumentKind,
    pub id: i64,
    /// Le numéro de la pièce ; `None` quand elle n'en a pas (facture
    /// fournisseur sans numéro, 15-1a2-ii) — la clé `documentNumber` est alors
    /// présente et `null`, jamais omise.
    pub number: Option<String>,
}

/// `details` de l'audit `lettering.created` / `lettering.removed` (AC10).
///
/// `document` (Story 15-1a2-i, AC10) : `Some` pour la synchronisation d'une
/// pièce — les trois clés `documentType`, `documentId`, `documentNumber`
/// (présente et `null` sans numéro) s'ajoutent ; `None` pour les routes et la
/// contre-passation, dont les `details` restent **inchangés**.
fn audit_details(group: &LetteringGroup, document: Option<&DocumentRef>) -> serde_json::Value {
    let mut details = serde_json::json!({
        "code": group.code,
        "origin": group.origin.as_str(),
        "accountId": group.account_id,
        "accountNumber": group.account_number,
        "lines": group.lines.iter().map(|l| serde_json::json!({
            "id": l.id,
            "entryId": l.entry_id,
            "entryNumber": l.entry_number,
            "fiscalYearId": l.fiscal_year_id,
            "fiscalYearName": l.fiscal_year_name,
            "debit": l.debit.to_string(),
            "credit": l.credit.to_string(),
        })).collect::<Vec<_>>(),
    });
    if let (Some(doc), Some(objet)) = (document, details.as_object_mut()) {
        objet.insert("documentType".into(), doc.document_type.as_str().into());
        objet.insert("documentId".into(), doc.id.into());
        objet.insert(
            "documentNumber".into(),
            doc.number
                .clone()
                .map_or(serde_json::Value::Null, serde_json::Value::String),
        );
    }
    details
}

/// **Pose la marque** d'un groupe de lettrage — primitive publique (R3), dont le
/// corps [`create_group_inner`] est la seule fonction qui l'écrit.
/// (R3). Dans la transaction de l'appelant, sans BEGIN ni COMMIT ; en cas
/// d'erreur, l'appelant laisse tomber la transaction (rollback).
///
/// Refus, **dans cet ordre** (AC3 — une requête qui cumule deux causes rend la
/// première) :
///
/// 1. moins de deux identifiants, ou un doublon → [`DbError::LetteringTooFewLines`] ;
/// 2. une ligne introuvable **ou d'une autre société** → [`DbError::NotFound`] ;
/// 3. comptes différents → [`DbError::LetteringAccountsDiffer`] ;
/// 4. compte non lettrable (R4) → [`DbError::LetteringAccountNotLetterable`] ;
///    puis **rang 4 bis** : *(mode `Manual`)* aucune ligne en période ouverte
///    (R7) → [`DbError::LetteringAllLinesInClosedPeriods`] ; *(mode `System`)*
///    l'exercice tenu ne couvre aucune ligne → [`DbError::Invariant`] ;
/// 5. *(mode `Manual`)* une ligne appartient à une pièce (R5) →
///    [`DbError::LetteringLineOwnedByDocument`] ;
/// 6. une ligne déjà lettrée → [`DbError::LetteringLineAlreadyLettered`] ;
/// 7. somme non nulle → [`DbError::LetteringUnbalanced`].
///
/// Sinon pose `lettering_key = MIN(id)` et `origin` sur **toutes** les lignes
/// en un `UPDATE … WHERE id IN (…) AND lettering_key IS NULL`, vérifie le
/// nombre de lignes trouvées (R7 point 4) et écrit l'audit `lettering.created`
/// (acteur par `for_actor`).
///
/// La séquence des verrous est au doc-comment du module. Le corps est
/// [`create_group_inner`], partagé avec la synchronisation des pièces : les
/// `details` de l'audit ne portent ici aucune clé `document*`.
pub async fn create_group_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    line_ids: &[i64],
    origin: Origin,
    mode: Mode,
    actor: Actor,
) -> Result<LetteringGroup, DbError> {
    create_group_inner(tx, company_id, line_ids, origin, mode, actor, None).await
}

/// Le corps de [`create_group_in_tx`] — **la seule fonction qui écrit la
/// marque** (R3) — avec, en plus, la pièce pour l'audit (`Some` depuis la
/// synchronisation, Story 15-1a2-i ; `None` depuis la primitive publique).
async fn create_group_inner(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    line_ids: &[i64],
    origin: Origin,
    mode: Mode,
    actor: Actor,
    document: Option<&DocumentRef>,
) -> Result<LetteringGroup, DbError> {
    // Rang 1 — forme, sans base.
    core_lettering::check_line_ids(line_ids).map_err(refusal)?;

    // Acte 1 — lecture VERROUILLANTE des lignes et de leurs en-têtes.
    let sql = with_placeholders(LOCK_LINES_BY_ID_SQL, line_ids.len());
    let mut q = sqlx::query_as::<_, LineRow>(&sql);
    for id in line_ids {
        q = q.bind(*id);
    }
    let lines = q
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
    // Rang 2 — introuvable ou d'une autre société : indiscernables (AC11).
    if lines.len() != line_ids.len() {
        return Err(DbError::NotFound);
    }

    // Rang 3 — même compte (pur).
    let comptes: Vec<i64> = lines.iter().map(|l| l.account_id).collect();
    core_lettering::check_same_account(&comptes).map_err(refusal)?;
    let account_id = comptes[0];

    let exercices_du_groupe: BTreeSet<i64> = lines.iter().map(|l| l.fiscal_year_id).collect();

    // R7 point 2 (Manual) — exercices du groupe verrouillés dans l'ordre ;
    // puis rang 4 (lettrabilité, exigée à la création seulement — C104) et
    // rang 4 bis (règle des périodes en `Manual`, exercice tenu en `System`).
    let (account_number, names) = match mode {
        Mode::Manual => {
            let exercices =
                lock_fiscal_years_of_group(tx, company_id, &exercices_du_groupe).await?;
            let account_number = require_letterable(tx, company_id, account_id).await?;
            let borne = books_locked_through(tx, company_id).await?;
            if !any_line_in_open_period(&lines, &exercices, borne) {
                return Err(DbError::LetteringAllLinesInClosedPeriods);
            }
            let names = exercices.into_iter().map(|(id, e)| (id, e.name)).collect();
            (account_number, names)
        }
        Mode::System {
            held_open_fiscal_year_id,
        } => {
            let account_number = require_letterable(tx, company_id, account_id).await?;
            check_held_fiscal_year(&lines, held_open_fiscal_year_id)?;
            let names = fiscal_year_names(tx, company_id, &exercices_du_groupe).await?;
            (account_number, names)
        }
    };

    // Rang 5 — (Manual) une ligne d'une pièce ne se lettre pas à la main (R5).
    if mode == Mode::Manual
        && let Some(refus) = first_document_owner(tx, company_id, &lines).await?
    {
        return Err(refus);
    }

    // Rang 6 — déjà lettrée (pur).
    let cles: Vec<Option<i64>> = lines.iter().map(|l| l.lettering_key).collect();
    core_lettering::check_none_lettered(&cles).map_err(refusal)?;

    // Rang 7 — somme nulle (pur).
    let montants: Vec<(Decimal, Decimal)> = lines.iter().map(|l| (l.debit, l.credit)).collect();
    core_lettering::check_balanced(&montants).map_err(refusal)?;

    // La marque : clé = plus petit `id` du groupe (les lignes sont triées).
    let key = lines[0].id;
    let sql = with_placeholders(
        "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = ? \
         WHERE id IN ({ids}) AND lettering_key IS NULL",
        lines.len(),
    );
    let mut q = sqlx::query(&sql).bind(key).bind(origin.as_str());
    for l in &lines {
        q = q.bind(l.id);
    }
    let resultat = q.execute(&mut **tx).await.map_err(map_db_error)?;
    // R7 point 4 — lignes TROUVÉES (sqlx pose `CLIENT_FOUND_ROWS`) contre
    // l'attendu ; ⛔ cet appel est exigé dans chaque primitive par le test
    // lexical `letterings_lexical.rs` (seule garde contre son retrait).
    core_lettering::check_rows_affected(lines.len() as u64, resultat.rows_affected())
        .map_err(refusal)?;

    let group = build_group(key, origin, account_id, account_number, &lines, &names)?;
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            actor.user_id,
            actor.api_key_id,
            "lettering.created",
            "lettering",
            key,
            Some(audit_details(&group, document)),
        ),
    )
    .await?;
    Ok(group)
}

/// **Retire la marque** d'un groupe — primitive publique (R3), dont le corps
/// [`dissolve_group_inner`] est la seule fonction qui l'efface.
/// Dans la transaction de l'appelant, sans BEGIN ni COMMIT.
///
/// Groupe introuvable ou d'une autre société → [`DbError::NotFound`]. En mode
/// `Manual` (la route), refus dans cet ordre (AC5) :
///
/// 1. groupe d'origine `document` → [`DbError::LetteringIsDocument`] ;
/// 2. groupe `reversal` dont une ligne appartient à une pièce →
///    [`DbError::LetteringLineOwnedByDocument`] (C106 : sinon la paire
///    resterait ouverte pour toujours, R5 interdisant de la relettrer) ;
/// 3. aucune ligne en période ouverte → [`DbError::LetteringAllLinesInClosedPeriods`].
///
/// ⛔ **La dissolution n'exige jamais la lettrabilité** (C104) : un groupe dont
/// le compte a été retypé ou rattaché à un compte bancaire depuis se dissout.
///
/// En mode `System { held_open_fiscal_year_id }` : ni refus 1 ni 2, aucun
/// verrou d'exercice, et l'exercice tenu doit couvrir une ligne
/// ([`DbError::Invariant`] sinon).
///
/// Remet `lettering_key` et `lettering_origin` à `NULL` sur toutes les lignes
/// du groupe (nombre de lignes trouvées vérifié) et écrit l'audit
/// `lettering.removed`. Rend le groupe tel qu'il était.
///
/// Le corps est [`dissolve_group_inner`], partagé avec la synchronisation des
/// pièces : les `details` de l'audit ne portent ici aucune clé `document*`.
pub async fn dissolve_group_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    key: i64,
    mode: Mode,
    actor: Actor,
) -> Result<LetteringGroup, DbError> {
    dissolve_group_inner(tx, company_id, key, mode, actor, None).await
}

/// Le corps de [`dissolve_group_in_tx`] — **la seule fonction qui retire la
/// marque** (R3) — avec, en plus, la pièce pour l'audit.
async fn dissolve_group_inner(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    key: i64,
    mode: Mode,
    actor: Actor,
    document: Option<&DocumentRef>,
) -> Result<LetteringGroup, DbError> {
    // Acte 1 — lecture VERROUILLANTE par `idx_jel_lettering`.
    let lines = sqlx::query_as::<_, LineRow>(LOCK_LINES_BY_KEY_SQL)
        .bind(key)
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
    if lines.is_empty() {
        return Err(DbError::NotFound);
    }
    let origin = lines[0]
        .lettering_origin
        .as_deref()
        .and_then(Origin::parse)
        .ok_or_else(|| {
            DbError::Invariant(format!("lettrage : origine illisible pour le groupe {key}"))
        })?;
    let account_id = lines[0].account_id;
    let exercices_du_groupe: BTreeSet<i64> = lines.iter().map(|l| l.fiscal_year_id).collect();

    let names = match mode {
        Mode::Manual => {
            let exercices =
                lock_fiscal_years_of_group(tx, company_id, &exercices_du_groupe).await?;
            // Refus 1 — le lettrage d'une pièce suit la pièce.
            if origin == Origin::Document {
                return Err(DbError::LetteringIsDocument);
            }
            // Refus 2 — une paire de contre-passation dont une ligne est celle
            // d'une pièce resterait ouverte pour toujours.
            if origin == Origin::Reversal
                && let Some(refus) = first_document_owner(tx, company_id, &lines).await?
            {
                return Err(refus);
            }
            // Refus 3 — règle des périodes.
            let borne = books_locked_through(tx, company_id).await?;
            if !any_line_in_open_period(&lines, &exercices, borne) {
                return Err(DbError::LetteringAllLinesInClosedPeriods);
            }
            exercices.into_iter().map(|(id, e)| (id, e.name)).collect()
        }
        Mode::System {
            held_open_fiscal_year_id,
        } => {
            check_held_fiscal_year(&lines, held_open_fiscal_year_id)?;
            fiscal_year_names(tx, company_id, &exercices_du_groupe).await?
        }
    };

    let account_number = group_account_number(tx, company_id, account_id).await?;

    let resultat = sqlx::query(
        "UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL \
         WHERE lettering_key = ?",
    )
    .bind(key)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    // R7 point 4 — lignes TROUVÉES (sqlx pose `CLIENT_FOUND_ROWS`) contre
    // l'attendu ; ⛔ cet appel est exigé dans chaque primitive par le test
    // lexical `letterings_lexical.rs` (seule garde contre son retrait).
    core_lettering::check_rows_affected(lines.len() as u64, resultat.rows_affected())
        .map_err(refusal)?;

    let group = build_group(key, origin, account_id, account_number, &lines, &names)?;
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            actor.user_id,
            actor.api_key_id,
            "lettering.removed",
            "lettering",
            key,
            Some(audit_details(&group, document)),
        ),
    )
    .await?;
    Ok(group)
}

// ---------------------------------------------------------------------------
// Story 15-1a2-i (#518) — le lettrage des pièces clientes
// ---------------------------------------------------------------------------

/// Issue d'une synchronisation de pièce (Story 15-1a2-i, P3).
///
/// ⛔ Aucune de ces issues n'est une erreur : **un règlement, un solde, un avoir,
/// un rapprochement ou un paiement fournisseur n'est jamais refusé à cause du
/// lettrage**. Seul un état
/// que les gestes ne produisent pas sort en [`DbError::Invariant`] — et la
/// tolérance nommée à [`sync_invoice_in_tx`] (un exercice créé pendant le geste).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOutcome {
    /// Rien à faire : le groupe existant est exactement la cible, ou il n'y a ni
    /// groupe ni cible (facture partiellement réglée, `|C(I)| = 1`, …).
    Unchanged,
    /// Un groupe `document` a été posé.
    Created { key: i64 },
    /// Un groupe `document` a été défait.
    Dissolved { key: i64 },
    /// Un groupe `document` a été défait puis un autre posé (branche
    /// défensive : aucun geste n'y mène, P3 étape 5).
    Recreated { dissolved: i64, created: i64 },
    /// Rien n'est écrit : la règle des périodes interdit de LETTRER (P7 point
    /// 1) — rendu par la synchronisation seule, jamais par la dissolution …
    AbstainedClosedPeriods,
    /// … ou le compte de créance n'est pas lettrable (P3 point 3) — étape
    /// **terminale** : rien n'est écrit, quel que soit le groupe existant.
    AccountNotLetterable,
}

/// La facture, **verrouillée** (l'appelant la tient déjà : la relecture
/// `FOR UPDATE` ne prend aucun verrou neuf et lit la ligne courante).
const SYNC_INVOICE_SQL: &str = "SELECT status, journal_entry_id, invoice_number FROM invoices \
     WHERE id = ? AND company_id = ? FOR UPDATE";

/// Les écritures des règlements **en vigueur** de la facture (lignes de
/// `invoice_settlements`, soldes compris), lecture **verrouillante**, par
/// l'index (société, facture), forcé (C-15-1a2-i-2).
const SYNC_SETTLEMENT_ENTRIES_SQL: &str = "SELECT journal_entry_id FROM invoice_settlements \
     FORCE INDEX (idx_invoice_settlements_company_invoice) WHERE invoice_id = ? AND company_id = ? ORDER BY id FOR UPDATE";

/// L'écriture de l'avoir **émis** de la facture, lecture **verrouillante** —
/// par `uq_credit_notes_invoice`, **forcé** : `idx_credit_notes_company_status`
/// verrouillerait tous les avoirs émis de la société (C-15-1a2-i-2).
const SYNC_CREDIT_NOTE_ENTRIES_SQL: &str = "SELECT journal_entry_id FROM credit_notes \
     FORCE INDEX (uq_credit_notes_invoice) WHERE invoice_id = ? AND company_id = ? AND status = 'issued' \
     AND journal_entry_id IS NOT NULL ORDER BY id FOR UPDATE";

/// Les lignes sur le compte de créance `A` des écritures de la pièce —
/// lignes **et** en-têtes, lecture **verrouillante** (sous `REPEATABLE READ`,
/// une lecture ordinaire lirait l'instantané — finding F-3). ⛔ Parcours
/// **forcé** par `idx_jel_entry` (C-15-1a2-i-2) : sur une table peu peuplée,
/// l'optimiseur choisit `idx_jel_account` (relevé `EXPLAIN` du T0), et le
/// `FOR UPDATE` verrouillerait alors toutes les lignes du compte de créance de
/// la société ; l'en-tête, de même, est pris par sa clé primaire, dans cet
/// ordre (`STRAIGHT_JOIN`) — l'optimiseur proposait l'index de société et de
/// date, qui verrouillerait les en-têtes de toute la société. `{ids}` : autant
/// de `?` que d'écritures.
const SYNC_DOCUMENT_LINES_SQL: &str = "SELECT jel.id, jel.entry_id, jel.account_id, jel.debit, \
     jel.credit, jel.lettering_key, jel.lettering_origin, je.fiscal_year_id, je.entry_date, \
     je.entry_number \
     FROM journal_entry_lines jel FORCE INDEX (idx_jel_entry) \
     STRAIGHT_JOIN journal_entries je FORCE INDEX (PRIMARY) ON je.id = jel.entry_id \
     WHERE jel.entry_id IN ({ids}) AND jel.account_id = ? AND je.company_id = ? \
     ORDER BY jel.id FOR UPDATE";

/// Les identifiants des lignes d'un groupe, **sans verrou** — la comparaison de
/// l'étape 4. ⚠️ Lecture d'instantané : tenir des lignes ne rafraîchit pas
/// la vue ; elle reste juste parce que le groupe `k` d'une facture ne change que
/// sous le verrou de la facture, que tout geste prend avant la synchronisation
/// (revue P2, B2 sur E-2).
const GROUP_LINE_IDS_SQL: &str = "SELECT jel.id \
     FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
     WHERE jel.lettering_key = ? AND je.company_id = ? ORDER BY jel.id";

/// Ce que l'étape 1 découvre d'une pièce — facture client ou fournisseur : `C`,
/// son compte (`A` créance, `B` dette), la pièce pour l'audit. Le **seul** point
/// où les deux familles diffèrent (Story 15-1a2-ii, P3 part ii) : la suite de
/// l'algorithme ([`sync_document_in_tx`], [`dissolve_document_in_tx`]) est
/// commune, jamais recopiée.
struct PieceDocument {
    account_id: i64,
    /// `C` : l'ancre et les lignes sur le compte de la pièce des écritures qui
    /// la soldent (règlements, avoir) — triées par `id`.
    lines: Vec<LineRow>,
    document: DocumentRef,
}

/// P3 étape 1 — la découverte, par des lectures **verrouillantes** : la
/// facture, ses règlements en vigueur, son avoir émis, puis les lignes sur `A`
/// de leurs écritures. `None` pour un brouillon (aucune écriture de vente,
/// rien à lettrer).
///
/// L'**ancre** est la ligne de vente sur `A` : la première ligne au débit de
/// l'écriture de vente ([`super::invoice_settlements::sale_receivable_account`]).
/// Une autre ligne de l'écriture de vente sur `A` (un arrondi négatif qui
/// débiterait `A`) n'est pas dans `C(I)`.
async fn discover_invoice_document(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    invoice_id: i64,
) -> Result<Option<PieceDocument>, DbError> {
    let (status, sale_entry_id, number): (String, Option<i64>, Option<String>) =
        sqlx::query_as(SYNC_INVOICE_SQL)
            .bind(invoice_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?
            .ok_or(DbError::NotFound)?;
    let Some(sale_entry_id) = sale_entry_id else {
        if status == "draft" {
            return Ok(None);
        }
        return Err(DbError::Invariant(format!(
            "lettrage de pièce : la facture {invoice_id} ({status}) n'a pas d'écriture de vente"
        )));
    };
    // Comme `settle_invoice` et `write_off_invoice` sur le même cas
    // (validation P3, L-9 d).
    let account_id =
        super::invoice_settlements::sale_receivable_account(tx, company_id, sale_entry_id)
            .await?
            .ok_or_else(|| DbError::Invariant("écriture de vente sans ligne de débit".into()))?;

    let mut entries = vec![sale_entry_id];
    let reglements: Vec<i64> = sqlx::query_scalar(SYNC_SETTLEMENT_ENTRIES_SQL)
        .bind(invoice_id)
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
    entries.extend(reglements);
    let avoirs: Vec<i64> = sqlx::query_scalar(SYNC_CREDIT_NOTE_ENTRIES_SQL)
        .bind(invoice_id)
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
    entries.extend(avoirs);

    let sql = with_placeholders(SYNC_DOCUMENT_LINES_SQL, entries.len());
    let mut q = sqlx::query_as::<_, LineRow>(&sql);
    for e in &entries {
        q = q.bind(*e);
    }
    let lues = q
        .bind(account_id)
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // L'ancre : la PREMIÈRE ligne au débit de l'écriture de vente (lignes triées
    // par `id`) — celle que lit `sale_receivable_account`.
    let ancre = lues
        .iter()
        .find(|l| l.entry_id == sale_entry_id && l.debit > Decimal::ZERO)
        .map(|l| l.id)
        .ok_or_else(|| {
            DbError::Invariant(format!(
                "lettrage de pièce : l'ancre de la facture {invoice_id} est introuvable"
            ))
        })?;
    let lines = lues
        .into_iter()
        .filter(|l| l.entry_id != sale_entry_id || l.id == ancre)
        .collect();
    Ok(Some(PieceDocument {
        account_id,
        lines,
        document: DocumentRef {
            document_type: DocumentKind::Invoice,
            id: invoice_id,
            number,
        },
    }))
}

/// P3 étape 2 — le groupe existant `E` : les clés **distinctes et non nulles**
/// des lignes de `C(I)`. Une ligne d'origine `manual` ou `reversal`, ou deux
/// clés distinctes → [`DbError::Invariant`] : un état que les gestes ne
/// produisent pas, **jamais un écrasement**.
fn existing_document_group(lines: &[LineRow]) -> Result<Option<i64>, DbError> {
    let mut cles = BTreeSet::new();
    for l in lines {
        match (l.lettering_key, l.lettering_origin.as_deref()) {
            (None, _) => {}
            (Some(k), Some("document")) => {
                cles.insert(k);
            }
            (Some(k), autre) => {
                return Err(DbError::Invariant(format!(
                    "lettrage de pièce : la ligne {} porte le groupe {k} d'origine {autre:?} — \
                     une ligne de pièce n'est lettrée que `document`",
                    l.id
                )));
            }
        }
    }
    if cles.len() > 1 {
        return Err(DbError::Invariant(format!(
            "lettrage de pièce : les lignes d'une même pièce portent plusieurs groupes {cles:?}"
        )));
    }
    Ok(cles.into_iter().next())
}

/// `(exercice, date)` des lignes — l'argument de [`lines_in_open_period`].
fn line_periods(lines: &[LineRow]) -> Vec<(i64, NaiveDate)> {
    lines
        .iter()
        .map(|l| (l.fiscal_year_id, l.entry_date))
        .collect()
}

/// **Synchronise le lettrage `document` d'une facture client** avec ses pièces
/// (Story 15-1a2-i, P1, P3) — idempotente, dans la transaction de l'appelant.
///
/// Le groupe d'une facture `I` (validée, ou annulée par avoir) est `C(I)` :
/// l'ancre (sa ligne de vente sur la créance `A`, lue sur l'écriture de vente),
/// plus les lignes sur `A` des écritures de **tous** ses règlements en vigueur
/// (soldes compris) et de son avoir émis. Il existe **si et seulement si**
/// `|C(I)| ≥ 2`, `Σ(débit − crédit) = 0` sur `C(I)`, `A` est lettrable, et au
/// moins une ligne de `C(I)` est en période ouverte ([`OpenPeriodRule`]).
/// ⚠️ Le critère est la somme **au grand livre**, pas le reste dû dérivé.
///
/// **Étapes et précédence des issues** :
///
/// 1. découverte **verrouillante** de `C(I)` (facture, règlements, avoir, lignes
///    et en-têtes `FOR UPDATE`) ; écriture de vente sans ligne au débit →
///    [`DbError::Invariant`] ;
/// 2. le groupe existant `E` — ligne `manual`/`reversal` ou deux clés →
///    [`DbError::Invariant`] ;
/// 3. ⛔ compte `A` **non lettrable** → [`SyncOutcome::AccountNotLetterable`],
///    **rien n'est écrit, quel que soit `E`** — étape **terminale** (C-15-1a2-29) :
///    un groupe posé avant que `A` ne devienne non lettrable **survit** (C104),
///    seule [`dissolve_invoice_document_group_in_tx`] le défait. Elle l'emporte
///    donc sur [`SyncOutcome::AbstainedClosedPeriods`] ;
/// 4. `E = {k}` et les lignes de `k` sont exactement la cible → `Unchanged` ;
/// 5. `E = {k}` sinon — **défensif, inatteignable par un geste** : aucune ligne
///    du groupe `k` en période ouverte → `AbstainedClosedPeriods`, rien d'écrit ;
///    sinon dissolution puis, s'il y a une cible, création. ⚠️ La dissolution est
///    en mode `System` sur l'exercice tenu : si aucune ligne de `k` n'y est,
///    elle rend [`DbError::Invariant`] (revue P2, B2-3). Le rattrapage de la
///    15-1a2-ii ne mène pas ici : il ne pose de groupe que sur des lignes
///    toutes libres, le groupe qu'aurait posé le geste — la synchronisation
///    rejouée après lui rend `Unchanged` (test d'accord, AC6 e) ;
/// 6. `E = ∅` : cible → création ; sinon `Unchanged`, ou
///    `AbstainedClosedPeriods` si la règle des périodes est la seule raison.
///
/// `held_open_fiscal_year_id` est l'exercice **ouvert** que l'appelant tient
/// `FOR UPDATE` : les écritures passent en mode `System`, qui n'évalue pas la
/// règle des périodes (elle l'est ici) et exige que cet exercice couvre une
/// ligne du groupe ([`DbError::Invariant`] sinon). `actor` est l'auteur du geste.
///
/// **Verrous** : après ceux du geste (la facture, l'écriture, son exercice),
/// la découverte prend `invoice_settlements` et `credit_notes` de la facture,
/// puis les lignes et en-têtes de `C(I)` ; la primitive les reprend ensuite.
/// Les cycles résiduels sont nommés au module (§ « Interblocages résiduels ») ;
/// les routes appelantes sont rejouées.
///
/// ⚠️ **Tolérance nommée** (revue P2, E2-1) : la règle des périodes et les noms
/// d'exercices sont lus **sans verrou**, sur l'instantané de la transaction. Un
/// exercice **créé** par une autre session après la première lecture ordinaire
/// du geste et avant son `find_open_covering_date … FOR UPDATE` en est absent :
/// la synchronisation rend alors [`DbError::Invariant`] et le geste échoue (500,
/// ou `INTERNAL_ERROR` per-proposal au rapprochement) — à refaire. Fenêtre de
/// la création d'exercice seule, que le lettrage `reversal` de la
/// contre-passation (15-1a-ii) partage déjà ; aucun verrou n'est pris sur les
/// exercices hors de l'ordre de la clôture.
pub async fn sync_invoice_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    invoice_id: i64,
    held_open_fiscal_year_id: i64,
    actor: Actor,
) -> Result<SyncOutcome, DbError> {
    // (1) Découverte — la seule étape propre aux factures clientes.
    let piece = discover_invoice_document(tx, company_id, invoice_id).await?;
    sync_document_in_tx(tx, company_id, piece, held_open_fiscal_year_id, actor).await
}

/// Étapes 2 à 6 de la synchronisation, **communes** aux factures clientes et
/// fournisseurs (Story 15-1a2-ii, P3 part ii) : la découverte (étape 1) est
/// faite par l'appelant et passée en paramètre — `None` → `Unchanged` (brouillon
/// client, facture fournisseur annulée). ⛔ Pas de seconde copie de
/// l'algorithme : le contrat, les étapes et leur précédence sont ceux de
/// [`sync_invoice_in_tx`].
async fn sync_document_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    piece: Option<PieceDocument>,
    held_open_fiscal_year_id: i64,
    actor: Actor,
) -> Result<SyncOutcome, DbError> {
    let Some(piece) = piece else {
        return Ok(SyncOutcome::Unchanged);
    };
    // (2) Le groupe existant.
    let existant = existing_document_group(&piece.lines)?;
    // (3) ⛔ Terminale : compte non lettrable, rien n'est écrit.
    if !is_letterable_account(tx, company_id, piece.account_id).await? {
        return Ok(SyncOutcome::AccountNotLetterable);
    }

    // La cible `T` (P1) : forme (au moins deux lignes, somme nulle), puis
    // périodes. `forme` seule distingue l'abstention d'un simple « rien ».
    let somme: Decimal = piece.lines.iter().map(|l| l.debit - l.credit).sum();
    let forme = piece.lines.len() >= 2 && somme == Decimal::ZERO;
    let cible = forme && lines_in_open_period(tx, company_id, &line_periods(&piece.lines)).await?;
    let ids_cible: Vec<i64> = piece.lines.iter().map(|l| l.id).collect();
    let mode = Mode::System {
        held_open_fiscal_year_id,
    };

    match existant {
        Some(k) => {
            // (4) Déjà juste ?
            let lignes_k: Vec<i64> = sqlx::query_scalar(GROUP_LINE_IDS_SQL)
                .bind(k)
                .bind(company_id)
                .fetch_all(&mut **tx)
                .await
                .map_err(map_db_error)?;
            if cible && lignes_k == ids_cible {
                return Ok(SyncOutcome::Unchanged);
            }
            // (5) Défensif — aucun geste n'y mène (P3) : un groupe `document`
            // n'existe que sur une facture soldée, qui n'accepte ni règlement,
            // ni solde, ni avoir.
            let periodes_k: Vec<(i64, NaiveDate)> = sqlx::query_as(GROUP_LINE_PERIODS_SQL)
                .bind(k)
                .bind(company_id)
                .fetch_all(&mut **tx)
                .await
                .map_err(map_db_error)?;
            if !lines_in_open_period(tx, company_id, &periodes_k).await? {
                return Ok(SyncOutcome::AbstainedClosedPeriods);
            }
            dissolve_group_inner(tx, company_id, k, mode, actor, Some(&piece.document)).await?;
            if cible {
                let cree = create_group_inner(
                    tx,
                    company_id,
                    &ids_cible,
                    Origin::Document,
                    mode,
                    actor,
                    Some(&piece.document),
                )
                .await?;
                Ok(SyncOutcome::Recreated {
                    dissolved: k,
                    created: cree.key,
                })
            } else {
                Ok(SyncOutcome::Dissolved { key: k })
            }
        }
        // (6) Aucun groupe.
        None if cible => {
            let cree = create_group_inner(
                tx,
                company_id,
                &ids_cible,
                Origin::Document,
                mode,
                actor,
                Some(&piece.document),
            )
            .await?;
            Ok(SyncOutcome::Created { key: cree.key })
        }
        None if forme => Ok(SyncOutcome::AbstainedClosedPeriods),
        None => Ok(SyncOutcome::Unchanged),
    }
}

/// **Défait le groupe `document` d'une facture client** (Story 15-1a2-i, P3) —
/// appelée par l'annulation d'un règlement (`cancel_settlement_in_tx`), **après**
/// ses refus (dont le rang 2 bis, Story 15-1a2-0) et **avant** la
/// contre-passation, qui lettre ensuite le règlement avec son miroir.
///
/// Étapes 1 et 2 de [`sync_invoice_in_tx`] ; aucun groupe → `Unchanged`
/// (**no-op** : l'annulation d'un règlement partiel n'a rien à défaire) ;
/// sinon dissolution en mode `System` → `Dissolved`.
///
/// ⛔ **Ne s'abstient jamais** (C-15-1a2-10) et n'évalue pas la règle des
/// périodes : le geste qui dissoudrait un groupe figé est **refusé en amont**
/// par le rang 2 bis — une seule garde par motif. ⛔ N'exige pas la
/// lettrabilité (C104) : un groupe posé avant que `A` ne devienne non lettrable
/// se dissout.
pub async fn dissolve_invoice_document_group_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    invoice_id: i64,
    held_open_fiscal_year_id: i64,
    actor: Actor,
) -> Result<SyncOutcome, DbError> {
    let piece = discover_invoice_document(tx, company_id, invoice_id).await?;
    dissolve_document_in_tx(tx, company_id, piece, held_open_fiscal_year_id, actor).await
}

/// Le corps de la dissolution, **commun** aux deux familles de pièces (Story
/// 15-1a2-ii, P3 part ii) : étape 2 de la synchronisation sur la découverte
/// passée en paramètre, puis dissolution en mode `System`. Aucune découverte
/// (`None`) ou aucun groupe → `Unchanged`.
async fn dissolve_document_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    piece: Option<PieceDocument>,
    held_open_fiscal_year_id: i64,
    actor: Actor,
) -> Result<SyncOutcome, DbError> {
    let Some(piece) = piece else {
        return Ok(SyncOutcome::Unchanged);
    };
    let Some(k) = existing_document_group(&piece.lines)? else {
        return Ok(SyncOutcome::Unchanged);
    };
    dissolve_group_inner(
        tx,
        company_id,
        k,
        Mode::System {
            held_open_fiscal_year_id,
        },
        actor,
        Some(&piece.document),
    )
    .await?;
    Ok(SyncOutcome::Dissolved { key: k })
}

// ---------------------------------------------------------------------------
// Story 15-1a2-ii (#518) — le lettrage des pièces fournisseurs
// ---------------------------------------------------------------------------

/// La facture fournisseur, **verrouillée** (l'appelant la tient déjà : la
/// relecture `FOR UPDATE` ne prend aucun verrou neuf et lit la ligne courante —
/// après l'`UPDATE` du paiement, le statut `paid` et l'écriture de règlement).
const SYNC_SUPPLIER_INVOICE_SQL: &str = "SELECT status, purchase_journal_entry_id, \
     settlement_journal_entry_id, supplier_invoice_number FROM supplier_invoices \
     WHERE id = ? AND company_id = ? FOR UPDATE";

/// P2 — la découverte d'une facture fournisseur, **par statut** (C-15-1a2-15) :
///
/// | statut | `C(S)` |
/// |---|---|
/// | `paid` | l'ancre et la ligne sur `B` du règlement |
/// | `open` | l'ancre **seule** — jamais de cible |
/// | `cancelled` | **aucune** (`None`) — sans lecture verrouillante des lignes |
///
/// L'**ancre** est la ligne d'achat sur la dette `B` : la première ligne au
/// crédit de l'écriture d'achat ([`super::supplier_invoices::purchase_payable_line`],
/// le lecteur de `pay_in_tx`). Une autre ligne de l'achat sur `B` n'est pas
/// dans `C(S)`.
///
/// ⛔ `cancelled` → `None` : l'achat d'une facture annulée reste **possédé** et
/// la contre-passation l'a lettré `reversal` avec son miroir (15-1a-ii R6) ; le
/// soumettre à l'étape 2 rendrait [`DbError::Invariant`] sur un état légitime.
async fn discover_supplier_invoice_document(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    supplier_invoice_id: i64,
) -> Result<Option<PieceDocument>, DbError> {
    let (status, purchase_entry_id, settlement_entry_id, number): (
        String,
        i64,
        Option<i64>,
        Option<String>,
    ) = sqlx::query_as(SYNC_SUPPLIER_INVOICE_SQL)
        .bind(supplier_invoice_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    let mut entries = vec![purchase_entry_id];
    match (status.as_str(), settlement_entry_id) {
        ("cancelled", _) => return Ok(None),
        ("open", _) => {}
        ("paid", Some(reglement)) => entries.push(reglement),
        (autre, _) => {
            return Err(DbError::Invariant(format!(
                "lettrage de pièce : la facture fournisseur {supplier_invoice_id} ({autre}) n'a \
                 pas d'écriture de règlement"
            )));
        }
    }
    // Comme `pay_in_tx` sur le même cas.
    let (account_id, _) =
        super::supplier_invoices::purchase_payable_line(tx, company_id, purchase_entry_id)
            .await?
            .ok_or_else(|| {
                DbError::Invariant("écriture d'achat sans ligne de crédit créanciers".into())
            })?;

    let sql = with_placeholders(SYNC_DOCUMENT_LINES_SQL, entries.len());
    let mut q = sqlx::query_as::<_, LineRow>(&sql);
    for e in &entries {
        q = q.bind(*e);
    }
    let lues = q
        .bind(account_id)
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // L'ancre : la PREMIÈRE ligne au crédit de l'écriture d'achat (lignes triées
    // par `id`) — celle que lit `purchase_payable_line`.
    let ancre = lues
        .iter()
        .find(|l| l.entry_id == purchase_entry_id && l.credit > Decimal::ZERO)
        .map(|l| l.id)
        .ok_or_else(|| {
            DbError::Invariant(format!(
                "lettrage de pièce : l'ancre de la facture fournisseur {supplier_invoice_id} est \
                 introuvable"
            ))
        })?;
    let lines = lues
        .into_iter()
        .filter(|l| l.entry_id != purchase_entry_id || l.id == ancre)
        .collect();
    Ok(Some(PieceDocument {
        account_id,
        lines,
        document: DocumentRef {
            document_type: DocumentKind::SupplierInvoice,
            id: supplier_invoice_id,
            number,
        },
    }))
}

/// **Synchronise le lettrage `document` d'une facture fournisseur** avec son
/// règlement (Story 15-1a2-ii, P2, P3 part ii) — idempotente, dans la
/// transaction de l'appelant.
///
/// Le groupe d'une facture **payée** `S` est `C(S)` : l'ancre (sa ligne d'achat
/// sur la dette `B`) et la ligne sur `B` de son règlement. Il existe **si et
/// seulement si** `|C(S)| ≥ 2`, la somme est nulle, `B` est lettrable et au
/// moins une ligne est en période ouverte. La découverte dépend du **statut**
/// (`discover_supplier_invoice_document`) ; le reste — étapes 2 à 6, précédence
/// des issues, mode `System`, tolérances nommées — est celui de
/// [`sync_invoice_in_tx`], **le même code** (`sync_document_in_tx`).
///
/// ⛔ Un paiement n'est **jamais refusé** à cause du lettrage : `B` non lettrable
/// → [`SyncOutcome::AccountNotLetterable`]. Une erreur **structurelle**
/// ([`DbError::Invariant`]) se propage, et annule le paiement avec sa transaction
/// — dans `confirm_batch`, le lot entier.
///
/// Audit : `documentType = "supplierInvoice"`, `documentNumber` nul quand la
/// facture n'a pas de numéro (C-15-1a2-17).
pub async fn sync_supplier_invoice_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    supplier_invoice_id: i64,
    held_open_fiscal_year_id: i64,
    actor: Actor,
) -> Result<SyncOutcome, DbError> {
    let piece = discover_supplier_invoice_document(tx, company_id, supplier_invoice_id).await?;
    sync_document_in_tx(tx, company_id, piece, held_open_fiscal_year_id, actor).await
}

/// **Défait le groupe `document` d'une facture fournisseur** (Story 15-1a2-ii,
/// P4 part ii) — appelée par l'annulation du règlement (`cancel_settlement_in_tx`)
/// et par l'annulation d'une facture (`cancel_in_tx`), **après** leurs refus
/// (dont le rang 2 bis, Story 15-1a2-0) et **avant** la contre-passation, qui
/// lettre ensuite ce qui est libre avec son miroir (15-1a-ii R6).
///
/// Mêmes règles que [`dissolve_invoice_document_group_in_tx`] (même corps) :
/// ne s'abstient jamais, n'exige pas la lettrabilité ; aucun groupe → no-op
/// (facture ouverte : elle n'en a jamais, P2).
pub async fn dissolve_supplier_invoice_document_group_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    supplier_invoice_id: i64,
    held_open_fiscal_year_id: i64,
    actor: Actor,
) -> Result<SyncOutcome, DbError> {
    let piece = discover_supplier_invoice_document(tx, company_id, supplier_invoice_id).await?;
    dissolve_document_in_tx(tx, company_id, piece, held_open_fiscal_year_id, actor).await
}

/// Lit un groupe **sans verrou** (`GET /letterings/{key}`). `None` si aucune
/// ligne de la société ne porte cette clé — une clé d'une autre société est
/// indiscernable d'une clé inexistante (AC11).
pub async fn find_group(
    conn: &mut MySqlConnection,
    company_id: i64,
    key: i64,
) -> Result<Option<LetteringGroup>, DbError> {
    let lines = sqlx::query_as::<_, LineRow>(FIND_GROUP_SQL)
        .bind(key)
        .bind(company_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    let Some(premiere) = lines.first() else {
        return Ok(None);
    };
    let origin = premiere
        .lettering_origin
        .as_deref()
        .and_then(Origin::parse)
        .ok_or_else(|| {
            DbError::Invariant(format!("lettrage : origine illisible pour le groupe {key}"))
        })?;
    let account_id = premiere.account_id;
    let exercices: BTreeSet<i64> = lines.iter().map(|l| l.fiscal_year_id).collect();
    let names = fiscal_year_names(&mut *conn, company_id, &exercices).await?;
    let account_number = group_account_number(conn, company_id, account_id).await?;
    build_group(key, origin, account_id, account_number, &lines, &names).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_round_trip() {
        for o in [Origin::Document, Origin::Reversal, Origin::Manual] {
            assert_eq!(Origin::parse(o.as_str()), Some(o));
        }
        assert_eq!(Origin::parse("autre"), None);
    }

    #[test]
    fn placeholders_are_expanded() {
        assert_eq!(
            with_placeholders("x IN ({ids}) AND", 3),
            "x IN (?, ?, ?) AND"
        );
    }

    /// AC10 (revue P1, A-4) — `documentNumber` est PRÉSENT et `null` quand la
    /// pièce n'a pas de numéro ; aucune clé `document*` sans pièce.
    #[test]
    fn audit_details_carry_the_document_or_nothing() {
        let group = LetteringGroup {
            key: 27,
            code: code_of(27),
            origin: Origin::Document,
            account_id: 3,
            account_number: "1100".into(),
            lines: vec![],
        };
        let sans = audit_details(&group, None);
        for cle in ["documentType", "documentId", "documentNumber"] {
            assert!(sans.get(cle).is_none(), "{cle} sans pièce");
        }
        let doc = DocumentRef {
            document_type: DocumentKind::Invoice,
            id: 9,
            number: None,
        };
        let avec = audit_details(&group, Some(&doc));
        assert_eq!(avec["documentType"], "invoice");
        assert_eq!(avec["documentId"], 9);
        assert!(
            avec.get("documentNumber")
                .is_some_and(serde_json::Value::is_null),
            "documentNumber présent et null : {avec}"
        );
        let numerote = DocumentRef {
            number: Some("F-1".into()),
            ..doc
        };
        assert_eq!(
            audit_details(&group, Some(&numerote))["documentNumber"],
            "F-1"
        );
        assert_eq!(avec["code"], "AA", "les clés d'origine restent");
    }

    #[test]
    fn manual_line_count_is_capped_at_200() {
        let ids: Vec<i64> = (1..=200).collect();
        assert!(check_manual_line_count(&ids).is_ok());
        let ids: Vec<i64> = (1..=201).collect();
        assert!(matches!(
            check_manual_line_count(&ids),
            Err(DbError::LetteringTooManyLines { max: 200 })
        ));
    }
}
