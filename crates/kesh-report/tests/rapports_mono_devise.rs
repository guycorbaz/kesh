//! Story 37-4, AC-5, AC-6, AC-8 — **non-régression mono-devise** des rapports.
//!
//! Les exports CSV d'un rapport sans compte en devise doivent rester **identiques
//! octet pour octet** à ceux d'avant la story, le JSON aussi, et le PDF garder
//! **les mêmes lignes de texte** (pas d'octet pour octet : le PDF porte un
//! horodatage et un identifiant de document). Les fixtures de
//! `tests/fixtures/mono_devise/` ont été **enregistrées depuis le commit de base**
//! de la story (`946aa86a`), avant toute modification des générateurs : elles
//! sont la sortie d'avant, non une sortie recopiée d'après.
//!
//! Deux scénarios, mêmes écritures :
//! - `chf` : une société en devise principale seulement ;
//! - `multi_sans_devise` : la même, passée multi-devises (société et
//!   exercices), avec un compte USD **archivé et sans ligne**, donc présent dans
//!   le plan comptable mais dans aucun rapport — la règle de colonnes du CSV
//!   regarde le **rapport**, non la société.
//!
//! Réenregistrer (seulement sur décision, jamais pour faire passer un test) :
//! `KESH_FIXTURES_ENREGISTRER=1 cargo nextest run -E 'binary(rapports_mono_devise)'`.

use std::path::PathBuf;

use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::csv::{
    render_balance_sheet_csv, render_general_ledger_csv, render_income_statement_csv,
    render_journal_report_csv, render_trial_balance_csv,
};
use kesh_report::general_ledger::{LedgerOptions, LedgerPeriod};
use kesh_report::pdf::{
    PdfContext, render_balance_sheet_pdf, render_general_ledger_pdf, render_income_statement_pdf,
    render_journal_report_pdf, render_trial_balance_pdf,
};
use kesh_report::period::ReportPeriod;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

/// Écriture équilibrée à deux lignes, en devise principale.
#[allow(clippy::too_many_arguments)]
async fn post(
    pool: &MySqlPool,
    s: &SeededCompany,
    fy: i64,
    date: NaiveDate,
    journal: Journal,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
    label: &str,
) {
    let ligne = |account_id, debit, credit| NewJournalEntryLine {
        account_id,
        debit,
        credit,
        project_id: None,
        foreign: None,
    };
    journal_entries::create(
        pool,
        fy,
        Acteur::utilisateur(s.admin_user_id),
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: date,
            journal,
            description: label.into(),
            project_id: None,
            lines: vec![
                ligne(debit_account, amount, Decimal::ZERO),
                ligne(credit_account, Decimal::ZERO, amount),
            ],
        },
    )
    .await
    .expect("écriture postée");
}

