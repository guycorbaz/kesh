//! Repository pour les écritures comptables en partie double.
//!
//! L'invariant central est l'**atomicité** : une écriture (en-tête +
//! lignes) est toujours créée ou rejetée en bloc.
//!
//! # Numérotation — ce que la contrainte garantit, et ce qu'elle ne garantit pas
//!
//! `SELECT MAX(entry_number) FOR UPDATE` + `UNIQUE (company_id,
//! fiscal_year_id, entry_number)` garantissent qu'**à un instant donné**, deux
//! écritures d'un même exercice ne portent jamais le même numéro.
//!
//! ⚠️ **Ils ne garantissent NI la continuité, NI l'univocité dans le temps** :
//!
//! - supprimer l'écriture n° 42 au milieu laisse un **trou définitif** ;
//! - supprimer la **dernière** fait **réattribuer son numéro** à la suivante,
//!   qui aura un contenu différent.
//!
//! Le commentaire de la migration `20260412000001_journal_entries.sql` affirme
//! « jamais de trou » : c'est **faux**, et ce fichier ne peut plus être corrigé
//! (P8 — son checksum est enregistré, le modifier empêche le démarrage).
//!
//! Pour un contrôleur, une séquence ni continue ni univoque est un signal
//! d'alarme. C'est une raison de plus de corriger par contre-passation plutôt
//! que par suppression. Suivi : issues du jalon « Vague 1 » (audit du
//! 2026-08-26).
//!
//! # Defense in depth
//!
//! Trois niveaux de garde-fou empêchent une écriture déséquilibrée :
//!
//! 1. `kesh_core::accounting::validate()` (logique pure, côté route)
//! 2. Contrainte DB `chk_jel_debit_credit_exclusive` (par ligne)
//! 3. Re-calcul `SUM(debit) = SUM(credit)` après INSERT dans ce
//!    repository (rollback si mismatch)
//!
//! # Immutabilité post-clôture (FR24, CO art. 957-964)
//!
//! Un `SELECT fiscal_years FOR UPDATE` en tête de transaction verrouille
//! l'exercice contre toute clôture concurrente. Si `status = 'Closed'`,
//! la création est refusée avec `DbError::IllegalStateTransition`.

use std::str::FromStr;

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::mysql::MySqlPool;
use sqlx::{QueryBuilder, Row};

use kesh_core::listing::{SortBy, SortDirection};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{
    Journal, JournalEntry, JournalEntryLine, JournalEntryWithLines, NewJournalEntry,
    NewJournalEntryLine,
};
use crate::errors::{ArchivedAccount, DbError, ReversalBlocker, map_db_error};
use crate::repositories::audit_log;
use crate::util::search::escape_boolean_ft;

const ENTRY_COLUMNS: &str = "id, company_id, fiscal_year_id, entry_number, entry_date, journal, description, \
     version, reverses_entry_id, created_at, updated_at";

const LINE_COLUMNS: &str = "id, entry_id, account_id, line_order, debit, credit, project_id";

/// Valide que chaque compte référencé par les lignes d'une écriture existe,
/// appartient à `company_id` et est actif. Facteur commun de [`create_in_tx`]
/// et [`update`] (validation historiquement dupliquée aux deux endroits).
///
/// Garde de postabilité (Story 14-3b, D-A0) — **saisie manuelle uniquement** :
/// si `enforce_postable`, un compte `postable = FALSE` est rejeté, SAUF s'il
/// figure dans `exempt_ids` (grandfather PAR COMPTE à l'update, D-A1 : un compte
/// déjà référencé par l'écriture reste éditable même devenu non-postable après
/// coup). Les flux automatiques (facture/avoir/réconciliation) passent
/// `enforce_postable = false` : ils postent sur des comptes de config approuvés,
/// dont l'un pourrait légitimement être devenu non-postable (14-3a) — leur
/// imposer la garde casserait le moteur comptable.
///
/// **Rollback-agnostique** : retourne `Err(DbError::InactiveOrInvalidAccounts)`
/// sans toucher à la transaction — le caller décide du rollback (`create_in_tx`
/// délègue au caller, `update` rollback autour de l'appel).
///
/// Précondition : `account_ids` non vide (garanti par les callers, qui rejettent
/// une écriture sans lignes en amont — un `IN ()` serait du SQL invalide).
async fn validate_lines_accounts_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    account_ids: &[i64],
    enforce_postable: bool,
    exempt_ids: &[i64],
) -> Result<(), DbError> {
    let placeholders = account_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let mut accounts_sql = format!(
        "SELECT id FROM accounts \
         WHERE company_id = ? AND active = TRUE AND id IN ({placeholders})"
    );
    // Garde de postabilité conditionnelle (D-A0) : la clause n'est ajoutée qu'en
    // saisie manuelle. Les comptes de `exempt_ids` (déjà référencés, D-A1) sont
    // tolérés même non-postables.
    if enforce_postable {
        if exempt_ids.is_empty() {
            accounts_sql.push_str(" AND postable = TRUE");
        } else {
            let exempt_placeholders = exempt_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            accounts_sql.push_str(&format!(
                " AND (postable = TRUE OR id IN ({exempt_placeholders}))"
            ));
        }
    }

    let mut q = sqlx::query_scalar::<_, i64>(&accounts_sql).bind(company_id);
    for id in account_ids {
        q = q.bind(id);
    }
    if enforce_postable {
        for id in exempt_ids {
            q = q.bind(id);
        }
    }
    let valid_ids: Vec<i64> = q.fetch_all(&mut **tx).await.map_err(map_db_error)?;

    let mut unique_requested: Vec<i64> = account_ids.to_vec();
    unique_requested.sort_unstable();
    unique_requested.dedup();

    if valid_ids.len() != unique_requested.len() {
        return Err(DbError::InactiveOrInvalidAccounts);
    }
    Ok(())
}

/// Crée une écriture comptable (en-tête + lignes) dans une transaction
/// atomique. Wrapper pool-level : ouvre sa propre transaction et la
/// valide/rollback selon le résultat. Délègue à [`create_in_tx`] pour
/// tout le travail métier.
///
/// Le `fiscal_year_id` doit être pré-validé par le caller via
/// [`fiscal_years::find_covering_date`](super::fiscal_years::find_covering_date)
/// — il est re-vérifié ici avec `FOR UPDATE` pour capturer les races
/// avec une clôture concurrente.
pub async fn create(
    pool: &MySqlPool,
    fiscal_year_id: i64,
    user_id: i64,
    new: NewJournalEntry,
) -> Result<JournalEntryWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    // Point d'entrée MANUEL (unique appelant `routes/journal_entries.rs`) :
    // la garde de postabilité s'applique (Story 14-3b, D-A0).
    match create_in_tx(&mut tx, fiscal_year_id, user_id, new, true).await {
        Ok(result) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(result)
        }
        Err(e) => {
            // Best-effort rollback. Si le rollback échoue lui-même, on
            // privilégie l'erreur métier originale — l'appelant la verra
            // en premier et le drop-guard SQLx annulera la tx en arrière-plan.
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Cœur métier de [`create`] — accepte une transaction ouverte par le
/// caller et **ne commit/rollback PAS** (responsabilité du caller).
///
/// Utilisée à deux endroits :
/// 1. [`create`] (wrapper pool-level) — cas standard.
/// 2. [`invoices::validate_invoice`](super::invoices::validate_invoice)
///    (Story 5.2) — pour garantir l'atomicité { numérotation facture +
///    insertion écriture comptable + UPDATE invoices.status } dans une
///    seule transaction (impossible si `create` ouvre sa propre tx).
///
/// Contrat : en cas de succès, retourne `Ok(JournalEntryWithLines)` et
/// la tx contient les inserts. En cas d'erreur, bubble-up sans toucher
/// à la tx — le caller doit rollback ou laisser le drop-guard agir.
///
/// `enforce_postable` (Story 14-3b, D-A0) : `true` uniquement pour la SAISIE
/// MANUELLE (`create` pool-level). Les flux automatiques appelants directs
/// (invoices, credit_notes, supplier_invoices, reconciliation) passent `false` —
/// ils postent sur des comptes de config approuvés, potentiellement devenus
/// non-postables (14-3a), qu'on ne doit pas rejeter.
pub async fn create_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    fiscal_year_id: i64,
    user_id: i64,
    new: NewJournalEntry,
    enforce_postable: bool,
) -> Result<JournalEntryWithLines, DbError> {
    create_in_tx_inner(
        tx,
        fiscal_year_id,
        user_id,
        new,
        enforce_postable,
        true,
        None,
    )
    .await
}

