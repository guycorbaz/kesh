//! Tests d'intégration pour `repositories::reconciliation` (Story 8-4 T3).
//!
//! H2 Pass 1 code review : 5 tests `#[sqlx::test]` couvrant AC #37-#40
//! et AC #47 — fenêtre temporelle, multi-tenant scoping, status filter,
//! amount tolerance, et exclusion des transactions auto-rejetées.
//!
//! Pattern `#[sqlx::test(migrations = "./test-schema")]` — DB éphémère
//! avec migrations auto-appliquées. **Pré-requis exécution** : MariaDB
//! démarré localement (cf. CLAUDE.md « Test Locally First » — la CI
//! utilise `cargo test --workspace -j1 -- --test-threads=1` pour
//! sérialiser les tests d'intégration DB).
//!
//! On évite `invoices::create` + `validate_invoice` (lourd, nécessite
//! settings/fiscal_years/accounts) en faisant des INSERT raw — l'objet
//! sous test est le repository de réconciliation, pas le pipeline de
//! création/validation invoice.

use chrono::NaiveDate;
use kesh_db::entities::{
    BankImportSourceFormat, ContactType, NewBankAccount, NewBankImport, NewBankTransaction,
    NewContact, NewUser, OrgType, Role,
};
use kesh_db::repositories::reconciliation::InvoiceDateWindow;
use kesh_db::repositories::{
    bank_accounts, bank_imports, contacts, reconciliation as reconciliation_repo, users,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

async fn create_test_company(pool: &MySqlPool, name: &str) -> i64 {
    let result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind("Rue Test 1")
    .bind(OrgType::Pme.as_str())
    .bind("FR")
    .bind("FR")
    .execute(pool)
    .await
    .expect("company insert");
    result.last_insert_id() as i64
}

async fn create_test_user(pool: &MySqlPool, username: &str, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash:
                "$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo"
                    .into(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create")
    .id
}

async fn create_test_bank_account(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .expect("bank_account create")
    .id
}

async fn create_test_contact(pool: &MySqlPool, company_id: i64, user_id: i64, name: &str) -> i64 {
    contacts::create(
        pool,
        user_id,
        NewContact {
            company_id,
            contact_type: ContactType::Entreprise,
            name: name.into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .expect("contact create")
    .id
}

/// Insert minimal d'une fake `journal_entry` row (sans pipeline complet)
/// pour pouvoir poser `invoice.journal_entry_id IS NOT NULL`. La
/// logique du repo `find_unpaid_*` ne joint PAS journal_entries, donc
/// seul l'id retourné est utilisé.
async fn insert_fake_journal_entry(pool: &MySqlPool, company_id: i64, fy_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description, version, created_at, updated_at) \
         VALUES (?, ?, 1, '2026-05-01', 'Ventes', 'fake je', 1, NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(fy_id)
    .execute(pool)
    .await
    .expect("journal_entry insert");
    result.last_insert_id() as i64
}

async fn insert_fake_fiscal_year(pool: &MySqlPool, company_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
         created_at, updated_at) \
         VALUES (?, ?, '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(format!("FY 2026 c{company_id}"))
    .execute(pool)
    .await
    .expect("fiscal_year insert");
    result.last_insert_id() as i64
}

/// Insert une `invoice` directe via raw SQL pour bypass le pipeline
/// `validate_invoice` (settings/fiscal years/accounts requis sinon).
/// L'objet sous test est le repository réconciliation, pas la creation
/// invoice complète.
#[allow(clippy::too_many_arguments)]
async fn insert_test_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    date: NaiveDate,
    total_amount: Decimal,
    status: &str,
    journal_entry_id: Option<i64>,
    paid_at: Option<chrono::NaiveDateTime>,
) -> i64 {
    let result = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id, paid_at, version, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(format!(
        "INV-TEST-{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ))
    .bind(status)
    .bind(date)
    .bind(total_amount)
    .bind(journal_entry_id)
    .bind(paid_at)
    .execute(pool)
    .await
    .expect("invoice insert");
    let invoice_id = result.last_insert_id() as i64;
    // #246 (21-2b), #420 (25-4-c) : le matching filtre sur le reste dû,
    // dérivé des lignes (TTC) et des règlements. Une facture sans ligne
    // aurait un reste dû = 0 et ne matcherait jamais. On pose une ligne
    // unique à vat_rate 0, sans règlement → reste dû = TTC = total_amount,
    // les assertions de montant existantes restent inchangées.
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'Ligne test', 1, ?, 0, ?)",
    )
    .bind(invoice_id)
    .bind(total_amount)
    .bind(total_amount)
    .execute(pool)
    .await
    .expect("invoice_line insert");
    invoice_id
}

fn make_new_import(
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
) -> NewBankImport {
    NewBankImport {
        company_id,
        bank_account_id,
        filename: "stmt.xml".into(),
        file_hash: file_hash.into(),
        source_format: BankImportSourceFormat::Camt053V04,
        statement_id: Some("STMT-001".into()),
        period_from: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        period_to: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
        opening_balance: Some(dec!(1000.00)),
        closing_balance: Some(dec!(1100.00)),
        transaction_count: 1,
        imported_by_user_id: user_id,
    }
}

fn make_new_tx(
    company_id: i64,
    bank_account_id: i64,
    booking_date: NaiveDate,
    amount: Decimal,
    reference: &str,
) -> NewBankTransaction {
    NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date,
        value_date: Some(booking_date),
        amount,
        currency: "CHF".into(),
        reference: Some(reference.into()),
        details: "Test tx".into(),
        end_to_end_id: None,
        transaction_id: None,
        counterparty_iban: None,
        counterparty_name: Some("ACME GMBH".into()),
    }
}

const HASH_X: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// AC #38, réécrit par la Story 28-7 (#548) — la fenêtre de dates est celle
/// de l'acceptation, [`InvoiceDateWindow`] : une facture datée de `D − 30`
/// à `D + 1` est candidate, `D` étant la date de paiement ; une facture
/// datée de `D − 31` (paiement plus de 30 jours après sa date d'ancrage,
/// cf. ci-dessous) et une facture datée de `D + 2` (paiement plus d'un jour
/// avant la facture) ne le sont pas. Les deux bornes, de part et d'autre.
/// Les factures n'ont ni échéance ni rappel : leur date d'ancrage est leur
/// date (Story 29-2 ; l'ancre est éprouvée par
/// `the_anchor_is_the_latest_of_date_due_date_and_last_reminder`), et leur
/// date d'exigibilité aussi : l'ordre d'imputation de la Story 29-5 s'y réduit
/// à `(date, id)`.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_filters_by_30_day_window(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;

    let d = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let mut by_offset = std::collections::BTreeMap::new();
    for k in [-31i64, -30, 1, 2] {
        let id = insert_test_invoice(
            &pool,
            company_id,
            contact_id,
            d + chrono::Duration::days(k),
            dec!(100.00),
            "validated",
            Some(je_id),
            None,
        )
        .await;
        by_offset.insert(k, id);
    }

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(d),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let ids: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        ids,
        vec![by_offset[&-30], by_offset[&1]],
        "seules D − 30 et D + 1 sont dans la fenêtre, par date croissante"
    );
}

