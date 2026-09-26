//! Tests d'intégration des avoirs (Story 12.1) — contre-passation TVA-aware.
//!
//! Vérifie bout-en-bout (DB éphémère) que `create_credit_note` :
//! - génère l'écriture de contre-passation inversant la facture (solde 1100 → 0),
//! - bascule la facture d'origine en `cancelled` (DC6),
//! - refuse les cas interdits (facture draft / payée / déjà créditée).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).
//! Fixture `seed_accounting_company` : compte 2000 réutilisé en TVA due.

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewCreditNote, NewInvoice, NewInvoiceLine, SettlementChoice};
use kesh_db::errors::DbError;
use kesh_db::repositories::{contacts, credit_notes, invoice_settlements_write, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

async fn make_contact(pool: &MySqlPool, company_id: i64, admin_id: i64) -> i64 {
    contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Entreprise,
            name: "Client Avoir".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: Some("30".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .expect("create contact")
    .id
}

/// Crée une facture brouillon `(vat_rate, unit_price)` à quantité 1 et la valide.
async fn create_and_validate(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    lines: &[(Decimal, Decimal)],
) -> i64 {
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date: d(2026, 6, 15),
        due_date: None,
        payment_terms: None,
        lines: lines
            .iter()
            .map(|(rate, price)| NewInvoiceLine {
                revenue_account_id: None,
                description: "Ligne".into(),
                quantity: dec!(1),
                unit_price: *price,
                vat_rate: *rate,
            })
            .collect(),
        project_id: None,
    };
    let (inv, _) = invoices::create(pool, seeded.admin_user_id, new)
        .await
        .expect("create invoice");
    invoices::validate_invoice(pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("validate invoice");
    inv.id
}

fn sum_debit(je: &kesh_db::entities::JournalEntryWithLines) -> Decimal {
    je.lines.iter().map(|l| l.debit).sum()
}
fn sum_credit(je: &kesh_db::entities::JournalEntryWithLines) -> Decimal {
    je.lines.iter().map(|l| l.credit).sum()
}

/// (a) Avoir mono-taux 8.1 % → contre-passation 3 lignes (crédit 1100 TTC,
/// débit 3000 HT, débit 2000 TVA), équilibrée, facture → cancelled, solde 1100 → 0.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_single_rate_reverses_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let invoice_id =
        create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(1000.00))]).await;

    let issued = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("create credit note");

    let je = &issued.journal_entry;
    assert_eq!(je.lines.len(), 3, "crédit 1100 + débit 3000 + 1 débit TVA");

    let receivable = seeded.accounts["1100"];
    let revenue = seeded.accounts["3000"];
    let vat = seeded.accounts["2000"];

    let creance = je
        .lines
        .iter()
        .find(|l| l.account_id == receivable)
        .unwrap();
    assert_eq!(
        creance.credit,
        dec!(1081.00),
        "créance TTC contre-passée au crédit"
    );
    assert_eq!(creance.debit, dec!(0));
    let produit = je.lines.iter().find(|l| l.account_id == revenue).unwrap();
    assert_eq!(
        produit.debit,
        dec!(1000.00),
        "produit HT contre-passé au débit"
    );
    let tva = je.lines.iter().find(|l| l.account_id == vat).unwrap();
    assert_eq!(tva.debit, dec!(81.00), "TVA due contre-passée au débit");

    assert_eq!(sum_debit(je), sum_credit(je), "écriture équilibrée");

    // Statut avoir + numéro.
    assert_eq!(issued.credit_note.status, "issued");
    assert!(
        issued
            .credit_note
            .credit_note_number
            .unwrap()
            .starts_with("AV-")
    );
    assert_eq!(issued.credit_note.total_amount, dec!(1000.0000), "total HT");

    // Facture d'origine → cancelled.
    let inv_status: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(inv_status, "cancelled");

    // Solde du compte 1100 (créance) sur toutes les écritures de la company → 0.
    let balance_1100: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE je.company_id = ? AND jel.account_id = ?",
    )
    .bind(seeded.company_id)
    .bind(receivable)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(balance_1100, dec!(0), "facture + avoir = solde 1100 à zéro");
}

/// (b) Avoir multi-taux → une ligne TVA contre-passée par taux (ASC).
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_multi_rate(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let invoice_id = create_and_validate(
        &pool,
        &seeded,
        contact,
        &[(dec!(8.10), dec!(1000.00)), (dec!(2.60), dec!(500.00))],
    )
    .await;

    let issued = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("create credit note");

    let vat = seeded.accounts["2000"];
    let vat_lines: Vec<Decimal> = issued
        .journal_entry
        .lines
        .iter()
        .filter(|l| l.account_id == vat)
        .map(|l| l.debit)
        .collect();
    assert_eq!(
        vat_lines,
        vec![dec!(13.00), dec!(81.00)],
        "TVA par taux ASC (2.6% puis 8.1%)"
    );
    assert_eq!(
        sum_debit(&issued.journal_entry),
        sum_credit(&issued.journal_entry)
    );
}

