//! Story 37-3a — les lignes d'écriture en devise, au dépôt : contraintes
//! `chk_jel_fx_*` de la migration (AC-1), contrôle central de l'AC-4 (ordre des
//! refus, règle d'une devise, accord ligne-compte, provenance du cours,
//! qualification), `PUT` et contre-passation (AC-7).
//!
//! Montage : la société semée par `seed_accounting_company` (exercice
//! 2020-2030), passée multi-devises en SQL, avec des comptes en USD et en EUR
//! et les cours retenus par la société (`exchange_rates`) posés en SQL.

use chrono::NaiveDate;
use kesh_db::entities::{
    Acteur, ForeignAmount, Journal, NewJournalEntry, NewJournalEntryLine, RateProvenance,
};
use kesh_db::errors::{DbError, ModificationBlocker};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, chf_line, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

struct Decor {
    s: SeededCompany,
    /// Compte 1000 (CHF).
    caisse: i64,
    /// Compte 3000 (CHF).
    ventes: i64,
    usd: i64,
    usd2: i64,
    eur: i64,
    eur2: i64,
}

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

async fn compte(pool: &MySqlPool, company_id: i64, numero: &str, devise: Option<&str>) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, ?, ?, 'Asset', ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
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
    let usd = compte(pool, s.company_id, "1020", Some("USD")).await;
    let usd2 = compte(pool, s.company_id, "1021", Some("USD")).await;
    let eur = compte(pool, s.company_id, "1030", Some("EUR")).await;
    let eur2 = compte(pool, s.company_id, "1031", Some("EUR")).await;
    Decor {
        caisse: s.accounts["1000"],
        ventes: s.accounts["3000"],
        s,
        usd,
        usd2,
        eur,
        eur2,
    }
}

/// Le cours retenu par la société pour un mois (ligne `exchange_rates`).
async fn cours(
    pool: &MySqlPool,
    company_id: i64,
    devise: &str,
    mois: NaiveDate,
    rate: Decimal,
    source: &str,
) {
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, ?, 'month', ?, ?, ?) \
         ON DUPLICATE KEY UPDATE rate = VALUES(rate), source = VALUES(source)",
    )
    .bind(company_id)
    .bind(devise)
    .bind(mois)
    .bind(rate)
    .bind(source)
    .execute(pool)
    .await
    .unwrap();
}

fn fx(amount: Decimal, rate: Decimal, source: Option<RateProvenance>) -> Option<ForeignAmount> {
    Some(ForeignAmount {
        declared_currency: None,
        amount,
        rate,
        source,
        rate_derived: false,
    })
}

/// Une ligne en devise au débit, CHF = montant × cours arrondi.
fn ligne_fx(account: i64, chf: Decimal, f: Option<ForeignAmount>) -> NewJournalEntryLine {
    NewJournalEntryLine {
        foreign: f,
        ..chf_line(account, chf, Decimal::ZERO)
    }
}

fn ecriture(dec: &Decor, date: NaiveDate, lines: Vec<NewJournalEntryLine>) -> NewJournalEntry {
    NewJournalEntry {
        company_id: dec.s.company_id,
        entry_date: date,
        journal: Journal::OD,
        description: "Écriture en devise".into(),
        project_id: None,
        lines,
    }
}

async fn creer(
    pool: &MySqlPool,
    dec: &Decor,
    new: NewJournalEntry,
) -> Result<kesh_db::entities::JournalEntryWithLines, DbError> {
    journal_entries::create(
        pool,
        dec.s.fiscal_year_id,
        Acteur::utilisateur(dec.s.admin_user_id),
        new,
    )
    .await
}

/// L'écriture USD de référence : 1234.56 USD à 0.91193 = 1125.83 CHF, contre
/// les ventes.
fn usd_afc(dec: &Decor, rate: Decimal, source: Option<RateProvenance>) -> NewJournalEntry {
    ecriture(
        dec,
        d(2026, 3, 15),
        vec![
            ligne_fx(dec.usd, dec!(1125.83), fx(dec!(1234.56), rate, source)),
            chf_line(dec.ventes, Decimal::ZERO, dec!(1125.83)),
        ],
    )
}

// ---------------------------------------------------------------------------
// AC-1 — les contraintes `chk_jel_fx_*`, en SQL brut
// ---------------------------------------------------------------------------

