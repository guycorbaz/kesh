//! Entité `RefreshToken` : jeton de rafraîchissement persistant.
//!
//! **Sécurité** : `RefreshToken` et `NewRefreshToken` ne dérivent PAS
//! `Debug` classique — impl manuelle masquant le champ `token` pour éviter
//! toute fuite dans les logs `tracing`. Pas de `Serialize`/`Deserialize`
//! non plus (défense en profondeur : un refresh_token ne doit jamais
//! fuiter en JSON).
//!
//! **Note sur le stockage plaintext** : dans cette story (1.5), le `token`
//! est stocké en clair en base. La story 1.6 ajoutera une colonne
//! `token_hash` (SHA-256) + rotation à chaque refresh pour éliminer
//! l'exposition session-takeover sur dump DB.

use chrono::NaiveDateTime;

/// Jeton de rafraîchissement persisté en base.
///
/// Créé au login (story 1.5), utilisé par le flux refresh (story 1.6),
/// invalidé au logout. Le champ `revoked_at` permet l'idempotence du
/// logout sans suppression destructive.
#[derive(Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
    /// Raison de la révocation : "logout", "rotation", "password_change",
    /// "admin_disable", "theft_detected". `None` = non révoqué ou pré-migration.
    pub revoked_reason: Option<String>,
}

impl std::fmt::Debug for RefreshToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefreshToken")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("token", &"***")
            .field("expires_at", &self.expires_at)
            .field("created_at", &self.created_at)
            .field("revoked_at", &self.revoked_at)
            .field("revoked_reason", &self.revoked_reason)
            .finish()
    }
}

/// Données pour la création d'un refresh token.
#[derive(Clone)]
pub struct NewRefreshToken {
    pub user_id: i64,
    pub token: String,
    pub expires_at: NaiveDateTime,
}

impl std::fmt::Debug for NewRefreshToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewRefreshToken")
            .field("user_id", &self.user_id)
            .field("token", &"***")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}
