-- Story 24-4a (#380) : la CONTRE-PASSATION d'une écriture comptable.
--
-- NON-BREAKING : `ADD COLUMN` nullable + `ADD CONSTRAINT` — un binaire antérieur
-- ignore la colonne, et n'écrit jamais de ligne qui violerait la contrainte
-- (il ne sait pas contre-passer). → PAS de bump `kesh_version_min_required`
-- (P1/P2), donc pas de bump de version Cargo (P2-bis).
--
-- DDL PUR — aucun `UPDATE`, aucun `INSERT`, aucun `DELETE` : ne relève NI du
-- registre `POST_RESTORE_BACKFILLS` NI des exemptions (garde-fou P7).
--
-- ## Pourquoi une colonne auto-référente, et pourquoi UNIQUE
--
-- Art. 958f CO et Olico art. 3 : l'exigence n'est pas qu'on ne se trompe jamais,
-- c'est que la correction soit APPARENTE. Un journal d'audit ne suffit donc pas
-- — le lien doit vivre DANS LES LIVRES, là où le réviseur regarde.
--
-- Cette seule colonne porte trois propriétés d'un même geste :
--   1. la correction est apparente (le grand livre montre les deux écritures) ;
--   2. `UNIQUE` interdit STRUCTURELLEMENT de contre-passer deux fois la même
--      écriture — l'idempotence ne repose sur aucun pré-SELECT, donc elle tient
--      sous concurrence ;
--   3. le renvoi croisé se dérive sans seconde colonne à tenir cohérente
--      (`WHERE reverses_entry_id = ?` donne l'écriture qui contre-passe).
--
-- ⛔ On n'ajoute PAS de booléen `is_reversed` sur l'origine : ce serait deux
-- colonnes à tenir d'accord là où une seule suffit, et le lien inverse s'y
-- perdrait.
--
-- ## `RESTRICT`, et ses deux conséquences ASSUMÉES
--
-- Supprimer une écriture qu'on a corrigée effacerait la correction : le refus
-- est donc VOULU. Il a deux effets que la story porte explicitement :
--
--   (a) `journal_entries::delete_all_by_company` supprime en UN SEUL statement,
--       et InnoDB vérifie les FK ligne à ligne sans différer — l'ordre de
--       parcours déciderait du succès. Le repository remet donc la colonne à
--       NULL AVANT le DELETE. (Sans quoi l'échec serait INTERMITTENT, ce qui est
--       pire qu'un échec franc.)
--   (b) `DELETE /api/v1/journal-entries/{id}` sur une origine contre-passée
--       devient un 409 explicite, pas une violation de contrainte au message
--       opaque.

ALTER TABLE journal_entries
    ADD COLUMN reverses_entry_id BIGINT NULL
        COMMENT 'Écriture que celle-ci contre-passe. NULL = écriture ordinaire.',
    ADD CONSTRAINT fk_journal_entries_reverses
        FOREIGN KEY (reverses_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT,
    ADD CONSTRAINT uq_journal_entries_reverses UNIQUE (reverses_entry_id);
