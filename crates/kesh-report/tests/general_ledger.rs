//! Tests d'intégration du grand livre (Story 24-1).
//!
//! ⚠️ **Les tests de CONCORDANCE sont la raison d'être de ce fichier.** Ils
//! appellent réellement `generate_balance_sheet`, `generate_trial_balance` et
//! `generate_income_statement` sur le même jeu de données, parce que le défaut
//! que le grand livre peut porter est **muet** : si le solde d'ouverture est pris
//! de travers, `closing = opening + mouvements` reste vrai, les totaux
//! s'additionnent, et le rapport est intérieurement cohérent tout en étant
//! extérieurement faux. Relire le SQL ne l'attraperait pas ; comparer aux
//! rapports existants, si.
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test).

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_report::general_ledger::{
    FiscalYearBreak, LedgerOptions, LedgerPeriod, LedgerSection, generate,
};
use kesh_report::period::ReportPeriod;
use kesh_report::{generate_balance_sheet, generate_income_statement, generate_trial_balance};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

/// Poste une écriture équilibrée à deux lignes, dans l'exercice seedé.
async fn post(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    date: NaiveDate,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
    label: &str,
) {
    post_fy(
        pool,
        seeded,
        seeded.fiscal_year_id,
        date,
        debit_account,
        credit_account,
        amount,
        label,
    )
    .await;
}

/// Poste une écriture équilibrée à deux lignes dans l'exercice `fiscal_year_id`.
#[allow(clippy::too_many_arguments)]
async fn post_fy(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    fiscal_year_id: i64,
    date: NaiveDate,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
    label: &str,
) {
    journal_entries::create(
        pool,
        fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: label.into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture postée");
}

fn full_year() -> LedgerPeriod {
    LedgerPeriod::new(ymd(2026, 1, 1), ymd(2026, 12, 31)).expect("période valide")
}

/// Deux exercices consécutifs : l'exercice seedé, rétréci pour finir le
/// 2025-12-31, puis l'exercice 2026. Rend `(fy2025, fy2026)`.
///
/// ⚠️ La fixture seede UN exercice de ONZE ANS (2020-2030, « large pour
/// tolérer la dérive d'horloge CI »). Il faut donc le RÉTRÉCIR avant de créer
/// le suivant, sinon les deux se chevauchent et un mouvement de 2025 tombe
/// dans le même exercice qu'une période de 2026 — auquel cas son report
/// devient correct et le test ne prouve plus rien.
async fn deux_exercices(pool: &MySqlPool, seeded: &SeededCompany) -> (i64, i64) {
    sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(pool)
        .await
        .expect("exercice seedé rétréci à 2025");

    let fy2026: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, 'FY 2026', '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .expect("exercice 2026 créé");

    (seeded.fiscal_year_id, fy2026)
}

/// La section d'un compte, lue avec `offset` et `limit`.
async fn section_paginee(
    pool: &MySqlPool,
    company_id: i64,
    period: &LedgerPeriod,
    account_id: i64,
    offset: i64,
    limit: Option<i64>,
) -> LedgerSection {
    let ledger = generate(
        pool,
        company_id,
        period,
        &LedgerOptions {
            account_ids: Some(vec![account_id]),
            offset,
            limit,
            ..Default::default()
        },
    )
    .await
    .expect("grand livre");
    ledger
        .sections
        .into_iter()
        .find(|s| s.account_id == account_id)
        .expect("un compte explicitement demandé est toujours rendu")
}

/// Les soldes progressifs d'une page.
fn soldes(s: &LedgerSection) -> Vec<Decimal> {
    s.lines.iter().map(|l| l.running_balance).collect()
}

/// Une rupture d'exercice, réduite à ce qui se compare.
fn rupture(b: &FiscalYearBreak) -> (NaiveDate, i64, Decimal) {
    (b.date, b.closing_fiscal_year_id, b.closing_balance)
}

/// Lit un compte page après page, `taille` lignes à la fois, et rend la suite
/// des `(line_id, solde progressif)` et des ruptures, toutes pages confondues.
async fn lecture_par_pages(
    pool: &MySqlPool,
    company_id: i64,
    period: &LedgerPeriod,
    account_id: i64,
    taille: i64,
) -> (Vec<(i64, Decimal)>, Vec<(NaiveDate, i64, Decimal)>) {
    let mut lignes = Vec::new();
    let mut ruptures = Vec::new();
    let mut offset = 0;
    loop {
        let s = section_paginee(pool, company_id, period, account_id, offset, Some(taille)).await;
        lignes.extend(s.lines.iter().map(|l| (l.line_id, l.running_balance)));
        ruptures.extend(s.fiscal_year_breaks.iter().map(rupture));
        offset += taille;
        if offset >= s.line_count {
            return (lignes, ruptures);
        }
    }
}

/// Un compte de bilan : la clôture du grand livre **doit** égaler le solde du
/// même compte au bilan. C'est l'invariant qui attrape une ouverture mal bornée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn concordance_compte_de_bilan_avec_le_bilan(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    post(
        &pool,
        &seeded,
        ymd(2026, 3, 10),
        caisse,
        ventes,
        dec!(500.00),
        "vente 1",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 6, 20),
        caisse,
        ventes,
        dec!(250.50),
        "vente 2",
    )
    .await;

    let period = full_year();
    let ledger = generate(&pool, seeded.company_id, &period, &LedgerOptions::default())
        .await
        .expect("grand livre");

    let section = ledger
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .expect("le compte caisse doit figurer");

    // Le bilan prend une `ReportPeriod` ; seule sa borne haute compte (il est
    // cumulatif). On la fait coïncider avec le `to` du grand livre.
    let rp = ReportPeriod::resolve(&pool, seeded.company_id, seeded.fiscal_year_id, None, None)
        .await
        .expect("période d'exercice");
    let bs = generate_balance_sheet(&pool, seeded.company_id, &rp)
        .await
        .expect("bilan");
    let bs_line = bs
        .assets
        .iter()
        .find(|a| a.account_id == caisse)
        .expect("la caisse doit figurer au bilan");

    assert_eq!(
        section.closing, bs_line.balance,
        "la clôture du grand livre doit égaler le solde du bilan"
    );
}

