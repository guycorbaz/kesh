-- Story 19-5 (Epic 19) — projet analytique par défaut sur une règle de
-- réconciliation. Quand une règle d'affectation est appliquée à l'accept
-- (`POST /reconciliation/accept` body `{type: 'rule'}`), le `default_project_id`
-- de la règle est recopié sur l'écriture générée (`NewJournalEntry.project_id`),
-- se propageant aux 2 lignes (banque + contrepartie) via `line.project_id.or()`.
--
-- Non-breaking (ADD COLUMN nullable) → pas de bump `kesh_version_min_required`.
-- Invariant `projects.company_id == reconciliation_rules.company_id` non
-- imposable par FK MariaDB → vérifié côté repo à la création/édition de la règle
-- et re-validé à l'accept (le projet a pu être archivé entre-temps).

ALTER TABLE reconciliation_rules
    ADD COLUMN default_project_id BIGINT NULL,
    ADD CONSTRAINT fk_reconciliation_rules_default_project
        FOREIGN KEY (default_project_id) REFERENCES projects(id) ON DELETE RESTRICT;

CREATE INDEX idx_reconciliation_rules_default_project ON reconciliation_rules (default_project_id);
