//! Le dernier relevé et le solde comptable à sa date — Story 25-6-a (#389).
//!
//! Le solde comptable n'est pas le solde bancaire : la liste des comptes
//! bancaires rend le solde de clôture du dernier relevé et le solde comptable à
//! sa date, **corrigé des dates de valeur** (le relevé compte par date de
//! comptabilisation bancaire, les écritures de rapprochement par date de
//! valeur). Chaque test nomme la mutation qu'il tue.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::NewBankAccount;
use kesh_db::repositories::bank_accounts::{self, BankAccountBalances};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

async fn bank_account(pool: &MySqlPool, seeded: &SeededCompany, iban: &str) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .expect("compte bancaire")
    .id
}

async fn link(pool: &MySqlPool, bank_account_id: i64, account_id: i64) {
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(account_id)
        .bind(bank_account_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Écriture équilibrée à deux lignes, en SQL brut (numéro unique par test).
async fn entry(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    date: NaiveDate,
    debit: i64,
    credit: i64,
    amount: Decimal,
) -> i64 {
    let n: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let id = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, ?, 'Banque', 'test')",
    )
    .bind(seeded.company_id)
    .bind(seeded.fiscal_year_id)
    .bind(n)
    .bind(date)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (order, account, d, c) in [
        (1, debit, amount, Decimal::ZERO),
        (2, credit, Decimal::ZERO, amount),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(account)
        .bind(order)
        .bind(d)
        .bind(c)
        .execute(pool)
        .await
        .unwrap();
    }
    id
}

/// Un import ; `closing` `None` = un import sans solde (CSV). Renvoie son id.
async fn import(
    pool: &MySqlPool,
    seeded: &SeededCompany,
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
         VALUES (?, ?, 'r.xml', ?, 'CAMT053', '2026-01-01', ?, ?, 0, ?)",
    )
    .bind(seeded.company_id)
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

/// Une transaction bancaire ; `matched` = l'écriture qui la rapproche.
async fn tx(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    bank_account_id: i64,
    import_id: i64,
    booking: NaiveDate,
    amount: Decimal,
    matched: Option<i64>,
) {
    sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, status, matched_entry_id) \
         VALUES (?, ?, ?, ?, ?, 'CHF', 'tx', ?, ?)",
    )
    .bind(seeded.company_id)
    .bind(import_id)
    .bind(bank_account_id)
    .bind(booking)
    .bind(amount)
    .bind(if matched.is_some() { "reconciled" } else { "pending" })
    .bind(matched)
    .execute(pool)
    .await
    .unwrap();
}

async fn balances(pool: &MySqlPool, seeded: &SeededCompany, id: i64) -> BankAccountBalances {
    bank_accounts::list_by_company_with_balances(pool, seeded.company_id, true)
        .await
        .unwrap()
        .into_iter()
        .find(|(b, _)| b.id == id)
        .expect("compte présent")
        .1
}

const IBAN_A: &str = "CH9300762011623852957";
const IBAN_B: &str = "CH5604835012345678009";

/// Un compte lié à 1100 (Banque CI), un compte de produits pour la contrepartie.
async fn linked(pool: &MySqlPool) -> (SeededCompany, i64, i64, i64) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    let ba = bank_account(pool, &seeded, IBAN_A).await;
    let bank = seeded.accounts["1100"];
    link(pool, ba, bank).await;
    let sales = seeded.accounts["3000"];
    (seeded, ba, bank, sales)
}

/// ⛔ Le dernier relevé est le plus récent **en date**, pas en ordre d'import ;
/// à date égale, le dernier importé. Mutation tuée : un tri par `imported_at`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_dernier_releve_est_le_plus_recent_en_date(pool: MySqlPool) {
    let (seeded, ba, _, _) = linked(&pool).await;
    import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(300))).await;
    import(&pool, &seeded, ba, ymd(2026, 2, 28), Some(dec!(200))).await; // importé APRÈS, plus ancien
    let b = balances(&pool, &seeded, ba).await;
    assert_eq!(b.statement_date, Some(ymd(2026, 3, 31)));
    assert_eq!(b.statement_closing_balance, Some(dec!(300)));

    import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(310))).await; // même date, importé après
    assert_eq!(
        balances(&pool, &seeded, ba).await.statement_closing_balance,
        Some(dec!(310))
    );
}

