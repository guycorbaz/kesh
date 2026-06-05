//! Repository pour `bank_profiles` — profils CSV par banque (Story 8-2 FR53).
//!
//! **Signatures génériques `Executor`** (Pass 3 L''2) : les helpers
//! acceptent à la fois `&MySqlPool` et `&mut Transaction` via le trait
//! `sqlx::Executor`. Permet l'appel transaction-bound depuis le handler
//! `POST /bank-imports` (cf. spec §profile-matching race condition Pass
//! 2 H'2 + Pass 3 M''4 séquencement Interprétation A).
//!
//! Toutes les requêtes scopent strictement par `company_id` (pattern
//! KF-002) — un profil de `company_A` n'est jamais visible à
//! `company_B` (404 systématique).

use sqlx::mysql::MySqlPool;
use sqlx::{Executor, MySql, Transaction};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::bank_profile::{BankProfile, NewBankProfile};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str = "id, company_id, bank_name, filename_pattern, \
     CAST(column_mapping AS CHAR) AS column_mapping_json, \
     date_format, decimal_separator, field_separator, encoding, \
     header_row_count, created_at, updated_at";

/// Crée un profil dans la transaction `tx` + écrit l'entrée audit_log
/// associée. Le caller commit la transaction.
///
/// # Erreurs
///
/// - [`DbError::UniqueConstraintViolation`] si `(company_id, bank_name)`
///   existe déjà → caller mappe vers `409 BANK_CSV_PROFILE_DUPLICATE`.
/// - [`DbError::CheckConstraintViolation`] sur les CHECKs DB
///   (séparateurs, header_row_count, etc.) — défense en profondeur,
///   normalement bloqué par la validation handler API.
pub async fn create(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    new_profile: &NewBankProfile,
    user_id: i64,
) -> Result<BankProfile, DbError> {
    let result = sqlx::query(
        "INSERT INTO bank_profiles \
         (company_id, bank_name, filename_pattern, column_mapping, date_format, \
          decimal_separator, field_separator, encoding, header_row_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(&new_profile.bank_name)
    .bind(&new_profile.filename_pattern)
    .bind(&new_profile.column_mapping_json)
    .bind(&new_profile.date_format)
    .bind(&new_profile.decimal_separator)
    .bind(&new_profile.field_separator)
    .bind(&new_profile.encoding)
    .bind(new_profile.header_row_count)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let id = result.last_insert_id() as i64;

    // Audit log atomique
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "bank_profile.created".to_string(),
            "bank_profiles".to_string(),
            id,
            Some(serde_json::json!({
                "bank_name": new_profile.bank_name,
                "filename_pattern": new_profile.filename_pattern,
            })),
        ),
    )
    .await?;

    let profile = find_by_id_for_company(&mut **tx, company_id, id)
        .await?
        .ok_or_else(|| {
            DbError::Invariant(format!(
                "bank_profile id={} créé mais SELECT-back returns None",
                id
            ))
        })?;
    Ok(profile)
}

/// Récupère un profil scopé par `company_id`. Retourne `None` (pas
/// d'erreur) pour cross-tenant (pattern KF-002 → 404 côté API).
///
/// **Signature générique** : accepte `&MySqlPool` ou `&mut Transaction`
/// via `Executor`. Critique pour l'appel transaction-bound dans
/// `POST /bank-imports`.
pub async fn find_by_id_for_company<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<BankProfile>, DbError>
where
    E: Executor<'e, Database = MySql>,
{
    let query = format!(
        "SELECT {} FROM bank_profiles WHERE id = ? AND company_id = ?",
        COLUMNS
    );
    sqlx::query_as::<_, BankProfile>(&query)
        .bind(id)
        .bind(company_id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_error)
}

