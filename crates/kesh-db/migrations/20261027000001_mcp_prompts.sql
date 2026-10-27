-- Story 32-10a-i (refs #723) — les prompts du serveur MCP et leur historique :
-- les tables `mcp_prompts` (l'identité d'un prompt, une ligne par prompt et
-- par société) et `mcp_prompt_versions` (une ligne par version, jamais
-- réécrite ni effacée).
--
-- Une version a une ORIGINE (`standard` : le texte du code, amorcé par Kesh ;
-- `admin` : écrite par un administrateur ; `agent` : proposée par une clé MCP
-- de portée `prompts:write`), un ÉTAT (`approved`, `pending`, `refused` ; seule
-- une version d'agent passe par `pending`) et une NATURE (`text`, ou
-- `withdrawal` : le prompt cesse d'être servi). La version EN VIGUEUR d'un
-- prompt est sa version approuvée de plus grand numéro ; aucun pointeur ne la
-- désigne.
--
-- Cette migration CRÉE les tables et n'écrit AUCUNE donnée (C-32-0-11) : les
-- prompts standard s'amorcent au démarrage et à la fin de chaque point
-- d'entrée qui crée une société (`kesh_db::repositories::mcp_prompts::
-- amorcer_standards`), de façon idempotente.
--
-- Tables APPLICATIVES : elles entrent à `TABLES_TO_TRUNCATE`
-- (crates/kesh-db/src/backup.rs), donc au `.keshbackup`, et la remise à zéro
-- les vide (puis la route ré-amorce les standards). Conséquence : l'inventaire
-- des tables change, si bien qu'une sauvegarde antérieure à cette migration ne
-- s'importe pas — la fenêtre d'importabilité se referme une fois de plus, sans
-- effet sur le rejeu (le registre `POST_RESTORE_BACKFILLS` est vide). L'export
-- global CSV ne les porte pas (`TABLES_HORS_EXPORT`,
-- crates/kesh-api/src/exports/global.rs).
--
-- Politique (CLAUDE.md § Migration breaking policy) : NON BREAKING —
-- `CREATE TABLE` d'entités neuves : pas de bump de
-- `kesh_version_min_required` ni de version Cargo. Comme toute migration,
-- l'image précédente refuse néanmoins de démarrer sur une base qui la porte
-- (sqlx : migration appliquée inconnue) ; le retour passe par la sauvegarde
-- d'avant la mise à jour.
--
-- Contraintes, sur le patron de `20261025000001_mcp_keys_and_server_settings`
-- et de `20261025000002_mcp_sends` : toute valeur d'énumération, et le nom
-- d'un prompt, se comparent en collation BINAIRE SANS REMPLISSAGE
-- (`utf8mb4_nopad_bin`) — la collation `utf8mb4_unicode_ci` des tables ferait
-- accepter `'Approved'`, `'approved '` ou `'Kesh-x'`. Les autres comparaisons
-- d'énumération passent par la collation de la table : elles sont sûres parce
-- que les `CHECK` d'énumération n'admettent que les valeurs exactes. Les
-- colonnes gardent la collation de la table (une collation binaire de COLONNE
-- les ferait lire comme `VARBINARY`) : toute comparaison de contenu se fait en
-- Rust, sur les octets. Le motif du nom s'ancre par `\z` et non par `$` :
-- éprouvé à la main sur le MariaDB de dev, `$` s'apparie avant un saut de
-- ligne final et laisserait passer `'valide\n'` (validation P1, R7). Le texte
-- d'une version porte un caractère qui n'est pas un blanc (`REGEXP`, car
-- `TRIM` ne retire que les espaces). `is_standard` et `nature` n'ont PAS de
-- `DEFAULT` : le dépôt les pose toujours, et une colonne sans défaut reste
-- requise à l'import.
--
-- Idempotence : tracked-by-sqlx (docs/migrations-idempotence-audit.md).
-- Triage P7 : aucune instruction n'écrit de données — ni registre ni
-- exemption.

CREATE TABLE mcp_prompts (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    name VARCHAR(64) NOT NULL COMMENT '[a-z0-9_-]{1,64} ; préfixe kesh- réservé aux standards',
    is_standard BOOLEAN NOT NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_mcp_prompts_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_mcp_prompts_company_name UNIQUE (company_id, name),
    CONSTRAINT chk_mcp_prompts_name CHECK (name COLLATE utf8mb4_nopad_bin REGEXP '^[a-z0-9_-]{1,64}\\z'),
    CONSTRAINT chk_mcp_prompts_reserved CHECK (
        (is_standard AND name COLLATE utf8mb4_nopad_bin LIKE 'kesh-%')
        OR (NOT is_standard AND name COLLATE utf8mb4_nopad_bin NOT LIKE 'kesh%'))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE mcp_prompt_versions (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    prompt_id BIGINT NOT NULL,
    number INT NOT NULL,
    origin VARCHAR(16) NOT NULL COMMENT 'standard | admin | agent',
    state VARCHAR(16) NOT NULL COMMENT 'approved | pending | refused',
    nature VARCHAR(16) NOT NULL COMMENT 'text | withdrawal',
    description VARCHAR(500) NULL,
    body TEXT NULL,
    author_user_id BIGINT NULL,
    author_mcp_key_id BIGINT NULL,
    decided_by_user_id BIGINT NULL,
    decided_at DATETIME(3) NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_mcp_prompt_versions_prompt FOREIGN KEY (prompt_id) REFERENCES mcp_prompts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_mcp_prompt_versions_author FOREIGN KEY (author_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT fk_mcp_prompt_versions_key FOREIGN KEY (author_mcp_key_id) REFERENCES mcp_keys(id) ON DELETE RESTRICT,
    CONSTRAINT fk_mcp_prompt_versions_decider FOREIGN KEY (decided_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT uq_mcp_prompt_versions_number UNIQUE (prompt_id, number),
    CONSTRAINT chk_mcp_prompt_versions_number CHECK (number >= 1),
    CONSTRAINT chk_mcp_prompt_versions_origin CHECK (origin COLLATE utf8mb4_nopad_bin IN ('standard', 'admin', 'agent')),
    CONSTRAINT chk_mcp_prompt_versions_state CHECK (state COLLATE utf8mb4_nopad_bin IN ('approved', 'pending', 'refused')),
    CONSTRAINT chk_mcp_prompt_versions_nature CHECK (nature COLLATE utf8mb4_nopad_bin IN ('text', 'withdrawal')),
    CONSTRAINT chk_mcp_prompt_versions_standard CHECK (origin <> 'standard' OR (
        state = 'approved' AND author_user_id IS NULL AND author_mcp_key_id IS NULL
        AND decided_by_user_id IS NULL AND decided_at IS NULL)),
    CONSTRAINT chk_mcp_prompt_versions_admin CHECK (origin <> 'admin' OR (
        state = 'approved' AND author_user_id IS NOT NULL AND author_mcp_key_id IS NULL
        AND decided_by_user_id IS NULL AND decided_at IS NULL)),
    CONSTRAINT chk_mcp_prompt_versions_agent CHECK (origin <> 'agent' OR (
        author_mcp_key_id IS NOT NULL AND author_user_id IS NULL AND nature = 'text'
        AND (state = 'pending') = (decided_by_user_id IS NULL)
        AND (decided_by_user_id IS NULL) = (decided_at IS NULL))),
    CONSTRAINT chk_mcp_prompt_versions_content CHECK (
        (nature = 'text' AND description IS NOT NULL AND body IS NOT NULL
            AND description REGEXP '[^[:space:]]' AND body REGEXP '[^[:space:]]')
        OR (nature = 'withdrawal' AND description IS NULL AND body IS NULL)),
    CONSTRAINT chk_mcp_prompt_versions_body_length CHECK (body IS NULL OR CHAR_LENGTH(body) <= 16000),
    INDEX idx_mcp_prompt_versions_state (prompt_id, state, number)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
