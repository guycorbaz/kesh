//! Bootstrap : matrice 6 cas (Story v011-5) — création/recovery du compte
//! admin selon l'état de la DB et la présence des vars `KESH_ADMIN_*`.
//!
//! Appelé depuis `main.rs` après l'exécution des migrations. Idempotent
//! et tolérant aux race conditions (démarrage concurrent de plusieurs
//! instances contre la même DB).
//!
//! **Matrice 6 cas** (cf. story v011-5 Scope) :
//!
//! | # | users | has_admin_env | comportement                                                  |
//! |---|-------|---------------|---------------------------------------------------------------|
//! | 1 | 0     | false         | INSERT stub company seule ; admin créé via `/setup` web        |
//! | 2 | 0     | true          | INSERT stub + admin (bootstrap déclaratif, ≡ v011-2)           |
//! | 3 | > 0   | false         | no-op (régime nominal post-bootstrap)                          |
//! | 4 | > 0   | true, match user, hash identique | no-op silencieux + warn « retirer les vars » |
//! | 5 | > 0   | true, match user, hash diff | **RECOVERY** : warn préventif + tx atomique UPDATE+audit_log + revoke_all + error! |
//! | 6 | > 0   | true, no match  | no-op + warn « no user matches KESH_ADMIN_USERNAME=<x> »      |
//!
//! Retourne le `user_count` post-bootstrap (utilisé par `main.rs` pour
//! initialiser `AppState::users_exist`).

use kesh_db::entities::{Language, NewAuditLogEntry, NewUser, OrgType, Role};
use kesh_db::errors::DbError;
use kesh_db::repositories::{audit_log, refresh_tokens, users};
use sqlx::MySqlPool;

use crate::auth::password::{hash_password_async, verify_password_async};
use crate::config::Config;
use crate::errors::AppError;

/// Valeurs placeholder d'une company stub. Partagées entre le bootstrap
/// (DB vide, Story v011-2) et le wizard onboarding (`ensure_company_with_language`
/// quand aucune company n'existe) pour éviter une divergence (DRY). Le wizard
/// repasse `is_stub = FALSE` quand l'utilisateur renseigne ses vraies
/// coordonnées (`set_coordinates`).
pub(crate) const STUB_COMPANY_NAME: &str = "(en cours de configuration)";
pub(crate) const STUB_COMPANY_ADDRESS: &str = "-";

