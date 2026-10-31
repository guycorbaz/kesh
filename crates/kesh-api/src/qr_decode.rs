//! Décodage QR côté serveur des factures importées (Story 12-5b — #194).
//!
//! Pipeline (DC1) :
//! - **Image** (PNG/JPG) : `image::load_from_memory` → luma → `rxing` (QR→texte).
//! - **PDF** : `pdfium-render` rend chaque page → `image::DynamicImage` → `rxing`.
//!
//! On retient le **premier** payload commençant par `SPC\n0200` (multi-QR par
//! page, multi-page tentés dans l'ordre) puis on le parse via `parse_spc_payload`
//! (12-5a). Le mapping des erreurs vers les codes du rapport d'import
//! (`PDF_RENDER_ERROR`, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`) est consommé par
//! `inbox_import` ; ce module expose une erreur typée. Depuis la Story 30-2a,
//! « aucun QR » (`Ok(None)`) et ces trois erreurs font entrer le document SANS
//! données QR (`qr: "none"` / `"unreadable"`) ; seule `ImageDecode` le refuse.

use std::sync::Mutex;

use image::DynamicImage;
use kesh_qrbill::{QrBillError, ScannedQrBill, parse_spc_payload};

/// Scans synthétiques de QR-factures et fixtures (Story v017-5, #854) — tests seulement.
#[cfg(test)]
#[path = "qr_decode_synthetique.rs"]
pub(crate) mod synthetique;

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

/// Plafond de résolution du second rendu d'une page PDF (Story v017-5, #854) :
/// au-delà de la résolution d'un scan courant (150 à 300 dpi), un rendu plus fin
/// n'apporte aucune information et coûte des pixels.
pub const MAX_RENDER_DPI: f32 = 300.0;

/// Facteur d'échelle du **second** rendu d'une page PDF de `width_pt × height_pt`
/// points (Story v017-5, #854) : la plus haute résolution que permet la borne
/// `max_dimension` (pixels par côté), plafonnée à [`MAX_RENDER_DPI`] —
/// `min(max_dimension / largeur, max_dimension / hauteur, MAX_RENDER_DPI / 72)`.
/// Environ 2,38 (171 dpi) pour une A4.
///
/// `None` — pas de second rendu — pour une dimension nulle, négative ou non finie,
/// ou une borne nulle (garde explicite : `f32::min` ignore un `NaN`).
pub fn render_scale(width_pt: f32, height_pt: f32, max_dimension: u16) -> Option<f32> {
    if !(width_pt.is_finite() && width_pt > 0.0 && height_pt.is_finite() && height_pt > 0.0)
        || max_dimension == 0
    {
        return None;
    }
    let borne = f32::from(max_dimension);
    Some(
        (borne / width_pt)
            .min(borne / height_pt)
            .min(MAX_RENDER_DPI / 72.0),
    )
}

/// Les deux temps du rendu d'une page PDF (Story v017-5, #854, C-v017-5-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Temps {
    /// Le rendu d'avant la story : échelle 1,0 (72 dpi), bornes `max_dimension`.
    Premier,
    /// Le rendu à [`render_scale`], joué seulement si aucune page du document n'a
    /// livré de QR-facture au premier temps.
    Second,
}

