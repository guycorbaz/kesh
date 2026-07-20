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
//! **Deux chemins transactionnels distincts dans `upsert_override`**
//! (code-review Pass 1) : le chemin création (`expected_version = None`)
//! tente l'`INSERT` **sans lecture verrouillante préalable**. Un
//! `SELECT ... FOR UPDATE` sur une ligne absente prend un gap lock InnoDB ;
//! or un gap lock n'empêche PAS une autre transaction de tenir aussi son
//! propre gap lock compatible sur la même lacune — seul l'`INSERT` lui-même
//! est bloquant. Deux transactions faisant chacune `SELECT FOR UPDATE` puis
//! `INSERT` sur la même lacune risquent donc de deadlocker (pattern InnoDB
//! documenté : deux gap locks partagés + deux tentatives d'écriture
//! symétriques dans la même lacune). Sans lecture préalable, l'`INSERT`
//! gère seul son verrouillage : la seconde transaction attend puis échoue
//! proprement sur `UNIQUE` (MariaDB 1062, `DbError::UniqueConstraintViolation`),
//! remappé en `OptimisticLockConflict` — comportement vérifié par un test de
//! concurrence réelle (`tokio::join!`, pas seulement séquentiel), cf.
//! `upsert_override_true_concurrent_create_yields_exactly_one_conflict`.
//! Le chemin modification (`Some(v)`) garde le `SELECT ... FOR UPDATE`
//! classique (verrou de ligne existante, pas de gap lock — pas le même
//! risque de deadlock) + `restore_default`.
//!
//! **Audit** : `NewAuditLogEntry::for_actor(user_id, actor_api_key_id, ...)`
//! — thread le PAT (Story 17-2a DC5) conformément à l'AC #11 de la spec.

use sqlx::mysql::MySqlPool;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{EffectiveEmailTemplate, EmailTemplate, EmailTemplateType, Language};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str = "id, company_id, template_type, language, level_number, subject, body, version, created_at, updated_at";

const LANGUAGES: [Language; 4] = [Language::Fr, Language::De, Language::It, Language::En];

/// Nombre de niveaux de rappel garantis (défauts Rust 1..3) — la borne
/// dynamique de `list_effective_for_company` ne descend jamais en dessous.
pub const MIN_REMINDER_LEVELS: i16 = 3;

fn template_snapshot_json(t: &EmailTemplate) -> serde_json::Value {
    serde_json::json!({
        "id": t.id,
        "companyId": t.company_id,
        "templateType": t.template_type.as_str(),
        "language": t.language.as_str(),
        "levelNumber": t.level_number,
        "subject": t.subject,
        "body": t.body,
        "version": t.version,
    })
}

/// Convertit un override persisté en template effectif. `requested_level` = le
/// SLOT demandé (peut différer de `row.level_number` quand la cascade retombe
/// sur l'override niveau 0 générique pour un slot N > 0) — cf. sémantique
/// `EffectiveEmailTemplate::level_number`.
fn to_effective_override(row: &EmailTemplate, requested_level: i16) -> EffectiveEmailTemplate {
    EffectiveEmailTemplate {
        template_type: row.template_type,
        language: row.language,
        level_number: requested_level,
        subject: row.subject.clone(),
        body: row.body.clone(),
        version: Some(row.version),
        is_default: false,
        allowed_variables: row.template_type.allowed_variables_owned(),
    }
}

fn to_effective_default(
    template_type: EmailTemplateType,
    language: Language,
    level_number: i16,
) -> EffectiveEmailTemplate {
    let (subject, body) = crate::entities::default_template(template_type, language, level_number);
    EffectiveEmailTemplate {
        template_type,
        language,
        level_number,
        subject: subject.to_string(),
        body: body.to_string(),
        version: None,
        is_default: true,
        allowed_variables: template_type.allowed_variables_owned(),
    }
}