/// Insère une ligne en SQL brut et rend le nom de la contrainte violée (lu
/// dans le message), `None` si l'insertion passe.
#[allow(clippy::too_many_arguments)]
async fn violation(
    pool: &MySqlPool,
    entry_id: i64,
    account: i64,
    ordre: i32,
    debit: &str,
    amount: Option<&str>,
    rate: Option<&str>,
    source: Option<&str>,
) -> Option<String> {
    let r = sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit, \
         amount_currency, exchange_rate, rate_source) VALUES (?, ?, ?, ?, 0, ?, ?, ?)",
    )
    .bind(entry_id)
    .bind(account)
    .bind(ordre)
    .bind(debit)
    .bind(amount)
    .bind(rate)
    .bind(source)
    .execute(pool)
    .await;
    match r {
        Ok(_) => None,
        Err(e) => {
            let m = e.to_string();
            let nom = m
                .split(['`', '\''])
                .find(|t| t.starts_with("chk_jel_"))
                .map(str::to_string);
            Some(nom.unwrap_or(m))
        }
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn ac1_chaque_contrainte_refuse_son_cas(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let entry_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, 999, '2026-03-15', 'OD', 'Brut') RETURNING id",
    )
    .bind(dec.s.company_id)
    .bind(dec.s.fiscal_year_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut n = 0;
    let mut ordre = || {
        n += 1;
        n
    };
    // (libellé, débit, montant, cours, source, contrainte attendue)
    type Cas<'a> = (
        &'a str,
        &'a str,
        Option<&'a str>,
        Option<&'a str>,
        Option<&'a str>,
        Option<&'a str>,
    );
    let cas: Vec<Cas> = vec![
        // (libellé, débit, montant, cours, source, contrainte attendue)
        (
            "trois colonnes partielles",
            "10",
            Some("10"),
            None,
            None,
            Some("chk_jel_fx_all_or_none"),
        ),
        (
            "source inconnue",
            "9.12",
            Some("10"),
            Some("0.912"),
            Some("ecb"),
            Some("chk_jel_fx_rate_source"),
        ),
        (
            "source en majuscules",
            "9.12",
            Some("10"),
            Some("0.912"),
            Some("AFC"),
            Some("chk_jel_fx_rate_source"),
        ),
        (
            "réévaluation au montant non nul",
            "5",
            Some("1"),
            Some("0.9"),
            Some("revaluation"),
            Some("chk_jel_fx_amount"),
        ),
        (
            "cours 0 (ligne de réévaluation)",
            "5",
            Some("0"),
            Some("0"),
            Some("revaluation"),
            Some("chk_jel_fx_rate"),
        ),
        (
            "CHF incohérent",
            "9.13",
            Some("10"),
            Some("0.912"),
            Some("manual"),
            Some("chk_jel_fx_chf"),
        ),
        ("ligne CHF", "10", None, None, None, None),
        (
            "réévaluation valide",
            "5",
            Some("0"),
            Some("0.9"),
            Some("revaluation"),
            None,
        ),
        (
            "ligne en devise valide",
            "9.12",
            Some("10"),
            Some("0.912"),
            Some("afc"),
            None,
        ),
    ];
    for (libelle, debit, amount, rate, source, attendu) in cas {
        let o = ordre();
        let r = violation(&pool, entry_id, dec.usd, o, debit, amount, rate, source).await;
        assert_eq!(r.as_deref(), attendu, "{libelle}");
    }
    // Montant 0 hors réévaluation et montant < 0 violent AUSSI `chk_jel_fx_chf`
    // (CHF > 0 par `chk_jel_debit_credit_exclusive`) : le nom rapporté est
    // constaté ici et écrit au Change Log.
    let zero = violation(
        &pool,
        entry_id,
        dec.usd,
        ordre(),
        "1",
        Some("0"),
        Some("1"),
        Some("manual"),
    )
    .await;
    let negatif = violation(
        &pool,
        entry_id,
        dec.usd,
        ordre(),
        "1",
        Some("-1"),
        Some("1"),
        Some("manual"),
    )
    .await;
    for (libelle, r) in [
        ("montant 0 hors réévaluation", zero),
        ("montant < 0", negatif),
    ] {
        let nom = r.unwrap_or_else(|| panic!("{libelle} : accepté"));
        assert!(
            nom == "chk_jel_fx_amount" || nom == "chk_jel_fx_chf",
            "{libelle} : {nom}"
        );
    }
}

/// AC-2 — `foreign()` d'une ligne à `amount_currency` posé sur un compte CHF
/// (état que le dépôt refuse, posé en SQL brut) → `None` : la ligne se lit
/// comme une ligne CHF.
#[sqlx::test(migrations = "./test-schema")]
async fn ac2_montant_sur_un_compte_chf_se_lit_comme_une_ligne_chf(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let je = creer(
        &pool,
        &dec,
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                chf_line(dec.caisse, dec!(9.12), Decimal::ZERO),
                chf_line(dec.ventes, Decimal::ZERO, dec!(9.12)),
            ],
        ),
    )
    .await
    .unwrap();
    sqlx::query(
        "UPDATE journal_entry_lines SET amount_currency = 10, exchange_rate = 0.912, \
         rate_source = 'manual' WHERE entry_id = ? AND account_id = ?",
    )
    .bind(je.entry.id)
    .bind(dec.caisse)
    .execute(&pool)
    .await
    .unwrap();
    let relue = journal_entries::find_by_id(&pool, dec.s.company_id, je.entry.id)
        .await
        .unwrap()
        .unwrap();
    let l = relue
        .lines
        .iter()
        .find(|l| l.account_id == dec.caisse)
        .unwrap();
    assert_eq!(l.amount_currency, Some(dec!(10)));
    assert!(l.foreign().is_none(), "compte CHF : vue CHF");
}

