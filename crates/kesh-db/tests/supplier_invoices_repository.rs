//! Tests d'intégration des factures fournisseurs & règlement binaire (Story 12.2, #191).
//!
//! Vérifie bout-en-bout (DB éphémère) que :
//! - `create` poste l'écriture d'achat (D charge / D 1171 / C 2000), total = TTC, statut `open` ;
//! - `pay` (compte interne / virement) poste D 2000 / C contrepartie → **solde 2000 = 0**, statut `paid` ;
//! - `cancel` contre-passe l'écriture d'achat, statut `cancelled` ;
//! - les cas interdits sont refusés (config absente, déjà payée/annulée, compte étranger, non-fournisseur).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{
    NewBankAccount, NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{bank_accounts, contacts, supplier_invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Contexte de test : compte 1171 dédié, settings (payable=2000, recoverable=1171),
/// fournisseur, et le compte de charge 4000.
struct Ctx {
    seeded: SeededCompany,
    supplier_id: i64,
    /// id du compte 1171 (impôt préalable / TVA récupérable).
    recoverable_id: i64,
}

async fn setup(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.unwrap();

    // Compte 1171 dédié (impôt préalable, Asset) — distinct de 2000 pour isoler le solde créanciers.
    let recoverable_id = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, '1171', 'Impôt préalable', 'Asset')",
    )
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    // Config : créanciers 2000 + TVA récupérable 1171 (le seed a posé recoverable=2000, on corrige).
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
            default_payment_terms: Some("30".into()),
        },
    )
    .await
    .expect("create supplier")
    .id;

    Ctx {
        seeded,
        supplier_id,
        recoverable_id,
    }
}

fn one_line(ctx: &Ctx, unit_price: Decimal, vat_rate: Decimal) -> NewSupplierInvoice {
    NewSupplierInvoice {
        company_id: ctx.seeded.company_id,
        contact_id: ctx.supplier_id,
        supplier_invoice_number: Some("FF-2026-001".into()),
        invoice_date: d(2026, 6, 15),
        due_date: Some(d(2026, 7, 15)),
        creditor_iban: None,
        creditor_qr_iban: None,
        payment_reference: None,
        expected_payment_amount: None,
        project_id: None,
        lines: vec![NewSupplierInvoiceLine {
            description: "Prestation".into(),
            quantity: dec!(1),
            unit_price,
            vat_rate,
            expense_account_id: ctx.seeded.accounts["4000"],
        }],
    }
}

/// Solde net d'un compte (Σ débit - Σ crédit) sur toutes les écritures de la company.
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
async fn create_posts_purchase_entry_open(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(1000.00), dec!(8.10)),
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("create supplier invoice");

    assert_eq!(created.invoice.status, "open");
    // TTC = 1000 + 81 = 1081.
    assert_eq!(created.invoice.total_amount, dec!(1081.00));
    assert!(created.invoice.purchase_journal_entry_id > 0);

    // D 4000=1000, D 1171=81, C 2000=1081.
    let charge = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["4000"]).await;
    let vat = account_balance(&pool, ctx.seeded.company_id, ctx.recoverable_id).await;
    let payable = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["2000"]).await;
    assert_eq!(charge, dec!(1000.00));
    assert_eq!(vat, dec!(81.00));
    assert_eq!(payable, dec!(-1081.00)); // crédité.
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pay_internal_account_settles_payable_to_zero(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(1000.00), dec!(8.10)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();

    let paid = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("pay internal");

    assert_eq!(paid.invoice.status, "paid");
    assert_eq!(
        paid.invoice.settlement_type.as_deref(),
        Some("internal_account")
    );
    assert!(paid.invoice.paid_at.is_some());

    // Solde créanciers 2000 = 0 (C 1081 à l'achat, D 1081 au règlement).
    let payable = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["2000"]).await;
    assert_eq!(payable, dec!(0.00));
    // Caisse 1000 créditée de 1081.
    let cash = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["1000"]).await;
    assert_eq!(cash, dec!(-1081.00));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pay_bank_transfer_uses_journal_account(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    // Compte bancaire lié au compte grand livre 1100.
    let bank = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: ctx.seeded.company_id,
            bank_name: "Banque Test".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ctx.seeded.accounts["1100"])
        .bind(bank.id)
        .execute(&pool)
        .await
        .unwrap();

    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(500.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();

    let paid = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::BankTransfer {
            bank_account_id: bank.id,
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("pay bank transfer");

    assert_eq!(paid.invoice.status, "paid");
    assert_eq!(
        paid.invoice.settlement_type.as_deref(),
        Some("bank_transfer")
    );
    assert_eq!(paid.invoice.settlement_bank_account_id, Some(bank.id));

    // Solde 2000 = 0 ; banque 1100 créditée de 500 (taux 0 → pas de TVA).
    let payable = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["2000"]).await;
    assert_eq!(payable, dec!(0.00));
    let bank_bal = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["1100"]).await;
    assert_eq!(bank_bal, dec!(-500.00));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cancel_reverses_purchase_entry(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(1000.00), dec!(8.10)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();

    let cancelled = supplier_invoices::cancel(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("cancel");
    assert_eq!(cancelled.invoice.status, "cancelled");

    // Après contre-passation, tous les comptes touchés reviennent à 0.
    let charge = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["4000"]).await;
    let vat = account_balance(&pool, ctx.seeded.company_id, ctx.recoverable_id).await;
    let payable = account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["2000"]).await;
    assert_eq!(charge, dec!(0.00));
    assert_eq!(vat, dec!(0.00));
    assert_eq!(payable, dec!(0.00));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_without_payable_account_fails_config(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    // Retirer le compte créanciers configuré.
    sqlx::query("UPDATE company_invoice_settings SET default_payable_account_id = NULL WHERE company_id = ?")
        .bind(ctx.seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(1000.00), dec!(8.10)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::ConfigurationRequired(_)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pay_cancelled_invoice_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    supplier_invoices::cancel(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();

    let err = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cancel_paid_invoice_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();

    let err = supplier_invoices::cancel(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_non_supplier_contact_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    // Contact client (is_supplier=false).
    let client_id = contacts::create(
        &pool,
        ctx.seeded.admin_user_id,
        NewContact {
            company_id: ctx.seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client SA".into(),
            is_client: true,
            is_supplier: false,
            address: None,
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: None,
        },
    )
    .await
    .unwrap()
    .id;

    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    new.contact_id = client_id;
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pay_foreign_internal_account_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();

    // Compte appartenant à une AUTRE company (company minimale, sans user pour
    // éviter le conflit uq_users_username global).
    let other_company_id = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Ailleurs\n1200 Genève', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let foreign_account_id = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, '1000', 'Caisse autre', 'Asset')",
    )
    .bind(other_company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let err = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: foreign_account_id,
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pay_already_paid_invoice_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let internal = SettlementChoice::InternalAccount {
        account_id: ctx.seeded.accounts["1000"],
    };
    supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        internal,
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    // Double paiement refusé.
    let err = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        internal,
        d(2026, 6, 21),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_date_outside_fiscal_year_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    new.invoice_date = d(2099, 1, 1); // hors de tout exercice ouvert.
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::FiscalYearInvalid));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pay_with_date_outside_fiscal_year_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let err = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2099, 1, 1),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::FiscalYearInvalid));
}

