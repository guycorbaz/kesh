//! Scans synthétiques de QR-factures — générateur de test (Story v017-5, #854).
//!
//! Engendre, à graine fixe, l'image en niveaux de gris d'une page A4 **scannée** :
//! une QR-facture dont le QR est dessiné par le générateur de Kesh
//! ([`kesh_qrbill::generator::render_qr_image`], niveau de correction **M** de la
//! norme SIX), à la place que lui donne la norme (section paiement en bas, QR de
//! 46 mm à 67 mm du bord gauche et 209 mm du haut, croix suisse de 7 mm), puis
//! dégradée comme un scan : flou gaussien, bruit uniforme, papier gris clair et
//! encre gris foncé ; l'image est encodée en JPEG et enveloppée dans un PDF d'une
//! page A4 (`printpdf` 0.7, `ImageXObject` au filtre DCT).
//!
//! ⛔ Aucune donnée réelle : IBAN et QR-IBAN d'exemple de SIX, référence QRR
//! d'exemple de SIX, noms et adresses fictifs.
//!
//! # Les fixtures versionnées
//!
//! Les quatre PDF de `crates/kesh-api/tests/fixtures/` (`qr-scan-150dpi.pdf`,
//! `qr-scan-200dpi.pdf`, `qr-scan-300dpi.pdf`, `scan-sans-qr.pdf`) sont **produits
//! une fois** par le test `#[ignore]` `qr_decode::synthetique::tests::ecrire_fixtures`
//! et versionnés comme données :
//!
//! ```sh
//! CARGO_TARGET_DIR=<worktree>/target cargo test -p kesh-api --lib \
//!   qr_decode::synthetique::tests::ecrire_fixtures -- --ignored --exact
//! ```
//!
//! La régénération n'est **pas** reproductible au bit près — `printpdf` tire un
//! identifiant d'instance et date le document à chaque écriture — mais l'image et
//! son JPEG le sont : les tests d'hôte (T2, T3) régénèrent les octets JPEG en
//! mémoire, identiques à ceux des fixtures. La preuve porte sur le **décodage**
//! (lu, non lu), non sur les octets du PDF.

use image::GrayImage;
use kesh_qrbill::generator::render_qr_image;
use kesh_qrbill::{Address, AddressType, Currency, QrBillData, Reference, build_payload};
use rust_decimal::Decimal;

/// QR-IBAN d'exemple de SIX (IID 31999) — exigé par une référence QRR.
pub(crate) const QR_IBAN_EXEMPLE: &str = "CH4431999123000889012";
/// Référence QRR d'exemple de SIX (27 chiffres, clé modulo 10 récursive).
pub(crate) const REFERENCE_QRR_EXEMPLE: &str = "210000000003139471430009017";

/// Qualité JPEG des fixtures (C-v017-5-5 : jamais sous 60).
pub(crate) const QUALITE_JPEG: u8 = 65;
/// Qualité JPEG de la page scannée sans QR (AC-1 : son bruit couvre toute la page,
/// seule la qualité la règle, jamais sous 60).
pub(crate) const QUALITE_JPEG_SANS_QR: u8 = 60;
/// Résolution de la page scannée sans QR (`scan-sans-qr.pdf`) : 200 dpi, la page
/// à 300 dpi dépassant 1 Mio même à la qualité 60 (AC-1 ; le banc, lui, engendre
/// sa page D1 à 300 dpi en mémoire).
pub(crate) const DPI_SANS_QR: f32 = 200.0;
/// Graine du générateur de bruit et des blocs de « texte ».
const GRAINE: u64 = 0x5EED_0854_0000_0001;

/// Dégradation d'un scan synthétique.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Degradation {
    /// Écart-type du flou gaussien, en millimètres.
    pub flou_mm: f32,
    /// Amplitude du bruit uniforme, en niveaux (± `bruit`).
    pub bruit: i16,
    /// Niveau du papier (0 = noir, 255 = blanc).
    pub papier: u8,
    /// Niveau de l'encre.
    pub encre: u8,
    /// Image binarisée avant l'encodage JPEG (palier 3 des branches d'arrêt, AC-2 (i)).
    pub binarise: bool,
}

