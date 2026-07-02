//! Tests d'intégration des lots de paiement pain.001 (Story 12.3, #191).
//!
//! Vérifie bout-en-bout (DB éphémère) :
//! - `create_batch` : accepte les factures valides, refuse per-facture (FailedProposal) ;
//! - `confirm_batch` : poste les règlements de TOUTES les factures (atomique) → `paid`,
//!   solde 2000 = 0 — **prouve l'absence de self-block** (H-FRESH-1, pay_in_tx guard-free) ;
//! - `cancel_batch` : déverrouille les factures ;
//! - le guard : `pay`/`cancel` direct refusés tant que la facture est dans un lot `generated` ;
//! - `generate_pain001_xml` : produit un XML pain.001 valide.

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{
    NewBankAccount, NewPaymentBatch, NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{bank_accounts, contacts, payment_batches, supplier_invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

const IBAN_A: &str = "CH5604835012345678009";
const QR_IBAN: &str = "CH4431999123000889012";
const QRR: &str = "210000000003139471430009017";

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

struct Ctx {
    seeded: SeededCompany,
    supplier_id: i64,
    bank_id: i64,
}

async fn setup(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.unwrap();
    let recoverable_id = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, '1171', 'Impôt préalable', 'Asset')",
    )
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ?, \
         default_vat_recoverable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(recoverable_id)
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap();
    let supplier_id = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Fournisseur SA".into(),
            is_client: false,
            is_supplier: true,
            address: Some("Rue 2\n1000 Lausanne".into()),
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: None,
        },
    )
    .await
    .unwrap()
    .id;
    // Compte bancaire source lié au compte grand livre 1100.
    let bank = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque Source".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(seeded.accounts["1100"])
        .bind(bank.id)
        .execute(pool)
        .await
        .unwrap();

    Ctx {
        seeded,
        supplier_id,
        bank_id: bank.id,
    }
}

/// Crée une facture fournisseur `open` avec coordonnées de paiement.
async fn make_invoice(
    pool: &MySqlPool,
    ctx: &Ctx,
    iban: Option<&str>,
    qr_iban: Option<&str>,
    reference: Option<&str>,
    unit_price: Decimal,
) -> i64 {
    supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: ctx.seeded.company_id,
            contact_id: ctx.supplier_id,
            supplier_invoice_number: Some("FF-001".into()),
            invoice_date: d(2026, 6, 15),
            due_date: None,
            creditor_iban: iban.map(String::from),
            creditor_qr_iban: qr_iban.map(String::from),
            payment_reference: reference.map(String::from),
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Achat".into(),
                quantity: dec!(1),
                unit_price,
                vat_rate: dec!(0),
                expense_account_id: ctx.seeded.accounts["4000"],
            }],
        },
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap()
    .invoice
    .id
}

fn new_batch(ctx: &Ctx, ids: Vec<i64>) -> NewPaymentBatch {
    NewPaymentBatch {
        company_id: ctx.seeded.company_id,
        bank_account_id: ctx.bank_id,
        requested_execution_date: d(2026, 7, 1),
        supplier_invoice_ids: ids,
    }
}

