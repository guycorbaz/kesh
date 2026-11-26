//! Story 53-1b (#1365) — la conversion de devises ne passe pas deux fois : le
//! refus d'une conversion dont une écriture est identique à une écriture du
//! même jour, au dépôt (`journal_entry_conversions::create_conversion_with_policy`).
//!
//! Tout se joue sur le squash `test-schema`. Les AC 1 à 6 de la fiche :
//!
//! 1. le refus (rang `entry` = rang **dans le corps** de la première écriture
//!    identique, dans l'ordre chronologique) et ce qu'il laisse intact ;
//! 2. une seule écriture identique suffit (Q7) ; 3. la confirmation et son
//!    audit (une entrée par écriture levée) ; 4. **la concurrence sur deux
//!    exercices**, pilotée par un `create_in_tx` réel non validé ; 5. la
//!    précédence des refus inchangée ; 6. la conversion de prestataire
//!    inchangée (aucune comparaison, aucun pré-verrou).
//!
//! Le décor : exercice F1 (2020 → 2026-12-31) et F2 (2027), tous deux
//! multi-devises ; comptes EUR, USD et de conversion (CHF) désignés.

/// Story 35-5a0 — la date du jour de l'horloge de test.
#[allow(dead_code)]
#[path = "support/jour.rs"]
mod jour;

use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    ForeignAmount, JournalEntryWithLines, NewJournalEntry, NewJournalEntryLine, RateProvenance,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::duplicate_policy::DuplicatePolicy;
use kesh_db::repositories::journal_entries;
use kesh_db::repositories::journal_entry_conversions::{
    ConversionError, ConversionErrorKind, ConversionLeg, ConversionPlan,
    create_conversion_with_policy, create_provider_conversion_in_tx,
};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, chf_line, seed_accounting_company,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

type Resultat = Result<[JournalEntryWithLines; 2], ConversionError>;

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

struct Decor {
    s: SeededCompany,
    /// Second exercice (2027), multi-devises.
    f2: i64,
    eur: i64,
    usd: i64,
    transit: i64,
}

async fn compte(pool: &MySqlPool, c: i64, numero: &str, type_: &str, devise: Option<&str>) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(c)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn monter(pool: &MySqlPool) -> Decor {
    let s = seed_accounting_company(pool).await.unwrap();
    let c = s.company_id;
    let gain = compte(pool, c, "6840", "Revenue", None).await;
    let loss = compte(pool, c, "6940", "Expense", None).await;
    let transit = compte(pool, c, "1090", "Asset", None).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(loss)
    .bind(transit)
    .bind(c)
    .execute(pool)
    .await
    .unwrap();
    // F1 : le seed (2020-2030) est ramené à 2020-2026 ; F2 : 2027.
    sqlx::query(
        "UPDATE fiscal_years SET multi_currency = TRUE, end_date = '2026-12-31' WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    let f2 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Exercice 2027', '2027-01-01', '2027-12-31', 'Open', TRUE)",
    )
    .bind(c)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let eur = compte(pool, c, "1021", "Asset", Some("EUR")).await;
    let usd = compte(pool, c, "1022", "Asset", Some("USD")).await;
    Decor {
        s,
        f2,
        eur,
        usd,
        transit,
    }
}

fn acteur(s: &SeededCompany) -> Acteur {
    Acteur::utilisateur(s.admin_user_id)
}

fn devise(montant: Decimal, cours: Decimal) -> Option<ForeignAmount> {
    Some(ForeignAmount {
        declared_currency: None,
        amount: montant,
        rate: cours,
        source: Some(RateProvenance::Manual),
        rate_derived: false,
    })
}

