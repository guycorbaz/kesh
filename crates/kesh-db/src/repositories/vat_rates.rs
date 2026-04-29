//! Repository read-only pour `vat_rates` (Story 7.2 — KF-003 closure).
//!
//! Pas de `create`/`update`/`delete` exposés v0.1 — réservés Epic 11-1
//! (CRUD admin + historique). Seuls les helpers `seed_default_swiss_rates*`
//! écrivent (variantes pool/tx, idempotents via `INSERT IGNORE`).
//!
//! **Multi-tenant** : toutes les fns lecture exigent `company_id` en paramètre.
//! Aucun lookup global type `find_by_id(rate)` — pattern Story 6-2 / 7-1
//! (Anti-Pattern 4).

use rust_decimal::Decimal;
use sqlx::mysql::MySqlPool;

use crate::entities::VatRate;
use crate::errors::{DbError, map_db_error};

/// Liste les taux TVA actifs d'une company, triés par taux décroissant.
///
/// Utilisé par `GET /api/v1/vat-rates` (handler `list_vat_rates`) et
/// indirectement par les composants frontend via le store de session.
pub async fn list_active_for_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<VatRate>, DbError> {
    sqlx::query_as::<_, VatRate>(
        "SELECT id, company_id, label, rate, valid_from, valid_to, active, created_at, updated_at \
         FROM vat_rates \
         WHERE company_id = ? AND active = TRUE \
         ORDER BY rate DESC",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Cherche un taux TVA actif pour `(company_id, rate)`.
///
/// Utilisé par la validation backend (`verify_vat_rates_against_db`).
/// Scale-invariant : `DECIMAL(5,2)` côté SQL et `Decimal::eq` côté Rust
/// ignorent le scale (`8.1 == 8.10 == 8.100`).
pub async fn find_active_by_rate(
    pool: &MySqlPool,
    company_id: i64,
    rate: &Decimal,
) -> Result<Option<VatRate>, DbError> {
    sqlx::query_as::<_, VatRate>(
        "SELECT id, company_id, label, rate, valid_from, valid_to, active, created_at, updated_at \
         FROM vat_rates \
         WHERE company_id = ? AND rate = ? AND active = TRUE \
         LIMIT 1",
    )
    .bind(company_id)
    .bind(rate)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Seed transactionnel : insère les 4 taux suisses 2024+ pour `company_id`
/// **dans la transaction du caller**.
///
/// Utilisé par `routes/onboarding::finalize_onboarding` pour Path B
/// (atomicité avec le reste du finalize : rollback global si l'un échoue).
///
/// **`INSERT IGNORE`** (cohérent avec `invoice_number_sequences::next_number_for`) :
/// si l'UNIQUE `(company_id, rate, valid_from)` est violée (re-seed après
/// backfill ou Path A déjà passé), la ligne est silencieusement ignorée.
/// Idempotent sans race.
///
/// **Pas d'audit log** : seed = contexte système, pas action utilisateur
/// (cohérent décision Story 3.5 / 3.7 §seed).
pub async fn seed_default_swiss_rates_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to) \
         VALUES \
            (?, 'product-vat-normal',  8.10, '2024-01-01', NULL), \
            (?, 'product-vat-special', 3.80, '2024-01-01', NULL), \
            (?, 'product-vat-reduced', 2.60, '2024-01-01', NULL), \
            (?, 'product-vat-exempt',  0.00, '2024-01-01', NULL)",
    )
    .bind(company_id)
    .bind(company_id)
    .bind(company_id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Pass 1 remediation #19 : `INSERT IGNORE` swallow silencieusement les
    // violations CHECK (MariaDB). Si une maintenance future modifiait les
    // valeurs seed pour des invalides (rate < 0, label vide), 0 row serait
    // insérée sans erreur. Cette assertion post-seed transforme un échec
    // silencieux en `Invariant` explicite.
    //
    // **Borne `>= 4`** (Pass 2 LOW) : v0.1 seed exactement 4 taux suisses
    // 2024+. La borne tolère qu'Epic 11-1 (CRUD admin) ajoute un 5e taux
    // (taux luxe, super-réduit, etc.) sans devoir modifier ce check.
    // L'objectif est uniquement de détecter les seeds *partiels* (< 4),
    // pas d'interdire les extensions.
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vat_rates WHERE company_id = ? AND active = TRUE")
            .bind(company_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?;
    if count < 4 {
        return Err(DbError::Invariant(format!(
            "vat_rates seed failed: company_id={company_id} has {count} rows (expected ≥ 4)"
        )));
    }

    Ok(())
}

/// Variante pool : ouvre une transaction interne, appelle la version
/// `_in_tx`, commit. Utilisée par `kesh_seed::seed_demo` (Path A) qui
/// n'a pas de transaction unique englobante.
pub async fn seed_default_swiss_rates(pool: &MySqlPool, company_id: i64) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    seed_default_swiss_rates_in_tx(&mut tx, company_id).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}
