//! Story 43-1b (#823) — un règlement pour plusieurs factures **fournisseur** :
//! le paiement groupé (`pay_supplier_invoices_in_tx`), sa vue, son
//! dé-rapprochement, le détachement d'une facture du groupe.
//!
//! Binaire à part du bras client (`grouped_settlements.rs`, Story 43-1a), pour
//! que les deux stories avancent sans se disputer le même fichier (choix
//! C-e431b-1). Tous les tests montent le squash (`test-schema`), que
//! `test_schema_guard.rs` prouve égal au schéma de `MIGRATOR`. Décor
//! synthétique.

use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice};
use kesh_db::errors::{ClaimSide, DbError, SettlementAccountRole};
use kesh_db::repositories::grouped_settlements::{
    self, GroupFamily, GroupRefusal, GroupedDocument, GroupedSettlement, IneligibleReason,
};
use kesh_db::repositories::{reconciliation_cancel, supplier_invoices};
use kesh_db::test_fixtures::{SeededCompany, designate_rounding_account, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Un IBAN suisse de test valide (le créancier des factures, le débiteur du
/// compte bancaire).
const IBAN: &str = "CH9300762011623852957";

fn jour(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, m, d).expect("date valide")
}

/// La date du jour des gestes (dé-rapprochement, annulation).
fn aujourd_hui() -> NaiveDate {
    jour(11, 17)
}

/// Le montage : la société du seed, un compte de banque `1020` lié au compte
/// bancaire, un compte de dette `2010` désigné créanciers, un compte d'écart
/// `6950` choisi par l'utilisateur.
struct Montage {
    s: SeededCompany,
    bank_account_id: i64,
    bank_ledger: i64,
    payable: i64,
    ecart: i64,
}

async fn compte(pool: &MySqlPool, company_id: i64, number: &str, kind: &str) -> i64 {
    sqlx::query("INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)")
        .bind(company_id)
        .bind(number)
        .bind(format!("Compte {number}"))
        .bind(kind)
        .execute(pool)
        .await
        .expect("compte")
        .last_insert_id() as i64
}

async fn monter(pool: &MySqlPool) -> Montage {
    let s = seed_accounting_company(pool).await.expect("seed");
    let bank_ledger = compte(pool, s.company_id, "1020", "Asset").await;
    let payable = compte(pool, s.company_id, "2010", "Liability").await;
    let ecart = compte(pool, s.company_id, "6950", "Expense").await;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(payable)
    .bind(s.company_id)
    .execute(pool)
    .await
    .expect("compte créanciers");
    let bank_account_id = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'Banque test', ?, ?)",
    )
    .bind(s.company_id)
    .bind(IBAN)
    .bind(bank_ledger)
    .execute(pool)
    .await
    .expect("compte bancaire")
    .last_insert_id() as i64;
    Montage {
        s,
        bank_account_id,
        bank_ledger,
        payable,
        ecart,
    }
}

/// Une facture fournisseur validée de `amount`, d'un fournisseur neuf (des
/// tiers différents), datée `date`.
async fn facture(
    pool: &MySqlPool,
    m: &Montage,
    numero: &str,
    amount: Decimal,
    date: NaiveDate,
    project_id: Option<i64>,
) -> i64 {
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
         VALUES (?, 'Entreprise', ?, TRUE) RETURNING id",
    )
    .bind(m.s.company_id)
    .bind(format!("Fournisseur {numero}"))
    .fetch_one(pool)
    .await
    .expect("fournisseur");
    supplier_invoices::create_validated(
        pool,
        NewSupplierInvoice {
            company_id: m.s.company_id,
            contact_id,
            supplier_invoice_number: Some(numero.into()),
            invoice_date: date,
            due_date: Some(date),
            creditor_iban: Some(IBAN.into()),
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id,
            lines: vec![NewSupplierInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: amount,
                vat_rate: dec!(0),
                expense_account_id: m.s.accounts["4000"],
            }],
        },
        m.s.admin_user_id,
    )
    .await
    .expect("facture fournisseur validée")
    .invoice
    .id
}

/// Porte le total d'une facture **et** les lignes de son écriture d'achat à
/// `total` (quatre décimales) — l'état qu'aucune saisie au centime ne produit,
/// mais que `DECIMAL(19,4)` admet (cas `r ≠ 0` de la décision).
async fn fixer_total(pool: &MySqlPool, id: i64, total: Decimal) {
    let purchase: i64 =
        sqlx::query_scalar("SELECT purchase_journal_entry_id FROM supplier_invoices WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap();
    sqlx::query("UPDATE supplier_invoices SET total_amount = ? WHERE id = ?")
        .bind(total)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    let n = sqlx::query(
        "UPDATE journal_entry_lines SET debit = IF(debit > 0, ?, 0), credit = IF(credit > 0, ?, 0) \
         WHERE entry_id = ?",
    )
    .bind(total)
    .bind(total)
    .bind(purchase)
    .execute(pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 2, "l'écriture d'achat a deux lignes");
}

/// Une transaction `pending` en CHF de `amount` (signé) sur le compte du
/// montage.
async fn transaction(pool: &MySqlPool, m: &Montage, amount: Decimal, on: NaiveDate) -> i64 {
    let import_id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, \
          period_to, imported_by_user_id) \
         VALUES (?, ?, 'test.xml', SHA2(UUID(), 256), 'camt053', ?, ?, ?)",
    )
    .bind(m.s.company_id)
    .bind(m.bank_account_id)
    .bind(on)
    .bind(on)
    .bind(m.s.admin_user_id)
    .execute(pool)
    .await
    .expect("import")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details) \
         VALUES (?, ?, ?, ?, ?, 'CHF', 'paiement groupé')",
    )
    .bind(m.s.company_id)
    .bind(import_id)
    .bind(m.bank_account_id)
    .bind(on)
    .bind(amount)
    .execute(pool)
    .await
    .expect("transaction")
    .last_insert_id() as i64
}

fn acteur(m: &Montage) -> Acteur {
    Acteur::utilisateur(m.s.admin_user_id)
}

/// Paie le groupe dans une transaction, **la valide** (refus compris : le
/// point de sauvegarde doit avoir tout défait), rend le résultat.
async fn payer_par(
    pool: &MySqlPool,
    m: &Montage,
    bt: i64,
    ids: &[i64],
    difference_account_id: Option<i64>,
    par: Acteur,
) -> Result<GroupedSettlement, GroupRefusal> {
    let mut tx = pool.begin().await.expect("transaction");
    let out = grouped_settlements::pay_supplier_invoices_in_tx(
        &mut tx,
        m.s.company_id,
        m.bank_account_id,
        bt,
        ids,
        difference_account_id,
        1,
        par,
        aujourd_hui(),
    )
    .await
    .expect("aucune erreur de base");
    tx.commit().await.expect("commit");
    out
}

async fn payer(
    pool: &MySqlPool,
    m: &Montage,
    bt: i64,
    ids: &[i64],
    difference_account_id: Option<i64>,
) -> Result<GroupedSettlement, GroupRefusal> {
    payer_par(pool, m, bt, ids, difference_account_id, acteur(m)).await
}

/// Une empreinte des tables que le geste écrit.
async fn empreinte(pool: &MySqlPool) -> Vec<i64> {
    let mut out = Vec::new();
    for sql in [
        "SELECT COUNT(*) FROM journal_entries",
        "SELECT COUNT(*) FROM journal_entry_lines",
        "SELECT COUNT(*) FROM audit_log",
        "SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key IS NOT NULL",
        "SELECT CAST(COALESCE(SUM(version), 0) AS SIGNED) FROM supplier_invoices",
        "SELECT COUNT(*) FROM supplier_invoices WHERE status = 'paid'",
        "SELECT COUNT(*) FROM supplier_invoices WHERE settlement_journal_entry_line_id IS NOT NULL",
        "SELECT CAST(COALESCE(SUM(version), 0) AS SIGNED) FROM bank_transactions",
        "SELECT COUNT(*) FROM bank_transactions WHERE matched_entry_id IS NOT NULL",
    ] {
        out.push(
            sqlx::query_scalar::<_, i64>(sql)
                .fetch_one(pool)
                .await
                .expect(sql),
        );
    }
    out
}

/// Les lignes d'une écriture : `(id, compte, débit, crédit, projet, clé de
/// lettrage)`, dans l'ordre.
type Ligne = (i64, i64, Decimal, Decimal, Option<i64>, Option<i64>);

