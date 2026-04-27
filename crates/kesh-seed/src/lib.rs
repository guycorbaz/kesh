//! kesh-seed — Génération de données de démonstration pour Kesh.
//!
//! Ce crate est une lib, pas un binaire. Appelé via l'endpoint API
//! `POST /api/v1/onboarding/seed-demo`.

use chrono::{Datelike, Utc};
use kesh_db::entities::onboarding::UiMode;
use kesh_db::entities::{Language, NewFiscalYear, OrgType};
use kesh_db::repositories::{companies, fiscal_years, onboarding};
use kesh_i18n::Locale;
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SeedError {
    #[error("Erreur base de données : {0}")]
    Db(#[from] kesh_db::errors::DbError),

    #[error("Erreur SQL brute : {0}")]
    Sqlx(#[from] sqlx::Error),
}

/// Convertit un `Locale` (kesh-i18n) en `Language` (kesh-db).
///
/// Fonction libre (pas un trait `From`) car la règle des orphelins Rust
/// interdit l'impl dans ce crate (ni `Locale` ni `Language` n'y sont définis).
pub fn locale_to_language(locale: &Locale) -> Language {
    match locale {
        Locale::FrCh => Language::Fr,
        Locale::DeCh => Language::De,
        Locale::ItCh => Language::It,
        Locale::EnCh => Language::En,
    }
}

/// Noms de la company démo selon la locale.
fn demo_company_name(locale: &Locale) -> &'static str {
    match locale {
        Locale::FrCh => "Démo SA",
        Locale::DeCh => "Demo AG",
        Locale::ItCh => "Demo SA",
        Locale::EnCh => "Demo Ltd",
    }
}

/// Adresse fictive suisse selon la locale.
fn demo_address(locale: &Locale) -> &'static str {
    match locale {
        Locale::FrCh => "Rue de la Démo 1, 1000 Lausanne",
        Locale::DeCh => "Demostrasse 1, 3000 Bern",
        Locale::ItCh => "Via Demo 1, 6500 Bellinzona",
        Locale::EnCh => "Demo Street 1, 8000 Zürich",
    }
}

