-- Story 8-3 — relax UNIQUE constraint on (company_id, file_hash) to allow
-- explicit re-import via confirmDuplicateFile=true (FR43 partie 1).
--
-- Le check applicatif via repositories::bank_imports::find_by_company_and_hash
-- (déjà existant 8-1b à bank_imports.rs:230) reste la source of truth pour la
-- détection. Sans le UNIQUE, le handler peut décider d'autoriser ou refuser
-- l'INSERT selon le flag multipart `confirmDuplicateFile`.
--
-- Trade-off documenté en spec §dedup-file (Limitations connues v0.1 L11) :
-- une race ouverte sur INSERT concurrent même hash sans flag confirm est
-- désormais possible, atténuée par la rareté empirique des imports
-- concurrents par le même user/company/hash. Mitigation curative
-- documentée (advisory lock GET_LOCK) à activer si KF émerge en prod.

-- L4 (Pass 1 review) — idempotency: support re-application or partial-state re-runs
-- without crashing on MariaDB error 1091 (index not found) or 1061 (duplicate key).
ALTER TABLE bank_imports DROP INDEX IF EXISTS uq_bank_imports_company_hash;
CREATE INDEX IF NOT EXISTS idx_bank_imports_company_hash ON bank_imports (company_id, file_hash);