/// Corps de [`create_in_tx`], avec les deux réglages que la contre-passation
/// (Story 24-4a, #380) est seule à employer.
///
/// `enforce_project_taggable` — `true` partout sauf pour la contre-passation.
/// Celle-ci **copie** les tags de projet de l'écriture d'origine, elle ne les
/// choisit pas : les re-valider ferait échouer la correction d'une écriture dont
/// un projet a été archivé depuis, c'est-à-dire rendrait cette écriture
/// définitivement incorrigible. Le dépôt tranche déjà ainsi pour les flux
/// automatiques document-level (cf. `pay_succeeds_when_project_archived_after_tagging`).
///
/// ⚠️ **L'asymétrie avec le compte archivé est VOULUE** : un compte archivé fait
/// échouer la contre-passation (la garde `active = TRUE` de [`validate_accounts`]
/// protège tous les flux, et poster sur un compte archivé le ressusciterait),
/// alors qu'un projet archivé est toléré. Étiqueter une ligne n'est pas écrire
/// dans un compte.
///
/// `reverses_entry_id` — l'écriture contre-passée, `None` pour toute écriture
/// ordinaire. L'`UNIQUE` de la colonne porte l'idempotence **structurellement**,
/// sans pré-`SELECT`, donc elle tient sous concurrence.
#[allow(clippy::too_many_arguments)]
async fn create_in_tx_inner(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    fiscal_year_id: i64,
    user_id: i64,
    new: NewJournalEntry,
    enforce_postable: bool,
    enforce_project_taggable: bool,
    reverses_entry_id: Option<i64>,
) -> Result<JournalEntryWithLines, DbError> {
    // Étape 0 : validation des projets analytiques par-ligne (Story 19-2).
    // AVANT le lock fiscal_years pour respecter l'ordre de verrouillage global
    // companies → projects → fiscal_years (Pattern 5) : le flux fournisseur 19-3
    // prend déjà sentinel + projects en amont de cet appel — valider ici après
    // le lock fiscal_years créerait une inversion ABBA inter-flux.
    //
    // Périmètre : UNIQUEMENT les tags par-ligne explicites. `new.project_id`
    // (document-level 19-3) n'est PAS re-validé ici — les flux pay/cancel
    // stampent volontairement un projet potentiellement archivé après coup
    // (cf. test supplier `pay_succeeds_when_project_archived_after_tagging`).
    if enforce_project_taggable {
        let line_project_ids: Vec<i64> = new.lines.iter().filter_map(|l| l.project_id).collect();
        super::projects::validate_taggable_in_tx(tx, new.company_id, &line_project_ids).await?;
    }

    // Étape 1 : re-lock de l'exercice contre une clôture concurrente + bornes de dates.
    let fy_row: Option<(i64, String, NaiveDate, NaiveDate)> = sqlx::query_as(
        "SELECT id, status, start_date, end_date FROM fiscal_years \
         WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(fiscal_year_id)
    .bind(new.company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    match fy_row {
        None => return Err(DbError::NotFound),
        Some((_, status, _, _)) if status == "Closed" => return Err(DbError::FiscalYearClosed),
        Some((_, _, fy_start, fy_end)) => {
            // Garde défensive symétrique à `update` (:671) — l'invariant
            // `entry_date ∈ [fy_start, fy_end]` est ce dont dépend l'équation du
            // bilan cumulatif (Story 14-1 Dev Note 4) : l'actif/passif cumulés
            // ignorent `fiscal_year_id`, mais `equity_result` (via income_statement)
            // le garde → l'égalité ne tient que si chaque écriture tombe dans les
            // bornes de son exercice. Le handler HTTP le garantit déjà via
            // `find_covering_date`, mais on l'impose ici pour tout caller de
            // `create_in_tx` (fix structurel : équation vraie par construction).
            if new.entry_date < fy_start || new.entry_date > fy_end {
                return Err(DbError::DateOutsideFiscalYear);
            }
        }
    }

    // Étape 2 : vérifier que tous les comptes existent, appartiennent
    // à la company, sont actifs et (saisie manuelle seulement) postables.
    if new.lines.is_empty() {
        return Err(DbError::Invariant(
            "NewJournalEntry sans lignes — devait être rejeté en amont".into(),
        ));
    }

    let account_ids: Vec<i64> = new.lines.iter().map(|l| l.account_id).collect();
    // Validation factorisée (helper rollback-agnostique). En création, aucun
    // compte n'est encore référencé → `exempt_ids` vide ; la garde de
    // postabilité dépend de `enforce_postable` (D-A0).
    validate_lines_accounts_in_tx(tx, new.company_id, &account_ids, enforce_postable, &[]).await?;

    // Étape 3 : calculer le prochain entry_number (sérialisé par gap lock).
    let next_number: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
    )
    .bind(new.company_id)
    .bind(fiscal_year_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Étape 4 : INSERT de l'en-tête.
    //
    // ⛔ **La colonne `reverses_entry_id` n'entre dans l'INSERT que si elle a
    // quelque chose à dire.** Ce n'est pas une coquetterie : plusieurs tests du
    // dépôt montent le schéma à un point de migration ANTÉRIEUR puis exercent le
    // vrai chemin applicatif (`invoice_lines_revenue_account_backfill`). Nommer
    // une colonne née le 2026-08-28 dans un INSERT joué contre le schéma de
    // juillet donne `1054 Unknown column` — un rouge sur un fichier que la story
    // ne touche pas, et que seul le gate réellement exécuté révèle.
    let header_result = if let Some(reverses) = reverses_entry_id {
        sqlx::query(
            "INSERT INTO journal_entries \
             (company_id, fiscal_year_id, entry_number, entry_date, journal, description, \
              reverses_entry_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new.company_id)
        .bind(fiscal_year_id)
        .bind(next_number)
        .bind(new.entry_date)
        .bind(new.journal)
        .bind(&new.description)
        .bind(reverses)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?
    } else {
        sqlx::query(
            "INSERT INTO journal_entries \
             (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new.company_id)
        .bind(fiscal_year_id)
        .bind(next_number)
        .bind(new.entry_date)
        .bind(new.journal)
        .bind(&new.description)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?
    };

    let last_id = header_result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT journal_entries".into(),
        ));
    }
    let entry_id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    // Étape 5 : INSERT des lignes avec line_order séquentiel. Le tag
    // analytique par-ligne (19-2, écritures manuelles) prime sur le tag
    // document-level (19-3, propagation facture) — aucun flux ne fournit
    // les deux à la fois.
    for (idx, line) in new.lines.iter().enumerate() {
        let line_order = (idx as i32) + 1;
        sqlx::query(
            "INSERT INTO journal_entry_lines \
             (entry_id, account_id, line_order, debit, credit, project_id) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(line.account_id)
        .bind(line_order)
        .bind(line.debit)
        .bind(line.credit)
        .bind(line.project_id.or(new.project_id))
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    }

    // Étape 6 : double-check balance applicative (defense in depth).
    let row = sqlx::query(
        "SELECT COALESCE(SUM(debit), 0) AS d, COALESCE(SUM(credit), 0) AS c \
         FROM journal_entry_lines WHERE entry_id = ?",
    )
    .bind(entry_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let total_debit: Decimal = row.try_get("d").map_err(map_db_error)?;
    let total_credit: Decimal = row.try_get("c").map_err(map_db_error)?;

    if total_debit != total_credit {
        return Err(DbError::Invariant(format!(
            "balance DB incohérente après INSERT : débit={total_debit}, crédit={total_credit}"
        )));
    }

    // Étape 7 : re-fetch entry + lines pour le retour.
    let entry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(entry_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(entry_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Étape 8 (Story 3.5) : INSERT audit_log avant le COMMIT.
    // Le caller qui appelle create_in_tx peut ajouter son propre audit
    // (ex. validate_invoice ajoute « invoice.validated » en complément).
    let snapshot = entry_snapshot_json(&entry, &lines);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "journal_entry.created".to_string(),
            "journal_entry".to_string(),
            entry_id,
            Some(snapshot),
        ),
    )
    .await?;

    Ok(JournalEntryWithLines { entry, lines })
}

/// Compte les écritures d'une company, **tous exercices confondus**.
///
/// Story 14-4 — garde « company vierge » du bilan d'ouverture (P3-BH3-1) :
/// la génération de l'écriture d'ouverture n'est autorisée que si ce compte
/// vaut `0`. Générique sur `Executor` (idiome projet, cf.
/// `reconciliation::find_contacts_by_ids`) : appelée sur `&mut *tx` **sous le
/// lock** dans [`create_opening_entry`], et sur `&pool` pour le
/// `GET /opening-balances/status` (P3-ECH-LOW-2 — pas deux fns dupliquées).
pub async fn count_by_company<'e, E>(executor: E, company_id: i64) -> Result<i64, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(executor)
        .await
        .map_err(map_db_error)
}

/// Crée l'**écriture d'ouverture** du bilan de départ (Story 14-4, D5/P1-C1).
///
/// Fn repo dédiée — miroir du pattern `invoices::validate_invoice` (own tx +
/// [`create_in_tx`] direct). **NE PAS** réutiliser [`create`] : le wrapper est
/// auto-contenu (ouvre/commit sa propre tx) et n'offre aucun point d'injection
/// pour le re-check « company vierge » sous le lock — la garde anti-course
/// serait irréalisable (finding P1-C1).
///
/// Algorithme (chaque garde **sous** les verrous — priorité des gardes alignée
/// sur `GET /status`, amendement ECH3-2 propagé au chemin d'écriture en Pass 4) :
///
/// 1. `tx = pool.begin()` puis sentinel `SELECT id FROM companies WHERE id=?
///    FOR UPDATE` (`None` → `NotFound`).
/// 2. `SELECT id, status FROM fiscal_years WHERE id=? AND company_id=? FOR UPDATE`
///    → `None` → `NotFound` (le statut est examiné à l'étape 3-bis).
/// 3. [`count_by_company`] `(&mut *tx)` **sous le lock** → `> 0` →
///    `Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY)` (garde double-ouverture
///    company-wide, P3-BH3-1 — évaluée AVANT le statut clos : sur
///    « clos + écritures », le verdict doit être `ALREADY_HAS_ENTRIES`, pas le
///    conseil trompeur « rouvrez l'exercice »).
/// 4. Statut `Closed` → `Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)`.
/// 5. [`create_in_tx`] (`enforce_postable = true` — la grille n'offre que des
///    comptes postables, saisie manuelle assistée) : re-lock ré-entrant même tx,
///    validation comptes/équilibre/date, INSERT, audit `journal_entry.created`.
/// 6. `commit`.
///
/// **Note atomicité** : l'écriture d'ouverture n'a aucun `project_id` (DTO sans
/// champ projet) → `create_in_tx` court-circuite l'Étape 0 (validation projets)
/// et ne verrouille QUE `fiscal_years` — pas d'inversion de l'ordre de verrou
/// global `companies → projects → fiscal_years` (P3-ECH confirmé).
///
/// **Portée exacte de la sérialisation** (Pass 1 BH-1 + Pass 3 BH3-1 code
/// review) : la garde « company vierge » est **company-wide**, donc le verrou
/// qui la protège l'est aussi — **sentinel `companies FOR UPDATE`** en tête de
/// transaction (Pattern 5, idiome projet : `bank_accounts`, `projects`,
/// `invoices`). Ce sentinel sérialise **toutes** les générations d'ouverture
/// concurrentes de la company entre elles, *y compris* quand deux requêtes ont
/// résolu des « premiers exercices » **différents** (un exercice antérieur créé
/// entre les deux pré-checks hors-lock) — sans lui, chacune verrouillerait une
/// ligne `fiscal_years` distincte, les deux liraient `count == 0` et
/// commiteraient un **doublon d'ouverture** (bilan doublé, Pass 3 BH3-1).
/// L'ordre `companies → fiscal_years` respecte l'ordre de verrou global
/// (`companies → projects → fiscal_years`) — pas d'inversion ABBA.
///
/// Une écriture normale postée concurremment dans un **autre** exercice ne se
/// sérialise pas avec la génération (elle ne prend pas le sentinel) — mais
/// l'entrelacement est **bénin** : l'état final (ouverture + écriture normale)
/// est identique à l'ordre séquentiel légal « génération puis écriture » ; et
/// toute écriture **commitée avant** le lock est vue par le `count_by_company`
/// sous le lock (→ refus). Résidu restant (une seule génération concurrente à
/// la création d'un exercice antérieur) : **mauvais millésime, pas de doublon**
/// — limitation L4 documentée du story file, remédiation = marqueur L3.
///
/// Les conflits métier sont émis en `DbError::Invariant(<KEY>)` namespacés
/// (pattern D7 de 14-2), re-mappés en messages distincts localisés par
/// `map_opening_balances_error` côté route.
pub async fn create_opening_entry(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    user_id: i64,
    new: NewJournalEntry,
) -> Result<JournalEntryWithLines, DbError> {
    use super::fiscal_years::{
        FY_OPENING_ALREADY_HAS_ENTRIES_KEY, FY_OPENING_FIRST_YEAR_CLOSED_KEY,
    };

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Étape 1 (Pass 3 BH3-1) : sentinel company-wide — la garde « company
    // vierge » porte sur TOUTE la company, le verrou qui la protège aussi.
    // Sans lui, deux générations ayant résolu des premiers exercices
    // DIFFÉRENTS verrouilleraient des lignes distinctes et commiteraient un
    // doublon d'ouverture (portée exacte : cf. doc de la fn).
    let company_row: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM companies WHERE id = ? FOR UPDATE")
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
    if company_row.is_none() {
        let _ = tx.rollback().await;
        return Err(DbError::NotFound);
    }

    // Étape 2 : lock de la ligne du premier exercice — statut vivant + borne
    // de dates, et sérialisation avec les écritures concurrentes de CE même
    // exercice ainsi que toute clôture.
    let fy_row: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, status FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(fiscal_year_id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let fy_status = match fy_row {
        None => {
            let _ = tx.rollback().await;
            return Err(DbError::NotFound);
        }
        Some((_, status)) => status,
    };

    // Étape 3 : garde « company vierge » SOUS le lock (P3-BH3-1), évaluée
    // AVANT le statut clos (priorité amendée ECH3-2, propagée au chemin
    // d'écriture en Pass 4 — BH4/ECH4 convergés) : sur « clos + écritures »,
    // le POST doit rendre le même verdict que GET /status
    // (`ALREADY_HAS_ENTRIES`), pas le conseil trompeur « rouvrez l'exercice ».
    let count = count_by_company(&mut *tx, company_id).await?;
    if count > 0 {
        let _ = tx.rollback().await;
        return Err(DbError::Invariant(
            FY_OPENING_ALREADY_HAS_ENTRIES_KEY.to_string(),
        ));
    }

    // Étape 4 : statut vivant du premier exercice (après la garde
    // company-vierge — même ordre que le GET /status et le pré-check handler).
    if fy_status == "Closed" {
        let _ = tx.rollback().await;
        return Err(DbError::Invariant(
            FY_OPENING_FIRST_YEAR_CLOSED_KEY.to_string(),
        ));
    }

    // Étape 5 : création atomique dans la même tx (re-lock ré-entrant).
    match create_in_tx(&mut tx, fiscal_year_id, user_id, new, true).await {
        Ok(result) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(result)
        }
        Err(e) => {
            // Best-effort rollback, miroir `create` (l'erreur métier prime).
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Retourne une écriture avec ses lignes, scopée à une company pour
/// éviter toute fuite cross-tenant.
///
/// **P10 defense in depth** : le paramètre `company_id` est obligatoire.
/// Une écriture d'une autre company retourne `None`, jamais de donnée.
/// Pattern à reproduire pour toute future route `GET /journal-entries/:id`.
pub async fn find_by_id(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<JournalEntryWithLines>, DbError> {
    let entry_opt = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?;

    let Some(entry) = entry_opt else {
        return Ok(None);
    };

    let lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(entry.id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok(Some(JournalEntryWithLines { entry, lines }))
}

/// Limites hard sur la pagination (borne défensive, la source canonique
/// est le route handler).
const MAX_LIMIT: i64 = 500;

/// Valeur maximale safe pour un filtre `amount_max` absent, exactement
/// alignée avec le maximum stockable en `DECIMAL(19,4)` : 15 chiffres
/// entiers + 4 décimales = `999'999'999'999'999.9999`.
///
/// Utilisé comme borne supérieure pour une sous-requête `HAVING SUM(debit)
/// BETWEEN ? AND ?` : tout `SUM(debit)` d'une écriture réelle sera
/// strictement inférieur à cette borne.
fn decimal_max_safe() -> Decimal {
    // `from_str` est infaillible pour un literal valide — le `expect`
    // ne panique jamais et sert de documentation d'invariant.
    Decimal::from_str("999999999999999.9999").expect("literal decimal constant must parse")
}

/// Paramètres de recherche, tri et pagination pour `list_by_company_paginated`.
#[derive(Debug, Clone)]
pub struct JournalEntryListQuery {
    pub description: Option<String>,
    /// Ne garder que les écritures **touchant ce compte** (issue #374).
    ///
    /// ⚠️ Le scoping multi-tenant reste porté par `company_id` sur
    /// `journal_entries` : `journal_entry_lines` **n'a pas de `company_id`**, et
    /// la sous-requête ne peut donc pas se scoper elle-même. C'est le `WHERE
    /// company_id` de la requête englobante qui l'enferme.
    pub account_id: Option<i64>,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub journal: Option<Journal>,
    pub sort_by: SortBy,
    pub sort_dir: SortDirection,
    pub limit: i64,
    pub offset: i64,
}

impl Default for JournalEntryListQuery {
    fn default() -> Self {
        Self {
            description: None,
            account_id: None,
            amount_min: None,
            amount_max: None,
            date_from: None,
            date_to: None,
            journal: None,
            sort_by: SortBy::default(),
            sort_dir: SortDirection::default(),
            limit: 50,
            offset: 0,
        }
    }
}

/// Résultat paginé retourné par `list_by_company_paginated`.
#[derive(Debug)]
pub struct JournalEntryListResult {
    pub items: Vec<JournalEntryWithLines>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Pousse les clauses WHERE dynamiques dans un `QueryBuilder`.
///
/// **CRITIQUE** : cette fonction doit être appelée sur DEUX `QueryBuilder`
/// DISTINCTS (count + items) — un `QueryBuilder` encode un état mutable et
/// ne peut pas être réutilisé après un `build_*`.
///
/// Préconditions : `qb` vient d'être initialisé avec le SELECT préfixe
/// (ex: `QueryBuilder::new("SELECT COUNT(*) FROM journal_entries")`).
fn push_where_clauses<'a>(
    qb: &mut QueryBuilder<'a, sqlx::MySql>,
    company_id: i64,
    query: &'a JournalEntryListQuery,
) {
    qb.push(" WHERE company_id = ");
    qb.push_bind(company_id);

    if let Some(ref desc) = query.description {
        let trimmed = desc.trim();
        if !trimmed.is_empty() {
            // Story 7-4 / KF-005 : `description` migrée vers FULLTEXT BOOLEAN
            // MODE (prefix wildcard auto-append).
            let escaped = escape_boolean_ft(trimmed);
            if escaped.is_empty() {
                // Edge case (Pass 1 F4) : input non-vide entièrement composé
                // d'opérateurs BOOLEAN MODE strippés (ex. `"+++"`). Pas de
                // colonne LIKE-friendly à fallback (description est la seule
                // colonne search). On force 0 résultats : le pré-refactor
                // `LIKE '%+++%'` retournait 0 par construction, on préserve
                // cette sémantique explicitement (pas de retour de toutes
                // les écritures par skip silencieux).
                qb.push(" AND FALSE");
            } else {
                let bool_query = format!("{escaped}*");
                qb.push(" AND MATCH(description) AGAINST(");
                qb.push_bind(bool_query);
                qb.push(" IN BOOLEAN MODE)");
            }
        }
    }

    if let Some(date_from) = query.date_from {
        qb.push(" AND entry_date >= ");
        qb.push_bind(date_from);
    }

    if let Some(date_to) = query.date_to {
        qb.push(" AND entry_date <= ");
        qb.push_bind(date_to);
    }

    if let Some(journal) = query.journal {
        qb.push(" AND journal = ");
        qb.push_bind(journal);
    }

    // Filtre par compte (#374) — le contournement le plus direct au manque de
    // grand livre, et utile en soi : « montre-moi tout ce qui a touché 1020 ».
    // `EXISTS` plutôt que `IN` : il s'arrête à la première ligne trouvée.
    if let Some(account_id) = query.account_id {
        qb.push(
            " AND EXISTS (SELECT 1 FROM journal_entry_lines jel \
             WHERE jel.entry_id = journal_entries.id AND jel.account_id = ",
        );
        qb.push_bind(account_id);
        qb.push(")");
    }

    // Filtre par plage de montants — sous-requête sur la somme des débits
    // par écriture (en partie double, SUM(debit) == SUM(credit)).
    if query.amount_min.is_some() || query.amount_max.is_some() {
        let min_val = query.amount_min.unwrap_or(Decimal::ZERO);
        let max_val = query.amount_max.unwrap_or_else(decimal_max_safe);
        qb.push(" AND id IN (SELECT entry_id FROM journal_entry_lines GROUP BY entry_id HAVING SUM(debit) BETWEEN ");
        qb.push_bind(min_val);
        qb.push(" AND ");
        qb.push_bind(max_val);
        qb.push(")");
    }
}

/// Liste paginée des écritures d'une company avec filtres et tri.
///
/// Deux queries séquentielles :
/// 1. `SELECT COUNT(*)` avec les filtres (pour le total).
/// 2. `SELECT ... ORDER BY ... LIMIT OFFSET` avec les filtres (pour les items).
///
/// Les lignes sont chargées ensuite via des SELECTs N+1 (acceptable pour
/// `limit <= 500` et un volume PME).
pub async fn list_by_company_paginated(
    pool: &MySqlPool,
    company_id: i64,
    query: JournalEntryListQuery,
) -> Result<JournalEntryListResult, DbError> {
    // Clamp défensif — la source de vérité est le route handler.
    let clamped_limit = query.limit.clamp(1, MAX_LIMIT);
    let clamped_offset = query.offset.max(0);

    // --- Query 1 : count total (deux QueryBuilder distincts, critique) ---
    let mut count_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new("SELECT COUNT(*) FROM journal_entries");
    push_where_clauses(&mut count_qb, company_id, &query);

    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    // --- Query 2 : items paginés ---
    let mut items_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new(format!("SELECT {ENTRY_COLUMNS} FROM journal_entries"));
    push_where_clauses(&mut items_qb, company_id, &query);

    // ORDER BY secondaire stable sur entry_number DESC pour éviter
    // les rangs instables en cas de dates/journaux identiques.
    let sort_col = query.sort_by.as_sql_column();
    let sort_dir_sql = query.sort_dir.as_sql_keyword();
    items_qb.push(format!(
        " ORDER BY {sort_col} {sort_dir_sql}, entry_number DESC LIMIT "
    ));
    items_qb.push_bind(clamped_limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(clamped_offset);

    let entries: Vec<JournalEntry> = items_qb
        .build_query_as::<JournalEntry>()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    // --- Charger les lignes de chaque entry (N+1 acceptable limit ≤ 500) ---
    let mut items = Vec::with_capacity(entries.len());
    for entry in entries {
        let lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
            "SELECT {LINE_COLUMNS} FROM journal_entry_lines \
             WHERE entry_id = ? ORDER BY line_order"
        ))
        .bind(entry.id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
        items.push(JournalEntryWithLines { entry, lines });
    }

    Ok(JournalEntryListResult {
        items,
        total,
        offset: clamped_offset,
        limit: clamped_limit,
    })
}

/// Helper — produit un snapshot JSON d'une écriture (en-tête + lignes)
/// pour l'audit log (`before`/`after`).
///
/// Contient les champs utiles pour une reconstitution partielle :
/// id, entryNumber, entryDate, journal, description, version, lines.
/// Les montants sont sérialisés comme strings (évite erreurs d'arrondi JSON).
fn entry_snapshot_json(entry: &JournalEntry, lines: &[JournalEntryLine]) -> serde_json::Value {
    serde_json::json!({
        "id": entry.id,
        "entryNumber": entry.entry_number,
        "entryDate": entry.entry_date.to_string(),
        "journal": entry.journal.as_str(),
        "description": entry.description,
        "version": entry.version,
        "lines": lines.iter().map(|l| serde_json::json!({
            "lineOrder": l.line_order,
            "accountId": l.account_id,
            "debit": l.debit.to_string(),
            "credit": l.credit.to_string(),
            "projectId": l.project_id,
        })).collect::<Vec<_>>()
    })
}

/// Met à jour une écriture existante avec verrouillage optimiste.
///
/// Stratégie « lock + check applicatif » :
/// 1. `SELECT ... FOR UPDATE` sur l'entry + jointure fiscal_years (exclusif)
/// 2. Check `fy.status == Open` sinon `FiscalYearClosed`
/// 3. Check `version_db == version_param` sinon `OptimisticLockConflict`
/// 4. Check `updated.entry_date` dans `[fy.start_date, fy.end_date]` sinon `DateOutsideFiscalYear`
/// 5. Vérifier tous les comptes actifs et appartenant à la company
/// 6. Snapshot "before" (SELECTs inline dans la tx)
/// 7. DELETE lines + UPDATE header (version += 1) + INSERT new lines
/// 8. Re-check balance applicatif
/// 9. Re-fetch + snapshot "after"
/// 10. INSERT audit_log avec `before`/`after`
/// 11. COMMIT
///
/// Compare l'état persisté (header + lignes) au payload — `true` si aucun
/// champ métier ne diffère (KF-004 : court-circuit no-op pour ne pas bumper
/// version inutilement).
///
/// Comparaison lignes en respectant `line_order` (la sémantique métier
/// d'une écriture comptable dépend de l'ordre — débit puis crédit, etc.).
fn is_no_op_change(
    before_entry: &JournalEntry,
    before_lines: &[JournalEntryLine],
    updated: &NewJournalEntry,
) -> bool {
    if before_entry.entry_date != updated.entry_date
        || before_entry.journal != updated.journal
        || before_entry.description != updated.description
    {
        return false;
    }
    if before_lines.len() != updated.lines.len() {
        return false;
    }
    before_lines.iter().zip(updated.lines.iter()).all(|(b, c)| {
        b.account_id == c.account_id
            && b.debit == c.debit
            && b.credit == c.credit
            && b.project_id == c.project_id
    })
}

/// Règle stricte : `tx.rollback()` explicite avant chaque `return Err`.
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
    updated: NewJournalEntry,
) -> Result<JournalEntryWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Étape 0 : validation des projets analytiques par-ligne (Story 19-2),
    // AVANT le FOR UPDATE sur l'entry+fiscal_year — même ordre de verrouillage
    // global que create_in_tx (companies → projects → fiscal_years, Pattern 5).
    //
    // Grandfathering : les projets DÉJÀ tagués sur cette écriture sont exemptés
    // de la validation — sinon archiver un projet rendrait toute écriture
    // historique non-éditable (409 « projet archivé » sur un simple fix de
    // libellé). Leur existence/company est garantie (FK + scoping à la pose du
    // tag). Le SELECT est scopé company (JOIN journal_entries) pour ne pas
    // offrir de bypass IDOR via l'entry d'une autre company.
    //
    // Portée VOLONTAIREMENT au niveau écriture (pas ligne par ligne) : un
    // projet archivé déjà présent sur l'écriture peut être re-tagué sur une
    // autre ligne de la MÊME écriture. Nécessaire pour corriger une
    // affectation (déplacer le tag de la ligne A vers la ligne B) après
    // archivage — un grandfathering par-ligne strict rendrait ce déplacement
    // impossible. Le périmètre analytique de l'écriture n'en est pas élargi :
    // le projet y figurait déjà. (Code-review 19-2 Pass 1 BH-M1, décision
    // documentée + testée par test_update_moves_archived_tag_between_lines.)
    let prior_project_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT jel.project_id FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? AND jel.project_id IS NOT NULL",
    )
    .bind(id)
    .bind(company_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let line_project_ids: Vec<i64> = updated
        .lines
        .iter()
        .filter_map(|l| l.project_id)
        .filter(|pid| !prior_project_ids.contains(pid))
        .collect();
    if let Err(e) =
        super::projects::validate_taggable_in_tx(&mut tx, company_id, &line_project_ids).await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    // Étape 1 : SELECT FOR UPDATE join fiscal_year.
    #[derive(sqlx::FromRow)]
    struct LockedRow {
        entry_version: i32,
        fy_status: String,
        fy_start: NaiveDate,
        fy_end: NaiveDate,
    }

    let locked: Option<LockedRow> = sqlx::query_as(
        "SELECT je.version AS entry_version, \
                fy.status AS fy_status, \
                fy.start_date AS fy_start, \
                fy.end_date AS fy_end \
         FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? \
         FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let locked = match locked {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(row) => row,
    };

    // Étape 2 : statut fiscal_year.
    if locked.fy_status == "Closed" {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::FiscalYearClosed);
    }

    // Étape 3 : version check applicatif.
    if locked.entry_version != version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    // Étape 4 : date dans l'exercice courant (anti-TOCTOU, M4 passe 1).
    if updated.entry_date < locked.fy_start || updated.entry_date > locked.fy_end {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::DateOutsideFiscalYear);
    }

    // Étape 5 : comptes actifs appartenant à la company + garde de postabilité
    // en saisie manuelle (Story 14-3b). L'update est toujours un flux MANUEL
    // (`enforce_postable = true`).
    if updated.lines.is_empty() {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "NewJournalEntry sans lignes — devait être rejeté en amont".into(),
        ));
    }

    // Grandfather PAR COMPTE (D-A1) : les comptes DÉJÀ référencés par les lignes
    // persistées sont exemptés de la garde de postabilité — sinon un compte
    // devenu non-postable APRÈS création (14-3a : ajouter un sous-compte bascule
    // le parent `postable = FALSE`) rendrait l'écriture historique inéditable
    // (même pour corriger sa seule date). Récupéré AVANT la validation (l'ancien
    // `before_lines` de l'« Étape 6 » était fetché trop tard, cf. spec T1). Le
    // verrou `FOR UPDATE` sur l'en-tête (Étape 1) garantit un snapshot cohérent.
    let exempt_account_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT account_id FROM journal_entry_lines WHERE entry_id = ?",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let account_ids: Vec<i64> = updated.lines.iter().map(|l| l.account_id).collect();
    if let Err(e) =
        validate_lines_accounts_in_tx(&mut tx, company_id, &account_ids, true, &exempt_account_ids)
            .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    // Étape 6 : snapshot "before" (SELECTs inline dans la tx — M2 tranché).
    let before_entry: JournalEntry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let before_lines: Vec<JournalEntryLine> = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let before_json = entry_snapshot_json(&before_entry, &before_lines);

    // KF-004 : court-circuit no-op AVANT le DELETE/UPDATE/INSERT.
    // Tous les guards (FY status, version check, date dans FY, comptes
    // actifs) ont déjà passé — un payload identique avec un état env
    // valide retourne l'entry inchangée. Le verrou `FOR UPDATE` est
    // libéré par le `tx.rollback()` (équivalent à commit côté locks
    // InnoDB pour ce qui est de leur libération).
    // NOTE concurrence (KF-004): grâce à `SELECT ... FOR UPDATE` étape 1,
    // cette fonction n'est PAS exposée à la race REPEATABLE READ décrite
    // dans la spec §race-condition. Les commits parallèles attendent le
    // verrou X-lock, donc le snapshot post-lock est forcément à jour.
    if is_no_op_change(&before_entry, &before_lines, &updated) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(JournalEntryWithLines {
            entry: before_entry,
            lines: before_lines,
        });
    }

    // Étape 7 : DELETE old lines + UPDATE header + INSERT new lines.
    sqlx::query("DELETE FROM journal_entry_lines WHERE entry_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let update_result = sqlx::query(
        "UPDATE journal_entries SET entry_date = ?, journal = ?, description = ?, \
         version = version + 1, updated_at = CURRENT_TIMESTAMP(3) \
         WHERE id = ?",
    )
    .bind(updated.entry_date)
    .bind(updated.journal)
    .bind(&updated.description)
    .bind(id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = update_result {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(map_db_error(e));
    }

    for (idx, line) in updated.lines.iter().enumerate() {
        let line_order = (idx as i32) + 1;
        let insert = sqlx::query(
            "INSERT INTO journal_entry_lines \
             (entry_id, account_id, line_order, debit, credit, project_id) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(line.account_id)
        .bind(line_order)
        .bind(line.debit)
        .bind(line.credit)
        .bind(line.project_id.or(updated.project_id))
        .execute(&mut *tx)
        .await;

        if let Err(e) = insert {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(map_db_error(e));
        }
    }

    // Étape 8 : re-check balance applicatif.
    let row = sqlx::query(
        "SELECT COALESCE(SUM(debit), 0) AS d, COALESCE(SUM(credit), 0) AS c \
         FROM journal_entry_lines WHERE entry_id = ?",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let total_debit: Decimal = row.try_get("d").map_err(map_db_error)?;
    let total_credit: Decimal = row.try_get("c").map_err(map_db_error)?;

    if total_debit != total_credit {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(format!(
            "balance DB incohérente après UPDATE : débit={total_debit}, crédit={total_credit}"
        )));
    }

    // Étape 9 : re-fetch pour le retour + snapshot "after".
    let after_entry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let after_lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let after_json = entry_snapshot_json(&after_entry, &after_lines);

    // Étape 10 : INSERT audit_log dans la même tx.
    let audit_details = serde_json::json!({
        "before": before_json,
        "after": after_json,
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "journal_entry.updated".to_string(),
            "journal_entry".to_string(),
            id,
            Some(audit_details),
        ),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(JournalEntryWithLines {
        entry: after_entry,
        lines: after_lines,
    })
}

/// Supprime une écriture et ses lignes (CASCADE), avec enregistrement
/// audit atomique.
///
/// Étapes :
/// 1. BEGIN tx
/// 2. SELECT FOR UPDATE join fiscal_year (lock entry + FY)
/// 3. Si `Closed` → rollback + `FiscalYearClosed`
/// 4. Snapshot "before" (re-fetch lines)
/// 5. INSERT audit_log (AVANT le DELETE pour préserver la trace)
/// 6. DELETE FROM journal_entries → lignes suivent par CASCADE
/// 7. COMMIT
pub async fn delete_by_id(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    // En cas d'Err, `delete_in_tx` ne rollback pas (n'a qu'un &mut) : `tx` est
    // droppé ici, ce qui déclenche le rollback automatique sqlx.
    delete_in_tx(&mut tx, company_id, id, user_id).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}

/// Variante `_in_tx` de [`delete_by_id`] : exécute les étapes 2-6 (lock FY,
/// garde `Closed`, snapshot, audit, DELETE CASCADE) dans une transaction
/// fournie par l'appelant, **sans** BEGIN/COMMIT. Permet à un autre repo de
/// supprimer l'écriture liée dans la MÊME transaction atomique (ex.
/// `invoices::delete` d'une facture validée — #219).
///
/// N'exécute **pas** de rollback en cas d'erreur (n'a qu'un `&mut` sur la tx) :
/// l'appelant, propriétaire de la transaction, est responsable du rollback
/// (le drop de `Transaction` déclenche le rollback automatique sqlx).
pub(crate) async fn delete_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<(), DbError> {
    // Étape 2 : lock entry + fiscal_year.
    let locked: Option<(i64, String)> = sqlx::query_as(
        "SELECT je.fiscal_year_id, fy.status \
         FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? \
         FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let (_fy_id, fy_status) = match locked {
        None => return Err(DbError::NotFound),
        Some(row) => row,
    };

    // Étape 3 : statut FY.
    if fy_status == "Closed" {
        return Err(DbError::FiscalYearClosed);
    }

    // Étape 3-bis (Story 24-4a, #380) : une écriture CONTRE-PASSÉE ne se
    // supprime plus — la supprimer effacerait la correction, ce que l'art. 958f
    // CO interdit précisément.
    //
    // ⚠️ La FK `RESTRICT` refuserait de toute façon, mais avec une 1451 au
    // message opaque. La garde est ici pour que l'utilisateur lise POURQUOI.
    if reversed_by(&mut **tx, company_id, id).await?.is_some() {
        return Err(DbError::EntryIsReversed);
    }

    // Étape 4 : snapshot avant suppression.
    let before_entry: JournalEntry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let before_lines: Vec<JournalEntryLine> = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let snapshot = entry_snapshot_json(&before_entry, &before_lines);

    // Étape 5 : INSERT audit_log AVANT le DELETE (ordre critique — la
    // trace doit exister avant que la source disparaisse).
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "journal_entry.deleted".to_string(),
            "journal_entry".to_string(),
            id,
            Some(snapshot),
        ),
    )
    .await?;

    // Étape 6 : DELETE (les lignes suivent par CASCADE).
    sqlx::query("DELETE FROM journal_entries WHERE id = ?")
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

    Ok(())
}

/// Story 9-2b T3.2.1 — Liste **toutes** les écritures d'une company sans
/// pagination ni filtre, pour l'export global ZIP (souveraineté).
///
/// Distincte de [`list_by_company_paginated`] (qui charge aussi les lignes par
/// N+1 pour l'UI). Cette fn retourne uniquement les en-têtes ; les lignes sont
/// fournies en un seul JOIN par [`list_all_lines_by_company`] pour éviter le
/// N+1 sur 5 000+ écritures (spec §scope-tables).
///
/// Tri stable `entry_date, id` (cohérent format d'export reproductible).
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<JournalEntry>, DbError> {
    sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries \
         WHERE company_id = ? \
         ORDER BY entry_date, id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Story 9-2b T3.2.2 — Liste **toutes** les lignes d'écriture d'une company
/// via un **single-query JOIN** (anti-N+1).
///
/// `JournalEntryLine` n'a pas de `company_id` direct ground-truth
/// (`entities/journal_entry.rs:144-151`) — le scoping multi-tenant passe par
/// `journal_entries.company_id` via `JOIN`.
///
/// Tri `entry_id, line_order` pour stabilité d'export (les lignes d'une même
/// entry sont contiguës et ordonnées).
pub async fn list_all_lines_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<JournalEntryLine>, DbError> {
    sqlx::query_as::<_, JournalEntryLine>(
        "SELECT jel.id, jel.entry_id, jel.account_id, jel.line_order, jel.debit, jel.credit, \
                jel.project_id \
         FROM journal_entry_lines jel \
         JOIN journal_entries je ON jel.entry_id = je.id \
         WHERE je.company_id = ? \
         ORDER BY jel.entry_id, jel.line_order",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Supprime toutes les écritures d'une company.
///
/// Les lignes suivent par `ON DELETE CASCADE`.
///
/// ⚠️ **Ce doc-comment annonçait « utilisé par `reset_demo` » — c'était faux**, et
/// cette erreur a coûté une passe de revue à la Story 24-4a. `reset_demo`
/// (`kesh-seed`) pose `SET FOREIGN_KEY_CHECKS=0` et fait ses propres `DELETE` en
/// SQL brut ; il n'emprunte pas ce chemin. Les seuls appelants sont les
/// **teardowns des tests** de ce module. Un commentaire périmé coûte ce qu'un
/// compteur faux coûte ailleurs.
///
/// ⛔ **Le `NULL` préalable n'est pas un raffinement, il est nécessaire.**
/// `reverses_entry_id` est une clé étrangère **auto-référente** en `RESTRICT`, et
/// InnoDB vérifie les FK **ligne par ligne, sans différer** : sur un `DELETE`
/// monolithique, si l'origine est traitée avant sa contre-passation, la
/// contrainte déclenche une 1451. Le résultat dépendrait donc de l'ordre de
/// parcours — un échec **intermittent**, ce qui est pire qu'un échec franc.
pub async fn delete_all_by_company(pool: &MySqlPool, company_id: i64) -> Result<u64, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    sqlx::query("UPDATE journal_entries SET reverses_entry_id = NULL WHERE company_id = ?")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;
    let rows = sqlx::query("DELETE FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
    tx.commit().await.map_err(map_db_error)?;
    Ok(rows)
}

/// Recense ce qui **empêche** de contre-passer une écriture (Story 24-4a, #380).
///
/// Rend `None` quand l'écriture est contre-passable, et `Some((motif, id de la
/// pièce))` sinon. ⛔ **Une seule requête** : les sept causes se calculent par
/// sous-requêtes corrélées, jamais par sept allers-retours.
///
/// ⚠️ La **précédence** est celle de l'ordre des tests ci-dessous, et elle est
/// figée : les causes se cumulent, et le champ exposé à l'écran est scalaire.
///
/// ⚠️ **Le compte archivé EST évalué ici**, en dernier de la précédence — parce
/// que l'AC 11 exige que l'écran masque le bouton **avant** le clic. Ne le
/// contrôler qu'à l'écriture ferait afficher un bouton qui échoue.
///
/// Son refus à l'ÉCRITURE reste un **400** qui NOMME les comptes à réactiver
/// ([`DbError::ReversalAccountsArchived`]) ; ce code-ci ne sert que la lecture.
pub async fn reversal_blocker<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<(ReversalBlocker, Option<i64>, Option<String>)>, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    /// Les sept causes de blocage, telles que la requête les rend.
    ///
    /// ⚠️ Une struct nommée plutôt qu'un 7-uplet : `clippy::type_complexity`
    /// refuse le second, et il avait surtout l'inconvénient de rendre l'ordre
    /// des colonnes muet — sept `Option<i64>` d'affilée ne se relisent pas.
    #[derive(sqlx::FromRow)]
    struct BlockerRow {
        reverses_entry_id: Option<i64>,
        reversed_by: Option<i64>,
        invoice_id: Option<i64>,
        invoice_number: Option<String>,
        credit_note_id: Option<i64>,
        credit_note_number: Option<String>,
        supplier_invoice_id: Option<i64>,
        supplier_invoice_number: Option<String>,
        settlement_id: Option<i64>,
        bank_transaction_id: Option<i64>,
        archived_account_number: Option<String>,
    }

    let row: Option<BlockerRow> = sqlx::query_as(
        "SELECT \
           je.reverses_entry_id, \
           (SELECT r.id FROM journal_entries r \
             WHERE r.reverses_entry_id = je.id AND r.company_id = je.company_id LIMIT 1) AS reversed_by, \
           (SELECT i.id FROM invoices i WHERE i.journal_entry_id = je.id LIMIT 1) AS invoice_id, \
           (SELECT i.invoice_number FROM invoices i WHERE i.journal_entry_id = je.id LIMIT 1) AS invoice_number, \
           (SELECT c.id FROM credit_notes c WHERE c.journal_entry_id = je.id LIMIT 1) AS credit_note_id, \
           (SELECT c.credit_note_number FROM credit_notes c WHERE c.journal_entry_id = je.id LIMIT 1) AS credit_note_number, \
           (SELECT s.id FROM supplier_invoices s \
             WHERE s.purchase_journal_entry_id = je.id OR s.settlement_journal_entry_id = je.id \
             LIMIT 1) AS supplier_invoice_id, \
           (SELECT s.supplier_invoice_number FROM supplier_invoices s \
             WHERE s.purchase_journal_entry_id = je.id OR s.settlement_journal_entry_id = je.id \
             LIMIT 1) AS supplier_invoice_number, \
           (SELECT st.id FROM invoice_settlements st WHERE st.journal_entry_id = je.id LIMIT 1) AS settlement_id, \
           (SELECT bt.id FROM bank_transactions bt WHERE bt.matched_entry_id = je.id LIMIT 1) AS bank_transaction_id, \
           (SELECT a.number FROM journal_entry_lines jel \
             JOIN accounts a ON a.id = jel.account_id \
             WHERE jel.entry_id = je.id AND a.active = FALSE \
             ORDER BY a.number LIMIT 1) AS archived_account_number \
         FROM journal_entries je \
         WHERE je.id = ? AND je.company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)?;

    let row = row.ok_or(DbError::NotFound)?;

    // ⚠️ Ordre = précédence. Ne pas réordonner sans réordonner la spec (D6).
    // ⚠️ Le NUMÉRO de la pièce accompagne son identifiant quand il existe : un
    // message qui dit « la facture F-2026-014 » se comprend, un `documentId: 47`
    // ne se comprend pas. Toutes les pièces n'en ont pas — un règlement et une
    // transaction bancaire n'ont que leur identifiant.
    let blocker = if row.reverses_entry_id.is_some() {
        Some((ReversalBlocker::IsAReversal, row.reverses_entry_id, None))
    } else if row.reversed_by.is_some() {
        Some((ReversalBlocker::AlreadyReversed, row.reversed_by, None))
    } else if row.invoice_id.is_some() {
        Some((
            ReversalBlocker::OwnedByInvoice,
            row.invoice_id,
            row.invoice_number,
        ))
    } else if row.credit_note_id.is_some() {
        Some((
            ReversalBlocker::OwnedByCreditNote,
            row.credit_note_id,
            row.credit_note_number,
        ))
    } else if row.supplier_invoice_id.is_some() {
        Some((
            ReversalBlocker::OwnedBySupplierInvoice,
            row.supplier_invoice_id,
            row.supplier_invoice_number,
        ))
    } else if row.settlement_id.is_some() {
        Some((ReversalBlocker::OwnedBySettlement, row.settlement_id, None))
    } else if row.bank_transaction_id.is_some() {
        Some((
            ReversalBlocker::MatchedBankTransaction,
            row.bank_transaction_id,
            None,
        ))
    } else if row.archived_account_number.is_some() {
        // ⛔ En dernier : c'est le seul motif que l'utilisateur peut lever
        // lui-même (réactiver le compte), et l'annoncer avant un motif de
        // propriété ferait croire qu'une facture deviendrait contre-passable.
        //
        // ⚠️ L'étiquette porte le **numéro du compte**, et `document_id` reste
        // `None` : un compte n'est pas une pièce. Sans ce numéro, l'écran dirait
        // « réactivez-LE » sans dire lequel, sur une écriture qui peut porter dix
        // lignes — le refus qui NOMME (le 400 de l'écriture) étant devenu
        // inatteignable depuis que le bouton est masqué. *(Passe 2 de revue.)*
        Some((
            ReversalBlocker::AccountArchived,
            None,
            row.archived_account_number,
        ))
    } else {
        None
    };
    Ok(blocker)
}

/// Rend l'écriture qui **contre-passe** celle-ci, s'il en existe une.
///
/// Dérivé de l'`UNIQUE` — pas de seconde colonne à tenir cohérente (D2).
///
/// ⚠️ **Scopée par `company_id` par DÉFENSE EN PROFONDEUR.** Aujourd'hui une
/// contre-passation naît toujours dans la société de son origine ([`reverse`]
/// reprend le `company_id` de l'appelant), si bien que le filtre est redondant.
/// Mais la sûreté de la lecture reposerait alors entièrement sur un invariant
/// d'ÉCRITURE, que rien n'oblige à tenir — et `reversal_blocker`, elle, filtre.
/// Deux lectures voisines qui ne se scopent pas pareil finissent par diverger.
pub async fn reversed_by<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<i64>, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM journal_entries WHERE reverses_entry_id = ? AND company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Contre-passe une écriture : crée l'écriture **inverse** (Story 24-4a, #380).
///
/// ⛔ **L'écriture d'origine n'est pas touchée** — ni ses lignes, ni sa date, ni
/// son `entry_number`, ni sa `version`. Corriger, en comptabilité, c'est ajouter
/// une écriture, jamais en réécrire une (art. 958f CO, Olico art. 3).
///
/// Séquence, calquée sur `supplier_invoices::cancel` :
/// 1. verrou `FOR UPDATE` sur l'origine + recensement des empêchements ;
/// 2. relecture des lignes **`ORDER BY line_order`** et inversion `D ↔ C` ;
/// 3. exercice **ouvert** couvrant **la date du jour** ;
/// 4. création au journal `OD`, `enforce_postable = false`, projets non re-validés ;
/// 5. audit `journal_entry.reversed`.
///
/// ⚠️ La contre-passation porte la date du **jour**, jamais celle de l'origine :
/// une origine dans un exercice clos serait sinon incorrigible, et dater la
/// correction du jour de l'erreur la rendrait invisible dans la période où elle
/// a été décidée.
pub async fn reverse(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<JournalEntryWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (1) Verrou sur l'origine, puis recensement des empêchements.
        let origin: Option<(i64, i64, String)> = sqlx::query_as(
            "SELECT je.id, je.entry_number, fy.name \
             FROM journal_entries je \
             JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
             WHERE je.id = ? AND je.company_id = ? FOR UPDATE",
        )
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;
        let (_, entry_number, origin_fy_name) = origin.ok_or(DbError::NotFound)?;

        // ⛔ **`AccountArchived` est le seul motif que l'ÉCRITURE ne traite pas
        // ici.** Le recensement le rend pour que l'écran masque le bouton avant
        // le clic (AC 11) ; mais s'arrêter dessus produirait un 409 muet, là où
        // l'écriture doit rendre un **400 qui NOMME les comptes à réactiver**
        // (étape 3 ci-dessous). Un refus qui ne dit pas quel compte n'est pas
        // utilisable.
        match reversal_blocker(&mut *tx, company_id, id).await? {
            Some((ReversalBlocker::AccountArchived, _, _)) | None => {}
            Some((blocker, document_id, document_label)) => {
                return Err(DbError::EntryNotReversable {
                    blocker,
                    document_id,
                    document_label,
                });
            }
        }

        // (2) Lignes de l'origine, dans l'ordre CONTRACTUEL, inversées D ↔ C.
        //
        // ⛔ `ORDER BY line_order` et non `id` : `uq_jel_entry_order` fait de
        // `line_order` la position, et le `project_id` se reprend POSITIONNELLEMENT
        // — une écriture manuelle porte un tag par ligne (Story 19-2), là où le
        // gabarit `cancel` n'a qu'un projet document-level à propager.
        let origin_lines: Vec<(i64, Decimal, Decimal, Option<i64>)> = sqlx::query_as(
            "SELECT account_id, debit, credit, project_id FROM journal_entry_lines \
             WHERE entry_id = ? ORDER BY line_order",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;
        if origin_lines.is_empty() {
            return Err(DbError::Invariant(
                "écriture sans ligne : rien à contre-passer".into(),
            ));
        }

        // (3) Comptes archivés depuis — refus qui NOMME, cf. `ReversalAccountsArchived`.
        let account_ids: Vec<i64> = origin_lines.iter().map(|(a, _, _, _)| *a).collect();
        let archived = archived_accounts_in_tx(&mut tx, company_id, &account_ids).await?;
        if !archived.is_empty() {
            return Err(DbError::ReversalAccountsArchived(archived));
        }

        let reversal_lines: Vec<NewJournalEntryLine> = origin_lines
            .iter()
            .map(
                |(account_id, debit, credit, project_id)| NewJournalEntryLine {
                    account_id: *account_id,
                    debit: *credit,
                    credit: *debit,
                    project_id: *project_id,
                },
            )
            .collect();

        // (4) Exercice ouvert du JOUR.
        let today = Utc::now().date_naive();
        let fy = super::fiscal_years::find_open_covering_date(&mut tx, company_id, today)
            .await?
            .ok_or(DbError::FiscalYearInvalid)?;

        // Le nom de l'exercice DE L'ORIGINE lève l'ambiguïté du numéro, qui
        // REPART À 1 à chaque exercice. Celui de la contre-passation
        // n'apprendrait rien.
        let description = if fy.name == origin_fy_name {
            format!("Contre-passation écriture n° {entry_number}")
        } else {
            format!("Contre-passation écriture n° {entry_number} ({origin_fy_name})")
        };

        let created = create_in_tx_inner(
            &mut tx,
            fy.id,
            user_id,
            NewJournalEntry {
                company_id,
                entry_date: today,
                journal: Journal::OD,
                description,
                // ⛔ `None` au niveau document : les projets sont repris PAR LIGNE.
                project_id: None,
                lines: reversal_lines,
            },
            // Les comptes viennent de l'origine : exiger la postabilité rendrait
            // l'écriture incorrigible à cause d'un changement de config postérieur.
            false,
            // Idem pour les projets — les tags sont COPIÉS, pas choisis.
            false,
            Some(id),
        )
        .await?;

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "journal_entry.reversed".to_string(),
                "journal_entry".to_string(),
                id,
                Some(serde_json::json!({
                    "reversalJournalEntryId": created.entry.id,
                })),
            ),
        )
        .await?;

        Ok(created)
    }
    .await;

    match result {
        Ok(created) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(created)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Les comptes de `account_ids` qui sont **archivés** (`active = FALSE`).
///
/// ⛔ Existe parce que `enforce_postable = false` NE lève PAS la garde
/// `active = TRUE` de [`validate_accounts`], qui est inconditionnelle.
async fn archived_accounts_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    account_ids: &[i64],
) -> Result<Vec<ArchivedAccount>, DbError> {
    if account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = account_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT id, number FROM accounts \
         WHERE company_id = ? AND active = FALSE AND id IN ({placeholders}) \
         ORDER BY number"
    );
    let mut q = sqlx::query_as::<_, (i64, String)>(&sql).bind(company_id);
    for id in account_ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(&mut **tx).await.map_err(map_db_error)?;
    Ok(rows
        .into_iter()
        .map(|(account_id, account_number)| ArchivedAccount {
            account_id,
            account_number: Some(account_number),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{NewJournalEntry, NewJournalEntryLine};
    use crate::repositories::{accounts, fiscal_years};
    use chrono::{Datelike, NaiveDate};
    use kesh_core::accounting::Journal as CoreJournal;
    use rust_decimal_macros::dec;

    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    /// Nettoie les écritures de test puis retourne
    /// `(company_id, fiscal_year_id, admin_user_id)`.
    ///
    /// Story 3.5 : le `admin_user_id` est récupéré ici et propagé aux
    /// tests pour satisfaire la nouvelle signature de `create` qui
    /// requiert un `user_id` pour l'audit log.
    async fn setup(pool: &MySqlPool) -> (i64, i64, i64) {
        // Récupérer la première company (créée par seed_demo).
        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one company in DB for tests (run seed-demo)");

        // Nettoyer les écritures existantes pour éviter les interférences.
        delete_all_by_company(pool, company_id).await.unwrap();

        // Garantir un exercice OUVERT couvrant aujourd'hui (auto-réparation #140).
        let today = chrono::Utc::now().naive_utc().date();
        let fy = ensure_open_fiscal_year(pool, company_id, today).await;

        // Récupérer l'admin user pour l'audit log (dupliqué depuis
        // audit_log::tests story 3.3 — voir spec 3.5 Dev Notes L1).
        let admin_user_id: i64 =
            sqlx::query_scalar("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
                .fetch_one(pool)
                .await
                .expect("need at least one admin user (run seed-demo or bootstrap)");

        (company_id, fy.id, admin_user_id)
    }

    /// Garantit qu'un exercice fiscal **ouvert** couvre `date` et le retourne.
    ///
    /// Auto-réparation #140 : la suite dépend d'un exercice ouvert couvrant le
    /// jour J, récupéré via `find_covering_date`. Or un run précédent peut
    /// laisser l'exercice clos (p. ex. `update_no_op_in_closed_fy_*` clôt sans
    /// recréer), tout comme une clôture manuelle en base dev pendant le
    /// dogfooding. Dans ces cas, `setup` récupérait un exercice `Closed` et tous
    /// les tests suivants échouaient en `FiscalYearClosed`. On rétablit donc un
    /// exercice ouvert : si l'exercice couvrant est clos, on le supprime (après
    /// purge des écritures pour respecter la FK) et on en recrée un ouvert pour
    /// l'année calendaire ; si aucun exercice ne couvre la date, on en crée un.
    /// Même pattern delete+recreate que `test_create_rejects_closed_fiscal_year`.
    async fn ensure_open_fiscal_year(
        pool: &MySqlPool,
        company_id: i64,
        date: NaiveDate,
    ) -> crate::entities::FiscalYear {
        use crate::entities::FiscalYearStatus;

        if let Some(fy) = fiscal_years::find_covering_date(pool, company_id, date)
            .await
            .expect("find_covering_date")
        {
            if fy.status == FiscalYearStatus::Open {
                return fy;
            }
            // Exercice couvrant mais clos → purge des écritures puis suppression
            // (un exercice clos ne peut pas être rouvert par politique métier).
            delete_all_by_company(pool, company_id).await.unwrap();
            sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
                .bind(fy.id)
                .execute(pool)
                .await
                .expect("delete closed fiscal year");
        }

        let year = date.year();
        fiscal_years::create_for_seed(
            pool,
            crate::entities::NewFiscalYear {
                company_id,
                name: format!("Exercice {year}"),
                start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
            },
        )
        .await
        .expect("create open fiscal year for tests")
    }

    /// Récupère 2 comptes actifs pour les tests (premier actif puis un autre).
    async fn two_accounts(pool: &MySqlPool, company_id: i64) -> (i64, i64) {
        let accs = accounts::list_by_company(pool, company_id, false)
            .await
            .unwrap();
        assert!(accs.len() >= 2, "need ≥ 2 active accounts (run seed-demo)");
        (accs[0].id, accs[1].id)
    }

    fn mk_entry(
        company_id: i64,
        date: NaiveDate,
        lines: Vec<NewJournalEntryLine>,
    ) -> NewJournalEntry {
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal: CoreJournal::Banque.into(),
            description: "Test entry".to_string(),
            project_id: None,
            lines,
        }
    }

    /// Story 3.5 — vérifie que `create` insère bien une entrée `audit_log`
    /// avec `action = "journal_entry.created"` et un `details_json`
    /// contenant le snapshot direct (PAS de wrapper `{before, after}`).
    #[tokio::test]
    async fn test_create_writes_audit_log() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(42),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(42),
                    project_id: None,
                },
            ],
        );
        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();

        let audit_entries = audit_log::find_by_entity(&pool, "journal_entry", created.entry.id, 10)
            .await
            .unwrap();

        let created_audit = audit_entries
            .iter()
            .find(|e| e.action == "journal_entry.created")
            .expect("audit entry with action journal_entry.created must exist");

        assert_eq!(created_audit.user_id, admin_user_id);
        assert_eq!(created_audit.entity_type, "journal_entry");
        assert_eq!(created_audit.entity_id, created.entry.id);

        let details = created_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");

        // Convention projet : snapshot direct (pas de wrapper {before, after}).
        assert!(
            details.get("before").is_none(),
            "create audit must NOT wrap in {{before, after}} — expected direct snapshot"
        );
        assert!(
            details.get("after").is_none(),
            "create audit must NOT wrap in {{before, after}} — expected direct snapshot"
        );

        // Le snapshot doit contenir les champs clés de l'écriture.
        assert_eq!(
            details.get("description").and_then(|v| v.as_str()),
            Some("Test entry")
        );
        let lines = details
            .get("lines")
            .and_then(|v| v.as_array())
            .expect("lines array must be present in snapshot");
        assert_eq!(lines.len(), 2);
    }

    #[tokio::test]
    async fn test_create_balanced_entry() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;

        let today = chrono::Utc::now().naive_utc().date();
        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(100),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(100),
                    project_id: None,
                },
            ],
        );

        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();
        assert_eq!(created.entry.entry_number, 1);
        assert_eq!(created.lines.len(), 2);
        assert_eq!(created.lines[0].line_order, 1);
        assert_eq!(created.lines[1].line_order, 2);
        assert_eq!(created.lines[0].debit, dec!(100));
        assert_eq!(created.lines[1].credit, dec!(100));
    }

    #[tokio::test]
    async fn test_create_sequential_numbering() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        for expected in 1..=3 {
            let new = mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(50),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(50),
                        project_id: None,
                    },
                ],
            );
            let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();
            assert_eq!(created.entry.entry_number, expected);
        }
    }

    #[tokio::test]
    async fn test_create_rejects_closed_fiscal_year() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Clore l'exercice (Story 3.7 : signature audit-aware avec user_id +
        // company_id pour défense en profondeur multi-tenant — Code Review F2).
        fiscal_years::close(&pool, admin_user_id, company_id, fy_id)
            .await
            .unwrap();

        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(100),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(100),
                    project_id: None,
                },
            ],
        );

        let result = create(&pool, fy_id, admin_user_id, new).await;
        assert!(
            matches!(result, Err(DbError::FiscalYearClosed)),
            "expected FiscalYearClosed, got {:?}",
            result
        );

        // Nettoyer : impossible de rouvrir un exercice clos — on doit
        // supprimer et recréer. Passer par SQL direct pour ce test.
        // P13 : supprimer d'abord les éventuelles écritures référençant
        // cet exercice pour éviter un échec FK RESTRICT si un test
        // concurrent en a inséré (garde-fou défensif).
        delete_all_by_company(&pool, company_id).await.unwrap();
        sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
            .bind(fy_id)
            .execute(&pool)
            .await
            .unwrap();

        // Recréer pour les tests suivants (Story 3.7 : pas d'audit log, contexte test).
        let year = chrono::Utc::now().naive_utc().date().year();
        fiscal_years::create_for_seed(
            &pool,
            crate::entities::NewFiscalYear {
                company_id,
                name: format!("Exercice {year}"),
                start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_find_covering_date_none() {
        let pool = test_pool().await;
        let (company_id, _fy_id, _admin_user_id) = setup(&pool).await;

        // Date très ancienne — aucun exercice ne la couvre.
        let old = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let result = fiscal_years::find_covering_date(&pool, company_id, old)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_covering_date_open() {
        let pool = test_pool().await;
        let (company_id, fy_id, _admin_user_id) = setup(&pool).await;
        let today = chrono::Utc::now().naive_utc().date();

        let result = fiscal_years::find_covering_date(&pool, company_id, today)
            .await
            .unwrap();
        let fy = result.expect("fiscal year should cover today");
        assert_eq!(fy.id, fy_id);
    }

    #[tokio::test]
    async fn test_find_by_id_returns_lines_in_order() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(30),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(20),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(50),
                    project_id: None,
                },
            ],
        );

        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();
        let fetched = find_by_id(&pool, company_id, created.entry.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.lines.len(), 3);
        assert_eq!(fetched.lines[0].line_order, 1);
        assert_eq!(fetched.lines[1].line_order, 2);
        assert_eq!(fetched.lines[2].line_order, 3);
    }

    #[tokio::test]
    async fn test_list_paginated_default() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 3 écritures à la même date.
        for _ in 0..3 {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: dec!(10),
                            credit: dec!(0),
                            project_id: None,
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: dec!(10),
                            project_id: None,
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        let result = list_by_company_paginated(&pool, company_id, JournalEntryListQuery::default())
            .await
            .unwrap();

        assert!(result.items.len() >= 3);
        assert!(result.total >= 3);
        assert_eq!(result.offset, 0);
        assert_eq!(result.limit, 50);
        // Tri par entry_number DESC à date égale (secondary sort stable).
        assert!(result.items[0].entry.entry_number > result.items[1].entry.entry_number);
    }

    #[tokio::test]
    async fn test_list_filter_description() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 2 écritures avec descriptions distinctes.
        let mut entry1 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(100),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(100),
                    project_id: None,
                },
            ],
        );
        entry1.description = "Facture fournisseur ABC".to_string();
        create(&pool, fy_id, admin_user_id, entry1).await.unwrap();

        let mut entry2 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(50),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(50),
                    project_id: None,
                },
            ],
        );
        entry2.description = "Virement bancaire XYZ".to_string();
        create(&pool, fy_id, admin_user_id, entry2).await.unwrap();

        // Filtre par « facture ».
        let query = JournalEntryListQuery {
            description: Some("facture".to_string()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 1);
        assert!(result.items[0].entry.description.contains("Facture"));

        // Filtre par « virement ».
        let query = JournalEntryListQuery {
            description: Some("virement".to_string()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 1);
        assert!(result.items[0].entry.description.contains("Virement"));
    }

    /// Story 7-4 / KF-005 / T9.4 — adapté du précédent
    /// `test_list_filter_description_escapes_percent` qui vérifiait
    /// l'échappement applicatif du `%` dans la clause LIKE.
    ///
    /// Sémantique nouvelle (BOOLEAN MODE) : `%` n'est PAS dans la
    /// strip-list de `escape_boolean_ft` (10 caractères opérateurs
    /// uniquement). Il est traité comme caractère **non-token** par le
    /// tokenizer InnoDB FULLTEXT, donc silencieusement ignoré. Le test
    /// vérifie que (i) la query `MATCH AGAINST '50%*'` passe sans erreur
    /// SQL et (ii) le row `"Remise 50% client"` est trouvé via le token
    /// `50` extrait par le tokenizer (puis match `50*` du prefix wildcard).
    /// Le seed a été ajusté pour que la 2e écriture ne contienne aucun
    /// token préfixé par `50` (sinon BOOLEAN MODE matcherait aussi).
    #[tokio::test]
    async fn test_list_filter_description_handles_special_chars() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let mut e1 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(10),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(10),
                    project_id: None,
                },
            ],
        );
        // Utilise `500` (3 chars, ≥ innodb_ft_min_token_size par défaut)
        // pour que le tokenizer indexe le token. `50` (2 chars) serait
        // filtré et la recherche retournerait 0.
        e1.description = "Remise 500% client".to_string();
        create(&pool, fy_id, admin_user_id, e1).await.unwrap();

        let mut e2 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(20),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(20),
                    project_id: None,
                },
            ],
        );
        // Pas de token préfixé `500` pour ne pas être capté par `500*`.
        e2.description = "Achat fournisseur ABC".to_string();
        create(&pool, fy_id, admin_user_id, e2).await.unwrap();

        // Recherche « 500% » : le `%` est un non-token côté InnoDB
        // FULLTEXT. La query `MATCH AGAINST '500%*' IN BOOLEAN MODE`
        // tokenize en `500` + prefix wildcard → matche
        // `"Remise 500% client"` via le token `500` mais pas
        // `"Achat fournisseur ABC"`.
        let query = JournalEntryListQuery {
            description: Some("500%".to_string()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(
            result.total, 1,
            "le `%` doit être traité comme non-token par InnoDB FULLTEXT et ne matcher que le token `500`"
        );
        assert!(result.items[0].entry.description.contains("500%"));
    }

    /// Régression Pass 1 F4 : un input non-vide entièrement composé
    /// d'opérateurs BOOLEAN MODE (ex. `"+++"`) doit retourner 0 résultats,
    /// PAS la totalité du journal (skip silencieux pré-patch).
    #[tokio::test]
    async fn test_filter_by_description_pure_operators_returns_zero() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Seed 2 écritures — si le skip silencieux régressait, on les
        // retrouverait toutes au lieu d'avoir 0 résultats.
        for desc in ["TestEntry Alpha", "TestEntry Beta"] {
            let mut entry = mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(1),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(1),
                        project_id: None,
                    },
                ],
            );
            entry.description = desc.to_string();
            create(&pool, fy_id, admin_user_id, entry).await.unwrap();
        }

        for gibberish in ["+++", "***", "()()", "~~~"] {
            let result = list_by_company_paginated(
                &pool,
                company_id,
                JournalEntryListQuery {
                    description: Some(gibberish.to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(
                result.total, 0,
                "input pure-opérateurs `{gibberish}` doit retourner 0, pas tout le journal"
            );
            assert!(result.items.is_empty());
        }
    }

    /// Régression detector inversé pour KF-005 v0.1 : asserte que la
    /// recherche par fragment-mid-word est PERDUE en BOOLEAN MODE +
    /// prefix wildcard. Si une future migration MariaDB ajoute le suffix
    /// wildcard support, ou si Kesh migre vers Sphinx/Manticore (v0.3+),
    /// OU si la config `innodb_ft_min_token_size=1` est appliquée, ce
    /// test FAILERA et devra être inversé pour asserter le nouveau
    /// comportement (match attendu).
    #[tokio::test]
    async fn test_search_no_longer_matches_mid_word() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let mut entry = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(10),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(10),
                    project_id: None,
                },
            ],
        );
        entry.description = "TestSalaire Mensuel".to_string();
        create(&pool, fy_id, admin_user_id, entry).await.unwrap();

        // « alaire » fragment mid-word de « TestSalaire » → 0 résultat.
        let mid = list_by_company_paginated(
            &pool,
            company_id,
            JournalEntryListQuery {
                description: Some("alaire".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(
            mid.total, 0,
            "régression mid-word search documentée : `alaire` ne doit plus matcher `TestSalaire`"
        );

        // Préfixe `testsal` matche bien.
        let prefix = list_by_company_paginated(
            &pool,
            company_id,
            JournalEntryListQuery {
                description: Some("testsal".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            prefix
                .items
                .iter()
                .any(|e| e.entry.description.contains("TestSalaire")),
            "préfixe `testsal` doit matcher `TestSalaire` via FULLTEXT prefix wildcard"
        );
    }

    #[tokio::test]
    async fn test_list_filter_amount_range() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // 3 écritures à 100, 500, 1000.
        for amount in [dec!(100), dec!(500), dec!(1000)] {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: amount,
                            credit: dec!(0),
                            project_id: None,
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: amount,
                            project_id: None,
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        // Filtre [200, 800] — doit retourner uniquement 500.
        let query = JournalEntryListQuery {
            amount_min: Some(dec!(200)),
            amount_max: Some(dec!(800)),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 1);
    }

    #[tokio::test]
    async fn test_list_filter_journal() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 2 écritures Banque + 1 Ventes.
        for _ in 0..2 {
            let mut e = mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(10),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(10),
                        project_id: None,
                    },
                ],
            );
            e.journal = CoreJournal::Banque.into();
            create(&pool, fy_id, admin_user_id, e).await.unwrap();
        }
        let mut ventes = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(20),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(20),
                    project_id: None,
                },
            ],
        );
        ventes.journal = CoreJournal::Ventes.into();
        create(&pool, fy_id, admin_user_id, ventes).await.unwrap();

        // Filtre Banque → 2 écritures.
        let query = JournalEntryListQuery {
            journal: Some(CoreJournal::Banque.into()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 2);
    }

    #[tokio::test]
    async fn test_list_pagination_offset_limit() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 5 écritures.
        for _ in 0..5 {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: dec!(10),
                            credit: dec!(0),
                            project_id: None,
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: dec!(10),
                            project_id: None,
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        // Page 1 : limit=2, offset=0.
        let page1 = list_by_company_paginated(
            &pool,
            company_id,
            JournalEntryListQuery {
                limit: 2,
                offset: 0,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(page1.items.len(), 2);
        assert_eq!(page1.total, 5);
        assert_eq!(page1.limit, 2);
        assert_eq!(page1.offset, 0);

        // Page 2 : limit=2, offset=2.
        let page2 = list_by_company_paginated(
            &pool,
            company_id,
            JournalEntryListQuery {
                limit: 2,
                offset: 2,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(page2.items.len(), 2);
        assert_eq!(page2.offset, 2);

        // Les écritures ne se chevauchent pas.
        let page1_ids: Vec<i64> = page1.items.iter().map(|i| i.entry.id).collect();
        let page2_ids: Vec<i64> = page2.items.iter().map(|i| i.entry.id).collect();
        for id in &page2_ids {
            assert!(!page1_ids.contains(id), "pages se chevauchent");
        }
    }

    #[tokio::test]
    async fn test_list_sort_by_entry_number_asc() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        for _ in 0..3 {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: dec!(10),
                            credit: dec!(0),
                            project_id: None,
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: dec!(10),
                            project_id: None,
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        let query = JournalEntryListQuery {
            sort_by: SortBy::EntryNumber,
            sort_dir: SortDirection::Asc,
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert!(result.items.len() >= 3);
        // Tri ascendant : 1, 2, 3...
        for i in 0..result.items.len() - 1 {
            assert!(
                result.items[i].entry.entry_number <= result.items[i + 1].entry.entry_number,
                "Tri ascendant cassé"
            );
        }
    }

    /// Filtre par compte (#374) — le contournement direct au manque de grand
    /// livre.
    ///
    /// ⚠️ Ce test travaille sur la **base partagée**, où d'autres tests ont déjà
    /// mouvementé `a1` et `a2`. Il ne peut donc pas compter en absolu : il crée
    /// un **compte neuf** et vérifie que le filtre rend exactement ce qui le
    /// touche — et rien de ce qui ne le touche pas.
    #[tokio::test]
    async fn test_list_filter_by_account() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Un compte que ce test est seul à mouvementer.
        let suffixe = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
        let neuf = accounts::create(
            &pool,
            admin_user_id,
            crate::entities::account::NewAccount {
                company_id,
                number: format!("9{}", suffixe % 900_000 + 100_000),
                name: "Compte du test de filtre".to_string(),
                account_type: crate::entities::AccountType::Expense,
                parent_id: None,
                role: None,
                postable: true,
            },
        )
        .await
        .unwrap();

        // Une écriture qui le touche, une qui ne le touche pas.
        let mut touche = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: neuf.id,
                    debit: dec!(42),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(42),
                    project_id: None,
                },
            ],
        );
        touche.description = "Touche le compte neuf".to_string();
        let attendue = create(&pool, fy_id, admin_user_id, touche).await.unwrap();

        let mut ailleurs = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(7),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(7),
                    project_id: None,
                },
            ],
        );
        ailleurs.description = "Ne le touche pas".to_string();
        create(&pool, fy_id, admin_user_id, ailleurs).await.unwrap();

        let result = list_by_company_paginated(
            &pool,
            company_id,
            JournalEntryListQuery {
                account_id: Some(neuf.id),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(result.total, 1, "une seule écriture touche ce compte");
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].entry.id, attendue.entry.id);

        // ⚠️ Le total doit être celui du FILTRE, pas celui de la company : un
        // `COUNT(*)` qui oublierait la clause rendrait une pagination fausse
        // sans qu'aucune page ne paraisse anormale.
        let sans_filtre =
            list_by_company_paginated(&pool, company_id, JournalEntryListQuery::default())
                .await
                .unwrap();
        assert!(
            sans_filtre.total > result.total,
            "le filtre doit réduire le total ({} vs {})",
            sans_filtre.total,
            result.total
        );
    }

    #[tokio::test]
    async fn test_list_count_accurate_after_filter() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 3 écritures : 2 matchent le filtre, 1 non.
        for desc in ["Match 1", "Match 2", "Autre"] {
            let mut e = mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(10),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(10),
                        project_id: None,
                    },
                ],
            );
            e.description = desc.to_string();
            create(&pool, fy_id, admin_user_id, e).await.unwrap();
        }

        let query = JournalEntryListQuery {
            description: Some("Match".to_string()),
            limit: 1, // limit petit pour forcer la pagination
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        // Total doit refléter TOUTES les matches, pas seulement la page.
        assert_eq!(result.total, 2);
        assert_eq!(result.items.len(), 1);
    }

    #[tokio::test]
    async fn test_check_constraint_rejects_debit_and_credit_same_line() {
        let pool = test_pool().await;
        let (company_id, _fy_id, admin_user_id) = setup(&pool).await;
        let (a1, _a2) = two_accounts(&pool, company_id).await;

        // Créer d'abord une entry valide pour récupérer un entry_id.
        // On va ensuite tenter un INSERT direct d'une ligne invalide.
        let today = chrono::Utc::now().naive_utc().date();
        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(10),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(0),
                    credit: dec!(10),
                    project_id: None,
                },
            ],
        );
        let created = create(&pool, _fy_id, admin_user_id, new).await.unwrap();

        // Tentative d'INSERT direct d'une ligne avec debit > 0 ET credit > 0.
        let direct_result = sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, 99, 5, 5)",
        )
        .bind(created.entry.id)
        .bind(a1)
        .execute(&pool)
        .await;

        assert!(direct_result.is_err(), "CHECK constraint should reject");
        let err = map_db_error(direct_result.unwrap_err());
        assert!(
            matches!(err, DbError::CheckConstraintViolation(_)),
            "expected CheckConstraintViolation, got {:?}",
            err
        );
    }

    /// KF-004 : payload identique (header + lignes même ordre + comptes
    /// toujours actifs + FY ouvert) → pas de bump version, `updated_at`
    /// inchangé, mêmes IDs DB pour les lignes (pas de DELETE+INSERT),
    /// pas d'audit_log `journal_entry.updated`.
    #[tokio::test]
    async fn update_no_op_returns_unchanged_entity_no_lines_churn() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(100),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(100),
                        project_id: None,
                    },
                ],
            ),
        )
        .await
        .unwrap();
        let version_initial = created.entry.version;
        let updated_at_initial = created.entry.updated_at;
        let line_ids_initial: Vec<i64> = created.lines.iter().map(|l| l.id).collect();

        // Payload strictement identique reconstruit depuis les `before` lines.
        let identical = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            project_id: None,
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                    project_id: None,
                })
                .collect(),
        };

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            version_initial,
            admin_user_id,
            identical,
        )
        .await
        .unwrap();

        assert_eq!(result.entry.version, version_initial);
        assert_eq!(result.entry.updated_at, updated_at_initial);
        let line_ids_after: Vec<i64> = result.lines.iter().map(|l| l.id).collect();
        assert_eq!(
            line_ids_after, line_ids_initial,
            "no-op : pas de DELETE+INSERT, IDs lignes identiques"
        );

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'journal_entry' AND entity_id = ? AND action = 'journal_entry.updated'",
        )
        .bind(created.entry.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 0);
    }

    /// KF-004 : si l'exercice est clôturé entre la création et l'update no-op,
    /// le check `FiscalYearClosed` rejette AVANT le no-op check (pas de leak).
    #[tokio::test]
    async fn update_no_op_in_closed_fy_returns_fiscal_year_closed() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(50),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(50),
                        project_id: None,
                    },
                ],
            ),
        )
        .await
        .unwrap();

        fiscal_years::close(&pool, admin_user_id, company_id, fy_id)
            .await
            .unwrap();

        let identical = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            project_id: None,
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                    project_id: None,
                })
                .collect(),
        };

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            created.entry.version,
            admin_user_id,
            identical,
        )
        .await;
        assert!(
            matches!(result, Err(DbError::FiscalYearClosed)),
            "expected FiscalYearClosed, got {:?}",
            result
        );

        // Nettoyage (cf. test_create_rejects_closed_fiscal_year).
        delete_all_by_company(&pool, company_id).await.unwrap();
        sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
            .bind(fy_id)
            .execute(&pool)
            .await
            .unwrap();
        let year = chrono::Utc::now().naive_utc().date().year();
        fiscal_years::create_for_seed(
            &pool,
            crate::entities::NewFiscalYear {
                company_id,
                name: format!("Exercice {year}"),
                start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
            },
        )
        .await
        .unwrap();
    }

    /// KF-004 : si un compte référencé par l'écriture a été archivé entre la
    /// création et l'update no-op, le check d'intégrité rejette AVANT le no-op
    /// check (pas de leak via no-op).
    #[tokio::test]
    async fn update_no_op_with_inactive_account_returns_inactive_error() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(75),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(75),
                        project_id: None,
                    },
                ],
            ),
        )
        .await
        .unwrap();

        // Archiver a1 directement en SQL (la fonction archive() exige de ne
        // pas avoir de sous-comptes ; on évite cette vérification ici car
        // elle est orthogonale au scope du test).
        sqlx::query("UPDATE accounts SET active = FALSE, version = version + 1 WHERE id = ?")
            .bind(a1)
            .execute(&pool)
            .await
            .unwrap();

        let identical = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            project_id: None,
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                    project_id: None,
                })
                .collect(),
        };

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            created.entry.version,
            admin_user_id,
            identical,
        )
        .await;
        assert!(
            matches!(result, Err(DbError::InactiveOrInvalidAccounts)),
            "expected InactiveOrInvalidAccounts, got {:?}",
            result
        );

        // Réactiver le compte pour les tests suivants.
        sqlx::query("UPDATE accounts SET active = TRUE WHERE id = ?")
            .bind(a1)
            .execute(&pool)
            .await
            .unwrap();
        delete_all_by_company(&pool, company_id).await.unwrap();
    }

    /// KF-004 régression : modifier la `description` → bump version.
    #[tokio::test]
    async fn update_partial_change_bumps_version() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(33),
                        credit: dec!(0),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(33),
                        project_id: None,
                    },
                ],
            ),
        )
        .await
        .unwrap();
        let version_initial = created.entry.version;

        let mut payload = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            project_id: None,
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                    project_id: None,
                })
                .collect(),
        };
        payload.description = "Description modifiée".into();

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            version_initial,
            admin_user_id,
            payload,
        )
        .await
        .unwrap();
        assert_eq!(result.entry.version, version_initial + 1);
        assert_eq!(result.entry.description, "Description modifiée");
    }

    // -----------------------------------------------------------------------
    // Story 19-2 — tag analytique par-ligne (écritures manuelles)
    // -----------------------------------------------------------------------

    /// Insère un projet analytique et retourne son id (calque le helper 19-3
    /// de `tests/supplier_invoices_repository.rs`).
    ///
    /// Idempotent inter-runs : la DB de test est partagée et `setup()` ne purge
    /// que les écritures — un projet laissé par un run précédent violerait
    /// l'unicité `(company_id, code)`. On supprime d'abord le reliquat (sans
    /// FK restante : les lignes d'écriture ont été purgées par `setup()`).
    async fn mk_project(pool: &MySqlPool, company_id: i64, code: &str, archived: bool) -> i64 {
        sqlx::query("DELETE FROM projects WHERE company_id = ? AND code = ?")
            .bind(company_id)
            .bind(code)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO projects (company_id, code, name, archived, version) \
             VALUES (?, ?, ?, ?, 0)",
        )
        .bind(company_id)
        .bind(code)
        .bind(code)
        .bind(archived)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    fn line(account_id: i64, debit: Decimal, credit: Decimal) -> NewJournalEntryLine {
        NewJournalEntryLine {
            account_id,
            debit,
            credit,
            project_id: None,
        }
    }

    fn tagged_line(
        account_id: i64,
        debit: Decimal,
        credit: Decimal,
        project_id: i64,
    ) -> NewJournalEntryLine {
        NewJournalEntryLine {
            account_id,
            debit,
            credit,
            project_id: Some(project_id),
        }
    }

    /// AC15 — tags par-ligne mixtes (2 projets distincts + 1 ligne sans tag)
    /// persistés et relus tels quels par `find_by_id` (LINE_COLUMNS).
    #[tokio::test]
    async fn test_create_line_projects_mixed() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();
        let p1 = mk_project(&pool, company_id, "P19-2-MIX-A", false).await;
        let p2 = mk_project(&pool, company_id, "P19-2-MIX-B", false).await;

        let new = mk_entry(
            company_id,
            today,
            vec![
                tagged_line(a1, dec!(30), dec!(0), p1),
                tagged_line(a1, dec!(20), dec!(0), p2),
                line(a2, dec!(0), dec!(50)),
            ],
        );
        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();

        let read = find_by_id(&pool, company_id, created.entry.id)
            .await
            .unwrap()
            .expect("entry must exist");
        let tags: Vec<Option<i64>> = read.lines.iter().map(|l| l.project_id).collect();
        assert_eq!(tags, vec![Some(p1), Some(p2), None]);
    }

    /// AC15 — un projet archivé sur une ligne est rejeté (`IllegalStateTransition`).
    #[tokio::test]
    async fn test_create_line_project_archived_rejected() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();
        let archived = mk_project(&pool, company_id, "P19-2-ARCH", true).await;

        let new = mk_entry(
            company_id,
            today,
            vec![
                tagged_line(a1, dec!(10), dec!(0), archived),
                line(a2, dec!(0), dec!(10)),
            ],
        );
        let err = create(&pool, fy_id, admin_user_id, new).await.unwrap_err();
        assert!(
            matches!(err, DbError::IllegalStateTransition(_)),
            "expected IllegalStateTransition, got {err:?}"
        );
    }

    /// AC15 — projet inexistant → `NotFound` (aucune écriture créée).
    #[tokio::test]
    async fn test_create_line_project_unknown_rejected() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let new = mk_entry(
            company_id,
            today,
            vec![
                tagged_line(a1, dec!(10), dec!(0), 999_999_999),
                line(a2, dec!(0), dec!(10)),
            ],
        );
        let err = create(&pool, fy_id, admin_user_id, new).await.unwrap_err();
        assert!(matches!(err, DbError::NotFound), "got {err:?}");
    }

    /// AC15 — projet d'une AUTRE company → `NotFound` (IDOR-safe, scoping
    /// `company_id` dans `validate_taggable_in_tx`).
    #[tokio::test]
    async fn test_create_line_project_cross_company_rejected() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Company étrangère éphémère + projet actif chez elle.
        let other_company: i64 = sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES ('Cross 19-2', 'Rue Test 1', 'Independant', 'FR', 'FR')",
        )
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        let foreign_project = mk_project(&pool, other_company, "P19-2-FOREIGN", false).await;

        let new = mk_entry(
            company_id,
            today,
            vec![
                tagged_line(a1, dec!(10), dec!(0), foreign_project),
                line(a2, dec!(0), dec!(10)),
            ],
        );
        let err = create(&pool, fy_id, admin_user_id, new).await.unwrap_err();
        assert!(matches!(err, DbError::NotFound), "got {err:?}");

        // Cleanup de la company éphémère (le projet suit par FK... non : pas de
        // CASCADE — suppression explicite du projet d'abord).
        sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(foreign_project)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM companies WHERE id = ?")
            .bind(other_company)
            .execute(&pool)
            .await
            .unwrap();
    }

    /// AC15 — fallback document-level 19-3 préservé : lignes sans tag +
    /// `new.project_id` Some → toutes les lignes stampées (`or()`).
    #[tokio::test]
    async fn test_create_entry_level_fallback_stamps_lines() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();
        let p = mk_project(&pool, company_id, "P19-2-DOC", false).await;

        let mut new = mk_entry(
            company_id,
            today,
            vec![line(a1, dec!(42), dec!(0)), line(a2, dec!(0), dec!(42))],
        );
        new.project_id = Some(p);
        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();

        assert!(
            created.lines.iter().all(|l| l.project_id == Some(p)),
            "toutes les lignes doivent porter le tag document-level, got {:?}",
            created
                .lines
                .iter()
                .map(|l| l.project_id)
                .collect::<Vec<_>>()
        );
    }

    /// AC15 — un update qui ne change QUE le projet d'une ligne n'est PAS un
    /// no-op : version bumpée et tag persisté (garde `is_no_op_change`).
    #[tokio::test]
    async fn test_update_project_only_change_is_not_noop() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();
        let p = mk_project(&pool, company_id, "P19-2-NOOP", false).await;

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![line(a1, dec!(10), dec!(0)), line(a2, dec!(0), dec!(10))],
            ),
        )
        .await
        .unwrap();
        let v0 = created.entry.version;

        // Même payload, seul le project_id de la 1re ligne change.
        let payload = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            project_id: None,
            lines: vec![
                tagged_line(a1, dec!(10), dec!(0), p),
                line(a2, dec!(0), dec!(10)),
            ],
        };
        let updated = update(
            &pool,
            company_id,
            created.entry.id,
            v0,
            admin_user_id,
            payload,
        )
        .await
        .unwrap();

        assert_eq!(
            updated.entry.version,
            v0 + 1,
            "changement de projet ≠ no-op"
        );
        assert_eq!(updated.lines[0].project_id, Some(p));
        assert_eq!(updated.lines[1].project_id, None);
    }

    /// AC15/DC2 — grandfathering : un tag DÉJÀ présent sur l'écriture reste
    /// éditable après archivage du projet (fix de libellé possible), mais un
    /// NOUVEAU projet archivé est refusé.
    #[tokio::test]
    async fn test_update_grandfathers_preexisting_archived_project() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();
        let p = mk_project(&pool, company_id, "P19-2-GRAND", false).await;
        let q = mk_project(&pool, company_id, "P19-2-GRAND-Q", true).await;

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    tagged_line(a1, dec!(10), dec!(0), p),
                    line(a2, dec!(0), dec!(10)),
                ],
            ),
        )
        .await
        .unwrap();

        // Archiver P après la pose du tag.
        sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
            .bind(p)
            .execute(&pool)
            .await
            .unwrap();

        // (a) Édition du libellé en conservant le tag P archivé → OK.
        let payload = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: "Libellé corrigé (grandfathering)".to_string(),
            project_id: None,
            lines: vec![
                tagged_line(a1, dec!(10), dec!(0), p),
                line(a2, dec!(0), dec!(10)),
            ],
        };
        let updated = update(
            &pool,
            company_id,
            created.entry.id,
            created.entry.version,
            admin_user_id,
            payload,
        )
        .await
        .expect("le tag pré-existant archivé doit être toléré à l'édition");
        assert_eq!(updated.lines[0].project_id, Some(p));

        // (b) Ajouter un AUTRE projet archivé (Q, jamais tagué ici) → refus.
        let payload = NewJournalEntry {
            company_id,
            entry_date: updated.entry.entry_date,
            journal: updated.entry.journal,
            description: updated.entry.description.clone(),
            project_id: None,
            lines: vec![
                tagged_line(a1, dec!(10), dec!(0), p),
                tagged_line(a2, dec!(0), dec!(10), q),
            ],
        };
        let err = update(
            &pool,
            company_id,
            updated.entry.id,
            updated.entry.version,
            admin_user_id,
            payload,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, DbError::IllegalStateTransition(_)),
            "nouveau projet archivé doit être refusé, got {err:?}"
        );
    }

    /// Review Pass 1 BH-M1 — la portée écriture du grandfathering permet de
    /// DÉPLACER un tag archivé d'une ligne à l'autre de la même écriture
    /// (correction d'affectation post-archivage). Un grandfathering par-ligne
    /// strict rendrait ce déplacement impossible.
    #[tokio::test]
    async fn test_update_moves_archived_tag_between_lines() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();
        let p = mk_project(&pool, company_id, "P19-2-MOVE", false).await;

        // Ligne 1 taguée P, ligne 2 vierge.
        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    tagged_line(a1, dec!(10), dec!(0), p),
                    line(a2, dec!(0), dec!(10)),
                ],
            ),
        )
        .await
        .unwrap();

        sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
            .bind(p)
            .execute(&pool)
            .await
            .unwrap();

        // Déplacer le tag : ligne 1 détaguée, ligne 2 taguée P (archivé mais
        // déjà présent sur l'écriture → exempté).
        let payload = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            project_id: None,
            lines: vec![
                line(a1, dec!(10), dec!(0)),
                tagged_line(a2, dec!(0), dec!(10), p),
            ],
        };
        let updated = update(
            &pool,
            company_id,
            created.entry.id,
            created.entry.version,
            admin_user_id,
            payload,
        )
        .await
        .expect("déplacer un tag archivé au sein de la même écriture doit passer");
        assert_eq!(updated.lines[0].project_id, None);
        assert_eq!(updated.lines[1].project_id, Some(p));
    }

    // ─── Story 14-3b : garde de postabilité à la saisie manuelle (chantier A) ───

    /// Bascule `postable` d'un compte directement en SQL — plus simple que de
    /// créer un sous-compte pour déclencher la règle `is_postable`. Restauré par
    /// le caller (base de test partagée entre `#[tokio::test]`).
    async fn set_postable(pool: &MySqlPool, account_id: i64, postable: bool) {
        sqlx::query("UPDATE accounts SET postable = ? WHERE id = ?")
            .bind(postable)
            .bind(account_id)
            .execute(pool)
            .await
            .unwrap();
    }

    /// Trois comptes actifs distincts issus du seed.
    async fn three_accounts(pool: &MySqlPool, company_id: i64) -> (i64, i64, i64) {
        let accs = accounts::list_by_company(pool, company_id, false)
            .await
            .unwrap();
        assert!(accs.len() >= 3, "need ≥ 3 active accounts (run seed-demo)");
        (accs[0].id, accs[1].id, accs[2].id)
    }

    /// AC-A / D-A0 — `create` pool-level (SAISIE MANUELLE, `enforce_postable =
    /// true`) : accepte une écriture sur des comptes postables, refuse une ligne
    /// visant un compte non-postable en `InactiveOrInvalidAccounts`.
    #[tokio::test]
    async fn test_create_manual_rejects_non_postable_line() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Cas OK — comptes postables (état seed).
        let ok = mk_entry(
            company_id,
            today,
            vec![line(a1, dec!(10), dec!(0)), line(a2, dec!(0), dec!(10))],
        );
        create(&pool, fy_id, admin_user_id, ok)
            .await
            .expect("écriture manuelle sur comptes postables acceptée");
        delete_all_by_company(&pool, company_id).await.unwrap();

        // Cas KO — a2 devient non-postable.
        set_postable(&pool, a2, false).await;
        let ko = mk_entry(
            company_id,
            today,
            vec![line(a1, dec!(10), dec!(0)), line(a2, dec!(0), dec!(10))],
        );
        let result = create(&pool, fy_id, admin_user_id, ko).await;
        set_postable(&pool, a2, true).await; // restaurer AVANT l'assert (base partagée)
        assert!(
            matches!(result, Err(DbError::InactiveOrInvalidAccounts)),
            "compte non-postable en saisie manuelle → InactiveOrInvalidAccounts, obtenu {:?}",
            result
        );
        delete_all_by_company(&pool, company_id).await.unwrap();
    }

    /// AC-A / D-A0 (non-régression) — un flux automatique (`create_in_tx` avec
    /// `enforce_postable = false`) poste SANS erreur sur un compte non-postable.
    /// Reproduit le cas « facture dont le compte produit est devenu non-postable ».
    #[tokio::test]
    async fn test_create_in_tx_auto_flow_allows_non_postable() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        set_postable(&pool, a2, false).await;
        let new = mk_entry(
            company_id,
            today,
            vec![line(a1, dec!(10), dec!(0)), line(a2, dec!(0), dec!(10))],
        );
        let mut tx = pool.begin().await.unwrap();
        let committed = create_in_tx(&mut tx, fy_id, admin_user_id, new, false)
            .await
            .is_ok();
        if committed {
            tx.commit().await.unwrap();
        } else {
            let _ = tx.rollback().await;
        }
        set_postable(&pool, a2, true).await;
        assert!(
            committed,
            "flux automatique (enforce_postable=false) doit accepter un compte non-postable"
        );
        delete_all_by_company(&pool, company_id).await.unwrap();
    }

    /// AC-A / D-A1 (grandfather PAR COMPTE + brèche L3) — une écriture manuelle
    /// sur un compte X qui devient non-postable APRÈS coup reste éditable tant
    /// qu'on ne référence pas un NOUVEAU compte non-postable :
    /// - éditer sans changer les comptes → OK (grandfather) ;
    /// - ajouter une 2e ligne sur X (déjà référencé, non-postable) → TOLÉRÉ (L3) ;
    /// - ajouter une ligne vers Y (non-postable, jamais référencé) → REJET.
    #[tokio::test]
    async fn test_update_grandfathers_non_postable_by_account() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2, a3) = three_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Écriture initiale (a1 débit / a2 crédit), tous postables.
        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![line(a1, dec!(50), dec!(0)), line(a2, dec!(0), dec!(50))],
            ),
        )
        .await
        .unwrap();

        // a1 devient non-postable APRÈS coup.
        set_postable(&pool, a1, false).await;

        // (a) Éditer le seul libellé (mêmes comptes) → OK, a1 grandfathered.
        let mut edit = mk_entry(
            company_id,
            today,
            vec![line(a1, dec!(50), dec!(0)), line(a2, dec!(0), dec!(50))],
        );
        edit.description = "Édition libellé".to_string();
        let edited = update(
            &pool,
            company_id,
            created.entry.id,
            created.entry.version,
            admin_user_id,
            edit,
        )
        .await
        .expect("grandfather : édition sans toucher au compte non-postable doit passer");

        // (b) L3 TOLÉRÉ : ajouter une 2e ligne sur a1 (déjà référencé) → OK.
        let tolerated = mk_entry(
            company_id,
            today,
            vec![
                line(a1, dec!(30), dec!(0)),
                line(a1, dec!(20), dec!(0)),
                line(a2, dec!(0), dec!(50)),
            ],
        );
        let after_tol = update(
            &pool,
            company_id,
            edited.entry.id,
            edited.entry.version,
            admin_user_id,
            tolerated,
        )
        .await
        .expect("L3 : ajout d'une ligne sur un compte non-postable DÉJÀ référencé est toléré");

        // (c) REJET : ajouter une ligne vers a3 (non-postable) jamais référencé.
        set_postable(&pool, a3, false).await;
        let bad = mk_entry(
            company_id,
            today,
            vec![
                line(a1, dec!(50), dec!(0)),
                line(a2, dec!(0), dec!(30)),
                line(a3, dec!(0), dec!(20)),
            ],
        );
        let result = update(
            &pool,
            company_id,
            after_tol.entry.id,
            after_tol.entry.version,
            admin_user_id,
            bad,
        )
        .await;
        set_postable(&pool, a1, true).await; // restaurer avant asserts
        set_postable(&pool, a3, true).await;
        assert!(
            matches!(result, Err(DbError::InactiveOrInvalidAccounts)),
            "ajout d'une ligne vers un compte non-postable jamais référencé → rejet, obtenu {:?}",
            result
        );
        delete_all_by_company(&pool, company_id).await.unwrap();
    }

    /// AC-E — le compte de résultat (`CurrentYearResult`, non-postable par
    /// construction `is_postable`) est refusé à la saisie manuelle.
    #[tokio::test]
    async fn test_create_manual_rejects_result_account() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, _a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let result_account_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM accounts \
             WHERE company_id = ? AND role = 'CurrentYearResult' AND active = TRUE LIMIT 1",
        )
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
        let Some(rid) = result_account_id else {
            // Plan sans compte de résultat annoté → rien à vérifier (les 3 charts
            // standards en portent un, cf. 2979).
            return;
        };
        let postable: bool = sqlx::query_scalar("SELECT postable FROM accounts WHERE id = ?")
            .bind(rid)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(
            !postable,
            "le compte de résultat doit être non-postable (14-3a)"
        );

        let new = mk_entry(
            company_id,
            today,
            vec![line(a1, dec!(10), dec!(0)), line(rid, dec!(0), dec!(10))],
        );
        let res = create(&pool, fy_id, admin_user_id, new).await;
        assert!(
            matches!(res, Err(DbError::InactiveOrInvalidAccounts)),
            "saisie manuelle sur le compte de résultat doit être rejetée, obtenu {:?}",
            res
        );
        delete_all_by_company(&pool, company_id).await.unwrap();
    }
}
