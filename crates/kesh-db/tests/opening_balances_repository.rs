//! Tests d'intégration Story 14-4 — bilan d'ouverture (soldes de départ).
//!
//! Couvre les helpers repo (`journal_entries::count_by_company`,
//! `accounts::find_types_by_ids_in_tx`) et la fn dédiée atomique
//! `journal_entries::create_opening_entry` (garde « company vierge » +
//! statut exercice sous `fiscal_years FOR UPDATE`, P1-C1/P3-BH3-1),
//! y compris la course concurrente (P1-M3-BH, miroir de
//! l'ancien `reopen_close_concurrent_is_serialized` de 14-2, course libre
//! remplacée par la Story 15-12a — test 13 a de `fiscal_years_repository.rs`).

use chrono::NaiveDate;
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    AccountRole, Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry,
    NewJournalEntryLine, NewUser, OrgType, Role,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::fiscal_years::{
    FY_OPENING_ALREADY_HAS_ENTRIES_KEY, FY_OPENING_FIRST_YEAR_CLOSED_KEY,
    FY_OPENING_NON_BALANCE_ACCOUNT_KEY,
};
use kesh_db::repositories::{accounts, audit_log, companies, fiscal_years, journal_entries, users};
use kesh_db::test_fixtures::attendre_une_requete_en_cours;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ============================================================
// Seed helpers minimal (miroir report_aggregates.rs)
// ============================================================

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "X".into(),
                building: String::new(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_user(pool: &MySqlPool, username: &str, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$YWFhYWFhYWFhYWFhYWFhYQ$0000000000000000000000000000000000000000000".into(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_fy(pool: &MySqlPool, user_id: i64, company_id: i64, name: &str, year: i32) -> i64 {
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: name.into(),
            start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_acc(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
    role: Option<AccountRole>,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id: None,
            role,
            postable: true,
            currency: None,
            own_fx_accounts: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// Écriture d'ouverture standard : Débit actif / Crédit report à-nouveau.
fn opening_entry(
    company_id: i64,
    entry_date: NaiveDate,
    asset_id: i64,
    retained_id: i64,
) -> NewJournalEntry {
    NewJournalEntry {
        company_id,
        entry_date,
        journal: Journal::OD,
        description: "Bilan d'ouverture".into(),
        project_id: None,
        lines: vec![
            NewJournalEntryLine {
                account_id: asset_id,
                debit: dec!(1000),
                credit: dec!(0),
                project_id: None,
                foreign: None,
            },
            NewJournalEntryLine {
                account_id: retained_id,
                debit: dec!(0),
                credit: dec!(1000),
                project_id: None,
                foreign: None,
            },
        ],
    }
}

// ============================================================
// journal_entries::count_by_company
// ============================================================

#[sqlx::test(migrations = "./test-schema")]
async fn count_by_company_zero_then_n(pool: MySqlPool) {
    let cid = create_company(&pool, "co-count").await;
    let uid = create_user(&pool, "u-count", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let retained = create_acc(
        &pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        0
    );

    let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    journal_entries::create(
        &pool,
        fy,
        Acteur::utilisateur(uid),
        opening_entry(cid, date, asset, retained),
    )
    .await
    .unwrap();

    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        1
    );

    // Générique `Executor` (P3-ECH-LOW-2) : appelable aussi sur `&mut *tx`.
    let mut tx = pool.begin().await.unwrap();
    assert_eq!(
        journal_entries::count_by_company(&mut *tx, cid)
            .await
            .unwrap(),
        1
    );
    tx.rollback().await.unwrap();
}

/// Le comptage est scopé company : les écritures d'une autre company ne
/// comptent pas.
#[sqlx::test(migrations = "./test-schema")]
async fn count_by_company_is_tenant_scoped(pool: MySqlPool) {
    let cid_a = create_company(&pool, "co-count-a").await;
    let cid_b = create_company(&pool, "co-count-b").await;
    let uid_b = create_user(&pool, "u-count-b", cid_b).await;
    let fy_b = create_fy(&pool, uid_b, cid_b, "FY2026", 2026).await;
    let asset_b = create_acc(
        &pool,
        uid_b,
        cid_b,
        "1000",
        "Banque",
        AccountType::Asset,
        None,
    )
    .await;
    let liab_b = create_acc(
        &pool,
        uid_b,
        cid_b,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    journal_entries::create(
        &pool,
        fy_b,
        Acteur::utilisateur(uid_b),
        opening_entry(cid_b, date, asset_b, liab_b),
    )
    .await
    .unwrap();

    assert_eq!(
        journal_entries::count_by_company(&pool, cid_a)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        journal_entries::count_by_company(&pool, cid_b)
            .await
            .unwrap(),
        1
    );
}

// ============================================================
// accounts::find_types_by_ids_in_tx
// ============================================================

#[sqlx::test(migrations = "./test-schema")]
async fn find_types_by_ids_returns_types_scoped_to_company(pool: MySqlPool) {
    let cid = create_company(&pool, "co-types").await;
    let uid = create_user(&pool, "u-types", cid).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;
    let revenue = create_acc(
        &pool,
        uid,
        cid,
        "3000",
        "Ventes",
        AccountType::Revenue,
        None,
    )
    .await;

    // Compte d'une AUTRE company : absent du résultat (pas d'erreur).
    let cid_other = create_company(&pool, "co-types-other").await;
    let uid_other = create_user(&pool, "u-types-other", cid_other).await;
    let foreign = create_acc(
        &pool,
        uid_other,
        cid_other,
        "1000",
        "Banque",
        AccountType::Asset,
        None,
    )
    .await;

    let mut tx = pool.begin().await.unwrap();
    let mut types =
        accounts::find_types_by_ids_in_tx(&mut tx, cid, &[asset, liab, revenue, foreign])
            .await
            .unwrap();
    types.sort();
    tx.rollback().await.unwrap();

    let mut expected = vec![
        (asset, "Asset".to_string()),
        (liab, "Liability".to_string()),
        (revenue, "Revenue".to_string()),
    ];
    expected.sort();
    // `foreign` (autre company) est ABSENT — il retombera dans la garde
    // `InactiveOrInvalidAccounts` de `create_in_tx` (contrat AC-B).
    assert_eq!(types, expected);
}

/// `ids` vide → `Ok(vec![])` sans requête SQL (un `IN ()` vide serait une
/// erreur de syntaxe MariaDB → 500, P3-ECH-LOW-1).
#[sqlx::test(migrations = "./test-schema")]
async fn find_types_by_ids_empty_ids_ok(pool: MySqlPool) {
    let cid = create_company(&pool, "co-types-empty").await;
    let mut tx = pool.begin().await.unwrap();
    let types = accounts::find_types_by_ids_in_tx(&mut tx, cid, &[])
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert!(types.is_empty());
}

// ============================================================
// journal_entries::create_opening_entry
// ============================================================

#[sqlx::test(migrations = "./test-schema")]
async fn create_opening_entry_happy_path_od_with_audit(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open").await;
    let uid = create_user(&pool, "u-open", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let retained = create_acc(
        &pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let created = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, fy_start, asset, retained),
    )
    .await
    .unwrap();

    assert_eq!(created.entry.journal, Journal::OD);
    assert_eq!(created.entry.entry_date, fy_start);
    assert_eq!(created.entry.fiscal_year_id, fy);
    assert_eq!(created.lines.len(), 2);
    assert_eq!(created.lines[0].debit, dec!(1000));
    assert_eq!(created.lines[1].credit, dec!(1000));

    // Audit `journal_entry.created` présent (posé par create_in_tx).
    let entries = audit_log::find_by_entity(&pool, "journal_entry", created.entry.id, 10)
        .await
        .unwrap();
    assert!(
        entries.iter().any(|e| e.action == "journal_entry.created"),
        "audit journal_entry.created attendu"
    );
}

/// Garde « company vierge » (P3-BH3-1) : une écriture existante dans un
/// AUTRE exercice bloque aussi la génération (company-wide, pas
/// premier-exercice-seulement) — aucune écriture ni audit supplémentaire.
#[sqlx::test(migrations = "./test-schema")]
async fn create_opening_entry_rejected_if_company_has_entries_in_any_fy(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-busy").await;
    let uid = create_user(&pool, "u-open-busy", cid).await;
    let fy_first = create_fy(&pool, uid, cid, "FY2025", 2025).await;
    let fy_later = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    // Écriture normale dans l'exercice POSTÉRIEUR (pas le premier).
    let d_later = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    journal_entries::create(
        &pool,
        fy_later,
        Acteur::utilisateur(uid),
        opening_entry(cid, d_later, asset, liab),
    )
    .await
    .unwrap();

    let fy_start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let err = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy_first,
        uid,
        opening_entry(cid, fy_start, asset, liab),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(&err, DbError::Invariant(k) if k == FY_OPENING_ALREADY_HAS_ENTRIES_KEY),
        "attendu Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY), obtenu {err:?}"
    );
    // Aucune 2e écriture insérée.
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        1
    );
}

/// Premier exercice `Closed` → `Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)`
/// (re-check sous le lock — le code 14-4 distinct, pas le générique
/// `FiscalYearClosed`, L5).
#[sqlx::test(migrations = "./test-schema")]
async fn create_opening_entry_rejected_if_first_year_closed(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-closed").await;
    let uid = create_user(&pool, "u-open-closed", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    fiscal_years::close(&pool, uid, cid, fy).await.unwrap();

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let err = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, fy_start, asset, liab),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(&err, DbError::Invariant(k) if k == FY_OPENING_FIRST_YEAR_CLOSED_KEY),
        "attendu Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY), obtenu {err:?}"
    );
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        0
    );
}