// ---------------------------------------------------------------------------
// AC-4 — contrôle central
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn ac4_ligne_afc_au_cours_retenu_passe_et_relue_en_usd(pool: MySqlPool) {
    let dec = decor(&pool).await;
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "afc",
    )
    .await;
    let je = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap();
    let f = je.lines[0].foreign().expect("ligne en devise");
    assert_eq!(f.currency, "USD");
    assert_eq!(f.amount, dec!(1234.56));
    assert_eq!(f.rate, dec!(0.91193));
    assert_eq!(f.source, RateProvenance::Afc);
    assert!(je.lines[1].foreign().is_none());
    // Relue par `find_by_id` (jointure).
    let relue = journal_entries::find_by_id(&pool, dec.s.company_id, je.entry.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(relue.lines[0].foreign().unwrap().currency, "USD");
    // L'audit porte `foreign` pour la seule ligne en devise.
    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'journal_entry.created' AND entity_id = ?",
    )
    .bind(je.entry.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        details["lines"][0]["foreign"],
        serde_json::json!({ "currency": "USD", "amount": "1234.56", "rate": "0.91193", "rateSource": "afc" })
    );
    assert!(details["lines"][1].get("foreign").is_none());
}

#[sqlx::test(migrations = "./test-schema")]
async fn ac4_rang4_cours_afc(pool: MySqlPool) {
    let dec = decor(&pool).await;
    // (c) aucune ligne de la société pour le mois : `afc` → RATE_UNAVAILABLE ;
    // `manual` → passe. Une valeur publiée n'entre jamais dans le contrôle.
    sqlx::query(
        "INSERT INTO _kesh_fetched_exchange_rates (source, currency, kind, rate_date, rate, unit) \
         VALUES ('afc', 'USD', 'month', '2026-03-01', 0.91193, 1)",
    )
    .execute(&pool)
    .await
    .ok();
    let e = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(&e, DbError::RateUnavailable { currency, month } if currency == "USD" && *month == d(2026, 3, 1)),
        "{e:?}"
    );
    creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Manual)),
    )
    .await
    .expect("manual sans cours retenu");
    // (a) ligne `afc` de la société : un autre cours → FX_RATE_NOT_AFC.
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91194),
        "afc",
    )
    .await;
    let e = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(e, DbError::FxRateNotAfc { line: 1, expected_rate: Some(r) } if r == dec!(0.91194)),
        "{e:?}"
    );
    // (b) ligne `manual` de la société : `afc` refusée, `expectedRate` null.
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "manual",
    )
    .await;
    let e = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(
            e,
            DbError::FxRateNotAfc {
                line: 1,
                expected_rate: None
            }
        ),
        "{e:?}"
    );
    creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Manual)),
    )
    .await
    .expect("manual au cours saisi");
    // (b bis) source `ecb` (A30) : traitée comme `manual`.
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "ecb",
    )
    .await;
    let e = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(
            e,
            DbError::FxRateNotAfc {
                expected_rate: None,
                ..
            }
        ),
        "{e:?}"
    );
    // (d) un cours journalier pour la date ne change rien.
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, 'USD', 'day', '2026-03-15', 0.91193, 'afc')",
    )
    .bind(dec.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let e = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(
            e,
            DbError::FxRateNotAfc {
                expected_rate: None,
                ..
            }
        ),
        "{e:?}"
    );
}

/// (g) Deux sociétés : chaque ligne `afc` se juge contre le cours de **sa**
/// société.
#[sqlx::test(migrations = "./test-schema")]
async fn ac4_rang4_cours_de_sa_societe(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let autre: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language, \
         multi_currency_enabled) VALUES ('Autre SA', 'Rue 2', 'Pme', 'FR', 'FR', TRUE) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    cours(&pool, autre, "USD", d(2026, 3, 1), dec!(0.91193), "afc").await;
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.90000),
        "afc",
    )
    .await;
    let e = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(e, DbError::FxRateNotAfc { expected_rate: Some(r), .. } if r == dec!(0.9)),
        "{e:?}"
    );
}

