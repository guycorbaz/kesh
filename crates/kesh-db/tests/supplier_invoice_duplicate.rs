//! Story 48-12a (#975) — le refus des factures fournisseur en double, au niveau
//! du dépôt (`kesh_db::repositories::supplier_invoices`).
//!
//! - règle 1 : même fournisseur, même numéro (normalisé), même **année civile**
//!   → `SupplierInvoiceDuplicate`, sans sortie à la saisie (AC-1, AC-2) ;
//! - règle 2 : même fournisseur, même date, même total TTC, l'une au moins sans
//!   numéro → `SupplierInvoiceProbableDuplicate`, levé par `Confirmed` et inscrit
//!   à l'audit (AC-3, AC-4) ;
//! - règle 4 : les annulées ne comptent pas (AC-5) ;
//! - D6 : la modification ne rejoue la règle que si l'un des quatre éléments
//!   change (AC-6) ;
//! - D6bis : la dévalidation d'une annulée (AC-7), sa précédence (AC-9).
//!
//! Décor synthétique : `seed_accounting_company` et `fournisseur_rapproche`.

#[path = "support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

#[allow(dead_code)]
#[path = "support/jour.rs"]
mod jour;

use chrono::NaiveDate;
use fournisseur_rapproche::{Fournisseur, IBAN_CREANCIER};
use kesh_db::entities::{Acteur, NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::errors::{DbError, DuplicateGesture, ExistingSupplierInvoice};
use kesh_db::repositories::supplier_invoices::{self, DuplicatePolicy};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

const REFUSE: DuplicatePolicy = DuplicatePolicy::Refuse;
const CONFIRMED: DuplicatePolicy = DuplicatePolicy::Confirmed;

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

async fn decor(pool: &MySqlPool) -> (SeededCompany, Fournisseur) {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let f = fournisseur_rapproche::preparer(pool, &seeded).await;
    (seeded, f)
}

/// Une facture d'une ligne, sans TVA, de `montant`, au fournisseur `contact`.
fn facture(
    seeded: &SeededCompany,
    contact: i64,
    numero: Option<&str>,
    date: NaiveDate,
    montant: Decimal,
) -> NewSupplierInvoice {
    NewSupplierInvoice {
        company_id: seeded.company_id,
        contact_id: contact,
        supplier_invoice_number: numero.map(String::from),
        invoice_date: date,
        due_date: None,
        creditor_iban: Some(IBAN_CREANCIER.into()),
        creditor_qr_iban: None,
        payment_reference: None,
        expected_payment_amount: None,
        project_id: None,
        lines: vec![NewSupplierInvoiceLine {
            description: "Fourniture".into(),
            quantity: dec!(1),
            unit_price: montant,
            vat_rate: dec!(0),
            expense_account_id: seeded.accounts["4000"],
        }],
    }
}

async fn creer(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    new: NewSupplierInvoice,
    policy: DuplicatePolicy,
) -> Result<supplier_invoices::SupplierInvoiceWithLines, DbError> {
    supplier_invoices::create_draft_with_policy(pool, new, seeded.admin_user_id, policy).await
}

async fn brouillon(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: Option<&str>,
    date: NaiveDate,
    montant: Decimal,
) -> supplier_invoices::SupplierInvoiceWithLines {
    creer(
        pool,
        seeded,
        facture(seeded, f.supplier_id, numero, date, montant),
        REFUSE,
    )
    .await
    .expect("brouillon")
}

async fn un_autre_fournisseur(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', 'Autre fournisseur SA', FALSE, TRUE)",
    )
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .expect("autre fournisseur")
    .last_insert_id() as i64
}

