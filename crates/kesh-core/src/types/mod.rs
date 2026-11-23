//! Types métier forts avec validation intégrée.
//!
//! Chaque type garantit l'intégrité des données dès la construction.
//! Aucune instance invalide ne peut exister en mémoire.

mod che_number;
mod iban;
mod money;
mod qr_iban;

pub use che_number::CheNumber;
pub use iban::Iban;
pub use money::{Money, format_two_decimals, round_for_display};
pub use qr_iban::QrIban;
