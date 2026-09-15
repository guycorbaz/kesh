-- Story 25-1c-zero (Epic 25, refs #378) — chaque entrée d'audit porte la société de son acteur.
--
-- LE DÉFAUT. `audit_log` est GLOBALE dans une application multi-société : aucune colonne ne dit
-- à quelle société appartient une trace. La consultation que livrera la Story 25-1c (#378)
-- exposerait donc les traces de TOUTES les sociétés à l'administrateur d'une seule. Des trois
-- issues possibles — ajouter la colonne, restreindre la route, documenter la limite — le Project
-- Lead a retenu la colonne (arbitrage du 2026-09-11).
--
-- CE QUE CETTE MIGRATION FAIT, et ce qu'elle ne fait pas. Elle ajoute la colonne et son index, et
-- remplit les entrées existantes. Elle ne change aucun comportement visible : l'alimentation des
-- entrées futures vit dans `repositories/audit_log.rs` (sous-SELECT dans `insert_in_tx`), et la
-- consultation par société appartient à la 25-1c.
--
-- LE PATRON N'EST PAS INVENTÉ ICI. `company_id` rejoint la famille des POINTEURS LOGIQUES SANS FK
-- d'`audit_log` — `entity_id` (20260413000001), `actor_api_key_id` (20260605000002), `user_id`
-- (20260910000001) — et elle est remplie comme `actor_label` : l'instantané de ce que la base porte
-- au moment de l'écriture.
--
-- TROIS CONTRAINTES, chacune adossée à un mode d'échec du dépôt :
--
--   1. NULL, JAMAIS `NOT NULL` SANS DÉFAUT. `check_schema_compat` (kesh-api,
--      `admin_backup/import.rs`) rejette en 400 tout backup dont la source ne porte pas une colonne
--      destination `NOT NULL` sans défaut : une colonne mal déclarée rendrait inimportables toutes
--      les sauvegardes existantes.
--   2. AUCUNE CLÉ ÉTRANGÈRE VERS `companies`. Le restore REMPLACE `companies`, alors que les
--      entrées d'audit locales sont CONSERVÉES et celles de l'archive fusionnées (Story 25-1a) :
--      c'est le scénario exact qui a fait retirer `fk_audit_log_user`.
--   3. NI `COALESCE`, NI REJEU POST-RESTORE. `NULL` signifie « société indéterminable » — l'acteur
--      n'existe pas. C'est un état LÉGITIME ET PERMANENT, non un trou à combler, et `NULL` n'est
--      pas une sentinelle : un rejeu gardé `IS NULL` ne distinguerait pas une entrée d'archive d'une
--      entrée locale, dont le `user_id` désigne, après un restore, le porteur de cet identifiant
--      dans l'instance SOURCE.
--
-- L'INDEX `(company_id, created_at)` sert la consultation par société et par période de la 25-1c.
-- Il est posé ici pour ne pas écrire une seconde migration — et réarmer P5 à P8 — pour une ligne.
--
-- P1/P3 : NON BREAKING. `ADD COLUMN` nullable et `ADD INDEX` : un binaire antérieur ignore la
-- colonne et continue d'insérer des lignes valides. ⇒ ni bump `kesh_version_min_required`, ni bump
-- de version Cargo (P2/P2-bis).
--
-- P7 : le backfill ci-dessous ÉCRIT DES DONNÉES ⇒ triage obligatoire. Il est EXEMPTÉ du rejeu
-- post-restore (`EXEMPT_MIGRATIONS`, `ExemptionBasis::PerishableSince(20260827000001)`), sur un
-- fondement de PARC et non de fenêtre : aucune version publiée ne se situe dans
-- [20260827000001 .. cette migration). Un backup pris dans cet intervalle — build de développement
-- non distribué — fusionne ses entrées avec `company_id = NULL`. Ce fondement SE PÉRIME si une
-- version est taguée depuis `main` avant le merge de cette migration ; `scripts/prepare-release.sh`
-- le contrôle.
--
-- IDEMPOTENCE : `yes`. Le DDL porte `IF NOT EXISTS` (rejoué, il rend les notes 1060 et 1061 sans
-- erreur) et le backfill est gardé par `company_id IS NULL` : une re-exécution manuelle hors sqlx
-- ne touche rien de ce qui est déjà rempli.

ALTER TABLE audit_log
    ADD COLUMN IF NOT EXISTS company_id BIGINT NULL COMMENT
        'Story 25-1c-zero — société de l''acteur AU MOMENT de l''écriture. Pointeur logique, sans FK (companies est remplacée au restore). NULL = société indéterminable, état permanent.',
    ADD INDEX IF NOT EXISTS idx_audit_log_company_date (company_id, created_at);

-- Backfill : la société actuelle de l'acteur. `JOIN` et non `LEFT JOIN`, sans `COALESCE` : une
-- entrée dont l'acteur n'existe plus garde `NULL` — possible depuis que `user_id` n'a plus de FK.
UPDATE audit_log a
  JOIN users u ON u.id = a.user_id
   SET a.company_id = u.company_id
 WHERE a.company_id IS NULL;
