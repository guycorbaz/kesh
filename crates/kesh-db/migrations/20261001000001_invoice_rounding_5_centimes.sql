-- Story 25-4-c4-a (#494) — l'arrondi à 5 centimes du total des pièces émises.
--
-- Arbitrage du Project Lead (2026-10-01) : le total TTC d'une facture émise
-- est arrondi au multiple de 0.05 le plus proche (123.44 → 123.45, 234.52 →
-- 234.50), l'écart sans TVA, écrit à la validation sur le compte de
-- différences d'arrondi. Les avoirs suivent la même règle. Réglage par
-- société, actif par défaut.
--
-- L'écart est une colonne d'EN-TÊTE, figée à la validation : une ligne de
-- facture ne peut pas être négative (`chk_invoice_lines_line_total_non_negative`)
-- et le rapport TVA la compterait. `DEFAULT 0` décrit exactement les pièces
-- antérieures, émises sans arrondi.
--
-- DDL seul, AUCUNE donnée écrite (triage P7 sans objet). Non-breaking
-- (`ADD COLUMN` avec défaut, ignoré par un binaire antérieur qui liste ses
-- colonnes) → PAS de bump `kesh_version_min_required`. Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (politique P5).

ALTER TABLE invoices
    ADD COLUMN rounding_amount DECIMAL(19,4) NOT NULL DEFAULT 0;

ALTER TABLE credit_notes
    ADD COLUMN rounding_amount DECIMAL(19,4) NOT NULL DEFAULT 0;

ALTER TABLE company_invoice_settings
    ADD COLUMN round_to_5_centimes BOOLEAN NOT NULL DEFAULT TRUE;
