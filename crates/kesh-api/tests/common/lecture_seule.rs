//! La `Lecture` de test en lecture seule — Story 32-4a (C-32-4a-9), déplacée
//! de `tests/services_motif.rs` (Story 32-3) pour servir aussi
//! `tests/services_rollout.rs`.
//!
//! Chaque connexion du pool passe `SET SESSION TRANSACTION READ ONLY` : toute
//! écriture y est refusée par MariaDB (erreur 1792, témoin
//! `ac9_temoin_la_lecture_de_test_refuse_l_ecriture` de `services_motif.rs`).
//! Aucun privilège requis. C'est le mode de session que la connexion du MCP
//! emploiera (Q11, tranchée le 2026-10-24 ; réalisé par la 32-6), sur un pool
//! de test et sans la fabrication de production (`AppState::lecture()`).

use kesh_api::services::poignees::Lecture;
use sqlx::MySqlPool;

/// Pool dont chaque connexion passe `SET SESSION TRANSACTION READ ONLY` — mêmes
/// options de connexion que le pool du test, aucun privilège requis.
pub async fn pool_lecture_seule(pool: &MySqlPool) -> MySqlPool {
    let options = (*pool.connect_options()).clone();
    sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET SESSION TRANSACTION READ ONLY")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .expect("pool en lecture seule")
}

/// Une `Lecture` sur [`pool_lecture_seule`].
pub async fn lecture_seule(pool: &MySqlPool) -> Lecture {
    Lecture::new(pool_lecture_seule(pool).await)
}
