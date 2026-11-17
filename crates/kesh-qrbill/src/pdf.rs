//! PDF generation for a Swiss QR Bill invoice (A4 portrait).
//!
//! Layout:
//! - Top: invoice header + lines table + total.
//! - Separator line at y = 105 mm (SIX §5: payment section occupies bottom 105 mm).
//! - Receipt column (62 mm wide) + Payment Part column (remaining width), separated by a
//!   vertical line at x = 62 mm ; each of the two dotted lines bears a scissors symbol
//!   drawn in strokes (SIX § 3.7, Story 36-12a). The "Acceptance point" heading is
//!   right-aligned at x = 57 mm (§ 3.6.4), measured by the private module `largeurs`.
//! - The payment section follows the SIX grid (Story 44-1a, #944): titles at baseline
//!   96.5 mm, QR code 46×46 mm from (67, 42) to (113, 88) with an 8×8 mm white square and a
//!   7×7 mm red Swiss cross centred at (90, 65), information column at x = 118 mm with
//!   *Payable by* in it, amount section at baseline 35 mm on both sides, acceptance point
//!   at baseline 20.5 mm. Text is wrapped at measured widths (52 mm on the receipt, 87 mm
//!   in the column); a very long address falls back to fixed-height steps (street omitted
//!   on the receipt, then lines shortened with "..."): the layout never fails.
//!
//! Uses `BuiltinFont::Helvetica` (PDF standard 14) — no external font embedding.

use crate::generator::{build_payload, render_qr_image};
use crate::largeurs::largeur_mm;
use crate::texte::texte_imprime;
use crate::types::{
    Address, AddressType, InvoicePdfData, QrBillData, QrBillError, QrBillI18n, Reference,
    ReminderPdf,
};
use chrono::{Datelike, NaiveDate};
use kesh_core::text::is_invisible;
use printpdf::{
    BuiltinFont, Color, IndirectFontRef, Line, Mm, OffsetDateTime, PdfDocument,
    PdfDocumentReference, PdfLayerReference, Point, Rgb,
};
use qrcodegen::QrCode;
use rust_decimal::{Decimal, RoundingStrategy};

const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const PAYMENT_H: f32 = 105.0;
const RECEIPT_W: f32 = 62.0;
const SEP_Y: f32 = PAYMENT_H; // separator between invoice & payment section
/// Bord droit du texte du récépissé, en mm : sa largeur moins la marge de 5 mm (§ 3.3).
const RECU_BORD_DROIT: f32 = RECEIPT_W - 5.0;

/// Public entry point — generates the PDF using `OffsetDateTime::now_utc()` as the creation date.
pub fn generate_qr_bill_pdf(
    data: &QrBillData,
    invoice: &InvoicePdfData,
    i18n: &QrBillI18n,
) -> Result<Vec<u8>, QrBillError> {
    generate_qr_bill_pdf_with_date(data, invoice, i18n, OffsetDateTime::now_utc())
}

/// Deterministic variant — exposes `creation_date` and uses a fixed document_id so
/// identical inputs yield byte-identical PDFs *modulo* printpdf's internal random
/// instance id (second element of trailer `/ID`). See `tests/pdf_test.rs`.
pub fn generate_qr_bill_pdf_with_date(
    data: &QrBillData,
    invoice: &InvoicePdfData,
    i18n: &QrBillI18n,
    creation_date: OffsetDateTime,
) -> Result<Vec<u8>, QrBillError> {
    // HIGH (review pass 1 G2 C) : cross-check QR vs PDF currency. Sans cette
    // garde, le QR encode `data.currency` (lu par la banque) tandis que le
    // PDF affiche `invoice.currency` (lu par l'humain) — divergence
    // potentiellement légale.
    if data.currency != invoice.currency {
        return Err(QrBillError::PdfGeneration(format!(
            "currency mismatch QR={:?} vs PDF={:?}",
            data.currency, invoice.currency
        )));
    }
    // Build and validate the payload up-front.
    let payload = build_payload(data)?;
    let qr = render_qr_image(&payload)?;
    // M-Edge (review pass 1 G2 C) : QR rendu dans une zone fixe 46 mm. La
    // norme SIX exige des modules ≥ 0.4 mm pour la lisibilité par scanner.
    // Si le payload force un QR très dense, on rejette plutôt que générer
    // un PDF illisible silencieusement.
    let module_mm = 46.0_f32 / qr.size() as f32;
    if module_mm < 0.4 {
        return Err(QrBillError::PdfGeneration(format!(
            "QR module {:.3}mm < 0.4mm — payload trop dense (réduire unstructured_message ou billing_information)",
            module_mm
        )));
    }

    let (doc, page_idx, layer_idx) = PdfDocument::new(
        format!("Invoice {}", invoice.invoice_number),
        Mm(PAGE_W),
        Mm(PAGE_H),
        "Layer 1",
    );
    let doc = doc
        .with_creator("kesh-qrbill")
        .with_producer("kesh-qrbill")
        .with_creation_date(creation_date)
        .with_mod_date(creation_date)
        .with_metadata_date(creation_date)
        .with_document_id(format!("kesh-{}", invoice.invoice_number));

    // Helvetica (regular + bold) — built-in, no external font data.
    let helv = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| QrBillError::PdfGeneration(format!("font: {e}")))?;
    let helv_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| QrBillError::PdfGeneration(format!("font bold: {e}")))?;

    let page = doc.get_page(page_idx);
    let layer = page.get_layer(layer_idx);

    // Facture : le contenu s'arrête au séparateur QR (SEP_Y).
    draw_invoice_section(&layer, invoice, i18n, &helv, &helv_bold, SEP_Y)?;
    draw_separator(&layer);
    dessiner_section_de_paiement(&layer, data, invoice, i18n, &helv, &helv_bold, &qr)?;

    finalize(doc)
}

/// Génère un PDF d'**avoir** (note de crédit) — Story 12.1.
///
/// Identique à la section haute d'une facture (en-tête société, destinataire,
/// lignes, total) mais **SANS section QR Bill** (le paiement irait dans l'autre
/// sens). Le titre/numéro sont fournis via les clés i18n surchargées
/// (`invoice-pdf-title` = « Avoir »…) et la référence à la facture d'origine via
/// `invoice.origin_reference`.
pub fn generate_credit_note_pdf(
    invoice: &InvoicePdfData,
    i18n: &QrBillI18n,
) -> Result<Vec<u8>, QrBillError> {
    generate_credit_note_pdf_with_date(invoice, i18n, OffsetDateTime::now_utc())
}

/// Variante déterministe de [`generate_credit_note_pdf`] (date fixée, tests).
pub fn generate_credit_note_pdf_with_date(
    invoice: &InvoicePdfData,
    i18n: &QrBillI18n,
    creation_date: OffsetDateTime,
) -> Result<Vec<u8>, QrBillError> {
    let (doc, page_idx, layer_idx) = PdfDocument::new(
        format!("Credit note {}", invoice.invoice_number),
        Mm(PAGE_W),
        Mm(PAGE_H),
        "Layer 1",
    );
    let doc = doc
        .with_creator("kesh-qrbill")
        .with_producer("kesh-qrbill")
        .with_creation_date(creation_date)
        .with_mod_date(creation_date)
        .with_metadata_date(creation_date)
        .with_document_id(format!("kesh-cn-{}", invoice.invoice_number));

    let helv = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| QrBillError::PdfGeneration(format!("font: {e}")))?;
    let helv_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| QrBillError::PdfGeneration(format!("font bold: {e}")))?;

    let page = doc.get_page(page_idx);
    let layer = page.get_layer(layer_idx);

    // Avoir : pas de section QR → le contenu peut descendre jusqu'à la marge
    // basse (pleine page), pas seulement jusqu'à SEP_Y. Évite de rejeter un
    // avoir pour une zone QR inexistante (#151 code-review).
    draw_invoice_section(
        &layer,
        invoice,
        i18n,
        &helv,
        &helv_bold,
        CONTENT_FLOOR_NO_QR,
    )?;

    finalize(doc)
}

fn finalize(doc: PdfDocumentReference) -> Result<Vec<u8>, QrBillError> {
    doc.save_to_bytes()
        .map_err(|e| QrBillError::PdfGeneration(format!("save: {e}")))
}

// ----- Invoice top section -----

/// Marge basse d'un document **sans section QR** (avoir) : le contenu peut
/// descendre jusqu'à `CONTENT_FLOOR_NO_QR` mm du bas de page. Pour une facture,
/// le plancher est `SEP_Y` (le séparateur QR).
const CONTENT_FLOOR_NO_QR: f32 = 15.0;

/// Largeur utile d'une ligne du bloc identité de l'émetteur, **en caractères**.
///
/// Le bloc de droite commence à `meta_x = 120.0`, la marge gauche est à `20.0` :
/// **100 mm** disponibles.
///
/// ⚠️ **Helvetica est proportionnelle — ce compte est une approximation, et le
/// calibrage vise le PIRE CAS RAISONNABLE.** Largeurs AFM Adobe, à 9 pt :
///
/// | Contenu | 50 car. | 46 car. |
/// |---|---|---|
/// | minuscules (moy. 490/1000 em) | 77,8 mm | 71,6 mm |
/// | **capitales** (moy. 677/1000 em) | **107,5 mm — déborde** | **98,9 mm** |
///
/// D'où 46 et non 50 : une URL ou un e-mail en capitales dépassait encore sur
/// le bloc de droite **après** troncature. *(Revue de code, passe 3.)*
///
/// ⚠️ La **raison sociale** n'entre pas dans ce calibrage : elle est dessinée à
/// 14 pt, hors du dispositif de troncature. Cette borne ne régit que le bloc
/// identité — IDE et coordonnées de contact. *(Revue de code, passe 6.)*
///
/// ⚠️ **Ce que cette borne ne couvre PAS** : une chaîne de 46 `W` (944/1000 em)
/// occuperait **137,9 mm** — par la méthode du tableau ci-dessus, qui reproduit
/// ses quatre valeurs à la décimale. Il faudrait mesurer la largeur réelle pour le garantir,
/// ce que `printpdf` n'expose pas pour les polices intégrées. Le cas est jugé
/// hors de portée d'un champ de coordonnées ; le documenter vaut mieux que
/// laisser croire à une garantie exacte.
const IDENTITY_MAX_CHARS: usize = 46;

/// Pas vertical entre deux lignes du bloc métadonnées (droite), en mm.
///
/// Extrait en constante parce qu'il est l'**invariant testé** par AC6 : le
/// delta d'ordonnée entre le cas `Some` et le cas `None` du numéro de client
/// vaut exactement ce pas, et la mutation à tuer (`my -= 4.5` sorti du
/// conditionnel) le met à zéro.
const META_LINE_STEP: f32 = 4.5;

/// Borne d'affichage du bloc métadonnées **de droite** (Story 16-3b, #151).
///
/// Le bloc démarre à `meta_x = 120.0` et la page fait `PAGE_W = 210.0` avec une
/// marge droite de 20 : **70 mm** disponibles — *moins* que les 100 mm du bloc
/// gauche, d'où une constante distincte et non un réemploi d'`IDENTITY_MAX_CHARS`.
///
/// Calibrage par la méthode de cette dernière (table AFM Helvetica 9 pt,
/// largeur moyenne des capitales 677/1000 em, 1 pt = 0,3528 mm) :
///
/// ```text
/// 70 / (9 × 0,677 × 0,3528) ≈ 32,6  →  32 caractères
/// ```
///
/// ⚠️ **LA BORNE PORTE SUR LA LIGNE COMPLÈTE `"{libellé}: {valeur}"`**, comme
/// au bloc gauche (`truncate_display(&format!(…), IDENTITY_MAX_CHARS)`). Les
/// 32 caractères sont un budget de **largeur** : ils couvrent nécessairement le
/// libellé. Tronquer la seule valeur puis formater le libellé autour produirait
/// une ligne d'environ 43 caractères ≈ 92 mm depuis `x = 120`, soit `x ≈ 212`
/// sur une page de 210 — du texte **hors feuille**.
///
/// ⚠️ **Conséquence à connaître** : « N° client: » (11 caractères) ne laisse que
/// ~21 caractères à la valeur, « Kundennummer: » (14) ~18. Un champ déclaré
/// `VARCHAR(50)` s'imprime donc sur une vingtaine de caractères. C'est un repli
/// d'**affichage** — la valeur reste entière en base et dans la fiche contact.
///
/// ⚠️ **Ce que le test de calibrage ne prouve PAS** : il montre la *cohérence*
/// de la troncature (deux chaînes longues coupées à la même taille), jamais que
/// la valeur *tient* dans la largeur disponible. Une constante trop généreuse y
/// passerait sans être vue. La justesse repose sur le calcul ci-dessus.
const META_MAX_CHARS: usize = 32;

/// Construit les lignes du bloc métadonnées (droite) : texte déjà formaté et
/// tronqué, avec son ordonnée en mm. **Le titre reste hors extraction** — il est
/// dessiné à 18 pt et suit un pas différent.
///
/// **Fonction pure, et c'est la SEULE façon de tester AC6.** Le delta de taille
/// du PDF est aveugle au décalage vertical (`pdf.rs` le documente et le dépôt
/// l'a déjà payé : « **Mesuré** — une première version de ce test comparait
/// deux générations entre elles et restait **verte** sous la mutation »), et la
/// parade de la Story 16-3a — mesurer au seuil d'une garde haute — n'existe pas
/// à droite, la garde ne surveillant que la colonne gauche. Sans cette
/// extraction, la conditionnalité du décrément serait un critère qu'aucun test
/// ne peut faire échouer.
///
/// `first_line_y` est l'ordonnée de la **première** ligne ; chaque ligne
/// suivante descend de `META_LINE_STEP`. Une ligne absente ne consomme aucun
/// espace : c'est exactement l'invariant qu'AC6 vérifie.
///
/// ⚠️ **La troncature ne s'applique qu'au numéro de client**, et le motif n'est
/// PAS que les autres lignes seraient incapables de déborder.
///
/// Le motif est qu'une borne à `META_MAX_CHARS` **couperait du contenu
/// existant** : « Réf. facture d'origine: FA-2026-0001 » fait 35 caractères et
/// serait tronqué sur tout avoir français. La borne est donc réservée au champ
/// que cette story introduit, et dont elle répond.
///
/// ⚠️ **Le débordement des autres lignes est un problème réel, distinct, et
/// ANTÉRIEUR à cette story — suivi dans l'issue #293.** Le numéro de facture
/// n'est pas « borné par le schéma de numérotation » : ce schéma est un champ
/// libre de l'écran *Administration → Facturation*, et `invoice_format` le laisse
/// rendre jusqu'à `MAX_RENDERED_LEN = 64` caractères — soit plus que les 50 du
/// numéro de client. `origin_reference` est un numéro de facture, donc de même
/// classe. Le traiter demande une borne PROPRE à ces lignes, plus haute, et un
/// arbitrage sur ce qu'on coupe ; ce n'est pas le périmètre de la 16-3b.
///
/// *(La rédaction précédente affirmait que ces valeurs étaient « engendrées par
/// le système, bornées par le schéma de numérotation ». Prémisse réfutée en
/// passe 1 de `bmad-code-review`, `invoice_format.rs:25-27` à l'appui.)*
fn build_meta_lines(
    inv: &InvoicePdfData,
    i18n: &QrBillI18n,
    first_line_y: f32,
) -> Vec<(String, f32)> {
    let mut out: Vec<(String, f32)> = Vec::new();
    let mut y = first_line_y;
    let push = |out: &mut Vec<(String, f32)>,
                y: &mut f32,
                key: &'static str,
                value: &str,
                bounded: bool| {
        if !out.is_empty() {
            *y -= META_LINE_STEP;
        }
        // ⚠️ La borne porte sur la LIGNE COMPLÈTE `"{libellé}: {valeur}"`, jamais
        // sur la seule valeur : les 32 caractères sont un budget de LARGEUR, et
        // couvrent donc nécessairement le libellé.
        let line = format!("{}: {}", i18n.get(key), value);
        out.push((
            if bounded {
                truncate_display(&line, META_MAX_CHARS)
            } else {
                line
            },
            *y,
        ));
    };

    push(
        &mut out,
        &mut y,
        "invoice-pdf-number",
        &inv.invoice_number,
        false,
    );
    // Story 16-3b (#151) : entre le n° de facture et la date — position fixée
    // par D3, et qu'aucun test de delta ne pourrait rattraper si elle changeait.
    //
    // ⚠️ Tester la VACUITÉ, pas la nullité — même raison qu'au bloc gauche de la
    // Story 16-3a, dont ce site est le jumeau. La normalisation qui transforme
    // `""` en `None` vit dans la route API ; une valeur vide arrivée autrement
    // imprimerait « N° client: » suivi de rien, en consommant un
    // `META_LINE_STEP` qui décalerait vers le bas la date, la référence
    // d'origine et l'échéance.
    //
    // Et le chemin n'est PAS que théorique ici : `str::trim` suit la propriété
    // Unicode `White_Space`, qui **n'inclut pas** les caractères de largeur
    // nulle. `U+200B` (ZWSP), `U+FEFF` (BOM) et `U+2060` (WJ) survivent donc à
    // la normalisation de la route et arrivent jusqu'ici. `trim().is_empty()`
    // ne les attrape pas non plus — d'où le filtre sur les caractères qui
    // MARQUENT quelque chose. *(Passe 1 de `bmad-code-review`.)*
    if let Some(client_number) = inv
        .debtor_client_number
        .as_deref()
        .map(str::trim)
        .filter(|v| v.chars().any(|c| !is_invisible(c)))
    {
        push(
            &mut out,
            &mut y,
            "invoice-pdf-client-number",
            client_number,
            true,
        );
    }
    push(
        &mut out,
        &mut y,
        "invoice-pdf-date",
        &format_date_ch(inv.invoice_date),
        false,
    );
    // Référence à la facture d'origine (avoirs uniquement, Story 12.1).
    if let Some(origin) = &inv.origin_reference {
        push(
            &mut out,
            &mut y,
            "invoice-pdf-origin-reference",
            origin,
            false,
        );
    }
    if let Some(due) = inv.due_date {
        push(
            &mut out,
            &mut y,
            "invoice-pdf-due-date",
            &format_date_ch(due),
            false,
        );
    }
    out
}

/// Dessine la section haute (en-tête + lignes + récap TVA + total).
///
/// `content_floor` = ordonnée (mm) sous laquelle le contenu ne doit PAS
/// descendre : `SEP_Y` pour une facture (la section QR occupe le bas), une
/// simple marge basse pour un avoir (aucune section QR → pleine page). La garde
/// de capacité en tient compte, ce qui évite de pénaliser un avoir pour une
/// zone QR inexistante (#151 code-review).
///
/// *(Ce doc-comment était devenu orphelin : l'extraction de `build_meta_lines`
/// en T9 s'était insérée ENTRE lui et sa fonction, si bien que rustdoc
/// l'attribuait au mauvais symbole et décrivait un paramètre `content_floor`
/// que `build_meta_lines` ne prend pas. Rendu à sa fonction en passe 3 de
/// `bmad-code-review` ; deux passes avaient lu ce hunk sans le voir.)*
fn draw_invoice_section(
    layer: &PdfLayerReference,
    inv: &InvoicePdfData,
    i18n: &QrBillI18n,
    helv: &IndirectFontRef,
    helv_bold: &IndirectFontRef,
    content_floor: f32,
) -> Result<(), QrBillError> {
    let left = 20.0;
    let mut y = PAGE_H - 20.0;

    // Creditor header (top-left). Le nom et l'adresse passent par `texte_imprime` : le
    // § 4.1.1 admet U+00A0, U+00AD et U+0218 à U+021B, que `lopdf` perdrait ici en
    // silence, sur une pièce qui se fige (Story 33-12a). Un avoir et un rappel passent par
    // le même chemin : U+2019 et U+2014 y deviennent `'` et `-` comme au code, et les 125
    // lettres leur base ; tout autre caractère hors du jeu y reste traité comme avant.
    layer.use_text(
        texte_imprime(&inv.creditor_name),
        14.0,
        Mm(left),
        Mm(y),
        helv_bold,
    );
    y -= 5.0;
    for line in &inv.creditor_address_lines {
        layer.use_text(texte_imprime(line), 9.0, Mm(left), Mm(y), helv);
        y -= 4.0;
    }
    // Bloc « identité » de l'émetteur : IDE puis coordonnées de contact
    // (Story 16-3a, #151). Rendu **conditionnel** ligne par ligne — une valeur
    // absente ne dessine rien et ne descend pas le curseur.
    //
    // ⚠️ Le pas de 4 mm et la respiration de 2 mm sont **séparés à dessein**, et
    // la respiration n'est posée que si AU MOINS une ligne a été dessinée :
    // - IDE seul, sans coordonnées → 4 + 2 = **6 mm**, exactement l'ancien pas ;
    // - ni IDE ni coordonnées → **0 mm**, exactement l'ancien comportement.
    //
    // C'est ce qui tient **D2** : une société qui ne renseigne rien produit le
    // PDF d'avant cette story, à l'octet près sur le bloc haut. Rendre la
    // respiration inconditionnelle décalerait tout le document de ces sociétés.
    let mut identity_lines = 0;
    for (key, value) in [
        ("invoice-pdf-ide", &inv.creditor_ide),
        ("invoice-pdf-phone", &inv.creditor_phone),
        ("invoice-pdf-email", &inv.creditor_email),
        ("invoice-pdf-website", &inv.creditor_website),
    ] {
        // ⚠️ Tester la VACUITÉ, pas la nullité : la normalisation qui transforme
        // `""` en `None` vit dans la route API. Une chaîne vide arrivée
        // autrement — restauration d'une sauvegarde produite ailleurs,
        // correction SQL directe — imprimerait « Tél.: » suivi de rien, en
        // consommant 4 mm du budget de la garde. *(Passe 3 de revue.)*
        if let Some(v) = value.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            // ⚠️ Troncature de LARGEUR — la garde de capacité plus bas ne
            // surveille que l'ordonnée `y`, donc l'empilement VERTICAL. Elle ne
            // voit rien d'une ligne unique trop longue.
            //
            // Le bloc de droite (« Facture », n°, date) démarre à `meta_x`
            // (120 mm) et la marge gauche est à 20 mm : 100 mm disponibles, d'où
            // `IDENTITY_MAX_CHARS` — **46** caractères à 9 pt, et non 50, cf. le
            // tableau de calibrage de la constante, qui montre que 50 capitales
            // débordent encore. Sans cette troncature, un site web de
            // 255 caractères ou un e-mail de 320 (`VARCHAR(320)`, jamais borné en
            // longueur à la saisie) s'imprimait par-dessus le bloc de droite,
            // voire hors page, **rendu en 200**.
            //
            // Tronquer plutôt que refuser : la valeur reste lisible dans les
            // réglages, et refuser une facture pour un champ décoratif serait
            // disproportionné — à la différence du débordement vertical, qui
            // rend le tableau des lignes illisible.
            layer.use_text(
                truncate_display(&format!("{}: {}", i18n.get(key), v), IDENTITY_MAX_CHARS),
                9.0,
                Mm(left),
                Mm(y),
                helv,
            );
            y -= 4.0;
            identity_lines += 1;
        }
    }
    if identity_lines > 0 {
        y -= 2.0;
    }

    // Title + metadata (right).
    let meta_x = 120.0;
    let meta_title_y = PAGE_H - 20.0;
    // Story 25-4-b2 (#416) : un rappel n'est pas la facture réémise — il le dit
    // en titre. La clé est résolue dans la locale que l'appelant a passée.
    let title_key = if inv.reminder.is_some() {
        "invoice-pdf-reminder-title"
    } else {
        "invoice-pdf-title"
    };
    layer.use_text(
        i18n.get(title_key),
        18.0,
        Mm(meta_x),
        Mm(meta_title_y),
        helv_bold,
    );
    // Les lignes du bloc sont construites par une fonction PURE, puis
    // seulement dessinées : c'est ce qui rend leur position testable.
    //
    // Le curseur du bloc droit n'est plus suivi ici : `build_meta_lines` porte
    // désormais toute la chaîne de décréments, et rien en aval ne s'en sert —
    // la colonne droite dispose de ~90 mm de marge (le tableau démarre à
    // `PAGE_H - 130`), et seule la colonne GAUCHE est surveillée par la garde
    // de capacité haute.
    let meta_first_line_y = meta_title_y - 7.0;
    for (text, line_y) in build_meta_lines(inv, i18n, meta_first_line_y) {
        layer.use_text(text, 9.0, Mm(meta_x), Mm(line_y), helv);
    }

    // Recipient (below creditor, left).
    y = y.min(PAGE_H - 55.0);
    layer.use_text(
        i18n.get("invoice-pdf-recipient"),
        9.0,
        Mm(left),
        Mm(y),
        helv_bold,
    );
    y -= 5.0;
    layer.use_text(texte_imprime(&inv.debtor_name), 10.0, Mm(left), Mm(y), helv);
    y -= 4.5;
    for line in &inv.debtor_address_lines {
        layer.use_text(texte_imprime(line), 9.0, Mm(left), Mm(y), helv);
        y -= 4.0;
    }

    // Lines table.
    let mut ty = PAGE_H - 130.0;

    // ⚠️ GARDE DE CAPACITÉ HAUTE (Story 16-3a, #151) — symétrique de
    // `TooManyLines`, qui ne surveille que le plancher QR.
    //
    // `ty` est une **constante** : le tableau n'est JAMAIS repoussé par ce qui
    // le précède. Un en-tête trop haut — adresse émetteur longue, trois
    // coordonnées renseignées, adresse destinataire longue — descendait donc
    // sur les en-têtes de colonnes **sans que rien ne le détecte**, produisant
    // un document illisible rendu en 200.
    //
    // Le `y.min(PAGE_H - 55.0)` plus haut est un **plafond**, pas un plancher :
    // il empêche le destinataire de remonter, jamais de descendre.
    //
    // Refuser proprement plutôt que superposer, comme le fait déjà la garde
    // basse.
    //
    // ⚠️ Les `+ 2.0` ne sont PAS l'écart réel entre la dernière ligne et le
    // tableau : au moment du test, `y` est la position LIBRE suivante, déjà 4 mm
    // sous la ligne de base du dernier texte dessiné. Au seuil exact, cette
    // ligne est donc à `ty + 6`, pas `ty + 2`. Le garde est plus conservateur
    // qu'il n'y paraît — qui recalibre sur ce chiffre se trompe d'un facteur
    // trois. *(Précision apportée en passe 3 de revue.)*
    if y < ty + 2.0 {
        return Err(QrBillError::HeaderOverflow(y));
    }
    let col_desc = left;
    let col_qty = left + 90.0;
    let col_unit = left + 110.0;
    let col_vat = left + 140.0;
    let col_tot = left + 160.0;

    layer.use_text(
        i18n.get("invoice-pdf-description"),
        9.0,
        Mm(col_desc),
        Mm(ty),
        helv_bold,
    );
    layer.use_text(
        i18n.get("invoice-pdf-quantity"),
        9.0,
        Mm(col_qty),
        Mm(ty),
        helv_bold,
    );
    layer.use_text(
        i18n.get("invoice-pdf-unit-price"),
        9.0,
        Mm(col_unit),
        Mm(ty),
        helv_bold,
    );
    layer.use_text(
        i18n.get("invoice-pdf-vat"),
        9.0,
        Mm(col_vat),
        Mm(ty),
        helv_bold,
    );
    layer.use_text(
        i18n.get("invoice-pdf-line-total"),
        9.0,
        Mm(col_tot),
        Mm(ty),
        helv_bold,
    );
    ty -= 5.0;
    // Header underline.
    hline(layer, left, PAGE_W - 20.0, ty);
    ty -= 3.0;

    // #151 : le bloc récap TVA (sous-total + 1 ligne par taux + espace) est
    // dessiné APRÈS la boucle et descend le curseur. On réserve sa hauteur dans
    // le seuil de la garde ci-dessous pour refuser une facture dont lignes +
    // récap ne tiennent pas au-dessus du séparateur QR (`SEP_Y`), plutôt que de
    // laisser le récap chevaucher la zone de paiement. `+15` couvrait déjà le
    // total seul ; on ajoute `sous-total + n×taux + espace` (chaque ligne = 4.5).
    //
    // Story 25-4-c4-b : une ligne « Arrondi » s'ajoute quand l'écart n'est pas nul
    // au centime, et fait apparaître le sous-total même sans TVA (`recap_lines`).
    // La réserve et le dessin lisent la MÊME liste (`recap_lines`) : ils ne
    // peuvent pas diverger.
    let recap = recap_lines(inv, i18n);
    let recap_reserve = if recap.is_empty() {
        0.0
    } else {
        4.5 * recap.len() as f32 + 1.0
    } + reminder_block_height(inv.reminder.as_ref());

    for line in &inv.lines {
        if ty < content_floor + 15.0 + recap_reserve {
            // Défense — capacité visuelle dépassée (lignes + récap TVA). On refuse
            // proprement plutôt que tronquer ou chevaucher la zone sous le plancher
            // (QR pour une facture). Erreur dédiée `TooManyLines` → le handler la
            // mappe en 400 « trop de lignes » actionnable (et non un 500 opaque).
            return Err(QrBillError::TooManyLines(inv.lines.len()));
        }
        layer.use_text(
            truncate_display(&line.description, 45),
            9.0,
            Mm(col_desc),
            Mm(ty),
            helv,
        );
        layer.use_text(format_ch(line.quantity, 2), 9.0, Mm(col_qty), Mm(ty), helv);
        layer.use_text(
            format_ch(line.unit_price, 2),
            9.0,
            Mm(col_unit),
            Mm(ty),
            helv,
        );
        layer.use_text(
            format!("{}%", format_ch(line.vat_rate, 2)),
            9.0,
            Mm(col_vat),
            Mm(ty),
            helv,
        );
        layer.use_text(
            format_ch(line.line_total, 2),
            9.0,
            Mm(col_tot),
            Mm(ty),
            helv,
        );
        ty -= 5.0;
    }

    // Total + récapitulatif TVA (#151, obligation LTVA art. 26).
    ty -= 2.0;
    hline(layer, col_unit, PAGE_W - 20.0, ty);
    ty -= 5.0;

    // Bloc récap s'il existe des lignes taxées OU un arrondi : Sous-total HT, une
    // ligne « TVA {taux}% » par taux, puis la ligne « Arrondi » (Story 25-4-c4-b).
    // Sinon (société non assujettie, total déjà rond) on n'affiche que le total —
    // comportement rétro-compatible.
    for line in &recap {
        layer.use_text(&line.label, 9.0, Mm(col_unit), Mm(ty), helv);
        layer.use_text(&line.amount, 9.0, Mm(col_tot), Mm(ty), helv);
        ty -= 4.5;
    }
    if !recap.is_empty() {
        ty -= 1.0; // léger espace avant le total en gras
    }

    let total = inv
        .total
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    layer.use_text(
        i18n.get("invoice-pdf-total-ttc"),
        10.0,
        Mm(col_unit),
        Mm(ty),
        helv_bold,
    );
    layer.use_text(
        format!("{} {}", inv.currency.code(), format_ch(total, 2)),
        10.0,
        Mm(col_tot),
        Mm(ty),
        helv_bold,
    );

    // Story 25-4-b2 (#416) — le bloc du rappel, sous le total. Sa hauteur est
    // RÉSERVÉE dans `recap_reserve` (garde `TooManyLines`) : ⛔ ne pas le traiter
    // comme `payment_terms` ci-dessous, qui se contente de clamper et tasserait
    // le bloc sur la zone de paiement au lieu de refuser.
    if let Some(r) = &inv.reminder {
        for line in reminder_lines(r, i18n, inv.currency.code()) {
            ty -= REMINDER_LINE_STEP;
            let x = if line.full_width { col_desc } else { col_unit };
            let font = if line.bold { helv_bold } else { helv };
            let label = if line.full_width {
                truncate_display(&line.label, REMINDER_NOTE_MAX_CHARS)
            } else {
                truncate_display(&line.label, REMINDER_LABEL_MAX_CHARS)
            };
            layer.use_text(label, 9.0, Mm(x), Mm(ty), font);
            if let Some(amount) = line.amount {
                layer.use_text(amount, 9.0, Mm(col_tot), Mm(ty), font);
            }
        }
    }

    if let Some(terms) = &inv.payment_terms {
        ty -= 8.0;
        layer.use_text(
            format!("{}: {}", i18n.get("invoice-pdf-payment-terms"), terms),
            9.0,
            Mm(col_desc),
            Mm(ty.max(content_floor + 5.0)),
            helv,
        );
    }

    Ok(())
}

