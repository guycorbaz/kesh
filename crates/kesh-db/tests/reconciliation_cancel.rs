//! Annuler un rapprochement bancaire — Story 25-3-b (#418), côté dépôt.
//!
//! Les cinq chemins réels (acceptation « facture », « éclatement », « règle »,
//! rapprochement et éclatement manuels), la précédence, l'exemption étroite,
//! les rôles et l'étanchéité sont exercés par la route, dans
//! `kesh-api/tests/reconciliation_e2e.rs` et `reconciliation_rules_e2e.rs`. Le
//! sixième chemin, le rattachement à un règlement saisi depuis la fiche (Story
//! 29-8), produit l'état d'une acceptation « facture » : son annulation est
//! exercée par `kesh-api/tests/reconciliation_link_e2e.rs` (T15), et ce
//! fichier n'en a pas besoin — la concurrence avec une clôture d'exercice y est
//! celle du chemin « facture ».
//! Ici : ce que seule une transaction tenue ouverte peut montrer — la
//! **concurrence avec une clôture d'exercice**.
//!
//! **Story 30-1a (#686)** — le **règlement fournisseur** rapproché, que rien
//! ne produisait avant la 30-1b : son état est posé par
//! `support/fournisseur_rapproche.rs` (règlement par le vrai chemin, lien en
//! SQL), et le dé-rapprochement est exercé ici au dépôt — classement, état
//! final, grand livre, motifs, courses et plan de la lecture verrouillante. La
//! forme JSON et l'audit le sont par la route (`reconciliation_e2e.rs`). Le
//! septième chemin, l'acceptation d'une facture fournisseur (Story 30-1b),
//! produit cet état ; son aller et retour est exercé par la route
//! (`an_accepted_supplier_invoice_is_undone_by_cancelling_the_reconciliation`).
//!
//! Pré-requis : MariaDB démarré.

#[path = "support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

use chrono::NaiveDate;
use fournisseur_rapproche::{ReglementRapproche, preparer, reglement_rapproche, solde};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::errors::{DbError, SettlementCancelBlocker};
use kesh_db::repositories::{journal_entries, reconciliation_cancel, supplier_invoices};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_des_requetes_en_cours, seed_accounting_company,
};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Une écriture **propre** de 80.00 (`D 1000 / C 3000`), créée par le vrai
/// chemin dans l'exercice du montage — le cas d'un rapprochement manuel.
async fn ecriture_propre(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: jour(2026, 1, 15),
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
    .id
}

/// Un exercice **2019**, ouvert, qui ne couvre pas le jour : clore un exercice
/// passé ne touche pas celui où la contre-passation est datée.
async fn exercice_2019(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, '2019', '2019-01-01', '2019-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

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
    let entry = ecriture_propre(pool, &seeded).await;
    let fy_2019 = exercice_2019(pool, seeded.company_id).await;
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

/// Story 15-1a-ii (AC9, R6) — le dé-rapprochement d'un rapprochement **hors
/// facture** (`cancel_in_tx`, qui appelle `reverse_in_tx`) lettre lui aussi :
/// la ligne `1000` (actif, lettrable — le compte bancaire du montage ne la
/// désigne pas) et son miroir forment un groupe `reversal` ; la ligne `3000`
/// (produit) reste ouverte.
#[sqlx::test(migrations = "./test-schema")]
async fn reconciliation_cancel_letters_the_reversal_pair(pool: MySqlPool) {
    let (seeded, _, entry, tx_id) = monter(&pool).await;
    reconciliation_cancel::cancel(&pool, seeded.company_id, tx_id, seeded.admin_user_id)
        .await
        .expect("dé-rapprochement");
    let marques: Vec<(i64, i64, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT jel.entry_id, jel.account_id, jel.lettering_key, jel.lettering_origin \
         FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE je.id = ? OR je.reverses_entry_id = ? ORDER BY jel.id",
    )
    .bind(entry)
    .bind(entry)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(marques.len(), 4);
    let caisse = seeded.accounts["1000"];
    let cles: Vec<Option<i64>> = marques
        .iter()
        .filter(|(_, a, _, _)| *a == caisse)
        .map(|(_, _, k, _)| *k)
        .collect();
    assert_eq!(cles.len(), 2);
    assert!(cles[0].is_some() && cles[0] == cles[1], "{marques:?}");
    for (_, a, k, o) in &marques {
        if *a == caisse {
            assert_eq!(o.as_deref(), Some("reversal"));
        } else {
            assert_eq!((k, o), (&None, &None), "le produit reste ouvert");
        }
    }
}

// ===========================================================================
// Story 30-1a (#686) — dé-rapprocher un règlement fournisseur
// ===========================================================================

/// La société, le fournisseur, et un règlement de 240.00 du 15 juin 2026
/// rapproché d'un débit.
async fn monter_fournisseur(pool: &MySqlPool) -> (SeededCompany, ReglementRapproche) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    let f = preparer(pool, &seeded).await;
    let r = reglement_rapproche(
        pool,
        &seeded,
        f,
        "FF-30-1A",
        jour(2026, 6, 15),
        dec!(240.00),
    )
    .await;
    (seeded, r)
}

