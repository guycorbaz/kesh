//! La balance des comptes porte un solde d'ouverture — Story 25-5-b (#385).
//!
//! ⚠️ **Les tests de CONCORDANCE sont la raison d'être de ce fichier.** Le défaut
//! redouté est muet : si l'ouverture est prise de travers, `clôture = ouverture +
//! mouvements` reste vrai et le rapport est intérieurement cohérent, extérieurement
//! faux. Ils appellent donc réellement le bilan, le compte de résultat et le grand
//! livre, et chacun nomme la mutation qu'il tue.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::general_ledger::{LedgerOptions, LedgerPeriod};
use kesh_report::period::ReportPeriod;
use kesh_report::trial_balance::TrialBalance;
use kesh_report::{generate_balance_sheet, generate_general_ledger, generate_trial_balance};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

/// Poste une écriture équilibrée à deux lignes dans l'exercice `fy`. Renvoie
/// l'id de l'écriture.
#[allow(clippy::too_many_arguments)]
async fn post(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    fy: i64,
    date: NaiveDate,
    debit_account: &str,
    credit_account: &str,
    amount: Decimal,
) -> i64 {
    journal_entries::create(
        pool,
        fy,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "écriture".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts[debit_account],
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts[credit_account],
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture postée")
    .entry
    .id
}

/// Deux exercices, 2025 et 2026. ⚠️ La fixture seede UN exercice 2020-2030 : il
/// faut le rétrécir, sinon 2025 et 2026 sont le même exercice et le report ne
/// prouve plus rien.
async fn two_years(pool: &MySqlPool, seeded: &SeededCompany) -> (i64, i64) {
    sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(pool)
        .await
        .expect("exercice seedé rétréci à 2025");
    let fy2026: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, 'FY 2026', '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .expect("exercice 2026 créé");
    (seeded.fiscal_year_id, fy2026)
}

/// Le jeu de référence : ventes 1000 et charge 400 en 2025 ; en 2026, une vente
/// de 200 en février, 50 la veille du 1er mars, 70 le 1er mars (la frontière),
/// 500 en avril, 30 en août.
async fn reference_books(pool: &MySqlPool, seeded: &SeededCompany) -> (i64, i64) {
    let (fy25, fy26) = two_years(pool, seeded).await;
    post(
        pool,
        seeded,
        fy25,
        ymd(2025, 3, 1),
        "1100",
        "3000",
        dec!(1000),
    )
    .await;
    post(
        pool,
        seeded,
        fy25,
        ymd(2025, 6, 1),
        "4000",
        "1000",
        dec!(400),
    )
    .await;
    post(
        pool,
        seeded,
        fy26,
        ymd(2026, 2, 1),
        "1100",
        "3000",
        dec!(200),
    )
    .await;
    let veille = ymd(2026, 3, 1).pred_opt().expect("veille");
    post(pool, seeded, fy26, veille, "1000", "3000", dec!(50)).await;
    post(
        pool,
        seeded,
        fy26,
        ymd(2026, 3, 1),
        "1000",
        "3000",
        dec!(70),
    )
    .await;
    post(
        pool,
        seeded,
        fy26,
        ymd(2026, 4, 15),
        "1100",
        "3000",
        dec!(500),
    )
    .await;
    post(
        pool,
        seeded,
        fy26,
        ymd(2026, 8, 1),
        "1100",
        "3000",
        dec!(30),
    )
    .await;
    (fy25, fy26)
}

fn period(fy: i64, start: NaiveDate, end: NaiveDate) -> ReportPeriod {
    ReportPeriod {
        fiscal_year_id: fy,
        start_date: start,
        end_date: end,
    }
}

async fn balance(pool: &MySqlPool, seeded: &SeededCompany, p: &ReportPeriod) -> TrialBalance {
    generate_trial_balance(pool, seeded.company_id, p)
        .await
        .expect("balance")
}

/// `(ouverture, débit, crédit, clôture)` d'un compte, par numéro.
fn row(tb: &TrialBalance, number: &str) -> (Decimal, Decimal, Decimal, Decimal) {
    let r = tb
        .rows
        .iter()
        .find(|r| r.account_number == number)
        .unwrap_or_else(|| panic!("le compte {number} doit figurer à la balance"));
    (
        r.opening_balance,
        r.total_debit,
        r.total_credit,
        r.closing_balance,
    )
}

/// ⛔ Toute la balance de l'exercice 2 concorde avec le bilan (comptes de bilan)
/// et le compte de résultat (comptes de résultat). Mutation tuée : « ouverture
/// bornée par l'exercice » (le défaut de #385 — la caisse et la banque perdraient
/// leur report) et « ouverture des comptes de résultat cumulée depuis l'origine »
/// (les ventes porteraient les 1000 de 2025).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_balance_de_l_exercice_concorde_avec_le_bilan_et_le_resultat(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (_, fy26) = reference_books(&pool, &seeded).await;
    let p = period(fy26, ymd(2026, 1, 1), ymd(2026, 12, 31));
    let tb = balance(&pool, &seeded, &p).await;

    assert_eq!(
        row(&tb, "3000").0,
        Decimal::ZERO,
        "les ventes repartent de zéro"
    );
    assert_eq!(row(&tb, "1100").0, dec!(1000), "la banque reporte 2025");
    assert_eq!(row(&tb, "1000").0, dec!(-400), "la caisse reporte 2025");
    assert_eq!(tb.retained_earnings, dec!(600));
    assert!(tb.opening_balanced);

    let bs = generate_balance_sheet(&pool, seeded.company_id, &p)
        .await
        .expect("bilan");
    assert_eq!(tb.retained_earnings, bs.retained_earnings);
    let bs_lines: Vec<_> = bs
        .assets
        .iter()
        .chain(&bs.liabilities)
        .chain(&bs.equity)
        .collect();
    let is = kesh_report::generate_income_statement(&pool, seeded.company_id, &p)
        .await
        .expect("compte de résultat");
    let is_lines: Vec<_> = is.revenues.iter().chain(&is.expenses).collect();

    for r in &tb.rows {
        let expected = if kesh_report::opening::is_bilan(r.account_type) {
            bs_lines
                .iter()
                .find(|b| b.account_id == r.account_id)
                .map_or(Decimal::ZERO, |b| b.balance)
        } else {
            is_lines
                .iter()
                .find(|b| b.account_id == r.account_id)
                .map_or(Decimal::ZERO, |b| b.balance)
        };
        assert_eq!(
            r.closing_balance, expected,
            "compte {} : la clôture doit égaler le bilan / le compte de résultat",
            r.account_number
        );
    }
}

