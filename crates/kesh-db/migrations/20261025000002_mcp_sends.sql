-- Story 32-8a (refs #723) — les envois du serveur MCP en attente de
-- validation : la table `mcp_sends`.
--
-- Une clé MCP dont la politique est `required` (Q12) ne transmet rien à son
-- agent sans l'accord d'un administrateur : ce qu'un outil s'apprête à envoyer
-- est DÉPOSÉ ici, sous la forme du texte même qui partira (caractères cachés
-- rendus visibles par la 32-8b), puis validé tel quel, validé modifié ou
-- refusé à l'écran ; l'agent relit l'envoi validé par son numéro (`id`).
--
-- États stockés : `pending`, `approved`, `refused`. L'état « échu » n'est
-- JAMAIS écrit : il se DÉRIVE à la lecture, à l'horloge de MariaDB, de
-- `expires_at` (échéance absolue posée au dépôt). Un envoi refusé n'a ni
-- contenu ni résumé ; `modified` n'est vrai que pour un envoi validé.
--
-- Table APPLICATIVE : elle entre à `TABLES_TO_TRUNCATE`
-- (crates/kesh-db/src/backup.rs), donc au `.keshbackup`, et la remise à zéro
-- la vide. Conséquence : l'inventaire des tables change, si bien qu'une
-- sauvegarde antérieure à cette migration ne s'importe pas (déjà vrai depuis
-- la 32-5a, Q4 de Guy). L'export global CSV ne la porte pas
-- (`TABLES_HORS_EXPORT`, crates/kesh-api/src/exports/global.rs).
--
-- Politique (CLAUDE.md § Migration breaking policy) : NON BREAKING —
-- `CREATE TABLE` d'une entité neuve : pas de bump de
-- `kesh_version_min_required` ni de version Cargo. Comme toute migration,
-- l'image précédente refuse néanmoins de démarrer sur une base qui la porte
-- (sqlx : migration appliquée inconnue) ; le retour passe par la sauvegarde
-- d'avant la mise à jour.
--
-- Contraintes, sur le patron de `20261025000001_mcp_keys_and_server_settings`
-- : le `CHECK` de `state` compare en collation BINAIRE SANS REMPLISSAGE
-- (`utf8mb4_nopad_bin`), car la collation `utf8mb4_unicode_ci` de la table
-- ferait accepter `'Pending'` ou `'pending '`. Les deux autres comparaisons de
-- `state` passent par la collation de la table : elles sont sûres parce que
-- `chk_mcp_sends_state` n'admet que les trois valeurs exactes. `content` garde
-- la collation de la table (une collation binaire de COLONNE la ferait lire
-- comme `VARBINARY`) : toute comparaison d'égalité du contenu se fait en Rust,
-- sur les octets. `state` et `modified` n'ont PAS de `DEFAULT` : le dépôt les
-- pose toujours, et une colonne sans défaut reste requise à l'import.
--
-- Idempotence : tracked-by-sqlx (docs/migrations-idempotence-audit.md).
-- Triage P7 : aucune instruction n'écrit de données — ni registre ni
-- exemption.

CREATE TABLE mcp_sends (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    mcp_key_id BIGINT NOT NULL,
    tool_name VARCHAR(64) NOT NULL,
    state VARCHAR(16) NOT NULL COMMENT 'pending | approved | refused — « échu » est dérivé, jamais écrit',
    summary TEXT NULL COMMENT 'Résumé de la demande, montré à l''écran, jamais envoyé',
    content MEDIUMTEXT NULL COMMENT 'Texte rendu visible ; remplacé à la validation modifiée ; NULL si refusé ou purgé',
    modified BOOLEAN NOT NULL,
    expires_at DATETIME(3) NOT NULL,
    decided_at DATETIME(3) NULL,
    decided_by_user_id BIGINT NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_mcp_sends_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_mcp_sends_key FOREIGN KEY (mcp_key_id) REFERENCES mcp_keys(id) ON DELETE RESTRICT,
    CONSTRAINT fk_mcp_sends_decided_by FOREIGN KEY (decided_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT chk_mcp_sends_state CHECK (state COLLATE utf8mb4_nopad_bin IN ('pending', 'approved', 'refused')),
    CONSTRAINT chk_mcp_sends_decision CHECK ((decided_at IS NULL) = (decided_by_user_id IS NULL)
        AND (state = 'pending') = (decided_at IS NULL)),
    CONSTRAINT chk_mcp_sends_refused_empty CHECK (state <> 'refused' OR (content IS NULL AND summary IS NULL)),
    CONSTRAINT chk_mcp_sends_modified CHECK (state = 'approved' OR modified = FALSE),
    CONSTRAINT chk_mcp_sends_tool CHECK (CHAR_LENGTH(tool_name) BETWEEN 1 AND 64),
    INDEX idx_mcp_sends_key (mcp_key_id, state, expires_at),
    INDEX idx_mcp_sends_company (company_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
