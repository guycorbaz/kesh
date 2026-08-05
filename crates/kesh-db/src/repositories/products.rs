//! Repository CRUD pour `Product` (Story 4.2).
//!
//! Pattern strictement calqué sur `contacts.rs` (Story 4.1) :
//! - Mutations (`create`, `update`, `archive`) avec audit log atomique
//!   dans la même transaction, rollback explicite sur erreur.
//! - Convention `details_json` : snapshot direct pour create/archive,
//!   wrapper `{before, after}` pour update.
//! - Liste paginée via deux `QueryBuilder` distincts (COUNT + SELECT).

use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use sqlx::mysql::MySqlPool;

use kesh_core::listing::SortDirection;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::product::{NewProduct, Product, ProductUpdate};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;
use crate::util::search::escape_boolean_ft;

const COLUMNS: &str = "id, company_id, name, description, unit_price, vat_rate, \
    default_revenue_account_id, active, version, created_at, updated_at";

/// Toujours scopé par `company_id` (anti-IDOR multi-tenant).
///
/// ⚠️ **Seconde liste de colonnes écrite à la main**, qui ne dérive pas de
/// [`COLUMNS`] : elle alimente six `query_as::<_, Product>`. `Product` dérive
/// `sqlx::FromRow`, donc une colonne absente ici produit un `ColumnNotFound`
/// **à l'exécution** — et met toutes les routes produits en 500. Toute
/// évolution du schéma doit toucher **les deux** constantes.
const FIND_BY_ID_SCOPED_SQL: &str = "SELECT id, company_id, name, description, unit_price, \
    vat_rate, default_revenue_account_id, active, version, created_at, updated_at FROM products \
    WHERE id = ? AND company_id = ?";

/// Snapshot JSON d'un produit pour l'audit log.
/// `unit_price` et `vat_rate` sérialisés en string décimal (pas de perte de précision).
fn product_snapshot_json(p: &Product) -> serde_json::Value {
    serde_json::json!({
        "id": p.id,
        "companyId": p.company_id,
        "name": p.name,
        "description": p.description,
        "unitPrice": p.unit_price.to_string(),
        "vatRate": p.vat_rate.to_string(),
        "defaultRevenueAccountId": p.default_revenue_account_id,
        "active": p.active,
        "version": p.version,
    })
}

/// Colonne de tri pour la liste des produits (whitelist anti-injection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProductSortBy {
    #[default]
    Name,
    UnitPrice,
    VatRate,
    CreatedAt,
}

impl ProductSortBy {
    pub fn as_sql_column(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::UnitPrice => "unit_price",
            Self::VatRate => "vat_rate",
            Self::CreatedAt => "created_at",
        }
    }
}

/// Paramètres de recherche, tri et pagination pour `list_by_company_paginated`.
#[derive(Debug, Clone)]
pub struct ProductListQuery {
    pub search: Option<String>,
    pub include_archived: bool,
    pub sort_by: ProductSortBy,
    pub sort_direction: SortDirection,
    pub limit: i64,
    pub offset: i64,
}

impl Default for ProductListQuery {
    fn default() -> Self {
        Self {
            search: None,
            include_archived: false,
            sort_by: ProductSortBy::default(),
            // Asc par défaut (catalogue alphabétique) — pattern contacts.
            sort_direction: SortDirection::Asc,
            limit: 20,
            offset: 0,
        }
    }
}

/// Résultat paginé retourné par `list_by_company_paginated`.
#[derive(Debug)]
pub struct ProductListResult {
    pub items: Vec<Product>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

fn push_where_clauses<'a>(
    qb: &mut QueryBuilder<'a, sqlx::MySql>,
    company_id: i64,
    query: &'a ProductListQuery,
) {
    qb.push(" WHERE company_id = ");
    qb.push_bind(company_id);

    if !query.include_archived {
        qb.push(" AND active = TRUE");
    }

    if let Some(ref search) = query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            // Story 7-4 / KF-005 : `name` et `description` migrés vers
            // FULLTEXT BOOLEAN MODE (prefix wildcard auto-append). 2 index
            // FULLTEXT séparés (pas combiné) pour conserver la flexibilité
            // d'un futur ranking par colonne.
            let escaped = escape_boolean_ft(trimmed);
            if escaped.is_empty() {
                // Edge case (Pass 1 F4) : input non-vide entièrement composé
                // d'opérateurs BOOLEAN MODE strippés (ex. `"+++"`). Pas de
                // colonne LIKE-friendly à fallback (name + description sont
                // les 2 colonnes search). On force 0 résultats : le
                // pré-refactor `LIKE '%+++%'` retournait 0 par construction,
                // on préserve cette sémantique explicitement (pas de retour
                // de la totalité du catalogue par skip silencieux).
                qb.push(" AND FALSE");
            } else {
                let bool_query = format!("{escaped}*");
                qb.push(" AND (MATCH(name) AGAINST(");
                qb.push_bind(bool_query.clone());
                qb.push(" IN BOOLEAN MODE) OR MATCH(description) AGAINST(");
                qb.push_bind(bool_query);
                qb.push(" IN BOOLEAN MODE))");
            }
        }
    }
}

