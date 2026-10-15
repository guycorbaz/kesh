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
            first_name: None,
            last_name: None,
            is_client: false,
            is_supplier: true,
            address: Some("Rue 2\n1000 Lausanne".into()),
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

// ===========================================================================
// Story 25-3-a-2 (#414) — annuler le règlement d'une facture payée par lot
// ===========================================================================

/// Crée un lot `[id]` et le confirme à `on` ; rend l'id du lot.
async fn confirmed_batch(pool: &MySqlPool, ctx: &Ctx, id: i64, on: NaiveDate) -> i64 {
    let batch_id =
        payment_batches::create_batch(pool, new_batch(ctx, vec![id]), ctx.seeded.admin_user_id)
            .await
            .unwrap()
            .batch
            .expect("lot créé")
            .batch
            .id;
    payment_batches::confirm_batch(
        pool,
        ctx.seeded.company_id,
        batch_id,
        on,
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("confirmation");
    batch_id
}

/// ⛔ **Une facture payée par un lot CONFIRMÉ s'annule, et le lot n'est pas
/// modifié** (arbitrage : il reste l'historique de l'ordre transmis). Le champ
/// historique `last_confirmed_batch_for_invoice` le nomme.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelling_a_batch_paid_settlement_leaves_the_batch_alone(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R1"), dec!(100.00)).await;
    let batch_id = confirmed_batch(&pool, &ctx, inv, d(2026, 7, 2)).await;

    let done = supplier_invoices::cancel_settlement(
        &pool,
        ctx.seeded.company_id,
        inv,
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("annulation");
    assert_eq!(done.invoice.invoice.status, "open");

    let (status, items): (String, i64) = sqlx::query_as(
        "SELECT pb.status, (SELECT COUNT(*) FROM payment_batch_items WHERE payment_batch_id = pb.id) \
         FROM payment_batches pb WHERE pb.id = ?",
    )
    .bind(batch_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (status.as_str(), items),
        ("confirmed", 1),
        "le lot est inchangé"
    );

    let last = payment_batches::last_confirmed_batch_for_invoice(&pool, ctx.seeded.company_id, inv)
        .await
        .unwrap();
    assert_eq!(last.map(|(id, _)| id), Some(batch_id));
}

/// ⛔ **La facture est bornée à la société, pas seulement le lot** (revue P1).
/// L'état est forcé à la main — aucun chemin de l'application ne met la facture
/// d'une société dans le lot d'une autre : on rattache le lot confirmé à une
/// seconde société. Sans la jointure sur `supplier_invoices.company_id`, la
/// seconde société verrait le lot sur la facture de la première.
#[sqlx::test(migrations = "./test-schema")]
async fn last_confirmed_batch_scopes_the_invoice_to_the_company(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R1"), dec!(100.00)).await;
    let batch_id = confirmed_batch(&pool, &ctx, inv, d(2026, 7, 2)).await;
    let other = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query("UPDATE payment_batches SET company_id = ? WHERE id = ?")
        .bind(other)
        .bind(batch_id)
        .execute(&pool)
        .await
        .unwrap();

    let last = payment_batches::last_confirmed_batch_for_invoice(&pool, other, inv)
        .await
        .unwrap();
    assert_eq!(last, None, "la facture n'est pas de cette société");
}

/// ⛔ **Le champ est HISTORIQUE** : après « lot confirmé → annulation →
/// règlement DIRECT », il désigne toujours l'ancien lot (qui n'a pas produit le
/// règlement courant) — c'est pourquoi le texte d'avertissement ne dit jamais
/// « payée par ce lot ». Puis, après un second lot confirmé, il désigne le plus
/// récent.
#[sqlx::test(migrations = "./test-schema")]
async fn last_confirmed_batch_is_historical_and_most_recent(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (c, u) = (ctx.seeded.company_id, ctx.seeded.admin_user_id);
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R1"), dec!(100.00)).await;
    let first = confirmed_batch(&pool, &ctx, inv, d(2026, 7, 2)).await;

    supplier_invoices::cancel_settlement(&pool, c, inv, u)
        .await
        .unwrap();
    supplier_invoices::pay(
        &pool,
        c,
        inv,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 7, 5),
        u,
    )
    .await
    .expect("règlement direct");
    let last = payment_batches::last_confirmed_batch_for_invoice(&pool, c, inv)
        .await
        .unwrap();
    assert_eq!(last.map(|(id, _)| id), Some(first), "toujours l'ancien lot");

    supplier_invoices::cancel_settlement(&pool, c, inv, u)
        .await
        .unwrap();
    let second = confirmed_batch(&pool, &ctx, inv, d(2026, 7, 9)).await;
    let last = payment_batches::last_confirmed_batch_for_invoice(&pool, c, inv)
        .await
        .unwrap();
    assert_eq!(last.map(|(id, _)| id), Some(second), "le plus récent");
    assert_ne!(first, second);
}

// ---------------------------------------------------------------------------
// Story 15-6b (#474, AC6) — refus à la création, garde à la confirmation
// ---------------------------------------------------------------------------