/// Le code du classement que rend la vue.
async fn kind_of(pool: &MySqlPool, company_id: i64, bt: i64) -> Option<&'static str> {
    reconciliation_cancel::get_view(pool, company_id, bt)
        .await
        .expect("vue")
        .expect("transaction")
        .kind
        .map(|k| k.code())
}

async fn compter(pool: &MySqlPool, sql: &str, id: i64) -> i64 {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(pool)
        .await
        .expect(sql)
}

/// T-a1 (AC-a1) — **le classement connaît le règlement fournisseur**, et lui
/// seul : un règlement fournisseur rapproché est `supplier_settlement` ; une
/// écriture propre reste `entry` ; un règlement **détaché** (facture payée puis
/// annulée, colonnes de règlement à `NULL`) reste `entry` — il n'a plus de
/// propriétaire (C-30-1-6). Tue M-a1.
#[sqlx::test(migrations = "./test-schema")]
async fn the_classification_knows_the_supplier_settlement(pool: MySqlPool) {
    let (seeded, r) = monter_fournisseur(&pool).await;
    let c = seeded.company_id;
    assert_eq!(
        kind_of(&pool, c, r.bank_transaction_id).await,
        Some("supplier_settlement")
    );

    // Une écriture propre, rapprochée d'une autre transaction.
    let propre = ecriture_propre(&pool, &seeded).await;
    let f = preparer_sans_reglages(&pool, &seeded, r).await;
    let bt_propre = fournisseur_rapproche::debit_rapproche(
        &pool,
        &seeded,
        f,
        propre,
        jour(2026, 1, 15),
        dec!(80.00),
    )
    .await;
    assert_eq!(kind_of(&pool, c, bt_propre).await, Some("entry"));

    // Le règlement détaché : la facture payée puis annulée.
    supplier_invoices::cancel(&pool, c, r.supplier_invoice_id, seeded.admin_user_id)
        .await
        .expect("annulation de la facture payée");
    assert_eq!(
        kind_of(&pool, c, r.bank_transaction_id).await,
        Some("entry"),
        "un règlement détaché n'est plus un règlement fournisseur"
    );
}

/// Le fournisseur et le compte de l'import du montage, relus (le montage n'en
/// crée qu'un de chaque).
async fn preparer_sans_reglages(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    r: ReglementRapproche,
) -> fournisseur_rapproche::Fournisseur {
    let (supplier_id, bank_account_id): (i64, i64) = sqlx::query_as(
        "SELECT si.contact_id, bt.bank_account_id FROM supplier_invoices si \
         JOIN bank_transactions bt ON bt.id = ? WHERE si.id = ? AND si.company_id = ?",
    )
    .bind(r.bank_transaction_id)
    .bind(r.supplier_invoice_id)
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    fournisseur_rapproche::Fournisseur {
        supplier_id,
        bank_account_id,
    }
}

