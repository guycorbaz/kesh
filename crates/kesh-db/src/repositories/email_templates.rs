//! Repository pour `email_templates` (Epic 20 #224, Story 20-1).
//!
//! Résolution override→défaut : [`get_effective`]/[`list_effective_for_company`]
//! ne renvoient jamais d'erreur "introuvable" pour une combinaison
//! type×langue valide — l'absence de ligne retombe sur le texte par défaut
//! (cf. [`crate::entities::email_template_defaults`]).
//!
//! **Verrou optimiste `expectedVersion` à sémantique double** (contrairement
//! aux repositories `update()` classiques où la ligne existe toujours) :
//! - `None` : le caller croit qu'aucun override n'existe → `INSERT`.
//! - `Some(v)` : le caller modifie l'override existant à la version `v` → `UPDATE`.
//!
//! Le `SELECT ... FOR UPDATE` dans [`upsert_override`]/[`restore_default`]
//! verrouille la ligne (ou le gap InnoDB si absente) dans la transaction,
//! éliminant la race entre deux `PUT`/`DELETE` concurrents sans avoir à
//! attraper une violation `UNIQUE` après coup.
//!
//! **Audit** : `NewAuditLogEntry::user(user_id, ...)` — pas `for_actor`
//! (le threading du PAT `api_key_id` est un pattern de route handler
//! `kesh-api`, cf. `exports.rs`/`bank_accounts.rs`, jamais utilisé dans un
//! repository `kesh-db` ; `company_invoice_settings.rs`, le modèle
//! structurel le plus proche de ce repository, utilise `::user` seul).

use sqlx::mysql::MySqlPool;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{EffectiveEmailTemplate, EmailTemplate, EmailTemplateType, Language};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str =
    "id, company_id, template_type, language, subject, body, version, created_at, updated_at";

const LANGUAGES: [Language; 4] = [Language::Fr, Language::De, Language::It, Language::En];

fn template_snapshot_json(t: &EmailTemplate) -> serde_json::Value {
    serde_json::json!({
        "id": t.id,
        "companyId": t.company_id,
        "templateType": t.template_type.as_str(),
        "language": t.language.as_str(),
        "subject": t.subject,
        "body": t.body,
        "version": t.version,
    })
}