// ----- Lignes de séparation et ciseaux (SIX § 3.7) -----
//
// La directive (`docs/six-references/ig-qr-bill-v2.4-en.pdf`, § 3.7) : une QR-facture en
// PDF indique par des lignes le format A6 de la partie paiement et le récépissé à sa
// gauche, et chacune de ces lignes porte le symbole des ciseaux. Story 36-12a (#944).

/// Épaisseur du trait des lignes et des ciseaux, en points (celle d'avant la 36-12a).
const SEP_EPAISSEUR: f32 = 0.3;

/// Longueur d'un trait du pointillé, en mm (motif d'avant la 36-12a).
const SEP_TRAIT: f32 = 2.0;

/// Pas du pointillé, en mm : un trait de 2 mm tous les 4 mm.
const SEP_PAS: f32 = 4.0;

/// Écart entre le pointillé et la boîte d'un ciseau, de chaque côté, en mm (C-36-12a-1).
const SEP_ECART_CISEAUX: f32 = 0.25;

/// Abscisse de l'origine du repère du ciseau de la ligne horizontale, en mm (les lames se
/// croisent 0,34 mm plus loin, vers leurs pointes) : sa boîte va de 46,15 à
/// 50, à droite des titres du récépissé (bord droit ≤ 36,05 dans les quatre langues),
/// à gauche de la ligne verticale (fiche 36-12a, D2 : centre x ∈ [45, 52]).
const CISEAU_H_X: f32 = 48.0;

/// Ordonnée de l'origine du repère du ciseau de la ligne verticale, en mm : sa boîte va de y = 99,5 à
/// 103,35, au-dessus de la zone de 5 mm du QR (qui monte à 93 depuis la Story 44-1a, à 98
/// avant, § 3.5.2) et sous la ligne
/// horizontale (D2, AC3 : centre y ∈ [101 ; 102]).
const CISEAU_V_Y: f32 = 101.5;

// Le symbole, dans son repère propre : `u` le long de la ligne, lames vers les `u`
// positifs ; `v` en travers. Origine en (0, 0) ; les lames se croisent sur l'axe en
// u ≈ 0,34 (revue P1, B-1). Grande dimension 3,85 mm (u de −1,85 à 2),
// petite 2,7 mm (v de −1,35 à 1,35) : la fiche borne la grande de 3 à 5 mm, la petite à
// 3,5 mm (AC3).

/// Centre des deux anneaux, le long de la ligne, en mm.
const ANNEAU_U: f32 = -1.25;
/// Écart des centres des anneaux à la ligne, de part et d'autre, en mm.
const ANNEAU_V: f32 = 0.75;
/// Rayon des anneaux, en mm.
const ANNEAU_R: f32 = 0.6;
/// Pointe des lames, le long de la ligne, en mm.
const LAME_POINTE_U: f32 = 2.0;
/// Écart de la pointe d'une lame à la ligne, du côté opposé à son anneau, en mm.
const LAME_POINTE_V: f32 = 0.42;
/// Les deux sommets du talon d'une lame, près de son anneau, `(u, v)` en mm, côté `v` > 0.
const LAME_TALON: [(f32, f32); 2] = [(-0.85, 0.30), (-0.70, 0.55)];
/// Facteur des points de contrôle d'un quart de cercle en Bézier cubique, rapporté au rayon.
const BEZIER_CERCLE: f32 = 0.552_284_7;

/// L'étendue d'un ciseau le long de sa ligne, dans son repère : de l'anneau à la pointe.
const CISEAU_U: (f32, f32) = (ANNEAU_U - ANNEAU_R, LAME_POINTE_U);

/// Les deux lignes de séparation et leurs ciseaux : l'horizontale à y = 105 sur la largeur
/// de la page, la verticale à x = 62 du bas de la page à l'horizontale (§ 3.3 : partie
/// paiement de 148 × 105 mm, récépissé de 62 × 105 à sa gauche).
///
/// Chaque ligne s'interrompt sous son ciseau : le pointillé s'arrête à
/// [`SEP_ECART_CISEAUX`] de la boîte du symbole et repart à la même distance de l'autre
/// côté, ses traits partant de ces deux points vers l'extérieur (C-36-12a-1).
///
/// Tout est au trait, en noir (`RG`) : aucun remplissage, qui poserait une couleur de
/// remplissage (`rg`) avant celles de la croix suisse.
fn draw_separator(layer: &PdfLayerReference) {
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(SEP_EPAISSEUR);

    // Horizontale : pointillé de part et d'autre du ciseau, de x = 0 à la largeur de page.
    let (avant, apres) = (
        CISEAU_H_X + CISEAU_U.0 - SEP_ECART_CISEAUX,
        CISEAU_H_X + CISEAU_U.1 + SEP_ECART_CISEAUX,
    );
    for (a, b) in traits_du_pointille(0.0, avant, true)
        .into_iter()
        .chain(traits_du_pointille(apres, PAGE_W, false))
    {
        layer.add_line(trait_droit((a, SEP_Y), (b, SEP_Y)));
    }
    dessiner_ciseaux(layer, CISEAU_H_X, SEP_Y, false);

    // Verticale : le ciseau tourné, lames vers le bas, donc l'anneau en haut.
    let (dessous, dessus) = (
        CISEAU_V_Y - CISEAU_U.1 - SEP_ECART_CISEAUX,
        CISEAU_V_Y - CISEAU_U.0 + SEP_ECART_CISEAUX,
    );
    for (a, b) in traits_du_pointille(0.0, dessous, true)
        .into_iter()
        .chain(traits_du_pointille(dessus, SEP_Y, false))
    {
        layer.add_line(trait_droit((RECEIPT_W, a), (RECEIPT_W, b)));
    }
    dessiner_ciseaux(layer, RECEIPT_W, CISEAU_V_Y, true);
}

/// Les traits d'un pointillé sur l'intervalle `[debut, fin]` : un trait de [`SEP_TRAIT`]
/// tous les [`SEP_PAS`], partant de `fin` vers `debut` si `depuis_la_fin`, de `debut` vers
/// `fin` sinon ; le dernier est écourté au bord de l'intervalle.
fn traits_du_pointille(debut: f32, fin: f32, depuis_la_fin: bool) -> Vec<(f32, f32)> {
    let mut traits = Vec::new();
    let mut decalage = 0.0_f32;
    while decalage < fin - debut {
        let (a, b) = if depuis_la_fin {
            (fin - decalage - SEP_TRAIT, fin - decalage)
        } else {
            (debut + decalage, debut + decalage + SEP_TRAIT)
        };
        let (a, b) = (a.max(debut), b.min(fin));
        if b > a {
            traits.push((a, b));
        }
        decalage += SEP_PAS;
    }
    traits
}

/// Un trait droit ouvert, de `de` à `a`, en mm.
fn trait_droit(de: (f32, f32), a: (f32, f32)) -> Line {
    Line {
        points: vec![
            (Point::new(Mm(de.0), Mm(de.1)), false),
            (Point::new(Mm(a.0), Mm(a.1)), false),
        ],
        is_closed: false,
    }
}

/// Le symbole des ciseaux, dessiné au trait (§ 3.7 : « scissors symbol » ; Helvetica en
/// WinAnsi ne porte pas ✂, C-36-12-2) : deux anneaux en Bézier et deux lames en triangles
/// fermés qui se croisent sur la ligne, 0,34 mm après l'origine `(cx, cy)` du repère du
/// symbole, du côté des pointes. Couché, lames vers la droite, sur la ligne
/// horizontale ; tourné d'un quart de tour, lames vers le bas, si `vertical`.
fn dessiner_ciseaux(layer: &PdfLayerReference, cx: f32, cy: f32, vertical: bool) {
    let point = |u: f32, v: f32| -> Point {
        if vertical {
            Point::new(Mm(cx + v), Mm(cy - u))
        } else {
            Point::new(Mm(cx + u), Mm(cy + v))
        }
    };
    for cote in [1.0_f32, -1.0] {
        // L'anneau, quatre quarts de cercle : chaque point est marqué comme appartenant à
        // une courbe, que printpdf écrit en `c`.
        let (au, av, r) = (ANNEAU_U, cote * ANNEAU_V, ANNEAU_R);
        let k = BEZIER_CERCLE * r;
        let anneau = [
            (au + r, av),
            (au + r, av + k),
            (au + k, av + r),
            (au, av + r),
            (au - k, av + r),
            (au - r, av + k),
            (au - r, av),
            (au - r, av - k),
            (au - k, av - r),
            (au, av - r),
            (au + k, av - r),
            (au + r, av - k),
            (au + r, av),
        ];
        layer.add_line(Line {
            points: anneau.iter().map(|&(u, v)| (point(u, v), true)).collect(),
            is_closed: true,
        });
        // La lame partie de cet anneau, qui croise l'autre sur la ligne et finit de l'autre
        // côté de celle-ci.
        let [t1, t2] = LAME_TALON;
        layer.add_line(Line {
            points: vec![
                (point(t1.0, cote * t1.1), false),
                (point(LAME_POINTE_U, -cote * LAME_POINTE_V), false),
                (point(t2.0, cote * t2.1), false),
            ],
            is_closed: true,
        });
    }
}

// ----- La grille de la section de paiement (SIX § 3.3 à 3.6.4, Story 44-1a, #944) -----
//
// Repère : mm depuis le bas à gauche de la page ; la section de paiement occupe y ∈ [0, 105].
// Les cotes viennent du texte de la directive (`docs/six-references/ig-qr-bill-v2.4-en.pdf`)
// quand il en donne, de la mesure des échantillons de SIX sinon (C-36-12-3,
// `kesh-gate-logs/e36-12/mesure-echantillon-six-nr1.log`). Le « haut » d'une ligne est sa
// base plus la hauteur des capitales d'Helvetica, son « bas » sa base moins la descendante.

/// Millimètres par point typographique.
const MM_PAR_POINT: f32 = 25.4 / 72.0;
/// Marge de la section de paiement, et espace minimal entre deux de ses sections, en mm
/// (§ 3.3, § 3.5).
const MARGE: f32 = 5.0;
/// Taille des titres « Récépissé » et « Section paiement », en points (§ 3.4).
const TITRE_TAILLE: f32 = 11.0;
/// Taille des intitulés, en points (§ 3.4 ; 6 pt en partie paiement aussi, C-36-12-6).
const INTITULE_TAILLE: f32 = 6.0;
/// Taille des valeurs, en points (§ 3.4, C-36-12-6).
const VALEUR_TAILLE: f32 = 8.0;
/// Hauteur des capitales d'Helvetica, en em (`CapHeight 718` de `phvr8a.afm`).
const HAUTEUR_CAPITALES: f32 = 0.718;
/// Descendante d'Helvetica, en em (`Descender -207` de `phvr8a.afm`).
const DESCENDANTE: f32 = 0.207;

/// Base des deux titres, en mm : haut à 99,29, dans [98, 100] (échantillons : 5,9 à 6,3 mm
/// du haut de la section).
const TITRE_BASE: f32 = 96.5;
/// Côté du code QR, en mm (§ 6.4).
const QR_COTE: f32 = 46.0;
/// Bord gauche du code QR et du titre de la partie paiement : la ligne verticale à 62, plus
/// la marge (§ 3.3) — 67.
const QR_X: f32 = RECEIPT_W + MARGE;
/// Haut du code QR, en mm : 88, la section montant passant à 5 mm sous son bas (42), § 3.5.2.
const QR_HAUT: f32 = 88.0;
/// Bord gauche de la colonne d'information : le QR et sa bordure de 5 mm (§ 3.5.2) — 118
/// (échantillon : 117,9).
const INFO_X: f32 = QR_X + QR_COTE + MARGE;
/// Largeur de la colonne d'information, jusqu'à la marge droite de la page — 87 mm.
const INFO_LARGEUR: f32 = PAGE_W - MARGE - INFO_X;
/// Largeur de l'information du récépissé, entre ses deux marges — 52 mm.
const RECU_LARGEUR: f32 = RECU_BORD_DROIT - MARGE;
/// Base du premier intitulé du récépissé : haut 92,0 (échantillon : 12,9 mm du haut ; écart
/// assumé aux 5 mm entre sections, fiche 44-1a, D8 (c)).
const RECU_INFO_BASE: f32 = 90.5;
/// Base du premier intitulé de la colonne d'information : haut 99,5 (échantillon : 5,2 mm
/// du haut).
const INFO_BASE: f32 = 98.0;
/// Base des intitulés de la section montant, la même des deux côtés (C-36-12-7) : haut
/// 36,52, à 5 mm sous le bas du QR à 0,48 mm près (§ 3.5 ; échantillons : 68,6 mm du haut).
const MONTANT_BASE: f32 = 35.0;
/// Pas de l'intitulé du montant à sa valeur, en mm.
const MONTANT_PAS: f32 = 3.5;
/// Décalage de l'intitulé et de la valeur du montant, à droite de la monnaie, au récépissé.
const RECU_MONTANT_DECALAGE: f32 = 12.0;
/// Le même décalage, en partie paiement.
const PAIEMENT_MONTANT_DECALAGE: f32 = 15.0;
/// Base du point de dépôt : haut 22,02 (échantillon : 82,7 mm du haut, § 3.6.4) ; son bord
/// droit reste à [`RECU_BORD_DROIT`] (Story 36-12a).
const POINT_DE_DEPOT_BASE: f32 = 20.5;
/// Pas d'un intitulé à sa valeur (IBAN, référence, première ligne du message), en mm.
const INTITULE_PAS: f32 = 3.0;
/// Pas de la dernière valeur d'une rubrique à l'intitulé suivant, en mm.
const RUBRIQUE_PAS: f32 = 6.0;
/// Pas vertical entre deux lignes d'adresse de la section de paiement, en mm — et de
/// l'intitulé *Payable by* (ou du compte) à la première d'entre elles.
const ADRESSE_PAS: f32 = 3.5;
/// Pas vertical d'une ligne du message à la suivante, en mm (même valeur qu'[`ADRESSE_PAS`],
/// constante distincte pour que les positions du message se testent).
const MESSAGE_PAS: f32 = 3.5;
/// Les points de suspension d'une ligne raccourcie : trois points ASCII (§ 3.5.4).
const POINTS_DE_SUSPENSION: &str = "...";

/// Le haut d'une ligne de `taille` points posée sur la base `base`, en mm.
const fn haut_de_ligne(base: f32, taille: f32) -> f32 {
    base + HAUTEUR_CAPITALES * taille * MM_PAR_POINT
}

/// Le bas d'une ligne de `taille` points posée sur la base `base`, en mm.
const fn bas_de_ligne(base: f32, taille: f32) -> f32 {
    base - DESCENDANTE * taille * MM_PAR_POINT
}

/// Le bas que l'information du récépissé ne franchit pas : le haut des intitulés du montant,
/// plus l'espace de 5 mm entre sections (§ 3.5, § 3.6) — 41,52.
const RECU_INFO_BAS_MIN: f32 = haut_de_ligne(MONTANT_BASE, INTITULE_TAILLE) + MARGE;
/// Le bas que la colonne d'information ne franchit pas : la marge du bas de la page.
const INFO_BAS_MIN: f32 = MARGE;

// ----- Adresses de la section de paiement -----

/// Le rôle d'une ligne d'adresse : le palier R1 omet la rue au récépissé (§ 3.6.2), R4
/// garde le nom et la dernière ligne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    /// Le nom (type S et K).
    Nom,
    /// La rue et le numéro (type S).
    Rue,
    /// Le NPA et la localité, le pays devant hors de Suisse (type S).
    Localite,
    /// Une ligne d'adresse du type K, que rien ne compose.
    LigneK,
}

/// Les lignes à imprimer pour une adresse, nom compris, chacune avec son rôle et passée par
/// [`texte_imprime`] : le seul chemin de tout texte d'adresse vers le reçu et la
/// partie paiement (Stories 33-12a et 33-12b, #941).
///
/// En type S, `name`, `rue numéro`, `NPA localité`, le pays devant le NPA hors de
/// Suisse, `LI` compris (`LI-9490 Vaduz`, sans espace, ce qui garde le préfixe au NPA
/// à la coupe, C-e3312-5). En type K, rien n'est composé : aucun chemin d'impression
/// n'emploie ce type, et la directive v2.4 l'a retiré (C-e3312-7).
///
/// Un champ vide après `texte_imprime` ne laisse ni ligne vide ni espace de tête.
fn lignes_adresse_par_role(addr: &Address) -> Vec<(Role, String)> {
    let lignes = match addr.address_type {
        AddressType::Structured => {
            let npa = texte_imprime(&addr.postal_code);
            let npa = if addr.country == "CH" || npa.is_empty() {
                npa
            } else {
                format!("{}-{npa}", addr.country)
            };
            [
                (Role::Nom, texte_imprime(&addr.name)),
                (
                    Role::Rue,
                    joindre(texte_imprime(&addr.line1), texte_imprime(&addr.line2)),
                ),
                (Role::Localite, joindre(npa, texte_imprime(&addr.town))),
            ]
        }
        AddressType::Combined => [
            (Role::Nom, texte_imprime(&addr.name)),
            (Role::LigneK, texte_imprime(&addr.line1)),
            (Role::LigneK, texte_imprime(&addr.line2)),
        ],
    };
    lignes.into_iter().filter(|(_, l)| !l.is_empty()).collect()
}

/// Les lignes de [`lignes_adresse_par_role`], sans leur rôle.
#[cfg(test)]
fn lignes_adresse(addr: &Address) -> Vec<String> {
    lignes_adresse_par_role(addr)
        .into_iter()
        .map(|(_, l)| l)
        .collect()
}

/// Deux morceaux séparés d'une espace, sans espace de tête ni de fin si l'un est vide.
fn joindre(a: String, b: String) -> String {
    match (a.is_empty(), b.is_empty()) {
        (_, true) => a,
        (true, false) => b,
        (false, false) => format!("{a} {b}"),
    }
}

/// La largeur d'une valeur imprimée (8 pt, Helvetica), en mm.
fn largeur_valeur(texte: &str) -> f32 {
    largeur_mm(texte, VALEUR_TAILLE, false)
}

/// Coupe une ligne de valeur (8 pt) en lignes d'au plus `largeur` mm, mesurées par la
/// table AFM d'Helvetica ([`largeur_mm`]), à une frontière de mot ; un mot plus large que
/// `largeur` se coupe au dernier caractère qui tient. Rien n'est tronqué (Story 44-1a, D3 ;
/// la coupe comptait des caractères jusqu'à la 0.18.0, calée sur la largeur moyenne des
/// capitales, et 27 `W` recouvraient le QR).
///
/// La coupe est gloutonne : si le premier mot de la localité ne tient pas à côté du
/// NPA, le NPA reste seul sur sa ligne, l'adresse restant entière (C-e3312-6). Les
/// espaces multiples intérieures se réduisent à une, sur toute ligne : les lignes
/// rejointes par une espace redonnent l'entrée à ces espaces près, hors un mot plus large
/// que la colonne, que seule leur concaténation redonne. Le dernier morceau d'un mot coupé
/// reste ouvert au mot suivant.
fn couper_ligne(ligne: &str, largeur: f32) -> Vec<String> {
    debug_assert!(largeur > 0.0, "une largeur nulle ne coupe rien");
    let mut lignes = Vec::new();
    let mut courante = String::new();
    for mot in ligne.split(' ').filter(|mot| !mot.is_empty()) {
        if !courante.is_empty() {
            if largeur_valeur(&format!("{courante} {mot}")) <= largeur {
                courante.push(' ');
                courante.push_str(mot);
                continue;
            }
            lignes.push(std::mem::take(&mut courante));
        }
        for c in mot.chars() {
            if !courante.is_empty() && largeur_valeur(&format!("{courante}{c}")) > largeur {
                lignes.push(std::mem::take(&mut courante));
            }
            courante.push(c);
        }
    }
    if !courante.is_empty() {
        lignes.push(courante);
    }
    lignes
}

/// Ramène une ligne de valeur (8 pt) à `largeur` mm sur une seule ligne : entière si elle
/// tient, sinon son début le plus long qui laisse la place des [`POINTS_DE_SUSPENSION`],
/// sans espace de fin, suivi d'eux (§ 3.5.4 ; paliers R4 et P3, Story 44-1a, D3 et D4). Les
/// espaces multiples intérieures se réduisent à une, comme dans [`couper_ligne`].
fn raccourcir(ligne: &str, largeur: f32) -> String {
    let ligne = ligne
        .split(' ')
        .filter(|mot| !mot.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if largeur_valeur(&ligne) <= largeur {
        return ligne;
    }
    let mut garde = String::new();
    for c in ligne.chars() {
        if largeur_valeur(&format!("{garde}{c}{POINTS_DE_SUSPENSION}")) > largeur {
            break;
        }
        garde.push(c);
    }
    format!("{}{POINTS_DE_SUSPENSION}", garde.trim_end())
}

// ----- La mise en page de la section de paiement : une fonction pure, puis le dessin -----

/// Un texte placé dans la section de paiement : sa ligne de base en mm, sa taille et sa graisse.
#[derive(Debug, Clone, PartialEq)]
struct Texte {
    x: f32,
    y: f32,
    texte: String,
    taille: f32,
    gras: bool,
}

impl Texte {
    /// Le bas de la ligne, descendantes comprises.
    fn bas(&self) -> f32 {
        bas_de_ligne(self.y, self.taille)
    }
}

/// Les paliers du récépissé (fiche 44-1a, D4), dans l'ordre où ils s'essaient : chacun ne
/// s'applique que si le précédent ne « tient » pas, c'est-à-dire si le bas de l'information
/// passe sous [`RECU_INFO_BAS_MIN`]. La 44-1b insère R2 et R3 entre R1 et R4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PalierRecu {
    /// Tout à 8 pt, adresses complètes, coupées à 52 mm.
    R0,
    /// La rue et le numéro omis, du créancier et du débiteur ensemble (§ 3.6.2) ; sans objet
    /// si aucune des deux adresses n'a de ligne de rue (type K compris).
    R1,
    /// Le dernier palier, qui tient par construction : nom et dernière ligne de chaque
    /// adresse, chacun ramené à 52 mm avec « ... » (fiche 44-1a, D4 et D8 (b)).
    R4,
}

/// Les paliers de la colonne d'information, dans l'ordre où ils s'essaient ; le critère est
/// le bas de la colonne, au-dessus de [`INFO_BAS_MIN`]. La 44-1b insère P1 et P2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PalierPaiement {
    /// Tout, adresses complètes et message entier, coupés aux mots à 87 mm.
    P0,
    /// Le dernier palier, qui tient par construction : message sur une ligne, chaque ligne
    /// d'adresse sur une ligne, ramenés à 87 mm avec « ... » (fiche 44-1a, D4 et D8 (a)).
    P3,
}

/// Les paliers du récépissé, du premier au dernier.
const PALIERS_RECU: [PalierRecu; 3] = [PalierRecu::R0, PalierRecu::R1, PalierRecu::R4];
/// Les paliers de la colonne d'information, du premier au dernier.
const PALIERS_PAIEMENT: [PalierPaiement; 2] = [PalierPaiement::P0, PalierPaiement::P3];

