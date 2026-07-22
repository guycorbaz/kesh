//! Repository CRUD pour `Account`.
//!
//! **Story 3.5** : toutes les fonctions CRUD (`create`, `update`, `archive`)
//! enregistrent une entrée d'audit dans la même transaction. La signature
//! accepte un `user_id` pour identifier l'auteur de l'action.
//!
//! **Exception** : `bulk_create_from_chart` (utilisée par le seed) ne
//! génère PAS d'entrée d'audit — contexte système, pas action utilisateur.

use sqlx::mysql::MySqlPool;

use crate::entities::account::{Account, AccountRole, AccountUpdate, NewAccount};
use crate::entities::audit_log::NewAuditLogEntry;
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

/// Colonnes de `accounts` sélectionnées pour hydrater [`Account`].
///
/// ⚠️ La colonne générée `singleton_role` est **volontairement absente** : elle
/// n'existe que pour porter la contrainte d'unicité partielle des rôles
/// singleton, elle n'a pas de représentation Rust.
const COLUMNS: &str = "id, company_id, number, name, account_type, parent_id, active, role, postable, version, created_at, updated_at";

/// ⚠️ Duplique volontairement [`COLUMNS`] (via `format!` pour rester synchrone).
/// Toute colonne ajoutée à [`Account`] doit apparaître dans les deux — l'oubli
/// ne casse pas la compilation mais fait échouer `FromRow` au runtime.
fn find_by_id_sql() -> String {
    format!("SELECT {COLUMNS} FROM accounts WHERE id = ?")
}

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
        // Story 14-3a : sans ces deux champs, l'audit log mentirait sur les
        // modifications de rôle et de postabilité (le diff before/after serait vide).
        "role": account.role.map(|r| r.as_str()),
        "postable": account.postable,
        "version": account.version,
    })
}