/// Relie le compte bancaire source du contexte au compte du grand livre `ledger`.
async fn link_bank(pool: &MySqlPool, ctx: &Ctx, ledger: i64) {
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ledger)
        .bind(ctx.bank_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Le compte de banque 1020, absent du seed.
async fn ledger_1020(pool: &MySqlPool, ctx: &Ctx) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, '1020', 'Banque', 'Asset', 1, 1) RETURNING id",
    )
    .bind(ctx.seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Story 15-6b, test 13 — **création** : A (dette sur 2000) refusée par
/// facture, B (dette sur 2001) retenue ; avec A seule, aucun lot.
#[sqlx::test(migrations = "./test-schema")]
async fn create_batch_refuses_an_invoice_whose_payable_is_the_bank_ledger(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let a = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("A"), dec!(100.00)).await;
    let p2001: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '2001', 'Créanciers bis', 'Liability') RETURNING id",
    )
    .bind(ctx.seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(p2001)
    .bind(ctx.seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let b = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("B"), dec!(200.00)).await;
    let payable = ctx.seeded.accounts["2000"];
    link_bank(&pool, &ctx, payable).await;

    let outcome =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![a, b]), ctx.seeded.admin_user_id)
            .await
            .unwrap();
    let batch = outcome.batch.expect("B forme un lot");
    assert_eq!(batch.items.len(), 1);
    assert_eq!(batch.items[0].supplier_invoice_id, b);
    assert_eq!(outcome.failed.len(), 1);
    let failed = &outcome.failed[0];
    assert_eq!(failed.supplier_invoice_id, a);
    assert_eq!(
        failed.error_code,
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    assert_eq!(
        failed.details,
        Some(serde_json::json!({
            "bankAccountId": ctx.bank_id,
            "accountId": payable,
            "accountNumber": "2000",
            "claim": "payable",
            "role": "counterparty",
        }))
    );

    // A seule : aucun lot n'est créé — donc aucun fichier pain.001.
    let outcome =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![a]), ctx.seeded.admin_user_id)
            .await
            .unwrap();
    assert!(outcome.batch.is_none(), "aucun lot");
    assert_eq!(
        outcome.failed[0].error_code,
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    let batches: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM payment_batches")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(batches, 1, "seul le lot de B existe");
}

/// Story 15-6b, test 14 — **écriture d'achat sans ligne de crédit** : la
/// facture va dans `failed[]` (donnée de CETTE facture), les autres restent.
#[sqlx::test(migrations = "./test-schema")]
async fn create_batch_reports_a_purchase_entry_without_credit_line_per_invoice(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let ok = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("OK"), dec!(100.00)).await;
    let bad = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("KO"), dec!(50.00)).await;
    let empty_entry: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, 999999, '2026-06-15', 'Achats', 'Écriture vide') RETURNING id",
    )
    .bind(ctx.seeded.company_id)
    .bind(ctx.seeded.fiscal_year_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE supplier_invoices SET purchase_journal_entry_id = ? WHERE id = ?")
        .bind(empty_entry)
        .bind(bad)
        .execute(&pool)
        .await
        .unwrap();

    let outcome = payment_batches::create_batch(
        &pool,
        new_batch(&ctx, vec![ok, bad]),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let batch = outcome.batch.expect("lot créé avec la facture saine");
    assert_eq!(batch.items.len(), 1);
    assert_eq!(batch.items[0].supplier_invoice_id, ok);
    assert_eq!(outcome.failed.len(), 1);
    assert_eq!(outcome.failed[0].supplier_invoice_id, bad);
    assert_eq!(
        outcome.failed[0].error_code,
        "SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED"
    );
    assert_eq!(
        outcome.failed[0].details,
        Some(serde_json::json!({
            "reason": "no_credit_line_on_purchase_entry",
            "purchaseEntryId": empty_entry,
        }))
    );
}

/// Story 15-6b, test 15 — **confirmation, puis remède** : le compte bancaire,
/// relié au 2000 après la création, bloque la confirmation (400 contextualisé,
/// lot toujours `generated`, rien d'écrit) ; relié de nouveau au 1020, elle
/// passe.
#[sqlx::test(migrations = "./test-schema")]
async fn confirm_batch_refuses_then_passes_once_the_bank_is_relinked(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let l1020 = ledger_1020(&pool, &ctx).await;
    link_bank(&pool, &ctx, l1020).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    let batch_id =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![inv]), ctx.seeded.admin_user_id)
            .await
            .unwrap()
            .batch
            .expect("lot créé")
            .batch
            .id;
    let payable = ctx.seeded.accounts["2000"];
    link_bank(&pool, &ctx, payable).await;
    let entries_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(&pool)
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
    .expect_err("la confirmation doit être refusée");
    match err {
        DbError::SettlementCounterpartyIsClaimAccount {
            account_id,
            ref account_number,
            claim: kesh_db::errors::ClaimSide::Payable,
            role: kesh_db::errors::SettlementAccountRole::Counterparty,
            batch: Some(ref ctx_batch),
        } => {
            assert_eq!(account_id, payable);
            assert_eq!(account_number.as_deref(), Some("2000"));
            assert_eq!(ctx_batch.payment_batch_id, batch_id);
            assert_eq!(ctx_batch.supplier_invoice_id, inv);
            assert_eq!(ctx_batch.supplier_invoice_number.as_deref(), Some("FF-001"));
        }
        other => panic!("refus contextualisé attendu, obtenu {other:?}"),
    }
    let (status,): (String,) = sqlx::query_as("SELECT status FROM payment_batches WHERE id = ?")
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "generated");
    let inv_status: String =
        sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
            .bind(inv)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(inv_status, "open");
    let entries_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(entries_after, entries_before, "aucun règlement écrit");

    // Le remède : relier de nouveau le compte bancaire à son compte de banque.
    link_bank(&pool, &ctx, l1020).await;
    let confirmed = payment_batches::confirm_batch(
        &pool,
        ctx.seeded.company_id,
        batch_id,
        d(2026, 7, 2),
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("la confirmation passe après le remède");
    assert_eq!(confirmed.batch.status, "confirmed");
    assert_eq!(
        account_balance(&pool, ctx.seeded.company_id, payable).await,
        Decimal::ZERO
    );
}

