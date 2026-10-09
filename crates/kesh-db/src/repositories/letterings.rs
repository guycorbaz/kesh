//! Lettrage — la marque, posée et retirée par un seul chemin (Story 15-1a-i, #518).
//!
//! Un **groupe de lettrage** est un ensemble d'au moins deux lignes
//! d'écriture d'une même société, sur un même compte **lettrable**, dont la
//! somme `Σ(débit − crédit)` est exactement nulle. La marque est portée par
//! chaque ligne (`journal_entry_lines.lettering_key`, `lettering_origin`, R1) ;
//! la clé du groupe est le **plus petit `id`** de ses lignes, son code affiché
//! cette clé écrite en lettres ([`kesh_core::lettering::code_from_key`], R2).
//!
//! ⛔ **UNE seule fonction écrit la marque** — [`create_group_in_tx`] — **et
//! une seule la retire** — [`dissolve_group_in_tx`] (R3). Aucun autre
//! `UPDATE` ni `INSERT` du code de production ne nomme `lettering_key` ou
//! `lettering_origin` : le test lexical
//! `crates/kesh-db/tests/letterings_lexical.rs` le vérifie. Exceptions,
//! nommées : la migration de rattrapage de la 15-1a2 (SQL de migration), la
//! **restauration d'une sauvegarde** (`backup.rs`, colonnes lues
//! dynamiquement — un groupe restauré est celui qui existait), et les
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

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{MySql, MySqlConnection, Transaction};

use kesh_core::lettering::{self as core_lettering, LetteringRefusal};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::errors::{DbError, ReversalBlocker, map_db_error};
use crate::repositories::{audit_log, fiscal_years, journal_entries};

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

/// Un compte est **lettrable** (R4) s'il appartient à la société, est de type
/// `Asset` ou `Liability`, et qu'aucun `bank_accounts.journal_account_id` ne le
/// désigne — **archivé ou non** (C127 : un compte qui a été celui d'un compte
/// bancaire relève de la réconciliation, et ne redevient pas lettrable à
/// l'archivage). Un compte archivé (`accounts.active = FALSE`) reste lettrable :
/// ses lignes existent et se soldent.
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
    let row: Option<(String, String, bool)> = sqlx::query_as(
        "SELECT a.account_type, a.number, \
                EXISTS (SELECT 1 FROM bank_accounts b WHERE b.journal_account_id = a.id) \
         FROM accounts a WHERE a.id = ? AND a.company_id = ?",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)?;
    Ok(row.map(|(account_type, number, banque)| {
        let lettrable = matches!(account_type.as_str(), "Asset" | "Liability") && !banque;
        (lettrable, number)
    }))
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
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<Option<NaiveDate>, DbError> {
    sqlx::query_scalar("SELECT books_locked_through FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)
}

/// Règle des périodes (R7) : au moins une ligne « en période ouverte ».
fn any_line_in_open_period(
    lines: &[LineRow],
    exercices: &BTreeMap<i64, FiscalYearState>,
    locked_through: Option<NaiveDate>,
) -> bool {
    lines.iter().any(|l| {
        exercices
            .get(&l.fiscal_year_id)
            .is_some_and(|e| e.open && !e.later_closed)
            && locked_through.is_none_or(|borne| l.entry_date > borne)
    })
}

/// R5 — la première ligne dont l'écriture appartient à une **pièce** (motifs
/// `OwnedByInvoice`, `OwnedByCreditNote`, `OwnedBySupplierInvoice`,
/// `OwnedBySettlement` de `reversal_blockers`, rangs 3 à 6 — réutilisés, jamais
/// une seconde liste), dans l'ordre des lignes.
async fn first_document_owner(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    lines: &[LineRow],
) -> Result<Option<DbError>, DbError> {
    let mut vues = BTreeSet::new();
    for line in lines {
        if !vues.insert(line.entry_id) {
            continue;
        }
        let motifs =
            journal_entries::reversal_blockers(&mut **tx, company_id, line.entry_id).await?;
        if let Some((blocker, document_id, document_label)) =
            motifs.into_iter().find(|(b, _, _)| {
                matches!(
                    b,
                    ReversalBlocker::OwnedByInvoice
                        | ReversalBlocker::OwnedByCreditNote
                        | ReversalBlocker::OwnedBySupplierInvoice
                        | ReversalBlocker::OwnedBySettlement
                )
            })
        {
            return Ok(Some(DbError::LetteringLineOwnedByDocument {
                blocker,
                document_id,
                document_label,
            }));
        }
    }
    Ok(None)
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
fn build_group(
    key: i64,
    origin: Origin,
    account_id: i64,
    account_number: String,
    lines: &[LineRow],
    names: &BTreeMap<i64, String>,
) -> LetteringGroup {
    LetteringGroup {
        key,
        code: code_of(key),
        origin,
        account_id,
        account_number,
        lines: lines
            .iter()
            .map(|l| LetteringLine {
                id: l.id,
                entry_id: l.entry_id,
                entry_number: l.entry_number,
                fiscal_year_id: l.fiscal_year_id,
                // Les deux lectures de noms rendent `Invariant` s'il en manque un
                // (E-2) : le repli vide n'est plus atteignable.
                fiscal_year_name: names.get(&l.fiscal_year_id).cloned().unwrap_or_default(),
                date: l.entry_date,
                debit: l.debit,
                credit: l.credit,
            })
            .collect(),
    }
}

/// `details` de l'audit `lettering.created` / `lettering.removed` (AC10).
fn audit_details(group: &LetteringGroup) -> serde_json::Value {
    serde_json::json!({
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
    })
}

/// **Pose la marque** d'un groupe de lettrage — la seule fonction qui l'écrit
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
/// La séquence des verrous est au doc-comment du module.
pub async fn create_group_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    line_ids: &[i64],
    origin: Origin,
    mode: Mode,
    actor: Actor,
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

    let group = build_group(key, origin, account_id, account_number, &lines, &names);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            actor.user_id,
            actor.api_key_id,
            "lettering.created",
            "lettering",
            key,
            Some(audit_details(&group)),
        ),
    )
    .await?;
    Ok(group)
}

/// **Retire la marque** d'un groupe — la seule fonction qui l'efface (R3).
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
pub async fn dissolve_group_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    key: i64,
    mode: Mode,
    actor: Actor,
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

    let group = build_group(key, origin, account_id, account_number, &lines, &names);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            actor.user_id,
            actor.api_key_id,
            "lettering.removed",
            "lettering",
            key,
            Some(audit_details(&group)),
        ),
    )
    .await?;
    Ok(group)
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
    Ok(Some(build_group(
        key,
        origin,
        account_id,
        account_number,
        &lines,
        &names,
    )))
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