/// Un compte de résultat : ses mouvements **doivent** égaler ceux de la balance,
/// et sa clôture le montant du compte de résultat.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn concordance_compte_de_resultat(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    post(
        &pool,
        &seeded,
        ymd(2026, 2, 1),
        caisse,
        ventes,
        dec!(1000.00),
        "vente",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 8, 1),
        caisse,
        ventes,
        dec!(300.00),
        "vente",
    )
    .await;

    let period = full_year();
    let ledger = generate(&pool, seeded.company_id, &period, &LedgerOptions::default())
        .await
        .expect("grand livre");
    let section = ledger
        .sections
        .iter()
        .find(|s| s.account_id == ventes)
        .expect("le compte de ventes doit figurer");

    let rp = ReportPeriod::resolve(&pool, seeded.company_id, seeded.fiscal_year_id, None, None)
        .await
        .expect("période d'exercice");

    let tb = generate_trial_balance(&pool, seeded.company_id, &rp)
        .await
        .expect("balance");
    let tb_row = tb
        .rows
        .iter()
        .find(|r| r.account_id == ventes)
        .expect("le compte doit figurer à la balance");

    assert_eq!(
        section.total_debit, tb_row.total_debit,
        "les mouvements débit doivent concorder avec la balance"
    );
    assert_eq!(
        section.total_credit, tb_row.total_credit,
        "les mouvements crédit doivent concorder avec la balance"
    );

    let is = generate_income_statement(&pool, seeded.company_id, &rp)
        .await
        .expect("compte de résultat");
    let is_row = is
        .revenues
        .iter()
        .find(|r| r.account_id == ventes)
        .expect("le compte doit figurer au compte de résultat");

    assert_eq!(
        section.closing, is_row.balance,
        "la clôture doit égaler le montant du compte de résultat"
    );

    // Story 25-5-b — en cours d'exercice (à partir du 1er mars), la vente de
    // février fait l'ouverture du compte de ventes ; le grand livre et la balance
    // doivent la porter de la même façon, et clore au même nombre. Sur
    // l'exercice entier, les deux ouvertures vaudraient 0 par construction.
    let (from, to) = (ymd(2026, 3, 1), ymd(2026, 12, 31));
    let mid_ledger = generate(
        &pool,
        seeded.company_id,
        &LedgerPeriod::new(from, to).expect("période"),
        &LedgerOptions::default(),
    )
    .await
    .expect("grand livre en cours d'exercice");
    let mid_section = mid_ledger
        .sections
        .iter()
        .find(|s| s.account_id == ventes)
        .expect("le compte de ventes doit figurer");
    let mid_rp = ReportPeriod {
        fiscal_year_id: seeded.fiscal_year_id,
        start_date: from,
        end_date: to,
    };
    let mid_tb = generate_trial_balance(&pool, seeded.company_id, &mid_rp)
        .await
        .expect("balance en cours d'exercice");
    let mid_row = mid_tb
        .rows
        .iter()
        .find(|r| r.account_id == ventes)
        .expect("le compte doit figurer à la balance");
    assert_eq!(mid_section.opening, dec!(1000.00));
    assert_eq!(
        mid_section.opening, mid_row.opening_balance,
        "l'ouverture doit concorder entre grand livre et balance"
    );
    assert_eq!(
        mid_section.closing, mid_row.closing_balance,
        "la clôture doit concorder entre grand livre et balance"
    );
}

/// La partie double, vue depuis le grand livre : c'est le test le moins cher du
/// lot, et il attrape toute perte de ligne dans les jointures.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partie_double_sur_le_livre_complet(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let clients = seeded.accounts["1100"];
    let ventes = seeded.accounts["3000"];

    post(
        &pool,
        &seeded,
        ymd(2026, 4, 1),
        clients,
        ventes,
        dec!(800.00),
        "facture",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 5, 1),
        caisse,
        clients,
        dec!(800.00),
        "encaissement",
    )
    .await;

    let ledger = generate(
        &pool,
        seeded.company_id,
        &full_year(),
        &LedgerOptions::default(),
    )
    .await
    .expect("grand livre");

    let total_debit: Decimal = ledger.sections.iter().map(|s| s.total_debit).sum();
    let total_credit: Decimal = ledger.sections.iter().map(|s| s.total_credit).sum();
    assert_eq!(total_debit, total_credit, "Σ débits == Σ crédits");
}

/// ⚠️ Le test qui attrape le piège des totaux calculés sur la page rendue.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_pagination_ne_change_ni_les_totaux_ni_les_soldes(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    for i in 1..=6u32 {
        post(
            &pool,
            &seeded,
            ymd(2026, 3, i),
            caisse,
            ventes,
            dec!(100.00),
            "mouvement",
        )
        .await;
    }

    let period = full_year();
    let petite = LedgerOptions {
        limit: Some(2),
        ..Default::default()
    };
    let grande = LedgerOptions {
        limit: Some(500),
        ..Default::default()
    };

    let a = generate(&pool, seeded.company_id, &period, &petite)
        .await
        .expect("page courte");
    let b = generate(&pool, seeded.company_id, &period, &grande)
        .await
        .expect("page longue");

    let sa = a.sections.iter().find(|s| s.account_id == caisse).unwrap();
    let sb = b.sections.iter().find(|s| s.account_id == caisse).unwrap();

    assert_eq!(sa.lines.len(), 2, "la page courte rend 2 lignes");
    assert_eq!(sb.lines.len(), 6, "la page longue rend les 6");

    assert_eq!(
        sa.opening, sb.opening,
        "l'ouverture ne dépend pas de la page"
    );
    assert_eq!(
        sa.closing, sb.closing,
        "la clôture ne dépend pas de la page"
    );
    assert_eq!(sa.total_debit, sb.total_debit, "les totaux non plus");
    assert_eq!(sa.total_credit, sb.total_credit);
    assert_eq!(sa.line_count, 6, "le compte de lignes porte sur la période");
}

