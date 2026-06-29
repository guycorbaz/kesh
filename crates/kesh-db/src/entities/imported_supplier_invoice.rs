//! Entité `ImportedSupplierInvoice` (Story 12.5b — import répertoire, #194).
//!
//! Facture fournisseur **importée depuis un dossier inbox** (PDF/image porteur
//! d'un Swiss QR Code), décodée côté serveur et mise en **staging** « à compléter ».
//! Découple l'ingestion (coordonnées QR parsées + fichier archivé) de la
//! comptabilisation (création de la `SupplierInvoice` réelle à la complétion,
//! 12-5c). Le lien justificatif est porté **côté import** (DC4) : colonnes
//! document + FK nullable `supplier_invoice_id` renseignée à la complétion.

use kesh_qrbill::{ScannedQrBill, ScannedReference};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Facture fournisseur importée persistée (staging).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ImportedSupplierInvoice {
    pub id: i64,
    pub company_id: i64,
    /// 'to_complete' (importée, en attente) | 'completed' (facture créée) |
    /// 'discarded' (écartée).
    pub status: String,
    /// Renseigné à la complétion (12-5c) : facture fournisseur réelle (12-2).
    pub supplier_invoice_id: Option<i64>,
    // Fichier archivé (KESH_DOCUMENTS_DIR, hors DB).
    pub file_hash: String,
    pub storage_path: String,
    pub original_filename: String,
    pub mime_type: String,
    pub byte_size: i64,
    // Coordonnées créancier parsées du QR (SPC).
    pub creditor_iban: String,
    pub is_qr_iban: bool,
    /// 'K' (Combined) ou 'S' (Structured).
    pub creditor_address_type: String,
    pub creditor_name: String,
    pub creditor_line1: Option<String>,
    pub creditor_line2: Option<String>,
    pub creditor_postal_code: Option<String>,
    pub creditor_town: Option<String>,
    pub creditor_country: String,
    /// 'QRR' | 'SCOR' | 'NON'.
    pub reference_type: String,
    pub reference_value: Option<String>,
    /// SPC autorise le montant vide (open amount) → `None`.
    pub amount: Option<Decimal>,
    pub currency: String,
    pub unstructured_message: Option<String>,
    pub billing_information: Option<String>,
    pub version: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

/// Métadonnées du fichier archivé sur disque (`KESH_DOCUMENTS_DIR`).
///
/// Produit par le helper de stockage (kesh-api), passé en valeurs simples pour
/// éviter une dépendance `kesh-db → kesh-api`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    /// Relatif à `KESH_DOCUMENTS_DIR`, = `{sha256hex}.{ext}`.
    pub storage_path: String,
    /// Nom d'origine (affichage seul ; jamais utilisé pour construire un chemin).
    pub original_filename: String,
    /// SHA-256 hex du contenu (idempotence + nommage).
    pub sha256: String,
    pub mime_type: String,
    pub byte_size: i64,
}

/// Données d'insertion d'une facture importée (staging `to_complete`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewImportedSupplierInvoice {
    pub company_id: i64,
    // Document.
    pub file_hash: String,
    pub storage_path: String,
    pub original_filename: String,
    pub mime_type: String,
    pub byte_size: i64,
    // Coordonnées QR.
    pub creditor_iban: String,
    pub is_qr_iban: bool,
    pub creditor_address_type: String,
    pub creditor_name: String,
    pub creditor_line1: Option<String>,
    pub creditor_line2: Option<String>,
    pub creditor_postal_code: Option<String>,
    pub creditor_town: Option<String>,
    pub creditor_country: String,
    pub reference_type: String,
    pub reference_value: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: String,
    pub unstructured_message: Option<String>,
    pub billing_information: Option<String>,
}

impl NewImportedSupplierInvoice {
    /// Projette un `ScannedQrBill` (12-5a) + les métadonnées du fichier archivé
    /// en données d'insertion staging. Aucun champ QR n'est perdu.
    pub fn from_scanned(company_id: i64, scanned: &ScannedQrBill, doc: DocumentMeta) -> Self {
        let (reference_type, reference_value) = match &scanned.reference {
            ScannedReference::Qrr(v) => ("QRR".to_string(), Some(v.clone())),
            ScannedReference::Scor(v) => ("SCOR".to_string(), Some(v.clone())),
            ScannedReference::None => ("NON".to_string(), None),
        };
        Self {
            company_id,
            file_hash: doc.sha256,
            storage_path: doc.storage_path,
            original_filename: doc.original_filename,
            mime_type: doc.mime_type,
            byte_size: doc.byte_size,
            creditor_iban: scanned.creditor_iban.clone(),
            is_qr_iban: scanned.is_qr_iban,
            creditor_address_type: scanned.creditor.address_type.to_string(),
            creditor_name: scanned.creditor.name.clone(),
            creditor_line1: non_empty(&scanned.creditor.street_or_line1),
            creditor_line2: non_empty(&scanned.creditor.building_or_line2),
            creditor_postal_code: scanned.creditor.postal_code.clone(),
            creditor_town: scanned.creditor.town.clone(),
            creditor_country: scanned.creditor.country.clone(),
            reference_type,
            reference_value,
            amount: scanned.amount,
            currency: scanned.currency.clone(),
            unstructured_message: scanned.unstructured_message.clone(),
            billing_information: scanned.billing_information.clone(),
        }
    }
}

/// `""` → `None`, sinon `Some(owned)`.
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}