/// Dégradation des fixtures : **palier 2** des branches d'arrêt (AC-2 (i)) — flou
/// de 0,2 mm, bruit de ± 24 niveaux, papier à 240, encre à 30. La dégradation
/// d'AC-1 (flou de 0,1 mm, bruit de ± 12) et le palier 1 (bruit de ± 24) restaient
/// lus à 72 dpi par pdfium : ils ne reproduisaient pas #854 (journaux au Dev Agent
/// Record de la story).
pub(crate) const DEGRADATION: Degradation = Degradation {
    flou_mm: 0.2,
    bruit: 24,
    papier: 240,
    encre: 30,
    binarise: false,
};

/// Données de la QR-facture synthétique : QR-IBAN et référence QRR d'exemple de
/// SIX, adresses structurées, débiteur final et message — un payload d'au moins
/// 280 octets UTF-8, donc un QR de version 12 à 15 au niveau M (assertions dans
/// les tests).
pub(crate) fn donnees_facture() -> QrBillData {
    QrBillData {
        creditor_iban: QR_IBAN_EXEMPLE.into(),
        creditor: Address {
            address_type: AddressType::Structured,
            name: "Commune fictive de Valbois-sur-Exemple, Service des finances".into(),
            line1: "Place de la Maison de Commune".into(),
            line2: "12".into(),
            postal_code: "1999".into(),
            town: "Valbois-sur-Exemple".into(),
            country: "CH".into(),
        },
        ultimate_debtor: Some(Address {
            address_type: AddressType::Structured,
            name: "Personne Fictive-Exemplaire".into(),
            line1: "Chemin des Essais".into(),
            line2: "7".into(),
            postal_code: "1998".into(),
            town: "Exempleville".into(),
            country: "CH".into(),
        }),
        amount: Some(Decimal::new(123_455, 2)),
        currency: Currency::Chf,
        reference: Reference::Qrr(REFERENCE_QRR_EXEMPLE.into()),
        unstructured_message: Some(
            "Facture synthetique de test, taxe d'exemple 2026, periode du 1er janvier au 31 decembre"
                .into(),
        ),
        billing_information: None,
    }
}

/// Payload SPC de la QR-facture synthétique.
pub(crate) fn payload() -> String {
    build_payload(&donnees_facture()).expect("payload synthétique valide")
}

/// Générateur pseudo-aléatoire à graine fixe (xorshift64*) : aucune dépendance,
/// même suite sur toute machine.
pub(crate) struct Alea(u64);

impl Alea {
    pub(crate) fn new(graine: u64) -> Self {
        Self(graine.max(1))
    }