fn to_effective_override(row: &EmailTemplate) -> EffectiveEmailTemplate {
    EffectiveEmailTemplate {
        template_type: row.template_type,
        language: row.language,
        subject: row.subject.clone(),
        body: row.body.clone(),
        version: Some(row.version),
        is_default: false,
        allowed_variables: row
            .template_type
            .allowed_variables()
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

fn to_effective_default(
    template_type: EmailTemplateType,
    language: Language,
) -> EffectiveEmailTemplate {
    let (subject, body) = crate::entities::default_template(template_type, language);
    EffectiveEmailTemplate {
        template_type,
        language,
        subject: subject.to_string(),
        body: body.to_string(),
        version: None,
        is_default: true,
        allowed_variables: template_type
            .allowed_variables()
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

/// Résout le template effectif pour `(company_id, template_type, language)` :
/// l'override en base si présent, sinon le défaut. Jamais d'erreur pour une
/// combinaison valide.
pub async fn get_effective(
    pool: &MySqlPool,
    company_id: i64,
    template_type: EmailTemplateType,
    language: Language,
) -> Result<EffectiveEmailTemplate, DbError> {
    let row = sqlx::query_as::<_, EmailTemplate>(&format!(
        "SELECT {COLUMNS} FROM email_templates WHERE company_id = ? AND template_type = ? AND language = ?"
    ))
    .bind(company_id)
    .bind(template_type)
    .bind(language)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?;

    Ok(match row {
        Some(r) => to_effective_override(&r),
        None => to_effective_default(template_type, language),
    })
}

/// Résout les templates effectifs pour toutes les combinaisons
/// `EmailTemplateType::ALL × [FR, DE, IT, EN]` (4 en v1). Une seule requête
/// pour charger les overrides existants de la company, puis complète avec
/// les défauts pour les combinaisons sans ligne.
pub async fn list_effective_for_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<EffectiveEmailTemplate>, DbError> {
    let rows = sqlx::query_as::<_, EmailTemplate>(&format!(
        "SELECT {COLUMNS} FROM email_templates WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    let mut out = Vec::with_capacity(EmailTemplateType::ALL.len() * LANGUAGES.len());
    for template_type in EmailTemplateType::ALL {
        for language in LANGUAGES {
            let found = rows
                .iter()
                .find(|r| r.template_type == template_type && r.language == language);
            out.push(match found {
                Some(r) => to_effective_override(r),
                None => to_effective_default(template_type, language),
            });
        }
    }
    Ok(out)
}

/// Compare l'override persisté au payload soumis — `true` si `subject`/`body`
/// sont identiques (KF-004 : court-circuit no-op pour ne pas bumper
/// `version` ni écrire d'audit inutilement).
fn is_no_op_change(before: &EmailTemplate, subject: &str, body: &str) -> bool {
    before.subject == subject && before.body == body
}

/// Crée ou modifie l'override de `(company_id, template_type, language)`.
///
/// `expected_version = None` : le caller croit qu'aucun override n'existe
/// → `INSERT`. Si une ligne existe déjà (race) → `OptimisticLockConflict`.
/// `expected_version = Some(v)` : modifie l'override existant à la version
/// `v` → `UPDATE`. Ligne absente (race avec un `restore_default` concurrent)
/// ou version stale → `OptimisticLockConflict` dans les deux cas.
///
/// `SELECT ... FOR UPDATE` verrouille la ligne (ou le gap InnoDB si absente)
/// pour la durée de la transaction — élimine la race sans dépendre d'une
/// violation `UNIQUE` après coup.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_override(
    pool: &MySqlPool,
    company_id: i64,
    template_type: EmailTemplateType,
    language: Language,
    expected_version: Option<i32>,
    user_id: i64,
    subject: String,
    body: String,
) -> Result<EmailTemplate, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let existing = sqlx::query_as::<_, EmailTemplate>(&format!(
        "SELECT {COLUMNS} FROM email_templates WHERE company_id = ? AND template_type = ? AND language = ? FOR UPDATE"
    ))
    .bind(company_id)
    .bind(template_type)
    .bind(language)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let (before_json, row_after) = match (expected_version, existing) {
        (None, Some(_)) | (Some(_), None) => {
            // Race : le caller a une vue périmée de l'existence de l'override.
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        (None, None) => {
            // Création : pas de ligne, pas de version attendue.
            let insert_id = sqlx::query(
                "INSERT INTO email_templates (company_id, template_type, language, subject, body) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(company_id)
            .bind(template_type)
            .bind(language)
            .bind(&subject)
            .bind(&body)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?
            .last_insert_id() as i64;

            let created = sqlx::query_as::<_, EmailTemplate>(&format!(
                "SELECT {COLUMNS} FROM email_templates WHERE id = ?"
            ))
            .bind(insert_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;

            (serde_json::Value::Null, created)
        }
        (Some(v), Some(row)) => {
            if row.version != v {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::OptimisticLockConflict);
            }

            // KF-004 : court-circuit no-op AVANT toute mutation.
            if is_no_op_change(&row, &subject, &body) {
                tx.rollback().await.map_err(map_db_error)?;
                return Ok(row);
            }

            let before_json = template_snapshot_json(&row);

            let rows_affected = sqlx::query(
                "UPDATE email_templates SET subject = ?, body = ?, version = version + 1 \
                 WHERE id = ? AND version = ?",
            )
            .bind(&subject)
            .bind(&body)
            .bind(row.id)
            .bind(v)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();

            if rows_affected == 0 {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::OptimisticLockConflict);
            }

            let updated = sqlx::query_as::<_, EmailTemplate>(&format!(
                "SELECT {COLUMNS} FROM email_templates WHERE id = ?"
            ))
            .bind(row.id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;

            (before_json, updated)
        }
    };

    let audit_details = serde_json::json!({
        "before": before_json,
        "after": template_snapshot_json(&row_after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "email_template.updated".to_string(),
            "email_template".to_string(),
            row_after.id,
            Some(audit_details),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(row_after)
}

/// Restaure le défaut pour `(company_id, template_type, language)` en
/// supprimant l'override s'il existe. **Idempotent** : aucune ligne existante
/// → no-op silencieux (pas d'erreur, pas d'audit).
pub async fn restore_default(
    pool: &MySqlPool,
    company_id: i64,
    template_type: EmailTemplateType,
    language: Language,
    user_id: i64,
) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let existing = sqlx::query_as::<_, EmailTemplate>(&format!(
        "SELECT {COLUMNS} FROM email_templates WHERE company_id = ? AND template_type = ? AND language = ? FOR UPDATE"
    ))
    .bind(company_id)
    .bind(template_type)
    .bind(language)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let Some(row) = existing else {
        // Idempotent : déjà sur le défaut, rien à faire, pas d'audit.
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(());
    };

    sqlx::query("DELETE FROM email_templates WHERE id = ?")
        .bind(row.id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let audit_details = serde_json::json!({ "before": template_snapshot_json(&row) });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "email_template.restored_default".to_string(),
            "email_template".to_string(),
            row.id,
            Some(audit_details),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}
