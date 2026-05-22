-- Story 10-2 — Migration table _kesh_version pour le tracking de version
-- DB et la downgrade protection au boot.
--
-- Référence : _bmad-output/implementation-artifacts/10-2-migrations-idempotence-downgrade-protection.md
-- AC #4-7 : schéma + INSERT row initiale + verdict idempotence.
-- AC #11-13 : la table est lue par check_downgrade_protection avant
--             MIGRATOR.run() et écrite par record_boot_version après.
--
-- Singleton-row pattern via CHECK (id = 1) — enforce MariaDB ≥ 10.2.
-- VARCHAR(40) couvre SemVer 2.0 incluant pre-release + build metadata
-- (e.g. `1.0.0-rc1.20260522+build.20260522.001` = 37 chars). Évite la
-- troncature silencieuse en mode non-strict ou l'échec d'UPDATE en mode
-- strict si une release future utilise un format complet.
--
-- INSERT row initiale figée à '0.1.0' : version Kesh courante au moment
-- de la création de cette migration. Cette valeur devient **historique**
-- dès le premier boot : `record_boot_version()` écrase
-- `kesh_version_last_applied` avec `CARGO_PKG_VERSION` au prochain
-- démarrage, et `kesh_version_min_required` n'est mis à jour qu'à
-- l'introduction d'une migration breaking (cf. politique P2 CLAUDE.md
-- `## Migration breaking policy`). La valeur '0.1.0' ne suit donc PAS
-- la version Kesh courante — refactor futur ne doit pas tenter de
-- « synchroniser » ce DEFAULT avec Cargo.toml.
--
-- Décision idempotence (cf. docs/migrations-idempotence-audit.md) :
-- tracked-by-sqlx — pas de IF NOT EXISTS ni INSERT IGNORE, conforme
-- à la convention historique majoritaire (15 autres CREATE TABLE
-- non-guarded). Re-exécution hors sqlx échouerait erreur 1050 (CREATE)
-- puis 1062 (INSERT) — intentionnel.

CREATE TABLE _kesh_version (
    id TINYINT UNSIGNED NOT NULL PRIMARY KEY DEFAULT 1,
    kesh_version_min_required VARCHAR(40) NOT NULL,
    kesh_version_last_applied VARCHAR(40) NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_boot_at DATETIME NULL,
    CONSTRAINT chk_kesh_version_single_row CHECK (id = 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO _kesh_version (id, kesh_version_min_required, kesh_version_last_applied)
    VALUES (1, '0.1.0', '0.1.0');
