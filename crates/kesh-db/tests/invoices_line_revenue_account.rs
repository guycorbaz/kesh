//! Tests d'intégration du **compte de produit par ligne** — Story 16-1a
//! (#152, CR #265).
//!
//! Couvre bout-en-bout (DB éphémère + écritures réellement insérées) :
//!
//! - **AC17** — le test pivot de D5 : facture ventilée puis avoir total, les
//!   deux écritures s'annulent compte par compte ; et le cas où le défaut
//!   société **change entre les deux pièces**, qui échoue si la matérialisation
//!   d'AC9-bis n'est pas implémentée.
//! - **AC18 / AC18-bis** — comptes invalides à la saisie et au posting, dont
//!   les deux trous que `create_in_tx` ne couvre pas (`postable`,
//!   `account_type`) et le compte par défaut de la société.
//! - **AC19** — invariant D3-bis : `NULL` et le défaut explicite ont le même
//!   verdict et produisent la même écriture.
//! - **AC20** — garde-fou D1 : le repli suit `settings.default_revenue_account_id`,
//!   PAS le compte portant le rôle `DefaultRevenue`.
//! - **AC21** — D4-bis : facture à montant nul → erreur métier, pas 500 SQL.
//! - **AC12-bis** — non-régression de la saisie quand le défaut société est
//!   archivé ou absent.
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{InvoiceUpdate, NewCreditNote, NewInvoice, NewInvoiceLine};
use kesh_db::errors::{DbError, RevenueAccountRejection};
use kesh_db::repositories::{contacts, credit_notes, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

const INVOICE_DATE: (i32, u32, u32) = (2026, 6, 15);

fn invoice_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(INVOICE_DATE.0, INVOICE_DATE.1, INVOICE_DATE.2).unwrap()
}

async fn make_contact(pool: &MySqlPool, company_id: i64, admin_id: i64) -> i64 {
    contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Entreprise,
            name: "Client 16-1a".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
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
    .expect("create contact")
    .id
}

/// Crée un compte supplémentaire et retourne son id.
async fn add_account(
    pool: &MySqlPool,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: &str,
) -> i64 {
    sqlx::query("INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)")
        .bind(company_id)
        .bind(number)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await
        .expect("insert account")
        .last_insert_id() as i64
}

async fn set_account_active(pool: &MySqlPool, account_id: i64, active: bool) {
    sqlx::query("UPDATE accounts SET active = ? WHERE id = ?")
        .bind(active)
        .bind(account_id)
        .execute(pool)
        .await
        .expect("update active");
}

async fn set_account_postable(pool: &MySqlPool, account_id: i64, postable: bool) {
    sqlx::query("UPDATE accounts SET postable = ? WHERE id = ?")
        .bind(postable)
        .bind(account_id)
        .execute(pool)
        .await
        .expect("update postable");
}

async fn set_account_type(pool: &MySqlPool, account_id: i64, account_type: &str) {
    sqlx::query("UPDATE accounts SET account_type = ? WHERE id = ?")
        .bind(account_type)
        .bind(account_id)
        .execute(pool)
        .await
        .expect("update account_type");
}

async fn set_default_revenue(pool: &MySqlPool, company_id: i64, account_id: Option<i64>) {
    sqlx::query(
        "UPDATE company_invoice_settings SET default_revenue_account_id = ? WHERE company_id = ?",
    )
    .bind(account_id)
    .bind(company_id)
    .execute(pool)
    .await
    .expect("update default revenue");
}

/// `(unit_price, vat_rate, revenue_account_id)` → lignes de facture.
fn lines_from(spec: &[(Decimal, Decimal, Option<i64>)]) -> Vec<NewInvoiceLine> {
    spec.iter()
        .map(|(price, rate, account)| NewInvoiceLine {
            description: "Ligne".into(),
            quantity: dec!(1),
            unit_price: *price,
            vat_rate: *rate,
            revenue_account_id: *account,
        })
        .collect()
}

async fn create_draft(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    spec: &[(Decimal, Decimal, Option<i64>)],
) -> Result<i64, DbError> {
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date: invoice_date(),
        due_date: None,
        payment_terms: None,
        lines: lines_from(spec),
        project_id: None,
    };
    invoices::create(pool, seeded.admin_user_id, new)
        .await
        .map(|(inv, _)| inv.id)
}

/// Solde net (débit − crédit) par compte, agrégé sur plusieurs écritures.
fn net_by_account(
    entries: &[&kesh_db::entities::JournalEntryWithLines],
) -> std::collections::BTreeMap<i64, Decimal> {
    let mut net = std::collections::BTreeMap::new();
    for je in entries {
        for l in &je.lines {
            *net.entry(l.account_id).or_insert(Decimal::ZERO) += l.debit - l.credit;
        }
    }
    net
}

