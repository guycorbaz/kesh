//! Story v017-4a (#849, AC-1) — les contraintes que pose la migration
//! `20261031000001_supplier_invoices_draft`, prouvées par la base elle-même :
//! chaque `INSERT` direct d'un état incohérent est refusé, et le refus **nomme**
//! la contrainte.
//!
//! - un brouillon **avec** écriture d'achat → `chk_supplier_invoices_entry_iff_not_draft` ;
//! - une facture validée (`open`) **sans** écriture → la même contrainte ;
//! - un brouillon qui porte `paid_at` → `chk_supplier_invoices_draft_without_settlement` ;
//! - un statut inconnu → `chk_supplier_invoices_status_v2` ;
//! - `link_origin` hors de son ensemble → `chk_imported_si_link_origin`.
//!
//! Et l'état légitime passe : un brouillon sans écriture, un document lié par
//! complétion. Montés sur le squash (`test-schema`), régénéré depuis les
//! migrations ; `test_schema_guard` en garde l'égalité avec le vrai schéma.
//!
//! Pré-requis : MariaDB démarré.

use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use sqlx::MySqlPool;

/// Un fournisseur et une écriture quelconque de la société : de quoi poser une
/// ligne de `supplier_invoices` à la main.
async fn decor(pool: &MySqlPool) -> (SeededCompany, i64, i64) {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let contact: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
         VALUES (?, 'Entreprise', 'Fournisseur schéma SA', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .expect("fournisseur");
    let entry: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, 1, '2026-06-15', 'Achats', 'Achat') RETURNING id",
    )
    .bind(seeded.company_id)
    .bind(seeded.fiscal_year_id)
    .fetch_one(pool)
    .await
    .expect("écriture");
    (seeded, contact, entry)
}

/// Tente l'insertion d'une facture fournisseur ; rend l'erreur brute de la base.
async fn inserer(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact: i64,
    status: &str,
    entry: Option<i64>,
    paid_at: Option<&str>,
) -> Result<i64, String> {
    sqlx::query_scalar(
        "INSERT INTO supplier_invoices (company_id, contact_id, status, invoice_date, \
         total_amount, purchase_journal_entry_id, paid_at) \
         VALUES (?, ?, ?, '2026-06-15', 100, ?, ?) RETURNING id",
    )
    .bind(seeded.company_id)
    .bind(contact)
    .bind(status)
    .bind(entry)
    .bind(paid_at)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())
}

fn refus_nomme(r: Result<i64, String>, contrainte: &str) {
    let erreur = r.expect_err("la base devait refuser la ligne");
    assert!(
        erreur.contains(contrainte),
        "le refus devait nommer `{contrainte}` : {erreur}"
    );
}

/// AC-1 — un brouillon ne porte jamais d'écriture d'achat.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_with_a_purchase_entry_is_refused(pool: MySqlPool) {
    let (seeded, contact, entry) = decor(&pool).await;
    refus_nomme(
        inserer(&pool, &seeded, contact, "draft", Some(entry), None).await,
        "chk_supplier_invoices_entry_iff_not_draft",
    );
}

/// AC-1 — une facture validée porte toujours son écriture d'achat. Tue M15.
#[sqlx::test(migrations = "./test-schema")]
async fn a_validated_invoice_without_a_purchase_entry_is_refused(pool: MySqlPool) {
    let (seeded, contact, _) = decor(&pool).await;
    refus_nomme(
        inserer(&pool, &seeded, contact, "open", None, None).await,
        "chk_supplier_invoices_entry_iff_not_draft",
    );
}

/// AC-1 — un brouillon n'a aucun règlement.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_bearing_paid_at_is_refused(pool: MySqlPool) {
    let (seeded, contact, _) = decor(&pool).await;
    refus_nomme(
        inserer(
            &pool,
            &seeded,
            contact,
            "draft",
            None,
            Some("2026-06-20 00:00:00"),
        )
        .await,
        "chk_supplier_invoices_draft_without_settlement",
    );
}

/// AC-1 — l'ensemble des statuts est fermé, `draft` compris.
#[sqlx::test(migrations = "./test-schema")]
async fn an_unknown_status_is_refused_by_the_new_constraint(pool: MySqlPool) {
    let (seeded, contact, entry) = decor(&pool).await;
    refus_nomme(
        inserer(&pool, &seeded, contact, "pending", Some(entry), None).await,
        "chk_supplier_invoices_status_v2",
    );
}

/// AC-1 — l'état légitime passe : un brouillon sans écriture ni règlement.
#[sqlx::test(migrations = "./test-schema")]
async fn a_draft_without_entry_is_accepted(pool: MySqlPool) {
    let (seeded, contact, _) = decor(&pool).await;
    let id = inserer(&pool, &seeded, contact, "draft", None, None)
        .await
        .expect("un brouillon sans écriture est légitime");
    let entry: Option<i64> =
        sqlx::query_scalar("SELECT purchase_journal_entry_id FROM supplier_invoices WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(entry, None);
}

/// AC-1, point 4 bis — `link_origin` : nul, `completion` ou `attachment`.
#[sqlx::test(migrations = "./test-schema")]
async fn link_origin_is_a_closed_set(pool: MySqlPool) {
    let (seeded, _, _) = decor(&pool).await;
    let inserer_doc = |hash: &'static str, origin: Option<&'static str>| {
        let pool = pool.clone();
        let company = seeded.company_id;
        async move {
            sqlx::query(
                "INSERT INTO imported_supplier_invoices (company_id, file_hash, storage_path, \
                 original_filename, mime_type, byte_size, link_origin) \
                 VALUES (?, ?, 'x.pdf', 'x.pdf', 'application/pdf', 1, ?)",
            )
            .bind(company)
            .bind(hash)
            .bind(origin)
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
        }
    };
    inserer_doc("a", None).await.expect("nul admis");
    inserer_doc("b", Some("completion"))
        .await
        .expect("completion admis");
    inserer_doc("c", Some("attachment"))
        .await
        .expect("attachment admis");
    let erreur = inserer_doc("d", Some("scan"))
        .await
        .expect_err("hors ensemble");
    assert!(
        erreur.contains("chk_imported_si_link_origin"),
        "le refus devait nommer la contrainte : {erreur}"
    );
}
