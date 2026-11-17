//! Story 43-1a (#823) — un règlement pour plusieurs factures : migration,
//! plafond de la différence, lettrage par ligne, règlement groupé client et son
//! dé-rapprochement.
//!
//! Tous les tests montent le squash (`test-schema`), que `test_schema_guard.rs`
//! prouve égal au schéma de `MIGRATOR`.

/// Story 35-5a0 — la date du jour de l'horloge de test.
#[allow(dead_code)]
#[path = "support/jour.rs"]
mod jour;

use kesh_db::repositories::company_invoice_settings::{
    DEFAULT_GROUP_DIFFERENCE_MAX, group_settlement_max_difference,
};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// La CHECK « ligne ⇒ écriture » (AC1.3, M2)
// ---------------------------------------------------------------------------

/// AC1.3 — poser la ligne de règlement d'une facture fournisseur sans son
/// écriture est refusé par la base (`chk_supplier_invoices_settlement_line`) ;
/// le fragment partagé `CLEAR_SETTLEMENT_SQL`, lui, vide les deux ensemble.
///
/// Sonde sous `FOREIGN_KEY_CHECKS = 0`, sur une seule connexion : la CHECK reste
/// active, seules les clés étrangères (vers des lignes factices) sont suspendues.
#[sqlx::test(migrations = "./test-schema")]
async fn a_settlement_line_without_its_entry_is_refused(pool: MySqlPool) {
    use kesh_db::repositories::supplier_invoices::CLEAR_SETTLEMENT_SQL;
    let mut conn = pool.acquire().await.expect("connexion");
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    let refus = sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, invoice_date, status, purchase_journal_entry_id, \
          settlement_journal_entry_line_id) \
         VALUES (1, 1, '2026-11-01', 'open', 1, 7)",
    )
    .execute(&mut *conn)
    .await;
    let err = refus.expect_err("ligne sans écriture acceptée");
    assert!(
        err.to_string()
            .contains("chk_supplier_invoices_settlement_line"),
        "refus inattendu : {err}"
    );
    let id = sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, invoice_date, status, purchase_journal_entry_id, \
          settlement_type, settlement_bank_account_id, settlement_journal_entry_id, \
          settlement_journal_entry_line_id, paid_at) \
         VALUES (1, 1, '2026-11-01', 'paid', 1, 'bank_transfer', 1, 3, 7, NOW())",
    )
    .execute(&mut *conn)
    .await
    .expect("ligne avec son écriture")
    .last_insert_id();
    // Vider l'écriture SEULE (la forme d'avant la 43-1a) : refusé.
    let seule = sqlx::query(
        "UPDATE supplier_invoices SET status = 'open', settlement_type = NULL, \
         settlement_bank_account_id = NULL, settlement_journal_entry_id = NULL, paid_at = NULL \
         WHERE id = ?",
    )
    .bind(id)
    .execute(&mut *conn)
    .await;
    assert!(seule.is_err(), "écriture vidée sans sa ligne acceptée");
    // Le fragment partagé : accepté, les deux à NULL.
    sqlx::query(&format!(
        "UPDATE supplier_invoices SET status = 'open', {CLEAR_SETTLEMENT_SQL} WHERE id = ?"
    ))
    .bind(id)
    .execute(&mut *conn)
    .await
    .expect("fragment partagé");
    let (entry, line): (Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT settlement_journal_entry_id, settlement_journal_entry_line_id \
         FROM supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert_eq!((entry, line), (None, None));
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Le plafond de la différence (AC3 bis, C-43-24)
// ---------------------------------------------------------------------------

/// AC3 bis, L9 — la constante du code et le `DEFAULT` de la colonne ne
/// dérivent pas : une seule valeur par défaut, lue des deux côtés.
#[sqlx::test(migrations = "./test-schema")]
async fn group_max_default_matches_the_column_default(pool: MySqlPool) {
    let column_default: String = sqlx::query_scalar(
        "SELECT COLUMN_DEFAULT FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'company_invoice_settings' \
           AND COLUMN_NAME = 'group_settlement_max_difference'",
    )
    .fetch_one(&pool)
    .await
    .expect("défaut de la colonne");
    let column_default: Decimal = column_default.parse().expect("défaut décimal");
    assert_eq!(column_default, DEFAULT_GROUP_DIFFERENCE_MAX);
    assert_eq!(DEFAULT_GROUP_DIFFERENCE_MAX, dec!(5.0000));
}

/// AC3 bis — la valeur de la société quand elle a sa ligne de réglages ; la
/// constante sinon.
#[sqlx::test(migrations = "./test-schema")]
async fn group_max_reads_the_setting_or_the_default(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.expect("seed");
    // Une société sans ligne de réglages (identifiant inexistant) : la constante.
    assert_eq!(
        group_settlement_max_difference(&pool, s.company_id + 1000)
            .await
            .expect("lecture"),
        DEFAULT_GROUP_DIFFERENCE_MAX
    );
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id) VALUES (?) \
         ON DUPLICATE KEY UPDATE company_id = company_id",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .expect("ligne de réglages");
    assert_eq!(
        group_settlement_max_difference(&pool, s.company_id)
            .await
            .expect("lecture"),
        dec!(5.0000)
    );
    sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 0 \
         WHERE company_id = ?",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await
    .expect("plafond à zéro");
    assert_eq!(
        group_settlement_max_difference(&pool, s.company_id)
            .await
            .expect("lecture"),
        Decimal::ZERO
    );
    // La CHECK de la colonne : au-delà de 100, refusé par la base.
    let refus = sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 100.0001 \
         WHERE company_id = ?",
    )
    .bind(s.company_id)
    .execute(&pool)
    .await;
    assert!(refus.is_err(), "plafond au-delà de 100 accepté");
}