fn credit_on(je: &kesh_db::entities::JournalEntryWithLines, account_id: i64) -> Decimal {
    je.lines
        .iter()
        .filter(|l| l.account_id == account_id)
        .map(|l| l.credit)
        .sum()
}

fn debit_on(je: &kesh_db::entities::JournalEntryWithLines, account_id: i64) -> Decimal {
    je.lines
        .iter()
        .filter(|l| l.account_id == account_id)
        .map(|l| l.debit)
        .sum()
}

fn expect_invalid_accounts(err: DbError) -> Vec<kesh_db::errors::RejectedRevenueAccount> {
    match err {
        DbError::InvalidRevenueAccounts(v) => v,
        other => panic!("attendu InvalidRevenueAccounts, reçu {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC17 — le test pivot de D5
// ---------------------------------------------------------------------------

/// **AC17 cas 1** — facture ventilée sur 2 comptes puis avoir total : les deux
/// écritures s'annulent **compte par compte**.
///
/// C'est le garde-fou du mode de défaillance le plus grave de la story. Un
/// résidu laisserait l'équation du bilan équilibrée (donc aucun signal) et le
/// compte de résultat faux.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_cancels_ventilated_invoice_account_by_account(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;
    let goods = add_account(&pool, seeded.company_id, "3400", "Marchandises", "Revenue").await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[
            (dec!(1000.00), dec!(8.10), Some(services)),
            (dec!(400.00), dec!(2.60), Some(goods)),
        ],
    )
    .await
    .expect("create draft");

    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect("validate");

    // La facture ventile bien sur les deux comptes.
    assert_eq!(credit_on(&validated.journal_entry, services), dec!(1000.00));
    assert_eq!(credit_on(&validated.journal_entry, goods), dec!(400.00));

    let issued = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: invoice_date(),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("create credit note");

    // L'avoir extourne les MÊMES comptes.
    assert_eq!(debit_on(&issued.journal_entry, services), dec!(1000.00));
    assert_eq!(debit_on(&issued.journal_entry, goods), dec!(400.00));

    let net = net_by_account(&[&validated.journal_entry, &issued.journal_entry]);
    for (account_id, amount) in &net {
        assert_eq!(
            *amount,
            Decimal::ZERO,
            "résidu de {amount} sur le compte {account_id}"
        );
    }
    assert!(
        net.contains_key(&services) && net.contains_key(&goods),
        "les deux comptes ventilés doivent apparaître dans l'agrégat"
    );
}

/// **AC17 cas 2** — le test qui **échoue si la matérialisation d'AC9-bis
/// manque**, et c'est sa seule raison d'être.
///
/// Facture à lignes toutes `NULL`, validée alors que le défaut société est
/// `3000`. Le défaut est **ensuite changé** en `3200`. L'avoir doit débiter
/// `3000` — le compte **effectivement crédité** — et surtout pas `3200`.
///
/// Sans matérialisation, la ligne `NULL` serait recopiée `NULL` dans le
/// snapshot d'avoir et le repli se résoudrait sur le défaut **courant** :
/// facture créditée 3000, avoir débité 3200, résidu permanent. Un test qui ne
/// change pas le défaut entre les deux pièces passe systématiquement et ne
/// prouve rien.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_uses_materialized_account_not_current_default(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let original_default = seeded.accounts["3000"];
    let new_default = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(1000.00), dec!(8.10), None)],
    )
    .await
    .expect("create draft");

    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect("validate");
    assert_eq!(
        credit_on(&validated.journal_entry, original_default),
        dec!(1000.00),
        "la facture crédite le défaut en vigueur à sa validation"
    );

    // AC9-bis : la ligne a été matérialisée en base ET dans la réponse rendue
    // par l'endpoint (`ValidatedInvoice.lines`, délibérément non re-fetchée).
    assert!(
        validated
            .lines
            .iter()
            .all(|l| l.revenue_account_id == Some(original_default)),
        "la copie en mémoire doit être mutée, pas seulement la base"
    );
    let persisted: Vec<Option<i64>> = sqlx::query_scalar(
        "SELECT revenue_account_id FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
    )
    .bind(invoice_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        persisted.iter().all(|a| *a == Some(original_default)),
        "aucune ligne de LA FACTURE QU'ON VIENT DE VALIDER ne doit rester NULL"
    );

    // L'administrateur change le compte de produit par défaut.
    set_default_revenue(&pool, seeded.company_id, Some(new_default)).await;

    let issued = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: invoice_date(),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("create credit note");

    assert_eq!(
        debit_on(&issued.journal_entry, original_default),
        dec!(1000.00),
        "l'avoir extourne le compte EFFECTIVEMENT crédité par la facture"
    );
    assert_eq!(
        debit_on(&issued.journal_entry, new_default),
        Decimal::ZERO,
        "l'avoir ne doit PAS suivre le nouveau défaut société"
    );

    let net = net_by_account(&[&validated.journal_entry, &issued.journal_entry]);
    for (account_id, amount) in &net {
        assert_eq!(
            *amount,
            Decimal::ZERO,
            "résidu de {amount} sur le compte {account_id}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC18 — comptes invalides à la saisie et au posting
// ---------------------------------------------------------------------------

/// **AC18** — compte d'une autre société refusé à la **création** (anti-IDOR).
#[sqlx::test(migrations = "./test-schema")]
async fn create_rejects_cross_company_account(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    // Compte appartenant à une AUTRE société.
    let other_company = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'X', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let foreign = add_account(&pool, other_company, "3000", "Ventes autre", "Revenue").await;

    let err = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(foreign))],
    )
    .await
    .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].line_number, Some(1));
    assert_eq!(
        rejected[0].reason,
        RevenueAccountRejection::UnknownOrCrossCompany
    );
}