/// Story 28-7 (C-28-7-10) — le `LIMIT 50` de la proposition est déterministe :
/// les candidates sortent par `(date, id)` croissants, et ce sont les plus
/// récentes qui sont écartées au-delà de 50.
///
/// 52 factures de même reste dû, datées sur `[D − 30, D + 1]` (plusieurs par
/// jour), insérées dans l'ordre des dates **décroissantes** : `id ASC` et
/// `date ASC` divergent, si bien qu'un tri sur le seul `id` (ou un parcours en
/// ordre de clé primaire) ne rend pas les mêmes 50. Trois factures sont datées
/// `D + 1` : la coupure du `LIMIT 50` **traverse** ce groupe (une retenue, deux
/// écartées), et c'est le départage `id ASC` qui désigne la retenue.
///
/// Depuis la Story 29-5 (#668), la requête trie dans l'ordre d'imputation —
/// date d'exigibilité, puis `(date, id)` : ces factures n'ayant pas
/// d'échéance, leur date d'exigibilité est leur date, et l'ordre s'y réduit à
/// `(date, id)`. L'ordre avec échéances est éprouvé par
/// `find_unpaid_invoices_orders_by_exigibility_before_limit`.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_orders_by_date_then_id_before_limit(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

    let mut inserted: Vec<(NaiveDate, i64)> = Vec::new();
    for j in 0..52i64 {
        // Trois à `D + 1` (j = 0, 1, 2), puis de `D` (j = 3) à `D − 30`
        // (j = 51), en dates décroissantes.
        let back = if j < 3 { 0 } else { 1 + (j - 3) * 30 / 48 };
        let date = d + chrono::Duration::days(1 - back);
        let id = insert_test_invoice(
            &pool,
            company_id,
            contact_id,
            date,
            dec!(100.00),
            "validated",
            Some(je_id),
            None,
        )
        .await;
        inserted.push((date, id));
    }
    // Montage : l'insertion en dates décroissantes fait bien diverger les deux
    // ordres — sinon le test passerait à vide sur un tri par `id`.
    let mut by_id = inserted.clone();
    by_id.sort_by_key(|(_, id)| *id);
    let mut by_date_id = inserted.clone();
    by_date_id.sort();
    assert_ne!(by_id, by_date_id, "montage : `id ASC` ≠ `(date, id) ASC`");
    // Montage : la coupure tombe au milieu du groupe `D + 1` — la 50e est la
    // première de ce groupe, les 51e et 52e en sont aussi.
    let latest = d + chrono::Duration::days(1);
    assert_eq!(
        by_date_id.first().unwrap().0,
        d - chrono::Duration::days(30)
    );
    assert!(
        by_date_id[49..].iter().all(|(date, _)| *date == latest) && by_date_id[48].0 < latest,
        "montage : la coupure traverse le groupe de D + 1"
    );

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(d),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let got: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    let expected: Vec<i64> = by_date_id.iter().take(50).map(|(_, id)| *id).collect();
    assert_eq!(got.len(), 50, "le plafond de 50 candidates");
    assert_eq!(
        got, expected,
        "les 50 premières de (date, id) croissants, dans cet ordre"
    );
    for (_, id) in by_date_id.iter().skip(50) {
        assert!(!got.contains(id), "les deux plus récentes sont écartées");
    }
}

/// AC #38 + AC #59 — multi-tenant scoping (KF-002 Pattern 1) :
/// `find_unpaid_invoices_for_window(company_A)` ne doit pas retourner
/// les invoices de company_B même si elles sont dans la même fenêtre.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_filters_by_company_id(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "CompanyA").await;
    let company_b = create_test_company(&pool, "CompanyB").await;
    let user_a = create_test_user(&pool, "alice", company_a).await;
    let user_b = create_test_user(&pool, "bob", company_b).await;
    let contact_a = create_test_contact(&pool, company_a, user_a, "Client A").await;
    let contact_b = create_test_contact(&pool, company_b, user_b, "Client B").await;
    let fy_a = insert_fake_fiscal_year(&pool, company_a).await;
    let fy_b = insert_fake_fiscal_year(&pool, company_b).await;
    let je_a = insert_fake_journal_entry(&pool, company_a, fy_a).await;
    let je_b = insert_fake_journal_entry(&pool, company_b, fy_b).await;

    let day = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let inv_a = insert_test_invoice(
        &pool,
        company_a,
        contact_a,
        day,
        dec!(100.00),
        "validated",
        Some(je_a),
        None,
    )
    .await;
    let inv_b = insert_test_invoice(
        &pool,
        company_b,
        contact_b,
        day,
        dec!(100.00),
        "validated",
        Some(je_b),
        None,
    )
    .await;

    // Query as company_a → only inv_a expected.
    let candidates_a = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_a,
        InvoiceDateWindow::for_payment_date(day),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find for A");
    let ids_a: Vec<i64> = candidates_a.iter().map(|c| c.invoice.id).collect();
    assert!(ids_a.contains(&inv_a));
    assert!(!ids_a.contains(&inv_b), "leak cross-tenant détecté");

    // Symétrique : company_b → only inv_b.
    let candidates_b = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_b,
        InvoiceDateWindow::for_payment_date(day),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find for B");
    let ids_b: Vec<i64> = candidates_b.iter().map(|c| c.invoice.id).collect();
    assert!(ids_b.contains(&inv_b));
    assert!(!ids_b.contains(&inv_a), "leak cross-tenant détecté");
}

/// AC #37 — filtre `status='validated' AND paid_at IS NULL AND
/// journal_entry_id IS NOT NULL`. Une draft, une cancelled, une déjà
/// payée, ou une validée sans journal_entry ne sont PAS retournées.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_filters_status_validated_and_unpaid(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    // Draft → NOT candidate.
    let _inv_draft = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        day,
        dec!(100.00),
        "draft",
        None,
        None,
    )
    .await;

    // Validated mais déjà payée → NOT candidate.
    let now_dt = chrono::NaiveDate::from_ymd_opt(2026, 5, 5)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let _inv_paid = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        day,
        dec!(100.00),
        "validated",
        Some(je_id),
        Some(now_dt),
    )
    .await;

    // Note : le cas « validated mais journal_entry_id NULL » est
    // empêché par le CHECK constraint `chk_invoices_validated_has_je`
    // au niveau schéma — pas testable ici, c'est une garantie DB. Le
    // filtre `journal_entry_id IS NOT NULL` du repo est defense-in-depth
    // contre un futur relâchement du constraint (cf. M1 Pass 1 spec
    // validate, étape qui a explicité le filtre dans le pseudo-SQL).

    // Validated + unpaid + journal_entry_id ≠ NULL → CANDIDATE.
    let inv_ok = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        day,
        dec!(100.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(day),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let ids: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        ids,
        vec![inv_ok],
        "seule l'invoice eligible doit être retournée"
    );
}

/// AC #39 + AC #40 — tolérance du paiement partiel, 0.05 CHF. Une
/// invoice à 100.04 (delta 0.04) est candidate, mais 100.06 (delta 0.06)
/// ne l'est pas. N'éprouve que le côté du reste dû **supérieur** au
/// paiement ; l'autre côté (trop-perçu) est la borne d'un demi-centime,
/// `find_unpaid_invoices_excludes_an_invoice_the_payment_exceeds` (Story 29-6).
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_filters_amount_within_tolerance(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let inv_in = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        day,
        dec!(100.04),
        "validated",
        Some(je_id),
        None,
    )
    .await;
    let _inv_out = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        day,
        dec!(100.06),
        "validated",
        Some(je_id),
        None,
    )
    .await;

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(day),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let ids: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        ids,
        vec![inv_in],
        "seule l'invoice dont le reste dû dépasse le paiement d'au plus 0.05 doit être retournée"
    );
}

