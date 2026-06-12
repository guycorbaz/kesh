//! Entité `PasswordResetToken` : magic-link de réinitialisation de mot de
//! passe self-service (Story 17-4a, recovery #122).
//!
//! **Sécurité (DC3, calque `ApiKey` 17-2a)** : `token_hash` contient le
//! `SHA-256(token)` hex (64 chars), JAMAIS le token en clair. Le token brut
//! (≥160 bits OsRng base62) ne vit que dans l'URL du lien email. `Debug` est
//! implémenté manuellement pour masquer `token_hash` (défense en profondeur).
//! Pas de `Serialize`/`Deserialize` (jamais exposé en JSON).

use chrono::NaiveDateTime;

/// Token de réinitialisation persisté en base.
///
/// Usage unique (`used_at`) + TTL (`expires_at`, 30 min — DC8). Le repo
/// `find_valid_by_hash` filtre `used_at IS NULL AND expires_at > NOW(3)`.
#[derive(Clone, sqlx::FromRow)]
pub struct PasswordResetToken {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl std::fmt::Debug for PasswordResetToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordResetToken")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("token_hash", &"***")
            .field("expires_at", &self.expires_at)
            .field("used_at", &self.used_at)
            .field("created_at", &self.created_at)
            .finish()
    }
}