/// **AC18** — compte de type `Expense` refusé à la création : c'est le trou que
/// `create_in_tx` ne couvre **jamais** (`account_type` n'y est pas vérifié).
#[sqlx::test(migrations = "./test-schema")]
async fn create_rejects_non_revenue_account(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let expense = seeded.accounts["4000"];

    let err = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(expense))],
    )
    .await
    .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::NotRevenue);
}

/// **AC18** — même contrôle à la **modification** du brouillon.
#[sqlx::test(migrations = "./test-schema")]
async fn update_rejects_invalid_account(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let archived = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;
    set_account_active(&pool, archived, false).await;

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(100.00), dec!(0), None)])
        .await
        .expect("create draft");

    let err = invoices::update(
        &pool,
        seeded.company_id,
        invoice_id,
        1,
        seeded.admin_user_id,
        InvoiceUpdate {
            contact_id: contact,
            date: invoice_date(),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines: lines_from(&[(dec!(100.00), dec!(0), Some(archived))]),
        },
    )
    .await
    .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::Inactive);
}

/// **AC18** — plusieurs lignes en défaut simultanément : le message les nomme
/// **toutes**. Cas courant quand un compte partagé est archivé.
#[sqlx::test(migrations = "./test-schema")]
async fn create_reports_every_invalid_line(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let archived = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;
    set_account_active(&pool, archived, false).await;
    let expense = seeded.accounts["4000"];
    let ok = seeded.accounts["3000"];

    let err = create_draft(
        &pool,
        &seeded,
        contact,
        &[
            (dec!(100.00), dec!(0), Some(ok)),       // ligne 1 : valide
            (dec!(100.00), dec!(0), Some(archived)), // ligne 2 : archivé
            (dec!(100.00), dec!(0), Some(expense)),  // ligne 3 : mauvais type
        ],
    )
    .await
    .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected.len(), 2, "les DEUX lignes fautives sont remontées");
    assert_eq!(rejected[0].line_number, Some(2));
    assert_eq!(rejected[0].reason, RevenueAccountRejection::Inactive);
    assert_eq!(rejected[1].line_number, Some(3));
    assert_eq!(rejected[1].reason, RevenueAccountRejection::NotRevenue);
}

/// **AC18** — compte **retypé** entre le brouillon et la validation. Le trou que
/// `create_in_tx` ne couvre jamais : sans re-validation au posting, le produit
/// atterrirait sur un compte de charge, faux et sans bruit.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rejects_account_retyped_after_draft(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect("create draft");

    set_account_type(&pool, services, "Expense").await;

    let err =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected[0].line_number, Some(1));
    assert_eq!(rejected[0].reason, RevenueAccountRejection::NotRevenue);
}

/// **AC18** — compte devenu **non-imputable** entre le brouillon et la
/// validation. `create_in_tx` laisse passer (`enforce_postable = false` sur le
/// flux automatique).
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rejects_account_made_non_postable(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect("create draft");

    set_account_postable(&pool, services, false).await;

    let err =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::NotPostable);
}

/// **AC18** — compte archivé entre le brouillon et la validation.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rejects_account_archived_after_draft(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect("create draft");

    set_account_active(&pool, services, false).await;

    let err =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::Inactive);
}

