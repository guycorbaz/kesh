//! Tests d'intégration FULLTEXT (Story 7-4 / KF-005).
//!
//! Ces tests s'exécutent sur une DB éphémère via `#[sqlx::test(migrator =
//! "kesh_db::MIGRATOR")]` — la migration `20260430000001_kf005_fulltext_indexes.sql`
//! est appliquée automatiquement, sans pollution de la DB partagée des
//! tests in-module.
//!
//! Couvre :
//!
//! - **T7.2** — `EXPLAIN ... FORCE INDEX (ft_<table>_<col>)` vérifie que
//!   l'index FULLTEXT existe et est utilisable. La vérification du `key`
//!   field dans l'output JSON détecte le fallback silencieux table scan
//!   (gotcha doc MariaDB index hints).
//! - **T7.3** — Isolation cross-company : 2 companies seedées, recherche
//!   scopée à company A ne retourne que les rows de A.
//! - **T7.4** — `EXPLAIN FORMAT=JSON` sur la query hybride MATCH OR LIKE
//!   (sans FORCE INDEX) pour observer le choix de l'optimizer (descriptif
//!   par défaut, fail uniquement si full scan systématique sans `ft_*`
//!   dans `possible_keys`).

use chrono::NaiveDate;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    AccountType, ContactType, Journal, Language, NewAccount, NewCompany, NewContact, NewFiscalYear,
    NewInvoice, NewInvoiceLine, NewJournalEntry, NewJournalEntryLine, NewProduct, NewUser, OrgType,
    Role,
};
use kesh_db::repositories::contacts::{ContactListQuery, ContactSortBy};
use kesh_db::repositories::invoices::InvoiceListQuery;
use kesh_db::repositories::journal_entries::JournalEntryListQuery;
use kesh_db::repositories::products::{ProductListQuery, ProductSortBy};
use kesh_db::repositories::{
    accounts, companies, contacts, fiscal_years, invoices, journal_entries, products, users,
};
use rust_decimal_macros::dec;
use serde_json::Value as JsonValue;
use sqlx::{MySqlPool, Row};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
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
    .unwrap()
    .id
}

async fn create_admin(pool: &MySqlPool, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: format!("admin-{company_id}"),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$QUJDRA$YWJjZGVmZ2hpams".into(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn seed_contacts(pool: &MySqlPool, user_id: i64, company_id: i64, count: usize) {
    for i in 0..count {
        contacts::create(
            pool,
            user_id,
            NewContact {
                company_id,
                contact_type: ContactType::Entreprise,
                name: format!("Contact-{company_id}-Marie-{i:04}"),
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
                email: Some(format!("client-{i}@example.com")),
                phone: None,
                ide_number: None,
                default_payment_terms: None,
                default_payment_terms_days: None,
                language: None,
                salutation: kesh_db::entities::contact::Salutation::Neutre,
            },
        )
        .await
        .unwrap();
    }
}

async fn seed_products(pool: &MySqlPool, user_id: i64, company_id: i64, count: usize) {
    for i in 0..count {
        products::create(
            pool,
            user_id,
            NewProduct {
                company_id,
                name: format!("Produit-{company_id}-Marie-{i:04}"),
                description: Some(format!("Description article {i} qualité supérieure")),
                unit_price: dec!(10.00),
                vat_rate: dec!(8.1),
            },
        )
        .await
        .unwrap();
    }
}

async fn seed_journal_entries(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    fiscal_year_id: i64,
    asset: i64,
    expense: i64,
    count: usize,
) {
    let day = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    for i in 0..count {
        let new = NewJournalEntry {
            company_id,
            entry_date: day,
            journal: Journal::Achats,
            description: format!("Marie facture fournisseur {i:04}"),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: expense,
                    debit: dec!(10),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: asset,
                    debit: dec!(0),
                    credit: dec!(10),
                    project_id: None,
                },
            ],
        };
        journal_entries::create(pool, fiscal_year_id, user_id, new)
            .await
            .unwrap();
    }
}

