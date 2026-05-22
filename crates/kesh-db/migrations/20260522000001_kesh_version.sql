-- Story 10-2 — Migration table _kesh_version pour le tracking de version
-- DB et la downgrade protection au boot.
--
-- Référence : _bmad-output/implementation-artifacts/10-2-migrations-idempotence-downgrade-protection.md
-- AC #4-7 : schéma + INSERT row initiale + verdict idempotence.
-- AC #11-13 : la table est lue par check_downgrade_protection avant
--             MIGRATOR.run() et écrite par record_boot_version après.
--
-- Singleton-row pattern via CHECK (id = 1) — enforce MariaDB ≥ 10.2.
-- VARCHAR(20) couvre largement SemVer (`major.minor.patch[-pre][+build]`).
-- DEFAULT '0.1.0' figé : version Kesh courante à la création de cette
-- migration. Sera écrasé par record_boot_version() au prochain boot.
--
-- Décision idempotence (cf. docs/migrations-idempotence-audit.md) :
-- tracked-by-sqlx — pas de IF NOT EXISTS ni INSERT IGNORE, conforme
-- à la convention historique majoritaire (15 autres CREATE TABLE
-- non-guarded). Re-exécution hors sqlx échouerait erreur 1050 (CREATE)
-- puis 1062 (INSERT) — intentionnel.

CREATE TABLE _kesh_version (
    id TINYINT UNSIGNED NOT NULL PRIMARY KEY DEFAULT 1,
    kesh_version_min_required VARCHAR(20) NOT NULL,
    kesh_version_last_applied VARCHAR(20) NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_boot_at DATETIME NULL,
    CONSTRAINT chk_kesh_version_single_row CHECK (id = 1)
);

INSERT INTO _kesh_version (id, kesh_version_min_required, kesh_version_last_applied)
    VALUES (1, '0.1.0', '0.1.0');
