//! Story 31-1a (#503) — montage commun aux tests de la sauvegarde complète :
//! une installation qui porte des **pièces** dans son dossier des documents.
//!
//! Inclus par `#[path]` dans `admin_full_export_e2e.rs` et
//! `admin_full_import_e2e.rs` (aucun crate de test séparé). Toutes les données
//! sont **synthétiques**.
//!
//! - [`seed_company_with_invoice`] : la société de `seed_accounting_company`,
//!   un compte bancaire, un client fictif et une facture **validée** ;
//! - [`seed_document`] : un document fournisseur sans QR, écrit par la
//!   primitive de production `store_document` puis inséré par le dépôt ;
//! - [`freeze_pdf`] : fige le PDF d'une facture par son premier
//!   téléchargement, le chemin réel de la production.

#![allow(dead_code)]

use std::path::Path;

use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::imported_supplier_invoice::NewImportedSupplierInvoice;
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::{bank_accounts, contacts, imported_supplier_invoices, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Société comptable complète + facture validée (non figée).
pub async fn seed_company_with_invoice(pool: &MySqlPool) -> (SeededCompany, i64) {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    bank_accounts::upsert_primary(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque fictive".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: Some("CH4431999123000889012".into()),
            is_primary: true,
        },
    )
    .await
    .expect("compte bancaire");
    let contact = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Client Fictif".into(),
            first_name: Some("Client".into()),
            last_name: Some("Fictif".into()),
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Rue de l'Exemple".into()),
            address_building: Some("1".into()),
            address_postal_code: Some("1000".into()),
            address_city: Some("Lausanne".into()),
            address_country: Some("CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Salutation::Neutre,
        },
    )
    .await
    .expect("client");
    let (invoice, _) = invoices::create(
        pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id: contact.id,
            date: chrono::NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
            payment_terms: Some("30 jours net".into()),
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Prestation fictive".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(8.10),
            }],
            project_id: None,
        },
    )
    .await
    .expect("facture");
    invoices::validate_invoice(pool, seeded.company_id, invoice.id, seeded.admin_user_id)
        .await
        .expect("validation");
    (seeded, invoice.id)
}

/// Un document fournisseur : `bytes` écrit par `store_document` sous
/// `documents` (extension `ext`), puis sa ligne. Rend `(id, storage_path)`.
pub async fn seed_document(
    pool: &MySqlPool,
    company_id: i64,
    documents: &Path,
    bytes: &[u8],
    ext: &str,
) -> (i64, String) {
    let doc = kesh_api::document_storage::store_document(
        documents,
        bytes,
        ext,
        "document-fictif",
        kesh_api::document_storage::mime_for_ext(ext),
    )
    .expect("store_document");
    let storage_path = doc.storage_path.clone();
    let row = imported_supplier_invoices::create(
        pool,
        &NewImportedSupplierInvoice::without_qr(company_id, doc),
    )
    .await
    .expect("document fournisseur");
    (row.id, storage_path)
}

/// Fige le PDF de la facture par son premier téléchargement ; rend
/// `(pdf_storage_path, octets servis)`.
pub async fn freeze_pdf(
    pool: &MySqlPool,
    client: &reqwest::Client,
    base_url: &str,
    jwt: &str,
    invoice_id: i64,
) -> (String, Vec<u8>) {
    let resp = client
        .get(format!("{base_url}/api/v1/invoices/{invoice_id}/pdf"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "gel du PDF");
    let bytes = resp.bytes().await.unwrap().to_vec();
    let path: Option<String> =
        sqlx::query_scalar("SELECT pdf_storage_path FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(pool)
            .await
            .unwrap();
    (path.expect("facture figée"), bytes)
}

/// Dossier temporaire neuf, propre au test (supprimé avec la valeur rendue).
pub fn dossier() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}
