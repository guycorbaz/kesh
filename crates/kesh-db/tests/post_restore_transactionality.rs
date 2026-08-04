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

//! # ⚠️ Ce que ces tests ont dû apprendre : où LIRE
//!
//! La rédaction d'origine lisait la destination **depuis le pool** après un
//! `drop(tx)`. Or une transaction jamais committée est invisible depuis une
//! autre connexion **par construction** : les assertions « la destination est
//! inchangée » et « l'entrée suivante n'a pas tourné » étaient vraies
//! identiquement, y compris si le rejeu n'avait jamais démarré ou s'il avait
//! continué après l'erreur. Elles ne pouvaient pas échouer — c'est le mode
//! d'échec « test muet » du garde-fou **P6** de `CLAUDE.md`, et il visait ici
//! précisément l'AC que la story existe pour prouver.
//!
//! Chaque test lit donc **dans la transaction** ce qui n'y est observable que
//! là, et ne lit depuis le pool que ce qui doit y être visible après rollback.

use kesh_db::post_restore::{
    BackfillTrigger, PostRestoreBackfill, ReplayOutcome, replay_with_registry,
};
use kesh_db::test_fixtures::seed_accounting_company;
use sqlx::{MySql, MySqlPool, Transaction};
use std::collections::BTreeMap;

/// Nom du compte, **vu de l'extérieur de la transaction de restore**.
const TOUCHED: &str = "TOUCHE PAR LE REJEU";

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

/// Nom du compte lu **hors** de la transaction de restore : ce que verrait un
/// autre client, donc ce qui subsiste après rollback.
///
/// Pas de `LIMIT 1` : il masquerait un doublon de `number` alors que le
/// statement de fixture, lui, n'est pas scopé par société et toucherait les deux
/// lignes. La borne est portée par [`assert_single_account`], qui l'énonce.
async fn account_name(pool: &MySqlPool, number: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT name FROM accounts WHERE number = ?")
        .bind(number)
        .fetch_one(pool)
        .await
        .expect("lecture du nom de compte")
}

/// Nom du compte lu **dans** la transaction de restore — le seul endroit où
/// l'effet d'un rejeu non committé est observable.
async fn account_name_in_tx(tx: &mut Transaction<'_, MySql>, number: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT name FROM accounts WHERE number = ?")
        .bind(number)
        .fetch_one(&mut **tx)
        .await
        .expect("lecture du nom de compte dans la transaction de restore")
}

/// Borne explicite du montage : le SQL des fixtures cible `number = '3000'` sans
/// scoping par société. Un second compte portant ce numéro rendrait
/// `rows_affected` supérieur à 1 et le nom lu arbitraire.
async fn assert_single_account(pool: &MySqlPool, number: &str) {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE number = ?")
        .bind(number)
        .fetch_one(pool)
        .await
        .expect("comptage des comptes de la fixture");
    assert_eq!(
        count, 1,
        "le montage suppose UN SEUL compte n° {number} ; le SQL des fixtures n'est pas scopé par \
         société et en toucherait {count}"
    );
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
    assert_single_account(&pool, "3000").await;
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
    assert_eq!(after, TOUCHED);
}

/// **C4 — l'échec d'un statement rejoué annule tout.**
///
/// Le montage est en **trois temps**, et l'ordre n'est pas cosmétique :
///
/// 1. le backfill **valide** tourne, et son effet est constaté **dans la
///    transaction** — c'est ce qui rend la suite non vacue, *sur cette
///    exécution-ci* et non par renvoi à un autre test tournant sur une autre
///    base éphémère ;
/// 2. le backfill **fautif** échoue ;
/// 3. après abandon de la transaction, la destination est constatée intacte
///    **depuis le pool**.
///
/// Écrit en un seul temps — un `replay_with_registry(&[VALID, FAUTIF])` suivi
/// d'une lecture depuis le pool — le test ne pouvait pas échouer : rien de ce
/// qui vit dans une transaction non committée n'est visible d'une autre
/// connexion. L'assertion finale reste la même, mais elle discrimine désormais :
/// elle tomberait si le rejeu committait, et l'étape 1 tomberait s'il
/// n'écrivait rien.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn a_failing_backfill_rolls_back_the_whole_restore(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    assert_single_account(&pool, "3000").await;
    let before = account_name(&pool, "3000").await;

    let mut tx = pool.begin().await.expect("begin");

    // (1) Le travail déjà fait dans la transaction, CONSTATÉ et non supposé.
    replay_with_registry(&mut tx, &BTreeMap::new(), &[VALID])
        .await
        .expect("le rejeu valide doit réussir");
    assert_eq!(
        account_name_in_tx(&mut tx, "3000").await,
        TOUCHED,
        "pré-condition INTERNE À CETTE EXÉCUTION : le backfill valide doit avoir écrit dans la \
         transaction. Sans cette lecture, l'assertion finale serait vraie à vide."
    );

    // (2) Puis l'échec.
    let err = replay_with_registry(&mut tx, &BTreeMap::new(), &[FAUTIF])
        .await
        .expect_err("le statement fautif doit faire échouer le rejeu");

    // (3) La transaction est abandonnée sans commit — c'est ce que fait le
    //     handler d'import en propageant l'erreur.
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
/// sont pas exécutées, et l'erreur nomme l'entrée et le statement.
///
/// Sans cette borne, une erreur pourrait être avalée par une entrée ultérieure
/// et le restore committer sur un état partiel.
///
/// ⚠️ **L'arrêt se constate DANS la transaction.** Lue depuis le pool, l'absence
/// d'effet de `VALID` est vraie qu'il ait tourné ou non — la transaction n'est
/// jamais committée. Le seul endroit où « `VALID` n'a pas tourné » se distingue
/// de « `VALID` a tourné puis a été annulé » est la transaction elle-même, avant
/// son abandon.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn replay_stops_at_the_first_failing_entry(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    assert_single_account(&pool, "3000").await;
    let before = account_name(&pool, "3000").await;

    let mut tx = pool.begin().await.expect("begin");
    let err = replay_with_registry(&mut tx, &BTreeMap::new(), &[FAUTIF, VALID])
        .await
        .expect_err("échec attendu dès la première entrée");

    // L'observation qui discrimine : dans la transaction, avant tout abandon.
    let in_tx = account_name_in_tx(&mut tx, "3000").await;
    drop(tx);
    assert_eq!(
        in_tx, before,
        "l'entrée VALID, placée APRÈS la fautive, ne doit pas avoir été exécutée — constaté DANS \
         la transaction, seul endroit où son effet serait visible"
    );

    let msg = err.to_string();
    // L'erreur doit NOMMER l'entrée fautive : un code MariaDB nu ne dit à
    // l'exploitant ni quelle reprise ni quel statement a échoué. C'est la
    // contrepartie du contexte ajouté dans `replay_with_registry`.
    assert!(
        msg.contains("fixture:fautif"),
        "l'erreur doit nommer l'ENTRÉE fautive par son label, obtenu : {msg}"
    );
    assert!(
        msg.contains("statement #1"),
        "l'erreur doit situer le statement fautif dans l'entrée, obtenu : {msg}"
    );
    assert!(
        msg.to_lowercase().contains("colonne_inexistante"),
        "l'erreur doit porter le statement fautif lui-même, obtenu : {msg}"
    );
}
