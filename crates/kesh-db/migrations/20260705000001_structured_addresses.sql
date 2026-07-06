-- Story #213 — Adresses structurées (QR-bill type S), conformité SIX.
--
-- SIX a supprimé l'adresse combinée (type K) des QR-factures / pain.001
-- (échéance 21.11.2025) : seule l'adresse structurée (type S), avec rue,
-- n° de bâtiment, NPA, localité et pays en champs séparés, est acceptée.
--
-- Ces colonnes deviennent la SOURCE DE VÉRITÉ pour la génération QR/pain.001.
-- Le champ libre `address` préexistant reste conservé comme chaîne d'affichage
-- DÉRIVÉE (recomposée depuis ces champs à l'écriture côté handler).
--
-- Companies (créancier QR) : champs requis → NOT NULL DEFAULT '' (la non-vacuité
-- des champs requis est imposée côté API, pas par la DB, pour messages i18n).
-- Contacts (débiteur/fournisseur) : optionnels → NULL.
--
-- Longueurs alignées SIX 2.2 §3.3 type S : rue ≤70, n° ≤16, NPA ≤16, localité
-- ≤35, pays = ISO-3166-1 alpha-2. Non-breaking (ADD COLUMN avec défaut /
-- nullable) → pas de bump `kesh_version_min_required`.

ALTER TABLE companies
    ADD COLUMN address_street      VARCHAR(70) NOT NULL DEFAULT '',
    ADD COLUMN address_building    VARCHAR(16) NOT NULL DEFAULT '',
    ADD COLUMN address_postal_code VARCHAR(16) NOT NULL DEFAULT '',
    ADD COLUMN address_city        VARCHAR(35) NOT NULL DEFAULT '',
    ADD COLUMN address_country     CHAR(2)     NOT NULL DEFAULT 'CH';

ALTER TABLE contacts
    ADD COLUMN address_street      VARCHAR(70) NULL,
    ADD COLUMN address_building    VARCHAR(16) NULL,
    ADD COLUMN address_postal_code VARCHAR(16) NULL,
    ADD COLUMN address_city        VARCHAR(35) NULL,
    ADD COLUMN address_country     CHAR(2)     NULL;