/// Un import sans solde (CSV) plus récent ne masque pas le relevé CAMT.
/// Mutation tuée : le filtre `closing_balance IS NOT NULL` retiré.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_import_sans_solde_ne_masque_pas_le_releve(pool: MySqlPool) {
    let (seeded, ba, _, _) = linked(&pool).await;
    import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(300))).await;
    import(&pool, &seeded, ba, ymd(2026, 4, 30), None).await;
    assert_eq!(
        balances(&pool, &seeded, ba).await.statement_date,
        Some(ymd(2026, 3, 31))
    );
}

/// ⛔ Le solde comptable À LA DATE du relevé : l'écriture du jour même y entre,
/// celle du lendemain non ; un relevé daté après la dernière écriture rend le
/// solde courant. Mutation tuée : `<=` → `<`, ou la borne retirée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_solde_comptable_a_la_date_du_releve(pool: MySqlPool) {
    let (seeded, ba, bank, sales) = linked(&pool).await;
    entry(&pool, &seeded, ymd(2026, 3, 31), bank, sales, dec!(100)).await;
    entry(&pool, &seeded, ymd(2026, 4, 1), bank, sales, dec!(50)).await;
    import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(100))).await;
    let b = balances(&pool, &seeded, ba).await;
    assert_eq!(b.ledger_balance_at_statement, Some(dec!(100)));
    assert_eq!(b.current_balance, Some(dec!(150)));

    import(&pool, &seeded, ba, ymd(2026, 5, 31), Some(dec!(150))).await;
    assert_eq!(
        balances(&pool, &seeded, ba)
            .await
            .ledger_balance_at_statement,
        Some(dec!(150))
    );
}

/// ⛔ La date de valeur : un crédit comptabilisé par la banque le jour du relevé
/// et rapproché par une écriture du lendemain — et le cas inverse — ne
/// produisent AUCUN écart. Mutation tuée : la correction retirée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_date_de_valeur_ne_cree_pas_d_ecart(pool: MySqlPool) {
    let (seeded, ba, bank, sales) = linked(&pool).await;
    let base = entry(&pool, &seeded, ymd(2026, 3, 10), bank, sales, dec!(800)).await;
    let late = entry(&pool, &seeded, ymd(2026, 4, 1), bank, sales, dec!(200)).await;
    let early = entry(&pool, &seeded, ymd(2026, 3, 31), bank, sales, dec!(70)).await;
    let imp = import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(1000))).await;
    tx(
        &pool,
        &seeded,
        ba,
        imp,
        ymd(2026, 3, 10),
        dec!(800),
        Some(base),
    )
    .await;
    // Comptabilisé par la banque le 31, écriture le 1er : la banque l'a, le
    // grand livre au 31 non.
    tx(
        &pool,
        &seeded,
        ba,
        imp,
        ymd(2026, 3, 31),
        dec!(200),
        Some(late),
    )
    .await;
    // Comptabilisé le 1er, écriture le 31 : le grand livre l'a, la banque non.
    tx(
        &pool,
        &seeded,
        ba,
        imp,
        ymd(2026, 4, 1),
        dec!(70),
        Some(early),
    )
    .await;

    let b = balances(&pool, &seeded, ba).await;
    // Grand livre au 31 = 800 + 70 ; + 200 − 70 = 1000 = le relevé.
    assert_eq!(b.ledger_balance_at_statement, Some(dec!(1000)));
    assert_eq!(b.statement_closing_balance, Some(dec!(1000)));
}

/// Une transaction NON rapprochée du jour du relevé n'est pas corrigée : son
/// absence du grand livre est l'écart. Mutation tuée : `status` ignoré.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_transaction_non_rapprochee_reste_un_ecart(pool: MySqlPool) {
    let (seeded, ba, bank, sales) = linked(&pool).await;
    let e = entry(&pool, &seeded, ymd(2026, 4, 2), bank, sales, dec!(200)).await;
    let imp = import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(200))).await;
    // Rapprochement annulé : `pending`, `matched_entry_id` remis à NULL.
    tx(&pool, &seeded, ba, imp, ymd(2026, 3, 31), dec!(200), None).await;
    let _ = e;
    assert_eq!(
        balances(&pool, &seeded, ba)
            .await
            .ledger_balance_at_statement,
        Some(Decimal::ZERO)
    );
}

