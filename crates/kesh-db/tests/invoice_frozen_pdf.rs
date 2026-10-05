//! Story 25-6-b (#387) — les gardes de la pose du PDF figé, côté base.
//!
//! Le service de `kesh-api` rend le PDF puis le pose par
//! `invoices::freeze_pdf`, un `UPDATE` conditionnel. Ses trois gardes —
//! `pdf_storage_path IS NULL`, `status = 'validated'`, `version` — ne se
//! prouvent pas par une course (`tokio::join!` passerait aussi sans elles) :
//! chaque test ici **intercale** l'événement concurrent de façon déterministe,
//! puis pose. ⚠️ Chaque test nomme la mutation qu'il tue.

use chrono::NaiveDate;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::contacts;
use kesh_db::repositories::invoices::{self, FrozenPdf};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

struct Seeded {
    company_id: i64,
    user_id: i64,
    invoice_id: i64,
}

async fn seed_validated(pool: &MySqlPool) -> Seeded {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let contact = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Marktgasse".into()),
            address_building: Some("28".into()),
            address_postal_code: Some("9400".into()),
            address_city: Some("Rorschach".into()),
            address_country: Some("CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Salutation::Neutre,
        },
    )
    .await
    .expect("contact");
    let (invoice, _) = invoices::create(
        pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id: contact.id,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: None,
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Conseil".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(8.10),
            }],
            project_id: None,
        },
    )
    .await
    .expect("facture");
    invoices::validate_invoice(pool, seeded.company_id, invoice.id, seeded.admin_user_id)
        .await
        .expect("validation");
    Seeded {
        company_id: seeded.company_id,
        user_id: seeded.admin_user_id,
        invoice_id: invoice.id,
    }
}

fn frozen(tag: char) -> FrozenPdf {
    let sha256: String = std::iter::repeat_n(tag, 64).collect();
    FrozenPdf {
        storage_path: format!("{sha256}.pdf"),
        sha256,
        language: "FR".into(),
    }
}

async fn version_of(pool: &MySqlPool, s: &Seeded) -> i32 {
    invoices::find_by_id(pool, s.company_id, s.invoice_id)
        .await
        .unwrap()
        .unwrap()
        .version
}

async fn audit_count(pool: &MySqlPool, s: &Seeded, action: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'invoice' AND entity_id = ? AND action = ?",
    )
    .bind(s.invoice_id)
    .bind(action)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "./test-schema")]