/// (c) Refus : facture brouillon (non validée).
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_refused_on_draft_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let (inv, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id: contact,
            date: d(2026, 6, 15),
            due_date: None,
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "L".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(8.10),
            }],
            project_id: None,
        },
    )
    .await
    .unwrap();

    let err = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id: inv.id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

/// (d) Refus : facture déjà payée (AC2bis).
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_refused_on_paid_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let invoice_id =
        create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(1000.00))]).await;

    // Marque la facture payée (directement en SQL pour le test).
    sqlx::query("UPDATE invoices SET paid_at = CURRENT_TIMESTAMP(3) WHERE id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    // Story 25-4-a (#456) : la garde AC2bis rend sa variante dédiée, plus le
    // générique — le motif « payée » et le motif « réglée en partie » sont un
    // seul et même refus. `settlement_id` est `None` : seul `paid_at` est posé.
    assert!(
        matches!(
            err,
            DbError::CreditNoteBlockedBySettlement {
                settlement_id: None,
                ..
            }
        ),
        "refus facture payée — reçu {err:?}"
    );
}

/// (e) Refus : un seul avoir par facture (AC3 / DC7).
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_refused_when_already_credited(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let invoice_id =
        create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(1000.00))]).await;

    let first = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await;
    assert!(first.is_ok());

    // 2e tentative : la facture est désormais cancelled ET déjà créditée → refus.
    let err = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: d(2026, 7, 2),
        },
        seeded.admin_user_id,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::IllegalStateTransition(_)));
}

// ---------------------------------------------------------------------------
// Story 19-4 — l'avoir hérite le projet de la facture (net par projet = 0)
// ---------------------------------------------------------------------------

/// L'avoir reprend le tag analytique de la facture d'origine : après la
/// contre-passation, la somme (débit − crédit) des lignes taguées sur le
/// projet vaut 0. Fonctionne aussi si le projet a été ARCHIVÉ entre la
/// validation et l'avoir (DC3 : pas de re-check archivé sur l'annulation,
/// miroir pay/cancel-after-archive 19-3).
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_inherits_project_and_nets_to_zero(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;

    // Projet actif + facture taguée validée.
    let project: i64 = sqlx::query(
        "INSERT INTO projects (company_id, code, name, archived, version) VALUES (?, 'AVOIR-P', 'AVOIR-P', FALSE, 0)",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id: contact,
        date: d(2026, 6, 15),
        due_date: None,
        payment_terms: None,
        project_id: Some(project),
        lines: vec![NewInvoiceLine {
            revenue_account_id: None,
            description: "Ligne".into(),
            quantity: dec!(1),
            unit_price: dec!(1000.00),
            vat_rate: dec!(8.10),
        }],
    };
    let (inv, _) = invoices::create(&pool, seeded.admin_user_id, new)
        .await
        .expect("create");
    invoices::validate_invoice(&pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("validate");

    // Archiver le projet APRÈS la validation : l'avoir doit rester possible.
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(project)
        .execute(&pool)
        .await
        .unwrap();

    let issued = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id: inv.id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("l'avoir doit passer même si le projet est archivé (DC3)");

    // Toutes les lignes de la contre-passation portent le projet hérité.
    assert!(
        issued
            .journal_entry
            .lines
            .iter()
            .all(|l| l.project_id == Some(project)),
        "lignes avoir: {:?}",
        issued
            .journal_entry
            .lines
            .iter()
            .map(|l| l.project_id)
            .collect::<Vec<_>>()
    );

    // Net par projet = 0 sur l'ensemble du grand livre.
    let (debit, credit): (Decimal, Decimal) = sqlx::query_as(
        "SELECT COALESCE(SUM(debit), 0), COALESCE(SUM(credit), 0) \
         FROM journal_entry_lines WHERE project_id = ?",
    )
    .bind(project)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(debit, credit, "net par projet doit être 0 après l'avoir");
}

// ─── Story 25-4-a (#456) — pas d'avoir sur une facture réglée, même en partie ───

async fn settle(pool: &MySqlPool, seeded: &SeededCompany, invoice_id: i64, amount: Decimal) -> i64 {
    let out = invoice_settlements_write::settle_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        invoice_id,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        amount,
        d(2026, 6, 20),
    )
    .await
    .expect("règlement");
    sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
        .bind(out.journal_entry_id)
        .fetch_one(pool)
        .await
        .expect("ligne de règlement")
}

