//! Supprimer un import bancaire non rapproché — Story 27-1 (#632), côté dépôt.
//!
//! `bank_imports::delete_unreconciled_in_tx` : la définition de « rapprochée »
//! (D1-D6) et les trois courses que seule une transaction tenue ouverte peut
//! montrer (AC3 (a), (b), (c)). La route, les rôles, les clés d'API et la trace
//! d'audit sont exercés dans `kesh-api/tests/bank_imports_e2e.rs`.
//!
//! ⛔ Les courses se synchronisent par
//! `kesh_db::test_fixtures::attendre_une_requete_en_cours` — jamais par un
//! `sleep` : la sonde panique au bout de dix secondes si la requête attendue
//! n'est jamais vue, ce qui rend une mutation du verrou **déterministe**.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::bank_imports::{self, DeleteBankImportOutcome};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Le texte exact de l'`UPDATE` d'`accept_one_invoice`
/// (`kesh-api/src/routes/reconciliation.rs`), rejoué par les courses.
const ACCEPT_UPDATE: &str = "UPDATE bank_transactions \
     SET status = 'reconciled', matched_entry_id = ?, version = version + 1 \
     WHERE id = ? AND company_id = ? AND status = 'pending' AND version = ?";

/// Un compte bancaire pour la société `company_id`.
async fn bank_account(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) \
         VALUES (?, 'Banque test', 'CH9300762011623852957')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Un import de `n` transactions `pending` sans lien. Rend `(import, transactions)`.
async fn import_with(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    n: usize,
) -> (i64, Vec<i64>) {
    let import_id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, \
          period_to, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'releve.xml', REPEAT('d', 64), 'CAMT053_V04', '2026-01-01', \
                 '2026-01-31', ?, ?)",
    )
    .bind(company_id)
    .bind(bank_account_id)
    .bind(n as i32)
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut ids = Vec::new();
    for k in 0..n {
        let id = sqlx::query(
            "INSERT INTO bank_transactions \
             (company_id, import_id, bank_account_id, booking_date, amount, currency, details) \
             VALUES (?, ?, ?, '2026-01-15', ?, 'CHF', 'test')",
        )
        .bind(company_id)
        .bind(import_id)
        .bind(bank_account_id)
        .bind(rust_decimal::Decimal::from(10 + k as i64))
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        ids.push(id);
    }
    (import_id, ids)
}

/// Une écriture réelle (le vrai chemin `journal_entries::create`), pour la clé
/// `fk_bank_transactions_matched_entry` — montage de `reconciliation_cancel.rs`.
async fn entry(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            journal: Journal::Banque,
            description: "Rapprochement".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts["1000"],
                    debit: dec!(10.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["3000"],
                    debit: dec!(0),
                    credit: dec!(10.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture")
    .entry
    .id
}

/// Une seconde société, avec son utilisateur (pour l'étanchéité).
async fn other_company(pool: &MySqlPool) -> (i64, i64) {
    let company_id = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre société', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let user_id = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active, company_id) \
         VALUES ('autre', REPEAT('x', 32), 'Admin', TRUE, ?)",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    (company_id, user_id)
}

async fn count_import(pool: &MySqlPool, import_id: i64) -> (i64, i64) {
    let headers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_imports WHERE id = ?")
        .bind(import_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let lines: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?")
            .bind(import_id)
            .fetch_one(pool)
            .await
            .unwrap();
    (headers, lines)
}

async fn delete(pool: &MySqlPool, company_id: i64, import_id: i64) -> DeleteBankImportOutcome {
    let mut tx = pool.begin().await.unwrap();
    let out = bank_imports::delete_unreconciled_in_tx(&mut tx, company_id, import_id)
        .await
        .expect("suppression");
    if matches!(out, DeleteBankImportOutcome::Deleted(_)) {
        tx.commit().await.unwrap();
    } else {
        tx.rollback().await.unwrap();
    }
    out
}

/// D1 — trois transactions dont une rejetée : tout part, le reste est intact.
#[sqlx::test(migrations = "./test-schema")]
async fn d1_deletes_import_and_transactions_rejected_included(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, txs) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 3).await;
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(txs[1])
        .execute(&pool)
        .await
        .unwrap();
    let (kept_import, _) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 2).await;
    let (other_co, other_user) = other_company(&pool).await;
    let other_ba = bank_account(&pool, other_co).await;
    let (other_import, _) = import_with(&pool, other_co, other_ba, other_user, 2).await;

    match delete(&pool, seeded.company_id, import_id).await {
        DeleteBankImportOutcome::Deleted(d) => {
            assert_eq!(d.transaction_count, 3);
            assert_eq!(d.rejected_count, 1);
            assert_eq!(d.import.id, import_id);
            assert_eq!(d.import.filename, "releve.xml");
        }
        other => panic!("attendu Deleted, reçu {other:?}"),
    }
    assert_eq!(
        count_import(&pool, import_id).await,
        (0, 0),
        "import et lignes supprimés"
    );
    assert_eq!(
        count_import(&pool, kept_import).await,
        (1, 2),
        "autre import intact"
    );
    assert_eq!(
        count_import(&pool, other_import).await,
        (1, 2),
        "autre société intacte"
    );
}

