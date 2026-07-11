-- Story 20-3b1 : e-mail de contact de la société (refinement epic-20).
--
-- Sert de `Reply-To` aux e-mails métier (décision #2 epic-20 : From =
-- KESH_SMTP_FROM avec display-name société + Reply-To = e-mail société).
-- NULL = Reply-To omis. La saisie UI arrive en 20-3b2.
--
-- Non-breaking (ADD COLUMN nullable) → pas de bump `kesh_version_min_required`.
ALTER TABLE companies
    ADD COLUMN email VARCHAR(320) NULL;