async fn lignes(pool: &MySqlPool, entry_id: i64) -> Vec<Ligne> {
    sqlx::query_as(
        "SELECT id, account_id, debit, credit, project_id, lettering_key \
         FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

fn equilibree(l: &[Ligne]) -> bool {
    let d: Decimal = l.iter().map(|x| x.2).sum();
    let c: Decimal = l.iter().map(|x| x.3).sum();
    d == c
}

/// `(statut, écriture, ligne, compte bancaire, type, paid_at, version)`.
type EtatFacture = (
    String,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<chrono::NaiveDateTime>,
    i32,
);

async fn etat(pool: &MySqlPool, id: i64) -> EtatFacture {
    sqlx::query_as(
        "SELECT status, settlement_journal_entry_id, settlement_journal_entry_line_id, \
         settlement_bank_account_id, settlement_type, paid_at, version \
         FROM supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Le numéro affichable d'une facture : l'interne (posé à la validation), à
/// défaut celui du fournisseur.
async fn numero(pool: &MySqlPool, id: i64) -> Option<String> {
    sqlx::query_scalar(
        "SELECT COALESCE(internal_number, supplier_invoice_number) FROM supplier_invoices \
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn cle_api(pool: &MySqlPool, m: &Montage) -> Acteur {
    let key = sqlx::query(
        "INSERT INTO api_keys (company_id, created_by_user_id, name, key_hash, scope) \
         VALUES (?, ?, 'cle', SHA2('cle-fournisseur', 256), 'read-write')",
    )
    .bind(m.s.company_id)
    .bind(m.s.admin_user_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    Acteur {
        user_id: m.s.admin_user_id,
        api_key_id: Some(key),
    }
}

async fn dernier_audit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Toutes les entrées d'audit écrites depuis `avant` portent la clé : `(total,
/// avec la clé)`.
async fn audit_par_cle(pool: &MySqlPool, avant: i64, cle: Acteur) -> (i64, i64) {
    sqlx::query_as(
        "SELECT COUNT(*), CAST(COALESCE(SUM(actor_api_key_id <=> ?), 0) AS SIGNED) \
         FROM audit_log WHERE id > ?",
    )
    .bind(cle.api_key_id)
    .bind(avant)
    .fetch_one(pool)
    .await
    .unwrap()
}

// ---------------------------------------------------------------------------
// AC1 à AC5 — le paiement groupé
// ---------------------------------------------------------------------------

/// AC1 à AC5 — deux factures de deux fournisseurs, payées au montant exact par
/// un débit, **par une clé d'API** : une écriture (banque au crédit, une dette
/// au débit par facture, par identifiant croissant), chaque facture `paid` avec
/// SA ligne, lettrée avec son achat seul ; la transaction rapprochée ; l'audit
/// du rapprochement puis un `supplier_invoice.paid` par facture, tous à la clé.
#[sqlx::test(migrations = "./test-schema")]
async fn two_suppliers_are_paid_and_lettered_one_by_one(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "FA-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "FA-2", dec!(200.00), jour(6, 2), None).await;
    let bt = transaction(&pool, &m, dec!(-300.00), jour(6, 15)).await;
    let versions = (etat(&pool, a).await.6, etat(&pool, b).await.6);
    let cle = cle_api(&pool, &m).await;
    let avant = dernier_audit(&pool).await;

    let done = payer_par(&pool, &m, bt, &[b, a], None, cle)
        .await
        .expect("payé");
    assert_eq!(
        done.invoices.iter().map(|i| i.id).collect::<Vec<_>>(),
        vec![a, b],
        "par identifiant croissant"
    );
    assert!(
        done.invoices
            .iter()
            .all(|i| i.kind == GroupFamily::Supplier)
    );
    assert!(done.invoices.iter().all(|i| i.settlement_id.is_none()));
    assert_eq!(done.invoices[0].amount, dec!(100.00));
    assert_eq!(done.decomposition.gap, Decimal::ZERO);
    assert_eq!(done.difference_account_id, None);

    let l = lignes(&pool, done.journal_entry_id).await;
    assert_eq!(l.len(), 3, "banque et deux dettes, ni arrondi ni écart");
    assert_eq!(
        (l[0].1, l[0].2, l[0].3),
        (m.bank_ledger, dec!(0), dec!(300.00))
    );
    assert_eq!((l[1].1, l[1].2, l[1].3), (m.payable, dec!(100.00), dec!(0)));
    assert_eq!((l[2].1, l[2].2, l[2].3), (m.payable, dec!(200.00), dec!(0)));
    assert_eq!(l[0].5, None, "la banque n'est pas lettrée");

    for (r, version) in done.invoices.iter().zip([versions.0, versions.1]) {
        let e = etat(&pool, r.id).await;
        assert_eq!(e.0, "paid");
        assert_eq!(e.1, Some(done.journal_entry_id));
        assert_eq!(e.2, Some(r.line_id), "la facture porte SA ligne");
        assert_eq!(e.3, Some(m.bank_account_id));
        assert_eq!(e.4.as_deref(), Some("bank_transfer"));
        assert_eq!(e.5, Some(jour(6, 15).and_hms_opt(0, 0, 0).unwrap()));
        assert_eq!(e.6, version + 1);
        // Une clé de lettrage par facture : son achat et sa seule ligne.
        let key = l
            .iter()
            .find(|x| x.0 == r.line_id)
            .and_then(|x| x.5)
            .expect("ligne de dette lettrée");
        let membres: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = ?")
                .bind(key)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(membres, 2, "l'achat et la seule ligne de la facture");
    }
    assert_ne!(l[1].5, l[2].5, "une clé par facture");

    let matched: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(bt)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(matched, Some(done.journal_entry_id));
    let accepted: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.accepted' \
         AND entity_id = ?",
    )
    .bind(bt)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(accepted["proposal_type"], "supplier_invoice_group");
    assert_eq!(accepted["batch_size"], 1);
    assert_eq!(
        accepted["supplier_invoices"].as_array().map(Vec::len),
        Some(2)
    );
    // Le rapprochement d'abord, puis un paiement par facture qui le cite.
    let paid: Vec<(i64, serde_json::Value)> = sqlx::query_as(
        "SELECT entity_id, details_json FROM audit_log WHERE action = 'supplier_invoice.paid' \
         AND id > ? ORDER BY id",
    )
    .bind(done.reconciliation_audit_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(paid.iter().map(|p| p.0).collect::<Vec<_>>(), vec![a, b]);
    for (p, r) in paid.iter().zip(&done.invoices) {
        assert_eq!(p.1["reconciliationAuditId"], done.reconciliation_audit_id);
        assert_eq!(p.1["settlementJournalEntryLineId"], r.line_id);
        assert_eq!(p.1["grouped"], true);
    }
    // Chaque entrée écrite par le geste porte la clé.
    let (total, avec_cle) = audit_par_cle(&pool, avant, cle).await;
    assert!(
        total >= 4,
        "écriture, rapprochement, deux paiements : {total}"
    );
    assert_eq!(total, avec_cle);
}

/// AC2 — l'écart dans les deux sens, au compte choisi : plus payé (`e > 0`) au
/// **débit**, moins payé (`e < 0`) au **crédit** ; l'écriture s'équilibre ;
/// l'audit porte `gap` et le compte. Au-delà du plafond : refusé, **même sans
/// compte** (le plafond d'abord) ; un écart sans compte : refusé. Un refus ne
/// laisse rien.
#[sqlx::test(migrations = "./test-schema")]
async fn the_gap_is_admitted_both_ways_on_the_chosen_account(pool: MySqlPool) {
    let m = monter(&pool).await;
    // Plus payé que les factures : 150.30 pour 150.00.
    let a = facture(&pool, &m, "E-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "E-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.30), jour(6, 15)).await;

    // Sans compte : refusé, rien d'écrit.
    let avant = empreinte(&pool).await;
    assert_eq!(
        payer(&pool, &m, bt, &[a, b], None).await,
        Err(GroupRefusal::DifferenceAccountRequired { gap: dec!(0.30) })
    );
    assert_eq!(empreinte(&pool).await, avant);

    let done = payer(&pool, &m, bt, &[a, b], Some(m.ecart))
        .await
        .expect("payé avec l'écart");
    assert_eq!(done.decomposition.gap, dec!(0.30));
    assert_eq!(done.decomposition.rounding_difference, Decimal::ZERO);
    assert_eq!(done.difference_account_id, Some(m.ecart));
    let l = lignes(&pool, done.journal_entry_id).await;
    assert_eq!(l.len(), 4);
    assert_eq!((l[3].1, l[3].2, l[3].3), (m.ecart, dec!(0.30), dec!(0)));
    assert!(equilibree(&l));
    let accepted: serde_json::Value =
        sqlx::query_scalar("SELECT details_json FROM audit_log WHERE id = ?")
            .bind(done.reconciliation_audit_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(accepted["difference_account_id"], m.ecart);
    assert_eq!(accepted["gap"], "0.30");
    assert!(accepted.get("bank_fees").is_none());

    // Moins payé : 149.70 pour 150.00.
    let c = facture(&pool, &m, "E-3", dec!(100.00), jour(6, 1), None).await;
    let d = facture(&pool, &m, "E-4", dec!(50.00), jour(6, 1), None).await;
    let bt2 = transaction(&pool, &m, dec!(-149.70), jour(6, 15)).await;
    let done = payer(&pool, &m, bt2, &[c, d], Some(m.ecart))
        .await
        .expect("payé avec l'écart");
    assert_eq!(done.decomposition.gap, dec!(-0.30));
    let l = lignes(&pool, done.journal_entry_id).await;
    assert_eq!((l[3].1, l[3].2, l[3].3), (m.ecart, dec!(0), dec!(0.30)));
    assert!(equilibree(&l));

    // Au-delà du plafond (5.00) : refusé, avec ou sans compte — le plafond
    // d'abord.
    let e = facture(&pool, &m, "E-5", dec!(100.00), jour(6, 1), None).await;
    let f = facture(&pool, &m, "E-6", dec!(50.00), jour(6, 1), None).await;
    let bt3 = transaction(&pool, &m, dec!(-160.00), jour(6, 15)).await;
    let avant = empreinte(&pool).await;
    let trop = GroupRefusal::DifferenceTooLarge {
        transaction_amount: dec!(-160.00),
        sum_applied: dec!(150.00),
        gap: dec!(10.00),
        max_difference: dec!(5.0000),
    };
    assert_eq!(
        payer(&pool, &m, bt3, &[e, f], None).await,
        Err(trop.clone())
    );
    assert_eq!(
        payer(&pool, &m, bt3, &[e, f], Some(m.ecart)).await,
        Err(trop)
    );
    assert_eq!(empreinte(&pool).await, avant);
}

/// AC2 — les comptes interdits pour l'écart, dans l'ordre du code : la banque,
/// un compte archivé, un compte non imputable, la dette d'une facture du
/// groupe ; et la dette égale à la banque. Rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn forbidden_accounts_are_refused(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "X-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "X-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.30), jour(6, 15)).await;
    let archive = compte(&pool, m.s.company_id, "6960", "Expense").await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    let titre = compte(&pool, m.s.company_id, "6900", "Expense").await;
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(titre)
        .execute(&pool)
        .await
        .unwrap();

    let avant = empreinte(&pool).await;
    assert_eq!(
        payer(&pool, &m, bt, &[a, b], Some(m.bank_ledger)).await,
        Err(GroupRefusal::DifferenceAccountIsBankLedger {
            account_id: m.bank_ledger
        })
    );
    assert_eq!(
        payer(&pool, &m, bt, &[a, b], Some(archive)).await,
        Err(GroupRefusal::DifferenceAccountInvalid {
            account_id: archive
        })
    );
    assert_eq!(
        payer(&pool, &m, bt, &[a, b], Some(titre)).await,
        Err(GroupRefusal::DifferenceAccountNotPostable {
            account_id: titre,
            account_number: "6900".into(),
        })
    );
    assert_eq!(
        payer(&pool, &m, bt, &[a, b], Some(m.payable)).await,
        Err(GroupRefusal::ClaimAccount {
            claim: ClaimSide::Payable,
            role: SettlementAccountRole::Counterparty,
            account_id: m.payable,
            account_number: Some("2010".into()),
            bank_account_id: None,
            piece: None,
        })
    );
    // La dette d'une facture est le compte de banque : refusé, la facture
    // nommée.
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(m.payable)
        .bind(m.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    match payer(&pool, &m, bt, &[a, b], Some(m.ecart)).await {
        Err(GroupRefusal::ClaimAccount {
            claim: ClaimSide::Payable,
            role: SettlementAccountRole::Counterparty,
            account_id,
            bank_account_id: Some(ba),
            piece: Some(piece),
            ..
        }) => {
            assert_eq!(account_id, m.payable);
            assert_eq!(ba, m.bank_account_id);
            assert_eq!(piece.document, GroupedDocument::SupplierInvoice(a));
            assert_eq!(piece.number, numero(&pool, a).await);
            assert!(piece.number.is_some());
        }
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(empreinte(&pool).await, avant);
}

/// AC1 — les refus des pièces et de la transaction, chacun sans rien écrire :
/// borne basse et doublon, sens, facture introuvable, déjà payée (non `open`),
/// dans un lot `generated`, datée après le paiement au-delà de la tolérance. (La
/// devise, la borne haute et les autres refus :
/// `the_other_refusals_write_nothing`.)
#[sqlx::test(migrations = "./test-schema")]
async fn each_invoice_and_transaction_refusal_writes_nothing(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "R-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "R-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    let credit = transaction(&pool, &m, dec!(150.00), jour(6, 15)).await;
    // Une facture datée plus de sept jours après le paiement.
    let tard = facture(&pool, &m, "R-3", dec!(50.00), jour(6, 30), None).await;
    let avant = empreinte(&pool).await;

    assert_eq!(
        payer(&pool, &m, bt, &[a], None).await,
        Err(GroupRefusal::TooFew { count: 1 })
    );
    assert_eq!(
        payer(&pool, &m, bt, &[a, a], None).await,
        Err(GroupRefusal::Duplicate {
            document: GroupedDocument::SupplierInvoice(a)
        })
    );
    assert_eq!(
        payer(&pool, &m, credit, &[a, b], None).await,
        Err(GroupRefusal::WrongDirection {
            amount: dec!(150.00)
        })
    );
    assert_eq!(
        payer(&pool, &m, bt, &[a, b + 1000], None).await,
        Err(GroupRefusal::InvoiceNotFound {
            document: GroupedDocument::SupplierInvoice(b + 1000)
        })
    );
    match payer(&pool, &m, bt, &[a, tard], None).await {
        Err(GroupRefusal::PaymentBeforeInvoiceDate {
            piece,
            tolerance_days: 7,
        }) => assert_eq!(piece.document, GroupedDocument::SupplierInvoice(tard)),
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(empreinte(&pool).await, avant);

    // Dans un lot `generated` : refusée, nommée.
    let lot = sqlx::query(
        "INSERT INTO payment_batches (company_id, bank_account_id, status, \
         requested_execution_date, total_amount, msg_id, payment_info_id) \
         VALUES (?, ?, 'generated', '2026-06-30', 50, 'MSG-43-1B', 'PMT-43-1B')",
    )
    .bind(m.s.company_id)
    .bind(m.bank_account_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO payment_batch_items (payment_batch_id, supplier_invoice_id, position, \
         end_to_end_id, amount) VALUES (?, ?, 1, 'E2E-43-1B', 50)",
    )
    .bind(lot)
    .bind(b)
    .execute(&pool)
    .await
    .unwrap();
    let avant = empreinte(&pool).await;
    match payer(&pool, &m, bt, &[a, b], None).await {
        Err(GroupRefusal::InvoiceNotEligible {
            piece,
            reason: IneligibleReason::SupplierInPaymentBatch,
        }) => {
            assert_eq!(piece.document, GroupedDocument::SupplierInvoice(b));
            assert_eq!(piece.number, numero(&pool, b).await);
        }
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(empreinte(&pool).await, avant);

    // Déjà payée depuis la fiche : non `open`.
    let c = facture(&pool, &m, "R-4", dec!(50.00), jour(6, 1), None).await;
    supplier_invoices::pay(
        &pool,
        m.s.company_id,
        c,
        SettlementChoice::BankTransfer {
            bank_account_id: m.bank_account_id,
        },
        jour(6, 2),
        m.s.admin_user_id,
        aujourd_hui(),
    )
    .await
    .expect("payée depuis la fiche");
    let avant = empreinte(&pool).await;
    match payer(&pool, &m, bt, &[a, c], None).await {
        Err(GroupRefusal::InvoiceNotEligible {
            piece,
            reason: IneligibleReason::SupplierNotOpen,
        }) => assert_eq!(piece.document, GroupedDocument::SupplierInvoice(c)),
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(empreinte(&pool).await, avant);
}

/// C-43-20 — chaque dette porte le projet de sa facture, **sans validation** :
/// un projet archivé après coup ne refuse pas.
#[sqlx::test(migrations = "./test-schema")]
async fn each_payable_line_carries_its_invoice_project(pool: MySqlPool) {
    let m = monter(&pool).await;
    let mut projets = Vec::new();
    for code in ["PF-A", "PF-B"] {
        projets.push(
            sqlx::query("INSERT INTO projects (company_id, code, name) VALUES (?, ?, ?)")
                .bind(m.s.company_id)
                .bind(code)
                .bind(format!("Projet {code}"))
                .execute(&pool)
                .await
                .expect("projet")
                .last_insert_id() as i64,
        );
    }
    let a = facture(&pool, &m, "P-1", dec!(100.00), jour(6, 1), Some(projets[0])).await;
    let b = facture(&pool, &m, "P-2", dec!(200.00), jour(6, 1), Some(projets[1])).await;
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(projets[0])
        .execute(&pool)
        .await
        .expect("projet archivé");
    let bt = transaction(&pool, &m, dec!(-300.00), jour(6, 15)).await;
    let done = payer(&pool, &m, bt, &[a, b], None)
        .await
        .expect("le projet archivé ne refuse pas");
    let l = lignes(&pool, done.journal_entry_id).await;
    assert_eq!(l[0].4, None, "ni projet d'en-tête sur la banque");
    assert_eq!(l[1].4, Some(projets[0]));
    assert_eq!(l[2].4, Some(projets[1]));
}

/// AC2 — `r` et `e` non nuls ensemble (cas 4 et 6 du tableau de la décision),
/// sur des totaux à quatre décimales : l'écriture s'équilibre, l'arrondi et
/// l'écart sont du côté qui l'équilibre, et la vue relit le compte d'écart.
#[sqlx::test(migrations = "./test-schema")]
async fn rounding_and_gap_together_balance_the_entry(pool: MySqlPool) {
    let m = monter(&pool).await;
    let arrondi = designate_rounding_account(&pool, m.s.company_id)
        .await
        .expect("compte d'arrondi");
    // Cas 4 : 3 × 10.0050 payés 30.06 → e = +0.05 (débit), r = −0.0050 (crédit).
    let mut ids = Vec::new();
    for n in ["Q-1", "Q-2", "Q-3"] {
        let id = facture(&pool, &m, n, dec!(10.00), jour(6, 1), None).await;
        fixer_total(&pool, id, dec!(10.0050)).await;
        ids.push(id);
    }
    let bt = transaction(&pool, &m, dec!(-30.06), jour(6, 15)).await;
    let done = payer(&pool, &m, bt, &ids, Some(m.ecart))
        .await
        .expect("payé");
    assert_eq!(done.decomposition.gap, dec!(0.05));
    assert_eq!(done.decomposition.rounding_difference, dec!(-0.0050));
    let l = lignes(&pool, done.journal_entry_id).await;
    assert_eq!(l.len(), 6);
    assert_eq!((l[4].1, l[4].2, l[4].3), (arrondi, dec!(0), dec!(0.0050)));
    assert_eq!((l[5].1, l[5].2, l[5].3), (m.ecart, dec!(0.05), dec!(0)));
    assert!(equilibree(&l), "{l:?}");
    let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, aujourd_hui())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.grouped_difference_account_id, Some(m.ecart));

    // Cas 6 : 10.0050 + 10.0000 payés 20.00 → e = −0.01 (crédit), r = +0.0050
    // (débit).
    let a = facture(&pool, &m, "Q-4", dec!(10.00), jour(6, 1), None).await;
    fixer_total(&pool, a, dec!(10.0050)).await;
    let b = facture(&pool, &m, "Q-5", dec!(10.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-20.00), jour(6, 15)).await;
    let done = payer(&pool, &m, bt, &[a, b], Some(m.ecart))
        .await
        .expect("payé");
    assert_eq!(done.decomposition.gap, dec!(-0.01));
    assert_eq!(done.decomposition.rounding_difference, dec!(0.0050));
    let l = lignes(&pool, done.journal_entry_id).await;
    assert_eq!((l[3].1, l[3].2, l[3].3), (arrondi, dec!(0.0050), dec!(0)));
    assert_eq!((l[4].1, l[4].2, l[4].3), (m.ecart, dec!(0), dec!(0.01)));
    assert!(equilibree(&l), "{l:?}");
    let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, aujourd_hui())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.grouped_difference_account_id, Some(m.ecart));
}

// ---------------------------------------------------------------------------
// AC3 — la concurrence : le paiement depuis la fiche, la création d'un lot
// ---------------------------------------------------------------------------

/// AC3, C-43-19 — un paiement depuis la fiche et la création d'un lot,
/// lancés pendant que le groupe tient ses factures, **attendent** son commit,
/// puis trouvent la facture payée : ni double paiement, ni lot.
#[sqlx::test(migrations = "./test-schema")]
async fn fiche_payment_and_batch_creation_wait_for_the_group(pool: MySqlPool) {
    use kesh_db::entities::NewPaymentBatch;
    use kesh_db::repositories::payment_batches;
    use kesh_db::test_fixtures::{attendre_des_requetes_en_cours, attendre_une_requete_en_cours};

    let m = monter(&pool).await;
    let a = facture(&pool, &m, "K-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "K-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;

    let mut tx = pool.begin().await.unwrap();
    grouped_settlements::pay_supplier_invoices_in_tx(
        &mut tx,
        m.s.company_id,
        m.bank_account_id,
        bt,
        &[a, b],
        None,
        1,
        acteur(&m),
        aujourd_hui(),
    )
    .await
    .expect("aucune erreur de base")
    .expect("payé, non validé");

    // Le paiement de `a` depuis la fiche.
    let fiche = tokio::spawn({
        let pool = pool.clone();
        let (company_id, bank_account_id, user_id) =
            (m.s.company_id, m.bank_account_id, m.s.admin_user_id);
        async move {
            supplier_invoices::pay(
                &pool,
                company_id,
                a,
                SettlementChoice::BankTransfer { bank_account_id },
                jour(6, 16),
                user_id,
                aujourd_hui(),
            )
            .await
        }
    });
    // La création d'un lot pour `b`.
    let lot = tokio::spawn({
        let pool = pool.clone();
        let (company_id, bank_account_id, user_id) =
            (m.s.company_id, m.bank_account_id, m.s.admin_user_id);
        async move {
            payment_batches::create_batch(
                &pool,
                NewPaymentBatch {
                    company_id,
                    bank_account_id,
                    requested_execution_date: jour(6, 20),
                    supplier_invoice_ids: vec![b],
                },
                user_id,
            )
            .await
        }
    });
    // Les deux attendent un verrou de facture (lecture verrouillante de
    // `supplier_invoices`) ; celle du lot, par sa clé primaire forcée.
    assert!(
        attendre_des_requetes_en_cours(&pool, &["FROM supplier_invoices", "FOR UPDATE"], 2, || {
            fiche.is_finished() || lot.is_finished()
        })
        .await,
        "le paiement depuis la fiche ou la création du lot n'a pas attendu"
    );
    assert!(
        attendre_une_requete_en_cours(&pool, &["FORCE INDEX (PRIMARY)", "FOR UPDATE"], || {
            lot.is_finished()
        })
        .await,
        "la création du lot n'a pas attendu"
    );
    tx.commit().await.unwrap();

    let fiche = fiche.await.unwrap();
    assert!(
        matches!(fiche, Err(DbError::IllegalStateTransition(_))),
        "paiement depuis la fiche : {fiche:?}"
    );
    let lot = lot.await.unwrap().expect("aucune erreur de base");
    assert!(
        lot.batch.is_none(),
        "un lot a été créé pour une facture payée"
    );
    assert_eq!(lot.failed.len(), 1);
    // Payées une fois, par le groupe.
    let paiements: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'supplier_invoice.paid'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(paiements, 2);
}

// ---------------------------------------------------------------------------
// AC6, AC6 bis — la vue et le dé-rapprochement
// ---------------------------------------------------------------------------

/// AC6 bis — la vue d'un groupe fournisseur : la liste des membres (famille,
/// numéro, montant) par identifiant croissant ; le compte d'écart relu pour
/// `e > 0` et pour `e < 0` ; `None` sans écart ; aucun motif d'annulation.
#[sqlx::test(migrations = "./test-schema")]
async fn the_view_of_a_supplier_group(pool: MySqlPool) {
    let m = monter(&pool).await;
    for (montant, attendu) in [
        (dec!(-150.30), Some(m.ecart)),
        (dec!(-149.70), Some(m.ecart)),
        (dec!(-150.00), None),
    ] {
        let a = facture(
            &pool,
            &m,
            &format!("V{montant}-1"),
            dec!(100.00),
            jour(6, 1),
            None,
        )
        .await;
        let b = facture(
            &pool,
            &m,
            &format!("V{montant}-2"),
            dec!(50.00),
            jour(6, 1),
            None,
        )
        .await;
        let bt = transaction(&pool, &m, montant, jour(6, 15)).await;
        payer(&pool, &m, bt, &[b, a], Some(m.ecart))
            .await
            .expect("payé");
        let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, aujourd_hui())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(view.kind.map(|k| k.code()), Some("grouped_settlement"));
        assert_eq!(view.grouped_invoices.len(), 2);
        assert_eq!(view.grouped_invoices[0].id, a);
        assert_eq!(view.grouped_invoices[0].kind, GroupFamily::Supplier);
        assert_eq!(view.grouped_invoices[0].number, numero(&pool, a).await);
        assert_eq!(view.grouped_invoices[1].amount, dec!(50.00));
        assert_eq!(view.grouped_difference_account_id, attendu, "{montant}");
        assert_eq!(view.cancel_blocker, None);
        assert_eq!(view.cancel_blocked_invoice, None);
        assert!(!view.unlink_only);
    }
}

/// AC6 — le dé-rapprochement d'un groupe fournisseur, par une clé d'API :
/// jamais un retrait seul (`Some(true)` refusé, rien d'écrit) ; puis tout
/// défait — transaction `pending`, factures `open`, colonnes de règlement
/// **ligne comprise** à `NULL`, écriture contre-passée, audit à la clé — et le
/// groupe se refait.
#[sqlx::test(migrations = "./test-schema")]
async fn unreconciling_a_supplier_group_undoes_it_entirely(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "U-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "U-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.30), jour(6, 15)).await;
    let done = payer(&pool, &m, bt, &[a, b], Some(m.ecart))
        .await
        .expect("payé");
    let versions = (etat(&pool, a).await.6, etat(&pool, b).await.6);
    let cle = cle_api(&pool, &m).await;

    // Jamais un retrait seul : refusé sans `Invariant`, rien d'écrit.
    let avant = empreinte(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    let changed = reconciliation_cancel::cancel_in_tx(
        &mut tx,
        m.s.company_id,
        bt,
        cle,
        Some(true),
        aujourd_hui(),
    )
    .await;
    assert!(
        matches!(
            changed,
            Err(DbError::ReconciliationCancelChanged { unlink_only: false })
        ),
        "{changed:?}"
    );
    tx.rollback().await.unwrap();
    assert_eq!(empreinte(&pool).await, avant);

    let audit_avant = dernier_audit(&pool).await;
    let undone = reconciliation_cancel::cancel(&pool, m.s.company_id, bt, cle, aujourd_hui())
        .await
        .expect("dé-rapproché, sans Invariant");
    assert_eq!(
        undone
            .grouped_invoices
            .iter()
            .map(|i| i.id)
            .collect::<Vec<_>>(),
        vec![a, b]
    );
    assert!(
        undone
            .grouped_invoices
            .iter()
            .all(|i| i.kind == GroupFamily::Supplier)
    );
    assert_eq!(undone.invoice_id, None);
    assert_eq!(undone.supplier_invoice_id, None);
    let reversal = undone.reversal_journal_entry_id.expect("contre-passée");
    assert_eq!(
        undone.bank_transaction.status,
        kesh_db::entities::bank_transaction::BankTransactionStatus::Pending
    );
    assert_eq!(undone.bank_transaction.matched_entry_id, None);
    for (id, version) in [(a, versions.0), (b, versions.1)] {
        let e = etat(&pool, id).await;
        assert_eq!(
            (e.0.as_str(), e.1, e.2, e.3, e.4, e.5),
            ("open", None, None, None, None, None),
            "facture {id} rouverte, colonnes vidées"
        );
        assert_eq!(e.6, version + 1);
    }
    let renverse: Option<i64> =
        sqlx::query_scalar("SELECT reverses_entry_id FROM journal_entries WHERE id = ?")
            .bind(reversal)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(renverse, Some(done.journal_entry_id));
    // L'audit : une annulation par facture, puis le geste, tous à la clé.
    let (total, avec_cle) = audit_par_cle(&pool, audit_avant, cle).await;
    assert!(
        total >= 4,
        "deux annulations, la contre-passation, le geste"
    );
    assert_eq!(total, avec_cle);
    let annulees: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.settlement_cancelled' \
         AND id > ? ORDER BY id",
    )
    .bind(audit_avant)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(annulees.len(), 2);
    assert_eq!(
        annulees[0]["settlementJournalEntryLineId"],
        done.invoices[0].line_id
    );
    assert_eq!(annulees[0]["reversalJournalEntryId"], reversal);
    let cancelled: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.cancelled' \
         AND entity_id = ?",
    )
    .bind(bt)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cancelled["kind"], "grouped_settlement");
    assert_eq!(
        cancelled["supplier_invoices"].as_array().map(Vec::len),
        Some(2)
    );

    // Le groupe se refait.
    let again = payer(&pool, &m, bt, &[a, b], Some(m.ecart)).await;
    assert!(again.is_ok(), "{again:?}");
}

/// AC6 — un motif de l'écriture (le compte d'écart archivé) refuse le
/// dé-rapprochement avant toute écriture, et la vue nomme la **première**
/// facture du groupe.
#[sqlx::test(migrations = "./test-schema")]
async fn an_entry_blocker_refuses_and_names_the_first_invoice(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "B-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "B-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.30), jour(6, 15)).await;
    payer(&pool, &m, bt, &[a, b], Some(m.ecart))
        .await
        .expect("payé");
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(m.ecart)
        .execute(&pool)
        .await
        .unwrap();
    let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, aujourd_hui())
        .await
        .unwrap()
        .unwrap();
    assert!(view.cancel_blocker.is_some(), "le compte archivé bloque");
    let piece = view.cancel_blocked_invoice.expect("facture nommée");
    assert_eq!(piece.document, GroupedDocument::SupplierInvoice(a));
    assert_eq!(piece.number, numero(&pool, a).await);

    let avant = empreinte(&pool).await;
    let refus =
        reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur(&m), aujourd_hui()).await;
    assert!(
        matches!(refus, Err(DbError::ReconciliationNotCancellable { .. })),
        "{refus:?}"
    );
    assert_eq!(empreinte(&pool).await, avant);
}

