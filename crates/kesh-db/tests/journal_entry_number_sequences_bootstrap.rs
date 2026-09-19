//! Story 25-2-c (refs #381) — l'AMORÇAGE du compteur d'écritures sur une base
//! qui en porte déjà (AC 4).
//!
//! # Pourquoi ce test, alors que l'allocateur a son plancher
//!
//! `next_number_for` retient le plus grand de son compteur et de
//! `MAX(entry_number) + 1` : un amorçage manqué ne ferait donc **pas** échouer
//! la saisie. Il ferait pire, parce que muet — le compteur absent, le plancher
//! seul décide, et **supprimer la dernière écriture d'avant la migration
//! rendrait son numéro réattribuable**. C'est exactement le défaut que la
//! story ferme, rouvert pour tout le parc existant, et aucun autre test ne le
//! verrait : ceux de `journal_entries` partent d'une base où la table existe
//! déjà.
//!
//! # Montage
//!
//! Comme `audit_log_company_id_backfill.rs` : les migrations **jusqu'à juste
//! avant** celle de la 25-2-c (index résolu **par version** — garde-fou P6), les
//! écritures d'une base d'avant insérées en SQL brut, puis `MIGRATOR.run()`.
//!
//! ⛔ **Les numéros portent des TROUS** (`1, 2, 5`) : un amorçage par
//! `COUNT(*) + 1` rendrait 4 au lieu de 6 et passerait au vert sur une série
//! contiguë. Les identifiants de sociétés et d'exercices sont **désalignés**
//! pour la même raison — un `GROUP BY` sur la mauvaise colonne ne doit pas
//! tomber juste par hasard.

use sqlx::MySqlPool;

use kesh_db::repositories::journal_entry_number_sequences;

mod common;

use common::{apply_migrations_up_to, migrations_before};

/// Version de la migration sous test.
const VERSION: i64 = 20260917000001;

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

async fn seed_fiscal_year(pool: &MySqlPool, id: i64, company_id: i64, year: i32) {
    sqlx::query(
        "INSERT INTO fiscal_years (id, company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?, 'Open')",
    )
    .bind(id)
    .bind(company_id)
    .bind(format!("Exercice {year}"))
    .bind(format!("{year}-01-01"))
    .bind(format!("{year}-12-31"))
    .execute(pool)
    .await
    .expect("insert fiscal_year");
}

async fn seed_entry(pool: &MySqlPool, company_id: i64, fiscal_year_id: i64, entry_number: i64) {
    sqlx::query(
        "INSERT INTO journal_entries \
         (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-03-01', 'OD', 'Écriture d''avant la 25-2-c')",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .bind(entry_number)
    .execute(pool)
    .await
    .expect("insert journal_entry");
}

/// Tire un numéro par l'allocateur réel, dans sa propre transaction validée.
async fn allocate(pool: &MySqlPool, company_id: i64, fiscal_year_id: i64) -> i64 {
    let mut tx = pool.begin().await.expect("begin");
    let n = journal_entry_number_sequences::next_number_for(&mut tx, company_id, fiscal_year_id)
        .await
        .expect("next_number_for");
    tx.commit().await.expect("commit");
    n
}

#[sqlx::test(migrations = false)]
async fn bootstrap_seeds_counter_from_existing_entries(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        migrations_before(VERSION, "journal_entry_number_sequences"),
    )
    .await
    .expect("migrations jusqu'à la 25-2-c");

    // Assertion de montage : la table n'existe pas encore, donc c'est bien
    // `MIGRATOR.run()` ci-dessous qui la crée ET l'amorce.
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = DATABASE() AND table_name = 'journal_entry_number_sequences'",
    )
    .fetch_one(&pool)
    .await
    .expect("information_schema");
    assert_eq!(
        exists, 0,
        "montage décalé : la table existe avant la migration"
    );

    seed_company(&pool, 10).await;
    seed_company(&pool, 20).await;
    seed_fiscal_year(&pool, 110, 10, 2025).await;
    seed_fiscal_year(&pool, 111, 10, 2026).await;
    seed_fiscal_year(&pool, 112, 10, 2027).await; // sans écriture
    seed_fiscal_year(&pool, 220, 20, 2026).await;

    for n in [1, 2, 5] {
        seed_entry(&pool, 10, 110, n).await;
    }
    for n in [1, 2, 3] {
        seed_entry(&pool, 10, 111, n).await;
    }
    for n in [3, 7] {
        seed_entry(&pool, 20, 220, n).await;
    }

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    // (1) Une ligne par couple mouvementé, à `MAX + 1` — et aucune pour
    // l'exercice sans écriture.
    let rows: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT company_id, fiscal_year_id, next_number FROM journal_entry_number_sequences \
         ORDER BY company_id, fiscal_year_id",
    )
    .fetch_all(&pool)
    .await
    .expect("select sequences");
    assert_eq!(rows, vec![(10, 110, 6), (10, 111, 4), (20, 220, 8)]);

    // (2) Ce que l'amorçage protège : la DERNIÈRE écriture d'avant la migration
    // est supprimée, et la numérotation reprend APRÈS elle. Sans amorçage, le
    // plancher seul rendrait 3, puis 4, puis 5 — le numéro de l'écriture
    // supprimée, redonné à une autre au troisième tirage.
    sqlx::query("DELETE FROM journal_entries WHERE company_id = 10 AND fiscal_year_id = 110 AND entry_number = 5")
        .execute(&pool)
        .await
        .expect("delete last entry");
    assert_eq!(allocate(&pool, 10, 110).await, 6);

    // (3) L'exercice jamais mouvementé part de 1, par création à la demande.
    assert_eq!(allocate(&pool, 10, 112).await, 1);
}
