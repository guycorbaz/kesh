//! Story 24-5 (#375) — invariant **I3** du backfill des comptes de clôture.
//!
//! # Ce que ce test prouve, et pourquoi le prouver
//!
//! La migration `20260909000001_closing_accounts_not_postable` ferme les comptes
//! 9000/9100/9200 d'une base existante. Trois propriétés doivent tenir, et aucune
//! ne se déduit de la lecture du SQL :
//!
//! 1. elle ferme **exactement** ces trois comptes — pas `9`, pas `90`, qui sont
//!    déjà fermés comme parents, et surtout **aucun autre compte du plan** ;
//! 2. elle ne touche **ni `account_type`, ni `version`** — déplacer un compte de
//!    classe changerait des états financiers **passés**, en silence ;
//! 3. elle ferme aussi un compte **archivé**, contrairement au gabarit des rôles
//!    (`20260722000001:129-138`) qui porte `AND active = TRUE`. Là, la clause
//!    protège l'unicité des rôles singleton ; ici il n'y a aucun rôle, et un 9000
//!    archivé puis **réactivé** doit rester non imputable.
//!
//! # Montage
//!
//! Comme `accounts_role_backfill.rs` : on applique les migrations **jusqu'à
//! juste avant** celle de la 24-5 (index résolu **par version**, jamais par
//! position — garde-fou P6), on insère l'état d'une base d'avant, puis on
//! applique le reste pour que le backfill travaille sur des données réelles.

use sqlx::MySqlPool;
use sqlx::Row;

mod common;

use common::{apply_migrations_up_to, migrations_before};

/// Index de la migration de la 24-5, résolu **par version**.
fn migrations_before_closing_backfill() -> usize {
    migrations_before(20260909000001, "closing_accounts_not_postable")
}

async fn seed_company(pool: &MySqlPool) -> i64 {
    sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Test 24-5', 'Rue du Test 1', 'Pme', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("insert company")
    .last_insert_id() as i64
}

/// Insère un compte imputable, dans l'état d'une base **d'avant** la 24-5.
async fn seed_account(pool: &MySqlPool, company_id: i64, number: &str, active: bool) {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, postable, active) \
         VALUES (?, ?, ?, 'Expense', TRUE, ?)",
    )
    .bind(company_id)
    .bind(number)
    .bind(format!("Compte {number}"))
    .bind(active)
    .execute(pool)
    .await
    .expect("insert account");
}

async fn read(pool: &MySqlPool, company_id: i64, number: &str) -> (bool, String, i32) {
    let row = sqlx::query(
        "SELECT postable, account_type, version FROM accounts WHERE company_id = ? AND number = ?",
    )
    .bind(company_id)
    .bind(number)
    .fetch_one(pool)
    .await
    .expect("select account");
    (
        row.get::<bool, _>("postable"),
        row.get::<String, _>("account_type"),
        row.get::<i32, _>("version"),
    )
}

#[sqlx::test(migrations = false)]
async fn backfill_closes_exactly_the_three_closing_accounts(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_closing_backfill())
        .await
        .expect("migrations jusqu'à juste avant la 24-5");

    let company_id = seed_company(&pool).await;
    // Les trois cibles, plus des comptes qui NE doivent PAS bouger — dont deux
    // voisins immédiats dans la numérotation, et un compte archivé.
    for number in ["9000", "9100", "9200", "9", "90", "9300", "8900", "6000"] {
        seed_account(&pool, company_id, number, true).await;
    }
    seed_account(&pool, company_id, "9001", false).await; // archivé

    let avant: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM accounts WHERE company_id = ? AND postable = TRUE",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(avant, 9, "les neuf comptes semés sont imputables au départ");

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("le reste des migrations, dont le backfill de la 24-5");

    // (1) Exactement les trois — et le décompte se recoupe.
    for number in ["9000", "9100", "9200"] {
        let (postable, _, _) = read(&pool, company_id, number).await;
        assert!(
            !postable,
            "le compte {number} doit être fermé par le backfill"
        );
    }
    for number in ["9", "90", "9300", "8900", "6000", "9001"] {
        let (postable, _, _) = read(&pool, company_id, number).await;
        assert!(
            postable,
            "le compte {number} ne doit PAS être touché — le backfill ne connaît que \
             9000/9100/9200, et `9`/`90` ne sont fermés qu'en tant que PARENTS d'un plan réel, \
             ce que ce montage plat ne reproduit pas"
        );
    }
    let apres: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM accounts WHERE company_id = ? AND postable = TRUE",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        apres,
        avant - 3,
        "exactement trois comptes changent d'état, ni plus ni moins"
    );
}

#[sqlx::test(migrations = false)]
async fn backfill_touches_neither_account_type_nor_version(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_closing_backfill())
        .await
        .expect("migrations jusqu'à juste avant la 24-5");

    let company_id = seed_company(&pool).await;
    seed_account(&pool, company_id, "9000", true).await;
    let (_, type_avant, version_avant) = read(&pool, company_id, "9000").await;

    kesh_db::MIGRATOR.run(&pool).await.expect("migrations");

    let (postable, type_apres, version_apres) = read(&pool, company_id, "9000").await;
    assert!(!postable);
    assert_eq!(
        type_apres, type_avant,
        "le type ne doit PAS changer : déplacer le compte de classe modifierait des \
         états financiers PASSÉS, en silence — art. 958f CO"
    );
    assert_eq!(
        version_apres, version_avant,
        "`version` ne doit pas être bumpée : la postabilité est ici une conséquence \
         structurelle, pas une modification demandée par un utilisateur"
    );
}

#[sqlx::test(migrations = false)]
async fn backfill_closes_an_archived_closing_account_too(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_closing_backfill())
        .await
        .expect("migrations jusqu'à juste avant la 24-5");

    let company_id = seed_company(&pool).await;
    seed_account(&pool, company_id, "9000", false).await; // archivé

    kesh_db::MIGRATOR.run(&pool).await.expect("migrations");

    let (postable, _, _) = read(&pool, company_id, "9000").await;
    assert!(
        !postable,
        "un 9000 ARCHIVÉ doit être fermé lui aussi — sans quoi le réactiver le rendrait \
         imputable et rouvrirait le défaut. C'est pourquoi le backfill ne porte PAS le \
         `AND active = TRUE` du gabarit des rôles, dont la clause sert un tout autre but"
    );
}

#[sqlx::test(migrations = false)]
async fn backfill_is_idempotent_on_an_already_closed_account(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_closing_backfill())
        .await
        .expect("migrations jusqu'à juste avant la 24-5");

    let company_id = seed_company(&pool).await;
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, postable, active) \
         VALUES (?, '9000', 'Déjà fermé', 'Expense', FALSE, TRUE)",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .expect("insert");
    let (_, _, version_avant) = read(&pool, company_id, "9000").await;

    kesh_db::MIGRATOR.run(&pool).await.expect("migrations");

    let (postable, _, version_apres) = read(&pool, company_id, "9000").await;
    assert!(!postable);
    assert_eq!(
        version_apres, version_avant,
        "la clause `AND postable = TRUE` rend le statement point-fixe : un compte déjà \
         fermé n'est pas réécrit. C'est cette garde qui porte le verdict `yes` de \
         l'audit d'idempotence"
    );
}
