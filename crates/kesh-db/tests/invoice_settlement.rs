//! Règlement manuel d'une facture client — Story 24-3 (#372).
//!
//! ⚠️ **Ce que ces tests protègent, et pourquoi ils sont ici.** Le mode de
//! règlement est **indifférent au traitement comptable** : espèces, poste,
//! compensation ou virement, seule change la contrepartie. C'est cette
//! indifférence qui doit tenir dans le temps — la tentation de traiter le cash
//! « plus simplement » est exactement ce qui a produit le défaut de #372.
//!
//! ⛔ **Et s'il fallait plus de rigueur d'un côté, ce serait le cash** : les
//! espèces n'ont aucune trace externe, ce qui en fait la zone la plus scrutée
//! par l'AFC. Une caisse ne peut jamais être créditrice.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    NewInvoice, NewInvoiceLine, NewJournalEntry, NewJournalEntryLine, SettlementChoice,
};
use kesh_db::repositories::{invoice_settlements, invoice_settlements_write, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

/// Une facture validée de `amount`, avec son écriture de vente
/// `D créance / C produit` — sans quoi il n'y aurait rien à solder.
async fn validated_invoice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    amount: Decimal,
    date: NaiveDate,
) -> i64 {
    let receivable = seeded.accounts["1100"];
    let revenue = seeded.accounts["3000"];

    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client règlement', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .expect("contact");

    let inv = invoices::create(
        pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date,
            due_date: Some(date),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: amount,
                vat_rate: dec!(0),
                revenue_account_id: Some(revenue),
            }],
        },
    )
    .await
    .expect("facture créée")
    .0;

    // Écriture de vente posée à la main : ce fichier teste le RÈGLEMENT, pas la
    // validation. Une ligne de débit unique sur la créance — l'invariante que
    // `settle_invoice` exploite.
    let je = kesh_db::repositories::journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date,
            journal: Journal::Ventes,
            description: "Vente".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: receivable,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: revenue,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture de vente");

    sqlx::query("UPDATE invoices SET status = 'validated', journal_entry_id = ? WHERE id = ?")
        .bind(je.entry.id)
        .bind(inv.id)
        .execute(pool)
        .await
        .expect("validation de montage");

    inv.id
}

/// Solde d'un compte — `débit − crédit` sur toutes ses lignes.
async fn solde(pool: &MySqlPool, account_id: i64) -> Decimal {
    sqlx::query_scalar::<_, Decimal>(
        "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM journal_entry_lines \
         WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .expect("solde")
}

/// ⛔ **Un règlement en ESPÈCES produit son écriture, exactement comme un
/// virement.** C'est le défaut de #372 : jusqu'ici, seul le chemin bancaire
/// comptabilisait.
#[sqlx::test(migrations = "./test-schema")]
async fn un_reglement_en_especes_meut_la_caisse(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let creance = seeded.accounts["1100"];
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;

    let out = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount { account_id: caisse },
        dec!(100.00),
        ymd(2026, 3, 5),
    )
    .await
    .expect("règlement espèces");

    assert!(out.fully_settled);
    assert_eq!(out.amount_due_after, Decimal::ZERO);
    assert_eq!(
        solde(&pool, caisse).await,
        dec!(100.00),
        "la caisse est DÉBITÉE — c'est ce que « payé en espèces » veut dire"
    );
    assert_eq!(
        solde(&pool, creance).await,
        Decimal::ZERO,
        "⛔ L'INVARIANTE : la créance se solde exactement"
    );

    let paid_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_some(), "solde nul ⇒ la facture est payée");
}

/// ⛔ **Deux MODES sur une même facture.** C'est ce qu'aucune colonne unique sur
/// `invoices` ne pourrait dire, et la raison pour laquelle le mode vit sur le
/// règlement.
#[sqlx::test(migrations = "./test-schema")]
async fn une_facture_se_regle_en_especes_puis_en_banque(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let creance = seeded.accounts["1100"];
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;

    // Un compte bancaire câblé sur un compte du grand livre.
    let bank_ledger: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, '1020', 'Banque', 'Asset', 1, 1) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .expect("compte banque");
    let bank_account_id: i64 = sqlx::query_scalar(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
         VALUES (?, 'UBS', 'CH4431999123000889012', TRUE, ?) RETURNING id",
    )
    .bind(seeded.company_id)
    .bind(bank_ledger)
    .fetch_one(&pool)
    .await
    .expect("compte bancaire");

    let out1 = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount { account_id: caisse },
        dec!(60.00),
        ymd(2026, 3, 5),
    )
    .await
    .expect("acompte espèces");
    assert!(
        !out1.fully_settled,
        "⛔ 60 sur 100 ne SOLDE pas — sinon la facture sortirait des relances"
    );
    assert_eq!(out1.amount_due_after, dec!(40.00));

    let out2 = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::BankTransfer { bank_account_id },
        dec!(40.00),
        ymd(2026, 3, 10),
    )
    .await
    .expect("solde par virement");
    assert!(out2.fully_settled);

    assert_eq!(solde(&pool, caisse).await, dec!(60.00));
    assert_eq!(solde(&pool, bank_ledger).await, dec!(40.00));
    assert_eq!(
        solde(&pool, creance).await,
        Decimal::ZERO,
        "la créance se solde, quels que soient les modes employés"
    );

    let modes: Vec<String> = sqlx::query_scalar(
        "SELECT settlement_type FROM invoice_settlements WHERE invoice_id = ? ORDER BY id",
    )
    .bind(inv_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(modes, vec!["internal_account", "bank_transfer"]);
}

