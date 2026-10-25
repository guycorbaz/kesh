//! Story 32-6a (refs #723) — `kesh_db::pool::pool_lecture_seule`, le pool des
//! outils du serveur MCP (AC-a25).
//!
//! Utilisateur `kesh`, base éphémère `#[sqlx::test]`, aucun privilège de plus.
//!
//! - (a) à (c) : écritures et verrous refusés en 1792, en autocommit comme dans
//!   une transaction ; (d) une lecture passe ;
//! - (e) et (e bis) : une forme qui défait le mode de session sur un emprunt ne
//!   survit pas à la restitution — la connexion est fermée, la suivante est
//!   neuve (`CONNECTION_ID()` change) ;
//! - (f) une connexion tuée pendant un emprunt n'empêche pas le suivant ;
//! - (g) une instruction plus longue que le délai d'instruction est
//!   **interrompue par MariaDB** (erreur 1969), même quand le futur qui
//!   l'attendait a été abandonné (revue P1 de la 32-6a, E1 = B2).
//!
//! ⚠️ (e) et (e bis) exécutent **volontairement** une forme interdite : ce sont
//! les deux exceptions écrites de la garde `lecture_seule_jamais_defaite`
//! (`kesh-api/tests/mcp_gardes_lexicales.rs`).

use std::time::{Duration, Instant};

use kesh_db::pool::pool_lecture_seule;
use sqlx::{Executor, MySqlPool};

/// Le délai d'instruction des tests (a) à (f) : large, sans effet sur eux.
const DELAI: Duration = Duration::from_secs(30);

async fn table(pool: &MySqlPool) {
    pool.execute("CREATE TABLE t326 (id INT PRIMARY KEY, v INT NOT NULL)")
        .await
        .unwrap();
    pool.execute("INSERT INTO t326 VALUES (1, 10)")
        .await
        .unwrap();
}

/// Le code MariaDB de l'erreur (1792, `ER_CANT_EXECUTE_IN_READ_ONLY_TRANSACTION`).
fn code_mariadb(e: &sqlx::Error) -> Option<u16> {
    match e {
        sqlx::Error::Database(d) => d
            .try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
            .map(|m| m.number()),
        _ => None,
    }
}

fn assert_refus(r: Result<impl std::fmt::Debug, sqlx::Error>, quoi: &str) {
    let e = r.expect_err(quoi);
    assert_eq!(code_mariadb(&e), Some(1792), "{quoi} : {e}");
}

async fn id_connexion(c: &mut sqlx::MySqlConnection) -> u64 {
    sqlx::query_scalar::<_, u64>("SELECT CONNECTION_ID()")
        .fetch_one(c)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "./test-schema")]
async fn a_b_c_d_ecritures_et_verrous_refuses_lecture_admise(pool: MySqlPool) {
    table(&pool).await;
    let ro = pool_lecture_seule(&pool, 2, DELAI);

    // (a) autocommit
    for sql in [
        "INSERT INTO t326 VALUES (2, 20)",
        "UPDATE t326 SET v = 11 WHERE id = 1",
        "DELETE FROM t326 WHERE id = 1",
    ] {
        assert_refus(sqlx::query(sql).execute(&ro).await, sql);
    }
    // (b) dans une transaction
    for sql in [
        "INSERT INTO t326 VALUES (2, 20)",
        "UPDATE t326 SET v = 11 WHERE id = 1",
        "DELETE FROM t326 WHERE id = 1",
    ] {
        let mut tx = ro.begin().await.unwrap();
        assert_refus(sqlx::query(sql).execute(&mut *tx).await, sql);
        tx.rollback().await.unwrap();
    }
    // (c) verrou
    assert_refus(
        sqlx::query("SELECT v FROM t326 WHERE id = 1 FOR UPDATE")
            .fetch_one(&ro)
            .await,
        "SELECT … FOR UPDATE",
    );
    // (d) lecture
    let v: i32 = sqlx::query_scalar("SELECT v FROM t326 WHERE id = 1")
        .fetch_one(&ro)
        .await
        .unwrap();
    assert_eq!(v, 10);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM t326")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1, "rien n'a été écrit");
}

/// (e) — séquence B de la spike (§ 4.2), forme 4 : `SET SESSION TRANSACTION
/// READ WRITE` sur un emprunt, rendu, puis réacquis → l'écriture reste
/// refusée, sur une autre connexion.
#[sqlx::test(migrations = "./test-schema")]
async fn e_la_forme_session_ne_survit_pas_a_la_restitution(pool: MySqlPool) {
    table(&pool).await;
    let ro = pool_lecture_seule(&pool, 1, DELAI);
    let premier = {
        let mut c = ro.acquire().await.unwrap();
        let id = id_connexion(&mut c).await;
        sqlx::query("SET SESSION TRANSACTION READ WRITE")
            .execute(&mut *c)
            .await
            .unwrap();
        id
    };
    let mut c = ro.acquire().await.unwrap();
    assert_ne!(id_connexion(&mut c).await, premier, "connexion neuve");
    assert_refus(
        sqlx::query("UPDATE t326 SET v = 12 WHERE id = 1")
            .execute(&mut *c)
            .await,
        "UPDATE après réacquisition",
    );
}

