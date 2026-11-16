//! Story 37-5 — réévaluation de fin d'exercice des comptes en devise, au dépôt
//! (`repositories::fx_revaluation`), contrôle K9 (`closing_checks`) et refus de
//! la clôture (`fiscal_years::close`, étape (d-bis)).
//!
//! Le décor : la société du seed passée en multi-devises, son exercice ramené à
//! 2024 (01.01–31.12), des comptes de gains (6950) et de pertes (6940) posés
//! sur la société, des comptes USD et EUR, des cours retenus posés par SQL dans
//! `exchange_rates` (aucun appel réseau).

use std::collections::BTreeMap;

use chrono::NaiveDate;
use kesh_core::accounting::fx_revaluation::RevaluationRateKind;
use kesh_db::entities::{
    Acteur, FiscalYear, ForeignAmount, Journal, NewJournalEntry, NewJournalEntryLine, RateKind,
    RateProvenance,
};
use kesh_db::errors::{DbError, FxAccountField, FxAccountReason};
use kesh_db::repositories::closing_checks::{self, CheckCode, CheckDetails, CheckStatus};
use kesh_db::repositories::exchange_rates::RateOrigin;
use kesh_db::repositories::fx_revaluation::{
    self, NotRevaluedReason, PostRequest, PostedEntry, RetainedRate, RevaluationAccountReason,
};
use kesh_db::repositories::{account_balances, fiscal_years, journal_entries};
use kesh_db::test_fixtures::{SeededCompany, chf_line, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

struct Decor {
    s: SeededCompany,
    fy: i64,
    caisse: i64,
    gain: i64,
    perte: i64,
    usd: i64,
    usd_passif: i64,
    eur: i64,
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

/// L'exercice du seed ramené à `debut`–`fin`, multi-devises, genre de clôture
/// `genre` ; la société en multi-devises, avec ses comptes de différence.
async fn decor_sur(pool: &MySqlPool, debut: NaiveDate, fin: NaiveDate, genre: &str) -> Decor {
    let s = seed_accounting_company(pool).await.unwrap();
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2024', start_date = ?, end_date = ?, \
         multi_currency = TRUE, closing_rate_kind = ? WHERE id = ?",
    )
    .bind(debut)
    .bind(fin)
    .bind(genre)
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    let gain = compte(pool, s.company_id, "6950", "Revenue", None).await;
    let perte = compte(pool, s.company_id, "6940", "Expense", None).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap();
    let usd = compte(pool, s.company_id, "1020", "Asset", Some("USD")).await;
    let usd_passif = compte(pool, s.company_id, "2060", "Liability", Some("USD")).await;
    let eur = compte(pool, s.company_id, "1030", "Asset", Some("EUR")).await;
    Decor {
        caisse: s.accounts["1000"],
        fy: s.fiscal_year_id,
        s,
        gain,
        perte,
        usd,
        usd_passif,
        eur,
    }
}

async fn decor(pool: &MySqlPool) -> Decor {
    decor_sur(pool, d(2024, 1, 1), d(2024, 12, 31), "day").await
}

/// Un cours retenu par la société.
async fn cours(
    pool: &MySqlPool,
    company_id: i64,
    devise: &str,
    genre: &str,
    date: NaiveDate,
    taux: Decimal,
    source: &str,
) {
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(devise)
    .bind(genre)
    .bind(date)
    .bind(taux)
    .bind(source)
    .execute(pool)
    .await
    .unwrap();
}

/// Une valeur publiée (source `afc`), pour la date de publication.
async fn publie(pool: &MySqlPool, devise: &str, jour: NaiveDate, taux: Decimal, pub_le: NaiveDate) {
    sqlx::query(
        "INSERT INTO _kesh_fetched_exchange_rates (source, currency, kind, rate_date, \
         rate_per_unit, published_unit, published_rate, source_date, first_fetched_at, \
         fetched_at, value_changed_at) \
         VALUES ('afc', ?, 'day', ?, ?, 1, ?, ?, NOW(3), NOW(3), NOW(3))",
    )
    .bind(devise)
    .bind(jour)
    .bind(taux)
    .bind(taux)
    .bind(pub_le)
    .execute(pool)
    .await
    .unwrap();
}

/// Une écriture à deux lignes : `compte` en devise contre la caisse CHF.
#[allow(clippy::too_many_arguments)]
async fn poser(
    pool: &MySqlPool,
    dec: &Decor,
    fy: i64,
    date: NaiveDate,
    compte: i64,
    au_debit: bool,
    chf: Decimal,
    montant: Decimal,
    taux: Decimal,
) -> i64 {
    let f = Some(ForeignAmount {
        declared_currency: None,
        amount: montant,
        rate: taux,
        source: Some(RateProvenance::Manual),
        rate_derived: false,
    });
    let (devise, contre) = if au_debit {
        (
            NewJournalEntryLine {
                foreign: f,
                ..chf_line(compte, chf, Decimal::ZERO)
            },
            chf_line(dec.caisse, Decimal::ZERO, chf),
        )
    } else {
        (
            NewJournalEntryLine {
                foreign: f,
                ..chf_line(compte, Decimal::ZERO, chf)
            },
            chf_line(dec.caisse, chf, Decimal::ZERO),
        )
    };
    journal_entries::create(
        pool,
        fy,
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
    .expect("écriture posée")
    .entry
    .id
}

fn retenu(taux: Decimal) -> RetainedRate {
    RetainedRate {
        rate: taux,
        kind: RevaluationRateKind::Request,
        source: "manual".into(),
    }
}

fn demande(fy: i64, cours: &[(&str, Decimal)], comptes: &[i64]) -> PostRequest {
    PostRequest {
        fiscal_year_id: fy,
        closing_rate_kind: RateKind::Day,
        rates: cours
            .iter()
            .map(|(c, r)| (c.to_string(), retenu(*r)))
            .collect(),
        account_ids: comptes.to_vec(),
    }
}

async fn passer(
    pool: &MySqlPool,
    dec: &Decor,
    req: &PostRequest,
) -> Result<Vec<PostedEntry>, DbError> {
    fx_revaluation::post(
        pool,
        Acteur::utilisateur(dec.s.admin_user_id),
        dec.s.company_id,
        req,
    )
    .await
}

async fn exercice(pool: &MySqlPool, id: i64) -> FiscalYear {
    fiscal_years::find_by_id(pool, id).await.unwrap().unwrap()
}

/// Le jeu de base : USD 1000 achetés 880.00 (actif), USD 500 empruntés 450.00
/// (passif), EUR 200 achetés 190.00, tous au 15.03.2024.
async fn jeu(pool: &MySqlPool) -> Decor {
    let dec = decor(pool).await;
    poser(
        pool,
        &dec,
        dec.fy,
        d(2024, 3, 15),
        dec.usd,
        true,
        dec!(880.00),
        dec!(1000),
        dec!(0.88),
    )
    .await;
    poser(
        pool,
        &dec,
        dec.fy,
        d(2024, 3, 15),
        dec.usd_passif,
        false,
        dec!(450.00),
        dec!(500),
        dec!(0.90),
    )
    .await;
    poser(
        pool,
        &dec,
        dec.fy,
        d(2024, 3, 15),
        dec.eur,
        true,
        dec!(190.00),
        dec!(200),
        dec!(0.95),
    )
    .await;
    dec
}

async fn solde(pool: &MySqlPool, dec: &Decor, compte: i64, au: NaiveDate) -> (Decimal, Decimal) {
    let mut conn = pool.acquire().await.unwrap();
    let r = account_balances::balances_as_of(&mut conn, dec.s.company_id, au, Some(&[compte]))
        .await
        .unwrap();
    (r[0].1, r[0].2.unwrap())
}

async fn audits(pool: &MySqlPool, action: &str) -> Vec<serde_json::Value> {
    sqlx::query_scalar::<_, Option<serde_json::Value>>(
        "SELECT details_json FROM audit_log WHERE action = ? ORDER BY id",
    )
    .bind(action)
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|v| v.unwrap_or_default())
    .collect()
}

// ---------------------------------------------------------------------------
// Cours par défaut (AC-1, AC-7)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn cours_par_defaut_jour_puis_mois_puis_rien(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let c = dec.s.company_id;
    cours(
        &pool,
        c,
        "USD",
        "day",
        d(2024, 12, 31),
        dec!(0.91193),
        "afc",
    )
    .await;
    cours(
        &pool,
        c,
        "USD",
        "month",
        d(2024, 12, 1),
        dec!(0.8846),
        "afc",
    )
    .await;
    cours(
        &pool,
        c,
        "EUR",
        "month",
        d(2024, 12, 1),
        dec!(0.9470),
        "afc",
    )
    .await;
    publie(
        &pool,
        "USD",
        d(2024, 12, 31),
        dec!(0.91193),
        d(2024, 12, 31),
    )
    .await;
    let fy = exercice(&pool, dec.fy).await;
    let devises = ["USD", "EUR", "JPY"]
        .map(String::from)
        .into_iter()
        .collect();
    let mut conn = pool.acquire().await.unwrap();
    let r = fx_revaluation::resolve_default_rates(&mut conn, c, &fy, &devises)
        .await
        .unwrap();
    let usd = r["USD"].clone().unwrap();
    assert_eq!(
        (usd.rate, usd.kind, usd.rate_date),
        (dec!(0.91193), RateKind::Day, d(2024, 12, 31))
    );
    assert_eq!(usd.source, RateOrigin::Afc);
    assert_eq!(usd.source_date, Some(d(2024, 12, 31)));
    let eur = r["EUR"].clone().unwrap();
    assert_eq!(
        (eur.rate, eur.kind, eur.rate_date),
        (dec!(0.9470), RateKind::Month, d(2024, 12, 1))
    );
    assert_eq!(
        eur.source_date, None,
        "pas de date de publication hors cours journalier"
    );
    assert!(r["JPY"].is_none(), "ni jour ni mois : RATE_UNAVAILABLE");

    // Genre `month` : le mois seul, même si le jour est présent.
    sqlx::query("UPDATE fiscal_years SET closing_rate_kind = 'month' WHERE id = ?")
        .bind(dec.fy)
        .execute(&pool)
        .await
        .unwrap();
    let fy = exercice(&pool, dec.fy).await;
    let r = fx_revaluation::resolve_default_rates(&mut conn, c, &fy, &devises)
        .await
        .unwrap();
    assert_eq!(r["USD"].as_ref().unwrap().kind, RateKind::Month);
    assert_eq!(r["USD"].as_ref().unwrap().rate, dec!(0.8846));
}