// ---------------------------------------------------------------------------
// AC7 — le détachement ; AC8 — les gestes unitaires refusés
// ---------------------------------------------------------------------------

/// AC7 — annuler une facture payée du groupe la **détache** : sa ligne revient
/// à `NULL` avec son écriture (pas de 500), l'audit nomme sa ligne. La vue ne
/// liste plus qu'elle-même, et relit encore le bon compte d'écart (la ligne
/// détachée reste dans l'écriture). Le dé-rapprochement contre-passe
/// l'écriture entière et ne rouvre que la facture encore attachée ; quand
/// plus aucune ne l'est, l'écriture est classée `entry`.
#[sqlx::test(migrations = "./test-schema")]
async fn a_detached_invoice_leaves_the_group_and_the_rest_unreconciles(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "D-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "D-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.30), jour(6, 15)).await;
    let done = payer(&pool, &m, bt, &[a, b], Some(m.ecart))
        .await
        .expect("payé");

    // Détacher la DERNIÈRE facture (sa ligne précède les lignes d'écart).
    supplier_invoices::cancel(&pool, m.s.company_id, b, m.s.admin_user_id, aujourd_hui())
        .await
        .expect("facture payée du groupe annulée");
    let e = etat(&pool, b).await;
    assert_eq!((e.0.as_str(), e.1, e.2), ("cancelled", None, None));
    let trace: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.cancelled' \
         AND entity_id = ?",
    )
    .bind(b)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        trace["settlementJournalEntryLineId"],
        done.invoices[1].line_id
    );
    assert_eq!(trace["settlementJournalEntryId"], done.journal_entry_id);

    let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, aujourd_hui())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.kind.map(|k| k.code()), Some("grouped_settlement"));
    assert_eq!(
        view.grouped_invoices
            .iter()
            .map(|i| i.id)
            .collect::<Vec<_>>(),
        vec![a]
    );
    assert_eq!(view.grouped_difference_account_id, Some(m.ecart));

    let undone =
        reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur(&m), aujourd_hui())
            .await
            .expect("dé-rapproché");
    assert_eq!(
        undone
            .grouped_invoices
            .iter()
            .map(|i| i.id)
            .collect::<Vec<_>>(),
        vec![a]
    );
    assert!(undone.reversal_journal_entry_id.is_some());
    assert_eq!(etat(&pool, a).await.0, "open");
    assert_eq!(
        etat(&pool, b).await.0,
        "cancelled",
        "la détachée n'est pas rouverte"
    );
    let reversal_lines: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines jel JOIN journal_entries je \
         ON je.id = jel.entry_id WHERE je.reverses_entry_id = ?",
    )
    .bind(done.journal_entry_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(reversal_lines, 4, "l'écriture entière contre-passée");

    // Plus aucune facture attachée : l'écriture est classée `entry`.
    let c = facture(&pool, &m, "D-3", dec!(100.00), jour(6, 1), None).await;
    let d = facture(&pool, &m, "D-4", dec!(50.00), jour(6, 1), None).await;
    let bt2 = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    payer(&pool, &m, bt2, &[c, d], None).await.expect("payé");
    for id in [c, d] {
        supplier_invoices::cancel(&pool, m.s.company_id, id, m.s.admin_user_id, aujourd_hui())
            .await
            .expect("détachée");
    }
    let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt2, aujourd_hui())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.kind.map(|k| k.code()), Some("entry"));
    assert!(view.grouped_invoices.is_empty());
}

