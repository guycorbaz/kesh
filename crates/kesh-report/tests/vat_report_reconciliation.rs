//! Tests d'intégration — Réconciliation rapport TVA ↔ grand livre (Story 18-1e, DC5).
//!
//! Vérifie que `vat_report::generate` calcule `reconciliation_delta` =
//! `total_vat_due` (dérivé des `invoice_lines`) − solde du compte TVA due
//! (`default_vat_payable_account_id`) au grand livre **isolé au périmètre ventes**
//! (lignes liées à une facture validée via `invoices.journal_entry_id`), et
//! `reconciliation_status` ("ok"/"delta", seuil 0.01).
//!
//! La fixture `seed_accounting_company` configure
//! `default_vat_payable_account_id = compte 2000` (réutilisé, Story 18-1b) — c'est
//! le « compte 2200 » en prod. Les écritures de vente sont générées par
//! `validate_invoice` (lien `journal_entry_id`).
//!
//! Pré-requis : MariaDB (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::{contacts, credit_notes, invoices, journal_entries};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::period::ReportPeriod;
use kesh_report::vat_report;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Période 2026 (incluse dans l'exercice fixture 2020-2030).
fn period_2026(fiscal_year_id: i64) -> ReportPeriod {
    ReportPeriod {
        fiscal_year_id,
        start_date: ymd(2026, 1, 1),
        end_date: ymd(2026, 12, 31),
    }
}

async fn seed_contact(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Client CI".into(),
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
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id
}

/// Crée une facture (lignes `(vat_rate, unit_price)` à quantité 1) puis la valide
/// via le flow normal `validate_invoice` (pose `journal_entry_id`, génère la ligne
/// TVA due sur `default_vat_payable_account_id`). Réplique le helper de
/// `vat_report_e2e.rs:144`. Renvoie l'`invoice_id`.
async fn create_validated_invoice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    date: NaiveDate,
    lines: &[(Decimal, Decimal)],
) -> i64 {
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date,
        due_date: Some(date),
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
        .unwrap();
    invoices::validate_invoice(pool, seeded.company_id, inv.id, seeded.admin_user_id)
        .await
        .expect("validate_invoice")
        .invoice
        .id
}

