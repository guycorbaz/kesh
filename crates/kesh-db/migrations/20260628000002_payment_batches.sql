-- Migration Story 12.3 : Lots de paiement pain.001 (mode virement) — Epic 12 Paiements (#191).
--
-- Flux deux temps : un lot regroupe N factures fournisseurs à régler par virement.
--   1. Génération : statut 'generated' + fichier pain.001 produit (aucune écriture).
--   2. Confirmation : statut 'confirmed', les écritures de règlement sont postées.
--   Annulation possible avant confirmation : statut 'cancelled'.
--
-- DC1 (non-breaking) : l'état « en cours de paiement » d'une facture n'ajoute PAS
-- de statut à supplier_invoices (sa contrainte CHECK reste 'open'/'paid'/'cancelled').
-- Il est porté par l'appartenance à un lot 'generated' (JOIN payment_batch_items).
-- Le verrouillage « 1 facture dans 1 seul lot actif » est APPLICATIF (SELECT … FOR
-- UPDATE sur supplier_invoices + guard) — PAS de UNIQUE SQL (MariaDB n'a pas d'index
-- UNIQUE partiel filtré ; une facture doit redevenir sélectionnable après annulation).
--
-- Non-breaking (CREATE TABLE seul) → pas de bump kesh_version_min_required.

CREATE TABLE payment_batches (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    bank_account_id BIGINT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'generated',
    requested_execution_date DATE NOT NULL,
    total_amount DECIMAL(19,4) NOT NULL DEFAULT 0,
    msg_id VARCHAR(35) NOT NULL,
    payment_info_id VARCHAR(35) NOT NULL,
    confirmed_at DATETIME(3) NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_payment_batches_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_payment_batches_bank_account
        FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_payment_batches_status
        CHECK (status IN ('generated', 'confirmed', 'cancelled')),
    CONSTRAINT chk_payment_batches_total_non_negative CHECK (total_amount >= 0),
    INDEX idx_payment_batches_company_status (company_id, status),
    INDEX idx_payment_batches_company_date (company_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE payment_batch_items (
    id BIGINT NOT NULL AUTO_INCREMENT,
    payment_batch_id BIGINT NOT NULL,
    supplier_invoice_id BIGINT NOT NULL,
    position INT NOT NULL,
    end_to_end_id VARCHAR(35) NOT NULL,
    amount DECIMAL(19,4) NOT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_payment_batch_items_batch
        FOREIGN KEY (payment_batch_id) REFERENCES payment_batches(id) ON DELETE CASCADE,
    CONSTRAINT fk_payment_batch_items_supplier_invoice
        FOREIGN KEY (supplier_invoice_id) REFERENCES supplier_invoices(id) ON DELETE RESTRICT,
    CONSTRAINT chk_payment_batch_items_amount_positive CHECK (amount > 0),
    -- Unicité de la facture DANS un lot donné (pas globale — réutilisable après cancel).
    CONSTRAINT uq_payment_batch_items_batch_invoice UNIQUE (payment_batch_id, supplier_invoice_id),
    CONSTRAINT uq_payment_batch_items_position UNIQUE (payment_batch_id, position),
    INDEX idx_payment_batch_items_invoice (supplier_invoice_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
