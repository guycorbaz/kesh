-- Story 16-3a (#151) : coordonnées de contact de l'émetteur sur le PDF de facture.
--
-- Le bloc gauche du PDF ne portait que nom, adresse et IDE. Un client qui a une
-- question sur la facture n'y trouvait aucun moyen de joindre l'émetteur.
-- `companies.email` existait déjà (Story 20-3b1) mais n'était pas rendue ; ces
-- deux colonnes la complètent.
--
-- Longueurs alignées sur le précédent du dépôt : `contacts.phone` est
-- VARCHAR(50) (20260414000001), `companies.email` VARCHAR(320)
-- (20260709000003). `website` prend 255, longueur usuelle d'une URL.
--
-- ⚠️ Ce sont des longueurs de STOCKAGE, PAS d'affichage. Le bloc gauche du PDF
-- ne dispose que de ~100 mm, soit une cinquantaine de caractères à 9 pt : une
-- valeur plus longue est TRONQUÉE au rendu (`IDENTITY_MAX_CHARS`, `pdf.rs`),
-- jamais refusée. Une version antérieure de ce commentaire prétendait que 255
-- caractères tenaient sur 100 mm — faux d'un facteur quatre, et c'est ce qui
-- avait laissé passer l'absence de troncature (revue de code, passe 1).
--
-- Non-breaking (ADD COLUMN nullable) → PAS de bump `kesh_version_min_required`
-- ni de version Cargo (garde-fous P1/P2 de CLAUDE.md).
--
-- DDL pur : aucun UPDATE, aucun INSERT. Ne relève donc NI du registre
-- `POST_RESTORE_BACKFILLS` NI des `EXEMPT_MIGRATIONS` (garde-fou P7) — il n'y a
-- rien à rejouer après un import d'installation, ce que constate
-- `every_data_backfill_migration_is_triaged` en ne la sélectionnant jamais.
ALTER TABLE companies
    ADD COLUMN phone VARCHAR(50) NULL,
    ADD COLUMN website VARCHAR(255) NULL;
