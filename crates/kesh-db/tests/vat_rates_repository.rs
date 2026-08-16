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

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, OrgType};
use kesh_db::repositories::{companies, vat_rates};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn sample_company(name: &str) -> NewCompany {
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
    }
}

async fn create_company_with_rates(pool: &MySqlPool, name: &str) -> i64 {
    let company = companies::create(pool, sample_company(name)).await.unwrap();
    vat_rates::seed_default_swiss_rates(pool, company.id)
        .await
        .unwrap();
    company.id
}

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
async fn find_active_by_rate_unknown_returns_none(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "CompA").await;

    // Ancien taux suisse 2018-2023, jamais seedé v0.1.
    let found = vat_rates::find_active_by_rate(&pool, company_id, &dec!(7.70))
        .await
        .unwrap();
    assert!(found.is_none());
}

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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
    // `20260428000001_vat_rates.sql` — paramètres liés via `.bind()`
    // (Pass 1 remediation #12 : pas d'interpolation de chaîne SQL).
    let backfill = |label: &str, rate: &str| {
        let label = label.to_string();
        let rate: rust_decimal::Decimal = rate.parse().unwrap();
        let pool = pool.clone();
        async move {
            sqlx::query(
                "INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to) \
                 SELECT id, ?, ?, '2024-01-01', NULL FROM companies",
            )
            .bind(label)
            .bind(rate)
            .execute(&pool)
            .await
            .unwrap();
        }
    };

    for (label, rate) in [
        ("product-vat-normal", "8.10"),
        ("product-vat-special", "3.80"),
        ("product-vat-reduced", "2.60"),
        ("product-vat-exempt", "0.00"),
    ] {
        backfill(label, rate).await;
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
        backfill(label, rate).await;
    }
    let rates_a = vat_rates::list_active_for_company(&pool, company_a.id)
        .await
        .unwrap();
    assert_eq!(rates_a.len(), 4, "backfill remains idempotent");
}

// ===========================================================================
// Story 11-1 — CRUD admin, sélection par catégorie, chevauchement, verrou.
// ===========================================================================

use chrono::NaiveDate;
use kesh_db::entities::{NewVatRate, UpdateVatRate};
use kesh_db::errors::DbError;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Helper : récupère (id, version) du taux actif d'une catégorie à une date.
async fn rate_id_version(
    pool: &MySqlPool,
    company_id: i64,
    category: &str,
    at: NaiveDate,
) -> (i64, i32) {
    let r = vat_rates::find_for_category_at_date(pool, company_id, category, at)
        .await
        .unwrap()
        .expect("rate should exist");
    (r.id, r.version)
}