/// Édite à la main la ligne TVA due (compte `account_id`) de l'écriture liée à la
/// facture `invoice_id` : remplace son `credit` par `new_credit`. Simule un
/// `PUT /journal-entries` qui désaligne l'écriture de la TVA facturée. SQL brut
/// (pas de double-check d'équilibre au niveau lecture).
async fn tamper_vat_line(pool: &MySqlPool, invoice_id: i64, account_id: i64, new_credit: Decimal) {
    let affected = sqlx::query(
        "UPDATE journal_entry_lines jel \
         INNER JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         SET jel.credit = ? \
         WHERE i.id = ? AND jel.account_id = ?",
    )
    .bind(new_credit)
    .bind(invoice_id)
    .bind(account_id)
    .execute(pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(affected, 1, "exactement 1 ligne TVA due à éditer");
}

async fn gen_report(pool: &MySqlPool, seeded: &SeededCompany) -> vat_report::VatReport {
    vat_report::generate(pool, seeded.company_id, &period_2026(seeded.fiscal_year_id))
        .await
        .unwrap()
}

/// (a) Cas nominal : une facture validée intacte → la TVA due dérivée == solde
/// 2200 ventes par construction → delta == 0, status "ok".
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_nominal_delta_zero(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;

    let report = gen_report(&pool, &seeded).await;
    assert_eq!(report.total_vat_due, dec!(81.00), "TVA due dérivée 81");
    assert_eq!(
        report.reconciliation_delta,
        dec!(0),
        "due == solde 2200 ventes"
    );
    assert_eq!(report.reconciliation_status, "ok");
}

/// (a-bis) **Story 16-1a (AC13)** — facture **multi-comptes de produit ×
/// multi-taux** : la ventilation du crédit produit par compte ne perturbe pas
/// le rapport TVA.
///
/// Le rapport ne lit que `default_vat_payable_account_id` et
/// `invoice_lines.(vat_rate, line_total)` — jamais un compte de produit. Ce
/// test l'ancre : trois comptes de produit distincts et deux taux, et la
/// réconciliation doit rester `ok` avec un delta nul. La ventilation multiplie
/// les lignes de crédit produit de l'écriture, pas les lignes de TVA due.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_unaffected_by_multi_account_ventilation(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;

    let mut extra = Vec::new();
    for (number, name) in [("3200", "Prestations"), ("3400", "Marchandises")] {
        let id =
            sqlx::query("INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, 'Revenue')")
                .bind(seeded.company_id)
                .bind(number)
                .bind(name)
                .execute(&pool)
                .await
                .unwrap()
                .last_insert_id() as i64;
        extra.push(id);
    }

    // 3 comptes (défaut 3000 + 3200 + 3400) × 2 taux (8.1 % et 2.6 %).
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id: contact,
        date: ymd(2026, 6, 15),
        due_date: None,
        payment_terms: None,
        lines: vec![
            NewInvoiceLine {
                description: "Honoraires".into(),
                quantity: dec!(1),
                unit_price: dec!(1000),
                vat_rate: dec!(8.10),
                revenue_account_id: None,
            },
            NewInvoiceLine {
                description: "Prestations".into(),
                quantity: dec!(1),
                unit_price: dec!(500),
                vat_rate: dec!(2.60),
                revenue_account_id: Some(extra[0]),
            },
            NewInvoiceLine {
                description: "Marchandises".into(),
                quantity: dec!(1),
                unit_price: dec!(400),
                vat_rate: dec!(8.10),
                revenue_account_id: Some(extra[1]),
            },
        ],
        project_id: None,
    };
    let (inv, _) = invoices::create(&pool, seeded.admin_user_id, new)
        .await
        .unwrap();
    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, inv.id, seeded.admin_user_id)
            .await
            .expect("validate");

    // L'écriture porte bien 3 crédits produit et 2 lignes de TVA due.
    let vat_account = seeded.accounts["2000"];
    let vat_lines = validated
        .journal_entry
        .lines
        .iter()
        .filter(|l| l.account_id == vat_account)
        .count();
    assert_eq!(
        vat_lines, 2,
        "une ligne de TVA due PAR TAUX, pas par compte"
    );
    assert_eq!(
        validated.journal_entry.lines.len(),
        1 + 3 + 2,
        "créance + 3 comptes produit + 2 taux"
    );

    let report = gen_report(&pool, &seeded).await;
    // 8.1 % sur 1400 = 81.00 + 32.40 ; 2.6 % sur 500 = 13.00.
    assert_eq!(report.total_vat_due, dec!(126.40));
    assert_eq!(
        report.reconciliation_delta,
        dec!(0),
        "la ventilation par compte de produit n'affecte pas la TVA due"
    );
    assert_eq!(report.reconciliation_status, "ok");
}

/// (b) Édition manuelle : on réduit le crédit de la ligne TVA due → delta != 0,
/// status "delta".
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_manual_edit_detected(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;
    // La ligne TVA due (compte 2000) passe de 81.00 à 71.00 (régularisation manuelle).
    tamper_vat_line(&pool, inv, seeded.accounts["2000"], dec!(71.00)).await;

    let report = gen_report(&pool, &seeded).await;
    assert_eq!(
        report.total_vat_due,
        dec!(81.00),
        "due dérivée inchangée (invoice_lines)"
    );
    assert_eq!(report.reconciliation_delta, dec!(10.00), "81 - 71 = 10");
    assert_eq!(report.reconciliation_status, "delta");
}