/// Matrice 6 cas du bootstrap admin v011-5.
///
/// Retourne le `user_count` post-bootstrap (utilisé par `main.rs` pour
/// initialiser `AppState::users_exist`).
///
/// **Détection `has_admin_env`** : `Some` ET non-vide pour les deux vars.
/// L'invariant `Config::from_env` (story v011-5 AC #0) garantit
/// `Some(s) ⟹ !s.is_empty()` ; le double-check `.is_empty()` est défensif.
///
/// **Lecture unique** des compteurs `company_count` et `user_count` au
/// début, partagée par toutes les branches (cleanup orphan stub race cas 2
/// inclus).
pub async fn ensure_admin_user(pool: &MySqlPool, config: &Config) -> Result<i64, AppError> {
    let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap company count: {e}")))?;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap user count: {e}")))?;

    let has_admin_env = config
        .admin_username
        .as_deref()
        .is_some_and(|u| !u.is_empty())
        && config
            .admin_password
            .as_deref()
            .is_some_and(|p| !p.is_empty());

    match (user_count, has_admin_env) {
        // Cas 1 — DB vide + pas d'env : créer la stub company seule.
        // L'admin sera créé via `POST /api/v1/setup/admin` (flow web).
        (0, false) => {
            insert_stub_company(pool).await?;
            tracing::info!(
                "bootstrap: setup-required — créer l'admin via POST /api/v1/setup/admin"
            );
            // user_count post-bootstrap reste 0 (le setup-UI fera l'INSERT).
            Ok(0)
        }
        // Cas 2 — DB vide + env set : comportement v011-2 préservé.
        (0, true) => {
            // SAFETY (invariant from_env): admin_username/password Some non-empty.
            let admin_username = config
                .admin_username
                .as_ref()
                .expect("has_admin_env ⟹ admin_username Some non-vide")
                .clone();
            let admin_password = config
                .admin_password
                .as_ref()
                .expect("has_admin_env ⟹ admin_password Some non-vide")
                .clone();

            // Choix de la company target : si DB est COMPLÈTEMENT vide
            // (`company_count == 0`), créer une stub ; sinon (partial state :
            // company existe sans user, ex. wizard onboarding interrompu),
            // attacher l'admin à la company existante (préservation v011-2).
            let (company_id, created_stub_this_boot): (i64, bool) = if company_count == 0 {
                let stub_id = insert_stub_company(pool).await?;
                tracing::info!(
                    stub_company_id = stub_id,
                    "bootstrap: company stub créée (DB vide). Compléter l'onboarding via l'UI."
                );
                (stub_id, true)
            } else {
                let existing_id: i64 =
                    sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
                        .fetch_one(pool)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("bootstrap get first company: {e}"))
                        })?;
                tracing::info!(
                    existing_company_id = existing_id,
                    "bootstrap: company préexistante, admin rattaché (partial state)"
                );
                (existing_id, false)
            };

            let hash = hash_password_async(admin_password).await?;
            let result = users::create(
                pool,
                NewUser {
                    username: admin_username.clone(),
                    password_hash: hash,
                    role: Role::Admin,
                    active: true,
                    company_id,
                    // Bootstrap par env vars : pas d'email (Story 17-4a).
                    // L'admin pourra le renseigner via /users ou re-setup.
                    email: None,
                },
            )
            .await;

            match result {
                Ok(_) => {
                    tracing::info!(
                        username = %admin_username,
                        "bootstrap: utilisateur admin créé — CHANGEZ LE MOT DE PASSE"
                    );
                    Ok(1)
                }
                Err(DbError::UniqueConstraintViolation(_)) => {
                    // Race condition : autre instance a bootstrappé.
                    tracing::info!("bootstrap: admin créé en parallèle par un autre process");
                    // Cleanup orphan stub (cas 2 race v011-2 Pass 1) — uniquement
                    // si on a créé la stub CE boot. Si on a attaché à une
                    // company préexistante, ne PAS la supprimer.
                    if created_stub_this_boot {
                        if let Err(e) = sqlx::query("DELETE FROM companies WHERE id = ?")
                            .bind(company_id)
                            .execute(pool)
                            .await
                        {
                            tracing::warn!(
                                orphan_company_id = company_id,
                                error = %e,
                                "bootstrap: échec suppression company stub orpheline (non-fatal)"
                            );
                        } else {
                            tracing::info!(
                                orphan_company_id = company_id,
                                "bootstrap: company stub orpheline supprimée après race admin"
                            );
                        }
                    }
                    // user_count final inconnu (l'autre instance peut avoir
                    // créé 1+ users) → relit la DB.
                    refresh_user_count(pool).await
                }
                Err(other) => Err(AppError::Database(other)),
            }
        }
        // Cas 3 — users > 0 + pas d'env : régime nominal post-bootstrap.
        (n, false) => {
            tracing::info!(existing_users = n, "bootstrap: régime nominal (skip)");
            Ok(n)
        }
        // Cas 4/5/6 — users > 0 + env set : recovery / no-op / no-match.
        (n, true) => {
            // SAFETY (invariant from_env): admin_username/password Some non-empty.
            let admin_username = config
                .admin_username
                .as_ref()
                .expect("has_admin_env ⟹ admin_username Some non-vide")
                .as_str();
            let admin_password = config
                .admin_password
                .as_ref()
                .expect("has_admin_env ⟹ admin_password Some non-vide")
                .clone();

            let user_opt = users::find_by_username(pool, admin_username).await?;

            match user_opt {
                // Cas 6 — env set mais aucun user ne matche le username.
                None => {
                    tracing::warn!(
                        kesh_admin_username = %admin_username,
                        "bootstrap: no user matches KESH_ADMIN_USERNAME, recovery skipped"
                    );
                    Ok(n)
                }
                Some(u) => {
                    // Verify Argon2 — sépare cas 4 (hash identique) du cas 5
                    // (hash diff = recovery).
                    let hash_matches =
                        verify_password_async(admin_password.clone(), u.password_hash.clone())
                            .await?;

                    if hash_matches {
                        // Cas 4 — hash identique : no-op silencieux (préserve
                        // l'idempotence des reboots avec .env non purgé).
                        tracing::warn!(
                            username = %u.username,
                            "bootstrap: vars .env présentes mais hash inchangé — RETIRER LES VARS DE .ENV"
                        );
                        Ok(n)
                    } else {
                        // Cas 5 — RECOVERY BREAK-GLASS.
                        // Warning préventif AVANT toute mutation (Q1 Option B).
                        tracing::warn!(
                            username = %u.username,
                            "⚠️ Recovery break-glass déclenché pour user '{}' — si vous avez \
                             changé votre mot de passe via l'UI, votre mdp sera écrasé par \
                             KESH_ADMIN_PASSWORD. Pour annuler : Ctrl-C + retirer la var de .env + redémarrer.",
                            u.username
                        );

                        // Hash du nouveau password (hors transaction, calcul CPU).
                        let new_hash = hash_password_async(admin_password).await?;

                        // Transaction atomique : UPDATE password + INSERT audit_log.
                        // Rollback si l'un des deux échoue → password inchangé.
                        let mut tx = pool
                            .begin()
                            .await
                            .map_err(|e| AppError::Internal(format!("recovery tx begin: {e}")))?;

                        sqlx::query("UPDATE users SET password_hash = ? WHERE username = ?")
                            .bind(&new_hash)
                            .bind(&u.username)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| {
                                AppError::Internal(format!("recovery UPDATE password: {e}"))
                            })?;

                        audit_log::insert_in_tx(
                            &mut tx,
                            // Story 17-2a — bootstrap (pas de CurrentUser ni de PAT) → ::user.
                            NewAuditLogEntry::user(
                                u.id,
                                "admin_break_glass_reset",
                                "user",
                                u.id,
                                Some(serde_json::json!({
                                    "username": u.username,
                                    "trigger": "env_vars_present_hash_diff",
                                })),
                            ),
                        )
                        .await?;

                        tx.commit()
                            .await
                            .map_err(|e| AppError::Internal(format!("recovery tx commit: {e}")))?;

                        // Post-commit best-effort : revoke refresh tokens + error! log final.
                        // `let _ = ...` ignore l'erreur (limitation L2 documentée).
                        // Best-effort revoke (cf. L2 limitation v0.2). On loggue
                        // l'erreur pour traçabilité plutôt que `let _ = ...` qui
                        // ferait perdre le diagnostic en cas de revoke fail.
                        //
                        // **Reason `"password_change"`** : la contrainte CHECK
                        // `chk_refresh_tokens_revoked_reason` (migration
                        // `20260406000001`) limite à 5 valeurs whitelist
                        // {logout, rotation, password_change, admin_disable,
                        // theft_detected}. Recovery break-glass = changement de
                        // password admin → mapping sémantique le plus proche.
                        // Le motif détaillé est dans `audit_log.action = "admin_break_glass_reset"`
                        // (VARCHAR libre). Évite une migration `ALTER CHECK`
                        // dédiée pour v0.1.2.
                        match refresh_tokens::revoke_all_for_user(pool, u.id, "password_change")
                            .await
                        {
                            Ok(count) => tracing::info!(
                                revoked_count = count,
                                "bootstrap: refresh tokens révoqués post-recovery"
                            ),
                            Err(e) => tracing::warn!(
                                error = %e,
                                "bootstrap: échec revoke refresh tokens post-recovery (non-fatal)"
                            ),
                        }

                        tracing::error!(
                            username = %u.username,
                            "🔓 Recovery effectué — RETIRER LES VARS DE .ENV"
                        );

                        Ok(n)
                    }
                }
            }
        }
    }
}