/// AC8 — sur une facture du groupe, les gestes unitaires sont refusés : le
/// retour en brouillon (`PaidReconciled`) et l'annulation du règlement (la
/// contre-passation refusée sur le motif `MATCHED_BANK_TRANSACTION`, code
/// d'erreur `ENTRY_NOT_REVERSABLE`). Rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn unit_gestures_on_a_grouped_invoice_are_refused(pool: MySqlPool) {
    use kesh_db::errors::SupplierUnvalidationBlocker;

    let m = monter(&pool).await;
    let a = facture(&pool, &m, "G-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "G-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    payer(&pool, &m, bt, &[a, b], None).await.expect("payé");
    let avant = empreinte(&pool).await;

    let version = etat(&pool, a).await.6;
    let brouillon =
        supplier_invoices::unvalidate(&pool, m.s.company_id, a, version, acteur(&m), aujourd_hui())
            .await;
    assert!(
        matches!(
            brouillon,
            Err(DbError::SupplierInvoiceNotUnvalidatable {
                blocker: SupplierUnvalidationBlocker::PaidReconciled,
                ..
            })
        ),
        "{brouillon:?}"
    );
    let annulation =
        supplier_invoices::cancel_settlement(&pool, m.s.company_id, a, acteur(&m), aujourd_hui())
            .await;
    assert!(
        matches!(
            annulation,
            Err(DbError::EntryNotReversable {
                blocker: kesh_db::errors::ReversalBlocker::MatchedBankTransaction,
                ..
            })
        ),
        "{:?}",
        annulation.err()
    );
    assert_eq!(empreinte(&pool).await, avant);
}

