//! Tests d'intégration pour `repositories::vat_rates` (Story 7.2 — KF-003).
//!
//! Couvre les AC #4, #5, #6, #21 du story file :
//! - `list_active_for_company` retourne les 4 taux seedés trié `rate DESC`.
//! - `find_active_by_rate` happy / scale-invariant / unknown / cross-tenant.
//! - `seed_default_swiss_rates*` idempotent.
//! - Migration backfill : sur companies pré-existantes, INSERT IGNORE pose 4 lignes.
//!
//! **Note sur le seed** : `#[sqlx::test]` applique le migrator sur une DB vide,
//! donc le bloc backfill de la migration `20260428000001_vat_rates.sql` agit
//! sur 0 company. Les tests appellent ensuite `seed_default_swiss_rates(...)`
//! après la création de la company — qui est exactement ce que `seed_demo`
//! et `finalize_onboarding` (Path A/B) font en prod.

use kesh_db::entities::{Language, NewCompany, OrgType};
use kesh_db::repositories::{companies, vat_rates};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn sample_company(name: &str) -> NewCompany {
    NewCompany {
        name: name.into(),
        address: format!("Rue {name} 1, 1000 Lausanne"),
        ide_number: None,
        org_type: OrgType::Pme,
        accounting_language: Language::Fr,
        instance_language: Language::Fr,
    }
}