/// Résout le template effectif pour `(company_id, template_type, language, level_number)` —
/// jamais d'erreur pour une combinaison valide (l'absence d'override retombe sur le défaut).
///
/// Cascade de résolution (Epic 21) : override DB `(type, langue, N)` → override DB
/// `(type, langue, 0)` générique → défaut Rust niveau N → défaut Rust générique. Une seule
/// requête (`level_number IN (N, 0) ORDER BY level_number DESC LIMIT 1`) préfère l'override
/// de niveau N à l'override générique. Le `level_number` retourné = le SLOT demandé (le
/// paramètre), pas la source de la cascade. Pour `invoice_send`, passer `level_number = 0`.
pub async fn get_effective(
    pool: &MySqlPool,
    company_id: i64,
    template_type: EmailTemplateType,
    language: Language,
    level_number: i16,
) -> Result<EffectiveEmailTemplate, DbError> {
    let row = sqlx::query_as::<_, EmailTemplate>(&format!(
        "SELECT {COLUMNS} FROM email_templates \
         WHERE company_id = ? AND template_type = ? AND language = ? AND level_number IN (?, 0) \
         ORDER BY level_number DESC LIMIT 1"
    ))
    .bind(company_id)
    .bind(template_type)
    .bind(language)
    .bind(level_number)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?;

    Ok(match row {
        Some(r) => to_effective_override(&r, level_number),
        None => to_effective_default(template_type, language, level_number),
    })
}

