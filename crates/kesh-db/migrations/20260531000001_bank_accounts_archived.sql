-- Story v014-1 — Ajout du flag `archived` pour soft-delete des bank_accounts.
--
-- Non-breaking migration (P3 CLAUDE.md) : ADD COLUMN NOT NULL DEFAULT FALSE,
-- les anciens binaires v0.1.3 ignorent le nouveau champ. Pas de bump
-- `kesh_version_min_required` requis.
--
-- Pas d'index ajouté (FINDING-8 Pass 3 Opus YAGNI) : la table bank_accounts
-- a ~10 rows max par company (L3 spec) et les query plans existants utilisent
-- déjà la FK company_id pour scoper. Un index (company_id, archived) n'apporte
-- aucun bénéfice perf vs full scan d'une table <100 rows + introduirait un
-- coût de maintenance write (PATCH/PUT/DELETE). Si volume futur > 1000
-- rows/company impose un index, l'ajouter ultérieurement dans une migration
-- dédiée avec EXPLAIN à l'appui.

ALTER TABLE bank_accounts
    ADD COLUMN archived BOOLEAN NOT NULL DEFAULT FALSE,
    ALGORITHM=INSTANT, LOCK=NONE;
