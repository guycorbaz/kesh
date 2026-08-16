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
use sqlx::mysql::MySqlConnectOptions;
use sqlx::{Connection, Executor, MySqlConnection, MySqlPool, Row};
use std::str::FromStr;

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
/// Le nom porte le **PID**, comme T3 l'exigeait. Un nom déterministe rendrait
/// la reprise des orphelines gratuite, mais au prix d'une collision bien pire :
/// deux gates concurrents sur le même serveur — cas NORMAL, la § « gate ciblé
/// entre les passes, gate complet au push » du CLAUDE.md l'organise — verraient
/// le `DROP … IF EXISTS` de l'un détruire la base que l'autre est en train de
/// lire, et le symptôme serait un faux « le squash a DÉRIVÉ ». *(Relevé par les
/// trois lentilles de la passe 1 de revue.)*
///
/// Le préfixe `_sqlx_test_` est celui que le grant de l'utilisateur applicatif
/// autorise — le même qui fait vivre `#[sqlx::test]`.
async fn mount_squash(label: &str) -> (MySqlConnection, String) {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL requis");
    let db = format!("_sqlx_test_guard_{label}_{}", std::process::id());

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

    let mut conn = MySqlConnection::connect_with(&swap_database(&url, &db))
        .await
        .expect("connexion squash");
    SQUASH.run_direct(&mut conn).await.expect(
        "le squash doit s'appliquer — s'il échoue ici, régénérez : scripts/regen-test-schema.sh",
    );
    (conn, db)
}

/// Détruit la base de travail.
///
/// ⚠️ **La destruction n'est PAS garantie en cas de panic** — T3 la demandait,
/// et l'écart se déclare ici plutôt que de se taire : l'obtenir exigerait de
/// dérouler le corps du test dans une tâche jointe pour rattraper le panic, ce
/// qui rendrait ces trois tests nettement moins lisibles. Le coût de l'écart
/// est borné depuis la Story 22-5 : le datadir de dev est en tmpfs, donc un
/// redémarrage du conteneur — geste déjà documenté comme le balayage des
/// orphelines — efface tout. Et sqlx ne nettoie de toute façon que les tests
/// verts.
async fn drop_db(db: &str) {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL requis");
    if let Ok(mut admin) = MySqlConnection::connect(&url).await {
        let _ = admin
            .execute(format!("DROP DATABASE IF EXISTS `{db}`").as_str())
            .await;
        let _ = admin.close().await;
    }
}

/// Rend les options de connexion de `url`, la base remplacée par `db`.
///
/// ⚠️ **Passe par `MySqlConnectOptions`, jamais par de la chirurgie de chaîne.**
/// Un `rfind('/')` perdrait la query-string — or la CI documente explicitement
/// `pool_max_conns=N` dessus, et un `?ssl-mode=…` est licite. Le garde-fou se
/// connecterait alors autrement que les 1102 tests qu'il valide, sans que rien
/// ne le dise. *(Relevé par deux lentilles de la passe 1 de revue.)*
fn swap_database(url: &str, db: &str) -> MySqlConnectOptions {
    MySqlConnectOptions::from_str(url)
        .expect("DATABASE_URL analysable comme URL MySQL")
        .database(db)
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

    // Vues, triggers, routines, events, séquences : le schéma n'en porte aucun
    // aujourd'hui — le relevé les couvre pour que leur PREMIER ajout ne passe
    // pas sous le radar (le script de régénération refuse d'ailleurs de les
    // dumper, cf. son § 4).
    //
    // ⚠️ `expect`, PAS `unwrap_or_default` : ce dernier transformait une erreur
    // SQL en « zéro objet », c'est-à-dire en « rien d'anormal », dans le SEUL
    // relevé dont la raison d'être est de voir apparaître un premier objet. Et
    // l'aveuglement était symétrique — la même fonction sert les deux côtés,
    // donc le diff serait resté vide. *(Relevé en passe 1 de revue.)*
    //
    // ⚠️ `EVENTS` et les `SEQUENCE` ne sont pas décoratifs non plus :
    // `mariadb-dump` omet les events faute de `--events`, et une séquence porte
    // `TABLE_TYPE = 'SEQUENCE'`, donc échappe au relevé des tables ci-dessus.
    // Sans ces deux lignes, leur ajout serait perdu par le squash ET invisible
    // au comparateur. *(Relevé par deux lentilles en passe 1.)*
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
        (
            "event",
            "SELECT EVENT_NAME FROM information_schema.EVENTS WHERE EVENT_SCHEMA = ? ORDER BY EVENT_NAME",
        ),
        (
            "sequence",
            "SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA = ? \
             AND TABLE_TYPE = 'SEQUENCE' ORDER BY TABLE_NAME",
        ),
    ] {
        let rows = sqlx::query(sql)
            .bind(db)
            .fetch_all(&mut *conn)
            .await
            .unwrap_or_else(|e| panic!("relevé des {kind}s : {e}"));
        for r in rows {
            facts.push(format!("{kind} {}", r.get::<String, _>(0)));
        }
    }

    facts
}

