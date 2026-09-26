//! Story 9-2b — les fonctions `serialize_<table>_csv` (Decision §scope-tables).
//!
//! ⚠️ **Ce nombre était écrit ici, et il mentait** : l'en-tête annonçait « 16
//! fonctions » alors qu'il y en avait **19**, puis **30** avec la 25-5-a. Il a
//! été retiré plutôt que corrigé — *un décompte qu'aucun calcul ne tient se
//! périme en silence, et celui-ci l'avait fait deux fois.* Le seul nombre qui
//! compte est désormais **dérivé** : cf. `TABLES_EXPORTEES` (`exports/global.rs`).
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
use kesh_db::entities::audit_log::AuditLogEntry;
use kesh_db::entities::contact_person::ContactPerson;
use kesh_db::entities::imported_supplier_invoice::ImportedSupplierInvoice;
use kesh_db::entities::{
    Account, BankAccount, BankImport, BankProfile, BankTransaction, Company,
    CompanyDunningSettings, CompanyInvoiceSettings, Contact, CreditNote, CreditNoteLine,
    DunningLevel, FiscalYear, Invoice, InvoiceLine, InvoiceReminder, InvoiceSettlement,
    JournalEntry, JournalEntryLine, PaymentBatch, PaymentBatchItem, Product, Project,
    ReconciliationRule, SupplierInvoice, SupplierInvoiceLine, VatRate,
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
    txt(s.clone().unwrap_or_default())
}