/// Qualification d'une ligne sans provenance (V1, CR-6) et cours déduit (A32).
#[sqlx::test(migrations = "./test-schema")]
async fn ac4_rang4_qualification_et_cours_deduit(pool: MySqlPool) {
    let dec = decor(&pool).await;
    // Aucune ligne de la société : sans provenance → `manual`, jamais 404.
    let je = creer(&pool, &dec, usd_afc(&dec, dec!(0.91193), None))
        .await
        .unwrap();
    assert_eq!(
        je.lines[0].foreign().unwrap().source,
        RateProvenance::Manual
    );
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "afc",
    )
    .await;
    // Cours égal à la ligne `afc` de la société → `afc`.
    let je = creer(&pool, &dec, usd_afc(&dec, dec!(0.91193), None))
        .await
        .unwrap();
    assert_eq!(je.lines[0].foreign().unwrap().source, RateProvenance::Afc);
    // Cours différent (CHF cohérent) → `manual`, cours du client gardé.
    let mut autre = usd_afc(&dec, dec!(0.9119281363), None);
    autre.lines[0].debit = dec!(1125.83);
    let je = creer(&pool, &dec, autre).await.unwrap();
    let f = je.lines[0].foreign().unwrap();
    assert_eq!(
        (f.source, f.rate),
        (RateProvenance::Manual, dec!(0.9119281363))
    );
    // Cours déduit (marqueur) que le cours retenu redonne : le cours retenu, `afc`.
    let mut deduit = usd_afc(&dec, dec!(0.9119281363), None);
    deduit.lines[0].foreign.as_mut().unwrap().rate_derived = true;
    let je = creer(&pool, &dec, deduit).await.unwrap();
    let f = je.lines[0].foreign().unwrap();
    assert_eq!((f.source, f.rate), (RateProvenance::Afc, dec!(0.91193)));
    // Même ligne annoncée `afc` : idem.
    let mut deduit_afc = usd_afc(&dec, dec!(0.9119281363), Some(RateProvenance::Afc));
    deduit_afc.lines[0].foreign.as_mut().unwrap().rate_derived = true;
    let je = creer(&pool, &dec, deduit_afc).await.unwrap();
    assert_eq!(je.lines[0].foreign().unwrap().rate, dec!(0.91193));
    // Cours déduit `manual` : jamais repris (CR6-P3-3).
    let mut deduit_manual = usd_afc(&dec, dec!(0.9119281363), Some(RateProvenance::Manual));
    deduit_manual.lines[0]
        .foreign
        .as_mut()
        .unwrap()
        .rate_derived = true;
    let je = creer(&pool, &dec, deduit_manual).await.unwrap();
    let f = je.lines[0].foreign().unwrap();
    assert_eq!(
        (f.source, f.rate),
        (RateProvenance::Manual, dec!(0.9119281363))
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn ac4_rang3_accord_ligne_compte(pool: MySqlPool) {
    let dec = decor(&pool).await;
    // Sans `foreign` sur un compte en devise.
    let e = creer(
        &pool,
        &dec,
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                chf_line(dec.ventes, Decimal::ZERO, dec!(10)),
                chf_line(dec.usd, dec!(10), Decimal::ZERO),
            ],
        ),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(e, DbError::AccountRequiresCurrency { line: 2 }),
        "{e:?}"
    );
    // Avec `foreign` sur un compte CHF, quelle que soit la devise annoncée —
    // `"EUR"` compris (précédence sur FX_CURRENCY_MISMATCH).
    for annoncee in [None, Some("CHF"), Some("EUR")] {
        let mut f = fx(dec!(10), dec!(0.912), Some(RateProvenance::Manual));
        f.as_mut().unwrap().declared_currency = annoncee.map(str::to_string);
        let e = creer(
            &pool,
            &dec,
            ecriture(
                &dec,
                d(2026, 3, 15),
                vec![
                    ligne_fx(dec.caisse, dec!(9.12), f),
                    chf_line(dec.ventes, Decimal::ZERO, dec!(9.12)),
                ],
            ),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(e, DbError::FxAccountInChf { line: 1 }),
            "{annoncee:?} : {e:?}"
        );
    }
    // Devise annoncée différente — aucune normalisation (`usd`).
    for annoncee in ["EUR", "usd"] {
        let mut f = fx(dec!(10), dec!(0.912), Some(RateProvenance::Manual));
        f.as_mut().unwrap().declared_currency = Some(annoncee.into());
        let e = creer(
            &pool,
            &dec,
            ecriture(
                &dec,
                d(2026, 3, 15),
                vec![
                    ligne_fx(dec.usd, dec!(9.12), f),
                    chf_line(dec.ventes, Decimal::ZERO, dec!(9.12)),
                ],
            ),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(&e, DbError::FxCurrencyMismatch { line: 1, expected, provided } if expected == "USD" && provided == annoncee),
            "{e:?}"
        );
    }
    // Devise annoncée égale → passe.
    let mut f = fx(dec!(10), dec!(0.912), Some(RateProvenance::Manual));
    f.as_mut().unwrap().declared_currency = Some("USD".into());
    creer(
        &pool,
        &dec,
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                ligne_fx(dec.usd, dec!(9.12), f),
                chf_line(dec.ventes, Decimal::ZERO, dec!(9.12)),
            ],
        ),
    )
    .await
    .expect("devise annoncée égale");
}

#[sqlx::test(migrations = "./test-schema")]
async fn ac4_rang2bis_une_seule_devise(pool: MySqlPool) {
    let dec = decor(&pool).await;
    // EUR et USD → refus, même si la ligne USD ne porte pas `foreign`
    // (précédence sur ACCOUNT_REQUIRES_CURRENCY).
    for usd_foreign in [true, false] {
        let usd = if usd_foreign {
            ligne_fx(
                dec.usd,
                dec!(9.12),
                fx(dec!(10), dec!(0.912), Some(RateProvenance::Manual)),
            )
        } else {
            chf_line(dec.usd, dec!(9.12), Decimal::ZERO)
        };
        let e = creer(
            &pool,
            &dec,
            ecriture(
                &dec,
                d(2026, 3, 15),
                vec![
                    usd,
                    NewJournalEntryLine {
                        foreign: fx(dec!(10), dec!(0.912), Some(RateProvenance::Manual)),
                        ..chf_line(dec.eur, Decimal::ZERO, dec!(9.12))
                    },
                ],
            ),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(&e, DbError::FxMultipleCurrencies { currencies } if currencies == &["EUR".to_string(), "USD".to_string()]),
            "{e:?}"
        );
    }
    // EUR–EUR (deux comptes EUR, l'un `afc`, l'autre `manual`) → passe.
    cours(
        &pool,
        dec.s.company_id,
        "EUR",
        d(2026, 3, 1),
        dec!(0.95),
        "afc",
    )
    .await;
    creer(
        &pool,
        &dec,
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                ligne_fx(
                    dec.eur,
                    dec!(9.50),
                    fx(dec!(10), dec!(0.95), Some(RateProvenance::Afc)),
                ),
                NewJournalEntryLine {
                    foreign: fx(dec!(10), dec!(0.96), Some(RateProvenance::Manual)),
                    ..chf_line(dec.eur2, Decimal::ZERO, dec!(9.60))
                },
                chf_line(dec.ventes, dec!(0.10), Decimal::ZERO),
            ],
        ),
    )
    .await
    .expect("EUR–EUR");
    // USD–CHF et CHF–USD.
    creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Manual)),
    )
    .await
    .expect("USD–CHF");
}