/// Charge les données de démonstration.
///
/// Récupère la company existante créée par ensure_company_with_language,
/// la met à jour avec les infos démo, crée un exercice fiscal,
/// et met `onboarding_state` à step=3, is_demo=true.
/// Passe par les repositories kesh-db pour respecter les contraintes DB.
/// `onboarding_version` est la version actuelle de l'onboarding_state,
/// passée par le handler pour éviter une double lecture (TOCTOU).
pub async fn seed_demo(
    pool: &MySqlPool,
    locale: &Locale,
    ui_mode: UiMode,
    onboarding_version: i32,
) -> Result<(), SeedError> {
    let lang = locale_to_language(locale);

    // P6: Lock-and-validate the singleton company row inside a short tx.
    // The FOR UPDATE lock covers ONLY the count-validation step (`len() != 1`):
    // it is committed before companies::update runs, so a concurrent reset_demo
    // could still DELETE the company between this commit and companies::update.
    // companies::update detects this via DbError::NotFound and the user retries.
    //
    // Residual race window (acceptable for v0.1 single-tenant single-user):
    // - bulk_create_from_chart and fiscal_years::create commit their own
    //   transactions outside this lock. If seed_demo aborts after they
    //   commit, the company is left with orphaned accounts/fiscal_year
    //   until reset_demo cleans up.
    // - The retry loop on insert_with_defaults below tolerates the cross-tx
    //   visibility gap; the single-tx refactor that eliminates both issues
    //   is tracked under KF-002-H-002 (issue #43, v0.2).
    let company = {
        let mut tx = pool.begin().await?;
        let companies_locked = sqlx::query_as::<_, kesh_db::entities::Company>(
            "SELECT id, name, address, ide_number, org_type, accounting_language, \
                    instance_language, version, created_at, updated_at \
             FROM companies ORDER BY id FOR UPDATE",
        )
        .fetch_all(&mut *tx)
        .await?;

        // P1-C2: Validate exactly 1 company exists (prevent corruption from race conditions).
        // Under FOR UPDATE the count is observed atomically with the subsequent update.
        if companies_locked.len() != 1 {
            tx.rollback().await.ok();
            return Err(SeedError::Db(kesh_db::errors::DbError::Invariant(format!(
                "Expected exactly 1 company for seed_demo, found {}",
                companies_locked.len()
            ))));
        }
        let company = companies_locked.into_iter().next().expect("len checked");

        // Release the lock before calling companies::update — that helper opens its
        // own transaction, and holding the FOR UPDATE here would deadlock against it.
        tx.commit().await?;

        // Update company with demo info (optimistic version check inside).
        use kesh_db::entities::CompanyUpdate;
        companies::update(
            pool,
            company.id,
            company.version,
            CompanyUpdate {
                name: demo_company_name(locale).to_string(),
                address: demo_address(locale).to_string(),
                ide_number: Some("CHE109322551".to_string()),
                org_type: OrgType::Pme,
                accounting_language: lang,
                instance_language: lang,
            },
        )
        .await?
    };

    // Plan comptable PME dans la langue comptable de la company démo
    let chart =
        kesh_core::chart_of_accounts::load_chart(company.org_type.as_str()).map_err(|e| {
            SeedError::Db(kesh_db::errors::DbError::Invariant(format!(
                "chart load: {e}"
            )))
        })?;
    let lang_key = company.accounting_language.as_str().to_lowercase();
    // Bulk insert uses its own transaction — commits before insert_with_defaults reads.
    // P6-L3: seed_demo updates the singleton company (set up earlier by
    // ensure_company_with_language); concurrent seed_demo calls are serialized
    // by the FOR UPDATE lock acquired in the count-validation block above.
    // Insert lookups (accounts 1100, 3000) are per-company and isolated.
    kesh_db::repositories::accounts::bulk_create_from_chart(pool, company.id, &chart, &lang_key)
        .await?;

    // Exercice fiscal (année courante) — un seul appel Utc::now()
    let current_year = Utc::now().naive_utc().date().year();
    let start = chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).expect("valid date");
    let end = chrono::NaiveDate::from_ymd_opt(current_year, 12, 31).expect("valid date");

    fiscal_years::create(
        pool,
        NewFiscalYear {
            company_id: company.id,
            name: format!("Exercice {current_year}"),
            start_date: start,
            end_date: end,
        },
    )
    .await?;

    // Story 2.6: Pre-fill invoice settings with default accounts (1100, 3000).
    // P1-H3 / P7: Retry with backoff for account lookup timing issues.
    // bulk_create_from_chart commits in its own tx; under MariaDB REPEATABLE READ a
    // separate session may not see the freshly-committed accounts immediately.
    // 50 ms backoff between retries gives the inserter time to flush and avoids
    // a busy-spin that would never resolve a permanent misconfiguration.
    // Permanent missing-account misconfig surfaces as Err after max_retries.
    let mut retries = 0;
    let max_retries = 3;
    loop {
        match kesh_db::repositories::company_invoice_settings::insert_with_defaults(
            pool, company.id,
        )
        .await
        {
            Ok(_settings) => {
                if retries > 0 {
                    tracing::debug!(
                        "company_invoice_settings inserted successfully (after {} retry attempts)",
                        retries
                    );
                }
                break;
            }
            Err(kesh_db::errors::DbError::InactiveOrInvalidAccounts) if retries < max_retries => {
                retries += 1;
                tracing::warn!(
                    "Account lookup failed (attempt {}/{}), retrying after backoff",
                    retries,
                    max_retries
                );
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Err(e) => return Err(SeedError::Db(e)),
        }
    }

    // Mettre à jour onboarding_state → step=3, is_demo=true
    onboarding::update_step(pool, 3, true, Some(ui_mode), onboarding_version).await?;

    tracing::info!("Données de démonstration chargées (locale: {locale})");
    Ok(())
}

