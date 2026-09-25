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
            default_payment_terms: Some("30".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

/// ⛔ **Un compte de charge du MAUVAIS TYPE est refusé à la création.**
///
/// Jumeau du test ci-dessous, sur l'autre moitié du même `match`. Le commentaire
/// de la garde affirme « de type Expense (AC6 — sinon l'écriture débiterait un
/// Passif/Actif) » : sans ce test, cette moitié-là n'était tenue par **rien**.
///
/// ⛔ Relevé en passe 5 de revue de code de la Story 24-5 (#375), finding P5-7 —
/// par mutation : remplacer `Some((true, true, ref t)) if t == "Expense"` par
/// `Some((true, true, _))` laissait les 20 tests du binaire au vert. Défaut
/// préexistant, mais la passe 4 avait réécrit ce `match` et son commentaire sans
/// voir que la moitié de ce qu'il affirme n'était exercée par personne.
#[sqlx::test(migrations = "./test-schema")]
async fn create_with_non_expense_account_is_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;

    // 1000 est un Asset, actif et postable : seul le TYPE doit le faire refuser.
    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    new.lines[0].expense_account_id = ctx.seeded.accounts["1000"];

    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::InactiveOrInvalidAccounts),
        "got {err:?}"
    );
}

/// ⛔ **Un compte de CHARGE non imputable est refusé à la création.**
///
/// Le type ne suffit pas : les comptes de clôture 9000/9100/9200 sont eux-mêmes
/// typés `Expense` — c'est le fait fondateur de la Story 24-5 — donc la garde
/// `t == "Expense"` les laissait passer. Ce flux écrit avec
/// `enforce_postable = false` : le `SELECT active, postable, account_type` est
/// le seul contrôle.
///
/// ⛔ Trouvé APRÈS la passe 4 de revue de code (#375), en refaisant l'énumération
/// exhaustive des appels à `journal_entries::create_in_tx` que cette passe avait
/// déclarée « rapide ». Cinquième chemin d'écriture de la story.
#[sqlx::test(migrations = "./test-schema")]
async fn create_with_non_postable_expense_account_is_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;

    // Le compte de charge de `one_line` reste ACTIF et de type Expense — c'est
    // `postable` seul qui doit refuser.
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(ctx.seeded.accounts["4000"])
        .execute(&pool)
        .await
        .unwrap();

    let err = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::InactiveOrInvalidAccounts),
        "got {err:?}"
    );
}

/// ⛔ **Un compte de contrepartie NON IMPUTABLE est refusé.**
///
/// Jumeau du test de `invoice_settlement.rs` : `pay` écrit sans la garde de
/// postabilité de la 14-3b, donc son `SELECT active, postable` est le seul
/// contrôle. Ici l'écran filtrait déjà `active && postable`, si bien que le trou
/// n'était atteignable que par appel direct à l'API — raison de plus pour que le
/// test existe : *rien d'autre n'en fait foi.*
///
/// ⛔ Trouvé par le grep de propagation du finding P3-1 (passe 3 de revue de code
/// de la Story 24-5, #375), qui a rendu ce site à côté de son jumeau client.
#[sqlx::test(migrations = "./test-schema")]
async fn pay_with_non_postable_account_is_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();

    // Le compte reste ACTIF — c'est `postable` seul qui doit refuser.
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(ctx.seeded.accounts["1000"])
        .execute(&pool)
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
    assert!(
        matches!(err, DbError::InactiveOrInvalidAccounts),
        "got {err:?}"
    );
}

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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
    .unwrap()
    .id;

    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    new.contact_id = client_id;
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
async fn create_with_date_outside_fiscal_year_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    new.invoice_date = d(2099, 1, 1); // hors de tout exercice ouvert.
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::FiscalYearInvalid));
}

#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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

