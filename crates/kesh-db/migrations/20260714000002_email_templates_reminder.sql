-- Migration Story 21-3 — extension `email_templates` pour les rappels par niveau.
-- Issue #231 (Epic 21). Option A+ du plan d'epic (templates de rappel DANS
-- email_templates, pas dans dunning_levels).
--
-- BREAKING SUR LES DONNÉES (politique P1/P2 CLAUDE.md `## Migration breaking policy`).
-- Le DDL lui-même (ADD COLUMN DEFAULT, ALTER UNIQUE/CHECK) reste compatible avec un
-- binaire antérieur, MAIS dès qu'une ligne `template_type = 'invoice_reminder'` existe,
-- un binaire v0.6 downgradé fait un 500 sur `list_effective_for_company` : son
-- `FromStr<EmailTemplateType>` est strict et rejette la valeur inconnue, `Decode` échoue.
-- → bump `kesh_version_min_required = '0.7.0'` (version cible de la PR / release Epic 21),
--   figé dans le SQL (comme '0.1.0' d'origine). Coût nul, honnête (D14). Le bump bloque
--   le binaire au boot (check_downgrade_protection), indépendamment de la présence de données.
--
-- MariaDB : le CHECK ne se modifie pas en place → DROP CONSTRAINT + ADD CONSTRAINT.
-- La contrainte UNIQUE se pilote comme un index → DROP INDEX + ADD CONSTRAINT UNIQUE.
--
-- `level_number SMALLINT NOT NULL DEFAULT 0` : 0 = template générique (JAMAIS de NULL
-- dans l'UNIQUE, leçon Epic 20 MariaDB) ; les lignes existantes deviennent niveau 0.
--
-- Idempotence (docs/migrations-idempotence-audit.md) : tracked-by-sqlx — re-exécution
-- hors sqlx échouerait (1060 colonne existante / contrainte existante). BREAKING → bump.

ALTER TABLE email_templates
    ADD COLUMN level_number SMALLINT NOT NULL DEFAULT 0;

-- Créer le NOUVEL UNIQUE (préfixe `company_id`) AVANT de dropper l'ancien :
-- le FK `fk_email_templates_company` a besoin d'un index couvrant `company_id`.
-- Sans cet ordre, MariaDB refuse le DROP (erreur 1553).
ALTER TABLE email_templates
    ADD CONSTRAINT uq_email_templates_company_type_language_level
        UNIQUE (company_id, template_type, language, level_number);

ALTER TABLE email_templates
    DROP INDEX uq_email_templates_company_type_language;

ALTER TABLE email_templates
    DROP CONSTRAINT chk_email_templates_template_type;

ALTER TABLE email_templates
    ADD CONSTRAINT chk_email_templates_template_type
        CHECK (template_type IN ('invoice_send', 'invoice_reminder'));

-- Bump breaking (P2) — DERNIÈRE instruction du fichier.
UPDATE _kesh_version SET kesh_version_min_required = '0.7.0' WHERE id = 1;