/// Story 9-2b T3.2.10 — Liste **tous** les profils bancaires d'une company
/// sans pagination, pour l'export global ZIP. Tri stable `id ASC`.
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<BankProfile>, DbError> {
    sqlx::query_as::<_, BankProfile>(&format!(
        "SELECT {} FROM bank_profiles \
         WHERE company_id = ? \
         ORDER BY id",
        COLUMNS
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Liste paginée des profils d'une company. Retourne `(profils, count_total)`.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    limit: i64,
    offset: i64,
) -> Result<(Vec<BankProfile>, i64), DbError> {
    let query = format!(
        "SELECT {} FROM bank_profiles WHERE company_id = ? \
         ORDER BY updated_at DESC LIMIT ? OFFSET ?",
        COLUMNS
    );
    let profiles = sqlx::query_as::<_, BankProfile>(&query)
        .bind(company_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_profiles WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    Ok((profiles, count))
}

/// Trouve les profils dont le `filename_pattern` (regex) matche le
/// filename donné. Retourne `Vec<BankProfile>` triés par `updated_at DESC`.
///
/// Filtrage Rust-side (regex MariaDB diffère). Pas de cache regex v0.1
/// (cf. Limitations connues L1).
pub async fn find_matching_profiles_for_filename<'e, E>(
    executor: E,
    company_id: i64,
    filename: &str,
) -> Result<Vec<BankProfile>, DbError>
where
    E: Executor<'e, Database = MySql>,
{
    let query = format!(
        "SELECT {} FROM bank_profiles \
         WHERE company_id = ? AND filename_pattern IS NOT NULL \
         ORDER BY updated_at DESC",
        COLUMNS
    );
    let candidates = sqlx::query_as::<_, BankProfile>(&query)
        .bind(company_id)
        .fetch_all(executor)
        .await
        .map_err(map_db_error)?;

    let mut matched = Vec::new();
    for profile in candidates {
        if let Some(ref pattern) = profile.filename_pattern {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(filename) {
                        matched.push(profile);
                    }
                }
                Err(e) => {
                    // Pass 1 review G2-BH-11 + G2-EH-7 : tracer la regex
                    // invalide en DB plutôt que de l'ignorer silencieusement
                    // (silent failure hostile au diagnostic).
                    tracing::warn!(
                        "bank_profile id={} : filename_pattern invalide ({:?}) — \
                         ignoré au matching ({})",
                        profile.id,
                        pattern,
                        e
                    );
                }
            }
        }
    }
    Ok(matched)
}

/// PUT full replacement (Pass 1 M18) — remplacement complet, body doit
/// contenir tous les champs. + audit log atomique dans `tx`.
pub async fn update(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    new_profile: &NewBankProfile,
    user_id: i64,
) -> Result<BankProfile, DbError> {
    // Vérifier que le profil existe et appartient à la company.
    let existing = find_by_id_for_company(&mut **tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)?;

    let result = sqlx::query(
        "UPDATE bank_profiles SET \
         bank_name = ?, filename_pattern = ?, column_mapping = ?, \
         date_format = ?, decimal_separator = ?, field_separator = ?, \
         encoding = ?, header_row_count = ? \
         WHERE id = ? AND company_id = ?",
    )
    .bind(&new_profile.bank_name)
    .bind(&new_profile.filename_pattern)
    .bind(&new_profile.column_mapping_json)
    .bind(&new_profile.date_format)
    .bind(&new_profile.decimal_separator)
    .bind(&new_profile.field_separator)
    .bind(&new_profile.encoding)
    .bind(new_profile.header_row_count)
    .bind(id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    // Audit log avec champs changés.
    let mut fields_changed = Vec::new();
    if existing.bank_name != new_profile.bank_name {
        fields_changed.push("bank_name");
    }
    if existing.filename_pattern != new_profile.filename_pattern {
        fields_changed.push("filename_pattern");
    }
    if existing.column_mapping_json != new_profile.column_mapping_json {
        fields_changed.push("column_mapping");
    }
    if existing.date_format != new_profile.date_format {
        fields_changed.push("date_format");
    }
    if existing.decimal_separator != new_profile.decimal_separator {
        fields_changed.push("decimal_separator");
    }
    if existing.field_separator != new_profile.field_separator {
        fields_changed.push("field_separator");
    }
    if existing.encoding != new_profile.encoding {
        fields_changed.push("encoding");
    }
    if existing.header_row_count != new_profile.header_row_count {
        fields_changed.push("header_row_count");
    }

    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "bank_profile.updated".to_string(),
            "bank_profiles".to_string(),
            id,
            Some(serde_json::json!({
                "bank_name": new_profile.bank_name,
                "fields_changed": fields_changed,
            })),
        ),
    )
    .await?;

    let updated = find_by_id_for_company(&mut **tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)?;
    Ok(updated)
}

