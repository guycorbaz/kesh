-- Migration Story 4.1 : carnet d'adresses (CRUD contacts)
-- FR25 (carnet unifié) + FR26 (flags client/fournisseur) + FR27 (IDE CHE) + FR28 schéma (default_payment_terms).

CREATE TABLE contacts (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    contact_type VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    is_client BOOLEAN NOT NULL DEFAULT FALSE,
    is_supplier BOOLEAN NOT NULL DEFAULT FALSE,
    address VARCHAR(500) NULL,
    email VARCHAR(320) NULL,
    phone VARCHAR(50) NULL,
    ide_number VARCHAR(12) NULL,
    default_payment_terms VARCHAR(100) NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_contacts_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_contacts_company_ide UNIQUE (company_id, ide_number),
    CONSTRAINT chk_contacts_name_not_empty CHECK (CHAR_LENGTH(TRIM(name)) > 0),
    CONSTRAINT chk_contacts_type CHECK (
        BINARY contact_type IN (BINARY 'Personne', BINARY 'Entreprise')
    ),
    INDEX idx_contacts_company_active (company_id, active),
    INDEX idx_contacts_company_name (company_id, name)
);
