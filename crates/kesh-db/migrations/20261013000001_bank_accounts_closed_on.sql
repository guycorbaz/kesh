-- Story 28-4a (#642) — clôture d'un compte bancaire, en lecture seule.
--
-- closed_on NULL = compte ouvert ; une date = compte clos, et sa date de
-- clôture (informative : c'est l'état qui garde, non la date). Le compte clos
-- reste lisible partout ; les gestes qui écrivent le refusent côté serveur.
--
-- Une seule colonne, et non un booléen plus une date : l'état et sa date ne
-- peuvent pas se contredire. Distincte d'archived, qui retire un compte qui
-- n'a jamais servi et ne se défait pas ; la clôture se défait (réouverture).
--
-- La contrainte chk_bank_accounts_closed_not_primary rend structurel
-- l'invariant I3 de la fiche : un compte clos n'est jamais le compte
-- principal (celui que porte la QR-facture). La clôture refuse le principal,
-- et toute désignation d'un compte clos comme principal échoue en base.
--
-- Non-breaking (P1/P3 CLAUDE.md) : colonne nullable ajoutée en fin de table,
-- contrainte de vérification satisfaite par toute ligne existante (closed_on
-- y est NULL) ; un binaire antérieur ignore la colonne, ses requêtes nomment
-- leurs colonnes. Aucun bump de la version minimale requise.
--
-- Aucune donnée écrite : rien à rejouer après restauration (garde P7).

ALTER TABLE bank_accounts
    ADD COLUMN closed_on DATE NULL,
    ALGORITHM=INSTANT, LOCK=NONE;
ALTER TABLE bank_accounts
    ADD CONSTRAINT chk_bank_accounts_closed_not_primary
        CHECK (closed_on IS NULL OR is_primary = FALSE);