/// Supprime toutes les données de démonstration et remet l'onboarding à zéro.
///
/// Orchestration FK-safe : désactive les checks FK, nettoie les tables dans
/// l'ordre correct, puis réinitialise onboarding_state.
/// Préserve les users et refresh_tokens.
pub async fn reset_demo(pool: &MySqlPool) -> Result<(), SeedError> {
    // Connexion dédiée : SET FOREIGN_KEY_CHECKS est une variable de session
    // MariaDB — sur un pool partagé, chaque execute() peut utiliser une
    // connexion différente. On acquiert une connexion unique pour garantir
    // que le flag reste actif pendant les DELETEs.
    let mut conn = pool.acquire().await?;

    sqlx::query("SET FOREIGN_KEY_CHECKS=0")
        .execute(&mut *conn)
        .await?;

    let result = async {
        // Story 3.3 — audit_log en premier : sous FK_CHECKS=0 l'ordre
        // importe peu, mais le DELETE explicite est plus safe si le
        // flag est un jour retiré. Les entrées d'audit FK vers users
        // RESTRICT — elles DOIVENT disparaître avant toute tentative
        // de suppression d'un user (bien que reset_demo préserve users).
        sqlx::query("DELETE FROM audit_log")
            .execute(&mut *conn)
            .await?;
        // Story 3.2 — écritures comptables.
        // Sous FOREIGN_KEY_CHECKS=0 le CASCADE sur journal_entry_lines
        // est techniquement inutile, mais on supprime explicitement
        // pour rester safe si le flag est un jour retiré.
        sqlx::query("DELETE FROM journal_entry_lines")
            .execute(&mut *conn)
            .await?;
        sqlx::query("DELETE FROM journal_entries")
            .execute(&mut *conn)
            .await?;
        sqlx::query("DELETE FROM accounts")
            .execute(&mut *conn)
            .await?;
        sqlx::query("DELETE FROM fiscal_years")
            .execute(&mut *conn)
            .await?;
        sqlx::query("DELETE FROM bank_accounts")
            .execute(&mut *conn)
            .await?;
        sqlx::query("DELETE FROM companies")
            .execute(&mut *conn)
            .await?;
        Ok::<(), sqlx::Error>(())
    }
    .await;

    // Toujours réactiver FK checks, même en cas d'erreur.
    if let Err(e) = sqlx::query("SET FOREIGN_KEY_CHECKS=1")
        .execute(&mut *conn)
        .await
    {
        tracing::warn!("Failed to re-enable FK checks: {e}");
    }

    // Libérer la connexion (drop implicite) avant les appels au pool
    drop(conn);

    result?;

    // Reset onboarding state — DELETE + INSERT dans un seul appel pour
    // éviter un état vide transitoire si init_state échoue.
    onboarding::delete_state(pool).await?;
    if let Err(e) = onboarding::init_state(pool).await {
        tracing::error!("Failed to re-init onboarding after delete: {e}");
        return Err(e.into());
    }

    tracing::info!("Données de démonstration supprimées, onboarding réinitialisé");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_to_language_maps_correctly() {
        assert_eq!(locale_to_language(&Locale::FrCh), Language::Fr);
        assert_eq!(locale_to_language(&Locale::DeCh), Language::De);
        assert_eq!(locale_to_language(&Locale::ItCh), Language::It);
        assert_eq!(locale_to_language(&Locale::EnCh), Language::En);
    }

    #[test]
    fn demo_names_are_locale_specific() {
        assert_eq!(demo_company_name(&Locale::FrCh), "Démo SA");
        assert_eq!(demo_company_name(&Locale::DeCh), "Demo AG");
    }
}
