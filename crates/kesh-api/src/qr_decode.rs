//! Décodage QR côté serveur des factures importées (Story 12-5b — #194).
//!
//! Pipeline (DC1) :
//! - **Image** (PNG/JPG) : `image::load_from_memory` → luma → `rxing` (QR→texte).
//! - **PDF** : `pdfium-render` rend chaque page → `image::DynamicImage` → `rxing`.
//!
//! On retient le **premier** payload commençant par `SPC\n0200` (multi-QR par
//! page, multi-page tentés dans l'ordre) puis on le parse via `parse_spc_payload`
//! (12-5a). Le mapping des erreurs vers les `error_code` HTTP
//! (`PDF_RENDER_ERROR`, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `NO_QR_CODE_FOUND`)
//! est consommé en 12-5c ; ce module expose une erreur typée.

use std::sync::Mutex;

use image::DynamicImage;
use kesh_qrbill::{QrBillError, ScannedQrBill, parse_spc_payload};

/// Sérialise tous les appels pdfium au niveau du **process** : pdfium est une
/// bibliothèque C non thread-safe. Défense en profondeur en plus du verrou de run
/// DC6 (12-5c) — garantit la sûreté même si un futur appelant oublie de sérialiser.
static PDFIUM_LOCK: Mutex<()> = Mutex::new(());

/// En-tête d'un payload Swiss QR (SPC v0200).
const SPC_HEADER: &str = "SPC\n0200";

/// Borne de dimension (largeur/hauteur en px) du décodage image directe — garde
/// anti-DoS (bombe de décompression) symétrique au cap de rendu PDF. Une image
/// déclarant des dimensions supérieures est rejetée **avant** allocation.
const MAX_IMAGE_DIMENSION: u32 = 10_000;
/// Borne d'allocation mémoire totale du décodage image (~256 MiB) — filet
/// complémentaire à [`MAX_IMAGE_DIMENSION`] contre les images sur-dimensionnées.
const MAX_IMAGE_ALLOC: u64 = 256 * 1024 * 1024;

/// Bornes du rendu PDF (anti-DoS, F4/L6). Le wiring `KESH_INBOX_MAX_PDF_PAGES`
/// env→`DecodeConfig` est fait par l'appelant (12-5c) ; ici, défauts en dur.
#[derive(Debug, Clone, Copy)]
pub struct DecodeConfig {
    /// Nombre max de pages PDF rendues (au-delà sans QR → `PdfRender`).
    pub max_pages: usize,
    /// Dimension max (largeur/hauteur en px) du rendu d'une page.
    pub max_dimension: u16,
}

impl Default for DecodeConfig {
    fn default() -> Self {
        Self {
            max_pages: 20,
            max_dimension: 2000,
        }
    }
}