async fn la_pose_fige_sans_toucher_la_version_et_trace_l_acteur(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v = version_of(&pool, &s).await;

    let applied = invoices::freeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        v,
        &frozen('a'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    assert!(applied);

    let inv = invoices::find_by_id(&pool, s.company_id, s.invoice_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inv.pdf_sha256.as_deref(), Some("a".repeat(64).as_str()));
    assert_eq!(inv.pdf_language.as_deref(), Some("FR"));
    assert!(inv.pdf_frozen_at.is_some());
    // B2 : le gel ne touche pas `version` (mutation : `version = version + 1`).
    assert_eq!(inv.version, v);

    let (user_id, details): (i64, Vec<u8>) = sqlx::query_as(
        "SELECT user_id, details_json FROM audit_log \
         WHERE entity_type = 'invoice' AND entity_id = ? AND action = 'invoice.pdf_frozen'",
    )
    .bind(s.invoice_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(user_id, s.user_id);
    let details: serde_json::Value = serde_json::from_slice(&details).unwrap();
    assert_eq!(details["pdfSha256"], "a".repeat(64));
    assert_eq!(details["pdfLanguage"], "FR");
}

/// Mutation tuée : garde `pdf_storage_path IS NULL` retirée — la seconde pose
/// remplacerait le document déjà émis.
#[sqlx::test(migrations = "./test-schema")]
async fn une_facture_deja_figee_ne_se_refige_pas(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v = version_of(&pool, &s).await;
    assert!(
        invoices::freeze_pdf(
            &pool,
            s.company_id,
            s.invoice_id,
            v,
            &frozen('a'),
            s.user_id,
            None
        )
        .await
        .unwrap()
    );

    let second = invoices::freeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        v,
        &frozen('b'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    assert!(!second, "la seconde pose doit être refusée");
    let inv = invoices::find_by_id(&pool, s.company_id, s.invoice_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inv.pdf_sha256.as_deref(), Some("a".repeat(64).as_str()));
    // Le refus n'écrit rien au journal.
    assert_eq!(audit_count(&pool, &s, "invoice.pdf_frozen").await, 1);
}

/// Mutation tuée : garde `status = 'validated'` retirée. La facture repasse en
/// brouillon **à version égale** — un `UPDATE` brut, pour isoler cette garde de
/// celle de version (une vraie dévalidation incrémente aussi `version`).
#[sqlx::test(migrations = "./test-schema")]
async fn rien_ne_se_fige_sur_un_brouillon(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v = version_of(&pool, &s).await;
    let je: Option<i64> = sqlx::query_scalar("SELECT journal_entry_id FROM invoices WHERE id = ?")
        .bind(s.invoice_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE invoices SET status = 'draft', journal_entry_id = NULL WHERE id = ?")
        .bind(s.invoice_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(je.is_some());
    assert_eq!(
        version_of(&pool, &s).await,
        v,
        "montage : version inchangée"
    );

    let applied = invoices::freeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        v,
        &frozen('a'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    assert!(!applied);
    let inv = invoices::find_by_id(&pool, s.company_id, s.invoice_id)
        .await
        .unwrap()
        .unwrap();
    assert!(inv.pdf_storage_path.is_none());
}

/// Mutation tuée : garde `version` retirée. Dévalidation → modification →
/// revalidation dans la fenêtre du rendu : le statut est de nouveau
/// `validated`, seule la version dit que les lignes ont changé.
#[sqlx::test(migrations = "./test-schema")]
async fn un_rendu_perime_ne_se_fige_pas(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v_rendu = version_of(&pool, &s).await;

    let (draft, _) = invoices::unvalidate(&pool, s.company_id, s.invoice_id, s.user_id, v_rendu)
        .await
        .unwrap();
    sqlx::query("UPDATE invoice_lines SET description = 'Conseil (modifié)' WHERE invoice_id = ?")
        .bind(s.invoice_id)
        .execute(&pool)
        .await
        .unwrap();
    invoices::validate_invoice(&pool, s.company_id, s.invoice_id, s.user_id)
        .await
        .unwrap();
    assert_ne!(version_of(&pool, &s).await, v_rendu);
    assert_eq!(draft.status, "draft");

    let applied = invoices::freeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        v_rendu,
        &frozen('a'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    assert!(
        !applied,
        "un rendu de l'ancienne version ne doit pas se figer"
    );
    let inv = invoices::find_by_id(&pool, s.company_id, s.invoice_id)
        .await
        .unwrap()
        .unwrap();
    assert!(inv.pdf_storage_path.is_none());
}

/// Isolation : une pose sous une autre société ne touche rien.
#[sqlx::test(migrations = "./test-schema")]
async fn la_pose_est_scopee_par_societe(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v = version_of(&pool, &s).await;
    let applied = invoices::freeze_pdf(
        &pool,
        s.company_id + 999,
        s.invoice_id,
        v,
        &frozen('a'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    assert!(!applied);
}

/// AC 5 : la dévalidation détache, et l'audit porte l'empreinte détachée.
#[sqlx::test(migrations = "./test-schema")]
async fn la_devalidation_detache_le_pdf(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v = version_of(&pool, &s).await;
    invoices::freeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        v,
        &frozen('a'),
        s.user_id,
        None,
    )
    .await
    .unwrap();

    let (after, _) = invoices::unvalidate(&pool, s.company_id, s.invoice_id, s.user_id, v)
        .await
        .unwrap();
    assert!(after.pdf_storage_path.is_none());
    assert!(after.pdf_sha256.is_none());
    assert!(after.pdf_frozen_at.is_none());
    assert!(after.pdf_language.is_none());

    let details: Vec<u8> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE entity_type = 'invoice' AND entity_id = ? AND action = 'invoice.unvalidated'",
    )
    .bind(s.invoice_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let details: serde_json::Value = serde_json::from_slice(&details).unwrap();
    assert_eq!(details["pdfSha256"], "a".repeat(64));
}

/// AC 1 : la contrainte tout-ou-rien rejette un état partiel, et une langue
/// hors des quatre.
#[sqlx::test(migrations = "./test-schema")]
async fn la_contrainte_rejette_un_etat_partiel(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let partiel = sqlx::query("UPDATE invoices SET pdf_sha256 = ? WHERE id = ?")
        .bind("a".repeat(64))
        .bind(s.invoice_id)
        .execute(&pool)
        .await;
    assert!(partiel.is_err(), "empreinte sans chemin : refusé");

    let courte = sqlx::query(
        "UPDATE invoices SET pdf_storage_path = 'x.pdf', pdf_sha256 = 'abc', \
         pdf_frozen_at = NOW(3), pdf_language = 'FR' WHERE id = ?",
    )
    .bind(s.invoice_id)
    .execute(&pool)
    .await;
    assert!(courte.is_err(), "empreinte de 3 caractères : refusé");

    let langue = sqlx::query(
        "UPDATE invoices SET pdf_storage_path = 'x.pdf', pdf_sha256 = ?, \
         pdf_frozen_at = NOW(3), pdf_language = 'fr' WHERE id = ?",
    )
    .bind("a".repeat(64))
    .bind(s.invoice_id)
    .execute(&pool)
    .await;
    assert!(langue.is_err(), "langue en minuscules : refusé");
}

/// AC 3-bis : deux refigeages concurrents, un seul réussit ; l'audit porte
/// l'ancienne et la nouvelle empreinte.
#[sqlx::test(migrations = "./test-schema")]
async fn deux_refigeages_concurrents_un_seul_reussit(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v = version_of(&pool, &s).await;
    invoices::freeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        v,
        &frozen('a'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    let old = "a".repeat(64);

    let first = invoices::refreeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        &old,
        &frozen('b'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    let second = invoices::refreeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        &old,
        &frozen('c'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    assert!(first);
    assert!(!second, "mutation tuée : garde `pdf_sha256 = old` retirée");

    let inv = invoices::find_by_id(&pool, s.company_id, s.invoice_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(inv.pdf_sha256.as_deref(), Some("b".repeat(64).as_str()));
    assert_eq!(audit_count(&pool, &s, "invoice.pdf_refrozen").await, 1);
    let details: Vec<u8> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE entity_type = 'invoice' AND entity_id = ? AND action = 'invoice.pdf_refrozen'",
    )
    .bind(s.invoice_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let details: serde_json::Value = serde_json::from_slice(&details).unwrap();
    assert_eq!(details["oldPdfSha256"], old);
    assert_eq!(details["pdfSha256"], "b".repeat(64));
}

/// AC 3-bis : un refigeage n'aboutit pas sur une facture qui n'est plus
/// validée (mutation tuée : garde `status` retirée du refigeage).
#[sqlx::test(migrations = "./test-schema")]
async fn le_refigeage_refuse_une_facture_qui_n_est_plus_validee(pool: MySqlPool) {
    let s = seed_validated(&pool).await;
    let v = version_of(&pool, &s).await;
    invoices::freeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        v,
        &frozen('a'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
        .bind(s.invoice_id)
        .execute(&pool)
        .await
        .unwrap();

    let applied = invoices::refreeze_pdf(
        &pool,
        s.company_id,
        s.invoice_id,
        &"a".repeat(64),
        &frozen('b'),
        s.user_id,
        None,
    )
    .await
    .unwrap();
    assert!(!applied);
}
