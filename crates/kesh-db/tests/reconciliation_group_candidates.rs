//! Story 43-2 (#823, AC5) — `find_group_candidates` : les factures candidates
//! d'un règlement groupé, clients pour un crédit, fournisseurs pour un débit,
//! tous tiers, cherchées par texte.
//!
//! Tous les tests montent le squash (`test-schema`). Décor synthétique.

use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::repositories::reconciliation_group_candidates::{
    CandidateFamily, GROUP_CANDIDATES_SCAN_LIMIT, GroupCandidatesQuery, find_group_candidates,
};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn jour(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, m, d).expect("date valide")
}

/// La borne haute de la fenêtre des tests.
fn borne() -> NaiveDate {
    jour(10, 31)
}

struct Montage {
    s: SeededCompany,
    /// Une écriture partagée par toutes les pièces : la recherche ne lit que
    /// sa présence (`journal_entry_id IS NOT NULL`, CHECK des fournisseurs).
    entry_id: i64,
}

async fn monter(pool: &MySqlPool) -> Montage {
    let s = seed_accounting_company(pool).await.expect("seed");
    let entry_id = journal_entries::create(
        pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: jour(1, 2),
            journal: Journal::Ventes,
            description: "Écriture de montage".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: s.accounts["1100"],
                    debit: dec!(1),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: s.accounts["3000"],
                    debit: Decimal::ZERO,
                    credit: dec!(1),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture")
    .entry
    .id;
    Montage { s, entry_id }
}

async fn contact(pool: &MySqlPool, company_id: i64, name: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', ?, TRUE, TRUE) RETURNING id",
    )
    .bind(company_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .expect("contact")
}

/// Une facture client `status` de TTC `amount` (une ligne, TVA 0).
async fn facture(
    pool: &MySqlPool,
    m: &Montage,
    contact_id: i64,
    number: &str,
    amount: Decimal,
    date: NaiveDate,
    status: &str,
) -> i64 {
    // Une écriture pour toute facture émise (validée ou annulée) : le
    // brouillon seul n'en a pas, et le filtre de statut reste discriminant.
    let entry = (status != "draft").then_some(m.entry_id);
    let id = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, due_date, \
         total_amount, journal_entry_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(m.s.company_id)
    .bind(contact_id)
    .bind(number)
    .bind(status)
    .bind(date)
    .bind(date)
    .bind(amount)
    .bind(entry)
    .execute(pool)
    .await
    .expect("facture")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, \
         vat_rate, line_total) VALUES (?, 0, 'Prestation', 1, ?, 0, ?)",
    )
    .bind(id)
    .bind(amount)
    .bind(amount)
    .execute(pool)
    .await
    .expect("ligne");
    id
}

/// Une facture fournisseur `status`.
#[allow(clippy::too_many_arguments)]
async fn facture_fournisseur(
    pool: &MySqlPool,
    m: &Montage,
    contact_id: i64,
    internal: &str,
    supplier_number: &str,
    reference: &str,
    amount: Decimal,
    date: NaiveDate,
    status: &str,
) -> i64 {
    let entry = (status != "draft").then_some(m.entry_id);
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, internal_number, \
         supplier_invoice_number, payment_reference, status, invoice_date, due_date, \
         total_amount, purchase_journal_entry_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(m.s.company_id)
    .bind(contact_id)
    .bind(internal)
    .bind(supplier_number)
    .bind(reference)
    .bind(status)
    .bind(date)
    .bind(date)
    .bind(amount)
    .bind(entry)
    .execute(pool)
    .await
    .expect("facture fournisseur")
    .last_insert_id() as i64
}

async fn chercher(
    pool: &MySqlPool,
    m: &Montage,
    family: CandidateFamily,
    q: Option<&str>,
    exclude: &[i64],
) -> Vec<i64> {
    find_group_candidates(
        pool,
        GroupCandidatesQuery {
            company_id: m.s.company_id,
            family,
            latest_invoice_date: borne(),
            q,
            exclude,
        },
    )
    .await
    .expect("recherche")
    .candidates
    .into_iter()
    .map(|c| c.id)
    .collect()
}

