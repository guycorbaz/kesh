//! Repository pour `FiscalYear`.
//!
//! **Pas de `delete`** : conformément au Code des obligations suisse
//! (art. 957-964), les exercices comptables ne sont jamais supprimés.
//! Les seules transitions autorisées sont `Open` → `Closed` via
//! `close`. La ré-ouverture d'un exercice clos n'est **pas** autorisée
//! au niveau du repository (garde-fou DB).
//!
//! ## Story 3.7 — Lock ordering & audit
//!
//! Les fns mutatrices [`create`], [`update_name`], [`close`] ouvrent leur
//! propre transaction interne et auditent via `audit_log::insert_in_tx`.
//! Aucune chaîne de locks cross-table : `fiscal_years` est isolé.
//! [`find_open_covering_date`] reste utilisé par `invoices::validate_invoice`
//! qui acquiert d'abord le lock sur `invoices` puis sur `fiscal_years`
//! (Pattern 5).
//!
//! Le pré-check `find_overlapping` + `find_by_name` (FOR UPDATE) au sein
//! de `create` distingue les deux UNIQUE constraints existantes
//! (`uq_fiscal_years_company_name` et `uq_fiscal_years_company_start_date`)
//! et couvre le cas d'overlap d'intervalles fermés non détecté par les
//! contraintes DB.

use chrono::NaiveDate;
use serde_json::json;
use sqlx::mysql::MySqlPool;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{FiscalYear, NewFiscalYear};
use crate::errors::{DbError, map_db_error};
use crate::repositories::MAX_LIST_LIMIT;
use crate::repositories::audit_log;

const FIND_BY_ID_SQL: &str = "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
     FROM fiscal_years WHERE id = ?";

// Story 3.7 P3-M3 : list_by_company trié `start_date DESC` (le plus récent en
// tête), cohérent avec l'AC #1 de la page UI.
const LIST_BY_COMPANY_SQL: &str = "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
     FROM fiscal_years WHERE company_id = ? ORDER BY start_date DESC LIMIT ?";

// --- Story 3.7 — clés Invariant namespacées (Pass 2 HP2-M4) ---

/// Pré-check `create` a détecté un chevauchement avec un exercice existant.
pub const FY_OVERLAP_KEY: &str = "fiscal_year:overlap";

/// Pré-check `create` ou `update_name` a détecté un nom déjà utilisé sur la
/// même company (autre row).
pub const FY_NAME_DUPLICATE_KEY: &str = "fiscal_year:name-duplicate";

/// `update_name` a reçu un nom vide (après trim).
pub const FY_NAME_EMPTY_KEY: &str = "fiscal_year:name-empty";

/// `create` ou `update_name` a reçu un nom dépassant `FY_NAME_MAX_LEN`
/// caractères (longueur de la colonne DB `VARCHAR(50)`). Story 3.7 Code
/// Review Pass 1 F3 — pré-validé applicativement pour renvoyer un 400
/// `VALIDATION_ERROR` au lieu d'un 500 `DataTooLong`.
pub const FY_NAME_TOO_LONG_KEY: &str = "fiscal_year:name-too-long";

/// Longueur maximale du nom d'un exercice (colonne DB `name VARCHAR(50)`).
pub const FY_NAME_MAX_LEN: usize = 50;

/// Construit le snapshot JSON utilisé par les entrées d'audit log.
fn snapshot_json(fy: &FiscalYear) -> serde_json::Value {
    json!({
        "id": fy.id,
        "companyId": fy.company_id,
        "name": fy.name,
        "startDate": fy.start_date,
        "endDate": fy.end_date,
        "status": fy.status.as_str(),
        "createdAt": fy.created_at,
        "updatedAt": fy.updated_at,
    })
}

/// Construit l'entrée d'audit log à partir d'un snapshot direct ou d'un
/// wrapper `{before, after}`.
fn build_audit_entry(
    user_id: i64,
    action: &str,
    entity_id: i64,
    details: serde_json::Value,
) -> NewAuditLogEntry {
    NewAuditLogEntry {
        user_id,
        action: action.to_string(),
        entity_type: "fiscal_year".to_string(),
        entity_id,
        details_json: Some(details),
    }
}

// --- Création ---

