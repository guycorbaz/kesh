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

use chrono::NaiveDate;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, QueryBuilder, Transaction};

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
/// ⚠️ **Non scopée par société, et sans pagination** : elle sert aux tests
/// d'intégration. La consultation passe par [`list_by_company_paginated`]
/// (Story 25-1c-a), qui filtre par société et pagine.
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

/// Borne haute de la pagination, **identique à celle de la route**
/// (`routes/audit_log.rs`, Story 25-1c-a AC 7). Un clamp qui diverge de la
/// route rendrait le contrôle de la route invérifiable depuis ce module.
pub const MAX_LIMIT: i64 = 200;

/// Filtres de consultation du journal d'audit (Story 25-1c-a).
///
/// ⚠️ **Les dates s'entendent en JOURS UTC.** Ce n'est pas le réglage du
/// serveur qui le garantit (`@@time_zone` vaut `SYSTEM`) mais **sqlx**, qui
/// impose `time_zone='+00:00'` à chaque session
/// (`sqlx-mysql-0.8.6/src/options/mod.rs:112`).
#[derive(Debug, Clone, Default)]
pub struct AuditLogListQuery {
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub action: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Page de consultation : les lignes, et le total **indépendant** de la page.
#[derive(Debug)]
pub struct AuditLogListResult {
    pub items: Vec<AuditLogEntry>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Pousse les clauses WHERE dans un `QueryBuilder`.
///
/// **CRITIQUE** : à appeler sur DEUX `QueryBuilder` **distincts** (count et
/// items) — un `QueryBuilder` encode un état mutable et ne se réutilise pas
/// après un `build_*` (cf. `journal_entries.rs:745-747`).
///
/// ⛔ **Précondition** : `date_from` et `date_to` sont dans
/// `[1000-01-01, 9999-12-31]`. Hors de cette plage, `push_bind` **panique**
/// (`sqlx-core-0.8.6/src/query_builder.rs:158`), et aucun `CatchPanic`
/// n'existe dans `kesh-api` : c'est la route qui valide et répond 400.
///
/// ⛔ **Filtre strict par société** : `company_id = ?`, sans `OR company_id IS
/// NULL` (arbitrage du 2026-09-15). Une entrée sans société n'apparaît dans
/// aucune consultation.
///
/// ⛔ **Bornes de date INCLUSIVES à la milliseconde, sans arithmétique** :
/// `created_at` est un `DATETIME(3)` ; aucune valeur ne tient entre
/// `23:59:59.999` et le lendemain. La forme « `< date_to + 1 jour` », essayée
/// d'abord, paniquait au dernier jour représentable et rendait zéro ligne à
/// l'an 10000.
fn push_where_clauses<'a>(
    qb: &mut QueryBuilder<'a, sqlx::MySql>,
    company_id: i64,
    query: &'a AuditLogListQuery,
) {
    qb.push(" WHERE company_id = ");
    qb.push_bind(company_id);

    if let Some(date_from) = query.date_from {
        // `and_hms_milli_opt(0, 0, 0, 0)` ne rend `None` que pour des valeurs
        // hors plage horaire — impossible avec des constantes littérales.
        let borne = date_from
            .and_hms_milli_opt(0, 0, 0, 0)
            .expect("00:00:00.000 est une heure valide");
        qb.push(" AND created_at >= ");
        qb.push_bind(borne);
    }

    if let Some(date_to) = query.date_to {
        let borne = date_to
            .and_hms_milli_opt(23, 59, 59, 999)
            .expect("23:59:59.999 est une heure valide");
        qb.push(" AND created_at <= ");
        qb.push_bind(borne);
    }

    if let Some(ref entity_type) = query.entity_type {
        qb.push(" AND entity_type = ");
        qb.push_bind(entity_type);
    }

    if let Some(entity_id) = query.entity_id {
        qb.push(" AND entity_id = ");
        qb.push_bind(entity_id);
    }

    if let Some(ref action) = query.action {
        qb.push(" AND action = ");
        qb.push_bind(action);
    }
}

/// Consultation paginée du journal d'audit d'une société (Story 25-1c-a).
///
/// Deux requêtes séquentielles, sur le patron de
/// `journal_entries::list_by_company_paginated` : un `SELECT COUNT(*)` pour le
/// total, puis les lignes de la page.
///
/// `ORDER BY created_at DESC, id DESC` : deux entrées de la même milliseconde
/// gardent un ordre stable. Les entrées fusionnées d'une archive gardent leur
/// `created_at` d'origine, et se rangent donc à leur date réelle.
pub async fn list_by_company_paginated(
    pool: &MySqlPool,
    company_id: i64,
    query: AuditLogListQuery,
) -> Result<AuditLogListResult, DbError> {
    // Clamp défensif — la source de vérité est le handler de la route.
    let clamped_limit = query.limit.clamp(1, MAX_LIMIT);
    let clamped_offset = query.offset.max(0);

    let mut count_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new("SELECT COUNT(*) FROM audit_log");
    push_where_clauses(&mut count_qb, company_id, &query);

    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    let mut items_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new(format!("SELECT {COLUMNS} FROM audit_log"));
    push_where_clauses(&mut items_qb, company_id, &query);
    items_qb.push(" ORDER BY created_at DESC, id DESC LIMIT ");
    items_qb.push_bind(clamped_limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(clamped_offset);

    let items: Vec<AuditLogEntry> = items_qb
        .build_query_as::<AuditLogEntry>()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    Ok(AuditLogListResult {
        items,
        total,
        offset: clamped_offset,
        limit: clamped_limit,
    })
}

/// Lignes de l'export CSV — **mêmes filtres, même ordre** que la consultation,
/// bornées à `max_rows`, sans offset.
///
/// ⛔ **Aucune seconde écriture de la clause WHERE** : `push_where_clauses` est
/// partagée avec [`list_by_company_paginated`]. C'est ce qui garantit que
/// l'export et l'écran montrent les mêmes lignes.
pub async fn list_for_export(
    pool: &MySqlPool,
    company_id: i64,
    query: &AuditLogListQuery,
    max_rows: i64,
) -> Result<Vec<AuditLogEntry>, DbError> {
    let mut qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new(format!("SELECT {COLUMNS} FROM audit_log"));
    push_where_clauses(&mut qb, company_id, query);
    qb.push(" ORDER BY created_at DESC, id DESC LIMIT ");
    qb.push_bind(max_rows);

    qb.build_query_as::<AuditLogEntry>()
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

    // ------------------------------------------------------------------
    // Story 25-1c-a (refs #378) — la consultation paginée et scopée.
    //
    // ⛔ Identifiants DÉSALIGNÉS, comme ci-dessus : sociétés 30 et 40,
    // utilisateurs 501 (société 40) et 602 (société 30). Un scoping fautif qui
    // lirait l'`id` de l'utilisateur au lieu de sa société ne peut pas tomber
    // juste par coïncidence.
    //
    // ⚠️ `seed_actor` n'est PAS modifié — les trois tests de la 25-1c-zero en
    // dépendent. L'acteur de la société 30 est semé par un helper distinct.
    // ------------------------------------------------------------------

    /// Ajoute l'utilisateur 602 dans la société **30**, déjà semée par
    /// [`seed_actor`]. Rend `(company_id, user_id)`.
    async fn seed_second_actor(pool: &MySqlPool) -> (i64, i64) {
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, role, active, company_id) \
             VALUES (602, 'acteur-30', ?, 'Comptable', TRUE, 30)",
        )
        .bind(FAKE_HASH)
        .execute(pool)
        .await
        .expect("insert user 602");
        (30, 602)
    }

