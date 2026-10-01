-- Story 25-4-e (#495) — un montant minimum configurable sous lequel une facture
-- n'est pas émise.
--
-- Arbitrage du Project Lead (2026-10-01) : le seuil est un RÉGLAGE par société.
-- `NULL` = aucun seuil — le défaut : la règle ne s'applique qu'une fois un
-- montant fixé. Comparé, à la validation, au total TTC ARRONDI (25-4-c4-a).
--
-- DDL seul, AUCUNE donnée écrite (triage P7 sans objet). Non-breaking
-- (`ADD COLUMN` nullable, ignoré par un binaire antérieur qui liste ses
-- colonnes) → PAS de bump `kesh_version_min_required`. Ligne ajoutée à
-- `docs/migrations-idempotence-audit.md` (politique P5).

ALTER TABLE company_invoice_settings
    ADD COLUMN minimum_invoice_amount DECIMAL(19,4) NULL;
