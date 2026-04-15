//! Story 5.3 — endpoint `GET /api/v1/invoices/:id/pdf`.
//!
//! Charge la facture validée (scopée à la company courante), les lignes, le
//! contact, le compte bancaire primary, puis délègue à `kesh-qrbill` pour
//! produire le PDF. La langue est résolue via `state.config.locale`
//! (instance-level, pattern Story 2.1) — pas de champ langue sur
//! `CurrentUser`.

use crate::middleware::auth::CurrentUser;
use axum::Extension;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use kesh_db::entities::{BankAccount, Company, Invoice, InvoiceLine, contact::Contact};
use kesh_db::errors::DbError;
use kesh_db::repositories::{bank_accounts, companies, contacts, invoices};
use kesh_i18n::Locale;
use kesh_qrbill::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, QrBillData, QrBillError,
    QrBillI18n, Reference,
    validation::{build_qrr, normalize_iban},
};
use std::collections::HashMap;

use crate::AppState;
use crate::errors::AppError;

/// Limite v0.1 : nombre de lignes pouvant tenir sur un PDF A4 mono-page.
///
/// Calcul géométrique (`kesh-qrbill::pdf`) :
/// - `ty` initial = `PAGE_H - 130 = 167` mm (après header facture)
/// - pas par ligne = `5` mm
/// - break défensif si `ty < SEP_Y + 15 = 120` mm
/// - la check a lieu **avant** le draw → draw N se fait à `ty = 167 - (N-1)*5`
/// - `167 - (N-1)*5 >= 120` ⇒ `N <= 10.4` ⇒ **9 lignes max tiennent**
///
/// Le rendu est **défensif** : toute ligne supplémentaire provoque une erreur
/// `QrBillError::PdfGeneration` plutôt qu'une troncature silencieuse
/// (cf. `pdf.rs::draw_invoice_section`).
pub const MAX_LINES_PER_PDF: usize = 9;

