-- Migration Story 12.1 : Avoirs (notes de crédit) — Epic 12 Avoirs & Paiements.
-- FR36 (annuler une facture validée uniquement par création d'un avoir) +
-- FR37 (séquence de numérotation séparée des avoirs).
--
-- Tables miroir des factures (cf. 20260416000001_invoices.sql) :
--   - credit_notes / credit_note_lines : structure identique à invoices/invoice_lines.
--   - credit_note_number_sequences : séquence no-gap dédiée (miroir
--     invoice_number_sequences, scope (company_id, fiscal_year_id)).
--   - ALTER company_invoice_settings : format de numéro d'avoir configurable.
--
-- Comptabilité : l'avoir est émis en single-step (DC5) ; à l'émission, une
-- écriture de CONTRE-PASSATION (swap débit↔crédit, montants positifs) est
-- générée et liée via credit_notes.journal_entry_id, et la facture d'origine
-- passe au statut 'cancelled' (DC6).
--
-- Conventions (cohérentes invoices) :
--   - `status` géré par CHECK texte (pas d'enum SQLx — cf. feedback_sqlx_mysql_gotchas).
--     INSERT pose toujours 'issued' (DC5) ; 'draft'/'cancelled' réservés compat-forward.
--   - `total_amount` stocké = HT (Σ line_total), miroir strict de invoices.total_amount.
--   - FK invoice_id ON DELETE RESTRICT : une facture créditée ne peut être supprimée.
--   - UNIQUE(invoice_id) : un seul avoir total par facture (DC3/DC7).
--   - ENGINE/CHARSET/COLLATE obligatoires (MariaDB 11.x utilise uca1400_ai_ci sinon).
--
-- Non-breaking (CREATE TABLE + ADD COLUMN … DEFAULT, ignorés par les anciens
-- binaires) → PAS de bump kesh_version_min_required (P1/P2).

CREATE TABLE credit_notes (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    contact_id BIGINT NOT NULL,
    invoice_id BIGINT NOT NULL,
    credit_note_number VARCHAR(64) NULL,
    status VARCHAR(16) NOT NULL,
    date DATE NOT NULL,
    total_amount DECIMAL(19,4) NOT NULL DEFAULT 0,
    journal_entry_id BIGINT NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_credit_notes_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_credit_notes_contact
        FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_credit_notes_invoice
        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE RESTRICT,
    CONSTRAINT fk_credit_notes_journal_entry
        FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT,
    CONSTRAINT chk_credit_notes_status
        CHECK (status IN ('draft', 'issued', 'cancelled')),
    CONSTRAINT chk_credit_notes_total_non_negative CHECK (total_amount >= 0),
    -- Un avoir 'issued' a forcément un numéro ET une écriture de contre-passation.
    CONSTRAINT chk_credit_notes_issued_has_je
        CHECK (status <> 'issued' OR (credit_note_number IS NOT NULL AND journal_entry_id IS NOT NULL)),
    CONSTRAINT uq_credit_notes_number UNIQUE (company_id, credit_note_number),
    -- Un seul avoir total par facture (DC3/DC7).
    CONSTRAINT uq_credit_notes_invoice UNIQUE (invoice_id),
    INDEX idx_credit_notes_company_status (company_id, status),
    INDEX idx_credit_notes_company_date (company_id, date),
    INDEX idx_credit_notes_contact (contact_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE credit_note_lines (
    id BIGINT NOT NULL AUTO_INCREMENT,
    credit_note_id BIGINT NOT NULL,
    position INT NOT NULL,
    description VARCHAR(1000) NOT NULL,
    quantity DECIMAL(19,4) NOT NULL,
    unit_price DECIMAL(19,4) NOT NULL,
    vat_rate DECIMAL(5,2) NOT NULL,
    line_total DECIMAL(19,4) NOT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_credit_note_lines_credit_note
        FOREIGN KEY (credit_note_id) REFERENCES credit_notes(id) ON DELETE CASCADE,
    CONSTRAINT chk_credit_note_lines_quantity_positive CHECK (quantity > 0),
    CONSTRAINT chk_credit_note_lines_unit_price_non_negative CHECK (unit_price >= 0),
    CONSTRAINT chk_credit_note_lines_vat_rate_range CHECK (vat_rate >= 0 AND vat_rate <= 100),
    CONSTRAINT chk_credit_note_lines_description_not_empty
        CHECK (CHAR_LENGTH(TRIM(description)) > 0),
    CONSTRAINT chk_credit_note_lines_line_total_non_negative CHECK (line_total >= 0),
    CONSTRAINT uq_credit_note_lines_position UNIQUE (credit_note_id, position),
    INDEX idx_credit_note_lines_credit_note (credit_note_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE credit_note_number_sequences (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    fiscal_year_id BIGINT NOT NULL,
    next_number BIGINT NOT NULL DEFAULT 1,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_cns_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_cns_fiscal_year FOREIGN KEY (fiscal_year_id) REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    CONSTRAINT uq_cns_company_fy UNIQUE (company_id, fiscal_year_id),
    CONSTRAINT chk_cns_next_positive CHECK (next_number >= 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Format de numéro d'avoir, configurable par société (défaut AV-2026-0001).
ALTER TABLE company_invoice_settings
    ADD COLUMN credit_note_number_format VARCHAR(64) NOT NULL DEFAULT 'AV-{YEAR}-{SEQ:04}',
    ADD CONSTRAINT chk_cis_cn_format_nonempty CHECK (CHAR_LENGTH(TRIM(credit_note_number_format)) > 0);
