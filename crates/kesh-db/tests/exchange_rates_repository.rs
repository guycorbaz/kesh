//! Story 37-2 — dépôt des cours de change (`repositories::exchange_rates`) :
//! règle d'écriture des valeurs publiées (AC-3), copie dans `exchange_rates`
//! (AC-3 bis), cours retenus par la société, notices et acquittement (AC-5),
//! état de la récupération et vidages (AC-1, AC-7).
//!
//! Le montage écrit en SQL brut (sociétés, comptes, exercices) : le dépôt des
//! cours ne lit que ces colonnes, et les gardes des dépôts de 37-1 ne sont pas
//! l'objet de ces tests.

use chrono::{NaiveDate, NaiveDateTime};
use kesh_core::currency::CurrencyCode;
use kesh_core::exchange_rates::{PublishedRate, RateKind, RateUnit};
use kesh_db::errors::DbError;
use kesh_db::repositories::exchange_rates::{
    self as er, FetchOutcomeStatus, FetchStatus, FetchTrigger, NoticeAck, RateOrigin, SOURCE_AFC,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

fn t(h: u32) -> NaiveDateTime {
    d(2024, 11, 20).and_hms_opt(h, 0, 0).unwrap()
}

fn mois(y: i32, m: u32) -> RateUnit {
    RateUnit::month_of(d(y, m, 1))
}

fn publie(code: &str, unite: i32, cours: Decimal) -> PublishedRate {
    PublishedRate {
        currency: CurrencyCode::parse(code).unwrap(),
        published_unit: unite,
        published_rate: cours,
        rate_per_unit: cours / Decimal::from(unite),
    }
}

async fn societe(pool: &MySqlPool, nom: &str, multi: bool) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language, \
         multi_currency_enabled) VALUES (?, 'Rue 1, 1000 Lausanne', 'Pme', 'FR', 'FR', ?) RETURNING id",
    )
    .bind(nom)
    .bind(multi)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn utilisateur(pool: &MySqlPool, company_id: i64, nom: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, company_id) VALUES (?, ?, 'Admin', ?) \
         RETURNING id",
    )
    .bind(nom)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA")
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    devise: Option<&str>,
    actif: bool,
) {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency, active) \
         VALUES (?, ?, ?, 'Asset', ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(devise)
    .bind(actif)
    .execute(pool)
    .await
    .unwrap();
}

#[allow(clippy::too_many_arguments)]
async fn exercice(
    pool: &MySqlPool,
    company_id: i64,
    nom: &str,
    debut: NaiveDate,
    fin: NaiveDate,
    ouvert: bool,
    multi: bool,
    ouverture: &str,
    cloture: &str,
) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency, \
         opening_rate_kind, closing_rate_kind) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(nom)
    .bind(debut)
    .bind(fin)
    .bind(if ouvert { "Open" } else { "Closed" })
    .bind(multi)
    .bind(ouverture)
    .bind(cloture)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Écrit les valeurs publiées d'une unité, dans sa propre transaction.
async fn ecrire(
    pool: &MySqlPool,
    unite: RateUnit,
    source_date: Option<NaiveDate>,
    cours: &[PublishedRate],
    instant: NaiveDateTime,
) -> er::FetchWriteOutcome {
    let mut tx = pool.begin().await.unwrap();
    let o = er::write_fetched_unit_in_tx(&mut tx, SOURCE_AFC, unite, source_date, cours, instant)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    o
}

async fn copier(
    pool: &MySqlPool,
    company_id: i64,
    units: &[RateUnit],
    devise: Option<&str>,
) -> u64 {
    let mut tx = pool.begin().await.unwrap();
    let n = er::copy_fetched_in_tx(&mut tx, company_id, units, devise)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    n
}

async fn retenu(
    pool: &MySqlPool,
    company_id: i64,
    devise: &str,
    u: RateUnit,
) -> Option<er::CompanyRate> {
    let mut conn = pool.acquire().await.unwrap();
    er::find_company_rate_in_tx(&mut conn, company_id, devise, u.kind, u.date)
        .await
        .unwrap()
}

async fn publiee(pool: &MySqlPool, devise: &str, u: RateUnit) -> Option<er::FetchedRate> {
    let mut conn = pool.acquire().await.unwrap();
    er::find_fetched_rate_in_tx(&mut conn, SOURCE_AFC, devise, u.kind, u.date)
        .await
        .unwrap()
}

async fn saisir(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    devise: &str,
    u: RateUnit,
    cours: Decimal,
    version: Option<i32>,
) -> Result<er::RateChange, DbError> {
    let mut tx = pool.begin().await.unwrap();
    let r = er::set_manual_rate(
        &mut tx, company_id, user_id, devise, u.kind, u.date, cours, version,
    )
    .await;
    if r.is_ok() {
        tx.commit().await.unwrap();
    }
    r
}

async fn reprendre(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    devise: &str,
    u: RateUnit,
    version: Option<i32>,
    vue: Option<Decimal>,
) -> Result<er::RateChange, DbError> {
    let mut tx = pool.begin().await.unwrap();
    let r = er::reset_rate(
        &mut tx, company_id, user_id, devise, u.kind, u.date, version, vue,
    )
    .await;
    if r.is_ok() {
        tx.commit().await.unwrap();
    }
    r
}

