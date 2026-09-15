//! Story 25-1c-zero (refs #378) — le backfill de `audit_log.company_id`.
//!
//! # Ce que ce test prouve, et pourquoi le prouver
//!
//! La migration `20260915000001_audit_log_company_id` ajoute la colonne et remplit
//! les entrées existantes par la société de leur acteur. Trois propriétés doivent
//! tenir, et aucune ne se déduit de la lecture du SQL :
//!
//! 1. chaque entrée prend **la société de son acteur**, et jamais celle d'un
//!    autre — ni l'`id` de l'utilisateur, qu'un `SET … = a.user_id` fautif
//!    écrirait ;
//! 2. une entrée dont l'acteur **n'existe plus** garde `NULL` — état légitime et
//!    permanent, « société indéterminable » : ni `LEFT JOIN`, ni `COALESCE` ;
//! 3. le backfill est **point-fixe** : rejoué, il ne réécrit aucune valeur déjà
//!    posée.
//!
//! # Montage
//!
//! Comme `closing_accounts_backfill.rs` : on applique les migrations **jusqu'à
//! juste avant** celle de la 25-1c-zero (index résolu **par version**, jamais par
//! position — garde-fou P6), on insère l'état d'une base d'avant en SQL brut, puis
//! on applique le reste pour que le backfill travaille sur des données réelles.
//!
//! ⛔ **Les identifiants sont posés À LA MAIN et DÉSALIGNÉS** — sociétés `10` et
//! `20`, utilisateurs `101` et `202`. Dans une base neuve, la première société et
//! le premier utilisateur prennent tous deux l'`id` 1 : un backfill qui écrirait
//! l'`id` de l'utilisateur au lieu de sa société passerait alors au vert.

use sqlx::MySqlPool;

mod common;

use common::{apply_migrations_up_to, migrations_before};

/// Version de la migration sous test.
const VERSION: i64 = 20260915000001;

/// Index de la migration de la 25-1c-zero, résolu **par version**.
fn migrations_before_company_backfill() -> usize {
    migrations_before(VERSION, "audit_log_company_id")
}

/// Hachage factice. ⚠️ `users` porte `CHECK (OCTET_LENGTH(password_hash) >= 20)` :
/// une valeur plus courte ferait échouer le montage en `ERROR 4025`.
const FAKE_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$factice$factice-factice";

async fn seed_company(pool: &MySqlPool, id: i64) {
    sqlx::query(
        "INSERT INTO companies (id, name, address, org_type, accounting_language, instance_language) \
         VALUES (?, ?, 'Rue du Test 1', 'Pme', 'FR', 'FR')",
    )
    .bind(id)
    .bind(format!("Société {id}"))
    .execute(pool)
    .await
    .expect("insert company");
}

async fn seed_user(pool: &MySqlPool, id: i64, company_id: i64) {
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, active, company_id) \
         VALUES (?, ?, ?, 'Admin', TRUE, ?)",
    )
    .bind(id)
    .bind(format!("user_{id}"))
    .bind(FAKE_HASH)
    .bind(company_id)
    .execute(pool)
    .await
    .expect("insert user");
}

/// Insère une entrée d'audit **dans l'état d'une base d'avant** : sans
/// `company_id`, qui n'existe pas encore. `entity_id` vaut 7, distinct de tous les
/// identifiants du montage.
async fn seed_entry(pool: &MySqlPool, user_id: i64, action: &str) -> i64 {
    sqlx::query(
        "INSERT INTO audit_log (user_id, actor_label, action, entity_type, entity_id) \
         VALUES (?, 'libellé', ?, 'contact', 7)",
    )
    .bind(user_id)
    .bind(action)
    .execute(pool)
    .await
    .expect("insert audit_log")
    .last_insert_id() as i64
}

async fn company_of(pool: &MySqlPool, entry_id: i64) -> Option<i64> {
    sqlx::query_scalar("SELECT company_id FROM audit_log WHERE id = ?")
        .bind(entry_id)
        .fetch_one(pool)
        .await
        .expect("select company_id")
}

async fn company_column_exists(pool: &MySqlPool) -> bool {
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'audit_log' AND COLUMN_NAME = 'company_id'",
    )
    .fetch_one(pool)
    .await
    .expect("information_schema");
    n > 0
}

