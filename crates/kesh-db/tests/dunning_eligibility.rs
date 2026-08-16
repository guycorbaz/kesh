//! Tests d'intégration de l'éligibilité aux rappels (Story 21-5a, #231).
//!
//! Couvre : non-échue absente, niveau 1 dû, niveau 2 dû après rappel, suspendue absente,
//! payée absente, terminal présent, dunning désactivé (0 niveau → vide, C1), seed lazy.

use chrono::{Duration, NaiveDateTime, Utc};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, NewInvoiceReminder, OrgType, ReminderChannel};
use kesh_db::repositories::{companies, dunning_eligibility, invoice_reminders};
use rust_decimal::Decimal;
use sqlx::MySqlPool;
use std::sync::atomic::{AtomicI64, Ordering};

static SEQ: AtomicI64 = AtomicI64::new(1);

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
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
    .expect("create company")
    .id
}

/// Contexte partagé : contact + année fiscale (réutilisée pour toutes les factures).
async fn setup_company(pool: &MySqlPool, name: &str) -> (i64, i64, i64) {
    let company_id = create_company(pool, name).await;
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, email) VALUES (?, 'Personne', 'Débiteur', 'debiteur@example.com') RETURNING id",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .expect("insert contact");
    let fy_id: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date) VALUES (?, '2026', '2026-01-01', '2026-12-31') RETURNING id",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .expect("insert fiscal_year");
    (company_id, contact_id, fy_id)
}

/// Crée une facture VALIDÉE (avec écriture liée, requise par chk_invoices_validated_has_je).
async fn validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    fy_id: i64,
    due_date: chrono::NaiveDate,
) -> i64 {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let je_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-06-01', 'Ventes', 'test') RETURNING id",
    )
    .bind(company_id)
    .bind(fy_id)
    .bind(n)
    .fetch_one(pool)
    .await
    .expect("insert journal_entry");
    sqlx::query_scalar(
        "INSERT INTO invoices (company_id, contact_id, status, date, due_date, total_amount, journal_entry_id) \
         VALUES (?, ?, 'validated', '2026-06-01', ?, 1000.00, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(due_date)
    .bind(je_id)
    .fetch_one(pool)
    .await
    .expect("insert validated invoice")
}

fn days_ago(n: i64) -> chrono::NaiveDate {
    Utc::now().date_naive() - Duration::days(n)
}

fn ts_days_ago(n: i64) -> NaiveDateTime {
    (Utc::now().naive_utc()) - Duration::days(n)
}

