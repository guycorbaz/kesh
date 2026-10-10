//! L'invariant des postes ouverts, contre les rapports — Story 15-1b (#518),
//! AC2, tests 1 et 2 de T6.
//!
//! Sur une base **mêlée** — la fixture partagée des pièces clientes de la
//! 15-1a2-i (soldée, partielle, héritée `paid_at`, créditée, rapprochée,
//! annulée, les deux exceptions nommées, une pièce sous la borne), des factures
//! fournisseurs (payée, annulée payée), des écritures manuelles lettrées et non
//! lettrées, une contre-passation, un groupe **à cheval** sur deux exercices et
//! un exercice **clos** —, pour chaque compte lettrable de la société :
//!
//! 1. aux trois dates (avant le groupe à cheval, entre ses deux lignes, après) :
//!    `openTotal == balance == Σ(débit − crédit)` des lignes datées ≤ X,
//!    recalculé dans le test ;
//! 2. à la fin de chaque exercice : `balance == debit_sense(type, clôture)` de
//!    la ligne du compte dans `trial_balance::generate` — l'accord avec le
//!    rapport que l'utilisateur ouvre pour clore.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::account::AccountType;
use kesh_db::repositories::letterings::{self, Actor, Mode, Origin};
use kesh_db::repositories::{journal_entries, supplier_invoices};
use kesh_report::opening::debit_sense;
use kesh_report::period::ReportPeriod;
use kesh_report::trial_balance;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

#[path = "../../kesh-db/tests/support/lettering_documents.rs"]
mod lettering_support;
use lettering_support::*;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Écriture posée en SQL direct (montage, y compris dans une période close).
async fn ecriture(
    pool: &MySqlPool,
    company: i64,
    fy: i64,
    date: NaiveDate,
    lignes: &[(i64, Decimal, Decimal)],
) -> (i64, Vec<i64>) {
    let numero: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ?",
    )
    .bind(company)
    .bind(fy)
    .fetch_one(pool)
    .await
    .unwrap();
    let entry = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, ?, ?, 'OD', 'invariant')",
    )
    .bind(company)
    .bind(fy)
    .bind(numero)
    .bind(date)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut ids = Vec::new();
    for (n, (compte, debit, credit)) in lignes.iter().enumerate() {
        ids.push(
            sqlx::query(
                "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(entry)
            .bind(*compte)
            .bind(n as i32 + 1)
            .bind(*debit)
            .bind(*credit)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_id() as i64,
        );
    }
    (entry, ids)
}