/// `GET /api/v1/invoices/:id/pdf` — téléchargement PDF d'une facture validée.
pub async fn get_invoice_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let company = get_company(&state).await?;
    tracing::info!(
        user_id = current_user.user_id,
        role = ?current_user.role,
        invoice_id = id,
        "PDF download requested"
    );

    // Chargement facture + lignes (scopé company).
    let (invoice, lines) = invoices::find_by_id_with_lines(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    if invoice.status != "validated" {
        return Err(AppError::InvoiceNotValidated);
    }

    if lines.len() > MAX_LINES_PER_PDF {
        return Err(AppError::InvoiceTooManyLinesForPdf(lines.len()));
    }

    // Contact (débiteur).
    let contact = contacts::find_by_id(&state.pool, invoice.contact_id)
        .await?
        .ok_or_else(|| {
            // M2 (review pass 1 G2) : messages localisés via clés FTL dédiées
            // (et non plus des chaînes françaises en dur).
            AppError::InvoiceNotPdfReady(crate::errors::t(
                "invoice-pdf-error-contact-missing",
                "Le contact lié à la facture est introuvable.",
            ))
        })?;

    // Primary bank account.
    let primary_bank = bank_accounts::find_primary(&state.pool, company.id)
        .await?
        .ok_or_else(|| {
            AppError::InvoiceNotPdfReady(crate::errors::t(
                "invoice-pdf-error-no-primary-bank",
                "Aucun compte bancaire principal n'est configuré pour cette company.",
            ))
        })?;

    // Construction des structures kesh-qrbill.
    // Pays ISO-3166-1 alpha-2 depuis companies.country / contacts.country
    // (ajoutés en v0.1 via migration 20260418000001, DEFAULT 'CH').
    let creditor_country = fetch_country(&state.pool, "companies", company.id).await?;
    let debtor_country = fetch_country(&state.pool, "contacts", contact.id).await?;

    let (qr_data, pdf_data) = build_qrbill_inputs(
        &invoice,
        &lines,
        &contact,
        &company,
        &primary_bank,
        &creditor_country,
        &debtor_country,
    )?;
    let i18n = build_i18n(&state.i18n, state.config.locale);

    // Génération.
    let pdf_bytes = kesh_qrbill::generate_qr_bill_pdf(&qr_data, &pdf_data, &i18n)
        .map_err(map_qrbill_error)?;

    // Content-Disposition : filename sanitizé.
    let filename = sanitize_filename(invoice.invoice_number.as_deref().unwrap_or("facture"));
    let disposition = format!("inline; filename=\"facture-{}.pdf\"", filename);

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

/// Convertit les entités DB en `QrBillData` + `InvoicePdfData`.
fn build_qrbill_inputs(
    invoice: &Invoice,
    lines: &[InvoiceLine],
    contact: &Contact,
    company: &Company,
    primary_bank: &BankAccount,
    creditor_country: &str,
    debtor_country: &str,
) -> Result<(QrBillData, InvoicePdfData), AppError> {
    // Adresse créancier (depuis `companies.address` TEXT libre).
    let creditor_addr = split_address(&company.address).map_err(|_| {
        AppError::InvoiceNotPdfReady(crate::errors::t(
            "invoice-pdf-error-company-address-empty",
            "Adresse entreprise vide.",
        ))
    })?;
    let creditor = Address {
        address_type: AddressType::Combined,
        name: company.name.clone(),
        line1: creditor_addr.0,
        line2: creditor_addr.1,
        country: creditor_country.to_string(),
    };

    // Adresse débiteur.
    let debtor_raw = contact.address.as_deref().unwrap_or("").trim();
    if debtor_raw.is_empty() {
        return Err(AppError::InvoiceNotPdfReady(crate::errors::t(
            "invoice-pdf-error-client-address-required",
            "Adresse du client obligatoire pour la génération PDF.",
        )));
    }
    let debtor_addr = split_address(debtor_raw).map_err(|_| {
        AppError::InvoiceNotPdfReady(crate::errors::t(
            "invoice-pdf-error-client-address-empty",
            "Adresse du client vide.",
        ))
    })?;
    let debtor = Address {
        address_type: AddressType::Combined,
        name: contact.name.clone(),
        line1: debtor_addr.0,
        line2: debtor_addr.1,
        country: debtor_country.to_string(),
    };

    // IBAN / QR-IBAN + référence.
    let (iban, reference) = match primary_bank.qr_iban.as_deref() {
        Some(qr) if !qr.trim().is_empty() => {
            let qrr = build_qrr(company.id as u64, invoice.id as u64).map_err(|e| {
                AppError::InvoiceNotPdfReady(format!("Impossible de générer la référence QRR: {e}"))
            })?;
            (normalize_iban(qr), Reference::Qrr(qrr))
        }
        _ => (normalize_iban(&primary_bank.iban), Reference::None),
    };

    let qr_data = QrBillData {
        creditor_iban: iban,
        creditor: creditor.clone(),
        ultimate_debtor: Some(debtor.clone()),
        amount: Some(invoice.total_amount),
        currency: Currency::Chf,
        reference,
        unstructured_message: invoice.invoice_number.as_ref().map(|n| {
            let msg = format!("Facture {n}");
            // SIX 2.2: unstructured_message max 140 chars (USTRD_MAX).
            msg.chars().take(140).collect::<String>()
        }),
        billing_information: None,
    };

    let invoice_lines_pdf: Vec<InvoiceLinePdf> = lines
        .iter()
        .map(|l| InvoiceLinePdf {
            description: l.description.clone(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
        })
        .collect();

    let pdf_data = InvoicePdfData {
        invoice_number: invoice
            .invoice_number
            .clone()
            .unwrap_or_else(|| format!("#{}", invoice.id)),
        invoice_date: invoice.date,
        due_date: invoice.due_date,
        payment_terms: invoice.payment_terms.clone(),
        creditor_name: company.name.clone(),
        creditor_address_lines: split_lines(&company.address),
        creditor_ide: company.ide_number.clone(),
        debtor_name: contact.name.clone(),
        debtor_address_lines: split_lines(debtor_raw),
        lines: invoice_lines_pdf,
        total: invoice.total_amount,
        currency: Currency::Chf,
    };

    Ok((qr_data, pdf_data))
}

/// Splits a free-form address (multi-line TEXT) into two non-empty lines.
/// Returns an error if no non-empty line is found. Si l'adresse contient 3+
/// lignes non vides, les lignes 2..N sont fusionnées dans `line2` (séparées
/// par ", ") pour préserver l'information (NPA / ville / pays). La longueur
/// résultante est validée par `kesh-qrbill::validation` (ADDR_LINE_MAX = 70).
fn split_address(raw: &str) -> Result<(String, String), ()> {
    let lines: Vec<&str> = raw
        .split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    match lines.len() {
        0 => Err(()),
        1 => Ok((lines[0].into(), String::new())),
        _ => Ok((lines[0].into(), lines[1..].join(", "))),
    }
}

/// Returns every non-empty line of a multi-line address (for display in the
/// invoice top section — unlike `split_address`, preserves lines beyond 2).
fn split_lines(raw: &str) -> Vec<String> {
    raw.split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Build a `QrBillI18n` by querying the shared Fluent bundle for every key.
fn build_i18n(bundle: &kesh_i18n::I18nBundle, locale: Locale) -> QrBillI18n {
    let mut entries: HashMap<&'static str, String> = HashMap::new();
    for key in kesh_qrbill::types::I18N_KEYS {
        let value = bundle.format(&locale, key, None);
        entries.insert(key, value);
    }
    QrBillI18n::new(entries)
}

/// Maps `QrBillError` to `AppError`. Business errors (invalid IBAN, field too
/// long, amount out of range) map to `InvoiceNotPdfReady` (400); PDF-rendering
/// errors map to `PdfGenerationFailed` (500, detail logged only).
fn map_qrbill_error(err: QrBillError) -> AppError {
    match err {
        QrBillError::InvalidIban(msg)
        | QrBillError::InvalidQrIban(msg)
        | QrBillError::InvalidQrr(msg) => AppError::InvoiceNotPdfReady(msg),
        QrBillError::FieldTooLong { field, max, got } => {
            AppError::InvoiceNotPdfReady(format!("Champ {field} trop long (max {max}, got {got})"))
        }
        QrBillError::FieldEmpty(field) => {
            AppError::InvoiceNotPdfReady(format!("Champ {field} vide (requis)"))
        }
        QrBillError::InvalidAmount(msg) | QrBillError::InvalidCurrency(msg) => {
            AppError::InvoiceNotPdfReady(msg)
        }
        QrBillError::InvalidCountry(c) => {
            AppError::InvoiceNotPdfReady(format!("Pays invalide: {c}"))
        }
        QrBillError::InvalidCharset { field, codepoint } => AppError::InvoiceNotPdfReady(format!(
            "Champ {field} contient un caractère non autorisé par SIX 2.2 (U+{codepoint:04X})"
        )),
        QrBillError::PdfGeneration(msg) => AppError::PdfGenerationFailed(msg),
    }
}

fn sanitize_filename(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Lit la colonne `country` (CHAR(2)) de `companies` ou `contacts`.
/// `table` doit être un littéral validé en call-site pour éviter toute
/// injection SQL — seuls "companies" et "contacts" sont acceptés.
async fn fetch_country(
    pool: &sqlx::MySqlPool,
    table: &'static str,
    id: i64,
) -> Result<String, AppError> {
    let sql = match table {
        "companies" => "SELECT country FROM companies WHERE id = ?",
        "contacts" => "SELECT country FROM contacts WHERE id = ?",
        _ => {
            return Err(AppError::Internal(format!(
                "fetch_country: table `{table}` non autorisée"
            )));
        }
    };
    let row: Option<(String,)> = sqlx::query_as(sql)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("fetch_country({table}): {e}")))?;
    row.map(|(c,)| c)
        .ok_or_else(|| AppError::Database(DbError::NotFound))
}

async fn get_company(state: &AppState) -> Result<Company, AppError> {
    let list = companies::list(&state.pool, 1, 0).await?;
    list.into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Aucune company en base".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_address_empty_rejected() {
        assert!(split_address("").is_err());
        assert!(split_address("   \n   \n").is_err());
    }

    #[test]
    fn split_address_single_line() {
        assert_eq!(
            split_address("Rue du Lac 1268, 2501 Biel").unwrap(),
            ("Rue du Lac 1268, 2501 Biel".into(), String::new())
        );
    }

    #[test]
    fn split_address_two_lines() {
        assert_eq!(
            split_address("Rue du Lac 1268\n2501 Biel").unwrap(),
            ("Rue du Lac 1268".into(), "2501 Biel".into())
        );
    }

    #[test]
    fn split_address_three_plus_lines_merges() {
        assert_eq!(
            split_address("Rue du Lac 1268\nCase postale 45\n2501 Biel").unwrap(),
            (
                "Rue du Lac 1268".into(),
                "Case postale 45, 2501 Biel".into()
            )
        );
    }

    #[test]
    fn split_address_trims_and_skips_blank_lines() {
        let raw = "\n  Rue du Lac 1268  \n\n  2501 Biel  \n";
        let (l1, l2) = split_address(raw).unwrap();
        assert_eq!(l1, "Rue du Lac 1268");
        assert_eq!(l2, "2501 Biel");
    }

    #[test]
    fn sanitize_filename_replaces_non_alphanumeric() {
        assert_eq!(sanitize_filename("F-2026-0042"), "F-2026-0042");
        assert_eq!(sanitize_filename("../../etc/passwd"), ".._.._etc_passwd");
        assert_eq!(sanitize_filename("F 2026 #42"), "F_2026__42");
    }
}