/// Story 19-3 — payer une facture dont le projet a été archivé APRÈS le tag reste
/// possible (la garde d'archivage n'est qu'à la création ; le règlement propage le tag).
#[sqlx::test(migrations = "./test-schema")]
async fn pay_succeeds_when_project_archived_after_tagging(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let project_id = make_project(&pool, ctx.seeded.company_id, "RENOV", false).await;
    let mut new = one_line(&ctx, dec!(500.00), dec!(8.10));
    new.project_id = Some(project_id);
    let created = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap();

    // Archiver le projet après coup.
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(project_id)
        .execute(&pool)
        .await
        .unwrap();

    // Le paiement doit réussir et propager le projet au règlement.
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
    .expect("pay doit réussir même si le projet est archivé");
    let settlement_je = paid.invoice.settlement_journal_entry_id.unwrap();
    let tags: Vec<Option<i64>> =
        sqlx::query_scalar("SELECT project_id FROM journal_entry_lines WHERE entry_id = ?")
            .bind(settlement_je)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(tags.iter().all(|t| *t == Some(project_id)));
}

// ===========================================================================
// Story 25-3-a-2 (#414) — annuler un règlement fournisseur
// ===========================================================================

use kesh_db::errors::{ReversalBlocker, SettlementCancelBlocker};
use kesh_db::repositories::journal_entries::{self, ReversalAuthority};
use kesh_db::repositories::{accounts, fiscal_years};

/// Facture créée à `inv_date` puis réglée en espèces (1000) à `paid_on` ; rend
/// `(id, écriture d'achat, écriture de règlement)`.
async fn paid_invoice(
    pool: &MySqlPool,
    ctx: &Ctx,
    inv_date: NaiveDate,
    paid_on: NaiveDate,
) -> (i64, i64, i64) {
    let mut new = one_line(ctx, dec!(100.00), dec!(0));
    new.invoice_date = inv_date;
    new.due_date = Some(inv_date);
    let created = supplier_invoices::create(pool, new, ctx.seeded.admin_user_id)
        .await
        .expect("création");
    let paid = supplier_invoices::pay(
        pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        paid_on,
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("règlement");
    (
        created.invoice.id,
        created.invoice.purchase_journal_entry_id,
        paid.invoice
            .settlement_journal_entry_id
            .expect("écriture de règlement"),
    )
}

async fn count(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.expect(sql)
}

/// ⛔ **Le geste nominal** : `paid → open`, colonnes de règlement vidées,
/// contre-passation liée, créanciers rétablis, deux lignes d'audit.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_settlement_reopens_and_clears_the_columns(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (id, _, entry) = paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;

    let done = supplier_invoices::cancel_settlement(
        &pool,
        ctx.seeded.company_id,
        id,
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("annulation");

    let inv = &done.invoice.invoice;
    assert_eq!(inv.status, "open");
    assert!(inv.settlement_type.is_none());
    assert!(inv.settlement_bank_account_id.is_none());
    assert!(inv.settlement_account_id.is_none());
    assert!(inv.settlement_journal_entry_id.is_none());
    assert!(inv.paid_at.is_none());
    let reverses: Option<i64> =
        sqlx::query_scalar("SELECT reverses_entry_id FROM journal_entries WHERE id = ?")
            .bind(done.reversal_journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reverses, Some(entry), "une vraie contre-passation, liée");
    assert_eq!(
        account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["2000"]).await,
        dec!(-100.00),
        "la dette envers le fournisseur renaît"
    );
    assert_eq!(
        account_balance(&pool, ctx.seeded.company_id, ctx.seeded.accounts["1000"]).await,
        dec!(0.00),
        "la caisse est rendue"
    );
    assert_eq!(
        count(
            &pool,
            &format!(
                "SELECT COUNT(*) FROM audit_log WHERE action = 'supplier_invoice.settlement_cancelled' \
                 AND entity_id = {id}"
            )
        )
        .await,
        1
    );
}

