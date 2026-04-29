//! Repository CRUD pour `Account`.
//!
//! **Story 3.5** : toutes les fonctions CRUD (`create`, `update`, `archive`)
//! enregistrent une entrée d'audit dans la même transaction. La signature
//! accepte un `user_id` pour identifier l'auteur de l'action.
//!
//! **Exception** : `bulk_create_from_chart` (utilisée par le seed) ne
//! génère PAS d'entrée d'audit — contexte système, pas action utilisateur.

use sqlx::mysql::MySqlPool;

use crate::entities::account::{Account, AccountUpdate, NewAccount};
use crate::entities::audit_log::NewAuditLogEntry;
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str = "id, company_id, number, name, account_type, parent_id, active, version, created_at, updated_at";

const FIND_BY_ID_SQL: &str = "SELECT id, company_id, number, name, account_type, parent_id, \
     active, version, created_at, updated_at FROM accounts WHERE id = ?";

/// Snapshot JSON d'un compte pour l'audit log (Story 3.5).
///
/// Contient les champs essentiels pour reconstituer l'état du compte
/// au moment de l'action. Les dates ne sont pas incluses car non
/// pertinentes pour l'audit (l'entrée d'audit a son propre `created_at`).
fn account_snapshot_json(account: &Account) -> serde_json::Value {
    serde_json::json!({
        "id": account.id,
        "companyId": account.company_id,
        "number": account.number,
        "name": account.name,
        "accountType": account.account_type.as_str(),
        "parentId": account.parent_id,
        "active": account.active,
        "version": account.version,
    })
}

