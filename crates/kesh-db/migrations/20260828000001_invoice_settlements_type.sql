-- Story 24-3 (#372) : le MODE de règlement d'une facture client.
--
-- NON-BREAKING : `ADD COLUMN` avec défaut + `ADD CONSTRAINT` — un binaire
-- antérieur ignore les colonnes et n'écrit jamais de ligne qui violerait les
-- contraintes (il ne connaît pas la table `invoice_settlements`, créée la veille
-- par la 24-2). → PAS de bump `kesh_version_min_required` (P1/P2).
--
-- ⚠️ ELLE ÉCRIT DES DONNÉES (le backfill du bas) → triage P7 OBLIGATOIRE.
-- Inscrite au registre `POST_RESTORE_BACKFILLS` en CLASSE B, cf. le bas du
-- fichier et `post_restore.rs`.
--
-- ## Pourquoi le mode vit sur le RÈGLEMENT, pas sur la facture
--
-- Le symétrique fournisseur porte `settlement_type` sur `supplier_invoices`
-- (20260628000001) parce qu'une facture fournisseur se règle EN UNE FOIS. Côté
-- client, non : la 24-2 a établi qu'une facture se règle en plusieurs virements,
-- et un avoir peut en éteindre une part. Une même facture peut donc être réglée
-- moitié en espèces, moitié par virement — le mode est une propriété de chaque
-- règlement, jamais de la facture.
--
-- ## Le vocabulaire est celui du fournisseur, mot pour mot
--
-- `('bank_transfer', 'internal_account')`, comme
-- `chk_supplier_invoices_settlement_type`. Deux vocabulaires pour la même notion
-- coûteraient à chaque lecture.
--
-- ⚠️ `internal_account` n'est PAS « la caisse » : c'est n'importe quel compte du
-- plan — caisse 1000, poste 1010, compte de compensation. Le mode de règlement
-- est indifférent au traitement comptable ; seule change la contrepartie.

ALTER TABLE invoice_settlements
    ADD COLUMN settlement_type VARCHAR(20) NOT NULL DEFAULT 'bank_transfer',
    ADD COLUMN settlement_bank_account_id BIGINT NULL,
    ADD COLUMN settlement_account_id BIGINT NULL,
    ADD CONSTRAINT fk_invoice_settlements_settlement_bank
        FOREIGN KEY (settlement_bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT fk_invoice_settlements_settlement_account
        FOREIGN KEY (settlement_account_id) REFERENCES accounts(id) ON DELETE RESTRICT;

-- ⛔ BACKFILL — il DOIT précéder la contrainte de contrepartie, sinon celle-ci
-- refuserait les lignes déjà écrites par la 24-2.
--
-- Ces lignes viennent toutes de la réconciliation bancaire : `bank_transfer` dit
-- d'elles la vérité, il ne la fabrique pas. Le compte bancaire se retrouve par
-- la LIGNE DE DÉBIT de l'écriture de règlement — c'est le compte au grand livre,
-- et `bank_accounts.journal_account_id` le rattache à son compte bancaire.
--
-- Gardé `IS NULL` : rejoué sur une base à jour, il ne touche rien.
UPDATE invoice_settlements s
JOIN journal_entry_lines jel
    ON jel.entry_id = s.journal_entry_id AND jel.debit > 0
JOIN bank_accounts ba
    ON ba.journal_account_id = jel.account_id AND ba.company_id = s.company_id
SET s.settlement_bank_account_id = ba.id
WHERE s.settlement_bank_account_id IS NULL;

-- La contrepartie est exigée, et elle est EXCLUSIVE : un règlement a un mode et
-- un seul, donc exactement une des deux références.
ALTER TABLE invoice_settlements
    ADD CONSTRAINT chk_invoice_settlements_type
        CHECK (settlement_type IN ('bank_transfer', 'internal_account')),
    ADD CONSTRAINT chk_invoice_settlements_counterparty
        CHECK ((settlement_type = 'bank_transfer'
                AND settlement_bank_account_id IS NOT NULL
                AND settlement_account_id IS NULL)
            OR (settlement_type = 'internal_account'
                AND settlement_account_id IS NOT NULL
                AND settlement_bank_account_id IS NULL));