/// ⛔ **Le trop-perçu est refusé, et RIEN n'est écrit.** Sinon la créance
/// passerait créditrice.
#[sqlx::test(migrations = "./test-schema")]
async fn le_trop_percu_est_refuse_sans_rien_ecrire(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let creance = seeded.accounts["1100"];
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;

    let err = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount { account_id: caisse },
        dec!(150.00),
        ymd(2026, 3, 5),
    )
    .await
    .expect_err("un trop-perçu doit être refusé");
    assert!(format!("{err:?}").contains("overpayment"), "got {err:?}");

    assert_eq!(solde(&pool, caisse).await, Decimal::ZERO, "caisse intacte");
    assert_eq!(
        solde(&pool, creance).await,
        dec!(100.00),
        "la créance est intacte"
    );
    assert_eq!(
        invoice_settlements::list_for_invoice(&pool, seeded.company_id, inv_id)
            .await
            .unwrap()
            .len(),
        0
    );
}

/// ⛔ **Un compte ARCHIVÉ est refusé.** Régler sur un compte qu'aucun écran ne
/// montre plus produirait une écriture invisible.
#[sqlx::test(migrations = "./test-schema")]
async fn un_compte_archive_est_refuse(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;

    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(caisse)
        .execute(&pool)
        .await
        .unwrap();

    let err = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount { account_id: caisse },
        dec!(100.00),
        ymd(2026, 3, 5),
    )
    .await
    .expect_err("un compte archivé doit être refusé");
    assert!(
        format!("{err:?}").contains("InactiveOrInvalid"),
        "got {err:?}"
    );
}

/// ⛔ **Un compte NON IMPUTABLE est refusé** — et c'est ce test qui manquait.
///
/// Ce flux écrit avec `enforce_postable = false` : la garde de la Story 14-3b ne
/// s'applique donc PAS, et seul le `SELECT active, postable` de
/// `settle_invoice` tient la promesse. Sans lui, « Enregistrer un règlement » →
/// « Compte interne » → 9000 rouvrait, **d'un geste ordinaire**, le défaut que
/// la Story 24-5 ferme : le menu de l'écran ne filtrait que `active`, et le
/// compte de clôture y figurait.
///
/// ⛔ Relevé en passe 3 de revue de code de la Story 24-5 (#375), finding P3-1 —
/// CRITICAL. Le test d'à côté n'éprouvait que l'archivage : *un flux qui n'a
/// qu'une garde testée sur deux est un flux dont la seconde peut disparaître
/// sans que rien ne rougisse.*
#[sqlx::test(migrations = "./test-schema")]
async fn un_compte_non_imputable_est_refuse(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;

    // Le compte reste ACTIF — c'est bien `postable` seul qui doit refuser.
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(caisse)
        .execute(&pool)
        .await
        .unwrap();

    let err = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount { account_id: caisse },
        dec!(100.00),
        ymd(2026, 3, 5),
    )
    .await
    .expect_err("un compte non imputable doit être refusé");
    assert!(
        format!("{err:?}").contains("InactiveOrInvalid"),
        "got {err:?}"
    );

    // Et rien n'a été écrit : ni règlement, ni écriture.
    assert_eq!(
        invoice_settlements::list_for_invoice(&pool, seeded.company_id, inv_id)
            .await
            .unwrap()
            .len(),
        0,
        "aucun règlement ne doit subsister"
    );
    assert_eq!(
        solde(&pool, caisse).await,
        Decimal::ZERO,
        "le compte refusé reste intact"
    );
}

