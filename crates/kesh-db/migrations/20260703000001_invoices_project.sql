-- Story 19-4 (Epic 19) — dimension analytique document-level sur les factures
-- de vente. Le `project_id` choisi sur la facture (brouillon) est propagé sur
-- les lignes de l'écriture de vente à la validation (`validate_invoice`, via
-- `NewJournalEntry.project_id`) et hérité par la contre-passation d'avoir.
--
-- Non-breaking (ADD COLUMN nullable) → pas de bump `kesh_version_min_required`.
-- Invariant `projects.company_id == invoices.company_id` non imposable par
-- FK MariaDB → vérifié côté repo à la création/édition.

ALTER TABLE invoices
    ADD COLUMN project_id BIGINT NULL,
    ADD CONSTRAINT fk_invoices_project
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT;

CREATE INDEX idx_invoices_project ON invoices (project_id);
