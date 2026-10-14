//! Préparer la clôture d'un exercice — les contrôles préalables, en lecture
//! seule (Story 29-1a, #232).
//!
//! L'exercice examiné `Y` est l'exercice de la société de test, redaté du
//! 01.01.2026 au 31.12.2026. Les états hérités (écritures déséquilibrées, à une
//! ligne, hors exercice ; comptes archivés portant une activité) se posent par
//! **SQL direct** : aucun chemin applicatif ne les produit, c'est le sujet.
//! Chaque test nomme la mutation de la fiche qu'il tue.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::repositories::bank_accounts;
use kesh_db::repositories::closing_checks::{
    self, CheckCode, CheckDetails, CheckStatus, ClosingCheck, ClosingChecks, EntryReason,
    NotCheckableReason, Severity, StatementReason,
};
use kesh_db::snapshot::ReadSnapshot;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company, seed_stub_company_only};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Société de test, son exercice redaté sur 2026 (`Y`).
async fn company(pool: &MySqlPool) -> SeededCompany {
    let seeded = seed_accounting_company(pool).await.unwrap();
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2026', start_date = '2026-01-01', \
         end_date = '2026-12-31' WHERE id = ?",
    )
    .bind(seeded.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    seeded
}

async fn fiscal_year(
    pool: &MySqlPool,
    company_id: i64,
    name: &str,
    start: NaiveDate,
    end: NaiveDate,
    status: &str,
) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(name)
    .bind(start)
    .bind(end)
    .bind(status)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn ledger_account(pool: &MySqlPool, company_id: i64, number: &str) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, 'Asset')",
    )
    .bind(company_id)
    .bind(number)
    .bind(format!("Banque {number}"))
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Compte bancaire posé en SQL direct ; `linked` = compte de grand livre lié.
async fn bank_account(pool: &MySqlPool, company_id: i64, name: &str, linked: Option<i64>) -> i64 {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_accounts")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) VALUES (?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(name)
    .bind(format!("CH00TEST{n:013}"))
    .bind(linked)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Un import ; `closing` `None` = un import sans solde (CSV).
async fn import(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    company_id: i64,
    bank_account_id: i64,
    period_to: NaiveDate,
    closing: Option<Decimal>,
) -> i64 {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_imports")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, \
          closing_balance, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'r.xml', ?, 'CAMT053', '2020-01-01', ?, ?, 0, ?)",
    )
    .bind(company_id)
    .bind(bank_account_id)
    .bind(format!("{n:064}"))
    .bind(period_to)
    .bind(closing)
    .bind(seeded.admin_user_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Une transaction bancaire au statut donné.
#[allow(clippy::too_many_arguments)]
async fn bank_tx(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    import_id: i64,
    booking: NaiveDate,
    amount: Decimal,
    status: &str,
    rejected: bool,
) {
    sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, status, \
          auto_match_rejected_at) \
         VALUES (?, ?, ?, ?, ?, 'CHF', 'tx', ?, IF(?, NOW(3), NULL))",
    )
    .bind(company_id)
    .bind(import_id)
    .bind(bank_account_id)
    .bind(booking)
    .bind(amount)
    .bind(status)
    .bind(rejected)
    .execute(pool)
    .await
    .unwrap();
}

/// Une écriture en SQL direct, avec les lignes données `(compte, débit, crédit)`.
async fn entry(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    date: NaiveDate,
    lines: &[(i64, Decimal, Decimal)],
) -> i64 {
    let n: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ?",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let id = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, ?, 'OD', 'test')",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .bind(n)
    .bind(date)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (order, (account, d, c)) in lines.iter().enumerate() {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(account)
        .bind(order as i64 + 1)
        .bind(d)
        .bind(c)
        .execute(pool)
        .await
        .unwrap();
    }
    id
}

/// Écriture équilibrée à deux lignes.
async fn balanced(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    date: NaiveDate,
    debit: i64,
    credit: i64,
    amount: Decimal,
) -> i64 {
    entry(
        pool,
        company_id,
        fiscal_year_id,
        date,
        &[
            (debit, amount, Decimal::ZERO),
            (credit, Decimal::ZERO, amount),
        ],
    )
    .await
}

/// Les contrôles, lus dans un instantané comme le fait la route.
async fn checks(pool: &MySqlPool, company_id: i64, fiscal_year_id: i64) -> ClosingChecks {
    let mut snap = ReadSnapshot::begin(pool).await.unwrap();
    let c = closing_checks::for_fiscal_year(snap.conn(), company_id, fiscal_year_id)
        .await
        .unwrap();
    snap.finish().await.unwrap();
    c
}

fn find(c: &ClosingChecks, code: CheckCode) -> &ClosingCheck {
    c.checks
        .iter()
        .find(|k| k.code == code)
        .expect("contrôle présent")
}

