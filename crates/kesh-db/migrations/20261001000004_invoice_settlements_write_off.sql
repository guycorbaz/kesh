-- Story 25-4-d2a (#384, #490) — le SOLDE du reste d'une facture, enregistré
-- comme une ligne de `invoice_settlements` de type `write_off`.
--
-- Arbitrages du Project Lead (2026-10-01) : solder ce qui reste dû sur une
-- facture validée en choisissant la NATURE de l'écart — escompte accordé, frais
-- bancaires retenus par la banque du client, perte sur débiteur, reste
-- d'arrondi (#490). Une ligne de cette table, parce que le reste dû la
-- retranche déjà (INVOICE_AMOUNT_DUE_DERIVED_SQL) : l'échéancier, la balance
-- âgée, le rapprochement, `paid_at` et l'annulation la suivent sans
-- modification.
--
-- `write_off_vat` fige la ventilation de la TVA corrigée par taux (escompte et
-- perte), pour le rapport TVA (Story 25-4-d2c) ; `[]` sans TVA. Une COLONNE et
-- non une table : une table neuve rendrait toutes les sauvegardes antérieures
-- inimportables (l'inventaire des tables d'un backup doit être identique).
--
-- MariaDB : un CHECK ne se modifie pas en place → DROP CONSTRAINT + ADD
-- CONSTRAINT, chacun dans son instruction (précédent :
-- `20260714000002_email_templates_reminder.sql`).
--
-- DDL seul, AUCUNE donnée écrite (triage P7 sans objet). NON BREAKING : un
-- binaire antérieur lit `settlement_type` en chaîne (jamais en énumération
-- stricte), liste ses colonnes (il ignore les deux neuves), contre-passe un
-- solde comme un règlement (l'annulation ne regarde pas le type), et
-- `check_schema_compat` n'exige d'une sauvegarde que les colonnes NOT NULL sans
-- défaut. `DROP CONSTRAINT` n'est pas une opération listée en P3 → PAS de bump
-- `kesh_version_min_required`. Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (politique P5).

ALTER TABLE invoice_settlements
    ADD COLUMN write_off_nature VARCHAR(20) NULL,
    ADD COLUMN write_off_vat JSON NULL;

ALTER TABLE invoice_settlements
    DROP CONSTRAINT chk_invoice_settlements_type;

ALTER TABLE invoice_settlements
    ADD CONSTRAINT chk_invoice_settlements_type
        CHECK (settlement_type IN ('bank_transfer', 'internal_account', 'write_off'));

ALTER TABLE invoice_settlements
    DROP CONSTRAINT chk_invoice_settlements_counterparty;

ALTER TABLE invoice_settlements
    ADD CONSTRAINT chk_invoice_settlements_counterparty
        CHECK ((settlement_type = 'bank_transfer'
                AND settlement_bank_account_id IS NOT NULL
                AND settlement_account_id IS NULL)
            OR (settlement_type = 'internal_account'
                AND settlement_account_id IS NOT NULL
                AND settlement_bank_account_id IS NULL)
            OR (settlement_type = 'write_off'
                AND settlement_account_id IS NOT NULL
                AND settlement_bank_account_id IS NULL));

-- La nature et la ventilation existent SSI la ligne est un solde.
ALTER TABLE invoice_settlements
    ADD CONSTRAINT chk_invoice_settlements_write_off_nature
        CHECK ((settlement_type = 'write_off') = (write_off_nature IS NOT NULL)
           AND (settlement_type = 'write_off') = (write_off_vat IS NOT NULL)
           AND (write_off_nature IS NULL
                OR write_off_nature IN ('discount', 'bank_fees', 'bad_debt', 'rounding')));
