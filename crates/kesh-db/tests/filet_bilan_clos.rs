//! Story 15-12b (#543) — le filet sous un bilan clos, côté base.
//!
//! Le bilan est cumulatif depuis l'origine : une écriture de N figure dans le
//! bilan de tout exercice postérieur. Depuis la Story 15-12a, « N ouvert, N+1
//! clos » n'est plus atteignable par l'application ; il subsiste dans les
//! données héritées. Ces tests posent donc l'état fautif **par SQL direct**,
//! par une connexion distincte et validée — jamais dans la transaction de
//! l'écrivain, qui verrait sa propre écriture et ne prouverait rien.
//!
//! Couvre :
//! 1. le filet de `journal_entries::create_in_tx_inner` (AC 8) — refus, plus
//!    proche postérieur nommé, précédence, aucune écriture créée ;
//! 2. les flux qui y passent sans route (AC 20) — validation, règlement et
//!    dévalidation d'une facture (avec la paire `INVOICE_EMAILED`), contre-
//!    passation ;
//! 3. la réparation (AC 16) — les deux voies du bandeau ;
//! 4. l'angle mort assumé des prédicteurs d'annulation (AC 18, C120, #568).

use chrono::{NaiveDate, Utc};
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    FiscalYearStatus, NewInvoice, NewInvoiceLine, NewJournalEntry, NewJournalEntryLine,
    NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::fiscal_years::FY_REOPEN_LIFO_BLOCKED_KEY;
use kesh_db::repositories::{
    contacts, fiscal_years, invoice_settlements_write, invoices, journal_entries,
    settlement_cancellation, supplier_invoices,
};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Société seedée ; son exercice (2020-2030 au seed) ramené à l'année 2026,
/// nommé « Exercice 2026 », pour que d'autres exercices annuels l'entourent.
async fn societe(pool: &MySqlPool) -> SeededCompany {
    let s = seed_accounting_company(pool).await.expect("seed");
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2026', start_date = '2026-01-01', \
         end_date = '2026-12-31' WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    s
}

/// Un exercice annuel au statut voulu, posé par SQL (la création refuserait un
/// exercice antérieur à un exercice clos, et la clôture un exercice précédé d'un
/// exercice ouvert : seul l'état importe ici).
async fn exercice(pool: &MySqlPool, s: &SeededCompany, annee: i32, status: &str) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(s.company_id)
    .bind(format!("Exercice {annee}"))
    .bind(d(annee, 1, 1))
    .bind(d(annee, 12, 31))
    .bind(status)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn poser_statut(pool: &MySqlPool, fy: i64, status: &str) {
    sqlx::query("UPDATE fiscal_years SET status = ? WHERE id = ?")
        .bind(status)
        .bind(fy)
        .execute(pool)
        .await
        .unwrap();
}

fn corps(s: &SeededCompany, date: NaiveDate) -> NewJournalEntry {
    NewJournalEntry {
        company_id: s.company_id,
        entry_date: date,
        journal: Journal::OD,
        description: "filet 15-12b".into(),
        project_id: None,
        lines: vec![
            NewJournalEntryLine {
                account_id: s.accounts["1000"],
                debit: dec!(100.00),
                credit: dec!(0),
                project_id: None,
            },
            NewJournalEntryLine {
                account_id: s.accounts["1100"],
                debit: dec!(0),
                credit: dec!(100.00),
                project_id: None,
            },
        ],
    }
}

