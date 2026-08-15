//! PDF generation for a Swiss QR Bill invoice (A4 portrait).
//!
//! Layout:
//! - Top: invoice header + lines table + total.
//! - Separator line at y = 105 mm (SIX §5: payment section occupies bottom 105 mm).
//! - Receipt column (62 mm wide) + Payment Part column (remaining width).
//! - QR Code 46×46 mm with 7×7 mm white square + Swiss cross in the center.
//!
//! Uses `BuiltinFont::Helvetica` (PDF standard 14) — no external font embedding.

use crate::generator::{build_payload, render_qr_image};
use crate::types::{InvoicePdfData, QrBillData, QrBillError, QrBillI18n, Reference};
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
    draw_receipt(&layer, data, invoice, i18n, &helv, &helv_bold)?;
    draw_payment_part(&layer, data, invoice, i18n, &helv, &helv_bold, &qr)?;

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
/// libre de l'écran *Paramètres → Facturation*, et `invoice_format` le laisse
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

    // Creditor header (top-left).
    layer.use_text(&inv.creditor_name, 14.0, Mm(left), Mm(y), helv_bold);
    y -= 5.0;
    for line in &inv.creditor_address_lines {
        layer.use_text(line, 9.0, Mm(left), Mm(y), helv);
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
    layer.use_text(
        i18n.get("invoice-pdf-title"),
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
    layer.use_text(&inv.debtor_name, 10.0, Mm(left), Mm(y), helv);
    y -= 4.5;
    for line in &inv.debtor_address_lines {
        layer.use_text(line, 9.0, Mm(left), Mm(y), helv);
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
    let recap_reserve = if inv.vat_lines.is_empty() {
        0.0
    } else {
        4.5 + 4.5 * inv.vat_lines.len() as f32 + 1.0
    };

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

    // Bloc récap seulement s'il existe des lignes taxées : Sous-total HT, puis
    // une ligne « TVA {taux}% » par taux. Sinon (société non assujettie / lignes
    // 0 %) on n'affiche que le total — comportement rétro-compatible.
    if !inv.vat_lines.is_empty() {
        let subtotal = inv
            .subtotal_ht
            .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
        layer.use_text(
            i18n.get("invoice-pdf-subtotal"),
            9.0,
            Mm(col_unit),
            Mm(ty),
            helv,
        );
        layer.use_text(
            format!("{} {}", inv.currency.code(), format_ch(subtotal, 2)),
            9.0,
            Mm(col_tot),
            Mm(ty),
            helv,
        );
        ty -= 4.5;
        for v in &inv.vat_lines {
            let amount = v
                .amount
                .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
            layer.use_text(
                format!(
                    "{} {}%",
                    i18n.get("invoice-pdf-vat"),
                    format_ch(v.rate_percent, 1)
                ),
                9.0,
                Mm(col_unit),
                Mm(ty),
                helv,
            );
            layer.use_text(
                format!("{} {}", inv.currency.code(), format_ch(amount, 2)),
                9.0,
                Mm(col_tot),
                Mm(ty),
                helv,
            );
            ty -= 4.5;
        }
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

// ----- Separator line at y = 105 mm -----

fn draw_separator(layer: &PdfLayerReference) {
    // Dotted line across the page width.
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.3);
    let mut x = 0.0_f32;
    while x < PAGE_W {
        let line = Line {
            points: vec![
                (Point::new(Mm(x), Mm(SEP_Y)), false),
                (Point::new(Mm((x + 2.0).min(PAGE_W)), Mm(SEP_Y)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
        x += 4.0;
    }
    // Scissors glyph is outside Helvetica's WinAnsi encoding; omitted in v0.1.
}

// ----- Receipt column (left, 62 mm) -----

fn draw_receipt(
    layer: &PdfLayerReference,
    data: &QrBillData,
    invoice: &InvoicePdfData,
    i18n: &QrBillI18n,
    helv: &IndirectFontRef,
    helv_bold: &IndirectFontRef,
) -> Result<(), QrBillError> {
    let x = 5.0;
    let top = SEP_Y - 5.0;

    layer.use_text(
        i18n.get("invoice-pdf-qr-section-receipt"),
        11.0,
        Mm(x),
        Mm(top),
        helv_bold,
    );

    let mut y = top - 7.0;
    layer.use_text(
        i18n.get("invoice-pdf-qr-account"),
        6.0,
        Mm(x),
        Mm(y),
        helv_bold,
    );
    y -= 3.0;
    layer.use_text(format_iban(&data.creditor_iban), 8.0, Mm(x), Mm(y), helv);
    y -= 3.5;
    layer.use_text(&data.creditor.name, 8.0, Mm(x), Mm(y), helv);
    y -= 3.5;
    layer.use_text(&data.creditor.line1, 8.0, Mm(x), Mm(y), helv);
    y -= 3.5;
    layer.use_text(&data.creditor.line2, 8.0, Mm(x), Mm(y), helv);

    // Amount + currency.
    y -= 8.0;
    layer.use_text(
        i18n.get("invoice-pdf-qr-currency"),
        6.0,
        Mm(x),
        Mm(y),
        helv_bold,
    );
    layer.use_text(
        i18n.get("invoice-pdf-qr-amount"),
        6.0,
        Mm(x + 12.0),
        Mm(y),
        helv_bold,
    );
    y -= 3.5;
    layer.use_text(invoice.currency.code(), 8.0, Mm(x), Mm(y), helv);
    let amount = data
        .amount
        .ok_or_else(|| QrBillError::InvalidAmount("montant requis pour rendu PDF".into()))?
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    layer.use_text(format_ch(amount, 2), 8.0, Mm(x + 12.0), Mm(y), helv);

    // Debtor.
    y -= 8.0;
    layer.use_text(
        i18n.get("invoice-pdf-qr-payable-by"),
        6.0,
        Mm(x),
        Mm(y),
        helv_bold,
    );
    if let Some(debtor) = &data.ultimate_debtor {
        y -= 3.5;
        layer.use_text(&debtor.name, 8.0, Mm(x), Mm(y), helv);
        y -= 3.5;
        layer.use_text(&debtor.line1, 8.0, Mm(x), Mm(y), helv);
        y -= 3.5;
        layer.use_text(&debtor.line2, 8.0, Mm(x), Mm(y), helv);
    }

    // "Acceptance point" at bottom-right of the receipt column.
    layer.use_text(
        i18n.get("invoice-pdf-qr-acceptance-point"),
        6.0,
        Mm(RECEIPT_W - 15.0),
        Mm(5.0),
        helv_bold,
    );

    Ok(())
}

// ----- Payment Part column (right) -----

#[allow(clippy::too_many_arguments)]
fn draw_payment_part(
    layer: &PdfLayerReference,
    data: &QrBillData,
    invoice: &InvoicePdfData,
    i18n: &QrBillI18n,
    helv: &IndirectFontRef,
    helv_bold: &IndirectFontRef,
    qr: &QrCode,
) -> Result<(), QrBillError> {
    let x0 = RECEIPT_W + 5.0;
    let top = SEP_Y - 5.0;

    layer.use_text(
        i18n.get("invoice-pdf-qr-section-payment"),
        11.0,
        Mm(x0),
        Mm(top),
        helv_bold,
    );

    // QR code 46×46 mm, top-left corner at (x0, top - 10).
    let qr_x = x0;
    let qr_top = top - 7.0;
    draw_qr_matrix(layer, qr, qr_x, qr_top, 46.0)?;
    draw_swiss_cross(layer, qr_x + 46.0 / 2.0, qr_top - 46.0 / 2.0);

    // Text column to the right of the QR code.
    let tx = qr_x + 50.0;
    let mut y = top - 7.0;
    layer.use_text(
        i18n.get("invoice-pdf-qr-account"),
        6.0,
        Mm(tx),
        Mm(y),
        helv_bold,
    );
    y -= 3.0;
    layer.use_text(format_iban(&data.creditor_iban), 8.0, Mm(tx), Mm(y), helv);
    y -= 3.5;
    layer.use_text(&data.creditor.name, 8.0, Mm(tx), Mm(y), helv);
    y -= 3.5;
    layer.use_text(&data.creditor.line1, 8.0, Mm(tx), Mm(y), helv);
    y -= 3.5;
    layer.use_text(&data.creditor.line2, 8.0, Mm(tx), Mm(y), helv);

    if let Reference::Qrr(qrr) = &data.reference {
        y -= 6.0;
        layer.use_text(
            i18n.get("invoice-pdf-qr-reference"),
            6.0,
            Mm(tx),
            Mm(y),
            helv_bold,
        );
        y -= 3.0;
        layer.use_text(format_qrr(qrr), 8.0, Mm(tx), Mm(y), helv);
    }

    if let Some(msg) = &data.unstructured_message {
        y -= 6.0;
        layer.use_text(
            i18n.get("invoice-pdf-qr-additional-info"),
            6.0,
            Mm(tx),
            Mm(y),
            helv_bold,
        );
        y -= 3.0;
        layer.use_text(msg, 8.0, Mm(tx), Mm(y), helv);
    }

    // Amount + currency (below QR).
    let amount_y = qr_top - 52.0;
    layer.use_text(
        i18n.get("invoice-pdf-qr-currency"),
        6.0,
        Mm(qr_x),
        Mm(amount_y),
        helv_bold,
    );
    layer.use_text(
        i18n.get("invoice-pdf-qr-amount"),
        6.0,
        Mm(qr_x + 15.0),
        Mm(amount_y),
        helv_bold,
    );
    let ay = amount_y - 3.5;
    layer.use_text(invoice.currency.code(), 8.0, Mm(qr_x), Mm(ay), helv);
    let amount = data
        .amount
        .ok_or_else(|| QrBillError::InvalidAmount("montant requis pour rendu PDF".into()))?
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    layer.use_text(format_ch(amount, 2), 8.0, Mm(qr_x + 15.0), Mm(ay), helv);

    // Debtor block (bottom-left).
    let dx = qr_x;
    let mut dy = ay - 8.0;
    layer.use_text(
        i18n.get("invoice-pdf-qr-payable-by"),
        6.0,
        Mm(dx),
        Mm(dy),
        helv_bold,
    );
    if let Some(debtor) = &data.ultimate_debtor {
        dy -= 3.5;
        layer.use_text(&debtor.name, 8.0, Mm(dx), Mm(dy), helv);
        dy -= 3.5;
        layer.use_text(&debtor.line1, 8.0, Mm(dx), Mm(dy), helv);
        dy -= 3.5;
        layer.use_text(&debtor.line2, 8.0, Mm(dx), Mm(dy), helv);
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
            total: dec!(1292.40), // 1200.00 + 92.40
            currency: Currency::Chf,
            origin_reference: None,
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
}
