//! Story 20.3a — service de génération du PDF QR-facture, factorisé depuis
//! le handler HTTP `get_invoice_pdf` (Story 5.3) pour être réutilisable par
//! l'envoi de facture par e-mail (Story 20-3b, PDF en pièce jointe).
//!
//! Charge la facture validée (scopée à la company), les lignes, le contact,
//! le compte bancaire primary, puis délègue à `kesh-qrbill` pour produire le
//! PDF. La `locale` est un paramètre : le handler HTTP passe
//! `state.config.locale` (iso-comportement Story 5.3), l'envoi e-mail
//! passera la langue du contact.
//!
//! Héberge aussi les helpers PDF partagés avec `credit_notes.rs`
//! (`MAX_LINES_PER_PDF`, `build_i18n`, `map_qrbill_error`,
//! `sanitize_filename`, `split_lines`) — couplage route→service plutôt que
//! route→route.

use kesh_db::entities::{BankAccount, Company, Invoice, InvoiceLine, contact::Contact};
use kesh_db::errors::DbError;
use kesh_db::repositories::{bank_accounts, contacts, invoices};
use kesh_i18n::Locale;
use kesh_qrbill::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, QrBillData, QrBillError,
    QrBillI18n, Reference,
    validation::{build_qrr, normalize_iban},
};
use std::collections::HashMap;

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

/// PDF de facture généré, prêt à être servi (handler HTTP) ou attaché
/// (e-mail Story 20-3b).
pub struct RenderedInvoicePdf {
    /// Contenu binaire du PDF.
    pub bytes: Vec<u8>,
    /// Nom de fichier déjà sanitizé, sans extension ni chemin — ex. "F-2026-0042"
    /// (le caller ajoute le préfixe/extension de son Content-Disposition ou
    /// de son attachment). Dérivé de `invoice.invoice_number` via
    /// `sanitize_filename`.
    pub filename_base: String,
}

