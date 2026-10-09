//! Story 15-8a (#532) — la modification d'une écriture, côté base ; Story
//! 15-8b — la suppression, pour la sérialisation (famille 2).
//!
//! Trois familles de tests, toutes sur le squash (`test-schema`), que
//! `test_schema_guard.rs` prouve égal au schéma de `MIGRATOR` :
//!
//! 1. **Le garde-fou d'inventaire (D3)** — un ensemble CLOS, pas une
//!    énumération : toute référence à `journal_entries` ou à
//!    `journal_entry_lines` doit être triée, faute de quoi la garde de
//!    modification laisserait réécrire une écriture qu'une pièce possède, ou
//!    perdrait en silence une colonne de ligne.
//! 2. **La sérialisation (D2, AC 8)** — le `FOR UPDATE` de l'écriture est le
//!    premier acte, et les exercices postérieurs se lisent verrouillés. Chaque
//!    entrelacement nomme la mutation qu'il tue.
//! 3. **Le cycle réel (AC 9)** — le `PUT` peut entrer en interblocage ; c'est
//!    pourquoi le handler le rejoue.
//!
//! ⚠️ Toutes les requêtes d'`information_schema` portent
//! `TABLE_SCHEMA = DATABASE()` : les bases éphémères voisines vivent sur le même
//! serveur.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{JournalEntryWithLines, NewJournalEntry, NewJournalEntryLine};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::journal_entries;
use kesh_db::retry::is_deadlock_error;
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// 1. Le garde-fou d'inventaire (D3, AC 13)
// ---------------------------------------------------------------------------

/// Les références vers `journal_entries`, triées : chacune est soit l'enfant
/// réécrit par la modification, soit un motif de `reversal_blockers` (gel).
const REFERENCES_TRIEES: &[(&str, &str, &str)] = &[
    (
        "journal_entry_lines",
        "entry_id",
        "enfant (CASCADE) — réécrit par la modification",
    ),
    (
        "journal_entries",
        "reverses_entry_id",
        "IsAReversal / AlreadyReversed",
    ),
    ("invoices", "journal_entry_id", "OwnedByInvoice"),
    ("credit_notes", "journal_entry_id", "OwnedByCreditNote"),
    (
        "supplier_invoices",
        "purchase_journal_entry_id",
        "OwnedBySupplierInvoice",
    ),
    (
        "supplier_invoices",
        "settlement_journal_entry_id",
        "OwnedBySupplierInvoice",
    ),
    (
        "invoice_settlements",
        "journal_entry_id",
        "OwnedBySettlement (règlements et soldes write_off)",
    ),
    (
        "bank_transactions",
        "matched_entry_id",
        "MatchedBankTransaction",
    ),
];

/// Les colonnes de `journal_entry_lines` que la modification réinsère.
const COLONNES_DES_LIGNES: &[&str] = &[
    "id",
    "entry_id",
    "account_id",
    "line_order",
    "debit",
    "credit",
    "project_id",
    // Story 15-1a-i (#518) — la marque du lettrage. Une ligne lettrée ne doit
    // pas être réécrite en silence : la modification et la suppression d'une
    // écriture qui porte une ligne lettrée sont gelées par
    // `journal_entries::lettering_guard` (motif `ModificationGuard::Lettered`,
    // `409 ENTRY_LETTERED` — AC8 de la Story 15-1a-ii), inconditionnel et
    // évalué après le verrou de période.
    "lettering_key",
    "lettering_origin",
];

/// Colonnes dont le nom évoque une écriture, sans clé étrangère, déclarées avec
/// leur justification.
const COLONNES_SANS_CLE_DECLAREES: &[(&str, &str, &str)] = &[(
    "company_invoice_settings",
    "journal_entry_description_template",
    "gabarit de libellé, pas une référence",
)];

const QUOI_FAIRE: &str = "trier la référence dans `reversal_blockers` (gel) ou la déclarer ici \
                          avec sa justification";

