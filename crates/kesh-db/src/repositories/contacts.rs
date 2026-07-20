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
use crate::util::search::{escape_boolean_ft, escape_like};

// pub(crate) : réutilisé par `repositories::reconciliation` (résolution des
// contacts des propositions) — une seule liste de colonnes à maintenir.
pub(crate) const COLUMNS: &str = "id, company_id, contact_type, name, first_name, last_name, is_client, is_supplier, \
    address, address_street, address_building, address_postal_code, address_city, \
    address_country, email, phone, ide_number, default_payment_terms, default_payment_terms_days, \
    language, salutation, active, version, created_at, updated_at";

const FIND_BY_ID_SQL: &str = "SELECT id, company_id, contact_type, name, first_name, last_name, is_client, is_supplier, \
    address, address_street, address_building, address_postal_code, address_city, \
    address_country, email, phone, ide_number, default_payment_terms, default_payment_terms_days, \
    language, salutation, active, version, created_at, updated_at FROM contacts WHERE id = ?";

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
        "defaultPaymentTermsDays": c.default_payment_terms_days,
        "language": c.language.map(|l| l.as_str()),
        "salutation": c.salutation.as_str(),
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
            // Story 7-4 / KF-005 : `name` migré vers FULLTEXT BOOLEAN MODE
            // (prefix wildcard auto-append). `email` reste LIKE car format
            // structuré (`@`/`.` séparateurs de tokens FULLTEXT cassent
            // les fragments du type `@gmail`).
            let escaped = escape_boolean_ft(trimmed);
            let email_pattern = format!("%{}%", escape_like(trimmed));
            if escaped.is_empty() {
                qb.push(" AND email LIKE ");
                qb.push_bind(email_pattern);
                qb.push(" ESCAPE '\\\\'");
            } else {
                let bool_query = format!("{escaped}*");
                qb.push(" AND (MATCH(name) AGAINST(");
                qb.push_bind(bool_query);
                qb.push(" IN BOOLEAN MODE) OR email LIKE ");
                qb.push_bind(email_pattern);
                qb.push(" ESCAPE '\\\\')");
            }
        }
    }
}

