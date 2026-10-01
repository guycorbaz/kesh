//! Solder le reste d'une facture — Story 25-4-d2a (#384, #490).
//!
//! ⚠️ **Ce que ces tests protègent.** Le solde éteint le reste dû d'une facture
//! validée en l'imputant au compte de sa NATURE ; l'escompte et la perte
//! corrigent la TVA au prorata des taux. Et un invariant tient dans le temps :
//! **tant qu'un solde existe, la facture est payée** — aucun autre règlement ne
//! s'annule avant lui.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    NewCreditNote, NewInvoice, NewInvoiceLine, NewJournalEntry, NewJournalEntryLine,
    SettlementChoice, SettlementWriteOffNature as Nature,
};
use kesh_db::errors::{DbError, SettlementCancelBlocker};
use kesh_db::repositories::{
    credit_notes, invoice_settlements, invoice_settlements_write, invoices, reconciliation,
};
use kesh_db::test_fixtures::{SeededCompany, designate_rounding_account, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

const D: fn() -> NaiveDate = || ymd(2026, 3, 10);

/// Une société de test dont les trois natures pointent la charge `4000`, et le
/// reste d'arrondi un compte `6940`. La TVA due est le `2000` du montage.
async fn company(pool: &MySqlPool) -> SeededCompany {
    let seeded = seed_accounting_company(pool).await.expect("société");
    sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ?, \
         default_bank_fees_account_id = ?, default_bad_debt_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["4000"])
    .bind(seeded.accounts["4000"])
    .bind(seeded.accounts["4000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .expect("comptes de nature");
    designate_rounding_account(pool, seeded.company_id)
        .await
        .expect("compte d'arrondi");
    seeded
}

/// Une facture validée portant `lines` (HT, taux), avec son écriture de vente
/// `D créance TTC / C produit HT / C TVA due par taux` — posée à la main : ce
/// fichier teste le SOLDE, pas la validation.
async fn validated_invoice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    lines: &[(Decimal, Decimal)],
) -> i64 {
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client solde', TRUE) RETURNING id",
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
            date: D(),
            due_date: Some(D()),
            payment_terms: None,
            project_id: None,
            lines: lines
                .iter()
                .map(|(ht, rate)| NewInvoiceLine {
                    description: "Prestation".into(),
                    quantity: dec!(1),
                    unit_price: *ht,
                    vat_rate: *rate,
                    revenue_account_id: Some(seeded.accounts["3000"]),
                })
                .collect(),
        },
    )
    .await
    .expect("facture")
    .0;

    let ttc = kesh_core::accounting::vat::invoice_total_ttc(lines.iter().copied());
    let total_ht: Decimal = lines.iter().map(|(ht, _)| *ht).sum();
    let mut entry_lines = vec![
        NewJournalEntryLine {
            account_id: seeded.accounts["1100"],
            debit: ttc,
            credit: Decimal::ZERO,
            project_id: None,
        },
        NewJournalEntryLine {
            account_id: seeded.accounts["3000"],
            debit: Decimal::ZERO,
            credit: total_ht,
            project_id: None,
        },
    ];
    for rate in kesh_core::accounting::vat::vat_breakdown_by_rate(lines.iter().copied()) {
        entry_lines.push(NewJournalEntryLine {
            account_id: seeded.accounts["2000"],
            debit: Decimal::ZERO,
            credit: rate.vat_amount,
            project_id: None,
        });
    }
    let je = kesh_db::repositories::journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: D(),
            journal: Journal::Ventes,
            description: "Vente".into(),
            project_id: None,
            lines: entry_lines,
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

async fn version(pool: &MySqlPool, inv: i64) -> i32 {
    sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(inv)
        .fetch_one(pool)
        .await
        .expect("version")
}

async fn write_off(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    inv: i64,
    nature: Nature,
) -> Result<invoice_settlements_write::WriteOffOutcome, DbError> {
    let v = version(pool, inv).await;
    invoice_settlements_write::write_off_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        nature,
        D(),
        v,
    )
    .await
}