/// Parcourt les pages d'un document et rend sa première QR-facture (Story v017-5,
/// #854, C-v017-5-15) — sans pdfium : `render(page, temps)` rend l'image de la page
/// (`Ok(None)` : pas d'image à ce temps), ou le message d'une erreur de rendu.
///
/// Conduite :
/// 1. **première passe** : les pages `0 .. min(page_count, max_pages)` dans l'ordre,
///    au premier temps ; la première QR-facture exploitable est rendue, une
///    QR-facture inexploitable est rendue en erreur aussitôt ; une erreur de rendu
///    est retenue (`last_render_err`) et la page est marquée ;
/// 2. **seconde passe**, seulement si aucune page de la première n'a livré de
///    QR-facture : les mêmes pages, dans l'ordre, **sauf** celles dont le premier
///    rendu a échoué, au second temps ; même conduite pour une QR-facture
///    exploitable ou inexploitable. Une erreur du second rendu est journalisée et
///    la page compte comme sans QR-facture : elle ne change **jamais** le
///    diagnostic du document ;
/// 3. **fin** : au-delà du cap, `PdfRender` (la dernière erreur de rendu, ou le
///    message du cap) ; sinon la dernière erreur de rendu ; sinon `Ok(None)`.
///
/// Un document dont une page livre une QR-facture au premier temps coûte donc ce
/// qu'il coûtait avant la story ; seul un document sans QR-facture au premier
/// temps paie la seconde passe.
pub(crate) fn decode_pages<R>(
    page_count: usize,
    max_pages: usize,
    mut render: R,
) -> Result<Option<ScannedQrBill>, DecodeError>
where
    R: FnMut(usize, Temps) -> Result<Option<DynamicImage>, String>,
{
    let limite = page_count.min(max_pages);
    // Une page illisible (ressource embarquée corrompue) ne doit pas abandonner
    // le décodage : le QR SPC peut être sur une page suivante. On retient la
    // dernière erreur de rendu pour la remonter SEULEMENT si aucune page ne porte
    // de QR (sinon on renverrait à tort `Ok(None)` alors qu'une page était illisible).
    let mut last_render_err: Option<String> = None;
    let mut rendue = vec![false; limite];
    for (i, ok) in rendue.iter_mut().enumerate() {
        match render(i, Temps::Premier) {
            Ok(Some(img)) => {
                *ok = true;
                if let Some(scanned) = first_spc(&decode_qr_from_image(&img))? {
                    return Ok(Some(scanned));
                }
            }
            Ok(None) => *ok = true,
            Err(e) => {
                last_render_err = Some(format!("rendu page {i}: {e}"));
            }
        }
    }
    // Seconde passe : aucune page n'a livré de QR-facture au premier temps.
    for (i, _) in rendue.iter().enumerate().filter(|(_, ok)| **ok) {
        match render(i, Temps::Second) {
            Ok(Some(img)) => {
                if let Some(scanned) = first_spc(&decode_qr_from_image(&img))? {
                    return Ok(Some(scanned));
                }
            }
            Ok(None) => {}
            Err(e) => {
                // La page s'est rendue au premier temps : l'échec du second rendu
                // n'est pas une erreur du document. Aucun contenu au journal.
                tracing::warn!(page = i, erreur = %e, "second rendu d'une page PDF impossible");
            }
        }
    }
    if page_count > max_pages {
        // Cap atteint (anti-DoS). Si une page antérieure a échoué au rendu, on
        // remonte CETTE erreur plutôt que le message de cap générique — sinon
        // le vrai diagnostic ("page 5 illisible") serait masqué par "trop de
        // pages sans QR" (code-review Pass 2 H1).
        let msg = last_render_err
            .take()
            .unwrap_or_else(|| format!("cap de {max_pages} pages atteint sans QR SPC"));
        return Err(DecodeError::PdfRender(msg));
    }
    match last_render_err {
        Some(msg) => Err(DecodeError::PdfRender(msg)),
        None => Ok(None),
    }
}