/// AC5 — un crédit : les factures clients ouvertes de **tous** les clients,
/// ni brouillon, ni payée, ni d'une autre société ; la borne de date : la
/// facture à la date limite est rendue, celle du lendemain ne l'est pas ; le
/// reste dû brut à quatre décimales.
#[sqlx::test(migrations = "./test-schema")]
async fn client_candidates_follow_the_settlement_rules(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = contact(&pool, m.s.company_id, "Alpha Conseil").await;
    let b = contact(&pool, m.s.company_id, "Beta Services").await;
    let f1 = facture(&pool, &m, a, "F-1", dec!(10.0050), jour(6, 1), "validated").await;
    let f2 = facture(&pool, &m, b, "F-2", dec!(300), jour(6, 2), "validated").await;
    let limite = facture(&pool, &m, a, "F-3", dec!(50), borne(), "validated").await;
    let _apres = facture(
        &pool,
        &m,
        a,
        "F-4",
        dec!(50),
        borne().succ_opt().unwrap(),
        "validated",
    )
    .await;
    let _brouillon = facture(&pool, &m, a, "F-5", dec!(50), jour(6, 3), "draft").await;
    let payee = facture(&pool, &m, b, "F-6", dec!(70), jour(6, 4), "validated").await;
    sqlx::query("UPDATE invoices SET paid_at = '2026-06-10 10:00:00' WHERE id = ?")
        .bind(payee)
        .execute(&pool)
        .await
        .unwrap();
    // Annulée, avec son écriture et sans `paid_at` : seul le statut l'écarte
    // (revue P1, B10).
    let _annulee = facture(&pool, &m, a, "F-7", dec!(40), jour(6, 3), "cancelled").await;
    // Réglée en partie : rendue avec son reste dû brut, distinct du TTC (B3).
    let partielle = facture(&pool, &m, b, "F-8", dec!(100), jour(6, 5), "validated").await;
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
         settled_on, settlement_type, settlement_account_id) \
         VALUES (?, ?, ?, 30.0050, '2026-06-06', 'internal_account', ?)",
    )
    .bind(m.s.company_id)
    .bind(partielle)
    .bind(m.entry_id)
    .bind(m.s.accounts["1000"])
    .execute(&pool)
    .await
    .expect("règlement partiel");
    // Créditée en entier par un avoir émis : validée, non payée, reste dû nul —
    // écartée par `HAVING amount_due > 0` (B3).
    let creditee = facture(&pool, &m, b, "F-9", dec!(50), jour(6, 6), "validated").await;
    // Un avoir ordinaire (revue P2, P2-8) : une ligne de 50, TVA 0, arrondi nul.
    let avoir = sqlx::query(
        "INSERT INTO credit_notes (company_id, contact_id, invoice_id, status, date, \
         credit_note_number, total_amount, journal_entry_id, rounding_amount) \
         VALUES (?, ?, ?, 'issued', '2026-06-07', 'AV-1', 50, ?, 0)",
    )
    .bind(m.s.company_id)
    .bind(b)
    .bind(creditee)
    .bind(m.entry_id)
    .execute(&pool)
    .await
    .expect("avoir")
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO credit_note_lines (credit_note_id, position, description, quantity, \
         unit_price, vat_rate, line_total) VALUES (?, 0, 'Avoir total', 1, 50, 0, 50)",
    )
    .bind(avoir)
    .execute(&pool)
    .await
    .expect("ligne d'avoir");
    let found = find_group_candidates(
        &pool,
        GroupCandidatesQuery {
            company_id: m.s.company_id,
            family: CandidateFamily::Client,
            latest_invoice_date: borne(),
            q: None,
            exclude: &[],
        },
    )
    .await
    .unwrap();
    let ids: Vec<i64> = found.candidates.iter().map(|c| c.id).collect();
    assert_eq!(
        ids,
        vec![f1, f2, partielle, limite],
        "date puis identifiant"
    );
    assert!(!found.scan_limit_reached);
    let first = &found.candidates[0];
    assert_eq!(first.amount_due, dec!(10.0050), "reste dû brut");
    assert_eq!(first.total_ttc, dec!(10.0050));
    assert_eq!(first.contact_name, "Alpha Conseil");
    assert_eq!(first.invoice_number.as_deref(), Some("F-1"));
    assert_eq!(first.internal_number, None);
    let part = &found.candidates[2];
    assert_eq!(part.total_ttc, dec!(100));
    assert_eq!(part.amount_due, dec!(69.9950), "reste dû, non le TTC");
}