/// Story 15-6b, revue de code P1 (findings L4 = E3) — **atomicité de la
/// confirmation** : un lot de deux factures, A (dette sur 2001) puis B (dette
/// sur 2000). Le compte bancaire, relié au 2000 après la création, laisse
/// passer A et refuse B. Le règlement de A, déjà écrit dans la transaction,
/// doit être défait avec elle : A reste `open`, aucune écriture neuve, le lot
/// reste `generated`. Le test à une seule facture (test 15) ne pouvait pas le
/// montrer — le refus y tombait avant toute écriture.
///
/// Mutation constatée rouge : `tx.rollback()` remplacé par `tx.commit()` dans
/// la branche d'échec de `confirm_batch` (l'effet d'un « commit par facture »).
#[sqlx::test(migrations = "./test-schema")]
async fn confirm_batch_rolls_back_the_invoices_settled_before_the_refused_one(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let l1020 = ledger_1020(&pool, &ctx).await;
    link_bank(&pool, &ctx, l1020).await;
    let p2001: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '2001', 'Créanciers bis', 'Liability') RETURNING id",
    )
    .bind(ctx.seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let set_payable = |account: i64| {
        sqlx::query(
            "UPDATE company_invoice_settings SET default_payable_account_id = ? \
             WHERE company_id = ?",
        )
        .bind(account)
        .bind(ctx.seeded.company_id)
        .execute(&pool)
    };
    set_payable(p2001).await.unwrap();
    let a = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("A"), dec!(100.00)).await;
    let payable = ctx.seeded.accounts["2000"];
    set_payable(payable).await.unwrap();
    let b = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("B"), dec!(200.00)).await;

    let batch_id =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![a, b]), ctx.seeded.admin_user_id)
            .await
            .unwrap()
            .batch
            .expect("lot créé avec A et B")
            .batch
            .id;
    // Montage : A est réglée AVANT B — sans quoi le refus tomberait avant
    // toute écriture et le test passerait à vide.
    let order: Vec<i64> = sqlx::query_scalar(
        "SELECT supplier_invoice_id FROM payment_batch_items \
         WHERE payment_batch_id = ? ORDER BY position",
    )
    .bind(batch_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(order, vec![a, b], "A doit précéder B dans le lot");

    link_bank(&pool, &ctx, payable).await;
    let entries_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(&pool)
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
    .expect_err("la confirmation doit être refusée sur B");
    match err {
        DbError::SettlementCounterpartyIsClaimAccount {
            batch: Some(ref ctx_batch),
            ..
        } => assert_eq!(ctx_batch.supplier_invoice_id, b, "le refus nomme B"),
        other => panic!("refus contextualisé attendu, obtenu {other:?}"),
    }

    for (id, label) in [(a, "A"), (b, "B")] {
        let status: String =
            sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status, "open", "{label} reste ouverte");
    }
    let entries_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        entries_after, entries_before,
        "le règlement de A est défait avec la transaction"
    );
    assert_eq!(
        account_balance(&pool, ctx.seeded.company_id, p2001).await,
        dec!(-100.00),
        "la dette de A reste entière"
    );
    let status: String = sqlx::query_scalar("SELECT status FROM payment_batches WHERE id = ?")
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "generated");
}

// ===========================================================================
// Story 29-4a (#651) — la revérification verrouillante de la création de lot
// ===========================================================================