    fn suivant(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Entier uniforme dans `[bas, haut]`.
    pub(crate) fn entre(&mut self, bas: i64, haut: i64) -> i64 {
        let n = (haut - bas + 1) as u64;
        bas + (self.suivant() % n) as i64
    }
}

/// Millimètres → pixels à `dpi`.
fn px(mm: f32, dpi: f32) -> i64 {
    (mm / 25.4 * dpi).round() as i64
}

/// Page vierge (papier) de `largeur_mm × hauteur_mm` à `dpi`.
fn page_vierge(largeur_mm: f32, hauteur_mm: f32, dpi: f32, papier: u8) -> GrayImage {
    GrayImage::from_pixel(
        px(largeur_mm, dpi) as u32,
        px(hauteur_mm, dpi) as u32,
        image::Luma([papier]),
    )
}

/// Remplit un rectangle (en pixels, bornes exclues à droite et en bas).
fn rectangle(img: &mut GrayImage, x0: i64, y0: i64, x1: i64, y1: i64, niveau: u8) {
    let (w, h) = (img.width() as i64, img.height() as i64);
    for y in y0.max(0)..y1.min(h) {
        for x in x0.max(0)..x1.min(w) {
            img.put_pixel(x as u32, y as u32, image::Luma([niveau]));
        }
    }
}

/// Lignes de « texte » figurées par des blocs sombres de largeurs tirées au sort,
/// entre `haut_mm` et `bas_mm`, de `gauche_mm` à `droite_mm`.
#[allow(clippy::too_many_arguments)]
fn lignes_de_texte(
    img: &mut GrayImage,
    dpi: f32,
    alea: &mut Alea,
    gauche_mm: f32,
    droite_mm: f32,
    haut_mm: f32,
    bas_mm: f32,
    encre: u8,
) {
    let interligne = 5.0_f32;
    let hauteur = 2.4_f32;
    let mut y = haut_mm;
    while y + hauteur <= bas_mm {
        // Longueur de la ligne : de 30 % à 100 % de la colonne.
        let colonne = droite_mm - gauche_mm;
        let fin = gauche_mm + colonne * alea.entre(30, 100) as f32 / 100.0;
        let mut x = gauche_mm;
        while x < fin {
            // Un « mot » de 3 à 18 mm, suivi d'une espace de 1,5 mm.
            let mot = alea.entre(3, 18) as f32;
            let x_fin = (x + mot).min(fin);
            rectangle(
                img,
                px(x, dpi),
                px(y, dpi),
                px(x_fin, dpi),
                px(y + hauteur, dpi),
                encre,
            );
            x = x_fin + 1.5;
        }
        y += interligne;
    }
}

/// Dessine le QR de `payload` (46 mm de côté) et la croix suisse (7 mm) à la place
/// de la norme : 67 mm du bord gauche, 209 mm du haut.
fn dessiner_qr(img: &mut GrayImage, dpi: f32, payload: &str, papier: u8, encre: u8) {
    let (x0_mm, y0_mm, cote_mm) = (67.0_f32, 209.0_f32, 46.0_f32);
    dessiner_modules(img, dpi, payload, (x0_mm, y0_mm, cote_mm), encre);
    // Croix suisse : garde blanche de 8 mm, carré « rouge » (gris foncé au scan)
    // de 7 mm, croix blanche de 4,55 × 1,3 mm.
    let (cx, cy) = (x0_mm + cote_mm / 2.0, y0_mm + cote_mm / 2.0);
    let carre = |img: &mut GrayImage, demi: f32, niveau: u8| {
        rectangle(
            img,
            px(cx - demi, dpi),
            px(cy - demi, dpi),
            px(cx + demi, dpi),
            px(cy + demi, dpi),
            niveau,
        )
    };
    carre(img, 4.0, papier);
    carre(img, 3.5, 90);
    rectangle(
        img,
        px(cx - 2.275, dpi),
        px(cy - 0.65, dpi),
        px(cx + 2.275, dpi),
        px(cy + 0.65, dpi),
        papier,
    );
    rectangle(
        img,
        px(cx - 0.65, dpi),
        px(cy - 2.275, dpi),
        px(cx + 0.65, dpi),
        px(cy + 2.275, dpi),
        papier,
    );
}

/// Dessine les modules du QR de `payload` dans le carré `(x, y, côté)` (mm, depuis
/// le coin supérieur gauche de la page).
fn dessiner_modules(
    img: &mut GrayImage,
    dpi: f32,
    payload: &str,
    carre_mm: (f32, f32, f32),
    encre: u8,
) {
    let qr = render_qr_image(payload).expect("QR synthétique");
    let taille = qr.size() as i64;
    let (x0, y0) = (px(carre_mm.0, dpi), px(carre_mm.1, dpi));
    let cote = px(carre_mm.2, dpi);
    for dy in 0..cote {
        let my = (dy * taille / cote) as i32;
        for dx in 0..cote {
            let mx = (dx * taille / cote) as i32;
            if qr.get_module(mx, my) {
                img.put_pixel((x0 + dx) as u32, (y0 + dy) as u32, image::Luma([encre]));
            }
        }
    }
}

/// Image d'une page A4 scannée portant une douzaine de QR **d'un autre usage**
/// (liens fictifs), sans QR-facture (banc, D3).
pub(crate) fn page_qr_non_spc(dpi: f32, d: Degradation) -> GrayImage {
    let mut alea = Alea::new(GRAINE ^ 0x0D3 ^ dpi as u64);
    let mut img = page_vierge(210.0, 297.0, dpi, d.papier);
    for ligne in 0..4 {
        for colonne in 0..3 {
            let k = ligne * 3 + colonne;
            let payload = format!("https://exemple.invalid/produit/{k:04}");
            let (x, y) = (15.0 + colonne as f32 * 65.0, 15.0 + ligne as f32 * 70.0);
            dessiner_modules(&mut img, dpi, &payload, (x, y, 30.0), d.encre);
            lignes_de_texte(
                &mut img,
                dpi,
                &mut alea,
                x,
                x + 55.0,
                y + 35.0,
                y + 55.0,
                d.encre,
            );
        }
    }
    degrader(&img, dpi, d, &mut alea, 0)
}

/// Flou gaussien séparable d'écart-type `sigma` pixels (noyau tronqué à 3 σ).
fn flou_gaussien(img: &GrayImage, sigma: f32) -> GrayImage {
    if sigma <= 0.0 {
        return img.clone();
    }
    let rayon = (3.0 * sigma).ceil() as i64;
    let mut noyau: Vec<f32> = (-rayon..=rayon)
        .map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let somme: f32 = noyau.iter().sum();
    noyau.iter_mut().for_each(|k| *k /= somme);
    let (w, h) = (img.width() as i64, img.height() as i64);
    let src = img.as_raw();
    let mut tmp = vec![0f32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0;
            for (k, poids) in noyau.iter().enumerate() {
                let xx = (x + k as i64 - rayon).clamp(0, w - 1);
                acc += poids * src[(y * w + xx) as usize] as f32;
            }
            tmp[(y * w + x) as usize] = acc;
        }
    }
    let mut out = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0;
            for (k, poids) in noyau.iter().enumerate() {
                let yy = (y + k as i64 - rayon).clamp(0, h - 1);
                acc += poids * tmp[(yy * w + x) as usize];
            }
            out[(y * w + x) as usize] = acc.round().clamp(0.0, 255.0) as u8;
        }
    }
    GrayImage::from_raw(w as u32, h as u32, out).expect("dimensions")
}

