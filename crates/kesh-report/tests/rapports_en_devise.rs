//! Story 37-4, AC-1 à AC-8 — les rapports **en devise**, sur un jeu à deux
//! exercices calculé à la main.
//!
//! Le jeu (société multi-devises, exercices 2025 et 2026) :
//!
//! | n° | date       | écriture                                                   |
//! |----|------------|------------------------------------------------------------|
//! | E1 | 2025-03-10 | 1020 Banque USD D 900.00 (USD 1000 × 0.90) / 2000 C 900.00 |
//! | E2 | 2025-06-15 | 1020 D 470.00 (USD 500 × 0.94) / 3200 Ventes USD C 470.00 (USD 500 × 0.94) |
//! | E3 | 2025-09-01 | 4000 Charges D 190.00 / 1020 C 190.00 (USD 200 × 0.95)     |
//! | E4 | 2025-12-31 | **réévaluation** 1020 D 15.00 (USD 0, cours 0.9312) / 3000 C 15.00 |
//! | E5 | 2026-02-10 | 1020 D 91.00 (USD 100 × 0.91) / 3200 C 91.00 (USD 100 × 0.91) |
//! | E6 | 2026-04-01 | 3200 D 91.00 (USD 91 × 1.00) / 1020 C 91.00 (USD 91 × 1.00) |
//! | E7 | 2026-05-05 | 1100 Banque D 470.00 / 2050 Emprunt EUR C 470.00 (EUR 500 × 0.94, manuel) |
//! | E8 | 2026-05-20 | 2050 D 94.00 (EUR 100 × 0.94, **AFC**) / 1100 C 94.00     |
//! | E9 | 2026-06-01 | 1030 Caisse EUR D 100.00 (EUR 105 × 0.9524) / 1000 C 100.00 |
//! | E10| 2026-06-02 | 1000 D 100.00 / 1030 C 100.00 (EUR 100 × 1.00) ; 1030 archivé ensuite |
//!
//! Soldes attendus, à la main (devise principale / devise) :
//! - 1020 Banque USD : fin 2025 1195.00 / 1300 ; fin 2026 1195.00 / 1309 ;
//! - 3200 Ventes USD : 2025 470.00 / 500 ; 2026 **0.00 / 9** (compte à CHF 0) ;
//! - 2050 Emprunt EUR : 376.00 / 400 ;
//! - 1030 Caisse EUR (archivé) : **0.00 / 5** (compte à CHF 0).
//!
//! T3 : une ligne en devise est ensuite posée **par SQL direct** sur la caisse
//! (compte CHF) : aucun rapport ne la montre en devise.

use chrono::NaiveDate;
use kesh_db::entities::{
    Acteur, ForeignAmount, Journal, NewJournalEntry, NewJournalEntryLine, RateProvenance,
};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, chf_line, seed_accounting_company};
use kesh_report::csv::{
    render_balance_sheet_csv, render_general_ledger_csv, render_income_statement_csv,
    render_trial_balance_csv,
};
use kesh_report::general_ledger::{
    GeneralLedger, LedgerOptions, LedgerPeriod, LedgerSection, generate as generate_ledger,
};
use kesh_report::pdf::{
    PdfContext, render_balance_sheet_pdf, render_general_ledger_pdf, render_income_statement_pdf,
    render_trial_balance_pdf,
};
use kesh_report::period::ReportPeriod;
use kesh_report::{generate_balance_sheet, generate_income_statement, generate_trial_balance};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

struct Jeu {
    s: SeededCompany,
    fy2025: i64,
    fy2026: i64,
    banque_usd: i64,
    ventes_usd: i64,
    emprunt_eur: i64,
    caisse_eur: i64,
    caisse: i64,
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    nom: &str,
    typ: &str,
    devise: &str,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(nom)
    .bind(typ)
    .bind(devise)
    .execute(pool)
    .await
    .expect("compte en devise")
    .last_insert_id() as i64
}

fn fx(amount: Decimal, rate: Decimal, source: RateProvenance) -> Option<ForeignAmount> {
    Some(ForeignAmount {
        declared_currency: None,
        amount,
        rate,
        source: Some(source),
        rate_derived: false,
    })
}