/// Écriture 1 : EUR 1000 × 0.947 = 947.00 au crédit du compte EUR, débit du
/// compte de conversion.
fn ecriture_1(dc: &Decor, date: NaiveDate) -> NewJournalEntry {
    NewJournalEntry {
        company_id: dc.s.company_id,
        entry_date: date,
        journal: Journal::OD,
        description: "Conversion EUR → USD (1/2)".into(),
        project_id: None,
        lines: vec![
            chf_line(dc.transit, dec!(947.00), Decimal::ZERO),
            NewJournalEntryLine {
                foreign: devise(dec!(1000), dec!(0.947)),
                ..chf_line(dc.eur, Decimal::ZERO, dec!(947.00))
            },
        ],
    }
}

/// Écriture 2 : USD 1000 × 0.947 = 947.00 au débit du compte USD, crédit du
/// compte de conversion (équilibrée : aucune ligne de différence).
fn ecriture_2(dc: &Decor, date: NaiveDate) -> NewJournalEntry {
    NewJournalEntry {
        company_id: dc.s.company_id,
        entry_date: date,
        journal: Journal::OD,
        description: "Conversion EUR → USD (2/2)".into(),
        project_id: None,
        lines: vec![
            NewJournalEntryLine {
                foreign: devise(dec!(1000), dec!(0.947)),
                ..chf_line(dc.usd, dec!(947.00), Decimal::ZERO)
            },
            chf_line(dc.transit, Decimal::ZERO, dec!(947.00)),
        ],
    }
}

async fn convertir(
    pool: &MySqlPool,
    dc: &Decor,
    entries: [NewJournalEntry; 2],
    policy: DuplicatePolicy,
) -> Resultat {
    create_conversion_with_policy(
        pool,
        dc.s.company_id,
        acteur(&dc.s),
        entries,
        false,
        50,
        policy,
    )
    .await
}

async fn compter(pool: &MySqlPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn compter_audit(pool: &MySqlPool, action: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = ?")
        .bind(action)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `r` est le refus « doublon probable » de l'écriture déjà enregistrée `id`,
/// visant l'écriture de rang `rang` du corps.
fn est_doublon_de(r: &Resultat, rang: usize, id: i64) -> bool {
    matches!(
        r,
        Err(ConversionError {
            entry: Some(e),
            kind: ConversionErrorKind::Db(DbError::JournalEntryProbableDuplicate { existing }),
        }) if *e == rang && existing.id == id
    )
}

// ---------------------------------------------------------------------------
// AC-1 — le refus
// ---------------------------------------------------------------------------

/// AC-1 (M1, M3) — le renvoi à l'identique d'une conversion passée est refusé,
/// l'écriture déjà enregistrée nommée, **rien** n'est écrit (écritures, audit).
#[sqlx::test(migrations = "./test-schema")]
async fn le_renvoi_a_l_identique_est_refuse_et_rien_n_est_ecrit(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let (d1, d2) = (d(2026, 3, 5), d(2026, 3, 6));
    let premiere = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("première conversion");
    assert_eq!(compter(&pool, "journal_entries").await, 2);

    let r = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await;
    // Dans l'ordre chronologique, la première écriture identique est celle du
    // 5 mars : rang 1 dans le corps.
    assert!(
        est_doublon_de(&r, 1, premiere[0].entry.id),
        "doublon attendu, obtenu {r:?}"
    );
    assert_eq!(compter(&pool, "journal_entries").await, 2, "rien d'écrit");
    assert_eq!(compter(&pool, "journal_entry_lines").await, 4);
    assert_eq!(compter_audit(&pool, "journal_entry.created").await, 2);
    assert_eq!(
        compter_audit(&pool, "journal_entry.duplicate_confirmed").await,
        0
    );
}

/// AC-1 (M3) — `entry` est le rang **dans le corps** de la première écriture
/// identique **dans l'ordre chronologique** : corps renversé (l'écriture du 6
/// mars en premier), c'est celle du 5 mars, de rang 2, qui est nommée.
#[sqlx::test(migrations = "./test-schema")]
async fn le_rang_est_celui_du_corps_dans_l_ordre_chronologique(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let (d1, d2) = (d(2026, 3, 5), d(2026, 3, 6));
    let premiere = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("première conversion");
    // Corps renversé : l'écriture 2 (6 mars) au rang 1, l'écriture 1 (5 mars) au rang 2.
    let r = convertir(
        &pool,
        &dc,
        [ecriture_2(&dc, d2), ecriture_1(&dc, d1)],
        DuplicatePolicy::Refuse,
    )
    .await;
    assert!(
        est_doublon_de(&r, 2, premiere[0].entry.id),
        "rang 2 (celle du 5 mars) attendu, obtenu {r:?}"
    );
    assert_eq!(compter(&pool, "journal_entries").await, 2);
}

// ---------------------------------------------------------------------------
// AC-2 — une seule écriture identique suffit (Q7) ; AC-3 — la confirmation
// ---------------------------------------------------------------------------

/// AC-2 (M7) — une conversion dont **une seule** écriture est identique à une
/// écriture existante est refusée, quelle que soit celle des deux ; la
/// confirmation la fait passer, avec **une** entrée d'audit (une écriture levée).
#[sqlx::test(migrations = "./test-schema")]
async fn une_seule_ecriture_identique_suffit(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let (d1, d2) = (d(2026, 3, 5), d(2026, 3, 6));
    let premiere = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("première conversion");

    // Seule l'écriture 1 (5 mars) est identique : la seconde est datée du 7.
    let r = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d(2026, 3, 7))],
        DuplicatePolicy::Refuse,
    )
    .await;
    assert!(
        est_doublon_de(&r, 1, premiere[0].entry.id),
        "écriture 1 identique, obtenu {r:?}"
    );

    // Seule l'écriture 2 (6 mars) est identique : la première est datée du 4,
    // donc chronologiquement la première, comparée sans refus.
    let r = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d(2026, 3, 4)), ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await;
    assert!(
        est_doublon_de(&r, 2, premiere[1].entry.id),
        "écriture 2 identique, obtenu {r:?}"
    );
    assert_eq!(compter(&pool, "journal_entries").await, 2, "rien d'écrit");

    // La confirmation la fait passer : une seule écriture levée.
    convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d(2026, 3, 4)), ecriture_2(&dc, d2)],
        DuplicatePolicy::Confirmed,
    )
    .await
    .expect("conversion confirmée");
    assert_eq!(compter(&pool, "journal_entries").await, 4);
    assert_eq!(
        compter_audit(&pool, "journal_entry.duplicate_confirmed").await,
        1
    );
}