fn statements(k: &ClosingCheck) -> &closing_checks::StatementDetails {
    match &k.details {
        CheckDetails::BankStatementAtYearEnd(d) => d,
        other => panic!("détails inattendus : {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// T0 — mesure (AC2) : MariaDB devant un `FOR UPDATE` dans l'instantané
// ---------------------------------------------------------------------------

/// T0 (AC2, P2 F L3) — **mesure**, non garantie : MariaDB 10.11 refuse-t-il
/// un `SELECT … FOR UPDATE` dans une transaction `READ ONLY` ? Issue
/// consignée au Dev Agent Record : **refusé**, erreur 1792 (`Cannot execute
/// statement in a READ ONLY transaction`). La garde statique T-a2 reste,
/// quelle que soit l'issue : elle ne dépend pas de ce comportement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t0_un_verrou_dans_l_instantane_est_refuse_par_le_serveur(pool: MySqlPool) {
    let mut snap = ReadSnapshot::begin(&pool).await.unwrap();
    let res = sqlx::query("SELECT 1 FROM fiscal_years LIMIT 1 FOR UPDATE")
        .execute(snap.conn())
        .await;
    let err = res.expect_err("MariaDB 10.11 refuse un verrou en READ ONLY (mesuré en T0)");
    let code = err
        .as_database_error()
        .and_then(|e| e.code())
        .map(|c| c.into_owned());
    assert_eq!(
        code.as_deref(),
        Some("25006"),
        "SQLSTATE de l'erreur 1792 : {err}"
    );
}

// ---------------------------------------------------------------------------
// T-a1 — l'ensemble clos, dans l'ordre
// ---------------------------------------------------------------------------

/// T-a1 (AC1) — sept codes, dans l'ordre de la fiche, sévérités comprises ;
/// sur une société vierge, tout est `ok` sauf K7 (aucun exercice suivant).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a1_les_sept_controles_dans_l_ordre(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = checks(&pool, seeded.company_id, seeded.fiscal_year_id).await;
    let rendus: Vec<(CheckCode, Severity)> =
        c.checks.iter().map(|k| (k.code, k.severity)).collect();
    assert_eq!(
        rendus,
        vec![
            (CheckCode::EarlierFiscalYearOpen, Severity::Blocking),
            (CheckCode::PendingBankTransactions, Severity::Warning),
            (CheckCode::BankStatementAtYearEnd, Severity::Warning),
            (CheckCode::DraftInvoices, Severity::Info),
            (
                CheckCode::ImportedSupplierInvoicesToComplete,
                Severity::Info
            ),
            (CheckCode::InconsistentEntries, Severity::Anomaly),
            (CheckCode::NextFiscalYear, Severity::Info),
        ]
    );
    for k in &c.checks {
        let attendu = if k.code == CheckCode::NextFiscalYear {
            CheckStatus::Attention
        } else {
            CheckStatus::Ok
        };
        assert_eq!(k.status, attendu, "{:?}", k.code);
    }
    assert_eq!(c.fiscal_year_id, seeded.fiscal_year_id);
    assert_eq!(c.fiscal_year_name, "Exercice 2026");
    assert_eq!(
        (c.start_date, c.end_date),
        (ymd(2026, 1, 1), ymd(2026, 12, 31))
    );

    // La forme JSON : camelCase, codes stables, montants en chaînes.
    let json = serde_json::to_value(&c).unwrap();
    assert_eq!(json["checks"][0]["code"], "EARLIER_FISCAL_YEAR_OPEN");
    assert_eq!(json["checks"][0]["severity"], "blocking");
    assert_eq!(json["checks"][6]["status"], "attention");
    assert!(json.get("fiscalYearId").is_some());
}

/// AC1 — un exercice d'une autre société, ou absent : `NotFound`, décidé dans
/// l'instantané.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_exercice_d_une_autre_societe_est_introuvable(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let other = seed_stub_company_only(&pool).await.unwrap();
    let mut snap = ReadSnapshot::begin(&pool).await.unwrap();
    let r = closing_checks::for_fiscal_year(snap.conn(), other, seeded.fiscal_year_id).await;
    assert!(
        matches!(r, Err(kesh_db::errors::DbError::NotFound)),
        "{r:?}"
    );
    let r = closing_checks::for_fiscal_year(snap.conn(), seeded.company_id, i64::MAX).await;
    assert!(
        matches!(r, Err(kesh_db::errors::DbError::NotFound)),
        "{r:?}"
    );
    snap.finish().await.unwrap();
}

// ---------------------------------------------------------------------------
// T-a3 — K1
// ---------------------------------------------------------------------------

/// T-a3 — deux antérieurs ouverts : le **plus ancien** (M-a18) ; tous clos : ok.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a3_k1_le_plus_ancien_anterieur_ouvert(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let y24 = fiscal_year(
        &pool,
        c,
        "Exercice 2024",
        ymd(2024, 1, 1),
        ymd(2024, 12, 31),
        "Open",
    )
    .await;
    let y25 = fiscal_year(
        &pool,
        c,
        "Exercice 2025",
        ymd(2025, 1, 1),
        ymd(2025, 12, 31),
        "Open",
    )
    .await;
    let k = checks(&pool, c, seeded.fiscal_year_id).await;
    let k1 = find(&k, CheckCode::EarlierFiscalYearOpen);
    assert_eq!(k1.status, CheckStatus::Attention);
    assert_eq!(k1.count, 1);
    match &k1.details {
        CheckDetails::EarlierFiscalYearOpen(d) => {
            let fy = d.fiscal_year.as_ref().expect("antérieur ouvert");
            assert_eq!((fy.id, fy.name.as_str()), (y24, "Exercice 2024"));
        }
        other => panic!("{other:?}"),
    }

    for id in [y24, y25] {
        sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    let k = checks(&pool, c, seeded.fiscal_year_id).await;
    let k1 = find(&k, CheckCode::EarlierFiscalYearOpen);
    assert_eq!((k1.status, k1.count), (CheckStatus::Ok, 0));
}