fn debit(account: i64, chf: Decimal, f: Option<ForeignAmount>) -> NewJournalEntryLine {
    NewJournalEntryLine {
        foreign: f,
        ..chf_line(account, chf, Decimal::ZERO)
    }
}

fn credit(account: i64, chf: Decimal, f: Option<ForeignAmount>) -> NewJournalEntryLine {
    NewJournalEntryLine {
        foreign: f,
        ..chf_line(account, Decimal::ZERO, chf)
    }
}

async fn poser(
    pool: &MySqlPool,
    s: &SeededCompany,
    fy: i64,
    date: NaiveDate,
    label: &str,
    lines: Vec<NewJournalEntryLine>,
) {
    journal_entries::create(
        pool,
        fy,
        Acteur::utilisateur(s.admin_user_id),
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: label.into(),
            project_id: None,
            lines,
        },
    )
    .await
    .unwrap_or_else(|e| panic!("écriture « {label} » : {e:?}"));
}

async fn jeu(pool: &MySqlPool) -> Jeu {
    let s = seed_accounting_company(pool).await.expect("seed");
    sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31', name = 'FY 2025' WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
    let fy2026: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, 'FY 2026', '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(s.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let fy2025 = s.fiscal_year_id;
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE company_id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .unwrap();
    // Cours AFC retenu pour mai 2026 (ligne E8, provenance `afc`).
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, 'EUR', 'month', '2026-05-01', 0.94, 'afc')",
    )
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap();

    let c = s.company_id;
    let banque_usd = compte(pool, c, "1020", "Banque USD", "Asset", "USD").await;
    let caisse_eur = compte(pool, c, "1030", "Caisse EUR", "Asset", "EUR").await;
    let emprunt_eur = compte(pool, c, "2050", "Emprunt EUR", "Liability", "EUR").await;
    let ventes_usd = compte(pool, c, "3200", "Ventes USD", "Revenue", "USD").await;
    let caisse = s.accounts["1000"];
    let banque = s.accounts["1100"];
    let capital = s.accounts["2000"];
    let ventes = s.accounts["3000"];
    let charges = s.accounts["4000"];
    use RateProvenance::{Afc, Manual, Revaluation};

    poser(
        pool,
        &s,
        fy2025,
        ymd(2025, 3, 10),
        "E1 apport en USD",
        vec![
            debit(banque_usd, dec!(900.00), fx(dec!(1000), dec!(0.90), Manual)),
            credit(capital, dec!(900.00), None),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2025,
        ymd(2025, 6, 15),
        "E2 vente USD",
        vec![
            debit(banque_usd, dec!(470.00), fx(dec!(500), dec!(0.94), Manual)),
            credit(ventes_usd, dec!(470.00), fx(dec!(500), dec!(0.94), Manual)),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2025,
        ymd(2025, 9, 1),
        "E3 charge payée en USD",
        vec![
            debit(charges, dec!(190.00), None),
            credit(banque_usd, dec!(190.00), fx(dec!(200), dec!(0.95), Manual)),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2025,
        ymd(2025, 12, 31),
        "E4 réévaluation",
        vec![
            debit(
                banque_usd,
                dec!(15.00),
                fx(Decimal::ZERO, dec!(0.9312), Revaluation),
            ),
            credit(ventes, dec!(15.00), None),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2026,
        ymd(2026, 2, 10),
        "E5 vente USD",
        vec![
            debit(banque_usd, dec!(91.00), fx(dec!(100), dec!(0.91), Manual)),
            credit(ventes_usd, dec!(91.00), fx(dec!(100), dec!(0.91), Manual)),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2026,
        ymd(2026, 4, 1),
        "E6 avoir USD",
        vec![
            debit(ventes_usd, dec!(91.00), fx(dec!(91), dec!(1.00), Manual)),
            credit(banque_usd, dec!(91.00), fx(dec!(91), dec!(1.00), Manual)),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2026,
        ymd(2026, 5, 5),
        "E7 emprunt EUR",
        vec![
            debit(banque, dec!(470.00), None),
            credit(emprunt_eur, dec!(470.00), fx(dec!(500), dec!(0.94), Manual)),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2026,
        ymd(2026, 5, 20),
        "E8 remboursement EUR",
        vec![
            debit(emprunt_eur, dec!(94.00), fx(dec!(100), dec!(0.94), Afc)),
            credit(banque, dec!(94.00), None),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2026,
        ymd(2026, 6, 1),
        "E9 achat EUR",
        vec![
            debit(
                caisse_eur,
                dec!(100.00),
                fx(dec!(105), dec!(0.9524), Manual),
            ),
            credit(caisse, dec!(100.00), None),
        ],
    )
    .await;
    poser(
        pool,
        &s,
        fy2026,
        ymd(2026, 6, 2),
        "E10 vente EUR",
        vec![
            debit(caisse, dec!(100.00), None),
            credit(caisse_eur, dec!(100.00), fx(dec!(100), dec!(1.00), Manual)),
        ],
    )
    .await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(caisse_eur)
        .execute(pool)
        .await
        .unwrap();

    Jeu {
        s,
        fy2025,
        fy2026,
        banque_usd,
        ventes_usd,
        emprunt_eur,
        caisse_eur,
        caisse,
    }
}

/// T3 — une ligne en devise posée **par SQL direct** sur la caisse (compte
/// CHF) : la ligne CHF de E9 reçoit un montant en devise cohérent avec les
/// contraintes du schéma, ce que le dépôt refuserait.
async fn ligne_en_devise_hors_depot_sur_un_compte_chf(pool: &MySqlPool, j: &Jeu) {
    let n = sqlx::query(
        "UPDATE journal_entry_lines SET amount_currency = 100, exchange_rate = 1, \
         rate_source = 'manual' WHERE account_id = ? AND credit = 100",
    )
    .bind(j.caisse)
    .execute(pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 1, "la ligne CHF de E9");
}

fn fy2026(j: &Jeu) -> ReportPeriod {
    ReportPeriod {
        fiscal_year_id: j.fy2026,
        start_date: ymd(2026, 1, 1),
        end_date: ymd(2026, 12, 31),
    }
}

async fn ledger(
    pool: &MySqlPool,
    j: &Jeu,
    from: NaiveDate,
    to: NaiveDate,
    ids: Option<Vec<i64>>,
    offset: i64,
    limit: Option<i64>,
) -> GeneralLedger {
    generate_ledger(
        pool,
        j.s.company_id,
        &LedgerPeriod::new(from, to).unwrap(),
        &LedgerOptions {
            account_ids: ids,
            offset,
            limit,
            ..Default::default()
        },
    )
    .await
    .expect("grand livre")
}

fn section(gl: &GeneralLedger, id: i64) -> &LedgerSection {
    gl.sections
        .iter()
        .find(|s| s.account_id == id)
        .unwrap_or_else(|| panic!("section du compte {id} absente"))
}

/// AC-2, AC-1 bis — grand livre sur une période qui franchit la fin
/// d'exercice : ouverture, mouvements, solde progressif, rupture, clôture et
/// totaux en devise, calculés à la main.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn grand_livre_en_devise_a_cheval_sur_deux_exercices(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let gl = ledger(&pool, &j, ymd(2025, 7, 1), ymd(2026, 6, 30), None, 0, None).await;

    // Compte de BILAN : chaîne continue, tous exercices.
    let b = section(&gl, j.banque_usd);
    assert_eq!(b.currency.as_deref(), Some("USD"));
    assert_eq!(
        (b.opening, b.opening_currency),
        (dec!(1370.00), Some(dec!(1500)))
    );
    let progressif: Vec<_> = b
        .lines
        .iter()
        .map(|l| (l.running_balance, l.running_balance_currency))
        .collect();
    assert_eq!(
        progressif,
        vec![
            (dec!(1180.00), Some(dec!(1300))),
            (dec!(1195.00), Some(dec!(1300))), // réévaluation : CHF seul
            (dec!(1286.00), Some(dec!(1400))),
            (dec!(1195.00), Some(dec!(1309))),
        ]
    );
    let reeval = &b.lines[1];
    assert!(reeval.is_revaluation());
    assert_eq!(
        (
            reeval.debit_currency,
            reeval.credit_currency,
            reeval.exchange_rate
        ),
        (Some(Decimal::ZERO), Some(Decimal::ZERO), Some(dec!(0.9312)))
    );
    assert_eq!(
        (b.lines[0].debit_currency, b.lines[0].credit_currency),
        (Some(Decimal::ZERO), Some(dec!(200)))
    );
    assert_eq!(b.lines[0].rate_source.as_deref(), Some("manual"));
    assert_eq!(
        (b.total_debit_currency, b.total_credit_currency),
        (Some(dec!(100)), Some(dec!(291)))
    );
    assert_eq!(
        (b.closing, b.closing_currency),
        (dec!(1195.00), Some(dec!(1309)))
    );
    assert!(b.fiscal_year_breaks.is_empty());

    // Compte de RÉSULTAT : ouverture depuis le début de l'exercice de `from`,
    // remise à zéro à la rupture, clôture = solde de l'exercice final (#778).
    let v = section(&gl, j.ventes_usd);
    assert_eq!(
        (v.opening, v.opening_currency),
        (dec!(470.00), Some(dec!(500)))
    );
    assert_eq!(v.fiscal_year_breaks.len(), 1);
    let r = &v.fiscal_year_breaks[0];
    assert_eq!(
        (
            r.closing_fiscal_year_id,
            r.closing_balance,
            r.closing_balance_currency
        ),
        (j.fy2025, dec!(470.00), Some(dec!(500)))
    );
    let progressif: Vec<_> = v
        .lines
        .iter()
        .map(|l| (l.running_balance, l.running_balance_currency))
        .collect();
    assert_eq!(
        progressif,
        vec![
            (dec!(91.00), Some(dec!(100))),
            (Decimal::ZERO, Some(dec!(9)))
        ]
    );
    assert_eq!(
        (v.closing, v.closing_currency),
        (Decimal::ZERO, Some(dec!(9)))
    );
    assert_eq!(
        (v.total_debit_currency, v.total_credit_currency),
        (Some(dec!(91)), Some(dec!(100)))
    );
    // `unnatural_balance` reste lu en devise principale.
    assert!(!v.unnatural_balance);

    // Un compte CHF n'a aucun champ en devise.
    let c = section(&gl, j.caisse);
    assert!(c.currency.is_none() && c.opening_currency.is_none() && c.closing_currency.is_none());
    assert!(c.lines.iter().all(|l| l.debit_currency.is_none()
        && l.running_balance_currency.is_none()
        && l.exchange_rate.is_none()));
}

/// #775 — au-delà de la première page, le solde progressif en devise part de
/// l'ouverture plus les lignes sautées, ruptures comprises.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn grand_livre_en_devise_pagine_offset(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let ids = Some(vec![j.ventes_usd]);
    let p2 = ledger(
        &pool,
        &j,
        ymd(2025, 7, 1),
        ymd(2026, 6, 30),
        ids.clone(),
        1,
        Some(1),
    )
    .await;
    let v = section(&p2, j.ventes_usd);
    assert_eq!(v.lines.len(), 1);
    assert_eq!(
        (
            v.lines[0].running_balance,
            v.lines[0].running_balance_currency
        ),
        (Decimal::ZERO, Some(dec!(9)))
    );
    // La rupture appartient à la page de la première ligne de 2026.
    assert!(v.fiscal_year_breaks.is_empty());
    assert_eq!(
        v.closing_currency,
        Some(dec!(9)),
        "totaux sur la période entière"
    );

    let b = ledger(
        &pool,
        &j,
        ymd(2025, 7, 1),
        ymd(2026, 6, 30),
        Some(vec![j.banque_usd]),
        2,
        Some(1),
    )
    .await;
    let s = section(&b, j.banque_usd);
    assert_eq!(s.lines[0].running_balance_currency, Some(dec!(1400)));
}

/// AC-3 — balance : colonnes en devise par compte en devise, totaux en devise
/// principale seulement ; compte archivé à CHF 0 et devise ≠ 0 rendu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn balance_en_devise(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let tb = generate_trial_balance(&pool, j.s.company_id, &fy2026(&j))
        .await
        .unwrap();
    let row = |id: i64| tb.rows.iter().find(|r| r.account_id == id).unwrap();

    let b = row(j.banque_usd);
    assert_eq!(b.currency.as_deref(), Some("USD"));
    assert_eq!(
        (
            b.opening_currency,
            b.total_debit_currency,
            b.total_credit_currency,
            b.balance_currency,
            b.closing_currency
        ),
        (
            Some(dec!(1300)),
            Some(dec!(100)),
            Some(dec!(91)),
            Some(dec!(9)),
            Some(dec!(1309))
        )
    );
    assert_eq!(
        (b.opening_balance, b.closing_balance),
        (dec!(1195.00), dec!(1195.00))
    );

    let v = row(j.ventes_usd);
    assert_eq!(
        (v.opening_currency, v.closing_currency, v.closing_balance),
        (Some(Decimal::ZERO), Some(dec!(9)), Decimal::ZERO)
    );

    let e = row(j.emprunt_eur);
    assert_eq!(
        (
            e.total_debit_currency,
            e.total_credit_currency,
            e.closing_currency,
            e.closing_balance
        ),
        (
            Some(dec!(100)),
            Some(dec!(500)),
            Some(dec!(400)),
            dec!(376.00)
        )
    );

    let ce = row(j.caisse_eur);
    assert!(!ce.active);
    assert_eq!(
        (ce.closing_balance, ce.closing_currency),
        (Decimal::ZERO, Some(dec!(5)))
    );

    // Totaux : devise principale, équilibrés ; aucun champ en devise sur un compte CHF.
    assert!(tb.balanced);
    assert!(tb.opening_balanced);
    let caisse = row(j.caisse);
    assert!(caisse.currency.is_none() && caisse.closing_currency.is_none());
}

/// AC-1 ter (balance) : un compte **archivé**, sans mouvement sur la période,
/// à CHF 0 et devise ≠ 0, est rendu par son seul solde en devise.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn balance_compte_archive_a_chf_zero_rendu_par_sa_devise(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let fy2027: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency, created_at, updated_at) \
         VALUES (?, 'FY 2027', '2027-01-01', '2027-12-31', 'Open', TRUE, NOW(3), NOW(3)) RETURNING id",
    )
    .bind(j.s.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let tb = generate_trial_balance(
        &pool,
        j.s.company_id,
        &ReportPeriod {
            fiscal_year_id: fy2027,
            start_date: ymd(2027, 1, 1),
            end_date: ymd(2027, 12, 31),
        },
    )
    .await
    .unwrap();
    let ce = tb
        .rows
        .iter()
        .find(|r| r.account_id == j.caisse_eur)
        .expect("archivé, EUR 5 : rendu");
    assert_eq!(
        (ce.opening_balance, ce.opening_currency),
        (Decimal::ZERO, Some(dec!(5)))
    );

    // Grand livre de 2027 : la caisse EUR, sans ligne, rendue par son ouverture en devise.
    let gl = ledger(&pool, &j, ymd(2027, 1, 1), ymd(2027, 12, 31), None, 0, None).await;
    let s = section(&gl, j.caisse_eur);
    assert_eq!(
        (s.opening, s.opening_currency, s.line_count),
        (Decimal::ZERO, Some(dec!(5)), 0)
    );
}

/// AC-4 — bilan et compte de résultat : devise et solde en devise ; compte à
/// CHF 0 et devise ≠ 0 listé ; totaux en devise principale.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bilan_et_resultat_en_devise(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let bs = generate_balance_sheet(&pool, j.s.company_id, &fy2026(&j))
        .await
        .unwrap();
    let a = |id: i64| {
        bs.assets
            .iter()
            .chain(&bs.liabilities)
            .find(|a| a.account_id == id)
            .cloned()
    };
    let b = a(j.banque_usd).unwrap();
    assert_eq!(
        (b.currency.as_deref(), b.balance, b.balance_currency),
        (Some("USD"), dec!(1195.00), Some(dec!(1309)))
    );
    let ce = a(j.caisse_eur).expect("CHF 0 / EUR 5 : listé au bilan");
    assert_eq!(
        (ce.balance, ce.balance_currency),
        (Decimal::ZERO, Some(dec!(5)))
    );
    let e = a(j.emprunt_eur).unwrap();
    assert_eq!(
        (e.balance, e.balance_currency),
        (dec!(376.00), Some(dec!(400)))
    );
    assert!(bs.equation_holds);
    let caisse = a(j.caisse).unwrap();
    assert!(caisse.currency.is_none() && caisse.balance_currency.is_none());

    let is_ = generate_income_statement(&pool, j.s.company_id, &fy2026(&j))
        .await
        .unwrap();
    let v = is_
        .revenues
        .iter()
        .find(|a| a.account_id == j.ventes_usd)
        .expect("CHF 0 / USD 9 : listé");
    assert_eq!(
        (v.balance, v.balance_currency),
        (Decimal::ZERO, Some(dec!(9)))
    );
    // 2025 : USD 500 de ventes, réévaluation sur un compte CHF.
    let is25 = generate_income_statement(
        &pool,
        j.s.company_id,
        &ReportPeriod {
            fiscal_year_id: j.fy2025,
            start_date: ymd(2025, 1, 1),
            end_date: ymd(2025, 12, 31),
        },
    )
    .await
    .unwrap();
    let v25 = is25
        .revenues
        .iter()
        .find(|a| a.account_id == j.ventes_usd)
        .unwrap();
    assert_eq!(
        (v25.balance, v25.balance_currency),
        (dec!(470.00), Some(dec!(500)))
    );
    assert_eq!(
        is25.total_revenues,
        dec!(485.00),
        "total en devise principale seulement"
    );
}

/// AC-8 — concordance en devise : la clôture du grand livre d'un exercice égale
/// la clôture de la balance, le solde au bilan (comptes de bilan) et au compte
/// de résultat (comptes de résultat). Appel réel des quatre générateurs.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn concordance_des_quatre_rapports_en_devise(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let p = fy2026(&j);
    let gl = ledger(&pool, &j, ymd(2026, 1, 1), ymd(2026, 12, 31), None, 0, None).await;
    let tb = generate_trial_balance(&pool, j.s.company_id, &p)
        .await
        .unwrap();
    let bs = generate_balance_sheet(&pool, j.s.company_id, &p)
        .await
        .unwrap();
    let is_ = generate_income_statement(&pool, j.s.company_id, &p)
        .await
        .unwrap();

    for id in [j.banque_usd, j.emprunt_eur, j.caisse_eur] {
        let g = section(&gl, id).closing_currency;
        let t = tb
            .rows
            .iter()
            .find(|r| r.account_id == id)
            .unwrap()
            .closing_currency;
        let b = bs
            .assets
            .iter()
            .chain(&bs.liabilities)
            .find(|a| a.account_id == id)
            .unwrap()
            .balance_currency;
        assert_eq!((g, t), (b, b), "compte de bilan {id}");
        assert_eq!(
            section(&gl, id).opening_currency,
            tb.rows
                .iter()
                .find(|r| r.account_id == id)
                .unwrap()
                .opening_currency
        );
    }
    let g = section(&gl, j.ventes_usd).closing_currency;
    let t = tb
        .rows
        .iter()
        .find(|r| r.account_id == j.ventes_usd)
        .unwrap()
        .closing_currency;
    let i = is_
        .revenues
        .iter()
        .find(|a| a.account_id == j.ventes_usd)
        .unwrap()
        .balance_currency;
    assert_eq!((g, t), (i, i), "compte de résultat");
}

/// T3 — la devise est celle du compte : le compte EUR à deux lignes (`afc` et
/// `manual`) donne la paire attendue ; une ligne en devise posée par SQL direct
/// sur un compte CHF n'apparaît dans aucun champ en devise des cinq rapports.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t3_devise_du_compte_et_ligne_hors_depot(pool: MySqlPool) {
    let j = jeu(&pool).await;
    ligne_en_devise_hors_depot_sur_un_compte_chf(&pool, &j).await;
    let p = fy2026(&j);

    let gl = ledger(&pool, &j, ymd(2026, 1, 1), ymd(2026, 12, 31), None, 0, None).await;
    let e = section(&gl, j.emprunt_eur);
    let sources: Vec<_> = e
        .lines
        .iter()
        .map(|l| l.rate_source.clone().unwrap())
        .collect();
    assert_eq!(sources, vec!["manual", "afc"]);
    assert_eq!(
        (e.total_debit_currency, e.total_credit_currency),
        (Some(dec!(100)), Some(dec!(500)))
    );
    let c = section(&gl, j.caisse);
    assert!(c.currency.is_none());
    assert!(c.lines.iter().all(|l| l.debit_currency.is_none()
        && l.credit_currency.is_none()
        && l.exchange_rate.is_none()
        && l.rate_source.is_none()
        && l.running_balance_currency.is_none()));
    let json = serde_json::to_string(c).unwrap();
    assert!(
        !json.contains("Currency") && !json.contains("exchangeRate"),
        "{json}"
    );

    let tb = generate_trial_balance(&pool, j.s.company_id, &p)
        .await
        .unwrap();
    let r = tb.rows.iter().find(|r| r.account_id == j.caisse).unwrap();
    assert!(
        r.currency.is_none() && r.opening_currency.is_none() && r.total_credit_currency.is_none()
    );

    let bs = generate_balance_sheet(&pool, j.s.company_id, &p)
        .await
        .unwrap();
    let a = bs.assets.iter().find(|a| a.account_id == j.caisse).unwrap();
    assert!(a.currency.is_none() && a.balance_currency.is_none());

    let jr = kesh_report::journal_report::generate(&pool, j.s.company_id, &p, None)
        .await
        .unwrap();
    let lignes: Vec<_> = jr
        .journals
        .iter()
        .flat_map(|s| &s.entries)
        .flat_map(|e| &e.lines)
        .collect();
    assert!(
        lignes
            .iter()
            .filter(|l| l.account_id == j.caisse)
            .all(|l| l.currency.is_none()
                && l.amount_currency.is_none()
                && l.exchange_rate.is_none())
    );
    // Les lignes du compte EUR, elles, portent la devise du compte.
    let eur: Vec<_> = lignes
        .iter()
        .filter(|l| l.account_id == j.emprunt_eur)
        .collect();
    assert!(eur.iter().all(|l| l.currency.as_deref() == Some("EUR")));
    assert_eq!(
        eur.iter()
            .map(|l| l.amount_currency.unwrap())
            .collect::<Vec<_>>(),
        vec![dec!(500), dec!(100)]
    );
}