/// Supprime un profil (CASCADE OK car FK CASCADE sur `companies`, mais
/// pas de FK depuis `bank_imports` → orphan accepted v0.1, cf. L4).
/// + audit log atomique avec snapshot `bank_name`.
pub async fn delete(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<(), DbError> {
    let existing = find_by_id_for_company(&mut **tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)?;

    let result = sqlx::query("DELETE FROM bank_profiles WHERE id = ? AND company_id = ?")
        .bind(id)
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "bank_profile.deleted".to_string(),
            "bank_profiles".to_string(),
            id,
            Some(serde_json::json!({
                "bank_name": existing.bank_name,
            })),
        ),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::MySqlPool;

    fn sample_profile_json() -> String {
        serde_json::json!({
            "date": 0,
            "amount": 1,
            "reference": 2,
            "details": 3
        })
        .to_string()
    }

    fn make_new_profile(name: &str) -> NewBankProfile {
        NewBankProfile {
            bank_name: name.to_string(),
            filename_pattern: Some(r"^export-\d+\.csv$".to_string()),
            column_mapping_json: sample_profile_json(),
            date_format: "%Y-%m-%d".to_string(),
            decimal_separator: ".".to_string(),
            field_separator: ";".to_string(),
            encoding: None,
            header_row_count: 1,
        }
    }

    async fn seed_company(pool: &MySqlPool) -> i64 {
        sqlx::query(
            "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("Test SA")
        .bind("Rue Test 1, 1000 Lausanne")
        .bind("Pme")
        .bind("FR")
        .bind("FR")
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    async fn seed_user(pool: &MySqlPool, company_id: i64) -> i64 {
        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) \
             VALUES (?, ?, 'Comptable', TRUE, ?)",
        )
        .bind(format!("user_{}", company_id))
        .bind("$argon2id$v=19$m=4096,t=3,p=1$dGVzdA$U/UGzVcJ5HptuaW9CKHwBQAVZRZpa+jJh1iJSjEDCs8")
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_inserts_profile_and_audit_log(pool: MySqlPool) {
        let company_id = seed_company(&pool).await;
        let user_id = seed_user(&pool, company_id).await;
        let new = make_new_profile("UBS");

        let mut tx = pool.begin().await.unwrap();
        let profile = create(&mut tx, company_id, &new, user_id).await.unwrap();
        tx.commit().await.unwrap();

        assert_eq!(profile.bank_name, "UBS");
        assert_eq!(profile.field_separator, ";");
        assert_eq!(profile.header_row_count, 1);

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE entity_type='bank_profiles'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn find_by_id_for_company_returns_none_for_other_company(pool: MySqlPool) {
        let company_a = seed_company(&pool).await;
        let company_b = seed_company(&pool).await;
        let user_a = seed_user(&pool, company_a).await;

        let mut tx = pool.begin().await.unwrap();
        let profile = create(&mut tx, company_a, &make_new_profile("UBS"), user_a)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Cross-tenant : company_b ne voit pas le profil de company_a.
        let result = find_by_id_for_company(&pool, company_b, profile.id)
            .await
            .unwrap();
        assert!(result.is_none());

        // Own company : voit son profil.
        let result = find_by_id_for_company(&pool, company_a, profile.id)
            .await
            .unwrap();
        assert!(result.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn list_by_company_only_returns_own(pool: MySqlPool) {
        let company_a = seed_company(&pool).await;
        let company_b = seed_company(&pool).await;
        let user_a = seed_user(&pool, company_a).await;
        let user_b = seed_user(&pool, company_b).await;

        let mut tx = pool.begin().await.unwrap();
        create(&mut tx, company_a, &make_new_profile("UBS"), user_a)
            .await
            .unwrap();
        create(&mut tx, company_a, &make_new_profile("Raiffeisen"), user_a)
            .await
            .unwrap();
        create(&mut tx, company_b, &make_new_profile("PostFinance"), user_b)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let (profiles_a, count_a) = list_by_company(&pool, company_a, 100, 0).await.unwrap();
        assert_eq!(count_a, 2);
        assert_eq!(profiles_a.len(), 2);

        let (profiles_b, count_b) = list_by_company(&pool, company_b, 100, 0).await.unwrap();
        assert_eq!(count_b, 1);
        assert_eq!(profiles_b[0].bank_name, "PostFinance");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn duplicate_bank_name_within_company_rejected(pool: MySqlPool) {
        let company_id = seed_company(&pool).await;
        let user_id = seed_user(&pool, company_id).await;

        let mut tx = pool.begin().await.unwrap();
        create(&mut tx, company_id, &make_new_profile("UBS"), user_id)
            .await
            .unwrap();
        let err = create(&mut tx, company_id, &make_new_profile("UBS"), user_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::UniqueConstraintViolation(_)));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn same_bank_name_allowed_across_companies(pool: MySqlPool) {
        let company_a = seed_company(&pool).await;
        let company_b = seed_company(&pool).await;
        let user_a = seed_user(&pool, company_a).await;
        let user_b = seed_user(&pool, company_b).await;

        let mut tx = pool.begin().await.unwrap();
        create(&mut tx, company_a, &make_new_profile("UBS"), user_a)
            .await
            .unwrap();
        create(&mut tx, company_b, &make_new_profile("UBS"), user_b)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn update_full_replacement_writes_audit_log_with_fields_changed(pool: MySqlPool) {
        let company_id = seed_company(&pool).await;
        let user_id = seed_user(&pool, company_id).await;

        let mut tx = pool.begin().await.unwrap();
        let original = create(&mut tx, company_id, &make_new_profile("UBS"), user_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let mut updated_profile = make_new_profile("UBS");
        updated_profile.date_format = "%d.%m.%Y".to_string();
        updated_profile.header_row_count = 2;

        let mut tx = pool.begin().await.unwrap();
        let result = update(&mut tx, company_id, original.id, &updated_profile, user_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(result.date_format, "%d.%m.%Y");
        assert_eq!(result.header_row_count, 2);

        let details: serde_json::Value = sqlx::query_scalar(
            "SELECT details_json FROM audit_log \
             WHERE entity_type='bank_profiles' AND action='bank_profile.updated' \
             ORDER BY id DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let fields = details["fields_changed"].as_array().unwrap();
        let names: Vec<&str> = fields.iter().filter_map(|v| v.as_str()).collect();
        assert!(names.contains(&"date_format"));
        assert!(names.contains(&"header_row_count"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_with_audit_log_snapshot(pool: MySqlPool) {
        let company_id = seed_company(&pool).await;
        let user_id = seed_user(&pool, company_id).await;

        let mut tx = pool.begin().await.unwrap();
        let profile = create(&mut tx, company_id, &make_new_profile("UBS"), user_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        delete(&mut tx, company_id, profile.id, user_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let still_there = find_by_id_for_company(&pool, company_id, profile.id)
            .await
            .unwrap();
        assert!(still_there.is_none());

        // Audit log conserve le bank_name snapshot
        let details: serde_json::Value = sqlx::query_scalar(
            "SELECT details_json FROM audit_log \
             WHERE entity_type='bank_profiles' AND action='bank_profile.deleted' \
             ORDER BY id DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(details["bank_name"].as_str().unwrap(), "UBS");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn find_matching_profiles_filters_by_filename_and_company(pool: MySqlPool) {
        let company_a = seed_company(&pool).await;
        let company_b = seed_company(&pool).await;
        let user_a = seed_user(&pool, company_a).await;
        let user_b = seed_user(&pool, company_b).await;

        let mut profile_a = make_new_profile("UBS");
        profile_a.filename_pattern = Some(r"^ubs-\d+\.csv$".to_string());
        let mut profile_b = make_new_profile("UBS");
        profile_b.filename_pattern = Some(r"^ubs-\d+\.csv$".to_string());

        let mut tx = pool.begin().await.unwrap();
        create(&mut tx, company_a, &profile_a, user_a)
            .await
            .unwrap();
        create(&mut tx, company_b, &profile_b, user_b)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Match company_a only
        let matched = find_matching_profiles_for_filename(&pool, company_a, "ubs-20260315.csv")
            .await
            .unwrap();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].company_id, company_a);

        // No match (extension différente)
        let no_match = find_matching_profiles_for_filename(&pool, company_a, "ubs-20260315.xml")
            .await
            .unwrap();
        assert!(no_match.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn parse_column_mapping_returns_typed_struct(pool: MySqlPool) {
        let company_id = seed_company(&pool).await;
        let user_id = seed_user(&pool, company_id).await;

        let mut tx = pool.begin().await.unwrap();
        let profile = create(&mut tx, company_id, &make_new_profile("UBS"), user_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let cm = profile.parse_column_mapping().unwrap();
        assert_eq!(cm.date, 0);
        assert_eq!(cm.amount, Some(1));
        assert_eq!(cm.reference, Some(2));
        assert_eq!(cm.details, Some(3));
    }
}