// ---------------------------------------------------------------------------
// Revue P1 de la 43-1b — vue après détachement, ordre des verrous, refus
// ---------------------------------------------------------------------------

/// Désigne `account` comme compte créanciers : les factures validées ensuite
/// portent leur dette sur ce compte (`generate_purchase_journal_lines`).
async fn dette_par_defaut(pool: &MySqlPool, m: &Montage, account: i64) {
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(account)
    .bind(m.s.company_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn vue(pool: &MySqlPool, m: &Montage, bt: i64) -> reconciliation_cancel::ReconciliationView {
    reconciliation_cancel::get_view(pool, m.s.company_id, bt, aujourd_hui())
        .await
        .expect("vue")
        .expect("transaction")
}

fn ids_de(view: &reconciliation_cancel::ReconciliationView) -> Vec<i64> {
    view.grouped_invoices.iter().map(|i| i.id).collect()
}

/// Revue P1 (A MEDIUM-1, B-2, E1) — la dernière facture du groupe, sur un
/// **autre** compte de dette, est détachée : sans écart d'origine, la vue ne
/// présente aucun compte d'écart (et non la dette de la facture détachée) ;
/// avec un écart, elle relit le compte choisi (et non un `Invariant`). Le
/// dé-rapprochement reste possible.
#[sqlx::test(migrations = "./test-schema")]
async fn a_detached_invoice_on_another_payable_account_leaves_no_false_gap(pool: MySqlPool) {
    let m = monter(&pool).await;
    let autre_dette = compte(&pool, m.s.company_id, "2011", "Liability").await;

    // Sans écart.
    let a = facture(&pool, &m, "W-1", dec!(100.00), jour(6, 1), None).await;
    dette_par_defaut(&pool, &m, autre_dette).await;
    let b = facture(&pool, &m, "W-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    payer(&pool, &m, bt, &[a, b], None).await.expect("payé");
    supplier_invoices::cancel(&pool, m.s.company_id, b, m.s.admin_user_id, aujourd_hui())
        .await
        .expect("détachée");
    let v = vue(&pool, &m, bt).await;
    assert_eq!(ids_de(&v), vec![a]);
    assert_eq!(
        v.grouped_difference_account_id, None,
        "la dette de la facture détachée n'est pas un compte d'écart"
    );

    // Avec un écart.
    dette_par_defaut(&pool, &m, m.payable).await;
    let c = facture(&pool, &m, "W-3", dec!(100.00), jour(6, 1), None).await;
    dette_par_defaut(&pool, &m, autre_dette).await;
    let d = facture(&pool, &m, "W-4", dec!(50.00), jour(6, 1), None).await;
    let bt2 = transaction(&pool, &m, dec!(-150.30), jour(6, 15)).await;
    payer(&pool, &m, bt2, &[c, d], Some(m.ecart))
        .await
        .expect("payé");
    supplier_invoices::cancel(&pool, m.s.company_id, d, m.s.admin_user_id, aujourd_hui())
        .await
        .expect("détachée");
    let v = vue(&pool, &m, bt2).await;
    assert_eq!(ids_de(&v), vec![c]);
    assert_eq!(v.grouped_difference_account_id, Some(m.ecart));
    reconciliation_cancel::cancel(&pool, m.s.company_id, bt2, acteur(&m), aujourd_hui())
        .await
        .expect("dé-rapproché");
}

/// Revue P1 (E5) — le détachement de la **première** facture, puis d'une
/// facture **du milieu** : la vue liste les autres et relit le compte d'écart ;
/// le dé-rapprochement contre-passe l'écriture entière.
#[sqlx::test(migrations = "./test-schema")]
async fn detaching_the_first_or_a_middle_invoice_keeps_the_view_right(pool: MySqlPool) {
    let m = monter(&pool).await;
    for detachee in [0usize, 1] {
        let mut ids = Vec::new();
        for (n, montant) in [dec!(100.00), dec!(50.00), dec!(25.00)]
            .into_iter()
            .enumerate()
        {
            ids.push(
                facture(
                    &pool,
                    &m,
                    &format!("M{detachee}-{n}"),
                    montant,
                    jour(6, 1),
                    None,
                )
                .await,
            );
        }
        let bt = transaction(&pool, &m, dec!(-175.30), jour(6, 15)).await;
        let done = payer(&pool, &m, bt, &ids, Some(m.ecart))
            .await
            .expect("payé");
        supplier_invoices::cancel(
            &pool,
            m.s.company_id,
            ids[detachee],
            m.s.admin_user_id,
            aujourd_hui(),
        )
        .await
        .expect("détachée");
        let restantes: Vec<i64> = ids
            .iter()
            .enumerate()
            .filter(|(k, _)| *k != detachee)
            .map(|(_, id)| *id)
            .collect();
        let v = vue(&pool, &m, bt).await;
        assert_eq!(ids_de(&v), restantes, "détachée : {detachee}");
        assert_eq!(v.grouped_difference_account_id, Some(m.ecart));
        let undone =
            reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur(&m), aujourd_hui())
                .await
                .expect("dé-rapproché");
        assert_eq!(
            undone
                .grouped_invoices
                .iter()
                .map(|i| i.id)
                .collect::<Vec<_>>(),
            restantes
        );
        let lignes_inverses: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM journal_entry_lines jel JOIN journal_entries je \
             ON je.id = jel.entry_id WHERE je.reverses_entry_id = ?",
        )
        .bind(done.journal_entry_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(lignes_inverses, 5, "banque, trois dettes, écart");
    }
}

/// Revue P1 (A LOW-2, LOW-3 ; E4) — les autres refus, chacun sans rien écrire :
/// borne haute (51), devise, facture d'un lot `generated` du compte au montant
/// du débit (la pièce nommée est **l'autre** facture, C-e431b-3), brouillon,
/// annulée, compte d'arrondi absent, compte bancaire clos, exercice clos.
#[sqlx::test(migrations = "./test-schema")]
async fn the_other_refusals_write_nothing(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "O-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "O-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    let euros = sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details) \
         SELECT company_id, import_id, bank_account_id, booking_date, amount, 'EUR', details \
         FROM bank_transactions WHERE id = ?",
    )
    .bind(bt)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let brouillon = supplier_invoices::create_draft(
        &pool,
        NewSupplierInvoice {
            company_id: m.s.company_id,
            contact_id: sqlx::query_scalar("SELECT contact_id FROM supplier_invoices WHERE id = ?")
                .bind(a)
                .fetch_one(&pool)
                .await
                .unwrap(),
            supplier_invoice_number: Some("O-3".into()),
            invoice_date: jour(6, 1),
            due_date: None,
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Brouillon".into(),
                quantity: dec!(1),
                unit_price: dec!(50.00),
                vat_rate: dec!(0),
                expense_account_id: m.s.accounts["4000"],
            }],
        },
        m.s.admin_user_id,
    )
    .await
    .expect("brouillon")
    .invoice
    .id;
    let annulee = facture(&pool, &m, "O-4", dec!(50.00), jour(6, 1), None).await;
    supplier_invoices::cancel(
        &pool,
        m.s.company_id,
        annulee,
        m.s.admin_user_id,
        aujourd_hui(),
    )
    .await
    .expect("annulée");
    let fraction = facture(&pool, &m, "O-5", dec!(50.00), jour(6, 1), None).await;
    fixer_total(&pool, fraction, dec!(49.9960)).await;
    // Une AUTRE facture de 150.00, dans un lot `generated` du compte.
    let tenue = facture(&pool, &m, "O-6", dec!(150.00), jour(6, 1), None).await;
    let lot = sqlx::query(
        "INSERT INTO payment_batches (company_id, bank_account_id, status, \
         requested_execution_date, total_amount, msg_id, payment_info_id) \
         VALUES (?, ?, 'generated', '2026-06-30', 150, 'MSG-43-1B-O', 'PMT-43-1B-O')",
    )
    .bind(m.s.company_id)
    .bind(m.bank_account_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let avant = empreinte(&pool).await;

    let trop: Vec<i64> = (1..=51).collect();
    assert_eq!(
        payer(&pool, &m, bt, &trop, None).await,
        Err(GroupRefusal::TooMany { count: 51 })
    );
    assert_eq!(
        payer(&pool, &m, euros, &[a, b], None).await,
        Err(GroupRefusal::CurrencyMismatch {
            currency: "EUR".into()
        })
    );
    for (pas_ouverte, nom) in [(brouillon, "brouillon"), (annulee, "annulée")] {
        match payer(&pool, &m, bt, &[a, pas_ouverte], None).await {
            Err(GroupRefusal::InvoiceNotEligible {
                piece,
                reason: IneligibleReason::SupplierNotOpen,
            }) => assert_eq!(
                piece.document,
                GroupedDocument::SupplierInvoice(pas_ouverte)
            ),
            other => panic!("{nom} : refus attendu, {other:?}"),
        }
    }
    // Sans compte d'arrondi : 100.00 + 49.9960 payés 150.00 → r = +0.0040.
    assert_eq!(
        payer(&pool, &m, bt, &[a, fraction], None).await,
        Err(GroupRefusal::RoundingAccountNotConfigured)
    );
    assert_eq!(empreinte(&pool).await, avant);

    // La garde du lot du compte : l'autre facture, au montant du débit.
    sqlx::query(
        "INSERT INTO payment_batch_items (payment_batch_id, supplier_invoice_id, position, \
         end_to_end_id, amount) VALUES (?, ?, 1, 'E2E-43-1B-O', 150)",
    )
    .bind(lot)
    .bind(tenue)
    .execute(&pool)
    .await
    .unwrap();
    let avant = empreinte(&pool).await;
    match payer(&pool, &m, bt, &[a, b], None).await {
        Err(GroupRefusal::HeldByPaymentBatch {
            piece,
            payment_batch_id,
        }) => {
            assert_eq!(piece.document, GroupedDocument::SupplierInvoice(tenue));
            assert_eq!(piece.number, numero(&pool, tenue).await);
            assert_eq!(payment_batch_id, Some(lot));
        }
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(empreinte(&pool).await, avant);
    sqlx::query("UPDATE payment_batches SET status = 'cancelled' WHERE id = ?")
        .bind(lot)
        .execute(&pool)
        .await
        .unwrap();

    // Le compte bancaire clos.
    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-07-01' WHERE id = ?")
        .bind(m.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        payer(&pool, &m, bt, &[a, b], None).await,
        Err(GroupRefusal::BankAccountClosed)
    );
    sqlx::query("UPDATE bank_accounts SET closed_on = NULL WHERE id = ?")
        .bind(m.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();

    // L'exercice clos.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(m.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        payer(&pool, &m, bt, &[a, b], None).await,
        Err(GroupRefusal::FiscalYearInvalid {
            value_date: jour(6, 15)
        })
    );
    assert_eq!(empreinte(&pool).await, avant);
}

/// Prend le verrou de la facture `id` dans une transaction qui le garde.
async fn tenir_facture(pool: &MySqlPool, id: i64) -> sqlx::Transaction<'static, sqlx::MySql> {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM supplier_invoices WHERE id = ? FOR UPDATE")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx
}

