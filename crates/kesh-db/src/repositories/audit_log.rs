//! Repository pour le journal d'audit.
//!
//! **Pas de méthode `delete`** : CO art. 957-964 impose la conservation 10 ans.
//!
//! ⚠️ **L'absence de `delete` ici ne suffit pas, et ce module l'a affirmé à
//! tort pendant des mois.** Elle garantit que le CRUD métier ne détruit rien —
//! pas que la piste est infalsifiable. Trois chemins la contournaient ; voici
//! ce qu'ils sont devenus (Story 25-1a, #376/#377) :
//!
//! 1. **L'import d'une sauvegarde** — `audit_log` figurait dans
//!    [`crate::backup::TABLES_TO_TRUNCATE`] et la table était **remplacée** par
//!    celle de l'archive. ✅ **Fermé** : la piste locale est désormais conservée
//!    et celle du backup **fusionnée**. La table reste dans la constante — qui
//!    sert aussi à produire l'export et à valider le manifeste — mais elle est
//!    exclue du `DELETE`, comme `onboarding_state`.
//! 2. **`reset_demo`** — `DELETE FROM audit_log` non scopé, sur une route montée
//!    « tout rôle authentifié ». ✅ **Fermé** : la route exige `require_admin_role`
//!    et vit dans le bloc admin. ⚠️ **Elle efface toujours la piste**, mais
//!    seulement sur une instance **non finalisée** : le handler refuse dès
//!    `step_completed >= 7`, inconditionnellement — *« even if
//!    KESH_PRODUCTION_RESET is set »*. Sur une installation en service, la piste
//!    est donc hors d'atteinte.
//! 3. ⚠️ **`/api/v1/_test/seed` et `/api/v1/_test/reset`** — ils appellent
//!    `truncate_all`, qui réutilise la même constante. **Ce chemin RESTE OUVERT,
//!    et c'est délibéré** : tout le montage de la suite E2E en dépend. Il est
//!    gardé au boot (`KESH_TEST_MODE`, et le serveur refuse de démarrer hors
//!    loopback en mode test), jamais actif sur un déploiement réel. *Le nommer
//!    ici est la seule façon de ne pas refabriquer l'affirmation trop large que
//!    l'issue #359 a dû corriger.*
//!
//! ⛔ **`user_id` n'est plus une FK** (Story 25-1a) : c'est un **pointeur
//! logique**, comme `actor_api_key_id` et `entity_id`. La contrainte a été
//! retirée pour que la piste survive au remplacement de `users` par un import —
//! et [`crate::entities::audit_log::AuditLogEntry::actor_label`] porte le nom de
//! l'acteur au moment de l'écriture, faute de quoi retirer la FK aurait
//! transformé un mensonge en trou.
//!
//! La méthode principale [`insert_in_tx`] prend une transaction en cours
//! pour garantir l'atomicité avec l'opération auditée (UPDATE/DELETE
//! d'une écriture, etc.). Si la transaction ROLLBACK, l'entrée d'audit
//! disparaît avec le reste — garantie de cohérence.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::audit_log::{AuditLogEntry, NewAuditLogEntry};
use crate::errors::{DbError, map_db_error};

// Story 17-2a (F-OPUS-3) — `actor_type` + `actor_api_key_id` ajoutés. Doit
// rester en bijection avec les champs de `AuditLogEntry` (FromRow) sinon sqlx
// échoue runtime `ColumnNotFound`.
const COLUMNS: &str = "id, user_id, actor_label, action, entity_type, entity_id, details_json, actor_type, actor_api_key_id, company_id, created_at";

