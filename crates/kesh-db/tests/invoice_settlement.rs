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