/// Ce que le dessin imprime de la section de paiement, hors lignes, ciseaux et code QR :
/// rendu par [`mise_en_page`], qui ne peut pas échouer. Les paliers et les bas ne servent
/// pas au dessin : les tests les lisent (fiche 44-1a, AC5 et AC6).
#[derive(Debug, Clone)]
#[cfg_attr(not(test), allow(dead_code))]
struct MiseEnPage {
    /// Les textes du récépissé : titre, information, section montant, point de dépôt.
    recu: Vec<Texte>,
    /// Le titre de la partie paiement, dessiné avant le code QR.
    titre_paiement: Texte,
    /// Les textes de la partie paiement après le code QR : colonne d'information, puis
    /// section montant.
    paiement: Vec<Texte>,
    /// Le palier retenu au récépissé.
    palier_recu: PalierRecu,
    /// Le palier retenu dans la colonne d'information.
    palier_paiement: PalierPaiement,
    /// Le bas de la dernière ligne de l'information du récépissé.
    bas_recu: f32,
    /// Le bas de la dernière ligne de la colonne d'information.
    bas_paiement: f32,
}

/// Une colonne de textes posés de haut en bas, chacun un pas sous le précédent ; le premier
/// sur la base donnée.
struct Colonne {
    x: f32,
    y: f32,
    textes: Vec<Texte>,
}

impl Colonne {
    fn new(x: f32, premiere_base: f32) -> Self {
        Colonne {
            x,
            y: premiere_base,
            textes: Vec::new(),
        }
    }

    /// Pose `texte` un `pas` sous le texte précédent (sur la première base s'il n'y en a pas).
    fn poser(&mut self, texte: String, pas: f32, taille: f32, gras: bool) {
        if !self.textes.is_empty() {
            self.y -= pas;
        }
        self.textes.push(Texte {
            x: self.x,
            y: self.y,
            texte,
            taille,
            gras,
        });
    }

    /// Un intitulé (6 pt gras), [`RUBRIQUE_PAS`] sous la dernière valeur.
    fn intitule(&mut self, texte: &str) {
        self.poser(texte.to_string(), RUBRIQUE_PAS, INTITULE_TAILLE, true);
    }

    /// Une valeur (8 pt), `pas` sous le texte précédent.
    fn valeur(&mut self, texte: String, pas: f32) {
        self.poser(texte, pas, VALEUR_TAILLE, false);
    }

    /// Le bas du dernier texte posé.
    fn bas(&self) -> f32 {
        self.textes.last().map_or(self.y, Texte::bas)
    }
}

/// Ce qu'un palier fait des lignes d'une adresse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Forme {
    /// Toutes les lignes, coupées à la largeur (R0, P0).
    Entiere,
    /// Toutes hors la rue, coupées à la largeur (R1).
    SansRue,
    /// La première et la dernière, chacune ramenée à la largeur avec « ... » (R4) ; une
    /// seule si l'adresse n'en a qu'une.
    Extremes,
    /// Chacune ramenée à la largeur avec « ... » (P3).
    UneLigneChacune,
}

/// Les lignes d'une adresse sous la forme `forme`, chacune d'au plus `largeur` mm.
fn lignes_sous_forme(addr: &Address, forme: Forme, largeur: f32) -> Vec<String> {
    let lignes = lignes_adresse_par_role(addr);
    match forme {
        Forme::Entiere | Forme::SansRue => lignes
            .iter()
            .filter(|(role, _)| forme == Forme::Entiere || *role != Role::Rue)
            .flat_map(|(_, l)| couper_ligne(l, largeur))
            .collect(),
        Forme::Extremes => {
            let mut extremes: Vec<&String> = lignes.iter().map(|(_, l)| l).take(1).collect();
            if lignes.len() > 1 {
                extremes.extend(lignes.last().map(|(_, l)| l));
            }
            extremes
                .into_iter()
                .map(|l| raccourcir(l, largeur))
                .collect()
        }
        Forme::UneLigneChacune => lignes.iter().map(|(_, l)| raccourcir(l, largeur)).collect(),
    }
}

/// L'information du récépissé au palier donné (§ 3.6.2) : compte et créancier, référence,
/// *Payable by* et débiteur — *Payable by* avant la section montant (§ 3.5.4, Story 44-1a).
fn information_du_recu(data: &QrBillData, i18n: &QrBillI18n, palier: PalierRecu) -> Colonne {
    let forme = match palier {
        PalierRecu::R0 => Forme::Entiere,
        PalierRecu::R1 => Forme::SansRue,
        PalierRecu::R4 => Forme::Extremes,
    };
    let mut c = Colonne::new(MARGE, RECU_INFO_BASE);
    c.intitule(i18n.get("invoice-pdf-qr-account"));
    c.valeur(format_iban(&data.creditor_iban), INTITULE_PAS);
    for ligne in lignes_sous_forme(&data.creditor, forme, RECU_LARGEUR) {
        c.valeur(ligne, ADRESSE_PAS);
    }
    if let Reference::Qrr(qrr) = &data.reference {
        c.intitule(i18n.get("invoice-pdf-qr-reference"));
        c.valeur(format_qrr(qrr), INTITULE_PAS);
    }
    c.intitule(i18n.get("invoice-pdf-qr-payable-by"));
    if let Some(debiteur) = &data.ultimate_debtor {
        for ligne in lignes_sous_forme(debiteur, forme, RECU_LARGEUR) {
            c.valeur(ligne, ADRESSE_PAS);
        }
    }
    c
}

/// La colonne d'information de la partie paiement au palier donné (§ 3.5.4) : compte et
/// créancier, référence, *Additional information*, puis *Payable by* et débiteur.
fn information_du_paiement(
    data: &QrBillData,
    i18n: &QrBillI18n,
    message: Option<&str>,
    palier: PalierPaiement,
) -> Colonne {
    let forme = match palier {
        PalierPaiement::P0 => Forme::Entiere,
        PalierPaiement::P3 => Forme::UneLigneChacune,
    };
    let mut c = Colonne::new(INFO_X, INFO_BASE);
    c.intitule(i18n.get("invoice-pdf-qr-account"));
    c.valeur(format_iban(&data.creditor_iban), INTITULE_PAS);
    for ligne in lignes_sous_forme(&data.creditor, forme, INFO_LARGEUR) {
        c.valeur(ligne, ADRESSE_PAS);
    }
    if let Reference::Qrr(qrr) = &data.reference {
        c.intitule(i18n.get("invoice-pdf-qr-reference"));
        c.valeur(format_qrr(qrr), INTITULE_PAS);
    }
    if let Some(message) = message {
        c.intitule(i18n.get("invoice-pdf-qr-additional-info"));
        let lignes = match palier {
            PalierPaiement::P0 => couper_ligne(message, INFO_LARGEUR),
            PalierPaiement::P3 => vec![raccourcir(message, INFO_LARGEUR)],
        };
        for (i, ligne) in lignes.into_iter().enumerate() {
            c.valeur(ligne, if i == 0 { INTITULE_PAS } else { MESSAGE_PAS });
        }
    }
    c.intitule(i18n.get("invoice-pdf-qr-payable-by"));
    if let Some(debiteur) = &data.ultimate_debtor {
        for ligne in lignes_sous_forme(debiteur, forme, INFO_LARGEUR) {
            c.valeur(ligne, ADRESSE_PAS);
        }
    }
    c
}

/// La section montant, à sa place fixe (C-36-12-7) : monnaie et montant, intitulés à la base
/// [`MONTANT_BASE`], valeurs [`MONTANT_PAS`] plus bas, le montant décalé de `decalage`.
fn section_montant(
    i18n: &QrBillI18n,
    x: f32,
    decalage: f32,
    devise: &str,
    montant: &str,
) -> [Texte; 4] {
    let texte = |x: f32, y: f32, texte: &str, taille: f32, gras: bool| Texte {
        x,
        y,
        texte: texte.to_string(),
        taille,
        gras,
    };
    let valeurs = MONTANT_BASE - MONTANT_PAS;
    [
        texte(
            x,
            MONTANT_BASE,
            i18n.get("invoice-pdf-qr-currency"),
            INTITULE_TAILLE,
            true,
        ),
        texte(
            x + decalage,
            MONTANT_BASE,
            i18n.get("invoice-pdf-qr-amount"),
            INTITULE_TAILLE,
            true,
        ),
        texte(x, valeurs, devise, VALEUR_TAILLE, false),
        texte(x + decalage, valeurs, montant, VALEUR_TAILLE, false),
    ]
}

/// La mise en page de la section de paiement, sur la grille de la directive (Story 44-1a,
/// D2) : une fonction **pure**, qui rend toujours une mise en page et **jamais une erreur**
/// (AC7). Le récépissé et la colonne d'information choisissent chacun leur palier, l'un
/// sans l'autre : le premier qui tient, ou le dernier, qui tient par construction.
///
/// `devise` et `montant` sont déjà formatés : le montant absent est refusé avant, par
/// [`dessiner_section_de_paiement`].
fn mise_en_page(data: &QrBillData, i18n: &QrBillI18n, devise: &str, montant: &str) -> MiseEnPage {
    // Récépissé : R0, R1 (s'il y a une rue à omettre), R4.
    let une_rue = std::iter::once(&data.creditor)
        .chain(data.ultimate_debtor.as_ref())
        .any(|a| {
            lignes_adresse_par_role(a)
                .iter()
                .any(|(r, _)| *r == Role::Rue)
        });
    let (dernier_recu, avant_recu) = PALIERS_RECU.split_last().expect("des paliers");
    let (palier_recu, info_recu) = avant_recu
        .iter()
        .copied()
        .filter(|p| *p != PalierRecu::R1 || une_rue)
        .map(|p| (p, information_du_recu(data, i18n, p)))
        .find(|(_, c)| c.bas() >= RECU_INFO_BAS_MIN)
        .unwrap_or_else(|| {
            (
                *dernier_recu,
                information_du_recu(data, i18n, *dernier_recu),
            )
        });
    debug_assert!(
        info_recu.bas() >= RECU_INFO_BAS_MIN,
        "le dernier palier du récépissé tient par construction"
    );

    // Colonne d'information : P0, P3.
    let message = data
        .unstructured_message
        .as_deref()
        .map(texte_imprime)
        .filter(|message| !message.is_empty());
    let (dernier_paiement, avant_paiement) = PALIERS_PAIEMENT.split_last().expect("des paliers");
    let (palier_paiement, info_paiement) = avant_paiement
        .iter()
        .copied()
        .map(|p| {
            (
                p,
                information_du_paiement(data, i18n, message.as_deref(), p),
            )
        })
        .find(|(_, c)| c.bas() >= INFO_BAS_MIN)
        .unwrap_or_else(|| {
            (
                *dernier_paiement,
                information_du_paiement(data, i18n, message.as_deref(), *dernier_paiement),
            )
        });
    debug_assert!(
        info_paiement.bas() >= INFO_BAS_MIN,
        "le dernier palier de la colonne tient par construction"
    );

    let titre = |x: f32, cle: &'static str| Texte {
        x,
        y: TITRE_BASE,
        texte: i18n.get(cle).to_string(),
        taille: TITRE_TAILLE,
        gras: true,
    };
    // Le point de dépôt, en bas à droite du récépissé, « printed right-aligned » (§ 3.6.4) :
    // son intitulé finit au bord droit du récépissé dans toutes les langues (Story 36-12a).
    let point_de_depot = i18n.get("invoice-pdf-qr-acceptance-point");
    let point_de_depot = Texte {
        x: RECU_BORD_DROIT - largeur_mm(point_de_depot, INTITULE_TAILLE, true),
        y: POINT_DE_DEPOT_BASE,
        texte: point_de_depot.to_string(),
        taille: INTITULE_TAILLE,
        gras: true,
    };

    let (bas_recu, bas_paiement) = (info_recu.bas(), info_paiement.bas());
    let mut recu = vec![titre(MARGE, "invoice-pdf-qr-section-receipt")];
    recu.extend(info_recu.textes);
    recu.extend(section_montant(
        i18n,
        MARGE,
        RECU_MONTANT_DECALAGE,
        devise,
        montant,
    ));
    recu.push(point_de_depot);
    let mut paiement = info_paiement.textes;
    paiement.extend(section_montant(
        i18n,
        QR_X,
        PAIEMENT_MONTANT_DECALAGE,
        devise,
        montant,
    ));
    MiseEnPage {
        recu,
        titre_paiement: titre(QR_X, "invoice-pdf-qr-section-payment"),
        paiement,
        palier_recu,
        palier_paiement,
        bas_recu,
        bas_paiement,
    }
}

/// Dessine un texte de la mise en page.
fn dessiner_texte(
    layer: &PdfLayerReference,
    t: &Texte,
    helv: &IndirectFontRef,
    helv_bold: &IndirectFontRef,
) {
    let police = if t.gras { helv_bold } else { helv };
    layer.use_text(t.texte.clone(), t.taille, Mm(t.x), Mm(t.y), police);
}

/// Dessine la section de paiement (récépissé et partie paiement, hors lignes et ciseaux) :
/// la [`mise_en_page`], le code QR de 46 mm à (67, 88) et sa croix centrée en (90, 65).
///
/// Seul le montant absent (et le QR, par [`draw_qr_matrix`]) peut faire échouer ce dessin :
/// la mise en page, elle, ne le peut pas.
#[allow(clippy::too_many_arguments)]
fn dessiner_section_de_paiement(
    layer: &PdfLayerReference,
    data: &QrBillData,
    invoice: &InvoicePdfData,
    i18n: &QrBillI18n,
    helv: &IndirectFontRef,
    helv_bold: &IndirectFontRef,
    qr: &QrCode,
) -> Result<(), QrBillError> {
    let montant = data
        .amount
        .ok_or_else(|| QrBillError::InvalidAmount("montant requis pour rendu PDF".into()))?
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    let page = mise_en_page(data, i18n, invoice.currency.code(), &format_ch(montant, 2));
    for t in &page.recu {
        dessiner_texte(layer, t, helv, helv_bold);
    }
    dessiner_texte(layer, &page.titre_paiement, helv, helv_bold);
    draw_qr_matrix(layer, qr, QR_X, QR_HAUT, QR_COTE)?;
    draw_swiss_cross(layer, QR_X + QR_COTE / 2.0, QR_HAUT - QR_COTE / 2.0);
    // Après la croix, qui remet le remplissage au noir (`chaque_texte_s_imprime_en_noir`).
    for t in &page.paiement {
        dessiner_texte(layer, t, helv, helv_bold);
    }
    Ok(())
}

fn draw_qr_matrix(
    layer: &PdfLayerReference,
    qr: &QrCode,
    x_mm: f32,
    top_mm: f32,
    size_mm: f32,
) -> Result<(), QrBillError> {
    let modules = qr.size() as f32;
    let module_mm = size_mm / modules;
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                let llx = x_mm + x as f32 * module_mm;
                // top-left coordinate system: y=0 at page top. printpdf origin is bottom-left.
                let ury = top_mm - y as f32 * module_mm;
                let lly = ury - module_mm;
                let urx = llx + module_mm;
                let rect = printpdf::Rect::new(Mm(llx), Mm(lly), Mm(urx), Mm(ury))
                    .with_mode(printpdf::path::PaintMode::Fill);
                layer.add_rect(rect);
            }
        }
    }
    Ok(())
}

/// Draw the Swiss cross logo per SIX QR Bill §5.2 spec, centered at `(cx, cy)`.
///
/// Geometry corrected M7 (review pass 1 G2 C) :
/// - Carré blanc extérieur 7 × 7 mm (zone de garde)
/// - Carré rouge plein 7 × 7 mm (le rouge remplit le carré blanc, pas 6×6)
/// - Croix blanche : branches 4.55 × 1.3 mm (SIX §5.2 strict, pas 6.3 × 1.3)
fn draw_swiss_cross(layer: &PdfLayerReference, cx: f32, cy: f32) {
    // Pass 2 : géométrie SIX §5.2 corrigée — carré blanc 8×8 mm (zone de
    // garde sur les modules QR sous-jacents) qui contient un carré rouge
    // 7×7 mm (logo officiel) lui-même surchargé d'une croix blanche
    // 4.55 × 1.3 mm. L'erreur de pass 1 (blanc=rouge=7×7) annulait la zone
    // de garde et rendait la croix collée aux modules QR.
    let outer = 8.0;
    let red_sq = 7.0;
    layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
    let bg = printpdf::Rect::new(
        Mm(cx - outer / 2.0),
        Mm(cy - outer / 2.0),
        Mm(cx + outer / 2.0),
        Mm(cy + outer / 2.0),
    )
    .with_mode(printpdf::path::PaintMode::Fill);
    layer.add_rect(bg);

    // Carré rouge SIX (CMYK 0/100/100/0 ≈ RGB 0.85/0/0).
    layer.set_fill_color(Color::Rgb(Rgb::new(0.85, 0.0, 0.0, None)));
    let red = printpdf::Rect::new(
        Mm(cx - red_sq / 2.0),
        Mm(cy - red_sq / 2.0),
        Mm(cx + red_sq / 2.0),
        Mm(cy + red_sq / 2.0),
    )
    .with_mode(printpdf::path::PaintMode::Fill);
    layer.add_rect(red);

    // Branches blanches de la croix : 4.55 mm × 1.3 mm (SIX §5.2 strict).
    let arm_len = 4.55_f32;
    let arm_w = 1.3_f32;
    layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
    let h = printpdf::Rect::new(
        Mm(cx - arm_len / 2.0),
        Mm(cy - arm_w / 2.0),
        Mm(cx + arm_len / 2.0),
        Mm(cy + arm_w / 2.0),
    )
    .with_mode(printpdf::path::PaintMode::Fill);
    let v = printpdf::Rect::new(
        Mm(cx - arm_w / 2.0),
        Mm(cy - arm_len / 2.0),
        Mm(cx + arm_w / 2.0),
        Mm(cy + arm_len / 2.0),
    )
    .with_mode(printpdf::path::PaintMode::Fill);
    layer.add_rect(h);
    layer.add_rect(v);

    // ⚠️ Le texte suivant hérite du remplissage : rester en blanc rendait la partie
    // paiement entière invisible sur le papier, le contenu du PDF restant intact.
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
}

// ----- Helpers -----

fn hline(layer: &PdfLayerReference, x1: f32, x2: f32, y: f32) {
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.2);
    let l = Line {
        points: vec![
            (Point::new(Mm(x1), Mm(y)), false),
            (Point::new(Mm(x2), Mm(y)), false),
        ],
        is_closed: false,
    };
    layer.add_line(l);
}

/// [`format_ch`] avec un signe **toujours** affiché — `+0.01`, `-0.02` : la ligne
/// « Arrondi » dit dans quel sens le total a bougé (Story 25-4-c4-b). L'appelant
/// passe une valeur déjà arrondie à `decimals`, pour que le signe et le montant
/// affiché disent la même chose.
pub fn format_signed_ch(value: Decimal, decimals: u32) -> String {
    let sign = if value.is_sign_negative() && !value.is_zero() {
        "-"
    } else {
        "+"
    };
    format!("{sign}{}", format_ch(value.abs(), decimals))
}

/// Swiss number format: apostrophe thousand separator, point decimal.
pub fn format_ch(value: Decimal, decimals: u32) -> String {
    let rounded = value.round_dp_with_strategy(decimals, RoundingStrategy::MidpointAwayFromZero);
    let s = rounded.abs().to_string();
    let (int_part, frac_part) = match s.split_once('.') {
        Some((i, f)) => (i.to_string(), f.to_string()),
        None => (s, String::new()),
    };
    // Insert apostrophes every 3 digits from the right.
    let mut with_sep = String::new();
    for (i, c) in int_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            with_sep.push('\'');
        }
        with_sep.push(c);
    }
    let int_str: String = with_sep.chars().rev().collect();
    let frac_padded = if decimals == 0 {
        String::new()
    } else {
        format!(".{:0<width$}", frac_part, width = decimals as usize)
    };
    let sign = if rounded.is_sign_negative() { "-" } else { "" };
    format!("{}{}{}", sign, int_str, frac_padded)
}

/// Format IBAN with grouped spaces every 4 characters (for display).
fn format_iban(iban: &str) -> String {
    let normalized: String = iban.chars().filter(|c| !c.is_whitespace()).collect();
    let mut out = String::with_capacity(normalized.len() + 6);
    for (i, c) in normalized.chars().enumerate() {
        if i > 0 && i % 4 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Format QRR (27 digits) as groups — SIX: `XX XXXXX XXXXX XXXXX XXXXX XXXXX` (2+5×5).
fn format_qrr(qrr: &str) -> String {
    if qrr.len() != 27 {
        return qrr.to_string();
    }
    format!(
        "{} {} {} {} {} {}",
        &qrr[0..2],
        &qrr[2..7],
        &qrr[7..12],
        &qrr[12..17],
        &qrr[17..22],
        &qrr[22..27],
    )
}

fn format_date_ch(d: NaiveDate) -> String {
    format!("{:02}.{:02}.{:04}", d.day(), d.month(), d.year())
}

// `is_invisible` — « vrai si le caractère ne marque rien sur la page » — vit
// dans `kesh_core::text` depuis la Story 22-1 (import en tête de fichier) : il
// était écrit deux fois, ici et dans la normalisation d'entrée de `kesh-api`.
// Cette garde de rendu doit rester LA MÊME CHOSE que la garde de saisie : le
// PDF tient face à une base qui n'est pas passée par la route (restauration,
// SQL direct), mais avec la définition unique de « invisible ». Son rôle ici :
// ne pas dessiner une ligne blanche qui consommerait un `META_LINE_STEP` pour
// une valeur faite de ZWSP/BOM/word-joiner, vides à l'impression.

/// Pas vertical entre deux lignes du bloc de rappel, en mm (Story 25-4-b2).
const REMINDER_LINE_STEP: f32 = 4.5;

/// Longueur maximale, en caractères, de la **mention** des frais de rappel, qui
/// court sur toute la largeur utile (170 mm, de `col_desc` à la marge droite).
/// Même calibrage que [`IDENTITY_MAX_CHARS`] (46 caractères pour 100 mm à 9 pt),
/// avec la même limite : une borne en caractères, pas une largeur mesurée.
///
/// ⚠️ Les gardes de capacité ne surveillent que l'ORDONNÉE : sans cette borne,
/// une traduction plus longue déborderait à droite sans que rien ne rougisse.
pub const REMINDER_NOTE_MAX_CHARS: usize = 78;

/// Longueur maximale des **libellés courts** du bloc de rappel (déjà réglé, reste
/// à payer, frais de rappel), dessinés dans la colonne des libellés : 50 mm de
/// `col_unit` à `col_tot`, soit 23 caractères au calibrage d'[`IDENTITY_MAX_CHARS`].
/// Au-delà, le libellé chevaucherait le montant (revue de code 25-4-b2, P1).
pub const REMINDER_LABEL_MAX_CHARS: usize = 23;

/// Une ligne du récapitulatif, au-dessus du total : sous-total HT, TVA par taux,
/// arrondi à 5 centimes.
#[derive(Debug, Clone, PartialEq)]
struct RecapLine {
    label: String,
    amount: String,
}

/// Le récapitulatif d'une pièce (#151, Story 25-4-c4-b) — sans le total, que le
/// gabarit dessine en gras à part.
///
/// Affiché s'il existe des lignes taxées **ou** un arrondi : « Sous-total », une
/// ligne « TVA {taux} % » par taux, puis « Arrondi » avec l'écart signé. Sinon
/// (société non assujettie, total déjà rond) : vide, et le gabarit n'imprime que
/// le total — comportement rétro-compatible.
///
/// Fonction pure : la hauteur réservée pour la garde `TooManyLines` se déduit de
/// la même liste que le dessin.
fn recap_lines(inv: &InvoicePdfData, i18n: &QrBillI18n) -> Vec<RecapLine> {
    // ⛔ Décidé AU CENTIME, comme il s'affiche : l'écart figé peut porter quatre
    // décimales (les lignes en ont quatre), et un écart de 0.0004 imprimerait
    // « +0.00 » sur une ligne qui n'explique rien (revue de code P1, lentille A).
    let rounding = inv
        .rounding
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    if inv.vat_lines.is_empty() && rounding.is_zero() {
        return Vec::new();
    }
    let currency = inv.currency.code();
    let money = |d: Decimal| {
        format!(
            "{currency} {}",
            format_ch(
                d.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero),
                2
            )
        )
    };
    let mut out = vec![RecapLine {
        label: i18n.get("invoice-pdf-subtotal").to_string(),
        amount: money(inv.subtotal_ht),
    }];
    for v in &inv.vat_lines {
        out.push(RecapLine {
            label: format!(
                "{} {}%",
                i18n.get("invoice-pdf-vat"),
                format_ch(v.rate_percent, 1)
            ),
            amount: money(v.amount),
        });
    }
    if !rounding.is_zero() {
        out.push(RecapLine {
            label: i18n.get("invoice-pdf-rounding").to_string(),
            amount: format!("{currency} {}", format_signed_ch(rounding, 2)),
        });
    }
    out
}

/// Une ligne du bloc de rappel, **construite** avant d'être dessinée — c'est ce
/// qui rend son contenu testable (le texte d'un PDF est hex-encodé dans les
/// opérateurs `Tj` et ne se compare pas).
#[derive(Debug, Clone, PartialEq)]
struct ReminderLine {
    label: String,
    amount: Option<String>,
    bold: bool,
    /// `true` ⇒ la ligne part de `col_desc` et court sur toute la largeur (la
    /// mention des frais, trop longue pour la colonne des libellés).
    full_width: bool,
}

/// Les lignes du bloc de rappel, dans l'ordre (Story 25-4-b2, AC 6 et 8) :
/// « déjà réglé » et « reste à payer » **seulement s'il y a eu un règlement**
/// (sinon le total est le reste — arbitrage Q2), puis « frais de rappel » et sa
/// mention **seulement si les frais sont non nuls** (arbitrage Q1).
fn reminder_lines(r: &ReminderPdf, i18n: &QrBillI18n, currency: &str) -> Vec<ReminderLine> {
    let money = |d: Decimal| {
        format!(
            "{} {}",
            currency,
            format_ch(
                d.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero),
                2
            )
        )
    };
    let mut out = Vec::new();
    if r.amount_settled > Decimal::ZERO {
        out.push(ReminderLine {
            label: i18n.get("invoice-pdf-settled").to_string(),
            amount: Some(format!("-{}", money(r.amount_settled))),
            bold: false,
            full_width: false,
        });
        out.push(ReminderLine {
            label: i18n.get("invoice-pdf-amount-due").to_string(),
            amount: Some(money(r.amount_due)),
            bold: true,
            full_width: false,
        });
    }
    if r.fees > Decimal::ZERO {
        out.push(ReminderLine {
            label: i18n.get("invoice-pdf-reminder-fees").to_string(),
            amount: Some(money(r.fees)),
            bold: false,
            full_width: false,
        });
        out.push(ReminderLine {
            label: i18n.get("invoice-pdf-reminder-fees-note").to_string(),
            amount: None,
            bold: false,
            full_width: true,
        });
    }
    out
}

/// Hauteur du bloc de rappel, à réserver dans la garde de capacité.
fn reminder_block_height(r: Option<&ReminderPdf>) -> f32 {
    r.map(|r| {
        // Le `QrBillI18n` par défaut suffit : seul le NOMBRE de lignes compte.
        reminder_lines(r, &QrBillI18n::default(), "CHF").len() as f32 * REMINDER_LINE_STEP
    })
    .unwrap_or(0.0)
}

