//! Repository CRUD pour `Account`.

use sqlx::mysql::MySqlPool;

use crate::entities::account::{Account, AccountUpdate, NewAccount};
use crate::errors::{map_db_error, DbError};

const COLUMNS: &str = "id, company_id, number, name, account_type, parent_id, active, version, created_at, updated_at";

const FIND_BY_ID_SQL: &str = "SELECT id, company_id, number, name, account_type, parent_id, \
     active, version, created_at, updated_at FROM accounts WHERE id = ?";

/// Crée un compte et retourne l'entité persistée.
pub async fn create(pool: &MySqlPool, new: NewAccount) -> Result<Account, DbError> {
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

/// Liste les comptes d'une company, triés par numéro.
///
/// Retourne le nombre de comptes d'une company.
pub async fn count_by_company(pool: &MySqlPool, company_id: i64) -> Result<i64, DbError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
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

/// Met à jour un compte actif (nom et type). Verrouillage optimiste.
/// Retourne `IllegalStateTransition` si le compte est archivé.
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    changes: AccountUpdate,
) -> Result<Account, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

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
        tx.rollback().await.map_err(map_db_error)?;
        let exists = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(map_db_error)?;
        return match exists {
            None => Err(DbError::NotFound),
            Some(a) if !a.active => Err(DbError::IllegalStateTransition(
                "impossible de modifier un compte archivé".into(),
            )),
            Some(_) => Err(DbError::OptimisticLockConflict),
        };
    }

    let account = sqlx::query_as::<_, Account>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Archive un compte (active = false). Verrouillage optimiste.
/// Retourne `IllegalStateTransition` si le compte a des sous-comptes actifs.
pub async fn archive(pool: &MySqlPool, id: i64, version: i32) -> Result<Account, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Vérifier que le compte n'a pas d'enfants actifs
    let children: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM accounts WHERE parent_id = ? AND active = TRUE",
    )
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
    let sql = format!(
        "SELECT {COLUMNS} FROM accounts WHERE id IN ({placeholders}) ORDER BY number"
    );
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
    let mut number_to_id: std::collections::HashMap<&str, i64> =
        std::collections::HashMap::new();
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
    let sql = format!(
        "SELECT {COLUMNS} FROM accounts WHERE id IN ({placeholders}) ORDER BY number"
    );
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
        cleanup_test_accounts(&pool, company_id).await;

        let new = NewAccount {
            company_id,
            number: "T100".into(),
            name: "Test Create".into(),
            account_type: AccountType::Asset,
            parent_id: None,
        };
        let account = create(&pool, new).await.unwrap();
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
        cleanup_test_accounts(&pool, company_id).await;

        // Créer un compte actif et un archivé
        let active = create(
            &pool,
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
        archive(&pool, archived.id, archived.version).await.unwrap();

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
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
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
        cleanup_test_accounts(&pool, company_id).await;

        let account = create(
            &pool,
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

        let archived = archive(&pool, account.id, account.version).await.unwrap();
        assert!(!archived.active);
        assert_eq!(archived.version, 2);

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn test_unique_constraint_on_company_number() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        create(
            &pool,
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
}