/// ⛔ **Un règlement ne PRÉCÈDE pas sa facture** — la seule garde de
/// `mark_as_paid` qui reste vraie, et qui a été portée plutôt que perdue.
#[sqlx::test(migrations = "./test-schema")]
async fn un_reglement_ne_precede_pas_sa_facture(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 10)).await;

    let err = invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount { account_id: caisse },
        dec!(100.00),
        ymd(2026, 3, 1),
    )
    .await
    .expect_err("un règlement antérieur à sa facture doit être refusé");
    assert!(
        format!("{err:?}").contains("settledOnBeforeInvoiceDate"),
        "got {err:?}"
    );

    // ⚠️ La veille reste ACCEPTÉE : `settled_on` est une date de valeur bancaire
    // et `invoice.date` une date métier locale — l'écart de fuseau suffit à
    // faire apparaître un règlement « la veille » d'une facture du même jour.
    invoice_settlements_write::settle_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount { account_id: caisse },
        dec!(100.00),
        ymd(2026, 3, 9),
    )
    .await
    .expect("la tolérance d'un jour doit passer");
}

// ===========================================================================
// Story 25-3-a-1 (#414) — annuler un règlement client
// ===========================================================================

use kesh_db::errors::{DbError, ReversalBlocker, SettlementCancelBlocker};
use kesh_db::repositories::journal_entries::{self, ReversalAuthority};
use kesh_db::repositories::settlement_cancellation::settlement_entry_cancel_blocker;
use kesh_db::repositories::{accounts, credit_notes, fiscal_years};

/// Règle `amount` en espèces et rend `(settlement_id, journal_entry_id)`.
async fn settle_cash(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    inv_id: i64,
    amount: Decimal,
    on: NaiveDate,
) -> (i64, i64) {
    let out = invoice_settlements_write::settle_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        amount,
        on,
    )
    .await
    .expect("règlement");
    let sid: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
            .bind(out.journal_entry_id)
            .fetch_one(pool)
            .await
            .expect("ligne de règlement");
    (sid, out.journal_entry_id)
}

/// Rapproche l'écriture `entry_id` d'une transaction bancaire et rend l'id de
/// la transaction.
///
/// ⚠️ **Montage en SQL, et dit tel** : le rapprochement réel
/// (`accept_one_invoice`) vit dans `kesh-api`. Le chemin réel est exercé par
/// `reconciliation_e2e.rs::cancelling_a_reconciled_settlement_is_refused` ;
/// ici, on n'a besoin que de l'ÉTAT qu'il produit — `matched_entry_id`.
async fn match_to_bank(pool: &MySqlPool, company_id: i64, entry_id: i64, on: NaiveDate) -> i64 {
    let bank_account = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) \
         VALUES (?, 'Banque test', 'CH9300762011623852957')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("compte bancaire")
    .last_insert_id() as i64;
    let user_id: i64 = sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("utilisateur");
    let import_id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'test.xml', REPEAT('a', 64), 'camt053', ?, ?, ?)",
    )
    .bind(company_id)
    .bind(bank_account)
    .bind(on)
    .bind(on)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("import")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, matched_entry_id) \
         VALUES (?, ?, ?, ?, 40.00, 'CHF', 'test', ?)",
    )
    .bind(company_id)
    .bind(import_id)
    .bind(bank_account)
    .bind(on)
    .bind(entry_id)
    .execute(pool)
    .await
    .expect("transaction rapprochée")
    .last_insert_id() as i64
}

async fn count(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.expect(sql)
}

