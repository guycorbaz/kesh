//! La politique de la règle anti-doublon, commune aux dépôts qui refusent un
//! doublon confirmable (Story 48-12a : factures fournisseur ; Story 53-1a :
//! écritures saisies ; Story 53-1b : conversion de devises, à venir).
//!
//! Une seule définition (DRY) : chaque dépôt la réexporte (`pub use`) sous son
//! propre chemin. Les règles — ce qui compte comme doublon, ce que la
//! confirmation lève — restent propres à chaque dépôt.

/// Ce que fait la règle anti-doublon d'un refus **confirmable** (Story 48-12a,
/// D5 ; patron du `confirm_designated` d'`accounts::archive`).
///
/// - [`DuplicatePolicy::Refuse`] : tout doublon est refusé. C'est la politique
///   des fonctions qui ont gardé leur signature : seeds, tests et futurs
///   appelants passent par elle sans le savoir (factures fournisseur :
///   `create_draft`, `create_validated`, `update_draft`, `unvalidate`… ;
///   écritures saisies : `journal_entries::create`).
/// - [`DuplicatePolicy::Confirmed`] : l'appelant a reçu `confirmNotDuplicate:
///   true`. Il lève les refus **que le dépôt déclare levables** — les factures
///   fournisseur par `DuplicateRule::lifted_by_confirmation` (la règle 2 ; la
///   règle 1 jamais, à la saisie), les écritures saisies toujours (Story 53-1a,
///   Q4). Un refus levé ainsi s'inscrit au journal d'audit
///   (`supplier_invoice.duplicate_confirmed`, `journal_entry.duplicate_confirmed`),
///   dans la même transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuplicatePolicy {
    /// Tout doublon est refusé.
    #[default]
    Refuse,
    /// L'utilisateur ou l'intégration a confirmé que ce n'est pas un doublon.
    Confirmed,
}

impl DuplicatePolicy {
    /// La politique que demande le champ `confirmNotDuplicate` d'une requête.
    pub fn from_confirmation(confirm_not_duplicate: bool) -> Self {
        if confirm_not_duplicate {
            Self::Confirmed
        } else {
            Self::Refuse
        }
    }
}
