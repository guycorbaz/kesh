# Squash du schéma de test — Story 22-5 (#251)

`0001_schema_squash.sql` est l'**unique** migration que rejouent les bases
éphémères de `#[sqlx::test]` : un batch DDL au lieu des 61 cycles
INSERT/DDL/UPDATE du vrai `MIGRATOR`. Les tests le visent par
`migrations = "./test-schema"` (dans `kesh-db`) ou `"../kesh-db/test-schema"`
(dans `kesh-api`, `kesh-report`).

**Il SE RÉGÉNÈRE, il ne s'édite JAMAIS** : `scripts/regen-test-schema.sh`.
Toute migration ajoutée sans régénération fait rougir
`crates/kesh-db/tests/test_schema_guard.rs`, qui compare à chaque gate le
schéma monté par ce squash à celui monté par le vrai `MIGRATOR` — structure,
ligne d'installation `_kesh_version`, et suivi `_sqlx_migrations`.

**Le garde-fou P8 ne s'applique pas ici** : ce fichier ne vit que dans des bases
**éphémères**, que sqlx détruit et recrée à neuf (`drop database if exists`
avant chaque `create`). Aucun checksum persistant ne le rencontre jamais — le
modifier ne peut donc pas empêcher un binaire de démarrer, contrairement aux
migrations de `crates/kesh-db/migrations/`, qui elles sont intouchables une fois
appliquées.

Les tests qui exercent **le chemin des migrations lui-même** (installation
fraîche, fenêtre d'upgrade, backfills à fenêtre, triage P7) restent sur le vrai
`MIGRATOR` : la liste est portée en dur par `test_schema_guard.rs`, et tout
fichier qui en sortirait sans y être inscrit rougit.
