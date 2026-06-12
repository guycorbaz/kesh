-- Story 17-4a — Colonne `email` sur `users` (recovery self-service #122).
--
-- Non-breaking migration (P1/P3 CLAUDE.md) : ADD COLUMN nullable — les
-- anciens binaires l'ignorent. Pas de bump `kesh_version_min_required`.
--
-- Nullable + NON-unique : multi-tenant, deux users de companies distinctes
-- peuvent partager un email (DC6). Les comptes existants → email NULL
-- (non-recouvrables par email → fallback break-glass #121).
--
-- Index non-unique pour le lookup recovery `find_by_email` (DC6).
--
-- `IF NOT EXISTS` partout (MariaDB ≥ 10.3) rend la migration ré-entrante.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS email VARCHAR(255) NULL;

CREATE INDEX IF NOT EXISTS idx_users_email
    ON users (email);
