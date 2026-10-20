//! Story 30-2c (#685) — joindre un document à une facture fournisseur après
//! coup, le détacher, et le retrouver depuis l'écriture.
//!
//! Tests du dépôt `imported_supplier_invoices` : [`repo::attach_in_tx`],
//! [`repo::detach_in_tx`], [`repo::find_source_document_of_entry`] et le filtre
//! `status = 'completed'` de [`repo::find_by_supplier_invoice_id_scoped`].
//!
//! ⛔ **Les tests de concurrence appellent les fonctions `_in_tx` dans des
//! transactions qu'ils ouvrent et valident eux-mêmes, SANS rejeu** (C-30-2-45) :
//! un interblocage doit se voir, et l'enveloppe `retry_app_on_deadlock` de la
//! route le masquerait. La fonction `_once` de la route n'ajoute à
//! `attach_in_tx` / `detach_in_tx` qu'un `INSERT` dans `audit_log`, qui ne touche
//! aucune des lignes en jeu.
//!
//! ⛔ Aucun test ne lit `information_schema.INNODB_TRX` (privilège `PROCESS`
//! absent de l'utilisateur `kesh`) : les attentes s'observent par
//! `attendre_une_requete_en_cours` (`PROCESSLIST`) et `sonde_verrou_nowait`.
//!
//! Décor **synthétique** (`docs/agents-orchestration.md` § 8).

#[path = "support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

use chrono::NaiveDate;
use kesh_db::entities::imported_supplier_invoice::{
    ImportedSupplierInvoice, NewImportedSupplierInvoice,
};
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::repositories::imported_supplier_invoices::{self as repo, AttachRefusal};
use kesh_db::repositories::supplier_invoices;
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company, sonde_verrou_nowait,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;
use std::time::Duration;

const BORNE: Duration = Duration::from_secs(5);

fn jour() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()
}

/// Un document à traiter (avec QR, synthétique) de la société.
async fn document(pool: &MySqlPool, company_id: i64, hash: &str) -> ImportedSupplierInvoice {
    repo::create(
        pool,
        &NewImportedSupplierInvoice {
            company_id,
            file_hash: hash.into(),
            storage_path: format!("{hash}.png"),
            original_filename: format!("{hash}.png"),
            mime_type: "image/png".into(),
            byte_size: 10,
            creditor_iban: Some("CH9300762011623852957".into()),
            is_qr_iban: Some(false),
            creditor_address_type: Some("K".into()),
            creditor_name: Some("Fournisseur SA".into()),
            creditor_line1: Some("Rue 1".into()),
            creditor_line2: Some("1000 Lausanne".into()),
            creditor_postal_code: None,
            creditor_town: None,
            creditor_country: Some("CH".into()),
            reference_type: Some("NON".into()),
            reference_value: None,
            amount: Some(dec!(100.00)),
            currency: Some("CHF".into()),
            unstructured_message: None,
            billing_information: None,
        },
    )
    .await
    .expect("document à traiter")
}

/// Une facture fournisseur **ouverte** ; rend `(facture, écriture d'achat)`.
async fn facture_ouverte(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    f: fournisseur_rapproche::Fournisseur,
    numero: &str,
    montant: Decimal,
) -> (i64, i64) {
    let created = supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id: f.supplier_id,
            supplier_invoice_number: Some(numero.into()),
            invoice_date: jour(),
            due_date: Some(jour()),
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: montant,
                vat_rate: dec!(0),
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .expect("facture fournisseur");
    (
        created.invoice.id,
        created.invoice.purchase_journal_entry_id,
    )
}

async fn decor(pool: &MySqlPool) -> (SeededCompany, fournisseur_rapproche::Fournisseur) {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let f = fournisseur_rapproche::preparer(pool, &seeded).await;
    (seeded, f)
}

/// Rattache dans une transaction propre, validée — sans rejeu.
async fn rattacher(
    pool: &MySqlPool,
    company_id: i64,
    doc: i64,
    invoice: i64,
) -> Result<ImportedSupplierInvoice, AttachRefusal> {
    let mut tx = pool.begin().await.unwrap();
    let r = repo::attach_in_tx(&mut tx, company_id, doc, invoice)
        .await
        .expect("aucune erreur de base");
    if r.is_ok() {
        tx.commit().await.unwrap();
    } else {
        tx.rollback().await.unwrap();
    }
    r
}