/// Insère une entrée d'audit dans une transaction en cours.
///
/// **Atomicité critique** : cette fonction prend `&mut Transaction<MySql>`
/// et **ne commit jamais**. Le caller (update/delete d'une écriture)
/// gère le commit global. Si l'opération auditée échoue après l'INSERT
/// audit, le ROLLBACK caller supprime aussi cette entrée d'audit —
/// garantie d'intégrité.
pub async fn insert_in_tx(
    tx: &mut Transaction<'_, MySql>,
    new: NewAuditLogEntry,
) -> Result<AuditLogEntry, DbError> {
    let result = sqlx::query(
        // ⛔ `actor_label` est rempli par un SOUS-SELECT, et non par l'appelant.
        //
        // C'est délibéré : le libellé doit être l'INSTANTANÉ du nom au moment de
        // l'écriture, et le faire descendre depuis `CurrentUser` obligerait à
        // toucher tous les sites qui appellent ce repository — sans
        // rien gagner, puisque la valeur cherchée est justement celle que la base
        // porte à cet instant.
        //
        // Le `COALESCE` n'est pas décoratif : depuis la Story 25-1a, `user_id`
        // est un **pointeur logique sans FK** (la contrainte a été retirée pour
        // que la piste survive au remplacement de `users` par un import). Un
        // acteur peut donc, en théorie, ne plus exister — et une piste doit
        // écrire « (inconnu) » plutôt que de refuser d'écrire.
        //
        // ⛔ `company_id` suit le MÊME patron (Story 25-1c-zero), et pour la même
        // raison : la société de l'acteur au moment de l'écriture est une valeur
        // que la base porte déjà. Un utilisateur appartient à exactement une
        // société, et une clé API écrit le `user_id` de son créateur.
        //
        // ⚠️ **Mais SANS `COALESCE`, et ce n'est pas un oubli.** Un libellé doit
        // toujours nommer quelque chose ; une société peut être indéterminable.
        // Un acteur inexistant donne donc `NULL` — état légitime et permanent,
        // que la migration `20260915000001` n'impose ni ne rattrape — et
        // l'`INSERT` réussit : une piste doit écrire plutôt que refuser d'écrire.
        "INSERT INTO audit_log (user_id, actor_label, company_id, action, entity_type, entity_id, details_json, actor_type, actor_api_key_id) \
         VALUES (?, COALESCE((SELECT username FROM users WHERE id = ?), '(inconnu)'), (SELECT company_id FROM users WHERE id = ?), ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.user_id)
    .bind(new.user_id) // sous-SELECT du libellé — cf. le commentaire ci-dessus
    .bind(new.user_id) // sous-SELECT de la société — idem, sans COALESCE
    .bind(&new.action)
    .bind(&new.entity_type)
    .bind(new.entity_id)
    .bind(&new.details_json)
    .bind(new.actor_type)
    .bind(new.actor_api_key_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // P6 : double check — `last_insert_id == 0` attrape les cas
    // `INSERT IGNORE` (non utilisés ici) ; `rows_affected == 0` attrape
    // un INSERT silencieusement ignoré par un trigger inattendu ou un
    // mode SQL permissif. Les deux gardes sont peu coûteuses.
    if result.rows_affected() == 0 {
        return Err(DbError::Invariant(
            "rows_affected == 0 après INSERT audit_log".into(),
        ));
    }
    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT audit_log".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let entry = sqlx::query_as::<_, AuditLogEntry>(&format!(
        "SELECT {COLUMNS} FROM audit_log WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    Ok(entry)
}

/// Liste les entrées d'audit pour une entité donnée, triées du plus
/// récent au plus ancien.
///
/// Utilisé par les tests d'intégration et par la future UI de
/// consultation (story 3.5 ou post-MVP).
pub async fn find_by_entity(
    pool: &MySqlPool,
    entity_type: &str,
    entity_id: i64,
    limit: i64,
) -> Result<Vec<AuditLogEntry>, DbError> {
    sqlx::query_as::<_, AuditLogEntry>(&format!(
        "SELECT {COLUMNS} FROM audit_log \
         WHERE entity_type = ? AND entity_id = ? \
         ORDER BY created_at DESC, id DESC \
         LIMIT ?"
    ))
    .bind(entity_type)
    .bind(entity_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    async fn get_admin_user_id(pool: &MySqlPool) -> i64 {
        sqlx::query_scalar("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one admin user")
    }

    #[tokio::test]
    async fn test_insert_and_find() {
        let pool = test_pool().await;
        let user_id = get_admin_user_id(&pool).await;

        let entity_id = 999_999_i64; // ID fictif, pas de FK à respecter
        let mut tx = pool.begin().await.unwrap();
        let inserted = insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "test.inserted",
                "test_entity",
                entity_id,
                Some(json!({"foo": "bar"})),
            ),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(inserted.user_id, user_id);
        assert_eq!(inserted.action, "test.inserted");
        assert_eq!(inserted.actor_type, crate::entities::ActorType::User);
        assert_eq!(inserted.actor_api_key_id, None);

        let found = find_by_entity(&pool, "test_entity", entity_id, 10)
            .await
            .unwrap();
        assert!(found.iter().any(|e| e.id == inserted.id));

        // Cleanup
        sqlx::query("DELETE FROM audit_log WHERE id = ?")
            .bind(inserted.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_insert_preserves_json_details() {
        let pool = test_pool().await;
        let user_id = get_admin_user_id(&pool).await;
        let entity_id = 888_888_i64;

        let details = json!({
            "before": {"description": "Old", "lines": 2},
            "after": {"description": "New", "lines": 3},
        });

        let mut tx = pool.begin().await.unwrap();
        let inserted = insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "test.json",
                "test_entity",
                entity_id,
                Some(details.clone()),
            ),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(inserted.details_json.as_ref().unwrap(), &details);

        sqlx::query("DELETE FROM audit_log WHERE id = ?")
            .bind(inserted.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_rollback_preserves_no_audit() {
        let pool = test_pool().await;
        let user_id = get_admin_user_id(&pool).await;
        let entity_id = 777_777_i64;

        let mut tx = pool.begin().await.unwrap();
        let inserted = insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(user_id, "test.rollback", "test_entity", entity_id, None),
        )
        .await
        .unwrap();
        let audit_id = inserted.id;
        tx.rollback().await.unwrap();

        // Après rollback, l'entrée ne doit plus exister.
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM audit_log WHERE id = ?")
            .bind(audit_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(exists.is_none(), "audit_log entry should be rolled back");
    }

    // ------------------------------------------------------------------
    // Story 25-1c-zero (refs #378) — `company_id` posé par sous-SELECT.
    //
    // ⛔ Base ÉPHÉMÈRE (squash), et non la base partagée des trois tests
    // ci-dessus : aucun résidu laissé au gate suivant (KF-039, #310).
    //
    // ⛔ Identifiants posés À LA MAIN et DEUX À DEUX DISTINCTS — sociétés 30
    // (factice) et 40, utilisateur 501, `entity_id` 7, clé API 9. Le squash
    // n'insère que `_kesh_version` : sans cela, la première société et le premier
    // utilisateur prendraient tous deux l'`id` 1, et un sous-SELECT fautif
    // `SELECT id FROM users` rendrait la bonne valeur par coïncidence.
    // ------------------------------------------------------------------

    /// Hachage factice : `users` exige `OCTET_LENGTH(password_hash) >= 20`.
    const FAKE_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$factice$factice-factice";

    /// Sème une société factice (30), puis la société de l'acteur (40) et
    /// l'acteur lui-même (501). Rend `(company_id, user_id)` de l'acteur.
    async fn seed_actor(pool: &MySqlPool) -> (i64, i64) {
        for (id, name) in [(30_i64, "Factice"), (40_i64, "Société de l'acteur")] {
            sqlx::query(
                "INSERT INTO companies (id, name, address, org_type, accounting_language, instance_language) \
                 VALUES (?, ?, 'Rue du Test 1', 'Pme', 'FR', 'FR')",
            )
            .bind(id)
            .bind(name)
            .execute(pool)
            .await
            .expect("insert company");
        }
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, role, active, company_id) \
             VALUES (501, 'acteur', ?, 'Comptable', TRUE, 40)",
        )
        .bind(FAKE_HASH)
        .execute(pool)
        .await
        .expect("insert user");
        (40, 501)
    }

    async fn insert(pool: &MySqlPool, new: NewAuditLogEntry) -> AuditLogEntry {
        let mut tx = pool.begin().await.unwrap();
        let entry = insert_in_tx(&mut tx, new).await.expect("insert_in_tx");
        tx.commit().await.unwrap();
        entry
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn insert_sets_the_company_of_a_user_actor(pool: MySqlPool) {
        let (company_id, user_id) = seed_actor(&pool).await;

        let entry = insert(
            &pool,
            NewAuditLogEntry::user(user_id, "test.user", "contact", 7, None),
        )
        .await;

        assert_eq!(
            entry.company_id,
            Some(company_id),
            "la société de l'acteur (40) — ni son id (501), ni la société factice (30)"
        );
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn insert_sets_the_company_of_an_api_key_creator(pool: MySqlPool) {
        let (company_id, user_id) = seed_actor(&pool).await;

        let entry = insert(
            &pool,
            NewAuditLogEntry::api_key(9, user_id, "test.api_key", "contact", 7, None),
        )
        .await;

        assert_eq!(entry.actor_api_key_id, Some(9));
        assert_eq!(
            entry.company_id,
            Some(company_id),
            "une mutation par clé API écrit le `user_id` du CRÉATEUR : même société"
        );
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn insert_with_an_unknown_actor_writes_a_null_company(pool: MySqlPool) {
        seed_actor(&pool).await;

        // Atteignable : `user_id` est un pointeur logique sans FK depuis la 25-1a.
        let entry = insert(
            &pool,
            NewAuditLogEntry::user(777, "test.inconnu", "contact", 7, None),
        )
        .await;

        assert_eq!(
            entry.company_id, None,
            "acteur inexistant ⇒ `NULL`, « société indéterminable » — et l'INSERT réussit"
        );
        assert_eq!(entry.actor_label, "(inconnu)");
    }
}