/// ⛔ **Le geste nominal** : contre-passation datée du jour, ligne retirée,
/// facture remise « à régler », deux lignes d'audit — et l'écriture d'origine
/// comme sa contre-passation deviennent définitivement intouchables.
#[sqlx::test(migrations = "./test-schema")]
async fn annuler_l_unique_reglement_remet_la_facture_a_regler(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let (caisse, creance) = (seeded.accounts["1000"], seeded.accounts["1100"]);
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (sid, entry_id) = settle_cash(&pool, &seeded, inv_id, dec!(100.00), ymd(2026, 3, 5)).await;
    let version_avant: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(inv_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let done = invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        sid,
    )
    .await
    .expect("annulation");

    assert_eq!(done.amount_due_after, dec!(100.00), "tout redevient dû");
    let (paid_at, version): (Option<chrono::NaiveDateTime>, i32) =
        sqlx::query_as("SELECT paid_at, version FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_none(), "résiduel positif ⇒ paid_at retombe");
    assert_eq!(version, version_avant + 1, "une annulation change l'état");
    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM invoice_settlements WHERE id = {sid}")
        )
        .await,
        0,
        "la ligne est RETIRÉE, non marquée"
    );
    let reverses: Option<i64> =
        sqlx::query_scalar("SELECT reverses_entry_id FROM journal_entries WHERE id = ?")
            .bind(done.reversal_journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reverses, Some(entry_id), "une vraie contre-passation, liée");
    assert_eq!(
        solde(&pool, caisse).await,
        Decimal::ZERO,
        "la caisse est rendue"
    );
    assert_eq!(
        solde(&pool, creance).await,
        dec!(100.00),
        "la créance renaît"
    );

    let origine = journal_entries::reversal_blocker(&pool, seeded.company_id, entry_id)
        .await
        .unwrap();
    assert_eq!(origine.map(|h| h.0), Some(ReversalBlocker::AlreadyReversed));
    let inverse =
        journal_entries::reversal_blocker(&pool, seeded.company_id, done.reversal_journal_entry_id)
            .await
            .unwrap();
    assert_eq!(inverse.map(|h| h.0), Some(ReversalBlocker::IsAReversal));

    assert_eq!(
        count(
            &pool,
            &format!(
                "SELECT COUNT(*) FROM audit_log WHERE action = 'invoice.settlement_cancelled' \
                 AND entity_id = {inv_id}"
            )
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            &format!(
                "SELECT COUNT(*) FROM audit_log WHERE action = 'journal_entry.reversed' \
                 AND entity_id = {entry_id}"
            )
        )
        .await,
        1,
        "le socle écrit AUSSI sa ligne : deux lignes, c'est voulu"
    );
}

/// ⛔ **Un règlement parmi deux** : la facture reste partiellement réglée.
/// `paid_at` ne retombe que si le résiduel redevient positif — ici il l'est,
/// mais l'autre règlement, lui, reste.
#[sqlx::test(migrations = "./test-schema")]
async fn annuler_un_reglement_parmi_deux(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (s1, _) = settle_cash(&pool, &seeded, inv_id, dec!(60.00), ymd(2026, 3, 5)).await;
    let (s2, _) = settle_cash(&pool, &seeded, inv_id, dec!(40.00), ymd(2026, 3, 6)).await;

    let done = invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        s2,
    )
    .await
    .expect("annulation");

    assert_eq!(done.amount_due_after, dec!(40.00));
    let paid_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_none());
    let restants = invoice_settlements::list_for_invoice(&pool, seeded.company_id, inv_id)
        .await
        .unwrap();
    assert_eq!(restants.len(), 1);
    assert_eq!(restants[0].id, s1, "l'AUTRE règlement reste");
}

/// Branche **défensive** : un résiduel resté ≤ 0 laisse `paid_at` intact.
///
/// ⚠️ **État FORGÉ, et dit tel** : l'application ne le produit pas — le
/// trop-perçu est refusé par l'encaissement et par le rapprochement. On insère
/// en SQL un second règlement qui couvre à lui seul la facture, pour que
/// l'annulation du premier laisse un résiduel nul.
#[sqlx::test(migrations = "./test-schema")]
async fn residuel_reste_nul_laisse_paid_at(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (s1, _) = settle_cash(&pool, &seeded, inv_id, dec!(100.00), ymd(2026, 3, 5)).await;
    let forged_entry = journal_entries::create(
        &pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: ymd(2026, 3, 6),
            journal: Journal::OD,
            description: "forgé".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts["1000"],
                    debit: dec!(100.00),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["1100"],
                    debit: Decimal::ZERO,
                    credit: dec!(100.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
         settled_on, settlement_type, settlement_account_id) \
         VALUES (?, ?, ?, 100.00, ?, 'internal_account', ?)",
    )
    .bind(seeded.company_id)
    .bind(inv_id)
    .bind(forged_entry.entry.id)
    .bind(ymd(2026, 3, 6))
    .bind(seeded.accounts["1000"])
    .execute(&pool)
    .await
    .unwrap();

    let done = invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv_id,
        s1,
    )
    .await
    .expect("annulation");
    assert_eq!(done.amount_due_after, Decimal::ZERO);
    let paid_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_some(), "résiduel nul ⇒ paid_at intact");
}

/// La seconde annulation du même règlement ne trouve plus rien.
#[sqlx::test(migrations = "./test-schema")]
async fn double_annulation_rend_not_found(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (sid, _) = settle_cash(&pool, &seeded, inv_id, dec!(100.00), ymd(2026, 3, 5)).await;
    let args = (seeded.admin_user_id, seeded.company_id, inv_id, sid);
    invoice_settlements_write::cancel_settlement(&pool, args.0, args.1, args.2, args.3)
        .await
        .expect("première");
    let err = invoice_settlements_write::cancel_settlement(&pool, args.0, args.1, args.2, args.3)
        .await
        .expect_err("seconde");
    assert!(matches!(err, DbError::NotFound), "got {err:?}");
}

