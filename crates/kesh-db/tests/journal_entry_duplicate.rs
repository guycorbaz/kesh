//! Story 53-1a (#1355) — la saisie d'une écriture ne passe pas deux fois : le
//! refus d'une écriture identique à une écriture du même jour, au dépôt
//! (`journal_entries::create` / `create_with_policy`).
//!
//! Tout se joue sur le squash `test-schema`. Les AC 1 à 7 de la fiche :
//!
//! 1. le refus et ce qu'il laisse intact ; 2. ce qui distingue deux écritures ;
//! 3. les exclusions (contre-passée, contre-passation) ; 4. la confirmation et
//!    son audit ; 5. l'ordre des refus ; 6. les flux automatiques inchangés ;
//! 7. **la concurrence**, pilotée par un `create_in_tx` réel non validé.

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
use kesh_db::repositories::{audit_log, journal_entries};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, chf_line, seed_accounting_company,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

/// La date des écritures de ces tests (dans l'exercice 2020-2030 du seed).
fn jour() -> NaiveDate {
    d(2026, 3, 2)
}

fn acteur(s: &SeededCompany) -> Acteur {
    Acteur::utilisateur(s.admin_user_id)
}

/// L'écriture de référence : 100.00 de `1000` à `1100`, libellé « Vente 1 »,
/// journal `Banque`.
fn corps(s: &SeededCompany) -> NewJournalEntry {
    NewJournalEntry {
        company_id: s.company_id,
        entry_date: jour(),
        journal: Journal::Banque,
        description: "Vente 1".into(),
        project_id: None,
        lines: vec![
            chf_line(s.accounts["1000"], dec!(100.00), Decimal::ZERO),
            chf_line(s.accounts["1100"], Decimal::ZERO, dec!(100.00)),
        ],
    }
}

async fn saisir(
    pool: &MySqlPool,
    s: &SeededCompany,
    new: NewJournalEntry,
) -> JournalEntryWithLines {
    journal_entries::create(pool, s.fiscal_year_id, acteur(s), new)
        .await
        .expect("saisie")
}

async fn tenter(
    pool: &MySqlPool,
    s: &SeededCompany,
    new: NewJournalEntry,
    policy: DuplicatePolicy,
) -> Result<JournalEntryWithLines, DbError> {
    journal_entries::create_with_policy(pool, s.fiscal_year_id, acteur(s), new, policy).await
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

fn est_doublon_de(r: &Result<JournalEntryWithLines, DbError>, id: i64) -> bool {
    matches!(r, Err(DbError::JournalEntryProbableDuplicate { existing }) if existing.id == id)
}

/// Un compte USD et l'exercice du seed passé en multi-devises (décor des
/// lignes en devise).
async fn decor_devises(pool: &MySqlPool, s: &SeededCompany) -> i64 {
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '1020', 'USD', 'Asset', 'USD')",
    )
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

fn ligne_usd(compte: i64, montant: Decimal, cours: Decimal) -> NewJournalEntryLine {
    NewJournalEntryLine {
        foreign: Some(ForeignAmount {
            declared_currency: None,
            amount: montant,
            rate: cours,
            source: Some(RateProvenance::Manual),
            rate_derived: false,
        }),
        ..chf_line(compte, dec!(90.00), Decimal::ZERO)
    }
}

// ---------------------------------------------------------------------------
// AC-1 — le refus
// ---------------------------------------------------------------------------

/// AC-1 (M1) — la seconde saisie identique est refusée, l'existante nommée ;
/// rien n'est écrit (écritures, lignes, audit).
#[sqlx::test(migrations = "./test-schema")]
async fn une_ecriture_identique_est_refusee_et_rien_n_est_ecrit(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let e = saisir(&pool, &s, corps(&s)).await;
    let (n_e, n_l, n_a) = (
        compter(&pool, "journal_entries").await,
        compter(&pool, "journal_entry_lines").await,
        compter(&pool, "audit_log").await,
    );

    let r = tenter(&pool, &s, corps(&s), DuplicatePolicy::Refuse).await;
    match &r {
        Err(DbError::JournalEntryProbableDuplicate { existing }) => {
            assert_eq!(existing.id, e.entry.id);
            assert_eq!(existing.entry_number, e.entry.entry_number);
            assert_eq!(existing.entry_date, jour());
        }
        autre => panic!("attendu JournalEntryProbableDuplicate, obtenu {autre:?}"),
    }
    assert_eq!(
        r.unwrap_err().error_code(),
        "JOURNAL_ENTRY_PROBABLE_DUPLICATE",
        "le code exposé"
    );
    assert_eq!(compter(&pool, "journal_entries").await, n_e);
    assert_eq!(compter(&pool, "journal_entry_lines").await, n_l);
    assert_eq!(compter(&pool, "audit_log").await, n_a);
}

