//! Tests d'intégration de la balance âgée débiteurs (Story 21-7).
//!
//! Vérifie : ventilation par bucket ET aux frontières (30/31, 60/61, 90/91),
//! réconciliation (Σ buckets = total ligne ; Σ lignes = total général),
//! parité TTC au centime avec le helper Rust `invoice_total_ttc`, scoping
//! multi-tenant, invariant D10 (facture suspendue INCLUSE), exclusion des
//! factures payées / brouillons, et `due_date NULL` → « Non échu ».
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).

use chrono::{Duration, NaiveDate};
use kesh_core::accounting::vat::invoice_total_ttc;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::{NewCreditNote, NewInvoice, NewInvoiceLine, SettlementChoice};
use kesh_db::repositories::invoices::ValidatedInvoice;
use kesh_db::repositories::{contacts, credit_notes, invoice_settlements_write, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::aged_receivables::generate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Date d'arrêté fixe (indépendante de l'horloge) pour des buckets déterministes.
fn as_of() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, 20).unwrap()
}

fn days_before(n: i64) -> NaiveDate {
    as_of() - Duration::days(n)
}

async fn mk_contact(pool: &MySqlPool, seeded: &SeededCompany, name: &str) -> i64 {
    contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: name.into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Salutation::Neutre,
        },
    )
    .await
    .expect("create contact")
    .id
}

