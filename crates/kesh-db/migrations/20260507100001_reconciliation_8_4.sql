-- Story 8-4 — réconciliation matching automatique (FR44).
--
-- 1. auto_match_rejected_at : tx.status reste 'pending' mais marquée
--    comme « manuellement revue sans match auto » (cf. §reject-flow).
--    Le filtre `find_pending_transactions_for_account` exclut les rows
--    avec auto_match_rejected_at IS NOT NULL — elles n'apparaissent
--    plus dans GET /proposals (réservées pour 8-5 manual).
--
-- M8 (Pass 1 review) — ALGORITHM=INSTANT évite la copie complète de
-- la table sur MariaDB 10.3+ (instant ADD COLUMN nullable). Sans
-- cette directive, ALTER TABLE bloque les writes pendant minutes/heures
-- sur grandes tables production. LOCK=NONE garantit la concurrent
-- DML pendant la migration.
ALTER TABLE bank_transactions
    ADD COLUMN auto_match_rejected_at DATETIME(3) NULL AFTER matched_entry_id,
    ALGORITHM=INSTANT, LOCK=NONE;

-- 2. Index pour find_unpaid_invoices_for_window — couvre les colonnes
--    filtrées par la query du repo (company_id, status, paid_at, date).
--    M1 Pass 1 patch : le filtre `journal_entry_id IS NOT NULL` est
--    ajouté au repo, mais l'index reste sur (company_id, status, paid_at,
--    date) pour couvrir le cas nominal sans bloating.
CREATE INDEX IF NOT EXISTS idx_invoices_company_validated_unpaid_date
    ON invoices (company_id, status, paid_at, date);