/// Crée un compte et retourne l'entité persistée, avec audit log atomique (Story 3.5).
pub async fn create(pool: &MySqlPool, user_id: i64, new: NewAccount) -> Result<Account, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, parent_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.number)
    .bind(&new.name)
    .bind(new.account_type)
    .bind(new.parent_id)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT accounts".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let account = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant(format!("account {id} introuvable après INSERT")))?;

    // Story 3.5 : audit log AVANT commit (snapshot direct, cohérent
    // avec la convention projet documentée en spec 3.5).
    // Rollback explicite pour cohérence avec les autres branches d'erreur.
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "account.created".to_string(),
            entity_type: "account".to_string(),
            entity_id: account.id,
            details_json: Some(account_snapshot_json(&account)),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Retourne un compte par ID (ou None).
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Account>, DbError> {
    sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Retourne un compte par ID si et seulement s'il appartient à la company spécifiée (ou None).
/// Story 6.2: Multi-tenant scoping — utilisé pour les handlers PUT/DELETE qui doivent vérifier IDOR.
pub async fn find_by_id_in_company(
    pool: &MySqlPool,
    id: i64,
    company_id: i64,
) -> Result<Option<Account>, DbError> {
    sqlx::query_as::<_, Account>(
        "SELECT id, company_id, number, name, account_type, parent_id, \
         active, version, created_at, updated_at FROM accounts WHERE id = ? AND company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les comptes d'une company, triés par numéro.
///
/// Retourne le nombre de comptes d'une company.
pub async fn count_by_company(pool: &MySqlPool, company_id: i64) -> Result<i64, DbError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;
    Ok(row.0)
}

/// `include_archived` : si `false`, seuls les comptes actifs sont retournés.
/// Pas de pagination — un plan comptable est borné à ~200-400 comptes.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    include_archived: bool,
) -> Result<Vec<Account>, DbError> {
    if include_archived {
        sqlx::query_as::<_, Account>(&format!(
            "SELECT {COLUMNS} FROM accounts WHERE company_id = ? ORDER BY number"
        ))
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
    } else {
        sqlx::query_as::<_, Account>(&format!(
            "SELECT {COLUMNS} FROM accounts WHERE company_id = ? AND active = TRUE ORDER BY number"
        ))
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
    }
}

/// Compare l'état persisté au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
fn is_no_op_change(before: &Account, changes: &AccountUpdate) -> bool {
    before.name == changes.name && before.account_type == changes.account_type
}

/// Met à jour un compte actif (nom et type). Verrouillage optimiste + audit log (Story 3.5).
/// Retourne `IllegalStateTransition` si le compte est archivé.
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
    changes: AccountUpdate,
) -> Result<Account, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Snapshot "before" AVANT l'UPDATE, dans la même transaction.
    let before_opt = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before = match before_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(a) if !a.active => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(
                "impossible de modifier un compte archivé".into(),
            ));
        }
        Some(a) if a.version != version => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        Some(a) => a,
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
        "UPDATE accounts SET name = ?, account_type = ?, version = version + 1 \
         WHERE id = ? AND version = ? AND active = TRUE",
    )
    .bind(&changes.name)
    .bind(changes.account_type)
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Défensif : ne devrait pas arriver puisqu'on a vérifié avant,
        // mais garde-fou contre une race theoretically possible entre
        // le SELECT et l'UPDATE (lecture repeatable InnoDB).
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let after = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Story 3.5 : audit log avec wrapper {before, after} pour update
    // (cohérent avec journal_entries::update).
    // Rollback explicite pour cohérence avec les autres branches d'erreur.
    let audit_details = serde_json::json!({
        "before": account_snapshot_json(&before),
        "after": account_snapshot_json(&after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "account.updated".to_string(),
            entity_type: "account".to_string(),
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

/// Archive un compte (active = false). Verrouillage optimiste + audit log (Story 3.5).
/// Retourne `IllegalStateTransition` si le compte a des sous-comptes actifs.
pub async fn archive(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<Account, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Vérifier que le compte n'a pas d'enfants actifs
    let children: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE parent_id = ? AND active = TRUE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;
    if children.0 > 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::IllegalStateTransition(
            "impossible d'archiver un compte avec des sous-comptes actifs".into(),
        ));
    }

    let rows = sqlx::query(
        "UPDATE accounts SET active = FALSE, version = version + 1 \
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
        let exists = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_db_error)?;
        return if exists.is_some() {
            Err(DbError::OptimisticLockConflict)
        } else {
            Err(DbError::NotFound)
        };
    }

    let account = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Story 3.5 : audit log (snapshot direct, cohérent avec create/delete).
    // Rollback explicite en cas d'erreur pour cohérence stylistique avec
    // les autres branches de la fonction (le Drop de tx rollback déjà
    // implicitement, mais être explicite évite tout ambiguïté).
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "account.archived".to_string(),
            entity_type: "account".to_string(),
            entity_id: id,
            details_json: Some(account_snapshot_json(&account)),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Crée plusieurs comptes dans une transaction unique.
///
/// Les `NewAccount` doivent avoir `parent_id` déjà résolu (ID réel ou None).
/// Pour le chargement depuis les fichiers JSON (résolution `parent_number` → `parent_id`),
/// utiliser `bulk_create_from_chart` qui gère le tri topologique et la résolution.
///
/// Soit tous les comptes sont créés, soit aucun (rollback complet).
pub async fn bulk_create(
    pool: &MySqlPool,
    accounts: Vec<NewAccount>,
) -> Result<Vec<Account>, DbError> {
    if accounts.is_empty() {
        return Ok(vec![]);
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let mut created_ids: Vec<i64> = Vec::with_capacity(accounts.len());

    for new in &accounts {
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, parent_id) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(new.company_id)
        .bind(&new.number)
        .bind(&new.name)
        .bind(new.account_type)
        .bind(new.parent_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        let last_id = result.last_insert_id();
        if last_id == 0 {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(
                "last_insert_id == 0 après INSERT accounts (bulk)".into(),
            ));
        }
        let id = i64::try_from(last_id).map_err(|_| {
            DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX"))
        })?;

        created_ids.push(id);
    }

    // Récupérer tous les comptes créés
    let placeholders = created_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql =
        format!("SELECT {COLUMNS} FROM accounts WHERE id IN ({placeholders}) ORDER BY number");
    let mut query = sqlx::query_as::<_, Account>(&sql);
    for id in &created_ids {
        query = query.bind(id);
    }
    let result = query.fetch_all(&mut *tx).await.map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(result)
}

/// Crée les comptes d'un plan comptable dans une transaction unique.
///
/// Prend les ChartEntry bruts et résout la hiérarchie parent_number → parent_id
/// en insérant en ordre topologique (tri par longueur de numéro, puis numéro).
///
/// `lang` : code langue lowercase (ex: "fr") pour extraire le nom du compte.
///
/// **Cette fonction ne génère PAS d'entrées d'audit log** (contexte seed
/// système, pas action utilisateur). Elle n'emprunte pas le chemin
/// `create` audité — c'est volontaire et conforme à FR88 (Story 3.5).
pub async fn bulk_create_from_chart(
    pool: &MySqlPool,
    company_id: i64,
    entries: &[kesh_core::chart_of_accounts::ChartEntry],
    lang: &str,
) -> Result<Vec<Account>, DbError> {
    if entries.is_empty() {
        return Ok(vec![]);
    }

    // Trier par longueur de numéro puis par numéro pour ordre topologique
    let mut sorted: Vec<_> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        a.number
            .len()
            .cmp(&b.number.len())
            .then(a.number.cmp(&b.number))
    });

    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let mut number_to_id: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    let mut created_ids: Vec<i64> = Vec::with_capacity(entries.len());

    for entry in &sorted {
        let name = kesh_core::chart_of_accounts::resolve_name(entry, lang);
        let parent_id = entry
            .parent_number
            .as_deref()
            .and_then(|pn| number_to_id.get(pn).copied());

        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, parent_id) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(&entry.number)
        .bind(&name)
        .bind(entry.account_type.as_str())
        .bind(parent_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        let last_id = result.last_insert_id();
        if last_id == 0 {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(
                "last_insert_id == 0 après INSERT accounts (bulk_chart)".into(),
            ));
        }
        let id = i64::try_from(last_id).map_err(|_| {
            DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX"))
        })?;

        number_to_id.insert(&entry.number, id);
        created_ids.push(id);
    }

    let placeholders = created_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql =
        format!("SELECT {COLUMNS} FROM accounts WHERE id IN ({placeholders}) ORDER BY number");
    let mut query = sqlx::query_as::<_, Account>(&sql);
    for id in &created_ids {
        query = query.bind(id);
    }
    let result = query.fetch_all(&mut *tx).await.map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(result)
}

