//! Story 46-1, partagé depuis la Story 46-3 — le **jeu PME** des rapports par
//! groupes : plan PME complet (`pme.json`), un compte en dollars sous
//! `10 Actifs circulants`, des écritures sur deux exercices (2025 et 2026).
//!
//! Inclus par `#[path]` dans `reports_e2e.rs` (fixtures JSON de la 46-1) et
//! `reports_export_e2e.rs` (fixtures des exports, 46-3) : chacun crée la
//! société, l'utilisateur et les deux exercices avec ses propres aides, puis
//! appelle [`peupler`]. Base `#[sqlx::test]` neuve : les identifiants sont
//! stables d'un run à l'autre. Données **synthétiques**.

#![allow(dead_code)]

use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::{accounts, journal_entries};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Jeu de données déterministe de la Story 46-1 : plan PME complet
/// (`pme.json`), un compte en dollars sous `10 Actifs circulants`, deux
/// exercices (2025 et 2026). Base `#[sqlx::test]` neuve : les identifiants
/// sont stables d'un run à l'autre, ce qui permet la fixture de l'AC-8.
pub struct JeuPme {
    pub jwt: String,
    pub fy_2025: i64,
    pub fy_2026: i64,
    /// Identifiant de chaque compte, par numéro.
    pub comptes: std::collections::HashMap<String, i64>,
}

impl JeuPme {
    /// Identifiant d'un compte du jeu, par son numéro.
    pub fn id(&self, numero: &str) -> i64 {
        *self
            .comptes
            .get(numero)
            .unwrap_or_else(|| panic!("compte {numero} absent du jeu PME"))
    }
}

/// Une écriture à deux lignes du jeu PME ; `usd`, s'il est donné, est le
/// montant en dollars de la ligne au débit (cours = CHF / USD).
async fn ecriture_pme(
    pool: &MySqlPool,
    jeu: &JeuPme,
    company_id: i64,
    user_id: i64,
    fy: i64,
    date: NaiveDate,
    debit: &str,
    credit: &str,
    montant: Decimal,
    usd: Option<Decimal>,
) {
    let foreign = usd.map(|m| kesh_db::entities::ForeignAmount {
        declared_currency: None,
        amount: m,
        rate: montant / m,
        source: Some(kesh_db::entities::RateProvenance::Manual),
        rate_derived: false,
    });
    journal_entries::create(
        pool,
        fy,
        Acteur::utilisateur(user_id),
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal: Journal::OD,
            description: format!("{debit} / {credit}"),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: jeu.id(debit),
                    debit: montant,
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign,
                },
                NewJournalEntryLine {
                    account_id: jeu.id(credit),
                    debit: Decimal::ZERO,
                    credit: montant,
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .unwrap();
}

/// Peuple la société `company_id` du jeu PME : le plan, le compte en dollars,
/// la multi-devise de la société et des exercices, les écritures.
pub async fn peupler(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    fy_2025: i64,
    fy_2026: i64,
    jwt: String,
) -> JeuPme {
    let d = |a, m, j| NaiveDate::from_ymd_opt(a, m, j).unwrap();
    let plan = kesh_core::chart_of_accounts::load_chart("pme").expect("plan PME");
    let crees = accounts::bulk_create_from_chart(pool, company_id, &plan, "fr")
        .await
        .unwrap();
    let mut comptes: std::collections::HashMap<String, i64> =
        crees.iter().map(|a| (a.number.clone(), a.id)).collect();

    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE company_id = ?")
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
    let usd = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, parent_id, currency) \
         VALUES (?, '1025', 'Banque USD', 'Asset', ?, 'USD')",
    )
    .bind(company_id)
    .bind(comptes["10"])
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    comptes.insert("1025".into(), usd);

    let jeu = JeuPme {
        jwt,
        fy_2025,
        fy_2026,
        comptes,
    };
    let e = |fy, date, debit, credit, montant, usd| {
        ecriture_pme(
            pool, &jeu, company_id, user_id, fy, date, debit, credit, montant, usd,
        )
    };
    // Exercice 2025.
    e(fy_2025, d(2025, 1, 5), "1020", "2800", dec!(20000), None).await;
    e(fy_2025, d(2025, 2, 10), "1100", "3000", dec!(8000), None).await;
    e(fy_2025, d(2025, 3, 3), "4000", "2000", dec!(3000), None).await;
    e(fy_2025, d(2025, 4, 1), "6000", "1020", dec!(1200), None).await;
    e(fy_2025, d(2025, 5, 2), "1020", "6950", dec!(50), None).await;
    e(fy_2025, d(2025, 6, 7), "7010", "1020", dec!(100), None).await;
    e(fy_2025, d(2025, 6, 8), "1020", "7000", dec!(300), None).await;
    e(
        fy_2025,
        d(2025, 7, 1),
        "1025",
        "1020",
        dec!(912),
        Some(dec!(1000)),
    )
    .await;
    e(fy_2025, d(2025, 8, 1), "1520", "2400", dec!(4000), None).await;
    // Exercice 2026.
    e(fy_2026, d(2026, 1, 20), "1020", "1100", dec!(5000), None).await;
    e(fy_2026, d(2026, 2, 25), "5000", "1020", dec!(2500), None).await;
    e(fy_2026, d(2026, 3, 1), "1020", "8000", dec!(400), None).await;
    e(fy_2026, d(2026, 3, 2), "8010", "1020", dec!(150), None).await;
    e(
        fy_2026,
        d(2026, 4, 9),
        "1025",
        "1020",
        dec!(455),
        Some(dec!(500)),
    )
    .await;
    jeu
}