/// ⛔ **LE test de la règle centrale.** Un compte de bilan reporte son solde d'un
/// exercice à l'autre ; un compte de résultat repart de zéro.
///
/// ⚠️ Il exige **deux exercices** : avec un seul, les deux ouvertures sont
/// identiques et le test ne prouve rien — c'est précisément le piège du test
/// muet, et la première version de ce test y est tombée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ouverture_bilan_cumule_resultat_repart_de_zero(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    let (fy2025, _fy2026) = deux_exercices(&pool, &seeded).await;

    journal_entries::create(
        &pool,
        fy2025,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: ymd(2025, 6, 30),
            journal: Journal::OD,
            description: "vente de l'exercice précédent".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: caisse,
                    debit: dec!(400.00),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: ventes,
                    debit: Decimal::ZERO,
                    credit: dec!(400.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture 2025");

    // Grand livre sur 2026 : le mouvement de 2025 est donc AVANT la période.
    let ledger = generate(
        &pool,
        seeded.company_id,
        &full_year(),
        &LedgerOptions::default(),
    )
    .await
    .expect("grand livre");

    let caisse_s = ledger
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .expect("la caisse porte un solde reporté, donc elle figure");

    assert_eq!(
        caisse_s.opening,
        dec!(400.00),
        "compte de BILAN : le solde se reporte d'un exercice à l'autre"
    );

    // Le compte de ventes n'a aucun mouvement en 2026 et son ouverture doit être
    // NULLE — sinon il ne serait même pas rendu.
    let ventes_s = ledger.sections.iter().find(|s| s.account_id == ventes);
    match ventes_s {
        None => { /* ouverture nulle et aucun mouvement : correctement écarté */ }
        Some(s) => assert_eq!(
            s.opening,
            Decimal::ZERO,
            "compte de RÉSULTAT : il repart de zéro au nouvel exercice, \
             le produit de 2025 ne doit PAS être reporté"
        ),
    }

    // Et en le demandant explicitement, l'ouverture doit valoir zéro.
    let force = generate(
        &pool,
        seeded.company_id,
        &full_year(),
        &LedgerOptions {
            account_ids: Some(vec![ventes]),
            ..Default::default()
        },
    )
    .await
    .expect("grand livre du seul compte de ventes");
    let forced = force
        .sections
        .iter()
        .find(|s| s.account_id == ventes)
        .expect("un compte explicitement demandé est toujours rendu");
    assert_eq!(
        forced.opening,
        Decimal::ZERO,
        "le produit de l'exercice clos n'est pas reporté"
    );
}

/// Un compte sans mouvement mais à solde d'ouverture non nul **doit** se voir :
/// c'est un solde qu'il faut justifier.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_sans_mouvement_mais_avec_solde_est_rendu(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    post(
        &pool,
        &seeded,
        ymd(2026, 1, 15),
        caisse,
        ventes,
        dec!(90.00),
        "vente",
    )
    .await;

    let period = LedgerPeriod::new(ymd(2026, 7, 1), ymd(2026, 7, 31)).unwrap();
    let ledger = generate(&pool, seeded.company_id, &period, &LedgerOptions::default())
        .await
        .expect("grand livre");

    let s = ledger
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .expect("le compte doit être rendu malgré l'absence de mouvement");

    assert!(s.lines.is_empty());
    assert_eq!(s.opening, s.closing, "sans mouvement, ouverture == clôture");
    assert_eq!(s.opening, dec!(90.00));
}

/// L'ordre doit être identique d'un appel à l'autre — sinon toute comparaison,
/// tout export et toute capture d'écran deviennent instables.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_ordre_des_lignes_est_stable(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    // Trois écritures LE MÊME JOUR : c'est là que le tri secondaire compte.
    for _ in 0..3 {
        post(
            &pool,
            &seeded,
            ymd(2026, 5, 5),
            caisse,
            ventes,
            dec!(10.00),
            "même jour",
        )
        .await;
    }

    let period = full_year();
    let a = generate(&pool, seeded.company_id, &period, &LedgerOptions::default())
        .await
        .expect("premier appel");
    let b = generate(&pool, seeded.company_id, &period, &LedgerOptions::default())
        .await
        .expect("second appel");

    let ids_a: Vec<i64> = a
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .unwrap()
        .lines
        .iter()
        .map(|l| l.line_id)
        .collect();
    let ids_b: Vec<i64> = b
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .unwrap()
        .lines
        .iter()
        .map(|l| l.line_id)
        .collect();

    assert_eq!(ids_a, ids_b, "deux appels rendent le même ordre");
    assert_eq!(ids_a.len(), 3);
}