/// D2 — une transaction `reconciled` avec lien : refus, tout intact.
#[sqlx::test(migrations = "./test-schema")]
async fn d2_refuses_when_one_transaction_is_reconciled(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, txs) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 3).await;
    let e = entry(&pool, &seeded).await;
    sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ? WHERE id = ?",
    )
    .bind(e)
    .bind(txs[0])
    .execute(&pool)
    .await
    .unwrap();

    match delete(&pool, seeded.company_id, import_id).await {
        DeleteBankImportOutcome::HasReconciled {
            reconciled_count,
            transaction_count,
        } => {
            assert_eq!(reconciled_count, 1);
            assert_eq!(transaction_count, 3);
        }
        other => panic!("attendu HasReconciled, reçu {other:?}"),
    }
    assert_eq!(
        count_import(&pool, import_id).await,
        (1, 3),
        "rien n'est supprimé"
    );
}

/// D3 — `pending` **avec** lien (état qu'aucun chemin ne produit) : seconde
/// branche de la disjonction seule.
#[sqlx::test(migrations = "./test-schema")]
async fn d3_pending_with_link_counts_as_reconciled(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, txs) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 2).await;
    let e = entry(&pool, &seeded).await;
    sqlx::query("UPDATE bank_transactions SET matched_entry_id = ? WHERE id = ?")
        .bind(e)
        .bind(txs[1])
        .execute(&pool)
        .await
        .unwrap();

    let out = delete(&pool, seeded.company_id, import_id).await;
    assert!(
        matches!(
            out,
            DeleteBankImportOutcome::HasReconciled {
                reconciled_count: 1,
                transaction_count: 2
            }
        ),
        "pending avec lien = rapprochée — reçu {out:?}"
    );
    assert_eq!(count_import(&pool, import_id).await, (1, 2));
}

/// D4 — `reconciled` **sans** lien (état laissé par un `ON DELETE SET NULL`) :
/// première branche de la disjonction seule.
#[sqlx::test(migrations = "./test-schema")]
async fn d4_reconciled_without_link_counts_as_reconciled(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, txs) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 2).await;
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE id = ?")
        .bind(txs[0])
        .execute(&pool)
        .await
        .unwrap();

    let out = delete(&pool, seeded.company_id, import_id).await;
    assert!(
        matches!(
            out,
            DeleteBankImportOutcome::HasReconciled {
                reconciled_count: 1,
                transaction_count: 2
            }
        ),
        "reconciled sans lien = rapprochée — reçu {out:?}"
    );
    assert_eq!(count_import(&pool, import_id).await, (1, 2));
}

/// D5 — l'import d'une autre société, et un identifiant inconnu : `NotFound`.
#[sqlx::test(migrations = "./test-schema")]
async fn d5_other_company_or_unknown_is_not_found(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let (other_co, other_user) = other_company(&pool).await;
    let other_ba = bank_account(&pool, other_co).await;
    let (other_import, _) = import_with(&pool, other_co, other_ba, other_user, 2).await;

    let out = delete(&pool, seeded.company_id, other_import).await;
    assert!(
        matches!(out, DeleteBankImportOutcome::NotFound),
        "reçu {out:?}"
    );
    assert_eq!(
        count_import(&pool, other_import).await,
        (1, 2),
        "autre société intacte"
    );

    let out = delete(&pool, seeded.company_id, other_import + 10_000).await;
    assert!(
        matches!(out, DeleteBankImportOutcome::NotFound),
        "reçu {out:?}"
    );
}

/// D6 — un import sans transaction est toujours supprimable.
#[sqlx::test(migrations = "./test-schema")]
async fn d6_empty_import_is_deleted(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, _) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 0).await;

    match delete(&pool, seeded.company_id, import_id).await {
        DeleteBankImportOutcome::Deleted(d) => {
            assert_eq!(d.transaction_count, 0);
            assert_eq!(d.rejected_count, 0);
        }
        other => panic!("attendu Deleted, reçu {other:?}"),
    }
    assert_eq!(count_import(&pool, import_id).await, (0, 0));
}