/// Montage des courses de W17 : le contexte, une facture payable, et le compte
/// source rendu **non principal** (un principal ne se clôt pas) ; rend sa
/// version.
async fn monter_la_course(pool: &MySqlPool) -> (Ctx, i64, i32) {
    let ctx = setup(pool).await;
    let facture = make_invoice(pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    sqlx::query("UPDATE bank_accounts SET is_primary = FALSE WHERE id = ?")
        .bind(ctx.bank_id)
        .execute(pool)
        .await
        .unwrap();
    let version: i32 = sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
        .bind(ctx.bank_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (ctx, facture, version)
}

/// Lance `create_batch` en tâche (il possède sa transaction, et l'annule
/// lui-même sur erreur), puis attend qu'il soit vu sur son `INSERT INTO
/// payment_batches` — sa vue est née à l'étape (0), lecture non verrouillante
/// du compte. Panique avec l'issue si la tâche a fini sans attendre.
async fn lot_en_attente(
    pool: &MySqlPool,
    ctx: &Ctx,
    facture: i64,
) -> tokio::task::JoinHandle<Result<payment_batches::CreateBatchOutcome, DbError>> {
    let p = pool.clone();
    let nouveau = new_batch(ctx, vec![facture]);
    let user = ctx.seeded.admin_user_id;
    let tache = tokio::spawn(async move { payment_batches::create_batch(&p, nouveau, user).await });
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        pool,
        &["INSERT INTO payment_batches"],
        || tache.is_finished(),
    )
    .await;
    if !vue {
        panic!(
            "la création du lot a fini sans attendre : {:?}",
            tache.await.map(|r| r.map(|o| o.batch.is_some()))
        );
    }
    tache
}

/// Lots et articles écrits.
async fn lignes_de_lot(pool: &MySqlPool) -> (i64, i64) {
    let lots: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM payment_batches")
        .fetch_one(pool)
        .await
        .unwrap();
    let articles: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM payment_batch_items")
        .fetch_one(pool)
        .await
        .unwrap();
    (lots, articles)
}

/// T29a-3 (Story 29-4a, #651, AC3) — **la fenêtre de la création de lot est
/// fermée**. Une clôture tient la ligne du compte source en exclusif sans
/// commiter ; `create_batch` lit le compte ouvert à son étape (0) (sa vue naît
/// là, avant le commit), valide la facture, puis attend sur son `INSERT INTO
/// payment_batches` (clé étrangère vers le compte). Au commit de la clôture,
/// l'`INSERT` passe, et la revérification verrouillante lit `closed_on` posé →
/// `BankAccountClosed` ; `create_batch` annule lui-même sa transaction, rien
/// d'écrit.
///
/// **Rouge au sol** à l'étape (5) : le lot `generated` naît sur un compte clos
/// (I2 défait). Tue M29-2 et M29-3 bis (M29-3, sur la variante des
/// annulations, n'est tuée que par T-a45 étendu).
#[sqlx::test(migrations = "./test-schema")]
async fn a_batch_started_before_the_closure_is_refused(pool: MySqlPool) {
    let (ctx, facture, v) = monter_la_course(&pool).await;
    let company_id = ctx.seeded.company_id;

    // (1) La clôture tient la ligne du compte en exclusif.
    let mut cloture = pool.begin().await.unwrap();
    let r = bank_accounts::close_in_tx(&mut cloture, company_id, ctx.bank_id, v, d(2026, 6, 30))
        .await
        .unwrap();
    assert!(
        matches!(r, bank_accounts::ClosureOutcome::Done { .. }),
        "{r:?}"
    );

    // (2) et (3) La création du lot démarre et attend sur son `INSERT`.
    let lot = lot_en_attente(&pool, &ctx, facture).await;

    // (4) La clôture commite.
    cloture.commit().await.unwrap();

    // (5) Le lot est refusé, rien d'écrit.
    let r = lot.await.expect("tâche du lot").map(|o| o.batch.is_some());
    assert!(
        matches!(r, Err(DbError::BankAccountClosed { bank_account_id }) if bank_account_id == ctx.bank_id),
        "⛔ le lot commencé avant la clôture devait être refusé — reçu {r:?}"
    );
    assert_eq!(lignes_de_lot(&pool).await, (0, 0), "rien d'écrit");
}

/// T29a-12 (Story 29-4a, #651, AC3) — **la course archivage / lot est fermée**.
/// Même patron que T29a-3, l'archivage (sous la sentinelle `companies`, comme
/// la route) à la place de la clôture. La revérification lit `archived = TRUE`
/// et rend `NotFound`, que `create_batch` convertit en
/// `IllegalStateTransition` — le refus même de son étape (0) pour un compte
/// archivé (AC2). Rien d'écrit.
///
/// **Rouge au sol** : le lot naît sur un compte archivé. Tue M29-22, M29-23 et
/// M29-25 (conversion retirée : `NotFound` au lieu d'`IllegalStateTransition`).
#[sqlx::test(migrations = "./test-schema")]
async fn a_batch_started_before_the_archiving_is_refused(pool: MySqlPool) {
    let (ctx, facture, v) = monter_la_course(&pool).await;
    let company_id = ctx.seeded.company_id;

    // (1) L'archivage, sous la sentinelle, sans commiter.
    let mut archivage = pool.begin().await.unwrap();
    bank_accounts::acquire_company_sentinel_lock(&mut archivage, company_id)
        .await
        .unwrap();
    bank_accounts::archive_for_company(&mut archivage, company_id, ctx.bank_id, v)
        .await
        .unwrap();

    // (2) et (3) La création du lot démarre et attend sur son `INSERT`.
    let lot = lot_en_attente(&pool, &ctx, facture).await;

    // (4) L'archivage commite.
    archivage.commit().await.unwrap();

    // (5) Le lot est refusé par le refus de l'étape (0), rien d'écrit.
    let r = lot.await.expect("tâche du lot").map(|o| o.batch.is_some());
    assert!(
        matches!(&r, Err(DbError::IllegalStateTransition(m)) if m.contains("archivé")),
        "⛔ le lot commencé avant l'archivage devait être refusé — reçu {r:?}"
    );
    assert_eq!(lignes_de_lot(&pool).await, (0, 0), "rien d'écrit");
}

// ===========================================================================
// Story 30-1a (#686, AC-a9) — la création d'un lot verrouille ses factures en
// tête, par identifiant croissant
// ===========================================================================

/// Les motifs **communs** au sol et après la correction (fiche, AC-a9, P4
/// F-L2) : la lecture verrouillante d'une facture par `validate_invoice_for_batch`
/// comme le verrouillage de tête portent tous deux ces deux fragments.
const ATTENTE_SUR_FACTURE: [&str; 2] = ["FROM supplier_invoices", "FOR UPDATE"];

/// Une connexion qui tient la facture `id` en exclusif.
async fn tenir_facture(pool: &MySqlPool, id: i64) -> sqlx::Transaction<'static, sqlx::MySql> {
    let mut tenue = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM supplier_invoices WHERE id = ? FOR UPDATE")
        .bind(id)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();
    tenue
}