async fn try_credit(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    invoice_id: i64,
) -> Result<credit_notes::IssuedCreditNote, DbError> {
    credit_notes::create_credit_note(
        pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: d(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
}

/// Ce qu'un refus ne doit pas avoir touché : avoirs, statut et version de la
/// facture, solde de la créance, séquence de numéros d'avoir.
async fn empreinte(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    invoice_id: i64,
) -> (i64, String, i32, Decimal, i64) {
    let avoirs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM credit_notes WHERE company_id = ?")
        .bind(seeded.company_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let (status, version): (String, i32) =
        sqlx::query_as("SELECT status, version FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(pool)
            .await
            .unwrap();
    let creance: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(debit) - SUM(credit), 0) FROM journal_entry_lines WHERE account_id = ?",
    )
    .bind(seeded.accounts["1100"])
    .fetch_one(pool)
    .await
    .unwrap();
    let sequence: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(next_number), 0) AS SIGNED) FROM credit_note_number_sequences WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (avoirs, status, version, creance, sequence)
}

/// AC 9, AC 10 — une facture réglée en partie (`paid_at` NULL) est refusée, et
/// **rien n'est écrit**. C'est le chemin que l'ancienne garde laissait passer.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_refused_on_partially_settled_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let invoice_id =
        create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00))]).await;
    let sid = settle(&pool, &seeded, invoice_id, dec!(40.00)).await;

    let paid_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        paid_at.is_none(),
        "anti-vacuité : la facture n'est PAS soldée"
    );

    let avant = empreinte(&pool, &seeded, invoice_id).await;
    let err = try_credit(&pool, &seeded, invoice_id).await.unwrap_err();
    match &err {
        DbError::CreditNoteBlockedBySettlement {
            invoice_id: iid,
            settlement_id,
            invoice_number,
        } => {
            assert_eq!(*iid, invoice_id);
            assert_eq!(*settlement_id, Some(sid));
            assert!(invoice_number.is_some(), "le numéro est rendu");
        }
        other => panic!("attendu CreditNoteBlockedBySettlement, reçu {other:?}"),
    }
    assert_eq!(err.error_code(), "CREDIT_NOTE_INVOICE_SETTLED");
    assert_eq!(
        empreinte(&pool, &seeded, invoice_id).await,
        avant,
        "rien n'est écrit"
    );
}

/// La promesse du message : annuler le règlement rouvre l'avoir.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_accepted_after_the_settlement_is_cancelled(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let invoice_id =
        create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00))]).await;
    let sid = settle(&pool, &seeded, invoice_id, dec!(40.00)).await;
    assert!(try_credit(&pool, &seeded, invoice_id).await.is_err());

    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        invoice_id,
        sid,
    )
    .await
    .expect("annulation du règlement");

    try_credit(&pool, &seeded, invoice_id)
        .await
        .expect("l'avoir passe une fois le règlement annulé");
}

/// AC 9 — ⛔ **entrelacement** : un règlement commité PENDANT que l'avoir
/// attend le verrou de la facture doit être vu.
///
/// 1. une transaction verrouille la facture et y rattache un règlement, sans
///    valider ;
/// 2. l'avoir démarre et attend au `FOR UPDATE` de la facture ;
/// 3. la transaction est validée.
///
/// La garde lit les règlements **après** le verrou, par une lecture
/// verrouillante : elle voit la ligne et refuse. Lue **avant** le verrou, en
/// lecture simple, elle aurait vu « aucun règlement » et laissé passer.
///
/// Le règlement vient du vrai chemin, sur une facture auxiliaire : seul son
/// rattachement se fait dans la transaction concurrente.
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_waits_for_a_concurrent_settlement(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let invoice_id =
        create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00))]).await;
    let parking = create_and_validate(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00))]).await;
    let sid = settle(&pool, &seeded, parking, dec!(40.00)).await;

    // (1) Le règlement concurrent, non validé, qui tient la facture.
    let mut concurrent = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM invoices WHERE id = ? FOR UPDATE")
        .bind(invoice_id)
        .execute(&mut *concurrent)
        .await
        .unwrap();
    sqlx::query("UPDATE invoice_settlements SET invoice_id = ? WHERE id = ?")
        .bind(invoice_id)
        .bind(sid)
        .execute(&mut *concurrent)
        .await
        .unwrap();

    // (2) L'avoir démarre et attend.
    let p = pool.clone();
    let (company_id, user_id) = (seeded.company_id, seeded.admin_user_id);
    let avoir = tokio::spawn(async move {
        credit_notes::create_credit_note(
            &p,
            NewCreditNote {
                company_id,
                invoice_id,
                date: d(2026, 7, 1),
            },
            user_id,
        )
        .await
    });
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["FROM invoices", "FOR UPDATE"],
        || avoir.is_finished(),
    )
    .await;
    if !vue {
        panic!("l'avoir a fini sans attendre le verrou : {:?}", avoir.await);
    }

    // (3) Le règlement concurrent est validé.
    concurrent.commit().await.unwrap();

    let result = avoir.await.expect("tâche d'avoir");
    assert!(
        matches!(
            result,
            Err(DbError::CreditNoteBlockedBySettlement {
                settlement_id: Some(s),
                ..
            }) if s == sid
        ),
        "l'avoir devait attendre le règlement puis refuser — reçu {:?}",
        result.map(|c| c.credit_note.id)
    );
}