/// AC5 — un crédit : une facture validée de l'autre société n'est jamais
/// rendue, même si son tiers porte le même nom.
#[sqlx::test(migrations = "./test-schema")]
async fn another_company_invoice_is_never_returned(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = contact(&pool, m.s.company_id, "Alpha Conseil").await;
    let mine = facture(&pool, &m, a, "F-1", dec!(10), jour(6, 1), "validated").await;
    let other_company = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let mut autre = m.s.clone();
    autre.company_id = other_company;
    let autre = Montage {
        s: autre,
        entry_id: m.entry_id,
    };
    let b = contact(&pool, other_company, "Alpha Conseil").await;
    let _theirs = facture(&pool, &autre, b, "F-1", dec!(10), jour(6, 1), "validated").await;
    let g = contact(&pool, other_company, "Gamma").await;
    let _theirs_supplier = facture_fournisseur(
        &pool,
        &autre,
        g,
        "FF-1",
        "G",
        "R",
        dec!(10),
        jour(6, 1),
        "open",
    )
    .await;
    assert!(
        chercher(&pool, &m, CandidateFamily::Supplier, None, &[])
            .await
            .is_empty()
    );
    assert_eq!(
        chercher(&pool, &m, CandidateFamily::Client, Some("Alpha"), &[]).await,
        vec![mine]
    );
    assert_eq!(
        chercher(&pool, &m, CandidateFamily::Client, None, &[]).await,
        vec![mine]
    );
}