/// Détache dans une transaction propre, validée — sans rejeu.
async fn detacher(
    pool: &MySqlPool,
    company_id: i64,
    invoice: i64,
) -> Option<ImportedSupplierInvoice> {
    let mut tx = pool.begin().await.unwrap();
    let r = repo::detach_in_tx(&mut tx, company_id, invoice)
        .await
        .expect("aucune erreur de base");
    tx.commit().await.unwrap();
    r
}

async fn statut_et_lien(pool: &MySqlPool, doc: i64) -> (String, Option<i64>) {
    sqlx::query_as(
        "SELECT status, supplier_invoice_id FROM imported_supplier_invoices WHERE id = ?",
    )
    .bind(doc)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// L'état comptable d'une facture, pour vérifier qu'aucun geste ne la touche.
async fn etat_facture(pool: &MySqlPool, invoice: i64) -> (String, i32, Option<i64>, Decimal) {
    sqlx::query_as(
        "SELECT status, version, settlement_journal_entry_id, total_amount \
         FROM supplier_invoices WHERE id = ?",
    )
    .bind(invoice)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn nombre_d_ecritures(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn autre_societe(pool: &MySqlPool) -> i64 {
    sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 2', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

// ---------------------------------------------------------------------------
// AC19 — joindre après coup
// ---------------------------------------------------------------------------

/// AC19 — les refus (document inconnu, d'une autre société, retiré ; facture
/// inconnue ou d'une autre société ; facture qui a déjà un document), puis le
/// succès sur une facture **payée** : le document passe `completed`, lié ;
/// la facture et le grand livre ne bougent pas. Tue M-c3 (staging `discarded`
/// accepté) et M-c4 (facture d'une autre société acceptée).
#[sqlx::test(migrations = "./test-schema")]
async fn attach_refusals_then_success_on_a_paid_invoice(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (paid, _, _) =
        fournisseur_rapproche::facture_payee(&pool, &seeded, f, "FF-P", jour(), dec!(100)).await;
    let doc = document(&pool, c, "doc-a").await;

    // Document inconnu.
    assert_eq!(
        rattacher(&pool, c, 999_999, paid).await.unwrap_err(),
        AttachRefusal::DocumentNotFound
    );
    // Document d'une autre société (IDOR, côté document).
    let autre = autre_societe(&pool).await;
    let doc_autre = document(&pool, autre, "doc-autre").await;
    assert_eq!(
        rattacher(&pool, c, doc_autre.id, paid).await.unwrap_err(),
        AttachRefusal::DocumentNotFound
    );
    // Facture inconnue ; facture d'une autre société (IDOR, côté facture : le
    // document de l'autre société, la facture de la nôtre).
    assert_eq!(
        rattacher(&pool, c, doc.id, 999_999).await.unwrap_err(),
        AttachRefusal::InvoiceNotFound
    );
    assert_eq!(
        rattacher(&pool, autre, doc_autre.id, paid)
            .await
            .unwrap_err(),
        AttachRefusal::InvoiceNotFound,
        "M-c4 : une facture d'une autre société ne se rattache pas"
    );
    // Document retiré de la liste.
    let retire = document(&pool, c, "doc-retire").await;
    let mut tx = pool.begin().await.unwrap();
    repo::mark_discarded(&mut tx, c, retire.id).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        rattacher(&pool, c, retire.id, paid).await.unwrap_err(),
        AttachRefusal::DocumentNotPending {
            current_status: "discarded".into()
        },
        "M-c3 : un document retiré ne se rattache pas"
    );
    assert_eq!(
        statut_et_lien(&pool, retire.id).await,
        ("discarded".into(), None)
    );

    // Succès sur une facture payée.
    let avant = etat_facture(&pool, paid).await;
    let ecritures = nombre_d_ecritures(&pool, c).await;
    let lu = rattacher(&pool, c, doc.id, paid).await.expect("rattaché");
    assert_eq!(lu.original_filename, "doc-a.png");
    assert_eq!(
        statut_et_lien(&pool, doc.id).await,
        ("completed".into(), Some(paid))
    );
    assert_eq!(
        etat_facture(&pool, paid).await,
        avant,
        "la facture ne change pas"
    );
    assert_eq!(
        nombre_d_ecritures(&pool, c).await,
        ecritures,
        "aucune écriture"
    );

    // Un second document vers la même facture : refusé par l'unicité.
    let doc2 = document(&pool, c, "doc-b").await;
    assert_eq!(
        rattacher(&pool, c, doc2.id, paid).await.unwrap_err(),
        AttachRefusal::AlreadyAttached
    );
    assert_eq!(
        statut_et_lien(&pool, doc2.id).await,
        ("to_complete".into(), None)
    );
    // Le document déjà traité ne se rattache pas une seconde fois.
    let (open, _) = facture_ouverte(&pool, &seeded, f, "FF-O", dec!(50)).await;
    assert_eq!(
        rattacher(&pool, c, doc.id, open).await.unwrap_err(),
        AttachRefusal::DocumentNotPending {
            current_status: "completed".into()
        }
    );
}

/// AC19, **sous la bascule écrite** (C-30-2-46) — test déterministe, sous
/// verrou tenu : une transaction A rattache D1 à I1 sans valider ; B rattache D2
/// à I2 (I2 = I1 + 1, aucune entrée de l'index entre elles ni au-delà).
///
/// ⚠️ Relevé au développement sur le code sans sentinelle : B **attendait** dans
/// son `UPDATE` (contrôle de doublon d'InnoDB sur l'intervalle jusqu'au
/// supremum), contrairement au banc de la fiche. D'où la bascule : B attend
/// désormais sur la **sentinelle** de la société, avant toute insertion dans
/// l'index, et aboutit après le `COMMIT` de A — sans interblocage ni refus.
///
/// Tue M-c13 (sentinelle retirée : B attend dans l'`UPDATE`, jamais sur
/// `companies`) et M-c11 (lecture verrouillante du lien réintroduite entre la
/// facture et la sentinelle : les deux verrous d'intervalle, compatibles entre
/// eux, bloquent chacun l'insertion de l'autre — interblocage).
#[sqlx::test(migrations = "./test-schema")]
async fn two_attachments_to_two_invoices_are_serialized_by_company(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (i1, _) = facture_ouverte(&pool, &seeded, f, "FF-1", dec!(10)).await;
    let (i2, _) = facture_ouverte(&pool, &seeded, f, "FF-2", dec!(20)).await;
    assert_eq!(i2, i1 + 1, "montage : deux factures consécutives");
    let d1 = document(&pool, c, "d1").await;
    let d2 = document(&pool, c, "d2").await;

    let mut a = pool.begin().await.unwrap();
    repo::attach_in_tx(&mut a, c, d1.id, i1)
        .await
        .unwrap()
        .expect("A rattache D1 à I1");

    let pool_b = pool.clone();
    let b = tokio::spawn(async move { rattacher(&pool_b, c, d2.id, i2).await });
    let vu =
        attendre_une_requete_en_cours(&pool, &["FROM companies WHERE id", "FOR UPDATE"], || {
            b.is_finished()
        })
        .await;
    assert!(
        vu,
        "B devait attendre la sentinelle de la société, tenue par A"
    );
    assert!(!b.is_finished());
    a.commit().await.unwrap();
    let b = tokio::time::timeout(BORNE, b)
        .await
        .expect("B aboutit après le COMMIT de A")
        .expect("aucune panique : aucune erreur de base, donc aucun interblocage");
    assert!(b.is_ok(), "B aboutit en succès : {b:?}");
    assert_eq!(
        statut_et_lien(&pool, d1.id).await,
        ("completed".into(), Some(i1))
    );
    assert_eq!(
        statut_et_lien(&pool, d2.id).await,
        ("completed".into(), Some(i2))
    );
}

/// AC19 — deux rattachements vers la **même** facture se sérialisent sur le
/// verrou de la facture : un succès, un refus `AlreadyAttached` (sous M-c2,
/// unicité retirée, deux succès). Tue M-c2.
#[sqlx::test(migrations = "./test-schema")]
async fn two_attachments_to_the_same_invoice_one_wins(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (inv, _) = facture_ouverte(&pool, &seeded, f, "FF-M", dec!(10)).await;
    let d1 = document(&pool, c, "m1").await;
    let d2 = document(&pool, c, "m2").await;

    let mut a = pool.begin().await.unwrap();
    repo::attach_in_tx(&mut a, c, d1.id, inv)
        .await
        .unwrap()
        .expect("A rattache D1");
    let pool_b = pool.clone();
    let b = tokio::spawn(async move { rattacher(&pool_b, c, d2.id, inv).await });
    // B attend le verrou de la facture (étape 2), tenu par A.
    let vu = attendre_une_requete_en_cours(
        &pool,
        &["FROM supplier_invoices WHERE id", "FOR UPDATE"],
        || b.is_finished(),
    )
    .await;
    assert!(vu, "B devait attendre la facture tenue par A");
    a.commit().await.unwrap();
    let b = tokio::time::timeout(BORNE, b)
        .await
        .expect("B aboutit")
        .unwrap();
    assert_eq!(b.unwrap_err(), AttachRefusal::AlreadyAttached);
    assert_eq!(
        statut_et_lien(&pool, d1.id).await,
        ("completed".into(), Some(inv))
    );
    assert_eq!(
        statut_et_lien(&pool, d2.id).await,
        ("to_complete".into(), None)
    );
}

/// La complétion, réduite à ce qu'elle verrouille et écrit : le document sous
/// verrou, son statut, une facture neuve, le lien.
async fn completer_dans(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    seeded: &SeededCompany,
    f: fournisseur_rapproche::Fournisseur,
    doc: i64,
) -> Result<i64, String> {
    let row = repo::find_by_id_scoped_for_update(tx, seeded.company_id, doc)
        .await
        .unwrap()
        .expect("document");
    if row.status != "to_complete" {
        return Err(row.status);
    }
    let created = supplier_invoices::create_in_tx(
        tx,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id: f.supplier_id,
            supplier_invoice_number: Some(format!("FF-C{doc}")),
            invoice_date: jour(),
            due_date: None,
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Complétion".into(),
                quantity: dec!(1),
                unit_price: dec!(100),
                vat_rate: dec!(0),
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    repo::mark_completed(tx, seeded.company_id, doc, created.invoice.id)
        .await
        .unwrap();
    Ok(created.invoice.id)
}

/// AC19 — rattachement contre complétion du **même** document : un seul gagne,
/// dans les deux ordres (ils se sérialisent sur le verrou du document).
#[sqlx::test(migrations = "./test-schema")]
async fn attachment_against_completion_of_the_same_document_one_wins(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (inv, _) = facture_ouverte(&pool, &seeded, f, "FF-R", dec!(10)).await;

    // Ordre 1 : le rattachement tient le document ; la complétion attend, puis refuse.
    let d = document(&pool, c, "r1").await;
    let mut a = pool.begin().await.unwrap();
    repo::attach_in_tx(&mut a, c, d.id, inv)
        .await
        .unwrap()
        .expect("rattaché");
    let (pool_b, seeded_b) = (pool.clone(), seeded.clone());
    let b = tokio::spawn(async move {
        let mut tx = pool_b.begin().await.unwrap();
        let r = completer_dans(&mut tx, &seeded_b, f, d.id).await;
        tx.rollback().await.unwrap();
        r
    });
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["FROM imported_supplier_invoices WHERE id", "FOR UPDATE"],
            || b.is_finished()
        )
        .await,
        "la complétion devait attendre le document"
    );
    a.commit().await.unwrap();
    let b = tokio::time::timeout(BORNE, b).await.unwrap().unwrap();
    assert_eq!(b.unwrap_err(), "completed", "la complétion perd");

    // Ordre 2 : la complétion tient le document ; le rattachement attend, puis refuse.
    let (inv2, _) = facture_ouverte(&pool, &seeded, f, "FF-R2", dec!(10)).await;
    let d2 = document(&pool, c, "r2").await;
    let mut a = pool.begin().await.unwrap();
    completer_dans(&mut a, &seeded, f, d2.id)
        .await
        .expect("complété");
    let pool_b = pool.clone();
    let b = tokio::spawn(async move { rattacher(&pool_b, c, d2.id, inv2).await });
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["FROM imported_supplier_invoices WHERE id", "FOR UPDATE"],
            || b.is_finished()
        )
        .await,
        "le rattachement devait attendre le document"
    );
    a.commit().await.unwrap();
    let b = tokio::time::timeout(BORNE, b).await.unwrap().unwrap();
    assert_eq!(
        b.unwrap_err(),
        AttachRefusal::DocumentNotPending {
            current_status: "completed".into()
        },
        "le rattachement perd"
    );
}

