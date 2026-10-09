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

#[path = "support/document_group.rs"]
mod document_group;
use document_group::{lignes_du_groupe, poser_groupe_document};

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

    // Story 25-3-c (#454) : une VRAIE contre-passation, liée à l'achat — que
    // le grand livre présente désormais comme déjà contre-passée.
    let purchase = created.invoice.purchase_journal_entry_id;
    let reverses: Option<i64> = sqlx::query_scalar(
        "SELECT reverses_entry_id FROM journal_entries \
         WHERE company_id = ? AND reverses_entry_id IS NOT NULL",
    )
    .bind(ctx.seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(reverses, Some(purchase));
    let blocker = kesh_db::repositories::journal_entries::reversal_blocker(
        &pool,
        ctx.seeded.company_id,
        purchase,
    )
    .await
    .unwrap();
    assert_eq!(
        blocker.map(|h| h.0),
        Some(kesh_db::errors::ReversalBlocker::AlreadyReversed)
    );
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
    // Story 15-5a (#429) — réécrit à dessein : le refus porte son vrai nom et
    // nomme le compte (auparavant `InactiveOrInvalidAccounts`).
    assert_eq!(
        named_non_postable(&err),
        vec![(ctx.seeded.accounts["4000"], "4000".to_string())],
        "got {err:?}"
    );
}

/// Story 15-5a — `(id, numéro)` des comptes nommés par un `AccountsNotPostable`,
/// ou panique sur toute autre erreur.
fn named_non_postable(err: &DbError) -> Vec<(i64, String)> {
    match err {
        DbError::AccountsNotPostable(list) => list
            .iter()
            .map(|a| (a.account_id, a.account_number.clone()))
            .collect(),
        other => panic!("attendu AccountsNotPostable, obtenu {other:?}"),
    }
}

/// Story 15-5a — un second compte de charge, actif et imputable.
async fn second_expense_account(pool: &MySqlPool, ctx: &Ctx, number: &str) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, 'Charge 2', 'Expense')",
    )
    .bind(ctx.seeded.company_id)
    .bind(number)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn set_flag(pool: &MySqlPool, account_id: i64, column: &str, value: bool) {
    sqlx::query(&format!("UPDATE accounts SET {column} = ? WHERE id = ?"))
        .bind(value)
        .bind(account_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Story 15-5a (AC4) — un compte de charge ARCHIVÉ reste
/// `InactiveOrInvalidAccounts` (cause (a)).
#[sqlx::test(migrations = "./test-schema")]
async fn create_with_archived_expense_account_is_inactive_or_invalid(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    set_flag(&pool, ctx.seeded.accounts["4000"], "active", false).await;
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

/// Story 15-5a (AC4, finding M4) — un compte d'ACTIF non imputable proposé
/// comme compte de charge cumule (a) mauvais type et (b) non imputable : la
/// cause (a) prime.
#[sqlx::test(migrations = "./test-schema")]
async fn create_with_non_postable_asset_as_expense_is_inactive_or_invalid(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    set_flag(&pool, ctx.seeded.accounts["1000"], "postable", false).await;
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

/// Story 15-5a (AC4, C22) — deux lignes sur deux comptes de charge non
/// imputables : UN seul refus les nomme tous deux, triés par numéro.
#[sqlx::test(migrations = "./test-schema")]
async fn create_with_two_non_postable_expense_lines_names_both(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let other = second_expense_account(&pool, &ctx, "4400").await;
    let first = ctx.seeded.accounts["4000"];
    set_flag(&pool, first, "postable", false).await;
    set_flag(&pool, other, "postable", false).await;

    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    let mut second = new.lines[0].clone();
    second.expense_account_id = other;
    // Ordre des lignes inverse de l'ordre des numéros : le refus trie.
    new.lines.insert(0, second);
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert_eq!(
        named_non_postable(&err),
        vec![(first, "4000".to_string()), (other, "4400".to_string())]
    );
}

/// Story 15-5a (C22) — ordre des passes : la FORME de toutes les lignes est
/// jugée avant les COMPTES. Ligne 1 au compte non imputable, ligne 2 de
/// quantité nulle → refus de forme.
#[sqlx::test(migrations = "./test-schema")]
async fn create_form_of_later_line_wins_over_non_postable_account(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    set_flag(&pool, ctx.seeded.accounts["4000"], "postable", false).await;
    let other = second_expense_account(&pool, &ctx, "4400").await;

    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    let mut bad = new.lines[0].clone();
    bad.expense_account_id = other;
    bad.quantity = dec!(0);
    new.lines.push(bad);
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::IllegalStateTransition(_)),
        "got {err:?}"
    );
}

/// Story 15-5a (C22) — ligne 1 au compte ARCHIVÉ, ligne 2 de prix négatif →
/// refus de forme (seul changement d'ordre observable de la story).
#[sqlx::test(migrations = "./test-schema")]
async fn create_form_of_later_line_wins_over_archived_account(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    set_flag(&pool, ctx.seeded.accounts["4000"], "active", false).await;
    let other = second_expense_account(&pool, &ctx, "4400").await;

    let mut new = one_line(&ctx, dec!(100.00), dec!(0));
    let mut bad = new.lines[0].clone();
    bad.expense_account_id = other;
    bad.unit_price = dec!(-5);
    new.lines.push(bad);
    let err = supplier_invoices::create(&pool, new, ctx.seeded.admin_user_id)
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::IllegalStateTransition(_)),
        "got {err:?}"
    );
}