/// Helper Story 19-3 : insère un projet analytique et retourne son id.
async fn make_project(pool: &MySqlPool, company_id: i64, code: &str, archived: bool) -> i64 {
    sqlx::query(
        "INSERT INTO projects (company_id, code, name, archived, version) VALUES (?, ?, ?, ?, 0)",
    )
    .bind(company_id)
    .bind(code)
    .bind(code)
    .bind(archived)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Story 19-3 — le projet document-level est propagé sur TOUTES les lignes de
/// l'écriture d'achat.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_project_tags_all_purchase_lines(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let project_id = make_project(&pool, ctx.seeded.company_id, "RENOV", false).await;

    let mut new = one_line(&ctx, dec!(100), dec!(8.1));
    new.project_id = Some(project_id);
    let created = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap();
    assert_eq!(created.invoice.project_id, Some(project_id));

    // Toutes les lignes de l'écriture d'achat portent le projet.
    let tags: Vec<Option<i64>> =
        sqlx::query_scalar("SELECT project_id FROM journal_entry_lines WHERE entry_id = ?")
            .bind(created.invoice.purchase_journal_entry_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(!tags.is_empty());
    assert!(
        tags.iter().all(|t| *t == Some(project_id)),
        "toutes les lignes d'achat doivent porter project_id={project_id}, got {tags:?}"
    );
}

/// Story 19-3 — un projet archivé est refusé (dépense sur projet clos interdite).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_archived_project_is_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let project_id = make_project(&pool, ctx.seeded.company_id, "OLD", true).await;

    let mut new = one_line(&ctx, dec!(100), dec!(8.1));
    new.project_id = Some(project_id);
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(
        matches!(err, kesh_db::errors::DbError::IllegalStateTransition(_)),
        "projet archivé → IllegalStateTransition, got {err:?}"
    );
}

/// Story 19-3 — le règlement hérite du projet de la facture (toutes les lignes de
/// l'écriture de règlement portent project_id).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pay_with_project_tags_settlement_entry(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let project_id = make_project(&pool, ctx.seeded.company_id, "RENOV", false).await;
    let mut new = one_line(&ctx, dec!(1000.00), dec!(8.10));
    new.project_id = Some(project_id);
    let created = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap();

    let paid = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("pay");
    let settlement_je = paid
        .invoice
        .settlement_journal_entry_id
        .expect("settlement je");

    let tags: Vec<Option<i64>> =
        sqlx::query_scalar("SELECT project_id FROM journal_entry_lines WHERE entry_id = ?")
            .bind(settlement_je)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(!tags.is_empty());
    assert!(
        tags.iter().all(|t| *t == Some(project_id)),
        "settlement: {tags:?}"
    );
}

/// Story 19-3 — après annulation, le net (Σ débit − Σ crédit) des lignes taguées du
/// projet est **nul** (la contre-passation reprend le projet).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cancel_nets_project_to_zero(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let project_id = make_project(&pool, ctx.seeded.company_id, "RENOV", false).await;
    let mut new = one_line(&ctx, dec!(1000.00), dec!(8.10));
    new.project_id = Some(project_id);
    let created = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap();

    supplier_invoices::cancel(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("cancel");

    // Net par projet = 0 (achat + contre-passation s'annulent).
    let net: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM journal_entry_lines WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        net,
        dec!(0.00),
        "net par projet doit être nul après annulation"
    );
}

/// Story 19-3 — un projet inexistant (ou d'une autre company) est rejeté (scoping
/// `find_by_id_in_tx(tx, company_id, pid)` → None → NotFound).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_unknown_project_is_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let mut new = one_line(&ctx, dec!(100), dec!(8.1));
    new.project_id = Some(999_999); // n'existe pas / hors company
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(
        matches!(err, kesh_db::errors::DbError::NotFound),
        "got {err:?}"
    );
}