async fn cles_vers(pool: &MySqlPool, table: &str) -> BTreeSet<(String, String)> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT TABLE_NAME, COLUMN_NAME FROM information_schema.KEY_COLUMN_USAGE \
         WHERE TABLE_SCHEMA = DATABASE() AND REFERENCED_TABLE_SCHEMA = DATABASE() \
           AND REFERENCED_TABLE_NAME = ?",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .collect()
}

async fn assert_inventaire_clos(pool: &MySqlPool) {
    // (1) Les clés étrangères vers `journal_entries` : l'ensemble exact.
    let attendu: BTreeSet<(String, String)> = REFERENCES_TRIEES
        .iter()
        .map(|(t, c, _)| (t.to_string(), c.to_string()))
        .collect();
    let reel = cles_vers(pool, "journal_entries").await;
    assert_eq!(
        reel, attendu,
        "les clés étrangères vers journal_entries ont changé — {QUOI_FAIRE}"
    );

    // (1 bis) Aucune clé vers une LIGNE : la modification supprime et réinsère
    // les lignes, dont les `id` changent — un `RESTRICT` ferait échouer le `PUT`
    // (1451), un `CASCADE` effacerait en silence.
    let vers_lignes = cles_vers(pool, "journal_entry_lines").await;
    assert!(
        vers_lignes.is_empty(),
        "une table référence journal_entry_lines : {vers_lignes:?} — la modification réécrit les \
         lignes ; {QUOI_FAIRE}"
    );

    // (2) Les colonnes de `journal_entry_lines` : toute colonne neuve serait
    // perdue en silence par la réinsertion.
    let colonnes: BTreeSet<String> = sqlx::query_scalar::<_, String>(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'journal_entry_lines'",
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .collect();
    let attendues: BTreeSet<String> = COLONNES_DES_LIGNES.iter().map(|c| c.to_string()).collect();
    assert_eq!(
        colonnes, attendues,
        "les colonnes de journal_entry_lines ont changé — la modification les supprime et les \
         réinsère : une colonne neuve y serait perdue ; {QUOI_FAIRE}"
    );

    // (3) Une colonne SANS clé étrangère échappe au point (1) : on inventorie
    // celles dont le nom évoque une écriture.
    let mut nommees: BTreeSet<(String, String)> = sqlx::query_as::<_, (String, String)>(
        "SELECT TABLE_NAME, COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME <> 'journal_entries' \
           AND (COLUMN_NAME LIKE '%journal_entry%' OR COLUMN_NAME LIKE '%entry_id%')",
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .collect();
    for (t, c, _) in REFERENCES_TRIEES {
        nommees.remove(&(t.to_string(), c.to_string()));
    }
    let declarees: BTreeSet<(String, String)> = COLONNES_SANS_CLE_DECLAREES
        .iter()
        .map(|(t, c, _)| (t.to_string(), c.to_string()))
        .collect();
    assert_eq!(
        nommees, declarees,
        "une colonne au nom d'écriture, sans clé étrangère, n'est pas déclarée — {QUOI_FAIRE}"
    );
}

/// AC 13 · D3 — l'inventaire des références est CLOS.
///
/// ⚠️ **Il a rougi avec la 15-1a-i (lettrage)**, qui ajoute
/// `journal_entry_lines.lettering_key` et `lettering_origin` : c'est voulu — une
/// ligne lettrée ne doit pas être réécrite en silence. Les deux colonnes sont
/// déclarées à `COLONNES_DES_LIGNES`, avec le gel qui les protège.
///
/// ⚠️ **Angle mort assumé** : la trace du paiement détaché vit dans
/// `audit_log.details_json`, pas dans une colonne ; elle est tenue par son
/// propre test (`supplier_invoices_repository.rs`,
/// `cancel_paid_invoice_detaches_its_settlement`).
#[sqlx::test(migrations = "./test-schema")]
async fn every_reference_to_an_entry_is_sorted(pool: MySqlPool) {
    assert_inventaire_clos(&pool).await;
}

/// AC 13 — les mutations du garde-fou, TUÉES : une colonne ou une clé factice
/// le fait rougir chaque fois. Sans cela, un garde-fou qui ne rougit jamais
/// serait indiscernable d'un garde-fou juste.
#[sqlx::test(migrations = "./test-schema")]
async fn the_inventory_guard_turns_red_on_each_mutation(pool: MySqlPool) {
    async fn rougit(pool: &MySqlPool) -> bool {
        let pool = pool.clone();
        tokio::spawn(async move { assert_inventaire_clos(&pool).await })
            .await
            .is_err()
    }

    let mutations: [(&str, &str, &str); 4] = [
        (
            "clé étrangère factice vers journal_entries",
            "CREATE TABLE mutation_ref (id BIGINT PRIMARY KEY, je BIGINT, \
             CONSTRAINT fk_mutation_ref FOREIGN KEY (je) REFERENCES journal_entries(id))",
            "DROP TABLE mutation_ref",
        ),
        (
            "clé étrangère factice vers journal_entry_lines",
            "CREATE TABLE mutation_lettrage (id BIGINT PRIMARY KEY, l BIGINT, \
             CONSTRAINT fk_mutation_lettrage FOREIGN KEY (l) REFERENCES journal_entry_lines(id))",
            "DROP TABLE mutation_lettrage",
        ),
        (
            "colonne neuve sur journal_entry_lines",
            "ALTER TABLE journal_entry_lines ADD COLUMN lettering_id BIGINT NULL",
            "ALTER TABLE journal_entry_lines DROP COLUMN lettering_id",
        ),
        (
            "colonne au nom d'écriture sans clé",
            "ALTER TABLE invoices ADD COLUMN reminder_journal_entry_id BIGINT NULL",
            "ALTER TABLE invoices DROP COLUMN reminder_journal_entry_id",
        ),
    ];
    for (quoi, poser, retirer) in mutations {
        sqlx::query(poser).execute(&pool).await.unwrap();
        let rouge = rougit(&pool).await;
        sqlx::query(retirer).execute(&pool).await.unwrap();
        assert!(rouge, "le garde-fou d'inventaire n'a pas rougi : {quoi}");
    }
    // Et il redevient vert une fois les mutations retirées.
    assert_inventaire_clos(&pool).await;
}

// ---------------------------------------------------------------------------
// Montage commun
// ---------------------------------------------------------------------------

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Société seedée et ses comptes (1000, 1100, 2000, 3000, 4000) ; son exercice
/// (2020-2030 au seed) ramené à l'année 2026, pour qu'un exercice 2027 puisse
/// le suivre sans chevauchement.
async fn societe(pool: &MySqlPool) -> SeededCompany {
    let s = seed_accounting_company(pool).await.expect("seed");
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2026', start_date = '2026-01-01', \
         end_date = '2026-12-31' WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    s
}

fn corps(
    s: &SeededCompany,
    date: NaiveDate,
    description: &str,
    project: Option<i64>,
) -> NewJournalEntry {
    NewJournalEntry {
        company_id: s.company_id,
        entry_date: date,
        journal: Journal::OD,
        description: description.into(),
        project_id: None,
        lines: vec![
            NewJournalEntryLine {
                account_id: s.accounts["1000"],
                debit: dec!(100.00),
                credit: dec!(0),
                project_id: project,
            },
            NewJournalEntryLine {
                account_id: s.accounts["1100"],
                debit: dec!(0),
                credit: dec!(100.00),
                project_id: None,
            },
        ],
    }
}

async fn ecriture(
    pool: &MySqlPool,
    s: &SeededCompany,
    fy: i64,
    date: NaiveDate,
) -> JournalEntryWithLines {
    journal_entries::create(pool, fy, s.admin_user_id, corps(s, date, "origine", None))
        .await
        .expect("écriture")
}

async fn description_de(pool: &MySqlPool, id: i64) -> (String, i32) {
    sqlx::query_as("SELECT description, version FROM journal_entries WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn exercice(
    pool: &MySqlPool,
    s: &SeededCompany,
    name: &str,
    annee: i32,
    status: &str,
) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(s.company_id)
    .bind(name)
    .bind(d(annee, 1, 1))
    .bind(d(annee, 12, 31))
    .bind(status)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

// ---------------------------------------------------------------------------
// 2. La sérialisation (D2, AC 8)
// ---------------------------------------------------------------------------

/// AC 8 · D2 — une contre-passation **en cours** : le `PUT` l'attend sur le
/// verrou de l'écriture, puis la voit et refuse — `EntryIsReversed`, écriture
/// inchangée.
///
/// ⛔ **Tue** « une lecture ordinaire avant le `FOR UPDATE` de l'étape 1 » : la
/// vue `REPEATABLE READ` s'ouvrirait avant le commit de B, la garde ne verrait
/// pas la contre-passation, et le `PUT` réécrirait une écriture contre-passée —
/// aucune clé étrangère ne s'y oppose.
#[sqlx::test(migrations = "./test-schema")]
async fn update_waits_for_a_concurrent_reversal_then_refuses(pool: MySqlPool) {
    let s = societe(&pool).await;
    let e = ecriture(&pool, &s, s.fiscal_year_id, d(2026, 3, 1)).await;

    let mut b = pool.begin().await.unwrap();
    journal_entries::reverse_in_tx(&mut b, s.company_id, e.entry.id, s.admin_user_id)
        .await
        .expect("contre-passation de B");

    let a = {
        let pool = pool.clone();
        let body = corps(&s, d(2026, 3, 2), "réécrite", None);
        let (cid, uid, id, v) = (s.company_id, s.admin_user_id, e.entry.id, e.entry.version);
        tokio::spawn(
            async move { journal_entries::update(&pool, cid, id, v, uid, None, body).await },
        )
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["je.version", "FOR UPDATE"], || a.is_finished())
            .await,
        "le PUT doit attendre sur le verrou de l'écriture"
    );
    b.commit().await.unwrap();

    let result = a.await.unwrap();
    assert!(
        matches!(result, Err(DbError::EntryIsReversed)),
        "attendu EntryIsReversed, obtenu {result:?}"
    );
    assert_eq!(
        description_de(&pool, e.entry.id).await,
        ("origine".to_string(), e.entry.version)
    );
}

/// AC 8 · C-15-8-22 — la clôture **en cours** d'un exercice postérieur
/// (transition **simulée par SQL** : la clôture la refuse depuis la Story
/// 15-12a, N étant ouvert ; le test garde la propriété du verrou pour les
/// données héritées) : le
/// `PUT` l'attend sur le verrou d'intervalle des exercices postérieurs, puis la
/// voit et refuse — `LaterFiscalYearClosed` nommant N+1, écriture inchangée.
///
/// ⛔ **Tue** « lire les exercices postérieurs par une lecture ordinaire »
/// (`find_later_closed` au lieu de `find_later_closed_in_tx`) : le `PUT` ne
/// bloquerait plus, lirait la vue d'avant le commit de B, et passerait.
#[sqlx::test(migrations = "./test-schema")]
async fn update_waits_for_a_concurrent_close_of_a_later_year_then_refuses(pool: MySqlPool) {
    let s = societe(&pool).await;
    let suivant = exercice(&pool, &s, "Exercice 2027", 2027, "Open").await;
    let e = ecriture(&pool, &s, s.fiscal_year_id, d(2026, 3, 1)).await;

    let mut b = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(suivant)
        .execute(&mut *b)
        .await
        .unwrap();

    let a = {
        let pool = pool.clone();
        let body = corps(&s, d(2026, 3, 2), "réécrite", None);
        let (cid, uid, id, v) = (s.company_id, s.admin_user_id, e.entry.id, e.entry.version);
        tokio::spawn(
            async move { journal_entries::update(&pool, cid, id, v, uid, None, body).await },
        )
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["ORDER BY start_date ASC", "FOR UPDATE"], || a
            .is_finished())
        .await,
        "le PUT doit attendre sur le verrou des exercices postérieurs"
    );
    b.commit().await.unwrap();

    let result = a.await.unwrap();
    match result {
        Err(DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        }) => {
            assert_eq!(fiscal_year_id, suivant);
            assert_eq!(fiscal_year_name, "Exercice 2027");
        }
        other => panic!("attendu LaterFiscalYearClosed, obtenu {other:?}"),
    }
    assert_eq!(
        description_de(&pool, e.entry.id).await,
        ("origine".to_string(), e.entry.version)
    );
}