/// Rang 2 — une ligne sur un compte en devise dans un exercice mono-devise,
/// avec ou sans `foreign` → 409 `MULTI_CURRENCY_DISABLED`, `fiscalYearId`.
#[sqlx::test(migrations = "./test-schema")]
async fn ac4_rang2_exercice_mono_devise(pool: MySqlPool) {
    let dec = decor(&pool).await;
    sqlx::query("UPDATE fiscal_years SET multi_currency = FALSE WHERE id = ?")
        .bind(dec.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    for new in [
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Manual)),
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                chf_line(dec.usd, dec!(10), Decimal::ZERO),
                chf_line(dec.ventes, Decimal::ZERO, dec!(10)),
            ],
        ),
    ] {
        let e = creer(&pool, &dec, new).await.unwrap_err();
        assert!(
            matches!(e, DbError::MultiCurrencyDisabled { fiscal_year_id: Some(id) } if id == dec.s.fiscal_year_id),
            "{e:?}"
        );
    }
}

/// Exemption de l'ouverture (rang 2 bis) : `create_opening_entry` à deux
/// devises passe ; le même corps par `create` est refusé.
#[sqlx::test(migrations = "./test-schema")]
async fn ac4_ouverture_a_deux_devises_exemptee(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let corps = || {
        ecriture(
            &dec,
            d(2020, 1, 1),
            vec![
                ligne_fx(
                    dec.usd,
                    dec!(9.12),
                    fx(dec!(10), dec!(0.912), Some(RateProvenance::Manual)),
                ),
                ligne_fx(
                    dec.eur,
                    dec!(9.50),
                    fx(dec!(10), dec!(0.95), Some(RateProvenance::Manual)),
                ),
                chf_line(dec.s.accounts["2000"], Decimal::ZERO, dec!(18.62)),
            ],
        )
    };
    let e = creer(&pool, &dec, corps()).await.unwrap_err();
    assert!(matches!(e, DbError::FxMultipleCurrencies { .. }), "{e:?}");
    let je = journal_entries::create_opening_entry(
        &pool,
        dec.s.company_id,
        dec.s.fiscal_year_id,
        dec.s.admin_user_id,
        corps(),
    )
    .await
    .expect("ouverture à deux devises");
    // Sa contre-passation passe aussi (exemptée).
    let today = d(2026, 3, 15);
    journal_entries::reverse(
        &pool,
        dec.s.company_id,
        je.entry.id,
        Acteur::utilisateur(dec.s.admin_user_id),
        today,
    )
    .await
    .expect("contre-passation d'une ouverture à deux devises");
}

// ---------------------------------------------------------------------------
// AC-7 — `PUT`, contre-passation, réévaluation
// ---------------------------------------------------------------------------

async fn modifier(
    pool: &MySqlPool,
    dec: &Decor,
    je: &kesh_db::entities::JournalEntryWithLines,
    new: NewJournalEntry,
) -> Result<kesh_db::entities::JournalEntryWithLines, DbError> {
    journal_entries::update(
        pool,
        dec.s.company_id,
        je.entry.id,
        je.entry.version,
        Acteur::utilisateur(dec.s.admin_user_id),
        new,
    )
    .await
}

