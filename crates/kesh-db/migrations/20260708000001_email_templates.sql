-- Epic 20 (#224) — Story 20-1 : socle templates d'e-mail company-scoped.
--
-- Une ligne = un OVERRIDE explicite du template par défaut pour
-- (company, template_type, language). Absence de ligne = fallback sur
-- le texte par défaut (constantes Rust, cf. email_template_defaults.rs) ;
-- ce n'est PAS une erreur, c'est le comportement zéro-config attendu.
--
-- `template_type` (pas `type` — convention projet : toujours suffixer
-- `_type`, cf. `contact_type`/`org_type`/`account_type`). v1 : `invoice_send`
-- seul (CHECK élargissable par migration future, non-breaking).
--
-- `language` calque `chk_companies_instance_language` (CHAR(2) + CHECK
-- BINARY pour comparaison case-sensitive, la collation par défaut MariaDB
-- étant case-insensitive).
--
-- `version` : verrou optimiste (cf. docs/optimistic-locking-patterns.md).
--
-- Non-breaking (ADD TABLE) → pas de bump kesh_version_min_required (P3).

CREATE TABLE email_templates (
    id              BIGINT      NOT NULL AUTO_INCREMENT,
    company_id      BIGINT      NOT NULL,
    template_type   VARCHAR(50) NOT NULL,
    language        CHAR(2)     NOT NULL,
    subject         TEXT        NOT NULL,
    body            TEXT        NOT NULL,
    version         INT         NOT NULL DEFAULT 1,
    created_at      DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at      DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    PRIMARY KEY (id),
    CONSTRAINT fk_email_templates_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    CONSTRAINT chk_email_templates_template_type
        CHECK (template_type IN ('invoice_send')),
    CONSTRAINT chk_email_templates_language
        CHECK (BINARY language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN')),
    CONSTRAINT uq_email_templates_company_type_language
        UNIQUE (company_id, template_type, language),
    INDEX idx_email_templates_company (company_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