// ---------------------------------------------------------------------------
// T-a4 — K2
// ---------------------------------------------------------------------------

/// T-a4 — transactions `pending` : premier et dernier jour comptés (M-a1,
/// M-a12), veille et lendemain non ; rejetée comptée dans `rejectedCount` ;
/// rapprochée non comptée ; autre société absente (M-a2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a4_k2_transactions_en_attente(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let ba = bank_account(&pool, c, "Banque A", None).await;
    let imp = import(&pool, &seeded, c, ba, ymd(2027, 1, 31), None).await;
    for (d, status, rejected) in [
        (ymd(2026, 1, 1), "pending", false),
        (ymd(2026, 12, 31), "pending", true),
        (ymd(2025, 12, 31), "pending", false),
        (ymd(2027, 1, 1), "pending", false),
        (ymd(2026, 6, 1), "reconciled", false),
    ] {
        bank_tx(&pool, c, ba, imp, d, dec!(10), status, rejected).await;
    }
    // Une autre société, même statut, même période.
    let other = seed_stub_company_only(&pool).await.unwrap();
    let oba = bank_account(&pool, other, "Banque autre", None).await;
    let oimp = import(&pool, &seeded, other, oba, ymd(2026, 12, 31), None).await;
    bank_tx(
        &pool,
        other,
        oba,
        oimp,
        ymd(2026, 6, 1),
        dec!(10),
        "pending",
        false,
    )
    .await;

    let k = checks(&pool, c, seeded.fiscal_year_id).await;
    let k2 = find(&k, CheckCode::PendingBankTransactions);
    assert_eq!((k2.status, k2.count), (CheckStatus::Attention, 2));
    match &k2.details {
        CheckDetails::PendingBankTransactions(d) => {
            assert_eq!(d.accounts.len(), 1, "{:?}", d.accounts);
            let a = &d.accounts[0];
            assert_eq!((a.bank_account_id, a.name.as_str()), (ba, "Banque A"));
            assert_eq!((a.count, a.rejected_count), (2, 1));
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-a5 — K3
// ---------------------------------------------------------------------------

/// Compte lié non partagé, relevé à solde au `period_to` donné.
async fn linked_with_statement(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    number: &str,
    period_to: NaiveDate,
    closing: Decimal,
) -> (i64, i64, i64) {
    let ledger = ledger_account(pool, seeded.company_id, number).await;
    let ba = bank_account(
        pool,
        seeded.company_id,
        &format!("Banque {number}"),
        Some(ledger),
    )
    .await;
    let imp = import(
        pool,
        seeded,
        seeded.company_id,
        ba,
        period_to,
        Some(closing),
    )
    .await;
    (ba, ledger, imp)
}

fn reasons_of(d: &closing_checks::StatementDetails, ba: i64) -> Option<StatementReason> {
    d.accounts
        .iter()
        .find(|a| a.bank_account_id == ba)
        .map(|a| a.reason)
}

fn not_checkable_of(d: &closing_checks::StatementDetails, ba: i64) -> Option<NotCheckableReason> {
    d.not_checkable
        .iter()
        .find(|a| a.bank_account_id == ba)
        .map(|a| a.reason)
}

/// T-a5 — `ok` : relevé au dernier jour, transaction rapprochée (écriture aux
/// livres) ; `GAP` : transaction non rapprochée ; relevé **postérieur** ignoré
/// (M-a3) ; `MISSING` ; `ENDS_BEFORE_YEAR_END`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a5_k3_motifs_et_borne(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;
    let sales = seeded.accounts["3000"];

    // OK : relevé au 31.12 à 100, écriture de 100 aux livres.
    let (ok, ok_l, _) =
        linked_with_statement(&pool, &seeded, "1021", ymd(2026, 12, 31), dec!(100)).await;
    balanced(&pool, c, fy, ymd(2026, 12, 15), ok_l, sales, dec!(100)).await;

    // GAP : relevé au 31.12 à 100, rien aux livres (la transaction n'est pas rapprochée).
    let (gap, _, gap_imp) =
        linked_with_statement(&pool, &seeded, "1022", ymd(2026, 12, 31), dec!(100)).await;
    bank_tx(
        &pool,
        c,
        gap,
        gap_imp,
        ymd(2026, 12, 15),
        dec!(100),
        "pending",
        false,
    )
    .await;

    // Borne : relevé au 31.12 à 100 aux livres égaux, PLUS un relevé postérieur
    // (31.01.2027) à 999 — ignoré : sans la borne, il serait retenu et ferait un écart.
    let (borne, borne_l, _) =
        linked_with_statement(&pool, &seeded, "1023", ymd(2026, 12, 31), dec!(100)).await;
    balanced(&pool, c, fy, ymd(2026, 12, 15), borne_l, sales, dec!(100)).await;
    import(&pool, &seeded, c, borne, ymd(2027, 1, 31), Some(dec!(999))).await;

    // MISSING : seul relevé à solde de l'exercice précédent ; une transaction dans Y.
    let (missing, _, missing_imp) =
        linked_with_statement(&pool, &seeded, "1024", ymd(2025, 12, 31), dec!(0)).await;
    bank_tx(
        &pool,
        c,
        missing,
        missing_imp,
        ymd(2026, 3, 1),
        dec!(5),
        "pending",
        false,
    )
    .await;

    // ENDS_BEFORE_YEAR_END : relevé arrêté au 30.11.
    let (early, _, _) =
        linked_with_statement(&pool, &seeded, "1025", ymd(2026, 11, 30), dec!(0)).await;

    let k = checks(&pool, c, fy).await;
    let k3 = find(&k, CheckCode::BankStatementAtYearEnd);
    let d = statements(k3);
    assert_eq!(reasons_of(d, ok), None, "relevé conforme : {d:?}");
    assert_eq!(
        reasons_of(d, borne),
        None,
        "le relevé postérieur est ignoré (M-a3) : {d:?}"
    );
    assert_eq!(reasons_of(d, gap), Some(StatementReason::Gap));
    assert_eq!(reasons_of(d, missing), Some(StatementReason::Missing));
    assert_eq!(
        reasons_of(d, early),
        Some(StatementReason::EndsBeforeYearEnd)
    );
    assert_eq!(k3.status, CheckStatus::Attention);
    assert_eq!(k3.count, d.accounts.len() as i64);

    let g = d
        .accounts
        .iter()
        .find(|a| a.bank_account_id == gap)
        .unwrap();
    assert_eq!(g.statement_date, Some(ymd(2026, 12, 31)));
    assert_eq!(g.statement_closing_balance, Some(dec!(100)));
    assert_eq!(g.ledger_balance_at_statement, Some(dec!(0)));
    assert_eq!(g.gap, Some(dec!(-100)));
    assert!(d.not_checkable.is_empty(), "{:?}", d.not_checkable);

    // Les montants partent en chaînes.
    let json = serde_json::to_value(&k).unwrap();
    let gap_json = json["checks"][2]["details"]["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["bankAccountId"] == gap)
        .unwrap()
        .clone();
    let gap_str = gap_json["gap"].as_str().expect("montant en chaîne");
    assert_eq!(gap_str.parse::<Decimal>().unwrap(), dec!(-100));
    assert_eq!(gap_json["reason"], "GAP");
}

/// T-a5 — l'écart s'arrondit au centime (M-a16) : `0.004` aux livres en
/// `DECIMAL(19,4)` n'en est pas un, `0.005` en est un d'un centime.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a5_k3_ecart_au_centime(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;
    let sales = seeded.accounts["3000"];
    let (a, a_l, _) =
        linked_with_statement(&pool, &seeded, "1031", ymd(2026, 12, 31), dec!(100)).await;
    balanced(&pool, c, fy, ymd(2026, 6, 1), a_l, sales, dec!(100.004)).await;
    let (b, b_l, _) =
        linked_with_statement(&pool, &seeded, "1032", ymd(2026, 12, 31), dec!(100)).await;
    balanced(&pool, c, fy, ymd(2026, 6, 1), b_l, sales, dec!(100.005)).await;

    let k = checks(&pool, c, fy).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    assert_eq!(reasons_of(d, a), None, "0.004 n'est pas un écart : {d:?}");
    let gb = d
        .accounts
        .iter()
        .find(|x| x.bank_account_id == b)
        .expect("0.005 est un écart");
    assert_eq!(
        (gb.reason, gb.gap),
        (StatementReason::Gap, Some(dec!(0.01)))
    );
}

/// T-a5 — population : compte archivé (avec activité dans Y posée en SQL
/// direct, relevé arrêté avant la fin de Y — M-a4), compte clos avant Y,
/// compte sans activité (M-a17), activité seulement postérieure : absents des
/// deux listes ; cas de contrôle non archivé : listé `ENDS_BEFORE_YEAR_END`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a5_k3_population_des_comptes_examines(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;

    // Archivé, relevé à solde au 30.11.2026 et transaction dans Y.
    let (archived, _, a_imp) =
        linked_with_statement(&pool, &seeded, "1041", ymd(2026, 11, 30), dec!(0)).await;
    bank_tx(
        &pool,
        c,
        archived,
        a_imp,
        ymd(2026, 5, 1),
        dec!(1),
        "pending",
        false,
    )
    .await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(archived)
        .execute(&pool)
        .await
        .unwrap();
    // Cas de contrôle : la même fixture, non archivée.
    let (control, _, ctl_imp) =
        linked_with_statement(&pool, &seeded, "1042", ymd(2026, 11, 30), dec!(0)).await;
    bank_tx(
        &pool,
        c,
        control,
        ctl_imp,
        ymd(2026, 5, 1),
        dec!(1),
        "pending",
        false,
    )
    .await;

    // Clos avant Y, sans import à solde, une transaction datée dans Y.
    let closed = bank_account(&pool, c, "Banque close", None).await;
    let cl_imp = import(&pool, &seeded, c, closed, ymd(2026, 5, 31), None).await;
    bank_tx(
        &pool,
        c,
        closed,
        cl_imp,
        ymd(2026, 5, 1),
        dec!(1),
        "pending",
        false,
    )
    .await;
    sqlx::query("UPDATE bank_accounts SET closed_on = '2025-12-31' WHERE id = ?")
        .bind(closed)
        .execute(&pool)
        .await
        .unwrap();

    // Sans activité dans Y (ni transaction, ni relevé, ni écriture).
    let ledger_idle = ledger_account(&pool, c, "1043").await;
    let idle = bank_account(&pool, c, "Banque dormante", Some(ledger_idle)).await;
    // Activité seulement postérieure à Y.
    let ledger_late = ledger_account(&pool, c, "1044").await;
    let late = bank_account(&pool, c, "Banque tardive", Some(ledger_late)).await;
    let late_imp = import(&pool, &seeded, c, late, ymd(2027, 1, 31), Some(dec!(5))).await;
    bank_tx(
        &pool,
        c,
        late,
        late_imp,
        ymd(2027, 1, 10),
        dec!(5),
        "pending",
        false,
    )
    .await;

    // Assertions de montage : les lignes existent (sans elles, rien n'est prouvé).
    for (ba, attendu) in [(archived, 1i64), (closed, 1), (late, 1)] {
        let n: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE bank_account_id = ?")
                .bind(ba)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(n, attendu, "montage du compte {ba}");
    }
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_imports WHERE bank_account_id = ? AND closing_balance IS NOT NULL \
         AND period_to BETWEEN '2026-01-01' AND '2026-12-31'",
    )
    .bind(archived)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        n, 1,
        "montage : le compte archivé porte un relevé à solde dans Y"
    );

    let k = checks(&pool, c, fy).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    assert_eq!(
        reasons_of(d, control),
        Some(StatementReason::EndsBeforeYearEnd)
    );
    for (ba, quoi) in [
        (archived, "archivé (M-a4)"),
        (closed, "clos avant Y"),
        (idle, "sans activité (M-a17)"),
        (late, "activité postérieure"),
    ] {
        assert_eq!(reasons_of(d, ba), None, "{quoi} : {d:?}");
        assert_eq!(not_checkable_of(d, ba), None, "{quoi} : {d:?}");
    }
}

