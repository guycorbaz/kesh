//! Garde-fou du squash de schéma de test — Story 22-5 (#251).
//!
//! Le squash (`test-schema/0001_schema_squash.sql`) remplace, pour ~1100 tests
//! `#[sqlx::test]`, le rejeu des 61 migrations réelles par un batch DDL unique.
//! Un artefact dérivé sans rappel automatique **dérive en silence** : ce fichier
//! est ce rappel. Il monte les DEUX chemins sur deux bases et les compare.
//!
//! ⚠️ **Ce fichier monte le VRAI `MIGRATOR`** : c'est sa fonction. Il est donc
//! inscrit à la liste d'exclusions du test de complétude qu'il héberge — sans
//! quoi ce test rougirait sur son propre hôte.
//!
//! Trois propriétés, trois assertions, parce qu'elles échouent différemment :
//!
//! 1. **structure** — `information_schema` complet, collations et actions FK
//!    comprises. C'est la dérive attendue quand une migration est ajoutée.
//! 2. **données d'installation** — la ligne `_kesh_version`. `information_schema`
//!    ne décrit QUE la structure : sans cette assertion, un squash privé de sa
//!    ligne d'installation passerait, et `check_downgrade_protection` comme le
//!    verrou d'installation changeraient de comportement dans 1100 tests.
//! 3. **suivi** — exactement UNE ligne dans `_sqlx_migrations`. Ce mode d'échec
//!    est **muet** : si le dump réintégrait cette table, son `DROP TABLE IF
//!    EXISTS` détruirait la ligne de suivi que sqlx vient d'insérer, l'`UPDATE
//!    success = TRUE` final affecterait zéro ligne **sans erreur**, et la
//!    structure resterait identique des deux côtés.

use sqlx::migrate::Migrator;
use sqlx::{Connection, Executor, MySqlConnection, MySqlPool, Row};

/// Le squash, monté par le même chemin que les tests basculés.
static SQUASH: Migrator = sqlx::migrate!("./test-schema");

/// Les fichiers de test autorisés à rester sur le chemin RÉEL des migrations —
/// parce qu'ils testent ce chemin lui-même. Portée en dur ici, et non dans une
/// prose de spec : un fichier qui veut rejoindre cette liste doit modifier le
/// test qui la garde.
const ALLOWED_REAL_MIGRATOR_FILES: &[&str] = &[
    // L'installation fraîche EST le sujet.
    "crates/kesh-db/tests/migrations_fresh_install.rs",
    // Fenêtre d'upgrade partielle (sub-Migrator sur un préfixe réel).
    "crates/kesh-db/tests/migrations_upgrade_path.rs",
    // Backfills à fenêtre : ils appliquent N migrations puis la suivante.
    "crates/kesh-db/tests/accounts_role_backfill.rs",
    "crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs",
    // Triage P7 — rejeu des backfills après restauration.
    "crates/kesh-db/tests/post_restore_class_a.rs",
    "crates/kesh-db/tests/post_restore_transactionality.rs",
    // Backfill D6 de la Story 22-1, sur schéma réel.
    "crates/kesh-db/tests/client_number_canonical_backfill.rs",
    // Le garde-fou lui-même : il compare les deux chemins, il lui faut le vrai.
    "crates/kesh-db/tests/test_schema_guard.rs",
];

/// Les deux graphies licites du squash, selon le crate du test.
const SQUASH_SPELLINGS: &[&str] = &["\"./test-schema\"", "\"../kesh-db/test-schema\""];

// ============================================================================
// Montage de la base « squash »
// ============================================================================

/// Crée une base neuve et y monte le SQUASH, puis rend `(connexion, nom)`.
///
/// Nom **déterministe par test** : un orphelin laissé par un rouge est repris
/// au run suivant (le `DROP … IF EXISTS` d'ouverture), et le préfixe
/// `_sqlx_test_` est celui que le grant de l'utilisateur applicatif autorise —
/// le même qui fait vivre `#[sqlx::test]`.
async fn mount_squash(label: &str) -> (MySqlConnection, String) {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL requis");
    let db = format!("_sqlx_test_guard_{label}");

    let mut admin = MySqlConnection::connect(&url)
        .await
        .expect("connexion admin");
    admin
        .execute(format!("DROP DATABASE IF EXISTS `{db}`").as_str())
        .await
        .expect("drop préalable");
    admin
        .execute(format!("CREATE DATABASE `{db}`").as_str())
        .await
        .expect("create");
    admin.close().await.ok();

    let squash_url = swap_database(&url, &db);
    let mut conn = MySqlConnection::connect(&squash_url)
        .await
        .expect("connexion squash");
    SQUASH.run_direct(&mut conn).await.expect(
        "le squash doit s'appliquer — s'il échoue ici, régénérez : scripts/regen-test-schema.sh",
    );
    (conn, db)
}

