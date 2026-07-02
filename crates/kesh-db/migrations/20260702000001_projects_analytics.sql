-- Story 19-1 — Socle de la comptabilité analytique par projet (Epic 19).
--
-- (1) Table `projects` : projets analytiques company-scoped, hiérarchie à 2 niveaux
--     (parent_id NULL = racine ; un sous-projet pointe une racine — contrainte 1 seul
--     niveau appliquée côté repo, MariaDB ne permettant pas de CHECK récursif).
-- (2) Dimension `project_id` sur `journal_entry_lines` (nullable = optionnel).
--
-- Non-breaking (CREATE TABLE nouvelle + ADD COLUMN nullable) → PAS de bump
-- `kesh_version_min_required` (cf. Migration breaking policy P1/P3).
--
-- Note tenant : la FK `project_id` garantit l'intégrité référentielle mais MariaDB
-- ne peut pas imposer `projects.company_id == journal_entries.company_id` ; cette
-- invariance est vérifiée côté handler au moment du tag (stories 19-2..19-5).

CREATE TABLE projects (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    parent_id BIGINT NULL,
    code VARCHAR(32) NOT NULL,
    name VARCHAR(150) NOT NULL,
    description TEXT NULL,
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    start_date DATE NULL,
    end_date DATE NULL,
    version INT NOT NULL DEFAULT 0,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_projects_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_projects_parent
        FOREIGN KEY (parent_id) REFERENCES projects(id) ON DELETE RESTRICT,
    CONSTRAINT uq_projects_company_code UNIQUE (company_id, code),
    INDEX idx_projects_company_parent (company_id, parent_id),
    INDEX idx_projects_company_archived (company_id, archived)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

ALTER TABLE journal_entry_lines
    ADD COLUMN project_id BIGINT NULL,
    ADD CONSTRAINT fk_jel_project
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT;

CREATE INDEX idx_jel_project ON journal_entry_lines (project_id);
