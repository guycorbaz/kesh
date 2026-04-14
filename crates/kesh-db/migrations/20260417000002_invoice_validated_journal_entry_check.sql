-- Story 5.2 — Review pass 2 Q3 (defense-in-depth).
--
-- Garantit qu'une facture `validated` a toujours `journal_entry_id NOT NULL`.
-- Protège contre un bug futur dans `validate_invoice` qui oublierait la
-- liaison, ou une corruption DB manuelle.
--
-- L'inverse n'est pas forcé : une facture `draft` peut avoir
-- `journal_entry_id = NULL` (cas nominal avant validation) ou exceptionnellement
-- non-null (aucun chemin applicatif actuel, mais autorisé par le CHECK).

ALTER TABLE invoices
    ADD CONSTRAINT chk_invoices_validated_has_je
    CHECK (status <> 'validated' OR journal_entry_id IS NOT NULL);
