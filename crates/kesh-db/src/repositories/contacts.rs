//! Repository CRUD pour `Contact` (Story 4.1).
//!
//! Pattern strictement calqué sur `accounts.rs` post-Story 3.5 :
//! - Toutes les mutations (`create`, `update`, `archive`) acceptent `user_id`
//!   et insèrent une entrée `audit_log` dans la même transaction.
//! - Rollback explicite sur erreur audit (cohérence stylistique).
//! - Convention `details_json` : snapshot direct pour create/archive,
//!   wrapper `{before, after}` pour update.
//!
//! Pagination + filtres dynamiques via `sqlx::QueryBuilder` (pattern
//! `journal_entries.rs` Story 3.4). Deux `QueryBuilder` distincts
//! (COUNT + SELECT) car un `QueryBuilder` est un état mutable unique.

use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use sqlx::mysql::MySqlPool;

use kesh_core::listing::SortDirection;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::contact::{Contact, ContactType, ContactUpdate, NewContact};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str = "id, company_id, contact_type, name, is_client, is_supplier, \
    address, email, phone, ide_number, default_payment_terms, active, version, \
    created_at, updated_at";

const FIND_BY_ID_SQL: &str = "SELECT id, company_id, contact_type, name, is_client, is_supplier, \
    address, email, phone, ide_number, default_payment_terms, active, version, \
    created_at, updated_at FROM contacts WHERE id = ?";

/// Échappe les caractères spéciaux pour `LIKE ? ESCAPE '\\'` (pattern
/// Story 3.4 — voir `journal_entries.rs:315-322`). Ordre critique :
/// backslash AVANT `%` et `_`, sinon le backslash injecté par la
/// première passe réinitialise les passes suivantes.
fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Snapshot JSON d'un contact pour l'audit log (Story 3.5 pattern + P8 `companyId`).
fn contact_snapshot_json(c: &Contact) -> serde_json::Value {
    serde_json::json!({
        "id": c.id,
        "companyId": c.company_id,
        "contactType": c.contact_type.as_str(),
        "name": c.name,
        "isClient": c.is_client,
        "isSupplier": c.is_supplier,
        "address": c.address,
        "email": c.email,
        "phone": c.phone,
        "ideNumber": c.ide_number,
        "defaultPaymentTerms": c.default_payment_terms,
        "active": c.active,
        "version": c.version,
    })
}

/// Colonne de tri pour les listes de contacts (whitelist anti-injection).
///
/// Enum local (pas dans `kesh_core::listing` qui est journal-entries-specific).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactSortBy {
    Name,
    CreatedAt,
    UpdatedAt,
}

impl ContactSortBy {
    /// Retourne la colonne SQL littérale (whitelist). **CRITIQUE** : la
    /// valeur est un `&'static str` littéral — jamais construite depuis
    /// l'input utilisateur.
    pub fn as_sql_column(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}

impl Default for ContactSortBy {
    /// Tri par nom par défaut (UX carnet d'adresses alphabétique).
    fn default() -> Self {
        Self::Name
    }
}

/// Paramètres de recherche, tri et pagination pour `list_by_company_paginated`.
#[derive(Debug, Clone)]
pub struct ContactListQuery {
    pub search: Option<String>,
    pub contact_type: Option<ContactType>,
    pub is_client: Option<bool>,
    pub is_supplier: Option<bool>,
    pub include_archived: bool,
    pub sort_by: ContactSortBy,
    pub sort_direction: SortDirection,
    pub limit: i64,
    pub offset: i64,
}

impl Default for ContactListQuery {
    fn default() -> Self {
        Self {
            search: None,
            contact_type: None,
            is_client: None,
            is_supplier: None,
            include_archived: false,
            sort_by: ContactSortBy::default(),
            // IMPORTANT : hardcoder Asc (SortDirection::default() est Desc,
            // convention comptable inappropriée pour un carnet d'adresses).
            sort_direction: SortDirection::Asc,
            limit: 20,
            offset: 0,
        }
    }
}

/// Résultat paginé retourné par `list_by_company_paginated`.
/// Converti en `ListResponse<ContactResponse>` côté handler API.
#[derive(Debug)]
pub struct ContactListResult {
    pub items: Vec<Contact>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Pousse les clauses WHERE dynamiques dans un `QueryBuilder`.
///
/// **CRITIQUE** : cette fonction doit être appelée sur DEUX `QueryBuilder`
/// DISTINCTS (count + items) — un `QueryBuilder` encode un état mutable et
/// ne peut pas être réutilisé après un `build_*`.
fn push_where_clauses<'a>(
    qb: &mut QueryBuilder<'a, sqlx::MySql>,
    company_id: i64,
    query: &'a ContactListQuery,
) {
    qb.push(" WHERE company_id = ");
    qb.push_bind(company_id);

    if !query.include_archived {
        qb.push(" AND active = TRUE");
    }

    if let Some(ct) = query.contact_type {
        qb.push(" AND contact_type = ");
        qb.push_bind(ct);
    }

    if let Some(is_client) = query.is_client {
        qb.push(" AND is_client = ");
        qb.push_bind(is_client);
    }

    if let Some(is_supplier) = query.is_supplier {
        qb.push(" AND is_supplier = ");
        qb.push_bind(is_supplier);
    }

    if let Some(ref search) = query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let pattern = format!("%{}%", escape_like(trimmed));
            qb.push(" AND (name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" ESCAPE '\\\\' OR email LIKE ");
            qb.push_bind(pattern);
            qb.push(" ESCAPE '\\\\')");
        }
    }
}