/// Supprime tous les comptes d'une company (utilisé par reset_demo et tests).
pub async fn delete_all_by_company(pool: &MySqlPool, company_id: i64) -> Result<u64, DbError> {
    // Supprimer d'abord les enfants (parent_id NOT NULL) puis les parents
    // En deux passes pour respecter la FK auto-référentielle
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Passe 1 : mettre tous les parent_id à NULL
    sqlx::query("UPDATE accounts SET parent_id = NULL WHERE company_id = ?")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Passe 2 : supprimer tous les comptes
    let rows = sqlx::query("DELETE FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();

    tx.commit().await.map_err(map_db_error)?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::account::AccountType;

    /// Helper : obtient le pool de test via DATABASE_URL depuis .env.
    /// Les tests d'intégration nécessitent une MariaDB réelle.
    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    /// Helper : obtient le company_id de la première company (créée par le seed/onboarding).
    async fn get_company_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM companies LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one company in DB for tests");
        row.0
    }

    /// Helper : obtient un user_id admin pour les appels write qui exigent un acteur audité.
    async fn get_admin_user_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one Admin user in DB for tests");
        row.0
    }

    /// Helper : nettoie les comptes de test (numéros commençant par "T").
    async fn cleanup_test_accounts(pool: &MySqlPool, company_id: i64) {
        // Détacher les parents d'abord
        sqlx::query(
            "UPDATE accounts SET parent_id = NULL WHERE company_id = ? AND number LIKE 'T%'",
        )
        .bind(company_id)
        .execute(pool)
        .await
        .ok();
        sqlx::query("DELETE FROM accounts WHERE company_id = ? AND number LIKE 'T%'")
            .bind(company_id)
            .execute(pool)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_create_and_find() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let new = NewAccount {
            company_id,
            number: "T100".into(),
            name: "Test Create".into(),
            account_type: AccountType::Asset,
            parent_id: None,
        };
        let account = create(&pool, admin_user_id, new).await.unwrap();
        assert_eq!(account.number, "T100");
        assert_eq!(account.name, "Test Create");
        assert_eq!(account.account_type, AccountType::Asset);
        assert!(account.active);
        assert_eq!(account.version, 1);

        let found = find_by_id(&pool, account.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().number, "T100");

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_list_by_company_filters_archived() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        // Créer un compte actif et un archivé
        let active = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T200".into(),
                name: "Active".into(),
                account_type: AccountType::Revenue,
                parent_id: None,
            },
        )
        .await
        .unwrap();

        let archived = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T201".into(),
                name: "To Archive".into(),
                account_type: AccountType::Expense,
                parent_id: None,
            },
        )
        .await
        .unwrap();
        archive(&pool, archived.id, archived.version, admin_user_id)
            .await
            .unwrap();

        // Sans archivés
        let without = list_by_company(&pool, company_id, false).await.unwrap();
        assert!(without.iter().any(|a| a.id == active.id));
        assert!(!without.iter().any(|a| a.id == archived.id));

        // Avec archivés
        let with = list_by_company(&pool, company_id, true).await.unwrap();
        assert!(with.iter().any(|a| a.id == active.id));
        assert!(with.iter().any(|a| a.id == archived.id));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_update_optimistic_locking() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T300".into(),
                name: "Original".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap();

        // Update réussit avec la bonne version
        let updated = update(
            &pool,
            account.id,
            account.version,
            admin_user_id,
            AccountUpdate {
                name: "Updated".into(),
                account_type: AccountType::Liability,
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.account_type, AccountType::Liability);
        assert_eq!(updated.version, 2);

        // Update échoue avec l'ancienne version
        let err = update(
            &pool,
            account.id,
            account.version, // version 1, mais en DB c'est 2
            admin_user_id,
            AccountUpdate {
                name: "Should Fail".into(),
                account_type: AccountType::Asset,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::OptimisticLockConflict));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_archive_sets_inactive() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T400".into(),
                name: "To Archive".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap();
        assert!(account.active);

        let archived = archive(&pool, account.id, account.version, admin_user_id)
            .await
            .unwrap();
        assert!(!archived.active);
        assert_eq!(archived.version, 2);

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_unique_constraint_on_company_number() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T500".into(),
                name: "First".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap();

        // Duplicate number → UniqueConstraintViolation
        let err = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T500".into(),
                name: "Duplicate".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::UniqueConstraintViolation(_)));

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Story 3.5 — vérifie que `create` insère une entrée `audit_log` avec
    /// `action = "account.created"` et un snapshot direct (pas de wrapper).
    #[tokio::test]
    async fn test_create_account_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T600".into(),
                name: "Audit Create".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "account", account.id, 10)
            .await
            .unwrap();
        let created_audit = entries
            .iter()
            .find(|e| e.action == "account.created")
            .expect("audit entry with action account.created must exist");

        assert_eq!(created_audit.user_id, admin_user_id);
        assert_eq!(created_audit.entity_type, "account");
        assert_eq!(created_audit.entity_id, account.id);

        let details = created_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");
        // Convention projet : snapshot direct pour create (pas de wrapper).
        assert!(details.get("before").is_none());
        assert!(details.get("after").is_none());
        assert_eq!(details.get("number").and_then(|v| v.as_str()), Some("T600"));
        assert_eq!(
            details.get("name").and_then(|v| v.as_str()),
            Some("Audit Create")
        );

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Story 3.5 — vérifie que `update` insère une entrée `audit_log` avec
    /// `action = "account.updated"` et un wrapper `{before, after}`.
    #[tokio::test]
    async fn test_update_account_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T601".into(),
                name: "Before Name".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap();

        let updated = update(
            &pool,
            account.id,
            account.version,
            admin_user_id,
            AccountUpdate {
                name: "After Name".into(),
                account_type: AccountType::Liability,
            },
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "account", updated.id, 10)
            .await
            .unwrap();
        let update_audit = entries
            .iter()
            .find(|e| e.action == "account.updated")
            .expect("audit entry with action account.updated must exist");

        let details = update_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");

        // Convention projet : update utilise un wrapper {before, after}.
        let before = details
            .get("before")
            .expect("update audit must wrap snapshot in {{before, after}}");
        let after = details
            .get("after")
            .expect("update audit must wrap snapshot in {{before, after}}");

        assert_eq!(
            before.get("name").and_then(|v| v.as_str()),
            Some("Before Name")
        );
        assert_eq!(
            after.get("name").and_then(|v| v.as_str()),
            Some("After Name")
        );
        assert_eq!(
            before.get("accountType").and_then(|v| v.as_str()),
            Some("Asset")
        );
        assert_eq!(
            after.get("accountType").and_then(|v| v.as_str()),
            Some("Liability")
        );

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Story 3.5 — vérifie que `archive` insère une entrée `audit_log` avec
    /// `action = "account.archived"` et un snapshot direct.
    #[tokio::test]
    async fn test_archive_account_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T602".into(),
                name: "To Archive Audit".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap();

        let archived = archive(&pool, account.id, account.version, admin_user_id)
            .await
            .unwrap();

        let entries = audit_log::find_by_entity(&pool, "account", archived.id, 10)
            .await
            .unwrap();
        let archive_audit = entries
            .iter()
            .find(|e| e.action == "account.archived")
            .expect("audit entry with action account.archived must exist");

        assert_eq!(archive_audit.user_id, admin_user_id);

        let details = archive_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");
        // Snapshot direct (pas de wrapper).
        assert!(details.get("before").is_none());
        assert!(details.get("after").is_none());
        assert_eq!(details.get("active").and_then(|v| v.as_bool()), Some(false));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_bulk_create_from_chart() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        // Charger un petit sous-ensemble du plan PME
        let entries = vec![
            kesh_core::chart_of_accounts::ChartEntry {
                number: "T1".into(),
                name: std::collections::HashMap::from([
                    ("fr".into(), "Test Actifs".into()),
                    ("de".into(), "Test Aktiven".into()),
                    ("it".into(), "Test Attivi".into()),
                    ("en".into(), "Test Assets".into()),
                ]),
                account_type: kesh_core::chart_of_accounts::AccountType::Asset,
                parent_number: None,
            },
            kesh_core::chart_of_accounts::ChartEntry {
                number: "T10".into(),
                name: std::collections::HashMap::from([
                    ("fr".into(), "Test Circulants".into()),
                    ("de".into(), "Test Umlauf".into()),
                    ("it".into(), "Test Circolante".into()),
                    ("en".into(), "Test Current".into()),
                ]),
                account_type: kesh_core::chart_of_accounts::AccountType::Asset,
                parent_number: Some("T1".into()),
            },
        ];

        let created = bulk_create_from_chart(&pool, company_id, &entries, "fr")
            .await
            .unwrap();
        assert_eq!(created.len(), 2);

        let root = created.iter().find(|a| a.number == "T1").unwrap();
        assert_eq!(root.name, "Test Actifs");
        assert!(root.parent_id.is_none());

        let child = created.iter().find(|a| a.number == "T10").unwrap();
        assert_eq!(child.name, "Test Circulants");
        assert_eq!(child.parent_id, Some(root.id));

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// KF-004 : payload identique → pas de bump version, pas d'audit_log.
    #[tokio::test]
    async fn update_no_op_returns_unchanged_entity_no_audit() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T800".into(),
                name: "Test NoOp".into(),
                account_type: AccountType::Revenue,
                parent_id: None,
            },
        )
        .await
        .unwrap();
        let version_initial = account.version;
        let updated_at_initial = account.updated_at;

        let result = update(
            &pool,
            account.id,
            version_initial,
            admin_user_id,
            AccountUpdate {
                name: account.name.clone(),
                account_type: account.account_type,
            },
        )
        .await
        .unwrap();

        assert_eq!(result.version, version_initial);
        assert_eq!(result.updated_at, updated_at_initial);

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? AND action = 'account.updated'",
        )
        .bind(account.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 0);

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// KF-004 régression : modifier `name` → bump version + audit log présent.
    #[tokio::test]
    async fn update_partial_change_bumps_version() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
            admin_user_id,
            NewAccount {
                company_id,
                number: "T801".into(),
                name: "Test Rename".into(),
                account_type: AccountType::Asset,
                parent_id: None,
            },
        )
        .await
        .unwrap();
        let version_initial = account.version;

        let result = update(
            &pool,
            account.id,
            version_initial,
            admin_user_id,
            AccountUpdate {
                name: "Test Rename Updated".into(),
                account_type: account.account_type,
            },
        )
        .await
        .unwrap();
        assert_eq!(result.version, version_initial + 1);
        assert_eq!(result.name, "Test Rename Updated");

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? AND action = 'account.updated'",
        )
        .bind(account.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 1);

        cleanup_test_accounts(&pool, company_id).await;
    }
}