/// Le jeu commun : deux exercices (2025, 2026), des ventes et des charges sur
/// les deux, un compte archivé qui porte encore un solde.
async fn jeu(pool: &MySqlPool) -> (SeededCompany, i64, i64) {
    let s = seed_accounting_company(pool).await.expect("seed");
    sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31', name = 'FY 2025' WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(pool)
        .await
        .expect("exercice seedé rétréci");
    let fy2026: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, 'FY 2026', '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(s.company_id)
    .fetch_one(pool)
    .await
    .expect("exercice 2026");
    let fy2025 = s.fiscal_year_id;

    let caisse = s.accounts["1000"];
    let banque = s.accounts["1100"];
    let capital = s.accounts["2000"];
    let ventes = s.accounts["3000"];
    let charges = s.accounts["4000"];

    post(
        pool,
        &s,
        fy2025,
        ymd(2025, 1, 10),
        Journal::OD,
        banque,
        capital,
        dec!(10000),
        "Apport",
    )
    .await;
    post(
        pool,
        &s,
        fy2025,
        ymd(2025, 8, 3),
        Journal::Ventes,
        caisse,
        ventes,
        dec!(1234.55),
        "Vente août",
    )
    .await;
    post(
        pool,
        &s,
        fy2025,
        ymd(2025, 11, 20),
        Journal::Achats,
        charges,
        banque,
        dec!(420.10),
        "Loyer",
    )
    .await;
    post(
        pool,
        &s,
        fy2026,
        ymd(2026, 2, 14),
        Journal::Ventes,
        caisse,
        ventes,
        dec!(990),
        "Vente février",
    )
    .await;
    post(
        pool,
        &s,
        fy2026,
        ymd(2026, 3, 1),
        Journal::Banque,
        banque,
        caisse,
        dec!(1500),
        "Versement",
    )
    .await;
    post(
        pool,
        &s,
        fy2026,
        ymd(2026, 5, 31),
        Journal::Achats,
        charges,
        caisse,
        dec!(77.35),
        "Fournitures",
    )
    .await;

    // Un compte archivé à solde non nul : la balance et le grand livre le rendent.
    let archive: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '1010', 'Ancienne caisse', 'Asset') RETURNING id",
    )
    .bind(s.company_id)
    .fetch_one(pool)
    .await
    .expect("compte 1010");
    post(
        pool,
        &s,
        fy2025,
        ymd(2025, 6, 1),
        Journal::Caisse,
        archive,
        caisse,
        dec!(50),
        "Fonds de caisse",
    )
    .await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(archive)
        .execute(pool)
        .await
        .expect("archivage");

    (s, fy2025, fy2026)
}

/// Passe la société en multi-devises, avec un compte USD archivé et sans ligne.
async fn multi_devises_sans_compte_en_devise(pool: &MySqlPool, s: &SeededCompany) {
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .expect("société multi-devises");
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE company_id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .expect("exercices multi-devises");
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency, active) \
         VALUES (?, '1025', 'Banque USD (fermée)', 'Asset', 'USD', FALSE)",
    )
    .bind(s.company_id)
    .execute(pool)
    .await
    .expect("compte USD archivé");
}

/// Les lignes de texte d'un PDF non compressé (profil de test) : chaque
/// opérateur `<hex> Tj` de printpdf, décodé en WinAnsi (approximé par Latin-1,
/// ce qui suffit à comparer deux rendus).
fn pdf_text_lines(bytes: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(bytes);
    let mut out = Vec::new();
    let mut rest: &str = &s;
    while let Some(end) = rest.find("> Tj") {
        let before = &rest[..end];
        if let Some(start) = before.rfind('<') {
            let hex = &before[start + 1..];
            let decoded: String = (0..hex.len() / 2)
                .filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
                .map(char::from)
                .collect();
            out.push(decoded);
        }
        rest = &rest[end + 4..];
    }
    out
}