/// AC-7 — le journal : les champs en devise au JSON ; la réévaluation porte
/// un montant nul et sa provenance.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn journal_porte_les_champs_en_devise(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let p = ReportPeriod {
        fiscal_year_id: j.fy2025,
        start_date: ymd(2025, 1, 1),
        end_date: ymd(2025, 12, 31),
    };
    let jr = kesh_report::journal_report::generate(&pool, j.s.company_id, &p, None)
        .await
        .unwrap();
    let reeval = jr
        .journals
        .iter()
        .flat_map(|s| &s.entries)
        .find(|e| e.description == "E4 réévaluation")
        .unwrap();
    let l = reeval
        .lines
        .iter()
        .find(|l| l.account_id == j.banque_usd)
        .unwrap();
    assert_eq!(
        (
            l.currency.as_deref(),
            l.amount_currency,
            l.rate_source.as_deref()
        ),
        (Some("USD"), Some(Decimal::ZERO), Some("revaluation"))
    );
    let v = serde_json::to_value(l).unwrap();
    assert_eq!(v["exchangeRate"], "0.9312000000");
}

/// Lignes de texte d'un PDF non compressé (profil de test), comme au test de
/// non-régression mono-devise.
fn pdf_text(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    let mut out = Vec::new();
    let mut rest: &str = &s;
    while let Some(end) = rest.find("> Tj") {
        let before = &rest[..end];
        if let Some(start) = before.rfind('<') {
            let hex = &before[start + 1..];
            let decoded: String = (0..hex.len() / 2)
                .filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
                .map(char::from)
                .collect();
            out.push(decoded);
        }
        rest = &rest[end + 4..];
    }
    out.join("\n")
}

