//! Une facture fournisseur **existante** au TTC sous le centime — Story v017-3b
//! (#687, AC-9, AC-11, AC-19).
//!
//! Depuis la v0.17.0, Kesh refuse de créer une telle facture : l'état ne se
//! produit plus que dans une base remplie par une version 0.16.x. Les tests le
//! montent donc en deux temps, sur leur base éphémère seulement : la facture se
//! crée au centime (`1 × 100.00`, taux 0), puis le fichier
//! `montage-sous-centime.sql` l'amène à `100.0040` — le même fichier que
//! l'opérateur joue à la main sur une base de démonstration.
//!
//! Partagé par `#[path]` entre les binaires de test (chaque binaire est un crate).

use rust_decimal::Decimal;
use sqlx::MySqlPool;

/// Le montage, écrit une seule fois (validation P3, R7).
const MONTAGE: &str = include_str!("montage-sous-centime.sql");

/// Amène la facture `invoice_id` (créée à `1 × 100.00`, taux 0, ouverte) à
/// `100.0040`, puis vérifie que les quatre valeurs ont bougé.
///
/// `raw_sql` joue toutes les instructions sur **une** connexion : la variable de
/// session `@invoice_id`, posée en tête, y reste visible (précédents
/// multi-instructions : `onboarding_audit_e2e.rs:3022`, `:3085`).
pub async fn amener_sous_le_centime(pool: &MySqlPool, invoice_id: i64) {
    let texte = format!("SET @invoice_id := {invoice_id};\n{MONTAGE}");
    sqlx::raw_sql(&texte)
        .execute(pool)
        .await
        .expect("montage sous le centime");
    let (total, ligne, debit, credit): (Decimal, Decimal, Decimal, Decimal) = sqlx::query_as(
        "SELECT si.total_amount, \
                (SELECT line_total FROM supplier_invoice_lines WHERE supplier_invoice_id = si.id), \
                (SELECT SUM(debit) FROM journal_entry_lines WHERE entry_id = si.purchase_journal_entry_id), \
                (SELECT SUM(credit) FROM journal_entry_lines WHERE entry_id = si.purchase_journal_entry_id) \
         FROM supplier_invoices si WHERE si.id = ?",
    )
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .expect("relecture du montage");
    let attendu = Decimal::new(1_000_040, 4);
    assert_eq!(
        (total, ligne, debit, credit),
        (attendu, attendu, attendu, attendu),
        "montage : la facture {invoice_id} n'était pas à 1 × 100.00, taux 0, ouverte"
    );
}