// ---------------------------------------------------------------------------
// AC24 — détacher
// ---------------------------------------------------------------------------

/// AC24 — détacher d'une facture ouverte puis d'une facture payée : le document
/// revient `to_complete` sans lien, la facture et le grand livre ne bougent pas ;
/// sans document, ou d'une autre société : `None`. Puis joindre **un autre**
/// document à la même facture réussit — tue M-c9 (lien laissé : l'unicité le
/// tiendrait encore).
#[sqlx::test(migrations = "./test-schema")]
async fn detach_restores_the_document_and_frees_the_invoice(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (open, _) = facture_ouverte(&pool, &seeded, f, "FF-D", dec!(10)).await;
    let (paid, _, _) =
        fournisseur_rapproche::facture_payee(&pool, &seeded, f, "FF-DP", jour(), dec!(20)).await;

    // Sans document : rien à détacher.
    assert!(detacher(&pool, c, open).await.is_none());
    assert!(detacher(&pool, c, 999_999).await.is_none());

    for (inv, hash) in [(open, "x-open"), (paid, "x-paid")] {
        let doc = document(&pool, c, hash).await;
        rattacher(&pool, c, doc.id, inv).await.expect("rattaché");
        // D'une autre société : rien.
        let autre = autre_societe(&pool).await;
        assert!(detacher(&pool, autre, inv).await.is_none(), "IDOR");

        let avant = etat_facture(&pool, inv).await;
        let ecritures = nombre_d_ecritures(&pool, c).await;
        let lu = detacher(&pool, c, inv).await.expect("détaché");
        assert_eq!(lu.id, doc.id);
        assert_eq!(
            statut_et_lien(&pool, doc.id).await,
            ("to_complete".into(), None)
        );
        assert_eq!(
            etat_facture(&pool, inv).await,
            avant,
            "la facture ne change pas"
        );
        assert_eq!(
            nombre_d_ecritures(&pool, c).await,
            ecritures,
            "aucune écriture"
        );
        // Revenu dans la liste « à traiter », données QR intactes.
        let liste = repo::list_by_status(&pool, c, "to_complete").await.unwrap();
        let revenu = liste
            .iter()
            .find(|r| r.id == doc.id)
            .expect("revenu dans la liste");
        assert_eq!(
            revenu.creditor_iban.as_deref(),
            Some("CH9300762011623852957")
        );
        // La lecture de la facture ne le trouve plus.
        assert!(
            repo::find_by_supplier_invoice_id_scoped(&pool, c, inv)
                .await
                .unwrap()
                .is_none()
        );

        // M-c9 : un autre document se joint à la même facture.
        let autre_doc = document(&pool, c, &format!("{hash}-2")).await;
        rattacher(&pool, c, autre_doc.id, inv)
            .await
            .expect("après détachement, un autre document se joint");
    }
}

