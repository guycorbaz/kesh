//! Story 43-1a (#823) — un règlement pour plusieurs factures : migration,
//! plafond de la différence, lettrage par ligne, règlement groupé client et son
//! dé-rapprochement.
//!
//! Tous les tests montent le squash (`test-schema`), que `test_schema_guard.rs`
//! prouve égal au schéma de `MIGRATOR`.

/// Story 35-5a0 — la date du jour de l'horloge de test.
#[allow(dead_code)]
#[path = "support/jour.rs"]
mod jour;

/// Story 48-14 (#1255, AC9) — le relevé de l'état d'un dé-rapprochement.
#[allow(dead_code)]
#[path = "support/caracterisation.rs"]
mod caracterisation;

use serde_json::{Value, json};

use kesh_db::repositories::company_invoice_settings::{
    DEFAULT_GROUP_DIFFERENCE_MAX, group_settlement_max_difference,
};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// La CHECK « ligne ⇒ écriture » (AC1.3, M2)
// ---------------------------------------------------------------------------

/// AC1.3 — poser la ligne de règlement d'une facture fournisseur sans son
/// écriture est refusé par la base (`chk_supplier_invoices_settlement_line`) ;
/// le fragment partagé `CLEAR_SETTLEMENT_SQL`, lui, vide les deux ensemble.
///
/// Sonde sous `FOREIGN_KEY_CHECKS = 0`, sur une seule connexion : la CHECK reste
/// active, seules les clés étrangères (vers des lignes factices) sont suspendues.
#[sqlx::test(migrations = "./test-schema")]
async fn a_settlement_line_without_its_entry_is_refused(pool: MySqlPool) {
    use kesh_db::repositories::supplier_invoices::CLEAR_SETTLEMENT_SQL;
    let mut conn = pool.acquire().await.expect("connexion");
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    let refus = sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, invoice_date, status, purchase_journal_entry_id, \
          settlement_journal_entry_line_id) \
         VALUES (1, 1, '2026-11-01', 'open', 1, 7)",
    )
    .execute(&mut *conn)
    .await;
    let err = refus.expect_err("ligne sans écriture acceptée");
    assert!(
        err.to_string()
            .contains("chk_supplier_invoices_settlement_line"),
        "refus inattendu : {err}"
    );
    let id = sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, invoice_date, status, purchase_journal_entry_id, \
          settlement_type, settlement_bank_account_id, settlement_journal_entry_id, \
          settlement_journal_entry_line_id, paid_at) \
         VALUES (1, 1, '2026-11-01', 'paid', 1, 'bank_transfer', 1, 3, 7, NOW())",
    )
    .execute(&mut *conn)
    .await
    .expect("ligne avec son écriture")
    .last_insert_id();
    // Vider l'écriture SEULE (la forme d'avant la 43-1a) : refusé.
    let seule = sqlx::query(
        "UPDATE supplier_invoices SET status = 'open', settlement_type = NULL, \
         settlement_bank_account_id = NULL, settlement_journal_entry_id = NULL, paid_at = NULL \
         WHERE id = ?",
    )
    .bind(id)
    .execute(&mut *conn)
    .await;
    assert!(seule.is_err(), "écriture vidée sans sa ligne acceptée");
    // Le fragment partagé : accepté, les deux à NULL.
    sqlx::query(&format!(
        "UPDATE supplier_invoices SET status = 'open', {CLEAR_SETTLEMENT_SQL} WHERE id = ?"
    ))
    .bind(id)
    .execute(&mut *conn)
    .await
    .expect("fragment partagé");
    let (entry, line): (Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT settlement_journal_entry_id, settlement_journal_entry_line_id \
         FROM supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert_eq!((entry, line), (None, None));
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Le plafond de la différence (AC3 bis, C-43-24)
// ---------------------------------------------------------------------------

/// AC3 bis, L9 — la constante du code et le `DEFAULT` de la colonne ne
/// dérivent pas : une seule valeur par défaut, lue des deux côtés.
#[sqlx::test(migrations = "./test-schema")]
async fn group_max_default_matches_the_column_default(pool: MySqlPool) {
    let column_default: String = sqlx::query_scalar(
        "SELECT COLUMN_DEFAULT FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'company_invoice_settings' \
           AND COLUMN_NAME = 'group_settlement_max_difference'",
    )
    .fetch_one(&pool)
    .await
    .expect("défaut de la colonne");
    let column_default: Decimal = column_default.parse().expect("défaut décimal");
    assert_eq!(column_default, DEFAULT_GROUP_DIFFERENCE_MAX);
    assert_eq!(DEFAULT_GROUP_DIFFERENCE_MAX, dec!(5.0000));
}

/// AC3 bis — la valeur de la société quand elle a sa ligne de réglages ; la
/// constante sinon.
#[sqlx::test(migrations = "./test-schema")]
async fn group_max_reads_the_setting_or_the_default(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.expect("seed");
    // Une société sans ligne de réglages (identifiant inexistant) : la constante.
    assert_eq!(
        group_settlement_max_difference(&pool, s.company_id + 1000)
            .await
            .expect("lecture"),
        DEFAULT_GROUP_DIFFERENCE_MAX
    );
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id) VALUES (?) \
         ON DUPLICATE KEY UPDATE company_id = company_id",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .expect("ligne de réglages");
    assert_eq!(
        group_settlement_max_difference(&pool, s.company_id)
            .await
            .expect("lecture"),
        dec!(5.0000)
    );
    sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 0 \
         WHERE company_id = ?",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .expect("plafond à zéro");
    assert_eq!(
        group_settlement_max_difference(&pool, s.company_id)
            .await
            .expect("lecture"),
        Decimal::ZERO
    );
    // La CHECK de la colonne : au-delà de 100, refusé par la base.
    let refus = sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 100.0001 \
         WHERE company_id = ?",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await;
    assert!(refus.is_err(), "plafond au-delà de 100 accepté");
}

// ---------------------------------------------------------------------------
// Montage commun du règlement groupé
// ---------------------------------------------------------------------------

use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewInvoice, NewInvoiceLine, NewJournalEntry};
use kesh_db::errors::DbError;
use kesh_db::repositories::grouped_settlements::{
    self, GroupRefusal, GroupedDocument, IneligibleReason,
};
use kesh_db::repositories::{invoices, journal_entries, reconciliation_cancel};
use kesh_db::test_fixtures::{SeededCompany, chf_line, designate_rounding_account};

/// La date de valeur des transactions des tests : dans l'exercice 2020-2030 du
/// seed, après la date des factures.
fn jour(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, m, d).expect("date valide")
}

/// Le montage : la société du seed, un compte bancaire lié au compte 1020
/// (créé : le 1100 du seed est la créance), le compte d'écart 4000.
struct Montage {
    s: SeededCompany,
    bank_account_id: i64,
    bank_ledger: i64,
}

async fn monter(pool: &MySqlPool) -> Montage {
    let s = seed_accounting_company(pool).await.expect("seed");
    let bank_ledger = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '1020', 'Banque groupée', 'Asset')",
    )
    .bind(s.company_id)
    .execute(pool)
    .await
    .expect("compte de banque")
    .last_insert_id() as i64;
    let bank_account_id = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'Banque test', 'CH9300762011623852957', ?)",
    )
    .bind(s.company_id)
    .bind(bank_ledger)
    .execute(pool)
    .await
    .expect("compte bancaire")
    .last_insert_id() as i64;
    Montage {
        s,
        bank_account_id,
        bank_ledger,
    }
}

/// Une facture validée de `amount` (quatre décimales admises), d'un client
/// neuf, avec son écriture de vente `D 1100 / C 3000` posée à la main (patron
/// de `invoice_settlement.rs::validated_invoice`).
async fn facture(
    pool: &MySqlPool,
    m: &Montage,
    numero: &str,
    amount: Decimal,
    date: NaiveDate,
    project_id: Option<i64>,
) -> i64 {
    let s = &m.s;
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', ?, TRUE) RETURNING id",
    )
    .bind(s.company_id)
    .bind(format!("Client {numero}"))
    .fetch_one(pool)
    .await
    .expect("contact");
    let inv = invoices::create(
        pool,
        s.admin_user_id,
        NewInvoice {
            company_id: s.company_id,
            contact_id,
            date,
            due_date: Some(date),
            payment_terms: None,
            project_id,
            lines: vec![NewInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: amount,
                vat_rate: dec!(0),
                revenue_account_id: Some(s.accounts["3000"]),
                line_type: kesh_db::entities::LineType::Priced,
            }],
        },
    )
    .await
    .expect("facture créée")
    .0;
    // L'exercice qui couvre la date (le seed, ou un exercice posé par le test).
    let exercice: i64 = sqlx::query_scalar(
        "SELECT id FROM fiscal_years WHERE company_id = ? AND start_date <= ? AND end_date >= ? \
         ORDER BY start_date DESC LIMIT 1",
    )
    .bind(s.company_id)
    .bind(date)
    .bind(date)
    .fetch_one(pool)
    .await
    .expect("exercice couvrant la date de la facture");
    let je = journal_entries::create_with_policy(
        pool,
        exercice,
        Acteur::utilisateur(s.admin_user_id),
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: date,
            journal: Journal::Ventes,
            description: "Vente".into(),
            project_id: None,
            lines: vec![
                chf_line(s.accounts["1100"], amount, Decimal::ZERO),
                chf_line(s.accounts["3000"], Decimal::ZERO, amount),
            ],
        },
        // Story 53-1a : écritures identiques à dessein (décor), politique Confirmed.
        kesh_db::repositories::duplicate_policy::DuplicatePolicy::Confirmed,
    )
    .await
    .expect("écriture de vente");
    sqlx::query(
        "UPDATE invoices SET status = 'validated', journal_entry_id = ?, invoice_number = ? \
         WHERE id = ?",
    )
    .bind(je.entry.id)
    .bind(numero)
    .bind(inv.id)
    .execute(pool)
    .await
    .expect("validation de montage");
    inv.id
}

/// Une transaction `pending` en CHF de `amount` sur le compte du montage.
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
         VALUES (?, ?, ?, ?, ?, 'CHF', 'virement groupé')",
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

/// Règle le groupe dans une transaction, la valide, rend le résultat.
async fn regler(
    pool: &MySqlPool,
    m: &Montage,
    bt: i64,
    ids: &[i64],
    difference_account_id: Option<i64>,
) -> Result<grouped_settlements::GroupedSettlement, GroupRefusal> {
    let mut tx = pool.begin().await.expect("transaction");
    let out = grouped_settlements::settle_invoices_in_tx(
        &mut tx,
        m.s.company_id,
        m.bank_account_id,
        bt,
        ids,
        difference_account_id,
        1,
        acteur(m),
        jour(11, 17),
    )
    .await
    .expect("aucune erreur de base");
    // D6 : la transaction est VALIDÉE, refus compris — le point de sauvegarde
    // doit avoir tout défait.
    tx.commit().await.expect("commit");
    out
}