    /// Pose `created_at` d'une entrée à la valeur exacte voulue.
    ///
    /// ⚠️ **Par `UPDATE` explicite** : `CURRENT_TIMESTAMP(3)` place toutes les
    /// entrées à la même milliseconde ou presque, ce qui rend les bornes de
    /// date et l'ordre invérifiables.
    async fn set_created_at(pool: &MySqlPool, id: i64, at: &str) {
        let ts = chrono::NaiveDateTime::parse_from_str(at, "%Y-%m-%d %H:%M:%S%.3f")
            .expect("horodatage de test valide");
        sqlx::query("UPDATE audit_log SET created_at = ? WHERE id = ?")
            .bind(ts)
            .bind(id)
            .execute(pool)
            .await
            .expect("update created_at");
    }

    fn query_with(limit: i64) -> AuditLogListQuery {
        AuditLogListQuery {
            limit,
            ..Default::default()
        }
    }

    fn jour(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("date de test valide")
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn list_scopes_strictly_by_company(pool: MySqlPool) {
        let (company_40, user_501) = seed_actor(&pool).await;
        let (company_30, user_602) = seed_second_actor(&pool).await;

        let chez_40 = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "contact.created", "contact", 7, None),
        )
        .await;
        let chez_30 = insert(
            &pool,
            NewAuditLogEntry::user(user_602, "contact.created", "contact", 8, None),
        )
        .await;

        let page = list_by_company_paginated(&pool, company_40, query_with(50))
            .await
            .expect("consultation de la 40");
        let ids: Vec<i64> = page.items.iter().map(|e| e.id).collect();