async fn insert_reminder(
    pool: &MySqlPool,
    company_id: i64,
    invoice_id: i64,
    level: i16,
    sent_days_ago: i64,
) {
    let mut tx = pool.begin().await.unwrap();
    invoice_reminders::insert_in_tx(
        &mut tx,
        &NewInvoiceReminder {
            company_id,
            invoice_id,
            level_number: level,
            fee_amount: Decimal::new(0, 2),
            sent_at: ts_days_ago(sent_days_ago),
            channel: ReminderChannel::Manual,
            sent_to: None,
            subject: "s".into(),
            body: "b".into(),
            note: None,
            actor_user_id: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

// Seed lazy défaut : grâce 5, niveaux 1/2/3 délais 10/10/10. Niveau 1 dû à échéance+15j.

#[sqlx::test(migrations = "./test-schema")]
async fn not_yet_overdue_is_absent(pool: MySqlPool) {
    let (company_id, contact_id, fy_id) = setup_company(&pool, "NotDue Co").await;
    // Échéance il y a 5 jours → grâce+délai(1)=15j non atteints.
    validated_invoice(&pool, company_id, contact_id, fy_id, days_ago(5)).await;
    let list = dunning_eligibility::list_reminder_candidates(&pool, company_id)
        .await
        .unwrap();
    assert!(
        list.is_empty(),
        "facture dans la grâce ne doit pas être candidate"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn level_one_due_present(pool: MySqlPool) {
    let (company_id, contact_id, fy_id) = setup_company(&pool, "Level1 Co").await;
    let inv = validated_invoice(&pool, company_id, contact_id, fy_id, days_ago(20)).await;
    let list = dunning_eligibility::list_reminder_candidates(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    let c = &list[0];
    assert_eq!(c.invoice_id, inv);
    assert_eq!(c.current_level, 0);
    assert_eq!(c.next_level, Some(1));
    assert!(!c.terminal);
    assert!(c.has_email);
}

#[sqlx::test(migrations = "./test-schema")]
async fn level_two_due_after_first_reminder(pool: MySqlPool) {
    let (company_id, contact_id, fy_id) = setup_company(&pool, "Level2 Co").await;
    let inv = validated_invoice(&pool, company_id, contact_id, fy_id, days_ago(40)).await;
    // Rappel niveau 1 envoyé il y a 12 jours → niveau 2 dû (délai 10).
    insert_reminder(&pool, company_id, inv, 1, 12).await;
    let list = dunning_eligibility::list_reminder_candidates(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].current_level, 1);
    assert_eq!(list[0].next_level, Some(2));
    assert!(!list[0].terminal);
}

#[sqlx::test(migrations = "./test-schema")]
async fn paused_and_paid_are_absent(pool: MySqlPool) {
    let (company_id, contact_id, fy_id) = setup_company(&pool, "PausePaid Co").await;
    let paused = validated_invoice(&pool, company_id, contact_id, fy_id, days_ago(30)).await;
    let paid = validated_invoice(&pool, company_id, contact_id, fy_id, days_ago(30)).await;
    sqlx::query("UPDATE invoices SET dunning_paused_at = UTC_TIMESTAMP(6) WHERE id = ?")
        .bind(paused)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE invoices SET paid_at = UTC_TIMESTAMP(6) WHERE id = ?")
        .bind(paid)
        .execute(&pool)
        .await
        .unwrap();
    let list = dunning_eligibility::list_reminder_candidates(&pool, company_id)
        .await
        .unwrap();
    assert!(
        list.is_empty(),
        "suspendue et payée absentes de la liste à rappeler"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn terminal_when_last_level_reached(pool: MySqlPool) {
    let (company_id, contact_id, fy_id) = setup_company(&pool, "Terminal Co").await;
    let inv = validated_invoice(&pool, company_id, contact_id, fy_id, days_ago(60)).await;
    // 3 rappels (tous les niveaux configurés) → terminal, présent quelle que soit la date.
    insert_reminder(&pool, company_id, inv, 1, 40).await;
    insert_reminder(&pool, company_id, inv, 2, 30).await;
    insert_reminder(&pool, company_id, inv, 3, 1).await;
    let list = dunning_eligibility::list_reminder_candidates(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].current_level, 3);
    assert_eq!(list[0].next_level, None);
    assert!(list[0].terminal);
    assert!(list[0].last_reminder_at.is_some());
}

#[sqlx::test(migrations = "./test-schema")]
async fn missing_due_date_is_absent(pool: MySqlPool) {
    let (company_id, contact_id, fy_id) = setup_company(&pool, "NoDue Co").await;
    // Facture validée SANS échéance (due_date NULL) → jamais candidate (AC 8/19).
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    let je_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-06-01', 'Ventes', 'test') RETURNING id",
    )
    .bind(company_id)
    .bind(fy_id)
    .bind(n)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, status, date, due_date, total_amount, journal_entry_id) \
         VALUES (?, ?, 'validated', '2026-01-01', NULL, 1000.00, ?)",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(je_id)
    .execute(&pool)
    .await
    .unwrap();
    let list = dunning_eligibility::list_reminder_candidates(&pool, company_id)
        .await
        .unwrap();
    assert!(list.is_empty(), "facture sans due_date absente de la liste");
}

#[sqlx::test(migrations = "./test-schema")]
async fn dunning_disabled_empty_levels_returns_empty(pool: MySqlPool) {
    let (company_id, contact_id, fy_id) = setup_company(&pool, "Disabled Co").await;
    validated_invoice(&pool, company_id, contact_id, fy_id, days_ago(60)).await;
    // 1er appel → seed lazy (3 niveaux), la facture est candidate.
    assert_eq!(
        dunning_eligibility::list_reminder_candidates(&pool, company_id)
            .await
            .unwrap()
            .len(),
        1
    );
    // La company vide volontairement ses niveaux (dunning désactivé, D7).
    sqlx::query("DELETE FROM dunning_levels WHERE company_id = ?")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap();
    // seeded_at reste posé → seed lazy no-op → liste vide (pas de résurrection, C1).
    let list = dunning_eligibility::list_reminder_candidates(&pool, company_id)
        .await
        .unwrap();
    assert!(
        list.is_empty(),
        "0 niveau = dunning désactivé → liste vide, ni candidate ni terminale"
    );
}
