//! Bootstrap : création automatique d'un utilisateur admin au premier
//! démarrage (FR3 — installation < 15 min).
//!
//! Appelé depuis `main.rs` après l'exécution des migrations. Idempotent
//! et tolérant aux race conditions (démarrage concurrent de plusieurs
//! instances contre la même DB).

use kesh_db::entities::{NewUser, Role};
use kesh_db::errors::DbError;
use kesh_db::repositories::users;
use sqlx::MySqlPool;

use crate::auth::password::hash_password_async;
use crate::config::Config;
use crate::errors::AppError;

/// Vérifie si la table `users` est vide, et si oui crée un compte admin
/// à partir de `KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD`.
///
/// Idempotent : appelé plusieurs fois, n'écrase jamais un user existant.
/// Tolérant aux races : si une autre instance a bootstrappé entre notre
/// `COUNT(*)` et notre `INSERT`, la branche `UniqueConstraintViolation`
/// est traitée comme succès silencieux.
pub async fn ensure_admin_user(pool: &MySqlPool, config: &Config) -> Result<(), AppError> {
    // Story 6.2: Check if companies exist before creating a user
    // (users.company_id is NOT NULL, so at least one company must exist)
    let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap company count: {e}")))?;

    if company_count == 0 {
        tracing::warn!(
            "⚠️  bootstrap: no company exists yet, skipping admin user creation (complete onboarding to create company + admin)"
        );
        return Ok(());
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap count: {e}")))?;

    if count > 0 {
        tracing::info!(existing_users = count, "bootstrap: users déjà initialisés");
        return Ok(());
    }

    let hash = hash_password_async(config.admin_password.clone()).await?;

    // Get the first company to assign to the bootstrap admin
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap get first company: {e}")))?;

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
    async fn bootstrap_creates_admin_on_empty_db(pool: MySqlPool) {
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
    async fn bootstrap_skips_silently_when_no_company_exists(pool: MySqlPool) {
        // Story 6.2: If no company exists, bootstrap should skip and return Ok (T0.2 Option A)
        // This verifies the idempotent behavior: API boots without error even if onboarding hasn't created a company yet

        let config = test_config();

        // Call ensure_admin_user on empty DB (no companies)
        let result = ensure_admin_user(&pool, &config).await;

        assert!(
            result.is_ok(),
            "bootstrap should not error when no company exists"
        );

        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("count should succeed");
        assert_eq!(
            user_count, 0,
            "admin user should not be created if no company exists"
        );
    }

    // NOTE: la branche `DbError::UniqueConstraintViolation` du step 3 est
    // défensive — elle couvre une TOCTOU race entre le COUNT et l'INSERT
    // concurrent depuis une autre instance. Non testable déterministiquement
    // en mono-thread (il faudrait mocker le pool SQLx ou injecter un délai).
    // Validée par revue de code uniquement. Cf. Dev Notes story 1.5.
}