/// Toute cellule de **texte libre** passe par ici (Story 25-5-a, #386).
///
/// ⛔ **`csv_sanitize` existait et cet export ne l'employait NULLE PART.** Elle
/// avait été extraite de `routes::invoices` par la 25-1c-a précisément pour
/// être réutilisée ; l'export de souveraineté, qui produit le fichier qu'un
/// utilisateur ouvrira dans un tableur pour migrer, était passé à côté.
///
/// Une cellule commençant par `=`, `+`, `-` ou `@` est interprétée comme une
/// **formule** par Excel et LibreOffice. Un nom de contact, une description de
/// ligne ou un `details_json` d'audit sont du texte que l'utilisateur contrôle.
///
/// ⚠️ La 25-5-a ajoute onze tables **pleines de texte libre** — libellés
/// d'avoirs, descriptions de lignes fournisseurs, noms de projets, personnes de
/// contact, instantanés d'audit —, donc elle élargissait la surface sans
/// fermer le trou. *L'omission est antérieure ; c'est cette story qui la rendait
/// coûteuse.*
fn txt(s: String) -> String {
    crate::util::csv_sanitize(s)
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
        "first_name",
        "last_name",
        "address",
        "address_street",
        "address_building",
        "address_postal_code",
        "address_city",
        "address_country",
        "email",
        "phone",
        "website",
        "ide_number",
        "org_type",
        "accounting_language",
        "instance_language",
        // Story 24-4c (#380) : sans cette colonne, l'export de souverainete ne
        // dirait pas jusqu'ou les livres sont arretes -- une information que le
        // reviseur cherche en premier. Elle est ici parce qu'un export complet
        // doit l'etre, PAS parce qu'elle protegerait d'une restauration : ce
        // CSV n'a aucun importeur (cf. l'audit `books.restored`).
        "books_locked_through",
        "is_stub",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("company", e))?;
    for c in rows {
        csv.write_record([
            c.id.to_string(),
            txt(c.name.clone()),
            fmt_opt_str(&c.first_name),
            fmt_opt_str(&c.last_name),
            txt(c.address.clone()),
            txt(c.address_street.clone()),
            txt(c.address_building.clone()),
            txt(c.address_postal_code.clone()),
            txt(c.address_city.clone()),
            txt(c.address_country.clone()),
            fmt_opt_str(&c.email),
            fmt_opt_str(&c.phone),
            fmt_opt_str(&c.website),
            fmt_opt_str(&c.ide_number),
            c.org_type.as_str().to_string(),
            c.accounting_language.as_str().to_string(),
            c.instance_language.as_str().to_string(),
            c.books_locked_through
                .map(|d| d.to_string())
                .unwrap_or_default(),
            fmt_bool(c.is_stub),
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
            txt(fy.name.clone()),
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
        // Story 14-3a — l'en-tête est écrit à la main : tout champ ajouté à
        // `Account` doit être répercuté ici ET dans le write_record ci-dessous.
        "role",
        "postable",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("accounts", e))?;
    for a in rows {
        csv.write_record([
            a.id.to_string(),
            a.company_id.to_string(),
            txt(a.number.clone()),
            txt(a.name.clone()),
            a.account_type.as_str().to_string(),
            fmt_opt_i64(a.parent_id),
            fmt_bool(a.active),
            a.role.map(|r| r.as_str().to_string()).unwrap_or_default(),
            fmt_bool(a.postable),
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
        // ⛔ snake_case : cet en-tête est un export TABLE, pas une réponse JSON.
        // ⚠️ Sans cette colonne, l'export comptable complet — celui que le
        // réviseur consulte — ne montrerait AUCUNE trace d'une contre-passation,
        // en contradiction avec l'exigence (art. 958f CO) que la Story 24-4a sert.
        // Et rien ne rougirait : `backup_inventory_matches_schema` ne compare que
        // la liste des TABLES, jamais les colonnes d'un CSV.
        "reverses_entry_id",
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
            txt(je.description.clone()),
            je.version.to_string(),
            je.reverses_entry_id
                .map(|v| v.to_string())
                .unwrap_or_default(),
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
        "project_id",
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
            // Tag analytique (Epic 19) — vide si ligne non taguée.
            jl.project_id.map(|p| p.to_string()).unwrap_or_default(),
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
        "first_name",
        "last_name",
        "salutation",
        "language",
        "client_number",
        "is_client",
        "is_supplier",
        "address",
        "address_street",
        "address_building",
        "address_postal_code",
        "address_city",
        "address_country",
        "email",
        "phone",
        "ide_number",
        "default_payment_terms",
        "default_payment_terms_days",
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
            txt(c.name.clone()),
            fmt_opt_str(&c.first_name),
            fmt_opt_str(&c.last_name),
            c.salutation.as_str().to_string(),
            c.language
                .map(|l| l.as_str().to_string())
                .unwrap_or_default(),
            fmt_opt_str(&c.client_number),
            fmt_bool(c.is_client),
            fmt_bool(c.is_supplier),
            fmt_opt_str(&c.address),
            fmt_opt_str(&c.address_street),
            fmt_opt_str(&c.address_building),
            fmt_opt_str(&c.address_postal_code),
            fmt_opt_str(&c.address_city),
            fmt_opt_str(&c.address_country),
            fmt_opt_str(&c.email),
            fmt_opt_str(&c.phone),
            fmt_opt_str(&c.ide_number),
            fmt_opt_str(&c.default_payment_terms),
            c.default_payment_terms_days
                .map(|d| d.to_string())
                .unwrap_or_default(),
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
        "default_revenue_account_id",
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
            txt(p.name.clone()),
            fmt_opt_str(&p.description),
            fmt_decimal(p.unit_price),
            fmt_decimal(p.vat_rate),
            fmt_opt_i64(p.default_revenue_account_id),
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
        // #262 : envoi e-mail (Epic 20) — omis jusqu'ici de l'export souveraineté.
        "emailed_at",
        "emailed_to",
        "project_id",
        // #262 : suspension de relance (Story 21-5a) — idem, désormais exportée.
        "dunning_paused_at",
        "dunning_paused_note",
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
            txt(i.status.clone()),
            fmt_date(i.date),
            fmt_opt_date(i.due_date),
            fmt_opt_str(&i.payment_terms),
            fmt_decimal(i.total_amount),
            fmt_opt_i64(i.journal_entry_id),
            fmt_opt_dt(i.paid_at),
            // Dernier envoi e-mail (Epic 20, Story 20-3b1).
            fmt_opt_dt(i.emailed_at),
            fmt_opt_str(&i.emailed_to),
            // Tag analytique document-level (Epic 19, Story 19-4).
            fmt_opt_i64(i.project_id),
            // Suspension de relance débiteur (Story 21-5a).
            fmt_opt_dt(i.dunning_paused_at),
            fmt_opt_str(&i.dunning_paused_note),
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
        // Story 16-1a : compte de produit de la ligne. Vide = la ligne suit le
        // compte par défaut de la société. Trois causes, et la troisième est de
        // loin la plus fréquente : brouillon ; facture dont l'écriture a été
        // retouchée à la main, que le backfill de 16-1a-bis refuse
        // délibérément de reprendre (D-B2) ; facture `cancelled`, que le même
        // backfill exclut par construction (D-B5) — donc TOUTE facture ayant
        // reçu un avoir.
        "revenue_account_id",
        "created_at",
    ])
    .map_err(|e| map_csv_err("invoice_lines", e))?;
    for il in rows {
        csv.write_record([
            il.id.to_string(),
            il.invoice_id.to_string(),
            il.position.to_string(),
            txt(il.description.clone()),
            fmt_decimal(il.quantity),
            fmt_decimal(il.unit_price),
            fmt_decimal(il.vat_rate),
            fmt_decimal(il.line_total),
            fmt_opt_i64(il.revenue_account_id),
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
        "archived",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("bank_accounts", e))?;
    for b in rows {
        csv.write_record([
            b.id.to_string(),
            b.company_id.to_string(),
            txt(b.bank_name.clone()),
            txt(b.iban.clone()),
            fmt_opt_str(&b.qr_iban),
            fmt_bool(b.is_primary),
            fmt_opt_i64(b.journal_account_id),
            fmt_bool(b.archived),
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
            txt(bi.filename.clone()),
            txt(bi.file_hash.clone()),
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
            txt(bt.currency.clone()),
            fmt_opt_str(&bt.reference),
            txt(bt.details.clone()),
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
            txt(v.category.clone()),
            txt(v.label.clone()),
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

/// Sérialise les `DunningLevel` (Story 21-3, #231) — niveaux de rappel configurés.
pub fn serialize_dunning_levels_csv<W: Write>(
    rows: &[DunningLevel],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "level_number",
        "delay_days",
        "fee_amount",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("dunning_levels", e))?;
    for l in rows {
        csv.write_record([
            l.id.to_string(),
            l.company_id.to_string(),
            l.level_number.to_string(),
            l.delay_days.to_string(),
            fmt_decimal(l.fee_amount),
            l.version.to_string(),
            fmt_dt(l.created_at),
            fmt_dt(l.updated_at),
        ])
        .map_err(|e| map_csv_err("dunning_levels", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("dunning_levels", e))
}

/// Sérialise un `CompanyDunningSettings` (1 row, lazy-create côté handler).
pub fn serialize_company_dunning_settings_csv<W: Write>(
    rows: &[CompanyDunningSettings],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "company_id",
        "grace_period_days",
        "seeded_at",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("company_dunning_settings", e))?;
    for s in rows {
        csv.write_record([
            s.company_id.to_string(),
            s.grace_period_days.to_string(),
            fmt_opt_dt(s.seeded_at),
            s.version.to_string(),
            fmt_dt(s.created_at),
            fmt_dt(s.updated_at),
        ])
        .map_err(|e| map_csv_err("company_dunning_settings", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("company_dunning_settings", e))
}

/// Sérialise l'historique `invoice_reminders` (Story 21-5a) — append-only, snapshots.
pub fn serialize_invoice_reminders_csv<W: Write>(
    rows: &[InvoiceReminder],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "invoice_id",
        "level_number",
        "fee_amount",
        "sent_at",
        "channel",
        "sent_to",
        "subject",
        "body",
        "note",
        "actor_user_id",
        "cancelled_at",
        "created_at",
    ])
    .map_err(|e| map_csv_err("invoice_reminders", e))?;
    for r in rows {
        csv.write_record([
            r.id.to_string(),
            r.company_id.to_string(),
            r.invoice_id.to_string(),
            r.level_number.to_string(),
            fmt_decimal(r.fee_amount),
            fmt_dt(r.sent_at),
            txt(r.channel.clone()),
            fmt_opt_str(&r.sent_to),
            txt(r.subject.clone()),
            txt(r.body.clone()),
            fmt_opt_str(&r.note),
            r.actor_user_id.map(|v| v.to_string()).unwrap_or_default(),
            fmt_opt_dt(r.cancelled_at),
            fmt_dt(r.created_at),
        ])
        .map_err(|e| map_csv_err("invoice_reminders", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("invoice_reminders", e))
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
        "default_vat_payable_account_id",
        "default_vat_recoverable_account_id",
        "default_vat_decompte_account_id",
        "default_sales_journal",
        "journal_entry_description_template",
        "credit_note_number_format",
        "default_payable_account_id",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("company_invoice_settings", e))?;
    for cis in rows {
        csv.write_record([
            cis.company_id.to_string(),
            txt(cis.invoice_number_format.clone()),
            fmt_opt_i64(cis.default_receivable_account_id),
            fmt_opt_i64(cis.default_revenue_account_id),
            fmt_opt_i64(cis.default_vat_payable_account_id),
            fmt_opt_i64(cis.default_vat_recoverable_account_id),
            fmt_opt_i64(cis.default_vat_decompte_account_id),
            cis.default_sales_journal.as_str().to_string(),
            txt(cis.journal_entry_description_template.clone()),
            txt(cis.credit_note_number_format.clone()),
            fmt_opt_i64(cis.default_payable_account_id),
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
        "default_project_id",
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
            txt(r.label.clone()),
            r.match_type.as_str().to_string(),
            txt(r.match_value.clone()),
            r.counterparty_account_id.to_string(),
            r.priority.to_string(),
            fmt_bool(r.active),
            fmt_opt_i64(r.default_project_id),
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
            txt(bp.bank_name.clone()),
            fmt_opt_str(&bp.filename_pattern),
            txt(bp.column_mapping_json.clone()),
            txt(bp.date_format.clone()),
            txt(bp.decimal_separator.clone()),
            txt(bp.field_separator.clone()),
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

// ---------------------------------------------------------------------------
// Story 25-5-a (#386) — les onze tables comptables qui manquaient
// ---------------------------------------------------------------------------
//
// ⛔ **Onze tables, et le trou était DEUX FOIS plus large que l'issue.** Elle en
// annonçait dix ; il y en avait vingt et une, dont `invoice_settlements`, née à
// l'Epic 24 — c'est-à-dire APRÈS l'écriture de l'issue. *L'export se périmait à
// chaque epic, en silence.* Les dix restantes sont exclues avec motif écrit,
// et la garde `export_couvre_toutes_les_tables` refuse désormais qu'une table
// soit ni exportée ni exclue.

pub fn serialize_credit_notes_csv<W: Write>(
    rows: &[CreditNote],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "contact_id",
        "invoice_id",
        "credit_note_number",
        "status",
        "date",
        "total_amount",
        "journal_entry_id",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("credit_notes", e))?;
    for cn in rows {
        csv.write_record([
            cn.id.to_string(),
            cn.company_id.to_string(),
            cn.contact_id.to_string(),
            cn.invoice_id.to_string(),
            fmt_opt_str(&cn.credit_note_number),
            txt(cn.status.clone()),
            fmt_date(cn.date),
            fmt_decimal(cn.total_amount),
            fmt_opt_i64(cn.journal_entry_id),
            cn.version.to_string(),
            fmt_dt(cn.created_at),
            fmt_dt(cn.updated_at),
        ])
        .map_err(|e| map_csv_err("credit_notes", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("credit_notes", e))
}

pub fn serialize_credit_note_lines_csv<W: Write>(
    rows: &[CreditNoteLine],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "credit_note_id",
        "position",
        "description",
        "quantity",
        "unit_price",
        "vat_rate",
        "line_total",
        "revenue_account_id",
        "created_at",
    ])
    .map_err(|e| map_csv_err("credit_note_lines", e))?;
    for l in rows {
        csv.write_record([
            l.id.to_string(),
            l.credit_note_id.to_string(),
            l.position.to_string(),
            txt(l.description.clone()),
            fmt_decimal(l.quantity),
            fmt_decimal(l.unit_price),
            fmt_decimal(l.vat_rate),
            fmt_decimal(l.line_total),
            fmt_opt_i64(l.revenue_account_id),
            fmt_dt(l.created_at),
        ])
        .map_err(|e| map_csv_err("credit_note_lines", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("credit_note_lines", e))
}

pub fn serialize_supplier_invoices_csv<W: Write>(
    rows: &[SupplierInvoice],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "contact_id",
        "supplier_invoice_number",
        "status",
        "invoice_date",
        "due_date",
        "total_amount",
        "creditor_iban",
        "creditor_qr_iban",
        "payment_reference",
        "expected_payment_amount",
        "project_id",
        "purchase_journal_entry_id",
        "settlement_type",
        "settlement_bank_account_id",
        "settlement_account_id",
        "settlement_journal_entry_id",
        "paid_at",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("supplier_invoices", e))?;
    for si in rows {
        csv.write_record([
            si.id.to_string(),
            si.company_id.to_string(),
            si.contact_id.to_string(),
            fmt_opt_str(&si.supplier_invoice_number),
            txt(si.status.clone()),
            fmt_date(si.invoice_date),
            fmt_opt_date(si.due_date),
            fmt_decimal(si.total_amount),
            fmt_opt_str(&si.creditor_iban),
            fmt_opt_str(&si.creditor_qr_iban),
            fmt_opt_str(&si.payment_reference),
            fmt_opt_decimal(si.expected_payment_amount),
            fmt_opt_i64(si.project_id),
            si.purchase_journal_entry_id.to_string(),
            fmt_opt_str(&si.settlement_type),
            fmt_opt_i64(si.settlement_bank_account_id),
            fmt_opt_i64(si.settlement_account_id),
            fmt_opt_i64(si.settlement_journal_entry_id),
            fmt_opt_dt(si.paid_at),
            si.version.to_string(),
            fmt_dt(si.created_at),
            fmt_dt(si.updated_at),
        ])
        .map_err(|e| map_csv_err("supplier_invoices", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("supplier_invoices", e))
}

pub fn serialize_supplier_invoice_lines_csv<W: Write>(
    rows: &[SupplierInvoiceLine],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "supplier_invoice_id",
        "position",
        "description",
        "quantity",
        "unit_price",
        "vat_rate",
        "line_total",
        "expense_account_id",
        "created_at",
    ])
    .map_err(|e| map_csv_err("supplier_invoice_lines", e))?;
    for l in rows {
        csv.write_record([
            l.id.to_string(),
            l.supplier_invoice_id.to_string(),
            l.position.to_string(),
            txt(l.description.clone()),
            fmt_decimal(l.quantity),
            fmt_decimal(l.unit_price),
            fmt_decimal(l.vat_rate),
            fmt_decimal(l.line_total),
            l.expense_account_id.to_string(),
            fmt_dt(l.created_at),
        ])
        .map_err(|e| map_csv_err("supplier_invoice_lines", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("supplier_invoice_lines", e))
}

pub fn serialize_payment_batches_csv<W: Write>(
    rows: &[PaymentBatch],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "bank_account_id",
        "status",
        "requested_execution_date",
        "total_amount",
        "msg_id",
        "payment_info_id",
        "confirmed_at",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("payment_batches", e))?;
    for b in rows {
        csv.write_record([
            b.id.to_string(),
            b.company_id.to_string(),
            b.bank_account_id.to_string(),
            txt(b.status.clone()),
            fmt_date(b.requested_execution_date),
            fmt_decimal(b.total_amount),
            txt(b.msg_id.clone()),
            txt(b.payment_info_id.clone()),
            fmt_opt_dt(b.confirmed_at),
            b.version.to_string(),
            fmt_dt(b.created_at),
            fmt_dt(b.updated_at),
        ])
        .map_err(|e| map_csv_err("payment_batches", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("payment_batches", e))
}

pub fn serialize_payment_batch_items_csv<W: Write>(
    rows: &[PaymentBatchItem],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "payment_batch_id",
        "supplier_invoice_id",
        "position",
        "end_to_end_id",
        "amount",
        "created_at",
    ])
    .map_err(|e| map_csv_err("payment_batch_items", e))?;
    for it in rows {
        csv.write_record([
            it.id.to_string(),
            it.payment_batch_id.to_string(),
            it.supplier_invoice_id.to_string(),
            it.position.to_string(),
            txt(it.end_to_end_id.clone()),
            fmt_decimal(it.amount),
            fmt_dt(it.created_at),
        ])
        .map_err(|e| map_csv_err("payment_batch_items", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("payment_batch_items", e))
}

pub fn serialize_invoice_settlements_csv<W: Write>(
    rows: &[InvoiceSettlement],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "invoice_id",
        "journal_entry_id",
        "amount",
        "settled_on",
        "settlement_type",
        "settlement_bank_account_id",
        "settlement_account_id",
        "created_at",
    ])
    .map_err(|e| map_csv_err("invoice_settlements", e))?;
    for st in rows {
        csv.write_record([
            st.id.to_string(),
            st.company_id.to_string(),
            st.invoice_id.to_string(),
            st.journal_entry_id.to_string(),
            fmt_decimal(st.amount),
            fmt_date(st.settled_on),
            txt(st.settlement_type.clone()),
            fmt_opt_i64(st.settlement_bank_account_id),
            fmt_opt_i64(st.settlement_account_id),
            fmt_dt(st.created_at),
        ])
        .map_err(|e| map_csv_err("invoice_settlements", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("invoice_settlements", e))
}

/// ⚠️ **La table de correspondance des projets, sans laquelle
/// `journal_entry_lines.csv` porte des identifiants ORPHELINS** — l'export
/// exportait déjà le `project_id` des lignes, mais rien ne disait à quel projet
/// il correspondait (issue #386).
pub fn serialize_projects_csv<W: Write>(rows: &[Project], writer: W) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "parent_id",
        "code",
        "name",
        "description",
        "archived",
        "start_date",
        "end_date",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("projects", e))?;
    for p in rows {
        csv.write_record([
            p.id.to_string(),
            p.company_id.to_string(),
            fmt_opt_i64(p.parent_id),
            txt(p.code.clone()),
            txt(p.name.clone()),
            fmt_opt_str(&p.description),
            fmt_bool(p.archived),
            fmt_opt_date(p.start_date),
            fmt_opt_date(p.end_date),
            p.version.to_string(),
            fmt_dt(p.created_at),
            fmt_dt(p.updated_at),
        ])
        .map_err(|e| map_csv_err("projects", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("projects", e))
}

pub fn serialize_contact_persons_csv<W: Write>(
    rows: &[ContactPerson],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "contact_id",
        "first_name",
        "last_name",
        "role",
        "email",
        "phone",
        "active",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("contact_persons", e))?;
    for cp in rows {
        csv.write_record([
            cp.id.to_string(),
            cp.company_id.to_string(),
            cp.contact_id.to_string(),
            txt(cp.first_name.clone()),
            txt(cp.last_name.clone()),
            fmt_opt_str(&cp.role),
            fmt_opt_str(&cp.email),
            fmt_opt_str(&cp.phone),
            fmt_bool(cp.active),
            cp.version.to_string(),
            fmt_dt(cp.created_at),
            fmt_dt(cp.updated_at),
        ])
        .map_err(|e| map_csv_err("contact_persons", e))?;
    }
    csv.flush().map_err(|e| map_flush_err("contact_persons", e))
}

/// La **piste de contrôle** (issue #386, V.6).
///
/// ⛔ **Non bornée** — cf. `audit_log::list_all_by_company` et son doc-comment :
/// la fonction paginée et `list_for_export` rejettent ou tronquent, ce qui est
/// juste pour un écran et faux pour la souveraineté.
///
/// ⚠️ `details_json` est un instantané JSON **écrit par l'application**, mais il
/// contient des valeurs saisies par l'utilisateur (libellés, motifs, noms) :
/// il passe par `txt` comme toute autre cellule de texte.
pub fn serialize_audit_log_csv<W: Write>(
    rows: &[AuditLogEntry],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "user_id",
        "actor_label",
        "actor_type",
        "actor_api_key_id",
        "action",
        "entity_type",
        "entity_id",
        "details_json",
        "company_id",
        "created_at",
    ])
    .map_err(|e| map_csv_err("audit_log", e))?;
    for e in rows {
        csv.write_record([
            e.id.to_string(),
            e.user_id.to_string(),
            txt(e.actor_label.clone()),
            e.actor_type.as_str().to_string(),
            fmt_opt_i64(e.actor_api_key_id),
            txt(e.action.clone()),
            txt(e.entity_type.clone()),
            e.entity_id.to_string(),
            txt(e
                .details_json
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default()),
            fmt_opt_i64(e.company_id),
            fmt_dt(e.created_at),
        ])
        .map_err(|err| map_csv_err("audit_log", err))?;
    }
    csv.flush().map_err(|e| map_flush_err("audit_log", e))
}

/// Les pièces fournisseurs **importées**, tous statuts (issue #386).
///
/// ⚠️ `storage_path` désigne un **fichier qui n'est PAS dans cet export** : les
/// justificatifs ne sont ni dans l'export de souveraineté, ni dans la
/// sauvegarde. C'est un manque connu, qui relève d'une story séparée sur la
/// sauvegarde. *Exporter le chemin sans le fichier vaut mieux que de taire les
/// deux : il dit au moins qu'une pièce existait.*
pub fn serialize_imported_supplier_invoices_csv<W: Write>(
    rows: &[ImportedSupplierInvoice],
    writer: W,
) -> Result<(), AppError> {
    let mut w = writer;
    write_csv_bom(&mut w)?;
    let mut csv = make_csv_writer(w);
    csv.write_record([
        "id",
        "company_id",
        "status",
        "supplier_invoice_id",
        "file_hash",
        "storage_path",
        "original_filename",
        "mime_type",
        "byte_size",
        "creditor_iban",
        "is_qr_iban",
        "creditor_address_type",
        "creditor_name",
        "creditor_line1",
        "creditor_line2",
        "creditor_postal_code",
        "creditor_town",
        "creditor_country",
        "reference_type",
        "reference_value",
        "amount",
        "currency",
        "unstructured_message",
        "billing_information",
        "version",
        "created_at",
        "updated_at",
    ])
    .map_err(|e| map_csv_err("imported_supplier_invoices", e))?;
    for i in rows {
        csv.write_record([
            i.id.to_string(),
            i.company_id.to_string(),
            txt(i.status.clone()),
            fmt_opt_i64(i.supplier_invoice_id),
            txt(i.file_hash.clone()),
            txt(i.storage_path.clone()),
            txt(i.original_filename.clone()),
            txt(i.mime_type.clone()),
            i.byte_size.to_string(),
            txt(i.creditor_iban.clone()),
            fmt_bool(i.is_qr_iban),
            txt(i.creditor_address_type.clone()),
            txt(i.creditor_name.clone()),
            // Adresse « combinée » (type K) : toute l'adresse tient dans ces
            // deux lignes, NPA et localité restant vides (revue P1).
            fmt_opt_str(&i.creditor_line1),
            fmt_opt_str(&i.creditor_line2),
            fmt_opt_str(&i.creditor_postal_code),
            fmt_opt_str(&i.creditor_town),
            txt(i.creditor_country.clone()),
            txt(i.reference_type.clone()),
            fmt_opt_str(&i.reference_value),
            fmt_opt_decimal(i.amount),
            txt(i.currency.clone()),
            fmt_opt_str(&i.unstructured_message),
            fmt_opt_str(&i.billing_information),
            i.version.to_string(),
            fmt_dt(i.created_at),
            fmt_dt(i.updated_at),
        ])
        .map_err(|e| map_csv_err("imported_supplier_invoices", e))?;
    }
    csv.flush()
        .map_err(|e| map_flush_err("imported_supplier_invoices", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use kesh_db::entities::{AccountRole, AccountType, Journal as JournalEnum};
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
            role: Some(AccountRole::Receivable),
            postable: true,
            version: 1,
            created_at: naive_dt(2026, 1, 1, 0, 0, 0),
            updated_at: naive_dt(2026, 1, 2, 12, 30, 45),
        }
    }

    /// Société minimale pour les tests d'export. Les champs non exportés sont
    /// remplis de valeurs neutres — seul l'en-tête CSV fait foi.
    fn sample_company() -> kesh_db::entities::Company {
        kesh_db::entities::Company {
            id: 1,
            name: "Test SA".into(),
            first_name: None,
            last_name: None,
            address: "Rue du Test 1, 1000 Lausanne".into(),
            address_street: "Rue du Test".into(),
            address_building: "1".into(),
            address_postal_code: "1000".into(),
            address_city: "Lausanne".into(),
            address_country: "CH".into(),
            ide_number: None,
            org_type: kesh_db::entities::OrgType::Pme,
            accounting_language: kesh_db::entities::Language::Fr,
            instance_language: kesh_db::entities::Language::Fr,
            email: None,
            phone: None,
            website: None,
            is_stub: false,
            books_locked_through: None,
            version: 1,
            created_at: naive_dt(2026, 1, 1, 0, 0, 0),
            updated_at: naive_dt(2026, 1, 1, 0, 0, 0),
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
            reverses_entry_id: None,
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
            project_id: None,
        }
    }

    fn sample_invoice() -> Invoice {
        Invoice {
            id: 500,
            company_id: 42,
            contact_id: 7,
            invoice_number: Some("2026-0001".into()),
            status: "validated".into(),
            date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            payment_terms: Some("30 jours".into()),
            total_amount: dec!(1234.50),
            journal_entry_id: Some(10),
            paid_at: None,
            emailed_at: Some(naive_dt(2026, 6, 2, 9, 30, 0)),
            emailed_to: Some("debiteur@example.ch".into()),
            project_id: None,
            dunning_paused_at: Some(naive_dt(2026, 6, 15, 14, 0, 0)),
            dunning_paused_note: Some("litige; \"en cours\"".into()), // RFC 4180 escape
            version: 3,
            created_at: naive_dt(2026, 6, 1, 8, 0, 0),
            updated_at: naive_dt(2026, 6, 15, 14, 0, 0),
        }
    }

    // ----- #262 — invoices serializer : complétude des colonnes souveraineté -----

    /// Garde anti-dérive (#262) : le header DOIT lister exactement les 19 colonnes
    /// de la struct `Invoice`, dans l'ordre. Tout `ADD COLUMN` répercuté dans la
    /// struct force la mise à jour de l'export ET de ce test — symétrique de la
    /// discipline P5 (audit idempotence). Empêche qu'un champ (comme `emailed_*` /
    /// `dunning_paused_*`, jadis oubliés) manque silencieusement de l'export.
    #[test]
    fn serialize_invoices_csv_header_exhaustif() {
        let mut buf = Vec::new();
        serialize_invoices_csv(&[], &mut buf).expect("serialize ok");
        let text = String::from_utf8(buf[3..].to_vec()).expect("utf8"); // skip BOM
        let header = text.lines().next().expect("header");
        assert_eq!(
            header,
            "id;company_id;contact_id;invoice_number;status;date;due_date;\
             payment_terms;total_amount;journal_entry_id;paid_at;emailed_at;emailed_to;\
             project_id;dunning_paused_at;dunning_paused_note;version;created_at;updated_at"
        );
    }

    /// #262 : les champs envoi e-mail (Epic 20) et suspension de relance (21-5a),
    /// jadis omis de l'export souveraineté, sont désormais bien sérialisés.
    #[test]
    fn serialize_invoices_csv_inclut_emailed_et_dunning_paused() {
        let mut buf = Vec::new();
        serialize_invoices_csv(&[sample_invoice()], &mut buf).expect("serialize ok");
        let text = String::from_utf8(buf).expect("utf8");
        assert!(
            text.contains("2026-06-02T09:30:00Z"),
            "emailed_at manquant: {text}"
        );
        assert!(
            text.contains("debiteur@example.ch"),
            "emailed_to manquant: {text}"
        );
        assert!(
            text.contains("2026-06-15T14:00:00Z"),
            "dunning_paused_at manquant: {text}"
        );
        // Note avec `;` et `"` → double-quoted + `""` interne (RFC 4180).
        assert!(
            text.contains("\"litige; \"\"en cours\"\"\""),
            "dunning_paused_note mal échappée: {text}"
        );
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

    /// Story 14-3a — l'en-tête du CSV est écrit **à la main** : l'oubli d'une
    /// colonne y est silencieux (aucune erreur, juste une donnée qui disparaît
    /// de l'export global). Ce test fige donc l'en-tête complet.
    #[test]
    fn serialize_accounts_csv_includes_role_and_postable() {
        let mut buf = Vec::new();
        serialize_accounts_csv(&[sample_account()], &mut buf).expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        let mut lines = text.split("\r\n");

        assert_eq!(
            lines.next().unwrap(),
            "id;company_id;number;name;account_type;parent_id;active;role;postable;version;created_at;updated_at",
            "en-tête CSV des comptes (12 colonnes depuis la Story 14-3a)"
        );

        let row = lines.next().unwrap();
        assert!(
            row.contains(";Receivable;true;"),
            "le rôle et la postabilité doivent figurer dans la ligne : {row}"
        );
    }

    /// Un compte sans rôle exporte une cellule **vide**, pas la chaîne "null".
    #[test]
    fn serialize_accounts_csv_renders_absent_role_as_empty_cell() {
        let mut account = sample_account();
        account.role = None;
        account.postable = false;
        let mut buf = Vec::new();
        serialize_accounts_csv(&[account], &mut buf).expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        let row = text.split("\r\n").nth(1).unwrap();
        assert!(
            row.contains(";;false;"),
            "rôle absent → cellule vide, postable=false : {row}"
        );
        assert!(
            !row.contains("null"),
            "ne jamais écrire la chaîne 'null' : {row}"
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

    /// Story 24-4a (#380) — **le lien de contre-passation doit être DANS l'export.**
    ///
    /// ⛔ Ce test existe parce que rien d'autre ne le tiendrait :
    /// `backup_inventory_matches_schema` ne compare que la liste des **tables**,
    /// jamais les colonnes d'un CSV, et les deux assertions du test voisin sont
    /// des sous-chaînes locales qui resteraient vraies sur un CSV à dix colonnes.
    /// Un revert partiel passerait donc au vert — dans l'artefact même que le
    /// réviseur consulte, et dont l'absence de trace est ce que l'art. 958f CO
    /// interdit.
    /// Story 24-4c (#380) — la borne du verrou dans l'export de souveraineté.
    ///
    /// ⛔ `serialize_company_csv` énumère ses colonnes À LA MAIN, et aucun test
    /// n'impose leur exhaustivité vis-à-vis du schéma : `backup_inventory_matches_schema`
    /// ne compare que la liste des **tables**, jamais les colonnes d'un CSV. La
    /// 24-4a a payé ce piège sur `journal_entries` ; ce test est là pour ne pas
    /// le repayer sur `companies`.
    #[test]
    fn serialize_company_csv_carries_the_books_lock() {
        let en_tete = {
            let mut buf = Vec::new();
            serialize_company_csv(&[], &mut buf).expect("serialize ok");
            String::from_utf8(buf[3..].to_vec()).unwrap()
        };
        assert!(
            en_tete.contains("books_locked_through"),
            "la colonne manque à l'en-tête : {en_tete}"
        );
        // ⚠️ snake_case, comme les autres : c'est un export TABLE, pas du JSON.
        assert!(
            !en_tete.contains("booksLockedThrough"),
            "en-tête en camelCase — cet export n'est pas une réponse JSON : {en_tete}"
        );

        let mut sans_verrou = sample_company();
        sans_verrou.books_locked_through = None;
        let mut verrouillee = sample_company();
        verrouillee.id = 2;
        verrouillee.books_locked_through = NaiveDate::from_ymd_opt(2026, 3, 31);

        let mut buf = Vec::new();
        serialize_company_csv(&[sans_verrou, verrouillee], &mut buf).expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        let lignes: Vec<&str> = text.lines().collect();
        assert_eq!(lignes.len(), 3, "en-tête + deux sociétés : {text}");

        // ⛔ Assertions POSITIONNELLES, index tiré de l'en-tête — un
        // `contains(";;")` resterait vert si un autre champ devenait vide.
        let colonne = en_tete
            .trim_end()
            .split(';')
            .position(|c| c == "books_locked_through")
            .expect("la colonne doit être à l'en-tête");
        assert_eq!(
            lignes[1].split(';').nth(colonne),
            Some(""),
            "une société sans verrou laisse la colonne VIDE : {}",
            lignes[1]
        );
        assert_eq!(
            lignes[2].split(';').nth(colonne),
            Some("2026-03-31"),
            "la borne est exportée telle quelle, en ISO : {}",
            lignes[2]
        );
    }

    #[test]
    fn serialize_journal_entries_csv_carries_the_reversal_link() {
        let en_tete = {
            let mut buf = Vec::new();
            serialize_journal_entries_csv(&[], &mut buf).expect("serialize ok");
            String::from_utf8(buf[3..].to_vec()).unwrap()
        };
        assert!(
            en_tete.contains("reverses_entry_id"),
            "la colonne manque à l'en-tête : {en_tete}"
        );
        // ⚠️ snake_case, comme les dix autres : c'est un export TABLE.
        assert!(
            !en_tete.contains("reversesEntryId"),
            "en-tête en camelCase — cet export n'est pas une réponse JSON : {en_tete}"
        );

        let mut ordinaire = sample_entry();
        ordinaire.reverses_entry_id = None;
        let mut contre_passation = sample_entry();
        contre_passation.id = 11;
        contre_passation.reverses_entry_id = Some(10);

        let mut buf = Vec::new();
        serialize_journal_entries_csv(&[ordinaire, contre_passation], &mut buf)
            .expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        let lignes: Vec<&str> = text.lines().collect();
        assert_eq!(lignes.len(), 3, "en-tête + deux écritures : {text}");

        // ⛔ **Assertions POSITIONNELLES, et l'index vient de l'en-tête.** Un
        // `contains(";;")` ne discriminerait que par accident — il suffirait
        // qu'un autre champ du gabarit devienne vide pour qu'il reste vert sans
        // la colonne. *(Relevé en passe 2 de revue de code.)*
        let colonne = en_tete
            .trim_end()
            .split(';')
            .position(|c| c == "reverses_entry_id")
            .expect("la colonne doit être à l'en-tête");
        assert_eq!(
            lignes[1].split(';').nth(colonne),
            Some(""),
            "une écriture ordinaire laisse la colonne VIDE, jamais un 0 trompeur : {}",
            lignes[1]
        );
        assert_eq!(
            lignes[2].split(';').nth(colonne),
            Some("10"),
            "la contre-passation porte l'identifiant de son origine : {}",
            lignes[2]
        );
    }

    // ----- AC #30(c) — journal_entry_lines serializer -----

    #[test]
    fn serialize_journal_entry_lines_csv_decimal_two_decimals() {
        let mut buf = Vec::new();
        serialize_journal_entry_lines_csv(&[sample_line()], &mut buf).expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        // Decimal formaté 2 décimales — debit 150.25 + credit 0.00,
        // project_id vide (ligne non taguée, Story 19-2).
        assert!(text.contains(";150.25;0.00;\r\n"), "got: {text}");
        // Header
        assert!(text.starts_with("id;entry_id;account_id;line_order;debit;credit;project_id"));
    }

    #[test]
    fn serialize_journal_entry_lines_csv_tagged_line_exports_project_id() {
        let mut line = sample_line();
        line.project_id = Some(7);
        let mut buf = Vec::new();
        serialize_journal_entry_lines_csv(&[line], &mut buf).expect("serialize ok");
        let text = std::str::from_utf8(&buf[3..]).unwrap();
        assert!(text.contains(";150.25;0.00;7\r\n"), "got: {text}");
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

    // ----- Story 25-5-a, revue P1 -----

    /// `sent_to` et `note` d'un rappel sont du texte saisi : ils passent par
    /// `csv_sanitize` comme les autres cellules (ils lui échappaient).
    #[test]
    fn serialize_invoice_reminders_csv_neutralise_destinataire_et_note() {
        let dt = NaiveDate::from_ymd_opt(2026, 6, 1)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();
        let r = InvoiceReminder {
            id: 1,
            company_id: 1,
            invoice_id: 1,
            level_number: 1,
            fee_amount: Decimal::ZERO,
            sent_at: dt,
            channel: "manual".into(),
            sent_to: Some("=HYPERLINK(\"x\")".into()),
            subject: "Rappel".into(),
            body: "Corps".into(),
            note: Some("+cmd|' /C calc'!A0".into()),
            actor_user_id: None,
            cancelled_at: None,
            created_at: dt,
        };
        let mut buf = Vec::new();
        serialize_invoice_reminders_csv(&[r], &mut buf).expect("serialize ok");
        let text = String::from_utf8(buf[3..].to_vec()).expect("utf8");
        assert!(
            text.contains("'=HYPERLINK"),
            "`sent_to` doit être neutralisé : {text}"
        );
        assert!(
            text.contains("'+cmd"),
            "`note` doit être neutralisée : {text}"
        );
    }

    /// Une adresse « combinée » (type K) tient dans `creditor_line1` et
    /// `creditor_line2` : les deux colonnes sortent.
    #[test]
    fn serialize_imported_supplier_invoices_csv_porte_les_lignes_d_adresse() {
        let mut buf = Vec::new();
        serialize_imported_supplier_invoices_csv(&[], &mut buf).expect("serialize ok");
        let text = String::from_utf8(buf[3..].to_vec()).expect("utf8");
        let header = text.lines().next().expect("header");
        assert!(
            header.contains("creditor_name;creditor_line1;creditor_line2;creditor_postal_code"),
            "{header}"
        );
    }

    /// ⛔ **Garde d'exhaustivité des COLONNES** (Story 25-5-a, revue P2).
    ///
    /// La garde de `exports/global.rs` tient les **tables** ; rien ne tenait
    /// les **colonnes** — et sept des dix-neuf tables d'origine en omettaient,
    /// dont les noms et l'adresse structurée des contacts et de la société.
    /// Chaque en-tête est confronté au schéma de `test-schema/` (une source
    /// **indépendante** du sérialiseur, que `test_schema_guard.rs` tient
    /// lui-même égale au schéma réel) : toute colonne du schéma absente de
    /// l'en-tête fait rougir, sauf celles de [`COLONNES_HORS_EXPORT`], qui
    /// portent leur motif.
    #[test]
    fn chaque_colonne_du_schema_est_exportee_ou_ecartee() {
        const SCHEMA: &str = include_str!("../../../kesh-db/test-schema/0001_schema_squash.sql");
        fn entete(f: impl Fn(&mut Vec<u8>) -> Result<(), AppError>) -> Vec<String> {
            let mut buf = Vec::new();
            f(&mut buf).expect("serialize ok");
            let text = String::from_utf8(buf[3..].to_vec()).expect("utf8");
            text.lines()
                .next()
                .expect("header")
                .split(';')
                .map(str::to_string)
                .collect()
        }
        fn colonnes(table: &str) -> Vec<String> {
            let debut = SCHEMA
                .find(&format!("CREATE TABLE `{table}` ("))
                .unwrap_or_else(|| panic!("table `{table}` absente du schéma"));
            let bloc = &SCHEMA[debut..];
            let bloc = &bloc[..bloc.find(") ENGINE").expect("fin de table")];
            bloc.lines()
                .skip(1)
                .filter_map(|l| l.trim().strip_prefix('`'))
                .filter_map(|l| l.split('`').next())
                .map(str::to_string)
                .collect()
        }
        let tables: Vec<(&str, Vec<String>)> = vec![
            ("companies", entete(|w| serialize_company_csv(&[], w))),
            (
                "fiscal_years",
                entete(|w| serialize_fiscal_years_csv(&[], w)),
            ),
            ("accounts", entete(|w| serialize_accounts_csv(&[], w))),
            (
                "journal_entries",
                entete(|w| serialize_journal_entries_csv(&[], w)),
            ),
            (
                "journal_entry_lines",
                entete(|w| serialize_journal_entry_lines_csv(&[], w)),
            ),
            ("contacts", entete(|w| serialize_contacts_csv(&[], w))),
            ("products", entete(|w| serialize_products_csv(&[], w))),
            ("invoices", entete(|w| serialize_invoices_csv(&[], w))),
            (
                "invoice_lines",
                entete(|w| serialize_invoice_lines_csv(&[], w)),
            ),
            (
                "bank_accounts",
                entete(|w| serialize_bank_accounts_csv(&[], w)),
            ),
            (
                "bank_imports",
                entete(|w| serialize_bank_imports_csv(&[], w)),
            ),
            (
                "bank_transactions",
                entete(|w| serialize_bank_transactions_csv(&[], w)),
            ),
            ("vat_rates", entete(|w| serialize_vat_rates_csv(&[], w))),
            (
                "dunning_levels",
                entete(|w| serialize_dunning_levels_csv(&[], w)),
            ),
            (
                "company_dunning_settings",
                entete(|w| serialize_company_dunning_settings_csv(&[], w)),
            ),
            (
                "invoice_reminders",
                entete(|w| serialize_invoice_reminders_csv(&[], w)),
            ),
            (
                "company_invoice_settings",
                entete(|w| serialize_company_invoice_settings_csv(&[], w)),
            ),
            (
                "reconciliation_rules",
                entete(|w| serialize_reconciliation_rules_csv(&[], w)),
            ),
            (
                "bank_profiles",
                entete(|w| serialize_bank_profiles_csv(&[], w)),
            ),
            (
                "credit_notes",
                entete(|w| serialize_credit_notes_csv(&[], w)),
            ),
            (
                "credit_note_lines",
                entete(|w| serialize_credit_note_lines_csv(&[], w)),
            ),
            (
                "supplier_invoices",
                entete(|w| serialize_supplier_invoices_csv(&[], w)),
            ),
            (
                "supplier_invoice_lines",
                entete(|w| serialize_supplier_invoice_lines_csv(&[], w)),
            ),
            (
                "payment_batches",
                entete(|w| serialize_payment_batches_csv(&[], w)),
            ),
            (
                "payment_batch_items",
                entete(|w| serialize_payment_batch_items_csv(&[], w)),
            ),
            (
                "invoice_settlements",
                entete(|w| serialize_invoice_settlements_csv(&[], w)),
            ),
            ("projects", entete(|w| serialize_projects_csv(&[], w))),
            (
                "contact_persons",
                entete(|w| serialize_contact_persons_csv(&[], w)),
            ),
            ("audit_log", entete(|w| serialize_audit_log_csv(&[], w))),
            (
                "imported_supplier_invoices",
                entete(|w| serialize_imported_supplier_invoices_csv(&[], w)),
            ),
        ];
        // ⛔ Contre le REGISTRE, pas contre un nombre (revue P3) : une table
        // ajoutée demain à `TABLES_EXPORTEES` sans entrer ici échapperait,
        // sinon, à tout contrôle de ses colonnes.
        let ici: std::collections::BTreeSet<&str> = tables.iter().map(|(t, _)| *t).collect();
        let registre: std::collections::BTreeSet<&str> = crate::exports::global::TABLES_EXPORTEES
            .iter()
            .copied()
            .collect();
        assert_eq!(
            ici, registre,
            "la garde de colonnes doit couvrir exactement `TABLES_EXPORTEES`"
        );
        let mut manques = Vec::new();
        for (table, entete) in &tables {
            for col in colonnes(table) {
                let ecartee = COLONNES_HORS_EXPORT
                    .iter()
                    .any(|(t, c, _)| t == table && *c == col);
                // Le seul renommage : `bank_profiles.column_mapping` sort sous
                // `column_mapping_json`. Écrit en dur, pas en règle générale
                // (revue P3) : une règle couvrirait sans motif toute colonne future.
                let renommee = *table == "bank_profiles"
                    && col == "column_mapping"
                    && entete.iter().any(|h| h == "column_mapping_json");
                if !entete.contains(&col) && !ecartee && !renommee {
                    manques.push(format!("{table}.{col}"));
                }
            }
        }
        assert!(
            manques.is_empty(),
            "colonnes ni exportées ni écartées (ajouter au sérialiseur, ou à \
             COLONNES_HORS_EXPORT avec un motif) : {manques:?}"
        );
        for (t, c, motif) in COLONNES_HORS_EXPORT {
            assert!(!motif.trim().is_empty(), "{t}.{c} : motif vide");
            assert!(
                colonnes(t).iter().any(|x| x == c),
                "{t}.{c} : écartée mais absente du schéma — exemption périmée"
            );
            // Une colonne écartée qui entrerait dans l'en-tête garderait un
            // motif devenu faux (revue P3).
            let entete = &tables
                .iter()
                .find(|(n, _)| n == t)
                .expect("table écartée exportée")
                .1;
            assert!(
                !entete.iter().any(|h| h == c),
                "{t}.{c} : écartée mais exportée — exemption périmée"
            );
        }
    }

    /// Les colonnes du schéma **volontairement** absentes de l'export, avec
    /// leur motif. Chacune est une colonne **technique** : aucune ne porte une
    /// information que l'export ne dirait pas déjà ailleurs.
    const COLONNES_HORS_EXPORT: &[(&str, &str, &str)] = &[
        (
            "accounts",
            "singleton_role",
            "colonne GÉNÉRÉE (unicité d'un rôle actif) : recalculée de `role` et `active`, exportés",
        ),
        (
            "contacts",
            "client_number_canonical",
            "forme de comparaison de `client_number` (exporté), jamais affichée",
        ),
        (
            "contacts",
            "client_number_uniq",
            "colonne GÉNÉRÉE (unicité des numéros actifs) : dérivée de `active` et de `client_number_canonical`, elle-même forme de `client_number` (exporté)",
        ),
        (
            "reconciliation_rules",
            "active_uniq",
            "colonne GÉNÉRÉE (unicité des règles actives) : dérivée de `active` et `match_value`, exportés",
        ),
        (
            "companies",
            "country",
            "code pays de la Story 5.3 que rien n'écrit (il vaut toujours son défaut, `CH`) : le pays réel est `address_country`, exporté. ⚠️ La QR-facture le lit pourtant — défaut tracé par #466",
        ),
        (
            "contacts",
            "country",
            "code pays de la Story 5.3 que rien n'écrit (il vaut toujours son défaut, `CH`) : le pays réel est `address_country`, exporté. ⚠️ La QR-facture le lit pourtant — défaut tracé par #466",
        ),
    ];
}