/// Story 29-6 (#665), AC5 — la proposition ne charge plus une facture que le
/// paiement **dépasse** : la borne basse du reste dû est
/// `least_due_without_overpayment(tx)` (= tx − 0.005), la plus petite valeur
/// que `classify_payment` n'estime pas trop-perçue ; la borne haute reste
/// `tx + tolérance` (paiement partiel). Attendu écrit en dur.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_excludes_an_invoice_the_payment_exceeds(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    let mut ids_by_due = Vec::new();
    for due in [
        dec!(99.95),
        dec!(99.99),
        dec!(99.9949),
        dec!(99.9950),
        dec!(99.9999),
        dec!(100.00),
        dec!(100.0049),
        dec!(100.05),
        dec!(100.06),
    ] {
        let id = insert_test_invoice(
            &pool,
            company_id,
            contact_id,
            day,
            due,
            "validated",
            Some(je_id),
            None,
        )
        .await;
        ids_by_due.push((due, id));
    }
    let id_of = |due: Decimal| {
        ids_by_due
            .iter()
            .find(|(d, _)| *d == due)
            .map(|(_, id)| *id)
            .unwrap()
    };

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(day),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let ids: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        ids,
        vec![
            id_of(dec!(99.9950)),
            id_of(dec!(99.9999)),
            id_of(dec!(100.00)),
            id_of(dec!(100.0049)),
            id_of(dec!(100.05)),
        ],
        "seuls les restes de 99.9950 à 100.05 sont proposés pour 100.00 : \
         un reste inférieur de plus d'un demi-centime est un trop-perçu (#665)"
    );
}

/// Story 29-6 (#665), AC5, second test — le filtre du trop-perçu est **dans la
/// requête**, avant `ORDER BY … LIMIT 50` (C-29-6-3, C-29-6-13) : cinquante
/// factures trop-perçues (99.99 pour un paiement de 100.00) puis une valide,
/// toutes à la même date et sans échéance, la valide ayant le plus grand `id`
/// (dernière place de l'ordre d'imputation). Un filtre posé après le
/// `fetch_all` laisserait les cinquante places du `LIMIT 50` aux trop-perçus,
/// et la valide ne serait jamais lue (mutation M8). Le nombre 50 est celui du
/// `LIMIT 50` de `find_unpaid_invoices_for_window` : s'il change, ce test suit.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_overpayments_take_no_place_in_the_limit(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

    for _ in 0..50 {
        insert_test_invoice(
            &pool,
            company_id,
            contact_id,
            day,
            dec!(99.99),
            "validated",
            Some(je_id),
            None,
        )
        .await;
    }
    let valid = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        day,
        dec!(100.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(day),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let ids: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        ids,
        vec![valid],
        "les trop-perçus ne doivent occuper aucune place du LIMIT 50"
    );
}

/// Une société, un client, un exercice, une écriture, et une facture validée
/// du 2026-05-01 par reste dû (dans l'ordre donné, donc d'`id` croissant).
/// Rend la date et les `id` des factures. Story 29-6.
async fn seed_invoices_by_due_296(
    pool: &MySqlPool,
    dues: &[Decimal],
) -> (i64, NaiveDate, Vec<i64>) {
    let company_id = create_test_company(pool, "Acme").await;
    let user_id = create_test_user(pool, "alice", company_id).await;
    let contact_id = create_test_contact(pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(pool, company_id).await;
    let je_id = insert_fake_journal_entry(pool, company_id, fy_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let mut ids = Vec::new();
    for due in dues {
        ids.push(
            insert_test_invoice(
                pool,
                company_id,
                contact_id,
                day,
                *due,
                "validated",
                Some(je_id),
                None,
            )
            .await,
        );
    }
    (company_id, day, ids)
}

/// Story 29-6 (#665), revue de code P1 (E2) — la borne basse au plus petit
/// paiement, un centime : `least_due_without_overpayment(0.01)` = 0.005. Un
/// reste de 0.0049 est un trop-perçu, un reste de 0.0050 solde à l'arrondi ;
/// la borne haute, 0.06, est admise, 0.07 non.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_bounds_hold_for_a_one_centime_payment(pool: MySqlPool) {
    let (company_id, day, ids) = seed_invoices_by_due_296(
        &pool,
        &[
            dec!(0.0049),
            dec!(0.0050),
            dec!(0.01),
            dec!(0.06),
            dec!(0.07),
        ],
    )
    .await;

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(day),
        dec!(0.01),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let got: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        got,
        vec![ids[1], ids[2], ids[3]],
        "pour 0.01 : restes de 0.0050, 0.01 et 0.06 ; 0.0049 est un trop-perçu, 0.07 hors tolérance"
    );
}

/// Story 29-6 (#665), revue de code P1 (E1) — les factures que la borne basse
/// écarte, à 5 centimes près, et elles seules : pour un paiement de 100.00,
/// les restes de 99.95 (inclus) à 99.9950 (exclu). Les deux listes sont
/// disjointes et se touchent à 99.9950. Attendu écrit en dur.
#[sqlx::test(migrations = "./test-schema")]
async fn find_invoices_slightly_overpaid_returns_what_the_lower_bound_excludes(pool: MySqlPool) {
    let (company_id, day, ids) = seed_invoices_by_due_296(
        &pool,
        &[
            dec!(99.94),
            dec!(99.9499),
            dec!(99.95),
            dec!(99.99),
            dec!(99.9949),
            dec!(99.9950),
            dec!(100.00),
        ],
    )
    .await;

    let overpaid = reconciliation_repo::find_invoices_slightly_overpaid_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(day),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_invoices_slightly_overpaid_for_window");

    let got: Vec<i64> = overpaid.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        got,
        vec![ids[2], ids[3], ids[4]],
        "pour 100.00 : restes de 99.95, 99.99 et 99.9949 ; 99.9950 est candidat, 99.9499 et 99.94 hors tolérance"
    );
}

/// AC #47 — `find_pending_transactions_for_account` exclut les
/// transactions avec `auto_match_rejected_at IS NOT NULL`. Une tx
/// rejetée manuellement n'est plus exposée via la liste pending.
#[sqlx::test(migrations = "./test-schema")]
async fn find_pending_transactions_excludes_auto_match_rejected(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();

    let mut tx = pool.begin().await.unwrap();
    let (_, inserted_txs) = bank_imports::create_with_transactions(
        &mut tx,
        make_new_import(company_id, bank_id, user_id, HASH_X),
        vec![
            make_new_tx(company_id, bank_id, day, dec!(100.00), "REF-1"),
            make_new_tx(company_id, bank_id, day, dec!(50.00), "REF-2"),
        ],
    )
    .await
    .expect("create_with_transactions");
    tx.commit().await.unwrap();

    // Mark first tx as auto-rejected.
    let rejected_tx_id = inserted_txs[0].id;
    let kept_tx_id = inserted_txs[1].id;
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(rejected_tx_id)
        .execute(&pool)
        .await
        .expect("update auto_match_rejected_at");

    let pending =
        reconciliation_repo::find_pending_transactions_for_account(&pool, company_id, bank_id, 100)
            .await
            .expect("find_pending_transactions_for_account");

    let ids: Vec<i64> = pending.iter().map(|t| t.id).collect();
    assert!(
        ids.contains(&kept_tx_id),
        "tx pending non-rejetée doit être retournée"
    );
    assert!(
        !ids.contains(&rejected_tx_id),
        "tx auto_match_rejected ne doit PAS être retournée (AC #47)"
    );
}

