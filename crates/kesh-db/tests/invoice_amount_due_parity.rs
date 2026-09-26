//! Le résiduel juste — Story 25-4-a (#455, #456).
//!
//! `amount_due` = `TTC − avoir émis − Σ règlements`. Trois garanties :
//!
//! 1. **L'avoir est compté TTC** (#455) : le terme « avoir » vaut le crédit que
//!    l'écriture de l'avoir porte à la créance — pas `credit_notes.total_amount`,
//!    qui est HT. C'est l'assertion de **concordance avec le grand livre** qui le
//!    prouve, pas une valeur recalculée à côté.
//! 2. **Les formes scalaire et jointe sont d'accord**, facture par facture, pour le
//!    réglé comme pour l'avoir — le test de parité que le doc-comment
//!    d'`INVOICE_SETTLED_SUBQUERY_SQL` annonçait sans qu'il existe. Les formes
//!    jointes n'avaient aucun appelant ; 25-4-b en sera le premier.
//! 3. **Chaque facture porte une TVA non nulle.** À 0 %, HT = TTC et #455 est
//!    invisible — c'est ainsi qu'il a passé tous les tests de la 24-2.
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewCreditNote, NewInvoice, NewInvoiceLine, SettlementChoice};
use kesh_db::repositories::invoice_settlements::{
    INVOICE_CREDITED_DERIVED_JOIN_SQL, INVOICE_CREDITED_SUBQUERY_SQL,
    INVOICE_SETTLED_DERIVED_JOIN_SQL, INVOICE_SETTLED_SUBQUERY_SQL, amount_due,
};
use kesh_db::repositories::{contacts, credit_notes, invoice_settlements_write, invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

async fn make_contact(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client Résiduel".into(),
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
    .expect("contact")
    .id
}

/// Une facture validée par le **vrai chemin** (écriture de vente TVA comprise) —
/// jamais par le helper à 0 % de `invoice_settlement.rs`.
async fn validated(
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

async fn settle(pool: &MySqlPool, seeded: &SeededCompany, invoice_id: i64, amount: Decimal) -> i64 {
    invoice_settlements_write::settle_invoice(
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
    .expect("règlement")
    .journal_entry_id
}

async fn credit(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    invoice_id: i64,
) -> credit_notes::IssuedCreditNote {
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
    .expect("avoir")
}

/// L'état hérité **règlement puis avoir**, que 25-4-a rend inatteignable par
/// l'application mais qu'un `.keshbackup` v0.12.0 peut ramener.
///
/// Gabarit « détacher, créditer, rattacher » (AC 13) : les deux écritures
/// viennent des **vrais chemins** ; seul le rattachement de la ligne de
/// règlement passe en SQL, le temps de l'avoir.
async fn settle_then_legacy_credit(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    invoice_id: i64,
    amount: Decimal,
) {
    settle(pool, seeded, invoice_id, amount).await;
    let parking = validated(pool, seeded, contact_id, &[(dec!(8.10), dec!(10.00))]).await;
    sqlx::query("UPDATE invoice_settlements SET invoice_id = ? WHERE invoice_id = ?")
        .bind(parking)
        .bind(invoice_id)
        .execute(pool)
        .await
        .expect("détacher");
    credit(pool, seeded, invoice_id).await;
    sqlx::query("UPDATE invoice_settlements SET invoice_id = ? WHERE invoice_id = ?")
        .bind(invoice_id)
        .bind(parking)
        .execute(pool)
        .await
        .expect("rattacher");
}

async fn credited_scalar(pool: &MySqlPool, invoice_id: i64) -> Decimal {
    sqlx::query_scalar(&format!(
        "SELECT {INVOICE_CREDITED_SUBQUERY_SQL} FROM invoices i WHERE i.id = ?"
    ))
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .expect("avoir scalaire")
}

/// AC 1, AC 7 — ⛔ le terme « avoir » du résiduel égale le crédit que
/// l'écriture de l'avoir porte à la créance. Multi-taux, arrondis limites.
#[sqlx::test(migrations = "./test-schema")]
async fn credited_amount_is_ttc_and_matches_the_ledger(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, &seeded).await;
    let invoice_id = validated(
        &pool,
        &seeded,
        contact,
        &[
            (dec!(8.10), dec!(1000.00)),
            (dec!(2.60), dec!(123.45)),
            (dec!(8.10), dec!(0.05)),
        ],
    )
    .await;

    let issued = credit(&pool, &seeded, invoice_id).await;
    let receivable = seeded.accounts["1100"];
    let ledger_credit: Decimal = issued
        .journal_entry
        .lines
        .iter()
        .filter(|l| l.account_id == receivable)
        .map(|l| l.credit)
        .sum();

    // 1000 + 81.00 ; 123.45 + 3.21 ; 0.05 + 0.00 (sous-centime) = 1207.71
    assert_eq!(ledger_credit, dec!(1207.71), "crédit créance de l'avoir");
    assert_eq!(credited_scalar(&pool, invoice_id).await, ledger_credit);
    assert_ne!(
        issued.credit_note.total_amount, ledger_credit,
        "le montant stocké est HT : s'il égalait le crédit, le test ne distinguerait rien"
    );
}

/// AC 8 — facture créditée, jamais réglée : il ne reste rien à payer.
#[sqlx::test(migrations = "./test-schema")]
async fn amount_due_of_a_credited_invoice_is_zero(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, &seeded).await;
    let invoice_id = validated(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00))]).await;
    credit(&pool, &seeded, invoice_id).await;

    assert_eq!(
        amount_due(&pool, invoice_id).await.unwrap(),
        Decimal::ZERO,
        "et non 8.10, la TVA (#455)"
    );
}