/// AC-1 — `create` (la signature qui n'a pas changé) refuse par défaut ; la
/// plus petite écriture en identifiant est citée quand il y en a plusieurs.
#[sqlx::test(migrations = "./test-schema")]
async fn create_refuse_par_defaut_et_cite_la_plus_petite(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let e1 = saisir(&pool, &s, corps(&s)).await;
    // Une seconde identique, confirmée à dessein.
    let e2 = tenter(&pool, &s, corps(&s), DuplicatePolicy::Confirmed)
        .await
        .expect("confirmée");
    assert!(e2.entry.id > e1.entry.id);

    let r = journal_entries::create(&pool, s.fiscal_year_id, acteur(&s), corps(&s)).await;
    assert!(
        est_doublon_de(&r, e1.entry.id),
        "citée : la plus petite, {r:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-2 — ce qui distingue
// ---------------------------------------------------------------------------

/// AC-2 (M3 à M7) — chaque différence fait passer ; les lignes permutées sont
/// refusées.
#[sqlx::test(migrations = "./test-schema")]
async fn ce_qui_distingue_deux_ecritures(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let projet: i64 = sqlx::query(
        "INSERT INTO projects (company_id, code, name) VALUES (?, 'P-53', 'Projet 53')",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let e = saisir(&pool, &s, corps(&s)).await;

    // Permutation : refus (M3, ordre indifférent).
    let mut permutee = corps(&s);
    permutee.lines.reverse();
    let r = tenter(&pool, &s, permutee, DuplicatePolicy::Refuse).await;
    assert!(est_doublon_de(&r, e.entry.id), "lignes permutées : {r:?}");

    type Variante = (&'static str, Box<dyn Fn(&mut NewJournalEntry)>);
    let s2 = s.accounts.clone();
    let variantes: Vec<Variante> = vec![
        ("autre date", Box::new(|n| n.entry_date = d(2026, 3, 3))),
        (
            "autre journal (M5)",
            Box::new(|n| n.journal = Journal::Caisse),
        ),
        (
            "autre libellé, un caractère (M4)",
            Box::new(|n| n.description = "Vente 2".into()),
        ),
        (
            "autre libellé, la casse (M4)",
            Box::new(|n| n.description = "vente 1".into()),
        ),
        (
            "un montant",
            Box::new(|n| {
                n.lines[0].debit = dec!(100.01);
                n.lines[1].credit = dec!(100.01);
            }),
        ),
        (
            "un compte",
            Box::new(move |n| n.lines[1].account_id = s2["2000"]),
        ),
        (
            "un projet de ligne (M6)",
            Box::new(move |n| n.lines[0].project_id = Some(projet)),
        ),
        (
            "une ligne de plus",
            Box::new({
                let c = s.accounts["1000"];
                move |n| {
                    n.lines[0].debit = dec!(60.00);
                    n.lines.push(chf_line(c, dec!(40.00), Decimal::ZERO));
                }
            }),
        ),
    ];
    for (nom, modifier) in variantes {
        let mut n = corps(&s);
        modifier(&mut n);
        tenter(&pool, &s, n, DuplicatePolicy::Refuse)
            .await
            .unwrap_or_else(|e| panic!("« {nom} » devait passer : {e:?}"));
    }
}

/// AC-2 (M7) — le montant en devise d'une ligne distingue deux écritures de
/// même compte et de même CHF.
#[sqlx::test(migrations = "./test-schema")]
async fn le_montant_en_devise_distingue(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let usd = decor_devises(&pool, &s).await;
    let avec = |montant: Decimal, cours: Decimal| {
        let mut n = corps(&s);
        n.description = "Achat en dollars".into();
        n.lines[0] = ligne_usd(usd, montant, cours);
        n.lines[1] = chf_line(s.accounts["1100"], Decimal::ZERO, dec!(90.00));
        n
    };
    let e = saisir(&pool, &s, avec(dec!(100), dec!(0.90))).await;
    // Même compte, même CHF (90.00), montant en devise différent : passe.
    tenter(
        &pool,
        &s,
        avec(dec!(50), dec!(1.80)),
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("autre montant en devise");
    // Le même montant en devise : refusé (cours et provenance n'entrent pas).
    let r = tenter(
        &pool,
        &s,
        avec(dec!(100), dec!(0.90)),
        DuplicatePolicy::Refuse,
    )
    .await;
    assert!(est_doublon_de(&r, e.entry.id), "{r:?}");
}

/// AC-2 — une autre société n'est jamais comparée.
#[sqlx::test(migrations = "./test-schema")]
async fn une_autre_societe_n_est_jamais_comparee(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    saisir(&pool, &s, corps(&s)).await;

    let c2 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre société', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let fy2 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, '2020-2030', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(c2)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut comptes = Vec::new();
    for (num, typ) in [("1000", "Asset"), ("1100", "Asset")] {
        comptes.push(
            sqlx::query(
                "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
            )
            .bind(c2)
            .bind(num)
            .bind(format!("Compte {num}"))
            .bind(typ)
            .execute(&pool)
            .await
            .unwrap()
            .last_insert_id() as i64,
        );
    }
    let autre = NewJournalEntry {
        company_id: c2,
        entry_date: jour(),
        journal: Journal::Banque,
        description: "Vente 1".into(),
        project_id: None,
        lines: vec![
            chf_line(comptes[0], dec!(100.00), Decimal::ZERO),
            chf_line(comptes[1], Decimal::ZERO, dec!(100.00)),
        ],
    };
    journal_entries::create(&pool, fy2, acteur(&s), autre)
        .await
        .expect("la société 2 n'est pas comparée à la société 1");
}

// ---------------------------------------------------------------------------
// AC-3 — les exclusions
// ---------------------------------------------------------------------------

/// AC-3 (M8) — une écriture contre-passée ne fait pas refuser une identique.
#[sqlx::test(migrations = "./test-schema")]
async fn une_ecriture_contre_passee_est_exclue(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let e = saisir(&pool, &s, corps(&s)).await;
    journal_entries::reverse(
        &pool,
        s.company_id,
        e.entry.id,
        acteur(&s),
        crate::jour::aujourd_hui_test(),
    )
    .await
    .expect("contre-passation");

    tenter(&pool, &s, corps(&s), DuplicatePolicy::Refuse)
        .await
        .expect("l'origine contre-passée n'est pas comparée");
}

/// AC-3 (M9) — une contre-passation ne fait pas refuser une écriture qui lui
/// est identique (date, journal OD, libellé, lignes inversées).
#[sqlx::test(migrations = "./test-schema")]
async fn une_contre_passation_est_exclue(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let e = saisir(&pool, &s, corps(&s)).await;
    let r = journal_entries::reverse(
        &pool,
        s.company_id,
        e.entry.id,
        acteur(&s),
        crate::jour::aujourd_hui_test(),
    )
    .await
    .expect("contre-passation");

    let identique_a_la_contre_passation = NewJournalEntry {
        company_id: s.company_id,
        entry_date: r.entry.entry_date,
        journal: r.entry.journal,
        description: r.entry.description.clone(),
        project_id: None,
        lines: r
            .lines
            .iter()
            .map(|l| chf_line(l.account_id, l.debit, l.credit))
            .collect(),
    };
    tenter(
        &pool,
        &s,
        identique_a_la_contre_passation,
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("la contre-passation n'est pas comparée");
}

// ---------------------------------------------------------------------------
// AC-4 — la confirmation et son audit
// ---------------------------------------------------------------------------

/// AC-4 (M10) — `Confirmed` enregistre et inscrit `journal_entry.duplicate_confirmed`
/// dans la même transaction, sur l'écriture créée, avec l'existante en détail.
#[sqlx::test(migrations = "./test-schema")]
async fn la_confirmation_enregistre_et_s_inscrit_a_l_audit(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let e = saisir(&pool, &s, corps(&s)).await;
    let n = tenter(&pool, &s, corps(&s), DuplicatePolicy::Confirmed)
        .await
        .expect("confirmée");

    assert_eq!(
        compter_audit(&pool, "journal_entry.duplicate_confirmed").await,
        1
    );
    let trace = audit_log::find_by_entity(&pool, "journal_entry", n.entry.id, 10)
        .await
        .unwrap()
        .into_iter()
        .find(|t| t.action == "journal_entry.duplicate_confirmed")
        .expect("trace sur l'écriture créée");
    assert_eq!(trace.user_id, s.admin_user_id);
    let details = trace.details_json.expect("détails");
    assert_eq!(details["journalEntryId"], e.entry.id);
    assert_eq!(details["entryNumber"], e.entry.entry_number);
}

/// AC-4 (M11) — `Confirmed` sans doublon n'écrit rien de plus.
#[sqlx::test(migrations = "./test-schema")]
async fn la_confirmation_sans_doublon_n_ecrit_pas_d_audit(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    tenter(&pool, &s, corps(&s), DuplicatePolicy::Confirmed)
        .await
        .expect("première saisie, confirmée pour rien");
    assert_eq!(
        compter_audit(&pool, "journal_entry.duplicate_confirmed").await,
        0
    );
    assert_eq!(compter_audit(&pool, "journal_entry.created").await, 1);
}

// ---------------------------------------------------------------------------
// AC-5 — l'ordre des refus (D6) : le doublon vient après tous les autres
// ---------------------------------------------------------------------------

/// AC-5 (M12) — identique **et** période verrouillée : `PeriodLocked`.
#[sqlx::test(migrations = "./test-schema")]
async fn la_periode_verrouillee_precede_le_doublon(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    saisir(&pool, &s, corps(&s)).await;
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(d(2026, 3, 31))
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let r = tenter(&pool, &s, corps(&s), DuplicatePolicy::Refuse).await;
    assert!(matches!(r, Err(DbError::PeriodLocked { .. })), "{r:?}");
}

/// AC-5 (M12b) — identique **et** compte non imputable : `AccountsNotPostable`.
#[sqlx::test(migrations = "./test-schema")]
async fn le_compte_non_imputable_precede_le_doublon(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    saisir(&pool, &s, corps(&s)).await;
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(s.accounts["1000"])
        .execute(&pool)
        .await
        .unwrap();
    let r = tenter(&pool, &s, corps(&s), DuplicatePolicy::Refuse).await;
    assert!(matches!(r, Err(DbError::AccountsNotPostable(_))), "{r:?}");
}

/// AC-5 (M12c) — identique **et** exercice clos (après la première écriture) :
/// `FiscalYearClosed`.
#[sqlx::test(migrations = "./test-schema")]
async fn l_exercice_clos_precede_le_doublon(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    saisir(&pool, &s, corps(&s)).await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let r = tenter(&pool, &s, corps(&s), DuplicatePolicy::Refuse).await;
    assert!(matches!(r, Err(DbError::FiscalYearClosed)), "{r:?}");
}

// ---------------------------------------------------------------------------
// AC-6 — les flux automatiques ne changent pas de règle
// ---------------------------------------------------------------------------

/// AC-6 (M13) — `create_in_tx` ne compare pas : deux écritures identiques
/// passent.
#[sqlx::test(migrations = "./test-schema")]
async fn create_in_tx_ne_compare_pas(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    for _ in 0..2 {
        let mut tx = pool.begin().await.unwrap();
        journal_entries::create_in_tx(&mut tx, s.fiscal_year_id, acteur(&s), corps(&s), true)
            .await
            .expect("flux automatique");
        tx.commit().await.unwrap();
    }
    assert_eq!(compter(&pool, "journal_entries").await, 2);
}

// ---------------------------------------------------------------------------
// AC-7 — la concurrence, pilotée
// ---------------------------------------------------------------------------

/// AC-7 (M2, M12c) — A crée E par un `create_in_tx` **réel** dans F, sans
/// valider ; B (`Refuse`, identique) part dans une tâche et **attend le verrou
/// de F** ; A valide ; B est refusée citant E.
///
/// ⛔ **Tue** « une lecture ordinaire avant le `FOR UPDATE` de l'exercice » (la
/// borne `books_locked_through` lue à l'étape 0-bis) : l'instantané
/// `REPEATABLE READ` de B naîtrait avant le `COMMIT` de A, la comparaison ne
/// verrait pas E, et B écrirait une seconde écriture. Tue aussi la comparaison
/// placée avant l'étape 1.
#[sqlx::test(migrations = "./test-schema")]
async fn une_saisie_concurrente_attend_l_exercice_puis_voit_l_ecriture(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();

    let mut a = pool.begin().await.unwrap();
    let e = journal_entries::create_in_tx(&mut a, s.fiscal_year_id, acteur(&s), corps(&s), true)
        .await
        .expect("écriture de A, non validée");

    let b = {
        let pool = pool.clone();
        let (fy, uid, new) = (s.fiscal_year_id, s.admin_user_id, corps(&s));
        tokio::spawn(async move {
            journal_entries::create_with_policy(
                &pool,
                fy,
                Acteur::utilisateur(uid),
                new,
                DuplicatePolicy::Refuse,
            )
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["multi_currency", "FOR UPDATE"], || b.is_finished())
            .await,
        "B doit attendre le verrou de l'exercice"
    );
    a.commit().await.unwrap();

    let r = b.await.unwrap();
    assert!(
        est_doublon_de(&r, e.entry.id),
        "B doit voir l'écriture de A après avoir attendu, obtenu {r:?}"
    );
    assert_eq!(
        compter(&pool, "journal_entries").await,
        1,
        "rien d'écrit par B"
    );
}