/// AC5 — un débit : les factures fournisseur `open` de tous les fournisseurs,
/// ni brouillon, ni payée, ni engagée dans un lot `generated` ; celle d'un lot
/// confirmé reste candidate ; la borne de date.
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_candidates_follow_the_settlement_rules(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = contact(&pool, m.s.company_id, "Gamma Fournitures").await;
    let b = contact(&pool, m.s.company_id, "Delta Énergie").await;
    let s1 = facture_fournisseur(
        &pool,
        &m,
        a,
        "FF-2026-001",
        "G-77",
        "RF18539007547034",
        dec!(120),
        jour(6, 1),
        "open",
    )
    .await;
    let s2 = facture_fournisseur(
        &pool,
        &m,
        b,
        "FF-2026-002",
        "D-1",
        "210000000003139471430009017",
        dec!(80),
        jour(6, 2),
        "open",
    )
    .await;
    let _draft =
        facture_fournisseur(&pool, &m, a, "FF-B", "B", "R", dec!(5), jour(6, 3), "draft").await;
    let _apres = facture_fournisseur(
        &pool,
        &m,
        a,
        "FF-2026-009",
        "A",
        "R",
        dec!(5),
        borne().succ_opt().unwrap(),
        "open",
    )
    .await;
    let lot = facture_fournisseur(
        &pool,
        &m,
        b,
        "FF-2026-003",
        "D-2",
        "R2",
        dec!(40),
        jour(6, 4),
        "open",
    )
    .await;
    let confirme = facture_fournisseur(
        &pool,
        &m,
        b,
        "FF-2026-004",
        "D-3",
        "R3",
        dec!(41),
        jour(6, 5),
        "open",
    )
    .await;
    let bank_account_id = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'Banque', 'CH9300762011623852957', ?)",
    )
    .bind(m.s.company_id)
    .bind(m.s.accounts["1000"])
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (si, status) in [(lot, "generated"), (confirme, "confirmed")] {
        let batch = sqlx::query(
            "INSERT INTO payment_batches (company_id, bank_account_id, status, \
             requested_execution_date, msg_id, payment_info_id) \
             VALUES (?, ?, ?, ?, REPLACE(UUID(), '-', ''), 'P')",
        )
        .bind(m.s.company_id)
        .bind(bank_account_id)
        .bind(status)
        .bind(jour(6, 6))
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        sqlx::query(
            "INSERT INTO payment_batch_items (payment_batch_id, supplier_invoice_id, position, \
             end_to_end_id, amount) VALUES (?, ?, 1, 'E2E', 1)",
        )
        .bind(batch)
        .bind(si)
        .execute(&pool)
        .await
        .unwrap();
    }
    // TTC nul (B11) et payée (B11) : écartées ; à la date limite (A14) : rendue.
    let _zero =
        facture_fournisseur(&pool, &m, a, "FF-Z", "Z", "R", dec!(0), jour(6, 7), "open").await;
    let payee =
        facture_fournisseur(&pool, &m, a, "FF-P", "P", "R", dec!(9), jour(6, 8), "open").await;
    sqlx::query(
        "UPDATE supplier_invoices SET status = 'paid', settlement_type = 'bank_transfer', \
         settlement_bank_account_id = ?, settlement_journal_entry_id = ?, paid_at = NOW() \
         WHERE id = ?",
    )
    .bind(bank_account_id)
    .bind(m.entry_id)
    .bind(payee)
    .execute(&pool)
    .await
    .expect("facture payée");
    let a_la_limite = facture_fournisseur(
        &pool,
        &m,
        b,
        "FF-2026-010",
        "L",
        "R",
        dec!(12),
        borne(),
        "open",
    )
    .await;

    let found = find_group_candidates(
        &pool,
        GroupCandidatesQuery {
            company_id: m.s.company_id,
            family: CandidateFamily::Supplier,
            latest_invoice_date: borne(),
            q: None,
            exclude: &[],
        },
    )
    .await
    .unwrap();
    let ids: Vec<i64> = found.candidates.iter().map(|c| c.id).collect();
    assert_eq!(ids, vec![s1, s2, confirme, a_la_limite]);
    let first = &found.candidates[0];
    assert_eq!(first.internal_number.as_deref(), Some("FF-2026-001"));
    assert_eq!(first.supplier_invoice_number.as_deref(), Some("G-77"));
    assert_eq!(first.payment_reference.as_deref(), Some("RF18539007547034"));
    assert_eq!(first.amount_due, dec!(120));
    assert_eq!(first.invoice_number, None);
    // Les factures clients ne sortent jamais pour un débit (les identifiants
    // des deux tables se recouvrent : la liste se compare, non l'identifiant).
    let _a_client = facture(&pool, &m, a, "F-1", dec!(10), jour(6, 1), "validated").await;
    assert_eq!(
        chercher(&pool, &m, CandidateFamily::Supplier, None, &[]).await,
        vec![s1, s2, confirme, a_la_limite]
    );
}

