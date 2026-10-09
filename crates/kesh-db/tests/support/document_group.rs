//! Aide de montage **réservée aux tests** — Story 15-1a2-0 (#518, D4 ; choix
//! C-15-1a2-0-1) : poser à la main un groupe de lettrage d'origine `document`,
//! la forme que produira la synchronisation des pièces de la 15-1a2-i.
//!
//! ⛔ **Pourquoi en SQL brut** : avant la 15-1a2-i, aucun chemin de production ne
//! pose d'origine `document` — le rang 2 bis est **dormant**. Le seul moyen de
//! l'éprouver est de poser le groupe ici, hors de `crates/*/src` : les
//! détecteurs lexicaux du lettrage (`letterings_lexical.rs`) ne balaient que le
//! code de production, et c'est juste — un groupe posé ainsi ne prétend à rien
//! d'atteignable.
//!
//! Ce fichier n'appartient à aucune crate ; il est **inclus par `#[path]`** là
//! où on l'emploie : `crates/kesh-db/tests/{invoice_settlement,
//! invoice_write_off, supplier_invoices_repository}.rs` et
//! `crates/kesh-api/tests/{invoice_echeancier_e2e, reconciliation_e2e,
//! supplier_settlement_cancel_e2e}.rs`.
#![allow(dead_code)]

use rust_decimal::Decimal;
use sqlx::MySqlPool;

/// Pose un groupe `document` sur **la** ligne du compte `account_id` de chacune
/// des écritures `entry_ids`, et rend sa clé.
///
/// Forme de la synchronisation : somme `débit − crédit` nulle, même compte,
/// clé = plus petite ligne. ⛔ **Assertion de montage** (D4 point 2) : une ligne
/// par écriture exactement, nombre de lignes marquées égal à l'attendu, somme
/// nulle sur la clé, clé = plus petite ligne — sans elle, un `UPDATE` qui ne
/// trouverait rien laisserait vertes, à vide, les assertions négatives.
pub async fn poser_groupe_document(pool: &MySqlPool, account_id: i64, entry_ids: &[i64]) -> i64 {
    let marqueurs = vec!["?"; entry_ids.len()].join(", ");
    let sql = format!(
        "SELECT id FROM journal_entry_lines WHERE account_id = ? AND entry_id IN ({marqueurs}) \
         ORDER BY id"
    );
    let mut q = sqlx::query_scalar::<_, i64>(&sql).bind(account_id);
    for e in entry_ids {
        q = q.bind(*e);
    }
    let lignes = q.fetch_all(pool).await.expect("lignes du groupe");
    assert_eq!(
        lignes.len(),
        entry_ids.len(),
        "montage : une ligne du compte {account_id} par écriture {entry_ids:?}"
    );
    let cle = lignes[0];

    let marqueurs = vec!["?"; lignes.len()].join(", ");
    let sql = format!(
        "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'document' \
         WHERE id IN ({marqueurs}) AND lettering_key IS NULL"
    );
    let mut q = sqlx::query(&sql).bind(cle);
    for l in &lignes {
        q = q.bind(*l);
    }
    let marquees = q
        .execute(pool)
        .await
        .expect("pose du groupe")
        .rows_affected();
    assert_eq!(marquees, lignes.len() as u64, "montage : lignes marquées");

    let (somme, min, n): (Decimal, i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(debit - credit), 0), MIN(id), COUNT(*) FROM journal_entry_lines \
         WHERE lettering_key = ? AND lettering_origin = 'document'",
    )
    .bind(cle)
    .fetch_one(pool)
    .await
    .expect("contrôle du groupe");
    assert_eq!(somme, Decimal::ZERO, "montage : groupe à somme nulle");
    assert_eq!(min, cle, "montage : clé = plus petite ligne");
    assert_eq!(n, lignes.len() as i64, "montage : taille du groupe");
    cle
}

/// Nombre de lignes marquées sous la clé `cle` (« les marques sont intactes »).
pub async fn lignes_du_groupe(pool: &MySqlPool, cle: i64) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = ? \
         AND lettering_origin = 'document'",
    )
    .bind(cle)
    .fetch_one(pool)
    .await
    .expect("lignes marquées")
}