/// Crée un produit avec audit log atomique.
pub async fn create(pool: &MySqlPool, user_id: i64, new: NewProduct) -> Result<Product, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO products (company_id, name, description, unit_price, vat_rate, \
         default_revenue_account_id) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.name)
    .bind(&new.description)
    .bind(new.unit_price)
    .bind(new.vat_rate)
    .bind(new.default_revenue_account_id)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT products".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let product = sqlx::query_as::<_, Product>(FIND_BY_ID_SCOPED_SQL)
        .bind(id)
        .bind(new.company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant(format!("product {id} introuvable après INSERT")))?;

    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "product.created".to_string(),
            "product".to_string(),
            product.id,
            Some(product_snapshot_json(&product)),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(product)
}

/// Retourne un produit par ID, scopé par `company_id` (anti-IDOR).
pub async fn find_by_id(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<Product>, DbError> {
    sqlx::query_as::<_, Product>(FIND_BY_ID_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Liste paginée avec filtres dynamiques.
pub async fn list_by_company_paginated(
    pool: &MySqlPool,
    company_id: i64,
    query: ProductListQuery,
) -> Result<ProductListResult, DbError> {
    let mut count_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new("SELECT COUNT(*) FROM products");
    push_where_clauses(&mut count_qb, company_id, &query);
    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    let mut items_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new(&format!("SELECT {COLUMNS} FROM products"));
    push_where_clauses(&mut items_qb, company_id, &query);
    items_qb.push(" ORDER BY ");
    items_qb.push(query.sort_by.as_sql_column());
    items_qb.push(" ");
    items_qb.push(query.sort_direction.as_sql_keyword());
    // Tiebreaker déterministe : garantit que la pagination ne skip/duplique pas
    // lorsque plusieurs lignes ont la même valeur sur la colonne de tri.
    items_qb.push(", id ASC");
    items_qb.push(" LIMIT ");
    items_qb.push_bind(query.limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(query.offset);

    let items: Vec<Product> = items_qb
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    Ok(ProductListResult {
        items,
        total,
        offset: query.offset,
        limit: query.limit,
    })
}

/// Compare l'état persisté au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
fn is_no_op_change(before: &Product, changes: &ProductUpdate) -> bool {
    before.name == changes.name
        && before.description == changes.description
        && before.unit_price == changes.unit_price
        && before.vat_rate == changes.vat_rate
        && before.default_revenue_account_id == changes.default_revenue_account_id
}

/// Met à jour un produit actif. Verrouillage optimiste + audit {before, after}.
/// Scopé par `company_id` (anti-IDOR multi-tenant).
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
    changes: ProductUpdate,
) -> Result<Product, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let before_opt = sqlx::query_as::<_, Product>(FIND_BY_ID_SCOPED_SQL)
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
        Some(p) if !p.active => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(
                "impossible de modifier un produit archivé".into(),
            ));
        }
        Some(p) if p.version != version => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        Some(p) => p,
    };

    // KF-004 : court-circuit no-op AVANT toute mutation.
    // NOTE concurrence (KF-004): sous REPEATABLE READ + plain SELECT, si une tx
    // parallèle commit entre notre BEGIN et ce check, on retourne notre snapshot
    // stale au lieu d'un 409. Race acceptée v0.1 (cf. spec 7-3 §race-condition).
    // Mitigation future: SELECT FOR UPDATE partout (non v0.1).
    if is_no_op_change(&before, &changes) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }

    let rows = sqlx::query(
        "UPDATE products SET name = ?, description = ?, unit_price = ?, vat_rate = ?, \
         default_revenue_account_id = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND active = TRUE",
    )
    .bind(&changes.name)
    .bind(&changes.description)
    .bind(changes.unit_price)
    .bind(changes.vat_rate)
    .bind(changes.default_revenue_account_id)
    .bind(id)
    .bind(company_id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let after = sqlx::query_as::<_, Product>(FIND_BY_ID_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let audit_details = serde_json::json!({
        "before": product_snapshot_json(&before),
        "after": product_snapshot_json(&after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "product.updated".to_string(),
            "product".to_string(),
            id,
            Some(audit_details),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Archive un produit (active = false). Verrouillage optimiste + audit.
/// Scopé par `company_id` (anti-IDOR multi-tenant).
pub async fn archive(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<Product, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let current_opt = sqlx::query_as::<_, Product>(FIND_BY_ID_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    match current_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(p) if !p.active => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(
                "produit déjà archivé".into(),
            ));
        }
        Some(p) if p.version != version => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        Some(_) => {}
    }

    let rows = sqlx::query(
        "UPDATE products SET active = FALSE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(id)
    .bind(company_id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let product = sqlx::query_as::<_, Product>(FIND_BY_ID_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "product.archived".to_string(),
            "product".to_string(),
            id,
            Some(product_snapshot_json(&product)),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(product)
}

/// Story 9-2b T3.2.3 — Liste **tous** les produits d'une company sans filtre
/// `active`, pour l'export global ZIP (souveraineté complète : produits
/// archivés inclus pour cohérence référentielle factures).
///
/// Distincte de [`list_by_company_paginated`] (qui propose un filtre
/// `active`). Tri stable `id ASC`.
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<Product>, DbError> {
    sqlx::query_as::<_, Product>(&format!(
        "SELECT {COLUMNS} FROM products \
         WHERE company_id = ? \
         ORDER BY id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Tests d'intégration DB (Story 4.2)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    async fn get_company_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM companies LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one company in DB for tests");
        row.0
    }

    async fn get_admin_user_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one Admin user in DB for tests");
        row.0
    }

    async fn cleanup_test_products(pool: &MySqlPool, company_id: i64) {
        sqlx::query("DELETE FROM products WHERE company_id = ? AND name LIKE 'TestProduct%'")
            .bind(company_id)
            .execute(pool)
            .await
            .ok();
    }

    fn new_product(company_id: i64, name: &str) -> NewProduct {
        NewProduct {
            company_id,
            name: name.to_string(),
            description: None,
            unit_price: dec!(100.00),
            vat_rate: dec!(8.10),
            default_revenue_account_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_and_find() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct 001"),
        )
        .await
        .unwrap();
        assert_eq!(product.name, "TestProduct 001");
        assert!(product.active);
        assert_eq!(product.version, 1);
        assert_eq!(product.unit_price, dec!(100.0000));
        assert_eq!(product.vat_rate, dec!(8.10));

        let found = find_by_id(&pool, company_id, product.id).await.unwrap();
        assert!(found.is_some());

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_create_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Audit"),
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "product", product.id, 10)
            .await
            .unwrap();
        let created_audit = entries
            .iter()
            .find(|e| e.action == "product.created")
            .expect("audit entry product.created must exist");

        assert_eq!(created_audit.user_id, admin_user_id);
        assert_eq!(created_audit.entity_type, "product");
        assert_eq!(created_audit.entity_id, product.id);

        let details = created_audit.details_json.as_ref().unwrap();
        assert!(details.get("before").is_none());
        assert_eq!(
            details.get("name").and_then(|v| v.as_str()),
            Some("TestProduct Audit")
        );
        assert_eq!(
            details.get("unitPrice").and_then(|v| v.as_str()),
            Some("100.0000")
        );
        assert_eq!(
            details.get("vatRate").and_then(|v| v.as_str()),
            Some("8.10")
        );

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_create_rejects_duplicate_name() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Dup"),
        )
        .await
        .unwrap();

        let err = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Dup"),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::UniqueConstraintViolation(_)));

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_optimistic_lock() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Lock"),
        )
        .await
        .unwrap();

        let updated = update(
            &pool,
            company_id,
            product.id,
            product.version,
            admin_user_id,
            ProductUpdate {
                name: "TestProduct Lock Updated".into(),
                description: Some("desc".into()),
                unit_price: dec!(250.5000),
                vat_rate: dec!(2.60),
                default_revenue_account_id: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "TestProduct Lock Updated");
        assert_eq!(updated.version, 2);
        assert_eq!(updated.unit_price, dec!(250.5000));

        let err = update(
            &pool,
            company_id,
            product.id,
            1,
            admin_user_id,
            ProductUpdate {
                name: "Should Fail".into(),
                description: None,
                unit_price: dec!(0),
                vat_rate: dec!(0),
                default_revenue_account_id: None,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::OptimisticLockConflict));

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_writes_audit_log_with_wrapper() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Before"),
        )
        .await
        .unwrap();

        let updated = update(
            &pool,
            company_id,
            product.id,
            product.version,
            admin_user_id,
            ProductUpdate {
                name: "TestProduct After".into(),
                description: None,
                unit_price: dec!(200.0000),
                vat_rate: dec!(8.10),
                default_revenue_account_id: None,
            },
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "product", updated.id, 10)
            .await
            .unwrap();
        let upd = entries
            .iter()
            .find(|e| e.action == "product.updated")
            .expect("audit entry product.updated must exist");

        let details = upd.details_json.as_ref().unwrap();
        let before = details.get("before").expect("wrapper must have 'before'");
        let after = details.get("after").expect("wrapper must have 'after'");
        assert_eq!(
            before.get("name").and_then(|v| v.as_str()),
            Some("TestProduct Before")
        );
        assert_eq!(
            after.get("name").and_then(|v| v.as_str()),
            Some("TestProduct After")
        );
        assert_eq!(
            after.get("unitPrice").and_then(|v| v.as_str()),
            Some("200.0000")
        );

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_rejects_archived() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct ArchUpd"),
        )
        .await
        .unwrap();
        let archived = archive(
            &pool,
            company_id,
            product.id,
            product.version,
            admin_user_id,
        )
        .await
        .unwrap();

        let err = update(
            &pool,
            company_id,
            archived.id,
            archived.version,
            admin_user_id,
            ProductUpdate {
                name: "Should Fail".into(),
                description: None,
                unit_price: dec!(1),
                vat_rate: dec!(0),
                default_revenue_account_id: None,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archive_sets_inactive_and_writes_audit() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Arch"),
        )
        .await
        .unwrap();
        let archived = archive(
            &pool,
            company_id,
            product.id,
            product.version,
            admin_user_id,
        )
        .await
        .unwrap();
        assert!(!archived.active);
        assert_eq!(archived.version, 2);

        let entries = audit_log::find_by_entity(&pool, "product", archived.id, 10)
            .await
            .unwrap();
        let arch = entries
            .iter()
            .find(|e| e.action == "product.archived")
            .expect("audit entry product.archived must exist");
        let details = arch.details_json.as_ref().unwrap();
        assert!(details.get("before").is_none());
        assert_eq!(details.get("active").and_then(|v| v.as_bool()), Some(false));

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archive_rejects_already_archived() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct DoubleArch"),
        )
        .await
        .unwrap();
        let archived = archive(
            &pool,
            company_id,
            product.id,
            product.version,
            admin_user_id,
        )
        .await
        .unwrap();
        let err = archive(
            &pool,
            company_id,
            archived.id,
            archived.version,
            admin_user_id,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_filter_by_search() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Alpha"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Beta"),
        )
        .await
        .unwrap();

        let result = list_by_company_paginated(
            &pool,
            company_id,
            ProductListQuery {
                search: Some("Alpha".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(result.items.iter().any(|p| p.name.contains("Alpha")));
        assert!(!result.items.iter().any(|p| p.name.contains("Beta")));

        cleanup_test_products(&pool, company_id).await;
    }

    /// Régression Pass 1 F4 : un input non-vide entièrement composé
    /// d'opérateurs BOOLEAN MODE (ex. `"+++"`) doit retourner 0 résultats,
    /// PAS la totalité du catalogue (skip silencieux pré-patch).
    #[tokio::test]
    async fn test_filter_by_search_pure_operators_returns_zero() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        // Seed 2 produits — si le skip silencieux régressait, on les
        // retrouverait tous au lieu d'avoir 0 résultats.
        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Gamma"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Delta"),
        )
        .await
        .unwrap();

        for gibberish in ["+++", "***", "()()", "~~~"] {
            let result = list_by_company_paginated(
                &pool,
                company_id,
                ProductListQuery {
                    search: Some(gibberish.into()),
                    limit: 100,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(
                result.total, 0,
                "input pure-opérateurs `{gibberish}` doit retourner 0, pas tout le catalogue"
            );
            assert!(result.items.is_empty());
        }

        cleanup_test_products(&pool, company_id).await;
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
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Cremant Alsace"),
        )
        .await
        .unwrap();

        // « mant » fragment mid-word de « Cremant » → 0 résultat.
        let mid = list_by_company_paginated(
            &pool,
            company_id,
            ProductListQuery {
                search: Some("mant".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(
            mid.total, 0,
            "régression mid-word search documentée : `mant` ne doit plus matcher `Cremant`"
        );

        // Préfixe `crem` matche bien.
        let prefix = list_by_company_paginated(
            &pool,
            company_id,
            ProductListQuery {
                search: Some("crem".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            prefix.items.iter().any(|p| p.name.contains("Cremant")),
            "préfixe `crem` doit matcher `Cremant` via FULLTEXT prefix wildcard"
        );

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_list_sort_order() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Charlie"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Alpha"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Bravo"),
        )
        .await
        .unwrap();

        let asc = list_by_company_paginated(
            &pool,
            company_id,
            ProductListQuery {
                search: Some("TestProduct".into()),
                sort_by: ProductSortBy::Name,
                sort_direction: SortDirection::Asc,
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let names: Vec<_> = asc.items.iter().map(|p| p.name.as_str()).collect();
        let a = names.iter().position(|n| n.contains("Alpha")).unwrap();
        let b = names.iter().position(|n| n.contains("Bravo")).unwrap();
        let c = names.iter().position(|n| n.contains("Charlie")).unwrap();
        assert!(a < b && b < c);

        // Couvre les 3 autres variants whitelist (ne crash pas).
        for sb in [
            ProductSortBy::UnitPrice,
            ProductSortBy::VatRate,
            ProductSortBy::CreatedAt,
        ] {
            let _ = list_by_company_paginated(
                &pool,
                company_id,
                ProductListQuery {
                    search: Some("TestProduct".into()),
                    sort_by: sb,
                    limit: 100,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archived_excluded_by_default() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let active = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct Active"),
        )
        .await
        .unwrap();
        let to_arch = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct ToArch"),
        )
        .await
        .unwrap();
        archive(
            &pool,
            company_id,
            to_arch.id,
            to_arch.version,
            admin_user_id,
        )
        .await
        .unwrap();

        let default_list = list_by_company_paginated(
            &pool,
            company_id,
            ProductListQuery {
                search: Some("TestProduct".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(default_list.items.iter().any(|p| p.id == active.id));
        assert!(!default_list.items.iter().any(|p| p.id == to_arch.id));

        let full = list_by_company_paginated(
            &pool,
            company_id,
            ProductListQuery {
                search: Some("TestProduct".into()),
                include_archived: true,
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(full.items.iter().any(|p| p.id == to_arch.id));

        cleanup_test_products(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_db_rejects_negative_price_via_direct_insert() {
        // Défense en profondeur : vérifie que le CHECK constraint fonctionne
        // indépendamment du handler (cas migration bugguée / seed SQL direct).
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let err = sqlx::query(
            "INSERT INTO products (company_id, name, unit_price, vat_rate) VALUES (?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind("TestProduct NegativePrice")
        .bind(dec!(-1.0000))
        .bind(dec!(8.10))
        .execute(&pool)
        .await
        .unwrap_err();

        let mapped = map_db_error(err);
        assert!(matches!(mapped, DbError::CheckConstraintViolation(_)));

        cleanup_test_products(&pool, company_id).await;
    }

    // ======================================================================
    // Story 16-2a (#144) — compte de produit sur la fiche produit.
    // ======================================================================

    /// Crée un compte de produit utilisable pour les tests, et rend son id.
    ///
    /// Numéro dérivé du suffixe pour éviter la collision sur
    /// `uq_accounts_company_number` entre tests.
    async fn seed_revenue_account(pool: &MySqlPool, company_id: i64, suffix: &str) -> i64 {
        let number = format!("39{suffix}");
        // ⚠️ Ne PAS jeter l'erreur du nettoyage par un `.ok()`. Ces tests
        // tournent sur `test_pool()` — base **partagée et persistante**,
        // contrairement aux `#[sqlx::test]` du reste du crate — et aucun ne
        // supprime son compte `39xx` en fin de test. Un run interrompu laisse
        // donc un produit référencer ce compte : le `DELETE` viole alors
        // `fk_products_default_revenue_account` (`ON DELETE RESTRICT`), et si
        // l'échec est avalé, c'est l'`INSERT` suivant qui panique sur
        // `uq_accounts_company_number` — le diagnostic arrive à un endroit qui
        // n'est pas celui du défaut. *(Passe 1 de revue.)*
        sqlx::query("DELETE FROM accounts WHERE company_id = ? AND number = ?")
            .bind(company_id)
            .bind(&number)
            .execute(pool)
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "nettoyage du compte {number} impossible — un produit d'un run \
                     précédent le référence-t-il encore ? ({e})"
                )
            });
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO accounts (company_id, number, name, account_type, active, version) \
             VALUES (?, ?, 'TestProduct compte', 'Revenue', TRUE, 1) RETURNING id",
        )
        .bind(company_id)
        .bind(&number)
        .fetch_one(pool)
        .await
        .expect("seed du compte de produit");
        row.0
    }

    /// Le champ persiste à la création et se relit à l'identique.
    #[tokio::test]
    async fn revenue_account_persists_and_reads_back() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;
        let account_id = seed_revenue_account(&pool, company_id, "01").await;

        let mut new = new_product(company_id, "TestProduct Persist");
        new.default_revenue_account_id = Some(account_id);
        let created = create(&pool, admin_user_id, new).await.unwrap();
        assert_eq!(created.default_revenue_account_id, Some(account_id));

        let found = find_by_id(&pool, company_id, created.id)
            .await
            .unwrap()
            .expect("produit relu");
        assert_eq!(
            found.default_revenue_account_id,
            Some(account_id),
            "le compte doit être relu par FIND_BY_ID_SCOPED_SQL — une colonne absente \
             de cette SECONDE liste écrite à la main donnerait un ColumnNotFound"
        );

        cleanup_test_products(&pool, company_id).await;
    }

    /// **(a)** Modifier le SEUL compte n'est PAS un no-op : `version` est bumpée
    /// et l'audit écrit.
    ///
    /// Cible de la **mutation 3** (retrait du champ d'`is_no_op_change`).
    /// Distinct du test (b) ci-dessous, et c'est délibéré : sous la mutation 3
    /// ce scénario devient un no-op, `update` rollback et rend `before` **avant
    /// d'écrire l'audit**. Fusionner les deux ferait rougir (b) sous la
    /// mutation 3 aussi, contredisant « chaque mutation tue le test visé ET LUI
    /// SEUL ».
    #[tokio::test]
    async fn changing_only_the_account_bumps_version_and_writes_audit() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;
        let account_id = seed_revenue_account(&pool, company_id, "02").await;

        let created = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct OnlyAccount"),
        )
        .await
        .unwrap();
        assert_eq!(created.default_revenue_account_id, None);

        let mut changes = product_to_update(&created);
        changes.default_revenue_account_id = Some(account_id);
        let updated = update(
            &pool,
            company_id,
            created.id,
            created.version,
            admin_user_id,
            changes,
        )
        .await
        .unwrap();

        assert_eq!(
            updated.version,
            created.version + 1,
            "modifier le seul compte doit bumper la version — non étendu, \
             `is_no_op_change` prendrait ce changement pour un no-op et l'API \
             répondrait succès sans rien écrire"
        );
        assert_eq!(updated.default_revenue_account_id, Some(account_id));

        let entries = audit_log::find_by_entity(&pool, "product", updated.id, 10)
            .await
            .unwrap();
        assert!(
            entries.iter().any(|e| e.action == "product.updated"),
            "une entrée d'audit product.updated doit exister"
        );

        cleanup_test_products(&pool, company_id).await;
    }

    /// **(b)** L'audit montre le changement DE COMPTE, clé par clé.
    ///
    /// Cible de la **mutation 4** (retrait du champ de
    /// `product_snapshot_json`). L'assertion porte sur **la clé**, jamais sur
    /// l'égalité globale des deux objets : `version` fait partie du snapshot et
    /// l'`UPDATE` la bumpe, donc `before != after` serait vrai **à tout coup**
    /// et le test resterait vert sous sa propre mutation.
    ///
    /// Le scénario modifie AUSSI le nom, pour que l'`UPDATE` aboutisse même
    /// sous la mutation 3 — sans quoi ce test rougirait pour la mauvaise raison.
    #[tokio::test]
    async fn audit_shows_the_account_change_key_by_key() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;
        let account_id = seed_revenue_account(&pool, company_id, "03").await;

        let created = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct AuditKey"),
        )
        .await
        .unwrap();

        let mut changes = product_to_update(&created);
        changes.name = "TestProduct AuditKey Renamed".into();
        changes.default_revenue_account_id = Some(account_id);
        let updated = update(
            &pool,
            company_id,
            created.id,
            created.version,
            admin_user_id,
            changes,
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "product", updated.id, 10)
            .await
            .unwrap();
        let upd = entries
            .iter()
            .find(|e| e.action == "product.updated")
            .expect("audit entry product.updated must exist");
        let details = upd.details_json.as_ref().unwrap();
        let before = details.get("before").expect("wrapper must have 'before'");
        let after = details.get("after").expect("wrapper must have 'after'");

        // La clé doit être PRÉSENTE des deux côtés : absente, `get` rendrait
        // `None` des deux côtés et une comparaison naïve les croirait égaux.
        assert!(
            before.get("defaultRevenueAccountId").is_some(),
            "la clé doit figurer dans le snapshot `before`"
        );
        assert!(
            after.get("defaultRevenueAccountId").is_some(),
            "la clé doit figurer dans le snapshot `after`"
        );
        assert!(
            before.get("defaultRevenueAccountId").unwrap().is_null(),
            "avant : aucun compte"
        );
        assert_eq!(
            after
                .get("defaultRevenueAccountId")
                .and_then(|v| v.as_i64()),
            Some(account_id),
            "après : le compte posé — sans cette clé au snapshot, l'audit tairait \
             la seule chose qui a changé"
        );

        cleanup_test_products(&pool, company_id).await;
    }

    /// Une modification sans aucun changement reste un no-op, même avec un
    /// compte posé — la garde d'`is_no_op_change` ne se déclenche pas à tort.
    #[tokio::test]
    async fn identical_payload_with_account_stays_no_op() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;
        let account_id = seed_revenue_account(&pool, company_id, "04").await;

        let mut new = new_product(company_id, "TestProduct NoOp");
        new.default_revenue_account_id = Some(account_id);
        let created = create(&pool, admin_user_id, new).await.unwrap();

        let unchanged = update(
            &pool,
            company_id,
            created.id,
            created.version,
            admin_user_id,
            product_to_update(&created),
        )
        .await
        .unwrap();

        assert_eq!(
            unchanged.version, created.version,
            "payload identique => no-op, version inchangée"
        );

        cleanup_test_products(&pool, company_id).await;
    }

    fn product_to_update(p: &Product) -> ProductUpdate {
        ProductUpdate {
            name: p.name.clone(),
            description: p.description.clone(),
            unit_price: p.unit_price,
            vat_rate: p.vat_rate,
            default_revenue_account_id: p.default_revenue_account_id,
        }
    }

    /// KF-004 : payload identique → pas de bump version, pas d'audit_log.
    #[tokio::test]
    async fn update_no_op_returns_unchanged_entity_no_audit() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct NoOp"),
        )
        .await
        .unwrap();
        let version_initial = product.version;
        let updated_at_initial = product.updated_at;

        let result = update(
            &pool,
            company_id,
            product.id,
            version_initial,
            admin_user_id,
            product_to_update(&product),
        )
        .await
        .unwrap();

        assert_eq!(result.version, version_initial);
        assert_eq!(result.updated_at, updated_at_initial);

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'product' AND entity_id = ? AND action = 'product.updated'",
        )
        .bind(product.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 0);

        cleanup_test_products(&pool, company_id).await;
    }

    /// KF-004 régression : modifier `unit_price` → bump version + audit log présent.
    #[tokio::test]
    async fn update_partial_change_bumps_version() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_products(&pool, company_id).await;

        let product = create(
            &pool,
            admin_user_id,
            new_product(company_id, "TestProduct PriceUp"),
        )
        .await
        .unwrap();
        let version_initial = product.version;

        let mut changes = product_to_update(&product);
        changes.unit_price = dec!(150.00);

        let result = update(
            &pool,
            company_id,
            product.id,
            version_initial,
            admin_user_id,
            changes,
        )
        .await
        .unwrap();
        assert_eq!(result.version, version_initial + 1);
        assert_eq!(result.unit_price, dec!(150.0000));

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'product' AND entity_id = ? AND action = 'product.updated'",
        )
        .bind(product.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 1);

        cleanup_test_products(&pool, company_id).await;
    }
}