/// Crée un nouvel exercice comptable avec audit log.
///
/// Algorithme (Story 3.7 Pass 1 H-5 + H-6) :
/// 1. `tx = pool.begin()`
/// 2. `find_overlapping FOR UPDATE` → `Invariant(FY_OVERLAP_KEY)` si chevauchement
/// 3. `find_by_name FOR UPDATE` → `Invariant(FY_NAME_DUPLICATE_KEY)` si nom déjà pris
/// 4. INSERT
/// 5. INSERT audit_log avec snapshot direct
/// 6. COMMIT
///
/// Les contraintes DB (`uq_fiscal_years_company_name`,
/// `uq_fiscal_years_company_start_date`, `chk_fiscal_years_dates`) restent
/// un filet de sécurité : sous race extrême, elles renvoient
/// `UniqueConstraintViolation` ou `CheckConstraintViolation` que le caller
/// remappe en messages génériques.
pub async fn create(
    pool: &MySqlPool,
    user_id: i64,
    new: NewFiscalYear,
) -> Result<FiscalYear, DbError> {
    // Story 3.7 Code Review Pass 1 F3 — pré-validation applicative de la
    // longueur du nom (VARCHAR(50) en DB) pour renvoyer un message
    // `VALIDATION_ERROR` au lieu d'un 500 `Data too long for column`.
    if new.name.chars().count() > FY_NAME_MAX_LEN {
        return Err(DbError::Invariant(FY_NAME_TOO_LONG_KEY.to_string()));
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Pré-check 1 : overlap d'intervalles fermés.
    if find_overlapping(&mut tx, new.company_id, new.start_date, new.end_date)
        .await?
        .is_some()
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(FY_OVERLAP_KEY.to_string()));
    }

    // Pré-check 2 : nom déjà utilisé sur cette company.
    if find_by_name(&mut tx, new.company_id, &new.name)
        .await?
        .is_some()
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(FY_NAME_DUPLICATE_KEY.to_string()));
    }

    let id = insert_fiscal_year_in_tx(&mut tx, &new).await?;

    let fy = fetch_fiscal_year_in_tx(&mut tx, id).await?;

    // Audit log : snapshot direct (1 état).
    audit_log::insert_in_tx(
        &mut tx,
        build_audit_entry(user_id, "fiscal_year.created", fy.id, snapshot_json(&fy)),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(fy)
}

/// Variante de `create` pour le seed démo : pas d'audit log, pas de `user_id`.
///
/// Story 3.7 T1.8 — cohérent avec la décision story 3.5 sur
/// `bulk_create_from_chart` : le contexte système (seed) ne génère pas
/// d'entrée d'audit. La tx interne fait toujours les pré-checks d'overlap
/// et de nom pour respecter les UNIQUE constraints même en seed.
pub async fn create_for_seed(pool: &MySqlPool, new: NewFiscalYear) -> Result<FiscalYear, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    if find_overlapping(&mut tx, new.company_id, new.start_date, new.end_date)
        .await?
        .is_some()
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(FY_OVERLAP_KEY.to_string()));
    }

    if find_by_name(&mut tx, new.company_id, &new.name)
        .await?
        .is_some()
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(FY_NAME_DUPLICATE_KEY.to_string()));
    }

    let id = insert_fiscal_year_in_tx(&mut tx, &new).await?;
    let fy = fetch_fiscal_year_in_tx(&mut tx, id).await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(fy)
}