/// Exercice inexistant (ou d'une autre company) → `NotFound`.
#[sqlx::test(migrations = "./test-schema")]
async fn create_opening_entry_not_found_for_missing_fy(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-404").await;
    let uid = create_user(&pool, "u-open-404", cid).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let err = journal_entries::create_opening_entry(
        &pool,
        cid,
        999_999,
        uid,
        opening_entry(cid, fy_start, asset, liab),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(err, DbError::NotFound),
        "attendu NotFound, obtenu {err:?}"
    );
}

/// Course concurrente (P1-M3-BH, miroir de l'ancien
/// `reopen_close_concurrent_is_serialized` de 14-2, remplacé par la Story
/// 15-12a) : deux générations simultanées sur la même company vierge — le
/// `fiscal_years FOR UPDATE` + garde `count_by_company` sous le lock
/// sérialisent : exactement UNE écriture créée, l'autre reçoit
/// `ALREADY_HAS_ENTRIES`.
#[sqlx::test(migrations = "./test-schema")]
async fn create_opening_entry_concurrent_generation_is_serialized(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-race").await;
    let uid = create_user(&pool, "u-open-race", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let retained = create_acc(
        &pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let p1 = pool.clone();
    let p2 = pool.clone();
    let new1 = opening_entry(cid, fy_start, asset, retained);
    let new2 = opening_entry(cid, fy_start, asset, retained);

    let t1 = tokio::spawn(async move {
        journal_entries::create_opening_entry(&p1, cid, fy, uid, new1).await
    });
    let t2 = tokio::spawn(async move {
        journal_entries::create_opening_entry(&p2, cid, fy, uid, new2).await
    });

    let (r1, r2) = tokio::join!(t1, t2);
    let r1 = r1.expect("task 1 should not panic");
    let r2 = r2.expect("task 2 should not panic");

    let ok_count = [&r1, &r2].iter().filter(|r| r.is_ok()).count();
    assert_eq!(
        ok_count, 1,
        "exactement une génération doit réussir: {r1:?} / {r2:?}"
    );
    let loser = if r1.is_err() { r1 } else { r2 };
    assert!(
        matches!(&loser, Err(DbError::Invariant(k)) if k == FY_OPENING_ALREADY_HAS_ENTRIES_KEY),
        "le perdant doit recevoir ALREADY_HAS_ENTRIES, obtenu {loser:?}"
    );

    // État final : UNE seule écriture, pas de doublon.
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        1
    );
}

/// Pass 3 code review (BH3-1) : deux générations concurrentes ayant résolu des
/// PREMIERS EXERCICES DIFFÉRENTS (cas : un exercice antérieur créé entre les
/// deux pré-checks hors-lock du handler). Sans le sentinel
/// `companies FOR UPDATE`, chacune verrouillerait une ligne `fiscal_years`
/// distincte, les deux liraient `count == 0` et commiteraient → bilan DOUBLÉ.
/// Le sentinel company-wide les sérialise : exactement UN succès, l'autre
/// `ALREADY_HAS_ENTRIES`, une seule écriture en base.
#[sqlx::test(migrations = "./test-schema")]
async fn create_opening_entry_concurrent_different_first_fys_serialized(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-race2").await;
    let uid = create_user(&pool, "u-open-race2", cid).await;
    // Deux exercices : la requête A a résolu FY2026, la requête B (après
    // création de l'exercice antérieur) a résolu FY2025.
    let fy2026 = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let fy2025 = create_fy(&pool, uid, cid, "FY2025", 2025).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let retained = create_acc(
        &pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    let p1 = pool.clone();
    let p2 = pool.clone();
    let new_a = opening_entry(
        cid,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        asset,
        retained,
    );
    let new_b = opening_entry(
        cid,
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        asset,
        retained,
    );

    let ta = tokio::spawn(async move {
        journal_entries::create_opening_entry(&p1, cid, fy2026, uid, new_a).await
    });
    let tb = tokio::spawn(async move {
        journal_entries::create_opening_entry(&p2, cid, fy2025, uid, new_b).await
    });

    let (ra, rb) = tokio::join!(ta, tb);
    let ra = ra.expect("task A should not panic");
    let rb = rb.expect("task B should not panic");

    let ok_count = [&ra, &rb].iter().filter(|r| r.is_ok()).count();
    assert_eq!(
        ok_count, 1,
        "exactement une génération doit réussir (sentinel company) : {ra:?} / {rb:?}"
    );
    let loser = if ra.is_err() { ra } else { rb };
    assert!(
        matches!(&loser, Err(DbError::Invariant(k)) if k == FY_OPENING_ALREADY_HAS_ENTRIES_KEY),
        "le perdant doit recevoir ALREADY_HAS_ENTRIES, obtenu {loser:?}"
    );
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        1,
        "PAS de doublon d'ouverture"
    );
}

// ============================================================
// Story 53-3 (#513) — le type des comptes se contrôle SOUS VERROU
// ============================================================

/// Société prête : un exercice 2026, un compte `Asset` « cible » (1000), un
/// compte de report (2970). Rend `(cid, uid, fy, cible, report)`.
async fn societe_prete(pool: &MySqlPool, nom: &str) -> (i64, i64, i64, i64, i64) {
    let cid = create_company(pool, nom).await;
    let uid = create_user(pool, &format!("u-{nom}"), cid).await;
    let fy = create_fy(pool, uid, cid, "FY2026", 2026).await;
    let cible = create_acc(pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let report = create_acc(
        pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;
    (cid, uid, fy, cible, report)
}

fn debut_2026() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
}

fn est_non_balance(
    r: &Result<kesh_db::entities::journal_entry::JournalEntryWithLines, DbError>,
) -> bool {
    matches!(r, Err(DbError::Invariant(k)) if k == FY_OPENING_NON_BALANCE_ACCOUNT_KEY)
}

/// **AC-3.** Entrelacement forcé : `tenue` prend `companies` en partage comme
/// `accounts::update`, retype le compte cible, ne commite pas ;
/// `create_opening_entry` est vue **en attente sur la société** ; `tenue`
/// commite ; l'écriture doit refuser. Sans le contrôle sous verrou, l'appel
/// rend `Ok` et une ligne d'ouverture est posée sur un compte de résultat.
async fn course_retypage_sous_societe(pool: MySqlPool, nom: &str, type_cible: &str) {
    let (cid, uid, fy, cible, report) = societe_prete(&pool, nom).await;

    let mut tenue = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM companies WHERE id = ? LOCK IN SHARE MODE")
        .bind(cid)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET account_type = ? WHERE id = ?")
        .bind(type_cible)
        .bind(cible)
        .execute(&mut *tenue)
        .await
        .unwrap();

    let p = pool.clone();
    let new = opening_entry(cid, debut_2026(), cible, report);
    let tache =
        tokio::spawn(
            async move { journal_entries::create_opening_entry(&p, cid, fy, uid, new).await },
        );
    let en_attente = attendre_une_requete_en_cours(
        &pool,
        &["FROM companies WHERE id = ?", "FOR UPDATE"],
        || tache.is_finished(),
    )
    .await;
    assert!(
        en_attente,
        "create_opening_entry aurait dû attendre le sentinelle de la société"
    );
    tenue.commit().await.unwrap();

    let r = tache.await.expect("la tâche ne doit pas paniquer");
    assert!(
        est_non_balance(&r),
        "attendu Invariant(FY_OPENING_NON_BALANCE_ACCOUNT_KEY), obtenu {r:?}"
    );
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        0,
        "aucune écriture d'ouverture ne doit rester"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn course_retypage_revenue_sous_societe_est_refusee(pool: MySqlPool) {
    course_retypage_sous_societe(pool, "co-513-rev", "Revenue").await;
}

#[sqlx::test(migrations = "./test-schema")]
async fn course_retypage_expense_sous_societe_est_refusee(pool: MySqlPool) {
    course_retypage_sous_societe(pool, "co-513-exp", "Expense").await;
}

/// **AC-4.** Un écrivain qui ne prend PAS la société (`UPDATE accounts` seul,
/// non commité) : `create_opening_entry` doit attendre **sur la lecture des
/// types** (`LOCK IN SHARE MODE`), non sur celle de `validate_lines_accounts_in_tx`
/// (`currency FROM accounts`). Tue la mutation « lecture ordinaire ».
async fn course_retypage_sans_societe(pool: MySqlPool, nom: &str, type_cible: &str) {
    let (cid, uid, fy, cible, report) = societe_prete(&pool, nom).await;

    let mut tenue = pool.begin().await.unwrap();
    sqlx::query("UPDATE accounts SET account_type = ? WHERE id = ?")
        .bind(type_cible)
        .bind(cible)
        .execute(&mut *tenue)
        .await
        .unwrap();

    let p = pool.clone();
    let new = opening_entry(cid, debut_2026(), cible, report);
    let tache =
        tokio::spawn(
            async move { journal_entries::create_opening_entry(&p, cid, fy, uid, new).await },
        );
    let en_attente = attendre_une_requete_en_cours(
        &pool,
        &["account_type FROM accounts", "LOCK IN SHARE MODE"],
        || tache.is_finished(),
    )
    .await;
    assert!(
        en_attente,
        "create_opening_entry aurait dû attendre sur la lecture verrouillante des types"
    );
    tenue.commit().await.unwrap();

    let r = tache.await.expect("la tâche ne doit pas paniquer");
    assert!(
        est_non_balance(&r),
        "attendu Invariant(FY_OPENING_NON_BALANCE_ACCOUNT_KEY), obtenu {r:?}"
    );
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        0
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn lecture_des_types_attend_un_ecrivain_sans_societe_revenue(pool: MySqlPool) {
    course_retypage_sans_societe(pool, "co-513-lock-rev", "Revenue").await;
}

#[sqlx::test(migrations = "./test-schema")]
async fn lecture_des_types_attend_un_ecrivain_sans_societe_expense(pool: MySqlPool) {
    course_retypage_sans_societe(pool, "co-513-lock-exp", "Expense").await;
}

/// Refus séquentiel (sans course), les deux types : l'écriture ne s'écrit pas
/// et n'est pas auditée.
#[sqlx::test(migrations = "./test-schema")]
async fn create_opening_entry_refuse_revenue_et_expense(pool: MySqlPool) {
    for (nom, type_cible) in [("co-513-seq-rev", "Revenue"), ("co-513-seq-exp", "Expense")] {
        let (cid, uid, fy, cible, report) = societe_prete(&pool, nom).await;
        sqlx::query("UPDATE accounts SET account_type = ? WHERE id = ?")
            .bind(type_cible)
            .bind(cible)
            .execute(&pool)
            .await
            .unwrap();
        let r = journal_entries::create_opening_entry(
            &pool,
            cid,
            fy,
            uid,
            opening_entry(cid, debut_2026(), cible, report),
        )
        .await;
        assert!(est_non_balance(&r), "{type_cible} : obtenu {r:?}");
        assert_eq!(
            journal_entries::count_by_company(&pool, cid).await.unwrap(),
            0
        );
        let audits: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'journal_entry.created' \
             AND user_id = ?",
        )
        .bind(uid)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(audits, 0, "un refus n'écrit aucune ligne d'audit");
    }
}

/// **AC-5 (a)** — société avec une écriture + ligne `Revenue` : le verdict
/// « société vierge » prime sur le type.
#[sqlx::test(migrations = "./test-schema")]
async fn verdict_a_deja_des_ecritures_prime_sur_le_type(pool: MySqlPool) {
    let (cid, uid, fy, cible, report) = societe_prete(&pool, "co-513-a").await;
    let revenue = create_acc(
        &pool,
        uid,
        cid,
        "3000",
        "Ventes",
        AccountType::Revenue,
        None,
    )
    .await;
    journal_entries::create(
        &pool,
        fy,
        Acteur::utilisateur(uid),
        opening_entry(
            cid,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            cible,
            report,
        ),
    )
    .await
    .unwrap();

    let r = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, debut_2026(), cible, revenue),
    )
    .await;
    assert!(
        matches!(&r, Err(DbError::Invariant(k)) if k == FY_OPENING_ALREADY_HAS_ENTRIES_KEY),
        "attendu ALREADY_HAS_ENTRIES, obtenu {r:?}"
    );
}

/// **AC-5 (b)** — premier exercice clos (société vierge) + ligne `Revenue` :
/// le verdict « exercice clos » prime sur le type.
#[sqlx::test(migrations = "./test-schema")]
async fn verdict_b_premier_exercice_clos_prime_sur_le_type(pool: MySqlPool) {
    let (cid, uid, fy, cible, _report) = societe_prete(&pool, "co-513-b").await;
    let revenue = create_acc(
        &pool,
        uid,
        cid,
        "3000",
        "Ventes",
        AccountType::Revenue,
        None,
    )
    .await;
    fiscal_years::close(&pool, uid, cid, fy).await.unwrap();

    let r = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, debut_2026(), cible, revenue),
    )
    .await;
    assert!(
        matches!(&r, Err(DbError::Invariant(k)) if k == FY_OPENING_FIRST_YEAR_CLOSED_KEY),
        "attendu FIRST_YEAR_CLOSED, obtenu {r:?}"
    );
}