/// Story 15-8b (#532), AC 5 — le pendant `DELETE` : une contre-passation **en
/// cours** ; la suppression l'attend sur le verrou joint de `delete_in_tx`
/// (écriture **et** exercice), puis la voit et refuse — `EntryIsReversed`,
/// écriture intacte.
///
/// ⚠️ Les motifs sont ceux du `SELECT` de `delete_in_tx`
/// (`je.fiscal_year_id`, `FOR UPDATE`), pas ceux du `PUT` : ce `SELECT` ne
/// contient pas `je.version`, et `attendre_une_requete_en_cours` paniquerait.
///
/// ⛔ **Tue** « une lecture ordinaire (la borne) avant le `FOR UPDATE` » : la
/// vue se figerait avant le commit de B, `reversed_by` ne verrait pas la
/// contre-passation, la garde non plus, et A ne rendrait plus `EntryIsReversed`
/// — elle buterait sur la clé `RESTRICT` de `reverses_entry_id` (1451). Le test
/// exige le **code**. ⚠️ Limite nommée : pour les motifs sans clé `RESTRICT`
/// (paiement détaché, rapprochement en `SET NULL`), la propriété repose sur le
/// même verrou et la même règle, sans test de concurrence propre.
#[sqlx::test(migrations = "./test-schema")]
async fn delete_waits_for_a_concurrent_reversal_then_refuses(pool: MySqlPool) {
    let s = societe(&pool).await;
    let e = ecriture(&pool, &s, s.fiscal_year_id, d(2026, 3, 1)).await;

    let mut b = pool.begin().await.unwrap();
    journal_entries::reverse_in_tx(&mut b, s.company_id, e.entry.id, s.admin_user_id)
        .await
        .expect("contre-passation de B");

    let a = {
        let pool = pool.clone();
        let (cid, uid, id) = (s.company_id, s.admin_user_id, e.entry.id);
        tokio::spawn(async move { journal_entries::delete_by_id(&pool, cid, id, uid, None).await })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["je.fiscal_year_id", "FOR UPDATE"], || a
            .is_finished())
        .await,
        "le DELETE doit attendre sur le verrou de l'écriture"
    );
    b.commit().await.unwrap();

    let result = a.await.unwrap();
    assert!(
        matches!(result, Err(DbError::EntryIsReversed)),
        "attendu EntryIsReversed, obtenu {result:?}"
    );
    assert_eq!(
        description_de(&pool, e.entry.id).await,
        ("origine".to_string(), e.entry.version)
    );
}