fn truncate_display(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Address, AddressType, Currency, InvoiceLinePdf, InvoiceVatLinePdf, QrBillData,
    };
    use rust_decimal_macros::dec;

    fn invoice_fixture() -> (QrBillData, InvoicePdfData, QrBillI18n) {
        let creditor = Address {
            address_type: AddressType::Combined,
            name: "Robert Schneider SA".into(),
            line1: "Rue du Lac 1268".into(),
            line2: "2501 Biel".into(),
            postal_code: String::new(),
            town: String::new(),
            country: "CH".into(),
        };
        let debtor = Address {
            address_type: AddressType::Combined,
            name: "Pia Rutschmann".into(),
            line1: "Marktgasse 28".into(),
            line2: "9400 Rorschach".into(),
            postal_code: String::new(),
            town: String::new(),
            country: "CH".into(),
        };
        let qrr = crate::validation::build_qrr(42, 100).unwrap();
        let data = QrBillData {
            creditor_iban: "CH4431999123000889012".into(),
            creditor: creditor.clone(),
            ultimate_debtor: Some(debtor.clone()),
            amount: Some(dec!(1234.50)),
            currency: Currency::Chf,
            reference: Reference::Qrr(qrr),
            unstructured_message: Some("Facture F-2026-0042".into()),
            billing_information: None,
        };
        let invoice = InvoicePdfData {
            invoice_number: "F-2026-0042".into(),
            invoice_date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 14),
            payment_terms: Some("30 jours net".into()),
            creditor_name: "Robert Schneider SA".into(),
            creditor_address_lines: vec!["Rue du Lac 1268".into(), "2501 Biel".into()],
            creditor_ide: Some("CHE-123.456.789".into()),
            // Story 16-3a (#151) — la fixture de base ne porte AUCUNE coordonnée :
            // c'est le cas nominal d'une société qui n'a rien renseigné, et il doit
            // rendre le PDF d'avant la story. Les tests qui les exercent les posent.
            creditor_phone: None,
            creditor_email: None,
            creditor_website: None,
            debtor_name: "Pia Rutschmann".into(),
            debtor_address_lines: vec!["Marktgasse 28".into(), "9400 Rorschach".into()],
            debtor_client_number: None,
            lines: vec![InvoiceLinePdf {
                description: "Conseil stratégique".into(),
                quantity: dec!(10),
                unit_price: dec!(120.00),
                vat_rate: dec!(7.70),
                line_total: dec!(1200.00),
            }],
            subtotal_ht: dec!(1200.00),
            vat_lines: vec![InvoiceVatLinePdf {
                rate_percent: dec!(7.70),
                amount: dec!(92.40), // 1200.00 × 7.70 %
            }],
            rounding: rust_decimal::Decimal::ZERO,
            total: dec!(1292.40), // 1200.00 + 92.40
            currency: Currency::Chf,
            origin_reference: None,
            reminder: None,
        };
        (data, invoice, QrBillI18n::default())
    }

    #[test]
    fn generates_valid_pdf_bytes() {
        let (data, invoice, i18n) = invoice_fixture();
        let bytes = generate_qr_bill_pdf(&data, &invoice, &i18n).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."), "missing PDF magic");
        assert!(
            bytes.len() > 1_000,
            "PDF suspiciously small: {}",
            bytes.len()
        );
    }

    /// Les textes dessinés d'un PDF, en mm depuis le bas à gauche, par relecture du
    /// contenu : `load_mem`, `get_page_content`, `Content::decode`, puis pour chaque
    /// `Tj`, le décodage WinAnsi, à la position du `Td` précédent.
    ///
    /// ⚠️ `Document::extract_text` n'est pas employé : il lit en StandardEncoding et
    /// rend `Shne` pour `Söhne` (constaté à l'étape 0 de la Story 33-12a).
    fn textes_du_pdf(bytes: &[u8]) -> Vec<(f32, f32, String)> {
        const MM_PAR_POINT: f32 = 25.4 / 72.0;
        let mut textes = Vec::new();
        let mut position = (0.0, 0.0);
        for op in operations_du_pdf(bytes) {
            match op.operator.as_str() {
                "Td" => {
                    position = (
                        nombre(&op.operands[0]) * MM_PAR_POINT,
                        nombre(&op.operands[1]) * MM_PAR_POINT,
                    );
                }
                "Tj" => textes.push((position.0, position.1, texte_du_tj(&op))),
                _ => {}
            }
        }
        textes
    }

    /// Les opérations de contenu d'un PDF, pages à la suite. Une QR-facture n'a qu'une
    /// page : ses lecteurs ne remettent ni position ni couleur à zéro entre deux pages.
    fn operations_du_pdf(bytes: &[u8]) -> Vec<printpdf::lopdf::content::Operation> {
        use printpdf::lopdf::{Document, content::Content};

        let doc = Document::load_mem(bytes).expect("PDF relisible");
        doc.get_pages()
            .values()
            .flat_map(|page| {
                let brut = doc.get_page_content(*page).expect("contenu de page");
                Content::decode(&brut)
                    .expect("contenu décodable")
                    .operations
            })
            .collect()
    }

    /// Une opérande numérique de contenu PDF.
    fn nombre(o: &printpdf::lopdf::Object) -> f32 {
        use printpdf::lopdf::Object;
        match o {
            Object::Integer(i) => *i as f32,
            Object::Real(r) => *r,
            autre => panic!("opérande numérique attendue : {autre:?}"),
        }
    }

    /// Le texte d'un `Tj`, décodé en WinAnsi.
    fn texte_du_tj(op: &printpdf::lopdf::content::Operation) -> String {
        use printpdf::lopdf::{Document, Object};
        let Object::String(octets, _) = &op.operands[0] else {
            panic!("opérande de Tj inattendue : {:?}", op.operands);
        };
        Document::decode_text(Some("WinAnsiEncoding"), octets)
    }

    /// Le texte dessiné à `(x, y)` mm, à la tolérance des arrondis en points.
    fn texte_a(textes: &[(f32, f32, String)], x: f32, y: f32) -> &str {
        textes
            .iter()
            .find(|(tx, ty, _)| (tx - x).abs() < 0.01 && (ty - y).abs() < 0.01)
            .map(|(_, _, t)| t.as_str())
            .unwrap_or_else(|| panic!("rien à ({x}, {y}) dans {textes:#?}"))
    }

    /// Une adresse de type S, la seule que la production imprime.
    fn adresse_s(name: &str, line1: &str, line2: &str, npa: &str, ville: &str) -> Address {
        Address {
            address_type: AddressType::Structured,
            name: name.into(),
            line1: line1.into(),
            line2: line2.into(),
            postal_code: npa.into(),
            town: ville.into(),
            country: "CH".into(),
        }
    }

    /// Les textes du PDF d'une QR-facture à référence QRR (celle de la fixture),
    /// pour ce créancier et ce débiteur.
    fn textes_pour(creancier: Address, debiteur: Address) -> Vec<(f32, f32, String)> {
        let (mut data, invoice, i18n) = invoice_fixture();
        data.creditor = creancier;
        data.ultimate_debtor = Some(debiteur);
        let pdf = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
        textes_du_pdf(&pdf)
    }

    /// Une adresse de type S, à l'étranger.
    fn adresse_etrangere(
        name: &str,
        line1: &str,
        line2: &str,
        npa: &str,
        ville: &str,
        pays: &str,
    ) -> Address {
        Address {
            country: pays.into(),
            ..adresse_s(name, line1, line2, npa, ville)
        }
    }

    fn creancier_ordinaire() -> Address {
        adresse_s("Robert Schneider SA", "Rue du Lac", "1268", "2501", "Biel")
    }

    fn debiteur_ordinaire() -> Address {
        adresse_s("Pia Rutschmann", "Marktgasse", "28", "9400", "Rorschach")
    }

    /// **AC1 et AC2** (Story 33-12b, #941) — la composition, par paires : `name`,
    /// `rue numéro`, `NPA localité`, sans ligne vide ni espace de tête, avec le pays
    /// hors de Suisse.
    #[test]
    fn lignes_adresse_compose_le_type_s() {
        for (addr, attendu) in [
            (
                adresse_s("Max", "Rue du Lac", "12", "1003", "Lausanne"),
                vec!["Max", "Rue du Lac 12", "1003 Lausanne"],
            ),
            (
                adresse_s("Max", "Rue du Lac", "", "1003", "Lausanne"),
                vec!["Max", "Rue du Lac", "1003 Lausanne"],
            ),
            (
                adresse_s("Max", "", "12", "1003", "Lausanne"),
                vec!["Max", "12", "1003 Lausanne"],
            ),
            (
                adresse_s("Max", "", "", "1003", "Lausanne"),
                vec!["Max", "1003 Lausanne"],
            ),
            (
                adresse_s(
                    "Čolić\u{A0}",
                    "\u{A0}\u{AD}",
                    "\u{AD}",
                    " 1003",
                    "Lausanne\u{A0}",
                ),
                vec!["Colic", "1003 Lausanne"],
            ),
            (
                adresse_etrangere("Max", "Hauptstrasse", "5", "78462", "Konstanz", "DE"),
                vec!["Max", "Hauptstrasse 5", "DE-78462 Konstanz"],
            ),
            // Inatteignable après `validate`, qui refuse ce NPA ; la garde reste écrite.
            (
                adresse_etrangere("Max", "Hauptstrasse", "5", "\u{AD}", "Konstanz", "DE"),
                vec!["Max", "Hauptstrasse 5", "Konstanz"],
            ),
            (
                adresse_etrangere("Max", "Städtle", "5", "9490", "Vaduz", "LI"),
                vec!["Max", "Städtle 5", "LI-9490 Vaduz"],
            ),
        ] {
            assert_eq!(lignes_adresse(&addr), attendu, "{addr:?}");
        }
    }

    /// **M14 de la Story 33-12b** — NPA et localité passent par `texte_imprime`.
    #[test]
    fn lignes_adresse_npa_localite_repliees() {
        let addr = adresse_etrangere("Max", "Rynek", "1", "90-001", "Łódź", "PL");
        assert_eq!(lignes_adresse(&addr)[2], "PL-90-001 Lódz");
    }

    /// **AC1 et AC2** (Story 33-12b, C-e3312-7) — en type K, rien n'est composé :
    /// aucune recomposition, aucun préfixe, y compris à l'étranger.
    #[test]
    fn lignes_adresse_type_k_ne_compose_rien() {
        let k = Address {
            address_type: AddressType::Combined,
            name: "Max".into(),
            line1: "Łódzka 2".into(),
            line2: "78462 Konstanz".into(),
            postal_code: String::new(),
            town: String::new(),
            country: "DE".into(),
        };
        assert_eq!(lignes_adresse(&k), ["Max", "Lódzka 2", "78462 Konstanz"]);
    }

    /// **AC3** (Story 33-12b), à la largeur depuis la Story 44-1a — un mot plus large que
    /// la colonne se coupe au dernier caractère qui tient, sans perte : 27 `A` font 50,83 mm
    /// (`A` = 667 à l'AFM), 28 font 52,71 mm, au-delà des 52 du récépissé.
    #[test]
    fn couper_ligne_mot_long_sans_perte() {
        let mot = "A".repeat(30);
        let lignes = couper_ligne(&mot, RECU_LARGEUR);
        assert_eq!(lignes, ["A".repeat(27), "A".repeat(3)]);
        assert_eq!(lignes.concat(), mot);

        // Un mot long après un mot court part sur sa propre ligne.
        let lignes = couper_ligne(&format!("Rue {mot}"), RECU_LARGEUR);
        assert_eq!(lignes, ["Rue".to_string(), "A".repeat(27), "A".repeat(3)]);

        // Un mot long coupé laisse sa fin ouverte au mot suivant.
        let lignes = couper_ligne(&format!("{mot} fin"), RECU_LARGEUR);
        assert_eq!(lignes, ["A".repeat(27), "AAA fin".to_string()]);
    }

    /// **AC3** (Story 33-12b), à la largeur depuis la Story 44-1a (AC4) — une rue longue :
    /// aucune ligne au-delà de la largeur, les lignes rejointes par une espace redonnent
    /// l'entrée. Au récépissé, la rue de 61 caractères ne fait plus trois lignes mais deux ;
    /// dans la colonne, une seule.
    #[test]
    fn couper_ligne_rue_entiere() {
        let rue = "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau 12";
        for (largeur, attendu) in [
            (
                RECU_LARGEUR,
                vec![
                    "Chemin des Trois-Fontaines de la Vieille",
                    "Tour du Haut-Plateau 12",
                ],
            ),
            (INFO_LARGEUR, vec![rue]),
        ] {
            let lignes = couper_ligne(rue, largeur);
            assert_eq!(lignes, attendu);
            assert!(lignes.iter().all(|l| largeur_valeur(l) <= largeur));
            assert_eq!(lignes.join(" "), rue);
        }
        assert_eq!(couper_ligne("", RECU_LARGEUR), Vec::<String>::new());
    }

    /// **AC3** (Story 33-12b, revue P1), à la largeur depuis la Story 44-1a — les
    /// frontières, là où elles mordent : une jointure qui fait exactement la largeur tient,
    /// un rien de moins de largeur la coupe ; un mot qui fait exactement la largeur tient
    /// seul, le suivant part à la ligne.
    #[test]
    fn couper_ligne_frontieres_en_largeur() {
        let juste = format!("{} {}", "é".repeat(12), "é".repeat(14));
        let l = largeur_valeur(&juste);
        assert_eq!(couper_ligne(&juste, l), std::slice::from_ref(&juste));
        assert_eq!(
            couper_ligne(&juste, l - 0.01),
            ["é".repeat(12), "é".repeat(14)]
        );

        // Une jointure à la largeur au milieu d'une ligne plus longue.
        let deux = "aaaaaaaaaaaaa bbbbbbbbbbbbb";
        assert_eq!(
            couper_ligne(&format!("{deux} cc"), largeur_valeur(deux)),
            [deux, "cc"]
        );

        // Un mot qui fait la largeur tient seul, le suivant part à la ligne.
        let mot = "é".repeat(27);
        assert_eq!(
            couper_ligne(&format!("{mot} x"), largeur_valeur(&mot)),
            [mot, "x".to_string()]
        );
    }

    /// **AC3** (Story 33-12b, revue P1, tranché) — les espaces multiples intérieures
    /// se réduisent à une, sur une ligne courte comme sur une ligne coupée : un seul
    /// chemin, et la jointure redonne l'entrée à ces espaces près. Aucune lettre perdue.
    #[test]
    fn couper_ligne_reduit_les_espaces_multiples() {
        assert_eq!(couper_ligne("Rue  du   Lac", RECU_LARGEUR), ["Rue du Lac"]);
        // 50,5 mm une fois les espaces réduites : une ligne au récépissé.
        assert_eq!(
            couper_ligne("Chemin  des Trois-Fontaines  de la Tour", RECU_LARGEUR),
            ["Chemin des Trois-Fontaines de la Tour"]
        );
        assert_eq!(
            couper_ligne(
                "Chemin  des Trois-Fontaines  de la Vieille Tour",
                RECU_LARGEUR
            ),
            ["Chemin des Trois-Fontaines de la Vieille", "Tour"]
        );
    }

    /// **AC3** (Story 33-12b, C-e3312-6) — la coupe est gloutonne : le NPA peut rester
    /// seul sur sa ligne, l'adresse restant entière ; le préfixe, lui, reste au NPA. À la
    /// largeur (Story 44-1a), il y faut une localité plus longue qu'avant : 59,29 mm.
    #[test]
    fn couper_ligne_npa_peut_rester_seul() {
        assert_eq!(
            couper_ligne("FR-69400 Villefranche-sur-Saône", RECU_LARGEUR),
            ["FR-69400 Villefranche-sur-Saône"]
        );
        assert_eq!(
            couper_ligne(
                "FR-69400 Villefranche-sur-Saône-et-Beaujolais",
                RECU_LARGEUR
            ),
            ["FR-69400", "Villefranche-sur-Saône-et-Beaujolais"]
        );
    }

    /// **AC4** (Story 44-1a) — `raccourcir` : une ligne qui tient sort entière (espaces
    /// réduites) ; sinon, son début le plus long qui laisse la place des trois points ASCII,
    /// sans espace de fin. 18 `W` et « ... » font 47,96 + 2,35 = 50,31 mm, 19 en font 52,97.
    #[test]
    fn raccourcir_laisse_la_place_des_points() {
        assert_eq!(raccourcir("Rue  du Lac", RECU_LARGEUR), "Rue du Lac");
        assert_eq!(
            raccourcir(&"W".repeat(70), RECU_LARGEUR),
            format!("{}...", "W".repeat(18))
        );
        assert_eq!(
            raccourcir(&"W".repeat(19), RECU_LARGEUR),
            "W".repeat(19),
            "19 `W` tiennent sans points"
        );
        // L'espace de fin tombe avant les points.
        assert_eq!(
            raccourcir(
                &format!("DE-{} {}", "W".repeat(16), "W".repeat(35)),
                RECU_LARGEUR
            ),
            format!("DE-{}...", "W".repeat(16))
        );
        for l in [RECU_LARGEUR, INFO_LARGEUR] {
            assert!(largeur_valeur(&raccourcir(&"@".repeat(140), l)) <= l);
        }
    }

    /// **AC1, AC5** (Story 33-12b) et **M13** — aux quatre blocs, créancier et débiteur
    /// distincts : `name`, `rue numéro`, `NPA localité`, dans cet ordre, aux positions
    /// exactes ; au reçu, la référence entre le créancier et *Payable by*. Positions de la
    /// grille de la Story 44-1a : le débiteur passe de x = 67 sous le montant à la colonne
    /// d'information (x = 118) et, au reçu, au-dessus du montant (base 35).
    #[test]
    fn quatre_blocs_impriment_la_composition() {
        let textes = textes_pour(creancier_ordinaire(), debiteur_ordinaire());
        for (x, y, attendu) in [
            (5.0, 84.0, "Robert Schneider SA"),
            (5.0, 80.5, "Rue du Lac 1268"),
            (5.0, 77.0, "2501 Biel"),
            (5.0, 71.0, "Reference"),
            (5.0, 68.0, "00 00000 00004 20000 00000 01008"),
            (5.0, 35.0, "Currency"),
            (5.0, 62.0, "Payable by"),
            (5.0, 58.5, "Pia Rutschmann"),
            (5.0, 55.0, "Marktgasse 28"),
            (5.0, 51.5, "9400 Rorschach"),
            (118.0, 91.5, "Robert Schneider SA"),
            (118.0, 88.0, "Rue du Lac 1268"),
            (118.0, 84.5, "2501 Biel"),
            (118.0, 78.5, "Reference"),
            (118.0, 57.0, "Pia Rutschmann"),
            (118.0, 53.5, "Marktgasse 28"),
            (118.0, 50.0, "9400 Rorschach"),
        ] {
            assert_eq!(texte_a(&textes, x, y), attendu, "à ({x}, {y})");
        }
    }

    /// **AC5** (Story 33-12b) — sans référence, aucune rubrique, ni au reçu ni en
    /// partie paiement. Depuis la Story 44-1a (D6), le montant ne remonte plus : il reste à
    /// la base 35 ; c'est *Payable by* qui remonte d'autant (base 71 au lieu de 62).
    #[test]
    fn sans_reference_aucune_rubrique() {
        let (mut data, invoice, i18n) = invoice_fixture();
        data.creditor_iban = "CH9300762011623852957".into();
        data.reference = Reference::None;
        data.creditor = creancier_ordinaire();
        let pdf = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
        let textes = textes_du_pdf(&pdf);
        assert!(!textes.iter().any(|(_, _, t)| t == "Reference"));
        assert_eq!(texte_a(&textes, 5.0, MONTANT_BASE), "Currency");
        assert_eq!(texte_a(&textes, 5.0, 71.0), "Payable by");
    }

    /// **AC4** (Story 33-12b) et **M11** — une rue longue : `y` exacts, pas de 3,5, bloc
    /// suivant décalé d'autant. Coupée à la largeur depuis la Story 44-1a, la rue de 61
    /// caractères fait deux lignes au reçu (52 mm), au lieu de trois, et une seule dans la
    /// colonne (87 mm), au lieu de deux ; huit lignes d'adresse au reçu : palier R0.
    #[test]
    fn rue_longue_les_y_suivent() {
        let longue = adresse_s(
            "Robert Schneider SA",
            "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau",
            "12",
            "2501",
            "Biel",
        );
        let debiteur = adresse_s(
            "Pia Rutschmann",
            "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau",
            "12",
            "9400",
            "Rorschach",
        );
        let textes = textes_pour(longue, debiteur);
        let rue = "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau 12";
        for (x, y, attendu) in [
            (5.0, 84.0, "Robert Schneider SA"),
            (5.0, 80.5, "Chemin des Trois-Fontaines de la Vieille"),
            (5.0, 77.0, "Tour du Haut-Plateau 12"),
            (5.0, 73.5, "2501 Biel"),
            (5.0, 67.5, "Reference"),
            (5.0, 58.5, "Payable by"),
            (5.0, 55.0, "Pia Rutschmann"),
            (5.0, 51.5, "Chemin des Trois-Fontaines de la Vieille"),
            (5.0, 48.0, "Tour du Haut-Plateau 12"),
            (5.0, 44.5, "9400 Rorschach"),
            (5.0, 35.0, "Currency"),
            (118.0, 88.0, rue),
            (118.0, 84.5, "2501 Biel"),
            (118.0, 78.5, "Reference"),
            (118.0, 69.5, "Additional information"),
            (118.0, 60.5, "Payable by"),
            (118.0, 53.5, rue),
            (118.0, 50.0, "9400 Rorschach"),
        ] {
            assert_eq!(texte_a(&textes, x, y), attendu, "à ({x}, {y})");
        }
    }

    /// **AC2 et AC4** (Story 33-12b) — le pays, créancier et débiteur, reçu et partie
    /// paiement : `LI` compris, jamais `CH`.
    #[test]
    fn le_pays_s_imprime_hors_de_suisse() {
        let textes = textes_pour(
            adresse_etrangere("Vaduz AG", "Städtle", "5", "9490", "Vaduz", "LI"),
            adresse_etrangere("Jan", "ulica Długa", "5", "90-001", "Łódź", "PL"),
        );
        for (x, y, attendu) in [
            (5.0, 77.0, "LI-9490 Vaduz"),
            (118.0, 84.5, "LI-9490 Vaduz"),
            (5.0, 55.0, "ulica Dluga 5"),
            (5.0, 51.5, "PL-90-001 Lódz"),
            (118.0, 50.0, "PL-90-001 Lódz"),
        ] {
            assert_eq!(texte_a(&textes, x, y), attendu, "à ({x}, {y})");
        }
        let suisse = textes_pour(creancier_ordinaire(), debiteur_ordinaire());
        assert!(!suisse.iter().any(|(_, _, t)| t.starts_with("CH-")));
    }

    /// **AC4** (Story 33-12b) — les cinq cas, tous à référence QRR : rien sur le point de
    /// dépôt, et aucune ligne vide dessinée dans la section de paiement. Depuis la Story
    /// 44-1a, le point de dépôt est à la base 20,5 et la section montant sous lui ne s'arrête
    /// plus à y = 5 + un pas : la règle devient le non-recouvrement de sa boîte.
    #[test]
    fn cinq_cas_au_dessus_du_point_de_depot() {
        let rue = "Avenue du Général-Guisan et de la Promenade des Anciens-Bains";
        for (creancier, debiteur) in [
            (creancier_ordinaire(), debiteur_ordinaire()),
            (
                adresse_s("Robert Schneider SA", rue, "1268", "2501", "Biel"),
                adresse_s("Pia Rutschmann", rue, "28", "9400", "Rorschach"),
            ),
            (
                creancier_ordinaire(),
                adresse_etrangere("Jan", "ulica Długa", "5", "90-001", "Łódź", "PL"),
            ),
            (
                adresse_etrangere("Vaduz AG", "Städtle", "5", "9490", "Vaduz", "LI"),
                debiteur_ordinaire(),
            ),
            (
                adresse_s("Robert Schneider SA", "Rue du Lac", "", "2501", "Biel"),
                adresse_s("Pia Rutschmann", "", "", "9400", "Rorschach"),
            ),
        ] {
            let (mut data, invoice, i18n) = invoice_fixture();
            data.creditor = creancier;
            data.ultimate_debtor = Some(debiteur);
            let pdf = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
            let section = textes_de_la_section(&pdf);
            assert!(
                section.iter().any(|t| t.texte == "Reference"),
                "montage : un QRR"
            );
            let depot: Vec<_> = section
                .iter()
                .filter(|t| t.texte == "Acceptance point")
                .map(boite_44)
                .collect();
            assert_eq!(depot.len(), 1, "montage : un point de dépôt");
            for t in &section {
                assert!(!t.texte.is_empty(), "ligne vide à ({}, {})", t.x, t.y);
                if t.texte != "Acceptance point" {
                    assert!(
                        boite_44(t).disjointe(&depot[0]),
                        "{:?} à ({}, {}) sur le point de dépôt {:?}",
                        t.texte,
                        t.x,
                        t.y,
                        depot[0]
                    );
                }
            }
        }
    }

    /// **AC6** (Story 33-12a, #941), repris à la jonction de la 33-12b — sur un PDF
    /// réel, les trois lignes de chacun des quatre blocs (`name`, `rue numéro`,
    /// `NPA localité`, dix champs en tout) et le message impriment la base des lettres
    /// hors WinAnsi.
    #[test]
    fn chaque_site_imprime_la_base_des_lettres() {
        let (mut data, invoice, i18n) = invoice_fixture();
        data.creditor = adresse_s("Čolić", "Łódzka", "1ł", "1003", "Łask");
        data.ultimate_debtor = Some(adresse_s("Ștefan", "Țară", "2ń", "1200", "Żary"));
        data.unstructured_message = Some("Merci Ž ć".into());

        let pdf = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
        let textes = textes_du_pdf(&pdf);

        // Positions de la grille de la Story 44-1a.
        for (x, y, attendu) in [
            (5.0, 84.0, "Colic"),
            (5.0, 80.5, "Lódzka 1l"),
            (5.0, 77.0, "1003 Lask"),
            (5.0, 58.5, "Stefan"),
            (5.0, 55.0, "Tara 2n"),
            (5.0, 51.5, "1200 Zary"),
            (118.0, 91.5, "Colic"),
            (118.0, 88.0, "Lódzka 1l"),
            (118.0, 84.5, "1003 Lask"),
            (118.0, 57.0, "Stefan"),
            (118.0, 53.5, "Tara 2n"),
            (118.0, 50.0, "1200 Zary"),
            (118.0, 66.5, "Merci Ž c"),
        ] {
            assert_eq!(texte_a(&textes, x, y), attendu, "à ({x}, {y})");
        }
    }

    /// Chaque texte dessiné du PDF, avec la couleur de remplissage (`rg`) en vigueur
    /// au moment de son `Tj` : noir par défaut, comme le veut la norme PDF.
    fn couleurs_des_textes(bytes: &[u8]) -> Vec<(String, Vec<f32>)> {
        let mut textes = Vec::new();
        let mut couleur = vec![0.0, 0.0, 0.0];
        for op in operations_du_pdf(bytes) {
            match op.operator.as_str() {
                "rg" => couleur = op.operands.iter().map(nombre).collect(),
                "Tj" => textes.push((texte_du_tj(&op), couleur.clone())),
                _ => {}
            }
        }
        textes
    }

    /// **#941, constaté à la capture de la Story 33-12a** — tout texte s'imprime en
    /// noir. `draw_swiss_cross` laissait le remplissage en blanc : la partie paiement
    /// entière (compte, créancier, référence, message, montant, débiteur) sortait
    /// invisible sur le papier, alors qu'elle était présente au contenu du PDF.
    /// Aucun test relisant le texte ne pouvait le voir.
    #[test]
    fn chaque_texte_s_imprime_en_noir() {
        let (data, invoice, i18n) = invoice_fixture();
        let pdf = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
        let textes = couleurs_des_textes(&pdf);
        assert!(
            textes.iter().any(|(t, _)| t == "Robert Schneider SA"),
            "montage : le créancier est lu"
        );
        let invisibles: Vec<_> = textes
            .iter()
            .filter(|(_, couleur)| couleur.as_slice() != [0.0, 0.0, 0.0])
            .collect();
        assert!(
            invisibles.is_empty(),
            "textes pas en noir : {invisibles:#?}"
        );

        // La croix elle-même : blanc, rouge, blanc, puis seulement le noir. Une remise
        // au noir placée avant ses branches garderait les textes noirs.
        let couleurs: Vec<Vec<i32>> = operations_du_pdf(&pdf)
            .iter()
            .filter(|op| op.operator == "rg")
            .map(|op| {
                op.operands
                    .iter()
                    .map(|o| (nombre(o) * 100.0).round() as i32)
                    .collect()
            })
            .collect();
        assert_eq!(
            couleurs,
            [
                [0, 0, 0],
                [100, 100, 100],
                [85, 0, 0],
                [100, 100, 100],
                [0, 0, 0]
            ]
        );
        // Et le noir vient après les branches, dernier rectangle du document.
        let operateurs: Vec<String> = operations_du_pdf(&pdf)
            .into_iter()
            .map(|op| op.operator)
            .collect();
        let dernier = |nom: &str| operateurs.iter().rposition(|o| o == nom);
        assert!(dernier("rg") > dernier("re"), "noir avant les branches");

        // Les 17 textes qui suivent la croix sont ceux que le défaut effaçait, avec cette
        // fixture : compte, créancier, référence, message, montant, débiteur.
        let apres_la_croix = operateurs[dernier("re").expect("la croix") + 1..]
            .iter()
            .filter(|o| *o == "Tj")
            .count();
        assert_eq!(apres_la_croix, 17);
    }

    /// **Revue P1 de la Story 33-12a (E-M2)** — le nom et l'adresse du corps passent
    /// par `texte_imprime`. U+00A0, U+00AD et U+0218 à U+021B, refusés avant la story,
    /// y atteignent `lopdf`, qui les perd : `MüllerSöhne`, puis le PDF se fige.
    #[test]
    fn corps_imprime_nom_et_adresse_par_texte_imprime() {
        let (mut data, mut invoice, i18n) = invoice_fixture();
        data.ultimate_debtor = Some(debiteur_ordinaire());
        invoice.creditor_name = "Müller\u{A0}Söhne".into();
        invoice.creditor_address_lines = vec!["Rue\u{A0}du Lac 1".into(), "2501 Biel".into()];
        invoice.debtor_name = "Ștefan\u{A0}SRL".into();
        invoice.debtor_address_lines = vec!["Grand\u{A0}Rue 3".into(), "1200 Genève".into()];

        let pdf = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
        let corps: Vec<String> = textes_du_pdf(&pdf)
            .into_iter()
            .filter(|(_, y, _)| *y > SEP_Y)
            .map(|(_, _, t)| t)
            .collect();
        for attendu in ["Müller Söhne", "Rue du Lac 1", "Stefan SRL", "Grand Rue 3"] {
            assert!(corps.iter().any(|t| t == attendu), "{attendu} : {corps:#?}");
        }
    }

    /// **Revues P1 (E-M2) et P2 (P2-M1) de la Story 33-12a** — un avoir ne passe par aucune
    /// validation de jeu. Son texte ordinaire sort inchangé ; U+2019 et U+2014 deviennent
    /// `'` et `-`, comme au code, ce qui change bien l'imprimé d'avant ; tout autre
    /// caractère hors du § 4.1.1 reste traité comme avant, gardé s'il est en WinAnsi, perdu
    /// par `lopdf` sinon.
    #[test]
    fn avoir_texte_ordinaire_inchange() {
        let (_, mut invoice, i18n) = invoice_fixture();
        invoice.creditor_name = "Müller & Söhne SA".into();
        invoice.creditor_address_lines = vec!["Rue du Lac 1268".into(), "L\u{2019}Atelier".into()];
        invoice.debtor_name = "Pia – Rutschmann".into();
        invoice.debtor_address_lines = vec!["AЩB".into(), "Paul\u{2014}Henri".into()];

        let pdf = generate_credit_note_pdf_with_date(&invoice, &i18n, fixed_date()).unwrap();
        let textes: Vec<String> = textes_du_pdf(&pdf).into_iter().map(|(_, _, t)| t).collect();
        for attendu in [
            "Müller & Söhne SA",
            "Rue du Lac 1268",
            "Pia – Rutschmann",
            "AB",
            "L'Atelier",
            "Paul-Henri",
        ] {
            assert!(
                textes.iter().any(|t| t == attendu),
                "{attendu} : {textes:#?}"
            );
        }
    }

    /// **Revue P1 (E-L4)** — un message fait d'invisibles sort vide du code : ni valeur
    /// ni intitulé sur le papier.
    #[test]
    fn message_d_invisibles_sans_intitule() {
        let (mut data, invoice, i18n) = invoice_fixture();
        data.unstructured_message = Some("\u{A0}\u{AD}".into());
        let pdf = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
        let textes = textes_du_pdf(&pdf);
        assert!(
            !textes.iter().any(|(_, _, t)| t == "Additional information"),
            "{textes:#?}"
        );
    }

    /// Date de création figée — sans elle, deux générations successives
    /// diffèrent et un delta de taille ne mesurerait plus rien.
    fn fixed_date() -> printpdf::OffsetDateTime {
        use printpdf::OffsetDateTime;
        OffsetDateTime::from_unix_timestamp(1_767_225_600).expect("timestamp valide")
    }

    /// **AC4** — chacune des trois coordonnées est rendue, **prise isolément**.
    ///
    /// ⚠️ **Une seule assertion « les trois ensemble > aucune » NE DISCRIMINE PAS** :
    /// avec deux champs rendus sur trois, le document reste plus gros que le
    /// témoin et l'assertion passe. Mesuré — la mutation « retirer le rendu du
    /// téléphone » laissait la suite entière verte. D'où **un cas par champ**,
    /// chacun comparé au même témoin sans coordonnées.
    ///
    /// Mesure par **delta de taille d'octets**, pas par recherche de texte : le
    /// dépôt ne compare jamais le contenu d'un PDF (« Plan C », cf.
    /// `tests/golden_test.rs`), et le texte y est **hex-encodé** dans les
    /// opérateurs `Tj` — un `contains` naïf échouerait toujours.
    #[test]
    fn each_contact_detail_is_rendered_on_its_own() {
        let (data, base, i18n) = invoice_fixture();
        let temoin = generate_qr_bill_pdf_with_date(&data, &base, &i18n, fixed_date())
            .unwrap()
            .len();

        for (label, invoice) in [
            (
                "téléphone",
                InvoicePdfData {
                    creditor_phone: Some("+41 21 123 45 67".into()),
                    ..base.clone()
                },
            ),
            (
                "e-mail",
                InvoicePdfData {
                    creditor_email: Some("contact@exemple.ch".into()),
                    ..base.clone()
                },
            ),
            (
                "site web",
                InvoicePdfData {
                    creditor_website: Some("https://exemple.ch".into()),
                    ..base.clone()
                },
            ),
        ] {
            let avec = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date())
                .unwrap()
                .len();
            assert!(
                avec > temoin,
                "le {label} SEUL doit ajouter du contenu au PDF \
                 (témoin sans coordonnées = {temoin} octets, avec = {avec}) — \
                 si cette assertion tombe, ce champ n'est plus rendu"
            );
        }
    }

    /// Une coordonnée **très longue** est TRONQUÉE, elle ne déborde pas sur le
    /// bloc de droite (revue de code, passe 1).
    ///
    /// ⚠️ La garde `HeaderOverflow` ne surveille que l'ordonnée `y` — elle est
    /// aveugle à une ligne **unique** trop longue. Un site web de 255 caractères
    /// (borne de saisie) ou un e-mail de 320 (`VARCHAR(320)`, non borné à la
    /// saisie) s'imprimait par-dessus « Facture »/n°/date, voire hors page, et
    /// le document était **rendu en 200**.
    ///
    /// Le test compare une valeur **au-delà** de la borne d'affichage à une
    /// valeur **juste en deçà** : si la troncature disparaît, la première
    /// produit un document plus gros que la seconde. C'est mesurable en octets
    /// là où une position ne l'est pas.
    #[test]
    fn an_overlong_contact_detail_is_truncated_not_overflowed() {
        let (data, base, i18n) = invoice_fixture();

        // ⚠️ Les DEUX valeurs doivent dépasser le seuil, sinon le test ne
        // mesure rien : une valeur courte n'est pas tronquée et produit
        // légitimement un document plus petit. (Première version de ce test :
        // 40 vs 255 — elle échouait pour cette raison, pas à cause du code.)
        let long = InvoicePdfData {
            creditor_website: Some("x".repeat(100)),
            ..base.clone()
        };
        let tres_long = InvoicePdfData {
            creditor_website: Some("x".repeat(255)),
            ..base.clone()
        };

        let a = generate_qr_bill_pdf_with_date(&data, &long, &i18n, fixed_date())
            .unwrap()
            .len();
        let b = generate_qr_bill_pdf_with_date(&data, &tres_long, &i18n, fixed_date())
            .unwrap()
            .len();

        assert_eq!(
            a, b,
            "deux valeurs au-delà du seuil (100 et 255 caractères) doivent \
             produire des documents de MÊME taille : toutes deux sont ramenées \
             à IDENTITY_MAX_CHARS. Si les tailles diffèrent, la troncature a \
             disparu et la ligne déborde sur le bloc de droite — en silence."
        );
    }

    /// **D2** — une société sans IDE **ni** coordonnées ne doit subir **aucun**
    /// décalage : son document reste celui d'avant la story.
    ///
    /// ⚠️ **Un delta de taille ne peut PAS mesurer ça** : un décalage vertical
    /// de 2 mm déplace le texte sans changer sa longueur, donc le PDF pèse le
    /// même nombre d'octets. Mesuré — une première version de ce test comparait
    /// deux générations entre elles et restait **verte** sous la mutation.
    ///
    /// On mesure donc là où 2 mm changent un **verdict** : au seuil de la garde
    /// haute. Calibrage (`PAGE_H = 297`, plafond `−55`, tableau à `−130`,
    /// marge `+2`) avec 8 lignes d'adresse émetteur — assez pour passer sous le
    /// plafond, sinon le `min()` masquerait l'écart — et 15 lignes côté
    /// destinataire :
    ///
    /// - respiration **conditionnelle** (correcte) → `y = 170.5` ⇒ le document passe ;
    /// - respiration **inconditionnelle** (buguée) → `y = 168.5` ⇒ refus.
    ///
    /// Ce test échoue donc si l'espacement redevient inconditionnel, et **lui
    /// seul** le voit.
    #[test]
    fn no_identity_line_costs_no_vertical_space() {
        let (data, base, i18n) = invoice_fixture();
        let invoice = InvoicePdfData {
            // Aucune ligne d'identité : ni IDE, ni téléphone, ni e-mail, ni site.
            creditor_ide: None,
            creditor_phone: None,
            creditor_email: None,
            creditor_website: None,
            creditor_address_lines: (0..8).map(|i| format!("Adresse {i}")).collect(),
            debtor_address_lines: (0..15).map(|i| format!("Destinataire {i}")).collect(),
            debtor_client_number: None,
            ..base
        };
        let res = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date());
        assert!(
            res.is_ok(),
            "sans AUCUNE ligne d'identité, aucune respiration ne doit être posée : \
             ce document tient à 2 mm près. S'il est refusé, l'espacement est \
             redevenu inconditionnel et TOUTES les sociétés sans coordonnées \
             voient leur PDF décalé — ce que D2 interdit. Erreur : {:?}",
            res.err()
        );
    }

    /// **AC6** — la garde de capacité **haute**. Le cas de test doit
    /// **FRANCHIR le seuil**, pas s'en approcher : l'issue attendue est le refus.
    ///
    /// ⚠️ Un cas généreux mais sous le seuil se rendrait correctement et ne
    /// mesurerait rien — le budget réel est large (le destinataire démarre au
    /// plus haut à `PAGE_H - 55`, le tableau à `PAGE_H - 130`, soit 75 mm).
    /// D'où une adresse émetteur **très** longue, les trois coordonnées, **et**
    /// une adresse destinataire longue.
    ///
    /// Sans la garde, ce document se rendait en `Ok`, avec l'en-tête imprimé
    /// par-dessus les colonnes du tableau.
    #[test]
    fn overlong_header_errors_instead_of_overprinting_the_lines_table() {
        let (data, base, i18n) = invoice_fixture();
        let invoice = InvoicePdfData {
            creditor_address_lines: (0..10).map(|i| format!("Ligne adresse {i}")).collect(),
            creditor_phone: Some("+41 21 123 45 67".into()),
            creditor_email: Some("contact@exemple.ch".into()),
            creditor_website: Some("https://exemple.ch".into()),
            debtor_address_lines: (0..10).map(|i| format!("Ligne destinataire {i}")).collect(),
            debtor_client_number: None,
            ..base
        };
        let err = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date())
            .expect_err("un en-tête débordant DOIT être refusé, pas rendu par-dessus le tableau");
        assert!(
            matches!(err, QrBillError::HeaderOverflow(_)),
            "erreur attendue HeaderOverflow, obtenue : {err:?}"
        );
    }

    fn reminder(settled: Decimal, due: Decimal, fees: Decimal) -> ReminderPdf {
        ReminderPdf {
            amount_settled: settled,
            amount_due: due,
            fees,
        }
    }

    /// Story 25-4-b2 (AC 6, 8) — le CONTENU du bloc, mesuré sur les lignes
    /// construites (le texte d'un PDF ne se compare pas, cf. `golden_test.rs`) :
    /// réglé et reste seulement après un règlement, frais et mention seulement
    /// avec des frais, montants au centime.
    /// Story 25-4-c4-b — la ligne « Arrondi » : signée, en dernier, et elle fait
    /// apparaître le sous-total même sans TVA ; rien sans arrondi ni TVA.
    #[test]
    fn recap_lines_show_the_rounding_line() {
        let (_, base, i18n) = invoice_fixture();
        let labels = |inv: &InvoicePdfData| {
            recap_lines(inv, &i18n)
                .into_iter()
                .map(|l| (l.label, l.amount))
                .collect::<Vec<_>>()
        };
        let sans_tva = InvoicePdfData {
            subtotal_ht: dec!(123.44),
            vat_lines: vec![],
            rounding: dec!(0.01),
            total: dec!(123.45),
            ..base.clone()
        };
        assert_eq!(
            labels(&sans_tva),
            vec![
                ("Subtotal".to_string(), "CHF 123.44".to_string()),
                ("Rounding".to_string(), "CHF +0.01".to_string()),
            ]
        );
        let avec_tva = InvoicePdfData {
            subtotal_ht: dec!(216.95),
            vat_lines: vec![InvoiceVatLinePdf {
                rate_percent: dec!(8.10),
                amount: dec!(17.57),
            }],
            rounding: dec!(-0.02),
            total: dec!(234.50),
            ..base.clone()
        };
        let l = labels(&avec_tva);
        assert_eq!(l.len(), 3);
        assert_eq!(l[2], ("Rounding".to_string(), "CHF -0.02".to_string()));
        let rond = InvoicePdfData {
            vat_lines: vec![],
            rounding: Decimal::ZERO,
            ..base
        };
        assert!(
            labels(&rond).is_empty(),
            "ni TVA ni arrondi : le total seul"
        );
        // Écart infra-centime : pas de ligne « +0.00 » ; un demi-centime s'affiche
        // au centime, signe compris (revue de code P1, lentille A).
        let infra = InvoicePdfData {
            rounding: dec!(0.0004),
            ..rond.clone()
        };
        assert!(
            labels(&infra).is_empty(),
            "0.0004 n'explique rien au centime"
        );
        let demi = InvoicePdfData {
            rounding: dec!(-0.0050),
            ..rond
        };
        assert_eq!(labels(&demi)[1].1, "CHF -0.01");
    }

    /// La ligne d'arrondi compte dans la réserve de la garde `TooManyLines` : une
    /// facture qui tient de justesse sans elle déborde avec elle.
    #[test]
    fn the_rounding_line_counts_in_the_recap_reserve() {
        let (data, base, i18n) = invoice_fixture();
        let fits_at = |n: usize, rounding: Decimal| {
            let lines: Vec<InvoiceLinePdf> = (0..n)
                .map(|i| InvoiceLinePdf {
                    description: format!("Ligne {i}"),
                    quantity: dec!(1),
                    unit_price: dec!(1.00),
                    vat_rate: dec!(0),
                    line_total: dec!(1.00),
                })
                .collect();
            let inv = InvoicePdfData {
                lines,
                subtotal_ht: Decimal::from(n as i64),
                vat_lines: vec![],
                rounding,
                total: Decimal::from(n as i64) + rounding,
                ..base.clone()
            };
            generate_qr_bill_pdf(&data, &inv, &i18n).is_ok()
        };
        // Le plus grand nombre de lignes qui tient sans arrondi…
        let max = (1..60)
            .take_while(|n| fits_at(*n, Decimal::ZERO))
            .last()
            .unwrap();
        assert!(max < 59, "la garde doit finir par refuser");
        // …ne tient plus une fois le sous-total et la ligne d'arrondi réservés.
        let max_rounded = (1..60)
            .take_while(|n| fits_at(*n, dec!(0.01)))
            .last()
            .unwrap();
        assert!(max_rounded < max, "réserve : {max_rounded} < {max}");
    }

    #[test]
    fn reminder_lines_show_only_what_is_not_zero() {
        let i18n = QrBillI18n::default();
        let full = reminder_lines(
            &reminder(dec!(900.00), dec!(181.0000), dec!(20.00)),
            &i18n,
            "CHF",
        );
        let labels: Vec<&str> = full.iter().map(|l| l.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Already paid",
                "Amount due",
                "Reminder fees",
                "Reminder fees are not included in the payment slip."
            ]
        );
        assert_eq!(full[0].amount.as_deref(), Some("-CHF 900.00"));
        assert_eq!(full[1].amount.as_deref(), Some("CHF 181.00"));
        assert!(full[1].bold, "le reste à payer est en gras");
        assert_eq!(full[2].amount.as_deref(), Some("CHF 20.00"));
        assert!(full[3].full_width && full[3].amount.is_none());

        // Q2 : sans règlement, le total EST le reste — ni réglé, ni reste.
        let unpaid = reminder_lines(&reminder(dec!(0), dec!(1081.00), dec!(20.00)), &i18n, "CHF");
        assert_eq!(unpaid.len(), 2);
        assert_eq!(unpaid[0].label, "Reminder fees");
        // Q1 : sans frais, ni ligne de frais, ni mention.
        let no_fees = reminder_lines(&reminder(dec!(900.00), dec!(181.00), dec!(0)), &i18n, "CHF");
        assert_eq!(no_fees.len(), 2);
        assert!(no_fees.iter().all(|l| !l.label.contains("fee")));
        assert!(
            reminder_lines(&reminder(dec!(0), dec!(1081.00), dec!(0)), &i18n, "CHF").is_empty()
        );
    }

    /// Story 25-4-b2 (AC 5) — un rappel se rend, et n'est pas la facture : titre
    /// et bloc ajoutent du texte (delta de taille, date figée).
    #[test]
    fn reminder_pdf_renders_and_differs_from_the_invoice() {
        let (data, base, i18n) = invoice_fixture();
        let facture = generate_qr_bill_pdf_with_date(&data, &base, &i18n, fixed_date())
            .unwrap()
            .len();
        let rappel = InvoicePdfData {
            reminder: Some(reminder(dec!(900.00), dec!(392.40), dec!(20.00))),
            ..base
        };
        let bytes = generate_qr_bill_pdf_with_date(&data, &rappel, &i18n, fixed_date()).unwrap();
        assert!(bytes.starts_with(b"%PDF-1."));
        assert!(
            bytes.len() > facture,
            "le rappel porte titre et bloc : {} ≤ {facture}",
            bytes.len()
        );
    }

    /// Story 25-4-b2 (AC 10) — ⛔ la hauteur du bloc est RÉSERVÉE : il existe un
    /// nombre de lignes que la facture tient et que le rappel complet (trois
    /// lignes et la mention) refuse en `TooManyLines`, au lieu de se tasser sur
    /// la zone de paiement. Et un rappel ne tient jamais là où la facture ne
    /// tient pas.
    #[test]
    fn reminder_block_is_reserved_in_the_capacity_guard() {
        let (data, base, i18n) = invoice_fixture();
        let with_lines = |n: usize, r: Option<ReminderPdf>| InvoicePdfData {
            lines: (0..n)
                .map(|i| InvoiceLinePdf {
                    description: format!("Ligne {i}"),
                    quantity: dec!(1),
                    unit_price: dec!(100.00),
                    vat_rate: dec!(7.70),
                    line_total: dec!(100.00),
                })
                .collect(),
            reminder: r,
            ..base.clone()
        };
        let full = || Some(reminder(dec!(100.00), dec!(50.00), dec!(20.00)));
        let mut discriminated = false;
        for n in 1..=12 {
            let facture_ok = generate_qr_bill_pdf(&data, &with_lines(n, None), &i18n).is_ok();
            let rappel = generate_qr_bill_pdf(&data, &with_lines(n, full()), &i18n);
            if rappel.is_ok() {
                assert!(
                    facture_ok,
                    "{n} lignes : le rappel tient là où la facture ne tient pas"
                );
            } else {
                assert!(matches!(rappel, Err(QrBillError::TooManyLines(m)) if m == n));
                discriminated |= facture_ok;
            }
        }
        assert!(
            discriminated,
            "aucun nombre de lignes ne sépare facture et rappel : la réserve ne sert à rien"
        );
    }

    /// Story 25-4-b2 (AC 8) — la mention de repli tient dans sa largeur. Les
    /// traductions sont bornées côté API, sur les 4 locales.
    #[test]
    fn reminder_fees_note_fallback_fits_its_width() {
        let note = QrBillI18n::default()
            .get("invoice-pdf-reminder-fees-note")
            .to_string();
        assert!(note.chars().count() <= REMINDER_NOTE_MAX_CHARS);
    }

    /// #151 (code-review HIGH) : une facture dont les lignes **plus** le bloc
    /// récap TVA multi-taux ne tiennent pas au-dessus du séparateur QR
    /// (`SEP_Y`) doit être **refusée** (`Err`) plutôt que rendue avec le récap
    /// chevauchant la zone de paiement. La garde de `draw_invoice_section`
    /// réserve la hauteur du récap ; sans ce fix le PDF s'imprimait par-dessus le QR.
    #[test]
    fn many_lines_plus_vat_recap_errors_instead_of_overprinting_qr() {
        let (data, base, i18n) = invoice_fixture();
        // 7 lignes sur 3 taux → réserve récap (3 taux) + lignes dépasse la bande.
        let lines: Vec<InvoiceLinePdf> = (0..7)
            .map(|i| {
                let vat_rate = match i % 3 {
                    0 => dec!(8.10),
                    1 => dec!(2.60),
                    _ => dec!(3.80),
                };
                InvoiceLinePdf {
                    description: format!("Ligne {i}"),
                    quantity: dec!(1),
                    unit_price: dec!(100.00),
                    vat_rate,
                    line_total: dec!(100.00),
                }
            })
            .collect();
        let invoice = InvoicePdfData {
            lines,
            subtotal_ht: dec!(700.00),
            vat_lines: vec![
                InvoiceVatLinePdf {
                    rate_percent: dec!(8.10),
                    amount: dec!(24.30),
                },
                InvoiceVatLinePdf {
                    rate_percent: dec!(3.80),
                    amount: dec!(7.60),
                },
                InvoiceVatLinePdf {
                    rate_percent: dec!(2.60),
                    amount: dec!(5.20),
                },
            ],
            rounding: rust_decimal::Decimal::ZERO,
            total: dec!(744.70),
            ..base
        };
        // Erreur DÉDIÉE `TooManyLines` (→ 400 côté handler), pas un `PdfGeneration`
        // opaque (→ 500). Le handler doit pouvoir donner un message actionnable.
        assert!(
            matches!(
                generate_qr_bill_pdf(&data, &invoice, &i18n),
                Err(QrBillError::TooManyLines(7))
            ),
            "lignes + récap débordant → TooManyLines(7), pas rendu sur le QR ni 500 opaque"
        );
    }

    /// #151 (code-review MEDIUM) : test POSITIF — une facture multi-taux qui
    /// TIENT au-dessus du séparateur QR se rend bien (complète le cas négatif).
    /// 5 lignes / 2 taux : réserve 14.5 → seuil 134.5 → 5 lignes passent.
    #[test]
    fn multi_rate_invoice_that_fits_renders_ok() {
        let (data, base, i18n) = invoice_fixture();
        let lines: Vec<InvoiceLinePdf> = (0..5)
            .map(|i| InvoiceLinePdf {
                description: format!("Ligne {i}"),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: if i % 2 == 0 { dec!(8.10) } else { dec!(2.60) },
                line_total: dec!(100.00),
            })
            .collect();
        let invoice = InvoicePdfData {
            lines,
            subtotal_ht: dec!(500.00),
            vat_lines: vec![
                InvoiceVatLinePdf {
                    rate_percent: dec!(8.10),
                    amount: dec!(24.30),
                },
                InvoiceVatLinePdf {
                    rate_percent: dec!(2.60),
                    amount: dec!(5.20),
                },
            ],
            rounding: rust_decimal::Decimal::ZERO,
            total: dec!(529.50),
            ..base
        };
        let bytes = generate_qr_bill_pdf(&data, &invoice, &i18n).expect("doit tenir et se rendre");
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    /// #151 (code-review HIGH #2) : un avoir n'a PAS de section QR → pleine page.
    /// 12 lignes + récap se rendent (auraient été rejetées par l'ancien cap 9
    /// partagé avec les factures).
    #[test]
    fn credit_note_uses_full_page_beyond_invoice_line_cap() {
        let (_data, base, i18n) = invoice_fixture();
        let lines: Vec<InvoiceLinePdf> = (0..12)
            .map(|i| InvoiceLinePdf {
                description: format!("Prestation créditée {i}"),
                quantity: dec!(1),
                unit_price: dec!(50.00),
                vat_rate: dec!(8.10),
                line_total: dec!(50.00),
            })
            .collect();
        let avoir = InvoicePdfData {
            lines,
            subtotal_ht: dec!(600.00),
            vat_lines: vec![InvoiceVatLinePdf {
                rate_percent: dec!(8.10),
                amount: dec!(48.60),
            }],
            rounding: rust_decimal::Decimal::ZERO,
            total: dec!(648.60),
            ..base
        };
        let bytes = generate_credit_note_pdf(&avoir, &i18n)
            .expect("avoir 12 lignes doit tenir en pleine page (pas de QR)");
        assert!(bytes.starts_with(b"%PDF-1."));
    }

    #[test]
    fn format_ch_swiss_thousands() {
        assert_eq!(format_ch(dec!(1234.50), 2), "1'234.50");
        assert_eq!(format_ch(dec!(1234567.89), 2), "1'234'567.89");
        assert_eq!(format_ch(dec!(0), 2), "0.00");
    }

    #[test]
    fn rounding_half_up_away_from_zero() {
        assert_eq!(format_ch(dec!(1234.5650), 2), "1'234.57");
        assert_eq!(format_ch(dec!(1234.5649), 2), "1'234.56");
        assert_eq!(format_ch(dec!(1234.5050), 2), "1'234.51");
    }

    #[test]
    fn format_iban_groups_by_four() {
        assert_eq!(
            format_iban("CH4431999123000889012"),
            "CH44 3199 9123 0008 8901 2"
        );
    }

    #[test]
    fn format_qrr_groups() {
        let q = "210000000000031394714300098";
        assert_eq!(format_qrr(q), "21 00000 00000 03139 47143 00098");
    }

    #[test]
    fn date_formatting_swiss() {
        assert_eq!(
            format_date_ch(NaiveDate::from_ymd_opt(2026, 4, 7).unwrap()),
            "07.04.2026"
        );
    }

    // -----------------------------------------------------------------------
    // Story 16-3b (#151) — numéro de client dans le bloc métadonnées
    // -----------------------------------------------------------------------

    /// Ordonnée de la ligne dont le **libellé** est `label`, dans les lignes
    /// construites par `build_meta_lines`.
    fn meta_line_y(lines: &[(String, f32)], label: &str) -> f32 {
        lines
            .iter()
            .find(|(text, _)| text.starts_with(label))
            .unwrap_or_else(|| panic!("ligne « {label} » absente de {lines:?}"))
            .1
    }

    /// **AC6, présence** — deux rendus, l'un avec le numéro, l'autre sans ; la
    /// différence de taille atteste que quelque chose a bien été dessiné.
    ///
    /// ⚠️ Ce test ne prouve **que** la présence. Le delta de taille est aveugle
    /// au décalage vertical — c'est écrit noir sur blanc plus haut dans ce
    /// fichier, et **mesuré** : une première version d'un test de ce genre
    /// restait verte sous la mutation. La conditionnalité est prouvée par le
    /// test suivant, sur la fonction pure.
    #[test]
    fn client_number_line_is_drawn_when_present() {
        let (data, base, i18n) = invoice_fixture();
        let without = generate_qr_bill_pdf_with_date(
            &data,
            &InvoicePdfData {
                debtor_client_number: None,
                ..base.clone()
            },
            &i18n,
            fixed_date(),
        )
        .expect("rendu sans numéro");
        let with = generate_qr_bill_pdf_with_date(
            &data,
            &InvoicePdfData {
                debtor_client_number: Some("CLI-2026-00042".into()),
                ..base
            },
            &i18n,
            fixed_date(),
        )
        .expect("rendu avec numéro");
        assert!(
            with.len() > without.len(),
            "le PDF portant le numéro doit être plus lourd : {} vs {}",
            with.len(),
            without.len()
        );
    }

    /// **AC6, vacuité** — une valeur qui ne marque rien sur la page ne doit
    /// dessiner aucune ligne, ni consommer de `META_LINE_STEP`.
    ///
    /// ⚠️ Les caractères de largeur nulle **survivent à `trim()`** : la propriété
    /// Unicode `White_Space` ne les couvre pas, si bien que la normalisation de
    /// la route API rend `Some("\u{200B}")` et non `None`. Sans filtre de
    /// vacuité ici, le PDF porte un « N° client: » suivi de rien qui décale
    /// vers le bas la date, la référence d'origine et l'échéance.
    ///
    /// La chaîne vide franche couvre l'autre porte d'entrée : une valeur écrite
    /// hors API — restauration d'une sauvegarde produite ailleurs, correction
    /// SQL directe — que la route n'a jamais normalisée.
    ///
    /// *(Passe 1 de `bmad-code-review` : le site jumeau du bloc gauche, posé par
    /// la 16-3a, portait déjà ce filtre ; le symptôme n'avait pas été propagé.)*
    #[test]
    fn a_blank_or_invisible_client_number_draws_no_line() {
        let (_, base, i18n) = invoice_fixture();
        let top = 100.0_f32;

        let reference = build_meta_lines(
            &InvoicePdfData {
                debtor_client_number: None,
                ..base.clone()
            },
            &i18n,
            top,
        );

        for (label, value) in [
            ("chaîne vide", ""),
            ("espaces ASCII", "   "),
            ("ZWSP U+200B", "\u{200B}"),
            ("BOM U+FEFF", "\u{FEFF}"),
            ("word joiner U+2060", "\u{2060}"),
            ("espace insécable", "\u{00A0}"),
        ] {
            let lines = build_meta_lines(
                &InvoicePdfData {
                    debtor_client_number: Some(value.into()),
                    ..base.clone()
                },
                &i18n,
                top,
            );
            assert_eq!(
                lines.len(),
                reference.len(),
                "« {label} » ne doit ajouter aucune ligne au bloc"
            );
            assert_eq!(
                lines.last().map(|(_, y)| *y),
                reference.last().map(|(_, y)| *y),
                "« {label} » ne doit consommer aucun META_LINE_STEP"
            );
        }
    }

    /// **AC6, conditionnalité** — l'invariant que le delta de taille ne peut PAS
    /// voir, et la raison d'être de l'extraction de `build_meta_lines`.
    ///
    /// ⚠️ **PAS « les deux ordonnées sont identiques » — c'est la signature du
    /// MUTANT.** Avec un code correct, la ligne « échéance » est
    /// `META_LINE_STEP` plus **basse** quand le numéro est présent ; c'est la
    /// mutation (`*y -= META_LINE_STEP` sorti du conditionnel) qui rend les deux
    /// égales. Une assertion d'égalité échouerait sur du bon code et pousserait
    /// à implémenter la mutation pour la verdir.
    ///
    /// ⚠️ **Ni une ordonnée absolue codée en dur** : elle dépend du fixture,
    /// alors que le delta est invariant.
    #[test]
    fn client_number_line_costs_exactly_one_step_and_nothing_when_absent() {
        let (_, base, i18n) = invoice_fixture();
        let top = 100.0_f32;

        let lines_none = build_meta_lines(
            &InvoicePdfData {
                debtor_client_number: None,
                ..base.clone()
            },
            &i18n,
            top,
        );
        let lines_some = build_meta_lines(
            &InvoicePdfData {
                debtor_client_number: Some("CLI-1".into()),
                ..base
            },
            &i18n,
            top,
        );

        let due = i18n.get("invoice-pdf-due-date");
        let y_none = meta_line_y(&lines_none, due);
        let y_some = meta_line_y(&lines_some, due);
        assert_eq!(
            y_none - y_some,
            META_LINE_STEP,
            "la ligne du numéro doit coûter exactement un pas ; un delta nul est \
             la signature de `*y -= META_LINE_STEP` sorti du conditionnel"
        );

        assert_eq!(
            lines_some.len(),
            lines_none.len() + 1,
            "une ligne de plus, et une seule"
        );
        assert!(
            !lines_none
                .iter()
                .any(|(t, _)| t.starts_with(i18n.get("invoice-pdf-client-number"))),
            "aucune ligne de numéro ne doit être dessinée quand il est absent"
        );
    }

    /// **AC6, position** — entre le n° de facture et la date (D3). Aucun test de
    /// delta ne pourrait rattraper un mauvais point d'insertion : tout
    /// emplacement au-dessus de l'échéance donne le même écart.
    #[test]
    fn client_number_line_sits_between_invoice_number_and_date() {
        let (_, base, i18n) = invoice_fixture();
        let lines = build_meta_lines(
            &InvoicePdfData {
                debtor_client_number: Some("CLI-1".into()),
                ..base
            },
            &i18n,
            100.0,
        );
        let labels: Vec<&str> = lines
            .iter()
            .map(|(t, _)| t.split(':').next().unwrap())
            .collect();
        let pos = |k: &'static str| {
            labels
                .iter()
                .position(|l| *l == i18n.get(k))
                .unwrap_or_else(|| panic!("clé {k} absente de {labels:?}"))
        };
        assert_eq!(
            pos("invoice-pdf-client-number"),
            pos("invoice-pdf-number") + 1
        );
        assert_eq!(
            pos("invoice-pdf-date"),
            pos("invoice-pdf-client-number") + 1
        );
    }

    /// **AC6-bis** — un numéro trop long est tronqué, jamais débordé, et la
    /// borne porte sur la **ligne complète**, libellé compris.
    ///
    /// C'est le piège le plus coûteux de la story : tronquer la seule valeur
    /// puis formater le libellé autour produit une ligne d'environ
    /// 43 caractères ≈ 92 mm depuis `x = 120`, soit `x ≈ 212` sur une page de
    /// **210** — du texte hors feuille. Le test le mesure au caractère près.
    #[test]
    fn an_overlong_client_number_is_truncated_on_the_whole_line() {
        let (_, base, i18n) = invoice_fixture();
        let build = |value: &str| {
            let lines = build_meta_lines(
                &InvoicePdfData {
                    debtor_client_number: Some(value.into()),
                    ..base.clone()
                },
                &i18n,
                100.0,
            );
            meta_line_text(&lines, i18n.get("invoice-pdf-client-number"))
        };

        let long = build(&"X".repeat(80));
        assert_eq!(
            long.chars().count(),
            META_MAX_CHARS,
            "la LIGNE COMPLÈTE doit être bornée, pas la seule valeur : « {long} »"
        );
        assert!(
            long.starts_with(i18n.get("invoice-pdf-client-number")),
            "le libellé doit survivre à la troncature : « {long} »"
        );

        // Cohérence : deux valeurs longues différentes donnent la même taille.
        assert_eq!(
            long.chars().count(),
            build(&"Y".repeat(120)).chars().count()
        );

        // Et une valeur courte n'est pas touchée.
        let short = build("CLI-1");
        assert!(short.ends_with("CLI-1"), "« {short} »");
    }

    /// Les autres lignes du bloc ne sont **pas** bornées à `META_MAX_CHARS`, et
    /// c'est délibéré : borner à 32 couperait « Réf. facture d'origine:
    /// F-2026-0042 » sur tout avoir français.
    ///
    /// ⚠️ Ce test fixe l'absence de borne **à 32**, pas l'absence de toute
    /// borne. Le numéro de facture peut atteindre 64 caractères via un schéma
    /// de numérotation configuré, et déborder de la page — problème antérieur à
    /// cette story, suivi dans l'issue **#293**. Le jour où une borne propre aux
    /// lignes système sera posée, ce test devra être ajusté, pas supprimé : ce
    /// qu'il protège, c'est que la borne du numéro de client ne leur soit pas
    /// appliquée telle quelle.
    #[test]
    fn system_generated_meta_lines_are_not_truncated() {
        let (_, base, i18n) = invoice_fixture();
        let origin = "Réf. facture d'origine très longue F-2026-0042";
        let lines = build_meta_lines(
            &InvoicePdfData {
                origin_reference: Some(origin.into()),
                reminder: None,
                ..base
            },
            &i18n,
            100.0,
        );
        let line = meta_line_text(&lines, i18n.get("invoice-pdf-origin-reference"));
        assert!(
            line.ends_with(origin),
            "la référence d'origine ne doit pas être tronquée : « {line} »"
        );
    }

    /// Texte de la ligne dont le libellé est `label`.
    fn meta_line_text(lines: &[(String, f32)], label: &str) -> String {
        lines
            .iter()
            .find(|(text, _)| text.starts_with(label))
            .unwrap_or_else(|| panic!("ligne « {label} » absente de {lines:?}"))
            .0
            .clone()
    }

    /// **AC8** — appariement **positionnel** de `I18N_KEYS` et `DEFAULT_EN`.
    ///
    /// L'assertion de compilation de `types.rs` ne couvre que les **longueurs** :
    /// décaler une entrée d'un cran les laisse égales, passe `cargo build`, et
    /// décale silencieusement le repli de toutes les entrées suivantes. Ce test
    /// résout la clé sur un bundle **vide** (donc via `DEFAULT_EN`) et compare
    /// au libellé attendu.
    #[test]
    fn client_number_key_resolves_to_its_own_english_default() {
        let empty = QrBillI18n::default();
        assert_eq!(empty.get("invoice-pdf-client-number"), "Client no.");
        // Les voisines immédiates, pour attraper un décalage d'un cran.
        assert_eq!(empty.get("invoice-pdf-website"), "Web");
        assert_eq!(
            empty.get("invoice-pdf-origin-reference"),
            "Original invoice"
        );
    }

    // ----- Story 36-12a (#944) : lignes de séparation, ciseaux, point de dépôt -----
    //
    // Définitions de la fiche (`36-12a-qr-facture-lignes-ciseaux.md`, « Définitions des
    // tests ») : un *segment de pointillé* est un chemin d'un `m` et d'un seul `l`, d'au
    // plus 2,05 mm, couché sur y = 105 ou x = 62 ; un *symbole de ciseaux* est l'union des
    // boîtes des autres chemins tracés de la section de paiement (boîte dans y ≤ 108), hors
    // `re`, réparti par le centre de boîte (x < 57 : horizontal ; x ∈ [57, 67] : vertical) ;
    // la *boîte d'un texte* va de `Td.x` à `Td.x + largeur + 0,5`, et de la ligne de base
    // − 0,228 em à + 0,962 em (`FontBBox` de `phvb8a.afm`).

    /// Millimètres par point : le contenu du PDF s'écrit en points.
    const MM_PAR_PT: f32 = 25.4 / 72.0;

    /// Une boîte fermée, en mm.
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Boite {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    }

    impl Boite {
        /// Deux boîtes fermées d'intersection vide (inégalités strictes).
        fn disjointe(&self, autre: &Boite) -> bool {
            self.x1 < autre.x0 || autre.x1 < self.x0 || self.y1 < autre.y0 || autre.y1 < self.y0
        }
        fn union(&self, autre: &Boite) -> Boite {
            Boite {
                x0: self.x0.min(autre.x0),
                y0: self.y0.min(autre.y0),
                x1: self.x1.max(autre.x1),
                y1: self.y1.max(autre.y1),
            }
        }
        fn elargie(&self, d: f32) -> Boite {
            Boite {
                x0: self.x0 - d,
                y0: self.y0 - d,
                x1: self.x1 + d,
                y1: self.y1 + d,
            }
        }
        fn centre(&self) -> (f32, f32) {
            ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
        }
        fn largeur(&self) -> f32 {
            self.x1 - self.x0
        }
        fn hauteur(&self) -> f32 {
            self.y1 - self.y0
        }
    }

    /// Un chemin tracé du contenu : ses opérateurs de construction (`m`, `l`, `c`, `v`,
    /// `y`), tous ses points en mm (points de contrôle compris, qui bornent une courbe de
    /// Bézier), l'opérateur qui le peint, et la couleur de trait (`RG`) en vigueur.
    #[derive(Debug, Clone)]
    struct Chemin {
        operateurs: Vec<String>,
        points: Vec<(f32, f32)>,
        peinture: String,
        trait_rg: Vec<f32>,
    }

    impl Chemin {
        fn boite(&self) -> Boite {
            let (x, y) = self.points[0];
            self.points.iter().fold(
                Boite {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                },
                |b, &(x, y)| {
                    b.union(&Boite {
                        x0: x,
                        y0: y,
                        x1: x,
                        y1: y,
                    })
                },
            )
        }
    }

    /// Les chemins tracés d'un PDF, hors rectangles (`re` : modules du QR, croix suisse).
    /// Un chemin va d'un `m` à son opérateur de peinture (`S`, `s`, ou tout remplissage).
    fn chemins_du_pdf(bytes: &[u8]) -> Vec<Chemin> {
        let mut chemins = Vec::new();
        let mut trait_rg = vec![0.0, 0.0, 0.0];
        let mut operateurs: Vec<String> = Vec::new();
        let mut points: Vec<(f32, f32)> = Vec::new();
        let mut rectangle = false;
        for op in operations_du_pdf(bytes) {
            match op.operator.as_str() {
                "RG" => trait_rg = op.operands.iter().map(nombre).collect(),
                o @ ("m" | "l" | "c" | "v" | "y") => {
                    operateurs.push(o.to_string());
                    for paire in op.operands.chunks(2) {
                        points.push((nombre(&paire[0]) * MM_PAR_PT, nombre(&paire[1]) * MM_PAR_PT));
                    }
                }
                "re" => rectangle = true,
                o @ ("S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" | "n") => {
                    if !operateurs.is_empty() && !rectangle {
                        chemins.push(Chemin {
                            operateurs: std::mem::take(&mut operateurs),
                            points: std::mem::take(&mut points),
                            peinture: o.to_string(),
                            trait_rg: trait_rg.clone(),
                        });
                    }
                    operateurs.clear();
                    points.clear();
                    rectangle = false;
                }
                _ => {}
            }
        }
        chemins
    }

    /// Les chemins de la section de paiement : boîte dans y ≤ 108 (les filets du corps de
    /// la facture, `hline`, en sont exclus).
    fn chemins_de_la_section(bytes: &[u8]) -> Vec<Chemin> {
        chemins_du_pdf(bytes)
            .into_iter()
            .filter(|c| c.boite().y1 <= 108.0)
            .collect()
    }

    /// L'étendue d'un segment de pointillé le long de sa ligne : x sur l'horizontale
    /// (`horizontale`), y sur la verticale ; `None` si le chemin n'en est pas un.
    fn segment(c: &Chemin, horizontale: bool) -> Option<(f32, f32)> {
        if c.operateurs != ["m", "l"] {
            return None;
        }
        let ((x0, y0), (x1, y1)) = (c.points[0], c.points[1]);
        if ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() > 2.05 {
            return None;
        }
        let pres = |a: f32, b: f32| (a - b).abs() < 0.01;
        if horizontale && pres(y0, SEP_Y) && pres(y1, SEP_Y) {
            Some((x0.min(x1), x0.max(x1)))
        } else if !horizontale && pres(x0, RECEIPT_W) && pres(x1, RECEIPT_W) {
            Some((y0.min(y1), y0.max(y1)))
        } else {
            None
        }
    }

    /// Ce que la section de paiement d'un PDF porte de lignes et de ciseaux.
    #[derive(Debug)]
    struct Releve {
        horizontaux: Vec<(f32, f32)>,
        verticaux: Vec<(f32, f32)>,
        ciseau_h: Option<Boite>,
        ciseau_v: Option<Boite>,
    }

    fn relever(bytes: &[u8]) -> Releve {
        let mut r = Releve {
            horizontaux: Vec::new(),
            verticaux: Vec::new(),
            ciseau_h: None,
            ciseau_v: None,
        };
        let joindre_boite = |acc: &mut Option<Boite>, b: Boite| {
            *acc = Some(acc.map_or(b, |a| a.union(&b)));
        };
        for c in chemins_de_la_section(bytes) {
            if let Some(s) = segment(&c, true) {
                r.horizontaux.push(s);
            } else if let Some(s) = segment(&c, false) {
                r.verticaux.push(s);
            } else {
                let b = c.boite();
                let (cx, _) = b.centre();
                if cx < 57.0 {
                    joindre_boite(&mut r.ciseau_h, b);
                } else if cx <= 67.0 {
                    joindre_boite(&mut r.ciseau_v, b);
                }
            }
        }
        r
    }

    /// Les trous d'une ligne de pointillé couvrant [0, `fin`] : de 0 au premier segment,
    /// entre deux segments, et du dernier à `fin`.
    fn trous(segments: &[(f32, f32)], fin: f32) -> Vec<(f32, f32)> {
        let mut tries = segments.to_vec();
        tries.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut trous = Vec::new();
        let mut atteint = 0.0_f32;
        for (a, b) in tries {
            if a > atteint {
                trous.push((atteint, a));
            }
            atteint = atteint.max(b);
        }
        if fin > atteint {
            trous.push((atteint, fin));
        }
        trous
    }

    /// Un texte dessiné, avec sa taille et sa graisse (lue au `BaseFont` de la police).
    #[derive(Debug)]
    struct TexteMesure {
        x: f32,
        y: f32,
        texte: String,
        taille: f32,
        gras: bool,
    }

    impl TexteMesure {
        /// La boîte du texte, selon la définition de la fiche.
        fn boite(&self) -> Boite {
            Boite {
                x0: self.x,
                y0: self.y - 0.228 * self.taille * MM_PAR_PT,
                x1: self.x + crate::largeurs::largeur_mm(&self.texte, self.taille, self.gras) + 0.5,
                y1: self.y + 0.962 * self.taille * MM_PAR_PT,
            }
        }
    }

    fn textes_mesures(bytes: &[u8]) -> Vec<TexteMesure> {
        use printpdf::lopdf::Document;
        let doc = Document::load_mem(bytes).expect("PDF relisible");
        // printpdf écrit `/Resources` et son `/Font` en objets indirects, que
        // `Document::get_page_fonts` ne suit pas : on les déréférence ici.
        let dictionnaire = |o: &printpdf::lopdf::Object| {
            doc.dereference(o)
                .and_then(|(_, o)| o.as_dict())
                .expect("dictionnaire")
                .clone()
        };
        let mut gras_par_police = std::collections::HashMap::new();
        for page in doc.get_pages().values() {
            let page = doc.get_dictionary(*page).expect("page");
            let ressources = dictionnaire(page.get(b"Resources").expect("Resources"));
            let polices = dictionnaire(ressources.get(b"Font").expect("Font"));
            for (nom, police) in polices.iter() {
                let police = dictionnaire(police);
                let base = police
                    .get(b"BaseFont")
                    .and_then(|o| o.as_name_str())
                    .expect("BaseFont")
                    .to_string();
                gras_par_police.insert(nom.clone(), base.ends_with("-Bold"));
            }
        }
        assert_eq!(gras_par_police.len(), 2, "montage : Helvetica et sa grasse");
        let mut textes = Vec::new();
        let (mut police, mut taille, mut position) = (Vec::new(), 0.0, (0.0, 0.0));
        for op in operations_du_pdf(bytes) {
            match op.operator.as_str() {
                "Tf" => {
                    police = op.operands[0].as_name().expect("nom de police").to_vec();
                    taille = nombre(&op.operands[1]);
                }
                "Td" => {
                    position = (
                        nombre(&op.operands[0]) * MM_PAR_PT,
                        nombre(&op.operands[1]) * MM_PAR_PT,
                    );
                }
                "Tj" => textes.push(TexteMesure {
                    x: position.0,
                    y: position.1,
                    texte: texte_du_tj(&op),
                    taille,
                    gras: gras_par_police[&police],
                }),
                _ => {}
            }
        }
        textes
    }

    /// Les traductions des quatre langues de Kesh, lues aux `messages.ftl` de `kesh-i18n` :
    /// ce que la production pose dans [`QrBillI18n`].
    const LANGUES: [(&str, &str); 4] = [
        (
            "fr",
            include_str!("../../kesh-i18n/locales/fr-CH/messages.ftl"),
        ),
        (
            "de",
            include_str!("../../kesh-i18n/locales/de-CH/messages.ftl"),
        ),
        (
            "en",
            include_str!("../../kesh-i18n/locales/en-CH/messages.ftl"),
        ),
        (
            "it",
            include_str!("../../kesh-i18n/locales/it-CH/messages.ftl"),
        ),
    ];

    /// Le [`QrBillI18n`] d'une langue de [`LANGUES`], par [`QrBillI18n::depuis_ftl`].
    fn i18n_de(langue: &str) -> QrBillI18n {
        let (_, ftl) = LANGUES
            .iter()
            .find(|(l, _)| *l == langue)
            .expect("langue connue");
        QrBillI18n::depuis_ftl(ftl)
    }

    /// Le PDF de la fixture, adresses ordinaires de type S, dans une langue.
    fn pdf_ordinaire(i18n: &QrBillI18n) -> Vec<u8> {
        let (mut data, invoice, _) = invoice_fixture();
        data.creditor = creancier_ordinaire();
        data.ultimate_debtor = Some(debiteur_ordinaire());
        generate_qr_bill_pdf_with_date(&data, &invoice, i18n, fixed_date()).unwrap()
    }

    /// Les PDF ordinaires en français et en anglais, nommés.
    fn pdfs_fr_en() -> Vec<(&'static str, Vec<u8>)> {
        ["fr", "en"]
            .into_iter()
            .map(|l| (l, pdf_ordinaire(&i18n_de(l))))
            .collect()
    }

    /// **AC1** (Story 36-12a) — la ligne horizontale : des traits sur y = 105 de 0 à 210 ;
    /// le seul trou de plus de 2,5 mm est sous le ciseau horizontal (boîte élargie de
    /// 0,5 mm) ; aucun trait ne rencontre la boîte d'un ciseau.
    #[test]
    fn la_ligne_horizontale_couvre_la_largeur() {
        for (langue, pdf) in pdfs_fr_en() {
            let r = relever(&pdf);
            let ciseau = r.ciseau_h.unwrap_or_else(|| {
                panic!("{langue} : la ligne s'interrompt sous un ciseau : {r:?}")
            });
            let grands: Vec<_> = trous(&r.horizontaux, PAGE_W)
                .into_iter()
                .filter(|(a, b)| b - a > 2.5)
                .collect();
            let e = ciseau.elargie(0.5);
            assert_eq!(
                grands.len(),
                1,
                "{langue} : un seul trou, sous le ciseau : {grands:?}"
            );
            for (a, b) in &grands {
                assert!(
                    *a >= e.x0 && *b <= e.x1,
                    "{langue} : trou {a}..{b} hors de {e:?}"
                );
            }
            for &(a, b) in &r.horizontaux {
                // Dans la page (revue P1, A-5), à l'arrondi des points près.
                assert!(
                    a >= -0.01 && b <= PAGE_W + 0.01,
                    "{langue} : trait {a}..{b} hors de [0, 210]"
                );
                let trait_ = Boite {
                    x0: a,
                    y0: SEP_Y,
                    x1: b,
                    y1: SEP_Y,
                };
                // À plus de 0,2 mm d'un ciseau : l'écart de 0,25 mm de C-36-12a-1, que
                // l'arrondi des points ne fait pas passer sous 0,2 (revue P1, B-4).
                for c in [r.ciseau_h, r.ciseau_v].into_iter().flatten() {
                    assert!(
                        trait_.elargie(0.2).disjointe(&c),
                        "{langue} : trait {a}..{b} à moins de 0,2 mm de {c:?}"
                    );
                }
            }
        }
    }

    /// **AC2** (Story 36-12a) — la ligne verticale : des traits sur x = 62 de 0 à 105, même
    /// règle avec le ciseau vertical.
    #[test]
    fn une_ligne_verticale_separe_le_recepisse() {
        for (langue, pdf) in pdfs_fr_en() {
            let r = relever(&pdf);
            assert!(!r.verticaux.is_empty(), "{langue} : aucun trait sur x = 62");
            let ciseau = r.ciseau_v.unwrap_or_else(|| {
                panic!("{langue} : la ligne s'interrompt sous un ciseau : {r:?}")
            });
            let grands: Vec<_> = trous(&r.verticaux, SEP_Y)
                .into_iter()
                .filter(|(a, b)| b - a > 2.5)
                .collect();
            let e = ciseau.elargie(0.5);
            assert_eq!(
                grands.len(),
                1,
                "{langue} : un seul trou, sous le ciseau : {grands:?}"
            );
            for (a, b) in &grands {
                assert!(
                    *a >= e.y0 && *b <= e.y1,
                    "{langue} : trou {a}..{b} hors de {e:?}"
                );
            }
            for &(a, b) in &r.verticaux {
                // Du bas de la page à l'horizontale, pas au-delà (revue P1, A-5).
                assert!(
                    a >= -0.01 && b <= SEP_Y + 0.01,
                    "{langue} : trait {a}..{b} hors de [0, 105]"
                );
                let trait_ = Boite {
                    x0: RECEIPT_W,
                    y0: a,
                    x1: RECEIPT_W,
                    y1: b,
                };
                for c in [r.ciseau_h, r.ciseau_v].into_iter().flatten() {
                    assert!(
                        trait_.elargie(0.2).disjointe(&c),
                        "{langue} : trait {a}..{b} à moins de 0,2 mm de {c:?}"
                    );
                }
            }
        }
    }

    /// **Revue P1, B-2** — un libellé qui porte une espace insécable : `lopdf` la retire (pas
    /// d'octet WinAnsi), et l'intitulé imprimé, « Acceptancepoint », finit toujours à x = 57.
    /// Largeur calculée à la main : 8335 − 278 (l'espace) = 8057 millièmes d'em à 6 pt gras,
    /// 17,054 mm, d'où `Td.x` = 39,946.
    #[test]
    fn le_point_de_depot_ignore_ce_qui_ne_s_imprime_pas() {
        let mut entries = std::collections::HashMap::new();
        entries.insert(
            "invoice-pdf-qr-acceptance-point",
            "Acceptance\u{A0}point".to_string(),
        );
        let textes = textes_du_pdf(&pdf_ordinaire(&QrBillI18n::new(entries)));
        let trouves: Vec<_> = textes
            .iter()
            .filter(|(_, _, t)| t == "Acceptancepoint")
            .collect();
        assert_eq!(
            trouves.len(),
            1,
            "montage : l'espace insécable retirée : {trouves:?}"
        );
        let (x, y, _) = trouves[0];
        assert!((x - 39.946).abs() <= 0.05, "Td.x = {x}, attendu 39,946");
        // Base 20,5 depuis la Story 44-1a (5 avant) ; l'abscisse ne change pas.
        assert!((y - 20.5).abs() < 0.01, "base {y}");
    }

    /// **AC3** (Story 36-12a) — un symbole sur chaque ligne, de 3 à 5 mm de long et 3,5 mm
    /// de large au plus : l'horizontal centré sur y = 105, centre x ∈ [45, 52] ; le vertical
    /// tourné d'un quart de tour, centré sur x = 62, centre y ∈ [101, 102].
    #[test]
    fn des_ciseaux_sur_chaque_ligne() {
        for (langue, pdf) in pdfs_fr_en() {
            let r = relever(&pdf);
            let h = r
                .ciseau_h
                .unwrap_or_else(|| panic!("{langue} : pas de ciseau horizontal"));
            let v = r
                .ciseau_v
                .unwrap_or_else(|| panic!("{langue} : pas de ciseau vertical"));
            for (nom, b) in [("horizontal", h), ("vertical", v)] {
                let grande = b.largeur().max(b.hauteur());
                let petite = b.largeur().min(b.hauteur());
                assert!(
                    (3.0..=5.0).contains(&grande) && petite <= 3.5,
                    "{langue} : ciseau {nom} de {grande} × {petite} mm"
                );
            }
            assert!(
                h.largeur() > h.hauteur(),
                "{langue} : horizontal couché : {h:?}"
            );
            assert!(
                v.hauteur() > v.largeur(),
                "{langue} : vertical debout : {v:?}"
            );
            let (cx, cy) = h.centre();
            assert!(
                (cy - SEP_Y).abs() <= 0.5 && (45.0..=52.0).contains(&cx),
                "{langue} : ciseau horizontal centré à ({cx}, {cy})"
            );
            let (cx, cy) = v.centre();
            assert!(
                (cx - RECEIPT_W).abs() <= 0.5 && (101.0..=102.0).contains(&cy),
                "{langue} : ciseau vertical centré à ({cx}, {cy})"
            );
        }
    }

    /// **AC4** (Story 36-12a) — les ciseaux ne touchent ni un texte ni le QR élargi de sa
    /// zone de 5 mm (§ 3.5.2) — le QR descendu de 5 mm par la Story 44-1a (de 47..93 à
    /// 42..88), sa zone élargie va de 37 à 93 ; sans cette mise à jour le test passerait à vide
    /// sur l'ancienne zone.
    #[test]
    fn les_ciseaux_ne_touchent_rien() {
        let qr = Boite {
            x0: 62.0,
            y0: 37.0,
            x1: 118.0,
            y1: 93.0,
        };
        for (langue, pdf) in pdfs_fr_en() {
            let r = relever(&pdf);
            let ciseaux = [
                r.ciseau_h
                    .unwrap_or_else(|| panic!("{langue} : pas de ciseau horizontal")),
                r.ciseau_v
                    .unwrap_or_else(|| panic!("{langue} : pas de ciseau vertical")),
            ];
            let textes = textes_mesures(&pdf);
            assert!(textes.len() > 30, "montage : les textes sont lus");
            for c in ciseaux {
                assert!(c.disjointe(&qr), "{langue} : {c:?} sur le QR");
                for t in &textes {
                    assert!(
                        c.disjointe(&t.boite()),
                        "{langue} : {c:?} sur {:?} ({:?})",
                        t.texte,
                        t.boite()
                    );
                }
            }
        }
    }

    /// **AC5** (Story 36-12a) — le point de dépôt aligné à droite au bord x = 57 (§ 3.6.4),
    /// dans les quatre langues et au repli anglais ; ligne de base 20,5 (5 jusqu'à la Story
    /// 44-1a, qui remonte le point de dépôt à la place des échantillons de SIX). Les abscisses
    /// attendues et les largeurs sont calculées à la main depuis `phvb8a.afm` (somme des
    /// `WX`, 6 pt), indépendamment de la table : fr 6945, de 7113, en 8335, it 10280.
    #[test]
    fn le_point_de_depot_est_aligne_a_droite() {
        let mut cas: Vec<(&str, QrBillI18n, &str, f32, f32)> = [
            ("fr", "Point de dépôt", 42.30, 14.70),
            ("de", "Annahmestelle", 41.94, 15.06),
            ("en", "Acceptance point", 39.36, 17.64),
            ("it", "Punto di accettazione", 35.24, 21.76),
        ]
        .into_iter()
        .map(|(l, libelle, x, largeur)| (l, i18n_de(l), libelle, x, largeur))
        .collect();
        cas.push((
            "repli",
            QrBillI18n::default(),
            "Acceptance point",
            39.36,
            17.64,
        ));
        // Les écarts de toutes les langues, réunis : un rouge les nomme toutes.
        let mut ecarts = Vec::new();
        for (langue, i18n, libelle, x_attendu, largeur) in cas {
            assert_eq!(
                i18n.get("invoice-pdf-qr-acceptance-point"),
                libelle,
                "montage : le libellé de {langue}"
            );
            let textes = textes_du_pdf(&pdf_ordinaire(&i18n));
            let trouves: Vec<_> = textes.iter().filter(|(_, _, t)| t == libelle).collect();
            assert_eq!(trouves.len(), 1, "{langue} : {trouves:?}");
            let (x, y, _) = trouves[0];
            if (x - x_attendu).abs() > 0.05 {
                ecarts.push(format!("{langue} : Td.x = {x}, attendu {x_attendu}"));
            }
            if (y - POINT_DE_DEPOT_BASE).abs() >= 0.01 {
                ecarts.push(format!("{langue} : ligne de base {y}"));
            }
            let marge = RECEIPT_W - (x + largeur);
            if marge < 4.5 {
                ecarts.push(format!("{langue} : marge de {marge} mm à la ligne"));
            }
        }
        assert!(ecarts.is_empty(), "{ecarts:#?}");
    }

    /// **AC6** (Story 36-12a) — la ligne verticale ne traverse aucun texte du récépissé, pour
    /// les adresses ordinaires de type S, une rue longue, sans référence, et — depuis la
    /// Story 44-1a, qui coupe à la largeur mesurée — une adresse de capitales très larges,
    /// en français et en anglais.
    #[test]
    fn la_ligne_verticale_ne_traverse_aucun_texte() {
        let rue = "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau";
        // Les textes traversés de toutes les langues et fixtures, réunis.
        let mut traverses = Vec::new();
        for langue in ["fr", "en"] {
            let i18n = i18n_de(langue);
            for (nom, creancier, debiteur, sans_reference) in [
                (
                    "ordinaire",
                    creancier_ordinaire(),
                    debiteur_ordinaire(),
                    false,
                ),
                (
                    "rue longue",
                    adresse_s("Robert Schneider SA", rue, "12", "2501", "Biel"),
                    adresse_s("Pia Rutschmann", rue, "12", "9400", "Rorschach"),
                    false,
                ),
                (
                    "sans référence",
                    creancier_ordinaire(),
                    debiteur_ordinaire(),
                    true,
                ),
                (
                    "capitales très larges",
                    adresse_s(&"W".repeat(70), &"W".repeat(70), "12", "2501", "Biel"),
                    adresse_s(&"W".repeat(27), "Marktgasse", "28", "9400", "Rorschach"),
                    false,
                ),
            ] {
                let (mut data, invoice, _) = invoice_fixture();
                data.creditor = creancier;
                data.ultimate_debtor = Some(debiteur);
                if sans_reference {
                    data.creditor_iban = "CH9300762011623852957".into();
                    data.reference = Reference::None;
                }
                let pdf =
                    generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, fixed_date()).unwrap();
                let r = relever(&pdf);
                assert!(
                    !r.verticaux.is_empty(),
                    "{langue}, {nom} : aucun trait sur x = 62"
                );
                let recu: Vec<_> = textes_mesures(&pdf)
                    .into_iter()
                    .filter(|t| t.y < SEP_Y && t.x < RECEIPT_W)
                    .collect();
                assert!(recu.len() > 10, "montage : les textes du récépissé");
                for t in &recu {
                    for &(a, b) in &r.verticaux {
                        let trait_ = Boite {
                            x0: RECEIPT_W,
                            y0: a,
                            x1: RECEIPT_W,
                            y1: b,
                        };
                        if !t.boite().disjointe(&trait_) {
                            traverses.push(format!(
                                "{langue}, {nom} : {:?} ({:?}) traversé par {a}..{b}",
                                t.texte,
                                t.boite()
                            ));
                        }
                    }
                }
            }
        }
        assert!(traverses.is_empty(), "{traverses:#?}");
    }

    /// **AC7** (Story 36-12a) — lignes et ciseaux au trait seul (`S` ou `s`), en noir (`RG`
    /// 0 0 0) : aucun remplissage, qui poserait un `rg` (cf. `chaque_texte_s_imprime_en_noir`).
    #[test]
    fn lignes_et_ciseaux_au_trait_noir() {
        for (langue, pdf) in pdfs_fr_en() {
            let r = relever(&pdf);
            assert!(
                r.ciseau_h.is_some() && r.ciseau_v.is_some(),
                "{langue} : deux ciseaux : {r:?}"
            );
            for c in chemins_de_la_section(&pdf) {
                assert!(
                    c.peinture == "S" || c.peinture == "s",
                    "{langue} : chemin peint par {} : {c:?}",
                    c.peinture
                );
                assert_eq!(c.trait_rg, [0.0, 0.0, 0.0], "{langue} : {c:?}");
            }
        }
    }

    // ----- Story 44-1a (#944) : la grille SIX, *Payable by*, la coupe à la largeur, les paliers -----
    //
    // Repère de la fiche : mm depuis le bas à gauche ; la *boîte* d'un texte va de `Td.x` à
    // `Td.x` + sa largeur (table AFM), et de sa ligne de base − 0,207 em (`Descender`) à + 0,718 em
    // (`CapHeight`) — la boîte que poppler rend pour la police standard (relevé indépendant,
    // `pire_cas_boites_independantes`).

    /// Tolérance des positions : l'arrondi des points écrits par printpdf, bien sous la plus
    /// serrée des marges de la grille (0,48 mm entre les intitulés du montant et la bordure du QR).
    const TOL: f32 = 0.01;

    /// La boîte d'un texte mesuré, au sens de la fiche 44-1a (sans le demi-millimètre de jeu de
    /// [`TexteMesure::boite`], qui servait la 36-12a).
    fn boite_44(t: &TexteMesure) -> Boite {
        Boite {
            x0: t.x,
            y0: t.y - 0.207 * t.taille * MM_PAR_PT,
            x1: t.x + crate::largeurs::largeur_mm(&t.texte, t.taille, t.gras),
            y1: t.y + 0.718 * t.taille * MM_PAR_PT,
        }
    }

    /// Les textes de la section de paiement (sous la ligne horizontale).
    fn textes_de_la_section(pdf: &[u8]) -> Vec<TexteMesure> {
        textes_mesures(pdf)
            .into_iter()
            .filter(|t| t.y < SEP_Y)
            .collect()
    }

    /// **AC3** (Story 44-1a) — ce qui sort de sa zone dans la section de paiement d'un PDF : le
    /// récépissé dans [5, 57], la colonne d'information dans [118, 205], le titre et les
    /// intitulés de la partie paiement à gauche de 118, rien dans le QR élargi de sa bordure de
    /// 5 mm (x ∈ [62, 118], y ∈ [37, 93]), rien sous y = 5, l'information du récépissé à 5 mm
    /// au moins au-dessus de la section montant (bas ≥ 41,52), rien sur le point de dépôt.
    fn zones_violees(pdf: &[u8]) -> Vec<String> {
        let textes = textes_de_la_section(pdf);
        let mut v = Vec::new();
        let qr = Boite {
            x0: 62.0,
            y0: 37.0,
            x1: 118.0,
            y1: 93.0,
        };
        let au_point_de_depot = |t: &TexteMesure| t.x < RECEIPT_W && (t.y - 20.5).abs() < TOL;
        let depot: Vec<Boite> = textes
            .iter()
            .filter(|t| au_point_de_depot(t))
            .map(boite_44)
            .collect();
        if depot.len() != 1 {
            v.push(format!("un seul point de dépôt à la base 20,5 : {depot:?}"));
        }
        for t in &textes {
            let b = boite_44(t);
            let nom = format!("{:?} à ({:.2}, {:.2})", t.texte, t.x, t.y);
            if b.y0 < 5.0 - TOL {
                v.push(format!("{nom} : bas {:.2} sous y = 5", b.y0));
            }
            if t.x < RECEIPT_W {
                if b.x0 < 5.0 - TOL || b.x1 > 57.0 + TOL {
                    v.push(format!(
                        "{nom} : hors de [5, 57] ({:.2}..{:.2})",
                        b.x0, b.x1
                    ));
                }
                // L'information : ni le titre (11 pt), ni la section montant et le point de dépôt
                // (bases 35, 31,5 et 20,5).
                if t.taille < 11.0 && t.y > 35.0 + TOL && b.y0 < 41.52 - TOL {
                    v.push(format!(
                        "{nom} : bas {:.2} à moins de 5 mm du montant",
                        b.y0
                    ));
                }
                if !au_point_de_depot(t) && depot.iter().any(|d| !b.disjointe(d)) {
                    v.push(format!("{nom} : sur le point de dépôt"));
                }
            } else if t.x >= 118.0 - TOL {
                if b.x1 > 205.0 + TOL {
                    v.push(format!("{nom} : au-delà de x = 205 ({:.2})", b.x1));
                }
            } else if b.x1 > 118.0 + TOL {
                v.push(format!(
                    "{nom} : déborde sur la colonne d'information ({:.2})",
                    b.x1
                ));
            }
            if b.x0 < qr.x1 - TOL && b.x1 > qr.x0 + TOL && b.y0 < qr.y1 - TOL && b.y1 > qr.y0 + TOL
            {
                v.push(format!("{nom} : dans le QR élargi ({b:?})"));
            }
        }
        v
    }

    /// Les rectangles remplis du contenu (`re`), en mm, avec la couleur de remplissage en vigueur.
    fn rectangles(pdf: &[u8]) -> Vec<(Boite, Vec<f32>)> {
        let mut couleur = vec![0.0, 0.0, 0.0];
        let mut rects = Vec::new();
        for op in operations_du_pdf(pdf) {
            match op.operator.as_str() {
                "rg" => couleur = op.operands.iter().map(nombre).collect(),
                "re" => {
                    let v: Vec<f32> = op.operands.iter().map(|o| nombre(o) * MM_PAR_PT).collect();
                    rects.push((
                        Boite {
                            x0: v[0],
                            y0: v[1],
                            x1: v[0] + v[2],
                            y1: v[1] + v[3],
                        },
                        couleur.clone(),
                    ));
                }
                _ => {}
            }
        }
        rects
    }

    /// Les modules noirs du QR : rectangles noirs de la section de paiement.
    fn modules_noirs(pdf: &[u8]) -> Vec<Boite> {
        rectangles(pdf)
            .into_iter()
            .filter(|(b, c)| c.as_slice() == [0.0, 0.0, 0.0] && b.y1 < SEP_Y)
            .map(|(b, _)| b)
            .collect()
    }

    /// Une QR-facture de la fixture (référence QRR) pour ce créancier, ce débiteur et ce message.
    fn donnees_44(
        creancier: Address,
        debiteur: Option<Address>,
        message: Option<&str>,
    ) -> QrBillData {
        let (mut data, _, _) = invoice_fixture();
        data.creditor = creancier;
        data.ultimate_debtor = debiteur;
        data.unstructured_message = message.map(Into::into);
        data
    }

    /// La même, sans référence (IBAN ordinaire).
    fn sans_ref(mut data: QrBillData) -> QrBillData {
        data.creditor_iban = "CH9300762011623852957".into();
        data.reference = Reference::None;
        data
    }

    /// Le PDF de ces données, avec le corps de la fixture, en anglais (libellés par défaut).
    fn pdf_44(data: &QrBillData) -> Vec<u8> {
        let (_, invoice, i18n) = invoice_fixture();
        generate_qr_bill_pdf_with_date(data, &invoice, &i18n, fixed_date()).unwrap()
    }

    /// Le pire cas de la fiche (AC5) : nom de 70 `W`, rue de 70 `W` et numéro de 16 chiffres, NPA
    /// étranger de 16 `W` et localité de 35 `W`.
    fn adresse_pire() -> Address {
        adresse_etrangere(
            &"W".repeat(70),
            &"W".repeat(70),
            "1234567890123456",
            &"W".repeat(16),
            &"W".repeat(35),
            "DE",
        )
    }

    /// L'adresse du contre-exemple de l'AC5 (P3 sans R4) : celle du pire cas, nom de 5 `A`.
    fn adresse_contre_exemple() -> Address {
        Address {
            name: "AAAAA".into(),
            ..adresse_pire()
        }
    }

    /// Un message de 139 caractères en huit lignes à 87 mm : mots de 17 et 16 `W` en
    /// alternance, dont deux ne tiennent jamais ensemble (45,29 + 0,78 + 42,63 > 87).
    fn message_huit_lignes() -> String {
        [17, 16, 17, 16, 17, 16, 17, 16]
            .map(|n| "W".repeat(n))
            .join(" ")
    }

    /// Les fixtures de l'AC3 de la 44-1a, nommées.
    fn fixtures_44() -> Vec<(&'static str, QrBillData)> {
        let rue = "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau";
        let message = Some("Facture F-2026-0042");
        let k = |name: &str, line1: &str, line2: &str| Address {
            address_type: AddressType::Combined,
            name: name.into(),
            line1: line1.into(),
            line2: line2.into(),
            postal_code: String::new(),
            town: String::new(),
            country: "CH".into(),
        };
        let huit = message_huit_lignes();
        vec![
            (
                "ordinaire",
                donnees_44(creancier_ordinaire(), Some(debiteur_ordinaire()), message),
            ),
            (
                "rue longue",
                donnees_44(
                    adresse_s("Robert Schneider SA", rue, "12", "2501", "Biel"),
                    Some(adresse_s("Pia Rutschmann", rue, "12", "9400", "Rorschach")),
                    message,
                ),
            ),
            (
                "étrangère",
                donnees_44(
                    adresse_etrangere("Vaduz AG", "Städtle", "5", "9490", "Vaduz", "LI"),
                    Some(adresse_etrangere(
                        "Jan",
                        "ulica Długa",
                        "5",
                        "90-001",
                        "Łódź",
                        "PL",
                    )),
                    message,
                ),
            ),
            (
                "sans référence",
                sans_ref(donnees_44(
                    creancier_ordinaire(),
                    Some(debiteur_ordinaire()),
                    message,
                )),
            ),
            (
                "sans message",
                donnees_44(creancier_ordinaire(), Some(debiteur_ordinaire()), None),
            ),
            (
                "débiteur absent",
                donnees_44(creancier_ordinaire(), None, message),
            ),
            ("type K", invoice_fixture().0),
            ("réaliste R1", donnees_realiste_r1()),
            (
                "R1 sur le seul débiteur",
                donnees_44(creancier_ordinaire(), Some(debiteur_r1_seul()), message),
            ),
            (
                "R4 sans P3",
                donnees_44(
                    adresse_s(&"W".repeat(70), "Rue du Lac", "1268", "2501", "Biel"),
                    Some(adresse_s(
                        &"W".repeat(70),
                        "Marktgasse",
                        "28",
                        "9400",
                        "Rorschach",
                    )),
                    message,
                ),
            ),
            (
                "P3 et R4",
                donnees_44(adresse_pire(), Some(adresse_pire()), Some(&huit)),
            ),
            (
                "P3 et R1 sans R4",
                donnees_44(
                    adresse_contre_exemple(),
                    Some(adresse_contre_exemple()),
                    Some(&huit),
                ),
            ),
            (
                "type K, neuf lignes",
                donnees_44(
                    k(&"W".repeat(70), "Rue du Lac 1268", "2501 Biel"),
                    Some(k("Pia Rutschmann", "Marktgasse 28", "9400 Rorschach")),
                    message,
                ),
            ),
        ]
    }

    /// La fixture réaliste de neuf lignes au récépissé (AC5) : noms de deux lignes des deux
    /// côtés, rue de deux lignes du côté du créancier, QR-IBAN et référence QRR.
    fn donnees_realiste_r1() -> QrBillData {
        donnees_44(
            adresse_s(
                "Fondation suisse pour la protection du paysage",
                "Chemin des Trois-Fontaines de la Vieille Tour",
                "12",
                "2501",
                "Biel",
            ),
            Some(adresse_s(
                "Coopérative d'habitation des Jardins du Lac",
                "Marktgasse",
                "28",
                "9400",
                "Rorschach",
            )),
            Some("Facture F-2026-0042"),
        )
    }

    /// Un débiteur de six lignes au récépissé (nom, rue et localité de deux lignes chacun) :
    /// avec un créancier ordinaire, neuf lignes, et seul le débiteur déborde.
    fn debiteur_r1_seul() -> Address {
        adresse_s(
            "Association des amis de la Fondation suisse pour le paysage alpin",
            "Chemin des Trois-Fontaines de la Vieille Tour du Haut-Plateau",
            "12",
            "9400",
            "Rorschacherberg-Wartensee-Buechen",
        )
    }

    /// Les valeurs (8 pt) de l'information du récépissé, de haut en bas.
    fn valeurs_du_recu(pdf: &[u8]) -> Vec<String> {
        let mut v: Vec<_> = textes_de_la_section(pdf)
            .into_iter()
            .filter(|t| t.x < RECEIPT_W && t.taille == 8.0 && t.y > 35.0 + TOL)
            .collect();
        v.sort_by(|a, b| b.y.total_cmp(&a.y));
        v.into_iter().map(|t| t.texte).collect()
    }

    /// Les textes de la colonne d'information (x ≥ 118), de haut en bas, avec leur base.
    fn colonne(pdf: &[u8]) -> Vec<(f32, String)> {
        let mut v: Vec<_> = textes_de_la_section(pdf)
            .into_iter()
            .filter(|t| t.x >= 118.0 - TOL)
            .collect();
        v.sort_by(|a, b| b.y.total_cmp(&a.y));
        v.into_iter().map(|t| (t.y, t.texte)).collect()
    }

    /// La référence QRR groupée de la fixture.
    const QRR_GROUPEE: &str = "00 00000 00004 20000 00000 01008";
    /// L'IBAN groupé de la fixture.
    const IBAN_GROUPE: &str = "CH44 3199 9123 0008 8901 2";

    /// **AC1** (Story 44-1a) — la grille de D2, mesurée dans le PDF, dans les quatre langues :
    /// titres à la base 96,5 (x = 5 et 67), QR de 46 mm de x = 67 à 113 et de y = 42 à 88,
    /// colonne d'information à x = 118, premiers intitulés aux bases 90,5 (récépissé) et 98,0
    /// (colonne), section montant à la base 35,0 des deux côtés (≥ 5 mm sous le QR), point de
    /// dépôt à la base 20,5 et au bord droit 57 ; le titre et les intitulés de la partie
    /// paiement à gauche de x = 118.
    #[test]
    fn la_grille_six_est_tenue() {
        let mut ecarts = Vec::new();
        for (langue, _) in LANGUES {
            let i18n = i18n_de(langue);
            let pdf = pdf_ordinaire(&i18n);
            let textes = textes_de_la_section(&pdf);
            let mut attendre = |cle: &'static str, x: f32, y: f32| {
                let libelle = i18n.get(cle);
                if !textes
                    .iter()
                    .any(|t| t.texte == libelle && (t.x - x).abs() < TOL && (t.y - y).abs() < TOL)
                {
                    ecarts.push(format!("{langue} : {libelle:?} attendu à ({x}, {y})"));
                }
            };
            attendre("invoice-pdf-qr-section-receipt", 5.0, 96.5);
            attendre("invoice-pdf-qr-section-payment", 67.0, 96.5);
            attendre("invoice-pdf-qr-account", 5.0, 90.5);
            attendre("invoice-pdf-qr-account", 118.0, 98.0);
            attendre("invoice-pdf-qr-currency", 5.0, 35.0);
            attendre("invoice-pdf-qr-amount", 17.0, 35.0);
            attendre("invoice-pdf-qr-currency", 67.0, 35.0);
            attendre("invoice-pdf-qr-amount", 82.0, 35.0);
            // Le point de dépôt : base 20,5, bord droit 57.
            let depot = i18n.get("invoice-pdf-qr-acceptance-point");
            match textes.iter().find(|t| t.texte == depot) {
                Some(t) => {
                    let bord = boite_44(t).x1;
                    if (t.y - 20.5).abs() >= TOL || (bord - 57.0).abs() > 0.05 {
                        ecarts.push(format!(
                            "{langue} : point de dépôt à ({}, {}), bord {bord}",
                            t.x, t.y
                        ));
                    }
                }
                None => ecarts.push(format!("{langue} : pas de point de dépôt")),
            }
            // Le titre et les intitulés de la partie paiement, à gauche de la colonne.
            for t in textes
                .iter()
                .filter(|t| t.x > RECEIPT_W && t.x < 118.0 - TOL)
            {
                let bord = boite_44(t).x1;
                if bord > 118.0 {
                    ecarts.push(format!("{langue} : {:?} finit à {bord}", t.texte));
                }
            }
            // Le QR : 46 × 46 mm, x de 67 à 113, y de 42 à 88.
            let modules = modules_noirs(&pdf);
            let qr = modules.iter().skip(1).fold(modules[0], |b, m| b.union(m));
            for (nom, mesure, attendu) in [
                ("x0", qr.x0, 67.0),
                ("x1", qr.x1, 113.0),
                ("y0", qr.y0, 42.0),
                ("y1", qr.y1, 88.0),
            ] {
                if (mesure - attendu).abs() > TOL {
                    ecarts.push(format!("{langue} : QR {nom} = {mesure}, attendu {attendu}"));
                }
            }
            // La section montant à 5 mm au moins sous le QR : haut des intitulés de 6 pt
            // (35 + 0,718 em) sous la bordure, à qr.y0 − 5.
            let haut_montant = 35.0 + 0.718 * 6.0 * MM_PAR_PT;
            if haut_montant > qr.y0 - 5.0 + TOL {
                ecarts.push(format!(
                    "{langue} : montant (haut {haut_montant}) à moins de 5 mm du QR ({})",
                    qr.y0
                ));
            }
        }
        assert!(ecarts.is_empty(), "{ecarts:#?}");
    }

    /// **AC2** (Story 44-1a) — *Payable by* dans la section d'information : au récépissé, à
    /// x = 5, après la référence et avant la section montant ; en partie paiement, à x = 118,
    /// sous *Additional information* (ou sous la référence, sans message) ; plus rien à x = 67
    /// sous la section montant.
    #[test]
    fn payable_by_dans_la_section_d_information() {
        let pdf = pdf_44(&donnees_44(
            creancier_ordinaire(),
            Some(debiteur_ordinaire()),
            Some("Facture F-2026-0042"),
        ));
        let textes = textes_du_pdf(&pdf);
        for (x, y, attendu) in [
            (5.0, 71.0, "Reference"),
            (5.0, 68.0, QRR_GROUPEE),
            (5.0, 62.0, "Payable by"),
            (5.0, 58.5, "Pia Rutschmann"),
            (5.0, 55.0, "Marktgasse 28"),
            (5.0, 51.5, "9400 Rorschach"),
            (118.0, 69.5, "Additional information"),
            (118.0, 66.5, "Facture F-2026-0042"),
            (118.0, 60.5, "Payable by"),
            (118.0, 57.0, "Pia Rutschmann"),
            (118.0, 53.5, "Marktgasse 28"),
            (118.0, 50.0, "9400 Rorschach"),
        ] {
            assert_eq!(texte_a(&textes, x, y), attendu, "à ({x}, {y})");
        }
        // Sous la section montant de la partie paiement (base 31,5), à x = 67 : rien.
        let sous: Vec<_> = textes
            .iter()
            .filter(|(x, y, _)| (*x - 67.0).abs() < TOL && *y < 31.5 - TOL)
            .collect();
        assert!(sous.is_empty(), "{sous:?}");

        // Sans message : *Payable by* sous la référence.
        let pdf = pdf_44(&donnees_44(
            creancier_ordinaire(),
            Some(debiteur_ordinaire()),
            None,
        ));
        let textes = textes_du_pdf(&pdf);
        assert_eq!(texte_a(&textes, 118.0, 69.5), "Payable by");
    }

    /// **AC3** (Story 44-1a) — aucune ligne hors de sa zone, pour chaque fixture.
    #[test]
    fn chaque_ligne_dans_sa_zone() {
        let mut v = Vec::new();
        for (nom, data) in fixtures_44() {
            for e in zones_violees(&pdf_44(&data)) {
                v.push(format!("{nom} : {e}"));
            }
        }
        assert!(v.is_empty(), "{v:#?}");
    }

    /// **AC4** (Story 44-1a) — la coupe mesure la largeur, avec des valeurs calculées à la main
    /// depuis `phvr8a.afm` (`W` = 944), hors de la table : 27 `W` à 8 pt = 71,93 mm, 19 `W` =
    /// 50,62 mm tiennent à 52, 20 = 53,28 non ; 32 `W` = 85,25 mm tiennent à 87, 33 = 87,91 non ;
    /// « Robert Schneider SA » = 26,51 mm et « Rue du Lac 1268 » = 21,49 mm.
    #[test]
    fn la_coupe_mesure_les_largeurs() {
        // Les valeurs écrites à la main, confrontées à la table (sa justesse est l'AC8 de la
        // 36-12a ; ici, elles fixent les attentes de la coupe).
        for (texte, mm) in [
            ("W".repeat(27), 71.93),
            ("W".repeat(19), 50.62),
            ("W".repeat(20), 53.28),
            ("W".repeat(32), 85.25),
            ("W".repeat(33), 87.91),
            ("Robert Schneider SA".to_string(), 26.51),
            ("Rue du Lac 1268".to_string(), 21.49),
        ] {
            let table = crate::largeurs::largeur_mm(&texte, 8.0, false);
            assert!((table - mm).abs() < 0.01, "{texte} : {table} pour {mm}");
        }
        // 27 `W` : 19 + 8 au récépissé (52 mm), une ligne dans la colonne (87 mm).
        let nom = |n: usize| adresse_s(&"W".repeat(n), "Rue du Lac", "1268", "2501", "Biel");
        let textes = textes_du_pdf(&pdf_44(&donnees_44(
            nom(27),
            Some(debiteur_ordinaire()),
            None,
        )));
        assert_eq!(texte_a(&textes, 5.0, 84.0), "W".repeat(19));
        assert_eq!(texte_a(&textes, 5.0, 80.5), "W".repeat(8));
        assert_eq!(texte_a(&textes, 5.0, 77.0), "Rue du Lac 1268");
        assert_eq!(texte_a(&textes, 118.0, 91.5), "W".repeat(27));
        assert_eq!(texte_a(&textes, 118.0, 88.0), "Rue du Lac 1268");
        // 33 `W` : 32 + 1 dans la colonne.
        let textes = textes_du_pdf(&pdf_44(&donnees_44(
            nom(33),
            Some(debiteur_ordinaire()),
            None,
        )));
        assert_eq!(texte_a(&textes, 118.0, 91.5), "W".repeat(32));
        assert_eq!(texte_a(&textes, 118.0, 88.0), "W");
        // Les deux lignes ordinaires, entières des deux côtés.
        let textes = textes_du_pdf(&pdf_ordinaire(&QrBillI18n::default()));
        for (x, y) in [(5.0, 84.0), (118.0, 91.5)] {
            assert_eq!(texte_a(&textes, x, y), "Robert Schneider SA");
        }
        for (x, y) in [(5.0, 80.5), (118.0, 88.0)] {
            assert_eq!(texte_a(&textes, x, y), "Rue du Lac 1268");
        }
    }

    /// **AC5** (Story 44-1a) — la cascade, palier par palier, lue dans le PDF.
    #[test]
    fn la_cascade_par_palier() {
        let par_nom: std::collections::HashMap<_, _> = fixtures_44().into_iter().collect();
        // Les paliers que la mise en page pure retient, fixture par fixture : « et seulement
        // lui » pour R1 et pour R4 sans P3 ; R1 et P3 décidés chacun de son côté (M14).
        use PalierPaiement::*;
        use PalierRecu::*;
        let mut paliers = Vec::new();
        for (nom, recu, paiement) in [
            ("ordinaire", R0, P0),
            ("rue longue", R0, P0),
            ("étrangère", R0, P0),
            ("sans référence", R0, P0),
            ("sans message", R0, P0),
            ("débiteur absent", R0, P0),
            ("type K", R0, P0),
            ("réaliste R1", R1, P0),
            ("R1 sur le seul débiteur", R1, P0),
            ("R4 sans P3", R4, P0),
            ("P3 et R4", R4, P3),
            ("P3 et R1 sans R4", R1, P3),
            ("type K, neuf lignes", R4, P0),
        ] {
            let page = mise_en_page(&par_nom[nom], &QrBillI18n::default(), "CHF", "1'234.50");
            if (page.palier_recu, page.palier_paiement) != (recu, paiement) {
                paliers.push(format!(
                    "{nom} : {:?}, {:?} au lieu de {recu:?}, {paiement:?}",
                    page.palier_recu, page.palier_paiement
                ));
            }
        }
        assert_eq!(par_nom.len(), 13, "montage : chaque fixture a son palier");
        assert!(paliers.is_empty(), "{paliers:#?}");

        let pdf = |nom: &str| pdf_44(&par_nom[nom]);
        let w = |n: usize| "W".repeat(n);
        // Les écarts de toutes les fixtures, réunis : un rouge les nomme toutes.
        let ecarts = std::cell::RefCell::new(Vec::new());
        let attendre = |nom: &str, obtenu: Vec<String>, attendu: Vec<String>| {
            if obtenu != attendu {
                ecarts.borrow_mut().push(format!(
                    "{nom} :\n  obtenu  {obtenu:?}\n  attendu {attendu:?}"
                ));
            }
        };
        let s = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        // R0 : l'ordinaire imprime sa rue au récépissé ; la rue longue (8 lignes) aussi.
        attendre(
            "ordinaire",
            valeurs_du_recu(&pdf("ordinaire")),
            s(&[
                IBAN_GROUPE,
                "Robert Schneider SA",
                "Rue du Lac 1268",
                "2501 Biel",
                QRR_GROUPEE,
                "Pia Rutschmann",
                "Marktgasse 28",
                "9400 Rorschach",
            ]),
        );
        attendre(
            "rue longue",
            valeurs_du_recu(&pdf("rue longue")),
            s(&[
                IBAN_GROUPE,
                "Robert Schneider SA",
                "Chemin des Trois-Fontaines de la Vieille",
                "Tour du Haut-Plateau 12",
                "2501 Biel",
                QRR_GROUPEE,
                "Pia Rutschmann",
                "Chemin des Trois-Fontaines de la Vieille",
                "Tour du Haut-Plateau 12",
                "9400 Rorschach",
            ]),
        );
        // R1 : neuf lignes ; la rue manque des deux adresses, les noms restent entiers.
        attendre(
            "réaliste R1",
            valeurs_du_recu(&pdf("réaliste R1")),
            s(&[
                IBAN_GROUPE,
                "Fondation suisse pour la protection du",
                "paysage",
                "2501 Biel",
                QRR_GROUPEE,
                "Coopérative d'habitation des Jardins du",
                "Lac",
                "9400 Rorschach",
            ]),
        );
        let col: Vec<String> = colonne(&pdf("réaliste R1"))
            .into_iter()
            .map(|(_, t)| t)
            .collect();
        for rue in [
            "Chemin des Trois-Fontaines de la Vieille Tour 12",
            "Marktgasse 28",
        ] {
            if !col.iter().any(|t| t == rue) {
                ecarts
                    .borrow_mut()
                    .push(format!("réaliste R1 : colonne sans {rue:?} (P0) : {col:?}"));
            }
        }
        // R1 sur le seul débiteur : le créancier perd aussi sa rue.
        attendre(
            "R1 sur le seul débiteur",
            valeurs_du_recu(&pdf("R1 sur le seul débiteur")),
            s(&[
                IBAN_GROUPE,
                "Robert Schneider SA",
                "2501 Biel",
                QRR_GROUPEE,
                "Association des amis de la Fondation",
                "suisse pour le paysage alpin",
                "9400",
                "Rorschacherberg-Wartensee-Buechen",
            ]),
        );
        // R4 sans P3 : noms de quatre lignes ; nom et localité, ramenés à 52 mm.
        let w18 = format!("{}...", w(18));
        attendre(
            "R4 sans P3",
            valeurs_du_recu(&pdf("R4 sans P3")),
            vec![
                IBAN_GROUPE.into(),
                w18.clone(),
                "2501 Biel".into(),
                QRR_GROUPEE.into(),
                w18.clone(),
                "9400 Rorschach".into(),
            ],
        );
        attendre(
            "R4 sans P3, colonne en P0",
            colonne(&pdf("R4 sans P3"))
                .into_iter()
                .map(|(_, t)| t)
                .filter(|t| t.starts_with('W'))
                .collect(),
            vec![w(32), w(32), w(6), w(32), w(32), w(6)],
        );
        // P3 et R4 : le pire cas.
        // « DE- » et 16 `W` : 47,49 mm ; l'espace et un `W` de plus ne laissent plus la place
        // des « ... » (2,35 mm) : l'espace de fin tombe, les points suivent le NPA.
        let localite_52 = format!("DE-{}...", w(16));
        attendre(
            "P3 et R4",
            valeurs_du_recu(&pdf("P3 et R4")),
            vec![
                IBAN_GROUPE.into(),
                w18.clone(),
                localite_52.clone(),
                QRR_GROUPEE.into(),
                w18.clone(),
                localite_52,
            ],
        );
        let ligne_87 = format!("{}...", w(31));
        // 47,49 + 0,78 + 13 × 2,664 + 2,35 = 85,26 mm ; un 14e `W` dépasse 87.
        let localite_87 = format!("DE-{} {}...", w(16), w(13));
        attendre(
            "P3 et R4, colonne",
            colonne(&pdf("P3 et R4"))
                .into_iter()
                .map(|(_, t)| t)
                .collect(),
            vec![
                "Account / Payable to".into(),
                IBAN_GROUPE.into(),
                ligne_87.clone(),
                ligne_87.clone(),
                localite_87.clone(),
                "Reference".into(),
                QRR_GROUPEE.into(),
                "Additional information".into(),
                format!("{} {}...", w(17), w(14)),
                "Payable by".into(),
                ligne_87.clone(),
                ligne_87,
                localite_87,
            ],
        );
        // P3 et R1 sans R4 : la colonne tronque, le récépissé omet seulement la rue.
        attendre(
            "P3 et R1 sans R4",
            valeurs_du_recu(&pdf("P3 et R1 sans R4")),
            vec![
                IBAN_GROUPE.into(),
                "AAAAA".into(),
                format!("DE-{}", w(16)),
                w(19),
                w(16),
                QRR_GROUPEE.into(),
                "AAAAA".into(),
                format!("DE-{}", w(16)),
                w(19),
                w(16),
            ],
        );
        let col = colonne(&pdf("P3 et R1 sans R4"));
        let messages: Vec<_> = col
            .iter()
            .skip_while(|(_, t)| t != "Additional information")
            .skip(1)
            .take_while(|(_, t)| t != "Payable by")
            .map(|(_, t)| t.clone())
            .collect();
        attendre(
            "P3 et R1 sans R4, message",
            messages,
            vec![format!("{} {}...", w(17), w(14))],
        );
        // Débiteur absent : l'intitulé seul.
        attendre(
            "débiteur absent",
            valeurs_du_recu(&pdf("débiteur absent")),
            s(&[
                IBAN_GROUPE,
                "Robert Schneider SA",
                "Rue du Lac 1268",
                "2501 Biel",
                QRR_GROUPEE,
            ]),
        );
        let col = colonne(&pdf("débiteur absent"));
        if col.last().map(|(_, t)| t.as_str()) != Some("Payable by") {
            ecarts
                .borrow_mut()
                .push(format!("débiteur absent : colonne {col:?}"));
        }
        // Type K de neuf lignes : R1 sans objet, R4.
        attendre(
            "type K, neuf lignes",
            valeurs_du_recu(&pdf("type K, neuf lignes")),
            vec![
                IBAN_GROUPE.into(),
                w18,
                "2501 Biel".into(),
                QRR_GROUPEE.into(),
                "Pia Rutschmann".into(),
                "9400 Rorschach".into(),
            ],
        );
        // P0 : un message long s'imprime entier, une ligne par mot, au pas de `MESSAGE_PAS`.
        let pdf_message = pdf_44(&donnees_44(
            creancier_ordinaire(),
            Some(debiteur_ordinaire()),
            Some(&[17, 16, 17, 16].map(w).join(" ")),
        ));
        let col = colonne(&pdf_message);
        for (y, attendu) in [
            (66.5, w(17)),
            (63.0, w(16)),
            (59.5, w(17)),
            (56.0, w(16)),
            (50.0, "Payable by".to_string()),
        ] {
            if !col
                .iter()
                .any(|(cy, t)| (cy - y).abs() < TOL && *t == attendu)
            {
                ecarts.borrow_mut().push(format!(
                    "message en P0 : {attendu:?} attendu à {y} : {col:?}"
                ));
            }
        }
        let ecarts = ecarts.into_inner();
        assert!(ecarts.is_empty(), "{}", ecarts.join("\n"));
    }

    /// **AC6** (Story 44-1a) — R4 et P3 tiennent par construction : leur bas, calculé depuis
    /// les constantes de la grille (deux lignes par adresse au récépissé, trois dans la
    /// colonne, une ligne de message), vaut 57,9 et 49,4 mm, au-dessus de 41,52 et de 5 ; et
    /// la mise en page pure rend ce même bas pour une entrée qui déclenche R4, puis P3 — elle
    /// emploie donc ces constantes (mutation M10).
    #[test]
    fn les_derniers_paliers_tiennent_par_construction() {
        assert!(
            (RECU_INFO_BAS_MIN - 41.52).abs() < 0.01,
            "{RECU_INFO_BAS_MIN}"
        );
        // R4 : compte, IBAN, nom et localité du créancier, référence, Payable by, nom et
        // localité du débiteur.
        let base_r4 = RECU_INFO_BASE
            - INTITULE_PAS
            - 2.0 * ADRESSE_PAS
            - RUBRIQUE_PAS
            - INTITULE_PAS
            - RUBRIQUE_PAS
            - 2.0 * ADRESSE_PAS;
        let bas_r4 = bas_de_ligne(base_r4, VALEUR_TAILLE);
        assert!((bas_r4 - 57.9).abs() < 0.05, "R4 : {bas_r4}");
        assert!(bas_r4 >= RECU_INFO_BAS_MIN);
        // P3 : compte, IBAN, trois lignes de créancier, référence, message d'une ligne,
        // Payable by, trois lignes de débiteur.
        let base_p3 = INFO_BASE
            - INTITULE_PAS
            - 3.0 * ADRESSE_PAS
            - RUBRIQUE_PAS
            - INTITULE_PAS
            - RUBRIQUE_PAS
            - INTITULE_PAS
            - RUBRIQUE_PAS
            - 3.0 * ADRESSE_PAS;
        let bas_p3 = bas_de_ligne(base_p3, VALEUR_TAILLE);
        assert!((bas_p3 - 49.4).abs() < 0.05, "P3 : {bas_p3}");
        assert!(bas_p3 >= INFO_BAS_MIN);

        // Ce que rend la mise en page : le pire cas déclenche R4 et P3.
        let page = mise_en_page(
            &donnees_44(
                adresse_pire(),
                Some(adresse_pire()),
                Some(&message_huit_lignes()),
            ),
            &QrBillI18n::default(),
            "CHF",
            "1'234.50",
        );
        assert_eq!(page.palier_recu, PalierRecu::R4);
        assert_eq!(page.palier_paiement, PalierPaiement::P3);
        assert!(
            (page.bas_recu - 57.9).abs() < 0.05,
            "R4 rendu : {}",
            page.bas_recu
        );
        assert!(
            (page.bas_paiement - 49.4).abs() < 0.05,
            "P3 rendu : {}",
            page.bas_paiement
        );
    }

    /// **AC8** (Story 44-1a) — aucune entrée admise par `validation::validate` ne fait échouer
    /// le PDF ni ne sort de sa zone : mots de ⌊L/2⌋+1 et ⌊L/3⌋+1 caractères (L = 19 au
    /// récépissé, 32 dans la colonne), chaînes uniformes, du caractère le plus large du jeu
    /// admis une fois imprimé ; nom 70, rue 70 + numéro 16, NPA 16, localité 35, en Suisse et à
    /// l'étranger ; avec et sans référence ; message de 0 et de 140 caractères ; type K aussi.
    #[test]
    fn aucune_entree_admise_n_echoue() {
        // Le caractère le plus large, par sa sortie de `texte_imprime` (un caractère peut s'y
        // replier en deux, `Ĳ` en `IJ`).
        let largeur =
            |c: &char| crate::largeurs::largeur_mm(&texte_imprime(&c.to_string()), 8.0, false);
        let plus_large = (0x20u32..=0x2100)
            .filter_map(char::from_u32)
            .filter(|c| crate::validation::est_caractere_qr(*c))
            .max_by(|a, b| largeur(a).total_cmp(&largeur(b)))
            .expect("un caractère");
        // Un champ de `n` caractères fait de mots de `mot` caractères (0 : un seul mot).
        let champ = |n: usize, mot: usize| -> String {
            let c = plus_large.to_string();
            if mot == 0 {
                return c.repeat(n);
            }
            let mut s = String::new();
            while s.chars().count() < n {
                if !s.is_empty() {
                    s.push(' ');
                }
                let reste = n - s.chars().count();
                s.push_str(&c.repeat(mot.min(reste)));
            }
            s.trim_end().to_string()
        };
        let mut cas = 0;
        let mut ecarts = Vec::new();
        for mot in [0, 19 / 2 + 1, 19 / 3 + 1, 32 / 2 + 1, 32 / 3 + 1] {
            for pays in ["CH", "DE"] {
                for reference in [true, false] {
                    for message in [0, 140] {
                        for type_k in [false, true] {
                            let adresse = if type_k {
                                Address {
                                    address_type: AddressType::Combined,
                                    name: champ(70, mot),
                                    line1: champ(70, mot),
                                    line2: champ(70, mot),
                                    postal_code: String::new(),
                                    town: String::new(),
                                    country: pays.into(),
                                }
                            } else {
                                adresse_etrangere(
                                    &champ(70, mot),
                                    &champ(70, mot),
                                    &champ(16, mot),
                                    &champ(16, mot),
                                    &champ(35, mot),
                                    pays,
                                )
                            };
                            let msg = (message > 0).then(|| champ(message, mot));
                            let mut data =
                                donnees_44(adresse.clone(), Some(adresse), msg.as_deref());
                            if !reference {
                                data = sans_ref(data);
                            }
                            crate::validation::validate(&data).unwrap_or_else(|e| {
                                panic!("montage : entrée refusée ({mot}, {pays}) : {e:?}")
                            });
                            let (_, invoice, i18n) = invoice_fixture();
                            let nom = format!(
                                "mot {mot}, {pays}, réf. {reference}, message {message}, K {type_k}"
                            );
                            match generate_qr_bill_pdf_with_date(
                                &data,
                                &invoice,
                                &i18n,
                                fixed_date(),
                            ) {
                                Ok(pdf) => {
                                    for e in zones_violees(&pdf) {
                                        ecarts.push(format!("{nom} : {e}"));
                                    }
                                }
                                Err(e) => ecarts.push(format!("{nom} : erreur {e:?}")),
                            }
                            cas += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(cas, 80, "montage : cas parcourus");
        let total = ecarts.len();
        ecarts.truncate(40);
        assert!(
            ecarts.is_empty(),
            "{plus_large:?}, {total} écarts, les 40 premiers : {ecarts:#?}"
        );
    }

    /// La charge utile de la fixture de type S (`pdf_ordinaire`), relevée à la base de la
    /// Story 44-1a (`4b3a20b0b`) : la mise en page n'y touche pas.
    const CHARGE_UTILE_S: &str = "SPC\n0200\n1\nCH4431999123000889012\nS\nRobert Schneider SA\nRue du Lac\n1268\n2501\nBiel\nCH\n\n\n\n\n\n\n\n1234.50\nCHF\nS\nPia Rutschmann\nMarktgasse\n28\n9400\nRorschach\nCH\nQRR\n000000000004200000000001008\nFacture F-2026-0042\nEPD\n";

    /// **AC9** (Story 44-1a) — le code QR inchangé : charge utile, matrice reconstruite des
    /// modules noirs du PDF, relecture par `rxing` ; la croix centrée en (90, 65).
    #[test]
    fn le_code_qr_est_inchange() {
        let (mut data, _, _) = invoice_fixture();
        data.creditor = creancier_ordinaire();
        data.ultimate_debtor = Some(debiteur_ordinaire());
        let charge = build_payload(&data).unwrap();
        assert_eq!(charge, CHARGE_UTILE_S);
        let qr = render_qr_image(&charge).unwrap();
        let n = qr.size() as usize;
        let pdf = pdf_ordinaire(&QrBillI18n::default());
        let modules = modules_noirs(&pdf);
        let cote = 46.0 / n as f32;
        let (x0, haut) = (
            modules.iter().map(|b| b.x0).fold(f32::MAX, f32::min),
            modules.iter().map(|b| b.y1).fold(f32::MIN, f32::max),
        );
        let mut matrice = vec![vec![false; n]; n];
        for b in &modules {
            let col = ((b.x0 - x0) / cote).round() as usize;
            let lig = ((haut - b.y1) / cote).round() as usize;
            matrice[lig][col] = true;
        }
        for (y, ligne) in matrice.iter().enumerate() {
            for (x, noir) in ligne.iter().enumerate() {
                assert_eq!(
                    *noir,
                    qr.get_module(x as i32, y as i32),
                    "module ({x}, {y})"
                );
            }
        }
        // Relecture par rxing de la matrice reconstruite.
        let (echelle, marge) = (10usize, 4usize);
        let dim = (n + 2 * marge) * echelle;
        let mut image = vec![255u8; dim * dim];
        for (y, ligne) in matrice.iter().enumerate() {
            for (x, noir) in ligne.iter().enumerate() {
                if *noir {
                    for dy in 0..echelle {
                        for dx in 0..echelle {
                            image
                                [((marge + y) * echelle + dy) * dim + (marge + x) * echelle + dx] =
                                0;
                        }
                    }
                }
            }
        }
        let lu = rxing::helpers::detect_in_luma(
            image,
            dim as u32,
            dim as u32,
            Some(rxing::BarcodeFormat::QR_CODE),
        )
        .expect("rxing relit le QR");
        assert_eq!(lu.getText(), charge);
        // La croix : carré blanc de 8 mm et carré rouge de 7 mm centrés en (90, 65).
        let rects = rectangles(&pdf);
        for (couleur, cote) in [(vec![1.0, 1.0, 1.0], 8.0), (vec![0.85, 0.0, 0.0], 7.0)] {
            let carre = rects
                .iter()
                .find(|(b, c)| *c == couleur && (b.largeur() - cote).abs() < TOL)
                .unwrap_or_else(|| panic!("carré de {cote} mm : {rects:?}"))
                .0;
            let (cx, cy) = carre.centre();
            assert!(
                (cx - 90.0).abs() < TOL
                    && (cy - 65.0).abs() < TOL
                    && (carre.hauteur() - cote).abs() < TOL,
                "carré de {cote} mm : {carre:?}"
            );
        }
    }

    /// **AC8** (Story 44-1a), relevé indépendant — le pire cas rendu en PDF, ses boîtes de mots
    /// relevées par `pdftotext -bbox` (poppler : boîtes de 718 / −207 millièmes d'em pour la
    /// police standard, en points depuis le haut de la page), sans passer par la table des
    /// largeurs ; l'AC3 vérifiée sur elles à 0,05 mm. `#[ignore]` : il exige poppler, rejoué à
    /// la clôture (`cargo test -p kesh-qrbill -- --ignored pire_cas_boites_independantes`).
    #[test]
    #[ignore]
    fn pire_cas_boites_independantes() {
        use std::io::Write;
        use std::process::{Command, Stdio};
        const T: f32 = 0.05;
        let pdf = pdf_44(&donnees_44(
            adresse_pire(),
            Some(adresse_pire()),
            Some(&message_huit_lignes()),
        ));
        // La hauteur de la page, lue à la `MediaBox`.
        let doc = printpdf::lopdf::Document::load_mem(&pdf).expect("PDF relisible");
        let page = doc.get_pages().values().next().copied().expect("une page");
        let media = doc
            .get_dictionary(page)
            .and_then(|p| p.get(b"MediaBox"))
            .and_then(|o| o.as_array())
            .expect("MediaBox")
            .clone();
        let hauteur_pt = nombre(&media[3]);
        let mut enfant = Command::new("pdftotext")
            .args(["-bbox", "-", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("pdftotext (poppler) présent");
        enfant
            .stdin
            .take()
            .expect("entrée")
            .write_all(&pdf)
            .expect("PDF écrit à pdftotext");
        let sortie = enfant.wait_with_output().expect("pdftotext terminé");
        let html = String::from_utf8(sortie.stdout).expect("UTF-8");
        let attribut = |ligne: &str, nom: &str| -> f32 {
            let debut = ligne.find(&format!("{nom}=\"")).expect(nom) + nom.len() + 2;
            ligne[debut..debut + ligne[debut..].find('"').expect("guillemet")]
                .parse()
                .expect("nombre")
        };
        let mm = |pt: f32| pt * MM_PAR_PT;
        let mut mots = 0;
        let mut ecarts = Vec::new();
        for ligne in html.lines().filter(|l| l.trim_start().starts_with("<word")) {
            let (x0, x1) = (mm(attribut(ligne, "xMin")), mm(attribut(ligne, "xMax")));
            let (haut, bas) = (
                mm(hauteur_pt - attribut(ligne, "yMin")),
                mm(hauteur_pt - attribut(ligne, "yMax")),
            );
            if haut >= SEP_Y {
                continue;
            }
            mots += 1;
            let mot = &ligne[ligne.find('>').expect(">") + 1..ligne.rfind('<').expect("<")];
            let nom = format!("{mot:?} ({x0:.2}..{x1:.2}, {bas:.2}..{haut:.2})");
            if bas < 5.0 - T {
                ecarts.push(format!("{nom} sous y = 5"));
            }
            if x0 < RECEIPT_W {
                if x0 < 5.0 - T || x1 > 57.0 + T {
                    ecarts.push(format!("{nom} hors de [5, 57]"));
                }
                // L'information du récépissé, au-dessus de 41,52 : ni le titre (haut ≥ 98), ni
                // la section montant (haut ≤ 36,52), ni le point de dépôt (haut ≤ 22,02).
                if haut > 36.52 + T && haut < 98.0 && bas < 41.52 - T {
                    ecarts.push(format!("{nom} sur la section montant"));
                }
            } else if x0 >= 118.0 - T {
                if x1 > 205.0 + T {
                    ecarts.push(format!("{nom} au-delà de 205"));
                }
            } else if x1 > 118.0 + T {
                ecarts.push(format!("{nom} sur la colonne"));
            }
            if x0 < 118.0 - T && x1 > 62.0 + T && bas < 93.0 - T && haut > 37.0 + T {
                ecarts.push(format!("{nom} dans le QR élargi"));
            }
        }
        println!("pire_cas_boites_independantes : {mots} mots relevés dans la section de paiement");
        assert!(mots > 30, "montage : mots relevés ({mots})");
        assert!(ecarts.is_empty(), "{ecarts:#?}");
    }
}
