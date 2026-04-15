-- Story 5.3 P3 — country code ISO-3166-1 alpha-2 sur companies et contacts.
-- Défaut 'CH' pour les données existantes (single-tenant Suisse v0.1).
-- La colonne est NOT NULL avec DEFAULT, ce qui évite de toucher aux INSERT
-- existants (les structs `NewCompany` / `NewContact` n'ont pas à porter le
-- champ en v0.1 — une story future ajoutera le paramétrage côté UI).

-- Étape 1 : ajout de la colonne avec DEFAULT 'CH' (pas de CHECK initial).
-- Les rows existantes reçoivent 'CH' automatiquement.
-- `IF NOT EXISTS` (MariaDB ≥ 10.3) rend la migration ré-entrante en cas de
-- crash partiel entre étapes (chaque ALTER TABLE provoque un commit implicite).
ALTER TABLE companies
    ADD COLUMN IF NOT EXISTS country CHAR(2) NOT NULL DEFAULT 'CH';

ALTER TABLE contacts
    ADD COLUMN IF NOT EXISTS country CHAR(2) NOT NULL DEFAULT 'CH';

-- Étape 2 supprimée après review Groupe A : un UPDATE correctif silencieux
-- pouvait écraser en 'CH' des codes pré-existants mal formés (ex. 'fr' en
-- minuscules, 'USA' sur 3 caractères) introduits manuellement hors Kesh,
-- causant une perte de données irréversible. Les codes non conformes
-- feront échouer le CHECK ci-dessous — le diagnostic est alors explicite
-- et le remédiation humaine (script ad-hoc).
--
-- Étape 3 : contrainte CHECK (MariaDB ≥ 10.2 / MySQL ≥ 8.0).
ALTER TABLE companies
    ADD CONSTRAINT IF NOT EXISTS chk_companies_country
    CHECK (country REGEXP '^[A-Z]{2}$');

ALTER TABLE contacts
    ADD CONSTRAINT IF NOT EXISTS chk_contacts_country
    CHECK (country REGEXP '^[A-Z]{2}$');
