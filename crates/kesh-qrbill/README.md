# kesh-qrbill

Swiss QR Bill (SIX 2.2) payload builder and PDF generator for the Kesh accounting
platform.

**Positioning.** This crate is intentionally standalone — zero dependency on
`kesh-core` or `kesh-db` (architecture DD-14). It accepts plain Rust types plus
an injected i18n bundle and returns a `Vec<u8>` containing the full A4 invoice
+ payment part PDF. It can be published to crates.io independently.

## Public API

```rust
use kesh_qrbill::{generate_qr_bill_pdf, validate, QrBillData, InvoicePdfData, QrBillI18n};

let pdf_bytes: Vec<u8> = generate_qr_bill_pdf(&qr_data, &invoice_data, &i18n)?;
```

- [`build_payload`] — raw SIX payload string (32/33 lines, SIX 2.2 §3 order).
- [`validate`] — static validation (IBAN mod-97, QR-IBAN IID range, QRR
  mod-10 recursive checksum, field lengths, currency, amount bounds).
- [`generate_qr_bill_pdf`] — convenience entry (current UTC as creation date).
- [`generate_qr_bill_pdf_with_date`] — deterministic variant (fixed
  `CreationDate`/`ModDate`; the internal random `/ID` instance id remains
  out of scope — use payload-level golden comparison per Plan C of Story 5.3).

## Conformance

- **Spec** : `docs/six-references/ig-qr-bill-v2.4-en.pdf` (v2.4 is wire-compatible
  with the PRD-targeted v2.2).
- **ECC level** : M (QR Code recovery).
- **Address type** : K (Combined — 2 free-form address lines).
- **Reference types** : QRR (when `qr_iban` is provided) or NON (classic IBAN).
  SCOR not supported in v0.1.
- **IBAN country codes** : CH and LI only (SIX rule for any QR Bill).
- **Amount bounds** : 0.01 … 999'999'999.99 (SIX §3).

## Dependencies

- [`printpdf`](https://docs.rs/printpdf) 0.7 — PDF generation (MIT/Apache-2.0).
- [`qrcodegen`](https://docs.rs/qrcodegen) 1.8 — Nayuki QR encoder (MIT).
- `rust_decimal`, `chrono`, `thiserror`, `time` — workspace defaults.

Dev-dependencies:

- [`rxing`](https://docs.rs/rxing) 0.7 — QR decoder used for round-trip tests
  (Apache-2.0, dev-only — no runtime licensing implication).

## Testing

```bash
cargo test -p kesh-qrbill
```

Covers ≥ 20 cases across validation, payload assembly, QR matrix rendering,
round-trip decoding via `rxing`, Swiss number/date formatting, PDF byte-level
sanity (magic bytes + size).