/// ⛔ **Deux cycles complets** : payer, annuler, payer, annuler. La première
/// écriture de règlement, sortie de la colonne, ressort en `ALREADY_REVERSED`
/// et ne gêne pas le second cycle.
#[sqlx::test(migrations = "./test-schema")]
async fn two_full_cycles_pay_cancel_pay_cancel(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (id, _, first) = paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;
    let (c, u) = (ctx.seeded.company_id, ctx.seeded.admin_user_id);
    supplier_invoices::cancel_settlement(&pool, c, id, u)
        .await
        .expect("première annulation");
    let repaid = supplier_invoices::pay(
        &pool,
        c,
        id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 6, 25),
        u,
    )
    .await
    .expect("second règlement");
    let second = repaid.invoice.settlement_journal_entry_id.unwrap();
    assert_ne!(first, second);
    supplier_invoices::cancel_settlement(&pool, c, id, u)
        .await
        .expect("seconde annulation");

    let blocker = journal_entries::reversal_blocker(&pool, c, first)
        .await
        .unwrap();
    assert_eq!(blocker.map(|h| h.0), Some(ReversalBlocker::AlreadyReversed));
}

/// Rang 1 : une facture non payée — et la seconde annulation du même règlement.
#[sqlx::test(migrations = "./test-schema")]
async fn not_paid_is_refused_with_its_code(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (c, u) = (ctx.seeded.company_id, ctx.seeded.admin_user_id);
    let open = supplier_invoices::create(&pool, one_line(&ctx, dec!(50.00), dec!(0)), u)
        .await
        .unwrap()
        .invoice
        .id;
    let err = supplier_invoices::cancel_settlement(&pool, c, open, u)
        .await
        .expect_err("facture ouverte");
    assert!(
        matches!(
            err,
            DbError::SettlementNotCancellable {
                blocker: SettlementCancelBlocker::SupplierInvoiceNotPaid
            }
        ),
        "got {err:?}"
    );

    let (id, _, _) = paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;
    supplier_invoices::cancel_settlement(&pool, c, id, u)
        .await
        .unwrap();
    let err = supplier_invoices::cancel_settlement(&pool, c, id, u)
        .await
        .expect_err("double annulation");
    assert!(
        matches!(
            err,
            DbError::SettlementNotCancellable {
                blocker: SettlementCancelBlocker::SupplierInvoiceNotPaid
            }
        ),
        "got {err:?}"
    );
}

/// ⛔ **Le socle — l'autorité fournisseur ne couvre QUE l'écriture de
/// règlement.** Appel DIRECT de la variante sur l'écriture d'ACHAT de la même
/// facture : refusé en `OWNED_BY_SUPPLIER_INVOICE`, comme sans autorité. Le
/// témoin positif — la même autorité sur l'écriture de règlement — passe.
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_authority_never_covers_the_purchase_entry(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (id, purchase, settlement) =
        paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;
    let authority = ReversalAuthority::SupplierSettlement {
        supplier_invoice_id: id,
    };

    let mut tx = pool.begin().await.unwrap();
    let err = journal_entries::reverse_owned_in_tx(
        &mut tx,
        ctx.seeded.company_id,
        purchase,
        ctx.seeded.admin_user_id,
        authority,
    )
    .await
    .expect_err("l'écriture d'achat ne se contre-passe pas au titre du règlement");
    assert!(
        matches!(
            err,
            DbError::EntryNotReversable {
                blocker: ReversalBlocker::OwnedBySupplierInvoice,
                ..
            }
        ),
        "got {err:?}"
    );
    drop(tx);

    let mut tx = pool.begin().await.unwrap();
    journal_entries::reverse_owned_in_tx(
        &mut tx,
        ctx.seeded.company_id,
        settlement,
        ctx.seeded.admin_user_id,
        authority,
    )
    .await
    .expect("témoin : l'écriture de règlement, elle, se contre-passe");
}