/// Applique la dégradation d'un scan : flou sur toute la page, bruit uniforme sur
/// les lignes `bruit_depuis_y ..` (0 : toute la page), binarisation éventuelle.
fn degrader(
    img: &GrayImage,
    dpi: f32,
    d: Degradation,
    alea: &mut Alea,
    bruit_depuis_y: u32,
) -> GrayImage {
    let mut out = flou_gaussien(img, d.flou_mm / 25.4 * dpi);
    for (_, y, p) in out.enumerate_pixels_mut() {
        if y < bruit_depuis_y {
            continue;
        }
        let v = p.0[0] as i64 + alea.entre(-(d.bruit as i64), d.bruit as i64);
        p.0[0] = v.clamp(0, 255) as u8;
    }
    if d.binarise {
        let seuil = (d.papier as u16 + d.encre as u16) / 2;
        for p in out.pixels_mut() {
            p.0[0] = if (p.0[0] as u16) < seuil { 0 } else { 255 };
        }
    }
    out
}

/// Image d'une page A4 scannée **portant** la QR-facture synthétique, à `dpi`.
pub(crate) fn page_qr_facture(dpi: f32, d: Degradation) -> GrayImage {
    let mut alea = Alea::new(GRAINE ^ dpi as u64);
    let mut img = page_vierge(210.0, 297.0, dpi, d.papier);
    // Corps de la facture.
    lignes_de_texte(&mut img, dpi, &mut alea, 20.0, 190.0, 20.0, 180.0, d.encre);
    // Ligne de séparation de la section paiement (192 mm du haut).
    rectangle(
        &mut img,
        0,
        px(192.0, dpi),
        px(210.0, dpi),
        px(192.2, dpi),
        d.encre,
    );
    // Récépissé et partie « indications » de la section paiement.
    lignes_de_texte(&mut img, dpi, &mut alea, 5.0, 57.0, 197.0, 287.0, d.encre);
    lignes_de_texte(
        &mut img, dpi, &mut alea, 118.0, 205.0, 197.0, 287.0, d.encre,
    );
    dessiner_qr(&mut img, dpi, &payload(), d.papier, d.encre);
    // Bruit restreint à la section paiement (AC-1 : pour tenir le plafond de
    // 1 Mio des fixtures à QR ; le QR et la croix y sont tout entiers).
    degrader(&img, dpi, d, &mut alea, px(192.0, dpi) as u32)
}

