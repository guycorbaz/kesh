-- Story 31-1c (#503) — réglage et état de la sauvegarde automatique faite par
-- Kesh lui-même (tâche de fond `kesh_api::auto_backup`).
--
-- Table SYSTÈME, comme `_kesh_version` : hors de `TABLES_TO_TRUNCATE`, donc
-- jamais exportée dans un `.keshbackup` ni restaurée par un import. Une table
-- applicative neuve ferait refuser à l'import TOUTE sauvegarde antérieure
-- (le contrôle de couverture exige l'égalité des inventaires de tables) ; et
-- ces réglages appartiennent à l'installation, comme le `.env`, non aux
-- livres. Une restauration conserve donc les réglages et l'état de la
-- destination.
--
-- Singleton (`CHECK (id = 1)`), ligne posée par l'INSERT final avec les
-- défauts : sauvegarde ACTIVÉE, toutes les 24 heures, 7 conservées.
--
-- `last_status` :
--   ok                 — sauvegarde écrite, toutes les pièces incluses ;
--   partial            — sauvegarde écrite, mais des pièces référencées n'ont
--                        pas pu y entrer (`filesNotIncluded` non vide) ;
--   failed             — échec d'écriture ou de relecture ;
--   insufficient_space — refus avant écriture (espace libre sous le seuil) ;
--   disk_full          — disque ou quota saturé pendant l'écriture.
-- `last_complete_file` : la plus récente sauvegarde automatique COMPLÈTE
-- (`ok`), que la rétention ne supprime jamais.
-- `version` : verrou optimiste du seul réglage (PUT de l'écran) ; les
-- écritures d'état de la tâche de fond ne la touchent pas.
--
-- Politique (CLAUDE.md § Migration breaking policy) : NON BREAKING —
-- `CREATE TABLE` d'une entité neuve et sa ligne singleton : pas de bump de
-- `kesh_version_min_required`. Comme toute migration, une 0.14.0 refuse
-- néanmoins de démarrer sur une base qui la porte (sqlx : migration appliquée
-- inconnue) ; le retour à la 0.14.0 passe par la sauvegarde d'avant la mise à
-- jour.
--
-- Idempotence : tracked-by-sqlx (docs/migrations-idempotence-audit.md).
-- Triage P7 : l'INSERT du singleton est exempté (post_restore.rs,
-- EXEMPT_MIGRATIONS) — table système, jamais restaurée.

CREATE TABLE _kesh_backup_schedule (
    id TINYINT UNSIGNED NOT NULL PRIMARY KEY DEFAULT 1,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    interval_hours INT NOT NULL DEFAULT 24,
    retention_count INT NOT NULL DEFAULT 7,
    last_attempt_at DATETIME(3) NULL,
    last_success_at DATETIME(3) NULL,
    last_success_file VARCHAR(255) NULL,
    last_success_bytes BIGINT NULL,
    last_status VARCHAR(32) NULL,
    last_files_not_included INT NULL,
    last_complete_file VARCHAR(255) NULL,
    version INT NOT NULL DEFAULT 1,
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT chk_kesh_backup_schedule_single_row CHECK (id = 1),
    CONSTRAINT chk_kesh_backup_schedule_interval CHECK (interval_hours IN (6, 12, 24, 168)),
    CONSTRAINT chk_kesh_backup_schedule_retention CHECK (retention_count BETWEEN 1 AND 60),
    CONSTRAINT chk_kesh_backup_schedule_status CHECK (
        last_status IN ('ok', 'partial', 'failed', 'insufficient_space', 'disk_full')
    )
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO _kesh_backup_schedule (id) VALUES (1);