/// Un cours saisi (`manual`) pour le jour est le cours du jour ; un cours saisi
/// pour le mois ne l'emporte pas sur un cours journalier présent ; une source
/// inconnue (A30, `ecb`) se rend telle quelle, sans date de publication.
#[sqlx::test(migrations = "./test-schema")]
async fn cours_saisi_et_source_inconnue(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let c = dec.s.company_id;
    cours(
        &pool,
        c,
        "USD",
        "day",
        d(2024, 12, 31),
        dec!(0.91193),
        "manual",
    )
    .await;
    publie(
        &pool,
        "USD",
        d(2024, 12, 31),
        dec!(0.91193),
        d(2024, 12, 31),
    )
    .await;
    cours(
        &pool,
        c,
        "EUR",
        "day",
        d(2024, 12, 31),
        dec!(0.94947),
        "afc",
    )
    .await;
    cours(
        &pool,
        c,
        "EUR",
        "month",
        d(2024, 12, 1),
        dec!(0.99),
        "manual",
    )
    .await;
    cours(&pool, c, "GBP", "month", d(2024, 12, 1), dec!(1.12), "ecb").await;
    let fy = exercice(&pool, dec.fy).await;
    let devises = ["USD", "EUR", "GBP"]
        .map(String::from)
        .into_iter()
        .collect();
    let mut conn = pool.acquire().await.unwrap();
    let r = fx_revaluation::resolve_default_rates(&mut conn, c, &fy, &devises)
        .await
        .unwrap();
    let usd = r["USD"].clone().unwrap();
    assert_eq!(
        (usd.kind, usd.source.clone()),
        (RateKind::Day, RateOrigin::Manual)
    );
    assert_eq!(
        usd.source_date, None,
        "cours saisi : pas de date de publication"
    );
    let eur = r["EUR"].clone().unwrap();
    assert_eq!((eur.rate, eur.kind), (dec!(0.94947), RateKind::Day));
    let gbp = r["GBP"].clone().unwrap();
    assert_eq!(gbp.source, RateOrigin::Other("ecb".into()));
    assert_eq!(gbp.source.as_str(), "ecb");
    assert_eq!(gbp.source_date, None);
}