async fn ecritures(pool: &MySqlPool, s: &SeededCompany) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(s.company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `LaterFiscalYearClosed` nommant l'exercice attendu, ou panique.
fn assert_posterieur_clos<T: std::fmt::Debug>(r: Result<T, DbError>, id: i64, nom: &str) {
    match r {
        Err(DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        }) => {
            assert_eq!(fiscal_year_id, id, "le PLUS PROCHE postérieur clos");
            assert_eq!(fiscal_year_name, nom);
        }
        other => panic!("attendu LaterFiscalYearClosed({nom}), obtenu {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 1. Le filet de `create_in_tx_inner` (AC 8)
// ---------------------------------------------------------------------------

/// AC 8 · AC 20 — la saisie dans N sous N+1 et N+2 clos est refusée, nomme le
/// plus proche (N+1), et ne laisse aucune écriture.
///
/// ⛔ Tue la mutation (v) — retirer le filet de `create_in_tx_inner`.
#[sqlx::test(migrations = "./test-schema")]
async fn la_saisie_sous_un_posterieur_clos_est_refusee(pool: MySqlPool) {
    let s = societe(&pool).await;
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let _n2 = exercice(&pool, &s, 2028, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    let r = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        s.admin_user_id,
        corps(&s, d(2026, 6, 1)),
    )
    .await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
    assert_eq!(ecritures(&pool, &s).await, avant, "aucune écriture créée");

    // Témoin : un exercice postérieur OUVERT ne gêne rien.
    poser_statut(&pool, n1, "Open").await;
    let _ = exercice(&pool, &s, 2025, "Closed").await; // antérieur clos : sans effet
    sqlx::query("DELETE FROM fiscal_years WHERE company_id = ? AND name = 'Exercice 2028'")
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    journal_entries::create(
        &pool,
        s.fiscal_year_id,
        s.admin_user_id,
        corps(&s, d(2026, 6, 1)),
    )
    .await
    .expect("sans postérieur clos, la saisie passe");
}

/// AC 8 — précédence, paire « exercice clos » > « postérieur clos » : N clos et
/// N+1 clos → `FiscalYearClosed` (l'état de l'exercice de l'écriture parle
/// d'abord, comme au `PUT`).
#[sqlx::test(migrations = "./test-schema")]
async fn l_exercice_clos_parle_avant_le_posterieur(pool: MySqlPool) {
    let s = societe(&pool).await;
    let _ = exercice(&pool, &s, 2027, "Closed").await;
    poser_statut(&pool, s.fiscal_year_id, "Closed").await;
    let r = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        s.admin_user_id,
        corps(&s, d(2026, 6, 1)),
    )
    .await;
    assert!(matches!(r, Err(DbError::FiscalYearClosed)), "obtenu {r:?}");
}

/// AC 8 — précédence, paire « postérieur clos » > « date hors exercice » : une
/// date hors de N sous N+1 clos rend le filet, non `DateOutsideFiscalYear`.
#[sqlx::test(migrations = "./test-schema")]
async fn le_posterieur_parle_avant_la_date_hors_exercice(pool: MySqlPool) {
    let s = societe(&pool).await;
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let r = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        s.admin_user_id,
        corps(&s, d(2025, 6, 1)),
    )
    .await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
}

/// AC 8 — précédence, paire « postérieur clos » > « période verrouillée » : le
/// verrou de période parle en dernier.
#[sqlx::test(migrations = "./test-schema")]
async fn le_posterieur_parle_avant_le_verrou_de_periode(pool: MySqlPool) {
    let s = societe(&pool).await;
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    sqlx::query("UPDATE companies SET books_locked_through = '2026-06-30' WHERE id = ?")
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let r = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        s.admin_user_id,
        corps(&s, d(2026, 6, 1)),
    )
    .await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
}

// ---------------------------------------------------------------------------
// 2. Les flux qui passent par les deux points de passage (AC 20)
// ---------------------------------------------------------------------------

async fn client(pool: &MySqlPool, s: &SeededCompany) -> i64 {
    contacts::create(
        pool,
        s.admin_user_id,
        NewContact {
            company_id: s.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client 15-12b".into(),
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
            salutation: Salutation::Neutre,
        },
    )
    .await
    .expect("contact")
    .id
}

/// Une facture brouillon de 500.00 (taux 0), datée du 15.06.2026.
async fn brouillon(pool: &MySqlPool, s: &SeededCompany, contact: i64) -> i64 {
    invoices::create(
        pool,
        s.admin_user_id,
        NewInvoice {
            company_id: s.company_id,
            contact_id: contact,
            date: d(2026, 6, 15),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: dec!(500.00),
                vat_rate: dec!(0),
            }],
        },
    )
    .await
    .expect("facture")
    .0
    .id
}

/// AC 20 — la **validation** d'une facture dans N sous N+1 clos est refusée :
/// la facture reste brouillon, aucune écriture.
#[sqlx::test(migrations = "./test-schema")]
async fn la_validation_d_une_facture_est_refusee(pool: MySqlPool) {
    let s = societe(&pool).await;
    let contact = client(&pool, &s).await;
    let inv = brouillon(&pool, &s, contact).await;
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    let r = invoices::validate_invoice(&pool, s.company_id, inv, s.admin_user_id).await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
    assert_eq!(ecritures(&pool, &s).await, avant, "aucune écriture créée");
    let statut: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
        .bind(inv)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "draft", "la facture reste un brouillon");
}

