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

    draw_invoice_section(&layer, invoice, i18n, &helv, &helv_bold)?;
    draw_separator(&layer);
    draw_receipt(&layer, data, invoice, i18n, &helv, &helv_bold)?;
    draw_payment_part(&layer, data, invoice, i18n, &helv, &helv_bold, &qr)?;

    finalize(doc)
}

fn finalize(doc: PdfDocumentReference) -> Result<Vec<u8>, QrBillError> {
    doc.save_to_bytes()
        .map_err(|e| QrBillError::PdfGeneration(format!("save: {e}")))
}

// ----- Invoice top section -----

fn draw_invoice_section(
    layer: &PdfLayerReference,
    inv: &InvoicePdfData,
    i18n: &QrBillI18n,
    helv: &IndirectFontRef,
    helv_bold: &IndirectFontRef,
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
    if let Some(ide) = &inv.creditor_ide {
        layer.use_text(
            format!("{}: {}", i18n.get("invoice-pdf-ide"), ide),
            9.0,
            Mm(left),
            Mm(y),
            helv,
        );
        y -= 6.0;
    }

    // Title + metadata (right).
    let meta_x = 120.0;
    let mut my = PAGE_H - 20.0;
    layer.use_text(
        i18n.get("invoice-pdf-title"),
        18.0,
        Mm(meta_x),
        Mm(my),
        helv_bold,
    );
    my -= 7.0;
    layer.use_text(
        format!("{}: {}", i18n.get("invoice-pdf-number"), inv.invoice_number),
        9.0,
        Mm(meta_x),
        Mm(my),
        helv,
    );
    my -= 4.5;
    layer.use_text(
        format!(
            "{}: {}",
            i18n.get("invoice-pdf-date"),
            format_date_ch(inv.invoice_date)
        ),
        9.0,
        Mm(meta_x),
        Mm(my),
        helv,
    );
    if let Some(due) = inv.due_date {
        my -= 4.5;
        layer.use_text(
            format!(
                "{}: {}",
                i18n.get("invoice-pdf-due-date"),
                format_date_ch(due)
            ),
            9.0,
            Mm(meta_x),
            Mm(my),
            helv,
        );
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

    for (idx, line) in inv.lines.iter().enumerate() {
        if ty < SEP_Y + 15.0 {
            // Défense — si la capacité visuelle est dépassée, on refuse plutôt
            // que tronquer silencieusement. Le handler HTTP doit garder
            // `MAX_LINES_PER_PDF` aligné avec cette géométrie.
            return Err(QrBillError::PdfGeneration(format!(
                "trop de lignes de facture pour un PDF mono-page ({}+ lignes ; la ligne {} déborde sous la séparation QR)",
                inv.lines.len(),
                idx + 1
            )));
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

    // Total.
    ty -= 2.0;
    hline(layer, col_unit, PAGE_W - 20.0, ty);
    ty -= 5.0;
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
            Mm(ty.max(SEP_Y + 5.0)),
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
    use crate::types::{Address, AddressType, Currency, InvoiceLinePdf, QrBillData};
    use rust_decimal_macros::dec;

    fn invoice_fixture() -> (QrBillData, InvoicePdfData, QrBillI18n) {
        let creditor = Address {
            address_type: AddressType::Combined,
            name: "Robert Schneider SA".into(),
            line1: "Rue du Lac 1268".into(),
            line2: "2501 Biel".into(),
            country: "CH".into(),
        };
        let debtor = Address {
            address_type: AddressType::Combined,
            name: "Pia Rutschmann".into(),
            line1: "Marktgasse 28".into(),
            line2: "9400 Rorschach".into(),
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
            debtor_name: "Pia Rutschmann".into(),
            debtor_address_lines: vec!["Marktgasse 28".into(), "9400 Rorschach".into()],
            lines: vec![InvoiceLinePdf {
                description: "Conseil stratégique".into(),
                quantity: dec!(10),
                unit_price: dec!(120.00),
                vat_rate: dec!(7.70),
                line_total: dec!(1200.00),
            }],
            total: dec!(1234.50),
            currency: Currency::Chf,
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
}