/// T-a5 — compte créé **après** Y portant des transactions de Y : examiné (pas
/// de critère sur `created_at`) ; compte clos **pendant** Y mesuré à sa date de
/// clôture (relevé arrêté le jour de la clôture : conforme).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a5_k3_creation_tardive_et_cloture_pendant_y(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;
    let sales = seeded.accounts["3000"];

    let (late_created, _, lc_imp) =
        linked_with_statement(&pool, &seeded, "1051", ymd(2026, 12, 31), dec!(50)).await;
    bank_tx(
        &pool,
        c,
        late_created,
        lc_imp,
        ymd(2026, 4, 1),
        dec!(50),
        "pending",
        false,
    )
    .await;
    sqlx::query("UPDATE bank_accounts SET created_at = '2027-03-01 10:00:00' WHERE id = ?")
        .bind(late_created)
        .execute(&pool)
        .await
        .unwrap();

    let (closed_during, cd_l, _) =
        linked_with_statement(&pool, &seeded, "1052", ymd(2026, 6, 30), dec!(20)).await;
    balanced(&pool, c, fy, ymd(2026, 2, 1), cd_l, sales, dec!(20)).await;
    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-06-30' WHERE id = ?")
        .bind(closed_during)
        .execute(&pool)
        .await
        .unwrap();

    let k = checks(&pool, c, fy).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    assert_eq!(
        reasons_of(d, late_created),
        Some(StatementReason::Gap),
        "examiné : {d:?}"
    );
    assert_eq!(
        reasons_of(d, closed_during),
        None,
        "mesuré à sa clôture : {d:?}"
    );
}