/// ⛔ **Le socle — exemption ÉTROITE.** L'autorité d'un règlement ne lève pas
/// le motif d'un AUTRE règlement.
#[sqlx::test(migrations = "./test-schema")]
async fn socle_l_autorite_ne_leve_que_son_reglement(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (s1, e1) = settle_cash(&pool, &seeded, inv_id, dec!(60.00), ymd(2026, 3, 5)).await;
    let (s2, _) = settle_cash(&pool, &seeded, inv_id, dec!(40.00), ymd(2026, 3, 6)).await;
    assert_ne!(s1, s2);

    let mut tx = pool.begin().await.unwrap();
    let err = journal_entries::reverse_owned_in_tx(
        &mut tx,
        seeded.company_id,
        e1,
        seeded.admin_user_id,
        ReversalAuthority::ClientSettlement { settlement_id: s2 },
    )
    .await
    .expect_err("l'autorité de s2 ne couvre pas l'écriture de s1");
    assert!(
        matches!(
            err,
            DbError::EntryNotReversable {
                blocker: ReversalBlocker::OwnedBySettlement,
                ..
            }
        ),
        "got {err:?}"
    );
}

/// ⛔ **Le socle — la précédence se POURSUIT après le motif levé.** Appel
/// DIRECT de la variante sur une écriture de règlement rapprochée : le motif du
/// règlement est levé, celui du rapprochement reste. *Sans cela, un paiement
/// que la banque dit rapproché serait contre-passé en silence.*
#[sqlx::test(migrations = "./test-schema")]
async fn socle_la_precedence_se_poursuit_apres_le_motif_leve(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (sid, entry_id) = settle_cash(&pool, &seeded, inv_id, dec!(40.00), ymd(2026, 3, 5)).await;
    let bt = match_to_bank(&pool, seeded.company_id, entry_id, ymd(2026, 3, 5)).await;

    let mut tx = pool.begin().await.unwrap();
    let err = journal_entries::reverse_owned_in_tx(
        &mut tx,
        seeded.company_id,
        entry_id,
        seeded.admin_user_id,
        ReversalAuthority::ClientSettlement { settlement_id: sid },
    )
    .await
    .expect_err("un règlement rapproché ne se contre-passe pas");
    match err {
        DbError::EntryNotReversable {
            blocker: ReversalBlocker::MatchedBankTransaction,
            document_id,
            ..
        } => assert_eq!(document_id, Some(bt)),
        other => panic!("attendu MATCHED_BANK_TRANSACTION, reçu {other:?}"),
    }
}

/// ⛔ **Composition et rollback** — le cas réel de la 25-3-b, qui appelle
/// `cancel_settlement_in_tx` dans sa propre transaction
/// (`reconciliation_cancel::cancel_in_tx`).
///
/// La lecture **positive** dans la transaction prouve que le geste a bien
/// écrit ; l'abandon prouve qu'il n'a rien commité. *Un test de rollback aux
/// assertions toutes négatives serait satisfait par « jamais écrit ».*
#[sqlx::test(migrations = "./test-schema")]
async fn le_geste_se_compose_et_s_annule_avec_sa_transaction(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (sid, entry_id) = settle_cash(&pool, &seeded, inv_id, dec!(100.00), ymd(2026, 3, 5)).await;
    let entries_avant = count(&pool, "SELECT COUNT(*) FROM journal_entries").await;

    {
        let mut tx = pool.begin().await.unwrap();
        invoice_settlements_write::cancel_settlement_in_tx(
            &mut tx,
            seeded.company_id,
            inv_id,
            sid,
            seeded.admin_user_id,
        )
        .await
        .expect("annulation dans la transaction");
        let inside: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?")
                .bind(entry_id)
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        assert_eq!(inside, 1, "DANS la transaction, la contre-passation existe");
        let row: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE id = ?")
            .bind(sid)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert_eq!(row, 0, "DANS la transaction, la ligne a disparu");
        // Drop sans commit : l'étape suivante de l'appelant a échoué.
    }

    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM journal_entries").await,
        entries_avant,
        "aucune écriture inverse n'a survécu"
    );
    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM invoice_settlements WHERE id = {sid}")
        )
        .await,
        1,
        "la ligne est revenue"
    );
}