/// Crée un fiscal_year dans la transaction du caller **uniquement si aucun
/// exercice n'existe déjà pour cette company**.
///
/// Story 3.7 Pass 1 H-4 + Pass 2 HP2-H1 — utilisé par
/// `routes/onboarding::finalize` pour auto-créer un exercice à la fin du
/// flow Path B sans risque TOCTOU.
///
/// L'INSERT atomique `INSERT … SELECT … FROM dual WHERE NOT EXISTS` exécuté
/// dans la même transaction que la sous-requête garantit qu'aucun doublon
/// n'est créé sous finalize concurrent.
///
/// - Si `rows_affected == 1` → INSERT effectif → audit log inséré → `Some(fy)`.
/// - Si `rows_affected == 0` → un fiscal_year existait déjà → idempotent → `None`.
///
/// Le helper EST responsable de l'audit log (cohérent avec la décision story 3.5).
pub async fn create_if_absent_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    new: NewFiscalYear,
) -> Result<Option<FiscalYear>, DbError> {
    // Pass 2 HP2-H1 : `FROM dual` requis pour portabilité MariaDB.
    //
    // Story 3.7 Code Review Pass 1 F1 — sous MariaDB REPEATABLE READ, deux
    // finalize concurrents peuvent tous deux passer le `WHERE NOT EXISTS`
    // (chaque tx voit le même snapshot pré-INSERT). Le second commit frappe
    // alors `uq_fiscal_years_company_name` ou `uq_fiscal_years_company_start_date`
    // → on catch ces UniqueConstraintViolation pour rester idempotent
    // (`Ok(None)` au lieu de remonter une 500 au client).
    let rows_affected = match sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         SELECT ?, ?, ?, ?, 'Open' FROM dual \
         WHERE NOT EXISTS (SELECT 1 FROM fiscal_years WHERE company_id = ?)",
    )
    .bind(new.company_id)
    .bind(&new.name)
    .bind(new.start_date)
    .bind(new.end_date)
    .bind(new.company_id)
    .execute(&mut **tx)
    .await
    {
        Ok(result) => result.rows_affected(),
        Err(err) => match map_db_error(err) {
            DbError::UniqueConstraintViolation(_) => return Ok(None),
            other => return Err(other),
        },
    };

    if rows_affected == 0 {
        return Ok(None);
    }

    // Le INSERT a réussi : retrouver l'id par (company_id, name) — name est
    // unique par company.
    let id: i64 =
        sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ? AND name = ?")
            .bind(new.company_id)
            .bind(&new.name)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?;

    let fy = sqlx::query_as::<_, FiscalYear>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    audit_log::insert_in_tx(
        tx,
        build_audit_entry(user_id, "fiscal_year.created", fy.id, snapshot_json(&fy)),
    )
    .await?;

    Ok(Some(fy))
}

/// Renomme un exercice (le seul champ mutable). Audit log avec wrapper
/// `{before, after}`.
///
/// Story 3.7 Pass 1 H-7 — pas d'optimistic version (pas de colonne `version`
/// sur `fiscal_years` en v0.1). Le SELECT FOR UPDATE fige le before-snapshot
/// dans la même transaction que l'UPDATE.
///
/// Renommage **autorisé sur exercices Closed** (Pass 1 M-3) — le CO art.
/// 957-964 protège l'intégrité des montants/dates, pas un libellé descriptif.
///
/// Story 3.7 Code Review Pass 1 F2 — `company_id` ajouté à la signature et
/// au SQL `WHERE` pour défense en profondeur multi-tenant. Le handler reste
/// censé pré-vérifier via `find_by_id_in_company` (404 anti-énumération),
/// mais le repo lui-même refuse désormais toute mutation cross-tenant.
pub async fn update_name(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    id: i64,
    new_name: String,
) -> Result<FiscalYear, DbError> {
    let trimmed_name = new_name.trim().to_string();
    if trimmed_name.is_empty() {
        return Err(DbError::Invariant(FY_NAME_EMPTY_KEY.to_string()));
    }
    // Story 3.7 Code Review Pass 1 F3 — pré-validation longueur (VARCHAR(50)).
    if trimmed_name.chars().count() > FY_NAME_MAX_LEN {
        return Err(DbError::Invariant(FY_NAME_TOO_LONG_KEY.to_string()));
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // SELECT FOR UPDATE scopé `(id, company_id)` — fige le before-snapshot
    // ET empêche toute mutation cross-tenant si le pre-check du handler est
    // contourné (Code Review Pass 1 F2).
    let before_opt = sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
         FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let before = match before_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(fy) => fy,
    };

    // Pré-check duplicate name (autre row même company) pour distinguer cette
    // erreur de l'overlap. On exclut la row courante (renommer en son propre
    // nom = no-op autorisé).
    let conflict_opt: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM fiscal_years \
         WHERE company_id = ? AND name = ? AND id <> ? \
         FOR UPDATE",
    )
    .bind(company_id)
    .bind(&trimmed_name)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    if conflict_opt.is_some() {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(FY_NAME_DUPLICATE_KEY.to_string()));
    }

    sqlx::query("UPDATE fiscal_years SET name = ? WHERE id = ? AND company_id = ?")
        .bind(&trimmed_name)
        .bind(id)
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let after = fetch_fiscal_year_in_tx(&mut tx, id).await?;

    let audit_details = json!({
        "before": snapshot_json(&before),
        "after": snapshot_json(&after),
    });
    audit_log::insert_in_tx(
        &mut tx,
        build_audit_entry(user_id, "fiscal_year.updated", id, audit_details),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Retrouve un exercice par son id (sans scope multi-tenant).
///
/// Conservé pour compatibilité avec les call sites internes au crate
/// (ex. `kesh-seed` qui n'a pas de notion de current_user). Les routes API
/// doivent utiliser [`find_by_id_in_company`] pour le scope multi-tenant.
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Retrouve un exercice par id **scopé à une company** (Story 3.7 Pass 1 H-3).
///
/// Pattern multi-tenant Anti-Pattern 4 fix : query scopée directement au lieu
/// du fetch-then-check côté handler.
///
/// Retourne `None` si l'exercice n'existe pas OU appartient à une autre
/// company (réponse 404 anti-énumération côté handler).
pub async fn find_by_id_in_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
         FROM fiscal_years WHERE id = ? AND company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Retourne l'exercice (ouvert OU clos) qui couvre une date donnée pour
/// une company, ou `None` si aucun exercice ne correspond.
///
/// **Lock-free** : cette fonction est utilisée comme pré-check côté
/// route handler pour distinguer les erreurs `NO_FISCAL_YEAR` et
/// `FISCAL_YEAR_CLOSED`. Le vrai lock contre les clôtures concurrentes
/// est repris dans `journal_entries::create` via `SELECT ... FOR UPDATE`.
///
/// Code Review Pass 1 F14 — ordre déterministe : si plusieurs exercices
/// couvrent la même date (cas anormal — bypass DB ou dette historique),
/// on priorise `Open` puis `start_date` le plus récent. Sous les UNIQUE
/// constraints actuelles ce cas ne peut survenir mais l'ordre déterministe
/// est une défense en profondeur cheap.
pub async fn find_covering_date(
    pool: &MySqlPool,
    company_id: i64,
    date: NaiveDate,
) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
         FROM fiscal_years \
         WHERE company_id = ? AND start_date <= ? AND end_date >= ? \
         ORDER BY status DESC, start_date DESC \
         LIMIT 1",
    )
    .bind(company_id)
    .bind(date)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Retourne l'exercice **ouvert** qui couvre une date donnée, avec lock