/// AC 8 — l'état hérité : 40 encaissés sur une vente annulée. Le reste dû vaut
/// −40, un trop-perçu visible — pas `TVA − 40`, et pas écrêté à zéro.
#[sqlx::test(migrations = "./test-schema")]
async fn amount_due_after_settlement_then_legacy_credit_is_minus_settled(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, &seeded).await;
    let invoice_id = validated(&pool, &seeded, contact, &[(dec!(8.10), dec!(100.00))]).await;
    settle_then_legacy_credit(&pool, &seeded, contact, invoice_id, dec!(40.00)).await;

    assert_eq!(amount_due(&pool, invoice_id).await.unwrap(), dec!(-40.00));
}

/// AC 6 — la forme scalaire et la forme jointe rendent la même valeur, facture
/// par facture, pour le réglé et pour l'avoir. Jeu : sans règlement, un
/// règlement, deux règlements, avoir émis, avoir brouillon, état hérité.
#[sqlx::test(migrations = "./test-schema")]
async fn settled_and_credited_forms_are_at_parity(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let contact = make_contact(&pool, &seeded).await;
    let lines = [(dec!(8.10), dec!(100.00)), (dec!(2.60), dec!(33.33))];

    let _plain = validated(&pool, &seeded, contact, &lines).await;
    let one = validated(&pool, &seeded, contact, &lines).await;
    settle(&pool, &seeded, one, dec!(40.00)).await;
    let two = validated(&pool, &seeded, contact, &lines).await;
    settle(&pool, &seeded, two, dec!(40.00)).await;
    settle(&pool, &seeded, two, dec!(12.34)).await;
    let credited = validated(&pool, &seeded, contact, &lines).await;
    credit(&pool, &seeded, credited).await;
    let legacy = validated(&pool, &seeded, contact, &lines).await;
    settle_then_legacy_credit(&pool, &seeded, contact, legacy, dec!(25.00)).await;

    // Avoir BROUILLON : sans écriture, il n'éteint rien — monté en SQL, aucun
    // chemin applicatif ne produisant de brouillon (single-step, DC5).
    let draft_target = validated(&pool, &seeded, contact, &lines).await;
    let cn = sqlx::query(
        "INSERT INTO credit_notes (company_id, invoice_id, contact_id, date, status, total_amount) \
         VALUES (?, ?, ?, ?, 'draft', 133.33)",
    )
    .bind(seeded.company_id)
    .bind(draft_target)
    .bind(contact)
    .bind(d(2026, 7, 1))
    .execute(&pool)
    .await
    .expect("avoir brouillon");
    sqlx::query(
        "INSERT INTO credit_note_lines (credit_note_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'Ligne', 1, 100.00, 8.10, 100.00)",
    )
    .bind(cn.last_insert_id())
    .execute(&pool)
    .await
    .expect("ligne d'avoir brouillon");

    let rows: Vec<(i64, Decimal, Decimal, Decimal, Decimal)> = sqlx::query_as(&format!(
        "SELECT i.id, {INVOICE_SETTLED_SUBQUERY_SQL}, COALESCE(st.settled, 0), \
                {INVOICE_CREDITED_SUBQUERY_SQL}, COALESCE(cnt.credited, 0) \
         FROM invoices i {INVOICE_SETTLED_DERIVED_JOIN_SQL} {INVOICE_CREDITED_DERIVED_JOIN_SQL} \
         WHERE i.company_id = ? ORDER BY i.id"
    ))
    .bind(seeded.company_id)
    .fetch_all(&pool)
    .await
    .expect("parité");

    // Six factures du jeu, plus la facture auxiliaire du gabarit.
    assert_eq!(rows.len(), 7, "le jeu entier est lu : {rows:?}");
    for (id, settled_s, settled_j, credited_s, credited_j) in &rows {
        assert_eq!(settled_s, settled_j, "réglé, facture {id}");
        assert_eq!(credited_s, credited_j, "avoir, facture {id}");
    }
    // Anti-vacuité : la parité porte sur des valeurs NON nulles des deux côtés.
    let by_id = |x: i64| rows.iter().find(|r| r.0 == x).unwrap();
    assert_eq!(by_id(two).1, dec!(52.34));
    assert_eq!(by_id(credited).3, dec!(142.30)); // 100 + 8.10 ; 33.33 + 0.87
    assert_eq!(
        by_id(draft_target).3,
        Decimal::ZERO,
        "un brouillon n'éteint rien"
    );
}
