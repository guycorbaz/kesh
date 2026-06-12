-- Story 17-4a — Table `password_reset_tokens` : magic-link de réinitialisation
-- de mot de passe self-service (recovery #122).
--
-- Non-breaking migration (P1/P3 CLAUDE.md) : CREATE TABLE d'une nouvelle
-- entité — les anciens binaires l'ignorent. Pas de bump
-- `kesh_version_min_required` requis.
--
-- Sécurité (DC3, calque api_keys 17-2a) : on ne stocke JAMAIS le token en
-- clair. Seul le SHA-256(token) hex (CHAR(64)) est persisté, avec un index
-- UNIQUE pour un lookup O(1). Le token brut (≥160 bits OsRng base62) ne vit
-- que dans l'URL du lien email. Une fuite DB ne permet pas la prise de
-- contrôle de compte.
--
-- Usage unique + TTL (DC8) : `used_at` marque la consommation (single-use),
-- `expires_at` borne la validité (30 min). Le repo filtre
-- `used_at IS NULL AND expires_at > NOW(3)`.
--
-- FK ON DELETE CASCADE (DC11) : tokens éphémères sans valeur d'audit propre
-- (l'audit du reset vit dans `audit_log`, FK RESTRICT séparée). Supprimer un
-- user purge ses tokens pendants.

CREATE TABLE password_reset_tokens (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    token_hash CHAR(64) NOT NULL,
    expires_at DATETIME(3) NOT NULL,
    used_at DATETIME(3) NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_prt_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT uq_prt_token_hash UNIQUE (token_hash),
    INDEX idx_prt_user (user_id),
    INDEX idx_prt_expires (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