/// (c) Isolation périmètre ventes (F-OPUS-4) : une écriture MANUELLE sur le compte
/// TVA due (non liée à une facture) n'entre PAS dans le solde de référence → le
/// delta reste 0 malgré l'OD.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_isolates_non_invoice_entries(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;

    // OD manuelle : crédit 2000 de 500 (auto-liquidation / régularisation AFC),
    // contrepartie débit 4000 — AUCUNE facture ne pointe cette écriture.
    journal_entries::create(
        &pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: ymd(2026, 7, 1),
            journal: Journal::OD,
            description: "Régularisation AFC manuelle".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts["4000"],
                    debit: dec!(500.00),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["2000"],
                    debit: Decimal::ZERO,
                    credit: dec!(500.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap();

    let report = gen_report(&pool, &seeded).await;
    // Si l'isolation échouait (somme de TOUTES les lignes 2000), le solde serait
    // 581 et delta = 81 - 581 = -500. L'isolation par lien facture garde 81 → 0.
    assert_eq!(
        report.reconciliation_delta,
        dec!(0),
        "OD non liée exclue du solde"
    );
    assert_eq!(report.reconciliation_status, "ok");
}

/// (d) Compte TVA due non configuré (NULL, DC5-null) → delta 0, status "ok", même
/// si une facture validée pré-existante porte de la TVA (rien à réconcilier).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_payable_account_null(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;
    // Compte TVA due retiré APRÈS validation (cas dégénéré DC5-null).
    sqlx::query(
        "UPDATE company_invoice_settings SET default_vat_payable_account_id = NULL \
         WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();

    let report = gen_report(&pool, &seeded).await;
    assert_eq!(
        report.total_vat_due,
        dec!(81.00),
        "due dérivée toujours lisible"
    );
    assert_eq!(
        report.reconciliation_delta,
        dec!(0),
        "compte NULL → rien à réconcilier"
    );
    assert_eq!(report.reconciliation_status, "ok");
}

/// (e) Anti-IDOR : la facture (et sa ligne TVA due éditée) d'une company réelle ne
/// fuit jamais dans le rapport d'une AUTRE company. On génère pour une company
/// inexistante (pattern 18-1d `recoverable_scoped_by_company`) : aucune row
/// `company_invoice_settings`, aucune facture → delta 0, status "ok", pas d'erreur.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_scoped_by_company(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    // Facture validée PUIS éditée à la main → la company réelle a un delta de 31.
    let inv = create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;
    tamper_vat_line(&pool, inv, seeded.accounts["2000"], dec!(50.00)).await;

    // Rapport pour une autre company (inexistante) → aucune fuite, pas d'erreur.
    let other_company = seeded.company_id + 99_999;
    let report = vat_report::generate(&pool, other_company, &period_2026(seeded.fiscal_year_id))
        .await
        .unwrap();
    assert_eq!(
        report.total_vat_due,
        dec!(0),
        "aucune facture chez l'autre company"
    );
    assert_eq!(
        report.reconciliation_delta,
        dec!(0),
        "pas de fuite cross-tenant"
    );
    assert_eq!(report.reconciliation_status, "ok");
}

/// (f) Seuil : un écart sous le centime (0.005) → "ok" ; un écart de 0.01 → "delta".
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_threshold_one_centime(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;

    // 81.00 → 80.995 : delta 0.005 < 0.01 → "ok".
    tamper_vat_line(&pool, inv, seeded.accounts["2000"], dec!(80.995)).await;
    let report = gen_report(&pool, &seeded).await;
    assert_eq!(report.reconciliation_delta, dec!(0.005));
    assert_eq!(report.reconciliation_status, "ok", "sous le centime = ok");

    // 81.00 → 80.99 : delta 0.01 >= seuil → "delta".
    tamper_vat_line(&pool, inv, seeded.accounts["2000"], dec!(80.99)).await;
    let report = gen_report(&pool, &seeded).await;
    assert_eq!(report.reconciliation_delta, dec!(0.01));
    assert_eq!(report.reconciliation_status, "delta", "1 centime = delta");
}