/// Image d'une page scannée **pleine, sans QR** de `largeur_mm × hauteur_mm` :
/// lignes de « texte » sur toute la hauteur, même flou, même bruit.
pub(crate) fn page_sans_qr(
    largeur_mm: f32,
    hauteur_mm: f32,
    dpi: f32,
    d: Degradation,
) -> GrayImage {
    let mut alea = Alea::new(GRAINE ^ 0xA5A5 ^ dpi as u64);
    let mut img = page_vierge(largeur_mm, hauteur_mm, dpi, d.papier);
    lignes_de_texte(
        &mut img,
        dpi,
        &mut alea,
        15.0,
        largeur_mm - 15.0,
        12.0,
        hauteur_mm - 12.0,
        d.encre,
    );
    degrader(&img, dpi, d, &mut alea, 0)
}

/// Encode une image en niveaux de gris en JPEG à `qualite`.
pub(crate) fn jpeg(img: &GrayImage, qualite: u8) -> Vec<u8> {
    use image::ImageEncoder;
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, qualite)
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::L8,
        )
        .expect("encodage JPEG");
    out
}

/// Octets JPEG du scan d'une QR-facture à `dpi` — ceux des fixtures `qr-scan-*`.
pub(crate) fn jpeg_qr_facture(dpi: f32) -> Vec<u8> {
    jpeg(&page_qr_facture(dpi, DEGRADATION), QUALITE_JPEG)
}

/// Une page d'un PDF image : octets JPEG, dimensions en pixels et en millimètres.
pub(crate) struct PageImage {
    pub jpeg: Vec<u8>,
    pub largeur_px: u32,
    pub hauteur_px: u32,
    pub largeur_mm: f32,
    pub hauteur_mm: f32,
}

impl PageImage {
    /// Page JPEG couvrant `largeur_mm × hauteur_mm`.
    pub(crate) fn new(img: &GrayImage, qualite: u8, largeur_mm: f32, hauteur_mm: f32) -> Self {
        Self {
            jpeg: jpeg(img, qualite),
            largeur_px: img.width(),
            hauteur_px: img.height(),
            largeur_mm,
            hauteur_mm,
        }
    }
}