/// La contrepartie est ce qui rend l'extrait lisible : sur une écriture à deux
/// lignes, elle donne le numéro de l'autre compte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_contrepartie_nomme_l_autre_compte(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    post(
        &pool,
        &seeded,
        ymd(2026, 9, 9),
        caisse,
        ventes,
        dec!(75.00),
        "vente",
    )
    .await;

    let ledger = generate(
        &pool,
        seeded.company_id,
        &full_year(),
        &LedgerOptions::default(),
    )
    .await
    .expect("grand livre");

    let line = &ledger
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .unwrap()
        .lines[0];

    assert_eq!(line.counterpart, vec!["3000".to_string()]);
    assert_eq!(line.description, "vente", "le libellé vient de l'écriture");
}

/// **L'export n'est pas paginé, et ce n'est pas un détail de confort.**
///
/// L'écran s'arrête à [`MAX_LEDGER_LIMIT`] lignes par compte et le dit ; le
/// fichier produit, lui, doit contenir le livre entier — c'est lui qui porte
/// l'obligation de produire les livres (Olico art. 3). Le test vaut d'abord
/// pour ce que l'écriture du code rend facile à casser : `limit: None` a
/// longtemps voulu dire « le défaut, soit 500 ». Il veut désormais dire « aucune
/// borne », et rien d'autre que ce test ne le tient.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_export_sans_borne_rend_plus_que_le_plafond_de_l_ecran(pool: MySqlPool) {
    use kesh_report::general_ledger::MAX_LEDGER_LIMIT;

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    // Trois lignes de plus que le plafond : de quoi distinguer « écrêté » de
    // « tout rendu » sans ambiguïté.
    let total = MAX_LEDGER_LIMIT + 3;
    for i in 0..total {
        post(
            &pool,
            &seeded,
            ymd(2026, 3, 1),
            caisse,
            ventes,
            dec!(1.00),
            &format!("mouvement {i}"),
        )
        .await;
    }

    let period = full_year();

    let ecran = generate(
        &pool,
        seeded.company_id,
        &period,
        &LedgerOptions {
            limit: Some(MAX_LEDGER_LIMIT),
            ..Default::default()
        },
    )
    .await
    .expect("vue écran");
    let export = generate(
        &pool,
        seeded.company_id,
        &period,
        &LedgerOptions {
            limit: None,
            ..Default::default()
        },
    )
    .await
    .expect("export");

    let se = ecran
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .expect("section caisse");
    let sx = export
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .expect("section caisse");

    assert_eq!(
        se.lines.len() as i64,
        MAX_LEDGER_LIMIT,
        "l'écran s'arrête au plafond"
    );
    assert_eq!(
        sx.lines.len() as i64,
        total,
        "l'export rend TOUTES les lignes, pas le plafond"
    );
    // `line_count` est le total réel dans les deux cas : c'est lui qui permet à
    // l'écran d'annoncer honnêtement qu'il tronque.
    assert_eq!(se.line_count, total);
    assert_eq!(sx.line_count, total);
    assert_eq!(se.closing, sx.closing, "le solde ne dépend pas de la page");
}

/// Une demande au-dessus du plafond reste écrêtée — `None` est le **seul**
/// moyen de lever la borne, pas « un très grand nombre ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_limite_explicite_trop_grande_reste_ecretee(pool: MySqlPool) {
    use kesh_report::general_ledger::MAX_LEDGER_LIMIT;

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];

    for i in 0..(MAX_LEDGER_LIMIT + 2) {
        post(
            &pool,
            &seeded,
            ymd(2026, 3, 1),
            caisse,
            ventes,
            dec!(1.00),
            &format!("mouvement {i}"),
        )
        .await;
    }

    let ledger = generate(
        &pool,
        seeded.company_id,
        &full_year(),
        &LedgerOptions {
            limit: Some(100_000),
            ..Default::default()
        },
    )
    .await
    .expect("grand livre");

    let s = ledger
        .sections
        .iter()
        .find(|s| s.account_id == caisse)
        .expect("section caisse");
    assert_eq!(s.lines.len() as i64, MAX_LEDGER_LIMIT);
}

// ─── #775 — le solde progressif au-delà de la première page ──────────────────
//
// Le défaut : chaque page repartait du solde d'ouverture de la PÉRIODE, sans les
// lignes sautées par `offset`. La page 1 était juste, toutes les autres fausses —
// et l'écran, qui n'envoie jamais `offset`, ne le montrait pas : seule l'API le
// servait. Les quatre tests ci-dessous lisent donc tous au moins une page au-delà
// de la première.

/// Continuité sur un compte de BILAN : le dernier solde d'une page est le solde
/// qui précède la première ligne de la page suivante.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn continuite_du_solde_d_une_page_a_l_autre_compte_de_bilan(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];
    let charges = seeded.accounts["4000"];

    // Avant la période : fait l'ouverture de la caisse, 1000.
    post(
        &pool,
        &seeded,
        ymd(2025, 12, 15),
        caisse,
        ventes,
        dec!(1000.00),
        "report",
    )
    .await;
    // Six mouvements de sens alternés : un solde qui ne fait que croître
    // laisserait passer une erreur de signe.
    post(
        &pool,
        &seeded,
        ymd(2026, 1, 10),
        caisse,
        ventes,
        dec!(100.00),
        "m1",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 2, 10),
        charges,
        caisse,
        dec!(30.00),
        "m2",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 3, 10),
        caisse,
        ventes,
        dec!(250.00),
        "m3",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 4, 10),
        charges,
        caisse,
        dec!(70.00),
        "m4",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 5, 10),
        caisse,
        ventes,
        dec!(5.00),
        "m5",
    )
    .await;
    post(
        &pool,
        &seeded,
        ymd(2026, 6, 10),
        charges,
        caisse,
        dec!(15.00),
        "m6",
    )
    .await;

    let period = full_year();
    let p1 = section_paginee(&pool, seeded.company_id, &period, caisse, 0, Some(2)).await;
    let p2 = section_paginee(&pool, seeded.company_id, &period, caisse, 2, Some(2)).await;
    let p3 = section_paginee(&pool, seeded.company_id, &period, caisse, 4, Some(2)).await;

    assert_eq!(p1.opening, dec!(1000.00));
    assert_eq!(soldes(&p1), vec![dec!(1100.00), dec!(1070.00)]);
    assert_eq!(
        soldes(&p2),
        vec![dec!(1320.00), dec!(1250.00)],
        "la page 2 part de l'ouverture PLUS les deux lignes sautées (#775)"
    );
    assert_eq!(soldes(&p3), vec![dec!(1255.00), dec!(1240.00)]);

    for (avant, apres) in [(&p1, &p2), (&p2, &p3)] {
        let dernier = avant.lines.last().expect("page non vide").running_balance;
        let premiere = &apres.lines[0];
        // Compte d'actif : le mouvement signé est débit − crédit.
        let precedent = premiere.running_balance - (premiere.debit - premiere.credit);
        assert_eq!(
            dernier, precedent,
            "le dernier solde d'une page doit précéder la première ligne de la suivante"
        );
    }
    // La clôture n'a jamais dépendu de la page ; la dernière page y aboutit.
    assert_eq!(p3.closing, dec!(1240.00));
}

