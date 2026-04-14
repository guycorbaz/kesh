-- Story 5.1 code review follow-up : défense en profondeur sur `line_total ≥ 0`.
-- Déjà garanti par l'application (`line_total = quantity × unit_price` avec
-- `quantity > 0` et `unit_price ≥ 0`), mais empêche un INSERT SQL direct
-- de contourner l'invariant métier.

ALTER TABLE invoice_lines
    ADD CONSTRAINT chk_invoice_lines_line_total_non_negative CHECK (line_total >= 0);