/// Crée une facture 1 ligne (quantité 1) et la valide. Retourne la facture validée.
async fn mk_validated(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    due_date: Option<NaiveDate>,
    vat_rate: Decimal,
    amount: Decimal,
) -> ValidatedInvoice {
    // date facture = échéance (ou as_of si pas d'échéance) — toujours dans la FY 2020-2030.
    let date = due_date.unwrap_or_else(as_of);
    let (inv, _lines) = invoices::create(
        pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date,
            due_date,
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: amount,
                vat_rate,
            }],
        },
    )
    .await
    .expect("create invoice");
    invoices::validate_invoice(pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("validate invoice")
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aged_buckets_reconciliation_and_parity(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let alpha = mk_contact(&pool, &seeded, "Alpha SA").await;
    let beta = mk_contact(&pool, &seeded, "Beta SA").await;

    // Alpha — postes ouverts couvrant chaque bucket ET chaque frontière.
    let z = Decimal::ZERO;
    // Non échu : échéance = as_of (days 0) + échéance NULL.
    mk_validated(&pool, &seeded, alpha, Some(as_of()), z, dec!(100)).await;
    mk_validated(&pool, &seeded, alpha, None, z, dec!(10)).await;
    // 1-30 : frontières days 1 et 30.
    mk_validated(&pool, &seeded, alpha, Some(days_before(1)), z, dec!(200)).await;
    mk_validated(&pool, &seeded, alpha, Some(days_before(30)), z, dec!(5)).await;
    // 31-60 : frontières days 31 et 60.
    mk_validated(&pool, &seeded, alpha, Some(days_before(31)), z, dec!(300)).await;
    mk_validated(&pool, &seeded, alpha, Some(days_before(60)), z, dec!(7)).await;
    // 61-90 : frontières days 61 et 90.
    mk_validated(&pool, &seeded, alpha, Some(days_before(61)), z, dec!(400)).await;
    mk_validated(&pool, &seeded, alpha, Some(days_before(90)), z, dec!(9)).await;
    // 90+ : days 91 (frontière).
    mk_validated(&pool, &seeded, alpha, Some(days_before(91)), z, dec!(500)).await;
    // 90+ SUSPENDUE (D10 — DOIT rester dans la balance âgée). VAT 8.1 → TTC 1081
    // (prouve que le montant est le TTC, pas le HT).
    let paused = mk_validated(
        &pool,
        &seeded,
        alpha,
        Some(days_before(100)),
        dec!(8.10),
        dec!(1000),
    )
    .await;
    invoices::set_dunning_pause(
        &pool,
        seeded.admin_user_id,
        paused.invoice.id,
        seeded.company_id,
        paused.invoice.version,
        true,
        Some("litige".into()),
    )
    .await
    .expect("pause");

    // Beta — uniquement une payée + un brouillon → Beta ne doit PAS apparaître.
    let paid = mk_validated(&pool, &seeded, beta, Some(days_before(50)), z, dec!(999)).await;
    // ⚠️ `UPDATE` brut, et non `settle_invoice` : ce test porte sur ce que la
    // BALANCE ÂGÉE exclut — il a besoin d'« une facture dont `paid_at` est
    // posé », pas d'un règlement comptable complet. Depuis la Story 24-3
    // (#372), `mark_as_paid` n'existe plus ; y substituer la machinerie de
    // règlement (plan comptable, écriture de vente, exercice ouvert) coupleraient
    // ce test à des changements sans rapport avec ce qu'il mesure.
    sqlx::query("UPDATE invoices SET paid_at = ? WHERE id = ?")
        .bind(as_of().and_hms_opt(12, 0, 0).unwrap())
        .bind(paid.invoice.id)
        .execute(&pool)
        .await
        .expect("montage : pose de paid_at");
    // Brouillon (créé, non validé) → exclu.
    invoices::create(
        &pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id: beta,
            date: as_of(),
            due_date: Some(days_before(40)),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Brouillon".into(),
                quantity: dec!(1),
                unit_price: dec!(888),
                vat_rate: z,
            }],
        },
    )
    .await
    .expect("create draft");

    let report = generate(&pool, seeded.company_id, as_of()).await.unwrap();

    // Beta exclue (payée + brouillon) → seule Alpha apparaît.
    assert_eq!(report.rows.len(), 1, "seule Alpha a des postes ouverts");
    let row = &report.rows[0];
    assert_eq!(row.contact_name, "Alpha SA");

    // Ventilation par bucket (frontières incluses).
    assert_eq!(
        row.buckets.not_due,
        dec!(110),
        "Non échu = 100 (as_of) + 10 (NULL)"
    );
    assert_eq!(
        row.buckets.days_1_to_30,
        dec!(205),
        "1-30 = 200 (j1) + 5 (j30)"
    );
    assert_eq!(
        row.buckets.days_31_to_60,
        dec!(307),
        "31-60 = 300 (j31) + 7 (j60)"
    );
    assert_eq!(
        row.buckets.days_61_to_90,
        dec!(409),
        "61-90 = 400 (j61) + 9 (j90)"
    );
    assert_eq!(
        row.buckets.days_over_90,
        dec!(1581),
        "90+ = 500 (j91) + 1081 (suspendue TTC, D10)"
    );

    // Réconciliation par ligne : Σ buckets = total.
    let sum = row.buckets.not_due
        + row.buckets.days_1_to_30
        + row.buckets.days_31_to_60
        + row.buckets.days_61_to_90
        + row.buckets.days_over_90;
    assert_eq!(sum, row.buckets.total, "réconciliation ligne");
    assert_eq!(row.buckets.total, dec!(2612));

    // Réconciliation générale : totals = Σ lignes (une seule ici).
    assert_eq!(report.totals.total, dec!(2612));
    assert_eq!(report.totals.not_due, dec!(110));
    assert_eq!(report.totals.days_over_90, dec!(1581));

    // Parité TTC : le total = Σ invoice_total_ttc des postes ouverts d'Alpha.
    // (La ligne suspendue TTC 1081 prouve que c'est le TTC, pas le HT 1000.)
    let expected_ttc: Decimal = [
        (dec!(100), z),
        (dec!(10), z),
        (dec!(200), z),
        (dec!(5), z),
        (dec!(300), z),
        (dec!(7), z),
        (dec!(400), z),
        (dec!(9), z),
        (dec!(500), z),
        (dec!(1000), dec!(8.10)),
    ]
    .into_iter()
    .map(|(amt, rate)| invoice_total_ttc(std::iter::once((amt, rate))))
    .sum();
    assert_eq!(report.totals.total, expected_ttc, "parité TTC helper Rust");
}