/// ⛔ **Étanchéité multi-tenant** : une autre société ne peut ni voir ni
/// annuler le règlement — et rien n'est écrit, table par table.
#[sqlx::test(migrations = "./test-schema")]
async fn une_autre_societe_ne_peut_pas_annuler(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (sid, _) = settle_cash(&pool, &seeded, inv_id, dec!(100.00), ymd(2026, 3, 5)).await;
    let other = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    let tables = [
        "journal_entries",
        "journal_entry_lines",
        "invoice_settlements",
        "invoices",
        "audit_log",
    ];
    let mut avant = Vec::new();
    for t in tables {
        avant.push(count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await);
    }
    let invoices_version: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(inv_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        other,
        inv_id,
        sid,
    )
    .await
    .expect_err("autre société");
    assert!(matches!(err, DbError::NotFound), "got {err:?}");
    let mut conn = pool.acquire().await.unwrap();
    let err = invoice_settlements_write::settlement_cancel_blocker(&mut conn, other, sid)
        .await
        .expect_err("autre société — lecture");
    assert!(matches!(err, DbError::NotFound), "got {err:?}");

    for (t, n) in tables.iter().zip(avant) {
        assert_eq!(
            count(&pool, &format!("SELECT COUNT(*) FROM {t}")).await,
            n,
            "rien d'écrit dans {t}"
        );
    }
    let v: i32 = sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(inv_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(v, invoices_version, "la facture n'a pas bougé");
}

// ---------------------------------------------------------------------------
// La précédence : chaque rang seul, et chaque paire — lecture ET écriture.
// ---------------------------------------------------------------------------

/// Les cinq rangs de [`SettlementCancelBlocker`], dans l'ordre.
const RANGS: [SettlementCancelBlocker; 5] = [
    SettlementCancelBlocker::InvoiceCredited,
    SettlementCancelBlocker::FiscalYearClosed,
    SettlementCancelBlocker::MatchedBankTransaction,
    SettlementCancelBlocker::AccountArchived,
    SettlementCancelBlocker::NoOpenFiscalYearToday,
];

/// Monte un règlement qui porte exactement les motifs de `motifs`, par les
/// chemins réels (sauf le rapprochement, cf. [`match_to_bank`]).
///
/// Calendrier : l'exercice seedé est **raccourci** à `D = aujourd'hui − 60 j`
/// (montage SQL : la fixture le fait courir jusqu'en 2030). Le règlement est
/// daté `D − 10`. Un second exercice `[D + 1, 2030-12-31]` est créé par le vrai
/// chemin, **sauf** pour le rang 5 — c'est ce qui laisse le jour sans exercice
/// ouvert (le cas de janvier). ⚠️ Le montage « régler aujourd'hui puis clore »
/// produirait le rang **2**, pas le 5.
///
/// Ordre des gestes : régler, créditer, rapprocher, archiver, clore — la
/// clôture en dernier, pour que les gestes précédents trouvent leur exercice
/// ouvert.
async fn monter(pool: &MySqlPool, motifs: &[SettlementCancelBlocker]) -> (SeededCompany, i64, i64) {
    kesh_db::test_fixtures::truncate_all(pool)
        .await
        .expect("truncate");
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let today = chrono::Utc::now().date_naive();
    let d = today - chrono::Duration::days(60);
    sqlx::query("UPDATE fiscal_years SET end_date = ? WHERE id = ?")
        .bind(d)
        .bind(seeded.fiscal_year_id)
        .execute(pool)
        .await
        .expect("raccourcir l'exercice");
    if !motifs.contains(&SettlementCancelBlocker::NoOpenFiscalYearToday) {
        fiscal_years::create(
            pool,
            seeded.admin_user_id,
            kesh_db::entities::NewFiscalYear {
                company_id: seeded.company_id,
                name: "Courant".into(),
                start_date: d + chrono::Duration::days(1),
                end_date: ymd(2030, 12, 31),
            },
        )
        .await
        .expect("exercice courant");
    }

    let inv_id =
        validated_invoice(pool, &seeded, dec!(100.00), d - chrono::Duration::days(20)).await;
    let (sid, entry_id) = settle_cash(
        pool,
        &seeded,
        inv_id,
        dec!(40.00),
        d - chrono::Duration::days(10),
    )
    .await;

    if motifs.contains(&SettlementCancelBlocker::InvoiceCredited) {
        credit_notes::create_credit_note(
            pool,
            kesh_db::entities::NewCreditNote {
                company_id: seeded.company_id,
                invoice_id: inv_id,
                date: d - chrono::Duration::days(5),
            },
            seeded.admin_user_id,
        )
        .await
        .expect("avoir après règlement partiel — le vrai chemin l'accepte");
    }
    if motifs.contains(&SettlementCancelBlocker::MatchedBankTransaction) {
        match_to_bank(
            pool,
            seeded.company_id,
            entry_id,
            d - chrono::Duration::days(10),
        )
        .await;
    }
    if motifs.contains(&SettlementCancelBlocker::AccountArchived) {
        let caisse = seeded.accounts["1000"];
        let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
            .bind(caisse)
            .fetch_one(pool)
            .await
            .unwrap();
        accounts::archive(pool, caisse, version, seeded.admin_user_id)
            .await
            .expect("archivage de la caisse");
    }
    if motifs.contains(&SettlementCancelBlocker::FiscalYearClosed) {
        fiscal_years::close(
            pool,
            seeded.admin_user_id,
            seeded.company_id,
            seeded.fiscal_year_id,
        )
        .await
        .expect("clôture");
    }
    (seeded, inv_id, sid)
}

/// L'erreur que l'ÉCRITURE doit rendre pour un motif donné.
fn ecriture_attendue(motif: SettlementCancelBlocker, err: &DbError) -> bool {
    match motif {
        SettlementCancelBlocker::InvoiceCredited
        | SettlementCancelBlocker::SupplierInvoiceNotPaid
        | SettlementCancelBlocker::FiscalYearClosed => {
            matches!(err, DbError::SettlementNotCancellable { blocker } if *blocker == motif)
        }
        SettlementCancelBlocker::MatchedBankTransaction => matches!(
            err,
            DbError::EntryNotReversable {
                blocker: ReversalBlocker::MatchedBankTransaction,
                ..
            }
        ),
        SettlementCancelBlocker::AccountArchived => {
            matches!(err, DbError::ReversalAccountsArchived(_))
        }
        SettlementCancelBlocker::NoOpenFiscalYearToday => {
            matches!(err, DbError::FiscalYearInvalid)
        }
        // Tête du dé-rapprochement (25-3-b) et motifs de l'annulation d'une
        // facture fournisseur (25-3-c) : jamais produits par l'annulation
        // d'un règlement.
        SettlementCancelBlocker::BankTransactionNotReconciled
        | SettlementCancelBlocker::SupplierInvoiceCancelled
        | SettlementCancelBlocker::SupplierInvoiceInPaymentBatch => false,
    }
}

/// ⛔ **Chaque rang seul, puis chaque paire : la lecture annonce le rang le
/// plus fort, et l'écriture refuse pour CE MÊME motif.** C'est ce qui prouve
/// que la lecture suit l'écriture — en particulier la paire 4-5, où le socle
/// contrôle les comptes archivés avant l'exercice du jour.
#[sqlx::test(migrations = "./test-schema")]
async fn la_precedence_de_l_annulation_lecture_et_ecriture(pool: MySqlPool) {
    let mut cas: Vec<Vec<SettlementCancelBlocker>> = RANGS.iter().map(|r| vec![*r]).collect();
    for (i, fort) in RANGS.iter().enumerate() {
        for faible in &RANGS[i + 1..] {
            cas.push(vec![*fort, *faible]);
        }
    }
    assert_eq!(cas.len(), 15, "cinq seuls, dix paires");

    for motifs in cas {
        let (seeded, inv_id, sid) = monter(&pool, &motifs).await;
        let attendu = motifs[0]; // ⚠️ `RANGS` est ordonné : le premier est le plus fort.

        let mut conn = pool.acquire().await.unwrap();
        let lu =
            invoice_settlements_write::settlement_cancel_blocker(&mut conn, seeded.company_id, sid)
                .await
                .expect("lecture");
        drop(conn);
        assert_eq!(
            lu.as_ref().map(|h| h.0),
            Some(attendu),
            "lecture, motifs {motifs:?}"
        );
        if attendu == SettlementCancelBlocker::AccountArchived {
            assert_eq!(
                lu.and_then(|h| h.2).as_deref(),
                Some("1000"),
                "le NUMÉRO du compte accompagne le motif"
            );
        }

        let err = invoice_settlements_write::cancel_settlement(
            &pool,
            seeded.admin_user_id,
            seeded.company_id,
            inv_id,
            sid,
        )
        .await
        .expect_err("l'annulation doit être refusée");
        assert!(
            ecriture_attendue(attendu, &err),
            "écriture, motifs {motifs:?} : attendu {attendu:?}, reçu {err:?}"
        );
        assert_eq!(
            count(
                &pool,
                &format!("SELECT COUNT(*) FROM invoice_settlements WHERE id = {sid}")
            )
            .await,
            1,
            "refusée ⇒ rien de retiré, motifs {motifs:?}"
        );
    }
}

/// La queue commune, **à son propre niveau** — sur un `entry_id`, sans rien
/// savoir de la pièce : c'est ce que la 25-3-a-2 réutilisera telle quelle.
#[sqlx::test(migrations = "./test-schema")]
async fn la_queue_commune_se_lit_sur_l_ecriture(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let inv_id = validated_invoice(&pool, &seeded, dec!(100.00), ymd(2026, 3, 1)).await;
    let (_, entry_id) = settle_cash(&pool, &seeded, inv_id, dec!(40.00), ymd(2026, 3, 5)).await;
    let mut conn = pool.acquire().await.unwrap();

    assert_eq!(
        settlement_entry_cancel_blocker(&mut conn, seeded.company_id, entry_id, None)
            .await
            .unwrap(),
        None,
        "rien n'empêche : la queue ne voit PAS le motif de propriété"
    );
    let bt = match_to_bank(&pool, seeded.company_id, entry_id, ymd(2026, 3, 5)).await;
    assert_eq!(
        settlement_entry_cancel_blocker(&mut conn, seeded.company_id, entry_id, None)
            .await
            .unwrap(),
        Some((
            SettlementCancelBlocker::MatchedBankTransaction,
            Some(bt),
            None
        )),
        "l'identifiant de la transaction accompagne le motif"
    );
    let err = settlement_entry_cancel_blocker(&mut conn, seeded.company_id + 999, entry_id, None)
        .await
        .expect_err("autre société");
    assert!(matches!(err, DbError::NotFound), "got {err:?}");
}

/// ⛔ **Une clôture concurrente ne passe pas entre la lecture du rang 2 et la
/// contre-passation** (passe 1 de revue de code ; synchronisation rendue
/// déterministe en passe 2).
///
/// ⚠️ **Pourquoi un ENTRELACEMENT, et non un état final.** Le socle verrouille
/// lui aussi l'exercice de l'écriture d'origine — mais à l'étape 4, APRÈS la
/// lecture du rang 2. Un test qui ne regarde que la fin verrait donc un verrou
/// dans les deux cas et ne prouverait rien (vérifié : une première rédaction
/// passait sous la mutation qui retire le verrou). Ici :
/// 1. une clôture est EN COURS, non validée — elle tient la ligne d'exercice ;
/// 2. l'annulation démarre pendant ce temps ;
/// 3. la clôture est validée.
///
/// **Avec** le verrou de l'étape 2-bis, l'annulation attend la clôture, puis lit
/// l'exercice CLOS et refuse. **Sans** lui, elle lit l'exercice encore ouvert
/// (la clôture n'est pas validée), passe le rang 2, attend au verrou du socle…
/// puis contre-passe un règlement d'exercice désormais clos.
///
/// Le règlement est dans un exercice **autre** que celui du jour, pour que le
/// verrou que le socle pose sur l'exercice du jour ne serve pas de garde par
/// accident.
#[sqlx::test(migrations = "./test-schema")]
async fn une_cloture_concurrente_attend_l_annulation(pool: MySqlPool) {
    let (seeded, inv_id, sid) = monter(&pool, &[]).await;

    // (1) La clôture en cours, non validée.
    let mut closing = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(&mut *closing)
        .await
        .unwrap();

    // (2) L'annulation démarre pendant ce temps.
    let p = pool.clone();
    let (company_id, user_id) = (seeded.company_id, seeded.admin_user_id);
    let annulation = tokio::spawn(async move {
        invoice_settlements_write::cancel_settlement(&p, user_id, company_id, inv_id, sid).await
    });
    // ⛔ **Synchronisation déterministe, pas un délai** (passe 2 de revue de
    // code) : on ne valide la clôture qu'une fois l'annulation VUE en attente
    // d'un verrou `FOR UPDATE` sur `fiscal_years` — dans la liste des processus,
    // filtrée sur la base éphémère de CE test (les tests voisins tournent en
    // parallèle sous le même utilisateur). Avec le verrou de l'étape 2-bis,
    // l'annulation y attend AVANT de juger ; sans lui, elle juge d'abord, puis
    // attend au verrou du socle — dans les deux cas elle attend, et c'est ce qui
    // rend l'issue indépendante du minutage.
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["fiscal_years", "FOR UPDATE"],
        || annulation.is_finished(),
    )
    .await;
    if !vue {
        panic!(
            "l'annulation a fini sans attendre de verrou : {:?}",
            annulation.await
        );
    }

    // (3) La clôture est validée.
    closing.commit().await.unwrap();

    let result = annulation.await.expect("tâche d'annulation");
    assert!(
        matches!(
            result,
            Err(DbError::SettlementNotCancellable {
                blocker: SettlementCancelBlocker::FiscalYearClosed
            })
        ),
        "l'annulation devait attendre la clôture puis refuser l'exercice clos — reçu {result:?}"
    );
    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM invoice_settlements WHERE id = {sid}")
        )
        .await,
        1,
        "rien n'a été annulé"
    );
}