/// T-a5 — `notCheckable` et précédence (M-a15) : sans compte lié et sans relevé
/// à solde → `NOT_COMPARABLE` ; compte lié partagé → `NOT_COMPARABLE` ; compte
/// CSV seul → `NO_BALANCE_STATEMENT` (non `MISSING`) ; relevés à solde tous
/// **postérieurs** à Y, transaction dans Y → `MISSING` (P3 R8). Seuls, ils
/// laissent le statut `ok`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a5_k3_non_verifiables_et_precedence(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;

    // Sans compte lié, CSV seul.
    let unlinked = bank_account(&pool, c, "Banque sans lien", None).await;
    let u_imp = import(&pool, &seeded, c, unlinked, ymd(2026, 3, 31), None).await;
    bank_tx(
        &pool,
        c,
        unlinked,
        u_imp,
        ymd(2026, 3, 1),
        dec!(1),
        "reconciled",
        false,
    )
    .await;
    // Partagé : deux comptes bancaires sur le même compte de grand livre.
    let shared_l = ledger_account(&pool, c, "1061").await;
    let shared_a = bank_account(&pool, c, "Banque partagée A", Some(shared_l)).await;
    let _shared_b = bank_account(&pool, c, "Banque partagée B", Some(shared_l)).await;
    import(
        &pool,
        &seeded,
        c,
        shared_a,
        ymd(2026, 12, 31),
        Some(dec!(1)),
    )
    .await;
    // Lié, CSV seul.
    let csv_l = ledger_account(&pool, c, "1062").await;
    let csv = bank_account(&pool, c, "Banque CSV", Some(csv_l)).await;
    let csv_imp = import(&pool, &seeded, c, csv, ymd(2026, 12, 31), None).await;
    bank_tx(
        &pool,
        c,
        csv,
        csv_imp,
        ymd(2026, 3, 1),
        dec!(1),
        "reconciled",
        false,
    )
    .await;

    let k = checks(&pool, c, fy).await;
    let k3 = find(&k, CheckCode::BankStatementAtYearEnd);
    let d = statements(k3);
    assert_eq!(
        not_checkable_of(d, unlinked),
        Some(NotCheckableReason::NotComparable)
    );
    assert_eq!(
        not_checkable_of(d, shared_a),
        Some(NotCheckableReason::NotComparable)
    );
    assert_eq!(
        not_checkable_of(d, csv),
        Some(NotCheckableReason::NoBalanceStatement)
    );
    assert!(d.accounts.is_empty(), "{:?}", d.accounts);
    assert_eq!(
        (k3.status, k3.count),
        (CheckStatus::Ok, 0),
        "notCheckable sans effet"
    );

    // Relevés à solde tous postérieurs à Y : MISSING, non NO_BALANCE_STATEMENT.
    let post_l = ledger_account(&pool, c, "1063").await;
    let post = bank_account(&pool, c, "Banque relevés 2027", Some(post_l)).await;
    let p_imp = import(&pool, &seeded, c, post, ymd(2027, 2, 28), Some(dec!(1))).await;
    bank_tx(
        &pool,
        c,
        post,
        p_imp,
        ymd(2026, 12, 20),
        dec!(1),
        "pending",
        false,
    )
    .await;
    let k = checks(&pool, c, fy).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    assert_eq!(reasons_of(d, post), Some(StatementReason::Missing), "{d:?}");
    assert_eq!(not_checkable_of(d, post), None);
}

