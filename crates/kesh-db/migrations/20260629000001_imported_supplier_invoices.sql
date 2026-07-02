-- Migration Story 12.5b : Import de factures fournisseurs depuis un répertoire (#194).
--
-- Table de STAGING `imported_supplier_invoices` : factures fournisseurs importées
-- depuis un dossier inbox (PDF/image porteurs d'un Swiss QR Code), décodées côté
-- serveur, en attente de complétion par l'utilisateur (compte/TVA/lignes).
--
-- Modèle (DC3 staging « à compléter », DC4 lien justificatif côté import) :
--   - Coordonnées QR parsées (créancier, IBAN/QR-IBAN, référence, montant, devise).
--   - Lien fichier archivé sur disque (KESH_DOCUMENTS_DIR, hors DB) : storage_path,
--     original_filename, sha256, mime_type, byte_size.
--   - `supplier_invoice_id` NULLABLE → renseigné à la complétion (12-5c) quand la
--     facture fournisseur réelle (12-2) est créée. AUCUN ALTER de supplier_invoices.
--   - `status` to_complete → completed (à la complétion) | discarded (écartée).
--
-- Idempotence : UNIQUE (company_id, file_hash) — un même fichier (hash SHA-256) ne
--   peut être importé deux fois dans la MÊME company ; deux companies distinctes
--   peuvent importer le même fichier (F-NEW-3).
--
-- Conventions (cohérentes supplier_invoices 12-2) :
--   - `status`/`creditor_address_type`/`reference_type` gérés par CHECK texte
--     (pas d'enum SQLx — cf. feedback_sqlx_mysql_gotchas).
--   - multi-tenant `company_id` FK RESTRICT (défense IDOR, KF-002).
--   - ENGINE/CHARSET/COLLATE obligatoires (MariaDB 11.x utilise uca1400_ai_ci sinon).
--
-- Non-breaking (CREATE TABLE, ignorée par les anciens binaires) → PAS de bump
-- kesh_version_min_required (P1/P3).

CREATE TABLE imported_supplier_invoices (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'to_complete',
    -- Renseigné à la complétion (12-5c) : lien vers la facture fournisseur réelle (12-2).
    supplier_invoice_id BIGINT NULL,
    -- Fichier archivé (KESH_DOCUMENTS_DIR, hors DB).
    file_hash CHAR(64) NOT NULL,
    storage_path VARCHAR(512) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    byte_size BIGINT NOT NULL,
    -- Coordonnées créancier parsées du QR (SPC).
    creditor_iban VARCHAR(34) NOT NULL,
    is_qr_iban BOOLEAN NOT NULL,
    creditor_address_type CHAR(1) NOT NULL,
    creditor_name VARCHAR(70) NOT NULL,
    creditor_line1 VARCHAR(70) NULL,
    creditor_line2 VARCHAR(70) NULL,
    creditor_postal_code VARCHAR(16) NULL,
    creditor_town VARCHAR(35) NULL,
    creditor_country CHAR(2) NOT NULL,
    -- Référence de paiement.
    reference_type VARCHAR(8) NOT NULL,
    reference_value VARCHAR(40) NULL,
    -- Montant / devise (SPC autorise montant vide → open amount).
    amount DECIMAL(19,4) NULL,
    currency VARCHAR(3) NOT NULL,
    unstructured_message VARCHAR(140) NULL,
    billing_information VARCHAR(140) NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_imported_si_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_imported_si_supplier_invoice
        FOREIGN KEY (supplier_invoice_id) REFERENCES supplier_invoices(id) ON DELETE SET NULL,
    CONSTRAINT chk_imported_si_status
        CHECK (status IN ('to_complete', 'completed', 'discarded')),
    CONSTRAINT chk_imported_si_address_type
        CHECK (creditor_address_type IN ('K', 'S')),
    CONSTRAINT chk_imported_si_reference_type
        CHECK (reference_type IN ('QRR', 'SCOR', 'NON')),
    CONSTRAINT chk_imported_si_byte_size_positive CHECK (byte_size > 0),
    CONSTRAINT uq_imported_company_hash UNIQUE (company_id, file_hash),
    INDEX idx_imported_company_status (company_id, status),
    INDEX idx_imported_supplier_invoice (supplier_invoice_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
