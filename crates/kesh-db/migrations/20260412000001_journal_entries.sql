-- Story 3.2 : Saisie d'écritures en partie double
-- Crée les tables journal_entries (en-tête) et journal_entry_lines (détail).
--
-- Intégrité comptable garantie par :
-- - CHECK BINARY sur le journal (5 valeurs : Achats, Ventes, Banque, Caisse, OD)
-- - CHECK d'exclusivité débit/crédit par ligne
-- - FK RESTRICT vers fiscal_years (immutabilité post-clôture contrôlée applicativement)
-- - FK RESTRICT vers accounts (un compte utilisé ne peut être supprimé)
-- - UNIQUE (company_id, fiscal_year_id, entry_number) : numérotation sans trou
-- - ON DELETE CASCADE sur journal_entry_lines → journal_entries : suppression bulk
--
-- La validation SUM(debit) = SUM(credit) n'est PAS dans un CHECK DB (impossible
-- cross-row en MariaDB) — elle est faite dans kesh-core::accounting::validate()
-- et re-vérifiée après INSERT dans le repository avec rollback si mismatch.

CREATE TABLE journal_entries (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT NOT NULL,
    fiscal_year_id BIGINT NOT NULL,
    entry_number BIGINT NOT NULL COMMENT 'Séquentiel par (company_id, fiscal_year_id), jamais de trou. BIGINT pour instances multi-décennies.',
    entry_date DATE NOT NULL,
    journal VARCHAR(10) NOT NULL COMMENT 'Achats|Ventes|Banque|Caisse|OD',
    description VARCHAR(500) NOT NULL,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_journal_entries_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_journal_entries_fiscal_year FOREIGN KEY (fiscal_year_id) REFERENCES fiscal_years(id) ON DELETE RESTRICT,
    CONSTRAINT uq_journal_entries_number UNIQUE (company_id, fiscal_year_id, entry_number),
    CONSTRAINT chk_journal_entries_journal CHECK (BINARY journal IN (BINARY 'Achats', BINARY 'Ventes', BINARY 'Banque', BINARY 'Caisse', BINARY 'OD')),
    CONSTRAINT chk_journal_entries_description_nonempty CHECK (CHAR_LENGTH(TRIM(description)) > 0),
    CONSTRAINT chk_journal_entries_entry_number_positive CHECK (entry_number > 0),
    INDEX idx_journal_entries_company_date (company_id, entry_date DESC),
    INDEX idx_journal_entries_fiscal_year (fiscal_year_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE journal_entry_lines (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    entry_id BIGINT NOT NULL,
    account_id BIGINT NOT NULL,
    line_order INT NOT NULL COMMENT 'Position dans l''écriture (1, 2, 3...)',
    debit DECIMAL(19,4) NOT NULL DEFAULT 0,
    credit DECIMAL(19,4) NOT NULL DEFAULT 0,
    CONSTRAINT fk_jel_entry FOREIGN KEY (entry_id) REFERENCES journal_entries(id) ON DELETE CASCADE,
    CONSTRAINT fk_jel_account FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_jel_debit_credit_exclusive CHECK ((debit = 0 AND credit > 0) OR (debit > 0 AND credit = 0)),
    CONSTRAINT chk_jel_debit_nonneg CHECK (debit >= 0),
    CONSTRAINT chk_jel_credit_nonneg CHECK (credit >= 0),
    CONSTRAINT uq_jel_entry_order UNIQUE (entry_id, line_order),
    INDEX idx_jel_entry (entry_id),
    INDEX idx_jel_account (account_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
