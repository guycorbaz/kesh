//! Repository `vat_rates` — read-only (Story 7.2) + CRUD admin (Story 11-1).
//!
//! Story 11-1 ajoute le CRUD admin (`create`/`update`/`deactivate`), la
//! sélection temporelle par catégorie (`find_for_category_at_date`), la
//! détection de chevauchement par catégorie et le verrou optimiste (`version`).
//!
//! **Multi-tenant** : toutes les fns exigent `company_id` en paramètre. Aucun
//! lookup global type `find_by_id(rate)` — pattern Story 6-2 / 7-1.
//!
//! **Concurrence** : les mutations (`create`/`update`/`deactivate`) doivent être
//! appelées par un handler qui a acquis le sentinel lock company
//! (`bank_accounts::acquire_company_sentinel_lock`) en début de transaction —
//! l'invariant de non-chevauchement par catégorie est une contrainte cross-row.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::{NewVatRate, UpdateVatRate, VatRate};
use crate::errors::{DbError, map_db_error};

/// Liste de colonnes des `SELECT … FROM vat_rates` mappés sur [`VatRate`]
/// (`FromRow` par nom → toutes les colonnes du struct doivent être présentes,
/// `category` et `version` compris — Story 11-1).
const VAT_RATE_COLS: &str = "id, company_id, category, label, rate, valid_from, valid_to, active, \
     version, created_at, updated_at";