/// Compte de RÉSULTAT dont l'exercice change **exactement** à la frontière de
/// deux pages : la rupture doit être posée sur la page suivante, avec le solde
/// de l'exercice clos, et le solde y repartir de zéro.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rupture_d_exercice_a_la_frontiere_de_deux_pages(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];
    let (fy2025, fy2026) = deux_exercices(&pool, &seeded).await;

    // Trois lignes en 2025, deux en 2026 : avec des pages de 3, la page 2 commence
    // sur la première ligne de 2026.
    for (date, montant) in [
        (ymd(2025, 3, 1), dec!(100.00)),
        (ymd(2025, 6, 1), dec!(200.00)),
        (ymd(2025, 9, 1), dec!(300.00)),
    ] {
        post_fy(
            &pool, &seeded, fy2025, date, caisse, ventes, montant, "vente",
        )
        .await;
    }
    for (date, montant) in [
        (ymd(2026, 2, 1), dec!(50.00)),
        (ymd(2026, 5, 1), dec!(70.00)),
    ] {
        post_fy(
            &pool, &seeded, fy2026, date, caisse, ventes, montant, "vente",
        )
        .await;
    }

    let period = LedgerPeriod::new(ymd(2025, 1, 1), ymd(2026, 12, 31)).expect("période");
    let p1 = section_paginee(&pool, seeded.company_id, &period, ventes, 0, Some(3)).await;
    let p2 = section_paginee(&pool, seeded.company_id, &period, ventes, 3, Some(3)).await;

    assert_eq!(soldes(&p1), vec![dec!(100.00), dec!(300.00), dec!(600.00)]);
    assert!(p1.fiscal_year_breaks.is_empty(), "aucune rupture en page 1");

    assert_eq!(
        soldes(&p2),
        vec![dec!(50.00), dec!(120.00)],
        "le compte de résultat repart de zéro au nouvel exercice, même en tête de page"
    );
    assert_eq!(
        p2.fiscal_year_breaks
            .iter()
            .map(rupture)
            .collect::<Vec<_>>(),
        vec![(ymd(2026, 2, 1), fy2025, dec!(600.00))],
        "la rupture qui tombe à la frontière se pose sur la page suivante, \
         avec le solde de l'exercice clos (#775)"
    );
    assert_eq!(
        p2.fiscal_year_breaks[0].closing_balance,
        p1.lines.last().expect("page 1 non vide").running_balance,
        "la rupture clôt l'exercice au dernier solde de la page précédente"
    );
    // #778 — la clôture est le solde de l'exercice en cours à la fin de la
    // période (le dernier solde progressif), non le cumul 720 ; elle ne dépend
    // pas de la page. Le cumul reste dans les totaux.
    assert_eq!(p1.closing, dec!(120.00), "clôture lue en page 1 (#778)");
    assert_eq!(p2.closing, dec!(120.00), "clôture lue en page 2 (#778)");
    assert_eq!(
        p2.total_credit,
        dec!(720.00),
        "le cumul reste dans les totaux"
    );
}

/// Page qui commence en plein milieu d'un exercice, sur un compte de RÉSULTAT à
/// ouverture non nulle, puis pages du nouvel exercice : les lignes sautées
/// portent à la fois l'ouverture et la remise à zéro.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn page_en_plein_milieu_d_exercice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];
    let (fy2025, fy2026) = deux_exercices(&pool, &seeded).await;

    // Avant la période, dans le même exercice : l'ouverture vaut 40.
    post_fy(
        &pool,
        &seeded,
        fy2025,
        ymd(2025, 3, 1),
        caisse,
        ventes,
        dec!(40.00),
        "avant",
    )
    .await;
    for (m, montant) in [
        (7, dec!(10.00)),
        (8, dec!(20.00)),
        (9, dec!(30.00)),
        (10, dec!(40.00)),
    ] {
        post_fy(
            &pool,
            &seeded,
            fy2025,
            ymd(2025, m, 1),
            caisse,
            ventes,
            montant,
            "v25",
        )
        .await;
    }
    for (m, montant) in [(2, dec!(5.00)), (3, dec!(6.00)), (4, dec!(7.00))] {
        post_fy(
            &pool,
            &seeded,
            fy2026,
            ymd(2026, m, 1),
            caisse,
            ventes,
            montant,
            "v26",
        )
        .await;
    }

    let period = LedgerPeriod::new(ymd(2025, 6, 1), ymd(2026, 12, 31)).expect("période");
    let page = |offset| section_paginee(&pool, seeded.company_id, &period, ventes, offset, Some(2));

    let p2 = page(2).await;
    assert_eq!(p2.opening, dec!(40.00));
    assert_eq!(
        soldes(&p2),
        vec![dec!(100.00), dec!(140.00)],
        "en plein exercice : ouverture 40 + 10 + 20 sautées, puis 30 et 40"
    );
    assert!(p2.fiscal_year_breaks.is_empty());

    let p3 = page(4).await;
    assert_eq!(soldes(&p3), vec![dec!(5.00), dec!(11.00)]);
    assert_eq!(
        p3.fiscal_year_breaks
            .iter()
            .map(rupture)
            .collect::<Vec<_>>(),
        vec![(ymd(2026, 2, 1), fy2025, dec!(140.00))]
    );

    // En plein exercice 2026 : les lignes de 2025 sautées ne se reportent pas.
    let p4 = page(6).await;
    assert_eq!(soldes(&p4), vec![dec!(18.00)]);
    assert!(
        p4.fiscal_year_breaks.is_empty(),
        "une rupture franchie dans les lignes sautées appartient à une page précédente"
    );
}