#[sqlx::test(migrations = "./test-schema")]
async fn seed_sets_categories(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Cat").await;
    let rates = vat_rates::list_active_for_company(&pool, company_id)
        .await
        .unwrap();
    let mut cats: Vec<&str> = rates.iter().map(|r| r.category.as_str()).collect();
    cats.sort_unstable();
    assert_eq!(cats, vec!["exempt", "normal", "reduced", "special"]);
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_for_category_at_date_boundaries(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Bnd").await;
    // Taux normal seedé : valid_from = 2024-01-01, valid_to = NULL.
    // Veille de valid_from → absent.
    assert!(
        vat_rates::find_for_category_at_date(&pool, company_id, "normal", d(2023, 12, 31))
            .await
            .unwrap()
            .is_none()
    );
    // Jour de valid_from → présent (8.10).
    let on = vat_rates::find_for_category_at_date(&pool, company_id, "normal", d(2024, 1, 1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(on.rate, dec!(8.10));
}

#[sqlx::test(migrations = "./test-schema")]
async fn year_to_year_rate_change_keeps_continuity(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Y2Y").await;
    let (normal_id, normal_v) = rate_id_version(&pool, company_id, "normal", d(2024, 6, 1)).await;

    // 1) Clôturer l'ancien normal (valid_to = 2025-01-01).
    let mut tx = pool.begin().await.unwrap();
    vat_rates::update_for_company(
        &mut tx,
        company_id,
        normal_id,
        &UpdateVatRate {
            label: "product-vat-normal".into(),
            valid_to: Some(d(2025, 1, 1)),
            active: true,
        },
        normal_v,
    )
    .await
    .unwrap();
    // 2) Créer le nouveau normal 8.50 valid_from = 2025-01-01.
    // Le handler garantit un label non vide (défaut = catégorie) ; au niveau
    // repo on respecte ce contrat (contrainte DB `chk_vat_rates_label_not_empty`).
    vat_rates::create_for_company(
        &mut tx,
        &NewVatRate {
            company_id,
            category: "normal".into(),
            label: "normal".into(),
            rate: dec!(8.50),
            valid_from: d(2025, 1, 1),
            valid_to: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Continuité : 2024 → 8.10, 2025 → 8.50.
    assert_eq!(
        vat_rates::find_for_category_at_date(&pool, company_id, "normal", d(2024, 6, 1))
            .await
            .unwrap()
            .unwrap()
            .rate,
        dec!(8.10)
    );
    assert_eq!(
        vat_rates::find_for_category_at_date(&pool, company_id, "normal", d(2025, 6, 1))
            .await
            .unwrap()
            .unwrap()
            .rate,
        dec!(8.50)
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn overlap_detected_with_open_ended_rate(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Ovl").await;
    let mut tx = pool.begin().await.unwrap();
    // Le normal seedé est [2024-01-01, +∞). Un nouveau normal [2025-01-01, +∞) chevauche.
    let overlap =
        vat_rates::has_overlap_in_category(&mut tx, company_id, "normal", d(2025, 1, 1), None, 0)
            .await
            .unwrap();
    assert!(
        overlap,
        "open-ended seeded rate should overlap a later open-ended rate"
    );

    // Plage adjacente : si on bornait le seedé à 2025-01-01, [2025-01-01, +∞) serait adjacent (pas de chevauchement).
    // On teste l'adjacence pure : exclure le seedé (exclude_id) ne s'applique pas ici ; on teste un point distinct.
    let adj = vat_rates::has_overlap_in_category(
        &mut tx,
        company_id,
        "normal",
        d(2025, 1, 1),
        Some(d(2025, 6, 1)),
        0,
    )
    .await
    .unwrap();
    // Toujours un chevauchement car le seedé est ouvert (+∞) — confirme la sémantique +∞.
    assert!(adj);
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn adjacent_ranges_do_not_overlap(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Adj").await;
    let (normal_id, normal_v) = rate_id_version(&pool, company_id, "normal", d(2024, 6, 1)).await;

    let mut tx = pool.begin().await.unwrap();
    // Clôturer le seedé à 2025-01-01 → [2024-01-01, 2025-01-01).
    vat_rates::update_for_company(
        &mut tx,
        company_id,
        normal_id,
        &UpdateVatRate {
            label: "x".into(),
            valid_to: Some(d(2025, 1, 1)),
            active: true,
        },
        normal_v,
    )
    .await
    .unwrap();
    // Un nouveau [2025-01-01, +∞) est ADJACENT (valid_to exclusif) → pas de chevauchement.
    let overlap =
        vat_rates::has_overlap_in_category(&mut tx, company_id, "normal", d(2025, 1, 1), None, 0)
            .await
            .unwrap();
    assert!(
        !overlap,
        "adjacent ranges must not overlap (valid_to exclusive)"
    );
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn update_optimistic_lock_conflict(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Lock").await;
    let (id, version) = rate_id_version(&pool, company_id, "normal", d(2024, 6, 1)).await;

    let mut tx = pool.begin().await.unwrap();
    let res = vat_rates::update_for_company(
        &mut tx,
        company_id,
        id,
        &UpdateVatRate {
            label: "x".into(),
            valid_to: None,
            active: true,
        },
        version + 99, // stale
    )
    .await;
    assert!(matches!(res, Err(DbError::OptimisticLockConflict)));
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn deactivate_excludes_from_selection(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Deact").await;
    let (id, version) = rate_id_version(&pool, company_id, "normal", d(2024, 6, 1)).await;

    let mut tx = pool.begin().await.unwrap();
    let deact = vat_rates::deactivate_for_company(&mut tx, company_id, id, version)
        .await
        .unwrap();
    assert!(!deact.active);
    tx.commit().await.unwrap();

    // Plus sélectionnable.
    assert!(
        vat_rates::find_for_category_at_date(&pool, company_id, "normal", d(2024, 6, 1))
            .await
            .unwrap()
            .is_none()
    );
    // Mais visible dans l'historique (list_all_by_company).
    let all = vat_rates::list_all_by_company(&pool, company_id)
        .await
        .unwrap();
    assert!(all.iter().any(|r| r.id == id && !r.active));
}

#[sqlx::test(migrations = "./test-schema")]
async fn extensible_category_accepted(pool: MySqlPool) {
    let company_id = create_company_with_rates(&pool, "Ext").await;
    let mut tx = pool.begin().await.unwrap();
    // Une catégorie officielle future inconnue est acceptée (modèle extensible).
    let created = vat_rates::create_for_company(
        &mut tx,
        &NewVatRate {
            company_id,
            category: "luxe_2030".into(),
            label: "Taux luxe".into(),
            rate: dec!(12.00),
            valid_from: d(2030, 1, 1),
            valid_to: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(created.category, "luxe_2030");
    tx.commit().await.unwrap();

    let found = vat_rates::find_for_category_at_date(&pool, company_id, "luxe_2030", d(2030, 6, 1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.rate, dec!(12.00));
}