/// Lance `create_batch` sur `ids` en tâche, puis attend de voir **au moins
/// `en_attente`** requêtes bloquées sur une facture (la sonde qui compte : deux
/// créations attendent sur le même texte préparé).
async fn lot_bloque_sur_facture(
    pool: &MySqlPool,
    ctx: &Ctx,
    ids: Vec<i64>,
    en_attente: i64,
) -> tokio::task::JoinHandle<Result<payment_batches::CreateBatchOutcome, DbError>> {
    let p = pool.clone();
    let nouveau = new_batch(ctx, ids);
    let user = ctx.seeded.admin_user_id;
    let tache = tokio::spawn(async move { payment_batches::create_batch(&p, nouveau, user).await });
    let vue = kesh_db::test_fixtures::attendre_des_requetes_en_cours(
        pool,
        &ATTENTE_SUR_FACTURE,
        en_attente,
        || tache.is_finished(),
    )
    .await;
    if !vue {
        panic!(
            "la création du lot a fini sans attendre la facture : {:?}",
            tache.await.map(|r| r.map(|o| o.batch.is_some()))
        );
    }
    tache
}

/// T-a15 (AC-a9) — **deux créations de lot simultanées ne mettent jamais la
/// même facture dans deux lots `generated`** (deux fichiers pain.001 : un
/// double paiement). Une connexion tient la facture ; A démarre et se bloque ;
/// B démarre et se bloque à son tour (la sonde attend DEUX requêtes) ; la
/// connexion relâche, A crée son lot et commite, B obtient la facture.
///
/// **Rouge au sol** : B a fait sa lecture ordinaire du compte (étape (0))
/// **avant** de se bloquer — sa vue est née ; il lit l'appartenance à un lot
/// dans cette vue, ne voit pas l'article de A, et crée un second lot. Après la
/// correction, B verrouille la facture **avant** toute lecture : sa vue naît
/// après le commit de A, et il refuse `ALREADY_IN_GENERATED_BATCH`. Tue M-a7.
///
/// Cas ajouté : une liste vide ne crée aucun lot et ne rend aucun refus (le
/// verrouillage de tête est court-circuité : `IN ()` est une erreur de syntaxe).
#[sqlx::test(migrations = "./test-schema")]
async fn two_concurrent_batches_never_take_the_same_invoice(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let facture = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;

    let tenue = tenir_facture(&pool, facture).await;
    let a = lot_bloque_sur_facture(&pool, &ctx, vec![facture], 1).await;
    let b = lot_bloque_sur_facture(&pool, &ctx, vec![facture], 2).await;
    tenue.rollback().await.unwrap();

    let a = a.await.unwrap().expect("A");
    let b = b.await.unwrap().expect("B");
    let crees = [&a, &b].iter().filter(|o| o.batch.is_some()).count();
    assert_eq!(
        crees, 1,
        "⛔ la même facture dans deux lots generated : A {:?}, B {:?}",
        a.failed, b.failed
    );
    let refuse = if a.batch.is_some() { &b } else { &a };
    assert_eq!(refuse.failed.len(), 1, "{:?}", refuse.failed);
    assert_eq!(refuse.failed[0].error_code, "ALREADY_IN_GENERATED_BATCH");
    assert_eq!(lignes_de_lot(&pool).await, (1, 1));

    // Liste vide : ni lot, ni refus, ni erreur.
    let vide =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![]), ctx.seeded.admin_user_id)
            .await
            .expect("liste vide");
    assert!(vide.batch.is_none() && vide.failed.is_empty(), "{vide:?}");
}

/// T-a12 étendu (AC-a9) — **le plan du verrouillage de tête** est un parcours
/// de la clé primaire (`range` sur `PRIMARY`) : les verrous suivent l'ordre du
/// parcours, croissant ; un parcours par `idx_supplier_invoices_company_status`
/// verrouillerait toutes les factures de la société, dans un autre ordre. Le
/// décor en porte quatre, pour que l'index de la société ne soit pas écarté
/// par accident. Chaque `?` est remplacé, dans l'ordre, et le test vérifie qu'il
/// n'en reste aucun.
#[sqlx::test(migrations = "./test-schema")]
async fn the_head_lock_of_a_batch_scans_the_primary_key(pool: MySqlPool) {
    use sqlx::Row;
    let ctx = setup(&pool).await;
    let mut ids = Vec::new();
    for prix in [dec!(10.00), dec!(20.00), dec!(30.00), dec!(40.00)] {
        ids.push(make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), prix).await);
    }
    let cibles = [ids[0], ids[2]];
    let mut lie = payment_batches::lock_invoices_sql(cibles.len());
    for id in cibles {
        lie = lie.replacen('?', &id.to_string(), 1);
    }
    lie = lie.replacen('?', &ctx.seeded.company_id.to_string(), 1);
    assert!(!lie.contains('?'), "un `?` reste à lier : {lie}");
    let lignes = sqlx::query(&format!("EXPLAIN {lie}"))
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(lignes.len(), 1, "{lie}");
    let cle: Option<String> = lignes[0].try_get("key").unwrap();
    let acces: Option<String> = lignes[0].try_get("type").unwrap();
    assert_eq!(cle.as_deref(), Some("PRIMARY"), "{lie}");
    assert_eq!(acces.as_deref(), Some("range"), "{lie}");
}