/// `SELECT ... FOR UPDATE` — utilisé dans une transaction multi-étapes
/// (ex. `invoices::validate_invoice` Story 5.2) pour empêcher une
/// clôture concurrente entre le check et la fin de la transaction.
///
/// Distinct de [`find_covering_date`] : (a) filtre `status = 'Open'`
/// (ignorent les exercices clos), (b) prend un `Transaction` au lieu
/// d'un `Pool` (doit s'exécuter dans la transaction métier du caller),
/// (c) applique `FOR UPDATE` sur la row trouvée.
///
/// Ordre des locks (Story 5.2 section Concurrence) : `fiscal_years`
/// s'acquiert **après** `invoices` et **avant** `invoice_number_sequences`
/// et `journal_entries`. Toute divergence = risque de deadlock avec
/// `journal_entries::create_in_tx` en cours sur la même company.
pub async fn find_open_covering_date(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    date: NaiveDate,
) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
         FROM fiscal_years \
         WHERE company_id = ? AND start_date <= ? AND end_date >= ? AND status = 'Open' \
         LIMIT 1 FOR UPDATE",
    )
    .bind(company_id)
    .bind(date)
    .bind(date)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Retourne le premier exercice de la company qui chevauche l'intervalle
/// `[start_date, end_date]` (intervalles fermés), ou `None`.
///
/// Story 3.7 Pass 1 H-6 — couvre les cas non détectés par les UNIQUE
/// constraints DB (ex. exercice Jan-Dec 2027 + tentative Jul 2027-Jun 2028 :
/// `start_date` différents mais intervalles chevauchants).
///
/// Doit être appelé dans une transaction caller — applique `FOR UPDATE`
/// pour empêcher une création concurrente entre le check et l'INSERT.
///
/// **Algèbre du chevauchement** (Code Review Pass 1 F4) : deux intervalles
/// fermés `[E.start, E.end]` (existant) et `[N.start, N.end]` (nouveau)
/// se chevauchent ssi `E.start <= N.end ∧ E.end >= N.start`. Les binds
/// ci-dessous appliquent cette formule — l'ordre `(end_date, start_date)`
/// est délibéré et NE doit PAS être réordonné pour suivre l'ordre des
/// paramètres de la fonction sous peine de casser la détection.
pub async fn find_overlapping(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
         FROM fiscal_years \
         WHERE company_id = ? AND start_date <= ? AND end_date >= ? \
         LIMIT 1 FOR UPDATE",
    )
    .bind(company_id)
    // start_date <= ?  → bind end_date du nouvel exercice (E.start <= N.end)
    .bind(end_date)
    // end_date   >= ?  → bind start_date du nouvel exercice (E.end >= N.start)
    .bind(start_date)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Retourne l'exercice qui porte ce nom dans la company, ou `None`.