/// Scoping multi-tenant au niveau unitaire : `generate` filtre par `company_id`
/// (le poste ouvert d'une company n'apparaît que dans SON rapport). La vraie
/// isolation à deux companies réelles (via l'API + JWT) est couverte par le test
/// E2E backend `aged_receivables_e2e.rs` (AC19) — ici une facture validée exige
/// une écriture (`chk_invoices_validated_has_je`), on prouve donc le filtre
/// `WHERE company_id = ?` en interrogeant un autre `company_id`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aged_scoping_filters_by_company(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let alpha = mk_contact(&pool, &seeded, "Alpha SA").await;
    mk_validated(
        &pool,
        &seeded,
        alpha,
        Some(days_before(10)),
        Decimal::ZERO,
        dec!(300),
    )
    .await;

    // Rapport de la company seedée : uniquement Alpha.
    let report_a = generate(&pool, seeded.company_id, as_of()).await.unwrap();
    assert_eq!(report_a.rows.len(), 1);
    assert_eq!(report_a.rows[0].contact_name, "Alpha SA");
    assert_eq!(report_a.totals.total, dec!(300));

    // Un AUTRE company_id ne voit rien de la company seedée (WHERE company_id).
    let other = generate(&pool, seeded.company_id + 99_999, as_of())
        .await
        .unwrap();
    assert!(
        other.rows.is_empty(),
        "un autre company_id ne voit pas les postes de A"
    );
    assert_eq!(other.totals.total, Decimal::ZERO);
}

/// Une échéance dans le FUTUR (due_date > as_of, `DATEDIFF` négatif) tombe dans
/// « Non échu » — au même titre que `due_date = as_of` ou `due_date NULL`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aged_future_due_date_is_not_due(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let c = mk_contact(&pool, &seeded, "Futur SA").await;
    // Échéance 10 jours APRÈS la date d'arrêté.
    let future = as_of() + Duration::days(10);
    mk_validated(&pool, &seeded, c, Some(future), Decimal::ZERO, dec!(250)).await;

    let report = generate(&pool, seeded.company_id, as_of()).await.unwrap();
    assert_eq!(report.rows.len(), 1);
    let row = &report.rows[0];
    assert_eq!(row.buckets.not_due, dec!(250), "échéance future = Non échu");
    assert_eq!(row.buckets.days_1_to_30, Decimal::ZERO);
    assert_eq!(row.buckets.days_over_90, Decimal::ZERO);
    assert_eq!(row.buckets.total, dec!(250));
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aged_empty_when_no_open_items(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let report = generate(&pool, seeded.company_id, as_of()).await.unwrap();
    assert!(report.rows.is_empty());
    assert_eq!(report.totals.total, Decimal::ZERO);
    assert_eq!(report.totals.not_due, Decimal::ZERO);
    assert_eq!(report.as_of, as_of());
}

// ─── Story 25-4-b1 (#416) — la balance âgée somme le RESTE DÛ ─────────────────

async fn settle(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    invoice_id: i64,
    amount: Decimal,
    on: NaiveDate,
) {
    invoice_settlements_write::settle_invoice(
        pool,
        seeded.admin_user_id,
        seeded.company_id,
        invoice_id,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        amount,
        on,
    )
    .await
    .expect("règlement");
}

/// Solde du compte débiteurs au grand livre, pour la société.
async fn receivable_balance(pool: &MySqlPool, seeded: &SeededCompany) -> Decimal {
    sqlx::query_scalar(
        "SELECT COALESCE(SUM(l.debit) - SUM(l.credit), 0) FROM journal_entry_lines l \
         INNER JOIN journal_entries e ON e.id = l.entry_id \
         WHERE e.company_id = ? AND l.account_id = ?",
    )
    .bind(seeded.company_id)
    .bind(seeded.accounts["1100"])
    .fetch_one(pool)
    .await
    .expect("solde 1100")
}