async fn settle_cash(pool: &MySqlPool, seeded: &SeededCompany, inv: i64, amount: Decimal) -> i64 {
    let out = invoice_settlements_write::settle_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        amount,
        D(),
    )
    .await
    .expect("règlement");
    sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
        .bind(out.journal_entry_id)
        .fetch_one(pool)
        .await
        .expect("ligne de règlement")
}

/// Les lignes d'une écriture : `(compte, débit, crédit)`, dans l'ordre.
async fn entry_lines(pool: &MySqlPool, entry_id: i64) -> Vec<(i64, Decimal, Decimal)> {
    sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines WHERE entry_id = ? ORDER BY id",
    )
    .bind(entry_id)
    .fetch_all(pool)
    .await
    .expect("lignes")
}

async fn balance(pool: &MySqlPool, account_id: i64) -> Decimal {
    sqlx::query_scalar(
        "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM journal_entry_lines WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .expect("solde")
}

/// ⛔ **L'escompte d'une facture non réglée, à deux taux** : l'écriture débite la
/// nature `A − Σ TVA` et la TVA due par taux, crédite la créance `A` ; la facture
/// est payée, sa version a bougé, la ligne porte nature et ventilation, l'audit
/// est écrit.
#[sqlx::test(migrations = "./test-schema")]
async fn discount_on_an_unpaid_invoice_corrects_vat_per_rate(pool: MySqlPool) {
    let seeded = company(&pool).await;
    // 100 × 1.081 + 100 × 1.026 = 210.70.
    let inv = validated_invoice(
        &pool,
        &seeded,
        &[(dec!(100.00), dec!(8.10)), (dec!(100.00), dec!(2.60))],
    )
    .await;
    let v_before = version(&pool, inv).await;

    let out = write_off(&pool, &seeded, inv, Nature::Discount)
        .await
        .expect("solde");
    assert_eq!(out.amount, dec!(210.70));

    let lines = entry_lines(&pool, out.journal_entry_id).await;
    assert_eq!(
        lines,
        vec![
            (seeded.accounts["4000"], dec!(200.00), dec!(0)),
            (seeded.accounts["2000"], dec!(8.10), dec!(0)),
            (seeded.accounts["2000"], dec!(2.60), dec!(0)),
            (seeded.accounts["1100"], dec!(0), dec!(210.70)),
        ]
    );
    let (journal, description): (String, String) =
        sqlx::query_as("SELECT journal, description FROM journal_entries WHERE id = ?")
            .bind(out.journal_entry_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(journal, "OD");
    assert!(description.contains("escompte accordé"), "{description}");

    assert_eq!(
        invoice_settlements::amount_due(&pool, inv).await.unwrap(),
        dec!(0)
    );
    let (paid_at, v_after): (Option<chrono::NaiveDateTime>, i32) =
        sqlx::query_as("SELECT paid_at, version FROM invoices WHERE id = ?")
            .bind(inv)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_some(), "un solde rend la facture payée");
    assert_eq!(v_after, v_before + 1);

    let (kind, nature, account, vat): (
        String,
        Option<String>,
        Option<i64>,
        Option<serde_json::Value>,
    ) = sqlx::query_as(
        "SELECT settlement_type, write_off_nature, settlement_account_id, write_off_vat \
             FROM invoice_settlements WHERE invoice_id = ?",
    )
    .bind(inv)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(kind, "write_off");
    assert_eq!(nature.as_deref(), Some("discount"));
    assert_eq!(account, Some(seeded.accounts["4000"]));
    let vat = vat.expect("ventilation figée");
    assert_eq!(vat.as_array().unwrap().len(), 2);
    assert_eq!(vat[0]["vatAmount"], "8.10");

    let audited: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'invoice.written_off' AND entity_id = ?",
    )
    .bind(inv)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audited, 1);
}