/// Une empreinte des tables que le geste écrit : nombre de lignes et somme de
/// contrôle des colonnes qui bougent.
async fn empreinte(pool: &MySqlPool) -> Vec<i64> {
    let mut out = Vec::new();
    for sql in [
        "SELECT COUNT(*) FROM journal_entries",
        "SELECT COUNT(*) FROM journal_entry_lines",
        "SELECT COUNT(*) FROM invoice_settlements",
        "SELECT COUNT(*) FROM audit_log",
        "SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key IS NOT NULL",
        "SELECT CAST(COALESCE(SUM(version), 0) AS SIGNED) FROM invoices",
        "SELECT COUNT(*) FROM invoices WHERE paid_at IS NOT NULL",
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

async fn reste(pool: &MySqlPool, invoice_id: i64) -> Decimal {
    kesh_db::repositories::invoice_settlements::amount_due(pool, invoice_id)
        .await
        .expect("reste dû")
}

// ---------------------------------------------------------------------------
// AC5, AC6 — le règlement groupé client
// ---------------------------------------------------------------------------

/// AC5, AC6 — deux factures de deux clients, réglées au montant exact : une
/// écriture (banque, une créance par facture), chaque facture soldée et lettrée
/// avec sa seule ligne de créance ; la banque n'est pas lettrée.
#[sqlx::test(migrations = "./test-schema")]
async fn two_clients_are_settled_and_lettered_one_by_one(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "F-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "F-2", dec!(200.00), jour(6, 2), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;

    let done = regler(&pool, &m, bt, &[b, a], None).await.expect("réglé");
    assert_eq!(done.invoices.len(), 2);
    assert_eq!(done.invoices[0].id, a, "par identifiant croissant");
    assert_eq!(done.decomposition.gap, Decimal::ZERO);
    assert_eq!(done.difference_account_id, None);

    for id in [a, b] {
        assert_eq!(reste(&pool, id).await, Decimal::ZERO);
        let paid: Option<chrono::NaiveDateTime> =
            sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(paid.is_some(), "paid_at posé");
    }
    // Les lignes : banque au débit, créances au crédit, rien d'autre.
    let lignes: Vec<(i64, i64, Decimal, Decimal, Option<i64>)> = sqlx::query_as(
        "SELECT id, account_id, debit, credit, lettering_key FROM journal_entry_lines \
         WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(done.journal_entry_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(lignes.len(), 3);
    assert_eq!(lignes[0].1, m.bank_ledger);
    assert_eq!(lignes[0].2, dec!(300.00));
    assert_eq!(lignes[0].4, None, "la banque n'est pas lettrée");
    // Une clé de lettrage par facture, chacune sur sa ligne de vente et sa
    // seule ligne de créance.
    let mut cles = std::collections::BTreeSet::new();
    for r in &done.invoices {
        let (line_key,): (Option<i64>,) =
            sqlx::query_as("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
                .bind(r.line_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        let key = line_key.expect("ligne de créance lettrée");
        let groupe: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = ?")
                .bind(key)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            groupe, 2,
            "la vente et la seule ligne de créance de la facture"
        );
        cles.insert(key);
    }
    assert_eq!(cles.len(), 2, "une clé par facture");
    // L'unicité nouvelle : deux lignes de règlement sur la même écriture.
    let n: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE journal_entry_id = ?")
            .bind(done.journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(n, 2);
    // La transaction pointe l'écriture ; l'audit du dépôt.
    let matched: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(bt)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(matched, Some(done.journal_entry_id));
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.accepted' \
         AND entity_id = ?",
    )
    .bind(bt)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["proposal_type"], "invoice_group");
    assert_eq!(details["batch_size"], 1);
    let paid_traces: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'invoice.paid' \
         AND JSON_EXTRACT(details_json, '$.reconciliation_audit_id') = ?",
    )
    .bind(done.reconciliation_audit_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(paid_traces, 2);
}

/// Le tableau de la décision, en écritures (quatre de ses cas) : deux restes de
/// 10.0050 payés 20.01 (`d = e = r = 0`, ni ligne d'arrondi ni ligne d'écart) ;
/// payés 20.02 (`e = +0.01` au crédit du compte choisi) ; 10.0049 + 10.0000
/// payés 20.00 (`r = −0.0049` au débit du compte d'arrondi, arrondi
/// automatique) ; 10.0050 + 10.0000 payés 20.00 (`e = −0.01` au débit du
/// compte choisi, `r = +0.0050` au crédit du compte d'arrondi). Les écritures
/// s'équilibrent, et la vue relit le compte d'écart sur sa ligne.
#[sqlx::test(migrations = "./test-schema")]
async fn sub_centime_remainders_go_to_rounding_and_the_gap_to_the_chosen_account(pool: MySqlPool) {
    let m = monter(&pool).await;
    let arrondi = designate_rounding_account(&pool, m.s.company_id)
        .await
        .unwrap();
    let ecart = m.s.accounts["4000"];
    for (rang, (restes, paye, attendu_arrondi, attendu_gap)) in [
        ([dec!(10.005), dec!(10.005)], dec!(20.01), dec!(0), dec!(0)),
        (
            [dec!(10.005), dec!(10.005)],
            dec!(20.02),
            dec!(0),
            dec!(0.01),
        ),
        (
            [dec!(10.0049), dec!(10.00)],
            dec!(20.00),
            dec!(-0.0049),
            dec!(0),
        ),
        (
            [dec!(10.005), dec!(10.00)],
            dec!(20.00),
            dec!(0.0050),
            dec!(-0.01),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let a = facture(
            &pool,
            &m,
            &format!("S-{rang}-1"),
            restes[0],
            jour(6, 1),
            None,
        )
        .await;
        let b = facture(
            &pool,
            &m,
            &format!("S-{rang}-2"),
            restes[1],
            jour(6, 1),
            None,
        )
        .await;
        assert_eq!(reste(&pool, a).await, restes[0], "montage : reste brut");
        let bt = transaction(&pool, &m, paye, jour(6, 15)).await;
        let done = regler(&pool, &m, bt, &[a, b], Some(ecart))
            .await
            .expect("réglé");
        assert_eq!(done.decomposition.rounding_difference, attendu_arrondi);
        assert_eq!(done.decomposition.gap, attendu_gap);
        let (d, c): (Decimal, Decimal) = sqlx::query_as(
            "SELECT SUM(debit), SUM(credit) FROM journal_entry_lines WHERE entry_id = ?",
        )
        .bind(done.journal_entry_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(d, c, "écriture équilibrée");
        let sur_arrondi: Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(credit - debit), 0) FROM journal_entry_lines \
             WHERE entry_id = ? AND account_id = ?",
        )
        .bind(done.journal_entry_id)
        .bind(arrondi)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(sur_arrondi, attendu_arrondi);
        let sur_ecart: Decimal = sqlx::query_scalar(
            "SELECT COALESCE(SUM(credit - debit), 0) FROM journal_entry_lines \
             WHERE entry_id = ? AND account_id = ?",
        )
        .bind(done.journal_entry_id)
        .bind(ecart)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            sur_ecart, attendu_gap,
            "moins reçu au débit du compte choisi"
        );
        for id in [a, b] {
            assert_eq!(reste(&pool, id).await, Decimal::ZERO);
        }
        // La vue relit le compte d'écart sur sa ligne (C-e431a-8).
        let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            view.grouped_difference_account_id,
            if attendu_gap.is_zero() {
                None
            } else {
                Some(ecart)
            }
        );
    }
}

/// Revue d'intégration de l'Epic 37, M2 — un compte d'écart **en devise** est
/// refusé par un refus nommé (`DifferenceAccountInForeignCurrency`), non par
/// l'erreur de base du contrôle central de l'écriture. Après le contrôle « non
/// imputable » : un compte en devise non imputable reste
/// `DifferenceAccountNotPostable`. Rien n'est écrit ; une fois la devise
/// retirée, le même compte est accepté.
#[sqlx::test(migrations = "./test-schema")]
async fn a_foreign_currency_difference_account_is_refused(pool: MySqlPool) {
    let m = monter(&pool).await;
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(m.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let en_euros: i64 = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '6949', 'Frais en euros', 'Expense', 'EUR')",
    )
    .bind(m.s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let a = facture(&pool, &m, "D-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "D-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(299.60), jour(6, 15)).await;

    let avant = empreinte(&pool).await;
    assert_eq!(
        regler(&pool, &m, bt, &[a, b], Some(en_euros)).await,
        Err(GroupRefusal::DifferenceAccountInForeignCurrency {
            account_id: en_euros,
            currency: "EUR".into(),
        })
    );
    assert_eq!(empreinte(&pool).await, avant, "rien d'écrit");

    // L'ordre des contrôles : non imputable d'abord.
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(en_euros)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        regler(&pool, &m, bt, &[a, b], Some(en_euros)).await,
        Err(GroupRefusal::DifferenceAccountNotPostable {
            account_id: en_euros,
            account_number: "6949".into(),
        })
    );

    // Sonde positive : le même compte, en CHF et imputable, est accepté.
    sqlx::query("UPDATE accounts SET postable = TRUE, currency = NULL WHERE id = ?")
        .bind(en_euros)
        .execute(&pool)
        .await
        .unwrap();
    let done = regler(&pool, &m, bt, &[a, b], Some(en_euros))
        .await
        .expect("compte en CHF accepté");
    assert_eq!(done.difference_account_id, Some(en_euros));
}

/// Décision de Guy du 2026-11-17 — moins reçu et trop-perçu admis jusqu'au
/// plafond, au compte choisi ; au-delà, refusé ; sans compte, refusé ; le compte
/// contrôlé. Après chaque refus, la transaction est VALIDÉE et les tables sont
/// inchangées (D6).
#[sqlx::test(migrations = "./test-schema")]
async fn the_gap_is_admitted_both_ways_up_to_the_ceiling(pool: MySqlPool) {
    let m = monter(&pool).await;
    let ecart = m.s.accounts["4000"];
    // Refus d'abord, tables comparées.
    let a = facture(&pool, &m, "G-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "G-2", dec!(200.00), jour(6, 1), None).await;
    let cas: Vec<(Decimal, Option<i64>, GroupRefusal)> = vec![
        (
            dec!(294.99),
            Some(ecart),
            GroupRefusal::DifferenceTooLarge {
                transaction_amount: dec!(294.99),
                sum_applied: dec!(300.00),
                gap: dec!(-5.01),
                max_difference: dec!(5),
            },
        ),
        (
            dec!(299.60),
            None,
            GroupRefusal::DifferenceAccountRequired { gap: dec!(-0.40) },
        ),
        (
            dec!(299.60),
            Some(m.bank_ledger),
            GroupRefusal::DifferenceAccountIsBankLedger {
                account_id: m.bank_ledger,
            },
        ),
        (
            dec!(299.60),
            Some(999_999),
            GroupRefusal::DifferenceAccountInvalid {
                account_id: 999_999,
            },
        ),
    ];
    for (paye, compte, attendu) in cas {
        let bt = transaction(&pool, &m, paye, jour(6, 15)).await;
        let avant = empreinte(&pool).await;
        let refus = regler(&pool, &m, bt, &[a, b], compte)
            .await
            .expect_err("refusé");
        assert_eq!(refus, attendu);
        assert_eq!(
            empreinte(&pool).await,
            avant,
            "rien d'écrit après {attendu:?}"
        );
    }
    // Le compte de tiers : refus `ClaimAccount`, rôle `Counterparty`.
    let bt = transaction(&pool, &m, dec!(299.60), jour(6, 15)).await;
    assert!(matches!(
        regler(&pool, &m, bt, &[a, b], Some(m.s.accounts["1100"])).await,
        Err(GroupRefusal::ClaimAccount {
            role: kesh_db::errors::SettlementAccountRole::Counterparty,
            ..
        })
    ));
    // Un compte non imputable.
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(m.s.accounts["2000"])
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        regler(&pool, &m, bt, &[a, b], Some(m.s.accounts["2000"])).await,
        Err(GroupRefusal::DifferenceAccountNotPostable { .. })
    ));
    // Moins reçu de 0.40 (299.60 pour 300.00) : accepté au plafond par défaut,
    // une fois le compte choisi imputable (le −5.00 exact est
    // `three_invoices_one_partly_settled_and_a_five_franc_gap`).
    let done = regler(&pool, &m, bt, &[a, b], Some(ecart))
        .await
        .expect("0.40");
    assert_eq!(done.decomposition.gap, dec!(-0.40));
    assert_eq!(done.difference_account_id, Some(ecart));
    // Trop-perçu de 0.05 (au-delà de l'arrondi automatique) : accepté, au
    // crédit du compte choisi.
    let c = facture(&pool, &m, "G-3", dec!(100.00), jour(6, 1), None).await;
    let d = facture(&pool, &m, "G-4", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.05), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[c, d], Some(ecart))
        .await
        .expect("trop-perçu");
    assert_eq!(done.decomposition.gap, dec!(0.05));
    let sur_ecart: Decimal = sqlx::query_scalar(
        "SELECT credit FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(done.journal_entry_id)
    .bind(ecart)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sur_ecart, dec!(0.05));
    // Plafond à zéro : un écart de deux centimes (au-delà de l'arrondi
    // automatique de deux factures) est refusé.
    sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 0 \
         WHERE company_id = ?",
    )
    .bind(m.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let e = facture(&pool, &m, "G-5", dec!(100.00), jour(6, 1), None).await;
    let f = facture(&pool, &m, "G-6", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(299.98), jour(6, 15)).await;
    assert!(matches!(
        regler(&pool, &m, bt, &[e, f], Some(ecart)).await,
        Err(GroupRefusal::DifferenceTooLarge { .. })
    ));
}

/// Les refus de la transaction et des factures — un par variante atteignable
/// ici ; rien d'écrit après chacun.
#[sqlx::test(migrations = "./test-schema")]
async fn each_refusal_writes_nothing(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "R-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "R-2", dec!(200.00), jour(6, 1), None).await;
    let tard = facture(&pool, &m, "R-3", dec!(50.00), jour(6, 20), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let debit = transaction(&pool, &m, dec!(-300.00), jour(6, 15)).await;
    let nul = transaction(&pool, &m, dec!(0.00), jour(6, 15)).await;
    let eur = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    sqlx::query("UPDATE bank_transactions SET currency = 'EUR' WHERE id = ?")
        .bind(eur)
        .execute(&pool)
        .await
        .unwrap();
    let cas: Vec<(i64, Vec<i64>, GroupRefusal)> = vec![
        (bt, vec![a], GroupRefusal::TooFew { count: 1 }),
        (
            bt,
            vec![a, b, a],
            GroupRefusal::Duplicate {
                document: GroupedDocument::Invoice(a),
            },
        ),
        (999_999, vec![a, b], GroupRefusal::TransactionNotFound),
        (
            debit,
            vec![a, b],
            GroupRefusal::WrongDirection {
                amount: dec!(-300.00),
            },
        ),
        (
            nul,
            vec![a, b],
            GroupRefusal::WrongDirection { amount: dec!(0.00) },
        ),
        (
            eur,
            vec![a, b],
            GroupRefusal::CurrencyMismatch {
                currency: "EUR".into(),
            },
        ),
        (
            bt,
            vec![a, 999_999],
            GroupRefusal::InvoiceNotFound {
                document: GroupedDocument::Invoice(999_999),
            },
        ),
    ];
    for (t, ids, attendu) in cas {
        let avant = empreinte(&pool).await;
        assert_eq!(regler(&pool, &m, t, &ids, None).await, Err(attendu.clone()));
        assert_eq!(
            empreinte(&pool).await,
            avant,
            "rien d'écrit après {attendu:?}"
        );
    }
    // M-B — paiement daté de plus d'un jour avant une facture.
    let avant = empreinte(&pool).await;
    assert!(matches!(
        regler(&pool, &m, bt, &[a, tard], None).await,
        Err(GroupRefusal::PaymentBeforeInvoiceDate { piece, tolerance_days: 1 })
            if piece.document == GroupedDocument::Invoice(tard)
    ));
    assert_eq!(empreinte(&pool).await, avant);
    // Une facture non validée.
    sqlx::query("UPDATE invoices SET status = 'draft' WHERE id = ?")
        .bind(b)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        regler(&pool, &m, bt, &[a, b], None).await,
        Err(GroupRefusal::InvoiceNotEligible {
            reason: IneligibleReason::NotValidated,
            ..
        })
    ));
    assert_eq!(empreinte(&pool).await, avant);
}

