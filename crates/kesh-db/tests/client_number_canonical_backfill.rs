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
    let BackfillError::Refused(report) = err else {
        panic!("attendu Refused, obtenu {err:?}");
    };
    assert_eq!(report.collisions.len(), 1, "un seul groupe en collision");
    assert!(report.overlong.is_empty());
    let group = &report.collisions[0];
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

/// Revue passe 1 (CRITICAL) — une canonique TROP LONGUE est refusée en
/// NOMMANT la fiche, pas en erreur SQL 1406 brute : 50 ligatures `ﬁ` saisies
/// (borne brute respectée) canonisent en 100 caractères > VARCHAR(50).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_refuses_and_names_overlong_canonicals(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    let fifty_ligatures = "\u{FB01}".repeat(50);
    let culprit = seed_contact(&pool, company, "Ligatures", Some(&fifty_ligatures), true).await;
    let sane = seed_contact(&pool, company, "Sain", Some("CLI-OK"), true).await;

    let mut conn = pool.acquire().await.unwrap();
    let err = backfill_client_number_canonical(&mut conn)
        .await
        .expect_err("la canonique trop longue doit refuser");
    let BackfillError::Refused(report) = err else {
        panic!("attendu Refused, obtenu {err:?}");
    };
    assert!(report.collisions.is_empty());
    assert_eq!(report.overlong.len(), 1);
    let o = &report.overlong[0];
    assert_eq!(
        (o.company_id, o.contact_id, o.canonical_chars),
        (company, culprit, 100)
    );
    let message = report.to_string();
    assert!(
        message.contains("dépasse 50 caractères") && message.contains(&culprit.to_string()),
        "le rapport nomme la fiche et la cause : {message}"
    );
    // Rien écrit — pas même la ligne saine.
    for id in [culprit, sane] {
        assert_eq!(canonical_of(&pool, id).await, None);
    }
}

/// Revue passe 1 (HIGH) — le backfill est une RÉCONCILIATION : une canonique
/// stockée PÉRIMÉE (client_number modifié par SQL direct sans sa canonique)
/// est réparée. Sans cela, l'index d'unicité — qui ne compare que la valeur
/// stockée — laisserait renaître #294 en silence.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_repairs_a_stale_canonical(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    let id = seed_contact(&pool, company, "Stale", Some("CLI-NEW"), true).await;
    // Canonique périmée posée à la main — l'état que laisse un UPDATE SQL
    // direct de client_number.
    sqlx::query("UPDATE contacts SET client_number_canonical = 'cli-old' WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let written = backfill_client_number_canonical(&mut conn).await.unwrap();
    assert_eq!(written, 1, "la divergence est réparée");
    assert_eq!(canonical_of(&pool, id).await.as_deref(), Some("cli-new"));
    // Et l'appel suivant ne réécrit rien : la réconciliation reste idempotente.
    assert_eq!(
        backfill_client_number_canonical(&mut conn).await.unwrap(),
        0
    );
}

/// Revue passe 1 (HIGH), second volet — la DÉTECTION recalcule toujours : deux
/// fiches dont les canoniques STOCKÉES divergent mais dont les numéros
/// AFFICHÉS collisionnent sont refusées, valeurs stockées ignorées.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn backfill_detects_collisions_hidden_by_stale_canonicals(pool: MySqlPool) {
    let company = seed_company(&pool).await;
    let a = seed_contact(&pool, company, "A", Some("CLI-1"), true).await;
    let b = seed_contact(&pool, company, "B", Some("cli-1"), true).await;
    // Canoniques stockées artificiellement distinctes : l'index ne voit rien,
    // seul le recalcul du pré-scan peut attraper la collision réelle.
    for (id, fake) in [(a, "fake-a"), (b, "fake-b")] {
        sqlx::query("UPDATE contacts SET client_number_canonical = ? WHERE id = ?")
            .bind(fake)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let mut conn = pool.acquire().await.unwrap();
    let err = backfill_client_number_canonical(&mut conn)
        .await
        .expect_err("la collision réelle doit être vue malgré les stockées");
    let BackfillError::Refused(report) = err else {
        panic!("attendu Refused, obtenu {err:?}");
    };
    assert_eq!(report.collisions.len(), 1);
    let ids: Vec<i64> = report.collisions[0]
        .contacts
        .iter()
        .map(|(i, _)| *i)
        .collect();
    assert!(ids.contains(&a) && ids.contains(&b));
}