/// Détruit la base de travail. Appelé en fin de test ; un panic antérieur la
/// laisse en place, et c'est le `DROP … IF EXISTS` du prochain run qui nettoie
/// (sqlx lui-même ne nettoie que les tests verts).
async fn drop_db(db: &str) {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL requis");
    if let Ok(mut admin) = MySqlConnection::connect(&url).await {
        let _ = admin
            .execute(format!("DROP DATABASE IF EXISTS `{db}`").as_str())
            .await;
        let _ = admin.close().await;
    }
}

fn swap_database(url: &str, db: &str) -> String {
    match url.rfind('/') {
        Some(i) => format!("{}/{}", &url[..i], db),
        None => format!("{url}/{db}"),
    }
}

// ============================================================================
// Relevé de schéma
// ============================================================================

/// Toutes les facettes du schéma, en lignes textuelles triées et comparables.
///
/// ⚠️ `TABLES.AUTO_INCREMENT` est **volontairement absent** : il varie avec les
/// insertions, et c'est exactement ce que le script de régénération normalise.
/// La normalisation et ce relevé s'excluent la même chose — ni plus (aveuglement),
/// ni moins (flake).
async fn schema_facts(conn: &mut MySqlConnection, db: &str) -> Vec<String> {
    let mut facts = Vec::new();

    let rows = sqlx::query(
        "SELECT TABLE_NAME, ENGINE, TABLE_COLLATION FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = ? AND TABLE_TYPE = 'BASE TABLE' ORDER BY TABLE_NAME",
    )
    .bind(db)
    .fetch_all(&mut *conn)
    .await
    .expect("relevé des tables");
    for r in rows {
        facts.push(format!(
            "table {} engine={} collation={}",
            r.get::<String, _>(0),
            r.get::<Option<String>, _>(1).unwrap_or_default(),
            r.get::<Option<String>, _>(2).unwrap_or_default()
        ));
    }

    let rows = sqlx::query(
        "SELECT TABLE_NAME, COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, EXTRA, \
                COLLATION_NAME, GENERATION_EXPRESSION, CAST(ORDINAL_POSITION AS CHAR) \
         FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = ? \
         ORDER BY TABLE_NAME, ORDINAL_POSITION",
    )
    .bind(db)
    .fetch_all(&mut *conn)
    .await
    .expect("relevé des colonnes");
    for r in rows {
        facts.push(format!(
            "column {}.{} pos={} type={} null={} default={:?} extra={} collation={:?} generated={:?}",
            r.get::<String, _>(0),
            r.get::<String, _>(1),
            r.get::<String, _>(8),
            r.get::<String, _>(2),
            r.get::<String, _>(3),
            r.get::<Option<String>, _>(4),
            r.get::<String, _>(5),
            r.get::<Option<String>, _>(6),
            r.get::<Option<String>, _>(7),
        ));
    }

    let rows = sqlx::query(
        "SELECT TABLE_NAME, INDEX_NAME, CAST(SEQ_IN_INDEX AS CHAR), COLUMN_NAME, \
                CAST(NON_UNIQUE AS CHAR), INDEX_TYPE \
         FROM information_schema.STATISTICS WHERE TABLE_SCHEMA = ? \
         ORDER BY TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX",
    )
    .bind(db)
    .fetch_all(&mut *conn)
    .await
    .expect("relevé des index");
    for r in rows {
        facts.push(format!(
            "index {}.{} seq={} col={:?} non_unique={} type={}",
            r.get::<String, _>(0),
            r.get::<String, _>(1),
            r.get::<String, _>(2),
            r.get::<Option<String>, _>(3),
            r.get::<String, _>(4),
            r.get::<String, _>(5),
        ));
    }

    // Clés étrangères AVEC leurs actions référentielles : un `ON DELETE CASCADE`
    // perdu ne se voit dans aucune autre facette.
    let rows = sqlx::query(
        "SELECT rc.CONSTRAINT_NAME, rc.TABLE_NAME, rc.REFERENCED_TABLE_NAME, \
                rc.UPDATE_RULE, rc.DELETE_RULE, kcu.COLUMN_NAME, kcu.REFERENCED_COLUMN_NAME, \
                CAST(kcu.ORDINAL_POSITION AS CHAR) \
         FROM information_schema.REFERENTIAL_CONSTRAINTS rc \
         JOIN information_schema.KEY_COLUMN_USAGE kcu \
           ON kcu.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA \
          AND kcu.CONSTRAINT_NAME = rc.CONSTRAINT_NAME \
         WHERE rc.CONSTRAINT_SCHEMA = ? \
         ORDER BY rc.TABLE_NAME, rc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION",
    )
    .bind(db)
    .fetch_all(&mut *conn)
    .await
    .expect("relevé des clés étrangères");
    for r in rows {
        facts.push(format!(
            "fk {}.{} -> {} on_update={} on_delete={} col={:?}->{:?} pos={}",
            r.get::<String, _>(1),
            r.get::<String, _>(0),
            r.get::<Option<String>, _>(2).unwrap_or_default(),
            r.get::<String, _>(3),
            r.get::<String, _>(4),
            r.get::<Option<String>, _>(5),
            r.get::<Option<String>, _>(6),
            r.get::<String, _>(7),
        ));
    }

    let rows = sqlx::query(
        "SELECT CONSTRAINT_NAME, TABLE_NAME, CHECK_CLAUSE FROM information_schema.CHECK_CONSTRAINTS \
         WHERE CONSTRAINT_SCHEMA = ? ORDER BY TABLE_NAME, CONSTRAINT_NAME",
    )
    .bind(db)
    .fetch_all(&mut *conn)
    .await
    .expect("relevé des contraintes CHECK");
    for r in rows {
        facts.push(format!(
            "check {}.{} = {}",
            r.get::<String, _>(1),
            r.get::<String, _>(0),
            r.get::<String, _>(2)
        ));
    }

    // Vues, triggers, routines : le schéma n'en porte aucun aujourd'hui — le
    // relevé les couvre pour que leur PREMIER ajout ne passe pas sous le radar
    // (le script de régénération refuse d'ailleurs de les dumper, cf. son § 4).
    for (kind, sql) in [
        (
            "view",
            "SELECT TABLE_NAME FROM information_schema.VIEWS WHERE TABLE_SCHEMA = ? ORDER BY TABLE_NAME",
        ),
        (
            "trigger",
            "SELECT TRIGGER_NAME FROM information_schema.TRIGGERS WHERE TRIGGER_SCHEMA = ? ORDER BY TRIGGER_NAME",
        ),
        (
            "routine",
            "SELECT ROUTINE_NAME FROM information_schema.ROUTINES WHERE ROUTINE_SCHEMA = ? ORDER BY ROUTINE_NAME",
        ),
    ] {
        let rows = sqlx::query(sql)
            .bind(db)
            .fetch_all(&mut *conn)
            .await
            .unwrap_or_default();
        for r in rows {
            facts.push(format!("{kind} {}", r.get::<String, _>(0)));
        }
    }

    facts
}