// ---------------------------------------------------------------------------
// Story 8-5a-base T1.2 — find_strictly_pending_by_id_for_account
// ---------------------------------------------------------------------------

/// Story 8-5a-base T1.2.1 — happy path : returns Some(tx) si
/// company_id/bank_account_id/id matchent ET status='pending'.
#[sqlx::test(migrations = "./test-schema")]
async fn find_strictly_pending_returns_tx_when_all_conditions_match(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let tx_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let mut import_tx = pool.begin().await.unwrap();
    let mut nbi = make_new_import(company_id, bank_id, user_id, HASH_X);
    nbi.transaction_count = 1;
    let (_, txs) = bank_imports::create_with_transactions(
        &mut import_tx,
        nbi,
        vec![make_new_tx(
            company_id,
            bank_id,
            tx_date,
            dec!(150.00),
            "REF-1",
        )],
    )
    .await
    .expect("seed tx");
    import_tx.commit().await.unwrap();
    let tx_id = txs[0].id;

    let result = reconciliation_repo::find_strictly_pending_by_id_for_account(
        &pool, company_id, bank_id, tx_id,
    )
    .await
    .expect("find_strictly_pending_by_id_for_account");

    let bt = result.expect("happy path → Some(tx)");
    assert_eq!(bt.id, tx_id);
    assert_eq!(bt.company_id, company_id);
    assert_eq!(bt.bank_account_id, bank_id);
}

/// Story 8-5a-base T1.2.2 — sécurité multi-tenant + scope account :
/// cross-tenant return None (KF-002 Pattern 1, jamais de leak).
#[sqlx::test(migrations = "./test-schema")]
async fn find_strictly_pending_returns_none_for_cross_tenant(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "AcmeA").await;
    let user_a = create_test_user(&pool, "alice", company_a).await;
    let bank_a = create_test_bank_account(&pool, company_a, "CH4431999123000889012").await;

    let company_b = create_test_company(&pool, "AcmeB").await;
    let user_b = create_test_user(&pool, "bob", company_b).await;
    let bank_b = create_test_bank_account(&pool, company_b, "CH9300762011623852957").await;

    let tx_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    // Insert une tx du côté company_b.
    let mut import_tx = pool.begin().await.unwrap();
    let mut nbi = make_new_import(company_b, bank_b, user_b, HASH_X);
    nbi.transaction_count = 1;
    let (_, txs) = bank_imports::create_with_transactions(
        &mut import_tx,
        nbi,
        vec![make_new_tx(
            company_b,
            bank_b,
            tx_date,
            dec!(150.00),
            "REF-B",
        )],
    )
    .await
    .expect("seed tx B");
    import_tx.commit().await.unwrap();
    let tx_id_b = txs[0].id;

    // user_a tente de lire la tx de company_b → doit être None.
    let cross_tenant = reconciliation_repo::find_strictly_pending_by_id_for_account(
        &pool, company_a, bank_a, tx_id_b,
    )
    .await
    .expect("find_strictly_pending_by_id_for_account cross-tenant");
    assert!(
        cross_tenant.is_none(),
        "cross-tenant doit retourner None (anti-leak KF-002)"
    );

    // user_a avec son own bank_id mais id de tx_b → None aussi.
    let cross_account = reconciliation_repo::find_strictly_pending_by_id_for_account(
        &pool, company_b, bank_a, tx_id_b,
    )
    .await
    .expect("find_strictly_pending cross-account");
    assert!(
        cross_account.is_none(),
        "cross-account (mauvais bank_id) doit retourner None"
    );
    // Sanity : avec le bon couple company+bank → Some.
    assert!(
        reconciliation_repo::find_strictly_pending_by_id_for_account(
            &pool, company_b, bank_b, tx_id_b,
        )
        .await
        .expect("happy")
        .is_some(),
        "lookup légitime doit retourner Some(tx)"
    );
    // suppress unused warning
    let _ = user_a;
}

/// Story 8-5a-base T1.2.3 — filtre status précis : tx déjà
/// `reconciled` retourne None (le helper distinct de
/// `find_pending_by_id_for_account` 8-4).
#[sqlx::test(migrations = "./test-schema")]
async fn find_strictly_pending_returns_none_for_reconciled_status(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH4431999123000889012").await;

    let tx_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let mut import_tx = pool.begin().await.unwrap();
    let mut nbi = make_new_import(company_id, bank_id, user_id, HASH_X);
    nbi.transaction_count = 1;
    let (_, txs) = bank_imports::create_with_transactions(
        &mut import_tx,
        nbi,
        vec![make_new_tx(
            company_id,
            bank_id,
            tx_date,
            dec!(150.00),
            "REF-RECON",
        )],
    )
    .await
    .expect("seed tx");
    import_tx.commit().await.unwrap();
    let tx_id = txs[0].id;

    // Marque la tx comme `reconciled` directement DB-side.
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE id = ?")
        .bind(tx_id)
        .execute(&pool)
        .await
        .expect("mark reconciled");

    let result = reconciliation_repo::find_strictly_pending_by_id_for_account(
        &pool, company_id, bank_id, tx_id,
    )
    .await
    .expect("find_strictly_pending_by_id_for_account");

    assert!(
        result.is_none(),
        "tx avec status='reconciled' doit retourner None (filtre status précis vs find_pending_by_id_for_account 8-4)"
    );

    // Sanity : `find_pending_by_id_for_account` 8-4, lui, retourne quand
    // même la tx (ne filtre pas status — F8'' Pass 3 spec note).
    let still_findable_by_legacy =
        reconciliation_repo::find_pending_by_id_for_account(&pool, company_id, bank_id, tx_id)
            .await
            .expect("find_pending_by_id_for_account");
    assert!(
        still_findable_by_legacy.is_some(),
        "find_pending_by_id_for_account 8-4 ne filtre PAS status, retourne la tx reconciled — démarcation explicite vs strictly_pending"
    );
}

