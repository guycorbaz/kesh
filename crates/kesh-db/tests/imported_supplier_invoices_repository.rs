//! Tests d'intégration pour `repositories::imported_supplier_invoices`
//! (Story 12-5b T3.5 — #194).
//!
//! Couvre : round-trip `create` + `find_by_id_scoped`, isolement multi-tenant
//! (anti-IDOR), contrainte `UNIQUE (company_id, file_hash)` (doublon rejeté même
//! company, même hash accepté sur 2 companies), `find_by_company_hash` scopé, et
//! `list_by_status` filtré + scopé.
//!
//! Pattern `#[sqlx::test(migrations = "./test-schema")]` — DB éphémère avec
//! migrations auto-appliquées.

use kesh_db::entities::imported_supplier_invoice::NewImportedSupplierInvoice;
use kesh_db::errors::DbError;
use kesh_db::repositories::imported_supplier_invoices as repo;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool, name: &str) -> i64 {
    let result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind("Rue Test 1")
    .bind("Independant")
    .bind("FR")
    .bind("FR")
    .execute(pool)
    .await
    .expect("company insert");
    result.last_insert_id() as i64
}

/// Construit une facture importée d'insertion avec un `file_hash` paramétrable
/// (pour exercer la contrainte d'unicité).
fn new_invoice(company_id: i64, file_hash: &str) -> NewImportedSupplierInvoice {
    NewImportedSupplierInvoice {
        company_id,
        file_hash: file_hash.into(),
        storage_path: format!("{file_hash}.pdf"),
        original_filename: "facture.pdf".into(),
        mime_type: "application/pdf".into(),
        byte_size: 4096,
        creditor_iban: Some("CH4431999123000889012".into()),
        is_qr_iban: Some(true),
        creditor_address_type: Some("K".into()),
        creditor_name: Some("Robert Schneider SA".into()),
        creditor_line1: Some("Rue du Lac 1268".into()),
        creditor_line2: Some("2501 Biel".into()),
        creditor_postal_code: None,
        creditor_town: None,
        creditor_country: Some("CH".into()),
        reference_type: Some("QRR".into()),
        reference_value: Some("210000000003139471430009017".into()),
        amount: Some(dec!(199.95)),
        currency: Some("CHF".into()),
        unstructured_message: Some("Facture test".into()),
        billing_information: None,
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn create_then_find_by_id_scoped_roundtrip(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme SA").await;

    let created = repo::create(&pool, &new_invoice(company_id, "hash-roundtrip"))
        .await
        .expect("create staging");

    // Défauts staging.
    assert_eq!(created.status, "to_complete");
    assert_eq!(created.supplier_invoice_id, None);
    assert_eq!(created.version, 1);
    // Champs persistés.
    assert_eq!(created.company_id, company_id);
    assert_eq!(created.file_hash, "hash-roundtrip");
    assert_eq!(
        created.creditor_iban.as_deref(),
        Some("CH4431999123000889012")
    );
    assert_eq!(created.is_qr_iban, Some(true));
    assert_eq!(created.amount, Some(dec!(199.95)));
    assert_eq!(created.reference_type.as_deref(), Some("QRR"));

    let fetched = repo::find_by_id_scoped(&pool, company_id, created.id)
        .await
        .expect("find ok")
        .expect("row present");
    assert_eq!(fetched.id, created.id);
    assert_eq!(
        fetched.creditor_name.as_deref(),
        Some("Robert Schneider SA")
    );
}

/// Story 30-2a (AC1, AC4) — un document sans QR (`without_qr`) s'insère par le
/// dépôt et se relit avec ses champs QR à `None`.
#[sqlx::test(migrations = "./test-schema")]
async fn create_without_qr_roundtrip(pool: MySqlPool) {
    use kesh_db::entities::imported_supplier_invoice::DocumentMeta;
    let company_id = create_test_company(&pool, "Sans QR SA").await;
    let new = NewImportedSupplierInvoice::without_qr(
        company_id,
        DocumentMeta {
            storage_path: "sansqr.png".into(),
            original_filename: "ticket.png".into(),
            sha256: "hash-sans-qr".into(),
            mime_type: "image/png".into(),
            byte_size: 10,
        },
    );
    let created = repo::create(&pool, &new).await.expect("create sans QR");
    assert!(!created.has_qr());
    assert_eq!(created.creditor_name, None);
    assert_eq!(created.currency, None);
    assert_eq!(created.is_qr_iban, None);
    assert_eq!(created.original_filename, "ticket.png");
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_id_scoped_is_multitenant(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Company A").await;
    let company_b = create_test_company(&pool, "Company B").await;

    let created = repo::create(&pool, &new_invoice(company_a, "hash-a"))
        .await
        .expect("create for A");

    // Company B ne doit PAS voir le staging de Company A (anti-IDOR).
    let leaked = repo::find_by_id_scoped(&pool, company_b, created.id)
        .await
        .expect("query ok");
    assert!(leaked.is_none(), "company B a vu le staging de company A");

    // Mais A le voit bien.
    assert!(
        repo::find_by_id_scoped(&pool, company_a, created.id)
            .await
            .expect("query ok")
            .is_some()
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn duplicate_company_hash_is_rejected(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Acme SA").await;

    repo::create(&pool, &new_invoice(company_id, "same-hash"))
        .await
        .expect("first insert ok");

    let err = repo::create(&pool, &new_invoice(company_id, "same-hash"))
        .await
        .expect_err("doublon (company, hash) doit être rejeté");
    assert!(
        matches!(err, DbError::UniqueConstraintViolation(_)),
        "attendu UniqueConstraintViolation, obtenu {err:?}"
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn same_hash_across_companies_is_allowed(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Company A").await;
    let company_b = create_test_company(&pool, "Company B").await;

    // Le même fichier (hash) peut être importé par deux companies distinctes.
    repo::create(&pool, &new_invoice(company_a, "shared-hash"))
        .await
        .expect("insert A ok");
    repo::create(&pool, &new_invoice(company_b, "shared-hash"))
        .await
        .expect("insert B ok (hash partagé inter-company autorisé)");
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_company_hash_is_scoped(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Company A").await;
    let company_b = create_test_company(&pool, "Company B").await;

    repo::create(&pool, &new_invoice(company_a, "lookup-hash"))
        .await
        .expect("insert A");

    // Trouvé dans A.
    assert!(
        repo::find_by_company_hash(&pool, company_a, "lookup-hash")
            .await
            .expect("query ok")
            .is_some()
    );
    // Pas visible depuis B (scopé company).
    assert!(
        repo::find_by_company_hash(&pool, company_b, "lookup-hash")
            .await
            .expect("query ok")
            .is_none()
    );
    // Hash inconnu → None.
    assert!(
        repo::find_by_company_hash(&pool, company_a, "no-such-hash")
            .await
            .expect("query ok")
            .is_none()
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn list_by_status_is_scoped_and_filtered(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Company A").await;
    let company_b = create_test_company(&pool, "Company B").await;

    repo::create(&pool, &new_invoice(company_a, "a1"))
        .await
        .expect("a1");
    repo::create(&pool, &new_invoice(company_a, "a2"))
        .await
        .expect("a2");
    repo::create(&pool, &new_invoice(company_b, "b1"))
        .await
        .expect("b1");

    // Company A : 2 stagings 'to_complete'.
    let a_pending = repo::list_by_status(&pool, company_a, "to_complete")
        .await
        .expect("list A");
    assert_eq!(a_pending.len(), 2);
    assert!(a_pending.iter().all(|i| i.company_id == company_a));

    // Company B : 1 seul (pas de fuite cross-company).
    let b_pending = repo::list_by_status(&pool, company_b, "to_complete")
        .await
        .expect("list B");
    assert_eq!(b_pending.len(), 1);

    // Statut sans staging → vide.
    let a_completed = repo::list_by_status(&pool, company_a, "completed")
        .await
        .expect("list completed");
    assert!(a_completed.is_empty());
}

// ---------------------------------------------------------------------------
// Story 30-2a (AC4) — colonnes QR facultatives, « tout ou rien », unicité du
// lien vers la facture. En SQL brut : c'est la contrainte de la base qui est
// sous test, non le code qui la respecte.
// ---------------------------------------------------------------------------

/// Les sept colonnes « tout ou rien » et leur valeur d'un document avec QR.
const QR_COLUMNS: [(&str, &str); 7] = [
    ("creditor_iban", "'CH9300762011623852957'"),
    ("is_qr_iban", "0"),
    ("creditor_address_type", "'K'"),
    ("creditor_name", "'Fournisseur SA'"),
    ("creditor_country", "'CH'"),
    ("reference_type", "'NON'"),
    ("currency", "'CHF'"),
];

/// `INSERT` d'une ligne de staging dont la colonne `null_column` (ou aucune, ou
/// toutes) est laissée à `NULL`, les autres renseignées.
fn staging_insert_sql(company_id: i64, hash: &str, null_columns: &[&str]) -> String {
    let cols: Vec<&str> = QR_COLUMNS.iter().map(|(c, _)| *c).collect();
    let vals: Vec<&str> = QR_COLUMNS
        .iter()
        .map(|(c, v)| if null_columns.contains(c) { "NULL" } else { *v })
        .collect();
    format!(
        "INSERT INTO imported_supplier_invoices \
         (company_id, file_hash, storage_path, original_filename, mime_type, byte_size, {}) \
         VALUES ({company_id}, '{hash}', '{hash}.png', 'doc.png', 'image/png', 10, {})",
        cols.join(", "),
        vals.join(", ")
    )
}

/// Un document sans QR (les sept à `NULL`) s'insère ; une ligne dont **une
/// seule** des sept est `NULL` est refusée, pour chacune des sept — sans quoi
/// une contrainte amputée d'une colonne (M-a5) survivrait.
#[sqlx::test(migrations = "./test-schema")]
async fn qr_columns_are_all_or_nothing(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Tout ou rien SA").await;
    let all: Vec<&str> = QR_COLUMNS.iter().map(|(c, _)| *c).collect();

    sqlx::query(&staging_insert_sql(company_id, "all-null", &all))
        .execute(&pool)
        .await
        .expect("un document sans QR : les sept colonnes à NULL");
    sqlx::query(&staging_insert_sql(company_id, "none-null", &[]))
        .execute(&pool)
        .await
        .expect("un document avec QR : les sept renseignées");

    for (column, _) in QR_COLUMNS {
        let err = sqlx::query(&staging_insert_sql(
            company_id,
            &format!("one-null-{column}"),
            &[column],
        ))
        .execute(&pool)
        .await
        .expect_err(&format!("{column} seule à NULL doit être refusée"));
        let msg = err.to_string();
        assert!(
            msg.contains("chk_imported_si_qr_all_or_nothing"),
            "{column} seule à NULL : refus attendu par la contrainte « tout ou rien », reçu {msg}"
        );
    }
}

/// Une facture fournisseur n'a qu'un document : `supplier_invoice_id` est
/// unique ; plusieurs `NULL` restent permis.
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_invoice_id_is_unique(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Unicité SA").await;
    // Connexion dédiée : la facture 999999 n'existe pas, la clé étrangère est
    // levée le temps des deux INSERT — c'est l'unicité qui est sous test.
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    for hash in ["n1", "n2"] {
        sqlx::query(&staging_insert_sql(company_id, hash, &[]))
            .execute(&mut *conn)
            .await
            .expect("plusieurs NULL permis");
    }
    sqlx::query(
        "UPDATE imported_supplier_invoices SET supplier_invoice_id = 999999 WHERE file_hash = 'n1'",
    )
    .execute(&mut *conn)
    .await
    .expect("premier lien");
    let err = sqlx::query(
        "UPDATE imported_supplier_invoices SET supplier_invoice_id = 999999 WHERE file_hash = 'n2'",
    )
    .execute(&mut *conn)
    .await
    .expect_err("un second document lié à la même facture doit être refusé");
    assert!(
        err.to_string().contains("uq_imported_supplier_invoice"),
        "refus attendu par uq_imported_supplier_invoice, reçu {err}"
    );
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
}

// ============================================================
// Story v017-5 (#854, C-v017-5-9) — T9 : la relecture n'écrit que des colonnes vides
// ============================================================

/// QR à quinze colonnes non `NULL`, valeurs **distinctes** de celles de
/// `new_invoice` sur chacune des quinze.
fn qr_quinze() -> kesh_db::entities::QrColumns {
    kesh_db::entities::QrColumns {
        creditor_iban: "CH9300762011623852957".into(),
        is_qr_iban: false,
        creditor_address_type: "S".into(),
        creditor_name: "Commune fictive de Valbois".into(),
        creditor_line1: Some("Place de la Maison de Commune".into()),
        creditor_line2: Some("12".into()),
        creditor_postal_code: Some("1999".into()),
        creditor_town: Some("Valbois-sur-Exemple".into()),
        creditor_country: "LI".into(),
        reference_type: "SCOR".into(),
        reference_value: Some("RF18539007547034".into()),
        amount: Some(dec!(1234.55)),
        currency: "EUR".into(),
        unstructured_message: Some("Message relu".into()),
        billing_information: Some("//S1/10/10201409".into()),
    }
}

/// Les quinze colonnes QR d'une ligne, en texte (booléen `0`/`1`, montant à quatre
/// décimales comme la colonne).
async fn quinze(pool: &MySqlPool, id: i64) -> Vec<Option<String>> {
    use sqlx::Row;
    let ligne = sqlx::query(
        "SELECT creditor_iban, CAST(is_qr_iban AS CHAR), creditor_address_type, creditor_name, \
         creditor_line1, creditor_line2, creditor_postal_code, creditor_town, creditor_country, \
         reference_type, reference_value, CAST(amount AS CHAR), currency, unstructured_message, \
         billing_information FROM imported_supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap();
    (0..15)
        .map(|i| ligne.try_get::<Option<String>, _>(i).unwrap())
        .collect()
}

fn quinze_de(q: &kesh_db::entities::QrColumns) -> Vec<Option<String>> {
    vec![
        Some(q.creditor_iban.clone()),
        Some(if q.is_qr_iban { "1" } else { "0" }.to_string()),
        Some(q.creditor_address_type.clone()),
        Some(q.creditor_name.clone()),
        q.creditor_line1.clone(),
        q.creditor_line2.clone(),
        q.creditor_postal_code.clone(),
        q.creditor_town.clone(),
        Some(q.creditor_country.clone()),
        Some(q.reference_type.clone()),
        q.reference_value.clone(),
        q.amount.map(|a| format!("{a:.4}")),
        Some(q.currency.clone()),
        q.unstructured_message.clone(),
        q.billing_information.clone(),
    ]
}

/// Document sans QR (`without_qr`) d'empreinte `file_hash`.
fn new_invoice_without_qr(company_id: i64, file_hash: &str) -> NewImportedSupplierInvoice {
    NewImportedSupplierInvoice::without_qr(
        company_id,
        kesh_db::entities::imported_supplier_invoice::DocumentMeta {
            storage_path: format!("{file_hash}.png"),
            original_filename: "scan.png".into(),
            sha256: file_hash.into(),
            mime_type: "image/png".into(),
            byte_size: 10,
        },
    )
}

async fn reactiver_avec_qr(pool: &MySqlPool, company_id: i64, id: i64) -> bool {
    let mut tx = pool.begin().await.unwrap();
    let r = repo::reactivate_to_complete_with_qr(&mut tx, company_id, id, &qr_quinze())
        .await
        .expect("reactivate_to_complete_with_qr");
    tx.commit().await.unwrap();
    r
}

async fn statut_et_facture(pool: &MySqlPool, id: i64) -> (String, Option<i64>) {
    sqlx::query_as(
        "SELECT status, supplier_invoice_id FROM imported_supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// T9 — `reactivate_to_complete_with_qr` : (1) ligne `discarded` **pourvue** d'un
/// QR différent → `false`, quinze colonnes inchangées ; (2) ligne `discarded`
/// sans QR portant un `supplier_invoice_id` résiduel → `true`, quinze colonnes
/// écrites, pointeur remis à `NULL` ; (3) ligne `discarded` sans QR → `true` ;
/// (4) ligne `to_complete` sans QR → `false`.
#[sqlx::test(migrations = "./test-schema")]
async fn reactivate_with_qr_writes_only_empty_discarded_rows(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Relecture SA").await;
    let attendu = quinze_de(&qr_quinze());

    // (1) Ligne pourvue d'un QR différent.
    let pourvue = repo::create(&pool, &new_invoice(company_id, "t9-pourvue"))
        .await
        .unwrap()
        .id;
    sqlx::query("UPDATE imported_supplier_invoices SET status = 'discarded' WHERE id = ?")
        .bind(pourvue)
        .execute(&pool)
        .await
        .unwrap();
    let avant = quinze(&pool, pourvue).await;
    assert_ne!(avant, attendu, "assertion de montage : valeurs distinctes");
    assert!(!reactiver_avec_qr(&pool, company_id, pourvue).await);
    assert_eq!(quinze(&pool, pourvue).await, avant);
    assert_eq!(statut_et_facture(&pool, pourvue).await.0, "discarded");

    // (2) Ligne sans QR, `discarded`, portant un pointeur résiduel.
    let residuelle = repo::create(&pool, &new_invoice_without_qr(company_id, "t9-residuelle"))
        .await
        .unwrap()
        .id;
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE imported_supplier_invoices SET status = 'discarded', supplier_invoice_id = 999999 \
         WHERE id = ?",
    )
    .bind(residuelle)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);
    assert!(reactiver_avec_qr(&pool, company_id, residuelle).await);
    assert_eq!(quinze(&pool, residuelle).await, attendu);
    assert_eq!(
        statut_et_facture(&pool, residuelle).await,
        ("to_complete".to_string(), None)
    );

    // (3) Ligne sans QR, `discarded`.
    let vide = repo::create(&pool, &new_invoice_without_qr(company_id, "t9-vide"))
        .await
        .unwrap()
        .id;
    sqlx::query("UPDATE imported_supplier_invoices SET status = 'discarded' WHERE id = ?")
        .bind(vide)
        .execute(&pool)
        .await
        .unwrap();
    assert!(reactiver_avec_qr(&pool, company_id, vide).await);
    assert_eq!(quinze(&pool, vide).await, attendu);
    assert_eq!(statut_et_facture(&pool, vide).await.0, "to_complete");

    // (4) Ligne `to_complete` sans QR.
    let active = repo::create(&pool, &new_invoice_without_qr(company_id, "t9-active"))
        .await
        .unwrap()
        .id;
    assert!(!reactiver_avec_qr(&pool, company_id, active).await);
    assert!(quinze(&pool, active).await.iter().all(Option::is_none));
}
