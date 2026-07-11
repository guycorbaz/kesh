-- Story 20-3b1 : langue et civilité par contact (décisions #11/#12 epic-20).
--
-- `language` : langue de correspondance du contact (e-mails + PDF facture).
--   NULL = hérite de `companies.instance_language` (résolution à l'envoi,
--   pas de copie à la création). CHECK calqué chk_companies_instance_language.
-- `salutation` : civilité pour la variable {salutation} des templates
--   d'e-mail (genre × langue × type de contact). Défaut 'Neutre'.
--
-- Non-breaking (ADD COLUMN nullable / DEFAULT) → pas de bump
-- `kesh_version_min_required` (politique Story 10-2 P1).
ALTER TABLE contacts
    ADD COLUMN language CHAR(2) NULL,
    ADD COLUMN salutation VARCHAR(10) NOT NULL DEFAULT 'Neutre';

ALTER TABLE contacts
    ADD CONSTRAINT chk_contacts_language
        CHECK (language IS NULL OR BINARY language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN')),
    ADD CONSTRAINT chk_contacts_salutation
        CHECK (salutation IN ('Monsieur', 'Madame', 'Neutre'));