/// #246 (Story 21-2b) — le matching filtre sur le **TTC**, pas le HT.
/// Régression corrigée : une facture avec TVA (HT 100 @ 8.1 % → TTC 108.10)
/// doit matcher un encaissement de 108.10 (TTC réel) et NON de 100.00 (HT).
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_matches_on_ttc_not_ht(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "TTC Co").await;
    let user_id = create_test_user(&pool, "bob", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client TTC").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;

    let tx_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();

    // Facture validée HT 100.00 @ 8.1 % → TTC 108.10.
    let result = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id, version, created_at, updated_at) \
         VALUES (?, ?, 'INV-VAT-1', 'validated', ?, ?, ?, 1, NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(inv_date)
    .bind(dec!(100.00))
    .bind(je_id)
    .execute(&pool)
    .await
    .expect("invoice insert");
    let inv_id = result.last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'Prestation', 1, 100.00, 8.10, 100.00)",
    )
    .bind(inv_id)
    .execute(&pool)
    .await
    .expect("line insert");

    // tx = 108.10 (TTC) → candidate trouvée, total_ttc exposé = 108.10.
    let found_ttc = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(tx_date),
        dec!(108.10),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid TTC");
    let cand = found_ttc.iter().find(|c| c.invoice.id == inv_id);
    assert!(
        cand.is_some(),
        "la facture doit matcher l'encaissement TTC 108.10"
    );
    assert_eq!(cand.unwrap().total_ttc, dec!(108.10));
    // #420 (Story 25-4-c) : sans règlement, le reste dû — la grandeur filtrée
    // et comparée — vaut exactement le TTC.
    assert_eq!(cand.unwrap().amount_due, dec!(108.10));

    // tx = 100.00 (HT) → PAS de match (le bug d'avant matchait le HT).
    let found_ht = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(tx_date),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid HT");
    assert!(
        !found_ht.iter().any(|c| c.invoice.id == inv_id),
        "le HT 100.00 ne doit plus matcher (régression #246 corrigée)"
    );
}

