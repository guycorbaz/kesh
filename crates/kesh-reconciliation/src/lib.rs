//! kesh-reconciliation — Réconciliation bancaire (Story 8-4 + 8-5a-base).
//!
//! Modules :
//!
//! - [`matching`] : helper pure `propose_matches` calculant des
//!   propositions d'appariement transaction bancaire ↔ facture avec
//!   un score de confiance pondéré (montant 0.50 + référence 0.40 +
//!   contact 0.10).
//! - [`mutex`] : helper `with_account_lock` utilisant un advisory
//!   lock MariaDB `GET_LOCK('reconcile:{company}:{account}', timeout)`
//!   pour serializer les flows accept/reject sur le même compte
//!   bancaire.
//! - [`manual`] : (Story 8-5a-base FR45) helper public
//!   `build_journal_entry_for_counterparty` pure (zéro I/O) qui
//!   construit une `NewJournalEntry` à 2 lignes équilibrées sign-aware
//!   pour le flow `POST /api/v1/reconciliation/manual`. Signature
//!   stable contractée pour 8-5a-bis (split) et 8-5b (rules engine).
//! - [`errors`] : enum [`ReconciliationError`] commun aux modules.

pub mod errors;
pub mod manual;
pub mod matching;
pub mod mutex;

pub use errors::ReconciliationError;
pub use manual::build_journal_entry_for_counterparty;
pub use matching::{MatchProposal, MatchScore, propose_matches};
pub use mutex::with_account_lock;