/// AC3 (a) — **l'acceptation d'abord** : la suppression attend son commit sur
/// la lecture verrouillante des transactions, puis la voit rapprochée et
/// refuse. Sous la mutation M3 (lecture ordinaire), c'est le `DELETE` de
/// l'entête qui attendrait, son texte ne porte pas `bank_transactions` : la
/// sonde panique à dix secondes — détection déterministe par absence du motif.
#[sqlx::test(migrations = "./test-schema")]
async fn ac3a_concurrent_accept_first_blocks_the_deletion(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, txs) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 2).await;
    let e = entry(&pool, &seeded).await;

    // (1) L'acceptation écrit, sans commit.
    let mut accept = pool.begin().await.unwrap();
    let r = sqlx::query(ACCEPT_UPDATE)
        .bind(e)
        .bind(txs[0])
        .bind(seeded.company_id)
        .bind(1_i32)
        .execute(&mut *accept)
        .await
        .unwrap();
    assert_eq!(r.rows_affected(), 1, "montage : l'acceptation écrit");

    // (2) La suppression démarre pendant ce temps.
    let p = pool.clone();
    let company_id = seeded.company_id;
    let suppression = tokio::spawn(async move { delete(&p, company_id, import_id).await });
    let vue = attendre_une_requete_en_cours(&pool, &["bank_transactions", "FOR UPDATE"], || {
        suppression.is_finished()
    })
    .await;
    if !vue {
        panic!(
            "la suppression a fini sans attendre la lecture verrouillante : {:?}",
            suppression.await
        );
    }

    // (3) L'acceptation est validée.
    accept.commit().await.unwrap();

    let out = suppression.await.expect("tâche");
    assert!(
        matches!(
            out,
            DeleteBankImportOutcome::HasReconciled {
                reconciled_count: 1,
                transaction_count: 2
            }
        ),
        "la suppression devait voir l'acceptation validée — reçu {out:?}"
    );
    assert_eq!(
        count_import(&pool, import_id).await,
        (1, 2),
        "import et lignes intacts"
    );
}

/// AC3 (b) — **la suppression d'abord** : l'`UPDATE` d'une acceptation attend,
/// puis ne trouve plus la ligne (`rows_affected() == 0`). Ce test n'établit
/// que le refus au niveau du dépôt ; ce que les flux en font est décrit à
/// l'AC3 (b) de la fiche (#634).
#[sqlx::test(migrations = "./test-schema")]
async fn ac3b_deletion_first_makes_accept_update_a_no_op(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, txs) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 2).await;
    let e = entry(&pool, &seeded).await;
    let version: i32 = sqlx::query_scalar("SELECT version FROM bank_transactions WHERE id = ?")
        .bind(txs[0])
        .fetch_one(&pool)
        .await
        .unwrap();

    // (1) La suppression, sans commit.
    let mut b = pool.begin().await.unwrap();
    let out = bank_imports::delete_unreconciled_in_tx(&mut b, seeded.company_id, import_id)
        .await
        .unwrap();
    assert!(
        matches!(out, DeleteBankImportOutcome::Deleted(_)),
        "reçu {out:?}"
    );

    // (2) L'acceptation démarre.
    let p = pool.clone();
    let (company_id, tx_id) = (seeded.company_id, txs[0]);
    let acceptation = tokio::spawn(async move {
        sqlx::query(ACCEPT_UPDATE)
            .bind(e)
            .bind(tx_id)
            .bind(company_id)
            .bind(version)
            .execute(&p)
            .await
            .map(|r| r.rows_affected())
    });
    let vue = attendre_une_requete_en_cours(&pool, &["UPDATE bank_transactions"], || {
        acceptation.is_finished()
    })
    .await;
    if !vue {
        panic!(
            "l'acceptation a fini sans attendre le verrou de la suppression : {:?}",
            acceptation.await
        );
    }

    // (3) La suppression est validée.
    b.commit().await.unwrap();

    let rows = acceptation.await.expect("tâche").expect("UPDATE");
    assert_eq!(rows, 0, "la ligne supprimée ne peut plus être rapprochée");
    assert_eq!(count_import(&pool, import_id).await, (0, 0));
}

/// AC3 (c) — **deux suppressions** : la seconde attend la première, puis rend
/// `NotFound`. Le résultat est tenu par l'un **ou** l'autre des deux contrôles
/// (le `FOR UPDATE` de l'entête, `rows_affected() != 1`) : chacun retiré seul
/// est une mutation équivalente (M14, M15) ; les deux ensemble rougissent
/// (`Deleted { transaction_count: 0 }`). L'import porte des transactions :
/// sous M14, la seconde attend sur la lecture verrouillante de ses lignes.
#[sqlx::test(migrations = "./test-schema")]
async fn ac3c_second_concurrent_deletion_is_not_found(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let ba = bank_account(&pool, seeded.company_id).await;
    let (import_id, _) = import_with(&pool, seeded.company_id, ba, seeded.admin_user_id, 2).await;

    // (1) La première suppression, sans commit.
    let mut b = pool.begin().await.unwrap();
    let out = bank_imports::delete_unreconciled_in_tx(&mut b, seeded.company_id, import_id)
        .await
        .unwrap();
    assert!(
        matches!(out, DeleteBankImportOutcome::Deleted(_)),
        "reçu {out:?}"
    );

    // (2) La seconde démarre.
    let p = pool.clone();
    let company_id = seeded.company_id;
    let seconde = tokio::spawn(async move { delete(&p, company_id, import_id).await });
    let vue = attendre_une_requete_en_cours(&pool, &["FOR UPDATE"], || seconde.is_finished()).await;
    if !vue {
        panic!(
            "la seconde suppression a fini sans attendre : {:?}",
            seconde.await
        );
    }

    // (3) La première est validée.
    b.commit().await.unwrap();

    let out = seconde.await.expect("tâche");
    assert!(
        matches!(out, DeleteBankImportOutcome::NotFound),
        "la seconde suppression devait rendre NotFound — reçu {out:?}"
    );
    assert_eq!(count_import(&pool, import_id).await, (0, 0));
}