/// T-a2 et T-a3 (AC-a2) — **le dé-rapprochement défait le règlement** : la
/// transaction revient « à rapprocher » (statut, lien, marqueur de rejet) ; la
/// facture revient `open`, colonnes de règlement et `paid_at` vidés ; la
/// contre-passation est datée du jour et liée ; le lettrage de pièce est
/// dissous et le paiement lettré avec son miroir ; la dette envers le
/// fournisseur renaît, et la banque est rendue. Tue M-a1 et M-a4 (une
/// contre-passation par `reverse_in_tx` est refusée : l'écriture appartient au
/// règlement).
#[sqlx::test(migrations = "./test-schema")]
async fn unreconciling_a_supplier_settlement_undoes_it(pool: MySqlPool) {
    let (seeded, r) = monter_fournisseur(&pool).await;
    let c = seeded.company_id;
    let (fournisseurs, banque) = (seeded.accounts["2000"], seeded.accounts["1100"]);
    assert_eq!(solde(&pool, c, fournisseurs).await, dec!(0.00), "montage");
    assert_eq!(solde(&pool, c, banque).await, dec!(-240.00), "montage");
    // Un marqueur de rejet posé : le dé-rapprochement doit le remettre à zéro.
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(r.bank_transaction_id)
        .execute(&pool)
        .await
        .unwrap();

    let done = reconciliation_cancel::cancel(&pool, c, r.bank_transaction_id, seeded.admin_user_id)
        .await
        .expect("⛔ le dé-rapprochement d'un règlement fournisseur doit aboutir");
    assert_eq!(done.invoice_id, None, "aucune facture CLIENT n'est en jeu");

    // La transaction.
    let (status, matched, rejected): (String, Option<i64>, Option<chrono::NaiveDateTime>) =
        sqlx::query_as(
            "SELECT status, matched_entry_id, auto_match_rejected_at FROM bank_transactions \
             WHERE id = ?",
        )
        .bind(r.bank_transaction_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        (status.as_str(), matched, rejected),
        ("pending", None, None)
    );

    // La facture.
    type Colonnes = (
        String,
        Option<String>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<chrono::NaiveDateTime>,
    );
    let facture: Colonnes = sqlx::query_as(
        "SELECT status, settlement_type, settlement_bank_account_id, settlement_account_id, \
         settlement_journal_entry_id, paid_at FROM supplier_invoices WHERE id = ?",
    )
    .bind(r.supplier_invoice_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(facture, ("open".to_string(), None, None, None, None, None));

    // La contre-passation, datée du jour et liée.
    let (reverses, date): (Option<i64>, NaiveDate) =
        sqlx::query_as("SELECT reverses_entry_id, entry_date FROM journal_entries WHERE id = ?")
            .bind(done.reversal_journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reverses, Some(r.settlement_entry_id));
    assert_eq!(date, chrono::Utc::now().date_naive());

    // Le lettrage : la pièce dissoute, le paiement lettré avec son miroir.
    let ligne = |entry: i64| {
        let pool = pool.clone();
        async move {
            sqlx::query_as::<_, (Option<i64>, Option<String>)>(
                "SELECT lettering_key, lettering_origin FROM journal_entry_lines \
                 WHERE entry_id = ? AND account_id = ?",
            )
            .bind(entry)
            .bind(fournisseurs)
            .fetch_one(&pool)
            .await
            .unwrap()
        }
    };
    assert_eq!(
        ligne(r.purchase_entry_id).await,
        (None, None),
        "la dette est de nouveau ouverte"
    );
    let paiement = ligne(r.settlement_entry_id).await;
    let miroir = ligne(done.reversal_journal_entry_id).await;
    assert!(paiement.0.is_some(), "{paiement:?}");
    assert_eq!(paiement.1.as_deref(), Some("reversal"));
    assert_eq!(miroir, paiement, "le paiement est lettré avec son miroir");

    // T-a3 — le grand livre revient à son état d'avant le règlement.
    assert_eq!(
        solde(&pool, c, fournisseurs).await,
        dec!(-240.00),
        "la dette envers le fournisseur renaît"
    );
    assert_eq!(
        solde(&pool, c, banque).await,
        dec!(0.00),
        "la banque est rendue"
    );
}

/// T-a4 (AC-a3) — **exercice du règlement clos** : refus
/// `ReconciliationNotCancellable` (exercice clos) par le geste lui-même,
/// **avant** de défaire le lien — rien d'écrit, le lien intact, la facture
/// toujours payée.
#[sqlx::test(migrations = "./test-schema")]
async fn a_closed_year_refuses_the_supplier_unreconciliation(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let fy_2019 = exercice_2019(&pool, seeded.company_id).await;
    let f = preparer(&pool, &seeded).await;
    let r = reglement_rapproche(&pool, &seeded, f, "FF-2019", jour(2019, 6, 3), dec!(75.50)).await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_2019)
        .execute(&pool)
        .await
        .unwrap();
    let ecritures = compter(
        &pool,
        "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
        seeded.company_id,
    )
    .await;

    let r2 = reconciliation_cancel::cancel(
        &pool,
        seeded.company_id,
        r.bank_transaction_id,
        seeded.admin_user_id,
    )
    .await;
    assert!(
        matches!(
            r2,
            Err(DbError::ReconciliationNotCancellable {
                blocker: SettlementCancelBlocker::FiscalYearClosed
            })
        ),
        "reçu {r2:?}"
    );
    assert_supplier_link_intact(&pool, &seeded, r, ecritures).await;
}

