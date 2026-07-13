//! Tests d'intégration pour `repositories::reconciliation` (Story 8-4 T3).
//!
//! H2 Pass 1 code review : 5 tests `#[sqlx::test]` couvrant AC #37-#40
//! et AC #47 — fenêtre temporelle, multi-tenant scoping, status filter,
//! amount tolerance, et exclusion des transactions auto-rejetées.
//!
//! Pattern `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` — DB éphémère
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
    // #246 (21-2b) : le matching filtre désormais sur le TTC (dérivé des
    // lignes). Une facture sans ligne aurait un TTC = 0 et ne matcherait
    // jamais. On pose une ligne unique à vat_rate 0 → TTC = total_amount,
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

/// AC #38 — fenêtre temporelle ± 30 jours autour de la booking_date
/// de la transaction. Une invoice 31 jours avant ou après la tx
/// n'est PAS retournée.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_unpaid_invoices_filters_by_30_day_window(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let contact_id = create_test_contact(&pool, company_id, user_id, "Client A").await;
    let fy_id = insert_fake_fiscal_year(&pool, company_id).await;
    let je_id = insert_fake_journal_entry(&pool, company_id, fy_id).await;

    let tx_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();

    // Inside window : tx_date - 15 jours.
    let inv_in = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;

    // Outside window : tx_date - 31 jours (avant la fenêtre).
    let _inv_out_before = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
        dec!(100.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;

    // Outside window : tx_date + 31 jours (après la fenêtre).
    let _inv_out_after = insert_test_invoice(
        &pool,
        company_id,
        contact_id,
        NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        dec!(100.00),
        "validated",
        Some(je_id),
        None,
    )
    .await;

    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        tx_date,
        dec!(100.00),
        30,
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let ids: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert!(
        ids.contains(&inv_in),
        "inv_in (within window) doit être candidat"
    );
    assert_eq!(
        candidates.len(),
        1,
        "seulement l'invoice dans la fenêtre ± 30j doit être retournée"
    );
}

/// AC #38 + AC #59 — multi-tenant scoping (KF-002 Pattern 1) :
/// `find_unpaid_invoices_for_window(company_A)` ne doit pas retourner
/// les invoices de company_B même si elles sont dans la même fenêtre.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
        day,
        dec!(100.00),
        30,
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
        day,
        dec!(100.00),
        30,
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
        day,
        dec!(100.00),
        30,
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

/// AC #39 + AC #40 — tolérance amount ± 0.05 CHF. Une invoice à
/// 100.04 (delta 0.04) est candidate, mais 100.06 (delta 0.06) ne
/// l'est pas.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
        day,
        dec!(100.00),
        30,
        dec!(0.05),
    )
    .await
    .expect("find_unpaid_invoices_for_window");

    let ids: Vec<i64> = candidates.iter().map(|c| c.invoice.id).collect();
    assert_eq!(
        ids,
        vec![inv_in],
        "seule l'invoice à ± 0.05 doit être retournée"
    );
}

/// AC #47 — `find_pending_transactions_for_account` exclut les
/// transactions avec `auto_match_rejected_at IS NOT NULL`. Une tx
/// rejetée manuellement n'est plus exposée via la liste pending.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
        tx_date,
        dec!(108.10),
        30,
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

    // tx = 100.00 (HT) → PAS de match (le bug d'avant matchait le HT).
    let found_ht = reconciliation_repo::find_unpaid_invoices_for_window(
        &pool,
        company_id,
        tx_date,
        dec!(100.00),
        30,
        dec!(0.05),
    )
    .await
    .expect("find_unpaid HT");
    assert!(
        !found_ht.iter().any(|c| c.invoice.id == inv_id),
        "le HT 100.00 ne doit plus matcher (régression #246 corrigée)"
    );
}