/// H-B — une facture dont le projet a été archivé après sa validation se règle
/// en groupe ; sa ligne de créance porte ce projet, la banque n'en porte aucun ;
/// deux factures à projets différents (AC6).
#[sqlx::test(migrations = "./test-schema")]
async fn each_receivable_line_carries_its_invoice_project(pool: MySqlPool) {
    let m = monter(&pool).await;
    let mut projets = Vec::new();
    for code in ["P-A", "P-B"] {
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
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], None)
        .await
        .expect("le projet archivé ne refuse pas");
    let lignes: Vec<(i64, Option<i64>)> = sqlx::query_as(
        "SELECT account_id, project_id FROM journal_entry_lines WHERE entry_id = ? \
         ORDER BY line_order",
    )
    .bind(done.journal_entry_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(lignes[0], (m.bank_ledger, None));
    assert_eq!(lignes[1].1, Some(projets[0]));
    assert_eq!(lignes[2].1, Some(projets[1]));
}

/// H-A, M1 — protocole en trois temps : l'instantané naît avant la course ; un
/// règlement partiel de la même facture est validé sur une autre connexion ; le
/// groupe refuse la facture (`race_during_update`), rien n'est écrit, et le
/// reste dû n'est jamais négatif.
#[sqlx::test(migrations = "./test-schema")]
async fn a_concurrent_partial_settlement_is_refused_by_the_version(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "C-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "C-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;

    // (1) L'instantané : une lecture cohérente dans la transaction du groupe.
    let mut tx = pool.begin().await.unwrap();
    let vu = kesh_db::repositories::invoice_settlements::amount_due(&mut *tx, a)
        .await
        .unwrap();
    assert_eq!(vu, dec!(100.00));
    // (2) Sur une autre connexion, un règlement partiel validé.
    let version: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(a)
        .fetch_one(&pool)
        .await
        .unwrap();
    kesh_db::repositories::invoice_settlements_write::settle_invoice(
        &pool,
        m.s.admin_user_id,
        m.s.company_id,
        a,
        kesh_db::entities::SettlementChoice::InternalAccount {
            account_id: m.s.accounts["1000"],
        },
        dec!(30.00),
        jour(6, 10),
        version,
    )
    .await
    .expect("règlement partiel concurrent");
    // (3) Le groupe : refusé sur la facture, rien d'écrit.
    let out = grouped_settlements::settle_invoices_in_tx(
        &mut tx,
        m.s.company_id,
        m.bank_account_id,
        bt,
        &[a, b],
        None,
        1,
        acteur(&m),
        jour(11, 17),
    )
    .await
    .expect("aucune erreur de base");
    assert!(
        matches!(
            &out,
            Err(GroupRefusal::InvoiceNotEligible {
                piece,
                reason: IneligibleReason::RaceDuringUpdate,
            }) if piece.document == GroupedDocument::Invoice(a)
        ),
        "{out:?}"
    );
    tx.commit().await.unwrap();
    assert_eq!(reste(&pool, a).await, dec!(70.00), "jamais négatif");
    let liees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_transactions WHERE id = ? AND matched_entry_id IS NOT NULL",
    )
    .bind(bt)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(liees, 0);
}

/// H-A — deux groupes qui partagent une facture, même protocole : le groupe 2
/// lit, le groupe 1 est validé, le groupe 2 refuse la facture partagée.
#[sqlx::test(migrations = "./test-schema")]
async fn two_groups_sharing_an_invoice_settle_it_once(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "D-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "D-2", dec!(200.00), jour(6, 1), None).await;
    let c = facture(&pool, &m, "D-3", dec!(50.00), jour(6, 1), None).await;
    let bt1 = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let bt2 = transaction(&pool, &m, dec!(150.00), jour(6, 15)).await;

    let mut tx2 = pool.begin().await.unwrap();
    kesh_db::repositories::invoice_settlements::amount_due(&mut *tx2, a)
        .await
        .unwrap();
    // Le groupe 2 n'a fait qu'une lecture cohérente (aucun verrou) : le groupe
    // 1 ne l'attend pas, l'issue est déterministe.
    regler(&pool, &m, bt1, &[a, b], None)
        .await
        .expect("groupe 1");
    let out = grouped_settlements::settle_invoices_in_tx(
        &mut tx2,
        m.s.company_id,
        m.bank_account_id,
        bt2,
        &[a, c],
        None,
        1,
        acteur(&m),
        jour(11, 17),
    )
    .await
    .expect("aucune erreur de base");
    assert!(matches!(
        out,
        Err(GroupRefusal::InvoiceNotEligible {
            reason: IneligibleReason::RaceDuringUpdate,
            ..
        })
    ));
    tx2.commit().await.unwrap();
    let reglements: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?")
            .bind(a)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reglements, 1);
}

// ---------------------------------------------------------------------------
// AC7, AC8, AC2, AC9 — le dé-rapprochement et les gestes unitaires
// ---------------------------------------------------------------------------

/// AC7 — le dé-rapprochement défait le groupe en entier : transaction
/// `pending`, règlements retirés, écriture contre-passée, factures à régler,
/// lettrage défait ; l'audit porte la clé d'API à chaque entrée.
#[sqlx::test(migrations = "./test-schema")]
async fn unreconciling_a_group_undoes_it_entirely(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "U-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "U-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(299.60), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], Some(m.s.accounts["4000"]))
        .await
        .expect("réglé");

    // La vue : la variante et la liste.
    let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.kind.map(|k| k.code()), Some("grouped_settlement"));
    assert_eq!(view.grouped_invoices.len(), 2);
    assert_eq!(view.cancel_blocker, None);

    let key = sqlx::query(
        "INSERT INTO api_keys (company_id, created_by_user_id, name, key_hash, scope) \
         VALUES (?, ?, 'cle', SHA2('cle', 256), 'read-write')",
    )
    .bind(m.s.company_id)
    .bind(m.s.admin_user_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let audit_avant: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    let acteur_cle = Acteur {
        user_id: m.s.admin_user_id,
        api_key_id: Some(key),
    };
    // Un groupe n'est jamais un retrait seul.
    let mut tx = pool.begin().await.unwrap();
    let changed =
        reconciliation_cancel::cancel_in_tx(&mut tx, m.s.company_id, bt, acteur_cle, Some(true))
            .await;
    assert!(matches!(
        changed,
        Err(DbError::ReconciliationCancelChanged { unlink_only: false })
    ));
    drop(tx);

    let undone = reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur_cle)
        .await
        .expect("dé-rapproché");
    assert_eq!(undone.grouped_invoices.len(), 2);
    assert!(undone.reversal_journal_entry_id.is_some());
    assert_eq!(
        undone.bank_transaction.status,
        kesh_db::entities::bank_transaction::BankTransactionStatus::Pending
    );
    for id in [a, b] {
        assert!(reste(&pool, id).await > Decimal::ZERO);
        let paid: Option<chrono::NaiveDateTime> =
            sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(paid, None);
    }
    let restants: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE journal_entry_id = ?")
            .bind(done.journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(restants, 0);
    // Chaque entrée écrite par le geste porte la clé.
    let (total, avec_cle): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), CAST(SUM(actor_api_key_id <=> ?) AS SIGNED) FROM audit_log WHERE id > ?",
    )
    .bind(key)
    .bind(audit_avant)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        total >= 4,
        "au moins deux annulations, la contre-passation, le geste"
    );
    assert_eq!(total, avec_cle);
    let cancelled: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.cancelled' \
         AND entity_id = ?",
    )
    .bind(bt)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cancelled["kind"], "grouped_settlement");
    assert_eq!(cancelled["invoices"].as_array().map(Vec::len), Some(2));
    // Et le groupe se refait : la transaction est revenue à `pending`.
    let again = regler(&pool, &m, bt, &[a, b], Some(m.s.accounts["4000"])).await;
    assert!(again.is_ok(), "{again:?}");
}