/// Rien n'a bougé : le lien, la facture payée, le nombre d'écritures.
async fn assert_supplier_link_intact(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    r: ReglementRapproche,
    ecritures: i64,
) {
    let (status, matched): (String, Option<i64>) =
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(r.bank_transaction_id)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        (status.as_str(), matched),
        ("reconciled", Some(r.settlement_entry_id)),
        "le lien est intact"
    );
    let (statut, reglement): (String, Option<i64>) = sqlx::query_as(
        "SELECT status, settlement_journal_entry_id FROM supplier_invoices WHERE id = ?",
    )
    .bind(r.supplier_invoice_id)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        (statut.as_str(), reglement),
        ("paid", Some(r.settlement_entry_id)),
        "la facture reste payée"
    );
    assert_eq!(
        compter(
            pool,
            "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
            seeded.company_id
        )
        .await,
        ecritures,
        "aucune écriture neuve"
    );
}

/// T-a5 (AC-a3) — **la vue annonce l'annulation possible** sur un règlement
/// fournisseur rapproché d'un exercice ouvert : la forme **exemptée** des
/// motifs pour cette transaction. La forme non exemptée rendrait
/// `MATCHED_BANK_TRANSACTION` sur son propre lien. Tue M-a3.
#[sqlx::test(migrations = "./test-schema")]
async fn the_view_says_a_reconciled_supplier_settlement_is_cancellable(pool: MySqlPool) {
    let (seeded, r) = monter_fournisseur(&pool).await;
    let view = reconciliation_cancel::get_view(&pool, seeded.company_id, r.bank_transaction_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.kind.map(|k| k.code()), Some("supplier_settlement"));
    assert_eq!(view.cancel_blocker, None, "{view:?}");
}