/// (g) Non-régression : la TVA due dérivée, la récupérable et le solde restent
/// corrects quand la réconciliation s'ajoute (cas nominal, delta 0).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_preserves_existing_totals(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    // Facture multi-taux : 8.1 % (1000 → 81) + 0 % (500 → 0). Une seule ligne 2200
    // (taux > 0). Due dérivée = 81, solde 2200 ventes = 81 → delta 0.
    create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000)), (dec!(0.00), dec!(500))],
    )
    .await;

    let report = gen_report(&pool, &seeded).await;
    assert_eq!(report.total_base_ht, dec!(1500), "CA HT 1000 + 500");
    assert_eq!(report.total_vat_due, dec!(81.00), "TVA due taux > 0 seul");
    assert_eq!(report.total_vat_recoverable, dec!(0), "aucun achat");
    assert_eq!(report.vat_balance, dec!(81.00));
    assert_eq!(report.reconciliation_delta, dec!(0));
    assert_eq!(report.reconciliation_status, "ok");
}

/// (h) Renderer CSV : l'écart de réconciliation apparaît dans le CSV (cas delta).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_rendered_in_csv(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;
    tamper_vat_line(&pool, inv, seeded.accounts["2000"], dec!(71.00)).await;

    let report = gen_report(&pool, &seeded).await;
    assert_eq!(report.reconciliation_status, "delta");

    let mut buf: Vec<u8> = Vec::new();
    kesh_report::csv::render_vat_report_csv(&report, &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    assert!(
        text.contains("Écart de réconciliation"),
        "libellé écart présent : {text}"
    );
    assert!(text.contains("10.00"), "valeur écart présente : {text}");
}

/// (i, LOW-1 review) Écart NÉGATIF : la ligne TVA due est GONFLÉE à la main
/// (91 > 81 facturé) → delta = -10, status "delta" (symétrie de `abs()`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reconciliation_negative_delta_detected(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    let inv = create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;
    // 81.00 → 91.00 : comptabilisé > facturé (delta négatif).
    tamper_vat_line(&pool, inv, seeded.accounts["2000"], dec!(91.00)).await;

    let report = gen_report(&pool, &seeded).await;
    assert_eq!(report.reconciliation_delta, dec!(-10.00), "81 - 91 = -10");
    assert_eq!(report.reconciliation_status, "delta", "abs() symétrique");
}

/// (DC12) Avoir → la facture annulée sort du décompte : TVA due dérivée tombe à 0
/// ET le solde 2200 ventes (filtré status='validated') tombe aussi → delta reste 0.
/// Le décompte TVA de la période exclut la TVA de la facture créditée.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn credit_note_excludes_vat_from_report(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = seed_contact(&pool, &seeded).await;
    let invoice_id = create_validated_invoice(
        &pool,
        &seeded,
        contact,
        ymd(2026, 6, 15),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;

    // Avant avoir : TVA due 81, réconciliation cohérente.
    let before = gen_report(&pool, &seeded).await;
    assert_eq!(before.total_vat_due, dec!(81.00));
    assert_eq!(before.reconciliation_status, "ok");

    // Émission de l'avoir → facture cancelled + contre-passation.
    credit_notes::create_credit_note(
        &pool,
        kesh_db::entities::NewCreditNote {
            company_id: seeded.company_id,
            invoice_id,
            date: ymd(2026, 7, 1),
        },
        seeded.admin_user_id,
    )
    .await
    .expect("create credit note");

    // Après avoir : la facture (cancelled) sort des deux calculs → TVA due 0,
    // delta toujours cohérent (≈ 0).
    let after = gen_report(&pool, &seeded).await;
    assert_eq!(
        after.total_vat_due,
        dec!(0.00),
        "facture annulée exclue du décompte"
    );
    assert!(
        after.reconciliation_delta.abs() < dec!(0.01),
        "réconciliation reste cohérente après avoir (delta {})",
        after.reconciliation_delta
    );
}
