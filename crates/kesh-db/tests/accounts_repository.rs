//! Tests d'intégration du repository `accounts` sur le schéma squash
//! (Story 15-7a1) : la variante transactionnelle de la création du plan et le
//! comptage générique sur l'exécuteur.

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, OrgType};
use kesh_db::repositories::{accounts, companies};
use sqlx::MySqlPool;

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("create company")
    .id
}

async fn count_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .expect("count")
}

/// Test 1 (AC 2) — `bulk_create_from_chart_in_tx` rend les comptes réellement
/// insérés, ne commite pas (un `rollback` de l'appelant efface tout), et une
/// liste vide rend `Ok(vec![])` sans requête, la transaction restant utilisable.
#[sqlx::test(migrations = "./test-schema")]
async fn bulk_create_from_chart_in_tx_returns_inserted_and_never_commits(pool: MySqlPool) {
    let company_id = create_company(&pool, "Plan tx").await;
    let chart = kesh_core::chart_of_accounts::load_chart("pme").expect("load chart");

    let mut tx = pool.begin().await.unwrap();
    let created = accounts::bulk_create_from_chart_in_tx(&mut tx, company_id, &chart, "fr")
        .await
        .expect("bulk_create_from_chart_in_tx");
    assert_eq!(created.len(), chart.len(), "un compte par entrée du plan");
    assert!(!created.is_empty());
    assert_eq!(
        count_in_tx(&mut tx, company_id).await,
        created.len() as i64,
        "N rendus = COUNT(*) vu dans la transaction"
    );
    assert!(created.iter().all(|a| a.company_id == company_id));
    tx.rollback().await.unwrap();

    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(after, 0, "aucun commit interne : le rollback efface tout");

    // Liste vide : court-circuit, pas de `IN ()` (erreur SQL), tx utilisable.
    let mut tx = pool.begin().await.unwrap();
    let none = accounts::bulk_create_from_chart_in_tx(&mut tx, company_id, &[], "fr")
        .await
        .expect("liste vide : Ok");
    assert!(none.is_empty());
    assert_eq!(
        count_in_tx(&mut tx, company_id).await,
        0,
        "transaction toujours utilisable"
    );
    tx.rollback().await.unwrap();
}

/// Test 2 (AC 2) — `count_by_company` générique : par `&mut *tx`, il voit
/// l'insertion non commitée de sa transaction ; par `&pool` (autre connexion),
/// il ne la voit pas. C'est la visibilité transactionnelle qui est prouvée.
#[sqlx::test(migrations = "./test-schema")]
async fn count_by_company_sees_uncommitted_rows_through_its_transaction(pool: MySqlPool) {
    let company_id = create_company(&pool, "Comptage tx").await;
    let chart = kesh_core::chart_of_accounts::load_chart("pme").expect("load chart");
    accounts::bulk_create_from_chart(&pool, company_id, &chart, "fr")
        .await
        .expect("seed chart");
    let n = accounts::count_by_company(&pool, company_id).await.unwrap();
    assert_eq!(n, chart.len() as i64);

    let mut tx = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '9999', 'Compte non commité', 'Asset')",
    )
    .bind(company_id)
    .execute(&mut *tx)
    .await
    .expect("insert non commité");

    assert_eq!(
        accounts::count_by_company(&mut *tx, company_id)
            .await
            .unwrap(),
        n + 1,
        "par la transaction : l'insertion non commitée est vue"
    );
    assert_eq!(
        accounts::count_by_company(&pool, company_id).await.unwrap(),
        n,
        "par le pool (autre connexion) : elle ne l'est pas"
    );
    tx.rollback().await.unwrap();
}

/// Revue P1 (B-3) — l'enveloppe pool `bulk_create_from_chart`, sur erreur
/// d'une insertion, annule ce qu'elle a déjà inséré et rend l'erreur d'origine.
/// Le compte qui entre en collision est le **dernier** de l'ordre topologique
/// (longueur, puis numéro) : tous les autres sont insérés avant l'échec, et
/// seul le rollback de l'enveloppe peut les effacer.
#[sqlx::test(migrations = "./test-schema")]
async fn bulk_create_from_chart_rolls_back_and_returns_original_error(pool: MySqlPool) {
    let company_id = create_company(&pool, "Plan en collision").await;
    let chart = kesh_core::chart_of_accounts::load_chart("pme").expect("load chart");
    let last = chart
        .iter()
        .max_by(|a, b| {
            a.number
                .len()
                .cmp(&b.number.len())
                .then(a.number.cmp(&b.number))
        })
        .expect("plan non vide");

    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, ?, 'Compte préexistant', 'Asset')",
    )
    .bind(company_id)
    .bind(&last.number)
    .execute(&pool)
    .await
    .expect("compte préexistant");

    let err = accounts::bulk_create_from_chart(&pool, company_id, &chart, "fr")
        .await
        .expect_err("collision sur le dernier compte du plan");
    assert!(
        matches!(err, kesh_db::errors::DbError::UniqueConstraintViolation(_)),
        "erreur d'origine attendue, obtenu {err:?}"
    );

    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(after, 1, "rollback : seul le compte préexistant subsiste");
}