/// AC 20 — le **règlement** d'une facture validée, daté dans N sous N+1 clos,
/// est refusé : aucune écriture, aucun règlement.
#[sqlx::test(migrations = "./test-schema")]
async fn le_reglement_d_une_facture_est_refuse(pool: MySqlPool) {
    let s = societe(&pool).await;
    let contact = client(&pool, &s).await;
    let inv = brouillon(&pool, &s, contact).await;
    invoices::validate_invoice(&pool, s.company_id, inv, s.admin_user_id)
        .await
        .expect("validation");
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    let r = invoice_settlements_write::settle_invoice(
        &pool,
        s.admin_user_id,
        s.company_id,
        inv,
        SettlementChoice::InternalAccount {
            account_id: s.accounts["1000"],
        },
        dec!(500.00),
        d(2026, 7, 1),
    )
    .await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
    assert_eq!(ecritures(&pool, &s).await, avant, "aucune écriture créée");
    let reglements: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?")
            .bind(inv)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reglements, 0);
}

/// AC 10 · AC 20 — la **dévalidation** d'une facture de N sous N+1 clos est
/// refusée (avant la Story 15-12b, ce chemin ne le contrôlait pas —
/// C-15-8-29) ; l'écriture et la facture restent.
///
/// ⛔ Tue la mutation (vi) — rétablir `if enforce_ownership` devant la lecture
/// de l'étape 2-bis de `delete_in_tx`.
#[sqlx::test(migrations = "./test-schema")]
async fn la_devalidation_sous_un_posterieur_clos_est_refusee(pool: MySqlPool) {
    let s = societe(&pool).await;
    let contact = client(&pool, &s).await;
    let inv = brouillon(&pool, &s, contact).await;
    let v = invoices::validate_invoice(&pool, s.company_id, inv, s.admin_user_id)
        .await
        .expect("validation");
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    let r =
        invoices::unvalidate(&pool, s.company_id, inv, s.admin_user_id, v.invoice.version).await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
    assert_eq!(
        ecritures(&pool, &s).await,
        avant,
        "aucune écriture supprimée"
    );
    let statut: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
        .bind(inv)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "validated");
}

/// AC 10 — paire de précédence : une facture **envoyée** sous un postérieur
/// clos répond `INVOICE_EMAILED` — les motifs propres de la facture parlent
/// avant ceux de `delete_in_tx`.
#[sqlx::test(migrations = "./test-schema")]
async fn la_facture_envoyee_parle_avant_le_posterieur_clos(pool: MySqlPool) {
    let s = societe(&pool).await;
    let contact = client(&pool, &s).await;
    let inv = brouillon(&pool, &s, contact).await;
    invoices::validate_invoice(&pool, s.company_id, inv, s.admin_user_id)
        .await
        .expect("validation");
    let envoyee = invoices::mark_emailed(
        &pool,
        s.company_id,
        inv,
        "client@example.ch",
        "Facture",
        s.admin_user_id,
        None,
    )
    .await
    .expect("envoi");
    let _ = exercice(&pool, &s, 2027, "Closed").await;

    let r = invoices::unvalidate(&pool, s.company_id, inv, s.admin_user_id, envoyee.version).await;
    match r {
        Err(e @ DbError::InvoiceNotUnvalidatable { .. }) => {
            assert_eq!(e.error_code(), "INVOICE_EMAILED", "obtenu {e:?}");
        }
        other => panic!("attendu INVOICE_EMAILED, obtenu {other:?}"),
    }
}

/// Une facture fournisseur `open` de 300.00, datée du 15.06.2026, et son
/// écriture d'achat ; le compte fournisseurs est posé dans les réglages.
async fn facture_fournisseur(pool: &MySqlPool, s: &SeededCompany) -> Result<i64, DbError> {
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(s.accounts["2000"])
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap();
    let fournisseur = NewContact {
        company_id: s.company_id,
        contact_type: ContactType::Entreprise,
        name: "Fournisseur 15-12b".into(),
        first_name: None,
        last_name: None,
        is_client: false,
        is_supplier: true,
        address: Some("Rue 2\n1000 Lausanne".into()),
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
        salutation: Salutation::Neutre,
    };
    let existant: Option<i64> =
        sqlx::query_scalar("SELECT id FROM contacts WHERE company_id = ? AND name = ?")
            .bind(s.company_id)
            .bind(&fournisseur.name)
            .fetch_optional(pool)
            .await
            .unwrap();
    let contact_id = match existant {
        Some(id) => id,
        None => {
            contacts::create(pool, s.admin_user_id, fournisseur)
                .await
                .expect("fournisseur")
                .id
        }
    };
    supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: s.company_id,
            contact_id,
            supplier_invoice_number: Some("FF-1512B".into()),
            invoice_date: d(2026, 6, 15),
            due_date: None,
            creditor_iban: Some("CH5604835012345678009".into()),
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Achat".into(),
                quantity: dec!(1),
                unit_price: dec!(300.00),
                vat_rate: dec!(0),
                expense_account_id: s.accounts["4000"],
            }],
        },
        s.admin_user_id,
    )
    .await
    .map(|f| f.invoice.id)
}

