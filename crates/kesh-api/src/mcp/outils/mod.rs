//! Les outils de production du serveur MCP. Tout `impl McpTool` de `src/` vit
//! ici (garde `outils_et_politique_en_leur_lieu`) : `kesh_version` (32-6a), qui
//! ne rend aucune donnée des livres, et les quinze outils de lecture de la
//! 32-9a — livres (`books:read`) et factures (`invoices:read`). Chacun rend ses
//! données par `sortie::donnees` seulement (garde
//! `sorties_sans_serialisation_parallele`).

pub mod balance_agee;
pub mod commun;
pub mod ecritures;
pub mod exercices;
pub mod factures;
pub mod factures_fournisseur;
pub mod grand_livre;
pub mod kesh_version;
pub mod plan_comptable;
pub mod postes_ouverts;
pub mod rapports;