async fn audits_maj(pool: &MySqlPool, id: i64) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'journal_entry.updated' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// A17, U1 — après la reprise d'un cours corrigé, le `PUT` du seul libellé
/// (lignes inchangées, même dans un autre ordre) passe ; une ligne `afc`
/// modifiée à l'ancien cours est refusée ; la même passée `manual` passe ; un
/// `PUT` identique sans `rateSource` est sans effet ; ne changer que la
/// provenance écrit et audite.
#[sqlx::test(migrations = "./test-schema")]
async fn ac7_put_apres_reprise_d_un_cours(pool: MySqlPool) {
    let dec = decor(&pool).await;
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "afc",
    )
    .await;
    let je = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap();
    // Reprise d'un cours corrigé.
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91194),
        "afc",
    )
    .await;
    // Identique sans `rateSource` → sans effet (aucune version, aucun audit).
    let meme = modifier(&pool, &dec, &je, usd_afc(&dec, dec!(0.91193), None))
        .await
        .unwrap();
    assert_eq!(meme.entry.version, je.entry.version, "no-op");
    assert_eq!(audits_maj(&pool, je.entry.id).await, 0);
    // Libellé seul, lignes dans un autre ordre → passe, `afc` gardé.
    let mut corps = usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc));
    corps.description = "Libellé changé".into();
    corps.lines.reverse();
    let apres = modifier(&pool, &dec, &je, corps)
        .await
        .expect("libellé seul");
    let f = apres
        .lines
        .iter()
        .find_map(|l| l.foreign())
        .expect("ligne en devise");
    assert_eq!((f.source, f.rate), (RateProvenance::Afc, dec!(0.91193)));
    // Ne changer que la provenance (afc → manual, CHF identique) écrit et audite.
    let avant = audits_maj(&pool, je.entry.id).await;
    let mut seule_source = usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Manual));
    seule_source.description = "Libellé changé".into();
    let ecrite = modifier(&pool, &dec, &apres, seule_source).await.unwrap();
    assert_eq!(ecrite.entry.version, apres.entry.version + 1);
    assert_eq!(audits_maj(&pool, je.entry.id).await, avant + 1);
    assert_eq!(
        ecrite.lines[0].foreign().unwrap().source,
        RateProvenance::Manual
    );
    // Ligne `afc` modifiée (montant) à l'ancien cours → refus, cours attendu.
    let mut modifiee = usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc));
    modifiee.lines[0].foreign.as_mut().unwrap().amount = dec!(100);
    modifiee.lines[0].debit = dec!(91.19);
    modifiee.lines[1].credit = dec!(91.19);
    let e = modifier(&pool, &dec, &ecrite, modifiee.clone())
        .await
        .unwrap_err();
    assert!(
        matches!(e, DbError::FxRateNotAfc { expected_rate: Some(r), .. } if r == dec!(0.91194)),
        "{e:?}"
    );
    // La même en `manual` → passe.
    modifiee.lines[0].foreign.as_mut().unwrap().source = Some(RateProvenance::Manual);
    let manuelle = modifier(&pool, &dec, &ecrite, modifiee.clone())
        .await
        .expect("manual");
    // Une ligne `manual` repassée `afc` au même (ancien) cours → refus.
    modifiee.lines[0].foreign.as_mut().unwrap().source = Some(RateProvenance::Afc);
    let e = modifier(&pool, &dec, &manuelle, modifiee)
        .await
        .unwrap_err();
    assert!(matches!(e, DbError::FxRateNotAfc { .. }), "{e:?}");
}

/// CR-6 — une ligne sans provenance renvoyée telle que lue garde sa
/// provenance stockée (date inchangée) ; date changée → qualifiée au mois neuf.
#[sqlx::test(migrations = "./test-schema")]
async fn ac7_put_sans_provenance_garde_ou_requalifie(pool: MySqlPool) {
    let dec = decor(&pool).await;
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "afc",
    )
    .await;
    let je = creer(&pool, &dec, usd_afc(&dec, dec!(0.91193), None))
        .await
        .unwrap();
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91194),
        "afc",
    )
    .await;
    let mut libelle = usd_afc(&dec, dec!(0.91193), None);
    libelle.description = "Libellé".into();
    let apres = modifier(&pool, &dec, &je, libelle).await.unwrap();
    assert_eq!(
        apres.lines[0].foreign().unwrap().source,
        RateProvenance::Afc,
        "gardée"
    );
    // Date changée (avril, sans cours retenu) → qualifiée `manual`.
    let mut avril = usd_afc(&dec, dec!(0.91193), None);
    avril.entry_date = d(2026, 4, 15);
    let apres2 = modifier(&pool, &dec, &apres, avril).await.unwrap();
    assert_eq!(
        apres2.lines[0].foreign().unwrap().source,
        RateProvenance::Manual
    );
}

/// CR6-P3-1 — ligne d'avant `afc` portée deux fois (une `afc`, une sans
/// provenance) : un seul appariement, quel que soit le rang des copies.
#[sqlx::test(migrations = "./test-schema")]
async fn ac7_put_un_seul_appariement(pool: MySqlPool) {
    let dec = decor(&pool).await;
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "afc",
    )
    .await;
    for ordre_inverse in [false, true] {
        let je = creer(
            &pool,
            &dec,
            usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
        )
        .await
        .unwrap();
        cours(
            &pool,
            dec.s.company_id,
            "USD",
            d(2026, 3, 1),
            dec!(0.91194),
            "afc",
        )
        .await;
        let mut lignes = vec![
            ligne_fx(
                dec.usd,
                dec!(1125.83),
                fx(dec!(1234.56), dec!(0.91193), Some(RateProvenance::Afc)),
            ),
            ligne_fx(
                dec.usd,
                dec!(1125.83),
                fx(dec!(1234.56), dec!(0.91193), None),
            ),
        ];
        if ordre_inverse {
            lignes.reverse();
        }
        lignes.push(chf_line(dec.ventes, Decimal::ZERO, dec!(2251.66)));
        let apres = modifier(&pool, &dec, &je, ecriture(&dec, d(2026, 3, 15), lignes))
            .await
            .expect("copie qualifiée, non reprise");
        let sources: Vec<RateProvenance> = apres
            .lines
            .iter()
            .filter_map(|l| l.foreign())
            .map(|f| f.source)
            .collect();
        assert!(sources.contains(&RateProvenance::Afc));
        assert!(sources.contains(&RateProvenance::Manual), "{sources:?}");
        cours(
            &pool,
            dec.s.company_id,
            "USD",
            d(2026, 3, 1),
            dec!(0.91193),
            "afc",
        )
        .await;
    }
}