/// **AC11-bis / D5-bis** — compte archivé **entre la validation et l'avoir** :
/// l'émission échoue, en nommant la ligne et le compte à réactiver, au lieu du
/// `400 INACTIVE_OR_INVALID_ACCOUNTS` générique.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_fails_when_snapshot_account_archived(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect("create draft");
    invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
        .await
        .expect("validate");

    set_account_active(&pool, services, false).await;

    let err = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: invoice_date(),
        },
        seeded.admin_user_id,
    )
    .await
    .expect_err("doit être refusé");

    match err {
        DbError::CreditNoteRevenueAccountsArchived(rejected) => {
            assert_eq!(rejected.len(), 1);
            assert_eq!(rejected[0].line_number, Some(1));
            assert_eq!(rejected[0].account_id, services);
            assert_eq!(rejected[0].account_number.as_deref(), Some("3200"));
        }
        other => panic!("attendu CreditNoteRevenueAccountsArchived, reçu {other:?}"),
    }
}

/// **D5-bis, portée** : `postable` et `account_type` ne sont **PAS** re-vérifiés
/// côté avoir. La contre-passation doit viser les mêmes comptes que l'écriture
/// d'origine, quelle qu'ait été leur évolution de configuration.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_ignores_postable_and_type_changes(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect("create draft");
    invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
        .await
        .expect("validate");

    set_account_postable(&pool, services, false).await;
    set_account_type(&pool, services, "Expense").await;

    let issued = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: invoice_date(),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("l'avoir doit passer : seule l'inactivité est bloquante");
    assert_eq!(debit_on(&issued.journal_entry, services), dec!(100.00));
}

// ---------------------------------------------------------------------------
// AC18-bis — le compte par défaut de la société (AC8-bis)
// ---------------------------------------------------------------------------

/// **AC18-bis cas 1** — facture à lignes **toutes `NULL`** et défaut société
/// **retypé** entre le brouillon et la validation → échec désignant le compte
/// par défaut, pas un numéro de ligne (aucune ligne ne le porte).
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rejects_retyped_company_default(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let default_revenue = seeded.accounts["3000"];

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(100.00), dec!(0), None)])
        .await
        .expect("create draft");

    set_account_type(&pool, default_revenue, "Expense").await;

    let err =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected.len(), 1);
    assert_eq!(
        rejected[0].line_number, None,
        "le compte par défaut n'est porté par aucune ligne"
    );
    assert_eq!(rejected[0].account_id, default_revenue);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::NotRevenue);
}

/// **AC18-bis cas 2** — défaut société rendu **non-imputable** : la validation
/// **passe** (exemption D3-bis) et l'écriture est générée normalement.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_accepts_non_postable_company_default(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let default_revenue = seeded.accounts["3000"];

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(100.00), dec!(0), None)])
        .await
        .expect("create draft");

    set_account_postable(&pool, default_revenue, false).await;

    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect("l'exemption D3-bis doit laisser passer");
    assert_eq!(
        credit_on(&validated.journal_entry, default_revenue),
        dec!(100.00)
    );
}

/// **AC18-bis cas 3** — défaut société **archivé**. L'échec doit désigner « le
/// compte de produit par défaut de la société », et **non** retomber sur le
/// `400 INACTIVE_OR_INVALID_ACCOUNTS` générique.
///
/// Ce cas avait été retiré en passe 3 de `validate` puis rétabli en passe 6 :
/// `get_or_create_default_in_tx` ne contrôle **pas** `active` (elle se réduit à
/// `INSERT IGNORE` + `SELECT … FOR UPDATE`), donc le défaut archivé n'est pas
/// rejeté en amont. C'est l'assertion sur l'identité du site rejeté qui
/// distingue AC8-bis implémenté d'AC8-bis absent.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rejects_archived_company_default(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let default_revenue = seeded.accounts["3000"];

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(100.00), dec!(0), None)])
        .await
        .expect("create draft");

    set_account_active(&pool, default_revenue, false).await;

    let err =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected.len(), 1);
    assert_eq!(
        rejected[0].line_number, None,
        "désigné comme le compte par défaut de la société, pas par un n° de ligne"
    );
    assert_eq!(rejected[0].account_id, default_revenue);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::Inactive);
}

// ---------------------------------------------------------------------------
// AC19 — invariant D3-bis
// ---------------------------------------------------------------------------

