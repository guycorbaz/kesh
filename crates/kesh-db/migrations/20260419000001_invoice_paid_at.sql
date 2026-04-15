-- Story 5.4 — Échéancier factures
--
-- Ajoute le marqueur manuel de paiement sur les factures validées.
-- La réconciliation automatique (Epic 6) posera également ce champ.
--
-- CHECK `paid_at IS NULL OR status IN ('validated','cancelled')` : défense en
-- profondeur contre un bug applicatif marquant payée une facture draft.
-- Autorise explicitement le status 'cancelled' pour permettre à une story
-- future de l'annuler après paiement (ex. remboursement).
-- Enforcée à partir de MariaDB 10.2 — projet Kesh cible 10.11+.
--
-- Deux index composites :
--   * `(company_id, status, paid_at)` pour filtrer rapidement impayées/payées.
--   * `(company_id, status, due_date)` pour tri par échéance + WHERE overdue.
--
-- `IF NOT EXISTS` partout (MariaDB ≥ 10.3) rend la migration ré-entrante en
-- cas de crash partiel entre ALTER TABLE (chacun provoque un commit implicite).

ALTER TABLE invoices
    ADD COLUMN IF NOT EXISTS paid_at DATETIME(3) NULL;

ALTER TABLE invoices
    ADD CONSTRAINT IF NOT EXISTS chk_invoices_paid_at_validated
    CHECK (paid_at IS NULL OR status IN ('validated', 'cancelled'));

-- Tolérance 1 jour alignée exactement avec la garde applicative
-- `mark_as_paid` (cf. invoices.rs) : `paid_at.date() >= invoice.date - 1 jour`.
-- La comparaison DATETIME vs DATE coerce la DATE à minuit, d'où la nécessité
-- de comparer sur `DATE(paid_at)` pour des bornes strictement cohérentes
-- (sinon un paiement à 23:59 le jour J-1 passe la DB mais la Rust rejette).
ALTER TABLE invoices
    ADD CONSTRAINT IF NOT EXISTS chk_invoices_paid_at_after_date
    CHECK (paid_at IS NULL OR DATE(paid_at) >= date - INTERVAL 1 DAY);

CREATE INDEX IF NOT EXISTS idx_invoices_payment_status
    ON invoices (company_id, status, paid_at);

CREATE INDEX IF NOT EXISTS idx_invoices_due_date
    ON invoices (company_id, status, due_date);