fn diff_report(real: &[String], squash: &[String]) -> String {
    let mut out = String::new();
    for f in real {
        if !squash.contains(f) {
            out.push_str(&format!("  MANQUE au squash : {f}\n"));
        }
    }
    for f in squash {
        if !real.contains(f) {
            out.push_str(&format!("  EN TROP au squash : {f}\n"));
        }
    }
    out
}

async fn current_db(conn: &mut MySqlConnection) -> String {
    sqlx::query_scalar::<_, String>("SELECT DATABASE()")
        .fetch_one(conn)
        .await
        .expect("nom de la base courante")
}

// ============================================================================
// 1. Structure
// ============================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn squash_matches_real_schema_structure(pool: MySqlPool) {
    let mut real = pool.acquire().await.unwrap();
    let real_db = current_db(&mut real).await;
    let real_facts = schema_facts(&mut real, &real_db).await;

    let (mut squash, squash_db) = mount_squash("structure").await;
    let squash_facts = schema_facts(&mut squash, &squash_db).await;
    let report = diff_report(&real_facts, &squash_facts);
    squash.close().await.ok();
    drop_db(&squash_db).await;

    assert!(
        report.is_empty(),
        "le squash a DÉRIVÉ du schéma réel — régénérez : scripts/regen-test-schema.sh\n{report}"
    );
    assert!(
        real_facts.len() > 500,
        "relevé suspect : seulement {} facettes de schéma — le relevé ne mesure plus rien",
        real_facts.len()
    );
}

