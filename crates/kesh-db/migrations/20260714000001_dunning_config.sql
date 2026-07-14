-- Migration Story 21-3 — socle de configuration des rappels débiteurs (dunning).
-- Issue #231 (Epic 21 « Échéances & relances débiteurs »).
--
-- Deux tables, toutes deux NON-BREAKING (CREATE TABLE — un binaire antérieur les
-- ignore) → PAS de bump `kesh_version_min_required` (cf. politique P2/P3 CLAUDE.md ;
-- le bump breaking est dans la migration `..._email_templates_reminder.sql`).
--
-- `dunning_levels` : collection company-scoped des niveaux de rappel, calquée sur
--   `vat_rates` (sentinel lock + verrou optimiste `version` côté repo). Niveaux
--   NUMÉROTÉS CONTIGUS (`level_number` unique 1-based) — hard-delete + renumérotation
--   côté repo (l'historique est protégé par les snapshots `invoice_reminders`, 21-5a).
--   `fee_amount DECIMAL(7,2)` borné 0..10'000 (bloque frais négatif et fat-finger, D5).
--
-- `company_dunning_settings` : singleton de config, calqué `company_invoice_settings`
--   (PK = `company_id`, get-or-create via INSERT IGNORE, verrou optimiste). `seeded_at`
--   discrimine « jamais seedé » (NULL → seed lazy) de « vidé volontairement » (NON-NULL
--   → dunning désactivé, PAS de résurrection des défauts) — sémantique D7.
--
-- PAS de backfill INSERT ici : le seed des 3 niveaux par défaut est LAZY (au 1er accès
-- config ou 1re évaluation d'éligibilité), sous sentinel lock, pour pouvoir poser
-- `seeded_at` et rester idempotent/annulable — ce qu'un backfill de migration ne permet pas.
--
-- Idempotence (docs/migrations-idempotence-audit.md) : tracked-by-sqlx — pas de
-- IF NOT EXISTS ni INSERT IGNORE (convention majoritaire) ; re-exécution hors sqlx
-- échouerait erreur 1050. Non-breaking (nouvelles tables) → pas de bump.

CREATE TABLE dunning_levels (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    level_number SMALLINT NOT NULL,
    delay_days INT NOT NULL,
    fee_amount DECIMAL(7,2) NOT NULL,
    version INT NOT NULL DEFAULT 0,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_dunning_levels_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_dunning_levels_company_level UNIQUE (company_id, level_number),
    CONSTRAINT chk_dunning_levels_fee_range CHECK (fee_amount >= 0 AND fee_amount <= 10000),
    CONSTRAINT chk_dunning_levels_delay_nonneg CHECK (delay_days >= 0),
    CONSTRAINT chk_dunning_levels_level_positive CHECK (level_number >= 1),
    INDEX idx_dunning_levels_company (company_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE company_dunning_settings (
    company_id BIGINT NOT NULL PRIMARY KEY,
    grace_period_days INT NOT NULL DEFAULT 5,
    seeded_at DATETIME(3) NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_cds_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT chk_cds_grace_nonneg CHECK (grace_period_days >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