/// Décode **tous** les QR présents sur une image (ordre `rxing`).
///
/// Story v017-5 (#854, C-v017-5-3) : format QR seul (`PossibleFormats`) et
/// `TryHarder` — mesure de coût, sans effet sur ce que Kesh retient (seul un
/// payload `SPC\n0200` compte, `first_spc`).
pub fn decode_qr_from_image(img: &DynamicImage) -> Vec<String> {
    let luma = img.to_luma8();
    let (w, h) = luma.dimensions();
    let mut hints = rxing::DecodeHints {
        PossibleFormats: Some(std::collections::HashSet::from([
            rxing::BarcodeFormat::QR_CODE,
        ])),
        TryHarder: Some(true),
        ..rxing::DecodeHints::default()
    };
    match rxing::helpers::detect_multiple_in_luma_with_hints(luma.into_raw(), w, h, &mut hints) {
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
    decode_spc_from_pdf_bytes_observed(bytes, cfg, &mut |_, _| {})
}

/// [`decode_spc_from_pdf_bytes`], avec un observateur appelé avant chaque rendu
/// d'une page (numéro de page, temps) — point d'accroche de T4 et du banc de coût
/// (Story v017-5, AC-5), qui comptent les rendus du second temps.
pub(crate) fn decode_spc_from_pdf_bytes_observed(
    bytes: &[u8],
    cfg: DecodeConfig,
    on_render: &mut dyn FnMut(usize, Temps),
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

    // Premier temps : la configuration d'avant la story, inchangée (échelle 1,0,
    // soit 72 dpi, bornes `max_dimension`).
    let bornes = |c: PdfRenderConfig| {
        c.set_maximum_width(cfg.max_dimension as i32)
            .set_maximum_height(cfg.max_dimension as i32)
    };

    let pages = document.pages();
    let page_count = pages.len() as usize;
    decode_pages(page_count, cfg.max_pages, |i, temps| {
        on_render(i, temps);
        let page = pages.get(i as u16).map_err(|e| e.to_string())?;
        let config = match temps {
            Temps::Premier => bornes(PdfRenderConfig::new()),
            // Second temps : `render_scale`, sous les mêmes bornes ; pas d'image
            // quand la page n'a pas de dimension exploitable.
            Temps::Second => {
                match render_scale(page.width().value, page.height().value, cfg.max_dimension) {
                    Some(s) => bornes(PdfRenderConfig::new().scale_page_by_factor(s)),
                    None => return Ok(None),
                }
            }
        };
        page.render_with_config(&config)
            .map(|bitmap| Some(bitmap.as_image()))
            .map_err(|e| e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_qrbill::{Address, AddressType, Currency, QrBillData, Reference, build_payload};
    use rust_decimal_macros::dec;

    /// Construit une image PNG portant le QR d'un payload SPC valide, via le
    /// même chemin que `kesh-qrbill` (round-trip generator → rxing).
    fn render_spc_qr_png() -> (Vec<u8>, String) {
        let payload = build_payload(&donnees_spc_test()).unwrap();
        let mut png = Vec::new();
        image::DynamicImage::ImageLuma8(qrcodegen_image(&payload))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        (png, payload)
    }

    /// Données de la QR-facture de test (≈ 130 octets de payload, QR de version
    /// basse) : celles de `render_spc_qr_png` et du témoin vectoriel de T4.
    fn donnees_spc_test() -> QrBillData {
        QrBillData {
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
        }
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
        // Story v017-5 (C-v017-5-6) : la fixture d'origine (`qr_invoice.pdf`)
        // n'a jamais existé ; ce test lit le scan synthétique à 300 dpi, par un
        // chemin absolu fixé à la compilation.
        let pdf = synthetique::lire_fixture("qr-scan-300dpi.pdf");
        let scanned = decode_spc_from_pdf_bytes(&pdf, DecodeConfig::default())
            .expect("decode ok")
            .expect("some QR SPC");
        assert!(!scanned.creditor_iban.is_empty());
    }

    // --- Story v017-5 (#854) : le QR d'une facture scannée ------------------

    use kesh_qrbill::ScannedReference;
    use kesh_qrbill::generator::render_qr_image;

    /// Dimensions de sortie d'un rendu à l'échelle `s`, arrondies comme
    /// `apply_to_page` (`render_config.rs:825-826`).
    fn sortie(w_pt: f32, h_pt: f32, s: f32) -> (f32, f32) {
        ((w_pt * s).round(), (h_pt * s).round())
    }

    fn echelle(w_pt: f32, h_pt: f32) -> f32 {
        render_scale(w_pt, h_pt, 2000).expect("échelle définie")
    }

    fn proche(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    // T1 — `render_scale`, onze cas (AC-3).

    #[test]
    fn render_scale_a4_portrait() {
        let s = echelle(595.276, 841.89);
        assert!(proche(s, 2.3756), "échelle {s}");
        assert_eq!(sortie(595.276, 841.89, s), (1414.0, 2000.0));
    }

    #[test]
    fn render_scale_a4_paysage() {
        let s = echelle(841.89, 595.276);
        assert!(proche(s, 2.3756), "échelle {s}");
        assert_eq!(sortie(841.89, 595.276, s), (2000.0, 1414.0));
    }

    #[test]
    fn render_scale_section_paiement() {
        let s = echelle(595.28, 297.64);
        assert!(proche(s, 3.3598), "échelle {s}");
    }

    #[test]
    fn render_scale_a6_plafonnee_a_300_dpi() {
        let s = echelle(297.64, 419.53);
        assert!(proche(s, 300.0 / 72.0), "échelle {s}");
    }

    #[test]
    fn render_scale_a3_sous_la_borne() {
        let s = echelle(841.89, 1190.55);
        assert!(proche(s, 1.6799), "échelle {s}");
        let (w, h) = sortie(841.89, 1190.55, s);
        assert!(w <= 2000.0 && h <= 2000.0, "{w} × {h}");
    }

    #[test]
    fn render_scale_page_minuscule_plafonnee() {
        let s = echelle(10.0, 10.0);
        assert!(proche(s, 300.0 / 72.0), "échelle {s}");
    }

    #[test]
    fn render_scale_largeur_nulle() {
        assert_eq!(render_scale(0.0, 841.89, 2000), None);
    }

    #[test]
    fn render_scale_largeur_negative() {
        assert_eq!(render_scale(-1.0, 841.89, 2000), None);
    }

    #[test]
    fn render_scale_hauteur_nan() {
        assert_eq!(render_scale(595.276, f32::NAN, 2000), None);
    }

    #[test]
    fn render_scale_hauteur_infinie() {
        assert_eq!(render_scale(595.276, f32::INFINITY, 2000), None);
    }

    #[test]
    fn render_scale_borne_nulle() {
        assert_eq!(render_scale(595.276, 841.89, 0), None);
    }

    /// T1-bis — la grandeur causale (AC-4) : pour le QR des fixtures, moins de
    /// 2,5 px par module au premier rendu (72 dpi), plus de 4 au second.
    #[test]
    fn grandeur_causale_pixels_par_module() {
        let modules = render_qr_image(&synthetique::payload()).unwrap().size() as f32;
        let px_qr = |dpi: f32| 46.0 / 25.4 * dpi;
        let premier = px_qr(72.0) / modules;
        assert!(premier < 2.5, "premier rendu : {premier} px par module");
        let s = render_scale(595.276, 841.89, DecodeConfig::default().max_dimension).unwrap();
        let second = px_qr(72.0 * s) / modules;
        assert!(second > 4.0, "second rendu : {second} px par module");
    }

    /// La QR-facture synthétique lue porte l'IBAN, le montant et la référence du payload.
    fn assert_facture_synthetique(s: &ScannedQrBill) {
        assert_eq!(s.creditor_iban, synthetique::QR_IBAN_EXEMPLE);
        assert_eq!(s.amount, Some(dec!(1234.55)));
        assert_eq!(
            s.reference,
            ScannedReference::Qrr(synthetique::REFERENCE_QRR_EXEMPLE.into())
        );
    }

    /// Décode l'image JPEG `jpeg` rééchantillonnée à `w × h` (filtre `Triangle`).
    fn lire_reechantillonne(
        jpeg: &[u8],
        w: u32,
        h: u32,
    ) -> Result<Option<ScannedQrBill>, DecodeError> {
        let img = image::load_from_memory(jpeg).unwrap().to_luma8();
        let petit = image::imageops::resize(&img, w, h, image::imageops::FilterType::Triangle);
        first_spc(&decode_qr_from_image(&DynamicImage::ImageLuma8(petit)))
    }

    /// T2 — chacun des trois scans, décodé depuis ses octets JPEG et rééchantillonné
    /// aux dimensions du second rendu d'une A4 (1414 × 2000), est lu (AC-4).
    #[test]
    fn t2_scans_lus_au_second_rendu() {
        for (nom, dpi) in synthetique::FIXTURES_QR {
            let debut = std::time::Instant::now();
            let jpeg = synthetique::jpeg_qr_facture(dpi);
            let lu = lire_reechantillonne(&jpeg, 1414, 2000)
                .unwrap_or_else(|e| panic!("{nom} : {e}"))
                .unwrap_or_else(|| panic!("{nom} : aucune QR-facture à 1414 × 2000"));
            assert_facture_synthetique(&lu);
            eprintln!("T2 {nom} : lu en {:?}", debut.elapsed());
        }
    }

    /// T2-diag — les mêmes images à 595 × 842 (premier rendu d'une A4) :
    /// diagnostic, joué à la main avec T4, résultat déclaré au Dev Agent Record.
    #[test]
    #[ignore = "diagnostic (AC-4) : joué à la main avec T4"]
    fn t2_diag_scans_au_premier_rendu() {
        for (nom, dpi) in synthetique::FIXTURES_QR {
            let jpeg = synthetique::jpeg_qr_facture(dpi);
            let r = lire_reechantillonne(&jpeg, 595, 842);
            eprintln!(
                "T2-diag {nom} à 595 × 842 : {}",
                match &r {
                    Ok(Some(_)) => "LU".to_string(),
                    Ok(None) => "non lu".to_string(),
                    Err(e) => format!("erreur {e}"),
                }
            );
        }
    }

    /// T3 — contrôle : les mêmes octets JPEG, à leur résolution native, par le
    /// chemin image, sont lus (AC-4).
    #[test]
    fn t3_scans_lus_par_le_chemin_image() {
        for (nom, dpi) in synthetique::FIXTURES_QR {
            let debut = std::time::Instant::now();
            let jpeg = synthetique::jpeg_qr_facture(dpi);
            let lu = decode_spc_from_image_bytes(&jpeg)
                .unwrap_or_else(|e| panic!("{nom} : {e}"))
                .unwrap_or_else(|| panic!("{nom} : aucune QR-facture à la résolution native"));
            assert_facture_synthetique(&lu);
            eprintln!("T3 {nom} : lu en {:?}", debut.elapsed());
        }
    }

    /// Décode un PDF en comptant les rendus du second temps.
    fn decoder_en_comptant(pdf: &[u8]) -> (Result<Option<ScannedQrBill>, DecodeError>, usize) {
        let mut seconds = 0usize;
        let r = decode_spc_from_pdf_bytes_observed(pdf, DecodeConfig::default(), &mut |_, t| {
            if t == Temps::Second {
                seconds += 1;
            }
        });
        (r, seconds)
    }

    fn decrire(r: &Result<Option<ScannedQrBill>, DecodeError>) -> String {
        match r {
            Ok(Some(_)) => "LU".into(),
            Ok(None) => "Ok(None)".into(),
            Err(e) => format!("Err({e})"),
        }
    }

    /// PDF du témoin vectoriel : une facture QR émise par `kesh-qrbill`, dont la
    /// QR-facture est celle de `render_spc_qr_png` (version basse).
    fn pdf_temoin_vectoriel() -> Vec<u8> {
        use kesh_qrbill::{InvoiceLinePdf, InvoicePdfData, QrBillI18n};
        let facture = InvoicePdfData {
            invoice_number: "F-TEST-1".into(),
            invoice_date: chrono::NaiveDate::from_ymd_opt(2026, 10, 31).unwrap(),
            due_date: None,
            payment_terms: None,
            creditor_name: "Robert Schneider SA".into(),
            creditor_address_lines: vec!["Rue du Lac 1268".into(), "2501 Biel".into()],
            creditor_ide: None,
            creditor_phone: None,
            creditor_email: None,
            creditor_website: None,
            debtor_name: "Personne Fictive".into(),
            debtor_address_lines: vec!["Chemin des Essais 7".into(), "1998 Exempleville".into()],
            debtor_client_number: None,
            lines: vec![InvoiceLinePdf {
                description: "Prestation de test".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(0),
                line_total: dec!(100.00),
            }],
            subtotal_ht: dec!(100.00),
            vat_lines: vec![],
            rounding: rust_decimal::Decimal::ZERO,
            total: dec!(100.00),
            currency: Currency::Chf,
            origin_reference: None,
            reminder: None,
        };
        kesh_qrbill::generate_qr_bill_pdf_with_date(
            &donnees_spc_test(),
            &facture,
            &QrBillI18n::new(std::collections::HashMap::new()),
            time::OffsetDateTime::UNIX_EPOCH,
        )
        .expect("PDF du témoin vectoriel")
    }

    /// T4 — la preuve par pdfium (AC-2, AC-5), jouée en local avec la bibliothèque
    /// épinglée :
    ///
    /// ```sh
    /// LD_LIBRARY_PATH=/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/lib \
    ///   cargo test -p kesh-api --lib qr_decode::tests::t4_pdfium -- --ignored --nocapture
    /// ```
    ///
    /// (α) sonde : la page de `qr-scan-300dpi.pdf` rendue à 595 × 842 n'est pas
    /// uniforme ; (β) les trois scans sont lus, `scan-sans-qr.pdf` rend
    /// `Ok(None)` ; (γ) le témoin vectoriel est lu dès la première passe.
    #[test]
    #[ignore = "nécessite libpdfium.so (LD_LIBRARY_PATH vers la release épinglée)"]
    fn t4_pdfium_scans_synthetiques() {
        use pdfium_render::prelude::*;
        // (α) La sonde, indépendante de la résolution.
        {
            let _g = PDFIUM_LOCK.lock().unwrap_or_else(|p| p.into_inner());
            let pdfium = Pdfium::new(Pdfium::bind_to_system_library().expect("libpdfium"));
            let octets = synthetique::lire_fixture("qr-scan-300dpi.pdf");
            let doc = pdfium
                .load_pdf_from_byte_slice(&octets, None)
                .expect("chargement");
            let page = doc.pages().get(0).expect("page 0");
            let img = page
                .render_with_config(&PdfRenderConfig::new())
                .expect("rendu")
                .as_image()
                .to_luma8();
            let mut histo = [0usize; 256];
            img.pixels().for_each(|p| histo[p.0[0] as usize] += 1);
            let total = (img.width() * img.height()) as usize;
            let niveaux = histo.iter().filter(|&&n| n * 100 > total).count();
            eprintln!(
                "T4 (α) sonde : {} × {} px, {niveaux} niveaux de gris sur plus de 1 % des pixels",
                img.width(),
                img.height()
            );
            assert_eq!((img.width(), img.height()), (595, 842));
            assert!(
                niveaux >= 2,
                "page uniforme : le rendu pdfium ne fonctionne pas"
            );
        }
        // (β) Les fixtures.
        let mut echecs = Vec::new();
        for (nom, _) in synthetique::FIXTURES_QR {
            let (r, seconds) = decoder_en_comptant(&synthetique::lire_fixture(nom));
            eprintln!(
                "T4 (β) {nom} : {} (rendus au second temps : {seconds})",
                decrire(&r)
            );
            match r {
                Ok(Some(s)) => assert_facture_synthetique(&s),
                _ => echecs.push(nom),
            }
        }
        let (r, seconds) =
            decoder_en_comptant(&synthetique::lire_fixture(synthetique::FIXTURE_SANS_QR));
        eprintln!(
            "T4 (β) {} : {} (rendus au second temps : {seconds})",
            synthetique::FIXTURE_SANS_QR,
            decrire(&r)
        );
        assert!(matches!(r, Ok(None)), "scan sans QR : {}", decrire(&r));
        // (γ) Le témoin vectoriel, lu dès la première passe.
        let (r, seconds) = decoder_en_comptant(&pdf_temoin_vectoriel());
        eprintln!(
            "T4 (γ) témoin vectoriel : {} (rendus au second temps : {seconds})",
            decrire(&r)
        );
        let lu = r
            .expect("témoin : décodage")
            .expect("témoin vectoriel non lu");
        assert_eq!(lu.creditor_iban, "CH9300762011623852957");
        assert_eq!(seconds, 0, "le témoin vectoriel a payé la seconde passe");
        assert!(echecs.is_empty(), "scans non lus : {echecs:?}");
    }

    /// Médiane de `n` mesures.
    fn mediane(mut v: Vec<std::time::Duration>) -> std::time::Duration {
        v.sort();
        v[v.len() / 2]
    }

    /// T5 — banc de coût (AC-6), en `--release`, sous `/usr/bin/time -v` pour la
    /// mémoire de pointe (commande dans la fiche).
    #[test]
    #[ignore = "banc de coût (AC-6) : nécessite libpdfium.so, profil --release"]
    fn banc_cout() {
        use synthetique::{DEGRADATION, PageImage, QUALITE_JPEG, QUALITE_JPEG_SANS_QR};
        // (a) Temps par page de chaque fixture, médiane de 5 passages.
        for nom in synthetique::FIXTURES_QR
            .iter()
            .map(|(n, _)| *n)
            .chain([synthetique::FIXTURE_SANS_QR])
        {
            let pdf = synthetique::lire_fixture(nom);
            let mut mesures = Vec::new();
            let mut issue = String::new();
            for _ in 0..5 {
                let debut = std::time::Instant::now();
                let (r, _) = decoder_en_comptant(&pdf);
                mesures.push(debut.elapsed());
                issue = decrire(&r);
            }
            eprintln!(
                "banc (a) {nom} : médiane {:?} par page ({issue})",
                mediane(mesures)
            );
        }
        // (b) Quatre documents « pire cas », engendrés en mémoire.
        let a4 = PageImage::new(
            &synthetique::page_sans_qr(210.0, 297.0, 300.0, DEGRADATION),
            QUALITE_JPEG_SANS_QR,
            210.0,
            297.0,
        );
        let d1: Vec<PageImage> = (0..20)
            .map(|_| PageImage {
                jpeg: a4.jpeg.clone(),
                ..a4
            })
            .collect();
        let carre = PageImage::new(
            &synthetique::page_sans_qr(297.0, 297.0, 200.0, DEGRADATION),
            QUALITE_JPEG_SANS_QR,
            297.0,
            297.0,
        );
        let d2: Vec<PageImage> = (0..20)
            .map(|_| PageImage {
                jpeg: carre.jpeg.clone(),
                ..carre
            })
            .collect();
        let qr_non_spc = PageImage::new(
            &synthetique::page_qr_non_spc(200.0, DEGRADATION),
            QUALITE_JPEG,
            210.0,
            297.0,
        );
        let d3: Vec<PageImage> = (0..20)
            .map(|_| PageImage {
                jpeg: qr_non_spc.jpeg.clone(),
                ..qr_non_spc
            })
            .collect();
        let d4 = synthetique::pdf_vectoriel(3, &build_payload(&donnees_spc_test()).unwrap());
        let documents = [
            (
                "D1 (20 × A4 scannée sans QR, 300 dpi)",
                synthetique::pdf_images(&d1),
            ),
            (
                "D2 (20 × carrée 842 pt sans QR, 200 dpi)",
                synthetique::pdf_images(&d2),
            ),
            (
                "D3 (20 × A4 à douze QR non SPC, 200 dpi)",
                synthetique::pdf_images(&d3),
            ),
            ("D4 (facture vectorielle de 3 pages, QR en page 3)", d4),
        ];
        for (nom, pdf) in &documents {
            let debut = std::time::Instant::now();
            let (r, seconds) = decoder_en_comptant(pdf);
            let duree = debut.elapsed();
            eprintln!(
                "banc (b) {nom} : {duree:?} ({:?} par page), {} ; rendus au second temps : {seconds} ; {} octets",
                duree / if nom.starts_with("D4") { 3 } else { 20 },
                decrire(&r),
                pdf.len()
            );
        }
    }

    // T10 — `decode_pages` sur un faux rendeur (AC-3), en CI.

    /// Ce que le faux rendeur rend pour une page à un temps.
    #[derive(Clone, Copy, Debug)]
    enum Faux {
        /// Une image sans QR.
        Vide,
        /// Une image portant une QR-facture exploitable.
        Spc,
        /// Une image portant une QR-facture inexploitable.
        SpcInvalide,
        /// Une image portant un QR d'un autre usage.
        NonSpc,
        /// Une erreur de rendu.
        Erreur,
    }

    fn image_de(f: Faux) -> Result<Option<DynamicImage>, String> {
        let qr = |payload: &str| Ok(Some(DynamicImage::ImageLuma8(qrcodegen_image(payload))));
        match f {
            Faux::Vide => Ok(Some(DynamicImage::ImageLuma8(
                image::GrayImage::from_pixel(64, 64, image::Luma([255u8])),
            ))),
            Faux::Spc => qr(&build_payload(&donnees_spc_test()).unwrap()),
            Faux::SpcInvalide => qr("SPC\n0200\n1\nPAS-UN-IBAN"),
            Faux::NonSpc => qr("https://exemple.invalid/produit/0001"),
            Faux::Erreur => Err("page illisible (simulée)".into()),
        }
    }

    /// Joue `decode_pages` sur un plan `(premier temps, second temps)` par page ;
    /// rend le résultat et la suite des appels.
    fn jouer(
        plan: &[(Faux, Faux)],
        page_count: usize,
        max_pages: usize,
    ) -> (
        Result<Option<ScannedQrBill>, DecodeError>,
        Vec<(usize, Temps)>,
    ) {
        let mut appels = Vec::new();
        let r = decode_pages(page_count, max_pages, |i, t| {
            appels.push((i, t));
            let (p, s) = plan.get(i).copied().unwrap_or((Faux::Vide, Faux::Vide));
            image_de(if t == Temps::Premier { p } else { s })
        });
        (r, appels)
    }

    fn seconds(appels: &[(usize, Temps)]) -> Vec<usize> {
        appels
            .iter()
            .filter(|(_, t)| *t == Temps::Second)
            .map(|(i, _)| *i)
            .collect()
    }

    /// (a) QR-facture lisible au premier temps en page 3 → trouvée, aucun second temps.
    #[test]
    fn t10_a_premier_temps_suffit() {
        let plan = [
            (Faux::Vide, Faux::Vide),
            (Faux::Vide, Faux::Vide),
            (Faux::Spc, Faux::Spc),
        ];
        let (r, appels) = jouer(&plan, 3, 20);
        assert!(matches!(r, Ok(Some(_))), "{}", decrire(&r));
        assert!(seconds(&appels).is_empty(), "appels : {appels:?}");
    }

    /// (b) Aucune QR-facture au premier temps, lisible au second en page 2 →
    /// trouvée, les trois pages rendues au premier temps avant tout second temps.
    #[test]
    fn t10_b_second_temps_lit() {
        let plan = [
            (Faux::Vide, Faux::Vide),
            (Faux::Vide, Faux::Spc),
            (Faux::Vide, Faux::Vide),
        ];
        let (r, appels) = jouer(&plan, 3, 20);
        assert!(matches!(r, Ok(Some(_))), "{}", decrire(&r));
        assert_eq!(
            &appels[..3],
            &[
                (0, Temps::Premier),
                (1, Temps::Premier),
                (2, Temps::Premier)
            ]
        );
        assert_eq!(seconds(&appels), vec![0, 1]);
    }

    /// (c) QR-facture inexploitable au premier temps → l'erreur, aucun second temps.
    #[test]
    fn t10_c_inexploitable_sans_second_temps() {
        let plan = [(Faux::SpcInvalide, Faux::Spc), (Faux::Vide, Faux::Spc)];
        let (r, appels) = jouer(&plan, 2, 20);
        assert!(
            matches!(
                r,
                Err(DecodeError::InvalidSpcPayload(_)) | Err(DecodeError::InvalidIban(_))
            ),
            "{}",
            decrire(&r)
        );
        assert!(seconds(&appels).is_empty(), "appels : {appels:?}");
    }

    /// (d) Erreur de rendu de la page 2 au premier temps, aucune QR-facture
    /// ailleurs → page 2 non rendue au second temps, résultat `PdfRender`.
    #[test]
    fn t10_d_page_en_erreur_ecartee_du_second_temps() {
        let plan = [
            (Faux::Vide, Faux::Vide),
            (Faux::Erreur, Faux::Spc),
            (Faux::Vide, Faux::Vide),
        ];
        let (r, appels) = jouer(&plan, 3, 20);
        assert!(
            matches!(r, Err(DecodeError::PdfRender(_))),
            "{}",
            decrire(&r)
        );
        assert!(!seconds(&appels).contains(&1), "appels : {appels:?}");
    }

    /// (e) Erreur de rendu au second temps, aucune QR-facture → `Ok(None)`, non
    /// `PdfRender` : le second temps ne change jamais le diagnostic.
    #[test]
    fn t10_e_erreur_du_second_rendu_sans_effet() {
        let plan = [(Faux::Vide, Faux::Erreur), (Faux::Vide, Faux::Vide)];
        let (r, _) = jouer(&plan, 2, 20);
        assert!(matches!(r, Ok(None)), "{}", decrire(&r));
    }

    /// (f) 25 pages, cap de 20, aucune QR-facture → pages 20 à 24 rendues à aucun
    /// temps, résultat `PdfRender` « cap ».
    #[test]
    fn t10_f_cap_de_pages() {
        let (r, appels) = jouer(&[], 25, 20);
        match &r {
            Err(DecodeError::PdfRender(m)) => assert!(m.contains("cap de 20 pages"), "{m}"),
            autre => panic!("attendu PdfRender « cap », obtenu {}", decrire(autre)),
        }
        assert!(appels.iter().all(|(i, _)| *i < 20), "appels : {appels:?}");
    }

    /// (g) Une page portant un QR non SPC au premier temps → second temps appelé.
    #[test]
    fn t10_g_qr_non_spc_declenche_le_second_temps() {
        let plan = [(Faux::NonSpc, Faux::NonSpc)];
        let (r, appels) = jouer(&plan, 1, 20);
        assert!(matches!(r, Ok(None)), "{}", decrire(&r));
        assert_eq!(seconds(&appels), vec![0]);
    }
}