/// **AC19** — défaut société **non-imputable** : une ligne `NULL` et une ligne
/// le désignant **explicitement** ont le même verdict et produisent la **même**
/// écriture. Sans l'exemption, le geste le plus naturel (sélectionner le compte
/// déjà utilisé par défaut) serait rejeté pour un résultat comptable identique.
#[sqlx::test(migrations = "./test-schema")]
async fn null_and_explicit_default_are_equivalent(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let default_revenue = seeded.accounts["3000"];
    set_account_postable(&pool, default_revenue, false).await;

    // Facture A : ligne NULL. Facture B : ligne désignant explicitement 3000.
    let a = create_draft(&pool, &seeded, contact, &[(dec!(250.00), dec!(8.10), None)])
        .await
        .expect("création avec NULL acceptée");
    let b = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(250.00), dec!(8.10), Some(default_revenue))],
    )
    .await
    .expect("création avec le défaut explicite acceptée — exemption D3-bis");

    let va = invoices::validate_invoice(&pool, seeded.company_id, a, seeded.admin_user_id)
        .await
        .expect("validate A");
    let vb = invoices::validate_invoice(&pool, seeded.company_id, b, seeded.admin_user_id)
        .await
        .expect("validate B");

    // Même écriture : mêmes comptes, mêmes montants, même ordre.
    let shape = |je: &kesh_db::entities::JournalEntryWithLines| -> Vec<(i64, Decimal, Decimal)> {
        je.lines
            .iter()
            .map(|l| (l.account_id, l.debit, l.credit))
            .collect()
    };
    assert_eq!(shape(&va.journal_entry), shape(&vb.journal_entry));
}

/// **AC19 (suite)** — dans une **même** facture, une ligne `NULL` et une ligne
/// désignant le défaut **fusionnent** en une seule ligne de crédit.
#[sqlx::test(migrations = "./test-schema")]
async fn null_and_explicit_default_merge_into_one_credit_line(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let default_revenue = seeded.accounts["3000"];

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[
            (dec!(600.00), dec!(0), None),
            (dec!(400.00), dec!(0), Some(default_revenue)),
        ],
    )
    .await
    .expect("create draft");
    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect("validate");

    let revenue_lines = validated
        .journal_entry
        .lines
        .iter()
        .filter(|l| l.account_id == default_revenue)
        .count();
    assert_eq!(revenue_lines, 1, "une SEULE ligne de crédit produit");
    assert_eq!(
        credit_on(&validated.journal_entry, default_revenue),
        dec!(1000.00)
    );
}

// ---------------------------------------------------------------------------
// AC20 — garde-fou D1
// ---------------------------------------------------------------------------

/// **AC20** — le repli suit `settings.default_revenue_account_id`, **pas** le
/// compte portant le rôle `DefaultRevenue`.
///
/// Les deux comptes sont **délibérément distincts** : post-onboarding ils
/// coïncident, et un test qui ne les dissocie pas passerait tout aussi bien
/// avec une résolution par rôle — il ne prouverait rien.
#[sqlx::test(migrations = "./test-schema")]
async fn fallback_follows_settings_column_not_role(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    // Le rôle `DefaultRevenue` est porté par 3000…
    let role_holder = seeded.accounts["3000"];
    sqlx::query("UPDATE accounts SET role = 'DefaultRevenue' WHERE id = ?")
        .bind(role_holder)
        .execute(&pool)
        .await
        .unwrap();
    // …mais la colonne de configuration pointe sur un AUTRE compte.
    let configured = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;
    set_default_revenue(&pool, seeded.company_id, Some(configured)).await;

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(500.00), dec!(0), None)])
        .await
        .expect("create draft");
    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect("validate");

    assert_eq!(
        credit_on(&validated.journal_entry, configured),
        dec!(500.00),
        "le repli suit la COLONNE de configuration"
    );
    assert_eq!(
        credit_on(&validated.journal_entry, role_holder),
        Decimal::ZERO,
        "le rôle DefaultRevenue ne sert qu'au pré-remplissage à l'onboarding"
    );
}

// ---------------------------------------------------------------------------
// AC21 — D4-bis, pièce à montant nul
// ---------------------------------------------------------------------------