/// Compare deux relevés et rend le rapport des écarts.
///
/// ⚠️ La comparaison porte sur des **multi-ensembles**, pas des ensembles : elle
/// compte les occurrences de chaque facette au lieu de tester l'appartenance.
/// Une comparaison par appartenance laissait passer une facette présente deux
/// fois d'un côté et une fois de l'autre ; la rattraper par l'égalité des
/// longueurs totales fermait ce cas-là mais pas les décalages **compensés**
/// (une facette en trop de chaque côté). Compter est plus simple que de
/// rattraper. *(Cas relevé en passe 1, rattrapage jugé trop étroit en passe 2.)*
fn diff_report(real: &[String], squash: &[String]) -> String {
    fn compter(v: &[String]) -> std::collections::BTreeMap<&str, usize> {
        let mut m = std::collections::BTreeMap::new();
        for f in v {
            *m.entry(f.as_str()).or_insert(0) += 1;
        }
        m
    }
    let (a, b) = (compter(real), compter(squash));
    let mut out = String::new();
    for (f, n) in &a {
        match b.get(f) {
            None => out.push_str(&format!("  MANQUE au squash : {f}\n")),
            Some(m) if m != n => out.push_str(&format!(
                "  MULTIPLICITÉ : {f} — {n}× côté réel, {m}× côté squash\n"
            )),
            _ => {}
        }
    }
    for f in b.keys() {
        if !a.contains_key(f) {
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
    // Plancher GLOBAL **et** planchers par facette : les seconds ferment
    // l'angle mort du premier (une catégorie entière peut disparaître sans que
    // le total franchisse 500), mais leur somme vaut 401 — les remplacer aurait
    // donc ABAISSÉ la garde contre une érosion diffuse répartie sur plusieurs
    // facettes. Les deux se complètent, ils ne se substituent pas.
    // *(Planchers par facette : passe 1. Retour du plancher global : passe 2,
    // qui a relevé le troc.)*
    assert!(
        real_facts.len() > 500,
        "relevé suspect : seulement {} facettes de schéma au total — le relevé \
         ne mesure plus grand-chose",
        real_facts.len()
    );
    for (prefix, floor) in [
        ("table ", 30),
        ("column ", 300),
        ("index ", 50),
        ("fk ", 20),
        ("check ", 1),
    ] {
        let n = real_facts.iter().filter(|f| f.starts_with(prefix)).count();
        assert!(
            n >= floor,
            "relevé suspect : {n} facette(s) « {} » côté réel (plancher {floor}) — \
             cette catégorie a cessé d'être relevée, et un écart y serait désormais MUET",
            prefix.trim()
        );
    }
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

/// ⚠️ `#[tokio::test]`, PAS `#[sqlx::test]` : ce test ne lit jamais la base du
/// vrai `MIGRATOR`, il monte la sienne. L'attribut `sqlx::test` lui faisait
/// payer les 61 migrations pour un `_pool` inutilisé — dans la story qui existe
/// pour supprimer ce coût. *(Relevé en passe 1 de revue.)*
#[tokio::test]
async fn squash_database_tracks_exactly_one_migration() {
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
         régénérez le squash : scripts/regen-test-schema.sh\n\
         Et si vous venez d'AJOUTER une migration : le nombre de migrations est \
         écrit en toutes lettres dans `.config/nextest.toml`, `CLAUDE.md`, \
         `crates/kesh-db/README.md`, `crates/kesh-db/test-schema/README.md` et \
         l'en-tête de CE fichier — aucun test ne les contrôle, ce rappel-ci en \
         tient lieu.",
        on_disk.len(),
        compiled.len()
    );
}

// ============================================================================
// 4-bis. Les checksums des migrations publiées (garde-fou P8)
// ============================================================================

/// ⚠️ **Ce test remplace un détecteur que la Story 22-5 a elle-même supprimé.**
///
/// Le garde-fou **P8** du `CLAUDE.md` pose qu'une migration déjà appliquée ne se
/// modifie plus — pas même un commentaire : `sqlx` enregistre son checksum dans
/// `_sqlx_migrations` et refuse de démarrer si l'octet change. Mais ce contrôle
/// ne se déclenche que contre une base **persistante** ayant déjà appliqué la
/// migration, « c'est-à-dire, en pratique, la suite E2E ou un `cargo run` de
/// dev ». Or T6 met le datadir de dev en tmpfs : `kesh` et `kesh_e2e` sont
/// reconstruites à chaque démarrage, donc **plus rien en local ne rencontre le
/// checksum**, et le défaut se déplacerait en aval — chez qui met à jour une
/// installation réelle, où il n'y a aucun retour en arrière propre.
///
/// Ce test ancre les checksums dans le dépôt. Il est plus fort que le détecteur
/// perdu : il rougit à **chaque gate**, et non le jour où une base persistante
/// croise par hasard la migration modifiée.
///
/// *(Arbitrage de Guy, passe 1 de revue de code — finding du Blind Hunter.)*
#[test]
fn published_migrations_keep_their_checksums() {
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations.sha384");
    let manifest = std::fs::read_to_string(&manifest_path).expect(
        "crates/kesh-db/migrations.sha384 est le registre des checksums publiés — \
         il est versionné, il ne se supprime pas",
    );

    let mut expected = std::collections::BTreeMap::new();
    for (n, line) in manifest.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (version, hex) = line.split_once(char::is_whitespace).unwrap_or_else(|| {
            panic!(
                "migrations.sha384 ligne {} : attendu `<version> <sha384>`",
                n + 1
            )
        });
        let version = version
            .parse::<i64>()
            .unwrap_or_else(|_| panic!("migrations.sha384 ligne {} : version illisible", n + 1));
        // ⚠️ Un doublon ÉCRASE silencieusement sur une `BTreeMap`. Si la seconde
        // ligne portait le checksum du fichier MODIFIÉ (conflit de fusion mal
        // résolu, copier-coller), le registre validerait la modification qu'il
        // existe pour interdire. *(Relevé par deux lentilles en passe 2.)*
        if expected
            .insert(version, hex.trim().to_ascii_lowercase())
            .is_some()
        {
            panic!(
                "migrations.sha384 ligne {} : version {version} EN DOUBLE. Le \
                 registre est append-only ; une seconde ligne écraserait la \
                 première sans que rien ne le signale.",
                n + 1
            );
        }
    }

    // Une entrée du registre dont la migration a disparu n'est visitée par
    // aucune boucle de comparaison : le contrôle se ferait sur un ensemble plus
    // petit qu'annoncé, sans rien dire. *(Relevé en passe 2.)*
    let orphelines: Vec<i64> = expected
        .keys()
        .copied()
        .filter(|v| !kesh_db::MIGRATOR.migrations.iter().any(|m| m.version == *v))
        .collect();
    assert!(
        orphelines.is_empty(),
        "version(s) inscrite(s) au registre de checksums mais absente(s) du \
         `MIGRATOR` : {orphelines:?}. Une migration a-t-elle été supprimée ou \
         renumérotée ? Le registre décrirait alors un monde qui n'existe plus."
    );

    let mut modified = Vec::new();
    let mut absents = Vec::new();
    for m in kesh_db::MIGRATOR.migrations.iter() {
        let hex: String = m.checksum.iter().map(|b| format!("{b:02x}")).collect();
        match expected.get(&m.version) {
            // La comparaison est insensible à la casse (le registre est
            // normalisé en minuscules à la lecture) : un hexadécimal saisi en
            // majuscules crierait sinon « MIGRATION MODIFIÉE » sans que rien ne
            // l'ait été. *(Relevé en passe 2.)*
            Some(known) if *known == hex => {}
            Some(known) => modified.push(format!(
                "  {} « {} »\n    registre : {known}\n    fichier  : {hex}",
                m.version, m.description
            )),
            None => absents.push(format!("{} {hex}", m.version)),
        }
    }

    assert!(
        modified.is_empty(),
        "UNE MIGRATION PUBLIÉE A ÉTÉ MODIFIÉE — c'est le garde-fou P8 du CLAUDE.md.\n\
         {}\n\
         Une migration déjà appliquée quelque part ne se modifie plus, pas même \
         un commentaire : `sqlx` compare le checksum et REFUSE DE DÉMARRER \
         (« was previously applied but has been modified »). Si la migration a \
         été publiée dans une release, il n'existe aucun retour en arrière \
         propre.\n\
         Ce qu'on voulait ajouter va AILLEURS : la ligne de \
         docs/migrations-idempotence-audit.md, les Dev Notes de la story, ou une \
         migration suivante. Annulez la modification.",
        modified.join("\n")
    );
    assert!(
        absents.is_empty(),
        "migration(s) absente(s) du registre de checksums. Si vous venez d'en \
         ajouter une, inscrivez-la — c'est le geste qui la déclare PUBLIÉE :\n{}\n\
         (à coller dans crates/kesh-db/migrations.sha384)",
        absents.join("\n")
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

const TOKEN: &str = "#[sqlx::test";

/// Les positions de `TOKEN` sur cette ligne, **hors littéral de chaîne**.
///
/// La règle est le nombre de guillemets non échappés qui précèdent : impair =
/// dans une chaîne. Elle suffit ici parce que toutes les mentions littérales du
/// dépôt vivent sur la ligne qui ouvre leur chaîne.
///
/// **Trois limites connues, et le choix de ne pas les fermer** :
/// - chaîne brute `r#"…"#` et littéral de caractère `'"'` faussent la parité —
///   25 et 14 occurrences dans `crates/`, aucune ne portant le jeton ;
/// - une chaîne littérale s'étendant sur plusieurs lignes **sans** continuation
///   `\` n'est pas suivie : l'analyse est par ligne, la parité repart de zéro.
///   *(Cette troisième limite a été relevée en passe 2 de revue, où elle n'était
///   pas reconnue.)*
///
/// Porter la parité d'une ligne à l'autre fermerait la troisième — mais
/// **désynchroniserait sur les 39 chaînes brutes et littéraux de caractère du
/// workspace**, échangeant un angle mort théorique contre de faux rouges bien
/// réels. Le compromis est donc assumé, et son coût est borné : un jeton logé
/// dans une telle chaîne serait compté par les DEUX compteurs, donc soit ignoré
/// (s'il ressemble à une graphie de squash), soit signalé comme contrevenant
/// avec son fichier et sa ligne. Jamais un silence sur un vrai test.
fn mentions_outside_strings(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut quotes = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'"' => quotes += 1,
            b'#' if line[i..].starts_with(TOKEN) && quotes.is_multiple_of(2) => out.push(i),
            _ => {}
        }
        i += 1;
    }
    out
}

/// La position du `//` ouvrant un commentaire de fin de ligne, **hors chaîne**.
fn line_comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut quotes = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'"' => quotes += 1,
            b'/' if quotes.is_multiple_of(2) && bytes.get(i + 1) == Some(&b'/') => {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Balaie **tout le workspace** — pas le seul crate de ce test, et pas le seul
/// répertoire `crates/` : les attributs vivent dans trois crates, dans `src/`
/// comme dans `tests/`, et un futur membre du workspace posé ailleurs (`xtask/`,
/// `tools/`) doit être vu. Un balayage mono-crate raterait 749 attributs **en
/// silence** ; un balayage cloué à `crates/` raterait le membre suivant.
/// *(Cloué à `crates/` jusqu'à la passe 1 de revue, où deux lentilles l'ont
/// relevé contre le doc-comment qui promettait déjà « tout le workspace ».)*
fn scan_attributes() -> (Vec<Attribute>, usize) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("racine du workspace")
        .to_path_buf();

    let mut files = Vec::new();
    collect_rs(&root, &mut files);

    let mut attrs = Vec::new();
    let mut raw_mentions = 0usize;
    for path in &files {
        let content = std::fs::read_to_string(path).expect("lecture d'un .rs");
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        // Profondeur, et non booléen : Rust autorise les commentaires de bloc
        // IMBRIQUÉS. Un `/* … /* interne */ … */` refermait le suivi au premier
        // `*/`, si bien que les lignes suivantes du bloc externe étaient
        // scannées comme du code — un attribut fantôme y aurait été compté dans
        // `attrs` ET dans `raw_mentions`, donc SANS déclencher l'invariant
        // sommant. *(Relevé en passe 2 de revue.)*
        let mut block_depth = 0usize;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if block_depth > 0 {
                block_depth = (block_depth + line.matches("/*").count())
                    .saturating_sub(line.matches("*/").count());
                continue;
            }
            // Un bloc `/* … */` est sauté qu'il se referme sur sa ligne ou non :
            // ne traiter que le cas non refermé laissait `/* #[sqlx::test] */`
            // compter comme mention brute sans jamais parser en attribut, donc
            // un rouge à trier là où il n'y a rien. *(Relevé en passe 2.)*
            if trimmed.starts_with("/*") {
                block_depth = line
                    .matches("/*")
                    .count()
                    .saturating_sub(line.matches("*/").count());
                continue;
            }
            // Les mentions en commentaire de ligne ne sont pas des attributs.
            if trimmed.starts_with("//") {
                continue;
            }

            // ⚠️ Le compte BRUT ne doit PAS repasser par le filtre qui alimente
            // `attrs`, sans quoi l'invariant sommant devient auto-référentiel :
            // il ne verrait que ce qu'il a déjà reconnu, tout en promettant dans
            // son message de couvrir « toute mention ». Un attribut qui n'ouvre
            // pas sa ligne (`#[ignore] #[sqlx::test(…)]`) échappait alors À LA
            // FOIS au contrôle de complétude et au garde-fou censé signaler
            // l'angle mort. *(HIGH de la passe 1 de revue.)*
            raw_mentions += mentions_outside_strings(line).len();

            if !trimmed.starts_with(TOKEN) {
                continue;
            }
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
///
/// ⚠️ Le commentaire de fin de ligne est retiré AVANT l'analyse : sinon le
/// `rfind(")]")` porte sur la ligne entière, et
/// `#[sqlx::test(migrator = "kesh_db::MIGRATOR")] // cf. "./test-schema"`
/// produirait des `args` contenant une graphie de squash — donc une exemption
/// **muette** pour un test resté sur le vrai migrator. *(Relevé en passe 1.)*
///
/// ⚠️ Le `//` se cherche **hors chaîne**, via `line_comment_start`. La première
/// rédaction gardait le découpage derrière une condition qui, l'appelant
/// garantissant déjà que la ligne OUVRE par `TOKEN`, était toujours vraie : elle
/// coupait donc au premier `//` rencontré, fût-il à l'intérieur des arguments
/// (`migrations = "./a//b"`). Un garde tautologique ne garde rien.
/// *(Relevé en passe 2 de revue.)*
fn parse_attribute(trimmed: &str) -> Option<String> {
    let code = match line_comment_start(trimmed) {
        Some(i) => &trimmed[..i],
        None => trimmed,
    };
    let rest = code.trim_end().strip_prefix(TOKEN)?;
    if let Some(inner) = rest.strip_prefix('(') {
        let end = inner.rfind(")]")?;
        Some(inner[..end].trim().to_string())
    } else if rest.starts_with(']') {
        Some(String::new())
    } else {
        None
    }
}

/// Répertoires sans code Rust du workspace, et dont la traversée coûte cher.
const SKIPPED_DIRS: &[&str] = &["target", ".git", "node_modules", "build", ".svelte-kit"];

fn collect_rs(dir: &std::path::Path, acc: &mut Vec<std::path::PathBuf>) {
    // ⚠️ Une erreur de lecture ROUGIT, elle ne se saute pas : un sous-arbre
    // devenu illisible ferait sous-compter le balayage en silence, et le
    // plancher global pourrait rester franchi malgré le répertoire manqué.
    // C'est le mode d'échec que tout ce fichier existe pour interdire.
    // *(Second volet d'un finding de passe 1, resté non traité, relevé en
    // passe 2 par l'audit d'acceptation.)*
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("lecture du répertoire {} : {e}", dir.display()));
    for entry in entries.flatten() {
        // Les liens symboliques ne sont pas suivis : un lien vers un ancêtre
        // ferait récurser jusqu'au débordement de pile. *(Relevé en passe 1.)*
        if entry.file_type().is_ok_and(|t| t.is_symlink()) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| SKIPPED_DIRS.contains(&n))
            {
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

    // La liste d'exclusions doit désigner des fichiers qui EXISTENT. Une entrée
    // morte n'est pas du bois mort : elle ré-exempte tacitement un futur fichier
    // recréé à ce chemin, alors que le principe posé plus haut est qu'on ne
    // rejoint cette liste qu'en modifiant ce test. *(Relevé en passe 1.)*
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("racine du workspace");
    let morts: Vec<&str> = ALLOWED_REAL_MIGRATOR_FILES
        .iter()
        .filter(|f| !root.join(f).exists())
        .copied()
        .collect();
    assert!(
        morts.is_empty(),
        "entrée(s) morte(s) dans ALLOWED_REAL_MIGRATOR_FILES — supprimées ou \
         renommées. Retirez-les : telles quelles, elles ré-exempteraient en \
         silence un futur fichier recréé à ce chemin.\n  {}",
        morts.join("\n  ")
    );

    // Invariant SOMMANT : toute mention de `#[sqlx::test` hors commentaire et
    // hors littéral de chaîne doit s'être parsée en attribut. Il attrape deux
    // angles morts distincts — l'attribut REPLIÉ sur plusieurs lignes (rustfmt
    // le fait dès qu'il s'allonge) et l'attribut qui n'ouvre PAS sa ligne
    // (`#[ignore] #[sqlx::test(…)]`), lequel échappait auparavant au compte brut
    // lui-même. « Détecter, c'est chercher large » — la leçon 16-1c, appliquée
    // au détecteur.
    assert_eq!(
        attrs.len(),
        raw_mentions,
        "{} mention(s) de `#[sqlx::test` n'ont pas été parsées comme attribut \
         d'une seule ligne ouvrant sa ligne — attribut replié par rustfmt, ou \
         précédé d'un autre attribut sur la même ligne ? Le contrôle de \
         complétude serait AVEUGLE dessus : dépliez l'attribut sur sa propre \
         ligne, ou étendez `parse_attribute`.",
        raw_mentions.saturating_sub(attrs.len())
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