/// Crée un compte et retourne l'entité persistée, avec audit log atomique (Story 3.5).
pub async fn create(pool: &MySqlPool, user_id: i64, new: NewAccount) -> Result<Account, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, parent_id, role, postable) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.number)
    .bind(&new.name)
    .bind(new.account_type)
    .bind(new.parent_id)
    .bind(new.role)
    .bind(new.postable)
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

    let account = sqlx::query_as::<_, Account>(&find_by_id_sql())
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
        NewAuditLogEntry::user(
            user_id,
            "account.created".to_string(),
            "account".to_string(),
            account.id,
            Some(account_snapshot_json(&account)),
        ),
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
    sqlx::query_as::<_, Account>(&find_by_id_sql())
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
    sqlx::query_as::<_, Account>(&format!(
        "SELECT {COLUMNS} FROM accounts WHERE id = ? AND company_id = ?"
    ))
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
    before.name == changes.name
        && before.account_type == changes.account_type
        // Story 14-3a : sans ces deux comparaisons, un PUT qui ne change QUE le
        // rôle serait silencieusement ignoré et retournerait 200 avec l'ancienne
        // valeur — un bug utilisateur invisible.
        && before.role == changes.role
        && before.postable == changes.postable
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
    let before_opt = sqlx::query_as::<_, Account>(&find_by_id_sql())
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

    // Story 14-3a : si le payload attribue un rôle singleton déjà porté par un
    // autre compte actif, échouer AVANT l'UPDATE pour pouvoir nommer ce compte.
    // La contrainte DB reste le filet (1062 sur course perdue).
    if let Some(role) = changes.role
        && before.role != Some(role)
        && let Some(holder) =
            find_singleton_role_holder(&mut tx, before.company_id, role, id).await?
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(role_conflict(role, holder));
    }

    let rows = sqlx::query(
        "UPDATE accounts SET name = ?, account_type = ?, role = ?, postable = ?, \
         version = version + 1 WHERE id = ? AND version = ? AND active = TRUE",
    )
    .bind(&changes.name)
    .bind(changes.account_type)
    .bind(changes.role)
    .bind(changes.postable)
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

    let after = sqlx::query_as::<_, Account>(&find_by_id_sql())
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
        NewAuditLogEntry::user(
            user_id,
            "account.updated".to_string(),
            "account".to_string(),
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
        let exists = sqlx::query_as::<_, Account>(&find_by_id_sql())
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

    let account = sqlx::query_as::<_, Account>(&find_by_id_sql())
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
        NewAuditLogEntry::user(
            user_id,
            "account.archived".to_string(),
            "account".to_string(),
            id,
            Some(account_snapshot_json(&account)),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Cherche le compte **actif** qui porte déjà `role` dans la société, en
/// excluant `exclude_id` (le compte qu'on est en train d'écrire).
///
/// Retourne `None` si le rôle est libre, ou si `role` n'est pas singleton (dans
/// ce cas plusieurs comptes peuvent légitimement le porter).
///
/// Sert **uniquement** à produire un message d'erreur qui nomme le compte en
/// conflit : la contrainte `uq_accounts_company_singleton_role` reste la source
/// de vérité et rattrape les courses perdues (1062).
async fn find_singleton_role_holder(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    role: AccountRole,
    exclude_id: i64,
) -> Result<Option<Account>, DbError> {
    if !role.is_singleton() {
        return Ok(None);
    }
    sqlx::query_as::<_, Account>(&format!(
        "SELECT {COLUMNS} FROM accounts \
         WHERE company_id = ? AND role = ? AND active = TRUE AND id <> ? LIMIT 1"
    ))
    .bind(company_id)
    .bind(role)
    .bind(exclude_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Construit l'erreur « rôle déjà attribué » à partir du compte détenteur.
fn role_conflict(role: AccountRole, holder: Account) -> DbError {
    DbError::AccountRoleAlreadyAssigned {
        role: role.as_str().to_string(),
        account_id: holder.id,
        account_number: holder.number,
        account_name: holder.name,
    }
}

/// Réactive un compte archivé (`active = false` → `true`). Story 14-3a, issue #269.
///
/// Symétrique de [`archive`] : verrouillage optimiste + audit `account.reactivated`
/// dans la même transaction.
///
/// # Gardes
///
/// - **Parent archivé** → `IllegalStateTransition`. Miroir du garde-fou de
///   [`archive`] (qui refuse d'archiver un compte ayant des enfants actifs) :
///   sans lui on obtiendrait un compte actif sous un parent inactif.
/// - **Rôle singleton repris entre-temps** → [`DbError::AccountRoleAlreadyAssigned`].
///   La réactivation fait repasser la colonne générée `singleton_role` de `NULL`
///   à sa valeur et heurterait le UNIQUE (1062) ; on le détecte en amont pour
///   pouvoir nommer le compte qui bloque.
/// - **Compte déjà actif** → no-op idempotent : entité retournée telle quelle,
///   sans bump de version ni audit (cohérent avec le court-circuit KF-004 d'[`update`]).
pub async fn reactivate(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<Account, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let before = match sqlx::query_as::<_, Account>(&find_by_id_sql())
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
    {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(a) => a,
    };

    // No-op idempotent : déjà actif → on ne touche à rien.
    if before.active {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }

    if before.version != version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    // Garde 1 : le parent doit être actif (sinon arborescence incohérente).
    if let Some(parent_id) = before.parent_id {
        let parent_active: Option<(bool,)> =
            sqlx::query_as("SELECT active FROM accounts WHERE id = ?")
                .bind(parent_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;
        if !parent_active.map(|(a,)| a).unwrap_or(false) {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(
                "impossible de réactiver un compte dont le parent est archivé".into(),
            ));
        }
    }

    // Garde 2 : le rôle singleton du compte a-t-il été repris pendant l'archivage ?
    if let Some(role) = before.role
        && let Some(holder) =
            find_singleton_role_holder(&mut tx, before.company_id, role, id).await?
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(role_conflict(role, holder));
    }

    let rows = sqlx::query(
        "UPDATE accounts SET active = TRUE, version = version + 1 \
         WHERE id = ? AND version = ? AND active = FALSE",
    )
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Défensif : une tx concurrente a modifié la ligne entre le SELECT et l'UPDATE.
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let after = sqlx::query_as::<_, Account>(&find_by_id_sql())
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "account.reactivated".to_string(),
            "account".to_string(),
            id,
            Some(account_snapshot_json(&after)),
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
            "INSERT INTO accounts (company_id, number, name, account_type, parent_id, role, postable) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new.company_id)
        .bind(&new.number)
        .bind(&new.name)
        .bind(new.account_type)
        .bind(new.parent_id)
        .bind(new.role)
        .bind(new.postable)
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

    // Numéros qui sont parents d'au moins une entrée → comptes titres, non-postables.
    let parent_numbers = kesh_core::chart_of_accounts::parent_numbers(entries);

    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let mut number_to_id: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    let mut created_ids: Vec<i64> = Vec::with_capacity(entries.len());

    for entry in &sorted {
        let name = kesh_core::chart_of_accounts::resolve_name(entry, lang);
        let parent_id = entry
            .parent_number
            .as_deref()
            .and_then(|pn| number_to_id.get(pn).copied());

        // Story 14-3a : le rôle vient du plan JSON, la postabilité est calculée
        // par la MÊME fonction pure que celle testée contre le backfill SQL de
        // la migration (invariant « seed ≡ backfill »).
        let role = entry.role.map(|r| r.as_str());
        let postable = kesh_core::chart_of_accounts::is_postable(entry, &parent_numbers);

        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, parent_id, role, postable) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(&entry.number)
        .bind(&name)
        .bind(entry.account_type.as_str())
        .bind(parent_id)
        .bind(role)
        .bind(postable)
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
            role: None,
            postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
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
                role: None,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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
                role: None,
                postable: true,
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

    // =======================================================================
    // Story 14-3a — rôles de comptes & postabilité
    // =======================================================================

    /// Helper : crée un compte de test avec rôle et postabilité explicites.
    async fn mk(
        pool: &MySqlPool,
        company_id: i64,
        user_id: i64,
        number: &str,
        role: Option<AccountRole>,
        postable: bool,
    ) -> Account {
        create(
            pool,
            user_id,
            NewAccount::new(
                company_id,
                number,
                format!("Test {number}"),
                AccountType::Asset,
                None,
            )
            .with_role(role, postable),
        )
        .await
        .unwrap_or_else(|e| panic!("create {number} failed: {e:?}"))
    }

    #[tokio::test]
    async fn role_and_postable_round_trip() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let a = mk(
            &pool,
            company_id,
            user_id,
            "T900",
            Some(AccountRole::VatPayable),
            false,
        )
        .await;
        assert_eq!(a.role, Some(AccountRole::VatPayable));
        assert!(!a.postable);

        // Relecture par un autre chemin (find_by_id) → Decode correct.
        let reread = find_by_id(&pool, a.id).await.unwrap().unwrap();
        assert_eq!(reread.role, Some(AccountRole::VatPayable));
        assert!(!reread.postable);

        // Et par find_by_id_in_company, qui a sa propre liste de colonnes.
        let scoped = find_by_id_in_company(&pool, a.id, company_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(scoped.role, Some(AccountRole::VatPayable));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn update_role_only_bumps_version_and_audits() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let a = mk(&pool, company_id, user_id, "T901", None, true).await;
        let updated = update(
            &pool,
            a.id,
            a.version,
            user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: a.account_type,
                role: Some(AccountRole::EquityCapital),
                postable: true,
            },
        )
        .await
        .unwrap();

        assert_eq!(updated.role, Some(AccountRole::EquityCapital));
        assert_eq!(
            updated.version,
            a.version + 1,
            "changer le rôle DOIT bumper la version"
        );

        // L'audit doit porter le diff de rôle (sinon il mentirait).
        let details: (serde_json::Value,) = sqlx::query_as(
            "SELECT details_json FROM audit_log WHERE entity_type = 'account' AND entity_id = ? \
             AND action = 'account.updated' ORDER BY id DESC LIMIT 1",
        )
        .bind(a.id)
        .fetch_one(&pool)
        .await
        .expect("audit account.updated attendu");
        assert_eq!(details.0["before"]["role"], serde_json::Value::Null);
        assert_eq!(details.0["after"]["role"], "EquityCapital");

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn update_no_op_includes_role_and_postable() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let a = mk(
            &pool,
            company_id,
            user_id,
            "T902",
            Some(AccountRole::EquityOther),
            false,
        )
        .await;
        let same = update(
            &pool,
            a.id,
            a.version,
            user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: a.account_type,
                role: a.role,
                postable: a.postable,
            },
        )
        .await
        .unwrap();
        assert_eq!(same.version, a.version, "no-op strict → pas de bump");

        // Changer UNIQUEMENT postable n'est pas un no-op.
        let toggled = update(
            &pool,
            a.id,
            a.version,
            user_id,
            AccountUpdate {
                name: a.name.clone(),
                account_type: a.account_type,
                role: a.role,
                postable: true,
            },
        )
        .await
        .unwrap();
        assert_eq!(toggled.version, a.version + 1);
        assert!(toggled.postable);

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn singleton_role_rejected_twice_in_same_company() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let holder = mk(
            &pool,
            company_id,
            user_id,
            "T910",
            Some(AccountRole::Receivable),
            true,
        )
        .await;
        let other = mk(&pool, company_id, user_id, "T911", None, true).await;

        let err = update(
            &pool,
            other.id,
            other.version,
            user_id,
            AccountUpdate {
                name: other.name.clone(),
                account_type: other.account_type,
                role: Some(AccountRole::Receivable),
                postable: true,
            },
        )
        .await
        .expect_err("le rôle Receivable est déjà pris");

        match err {
            DbError::AccountRoleAlreadyAssigned {
                ref role,
                account_id,
                ref account_number,
                ..
            } => {
                assert_eq!(role, "Receivable");
                assert_eq!(
                    account_id, holder.id,
                    "l'erreur doit NOMMER le compte en conflit"
                );
                assert_eq!(account_number, "T910");
            }
            other => panic!("attendu AccountRoleAlreadyAssigned, obtenu {other:?}"),
        }

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn multi_valued_role_allowed_twice() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        // EquityOther n'est PAS singleton : 2850 + 2860 coexistent dans les
        // plans association et indépendant.
        mk(
            &pool,
            company_id,
            user_id,
            "T920",
            Some(AccountRole::EquityOther),
            true,
        )
        .await;
        mk(
            &pool,
            company_id,
            user_id,
            "T921",
            Some(AccountRole::EquityOther),
            true,
        )
        .await;
        mk(
            &pool,
            company_id,
            user_id,
            "T922",
            Some(AccountRole::EquityCapital),
            true,
        )
        .await;

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Le test le plus important de la story : l'archivage LIBÈRE le rôle
    /// singleton, et la réactivation est refusée si le rôle a été repris.
    #[tokio::test]
    async fn singleton_role_is_released_on_archive_and_blocks_reactivation() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        // A porte le rôle.
        let a = mk(
            &pool,
            company_id,
            user_id,
            "T930",
            Some(AccountRole::Payable),
            true,
        )
        .await;

        // Archiver A → le rôle est libéré (singleton_role repasse à NULL).
        let a = archive(&pool, a.id, a.version, user_id).await.unwrap();
        assert!(!a.active);
        assert_eq!(
            a.role,
            Some(AccountRole::Payable),
            "archiver ne doit PAS effacer le rôle"
        );

        // B, actif, peut désormais prendre le rôle.
        let b = mk(
            &pool,
            company_id,
            user_id,
            "T931",
            Some(AccountRole::Payable),
            true,
        )
        .await;

        // Réactiver A est refusé, en nommant B.
        let err = reactivate(&pool, a.id, a.version, user_id)
            .await
            .expect_err("le rôle Payable a été repris par B");
        match err {
            DbError::AccountRoleAlreadyAssigned { account_id, .. } => {
                assert_eq!(account_id, b.id);
            }
            other => panic!("attendu AccountRoleAlreadyAssigned, obtenu {other:?}"),
        }

        // Libérer le rôle de B → A redevient réactivable.
        update(
            &pool,
            b.id,
            b.version,
            user_id,
            AccountUpdate {
                name: b.name.clone(),
                account_type: b.account_type,
                role: None,
                postable: true,
            },
        )
        .await
        .unwrap();

        let a = reactivate(&pool, a.id, a.version, user_id).await.unwrap();
        assert!(a.active);
        assert_eq!(a.role, Some(AccountRole::Payable));

        cleanup_test_accounts(&pool, company_id).await;
    }

    #[tokio::test]
    async fn reactivate_nominal_idempotent_and_guards() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let user_id = get_admin_user_id(&pool).await;
        cleanup_test_accounts(&pool, company_id).await;

        let parent = mk(&pool, company_id, user_id, "T940", None, true).await;
        let child = create(
            &pool,
            user_id,
            NewAccount::new(
                company_id,
                "T941",
                "Test enfant",
                AccountType::Asset,
                Some(parent.id),
            ),
        )
        .await
        .unwrap();

        // --- nominal : archiver puis réactiver
        let archived = archive(&pool, child.id, child.version, user_id)
            .await
            .unwrap();
        assert!(!archived.active);
        let back = reactivate(&pool, archived.id, archived.version, user_id)
            .await
            .unwrap();
        assert!(back.active);
        assert_eq!(back.version, archived.version + 1);

        // audit account.reactivated écrit
        let n: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? \
             AND action = 'account.reactivated'",
        )
        .bind(child.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n.0, 1);

        // --- déjà actif : no-op, pas de bump, pas d'audit supplémentaire
        let noop = reactivate(&pool, back.id, back.version, user_id)
            .await
            .unwrap();
        assert_eq!(
            noop.version, back.version,
            "réactiver un compte actif = no-op"
        );
        let n2: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'account' AND entity_id = ? \
             AND action = 'account.reactivated'",
        )
        .bind(child.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n2.0, 1, "no-op ne doit PAS écrire d'audit");

        // --- mauvaise version → conflit optimiste
        let archived = archive(&pool, back.id, back.version, user_id)
            .await
            .unwrap();
        let err = reactivate(&pool, archived.id, archived.version + 99, user_id)
            .await
            .expect_err("version incorrecte");
        assert!(
            matches!(err, DbError::OptimisticLockConflict),
            "obtenu {err:?}"
        );

        // --- parent archivé → refus
        archive(&pool, parent.id, parent.version, user_id)
            .await
            .unwrap();
        let err = reactivate(&pool, archived.id, archived.version, user_id)
            .await
            .expect_err("parent archivé");
        assert!(
            matches!(err, DbError::IllegalStateTransition(_)),
            "attendu IllegalStateTransition, obtenu {err:?}"
        );

        cleanup_test_accounts(&pool, company_id).await;
    }

    /// Garde-fou central : la liste des rôles singleton vit à TROIS endroits
    /// (SQL de la migration, `kesh-db`, `kesh-core`). Ce test compare la source
    /// Rust à l'expression SQL **réellement en base** — sans lui, les deux enums
    /// Rust pourraient dériver de concert avec le SQL sans que rien n'échoue.
    #[tokio::test]
    async fn singleton_list_matches_sql_generation_expression() {
        let pool = test_pool().await;

        let expr: (String,) = sqlx::query_as(
            "SELECT GENERATION_EXPRESSION FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'accounts' \
               AND COLUMN_NAME = 'singleton_role'",
        )
        .fetch_one(&pool)
        .await
        .expect("colonne générée singleton_role introuvable");

        // Extrait les littéraux entre quotes simples de l'expression normalisée
        // par MariaDB, p.ex. : case when `active` <> 0 and `role` in ('A','B') …
        let mut from_sql: Vec<String> = expr
            .0
            .split('\'')
            .skip(1)
            .step_by(2)
            .map(|s| s.to_string())
            .collect();
        from_sql.sort();
        from_sql.dedup();

        let from_rust: Vec<String> = AccountRole::singletons()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(
            from_sql, from_rust,
            "la liste des rôles singleton du SQL ({from_sql:?}) diverge de celle de Rust ({from_rust:?}) \
             — synchroniser migration + kesh-db + kesh-core"
        );

        // Et les deux enums Rust doivent être identiques entre eux.
        let core: Vec<&str> = kesh_core::chart_of_accounts::AccountRole::ALL
            .iter()
            .map(|r| r.as_str())
            .collect();
        let db: Vec<&str> = AccountRole::ALL.iter().map(|r| r.as_str()).collect();
        assert_eq!(core, db, "les deux enums AccountRole ont divergé");

        for (c, d) in kesh_core::chart_of_accounts::AccountRole::ALL
            .iter()
            .zip(AccountRole::ALL.iter())
        {
            assert_eq!(
                c.is_singleton(),
                d.is_singleton(),
                "is_singleton() diverge pour {}",
                c.as_str()
            );
        }
    }
}
