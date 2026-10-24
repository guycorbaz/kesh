# Prompt — validation P1, Story 32-12 (une sauvegarde antérieure reste importable)

*Versionné le 2026-10-24. **Deux lentilles, R et F**, chacune en contexte frais, **Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-12`, branche `story/32-12`. Fiche :
`_bmad-output/implementation-artifacts/32-12-import-sauvegarde-anterieure.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3212.md` ; fichier d'epic
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-12*, § *Ce que le code dit déjà* point 4, Q4). Règles :
`CLAUDE.md` (§ *Review Iteration Rule*, *Règle de splitting préventif*, *Exception kesh-db*, *Migration breaking policy*
**P1, P2-bis, P7**, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*),
`docs/agents-orchestration.md` § 3 et § 6. Issue : `gh issue view 722` (lecture seule).

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Inventaire actuel.** `parse_and_verify` (`crates/kesh-api/src/admin_backup/import.rs`, l. 172 et suivantes)
   compare-t-il bien l'inventaire dans les deux sens, et nulle part ailleurs ? Un autre site de l'import, de l'export,
   de la sauvegarde automatique (`crates/kesh-api/src/auto_backup*`), du handler (`routes/admin.rs`) ou de
   `kesh-db` suppose-t-il que toutes les tables de `TABLES_TO_TRUNCATE` sont dans `parsed.tables` (indexation directe
   `parsed.tables[..]`, `.unwrap()` sur `get`, comptages) ? Inventaire **clos** des sites qui lisent `parsed.tables` ou
   `TABLES_TO_TRUNCATE` (`grep -rn "TABLES_TO_TRUNCATE\|\.tables\.get\|\.tables\[" crates/`).
2. **Point 1 — la carte table → migration.** La reconnaissance `creates_application_table`/`normalize`
   (`crates/kesh-db/src/post_restore.rs`) suffit-elle à trouver **une** migration créatrice pour **chacune** des 39
   tables de `TABLES_TO_TRUNCATE` ? Cherche une table créée par `CREATE TABLE … LIKE`, `RENAME TABLE`, deux fois, ou
   avec une graphie que la reconnaissance rate (`grep -rniE "create +table|rename +table" crates/kesh-db/migrations/`).
   `20260917000001` est-elle bien la dernière migration créatrice de table **applicative**, et à la v0.15.0 ?
3. **Point 2 — les exemptions.** Revérifie une à une les huit exemptions « Hors fenêtre » et les fondements non
   marqués (`20260917000001`, les deux `PerishableSince`) : la fiche a-t-elle raison de dire qu'**aucune** ne devient
   fausse avec le plancher ? Le compte de huit tient-il (`assert_eq!(checked, 8)`) ? L'analyse des deux tests de
   fenêtre (ce qu'ils feraient à M1 après la 32-5) est-elle exacte ? `scripts/prepare-release.sh` (lecture seule,
   **ne pas l'exécuter**) dépend-il de la fenêtre ou de l'inventaire ?
4. **Point 3 — l'état après import.** `restore_body` (`crates/kesh-db/src/backup.rs`) vide-t-il vraiment une table
   absente de la carte ? Le rejeu (`replay_post_restore_backfills`, `missing_sentinels`), le backfill du numéro de
   client (`kesh_db::backfill`), `repair_installation_in_tx`, `force_onboarding_done_if_eligible` et le contrôle des
   pièces supportent-ils une table absente ? Les tables système (`_kesh_backup_schedule`) : exclues par nom ou par
   préfixe — la fiche dit juste ?
5. **Point 4 — le bilan.** Les champs de la réponse et de l'audit cités existent-ils aux lignes dites ? `parsed` est-il
   consommé avant l'endroit où la fiche veut lire `tables_absent` ? Les deux écrans (`/admin/restore`,
   `/onboarding/restore`) emploient-ils le même composant ?
6. **Point 5 — la fixture.** Le générateur 0.14.0 (`admin_full_export_e2e.rs`) et la fixture existante sont-ils tels
   que décrits ? Le code de la branche est-il bien celui de la v0.15.0 (`git diff --stat v0.15.0 HEAD -- crates/
   frontend/src`) ? La fixture produite aurait-elle bien deux pièces sous `files/` et 39 tables ?
7. **Manuel.** `docs/manual/fr/admin-manual.tex` § *Importer* (l. 2242 et suivantes), liste des refus, § *Ce que dit
   l'écran*, § *Reprises de données rejouées à l'import* : les retouches prescrites (AC20) sont-elles exactes après
   correctif, et une phrase **ailleurs** lie-t-elle l'importabilité à un inventaire identique ou à « la même version »
   (manuel admin, manuel utilisateur, `docs/api-external.md`, README, `website/`) ? Contrôle le **PDF aplati** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiF "même version"` (et
   « restaurer » autour de « version »).