/// #420 (Story 25-4-c) — le filtre porte sur le **reste dû**, plus sur le TTC.
/// Régression corrigée : une facture de 1 000.— (TVA 8.1 %) déjà réglée de
/// 400.— était **exclue** des candidates d'un virement de 600.— — le solde
/// exact qu'attendait le client —, parce que le filtre comparait le TTC.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_matches_on_amount_due_of_partially_settled_invoice(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Reste Co").await;
    let user_id = create_test_user(&pool, "carla", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client reste").await;
    let bank_id = create_test_bank_account(&pool, company_id, "CH9300762011623852957").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let sale_je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;

    let tx_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let inv_date = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();

    // HT 925.07 @ 8.1 % → TVA 74.93 → TTC 1 000.00.
    let inv_id = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id, version, created_at, updated_at) \
         VALUES (?, ?, 'INV-PART-1', 'validated', ?, ?, ?, 1, NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(inv_date)
    .bind(dec!(925.07))
    .bind(sale_je_id)
    .execute(&pool)
    .await
    .expect("invoice insert")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'Prestation', 1, 925.07, 8.10, 925.07)",
    )
    .bind(inv_id)
    .execute(&pool)
    .await
    .expect("line insert");

    // Règlement partiel de 400.— : il lui faut sa propre écriture
    // (`uq_invoice_settlements_entry`).
    let settle_je_id = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description, version, created_at, updated_at) \
         VALUES (?, ?, 2, '2026-05-05', 'Banque', 'règlement partiel', 1, NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(fy_id)
    .execute(&pool)
    .await
    .expect("settlement journal_entry insert")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
         settled_on, settlement_type, settlement_bank_account_id) \
         VALUES (?, ?, ?, 400.00, '2026-05-05', 'bank_transfer', ?)",
    )
    .bind(company_id)
    .bind(inv_id)
    .bind(settle_je_id)
    .bind(bank_id)
    .execute(&pool)
    .await
    .expect("settlement insert");

    // Virement du solde, 600.— → la facture est candidate, avec son reste
    // ET son TTC (la mention « reste dû sur 1 000.00 » de l'écran).
    let found_due = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(tx_date),
        dec!(600.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid reste dû");
    let cand = found_due
        .iter()
        .find(|c| c.invoice.id == inv_id)
        .expect("la facture réglée en partie doit être candidate pour son reste de 600.00");
    assert_eq!(cand.amount_due, dec!(600.00));
    assert_eq!(cand.total_ttc, dec!(1000.00));

    // Virement du TTC, 1 000.— → plus candidate : il dépasserait le reste.
    let found_ttc = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(tx_date),
        dec!(1000.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid TTC");
    assert!(
        !found_ttc.iter().any(|c| c.invoice.id == inv_id),
        "le TTC 1 000.00 ne doit plus matcher une facture dont il reste 600.00"
    );
}

// ---------------------------------------------------------------------------
// Story 27-4 (#634) — lectures sans filtre de statut des lots de réconciliation
// ---------------------------------------------------------------------------

/// Montage commun aux tests de la Story 27-4 : société A avec deux comptes
/// (`bank_a`, `bank_a2`), société B avec un compte ; sur `bank_a`, trois
/// transactions — `pending`, `reconciled` (sans lien, posé en base) et
/// rejetée ; une transaction sur `bank_a2` et une sur la société B.
struct Seed274 {
    company_a: i64,
    bank_a: i64,
    pending: i64,
    reconciled: i64,
    rejected: i64,
    other_account: i64,
    other_company: i64,
}

async fn seed_one_import(
    pool: &MySqlPool,
    company_id: i64,
    bank_id: i64,
    user_id: i64,
    hash: &str,
    refs: &[&str],
) -> Vec<i64> {
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let mut nbi = make_new_import(company_id, bank_id, user_id, hash);
    nbi.transaction_count = refs.len() as i32;
    let mut tx = pool.begin().await.unwrap();
    let (_, txs) = bank_imports::create_with_transactions(
        &mut tx,
        nbi,
        refs.iter()
            .map(|r| make_new_tx(company_id, bank_id, day, dec!(10.00), r))
            .collect(),
    )
    .await
    .expect("create_with_transactions");
    tx.commit().await.unwrap();
    txs.iter().map(|t| t.id).collect()
}

async fn seed_27_4(pool: &MySqlPool) -> Seed274 {
    let company_a = create_test_company(pool, "Acme274A").await;
    let user_a = create_test_user(pool, "alice274", company_a).await;
    let bank_a = create_test_bank_account(pool, company_a, "CH4431999123000889012").await;
    let bank_a2 = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: company_a,
            bank_name: "PostFinance".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .expect("second bank_account")
    .id;
    let company_b = create_test_company(pool, "Acme274B").await;
    let user_b = create_test_user(pool, "bob274", company_b).await;
    let bank_b = create_test_bank_account(pool, company_b, "CH4431999123000889012").await;

    let ids = seed_one_import(pool, company_a, bank_a, user_a, HASH_X, &["P", "R", "J"]).await;
    let (pending, reconciled, rejected) = (ids[0], ids[1], ids[2]);
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE id = ?")
        .bind(reconciled)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(rejected)
        .execute(pool)
        .await
        .unwrap();
    let other_account = seed_one_import(
        pool,
        company_a,
        bank_a2,
        user_a,
        "1111111111111111111111111111111111111111111111111111111111111111",
        &["A2"],
    )
    .await[0];
    let other_company = seed_one_import(
        pool,
        company_b,
        bank_b,
        user_b,
        "2222222222222222222222222222222222222222222222222222222222222222",
        &["B"],
    )
    .await[0];
    Seed274 {
        company_a,
        bank_a,
        pending,
        reconciled,
        rejected,
        other_account,
        other_company,
    }
}

/// Story 27-4, AC11 — `find_by_ids_for_account` rend les lignes du compte
/// **quel que soit leur statut** (pending, reconciled, rejetée), aucune d'un
/// autre compte ni d'une autre société ; liste vide → vide ; 501 → erreur.
#[sqlx::test(migrations = "./test-schema")]
async fn find_by_ids_for_account_returns_every_status_and_nothing_out_of_scope(pool: MySqlPool) {
    let s = seed_27_4(&pool).await;

    let found = reconciliation_repo::find_by_ids_for_account(
        &pool,
        s.company_a,
        s.bank_a,
        &[
            s.pending,
            s.reconciled,
            s.rejected,
            s.other_account,
            s.other_company,
            i64::MAX,
        ],
    )
    .await
    .expect("find_by_ids_for_account");
    let mut keys: Vec<i64> = found.keys().copied().collect();
    keys.sort_unstable();
    let mut expected = vec![s.pending, s.reconciled, s.rejected];
    expected.sort_unstable();
    assert_eq!(
        keys, expected,
        "les trois lignes du compte, et elles seules"
    );
    assert_eq!(found[&s.reconciled].status.as_str(), "reconciled");
    assert!(found[&s.rejected].auto_match_rejected_at.is_some());

    let empty = reconciliation_repo::find_by_ids_for_account(&pool, s.company_a, s.bank_a, &[])
        .await
        .expect("liste vide");
    assert!(empty.is_empty());

    let ids_501: Vec<i64> = (1..=501).collect();
    let too_many =
        reconciliation_repo::find_by_ids_for_account(&pool, s.company_a, s.bank_a, &ids_501).await;
    assert!(too_many.is_err(), "501 identifiants doivent être refusés");
    let ids_500: Vec<i64> = (1..=500).collect();
    assert!(
        reconciliation_repo::find_by_ids_for_account(&pool, s.company_a, s.bank_a, &ids_500)
            .await
            .is_ok(),
        "500 identifiants passent"
    );
    assert_eq!(reconciliation_repo::MAX_IN_CLAUSE_IDS, 500);
}

/// Story 27-4, AC11 — `find_by_id_for_account_for_update`, dans une
/// transaction : rend la ligne pending, reconciled, rejetée ; `None` pour un
/// identifiant absent, d'un autre compte, d'une autre société. Le caractère
/// **courant** de la lecture est prouvé côté route (AC6-8).
#[sqlx::test(migrations = "./test-schema")]
async fn find_by_id_for_account_for_update_scopes_without_status_filter(pool: MySqlPool) {
    let s = seed_27_4(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    for id in [s.pending, s.reconciled, s.rejected] {
        let row = reconciliation_repo::find_by_id_for_account_for_update(
            &mut *tx,
            s.company_a,
            s.bank_a,
            id,
        )
        .await
        .expect("for update");
        assert_eq!(row.map(|r| r.id), Some(id), "ligne {id} du compte attendue");
    }
    for id in [s.other_account, s.other_company, i64::MAX] {
        let row = reconciliation_repo::find_by_id_for_account_for_update(
            &mut *tx,
            s.company_a,
            s.bank_a,
            id,
        )
        .await
        .expect("for update");
        assert!(row.is_none(), "{id} hors du compte : None attendu");
    }
    tx.rollback().await.unwrap();
}

// ---------------------------------------------------------------------------
// Story 29-2 (#529) — la date d'ancrage : échéance et dernier rappel
// ---------------------------------------------------------------------------

/// Pose l'échéance d'une facture insérée par [`insert_test_invoice`] (qui la
/// laisse `NULL`, comme tous ses appels antérieurs à la Story 29-2).
async fn set_test_due_date(pool: &MySqlPool, invoice_id: i64, due_date: Option<NaiveDate>) {
    sqlx::query("UPDATE invoices SET due_date = ? WHERE id = ?")
        .bind(due_date)
        .bind(invoice_id)
        .execute(pool)
        .await
        .expect("due_date update");
}

/// Insère un rappel envoyé le jour `sent_on` (à midi), annulé ou non. Les
/// colonnes `NOT NULL` reçoivent des valeurs synthétiques admises par les
/// contraintes de `invoice_reminders` (niveau ≥ 1, canal `manual`).
async fn insert_test_reminder(
    pool: &MySqlPool,
    company_id: i64,
    invoice_id: i64,
    sent_on: NaiveDate,
    cancelled: bool,
) {
    let sent_at = sent_on.and_hms_opt(12, 30, 0).unwrap();
    sqlx::query(
        "INSERT INTO invoice_reminders (company_id, invoice_id, level_number, fee_amount, \
         sent_at, channel, subject, body, cancelled_at) \
         VALUES (?, ?, 1, 0, ?, 'manual', 'Rappel', 'Rappel de test', ?)",
    )
    .bind(company_id)
    .bind(invoice_id)
    .bind(sent_at)
    .bind(cancelled.then_some(sent_at))
    .execute(pool)
    .await
    .expect("invoice_reminder insert");
}

/// Un cas de la table d'AC5 : décalages par rapport à `D`.
struct AnchorCase {
    name: &'static str,
    invoice: i64,
    due: Option<i64>,
    /// `(décalage, annulé)`, dans l'ordre d'envoi.
    reminders: &'static [(i64, bool)],
    anchor: i64,
    candidate: bool,
}

/// Story 29-2 (#529), AC5 — la borne basse s'ancre sur la plus tardive de la
/// date de facture, de l'échéance et du dernier rappel non annulé ; la borne
/// haute reste sur la date de facture (#548).
///
/// Pour chaque cas, montant distinct (aucune facture candidate d'un autre
/// cas) : la liste rendue par la proposition est assertée comme un `Vec`
/// **ordonné, répétitions comprises** (une table dérivée qui rendrait une
/// ligne par rappel dédoublerait f — M15), et l'ancre scalaire de
/// l'acceptation est assertée pour **chaque** cas, candidat ou non : c'est la
/// seule assertion qui voie l'ancre de g, refusée dans les deux formes (une
/// ancre qui suivrait l'échéance antérieure à la date — mutation M14bis).
/// Le cas l (échéance et rappel antérieurs à la date) tue M14 par l'ancre
/// scalaire, assertée la première ; la liste la verrait aussi (ancre mutée
/// `D − 50`, hors fenêtre), ce qui n'a pas été rejoué.
#[sqlx::test(migrations = "./test-schema")]
async fn the_anchor_is_the_latest_of_date_due_date_and_last_reminder(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;

    let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
    let at = |k: i64| d + chrono::Duration::days(k);
    let cases = [
        AnchorCase {
            name: "a",
            invoice: -45,
            due: None,
            reminders: &[],
            anchor: -45,
            candidate: false,
        },
        AnchorCase {
            name: "b",
            invoice: -45,
            due: Some(-30),
            reminders: &[],
            anchor: -30,
            candidate: true,
        },
        AnchorCase {
            name: "c",
            invoice: -45,
            due: Some(-31),
            reminders: &[],
            anchor: -31,
            candidate: false,
        },
        AnchorCase {
            name: "d",
            invoice: -90,
            due: Some(-60),
            reminders: &[(-10, false)],
            anchor: -10,
            candidate: true,
        },
        AnchorCase {
            name: "e",
            invoice: -90,
            due: Some(-60),
            reminders: &[(-10, true)],
            anchor: -60,
            candidate: false,
        },
        AnchorCase {
            name: "f",
            invoice: -90,
            due: Some(-60),
            reminders: &[(-50, false), (-20, false)],
            anchor: -20,
            candidate: true,
        },
        AnchorCase {
            name: "g",
            invoice: -45,
            due: Some(-50),
            reminders: &[],
            anchor: -45,
            candidate: false,
        },
        AnchorCase {
            name: "h",
            invoice: 2,
            due: Some(32),
            reminders: &[],
            anchor: 32,
            candidate: false,
        },
        AnchorCase {
            name: "i",
            invoice: -20,
            due: None,
            reminders: &[],
            anchor: -20,
            candidate: true,
        },
        AnchorCase {
            name: "j",
            invoice: -10,
            due: Some(20),
            reminders: &[],
            anchor: 20,
            candidate: true,
        },
        AnchorCase {
            name: "k",
            invoice: -45,
            due: Some(15),
            reminders: &[],
            anchor: 15,
            candidate: true,
        },
        // Revue de code P1 (E1, A2) : échéance **et** rappel non annulé tous
        // deux antérieurs à la date de la facture — l'ancre est la date. Seul
        // cas où le `i.date` du premier terme de l'ancre est seul à porter
        // `F` : sans lui (mutation M14), l'ancre tomberait à `D − 50`, hors
        // fenêtre. Jeu hors API (une échéance antérieure à la date est refusée
        // par `routes/invoices.rs`), posé par fixture SQL comme g.
        AnchorCase {
            name: "l",
            invoice: -25,
            due: Some(-50),
            reminders: &[(-60, false)],
            anchor: -25,
            candidate: true,
        },
    ];

    let window = InvoiceDateWindow::for_payment_date(d);
    for (n, case) in cases.iter().enumerate() {
        // 101.00, 102.00, … : écarts de 1.00, bien au-delà de la tolérance.
        let amount = Decimal::new(10_100 + 100 * n as i64, 2);
        let id = insert_test_invoice(
            &pool,
            company_id,
            contact_id,
            at(case.invoice),
            amount,
            "validated",
            Some(je_id),
            None,
        )
        .await;
        set_test_due_date(&pool, id, case.due.map(at)).await;
        for &(k, cancelled) in case.reminders {
            insert_test_reminder(&pool, company_id, id, at(k), cancelled).await;
        }

        let anchor = reconciliation_repo::invoice_payment_anchor_date(&pool, company_id, id)
            .await
            .expect("invoice_payment_anchor_date");
        assert_eq!(anchor, Some(at(case.anchor)), "cas {} : ancre", case.name);

        let listed: Vec<i64> = reconciliation_repo::find_unpaid_invoices_for_window(
            &pool,
            company_id,
            window,
            amount,
            dec!(0.05),
        )
        .await
        .expect("find_unpaid_invoices_for_window")
        .iter()
        .map(|c| c.invoice.id)
        .collect();
        let expected = if case.candidate { vec![id] } else { vec![] };
        assert_eq!(listed, expected, "cas {} : candidates", case.name);
    }
}

/// Story 29-2, AC5 — le rappel d'une facture X ne déplace pas l'ancre d'une
/// facture Y : la table dérivée des rappels se joint par `invoice_id`
/// (mutation M9, jointure croisée).
#[sqlx::test(migrations = "./test-schema")]
async fn a_reminder_moves_only_the_anchor_of_its_own_invoice(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
    let at = |k: i64| d + chrono::Duration::days(k);

    let x = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        at(-90),
        dec!(300.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;
    insert_test_reminder(&pool, company_id, x, at(-10), false).await;
    let y = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        at(-45),
        dec!(400.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;

    assert_eq!(
        reconciliation_repo::invoice_payment_anchor_date(&pool, company_id, y)
            .await
            .unwrap(),
        Some(at(-45)),
        "Y garde sa propre date"
    );
    let listed: Vec<i64> = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(d),
        dec!(400.00),
        dec!(0.05),
    )
    .await
    .unwrap()
    .iter()
    .map(|c| c.invoice.id)
    .collect();
    assert_eq!(listed, Vec::<i64>::new(), "Y n'est pas candidate");
    let listed_x: Vec<i64> = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(d),
        dec!(300.00),
        dec!(0.05),
    )
    .await
    .unwrap()
    .iter()
    .map(|c| c.invoice.id)
    .collect();
    assert_eq!(listed_x, vec![x], "X l'est, par son rappel");
}

/// Story 29-2, AC5 — l'ancre scalaire est scopée société (KF-002) : la
/// facture d'une société lue sous une autre rend `Ok(None)` (mutation M11), et
/// la proposition sous l'autre société ne la rend pas.
#[sqlx::test(migrations = "./test-schema")]
async fn the_anchor_of_an_invoice_is_scoped_to_its_company(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let other_company_id = create_test_company(&pool, "Autre").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
    let at = |k: i64| d + chrono::Duration::days(k);

    // Le cas d d'AC5 : candidate dans sa société.
    let inv = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        at(-90),
        dec!(500.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;
    set_test_due_date(&pool, inv, Some(at(-60))).await;
    insert_test_reminder(&pool, company_id, inv, at(-10), false).await;

    assert_eq!(
        reconciliation_repo::invoice_payment_anchor_date(&pool, company_id, inv)
            .await
            .unwrap(),
        Some(at(-10))
    );
    assert_eq!(
        reconciliation_repo::invoice_payment_anchor_date(&pool, other_company_id, inv)
            .await
            .unwrap(),
        None,
        "facture d'une autre société : None"
    );
    let listed = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        other_company_id,
        InvoiceDateWindow::for_payment_date(d),
        dec!(500.00),
        dec!(0.05),
    )
    .await
    .unwrap();
    assert!(listed.is_empty(), "pas de candidate sous l'autre société");
}

// ---------------------------------------------------------------------------
// Story 29-5 (#668) — l'ordre d'imputation (art. 87 CO) avant le `LIMIT 50`
// ---------------------------------------------------------------------------

/// Story 29-5, AC4 — la proposition trie ses candidates par **date
/// d'exigibilité** `X = max(F, E)` croissante, puis `F`, puis `id`, **avant**
/// le `LIMIT 50` : au-delà de 50, ce sont les factures d'exigibilité la plus
/// lointaine qui sont écartées, non plus les plus récentes par date.
///
/// 51 factures de même reste dû, `D` = 2026-06-15 :
/// - `fd` : `F = D`, `E = D` — la plus récente, échue le jour du paiement,
///   de plus petite `X` ;
/// - `nd` : `F = D + 1`, **sans échéance** — `X = D + 1`, deuxième ; sous la
///   mutation M9 (`GREATEST` sans `COALESCE`), son `X` vaut `NULL` ;
/// - quarante-neuf factures `k = 0..48`, plus anciennes (`F` de `D − 30` à
///   `D − 1`), d'échéances lointaines **distinctes** `E = D + 40 + k`.
///
/// Insertion : `nd`, puis les quarante-neuf par `k` décroissant, puis `fd` —
/// `id ASC` et `(date, id)` divergent tous deux de l'ordre attendu, écrit en
/// dur par les `k` (jamais par `imputation_order`).
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_orders_by_exigibility_before_limit(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
    let at = |k: i64| d + chrono::Duration::days(k);

    let insert = |date: NaiveDate| {
        let pool = pool.clone();
        async move {
            insert_test_invoice(
                &pool,
                company_id,
                contact_id,
                date,
                dec!(100.00),
                "validated",
                Some(je_id),
                None,
            )
            .await
        }
    };

    // (F, id) de chaque facture, pour le calcul à la main de l'ancien ordre.
    let mut inserted: Vec<(NaiveDate, i64)> = Vec::new();
    let nd = insert(at(1)).await;
    inserted.push((at(1), nd));
    let mut by_k = [0i64; 49];
    for k in (0..49i64).rev() {
        let f = at(-1 - (k % 30));
        let id = insert(f).await;
        set_test_due_date(&pool, id, Some(at(40 + k))).await;
        by_k[k as usize] = id;
        inserted.push((f, id));
    }
    let fd = insert(d).await;
    set_test_due_date(&pool, fd, Some(d)).await;
    inserted.push((d, fd));

    // L'attendu : `fd`, `nd`, puis `k = 0..47` ; `k = 48` est écartée.
    let mut expected = vec![fd, nd];
    expected.extend(by_k[..48].iter().copied());
    // Montage : l'ancien ordre `(date, id)` ne rend pas l'attendu — sinon le
    // test passerait à vide sous l'ancien `ORDER BY` (mutation M8).
    let mut by_date_id = inserted.clone();
    by_date_id.sort();
    let old: Vec<i64> = by_date_id.iter().take(50).map(|(_, id)| *id).collect();
    assert_ne!(
        old, expected,
        "montage : `(date, id)` ≠ ordre d'exigibilité"
    );

    let got: Vec<i64> = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(d),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window")
    .iter()
    .map(|c| c.invoice.id)
    .collect();
    assert_eq!(
        got, expected,
        "la facture échue le jour du paiement, la facture sans échéance, puis les 48 par échéance croissante"
    );
    assert!(
        !got.contains(&by_k[48]),
        "l'échéance la plus lointaine est écartée"
    );
}

/// Story 29-5, AC4 (validation P1, F2 ; mutation M12) — à date d'exigibilité
/// égale, la date de la facture départage **avant** l'identifiant, et la
/// coupure du `LIMIT 50` traverse un tel groupe.
///
/// 51 factures de même reste dû, `D` = 2026-06-15 : quarante-neuf de
/// remplissage **sans échéance**, `F = D − 25` ; puis G1 (`F = D − 20`) et G2
/// (`F = D − 10`), même échéance `E = D + 10`, **G2 insérée avant G1** (plus
/// petit `id`). Ordre correct `(X, F, id)` : les quarante-neuf, puis G1 ; G2
/// écartée. Sous `(X, id)`, G2 serait gardée et G1 écartée.
#[sqlx::test(migrations = "./test-schema")]
async fn find_unpaid_invoices_orders_same_exigibility_by_invoice_date_before_limit(
    pool: MySqlPool,
) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
    let at = |k: i64| d + chrono::Duration::days(k);

    let insert = |date: NaiveDate| {
        let pool = pool.clone();
        async move {
            insert_test_invoice(
                &pool,
                company_id,
                contact_id,
                date,
                dec!(100.00),
                "validated",
                Some(je_id),
                None,
            )
            .await
        }
    };

    let g2 = insert(at(-10)).await;
    set_test_due_date(&pool, g2, Some(at(10))).await;
    let g1 = insert(at(-20)).await;
    set_test_due_date(&pool, g1, Some(at(10))).await;
    let mut fill = Vec::new();
    for _ in 0..49 {
        fill.push(insert(at(-25)).await);
    }
    assert!(
        g2 < g1 && g1 < fill[0],
        "montage : G2, G1, puis le remplissage"
    );

    let mut expected = fill.clone();
    expected.push(g1);
    // Montage : l'ordre `(X, id)`, calculé à la main sur les mêmes entrées —
    // le remplissage (X = D − 25), puis G2 et G1 (X = D + 10) par `id`.
    let mut by_x_id = fill.clone();
    by_x_id.push(g2);
    assert_ne!(by_x_id, expected, "montage : `(X, id)` ≠ `(X, F, id)`");

    let got: Vec<i64> = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        InvoiceDateWindow::for_payment_date(d),
        dec!(100.00),
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window")
    .iter()
    .map(|c| c.invoice.id)
    .collect();
    assert_eq!(got, expected, "le remplissage, puis G1 (la plus ancienne)");
    assert!(!got.contains(&g2), "G2, plus récente, est écartée");
}

/// Story 29-5, AC4 (C-29-5-8) — **parité** des deux écritures de la date
/// d'exigibilité : l'expression SQL [`INVOICE_EXIGIBILITY_DATE_SQL`] et la
/// fonction `invoice_exigibility_date`, comparées sur six factures à une
/// valeur écrite en dur. Le décodage en `NaiveDate` constate aussi que le
/// `CAST` rend un type que sqlx décode (mutation M9 : un `NULL` n'y passe
/// pas ; M10 : le cas `E < F`).
#[sqlx::test(migrations = "./test-schema")]
async fn exigibility_date_sql_and_rust_agree(pool: MySqlPool) {
    use kesh_db::repositories::invoices;
    use kesh_db::repositories::reconciliation::{
        INVOICE_EXIGIBILITY_DATE_SQL, invoice_exigibility_date,
    };

    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;
    let ymd = |y: i32, m: u32, j: u32| NaiveDate::from_ymd_opt(y, m, j).unwrap();

    // (nom, F, E, X attendue)
    let cases = [
        ("E nulle", ymd(2026, 5, 20), None, ymd(2026, 5, 20)),
        (
            "E > F",
            ymd(2026, 5, 20),
            Some(ymd(2026, 6, 19)),
            ymd(2026, 6, 19),
        ),
        (
            "E = F",
            ymd(2026, 5, 20),
            Some(ymd(2026, 5, 20)),
            ymd(2026, 5, 20),
        ),
        (
            "E < F",
            ymd(2026, 5, 20),
            Some(ymd(2026, 5, 10)),
            ymd(2026, 5, 20),
        ),
        (
            "E lointaine",
            ymd(2026, 5, 20),
            Some(ymd(2099, 12, 31)),
            ymd(2099, 12, 31),
        ),
        (
            "F au 1er jour de l'exercice",
            ymd(2026, 1, 1),
            None,
            ymd(2026, 1, 1),
        ),
    ];
    let mut by_id = std::collections::BTreeMap::new();
    for (name, f, e, x) in cases {
        let id = insert_test_invoice(
            &pool,
            company_id,
            contact_id,
            f,
            dec!(100.00),
            "validated",
            Some(je_id),
            None,
        )
        .await;
        set_test_due_date(&pool, id, e).await;
        by_id.insert(id, (name, x));
    }

    let rows: Vec<(i64, NaiveDate)> = sqlx::query_as(&format!(
        "SELECT i.id, {INVOICE_EXIGIBILITY_DATE_SQL} FROM invoices i \
         WHERE i.company_id = ? ORDER BY i.id"
    ))
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .expect("SELECT de l'exigibilité, décodée en NaiveDate");
    assert_eq!(rows.len(), by_id.len(), "une ligne par facture");
    for (id, sql_x) in rows {
        let (name, x) = by_id[&id];
        assert_eq!(sql_x, x, "{name} : SQL");
        let invoice = invoices::find_by_id(&pool, company_id, id)
            .await
            .expect("find_by_id")
            .expect("facture relue");
        assert_eq!(invoice_exigibility_date(&invoice), x, "{name} : Rust");
    }
}