/// PDF dont chaque page est une image JPEG (filtre DCT) couvrant toute la page.
pub(crate) fn pdf_images(pages: &[PageImage]) -> Vec<u8> {
    use printpdf::{
        ColorBits, ColorSpace, Image, ImageFilter, ImageTransform, ImageXObject, Mm, PdfDocument,
        Px,
    };
    let premiere = pages.first().expect("au moins une page");
    let (doc, p1, l1) = PdfDocument::new(
        "Scan synthetique (Kesh, tests)",
        Mm(premiere.largeur_mm),
        Mm(premiere.hauteur_mm),
        "scan",
    );
    for (i, page) in pages.iter().enumerate() {
        let (p, l) = if i == 0 {
            (p1, l1)
        } else {
            doc.add_page(Mm(page.largeur_mm), Mm(page.hauteur_mm), "scan")
        };
        let calque = doc.get_page(p).get_layer(l);
        let xobj = ImageXObject {
            width: Px(page.largeur_px as usize),
            height: Px(page.hauteur_px as usize),
            color_space: ColorSpace::Greyscale,
            bits_per_component: ColorBits::Bit8,
            interpolate: false,
            image_data: page.jpeg.clone(),
            image_filter: Some(ImageFilter::DCT),
            smask: None,
            clipping_bbox: None,
        };
        // dpi tel que l'image couvre exactement la largeur de la page.
        let dpi = page.largeur_px as f32 / (page.largeur_mm / 25.4);
        Image::from(xobj).add_to_layer(
            calque,
            ImageTransform {
                dpi: Some(dpi),
                ..Default::default()
            },
        );
    }
    doc.save_to_bytes().expect("PDF synthétique")
}

/// PDF d'une page A4 portant le scan de la QR-facture à `dpi`.
pub(crate) fn pdf_qr_scan(dpi: f32, d: Degradation, qualite: u8) -> Vec<u8> {
    pdf_images(&[PageImage::new(
        &page_qr_facture(dpi, d),
        qualite,
        210.0,
        297.0,
    )])
}

/// PDF d'une page A4 scannée sans QR, à `dpi`.
pub(crate) fn pdf_sans_qr(dpi: f32, d: Degradation, qualite: u8) -> Vec<u8> {
    pdf_images(&[PageImage::new(
        &page_sans_qr(210.0, 297.0, dpi, d),
        qualite,
        210.0,
        297.0,
    )])
}

/// PDF **vectoriel** de `pages` pages A4 dont la dernière porte, dessinés en
/// rectangles pleins, les modules du QR de `payload` (46 mm) — une facture née
/// numérique dont la QR-facture est en dernière page (banc, D4).
pub(crate) fn pdf_vectoriel(pages: usize, payload: &str) -> Vec<u8> {
    use printpdf::{Color, Mm, PdfDocument, Rect, Rgb, path::PaintMode};
    let (doc, p1, l1) = PdfDocument::new(
        "Facture synthetique (Kesh, tests)",
        Mm(210.0),
        Mm(297.0),
        "f",
    );
    let mut derniere = (p1, l1);
    for _ in 1..pages {
        derniere = doc.add_page(Mm(210.0), Mm(297.0), "f");
    }
    let calque = doc.get_page(derniere.0).get_layer(derniere.1);
    let qr = render_qr_image(payload).expect("QR vectoriel");
    let module = 46.0 / qr.size() as f32;
    let (x0, haut) = (67.0_f32, 297.0_f32 - 209.0);
    calque.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                let llx = x0 + x as f32 * module;
                let ury = haut - y as f32 * module;
                calque.add_rect(
                    Rect::new(Mm(llx), Mm(ury - module), Mm(llx + module), Mm(ury))
                        .with_mode(PaintMode::Fill),
                );
            }
        }
    }
    doc.save_to_bytes().expect("PDF vectoriel")
}

/// Chemin absolu d'une fixture, fixé à la compilation (indépendant du répertoire
/// courant).
pub(crate) fn chemin_fixture(nom: &str) -> String {
    format!("{}/tests/fixtures/{nom}", env!("CARGO_MANIFEST_DIR"))
}

/// Lit une fixture versionnée à l'exécution.
pub(crate) fn lire_fixture(nom: &str) -> Vec<u8> {
    std::fs::read(chemin_fixture(nom)).unwrap_or_else(|e| panic!("fixture {nom} : {e}"))
}

