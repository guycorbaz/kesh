-- Story #213 — Personnes de contact d'une entreprise (CRM).
--
-- Une entreprise (contact de type `Entreprise`) peut avoir 0..N personnes de
-- contact (interlocuteurs). PUREMENT INFORMATIF : ces personnes ne sont JAMAIS
-- utilisées sur les factures / QR-bill / pain.001 (décision Guy 2026-07-05).
--
-- Scopé par `company_id` (multi-tenant) + rattaché au `contact_id` parent.
-- Archivage soft (`active`) cohérent avec `contacts`. Non-breaking (nouvelle
-- table) → pas de bump kesh_version_min_required.

CREATE TABLE contact_persons (
    id          BIGINT       NOT NULL AUTO_INCREMENT,
    company_id  BIGINT       NOT NULL,
    contact_id  BIGINT       NOT NULL,
    first_name  VARCHAR(70)  NOT NULL,
    last_name   VARCHAR(70)  NOT NULL,
    role        VARCHAR(100) NULL,
    email       VARCHAR(320) NULL,
    phone       VARCHAR(50)  NULL,
    active      BOOLEAN      NOT NULL DEFAULT TRUE,
    version     INT          NOT NULL DEFAULT 1,
    created_at  DATETIME(3)  NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at  DATETIME(3)  NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_contact_persons_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    CONSTRAINT fk_contact_persons_contact FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
    INDEX idx_contact_persons_contact (contact_id, active),
    CONSTRAINT chk_contact_persons_names CHECK (CHAR_LENGTH(TRIM(first_name)) > 0 AND CHAR_LENGTH(TRIM(last_name)) > 0)
);