async fn account_balance(pool: &MySqlPool, company_id: i64, account_id: i64) -> Decimal {
    sqlx::query_scalar::<_, Decimal>(
        "SELECT COALESCE(SUM(jel.debit - jel.credit), 0) FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE je.company_id = ? AND jel.account_id = ?",
    )
    .bind(company_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_batch_accepts_valid_invoices(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv1 = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("Réf 1"), dec!(100.00)).await;
    let inv2 = make_invoice(&pool, &ctx, None, Some(QR_IBAN), Some(QRR), dec!(200.00)).await;

    let outcome = payment_batches::create_batch(
        &pool,
        new_batch(&ctx, vec![inv1, inv2]),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let batch = outcome.batch.expect("lot créé");
    assert_eq!(batch.batch.status, "generated");
    assert_eq!(batch.items.len(), 2);
    assert_eq!(batch.batch.total_amount, dec!(300.00));
    assert!(outcome.failed.is_empty());
    assert!(batch.batch.msg_id.starts_with("KESH-"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_batch_rejects_invalid_per_invoice(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let ok = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    let no_coords = make_invoice(&pool, &ctx, None, None, None, dec!(50.00)).await;

    let outcome = payment_batches::create_batch(
        &pool,
        new_batch(&ctx, vec![ok, no_coords, 999_999]),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let batch = outcome.batch.expect("lot créé (1 accepté)");
    assert_eq!(batch.items.len(), 1);
    let codes: Vec<&str> = outcome
        .failed
        .iter()
        .map(|f| f.error_code.as_str())
        .collect();
    assert!(codes.contains(&"NO_PAYMENT_COORDINATES"));
    assert!(codes.contains(&"SUPPLIER_INVOICE_NOT_FOUND"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn confirm_batch_posts_settlements_no_self_block(pool: MySqlPool) {
    // CŒUR H-FRESH-1 : confirm_batch appelle pay_in_tx pendant que le lot est
    // generated → DOIT aboutir (pas de self-block) et solder 2000.
    let ctx = setup(&pool).await;
    let inv1 = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R1"), dec!(100.00)).await;
    let inv2 = make_invoice(&pool, &ctx, None, Some(QR_IBAN), Some(QRR), dec!(200.00)).await;
    let outcome = payment_batches::create_batch(
        &pool,
        new_batch(&ctx, vec![inv1, inv2]),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let batch_id = outcome.batch.unwrap().batch.id;

    let confirmed = payment_batches::confirm_batch(
        &pool,
        ctx.seeded.company_id,
        batch_id,
        d(2026, 7, 2),
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("confirmation aboutit (pas de self-block)");
    assert_eq!(confirmed.batch.status, "confirmed");
    assert!(confirmed.batch.confirmed_at.is_some());

    // Les 2 factures sont payées.
    for id in [inv1, inv2] {
        let (inv, _) = supplier_invoices::get(&pool, ctx.seeded.company_id, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(inv.status, "paid");
        assert_eq!(inv.settlement_type.as_deref(), Some("bank_transfer"));
    }
    // Solde créanciers 2000 = 0 (achats C 300 + règlements D 300).
    let payable = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["2000"]).await;
    assert_eq!(payable, dec!(0.00));
    // Banque 1100 créditée de 300.
    let bank = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["1100"]).await;
    assert_eq!(bank, dec!(-300.00));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cancel_batch_unlocks_invoices(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    let outcome =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![inv]), ctx.seeded.admin_user_id)
            .await
            .unwrap();
    let batch_id = outcome.batch.unwrap().batch.id;

    // Pendant generated : pay direct refusé.
    let err = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        inv,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 7, 2),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));

    // Annulation du lot → déverrouille.
    let cancelled = payment_batches::cancel_batch(
        &pool,
        ctx.seeded.company_id,
        batch_id,
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    assert_eq!(cancelled.batch.status, "cancelled");

    // pay direct fonctionne à nouveau.
    let paid = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        inv,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 7, 2),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    assert_eq!(paid.invoice.status, "paid");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invoice_cannot_be_in_two_generated_batches(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    payment_batches::create_batch(&pool, new_batch(&ctx, vec![inv]), ctx.seeded.admin_user_id)
        .await
        .unwrap()
        .batch
        .unwrap();
    // 2e lot avec la même facture → refusée (déjà dans un lot generated).
    let outcome2 =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![inv]), ctx.seeded.admin_user_id)
            .await
            .unwrap();
    assert!(outcome2.batch.is_none());
    assert_eq!(outcome2.failed[0].error_code, "ALREADY_IN_GENERATED_BATCH");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn confirm_already_confirmed_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    let batch_id =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![inv]), ctx.seeded.admin_user_id)
            .await
            .unwrap()
            .batch
            .unwrap()
            .batch
            .id;
    payment_batches::confirm_batch(
        &pool,
        ctx.seeded.company_id,
        batch_id,
        d(2026, 7, 2),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let err = payment_batches::confirm_batch(
        &pool,
        ctx.seeded.company_id,
        batch_id,
        d(2026, 7, 2),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn generate_pain001_xml_is_well_formed(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv1 = make_invoice(
        &pool,
        &ctx,
        Some(IBAN_A),
        None,
        Some("Facture 1"),
        dec!(100.00),
    )
    .await;
    let inv2 = make_invoice(&pool, &ctx, None, Some(QR_IBAN), Some(QRR), dec!(200.00)).await;
    let batch_id = payment_batches::create_batch(
        &pool,
        new_batch(&ctx, vec![inv1, inv2]),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap()
    .batch
    .unwrap()
    .batch
    .id;

    let xml = payment_batches::generate_pain001_xml(&pool, ctx.seeded.company_id, batch_id)
        .await
        .unwrap();
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(xml.contains("pain.001.001.09"));
    assert_eq!(xml.matches("<NbOfTxs>2</NbOfTxs>").count(), 2);
    assert!(xml.contains("<CtrlSum>300.00</CtrlSum>"));
    assert!(xml.contains(&format!("<IBAN>{IBAN_A}</IBAN>")));
    assert!(xml.contains(&format!("<Ref>{QRR}</Ref>")));
}