/// Story 15-5a (AC4) — règlement fournisseur sur un compte interne ARCHIVÉ :
/// `InactiveOrInvalidAccounts` (cause (a)).
#[sqlx::test(migrations = "./test-schema")]
async fn pay_with_archived_account_is_inactive_or_invalid(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(100.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    set_flag(&pool, ctx.seeded.accounts["1000"], "active", false).await;
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
    // Story 15-5a (#429) — réécrit à dessein : `AccountsNotPostable` nommant le
    // compte (auparavant `InactiveOrInvalidAccounts`).
    assert_eq!(
        named_non_postable(&err),
        vec![(ctx.seeded.accounts["1000"], "1000".to_string())],
        "got {err:?}"
    );
}

/// ⛔ **Story 25-3-c (#454) — une facture PAYÉE s'annule**, et son règlement en
/// est DÉTACHÉ (arbitrage du 2026-09-26) : il reste au grand livre, sans
/// contre-passation, sans propriétaire — un paiement sans facture. Ce test
/// posait l'ancien comportement (refus) ; il est réécrit, non supprimé.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_paid_invoice_detaches_its_settlement(pool: MySqlPool) {
    use kesh_db::errors::ReversalBlocker;
    use kesh_db::repositories::journal_entries;

    let ctx = setup(&pool).await;
    let (c, u) = (ctx.seeded.company_id, ctx.seeded.admin_user_id);
    let created = supplier_invoices::create(&pool, one_line(&ctx, dec!(100.00), dec!(8.10)), u)
        .await
        .unwrap();
    let paid = supplier_invoices::pay(
        &pool,
        c,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: ctx.seeded.accounts["1000"],
        },
        d(2026, 6, 20),
        u,
    )
    .await
    .unwrap();
    let settlement = paid.invoice.settlement_journal_entry_id.unwrap();

    let cancelled = supplier_invoices::cancel(&pool, c, created.invoice.id, u)
        .await
        .expect("une facture payée s'annule");
    let inv = &cancelled.invoice;
    assert_eq!(inv.status, "cancelled");
    assert!(inv.settlement_type.is_none());
    assert!(inv.settlement_bank_account_id.is_none());
    assert!(inv.settlement_account_id.is_none());
    assert!(
        inv.settlement_journal_entry_id.is_none(),
        "règlement détaché"
    );
    assert!(inv.paid_at.is_none());

    // L'écriture de règlement existe toujours, et n'est pas contre-passée.
    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = {settlement}")
        )
        .await,
        0
    );
    // Soldes : charge et TVA à zéro, 2000 débiteur du TTC, caisse créditée.
    let ttc = dec!(108.10);
    assert_eq!(
        account_balance(&pool, c, ctx.seeded.accounts["4000"]).await,
        dec!(0.00)
    );
    assert_eq!(
        account_balance(&pool, c, ctx.recoverable_id).await,
        dec!(0.00)
    );
    assert_eq!(
        account_balance(&pool, c, ctx.seeded.accounts["2000"]).await,
        ttc
    );
    assert_eq!(
        account_balance(&pool, c, ctx.seeded.accounts["1000"]).await,
        -ttc
    );

    // ⛔ La preuve du détachement : plus de propriétaire, et la
    // contre-passation à la main passe.
    let motifs = journal_entries::reversal_blockers(&pool, c, settlement)
        .await
        .unwrap();
    assert!(
        !motifs
            .iter()
            .any(|(b, _, _)| *b == ReversalBlocker::OwnedBySupplierInvoice),
        "le règlement n'appartient plus à la facture : {motifs:?}"
    );
    // ⛔ Story 15-8a (#532, C-15-8-20) — mais il ne se MODIFIE pas : c'est une
    // sortie de banque réelle. Plus aucune colonne ne le référence ; la garde
    // de modification le reconnaît à la trace d'audit de l'annulation (dette
    // #541). Chemin réel (create → pay → cancel), et non une trace posée à la
    // main : c'est aussi ce qui vérifie la comparaison `JSON_VALUE(...) = ?`.
    {
        use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
        use kesh_db::errors::ModificationGuard;
        let attendu = ModificationGuard::DetachedSupplierSettlement {
            supplier_invoice_id: created.invoice.id,
            supplier_invoice_number: Some("FF-2026-001".into()),
        };
        let mut conn = pool.acquire().await.unwrap();
        let garde = journal_entries::modification_guard(&mut conn, c, settlement)
            .await
            .unwrap();
        drop(conn);
        assert_eq!(garde, Some(attendu.clone()));

        let current = journal_entries::find_by_id(&pool, c, settlement)
            .await
            .unwrap()
            .unwrap();
        let corps = NewJournalEntry {
            company_id: c,
            entry_date: current.entry.entry_date,
            journal: current.entry.journal,
            description: "Tentative de réécriture".into(),
            project_id: None,
            lines: current
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                    project_id: l.project_id,
                })
                .collect(),
        };
        let err =
            journal_entries::update(&pool, c, settlement, current.entry.version, u, None, corps)
                .await
                .expect_err("le paiement détaché ne se modifie pas");
        assert!(
            matches!(&err, DbError::EntryNotModifiable(g) if *g == attendu),
            "got {err:?}"
        );
    }

    journal_entries::reverse(&pool, c, settlement, u)
        .await
        .expect("un paiement sans facture se contre-passe depuis sa fiche");

    // L'audit garde le lien que la colonne a perdu.
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.cancelled' \
         AND entity_id = ?",
    )
    .bind(created.invoice.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["previousStatus"], "paid");
    assert_eq!(details["settlementJournalEntryId"], settlement);
    assert_eq!(details["settlementType"], "internal_account");

    // L'annulation du règlement n'a plus d'objet.
    let err = supplier_invoices::cancel_settlement(&pool, c, created.invoice.id, u)
        .await
        .expect_err("plus de règlement");
    assert!(
        matches!(
            err,
            DbError::SettlementNotCancellable {
                blocker: kesh_db::errors::SettlementCancelBlocker::SupplierInvoiceNotPaid
            }
        ),
        "got {err:?}"
    );
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