/// Durée et fin d'exercice quelconques (S4, T2) : la date de fin et son mois.
#[sqlx::test(migrations = "./test-schema")]
async fn fin_d_exercice_quelconque(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let c = dec.s.company_id;
    // 30.06.2024 (un dimanche) : cours du jour = publication du vendredi 28.06.
    cours(&pool, c, "USD", "day", d(2024, 6, 30), dec!(0.8990), "afc").await;
    publie(&pool, "USD", d(2024, 6, 30), dec!(0.8990), d(2024, 6, 28)).await;
    cours(&pool, c, "USD", "month", d(2024, 6, 1), dec!(0.8900), "afc").await;
    cours(&pool, c, "USD", "month", d(2025, 3, 1), dec!(0.8800), "afc").await;
    cours(&pool, c, "USD", "month", d(2024, 3, 1), dec!(0.8700), "afc").await;
    let devises = ["USD"].map(String::from).into_iter().collect();
    let mut conn = pool.acquire().await.unwrap();
    let cas = [
        // (début, fin, genre, cours, genre attendu, date de publication)
        (
            d(2023, 7, 1),
            d(2024, 6, 30),
            "day",
            dec!(0.8990),
            RateKind::Day,
            Some(d(2024, 6, 28)),
        ),
        (
            d(2023, 7, 1),
            d(2024, 6, 30),
            "month",
            dec!(0.8900),
            RateKind::Month,
            None,
        ),
        // Exercice qui finit un 14 : pas de jour → repli sur mars 2025.
        (
            d(2024, 3, 15),
            d(2025, 3, 14),
            "day",
            dec!(0.8800),
            RateKind::Month,
            None,
        ),
        // 31.03.2024, dimanche de Pâques, genre `month` : mars 2024.
        (
            d(2023, 4, 1),
            d(2024, 3, 31),
            "month",
            dec!(0.8700),
            RateKind::Month,
            None,
        ),
    ];
    for (debut, fin, genre, taux, attendu, pub_le) in cas {
        sqlx::query(
            "UPDATE fiscal_years SET start_date = ?, end_date = ?, closing_rate_kind = ? WHERE id = ?",
        )
        .bind(debut)
        .bind(fin)
        .bind(genre)
        .bind(dec.fy)
        .execute(&pool)
        .await
        .unwrap();
        let fy = exercice(&pool, dec.fy).await;
        let r = fx_revaluation::resolve_default_rates(&mut conn, c, &fy, &devises)
            .await
            .unwrap();
        let usd = r["USD"].clone().unwrap();
        assert_eq!(
            (usd.rate, usd.kind, usd.source_date),
            (taux, attendu, pub_le),
            "{fin} {genre}"
        );
    }
}

// ---------------------------------------------------------------------------
// Passation (AC-2, AC-3)
// ---------------------------------------------------------------------------

/// R9 : trois comptes à écart non nul → trois écritures de deux lignes, une
/// devise chacune ; un compte à écart nul → pas d'écriture ; le solde CHF au
/// 31.12 vaut round(solde × cours) exactement ; l'audit liste les trois.
#[sqlx::test(migrations = "./test-schema")]
async fn une_ecriture_par_compte(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    // Un quatrième compte, à écart nul au cours choisi.
    let usd2 = compte(&pool, dec.s.company_id, "1025", "Asset", Some("USD")).await;
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 4, 1),
        usd2,
        true,
        dec!(91.19),
        dec!(100),
        dec!(0.9119),
    )
    .await;
    let req = demande(
        dec.fy,
        &[("USD", dec!(0.9119)), ("EUR", dec!(0.94947))],
        &[dec.usd, dec.usd_passif, dec.eur, usd2],
    );
    let posted = passer(&pool, &dec, &req).await.unwrap();
    assert_eq!(posted.len(), 3);
    // Dans l'ordre des numéros : 1020, 1030, 2060.
    assert_eq!(
        posted.iter().map(|p| p.account_id).collect::<Vec<_>>(),
        vec![dec.usd, dec.eur, dec.usd_passif]
    );
    // USD actif : 1000 × 0.9119 = 911.90 − 880 = +31.90 (gain).
    assert_eq!(posted[0].delta, dec!(31.90));
    assert_eq!(posted[0].counterpart_account_id, dec.gain);
    // EUR : 200 × 0.94947 = 189.89 − 190 = −0.11 (perte).
    assert_eq!(posted[1].delta, dec!(-0.11));
    assert_eq!(posted[1].counterpart_account_id, dec.perte);
    // USD passif : −500 × 0.9119 = −455.95 − (−450) = −5.95 (perte).
    assert_eq!(posted[2].delta, dec!(-5.95));
    assert_eq!(posted[2].counterpart_account_id, dec.perte);

    for p in &posted {
        let e = journal_entries::find_by_id(&pool, dec.s.company_id, p.entry_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(e.entry.entry_date, d(2024, 12, 31));
        assert_eq!(e.lines.len(), 2);
        let ligne = e
            .lines
            .iter()
            .find(|l| l.account_id == p.account_id)
            .unwrap();
        let f = ligne.foreign().expect("ligne en devise");
        assert_eq!(f.amount, Decimal::ZERO);
        assert_eq!(f.source, RateProvenance::Revaluation);
        assert_eq!(f.rate.normalize(), p.rate.normalize());
        // T3 : la devise relue est celle du compte (jointure), USD ou EUR.
        assert_eq!(f.currency, p.currency);
        assert_eq!(ligne.debit + ligne.credit, p.delta.abs());
        let cp = e
            .lines
            .iter()
            .find(|l| l.account_id == p.counterpart_account_id)
            .unwrap();
        assert!(cp.foreign().is_none());
        let prefixe = if p.delta > Decimal::ZERO {
            "Gain de change "
        } else {
            "Perte de change "
        };
        assert!(
            e.entry.description.starts_with(prefixe),
            "{}",
            e.entry.description
        );
        assert!(e.entry.description.ends_with("au 31.12.2024 (non réalisé)"));
    }
    // Solde CHF = round(solde devise × cours), exactement.
    assert_eq!(
        solde(&pool, &dec, dec.usd, d(2024, 12, 31)).await,
        (dec!(911.90), dec!(1000))
    );
    assert_eq!(
        solde(&pool, &dec, dec.eur, d(2024, 12, 31)).await,
        (dec!(189.89), dec!(200))
    );
    assert_eq!(
        solde(&pool, &dec, dec.usd_passif, d(2024, 12, 31)).await,
        (dec!(-455.95), dec!(-500))
    );

    let a = audits(&pool, "fx_revaluation.posted").await;
    assert_eq!(a.len(), 1);
    let ids: Vec<i64> = a[0]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["entryId"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, posted.iter().map(|p| p.entry_id).collect::<Vec<_>>());
    assert_eq!(a[0]["sources"]["USD"], "manual");
    assert_eq!(a[0]["rateKinds"]["USD"], "request");
    assert_eq!(a[0]["closingRateKind"], "day");
    assert_eq!(a[0]["entries"][0]["fxAccountsOrigin"], "company");

    // Rejouable : un second passage sans mouvement ne passe rien, sans audit.
    assert!(passer(&pool, &dec, &req).await.unwrap().is_empty());
    assert_eq!(audits(&pool, "fx_revaluation.posted").await.len(), 1);
}