/// AC5 — `q`, un test par champ : nom (≥ 3 caractères, par l'index), nom à
/// jetons courts (« BL SA », par le repli `LIKE`), numéro client ; numéro
/// interne, numéro du fournisseur, référence.
#[sqlx::test(migrations = "./test-schema")]
async fn each_search_field_finds_its_invoice(pool: MySqlPool) {
    let m = monter(&pool).await;
    let alpha = contact(&pool, m.s.company_id, "Alpha Conseil").await;
    let bl = contact(&pool, m.s.company_id, "BL SA").await;
    let fa = facture(
        &pool,
        &m,
        alpha,
        "F-2026-0012",
        dec!(10),
        jour(6, 1),
        "validated",
    )
    .await;
    let fb = facture(
        &pool,
        &m,
        bl,
        "F-2026-0013",
        dec!(10),
        jour(6, 2),
        "validated",
    )
    .await;
    let client = |q: &'static str| {
        let pool = pool.clone();
        let m = &m;
        async move { chercher(&pool, m, CandidateFamily::Client, Some(q), &[]).await }
    };
    assert_eq!(client("Alph").await, vec![fa], "nom, préfixe");
    assert_eq!(client("  Alpha  ").await, vec![fa], "terme taillé");
    assert_eq!(
        client("BL SA").await,
        vec![fb],
        "jetons courts : repli LIKE"
    );
    assert_eq!(client("BL").await, vec![fb], "jeton court seul");
    assert_eq!(client("0012").await, vec![fa], "numéro client");
    assert_eq!(
        client("F-2026-001").await,
        vec![fa, fb],
        "fragment de numéro"
    );
    assert!(client("Inconnu").await.is_empty());
    assert_eq!(client("").await, vec![fa, fb], "vide : aucun filtre");
    assert_eq!(client("   ").await, vec![fa, fb], "blanc : aucun filtre");

    let g = contact(&pool, m.s.company_id, "Gamma Fournitures").await;
    let sa = facture_fournisseur(
        &pool,
        &m,
        g,
        "FF-2026-021",
        "G-77",
        "RF18539007547034",
        dec!(10),
        jour(6, 1),
        "open",
    )
    .await;
    let sb = facture_fournisseur(
        &pool,
        &m,
        bl,
        "FF-2026-022",
        "BLX-9",
        "210000000003139471430009017",
        dec!(10),
        jour(6, 2),
        "open",
    )
    .await;
    let supplier = |q: &'static str| {
        let pool = pool.clone();
        let m = &m;
        async move { chercher(&pool, m, CandidateFamily::Supplier, Some(q), &[]).await }
    };
    assert_eq!(supplier("Gamma").await, vec![sa], "nom");
    assert_eq!(supplier("BL SA").await, vec![sb], "nom à jetons courts");
    assert_eq!(supplier("2026-021").await, vec![sa], "numéro interne");
    assert_eq!(supplier("G-77").await, vec![sa], "numéro du fournisseur");
    assert_eq!(
        supplier("BLX").await,
        vec![sb],
        "numéro du fournisseur, préfixe"
    );
    assert_eq!(supplier("0003139471").await, vec![sb], "référence");
    assert_eq!(supplier("RF1853").await, vec![sa], "référence RF");
}

/// AC5 — injection : les caractères de `LIKE` et des opérateurs booléens ne
/// rendent ni résultat parasite ni erreur ; un `%` littéral se trouve.
#[sqlx::test(migrations = "./test-schema")]
async fn special_characters_neither_leak_nor_fail(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = contact(&pool, m.s.company_id, "Alpha Conseil").await;
    let f = facture(&pool, &m, a, "F-1", dec!(10), jour(6, 1), "validated").await;
    let pct = facture(&pool, &m, a, "F-100%", dec!(10), jour(6, 2), "validated").await;
    // (`-` n'y est pas : il est littéral dans un numéro, `F-1` le contient.)
    for q in ["%", "_", "\\", "*", "+", "\"", "%_%", "a%b", "~@<>()"] {
        let found = chercher(&pool, &m, CandidateFamily::Client, Some(q), &[]).await;
        let expected: Vec<i64> = if q == "%" { vec![pct] } else { vec![] };
        assert_eq!(found, expected, "terme {q:?}");
        let found = chercher(&pool, &m, CandidateFamily::Supplier, Some(q), &[]).await;
        assert!(found.is_empty(), "terme {q:?}");
    }
    // Un opérateur booléen collé à un nom est retiré, le nom se trouve
    // (`escape_boolean_ft`) : `"Alpha` et `+Alpha*` valent `Alpha`.
    for q in ["\"Alpha", "+Alpha*"] {
        assert_eq!(
            chercher(&pool, &m, CandidateFamily::Client, Some(q), &[]).await,
            vec![f, pct],
            "terme {q:?}"
        );
    }
    // Sonde : le même terme sans caractère spécial trouve bien.
    assert_eq!(
        chercher(&pool, &m, CandidateFamily::Client, Some("F-1"), &[]).await,
        vec![f, pct]
    );
}