/// Crée un contact et retourne l'entité persistée, avec audit log atomique.
pub async fn create(pool: &MySqlPool, user_id: i64, new: NewContact) -> Result<Contact, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier, \
         address, email, phone, ide_number, default_payment_terms) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.contact_type)
    .bind(&new.name)
    .bind(new.is_client)
    .bind(new.is_supplier)
    .bind(&new.address)
    .bind(&new.email)
    .bind(&new.phone)
    .bind(&new.ide_number)
    .bind(&new.default_payment_terms)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT contacts".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let contact = sqlx::query_as::<_, Contact>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant(format!("contact {id} introuvable après INSERT")))?;

    // Audit log (snapshot direct, pattern Story 3.5 create).
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "contact.created".to_string(),
            entity_type: "contact".to_string(),
            entity_id: contact.id,
            details_json: Some(contact_snapshot_json(&contact)),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(contact)
}

/// Retourne un contact par ID (ou None).
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Contact>, DbError> {
    sqlx::query_as::<_, Contact>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Retourne un contact par ID si et seulement s'il appartient à la company spécifiée (ou None).
/// Story 6.2: Multi-tenant scoping — utilisé pour les handlers PUT/DELETE qui doivent vérifier IDOR.
pub async fn find_by_id_in_company(
    pool: &MySqlPool,
    id: i64,
    company_id: i64,
) -> Result<Option<Contact>, DbError> {
    sqlx::query_as::<_, Contact>(
        "SELECT id, company_id, contact_type, name, is_client, is_supplier, \
         address, email, phone, ide_number, default_payment_terms, active, version, \
         created_at, updated_at FROM contacts WHERE id = ? AND company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste simple (non paginée) — usage interne / tests.
#[allow(dead_code)]
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    include_archived: bool,
) -> Result<Vec<Contact>, DbError> {
    if include_archived {
        sqlx::query_as::<_, Contact>(&format!(
            "SELECT {COLUMNS} FROM contacts WHERE company_id = ? ORDER BY name"
        ))
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
    } else {
        sqlx::query_as::<_, Contact>(&format!(
            "SELECT {COLUMNS} FROM contacts WHERE company_id = ? AND active = TRUE ORDER BY name"
        ))
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
    }
}

/// Liste paginée avec filtres dynamiques (usage UI).
pub async fn list_by_company_paginated(
    pool: &MySqlPool,
    company_id: i64,
    query: ContactListQuery,
) -> Result<ContactListResult, DbError> {
    // COUNT(*) avec les mêmes WHERE.
    let mut count_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new("SELECT COUNT(*) FROM contacts");
    push_where_clauses(&mut count_qb, company_id, &query);
    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    // SELECT items paginés.
    let mut items_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new(&format!("SELECT {COLUMNS} FROM contacts"));
    push_where_clauses(&mut items_qb, company_id, &query);
    items_qb.push(" ORDER BY ");
    items_qb.push(query.sort_by.as_sql_column());
    items_qb.push(" ");
    items_qb.push(query.sort_direction.as_sql_keyword());
    items_qb.push(" LIMIT ");
    items_qb.push_bind(query.limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(query.offset);

    let items: Vec<Contact> = items_qb
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    Ok(ContactListResult {
        items,
        total,
        offset: query.offset,
        limit: query.limit,
    })
}

/// Met à jour un contact actif. Verrouillage optimiste + audit log (wrapper before/after).
/// Retourne `IllegalStateTransition` si le contact est archivé.
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
    changes: ContactUpdate,
) -> Result<Contact, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Snapshot "before" AVANT l'UPDATE, dans la même transaction.
    let before_opt = sqlx::query_as::<_, Contact>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before = match before_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(c) if !c.active => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(
                "impossible de modifier un contact archivé".into(),
            ));
        }
        Some(c) if c.version != version => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        Some(c) => c,
    };

    let rows = sqlx::query(
        "UPDATE contacts SET contact_type = ?, name = ?, is_client = ?, is_supplier = ?, \
         address = ?, email = ?, phone = ?, ide_number = ?, default_payment_terms = ?, \
         version = version + 1 \
         WHERE id = ? AND version = ? AND active = TRUE",
    )
    .bind(changes.contact_type)
    .bind(&changes.name)
    .bind(changes.is_client)
    .bind(changes.is_supplier)
    .bind(&changes.address)
    .bind(&changes.email)
    .bind(&changes.phone)
    .bind(&changes.ide_number)
    .bind(&changes.default_payment_terms)
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Défensif : race theorique entre le SELECT et l'UPDATE.
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let after = sqlx::query_as::<_, Contact>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Audit log avec wrapper {before, after} (pattern Story 3.5 update).
    let audit_details = serde_json::json!({
        "before": contact_snapshot_json(&before),
        "after": contact_snapshot_json(&after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "contact.updated".to_string(),
            entity_type: "contact".to_string(),
            entity_id: id,
            details_json: Some(audit_details),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Archive un contact (active = false). Verrouillage optimiste + audit log.
/// Retourne `IllegalStateTransition` si le contact est déjà archivé.
pub async fn archive(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<Contact, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Pré-check : existence + pas déjà archivé (état courant) avant l'UPDATE.
    let current_opt = sqlx::query_as::<_, Contact>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    match current_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(c) if !c.active => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(
                "contact déjà archivé".into(),
            ));
        }
        Some(c) if c.version != version => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        Some(_) => {}
    }

    let rows = sqlx::query(
        "UPDATE contacts SET active = FALSE, version = version + 1 \
         WHERE id = ? AND version = ?",
    )
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let contact = sqlx::query_as::<_, Contact>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Audit log (snapshot direct, pattern Story 3.5 archive).
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "contact.archived".to_string(),
            entity_type: "contact".to_string(),
            entity_id: id,
            details_json: Some(contact_snapshot_json(&contact)),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(contact)
}