/// Lance le paiement groupé dans une tâche, sur sa propre transaction validée.
fn payer_en_tache(
    pool: &MySqlPool,
    m: &Montage,
    bt: i64,
    ids: Vec<i64>,
) -> tokio::task::JoinHandle<Result<Result<GroupedSettlement, GroupRefusal>, DbError>> {
    let pool = pool.clone();
    let (company_id, bank_account_id, par) = (m.s.company_id, m.bank_account_id, acteur(m));
    tokio::spawn(async move {
        let mut tx = pool.begin().await.map_err(DbError::Sqlx)?;
        let out = grouped_settlements::pay_supplier_invoices_in_tx(
            &mut tx,
            company_id,
            bank_account_id,
            bt,
            &ids,
            None,
            1,
            par,
            aujourd_hui(),
        )
        .await?;
        tx.commit().await.map_err(DbError::Sqlx)?;
        Ok(out)
    })
}

/// Revue P1 (B-1, E2) — un paiement depuis la fiche de la **deuxième**
/// facture, lancé pendant que le groupe attend cette facture : l'ordre facture,
/// compte, lot tient (le groupe verrouille toutes ses factures avant le
/// compte), la fiche paie sans interblocage, et le groupe refuse ensuite la
/// facture devenue payée.
#[sqlx::test(migrations = "./test-schema")]
async fn a_fiche_payment_between_two_invoices_does_not_deadlock(pool: MySqlPool) {
    use kesh_db::test_fixtures::attendre_une_requete_en_cours;

    let m = monter(&pool).await;
    let a = facture(&pool, &m, "L-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "L-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;

    let mut fiche = tenir_facture(&pool, b).await;
    let groupe = payer_en_tache(&pool, &m, bt, vec![a, b]);
    assert!(
        attendre_une_requete_en_cours(&pool, &["supplier_invoices", "FOR UPDATE"], || {
            groupe.is_finished()
        })
        .await,
        "le groupe n'attend pas la deuxième facture"
    );
    // La fiche paie `b` dans la transaction qui la tient : facture, puis compte.
    let paye = supplier_invoices::pay_in_tx(
        &mut fiche,
        m.s.company_id,
        b,
        SettlementChoice::BankTransfer {
            bank_account_id: m.bank_account_id,
        },
        jour(6, 16),
        m.s.admin_user_id,
    )
    .await;
    assert!(paye.is_ok(), "paiement depuis la fiche : {:?}", paye.err());
    fiche.commit().await.unwrap();

    match groupe.await.unwrap() {
        Ok(Err(GroupRefusal::InvoiceNotEligible {
            piece,
            reason: IneligibleReason::SupplierNotOpen,
        })) => assert_eq!(piece.document, GroupedDocument::SupplierInvoice(b)),
        other => panic!("refus attendu après la fiche : {other:?}"),
    }
}

/// Revue P1 (A LOW-1, LOW-5) — la transaction change (rejetée) pendant que le
/// groupe attend une facture : son `UPDATE … AND version = ?` la refuse
/// **après** l'écriture et les factures, et le point de sauvegarde défait
/// tout.
#[sqlx::test(migrations = "./test-schema")]
async fn a_transaction_changed_meanwhile_is_refused_after_the_entry(pool: MySqlPool) {
    use kesh_db::test_fixtures::attendre_une_requete_en_cours;

    let m = monter(&pool).await;
    let a = facture(&pool, &m, "T-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "T-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;

    let mut autre = tenir_facture(&pool, b).await;
    let groupe = payer_en_tache(&pool, &m, bt, vec![a, b]);
    assert!(
        attendre_une_requete_en_cours(&pool, &["supplier_invoices", "FOR UPDATE"], || {
            groupe.is_finished()
        })
        .await,
        "le groupe n'attend pas la deuxième facture"
    );
    sqlx::query(
        "UPDATE bank_transactions SET auto_match_rejected_at = NOW(3), version = version + 1 \
         WHERE id = ?",
    )
    .bind(bt)
    .execute(&mut *autre)
    .await
    .unwrap();
    autre.commit().await.unwrap();
    let avant = empreinte(&pool).await;

    assert_eq!(
        groupe.await.unwrap().expect("aucune erreur de base"),
        Err(GroupRefusal::TransactionNotPending)
    );
    assert_eq!(empreinte(&pool).await, avant, "rien d'écrit");
}

/// Revue P1 (A LOW-4) — le dé-rapprochement défait le lettrage de chaque
/// facture : la ligne de dette de son achat n'est plus lettrée avec le
/// paiement.
#[sqlx::test(migrations = "./test-schema")]
async fn unreconciling_unletters_each_purchase(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "Y-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "Y-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    let done = payer(&pool, &m, bt, &[a, b], None).await.expect("payé");
    let cle_achat = |id: i64| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, Option<i64>>(
                "SELECT jel.lettering_key FROM journal_entry_lines jel \
                 JOIN supplier_invoices si ON si.purchase_journal_entry_id = jel.entry_id \
                 WHERE si.id = ? AND jel.credit > 0",
            )
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap()
        }
    };
    for (id, r) in [a, b].into_iter().zip(&done.invoices) {
        let cle = cle_achat(id).await.expect("achat lettré");
        let ligne: Option<i64> =
            sqlx::query_scalar("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
                .bind(r.line_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(ligne, Some(cle));
    }
    reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur(&m), aujourd_hui())
        .await
        .expect("dé-rapproché");
    for id in [a, b] {
        assert_eq!(
            cle_achat(id).await,
            None,
            "l'achat de {id} n'est plus lettré"
        );
    }
}

// ---------------------------------------------------------------------------
// Revue P2 de la 43-1b
// ---------------------------------------------------------------------------

/// Revue P2 (LOW-1) — la trace du détachement de la **dernière** facture d'un
/// groupe **sans écart** a disparu : la vue ne peut plus nommer sa ligne de
/// dette. Elle rend `Invariant` (le nombre de lignes de dette nommées ne
/// recoupe plus celui du paiement), jamais la dette détachée comme compte
/// d'écart.
#[sqlx::test(migrations = "./test-schema")]
async fn a_missing_detachment_trace_makes_the_view_an_invariant(pool: MySqlPool) {
    let m = monter(&pool).await;
    let autre_dette = compte(&pool, m.s.company_id, "2011", "Liability").await;
    let a = facture(&pool, &m, "Z-1", dec!(100.00), jour(6, 1), None).await;
    dette_par_defaut(&pool, &m, autre_dette).await;
    let b = facture(&pool, &m, "Z-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    payer(&pool, &m, bt, &[a, b], None).await.expect("payé");
    supplier_invoices::cancel(&pool, m.s.company_id, b, m.s.admin_user_id, aujourd_hui())
        .await
        .expect("détachée");
    let n = sqlx::query(
        "DELETE FROM audit_log WHERE action = 'supplier_invoice.cancelled' AND entity_id = ?",
    )
    .bind(b)
    .execute(&pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 1, "la trace du détachement existait");
    let res = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, aujourd_hui()).await;
    assert!(matches!(res, Err(DbError::Invariant(_))), "{res:?}");
}

/// Revue P2 (LOW-3, E3) — le compte du grand livre du compte bancaire change
/// pendant que le groupe attend une facture : la ligne de banque porte le
/// compte **lu sous verrou** (le nouveau), non celui de l'instantané.
#[sqlx::test(migrations = "./test-schema")]
async fn the_bank_ledger_is_the_one_read_under_lock(pool: MySqlPool) {
    use kesh_db::test_fixtures::attendre_une_requete_en_cours;

    let m = monter(&pool).await;
    let nouveau = compte(&pool, m.s.company_id, "1021", "Asset").await;
    let a = facture(&pool, &m, "J-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "J-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;

    let mut autre = tenir_facture(&pool, b).await;
    let groupe = payer_en_tache(&pool, &m, bt, vec![a, b]);
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["FORCE INDEX (PRIMARY)", "supplier_invoices", "FOR UPDATE"],
            || groupe.is_finished()
        )
        .await,
        "le groupe n'attend pas la deuxième facture"
    );
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(nouveau)
        .bind(m.bank_account_id)
        .execute(&mut *autre)
        .await
        .unwrap();
    autre.commit().await.unwrap();

    let done = groupe
        .await
        .unwrap()
        .expect("aucune erreur de base")
        .expect("payé");
    let l = lignes(&pool, done.journal_entry_id).await;
    assert_eq!(l[0].1, nouveau, "la banque au compte lu sous verrou");
}