/// Story 9-2b — Liste **tous** les taux TVA d'une company sans filtre `active`,
/// pour l'export global ZIP et la **vue historique admin** (Story 11-1). Tri
/// stable `id ASC` (le handler ré-ordonne pour l'affichage admin).
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<VatRate>, DbError> {
    sqlx::query_as::<_, VatRate>(&format!(
        "SELECT {VAT_RATE_COLS} FROM vat_rates WHERE company_id = ? ORDER BY id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les taux TVA **actifs** d'une company, triés par taux décroissant.
///
/// Utilisé par `GET /api/v1/vat-rates` (sans param) — comportement v0.1
/// **préservé** (non-régression facturation/produits).
pub async fn list_active_for_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<VatRate>, DbError> {
    sqlx::query_as::<_, VatRate>(&format!(
        "SELECT {VAT_RATE_COLS} FROM vat_rates \
         WHERE company_id = ? AND active = TRUE ORDER BY rate DESC"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Cherche un taux TVA actif pour `(company_id, rate)`. Utilisé par la
/// validation backend (`verify_vat_rates_against_db`). Scale-invariant.
pub async fn find_active_by_rate(
    pool: &MySqlPool,
    company_id: i64,
    rate: &Decimal,
) -> Result<Option<VatRate>, DbError> {
    sqlx::query_as::<_, VatRate>(&format!(
        "SELECT {VAT_RATE_COLS} FROM vat_rates \
         WHERE company_id = ? AND rate = ? AND active = TRUE LIMIT 1"
    ))
    .bind(company_id)
    .bind(rate)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Story 11-1 — **le** taux d'une catégorie en vigueur à une date donnée.
///
/// Prédicat : `valid_from <= date AND (valid_to IS NULL OR date < valid_to)
/// AND active`. **Déterministe** (au plus une ligne) grâce à l'invariant de
/// non-chevauchement par catégorie (cf. [`has_overlap_in_category`]). C'est la
/// fonction que Story 11-2 (calcul TVA) consommera pour une ligne de facture.
pub async fn find_for_category_at_date(
    pool: &MySqlPool,
    company_id: i64,
    category: &str,
    date: NaiveDate,
) -> Result<Option<VatRate>, DbError> {
    sqlx::query_as::<_, VatRate>(&format!(
        "SELECT {VAT_RATE_COLS} FROM vat_rates \
         WHERE company_id = ? AND category = ? AND active = TRUE \
           AND valid_from <= ? AND (valid_to IS NULL OR ? < valid_to) \
         LIMIT 1"
    ))
    .bind(company_id)
    .bind(category)
    .bind(date)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Cherche un taux par id, scopé company (helper interne post-mutation).
pub async fn find_by_id_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<Option<VatRate>, DbError> {
    sqlx::query_as::<_, VatRate>(&format!(
        "SELECT {VAT_RATE_COLS} FROM vat_rates WHERE company_id = ? AND id = ?"
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Story 11-1 — détecte si un taux **actif** de la même catégorie chevauche la
/// période `[valid_from, valid_to)` proposée (en excluant `exclude_id`, utile
/// sur un UPDATE ; passer `0` à la création — aucun id réel n'est 0).
///
/// Prédicat de chevauchement de deux intervalles avec `valid_to NULL` (=+∞) :
/// `new.valid_from < COALESCE(existing.valid_to, '9999-12-31') AND
///  existing.valid_from < COALESCE(new.valid_to, '9999-12-31')`. Le `COALESCE`
/// est obligatoire — sans lui, `x < NULL` vaut NULL/faux en SQL et raterait le
/// chevauchement (cas dominant : taux seedés `valid_to NULL`). Deux plages
/// adjacentes (`valid_to_A == valid_from_B`) ne chevauchent **pas** (borne
/// `valid_to` exclusive).
///
/// **À appeler sous sentinel lock** (la garantie de non-chevauchement est une
/// contrainte cross-row, pas exprimable par une UNIQUE MariaDB).
pub async fn has_overlap_in_category(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    category: &str,
    valid_from: NaiveDate,
    valid_to: Option<NaiveDate>,
    exclude_id: i64,
) -> Result<bool, DbError> {
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM vat_rates \
         WHERE company_id = ? AND category = ? AND active = TRUE AND id <> ? \
           AND ? < COALESCE(valid_to, '9999-12-31') \
           AND valid_from < COALESCE(?, '9999-12-31') \
         LIMIT 1",
    )
    .bind(company_id)
    .bind(category)
    .bind(exclude_id)
    .bind(valid_from)
    .bind(valid_to)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(exists.is_some())
}

/// Story 11-1 — crée un taux TVA (CRUD admin). À appeler **sous sentinel lock**,
/// après vérification de non-chevauchement par le handler. Retourne le taux créé.
pub async fn create_for_company(
    tx: &mut Transaction<'_, MySql>,
    new: &NewVatRate,
) -> Result<VatRate, DbError> {
    let result = sqlx::query(
        "INSERT INTO vat_rates \
            (company_id, category, label, rate, valid_from, valid_to, active, version) \
         VALUES (?, ?, ?, ?, ?, ?, TRUE, 0)",
    )
    .bind(new.company_id)
    .bind(&new.category)
    .bind(&new.label)
    .bind(new.rate)
    .bind(new.valid_from)
    .bind(new.valid_to)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let id = result.last_insert_id() as i64;
    find_by_id_for_company(tx, new.company_id, id)
        .await?
        .ok_or_else(|| {
            DbError::Invariant(format!("vat_rate create: id={id} not found post-insert"))
        })
}

/// Story 11-1 — modifie un taux existant (`label`, `valid_to`, `active`) avec
/// **verrou optimiste** (`version`). `rate` et `category` ne sont pas
/// modifiables. À appeler **sous sentinel lock**. Conflit de version →
/// `DbError::OptimisticLockConflict` (mappé HTTP 409).
pub async fn update_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    fields: &UpdateVatRate,
    expected_version: i32,
) -> Result<VatRate, DbError> {
    let existing = find_by_id_for_company(tx, company_id, id).await?;
    let existing = match existing {
        Some(v) => v,
        None => return Err(DbError::NotFound),
    };
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE vat_rates SET label = ?, valid_to = ?, active = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(&fields.label)
    .bind(fields.valid_to)
    .bind(fields.active)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    find_by_id_for_company(tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)
}

/// Story 11-1 — désactive (soft-delete) un taux : `active = FALSE`, verrou
/// optimiste. **Jamais de hard-delete** (audit-trail : un taux a pu servir à
/// des factures/écritures historiques). À appeler **sous sentinel lock**.
pub async fn deactivate_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
) -> Result<VatRate, DbError> {
    let existing = find_by_id_for_company(tx, company_id, id).await?;
    let existing = match existing {
        Some(v) => v,
        None => return Err(DbError::NotFound),
    };
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE vat_rates SET active = FALSE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    find_by_id_for_company(tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)
}

/// Seed transactionnel : insère les 4 taux suisses 2024+ pour `company_id`
/// **dans la transaction du caller** (Path B onboarding, atomicité globale).
///
/// **`INSERT IGNORE`** idempotent (UNIQUE `(company_id, rate, valid_from)`).
/// Story 11-1 : pose explicitement `category` (`normal`/`special`/`reduced`/
/// `exempt`) — sinon le défaut `'custom'` casserait la continuité par catégorie.
/// **Pas d'audit log** : seed = contexte système.
pub async fn seed_default_swiss_rates_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT IGNORE INTO vat_rates (company_id, category, label, rate, valid_from, valid_to) \
         VALUES \
            (?, 'normal',  'product-vat-normal',  8.10, '2024-01-01', NULL), \
            (?, 'special', 'product-vat-special', 3.80, '2024-01-01', NULL), \
            (?, 'reduced', 'product-vat-reduced', 2.60, '2024-01-01', NULL), \
            (?, 'exempt',  'product-vat-exempt',  0.00, '2024-01-01', NULL)",
    )
    .bind(company_id)
    .bind(company_id)
    .bind(company_id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Assertion post-seed (transforme un échec silencieux d'INSERT IGNORE en
    // erreur explicite). Borne `>= 4` : tolère qu'un admin ait déjà ajouté des
    // taux (re-finalize / re-seed).
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

/// Variante pool : ouvre une transaction interne, commit. Utilisée par
/// `kesh_seed::seed_demo` (Path A).
pub async fn seed_default_swiss_rates(pool: &MySqlPool, company_id: i64) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    seed_default_swiss_rates_in_tx(&mut tx, company_id).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}