/// Tests 1 et 2 (AC2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn open_items_balance_matches_the_ledger_and_the_trial_balance(pool: MySqlPool) {
    let fixture = seed_lettering_documents(&pool).await;
    let s = &fixture.seeded;
    let company = s.company_id;
    let fy = s.fiscal_year_id;
    let actor = Actor {
        user_id: s.admin_user_id,
        api_key_id: None,
    };
    // Deux exercices : 2024 (clos à la fin du montage) et 2025-2030.
    sqlx::query("UPDATE fiscal_years SET start_date = '2025-01-01' WHERE id = ?")
        .bind(fy)
        .execute(&pool)
        .await
        .unwrap();
    let fy24 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2024', '2024-01-01', '2024-12-31', 'Open')",
    )
    .bind(company)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let creance = s.accounts["1100"];
    let contre = s.accounts["3000"];

    // Fournisseurs : une payée, une annulée payée.
    let a = achats(&pool, s).await;
    let payee =
        facture_fournisseur(&pool, s, &a, dec!(70.00), jours_avant(100), Some("FF-1")).await;
    payer(&pool, s, &a, payee, jours_avant(90)).await;
    let annulee =
        facture_fournisseur(&pool, s, &a, dec!(45.00), jours_avant(100), Some("FF-2")).await;
    payer(&pool, s, &a, annulee, jours_avant(90)).await;
    supplier_invoices::cancel(&pool, company, annulee, s.admin_user_id)
        .await
        .expect("annulation de la facture payée");

    // Écritures manuelles sur la créance : une non lettrée en 2024, une paire
    // lettrée en 2025-2030, une contre-passée.
    ecriture(
        &pool,
        company,
        fy24,
        ymd(2024, 3, 1),
        &[(creance, dec!(12), dec!(0)), (contre, dec!(0), dec!(12))],
    )
    .await;
    let (_, p1) = ecriture(
        &pool,
        company,
        fy,
        jours_avant(60),
        &[(creance, dec!(33), dec!(0)), (contre, dec!(0), dec!(33))],
    )
    .await;
    let (_, p2) = ecriture(
        &pool,
        company,
        fy,
        jours_avant(50),
        &[(creance, dec!(0), dec!(33)), (contre, dec!(33), dec!(0))],
    )
    .await;
    let lettrer = |ids: Vec<i64>| {
        let pool = pool.clone();
        async move {
            let mut tx = pool.begin().await.unwrap();
            letterings::create_group_in_tx(
                &mut tx,
                company,
                &ids,
                Origin::Manual,
                Mode::Manual,
                actor,
            )
            .await
            .expect("lettrage");
            tx.commit().await.unwrap();
        }
    };
    lettrer(vec![p1[0], p2[0]]).await;
    let (e_rev, _) = ecriture(
        &pool,
        company,
        fy,
        jours_avant(40),
        &[(creance, dec!(8), dec!(0)), (contre, dec!(0), dec!(8))],
    )
    .await;
    journal_entries::reverse(&pool, company, e_rev, s.admin_user_id)
        .await
        .expect("contre-passation");
    // Le groupe à cheval : 2024-06-30 et 2026-03-01 (au-dessus de la borne).
    let (_, a1) = ecriture(
        &pool,
        company,
        fy24,
        ymd(2024, 6, 30),
        &[(creance, dec!(100), dec!(0)), (contre, dec!(0), dec!(100))],
    )
    .await;
    let (_, a2) = ecriture(
        &pool,
        company,
        fy,
        ymd(2026, 3, 1),
        &[(creance, dec!(0), dec!(100)), (contre, dec!(100), dec!(0))],
    )
    .await;
    lettrer(vec![a1[0], a2[0]]).await;
    // L'exercice 2024 est clos.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy24)
        .execute(&pool)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let comptes = letterings::letterable_account_ids(&mut conn, company)
        .await
        .unwrap();
    assert!(
        comptes.contains(&creance) && comptes.contains(&a.payable),
        "montage : la créance et la dette sont lettrables"
    );
    let aujourdhui = chrono::Utc::now().date_naive();
    let dates = [ymd(2024, 6, 1), ymd(2024, 12, 31), aujourdhui];
    let mut lignes_ouvertes = 0;
    for &compte in &comptes {
        for x in dates {
            let v = letterings::open_items(&mut conn, company, compte, x, 500, 0)
                .await
                .expect("postes ouverts");
            let somme: Decimal = sqlx::query_scalar(
                "SELECT COALESCE(SUM(jel.debit - jel.credit), 0) FROM journal_entry_lines jel \
                 JOIN journal_entries je ON je.id = jel.entry_id \
                 WHERE jel.account_id = ? AND je.company_id = ? AND je.entry_date <= ?",
            )
            .bind(compte)
            .bind(company)
            .bind(x)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(
                v.open_total, v.balance,
                "compte {compte} au {x} : openTotal"
            );
            assert_eq!(v.balance, somme, "compte {compte} au {x} : balance");
            assert!(v.total <= 500, "montage : une seule page");
            let page: Decimal = v.items.iter().map(|i| i.debit - i.credit).sum();
            assert_eq!(
                page, v.open_total,
                "compte {compte} au {x} : Σ des lignes listées"
            );
            lignes_ouvertes += v.items.len();
        }
    }
    assert!(
        lignes_ouvertes > 20,
        "montage : la vue liste des lignes ({lignes_ouvertes})"
    );
    // Le groupe à cheval, entre ses deux lignes : la ligne de 2024 est ouverte.
    let entre = letterings::open_items(&mut conn, company, creance, ymd(2024, 12, 31), 500, 0)
        .await
        .unwrap();
    assert!(
        entre.items.iter().any(|i| i.line_id == a1[0]),
        "montage : groupe à cheval"
    );

    // Test 2 — à la fin de chaque exercice, contre la Balance.
    for (exercice, fin) in [(fy24, ymd(2024, 12, 31)), (fy, ymd(2030, 12, 31))] {
        let periode = ReportPeriod::resolve(&pool, company, exercice, None, None)
            .await
            .expect("période");
        let balance = trial_balance::generate(&pool, company, &periode)
            .await
            .expect("balance des comptes");
        for &compte in &comptes {
            let v = letterings::open_items(&mut conn, company, compte, fin, 500, 0)
                .await
                .unwrap();
            let ligne = balance.rows.iter().find(|r| r.account_id == compte);
            match ligne {
                Some(r) => assert_eq!(
                    v.balance,
                    debit_sense(r.account_type, r.closing_balance),
                    "compte {compte}, fin {fin} : Balance"
                ),
                None => assert_eq!(
                    v.balance,
                    Decimal::ZERO,
                    "compte {compte} absent de la Balance"
                ),
            }
        }
        // Montage : la créance figure à la Balance, avec un solde non nul.
        let r = balance
            .rows
            .iter()
            .find(|r| r.account_id == creance)
            .expect("créance à la Balance");
        assert_eq!(r.account_type, AccountType::Asset);
        assert_ne!(
            r.closing_balance,
            Decimal::ZERO,
            "montage : solde non nul au {fin}"
        );
    }
}