/// AC-3 : après une écriture tardive, seul le résidu du compte touché ; les
/// réévaluations déjà passées de l'exercice (A39) ; une réévaluation d'un
/// autre exercice n'y entre pas.
#[sqlx::test(migrations = "./test-schema")]
async fn residu_apres_ecriture_tardive(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let req = demande(
        dec.fy,
        &[("USD", dec!(0.92)), ("EUR", dec!(0.95))],
        &[dec.usd, dec.eur],
    );
    let premier = passer(&pool, &dec, &req).await.unwrap();
    assert_eq!(premier.len(), 1, "EUR à 0.95 : écart nul");
    let d1 = premier[0].delta;
    assert_eq!(d1, dec!(40.00));
    // Écriture tardive : USD 100 de plus, à 0.90.
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 12, 20),
        dec.usd,
        true,
        dec!(90.00),
        dec!(100),
        dec!(0.90),
    )
    .await;
    let fy = exercice(&pool, dec.fy).await;
    let mut conn = pool.acquire().await.unwrap();
    let prev = fx_revaluation::previous_revaluations(
        &mut conn,
        dec.s.company_id,
        &fy,
        &[dec.usd, dec.eur],
    )
    .await
    .unwrap();
    assert_eq!(prev[&dec.usd].total, d1);
    assert_eq!(prev[&dec.usd].entry_ids, vec![premier[0].entry_id]);
    assert!(!prev.contains_key(&dec.eur));
    drop(conn);
    let second = passer(&pool, &dec, &req).await.unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].account_id, dec.usd);
    // 1100 × 0.92 = 1012 ; 880 + 40 + 90 = 1010 → +2.
    assert_eq!(second[0].delta, dec!(2.00));
    assert_ne!(
        second[0].entry_id, premier[0].entry_id,
        "une nouvelle écriture, rien de modifié"
    );
    let fy = exercice(&pool, dec.fy).await;
    let mut conn = pool.acquire().await.unwrap();
    let prev = fx_revaluation::previous_revaluations(&mut conn, dec.s.company_id, &fy, &[dec.usd])
        .await
        .unwrap();
    assert_eq!(prev[&dec.usd].total, dec!(42.00));
    assert_eq!(prev[&dec.usd].entry_ids.len(), 2);
    // Une réévaluation datée hors de l'exercice (un exercice qui finirait le
    // 30.12) n'y entre pas : seules comptent les lignes datées dans l'exercice.
    let mut autre = fy.clone();
    autre.end_date = d(2024, 12, 30);
    let prev =
        fx_revaluation::previous_revaluations(&mut conn, dec.s.company_id, &autre, &[dec.usd])
            .await
            .unwrap();
    assert!(!prev.contains_key(&dec.usd));
}

/// AC-3 : une réévaluation supprimée se repasse ; contre-passée hors de
/// l'exercice, elle reste au cumul ; contre-passée le jour même dans un
/// exercice en cours, le nouveau passage repasse l'écart.
#[sqlx::test(migrations = "./test-schema")]
async fn suppression_et_contre_passation(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let acteur = Acteur::utilisateur(dec.s.admin_user_id);
    let req = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd]);
    let p = passer(&pool, &dec, &req).await.unwrap();
    journal_entries::delete_by_id(&pool, dec.s.company_id, p[0].entry_id, acteur)
        .await
        .unwrap();
    let p2 = passer(&pool, &dec, &req).await.unwrap();
    assert_eq!(p2.len(), 1, "la suppression rouvre l'écart");
    assert_eq!(p2[0].delta, dec!(40.00));

    // Contre-passée le jour même, dans l'exercice en cours (31.12.2024) :
    // l'inverse tombe dans le cumul, le nouveau passage repasse l'écart.
    journal_entries::reverse(
        &pool,
        dec.s.company_id,
        p2[0].entry_id,
        acteur,
        d(2024, 12, 31),
    )
    .await
    .unwrap();
    let p3 = passer(&pool, &dec, &req).await.unwrap();
    assert_eq!(p3.len(), 1);
    assert_eq!(p3[0].delta, dec!(40.00));

    // Contre-passée hors de l'exercice (exercice 2025, jour du 15.01.2025) : le
    // cumul arrêté au 31.12.2024 compte encore l'original, rien ne se repasse.
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Exercice 2025', '2025-01-01', '2025-12-31', 'Open', TRUE)",
    )
    .bind(dec.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    journal_entries::reverse(
        &pool,
        dec.s.company_id,
        p3[0].entry_id,
        acteur,
        d(2025, 1, 15),
    )
    .await
    .unwrap();
    assert!(passer(&pool, &dec, &req).await.unwrap().is_empty());
}

/// S2 : contrepartie sur les comptes propres du compte réévalué ; ceux de la
/// société pour les autres ; un compte propre archivé refuse avec
/// `holderAccountId`, rien de passé ; décoché, la passation passe.
#[sqlx::test(migrations = "./test-schema")]
async fn comptes_de_difference_propres(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let g = compte(&pool, dec.s.company_id, "6841", "Revenue", None).await;
    let p = compte(&pool, dec.s.company_id, "6842", "Expense", None).await;
    sqlx::query("UPDATE accounts SET fx_gain_account_id = ?, fx_loss_account_id = ? WHERE id = ?")
        .bind(g)
        .bind(p)
        .bind(dec.usd)
        .execute(&pool)
        .await
        .unwrap();
    let req = demande(
        dec.fy,
        &[("USD", dec!(0.92)), ("EUR", dec!(0.90))],
        &[dec.usd, dec.eur],
    );

    // Compte propre archivé (état hérité, posé par SQL) : refus, rien de passé.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    match passer(&pool, &dec, &req).await {
        Err(DbError::FxAccountInvalid {
            field: FxAccountField::Loss,
            reason: FxAccountReason::Archived,
            holder_account_id: Some(h),
        }) => assert_eq!(h, dec.usd),
        other => panic!("{other:?}"),
    }
    assert!(audits(&pool, "fx_revaluation.posted").await.is_empty());
    // Décoché : la passation passe (ses comptes propres ne sont plus employés).
    let seul_eur = demande(dec.fy, &[("EUR", dec!(0.90))], &[dec.eur]);
    assert_eq!(passer(&pool, &dec, &seul_eur).await.unwrap().len(), 1);

    sqlx::query("UPDATE accounts SET active = TRUE WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    let posted = passer(&pool, &dec, &req).await.unwrap();
    let usd = posted.iter().find(|e| e.account_id == dec.usd).unwrap();
    assert_eq!(usd.counterpart_account_id, g);
    assert_eq!(serde_json::to_value(usd.fx_accounts_origin).unwrap(), "own");

    // Compte de la société invalide alors que tous les comptes sélectionnés ont
    // des comptes propres : la passation passe.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(dec.gain)
        .execute(&pool)
        .await
        .unwrap();
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 12, 1),
        dec.usd,
        true,
        dec!(10.00),
        dec!(10),
        dec!(1),
    )
    .await;
    let seul_usd = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd]);
    assert_eq!(passer(&pool, &dec, &seul_usd).await.unwrap().len(), 1);
}