/// T-a16 (AC-a9) — **deux lots aux factures croisées ne s'interbloquent pas**,
/// au niveau du dépôt, **sans rejeu**. (1) Une connexion C tient la facture 1 ;
/// (2) A = {1, 2} démarre et se bloque sur 1 sans rien tenir ; (3) B = {2, 1}
/// démarre — **au sol**, B tient 2 et se bloque sur 1 — ; la sonde attend deux
/// requêtes ; (4) C relâche : A, premier en file, obtient 1, puis demande 2,
/// que tient B → **1213 au sol**. Après la correction, B trie {1, 2} et attend 1
/// sans rien tenir : aucun interblocage, A crée le lot des deux factures, B les
/// refuse toutes deux.
///
/// ⛔ **Ce qui tue M-a9, déterministe** : pendant que A et B attendent la
/// facture 1, la facture 2 doit être **libre** — personne ne tient une facture
/// avant celles d'identifiant inférieur. Une sonde `NOWAIT` le vérifie ; au sol
/// et sous M-a9 (verrous dans l'ordre de la requête), B tient 2 et la sonde
/// échoue. *Constaté à la campagne de mutations* : l'interblocage seul ne
/// suffisait pas à tuer M-a9 — MariaDB 10.11 n'a pas toujours octroyé la
/// facture 1 au premier arrivé (A), et quand B l'obtient, aucun cycle ne se
/// forme. L'hypothèse « octroi en file » de la fiche n'est donc pas tenue pour
/// acquise ; l'absence d'interblocage reste vérifiée, en complément.
#[sqlx::test(migrations = "./test-schema")]
async fn crossed_batches_do_not_deadlock(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let un = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R1"), dec!(100.00)).await;
    let deux = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R2"), dec!(50.00)).await;
    assert!(un < deux, "montage : identifiants croissants");

    let tenue = tenir_facture(&pool, un).await;
    let a = lot_bloque_sur_facture(&pool, &ctx, vec![un, deux], 1).await;
    let b = lot_bloque_sur_facture(&pool, &ctx, vec![deux, un], 2).await;
    let deux_libre = kesh_db::test_fixtures::sonde_verrou_nowait(
        &pool,
        "SELECT id FROM supplier_invoices WHERE id = ? FOR UPDATE NOWAIT",
        deux,
    )
    .await;
    tenue.rollback().await.unwrap();
    assert!(
        deux_libre,
        "⛔ une création de lot tient la facture {deux} en attendant la facture {un} : \
         les factures ne sont pas verrouillées par identifiant croissant"
    );

    let a = a.await.unwrap();
    let b = b.await.unwrap();
    let (a, b) = match (a, b) {
        (Ok(a), Ok(b)) => (a, b),
        (a, b) => panic!(
            "⛔ interblocage entre deux lots aux factures croisées — A : {:?}, B : {:?}",
            a.map(|o| o.batch.is_some()),
            b.map(|o| o.batch.is_some())
        ),
    };
    let (cree, refuse) = if a.batch.is_some() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    assert_eq!(cree.batch.as_ref().map(|l| l.items.len()), Some(2));
    assert!(refuse.batch.is_none());
    assert_eq!(refuse.failed.len(), 2, "{:?}", refuse.failed);
    assert!(
        refuse
            .failed
            .iter()
            .all(|f| f.error_code == "ALREADY_IN_GENERATED_BATCH"),
        "{:?}",
        refuse.failed
    );
    // `failed[]` garde l'ordre de la requête de B.
    let ordre: Vec<i64> = refuse
        .failed
        .iter()
        .map(|f| f.supplier_invoice_id)
        .collect();
    let attendu = if std::ptr::eq(refuse, &b) {
        vec![deux, un]
    } else {
        vec![un, deux]
    };
    assert_eq!(ordre, attendu);
}

// ---------------------------------------------------------------------------
// Story 30-1b (#686) — le règlement d'une facture fournisseur par la
// réconciliation : les verrous et les refus de l'étape 3 (AC-b6, AC-b8,
// AC-b10), et la date du paiement (AC-b16)
// ---------------------------------------------------------------------------

use kesh_db::repositories::supplier_invoices::{LockedForSettlement, SupplierSettlementRefusal};

/// `lock_for_bank_settlement_in_tx` dans une transaction neuve, puis `ROLLBACK`.
async fn refus_301b(
    pool: &MySqlPool,
    ctx: &Ctx,
    invoice_id: i64,
    bank_id: i64,
    amount: Decimal,
) -> Result<LockedForSettlement, SupplierSettlementRefusal> {
    let mut tx = pool.begin().await.unwrap();
    let r = supplier_invoices::lock_for_bank_settlement_in_tx(
        &mut tx,
        ctx.seeded.company_id,
        invoice_id,
        bank_id,
        amount,
    )
    .await
    .expect("aucune erreur de base");
    tx.rollback().await.unwrap();
    r
}