// ============================================================================
// 2. Données d'installation
// ============================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn squash_seeds_the_kesh_version_row(pool: MySqlPool) {
    let mut real = pool.acquire().await.unwrap();
    let expected: (String, String, String) = sqlx::query_as(
        "SELECT CAST(id AS CHAR), kesh_version_min_required, kesh_version_last_applied \
         FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(&mut *real)
    .await
    .expect("le chemin réel doit poser _kesh_version");

    let (mut squash, squash_db) = mount_squash("kesh_version").await;
    let got: Option<(String, String, String)> = sqlx::query_as(
        "SELECT CAST(id AS CHAR), kesh_version_min_required, kesh_version_last_applied \
         FROM _kesh_version WHERE id = 1",
    )
    .fetch_optional(&mut squash)
    .await
    .expect("lecture _kesh_version côté squash");
    squash.close().await.ok();
    drop_db(&squash_db).await;

    assert_eq!(
        got.as_ref(),
        Some(&expected),
        "la ligne d'installation _kesh_version diffère (ou manque) au squash — \
         `information_schema` ne la voit PAS, seule cette assertion la tient. \
         Régénérez : scripts/regen-test-schema.sh"
    );
}

// ============================================================================
// 3. Suivi — le mode d'échec MUET
// ============================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn squash_database_tracks_exactly_one_migration(_pool: MySqlPool) {
    let (mut squash, squash_db) = mount_squash("tracking").await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&mut squash)
        .await
        .expect("_sqlx_migrations doit exister côté squash");
    let succeeded: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_one(&mut squash)
            .await
            .expect("lecture du succès");
    squash.close().await.ok();
    drop_db(&squash_db).await;

    assert_eq!(
        (count, succeeded),
        (1, 1),
        "le squash doit laisser EXACTEMENT une migration suivie et réussie. \
         Zéro ligne signale que le dump réintègre `_sqlx_migrations` : son \
         `DROP TABLE IF EXISTS` détruit la ligne de suivi que sqlx vient \
         d'insérer, et l'`UPDATE success = TRUE` final n'affecte alors RIEN, \
         sans erreur — la structure, elle, reste identique. Excluez la table \
         du dump (scripts/regen-test-schema.sh, § 5)."
    );
}

// ============================================================================
// 4. Le MIGRATOR compilé vs le répertoire sur disque
// ============================================================================

/// ⚠️ **Ce test existe parce qu'une mutation l'a exigé, pas parce qu'on l'avait
/// prévu.** En jouant la mutation « une migration ajoutée sans régénération du
/// squash doit rougir » (AC2), elle est restée VERTE : `sqlx::migrate!` est une
/// macro **compile-time**, et un fichier `.sql` AJOUTÉ ne crée aucune
/// dépendance de compilation — le crate n'est pas recompilé, le `MIGRATOR`
/// compilé ignore la migration neuve, et les trois assertions de schéma
/// comparent alors deux mondes également périmés. Le garde-fou se serait tu :
/// exactement le mode d'échec du test muet que le dépôt paie depuis
/// `backfill_skips_archived_accounts`.
///
/// Cette assertion-ci lit le répertoire **au runtime** et le confronte au
/// `MIGRATOR` compilé. Elle rougit donc dans le cas resté silencieux, en
/// disant quoi faire.
#[test]
fn the_real_migrator_matches_the_migrations_directory() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let mut on_disk: Vec<i64> = std::fs::read_dir(&dir)
        .expect("lecture de crates/kesh-db/migrations/")
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.ends_with(".sql") {
                return None;
            }
            name.split('_').next()?.parse::<i64>().ok()
        })
        .collect();
    on_disk.sort_unstable();

    let compiled: Vec<i64> = kesh_db::MIGRATOR
        .migrations
        .iter()
        .map(|m| m.version)
        .collect();

    assert_eq!(
        on_disk,
        compiled,
        "le `MIGRATOR` compilé ne correspond plus au répertoire de migrations \
         ({} fichiers sur disque, {} migrations compilées).\n\
         `sqlx::migrate!` est une macro COMPILE-TIME : un fichier AJOUTÉ ne \
         déclenche aucune recompilation, donc tout le garde-fou de schéma \
         compare deux mondes périmés SANS RIEN DIRE.\n\
         Forcez la recompilation (`touch crates/kesh-db/src/lib.rs`), puis \
         régénérez le squash : scripts/regen-test-schema.sh",
        on_disk.len(),
        compiled.len()
    );
}

// ============================================================================
// 5. Complétude de la bascule (Story 22-5, AC3)
// ============================================================================

/// Un attribut `#[sqlx::test]` du workspace, avec sa provenance.
struct Attribute {
    file: String,
    line: usize,
    args: String,
}