/// AC8, AC9, AC2 — les gestes unitaires sur une facture du groupe sont
/// refusés, et l'écriture groupée refuse la modification et la
/// contre-passation directe ; ses clés de ligne restent intactes.
#[sqlx::test(migrations = "./test-schema")]
async fn a_grouped_entry_refuses_modification_and_keeps_its_line_keys(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "M-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "M-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], None).await.expect("réglé");

    // AC8 — annuler le règlement d'une facture du groupe : refusé.
    let sid = done.invoices[0].settlement_id.expect("règlement client");
    let annul = kesh_db::repositories::invoice_settlements_write::cancel_settlement(
        &pool,
        acteur(&m),
        m.s.company_id,
        a,
        sid,
    )
    .await;
    assert!(annul.is_err(), "annulation unitaire acceptée : {annul:?}");
    // AC8 — un nouveau règlement : la facture est payée, son reste est nul.
    let version: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(a)
        .fetch_one(&pool)
        .await
        .unwrap();
    let nouveau = kesh_db::repositories::invoice_settlements_write::settle_invoice(
        &pool,
        m.s.admin_user_id,
        m.s.company_id,
        a,
        kesh_db::entities::SettlementChoice::InternalAccount {
            account_id: m.s.accounts["1000"],
        },
        dec!(1.00),
        jour(6, 20),
        version,
    )
    .await;
    assert!(nouveau.is_err(), "nouveau règlement accepté");
    // AC9 — la contre-passation directe : refusée.
    let rev = journal_entries::reverse(
        &pool,
        m.s.company_id,
        done.journal_entry_id,
        acteur(&m),
        jour(11, 17),
    )
    .await;
    assert!(rev.is_err(), "contre-passation directe acceptée");
    // AC2 — la modification : refusée, aucune ligne réécrite.
    let lignes_avant: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM journal_entry_lines WHERE entry_id = ? ORDER BY id")
            .bind(done.journal_entry_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    let entry_version: i32 = sqlx::query_scalar("SELECT version FROM journal_entries WHERE id = ?")
        .bind(done.journal_entry_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let put = journal_entries::update(
        &pool,
        m.s.company_id,
        done.journal_entry_id,
        entry_version,
        acteur(&m),
        NewJournalEntry {
            company_id: m.s.company_id,
            entry_date: jour(6, 15),
            journal: Journal::Banque,
            description: "modifiée".into(),
            project_id: None,
            lines: vec![
                chf_line(m.bank_ledger, dec!(300.00), Decimal::ZERO),
                chf_line(m.s.accounts["1100"], Decimal::ZERO, dec!(300.00)),
            ],
        },
    )
    .await;
    assert!(put.is_err(), "modification d'une écriture groupée acceptée");
    let lignes_apres: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM journal_entry_lines WHERE entry_id = ? ORDER BY id")
            .bind(done.journal_entry_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(lignes_avant, lignes_apres);
    let cles: Vec<Option<i64>> = sqlx::query_scalar(
        "SELECT journal_entry_line_id FROM invoice_settlements WHERE journal_entry_id = ? \
         ORDER BY invoice_id",
    )
    .bind(done.journal_entry_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        cles,
        done.invoices
            .iter()
            .map(|r| Some(r.line_id))
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// AC12 — la montée depuis le schéma d'avant (résolution par version, P6)
// ---------------------------------------------------------------------------

#[path = "common/mod.rs"]
mod common;

/// Version de la migration de la Story 43-1a.
const GROUPED_VERSION: i64 = 20261118000001;

/// AC12 — une base au schéma d'avant la 43-1a, **peuplée** (règlement client
/// unitaire, facture fournisseur payée, réglages de facturation), monte sans
/// écart : données intactes, colonnes neuves à `NULL`, plafond à 5.0000 ; puis
/// un groupe se crée. Montage résolu par version, assertion de montage sur la
/// colonne absente.
#[sqlx::test(migrations = false)]
async fn the_upgrade_keeps_unit_settlements_and_allows_a_group(pool: MySqlPool) {
    common::apply_migrations_up_to(
        &pool,
        common::migrations_before(GROUPED_VERSION, "grouped_settlements"),
    )
    .await
    .expect("migrations avant grouped_settlements");
    let colonne: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() \
         AND TABLE_NAME = 'invoice_settlements' AND COLUMN_NAME = 'journal_entry_line_id'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        colonne, 0,
        "assertion de montage : la migration n'a pas tourné"
    );

    // Le peuplement, au schéma d'avant : une facture réglée par une écriture
    // unitaire, une facture fournisseur payée. Story 49-1a : le dépôt de la
    // facture écrit `line_type`, colonne d'une migration postérieure — posée le
    // temps du semis, retirée avant la montée.
    common::colonnes_types_de_ligne(&pool, true).await;
    let m = monter(&pool).await;
    let reglee = facture(&pool, &m, "AV-1", dec!(80.00), jour(5, 1), None).await;
    let reglement = journal_entries::create(
        &pool,
        m.s.fiscal_year_id,
        acteur(&m),
        NewJournalEntry {
            company_id: m.s.company_id,
            entry_date: jour(5, 10),
            journal: Journal::Banque,
            description: "Encaissement d'avant".into(),
            project_id: None,
            lines: vec![
                chf_line(m.bank_ledger, dec!(80.00), Decimal::ZERO),
                chf_line(m.s.accounts["1100"], Decimal::ZERO, dec!(80.00)),
            ],
        },
    )
    .await
    .expect("écriture de règlement");
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
         settled_on, settlement_type, settlement_bank_account_id) \
         VALUES (?, ?, ?, 80.00, ?, 'bank_transfer', ?)",
    )
    .bind(m.s.company_id)
    .bind(reglee)
    .bind(reglement.entry.id)
    .bind(jour(5, 10))
    .bind(m.bank_account_id)
    .execute(&pool)
    .await
    .expect("règlement d'avant");
    let contact: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
         VALUES (?, 'Entreprise', 'Fournisseur d''avant', TRUE) RETURNING id",
    )
    .bind(m.s.company_id)
    .fetch_one(&pool)
    .await
    .expect("fournisseur");
    let fournisseur = sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, invoice_date, status, \
         total_amount, purchase_journal_entry_id, settlement_type, settlement_bank_account_id, \
         settlement_journal_entry_id, paid_at) \
         VALUES (?, ?, ?, 'paid', 80.00, ?, 'bank_transfer', ?, ?, NOW())",
    )
    .bind(m.s.company_id)
    .bind(contact)
    .bind(jour(5, 1))
    .bind(reglement.entry.id)
    .bind(m.bank_account_id)
    .bind(reglement.entry.id)
    .execute(&pool)
    .await
    .expect("facture fournisseur payée")
    .last_insert_id() as i64;
    let empreinte_avant = (
        empreinte(&pool).await,
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM company_invoice_settings")
            .fetch_one(&pool)
            .await
            .unwrap(),
    );

    // La montée.
    common::colonnes_types_de_ligne(&pool, false).await;
    kesh_db::MIGRATOR.run(&pool).await.expect("montée");

    assert_eq!(
        (
            empreinte(&pool).await,
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM company_invoice_settings")
                .fetch_one(&pool)
                .await
                .unwrap(),
        ),
        empreinte_avant,
        "données intactes"
    );
    let ligne: Option<i64> = sqlx::query_scalar(
        "SELECT journal_entry_line_id FROM invoice_settlements WHERE invoice_id = ?",
    )
    .bind(reglee)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(ligne, None);
    let ligne: Option<i64> = sqlx::query_scalar(
        "SELECT settlement_journal_entry_line_id FROM supplier_invoices WHERE id = ?",
    )
    .bind(fournisseur)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(ligne, None);
    let plafond: Decimal = sqlx::query_scalar(
        "SELECT group_settlement_max_difference FROM company_invoice_settings \
         WHERE company_id = ?",
    )
    .bind(m.s.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(plafond, dec!(5.0000));

    // Après la montée, un groupe se crée.
    let a = facture(&pool, &m, "AP-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "AP-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    regler(&pool, &m, bt, &[a, b], None)
        .await
        .expect("un groupe après la montée");
}

// ---------------------------------------------------------------------------
// AC11 — le rattrapage de lettrage retiré, face à un groupe
// ---------------------------------------------------------------------------

/// AC11 — test d'accord du rattrapage `20261010000001` (retiré du rejeu,
/// `RETIRED_BACKFILLS`) sur un règlement groupé non lettré.
///
/// ⚠️ **Divergence assumée, écrite ici** : le SQL du rattrapage ne connaît pas
/// `invoice_settlements.journal_entry_line_id` (une migration appliquée ne se
/// modifie plus, P8). Pour une facture d'un groupe, son `C(I)` prend **toutes**
/// les lignes du compte de créance de l'écriture groupée — celles des autres
/// factures comprises —, si bien que la somme n'est pas nulle : il
/// **s'abstient**, il ne lettre rien de faux. La synchronisation vivante, qui
/// filtre par ligne (AC4), lettre chaque facture. Le rattrapage ne se rejoue
/// plus à l'import (fenêtre fermée depuis `20261025000001`) : la divergence
/// n'a pas de chemin en production ; ce test la tient décrite.
#[sqlx::test(migrations = "./test-schema")]
async fn the_retired_backfill_abstains_on_a_group_that_sync_letters(pool: MySqlPool) {
    use kesh_db::post_restore::RETIRED_BACKFILLS;
    use kesh_db::repositories::{fiscal_years, letterings};

    let m = monter(&pool).await;
    let a = facture(&pool, &m, "B-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "B-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], None).await.expect("réglé");
    let lignes: Vec<i64> = done.invoices.iter().map(|r| r.line_id).collect();

    // La fixture : le groupe non lettré (marques `document` effacées).
    sqlx::query(
        "UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL \
         WHERE lettering_origin = 'document'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let lettrees = |pool: MySqlPool, lignes: Vec<i64>| async move {
        let mut n = 0;
        for l in lignes {
            let k: Option<i64> =
                sqlx::query_scalar("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
                    .bind(l)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            n += i64::from(k.is_some());
        }
        n
    };
    assert_eq!(lettrees(pool.clone(), lignes.clone()).await, 0, "montage");

    // Le SQL du rattrapage : il s'abstient.
    let m1 = RETIRED_BACKFILLS
        .iter()
        .find(|b| b.version == 20261010000001)
        .expect("rattrapage retiré");
    sqlx::raw_sql(m1.sql)
        .execute(&pool)
        .await
        .expect("rattrapage exécuté");
    assert_eq!(
        lettrees(pool.clone(), lignes.clone()).await,
        0,
        "le rattrapage ne lettre pas un groupe"
    );

    // La synchronisation vivante : elle lettre chaque facture.
    let mut tx = pool.begin().await.unwrap();
    let fy = fiscal_years::find_open_covering_date(&mut tx, m.s.company_id, jour(6, 15))
        .await
        .unwrap()
        .expect("exercice");
    for id in [a, b] {
        letterings::sync_invoice_in_tx(&mut tx, m.s.company_id, id, fy.id, acteur(&m))
            .await
            .expect("synchronisation");
    }
    tx.commit().await.unwrap();
    assert_eq!(lettrees(pool.clone(), lignes).await, 2);
}

// ---------------------------------------------------------------------------
// Story 43-7 (#823) — le plafond réglé par l'administrateur est lu au geste
// ---------------------------------------------------------------------------

/// AC5 — le plafond est posé par le chemin d'écriture des réglages
/// (`company_invoice_settings::update`, celui du `PUT`), puis le geste le lit :
/// à 1.00, un écart de −1.01 **et** de +1.01 est refusé (`DifferenceTooLarge`,
/// `max_difference = 1.00`), un écart de −1.00 et de +1.00 est accepté ; le
/// plafond se juge d'abord, donc le refus tient aussi sans compte d'écart.
#[sqlx::test(migrations = "./test-schema")]
async fn the_ceiling_set_through_the_settings_is_read_by_the_gesture(pool: MySqlPool) {
    use kesh_db::entities::CompanyInvoiceSettingsUpdate;
    use kesh_db::repositories::company_invoice_settings;

    let m = monter(&pool).await;
    let ecart = m.s.accounts["4000"];
    let s = company_invoice_settings::get_or_create_default(&pool, m.s.company_id)
        .await
        .unwrap();
    let saved = company_invoice_settings::update(
        &pool,
        m.s.company_id,
        s.version,
        m.s.admin_user_id,
        CompanyInvoiceSettingsUpdate {
            invoice_number_format: s.invoice_number_format.clone(),
            default_receivable_account_id: s.default_receivable_account_id,
            default_revenue_account_id: s.default_revenue_account_id,
            default_vat_payable_account_id: s.default_vat_payable_account_id,
            default_vat_recoverable_account_id: s.default_vat_recoverable_account_id,
            default_vat_decompte_account_id: s.default_vat_decompte_account_id,
            default_sales_journal: s.default_sales_journal,
            journal_entry_description_template: s.journal_entry_description_template.clone(),
            credit_note_number_format: s.credit_note_number_format.clone(),
            supplier_internal_number_format: s.supplier_internal_number_format.clone(),
            default_payable_account_id: s.default_payable_account_id,
            default_rounding_account_id: s.default_rounding_account_id,
            round_to_5_centimes: s.round_to_5_centimes,
            minimum_invoice_amount: s.minimum_invoice_amount,
            default_discount_account_id: s.default_discount_account_id,
            default_bank_fees_account_id: s.default_bank_fees_account_id,
            default_bad_debt_account_id: s.default_bad_debt_account_id,
            group_settlement_max_difference: dec!(1.00),
        },
    )
    .await
    .expect("le plafond seul change : écrit");
    assert_eq!(saved.version, s.version + 1);
    assert_eq!(saved.group_settlement_max_difference, dec!(1.00));

    let a = facture(&pool, &m, "P-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "P-2", dec!(200.00), jour(6, 1), None).await;
    // Refusé, avec compte comme sans compte (le plafond se juge d'abord).
    for (paye, gap) in [(dec!(298.99), dec!(-1.01)), (dec!(301.01), dec!(1.01))] {
        let bt = transaction(&pool, &m, paye, jour(6, 15)).await;
        for compte in [Some(ecart), None] {
            let avant = empreinte(&pool).await;
            let refus = regler(&pool, &m, bt, &[a, b], compte)
                .await
                .expect_err("au-delà du plafond");
            assert_eq!(
                refus,
                GroupRefusal::DifferenceTooLarge {
                    transaction_amount: paye,
                    sum_applied: dec!(300.00),
                    gap,
                    max_difference: dec!(1.00),
                },
                "{paye} avec compte {compte:?}"
            );
            assert_eq!(empreinte(&pool).await, avant, "rien d'écrit");
        }
    }
    // Accepté au plafond, dans les deux sens, sur le compte choisi.
    for (n, (paye, gap)) in [(dec!(299.00), dec!(-1.00)), (dec!(301.00), dec!(1.00))]
        .into_iter()
        .enumerate()
    {
        let c = facture(
            &pool,
            &m,
            &format!("P-{}", 3 + 2 * n),
            dec!(100.00),
            jour(6, 1),
            None,
        )
        .await;
        let d = facture(
            &pool,
            &m,
            &format!("P-{}", 4 + 2 * n),
            dec!(200.00),
            jour(6, 1),
            None,
        )
        .await;
        let bt = transaction(&pool, &m, paye, jour(6, 15)).await;
        let done = regler(&pool, &m, bt, &[c, d], Some(ecart))
            .await
            .unwrap_or_else(|e| panic!("{paye} au plafond : {e:?}"));
        assert_eq!(done.decomposition.gap, gap);
        assert_eq!(done.difference_account_id, Some(ecart));
    }
}

// ---------------------------------------------------------------------------
// Revue P1 (B-1, E-1) — une clôture concurrente attend le dé-rapprochement d'un
// groupe
// ---------------------------------------------------------------------------

/// Un exercice 2019 ouvert (le seed couvre 2020-2030), rendu par son id.
async fn exercice_2019(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2019', '2019-01-01', '2019-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("exercice 2019")
    .last_insert_id() as i64
}

fn en_2019(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2019, m, d).expect("date valide")
}

/// ⛔ Revue P1 (B-1, E-1) — **une clôture concurrente attend le dé-rapprochement
/// d'un groupe**, sur le patron de `reconciliation_cancel.rs::
/// a_concurrent_close_waits_for_the_unreconciliation`. Le groupe est réglé en
/// 2019 ; la contre-passation, datée du jour, tombe dans un autre exercice,
/// ouvert. Si le geste figeait son instantané avant le verrou de l'exercice
/// (une lecture ordinaire des membres), il lirait 2019 encore ouvert, défairait
/// le lien et le lettrage et contre-passerait **avec succès** un règlement
/// d'exercice clos. Avec l'ordre de l'unitaire, il attend la clôture, relit, et
/// refuse (`FiscalYearClosed`) ; rien n'est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn a_concurrent_close_waits_for_the_group_unreconciliation(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker;

    let m = monter(&pool).await;
    let fy_2019 = exercice_2019(&pool, m.s.company_id).await;
    let a = facture(&pool, &m, "E-1", dec!(100.00), en_2019(6, 1), None).await;
    let b = facture(&pool, &m, "E-2", dec!(200.00), en_2019(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), en_2019(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], None)
        .await
        .expect("réglé en 2019");
    let avant = empreinte(&pool).await;

    // (1) La clôture en cours, non validée.
    let mut closing = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_2019)
        .execute(&mut *closing)
        .await
        .unwrap();

    // (2) Le dé-rapprochement démarre pendant ce temps.
    let p = pool.clone();
    let (company_id, user_id) = (m.s.company_id, m.s.admin_user_id);
    let annulation = tokio::spawn(async move {
        reconciliation_cancel::cancel(&p, company_id, bt, Acteur::utilisateur(user_id)).await
    });
    // Synchronisation déterministe : la clôture n'est validée qu'une fois le
    // geste VU en attente du `FOR UPDATE` de l'exercice.
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["fiscal_years", "FOR UPDATE"],
        || annulation.is_finished(),
    )
    .await;
    if !vue {
        panic!(
            "le dé-rapprochement a fini sans attendre de verrou : {:?}",
            annulation.await
        );
    }

    // (3) La clôture est validée.
    closing.commit().await.unwrap();

    let result = annulation.await.expect("tâche");
    assert!(
        matches!(
            result,
            Err(DbError::ReconciliationNotCancellable {
                blocker: SettlementCancelBlocker::FiscalYearClosed
            })
        ),
        "le geste devait attendre la clôture puis refuser l'exercice clos — reçu {result:?}"
    );
    // Rien d'écrit : la clôture a seulement relevé la version de l'exercice.
    assert_eq!(empreinte(&pool).await, avant, "rien n'a été défait");
    let reversals: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(done.journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reversals, 0, "rien n'a été contre-passé");
}

