-- Story 19-3 (Epic 19) — dimension analytique document-level sur les factures
-- fournisseurs. Le `project_id` choisi sur la facture est propagé sur les lignes
-- de l'écriture d'achat à la comptabilisation (via `NewJournalEntry.project_id`).
--
-- Non-breaking (ADD COLUMN nullable) → pas de bump `kesh_version_min_required`.
-- Invariant `projects.company_id == supplier_invoices.company_id` non imposable par
-- FK MariaDB → vérifié côté repo à la création.

ALTER TABLE supplier_invoices
    ADD COLUMN project_id BIGINT NULL,
    ADD CONSTRAINT fk_supplier_invoices_project
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT;

CREATE INDEX idx_supplier_invoices_project ON supplier_invoices (project_id);