/// Erreur de décodage. Le mapping HTTP est fait en 12-5c.
#[derive(Debug)]
pub enum DecodeError {
    /// `image` n'a pas pu décoder le fichier image (corrompu / format).
    ImageDecode(String),
    /// pdfium a échoué (binding absent, PDF illisible, rendu impossible).
    PdfRender(String),
    /// Un QR SPC a été décodé mais `parse_spc_payload` l'a rejeté (hors IBAN —
    /// l'IBAN invalide a son variant dédié `InvalidIban` pour le mapping HTTP).
    InvalidSpcPayload(QrBillError),
    /// L'IBAN créancier du QR est invalide (propagé du parseur 12-5a). Variant
    /// distinct pour que 12-5c mappe vers `INVALID_IBAN` (AC2).
    InvalidIban(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImageDecode(m) => write!(f, "décodage image impossible: {m}"),
            Self::PdfRender(m) => write!(f, "rendu PDF impossible: {m}"),
            Self::InvalidSpcPayload(e) => write!(f, "payload SPC invalide: {e}"),
            Self::InvalidIban(m) => write!(f, "IBAN créancier invalide: {m}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Décode **tous** les QR présents sur une image (ordre `rxing`).
pub fn decode_qr_from_image(img: &DynamicImage) -> Vec<String> {
    let luma = img.to_luma8();
    let (w, h) = luma.dimensions();
    match rxing::helpers::detect_multiple_in_luma(luma.into_raw(), w, h) {
        Ok(results) => results
            .into_iter()
            .map(|r| r.getText().to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Retient le premier payload SPC d'une liste et le parse. `None` si aucun QR
/// n'est un payload SPC.
fn first_spc(payloads: &[String]) -> Result<Option<ScannedQrBill>, DecodeError> {
    for p in payloads {
        if p.starts_with(SPC_HEADER) {
            return match parse_spc_payload(p) {
                Ok(bill) => Ok(Some(bill)),
                // IBAN / QR-IBAN invalide → variant dédié (AC2, mapping HTTP
                // `INVALID_IBAN` en 12-5c), distinct des autres rejets SPC.
                Err(QrBillError::InvalidIban(m)) | Err(QrBillError::InvalidQrIban(m)) => {
                    Err(DecodeError::InvalidIban(m))
                }
                Err(e) => Err(DecodeError::InvalidSpcPayload(e)),
            };
        }
    }
    Ok(None)
}

/// Décode un fichier **image** (PNG/JPG) en `ScannedQrBill`. `None` si aucun QR SPC.
pub fn decode_spc_from_image_bytes(bytes: &[u8]) -> Result<Option<ScannedQrBill>, DecodeError> {
    // Décodage borné (anti-bombe de décompression) : `image::load_from_memory`
    // n'applique AUCUNE limite ; on passe par `ImageReader` avec des `Limits`
    // explicites, symétriques au cap de rendu PDF. Une image déclarant des
    // dimensions/allocations excessives est rejetée avant d'épuiser la mémoire.
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| DecodeError::ImageDecode(e.to_string()))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC);
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|e| DecodeError::ImageDecode(e.to_string()))?;
    first_spc(&decode_qr_from_image(&img))
}

/// Décode un **PDF** en `ScannedQrBill` : rend chaque page (jusqu'au cap), tente
/// chaque QR. `None` si aucune page ne porte de QR SPC.
///
/// # Thread safety
/// pdfium (bibliothèque C) n'est **pas** thread-safe. L'appelant DOIT garantir
/// qu'au plus un thread exécute cette fonction à la fois (en 12-5c, le verrou de
/// run DC6 le garantit). Deux appels concurrents partageraient l'état global C de
/// pdfium → comportement indéfini.
pub fn decode_spc_from_pdf_bytes(
    bytes: &[u8],
    cfg: DecodeConfig,
) -> Result<Option<ScannedQrBill>, DecodeError> {
    use pdfium_render::prelude::*;

    // Sérialisation process-wide (pdfium non thread-safe) — cf. PDFIUM_LOCK.
    // `into_inner` récupère le garde même si un thread a paniqué en le tenant
    // (le poison n'affecte pas l'état C de pdfium, réinitialisé à chaque appel).
    let _pdfium_guard = PDFIUM_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    // Binding au binaire natif (libpdfium.so installé dans l'image Docker, DC1-bis).
    let bindings = Pdfium::bind_to_system_library()
        .map_err(|e| DecodeError::PdfRender(format!("binding pdfium: {e}")))?;
    let pdfium = Pdfium::new(bindings);

    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| DecodeError::PdfRender(format!("chargement PDF: {e}")))?;

    let render_config = PdfRenderConfig::new()
        .set_maximum_width(cfg.max_dimension as i32)
        .set_maximum_height(cfg.max_dimension as i32);

    // Une page illisible (ressource embarquée corrompue) ne doit pas abandonner
    // le décodage : le QR SPC peut être sur une page suivante. On retient la
    // dernière erreur de rendu pour la remonter SEULEMENT si aucune page ne porte
    // de QR (sinon on renverrait à tort `Ok(None)` alors qu'une page était illisible).
    let mut last_render_err: Option<String> = None;
    for (i, page) in document.pages().iter().enumerate() {
        if i >= cfg.max_pages {
            // Cap atteint (anti-DoS). Si une page antérieure a échoué au rendu, on
            // remonte CETTE erreur plutôt que le message de cap générique — sinon
            // le vrai diagnostic ("page 5 illisible") serait masqué par "trop de
            // pages sans QR" (code-review Pass 2 H1).
            let msg = last_render_err
                .take()
                .unwrap_or_else(|| format!("cap de {} pages atteint sans QR SPC", cfg.max_pages));
            return Err(DecodeError::PdfRender(msg));
        }
        let bitmap = match page.render_with_config(&render_config) {
            Ok(b) => b,
            Err(e) => {
                last_render_err = Some(format!("rendu page {i}: {e}"));
                continue;
            }
        };
        let img = bitmap.as_image();
        if let Some(scanned) = first_spc(&decode_qr_from_image(&img))? {
            return Ok(Some(scanned));
        }
    }
    match last_render_err {
        Some(msg) => Err(DecodeError::PdfRender(msg)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_qrbill::{Address, AddressType, Currency, QrBillData, Reference, build_payload};
    use rust_decimal_macros::dec;

    /// Construit une image PNG portant le QR d'un payload SPC valide, via le
    /// même chemin que `kesh-qrbill` (round-trip generator → rxing).
    fn render_spc_qr_png() -> (Vec<u8>, String) {
        let data = QrBillData {
            creditor_iban: "CH9300762011623852957".into(),
            creditor: Address {
                address_type: AddressType::Combined,
                name: "Robert Schneider SA".into(),
                line1: "Rue du Lac 1268".into(),
                line2: "2501 Biel".into(),
                postal_code: String::new(),
                town: String::new(),
                country: "CH".into(),
            },
            ultimate_debtor: None,
            amount: Some(dec!(100.00)),
            currency: Currency::Chf,
            reference: Reference::None,
            unstructured_message: Some("Facture test".into()),
            billing_information: None,
        };
        let payload = build_payload(&data).unwrap();

        // Rasterise le QR en PNG (module 10px, quiet zone 4 modules).
        let code = qrcodegen_image(&payload);
        let mut png = Vec::new();
        image::DynamicImage::ImageLuma8(code)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        (png, payload)
    }

    /// Rasterise un payload en bitmap luma via `qrcodegen` (dép de kesh-qrbill,
    /// ré-exportée pour les tests ? sinon reconstruire). Ici on passe par rxing
    /// writer pour rester self-contained.
    fn qrcodegen_image(payload: &str) -> image::GrayImage {
        use rxing::{BarcodeFormat, EncodeHints, Writer};
        let writer = rxing::qrcode::QRCodeWriter {};
        let matrix = writer
            .encode_with_hints(
                payload,
                &BarcodeFormat::QR_CODE,
                512,
                512,
                &EncodeHints::default(),
            )
            .expect("encode QR");
        let w = matrix.getWidth();
        let h = matrix.getHeight();
        image::GrayImage::from_fn(w, h, |x, y| {
            if matrix.get(x, y) {
                image::Luma([0u8])
            } else {
                image::Luma([255u8])
            }
        })
    }

    #[test]
    fn decode_spc_from_png_roundtrip() {
        let (png, payload) = render_spc_qr_png();
        let scanned = decode_spc_from_image_bytes(&png)
            .expect("decode ok")
            .expect("some QR SPC");
        assert_eq!(scanned.creditor_iban, "CH9300762011623852957");
        assert_eq!(scanned.amount, Some(dec!(100.00)));
        // Sanity : le payload décodé est bien le SPC d'origine.
        assert!(payload.starts_with(SPC_HEADER));
    }

    #[test]
    fn image_without_qr_yields_none() {
        // Image blanche 64×64, aucun QR.
        let white = image::DynamicImage::ImageLuma8(image::GrayImage::from_pixel(
            64,
            64,
            image::Luma([255u8]),
        ));
        let mut png = Vec::new();
        white
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        assert!(decode_spc_from_image_bytes(&png).unwrap().is_none());
    }

    #[test]
    fn corrupt_image_yields_error() {
        let err = decode_spc_from_image_bytes(b"not an image").unwrap_err();
        assert!(matches!(err, DecodeError::ImageDecode(_)));
    }

    // Les tests PDF nécessitent `libpdfium.so` (binaire natif), absent de l'hôte
    // CI standard → `#[ignore]`. Exécutés en local/Docker où pdfium est bundlé
    // (DC1-bis). `cargo test -- --ignored` pour les lancer avec la lib présente.
    #[test]
    #[ignore = "nécessite libpdfium.so (bundlé image Docker, absent hôte CI)"]
    fn decode_spc_from_pdf_single_page() {
        // Fixture PDF générée à la volée n'est pas triviale sans pdfium ; ce test
        // valide surtout le binding + le pipeline quand la lib est présente.
        // Un PDF fixture porteur d'un QR SPC peut être déposé dans tests/fixtures/.
        let pdf = std::fs::read("tests/fixtures/qr_invoice.pdf")
            .expect("fixture tests/fixtures/qr_invoice.pdf");
        let scanned = decode_spc_from_pdf_bytes(&pdf, DecodeConfig::default())
            .expect("decode ok")
            .expect("some QR SPC");
        assert!(!scanned.creditor_iban.is_empty());
    }
}