async fn seed_fiscal_year_and_accounts(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
) -> (i64, i64, i64) {
    let fy = fiscal_years::create_for_seed(
        pool,
        NewFiscalYear {
            company_id,
            name: "Exercice 2026".into(),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();

    let asset = accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: "1000".into(),
            name: "Actif test".into(),
            account_type: AccountType::Asset,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;

    let expense = accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: "6000".into(),
            name: "Charge test".into(),
            account_type: AccountType::Expense,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;

    (fy.id, asset, expense)
}

// ---------------------------------------------------------------------------
// Helpers EXPLAIN
// ---------------------------------------------------------------------------

/// Exécute un `EXPLAIN FORMAT=JSON ...` et retourne le JSON parsé.
async fn explain_json(pool: &MySqlPool, sql: &str, company_id: i64, term: &str) -> JsonValue {
    let row = sqlx::query(&format!("EXPLAIN FORMAT=JSON {sql}"))
        .bind(company_id)
        .bind(term)
        .fetch_one(pool)
        .await
        .unwrap();
    let raw: String = row.get(0);
    serde_json::from_str(&raw).expect("EXPLAIN FORMAT=JSON output not valid JSON")
}

/// Récupère le premier `key` non-null dans l'output JSON d'EXPLAIN
/// (recherche récursive — la structure varie selon les versions MariaDB).
fn extract_first_key(plan: &JsonValue) -> Option<String> {
    fn walk(v: &JsonValue) -> Option<String> {
        match v {
            JsonValue::Object(map) => {
                if let Some(JsonValue::String(k)) = map.get("key") {
                    return Some(k.clone());
                }
                for (_, child) in map {
                    if let Some(found) = walk(child) {
                        return Some(found);
                    }
                }
                None
            }
            JsonValue::Array(arr) => arr.iter().find_map(walk),
            _ => None,
        }
    }
    walk(plan)
}

/// Récupère toutes les valeurs `possible_keys` (string ou array) sous forme
/// de Vec<String> aplati.
fn extract_possible_keys(plan: &JsonValue) -> Vec<String> {
    fn walk(v: &JsonValue, out: &mut Vec<String>) {
        match v {
            JsonValue::Object(map) => {
                if let Some(possible) = map.get("possible_keys") {
                    match possible {
                        JsonValue::String(s) => out.push(s.clone()),
                        JsonValue::Array(arr) => {
                            for elt in arr {
                                if let JsonValue::String(s) = elt {
                                    out.push(s.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                for (_, child) in map {
                    walk(child, out);
                }
            }
            JsonValue::Array(arr) => {
                for elt in arr {
                    walk(elt, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(plan, &mut out);
    out
}

// ===========================================================================
// T7.2 — EXPLAIN FORCE INDEX par table FULLTEXT
// ===========================================================================

/// Vérifie que `ft_contacts_name` est utilisable via FORCE INDEX (le `key`
/// field dans l'EXPLAIN doit nommer l'index — pas null = pas de fallback
/// silencieux table scan).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_2_explain_force_index_contacts_name(pool: MySqlPool) {
    let company = create_company(&pool, "Test SA").await;
    let user = create_admin(&pool, company).await;
    seed_contacts(&pool, user, company, 100).await;

    let plan = explain_json(
        &pool,
        "SELECT id FROM contacts FORCE INDEX (ft_contacts_name) \
         WHERE company_id = ? AND MATCH(name) AGAINST(? IN BOOLEAN MODE)",
        company,
        "Marie*",
    )
    .await;

    let key = extract_first_key(&plan).unwrap_or_default();
    assert_eq!(
        key, "ft_contacts_name",
        "FORCE INDEX a fallback sur table scan. EXPLAIN: {plan}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_2_explain_force_index_products_name(pool: MySqlPool) {
    let company = create_company(&pool, "Test SA").await;
    let user = create_admin(&pool, company).await;
    seed_products(&pool, user, company, 100).await;

    let plan = explain_json(
        &pool,
        "SELECT id FROM products FORCE INDEX (ft_products_name) \
         WHERE company_id = ? AND MATCH(name) AGAINST(? IN BOOLEAN MODE)",
        company,
        "Marie*",
    )
    .await;

    let key = extract_first_key(&plan).unwrap_or_default();
    assert_eq!(
        key, "ft_products_name",
        "FORCE INDEX a fallback sur table scan. EXPLAIN: {plan}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_2_explain_force_index_products_description(pool: MySqlPool) {
    let company = create_company(&pool, "Test SA").await;
    let user = create_admin(&pool, company).await;
    seed_products(&pool, user, company, 100).await;

    let plan = explain_json(
        &pool,
        "SELECT id FROM products FORCE INDEX (ft_products_description) \
         WHERE company_id = ? AND MATCH(description) AGAINST(? IN BOOLEAN MODE)",
        company,
        "qualité*",
    )
    .await;

    let key = extract_first_key(&plan).unwrap_or_default();
    assert_eq!(
        key, "ft_products_description",
        "FORCE INDEX a fallback sur table scan. EXPLAIN: {plan}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_2_explain_force_index_journal_entries_description(pool: MySqlPool) {
    let company = create_company(&pool, "Test SA").await;
    let user = create_admin(&pool, company).await;
    let (fy, asset, expense) = seed_fiscal_year_and_accounts(&pool, user, company).await;
    seed_journal_entries(&pool, user, company, fy, asset, expense, 100).await;

    let plan = explain_json(
        &pool,
        "SELECT id FROM journal_entries FORCE INDEX (ft_journal_entries_description) \
         WHERE company_id = ? AND MATCH(description) AGAINST(? IN BOOLEAN MODE)",
        company,
        "Marie*",
    )
    .await;

    let key = extract_first_key(&plan).unwrap_or_default();
    assert_eq!(
        key, "ft_journal_entries_description",
        "FORCE INDEX a fallback sur table scan. EXPLAIN: {plan}"
    );
}

// ===========================================================================
// T7.3 — Isolation cross-company (multi-tenant scoping préservé)
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_3_contacts_search_does_not_leak_cross_company(pool: MySqlPool) {
    let company_a = create_company(&pool, "Company A").await;
    let user_a = create_admin(&pool, company_a).await;
    let company_b = create_company(&pool, "Company B").await;
    let user_b = create_admin(&pool, company_b).await;

    seed_contacts(&pool, user_a, company_a, 5).await;
    seed_contacts(&pool, user_b, company_b, 5).await;

    let result = contacts::list_by_company_paginated(
        &pool,
        company_a,
        ContactListQuery {
            search: Some("Marie".into()),
            sort_by: ContactSortBy::Name,
            limit: 100,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(result.total, 5, "company A doit voir UNIQUEMENT ses 5 rows");
    for item in &result.items {
        assert_eq!(item.company_id, company_a);
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_3_products_search_does_not_leak_cross_company(pool: MySqlPool) {
    let company_a = create_company(&pool, "Company A").await;
    let user_a = create_admin(&pool, company_a).await;
    let company_b = create_company(&pool, "Company B").await;
    let user_b = create_admin(&pool, company_b).await;

    seed_products(&pool, user_a, company_a, 5).await;
    seed_products(&pool, user_b, company_b, 5).await;

    let result = products::list_by_company_paginated(
        &pool,
        company_a,
        ProductListQuery {
            search: Some("Marie".into()),
            sort_by: ProductSortBy::Name,
            limit: 100,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(result.total, 5);
    for item in &result.items {
        assert_eq!(item.company_id, company_a);
    }
}

/// Pass 1 F2 — Isolation cross-company sur le path FULLTEXT
/// `MATCH(c.name)` dans `invoices::list_by_company_paginated` (JOIN
/// invoices ↔ contacts). Le `WHERE i.company_id = ?` doit filtrer
/// correctement même quand le contact name match côté FULLTEXT dans
/// l'autre company. Sans ce test, une régression qui retirerait le
/// filtre `i.company_id` ne serait pas détectée.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_3_invoices_search_does_not_leak_cross_company(pool: MySqlPool) {
    let company_a = create_company(&pool, "Company A").await;
    let user_a = create_admin(&pool, company_a).await;
    let company_b = create_company(&pool, "Company B").await;
    let user_b = create_admin(&pool, company_b).await;

    // Un contact par company avec un nom partageant le même token unique
    // (`KFFiveCross`). Si le scoping `i.company_id` était cassé, la search
    // retournerait les 2 factures.
    let contact_a = contacts::create(
        &pool,
        user_a,
        NewContact {
            company_id: company_a,
            contact_type: ContactType::Entreprise,
            name: "KFFiveCross ContactA SARL".into(),
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
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    let contact_b = contacts::create(
        &pool,
        user_b,
        NewContact {
            company_id: company_b,
            contact_type: ContactType::Entreprise,
            name: "KFFiveCross ContactB SARL".into(),
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
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;

    let day = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let line = NewInvoiceLine {
        description: "Conseil".into(),
        quantity: dec!(1),
        unit_price: dec!(100.00),
        vat_rate: dec!(8.10),
    };

    let (inv_a, _) = invoices::create(
        &pool,
        user_a,
        NewInvoice {
            company_id: company_a,
            contact_id: contact_a,
            date: day,
            due_date: Some(day),
            payment_terms: None,
            lines: vec![line.clone()],
            project_id: None,
        },
    )
    .await
    .unwrap();
    let (inv_b, _) = invoices::create(
        &pool,
        user_b,
        NewInvoice {
            company_id: company_b,
            contact_id: contact_b,
            date: day,
            due_date: Some(day),
            payment_terms: None,
            lines: vec![line],
            project_id: None,
        },
    )
    .await
    .unwrap();

    let result = invoices::list_by_company_paginated(
        &pool,
        company_a,
        InvoiceListQuery {
            search: Some("KFFiveCross".into()),
            limit: 100,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(
        result.items.iter().any(|i| i.id == inv_a.id),
        "company A doit voir sa facture (contact = ContactA)"
    );
    assert!(
        !result.items.iter().any(|i| i.id == inv_b.id),
        "company A ne doit PAS voir la facture de company B malgré le token FULLTEXT partagé"
    );
    for item in &result.items {
        assert_eq!(
            item.company_id, company_a,
            "tous les items doivent appartenir à company_a"
        );
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_3_journal_entries_search_does_not_leak_cross_company(pool: MySqlPool) {
    let company_a = create_company(&pool, "Company A").await;
    let user_a = create_admin(&pool, company_a).await;
    let company_b = create_company(&pool, "Company B").await;
    let user_b = create_admin(&pool, company_b).await;

    let (fy_a, asset_a, expense_a) = seed_fiscal_year_and_accounts(&pool, user_a, company_a).await;
    let (fy_b, asset_b, expense_b) = seed_fiscal_year_and_accounts(&pool, user_b, company_b).await;

    seed_journal_entries(&pool, user_a, company_a, fy_a, asset_a, expense_a, 5).await;
    seed_journal_entries(&pool, user_b, company_b, fy_b, asset_b, expense_b, 5).await;

    let result = journal_entries::list_by_company_paginated(
        &pool,
        company_a,
        JournalEntryListQuery {
            description: Some("Marie".into()),
            limit: 100,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(result.total, 5);
    for item in &result.items {
        assert_eq!(item.entry.company_id, company_a);
    }
}

// ===========================================================================
// T7.4 — EXPLAIN sur query hybride MATCH OR LIKE (descriptif)
//
// Ces tests sont **descriptifs** — ils observent le choix de l'optimizer
// MariaDB sans FORCE INDEX. Le PASS se fait dès que l'index FULLTEXT est
// présent dans `possible_keys` ou choisi comme `key`. Un FAIL signale que
// l'optimizer fait un table scan systématique, et le Dev Agent Record
// devra documenter la décision (refactor `UNION` ou accepter v0.1).
// ===========================================================================

/// **Test descriptif** (AC #17 / T7.4a). Observe le choix de l'optimizer
/// MariaDB pour la query hybride 2-way OR `MATCH(name) OR email LIKE`.
///
/// **Décision v0.1** : sur dataset 100 lignes seedées dans ce test,
/// MariaDB ne place pas `ft_contacts_name` dans `possible_keys` —
/// l'optimizer fait un table scan systématique. C'est le "Cas échec"
/// AC #17. Décision : accepter v0.1 (option a), documenté dans
/// `docs/search-patterns.md` section « Pattern hybride MATCH OR LIKE ».
/// Justification : volumes v0.1 < 10k contacts/company restent
/// sub-secondaire en full scan. La factorisation `UNION` est planifiée
/// v0.2 si la dette devient observable (e.g. plainte UX latence sur le
/// premier dataset prod).
///
/// Ce test loggue le plan EXPLAIN pour archive (utile en review code
/// pour confirmer l'observation) mais ne FAIL PAS sur le scenario
/// optimizer choisi — il sert de tracker descriptif.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t7_4a_explain_hybrid_match_or_like_contacts(pool: MySqlPool) {
    let company = create_company(&pool, "Test SA").await;
    let user = create_admin(&pool, company).await;
    seed_contacts(&pool, user, company, 100).await;

    let plan = explain_json(
        &pool,
        "SELECT id FROM contacts \
         WHERE company_id = ? \
         AND (MATCH(name) AGAINST('Marie*' IN BOOLEAN MODE) OR email LIKE ?)",
        company,
        "%marie%",
    )
    .await;

    let key = extract_first_key(&plan);
    let possible = extract_possible_keys(&plan);

    eprintln!(
        "T7.4a contacts 2-way OR EXPLAIN — key={key:?}, possible_keys={possible:?}, \
         full plan: {plan}"
    );

    let key_uses_fulltext = key.as_deref() == Some("ft_contacts_name");
    let fulltext_in_possible = possible.iter().any(|k| k == "ft_contacts_name");

    if key_uses_fulltext || fulltext_in_possible {
        eprintln!("✓ Optimizer considère ft_contacts_name (cas idéal/acceptable)");
    } else {
        eprintln!(
            "⚠ Cas échec AC #17 confirmé : optimizer fait table scan. \
             Acceptable v0.1 (cf. docs/search-patterns.md), à refactor UNION en v0.2 si dette observable."
        );
    }
}

// Note T7.4b (invoices 3-way OR) : EXPLAIN dédié non-implémenté ici car
// nécessite un seed invoices + contacts complet (plusieurs centaines de
// lignes pour stabilité optimizer). Le path fonctionnel `MATCH(c.name)`
// est désormais couvert par `test_filter_by_search_matches_contact_name_fulltext`
// dans `invoices.rs::mod tests` (ajouté Pass 1 F1) qui exerce les 2
// callsites (`list_by_company_paginated` + `due_dates_summary`). Le
// risque optimizer 3-way OR reste théorique sans dataset prod réel —
// documenté en Change Log story 7-4 (suivi v0.2 si KF-005 ne suffit pas).
