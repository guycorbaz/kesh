//! Story 37-4, AC-7 — les rapports qui restent **en devise principale** :
//! accueil, projets (dépenses et rendement), TVA (impôt préalable) et balance
//! âgée. Une écriture en USD y est comptée au montant CHF de ses lignes, jamais
//! au montant en devise ; aucun de ces rapports ne porte de champ en devise.
//!
//! Jeu : société multi-devises, compte 1020 Banque USD et 3200 Ventes USD ;
//! cours 0.91 ; les montants en devise (100, 50, 10) diffèrent à dessein des
//! montants CHF (91.00, 45.50, 9.10).

use chrono::NaiveDate;
use kesh_db::entities::{
    Acteur, ForeignAmount, Journal, NewJournalEntry, NewJournalEntryLine, RateProvenance,
};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, chf_line, seed_accounting_company};
use kesh_report::period::ReportPeriod;
use kesh_report::project_report::{
    ProjectPeriodMode, generate_project_expenses, generate_project_return, resolve_scope,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn usd(amount: Decimal) -> Option<ForeignAmount> {
    Some(ForeignAmount {
        declared_currency: None,
        amount,
        rate: dec!(0.91),
        source: Some(RateProvenance::Manual),
        rate_derived: false,
    })
}

fn ligne(
    account: i64,
    debit: Decimal,
    credit: Decimal,
    project_id: Option<i64>,
    foreign: Option<ForeignAmount>,
) -> NewJournalEntryLine {
    NewJournalEntryLine {
        project_id,
        foreign,
        ..chf_line(account, debit, credit)
    }
}

async fn poser(
    pool: &MySqlPool,
    s: &SeededCompany,
    date: NaiveDate,
    lines: Vec<NewJournalEntryLine>,
) {
    journal_entries::create(
        pool,
        s.fiscal_year_id,
        Acteur::utilisateur(s.admin_user_id),
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "Écriture en USD".into(),
            project_id: None,
            lines,
        },
    )
    .await
    .expect("écriture");
}

struct Jeu {
    s: SeededCompany,
    projet: i64,
}

async fn jeu(pool: &MySqlPool) -> Jeu {
    let s = seed_accounting_company(pool).await.unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
    let compte = |numero: &'static str, typ: &'static str| async move {
        sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, currency) \
             VALUES (?, ?, ?, ?, 'USD')",
        )
        .bind(s.company_id)
        .bind(numero)
        .bind(format!("Compte {numero}"))
        .bind(typ)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    };
    let banque_usd = compte("1020", "Asset").await;
    let ventes_usd = compte("3200", "Revenue").await;
    let projet: i64 = sqlx::query_scalar(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, NULL, 'P1', 'Projet P1', FALSE) RETURNING id",
    )
    .bind(s.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let (charges, caisse) = (s.accounts["4000"], s.accounts["1000"]);
    let z = Decimal::ZERO;

    // Vente USD 100 = CHF 91.00, ventes imputées au projet.
    poser(
        pool,
        &s,
        ymd(2026, 3, 15),
        vec![
            ligne(banque_usd, dec!(91.00), z, None, usd(dec!(100))),
            ligne(ventes_usd, z, dec!(91.00), Some(projet), usd(dec!(100))),
        ],
    )
    .await;
    // Charge du projet payée USD 50 = CHF 45.50.
    poser(
        pool,
        &s,
        ymd(2026, 3, 20),
        vec![
            ligne(charges, dec!(45.50), z, Some(projet), None),
            ligne(banque_usd, z, dec!(45.50), None, usd(dec!(50))),
        ],
    )
    .await;
    // Impôt préalable (1000, compte récupérable de la société) payé USD 10 = CHF 9.10.
    poser(
        pool,
        &s,
        ymd(2026, 3, 25),
        vec![
            ligne(caisse, dec!(9.10), z, None, None),
            ligne(banque_usd, z, dec!(9.10), None, usd(dec!(10))),
        ],
    )
    .await;
    Jeu { s, projet }
}

fn periode(s: &SeededCompany) -> ReportPeriod {
    ReportPeriod {
        fiscal_year_id: s.fiscal_year_id,
        start_date: ymd(2026, 1, 1),
        end_date: ymd(2026, 12, 31),
    }
}

/// Tous les montants décimaux écrits en chaîne dans un JSON (les montants
/// sérialisent en chaîne), à n'importe quelle profondeur.
fn montants_decimaux(v: &serde_json::Value, out: &mut Vec<Decimal>) {
    match v {
        serde_json::Value::String(t) => {
            if let Ok(d) = t.parse::<Decimal>() {
                out.push(d);
            }
        }
        serde_json::Value::Array(a) => a.iter().for_each(|x| montants_decimaux(x, out)),
        serde_json::Value::Object(o) => o.values().for_each(|x| montants_decimaux(x, out)),
        _ => {}
    }
}

/// Accueil : les produits de l'exercice sont les CHF de la ligne (91.00, non
/// USD 100).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accueil_en_chf(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let h = kesh_report::generate_homepage_summary(&pool, j.s.company_id, ymd(2026, 3, 31))
        .await
        .unwrap();
    assert_eq!(h.total_revenues, dec!(91.00));
    assert_eq!(h.total_expenses, dec!(45.50));
    let json = serde_json::to_string(&h).unwrap();
    assert!(!json.contains("urrency"), "aucun champ en devise : {json}");
}

/// Projets : dépenses et rendement au CHF des lignes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn projets_en_chf(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let scope = resolve_scope(&pool, j.s.company_id, j.projet)
        .await
        .unwrap();
    let mode = ProjectPeriodMode::FiscalYear {
        period: periode(&j.s),
    };
    let dep = generate_project_expenses(&pool, j.s.company_id, &scope, &mode)
        .await
        .unwrap();
    assert_eq!(dep.grand_total, dec!(45.50));
    let ren = generate_project_return(&pool, j.s.company_id, &scope, &mode)
        .await
        .unwrap();
    let valeur = serde_json::to_value(&ren).unwrap();
    let json = valeur.to_string();
    // Un montant égal au CHF de la ligne (91), jamais au montant en devise
    // (100) : comparé en nombre, non cherché dans le texte (revue de code P2).
    let mut montants = Vec::new();
    montants_decimaux(&valeur, &mut montants);
    assert!(montants.contains(&dec!(91)), "{json}");
    assert!(!montants.contains(&dec!(100)), "{json}");
    assert!(!json.contains("urrency"), "{json}");
}

/// TVA : l'impôt préalable est le solde CHF du compte récupérable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tva_en_chf(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let v = kesh_report::vat_report::generate(&pool, j.s.company_id, &periode(&j.s))
        .await
        .unwrap();
    assert_eq!(v.total_vat_recoverable, dec!(9.10));
    assert!(!serde_json::to_string(&v).unwrap().contains("urrency"));
}

/// Balance âgée : elle lit les factures (hors périmètre de la devise, D7) ;
/// les écritures en USD ne l'atteignent pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn balance_agee_inchangee(pool: MySqlPool) {
    let j = jeu(&pool).await;
    let a = kesh_report::aged_receivables::generate(&pool, j.s.company_id, ymd(2026, 12, 31))
        .await
        .unwrap();
    let json = serde_json::to_string(&a).unwrap();
    assert!(
        !json.contains("urrency") && !json.contains("\"91"),
        "{json}"
    );
}
