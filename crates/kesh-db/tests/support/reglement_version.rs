//! Règlement d'une facture client à sa version courante — Story 34-5 (#951).
//!
//! `invoice_settlements_write::settle_invoice` exige la version de la facture
//! (`expected_version`), comparée sous le verrou de l'étape (1). Les tests qui
//! ne portent pas sur ce contrôle règlent à la version lue juste avant, par
//! cette aide unique plutôt que par une lecture recopiée à chaque appel
//! (C-e343-13). Les tests du contrôle lui-même passent leur version à la main.
//!
//! Inclus par `#[path = "support/reglement_version.rs"]` : chaque binaire de test
//! compile sa propre copie, d'où l'`allow(dead_code)`.
#![allow(dead_code)]

use chrono::NaiveDate;
use kesh_db::entities::SettlementChoice;
use kesh_db::errors::DbError;
use kesh_db::repositories::invoice_settlements_write::{self, SettlementOutcome};
use rust_decimal::Decimal;
use sqlx::MySqlPool;

/// La version que la facture porte en base.
pub async fn version_de(pool: &MySqlPool, invoice_id: i64) -> i32 {
    sqlx::query_scalar("SELECT version FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_one(pool)
        .await
        .expect("version de la facture")
}

/// `settle_invoice` à la version courante de la facture : mêmes arguments, la
/// version en moins.
pub async fn settle_at_current_version(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    invoice_id: i64,
    choice: SettlementChoice,
    amount: Decimal,
    settled_on: NaiveDate,
) -> Result<SettlementOutcome, DbError> {
    let version = version_de(pool, invoice_id).await;
    invoice_settlements_write::settle_invoice(
        pool,
        user_id,
        company_id,
        invoice_id,
        choice,
        amount,
        settled_on,
        version,
    )
    .await
}
