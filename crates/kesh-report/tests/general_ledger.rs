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
use kesh_report::general_ledger::{LedgerOptions, LedgerPeriod, generate};
use kesh_report::period::ReportPeriod;
use kesh_report::{generate_balance_sheet, generate_income_statement, generate_trial_balance};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("date valide")
}

/// Poste une écriture équilibrée à deux lignes.
async fn post(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    date: NaiveDate,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
    label: &str,
) {
    journal_entries::create(
        pool,
        seeded.fiscal_year_id,
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

    // ⚠️ La fixture seede UN exercice de ONZE ANS (2020-2030, « large pour
    // tolérer la dérive d'horloge CI »). Il faut donc le RÉTRÉCIR avant de créer
    // le suivant, sinon les deux se chevauchent et un mouvement de 2025 tombe
    // dans le même exercice qu'une période de 2026 — auquel cas son report
    // devient correct et le test ne prouve plus rien.
    sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .expect("exercice seedé rétréci à 2025");
    let fy2025 = seeded.fiscal_year_id;

    let _fy2026: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, updated_at) \
         VALUES (?, 'FY 2026', '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .expect("exercice 2026 créé");

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
