-- Story 25-1a (Epic 25, refs #376) — la piste de contrôle survit à un import.
--
-- LE DÉFAUT. L'import d'une sauvegarde `DELETE`ait puis réinsérait `audit_log`
-- comme n'importe quelle table applicative : la piste de contrôle de l'instance
-- était **intégralement remplacée** par celle de l'archive. Un utilisateur pouvait
-- donc effacer ses traces en important un backup — le scénario de BLANCHIMENT que
-- la Story 25-1a ferme. L'audit du 2026-08-26 le relève en III.3.
--
-- CE QUE CETTE MIGRATION PRÉPARE, et ce qu'elle ne fait pas. Elle ne change aucun
-- comportement à elle seule : elle rend POSSIBLE la conservation de la piste
-- locale au restore, qui est implémentée dans `backup.rs`. Deux verrous s'y
-- opposaient :
--
--   1. `user_id` porte une FK vers `users(id)`, et le restore REMPLACE `users`.
--      Une entrée conservée pointerait vers un identifiant disparu — ou, pire,
--      RÉATTRIBUÉ À QUELQU'UN D'AUTRE, les espaces d'identifiants de deux
--      instances se recouvrant. La piste mentirait alors silencieusement sur qui
--      a fait quoi : le défaut même qu'elle existe pour ne pas avoir.
--   2. Rien ne portait le NOM de l'acteur. Sans lui, retirer la FK ne fait que
--      transformer un mensonge en trou.
--
-- LE PATRON N'EST PAS INVENTÉ ICI. `actor_api_key_id` (20260605000002) et
-- `entity_id` (20260413000001) sont déjà des **pointeurs logiques sans FK**, au
-- motif écrit que « l'audit survit 10 ans à la révocation/suppression de la clé ».
-- `user_id` rejoint cette famille, et `actor_label` joue pour lui le rôle que
-- `entity_type` joue pour `entity_id` : nommer ce que le pointeur ne garantit plus.
--
-- ⚠️ `actor_label` est un INSTANTANÉ, pas une jointure. Il porte le `username` **au
-- moment de l'écriture** et ne suit pas les renommages ultérieurs — c'est
-- délibéré : une piste de contrôle doit dire qui a agi SOUS QUEL NOM à l'instant
-- de l'acte, non sous quel nom cette personne s'appelle aujourd'hui.
--
-- P1/P3 : NON BREAKING. `ADD COLUMN` avec défaut + `DROP FOREIGN KEY` — un binaire
-- antérieur ignore la colonne et continue d'insérer des lignes valides (il ne
-- dépend pas de la FK pour écrire). ⇒ ni bump `kesh_version_min_required`, ni bump
-- de version Cargo (P2/P2-bis).
--
-- P7 : le backfill ci-dessous ÉCRIT DES DONNÉES ⇒ triage obligatoire. Il est de
-- CLASSE B — sentinelle `(audit_log, actor_label)` —, valide parce que la colonne
-- est ajoutée par le MÊME fichier que l'UPDATE qui la remplit.
--
-- IDEMPOTENCE : `yes`. Le backfill est gardé par `actor_label = ''`, donc une
-- re-exécution manuelle hors sqlx est sans effet sur les lignes déjà remplies.

ALTER TABLE audit_log
    ADD COLUMN actor_label VARCHAR(64) NOT NULL DEFAULT '' COMMENT
        'Story 25-1a — nom de l''acteur AU MOMENT de l''écriture (instantané, pas une jointure). Survit au remplacement de users par un import de sauvegarde.',
    DROP FOREIGN KEY fk_audit_log_user;

-- Backfill : le nom actuel est la meilleure approximation disponible pour les
-- entrées antérieures. `'(inconnu)'` pour les acteurs déjà disparus — impossible
-- aujourd'hui (la FK RESTRICT l'interdisait), mais la garde coûte une ligne et
-- ferme le cas où cette migration serait rejouée après un import.
UPDATE audit_log a
    LEFT JOIN users u ON u.id = a.user_id
   SET a.actor_label = COALESCE(u.username, '(inconnu)')
 WHERE a.actor_label = '';