/// **AC-5 (c)** — ligne `Revenue` + une autre ligne sur un compte **archivé** :
/// le refus de type précède `INACTIVE_OR_INVALID_ACCOUNTS`.
#[sqlx::test(migrations = "./test-schema")]
async fn verdict_c_type_precede_comptes_inactifs(pool: MySqlPool) {
    let (cid, uid, fy, cible, _report) = societe_prete(&pool, "co-513-c").await;
    let revenue = create_acc(
        &pool,
        uid,
        cid,
        "3000",
        "Ventes",
        AccountType::Revenue,
        None,
    )
    .await;
    sqlx::query("UPDATE accounts SET active = 0 WHERE id = ?")
        .bind(cible)
        .execute(&pool)
        .await
        .unwrap();

    let r = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, debut_2026(), cible, revenue),
    )
    .await;
    assert!(est_non_balance(&r), "attendu NON_BALANCE, obtenu {r:?}");
}

/// **AC-5 (d)** — un compte `Revenue` **d'une autre société** : jamais
/// `NON_BALANCE` (anti-énumération KF-002), toujours `InactiveOrInvalidAccounts`.
#[sqlx::test(migrations = "./test-schema")]
async fn verdict_d_compte_d_une_autre_societe_reste_invalide(pool: MySqlPool) {
    let (cid, uid, fy, cible, _report) = societe_prete(&pool, "co-513-d").await;
    let autre = create_company(&pool, "co-513-d-autre").await;
    let uid_autre = create_user(&pool, "u-513-d-autre", autre).await;
    let etranger = create_acc(
        &pool,
        uid_autre,
        autre,
        "3000",
        "Ventes",
        AccountType::Revenue,
        None,
    )
    .await;

    let r = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, debut_2026(), cible, etranger),
    )
    .await;
    assert!(
        matches!(&r, Err(DbError::InactiveOrInvalidAccounts)),
        "attendu InactiveOrInvalidAccounts, obtenu {r:?}"
    );
}
