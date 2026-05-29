//! Bootstrap : création automatique d'un utilisateur admin au premier
//! démarrage (FR3 — installation < 15 min).
//!
//! Appelé depuis `main.rs` après l'exécution des migrations. Idempotent
//! et tolérant aux race conditions (démarrage concurrent de plusieurs
//! instances contre la même DB).

use kesh_db::entities::{Language, NewUser, OrgType, Role};
use kesh_db::errors::DbError;
use kesh_db::repositories::users;
use sqlx::MySqlPool;

use crate::auth::password::hash_password_async;
use crate::config::Config;
use crate::errors::AppError;

/// Valeurs placeholder d'une company stub. Partagées entre le bootstrap
/// (DB vide, Story v011-2) et le wizard onboarding (`ensure_company_with_language`
/// quand aucune company n'existe) pour éviter une divergence (DRY). Le wizard
/// repasse `is_stub = FALSE` quand l'utilisateur renseigne ses vraies
/// coordonnées (`set_coordinates`).
pub(crate) const STUB_COMPANY_NAME: &str = "(en cours de configuration)";
pub(crate) const STUB_COMPANY_ADDRESS: &str = "-";

/// Crée le compte admin initial (`KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD`)
/// quand la table `users` est vide.
///
/// Story v011-2 (fix catch-22 #120) : sur fresh install où `companies` ET
/// `users` sont vides, crée aussi une **company stub** (`is_stub = TRUE`) à
/// laquelle rattacher l'admin (sinon `users.company_id NOT NULL` empêche la
/// création et le wizard d'onboarding, gardé par auth, est inatteignable).
/// Si une company existe déjà sans user (partial state), l'admin est rattaché
/// à cette company existante sans créer de stub.
///
/// Idempotent : appelé plusieurs fois, n'écrase jamais un user existant et ne
/// crée pas de company en double (la garde `user_count > 0` court-circuite).
/// Tolérant aux races : si une autre instance a bootstrappé entre notre
/// `COUNT(*)` et notre `INSERT`, la branche `UniqueConstraintViolation`
/// est traitée comme succès silencieux.
pub async fn ensure_admin_user(pool: &MySqlPool, config: &Config) -> Result<(), AppError> {
    // Story v011-2 (fix catch-22 onboarding, Issue #120) : on lit d'abord les
    // deux compteurs. La règle d'origine (Story 6.2) skippait la création admin
    // quand aucune company n'existait, mais le wizard d'onboarding exige une
    // auth → deadlock sur fresh install. On crée donc une company stub + admin
    // quand TOUT est vide.
    let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap company count: {e}")))?;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap user count: {e}")))?;

    if user_count > 0 {
        tracing::info!(
            existing_users = user_count,
            "bootstrap: users déjà initialisés"
        );
        return Ok(());
    }

    // user_count == 0 à partir d'ici.
    let company_id: i64 = if company_count == 0 {
        // Fresh install : créer une company stub minimaliste (is_stub=TRUE).
        // Le wizard la complétera ; `set_coordinates` repassera is_stub=FALSE.
        // Valeurs placeholder satisfaisant les CHECK NOT NULL de `companies`.
        let result = sqlx::query(
            "INSERT INTO companies \
             (name, address, org_type, accounting_language, instance_language, is_stub) \
             VALUES (?, ?, ?, ?, ?, TRUE)",
        )
        .bind(STUB_COMPANY_NAME)
        .bind(STUB_COMPANY_ADDRESS)
        .bind(OrgType::Independant)
        .bind(Language::Fr)
        .bind(Language::Fr)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap create stub company: {e}")))?;

        let stub_id = i64::try_from(result.last_insert_id()).map_err(|_| {
            AppError::Internal("bootstrap stub company last_insert_id dépasse i64::MAX".into())
        })?;

        tracing::info!(
            stub_company_id = stub_id,
            "bootstrap: company stub créée (DB vide). Compléter l'onboarding via l'UI pour renommer/configurer."
        );
        stub_id
    } else {
        // Partial state : une company existe (ex. créée par le wizard) mais
        // aucun user → créer l'admin sur la company existante (pas de nouveau stub).
        sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Internal(format!("bootstrap get first company: {e}")))?
    };

    let hash = hash_password_async(config.admin_password.clone()).await?;

    let result = users::create(
        pool,
        NewUser {
            username: config.admin_username.clone(),
            password_hash: hash,
            role: Role::Admin,
            active: true,
            company_id,
        },
    )
    .await;

    match result {
        Ok(_) => {
            tracing::info!(
                username = %config.admin_username,
                "bootstrap: utilisateur admin créé — CHANGEZ LE MOT DE PASSE"
            );
        }
        Err(DbError::UniqueConstraintViolation(_)) => {
            // Race condition : une autre instance a bootstrapp entre notre
            // COUNT et notre INSERT. Branche défensive, non testable
            // déterministiquement en mono-thread.
            tracing::info!("bootstrap: admin créé en parallèle par un autre process");

            // Story v011-2 (code-review Pass 1) : si on a créé une company stub
            // ce boot (`company_count == 0`) mais perdu la race sur l'admin, notre
            // stub est orpheline (aucun user attaché car l'INSERT admin a échoué)
            // → la supprimer pour ne pas laisser de company stub en double.
            // DELETE sûr : aucune FK entrante sur un fresh boot (ni user, ni
            // account). Non-fatal : un échec de cleanup ne doit pas tuer le boot.
            if company_count == 0 {
                match sqlx::query("DELETE FROM companies WHERE id = ?")
                    .bind(company_id)
                    .execute(pool)
                    .await
                {
                    Ok(_) => tracing::info!(
                        orphan_company_id = company_id,
                        "bootstrap: company stub orpheline supprimée après race admin"
                    ),
                    Err(e) => tracing::warn!(
                        orphan_company_id = company_id,
                        error = %e,
                        "bootstrap: échec suppression company stub orpheline après race (non-fatal)"
                    ),
                }
            }
        }
        Err(other) => return Err(AppError::Database(other)),
    }

    // Patch #11 + V4 : post-bootstrap sanity check — si deux instances ont
    // démarré en parallèle avec des `KESH_ADMIN_USERNAME` différents
    // (ex. deployment mistake), les deux INSERTs réussissent et on se
    // retrouve avec plusieurs admins. Cette branche n'est PAS couverte
    // par l'handling `UniqueConstraintViolation` ci-dessus. On loggue
    // un warning explicite pour alerter l'opérateur.
    //
    // **Patch V4** : si le SELECT COUNT lui-même échoue (rupture DB
    // transitoire juste après l'INSERT réussi), on NE doit PAS faire
    // échouer le bootstrap — l'admin vient d'être créé avec succès, et
    // refuser de démarrer ici mettrait le serveur en boucle d'exit(1)
    // alors que la DB va revenir. On loggue simplement un warning et
    // on retourne Ok.
    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
    {
        Ok(final_count) if final_count > 1 => {
            tracing::warn!(
                users_count = final_count,
                "bootstrap: plusieurs utilisateurs existent après bootstrap. \
                 Déploiement concurrent avec config divergente ? \
                 Vérifiez que tous les replicas utilisent le même KESH_ADMIN_USERNAME."
            );
        }
        Ok(_) => {}
        Err(e) => {
            // Le sanity check est informatif, pas structurel. Son échec
            // ne doit pas tuer le démarrage après un INSERT réussi.
            tracing::warn!(
                error = %e,
                "bootstrap: sanity check post-insert a échoué (non-fatal)"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_helpers::make_test_config;

    /// Construit un `Config` de test sans passer par les variables
    /// d'environnement (évite la contention parallèle avec les tests
    /// `config::tests`).
    fn test_config() -> Config {
        make_test_config("admin", "test-bootstrap-password")
    }

    #[sqlx::test(migrator = "kesh_db::MIGRATOR")]
    async fn bootstrap_creates_admin_on_existing_company(pool: MySqlPool) {
        // Story v011-2 (Issue #120) cas (c) — partial state : une company existe
        // déjà (ex. créée par le wizard) mais aucun user. Le bootstrap crée l'admin
        // sur la company existante, sans créer de nouveau stub.
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("Test Company")
        .bind("123 Test St")
        .bind("Independant")
        .bind("FR")
        .bind("FR")
        .execute(&pool)
        .await
        .expect("company insert should succeed");

        let config = test_config();

        ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap should succeed");

        let users: Vec<(i64, String, String, bool)> =
            sqlx::query_as("SELECT id, username, role, active FROM users")
                .fetch_all(&pool)
                .await
                .expect("select should succeed");

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].1, "admin");
        assert_eq!(users[0].2, "Admin");
        assert!(users[0].3);

        // Pas de nouveau stub : la company préexistante reste unique et is_stub=FALSE.
        let companies: Vec<(bool,)> = sqlx::query_as("SELECT is_stub FROM companies")
            .fetch_all(&pool)
            .await
            .expect("select companies should succeed");
        assert_eq!(
            companies.len(),
            1,
            "partial state must not create a new stub company"
        );
        assert!(
            !companies[0].0,
            "existing company must not be marked is_stub"
        );
    }

    #[sqlx::test(migrator = "kesh_db::MIGRATOR")]
    async fn bootstrap_is_idempotent_on_repeated_calls(pool: MySqlPool) {
        // Create a company first (required by users.company_id FK)
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("Test Company")
        .bind("123 Test St")
        .bind("Independant")
        .bind("FR")
        .bind("FR")
        .execute(&pool)
        .await
        .expect("company insert should succeed");

        let config = test_config();

        ensure_admin_user(&pool, &config)
            .await
            .expect("first bootstrap should succeed");
        ensure_admin_user(&pool, &config)
            .await
            .expect("second bootstrap should succeed");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("count should succeed");
        assert_eq!(count, 1, "should not duplicate admin on repeated calls");
    }

    #[sqlx::test(migrator = "kesh_db::MIGRATOR")]
    async fn bootstrap_skips_if_users_already_exist(pool: MySqlPool) {
        // Create a company first (required by FK)
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("Test Company")
        .bind("123 Test St")
        .bind("Independant")
        .bind("FR")
        .bind("FR")
        .execute(&pool)
        .await
        .expect("company insert should succeed");

        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("get company_id should succeed");

        // Insérer manuellement un user arbitraire (pas admin)
        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("alice")
        .bind("$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo")
        .bind("Comptable")
        .bind(true)
        .bind(company_id)
        .execute(&pool)
        .await
        .expect("pre-insert should succeed");

        let config = test_config();
        ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap should succeed");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("count should succeed");
        assert_eq!(count, 1, "should not create admin if users already exist");

        let usernames: Vec<String> = sqlx::query_scalar("SELECT username FROM users")
            .fetch_all(&pool)
            .await
            .expect("select should succeed");
        assert_eq!(usernames, vec!["alice".to_string()]);
    }

    #[sqlx::test(migrator = "kesh_db::MIGRATOR")]
    async fn bootstrap_creates_stub_and_admin_on_empty_db(pool: MySqlPool) {
        // Story v011-2 (Issue #120) cas (a) — fresh install : sur DB vide (ni company
        // ni user), le bootstrap crée une company stub (is_stub=TRUE) ET l'admin du
        // `.env` attaché. Casse le catch-22 (pas d'admin sans company, pas de company
        // sans auth). Remplace l'ancien comportement Story 6.2 (skip silencieux), dont
        // le test affirmait l'inverse — l'assertion est donc inversée.
        let config = test_config();

        ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap should succeed on empty DB");

        // Exactement 1 company, marquée stub.
        let companies: Vec<(i64, bool)> = sqlx::query_as("SELECT id, is_stub FROM companies")
            .fetch_all(&pool)
            .await
            .expect("select companies should succeed");
        assert_eq!(
            companies.len(),
            1,
            "exactly one stub company must be created"
        );
        assert!(
            companies[0].1,
            "bootstrap company must be marked is_stub=TRUE"
        );

        // Exactement 1 admin, attaché à la company stub.
        let users: Vec<(String, String, i64)> =
            sqlx::query_as("SELECT username, role, company_id FROM users")
                .fetch_all(&pool)
                .await
                .expect("select users should succeed");
        assert_eq!(users.len(), 1, "exactly one admin must be created");
        assert_eq!(users[0].0, "admin");
        assert_eq!(users[0].1, "Admin");
        assert_eq!(
            users[0].2, companies[0].0,
            "admin must be attached to the stub company"
        );
    }

    #[sqlx::test(migrator = "kesh_db::MIGRATOR")]
    async fn bootstrap_idempotent_on_empty_db(pool: MySqlPool) {
        // Story v011-2 (Issue #120) cas (b) — deux appels sur DB vide ne doivent
        // créer qu'une seule company stub et un seul admin (idempotence fresh-install).
        let config = test_config();

        ensure_admin_user(&pool, &config)
            .await
            .expect("first bootstrap should succeed");
        ensure_admin_user(&pool, &config)
            .await
            .expect("second bootstrap should succeed");

        let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
            .fetch_one(&pool)
            .await
            .expect("count companies should succeed");
        assert_eq!(
            company_count, 1,
            "no duplicate stub company on repeated calls"
        );

        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("count users should succeed");
        assert_eq!(user_count, 1, "no duplicate admin on repeated calls");

        // Code-review Pass 1 : la company stub reste marquée is_stub=TRUE après le
        // 2e appel (le guard `user_count > 0` court-circuite sans toucher companies),
        // et l'admin reste attaché à cette même company.
        let (company_id, is_stub): (i64, bool) =
            sqlx::query_as("SELECT id, is_stub FROM companies ORDER BY id LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("select stub company should succeed");
        assert!(
            is_stub,
            "stub company must stay is_stub=TRUE across repeated calls"
        );
        let admin_company_id: i64 = sqlx::query_scalar("SELECT company_id FROM users LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("select admin company_id should succeed");
        assert_eq!(
            admin_company_id, company_id,
            "admin must stay attached to the stub company"
        );
    }

    // NOTE: la branche `DbError::UniqueConstraintViolation` du step 3 est
    // défensive — elle couvre une TOCTOU race entre le COUNT et l'INSERT
    // concurrent depuis une autre instance. Non testable déterministiquement
    // en mono-thread (il faudrait mocker le pool SQLx ou injecter un délai).
    // Validée par revue de code uniquement. Cf. Dev Notes story 1.5.
}
