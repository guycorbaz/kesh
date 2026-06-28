-- Migration Story 12.2 : Factures fournisseurs & règlement binaire — Epic 12 Paiements (#191).
--
-- Nouveau sous-système (entité DÉDIÉE, distincte de `invoices` ventes) :
--   - supplier_invoices / supplier_invoice_lines : facture reçue d'un créancier.
--   - ALTER company_invoice_settings : compte créanciers (2000) par défaut + backfill.
--
-- Modèle métier (RÈGLEMENT BINAIRE, cf. spec 12-2) :
--   - Enregistrer pose l'écriture d'achat (D charge / D impôt préalable 1171 /
--     C créanciers 2000) en single-step et passe la facture en 'open'.
--   - Payer pose l'écriture de règlement (D 2000 / C contrepartie) et passe en
--     'paid'. Contrepartie = compte bancaire source (virement) OU compte libre
--     du plan comptable (compte interne). PAS de table de modes.
--   - Annuler une facture 'open' contre-passe l'écriture d'achat → 'cancelled'.
--
-- Conventions (cohérentes invoices/credit_notes) :
--   - `status` géré par CHECK texte (pas d'enum SQLx — cf. feedback_sqlx_mysql_gotchas).
--   - `total_amount` stocké = TTC (Σ HT + Σ TVA) = la ligne C 2000 de l'écriture
--     d'achat → le règlement débite EXACTEMENT ce montant et solde 2000 à 0 [O-M2].
--   - Coordonnées de paiement (IBAN/QR-IBAN/référence/montant attendu) = données
--     de la facture, ORTHOGONALES au mode (consommées par le virement pain.001, 12-3).
--   - FK ON DELETE RESTRICT partout (cohérence schéma).
--   - ENGINE/CHARSET/COLLATE obligatoires (MariaDB 11.x utilise uca1400_ai_ci sinon).
--
-- Non-breaking (CREATE TABLE + ADD COLUMN nullable, ignorés par les anciens
-- binaires) → PAS de bump kesh_version_min_required (P1/P3).

CREATE TABLE supplier_invoices (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    contact_id BIGINT NOT NULL,
    supplier_invoice_number VARCHAR(64) NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'open',
    invoice_date DATE NOT NULL,
    due_date DATE NULL,
    total_amount DECIMAL(19,4) NOT NULL DEFAULT 0,
    -- Coordonnées de paiement (orthogonales au mode — données de la facture).
    creditor_iban VARCHAR(34) NULL,
    creditor_qr_iban VARCHAR(34) NULL,
    payment_reference VARCHAR(64) NULL,
    expected_payment_amount DECIMAL(19,4) NULL,
    -- Écriture d'achat (posée à l'enregistrement, single-step).
    purchase_journal_entry_id BIGINT NOT NULL,
    -- Règlement (renseigné au paiement uniquement).
    settlement_type VARCHAR(20) NULL,
    settlement_bank_account_id BIGINT NULL,
    settlement_account_id BIGINT NULL,
    settlement_journal_entry_id BIGINT NULL,
    paid_at DATETIME(3) NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_supplier_invoices_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_supplier_invoices_contact
        FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_supplier_invoices_purchase_je
        FOREIGN KEY (purchase_journal_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT,
    CONSTRAINT fk_supplier_invoices_settlement_bank
        FOREIGN KEY (settlement_bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_supplier_invoices_settlement_account
        FOREIGN KEY (settlement_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_supplier_invoices_settlement_je
        FOREIGN KEY (settlement_journal_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT,
    CONSTRAINT chk_supplier_invoices_status
        CHECK (status IN ('open', 'paid', 'cancelled')),
    CONSTRAINT chk_supplier_invoices_settlement_type
        CHECK (settlement_type IS NULL OR settlement_type IN ('bank_transfer', 'internal_account')),
    CONSTRAINT chk_supplier_invoices_total_non_negative CHECK (total_amount >= 0),
    -- Une facture 'paid' a forcément une écriture de règlement, un horodatage et un type.
    CONSTRAINT chk_supplier_invoices_paid_has_settlement
        CHECK (status <> 'paid' OR (settlement_journal_entry_id IS NOT NULL
            AND paid_at IS NOT NULL AND settlement_type IS NOT NULL)),
    INDEX idx_supplier_invoices_company_status (company_id, status),
    INDEX idx_supplier_invoices_company_date (company_id, invoice_date),
    INDEX idx_supplier_invoices_contact (contact_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE supplier_invoice_lines (
    id BIGINT NOT NULL AUTO_INCREMENT,
    supplier_invoice_id BIGINT NOT NULL,
    position INT NOT NULL,
    description VARCHAR(1000) NOT NULL,
    quantity DECIMAL(19,4) NOT NULL,
    unit_price DECIMAL(19,4) NOT NULL,
    vat_rate DECIMAL(5,2) NOT NULL,
    line_total DECIMAL(19,4) NOT NULL,
    -- Compte de charge (6xxx) de la ligne.
    expense_account_id BIGINT NOT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_supplier_invoice_lines_invoice
        FOREIGN KEY (supplier_invoice_id) REFERENCES supplier_invoices(id) ON DELETE CASCADE,
    CONSTRAINT fk_supplier_invoice_lines_expense_account
        FOREIGN KEY (expense_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_supplier_invoice_lines_quantity_positive CHECK (quantity > 0),
    CONSTRAINT chk_supplier_invoice_lines_unit_price_positive CHECK (unit_price > 0),
    CONSTRAINT chk_supplier_invoice_lines_vat_rate_range CHECK (vat_rate >= 0 AND vat_rate <= 100),
    CONSTRAINT chk_supplier_invoice_lines_line_total_positive CHECK (line_total > 0),
    CONSTRAINT chk_supplier_invoice_lines_description_not_empty
        CHECK (CHAR_LENGTH(TRIM(description)) > 0),
    CONSTRAINT uq_supplier_invoice_lines_position UNIQUE (supplier_invoice_id, position),
    INDEX idx_supplier_invoice_lines_invoice (supplier_invoice_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Compte créanciers (2000) par défaut, contrepartie de l'écriture d'achat (Liability).
-- ADD COLUMN nullable + FK RESTRICT (le compte 2000 existe déjà dans les charts d'origine).
ALTER TABLE company_invoice_settings
    ADD COLUMN default_payable_account_id BIGINT NULL,
    ADD CONSTRAINT fk_cis_payable_account
        FOREIGN KEY (default_payable_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;

-- Backfill : lier au compte 2000 « Créanciers » existant pour les companies déjà
-- onboardées (PAS d'INSERT — le compte existe). Idempotent (WHERE … IS NULL).
UPDATE company_invoice_settings cis
    INNER JOIN accounts a
        ON a.company_id = cis.company_id AND a.number = '2000'
    SET cis.default_payable_account_id = a.id
    WHERE cis.default_payable_account_id IS NULL;
