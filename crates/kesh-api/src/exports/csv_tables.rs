//! Story 9-2b — 16 fonctions `serialize_<table>_csv` (Decision §scope-tables).
//!
//! Format CSV identique Story 9-2a §csv-format :
//! - UTF-8 BOM en tête (`\xEF\xBB\xBF`)
//! - Délimiteur `;`
//! - Terminator CRLF (`\r\n`)
//! - Escaping RFC 4180 automatique (chaînes contenant `;`/`"`/`\n` entourées
//!   de `"…"`, `"` interne doublé en `""`)
//! - Dates au format ISO 8601 (`YYYY-MM-DD` pour `NaiveDate`,
//!   `YYYY-MM-DDTHH:MM:SSZ` pour `NaiveDateTime` UTC)
//! - Decimal toujours formaté `{:.2}` (cohérent `kesh-report::csv::format_amount_iso`)
//!
//! Pattern privé `write_csv_bom` + `make_csv_writer` factorisés en haut du
//! fichier (Decision §csv-helper-reuse — duplication intentionnelle vs
//! `kesh-report::csv` pour respecter DD-12).
//!
//! Chaque serializer accepte n'importe quel `W: Write` (typiquement `Vec<u8>`
//! côté handler), retourne `Result<(), AppError>`. Une erreur csv interne est
//! mappée vers `AppError::GlobalExportFailed(format!("csv {table}: {e}"))`.

use std::io::Write;

use chrono::{NaiveDate, NaiveDateTime, SecondsFormat, TimeZone, Utc};
use rust_decimal::Decimal;

use crate::errors::AppError;
use kesh_db::entities::{
    Account, BankAccount, BankImport, BankProfile, BankTransaction, Company,
    CompanyInvoiceSettings, Contact, FiscalYear, Invoice, InvoiceLine, JournalEntry,
    JournalEntryLine, Product, ReconciliationRule, VatRate,
};

// ===========================================================================
// Helpers privés
// ===========================================================================

/// Écrit le BOM UTF-8 (`\xEF\xBB\xBF`) en tête du writer.
///
/// Indispensable pour qu'Excel CH/DE et LibreOffice détectent l'encoding
/// UTF-8 (sans BOM, Excel interprète le fichier en ISO-8859-1 → caractères
/// accentués cassés). Cohérent Story 9-2a `kesh-report::csv::*` qui injecte
/// le BOM avant chaque rapport.
fn write_csv_bom<W: Write>(writer: &mut W) -> Result<(), AppError> {
    writer
        .write_all(&[0xEF, 0xBB, 0xBF])
        .map_err(|e| AppError::GlobalExportFailed(format!("csv bom write: {e}")))
}

/// Crée un `csv::Writer` configuré pour le format Story 9-2a (`;` + CRLF +
/// RFC 4180 escaping automatique).
fn make_csv_writer<W: Write>(writer: W) -> csv::Writer<W> {
    csv::WriterBuilder::new()
        .delimiter(b';')
        .terminator(csv::Terminator::CRLF)
        .from_writer(writer)
}

/// Map une `csv::Error` vers `AppError::GlobalExportFailed`.
///
/// Le `table` permet de tracer rapidement quelle table a échoué dans les
/// logs serveur — utile pour diagnostiquer un cas avec une colonne corrompue
/// en DB.
fn map_csv_err(table: &str, e: csv::Error) -> AppError {
    AppError::GlobalExportFailed(format!("csv {table}: {e}"))
}

/// Map une erreur d'I/O générique (flush) vers `AppError::GlobalExportFailed`.
fn map_flush_err(table: &str, e: std::io::Error) -> AppError {
    AppError::GlobalExportFailed(format!("csv {table} flush: {e}"))
}

/// Format `NaiveDate` → `YYYY-MM-DD`.
fn fmt_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// Format `Option<NaiveDate>` → `YYYY-MM-DD` ou chaîne vide si `None`.
fn fmt_opt_date(d: Option<NaiveDate>) -> String {
    d.map(fmt_date).unwrap_or_default()
}

