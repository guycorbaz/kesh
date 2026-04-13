-- Migration Story 4.2 : catalogue produits/services.
-- FR29 (catalogue) + fondation FR30 (pré-remplissage facture, câblé Story 5.1).
--
-- - `unit_price DECIMAL(19,4)` : cohérent avec journal_entry_lines.debit/credit
--   (évite toute perte de précision lors de la copie catalogue → lignes facture
--   → journal_entries en Epic 5).
-- - `vat_rate DECIMAL(5,2)` : pourcentage direct (ex: 8.10, pas 0.081).
-- - Collation `utf8mb4_unicode_ci` (case-insensitive) : deux produits "Logo"
--   et "logo" sont considérés duplicats (voulu). La clause ENGINE/CHARSET/COLLATE
--   est OBLIGATOIRE — MariaDB 11.x utilise `uca1400_ai_ci` sinon, divergent
--   des autres tables du projet.

CREATE TABLE products (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(1000) NULL,
    unit_price DECIMAL(19,4) NOT NULL,
    vat_rate DECIMAL(5,2) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_products_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_products_company_name UNIQUE (company_id, name),
    CONSTRAINT chk_products_name_not_empty CHECK (CHAR_LENGTH(TRIM(name)) > 0),
    CONSTRAINT chk_products_price_non_negative CHECK (unit_price >= 0),
    CONSTRAINT chk_products_price_upper_bound CHECK (unit_price <= 1000000000),
    CONSTRAINT chk_products_vat_rate_range CHECK (vat_rate >= 0 AND vat_rate <= 100),
    INDEX idx_products_company_active (company_id, active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
