//! Entité `ImportedSupplierInvoice` (Story 12.5b — import répertoire, #194).
//!
//! Facture fournisseur **importée depuis un dossier inbox** (PDF/image porteur
//! d'un Swiss QR Code), décodée côté serveur et mise en **staging** « à compléter ».
//! Découple l'ingestion (coordonnées QR parsées + fichier archivé) de la
//! comptabilisation (création de la `SupplierInvoice` réelle à la complétion,
//! 12-5c). Le lien justificatif est porté **côté import** (DC4) : colonnes
//! document + FK nullable `supplier_invoice_id` renseignée à la complétion.
//!
//! ## Documents sans QR (Story 30-2a, #685)
//!
//! Depuis la 0.14.0, **tout** document PDF ou image entre, avec ou sans
//! QR-code. Les sept colonnes que remplit le QR (`creditor_iban`, `is_qr_iban`,
//! `creditor_address_type`, `creditor_name`, `creditor_country`,
//! `reference_type`, `currency`) sont `NULL` ensemble ou renseignées ensemble
//! (contrainte `chk_imported_si_qr_all_or_nothing`) : « le document porte un
//! QR » se lit [`ImportedSupplierInvoice::has_qr`]. Un QR décodé mais qui ne
//! tient pas dans les colonnes ([`check_scanned_fits_columns`]) laisse entrer
//! le document **sans** ses données QR, à saisir à la main.

use kesh_qrbill::validation::{amount_max, amount_min};
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
    // Coordonnées créancier parsées du QR (SPC). Les sept champs sans
    // `Option` jusqu'à la 0.13.x sont `None` ensemble pour un document sans QR
    // (Story 30-2a) — jamais un sous-ensemble (contrainte « tout ou rien »).
    pub creditor_iban: Option<String>,
    pub is_qr_iban: Option<bool>,
    /// 'K' (Combined) ou 'S' (Structured).
    pub creditor_address_type: Option<String>,
    pub creditor_name: Option<String>,
    pub creditor_line1: Option<String>,
    pub creditor_line2: Option<String>,
    pub creditor_postal_code: Option<String>,
    pub creditor_town: Option<String>,
    pub creditor_country: Option<String>,
    /// 'QRR' | 'SCOR' | 'NON'.
    pub reference_type: Option<String>,
    pub reference_value: Option<String>,
    /// SPC autorise le montant vide (open amount) → `None` ; `None` aussi pour
    /// un document sans QR.
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub unstructured_message: Option<String>,
    pub billing_information: Option<String>,
    pub version: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ImportedSupplierInvoice {
    /// Le document porte-t-il des données QR ? (Story 30-2a) — lu sur
    /// `creditor_iban`, représentant des sept colonnes « tout ou rien ».
    pub fn has_qr(&self) -> bool {
        self.creditor_iban.is_some()
    }
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
    // Coordonnées QR (toutes `None` pour un document sans QR, Story 30-2a).
    pub creditor_iban: Option<String>,
    pub is_qr_iban: Option<bool>,
    pub creditor_address_type: Option<String>,
    pub creditor_name: Option<String>,
    pub creditor_line1: Option<String>,
    pub creditor_line2: Option<String>,
    pub creditor_postal_code: Option<String>,
    pub creditor_town: Option<String>,
    pub creditor_country: Option<String>,
    pub reference_type: Option<String>,
    pub reference_value: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub unstructured_message: Option<String>,
    pub billing_information: Option<String>,
}

/// Pourquoi un QR décodé ne peut pas être stocké tel quel (Story 30-2a, AC2).
///
/// Le document entre alors **sans** ses données QR : la saisie se fait à la
/// main. Le code rendu par [`QrColumnsMismatch::code`] va au rapport d'import
/// (`qrError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrColumnsMismatch {
    /// Une valeur du QR dépasse la largeur de sa colonne (en caractères).
    FieldTooLong {
        /// Colonne du staging dont la largeur est dépassée.
        column: &'static str,
    },
    /// Montant hors de `amount_min()..=amount_max()` (SIX, champ `Amt`) ou à
    /// plus de deux décimales significatives.
    AmountOutOfRange,
}