        assert!(
            ids.contains(&chez_40.id),
            "l'entrée de la société consultée"
        );
        assert!(
            !ids.contains(&chez_30.id),
            "une entrée de la société 30 ne doit JAMAIS apparaître dans la consultation de la 40"
        );
        assert_eq!(page.total, 1);
        assert_eq!(company_30, 30);
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn list_excludes_entries_without_a_company(pool: MySqlPool) {
        let (company_40, user_501) = seed_actor(&pool).await;
        let (company_30, _user_602) = seed_second_actor(&pool).await;

        // Acteur inexistant ⇒ `company_id` NULL (cf. la 25-1c-zero).
        let sans_societe = insert(
            &pool,
            NewAuditLogEntry::user(777, "contact.created", "contact", 9, None),
        )
        .await;
        assert_eq!(
            sans_societe.company_id, None,
            "montage : entrée sans société"
        );

        let avec_societe = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "contact.created", "contact", 7, None),
        )
        .await;

        for company_id in [company_40, company_30] {
            let page = list_by_company_paginated(&pool, company_id, query_with(50))
                .await
                .expect("consultation");
            assert!(
                !page.items.iter().any(|e| e.id == sans_societe.id),
                "filtre STRICT : une entrée sans société n'appartient à aucune consultation \
                 (société {company_id})"
            );
        }

        let export = list_for_export(&pool, company_40, &query_with(50), 100)
            .await
            .expect("export");
        assert!(
            !export.iter().any(|e| e.id == sans_societe.id),
            "l'export applique le même filtre que la consultation"
        );
        assert!(export.iter().any(|e| e.id == avec_societe.id));
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn list_date_bounds_are_inclusive_to_the_millisecond(pool: MySqlPool) {
        let (company_40, user_501) = seed_actor(&pool).await;

        let pose = |suffixe: i64, at: &'static str| {
            let pool = pool.clone();
            async move {
                let e = insert(
                    &pool,
                    NewAuditLogEntry::user(user_501, "contact.created", "contact", suffixe, None),
                )
                .await;
                set_created_at(&pool, e.id, at).await;
                e.id
            }
        };

        let veille_fin = pose(1, "2026-09-14 23:59:59.999").await;
        let debut = pose(2, "2026-09-15 00:00:00.000").await;
        let fin = pose(3, "2026-09-15 23:59:59.999").await;
        let lendemain = pose(4, "2026-09-16 00:00:00.000").await;
        let dernier_jour = pose(5, "9999-12-31 23:59:59.999").await;

        let query = AuditLogListQuery {
            date_from: Some(jour("2026-09-15")),
            date_to: Some(jour("2026-09-15")),
            limit: 50,
            ..Default::default()
        };
        let page = list_by_company_paginated(&pool, company_40, query)
            .await
            .expect("consultation bornée");
        let ids: Vec<i64> = page.items.iter().map(|e| e.id).collect();

        assert!(
            ids.contains(&debut),
            "00:00:00.000 du jour `date_from` est INCLUS"
        );
        assert!(
            ids.contains(&fin),
            "23:59:59.999 du jour `date_to` est INCLUS"
        );
        assert!(
            !ids.contains(&veille_fin),
            "la veille à 23:59:59.999 est EXCLUE"
        );
        assert!(
            !ids.contains(&lendemain),
            "le lendemain à 00:00:00.000 est EXCLU"
        );

        // Dernier jour représentable : la borne ne calcule rien, donc ne déborde pas.
        let query_max = AuditLogListQuery {
            date_from: Some(jour("9999-12-31")),
            date_to: Some(jour("9999-12-31")),
            limit: 50,
            ..Default::default()
        };
        let page_max = list_by_company_paginated(&pool, company_40, query_max)
            .await
            .expect("consultation au dernier jour représentable");
        assert_eq!(
            page_max.items.iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![dernier_jour],
            "au 9999-12-31, l'entrée de 23:59:59.999 est rendue"
        );
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn list_filters_by_entity_type_entity_id_and_action(pool: MySqlPool) {
        let (company_40, user_501) = seed_actor(&pool).await;

        let contact_7 = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "contact.created", "contact", 7, None),
        )
        .await;
        let contact_8 = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "contact.updated", "contact", 8, None),
        )
        .await;
        let facture_7 = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "invoice.validated", "invoice", 7, None),
        )
        .await;

        let par_type = AuditLogListQuery {
            entity_type: Some("contact".into()),
            limit: 50,
            ..Default::default()
        };
        let ids = list_by_company_paginated(&pool, company_40, par_type)
            .await
            .expect("filtre type")
            .items
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>();
        assert!(ids.contains(&contact_7.id) && ids.contains(&contact_8.id));
        assert!(!ids.contains(&facture_7.id));

        let par_couple = AuditLogListQuery {
            entity_type: Some("contact".into()),
            entity_id: Some(7),
            limit: 50,
            ..Default::default()
        };
        let ids = list_by_company_paginated(&pool, company_40, par_couple)
            .await
            .expect("filtre couple")
            .items
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![contact_7.id]);

        let par_action = AuditLogListQuery {
            action: Some("invoice.validated".into()),
            limit: 50,
            ..Default::default()
        };
        let ids = list_by_company_paginated(&pool, company_40, par_action)
            .await
            .expect("filtre action")
            .items
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![facture_7.id]);
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn list_orders_by_created_at_then_id_descending(pool: MySqlPool) {
        let (company_40, user_501) = seed_actor(&pool).await;

        let ancienne = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "contact.created", "contact", 1, None),
        )
        .await;
        let meme_ms_1 = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "contact.created", "contact", 2, None),
        )
        .await;
        let meme_ms_2 = insert(
            &pool,
            NewAuditLogEntry::user(user_501, "contact.created", "contact", 3, None),
        )
        .await;

        set_created_at(&pool, ancienne.id, "2026-09-10 08:00:00.000").await;
        // Deux entrées à la MÊME milliseconde : seul `id DESC` les départage.
        set_created_at(&pool, meme_ms_1.id, "2026-09-11 08:00:00.000").await;
        set_created_at(&pool, meme_ms_2.id, "2026-09-11 08:00:00.000").await;

        let page = list_by_company_paginated(&pool, company_40, query_with(50))
            .await
            .expect("consultation");

        assert_eq!(
            page.items.iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![meme_ms_2.id, meme_ms_1.id, ancienne.id],
            "created_at DESC, puis id DESC à égalité de milliseconde"
        );
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn list_paginates_with_a_total_independent_of_the_page(pool: MySqlPool) {
        let (company_40, user_501) = seed_actor(&pool).await;

        let mut ids = Vec::new();
        for i in 1..=5 {
            let e = insert(
                &pool,
                NewAuditLogEntry::user(user_501, "contact.created", "contact", i, None),
            )
            .await;
            set_created_at(&pool, e.id, &format!("2026-09-1{i} 08:00:00.000")).await;
            ids.push(e.id);
        }
        ids.reverse(); // ordre attendu : du plus récent au plus ancien

        let page1 = list_by_company_paginated(
            &pool,
            company_40,
            AuditLogListQuery {
                limit: 2,
                ..Default::default()
            },
        )
        .await
        .expect("page 1");
        assert_eq!(
            page1.total, 5,
            "le total ne dépend ni de `limit` ni d'`offset`"
        );
        assert_eq!(
            page1.items.iter().map(|e| e.id).collect::<Vec<_>>(),
            ids[..2]
        );

        let page2 = list_by_company_paginated(
            &pool,
            company_40,
            AuditLogListQuery {
                limit: 2,
                offset: 2,
                ..Default::default()
            },
        )
        .await
        .expect("page 2");
        assert_eq!(page2.total, 5);
        assert_eq!(
            page2.items.iter().map(|e| e.id).collect::<Vec<_>>(),
            ids[2..4]
        );

        // Clamp défensif : `limit` hors bornes est ramené dans [1, MAX_LIMIT].
        let trop_grand = list_by_company_paginated(
            &pool,
            company_40,
            AuditLogListQuery {
                limit: MAX_LIMIT + 1_000,
                offset: -5,
                ..Default::default()
            },
        )
        .await
        .expect("clamp");
        assert_eq!(trop_grand.limit, MAX_LIMIT);
        assert_eq!(trop_grand.offset, 0);

        let trop_petit = list_by_company_paginated(
            &pool,
            company_40,
            AuditLogListQuery {
                limit: 0,
                ..Default::default()
            },
        )
        .await
        .expect("clamp bas");
        assert_eq!(trop_petit.limit, 1);
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn export_returns_the_same_rows_as_the_unpaginated_list(pool: MySqlPool) {
        let (company_40, user_501) = seed_actor(&pool).await;

        for i in 1..=4 {
            let e = insert(
                &pool,
                NewAuditLogEntry::user(user_501, "contact.created", "contact", i, None),
            )
            .await;
            set_created_at(&pool, e.id, &format!("2026-09-1{i} 08:00:00.000")).await;
        }

        let consultation = list_by_company_paginated(&pool, company_40, query_with(MAX_LIMIT))
            .await
            .expect("consultation non paginée");
        let export = list_for_export(&pool, company_40, &query_with(MAX_LIMIT), 100)
            .await
            .expect("export");

        assert_eq!(
            export.iter().map(|e| e.id).collect::<Vec<_>>(),
            consultation.items.iter().map(|e| e.id).collect::<Vec<_>>(),
            "mêmes lignes, même ordre — c'est ce que garantit la clause WHERE partagée"
        );

        let borne = list_for_export(&pool, company_40, &query_with(MAX_LIMIT), 2)
            .await
            .expect("export borné");
        assert_eq!(borne.len(), 2, "`max_rows` borne l'export");
    }
}