/// INSERT d'une company stub (`is_stub = TRUE`). Helper partagé entre cas
/// 1 et cas 2. Renvoie l'ID de la company stub créée.
async fn insert_stub_company(pool: &MySqlPool) -> Result<i64, AppError> {
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

    i64::try_from(result.last_insert_id()).map_err(|_| {
        AppError::Internal("bootstrap stub company last_insert_id dépasse i64::MAX".into())
    })
}

/// Re-SELECT du user_count post-INSERT pour le cas 2 race.
async fn refresh_user_count(pool: &MySqlPool) -> Result<i64, AppError> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("bootstrap final user count: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::password::hash_password_async;
    use crate::config::test_helpers::make_test_config;

    /// Story v011-5 — Config avec env admin vars présentes (cas 2/4/5/6).
    fn test_config_with_env() -> Config {
        make_test_config("admin", "test-bootstrap-password")
    }

    /// Story v011-5 — Config SANS env admin vars (cas 1/3).
    fn test_config_no_env() -> Config {
        let mut c = make_test_config("admin", "test-bootstrap-password");
        c.admin_username = None;
        c.admin_password = None;
        c
    }

    /// Cas 1 (NEW v011-5) — DB vide + pas d'env : ne crée QUE la stub company,
    /// pas d'admin.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_db_empty_no_env_creates_stub_only(pool: MySqlPool) {
        let config = test_config_no_env();

        let count = ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap should succeed on empty DB without env");
        assert_eq!(count, 0, "no admin should be created");

        // 1 stub company, pas d'user.
        let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
            .fetch_one(&pool)
            .await
            .expect("count companies");
        assert_eq!(company_count, 1, "exactly one stub company must be created");

        let is_stub: bool = sqlx::query_scalar("SELECT is_stub FROM companies LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("select is_stub");
        assert!(is_stub, "company must be marked is_stub=TRUE");

        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("count users");
        assert_eq!(user_count, 0, "no admin must be created (setup-required)");
    }

    /// Cas 2 — DB vide + env set : stub + admin (v011-2 préservé).
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_db_empty_with_env_creates_stub_and_admin(pool: MySqlPool) {
        let config = test_config_with_env();

        let count = ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap should succeed on empty DB with env");
        assert_eq!(count, 1, "one admin must be created");

        // 1 company stub, 1 admin attaché.
        let companies: Vec<(i64, bool)> = sqlx::query_as("SELECT id, is_stub FROM companies")
            .fetch_all(&pool)
            .await
            .expect("select companies");
        assert_eq!(companies.len(), 1);
        assert!(companies[0].1);

        let users: Vec<(String, String, i64)> =
            sqlx::query_as("SELECT username, role, company_id FROM users")
                .fetch_all(&pool)
                .await
                .expect("select users");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].0, "admin");
        assert_eq!(users[0].1, "Admin");
        assert_eq!(users[0].2, companies[0].0);
    }

    /// Cas 2 bis — bootstrap idempotent on empty DB avec env.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_idempotent_on_empty_db(pool: MySqlPool) {
        let config = test_config_with_env();

        ensure_admin_user(&pool, &config)
            .await
            .expect("first bootstrap");
        ensure_admin_user(&pool, &config)
            .await
            .expect("second bootstrap");

        let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(company_count, 1, "no duplicate stub");

        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(user_count, 1, "no duplicate admin");
    }

    /// Cas 2 ter — partial state : une company existe déjà mais aucun user.
    /// Le bootstrap crée l'admin sur la company existante.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_creates_admin_on_existing_company(pool: MySqlPool) {
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
        .expect("company insert");

        let config = test_config_with_env();

        let count = ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap should succeed");
        assert_eq!(count, 1);

        let users: Vec<(String, String, bool)> =
            sqlx::query_as("SELECT username, role, active FROM users")
                .fetch_all(&pool)
                .await
                .expect("select");

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].0, "admin");
        assert_eq!(users[0].1, "Admin");
        assert!(users[0].2);

        // Pas de nouveau stub : la company préexistante reste unique.
        let companies_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
            .fetch_one(&pool)
            .await
            .expect("count companies");
        assert_eq!(companies_count, 1);
    }

    /// Cas 3 — users > 0 + pas d'env : no-op.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_users_exist_no_env_noop(pool: MySqlPool) {
        // Seed : 1 company + 1 user pré-existant.
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
        .expect("company insert");

        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("get company_id");

        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("alice")
        .bind("$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo")
        .bind("Comptable")
        .bind(true)
        .bind(company_id)
        .execute(&pool)
        .await
        .expect("pre-insert user");

        let config = test_config_no_env();
        let count = ensure_admin_user(&pool, &config).await.expect("bootstrap");
        assert_eq!(count, 1);

        let usernames: Vec<String> = sqlx::query_scalar("SELECT username FROM users")
            .fetch_all(&pool)
            .await
            .expect("select");
        assert_eq!(usernames, vec!["alice".to_string()]);
    }

    /// Cas 4 (NEW v011-5) — recovery hash identique : no-op silencieux.
    /// Le user existe, env set, hash matche → pas de modification DB.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_recovery_same_hash_noop(pool: MySqlPool) {
        // Seed : 1 company + 1 admin avec un hash CONNU.
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("Test Company")
        .bind("Addr")
        .bind("Independant")
        .bind("FR")
        .bind("FR")
        .execute(&pool)
        .await
        .expect("company insert");

        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("company_id");

        // Hash le password "test-bootstrap-password" pour qu'il matche l'env.
        let hash_initial = hash_password_async("test-bootstrap-password".to_string())
            .await
            .expect("hash");

        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("admin")
        .bind(&hash_initial)
        .bind("Admin")
        .bind(true)
        .bind(company_id)
        .execute(&pool)
        .await
        .expect("pre-insert admin");

        let config = test_config_with_env();
        let count = ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap recovery same-hash");
        assert_eq!(count, 1);

        // Hash inchangé.
        let hash_after: String =
            sqlx::query_scalar("SELECT password_hash FROM users WHERE username = ?")
                .bind("admin")
                .fetch_one(&pool)
                .await
                .expect("select hash");
        assert_eq!(
            hash_after, hash_initial,
            "hash must remain unchanged (no-op)"
        );

        // Pas d'audit log.
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'admin_break_glass_reset'",
        )
        .fetch_one(&pool)
        .await
        .expect("audit count");
        assert_eq!(audit_count, 0, "no audit log entry (no-op)");
    }

    /// Cas 5 (NEW v011-5) — recovery hash diff : UPDATE password + audit_log + revoke tokens.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_recovery_diff_hash_resets(pool: MySqlPool) {
        // Seed : 1 company + 1 admin avec un hash D'UN AUTRE password.
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("Test Company")
        .bind("Addr")
        .bind("Independant")
        .bind("FR")
        .bind("FR")
        .execute(&pool)
        .await
        .expect("company insert");

        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("company_id");

        let hash_old = hash_password_async("OLD-password-different".to_string())
            .await
            .expect("hash");

        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("admin")
        .bind(&hash_old)
        .bind("Admin")
        .bind(true)
        .bind(company_id)
        .execute(&pool)
        .await
        .expect("pre-insert admin");

        let admin_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
            .bind("admin")
            .fetch_one(&pool)
            .await
            .expect("admin_id");

        // Seed : 2 refresh tokens actifs pour l'admin (vérifier revocation).
        // UUIDs requis (DB constraint `chk_refresh_tokens_token_format`).
        let token1 = uuid::Uuid::new_v4().to_string();
        let token2 = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?), (?, ?, ?)",
        )
        .bind(admin_id)
        .bind(&token1)
        .bind(chrono::Utc::now().naive_utc() + chrono::Duration::days(1))
        .bind(admin_id)
        .bind(&token2)
        .bind(chrono::Utc::now().naive_utc() + chrono::Duration::days(1))
        .execute(&pool)
        .await
        .expect("seed refresh_tokens");

        let config = test_config_with_env();
        let count = ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap recovery diff-hash");
        assert_eq!(count, 1);

        // Hash changé (différent du seed `hash_old`).
        let hash_new: String =
            sqlx::query_scalar("SELECT password_hash FROM users WHERE username = ?")
                .bind("admin")
                .fetch_one(&pool)
                .await
                .expect("select hash");
        assert_ne!(hash_new, hash_old, "hash must be reset by recovery");

        // Le nouveau password matche le hash.
        let verifies = verify_password_async("test-bootstrap-password".to_string(), hash_new)
            .await
            .expect("verify");
        assert!(verifies, "new hash must verify against env password");

        // Audit log entry présente avec les champs attendus.
        let audit: Vec<(i64, String, String, i64)> = sqlx::query_as(
            "SELECT user_id, action, entity_type, entity_id FROM audit_log WHERE action = 'admin_break_glass_reset'",
        )
        .fetch_all(&pool)
        .await
        .expect("audit query");
        assert_eq!(audit.len(), 1, "exactly one recovery audit entry");
        assert_eq!(audit[0].0, admin_id);
        assert_eq!(audit[0].1, "admin_break_glass_reset");
        assert_eq!(audit[0].2, "user");
        assert_eq!(audit[0].3, admin_id);

        // Refresh tokens révoqués. Debug : inspecter le state complet d'abord.
        let all_tokens: Vec<(i64, String, Option<chrono::NaiveDateTime>, Option<String>)> =
            sqlx::query_as("SELECT user_id, token, revoked_at, revoked_reason FROM refresh_tokens")
                .fetch_all(&pool)
                .await
                .expect("all tokens");
        // `revoked_reason = 'password_change'` (cf. cas 5 bootstrap — contrainte
        // CHECK limite à un whitelist, le motif détaillé est dans audit_log.action).
        let revoked_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ? AND revoked_at IS NOT NULL AND revoked_reason = 'password_change'",
        )
        .bind(admin_id)
        .fetch_one(&pool)
        .await
        .expect("revoked count");
        assert_eq!(
            revoked_count, 2,
            "both refresh tokens must be revoked, full state: {:?}",
            all_tokens
        );
    }

    /// Cas 6 (NEW v011-5) — env set, no match username : no-op + warn.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_recovery_no_match_username_warns(pool: MySqlPool) {
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("Test Company")
        .bind("Addr")
        .bind("Independant")
        .bind("FR")
        .bind("FR")
        .execute(&pool)
        .await
        .expect("company insert");

        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("company_id");

        // Seed un user avec un username différent de l'env (`admin` dans config).
        let hash = hash_password_async("alice-password".to_string())
            .await
            .expect("hash");

        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("alice")
        .bind(&hash)
        .bind("Admin")
        .bind(true)
        .bind(company_id)
        .execute(&pool)
        .await
        .expect("pre-insert alice");

        let alice_hash_before: String =
            sqlx::query_scalar("SELECT password_hash FROM users WHERE username = ?")
                .bind("alice")
                .fetch_one(&pool)
                .await
                .expect("select alice hash");

        let config = test_config_with_env(); // KESH_ADMIN_USERNAME=admin
        let count = ensure_admin_user(&pool, &config)
            .await
            .expect("bootstrap no-match");
        assert_eq!(count, 1);

        // alice inchangée.
        let alice_hash_after: String =
            sqlx::query_scalar("SELECT password_hash FROM users WHERE username = ?")
                .bind("alice")
                .fetch_one(&pool)
                .await
                .expect("select alice hash");
        assert_eq!(alice_hash_after, alice_hash_before);

        // Pas de nouvel user `admin` créé.
        let admin_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = 'admin'")
                .fetch_one(&pool)
                .await
                .expect("admin count");
        assert_eq!(admin_count, 0);

        // Pas d'audit log.
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'admin_break_glass_reset'",
        )
        .fetch_one(&pool)
        .await
        .expect("audit count");
        assert_eq!(audit_count, 0);
    }

    /// Atomicité recovery cas 5 : si audit_log::insert_in_tx échoue
    /// (FK violation simulée via user_id inexistant), le password n'est PAS
    /// modifié (transaction rollback).
    ///
    /// **Note** : ne peut pas vraiment force-fail `insert_in_tx` ici sans
    /// mocking. Test approximé en vérifiant que la combinaison
    /// `UPDATE + audit + commit` est cohérente. Le test cas 5
    /// (`bootstrap_recovery_diff_hash_resets`) prouve déjà que les 3 effets
    /// (UPDATE password, INSERT audit, revoke tokens) sont effectifs ensemble.
    ///
    /// Pour un test rollback réel : il faudrait soit injecter un trait mockable,
    /// soit casser le schéma audit_log temporairement (TRUNCATE puis ré-INSERT
    /// admin avec un id désaligné). Hors scope v011-5 — limitation documentée.
    #[sqlx::test(migrations = "../kesh-db/test-schema")]
    async fn bootstrap_recovery_atomicity_smoke(pool: MySqlPool) {
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind("Test Company")
        .bind("Addr")
        .bind("Independant")
        .bind("FR")
        .bind("FR")
        .execute(&pool)
        .await
        .expect("company insert");

        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("company_id");

        let hash_old = hash_password_async("different-password".to_string())
            .await
            .expect("hash");

        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("admin")
        .bind(&hash_old)
        .bind("Admin")
        .bind(true)
        .bind(company_id)
        .execute(&pool)
        .await
        .expect("pre-insert admin");

        let config = test_config_with_env();
        ensure_admin_user(&pool, &config).await.expect("bootstrap");

        // Vérification : UPDATE et audit_log atomiques (les 2 ou aucun).
        let hash_after: String =
            sqlx::query_scalar("SELECT password_hash FROM users WHERE username = ?")
                .bind("admin")
                .fetch_one(&pool)
                .await
                .expect("select hash");
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'admin_break_glass_reset'",
        )
        .fetch_one(&pool)
        .await
        .expect("audit");

        // Atomicité : si UPDATE a passé, audit existe ; sinon ni l'un ni l'autre.
        let updated = hash_after != hash_old;
        let has_audit = audit_count == 1;
        assert_eq!(
            updated, has_audit,
            "atomicity broken: UPDATE applied without audit OR audit without UPDATE"
        );
    }
}