/// **AC21 / AC13-bis** — facture entièrement à zéro → erreur **métier**, et non
/// le 500 SQL sur `chk_jel_debit_credit_exclusive` que produisait le code avant
/// cette story.
#[sqlx::test(migrations = "./test-schema")]
async fn validate_rejects_zero_total_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(0.00), dec!(8.10), None), (dec!(0.00), dec!(0), None)],
    )
    .await
    .expect("un brouillon à zéro reste créable");

    let err =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect_err("doit être refusé");
    match err {
        DbError::InvalidInput(code) => assert_eq!(code, "invoiceTotalZero"),
        other => panic!("attendu InvalidInput(invoiceTotalZero), reçu {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC12-bis — non-régression de la saisie
// ---------------------------------------------------------------------------

/// **AC12-bis** — création et modification d'un brouillon réussissent quand le
/// défaut société est **archivé**, tant qu'aucune ligne ne référence
/// explicitement un compte invalide.
///
/// Note de portée : ce test **ne prouve pas** que la saisie évite
/// `get_or_create_default_in_tx` — cette fonction ne contrôle pas `active`,
/// donc elle n'échouerait pas non plus ici. La garantie « pas de lazy-create ni
/// de verrou sur `company_invoice_settings` à la saisie » relève de la revue de
/// code ; ce qui est vérifié ici, c'est l'absence d'écriture (assertion sur la
/// table, ci-dessous) et la non-régression fonctionnelle.
#[sqlx::test(migrations = "./test-schema")]
async fn draft_crud_survives_archived_company_default(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    set_account_active(&pool, seeded.accounts["3000"], false).await;

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(100.00), dec!(0), None)])
        .await
        .expect("création OK malgré un défaut archivé");

    invoices::update(
        &pool,
        seeded.company_id,
        invoice_id,
        1,
        seeded.admin_user_id,
        InvoiceUpdate {
            contact_id: contact,
            date: invoice_date(),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines: lines_from(&[(dec!(200.00), dec!(0), None)]),
        },
    )
    .await
    .expect("modification OK malgré un défaut archivé");
}

/// **AC12-bis (suite)** — même chose quand `company_invoice_settings` n'a
/// **aucune ligne** pour la société : la saisie doit passer, et surtout ne pas
/// en créer une (le lazy-create appartient au posting, pas à la saisie).
#[sqlx::test(migrations = "./test-schema")]
async fn draft_creation_neither_reads_nor_creates_settings_row(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    sqlx::query("DELETE FROM company_invoice_settings WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();

    create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect("création OK sans ligne de configuration");

    let rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM company_invoice_settings WHERE company_id = ?")
            .bind(seeded.company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        rows, 0,
        "la saisie ne doit PAS créer la ligne de configuration \
         (pas de get_or_create_default_in_tx sur ce chemin)"
    );
}

/// **AC7 (suite)** — sans ligne de configuration, l'exemption D3-bis ne
/// s'applique à aucun compte : un compte non-imputable est alors refusé.
#[sqlx::test(migrations = "./test-schema")]
async fn missing_settings_row_disables_postable_exemption(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;
    set_account_postable(&pool, services, false).await;

    sqlx::query("DELETE FROM company_invoice_settings WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect_err("doit être refusé");
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::NotPostable);
}

// ---------------------------------------------------------------------------
// Modification du seul compte d'une ligne
// ---------------------------------------------------------------------------

/// Changer **uniquement** le compte de produit d'une ligne n'est pas un no-op :
/// sans la prise en compte du champ dans le court-circuit KF-004, la
/// modification serait silencieusement perdue avec un `200 OK`.
#[sqlx::test(migrations = "./test-schema")]
async fn changing_only_the_account_is_not_a_no_op(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(100.00), dec!(0), None)])
        .await
        .expect("create draft");

    let (invoice, lines) = invoices::update(
        &pool,
        seeded.company_id,
        invoice_id,
        1,
        seeded.admin_user_id,
        InvoiceUpdate {
            contact_id: contact,
            date: invoice_date(),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines: lines_from(&[(dec!(100.00), dec!(0), Some(services))]),
        },
    )
    .await
    .expect("update");

    assert_eq!(invoice.version, 2, "la version doit être incrémentée");
    assert_eq!(lines[0].revenue_account_id, Some(services));
}

/// Revue de code passe 1 (2026-07-28) — le snapshot d'audit `before` doit
/// témoigner de l'état **avant** matérialisation.
///
/// `validate_invoice` mute `lines_before` en mémoire pour refléter l'`UPDATE`
/// de matérialisation. Tant que la même variable alimentait les deux clés du
/// snapshot, `before` et `after` affichaient tous deux le compte matérialisé :
/// la transition `NULL` → compte effectif — la seule que cette story
/// introduit — était irrécupérable depuis le journal d'audit.
#[sqlx::test(migrations = "./test-schema")]
async fn audit_before_snapshot_predates_materialization(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let default_revenue = seeded.accounts["3000"];
    let explicit = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    // Ligne 1 sans compte (sera matérialisée), ligne 2 avec un compte explicite
    // (doit rester identique des deux côtés du snapshot).
    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[
            (dec!(1000.00), dec!(8.10), None),
            (dec!(500.00), dec!(8.10), Some(explicit)),
        ],
    )
    .await
    .expect("create draft");

    invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
        .await
        .expect("validate");

    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE action = 'invoice.validated' AND entity_type = 'invoice' AND entity_id = ?",
    )
    .bind(invoice_id)
    .fetch_one(&pool)
    .await
    .expect("une entrée d'audit invoice.validated");

    let account_at = |side: &str, idx: usize| -> serde_json::Value {
        details[side]["lines"][idx]["revenueAccountId"].clone()
    };

    assert!(
        account_at("before", 0).is_null(),
        "before.lines[0] doit rester NULL — c'est l'état d'avant matérialisation ; \
         obtenu {:?}",
        account_at("before", 0)
    );
    assert_eq!(
        account_at("after", 0).as_i64(),
        Some(default_revenue),
        "after.lines[0] doit porter le compte matérialisé"
    );

    // La ligne au compte explicite est le témoin : elle prouve que `before`
    // n'est pas simplement « tout à NULL » par accident de sérialisation.
    assert_eq!(
        account_at("before", 1).as_i64(),
        Some(explicit),
        "before.lines[1] doit conserver le compte choisi explicitement"
    );
    assert_eq!(
        account_at("after", 1).as_i64(),
        Some(explicit),
        "after.lines[1] doit être inchangé"
    );
}

