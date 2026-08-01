//! Story 16-1c (#281) — cas **C4** : l'échec d'un backfill rejoué annule le
//! restore **entier**.
//!
//! # Pourquoi ce test vit ici et pas en E2E HTTP
//!
//! C'est le **seul niveau où l'échec est observable**. Par le chemin
//! `POST /api/v1/admin/full-import`, une erreur de rejeu remonte en
//! `AppError::AdminFullImportFailed`, qui rend un `500` générique dont le détail
//! est **loggé et jamais exposé** — strictement indiscernable d'un échec
//! d'`INSERT` du restore, survenu bien plus tôt. Un test HTTP serait donc vert
//! pour la mauvaise raison, exactement le mode d'échec que cette story existe
//! pour empêcher.
//!
//! Et l'entrée fautive ne peut pas non plus être injectée par `#[cfg(test)]` :
//! ce marqueur **ne traverse pas la frontière de crate**. Depuis un test
//! d'intégration de `kesh-api`, `kesh-db` est une dépendance ordinaire et son
//! `cfg(test)` vaut faux — une entrée déclarée ainsi ne serait vue par *aucun*
//! test, et non par tous. La déclarer inconditionnellement livrerait en
//! production une entrée dont le seul rôle est de faire échouer tout restore.
//!
//! D'où [`replay_with_registry`], qui prend le registre en **paramètre**. C'est
//! la porte d'injection qui rend cet AC testable.

use kesh_db::post_restore::{
    BackfillTrigger, PostRestoreBackfill, ReplayOutcome, replay_with_registry,
};
use kesh_db::test_fixtures::seed_accounting_company;
use sqlx::MySqlPool;
use std::collections::BTreeMap;

/// Un backfill valide, qui touche une ligne observable.
const VALID: PostRestoreBackfill = PostRestoreBackfill {
    version: 20260722000001,
    label: "fixture:valide",
    trigger: BackfillTrigger::Unconditional,
    sql: "UPDATE accounts SET name = 'TOUCHE PAR LE REJEU' WHERE number = '3000';",
};

/// Un backfill fautif : la colonne n'existe pas, MariaDB rend l'erreur 1054.
const FAUTIF: PostRestoreBackfill = PostRestoreBackfill {
    version: 20260729000001,
    label: "fixture:fautif",
    trigger: BackfillTrigger::Unconditional,
    sql: "UPDATE accounts SET colonne_inexistante = 1 WHERE number = '3000';",
};

async fn account_name(pool: &MySqlPool, number: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT name FROM accounts WHERE number = ? LIMIT 1")
        .bind(number)
        .fetch_one(pool)
        .await
        .expect("lecture du nom de compte")
}

/// **Pré-condition du test d'échec** — sans elle, C4 serait muet.
///
/// Prouve que le statement `VALID` touche réellement la ligne observée. Si ce
/// test tombait, l'assertion « la destination est inchangée » du test suivant
/// serait vraie **à vide** : elle passerait aussi bien si le rejeu n'avait
/// jamais démarré.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn replay_touches_the_observed_row_when_it_succeeds(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let before = account_name(&pool, "3000").await;

    let mut tx = pool.begin().await.expect("begin");
    let report = replay_with_registry(&mut tx, &BTreeMap::new(), &[VALID])
        .await
        .expect("le rejeu valide doit réussir");
    tx.commit().await.expect("commit");

    assert_eq!(report.len(), 1);
    assert_eq!(report[0].outcome, ReplayOutcome::ReplayedUnconditional);
    assert_eq!(
        report[0].rows_affected, 1,
        "le statement de fixture doit toucher exactement la ligne observée"
    );

    let after = account_name(&pool, "3000").await;
    assert_ne!(before, after, "le rejeu doit avoir modifié la ligne");
    assert_eq!(after, "TOUCHE PAR LE REJEU");
}

/// **C4 — l'échec d'un statement rejoué annule tout.**
///
/// Le registre enchaîne un backfill **valide** puis un **fautif**. Le premier
/// modifie la ligne observée ; le second échoue. Après rollback, la ligne doit
/// être **intacte** — ce qui prouve deux choses à la fois : que le rejeu avait
/// bien démarré (sinon rien n'aurait été à annuler, cf. le test de
/// pré-condition ci-dessus), et que son échec a bien emporté le travail déjà
/// fait dans la transaction.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn a_failing_backfill_rolls_back_the_whole_restore(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let before = account_name(&pool, "3000").await;

    let mut tx = pool.begin().await.expect("begin");
    let err = replay_with_registry(&mut tx, &BTreeMap::new(), &[VALID, FAUTIF])
        .await
        .expect_err("le statement fautif doit faire échouer le rejeu");
    // La transaction est abandonnée sans commit — c'est ce que fait le handler
    // d'import en propageant l'erreur.
    drop(tx);

    let after = account_name(&pool, "3000").await;
    assert_eq!(
        after, before,
        "après l'échec du rejeu, la destination doit être INCHANGÉE — y compris \
         l'effet du backfill valide qui l'a précédé dans la même transaction. \
         L'erreur remontée était : {err}"
    );
}

/// Le rejeu s'arrête **au premier** statement fautif : les entrées suivantes ne
/// sont pas exécutées, et l'erreur remonte telle quelle.
///
/// Sans cette borne, une erreur pourrait être avalée par une entrée ultérieure
/// et le restore committer sur un état partiel.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn replay_stops_at_the_first_failing_entry(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");

    let mut tx = pool.begin().await.expect("begin");
    let err = replay_with_registry(&mut tx, &BTreeMap::new(), &[FAUTIF, VALID])
        .await
        .expect_err("échec attendu dès la première entrée");
    drop(tx);

    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("colonne_inexistante") || msg.contains("1054"),
        "l'erreur doit nommer le statement fautif, obtenu : {msg}"
    );

    // La transaction ayant été abandonnée, rien n'a pu être committé — et
    // l'entrée `VALID`, placée APRÈS la fautive, n'a de toute façon pas tourné.
    let after = account_name(&pool, "3000").await;
    assert_ne!(
        after, "TOUCHE PAR LE REJEU",
        "l'entrée suivant la fautive ne doit pas avoir été exécutée"
    );
}