/// Ni comptes propres ni comptes de la société : `reason: missing`, même à
/// écart nul, `field` des gains, `holderAccountId` = le compte.
#[sqlx::test(migrations = "./test-schema")]
async fn sans_comptes_de_difference(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    sqlx::query(
        "UPDATE companies SET fx_gain_account_id = NULL, fx_loss_account_id = NULL WHERE id = ?",
    )
    .bind(dec.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let req = demande(dec.fy, &[("EUR", dec!(0.95))], &[dec.eur]);
    match passer(&pool, &dec, &req).await {
        Err(DbError::FxAccountInvalid {
            field: FxAccountField::Gain,
            reason: FxAccountReason::Missing,
            holder_account_id: Some(h),
        }) => assert_eq!(h, dec.eur),
        other => panic!("{other:?}"),
    }
}

/// AC-4 : l'ordre des refus sous verrou.
#[sqlx::test(migrations = "./test-schema")]
async fn refus_sous_verrou_dans_l_ordre(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let base = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd]);

    // Comptes non admissibles : CHF, archivé, d'une autre société, inexistant —
    // tous `notEligible`, réponses identiques pour les deux derniers.
    let archive = compte(&pool, dec.s.company_id, "1040", "Asset", Some("USD")).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    let autre = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre société', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let etranger = compte(&pool, autre, "1020", "Asset", Some("USD")).await;
    for (id, quoi) in [
        (dec.caisse, "CHF"),
        (archive, "archivé"),
        (etranger, "autre société"),
        (999_999, "inexistant"),
    ] {
        let req = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd, id]);
        match passer(&pool, &dec, &req).await {
            Err(DbError::FxRevaluationAccountInvalid {
                reason: RevaluationAccountReason::NotEligible,
                account_ids,
            }) => assert_eq!(account_ids, vec![id], "{quoi}"),
            other => panic!("{quoi} : {other:?}"),
        }
    }

    // Genre de clôture changé depuis la lecture : 409.
    let mut changee = base.clone();
    changee.closing_rate_kind = RateKind::Month;
    assert!(matches!(
        passer(&pool, &dec, &changee).await,
        Err(DbError::OptimisticLockConflict)
    ));

    // Une devise sélectionnée sans cours résolu : 404 RATE_UNAVAILABLE (forme mois).
    let sans = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd, dec.eur]);
    match passer(&pool, &dec, &sans).await {
        Err(DbError::RateUnavailable { currency, month }) => {
            assert_eq!((currency.as_str(), month), ("EUR", d(2024, 12, 1)));
        }
        other => panic!("{other:?}"),
    }

    // Livres verrouillés exactement à la date de fin (borne inclusive).
    sqlx::query("UPDATE companies SET books_locked_through = '2024-12-31' WHERE id = ?")
        .bind(dec.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        passer(&pool, &dec, &base).await,
        Err(DbError::PeriodLocked { .. })
    ));
    sqlx::query("UPDATE companies SET books_locked_through = '2024-12-30' WHERE id = ?")
        .bind(dec.s.company_id)
        .execute(&pool)
        .await
        .unwrap();

    // Exercice postérieur clos (état hérité, posé par SQL).
    let posterieur: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Exercice 2025', '2025-01-01', '2025-12-31', 'Closed', TRUE) RETURNING id",
    )
    .bind(dec.s.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(matches!(
        passer(&pool, &dec, &base).await,
        Err(DbError::LaterFiscalYearClosed { .. })
    ));
    sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
        .bind(posterieur)
        .execute(&pool)
        .await
        .unwrap();

    // Exercice antérieur ouvert : il parle avant les comptes et le genre.
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Exercice 2023', '2023-01-01', '2023-12-31', 'Open', TRUE)",
    )
    .bind(dec.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    assert!(matches!(
        passer(&pool, &dec, &changee).await,
        Err(DbError::EarlierFiscalYearOpen { .. })
    ));

    // Exercice clos : il parle d'abord.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(dec.fy)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        passer(&pool, &dec, &changee).await,
        Err(DbError::FiscalYearClosed)
    ));
    assert!(audits(&pool, "fx_revaluation.posted").await.is_empty());
}

/// AC-4 : produit devise × cours hors bornes → `FX_VALUE_OUT_OF_RANGE`, sans
/// écriture.
#[sqlx::test(migrations = "./test-schema")]
async fn calcul_hors_bornes(pool: MySqlPool) {
    let dec = decor(&pool).await;
    // 9·10¹² USD à 1 : dans les bornes ; réévalués à 1000 → 9·10¹⁵ CHF.
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 3, 1),
        dec.usd,
        true,
        dec!(9000000000000),
        dec!(9000000000000),
        dec!(1),
    )
    .await;
    let req = demande(dec.fy, &[("USD", dec!(1000))], &[dec.usd]);
    match passer(&pool, &dec, &req).await {
        Err(DbError::FxRevaluationValueOutOfRange {
            currency,
            account_id,
        }) => {
            assert_eq!((currency.as_str(), account_id), ("USD", dec.usd));
        }
        other => panic!("{other:?}"),
    }
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE rate_source = 'revaluation'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 0);
}

/// Libellé d'un nom de compte maximal (255 caractères) : sous 500 caractères,
/// passé tel quel à l'écriture.
#[sqlx::test(migrations = "./test-schema")]
async fn libelle_avec_nom_maximal(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    sqlx::query("UPDATE accounts SET name = ? WHERE id = ?")
        .bind("ü".repeat(255))
        .bind(dec.usd)
        .execute(&pool)
        .await
        .unwrap();
    let p = passer(
        &pool,
        &dec,
        &demande(dec.fy, &[("USD", dec!(0.9123456789))], &[dec.usd]),
    )
    .await
    .unwrap();
    let e = journal_entries::find_by_id(&pool, dec.s.company_id, p[0].entry_id)
        .await
        .unwrap()
        .unwrap();
    assert!(e.entry.description.chars().count() <= 500);
    assert!(
        e.entry
            .description
            .contains("USD 0.9123456789 au 31.12.2024")
    );
}

/// Atomicité (R9) : une erreur survenue après la deuxième écriture annule les
/// trois. Mécanisme : un déclencheur `BEFORE INSERT` sur les lignes qui refuse
/// la ligne du troisième compte réévalué (SIGNAL), posé dans la base éphémère
/// du test.
#[sqlx::test(migrations = "./test-schema")]
async fn atomicite_toutes_ou_aucune(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    sqlx::raw_sql(&format!(
        "CREATE TRIGGER refuse_troisieme BEFORE INSERT ON journal_entry_lines FOR EACH ROW \
         BEGIN IF NEW.account_id = {} AND NEW.rate_source = 'revaluation' THEN \
         SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'refus du test'; END IF; END",
        dec.usd_passif
    ))
    .execute(&pool)
    .await
    .expect("déclencheur posé");
    let req = demande(
        dec.fy,
        &[("USD", dec!(0.92)), ("EUR", dec!(0.90))],
        &[dec.usd, dec.eur, dec.usd_passif],
    );
    assert!(passer(&pool, &dec, &req).await.is_err());
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE rate_source = 'revaluation'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 0, "aucune des trois écritures n'existe après le refus");
    assert!(audits(&pool, "fx_revaluation.posted").await.is_empty());
}

