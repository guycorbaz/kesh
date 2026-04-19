//! Routes CRUD pour les factures brouillon (Story 5.1 — FR31, FR32).
//!
//! - GETs (`list`, `get`) → `authenticated_routes` (tout rôle).
//! - Mutations (`create`, `update`, `delete`) → `comptable_routes`.
//!
//! `total_amount` est recalculé par le repository à partir des lignes —
//! le frontend peut l'afficher en temps réel, mais la valeur persistée
//! est celle du backend.
//!
//! **Security Note (Story 6.2):** All handlers scope by `current_user.company_id` from JWT.
//! The company_id in JWT can become stale if a user is reassigned to a different company
//! during an active session. See `middleware/auth.rs` for staleness window (proportional to
//! `KESH_JWT_EXPIRY_MINUTES`, default 15 min).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use kesh_core::listing::SortDirection;
use kesh_db::entities::invoice::{Invoice, InvoiceLine, InvoiceUpdate, NewInvoice, NewInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::{
    contacts,
    invoices::{
        self, DueDatesSummary, InvoiceListItem, InvoiceListQuery, InvoiceSortBy,
        PaymentStatusFilter,
    },
};
use kesh_i18n::formatting::{format_date, format_money};
use kesh_i18n::{FluentArgs, Locale};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;
use crate::routes::limits::{
    MAX_DECIMAL_SCALE, MAX_LINE_TOTAL, MAX_QUANTITY, MAX_UNIT_PRICE, scale_within,
};
use crate::routes::vat::validate_vat_rate;

// ---------------------------------------------------------------------------
// Limites
// ---------------------------------------------------------------------------

const MAX_DESCRIPTION_LEN: usize = 1000;
const MAX_PAYMENT_TERMS_LEN: usize = 255;
const MAX_LIST_LIMIT: i64 = 100;
const DEFAULT_LIST_LIMIT: i64 = 20;
const MAX_SEARCH_LEN: usize = 100;
/// Cap lignes par facture : anti-DoS (N+1 INSERT dans transaction) + anti-overflow
/// `total_amount` (avec MAX_LINE_TOTAL = 10¹², 200 lignes = 2·10¹⁴ < DECIMAL(19,4) max).
const MAX_LINES: usize = 200;

const VAT_ERROR_MSG: &str = "Taux TVA non autorisé. Valeurs acceptées : 0.00%, 2.60%, 3.80%, 8.10%";

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceLineRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceRequest {
    pub contact_id: i64,
    pub date: NaiveDate,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub payment_terms: Option<String>,
    pub lines: Vec<CreateInvoiceLineRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInvoiceRequest {
    pub contact_id: i64,
    pub date: NaiveDate,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub payment_terms: Option<String>,
    pub lines: Vec<CreateInvoiceLineRequest>,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInvoicesQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub contact_id: Option<i64>,
    #[serde(default)]
    pub date_from: Option<NaiveDate>,
    #[serde(default)]
    pub date_to: Option<NaiveDate>,
    #[serde(default)]
    pub sort_by: Option<InvoiceSortBy>,
    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineResponse {
    pub id: i64,
    pub invoice_id: i64,
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    pub created_at: NaiveDateTime,
}

impl From<InvoiceLine> for InvoiceLineResponse {
    fn from(l: InvoiceLine) -> Self {
        Self {
            id: l.id,
            invoice_id: l.invoice_id,
            position: l.position,
            description: l.description,
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
            created_at: l.created_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceResponse {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub invoice_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub total_amount: Decimal,
    pub journal_entry_id: Option<i64>,
    pub paid_at: Option<NaiveDateTime>,
    /// P6 (review pass 2) : `is_overdue` calculé backend (source unique de
    /// vérité pour « aujourd'hui » — évite la désync TZ client/serveur).
    /// `true` ssi `status == 'validated' && paid_at IS NULL && due_date < today_utc`.
    pub is_overdue: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub lines: Vec<InvoiceLineResponse>,
}

/// B3 (review pass 1 G2 B) : règle « en retard » centralisée — une seule
/// définition pour `InvoiceResponse`, `DueDateItemResponse` et le payment
/// status du CSV. Si la sémantique évolue (ex. délai de grâce), un seul
/// site à modifier.
pub fn is_invoice_overdue(
    status: &str,
    paid_at: Option<NaiveDateTime>,
    due_date: Option<NaiveDate>,
    today: NaiveDate,
) -> bool {
    status == "validated" && paid_at.is_none() && due_date.is_some_and(|d| d < today)
}

impl InvoiceResponse {
    fn from_parts(invoice: Invoice, lines: Vec<InvoiceLine>) -> Self {
        let today = chrono::Utc::now().naive_utc().date();
        let is_overdue =
            is_invoice_overdue(&invoice.status, invoice.paid_at, invoice.due_date, today);
        Self {
            id: invoice.id,
            company_id: invoice.company_id,
            contact_id: invoice.contact_id,
            invoice_number: invoice.invoice_number,
            status: invoice.status,
            date: invoice.date,
            due_date: invoice.due_date,
            payment_terms: invoice.payment_terms,
            total_amount: invoice.total_amount,
            journal_entry_id: invoice.journal_entry_id,
            paid_at: invoice.paid_at,
            is_overdue,
            version: invoice.version,
            created_at: invoice.created_at,
            updated_at: invoice.updated_at,
            lines: lines.into_iter().map(InvoiceLineResponse::from).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceListItemResponse {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub contact_name: String,
    pub invoice_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub total_amount: Decimal,
    pub paid_at: Option<NaiveDateTime>,
    /// B22 (review pass 2 G2 B) : exposé sur la liste standard pour cohérence
    /// avec `InvoiceResponse` et `DueDateItemResponse` — le frontend ne doit
    /// jamais recalculer le statut overdue côté client (désync TZ possible).
    pub is_overdue: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<InvoiceListItem> for InvoiceListItemResponse {
    fn from(i: InvoiceListItem) -> Self {
        let today = chrono::Utc::now().naive_utc().date();
        let is_overdue = is_invoice_overdue(&i.status, i.paid_at, i.due_date, today);
        Self {
            id: i.id,
            company_id: i.company_id,
            contact_id: i.contact_id,
            contact_name: i.contact_name,
            invoice_number: i.invoice_number,
            status: i.status,
            date: i.date,
            due_date: i.due_date,
            payment_terms: i.payment_terms,
            total_amount: i.total_amount,
            paid_at: i.paid_at,
            is_overdue,
            version: i.version,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn normalize_optional(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t: String = v.trim().nfc().collect();
        if t.is_empty() { None } else { Some(t) }
    })
}

fn validate_line(req: CreateInvoiceLineRequest, index: usize) -> Result<NewInvoiceLine, AppError> {
    let description: String = req.description.trim().nfc().collect();
    if description.is_empty() {
        return Err(AppError::Validation(format!(
            "Ligne {} : la description est obligatoire",
            index + 1
        )));
    }
    if description.chars().count() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(format!(
            "Ligne {} : description trop longue (max {MAX_DESCRIPTION_LEN})",
            index + 1
        )));
    }

    if req.quantity <= Decimal::ZERO {
        return Err(AppError::Validation(format!(
            "Ligne {} : la quantité doit être strictement positive",
            index + 1
        )));
    }
    if !scale_within(&req.quantity, MAX_DECIMAL_SCALE) {
        return Err(AppError::Validation(format!(
            "Ligne {} : la quantité doit avoir au plus {MAX_DECIMAL_SCALE} décimales",
            index + 1
        )));
    }
    if req.quantity > *MAX_QUANTITY {
        return Err(AppError::Validation(format!(
            "Ligne {} : quantité trop élevée",
            index + 1
        )));
    }

    if req.unit_price < Decimal::ZERO {
        return Err(AppError::Validation(format!(
            "Ligne {} : le prix unitaire doit être positif ou nul",
            index + 1
        )));
    }
    if !scale_within(&req.unit_price, MAX_DECIMAL_SCALE) {
        return Err(AppError::Validation(format!(
            "Ligne {} : le prix unitaire doit avoir au plus {MAX_DECIMAL_SCALE} décimales",
            index + 1
        )));
    }
    if req.unit_price > *MAX_UNIT_PRICE {
        return Err(AppError::Validation(format!(
            "Ligne {} : prix unitaire trop élevé",
            index + 1
        )));
    }

    if !validate_vat_rate(&req.vat_rate) {
        return Err(AppError::Validation(VAT_ERROR_MSG.into()));
    }

    // Anti-overflow : `qty × unit_price` doit rester sous `MAX_LINE_TOTAL`
    // pour garantir que `Σ line_total` sur MAX_LINES tient dans DECIMAL(19,4).
    let line_total = req.quantity * req.unit_price;
    if line_total > *MAX_LINE_TOTAL {
        return Err(AppError::Validation(format!(
            "Ligne {} : total de ligne trop élevé",
            index + 1
        )));
    }

    Ok(NewInvoiceLine {
        description,
        quantity: req.quantity,
        unit_price: req.unit_price,
        vat_rate: req.vat_rate,
    })
}

fn validate_payment_terms(pt: Option<String>) -> Result<Option<String>, AppError> {
    let norm = normalize_optional(pt);
    if let Some(ref s) = norm {
        if s.chars().count() > MAX_PAYMENT_TERMS_LEN {
            return Err(AppError::Validation(format!(
                "Les conditions de paiement doivent faire au plus {MAX_PAYMENT_TERMS_LEN} caractères"
            )));
        }
    }
    Ok(norm)
}

async fn ensure_contact_belongs_to_company(
    state: &AppState,
    contact_id: i64,
    company_id: i64,
) -> Result<(), AppError> {
    let contact = contacts::find_by_id(&state.pool, contact_id).await?;
    match contact {
        None => Err(AppError::Validation("Contact introuvable".into())),
        Some(c) if c.company_id != company_id => {
            Err(AppError::Validation("Contact introuvable".into()))
        }
        Some(c) if !c.active => Err(AppError::Validation("Contact archivé".into())),
        Some(_) => Ok(()),
    }
}

fn validate_lines(reqs: Vec<CreateInvoiceLineRequest>) -> Result<Vec<NewInvoiceLine>, AppError> {
    if reqs.is_empty() {
        return Err(AppError::Validation(
            "Une facture doit contenir au moins une ligne".into(),
        ));
    }
    if reqs.len() > MAX_LINES {
        return Err(AppError::Validation(format!(
            "Une facture doit contenir au plus {MAX_LINES} lignes"
        )));
    }
    reqs.into_iter()
        .enumerate()
        .map(|(i, l)| validate_line(l, i))
        .collect()
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list_invoices(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListInvoicesQuery>,
) -> Result<Json<ListResponse<InvoiceListItemResponse>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    if !(1..=MAX_LIST_LIMIT).contains(&limit) {
        return Err(AppError::Validation(format!(
            "limit doit être compris entre 1 et {MAX_LIST_LIMIT}"
        )));
    }
    let offset = params.offset.unwrap_or(0);
    if offset < 0 {
        return Err(AppError::Validation(
            "offset doit être positif ou nul".into(),
        ));
    }

    // Normalisation search : trim côté handler, collapse empty → None (cohérent payment_terms).
    let search = match params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(s) if s.chars().count() > MAX_SEARCH_LEN => {
            return Err(AppError::Validation(format!(
                "search doit faire au plus {MAX_SEARCH_LEN} caractères"
            )));
        }
        Some(s) => Some(s.to_string()),
        None => None,
    };

    if let Some(ref st) = params.status {
        if !matches!(st.as_str(), "draft" | "validated" | "cancelled") {
            return Err(AppError::Validation("status invalide".into()));
        }
    }

    if let (Some(df), Some(dt)) = (params.date_from, params.date_to) {
        if df > dt {
            return Err(AppError::Validation(
                "dateFrom doit être antérieur ou égal à dateTo".into(),
            ));
        }
    }

    let query = InvoiceListQuery {
        search,
        status: params.status,
        contact_id: params.contact_id,
        date_from: params.date_from,
        date_to: params.date_to,
        payment_status: None,
        due_before: None,
        sort_by: params.sort_by.unwrap_or_default(),
        sort_direction: params.sort_direction.unwrap_or(SortDirection::Desc),
        limit,
        offset,
    };

    let result =
        invoices::list_by_company_paginated(&state.pool, current_user.company_id, query).await?;
    Ok(Json(ListResponse {
        items: result
            .items
            .into_iter()
            .map(InvoiceListItemResponse::from)
            .collect(),
        total: result.total,
        limit: result.limit,
        offset: result.offset,
    }))
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let (invoice, lines) =
        invoices::find_by_id_with_lines(&state.pool, current_user.company_id, id)
            .await?
            .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(InvoiceResponse::from_parts(invoice, lines)))
}

pub async fn create_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<InvoiceResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    ensure_contact_belongs_to_company(&state, req.contact_id, company.id).await?;
    let lines = validate_lines(req.lines)?;
    let payment_terms = validate_payment_terms(req.payment_terms)?;

    let new = NewInvoice {
        company_id: company.id,
        contact_id: req.contact_id,
        date: req.date,
        // Review P6 : défaut due_date = invoice.date (Scope §12).
        // L'utilisateur peut override avec `date + N jours` selon conditions
        // de paiement. Pas de calcul auto depuis payment_terms (décision Guy).
        due_date: Some(req.due_date.unwrap_or(req.date)),
        payment_terms,
        lines,
    };

    let (invoice, persisted_lines) =
        invoices::create(&state.pool, current_user.user_id, new).await?;
    Ok((
        StatusCode::CREATED,
        Json(InvoiceResponse::from_parts(invoice, persisted_lines)),
    ))
}

pub async fn update_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    ensure_contact_belongs_to_company(&state, req.contact_id, company.id).await?;
    let lines = validate_lines(req.lines)?;
    let payment_terms = validate_payment_terms(req.payment_terms)?;

    let changes = InvoiceUpdate {
        contact_id: req.contact_id,
        date: req.date,
        // Review P6 : défaut due_date = invoice.date si non fournie (Scope §12).
        due_date: Some(req.due_date.unwrap_or(req.date)),
        payment_terms,
        lines,
    };

    let (invoice, persisted_lines) = invoices::update(
        &state.pool,
        company.id,
        id,
        req.version,
        current_user.user_id,
        changes,
    )
    .await?;
    Ok(Json(InvoiceResponse::from_parts(invoice, persisted_lines)))
}

pub async fn delete_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    invoices::delete(&state.pool, company.id, id, current_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/v1/invoices/:id/validate` (Story 5.2 — comptable_routes).
///
/// Transition atomique `draft → validated` : attribue un numéro,
/// génère l'écriture comptable, persiste. Renvoie la facture
/// validée (incluant `invoiceNumber`, `journalEntryId`, statut).
pub async fn validate_invoice_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    // Review P3 : utiliser directement le résultat transactionnel au lieu
    // d'un re-fetch post-commit (évite une fenêtre de race + DB roundtrip).
    let validated =
        invoices::validate_invoice(&state.pool, company.id, id, current_user.user_id).await?;
    Ok(Json(InvoiceResponse::from_parts(
        validated.invoice,
        validated.lines,
    )))
}

// ---------------------------------------------------------------------------
// Story 5.4 — Échéancier factures
// ---------------------------------------------------------------------------

const MAX_EXPORT_ROWS: i64 = 10_000;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaymentStatusParam {
    All,
    Paid,
    Unpaid,
    Overdue,
}

impl From<PaymentStatusParam> for PaymentStatusFilter {
    fn from(p: PaymentStatusParam) -> Self {
        match p {
            PaymentStatusParam::All => PaymentStatusFilter::All,
            PaymentStatusParam::Paid => PaymentStatusFilter::Paid,
            PaymentStatusParam::Unpaid => PaymentStatusFilter::Unpaid,
            PaymentStatusParam::Overdue => PaymentStatusFilter::Overdue,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDueDatesQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub contact_id: Option<i64>,
    #[serde(default)]
    pub date_from: Option<NaiveDate>,
    #[serde(default)]
    pub date_to: Option<NaiveDate>,
    #[serde(default)]
    pub due_before: Option<NaiveDate>,
    #[serde(default)]
    pub payment_status: Option<PaymentStatusParam>,
    #[serde(default)]
    pub sort_by: Option<InvoiceSortBy>,
    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

/// B22 (review pass 2 G2 B) : `is_overdue` est désormais porté par
/// `InvoiceListItemResponse` directement (cohérence avec la liste standard).
/// Le wrapper historique est supprimé pour éviter une collision serde
/// `#[serde(flatten)]` sur la même clé `isOverdue`.
pub type DueDateItemResponse = InvoiceListItemResponse;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DueDatesResponse {
    pub items: Vec<DueDateItemResponse>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
    pub summary: DueDatesSummary,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkPaidRequest {
    #[serde(default)]
    pub paid_at: Option<NaiveDateTime>,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnmarkPaidRequest {
    pub version: i32,
}

/// B12 (review pass 1 G2 B) : `version` sérialisée par le frontend ne doit
/// jamais être négative. Sans cette garde, un payload `version=-1` produit
/// un 409 OPTIMISTIC_LOCK_CONFLICT confus au lieu d'un 400 explicite.
fn validate_version(v: i32) -> Result<(), AppError> {
    if v < 0 {
        return Err(AppError::Validation(
            "version doit être un entier non négatif".into(),
        ));
    }
    Ok(())
}

/// Construit une `InvoiceListQuery` pour l'échéancier — force `status =
/// 'validated'` et applique les défauts de tri (`due_date ASC`).
fn build_due_dates_query(params: ListDueDatesQuery) -> Result<InvoiceListQuery, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    if !(1..=MAX_LIST_LIMIT).contains(&limit) {
        return Err(AppError::Validation(format!(
            "limit doit être compris entre 1 et {MAX_LIST_LIMIT}"
        )));
    }
    let offset = params.offset.unwrap_or(0);
    if offset < 0 {
        return Err(AppError::Validation(
            "offset doit être positif ou nul".into(),
        ));
    }
    // B16 (review pass 1 G2 B) : cap raisonnable (cf. list_invoices).
    const MAX_OFFSET: i64 = 1_000_000;
    if offset > MAX_OFFSET {
        return Err(AppError::Validation(format!(
            "offset doit être ≤ {MAX_OFFSET}"
        )));
    }

    let search = match params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(s) if s.chars().count() > MAX_SEARCH_LEN => {
            return Err(AppError::Validation(format!(
                "search doit faire au plus {MAX_SEARCH_LEN} caractères"
            )));
        }
        Some(s) => Some(s.to_string()),
        None => None,
    };

    if let (Some(df), Some(dt)) = (params.date_from, params.date_to) {
        if df > dt {
            return Err(AppError::Validation(
                "dateFrom doit être antérieur ou égal à dateTo".into(),
            ));
        }
    }
    // B11 (review pass 1 G2 B) : `dueBefore` doit être ≥ `dateFrom` (sinon
    // intersection vide silencieuse — l'utilisateur s'attend à un message
    // explicite).
    if let (Some(df), Some(db)) = (params.date_from, params.due_before) {
        if db < df {
            return Err(AppError::Validation(
                "dueBefore doit être ≥ dateFrom".into(),
            ));
        }
    }
    // B21 (review pass 2 G2 B) : pareil pour `dateTo` — une intersection
    // vide due_before < date_to mérite un 400 explicite (uniquement quand
    // les deux sont posés ; le cas indépendant reste autorisé).
    if let (Some(dt), Some(db)) = (params.date_to, params.due_before) {
        if db < dt {
            return Err(AppError::Validation("dueBefore doit être ≥ dateTo".into()));
        }
    }

    Ok(InvoiceListQuery {
        search,
        // Forcé côté backend — sécurité par défaut (Scope §5).
        status: Some("validated".into()),
        contact_id: params.contact_id,
        date_from: params.date_from,
        date_to: params.date_to,
        // B1 (review pass 1 G2 B) : défaut backend = `Unpaid` pour respecter
        // AC#3 (échéancier chargé par défaut → impayées). Tout override
        // explicite du caller (`?paymentStatus=all|paid|overdue`) reste
        // honoré — seul le cas « pas de valeur fournie » bascule sur Unpaid.
        payment_status: Some(
            params
                .payment_status
                .map(Into::into)
                .unwrap_or(PaymentStatusFilter::Unpaid),
        ),
        due_before: params.due_before,
        sort_by: params.sort_by.unwrap_or(InvoiceSortBy::DueDate),
        sort_direction: params.sort_direction.unwrap_or(SortDirection::Asc),
        limit,
        offset,
    })
}

/// `GET /api/v1/invoices/due-dates` — page échéancier.
pub async fn list_due_dates_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListDueDatesQuery>,
) -> Result<Json<DueDatesResponse>, AppError> {
    let query = build_due_dates_query(params)?;

    // B15 (review pass 1 G2 B) : strip explicite de `payment_status` avant
    // la summary — rend l'invariant AC#11 (« le summary ignore paymentStatus »)
    // visible côté handler au lieu de dépendre uniquement du contrat repo.
    let summary_query = InvoiceListQuery {
        payment_status: None,
        ..query.clone()
    };
    // List + summary en parallèle (read-only, pas de tx commune).
    let (list_res, summary_res) = tokio::join!(
        invoices::list_by_company_paginated(&state.pool, current_user.company_id, query.clone()),
        invoices::due_dates_summary(&state.pool, current_user.company_id, &summary_query),
    );
    let list = list_res?;
    let summary = summary_res?;

    // B22 (review pass 2 G2 B) : `is_overdue` est désormais calculé dans le
    // `From<InvoiceListItem>` — chaque item porte sa propre valeur. Le `today`
    // utilisé est `Utc::now()` à l'instant de la conversion, ce qui suffit
    // pour la durée d'une requête (race midnight rare et acceptable).
    let items = list
        .items
        .into_iter()
        .map(InvoiceListItemResponse::from)
        .collect();

    Ok(Json(DueDatesResponse {
        items,
        total: list.total,
        offset: list.offset,
        limit: list.limit,
        summary,
    }))
}

// N2 (review pass 3 B) : `validate_paid_at_bounds` supprimé.
// Domaine métier : `paid_at` = date d'exécution bancaire effective. Elle peut
// légitimement être dans le futur (ordre de virement programmé, décalage
// week-end/jour férié appliqué par la banque). La seule borne reste la borne
// basse `paid_at >= invoice.date - 1j`, validée dans le repository.

/// `POST /api/v1/invoices/:id/mark-paid` — marquage manuel.
pub async fn mark_invoice_paid_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<MarkPaidRequest>,
) -> Result<Json<InvoiceResponse>, AppError> {
    validate_version(req.version)?;
    let company = get_company_for(&current_user, &state.pool).await?;
    let paid_at = req
        .paid_at
        .unwrap_or_else(|| chrono::Utc::now().naive_utc());

    // M1 (review pass 1 G2) : mark_as_paid retourne `(Invoice, Vec<InvoiceLine>)`
    // depuis la même transaction, supprimant la race DELETE/UPDATE entre
    // l'UPDATE paid_at et un re-fetch post-commit.
    let (updated, lines) = invoices::mark_as_paid(
        &state.pool,
        current_user.user_id,
        id,
        company.id,
        req.version,
        Some(paid_at),
    )
    .await?;
    Ok(Json(InvoiceResponse::from_parts(updated, lines)))
}

/// `POST /api/v1/invoices/:id/unmark-paid` — annule un marquage payé.
pub async fn unmark_invoice_paid_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UnmarkPaidRequest>,
) -> Result<Json<InvoiceResponse>, AppError> {
    validate_version(req.version)?;
    let company = get_company_for(&current_user, &state.pool).await?;
    // M1 (review pass 1 G2) : idem mark_invoice_paid_handler — fetch atomique.
    let (updated, lines) = invoices::mark_as_paid(
        &state.pool,
        current_user.user_id,
        id,
        company.id,
        req.version,
        None,
    )
    .await?;
    Ok(Json(InvoiceResponse::from_parts(updated, lines)))
}

/// Clés FTL des en-têtes CSV (locale = `companies.accounting_language`).
const CSV_HEADER_KEYS: [&str; 7] = [
    "echeancier-csv-header-number",
    "echeancier-csv-header-date",
    "echeancier-csv-header-due-date",
    "echeancier-csv-header-contact",
    "echeancier-csv-header-total",
    "echeancier-csv-header-payment-status",
    "echeancier-csv-header-paid-at",
];

/// M3 (review pass 1 G2) : neutralise l'injection de formules Excel/Calc.
/// Si une cellule commence par `=`, `+`, `-`, `@` (voir OWASP CSV Injection),
/// on préfixe d'une apostrophe simple pour forcer l'interprétation texte.
///
/// P3 (review pass 2) : supprime aussi les CR/LF en défense en profondeur.
/// `csv::Writer` quote les champs contenant `\n`/`\r`, mais un champ remplacé
/// par un espace reste lisible sans casser l'alignement des lignes si un
/// parseur tiers naïf lit le fichier.
fn csv_sanitize(raw: String) -> String {
    // B9 (review pass 1 G2 B) : neutralise aussi TAB (Excel l'interprète
    // comme déclencheur dans certains contextes) et le whitespace de tête
    // (un attaquant peut bypasser le check via " =cmd").
    let raw: String = raw
        .chars()
        .map(|c| {
            if c == '\r' || c == '\n' || c == '\t' {
                ' '
            } else {
                c
            }
        })
        .collect();
    if let Some(first) = raw.trim_start().chars().next() {
        if matches!(first, '=' | '+' | '-' | '@') {
            let mut out = String::with_capacity(raw.len() + 1);
            out.push('\'');
            out.push_str(&raw);
            return out;
        }
    }
    raw
}

const CSV_HEADER_FALLBACKS: [&str; 7] = [
    "Numéro",
    "Date",
    "Date d'échéance",
    "Client",
    "Total",
    "Statut paiement",
    "Date paiement",
];

/// `GET /api/v1/invoices/due-dates/export.csv` — export CSV échéancier.
///
/// Format : UTF-8 + BOM, séparateur `;`, CRLF, montants suisses (1'234.56),
/// dates dd.mm.yyyy. Limite dure 10'000 lignes (sinon 400).
pub async fn export_due_dates_csv_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(mut params): Query<ListDueDatesQuery>,
) -> Result<axum::response::Response, AppError> {
    use axum::http::HeaderValue;
    use axum::http::header;
    use axum::response::IntoResponse;

    let company = get_company_for(&current_user, &state.pool).await?;

    // M4 (review pass 1 G2) : le `limit`/`offset` côté client ne s'appliquent
    // pas à l'export CSV (pas de pagination). On les force à des valeurs
    // validables pour que `build_due_dates_query` ne rejette pas l'appel
    // avec un 400 sur un paramètre sans effet. La limite réelle est
    // appliquée par `list_for_export(MAX_EXPORT_ROWS + 1)`.
    params.limit = None;
    params.offset = None;
    let query = build_due_dates_query(params)?;

    // +1 pour détecter le dépassement de la limite dure.
    // `truncated` (P3 review pass 3 A) capte le cas où le clamp repo 50_000
    // aurait silencieusement rabaissé l'intent — impossible tant que
    // `MAX_EXPORT_ROWS + 1 <= 50_000`, mais on le propage en sécurité.
    let (rows, truncated) = invoices::list_for_export(
        &state.pool,
        current_user.company_id,
        &query,
        MAX_EXPORT_ROWS + 1,
    )
    .await?;
    if truncated || rows.len() as i64 > MAX_EXPORT_ROWS {
        // H1 (review pass 1 G2) : code client dédié `RESULT_TOO_LARGE`
        // (spec §84 / AC#10), distinct de `VALIDATION_ERROR`.
        let locale = Locale::from(company.accounting_language.as_str());
        let key = "echeancier-export-error-too-large";
        let mut args = FluentArgs::new();
        args.set("limit", MAX_EXPORT_ROWS);
        let msg = state.i18n.format(&locale, key, Some(&args));
        let msg = if msg == key || msg.is_empty() {
            format!("Trop de résultats (> {MAX_EXPORT_ROWS}). Veuillez affiner vos filtres.")
        } else {
            msg
        };
        return Err(AppError::ResultTooLarge(msg));
    }

    let locale = Locale::from(company.accounting_language.as_str());
    let today = chrono::Utc::now().naive_utc().date();

    // Build CSV en mémoire (~2 Mo max → pas de streaming nécessaire).
    let mut buf: Vec<u8> = Vec::with_capacity(rows.len().saturating_mul(200) + 4);
    // BOM UTF-8 — garantit qu'Excel Windows interprète correctement les accents.
    buf.extend_from_slice(&[0xEF, 0xBB, 0xBF]);

    {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b';')
            .terminator(csv::Terminator::CRLF)
            .from_writer(&mut buf);

        // En-têtes (locale = accounting_language).
        let headers: Vec<String> = CSV_HEADER_KEYS
            .iter()
            .zip(CSV_HEADER_FALLBACKS.iter())
            .map(|(k, fb)| {
                let v = state.i18n.format(&locale, k, None);
                if v.is_empty() || v == *k {
                    fb.to_string()
                } else {
                    v
                }
            })
            .collect();
        wtr.write_record(&headers)
            .map_err(|e| AppError::Internal(format!("csv header: {e}")))?;

        for inv in rows {
            // B3 (review pass 1 G2 B) : utilise le helper centralisé.
            let payment_status_key = if inv.paid_at.is_some() {
                "payment-status-paid"
            } else if is_invoice_overdue(&inv.status, inv.paid_at, inv.due_date, today) {
                "payment-status-overdue"
            } else {
                "payment-status-unpaid"
            };
            let payment_status = state.i18n.format(&locale, payment_status_key, None);
            let payment_status =
                if payment_status.is_empty() || payment_status == payment_status_key {
                    payment_status_key.to_string()
                } else {
                    payment_status
                };

            wtr.write_record(&[
                csv_sanitize(inv.invoice_number.clone().unwrap_or_default()),
                format_date(&inv.date),
                inv.due_date.as_ref().map(format_date).unwrap_or_default(),
                csv_sanitize(inv.contact_name.clone()),
                format_money(&inv.total_amount),
                // B19 (review pass 2 G2 B) : défense en profondeur — la valeur
                // vient d'un fichier FTL contrôlé, mais une compromission
                // (clé locale altérée) ne doit pas ouvrir un vecteur d'injection.
                csv_sanitize(payment_status),
                inv.paid_at
                    .map(|d| format_date(&d.date()))
                    .unwrap_or_default(),
            ])
            .map_err(|e| AppError::Internal(format!("csv row: {e}")))?;
        }

        wtr.flush()
            .map_err(|e| AppError::Internal(format!("csv flush: {e}")))?;
    }

    let filename = format!("echeancier-{}.csv", today.format("%Y-%m-%d"));
    let disposition = format!("attachment; filename=\"{}\"", filename);

    let mut resp = (StatusCode::OK, buf).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn line(desc: &str, qty: Decimal, price: Decimal, vat: Decimal) -> CreateInvoiceLineRequest {
        CreateInvoiceLineRequest {
            description: desc.into(),
            quantity: qty,
            unit_price: price,
            vat_rate: vat,
        }
    }

    #[test]
    fn validate_lines_rejects_empty() {
        let err = validate_lines(vec![]).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_zero_quantity() {
        let err = validate_line(line("X", dec!(0), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_negative_quantity() {
        let err = validate_line(line("X", dec!(-1), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_empty_description() {
        let err = validate_line(line("   ", dec!(1), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_description_too_long() {
        let long = "a".repeat(1001);
        let err = validate_line(line(&long, dec!(1), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_bad_vat() {
        let err = validate_line(line("X", dec!(1), dec!(10), dec!(99.99)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_accepts_valid_vats() {
        for v in ["0.00", "2.60", "3.80", "8.10"] {
            let d: Decimal = v.parse().unwrap();
            assert!(validate_line(line("X", dec!(1), dec!(10), d), 0).is_ok());
        }
    }

    #[test]
    fn validate_line_rejects_scale_above_four() {
        let err = validate_line(line("X", dec!(1.12345), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        let err = validate_line(line("X", dec!(1), dec!(10.12345), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_over_caps() {
        let err = validate_line(line("X", dec!(1000001), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        let err =
            validate_line(line("X", dec!(1), dec!(1000000000.0001), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_normalizes_description_to_nfc() {
        // "é" NFD → doit devenir NFC.
        let nfd = "Caf\u{0065}\u{0301}";
        let v = validate_line(line(nfd, dec!(1), dec!(10), dec!(8.10)), 0).unwrap();
        assert_eq!(v.description, "Caf\u{00E9}");
    }

    #[test]
    fn validate_payment_terms_trims_and_collapses_empty() {
        assert_eq!(validate_payment_terms(None).unwrap(), None);
        assert_eq!(validate_payment_terms(Some("  ".into())).unwrap(), None);
        assert_eq!(
            validate_payment_terms(Some("  30 j  ".into())).unwrap(),
            Some("30 j".into())
        );
    }

    #[test]
    fn validate_payment_terms_rejects_too_long() {
        let long = "a".repeat(256);
        let err = validate_payment_terms(Some(long)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