/// T-a6 (AC-a4) — **course : dé-rapprochement contre annulation de la même
/// facture**, qui détache le règlement. Une connexion tient la facture ; les
/// deux gestes démarrent et l'attendent tous deux (sonde qui **compte** : deux
/// requêtes en attente sur la facture) ; la connexion relâche. Deux issues
/// admises, selon l'ordre d'octroi : dé-rapprochement puis annulation d'une
/// facture `open`, ou annulation (règlement détaché, transaction toujours
/// rapprochée) puis dé-rapprochement classé `entry` qui contre-passe le
/// règlement détaché. Les deux aboutissent au **même** état final, qu'on
/// asserte : jamais d'état mixte.
///
/// ⚠️ **Un interblocage est une troisième issue, constatée au développement**
/// (C-30-1-42) : la lecture verrouillante du classement passe par l'index de
/// `settlement_journal_entry_id` et en tient l'enregistrement **avant** la
/// ligne ; l'annulation de la facture tient la ligne par sa clé primaire, puis
/// réécrit la colonne — elle demande l'enregistrement d'index. Cycle, `1213` ;
/// la victime n'a rien écrit (annulée). Les deux routes **rejouent**
/// (`retry_app_on_deadlock` pour le dé-rapprochement, `retry_on_deadlock` pour
/// `supplier_invoices::cancel`) : le test joue donc chaque geste dans
/// l'enveloppe `DbError` du dépôt, comme ses routes, et l'issue rejouée est
/// l'une des deux ci-dessus.
///
/// ⚠️ Test de non-régression sans mutation dédiée (fiche, § *Mutations*) : il
/// établit l'absence d'état mixte ; aucune ligne neuve ne le décide.
#[sqlx::test(migrations = "./test-schema")]
async fn unreconciliation_and_invoice_cancellation_race_without_a_mixed_state(pool: MySqlPool) {
    use kesh_db::retry::retry_on_deadlock;
    let (seeded, r) = monter_fournisseur(&pool).await;
    let (c, u) = (seeded.company_id, seeded.admin_user_id);

    let mut tenue = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM supplier_invoices WHERE id = ? FOR UPDATE")
        .bind(r.supplier_invoice_id)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();

    let p = pool.clone();
    let bt = r.bank_transaction_id;
    let derapprochement = tokio::spawn(async move {
        retry_on_deadlock("test::reconciliation_cancel", || {
            reconciliation_cancel::cancel(&p, c, bt, u)
        })
        .await
    });
    let p = pool.clone();
    let facture = r.supplier_invoice_id;
    let annulation = tokio::spawn(async move {
        retry_on_deadlock("test::supplier_invoices::cancel", || {
            supplier_invoices::cancel(&p, c, facture, u)
        })
        .await
    });
    let vues =
        attendre_des_requetes_en_cours(&pool, &["FROM supplier_invoices", "FOR UPDATE"], 2, || {
            derapprochement.is_finished() || annulation.is_finished()
        })
        .await;
    assert!(
        vues,
        "les deux gestes devaient attendre la facture — dé-rapprochement fini : {}, \
         annulation finie : {}",
        derapprochement.is_finished(),
        annulation.is_finished()
    );
    tenue.rollback().await.unwrap();

    let d = derapprochement.await.unwrap();
    let a = annulation.await.unwrap();
    assert!(d.is_ok(), "dé-rapprochement : {d:?}");
    assert!(a.is_ok(), "annulation : {:?}", a.map(|_| ()));

    let (status, matched): (String, Option<i64>) =
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(bt)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((status.as_str(), matched), ("pending", None));
    let (statut, reglement): (String, Option<i64>) = sqlx::query_as(
        "SELECT status, settlement_journal_entry_id FROM supplier_invoices WHERE id = ?",
    )
    .bind(facture)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((statut.as_str(), reglement), ("cancelled", None));
    for (origine, nom) in [
        (r.settlement_entry_id, "le règlement"),
        (r.purchase_entry_id, "l'achat"),
    ] {
        assert_eq!(
            compter(
                &pool,
                "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?",
                origine
            )
            .await,
            1,
            "{nom} est contre-passé exactement une fois"
        );
    }
}

