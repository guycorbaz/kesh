//! Annuler un rapprochement bancaire — Story 25-3-b (#418), côté dépôt.
//!
//! Les cinq chemins réels (acceptation « facture », « éclatement », « règle »,
//! rapprochement et éclatement manuels), la précédence, l'exemption étroite,
//! les rôles et l'étanchéité sont exercés par la route, dans
//! `kesh-api/tests/reconciliation_e2e.rs` et `reconciliation_rules_e2e.rs`.
//! Ici : ce que seule une transaction tenue ouverte peut montrer — la
//! **concurrence avec une clôture d'exercice**.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::errors::{DbError, SettlementCancelBlocker};
use kesh_db::repositories::{journal_entries, reconciliation_cancel};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Une écriture **propre** à une transaction (le cas d'un rapprochement
/// manuel), dans un exercice **2019** qui ne couvre pas le jour, et la
/// transaction qui la pointe. Rend `(seeded, exercice 2019, écriture, transaction)`.
///
/// ⚠️ **Montage en partie forgé, et dit tel** : le rapprochement réel vit dans
/// `kesh-api` ; ici, on ne veut que l'ÉTAT qu'il produit. L'écriture est créée
/// par le vrai chemin dans l'exercice du montage, puis déplacée en 2019 — le
/// montage commun n'a qu'un exercice (2020-2030), qui couvre aussi le jour.
async fn monter(pool: &MySqlPool) -> (SeededCompany, i64, i64, i64) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    let entry = journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            journal: Journal::Banque,
            description: "Rapprochement manuel".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts["1000"],
                    debit: dec!(80.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["3000"],
                    debit: dec!(0),
                    credit: dec!(80.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture")
    .entry
    .id;
    let fy_2019 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, '2019', '2019-01-01', '2019-12-31', 'Open')",
    )
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "UPDATE journal_entries SET fiscal_year_id = ?, entry_date = '2019-06-03' WHERE id = ?",
    )
    .bind(fy_2019)
    .bind(entry)
    .execute(pool)
    .await
    .unwrap();

    let bank_account = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) \
         VALUES (?, 'Banque test', 'CH9300762011623852957')",
    )
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let import_id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'test.xml', REPEAT('b', 64), 'camt053', '2019-06-03', '2019-06-03', ?)",
    )
    .bind(seeded.company_id)
    .bind(bank_account)
    .bind(seeded.admin_user_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let tx_id = sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, \
          status, matched_entry_id) \
         VALUES (?, ?, ?, '2019-06-03', 80.00, 'CHF', 'test', 'reconciled', ?)",
    )
    .bind(seeded.company_id)
    .bind(import_id)
    .bind(bank_account)
    .bind(entry)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    (seeded, fy_2019, entry, tx_id)
}

/// ⛔ **Une clôture concurrente attend le dé-rapprochement** — la leçon de la
/// revue de la 25-3-a-1, appliquée ici. Sans le verrou de l'étape 3 (écriture
/// **et** exercice, avant de juger), le geste lirait l'exercice 2019 encore
/// ouvert, défairait le lien, et contre-passerait **avec succès** une écriture
/// d'exercice clos : la contre-passation est datée du jour, dans un autre
/// exercice, ouvert. Avec lui, il attend la clôture, relit, et refuse.
///
/// ⚠️ Montée sur une écriture PROPRE, non sur un règlement : pour un règlement,
/// le geste de la 25-3-a-1 verrouille lui aussi l'exercice, et masquerait la
/// mutation du verrou de ce geste-ci.
#[sqlx::test(migrations = "./test-schema")]
async fn a_concurrent_close_waits_for_the_unreconciliation(pool: MySqlPool) {
    let (seeded, fy_2019, entry, tx_id) = monter(&pool).await;

    // (1) La clôture en cours, non validée.
    let mut closing = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_2019)
        .execute(&mut *closing)
        .await
        .unwrap();

    // (2) Le dé-rapprochement démarre pendant ce temps.
    let p = pool.clone();
    let (company_id, user_id) = (seeded.company_id, seeded.admin_user_id);
    let annulation =
        tokio::spawn(
            async move { reconciliation_cancel::cancel(&p, company_id, tx_id, user_id).await },
        );
    // ⛔ Synchronisation déterministe : on ne valide la clôture qu'une fois le
    // geste VU en attente d'un `FOR UPDATE` sur `fiscal_years`.
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["fiscal_years", "FOR UPDATE"],
        || annulation.is_finished(),
    )
    .await;
    if !vue {
        panic!(
            "le dé-rapprochement a fini sans attendre de verrou : {:?}",
            annulation.await
        );
    }

    // (3) La clôture est validée.
    closing.commit().await.unwrap();

    let result = annulation.await.expect("tâche");
    assert!(
        matches!(
            result,
            Err(DbError::ReconciliationNotCancellable {
                blocker: SettlementCancelBlocker::FiscalYearClosed
            })
        ),
        "le geste devait attendre la clôture puis refuser l'exercice clos — reçu {result:?}"
    );
    let matched: Option<i64> =
        sqlx::query_scalar("SELECT matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(tx_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(matched, Some(entry), "le lien est intact");
    let reversals: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(entry)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reversals, 0, "rien n'a été contre-passé");
}
