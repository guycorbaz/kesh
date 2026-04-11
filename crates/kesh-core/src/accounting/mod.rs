//! Moteur comptable — logique de partie double, immutabilité, balance.
//!
//! Ce module contient la logique métier pure (sans I/O) qui garantit
//! l'intégrité des écritures comptables conformément au Code des
//! obligations suisse (art. 957-964) :
//!
//! - **Partie double** : chaque écriture doit être équilibrée
//!   (`total_debit == total_credit`).
//! - **Immutabilité post-clôture** : vérifiée au niveau persistance
//!   (`kesh-db`), pas ici.
//! - **Intégrité arithmétique** : montants en `rust_decimal`, jamais
//!   de `f64`.
//!
//! L'API principale est [`validate`], qui transforme un
//! [`JournalEntryDraft`] en [`BalancedEntry`] si toutes les règles
//! métier passent.

pub mod balance;

pub use balance::{
    validate, BalancedEntry, Journal, JournalEntryDraft, JournalEntryLineDraft,
};