8. **Tests et mutations.** Chaque test prescrit rougit-il avant le correctif pour la bonne raison ? Chaque mutation M1-M8
   est-elle tuée par un test nommé, ou honnêtement déclarée équivalente ? Un test qui passerait à vide (AC11 vacant,
   AC2 (iv) liste vide) — la fiche le dit-elle ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chacun des cinq points « Ce que le code établit » est-il établi au code, ou supposé ? En
   particulier : la version Cargo reste-t-elle à `0.15.0` après une migration non breaking (P2-bis) — l'argument
   contre la version déclarée (C-32-12-3) tient-il ?
2. **La règle** : le plancher et la règle du suffixe (C-32-12-2, C-32-12-4) ferment-ils bien les deux cas qu'ils
   visent (sauvegarde trop ancienne ; ordre impossible) ? Un cas légitime qu'ils refuseraient (sauvegarde 0.16.0
   importée en 0.16.0 ; sauvegarde 0.14.0, format 1) ? Un cas illégitime qu'ils accepteraient et que la fiche ne
   nomme pas ?
3. **Périmètre et découpage** : les cinq modules sont-ils bien cinq ? Une migration est-elle nécessaire, en réalité ?
   `kesh-api` peut-il appeler une fonction de `kesh-db` qui lit `MIGRATOR` sans dépendance nouvelle ? La porte
   `parse_and_verify_with` est-elle compatible avec `spawn_blocking` et le type générique `R` ?
4. **Faisabilité des tests** : l'AC7 peut-il enchaîner `parse_and_verify_with`, `check_schema_compat`,
   `restore_tables_in_tx` et `replay_post_restore_backfills` hors handler (visibilité `pub`, transaction) ? L'AC4 peut-il
   tourner sur `test-schema` ? L'AC11 a-t-il une matière, même par M7 ?
5. **Décomptes** : recompte chaque nombre de la fiche et du registre (39 tables, 83 migrations, 8 exemptions « Hors
   fenêtre », 20 exemptions au total s'il est cité, 12 490 octets, lignes citées) ; signale celui qui ne se retrouve pas.
6. **Cohérence interne** : AC1-AC21, T0-T7, registre C-32-12-1 à 14, forme réduite de l'encadré de tête — contradiction,
   critère invérifiable, commande de contrôle mal formée (essaie mentalement chaque `grep` de l'AC17 contre le code
   actuel) ?
7. **Ce que la fiche omet** : un site, un test existant qui rougira sans être nommé (un test qui attend
   l'égalité d'inventaire, le compte de tables du manifeste, la forme exacte de la réponse d'import — y compris côté
   Vitest et Playwright), une conséquence pour la recette visuelle, le CHANGELOG, la 32-5.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (R1 à R8, F1 à F7) — sans elle,
« 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un
défaut. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p1-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p1-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire ; aucune redirection d'écriture (`>`, `tee`)
ailleurs que vers ton rapport. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `unzip` vers le disque (`unzip -l`
et `unzip -p` vers la sortie standard seulement), `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
`gh` en écriture. `pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame|ls-tree|tag --contains`,
`gh issue view`, `pdftotext … -`, `unzip -l|-p`.
