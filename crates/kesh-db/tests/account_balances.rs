//! Story 37-4, AC-1 quater — `account_balances::balances_as_of`, la fonction à
//! date que la réévaluation de fin d'exercice (37-5) appelle dans sa
//! transaction : borne inclusive, réévaluation du jour comptée, comptes sans
//! ligne, `None` / `Some(ids)` / `Some(&[])`, comptes d'une autre société
//! ignorés (KF-002), lecture dans une transaction.

use chrono::NaiveDate;
use kesh_db::entities::{
    Acteur, ForeignAmount, Journal, NewJournalEntry, NewJournalEntryLine, RateProvenance,
};
use kesh_db::repositories::account_balances::{Cote, balances_as_of, somme_en_devise};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, chf_line, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

struct Decor {
    s: SeededCompany,
    usd: i64,
    /// Compte USD sans aucune ligne.
    usd_vide: i64,
    eur_passif: i64,
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    typ: &str,
    devise: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(typ)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn decor(pool: &MySqlPool) -> Decor {
    let s = seed_accounting_company(pool).await.unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
    let usd = compte(pool, s.company_id, "1020", "Asset", Some("USD")).await;
    let usd_vide = compte(pool, s.company_id, "1021", "Asset", Some("USD")).await;
    let eur_passif = compte(pool, s.company_id, "2050", "Liability", Some("EUR")).await;
    Decor {
        s,
        usd,
        usd_vide,
        eur_passif,
    }
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

/// Écriture à deux lignes : `compte` en devise au débit (ou au crédit) contre
/// la caisse CHF.
async fn poser(
    pool: &MySqlPool,
    dec: &Decor,
    date: NaiveDate,
    compte: i64,
    au_debit: bool,
    chf: Decimal,
    f: Option<ForeignAmount>,
) {
    let caisse = dec.s.accounts["1000"];
    let (devise, contre) = if au_debit {
        (
            NewJournalEntryLine {
                foreign: f,
                ..chf_line(compte, chf, Decimal::ZERO)
            },
            chf_line(caisse, Decimal::ZERO, chf),
        )
    } else {
        (
            NewJournalEntryLine {
                foreign: f,
                ..chf_line(compte, Decimal::ZERO, chf)
            },
            chf_line(caisse, chf, Decimal::ZERO),
        )
    };
    journal_entries::create(
        pool,
        dec.s.fiscal_year_id,
        Acteur::utilisateur(dec.s.admin_user_id),
        NewJournalEntry {
            company_id: dec.s.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "Écriture en devise".into(),
            project_id: None,
            lines: vec![devise, contre],
        },
    )
    .await
    .expect("écriture posée");
}

/// Le jeu : USD 1000 achetés à 0.90 (900.00), USD 200 vendus à 0.95 (190.00),
/// puis réévaluation +15.00 au 2026-03-31 ; EUR 500 empruntés à 0.94 (passif).
async fn jeu(pool: &MySqlPool) -> Decor {
    let dec = decor(pool).await;
    poser(
        pool,
        &dec,
        d(2026, 1, 10),
        dec.usd,
        true,
        dec!(900.00),
        fx(dec!(1000), dec!(0.90), RateProvenance::Manual),
    )
    .await;
    poser(
        pool,
        &dec,
        d(2026, 2, 10),
        dec.usd,
        false,
        dec!(190.00),
        fx(dec!(200), dec!(0.95), RateProvenance::Manual),
    )
    .await;
    poser(
        pool,
        &dec,
        d(2026, 2, 20),
        dec.eur_passif,
        false,
        dec!(470.00),
        fx(dec!(500), dec!(0.94), RateProvenance::Manual),
    )
    .await;
    poser(
        pool,
        &dec,
        d(2026, 3, 31),
        dec.usd,
        true,
        dec!(15.00),
        fx(Decimal::ZERO, dec!(0.9312), RateProvenance::Revaluation),
    )
    .await;
    dec
}

async fn appel(
    pool: &MySqlPool,
    company_id: i64,
    as_of: NaiveDate,
    ids: Option<&[i64]>,
) -> Vec<(i64, Decimal, Option<Decimal>)> {
    let mut conn = pool.acquire().await.unwrap();
    balances_as_of(&mut conn, company_id, as_of, ids)
        .await
        .unwrap()
}

#[test]
fn le_fragment_lit_le_cote_sur_le_montant_principal() {
    assert_eq!(
        somme_en_devise("jel", Cote::Credit, None),
        "COALESCE(SUM(CASE WHEN jel.credit > 0 THEN jel.amount_currency ELSE 0 END), 0)"
    );
    assert!(!somme_en_devise("x", Cote::Debit, Some("x.a = ?")).contains("currency ="));
}

/// `None` : tous les comptes en devise de la société, et eux seuls, chacun
/// rendu même sans ligne ; sens débit brut, sans signe par type.
#[sqlx::test(migrations = "./test-schema")]
async fn none_rend_tous_les_comptes_en_devise(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let r = appel(&pool, dec.s.company_id, d(2026, 12, 31), None).await;
    assert_eq!(
        r,
        vec![
            // 900 − 190 + 15 ; USD 1000 − 200 (réévaluation : 0).
            (dec.usd, dec!(725.00), Some(dec!(800))),
            (dec.usd_vide, Decimal::ZERO, Some(Decimal::ZERO)),
            // Passif au crédit : sens débit brut, négatif.
            (dec.eur_passif, dec!(-470.00), Some(dec!(-500))),
        ]
    );
}

/// Borne inclusive : la réévaluation posée au `as_of` est comptée en devise
/// principale (pas en devise) ; la veille, elle ne l'est pas.
#[sqlx::test(migrations = "./test-schema")]
async fn la_borne_est_inclusive_et_compte_la_reevaluation_du_jour(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let ids = [dec.usd];
    let veille = appel(&pool, dec.s.company_id, d(2026, 3, 30), Some(&ids)).await;
    let jour = appel(&pool, dec.s.company_id, d(2026, 3, 31), Some(&ids)).await;
    assert_eq!(veille, vec![(dec.usd, dec!(710.00), Some(dec!(800)))]);
    assert_eq!(jour, vec![(dec.usd, dec!(725.00), Some(dec!(800)))]);
    // Une ligne posée le jour même compte ; celle du lendemain, non.
    let avant_achat = appel(&pool, dec.s.company_id, d(2026, 1, 9), Some(&ids)).await;
    let jour_achat = appel(&pool, dec.s.company_id, d(2026, 1, 10), Some(&ids)).await;
    assert_eq!(
        avant_achat,
        vec![(dec.usd, Decimal::ZERO, Some(Decimal::ZERO))]
    );
    assert_eq!(jour_achat, vec![(dec.usd, dec!(900.00), Some(dec!(1000)))]);
}

/// `Some(ids)` : exactement ces comptes ; un compte CHF rend `None` en devise ;
/// un identifiant d'une autre société est ignoré sans erreur (KF-002) ;
/// `Some(&[])` rend un vecteur vide.
#[sqlx::test(migrations = "./test-schema")]
async fn some_ids_borne_a_la_societe_et_rend_chaque_compte(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let autre: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let usd_autre = compte(&pool, autre, "1020", "Asset", Some("USD")).await;
    let caisse = dec.s.accounts["1000"];

    let r = appel(
        &pool,
        dec.s.company_id,
        d(2026, 12, 31),
        Some(&[dec.usd_vide, caisse, usd_autre]),
    )
    .await;
    assert_eq!(
        r,
        vec![
            // Caisse : −900 + 190 + 470 − 15 = −255, aucune devise.
            (caisse, dec!(-255.00), None),
            (dec.usd_vide, Decimal::ZERO, Some(Decimal::ZERO)),
        ]
    );
    assert!(
        appel(&pool, dec.s.company_id, d(2026, 12, 31), Some(&[]))
            .await
            .is_empty()
    );
    // L'autre société ne voit que son compte, sans ligne.
    assert_eq!(
        appel(&pool, autre, d(2026, 12, 31), None).await,
        vec![(usd_autre, Decimal::ZERO, Some(Decimal::ZERO))]
    );
}

/// Revue de code P1 — un identifiant répété n'est rendu qu'une fois, et la
/// requête liée aux identifiants distincts (le décompte de l'appelant se fait
/// sur eux).
#[sqlx::test(migrations = "./test-schema")]
async fn un_identifiant_repete_n_est_rendu_qu_une_fois(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let r = appel(
        &pool,
        dec.s.company_id,
        d(2026, 12, 31),
        Some(&[dec.usd, dec.usd_vide, dec.usd]),
    )
    .await;
    assert_eq!(
        r,
        vec![
            (dec.usd, dec!(725.00), Some(dec!(800))),
            (dec.usd_vide, Decimal::ZERO, Some(Decimal::ZERO)),
        ]
    );
}

/// Dans une transaction : la lecture voit ce que la transaction a écrit et
/// n'a pas encore validé — c'est la connexion fournie qui lit.
#[sqlx::test(migrations = "./test-schema")]
async fn la_lecture_se_fait_dans_la_transaction_fournie(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    let entry: i64 = sqlx::query_scalar("SELECT MIN(id) FROM journal_entries WHERE company_id = ?")
        .bind(dec.s.company_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    // Ligne posée en SQL dans la transaction, non validée (le contrôle central
    // n'est pas l'objet de ce test).
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit, \
         amount_currency, exchange_rate, rate_source) VALUES (?, ?, 99, 9.00, 0, 10, 0.9, 'manual')",
    )
    .bind(entry)
    .bind(dec.usd_vide)
    .execute(&mut *tx)
    .await
    .unwrap();
    let dans = balances_as_of(
        &mut tx,
        dec.s.company_id,
        d(2026, 12, 31),
        Some(&[dec.usd_vide]),
    )
    .await
    .unwrap();
    assert_eq!(dans, vec![(dec.usd_vide, dec!(9.00), Some(dec!(10)))]);
    tx.rollback().await.unwrap();
    assert_eq!(
        appel(
            &pool,
            dec.s.company_id,
            d(2026, 12, 31),
            Some(&[dec.usd_vide])
        )
        .await,
        vec![(dec.usd_vide, Decimal::ZERO, Some(Decimal::ZERO))]
    );
}