/// Deux passations concurrentes : la seconde attend le verrou de l'exercice
/// pendant que la première commite, relit les soldes après les verrous et ne
/// passe rien.
#[sqlx::test(migrations = "./test-schema")]
async fn deux_passations_concurrentes(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let req = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd]);
    // Un tiers tient l'exercice ; les deux passations s'y arrêtent.
    let mut tiers = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(dec.fy)
        .execute(&mut *tiers)
        .await
        .unwrap();
    let lancer = || {
        let pool = pool.clone();
        let req = req.clone();
        let (c, u) = (dec.s.company_id, dec.s.admin_user_id);
        tokio::spawn(
            async move { fx_revaluation::post(&pool, Acteur::utilisateur(u), c, &req).await },
        )
    };
    let a = lancer();
    let b = lancer();
    assert!(
        kesh_db::test_fixtures::attendre_des_requetes_en_cours(
            &pool,
            &["SELECT id, company_id, name", "FOR UPDATE"],
            2,
            || a.is_finished() || b.is_finished()
        )
        .await,
        "les deux passations attendent l'exercice"
    );
    tiers.commit().await.unwrap();
    let ra = a.await.unwrap().unwrap();
    let rb = b.await.unwrap().unwrap();
    assert_eq!(ra.len() + rb.len(), 1, "une seule écriture : {ra:?} {rb:?}");
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE rate_source = 'revaluation'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
}

/// Une désignation propre concurrente (qui prend `companies` en exclusif)
/// attend la fin de la passation, qui tient `companies` en partage.
#[sqlx::test(migrations = "./test-schema")]
async fn designation_concurrente_attend_la_passation(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    // La passation s'arrête sur l'exercice, `companies` déjà tenu en partage.
    let mut tiers = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(dec.fy)
        .execute(&mut *tiers)
        .await
        .unwrap();
    let req = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd]);
    let (pool2, c, u) = (pool.clone(), dec.s.company_id, dec.s.admin_user_id);
    let passation =
        tokio::spawn(
            async move { fx_revaluation::post(&pool2, Acteur::utilisateur(u), c, &req).await },
        );
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["SELECT id, company_id, name", "FOR UPDATE"],
            || passation.is_finished()
        )
        .await
    );
    // Le geste exclusif sur `companies` attend la passation.
    let pool3 = pool.clone();
    let exclusif = tokio::spawn(async move {
        let mut tx = pool3.begin().await.unwrap();
        sqlx::query("SELECT id FROM companies WHERE id = ? FOR UPDATE")
            .bind(c)
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    });
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["SELECT id FROM companies", "FOR UPDATE"],
            || exclusif.is_finished()
        )
        .await,
        "le geste exclusif attend le partage de la passation"
    );
    tiers.commit().await.unwrap();
    assert_eq!(passation.await.unwrap().unwrap().len(), 1);
    exclusif.await.unwrap();
}

// ---------------------------------------------------------------------------
// K9 (AC-9)
// ---------------------------------------------------------------------------

async fn k9(pool: &MySqlPool, dec: &Decor) -> closing_checks::ClosingCheck {
    let mut conn = pool.acquire().await.unwrap();
    let c = closing_checks::for_fiscal_year(&mut conn, dec.s.company_id, dec.fy)
        .await
        .unwrap();
    assert_eq!(c.checks.len(), 9, "neuf contrôles");
    assert_eq!(
        c.checks[7].code,
        CheckCode::DraftSupplierInvoices,
        "K8 reste en 8e"
    );
    c.checks[8].clone()
}

fn k9_comptes(k: &closing_checks::ClosingCheck) -> Vec<(i64, NotRevaluedReason, Option<Decimal>)> {
    match &k.details {
        CheckDetails::ForeignAccountsNotRevalued(d) => d
            .accounts
            .iter()
            .map(|a| (a.account_id, a.reason, a.delta))
            .collect(),
        other => panic!("{other:?}"),
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn k9_sans_compte_en_devise(pool: MySqlPool) {
    // Multi-devises sans compte en devise, sans aucun cours : ok.
    let s = seed_accounting_company(&pool).await.unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let c = closing_checks::for_fiscal_year(&mut conn, s.company_id, s.fiscal_year_id)
        .await
        .unwrap();
    assert_eq!(c.checks[8].code, CheckCode::ForeignAccountsNotRevalued);
    assert_eq!(c.checks[8].status, CheckStatus::Ok);
}

/// Mono-devise avec un compte en devise à solde (exercice clos avant
/// l'activation puis rouvert) : ok sans autre lecture.
#[sqlx::test(migrations = "./test-schema")]
async fn k9_mono_devise(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    sqlx::query("UPDATE fiscal_years SET multi_currency = FALSE WHERE id = ?")
        .bind(dec.fy)
        .execute(&pool)
        .await
        .unwrap();
    let k = k9(&pool, &dec).await;
    assert_eq!((k.status, k.count), (CheckStatus::Ok, 0));
}

#[sqlx::test(migrations = "./test-schema")]
async fn k9_statuts(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let c = dec.s.company_id;
    // Sans aucun cours : les trois comptes à solde → RATE_UNAVAILABLE.
    let k = k9(&pool, &dec).await;
    assert_eq!((k.status, k.count), (CheckStatus::Attention, 3));
    assert!(
        k9_comptes(&k)
            .iter()
            .all(|(_, r, d)| *r == NotRevaluedReason::RateUnavailable && d.is_none())
    );
    // Un compte en devise sans solde et sans cours ne compte pas.
    compte(&pool, c, "1050", "Asset", Some("GBP")).await;
    assert_eq!(k9(&pool, &dec).await.count, 3);

    cours(&pool, c, "USD", "day", d(2024, 12, 31), dec!(0.92), "afc").await;
    cours(&pool, c, "EUR", "day", d(2024, 12, 31), dec!(0.95), "afc").await;
    let k = k9(&pool, &dec).await;
    // EUR à 0.95 : écart nul ; USD actif +40, USD passif −10.
    assert_eq!(
        k9_comptes(&k),
        vec![
            (dec.usd, NotRevaluedReason::NotRevalued, Some(dec!(40.00))),
            (
                dec.usd_passif,
                NotRevaluedReason::NotRevalued,
                Some(dec!(-10.00))
            ),
        ]
    );
    // Passation à un cours `request` : K9 reste en attention au cours par défaut.
    let p = demande(dec.fy, &[("USD", dec!(0.91))], &[dec.usd, dec.usd_passif]);
    passer(&pool, &dec, &p).await.unwrap();
    assert_eq!(k9(&pool, &dec).await.status, CheckStatus::Attention);
    // Passation au cours par défaut : ok.
    let p = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd, dec.usd_passif]);
    passer(&pool, &dec, &p).await.unwrap();
    let k = k9(&pool, &dec).await;
    assert_eq!((k.status, k.count), (CheckStatus::Ok, 0));
    // Écriture tardive : attention.
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 12, 30),
        dec.usd,
        true,
        dec!(90.00),
        dec!(100),
        dec!(0.90),
    )
    .await;
    assert_eq!(
        k9_comptes(&k9(&pool, &dec).await),
        vec![(dec.usd, NotRevaluedReason::NotRevalued, Some(dec!(2.00)))]
    );
    // Compte décoché à la passation, écart restant : attention (A37).
    let seul_passif = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd_passif]);
    assert!(passer(&pool, &dec, &seul_passif).await.unwrap().is_empty());
    assert_eq!(k9(&pool, &dec).await.status, CheckStatus::Attention);
    // Compte archivé à écart non nul : attention.
    passer(
        &pool,
        &dec,
        &demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd]),
    )
    .await
    .unwrap();
    assert_eq!(k9(&pool, &dec).await.status, CheckStatus::Ok);
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 12, 30),
        dec.eur,
        true,
        dec!(10.00),
        dec!(10),
        dec!(1),
    )
    .await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(dec.eur)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        k9_comptes(&k9(&pool, &dec).await),
        vec![(dec.eur, NotRevaluedReason::NotRevalued, Some(dec!(-0.50)))]
    );
}

