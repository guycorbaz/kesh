//! Repository pour `OnboardingState` (table single-row).
//!
//! Pattern simplifié : pas de pagination, pas de list.
//! La table contient au plus une ligne par instance Kesh.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::onboarding::{OnboardingState, UiMode};
use crate::errors::{DbError, map_db_error};

const SELECT_SQL: &str = "SELECT id, step_completed, is_demo, ui_mode, version, created_at, updated_at \
     FROM onboarding_state LIMIT 1";

/// Requête du verrou d'état d'onboarding : la ligne singleton, `FOR UPDATE`,
/// par l'index unique `uq_onboarding_singleton` (Story 15-7a1).
///
/// Publique parce que **partagée avec le test 7 du dépôt**
/// (`tests/onboarding_repository.rs`), qui l'exécute sur une seconde connexion
/// pour prouver que le verrou est tenu — n'est pas une API à étendre. Elle
/// n'est **pas** dérivée de [`SELECT_SQL`] : celle-ci finit par `LIMIT 1` sans
/// `WHERE`, et y accoler `FOR UPDATE` serait une autre requête (balayage au lieu
/// de l'index unique, autres verrous de trou sur table vide).
pub const LOCK_SQL: &str = "SELECT id, step_completed, is_demo, ui_mode, version, created_at, updated_at \
     FROM onboarding_state WHERE singleton = TRUE FOR UPDATE";

/// Verrouille la ligne d'état d'onboarding **dans la transaction de
/// l'appelant** et la rend (`None` si la ligne n'existe pas) — Story 15-7a1,
/// choix C-15-7-20.
///
/// Primitive **sans** comparaison d'étape ni `rollback` : le sort de `None` et
/// la vérification de l'étape sont décidés par chaque appelant. Ne commite
/// jamais.
///
/// ⚠️ **Ordre d'appel** : `lock_state_in_tx` s'appelle **en premier** dans la
/// transaction, avant toute lecture cohérente (non verrouillante). Sous
/// REPEATABLE READ, la première lecture cohérente fige l'instantané ; une garde
/// fondée sur une lecture ultérieure (par ex.
/// `accounts::count_by_company(&mut *tx, …) == 0`) n'est exacte que si ce
/// verrou l'a précédée. Ordre des verrous de référence :
/// `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (onboarding_state, puis company,
/// puis accounts).
pub async fn lock_state_in_tx(
    tx: &mut Transaction<'_, MySql>,
) -> Result<Option<OnboardingState>, DbError> {
    sqlx::query_as::<_, OnboardingState>(LOCK_SQL)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)
}

/// Retourne l'état d'onboarding (ou None si jamais initialisé).
pub async fn get_state(pool: &MySqlPool) -> Result<Option<OnboardingState>, DbError> {
    sqlx::query_as::<_, OnboardingState>(SELECT_SQL)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Initialise l'état d'onboarding avec les defaults (step=0, is_demo=false).
///
/// INSERT puis SELECT dans une transaction (MariaDB n'a pas RETURNING).
pub async fn init_state(pool: &MySqlPool) -> Result<OnboardingState, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result =
        sqlx::query("INSERT INTO onboarding_state (step_completed, is_demo) VALUES (0, FALSE)")
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT onboarding_state".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let state = sqlx::query_as::<_, OnboardingState>(
        "SELECT id, step_completed, is_demo, ui_mode, version, created_at, updated_at \
         FROM onboarding_state WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| DbError::Invariant(format!("onboarding_state {id} introuvable après INSERT")))?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(state)
}

/// Met à jour l'état d'onboarding avec verrouillage optimiste.
///
/// Retourne `DbError::OptimisticLockConflict` si la version ne correspond pas.
pub async fn update_step(
    pool: &MySqlPool,
    step: i32,
    is_demo: bool,
    ui_mode: Option<UiMode>,
    version: i32,
) -> Result<OnboardingState, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let state = update_step_in_tx(&mut tx, step, is_demo, ui_mode, version).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(state)
}

/// Variante transaction-aware de [`update_step`] — Story 25-1b.
///
/// ⛔ **Cette fonction n'écrit AUCUNE trace d'audit, et l'enveloppe ci-dessus
/// non plus.** Dix appelants la partagent — le seed et huit routes d'onboarding,
/// toutes hors du périmètre de la story et suivies par l'issue #434. Y placer
/// l'audit ferait écrire « changement de mode d'affichage » à chaque étape de
/// l'installation : *une trace au mauvais étage ne manque pas, elle ment.*
///
/// Seul `PUT /api/v1/profile/mode` trace, et il le fait dans son handler.
///
/// Ne commite jamais.
pub async fn update_step_in_tx(
    tx: &mut Transaction<'_, MySql>,
    step: i32,
    is_demo: bool,
    ui_mode: Option<UiMode>,
    version: i32,
) -> Result<OnboardingState, DbError> {
    // Filtre sur id ET version pour éviter un UPDATE multi-row si la table
    // est corrompue (normalement single-row, mais défensif).
    let current_id = sqlx::query_as::<_, (i64,)>("SELECT id FROM onboarding_state LIMIT 1")
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?
        .0;

    let rows_affected = sqlx::query(
        "UPDATE onboarding_state \
         SET step_completed = ?, is_demo = ?, ui_mode = ?, version = version + 1 \
         WHERE id = ? AND version = ?",
    )
    .bind(step)
    .bind(is_demo)
    .bind(ui_mode)
    .bind(current_id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    let state = sqlx::query_as::<_, OnboardingState>(SELECT_SQL)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            DbError::Invariant("onboarding_state introuvable après UPDATE réussi".into())
        })?;

    Ok(state)
}

/// Supprime la row onboarding_state (bas niveau).
///
/// L'orchestration complète du reset (nettoyage FK-safe des tables de données)
/// est dans `kesh_seed::reset_demo()`.
pub async fn delete_state(pool: &MySqlPool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM onboarding_state")
        .execute(pool)
        .await
        .map_err(map_db_error)?;
    Ok(())
}
