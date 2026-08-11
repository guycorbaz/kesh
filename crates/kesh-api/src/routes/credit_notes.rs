//! Story 12.1 — endpoints avoirs (notes de crédit).
//!
//! - `GET    /api/v1/credit-notes`        liste paginée (tout rôle)
//! - `POST   /api/v1/credit-notes`        create+issue (Comptable+, RBAC au routing)
//! - `GET    /api/v1/credit-notes/{id}`   détail + lignes (tout rôle)
//! - `GET    /api/v1/credit-notes/{id}/pdf` PDF « Avoir » sans QR Bill (tout rôle)

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{NaiveDate, NaiveDateTime};
use kesh_db::entities::{CreditNote, CreditNoteLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::{contacts, credit_notes};
use kesh_qrbill::{Currency, InvoiceLinePdf, InvoicePdfData};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;
use crate::routes::invoice_pdf_service::{
    build_i18n, map_qrbill_error, sanitize_filename, split_lines,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteLineResponse {
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    /// Compte de produit débité par cette ligne d'avoir — **lorsqu'il est
    /// renseigné** (Story 16-1a). Symétrique de
    /// `InvoiceLineResponse.revenue_account_id`.
    ///
    /// Recopié tel quel depuis la ligne de facture au moment de l'émission. Il
    /// vaut donc `null` quand la ligne de facture d'origine est elle-même
    /// `null`. Depuis la Story 16-1a-bis, le parc validé avant 16-1a a été
    /// **backfillé** : le cas résiduel se limite aux pièces dont l'écriture
    /// comptable a été retouchée à la main, que le backfill refuse
    /// délibérément de reprendre faute de pouvoir identifier le compte sans
    /// ambiguïté (décision D-B2). Un avoir émis **avant** ce backfill conserve
    /// par ailleurs le compte que son écriture avait réellement débité, qui
    /// peut légitimement différer de celui de sa facture (D-B7).
    ///
    /// ⚠️ `null` ne signifie **pas** « aucune imputation » : l'écriture débite
    /// alors le compte de produit **par défaut de la société tel qu'il est au
    /// moment de l'avoir** (`credit_notes::generate_credit_note_journal_lines`,
    /// `unwrap_or(default_revenue_account_id)`), lequel peut différer du compte
    /// réellement crédité par la facture. Un client qui contrôle la
    /// contre-passation compte par compte doit traiter ce cas explicitement —
    /// cf. le test `null_line_credit_note_falls_back_to_current_default_known_limitation`
    /// et l'avertissement correspondant du CHANGELOG.
    pub revenue_account_id: Option<i64>,
}

impl From<CreditNoteLine> for CreditNoteLineResponse {
    fn from(l: CreditNoteLine) -> Self {
        Self {
            position: l.position,
            description: l.description,
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
            revenue_account_id: l.revenue_account_id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteResponse {
    pub id: i64,
    pub contact_id: i64,
    pub invoice_id: i64,
    pub credit_note_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub total_amount: Decimal,
    pub journal_entry_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub lines: Vec<CreditNoteLineResponse>,
}

impl CreditNoteResponse {
    fn from_parts(cn: CreditNote, lines: Vec<CreditNoteLine>) -> Self {
        Self {
            id: cn.id,
            contact_id: cn.contact_id,
            invoice_id: cn.invoice_id,
            credit_note_number: cn.credit_note_number,
            status: cn.status,
            date: cn.date,
            total_amount: cn.total_amount,
            journal_entry_id: cn.journal_entry_id,
            version: cn.version,
            created_at: cn.created_at,
            lines: lines.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteListItemResponse {
    pub id: i64,
    pub contact_id: i64,
    pub invoice_id: i64,
    pub credit_note_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub total_amount: Decimal,
}

impl From<CreditNote> for CreditNoteListItemResponse {
    fn from(cn: CreditNote) -> Self {
        Self {
            id: cn.id,
            contact_id: cn.contact_id,
            invoice_id: cn.invoice_id,
            credit_note_number: cn.credit_note_number,
            status: cn.status,
            date: cn.date,
            total_amount: cn.total_amount,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCreditNoteRequest {
    pub invoice_id: i64,
    pub date: NaiveDate,
}

#[derive(Deserialize)]
pub struct ListCreditNotesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// `GET /api/v1/credit-notes` — liste paginée (les plus récents d'abord).
pub async fn list_credit_notes(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListCreditNotesQuery>,
) -> Result<Json<ListResponse<CreditNoteListItemResponse>>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);
    let (items, total) = credit_notes::list(&state.pool, company.id, limit, offset).await?;
    Ok(Json(ListResponse {
        items: items.into_iter().map(Into::into).collect(),
        total,
        offset,
        limit,
    }))
}

/// `GET /api/v1/credit-notes/{id}` — détail + lignes.
pub async fn get_credit_note(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<CreditNoteResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let (cn, lines) = credit_notes::get(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(CreditNoteResponse::from_parts(cn, lines)))
}

/// `POST /api/v1/credit-notes` — crée et émet un avoir (Comptable+, RBAC routing).
pub async fn create_credit_note(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateCreditNoteRequest>,
) -> Result<(StatusCode, Json<CreditNoteResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let issued = credit_notes::create_credit_note(
        &state.pool,
        kesh_db::entities::NewCreditNote {
            company_id: company.id,
            invoice_id: req.invoice_id,
            date: req.date,
        },
        current_user.user_id,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreditNoteResponse::from_parts(
            issued.credit_note,
            issued.lines,
        )),
    ))
}

/// Construit les données PDF d'un **avoir** — extraite du handler (Story 16-3a,
/// #151) pour être **testable sans base de données**, sur le patron de
/// `build_qrbill_inputs` côté facture.
///
/// ⚠️ Cette extraction n'est pas cosmétique : facture et avoir partagent la
/// **même** fonction de rendu (`draw_invoice_section`), si bien que la seule
/// divergence possible entre les deux documents est **ici**, au site de
/// construction. Un test posé dans `kesh-qrbill` ne pourrait donc pas voir un
/// champ oublié côté avoir — il resterait vert. C'est le seul niveau où la
/// couverture de l'avoir est réelle.
#[allow(clippy::too_many_arguments)]
fn build_credit_note_pdf_data(
    cn: &CreditNote,
    pdf_lines: Vec<InvoiceLinePdf>,
    contact: &kesh_db::entities::Contact,
    company: &kesh_db::entities::Company,
    subtotal_ht: Decimal,
    vat_lines: Vec<kesh_qrbill::InvoiceVatLinePdf>,
    ttc: Decimal,
    origin_reference: Option<String>,
) -> InvoicePdfData {
    InvoicePdfData {
        invoice_number: cn
            .credit_note_number
            .clone()
            .unwrap_or_else(|| format!("#{}", cn.id)),
        invoice_date: cn.date,
        due_date: None,
        payment_terms: None,
        creditor_name: company.name.clone(),
        creditor_address_lines: split_lines(&company.address),
        creditor_ide: company.ide_number.clone(),
        // Story 16-3a (#151) — coordonnées de contact, ici pour l'AVOIR.
        creditor_phone: company.phone.clone(),
        creditor_email: company.email.clone(),
        creditor_website: company.website.clone(),
        debtor_name: contact.name.clone(),
        debtor_address_lines: split_lines(contact.address.as_deref().unwrap_or("")),
        lines: pdf_lines,
        subtotal_ht,
        vat_lines,
        total: ttc,
        currency: Currency::Chf,
        origin_reference,
    }
}

/// `GET /api/v1/credit-notes/{id}/pdf` — PDF « Avoir » (sans QR Bill).
pub async fn get_credit_note_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let (cn, lines) = credit_notes::get(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    // Pas de pré-filtre `MAX_LINES_PER_PDF` ici : un avoir n'a PAS de section QR
    // (pleine page disponible) → sa capacité géométrique réelle est bien plus
    // haute que celle d'une facture. La garde de `draw_invoice_section`
    // (plancher = marge basse pour l'avoir) refuse proprement le dépassement en
    // 400 « trop de lignes » si besoin (#151 code-review).

    let contact = contacts::find_by_id(&state.pool, cn.contact_id)
        .await?
        .ok_or_else(|| {
            AppError::InvoiceNotPdfReady(crate::errors::t(
                "invoice-pdf-error-contact-missing",
                "Le contact lié à l'avoir est introuvable.",
            ))
        })?;

    // Numéro de la facture d'origine (pour la référence affichée).
    let origin_number: Option<String> =
        sqlx::query_scalar("SELECT invoice_number FROM invoices WHERE id = ? AND company_id = ?")
            .bind(cn.invoice_id)
            .bind(company.id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(kesh_db::errors::map_db_error(e)))?
            .flatten();

    let pdf_lines: Vec<InvoiceLinePdf> = lines
        .iter()
        .map(|l| InvoiceLinePdf {
            description: l.description.clone(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
        })
        .collect();

    // TTC = HT + TVA (cohérent avec la contre-passation) — helper canonique
    // #246 (Story 21-2a), même arithmétique que le débit créance.
    let ttc: Decimal = kesh_core::accounting::vat::invoice_total_ttc(
        lines.iter().map(|l| (l.line_total, l.vat_rate)),
    );
    // #151 : récap TVA de l'avoir (montants positifs — la contre-passation gère
    // le signe séparément ; le PDF « Avoir » présente les montants crédités).
    let subtotal_ht: Decimal = lines.iter().map(|l| l.line_total).sum();
    let vat_lines = crate::routes::invoice_pdf_service::vat_lines_pdf(
        lines.iter().map(|l| (l.line_total, l.vat_rate)),
    );

    let pdf_data = build_credit_note_pdf_data(
        &cn,
        pdf_lines,
        &contact,
        &company,
        subtotal_ht,
        vat_lines,
        ttc,
        origin_number,
    );

    // i18n facture + surcharge titre/numéro « Avoir » (clés credit-note-pdf-*).
    let mut i18n = build_i18n(&state.i18n, state.config.locale);
    i18n.entries.insert(
        "invoice-pdf-title",
        state
            .i18n
            .format(&state.config.locale, "credit-note-pdf-title", None),
    );
    i18n.entries.insert(
        "invoice-pdf-number",
        state
            .i18n
            .format(&state.config.locale, "credit-note-pdf-number", None),
    );

    let pdf_bytes =
        kesh_qrbill::generate_credit_note_pdf(&pdf_data, &i18n).map_err(map_qrbill_error)?;

    let filename = sanitize_filename(cn.credit_note_number.as_deref().unwrap_or("avoir"));
    let disposition = format!("inline; filename=\"avoir-{filename}.pdf\"");

    let mut resp = (StatusCode::OK, pdf_bytes).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition).unwrap_or_else(|_| HeaderValue::from_static("inline")),
    );
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn company_with_contact_details() -> kesh_db::entities::Company {
        kesh_db::entities::Company {
            id: 1,
            name: "Démo SA".into(),
            first_name: None,
            last_name: None,
            address: "Rue du Lac 1\n1000 Lausanne".into(),
            address_street: "Rue du Lac".into(),
            address_building: "1".into(),
            address_postal_code: "1000".into(),
            address_city: "Lausanne".into(),
            address_country: "CH".into(),
            ide_number: Some("CHE-123.456.789".into()),
            org_type: kesh_db::entities::OrgType::Pme,
            accounting_language: kesh_db::entities::Language::Fr,
            instance_language: kesh_db::entities::Language::Fr,
            email: Some("contact@demo.ch".into()),
            phone: Some("+41 21 123 45 67".into()),
            website: Some("https://demo.ch".into()),
            is_stub: false,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    fn contact() -> kesh_db::entities::Contact {
        kesh_db::entities::Contact {
            id: 1,
            company_id: 1,
            contact_type: kesh_db::entities::ContactType::Entreprise,
            name: "Client SA".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Marktgasse 28\n9400 Rorschach".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Default::default(),
            active: true,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    fn credit_note() -> CreditNote {
        CreditNote {
            id: 1,
            company_id: 1,
            contact_id: 1,
            invoice_id: 1,
            credit_note_number: Some("A-2026-0001".into()),
            status: "issued".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 8, 6).unwrap(),
            total_amount: dec!(100.00),
            journal_entry_id: None,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        }
    }

    /// **AC5** — l'AVOIR porte les coordonnées de l'émetteur, comme la facture.
    ///
    /// ⚠️ **Ce test ne peut PAS vivre dans `kesh-qrbill`**, et c'est tout son
    /// intérêt : facture et avoir partagent la **même** fonction de rendu
    /// (`draw_invoice_section`, un seul appel par générateur). La seule
    /// divergence possible entre les deux documents est donc **au site de
    /// construction** — ici. Un test de rendu resterait vert alors que l'avoir
    /// sortirait sans coordonnées, ce que D3 qualifie de « piège qui coûterait
    /// le plus cher ».
    ///
    /// ⚠️ Et il ne part **pas** d'une fixture de facture par `..base` : le
    /// champ serait hérité et le test passerait **sans exercer aucun code
    /// d'avoir**. La `Company` est construite entière, exprès.
    #[test]
    fn credit_note_pdf_carries_the_issuer_contact_details() {
        let company = company_with_contact_details();
        let data = build_credit_note_pdf_data(
            &credit_note(),
            vec![],
            &contact(),
            &company,
            dec!(100.00),
            vec![],
            dec!(100.00),
            None,
        );

        assert_eq!(
            data.creditor_phone.as_deref(),
            Some("+41 21 123 45 67"),
            "l'avoir doit porter le téléphone de l'émetteur"
        );
        assert_eq!(
            data.creditor_email.as_deref(),
            Some("contact@demo.ch"),
            "l'avoir doit porter l'e-mail de l'émetteur"
        );
        assert_eq!(
            data.creditor_website.as_deref(),
            Some("https://demo.ch"),
            "l'avoir doit porter le site web de l'émetteur"
        );
    }
}