async fn nb_factures(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM supplier_invoices WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn confirmations(pool: &MySqlPool, entity_id: i64) -> Vec<(serde_json::Value, Option<i64>)> {
    sqlx::query_as(
        "SELECT details_json, actor_api_key_id FROM audit_log \
         WHERE action = 'supplier_invoice.duplicate_confirmed' AND entity_id = ? ORDER BY id",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

fn existante(id: i64, interne: Option<&str>, statut: &str) -> ExistingSupplierInvoice {
    ExistingSupplierInvoice {
        id,
        internal_number: interne.map(String::from),
        status: statut.into(),
    }
}

/// Une facture validée de `montant`, datée `date`, puis annulée ; rend
/// `(id, version)`.
async fn annulee(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: Option<&str>,
    date: NaiveDate,
    montant: Decimal,
) -> (i64, i32) {
    let v = supplier_invoices::create_validated(
        pool,
        facture(seeded, f.supplier_id, numero, date, montant),
        seeded.admin_user_id,
    )
    .await
    .expect("validée");
    let x = supplier_invoices::cancel(
        pool,
        seeded.company_id,
        v.invoice.id,
        seeded.admin_user_id,
        crate::jour::aujourd_hui_test(),
    )
    .await
    .expect("annulée");
    (x.invoice.id, x.invoice.version)
}

async fn repasser(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    (id, version): (i64, i32),
    acteur: Acteur,
    policy: DuplicatePolicy,
) -> Result<supplier_invoices::SupplierInvoiceSettlementView, DbError> {
    supplier_invoices::unvalidate_with_policy(
        pool,
        seeded.company_id,
        id,
        version,
        acteur,
        crate::jour::aujourd_hui_test(),
        policy,
    )
    .await
}

async fn statut(pool: &MySqlPool, id: i64) -> String {
    sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// Règle 1 (AC-1)
// ---------------------------------------------------------------------------

/// AC-1 — même fournisseur, même numéro normalisé, même année : refus, `details`
/// de l'existante, `Confirmed` sans effet ; rien n'est écrit. Autre fournisseur,
/// autre société : accepté. Tue « filtre `contact_id` omis », « filtre société
/// omis », « `Confirmed` lève la règle 1 à la création ».
#[sqlx::test(migrations = "./test-schema")]
async fn rule_1_same_number_same_year_is_refused_without_exit(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let a = brouillon(
        &pool,
        &seeded,
        f,
        Some("FA 2026-1"),
        jour(2026, 3, 1),
        dec!(100),
    )
    .await;
    let avant = nb_factures(&pool, c).await;
    for (numero, policy) in [
        ("FA 2026-1", REFUSE),
        ("fa2026-1", REFUSE),
        ("FA\u{A0}2026-1", REFUSE),
        (" FA2026-1\u{200B}", CONFIRMED),
    ] {
        let err = creer(
            &pool,
            &seeded,
            facture(
                &seeded,
                f.supplier_id,
                Some(numero),
                jour(2026, 9, 9),
                dec!(5),
            ),
            policy,
        )
        .await
        .expect_err(numero);
        assert_eq!(
            format!("{err:?}"),
            format!(
                "{:?}",
                DbError::SupplierInvoiceDuplicate {
                    existing: existante(a.invoice.id, None, "draft"),
                    gesture: DuplicateGesture::Entry,
                }
            ),
            "{numero:?} {policy:?}"
        );
        assert_eq!(err.error_code(), "SUPPLIER_INVOICE_DUPLICATE");
    }
    assert_eq!(nb_factures(&pool, c).await, avant, "rien n'est écrit");

    // Autre fournisseur : accepté.
    let autre = un_autre_fournisseur(&pool, &seeded).await;
    creer(
        &pool,
        &seeded,
        facture(
            &seeded,
            autre,
            Some("FA 2026-1"),
            jour(2026, 3, 1),
            dec!(100),
        ),
        REFUSE,
    )
    .await
    .expect("autre fournisseur");

    // Autre société, même contact (posée par SQL) : ne compte pas.
    let societe = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 2', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, status, \
         invoice_date, total_amount) VALUES (?, ?, 'X-1', 'draft', '2026-04-04', 77)",
    )
    .bind(societe)
    .bind(f.supplier_id)
    .execute(&pool)
    .await
    .unwrap();
    creer(
        &pool,
        &seeded,
        facture(
            &seeded,
            f.supplier_id,
            Some("X-1"),
            jour(2026, 4, 4),
            dec!(77),
        ),
        REFUSE,
    )
    .await
    .expect("une facture d'une autre société ne compte pas");
}

/// AC-1 — la facture citée est la plus petite en identifiant, et une validée
/// porte son numéro interne.
#[sqlx::test(migrations = "./test-schema")]
async fn rule_1_cites_the_smallest_id_with_its_internal_number(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let (id_a, _) = fournisseur_rapproche::facture_ouverte(
        &pool,
        &seeded,
        f,
        Some("N-7"),
        None,
        jour(2026, 2, 2),
        None,
        dec!(10),
    )
    .await;
    let interne: Option<String> =
        sqlx::query_scalar("SELECT internal_number FROM supplier_invoices WHERE id = ?")
            .bind(id_a)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(interne.is_some(), "une validée a un numéro interne");
    let err = creer(
        &pool,
        &seeded,
        facture(
            &seeded,
            f.supplier_id,
            Some("n-7"),
            jour(2026, 12, 1),
            dec!(99),
        ),
        REFUSE,
    )
    .await
    .expect_err("doublon");
    match err {
        DbError::SupplierInvoiceDuplicate { existing, .. } => {
            assert_eq!(existing, existante(id_a, interne.as_deref(), "open"));
        }
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Année civile (AC-2)
// ---------------------------------------------------------------------------

async fn essai(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    d1: NaiveDate,
    d2: NaiveDate,
) -> bool {
    let n = format!("Y-{}-{}", d1, d2);
    brouillon(pool, seeded, f, Some(&n), d1, dec!(1)).await;
    creer(
        pool,
        seeded,
        facture(seeded, f.supplier_id, Some(&n), d2, dec!(2)),
        REFUSE,
    )
    .await
    .is_ok()
}

/// AC-2 — le numéro se compare dans l'**année civile** de la date de facture,
/// jamais par exercice : 15 de 2026 et 15 de 2027 acceptées ; deux 15 de 2026
/// refusées de part et d'autre d'une limite d'exercice (01.07–30.06) ; 31.12 et
/// 01.01 acceptées ; 01.01 et 31.12 de la même année refusées ; sans aucun
/// exercice, même résultat. Tue « toute l'histoire », « par exercice »,
/// « l'année de l'échéance », « bornes décalées d'un jour ».
#[sqlx::test(migrations = "./test-schema")]
async fn rule_1_compares_within_the_calendar_year(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    // Deux exercices de part et d'autre du 1er juillet 2026.
    sqlx::query("UPDATE fiscal_years SET end_date = '2026-06-30' WHERE company_id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2026-2027', '2026-07-01', '2027-06-30', 'Open')",
    )
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    let cas = [
        (jour(2026, 5, 5), jour(2027, 5, 5), true),
        (jour(2026, 3, 15), jour(2026, 9, 15), false),
        (jour(2026, 12, 31), jour(2027, 1, 1), true),
        (jour(2026, 1, 1), jour(2026, 12, 31), false),
        (jour(2027, 1, 1), jour(2026, 12, 31), true),
        // Borne haute comprise : l'existante du 31.12 compte pour une facture
        // de la même année (lecture bornée à l'année civile, revue P1 B2).
        (jour(2026, 12, 31), jour(2026, 6, 1), false),
    ];
    for (d1, d2, accepte) in cas {
        assert_eq!(
            essai(&pool, &seeded, f, d1, d2).await,
            accepte,
            "{d1} / {d2}"
        );
    }
    // L'échéance n'entre pas dans la règle : même année de facture, échéances
    // de deux années différentes → refus.
    brouillon(&pool, &seeded, f, Some("ECH"), jour(2026, 12, 1), dec!(1)).await;
    let mut n = facture(
        &seeded,
        f.supplier_id,
        Some("ECH"),
        jour(2026, 12, 2),
        dec!(3),
    );
    n.due_date = Some(jour(2027, 1, 31));
    assert!(
        creer(&pool, &seeded, n, REFUSE).await.is_err(),
        "année de l'échéance"
    );

    // Sans aucun exercice : même résultat.
    sqlx::query("DELETE FROM fiscal_years WHERE company_id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    for (d1, d2, accepte) in cas {
        let d1 = d1.with_year_offset(10);
        let d2 = d2.with_year_offset(10);
        assert_eq!(
            essai(&pool, &seeded, f, d1, d2).await,
            accepte,
            "sans exercice {d1} / {d2}"
        );
    }
}

trait Decale {
    fn with_year_offset(self, n: i32) -> NaiveDate;
}
impl Decale for NaiveDate {
    fn with_year_offset(self, n: i32) -> NaiveDate {
        use chrono::Datelike;
        self.with_year(self.year() + n).unwrap()
    }
}

// ---------------------------------------------------------------------------
// Règles 2 et 3 (AC-3, AC-4)
// ---------------------------------------------------------------------------

/// AC-3 — sans numéro à comparer, même date et même total : refus probable ;
/// `Confirmed` passe et écrit l'audit (`rule: 2`) dans la même transaction ;
/// date ou total différents : accepté ; `Confirmed` sans doublon n'écrit rien.
/// Tue « total omis », « date omise », « audit non écrit ».
#[sqlx::test(migrations = "./test-schema")]
async fn rule_2_same_date_same_total_is_confirmable_and_audited(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let a = brouillon(&pool, &seeded, f, None, jour(2026, 4, 1), dec!(120.50)).await;
    let err = creer(
        &pool,
        &seeded,
        facture(&seeded, f.supplier_id, None, jour(2026, 4, 1), dec!(120.5)),
        REFUSE,
    )
    .await
    .expect_err("doublon probable");
    assert_eq!(
        format!("{err:?}"),
        format!(
            "{:?}",
            DbError::SupplierInvoiceProbableDuplicate {
                existing: existante(a.invoice.id, None, "draft"),
                gesture: DuplicateGesture::Entry,
            }
        )
    );
    assert_eq!(err.error_code(), "SUPPLIER_INVOICE_PROBABLE_DUPLICATE");

    // Date différente, total différent : acceptés.
    let d = brouillon(&pool, &seeded, f, None, jour(2026, 4, 2), dec!(120.50)).await;
    let t = brouillon(&pool, &seeded, f, None, jour(2026, 4, 1), dec!(120.55)).await;
    assert!(confirmations(&pool, d.invoice.id).await.is_empty());
    assert!(confirmations(&pool, t.invoice.id).await.is_empty());

    // Confirmé : enregistré, audit `rule: 2`.
    let b = creer(
        &pool,
        &seeded,
        facture(&seeded, f.supplier_id, None, jour(2026, 4, 1), dec!(120.50)),
        CONFIRMED,
    )
    .await
    .expect("confirmé");
    let audits = confirmations(&pool, b.invoice.id).await;
    assert_eq!(audits.len(), 1, "une entrée d'audit");
    assert_eq!(
        audits[0].0,
        serde_json::json!({ "supplierInvoiceId": a.invoice.id, "internalNumber": null, "rule": 2 })
    );

    // `Confirmed` sans doublon : rien de plus.
    let s = creer(
        &pool,
        &seeded,
        facture(&seeded, f.supplier_id, None, jour(2026, 8, 8), dec!(1)),
        CONFIRMED,
    )
    .await
    .expect("sans doublon");
    assert!(confirmations(&pool, s.invoice.id).await.is_empty());
}

/// Revue P1 (B2) — la lecture des candidates est bornée à l'année civile : la
/// règle 2 tient encore aux deux bornes, le 1er janvier et le 31 décembre.
/// Tue « borne basse exclue », « borne haute exclue », « année décalée ».
#[sqlx::test(migrations = "./test-schema")]
async fn rule_2_holds_on_both_bounds_of_the_calendar_year(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    for date in [jour(2026, 1, 1), jour(2026, 12, 31)] {
        let a = brouillon(&pool, &seeded, f, None, date, dec!(33)).await;
        let err = creer(
            &pool,
            &seeded,
            facture(&seeded, f.supplier_id, None, date, dec!(33)),
            REFUSE,
        )
        .await
        .expect_err("doublon probable à la borne");
        assert!(
            matches!(err, DbError::SupplierInvoiceProbableDuplicate { ref existing, .. } if existing.id == a.invoice.id),
            "{date} : {err:?}"
        );
    }
}

/// AC-4 — deux numéros différents, même date et même total : jamais un
/// doublon ; un numéro et un absent : refus probable (les deux sens). Tue
/// « règle 2 appliquée à deux numéros ».
#[sqlx::test(migrations = "./test-schema")]
async fn rule_3_only_the_number_decides_when_both_have_one(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let a1 = brouillon(&pool, &seeded, f, Some("A-1"), jour(2026, 5, 1), dec!(50)).await;
    brouillon(&pool, &seeded, f, Some("A-2"), jour(2026, 5, 1), dec!(50)).await;
    // Un blanc seul vaut absent.
    let err = creer(
        &pool,
        &seeded,
        facture(
            &seeded,
            f.supplier_id,
            Some(" \u{200B}"),
            jour(2026, 5, 1),
            dec!(50),
        ),
        REFUSE,
    )
    .await
    .expect_err("sans numéro contre numéroté");
    // Deux candidates de la règle 2 (A-1, A-2) : la plus petite en identifiant
    // est citée. Tue « `max_by_key` à la règle 2 ».
    assert!(
        matches!(err, DbError::SupplierInvoiceProbableDuplicate { ref existing, .. } if existing.id == a1.invoice.id),
        "{err:?}"
    );
    brouillon(&pool, &seeded, f, None, jour(2026, 6, 1), dec!(60)).await;
    let err = creer(
        &pool,
        &seeded,
        facture(
            &seeded,
            f.supplier_id,
            Some("B-9"),
            jour(2026, 6, 1),
            dec!(60),
        ),
        REFUSE,
    )
    .await
    .expect_err("numéroté contre sans numéro");
    assert!(
        matches!(err, DbError::SupplierInvoiceProbableDuplicate { .. }),
        "{err:?}"
    );
}

// ---------------------------------------------------------------------------
// Règle 4 (AC-5)
// ---------------------------------------------------------------------------

/// AC-5 — une annulée ne compte pas (numéro, date et total) ; brouillon,
/// validée et payée comptent. Tue « `status <> 'cancelled'` retiré ».
#[sqlx::test(migrations = "./test-schema")]
async fn rule_4_cancelled_invoices_do_not_count(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    annulee(&pool, &seeded, f, Some("C-1"), jour(2026, 2, 1), dec!(30)).await;
    annulee(&pool, &seeded, f, None, jour(2026, 2, 2), dec!(31)).await;
    brouillon(&pool, &seeded, f, Some("C-1"), jour(2026, 7, 1), dec!(1)).await;
    brouillon(&pool, &seeded, f, None, jour(2026, 2, 2), dec!(31)).await;

    // Validée et payée comptent.
    fournisseur_rapproche::facture_ouverte(
        &pool,
        &seeded,
        f,
        Some("O-1"),
        None,
        jour(2026, 3, 3),
        None,
        dec!(5),
    )
    .await;
    fournisseur_rapproche::facture_payee(&pool, &seeded, f, "P-1", jour(2026, 3, 4), dec!(6)).await;
    for n in ["O-1", "P-1"] {
        let err = creer(
            &pool,
            &seeded,
            facture(&seeded, f.supplier_id, Some(n), jour(2026, 10, 10), dec!(9)),
            REFUSE,
        )
        .await
        .expect_err(n);
        assert!(
            matches!(err, DbError::SupplierInvoiceDuplicate { .. }),
            "{n} {err:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Modification (AC-6, D6)
// ---------------------------------------------------------------------------

async fn modifier(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    d: &supplier_invoices::SupplierInvoiceWithLines,
    new: NewSupplierInvoice,
    policy: DuplicatePolicy,
) -> Result<supplier_invoices::SupplierInvoiceWithLines, DbError> {
    let mut tx = pool.begin().await.unwrap();
    let r = supplier_invoices::update_draft_in_tx_with_policy(
        &mut tx,
        seeded.company_id,
        d.invoice.id,
        d.invoice.version,
        new,
        seeded.admin_user_id,
        policy,
    )
    .await;
    if r.is_ok() {
        tx.commit().await.unwrap();
    }
    r
}

/// AC-6 — la modification exclut la facture elle-même ; un changement de
/// fournisseur, de numéro, de date ou de total rejoue la règle ; un changement
/// de date d'un doublon préexistant, dans la même année, est refusé ; sans
/// changement des quatre, la modification passe même si la facture est déjà le
/// double d'une autre. Tue « `id <> ?` retiré », « règle toujours rejouée »,
/// « règle jamais rejouée ».
#[sqlx::test(migrations = "./test-schema")]
async fn update_replays_the_rule_only_when_one_of_the_four_changes(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    // La facture elle-même ne compte pas : la modifier sans rien changer passe.
    let a = brouillon(&pool, &seeded, f, Some("M-1"), jour(2026, 3, 1), dec!(10)).await;
    let mut n = facture(
        &seeded,
        f.supplier_id,
        Some("M-1"),
        jour(2026, 3, 1),
        dec!(10),
    );
    n.due_date = Some(jour(2026, 4, 1));
    let a = modifier(&pool, &seeded, &a, n, REFUSE)
        .await
        .expect("elle-même exclue");
    // Changer la date dans la même année rejoue la règle — seule, elle passe.
    let a = modifier(
        &pool,
        &seeded,
        &a,
        facture(
            &seeded,
            f.supplier_id,
            Some("M-1"),
            jour(2026, 3, 2),
            dec!(10),
        ),
        REFUSE,
    )
    .await
    .expect("seule de son numéro");

    // Un doublon préexistant (posé par SQL, comme avant la 0.20.0).
    let b_id = sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, status, \
         invoice_date, total_amount) VALUES (?, ?, 'M-1', 'draft', '2026-03-02', 10)",
    )
    .bind(c)
    .bind(f.supplier_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO supplier_invoice_lines (supplier_invoice_id, position, description, quantity, \
         unit_price, vat_rate, line_total, expense_account_id) VALUES (?, 0, 'x', 1, 10, 0, 10, ?)",
    )
    .bind(b_id)
    .bind(seeded.accounts["4000"])
    .execute(&pool)
    .await
    .unwrap();
    let (invoice, lines) = supplier_invoices::get(&pool, c, b_id)
        .await
        .unwrap()
        .unwrap();
    let b = supplier_invoices::SupplierInvoiceWithLines { invoice, lines };
    // Sans changement des quatre (seule l'échéance change) : passe.
    let mut n = facture(
        &seeded,
        f.supplier_id,
        Some("M-1"),
        jour(2026, 3, 2),
        dec!(10),
    );
    n.due_date = Some(jour(2026, 5, 5));
    let b = modifier(&pool, &seeded, &b, n, REFUSE)
        .await
        .expect("rien des quatre ne change");
    // Le numéro ne change que par la casse et un blanc : **normalisé**, il ne
    // change pas (D6) — passe. Tue « numéro brut comparé dans `key_changed` ».
    let b = modifier(
        &pool,
        &seeded,
        &b,
        facture(
            &seeded,
            f.supplier_id,
            Some("m-1\u{A0}"),
            jour(2026, 3, 2),
            dec!(10),
        ),
        REFUSE,
    )
    .await
    .expect("numéro égal une fois normalisé");
    // Changement de date dans la même année : refusé, sans sortie.
    for policy in [REFUSE, CONFIRMED] {
        let err = modifier(
            &pool,
            &seeded,
            &b,
            facture(
                &seeded,
                f.supplier_id,
                Some("M-1"),
                jour(2026, 3, 9),
                dec!(10),
            ),
            policy,
        )
        .await
        .expect_err("doublon préexistant, date changée");
        assert!(
            matches!(err, DbError::SupplierInvoiceDuplicate { ref existing, .. } if existing.id == a.invoice.id),
            "{err:?}"
        );
    }
    // Changement de total : refusé aussi.
    let err = modifier(
        &pool,
        &seeded,
        &b,
        facture(
            &seeded,
            f.supplier_id,
            Some("M-1"),
            jour(2026, 3, 2),
            dec!(11),
        ),
        REFUSE,
    )
    .await
    .expect_err("total changé");
    assert!(
        matches!(err, DbError::SupplierInvoiceDuplicate { .. }),
        "{err:?}"
    );
    // Changement de numéro vers un numéro libre : passe.
    let b = modifier(
        &pool,
        &seeded,
        &b,
        facture(
            &seeded,
            f.supplier_id,
            Some("M-1-b"),
            jour(2026, 3, 2),
            dec!(10),
        ),
        REFUSE,
    )
    .await
    .expect("15-b");
    // Changement de fournisseur vers un fournisseur qui a déjà ce numéro : refusé.
    let autre = un_autre_fournisseur(&pool, &seeded).await;
    brouillon_de(&pool, &seeded, autre, Some("M-1-b"), jour(2026, 1, 1)).await;
    let err = modifier(
        &pool,
        &seeded,
        &b,
        facture(&seeded, autre, Some("M-1-b"), jour(2026, 3, 2), dec!(10)),
        REFUSE,
    )
    .await
    .expect_err("fournisseur changé");
    assert!(
        matches!(err, DbError::SupplierInvoiceDuplicate { .. }),
        "{err:?}"
    );
    // Règle 2 à la modification : confirmable, audit sur la facture modifiée.
    let p = brouillon(&pool, &seeded, f, None, jour(2026, 11, 1), dec!(70)).await;
    let q = brouillon(&pool, &seeded, f, None, jour(2026, 11, 2), dec!(70)).await;
    let err = modifier(
        &pool,
        &seeded,
        &q,
        facture(&seeded, f.supplier_id, None, jour(2026, 11, 1), dec!(70)),
        REFUSE,
    )
    .await
    .expect_err("probable");
    assert!(
        matches!(err, DbError::SupplierInvoiceProbableDuplicate { .. }),
        "{err:?}"
    );
    modifier(
        &pool,
        &seeded,
        &q,
        facture(&seeded, f.supplier_id, None, jour(2026, 11, 1), dec!(70)),
        CONFIRMED,
    )
    .await
    .expect("confirmé");
    let audits = confirmations(&pool, q.invoice.id).await;
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0].0["supplierInvoiceId"], p.invoice.id);
    assert_eq!(audits[0].0["rule"], 2);
}

async fn brouillon_de(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact: i64,
    numero: Option<&str>,
    date: NaiveDate,
) {
    creer(
        pool,
        seeded,
        facture(seeded, contact, numero, date, dec!(1)),
        REFUSE,
    )
    .await
    .expect("brouillon d'un autre fournisseur");
}

// ---------------------------------------------------------------------------
// Ordre des refus de saisie (AC-9, D4)
// ---------------------------------------------------------------------------

/// AC-9 — les refus de saisie passent avant le doublon, dans l'ordre d'avant la
/// story : aucune ligne, compte non imputable ; fournisseur absent → `NotFound`
/// **après** « aucune ligne » ; un refus de doublon n'écrit aucune ligne ; au
/// `PUT`, facture absente, pas un brouillon et version passent avant. Tue
/// « comparaison avant `check_draft_input_in_tx` » et « `lock_supplier_in_tx`
/// qui rend `NotFound` ».
#[sqlx::test(migrations = "./test-schema")]
async fn entry_refusals_keep_their_precedence(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let a = brouillon(&pool, &seeded, f, Some("P-1"), jour(2026, 1, 10), dec!(10)).await;
    // Aucune ligne, fournisseur absent : « aucune ligne » d'abord.
    let mut n = facture(&seeded, 999_999, Some("P-1"), jour(2026, 1, 10), dec!(10));
    n.lines.clear();
    let err = creer(&pool, &seeded, n, REFUSE)
        .await
        .expect_err("aucune ligne");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");
    // Fournisseur absent, avec lignes : `NotFound`.
    let err = creer(
        &pool,
        &seeded,
        facture(&seeded, 999_999, Some("P-1"), jour(2026, 1, 10), dec!(10)),
        REFUSE,
    )
    .await
    .expect_err("fournisseur absent");
    assert!(matches!(err, DbError::NotFound), "{err:?}");
    // Compte inexistant + doublon : le refus du compte d'abord.
    let mut n = facture(
        &seeded,
        f.supplier_id,
        Some("P-1"),
        jour(2026, 1, 10),
        dec!(10),
    );
    n.lines[0].expense_account_id = seeded.accounts["1000"];
    let err = creer(&pool, &seeded, n, REFUSE).await.expect_err("compte");
    assert!(matches!(err, DbError::InactiveOrInvalidAccounts), "{err:?}");
    // Projet inconnu + doublon : le refus du projet d'abord (revue P1, A6).
    let mut n = facture(
        &seeded,
        f.supplier_id,
        Some("P-1"),
        jour(2026, 1, 10),
        dec!(10),
    );
    n.project_id = Some(999_999);
    let err = creer(&pool, &seeded, n, REFUSE).await.expect_err("projet");
    assert!(matches!(err, DbError::NotFound), "{err:?}");
    // Contact qui n'est plus fournisseur + doublon : ce refus d'abord (A6).
    sqlx::query("UPDATE contacts SET is_supplier = FALSE WHERE id = ?")
        .bind(f.supplier_id)
        .execute(&pool)
        .await
        .unwrap();
    let err = creer(
        &pool,
        &seeded,
        facture(
            &seeded,
            f.supplier_id,
            Some("P-1"),
            jour(2026, 1, 10),
            dec!(10),
        ),
        REFUSE,
    )
    .await
    .expect_err("non fournisseur");
    assert!(
        matches!(err, DbError::IllegalStateTransition(ref m) if m.contains("pas un fournisseur")),
        "{err:?}"
    );
    sqlx::query("UPDATE contacts SET is_supplier = TRUE WHERE id = ?")
        .bind(f.supplier_id)
        .execute(&pool)
        .await
        .unwrap();
    // Refus de doublon : aucune ligne écrite.
    let lignes_avant: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM supplier_invoice_lines")
        .fetch_one(&pool)
        .await
        .unwrap();
    creer(
        &pool,
        &seeded,
        facture(
            &seeded,
            f.supplier_id,
            Some("P-1"),
            jour(2026, 1, 10),
            dec!(10),
        ),
        REFUSE,
    )
    .await
    .expect_err("doublon");
    let lignes_apres: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM supplier_invoice_lines")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(lignes_avant, lignes_apres);
    // Au `PUT` : facture absente, pas un brouillon, version — avant tout refus
    // de fournisseur ou de doublon.
    let b = brouillon(&pool, &seeded, f, Some("P-2"), jour(2026, 1, 11), dec!(10)).await;
    let dup = facture(
        &seeded,
        f.supplier_id,
        Some("P-1"),
        jour(2026, 1, 10),
        dec!(10),
    );
    let mut tx = pool.begin().await.unwrap();
    let err = supplier_invoices::update_draft_in_tx_with_policy(
        &mut tx,
        c,
        999_999,
        1,
        dup.clone(),
        seeded.admin_user_id,
        REFUSE,
    )
    .await
    .expect_err("absente");
    assert!(matches!(err, DbError::NotFound), "{err:?}");
    let err = supplier_invoices::update_draft_in_tx_with_policy(
        &mut tx,
        c,
        b.invoice.id,
        b.invoice.version + 7,
        dup.clone(),
        seeded.admin_user_id,
        REFUSE,
    )
    .await
    .expect_err("version");
    assert!(matches!(err, DbError::OptimisticLockConflict), "{err:?}");
    tx.rollback().await.unwrap();
    let (o, _) = fournisseur_rapproche::facture_ouverte(
        &pool,
        &seeded,
        f,
        Some("P-3"),
        None,
        jour(2026, 1, 12),
        None,
        dec!(3),
    )
    .await;
    let mut tx = pool.begin().await.unwrap();
    let err = supplier_invoices::update_draft_in_tx_with_policy(
        &mut tx,
        c,
        o,
        1,
        facture(&seeded, 999_999, Some("P-1"), jour(2026, 1, 10), dec!(10)),
        seeded.admin_user_id,
        REFUSE,
    )
    .await
    .expect_err("pas un brouillon");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");
    tx.rollback().await.unwrap();
    let _ = a;
}

// ---------------------------------------------------------------------------
// Dévalidation d'une annulée (AC-7, D6bis)
// ---------------------------------------------------------------------------

async fn cle_api(pool: &MySqlPool, seeded: &SeededCompany) -> Acteur {
    let key = sqlx::query(
        "INSERT INTO api_keys (company_id, created_by_user_id, name, key_hash, scope) \
         VALUES (?, ?, 'cle', SHA2('cle-doublon', 256), 'read-write')",
    )
    .bind(seeded.company_id)
    .bind(seeded.admin_user_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    Acteur {
        user_id: seeded.admin_user_id,
        api_key_id: Some(key),
    }
}

/// AC-7 — A (n° 15) annulée, B (n° 15) active : refus `SupplierInvoiceDuplicate`
/// geste `Unvalidation` (confirmable), `details` = B, A reste annulée ; avec
/// `Confirmed`, le repassage passe et l'audit `rule: 1` est au nom de la clé ;
/// puis le brouillon se supprime (administrateur). Tue « dévalidation non
/// contrôlée », « règle 1 refusée malgré `Confirmed` », « audit par `user` ».
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidating_a_cancelled_duplicate_is_refused_then_confirmable(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let qui = Acteur::utilisateur(seeded.admin_user_id);
    let a = annulee(&pool, &seeded, f, Some("15"), jour(2026, 3, 1), dec!(40)).await;
    let b = brouillon(&pool, &seeded, f, Some("15"), jour(2026, 9, 1), dec!(41)).await;
    let err = repasser(&pool, &seeded, a, qui, REFUSE)
        .await
        .expect_err("doublon");
    assert_eq!(
        format!("{err:?}"),
        format!(
            "{:?}",
            DbError::SupplierInvoiceDuplicate {
                existing: existante(b.invoice.id, None, "draft"),
                gesture: DuplicateGesture::Unvalidation,
            }
        )
    );
    assert_eq!(statut(&pool, a.0).await, "cancelled");
    assert!(confirmations(&pool, a.0).await.is_empty());

    let cle = cle_api(&pool, &seeded).await;
    let v = repasser(&pool, &seeded, a, cle, CONFIRMED)
        .await
        .expect("confirmé");
    assert_eq!(v.invoice.status, "draft");
    let audits = confirmations(&pool, a.0).await;
    assert_eq!(audits.len(), 1);
    assert_eq!(
        audits[0].0,
        serde_json::json!({ "supplierInvoiceId": b.invoice.id, "internalNumber": null, "rule": 1 })
    );
    assert_eq!(audits[0].1, cle.api_key_id, "l'acteur est la clé");
    // Le brouillon obtenu se retire (administrateur).
    supplier_invoices::delete_draft(&pool, c, a.0, seeded.admin_user_id)
        .await
        .expect("supprimé");
}

/// AC-7 — règle 2 (l'une sans numéro) : refus probable, levé par `Confirmed`
/// (audit `rule: 2`) ; B annulée ou supprimée : passe sans audit ; une facture
/// **validée** qui repasse en brouillon n'est pas contrôlée. Tue « contrôler
/// aussi une validée ».
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidation_rule_2_other_cancelled_and_validated_invoices(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let qui = Acteur::utilisateur(seeded.admin_user_id);
    let a = annulee(&pool, &seeded, f, None, jour(2026, 3, 1), dec!(40)).await;
    let b = brouillon(&pool, &seeded, f, Some("Z-1"), jour(2026, 3, 1), dec!(40)).await;
    let err = repasser(&pool, &seeded, a, qui, REFUSE)
        .await
        .expect_err("probable");
    assert!(
        matches!(err, DbError::SupplierInvoiceProbableDuplicate { ref existing, gesture: DuplicateGesture::Unvalidation } if existing.id == b.invoice.id),
        "{err:?}"
    );
    repasser(&pool, &seeded, a, qui, CONFIRMED)
        .await
        .expect("confirmé");
    let audits = confirmations(&pool, a.0).await;
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0].0["rule"], 2);
    assert_eq!(audits[0].1, None, "session : sans clé");

    // B supprimée : la seconde annulée repasse sans audit.
    let a2 = annulee(&pool, &seeded, f, Some("Y-1"), jour(2026, 4, 1), dec!(9)).await;
    let b2 = brouillon(&pool, &seeded, f, Some("Y-1"), jour(2026, 4, 2), dec!(9)).await;
    supplier_invoices::delete_draft(&pool, c, b2.invoice.id, seeded.admin_user_id)
        .await
        .unwrap();
    repasser(&pool, &seeded, a2, qui, REFUSE)
        .await
        .expect("B supprimée");
    assert!(confirmations(&pool, a2.0).await.is_empty());
    // B annulée : passe aussi.
    let a3 = annulee(&pool, &seeded, f, Some("X-1"), jour(2026, 5, 1), dec!(8)).await;
    annulee(&pool, &seeded, f, Some("X-1"), jour(2026, 5, 2), dec!(8)).await;
    repasser(&pool, &seeded, a3, qui, REFUSE)
        .await
        .expect("B annulée");

    // Une VALIDÉE qui repasse en brouillon : aucun contrôle, même si un
    // doublon préexistant (posé par SQL) existe.
    let (o, _) = fournisseur_rapproche::facture_ouverte(
        &pool,
        &seeded,
        f,
        Some("W-1"),
        None,
        jour(2026, 6, 1),
        None,
        dec!(7),
    )
    .await;
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, status, \
         invoice_date, total_amount) VALUES (?, ?, 'W-1', 'draft', '2026-06-02', 7)",
    )
    .bind(c)
    .bind(f.supplier_id)
    .execute(&pool)
    .await
    .unwrap();
    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(o)
        .fetch_one(&pool)
        .await
        .unwrap();
    repasser(&pool, &seeded, (o, version), qui, REFUSE)
        .await
        .expect("une validée n'est pas contrôlée");
}

/// AC-9, D4 — précédence de la dévalidation d'une annulée : exercice de
/// l'achat clos **avant** le doublon ; doublon **avant** la période
/// verrouillée (refus de `delete_in_tx`). Tue « contrôler avant l'étape (6) »
/// et « contrôler après l'étape (9) ».
#[sqlx::test(migrations = "./test-schema")]
async fn unvalidation_duplicate_sits_between_closed_year_and_period_lock(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let qui = Acteur::utilisateur(seeded.admin_user_id);
    let a = annulee(&pool, &seeded, f, Some("R-1"), jour(2026, 3, 1), dec!(40)).await;
    brouillon(&pool, &seeded, f, Some("R-1"), jour(2026, 3, 5), dec!(1)).await;

    // Période verrouillée : le doublon d'abord.
    sqlx::query("UPDATE companies SET books_locked_through = '2030-12-31' WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    let err = repasser(&pool, &seeded, a, qui, REFUSE)
        .await
        .expect_err("doublon");
    assert!(
        matches!(err, DbError::SupplierInvoiceDuplicate { .. }),
        "{err:?}"
    );
    // … et confirmé, c'est la période verrouillée qui refuse.
    let err = repasser(&pool, &seeded, a, qui, CONFIRMED)
        .await
        .expect_err("période");
    assert!(matches!(err, DbError::PeriodLocked { .. }), "{err:?}");
    assert!(
        confirmations(&pool, a.0).await.is_empty(),
        "le refus annule l'audit"
    );
    sqlx::query("UPDATE companies SET books_locked_through = NULL WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();

    // Annulée dont l'achat est **rapproché**, et doublon : le rapprochement
    // d'abord, avec ou sans confirmation (revue P1, A5 ; « payée » est vide par
    // construction : une payée n'est pas annulée).
    let r = annulee(&pool, &seeded, f, Some("R-9"), jour(2026, 3, 1), dec!(41)).await;
    brouillon(&pool, &seeded, f, Some("R-9"), jour(2026, 3, 6), dec!(1)).await;
    let achat: i64 =
        sqlx::query_scalar("SELECT purchase_journal_entry_id FROM supplier_invoices WHERE id = ?")
            .bind(r.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    let t = fournisseur_rapproche::transaction_en_attente(
        &pool,
        &seeded,
        f.bank_account_id,
        jour(2026, 3, 2),
        None,
        dec!(-41),
        None,
        None,
    )
    .await;
    fournisseur_rapproche::link_bank_transaction_for_test(&pool, t, achat).await;
    for policy in [REFUSE, CONFIRMED] {
        let err = repasser(&pool, &seeded, r, qui, policy)
            .await
            .expect_err("rapprochée");
        assert_eq!(err.error_code(), "MATCHED_BANK_TRANSACTION", "{err:?}");
    }
    assert_eq!(statut(&pool, r.0).await, "cancelled");

    // Exercice de l'achat clos : son refus avant le doublon.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE company_id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    let err = repasser(&pool, &seeded, a, qui, REFUSE)
        .await
        .expect_err("exercice clos");
    assert!(matches!(err, DbError::FiscalYearClosed), "{err:?}");
    assert_eq!(statut(&pool, a.0).await, "cancelled");
}

// ---------------------------------------------------------------------------
// La décision pure (D2)
// ---------------------------------------------------------------------------

/// D2 — la règle 1 passe avant la règle 2 sur toutes les candidates, et la
/// facture citée est la plus petite en identifiant parmi celles de la règle
/// retenue.
#[test]
fn pure_decision_rule_1_first_then_smallest_id() {
    use supplier_invoices::{DuplicateCandidate, DuplicateRule, find_duplicate};
    let cand = |id, n: Option<&str>, d, t| DuplicateCandidate {
        id,
        internal_number: None,
        supplier_invoice_number: n.map(String::from),
        status: "draft".into(),
        invoice_date: d,
        total_amount: t,
    };
    let d = jour(2026, 1, 1);
    let candidates = [
        cand(3, None, d, dec!(10)),
        cand(9, Some("A"), jour(2026, 12, 31), dec!(1)),
        cand(7, Some(" a "), jour(2026, 6, 1), dec!(2)),
    ];
    let f = find_duplicate(Some("A"), d, dec!(10), &candidates).unwrap();
    assert_eq!(
        (f.rule, f.existing.id),
        (DuplicateRule::SameNumberSameYear, 7)
    );
    let f = find_duplicate(Some("B"), d, dec!(10.000), &candidates).unwrap();
    assert_eq!(
        (f.rule, f.existing.id),
        (DuplicateRule::SameDateSameTotal, 3)
    );
    assert!(find_duplicate(Some("B"), d, dec!(10.01), &candidates).is_none());
}
