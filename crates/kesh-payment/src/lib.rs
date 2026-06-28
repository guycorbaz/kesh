//! Crate de génération de fichiers de paiement ISO 20022.
//!
//! Story 12.3 — générateur `pain.001.001.09` (CustomerCreditTransferInitiation),
//! variante Swiss Payment Standards (SIX). Pur, sans I/O DB : les structures
//! d'entrée sont supposées déjà validées (IBAN/QR-IBAN/QRR vérifiés en amont
//! côté `kesh-db`).

pub mod pain001;
