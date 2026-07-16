//! Tests d'intégration du repo `invoice_reminders` (Story 21-5a, #231).
//!
//! Couvre : insert append-only, `current_level` = MAX(level_number) des non-annulés,
//! `cancel_in_tx` soft (exclut du MAX), `list_for_invoice` (annulés inclus),
//! scoping company (cross-tenant → vide / not-found).

use chrono::NaiveDateTime;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, NewInvoiceReminder, OrgType, ReminderChannel};
use kesh_db::repositories::{companies, invoice_reminders};
use rust_decimal::Decimal;
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.to_string(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("create test company")
    .id
}

/// Seed un contact + une facture validée, retourne l'`invoice_id`.
async fn create_test_invoice(pool: &MySqlPool, company_id: i64) -> i64 {
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name) VALUES (?, 'Personne', 'Débiteur Test') RETURNING id",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .expect("insert contact");

    // `draft` : le repo `invoice_reminders` ne dépend pas du statut de la facture,
    // et `validated` imposerait un `journal_entry_id` (CHECK chk_invoices_validated_has_je).
    sqlx::query_scalar(
        "INSERT INTO invoices (company_id, contact_id, status, date, due_date, total_amount) \
         VALUES (?, ?, 'draft', '2026-06-01', '2026-06-30', 1000.00) RETURNING id",
    )
    .bind(company_id)
    .bind(contact_id)
    .fetch_one(pool)
    .await
    .expect("insert invoice")
}

fn ts(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").expect("ts")
}

fn manual_reminder(
    company_id: i64,
    invoice_id: i64,
    level: i16,
    fee: &str,
    sent: &str,
) -> NewInvoiceReminder {
    NewInvoiceReminder {
        company_id,
        invoice_id,
        level_number: level,
        fee_amount: fee.parse::<Decimal>().unwrap(),
        sent_at: ts(sent),
        channel: ReminderChannel::Manual,
        sent_to: None,
        subject: format!("Rappel manuel — niveau {level}"),
        body: "corps".into(),
        note: Some("note".into()),
        actor_user_id: None,
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn insert_and_current_level_uses_max_non_cancelled(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Reminder Co").await;
    let invoice_id = create_test_invoice(&pool, company_id).await;

    // Aucun rappel → niveau courant 0.
    assert_eq!(
        invoice_reminders::current_level(&pool, company_id, invoice_id)
            .await
            .unwrap(),
        0
    );

    // Insère niveau 1 puis 2.
    let mut tx = pool.begin().await.unwrap();
    let r1 = invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 1, "0.00", "2026-07-10 09:00:00"),
    )
    .await
    .unwrap();
    invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 2, "20.00", "2026-07-20 09:00:00"),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(
        invoice_reminders::current_level(&pool, company_id, invoice_id)
            .await
            .unwrap(),
        2
    );
    assert_eq!(r1.channel, "manual");
    assert_eq!(r1.fee_amount, Decimal::new(0, 2));
    assert!(r1.cancelled_at.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cancel_soft_excludes_from_max_level(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Cancel Co").await;
    let invoice_id = create_test_invoice(&pool, company_id).await;

    let mut tx = pool.begin().await.unwrap();
    invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 1, "0.00", "2026-07-10 09:00:00"),
    )
    .await
    .unwrap();
    let r2 = invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 2, "20.00", "2026-07-20 09:00:00"),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        invoice_reminders::current_level(&pool, company_id, invoice_id)
            .await
            .unwrap(),
        2
    );

    // Annule le niveau 2 → le niveau courant retombe à 1.
    let mut tx = pool.begin().await.unwrap();
    let cancelled = invoice_reminders::cancel_in_tx(&mut tx, company_id, r2.id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(cancelled);
    assert_eq!(
        invoice_reminders::current_level(&pool, company_id, invoice_id)
            .await
            .unwrap(),
        1
    );

    // Ré-annulation idempotente → false (déjà annulé).
    let mut tx = pool.begin().await.unwrap();
    let again = invoice_reminders::cancel_in_tx(&mut tx, company_id, r2.id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(!again);

    // La ligne annulée reste dans l'historique (append-only).
    let history = invoice_reminders::list_for_invoice(&pool, company_id, invoice_id)
        .await
        .unwrap();
    assert_eq!(history.len(), 2);
    assert!(
        history
            .iter()
            .any(|r| r.id == r2.id && r.cancelled_at.is_some())
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn sum_fees_deduped_excludes_level_and_cancelled(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Fees Co").await;
    let invoice_id = create_test_invoice(&pool, company_id).await;

    let mut tx = pool.begin().await.unwrap();
    // Niveau 1 (0), niveau 2 (20), niveau 2 ré-émis (25 — D18), niveau 3 (40 puis annulé).
    invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 1, "0.00", "2026-07-01 09:00:00"),
    )
    .await
    .unwrap();
    invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 2, "20.00", "2026-07-10 09:00:00"),
    )
    .await
    .unwrap();
    invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 2, "25.00", "2026-07-12 09:00:00"),
    )
    .await
    .unwrap();
    let r3 = invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_id, invoice_id, 3, "40.00", "2026-07-20 09:00:00"),
    )
    .await
    .unwrap();
    invoice_reminders::cancel_in_tx(&mut tx, company_id, r3.id)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // En excluant le niveau 3 (celui en cours d'envoi) : dédup par niveau → niv1(0) + niv2(MAX(20,25)=25) = 25.
    // Le niveau 3 est annulé donc absent de toute façon.
    let sum = invoice_reminders::sum_fees_deduped_excluding(&pool, company_id, invoice_id, 3)
        .await
        .unwrap();
    assert_eq!(
        sum,
        Decimal::new(2500, 2),
        "dédup par niveau (MAX) + exclut annulés"
    );

    // En excluant le niveau 2 : seul niv1(0) reste = 0.
    let sum2 = invoice_reminders::sum_fees_deduped_excluding(&pool, company_id, invoice_id, 2)
        .await
        .unwrap();
    assert_eq!(sum2, Decimal::new(0, 2), "niveau 2 exclu");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn scoping_is_company_isolated(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Tenant A").await;
    let company_b = create_test_company(&pool, "Tenant B").await;
    let invoice_a = create_test_invoice(&pool, company_a).await;

    let mut tx = pool.begin().await.unwrap();
    let r = invoice_reminders::insert_in_tx(
        &mut tx,
        &manual_reminder(company_a, invoice_a, 1, "0.00", "2026-07-10 09:00:00"),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Company B ne voit rien de l'invoice de A.
    assert_eq!(
        invoice_reminders::current_level(&pool, company_b, invoice_a)
            .await
            .unwrap(),
        0
    );
    assert!(
        invoice_reminders::list_for_invoice(&pool, company_b, invoice_a)
            .await
            .unwrap()
            .is_empty()
    );

    // Annulation cross-tenant refusée (find scoped → pas d'update).
    let mut tx = pool.begin().await.unwrap();
    let cancelled = invoice_reminders::cancel_in_tx(&mut tx, company_b, r.id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(!cancelled);
    // Toujours actif côté A.
    assert_eq!(
        invoice_reminders::current_level(&pool, company_a, invoice_a)
            .await
            .unwrap(),
        1
    );
}
