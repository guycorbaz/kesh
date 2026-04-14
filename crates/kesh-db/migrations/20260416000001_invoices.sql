-- Migration Story 5.1 : factures brouillon + lignes.
-- FR31 (lignes libres + catalogue) + FR32 (CRUD brouillon).
--
-- - `invoice_number VARCHAR(64) NULL` : reste NULL en brouillon.
--   Story 5.2 ajoutera la contrainte d'unicité sur validation.
-- - `status` géré par CHECK texte (pas d'enum SQLx — cf. feedback_sqlx_mysql_gotchas).
-- - `total_amount` stocké (source de vérité = lignes, recalcul backend).
-- - FK `contact_id ON DELETE RESTRICT` : la suppression d'un contact
--   ayant des factures renvoie désormais une FK violation mappée en 409
--   côté handler contacts (régression assumée Story 4.1).
-- - FK `invoice_lines.invoice_id ON DELETE CASCADE` : hard-delete brouillon
--   supprime les lignes atomiquement.
-- - ENGINE/CHARSET/COLLATE obligatoires (MariaDB 11.x utilise uca1400_ai_ci sinon).

CREATE TABLE invoices (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    contact_id BIGINT NOT NULL,
    invoice_number VARCHAR(64) NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'draft',
    date DATE NOT NULL,
    due_date DATE NULL,
    payment_terms VARCHAR(255) NULL,
    total_amount DECIMAL(19,4) NOT NULL DEFAULT 0,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_invoices_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_invoices_contact
        FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_invoices_status
        CHECK (status IN ('draft', 'validated', 'cancelled')),
    CONSTRAINT chk_invoices_total_non_negative CHECK (total_amount >= 0),
    INDEX idx_invoices_company_status (company_id, status),
    INDEX idx_invoices_company_date (company_id, date),
    INDEX idx_invoices_contact (contact_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE invoice_lines (
    id BIGINT NOT NULL AUTO_INCREMENT,
    invoice_id BIGINT NOT NULL,
    position INT NOT NULL,
    description VARCHAR(1000) NOT NULL,
    quantity DECIMAL(19,4) NOT NULL,
    unit_price DECIMAL(19,4) NOT NULL,
    vat_rate DECIMAL(5,2) NOT NULL,
    line_total DECIMAL(19,4) NOT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_invoice_lines_invoice
        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
    CONSTRAINT chk_invoice_lines_quantity_positive CHECK (quantity > 0),
    CONSTRAINT chk_invoice_lines_unit_price_non_negative CHECK (unit_price >= 0),
    CONSTRAINT chk_invoice_lines_vat_rate_range CHECK (vat_rate >= 0 AND vat_rate <= 100),
    CONSTRAINT chk_invoice_lines_description_not_empty
        CHECK (CHAR_LENGTH(TRIM(description)) > 0),
    CONSTRAINT uq_invoice_lines_position UNIQUE (invoice_id, position),
    INDEX idx_invoice_lines_invoice (invoice_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