/// Monte la base d'avant, avec les deux sociétés et leurs deux utilisateurs.
///
/// ⛔ **L'assertion de montage est ici, et elle n'est pas décorative** : si la
/// colonne existait déjà, c'est que le backfill aurait tourné AVANT l'insertion
/// des données — et chaque test de ce fichier passerait à vide, le mode d'échec de
/// `backfill_skips_archived_accounts` (Story 16-1a).
async fn mount(pool: &MySqlPool) {
    apply_migrations_up_to(pool, migrations_before_company_backfill())
        .await
        .expect("migrations jusqu'à juste avant la 25-1c-zero");
    assert!(
        !company_column_exists(pool).await,
        "montage : `audit_log.company_id` ne doit PAS exister avant la migration sous test"
    );
    seed_company(pool, 10).await;
    seed_company(pool, 20).await;
    seed_user(pool, 101, 10).await;
    seed_user(pool, 202, 20).await;
}

#[sqlx::test(migrations = false)]
async fn backfill_attributes_each_entry_to_its_actors_company(pool: MySqlPool) {
    mount(&pool).await;
    let e1 = seed_entry(&pool, 101, "test.a").await;
    let e2 = seed_entry(&pool, 202, "test.b").await;
    let e3 = seed_entry(&pool, 101, "test.c").await;

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("le reste des migrations, dont le backfill de la 25-1c-zero");

    assert_eq!(
        company_of(&pool, e1).await,
        Some(10),
        "l'entrée de l'utilisateur 101 prend la société 10 — ni son propre id, ni la société de l'autre"
    );
    assert_eq!(company_of(&pool, e2).await, Some(20));
    assert_eq!(company_of(&pool, e3).await, Some(10));
}

#[sqlx::test(migrations = false)]
async fn backfill_leaves_null_when_the_actor_no_longer_exists(pool: MySqlPool) {
    mount(&pool).await;
    // Atteignable : depuis la Story 25-1a, `audit_log.user_id` n'a plus de FK.
    let orpheline = seed_entry(&pool, 999, "test.orpheline").await;
    let temoin = seed_entry(&pool, 101, "test.temoin").await;

    kesh_db::MIGRATOR.run(&pool).await.expect("migrations");

    assert_eq!(
        company_of(&pool, orpheline).await,
        None,
        "un acteur inexistant laisse `NULL` — « société indéterminable », état permanent. \
         Un `LEFT JOIN … COALESCE` y écrirait une société fictive"
    );
    assert_eq!(
        company_of(&pool, temoin).await,
        Some(10),
        "témoin : le backfill a bien tourné sur les autres lignes"
    );
}

#[sqlx::test(migrations = false)]
async fn backfill_is_idempotent_and_never_overwrites_a_set_value(pool: MySqlPool) {
    mount(&pool).await;
    let e = seed_entry(&pool, 101, "test.idempotence").await;

    kesh_db::MIGRATOR.run(&pool).await.expect("migrations");
    assert_eq!(company_of(&pool, e).await, Some(10));

    // Une valeur que le backfill ne produirait PAS pour cette entrée. Elle ne peut
    // être posée qu'APRÈS la migration : la colonne n'existe pas avant.
    sqlx::query("UPDATE audit_log SET company_id = 20 WHERE id = ?")
        .bind(e)
        .execute(&pool)
        .await
        .expect("pose d'une valeur divergente");

    // ⛔ Le SQL EMBARQUÉ dans le MIGRATOR, fichier entier — jamais une copie : une
    // copie resterait gardée quand la migration ne l'est plus, et la mutation qui
    // retire la garde passerait au vert. Le DDL rejoué est sans effet (`IF NOT
    // EXISTS`).
    let sql = &kesh_db::MIGRATOR
        .migrations
        .iter()
        .find(|m| m.version == VERSION)
        .expect("migration 20260915000001 présente dans le MIGRATOR")
        .sql;
    sqlx::raw_sql(sql)
        .execute(&pool)
        .await
        .expect("rejeu de la migration entière");

    // L'assertion porte sur la VALEUR, pas sur `rows_affected`, dont le sens dépend
    // du drapeau `CLIENT_FOUND_ROWS` de la connexion.
    assert_eq!(
        company_of(&pool, e).await,
        Some(20),
        "la garde `company_id IS NULL` rend le backfill point-fixe : une valeur déjà posée \
         n'est jamais réécrite. C'est elle qui porte le verdict `yes` de l'audit d'idempotence"
    );
}
