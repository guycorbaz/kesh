//! Helpers partagés par les tests d'intégration qui doivent observer une
//! migration **en train de s'appliquer** sur une base déjà peuplée.
//!
//! # Pourquoi ce module existe
//!
//! Une migration de **backfill** ne peut pas être testée sous `#[sqlx::test]`
//! ordinaire : ce mode crée la base éphémère vide, applique **toutes** les
//! migrations, **puis** laisse le test insérer ses données. Le backfill
//! tournerait donc systématiquement sur des tables vides et le test ne
//! prouverait rien.
//!
//! Le montage correct est en trois temps :
//!
//! 1. `#[sqlx::test(migrations = false)]` — base vide, aucune migration ;
//! 2. [`apply_migrations_up_to`] avec l'index rendu par [`migrations_before`],
//!    puis insertion des données de test **en SQL brut** ;
//! 3. `kesh_db::MIGRATOR.run()` — la migration sous test s'exécute alors sur
//!    des données réelles.
//!
//! # Le piège que [`migrations_before`] existe pour fermer (garde-fou P6)
//!
//! L'index de la fenêtre doit être résolu **par version**, jamais par position
//! (`total - 1`, `&all[..n]` calculé à la main). Un index positionnel suppose
//! une frontière qui se **décale d'un cran à chaque migration ajoutée** par une
//! story ultérieure — et le symptôme est silencieux.
//!
//! Précédent vécu, Story 16-1a : le `total - 1` de `accounts_role_backfill.rs`
//! supposait que la migration 14-3a était la dernière du dépôt. L'ajout de
//! `20260727000001_invoice_lines_revenue_account` a fait appliquer le backfill
//! **avant** l'insertion des comptes de test. Deux tests sont tombés
//! bruyamment grâce à leur assertion de montage — mais
//! `backfill_skips_archived_accounts`, qui n'en avait pas, s'est mis à
//! **passer à vide** : ses rôles ressortaient `NULL` non pas parce que le
//! backfill écartait correctement un compte archivé, mais parce qu'il ne
//! tournait plus du tout sur ces lignes. Un test muet ne détecte plus aucune
//! régression, et rien ne le signale.
//!
//! Cf. § « Migration breaking policy », garde-fou **P6** de `CLAUDE.md`.

use std::borrow::Cow;

use sqlx::MySqlPool;
use sqlx::migrate::Migrator;

/// Applique les `n` premières migrations du `MIGRATOR` (checksums réels
/// préservés — c'est bien le SQL du dépôt qui s'exécute, pas une copie).
///
/// `n` provient de [`migrations_before`] : ne jamais le calculer
/// positionnellement au call site.
pub async fn apply_migrations_up_to(
    pool: &MySqlPool,
    n: usize,
) -> Result<(), sqlx::migrate::MigrateError> {
    let all = &kesh_db::MIGRATOR.migrations;
    assert!(
        n <= all.len(),
        "fenêtre de migrations hors bornes : {n} > {} — index positionnel ?",
        all.len()
    );
    let sub = Migrator {
        migrations: Cow::Borrowed(&all[..n]),
        ignore_missing: kesh_db::MIGRATOR.ignore_missing,
        locking: kesh_db::MIGRATOR.locking,
        no_tx: kesh_db::MIGRATOR.no_tx,
    };
    sub.run(pool).await
}

/// Nombre de migrations à appliquer pour se placer **juste avant** celle de
/// version `version`, résolu **par version** dans le `MIGRATOR`.
///
/// `label` n'est utilisé que pour rendre le panic lisible si la migration a été
/// renommée ou supprimée — auquel cas le test doit échouer bruyamment plutôt
/// que de retomber sur un index plausible.
pub fn migrations_before(version: i64, label: &str) -> usize {
    kesh_db::MIGRATOR
        .migrations
        .iter()
        .position(|m| m.version == version)
        .unwrap_or_else(|| {
            panic!("migration {label} ({version}) introuvable dans le MIGRATOR — renommée ?")
        })
}