/// La perte sur débiteur d'une facture **réglée en partie** : le prorata porte
/// sur le reste.
#[sqlx::test(migrations = "./test-schema")]
async fn bad_debt_on_a_partly_paid_invoice_prorates_the_remainder(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let inv = validated_invoice(&pool, &seeded, &[(dec!(1000.00), dec!(8.10))]).await; // 1081.00
    settle_cash(&pool, &seeded, inv, dec!(864.80)).await; // reste 216.20 = 20 %
    let out = write_off(&pool, &seeded, inv, Nature::BadDebt)
        .await
        .expect("solde");
    assert_eq!(out.amount, dec!(216.20));
    assert_eq!(
        entry_lines(&pool, out.journal_entry_id).await,
        vec![
            (seeded.accounts["4000"], dec!(200.00), dec!(0)),
            (seeded.accounts["2000"], dec!(16.20), dec!(0)),
            (seeded.accounts["1100"], dec!(0), dec!(216.20)),
        ]
    );
    assert_eq!(
        invoice_settlements::amount_due(&pool, inv).await.unwrap(),
        dec!(0)
    );
}

/// Frais bancaires et reste d'arrondi ne corrigent pas la TVA : une seule ligne
/// au débit, la ventilation est `[]`.
#[sqlx::test(migrations = "./test-schema")]
async fn bank_fees_carry_no_vat(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let inv = validated_invoice(&pool, &seeded, &[(dec!(1000.00), dec!(8.10))]).await;
    settle_cash(&pool, &seeded, inv, dec!(1066.00)).await; // reste 15.00 de frais
    let out = write_off(&pool, &seeded, inv, Nature::BankFees)
        .await
        .expect("solde");
    assert_eq!(
        entry_lines(&pool, out.journal_entry_id).await,
        vec![
            (seeded.accounts["4000"], dec!(15.00), dec!(0)),
            (seeded.accounts["1100"], dec!(0), dec!(15.00)),
        ]
    );
    let vat: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT write_off_vat FROM invoice_settlements WHERE journal_entry_id = ?",
    )
    .bind(out.journal_entry_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(vat, Some(serde_json::json!([])));
}

/// ⛔ **#490** : un reste brut inférieur au demi-centime, insoldable par un
/// paiement, se solde en nature `rounding` sur le compte de différences
/// d'arrondi — la créance tombe à zéro.
#[sqlx::test(migrations = "./test-schema")]
async fn issue_490_a_sub_half_centime_remainder_is_written_off_as_rounding(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let inv = validated_invoice(&pool, &seeded, &[(dec!(10.00), dec!(0))]).await;
    // Un règlement de 9.9960 — montage en SQL, l'écriture par le vrai chemin :
    // il laisse 0.0040, que le règlement manuel refuse en trop-perçu.
    let je = kesh_db::repositories::journal_entries::create(
        &pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: D(),
            journal: Journal::OD,
            description: "Règlement de montage".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts["1000"],
                    debit: dec!(9.9960),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["1100"],
                    debit: Decimal::ZERO,
                    credit: dec!(9.9960),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture de montage");
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
         settled_on, settlement_type, settlement_account_id) \
         VALUES (?, ?, ?, 9.9960, ?, 'internal_account', ?)",
    )
    .bind(seeded.company_id)
    .bind(inv)
    .bind(je.entry.id)
    .bind(D())
    .bind(seeded.accounts["1000"])
    .execute(&pool)
    .await
    .expect("règlement de montage");
    assert_eq!(
        invoice_settlements::amount_due(&pool, inv).await.unwrap(),
        dec!(0.0040)
    );

    let out = write_off(&pool, &seeded, inv, Nature::Rounding)
        .await
        .expect("solde #490");
    assert_eq!(out.amount, dec!(0.0040));
    assert_eq!(
        invoice_settlements::amount_due(&pool, inv).await.unwrap(),
        dec!(0)
    );
    assert_eq!(
        balance(&pool, seeded.accounts["1100"]).await,
        dec!(0),
        "créance à zéro"
    );
}