/// Crée un contact et retourne l'entité persistée, avec audit log atomique.
pub async fn create(pool: &MySqlPool, user_id: i64, new: NewContact) -> Result<Contact, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Colonne `address` dérivée (#213) des composants structurés.
    let display = crate::entities::contact::derive_contact_address_display(
        new.address_street.as_deref(),
        new.address_building.as_deref(),
        new.address_postal_code.as_deref(),
        new.address_city.as_deref(),
    );
    let result = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, first_name, last_name, is_client, is_supplier, \
         address, address_street, address_building, address_postal_code, address_city, \
         address_country, email, phone, ide_number, default_payment_terms, \
         default_payment_terms_days, language, salutation) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.contact_type)
    .bind(&new.name)
    .bind(&new.first_name)
    .bind(&new.last_name)
    .bind(new.is_client)
    .bind(new.is_supplier)
    .bind(&display)
    .bind(&new.address_street)
    .bind(&new.address_building)
    .bind(&new.address_postal_code)
    .bind(&new.address_city)
    .bind(&new.address_country)
    .bind(&new.email)
    .bind(&new.phone)
    .bind(&new.ide_number)
    .bind(&new.default_payment_terms)
    .bind(new.default_payment_terms_days)
    .bind(new.language)
    .bind(new.salutation)
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
        NewAuditLogEntry::user(
            user_id,
            "contact.created".to_string(),
            "contact".to_string(),
            contact.id,
            Some(contact_snapshot_json(&contact)),
        ),
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
///
/// **M7 Pass 1 code review** : Executor générique pour usage en transaction
/// (ex. `&mut **tx` depuis `kesh-api/routes/reconciliation.rs accept_one`),
/// remplace le helper dupliqué `reconciliation::find_contact_by_id_for_company`.
pub async fn find_by_id_in_company<'e, E>(
    executor: E,
    id: i64,
    company_id: i64,
) -> Result<Option<Contact>, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    sqlx::query_as::<_, Contact>(&format!(
        "SELECT {COLUMNS} FROM contacts WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
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

/// Compare l'état persisté (`before`) au payload de modification (`changes`).
/// Retourne `true` si aucun champ métier ne diffère — auquel cas `update()`
/// court-circuite la mutation pour ne pas bumper `version` inutilement (KF-004).
///
/// Ne compare PAS : `id`, `company_id`, `version`, `created_at`, `updated_at`,
/// `active` (gérés hors changements user-form).
fn is_no_op_change(before: &Contact, changes: &ContactUpdate) -> bool {
    before.contact_type == changes.contact_type
        && before.name == changes.name
        && before.first_name == changes.first_name
        && before.last_name == changes.last_name
        && before.is_client == changes.is_client
        && before.is_supplier == changes.is_supplier
        && before.address_street == changes.address_street
        && before.address_building == changes.address_building
        && before.address_postal_code == changes.address_postal_code
        && before.address_city == changes.address_city
        && before.address_country == changes.address_country
        && before.email == changes.email
        && before.phone == changes.phone
        && before.ide_number == changes.ide_number
        && before.default_payment_terms == changes.default_payment_terms
        && before.default_payment_terms_days == changes.default_payment_terms_days
        && before.language == changes.language
        && before.salutation == changes.salutation
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

    // KF-004 : court-circuit no-op AVANT toute mutation.
    // NOTE concurrence (KF-004): sous REPEATABLE READ + plain SELECT, si une tx
    // parallèle commit entre notre BEGIN et ce check, on retourne notre snapshot
    // stale au lieu d'un 409. Race acceptée v0.1 (cf. spec 7-3 §race-condition).
    // Mitigation future: SELECT FOR UPDATE partout (non v0.1).
    if is_no_op_change(&before, &changes) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }

    let display = crate::entities::contact::derive_contact_address_display(
        changes.address_street.as_deref(),
        changes.address_building.as_deref(),
        changes.address_postal_code.as_deref(),
        changes.address_city.as_deref(),
    );
    let rows = sqlx::query(
        "UPDATE contacts SET contact_type = ?, name = ?, first_name = ?, last_name = ?, is_client = ?, is_supplier = ?, \
         address = ?, address_street = ?, address_building = ?, address_postal_code = ?, \
         address_city = ?, address_country = ?, \
         email = ?, phone = ?, ide_number = ?, default_payment_terms = ?, \
         default_payment_terms_days = ?, language = ?, salutation = ?, \
         version = version + 1 \
         WHERE id = ? AND version = ? AND active = TRUE",
    )
    .bind(changes.contact_type)
    .bind(&changes.name)
    .bind(&changes.first_name)
    .bind(&changes.last_name)
    .bind(changes.is_client)
    .bind(changes.is_supplier)
    .bind(&display)
    .bind(&changes.address_street)
    .bind(&changes.address_building)
    .bind(&changes.address_postal_code)
    .bind(&changes.address_city)
    .bind(&changes.address_country)
    .bind(&changes.email)
    .bind(&changes.phone)
    .bind(&changes.ide_number)
    .bind(&changes.default_payment_terms)
    .bind(changes.default_payment_terms_days)
    .bind(changes.language)
    .bind(changes.salutation)
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
        NewAuditLogEntry::user(
            user_id,
            "contact.updated".to_string(),
            "contact".to_string(),
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
        NewAuditLogEntry::user(
            user_id,
            "contact.archived".to_string(),
            "contact".to_string(),
            id,
            Some(contact_snapshot_json(&contact)),
        ),
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
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: crate::entities::contact::Salutation::Neutre,
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
                first_name: None,
                last_name: None,
                is_client: true,
                is_supplier: true,
                address: None,
                address_street: None,
                address_building: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
                default_payment_terms_days: None,
                language: None,
                salutation: crate::entities::contact::Salutation::Neutre,
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
                first_name: None,
                last_name: None,
                is_client: true,
                is_supplier: false,
                address: None,
                address_street: None,
                address_building: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
                default_payment_terms_days: None,
                language: None,
                salutation: crate::entities::contact::Salutation::Neutre,
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
                first_name: None,
                last_name: None,
                is_client: true,
                is_supplier: false,
                address: None,
                address_street: None,
                address_building: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
                default_payment_terms_days: None,
                language: None,
                salutation: crate::entities::contact::Salutation::Neutre,
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
                first_name: None,
                last_name: None,
                is_client: true,
                is_supplier: false,
                address: None,
                address_street: None,
                address_building: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: None,
                default_payment_terms_days: None,
                language: None,
                salutation: crate::entities::contact::Salutation::Neutre,
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

    /// Story 7-4 / KF-005 / T9.2 — adapté du précédent
    /// `test_filter_escape_like_wildcard` qui vérifiait l'échappement
    /// applicatif du `%` dans la clause LIKE.
    ///
    /// Sémantique nouvelle (BOOLEAN MODE) : `%` n'est PAS dans la
    /// strip-list de `escape_boolean_ft` (10 caractères opérateurs
    /// uniquement). Il est traité comme caractère **non-token** par le
    /// tokenizer InnoDB FULLTEXT, donc silencieusement ignoré. Le test
    /// vérifie que (i) la query passe sans erreur SQL et (ii) seul
    /// `"TestContact 100% Promo"` est trouvé via le token `100` extrait
    /// par le tokenizer (puis match `100*` du prefix wildcard).
    #[tokio::test]
    async fn test_search_handles_special_chars() {
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

    /// Régression detector inversé pour KF-005 v0.1 : asserte que la recherche
    /// par fragment-mid-word est PERDUE en BOOLEAN MODE + prefix wildcard.
    /// Si une future migration MariaDB ajoute le suffix wildcard support,
    /// ou si Kesh migre vers Sphinx/Manticore (v0.3+), OU si la config
    /// `innodb_ft_min_token_size=1` est appliquée, ce test FAILERA et
    /// devra être inversé pour asserter le nouveau comportement (match attendu).
    #[tokio::test]
    async fn test_search_no_longer_matches_mid_word() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Camargo Associes"),
        )
        .await
        .unwrap();

        // « argo » est un fragment mid-word de « Camargo » → 0 résultat
        // attendu (régression v0.1 documentée KF-005).
        let mid = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("argo".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(
            mid.total, 0,
            "régression mid-word search documentée : `argo` ne doit plus matcher `Camargo`"
        );

        // Préfixe `camar` matche bien (sémantique préservée).
        let prefix = list_by_company_paginated(
            &pool,
            company_id,
            ContactListQuery {
                search: Some("camar".into()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            prefix.items.iter().any(|c| c.name.contains("Camargo")),
            "préfixe `camar` doit matcher `Camargo` via FULLTEXT prefix wildcard"
        );

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

    fn contact_to_update(c: &Contact) -> ContactUpdate {
        ContactUpdate {
            contact_type: c.contact_type,
            name: c.name.clone(),
            first_name: None,
            last_name: None,
            is_client: c.is_client,
            is_supplier: c.is_supplier,
            address: c.address.clone(),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: c.email.clone(),
            phone: c.phone.clone(),
            ide_number: c.ide_number.clone(),
            default_payment_terms: c.default_payment_terms.clone(),
            default_payment_terms_days: c.default_payment_terms_days,
            language: c.language,
            salutation: c.salutation,
        }
    }

    /// Story 21-1 (#245) : aller-retour du délai de paiement en jours +
    /// modification ISOLÉE du champ → version bumpée (preuve de l'extension
    /// `is_no_op_change`, sinon le changement serait silencieusement ignoré),
    /// puis re-soumission identique → no-op KF-004 (version inchangée).
    #[tokio::test]
    async fn payment_terms_days_roundtrip_and_isolated_update_bumps_version() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;

        let mut new = new_contact(company_id, "TestContact PTD 21-1");
        new.default_payment_terms_days = Some(30);
        let created = create(&pool, user_id, new).await.unwrap();
        assert_eq!(created.default_payment_terms_days, Some(30));

        let found = find_by_id(&pool, created.id).await.unwrap().unwrap();
        assert_eq!(found.default_payment_terms_days, Some(30));

        // Modification isolée du seul champ jours → PAS un no-op.
        let mut changes = contact_to_update(&found);
        changes.default_payment_terms_days = Some(60);
        let updated = update(&pool, found.id, found.version, user_id, changes)
            .await
            .unwrap();
        assert_eq!(updated.default_payment_terms_days, Some(60));
        assert_eq!(
            updated.version,
            found.version + 1,
            "un changement isolé de default_payment_terms_days doit bumper la version"
        );

        // Review Pass 1 AA-2 : l'audit `contact.updated` est bien écrit pour
        // ce changement isolé (pas seulement déduit du bump de version).
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'contact' AND entity_id = ? AND action = 'contact.updated'",
        )
        .bind(found.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(audit_count, 1, "1 entrée audit pour l'update isolé");

        // Payload identique re-soumis → no-op KF-004.
        let noop = contact_to_update(&updated);
        let after = update(&pool, updated.id, updated.version, user_id, noop)
            .await
            .unwrap();
        assert_eq!(after.version, updated.version, "no-op : version inchangée");
    }

    /// KF-004 : payload identique à l'état persisté → pas de bump version,
    /// `updated_at` inchangé, **aucune entrée audit_log `contact.updated`**.
    #[tokio::test]
    async fn update_no_op_returns_unchanged_entity_no_audit() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let mut new = new_contact(company_id, "TestContact NoOp");
        new.address = Some("Rue 1".into());
        new.email = Some("a@b.ch".into());
        let contact = create(&pool, admin_user_id, new).await.unwrap();
        let version_initial = contact.version;
        let updated_at_initial = contact.updated_at;

        let result = update(
            &pool,
            contact.id,
            version_initial,
            admin_user_id,
            contact_to_update(&contact),
        )
        .await
        .unwrap();

        assert_eq!(
            result.version, version_initial,
            "version doit être inchangée"
        );
        assert_eq!(
            result.updated_at, updated_at_initial,
            "updated_at doit être inchangé"
        );
        assert_eq!(result.name, contact.name);

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'contact' AND entity_id = ? AND action = 'contact.updated'",
        )
        .bind(contact.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            count.0, 0,
            "aucune entrée audit_log contact.updated ne doit exister"
        );

        cleanup_test_contacts(&pool, company_id).await;
    }

    /// KF-004 régression : modifier un seul champ doit toujours bumper version
    /// et écrire l'entrée audit_log.
    #[tokio::test]
    async fn update_partial_change_bumps_version() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_contacts(&pool, company_id).await;

        let contact = create(
            &pool,
            admin_user_id,
            new_contact(company_id, "TestContact Partial"),
        )
        .await
        .unwrap();
        let version_initial = contact.version;

        let mut changes = contact_to_update(&contact);
        changes.name = "TestContact Partial Renamed".into();

        let result = update(&pool, contact.id, version_initial, admin_user_id, changes)
            .await
            .unwrap();
        assert_eq!(result.version, version_initial + 1);
        assert_eq!(result.name, "TestContact Partial Renamed");

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'contact' AND entity_id = ? AND action = 'contact.updated'",
        )
        .bind(contact.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 1);

        cleanup_test_contacts(&pool, company_id).await;
    }
}
