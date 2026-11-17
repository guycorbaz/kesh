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
use kesh_db::entities::{NewInvoice, NewInvoiceLine, NewJournalEntry, NewJournalEntryLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::grouped_settlements::{
    self, GroupRefusal, GroupedDocument, IneligibleReason,
};
use kesh_db::repositories::{invoices, journal_entries, reconciliation_cancel};
use kesh_db::test_fixtures::{SeededCompany, designate_rounding_account};

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
            }],
        },
    )
    .await
    .expect("facture créée")
    .0;
    let je = journal_entries::create(
        pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: date,
            journal: Journal::Ventes,
            description: "Vente".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: s.accounts["1100"],
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: s.accounts["3000"],
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
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
        "SELECT COALESCE(SUM(version), 0) FROM invoices",
        "SELECT COUNT(*) FROM invoices WHERE paid_at IS NOT NULL",
        "SELECT COALESCE(SUM(version), 0) FROM bank_transactions",
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

/// La fiche — deux restes de 10.0050 (affichés 10.01). Contre 20.02 : arrondi
/// de +0.0100 au crédit, aucun écart. Contre 20.01 : l'écart d'un centime est
/// dans l'arrondi automatique (`centime(2 × 0.005)`), qui annule les fractions —
/// ni ligne d'arrondi ni ligne d'écart. Contre 20.00 : arrondi de +0.0100 au
/// crédit **et** écart de 0.02 au débit du compte choisi. Les écritures
/// s'équilibrent, et la vue relit le compte d'écart sur sa ligne.
#[sqlx::test(migrations = "./test-schema")]
async fn sub_centime_remainders_go_to_rounding_and_the_gap_to_the_chosen_account(pool: MySqlPool) {
    let m = monter(&pool).await;
    let arrondi = designate_rounding_account(&pool, m.s.company_id)
        .await
        .unwrap();
    let ecart = m.s.accounts["4000"];
    for (paye, attendu_arrondi, attendu_gap) in [
        (dec!(20.02), dec!(0.0100), dec!(0.00)),
        (dec!(20.01), dec!(0.0000), dec!(0.00)),
        (dec!(20.00), dec!(0.0100), dec!(-0.02)),
    ] {
        let a = facture(
            &pool,
            &m,
            &format!("S-{paye}-1"),
            dec!(10.005),
            jour(6, 1),
            None,
        )
        .await;
        let b = facture(
            &pool,
            &m,
            &format!("S-{paye}-2"),
            dec!(10.005),
            jour(6, 1),
            None,
        )
        .await;
        assert_eq!(reste(&pool, a).await, dec!(10.0050), "montage : reste brut");
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
        let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, jour(11, 17))
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
    // Moins reçu de 5.00 : accepté au plafond par défaut.
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
    let view = reconciliation_cancel::get_view(&pool, m.s.company_id, bt, jour(11, 17))
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
    let changed = reconciliation_cancel::cancel_in_tx(
        &mut tx,
        m.s.company_id,
        bt,
        acteur_cle,
        Some(true),
        jour(11, 17),
    )
    .await;
    assert!(matches!(
        changed,
        Err(DbError::ReconciliationCancelChanged { unlink_only: false })
    ));
    drop(tx);

    let undone = reconciliation_cancel::cancel(&pool, m.s.company_id, bt, acteur_cle, jour(11, 17))
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
        "SELECT COUNT(*), CAST(SUM(api_key_id = ?) AS SIGNED) FROM audit_log WHERE id > ?",
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
        jour(11, 17),
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
                NewJournalEntryLine {
                    account_id: m.bank_ledger,
                    debit: dec!(300.00),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: m.s.accounts["1100"],
                    debit: Decimal::ZERO,
                    credit: dec!(300.00),
                    project_id: None,
                },
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
    // unitaire, une facture fournisseur payée.
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
                NewJournalEntryLine {
                    account_id: m.bank_ledger,
                    debit: dec!(80.00),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: m.s.accounts["1100"],
                    debit: Decimal::ZERO,
                    credit: dec!(80.00),
                    project_id: None,
                },
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
