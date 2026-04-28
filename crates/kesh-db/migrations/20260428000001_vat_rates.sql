-- Migration Story 7.2 : table `vat_rates` — KF-003 closure (whitelist TVA → DB-driven).
-- Issue #3, décision rétro Epic 6 (Tech Debt Closure).
--
-- - `rate DECIMAL(5,2)` : aligné `products.vat_rate` (cohérence et JOIN futurs Epic 11).
-- - `label VARCHAR(64)` : clé i18n stockée en clair (ex. 'product-vat-normal'),
--   PAS texte traduit. Le frontend résout via `i18nMsg(label, fallback)`.
-- - `valid_from DATE` inclusif, `valid_to DATE` exclusif (NULL = pas d'expiration).
-- - **Pas de colonne `version`** v0.1 : la table est read-only (seul le seed écrit).
--   Epic 11-1 ajoutera `version INT` lors de l'introduction du CRUD admin.
-- - Backfill : 4 taux suisses 2024+ pour TOUTES les companies existantes via INSERT IGNORE
--   (pattern projet aligné `invoice_number_sequences.rs:19`).

CREATE TABLE vat_rates (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    label VARCHAR(64) NOT NULL,
    rate DECIMAL(5,2) NOT NULL,
    valid_from DATE NOT NULL,
    valid_to DATE NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_vat_rates_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_vat_rates_company_rate_valid_from UNIQUE (company_id, rate, valid_from),
    CONSTRAINT chk_vat_rates_rate_range CHECK (rate >= 0 AND rate <= 100),
    CONSTRAINT chk_vat_rates_label_not_empty CHECK (CHAR_LENGTH(TRIM(label)) > 0),
    CONSTRAINT chk_vat_rates_dates CHECK (valid_to IS NULL OR valid_to > valid_from),
    INDEX idx_vat_rates_company_active (company_id, active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Backfill : 4 taux suisses 2024+ par company existante.
-- INSERT IGNORE pour cohérence avec le helper de seed (idempotence pure) et
-- ré-exécutions multiples (cas dev local — sqlx skip via _sqlx_migrations
-- mais la sécurité du SQL prime).
INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to)
SELECT id, 'product-vat-normal',  8.10, '2024-01-01', NULL FROM companies;

INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to)
SELECT id, 'product-vat-special', 3.80, '2024-01-01', NULL FROM companies;

INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to)
SELECT id, 'product-vat-reduced', 2.60, '2024-01-01', NULL FROM companies;

INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to)
SELECT id, 'product-vat-exempt',  0.00, '2024-01-01', NULL FROM companies;