// ---------------------------------------------------------------------------
// Revue P1 (A-1, E-2) — chaque refus du dé-rapprochement d'un groupe (AC7.1)
// ---------------------------------------------------------------------------

/// Un groupe réglé : deux factures (100 et 200) payées 300 le 15 juin.
async fn groupe_regle(pool: &MySqlPool, m: &Montage, prefixe: &str) -> (i64, i64, i64, i64) {
    let a = facture(
        pool,
        m,
        &format!("{prefixe}-1"),
        dec!(100.00),
        jour(6, 1),
        None,
    )
    .await;
    let b = facture(
        pool,
        m,
        &format!("{prefixe}-2"),
        dec!(200.00),
        jour(6, 1),
        None,
    )
    .await;
    let bt = transaction(pool, m, dec!(300.00), jour(6, 15)).await;
    let done = regler(pool, m, bt, &[a, b], None).await.expect("réglé");
    (a, b, bt, done.journal_entry_id)
}

/// Le dé-rapprochement refusé par `attendu`, rien d'écrit, et la vue qui nomme
/// le motif et une facture du groupe (`cancelBlockedInvoice`, 43-2). Sans date
/// du jour depuis la Story 53-6a (D6).
async fn refus_du_derapprochement(
    pool: &MySqlPool,
    m: &Montage,
    bt: i64,
    attendu: kesh_db::errors::SettlementCancelBlocker,
) -> Option<GroupedDocument> {
    let avant = empreinte(pool).await;
    let res = reconciliation_cancel::cancel(pool, m.s.company_id, bt, acteur(m)).await;
    match res {
        Err(DbError::ReconciliationNotCancellable { blocker }) => {
            assert_eq!(blocker, attendu, "motif du refus")
        }
        autre => panic!("refus {attendu:?} attendu, reçu {autre:?}"),
    }
    assert_eq!(
        empreinte(pool).await,
        avant,
        "rien d'écrit après {attendu:?}"
    );
    let view = reconciliation_cancel::get_view(pool, m.s.company_id, bt)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.cancel_blocker.map(|h| h.0), Some(attendu), "la vue");
    let bloquee = view
        .cancel_blocked_invoice
        .expect("la vue nomme la facture du motif");
    Some(bloquee.document)
}

#[sqlx::test(migrations = "./test-schema")]
async fn group_unreconciliation_refuses_a_closed_year(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker as B;
    let m = monter(&pool).await;
    let fy = exercice_2019(&pool, m.s.company_id).await;
    let a = facture(&pool, &m, "Y-1", dec!(100.00), en_2019(6, 1), None).await;
    let b = facture(&pool, &m, "Y-2", dec!(200.00), en_2019(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), en_2019(6, 15)).await;
    regler(&pool, &m, bt, &[a, b], None).await.expect("réglé");
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy)
        .execute(&pool)
        .await
        .unwrap();
    let nommee = refus_du_derapprochement(&pool, &m, bt, B::FiscalYearClosed).await;
    assert_eq!(
        nommee,
        Some(GroupedDocument::Invoice(a)),
        "la première facture"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn group_unreconciliation_refuses_a_lettering_frozen_by_the_period(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker as B;
    let m = monter(&pool).await;
    let (_, _, bt, _) = groupe_regle(&pool, &m, "L").await;
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(jour(6, 30))
        .bind(m.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    // Story 53-6a (AC-9) : test de PRÉCÉDENCE sous `OriginDate` — l'écriture
    // du 15 juin est aussi sous la borne, et le rang 2 bis répond d'abord.
    refus_du_derapprochement(&pool, &m, bt, B::DocumentLetteringInClosedPeriods).await;
}

#[sqlx::test(migrations = "./test-schema")]
async fn group_unreconciliation_refuses_another_linked_transaction(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker as B;
    let m = monter(&pool).await;
    let (_, _, bt, entry) = groupe_regle(&pool, &m, "T").await;
    let autre = transaction(&pool, &m, dec!(300.00), jour(6, 16)).await;
    sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ? WHERE id = ?",
    )
    .bind(entry)
    .bind(autre)
    .execute(&pool)
    .await
    .unwrap();
    refus_du_derapprochement(&pool, &m, bt, B::MatchedBankTransaction).await;
}

#[sqlx::test(migrations = "./test-schema")]
async fn group_unreconciliation_refuses_an_archived_account(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker as B;
    let m = monter(&pool).await;
    let (_, _, bt, _) = groupe_regle(&pool, &m, "A").await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(m.s.accounts["1100"])
        .execute(&pool)
        .await
        .unwrap();
    refus_du_derapprochement(&pool, &m, bt, B::AccountArchived).await;
}

/// Story 53-6a (#1674, AC-9, D5) — **l'écriture du groupe sous la borne du
/// verrou** : refus `PeriodLocked` par le geste groupé lui-même
/// (`ReconciliationNotCancellable`, 409 à la route), la vue le dit avec la
/// borne ; au lendemain de la borne, l'annulation passe. Montage par les
/// chemins réels (D5) : les factures du 16 juin, payées le 15 (tolérance d'un
/// jour), le groupe de lettrage `document` n'est pas figé (la ligne de la
/// facture est en période ouverte). Remplace le test du rang 5
/// (`…_without_an_open_year_today`), sans objet sous `OriginDate`. Tue M6
/// (rang retiré), M11 (seuil strict) et M4 (date du jour).
#[sqlx::test(migrations = "./test-schema")]
async fn s536a_group_unreconciliation_refuses_an_entry_under_the_period_lock(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker as B;
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "P-1", dec!(100.00), jour(6, 16), None).await;
    let b = facture(&pool, &m, "P-2", dec!(200.00), jour(6, 16), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], None).await.expect("réglé");
    let verrou = |date: NaiveDate| {
        let pool = pool.clone();
        let c = m.s.company_id;
        async move {
            sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
                .bind(date)
                .bind(c)
                .execute(&pool)
                .await
                .unwrap();
        }
    };
    // Le jour de la borne : refusé (seuil inclusif).
    verrou(jour(6, 15)).await;
    refus_du_derapprochement(&pool, &m, bt, B::PeriodLocked).await;
    let vue = reconciliation_cancel::get_view(&pool, m.s.company_id, bt)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        vue.cancel_blocker,
        Some((B::PeriodLocked, None, Some("2026-06-15".to_string()))),
        "la borne en étiquette"
    );
    assert_eq!(vue.reversal_date, Some(jour(6, 15)));
    // Au lendemain de la borne : l'annulation passe, datée de l'écriture.
    verrou(jour(6, 14)).await;
    let undone = reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur(&m))
        .await
        .expect("dé-rapproché au lendemain de la borne");
    let date: NaiveDate = sqlx::query_scalar("SELECT entry_date FROM journal_entries WHERE id = ?")
        .bind(undone.reversal_journal_entry_id.expect("contre-passée"))
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_ne!(crate::jour::aujourd_hui_test(), jour(6, 15));
    assert_eq!(
        date,
        jour(6, 15),
        "datée de l'écriture {}",
        done.journal_entry_id
    );
}