/// AC24 (M-c10) — le détachement ne demande **pas** la facture : une autre
/// connexion la tient `FOR UPDATE`, le détachement aboutit sous 5 s.
#[sqlx::test(migrations = "./test-schema")]
async fn detach_does_not_wait_for_the_invoice(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (inv, _) = facture_ouverte(&pool, &seeded, f, "FF-T", dec!(10)).await;
    let doc = document(&pool, c, "t1").await;
    rattacher(&pool, c, doc.id, inv).await.expect("rattaché");

    let mut tiers = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM supplier_invoices WHERE id = ? FOR UPDATE")
        .bind(inv)
        .fetch_one(&mut *tiers)
        .await
        .unwrap();
    let r = tokio::time::timeout(BORNE, detacher(&pool, c, inv)).await;
    assert!(
        matches!(r, Ok(Some(_))),
        "le détachement doit aboutir pendant qu'un tiers tient la facture : {r:?}"
    );
    tiers.rollback().await.unwrap();
}

/// AC24 — un rattachement vers la facture X pendant un détachement **non
/// validé** de X : son `UPDATE` attend l'entrée d'unicité que tient le
/// détachement, puis, celui-ci validé, aboutit — en succès, sans interblocage
/// ni refus.
#[sqlx::test(migrations = "./test-schema")]
async fn attach_waits_for_an_uncommitted_detach_then_succeeds(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (x, _) = facture_ouverte(&pool, &seeded, f, "FF-X", dec!(10)).await;
    let old = document(&pool, c, "old").await;
    let new = document(&pool, c, "new").await;
    rattacher(&pool, c, old.id, x).await.expect("rattaché");

    let mut d = pool.begin().await.unwrap();
    repo::detach_in_tx(&mut d, c, x)
        .await
        .unwrap()
        .expect("D détache S_old de X, sans valider");

    let pool_t = pool.clone();
    let tache = tokio::spawn(async move { rattacher(&pool_t, c, new.id, x).await });
    let vu = attendre_une_requete_en_cours(
        &pool,
        &["UPDATE imported_supplier_invoices", "supplier_invoice_id"],
        || tache.is_finished(),
    )
    .await;
    assert!(vu, "l'UPDATE du rattachement devait être vu en cours");
    assert!(
        !tache.is_finished(),
        "le rattachement attend le verrou de D"
    );
    assert!(
        !sonde_verrou_nowait(
            &pool,
            "SELECT id FROM imported_supplier_invoices WHERE id = ? FOR UPDATE NOWAIT",
            old.id
        )
        .await,
        "la ligne de S_old est tenue par D"
    );
    d.commit().await.unwrap();
    let r = tokio::time::timeout(BORNE, tache)
        .await
        .expect("aboutit")
        .unwrap();
    assert!(r.is_ok(), "le rattachement aboutit en succès : {r:?}");
    assert_eq!(
        statut_et_lien(&pool, new.id).await,
        ("completed".into(), Some(x))
    );
    assert_eq!(
        statut_et_lien(&pool, old.id).await,
        ("to_complete".into(), None)
    );
}

