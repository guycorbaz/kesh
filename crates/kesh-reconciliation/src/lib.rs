//! kesh-reconciliation — Réconciliation bancaire (Story 8-4).
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
//! - [`errors`] : enum [`ReconciliationError`] commun aux 2 modules.

pub mod errors;
pub mod matching;
pub mod mutex;

pub use errors::ReconciliationError;
pub use matching::{MatchProposal, MatchScore, propose_matches};
pub use mutex::with_account_lock;