/// Un reste d'arrondi ne dépasse pas 5 centimes.
#[sqlx::test(migrations = "./test-schema")]
async fn rounding_nature_is_refused_from_five_centimes(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let inv = validated_invoice(&pool, &seeded, &[(dec!(10.00), dec!(0))]).await;
    settle_cash(&pool, &seeded, inv, dec!(9.95)).await; // reste 0.05
    let err = write_off(&pool, &seeded, inv, Nature::Rounding)
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::InvalidInput(ref c) if c == "writeOffRoundingTooLarge"),
        "{err:?}"
    );
    // 0.04 passe.
    let inv = validated_invoice(&pool, &seeded, &[(dec!(10.00), dec!(0))]).await;
    settle_cash(&pool, &seeded, inv, dec!(9.96)).await;
    write_off(&pool, &seeded, inv, Nature::Rounding)
        .await
        .expect("0.04 est un reste d'arrondi");
}

/// Les refus : brouillon, facture annulée par avoir, facture payée sans ligne de
/// règlement, version périmée, compte non configuré ou archivé, date avant la
/// facture, date hors exercice, période verrouillée, compte TVA absent.
#[sqlx::test(migrations = "./test-schema")]
async fn write_off_refusals(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let lines = [(dec!(100.00), dec!(8.10))];

    // Brouillon.
    let inv = validated_invoice(&pool, &seeded, &lines).await;
    sqlx::query("UPDATE invoices SET status = 'draft', journal_entry_id = NULL WHERE id = ?")
        .bind(inv)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        write_off(&pool, &seeded, inv, Nature::Discount)
            .await
            .unwrap_err(),
        DbError::IllegalStateTransition(_)
    ));

    // Facture annulée par avoir.
    let inv = validated_invoice(&pool, &seeded, &lines).await;
    credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id: inv,
            date: D(),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("avoir");
    assert!(matches!(
        write_off(&pool, &seeded, inv, Nature::Discount)
            .await
            .unwrap_err(),
        DbError::IllegalStateTransition(_)
    ));

    // ⛔ Facture payée par l'ancien chemin : `paid_at` posé, aucune ligne.
    let inv = validated_invoice(&pool, &seeded, &lines).await;
    sqlx::query("UPDATE invoices SET paid_at = NOW() WHERE id = ?")
        .bind(inv)
        .execute(&pool)
        .await
        .unwrap();
    let err = write_off(&pool, &seeded, inv, Nature::BadDebt)
        .await
        .unwrap_err();
    assert!(
        matches!(err, DbError::InvalidInput(ref c) if c == "invoiceAlreadyPaid"),
        "{err:?}"
    );

    // Version périmée.
    let inv = validated_invoice(&pool, &seeded, &lines).await;
    let v = version(&pool, inv).await;
    let err = invoice_settlements_write::write_off_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        Nature::Discount,
        D(),
        v - 1,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");

    // Date avant la facture.
    let err = invoice_settlements_write::write_off_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        Nature::Discount,
        ymd(2026, 3, 1),
        v,
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, DbError::InvalidInput(ref c) if c == "settledOnBeforeInvoiceDate"),
        "{err:?}"
    );

    // Date hors de tout exercice ouvert.
    let err = invoice_settlements_write::write_off_invoice(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        Nature::Discount,
        ymd(2031, 6, 1),
        v,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::FiscalYearInvalid), "{err:?}");

    // Période verrouillée.
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(D())
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let err = write_off(&pool, &seeded, inv, Nature::Discount)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::PeriodLocked { .. }), "{err:?}");
    sqlx::query("UPDATE companies SET books_locked_through = NULL WHERE id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();

    // Compte TVA absent (la nature corrige la TVA).
    sqlx::query("UPDATE company_invoice_settings SET default_vat_payable_account_id = NULL WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let err = write_off(&pool, &seeded, inv, Nature::Discount)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::ConfigurationRequired(_)), "{err:?}");
    // … mais les frais bancaires, sans TVA, passent.
    write_off(&pool, &seeded, inv, Nature::BankFees)
        .await
        .expect("frais sans compte TVA");

    // Compte de nature non configuré, puis archivé.
    let inv = validated_invoice(&pool, &seeded, &lines).await;
    sqlx::query("UPDATE company_invoice_settings SET default_bad_debt_account_id = NULL WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let err = write_off(&pool, &seeded, inv, Nature::BadDebt)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            DbError::WriteOffAccountNotConfigured { nature: "bad_debt" }
        ),
        "{err:?}"
    );
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(seeded.accounts["4000"])
        .execute(&pool)
        .await
        .unwrap();
    let err = write_off(&pool, &seeded, inv, Nature::BankFees)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            DbError::WriteOffAccountNotConfigured {
                nature: "bank_fees"
            }
        ),
        "{err:?}"
    );

    // Aucun refus n'a rien écrit.
    let write_offs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM invoice_settlements WHERE settlement_type = 'write_off'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(write_offs, 1, "seul le solde de frais bancaires est passé");
}