// ---------------------------------------------------------------------------
// AC18, AC21 — lectures
// ---------------------------------------------------------------------------

/// AC18 — la lecture partagée ne retrouve qu'un document `completed` : un lien
/// resté sur un document d'un autre statut ne compte pas.
#[sqlx::test(migrations = "./test-schema")]
async fn shared_lookup_only_sees_completed_documents(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (inv, _) = facture_ouverte(&pool, &seeded, f, "FF-L", dec!(10)).await;
    let doc = document(&pool, c, "l1").await;
    rattacher(&pool, c, doc.id, inv).await.expect("rattaché");
    assert_eq!(
        repo::find_by_supplier_invoice_id_scoped(&pool, c, inv)
            .await
            .unwrap()
            .map(|r| r.id),
        Some(doc.id)
    );
    // Lien laissé sur un document repassé « à traiter » (état qu'aucun chemin
    // n'écrit plus) : la lecture ne le voit pas.
    sqlx::query("UPDATE imported_supplier_invoices SET status = 'to_complete' WHERE id = ?")
        .bind(doc.id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        repo::find_by_supplier_invoice_id_scoped(&pool, c, inv)
            .await
            .unwrap()
            .is_none()
    );
}

/// AC21 — le document d'une écriture d'**achat** et de **règlement** (M-c5) ;
/// rien pour une écriture d'une facture sans document, ni d'une autre société,
/// ni après détachement.
#[sqlx::test(migrations = "./test-schema")]
async fn entry_lookup_covers_purchase_and_settlement(pool: MySqlPool) {
    let (seeded, f) = decor(&pool).await;
    let c = seeded.company_id;
    let (inv, achat, reglement) =
        fournisseur_rapproche::facture_payee(&pool, &seeded, f, "FF-E", jour(), dec!(30)).await;
    let (_sans, achat_sans) = facture_ouverte(&pool, &seeded, f, "FF-S", dec!(5)).await;
    let doc = document(&pool, c, "e1").await;

    assert!(
        repo::find_source_document_of_entry(&pool, c, achat)
            .await
            .unwrap()
            .is_none(),
        "pas encore de document"
    );
    rattacher(&pool, c, doc.id, inv).await.expect("rattaché");
    let attendu = repo::EntrySourceDocument {
        supplier_invoice_id: inv,
        supplier_invoice_number: Some("FF-E".into()),
        original_filename: "e1.png".into(),
    };
    for entry in [achat, reglement] {
        assert_eq!(
            repo::find_source_document_of_entry(&pool, c, entry)
                .await
                .unwrap(),
            Some(attendu.clone()),
            "écriture {entry} (achat {achat}, règlement {reglement})"
        );
    }
    assert!(
        repo::find_source_document_of_entry(&pool, c, achat_sans)
            .await
            .unwrap()
            .is_none()
    );
    let autre = autre_societe(&pool).await;
    assert!(
        repo::find_source_document_of_entry(&pool, autre, achat)
            .await
            .unwrap()
            .is_none(),
        "IDOR"
    );
    detacher(&pool, c, inv).await.expect("détaché");
    assert!(
        repo::find_source_document_of_entry(&pool, c, reglement)
            .await
            .unwrap()
            .is_none()
    );
}
