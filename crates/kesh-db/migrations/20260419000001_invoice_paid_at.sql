-- Story 5.4 — Échéancier factures
--
-- Ajoute le marqueur manuel de paiement sur les factures validées.
-- La réconciliation automatique (Epic 6) posera également ce champ.
--
-- CHECK `paid_at IS NULL OR status = 'validated'` : défense en profondeur
-- contre un bug applicatif marquant payée une facture draft/cancelled.
-- Enforcée à partir de MariaDB 10.2 — projet Kesh cible 10.11+.
--
-- Deux index composites :
--   * `(company_id, status, paid_at)` pour filtrer rapidement impayées/payées.
--   * `(company_id, status, due_date)` pour tri par échéance + WHERE overdue.

ALTER TABLE invoices
    ADD COLUMN paid_at DATETIME(3) NULL;

ALTER TABLE invoices
    ADD CONSTRAINT chk_invoices_paid_at_validated
    CHECK (paid_at IS NULL OR status = 'validated');

-- P4 (review pass 1) : défense en profondeur contre une régression chronologique.
-- Tolérance 1 jour pour absorber l'écart UTC naïf ↔ date métier CET/CEST
-- (voir P2 dans repositories/invoices.rs mark_as_paid).
ALTER TABLE invoices
    ADD CONSTRAINT chk_invoices_paid_at_after_date
    CHECK (paid_at IS NULL OR paid_at >= date - INTERVAL 1 DAY);

CREATE INDEX idx_invoices_payment_status
    ON invoices (company_id, status, paid_at);

CREATE INDEX idx_invoices_due_date
    ON invoices (company_id, status, due_date);
