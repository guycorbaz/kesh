-- Story 26-2a (#622) — description optionnelle d'un compte bancaire.
--
-- Une courte description (« Personnel », « Épargne ») qui distingue deux
-- comptes de la même banque ; NULL = pas de description, l'affichage reste
-- celui d'avant la colonne.
--
-- Non-breaking (P1/P3 CLAUDE.md) : colonne nullable ajoutée en fin de table ;
-- un binaire antérieur l'ignore, ses requêtes nomment leurs colonnes. Aucun
-- bump de la version minimale requise.
--
-- 60 caractères : un nom court de compte, pas un commentaire. La borne vit
-- aussi dans la constante BANK_ACCOUNT_DESCRIPTION_MAX_CHARS de kesh-db, qu'un
-- test confronte au schéma de test régénéré.
--
-- Ni contrainte ni index : la description n'est pas unique (deux comptes
-- peuvent porter la même), et rien ne la cherche.
--
-- Aucune donnée écrite : rien à rejouer après restauration (garde P7).

ALTER TABLE bank_accounts
    ADD COLUMN description VARCHAR(60) NULL,
    ALGORITHM=INSTANT, LOCK=NONE;
