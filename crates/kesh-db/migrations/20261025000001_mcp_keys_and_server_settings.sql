-- Story 32-5a (refs #723) — les clés du serveur MCP et les réglages du
-- serveur, en base.
--
-- `mcp_keys` : second type de clé, propre au serveur MCP (0.16.0), DISTINCT
-- des clés d'API (`api_keys`, préfixe `kesh_pat_`) : préfixe `kesh_mcp_`,
-- portées fines (tableau JSON trié, sans doublon, non vide : books:read,
-- invoices:read, contacts:read, documents:read, prompts:write), politique de
-- validation avant envoi PAR CLÉ (`send_validation`, Q12 : `required` par
-- défaut). Comme pour `api_keys`, seul le condensat SHA-256 hex du jeton est
-- stocké (index UNIQUE), jamais le secret.
--
-- `mcp_server_settings` : une ligne par société AU PLUS (clé primaire
-- `company_id`), absente tant que l'administrateur n'a rien enregistré — la
-- lecture rend alors les défauts du code (`McpServerSettings::defaults`).
-- Serveur DÉSACTIVÉ par défaut ; filtre d'adresse ; plages admises,
-- mandataires de confiance et origines en tableaux JSON de chaînes
-- canoniques, validées à l'écriture par Kesh.
--
-- Les deux tables sont des tables APPLICATIVES : elles entrent à
-- `TABLES_TO_TRUNCATE` (crates/kesh-db/src/backup.rs), donc au `.keshbackup`.
-- Conséquence (Q4 de Guy, 2026-10-24) : le contrôle de couverture de l'import
-- exige l'égalité des inventaires de tables, si bien que LES SAUVEGARDES
-- ANTÉRIEURES À CETTE MIGRATION NE S'IMPORTENT PLUS ; et la fenêtre
-- d'importabilité de P7 se referme (le rejeu du lettrage `20261010000001`
-- sort du registre `POST_RESTORE_BACKFILLS`).
--
-- Politique (CLAUDE.md § Migration breaking policy) : NON BREAKING —
-- `CREATE TABLE` de deux entités neuves : pas de bump de
-- `kesh_version_min_required`. Comme toute migration, une 0.15.0 refuse
-- néanmoins de démarrer sur une base qui la porte (sqlx : migration appliquée
-- inconnue) ; le retour passe par la sauvegarde d'avant la mise à jour.
--
-- Deux contraintes s'écartent du patron d'`api_keys`, éprouvées à la main puis
-- par les tests de `crates/kesh-db/tests/mcp_keys_repository.rs` : le nom doit
-- porter un caractère qui n'est pas un blanc (`REGEXP`, car `TRIM` ne retire
-- que les espaces, pas une tabulation) ; le `CHECK` de `send_validation`
-- compare en collation BINAIRE SANS REMPLISSAGE (`utf8mb4_nopad_bin`), car la
-- collation `utf8mb4_unicode_ci` de la table ferait accepter `'Required'` ou
-- `'required '`, valeurs que le décodage strict de Kesh refuse. La colonne
-- garde la collation de la table : une collation binaire de COLONNE la ferait
-- lire comme `VARBINARY`.
--
-- Idempotence : tracked-by-sqlx (docs/migrations-idempotence-audit.md).
-- Triage P7 : aucune instruction n'écrit de données — ni registre ni
-- exemption.

CREATE TABLE mcp_keys (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    created_by_user_id BIGINT NOT NULL COMMENT 'Créateur — la clé lit en son nom, tant qu''il est actif et Administrateur',
    name VARCHAR(255) NOT NULL,
    key_hash CHAR(64) NOT NULL COMMENT 'SHA-256(jeton) hex — jamais le secret',
    scopes JSON NOT NULL COMMENT 'Tableau trié, sans doublon : books:read, invoices:read, contacts:read, documents:read, prompts:write',
    send_validation VARCHAR(16) NOT NULL DEFAULT 'required' COMMENT 'required | exempt (Q12)',
    expires_at DATETIME(3) NULL,
    last_used_at DATETIME(3) NULL,
    revoked_at DATETIME(3) NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_mcp_keys_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_mcp_keys_created_by FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT chk_mcp_keys_name_nonempty CHECK (name REGEXP '[^[:space:]]'),
    CONSTRAINT chk_mcp_keys_scopes CHECK (JSON_VALID(scopes) AND JSON_TYPE(scopes) = 'ARRAY' AND JSON_LENGTH(scopes) >= 1),
    CONSTRAINT chk_mcp_keys_send_validation CHECK (send_validation COLLATE utf8mb4_nopad_bin IN ('required', 'exempt')),
    CONSTRAINT uq_mcp_keys_key_hash UNIQUE (key_hash),
    INDEX idx_mcp_keys_company (company_id),
    INDEX idx_mcp_keys_created (created_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE mcp_server_settings (
    company_id BIGINT NOT NULL PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    address_filter_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    allowed_networks JSON NOT NULL,
    trusted_proxies JSON NOT NULL,
    allowed_origins JSON NOT NULL,
    version INT NOT NULL DEFAULT 1,
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_mcp_server_settings_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT chk_mcp_settings_networks CHECK (JSON_VALID(allowed_networks) AND JSON_TYPE(allowed_networks) = 'ARRAY'),
    CONSTRAINT chk_mcp_settings_proxies CHECK (JSON_VALID(trusted_proxies) AND JSON_TYPE(trusted_proxies) = 'ARRAY'),
    CONSTRAINT chk_mcp_settings_origins CHECK (JSON_VALID(allowed_origins) AND JSON_TYPE(allowed_origins) = 'ARRAY')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