/// Les refus de l'étape 3 d'`accept_one_supplier_invoice` (AC-b6), chacun par
/// un décor qui le provoque seul : facture absente ou d'une autre société
/// (`NotFound`, indiscernables), non ouverte, compte bancaire absent ou
/// archivé, clos, sans compte au grand livre, facture en lot `generated`
/// (`InGeneratedBatch`, tout compte), autre facture au montant exact en lot
/// `generated` **de ce compte** (`HeldByPaymentBatch`, garde (d)).
#[sqlx::test(migrations = "./test-schema")]
async fn lock_for_bank_settlement_refuses_each_case(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let ok = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    let locked = refus_301b(&pool, &ctx, ok, ctx.bank_id, dec!(100.00))
        .await
        .expect("admise");
    assert_eq!(locked.invoice.id, ok);
    assert_eq!(locked.invoice.status, "open");

    assert!(matches!(
        refus_301b(&pool, &ctx, 999_999, ctx.bank_id, dec!(100.00)).await,
        Err(SupplierSettlementRefusal::NotFound)
    ));
    // La facture `ok`, demandée au nom d'une autre société : indiscernable
    // d'une facture absente.
    let other_company = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 3', 'Pme', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut other_tx = pool.begin().await.unwrap();
    assert!(matches!(
        supplier_invoices::lock_for_bank_settlement_in_tx(
            &mut other_tx,
            other_company,
            ok,
            ctx.bank_id,
            dec!(100.00),
        )
        .await
        .unwrap(),
        Err(SupplierSettlementRefusal::NotFound)
    ));
    other_tx.rollback().await.unwrap();

    let paid = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(100.00)).await;
    supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        paid,
        SettlementChoice::BankTransfer {
            bank_account_id: ctx.bank_id,
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    assert!(matches!(
        refus_301b(&pool, &ctx, paid, ctx.bank_id, dec!(100.00)).await,
        Err(SupplierSettlementRefusal::NotOpen { status }) if status == "paid"
    ));

    assert!(matches!(
        refus_301b(&pool, &ctx, ok, 999_999, dec!(100.00)).await,
        Err(SupplierSettlementRefusal::BankAccountNotFound)
    ));
    let set_bank = |sql: &'static str| {
        let pool = pool.clone();
        let id = ctx.bank_id;
        async move {
            sqlx::query(sql).bind(id).execute(&pool).await.unwrap();
        }
    };
    set_bank("UPDATE bank_accounts SET journal_account_id = NULL WHERE id = ?").await;
    assert!(matches!(
        refus_301b(&pool, &ctx, ok, ctx.bank_id, dec!(100.00)).await,
        Err(SupplierSettlementRefusal::BankAccountNotConfigured)
    ));
    set_bank("UPDATE bank_accounts SET is_primary = FALSE, closed_on = '2026-06-30' WHERE id = ?")
        .await;
    assert!(matches!(
        refus_301b(&pool, &ctx, ok, ctx.bank_id, dec!(100.00)).await,
        Err(SupplierSettlementRefusal::BankAccountClosed)
    ));
    set_bank("UPDATE bank_accounts SET archived = TRUE WHERE id = ?").await;
    assert!(matches!(
        refus_301b(&pool, &ctx, ok, ctx.bank_id, dec!(100.00)).await,
        Err(SupplierSettlementRefusal::BankAccountNotFound)
    ));
    sqlx::query(
        "UPDATE bank_accounts SET archived = FALSE, closed_on = NULL, journal_account_id = ? \
         WHERE id = ?",
    )
    .bind(ctx.seeded.accounts["1100"])
    .bind(ctx.bank_id)
    .execute(&pool)
    .await
    .unwrap();

    // En lot `generated` : refusée, quel que soit le compte du lot.
    let held = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(80.00)).await;
    let batch =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![held]), ctx.seeded.admin_user_id)
            .await
            .unwrap()
            .batch
            .unwrap()
            .batch
            .id;
    assert!(matches!(
        refus_301b(&pool, &ctx, held, ctx.bank_id, dec!(80.00)).await,
        Err(SupplierSettlementRefusal::InGeneratedBatch { payment_batch_id }) if payment_batch_id == batch
    ));
    // (d) — une AUTRE facture au même montant, ouverte, pour le débit d'un
    // montant qu'une facture du lot du compte porte : refusée.
    let other_same = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(80.00)).await;
    assert!(matches!(
        refus_301b(&pool, &ctx, other_same, ctx.bank_id, dec!(80.00)).await,
        Err(SupplierSettlementRefusal::HeldByPaymentBatch { supplier_invoice_id, payment_batch_id })
            if supplier_invoice_id == held && payment_batch_id == batch
    ));
    // Le même débit sur un autre compte n'est pas retenu par ce lot.
    let other_bank = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: ctx.seeded.company_id,
            bank_name: "Autre banque".into(),
            iban: "CH4431999123000889012".into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap()
    .id;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ctx.seeded.accounts["1100"])
        .bind(other_bank)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        refus_301b(&pool, &ctx, other_same, other_bank, dec!(80.00))
            .await
            .is_ok(),
        "lot d'un autre compte : admise"
    );
}

/// T-b22 (AC-b8) — **la garde du lot est verrouillante**. La vue de la
/// transaction d'acceptation est née (une lecture ordinaire) ; un
/// `create_batch` de la facture est validé sur une autre connexion ; puis
/// l'acceptation verrouille la facture : elle doit voir le lot (refus
/// `InGeneratedBatch`) et ne rien écrire. Rouge avec une lecture ordinaire
/// des articles (M-b8) : la vue, née avant, ne voit pas le lot.
#[sqlx::test(migrations = "./test-schema")]
async fn the_payment_batch_guard_reads_past_the_view(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(150.00)).await;

    let mut acceptance = pool.begin().await.unwrap();
    let _: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM payment_batch_items")
        .fetch_one(&mut *acceptance)
        .await
        .unwrap();
    let batch =
        payment_batches::create_batch(&pool, new_batch(&ctx, vec![inv]), ctx.seeded.admin_user_id)
            .await
            .unwrap()
            .batch
            .expect("lot créé sur l'autre connexion")
            .batch
            .id;

    let r = supplier_invoices::lock_for_bank_settlement_in_tx(
        &mut acceptance,
        ctx.seeded.company_id,
        inv,
        ctx.bank_id,
        dec!(150.00),
    )
    .await
    .unwrap();
    assert!(
        matches!(r, Err(SupplierSettlementRefusal::InGeneratedBatch { payment_batch_id }) if payment_batch_id == batch),
        "⛔ le lot validé après la naissance de la vue doit être vu : {:?}",
        r.map(|l| l.invoice.id)
    );
    acceptance.rollback().await.unwrap();
}