/// ⛔ Période en cours d'exercice (mars–juin), avec une écriture LE 1er mars et une
/// la veille. Mutation tuée : `<` → `<=` sur la borne d'ouverture (la vente du 1er
/// mars serait comptée deux fois — en ouverture et en mouvement).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn en_cours_d_exercice_la_frontiere_n_est_comptee_qu_une_fois(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (_, fy26) = reference_books(&pool, &seeded).await;
    let (start, end) = (ymd(2026, 3, 1), ymd(2026, 6, 30));
    let tb = balance(&pool, &seeded, &period(fy26, start, end)).await;

    // Ventes : 200 + 50 (la veille) en ouverture ; 70 (le 1er mars) + 500 en mouvement.
    assert_eq!(
        row(&tb, "3000"),
        (dec!(250), Decimal::ZERO, dec!(570), dec!(820))
    );
    // Caisse : −400 + 50 en ouverture ; 70 en mouvement.
    assert_eq!(
        row(&tb, "1000"),
        (dec!(-350), dec!(70), Decimal::ZERO, dec!(-280))
    );
    assert!(tb.opening_balanced);

    // La même clôture que le grand livre, compte par compte.
    let ledger = generate_general_ledger(
        &pool,
        seeded.company_id,
        &LedgerPeriod::new(start, end).expect("période"),
        &LedgerOptions::default(),
    )
    .await
    .expect("grand livre");
    for s in &ledger.sections {
        let r = tb
            .rows
            .iter()
            .find(|r| r.account_id == s.account_id)
            .expect("tout compte du grand livre figure à la balance");
        assert_eq!(
            r.opening_balance, s.opening,
            "ouverture {}",
            s.account_number
        );
        assert_eq!(r.closing_balance, s.closing, "clôture {}", s.account_number);
    }

    // Et la clôture d'un compte de bilan, celle du bilan arrêté au 30 juin.
    let bs = generate_balance_sheet(&pool, seeded.company_id, &period(fy26, start, end))
        .await
        .expect("bilan");
    let caisse = bs
        .assets
        .iter()
        .find(|a| a.account_number == "1000")
        .expect("caisse au bilan");
    assert_eq!(row(&tb, "1000").3, caisse.balance);
}