/// AC-5 — CSV et PDF des quatre rapports : colonnes et mentions en devise,
/// montants arrondis, cours par unité ; aucun total en devise hors grand livre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn exports_csv_et_pdf_en_devise(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let p = fy2026(&j);
    let ctx = PdfContext::fr_ch_default("CI Test Company");
    let gl = ledger(&pool, &j, ymd(2025, 7, 1), ymd(2026, 6, 30), None, 0, None).await;
    let tb = generate_trial_balance(&pool, j.s.company_id, &p)
        .await
        .unwrap();
    let bs = generate_balance_sheet(&pool, j.s.company_id, &p)
        .await
        .unwrap();
    let is_ = generate_income_statement(&pool, j.s.company_id, &p)
        .await
        .unwrap();

    let csv = |f: &dyn Fn(&mut Vec<u8>)| {
        let mut b = Vec::new();
        f(&mut b);
        String::from_utf8(b).unwrap()
    };
    // Grand livre.
    let c = csv(&|b| render_general_ledger_csv(&gl, b).unwrap());
    let l: Vec<&str> = c.lines().collect();
    assert!(
        l[0].ends_with(
            ";Devise;DebitDevise;CreditDevise;Cours;SoldeProgressifDevise;MentionDevise"
        ),
        "{}",
        l[0]
    );
    assert!(
        c.contains(
            "1020;Banque USD;2025-07-01;;;;Solde d'ouverture;;;;1370.00;USD;;;;1500.00;\r\n"
        ),
        "{c}"
    );
    assert!(
        c.contains(";15.00;0.00;1195.00;USD;;;0.9312;1300.00;Réévaluation au cours 0.9312\r\n"),
        "{c}"
    );
    assert!(
        c.contains(";0.00;190.00;1180.00;USD;0.00;200.00;0.9500;1300.00;\r\n"),
        "cours par unité à 4 décimales au moins : {c}"
    );
    assert!(
        c.contains("Total des mouvements;;106.00;281.00;;USD;100.00;291.00;;;\r\n"),
        "{c}"
    );
    // Rupture d'exercice du compte de résultat, en devise.
    assert!(c.contains("3200;Ventes USD;2026-02-10;;;;Clôture de l'exercice — le solde repart de zéro;;;;470.00;USD;;;;500.00;\r\n"), "{c}");
    // Compte CHF : colonnes vides.
    assert!(
        c.contains("1000;Caisse CI;2025-07-01;;;;Solde d'ouverture;;;;0.00;;;;;;\r\n"),
        "{c}"
    );

    // Balance.
    let c = csv(&|b| render_trial_balance_csv(&tb, b).unwrap());
    assert!(
        c.contains(
            "1020;Banque USD;1195.00;91.00;91.00;1195.00;USD;1300.00;100.00;91.00;1309.00\r\n"
        ),
        "{c}"
    );
    assert!(c.contains("Total;;;"), "{c}");
    assert!(
        c.lines().last().unwrap().ends_with(";;;;;"),
        "aucun total en devise : {c}"
    );

    // Bilan et compte de résultat.
    let c = csv(&|b| render_balance_sheet_csv(&bs, b).unwrap());
    assert!(
        c.starts_with("\u{feff}Section;NumeroCompte;NomCompte;Solde;Devise;SoldeDevise\r\n"),
        "{c}"
    );
    assert!(
        c.contains("Actifs;1030;Caisse EUR;0.00;EUR;5.00\r\n"),
        "{c}"
    );
    assert!(c.contains("Total actifs;;;"), "{c}");
    let c = csv(&|b| render_income_statement_csv(&is_, b).unwrap());
    assert!(
        c.contains("Produits;3200;Ventes USD;0.00;USD;9.00\r\n"),
        "{c}"
    );

    // PDF.
    let t = pdf_text(&render_general_ledger_pdf(&gl, &ctx).unwrap());
    // (Le tiret cadratin est en WinAnsi 0x97, que la lecture Latin-1 ne rend pas.)
    assert!(t.contains("Banque USD · USD"), "{t}");
    assert!(
        t.contains("USD 200.00 × 0.9500 · solde USD 1'300.00"),
        "{t}"
    );
    assert!(
        t.contains("Réévaluation au cours 0.9312 · solde USD 1'300.00"),
        "{t}"
    );
    assert!(t.contains("USD 1'500.00"), "ouverture en devise : {t}");
    assert!(t.contains("débit USD 100.00 · crédit USD 291.00"), "{t}");
    let t = pdf_text(&render_trial_balance_pdf(&tb, &ctx).unwrap());
    assert!(
        t.contains("USD ouverture 1'300.00 · débit 100.00 · crédit 91.00 · clôture 1'309.00"),
        "{t}"
    );
    let t = pdf_text(&render_balance_sheet_pdf(&bs, &ctx).unwrap());
    assert!(t.contains("USD 1'309.00"), "{t}");
    let t = pdf_text(&render_income_statement_pdf(&is_, &ctx).unwrap());
    assert!(t.contains("USD 9.00"), "{t}");
}
