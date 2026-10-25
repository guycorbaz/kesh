//! La `Lecture` de test en lecture seule — Story 32-4a (C-32-4a-9), déplacée
//! de `tests/services_motif.rs` (Story 32-3) pour servir aussi
//! `tests/services_rollout.rs`.
//!
//! Chaque connexion du pool passe `SET SESSION TRANSACTION READ ONLY` : toute
//! écriture y est refusée par MariaDB (erreur 1792, témoin
//! `ac9_temoin_la_lecture_de_test_refuse_l_ecriture` de `services_motif.rs`).
//! Aucun privilège requis. Depuis la Story 32-6a (C-32-6-26), le pool est
//! fabriqué par **la fabrique de production** du pool des outils du MCP,
//! `kesh_db::pool::pool_lecture_seule` — si bien que les tests de services
//! s'exécutent sur la connexion même que le MCP emploie (chaque emprunt part
//! d'une connexion neuve, fermée à sa restitution).

use kesh_api::services::poignees::Lecture;
use sqlx::MySqlPool;

/// Pool dont chaque connexion passe `SET SESSION TRANSACTION READ ONLY` — mêmes
/// options de connexion que le pool du test, aucun privilège requis, quatre
/// connexions au plus, délai d'instruction de production (signature inchangée
/// depuis la 32-4a).
pub async fn pool_lecture_seule(pool: &MySqlPool) -> MySqlPool {
    kesh_db::pool::pool_lecture_seule(pool, 4, kesh_api::mcp::DELAI_OUTIL)
}

/// Une `Lecture` sur [`pool_lecture_seule`].
pub async fn lecture_seule(pool: &MySqlPool) -> Lecture {
    Lecture::new(pool_lecture_seule(pool).await)
}
