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
            // Normalisation sur {'K','S'} : le parseur (12-5a) traite tout type
            // ≠ 'S' comme 'K' (adresse combinée). On s'aligne pour respecter le
            // CHECK SQL `IN ('K','S')` — un QR tiers émettant un autre caractère
            // ne doit pas provoquer une CHECK violation DB opaque.
            creditor_address_type: if scanned.creditor.address_type == 'S' {
                "S".to_string()
            } else {
                "K".to_string()
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_qrbill::ScannedAddress;
    use rust_decimal_macros::dec;

    fn doc() -> DocumentMeta {
        DocumentMeta {
            storage_path: "abc123.pdf".into(),
            original_filename: "facture.pdf".into(),
            sha256: "abc123".into(),
            mime_type: "application/pdf".into(),
            byte_size: 4096,
        }
    }

    /// Mapping round-trip : tout champ QR (adresse combinée + QRR + montant) est
    /// projeté, et les métadonnées document proviennent du `DocumentMeta`.
    #[test]
    fn from_scanned_projects_all_qr_fields() {
        let scanned = ScannedQrBill {
            creditor_iban: "CH4431999123000889012".into(),
            is_qr_iban: true,
            creditor: ScannedAddress {
                address_type: 'K',
                name: "Robert Schneider SA".into(),
                street_or_line1: "Rue du Lac 1268".into(),
                building_or_line2: "2501 Biel".into(),
                postal_code: None,
                town: None,
                country: "CH".into(),
            },
            amount: Some(dec!(199.95)),
            currency: "CHF".into(),
            reference: ScannedReference::Qrr("210000000003139471430009017".into()),
            unstructured_message: Some("Facture test".into()),
            billing_information: None,
        };

        let new = NewImportedSupplierInvoice::from_scanned(42, &scanned, doc());

        assert_eq!(new.company_id, 42);
        assert_eq!(new.creditor_iban, "CH4431999123000889012");
        assert!(new.is_qr_iban);
        assert_eq!(new.creditor_address_type, "K");
        assert_eq!(new.creditor_name, "Robert Schneider SA");
        assert_eq!(new.creditor_line1.as_deref(), Some("Rue du Lac 1268"));
        assert_eq!(new.creditor_line2.as_deref(), Some("2501 Biel"));
        assert_eq!(new.creditor_country, "CH");
        assert_eq!(new.reference_type, "QRR");
        assert_eq!(
            new.reference_value.as_deref(),
            Some("210000000003139471430009017")
        );
        assert_eq!(new.amount, Some(dec!(199.95)));
        assert_eq!(new.currency, "CHF");
        assert_eq!(new.unstructured_message.as_deref(), Some("Facture test"));
        assert_eq!(new.billing_information, None);
        // Métadonnées document.
        assert_eq!(new.file_hash, "abc123");
        assert_eq!(new.storage_path, "abc123.pdf");
        assert_eq!(new.original_filename, "facture.pdf");
        assert_eq!(new.byte_size, 4096);
    }

    /// Adresse structurée (`S`) avec champs vides → projetés en `None` ;
    /// référence `NON` → `reference_type = "NON"`, valeur `None`.
    #[test]
    fn from_scanned_handles_structured_address_and_no_reference() {
        let scanned = ScannedQrBill {
            creditor_iban: "CH5604835012345678009".into(),
            is_qr_iban: false,
            creditor: ScannedAddress {
                address_type: 'S',
                name: "Fournisseur SA".into(),
                street_or_line1: String::new(),
                building_or_line2: String::new(),
                postal_code: Some("1003".into()),
                town: Some("Lausanne".into()),
                country: "CH".into(),
            },
            amount: None,
            currency: "EUR".into(),
            reference: ScannedReference::None,
            unstructured_message: None,
            billing_information: None,
        };

        let new = NewImportedSupplierInvoice::from_scanned(7, &scanned, doc());

        assert_eq!(new.creditor_address_type, "S");
        // Champs vides → None (pas de chaîne vide en base).
        assert_eq!(new.creditor_line1, None);
        assert_eq!(new.creditor_line2, None);
        assert_eq!(new.creditor_postal_code.as_deref(), Some("1003"));
        assert_eq!(new.creditor_town.as_deref(), Some("Lausanne"));
        assert_eq!(new.reference_type, "NON");
        assert_eq!(new.reference_value, None);
        assert_eq!(new.amount, None);
        assert!(!new.is_qr_iban);
    }

    /// Un `address_type` hors {'K','S'} (QR tiers non conforme) est normalisé en
    /// 'K' — pas de CHECK violation DB opaque en aval (code-review 12-5b P3).
    #[test]
    fn from_scanned_normalizes_unexpected_address_type_to_k() {
        let scanned = ScannedQrBill {
            creditor_iban: "CH4431999123000889012".into(),
            is_qr_iban: true,
            creditor: ScannedAddress {
                address_type: 'C', // ni 'K' ni 'S'
                name: "Fournisseur".into(),
                street_or_line1: "Ligne 1".into(),
                building_or_line2: "Ligne 2".into(),
                postal_code: None,
                town: None,
                country: "CH".into(),
            },
            amount: Some(dec!(10.00)),
            currency: "CHF".into(),
            reference: ScannedReference::None,
            unstructured_message: None,
            billing_information: None,
        };
        let new = NewImportedSupplierInvoice::from_scanned(1, &scanned, doc());
        assert_eq!(new.creditor_address_type, "K");
    }
}