/// Un calcul hors bornes ne fait pas perdre les huit autres contrôles : K9 en
/// attention, `delta` nul, motif `NOT_REVALUED`.
#[sqlx::test(migrations = "./test-schema")]
async fn k9_hors_bornes(pool: MySqlPool) {
    let dec = decor(&pool).await;
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 3, 1),
        dec.usd,
        true,
        dec!(9000000000000),
        dec!(9000000000000),
        dec!(1),
    )
    .await;
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        "day",
        d(2024, 12, 31),
        dec!(1000),
        "manual",
    )
    .await;
    let k = k9(&pool, &dec).await;
    assert_eq!(
        k9_comptes(&k),
        vec![(dec.usd, NotRevaluedReason::NotRevalued, None)]
    );
}

/// Recette de Beat (CR8-P1-3) : livres importés, passation à 0.91193 ; le
/// cours journalier du 31.12.2024 dans `exchange_rates` → K9 ok sans saisie ;
/// sans lui (repli mensuel 0.8846) → attention.
#[sqlx::test(migrations = "./test-schema")]
async fn k9_recette_de_beat(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let c = dec.s.company_id;
    poser(
        &pool,
        &dec,
        dec.fy,
        d(2024, 6, 1),
        dec.usd,
        true,
        dec!(870.00),
        dec!(1000),
        dec!(0.87),
    )
    .await;
    cours(
        &pool,
        c,
        "USD",
        "month",
        d(2024, 12, 1),
        dec!(0.8846),
        "afc",
    )
    .await;
    passer(
        &pool,
        &dec,
        &demande(dec.fy, &[("USD", dec!(0.91193))], &[dec.usd]),
    )
    .await
    .unwrap();
    assert_eq!(
        k9(&pool, &dec).await.status,
        CheckStatus::Attention,
        "repli mensuel"
    );
    cours(
        &pool,
        c,
        "USD",
        "day",
        d(2024, 12, 31),
        dec!(0.91193),
        "afc",
    )
    .await;
    assert_eq!(
        k9(&pool, &dec).await.status,
        CheckStatus::Ok,
        "cours du jour récupéré"
    );
}

// ---------------------------------------------------------------------------
// Clôture (AC-9, (d-bis))
// ---------------------------------------------------------------------------

async fn clore(pool: &MySqlPool, dec: &Decor) -> Result<FiscalYear, DbError> {
    fiscal_years::close(pool, dec.s.admin_user_id, dec.s.company_id, dec.fy).await
}

#[sqlx::test(migrations = "./test-schema")]
async fn cloture_refusee_puis_permise(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let c = dec.s.company_id;
    cours(&pool, c, "USD", "day", d(2024, 12, 31), dec!(0.92), "afc").await;
    cours(&pool, c, "EUR", "day", d(2024, 12, 31), dec!(0.95), "afc").await;
    match clore(&pool, &dec).await {
        Err(DbError::ForeignAccountsNotRevalued(found)) => {
            assert_eq!(found.count, 2);
            let json = serde_json::to_value(&found).unwrap();
            assert_eq!(json["accounts"][0]["accountId"], dec.usd);
            assert_eq!(json["accounts"][0]["delta"], "40.00");
            assert_eq!(json["accounts"][0]["reason"], "NOT_REVALUED");
            assert!(json.get("count").is_none());
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        exercice(&pool, dec.fy).await.status,
        kesh_db::entities::FiscalYearStatus::Open
    );
    // Antérieur ouvert ET non réévalué : l'antérieur parle d'abord.
    let anterieur: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Exercice 2023', '2023-01-01', '2023-12-31', 'Open', TRUE) RETURNING id",
    )
    .bind(c)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(matches!(
        clore(&pool, &dec).await,
        Err(DbError::EarlierFiscalYearOpen { .. })
    ));
    sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
        .bind(anterieur)
        .execute(&pool)
        .await
        .unwrap();
    // Après passation au cours par défaut : la clôture passe.
    passer(
        &pool,
        &dec,
        &demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd, dec.usd_passif]),
    )
    .await
    .unwrap();
    clore(&pool, &dec)
        .await
        .expect("clôture permise après réévaluation");
    // Déjà clos.
    assert!(matches!(
        clore(&pool, &dec).await,
        Err(DbError::IllegalStateTransition(_))
    ));
    // La passation sur un exercice clos est refusée.
    assert!(matches!(
        passer(
            &pool,
            &dec,
            &demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd])
        )
        .await,
        Err(DbError::FiscalYearClosed)
    ));
}

/// Mono-devise : la clôture passe sans cours en base, compte en devise ou non.
#[sqlx::test(migrations = "./test-schema")]
async fn cloture_mono_devise_sans_cours(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    sqlx::query("UPDATE fiscal_years SET multi_currency = FALSE WHERE id = ?")
        .bind(dec.fy)
        .execute(&pool)
        .await
        .unwrap();
    clore(&pool, &dec)
        .await
        .expect("mono-devise : aucun cours lu");
}

/// La clôture tenue à son verrou d'exercice : un tiers tient l'exercice, la
/// clôture fait ses lectures d'avant `begin` puis attend ; ce que le tiers
/// commite pendant l'attente est vu.
async fn cloture_pendant<F, Fut>(
    pool: &MySqlPool,
    dec: &Decor,
    pendant: F,
) -> Result<FiscalYear, DbError>
where
    F: FnOnce(sqlx::Transaction<'static, sqlx::MySql>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut tiers = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(dec.fy)
        .execute(&mut *tiers)
        .await
        .unwrap();
    let (p, u, c, id) = (pool.clone(), dec.s.admin_user_id, dec.s.company_id, dec.fy);
    let cloture = tokio::spawn(async move { fiscal_years::close(&p, u, c, id).await });
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            pool,
            &["SELECT id, company_id, name", "FOR UPDATE"],
            || cloture.is_finished()
        )
        .await,
        "la clôture attend l'exercice"
    );
    pendant(tiers).await;
    cloture.await.unwrap()
}

#[sqlx::test(migrations = "./test-schema")]
async fn cloture_genre_change_pendant_l_attente(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let r = cloture_pendant(&pool, &dec, |mut tiers| async move {
        sqlx::query("UPDATE fiscal_years SET closing_rate_kind = 'month' WHERE id = ?")
            .bind(dec.fy)
            .execute(&mut *tiers)
            .await
            .unwrap();
        tiers.commit().await.unwrap();
    })
    .await;
    assert!(matches!(r, Err(DbError::OptimisticLockConflict)), "{r:?}");
}