/// **AC12-bis** (revue de code passe 2) — ligne de configuration **présente**
/// mais colonne `default_revenue_account_id` à `NULL`.
///
/// Distinct de `draft_creation_neither_reads_nor_creates_settings_row`, qui
/// supprime la ligne entière : c'est ici la branche que le `Option<Option<i64>>`
/// + `.flatten()` de `read_default_revenue_account_id` existe pour absorber.
/// Aucun test ne la couvrait.
///
/// **Ce qui est propre à ce test, c'est son montage** — ligne de configuration
/// **présente** et colonne à `NULL` — et non l'une ou l'autre de ses assertions.
/// La lecture du défaut est déclenchée par la seule présence d'un compte
/// explicite sur une ligne (`invoices.rs:641`, `if !sites.is_empty()`), donc
/// **les deux** assertions traversent `read_default_revenue_account_id` et
/// décodent ce `NULL` en `Option<Option<i64>>`. Un refactor qui ramènerait ce
/// type à un `Option<i64>` nu ferait échouer les deux.
///
/// Les deux assertions vérifient des choses différentes ; aucune n'est
/// redondante :
///
/// 1. un compte explicite **imputable** est accepté et persisté — la saisie ne
///    dépend pas du défaut société ;
/// 2. un compte explicite **non imputable** est **rejeté** : l'exemption D3-bis
///    exige un défaut auquel se comparer, et il n'y en a pas. Cette assertion
///    épingle l'**équivalence voulue** avec « ligne absente » — `.flatten()`
///    existe précisément pour faire converger les deux branches sur le même
///    `None` — et attraperait un refactor qui accorderait l'exemption au seul
///    motif que la ligne de configuration existe. Miroir de
///    `missing_settings_row_disables_postable_exemption`.
///
/// *(Historique de ce commentaire, instructif en soi : la passe 3 de revue a
/// écrit que l'assertion 2 était « discriminante » — inversé ; la passe 4 a
/// répondu que l'assertion 1 était « la seule » à exercer le décodage —
/// sur-exclusif, et faux pour la même raison. Deux rédactions successives
/// affirmant une propriété que personne n'avait vérifiée contre le code. Celle-ci
/// l'a été, en passe 5.)*
#[sqlx::test(migrations = "./test-schema")]
async fn draft_crud_survives_null_company_default_column(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let services = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    // La ligne existe, la colonne est vidée.
    set_default_revenue(&pool, seeded.company_id, None).await;

    // (1) Compte imputable → accepté et persisté.
    let invoice_id = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(services))],
    )
    .await
    .expect("la saisie ne doit pas dépendre du défaut société");

    let persisted: Vec<Option<i64>> = sqlx::query_scalar(
        "SELECT revenue_account_id FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
    )
    .bind(invoice_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        persisted,
        vec![Some(services)],
        "le compte explicite doit être persisté malgré l'absence de défaut"
    );

    // (2) L'équivalence voulue : colonne `NULL` ⇒ aucune exemption D3-bis
    // possible, exactement comme « ligne absente ».
    let marchandises =
        add_account(&pool, seeded.company_id, "3400", "Marchandises", "Revenue").await;
    set_account_postable(&pool, marchandises, false).await;

    let err = create_draft(
        &pool,
        &seeded,
        contact,
        &[(dec!(100.00), dec!(0), Some(marchandises))],
    )
    .await
    .expect_err(
        "un compte non imputable doit être refusé : sans défaut société, \
                 il n'existe aucun compte auquel l'exemption D3-bis puisse s'appliquer",
    );
    let rejected = expect_invalid_accounts(err);
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].reason, RevenueAccountRejection::NotPostable);
}