/// Une période d'un seul jour : l'écriture du jour en mouvement, celle de la
/// veille en ouverture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_periode_d_un_jour(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (_, fy26) = reference_books(&pool, &seeded).await;
    let day = ymd(2026, 3, 1);
    let tb = balance(&pool, &seeded, &period(fy26, day, day)).await;
    assert_eq!(
        row(&tb, "1000"),
        (dec!(-350), dec!(70), Decimal::ZERO, dec!(-280))
    );
    assert!(tb.opening_balanced);
}

/// Le premier exercice d'une société : rien n'est reporté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_premier_exercice_n_a_rien_a_reporter(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let fy = seeded.fiscal_year_id;
    post(
        &pool,
        &seeded,
        fy,
        ymd(2026, 3, 1),
        "1100",
        "3000",
        dec!(1000),
    )
    .await;
    let tb = balance(
        &pool,
        &seeded,
        &period(fy, ymd(2020, 1, 1), ymd(2030, 12, 31)),
    )
    .await;
    assert_eq!(tb.retained_earnings, Decimal::ZERO);
    assert!(tb.rows.iter().all(|r| r.opening_balance.is_zero()));
    assert_eq!(row(&tb, "1100").3, dec!(1000));
    assert!(tb.opening_balanced);
}

/// Une société sans aucune écriture : tout à zéro, contrôle vrai.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_societe_sans_ecriture(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let fy = seeded.fiscal_year_id;
    let tb = balance(
        &pool,
        &seeded,
        &period(fy, ymd(2020, 1, 1), ymd(2030, 12, 31)),
    )
    .await;
    assert!(
        !tb.rows.is_empty(),
        "les comptes actifs sont toujours rendus"
    );
    assert!(
        tb.rows
            .iter()
            .all(|r| r.opening_balance.is_zero() && r.closing_balance.is_zero())
    );
    assert_eq!(tb.retained_earnings, Decimal::ZERO);
    assert!(tb.opening_balanced);
}

/// L'exercice 2 ouvert et encore vide — la première vue d'une année neuve :
/// ouvertures = clôtures de 2025 pour le bilan, nulles pour le résultat.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_exercice_neuf_encore_vide_montre_le_report(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (fy25, fy26) = two_years(&pool, &seeded).await;
    post(
        &pool,
        &seeded,
        fy25,
        ymd(2025, 3, 1),
        "1100",
        "3000",
        dec!(1000),
    )
    .await;
    post(
        &pool,
        &seeded,
        fy25,
        ymd(2025, 6, 1),
        "4000",
        "1000",
        dec!(400),
    )
    .await;
    let tb = balance(
        &pool,
        &seeded,
        &period(fy26, ymd(2026, 1, 1), ymd(2026, 12, 31)),
    )
    .await;
    assert_eq!(
        row(&tb, "1100"),
        (dec!(1000), Decimal::ZERO, Decimal::ZERO, dec!(1000))
    );
    assert_eq!(row(&tb, "3000").0, Decimal::ZERO);
    assert_eq!(row(&tb, "4000").0, Decimal::ZERO);
    assert_eq!(tb.retained_earnings, dec!(600));
    assert!(tb.opening_balanced);
}

/// Des pertes cumulées : résultat reporté négatif, contrôle vrai.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn des_pertes_cumulees(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (fy25, fy26) = two_years(&pool, &seeded).await;
    post(
        &pool,
        &seeded,
        fy25,
        ymd(2025, 3, 1),
        "1100",
        "3000",
        dec!(100),
    )
    .await;
    post(
        &pool,
        &seeded,
        fy25,
        ymd(2025, 6, 1),
        "4000",
        "1100",
        dec!(900),
    )
    .await;
    let tb = balance(
        &pool,
        &seeded,
        &period(fy26, ymd(2026, 1, 1), ymd(2026, 12, 31)),
    )
    .await;
    assert_eq!(tb.retained_earnings, dec!(-800));
    assert!(tb.opening_balanced);
}

/// Insère une ligne **déséquilibrée** dans une écriture existante, hors
/// `create_in_tx` (qui garantit la partie double).
async fn break_entry(pool: &MySqlPool, entry_id: i64, account_id: i64) {
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         VALUES (?, ?, 99, 10, 0)",
    )
    .bind(entry_id)
    .bind(account_id)
    .execute(pool)
    .await
    .expect("ligne déséquilibrée insérée");
}

