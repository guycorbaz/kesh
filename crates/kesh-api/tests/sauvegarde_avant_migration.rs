//! Story v0174-mise-a-jour (#847), T8 — le **binaire** refuse de migrer quand
//! la sauvegarde avant mise à jour échoue.
//!
//! Seul un lancement du binaire réel établit que `main.rs` appelle l'étape
//! **avant** `MIGRATOR.run` et s'arrête sur son refus (mutations M1, M2 de la
//! fiche) : les tests internes du module (`src/pre_upgrade_backup/tests.rs`) ne
//! voient que la valeur rendue.
//!
//! Montage : une base **partielle** (migrations jusqu'à
//! `20261027000001_mcp_prompts` exclue, résolue par version — P6), un dossier
//! de sauvegarde qui est un **fichier** (la création de `pre-upgrade/` échoue),
//! et `KESH_PORT` **occupé** : si le binaire continuait malgré le refus, il
//! migrerait la base puis échouerait à l'écoute — `_sqlx_migrations` aurait
//! changé, ce que le test voit. Fichier inscrit à `ALLOWED_REAL_MIGRATOR_FILES`.

#[path = "common/binaire.rs"]
mod binaire;
#[path = "../../kesh-db/tests/common/mod.rs"]
mod montage;

use sqlx::MySqlPool;

/// Secret JWT valide (≥ 32 octets, ni gabarit ni valeur publiée).
const SECRET: &str = "0123456789abcdef0123456789abcdef-v0174m-t8";

#[sqlx::test(migrations = false)]
async fn t8_le_binaire_refuse_de_migrer(pool: MySqlPool) {
    montage::apply_migrations_up_to(
        &pool,
        montage::migrations_before(20261027000001, "mcp_prompts"),
    )
    .await
    .expect("base partielle");
    sqlx::query("UPDATE _kesh_version SET kesh_version_last_applied = '0.17.3' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("version de départ");
    let avant: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("compte avant");
    assert!(
        (avant as usize) < kesh_db::MIGRATOR.migrations.len(),
        "assertion de montage : des migrations restent à appliquer"
    );

    // L'URL de la base éphémère : celle des tests, sur la base du pool.
    let base = pool
        .connect_options()
        .get_database()
        .expect("nom de la base éphémère")
        .to_string();
    let mut url = url::Url::parse(
        &std::env::var("DATABASE_URL").expect("DATABASE_URL obligatoire (MariaDB de test)"),
    )
    .expect("DATABASE_URL lisible");
    url.set_path(&base);

    let dossier = tempfile::tempdir().expect("dossier temporaire");
    let fichier = dossier.path().join("pas-un-dossier");
    std::fs::write(&fichier, b"fichier").expect("fichier");
    let occupe = std::net::TcpListener::bind("127.0.0.1:0").expect("port occupé");
    let port = occupe.local_addr().unwrap().port().to_string();
    let locales = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kesh-i18n/locales")
        .canonicalize()
        .expect("locales");

    let sortie = binaire::lancer_binaire(&[
        ("DATABASE_URL", url.as_str()),
        ("KESH_JWT_SECRET", SECRET),
        ("KESH_HOST", "127.0.0.1"),
        ("KESH_PORT", &port),
        ("KESH_ADMIN_BACKUP_DIR", &fichier.to_string_lossy()),
        ("KESH_LOCALES_DIR", &locales.to_string_lossy()),
    ]);
    drop(occupe);
    let texte = binaire::texte(&sortie);
    assert!(!sortie.status.success(), "refus attendu : {texte}");
    assert!(texte.contains("mise à jour refusée"), "{texte}");
    assert!(texte.contains("dossier"), "{texte}");
    let apres: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("compte après");
    assert_eq!(avant, apres, "aucune migration appliquée : {texte}");
}