/// AC5 — `exclude` retire les factures déjà imputées, avant le plafond.
#[sqlx::test(migrations = "./test-schema")]
async fn exclude_removes_the_listed_invoices(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = contact(&pool, m.s.company_id, "Alpha Conseil").await;
    let f1 = facture(&pool, &m, a, "F-1", dec!(10), jour(6, 1), "validated").await;
    let f2 = facture(&pool, &m, a, "F-2", dec!(10), jour(6, 2), "validated").await;
    let f3 = facture(&pool, &m, a, "F-3", dec!(10), jour(6, 3), "validated").await;
    assert_eq!(
        chercher(&pool, &m, CandidateFamily::Client, None, &[f1, f3]).await,
        vec![f2]
    );
    assert_eq!(
        chercher(&pool, &m, CandidateFamily::Client, Some("F-"), &[f2]).await,
        vec![f1, f3]
    );
}

/// AC5, L2 — au-delà de 500 factures, l'échantillon est déterminé (les plus
/// anciennes par date puis identifiant) et le plafond se dit ; à 500 tout
/// juste, il ne se dit pas.
#[sqlx::test(migrations = "./test-schema")]
async fn the_scan_limit_is_deterministic_and_reported(pool: MySqlPool) {
    let m = monter(&pool).await;
    let a = contact(&pool, m.s.company_id, "Alpha Conseil").await;
    // 500 factures validées, dates du 2026-01-01 au 2026-05-15 (par 100).
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id) \
         SELECT ?, ?, CONCAT('M-', seq), 'validated', \
                DATE_ADD('2026-01-01', INTERVAL (seq MOD 100) DAY), 10, ? \
         FROM seq_1_to_500",
    )
    .bind(m.s.company_id)
    .bind(a)
    .bind(m.entry_id)
    .execute(&pool)
    .await
    .expect("factures en masse");
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, \
         vat_rate, line_total) \
         SELECT id, 0, 'Prestation', 1, 10, 0, 10 FROM invoices WHERE company_id = ?",
    )
    .bind(m.s.company_id)
    .execute(&pool)
    .await
    .expect("lignes en masse");
    let query = GroupCandidatesQuery {
        company_id: m.s.company_id,
        family: CandidateFamily::Client,
        latest_invoice_date: borne(),
        q: None,
        exclude: &[],
    };
    let full = find_group_candidates(&pool, query).await.unwrap();
    assert_eq!(full.candidates.len() as i64, GROUP_CANDIDATES_SCAN_LIMIT);
    assert!(!full.scan_limit_reached, "500 tout juste : rien de coupé");

    // Une 501e, la plus récente : hors de l'échantillon, plafond signalé.
    let late = facture(&pool, &m, a, "M-LATE", dec!(10), jour(9, 1), "validated").await;
    // Une 502e, la plus ancienne : dans l'échantillon.
    let early = facture(&pool, &m, a, "M-EARLY", dec!(10), jour(1, 1), "validated").await;
    let cut = find_group_candidates(&pool, query).await.unwrap();
    assert_eq!(cut.candidates.len() as i64, GROUP_CANDIDATES_SCAN_LIMIT);
    assert!(cut.scan_limit_reached);
    let ids: Vec<i64> = cut.candidates.iter().map(|c| c.id).collect();
    assert!(ids.contains(&early));
    assert!(!ids.contains(&late));
    let mut sorted = cut.candidates.clone();
    sorted.sort_by(|x, y| x.date.cmp(&y.date).then(x.id.cmp(&y.id)));
    assert_eq!(sorted, cut.candidates, "date puis identifiant");
    let again = find_group_candidates(&pool, query).await.unwrap();
    assert_eq!(again, cut, "même échantillon à chaque lecture");
}
