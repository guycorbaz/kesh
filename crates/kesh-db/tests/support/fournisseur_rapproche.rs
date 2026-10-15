//! Aide de montage **réservée aux tests** — Story 30-1a (#686) : un règlement
//! fournisseur **rapproché** d'une transaction bancaire au débit.
//!
//! ⛔ **Pourquoi un état posé et non produit** : avant la Story 30-1b, aucun
//! chemin ne rapproche un débit d'un règlement fournisseur. Le décor règle donc
//! la facture **par le vrai chemin** (`supplier_invoices::pay`, virement depuis
//! le compte de l'import), puis pose le lien en SQL
//! ([`link_bank_transaction_for_test`]). La 30-1b rejouera ces tests par
//! l'acceptation.
//!
//! ⚠️ **Hors de `crates/*/src`, délibérément** (C-30-1-43) : l'inventaire des
//! écrivains d'une ligne rattachée à un compte bancaire
//! (`kesh-api/tests/bank_account_closed_inventory.rs`) balaie le code de
//! `src/`, fixtures comprises ; une aide qui ne prétend à rien d'atteignable
//! n'a pas à y être classée.
//!
//! Ce fichier n'appartient à aucune crate ; il est **inclus par `#[path]`** :
//! `crates/kesh-db/tests/reconciliation_cancel.rs` et
//! `crates/kesh-api/tests/reconciliation_e2e.rs`. Décor **synthétique** (aucune
//! donnée réelle, `docs/agents-orchestration.md` § 8).
#![allow(dead_code)]

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::{
    NewBankAccount, NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice,
};
use kesh_db::repositories::{bank_accounts, contacts, supplier_invoices};
use kesh_db::test_fixtures::SeededCompany;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Le fournisseur et le compte bancaire de l'import, lié au compte `1100` du
/// grand livre.
#[derive(Debug, Clone, Copy)]
pub struct Fournisseur {
    pub supplier_id: i64,
    pub bank_account_id: i64,
}

/// Une facture fournisseur réglée par virement, puis rapprochée.
#[derive(Debug, Clone, Copy)]
pub struct ReglementRapproche {
    pub supplier_invoice_id: i64,
    pub purchase_entry_id: i64,
    pub settlement_entry_id: i64,
    pub bank_transaction_id: i64,
}

/// Le compte créanciers (`2000`) aux réglages, un fournisseur, et le compte
/// bancaire de l'import lié au `1100`.
pub async fn preparer(pool: &MySqlPool, seeded: &SeededCompany) -> Fournisseur {
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .expect("compte créanciers");
    let supplier_id = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Fournisseur rapproché SA".into(),
            first_name: None,
            last_name: None,
            is_client: false,
            is_supplier: true,
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
            salutation: Salutation::Neutre,
        },
    )
    .await
    .expect("fournisseur")
    .id;
    let bank = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque du rapprochement".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .expect("compte bancaire");
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(seeded.accounts["1100"])
        .bind(bank.id)
        .execute(pool)
        .await
        .expect("lien au grand livre");
    Fournisseur {
        supplier_id,
        bank_account_id: bank.id,
    }
}

/// Une facture fournisseur de `montant` (sans TVA) datée `jour`, réglée le même
/// jour par virement depuis le compte de l'import ; rend
/// `(facture, écriture d'achat, écriture de règlement)`.
pub async fn facture_payee(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
    jour: NaiveDate,
    montant: Decimal,
) -> (i64, i64, i64) {
    let created = supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id: f.supplier_id,
            supplier_invoice_number: Some(numero.into()),
            invoice_date: jour,
            due_date: Some(jour),
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: montant,
                vat_rate: dec!(0),
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .expect("facture fournisseur");
    let paid = supplier_invoices::pay(
        pool,
        seeded.company_id,
        created.invoice.id,
        SettlementChoice::BankTransfer {
            bank_account_id: f.bank_account_id,
        },
        jour,
        seeded.admin_user_id,
    )
    .await
    .expect("règlement par virement");
    (
        created.invoice.id,
        created.invoice.purchase_journal_entry_id,
        paid.invoice
            .settlement_journal_entry_id
            .expect("écriture de règlement"),
    )
}

/// Une transaction au **débit** de `montant` le `jour`, sur le compte de
/// l'import, rapprochée de l'écriture `entry_id`.
pub async fn debit_rapproche(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    entry_id: i64,
    jour: NaiveDate,
    montant: Decimal,
) -> i64 {
    let import_id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, \
          period_to, imported_by_user_id) \
         VALUES (?, ?, 'releve.xml', SHA2(UUID(), 256), 'camt053', ?, ?, ?)",
    )
    .bind(seeded.company_id)
    .bind(f.bank_account_id)
    .bind(jour)
    .bind(jour)
    .bind(seeded.admin_user_id)
    .execute(pool)
    .await
    .expect("import")
    .last_insert_id() as i64;
    let tx_id = sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, value_date, amount, currency, \
          details, counterparty_name) \
         VALUES (?, ?, ?, ?, ?, ?, 'CHF', 'Paiement fournisseur', 'Fournisseur rapproché SA')",
    )
    .bind(seeded.company_id)
    .bind(import_id)
    .bind(f.bank_account_id)
    .bind(jour)
    .bind(jour)
    .bind(-montant)
    .execute(pool)
    .await
    .expect("transaction au débit")
    .last_insert_id() as i64;
    link_bank_transaction_for_test(pool, tx_id, entry_id).await;
    tx_id
}

/// Rapproche la transaction bancaire `bank_transaction_id` de l'écriture
/// `entry_id` **en SQL brut** : l'**état** que produit un rapprochement, posé
/// sans passer par le geste qui le produit.
///
/// ⛔ **Assertion de montage** : exactement une ligne modifiée, sans quoi un
/// identifiant faux laisserait le décor vide et les tests verts à vide.
pub async fn link_bank_transaction_for_test(
    pool: &MySqlPool,
    bank_transaction_id: i64,
    entry_id: i64,
) {
    let done = sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ?, \
         version = version + 1 WHERE id = ?",
    )
    .bind(entry_id)
    .bind(bank_transaction_id)
    .execute(pool)
    .await
    .expect("lien posé");
    assert_eq!(
        done.rows_affected(),
        1,
        "montage : la transaction {bank_transaction_id} doit exister"
    );
}

/// [`facture_payee`] puis [`debit_rapproche`] : l'état que produira
/// l'acceptation de la 30-1b.
pub async fn reglement_rapproche(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
    jour: NaiveDate,
    montant: Decimal,
) -> ReglementRapproche {
    let (supplier_invoice_id, purchase_entry_id, settlement_entry_id) =
        facture_payee(pool, seeded, f, numero, jour, montant).await;
    let bank_transaction_id =
        debit_rapproche(pool, seeded, f, settlement_entry_id, jour, montant).await;
    ReglementRapproche {
        supplier_invoice_id,
        purchase_entry_id,
        settlement_entry_id,
        bank_transaction_id,
    }
}

/// Solde net d'un compte (Σ débit − Σ crédit) sur toutes les écritures.
pub async fn solde(pool: &MySqlPool, company_id: i64, account_id: i64) -> Decimal {
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