/// Balaie **tout le workspace** — pas le seul crate de ce test : les attributs
/// vivent dans trois crates et dans `src/` comme dans `tests/`. Un balayage
/// mono-crate raterait 749 attributs **en silence**.
fn scan_attributes() -> (Vec<Attribute>, usize) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("racine du workspace")
        .to_path_buf();

    let mut files = Vec::new();
    collect_rs(&root.join("crates"), &mut files);

    let mut attrs = Vec::new();
    let mut raw_mentions = 0usize;
    for path in &files {
        let content = std::fs::read_to_string(path).expect("lecture d'un .rs");
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            // Les mentions en doc-comment ne sont pas des attributs.
            if trimmed.starts_with("//") {
                continue;
            }
            // `starts_with`, pas `contains` : une VRAIE mention d'attribut ouvre
            // la ligne (après indentation), qu'elle soit d'un seul tenant ou
            // repliée par rustfmt. Le `contains` comptait aussi les chaînes
            // littérales de CE fichier — le détecteur se détectait lui-même, et
            // l'invariant sommant rougissait sur six faux positifs.
            if !trimmed.starts_with("#[sqlx::test") {
                continue;
            }
            raw_mentions += 1;
            if let Some(a) = parse_attribute(trimmed) {
                attrs.push(Attribute {
                    file: rel.clone(),
                    line: i + 1,
                    args: a,
                });
            }
        }
    }
    (attrs, raw_mentions)
}

/// Parse un attribut **d'une seule ligne**. Rend `None` si la ligne mentionne
/// `#[sqlx::test` sans former un attribut complet — c'est le cas d'un attribut
/// REPLIÉ par rustfmt, que l'invariant sommant de l'appelant transforme en
/// échec bruyant plutôt qu'en angle mort.
fn parse_attribute(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("#[sqlx::test")?;
    if let Some(inner) = rest.strip_prefix('(') {
        let end = inner.rfind(")]")?;
        Some(inner[..end].trim().to_string())
    } else if rest.starts_with(']') {
        Some(String::new())
    } else {
        None
    }
}

fn collect_rs(dir: &std::path::Path, acc: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collect_rs(&path, acc);
        } else if path.extension().is_some_and(|e| e == "rs") {
            acc.push(path);
        }
    }
}

#[test]
fn every_sqlx_test_attribute_is_accounted_for() {
    let (attrs, raw_mentions) = scan_attributes();

    // Plancher fail-loud : un balayage qui rate sa cible se TAIT, il ne rougit
    // pas. Ces deux garde-fous transforment le silence en échec.
    assert!(
        attrs.len() > 1100,
        "balayage suspect : seulement {} attributs #[sqlx::test] vus dans le \
         workspace (attendu > 1100). Le balayage a-t-il perdu sa racine ?",
        attrs.len()
    );
    for crate_dir in ["crates/kesh-api/", "crates/kesh-db/", "crates/kesh-report/"] {
        assert!(
            attrs.iter().any(|a| a.file.starts_with(crate_dir)),
            "balayage suspect : aucun attribut vu dans {crate_dir} — la frontière \
             de crate n'est pas franchie"
        );
    }

    // Invariant SOMMANT : toute mention de `#[sqlx::test` doit s'être parsée en
    // attribut. Un attribut REPLIÉ sur plusieurs lignes (rustfmt le fait dès
    // qu'il s'allonge) casserait sinon le parse en silence — « détecter, c'est
    // chercher large ».
    assert_eq!(
        attrs.len(),
        raw_mentions,
        "{} mention(s) de `#[sqlx::test` n'ont pas pu être parsées comme \
         attribut d'une seule ligne — attribut replié sur plusieurs lignes ? \
         Le contrôle de complétude serait aveugle dessus : dépliez l'attribut, \
         ou étendez `parse_attribute`.",
        raw_mentions - attrs.len()
    );

    let mut offenders = Vec::new();
    for a in &attrs {
        if SQUASH_SPELLINGS.iter().any(|s| a.args.contains(s)) {
            continue;
        }
        if ALLOWED_REAL_MIGRATOR_FILES.contains(&a.file.as_str()) {
            continue;
        }
        offenders.push(format!(
            "  {}:{} → #[sqlx::test({})]",
            a.file, a.line, a.args
        ));
    }

    assert!(
        offenders.is_empty(),
        "ces attributs `#[sqlx::test]` n'utilisent PAS le squash et ne sont pas \
         dans la liste d'exclusions — ils paieraient les 61 migrations à chaque \
         test, sans que rien ne le signale (Story 22-5, #251).\n\
         Utilisez `migrations = \"./test-schema\"` (kesh-db) ou \
         `migrations = \"../kesh-db/test-schema\"` (kesh-api, kesh-report) ; si le \
         test exerce VRAIMENT le chemin des migrations, inscrivez son fichier à \
         `ALLOWED_REAL_MIGRATOR_FILES` de ce test, avec son motif.\n{}",
        offenders.join("\n")
    );
}