/// AC 20 — la **saisie** d'une facture fournisseur et son **paiement**, datés
/// dans N sous N+1 clos, sont refusés ; aucune écriture.
#[sqlx::test(migrations = "./test-schema")]
async fn la_facture_fournisseur_et_son_paiement_sont_refuses(pool: MySqlPool) {
    let s = societe(&pool).await;
    let inv = facture_fournisseur(&pool, &s)
        .await
        .expect("facture fournisseur");
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    let r = supplier_invoices::pay(
        &pool,
        s.company_id,
        inv,
        SettlementChoice::InternalAccount {
            account_id: s.accounts["1000"],
        },
        d(2026, 7, 2),
        s.admin_user_id,
    )
    .await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
    let statut: String = sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(inv)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "open", "la facture n'est pas payée");

    let r = facture_fournisseur(&pool, &s).await;
    assert_posterieur_clos(r, n1, "Exercice 2027");
    assert_eq!(ecritures(&pool, &s).await, avant, "aucune écriture créée");
}

/// AC 12 — la **contre-passation** est datée du jour : elle tombe dans
/// l'exercice qui couvre le jour, et le filet la refuse si CET exercice a un
/// postérieur clos (un exercice futur clôturé d'avance). Aucune écriture.
#[sqlx::test(migrations = "./test-schema")]
async fn la_contre_passation_du_jour_sous_un_futur_clos_est_refusee(pool: MySqlPool) {
    // Exercice du seed (2020-2030) laissé tel quel : il couvre le jour.
    let s = seed_accounting_company(&pool).await.expect("seed");
    let e = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        s.admin_user_id,
        corps(&s, Utc::now().date_naive()),
    )
    .await
    .expect("origine");
    let futur = exercice(&pool, &s, 2031, "Closed").await;
    let avant = ecritures(&pool, &s).await;

    let mut tx = pool.begin().await.unwrap();
    let r =
        journal_entries::reverse_in_tx(&mut tx, s.company_id, e.entry.id, s.admin_user_id).await;
    drop(tx);
    assert_posterieur_clos(r, futur, "Exercice 2031");
    assert_eq!(ecritures(&pool, &s).await, avant);
}

// ---------------------------------------------------------------------------
// 3. La réparation (AC 16) — les deux voies du bandeau
// ---------------------------------------------------------------------------

/// AC 16, premier scénario — N ouvert, N+1 clos, N+2 ouvert : rouvrir N+1
/// (rien de postérieur n'est clos), clôturer N, puis N+1. État final sain.
#[sqlx::test(migrations = "./test-schema")]
async fn la_reparation_par_reouverture_puis_cloture_dans_l_ordre(pool: MySqlPool) {
    let s = societe(&pool).await;
    let (n, u, c) = (s.fiscal_year_id, s.admin_user_id, s.company_id);
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let n2 = exercice(&pool, &s, 2028, "Open").await;

    let r = journal_entries::create(&pool, n, u, corps(&s, d(2026, 6, 1))).await;
    assert_posterieur_clos(r, n1, "Exercice 2027");

    let r = fiscal_years::close(&pool, u, c, n2).await;
    match r {
        Err(DbError::EarlierFiscalYearOpen { fiscal_year_id, .. }) => {
            assert_eq!(fiscal_year_id, n, "le PLUS ANCIEN antérieur ouvert")
        }
        other => panic!("attendu EarlierFiscalYearOpen(N), obtenu {other:?}"),
    }

    fiscal_years::reopen(&pool, u, c, n1, "réparation".into())
        .await
        .expect("réouverture de N+1 : rien de postérieur n'est clos");
    fiscal_years::close(&pool, u, c, n)
        .await
        .expect("clôture de N");
    let r = journal_entries::create(&pool, n, u, corps(&s, d(2026, 6, 1))).await;
    assert!(matches!(r, Err(DbError::FiscalYearClosed)), "obtenu {r:?}");
    fiscal_years::close(&pool, u, c, n1)
        .await
        .expect("clôture de N+1");

    // État sain : les clos forment un préfixe, et N+2 accueille les écritures.
    for (id, attendu) in [
        (n, FiscalYearStatus::Closed),
        (n1, FiscalYearStatus::Closed),
        (n2, FiscalYearStatus::Open),
    ] {
        let fy = fiscal_years::find_by_id(&pool, id).await.unwrap().unwrap();
        assert_eq!(fy.status, attendu, "exercice {}", fy.name);
    }
    journal_entries::create(&pool, n2, u, corps(&s, d(2028, 3, 1)))
        .await
        .expect("N+2 accueille les écritures");
}