/// AC-3 (M1) — `Confirmed` : les deux passent, une entrée
/// `journal_entry.duplicate_confirmed` **par écriture levée** (deux ici).
#[sqlx::test(migrations = "./test-schema")]
async fn la_confirmation_enregistre_et_s_inscrit_a_l_audit(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let (d1, d2) = (d(2026, 3, 5), d(2026, 3, 6));
    convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("première conversion");
    let seconde = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d2)],
        DuplicatePolicy::Confirmed,
    )
    .await
    .expect("conversion confirmée");
    assert_ne!(seconde[0].entry.id, seconde[1].entry.id);
    assert_eq!(compter(&pool, "journal_entries").await, 4);
    assert_eq!(
        compter_audit(&pool, "journal_entry.duplicate_confirmed").await,
        2
    );
    // Sans doublon, la confirmation n'écrit aucune entrée d'audit de doublon.
    convertir(
        &pool,
        &dc,
        [
            ecriture_1(&dc, d(2026, 4, 1)),
            ecriture_2(&dc, d(2026, 4, 2)),
        ],
        DuplicatePolicy::Confirmed,
    )
    .await
    .expect("conversion neuve confirmée");
    assert_eq!(
        compter_audit(&pool, "journal_entry.duplicate_confirmed").await,
        2
    );
}

// ---------------------------------------------------------------------------
// AC-4 — la concurrence, sur deux exercices
// ---------------------------------------------------------------------------