#[sqlx::test(migrations = "./test-schema")]
async fn cloture_ecriture_tardive_pendant_l_attente(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let c = dec.s.company_id;
    cours(&pool, c, "USD", "day", d(2024, 12, 31), dec!(0.92), "afc").await;
    cours(&pool, c, "EUR", "day", d(2024, 12, 31), dec!(0.95), "afc").await;
    passer(
        &pool,
        &dec,
        &demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd, dec.usd_passif]),
    )
    .await
    .unwrap();
    let (caisse, usd, fy, u) = (dec.caisse, dec.usd, dec.fy, dec.s.admin_user_id);
    let r = cloture_pendant(&pool, &dec, |mut tiers| async move {
        let ligne = NewJournalEntryLine {
            foreign: Some(ForeignAmount {
                declared_currency: None,
                amount: dec!(100),
                rate: dec!(0.90),
                source: Some(RateProvenance::Manual),
                rate_derived: false,
            }),
            ..chf_line(usd, dec!(90.00), Decimal::ZERO)
        };
        journal_entries::create_in_tx(
            &mut tiers,
            fy,
            Acteur::utilisateur(u),
            NewJournalEntry {
                company_id: c,
                entry_date: d(2024, 12, 30),
                journal: Journal::OD,
                description: "Tardive".into(),
                project_id: None,
                lines: vec![ligne, chf_line(caisse, Decimal::ZERO, dec!(90.00))],
            },
            true,
        )
        .await
        .unwrap();
        tiers.commit().await.unwrap();
    })
    .await;
    assert!(
        matches!(r, Err(DbError::ForeignAccountsNotRevalued(_))),
        "{r:?}"
    );
}

/// Un compte en devise créé et mouvementé après la lecture d'avant `begin`
/// (l'`EXISTS` y était vide) : sans cours résolu, il refuse en
/// `RATE_UNAVAILABLE` — le contrôle échoue fermé.
#[sqlx::test(migrations = "./test-schema")]
async fn cloture_compte_en_devise_apparu_pendant_l_attente(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    sqlx::query(
        "UPDATE fiscal_years SET start_date = '2024-01-01', end_date = '2024-12-31', \
         multi_currency = TRUE WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let dec = Decor {
        caisse: s.accounts["1000"],
        fy: s.fiscal_year_id,
        gain: 0,
        perte: 0,
        usd: 0,
        usd_passif: 0,
        eur: 0,
        s,
    };
    let (c, fy, u, caisse) = (dec.s.company_id, dec.fy, dec.s.admin_user_id, dec.caisse);
    let r = cloture_pendant(&pool, &dec, |mut tiers| async move {
        let usd: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (company_id, number, name, account_type, currency) \
             VALUES (?, '1020', 'USD', 'Asset', 'USD') RETURNING id",
        )
        .bind(c)
        .fetch_one(&mut *tiers)
        .await
        .unwrap();
        let ligne = NewJournalEntryLine {
            foreign: Some(ForeignAmount {
                declared_currency: None,
                amount: dec!(100),
                rate: dec!(0.90),
                source: Some(RateProvenance::Manual),
                rate_derived: false,
            }),
            ..chf_line(usd, dec!(90.00), Decimal::ZERO)
        };
        journal_entries::create_in_tx(
            &mut tiers,
            fy,
            Acteur::utilisateur(u),
            NewJournalEntry {
                company_id: c,
                entry_date: d(2024, 6, 1),
                journal: Journal::OD,
                description: "Compte neuf".into(),
                project_id: None,
                lines: vec![ligne, chf_line(caisse, Decimal::ZERO, dec!(90.00))],
            },
            true,
        )
        .await
        .unwrap();
        tiers.commit().await.unwrap();
    })
    .await;
    match r {
        Err(DbError::ForeignAccountsNotRevalued(found)) => {
            assert_eq!(found.accounts[0].reason, NotRevaluedReason::RateUnavailable);
        }
        other => panic!("{other:?}"),
    }
}

/// Clôture et passation concurrentes : la passation commite d'abord, la
/// clôture relit les soldes après ses verrous et passe.
#[sqlx::test(migrations = "./test-schema")]
async fn cloture_apres_passation_concurrente(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let c = dec.s.company_id;
    cours(&pool, c, "USD", "day", d(2024, 12, 31), dec!(0.92), "afc").await;
    cours(&pool, c, "EUR", "day", d(2024, 12, 31), dec!(0.95), "afc").await;
    let req = demande(dec.fy, &[("USD", dec!(0.92))], &[dec.usd, dec.usd_passif]);
    let (pool2, u) = (pool.clone(), dec.s.admin_user_id);
    let r = cloture_pendant(&pool, &dec, |tiers| async move {
        // La passation, lancée pendant l'attente, s'arrête aussi ; le tiers
        // relâche, la passation (premier arrivé sur `companies` puis
        // l'exercice) et la clôture se suivent. On force l'ordre : le tiers
        // passe la réévaluation dans sa propre session après avoir relâché.
        tiers.commit().await.unwrap();
        fx_revaluation::post(&pool2, Acteur::utilisateur(u), c, &req)
            .await
            .unwrap();
    })
    .await;
    // La clôture a pu passer avant ou après : si elle a refusé, c'est K9 ; si
    // elle a passé, la passation a été refusée (exercice clos) — elle ne passe
    // donc qu'à la condition d'avoir vu la réévaluation.
    match r {
        Ok(_) => {
            let n: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM journal_entry_lines WHERE rate_source = 'revaluation'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(n > 0, "une clôture passée a vu la réévaluation");
        }
        Err(DbError::ForeignAccountsNotRevalued(_)) => {}
        Err(other) => panic!("{other:?}"),
    }
}

/// `previous_revaluations` et le décompte de K9 se lisent sur la même liste
/// (DRY) : `lines_for_fiscal_year` sans archivé sans solde.
#[sqlx::test(migrations = "./test-schema")]
async fn lignes_de_l_apercu(pool: MySqlPool) {
    let dec = jeu(&pool).await;
    let archive_vide = compte(&pool, dec.s.company_id, "1060", "Asset", Some("USD")).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(archive_vide)
        .execute(&pool)
        .await
        .unwrap();
    // Un compte de résultat en devise n'est jamais candidat (posé par SQL).
    compte(&pool, dec.s.company_id, "6000", "Expense", Some("USD")).await;
    let fy = exercice(&pool, dec.fy).await;
    let cours = BTreeMap::from([("USD".to_string(), dec!(0.92))]);
    let mut conn = pool.acquire().await.unwrap();
    let lignes = fx_revaluation::lines_for_fiscal_year(&mut conn, dec.s.company_id, &fy, &cours)
        .await
        .unwrap();
    let ids: Vec<i64> = lignes.iter().map(|l| l.account.id).collect();
    assert_eq!(
        ids,
        vec![dec.usd, dec.eur, dec.usd_passif],
        "archivé sans solde et résultat écartés"
    );
    assert_eq!(
        lignes[1].outcome,
        fx_revaluation::LineOutcome::RateUnavailable
    );
}
