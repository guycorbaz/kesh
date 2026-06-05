-- Story 17-2a (DC5) — Extension audit_log : distinguer un acteur humain (UI
-- web, JWT) d'une mutation via clé API externe (PAT).
--
-- Non-breaking migration (P1/P3 CLAUDE.md) :
--   - `actor_type` : ADD COLUMN NOT NULL DEFAULT 'user' → les lignes
--     existantes prennent 'user' (sémantique historique préservée).
--   - `actor_api_key_id` : ADD COLUMN nullable → NULL pour l'existant.
-- Les anciens binaires ignorent les deux colonnes. Pas de bump
-- `kesh_version_min_required`.
--
-- `user_id` reste NOT NULL FK users(id) ON DELETE RESTRICT : tout audit a un
-- acteur user, même via PAT (user_id = créateur de la clé). Ne PAS le passer
-- nullable.
--
-- Pas de FK sur `actor_api_key_id` : pointeur logique (cohérent `entity_id`).
-- La clé peut être révoquée/supprimée alors que l'audit doit survivre 10 ans
-- (CO art. 957-964).
--
-- Dialecte MariaDB : ADD COLUMN (pas `ALTER COLUMN TYPE`, syntaxe PostgreSQL
-- non supportée).

ALTER TABLE audit_log
    ADD COLUMN actor_type ENUM('user', 'api_key') NOT NULL DEFAULT 'user' COMMENT 'Story 17-2a — user (UI/JWT) ou api_key (PAT)',
    ADD COLUMN actor_api_key_id BIGINT NULL COMMENT 'Story 17-2a — id clé API si actor_type=api_key (pointeur logique, pas de FK)';
