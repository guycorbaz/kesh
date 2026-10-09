//! kesh-seed — Génération de données de démonstration pour Kesh.
//!
//! Ce crate est une lib, pas un binaire. Appelé via l'endpoint API
//! `POST /api/v1/onboarding/seed-demo`.

use chrono::{Datelike, Utc};
use kesh_db::entities::audit_log::{AUDIT_ENTITY_ID_NONE, NewAuditLogEntry};
use kesh_db::entities::onboarding::UiMode;
use kesh_db::entities::{Language, NewFiscalYear, OrgType};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::{
    audit_log, companies, company_invoice_settings, fiscal_years, onboarding, vat_rates,
};
use kesh_db::retry::{DEFAULT_MAX_DEADLOCK_ATTEMPTS, is_deadlock_error, retry_with};
use kesh_i18n::Locale;
use sqlx::{MySql, MySqlPool, Transaction};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SeedError {
    #[error("Erreur base de données : {0}")]
    Db(#[from] kesh_db::errors::DbError),

    #[error("Erreur SQL brute : {0}")]
    Sqlx(#[from] sqlx::Error),

    /// L'étape d'onboarding n'est plus 2 **sous le verrou** de la dernière
    /// transaction (Story 15-7b1) : une installation concurrente l'a franchie
    /// (`start-production`). Le handler le rend en
    /// `400 ONBOARDING_STEP_ALREADY_COMPLETED`.
    #[error("étape d'onboarding déjà franchie")]
    StepAlreadyCompleted,
}

/// L'erreur d'**un essai** de la dernière transaction de [`seed_demo`] —
/// Story 15-7b1 (AC 1, choix C-15-7-27, C-15-7-37).
///
/// ⚠️ **Sans `From<sqlx::Error>`, et c'est voulu** : le prédicat de rejeu
/// [`is_seed_retryable`] ne reconnaît un interblocage que sous
/// `SeedAttemptError::Db(DbError::Sqlx(_))`. Un `?` brut sur une `sqlx::Error`
/// produirait une autre variante et rendrait le rejeu **muet** ; sans la
/// conversion, ce `?` ne compile pas — `begin` et `commit` passent par
/// `map_db_error`. Le type est la garde de la conversion.
///
/// Public (et caché) pour la 15-7b2, qui l'étend à la remise à zéro.
#[doc(hidden)]
#[derive(Debug)]
pub enum SeedAttemptError {
    Db(DbError),
    StepAlreadyCompleted,
}

impl From<DbError> for SeedAttemptError {
    fn from(e: DbError) -> Self {
        Self::Db(e)
    }
}

impl From<SeedAttemptError> for SeedError {
    fn from(e: SeedAttemptError) -> Self {
        match e {
            SeedAttemptError::Db(d) => Self::Db(d),
            SeedAttemptError::StepAlreadyCompleted => Self::StepAlreadyCompleted,
        }
    }
}

/// Prédicat de rejeu de la dernière transaction de [`seed_demo`] : un
/// interblocage MariaDB (1213), et lui seul — Story 15-7b1 (décision C54 de
/// l'epic : tout flux d'écriture rejoue sur 1213). Partagé avec la 15-7b2.
#[doc(hidden)]
pub fn is_seed_retryable(e: &SeedAttemptError) -> bool {
    matches!(e, SeedAttemptError::Db(d) if is_deadlock_error(d))
}

/// L'auteur d'un chargement de démonstration : `(user_id, api_key_id)`,
/// threadé depuis le `CurrentUser` du handler — `seed-demo` est atteignable
/// par un jeton d'API (Story 15-7b1, AC 7).
pub type SeedActor = (i64, Option<i64>);

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

/// Adresse fictive suisse structurée (#213) selon la locale.
fn demo_address(locale: &Locale) -> kesh_db::entities::address::StructuredAddress {
    use kesh_db::entities::address::StructuredAddress;
    let (street, npa, city) = match locale {
        Locale::FrCh => ("Rue de la Démo", "1000", "Lausanne"),
        Locale::DeCh => ("Demostrasse", "3000", "Bern"),
        Locale::ItCh => ("Via Demo", "6500", "Bellinzona"),
        Locale::EnCh => ("Demo Street", "8000", "Zürich"),
    };
    StructuredAddress {
        street: street.to_string(),
        building: "1".to_string(),
        postal_code: npa.to_string(),
        city: city.to_string(),
        country: "CH".to_string(),
    }
}

/// Charge les données de démonstration et l'inscrit au journal d'audit.
///
/// Récupère la société unique (posée par le premier démarrage ou par l'étape
/// `language`), la renomme en société de démonstration, charge le plan
/// comptable PME, crée l'exercice de l'année, puis — dans une **dernière
/// transaction** — lève le drapeau provisoire, pose les réglages de
/// facturation et les quatre taux de TVA, franchit l'étape 2 → 3
/// (`is_demo = true`) et écrit `installation.demo_seeded` puis
/// `installation.step_completed` (Story 15-7b1, AC 1).
///
/// `actor` = `(user_id, api_key_id)` : l'entrée est attribuée au jeton d'API
/// quand la route en a été atteinte (`for_actor`, jamais `::user` — #431).
/// La version et le mode d'affichage de l'onboarding sont **relus sous
/// verrou** dans la dernière transaction : ils ne sont plus des paramètres.
///
/// # Ce qui est atomique, et ce qui ne l'est pas
///
/// ⚠️ Seule la **dernière transaction** est atomique avec sa trace. Les quatre
/// premières validations — le verrou de comptage, le renommage de la société,
/// le plan comptable, l'exercice — commitent chacune la leur, AVANT elle.
/// **Après un échec de la dernière transaction, la société renommée, le plan et
/// l'exercice restent commités ; relancer `seed-demo` échoue (500)** — les
/// comptes s'insèrent sans `IGNORE` et l'exercice refuse le chevauchement.
/// Seule la remise à zéro rend l'installation à l'étape de départ, **et
/// seulement par l'API** : l'interface ne la propose que dans la bannière de
/// démonstration, affichée si `isDemo`
/// (`frontend/src/routes/(app)/+layout.svelte`), alors que l'échec laisse
/// `is_demo = false`. Atomicité des quatre premières validations : #538.
///
/// # Rejeu
///
/// La dernière transaction est rejouée sur interblocage (1213) par
/// `kesh_db::retry::retry_with` ([`is_seed_retryable`]) : chaque essai ouvre
/// sa transaction et la conclut ; un essai annulé n'écrit ni taux, ni
/// réglages, ni trace. La boucle `InactiveOrInvalidAccounts` (trois rejeux,
/// quatre essais, 50 ms) l'enveloppe, conservée par prudence (C-15-7-14).
pub async fn seed_demo(
    pool: &MySqlPool,
    locale: &Locale,
    actor: SeedActor,
) -> Result<(), SeedError> {
    let lang = locale_to_language(locale);

    // P6: Lock-and-validate the singleton company row inside a short tx.
    // The FOR UPDATE lock covers ONLY the count-validation step (`len() != 1`):
    // it is committed before companies::update runs, so a concurrent reset_demo
    // could still DELETE the company between this commit and companies::update.
    // companies::update detects this via DbError::NotFound.
    //
    // Fenêtre résiduelle (Story 15-7b1) : `companies::update`,
    // `bulk_create_from_chart` et `fiscal_years::create_for_seed` commitent
    // chacune leur transaction, hors de ce verrou et AVANT la dernière
    // transaction. Si celle-ci échoue, la société renommée, le plan et
    // l'exercice restent commités et un nouveau `seed-demo` échoue jusqu'à
    // une remise à zéro. Rendre ces quatre validations atomiques : #538.
    let company = {
        let mut tx = pool.begin().await?;
        let companies_locked = sqlx::query_as::<_, kesh_db::entities::Company>(
            "SELECT id, name, first_name, last_name, address, address_street, address_building, address_postal_code, \
                    address_city, address_country, ide_number, org_type, accounting_language, \
                    instance_language, email, phone, website, is_stub, books_locked_through, version, created_at, updated_at \
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
                first_name: None,
                last_name: None,
                address_structured: demo_address(locale),
                ide_number: Some("CHE109322551".to_string()),
                org_type: OrgType::Pme,
                accounting_language: lang,
                instance_language: lang,
                email: None,
                // Story 16-3a (#151) — la société de démonstration porte des
                // coordonnées, pour que le PDF de démo montre le bloc émetteur
                // complet plutôt qu'un cas dégradé.
                phone: Some("+41 21 123 45 67".to_string()),
                website: Some("https://example.ch".to_string()),
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
    // Le plan s'insère dans sa propre transaction, commitée avant la dernière
    // (fenêtre résiduelle ci-dessus, #538). Deux `seed-demo` concurrents
    // passent tous deux la pré-vérification non verrouillée du handler ; le
    // perdant échoue ici (doublon de comptes) ou à `companies::update`
    // (version), en 500.
    let accounts = kesh_db::repositories::accounts::bulk_create_from_chart(
        pool, company.id, &chart, &lang_key,
    )
    .await?;

    // Exercice fiscal (année courante) — un seul appel Utc::now()
    let current_year = Utc::now().naive_utc().date().year();
    let start = chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).expect("valid date");
    let end = chrono::NaiveDate::from_ymd_opt(current_year, 12, 31).expect("valid date");

    // Story 3.7 T1.9 — `create_for_seed` n'écrit pas `fiscal_year.created` :
    // la démonstration est tracée par sa synthèse, `installation.demo_seeded`
    // (Story 15-7b1), qui porte `fiscal_year_id`.
    let fiscal_year = fiscal_years::create_for_seed(
        pool,
        NewFiscalYear {
            company_id: company.id,
            name: format!("Exercice {current_year}"),
            start_date: start,
            end_date: end,
        },
    )
    .await?;

    let facts = DemoFacts {
        company_id: company.id,
        org_type: company.org_type,
        accounts_created: accounts.len(),
        fiscal_year_id: fiscal_year.id,
        actor,
    };

    // Rejeu conservé par prudence, sans effet attendu (C-15-7-14, arbitrage
    // réservé à Guy) : dans la dernière transaction, les comptes de rôle sont
    // lus par lectures verrouillantes (dernier état commité), si bien que
    // `InactiveOrInvalidAccounts` y est permanent. Il enveloppe le rejeu sur
    // interblocage, qui s'y ajoute sans le remplacer.
    let mut retries = 0;
    let max_retries = 3;
    loop {
        let result = retry_with(
            "kesh_seed::seed_demo",
            DEFAULT_MAX_DEADLOCK_ATTEMPTS,
            is_seed_retryable,
            || final_attempt(pool, &facts),
        )
        .await;
        match result {
            Ok(()) => break,
            Err(SeedAttemptError::Db(DbError::InactiveOrInvalidAccounts))
                if retries < max_retries =>
            {
                retries += 1;
                tracing::warn!(
                    "Account lookup failed (attempt {}/{}), retrying after backoff",
                    retries,
                    max_retries
                );
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }

    tracing::info!("Données de démonstration chargées (locale: {locale})");
    Ok(())
}

/// Ce que les quatre premières validations de [`seed_demo`] ont produit, et
/// que la dernière transaction inscrit au journal.
struct DemoFacts {
    company_id: i64,
    org_type: OrgType,
    accounts_created: usize,
    fiscal_year_id: i64,
    actor: SeedActor,
}

/// Un essai de la dernière transaction de [`seed_demo`] : `BEGIN` → corps →
/// `COMMIT`, annulation explicite sur erreur. L'erreur d'une annulation est
/// journalisée et **jamais** substituée à l'erreur d'origine.
async fn final_attempt(pool: &MySqlPool, facts: &DemoFacts) -> Result<(), SeedAttemptError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match final_body(&mut tx, facts).await {
        Ok(()) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(())
        }
        Err(e) => {
            if let Err(rb) = tx.rollback().await {
                tracing::warn!("seed_demo : annulation de la dernière transaction en échec : {rb}");
            }
            Err(e)
        }
    }
}

/// Le corps de la dernière transaction de [`seed_demo`] — Story 15-7b1, AC 1.
///
/// Séquence de verrous, telle qu'écrite à la ligne `seed_demo` du Pattern 5
/// (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`) : `onboarding_state → companies →
/// accounts → company_invoice_settings → vat_rates → audit_log` — l'ordre de
/// `finalize` (réglages puis taux). Ne commite jamais.
async fn final_body(
    tx: &mut Transaction<'_, MySql>,
    facts: &DemoFacts,
) -> Result<(), SeedAttemptError> {
    // 1. Verrou d'état EN PREMIER, revérification de l'étape sous verrou. La
    //    ligne est garantie par le handler (`get_or_init_state`) : son absence
    //    est une corruption.
    let state = onboarding::lock_state_in_tx(tx).await?.ok_or_else(|| {
        DbError::Invariant("onboarding_state absent sous verrou pendant seed_demo".into())
    })?;
    if state.step_completed != 2 {
        return Err(SeedAttemptError::StepAlreadyCompleted);
    }

    // 2. Le drapeau provisoire, borné à la société et `version` bumpée s'il
    //    était levé (changement de sémantique assumé : l'ancien `UPDATE` du
    //    handler n'était borné à aucune société, F-3 de la P3 de la 15-7a1).
    companies::clear_stub_in_tx(tx, facts.company_id).await?;

    // 3. Réglages de facturation (lectures verrouillantes des comptes de rôle),
    //    puis les quatre taux de TVA — l'ordre de `finalize`.
    let (_settings, invoice_settings_created) =
        company_invoice_settings::insert_with_defaults_in_tx(tx, facts.company_id).await?;
    let rates = vat_rates::seed_default_swiss_rates_in_tx(tx, facts.company_id).await?;

    // 4. L'étape, sur la version et le mode relus sous verrou.
    let ui_mode = state.ui_mode.unwrap_or(UiMode::Guided);
    onboarding::update_step_in_tx(tx, 3, true, Some(ui_mode), state.version).await?;

    // 5. La synthèse, puis l'étape — en dernier.
    let (user_id, api_key_id) = facts.actor;
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            api_key_id,
            "installation.demo_seeded",
            "installation",
            AUDIT_ENTITY_ID_NONE,
            Some(serde_json::json!({
                "company_id": facts.company_id,
                "org_type": facts.org_type,
                "accounts_created": facts.accounts_created,
                "fiscal_year_id": facts.fiscal_year_id,
                "vat_rates_created": rates.len(),
                "invoice_settings_created": invoice_settings_created,
            })),
        ),
    )
    .await?;
    onboarding::record_step_completed_in_tx(tx, user_id, api_key_id, 2, 3, "seed_demo").await?;
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
        // Story 25-2-c (#381) — les compteurs de numéros d'écriture référencent
        // `fiscal_years` (FK RESTRICT). Sous `FOREIGN_KEY_CHECKS=0` leur oubli
        // ne ferait rien ÉCHOUER : il laisserait des lignes orphelines pointant
        // vers des exercices effacés, et la base porterait des références mortes
        // dès la réactivation des contrôles. C'est une question de cohérence de
        // données, pas de contrainte technique.
        sqlx::query("DELETE FROM journal_entry_number_sequences")
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