/// T-a5 — activité au grand livre (C-29-1-24, M-a20) : relevé de l'exercice
/// précédent, aucune transaction dans Y, une écriture manuelle dans Y sur le
/// compte lié → examiné, `MISSING` ; sans cette écriture → absent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a5_k3_activite_au_grand_livre(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;
    let (ba, ledger, _) =
        linked_with_statement(&pool, &seeded, "1071", ymd(2025, 12, 31), dec!(0)).await;

    let k = checks(&pool, c, fy).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    assert_eq!(reasons_of(d, ba), None, "sans écriture dans Y : absent");

    balanced(
        &pool,
        c,
        fy,
        ymd(2026, 7, 1),
        ledger,
        seeded.accounts["3000"],
        dec!(30),
    )
    .await;
    let k = checks(&pool, c, fy).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    assert_eq!(reasons_of(d, ba), Some(StatementReason::Missing), "{d:?}");
}

/// T-a5 — une autre société portant l'anomalie n'apparaît pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a5_k3_autre_societe_absente(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let other = seed_stub_company_only(&pool).await.unwrap();
    let ol = ledger_account(&pool, other, "1081").await;
    let oba = bank_account(&pool, other, "Banque autre", Some(ol)).await;
    let oimp = import(
        &pool,
        &seeded,
        other,
        oba,
        ymd(2026, 12, 31),
        Some(dec!(100)),
    )
    .await;
    bank_tx(
        &pool,
        other,
        oba,
        oimp,
        ymd(2026, 12, 1),
        dec!(100),
        "pending",
        false,
    )
    .await;
    let k = checks(&pool, seeded.company_id, seeded.fiscal_year_id).await;
    let k3 = find(&k, CheckCode::BankStatementAtYearEnd);
    let d = statements(k3);
    assert!(d.accounts.is_empty() && d.not_checkable.is_empty(), "{d:?}");
}

// ---------------------------------------------------------------------------
// T-a6 — l'extraction de la requête du tableau de bord
// ---------------------------------------------------------------------------

/// T-a6 — `None` et une borne postérieure à tous les relevés rendent les mêmes
/// valeurs (M-a5) ; une borne **antérieure** au dernier relevé désigne le
/// relevé précédent (filtre **avant** `ROW_NUMBER`) — lu par K3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a6_la_borne_designe_le_releve_precedent(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;
    let (ba, ledger, _) =
        linked_with_statement(&pool, &seeded, "1091", ymd(2026, 12, 31), dec!(70)).await;
    import(&pool, &seeded, c, ba, ymd(2027, 1, 31), Some(dec!(90))).await;
    balanced(
        &pool,
        c,
        fy,
        ymd(2026, 6, 1),
        ledger,
        seeded.accounts["3000"],
        dec!(70),
    )
    .await;

    // Sans borne (tableau de bord) : le relevé de janvier 2027.
    let list = bank_accounts::list_by_company_with_balances(
        &pool,
        c,
        bank_accounts::BankAccountListFilter::ALL,
    )
    .await
    .unwrap();
    let (_, b) = list.iter().find(|(a, _)| a.id == ba).unwrap();
    assert_eq!(b.statement_date, Some(ymd(2027, 1, 31)));
    assert_eq!(b.statement_closing_balance, Some(dec!(90)));

    // Borné au 31.12.2026 (K3) : le relevé de décembre, conforme aux livres.
    let k = checks(&pool, c, fy).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    assert_eq!(
        reasons_of(d, ba),
        None,
        "le relevé de décembre égale les livres : {d:?}"
    );

    // Une borne postérieure à tous les relevés : mêmes valeurs que sans borne
    // (lu par un exercice 2027).
    let y27 = fiscal_year(
        &pool,
        c,
        "Exercice 2027",
        ymd(2027, 1, 1),
        ymd(2027, 12, 31),
        "Open",
    )
    .await;
    let k = checks(&pool, c, y27).await;
    let d = statements(find(&k, CheckCode::BankStatementAtYearEnd));
    let a = d
        .accounts
        .iter()
        .find(|x| x.bank_account_id == ba)
        .expect("examiné en 2027");
    assert_eq!(a.statement_date, b.statement_date);
    assert_eq!(a.statement_closing_balance, b.statement_closing_balance);
    assert_eq!(a.ledger_balance_at_statement, b.ledger_balance_at_statement);
    assert_eq!(a.reason, StatementReason::EndsBeforeYearEnd);
}