/// U2 — changer le compte d'une ligne au `PUT`.
#[sqlx::test(migrations = "./test-schema")]
async fn ac7_changer_le_compte_d_une_ligne(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let manual = |a: Decimal, r: Decimal| fx(a, r, Some(RateProvenance::Manual));
    let je = creer(
        &pool,
        &dec,
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                ligne_fx(dec.usd, dec!(9.12), manual(dec!(10), dec!(0.912))),
                chf_line(dec.ventes, Decimal::ZERO, dec!(9.12)),
            ],
        ),
    )
    .await
    .unwrap();
    let corps = |compte: i64, f: Option<ForeignAmount>| {
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                ligne_fx(compte, dec!(9.12), f),
                chf_line(dec.ventes, Decimal::ZERO, dec!(9.12)),
            ],
        )
    };
    // USD → EUR en annonçant encore USD → mismatch.
    let mut annonce = manual(dec!(10), dec!(0.912));
    annonce.as_mut().unwrap().declared_currency = Some("USD".into());
    let e = modifier(&pool, &dec, &je, corps(dec.eur, annonce))
        .await
        .unwrap_err();
    assert!(matches!(e, DbError::FxCurrencyMismatch { .. }), "{e:?}");
    // devise → CHF avec `foreign` → FX_ACCOUNT_IN_CHF.
    let e = modifier(
        &pool,
        &dec,
        &je,
        corps(dec.caisse, manual(dec!(10), dec!(0.912))),
    )
    .await
    .unwrap_err();
    assert!(matches!(e, DbError::FxAccountInChf { line: 1 }), "{e:?}");
    // USD → EUR avec `foreign` en EUR → passe, réponse en EUR ; le compte USD
    // quitté ne porte plus de ligne.
    let eur = modifier(
        &pool,
        &dec,
        &je,
        corps(dec.eur, manual(dec!(10), dec!(0.912))),
    )
    .await
    .unwrap();
    assert_eq!(eur.lines[0].foreign().unwrap().currency, "EUR");
    let restantes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE account_id = ?")
            .bind(dec.usd)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(restantes, 0, "verrou de devise du compte quitté levé");
    // devise → CHF sans `foreign` → la ligne redevient CHF.
    let chf = modifier(&pool, &dec, &eur, corps(dec.caisse, None))
        .await
        .unwrap();
    assert!(chf.lines[0].amount_currency.is_none() && chf.lines[0].rate_source.is_none());
    // CHF → devise sans `foreign` → ACCOUNT_REQUIRES_CURRENCY.
    let e = modifier(&pool, &dec, &chf, corps(dec.usd, None))
        .await
        .unwrap_err();
    assert!(
        matches!(e, DbError::AccountRequiresCurrency { line: 1 }),
        "{e:?}"
    );
}

/// Rang 2 bis au `PUT` : passer toutes les lignes USD sur des comptes EUR
/// passe ; en passer une seule (USD + EUR) est refusé ; une ouverture à deux
/// devises garde son `PUT` du seul libellé, mais ne reçoit pas une troisième.
#[sqlx::test(migrations = "./test-schema")]
async fn ac7_put_regle_d_une_devise(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let m = |a: Decimal, r: Decimal| fx(a, r, Some(RateProvenance::Manual));
    let je = creer(
        &pool,
        &dec,
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                ligne_fx(dec.usd, dec!(9.12), m(dec!(10), dec!(0.912))),
                ligne_fx(dec.usd2, dec!(9.12), m(dec!(10), dec!(0.912))),
                chf_line(dec.ventes, Decimal::ZERO, dec!(18.24)),
            ],
        ),
    )
    .await
    .unwrap();
    let corps = |a: i64, b: i64| {
        ecriture(
            &dec,
            d(2026, 3, 15),
            vec![
                ligne_fx(a, dec!(9.12), m(dec!(10), dec!(0.912))),
                ligne_fx(b, dec!(9.12), m(dec!(10), dec!(0.912))),
                chf_line(dec.ventes, Decimal::ZERO, dec!(18.24)),
            ],
        )
    };
    let e = modifier(&pool, &dec, &je, corps(dec.usd, dec.eur))
        .await
        .unwrap_err();
    assert!(matches!(e, DbError::FxMultipleCurrencies { .. }), "{e:?}");
    modifier(&pool, &dec, &je, corps(dec.eur, dec.eur2))
        .await
        .expect("toutes les lignes USD sur des comptes EUR");
}