/// ⛔ **La vue de règlement décrit un seul état** (revue P1) : la facture et
/// son motif de refus viennent de la même lecture — `paid` et annulable,
/// puis, après l'annulation, `open` et `SUPPLIER_INVOICE_NOT_PAID` ; une autre
/// société n'y lit rien. ⚠️ Ce test ne prouve pas l'instantané lui-même (aucun
/// point d'entrée ne permet d'intercaler une écriture entre deux lectures) :
/// il prouve que la vue assemble la facture et ses champs, cohérents entre eux.
#[sqlx::test(migrations = "./test-schema")]
async fn settlement_view_reads_the_invoice_and_its_motive_together(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let c = ctx.seeded.company_id;
    let (id, _, _) = paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;

    let view = supplier_invoices::get_settlement_view(&pool, c, id)
        .await
        .unwrap()
        .expect("facture");
    assert_eq!(view.invoice.status, "paid");
    assert!(!view.lines.is_empty(), "les lignes suivent");
    assert!(
        view.settlement_cancel_blocker.is_none(),
        "{:?}",
        view.settlement_cancel_blocker
    );
    assert_eq!(view.last_confirmed_batch, None, "réglée hors lot");

    supplier_invoices::cancel_settlement(&pool, c, id, ctx.seeded.admin_user_id)
        .await
        .unwrap();
    let view = supplier_invoices::get_settlement_view(&pool, c, id)
        .await
        .unwrap()
        .expect("facture");
    assert_eq!(view.invoice.status, "open");
    assert!(matches!(
        view.settlement_cancel_blocker,
        Some((SettlementCancelBlocker::SupplierInvoiceNotPaid, None, None))
    ));

    let other = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    assert!(
        supplier_invoices::get_settlement_view(&pool, other, id)
            .await
            .unwrap()
            .is_none(),
        "autre société"
    );
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

/// Rang 2 bis (Story 15-1a2-0) : pose le groupe `document` sur les deux lignes
/// du compte créanciers (2000) — l'achat et son paiement, somme nulle —, puis
/// le verrou de période ENSUITE, à `le_plus_recent` (D4). Rend la clé.
async fn figer(
    pool: &MySqlPool,
    ctx: &Ctx,
    achat: i64,
    paiement: i64,
    le_plus_recent: NaiveDate,
) -> i64 {
    let cle = poser_groupe_document(pool, ctx.seeded.accounts["2000"], &[achat, paiement]).await;
    kesh_db::repositories::companies::lock_books(
        pool,
        ctx.seeded.admin_user_id,
        ctx.seeded.company_id,
        le_plus_recent,
    )
    .await
    .expect("verrou de période");
    cle
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
    let (id, achat, paiement) = paid_invoice(
        pool,
        &ctx,
        dd - chrono::Duration::days(20),
        dd - chrono::Duration::days(10),
    )
    .await;
    if motifs.contains(&SettlementCancelBlocker::DocumentLetteringInClosedPeriods) {
        figer(pool, &ctx, achat, paiement, dd - chrono::Duration::days(10)).await;
    }
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
///
/// Rang **2 bis** (Story 15-1a2-0, AC2 c) : seul, puis contre chacun de ses
/// voisins de la queue. ⛔ La tête `SupplierInvoiceNotPaid` × 2 bis **n'existe
/// pas** : une facture non payée n'a pas de règlement, donc pas de groupe
/// `document` — nommée ici, non montée.
#[sqlx::test(migrations = "./test-schema")]
async fn tail_motives_through_the_supplier_path(pool: MySqlPool) {
    use SettlementCancelBlocker::*;
    let cas: [(&[SettlementCancelBlocker], SettlementCancelBlocker); 8] = [
        (&[FiscalYearClosed], FiscalYearClosed),
        (&[AccountArchived], AccountArchived),
        (&[NoOpenFiscalYearToday], NoOpenFiscalYearToday),
        (&[AccountArchived, NoOpenFiscalYearToday], AccountArchived),
        (
            &[DocumentLetteringInClosedPeriods],
            DocumentLetteringInClosedPeriods,
        ),
        (
            &[FiscalYearClosed, DocumentLetteringInClosedPeriods],
            FiscalYearClosed,
        ),
        (
            &[DocumentLetteringInClosedPeriods, AccountArchived],
            DocumentLetteringInClosedPeriods,
        ),
        (
            &[DocumentLetteringInClosedPeriods, NoOpenFiscalYearToday],
            DocumentLetteringInClosedPeriods,
        ),
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
            DocumentLetteringInClosedPeriods => matches!(
                err,
                DbError::SettlementNotCancellable {
                    blocker: DocumentLetteringInClosedPeriods
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

// ---------------------------------------------------------------------------
// Story 25-3-c (#454) — annuler une facture fournisseur, même payée
// ---------------------------------------------------------------------------

/// ⛔ **Le socle — l'autorité ACHAT ne couvre QUE l'écriture d'achat.** Appel
/// DIRECT sur l'écriture de RÈGLEMENT : refusé en `OWNED_BY_SUPPLIER_INVOICE`,
/// comme sans autorité. Le témoin positif — l'écriture d'achat — passe.
#[sqlx::test(migrations = "./test-schema")]
async fn purchase_authority_never_covers_the_settlement_entry(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (id, purchase, settlement) =
        paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;
    let authority = ReversalAuthority::SupplierPurchase {
        supplier_invoice_id: id,
    };

    let mut tx = pool.begin().await.unwrap();
    let err = journal_entries::reverse_owned_in_tx(
        &mut tx,
        ctx.seeded.company_id,
        settlement,
        ctx.seeded.admin_user_id,
        authority,
    )
    .await
    .expect_err("le règlement ne se contre-passe pas au titre de l'achat");
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
        purchase,
        ctx.seeded.admin_user_id,
        authority,
    )
    .await
    .expect("témoin : l'écriture d'achat, elle, se contre-passe");
}

/// Engage la facture dans un lot `generated` — état produit en production par
/// `payment_batches::create_batch`, forgé ici en SQL (le lot n'est pas l'objet
/// du test, seul son statut compte).
async fn engager_dans_un_lot(pool: &MySqlPool, ctx: &Ctx, supplier_invoice_id: i64) {
    let bank = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary) \
         VALUES (?, 'Banque lot', 'CH9300762011623852957', FALSE)",
    )
    .bind(ctx.seeded.company_id)
    .execute(pool)
    .await
    .expect("compte bancaire")
    .last_insert_id() as i64;
    let batch = sqlx::query(
        "INSERT INTO payment_batches (company_id, bank_account_id, status, \
         requested_execution_date, total_amount, msg_id, payment_info_id) \
         VALUES (?, ?, 'generated', '2026-06-30', 100, 'MSG-25-3-C', 'PMT-25-3-C')",
    )
    .bind(ctx.seeded.company_id)
    .bind(bank)
    .execute(pool)
    .await
    .expect("lot")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO payment_batch_items (payment_batch_id, supplier_invoice_id, position, \
         end_to_end_id, amount) VALUES (?, ?, 1, 'E2E-25-3-C', 100)",
    )
    .bind(batch)
    .bind(supplier_invoice_id)
    .execute(pool)
    .await
    .expect("ligne de lot");
}

/// Monte une facture fournisseur dont l'ACHAT est daté dans un exercice
/// raccourci à `D = aujourd'hui − 60 j` (même calendrier que [`monter`]),
/// payée ou non, puis applique les motifs demandés : exercice clos, compte de
/// charge 4000 archivé, aucun exercice couvrant le jour, lot `generated`.
async fn monter_achat(
    pool: &MySqlPool,
    motifs: &[SettlementCancelBlocker],
    payee: bool,
) -> (Ctx, i64) {
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
    let id = if payee {
        let (id, achat, paiement) = paid_invoice(
            pool,
            &ctx,
            dd - chrono::Duration::days(20),
            dd - chrono::Duration::days(10),
        )
        .await;
        if motifs.contains(&SettlementCancelBlocker::DocumentLetteringInClosedPeriods) {
            figer(pool, &ctx, achat, paiement, dd - chrono::Duration::days(10)).await;
        }
        id
    } else {
        assert!(
            !motifs.contains(&SettlementCancelBlocker::DocumentLetteringInClosedPeriods),
            "montage : un groupe `document` suppose une facture payée"
        );
        let mut new = one_line(&ctx, dec!(100.00), dec!(0));
        new.invoice_date = dd - chrono::Duration::days(20);
        new.due_date = Some(new.invoice_date);
        supplier_invoices::create(pool, new, ctx.seeded.admin_user_id)
            .await
            .expect("création")
            .invoice
            .id
    };
    if motifs.contains(&SettlementCancelBlocker::SupplierInvoiceInPaymentBatch) {
        engager_dans_un_lot(pool, &ctx, id).await;
    }
    if motifs.contains(&SettlementCancelBlocker::AccountArchived) {
        let charge = ctx.seeded.accounts["4000"];
        let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
            .bind(charge)
            .fetch_one(pool)
            .await
            .unwrap();
        accounts::archive(pool, charge, version, ctx.seeded.admin_user_id)
            .await
            .expect("archivage du compte de charge");
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

/// ⛔ **La précédence de l'annulation d'une facture, lecture ET clic.** Chaque
/// motif atteignable seul, puis les paires où le lot est en concurrence : le
/// lot passe TOUJOURS en dernier. La lecture annonce le motif, l'écriture
/// refuse pour ce même motif.
///
/// Rang **2 bis** (Story 15-1a2-0, AC2 c), sur une facture **payée** : seul,
/// puis contre l'exercice clos, le compte archivé et l'exercice du jour.
/// ⛔ Le rang 6 (`SupplierInvoiceInPaymentBatch`) × 2 bis est
/// **INATTEIGNABLE**, nommé et non monté : il suppose une facture payée dans un
/// lot `generated`, et `supplier_invoices.rs` l'écrit — « une facture `paid` ne
/// peut pas être dans un lot `generated` » (`create_batch` exige `open`, `pay`
/// et `cancel` refusent une facture en lot `generated`, `confirm_batch` règle
/// et confirme dans la même transaction). [`engager_dans_un_lot`] forge un lot
/// sur n'importe quelle facture : « montable » serait toujours vrai.
#[sqlx::test(migrations = "./test-schema")]
async fn invoice_cancel_motives_and_their_precedence(pool: MySqlPool) {
    use SettlementCancelBlocker::*;
    let cas: [(&[SettlementCancelBlocker], bool, SettlementCancelBlocker); 11] = [
        (
            &[DocumentLetteringInClosedPeriods],
            true,
            DocumentLetteringInClosedPeriods,
        ),
        (
            &[FiscalYearClosed, DocumentLetteringInClosedPeriods],
            true,
            FiscalYearClosed,
        ),
        (
            &[DocumentLetteringInClosedPeriods, AccountArchived],
            true,
            DocumentLetteringInClosedPeriods,
        ),
        (
            &[DocumentLetteringInClosedPeriods, NoOpenFiscalYearToday],
            true,
            DocumentLetteringInClosedPeriods,
        ),
        (&[FiscalYearClosed], false, FiscalYearClosed),
        (&[FiscalYearClosed], true, FiscalYearClosed),
        (&[AccountArchived], false, AccountArchived),
        (&[NoOpenFiscalYearToday], false, NoOpenFiscalYearToday),
        (
            &[SupplierInvoiceInPaymentBatch],
            false,
            SupplierInvoiceInPaymentBatch,
        ),
        (
            &[FiscalYearClosed, SupplierInvoiceInPaymentBatch],
            false,
            FiscalYearClosed,
        ),
        (
            &[AccountArchived, SupplierInvoiceInPaymentBatch],
            false,
            AccountArchived,
        ),
    ];
    for (motifs, payee, attendu) in cas {
        let (ctx, id) = monter_achat(&pool, motifs, payee).await;
        let mut conn = pool.acquire().await.unwrap();
        let lu = supplier_invoices::supplier_invoice_cancel_blocker(
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
            "lecture, motifs {motifs:?} (payée : {payee})"
        );
        if attendu == AccountArchived {
            assert_eq!(lu.and_then(|h| h.2).as_deref(), Some("4000"));
        }
        let err =
            supplier_invoices::cancel(&pool, ctx.seeded.company_id, id, ctx.seeded.admin_user_id)
                .await
                .expect_err("refusée");
        let ok = match attendu {
            FiscalYearClosed | DocumentLetteringInClosedPeriods | SupplierInvoiceInPaymentBatch => {
                matches!(
                    err,
                    DbError::SupplierInvoiceNotCancellable { blocker } if blocker == attendu
                )
            }
            AccountArchived => matches!(err, DbError::ReversalAccountsArchived(_)),
            NoOpenFiscalYearToday => matches!(err, DbError::FiscalYearInvalid),
            _ => false,
        };
        assert!(ok, "écriture, motifs {motifs:?} : reçu {err:?}");
    }
}

/// Rang 1 : une facture déjà annulée — la seconde annulation.
#[sqlx::test(migrations = "./test-schema")]
async fn a_second_cancellation_is_refused_with_its_code(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (c, u) = (ctx.seeded.company_id, ctx.seeded.admin_user_id);
    let id = supplier_invoices::create(&pool, one_line(&ctx, dec!(50.00), dec!(0)), u)
        .await
        .unwrap()
        .invoice
        .id;
    supplier_invoices::cancel(&pool, c, id, u).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let lu = supplier_invoices::supplier_invoice_cancel_blocker(&mut conn, c, id)
        .await
        .unwrap();
    drop(conn);
    assert_eq!(
        lu.map(|h| h.0),
        Some(SettlementCancelBlocker::SupplierInvoiceCancelled)
    );
    let err = supplier_invoices::cancel(&pool, c, id, u)
        .await
        .expect_err("double annulation");
    assert!(
        matches!(
            err,
            DbError::SupplierInvoiceNotCancellable {
                blocker: SettlementCancelBlocker::SupplierInvoiceCancelled
            }
        ),
        "got {err:?}"
    );
}

/// ⛔ **Une clôture concurrente attend l'annulation de la facture** — et, une
/// fois validée, la fait refuser. Prouve le verrou de l'écriture d'achat ET de
/// son exercice, et qu'aucune lecture non verrouillante ne le précède (sinon
/// l'instantané, figé avant l'attente, relirait l'exercice « ouvert »).
#[sqlx::test(migrations = "./test-schema")]
async fn a_concurrent_close_waits_for_the_invoice_cancellation(pool: MySqlPool) {
    let (ctx, id) = monter_achat(&pool, &[], true).await;
    let mut closing = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.seeded.fiscal_year_id)
        .execute(&mut *closing)
        .await
        .unwrap();
    let p = pool.clone();
    let (c, u) = (ctx.seeded.company_id, ctx.seeded.admin_user_id);
    let annulation = tokio::spawn(async move { supplier_invoices::cancel(&p, c, id, u).await });
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
            Err(DbError::SupplierInvoiceNotCancellable {
                blocker: SettlementCancelBlocker::FiscalYearClosed
            })
        ),
        "l'annulation devait attendre la clôture puis refuser — reçu {result:?}"
    );
}

/// ⛔ **Composition et rollback** : dans une transaction fournie, le geste
/// écrit (lecture positive), puis l'abandon efface tout.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_composes_and_rolls_back(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (id, _, _) = paid_invoice(&pool, &ctx, d(2026, 6, 15), d(2026, 6, 20)).await;
    let tables = ["journal_entries", "journal_entry_lines", "audit_log"];
    let mut avant = Vec::new();
    for t in tables {
        avant.push(count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await);
    }
    let mut tx = pool.begin().await.unwrap();
    let done = supplier_invoices::cancel_in_tx(
        &mut tx,
        ctx.seeded.company_id,
        id,
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("annulation dans la transaction");
    assert_eq!(done.invoice.status, "cancelled");
    let dedans: String = sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(dedans, "cancelled", "lecture positive dans la transaction");
    tx.rollback().await.unwrap();

    let apres: (String, Option<i64>) = sqlx::query_as(
        "SELECT status, settlement_journal_entry_id FROM supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(apres.0, "paid");
    assert!(apres.1.is_some(), "le règlement est toujours attaché");
    for (t, n) in tables.iter().zip(avant) {
        assert_eq!(
            count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await,
            n,
            "rien ne reste dans {t}"
        );
    }
}

/// ⛔ **Étanchéité multi-tenant** : table par table, rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn another_company_cannot_cancel_the_invoice(pool: MySqlPool) {
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
    let err = supplier_invoices::cancel(&pool, other, id, ctx.seeded.admin_user_id)
        .await
        .expect_err("autre société");
    assert!(matches!(err, DbError::NotFound), "got {err:?}");
    let mut conn = pool.acquire().await.unwrap();
    let err = supplier_invoices::supplier_invoice_cancel_blocker(&mut conn, other, id)
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

/// Une facture à DEUX lignes de comptes différents, taguée projet : la
/// contre-passation du socle reprend le projet sur CHAQUE ligne.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_keeps_the_project_on_every_line(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let project_id = make_project(&pool, ctx.seeded.company_id, "DEUX", false).await;
    let mut new = one_line(&ctx, dec!(100.00), dec!(8.10));
    new.project_id = Some(project_id);
    new.lines.push(NewSupplierInvoiceLine {
        description: "Seconde".into(),
        quantity: dec!(1),
        unit_price: dec!(50.00),
        vat_rate: dec!(0),
        expense_account_id: ctx.seeded.accounts["4000"],
    });
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
    .unwrap();
    let sans_projet = count(
        &pool,
        &format!(
            "SELECT COUNT(*) FROM journal_entry_lines jel \
             JOIN journal_entries je ON je.id = jel.entry_id \
             WHERE je.reverses_entry_id = {} AND (jel.project_id IS NULL OR jel.project_id <> {project_id})",
            created.invoice.purchase_journal_entry_id
        ),
    )
    .await;
    assert_eq!(
        sans_projet, 0,
        "chaque ligne de la contre-passation porte le projet"
    );
}

// ---------------------------------------------------------------------------
// Story 15-5d (#429) — la garde À L'USAGE des comptes de réglage, côté achat
// ---------------------------------------------------------------------------

/// Les créanciers et la TVA récupérable désignés dans les réglages, contrôlés
/// au moment où la saisie y écrit : `DesignatedAccountsNotPostable` (AC1, AC7).
/// Montage de ce fichier (`setup` : créanciers `2000`, TVA récupérable `1171`) ;
/// la saisie fournisseur n'a pas d'étape d'arrondi. Chaque compte de test est
/// désigné **avant** d'être rendu non imputable.
mod garde_usage_comptes_reglage {
    use super::*;
    use kesh_db::test_fixtures::{attendre_une_requete_en_cours, sonde_verrou_nowait};

    const ACCESSEUR: &[&str] = &[
        "FROM accounts FORCE INDEX (PRIMARY) WHERE company_id",
        "LOCK IN SHARE MODE",
    ];
    const SONDE_EXERCICE: &str = "SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE NOWAIT";
    /// Motifs de `fiscal_years::find_open_covering_date`.
    const EXERCICE: &[&str] = &["FROM fiscal_years", "FOR UPDATE"];
    const SONDE_COMPTE: &str = "SELECT id FROM accounts WHERE id = ? FOR UPDATE NOWAIT";

    async fn set_postable(pool: &MySqlPool, account_id: i64, postable: bool) {
        sqlx::query("UPDATE accounts SET postable = ? WHERE id = ?")
            .bind(postable)
            .bind(account_id)
            .execute(pool)
            .await
            .unwrap();
    }

    fn designated_rejected(err: DbError) -> Vec<(i64, String)> {
        match err {
            DbError::DesignatedAccountsNotPostable(list) => list
                .iter()
                .map(|a| (a.account_id, a.account_number.clone()))
                .collect(),
            other => panic!("attendu DesignatedAccountsNotPostable, obtenu {other:?}"),
        }
    }

    /// Aucune facture fournisseur et aucune écriture pour la société.
    async fn assert_nothing_written(pool: &MySqlPool, ctx: &Ctx) {
        for sql in [
            "SELECT COUNT(*) FROM supplier_invoices WHERE company_id = ?",
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
        ] {
            let n: i64 = sqlx::query_scalar(sql)
                .bind(ctx.seeded.company_id)
                .fetch_one(pool)
                .await
                .unwrap();
            assert_eq!(n, 0, "rien ne doit être écrit : {sql}");
        }
    }

    /// Créanciers non imputables → saisie refusée, rien d'écrit.
    #[sqlx::test(migrations = "./test-schema")]
    async fn payable_not_postable_refuses_the_entry(pool: MySqlPool) {
        let ctx = setup(&pool).await;
        let payable = ctx.seeded.accounts["2000"];
        set_postable(&pool, payable, false).await;

        let err = supplier_invoices::create(
            &pool,
            one_line(&ctx, dec!(100.00), dec!(0)),
            ctx.seeded.admin_user_id,
        )
        .await
        .expect_err("refusée");
        assert_eq!(err.error_code(), "ACCOUNT_NOT_POSTABLE");
        assert_eq!(
            designated_rejected(err),
            vec![(payable, "2000".to_string())]
        );
        assert_nothing_written(&pool, &ctx).await;
    }

    /// TVA récupérable non imputable : avec TVA → refusée ; sans TVA → acceptée.
    #[sqlx::test(migrations = "./test-schema")]
    async fn vat_recoverable_not_postable_refuses_only_when_vat_is_written(pool: MySqlPool) {
        let ctx = setup(&pool).await;
        set_postable(&pool, ctx.recoverable_id, false).await;

        let err = supplier_invoices::create(
            &pool,
            one_line(&ctx, dec!(100.00), dec!(8.10)),
            ctx.seeded.admin_user_id,
        )
        .await
        .expect_err("avec TVA : refusée");
        assert_eq!(
            designated_rejected(err),
            vec![(ctx.recoverable_id, "1171".to_string())]
        );
        assert_nothing_written(&pool, &ctx).await;

        supplier_invoices::create(
            &pool,
            one_line(&ctx, dec!(100.00), dec!(0)),
            ctx.seeded.admin_user_id,
        )
        .await
        .expect("sans TVA : la TVA récupérable n'est pas écrite, la saisie passe");
    }

    /// Ordre des refus — aucun exercice ouvert ET créanciers non imputables →
    /// `FiscalYearInvalid` (le verrou précède l'exercice, le refus non).
    #[sqlx::test(migrations = "./test-schema")]
    async fn no_open_fiscal_year_comes_before_the_guard(pool: MySqlPool) {
        let ctx = setup(&pool).await;
        sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
            .bind(ctx.seeded.fiscal_year_id)
            .execute(&pool)
            .await
            .unwrap();
        set_postable(&pool, ctx.seeded.accounts["2000"], false).await;

        let err = supplier_invoices::create(
            &pool,
            one_line(&ctx, dec!(100.00), dec!(8.10)),
            ctx.seeded.admin_user_id,
        )
        .await
        .expect_err("refusée");
        assert!(
            matches!(err, DbError::FiscalYearInvalid),
            "attendu FiscalYearInvalid, obtenu {err:?}"
        );
    }

    /// **Test de place 2** — achat : le verrou des comptes désignés précède
    /// l'exercice, et il lit la ligne FRAÎCHE (finding F3-2, C43, C87).
    ///
    /// La bloqueuse exécute l'`UPDATE accounts SET postable = FALSE` des
    /// créanciers sans conclure ; la saisie est vue en attente sur l'accesseur ;
    /// une sonde sur l'exercice réussit ; la bloqueuse valide ; la saisie lit la
    /// ligne fraîche et refuse. Sous l'ordre fautif (verrou après l'exercice),
    /// la sonde échoue.
    #[sqlx::test(migrations = "./test-schema")]
    async fn place_2_purchase_lock_precedes_fiscal_year_and_reads_fresh(pool: MySqlPool) {
        let ctx = setup(&pool).await;
        let payable = ctx.seeded.accounts["2000"];

        let mut bloqueuse = pool.begin().await.unwrap();
        sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
            .bind(payable)
            .execute(&mut *bloqueuse)
            .await
            .unwrap();
        let (p, new, u) = (
            pool.clone(),
            one_line(&ctx, dec!(100.00), dec!(8.10)),
            ctx.seeded.admin_user_id,
        );
        let tache = tokio::spawn(async move { supplier_invoices::create(&p, new, u).await });
        let vue = attendre_une_requete_en_cours(&pool, ACCESSEUR, || tache.is_finished()).await;
        assert!(
            vue,
            "la saisie n'a pas été vue en attente sur l'accesseur : {:?}",
            tache.await.map(|r| r.map(|_| ()))
        );
        assert!(
            sonde_verrou_nowait(&pool, SONDE_EXERCICE, ctx.seeded.fiscal_year_id).await,
            "la saisie ne doit pas encore tenir l'exercice"
        );
        bloqueuse.commit().await.unwrap();

        let err = tache.await.expect("tâche").expect_err("refusée");
        assert_eq!(
            designated_rejected(err),
            vec![(payable, "2000".to_string())]
        );
        assert_nothing_written(&pool, &ctx).await;
    }

    /// **Test de mode — achat** (revue de code P1, E2) : le verrou des comptes
    /// désignés de la saisie est **partagé**, patron du test de mode 4 de la
    /// vente. L'accesseur est commun aux deux flux ; ce test couvre le site
    /// d'achat lui-même (ses candidats, sa place).
    ///
    /// La bloqueuse reproduit ce que tient un règlement fournisseur à
    /// l'insertion de ses lignes : l'exercice en exclusif, les créanciers (et
    /// la TVA récupérable) en partagé. La saisie est vue en attente **sur
    /// l'exercice** — donc passée l'accesseur malgré les partagés de la
    /// bloqueuse. Sous un accesseur en `FOR UPDATE`, elle attendrait sur les
    /// comptes, et `attendre_une_requete_en_cours` paniquerait.
    #[sqlx::test(migrations = "./test-schema")]
    async fn mode_purchase_lock_is_shared(pool: MySqlPool) {
        let ctx = setup(&pool).await;
        let payable = ctx.seeded.accounts["2000"];
        let recoverable = ctx.recoverable_id;

        let mut bloqueuse = pool.begin().await.unwrap();
        sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
            .bind(ctx.seeded.fiscal_year_id)
            .fetch_all(&mut *bloqueuse)
            .await
            .unwrap();
        // ⛔ `name` est lu À DESSEIN (C-15-5d-1) : `SELECT id … LOCK IN SHARE
        // MODE` est couvert par l'index secondaire `fk_accounts_parent` et ne
        // verrouillerait pas la ligne de la clé primaire.
        sqlx::query("SELECT id, name FROM accounts WHERE id IN (?, ?) LOCK IN SHARE MODE")
            .bind(payable)
            .bind(recoverable)
            .fetch_all(&mut *bloqueuse)
            .await
            .unwrap();
        // Le montage lui-même est vérifié : les deux lignes sont bien tenues.
        for compte in [payable, recoverable] {
            assert!(
                !sonde_verrou_nowait(&pool, SONDE_COMPTE, compte).await,
                "la bloqueuse doit tenir le compte {compte} en partagé"
            );
        }
        let (p, new, u) = (
            pool.clone(),
            one_line(&ctx, dec!(100.00), dec!(8.10)),
            ctx.seeded.admin_user_id,
        );
        let tache = tokio::spawn(async move { supplier_invoices::create(&p, new, u).await });
        let vue = attendre_une_requete_en_cours(&pool, EXERCICE, || tache.is_finished()).await;
        assert!(
            vue,
            "la saisie a fini sans attendre l'exercice : {:?}",
            tache.await.map(|r| r.map(|_| ()))
        );
        bloqueuse.rollback().await.unwrap();

        tache
            .await
            .expect("tâche")
            .expect("la saisie réussit une fois l'exercice rendu");
    }
}

// ---------------------------------------------------------------------------
// Story 15-6b (#474, AC4) — un règlement ne vise pas le compte qu'il solde
// ---------------------------------------------------------------------------

/// Le règlement fournisseur refusé ne laisse rien : facture `open`, version
/// inchangée, aucune écriture neuve.
async fn assert_supplier_nothing_written(
    pool: &MySqlPool,
    invoice_id: i64,
    (entries_before, version_before): (i64, i32),
) {
    let (status, version, settlement): (String, i32, Option<i64>) = sqlx::query_as(
        "SELECT status, version, settlement_journal_entry_id FROM supplier_invoices WHERE id = ?",
    )
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(status, "open");
    assert_eq!(version, version_before);
    assert!(settlement.is_none());
    let entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(entries, entries_before, "aucune écriture neuve");
}

async fn supplier_before(pool: &MySqlPool, invoice_id: i64) -> (i64, i32) {
    let entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap();
    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (entries, version)
}

fn assert_payable_refusal(err: &DbError, payable: i64) {
    match err {
        DbError::SettlementCounterpartyIsClaimAccount {
            account_id,
            account_number,
            claim: kesh_db::errors::ClaimSide::Payable,
            role: kesh_db::errors::SettlementAccountRole::Counterparty,
            batch: None,
        } => {
            assert_eq!(*account_id, payable);
            assert_eq!(account_number.as_deref(), Some("2000"));
        }
        other => panic!("refus de contrepartie attendu, obtenu {other:?}"),
    }
}

/// Story 15-6b, test 5 — **compte interne = 2000** : `D 2000 / C 2000` refusé.
#[sqlx::test(migrations = "./test-schema")]
async fn pay_with_the_payable_account_as_internal_account_is_refused(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let payable = ctx.seeded.accounts["2000"];
    let created = supplier_invoices::create(
        &pool,
        one_line(&ctx, dec!(500.00), dec!(0)),
        ctx.seeded.admin_user_id,
    )
    .await
    .unwrap();
    let before = supplier_before(&pool, created.invoice.id).await;

    let err = supplier_invoices::pay(
        &pool,
        ctx.seeded.company_id,
        created.invoice.id,
        SettlementChoice::InternalAccount {
            account_id: payable,
        },
        d(2026, 6, 20),
        ctx.seeded.admin_user_id,
    )
    .await
    .expect_err("la dette comme contrepartie doit être refusée");
    assert_payable_refusal(&err, payable);
    assert_supplier_nothing_written(&pool, created.invoice.id, before).await;
}

/// Story 15-6b, test 6 — **virement sur un compte bancaire lié au 2000** : même
/// refus. (La fixture `pay_bank_transfer_uses_journal_account` lie au 1100 :
/// elle reste valide, le compte soldé est 2000.)
#[sqlx::test(migrations = "./test-schema")]
async fn pay_by_a_bank_account_linked_to_the_payable_account_is_refused(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let payable = ctx.seeded.accounts["2000"];
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
        .bind(payable)
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
    let before = supplier_before(&pool, created.invoice.id).await;

    let err = supplier_invoices::pay(
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
    .expect_err("un compte bancaire lié à la dette doit être refusé");
    assert_payable_refusal(&err, payable);
    assert_supplier_nothing_written(&pool, created.invoice.id, before).await;
}

/// Story 15-1a-ii (AC9 (d), R6, C106) — l'annulation d'une facture
/// fournisseur validée et non payée contre-passe son achat : la ligne
/// fournisseurs (`2000`, passif, lettrable) et son miroir forment un groupe
/// `reversal`. L'origine reste l'écriture d'**achat** de la facture (motif
/// `OwnedBySupplierInvoice`) : ce groupe ne se dissout pas à la main —
/// `dissolve_group_in_tx` en mode `Manual` (celui de `DELETE /letterings`)
/// rend `LetteringLineOwnedByDocument`, sans quoi la paire resterait ouverte
/// pour toujours (R5 interdit de la relettrer).
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand(pool: MySqlPool) {
    use kesh_db::repositories::letterings::{self, Actor, Mode};
    let (ctx, id) = monter_achat(&pool, &[], false).await;
    let achat: i64 =
        sqlx::query_scalar("SELECT purchase_journal_entry_id FROM supplier_invoices WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    supplier_invoices::cancel(&pool, ctx.seeded.company_id, id, ctx.seeded.admin_user_id)
        .await
        .expect("annulation");

    let fournisseurs = ctx.seeded.accounts["2000"];
    let (ligne, key, origin): (i64, Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT id, lettering_key, lettering_origin FROM journal_entry_lines \
         WHERE entry_id = ? AND account_id = ?",
    )
    .bind(achat)
    .bind(fournisseurs)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        key,
        Some(ligne),
        "la ligne fournisseurs de l'achat est lettrée"
    );
    assert_eq!(origin.as_deref(), Some("reversal"));
    let miroir: Option<i64> = sqlx::query_scalar(
        "SELECT jel.lettering_key FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE je.reverses_entry_id = ? AND jel.account_id = ?",
    )
    .bind(achat)
    .bind(fournisseurs)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(miroir, key, "avec son miroir");

    let mut tx = pool.begin().await.unwrap();
    let r = letterings::dissolve_group_in_tx(
        &mut tx,
        ctx.seeded.company_id,
        ligne,
        Mode::Manual,
        Actor {
            user_id: ctx.seeded.admin_user_id,
            api_key_id: None,
        },
    )
    .await;
    tx.rollback().await.unwrap();
    assert!(
        matches!(
            r,
            Err(DbError::LetteringLineOwnedByDocument {
                blocker: kesh_db::errors::ReversalBlocker::OwnedBySupplierInvoice,
                ..
            })
        ),
        "obtenu {r:?}"
    );
}

/// Story 15-1a2-0 (#518, AC5) — achat et paiement par les gestes, groupe sur les
/// deux lignes créanciers, verrou ENSUITE : l'annulation du **paiement** et
/// celle de la **facture** sont refusées au rang 2 bis, chacune dans sa
/// famille ; rien n'est écrit, la facture reste `paid` ; les deux prédicteurs
/// le disent.
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_cancels_are_refused_under_a_frozen_lettering(pool: MySqlPool) {
    use SettlementCancelBlocker::DocumentLetteringInClosedPeriods as Fige;
    let ctx = setup(&pool).await;
    let dd = chrono::Utc::now().date_naive() - chrono::Duration::days(60);
    let (id, achat, paiement) = paid_invoice(
        &pool,
        &ctx,
        dd - chrono::Duration::days(20),
        dd - chrono::Duration::days(10),
    )
    .await;
    let cle = figer(
        &pool,
        &ctx,
        achat,
        paiement,
        dd - chrono::Duration::days(10),
    )
    .await;
    let company_id = ctx.seeded.company_id;
    let user = ctx.seeded.admin_user_id;

    let mut conn = pool.acquire().await.unwrap();
    let paiement_lu =
        supplier_invoices::supplier_settlement_cancel_blocker(&mut conn, company_id, id)
            .await
            .expect("prédicteur du paiement");
    let facture_lue = supplier_invoices::supplier_invoice_cancel_blocker(&mut conn, company_id, id)
        .await
        .expect("prédicteur de la facture");
    drop(conn);
    assert_eq!(
        paiement_lu.map(|h| h.0),
        Some(Fige),
        "settlementCancelBlockedBy"
    );
    assert_eq!(facture_lue.map(|h| h.0), Some(Fige), "cancelBlockedBy");

    let tables = ["journal_entries", "audit_log"];
    let mut avant = Vec::new();
    for t in tables {
        avant.push(count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await);
    }

    let err = supplier_invoices::cancel_settlement(&pool, company_id, id, user)
        .await
        .expect_err("paiement figé");
    assert!(
        matches!(err, DbError::SettlementNotCancellable { blocker: Fige }),
        "{err:?}"
    );
    let err = supplier_invoices::cancel(&pool, company_id, id, user)
        .await
        .expect_err("facture figée");
    assert!(
        matches!(
            err,
            DbError::SupplierInvoiceNotCancellable { blocker: Fige }
        ),
        "{err:?}"
    );

    for (t, n) in tables.iter().zip(avant) {
        assert_eq!(
            count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await,
            n,
            "rien d'écrit dans {t}"
        );
    }
    let (status, reglement): (String, Option<i64>) = sqlx::query_as(
        "SELECT status, settlement_journal_entry_id FROM supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "paid", "la facture reste payée");
    assert_eq!(reglement, Some(paiement), "son règlement reste attaché");
    assert_eq!(
        lignes_du_groupe(&pool, cle).await,
        2,
        "les marques sont intactes"
    );
}
