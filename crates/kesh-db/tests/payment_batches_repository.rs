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