/// Équivalence : pour toute taille de page, la lecture paginée rend, ligne pour
/// ligne, les soldes et les ruptures de la lecture sans pagination — sur un
/// compte d'actif, un compte de produits et un compte de charges.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_lecture_paginee_egale_la_lecture_entiere(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];
    let charges = seeded.accounts["4000"];
    let (fy2025, fy2026) = deux_exercices(&pool, &seeded).await;

    // Avant la période : ouvertures non nulles pour les trois comptes.
    post_fy(
        &pool,
        &seeded,
        fy2025,
        ymd(2025, 2, 1),
        caisse,
        ventes,
        dec!(500.00),
        "o1",
    )
    .await;
    post_fy(
        &pool,
        &seeded,
        fy2025,
        ymd(2025, 3, 1),
        charges,
        caisse,
        dec!(80.00),
        "o2",
    )
    .await;
    // Dans la période, avec des écritures du même jour de part et d'autre de la
    // fin d'exercice : c'est là que l'ordre de tri et la rupture comptent.
    let mouvements = [
        (fy2025, ymd(2025, 7, 1), caisse, ventes, dec!(11.00)),
        (fy2025, ymd(2025, 9, 1), charges, caisse, dec!(7.00)),
        (fy2025, ymd(2025, 12, 31), caisse, ventes, dec!(13.00)),
        (fy2025, ymd(2025, 12, 31), charges, caisse, dec!(3.00)),
        (fy2025, ymd(2025, 12, 31), ventes, caisse, dec!(2.00)),
        (fy2026, ymd(2026, 1, 1), caisse, ventes, dec!(17.00)),
        (fy2026, ymd(2026, 1, 1), charges, caisse, dec!(19.00)),
        (fy2026, ymd(2026, 4, 1), caisse, ventes, dec!(23.00)),
        (fy2026, ymd(2026, 8, 1), charges, caisse, dec!(29.00)),
        (fy2026, ymd(2026, 8, 1), caisse, ventes, dec!(31.00)),
    ];
    for (fy, date, d, c, montant) in mouvements {
        post_fy(&pool, &seeded, fy, date, d, c, montant, "mvt").await;
    }

    let period = LedgerPeriod::new(ymd(2025, 6, 1), ymd(2026, 12, 31)).expect("période");
    for compte in [caisse, ventes, charges] {
        let entiere = section_paginee(&pool, seeded.company_id, &period, compte, 0, None).await;
        let attendu_lignes: Vec<(i64, Decimal)> = entiere
            .lines
            .iter()
            .map(|l| (l.line_id, l.running_balance))
            .collect();
        let attendu_ruptures: Vec<_> = entiere.fiscal_year_breaks.iter().map(rupture).collect();
        assert!(
            attendu_lignes.len() >= 4,
            "assez de lignes pour plusieurs pages"
        );
        // Sans rupture, l'équivalence ne prouverait rien des remises à zéro.
        if compte != caisse {
            assert_eq!(
                attendu_ruptures.len(),
                1,
                "compte de résultat : une rupture"
            );
        }

        // #778 — la clôture est le dernier solde progressif de la lecture entière.
        assert_eq!(
            entiere.closing,
            attendu_lignes.last().expect("lignes").1,
            "compte {compte} : clôture = dernier solde progressif"
        );

        for taille in 1..=attendu_lignes.len() as i64 {
            let (lignes, ruptures) =
                lecture_par_pages(&pool, seeded.company_id, &period, compte, taille).await;
            assert_eq!(
                lignes, attendu_lignes,
                "compte {compte}, pages de {taille} : soldes différents de la lecture entière"
            );
            assert_eq!(
                ruptures, attendu_ruptures,
                "compte {compte}, pages de {taille} : ruptures différentes de la lecture entière"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// #777 et #778 — l'exercice de l'ouverture et la clôture d'un compte de
// résultat quand la période franchit une fin d'exercice. Montants et dates
// synthétiques.
// ---------------------------------------------------------------------------

/// Crée un exercice de la société seedée.
async fn creer_exercice(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    name: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(seeded.company_id)
    .bind(name)
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .expect("exercice créé")
}

/// Trois exercices consécutifs : l'exercice seedé rétréci à 2024, puis 2025 et
/// 2026. Rend `(fy2024, fy2025, fy2026)` (cf. `deux_exercices` pour le
/// rétrécissement).
async fn trois_exercices(pool: &MySqlPool, seeded: &SeededCompany) -> (i64, i64, i64) {
    sqlx::query("UPDATE fiscal_years SET end_date = '2024-12-31' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(pool)
        .await
        .expect("exercice seedé rétréci à 2024");
    let fy2025 = creer_exercice(pool, seeded, "FY 2025", ymd(2025, 1, 1), ymd(2025, 12, 31)).await;
    let fy2026 = creer_exercice(pool, seeded, "FY 2026", ymd(2026, 1, 1), ymd(2026, 12, 31)).await;
    (seeded.fiscal_year_id, fy2025, fy2026)
}

/// Les ruptures d'une section, réduites à ce qui se compare.
fn ruptures(s: &LedgerSection) -> Vec<(NaiveDate, i64, Decimal)> {
    s.fiscal_year_breaks.iter().map(rupture).collect()
}

/// #777 — le cas de l'issue. La période commence dans l'exercice 2025, dont
/// l'ouverture porte une vente de 400 ; sa seule ligne tombe en 2026. Le compte
/// de résultat doit clore 2025 à 400 avant cette ligne, puis repartir de zéro ;
/// le compte de bilan, lui, ne change pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ouverture_de_resultat_close_avant_une_premiere_ligne_d_un_exercice_posterieur(
    pool: MySqlPool,
) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];
    let (fy2025, fy2026) = deux_exercices(&pool, &seeded).await;

    post_fy(
        &pool,
        &seeded,
        fy2025,
        ymd(2025, 3, 1),
        caisse,
        ventes,
        dec!(400.00),
        "vente 2025",
    )
    .await;
    post_fy(
        &pool,
        &seeded,
        fy2026,
        ymd(2026, 2, 1),
        caisse,
        ventes,
        dec!(100.00),
        "vente 2026",
    )
    .await;

    let period = LedgerPeriod::new(ymd(2025, 6, 1), ymd(2026, 12, 31)).expect("période");
    let v = section_paginee(&pool, seeded.company_id, &period, ventes, 0, None).await;
    assert_eq!(v.opening, dec!(400.00), "l'ouverture appartient à 2025");
    assert_eq!(
        ruptures(&v),
        vec![(ymd(2026, 2, 1), fy2025, dec!(400.00))],
        "une rupture clôt 2025 à 400 avant la première ligne, de 2026 (#777)"
    );
    assert_eq!(
        soldes(&v),
        vec![dec!(100.00)],
        "le solde repart de zéro en 2026 : l'ouverture de 2025 ne s'y reporte pas (#777)"
    );
    assert_eq!(v.closing, dec!(100.00));

    // Concordance : clos en 2026, le compte vaut ce que dit la balance de 2026.
    let tb = generate_trial_balance(
        &pool,
        seeded.company_id,
        &ReportPeriod {
            fiscal_year_id: fy2026,
            start_date: ymd(2026, 1, 1),
            end_date: ymd(2026, 12, 31),
        },
    )
    .await
    .expect("balance 2026");
    let row = tb
        .rows
        .iter()
        .find(|r| r.account_id == ventes)
        .expect("ventes à la balance 2026");
    assert_eq!(v.closing, row.closing_balance, "clôture = balance de 2026");

    // Compte de BILAN : la chaîne est continue, rien ne change.
    let c = section_paginee(&pool, seeded.company_id, &period, caisse, 0, None).await;
    assert_eq!(c.opening, dec!(400.00));
    assert_eq!(soldes(&c), vec![dec!(500.00)]);
    assert!(
        c.fiscal_year_breaks.is_empty(),
        "un compte de bilan ne rompt pas"
    );
    assert_eq!(c.closing, dec!(500.00));
}

/// Trois exercices : un compte de produits mouvementé dans chacun, un compte de
/// charges dont la première ligne saute l'exercice du milieu (#777), et le
/// compte de bilan qui en reçoit la contrepartie, continu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_de_resultat_sur_trois_exercices(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];
    let charges = seeded.accounts["4000"];
    let (fy2024, fy2025, fy2026) = trois_exercices(&pool, &seeded).await;

    let mouvements = [
        // Avant la période, dans l'exercice qui contient `from`.
        (fy2024, ymd(2024, 2, 1), caisse, ventes, dec!(10.00)),
        (fy2024, ymd(2024, 2, 1), charges, caisse, dec!(15.00)),
        // Dans la période.
        (fy2024, ymd(2024, 7, 1), caisse, ventes, dec!(20.00)),
        (fy2025, ymd(2025, 3, 1), caisse, ventes, dec!(30.00)),
        (fy2025, ymd(2025, 9, 1), caisse, ventes, dec!(40.00)),
        (fy2026, ymd(2026, 5, 1), caisse, ventes, dec!(50.00)),
        (fy2026, ymd(2026, 6, 1), charges, caisse, dec!(25.00)),
    ];
    for (fy, date, d, c, montant) in mouvements {
        post_fy(&pool, &seeded, fy, date, d, c, montant, "mvt").await;
    }

    let period = LedgerPeriod::new(ymd(2024, 6, 1), ymd(2026, 12, 31)).expect("période");

    let v = section_paginee(&pool, seeded.company_id, &period, ventes, 0, None).await;
    assert_eq!(v.opening, dec!(10.00));
    assert_eq!(
        soldes(&v),
        vec![dec!(30.00), dec!(30.00), dec!(70.00), dec!(50.00)]
    );
    assert_eq!(
        ruptures(&v),
        vec![
            (ymd(2025, 3, 1), fy2024, dec!(30.00)),
            (ymd(2026, 5, 1), fy2025, dec!(70.00)),
        ]
    );
    assert_eq!(v.closing, dec!(50.00), "clôture = solde de 2026 (#778)");
    assert_eq!(
        v.total_credit,
        dec!(140.00),
        "le cumul reste dans les totaux"
    );

    let ch = section_paginee(&pool, seeded.company_id, &period, charges, 0, None).await;
    assert_eq!(ch.opening, dec!(15.00));
    assert_eq!(
        ruptures(&ch),
        vec![(ymd(2026, 6, 1), fy2024, dec!(15.00))],
        "la première ligne, de 2026, clôt l'exercice de `from` (#777)"
    );
    assert_eq!(soldes(&ch), vec![dec!(25.00)]);
    assert_eq!(ch.closing, dec!(25.00));

    let c = section_paginee(&pool, seeded.company_id, &period, caisse, 0, None).await;
    assert_eq!(c.opening, dec!(-5.00));
    assert_eq!(
        soldes(&c),
        vec![
            dec!(15.00),
            dec!(45.00),
            dec!(85.00),
            dec!(135.00),
            dec!(110.00)
        ]
    );
    assert!(c.fiscal_year_breaks.is_empty());
    assert_eq!(c.closing, dec!(110.00));

    // Les pages rendent ce que rend la lecture entière, ruptures comprises.
    for (compte, entiere) in [(ventes, &v), (charges, &ch), (caisse, &c)] {
        let attendu: Vec<(i64, Decimal)> = entiere
            .lines
            .iter()
            .map(|l| (l.line_id, l.running_balance))
            .collect();
        for taille in 1..=attendu.len() as i64 {
            let (lignes, r) =
                lecture_par_pages(&pool, seeded.company_id, &period, compte, taille).await;
            assert_eq!(lignes, attendu, "compte {compte}, pages de {taille}");
            assert_eq!(r, ruptures(entiere), "compte {compte}, pages de {taille}");
        }
    }
}