/// Réévaluation : `PUT` → 409 `ENTRY_IS_FX_REVALUATION` (avant la version) ;
/// `DELETE` → passe ; motif d'écran `FxRevaluation`. Contre-passation d'une
/// réévaluation : les trois colonnes recopiées.
#[sqlx::test(migrations = "./test-schema")]
async fn ac7_reevaluation_non_modifiable_supprimable(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let reeval = || {
        ecriture(
            &dec,
            d(2026, 3, 31),
            vec![
                ligne_fx(
                    dec.usd,
                    dec!(5),
                    fx(Decimal::ZERO, dec!(0.92), Some(RateProvenance::Revaluation)),
                ),
                chf_line(dec.ventes, Decimal::ZERO, dec!(5)),
            ],
        )
    };
    let je = creer(&pool, &dec, reeval())
        .await
        .expect("réévaluation (chemin interne)");
    let f = je.lines[0].foreign().unwrap();
    assert_eq!(
        (f.amount, f.source),
        (Decimal::ZERO, RateProvenance::Revaluation)
    );
    // Le motif réel prime sur la version (version fausse).
    let e = journal_entries::update(
        &pool,
        dec.s.company_id,
        je.entry.id,
        je.entry.version + 7,
        Acteur::utilisateur(dec.s.admin_user_id),
        reeval(),
    )
    .await
    .unwrap_err();
    assert!(matches!(e, DbError::EntryIsFxRevaluation), "{e:?}");
    let mut tx = pool.begin().await.unwrap();
    let motif = journal_entries::modification_blocker(&mut tx, dec.s.company_id, je.entry.id)
        .await
        .unwrap();
    drop(tx);
    assert_eq!(motif, Some(ModificationBlocker::FxRevaluation));
    assert_eq!(motif.unwrap().code(), "ENTRY_IS_FX_REVALUATION");
    // Contre-passation : recopiée, côtés inversés, même compte.
    let inverse = journal_entries::reverse(
        &pool,
        dec.s.company_id,
        je.entry.id,
        Acteur::utilisateur(dec.s.admin_user_id),
        d(2026, 4, 1),
    )
    .await
    .expect("contre-passation d'une réévaluation");
    let l = inverse
        .lines
        .iter()
        .find(|l| l.account_id == dec.usd)
        .unwrap();
    assert_eq!(l.credit, dec!(5));
    let f = l.foreign().unwrap();
    assert_eq!(
        (f.currency.as_str(), f.amount, f.rate, f.source),
        (
            "USD",
            Decimal::ZERO,
            dec!(0.92),
            RateProvenance::Revaluation
        )
    );
    // Une seconde réévaluation se supprime.
    let je2 = creer(&pool, &dec, reeval()).await.unwrap();
    journal_entries::delete_by_id(
        &pool,
        dec.s.company_id,
        je2.entry.id,
        Acteur::utilisateur(dec.s.admin_user_id),
    )
    .await
    .expect("suppression d'une réévaluation permise");
}

/// Contre-passation d'une ligne `afc` : recopiée sur le même compte (USD),
/// exemptée de FX_RATE_NOT_AFC (le cours retenu a changé depuis) ; vers un
/// exercice du jour mono-devise → 409.
#[sqlx::test(migrations = "./test-schema")]
async fn ac7_contre_passation_d_une_ligne_en_devise(pool: MySqlPool) {
    let dec = decor(&pool).await;
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.91193),
        "afc",
    )
    .await;
    let je = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap();
    let je2 = creer(
        &pool,
        &dec,
        usd_afc(&dec, dec!(0.91193), Some(RateProvenance::Afc)),
    )
    .await
    .unwrap();
    cours(
        &pool,
        dec.s.company_id,
        "USD",
        d(2026, 3, 1),
        dec!(0.5),
        "afc",
    )
    .await;
    let inverse = journal_entries::reverse(
        &pool,
        dec.s.company_id,
        je.entry.id,
        Acteur::utilisateur(dec.s.admin_user_id),
        d(2026, 3, 20),
    )
    .await
    .expect("contre-passation exemptée de FX_RATE_NOT_AFC");
    let l = inverse
        .lines
        .iter()
        .find(|l| l.account_id == dec.usd)
        .unwrap();
    assert_eq!(l.credit, dec!(1125.83));
    let f = l.foreign().unwrap();
    assert_eq!(
        (f.currency.as_str(), f.rate, f.source),
        ("USD", dec!(0.91193), RateProvenance::Afc)
    );
    // Exercice du jour mono-devise (clos avant l'activation puis rouvert,
    // simulé) → 409.
    sqlx::query("UPDATE fiscal_years SET multi_currency = FALSE WHERE id = ?")
        .bind(dec.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let e = journal_entries::reverse(
        &pool,
        dec.s.company_id,
        je2.entry.id,
        Acteur::utilisateur(dec.s.admin_user_id),
        d(2026, 3, 20),
    )
    .await
    .unwrap_err();
    assert!(matches!(e, DbError::MultiCurrencyDisabled { .. }), "{e:?}");
}

// ---------------------------------------------------------------------------
// AC-5 — chemins automatiques
// ---------------------------------------------------------------------------

/// Un chemin automatique (règlement par compte interne) refuse un compte en
/// devise dès la pièce (400 `ACCOUNT_IN_FOREIGN_CURRENCY`) ; une écriture
/// automatique sur un compte en devise tombe sur le contrôle central
/// (`ACCOUNT_REQUIRES_CURRENCY`).
#[sqlx::test(migrations = "./test-schema")]
async fn ac5_compte_de_piece_en_devise_refuse(pool: MySqlPool) {
    let dec = decor(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let e = kesh_db::repositories::accounts::refuse_foreign_currency_accounts_in_tx(
        &mut conn,
        dec.s.company_id,
        &[dec.caisse, dec.eur, dec.usd],
    )
    .await
    .unwrap_err();
    assert!(
        matches!(&e, DbError::AccountInForeignCurrency { account_id: Some(id), currency } if *id == dec.usd && currency == "USD"),
        "premier compte en devise par identifiant : {e:?}"
    );
    kesh_db::repositories::accounts::refuse_foreign_currency_accounts_in_tx(
        &mut conn,
        dec.s.company_id,
        &[dec.caisse, dec.ventes],
    )
    .await
    .expect("comptes CHF");
}