/// T-a11 (AC-a4) — **dé-rapprochement pendant la clôture de l'exercice du
/// règlement**. La clôture tient l'exercice 2019 sans commiter ; le
/// dé-rapprochement verrouille la transaction, la facture (classement
/// verrouillant), puis attend l'exercice ; la clôture commite. Il doit alors
/// refuser l'exercice clos, rien d'écrit. Tue M-a5 : une lecture de classement
/// non verrouillante ferait naître la vue **avant** le verrou de l'exercice,
/// et les motifs liraient l'exercice encore ouvert.
///
/// ⚠️ La clôture est jouée par son effet (`status = 'Closed'` dans une
/// transaction tenue ouverte), patron de
/// `a_concurrent_close_waits_for_the_unreconciliation` : la clôture réelle
/// exige un exercice suivant et des écritures de bouclement que le montage n'a
/// pas, et c'est le verrou de la ligne `fiscal_years` qui est en jeu.
#[sqlx::test(migrations = "./test-schema")]
async fn a_supplier_unreconciliation_waits_for_a_concurrent_close(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let fy_2019 = exercice_2019(&pool, seeded.company_id).await;
    let f = preparer(&pool, &seeded).await;
    let r = reglement_rapproche(&pool, &seeded, f, "FF-2019", jour(2019, 6, 3), dec!(75.50)).await;
    let ecritures = compter(
        &pool,
        "SELECT COUNT(*) FROM journal_entries WHERE company_id = ?",
        seeded.company_id,
    )
    .await;

    let mut closing = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_2019)
        .execute(&mut *closing)
        .await
        .unwrap();

    let p = pool.clone();
    let (c, u, bt) = (
        seeded.company_id,
        seeded.admin_user_id,
        r.bank_transaction_id,
    );
    let annulation = tokio::spawn(async move { reconciliation_cancel::cancel(&p, c, bt, u).await });
    let vue = attendre_des_requetes_en_cours(&pool, &["fiscal_years", "FOR UPDATE"], 1, || {
        annulation.is_finished()
    })
    .await;
    if !vue {
        panic!(
            "le dé-rapprochement a fini sans attendre l'exercice : {:?}",
            annulation.await
        );
    }
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
    assert_supplier_link_intact(&pool, &seeded, r, ecritures).await;
}

/// T-a12 (AC-a4) — **le plan de la lecture verrouillante du classement** passe
/// par l'index de `settlement_journal_entry_id`. Un parcours par
/// `idx_supplier_invoices_company_status` verrouillerait, en forme
/// verrouillante, **toutes** les factures payées de la société. Le décor en
/// porte trois, pour que l'index de la société ne soit pas le plus sélectif
/// par accident. L'`EXPLAIN` exige des valeurs : chaque `?` est remplacé,
/// dans l'ordre, et le test vérifie qu'il n'en reste aucun.
#[sqlx::test(migrations = "./test-schema")]
async fn the_supplier_classification_reads_the_settlement_entry_index(pool: MySqlPool) {
    use sqlx::Row;
    let (seeded, r) = monter_fournisseur(&pool).await;
    let f = preparer_sans_reglages(&pool, &seeded, r).await;
    for (numero, montant) in [("FF-PLAN-2", dec!(10.00)), ("FF-PLAN-3", dec!(20.00))] {
        fournisseur_rapproche::facture_payee(&pool, &seeded, f, numero, jour(2026, 6, 16), montant)
            .await;
    }
    let lie = reconciliation_cancel::SUPPLIER_SETTLEMENT_CLASSIFY_SQL
        .replacen('?', &seeded.company_id.to_string(), 1)
        .replacen('?', &r.settlement_entry_id.to_string(), 1);
    assert!(!lie.contains('?'), "un `?` reste à lier : {lie}");
    let lignes = sqlx::query(&format!("EXPLAIN {lie}"))
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(lignes.len(), 1, "{lie}");
    let cle: Option<String> = lignes[0].try_get("key").unwrap();
    assert_eq!(
        cle.as_deref(),
        Some("fk_supplier_invoices_settlement_je"),
        "⛔ le classement verrouillerait d'autres factures que la sienne"
    );
}