/// AC 16, second scénario — la **réparation directe** du bandeau : N ouvert
/// (son antérieur clos), N+1 et N+2 clos. Rouvrir N+1 d'abord est refusé (LIFO
/// — le geste que le bandeau ne prescrit jamais) ; clôturer N est accepté et
/// rend l'état sain.
#[sqlx::test(migrations = "./test-schema")]
async fn la_reparation_directe_cloture_le_plus_ancien_ouvert(pool: MySqlPool) {
    let s = societe(&pool).await;
    let (n, u, c) = (s.fiscal_year_id, s.admin_user_id, s.company_id);
    let _avant = exercice(&pool, &s, 2025, "Closed").await;
    let n1 = exercice(&pool, &s, 2027, "Closed").await;
    let _n2 = exercice(&pool, &s, 2028, "Closed").await;

    let r = fiscal_years::reopen(&pool, u, c, n1, "à tort".into()).await;
    assert!(
        matches!(r, Err(DbError::Invariant(ref k)) if k == FY_REOPEN_LIFO_BLOCKED_KEY),
        "rouvrir N+1 avant N+2 est refusé : {r:?}"
    );

    fiscal_years::close(&pool, u, c, n)
        .await
        .expect("clôture de N, aucun antérieur ouvert");
    let r = journal_entries::create(&pool, n, u, corps(&s, d(2026, 6, 1))).await;
    assert!(matches!(r, Err(DbError::FiscalYearClosed)), "obtenu {r:?}");
}

// ---------------------------------------------------------------------------
// 4. L'angle mort assumé des prédicteurs (AC 18, C120, #568)
// ---------------------------------------------------------------------------

/// AC 18 · C120 — **angle mort assumé, fixé par écrit** : dans l'état hérité où
/// l'exercice du jour est suivi d'un exercice clos (un exercice futur clôturé
/// d'avance), `settlement_entry_cancel_blocker` — la queue commune des quatre
/// prédicteurs d'annulation (`cancelBlockedBy`, `settlementCancelBlockedBy`) —
/// rend `None` : l'écran annonce l'annulation possible. Le clic, lui, est
/// refusé par le filet (la contre-passation datée du jour).
///
/// ⚠️ Patron C-15-8-29 : ce test **rougira le jour où le rang sera ajouté**
/// (issue #568). Il faudra alors l'inverser — c'est voulu.
#[sqlx::test(migrations = "./test-schema")]
async fn predicteur_muet_sous_un_exercice_futur_clos(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.expect("seed");
    let e = journal_entries::create(
        &pool,
        s.fiscal_year_id,
        s.admin_user_id,
        corps(&s, Utc::now().date_naive()),
    )
    .await
    .expect("écriture");
    let futur = exercice(&pool, &s, 2031, "Closed").await;

    let mut conn = pool.acquire().await.unwrap();
    let annonce = settlement_cancellation::settlement_entry_cancel_blocker(
        &mut conn,
        s.company_id,
        e.entry.id,
        None,
    )
    .await
    .expect("lecture");
    drop(conn);
    assert!(
        annonce.is_none(),
        "angle mort assumé (#568) : le prédicteur ne voit pas l'exercice futur clos — obtenu {annonce:?}"
    );

    let mut tx = pool.begin().await.unwrap();
    let r =
        journal_entries::reverse_in_tx(&mut tx, s.company_id, e.entry.id, s.admin_user_id).await;
    drop(tx);
    assert_posterieur_clos(r, futur, "Exercice 2031");
}