/// Story 53-6a (#1674, AC-7, R4) — **un groupe dont l'écriture est dans un
/// exercice antérieur encore ouvert** (2019), le jour dans l'exercice du seed :
/// l'inverse est datée de l'écriture (15 juin 2019), dans 2019 ; rien n'est
/// écrit dans l'exercice du jour ; les deux factures redeviennent à régler.
/// Tue M4.
#[sqlx::test(migrations = "./test-schema")]
async fn s536a_group_unreconciliation_reverses_in_the_entry_year(pool: MySqlPool) {
    let m = monter(&pool).await;
    let fy = exercice_2019(&pool, m.s.company_id).await;
    let a = facture(&pool, &m, "O-1", dec!(100.00), en_2019(6, 1), None).await;
    let b = facture(&pool, &m, "O-2", dec!(200.00), en_2019(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), en_2019(6, 15)).await;
    regler(&pool, &m, bt, &[a, b], None).await.expect("réglé");
    let du_jour: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE fiscal_year_id = ?")
            .bind(m.s.fiscal_year_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    let undone = reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur(&m))
        .await
        .expect("dé-rapproché");
    let (date, exercice): (NaiveDate, i64) =
        sqlx::query_as("SELECT entry_date, fiscal_year_id FROM journal_entries WHERE id = ?")
            .bind(undone.reversal_journal_entry_id.expect("contre-passée"))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((date, exercice), (en_2019(6, 15), fy));
    let apres: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE fiscal_year_id = ?")
            .bind(m.s.fiscal_year_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(apres, du_jour, "rien dans l'exercice du jour");
    for id in [a, b] {
        let paid_at: Option<chrono::NaiveDateTime> =
            sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(paid_at, None, "la facture {id} redevient à régler");
    }
}

/// Story 53-6a (#1674, AC-3, AC-7, R4) — **l'exercice du groupe clôturé puis
/// rouvert** : l'inverse y est datée de l'écriture (15 juin 2019), comme à
/// l'exercice jamais clôturé ; rien dans l'exercice du jour ; puis 2019 se
/// clôture de nouveau sans obstacle nouveau. Tue M4.
#[sqlx::test(migrations = "./test-schema")]
async fn s536a_group_unreconciliation_reverses_in_a_reopened_year(pool: MySqlPool) {
    let m = monter(&pool).await;
    let fy = exercice_2019(&pool, m.s.company_id).await;
    let a = facture(&pool, &m, "R-1", dec!(100.00), en_2019(6, 1), None).await;
    let b = facture(&pool, &m, "R-2", dec!(200.00), en_2019(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), en_2019(6, 15)).await;
    regler(&pool, &m, bt, &[a, b], None).await.expect("réglé");
    let (c, u) = (m.s.company_id, m.s.admin_user_id);
    kesh_db::repositories::fiscal_years::close(&pool, u, c, fy)
        .await
        .expect("clôture de 2019");
    kesh_db::repositories::fiscal_years::reopen(&pool, u, c, fy, "correction".into())
        .await
        .expect("réouverture de 2019");
    let du_jour: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE fiscal_year_id = ?")
            .bind(m.s.fiscal_year_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    let undone = reconciliation_cancel::cancel(&pool, c, bt, acteur(&m))
        .await
        .expect("dé-rapproché dans l'exercice rouvert");
    let (date, exercice): (NaiveDate, i64) =
        sqlx::query_as("SELECT entry_date, fiscal_year_id FROM journal_entries WHERE id = ?")
            .bind(undone.reversal_journal_entry_id.expect("contre-passée"))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((date, exercice), (en_2019(6, 15), fy));
    let apres: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE fiscal_year_id = ?")
            .bind(m.s.fiscal_year_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(apres, du_jour, "rien dans l'exercice du jour");
    kesh_db::repositories::fiscal_years::close(&pool, u, c, fy)
        .await
        .expect("2019 se clôture de nouveau");
}

#[sqlx::test(migrations = "./test-schema")]
async fn group_unreconciliation_refuses_a_credited_invoice_and_names_it(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker as B;
    let m = monter(&pool).await;
    let (_, b, bt, _) = groupe_regle(&pool, &m, "C").await;
    // L'état hérité qu'une facture réglée créditée représente (rang 1).
    sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
        .bind(b)
        .execute(&pool)
        .await
        .unwrap();
    let nommee = refus_du_derapprochement(&pool, &m, bt, B::InvoiceCredited).await;
    assert_eq!(
        nommee,
        Some(GroupedDocument::Invoice(b)),
        "la facture créditée"
    );
}

// ---------------------------------------------------------------------------
// Revue P1 (A-2) — AC8, AC9 : l'avoir et le délettrage manuel, avec leur motif
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn a_grouped_invoice_refuses_a_credit_note_and_a_manual_unlettering(pool: MySqlPool) {
    use kesh_db::repositories::{credit_notes, letterings};
    let m = monter(&pool).await;
    let (a, _, _, _) = groupe_regle(&pool, &m, "K").await;
    // AC8 — l'avoir : la facture est réglée.
    let avoir = credit_notes::create_credit_note(
        &pool,
        kesh_db::entities::NewCreditNote {
            company_id: m.s.company_id,
            invoice_id: a,
            date: jour(11, 17),
        },
        m.s.admin_user_id,
    )
    .await;
    assert!(
        matches!(avoir, Err(DbError::CreditNoteBlockedBySettlement { invoice_id, .. }) if invoice_id == a),
        "{avoir:?}"
    );
    // AC9 — le délettrage manuel du groupe de la facture : un groupe `document`.
    let key: i64 = sqlx::query_scalar(
        "SELECT jel.lettering_key FROM journal_entry_lines jel \
         JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         WHERE i.id = ? AND jel.lettering_key IS NOT NULL LIMIT 1",
    )
    .bind(a)
    .fetch_one(&pool)
    .await
    .expect("la vente est lettrée");
    let mut tx = pool.begin().await.unwrap();
    let delettre = letterings::dissolve_group_in_tx(
        &mut tx,
        m.s.company_id,
        key,
        letterings::Mode::Manual,
        acteur(&m),
    )
    .await;
    drop(tx);
    assert!(
        matches!(delettre, Err(DbError::LetteringIsDocument)),
        "{delettre:?}"
    );
}

// ---------------------------------------------------------------------------
// Revue P1 (A-3, E-2) — un test par variante de `GroupRefusal`, empreinte
// comparée après une transaction VALIDÉE (preuve de D6)
// ---------------------------------------------------------------------------

/// Le règlement du groupe `ids` par `bt` est refusé, rien d'écrit.
async fn refus_du_groupe(
    pool: &MySqlPool,
    m: &Montage,
    bt: i64,
    ids: &[i64],
    difference_account_id: Option<i64>,
) -> GroupRefusal {
    let avant = empreinte(pool).await;
    let refus = regler(pool, m, bt, ids, difference_account_id)
        .await
        .expect_err("refusé");
    assert_eq!(empreinte(pool).await, avant, "rien d'écrit après {refus:?}");
    refus
}

#[sqlx::test(migrations = "./test-schema")]
async fn refusals_after_the_first_write_leave_nothing(pool: MySqlPool) {
    let m = monter(&pool).await;
    // PeriodLocked — l'écriture est refusée par la borne de période.
    let a = facture(&pool, &m, "W-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "W-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(jour(6, 30))
        .bind(m.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        refus_du_groupe(&pool, &m, bt, &[a, b], None).await,
        GroupRefusal::PeriodLocked { .. }
    ));
    sqlx::query("UPDATE companies SET books_locked_through = NULL WHERE id = ?")
        .bind(m.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    // FiscalYearInvalid — aucun exercice ne couvre la date de valeur.
    let tard = transaction(
        &pool,
        &m,
        dec!(300.00),
        NaiveDate::from_ymd_opt(2031, 2, 1).unwrap(),
    )
    .await;
    assert!(matches!(
        refus_du_groupe(&pool, &m, tard, &[a, b], None).await,
        GroupRefusal::FiscalYearInvalid { .. }
    ));
    // TransactionNotPending — à l'étape 10, après l'écriture, les règlements et
    // les factures : la transaction a changé depuis l'instantané du geste.
    let mut tx = pool.begin().await.unwrap();
    kesh_db::repositories::invoice_settlements::amount_due(&mut *tx, a)
        .await
        .unwrap();
    sqlx::query("UPDATE bank_transactions SET version = version + 1 WHERE id = ?")
        .bind(bt)
        .execute(&pool)
        .await
        .unwrap();
    let avant = empreinte(&pool).await;
    let out = grouped_settlements::settle_invoices_in_tx(
        &mut tx,
        m.s.company_id,
        m.bank_account_id,
        bt,
        &[a, b],
        None,
        1,
        acteur(&m),
        jour(11, 17),
    )
    .await
    .expect("aucune erreur de base");
    assert_eq!(out, Err(GroupRefusal::TransactionNotPending));
    tx.commit().await.unwrap();
    assert_eq!(
        empreinte(&pool).await,
        avant,
        "rien d'écrit après TransactionNotPending"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn a_later_closed_year_refuses_the_group(pool: MySqlPool) {
    let m = monter(&pool).await;
    exercice_2019(&pool, m.s.company_id).await;
    let a = facture(&pool, &m, "Z-1", dec!(100.00), en_2019(6, 1), None).await;
    let b = facture(&pool, &m, "Z-2", dec!(200.00), en_2019(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), en_2019(6, 15)).await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(m.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        refus_du_groupe(&pool, &m, bt, &[a, b], None).await,
        GroupRefusal::LaterFiscalYearClosed { .. }
    ));
}

#[sqlx::test(migrations = "./test-schema")]
async fn each_invoice_and_bank_account_refusal_writes_nothing(pool: MySqlPool) {
    use kesh_db::errors::SettlementAccountRole as R;
    let m = monter(&pool).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let a = facture(&pool, &m, "V-1", dec!(100.00), jour(6, 1), None).await;
    let refus_facture = |r: GroupRefusal| match r {
        GroupRefusal::InvoiceNotEligible { reason, .. } => Some(reason),
        _ => None,
    };

    // AlreadyPaid.
    let b = facture(&pool, &m, "V-2", dec!(200.00), jour(6, 1), None).await;
    sqlx::query("UPDATE invoices SET paid_at = NOW() WHERE id = ?")
        .bind(b)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        refus_facture(refus_du_groupe(&pool, &m, bt, &[a, b], None).await),
        Some(IneligibleReason::AlreadyPaid)
    );
    // NothingDue : soldée par un règlement, `paid_at` effacé ensuite (l'état
    // que le rang « déjà payée » ne voit pas).
    let c = facture(&pool, &m, "V-3", dec!(200.00), jour(6, 1), None).await;
    let version_c: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(c)
        .fetch_one(&pool)
        .await
        .unwrap();
    kesh_db::repositories::invoice_settlements_write::settle_invoice(
        &pool,
        m.s.admin_user_id,
        m.s.company_id,
        c,
        kesh_db::entities::SettlementChoice::InternalAccount {
            account_id: m.s.accounts["1000"],
        },
        dec!(200.00),
        jour(6, 2),
        version_c,
    )
    .await
    .expect("soldée");
    sqlx::query("UPDATE invoices SET paid_at = NULL WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        refus_facture(refus_du_groupe(&pool, &m, bt, &[a, c], None).await),
        Some(IneligibleReason::NothingDue)
    );
    // JournalEntryNotSet : INATTEIGNABLE en base — la CHECK
    // `chk_invoices_validated_has_je` refuse une facture `validated` sans
    // écriture de vente (4025, constaté au premier passage de ce test,
    // `p1-vert-entrelacement.log`), et une facture non validée est refusée
    // avant (`NotValidated`). La garde reste, défensive.
    let d = facture(&pool, &m, "V-4", dec!(200.00), jour(6, 1), None).await;
    let refus_check = sqlx::query("UPDATE invoices SET journal_entry_id = NULL WHERE id = ?")
        .bind(d)
        .execute(&pool)
        .await;
    assert_eq!(
        numero_mariadb(refus_check),
        Some(4025),
        "la CHECK protège l'invariant"
    );
    // SaleEntryMalformed : l'écriture de vente n'est pas une écriture de la
    // société (identifiant inexistant, clés étrangères suspendues le temps du
    // montage).
    let e = facture(&pool, &m, "V-5", dec!(200.00), jour(6, 1), None).await;
    {
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("UPDATE invoices SET journal_entry_id = 999999 WHERE id = ?")
            .bind(e)
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    assert!(matches!(
        refus_du_groupe(&pool, &m, bt, &[a, e], None).await,
        GroupRefusal::SaleEntryMalformed {
            sale_entry_id: 999999,
            ..
        }
    ));

    let f = facture(&pool, &m, "V-6", dec!(200.00), jour(6, 1), None).await;
    let poser = |sql: &'static str| {
        let pool = pool.clone();
        async move { sqlx::query(sql).execute(&pool).await.unwrap() }
    };
    // ClaimAccount { Counterparty, bank_account_id: Some } : la banque est la créance.
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(m.s.accounts["1100"])
        .bind(m.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    match refus_du_groupe(&pool, &m, bt, &[a, f], None).await {
        GroupRefusal::ClaimAccount {
            role: R::Counterparty,
            bank_account_id,
            account_id,
            piece,
            ..
        } => {
            assert_eq!(bank_account_id, Some(m.bank_account_id));
            assert_eq!(account_id, m.s.accounts["1100"]);
            assert!(piece.is_some());
        }
        autre => panic!("{autre:?}"),
    }
    // BankAccountNotConfigured.
    poser("UPDATE bank_accounts SET journal_account_id = NULL").await;
    assert_eq!(
        refus_du_groupe(&pool, &m, bt, &[a, f], None).await,
        GroupRefusal::BankAccountNotConfigured
    );
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(m.bank_ledger)
        .bind(m.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    // BankAccountClosed.
    poser("UPDATE bank_accounts SET closed_on = '2026-11-01'").await;
    assert_eq!(
        refus_du_groupe(&pool, &m, bt, &[a, f], None).await,
        GroupRefusal::BankAccountClosed
    );
    poser("UPDATE bank_accounts SET closed_on = NULL").await;
    // BankAccountNotFound (archivé).
    poser("UPDATE bank_accounts SET archived = TRUE").await;
    assert_eq!(
        refus_du_groupe(&pool, &m, bt, &[a, f], None).await,
        GroupRefusal::BankAccountNotFound
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn rounding_refusals_write_nothing(pool: MySqlPool) {
    use kesh_db::errors::SettlementAccountRole as R;
    let m = monter(&pool).await;
    // `r = −0.0049` : 10.0049 + 10.00 payés 20.00 — compte d'arrondi requis.
    let a = facture(&pool, &m, "R-1", dec!(10.0049), jour(6, 1), None).await;
    let b = facture(&pool, &m, "R-2", dec!(10.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(20.00), jour(6, 15)).await;
    assert_eq!(
        refus_du_groupe(&pool, &m, bt, &[a, b], None).await,
        GroupRefusal::RoundingAccountNotConfigured
    );
    // ClaimAccount { Rounding } : le compte d'arrondi désigné est le compte de
    // créance d'une facture — la vente de `c` le débite.
    let arrondi = designate_rounding_account(&pool, m.s.company_id)
        .await
        .expect("compte d'arrondi");
    let c = facture(&pool, &m, "R-3", dec!(10.0049), jour(6, 1), None).await;
    sqlx::query(
        "UPDATE journal_entry_lines SET account_id = ? \
         WHERE account_id = ? AND entry_id = (SELECT journal_entry_id FROM invoices WHERE id = ?)",
    )
    .bind(arrondi)
    .bind(m.s.accounts["1100"])
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    assert!(matches!(
        refus_du_groupe(&pool, &m, bt, &[b, c], None).await,
        GroupRefusal::ClaimAccount {
            role: R::Rounding,
            bank_account_id: None,
            ..
        }
    ));
}

/// D6 — le refus `RaceDuringUpdate` (après l'écriture et les règlements) ne
/// laisse rien : empreinte comparée après une transaction validée.
#[sqlx::test(migrations = "./test-schema")]
async fn a_race_during_update_leaves_nothing(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "N-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "N-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let mut tx = pool.begin().await.unwrap();
    kesh_db::repositories::invoice_settlements::amount_due(&mut *tx, b)
        .await
        .unwrap();
    // Une autre connexion relève la version de `b` (sans en changer le reste).
    sqlx::query("UPDATE invoices SET version = version + 1 WHERE id = ?")
        .bind(b)
        .execute(&pool)
        .await
        .unwrap();
    let avant = empreinte(&pool).await;
    let out = grouped_settlements::settle_invoices_in_tx(
        &mut tx,
        m.s.company_id,
        m.bank_account_id,
        bt,
        &[a, b],
        None,
        1,
        acteur(&m),
        jour(11, 17),
    )
    .await
    .expect("aucune erreur de base");
    assert!(
        matches!(
            &out,
            Err(GroupRefusal::InvoiceNotEligible {
                piece,
                reason: IneligibleReason::RaceDuringUpdate
            }) if piece.document == GroupedDocument::Invoice(b)
        ),
        "{out:?}"
    );
    tx.commit().await.unwrap();
    assert_eq!(
        empreinte(&pool).await,
        avant,
        "rien d'écrit après RaceDuringUpdate"
    );
}

// ---------------------------------------------------------------------------
// Revue P1 (A-3, E-2) — en écritures réelles
// ---------------------------------------------------------------------------

/// Trois factures, dont une déjà réglée en partie ; écart de −5.00 accepté au
/// plafond par défaut ; clés d'audit relues en base.
#[sqlx::test(migrations = "./test-schema")]
async fn three_invoices_one_partly_settled_and_a_five_franc_gap(pool: MySqlPool) {
    let m = monter(&pool).await;
    let ecart = m.s.accounts["4000"];
    let a = facture(&pool, &m, "P3-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "P3-2", dec!(200.00), jour(6, 1), None).await;
    let c = facture(&pool, &m, "P3-3", dec!(50.00), jour(6, 1), None).await;
    // `a` est déjà réglée de 30.00 par un règlement unitaire.
    let version: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(a)
        .fetch_one(&pool)
        .await
        .unwrap();
    kesh_db::repositories::invoice_settlements_write::settle_invoice(
        &pool,
        m.s.admin_user_id,
        m.s.company_id,
        a,
        kesh_db::entities::SettlementChoice::InternalAccount {
            account_id: m.s.accounts["1000"],
        },
        dec!(30.00),
        jour(6, 5),
        version,
    )
    .await
    .expect("règlement partiel");
    // Restes : 70 + 200 + 50 = 320 ; payé 315 : écart de −5.00.
    let bt = transaction(&pool, &m, dec!(315.00), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b, c], Some(ecart))
        .await
        .expect("réglé");
    assert_eq!(done.invoices.len(), 3);
    assert_eq!(
        done.invoices[0].amount,
        dec!(70.0000),
        "le reste de la facture partielle"
    );
    assert_eq!(done.decomposition.gap, dec!(-5.00));
    for id in [a, b, c] {
        assert_eq!(reste(&pool, id).await, Decimal::ZERO);
    }
    // La facture partielle : sa vente, son règlement unitaire et sa ligne du
    // groupe forment un seul groupe de lettrage.
    let key: Option<i64> =
        sqlx::query_scalar("SELECT lettering_key FROM journal_entry_lines WHERE id = ?")
            .bind(done.invoices[0].line_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let membres: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = ?")
            .bind(key.expect("lettrée"))
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(membres, 3, "vente, règlement unitaire, ligne du groupe");
    // L'écart de 5.00 au débit du compte choisi.
    let debit: Decimal = sqlx::query_scalar(
        "SELECT debit FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(done.journal_entry_id)
    .bind(ecart)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(debit, dec!(5.00));
    // Les clés d'audit, relues en base.
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.accepted' \
         AND entity_id = ?",
    )
    .bind(bt)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        details["gap"]
            .as_str()
            .map(|s| s.parse::<Decimal>().unwrap()),
        Some(dec!(-5.00))
    );
    assert_eq!(details["difference_account_id"], ecart);
    assert_eq!(
        details["rounding_difference"]
            .as_str()
            .map(|s| s.parse::<Decimal>().unwrap()),
        Some(Decimal::ZERO)
    );
    assert_eq!(details["invoices"].as_array().map(Vec::len), Some(3));
}

/// Cinquante factures réglées en base ; le libellé de l'écriture coupé à 500
/// **caractères**, numéros accentués (revue P2, LOW-4 : la coupe se fait par
/// caractère, jamais au milieu d'un caractère multi-octets).
#[sqlx::test(migrations = "./test-schema")]
async fn fifty_invoices_settle_and_the_label_is_cut_at_500(pool: MySqlPool) {
    let m = monter(&pool).await;
    let mut ids = Vec::new();
    for k in 0..50 {
        ids.push(
            facture(
                &pool,
                &m,
                &format!("FACTURE-ÉTÉ-ÀÔÜ-{k:04}"),
                dec!(10.00),
                jour(6, 1),
                None,
            )
            .await,
        );
    }
    let bt = transaction(&pool, &m, dec!(500.00), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &ids, None).await.expect("cinquante");
    assert_eq!(done.invoices.len(), 50);
    let description: String =
        sqlx::query_scalar("SELECT description FROM journal_entries WHERE id = ?")
            .bind(done.journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(description.chars().count(), 500, "{description}");
    assert!(
        description.len() > 500,
        "des caractères multi-octets dans le libellé coupé"
    );
    assert!(description.contains("ÉTÉ-ÀÔÜ"));
    assert!(description.ends_with("..."));
    let lignes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?")
            .bind(done.journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lignes, 51, "la banque et une créance par facture");
}

// ---------------------------------------------------------------------------
// Revue P2 — ordres épinglés, point de sauvegarde, invariants
// ---------------------------------------------------------------------------

/// Le numéro d'erreur MariaDB d'un résultat, s'il en porte un.
fn numero_mariadb<T>(res: Result<T, sqlx::Error>) -> Option<u16> {
    res.err()?
        .as_database_error()?
        .try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
        .map(|e| e.number())
}

/// ⛔ Revue P2, MEDIUM-1 — l'ordre de C-e431a-15 : les **têtes de toutes les
/// factures** (rang 1, rang 1 bis) avant la **queue de l'écriture** (rangs 2 à
/// 5). La seconde facture `b` est créditée **et** l'exercice du règlement est
/// clos : attendu `InvoiceCredited` nommant `b`. L'ancien ordre (queue après
/// chaque tête) rendait `FiscalYearClosed` nommant `a`.
#[sqlx::test(migrations = "./test-schema")]
async fn every_invoice_head_comes_before_the_entry_ranks(pool: MySqlPool) {
    use kesh_db::errors::SettlementCancelBlocker as B;
    let m = monter(&pool).await;
    let fy = exercice_2019(&pool, m.s.company_id).await;
    let a = facture(&pool, &m, "O-1", dec!(100.00), en_2019(6, 1), None).await;
    let b = facture(&pool, &m, "O-2", dec!(200.00), en_2019(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), en_2019(6, 15)).await;
    regler(&pool, &m, bt, &[a, b], None).await.expect("réglé");
    sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
        .bind(b)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy)
        .execute(&pool)
        .await
        .unwrap();
    let nommee = refus_du_derapprochement(&pool, &m, bt, B::InvoiceCredited).await;
    assert_eq!(
        nommee,
        Some(GroupedDocument::Invoice(b)),
        "la tête de la seconde facture passe devant l'exercice clos de l'écriture"
    );
}

/// Revue P2, LOW-1 — au succès, le point de sauvegarde est relâché : un
/// `ROLLBACK TO` le cherche en vain (1305).
#[sqlx::test(migrations = "./test-schema")]
async fn a_successful_group_releases_its_savepoint(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "S-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "S-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let mut tx = pool.begin().await.unwrap();
    grouped_settlements::settle_invoices_in_tx(
        &mut tx,
        m.s.company_id,
        m.bank_account_id,
        bt,
        &[a, b],
        None,
        1,
        acteur(&m),
        jour(11, 17),
    )
    .await
    .expect("aucune erreur de base")
    .expect("réglé");
    let sonde = sqlx::query("ROLLBACK TO SAVEPOINT grouped_settlement")
        .execute(&mut *tx)
        .await;
    assert_eq!(numero_mariadb(sonde), Some(1305), "le point est relâché");
    tx.commit().await.unwrap();
    assert_eq!(reste(&pool, a).await, Decimal::ZERO, "le règlement tient");
}

/// Revue P2, LOW-2 — la ligne d'écart absente d'une écriture groupée : la vue
/// rend `Invariant` (`difference_account_of`), jamais « sans écart ».
#[sqlx::test(migrations = "./test-schema")]
async fn a_missing_gap_line_is_an_invariant_for_the_view(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "G-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "G-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(299.60), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], Some(m.s.accounts["4000"]))
        .await
        .expect("réglé avec écart");
    // Sonde positive : la vue relit le compte d'écart.
    let vue = reconciliation_cancel::get_view(&pool, m.s.company_id, bt)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        vue.grouped_difference_account_id,
        Some(m.s.accounts["4000"])
    );
    sqlx::query(
        "DELETE FROM journal_entry_lines WHERE entry_id = ? AND account_id = ? AND debit > 0",
    )
    .bind(done.journal_entry_id)
    .bind(m.s.accounts["4000"])
    .execute(&pool)
    .await
    .unwrap();
    let res = reconciliation_cancel::get_view(&pool, m.s.company_id, bt).await;
    assert!(
        matches!(&res, Err(DbError::Invariant(msg)) if msg.contains("ligne d'écart absente")),
        "{res:?}"
    );
}

/// Revue P2 de la 43-1a (LOW-2), adapté par la 43-1b (revue P2, LOW-7) — une
/// écriture classée groupée **fournisseur malformée** : ses règlements clients
/// retirés, une facture fournisseur fabriquée qui pointe la ligne de
/// **banque**, forme qu'aucun paiement ne produit. La vue rend `Invariant`
/// (le bras fournisseur de `difference_account_of`), jamais une réponse ni
/// « annulable ». Avant la 43-1b, ce test épinglait le message « aucun
/// règlement client » ; la branche défensive « aucun règlement » du bras
/// fournisseur est tenue par
/// `grouped_settlements_supplier::a_group_cancel_on_an_entry_without_settlement_is_an_invariant`.
#[sqlx::test(migrations = "./test-schema")]
async fn a_malformed_supplier_group_is_an_invariant_for_the_view(pool: MySqlPool) {
    let m = monter(&pool).await;
    let (_, _, bt, entry) = groupe_regle(&pool, &m, "F").await;
    let ligne: i64 = sqlx::query_scalar(
        "SELECT id FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order LIMIT 1",
    )
    .bind(entry)
    .fetch_one(&pool)
    .await
    .unwrap();
    let contact: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
         VALUES (?, 'Entreprise', 'Fournisseur', TRUE) RETURNING id",
    )
    .bind(m.s.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, invoice_date, status, \
         total_amount, purchase_journal_entry_id, settlement_type, settlement_bank_account_id, \
         settlement_journal_entry_id, settlement_journal_entry_line_id, paid_at) \
         VALUES (?, ?, ?, 'paid', 300.00, ?, 'bank_transfer', ?, ?, ?, NOW())",
    )
    .bind(m.s.company_id)
    .bind(contact)
    .bind(jour(5, 1))
    .bind(entry)
    .bind(m.bank_account_id)
    .bind(entry)
    .bind(ligne)
    .execute(&pool)
    .await
    .expect("facture fournisseur qui pointe le groupe");
    sqlx::query("DELETE FROM invoice_settlements WHERE journal_entry_id = ?")
        .bind(entry)
        .execute(&pool)
        .await
        .unwrap();
    let res = reconciliation_cancel::get_view(&pool, m.s.company_id, bt).await;
    assert!(matches!(&res, Err(DbError::Invariant(_))), "{res:?}");
}

/// Revue P2, LOW-5 — une partie de l'ordre de l'étape 4 (AC5.4) : le reste dû
/// nul passe devant la date, l'écriture de vente malformée aussi. Ni
/// `AlreadyPaid` ni `NotValidated` contre `NothingDue`, ni `SaleEntryMalformed`
/// contre `NothingDue` ne sont épinglés ici (revue P3, LOW-1).
#[sqlx::test(migrations = "./test-schema")]
async fn step_four_refuses_before_the_date_when_nothing_is_due_or_the_sale_is_malformed(
    pool: MySqlPool,
) {
    let m = monter(&pool).await;
    let bt = transaction(&pool, &m, dec!(300.00), jour(6, 15)).await;
    let a = facture(&pool, &m, "Q-1", dec!(100.00), jour(6, 1), None).await;
    // Soldée ET datée bien après la transaction : `NothingDue`, non la date.
    let c = facture(&pool, &m, "Q-2", dec!(200.00), jour(9, 1), None).await;
    let version_c: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(c)
        .fetch_one(&pool)
        .await
        .unwrap();
    kesh_db::repositories::invoice_settlements_write::settle_invoice(
        &pool,
        m.s.admin_user_id,
        m.s.company_id,
        c,
        kesh_db::entities::SettlementChoice::InternalAccount {
            account_id: m.s.accounts["1000"],
        },
        dec!(200.00),
        jour(9, 2),
        version_c,
    )
    .await
    .expect("soldée");
    sqlx::query("UPDATE invoices SET paid_at = NULL WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    match refus_du_groupe(&pool, &m, bt, &[a, c], None).await {
        GroupRefusal::InvoiceNotEligible { reason, .. } => {
            assert_eq!(reason, IneligibleReason::NothingDue)
        }
        autre => panic!("NothingDue attendu, reçu {autre:?}"),
    }
    // Écriture de vente malformée ET datée après la transaction :
    // `SaleEntryMalformed`, non la date.
    let e = facture(&pool, &m, "Q-3", dec!(200.00), jour(9, 1), None).await;
    {
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("UPDATE invoices SET journal_entry_id = 999999 WHERE id = ?")
            .bind(e)
            .execute(&mut *conn)
            .await
            .unwrap();
        sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    assert!(matches!(
        refus_du_groupe(&pool, &m, bt, &[a, e], None).await,
        GroupRefusal::SaleEntryMalformed {
            sale_entry_id: 999999,
            ..
        }
    ));
    // Sonde : la date seule, sur une facture saine, est bien refusée.
    let f = facture(&pool, &m, "Q-4", dec!(200.00), jour(9, 1), None).await;
    assert!(matches!(
        refus_du_groupe(&pool, &m, bt, &[a, f], None).await,
        GroupRefusal::PaymentBeforeInvoiceDate { .. }
    ));
}

// ---------------------------------------------------------------------------
// Story 48-14 (#1255, AC9) — caractérisation du dé-rapprochement client
// ---------------------------------------------------------------------------

/// L'empreinte d'un groupe client : son écriture, sa transaction, ses
/// règlements, et chaque facture (`paid_at`, version, reste dû, écriture de
/// vente avec son lettrage).
async fn empreinte_du_groupe(pool: &MySqlPool, entry_id: i64, bt: i64, ids: &[i64]) -> Value {
    let reglements: Vec<(i64, Decimal, NaiveDate, Option<i64>)> = sqlx::query_as(
        "SELECT invoice_id, amount, settled_on, journal_entry_line_id \
         FROM invoice_settlements WHERE journal_entry_id = ? ORDER BY id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .unwrap();
    let mut factures = Vec::new();
    for id in ids {
        factures.push(etat_facture(pool, *id).await);
    }
    json!({
        "entry": caracterisation::ecriture(pool, entry_id).await,
        "transaction": caracterisation::transaction_bancaire(pool, bt).await,
        "settlements": reglements
            .into_iter()
            .map(|(i, a, d, l)| json!([i, a.to_string(), d.to_string(), l]))
            .collect::<Vec<_>>(),
        "invoices": factures,
    })
}

/// Une facture client : `paid_at` (posé ou non), version, reste dû, et son
/// écriture de vente (lettrage compris).
async fn etat_facture(pool: &MySqlPool, id: i64) -> Value {
    let (paid_at, version, sale): (Option<chrono::NaiveDateTime>, i32, i64) =
        sqlx::query_as("SELECT paid_at, version, journal_entry_id FROM invoices WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap();
    json!({
        "paid": paid_at.is_some(),
        "version": version,
        "due": reste(pool, id).await.to_string(),
        "sale": caracterisation::ecriture(pool, sale).await,
    })
}

/// AC9 (Story 48-14, #1255) — le dé-rapprochement d'un groupe client laisse
/// **exactement** l'état qu'il laissait avant la réécriture du lien défait et
/// de l'audit en aides communes : la transaction, l'écriture inverse (en-tête
/// et lignes), chaque facture et son lettrage, et **toutes** les lignes d'audit
/// écrites par le geste, entières hors horodatage, dans l'ordre. Le geste est
/// joué par une clé d'API ; la transaction portait un `auto_match_rejected_at`
/// non nul (`wasPreviouslyRejected` vrai, colonne remise à `NULL`). Un second
/// groupe, de même statut et même version, n'est pas touché. Les valeurs
/// attendues sont écrites en dur : relevées sur l'ancien code, elles doivent
/// valoir après la réécriture.
#[sqlx::test(migrations = "./test-schema")]
async fn unreconciling_a_group_leaves_the_characterised_state(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = facture(&pool, &m, "K-1", dec!(100.00), jour(6, 1), None).await;
    let b = facture(&pool, &m, "K-2", dec!(200.00), jour(6, 1), None).await;
    let bt = transaction(&pool, &m, dec!(299.60), jour(6, 15)).await;
    let done = regler(&pool, &m, bt, &[a, b], Some(m.s.accounts["4000"]))
        .await
        .expect("réglé");
    // Le second groupe, intact.
    let c = facture(&pool, &m, "K-3", dec!(50.00), jour(6, 1), None).await;
    let d = facture(&pool, &m, "K-4", dec!(70.00), jour(6, 1), None).await;
    let bt2 = transaction(&pool, &m, dec!(120.00), jour(6, 15)).await;
    let done2 = regler(&pool, &m, bt2, &[c, d], None)
        .await
        .expect("second groupe réglé");
    sqlx::query(
        "UPDATE bank_transactions SET auto_match_rejected_at = '2026-06-20 10:00:00.000' \
         WHERE id = ?",
    )
    .bind(bt)
    .execute(&pool)
    .await
    .unwrap();
    // Prémisse de M7 : même statut, même version pour les deux transactions.
    let t1 = caracterisation::transaction_bancaire(&pool, bt).await;
    let t2 = caracterisation::transaction_bancaire(&pool, bt2).await;
    assert_eq!(
        (&t1["status"], &t1["version"]),
        (&t2["status"], &t2["version"])
    );

    let key = caracterisation::cle_api(&pool, m.s.company_id, m.s.admin_user_id).await;
    let cle = Acteur {
        user_id: m.s.admin_user_id,
        api_key_id: Some(key),
    };
    let intact = empreinte_du_groupe(&pool, done2.journal_entry_id, bt2, &[c, d]).await;
    let avant = caracterisation::dernier_audit(&pool).await;

    let undone = reconciliation_cancel::cancel(&pool, m.s.company_id, bt, cle)
        .await
        .expect("dé-rapproché");
    let reversal = undone.reversal_journal_entry_id.expect("contre-passée");

    let etat = json!({
        "transaction": caracterisation::transaction_bancaire(&pool, bt).await,
        "entry": caracterisation::ecriture(&pool, done.journal_entry_id).await,
        "reversal": caracterisation::ecriture(&pool, reversal).await,
        "invoices": [etat_facture(&pool, a).await, etat_facture(&pool, b).await],
        "settlements_left": empreinte_du_groupe(&pool, done.journal_entry_id, bt, &[]).await["settlements"].clone(),
        "audits": caracterisation::audits_depuis(&pool, avant).await,
    });
    // Les identifiants que rend le règlement (ou que le montage a posés) ; les
    // valeurs, elles, sont écrites en dur, relevées sur l'ancien code
    // (`kesh-gate-logs/e4814/carac-releve-ancien-code.log`).
    let entry = done.journal_entry_id;
    let (fy, user, company) = (m.s.fiscal_year_id, m.s.admin_user_id, m.s.company_id);
    let (receivable, gap_account) = (m.s.accounts["1100"], m.s.accounts["4000"]);
    let (g2, g3) = (done.invoices[0].line_id, done.invoices[1].line_id);
    let (s1, s2) = (
        done.invoices[0].settlement_id.unwrap(),
        done.invoices[1].settlement_id.unwrap(),
    );
    let (r2, r3) = (
        caracterisation::ligne(&pool, reversal, 2).await,
        caracterisation::ligne(&pool, reversal, 3).await,
    );
    let mut ventes = Vec::new();
    for id in [a, b] {
        let sale: i64 = sqlx::query_scalar("SELECT journal_entry_id FROM invoices WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        ventes.push((sale, caracterisation::ligne(&pool, sale, 1).await));
    }
    let ((sale_a, la1), (sale_b, lb1)) = (ventes[0], ventes[1]);
    let ligne = |order: i32, account: &str, debit: &str, credit: &str, key: Option<i64>| {
        json!({
            "line_order": order, "account": account, "debit": debit, "credit": credit,
            "project_id": null, "lettering_key": key,
            "lettering_origin": key.map(|_| "reversal"),
        })
    };
    let vente = |number: i64, montant: &str| {
        json!({
            "entry_number": number, "entry_date": "2026-06-01", "fiscal_year_id": fy,
            "journal": "Ventes", "description": "Vente", "reverses_entry_id": null, "version": 1,
            "lines": [
                ligne(1, "1100", montant, "0.0000", None),
                ligne(2, "3000", "0.0000", montant, None),
            ],
        })
    };
    let acte = |action: &str, entity_type: &str, entity_id: i64, details: Value| {
        json!({
            "action": action, "entity_type": entity_type, "entity_id": entity_id,
            "user_id": user, "actor_type": "api_key", "actor_api_key_id": key,
            "actor_label": "admin", "company_id": company, "details_json": details,
        })
    };
    let ligne_lettree = |id: i64, entry_id: i64, number: i64, debit: &str, credit: &str| {
        json!({
            "id": id, "entryId": entry_id, "entryNumber": number, "fiscalYearId": fy,
            "fiscalYearName": "Exercice CI 2020-2030", "debit": debit, "credit": credit,
        })
    };
    let attendu = json!({
        "transaction": {
            "status": "pending", "matched_entry_id": null, "auto_match_rejected_at": null,
            "version": 3,
        },
        "entry": {
            "entry_number": 3, "entry_date": "2026-06-15", "fiscal_year_id": fy,
            "journal": "Banque", "description": "Encaissement groupé de 2 factures : K-1, K-2",
            "reverses_entry_id": null, "version": 1,
            "lines": [
                ligne(1, "1020", "299.6000", "0.0000", None),
                ligne(2, "1100", "0.0000", "100.0000", Some(g2)),
                ligne(3, "1100", "0.0000", "200.0000", Some(g3)),
                ligne(4, "4000", "0.4000", "0.0000", None),
            ],
        },
        // Story 53-6a (#1674) : datée de l'écriture du groupe (15 juin), non du
        // jour — seul changement d'attendu de ce relevé, avec `dating` ci-dessous.
        "reversal": {
            "entry_number": 7, "entry_date": "2026-06-15", "fiscal_year_id": fy,
            "journal": "OD", "description": "Contre-passation écriture n° 3",
            "reverses_entry_id": entry, "version": 1,
            "lines": [
                ligne(1, "1020", "0.0000", "299.6000", None),
                ligne(2, "1100", "100.0000", "0.0000", Some(g2)),
                ligne(3, "1100", "200.0000", "0.0000", Some(g3)),
                ligne(4, "4000", "0.0000", "0.4000", None),
            ],
        },
        "invoices": [
            { "paid": false, "version": 3, "due": "100.0000", "sale": vente(1, "100.0000") },
            { "paid": false, "version": 3, "due": "200.0000", "sale": vente(2, "200.0000") },
        ],
        "settlements_left": [],
        "audits": [
            acte("lettering.removed", "lettering", la1, json!({
                "accountId": receivable, "accountNumber": "1100", "code": "A",
                "documentId": a, "documentNumber": "K-1", "documentType": "invoice",
                "origin": "document",
                "lines": [
                    ligne_lettree(la1, sale_a, 1, "100.0000", "0.0000"),
                    ligne_lettree(g2, entry, 3, "0.0000", "100.0000"),
                ],
            })),
            acte("lettering.removed", "lettering", lb1, json!({
                "accountId": receivable, "accountNumber": "1100", "code": "C",
                "documentId": b, "documentNumber": "K-2", "documentType": "invoice",
                "origin": "document",
                "lines": [
                    ligne_lettree(lb1, sale_b, 2, "200.0000", "0.0000"),
                    ligne_lettree(g3, entry, 3, "0.0000", "200.0000"),
                ],
            })),
            acte("journal_entry.created", "journal_entry", reversal, json!({
                "id": reversal, "entryNumber": 7, "entryDate": "2026-06-15", "journal": "OD",
                "description": "Contre-passation écriture n° 3", "version": 1,
                "lines": [
                    { "lineOrder": 1, "accountId": m.bank_ledger, "debit": "0.0000", "credit": "299.6000", "projectId": null },
                    { "lineOrder": 2, "accountId": receivable, "debit": "100.0000", "credit": "0.0000", "projectId": null },
                    { "lineOrder": 3, "accountId": receivable, "debit": "200.0000", "credit": "0.0000", "projectId": null },
                    { "lineOrder": 4, "accountId": gap_account, "debit": "0.0000", "credit": "0.4000", "projectId": null },
                ],
            })),
            acte("journal_entry.reversed", "journal_entry", entry, json!({
                "reversalJournalEntryId": reversal, "dating": "originDate",
            })),
            acte("lettering.created", "lettering", g2, json!({
                "accountId": receivable, "accountNumber": "1100", "code": "F", "origin": "reversal",
                "lines": [
                    ligne_lettree(g2, entry, 3, "0.0000", "100.0000"),
                    ligne_lettree(r2, reversal, 7, "100.0000", "0.0000"),
                ],
            })),
            acte("lettering.created", "lettering", g3, json!({
                "accountId": receivable, "accountNumber": "1100", "code": "G", "origin": "reversal",
                "lines": [
                    ligne_lettree(g3, entry, 3, "0.0000", "200.0000"),
                    ligne_lettree(r3, reversal, 7, "200.0000", "0.0000"),
                ],
            })),
            acte("invoice.settlement_cancelled", "invoice", a, json!({
                "settlementId": s1, "settledAmount": "100.0000", "settlementJournalEntryId": entry,
                "reversalJournalEntryId": reversal, "amountDueAfter": "100.0000", "grouped": true,
            })),
            acte("invoice.settlement_cancelled", "invoice", b, json!({
                "settlementId": s2, "settledAmount": "200.0000", "settlementJournalEntryId": entry,
                "reversalJournalEntryId": reversal, "amountDueAfter": "200.0000", "grouped": true,
            })),
            acte("reconciliation.cancelled", "bank_transaction", bt, json!({
                "kind": "grouped_settlement", "matchedEntryId": entry,
                "reversalJournalEntryId": reversal, "unlinkOnly": false, "invoiceId": null,
                "settlementId": null, "supplierInvoiceId": null,
                "invoices": [
                    { "invoice_id": a, "settlement_id": s1, "number": "K-1" },
                    { "invoice_id": b, "settlement_id": s2, "number": "K-2" },
                ],
                "amount": "299.60", "wasPreviouslyRejected": true,
            })),
        ],
    });
    assert_eq!(etat, attendu, "l'état laissé par le dé-rapprochement");
    assert_eq!(
        empreinte_du_groupe(&pool, done2.journal_entry_id, bt2, &[c, d]).await,
        intact,
        "le second groupe n'est pas touché"
    );
}