/// ⛔ Le contrôle d'ouverture TOMBE quand l'égalité est réellement cassée — sur un
/// compte de bilan, puis sur un compte de résultat (l'écart passe alors par le
/// résultat reporté). Le rapport est rendu : les mouvements de 2026 restent
/// équilibrés. Mutation tuée : un contrôle tautologique (toujours vrai).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_controle_d_ouverture_tombe_sur_un_compte_de_bilan(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (fy25, fy26) = two_years(&pool, &seeded).await;
    let e = post(
        &pool,
        &seeded,
        fy25,
        ymd(2025, 3, 1),
        "1100",
        "3000",
        dec!(1000),
    )
    .await;
    break_entry(&pool, e, seeded.accounts["1000"]).await;
    let tb = balance(
        &pool,
        &seeded,
        &period(fy26, ymd(2026, 1, 1), ymd(2026, 12, 31)),
    )
    .await;
    assert!(!tb.opening_balanced);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_controle_d_ouverture_tombe_sur_un_compte_de_resultat(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (fy25, fy26) = two_years(&pool, &seeded).await;
    let e = post(
        &pool,
        &seeded,
        fy25,
        ymd(2025, 3, 1),
        "1100",
        "3000",
        dec!(1000),
    )
    .await;
    break_entry(&pool, e, seeded.accounts["4000"]).await;
    let tb = balance(
        &pool,
        &seeded,
        &period(fy26, ymd(2026, 1, 1), ymd(2026, 12, 31)),
    )
    .await;
    assert!(!tb.opening_balanced);
}

async fn archive(pool: &MySqlPool, account_id: i64) {
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("compte archivé");
}

/// Un compte archivé dont les lignes de la période **se compensent** (net nul),
/// sans ouverture : il figure — il a été mouvementé. Mutation tuée : une règle
/// d'inclusion sur le **montant** des mouvements plutôt que sur leur existence.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_archive_aux_mouvements_compenses_figure(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let fy = seeded.fiscal_year_id;
    post(
        &pool,
        &seeded,
        fy,
        ymd(2026, 3, 1),
        "1000",
        "1100",
        dec!(100),
    )
    .await;
    post(
        &pool,
        &seeded,
        fy,
        ymd(2026, 3, 2),
        "1100",
        "1000",
        dec!(100),
    )
    .await;
    archive(&pool, seeded.accounts["1100"]).await;
    let tb = balance(
        &pool,
        &seeded,
        &period(fy, ymd(2026, 1, 1), ymd(2026, 12, 31)),
    )
    .await;
    assert_eq!(
        row(&tb, "1100"),
        (Decimal::ZERO, dec!(100), dec!(100), Decimal::ZERO)
    );
}

/// ⛔ Un compte archivé qui porte encore un solde figure à la balance — comptes de
/// bilan et de résultat. Mutation tuée : la règle d'inclusion d'avant (archivé =
/// seulement s'il a un mouvement dans la période) — le compte disparaîtrait, sa
/// clôture ne concorderait plus avec le bilan et le contrôle rougirait.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_archive_porteur_d_un_solde_figure(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (_, fy26) = reference_books(&pool, &seeded).await;
    archive(&pool, seeded.accounts["1100"]).await;
    archive(&pool, seeded.accounts["3000"]).await;

    // Exercice 2 entier : la banque archivée porte son report et ses mouvements.
    let p = period(fy26, ymd(2026, 1, 1), ymd(2026, 12, 31));
    let tb = balance(&pool, &seeded, &p).await;
    let banque = tb
        .rows
        .iter()
        .find(|r| r.account_number == "1100")
        .expect("banque archivée présente");
    assert!(!banque.active);
    assert!(tb.opening_balanced);

    // Juillet : aucun mouvement de la banque ni des ventes ; les deux portent une
    // ouverture non nulle et doivent figurer.
    let july = period(fy26, ymd(2026, 7, 1), ymd(2026, 7, 31));
    let tb = balance(&pool, &seeded, &july).await;
    assert_eq!(row(&tb, "1100").0, dec!(1700));
    assert_eq!(row(&tb, "3000").0, dec!(820));
    assert!(tb.opening_balanced);
    let bs = generate_balance_sheet(&pool, seeded.company_id, &july)
        .await
        .expect("bilan");
    let b = bs
        .assets
        .iter()
        .find(|a| a.account_number == "1100")
        .expect("banque au bilan");
    assert_eq!(row(&tb, "1100").3, b.balance);
}
