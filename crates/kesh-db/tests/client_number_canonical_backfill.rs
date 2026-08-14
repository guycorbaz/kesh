//! Story 22-1 (#294/#295) — tests du backfill Rust `backfill_client_number_canonical`.
//!
//! AC6 : le parc existant est repris par le mécanisme **D6** — la migration
//! `20260814000001` est du DDL pur, c'est la fonction Rust qui remplit, et
//! elle **refuse en nommant les collisions** (D5) sans rien écrire.
//!
//! Le montage : `#[sqlx::test]` applique TOUTES les migrations (colonne
//! canonique comprise), puis chaque test insère des contacts en SQL **brut**
//! avec `client_number` posé et `client_number_canonical` laissé `NULL` —
//! l'état exact d'un parc migré-pas-encore-backfillé (et d'un backup restauré
//! antérieur à la story). Passer par `repositories::contacts::create` serait
//! un contresens : il remplit la canonique, c'est précisément ce que le parc
//! legacy n'a pas.

use kesh_db::backfill::{BackfillError, backfill_client_number_canonical};
use sqlx::MySqlPool;

async fn seed_company(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Backfill SA', 'x', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("seed company")
}

async fn seed_contact(
    pool: &MySqlPool,
    company_id: i64,
    name: &str,
    client_number: Option<&str>,
    active: bool,
) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, active, client_number) \
         VALUES (?, 'Personne', ?, TRUE, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(name)
    .bind(active)
    .bind(client_number)
    .fetch_one(pool)
    .await
    .expect("seed contact")
}

async fn canonical_of(pool: &MySqlPool, id: i64) -> Option<String> {
    sqlx::query_scalar("SELECT CAST(client_number_canonical AS CHAR) FROM contacts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("read canonical")
}

/// AC6 — le cas nominal : chaque ligne du parc reçoit SA canonique, calculée
/// par la même fonction que les écritures (une seule définition, D2).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_fills_every_legacy_row(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    let a = seed_contact(&pool, company, "A", Some("CLI-1"), true).await;
    let b = seed_contact(&pool, company, "B", Some("  CL\u{00C9}-2 "), true).await;
    let c = seed_contact(&pool, company, "C", None, true).await;

    let mut conn = pool.acquire().await.unwrap();
    let written = backfill_client_number_canonical(&mut conn)
        .await
        .expect("backfill nominal");
    assert_eq!(written, 2, "deux lignes porteuses d'un numéro");
    assert_eq!(canonical_of(&pool, a).await.as_deref(), Some("cli-1"));
    assert_eq!(
        canonical_of(&pool, b).await.as_deref(),
        Some("cl\u{00E9}-2")
    );
    assert_eq!(
        canonical_of(&pool, c).await,
        None,
        "sans numéro, rien à remplir"
    );
}

/// D5 — la COLLISION refuse, nomme les DEUX fiches, et n'écrit RIEN. Le
/// rapport est l'outil de réparation : société, ids, valeurs affichées.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_refuses_and_names_collisions_without_writing(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    let a = seed_contact(&pool, company, "A", Some("CLI-1"), true).await;
    let b = seed_contact(&pool, company, "B", Some("CLI\u{200B}-1"), true).await;
    let sane = seed_contact(&pool, company, "S", Some("CLI-OK"), true).await;

    let mut conn = pool.acquire().await.unwrap();
    let err = backfill_client_number_canonical(&mut conn)
        .await
        .expect_err("la collision doit refuser");
    let BackfillError::Collisions(report) = err else {
        panic!("attendu Collisions, obtenu {err:?}");
    };
    assert_eq!(report.0.len(), 1, "un seul groupe en collision");
    let group = &report.0[0];
    assert_eq!(group.company_id, company);
    assert_eq!(group.canonical, "cli-1");
    let ids: Vec<i64> = group.contacts.iter().map(|(id, _)| *id).collect();
    assert!(
        ids.contains(&a) && ids.contains(&b),
        "les deux fiches nommées"
    );
    let message = report.to_string();
    // Le rapport rend les valeurs en `{:?}` — l'invisible y devient VISIBLE
    // (`\u{200b}` en toutes lettres), ce qui est une propriété voulue : un
    // exploitant ne peut pas réparer ce qu'il ne peut pas voir.
    assert!(
        message.contains("CLI-1") && message.contains("\\u{200b}"),
        "les valeurs affichées figurent au rapport, invisible ESCAPÉ : {message}"
    );

    // Rien n'a été écrit — pas même la ligne saine.
    for id in [a, b, sane] {
        assert_eq!(
            canonical_of(&pool, id).await,
            None,
            "aucune écriture ne doit précéder un refus (contact {id})"
        );
    }
}

/// AC6 — IDEMPOTENCE : un second appel sur base remplie n'écrit rien. C'est
/// ce qui autorise l'appel à CHAQUE boot (D6) sans autre garde.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_is_idempotent(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    seed_contact(&pool, company, "A", Some("CLI-1"), true).await;

    let mut conn = pool.acquire().await.unwrap();
    let first = backfill_client_number_canonical(&mut conn).await.unwrap();
    let second = backfill_client_number_canonical(&mut conn).await.unwrap();
    assert_eq!(first, 1);
    assert_eq!(second, 0, "second appel : zéro écriture");
}

/// D2, vacuité appliquée au parc : une valeur historique INTÉGRALEMENT
/// invisible est ramenée à `NULL` sur les DEUX colonnes — elle n'identifie
/// rien et ne doit pas squatter l'unicité. Et elle n'entre pas en collision
/// avec une autre valeur invisible : deux « vides » coexistent.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_nulls_entirely_invisible_legacy_values(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    let a = seed_contact(&pool, company, "A", Some("\u{200B}\u{FEFF}"), true).await;
    let b = seed_contact(&pool, company, "B", Some(" \u{00AD} "), true).await;

    let mut conn = pool.acquire().await.unwrap();
    backfill_client_number_canonical(&mut conn)
        .await
        .expect("deux vides coexistent, pas de collision");
    for id in [a, b] {
        let (number, canonical): (Option<String>, Option<String>) = sqlx::query_as(
            "SELECT client_number, CAST(client_number_canonical AS CHAR) FROM contacts WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            number, None,
            "valeur invisible ramenée à NULL (contact {id})"
        );
        assert_eq!(canonical, None);
    }
}

/// Les contacts ARCHIVÉS sont remplis (cohérence du parc) mais ne comptent
/// PAS comme collision : la colonne générée les exclut de la contrainte, et
/// aucune route ne réactive (§ Périmètre de la story). Un actif et un archivé
/// au même numéro coexistent donc — exactement comme avant la story.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_fills_archived_rows_without_counting_them_as_collisions(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    let active = seed_contact(&pool, company, "A", Some("CLI-1"), true).await;
    let archived = seed_contact(&pool, company, "Z", Some("cli\u{200B}-1"), false).await;

    let mut conn = pool.acquire().await.unwrap();
    let written = backfill_client_number_canonical(&mut conn)
        .await
        .expect("un archivé ne provoque pas de refus");
    assert_eq!(written, 2, "l'actif ET l'archivé sont remplis");
    assert_eq!(canonical_of(&pool, active).await.as_deref(), Some("cli-1"));
    assert_eq!(
        canonical_of(&pool, archived).await.as_deref(),
        Some("cli-1")
    );
}