///
/// Story 3.7 Pass 1 H-5 — distingue le cas « nom déjà utilisé » du cas
/// « overlap » avant l'INSERT, pour fournir un message i18n précis.
pub async fn find_by_name(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    name: &str,
) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
         FROM fiscal_years \
         WHERE company_id = ? AND name = ? \
         LIMIT 1 FOR UPDATE",
    )
    .bind(company_id)
    .bind(name)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Liste les exercices d'une company, **triés par date de début décroissante**
/// (le plus récent en tête). Story 3.7 Pass 2 HP2-M8.
///
/// Limité à `MAX_LIST_LIMIT` exercices. Une entreprise a typiquement moins
/// de 100 exercices sur toute sa durée de vie, donc la limite n'est pas
/// atteignable en pratique mais garantit une borne défensive contre les OOM.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(LIST_BY_COMPANY_SQL)
        .bind(company_id)
        .bind(MAX_LIST_LIMIT)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

/// Clôture un exercice (`Open` → `Closed`) avec audit log.
///
/// Transaction atomique avec guard SQL `WHERE status = 'Open'`. Conformément
/// au CO suisse (art. 957-964), un exercice clos ne peut JAMAIS être
/// rouvert via cette API.
///
/// Story 3.7 Code Review Pass 1 F2 — `company_id` ajouté à la signature et
/// au SQL `WHERE` pour défense en profondeur multi-tenant. Toute tentative
/// de clôture cross-tenant (handler bypass, futur caller direct) est
/// rejetée avec `NotFound` même si l'`id` existe sous une autre company.
///
/// Retourne :
/// - `DbError::NotFound` si l'exercice n'existe pas dans cette company
/// - `DbError::IllegalStateTransition` si l'exercice est déjà clos
///   (le guard `WHERE status = 'Open'` a échoué — transition interdite)
pub async fn close(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    id: i64,
) -> Result<FiscalYear, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let rows_affected = sqlx::query(
        "UPDATE fiscal_years SET status = 'Closed' \
         WHERE id = ? AND company_id = ? AND status = 'Open'",
    )
    .bind(id)
    .bind(company_id)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        // Soit l'exercice n'existe pas dans cette company, soit il est déjà clos.
        let current: Option<(String,)> =
            sqlx::query_as("SELECT status FROM fiscal_years WHERE id = ? AND company_id = ?")
                .bind(id)
                .bind(company_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;
        tx.rollback().await.map_err(map_db_error)?;
        return match current {
            None => Err(DbError::NotFound),
            Some((status,)) if status == "Closed" => Err(DbError::IllegalStateTransition(format!(
                "fiscal_year {id} déjà clos — réouverture interdite (CO art. 957-964)"
            ))),
            // Défensif : sous REPEATABLE READ InnoDB et dans la même transaction,
            // le SELECT post-UPDATE devrait voir cohérent. Cette branche ne peut
            // survenir que via un trigger inattendu ou une isolation plus faible.
            Some((status,)) if status == "Open" => Err(DbError::Invariant(format!(
                "fiscal_year {id} est Open mais l'UPDATE n'a affecté aucune ligne \
                 (race condition ou trigger inattendu)"
            ))),
            Some((status,)) => Err(DbError::Invariant(format!(
                "fiscal_year {id} a un statut inattendu hors schéma : {status}"
            ))),
        };
    }

    let fy = fetch_fiscal_year_in_tx(&mut tx, id).await?;

    audit_log::insert_in_tx(
        &mut tx,
        build_audit_entry(user_id, "fiscal_year.closed", fy.id, snapshot_json(&fy)),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(fy)
}

// --- Helpers privés ---

async fn insert_fiscal_year_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    new: &NewFiscalYear,
) -> Result<i64, DbError> {
    let result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.name)
    .bind(new.start_date)
    .bind(new.end_date)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT fiscal_year".into(),
        ));
    }
    i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))
}

async fn fetch_fiscal_year_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    id: i64,
) -> Result<FiscalYear, DbError> {
    let fy_opt = sqlx::query_as::<_, FiscalYear>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

    fy_opt
        .ok_or_else(|| DbError::Invariant(format!("fiscal_year {id} introuvable après opération")))
}