/// ⛔ **Composition et rollback** : dans une transaction fournie, le geste
/// écrit (lecture positive), puis l'abandon efface tout.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_settlement_composes_and_rolls_back(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (id, _, entry) = paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;
    let avant = count(&pool, "SELECT COUNT(*) FROM journal_entries").await;
    {
        let mut tx = pool.begin().await.unwrap();
        supplier_invoices::cancel_settlement_in_tx(
            &mut tx,
            ctx.seeded.company_id,
            id,
            ctx.seeded.admin_user_id,
        )
        .await
        .expect("annulation dans la transaction");
        let inside: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?")
                .bind(entry)
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert_eq!(inside, 1, "DANS la transaction, la contre-passation existe");
        let status: String =
            sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert_eq!(
            status, "open",
            "DANS la transaction, la facture est rouverte"
        );
    }
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM journal_entries").await,
        avant
    );
    let status: String = sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "paid", "rien n'a survécu à l'abandon");
}

/// ⛔ **Étanchéité multi-tenant** : table par table, rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn another_company_cannot_cancel_the_settlement(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (id, _, _) = paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;
    let other = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let tables = [
        "journal_entries",
        "journal_entry_lines",
        "supplier_invoices",
        "audit_log",
    ];
    let mut avant = Vec::new();
    for t in tables {
        avant.push(count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await);
    }
    let err = supplier_invoices::cancel_settlement(&pool, other, id, ctx.seeded.admin_user_id)
        .await
        .expect_err("autre société");
    assert!(matches!(err, DbError::NotFound), "got {err:?}");
    let mut conn = pool.acquire().await.unwrap();
    let err = supplier_invoices::supplier_settlement_cancel_blocker(&mut conn, other, id)
        .await
        .expect_err("autre société — lecture");
    assert!(matches!(err, DbError::NotFound), "got {err:?}");
    for (t, n) in tables.iter().zip(avant) {
        assert_eq!(
            count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await,
            n,
            "rien d'écrit dans {t}"
        );
    }
}

/// Monte un règlement fournisseur dans un exercice raccourci à
/// `D = aujourd'hui − 60 j`, avec ou sans exercice couvrant le jour, compte de
/// caisse archivé et exercice du règlement clos selon `motifs`. Même calendrier
/// que le montage client de la 25-3-a-1 (raccourcissement en SQL, dit tel).
async fn monter(pool: &MySqlPool, motifs: &[SettlementCancelBlocker]) -> (Ctx, i64) {
    kesh_db::test_fixtures::truncate_all(pool)
        .await
        .expect("truncate");
    let ctx = setup(pool).await;
    let today = chrono::Utc::now().date_naive();
    let dd = today - chrono::Duration::days(60);
    sqlx::query("UPDATE fiscal_years SET end_date = ? WHERE id = ?")
        .bind(dd)
        .bind(ctx.seeded.fiscal_year_id)
        .execute(pool)
        .await
        .expect("raccourcir l'exercice");
    if !motifs.contains(&SettlementCancelBlocker::NoOpenFiscalYearToday) {
        fiscal_years::create(
            pool,
            ctx.seeded.admin_user_id,
            kesh_db::entities::NewFiscalYear {
                company_id: ctx.seeded.company_id,
                name: "Courant".into(),
                start_date: dd + chrono::Duration::days(1),
                end_date: d(2030, 12, 31),
            },
        )
        .await
        .expect("exercice courant");
    }
    let (id, _, _) = paid_invoice(
        pool,
        &ctx,
        dd - chrono::Duration::days(20),
        dd - chrono::Duration::days(10),
    )
    .await;
    if motifs.contains(&SettlementCancelBlocker::AccountArchived) {
        let caisse = ctx.seeded.accounts["1000"];
        let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
            .bind(caisse)
            .fetch_one(pool)
            .await
            .unwrap();
        accounts::archive(pool, caisse, version, ctx.seeded.admin_user_id)
            .await
            .expect("archivage de la caisse");
    }
    if motifs.contains(&SettlementCancelBlocker::FiscalYearClosed) {
        fiscal_years::close(
            pool,
            ctx.seeded.admin_user_id,
            ctx.seeded.company_id,
            ctx.seeded.fiscal_year_id,
        )
        .await
        .expect("clôture");
    }
    (ctx, id)
}