/// ⛔ **L'invariant dans le temps** : tant qu'un solde existe, le règlement
/// antérieur ne s'annule pas (rang 1 bis). Une fois le solde annulé — TVA
/// contre-passée, facture rouverte —, il s'annule.
#[sqlx::test(migrations = "./test-schema")]
async fn a_settlement_cannot_be_cancelled_while_a_write_off_exists(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let inv = validated_invoice(&pool, &seeded, &[(dec!(1000.00), dec!(8.10))]).await;
    let settlement = settle_cash(&pool, &seeded, inv, dec!(864.80)).await;
    write_off(&pool, &seeded, inv, Nature::Discount)
        .await
        .expect("solde");
    let write_off_id: i64 = sqlx::query_scalar(
        "SELECT id FROM invoice_settlements WHERE invoice_id = ? AND settlement_type = 'write_off'",
    )
    .bind(inv)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Lecture : le règlement est bloqué, le solde non.
    let mut conn = pool.acquire().await.unwrap();
    let hit = invoice_settlements_write::settlement_cancel_blocker(
        &mut conn,
        seeded.company_id,
        settlement,
    )
    .await
    .unwrap();
    assert_eq!(
        hit.map(|h| h.0),
        Some(SettlementCancelBlocker::WriteOffExists)
    );
    assert!(
        invoice_settlements_write::settlement_cancel_blocker(
            &mut conn,
            seeded.company_id,
            write_off_id
        )
        .await
        .unwrap()
        .is_none()
    );
    drop(conn);

    // Écriture : refusée, sans rien écrire.
    let err = invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        settlement,
    )
    .await
    .unwrap_err();
    assert!(
        matches!(
            err,
            DbError::SettlementNotCancellable {
                blocker: SettlementCancelBlocker::WriteOffExists
            }
        ),
        "{err:?}"
    );

    // L'avoir et la dévalidation restent refusés tant que le solde existe.
    assert!(
        credit_notes::create_credit_note(
            &pool,
            NewCreditNote {
                company_id: seeded.company_id,
                invoice_id: inv,
                date: D()
            },
            seeded.admin_user_id,
        )
        .await
        .is_err()
    );
    let v = version(&pool, inv).await;
    assert!(
        invoices::unvalidate(&pool, seeded.company_id, inv, seeded.admin_user_id, v)
            .await
            .is_err()
    );

    // Le solde s'annule : TVA contre-passée, reste rétabli, facture rouverte.
    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        write_off_id,
    )
    .await
    .expect("annulation du solde");
    assert_eq!(
        invoice_settlements::amount_due(&pool, inv).await.unwrap(),
        dec!(216.20)
    );
    let paid_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(inv)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid_at.is_none());
    // La TVA due revient à celle de la vente (−81.00 au crédit), la correction est
    // contre-passée.
    assert_eq!(balance(&pool, seeded.accounts["2000"]).await, dec!(-81.00));

    // Puis le règlement s'annule.
    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        settlement,
    )
    .await
    .expect("annulation du règlement");
    assert_eq!(
        invoice_settlements::amount_due(&pool, inv).await.unwrap(),
        dec!(1081.00)
    );
}