/// **Frontière du résidu non reprenable** (revue de code passe 2, décision Guy
/// du 2026-07-28 ; **recadré à la livraison de 16-1a-bis**) — ce test **ne
/// corrige rien**, il fixe la limite connue.
///
/// Une ligne dont `revenue_account_id` est `NULL` fait se replier l'avoir sur le
/// compte par défaut **au moment de l'avoir**. Si l'administrateur a changé ce
/// défaut entre-temps, la contre-passation débite un autre compte que celui que
/// la facture a crédité — résidu permanent, bilan équilibré, compte de résultat
/// faux, **et aucune erreur** puisque le nouveau compte est actif.
///
/// # Ce que la livraison de 16-1a-bis a changé, et ce qu'elle n'a pas changé
///
/// La rédaction d'origine annonçait « le jour où le backfill est livré, ce test
/// DOIT changer de verdict ». **Ce n'est pas ce qui se produit, et c'est
/// correct** : le test remet la ligne à `NULL` *après* que les migrations ont
/// tourné, donc le backfill ne la voit jamais. Le comportement décrit ici n'est
/// pas celui du parc antérieur — c'est celui d'**une ligne `NULL`, quelle qu'en
/// soit la raison**.
///
/// Ce qui a changé, c'est la **population** concernée. Avant 16-1a-bis :
/// l'intégralité du parc validé. Depuis : uniquement les lignes que le backfill
/// a **délibérément** laissées `NULL`, faute de pouvoir identifier le compte
/// sans ambiguïté — écriture retouchée à la main, zéro ou plusieurs candidats
/// (cf. `invoice_lines_revenue_account_backfill.rs`, décision D-B2). Sur ces
/// lignes-là, le repli sur le défaut courant reste le comportement, et reste
/// une limitation assumée : la seule alternative serait de deviner un compte
/// sur une pièce comptable réelle.
#[sqlx::test(migrations = "./test-schema")]
async fn null_line_credit_note_falls_back_to_current_default_known_limitation(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let original_default = seeded.accounts["3000"];
    let new_default = add_account(&pool, seeded.company_id, "3200", "Prestations", "Revenue").await;

    let invoice_id = create_draft(&pool, &seeded, contact, &[(dec!(1000.00), dec!(0), None)])
        .await
        .expect("create draft");
    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
            .await
            .expect("validate");
    assert_eq!(
        credit_on(&validated.journal_entry, original_default),
        dec!(1000.00),
        "la facture crédite le défaut en vigueur à sa validation"
    );

    // Simule une ligne que le backfill de 16-1a-bis ne peut pas reprendre : on
    // efface la matérialisation que 16-1a vient de poser. Noter que le backfill
    // a déjà tourné (à la création de la base éphémère), donc cet effacement
    // n'est jamais rattrapé — c'est bien l'état résiduel qui est testé ici.
    sqlx::query("UPDATE invoice_lines SET revenue_account_id = NULL WHERE invoice_id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();

    // L'administrateur change le compte de produit par défaut.
    set_default_revenue(&pool, seeded.company_id, Some(new_default)).await;

    let credit_note = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: invoice_date(),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("l'avoir est émis sans erreur — c'est précisément le problème");

    assert_eq!(
        debit_on(&credit_note.journal_entry, new_default),
        dec!(1000.00),
        "LIMITATION CONNUE (résiduelle depuis 16-1a-bis) : sur une ligne restée \
         `NULL`, l'avoir débite le défaut COURANT (3200), pas le compte que la \
         facture a réellement crédité (3000). Ne concerne plus que les pièces \
         dont l'écriture a été retouchée à la main, que le backfill refuse \
         délibérément de reprendre (D-B2)."
    );
    assert_eq!(
        debit_on(&credit_note.journal_entry, original_default),
        Decimal::ZERO,
        "le compte historiquement crédité n'est PAS extourné — résidu permanent"
    );

    // Le résidu, chiffré : 3000 reste créditeur de 1000, 3200 débiteur de 1000.
    let net = net_by_account(&[&validated.journal_entry, &credit_note.journal_entry]);
    assert_eq!(
        net.get(&original_default).copied().unwrap_or(Decimal::ZERO),
        dec!(-1000.00),
        "résidu créditeur sur le compte d'origine"
    );
    assert_eq!(
        net.get(&new_default).copied().unwrap_or(Decimal::ZERO),
        dec!(1000.00),
        "résidu débiteur sur le nouveau défaut — les deux s'annulent au bilan, \
         d'où l'invisibilité du défaut"
    );
}
