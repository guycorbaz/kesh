-- BREAKING (P1, schéma) : `supplier_invoices.purchase_journal_entry_id` devient
-- NULLABLE. Un binaire 0.16.x ou rc1 décode cette colonne en entier non nul :
-- il échouerait sur le premier brouillon (liste, fiche, export). Le seuil
-- `kesh_version_min_required` est reposé à 0.17.0 en dernière instruction
-- (P2 ; même valeur que `20261030000001` — aucun bump Cargo, P2-bis déjà tenu :
-- les dix crates sont en 0.17.0).
--
-- Story v017-4a (#849) — le cycle brouillon → validée → payée d'une facture
-- fournisseur (C-v017-4-2, C-v017-4-3). Une facture naît en brouillon
-- (`draft`), SANS écriture d'achat ; la validation passe l'écriture et la
-- facture devient `open` (« validée, à payer » — la valeur stockée ne change
-- pas, l'écran dit « Validée ») ; la dévalidation la supprime et ramène le
-- brouillon. Aucune ligne existante n'est réécrite : toutes les factures
-- existantes ont leur écriture (`NOT NULL` jusqu'ici) et restent `open`,
-- `paid` ou `cancelled`.
--
-- Ordre des instructions — jamais un état sans garde, chaque ALTER étant
-- atomique sous MariaDB 10.11 :
--  1. la colonne devient nullable, sa clé étrangère gardée (forme essayée sur
--     MariaDB 10.11 au développement, Dev Agent Record : acceptée telle quelle ;
--     les replis par DROP/ADD FOREIGN KEY n'ont pas été retenus) ;
--  2. les trois CHECK neufs, posés ENSEMBLE, la contrainte de statut sous un
--     NOM NEUF (`…_status_v2`) — AVANT le retrait de l'ancienne : un échec entre
--     2 et 3 laisse `status` gardé par les deux contraintes (l'ancienne, plus
--     stricte, refuse `draft` ; aucune ligne `draft` n'existe), jamais par
--     aucune. Précédent du dépôt : un CHECK ne se modifie pas en place sous
--     MariaDB (`20261001000004_invoice_settlements_write_off.sql`) ;
--  3. l'ancienne contrainte de statut retirée ;
--  4. `imported_supplier_invoices.link_origin` (C-v017-4-29), NULLABLE et sans
--     défaut, non breaking pour elle-même : NULL = lien antérieur à la 0.17.0 ;
--     aucun backfill (on ne sait pas, en base, si un lien vient d'une
--     complétion ou d'un rattachement). Une sauvegarde 0.16.x ou rc1, qui n'a
--     pas la colonne, s'importe (colonne non requise) ;
--  5. le seuil, en dernière instruction.
--
-- Aucune écriture d'une table applicative. P5 : ligne à
-- docs/migrations-idempotence-audit.md. P7 : entrée EXEMPT_MIGRATIONS (seule
-- écriture : `_kesh_version`, table système jamais restaurée).
ALTER TABLE supplier_invoices MODIFY COLUMN purchase_journal_entry_id BIGINT NULL;

ALTER TABLE supplier_invoices
    ADD CONSTRAINT chk_supplier_invoices_status_v2
        CHECK (status IN ('draft', 'open', 'paid', 'cancelled')),
    ADD CONSTRAINT chk_supplier_invoices_entry_iff_not_draft
        CHECK ((status = 'draft') = (purchase_journal_entry_id IS NULL)),
    ADD CONSTRAINT chk_supplier_invoices_draft_without_settlement
        CHECK (status <> 'draft' OR (settlement_type IS NULL
            AND settlement_bank_account_id IS NULL AND settlement_account_id IS NULL
            AND settlement_journal_entry_id IS NULL AND paid_at IS NULL));

ALTER TABLE supplier_invoices DROP CONSTRAINT chk_supplier_invoices_status;

ALTER TABLE imported_supplier_invoices
    ADD COLUMN link_origin VARCHAR(16) NULL,
    ADD CONSTRAINT chk_imported_si_link_origin
        CHECK (link_origin IS NULL OR link_origin IN ('completion', 'attachment'));

UPDATE _kesh_version SET kesh_version_min_required = '0.17.0' WHERE id = 1;
