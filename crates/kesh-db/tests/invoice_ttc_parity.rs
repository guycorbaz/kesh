//! Test de parité 4-voies du TTC canonique (#246, Story 21-2a).
//!
//! Le TTC d'une facture a QUATRE définitions qui DOIVENT coïncider au centime :
//!   1. le helper Rust `kesh_core::accounting::vat::invoice_total_ttc` ;
//!   2. la forme SQL **scalaire** (`INVOICE_TTC_SUBQUERY_SQL`, sous-requête
//!      corrélée — 1er usage de `ROUND` SQL du projet) ;
//!   3. la forme SQL **agrégat** (`INVOICE_TTC_DERIVED_JOIN_SQL`, via
//!      `due_dates_summary`) ;
//!   4. le débit créance de l'écriture comptable (`generate_invoice_journal_lines`).
//!
//! Chaque forme SQL porte son propre `ROUND` — tester l'une ne couvre PAS
//! l'autre (validate V3-3). Les lignes seedées incluent les cas d'arrondi
//! limites : half-away (123.455 → 123.46), sous-centime (0.05 @ 8.1 → TVA
//! 0.00 par ligne), taux 0.
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_core::accounting::vat::invoice_total_ttc;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::invoices::{
    INVOICE_TTC_SUBQUERY_SQL, InvoiceListQuery, due_dates_summary,
};
use kesh_db::repositories::{contacts, invoices};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

#[sqlx::test(migrations = "./test-schema")]
async fn ttc_four_way_parity_mixed_rates_and_rounding_edges(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact_id = contacts::create(
        &pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client Parité TTC".into(),
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
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .expect("create contact")
    .id;

    // Lignes à quantité 1 (line_total = unit_price), taux mixtes + arrondis limites.
    let line_specs: &[(Decimal, Decimal)] = &[
        (dec!(8.10), dec!(100.00)),   // TVA 8.10
        (dec!(1.00), dec!(12345.50)), // 123.455 → 123.46 (half-away)
        (dec!(2.60), dec!(50.00)),    // 1.30
        (dec!(0.00), dec!(10.00)),    // taux 0 → 0.00
        (dec!(8.10), dec!(0.05)),     // 0.004050 → 0.00 (sous-centime, ×2 :
        (dec!(8.10), dec!(0.05)),     //  les arrondis PAR LIGNE s'additionnent)
    ];
    let new = NewInvoice {
        company_id: seeded.company_id,
        contact_id,
        date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        due_date: None,
        payment_terms: None,
        project_id: None,
        lines: line_specs
            .iter()
            .map(|(rate, price)| NewInvoiceLine {
                revenue_account_id: None,
                description: "Ligne parité".into(),
                quantity: dec!(1),
                unit_price: *price,
                vat_rate: *rate,
            })
            .collect(),
    };
    let (inv, lines) = invoices::create(&pool, seeded.admin_user_id, new)
        .await
        .expect("create invoice");

    // Voie 1 — helper Rust canonique.
    let rust_ttc = invoice_total_ttc(lines.iter().map(|l| (l.line_total, l.vat_rate)));

    // Voie 2 — forme SQL scalaire (sous-requête corrélée, alias `i` requis).
    let sql_scalar_ttc: Decimal = sqlx::query_scalar(&format!(
        "SELECT {INVOICE_TTC_SUBQUERY_SQL} FROM invoices i WHERE i.id = ?"
    ))
    .bind(inv.id)
    .fetch_one(&pool)
    .await
    .expect("scalar ttc query");
    assert_eq!(
        sql_scalar_ttc, rust_ttc,
        "forme SQL scalaire ≠ helper Rust (ROUND vs MidpointAwayFromZero ?)"
    );

    // Voie 4 — débit créance de l'écriture (validation de la facture).
    let validated =
        invoices::validate_invoice(&pool, seeded.company_id, inv.id, seeded.admin_user_id)
            .await
            .expect("validate invoice");
    let receivable = seeded.accounts["1100"];
    let creance = validated
        .journal_entry
        .lines
        .iter()
        .find(|l| l.account_id == receivable)
        .expect("ligne créance");
    assert_eq!(
        creance.debit, rust_ttc,
        "débit créance ≠ helper Rust (équivalence par associativité violée ?)"
    );

    // Voie 3 — forme SQL agrégat (table dérivée) via due_dates_summary.
    // La facture validée impayée est la seule de la company → unpaid_total = son TTC.
    let summary = due_dates_summary(&pool, seeded.company_id, &InvoiceListQuery::default())
        .await
        .expect("summary");
    assert_eq!(summary.unpaid_count, 1);
    assert_eq!(
        summary.unpaid_total, rust_ttc,
        "forme SQL agrégat (table dérivée) ≠ helper Rust"
    );

    // Valeur absolue attendue (fige le comportement, indépendamment des 4 voies) :
    // HT = 12505.60 ; TVA = 8.10 + 123.46 + 1.30 + 0 + 0.00 + 0.00 = 132.86.
    assert_eq!(rust_ttc, dec!(12638.46));
}

/// La colonne calculée `total_ttc` des items de liste (forme scalaire dans le
/// SELECT) coïncide avec le helper — et une facture à taux 0 a TTC = HT.
#[sqlx::test(migrations = "./test-schema")]
async fn list_items_total_ttc_matches_helper(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact_id = contacts::create(
        &pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client Liste TTC".into(),
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
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .expect("create contact")
    .id;

    for (rate, price, expected_ttc) in [
        (dec!(8.10), dec!(100.00), dec!(108.10)),
        (dec!(0.00), dec!(1234.56), dec!(1234.56)), // taux 0 → TTC = HT
    ] {
        let new = NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Ligne".into(),
                quantity: dec!(1),
                unit_price: price,
                vat_rate: rate,
            }],
        };
        let (inv, lines) = invoices::create(&pool, seeded.admin_user_id, new)
            .await
            .expect("create invoice");
        assert_eq!(
            invoice_total_ttc(lines.iter().map(|l| (l.line_total, l.vat_rate))),
            expected_ttc
        );

        // NB : `InvoiceListQuery::default()` (derive) a `limit: 0` → LIMIT 0.
        let result = invoices::list_by_company_paginated(
            &pool,
            seeded.company_id,
            InvoiceListQuery {
                limit: 50,
                ..Default::default()
            },
        )
        .await
        .expect("list");
        let item = result
            .items
            .iter()
            .find(|it| it.id == inv.id)
            .expect("item présent");
        assert_eq!(item.total_ttc, expected_ttc, "colonne SQL total_ttc");
        assert_eq!(item.total_amount, price, "total_amount reste HT");
    }
}