/// T-b34 (AC-b10, C3) — **une acceptation et un `create_batch` sur le même
/// compte ne s'interbloquent pas**, au niveau du dépôt, sans enveloppe de
/// rejeu. Décor : `a` (l'acceptation) puis `b` (le lot), `b > a`, aucun article
/// existant — l'article de `b` tombe dans l'intervalle de queue de `a` ; le TTC
/// de `b` diffère du débit (sinon la garde (d) refuserait).
///
/// Chorégraphie forcée par sondes : (1) une troisième connexion tient le compte
/// `FOR UPDATE` ; (2) `create_batch(b)` tient `b` et se bloque sur la clé
/// étrangère du compte ; (3) l'acceptation de `a` se bloque sur le compte ;
/// (4) la troisième relâche. Le code prescrit n'a pas lu les articles au point
/// (3) : aucun `1213`, les deux aboutissent. Mutant M-b23 (articles lus avant
/// le compte) : l'acceptation tient l'intervalle au point (3), l'insertion de
/// l'article de `b` l'attend, et elle attend le compte que `b` tient — `1213`.
#[sqlx::test(migrations = "./test-schema")]
async fn an_acceptance_and_a_batch_creation_on_the_same_account_do_not_deadlock(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let a = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("A"), dec!(120.00)).await;
    let b = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("B"), dec!(70.00)).await;
    assert!(b > a);

    // (1) La troisième connexion tient le compte.
    let mut tiers = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM bank_accounts WHERE id = ? FOR UPDATE")
        .bind(ctx.bank_id)
        .fetch_one(&mut *tiers)
        .await
        .unwrap();

    // (2) `create_batch(b)` démarre et se bloque sur la clé étrangère du compte.
    let p = pool.clone();
    let nouveau = new_batch(&ctx, vec![b]);
    let user = ctx.seeded.admin_user_id;
    let lot = tokio::spawn(async move { payment_batches::create_batch(&p, nouveau, user).await });
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["INSERT INTO payment_batches"],
            || lot.is_finished(),
        )
        .await,
        "la création du lot doit attendre le compte"
    );

    // (3) L'acceptation de `a` démarre et se bloque sur le compte.
    let p = pool.clone();
    let company_id = ctx.seeded.company_id;
    let bank_id = ctx.bank_id;
    let acceptation = tokio::spawn(async move {
        let mut tx = p.begin().await.map_err(kesh_db::errors::map_db_error)?;
        let locked = supplier_invoices::lock_for_bank_settlement_in_tx(
            &mut tx,
            company_id,
            a,
            bank_id,
            dec!(120.00),
        )
        .await?;
        let locked = match locked {
            Ok(l) => l,
            Err(refus) => panic!("acceptation refusée : {refus:?}"),
        };
        supplier_invoices::pay_in_tx_as(
            &mut tx,
            company_id,
            locked.invoice.id,
            SettlementChoice::BankTransfer {
                bank_account_id: bank_id,
            },
            d(2026, 6, 25),
            kesh_db::repositories::letterings::Actor {
                user_id: user,
                api_key_id: None,
            },
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), DbError>(())
    });
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &[supplier_invoices::BANK_SETTLEMENT_ACCOUNT_LOCK_MARKER],
            || acceptation.is_finished(),
        )
        .await,
        "l'acceptation doit attendre le compte"
    );

    // (4) La troisième relâche.
    tiers.rollback().await.unwrap();

    let lot = lot.await.unwrap();
    let acceptation = acceptation.await.unwrap();
    assert!(
        lot.is_ok() && acceptation.is_ok(),
        "⛔ aucun interblocage attendu : lot {:?}, acceptation {:?}",
        lot.as_ref().map(|o| o.batch.is_some()),
        acceptation
    );
    assert!(lot.unwrap().batch.is_some(), "le lot de b est créé");
    let (inv, _) = supplier_invoices::get(&pool, ctx.seeded.company_id, a)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inv.status, "paid", "a est réglée");
}

/// T-b31 (AC-b16) — la confirmation d'un lot pose `paid_at` à la **date de
/// paiement** qu'elle reçoit (minuit), non à l'heure de la confirmation.
#[sqlx::test(migrations = "./test-schema")]
async fn confirming_a_batch_sets_paid_at_to_the_payment_date(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let inv = make_invoice(&pool, &ctx, Some(IBAN_A), None, Some("R"), dec!(64.00)).await;
    let batch =
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
        batch,
        d(2026, 7, 2),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let (paid, _) = supplier_invoices::get(&pool, ctx.seeded.company_id, inv)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        paid.paid_at,
        Some(d(2026, 7, 2).and_hms_opt(0, 0, 0).unwrap()),
        "paid_at = la date de paiement du lot, à minuit"
    );
}
