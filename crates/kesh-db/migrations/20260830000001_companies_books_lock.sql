-- Story 24-4c (#380) : le VERROU DE PÉRIODE — arrêter les livres à une date.
--
-- NON-BREAKING : `ADD COLUMN` nullable, sans défaut non nul — un binaire
-- antérieur ignore la colonne et ne peut écrire aucune ligne qui la violerait
-- (il ne sait pas verrouiller). → PAS de bump `kesh_version_min_required`
-- (P1/P2), donc pas de bump de version Cargo (P2-bis).
--
-- DDL PUR — aucun `UPDATE`, `INSERT`, `REPLACE` ni `DELETE` : ne relève NI du
-- registre `POST_RESTORE_BACKFILLS` NI des exemptions (garde-fou P7).
--
-- ## Ce que cette colonne ferme, et ce qu'elle NE ferme PAS
--
-- La 24-4b a supprimé `journal_entries::update` et refuse le `DELETE` : une
-- écriture enregistrée n'est plus modifiable. Ce qui restait ouvert, c'est
-- l'ANTIDATAGE — créer aujourd'hui une écriture datée d'un trimestre déjà
-- déclaré, ce qui change ses totaux de TVA sans que rien ne le signale, le
-- rapport TVA se recalculant à la volée.
--
-- ⚠️ `NULL` = aucun verrou. C'est la valeur de toutes les installations
-- existantes après cette migration : leur comportement est INCHANGÉ.
--
-- ## Pourquoi une date et non une table de périodes
--
-- Une période « déclarée » ne se ferme jamais au milieu : on déclare le T1
-- après le T1, jamais avant. Une BORNE exprime donc exactement ce qu'on veut
-- dire, avec un champ au lieu d'une table — et elle interdit par construction
-- les trous (T1 et T3 verrouillés, T2 ouvert) qui n'ont aucun sens comptable.
--
-- ⚠️ Le verrou ne se dérive PAS du décompte TVA : le décompte n'existe pas
-- comme objet dans ce dépôt (ni entité, ni table, ni colonne). Quand il
-- existera, il PROPOSERA la date — la borne restera le mécanisme.
--
-- ## Le seuil est INCLUSIF
--
-- `entry_date <= books_locked_through` est refusé : une borne au 31.03
-- verrouille le 31.03. C'est ce que « jusqu'au 31 mars inclus » veut dire.

ALTER TABLE companies
    ADD COLUMN books_locked_through DATE NULL
        COMMENT 'Story 24-4c (#380) : borne INCLUSIVE du verrou de période. NULL = aucun verrou. Aucune écriture ne peut être créée avec une entry_date <= cette date.';