/// (e bis) — forme 2 : `SET TRANSACTION READ WRITE` (sans `SESSION`), qui vaut
/// pour la transaction suivante et que `@@session` ne montre pas.
#[sqlx::test(migrations = "./test-schema")]
async fn e_bis_la_forme_transaction_ne_survit_pas_a_la_restitution(pool: MySqlPool) {
    table(&pool).await;
    let ro = pool_lecture_seule(&pool, 1, DELAI);
    let premier = {
        let mut c = ro.acquire().await.unwrap();
        let id = id_connexion(&mut c).await;
        sqlx::query("SET TRANSACTION READ WRITE")
            .execute(&mut *c)
            .await
            .unwrap();
        id
    };
    let mut c = ro.acquire().await.unwrap();
    assert_ne!(id_connexion(&mut c).await, premier, "connexion neuve");
    assert_refus(
        sqlx::query("UPDATE t326 SET v = 13 WHERE id = 1")
            .execute(&mut *c)
            .await,
        "UPDATE après réacquisition",
    );
}

/// (f) — une connexion tuée pendant un emprunt (par le pool ordinaire)
/// n'empêche pas l'emprunt suivant, qui part d'une connexion neuve, refuse
/// l'écriture et porte un autre identifiant.
#[sqlx::test(migrations = "./test-schema")]
async fn f_une_connexion_tuee_n_empeche_pas_l_emprunt_suivant(pool: MySqlPool) {
    table(&pool).await;
    let ro = pool_lecture_seule(&pool, 1, DELAI);
    let tuee = {
        let mut c = ro.acquire().await.unwrap();
        let id = id_connexion(&mut c).await;
        sqlx::query(&format!("KILL {id}"))
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            sqlx::query("SELECT 1").execute(&mut *c).await.is_err(),
            "la connexion tuée échoue dans son emprunt"
        );
        id
    };
    let mut c = ro.acquire().await.unwrap();
    assert_ne!(id_connexion(&mut c).await, tuee);
    assert_refus(
        sqlx::query("UPDATE t326 SET v = 14 WHERE id = 1")
            .execute(&mut *c)
            .await,
        "UPDATE sur la connexion suivante",
    );
}

/// Les instructions `SLEEP` encore en cours côté serveur, toutes sessions de
/// l'utilisateur confondues (l'utilisateur voit ses propres sessions sans
/// privilège `PROCESS`).
async fn sommeils_en_cours(pool: &MySqlPool, marque: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.PROCESSLIST \
         WHERE INFO LIKE CONCAT('%', ?, '%') AND INFO NOT LIKE '%PROCESSLIST%'",
    )
    .bind(marque)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// (g) — `max_statement_time` : une instruction plus longue que le délai est
/// interrompue **par MariaDB**. (1) attendue jusqu'au bout, elle échoue en 1969
/// bien avant sa fin ; (2) abandonnée par son futur (comme au délai d'un outil
/// MCP), elle ne survit pas côté serveur : après le délai, plus aucune session
/// ne l'exécute — sans la borne, le serveur la poursuivrait jusqu'à son terme ;
/// (3) l'emprunt suivant part d'une connexion neuve.
#[sqlx::test(migrations = "./test-schema")]
async fn g_une_instruction_trop_longue_est_interrompue_par_le_serveur(pool: MySqlPool) {
    let ro = pool_lecture_seule(&pool, 1, Duration::from_millis(500));
    let borne: f64 = {
        let mut c = ro.acquire().await.unwrap();
        sqlx::query_scalar("SELECT @@session.max_statement_time")
            .fetch_one(&mut *c)
            .await
            .unwrap()
    };
    assert_eq!(borne, 0.5, "la borne est posée sur chaque connexion");

    // (1) attendue jusqu'au bout.
    let debut = Instant::now();
    let premier = {
        let mut c = ro.acquire().await.unwrap();
        let id = id_connexion(&mut c).await;
        let e = sqlx::query("SELECT SLEEP(5) AS sommeil_g1_326")
            .fetch_one(&mut *c)
            .await
            .expect_err("interrompue par max_statement_time");
        assert_eq!(code_mariadb(&e), Some(1969), "{e}");
        id
    };
    assert!(debut.elapsed() < Duration::from_secs(3), "interrompue tôt");

    // (2) abandonnée par son futur.
    let abandon = tokio::time::timeout(Duration::from_millis(100), async {
        let mut c = ro.acquire().await.unwrap();
        sqlx::query("SELECT SLEEP(5) AS sommeil_g2_326")
            .fetch_one(&mut *c)
            .await
    })
    .await;
    assert!(abandon.is_err(), "le futur est abandonné en vol");
    assert_eq!(
        sommeils_en_cours(&pool, "sommeil_g2_326").await,
        1,
        "témoin : juste après l'abandon, le serveur exécute encore l'instruction"
    );
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert_eq!(
        sommeils_en_cours(&pool, "sommeil_g2_326").await,
        0,
        "après le délai, le serveur a interrompu l'instruction abandonnée"
    );

    // (3) l'emprunt suivant part d'une connexion neuve.
    let mut c = ro.acquire().await.unwrap();
    assert_ne!(id_connexion(&mut c).await, premier, "connexion neuve");
}