/// Story 15-8b, AC 5 · C-15-8-22 — la clôture **en cours** d'un exercice
/// postérieur (transition **simulée par SQL** : la clôture la refuse depuis la
/// Story 15-12a ; le test garde la propriété du verrou pour les données
/// héritées) : la suppression l'attend sur le verrou d'intervalle des
/// exercices postérieurs (étape 2-bis), puis la voit et refuse —
/// `LaterFiscalYearClosed` nommant N+1, écriture intacte.
///
/// ⛔ **Tue** « lire les exercices postérieurs par une lecture ordinaire » : le
/// `DELETE` ne bloquerait plus, lirait la vue d'avant le commit de B, et
/// supprimerait une écriture que le bilan clos de N+1 reprend.
#[sqlx::test(migrations = "./test-schema")]
async fn delete_waits_for_a_concurrent_close_of_a_later_year_then_refuses(pool: MySqlPool) {
    let s = societe(&pool).await;
    let suivant = exercice(&pool, &s, "Exercice 2027", 2027, "Open").await;
    let e = ecriture(&pool, &s, s.fiscal_year_id, d(2026, 3, 1)).await;

    let mut b = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(suivant)
        .execute(&mut *b)
        .await
        .unwrap();

    let a = {
        let pool = pool.clone();
        let (cid, uid, id) = (s.company_id, s.admin_user_id, e.entry.id);
        tokio::spawn(async move { journal_entries::delete_by_id(&pool, cid, id, uid, None).await })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["ORDER BY start_date ASC", "FOR UPDATE"], || a
            .is_finished())
        .await,
        "le DELETE doit attendre sur le verrou des exercices postérieurs"
    );
    b.commit().await.unwrap();

    let result = a.await.unwrap();
    match result {
        Err(DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        }) => {
            assert_eq!(fiscal_year_id, suivant);
            assert_eq!(fiscal_year_name, "Exercice 2027");
        }
        other => panic!("attendu LaterFiscalYearClosed, obtenu {other:?}"),
    }
    assert_eq!(
        description_de(&pool, e.entry.id).await,
        ("origine".to_string(), e.entry.version)
    );
}

