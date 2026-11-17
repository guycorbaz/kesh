-- NON BREAKING au sens du paragraphe ⚠️ de P1 (CLAUDE.md, Migration breaking
-- policy) : « sans relever `kesh_version_min_required` et sans corrompre le
-- schéma ». `kesh_version_min_required` reste à 0.17.6 ; aucune version Cargo
-- n'est relevée pour elle (P2/P2-bis).
--
-- Story 43-1a (#823) — un règlement pour plusieurs factures (Epic 43, C-43-2 à
-- C-43-5, C-43-24). DDL seul : aucune ligne écrite, ni registre de rejeu ni
-- exemption (P7). Ligne ajoutée à docs/migrations-idempotence-audit.md (P5).
--
-- 1. invoice_settlements.journal_entry_line_id : la ligne de créance qui porte
--    CE règlement dans une écriture qui en porte plusieurs (règlement groupé).
--    NULL = règlement unitaire, la règle d'avant (toutes les lignes du compte
--    de tiers de l'écriture). Clé étrangère vers une LIGNE : une écriture dont
--    une ligne est référencée par un règlement est possédée, donc jamais
--    réécrite par la modification d'écriture (Story 43-1a, AC2).
-- 2. L'unicité change de forme : (journal_entry_id, invoice_id) au lieu de
--    (journal_entry_id). La clé composée est posée AVANT le DROP INDEX : la
--    clé étrangère fk_invoice_settlements_entry s'appuie alors sur elle
--    (colonne de tête) — erreur 1553 sinon, précédent 30-2a
--    (20261015000001). Prouvé sur MariaDB 10.11 (Story 43-1a, T0).
-- 3. supplier_invoices.settlement_journal_entry_line_id : le symétrique
--    fournisseur (Story 43-1b), avec une CHECK « ligne ⇒ écriture ». Les
--    colonnes restent même si la 43-1b sort de la release : vides, sans effet.
-- 4. company_invoice_settings.group_settlement_max_difference : le plafond de
--    l'écart (moins-payé ou trop-perçu) admis dans un règlement groupé, au
--    compte que l'utilisateur choisit, réglage de la société (C-43-24),
--    5.00 par défaut, entre 0 et 100. Une sauvegarde 0.18.0 n'a pas la
--    colonne : à son import, la valeur par défaut s'applique — voulu (P7).
--
-- Ce que la 0.18.0 voit : elle refuse de démarrer sur une base migrée
-- (« was previously applied but is missing ») et refuse toute sauvegarde 0.19.0
-- (colonnes inconnues) ; aucun chemin ne lui fait donc lire un groupe, que son
-- classement des rapprochements (une écriture, un règlement) lirait mal.
-- DROP INDEX n'est pas dans la liste de P3.
--
-- DDL non transactionnel en MariaDB : un échec au milieu laisse les premières
-- instructions posées et la migration non inscrite ; le rejeu échoue alors en
-- 1060. Reprise : restaurer la sauvegarde pre-upgrade/ (manuel
-- d'administration, « Passer à la 0.19.0 »).
ALTER TABLE invoice_settlements
    ADD COLUMN journal_entry_line_id BIGINT NULL
        COMMENT 'Ligne de créance de ce règlement dans une écriture groupée ; NULL = règlement unitaire',
    ADD CONSTRAINT fk_invoice_settlements_entry_line
        FOREIGN KEY (journal_entry_line_id) REFERENCES journal_entry_lines(id) ON DELETE RESTRICT,
    ADD UNIQUE KEY uq_invoice_settlements_entry_line (journal_entry_line_id),
    ADD UNIQUE KEY uq_invoice_settlements_entry_invoice (journal_entry_id, invoice_id);

ALTER TABLE invoice_settlements
    DROP INDEX uq_invoice_settlements_entry;

ALTER TABLE supplier_invoices
    ADD COLUMN settlement_journal_entry_line_id BIGINT NULL
        COMMENT 'Ligne de dette de ce règlement dans une écriture groupée ; NULL = règlement unitaire',
    ADD CONSTRAINT fk_supplier_invoices_settlement_line
        FOREIGN KEY (settlement_journal_entry_line_id) REFERENCES journal_entry_lines(id) ON DELETE RESTRICT,
    ADD UNIQUE KEY uq_supplier_invoices_settlement_line (settlement_journal_entry_line_id),
    ADD CONSTRAINT chk_supplier_invoices_settlement_line
        CHECK (settlement_journal_entry_line_id IS NULL OR settlement_journal_entry_id IS NOT NULL);

ALTER TABLE company_invoice_settings
    ADD COLUMN group_settlement_max_difference DECIMAL(19,4) NOT NULL DEFAULT 5.0000
        COMMENT 'Plafond de l''écart admis dans un règlement groupé (0 à 100)',
    ADD CONSTRAINT chk_company_invoice_settings_group_max
        CHECK (group_settlement_max_difference >= 0 AND group_settlement_max_difference <= 100);
