-- Story v0172-session (0.17.3, #876, CR #891) — réglages de l'installation
-- faits dans l'interface : la durée d'inactivité de la session, en minutes,
-- réglée par l'Administrateur dans Administration → Paramètres (section
-- Session). Elle remplace l'ancienne variable d'environnement de durée
-- d'inactivité, qui n'est plus lue.
--
-- Table SYSTÈME, comme `_kesh_version` et `_kesh_backup_schedule` : hors de
-- `TABLES_TO_TRUNCATE`, donc jamais exportée dans un `.keshbackup`, jamais
-- restaurée par un import, jamais vidée par la réinitialisation de
-- démonstration. Une table applicative neuve ferait refuser à l'import TOUTE
-- sauvegarde antérieure (le contrôle de couverture exige l'égalité des
-- inventaires de tables) ; et ce réglage appartient à l'installation, non aux
-- livres. Une restauration conserve donc le réglage de la destination.
--
-- Singleton (`CHECK (id = 1)`), SANS ligne posée : tant qu'aucune n'est
-- enregistrée, le dépôt `installation_settings` rend les défauts du code
-- (60 minutes, version 0) — précédent `mcp_server_settings`. La migration est
-- donc du DDL pur.
--
-- `session_idle_minutes` : de 1 minute à 1440 (24 heures) — les bornes de
-- l'ancienne variable, si bien qu'aucune valeur posée hier n'en sort.
-- `version` : verrou optimiste du réglage (PUT de l'écran).
--
-- Politique (CLAUDE.md § Migration breaking policy) : NON BREAKING —
-- `CREATE TABLE` d'une entité neuve : pas de bump de
-- `kesh_version_min_required`. Comme toute migration, une 0.17.2 (ou 0.17.1)
-- refuse néanmoins de démarrer sur une base qui la porte (sqlx : migration
-- appliquée inconnue) ; le retour à une version antérieure passe par la
-- sauvegarde d'avant la mise à jour, ou par le retrait de la table ET de la
-- ligne de `_sqlx_migrations`.
--
-- Idempotence : tracked-by-sqlx (docs/migrations-idempotence-audit.md).
-- Triage P7 : aucune écriture de données (DDL seul), rien à trier.

CREATE TABLE _kesh_installation_settings (
    id TINYINT UNSIGNED NOT NULL PRIMARY KEY DEFAULT 1,
    session_idle_minutes INT NOT NULL DEFAULT 60,
    version INT NOT NULL DEFAULT 1,
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT chk_kesh_installation_settings_single_row CHECK (id = 1),
    CONSTRAINT chk_kesh_installation_settings_idle CHECK (session_idle_minutes BETWEEN 1 AND 1440)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