// ---------------------------------------------------------------------------
// 3. Le cycle réel (AC 9)
// ---------------------------------------------------------------------------

/// AC 9 · C-15-8-19 — le cycle **projet ↔ exercice** est réel, pas décoratif :
/// A modifie E1 en y posant un projet NOUVEAU P (verrou de P à l'étape 1-bis),
/// puis attend l'exercice que B tient ; B demande P en partagé. InnoDB casse le
/// cycle par une erreur 1213, que `is_deadlock_error` reconnaît — c'est ce que
/// le handler `PUT` rejoue (enveloppe `retry_on_deadlock`).
#[sqlx::test(migrations = "./test-schema")]
async fn update_and_a_reversal_of_the_same_year_can_deadlock(pool: MySqlPool) {
    let s = societe(&pool).await;
    let e1 = ecriture(&pool, &s, s.fiscal_year_id, d(2026, 3, 1)).await;
    let p = sqlx::query(
        "INSERT INTO projects (company_id, code, name, archived, version) VALUES (?, 'P-CYCLE', 'P', FALSE, 0)",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    let mut b = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(s.fiscal_year_id)
        .execute(&mut *b)
        .await
        .unwrap();

    let a = {
        let pool = pool.clone();
        let body = corps(&s, d(2026, 3, 1), "projet posé", Some(p));
        let (cid, uid, id, v) = (s.company_id, s.admin_user_id, e1.entry.id, e1.entry.version);
        tokio::spawn(
            async move { journal_entries::update(&pool, cid, id, v, uid, None, body).await },
        )
    };
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &[
                "SELECT status, start_date, end_date FROM fiscal_years",
                "FOR UPDATE"
            ],
            || a.is_finished()
        )
        .await,
        "le PUT doit attendre sur l'exercice, projet déjà tenu"
    );

    let b_result = sqlx::query("SELECT id FROM projects WHERE id = ? LOCK IN SHARE MODE")
        .bind(p)
        .execute(&mut *b)
        .await
        .map_err(map_db_error);
    let b_deadlock = matches!(&b_result, Err(e) if is_deadlock_error(e));
    if b_result.is_ok() {
        b.commit().await.unwrap();
    } else {
        let _ = b.rollback().await;
    }
    let a_result = a.await.unwrap();
    let a_deadlock = matches!(&a_result, Err(e) if is_deadlock_error(e));
    assert!(
        a_deadlock || b_deadlock,
        "un interblocage était attendu ; A = {a_result:?}, B = {b_result:?}"
    );
}