/// Résout les templates effectifs pour toutes les combinaisons
/// `type × langue × niveau`. `invoice_send` reste au seul niveau 0 ;
/// `invoice_reminder` expose les niveaux `0..=max(max_reminder_level, 3)`
/// (0 = générique, 1.. = niveaux configurés + défauts Rust). `max_reminder_level`
/// est fourni par l'appelant (via `dunning_levels::count_for_company`) — le repo
/// `email_templates` reste découplé de `dunning_levels`. Une seule requête charge
/// les overrides, puis complète par la cascade (override niveau N → niveau 0 → défaut).
pub async fn list_effective_for_company(
    pool: &MySqlPool,
    company_id: i64,
    max_reminder_level: i16,
) -> Result<Vec<EffectiveEmailTemplate>, DbError> {
    let rows = sqlx::query_as::<_, EmailTemplate>(&format!(
        "SELECT {COLUMNS} FROM email_templates WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    let reminder_max = max_reminder_level.max(MIN_REMINDER_LEVELS);
    let mut out = Vec::new();
    for template_type in EmailTemplateType::ALL {
        let levels: Vec<i16> = match template_type {
            EmailTemplateType::InvoiceSend => vec![0],
            EmailTemplateType::InvoiceReminder => (0..=reminder_max).collect(),
        };
        for language in LANGUAGES {
            for &level in &levels {
                // Cascade sur les rows préchargées : override niveau exact,
                // sinon override niveau 0 générique (pour level > 0), sinon défaut.
                let found = rows
                    .iter()
                    .find(|r| {
                        r.template_type == template_type
                            && r.language == language
                            && r.level_number == level
                    })
                    .or_else(|| {
                        if level != 0 {
                            rows.iter().find(|r| {
                                r.template_type == template_type
                                    && r.language == language
                                    && r.level_number == 0
                            })
                        } else {
                            None
                        }
                    });
                out.push(match found {
                    Some(r) => to_effective_override(r, level),
                    None => to_effective_default(template_type, language, level),
                });
            }
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

/// Écrit l'entrée d'audit `email_template.updated` dans la transaction ;
/// rollback + propagation si l'insertion audit échoue. Partagé par les deux
/// chemins création/modification de [`upsert_override`].
async fn audit_update_and_commit(
    mut tx: sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    actor_api_key_id: Option<i64>,
    before_json: serde_json::Value,
    row_after: EmailTemplate,
) -> Result<EmailTemplate, DbError> {
    let audit_details = serde_json::json!({
        "before": before_json,
        "after": template_snapshot_json(&row_after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
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

/// Crée ou modifie l'override de `(company_id, template_type, language)`.
///
/// `expected_version = None` : le caller croit qu'aucun override n'existe
/// → `INSERT`. `expected_version = Some(v)` : modifie l'override existant à
/// la version `v` → `UPDATE`.
///
/// **Deux chemins transactionnels distincts, volontairement** (code-review
/// Pass 1) : le chemin création n'effectue AUCUNE lecture verrouillante
/// préalable — il tente l'`INSERT` directement. Un `SELECT ... FOR UPDATE`
/// sur une ligne absente avant l'`INSERT` (comme le faisait la version
/// initiale) prend un gap lock InnoDB ; deux transactions concurrentes
/// peuvent chacune obtenir ce gap lock (compatible entre elles), puis
/// risquer un deadlock en tentant chacune l'`INSERT` ensuite (pattern InnoDB
/// documenté). Sans lecture préalable, l'`INSERT` gère son propre
/// verrouillage : la seconde transaction attend simplement la première puis
/// échoue proprement sur `UNIQUE` (1062), remappé en `OptimisticLockConflict`
/// — vérifié par un test de concurrence réelle (`tokio::join!`), cf.
/// `upsert_override_true_concurrent_create_yields_exactly_one_conflict`.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_override(
    pool: &MySqlPool,
    company_id: i64,
    template_type: EmailTemplateType,
    language: Language,
    level_number: i16,
    expected_version: Option<i32>,
    user_id: i64,
    actor_api_key_id: Option<i64>,
    subject: String,
    body: String,
) -> Result<EmailTemplate, DbError> {
    match expected_version {
        None => {
            let mut tx = pool.begin().await.map_err(map_db_error)?;

            let insert_result = sqlx::query(
                "INSERT INTO email_templates (company_id, template_type, language, level_number, subject, body) \
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(company_id)
            .bind(template_type)
            .bind(language)
            .bind(level_number)
            .bind(&subject)
            .bind(&body)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error);

            let insert_id = match insert_result {
                Ok(result) => result.last_insert_id() as i64,
                // Quelqu'un d'autre a créé la ligne entre-temps (race sur
                // (company_id, template_type, language)) → conflit optimiste,
                // pas une violation UNIQUE brute.
                Err(DbError::UniqueConstraintViolation(_)) => {
                    tx.rollback().await.map_err(map_db_error)?;
                    return Err(DbError::OptimisticLockConflict);
                }
                Err(e) => {
                    tx.rollback().await.map_err(map_db_error)?;
                    return Err(e);
                }
            };

            let created = sqlx::query_as::<_, EmailTemplate>(&format!(
                "SELECT {COLUMNS} FROM email_templates WHERE id = ?"
            ))
            .bind(insert_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;

            audit_update_and_commit(
                tx,
                user_id,
                actor_api_key_id,
                serde_json::Value::Null,
                created,
            )
            .await
        }
        Some(v) => {
            let mut tx = pool.begin().await.map_err(map_db_error)?;

            let existing = sqlx::query_as::<_, EmailTemplate>(&format!(
                "SELECT {COLUMNS} FROM email_templates WHERE company_id = ? AND template_type = ? AND language = ? AND level_number = ? FOR UPDATE"
            ))
            .bind(company_id)
            .bind(template_type)
            .bind(language)
            .bind(level_number)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;

            let Some(row) = existing else {
                // Ligne disparue (race avec un `restore_default` concurrent).
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::OptimisticLockConflict);
            };

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

            audit_update_and_commit(tx, user_id, actor_api_key_id, before_json, updated).await
        }
    }
}

/// Restaure le défaut pour `(company_id, template_type, language)` en
/// supprimant l'override s'il existe. **Idempotent** : aucune ligne existante
/// → no-op silencieux (pas d'erreur, pas d'audit).
pub async fn restore_default(
    pool: &MySqlPool,
    company_id: i64,
    template_type: EmailTemplateType,
    language: Language,
    level_number: i16,
    user_id: i64,
    actor_api_key_id: Option<i64>,
) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let existing = sqlx::query_as::<_, EmailTemplate>(&format!(
        "SELECT {COLUMNS} FROM email_templates WHERE company_id = ? AND template_type = ? AND language = ? AND level_number = ? FOR UPDATE"
    ))
    .bind(company_id)
    .bind(template_type)
    .bind(language)
    .bind(level_number)
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
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
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