/// Génère le PDF QR-facture d'une facture **validée**, scopée à
/// `company` (anti-IDOR).
///
/// Contrat d'autorisation : `company` DOIT provenir d'une source déjà
/// autorisée pour l'utilisateur courant (typiquement `get_company_for`) —
/// le service ne re-vérifie pas ce droit. Le scoping de la facture est
/// garanti par `find_by_id_with_lines(pool, company.id, …)`.
///
/// Reproduit exactement la séquence historique de `get_invoice_pdf`
/// (Story 5.3) : chargement facture + lignes, validations (statut,
/// nombre de lignes, contact, compte bancaire primary), mapping
/// `kesh-qrbill`, génération. Toutes les erreurs remontent en `AppError`
/// avec les mêmes variantes que le endpoint historique.
pub async fn render(
    pool: &sqlx::MySqlPool,
    i18n: &kesh_i18n::I18nBundle,
    locale: Locale,
    company: &Company,
    invoice_id: i64,
) -> Result<RenderedInvoicePdf, AppError> {
    // Chargement facture + lignes (scopé company).
    let (invoice, lines) = invoices::find_by_id_with_lines(pool, company.id, invoice_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    if invoice.status != "validated" {
        return Err(AppError::InvoiceNotValidated);
    }

    if lines.len() > MAX_LINES_PER_PDF {
        return Err(AppError::InvoiceTooManyLinesForPdf(lines.len()));
    }

    // Contact (débiteur).
    let contact = contacts::find_by_id(pool, invoice.contact_id)
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
    let primary_bank = bank_accounts::find_primary(pool, company.id)
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
    let creditor_country = fetch_country(pool, "companies", company.id).await?;
    let debtor_country = fetch_country(pool, "contacts", contact.id).await?;

    let (qr_data, pdf_data) = build_qrbill_inputs(
        &invoice,
        &lines,
        &contact,
        company,
        &primary_bank,
        &creditor_country,
        &debtor_country,
    )?;
    let qr_i18n = build_i18n(i18n, locale);

    // Génération.
    let bytes = kesh_qrbill::generate_qr_bill_pdf(&qr_data, &pdf_data, &qr_i18n)
        .map_err(map_qrbill_error)?;

    let filename_base = sanitize_filename(invoice.invoice_number.as_deref().unwrap_or("facture"));

    Ok(RenderedInvoicePdf {
        bytes,
        filename_base,
    })
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
    // Adresse créancier — STRUCTURÉE type S (#213, conformité SIX 21.11.2025).
    let ca = company.structured_address();
    if ca.postal_code.trim().is_empty() || ca.city.trim().is_empty() {
        return Err(AppError::InvoiceNotPdfReady(crate::errors::t(
            "invoice-pdf-error-company-address-empty",
            "Adresse entreprise incomplète (NPA et localité requis).",
        )));
    }
    let creditor = Address {
        address_type: AddressType::Structured,
        name: company.name.clone(),
        line1: ca.street.clone(),
        line2: ca.building.clone(),
        postal_code: ca.postal_code.clone(),
        town: ca.city.clone(),
        country: if ca.country.trim().is_empty() {
            creditor_country.to_string()
        } else {
            ca.country.clone()
        },
    };

    // Adresse débiteur — STRUCTURÉE type S (#213). Requise et complète (NPA + localité).
    let da = contact.structured_address();
    let debtor = match da {
        Some(a) if !a.postal_code.trim().is_empty() && !a.city.trim().is_empty() => Address {
            address_type: AddressType::Structured,
            name: contact.name.clone(),
            line1: a.street.clone(),
            line2: a.building.clone(),
            postal_code: a.postal_code.clone(),
            town: a.city.clone(),
            country: if a.country.trim().is_empty() {
                debtor_country.to_string()
            } else {
                a.country.clone()
            },
        },
        _ => {
            return Err(AppError::InvoiceNotPdfReady(crate::errors::t(
                "invoice-pdf-error-client-address-required",
                "Adresse du client obligatoire et complète (NPA et localité) pour la génération PDF.",
            )));
        }
    };

    // IBAN / QR-IBAN + référence.
    let (iban, reference) = match primary_bank.qr_iban.as_deref() {
        Some(qr) if !qr.trim().is_empty() => {
            // B8 (review pass 1 G2 B) : message i18n cohérent avec les
            // autres erreurs PDF (le mapping côté errors.rs résoud la clé).
            let qrr = build_qrr(company.id as u64, invoice.id as u64).map_err(|e| {
                tracing::warn!("build_qrr failed: {e}");
                AppError::InvoiceNotPdfReady("qrbill-error-qrr-generation".into())
            })?;
            (normalize_iban(qr), Reference::Qrr(qrr))
        }
        _ => (normalize_iban(&primary_bank.iban), Reference::None),
    };

    // #246 (Story 21-2a) : le montant réclamé par le QR et affiché sous
    // « Total TTC » est le TTC canonique (helper kesh-core, même arithmétique
    // que le débit créance) — `total_amount` est le HT comptable et ne doit
    // JAMAIS être présenté comme montant dû.
    let total_ttc = kesh_core::accounting::vat::invoice_total_ttc(
        lines.iter().map(|l| (l.line_total, l.vat_rate)),
    );

    let qr_data = QrBillData {
        creditor_iban: iban,
        creditor: creditor.clone(),
        ultimate_debtor: Some(debtor.clone()),
        amount: Some(total_ttc),
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
        debtor_address_lines: split_lines(contact.address.as_deref().unwrap_or_default()),
        lines: invoice_lines_pdf,
        total: total_ttc,
        currency: Currency::Chf,
        origin_reference: None,
    };

    Ok((qr_data, pdf_data))
}

/// Returns every non-empty line of a multi-line address (for display in the
/// invoice top section, derived `address` column).
pub(crate) fn split_lines(raw: &str) -> Vec<String> {
    raw.split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Build a `QrBillI18n` by querying the shared Fluent bundle for every key.
pub(crate) fn build_i18n(bundle: &kesh_i18n::I18nBundle, locale: Locale) -> QrBillI18n {
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
pub(crate) fn map_qrbill_error(err: QrBillError) -> AppError {
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
        // Émis uniquement par le parseur SPC (Story 12-5, chemin import) — n'arrive
        // pas dans la génération PDF. Mappé comme une erreur de validation par défense.
        QrBillError::InvalidPayload(msg) => AppError::InvoiceNotPdfReady(msg),
    }
}

pub(crate) fn sanitize_filename(raw: &str) -> String {
    // B20 (review pass 2 G2 B) : cap à 64 caractères pour borner la taille
    // du header `Content-Disposition` (un `invoice_number` arbitrairement
    // long polluerait la réponse HTTP).
    raw.chars()
        .take(64)
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
pub(crate) async fn fetch_country(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_replaces_non_alphanumeric() {
        assert_eq!(sanitize_filename("F-2026-0042"), "F-2026-0042");
        assert_eq!(sanitize_filename("../../etc/passwd"), ".._.._etc_passwd");
        assert_eq!(sanitize_filename("F 2026 #42"), "F_2026__42");
    }
}
