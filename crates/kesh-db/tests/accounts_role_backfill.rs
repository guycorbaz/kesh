//! Story 14-3a — invariant « seed ≡ backfill » pour les rôles de comptes.
//!
//! # Pourquoi ce test est le filet principal de la story
//!
//! Les rôles et la postabilité d'un compte sont produits par **deux sources
//! indépendantes** :
//!
//! 1. au **seed** d'une nouvelle société — les annotations `"role"` des 3 plans
//!    JSON (`kesh-core/assets/charts/*.json`) + le calcul de
//!    [`kesh_core::chart_of_accounts::is_postable`] ;
//! 2. au **backfill** d'une base existante — la liste de numéros codée dans les
//!    `UPDATE` de la migration `20260722000001_accounts_role_postable.sql`.
//!
//! Rien n'oblige structurellement ces deux sources à concorder : elles peuvent
//! diverger silencieusement (un rôle ajouté au JSON mais pas au SQL, ou
//! l'inverse), et le symptôme n'apparaîtrait qu'en production, chez un
//! utilisateur migré, sous la forme d'un rôle manquant. Ce test compare les deux.
//!
//! # Montage — pourquoi `#[sqlx::test(migrations = false)]` est indispensable
//!
//! Sous `#[sqlx::test]` (le mode par défaut du repo), la base éphémère est créée
//! **vide**, **toutes** les migrations tournent, **puis** le test insère ses
//! données. Le backfill s'exécuterait donc systématiquement sur une table
//! `accounts` vide et ne rencontrerait jamais un plan comptable : le test ne
//! prouverait rien.
//!
//! On applique donc les migrations **situées avant celle de la 14-3a** (index
//! résolu par version, cf. [`migrations_before_role_backfill`] — surtout pas
//! `total - 1`, qui supposerait qu'elle est la dernière du dépôt), on insère le
//! plan **en SQL brut** (surtout pas via `bulk_create_from_chart`, qui binderait
//! `role`/`postable` — colonnes qui n'existent pas encore à ce stade →
//! `ERROR 1054`), puis on applique le reste des migrations pour que le backfill
//! travaille sur des données réelles.

use kesh_core::chart_of_accounts::{ChartEntry, is_postable, load_chart, parent_numbers};
use sqlx::MySqlPool;

mod common;

use common::{apply_migrations_up_to, migrations_before};

/// Nombre de migrations à appliquer pour se placer **juste avant** celle de la
/// Story 14-3a (`20260722000001_accounts_role_postable`).
///
/// Résolu **par version** via [`migrations_before`], jamais par position — cf.
/// le § « garde-fou P6 » de `tests/common/mod.rs`, dont le précédent est
/// précisément la régression subie par ce fichier.
fn migrations_before_role_backfill() -> usize {
    migrations_before(20260722000001, "accounts_role_postable")
}

/// Insère une société minimale et retourne son id.
async fn seed_company(pool: &MySqlPool, org_type: &str) -> i64 {
    sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, 'Rue du Test 1', ?, 'FR', 'FR')",
    )
    .bind(format!("Test {org_type}"))
    .bind(org_type)
    .execute(pool)
    .await
    .expect("insert company")
    .last_insert_id() as i64
}

/// Insère le plan comptable **en SQL brut**, sans `role` ni `postable` (ces
/// colonnes n'existent pas encore) — exactement l'état d'une base v0.7.0.
///
/// Ordre topologique par longueur de numéro puis numéro, identique à
/// `bulk_create_from_chart`, pour que `parent_id` soit résoluble.
async fn seed_chart_raw(pool: &MySqlPool, company_id: i64, entries: &[ChartEntry]) {
    let mut sorted: Vec<&ChartEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        a.number
            .len()
            .cmp(&b.number.len())
            .then(a.number.cmp(&b.number))
    });

    let mut number_to_id: std::collections::HashMap<&str, i64> = std::collections::HashMap::new();
    for e in sorted {
        let parent_id = e
            .parent_number
            .as_deref()
            .and_then(|pn| number_to_id.get(pn).copied());
        let id = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, parent_id) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(&e.number)
        .bind(kesh_core::chart_of_accounts::resolve_name(e, "fr"))
        .bind(e.account_type.as_str())
        .bind(parent_id)
        .execute(pool)
        .await
        .expect("insert account")
        .last_insert_id() as i64;
        number_to_id.insert(&e.number, id);
    }
}