// ---------------------------------------------------------------------------
// Tests d'intégration DB (Story 4.1)
// ---------------------------------------------------------------------------
//
// Pattern identique à `accounts::tests` (Story 3.5) : pool réel via
// DATABASE_URL, helpers privés dupliqués (get_admin_user_id 4e copie,
// décision documentée spec 3.5 L1), cleanup par préfixe "TestContact" sur
// `name` pour éviter les collisions cross-tests.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::contact::ContactType;

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

    async fn cleanup_test_contacts(pool: &MySqlPool, company_id: i64) {
        sqlx::query("DELETE FROM contacts WHERE company_id = ? AND name LIKE 'TestContact%'")
            .bind(company_id)
            .execute(pool)
            .await
            .ok();
    }

    fn new_contact(company_id: i64, name: &str) -> NewContact {
        NewContact {
            company_id,
            contact_type: ContactType::Entreprise,
            name: name.to_string(),
            is_client: true,
            is_supplier: false,
            address: None,
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: None,
        }
    }

    #[tokio::test]
    async fn test_create_and_find() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact 001"),
        )
        .await
        .unwrap();
        assert_eq!(contact.name, "TestContact 001");
        assert_eq!(contact.contact_type, ContactType::Entreprise);
        assert!(contact.is_client);
        assert!(contact.active);
        assert_eq!(contact.version, 1);

        let found = find_by_id(&pool, contact.id).await.unwrap();
        assert!(found.is_some());

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_create_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Audit"),
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "contact", contact.id, 10)
            .await
            .unwrap();
        let created_audit = entries
            .iter()
            .find(|e| e.action == "contact.created")
            .expect("audit entry contact.created must exist");

        assert_eq!(created_audit.user_id, admin_user_id);
        assert_eq!(created_audit.entity_type, "contact");
        assert_eq!(created_audit.entity_id, contact.id);

        let details = created_audit.details_json.as_ref().unwrap();
        // Snapshot direct : pas de wrapper.
        assert!(details.get("before").is_none());
        assert!(details.get("after").is_none());
        assert_eq!(
            details.get("name").and_then(|v| v.as_str()),
            Some("TestContact Audit")
        );
        assert_eq!(
            details.get("companyId").and_then(|v| v.as_i64()),
            Some(company_id)
        );

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_create_rejects_duplicate_ide() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let mut a = new_contact(company_id, "TestContact IDE A");
        a.ide_number = Some("CHE109322551".into());
        create(&pool, admin_user_id, a).await.unwrap();

        let mut b = new_contact(company_id, "TestContact IDE B");
        b.ide_number = Some("CHE109322551".into());
        let err = create(&pool, admin_user_id, b).await.unwrap_err();
        assert!(matches!(err, DbError::UniqueConstraintViolation(_)));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_create_allows_null_ide_duplicates() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Null A"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Null B"),
        )
        .await
        .unwrap();
        // NULL distinct dans l'index UNIQUE MariaDB.

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_create_stores_normalized_ide() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let mut n = new_contact(company_id, "TestContact MWST");
        n.ide_number = Some("CHE109322551".into());
        let contact = create(&pool, admin_user_id, n).await.unwrap();

        assert_eq!(contact.ide_number, Some("CHE109322551".to_string()));
        assert_eq!(contact.ide_number.as_ref().unwrap().len(), 12);

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_optimistic_lock() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Lock"),
        )
        .await
        .unwrap();

        let updated = update(
            &pool,
            contact.id,
            contact.version,
            admin_user_id,
            ContactUpdate {
                contact_type: ContactType::Personne,
                name: "TestContact Lock Updated".into(),
                is_client: true,
                is_supplier: true,
                address: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "TestContact Lock Updated");
        assert_eq!(updated.version, 2);
        assert!(updated.is_supplier);

        let err = update(
            &pool,
            contact.id,
            1,
            admin_user_id,
            ContactUpdate {
                contact_type: ContactType::Personne,
                name: "Should Fail".into(),
                is_client: true,
                is_supplier: false,
                address: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::OptimisticLockConflict));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_writes_audit_log_with_wrapper() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Before"),
        )
        .await
        .unwrap();

        let updated = update(
            &pool,
            contact.id,
            contact.version,
            admin_user_id,
            ContactUpdate {
                contact_type: ContactType::Personne,
                name: "TestContact After".into(),
                is_client: true,
                is_supplier: false,
                address: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
            },
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "contact", updated.id, 10)
            .await
            .unwrap();
        let update_audit = entries
            .iter()
            .find(|e| e.action == "contact.updated")
            .expect("audit entry contact.updated must exist");

        let details = update_audit.details_json.as_ref().unwrap();
        let before = details.get("before").expect("wrapper must have 'before'");
        let after = details.get("after").expect("wrapper must have 'after'");
        assert_eq!(
            before.get("name").and_then(|v| v.as_str()),
            Some("TestContact Before")
        );
        assert_eq!(
            after.get("name").and_then(|v| v.as_str()),
            Some("TestContact After")
        );
        assert_eq!(
            before.get("contactType").and_then(|v| v.as_str()),
            Some("Entreprise")
        );
        assert_eq!(
            after.get("contactType").and_then(|v| v.as_str()),
            Some("Personne")
        );

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_rejects_archived_contact() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact ToArch"),
        )
        .await
        .unwrap();
        let archived = archive(&pool, contact.id, contact.version, admin_user_id)
            .await
            .unwrap();

        let err = update(
            &pool,
            archived.id,
            archived.version,
            admin_user_id,
            ContactUpdate {
                contact_type: ContactType::Entreprise,
                name: "Should Fail".into(),
                is_client: true,
                is_supplier: false,
                address: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archive_sets_inactive_and_writes_audit() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Arch"),
        )
        .await
        .unwrap();
        assert!(contact.active);

        let archived = archive(&pool, contact.id, contact.version, admin_user_id)
            .await
            .unwrap();
        assert!(!archived.active);
        assert_eq!(archived.version, 2);

        let entries = audit_log::find_by_entity(&pool, "contact", archived.id, 10)
            .await
            .unwrap();
        let archive_audit = entries
            .iter()
            .find(|e| e.action == "contact.archived")
            .expect("audit entry contact.archived must exist");

        let details = archive_audit.details_json.as_ref().unwrap();
        assert!(details.get("before").is_none());
        assert_eq!(details.get("active").and_then(|v| v.as_bool()), Some(false));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archive_rejects_already_archived() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact DoubleArch"),
        )
        .await
        .unwrap();
        let archived = archive(&pool, contact.id, contact.version, admin_user_id)
            .await
            .unwrap();

        let err = archive(&pool, archived.id, archived.version, admin_user_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_filter_by_contact_type() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let mut p1 = new_contact(company_id, "TestContact Personne 1");
        p1.contact_type = ContactType::Personne;
        create(&pool, admin_user_id, p1).await.unwrap();

        let e1 = new_contact(company_id, "TestContact Entreprise 1");
        create(&pool, admin_user_id, e1).await.unwrap();

        let result = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                contact_type: Some(ContactType::Entreprise),
                search: Some("TestContact".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            result
                .items
                .iter()
                .all(|c| c.contact_type == ContactType::Entreprise)
        );
        assert!(
            result
                .items
                .iter()
                .any(|c| c.name == "TestContact Entreprise 1")
        );

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_filter_by_is_client() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let mut client = new_contact(company_id, "TestContact ClientOnly");
        client.is_client = true;
        client.is_supplier = false;
        create(&pool, admin_user_id, client).await.unwrap();

        let mut supplier = new_contact(company_id, "TestContact SupplierOnly");
        supplier.is_client = false;
        supplier.is_supplier = true;
        create(&pool, admin_user_id, supplier).await.unwrap();

        let result = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                is_client: Some(true),
                search: Some("TestContact".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            result
                .items
                .iter()
                .any(|c| c.name == "TestContact ClientOnly")
        );
        assert!(result.items.iter().all(|c| c.is_client));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_filter_by_search_name() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Alpha SA"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Beta GmbH"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Gamma Srl"),
        )
        .await
        .unwrap();

        let result = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("Beta".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(result.items.iter().any(|c| c.name.contains("Beta")));
        assert!(!result.items.iter().any(|c| c.name.contains("Alpha")));
        assert!(!result.items.iter().any(|c| c.name.contains("Gamma")));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_filter_escape_like_wildcard() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact 100% Promo"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Other"),
        )
        .await
        .unwrap();

        // Rechercher exactement "100%" — sans escape le `%` serait wildcard.
        let result = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("100%".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(result.items.iter().any(|c| c.name.contains("100% Promo")));
        assert!(!result.items.iter().any(|c| c.name.contains("Other")));

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_list_sort_order_all_variants() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Charlie"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Alpha"),
        )
        .await
        .unwrap();
        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Bravo"),
        )
        .await
        .unwrap();

        // Name ASC
        let asc = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("TestContact".into()),
                sort_by: ContactSortBy::Name,
                sort_direction: SortDirection::Asc,
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let asc_names: Vec<_> = asc.items.iter().map(|c| c.name.as_str()).collect();
        let a_pos = asc_names.iter().position(|n| n.contains("Alpha")).unwrap();
        let b_pos = asc_names.iter().position(|n| n.contains("Bravo")).unwrap();
        let c_pos = asc_names
            .iter()
            .position(|n| n.contains("Charlie"))
            .unwrap();
        assert!(a_pos < b_pos && b_pos < c_pos);

        // Name DESC
        let desc = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("TestContact".into()),
                sort_by: ContactSortBy::Name,
                sort_direction: SortDirection::Desc,
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let desc_names: Vec<_> = desc.items.iter().map(|c| c.name.as_str()).collect();
        let cd = desc_names
            .iter()
            .position(|n| n.contains("Charlie"))
            .unwrap();
        let ad = desc_names.iter().position(|n| n.contains("Alpha")).unwrap();
        assert!(cd < ad);

        // CreatedAt ASC — garantit que le tri ne crash pas.
        let created_asc = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("TestContact".into()),
                sort_by: ContactSortBy::CreatedAt,
                sort_direction: SortDirection::Asc,
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(created_asc.items.len(), desc.items.len());

        // UpdatedAt DESC — couvre le variant whitelist (P34).
        let updated_desc = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("TestContact".into()),
                sort_by: ContactSortBy::UpdatedAt,
                sort_direction: SortDirection::Desc,
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(updated_desc.items.len(), desc.items.len());

        cleanup_test_contacts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archived_excluded_by_default() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let active = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Active"),
        )
        .await
        .unwrap();
        let to_arch = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact ToArchive"),
        )
        .await
        .unwrap();
        archive(&pool, to_arch.id, to_arch.version, admin_user_id)
            .await
            .unwrap();

        let default_list = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("TestContact".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(default_list.items.iter().any(|c| c.id == active.id));
        assert!(!default_list.items.iter().any(|c| c.id == to_arch.id));

        let full_list = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("TestContact".into()),
                include_archived: true,
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(full_list.items.iter().any(|c| c.id == to_arch.id));

        cleanup_test_contacts(&pool, company_id).await;
    }
}