/// Format `NaiveDateTime` → `YYYY-MM-DDTHH:MM:SSZ` (UTC strict).
///
/// On suppose que la DB stocke les timestamps en UTC naïf (convention Kesh
/// — pas de timezone côté DB, l'API insère via `chrono::Utc::now()`).
fn fmt_dt(dt: NaiveDateTime) -> String {
    Utc.from_utc_datetime(&dt)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Format `Option<NaiveDateTime>` → ISO 8601 UTC ou chaîne vide.
fn fmt_opt_dt(dt: Option<NaiveDateTime>) -> String {
    dt.map(fmt_dt).unwrap_or_default()
}

/// Format `Decimal` → toujours 2 décimales (`{:.2}`).
fn fmt_decimal(d: Decimal) -> String {
    format!("{d:.2}")
}

/// Format `Option<Decimal>` → 2 décimales ou chaîne vide.
fn fmt_opt_decimal(d: Option<Decimal>) -> String {
    d.map(fmt_decimal).unwrap_or_default()
}

/// Format `Option<i64>` → `i64.to_string()` ou chaîne vide.
fn fmt_opt_i64(v: Option<i64>) -> String {
    v.map(|i| i.to_string()).unwrap_or_default()
}

/// Format `Option<String>` → string ou chaîne vide.
fn fmt_opt_str(s: &Option<String>) -> String {
    s.clone().unwrap_or_default()
}

/// Format `bool` → `"true"` / `"false"` (interopérable Excel / Sheets).
fn fmt_bool(b: bool) -> String {
    if b { "true".into() } else { "false".into() }
}

// ===========================================================================
// 16 serializers (ordre §scope-tables)
// ===========================================================================

/// Sérialise une `Company` en CSV 1-row (referential principal du tenant).
pub fn serialize_company_csv<W: Write>(rows: &[Company], writer: W) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "name",
        "address",
        "ide_number",
        "org_type",
        "accounting_language",
        "instance_language",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("company", e))?;
    for c in rows {
        csv.write_record([
            c.id.to_string(),
            c.name.clone(),
            c.address.clone(),
            fmt_opt_str(&c.ide_number),
            c.org_type.as_str().to_string(),
            c.accounting_language.as_str().to_string(),
            c.instance_language.as_str().to_string(),
            c.version.to_string(),
            fmt_dt(c.created_at),
            fmt_dt(c.updated_at),
        ])
        .map_err(|e| map_csv_err("company", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("company", e))
}

/// Sérialise les `FiscalYear` en CSV.
pub fn serialize_fiscal_years_csv<W: Write>(
    rows: &[FiscalYear],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "name",
        "start_date",
        "end_date",
        "status",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("fiscal_years", e))?;
    for fy in rows {
        csv.write_record([
            fy.id.to_string(),
            fy.company_id.to_string(),
            fy.name.clone(),
            fmt_date(fy.start_date),
            fmt_date(fy.end_date),
            fy.status.as_str().to_string(),
            fmt_dt(fy.created_at),
            fmt_dt(fy.updated_at),
        ])
        .map_err(|e| map_csv_err("fiscal_years", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("fiscal_years", e))
}

/// Sérialise les `Account` en CSV (plan comptable, archives incluses).
pub fn serialize_accounts_csv<W: Write>(rows: &[Account], writer: W) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "number",
        "name",
        "account_type",
        "parent_id",
        "active",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("accounts", e))?;
    for a in rows {
        csv.write_record([
            a.id.to_string(),
            a.company_id.to_string(),
            a.number.clone(),
            a.name.clone(),
            a.account_type.as_str().to_string(),
            fmt_opt_i64(a.parent_id),
            fmt_bool(a.active),
            a.version.to_string(),
            fmt_dt(a.created_at),
            fmt_dt(a.updated_at),
        ])
        .map_err(|e| map_csv_err("accounts", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("accounts", e))
}

/// Sérialise les en-têtes `JournalEntry` en CSV (cœur comptable).
pub fn serialize_journal_entries_csv<W: Write>(
    rows: &[JournalEntry],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "fiscal_year_id",
        "entry_number",
        "entry_date",
        "journal",
        "description",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("journal_entries", e))?;
    for je in rows {
        csv.write_record([
            je.id.to_string(),
            je.company_id.to_string(),
            je.fiscal_year_id.to_string(),
            je.entry_number.to_string(),
            fmt_date(je.entry_date),
            je.journal.as_str().to_string(),
            je.description.clone(),
            je.version.to_string(),
            fmt_dt(je.created_at),
            fmt_dt(je.updated_at),
        ])
        .map_err(|e| map_csv_err("journal_entries", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("journal_entries", e))
}

/// Sérialise les lignes `JournalEntryLine` en CSV.
pub fn serialize_journal_entry_lines_csv<W: Write>(
    rows: &[JournalEntryLine],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "entry_id",
        "account_id",
        "line_order",
        "debit",
        "credit",
    ])
    .map_err(|e| map_csv_err("journal_entry_lines", e))?;
    for jl in rows {
        csv.write_record([
            jl.id.to_string(),
            jl.entry_id.to_string(),
            jl.account_id.to_string(),
            jl.line_order.to_string(),
            fmt_decimal(jl.debit),
            fmt_decimal(jl.credit),
        ])
        .map_err(|e| map_csv_err("journal_entry_lines", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("journal_entry_lines", e))
}

/// Sérialise les `Contact` en CSV (carnet d'adresses, archives incluses).
pub fn serialize_contacts_csv<W: Write>(rows: &[Contact], writer: W) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "contact_type",
        "name",
        "is_client",
        "is_supplier",
        "address",
        "email",
        "phone",
        "ide_number",
        "default_payment_terms",
        "active",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("contacts", e))?;
    for c in rows {
        csv.write_record([
            c.id.to_string(),
            c.company_id.to_string(),
            c.contact_type.as_str().to_string(),
            c.name.clone(),
            fmt_bool(c.is_client),
            fmt_bool(c.is_supplier),
            fmt_opt_str(&c.address),
            fmt_opt_str(&c.email),
            fmt_opt_str(&c.phone),
            fmt_opt_str(&c.ide_number),
            fmt_opt_str(&c.default_payment_terms),
            fmt_bool(c.active),
            c.version.to_string(),
            fmt_dt(c.created_at),
            fmt_dt(c.updated_at),
        ])
        .map_err(|e| map_csv_err("contacts", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("contacts", e))
}

/// Sérialise les `Product` en CSV (catalogue, archives incluses).
pub fn serialize_products_csv<W: Write>(rows: &[Product], writer: W) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "name",
        "description",
        "unit_price",
        "vat_rate",
        "active",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("products", e))?;
    for p in rows {
        csv.write_record([
            p.id.to_string(),
            p.company_id.to_string(),
            p.name.clone(),
            fmt_opt_str(&p.description),
            fmt_decimal(p.unit_price),
            fmt_decimal(p.vat_rate),
            fmt_bool(p.active),
            p.version.to_string(),
            fmt_dt(p.created_at),
            fmt_dt(p.updated_at),
        ])
        .map_err(|e| map_csv_err("products", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("products", e))
}

/// Sérialise les en-têtes `Invoice` en CSV (drafts + validated + paid).
pub fn serialize_invoices_csv<W: Write>(rows: &[Invoice], writer: W) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "contact_id",
        "invoice_number",
        "status",
        "date",
        "due_date",
        "payment_terms",
        "total_amount",
        "journal_entry_id",
        "paid_at",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("invoices", e))?;
    for i in rows {
        csv.write_record([
            i.id.to_string(),
            i.company_id.to_string(),
            i.contact_id.to_string(),
            fmt_opt_str(&i.invoice_number),
            i.status.clone(),
            fmt_date(i.date),
            fmt_opt_date(i.due_date),
            fmt_opt_str(&i.payment_terms),
            fmt_decimal(i.total_amount),
            fmt_opt_i64(i.journal_entry_id),
            fmt_opt_dt(i.paid_at),
            i.version.to_string(),
            fmt_dt(i.created_at),
            fmt_dt(i.updated_at),
        ])
        .map_err(|e| map_csv_err("invoices", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("invoices", e))
}

/// Sérialise les lignes `InvoiceLine` en CSV.
pub fn serialize_invoice_lines_csv<W: Write>(
    rows: &[InvoiceLine],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "invoice_id",
        "position",
        "description",
        "quantity",
        "unit_price",
        "vat_rate",
        "line_total",
        "created_at",
    ])
    .map_err(|e| map_csv_err("invoice_lines", e))?;
    for il in rows {
        csv.write_record([
            il.id.to_string(),
            il.invoice_id.to_string(),
            il.position.to_string(),
            il.description.clone(),
            fmt_decimal(il.quantity),
            fmt_decimal(il.unit_price),
            fmt_decimal(il.vat_rate),
            fmt_decimal(il.line_total),
            fmt_dt(il.created_at),
        ])
        .map_err(|e| map_csv_err("invoice_lines", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("invoice_lines", e))
}

/// Sérialise les `BankAccount` en CSV.
pub fn serialize_bank_accounts_csv<W: Write>(
    rows: &[BankAccount],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "bank_name",
        "iban",
        "qr_iban",
        "is_primary",
        "journal_account_id",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("bank_accounts", e))?;
    for b in rows {
        csv.write_record([
            b.id.to_string(),
            b.company_id.to_string(),
            b.bank_name.clone(),
            b.iban.clone(),
            fmt_opt_str(&b.qr_iban),
            fmt_bool(b.is_primary),
            fmt_opt_i64(b.journal_account_id),
            b.version.to_string(),
            fmt_dt(b.created_at),
            fmt_dt(b.updated_at),
        ])
        .map_err(|e| map_csv_err("bank_accounts", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("bank_accounts", e))
}

/// Sérialise les `BankImport` en CSV (historique d'imports CAMT.053 / CSV).
pub fn serialize_bank_imports_csv<W: Write>(
    rows: &[BankImport],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "bank_account_id",
        "filename",
        "file_hash",
        "source_format",
        "statement_id",
        "period_from",
        "period_to",
        "opening_balance",
        "closing_balance",
        "transaction_count",
        "imported_at",
        "imported_by_user_id",
    ])
    .map_err(|e| map_csv_err("bank_imports", e))?;
    for bi in rows {
        csv.write_record([
            bi.id.to_string(),
            bi.company_id.to_string(),
            bi.bank_account_id.to_string(),
            bi.filename.clone(),
            bi.file_hash.clone(),
            bi.source_format.as_str().to_string(),
            fmt_opt_str(&bi.statement_id),
            fmt_date(bi.period_from),
            fmt_date(bi.period_to),
            fmt_opt_decimal(bi.opening_balance),
            fmt_opt_decimal(bi.closing_balance),
            bi.transaction_count.to_string(),
            fmt_dt(bi.imported_at),
            bi.imported_by_user_id.to_string(),
        ])
        .map_err(|e| map_csv_err("bank_imports", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("bank_imports", e))
}

/// Sérialise les `BankTransaction` en CSV.
pub fn serialize_bank_transactions_csv<W: Write>(
    rows: &[BankTransaction],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "import_id",
        "bank_account_id",
        "booking_date",
        "value_date",
        "amount",
        "currency",
        "reference",
        "details",
        "end_to_end_id",
        "transaction_id",
        "counterparty_iban",
        "counterparty_name",
        "status",
        "matched_entry_id",
        "auto_match_rejected_at",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("bank_transactions", e))?;
    for bt in rows {
        csv.write_record([
            bt.id.to_string(),
            bt.company_id.to_string(),
            bt.import_id.to_string(),
            bt.bank_account_id.to_string(),
            fmt_date(bt.booking_date),
            fmt_opt_date(bt.value_date),
            fmt_decimal(bt.amount),
            bt.currency.clone(),
            fmt_opt_str(&bt.reference),
            bt.details.clone(),
            fmt_opt_str(&bt.end_to_end_id),
            fmt_opt_str(&bt.transaction_id),
            fmt_opt_str(&bt.counterparty_iban),
            fmt_opt_str(&bt.counterparty_name),
            bt.status.as_str().to_string(),
            fmt_opt_i64(bt.matched_entry_id),
            fmt_opt_dt(bt.auto_match_rejected_at),
            bt.version.to_string(),
            fmt_dt(bt.created_at),
            fmt_dt(bt.updated_at),
        ])
        .map_err(|e| map_csv_err("bank_transactions", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("bank_transactions", e))
}

/// Sérialise les `VatRate` en CSV (actifs + historiques).
pub fn serialize_vat_rates_csv<W: Write>(rows: &[VatRate], writer: W) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        // Story 11-1 : `category` (discriminant métier) + `version` ajoutés à
        // l'export de souveraineté — sans `category`, un CSV historique serait
        // inexploitable pour reconstituer les calculs TVA par catégorie (11-2).
        "category",
        "label",
        "rate",
        "valid_from",
        "valid_to",
        "active",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("vat_rates", e))?;
    for v in rows {
        csv.write_record([
            v.id.to_string(),
            v.company_id.to_string(),
            v.category.clone(),
            v.label.clone(),
            fmt_decimal(v.rate),
            fmt_date(v.valid_from),
            fmt_opt_date(v.valid_to),
            fmt_bool(v.active),
            v.version.to_string(),
            fmt_dt(v.created_at),
            fmt_dt(v.updated_at),
        ])
        .map_err(|e| map_csv_err("vat_rates", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("vat_rates", e))
}

/// Sérialise un `CompanyInvoiceSettings` (1 row, lazy-create acceptée côté
/// handler — cf. §scope-tables row 14).
pub fn serialize_company_invoice_settings_csv<W: Write>(
    rows: &[CompanyInvoiceSettings],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "company_id",
        "invoice_number_format",
        "default_receivable_account_id",
        "default_revenue_account_id",
        "default_sales_journal",
        "journal_entry_description_template",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("company_invoice_settings", e))?;
    for cis in rows {
        csv.write_record([
            cis.company_id.to_string(),
            cis.invoice_number_format.clone(),
            fmt_opt_i64(cis.default_receivable_account_id),
            fmt_opt_i64(cis.default_revenue_account_id),
            cis.default_sales_journal.as_str().to_string(),
            cis.journal_entry_description_template.clone(),
            cis.version.to_string(),
            fmt_dt(cis.created_at),
            fmt_dt(cis.updated_at),
        ])
        .map_err(|e| map_csv_err("company_invoice_settings", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("company_invoice_settings", e))
}

/// Sérialise les `ReconciliationRule` en CSV (actives + soft-deleted).
pub fn serialize_reconciliation_rules_csv<W: Write>(
    rows: &[ReconciliationRule],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "label",
        "match_type",
        "match_value",
        "counterparty_account_id",
        "priority",
        "active",
        "applied_count",
        "last_applied_at",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("reconciliation_rules", e))?;
    for r in rows {
        csv.write_record([
            r.id.to_string(),
            r.company_id.to_string(),
            r.label.clone(),
            r.match_type.as_str().to_string(),
            r.match_value.clone(),
            r.counterparty_account_id.to_string(),
            r.priority.to_string(),
            fmt_bool(r.active),
            r.applied_count.to_string(),
            fmt_opt_dt(r.last_applied_at),
            r.version.to_string(),
            fmt_dt(r.created_at),
            fmt_dt(r.updated_at),
        ])
        .map_err(|e| map_csv_err("reconciliation_rules", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("reconciliation_rules", e))
}

/// Sérialise les `BankProfile` en CSV (profils CSV configurés).
pub fn serialize_bank_profiles_csv<W: Write>(
    rows: &[BankProfile],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "bank_name",
        "filename_pattern",
        "column_mapping_json",
        "date_format",
        "decimal_separator",
        "field_separator",
        "encoding",
        "header_row_count",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("bank_profiles", e))?;
    for bp in rows {
        csv.write_record([
            bp.id.to_string(),
            bp.company_id.to_string(),
            bp.bank_name.clone(),
            fmt_opt_str(&bp.filename_pattern),
            bp.column_mapping_json.clone(),
            bp.date_format.clone(),
            bp.decimal_separator.clone(),
            bp.field_separator.clone(),
            fmt_opt_str(&bp.encoding),
            bp.header_row_count.to_string(),
            fmt_dt(bp.created_at),
            fmt_dt(bp.updated_at),
        ])
        .map_err(|e| map_csv_err("bank_profiles", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("bank_profiles", e))
}

// ===========================================================================
// Tests unit (Story 9-2b T2.6 + AC #30 / T10.1)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use kesh_db::entities::{AccountType, Journal as JournalEnum};
    use rust_decimal_macros::dec;

    fn naive_dt(y: i32, m: u32, d: u32, h: u32, mi: u32, s: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(h, mi, s).unwrap())
    }

    fn sample_account() -> Account {
        Account {
            id: 1,
            company_id: 42,
            number: "1000".into(),
            name: "Caisse; \"principale\"".into(), // RFC 4180 escape test
            account_type: AccountType::Asset,
            parent_id: None,
            active: true,
            version: 1,
            created_at: naive_dt(2026, 1, 1, 0, 0, 0),
            updated_at: naive_dt(2026, 1, 2, 12, 30, 45),
        }
    }

    fn sample_entry() -> JournalEntry {
        JournalEntry {
            id: 10,
            company_id: 42,
            fiscal_year_id: 1,
            entry_number: 7,
            entry_date: NaiveDate::from_ymd_opt(2026, 5, 17).unwrap(),
            journal: JournalEnum::Ventes,
            description: "Vente test".into(),
            version: 1,
            created_at: naive_dt(2026, 5, 17, 8, 15, 0),
            updated_at: naive_dt(2026, 5, 17, 8, 15, 0),
        }
    }

    fn sample_line() -> JournalEntryLine {
        JournalEntryLine {
            id: 100,
            entry_id: 10,
            account_id: 1,
            line_order: 1,
            debit: dec!(150.25),
            credit: dec!(0.00),
        }
    }

    // ----- AC #30(a) — accounts serializer -----

    #[test]
    fn serialize_accounts_csv_bom_header_and_iso_dates() {
        let mut buf = Vec::new();
        serialize_accounts_csv(&[sample_account()], &mut buf).expect("serialize ok");
        // BOM UTF-8
        assert_eq!(&buf[0..3], &[0xEF, 0xBB, 0xBF], "missing UTF-8 BOM");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        // Header présent
        assert!(text.starts_with("id;company_id;number;name"), "{text}");
        // CRLF
        assert!(text.contains("\r\n"), "expected CRLF terminator");
        // RFC 4180 escape : nom contient `;` et `"` → double-quoted + `""` interne
        assert!(
            text.contains("\"Caisse; \"\"principale\"\"\""),
            "missing RFC 4180 escape, got: {text}"
        );
        // ISO 8601 UTC `Z` sur created_at
        assert!(
            text.contains("2026-01-02T12:30:45Z"),
            "missing ISO 8601 UTC, got: {text}"
        );
    }

    #[test]
    fn serialize_accounts_csv_empty_produces_bom_header_only() {
        let mut buf = Vec::new();
        serialize_accounts_csv(&[], &mut buf).expect("serialize empty ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        // Une seule ligne (le header) + CRLF terminator
        let data_lines: Vec<&str> = text.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), 1, "expected header only, got: {text}");
        assert!(data_lines[0].starts_with("id;company_id;number"), "{text}");
    }

    // ----- AC #30(b) — journal_entries serializer -----

    #[test]
    fn serialize_journal_entries_csv_iso_date_and_journal_enum() {
        let mut buf = Vec::new();
        serialize_journal_entries_csv(&[sample_entry()], &mut buf).expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        // entry_date ISO 8601 (YYYY-MM-DD)
        assert!(
            text.contains("2026-05-17;"),
            "missing ISO date, got: {text}"
        );
        // journal enum string (as_str())
        assert!(text.contains(";Ventes;"), "missing journal enum: {text}");
    }

    // ----- AC #30(c) — journal_entry_lines serializer -----

    #[test]
    fn serialize_journal_entry_lines_csv_decimal_two_decimals() {
        let mut buf = Vec::new();
        serialize_journal_entry_lines_csv(&[sample_line()], &mut buf).expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        // Decimal formaté 2 décimales — debit 150.25 + credit 0.00
        assert!(text.contains(";150.25;0.00\r\n"), "got: {text}");
        // Header
        assert!(text.starts_with("id;entry_id;account_id;line_order;debit;credit"));
    }

    #[test]
    fn serialize_journal_entry_lines_csv_empty_produces_bom_header_only() {
        let mut buf = Vec::new();
        serialize_journal_entry_lines_csv(&[], &mut buf).expect("serialize empty ok");
        assert_eq!(&buf[0..3], &[0xEF, 0xBB, 0xBF]);
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        assert_eq!(
            text.lines().count(),
            1,
            "expected header-only, got: {text:?}"
        );
    }
}