impl QrColumnsMismatch {
    /// Code canonique du rapport d'import (`qrError`).
    pub fn code(self) -> &'static str {
        match self {
            Self::FieldTooLong { .. } => "FIELD_TOO_LONG",
            Self::AmountOutOfRange => "QR_AMOUNT_OUT_OF_RANGE",
        }
    }
}

/// Contrôle **pur** d'un QR décodé contre les colonnes du staging (Story
/// 30-2a, AC2) — appelé **avant** l'archivage du document.
///
/// Le parseur 12-5a ne borne ni les longueurs, ni le pays, ni la devise, ni le
/// montant (`kesh-qrbill/src/parser.rs`). Sans ce contrôle, une valeur trop
/// longue sortait en `DataLengthOrRange` à l'`INSERT` (document refusé), et un
/// montant de plus de quinze chiffres entiers aussi (`DECIMAL(19,4)`, erreur
/// 1264) ; ici, le document entre sans ses données QR.
///
/// - **Longueurs en caractères** (`chars().count()`) : les colonnes `VARCHAR`
///   et `CHAR` en `utf8mb4` comptent des caractères, non des octets — un nom
///   accentué de 70 caractères tient dans `VARCHAR(70)`.
/// - **Montant présent** dans `amount_min()..=amount_max()` et à au plus deux
///   décimales **significatives** (`20.000` passe, `20.005` non). Seule la
///   plage protège la colonne ; les deux décimales sont la règle SIX du champ
///   `Amt`. Un montant vide (« open amount ») n'est pas contrôlé.
/// - **Pas de contrôle de domaine** : le type d'adresse est normalisé par
///   [`NewImportedSupplierInvoice::from_scanned`], le type de référence borné
///   par le parseur.
pub fn check_scanned_fits_columns(scanned: &ScannedQrBill) -> Result<(), QrColumnsMismatch> {
    let reference_value = match &scanned.reference {
        ScannedReference::Qrr(v) | ScannedReference::Scor(v) => Some(v.as_str()),
        ScannedReference::None => None,
    };
    let bounded: [(&'static str, Option<&str>, usize); 11] = [
        ("creditor_iban", Some(scanned.creditor_iban.as_str()), 34),
        ("creditor_name", Some(scanned.creditor.name.as_str()), 70),
        (
            "creditor_line1",
            Some(scanned.creditor.street_or_line1.as_str()),
            70,
        ),
        (
            "creditor_line2",
            Some(scanned.creditor.building_or_line2.as_str()),
            70,
        ),
        (
            "creditor_postal_code",
            scanned.creditor.postal_code.as_deref(),
            16,
        ),
        ("creditor_town", scanned.creditor.town.as_deref(), 35),
        (
            "creditor_country",
            Some(scanned.creditor.country.as_str()),
            2,
        ),
        ("reference_value", reference_value, 40),
        ("currency", Some(scanned.currency.as_str()), 3),
        (
            "unstructured_message",
            scanned.unstructured_message.as_deref(),
            140,
        ),
        (
            "billing_information",
            scanned.billing_information.as_deref(),
            140,
        ),
    ];
    for (column, value, max_chars) in bounded {
        if value.is_some_and(|v| v.chars().count() > max_chars) {
            return Err(QrColumnsMismatch::FieldTooLong { column });
        }
    }
    if let Some(amount) = scanned.amount
        && (amount < amount_min() || amount > amount_max() || amount.normalize().scale() > 2)
    {
        return Err(QrColumnsMismatch::AmountOutOfRange);
    }
    Ok(())
}

impl NewImportedSupplierInvoice {
    /// Document **sans** données QR (Story 30-2a, AC1) : aucun QR trouvé, ou un
    /// QR inexploitable. Les sept colonnes « tout ou rien » et les champs
    /// facultatifs du QR sont `None` ; seules les métadonnées du fichier
    /// archivé sont renseignées.
    pub fn without_qr(company_id: i64, doc: DocumentMeta) -> Self {
        Self {
            company_id,
            file_hash: doc.sha256,
            storage_path: doc.storage_path,
            original_filename: doc.original_filename,
            mime_type: doc.mime_type,
            byte_size: doc.byte_size,
            creditor_iban: None,
            is_qr_iban: None,
            creditor_address_type: None,
            creditor_name: None,
            creditor_line1: None,
            creditor_line2: None,
            creditor_postal_code: None,
            creditor_town: None,
            creditor_country: None,
            reference_type: None,
            reference_value: None,
            amount: None,
            currency: None,
            unstructured_message: None,
            billing_information: None,
        }
    }

    /// Projette un `ScannedQrBill` (12-5a) + les métadonnées du fichier archivé
    /// en données d'insertion staging. Aucun champ QR n'est perdu. Les sept
    /// colonnes « tout ou rien » sont toutes `Some` (Story 30-2a) ; l'appelant
    /// a d'abord vérifié [`check_scanned_fits_columns`].
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
            creditor_iban: Some(scanned.creditor_iban.clone()),
            is_qr_iban: Some(scanned.is_qr_iban),
            // Normalisation sur {'K','S'} : le parseur (12-5a) traite tout type
            // ≠ 'S' comme 'K' (adresse combinée). On s'aligne pour respecter le
            // CHECK SQL `IN ('K','S')` — un QR tiers émettant un autre caractère
            // ne doit pas provoquer une CHECK violation DB opaque.
            creditor_address_type: Some(if scanned.creditor.address_type == 'S' {
                "S".to_string()
            } else {
                "K".to_string()
            }),
            creditor_name: Some(scanned.creditor.name.clone()),
            creditor_line1: non_empty(&scanned.creditor.street_or_line1),
            creditor_line2: non_empty(&scanned.creditor.building_or_line2),
            creditor_postal_code: scanned.creditor.postal_code.clone(),
            creditor_town: scanned.creditor.town.clone(),
            creditor_country: Some(scanned.creditor.country.clone()),
            reference_type: Some(reference_type),
            reference_value,
            amount: scanned.amount,
            currency: Some(scanned.currency.clone()),
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
        assert_eq!(new.creditor_iban.as_deref(), Some("CH4431999123000889012"));
        assert_eq!(new.is_qr_iban, Some(true));
        assert_eq!(new.creditor_address_type.as_deref(), Some("K"));
        assert_eq!(new.creditor_name.as_deref(), Some("Robert Schneider SA"));
        assert_eq!(new.creditor_line1.as_deref(), Some("Rue du Lac 1268"));
        assert_eq!(new.creditor_line2.as_deref(), Some("2501 Biel"));
        assert_eq!(new.creditor_country.as_deref(), Some("CH"));
        assert_eq!(new.reference_type.as_deref(), Some("QRR"));
        assert_eq!(
            new.reference_value.as_deref(),
            Some("210000000003139471430009017")
        );
        assert_eq!(new.amount, Some(dec!(199.95)));
        assert_eq!(new.currency.as_deref(), Some("CHF"));
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

        assert_eq!(new.creditor_address_type.as_deref(), Some("S"));
        // Champs vides → None (pas de chaîne vide en base).
        assert_eq!(new.creditor_line1, None);
        assert_eq!(new.creditor_line2, None);
        assert_eq!(new.creditor_postal_code.as_deref(), Some("1003"));
        assert_eq!(new.creditor_town.as_deref(), Some("Lausanne"));
        assert_eq!(new.reference_type.as_deref(), Some("NON"));
        assert_eq!(new.reference_value, None);
        assert_eq!(new.amount, None);
        assert_eq!(new.is_qr_iban, Some(false));
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
        assert_eq!(new.creditor_address_type.as_deref(), Some("K"));
    }

    /// Story 30-2a (AC1) — un document sans QR ne porte que ses métadonnées.
    #[test]
    fn without_qr_leaves_every_qr_field_empty() {
        let new = NewImportedSupplierInvoice::without_qr(9, doc());
        assert_eq!(new.company_id, 9);
        assert_eq!(new.file_hash, "abc123");
        assert_eq!(new.storage_path, "abc123.pdf");
        assert_eq!(new.original_filename, "facture.pdf");
        assert_eq!(new.mime_type, "application/pdf");
        assert_eq!(new.byte_size, 4096);
        assert!(new.creditor_iban.is_none());
        assert!(new.is_qr_iban.is_none());
        assert!(new.creditor_address_type.is_none());
        assert!(new.creditor_name.is_none());
        assert!(new.creditor_line1.is_none());
        assert!(new.creditor_line2.is_none());
        assert!(new.creditor_postal_code.is_none());
        assert!(new.creditor_town.is_none());
        assert!(new.creditor_country.is_none());
        assert!(new.reference_type.is_none());
        assert!(new.reference_value.is_none());
        assert!(new.amount.is_none());
        assert!(new.currency.is_none());
        assert!(new.unstructured_message.is_none());
        assert!(new.billing_information.is_none());
    }

    /// QR conforme servant de base aux cas du contrôle pur.
    fn valid_scanned() -> ScannedQrBill {
        ScannedQrBill {
            creditor_iban: "CH9300762011623852957".into(),
            is_qr_iban: false,
            creditor: ScannedAddress {
                address_type: 'K',
                name: "Fournisseur SA".into(),
                street_or_line1: "Rue 1".into(),
                building_or_line2: "1000 Lausanne".into(),
                postal_code: None,
                town: None,
                country: "CH".into(),
            },
            amount: Some(dec!(20.00)),
            currency: "CHF".into(),
            reference: ScannedReference::Scor("RF18539007547034".into()),
            unstructured_message: None,
            billing_information: None,
        }
    }

    /// Story 30-2a (AC2) — chaque colonne bornée, à sa limite (accepté) et une
    /// unité au-delà (`FIELD_TOO_LONG` nommant la colonne), en caractères.
    #[test]
    fn check_scanned_fits_columns_bounds_every_column_in_chars() {
        assert_eq!(check_scanned_fits_columns(&valid_scanned()), Ok(()));
        type Setter = fn(&mut ScannedQrBill, String);
        let cases: [(&str, usize, Setter); 11] = [
            ("creditor_iban", 34, |s, v| s.creditor_iban = v),
            ("creditor_name", 70, |s, v| s.creditor.name = v),
            ("creditor_line1", 70, |s, v| s.creditor.street_or_line1 = v),
            ("creditor_line2", 70, |s, v| {
                s.creditor.building_or_line2 = v
            }),
            ("creditor_postal_code", 16, |s, v| {
                s.creditor.postal_code = Some(v)
            }),
            ("creditor_town", 35, |s, v| s.creditor.town = Some(v)),
            ("creditor_country", 2, |s, v| s.creditor.country = v),
            ("reference_value", 40, |s, v| {
                s.reference = ScannedReference::Scor(v)
            }),
            ("currency", 3, |s, v| s.currency = v),
            ("unstructured_message", 140, |s, v| {
                s.unstructured_message = Some(v)
            }),
            ("billing_information", 140, |s, v| {
                s.billing_information = Some(v)
            }),
        ];
        for (column, max, set) in cases {
            let mut at_limit = valid_scanned();
            set(&mut at_limit, "é".repeat(max));
            assert_eq!(
                check_scanned_fits_columns(&at_limit),
                Ok(()),
                "{column} : {max} caractères accentués tiennent"
            );
            let mut over = valid_scanned();
            set(&mut over, "é".repeat(max + 1));
            assert_eq!(
                check_scanned_fits_columns(&over),
                Err(QrColumnsMismatch::FieldTooLong { column }),
                "{column} : {} caractères dépassent",
                max + 1
            );
        }
    }

    /// Story 30-2a (AC2) — plage et décimales du montant ; montant vide admis.
    #[test]
    fn check_scanned_fits_columns_bounds_the_amount() {
        let with = |amount: Option<Decimal>| {
            let mut s = valid_scanned();
            s.amount = amount;
            check_scanned_fits_columns(&s)
        };
        assert_eq!(with(None), Ok(()), "montant ouvert");
        assert_eq!(with(Some(dec!(0.01))), Ok(()));
        assert_eq!(with(Some(dec!(999999999.99))), Ok(()));
        assert_eq!(with(Some(dec!(20.000))), Ok(()), "zéros non significatifs");
        for out in [
            dec!(0.00),
            dec!(-5.00),
            dec!(1000000000.00),
            dec!(1234567890123456),
            dec!(20.005),
        ] {
            assert_eq!(
                with(Some(out)),
                Err(QrColumnsMismatch::AmountOutOfRange),
                "{out}"
            );
        }
        assert_eq!(
            QrColumnsMismatch::AmountOutOfRange.code(),
            "QR_AMOUNT_OUT_OF_RANGE"
        );
        assert_eq!(
            QrColumnsMismatch::FieldTooLong { column: "currency" }.code(),
            "FIELD_TOO_LONG"
        );
    }
}
