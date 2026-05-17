//! Story 9-2b T5 — Handler HTTP `GET /api/v1/exports/global.zip`.
//!
//! Pipeline (cf. spec T5.1) :
//! 1. Vérifie `current_user.company_id > 0` (AC #19 défensif).
//! 2. Charge la `Company` (filename + manifest).
//! 3. Résout `locale_bcp47` via `util::map_language_to_bcp47`.
//! 4. Calcule le filename `kesh-export-{slug}-{YYYY-MM-DD}.zip` (UTC).
//! 5. Crée le tracing span `global_export` (fields placeholder
//!    `tracing::field::Empty` populés post-render).
//! 6. Appelle `exports::global::build_global_export(pool, company, locale)`.
//! 7. `span.record("byte_size" / "csv_count" / "duration_ms", …)`.
//! 8. Émet l'audit `exports.global` best-effort.
//! 9. Construit la réponse 200 (`application/zip` + `Content-Disposition`).

use axum::{
    Extension,
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
};
use chrono::{NaiveDate, Utc};
use sqlx::MySqlPool;

use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;

use crate::AppState;
use crate::errors::AppError;
use crate::exports::global::build_global_export;
use crate::middleware::auth::CurrentUser;

/// GET /api/v1/exports/global.zip — Export global ZIP de souveraineté.
///
/// Route mountée dans `authenticated_routes` (Story 9-2b T7.1, anti-IDOR
/// Pass 1 BH-H1). Tous rôles authentifiés (AC #11 Consultation 200).
pub async fn export_global(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Response, AppError> {
    // AC #19 — garde défensif `company_id > 0`. Le middleware injecte la
    // claim JWT, mais on documente cette invariant côté handler (L18 :
    // si une future migration du middleware retire la garantie, ce
    // contrôle bloque l'IDOR plutôt que de leaker un export vide).
    if current_user.company_id <= 0 {
        return Err(AppError::Forbidden);
    }

    let company =
        kesh_db::repositories::companies::find_by_id(&state.pool, current_user.company_id)
            .await?
            .ok_or(AppError::Forbidden)?;

    // Decision §locale-source — mapping enum DB SCREAMING → BCP-47 régional Suisse.
    let locale_bcp47 = crate::util::map_language_to_bcp47(company.accounting_language.as_str());

    // Decision §filename — `kesh-export-{slug}-{YYYY-MM-DD}.zip` (UTC, L13).
    let export_date = Utc::now().date_naive();
    let filename = build_global_filename(&company.name, export_date);

    // Tracing span — fields populated post-render via `span.record(...)`
    // (cohérent Story 9-2a Pass 3 BH3-M1 — `info_span!` évalue les fields à
    // la création, donc placeholder `tracing::field::Empty` mandaté).
    //
    // Pass 1 code-review H1 (C1 Blind F01 + C1 ECH 1 + C1 AA-MEDIUM-02) :
    // utiliser `.instrument(span.clone()).await` plutôt que `let _enter = span.enter()`.
    // En contexte async, `span.enter()` retourne un guard non-`Send` qui peut
    // corrompre le contexte de tracing si la future est re-schedulée sur un
    // autre thread tokio. `.instrument()` propage le span proprement à travers
    // tous les points `.await`.
    use tracing::Instrument;
    let span = tracing::info_span!(
        "global_export",
        company_id = current_user.company_id,
        byte_size = tracing::field::Empty,
        csv_count = tracing::field::Empty,
        duration_ms = tracing::field::Empty,
    );

    let (zip_bytes, meta) = build_global_export(&state.pool, &company, locale_bcp47)
        .instrument(span.clone())
        .await?;

    span.record("byte_size", meta.byte_size);
    span.record("csv_count", meta.csv_count);
    span.record("duration_ms", meta.duration_ms);

    // Audit best-effort — INSERT échec → warn + retour 200 (UX-DR38 :
    // ne jamais faire échouer le download user-facing).
    emit_global_export_audit(
        &state.pool,
        current_user.user_id,
        current_user.company_id,
        meta.byte_size,
        meta.csv_count,
        "all",
        meta.duration_ms,
    )
    .instrument(span.clone())
    .await;

    // Content-Disposition RFC 5987 + ASCII fallback (helper partagé Story 9-2a).
    let content_disposition = crate::util::build_content_disposition(&filename, locale_bcp47)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(Body::from(zip_bytes))
        .map_err(|e| AppError::Internal(format!("response build: {e}")))
}

/// Decision §filename — Construit le filename `kesh-export-{slug}-{YYYY-MM-DD}.zip`.
///
/// Signature distincte de `build_filename` Story 9-2a (qui prend une
/// `ReportPeriod`) : pour l'export global, le scope est `"all"` donc seule
/// la date d'export figure dans le filename. La date est en **UTC** (L13).
///
/// `slugify(company_name, "company")` truncate à 20 chars (Story 9-2a)
/// → filename composé ~47 chars max.
pub(crate) fn build_global_filename(company_name: &str, export_date: NaiveDate) -> String {
    let slug = crate::util::slugify(company_name, "company");
    format!(
        "kesh-export-{}-{}.zip",
        slug,
        export_date.format("%Y-%m-%d")
    )
}

/// Émet une ligne `audit_log` `action = 'exports.global'` (best-effort).
///
/// Distincte de `report.exported` (Story 9-2a) — scope sémantique différent
/// (toutes tables vs 1 rapport agrégé). `entity_type = 'export'`,
/// `entity_id = AUDIT_ENTITY_ID_NONE` (pas d'entité 1:1).
///
/// `details_json` inclut `company_id` (Pass 3 ECH3-C1 — la table `audit_log`
/// n'a pas de colonne `company_id`, donc les requêtes multi-tenant doivent
/// passer par `details_json->>'$.company_id'` ou par FK `users.company_id`).
#[allow(clippy::too_many_arguments)]
async fn emit_global_export_audit(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    byte_size: usize,
    csv_count: usize,
    fiscal_year_scope: &str,
    duration_ms: u64,
) {
    let result = async {
        let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id,
                action: "exports.global".to_string(),
                entity_type: "export".to_string(),
                entity_id: AUDIT_ENTITY_ID_NONE,
                // Pass 1 code-review H2 (C1 AA-MEDIUM-03) — clés snake_case pour
                // permettre les SQL JSON paths `details_json->>'$.company_id'`
                // (cohérent AC #23 + AC #29(m) ground-truth spec).
                details_json: Some(serde_json::json!({
                    "company_id": company_id,
                    "byte_size": byte_size,
                    "csv_count": csv_count,
                    "fiscal_year_scope": fiscal_year_scope,
                    "duration_ms": duration_ms,
                })),
            },
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(
            error = ?e,
            user_id,
            company_id,
            byte_size,
            csv_count,
            "audit insert failed (exports.global) — non-blocking"
        );
    }
}

// ===========================================================================
// Tests unit (Story 9-2b T5 — build_global_filename)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_global_filename_kesh_pattern() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
        assert_eq!(
            build_global_filename("CI Test Company", date),
            "kesh-export-ci-test-company-2026-05-17.zip"
        );
    }

    #[test]
    fn build_global_filename_handles_non_ascii_company() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
        let name = build_global_filename("Müller AG", date);
        assert_eq!(name, "kesh-export-muller-ag-2026-05-17.zip");
    }

    #[test]
    fn build_global_filename_truncates_long_company() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
        let name = build_global_filename("Acme SA Fribourg Extension Long", date);
        // slug truncated to 20 chars + strip trailing `-`
        assert!(
            name.starts_with("kesh-export-") && name.ends_with("-2026-05-17.zip"),
            "got: {name}"
        );
        // Total length ≤ kesh-export- (12) + 20 + -YYYY-MM-DD.zip (15) = 47 max
        assert!(name.len() <= 47, "got: {name}");
    }

    #[test]
    fn build_global_filename_empty_falls_back_to_company() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
        assert_eq!(
            build_global_filename("", date),
            "kesh-export-company-2026-05-17.zip"
        );
    }
}