/// ⛔ Le lien changé après des rapprochements : les écritures portent sur
/// l'ANCIEN compte et sortent de la correction. Mutation tuée : sommer
/// `bank_transactions.amount` au lieu de la ligne de l'écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_lien_change_sort_de_la_correction(pool: MySqlPool) {
    let (seeded, ba, bank, sales) = linked(&pool).await;
    let late = entry(&pool, &seeded, ymd(2026, 4, 1), bank, sales, dec!(200)).await;
    let imp = import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(200))).await;
    tx(
        &pool,
        &seeded,
        ba,
        imp,
        ymd(2026, 3, 31),
        dec!(200),
        Some(late),
    )
    .await;
    // Relié à la caisse, qui ne porte aucune de ces écritures.
    link(&pool, ba, seeded.accounts["1000"]).await;
    assert_eq!(
        balances(&pool, &seeded, ba)
            .await
            .ledger_balance_at_statement,
        Some(Decimal::ZERO)
    );
}

/// Un compte de PASSIF lié (ligne de crédit tirée de 1000) : solde comptable
/// et relevé de même signe — `débit − crédit` est la convention du relevé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_de_passif_lie_a_le_signe_du_releve(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, &seeded, IBAN_A).await;
    let credit_line = seeded.accounts["2000"]; // Capital CI, de type Liability
    link(&pool, ba, credit_line).await;
    entry(
        &pool,
        &seeded,
        ymd(2026, 3, 5),
        seeded.accounts["4000"],
        credit_line,
        dec!(1000),
    )
    .await;
    import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(-1000))).await;
    let b = balances(&pool, &seeded, ba).await;
    assert_eq!(b.ledger_balance_at_statement, Some(dec!(-1000)));
    assert_eq!(b.statement_closing_balance, Some(dec!(-1000)));
}

/// ⛔ Deux comptes bancaires sur le MÊME compte de grand livre — l'un archivé :
/// le solde ne s'attribue à aucun, pas d'écart. Mutation tuée : le partage
/// compté sur les seuls comptes actifs, ou pas compté du tout.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_de_grand_livre_partage_n_a_pas_d_ecart(pool: MySqlPool) {
    let (seeded, ba, bank, sales) = linked(&pool).await;
    let other = bank_account(&pool, &seeded, IBAN_B).await;
    link(&pool, other, bank).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    entry(&pool, &seeded, ymd(2026, 3, 5), bank, sales, dec!(100)).await;
    import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(100))).await;
    let actives = bank_accounts::list_by_company_with_balances(&pool, seeded.company_id, false)
        .await
        .unwrap();
    let mine = &actives.iter().find(|(b, _)| b.id == ba).expect("présent").1;
    assert_eq!(mine.ledger_balance_at_statement, None);
    assert_eq!(
        mine.current_balance,
        Some(dec!(100)),
        "le solde comptable reste"
    );
}

/// Sans compte lié : ni solde comptable ni écart, le relevé reste.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn sans_compte_lie_le_releve_reste(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, &seeded, IBAN_A).await;
    import(&pool, &seeded, ba, ymd(2026, 3, 31), Some(dec!(42))).await;
    let b = balances(&pool, &seeded, ba).await;
    assert_eq!(b.current_balance, None);
    assert_eq!(b.ledger_balance_at_statement, None);
    assert_eq!(b.statement_closing_balance, Some(dec!(42)));
}

/// Isolation : le relevé et les écritures d'une autre société n'entrent pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_autre_societe_n_entre_pas(pool: MySqlPool) {
    let (seeded, ba, bank, sales) = linked(&pool).await;
    entry(&pool, &seeded, ymd(2026, 3, 5), bank, sales, dec!(100)).await;
    // Une seconde société, son compte bancaire et son relevé — en SQL brut, la
    // fixture ne se pose qu'une fois par base.
    let other_company = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 2', 'Pme', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let oba = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: other_company,
            bank_name: "Autre".into(),
            iban: IBAN_B.into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap()
    .id;
    sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, \
          closing_balance, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'o.xml', REPEAT('f', 64), 'CAMT053', '2026-01-01', '2026-03-31', 999, 0, ?)",
    )
    .bind(other_company)
    .bind(oba)
    .bind(seeded.admin_user_id)
    .execute(&pool)
    .await
    .unwrap();
    let b = balances(&pool, &seeded, ba).await;
    assert_eq!(b.statement_closing_balance, None);
    assert_eq!(b.current_balance, Some(dec!(100)));
}
