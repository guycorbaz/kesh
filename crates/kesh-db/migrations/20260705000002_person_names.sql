-- Story #213 — Séparation Personne / Entreprise (prénom + nom).
--
-- Un contact `Personne` (et une société `OrgType::Independant`) a un prénom et
-- un nom distincts ; le champ `name` reste la SOURCE d'affichage/QR canonique,
-- recomposé côté handler (« Prénom Nom ») pour les personnes physiques. Pour les
-- entreprises/PME, `name` = raison sociale, prénom/nom NULL.
--
-- Non-breaking (ADD COLUMN nullable) → pas de bump kesh_version_min_required.

ALTER TABLE companies
    ADD COLUMN first_name VARCHAR(70) NULL,
    ADD COLUMN last_name  VARCHAR(70) NULL;

ALTER TABLE contacts
    ADD COLUMN first_name VARCHAR(70) NULL,
    ADD COLUMN last_name  VARCHAR(70) NULL;