/// #778 — la clôture d'un compte de résultat à cheval sur deux exercices est le
/// solde de l'exercice en cours à la fin de la période, même quand aucune ligne
/// de cet exercice ne figure dans la période : la fin d'exercice franchie après
/// la dernière ligne est alors rendue, et la clôture vaut zéro.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cloture_d_un_compte_de_resultat_a_cheval_sur_deux_exercices(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let caisse = seeded.accounts["1000"];
    let ventes = seeded.accounts["3000"];
    let (fy2025, _fy2026) = deux_exercices(&pool, &seeded).await;

    for (date, montant, label) in [
        (ymd(2025, 3, 1), dec!(400.00), "avant la période"),
        (ymd(2025, 7, 1), dec!(100.00), "v1"),
        (ymd(2025, 9, 1), dec!(200.00), "v2"),
    ] {
        post_fy(&pool, &seeded, fy2025, date, caisse, ventes, montant, label).await;
    }

    // Deux lignes en 2025, aucune en 2026.
    let period = LedgerPeriod::new(ymd(2025, 6, 1), ymd(2026, 6, 30)).expect("période");
    let v = section_paginee(&pool, seeded.company_id, &period, ventes, 0, None).await;
    assert_eq!(v.opening, dec!(400.00));
    assert_eq!(soldes(&v), vec![dec!(500.00), dec!(700.00)]);
    assert_eq!(
        ruptures(&v),
        vec![(ymd(2026, 1, 1), fy2025, dec!(700.00))],
        "la fin d'exercice franchie après la dernière ligne est rendue, \
         datée du premier jour de 2026"
    );
    assert_eq!(
        v.closing,
        Decimal::ZERO,
        "clôture = solde de l'exercice en cours à la fin de la période (#778)"
    );
    assert_eq!(
        v.total_credit,
        dec!(300.00),
        "le cumul reste dans les totaux"
    );
    assert!(!v.unnatural_balance);

    // Paginée : la rupture finale n'apparaît que sur la dernière page, et la
    // clôture ne dépend pas de la page.
    let p1 = section_paginee(&pool, seeded.company_id, &period, ventes, 0, Some(1)).await;
    let p2 = section_paginee(&pool, seeded.company_id, &period, ventes, 1, Some(1)).await;
    let p3 = section_paginee(&pool, seeded.company_id, &period, ventes, 2, Some(1)).await;
    assert!(p1.fiscal_year_breaks.is_empty(), "pas en page 1");
    assert_eq!(ruptures(&p2), ruptures(&v), "en dernière page");
    assert!(p3.fiscal_year_breaks.is_empty(), "pas au-delà de la fin");
    for p in [&p1, &p2, &p3] {
        assert_eq!(p.closing, Decimal::ZERO);
    }

    // Aucune ligne dans la période : l'ouverture se clôt quand même, et la
    // clôture vaut zéro.
    let vide = LedgerPeriod::new(ymd(2025, 10, 1), ymd(2026, 6, 30)).expect("période");
    let s = section_paginee(&pool, seeded.company_id, &vide, ventes, 0, None).await;
    assert_eq!(s.opening, dec!(700.00));
    assert!(s.lines.is_empty());
    assert_eq!(ruptures(&s), vec![(ymd(2026, 1, 1), fy2025, dec!(700.00))]);
    assert_eq!(s.closing, Decimal::ZERO);

    // Période d'un seul exercice : clôture = ouverture + mouvements, sans rupture.
    let un_seul = LedgerPeriod::new(ymd(2025, 6, 1), ymd(2025, 12, 31)).expect("période");
    let s = section_paginee(&pool, seeded.company_id, &un_seul, ventes, 0, None).await;
    assert!(s.fiscal_year_breaks.is_empty());
    assert_eq!(s.closing, dec!(700.00));

    // Compte de bilan sur la même période : continu.
    let c = section_paginee(&pool, seeded.company_id, &period, caisse, 0, None).await;
    assert!(c.fiscal_year_breaks.is_empty());
    assert_eq!(c.closing, dec!(700.00));
}