async fn create_company_with_rates(pool: &MySqlPool, name: &str) -> i64 {
    let company = companies::create(pool, sample_company(name)).await.unwrap();
    vat_rates::seed_default_swiss_rates(pool, company.id)
        .await
        .unwrap();
    company.id
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_active_for_company_returns_seeded_rates_desc(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "CompA").await;

    let rates = vat_rates::list_active_for_company(&pool, company_id)
        .await
        .expect("list should succeed");

    assert_eq!(rates.len(), 4, "should have 4 seeded rates");
    // Tri DESC : 8.10, 3.80, 2.60, 0.00
    assert_eq!(rates[0].rate, dec!(8.10));
    assert_eq!(rates[1].rate, dec!(3.80));
    assert_eq!(rates[2].rate, dec!(2.60));
    assert_eq!(rates[3].rate, dec!(0.00));

    // Tous scopés à la bonne company
    for r in &rates {
        assert_eq!(r.company_id, company_id);
        assert!(r.active);
        assert!(r.valid_to.is_none());
    }

    // Labels correspondent aux clés i18n attendues
    assert_eq!(rates[0].label, "product-vat-normal");
    assert_eq!(rates[1].label, "product-vat-special");
    assert_eq!(rates[2].label, "product-vat-reduced");
    assert_eq!(rates[3].label, "product-vat-exempt");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_active_for_company_excludes_other_company(pool: MySqlPool) {
    let company_a = create_company_with_rates(&pool, "CompA").await;
    let company_b = create_company_with_rates(&pool, "CompB").await;

    let rates_a = vat_rates::list_active_for_company(&pool, company_a)
        .await
        .unwrap();
    let rates_b = vat_rates::list_active_for_company(&pool, company_b)
        .await
        .unwrap();

    assert_eq!(rates_a.len(), 4);
    assert_eq!(rates_b.len(), 4);

    // Aucune fuite cross-tenant
    for r in &rates_a {
        assert_eq!(r.company_id, company_a);
    }
    for r in &rates_b {
        assert_eq!(r.company_id, company_b);
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_active_by_rate_happy(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "CompA").await;

    let found = vat_rates::find_active_by_rate(&pool, company_id, &dec!(8.10))
        .await
        .unwrap();

    let row = found.expect("rate 8.10 should exist for the company");
    assert_eq!(row.rate, dec!(8.10));
    assert_eq!(row.label, "product-vat-normal");
    assert_eq!(row.company_id, company_id);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_active_by_rate_scale_invariant(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "CompA").await;

    // dec!(8.1) (scale 1) doit matcher la row stockée DECIMAL(5,2).
    let found_short = vat_rates::find_active_by_rate(&pool, company_id, &dec!(8.1))
        .await
        .unwrap();
    assert!(
        found_short.is_some(),
        "8.1 (scale 1) should match 8.10 stored (scale-invariant)"
    );

    let found_long = vat_rates::find_active_by_rate(&pool, company_id, &dec!(8.100))
        .await
        .unwrap();
    assert!(
        found_long.is_some(),
        "8.100 (scale 3) should match 8.10 stored"
    );

    assert_eq!(
        found_short.unwrap().id,
        found_long.unwrap().id,
        "both queries return the same row"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_active_by_rate_unknown_returns_none(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "CompA").await;

    // Ancien taux suisse 2018-2023, jamais seedé v0.1.
    let found = vat_rates::find_active_by_rate(&pool, company_id, &dec!(7.70))
        .await
        .unwrap();
    assert!(found.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_active_by_rate_other_company_returns_none(pool: MySqlPool) {
    let company_a = create_company_with_rates(&pool, "CompA").await;
    let company_b = create_company_with_rates(&pool, "CompB").await;

    // 8.10 existe pour les deux companies, mais la query est strictement scopée.
    let found = vat_rates::find_active_by_rate(&pool, company_a, &dec!(8.10))
        .await
        .unwrap();
    let row = found.unwrap();
    assert_eq!(row.company_id, company_a);
    assert_ne!(row.company_id, company_b);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn seed_default_swiss_rates_idempotent(pool: MySqlPool) {
    let company = companies::create(&pool, sample_company("CompA"))
        .await
        .unwrap();

    // Premier seed : 4 rows posées.
    vat_rates::seed_default_swiss_rates(&pool, company.id)
        .await
        .expect("first seed should succeed");
    let rates = vat_rates::list_active_for_company(&pool, company.id)
        .await
        .unwrap();
    assert_eq!(rates.len(), 4);

    // Re-seed : INSERT IGNORE → no-op, toujours 4 rows.
    vat_rates::seed_default_swiss_rates(&pool, company.id)
        .await
        .expect("re-seed should be idempotent");
    let rates = vat_rates::list_active_for_company(&pool, company.id)
        .await
        .unwrap();
    assert_eq!(rates.len(), 4, "still 4 rates after re-seed");

    // Et un troisième appel reste idempotent.
    vat_rates::seed_default_swiss_rates(&pool, company.id)
        .await
        .unwrap();
    let rates = vat_rates::list_active_for_company(&pool, company.id)
        .await
        .unwrap();
    assert_eq!(rates.len(), 4);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn migration_backfill_pattern_seeds_existing_companies(pool: MySqlPool) {
    // AC #21 — simule le pattern « companies pré-existantes au moment du
    // backfill ». Comme `#[sqlx::test]` applique le migrator sur une DB vide,
    // on rejoue ici les 4 INSERT IGNORE du backfill manuellement après
    // création de 2 companies fixture pour valider que le pattern SQL est
    // correct (toutes les companies présentes au moment du run reçoivent
    // 4 lignes).
    let company_a = companies::create(&pool, sample_company("PreExistingA"))
        .await
        .unwrap();
    let company_b = companies::create(&pool, sample_company("PreExistingB"))
        .await
        .unwrap();

    // Pattern strictement identique au bloc backfill de
    // `20260428000001_vat_rates.sql`.
    for (label, rate) in [
        ("product-vat-normal", "8.10"),
        ("product-vat-special", "3.80"),
        ("product-vat-reduced", "2.60"),
        ("product-vat-exempt", "0.00"),
    ] {
        sqlx::query(&format!(
            "INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to) \
             SELECT id, '{label}', {rate}, '2024-01-01', NULL FROM companies"
        ))
        .execute(&pool)
        .await
        .unwrap();
    }

    let rates_a = vat_rates::list_active_for_company(&pool, company_a.id)
        .await
        .unwrap();
    let rates_b = vat_rates::list_active_for_company(&pool, company_b.id)
        .await
        .unwrap();
    assert_eq!(rates_a.len(), 4);
    assert_eq!(rates_b.len(), 4);

    // Re-run du backfill : INSERT IGNORE → toujours 4 par company.
    for (label, rate) in [
        ("product-vat-normal", "8.10"),
        ("product-vat-special", "3.80"),
        ("product-vat-reduced", "2.60"),
        ("product-vat-exempt", "0.00"),
    ] {
        sqlx::query(&format!(
            "INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to) \
             SELECT id, '{label}', {rate}, '2024-01-01', NULL FROM companies"
        ))
        .execute(&pool)
        .await
        .unwrap();
    }
    let rates_a = vat_rates::list_active_for_company(&pool, company_a.id)
        .await
        .unwrap();
    assert_eq!(rates_a.len(), 4, "backfill remains idempotent");
}