/// Le flux JPEG (filtre DCT, sans autre compression) d'une fixture versionnée, extrait
/// du PDF sans pdfium : du marqueur SOI (`FF D8 FF`) au dernier EOI (`FF D9`). Chaque
/// fixture porte une seule image (revue de code P1, B-3).
pub(crate) fn jpeg_de_fixture(nom: &str) -> Vec<u8> {
    let pdf = lire_fixture(nom);
    let debut = pdf
        .windows(3)
        .position(|w| w == [0xFF, 0xD8, 0xFF])
        .unwrap_or_else(|| panic!("{nom} : aucun flux JPEG"));
    let fin = pdf
        .windows(2)
        .rposition(|w| w == [0xFF, 0xD9])
        .unwrap_or_else(|| panic!("{nom} : flux JPEG sans fin"));
    pdf[debut..fin + 2].to_vec()
}

/// Nom des trois fixtures à QR, par résolution de scan.
pub(crate) const FIXTURES_QR: [(&str, f32); 3] = [
    ("qr-scan-150dpi.pdf", 150.0),
    ("qr-scan-200dpi.pdf", 200.0),
    ("qr-scan-300dpi.pdf", 300.0),
];
/// Nom de la fixture sans QR.
pub(crate) const FIXTURE_SANS_QR: &str = "scan-sans-qr.pdf";

#[cfg(test)]
mod tests {
    use super::*;

    /// Plafond de taille d'une fixture versionnée (AC-1).
    const PLAFOND_FIXTURE: usize = 1024 * 1024;

    /// Le payload fait au moins 280 octets UTF-8 et donne un QR de version 12 à 15
    /// au niveau M (au-delà de 15, le second rendu tomberait sous 4 px par module).
    #[test]
    fn payload_synthetique_version_12_a_15() {
        let p = payload();
        assert!(p.len() >= 280, "payload de {} octets", p.len());
        let qr = render_qr_image(&p).unwrap();
        let v = qr.version().value();
        assert!((12..=15).contains(&v), "version {v}");
    }

    /// Produit les quatre fixtures versionnées (à lancer à la main, cf. en-tête).
    #[test]
    #[ignore = "générateur des fixtures versionnées — à lancer à la main"]
    fn ecrire_fixtures() {
        // Revue de code P1, A1 : les quatre PDF sont engendrés et leur taille
        // contrôlée AVANT d'en écrire un seul — un dépassement laisse le jeu de
        // fixtures entier inchangé, au lieu d'en mêler deux générations.
        let mut jeu: Vec<(&str, Vec<u8>)> = FIXTURES_QR
            .iter()
            .map(|(nom, dpi)| (*nom, pdf_qr_scan(*dpi, DEGRADATION, QUALITE_JPEG)))
            .collect();
        jeu.push((
            FIXTURE_SANS_QR,
            pdf_sans_qr(DPI_SANS_QR, DEGRADATION, QUALITE_JPEG_SANS_QR),
        ));
        for (nom, pdf) in &jeu {
            eprintln!("{nom} : {} octets", pdf.len());
        }
        let trop_gros: Vec<&str> = jeu
            .iter()
            .filter(|(_, pdf)| pdf.len() > PLAFOND_FIXTURE)
            .map(|(nom, _)| *nom)
            .collect();
        assert!(
            trop_gros.is_empty(),
            "dépassent 1 Mio, rien n'est écrit : {trop_gros:?}"
        );
        for (nom, pdf) in &jeu {
            std::fs::write(chemin_fixture(nom), pdf).unwrap();
        }
    }

    /// Les fixtures versionnées tiennent sous le plafond de 1 Mio.
    #[test]
    fn fixtures_sous_le_plafond() {
        for nom in FIXTURES_QR.iter().map(|(n, _)| *n).chain([FIXTURE_SANS_QR]) {
            let n = lire_fixture(nom).len();
            assert!(n <= PLAFOND_FIXTURE, "{nom} : {n} octets");
        }
    }
}