/// Le cœur : pour chacun des 3 plans, l'état produit par le **backfill** doit
/// être identique, compte par compte, à celui qu'aurait produit le **seed**.
#[sqlx::test(migrations = false)]
async fn backfill_matches_seed_for_every_chart(pool: MySqlPool) {
    // Toutes les migrations situées AVANT celle de la Story 14-3a.
    apply_migrations_up_to(&pool, migrations_before_role_backfill())
        .await
        .expect("apply_migrations_up_to(migrations_before_role_backfill())");

    // Une société par plan, chacune avec son plan comptable inséré en SQL brut.
    let mut companies = Vec::new();
    for org in ["Pme", "Association", "Independant"] {
        let chart = load_chart(org).expect("load_chart");
        let company_id = seed_company(&pool, org).await;
        seed_chart_raw(&pool, company_id, &chart).await;
        companies.push((org, company_id, chart));
    }

    // Sanity : la colonne `role` ne doit pas encore exister.
    let has_role: Option<(String,)> = sqlx::query_as(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'accounts' AND COLUMN_NAME = 'role'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(
        has_role.is_none(),
        "le montage est cassé : `role` existe déjà avant la dernière migration"
    );

    // Applique la migration 14-3a → le backfill tourne sur des données réelles.
    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    for (org, company_id, chart) in &companies {
        let parents = parent_numbers(chart);

        let rows: Vec<(String, Option<String>, bool)> = sqlx::query_as(
            "SELECT number, role, postable FROM accounts WHERE company_id = ? ORDER BY number",
        )
        .bind(company_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(
            rows.len(),
            chart.len(),
            "{org} : le backfill ne doit ni créer ni supprimer de compte"
        );

        for (number, role, postable) in rows {
            let entry = chart
                .iter()
                .find(|e| e.number == number)
                .unwrap_or_else(|| panic!("{org} : compte {number} absent du plan"));

            // --- rôle : backfill (SQL, par numéro) == seed (annotation JSON)
            let expected_role = entry.role.map(|r| r.as_str().to_string());
            assert_eq!(
                role, expected_role,
                "{org} / compte {number} : le rôle posé par le backfill SQL diverge de \
                 l'annotation du plan JSON — synchroniser la migration et le chart"
            );

            // --- postabilité : backfill (2 UPDATE) == seed (`is_postable`)
            let expected_postable = is_postable(entry, &parents);
            assert_eq!(
                postable, expected_postable,
                "{org} / compte {number} : postabilité divergente entre backfill SQL et \
                 `is_postable` (parent d'une autre entrée ? rôle CurrentYearResult ?)"
            );
        }
    }
}

/// Un compte **archivé** portant un numéro cible ne doit PAS recevoir le rôle :
/// sinon il squatterait le rôle singleton et bloquerait son remplaçant actif via
/// `uq_accounts_company_singleton_role`.
#[sqlx::test(migrations = false)]
async fn backfill_skips_archived_accounts(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_role_backfill())
        .await
        .expect("apply_migrations_up_to(migrations_before_role_backfill())");

    let company_id = seed_company(&pool, "Pme").await;

    // Un 1100 archivé (l'ancien compte débiteurs) et un 1101 actif (son remplaçant).
    for (number, active) in [("1100", false), ("1101", true)] {
        sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, active) \
             VALUES (?, ?, 'Débiteurs', 'Asset', ?)",
        )
        .bind(company_id)
        .bind(number)
        .bind(active)
        .execute(&pool)
        .await
        .unwrap();
    }

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    let role_1100: (Option<String>,) =
        sqlx::query_as("SELECT role FROM accounts WHERE company_id = ? AND number = '1100'")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        role_1100.0, None,
        "un compte archivé ne doit pas recevoir de rôle au backfill — il bloquerait \
         son remplaçant actif via la contrainte d'unicité partielle"
    );

    // Et le remplaçant actif peut bien prendre le rôle.
    sqlx::query("UPDATE accounts SET role = 'Receivable' WHERE company_id = ? AND number = '1101'")
        .bind(company_id)
        .execute(&pool)
        .await
        .expect("le rôle Receivable doit être libre pour le compte actif 1101");
}

/// La colonne générée `singleton_role` ne doit apparaître **ni** dans le
/// manifeste de backup **ni** dans l'export CSV : elle est non-insérable, un
/// restore qui tenterait de l'écrire échouerait.
// Ce test n'exerce PAS le chemin des migrations — il vérifie qu'une colonne
// générée est absente du manifeste de backup. Il vit dans un fichier exempté
// (les deux tests de backfill à fenêtre, eux, gèrent leurs migrations), et
// l'exemption étant au grain du FICHIER, il payait les 61 migrations à
// perpétuité sans que rien ne le signale. *(Relevé en passe 1 de revue de
// code ; la spec avait tranché « reste exclu » sans distinguer ce cas.)*
#[sqlx::test(migrations = "./test-schema")]
async fn generated_column_is_excluded_from_backup(pool: MySqlPool) {
    let export = kesh_db::backup::export_table(&pool, "accounts")
        .await
        .expect("export_table accounts");

    assert!(
        !export.column_names.iter().any(|c| c == "singleton_role"),
        "singleton_role (VIRTUAL GENERATED) ne doit pas figurer dans le manifeste \
         de backup — le restore ferait un INSERT dessus. Colonnes : {:?}",
        export.column_names
    );
    // Les deux vraies colonnes, elles, doivent y être.
    assert!(export.column_names.iter().any(|c| c == "role"));
    assert!(export.column_names.iter().any(|c| c == "postable"));
}