/// Les sorties de tous les rapports pour un scénario : `(nom de fichier, octets)`.
async fn sorties(pool: &MySqlPool, s: &SeededCompany, fy2026: i64) -> Vec<(String, Vec<u8>)> {
    let ctx = PdfContext::fr_ch_default("CI Test Company");
    let periode = ReportPeriod {
        fiscal_year_id: fy2026,
        start_date: ymd(2026, 1, 1),
        end_date: ymd(2026, 12, 31),
    };
    let gl_period = LedgerPeriod::new(ymd(2025, 7, 1), ymd(2026, 6, 30)).expect("période");
    let gl = kesh_report::general_ledger::generate(
        pool,
        s.company_id,
        &gl_period,
        &LedgerOptions::default(),
    )
    .await
    .expect("grand livre");
    let tb = kesh_report::generate_trial_balance(pool, s.company_id, &periode)
        .await
        .expect("balance");
    let bs = kesh_report::generate_balance_sheet(pool, s.company_id, &periode)
        .await
        .expect("bilan");
    let is_ = kesh_report::generate_income_statement(pool, s.company_id, &periode)
        .await
        .expect("compte de résultat");
    let jr = kesh_report::journal_report::generate(pool, s.company_id, &periode, None)
        .await
        .expect("journal");

    let mut out = Vec::new();
    let mut csv = |nom: &str, f: &dyn Fn(&mut Vec<u8>)| {
        let mut buf = Vec::new();
        f(&mut buf);
        out.push((format!("{nom}.csv"), buf));
    };
    csv("grand_livre", &|b| {
        render_general_ledger_csv(&gl, b).unwrap()
    });
    csv("balance", &|b| render_trial_balance_csv(&tb, b).unwrap());
    csv("bilan", &|b| render_balance_sheet_csv(&bs, b).unwrap());
    csv("resultat", &|b| {
        render_income_statement_csv(&is_, b).unwrap()
    });
    csv("journal", &|b| render_journal_report_csv(&jr, b).unwrap());

    let json = |v: serde_json::Value| serde_json::to_vec_pretty(&v).unwrap();
    out.push((
        "grand_livre.json".into(),
        json(serde_json::to_value(&gl).unwrap()),
    ));
    out.push((
        "balance.json".into(),
        json(serde_json::to_value(&tb).unwrap()),
    ));
    out.push((
        "bilan.json".into(),
        json(serde_json::to_value(&bs).unwrap()),
    ));
    out.push((
        "resultat.json".into(),
        json(serde_json::to_value(&is_).unwrap()),
    ));
    out.push((
        "journal.json".into(),
        json(serde_json::to_value(&jr).unwrap()),
    ));

    let texte = |b: Vec<u8>| (pdf_text_lines(&b).join("\n") + "\n").into_bytes();
    out.push((
        "grand_livre.pdf.txt".into(),
        texte(render_general_ledger_pdf(&gl, &ctx).unwrap()),
    ));
    out.push((
        "balance.pdf.txt".into(),
        texte(render_trial_balance_pdf(&tb, &ctx).unwrap()),
    ));
    out.push((
        "bilan.pdf.txt".into(),
        texte(render_balance_sheet_pdf(&bs, &ctx).unwrap()),
    ));
    out.push((
        "resultat.pdf.txt".into(),
        texte(render_income_statement_pdf(&is_, &ctx).unwrap()),
    ));
    out.push((
        "journal.pdf.txt".into(),
        texte(render_journal_report_pdf(&jr, &ctx).unwrap()),
    ));
    out
}

/// Compare chaque sortie à sa fixture (ou l'enregistre, sur demande explicite).
fn comparer(scenario: &str, sorties: Vec<(String, Vec<u8>)>) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/mono_devise")
        .join(scenario);
    let enregistrer = std::env::var("KESH_FIXTURES_ENREGISTRER").is_ok_and(|v| v == "1");
    if enregistrer {
        std::fs::create_dir_all(&dir).unwrap();
    }
    for (nom, octets) in sorties {
        let chemin = dir.join(&nom);
        if enregistrer {
            std::fs::write(&chemin, &octets).unwrap();
            continue;
        }
        let attendu = std::fs::read(&chemin)
            .unwrap_or_else(|e| panic!("fixture absente {}: {e}", chemin.display()));
        assert!(
            attendu == octets,
            "{scenario}/{nom} : sortie différente de la fixture du commit de base\n--- attendu\n{}\n--- obtenu\n{}",
            String::from_utf8_lossy(&attendu),
            String::from_utf8_lossy(&octets)
        );
    }
}

/// Société en devise principale seulement : sorties identiques à celles d'avant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rapports_chf_identiques_au_commit_de_base(pool: MySqlPool) {
    let (s, _fy2025, fy2026) = jeu(&pool).await;
    let out = sorties(&pool, &s, fy2026).await;
    comparer("chf", out);
}

/// Société multi-devises sans compte en devise **dans le rapport** : sorties
/// identiques à celles d'avant (aucune colonne en devise, AC-5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rapports_multi_devises_sans_compte_en_devise_identiques(pool: MySqlPool) {
    let (s, _fy2025, fy2026) = jeu(&pool).await;
    multi_devises_sans_compte_en_devise(&pool, &s).await;
    let out = sorties(&pool, &s, fy2026).await;
    comparer("multi_sans_devise", out);
}