/// Conversion dont la première écriture (chronologique) est dans F1 et la
/// seconde dans F2 ; `piege` : l'écriture que A crée sans la valider (dans
/// F1 ou F2), identique à celle de la conversion.
///
/// ⚠️ Le motif d'attente (`FROM fiscal_years`, `FOR UPDATE`) est commun au
/// pré-verrou de B (`SELECT id, status FROM fiscal_years … FOR UPDATE`) et à
/// l'étape 1 de la création : il **ne distingue pas F1 de F2** et peut être vu
/// un instant sur l'exercice libre. Aucun témoin d'attente plus fin n'est
/// disponible (`INNODB_TRX` exige le privilège `PROCESS`, que `kesh` n'a pas,
/// `docs/agents-orchestration.md` § 6.1) : la preuve est **le verdict** — sous
/// M2b, B ne voit pas A (instantané antérieur) et rend `Ok`.
async fn course_deux_exercices(pool: MySqlPool, piege_dans_f2: bool) {
    let dc = monter(&pool).await;
    let (d1, d2) = (d(2026, 12, 30), d(2027, 1, 2));
    let (e1, e2) = (ecriture_1(&dc, d1), ecriture_2(&dc, d2));
    let (fy_piege, piege, rang) = if piege_dans_f2 {
        (dc.f2, e2.clone(), 2)
    } else {
        (dc.s.fiscal_year_id, e1.clone(), 1)
    };

    let mut a = pool.begin().await.unwrap();
    let existante = journal_entries::create_in_tx(&mut a, fy_piege, acteur(&dc.s), piege, true)
        .await
        .expect("écriture de A, non validée");

    let b = {
        let pool = pool.clone();
        let (company, acteur_b) = (dc.s.company_id, acteur(&dc.s));
        tokio::spawn(async move {
            create_conversion_with_policy(
                &pool,
                company,
                acteur_b,
                [e1, e2],
                false,
                50,
                DuplicatePolicy::Refuse,
            )
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM fiscal_years", "FOR UPDATE"], || {
            b.is_finished()
        })
        .await,
        "B doit attendre le verrou d'un exercice"
    );
    a.commit().await.unwrap();

    let r = b.await.unwrap();
    assert!(
        est_doublon_de(&r, rang, existante.entry.id),
        "B doit voir l'écriture de A après avoir attendu, obtenu {r:?}"
    );
    assert_eq!(
        compter(&pool, "journal_entries").await,
        1,
        "rien d'écrit par B"
    );
}

/// AC-4 (M2b, « pré-verrou remis après `conversion_line_accounts_in_tx` »,
/// M7) — l'écriture de A est identique à la **seconde** écriture (F2).
#[sqlx::test(migrations = "./test-schema")]
async fn une_conversion_concurrente_voit_l_ecriture_de_la_seconde(pool: MySqlPool) {
    course_deux_exercices(pool, true).await;
}

/// AC-4 — l'écriture de A est identique à la **première** écriture (F1).
#[sqlx::test(migrations = "./test-schema")]
async fn une_conversion_concurrente_voit_l_ecriture_de_la_premiere(pool: MySqlPool) {
    course_deux_exercices(pool, false).await;
}

// ---------------------------------------------------------------------------
// AC-5 — la précédence des refus est inchangée
// ---------------------------------------------------------------------------