/// ⛔ **La queue commune, par le chemin fournisseur** : chaque motif atteignable
/// seul, et la paire compte archivé + exercice du jour absent — la lecture
/// annonce le motif, l'écriture refuse pour CE MÊME motif.
#[sqlx::test(migrations = "./test-schema")]
async fn tail_motives_through_the_supplier_path(pool: MySqlPool) {
    use SettlementCancelBlocker::*;
    let cas: [(&[SettlementCancelBlocker], SettlementCancelBlocker); 4] = [
        (&[FiscalYearClosed], FiscalYearClosed),
        (&[AccountArchived], AccountArchived),
        (&[NoOpenFiscalYearToday], NoOpenFiscalYearToday),
        (&[AccountArchived, NoOpenFiscalYearToday], AccountArchived),
    ];
    for (motifs, attendu) in cas {
        let (ctx, id) = monter(&pool, motifs).await;
        let mut conn = pool.acquire().await.unwrap();
        let lu = supplier_invoices::supplier_settlement_cancel_blocker(
            &mut conn,
            ctx.seeded.company_id,
            id,
        )
        .await
        .expect("lecture");
        drop(conn);
        assert_eq!(
            lu.as_ref().map(|h| h.0),
            Some(attendu),
            "lecture, motifs {motifs:?}"
        );
        if attendu == AccountArchived {
            assert_eq!(lu.and_then(|h| h.2).as_deref(), Some("1000"));
        }
        let err = supplier_invoices::cancel_settlement(
            &pool,
            ctx.seeded.company_id,
            id,
            ctx.seeded.admin_user_id,
        )
        .await
        .expect_err("refusée");
        let ok = match attendu {
            FiscalYearClosed => matches!(
                err,
                DbError::SettlementNotCancellable {
                    blocker: FiscalYearClosed
                }
            ),
            AccountArchived => matches!(err, DbError::ReversalAccountsArchived(_)),
            NoOpenFiscalYearToday => matches!(err, DbError::FiscalYearInvalid),
            _ => false,
        };
        assert!(ok, "écriture, motifs {motifs:?} : reçu {err:?}");
    }
}

/// ⛔ **Une clôture concurrente attend l'annulation** — même garde que la
/// 25-3-a-1 (étape 1-bis ici), même preuve par entrelacement : la clôture n'est
/// validée qu'une fois l'annulation vue en attente d'un verrou sur l'exercice.
#[sqlx::test(migrations = "./test-schema")]
async fn a_concurrent_close_waits_for_the_cancellation(pool: MySqlPool) {
    let (ctx, id) = monter(&pool, &[]).await;
    let mut closing = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.seeded.fiscal_year_id)
        .execute(&mut *closing)
        .await
        .unwrap();
    let p = pool.clone();
    let (c, u) = (ctx.seeded.company_id, ctx.seeded.admin_user_id);
    let annulation =
        tokio::spawn(async move { supplier_invoices::cancel_settlement(&p, c, id, u).await });
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["fiscal_years", "FOR UPDATE"],
        || annulation.is_finished(),
    )
    .await;
    if !vue {
        panic!(
            "l'annulation a fini sans attendre de verrou : {:?}",
            annulation.await.map(|r| r.map(|_| ()))
        );
    }
    closing.commit().await.unwrap();
    let result = annulation.await.expect("tâche").map(|_| ());
    assert!(
        matches!(
            result,
            Err(DbError::SettlementNotCancellable {
                blocker: SettlementCancelBlocker::FiscalYearClosed
            })
        ),
        "l'annulation devait attendre la clôture puis refuser — reçu {result:?}"
    );
}