/// Le rang : le motif « un solde existe » précède un exercice clos — sinon on
/// ferait rouvrir un exercice pour rien.
#[sqlx::test(migrations = "./test-schema")]
async fn the_write_off_motive_comes_before_a_closed_fiscal_year(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let inv = validated_invoice(&pool, &seeded, &[(dec!(100.00), dec!(0))]).await;
    let settlement = settle_cash(&pool, &seeded, inv, dec!(60.00)).await;
    write_off(&pool, &seeded, inv, Nature::BankFees)
        .await
        .expect("solde");
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let hit = invoice_settlements_write::settlement_cancel_blocker(
        &mut conn,
        seeded.company_id,
        settlement,
    )
    .await
    .unwrap();
    assert_eq!(
        hit.map(|h| h.0),
        Some(SettlementCancelBlocker::WriteOffExists)
    );
}

/// Le sens inverse : une facture soldée sort des candidats du rapprochement.
#[sqlx::test(migrations = "./test-schema")]
async fn a_written_off_invoice_is_no_reconciliation_candidate(pool: MySqlPool) {
    let seeded = company(&pool).await;
    let inv = validated_invoice(&pool, &seeded, &[(dec!(100.00), dec!(0))]).await;
    settle_cash(&pool, &seeded, inv, dec!(98.00)).await;
    let before = reconciliation::find_unpaid_invoices_for_window(
        &pool,
        seeded.company_id,
        D(),
        dec!(2.00),
        30,
        dec!(0),
    )
    .await
    .unwrap();
    assert!(
        before.iter().any(|c| c.invoice.id == inv),
        "candidate avant le solde"
    );
    write_off(&pool, &seeded, inv, Nature::Discount)
        .await
        .expect("solde");
    let after = reconciliation::find_unpaid_invoices_for_window(
        &pool,
        seeded.company_id,
        D(),
        dec!(2.00),
        30,
        dec!(0),
    )
    .await
    .unwrap();
    assert!(
        after.iter().all(|c| c.invoice.id != inv),
        "plus candidate après le solde"
    );
}

/// La garde `Σ TVA < A` de la fonction pure — inatteignable par le prorata,
/// donc exercée avec des parts injectées.
#[test]
fn write_off_lines_refuse_vat_reaching_the_amount() {
    use kesh_core::accounting::vat::VatRateShare;
    let share = |vat| VatRateShare {
        rate_percent: dec!(8.10),
        base_ht: dec!(0),
        vat_amount: vat,
    };
    let err = invoice_settlements::write_off_journal_lines(
        1,
        2,
        Some(3),
        dec!(1.00),
        &[share(dec!(1.00))],
    )
    .unwrap_err();
    assert!(matches!(err, DbError::Invariant(_)), "{err:?}");
    // Juste sous le montant : l'écriture se construit, équilibrée.
    let lines = invoice_settlements::write_off_journal_lines(
        1,
        2,
        Some(3),
        dec!(1.00),
        &[share(dec!(0.99))],
    )
    .unwrap();
    let debit: Decimal = lines.iter().map(|l| l.debit).sum();
    let credit: Decimal = lines.iter().map(|l| l.credit).sum();
    assert_eq!(debit, credit);
    assert_eq!(lines[0].debit, dec!(0.01));
    // Des parts sans compte de TVA : erreur de l'appelant.
    assert!(
        invoice_settlements::write_off_journal_lines(1, 2, None, dec!(10), &[share(dec!(1))])
            .is_err()
    );
}