/// Revue P2 (LOW-7) — la branche défensive « aucun règlement » du
/// dé-rapprochement groupé : une écriture que ne règle aucune facture, ni
/// client ni fournisseur, rend `Invariant` et n'écrit rien.
#[sqlx::test(migrations = "./test-schema")]
async fn a_group_cancel_on_an_entry_without_settlement_is_an_invariant(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "N-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "N-2", dec!(50.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(-150.00), jour(6, 15)).await;
    payer(&pool, &m, bt, &[a, b], None).await.expect("payé");
    // L'écriture d'ACHAT de `a` : aucune facture ne la désigne comme règlement.
    let achat: i64 =
        sqlx::query_scalar("SELECT purchase_journal_entry_id FROM supplier_invoices WHERE id = ?")
            .bind(a)
            .fetch_one(&pool)
            .await
            .unwrap();
    let avant = empreinte(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    let transaction_bancaire =
        kesh_db::repositories::reconciliation::find_by_id_for_account_for_update(
            &mut *tx,
            m.s.company_id,
            m.bank_account_id,
            bt,
        )
        .await
        .unwrap()
        .expect("transaction");
    let res = grouped_settlements::cancel_in_tx(
        &mut tx,
        m.s.company_id,
        &transaction_bancaire,
        achat,
        acteur(&m),
        None,
        aujourd_hui(),
    )
    .await;
    assert!(matches!(res, Err(DbError::Invariant(_))), "{res:?}");
    tx.rollback().await.unwrap();
    assert_eq!(empreinte(&pool).await, avant);
}