/// AC 3 — une facture de 1 081.— (1 000.— à 8,1 %) réglée de 900.— pèse **181.—**
/// dans sa tranche, pas 1 081.—.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aged_uses_amount_due_not_ttc(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let alpha = mk_contact(&pool, &seeded, "Alpha SA").await;
    let v = mk_validated(
        &pool,
        &seeded,
        alpha,
        Some(days_before(45)),
        dec!(8.10),
        dec!(1000.00),
    )
    .await;
    settle(&pool, &seeded, v.invoice.id, dec!(900.00), days_before(40)).await;

    let r = generate(&pool, seeded.company_id, as_of()).await.unwrap();
    assert_eq!(r.rows.len(), 1);
    assert_eq!(
        r.rows[0].buckets.days_31_to_60,
        dec!(181.00),
        "reste dû, pas le TTC"
    );
    assert_eq!(r.rows[0].buckets.total, dec!(181.00));
    assert_eq!(r.totals.total, dec!(181.00));
}

/// AC 3 — ⛔ un règlement partiel ne rajeunit pas la créance : la tranche suit
/// l'échéance de la facture, pas la date du règlement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aged_partial_payment_keeps_original_bucket(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let alpha = mk_contact(&pool, &seeded, "Alpha SA").await;
    let v = mk_validated(
        &pool,
        &seeded,
        alpha,
        Some(days_before(100)),
        dec!(8.10),
        dec!(100.00),
    )
    .await;
    // Réglé en partie HIER : la créance restante reste à plus de 90 jours.
    settle(&pool, &seeded, v.invoice.id, dec!(8.10), days_before(1)).await;

    let r = generate(&pool, seeded.company_id, as_of()).await.unwrap();
    let b = &r.rows[0].buckets;
    assert_eq!(b.days_over_90, dec!(100.00));
    assert_eq!(
        b.not_due + b.days_1_to_30 + b.days_31_to_60 + b.days_61_to_90,
        Decimal::ZERO
    );
}

/// AC 5 — ⛔ le total de la balance âgée égale le solde du compte débiteurs au
/// grand livre, **dans la règle** : compte mû seulement par les ventes,
/// règlements clients et avoirs de factures validées, datés au plus tard à
/// `as_of`. Quatre états vivants : sans règlement, réglée en partie, soldée,
/// créditée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aged_total_matches_receivable_ledger(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let alpha = mk_contact(&pool, &seeded, "Alpha SA").await;
    let beta = mk_contact(&pool, &seeded, "Beta SA").await;

    let _open = mk_validated(
        &pool,
        &seeded,
        alpha,
        Some(days_before(20)),
        dec!(8.10),
        dec!(500.00),
    )
    .await;
    let partial = mk_validated(
        &pool,
        &seeded,
        alpha,
        Some(days_before(70)),
        dec!(2.60),
        dec!(333.33),
    )
    .await;
    settle(
        &pool,
        &seeded,
        partial.invoice.id,
        dec!(100.00),
        days_before(10),
    )
    .await;
    let paid = mk_validated(
        &pool,
        &seeded,
        beta,
        Some(days_before(30)),
        dec!(8.10),
        dec!(200.00),
    )
    .await;
    settle(
        &pool,
        &seeded,
        paid.invoice.id,
        dec!(216.20),
        days_before(5),
    )
    .await;
    let credited = mk_validated(
        &pool,
        &seeded,
        beta,
        Some(days_before(15)),
        dec!(8.10),
        dec!(50.00),
    )
    .await;
    credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: seeded.company_id,
            invoice_id: credited.invoice.id,
            date: days_before(2),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("avoir");

    let r = generate(&pool, seeded.company_id, as_of()).await.unwrap();
    let ledger = receivable_balance(&pool, &seeded).await;
    // 540.50 + (341.996… arrondi par ligne : 333.33 + 8.67 = 342.00) − 100 = 782.50
    assert_eq!(
        ledger,
        dec!(782.50),
        "anti-vacuité : le grand livre porte les quatre états"
    );
    assert_eq!(r.totals.total, ledger, "balance âgée = compte débiteurs");
}