/// AC-5 (M8) — une conversion datée dans un exercice **clos** et portant un
/// projet **archivé** sur une ligne rend `FISCAL_YEAR_CLOSED` de la bonne
/// écriture, non l'état illégal du projet (l'étape 0 de la création vient
/// après le jugement du statut de l'exercice).
#[sqlx::test(migrations = "./test-schema")]
async fn l_exercice_clos_precede_le_projet_archive(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let projet = sqlx::query(
        "INSERT INTO projects (company_id, code, name, archived) VALUES (?, 'ARCH', 'Archivé', TRUE)",
    )
    .bind(dc.s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(dc.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let (d1, d2) = (d(2026, 3, 5), d(2026, 3, 6));
    let mut e1 = ecriture_1(&dc, d1);
    e1.lines[0].project_id = Some(projet);
    let r = convertir(
        &pool,
        &dc,
        [e1, ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await;
    assert!(
        matches!(
            &r,
            Err(ConversionError {
                entry: Some(1),
                kind: ConversionErrorKind::FiscalYearClosed(date),
            }) if *date == d1
        ),
        "FISCAL_YEAR_CLOSED de l'écriture 1 attendu, obtenu {r:?}"
    );
    assert_eq!(compter(&pool, "journal_entries").await, 0);
}

/// AC-5 — une date sans exercice : `NO_FISCAL_YEAR` de **cette** écriture
/// (rang du corps), tout est annulé.
#[sqlx::test(migrations = "./test-schema")]
async fn une_date_sans_exercice_est_refusee_et_tout_est_annule(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let hors = d(2031, 1, 5);
    let r = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d(2026, 3, 5)), ecriture_2(&dc, hors)],
        DuplicatePolicy::Refuse,
    )
    .await;
    assert!(
        matches!(
            &r,
            Err(ConversionError {
                entry: Some(2),
                kind: ConversionErrorKind::NoFiscalYear(date),
            }) if *date == hors
        ),
        "NO_FISCAL_YEAR de l'écriture 2 attendu, obtenu {r:?}"
    );
    assert_eq!(
        compter(&pool, "journal_entries").await,
        0,
        "la première écriture est annulée avec la conversion"
    );
}

/// AC-5 — un refus propre à la conversion (compte de conversion déséquilibré)
/// **précède** le doublon : le renvoi d'une conversion passée, dont le compte
/// de conversion ne revient plus à zéro, rend le refus de conversion.
#[sqlx::test(migrations = "./test-schema")]
async fn le_refus_de_conversion_precede_le_doublon(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let (d1, d2) = (d(2026, 3, 5), d(2026, 3, 6));
    convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), ecriture_2(&dc, d2)],
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("première conversion");
    let mut e2 = ecriture_2(&dc, d2);
    e2.lines[1] = chf_line(dc.transit, Decimal::ZERO, dec!(900.00));
    let r = convertir(
        &pool,
        &dc,
        [ecriture_1(&dc, d1), e2],
        DuplicatePolicy::Refuse,
    )
    .await;
    assert!(
        matches!(
            &r,
            Err(ConversionError {
                entry: None,
                kind: ConversionErrorKind::TransitUnbalanced { .. },
            })
        ),
        "FX_CONVERSION_TRANSIT_UNBALANCED attendu avant le doublon, obtenu {r:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-6 — la conversion de prestataire est inchangée
// ---------------------------------------------------------------------------

/// AC-6 (M4) — deux conversions de prestataire identiques (transactions
/// distinctes, branche double) passent : aucune comparaison. Les conversions
/// Wise, PayPal et Stripe passent par ce chemin.
#[sqlx::test(migrations = "./test-schema")]
async fn deux_conversions_de_prestataire_identiques_passent(pool: MySqlPool) {
    let dc = monter(&pool).await;
    let (d1, d2) = (d(2026, 3, 5), d(2026, 3, 6));
    for _ in 0..2 {
        let mut tx = pool.begin().await.unwrap();
        let plan = ConversionPlan::Double {
            source: ConversionLeg {
                ledger_account_id: dc.eur,
                chf: dec!(947.00),
                foreign: devise(dec!(1000), dec!(0.947)),
            },
            target: ConversionLeg {
                ledger_account_id: dc.usd,
                chf: dec!(947.00),
                foreign: devise(dec!(1000), dec!(0.947)),
            },
        };
        let ids = create_provider_conversion_in_tx(
            &mut tx,
            dc.s.company_id,
            acteur(&dc.s),
            plan,
            d1,
            d2,
            "Conversion prestataire".into(),
        )
        .await
        .expect("conversion de prestataire");
        assert_eq!(ids.len(), 2);
        tx.commit().await.unwrap();
    }
    assert_eq!(compter(&pool, "journal_entries").await, 4);
    assert_eq!(
        compter_audit(&pool, "journal_entry.duplicate_confirmed").await,
        0
    );
}