// ---------------------------------------------------------------------------
// T-a7 — K4
// ---------------------------------------------------------------------------

/// Une facture ; une facture validée porte une écriture (`chk_invoices_validated_has_je`).
async fn invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact: i64,
    date: NaiveDate,
    status: &str,
    journal_entry_id: Option<i64>,
) -> i64 {
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, status, date, journal_entry_id) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(contact)
    .bind(status)
    .bind(date)
    .bind(journal_entry_id)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
}

async fn contact(pool: &MySqlPool, company_id: i64, name: &str) -> i64 {
    sqlx::query("INSERT INTO contacts (company_id, contact_type, name, is_client) VALUES (?, 'Entreprise', ?, TRUE)")
        .bind(company_id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
}

/// T-a7 — brouillons au premier et au dernier jour comptés (M-a13), validé et
/// annulé non (M-a6), veille et lendemain non ; libellé = nom du contact.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a7_k4_brouillons(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let ct = contact(&pool, c, "Dubarde SàRL").await;
    let je = balanced(
        &pool,
        c,
        seeded.fiscal_year_id,
        ymd(2026, 6, 1),
        seeded.accounts["1100"],
        seeded.accounts["3000"],
        dec!(10),
    )
    .await;
    let first = invoice(&pool, c, ct, ymd(2026, 1, 1), "draft", None).await;
    let last = invoice(&pool, c, ct, ymd(2026, 12, 31), "draft", None).await;
    invoice(&pool, c, ct, ymd(2026, 6, 1), "validated", Some(je)).await;
    for (d, s) in [
        (ymd(2026, 6, 2), "cancelled"),
        (ymd(2025, 12, 31), "draft"),
        (ymd(2027, 1, 1), "draft"),
    ] {
        invoice(&pool, c, ct, d, s, None).await;
    }
    let other = seed_stub_company_only(&pool).await.unwrap();
    let oct = contact(&pool, other, "Autre").await;
    invoice(&pool, other, oct, ymd(2026, 6, 1), "draft", None).await;

    let k = checks(&pool, c, seeded.fiscal_year_id).await;
    let k4 = find(&k, CheckCode::DraftInvoices);
    assert_eq!((k4.status, k4.count), (CheckStatus::Attention, 2));
    match &k4.details {
        CheckDetails::DraftInvoices(d) => {
            let ids: Vec<i64> = d.invoices.iter().map(|i| i.id).collect();
            assert_eq!(ids, vec![first, last]);
            assert_eq!(d.invoices[0].label, "Dubarde SàRL");
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-a8 — K5
// ---------------------------------------------------------------------------

async fn imported(pool: &MySqlPool, company_id: i64, status: &str, n: i64) {
    sqlx::query(
        "INSERT INTO imported_supplier_invoices \
         (company_id, status, file_hash, storage_path, original_filename, mime_type, byte_size, \
          creditor_iban, is_qr_iban, creditor_address_type, creditor_name, creditor_country, \
          reference_type, currency) \
         VALUES (?, ?, ?, 'p', 'f.pdf', 'application/pdf', 1, 'CH0000000000000000000', FALSE, 'S', \
                 'Fournisseur', 'CH', 'NON', 'CHF')",
    )
    .bind(company_id)
    .bind(status)
    .bind(format!("{n:064}"))
    .execute(pool)
    .await
    .unwrap();
}

/// T-a8 — `to_complete` compté, `completed` / `discarded` non ; autre société absente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a8_k5_pieces_a_completer(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    imported(&pool, c, "to_complete", 1).await;
    imported(&pool, c, "completed", 2).await;
    imported(&pool, c, "discarded", 3).await;
    let other = seed_stub_company_only(&pool).await.unwrap();
    imported(&pool, other, "to_complete", 4).await;
    let k = checks(&pool, c, seeded.fiscal_year_id).await;
    let k5 = find(&k, CheckCode::ImportedSupplierInvoicesToComplete);
    assert_eq!((k5.status, k5.count), (CheckStatus::Attention, 1));
}

// ---------------------------------------------------------------------------
// T-a9 — K6
// ---------------------------------------------------------------------------

/// T-a9 — les quatre motifs, posés en SQL direct (M-a7, M-a8, M-a14) ; une
/// écriture ordinaire n'y figure pas ; une écriture déséquilibrée **et** hors
/// exercice compte une fois dans `count`, deux fois dans `countByReason` ;
/// liste dédupliquée, dans l'ordre `entry_date, id`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a9_k6_ecritures_incoherentes(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;
    let a = seeded.accounts["1000"];
    let b = seeded.accounts["3000"];
    let y27 = fiscal_year(
        &pool,
        c,
        "Exercice 2027",
        ymd(2027, 1, 1),
        ymd(2027, 12, 31),
        "Open",
    )
    .await;

    // Ordinaire.
    balanced(&pool, c, fy, ymd(2026, 5, 1), a, b, dec!(10)).await;
    // Déséquilibrée.
    let unbalanced = entry(
        &pool,
        c,
        fy,
        ymd(2026, 3, 1),
        &[(a, dec!(10), dec!(0)), (b, dec!(0), dec!(9))],
    )
    .await;
    // Une seule ligne (et donc déséquilibrée aussi : une ligne porte un débit
    // OU un crédit, `chk_jel_debit_credit_exclusive`).
    let single = entry(&pool, c, fy, ymd(2026, 2, 1), &[(a, dec!(5), dec!(0))]).await;
    // De Y datée du lendemain de sa fin, ET déséquilibrée.
    let outside = entry(
        &pool,
        c,
        fy,
        ymd(2027, 1, 1),
        &[(a, dec!(3), dec!(0)), (b, dec!(0), dec!(1))],
    )
    .await;
    // D'un autre exercice, datée du DERNIER jour de Y.
    let other_inside = balanced(&pool, c, y27, ymd(2026, 12, 31), a, b, dec!(4)).await;
    // Une autre société, déséquilibrée dans les mêmes dates.
    let other = seed_stub_company_only(&pool).await.unwrap();
    let ofy = fiscal_year(
        &pool,
        other,
        "Autre 2026",
        ymd(2026, 1, 1),
        ymd(2026, 12, 31),
        "Open",
    )
    .await;
    entry(
        &pool,
        other,
        ofy,
        ymd(2026, 3, 1),
        &[(a, dec!(10), dec!(0))],
    )
    .await;

    let k = checks(&pool, c, fy).await;
    let k6 = find(&k, CheckCode::InconsistentEntries);
    assert_eq!((k6.status, k6.count), (CheckStatus::Attention, 4));
    match &k6.details {
        CheckDetails::InconsistentEntries(d) => {
            assert_eq!(d.count_by_reason.unbalanced, 3, "{:?}", d.count_by_reason);
            assert_eq!(d.count_by_reason.too_few_lines, 1);
            assert_eq!(d.count_by_reason.date_outside_its_year, 1);
            assert_eq!(d.count_by_reason.other_year_dated_inside, 1);
            let ids: Vec<i64> = d.entries.iter().map(|e| e.id).collect();
            assert_eq!(ids, vec![single, unbalanced, other_inside, outside]);
            let o = d.entries.iter().find(|e| e.id == outside).unwrap();
            assert_eq!(
                o.reasons,
                vec![EntryReason::Unbalanced, EntryReason::DateOutsideItsYear]
            );
            let s = d.entries.iter().find(|e| e.id == single).unwrap();
            assert_eq!(
                s.reasons,
                vec![EntryReason::Unbalanced, EntryReason::TooFewLines]
            );
            let oi = d.entries.iter().find(|e| e.id == other_inside).unwrap();
            assert_eq!(oi.fiscal_year_name, "Exercice 2027");
            assert_eq!(oi.reasons, vec![EntryReason::OtherYearDatedInside]);
        }
        other => panic!("{other:?}"),
    }
}

/// T-a9 — la liste est bornée à 20, le compte ne l'est pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a9_k6_liste_bornee(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let a = seeded.accounts["1000"];
    for i in 0..23 {
        entry(
            &pool,
            c,
            seeded.fiscal_year_id,
            ymd(2026, 1, 1 + i),
            &[(a, dec!(1), dec!(0))],
        )
        .await;
    }
    let k = checks(&pool, c, seeded.fiscal_year_id).await;
    let k6 = find(&k, CheckCode::InconsistentEntries);
    assert_eq!(k6.count, 23);
    match &k6.details {
        CheckDetails::InconsistentEntries(d) => assert_eq!(d.entries.len(), 20),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-a10 — K7
// ---------------------------------------------------------------------------

/// T-a10 — sans suivant : `attention` ; avec N+1 et N+2 : `ok` et les données
/// de **N+1** (M-a11) ; un antérieur seul n'est pas un suivant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t_a10_k7_exercice_suivant(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let c = seeded.company_id;
    let fy = seeded.fiscal_year_id;
    fiscal_year(
        &pool,
        c,
        "Exercice 2025",
        ymd(2025, 1, 1),
        ymd(2025, 12, 31),
        "Closed",
    )
    .await;
    let k = checks(&pool, c, fy).await;
    let k7 = find(&k, CheckCode::NextFiscalYear);
    assert_eq!((k7.status, k7.count), (CheckStatus::Attention, 1));

    let y28 = fiscal_year(
        &pool,
        c,
        "Exercice 2028",
        ymd(2028, 1, 1),
        ymd(2028, 12, 31),
        "Open",
    )
    .await;
    let y27 = fiscal_year(
        &pool,
        c,
        "Exercice 2027",
        ymd(2027, 1, 1),
        ymd(2027, 12, 31),
        "Open",
    )
    .await;
    let _ = y28;
    let k = checks(&pool, c, fy).await;
    let k7 = find(&k, CheckCode::NextFiscalYear);
    assert_eq!((k7.status, k7.count), (CheckStatus::Ok, 0));
    match &k7.details {
        CheckDetails::NextFiscalYear(d) => {
            let n = d.fiscal_year.as_ref().expect("suivant");
            assert_eq!(
                (n.id, n.name.as_str(), n.start_date),
                (y27, "Exercice 2027", ymd(2027, 1, 1))
            );
        }
        other => panic!("{other:?}"),
    }
}
