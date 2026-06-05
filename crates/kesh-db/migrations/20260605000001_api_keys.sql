-- Story 17-2a — Table api_keys : clés d'accès API externes (PAT) pour
-- intégrations IA & logiciels tiers (#100).
--
-- Non-breaking migration (P1/P3 CLAUDE.md) : CREATE TABLE d'une nouvelle
-- entité — les anciens binaires l'ignorent. Pas de bump
-- `kesh_version_min_required` requis.
--
-- Sécurité (DC1) : on ne stocke JAMAIS le secret en clair. Seul le
-- SHA-256(token) hex (CHAR(64)) est persisté, avec un index UNIQUE qui
-- permet un lookup auth O(1) par un seul SELECT indexé (pattern GitHub PAT).
-- Le secret aléatoire ≥ 160 bits (haute entropie) rend le hashing lent
-- (Argon2id) inutile par requête.
--
-- Multi-tenant (DC + KF-002) : 1 company_id par clé, FK ON DELETE RESTRICT.
-- `created_by_user_id` conserve la responsabilité (imputabilité de la
-- création même après révocation).

CREATE TABLE api_keys (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    created_by_user_id BIGINT NOT NULL COMMENT 'Créateur de la clé — le PAT authentifie en son nom (DC2)',
    name VARCHAR(255) NOT NULL COMMENT 'Libellé human-readable de l''intégration',
    key_hash CHAR(64) NOT NULL COMMENT 'SHA-256(token) hex — jamais le secret en clair (DC1)',
    scope VARCHAR(16) NOT NULL COMMENT 'read | read-write (DC3)',
    expires_at DATETIME(3) NULL COMMENT 'Expiration optionnelle ; NULL = permanente jusqu''à révocation',
    last_used_at DATETIME(3) NULL COMMENT 'Best-effort, mis à jour au lookup auth (eventual consistency)',
    revoked_at DATETIME(3) NULL COMMENT 'Soft-delete — révocation immédiate (find_active exclut)',
    version INT NOT NULL DEFAULT 1 COMMENT 'Optimistic lock pour la révocation',
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_api_keys_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_api_keys_created_by FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT chk_api_keys_name_nonempty CHECK (CHAR_LENGTH(TRIM(name)) > 0),
    CONSTRAINT chk_api_keys_scope CHECK (scope IN ('read', 'read-write')),
    CONSTRAINT uq_api_keys_key_hash UNIQUE (key_hash),
    INDEX idx_api_keys_company (company_id),
    INDEX idx_api_keys_created (created_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