async fn lignes_de(
    pool: &MySqlPool,
    company_id: i64,
) -> Vec<(String, String, NaiveDate, Decimal, String)> {
    sqlx::query_as(
        "SELECT currency, kind, rate_date, rate, source FROM exchange_rates WHERE company_id = ? \
         ORDER BY rate_date, kind, currency",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

// ---------------------------------------------------------------------------
// Règle d'écriture des valeurs publiées (AC-3)
// ---------------------------------------------------------------------------

/// (a) insérée, (b) confirmée (seul `fetched_at` avance), (c) corrigée
/// (valeur remplacée, `value_changed_at` avancé, `first_fetched_at`
/// inchangé), puis retour de la source à la valeur d'origine : (c) encore.
#[sqlx::test(migrations = "./test-schema")]
async fn regle_d_ecriture_inseree_confirmee_corrigee(pool: MySqlPool) {
    let dec_24 = mois(2024, 12);
    let o = ecrire(
        &pool,
        dec_24,
        None,
        &[
            publie("USD", 1, dec!(0.8846)),
            publie("JPY", 100, dec!(0.5733)),
        ],
        t(1),
    )
    .await;
    assert_eq!((o.inserted, o.confirmed, o.corrected.len()), (2, 0, 0));
    let usd = publiee(&pool, "USD", dec_24).await.unwrap();
    assert_eq!(
        (usd.first_fetched_at, usd.fetched_at, usd.value_changed_at),
        (t(1), t(1), t(1))
    );
    assert_eq!(
        publiee(&pool, "JPY", dec_24).await.unwrap().rate_per_unit,
        dec!(0.005733)
    );

    // (b) même valeur (0.884600 == 0.8846 en valeur).
    let o = ecrire(
        &pool,
        dec_24,
        None,
        &[publie("USD", 1, dec!(0.884600))],
        t(2),
    )
    .await;
    assert_eq!((o.inserted, o.confirmed, o.corrected.len()), (0, 1, 0));
    let usd = publiee(&pool, "USD", dec_24).await.unwrap();
    assert_eq!(
        (usd.first_fetched_at, usd.fetched_at, usd.value_changed_at),
        (t(1), t(2), t(1))
    );

    // (c) correction.
    let o = ecrire(&pool, dec_24, None, &[publie("USD", 1, dec!(0.8850))], t(3)).await;
    assert_eq!(o.written(), 1);
    assert_eq!(o.corrected.len(), 1);
    let c = &o.corrected[0];
    assert_eq!(
        (c.currency.as_str(), c.old_rate, c.new_rate),
        ("USD", dec!(0.8846), dec!(0.8850))
    );
    let usd = publiee(&pool, "USD", dec_24).await.unwrap();
    assert_eq!(usd.rate_per_unit, dec!(0.885));
    assert_eq!(
        (usd.first_fetched_at, usd.fetched_at, usd.value_changed_at),
        (t(1), t(3), t(3))
    );

    // Retour à la valeur d'origine : une nouvelle correction.
    let o = ecrire(&pool, dec_24, None, &[publie("USD", 1, dec!(0.8846))], t(4)).await;
    assert_eq!(o.corrected.len(), 1);
    let usd = publiee(&pool, "USD", dec_24).await.unwrap();
    assert_eq!(usd.published_rate, dec!(0.8846));
    assert_eq!((usd.first_fetched_at, usd.value_changed_at), (t(1), t(4)));

    // Une unité changée sans changement de cours publié est une correction.
    let o = ecrire(&pool, dec_24, None, &[publie("JPY", 1, dec!(0.5733))], t(5)).await;
    assert_eq!(o.corrected.len(), 1);
}

/// (b) pour un jour : rendu sous un autre `<datum>` à valeur égale,
/// `source_date` reste celle de la valeur actuelle ; (c) la remplace.
#[sqlx::test(migrations = "./test-schema")]
async fn jour_rendu_sous_un_autre_datum(pool: MySqlPool) {
    let dim = RateUnit::day(d(2024, 6, 30));
    ecrire(
        &pool,
        dim,
        Some(d(2024, 6, 28)),
        &[publie("EUR", 1, dec!(0.96215))],
        t(1),
    )
    .await;
    let o = ecrire(
        &pool,
        dim,
        Some(d(2024, 6, 27)),
        &[publie("EUR", 1, dec!(0.96215))],
        t(2),
    )
    .await;
    assert_eq!(o.confirmed, 1);
    assert_eq!(
        publiee(&pool, "EUR", dim).await.unwrap().source_date,
        Some(d(2024, 6, 28))
    );
    let o = ecrire(
        &pool,
        dim,
        Some(d(2024, 6, 29)),
        &[publie("EUR", 1, dec!(0.96300))],
        t(3),
    )
    .await;
    assert_eq!(o.corrected.len(), 1);
    assert_eq!(
        publiee(&pool, "EUR", dim).await.unwrap().source_date,
        Some(d(2024, 6, 29))
    );

    // source_date incohérente avec le genre : refus, rien d'écrit.
    let mut tx = pool.begin().await.unwrap();
    let e = er::write_fetched_unit_in_tx(
        &mut tx,
        SOURCE_AFC,
        mois(2024, 6),
        Some(d(2024, 6, 1)),
        &[publie("EUR", 1, dec!(1))],
        t(4),
    )
    .await;
    assert!(matches!(e, Err(DbError::Invariant(_))));
    let e = er::write_fetched_unit_in_tx(
        &mut tx,
        SOURCE_AFC,
        dim,
        None,
        &[publie("EUR", 1, dec!(1))],
        t(4),
    )
    .await;
    assert!(matches!(e, Err(DbError::Invariant(_))));
}

/// ⛔ Une récupération ne modifie aucune ligne de `exchange_rates` : ni une
/// ligne `manual` (avec sa version), ni une ligne `afc` — la correction
/// s'annonce.
#[sqlx::test(migrations = "./test-schema")]
async fn une_recuperation_ne_touche_pas_les_cours_retenus(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let u = utilisateur(&pool, a, "a_admin").await;
    compte(&pool, a, "1020", Some("USD"), true).await;
    compte(&pool, a, "1021", Some("EUR"), true).await;
    let dec_24 = mois(2024, 12);
    ecrire(
        &pool,
        dec_24,
        None,
        &[publie("USD", 1, dec!(0.88)), publie("EUR", 1, dec!(0.93))],
        t(1),
    )
    .await;
    assert_eq!(copier(&pool, a, &[dec_24], None).await, 2);
    let manuel = saisir(&pool, a, u, "USD", dec_24, dec!(0.90), Some(1))
        .await
        .unwrap();
    let manuel = manuel.after.unwrap();
    assert_eq!(
        (manuel.source.clone(), manuel.version),
        (RateOrigin::Manual, 2)
    );
    let avant = lignes_de(&pool, a).await;

    ecrire(
        &pool,
        dec_24,
        None,
        &[publie("USD", 1, dec!(0.87)), publie("EUR", 1, dec!(0.95))],
        t(2),
    )
    .await;
    assert_eq!(copier(&pool, a, &[dec_24], None).await, 0);
    assert_eq!(lignes_de(&pool, a).await, avant);
    let usd = retenu(&pool, a, "USD", dec_24).await.unwrap();
    assert_eq!((usd.rate, usd.version), (dec!(0.9), 2));
    let mut conn = pool.acquire().await.unwrap();
    let notices = er::list_notices(&mut conn, a).await.unwrap();
    assert_eq!(
        notices.len(),
        1,
        "seule la ligne afc s'annonce : {notices:?}"
    );
    assert_eq!(
        (
            notices[0].currency.as_str(),
            notices[0].effective_rate,
            notices[0].published_rate
        ),
        ("EUR", dec!(0.93), dec!(0.95))
    );
}

// ---------------------------------------------------------------------------
// Copie (AC-3 bis)
// ---------------------------------------------------------------------------

/// A copiée (exercice multi-devises ouvert), B mono-devise non, C (exercice
/// clos) non, D (exercice mono-devise d'une société multi-devises) non ;
/// devise sans compte (SEK) non ; compte archivé (JPY) copié ; `currency`
/// restreint ; ligne `manual` inchangée ; `updated_by` NULL.
#[sqlx::test(migrations = "./test-schema")]
async fn copie_aux_societes_et_devises_utiles(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let b = societe(&pool, "B SA", false).await;
    let c = societe(&pool, "C SA", true).await;
    let dd = societe(&pool, "D SA", true).await;
    for s in [a, c, dd] {
        compte(&pool, s, "1020", Some("USD"), true).await;
    }
    compte(&pool, a, "1030", Some("JPY"), false).await;
    compte(&pool, b, "1000", None, true).await;
    let fy = |s, ouvert, multi| {
        exercice(
            &pool,
            s,
            "2024",
            d(2024, 1, 1),
            d(2024, 12, 31),
            ouvert,
            multi,
            "month",
            "day",
        )
    };
    fy(a, true, true).await;
    fy(b, true, false).await;
    fy(c, false, true).await;
    fy(dd, true, false).await;

    let dec_24 = mois(2024, 12);
    ecrire(
        &pool,
        dec_24,
        None,
        &[
            publie("USD", 1, dec!(0.88)),
            publie("JPY", 100, dec!(0.57)),
            publie("SEK", 100, dec!(8.1)),
        ],
        t(1),
    )
    .await;
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(
        er::companies_with_unit_in_tx(&mut conn, dec_24)
            .await
            .unwrap(),
        vec![a]
    );
    // Mois d'ouverture `month` (décembre 2023) et jour de clôture : utiles à A.
    assert_eq!(
        er::companies_with_unit_in_tx(&mut conn, mois(2023, 12))
            .await
            .unwrap(),
        vec![a]
    );
    assert_eq!(
        er::companies_with_unit_in_tx(&mut conn, RateUnit::day(d(2024, 12, 31)))
            .await
            .unwrap(),
        vec![a]
    );
    assert!(
        er::companies_with_unit_in_tx(&mut conn, mois(2023, 11))
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        er::companies_with_unit_in_tx(&mut conn, RateUnit::day(d(2023, 12, 31)))
            .await
            .unwrap()
            .is_empty()
    );
    drop(conn);

    // `currency` restreint : la seule devise donnée.
    assert_eq!(copier(&pool, a, &[dec_24], Some("JPY")).await, 1);
    assert_eq!(
        copier(&pool, a, &[dec_24], None).await,
        1,
        "USD seulement ; SEK sans compte"
    );
    let lignes = lignes_de(&pool, a).await;
    assert_eq!(
        lignes.iter().map(|l| l.0.as_str()).collect::<Vec<_>>(),
        ["JPY", "USD"]
    );
    let jpy = retenu(&pool, a, "JPY", dec_24).await.unwrap();
    assert_eq!(
        (jpy.rate, jpy.source, jpy.updated_by, jpy.version),
        (dec!(0.0057), RateOrigin::Afc, None, 1)
    );
    assert_eq!(
        copier(&pool, b, &[dec_24], None).await,
        0,
        "B : aucun compte en devise"
    );

    // Ligne manual présente : inchangée par la copie.
    let u = utilisateur(&pool, c, "c_admin").await;
    saisir(&pool, c, u, "USD", dec_24, dec!(0.99), None)
        .await
        .unwrap();
    assert_eq!(copier(&pool, c, &[dec_24], None).await, 0);
    assert_eq!(
        retenu(&pool, c, "USD", dec_24).await.unwrap().rate,
        dec!(0.99)
    );
    assert_eq!(copier(&pool, a, &[], None).await, 0);
}

/// Deux copies concurrentes de la même société et des mêmes unités : une
/// ligne par clé, aucune erreur.
#[sqlx::test(migrations = "./test-schema")]
async fn deux_copies_concurrentes(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    compte(&pool, a, "1020", Some("USD"), true).await;
    compte(&pool, a, "1021", Some("EUR"), true).await;
    let units = [mois(2024, 11), mois(2024, 12)];
    for u in units {
        ecrire(
            &pool,
            u,
            None,
            &[publie("USD", 1, dec!(0.88)), publie("EUR", 1, dec!(0.93))],
            t(1),
        )
        .await;
    }
    let copie = |pool: MySqlPool| async move {
        kesh_db::retry::retry_on_deadlock("test::copie", || {
            let pool = pool.clone();
            async move {
                let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
                let n = er::copy_fetched_in_tx(&mut tx, a, &units, None).await?;
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
                Ok::<_, DbError>(n)
            }
        })
        .await
    };
    let (x, y) = tokio::join!(copie(pool.clone()), copie(pool.clone()));
    let total = x.unwrap() + y.unwrap();
    assert_eq!(total, 4);
    assert_eq!(lignes_de(&pool, a).await.len(), 4);
}

/// Revue de code P1 (37-2-CR-P1-3) — une récupération et une copie
/// concurrentes (création d'un exercice ou d'un compte en devise) : la
/// récupération écrit les valeurs publiées sans encore valider ; la copie
/// concurrente, faite pendant ce temps, **attend** ces valeurs (lecture
/// verrouillée) au lieu de les manquer dans son instantané ; la récupération,
/// qui ne voit pas encore la société, valide ; la copie reprend et copie.
/// Sans la lecture verrouillée, aucune des deux ne copiait, et l'unité, plus
/// « manquante », ne se redemandait jamais.
#[sqlx::test(migrations = "./test-schema")]
async fn copie_concurrente_d_une_recuperation_non_validee(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    compte(&pool, a, "1020", Some("USD"), true).await;
    let u = mois(2024, 11);

    // La récupération : valeurs publiées écrites, transaction ouverte.
    let mut recuperation = pool.begin().await.unwrap();
    er::write_fetched_unit_in_tx(
        &mut recuperation,
        SOURCE_AFC,
        u,
        None,
        &[publie("USD", 1, dec!(0.88))],
        t(1),
    )
    .await
    .unwrap();

    // La copie concurrente, dans sa propre transaction.
    let p = pool.clone();
    let copie = tokio::spawn(async move {
        let mut tx = p.begin().await.unwrap();
        let n = er::copy_fetched_in_tx(&mut tx, a, &[u], None)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        n
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(
        !copie.is_finished(),
        "la copie doit attendre la récupération non validée"
    );
    recuperation.commit().await.unwrap();
    assert_eq!(copie.await.unwrap(), 1);
    assert_eq!(retenu(&pool, a, "USD", u).await.unwrap().rate, dec!(0.88));
}

/// Les exercices multi-devises ouverts, par société ou de toutes.
#[sqlx::test(migrations = "./test-schema")]
async fn exercices_multi_devises_ouverts(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let b = societe(&pool, "B SA", true).await;
    let fa = exercice(
        &pool,
        a,
        "2024",
        d(2024, 7, 1),
        d(2025, 6, 30),
        true,
        true,
        "month",
        "day",
    )
    .await;
    exercice(
        &pool,
        a,
        "2023",
        d(2023, 7, 1),
        d(2024, 6, 30),
        false,
        true,
        "day",
        "day",
    )
    .await;
    let fb = exercice(
        &pool,
        b,
        "2025",
        d(2025, 1, 1),
        d(2025, 12, 31),
        true,
        true,
        "day",
        "month",
    )
    .await;
    exercice(
        &pool,
        b,
        "2024",
        d(2024, 1, 1),
        d(2024, 12, 31),
        true,
        false,
        "day",
        "day",
    )
    .await;
    let mut conn = pool.acquire().await.unwrap();
    let tous = er::list_open_multi_currency_fiscal_years(&mut conn, None)
        .await
        .unwrap();
    assert_eq!(tous.iter().map(|f| f.id).collect::<Vec<_>>(), vec![fa, fb]);
    assert_eq!(tous[0].exercice.opening_kind, RateKind::Month);
    assert_eq!(tous[1].exercice.closing_kind, RateKind::Month);
    let de_b = er::list_open_multi_currency_fiscal_years(&mut conn, Some(b))
        .await
        .unwrap();
    assert_eq!(de_b.len(), 1);
    assert_eq!(de_b[0].company_id, b);
}

// ---------------------------------------------------------------------------
// Cours retenus : lecture tolérante, saisie, reprise
// ---------------------------------------------------------------------------

/// Une ligne `source = 'ecb'` posée par SQL (A30) se lit `Other("ecb")` ; son
/// `PUT` la passe en `manual`, son ↑ en `afc`, son ↑ sans valeur publiée la
/// supprime.
#[sqlx::test(migrations = "./test-schema")]
async fn source_inconnue_lue_et_traitee_comme_manual(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let u = utilisateur(&pool, a, "a_admin").await;
    let pose = |devise: &'static str, date: NaiveDate| {
        let pool = pool.clone();
        async move {
            sqlx::query(
                "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
                 VALUES (?, ?, 'month', ?, 0.5, 'ecb')",
            )
            .bind(a)
            .bind(devise)
            .bind(date)
            .execute(&pool)
            .await
            .unwrap();
        }
    };
    let nov = mois(2024, 11);
    let dec_24 = mois(2024, 12);
    pose("USD", nov.date).await;
    pose("EUR", nov.date).await;
    pose("GBP", dec_24.date).await;
    let usd = retenu(&pool, a, "USD", nov).await.unwrap();
    assert_eq!(usd.source, RateOrigin::Other("ecb".into()));
    assert!(!usd.source.is_afc());

    let r = saisir(&pool, a, u, "USD", nov, dec!(0.6), Some(1))
        .await
        .unwrap();
    assert_eq!(r.before.unwrap().source, RateOrigin::Other("ecb".into()));
    assert_eq!(r.after.unwrap().source, RateOrigin::Manual);

    ecrire(&pool, nov, None, &[publie("EUR", 1, dec!(0.93))], t(1)).await;
    let r = reprendre(&pool, a, u, "EUR", nov, Some(1), Some(dec!(0.93)))
        .await
        .unwrap();
    let apres = r.after.unwrap();
    assert_eq!(
        (apres.source, apres.rate, apres.version),
        (RateOrigin::Afc, dec!(0.93), 2)
    );

    let r = reprendre(&pool, a, u, "GBP", dec_24, Some(1), None)
        .await
        .unwrap();
    assert!(r.after.is_none(), "sans valeur publiée : supprimée");
    assert!(retenu(&pool, a, "GBP", dec_24).await.is_none());
}

/// `set_manual_rate` : création, remplacement d'une ligne `afc` (devient
/// `manual`, `acknowledged_*` à NULL), conflits de version, version + 1.
#[sqlx::test(migrations = "./test-schema")]
async fn saisie_d_un_cours(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let u = utilisateur(&pool, a, "a_admin").await;
    compte(&pool, a, "1020", Some("USD"), true).await;
    let dec_24 = mois(2024, 12);

    let r = saisir(&pool, a, u, "EUR", dec_24, dec!(0.9123456789), None)
        .await
        .unwrap();
    assert!(r.before.is_none());
    let cree = r.after.unwrap();
    assert_eq!(
        (
            cree.rate,
            cree.source.clone(),
            cree.version,
            cree.updated_by
        ),
        (dec!(0.9123456789), RateOrigin::Manual, 1, Some(u))
    );
    // Création sur une ligne présente, version périmée, version sur une case
    // vide : conflit.
    assert!(matches!(
        saisir(&pool, a, u, "EUR", dec_24, dec!(1), None).await,
        Err(DbError::OptimisticLockConflict)
    ));
    assert!(matches!(
        saisir(&pool, a, u, "EUR", dec_24, dec!(1), Some(7)).await,
        Err(DbError::OptimisticLockConflict)
    ));
    assert!(matches!(
        saisir(&pool, a, u, "GBP", dec_24, dec!(1), Some(1)).await,
        Err(DbError::OptimisticLockConflict)
    ));
    let r = saisir(&pool, a, u, "EUR", dec_24, dec!(0.95), Some(1))
        .await
        .unwrap();
    assert_eq!(r.after.unwrap().version, 2);

    // Remplacement d'une ligne afc acquittée.
    ecrire(&pool, dec_24, None, &[publie("USD", 1, dec!(0.88))], t(1)).await;
    copier(&pool, a, &[dec_24], None).await;
    ecrire(&pool, dec_24, None, &[publie("USD", 1, dec!(0.89))], t(2)).await;
    let mut tx = pool.begin().await.unwrap();
    let ack = er::acknowledge_corrections(
        &mut tx,
        a,
        &[NoticeAck {
            currency: "USD".into(),
            kind: RateKind::Month,
            rate_date: dec_24.date,
            published_rate: dec!(0.89),
        }],
        t(3),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(ack.len(), 1);
    let r = saisir(&pool, a, u, "USD", dec_24, dec!(0.89), Some(1))
        .await
        .unwrap();
    let apres = r.after.unwrap();
    assert_eq!(
        (
            apres.source,
            apres.acknowledged_rate,
            apres.acknowledged_at,
            apres.version
        ),
        (RateOrigin::Manual, None, None, 2)
    );
    // Égal à la valeur publiée : conservé manual.
    assert_eq!(apres.rate, dec!(0.89));
}

/// Deux créations simultanées du même cours : une réussit, l'autre rend un
/// conflit (jamais une erreur interne), une fois rejouée sur interblocage.
#[sqlx::test(migrations = "./test-schema")]
async fn deux_creations_simultanees(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let u = utilisateur(&pool, a, "a_admin").await;
    let dec_24 = mois(2024, 12);
    let creation = |pool: MySqlPool, cours: Decimal| async move {
        kesh_db::retry::retry_on_deadlock("test::saisie", || {
            let pool = pool.clone();
            async move {
                let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
                let r = er::set_manual_rate(
                    &mut tx,
                    a,
                    u,
                    "USD",
                    RateKind::Month,
                    dec_24.date,
                    cours,
                    None,
                )
                .await?;
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
                Ok::<_, DbError>(r)
            }
        })
        .await
    };
    let (x, y) = tokio::join!(
        creation(pool.clone(), dec!(0.9)),
        creation(pool.clone(), dec!(0.91))
    );
    let ok = [x.is_ok(), y.is_ok()].iter().filter(|b| **b).count();
    assert_eq!(ok, 1, "{x:?} / {y:?}");
    for r in [x, y] {
        if let Err(e) = r {
            assert!(matches!(e, DbError::OptimisticLockConflict), "{e:?}");
        }
    }
    assert_eq!(lignes_de(&pool, a).await.len(), 1);
}

/// `reset_rate` (↑) : tous les cas de l'AC-5.
#[sqlx::test(migrations = "./test-schema")]
async fn reprise_de_la_valeur_publiee(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let u = utilisateur(&pool, a, "a_admin").await;
    compte(&pool, a, "1020", Some("USD"), true).await;
    let dec_24 = mois(2024, 12);
    let nov = mois(2024, 11);

    // Ligne manual → afc à la valeur publiée.
    ecrire(
        &pool,
        dec_24,
        None,
        &[publie("USD", 1, dec!(0.88)), publie("EUR", 1, dec!(0.93))],
        t(1),
    )
    .await;
    saisir(&pool, a, u, "EUR", dec_24, dec!(0.95), None)
        .await
        .unwrap();
    let r = reprendre(&pool, a, u, "EUR", dec_24, Some(1), Some(dec!(0.93)))
        .await
        .unwrap();
    assert_eq!(r.published_rate, Some(dec!(0.93)));
    assert_eq!(r.before.as_ref().unwrap().source, RateOrigin::Manual);
    let apres = r.after.unwrap();
    assert_eq!(
        (apres.rate, apres.source, apres.version, apres.updated_by),
        (dec!(0.93), RateOrigin::Afc, 2, Some(u))
    );

    // Ligne afc corrigée → nouvelle valeur ; `publishedRate` périmé → conflit ;
    // version absente ou périmée → conflit.
    copier(&pool, a, &[dec_24], None).await;
    ecrire(&pool, dec_24, None, &[publie("USD", 1, dec!(0.87))], t(2)).await;
    assert!(matches!(
        reprendre(&pool, a, u, "USD", dec_24, Some(1), Some(dec!(0.88))).await,
        Err(DbError::OptimisticLockConflict)
    ));
    assert!(matches!(
        reprendre(&pool, a, u, "USD", dec_24, None, Some(dec!(0.87))).await,
        Err(DbError::OptimisticLockConflict)
    ));
    assert!(matches!(
        reprendre(&pool, a, u, "USD", dec_24, Some(9), Some(dec!(0.87))).await,
        Err(DbError::OptimisticLockConflict)
    ));
    let r = reprendre(&pool, a, u, "USD", dec_24, Some(1), Some(dec!(0.87)))
        .await
        .unwrap();
    assert_eq!(r.after.unwrap().rate, dec!(0.87));

    // Case vide dont la valeur est publiée → ligne afc créée ; une version sur
    // une case vide est interdite.
    ecrire(&pool, nov, None, &[publie("GBP", 1, dec!(1.12))], t(3)).await;
    assert!(matches!(
        reprendre(&pool, a, u, "GBP", nov, Some(1), None).await,
        Err(DbError::InvalidInput(_))
    ));
    let r = reprendre(&pool, a, u, "GBP", nov, None, Some(dec!(1.12)))
        .await
        .unwrap();
    assert!(r.before.is_none());
    let cree = r.after.unwrap();
    assert_eq!(
        (cree.source, cree.rate, cree.updated_by),
        (RateOrigin::Afc, dec!(1.12), Some(u))
    );

    // Sans valeur publiée : manual supprimée, afc → 404, rien → 404.
    let oct = mois(2024, 10);
    saisir(&pool, a, u, "CAD", oct, dec!(0.6), None)
        .await
        .unwrap();
    assert!(matches!(
        reprendre(&pool, a, u, "CAD", oct, Some(2), None).await,
        Err(DbError::OptimisticLockConflict)
    ));
    // Revue de code P1 (37-2-CR-P1-1) : l'écran a montré une valeur publiée
    // qui a disparu depuis (tables système vidées) → conflit, et le cours
    // saisi reste ; jamais « reprendre » transformé en suppression.
    assert!(matches!(
        reprendre(&pool, a, u, "CAD", oct, Some(1), Some(dec!(0.61))).await,
        Err(DbError::OptimisticLockConflict)
    ));
    assert_eq!(retenu(&pool, a, "CAD", oct).await.unwrap().rate, dec!(0.6));
    let r = reprendre(&pool, a, u, "CAD", oct, Some(1), None)
        .await
        .unwrap();
    assert!(r.after.is_none());
    assert!(r.published_rate.is_none());
    sqlx::query("INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) VALUES (?, 'AUD', 'month', ?, 0.55, 'afc')")
        .bind(a)
        .bind(oct.date)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        reprendre(&pool, a, u, "AUD", oct, Some(1), None).await,
        Err(DbError::NotFound)
    ));
    assert!(matches!(
        reprendre(&pool, a, u, "NOK", oct, None, None).await,
        Err(DbError::NotFound)
    ));
}

// ---------------------------------------------------------------------------
// Notices et acquittement (AC-5)
// ---------------------------------------------------------------------------

/// Deux sociétés, une ligne afc USD décembre dans chacune : correction →
/// notice dans A et B ; acquittement dans A → absente de A, présente dans B ;
/// ↑ dans B → B à la nouvelle valeur, A inchangée ; nouvelle correction →
/// notice de retour dans A ; ligne manual qui diffère → aucune notice.
#[sqlx::test(migrations = "./test-schema")]
async fn notices_et_acquittement_par_societe(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let b = societe(&pool, "B SA", true).await;
    let ua = utilisateur(&pool, a, "a_admin").await;
    let ub = utilisateur(&pool, b, "b_admin").await;
    for s in [a, b] {
        compte(&pool, s, "1020", Some("USD"), true).await;
        compte(&pool, s, "1021", Some("EUR"), true).await;
    }
    let dec_24 = mois(2024, 12);
    ecrire(
        &pool,
        dec_24,
        None,
        &[publie("USD", 1, dec!(0.88)), publie("EUR", 1, dec!(0.93))],
        t(1),
    )
    .await;
    copier(&pool, a, &[dec_24], None).await;
    copier(&pool, b, &[dec_24], None).await;
    saisir(&pool, a, ua, "EUR", dec_24, dec!(0.99), Some(1))
        .await
        .unwrap();
    let notices = |s: i64| {
        let pool = pool.clone();
        async move {
            let mut conn = pool.acquire().await.unwrap();
            er::list_notices(&mut conn, s).await.unwrap()
        }
    };
    assert!(notices(a).await.is_empty());

    ecrire(
        &pool,
        dec_24,
        None,
        &[publie("USD", 1, dec!(0.89)), publie("EUR", 1, dec!(0.94))],
        t(2),
    )
    .await;
    let na = notices(a).await;
    assert_eq!(na.len(), 1, "la ligne manual EUR ne s'annonce pas : {na:?}");
    assert_eq!(
        (
            na[0].currency.as_str(),
            na[0].kind,
            na[0].rate_date,
            na[0].effective_rate,
            na[0].published_rate,
            na[0].published_unit,
            na[0].value_changed_at
        ),
        (
            "USD",
            RateKind::Month,
            dec_24.date,
            dec!(0.88),
            dec!(0.89),
            1,
            t(2)
        )
    );
    assert_eq!(notices(b).await.len(), 2);

    let ack = |s: i64, devise: &str, valeur: Decimal| {
        let pool = pool.clone();
        let item = NoticeAck {
            currency: devise.into(),
            kind: RateKind::Month,
            rate_date: dec_24.date,
            published_rate: valeur,
        };
        async move {
            let mut tx = pool.begin().await.unwrap();
            let r = er::acknowledge_corrections(&mut tx, s, &[item.clone(), item], t(3))
                .await
                .unwrap();
            tx.commit().await.unwrap();
            r
        }
    };
    // Valeur qui n'est plus la publiée : pas acquittée.
    assert!(ack(a, "USD", dec!(0.88)).await.is_empty());
    // Ligne manual : pas acquittée.
    assert!(ack(a, "EUR", dec!(0.94)).await.is_empty());
    let avant = retenu(&pool, a, "USD", dec_24).await.unwrap();
    let faits = ack(a, "USD", dec!(0.89)).await;
    assert_eq!(faits.len(), 1, "doublon du corps ignoré");
    let apres = retenu(&pool, a, "USD", dec_24).await.unwrap();
    assert_eq!(
        (
            apres.rate,
            apres.source.clone(),
            apres.version,
            apres.updated_at
        ),
        (
            avant.rate,
            avant.source.clone(),
            avant.version,
            avant.updated_at
        )
    );
    assert_eq!(
        (apres.acknowledged_rate, apres.acknowledged_at),
        (Some(dec!(0.89)), Some(t(3)))
    );
    assert!(notices(a).await.is_empty());
    assert_eq!(
        notices(b)
            .await
            .iter()
            .filter(|n| n.currency == "USD")
            .count(),
        1
    );

    // La case dit « corrigé, acquitté » dans A.
    let mut conn = pool.acquire().await.unwrap();
    let cases = er::list_cells_in_tx(&mut conn, a, RateKind::Month, dec_24.date, dec_24.date)
        .await
        .unwrap();
    let usd = cases.iter().find(|c| c.currency == "USD").unwrap();
    assert!(usd.corrected() && usd.acknowledged());
    let eur = cases.iter().find(|c| c.currency == "EUR").unwrap();
    assert!(!eur.corrected(), "ligne manual : jamais « corrigée »");
    drop(conn);

    let r = reprendre(&pool, b, ub, "USD", dec_24, Some(1), Some(dec!(0.89)))
        .await
        .unwrap();
    assert_eq!(r.after.unwrap().rate, dec!(0.89));
    assert_eq!(
        retenu(&pool, a, "USD", dec_24).await.unwrap().rate,
        dec!(0.88)
    );

    ecrire(&pool, dec_24, None, &[publie("USD", 1, dec!(0.90))], t(4)).await;
    assert_eq!(
        notices(a).await.len(),
        1,
        "nouvelle correction : notice de retour"
    );
}

// ---------------------------------------------------------------------------
// Lecture groupée, isolation, état, vidages
// ---------------------------------------------------------------------------

/// Les cases d'une plage : lignes de la société et valeurs publiées, union,
/// triées ; la société B ne voit pas les cours retenus de A.
#[sqlx::test(migrations = "./test-schema")]
async fn lecture_groupee_et_isolation(pool: MySqlPool) {
    let a = societe(&pool, "A SA", true).await;
    let b = societe(&pool, "B SA", true).await;
    let u = utilisateur(&pool, a, "a_admin").await;
    compte(&pool, a, "1020", Some("USD"), true).await;
    compte(&pool, a, "1030", Some("JPY"), false).await;
    compte(&pool, a, "1031", Some("USD"), true).await;
    for (u_, c) in [(mois(2024, 11), dec!(0.87)), (mois(2024, 12), dec!(0.88))] {
        ecrire(&pool, u_, None, &[publie("USD", 1, c)], t(1)).await;
    }
    ecrire(
        &pool,
        RateUnit::day(d(2024, 12, 31)),
        Some(d(2024, 12, 31)),
        &[publie("USD", 1, dec!(0.91193))],
        t(1),
    )
    .await;
    copier(&pool, a, &[mois(2024, 12)], None).await;
    saisir(&pool, a, u, "CAD", mois(2024, 11), dec!(0.62), None)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let cases = er::list_cells_in_tx(
        &mut conn,
        a,
        RateKind::Month,
        d(2024, 11, 1),
        d(2024, 12, 1),
    )
    .await
    .unwrap();
    let vue: Vec<(NaiveDate, &str, bool, bool)> = cases
        .iter()
        .map(|c| {
            (
                c.rate_date,
                c.currency.as_str(),
                c.company.is_some(),
                c.published.is_some(),
            )
        })
        .collect();
    assert_eq!(
        vue,
        [
            (d(2024, 11, 1), "CAD", true, false),
            (d(2024, 11, 1), "USD", false, true),
            (d(2024, 12, 1), "USD", true, true),
        ]
    );
    assert!(cases.iter().all(|c| c.kind == RateKind::Month));
    let jours = er::list_cells_in_tx(
        &mut conn,
        a,
        RateKind::Day,
        d(2024, 12, 31),
        d(2024, 12, 31),
    )
    .await
    .unwrap();
    assert_eq!(jours.len(), 1);
    assert_eq!(
        jours[0].published.as_ref().unwrap().source_date,
        Some(d(2024, 12, 31))
    );

    let de_b = er::list_cells_in_tx(
        &mut conn,
        b,
        RateKind::Month,
        d(2024, 11, 1),
        d(2024, 12, 1),
    )
    .await
    .unwrap();
    assert!(
        de_b.iter().all(|c| c.company.is_none()),
        "B ne voit pas les cours de A"
    );
    assert!(
        er::find_company_rate_in_tx(&mut conn, b, "USD", RateKind::Month, d(2024, 12, 1))
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        er::list_account_currencies(&mut conn, a).await.unwrap(),
        vec!["JPY".to_string(), "USD".to_string()]
    );
    assert!(
        er::list_account_currencies(&mut conn, b)
            .await
            .unwrap()
            .is_empty()
    );
}

/// État de la récupération (singleton paresseux), unités manquantes, date
/// d'obtention, vidage des deux tables système.
#[sqlx::test(migrations = "./test-schema")]
async fn etat_unites_manquantes_et_vidage(pool: MySqlPool) {
    let mut conn = pool.acquire().await.unwrap();
    assert!(er::get_fetch_status(&mut conn).await.unwrap().is_none());
    let etat = FetchStatus {
        trigger: FetchTrigger::Auto,
        last_run_at: t(1),
        status: FetchOutcomeStatus::Ok,
        error: Some("invalid_rates:2".into()),
        kind: RateKind::Month,
        rate_date: d(2024, 12, 1),
    };
    er::record_fetch_status(&mut conn, &etat).await.unwrap();
    assert_eq!(er::get_fetch_status(&mut conn).await.unwrap(), Some(etat));
    let etat2 = FetchStatus {
        trigger: FetchTrigger::Manual,
        last_run_at: t(2),
        status: FetchOutcomeStatus::NotPublished,
        error: None,
        kind: RateKind::Day,
        rate_date: d(2024, 12, 31),
    };
    er::record_fetch_status(&mut conn, &etat2).await.unwrap();
    assert_eq!(er::get_fetch_status(&mut conn).await.unwrap(), Some(etat2));
    drop(conn);

    ecrire(
        &pool,
        mois(2024, 12),
        None,
        &[publie("USD", 1, dec!(0.88))],
        t(3),
    )
    .await;
    ecrire(
        &pool,
        mois(2024, 12),
        None,
        &[publie("EUR", 1, dec!(0.93))],
        t(5),
    )
    .await;
    ecrire(
        &pool,
        mois(2024, 12),
        None,
        &[publie("USD", 1, dec!(0.89))],
        t(6),
    )
    .await;
    let mut conn = pool.acquire().await.unwrap();
    let units = [
        mois(2024, 11),
        mois(2024, 12),
        RateUnit::day(d(2024, 12, 1)),
    ];
    assert_eq!(
        er::missing_units(&mut conn, &units).await.unwrap(),
        vec![mois(2024, 11), RateUnit::day(d(2024, 12, 1))]
    );
    assert!(er::missing_units(&mut conn, &[]).await.unwrap().is_empty());
    // Résumé d'une unité (raccourci du `POST /fetch`, phase B1) : deux devises,
    // le plus grand `fetched_at` ; `None` pour une unité absente.
    assert_eq!(
        er::fetched_unit_summary(&mut conn, SOURCE_AFC, mois(2024, 12))
            .await
            .unwrap(),
        Some(er::FetchedUnitSummary {
            count: 2,
            last_fetched_at: t(6),
            source_date: None,
        })
    );
    assert_eq!(
        er::fetched_unit_summary(&mut conn, SOURCE_AFC, mois(2024, 11))
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        er::month_first_fetched_at(&mut conn, d(2024, 12, 1))
            .await
            .unwrap(),
        Some(t(3)),
        "jamais avancée par une correction"
    );
    assert_eq!(
        er::month_first_fetched_at(&mut conn, d(2024, 11, 1))
            .await
            .unwrap(),
        None
    );

    er::clear_fetched_state(&mut conn).await.unwrap();
    assert!(er::get_fetch_status(&mut conn).await.unwrap().is_none());
    assert_eq!(
        er::missing_units(&mut conn, &units).await.unwrap(),
        units.to_vec()
    );
}
