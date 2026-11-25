//! Story 48-12a (#975, D3, AC-8) — le verrou du fournisseur de la règle
//! anti-doublon, par des tests **pilotés** (patron
//! `supplier_invoices_draft.rs`, `deleting_a_draft_during_an_attachment_does_not_deadlock`) :
//! A tient sa transaction ouverte, B part dans une tâche **sans enveloppe de
//! rejeu**, l'attente de B se prouve par `attendre_une_requete_en_cours`
//! (`PROCESSLIST` filtrée sur la base du test, sans privilège), puis A conclut.
//!
//! - création, modification, dévalidation d'une annulée : B attend le verrou du
//!   fournisseur pris par A, puis, A commité, **rend le refus** — sans le
//!   verrou, ou avec un verrou pris après la première lecture ordinaire, B ne
//!   voit pas la facture de A et le doublon passe ;
//! - `PUT` et dévalidation de la même facture : sérialisés sur la facture,
//!   jamais d'interblocage ;
//! - création d'un avoir et dévalidation d'une annulée au même contact : aucun
//!   interblocage par l'exercice de l'achat (le fournisseur se prend après les
//!   exercices).
//!
//! ⚠️ Les motifs `"FROM contacts WHERE id"` et `"FOR UPDATE"` sont couplés à la
//! forme exacte du SQL de `supplier_invoices::lock_supplier_in_tx`.

#[path = "support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

#[allow(dead_code)]
#[path = "support/jour.rs"]
mod jour;

use chrono::NaiveDate;
use fournisseur_rapproche::{Fournisseur, IBAN_CREANCIER};
use kesh_db::entities::{Acteur, NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::supplier_invoices::{self, DuplicatePolicy};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;
use std::time::Duration;

const BORNE: Duration = Duration::from_secs(10);
const MOTIFS_FOURNISSEUR: &[&str] = &["FROM contacts WHERE id", "FOR UPDATE"];

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

async fn decor(pool: &MySqlPool) -> (SeededCompany, Fournisseur) {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let f = fournisseur_rapproche::preparer(pool, &seeded).await;
    (seeded, f)
}

fn facture(
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: Option<&str>,
    date: NaiveDate,
    montant: Decimal,
) -> NewSupplierInvoice {
    NewSupplierInvoice {
        company_id: seeded.company_id,
        contact_id: f.supplier_id,
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

/// A : une transaction qui a créé la facture `numero` sans commiter — elle
/// tient le verrou du fournisseur.
async fn a_cree_sans_commiter<'c>(
    pool: &'c MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
) -> sqlx::Transaction<'c, sqlx::MySql> {
    let mut a = pool.begin().await.unwrap();
    supplier_invoices::create_draft_in_tx_with_policy(
        &mut a,
        facture(seeded, f, Some(numero), jour(2026, 6, 1), dec!(10)),
        seeded.admin_user_id,
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("A crée");
    a
}

/// Attend que B soit vu en attente du verrou du fournisseur (asserté), puis
/// commite A et rend l'issue de B.
async fn attendre_puis_commiter<T: Send + 'static>(
    pool: &MySqlPool,
    a: sqlx::Transaction<'_, sqlx::MySql>,
    b: tokio::task::JoinHandle<Result<T, DbError>>,
) -> Result<T, DbError> {
    let attend = attendre_une_requete_en_cours(pool, MOTIFS_FOURNISSEUR, || b.is_finished()).await;
    assert!(
        attend,
        "B doit attendre le verrou du fournisseur tenu par A"
    );
    a.commit().await.unwrap();
    tokio::time::timeout(BORNE, b)
        .await
        .expect("B conclut")
        .expect("tâche B")
}

/// AC-8 — création : B attend le fournisseur, puis refuse le doublon que A a
/// commité. Tue « verrou retiré » (B n'attend pas : `false`) et « verrou après
/// la première lecture ordinaire » (B ne voit pas A : B passe).
#[sqlx::test(migrations = "./test-schema")]
async fn concurrent_creation_waits_for_the_supplier_then_refuses(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let a = a_cree_sans_commiter(&pool, &seeded, f, "L-1").await;
    let (p, new, u) = (
        pool.clone(),
        facture(&seeded, f, Some("L-1"), jour(2026, 7, 1), dec!(20)),
        seeded.admin_user_id,
    );
    let b = tokio::spawn(async move {
        supplier_invoices::create_draft_with_policy(&p, new, u, DuplicatePolicy::Refuse).await
    });
    let err = attendre_puis_commiter(&pool, a, b)
        .await
        .expect_err("doublon");
    assert!(
        matches!(err, DbError::SupplierInvoiceDuplicate { .. }),
        "{err:?}"
    );
}

/// AC-8 — modification d'un autre brouillon vers le numéro de A : B attend le
/// fournisseur, puis refuse. Tue « `update_draft_in_tx_with_policy` appelle
/// `lock_draft_in_tx` seul ».
#[sqlx::test(migrations = "./test-schema")]
async fn concurrent_update_waits_for_the_supplier_then_refuses(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let autre = supplier_invoices::create_draft(
        &pool,
        facture(&seeded, f, Some("L-0"), jour(2026, 7, 1), dec!(20)),
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    let a = a_cree_sans_commiter(&pool, &seeded, f, "L-2").await;
    let (p, new, u, c, id, v) = (
        pool.clone(),
        facture(&seeded, f, Some("L-2"), jour(2026, 7, 1), dec!(20)),
        seeded.admin_user_id,
        seeded.company_id,
        autre.invoice.id,
        autre.invoice.version,
    );
    let b = tokio::spawn(async move {
        let mut tx = p.begin().await.unwrap();
        let r = supplier_invoices::update_draft_in_tx_with_policy(
            &mut tx,
            c,
            id,
            v,
            new,
            u,
            DuplicatePolicy::Refuse,
        )
        .await;
        if r.is_ok() {
            tx.commit().await.unwrap();
        }
        r
    });
    let err = attendre_puis_commiter(&pool, a, b)
        .await
        .expect_err("doublon");
    assert!(
        matches!(err, DbError::SupplierInvoiceDuplicate { .. }),
        "{err:?}"
    );
}

/// Une facture validée, puis annulée ; rend `(id, version)`.
async fn annulee(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: Fournisseur,
    numero: &str,
) -> (i64, i32) {
    let v = supplier_invoices::create_validated(
        pool,
        facture(seeded, f, Some(numero), jour(2026, 3, 1), dec!(30)),
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    let x = supplier_invoices::cancel(
        pool,
        seeded.company_id,
        v.invoice.id,
        seeded.admin_user_id,
        crate::jour::aujourd_hui_test(),
    )
    .await
    .unwrap();
    (x.invoice.id, x.invoice.version)
}

/// AC-8 — dévalidation d'une annulée au numéro de A : B attend le fournisseur,
/// puis refuse ; l'annulée reste annulée.
#[sqlx::test(migrations = "./test-schema")]
async fn concurrent_unvalidation_waits_for_the_supplier_then_refuses(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let (id, v) = annulee(&pool, &seeded, f, "L-3").await;
    let a = a_cree_sans_commiter(&pool, &seeded, f, "L-3").await;
    let (p, c, u) = (pool.clone(), seeded.company_id, seeded.admin_user_id);
    let b = tokio::spawn(async move {
        supplier_invoices::unvalidate_with_policy(
            &p,
            c,
            id,
            v,
            Acteur::utilisateur(u),
            crate::jour::aujourd_hui_test(),
            DuplicatePolicy::Refuse,
        )
        .await
    });
    let err = attendre_puis_commiter(&pool, a, b)
        .await
        .expect_err("doublon");
    assert!(
        matches!(err, DbError::SupplierInvoiceDuplicate { .. }),
        "{err:?}"
    );
    let s: String = sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(s, "cancelled");
}

/// AC-8, V2-5 — un `PUT` et une dévalidation de la **même** facture se
/// sérialisent sur la facture, sans interblocage. A rejoue la tête de la
/// dévalidation d'une annulée (facture `FOR UPDATE`) ; B = modification de
/// cette facture (même fournisseur), sans enveloppe, attend la facture ; A
/// prend alors le fournisseur et commite ; B rend `IllegalStateTransition`,
/// jamais l'erreur 1213. Tue « fournisseur avant la facture au `PUT` ».
#[sqlx::test(migrations = "./test-schema")]
async fn put_and_unvalidation_of_the_same_invoice_do_not_deadlock(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (id, v) = annulee(&pool, &seeded, f, "L-4").await;
    let mut a = pool.begin().await.unwrap();
    let _: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(c)
    .fetch_optional(&mut *a)
    .await
    .unwrap();
    let (p, new, u) = (
        pool.clone(),
        facture(&seeded, f, Some("L-4-b"), jour(2026, 3, 1), dec!(30)),
        seeded.admin_user_id,
    );
    let b = tokio::spawn(async move {
        let mut tx = p.begin().await.unwrap();
        let r = supplier_invoices::update_draft_in_tx_with_policy(
            &mut tx,
            c,
            id,
            v,
            new,
            u,
            DuplicatePolicy::Refuse,
        )
        .await;
        let _ = tx.rollback().await;
        r
    });
    let attend = attendre_une_requete_en_cours(
        &pool,
        &["FROM supplier_invoices WHERE id", "FOR UPDATE"],
        || b.is_finished(),
    )
    .await;
    assert!(attend, "B doit attendre la facture tenue par A");
    let _: Option<i64> =
        sqlx::query_scalar("SELECT id FROM contacts WHERE id = ? AND company_id = ? FOR UPDATE")
            .bind(f.supplier_id)
            .bind(c)
            .fetch_optional(&mut *a)
            .await
            .expect("A obtient le fournisseur : aucun interblocage");
    a.commit().await.unwrap();
    let err = tokio::time::timeout(BORNE, b)
        .await
        .expect("B conclut")
        .expect("tâche B")
        .expect_err("la facture n'est pas un brouillon");
    assert!(matches!(err, DbError::IllegalStateTransition(_)), "{err:?}");
}

/// AC-8, relevé A ligne 7 — la création d'un avoir tient l'exercice de l'achat
/// (`find_open_covering_date`, `FOR UPDATE`) puis demande le contact en partage
/// (clé étrangère de son `INSERT`). A rejoue cela : il tient l'exercice ; B =
/// dévalidation d'une annulée du même contact, sans enveloppe, attend
/// l'exercice à l'étape (3) ; A demande le contact `LOCK IN SHARE MODE` et
/// **l'obtient** (B ne le tient pas encore), puis annule ; B aboutit, jamais
/// l'erreur 1213. Tue « fournisseur verrouillé aussitôt après l'étape (1) ».
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_and_unvalidation_do_not_deadlock_on_the_purchase_year(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (id, v) = annulee(&pool, &seeded, f, "L-5").await;
    let mut a = pool.begin().await.unwrap();
    let _: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(seeded.fiscal_year_id)
    .bind(c)
    .fetch_optional(&mut *a)
    .await
    .unwrap();
    let (p, u) = (pool.clone(), seeded.admin_user_id);
    let b = tokio::spawn(async move {
        supplier_invoices::unvalidate_with_policy(
            &p,
            c,
            id,
            v,
            Acteur::utilisateur(u),
            crate::jour::aujourd_hui_test(),
            DuplicatePolicy::Refuse,
        )
        .await
    });
    let attend =
        attendre_une_requete_en_cours(&pool, &["JOIN fiscal_years fy", "FOR UPDATE"], || {
            b.is_finished()
        })
        .await;
    assert!(attend, "B doit attendre l'exercice tenu par A");
    let _: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM contacts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE",
    )
    .bind(f.supplier_id)
    .bind(c)
    .fetch_optional(&mut *a)
    .await
    .expect("A obtient le contact : B ne le tient pas encore");
    a.rollback().await.unwrap();
    let vue = tokio::time::timeout(BORNE, b)
        .await
        .expect("B conclut")
        .expect("tâche B")
        .expect("B aboutit, sans interblocage");
    assert_eq!(vue.invoice.status, "draft");
}
