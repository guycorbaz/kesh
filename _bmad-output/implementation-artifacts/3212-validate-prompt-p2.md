# Prompt — validation P2, Story 32-12 (une sauvegarde antérieure reste importable)

*Versionné le 2026-10-24. Passe **complète**, **deux lentilles, R et F**, chacune en contexte frais, **Opus**. Lecture
seule stricte. Elle suit la P1 (Sonnet ; R 0/0/2/6, F 0/1/2/5), remédiée au commit `544c2821`.*

Dépôt `/home/gcorbaz/devel/kesh-32-12`, branche `story/32-12`. Fiche :
`_bmad-output/implementation-artifacts/32-12-import-sauvegarde-anterieure.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3212.md` (C-32-12-1 à 21) ; fichier d'epic
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-12*, § *Ce que le code dit déjà* point 4, Q4) ; rapports
de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p1-R.md` et `3212-validate-p1-F.md`. Règles : `CLAUDE.md`
(§ *Review Iteration Rule*, *La passe ciblée*, *Règle de splitting préventif*, *Exception kesh-db*, *Migration breaking
policy* **P1, P2-bis, P7, P8**, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*,
*Propagation post-patch*, *Recompter ses propres comptes rendus*, *Recette visuelle*), `docs/agents-orchestration.md`
§ 3 et § 6. Issue : `gh issue view 722` (lecture seule).

## Attention particulière : la remédiation de la P1

La sévérité se déplace vers ce qu'on vient d'écrire. Lis `git show 544c2821` **en entier** avant le reste, et pour
chaque hunk demande-toi ce qu'il a cassé ailleurs dans la fiche ou le registre :

- **AC7 déplacé dans la crate** (`crates/kesh-api/src/admin_backup/import.rs`, `mod tests`, `#[sqlx::test(migrations =
  "../kesh-db/test-schema")]`, `include_bytes!("../../tests/fixtures/keshbackup-0.15.0-format2.keshbackup")`, retrait
  de `products` par une fonction locale ; porte `pub(crate)` ; `lib.rs` inchangé, C-32-12-15). Le chemin relatif de
  `include_bytes!` est-il juste depuis ce fichier ? Les fonctions que l'AC7 enchaîne (`check_schema_compat`,
  `kesh_db::backup::restore_tables_in_tx`, `kesh_db::post_restore::replay_post_restore_backfills`) sont-elles
  appelables depuis ce `mod tests`, avec quelle connexion ou transaction ? Le précédent `auth/bootstrap.rs:482` vaut-il
  pour `import.rs` (dépendances de dev, `sqlx` en `dev-dependencies` avec la feature voulue) ? Un format 2 dont on
  retire `data/products.ndjson` et `tables.products` reste-t-il cohérent avec l'inventaire des noms d'entrées de
  `verifier_format_2` et avec la cohérence des pièces ? La fixture 0.14.0 porte `products` à 0 ligne : la fiche
  affirme-t-elle sans le savoir que la 0.15.0 fera de même ?
- **AC2 (iii) en union et disjonction** (C-32-12-16) : la mutation M3 est-elle encore tuée, et par quoi exactement ?
  « tolérées » désigne-t-il partout la même chose (liste écrite de (iv) ou ensemble calculé) ?
- **Motif ciblé de l'AC17** (C-32-12-17) : exécute-le tel qu'écrit, compte (`| wc -l`, la fiche dit 28), et cherche ce
  qu'il **manque encore** — un texte qui devient faux sans porter aucun de ses mots (inventaire des sites non résolus,
  pas énumération des formes). Les trois migrations déclarées intouchables (P8) sont-elles les seules ? La rectification
  au doc-comment d'`IMPORT_FLOOR_MIGRATION` est-elle exécutable et suffisante ?
- **Mutation M9** et son report ; **deux créatrices → la plus ancienne** (C-32-12-18) : cohérent avec l'AC1 (i) et
  avec la carte `OnceLock` ? ; **repli de la fixture** (C-32-12-21) : cohérent avec C-32-12-10, T1, AC15 ; **texte de
  l'encart** (C-32-12-19) : cohérent avec AC20 (iii) et le contrôle du PDF ; **numéros de ligne relevés** : justes ?
- Le **Change Log** « Validation P1 » : ses décomptes et son sort de chaque finding se retrouvent-ils dans la fiche ?

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Inventaire actuel.** `parse_and_verify` (`import.rs:172` et suivantes) compare-t-il l'inventaire dans les deux sens
   (l. 232-250), et nulle part ailleurs ? Inventaire **clos** des sites qui lisent `parsed.tables` ou
   `TABLES_TO_TRUNCATE` (`grep -rn "TABLES_TO_TRUNCATE\|\.tables\.get\|\.tables\[" crates/`).
2. **Point 1 — la carte table → migration.** Une migration créatrice pour **chacune** des 39 tables ; aucun
   `CREATE TABLE … LIKE`, `RENAME TABLE`, double création, graphie ratée ; `20260917000001` dernière créatrice
   applicative à la v0.15.0. Le raisonnement prescrit au doc-comment (R6 de la P1 : `split_statements` suffit) est-il
   vrai ?
3. **Point 2 — les exemptions.** Les huit « Hors fenêtre » et les fondements non marqués restent-ils vrais avec le
   plancher ? `assert_eq!(checked, 8)` ? Analyse des deux tests de fenêtre après la 32-5 ? `scripts/prepare-release.sh`
   (lecture seule, **ne pas l'exécuter**) dépend-il de la fenêtre ?
4. **Point 3 — l'état après import.** `restore_body` vide une table absente ; rejeu, `missing_sentinels`,
   `repair_installation_in_tx`, `force_onboarding_done_if_eligible`, contrôle des pièces ; tables système exclues par nom.
5. **Point 4 — le bilan.** Champs de la réponse et de l'audit aux lignes dites (`admin.rs:262-270`, `581-610`) ;
   `parsed` consommé où ? Même composant pour `/admin/restore` et `/onboarding/restore` ?
6. **Point 5 — la fixture.** Générateur 0.14.0 (`admin_full_export_e2e.rs:534`) et fixture existante tels que décrits ;
   `git diff --stat v0.15.0 HEAD -- crates/ frontend/src` ; deux pièces et 39 tables plausibles ; repli juste ?
7. **Manuel.** `docs/manual/fr/admin-manual.tex` § *Importer* (l. 2242 et suivantes), liste des refus, § *Ce que dit
   l'écran*, § *Reprises de données rejouées à l'import* : les retouches de l'AC20 sont-elles exactes, et une phrase
   **ailleurs** lie-t-elle l'importabilité à un inventaire identique ou à « la même version » (manuel admin, manuel
   utilisateur, `docs/api-external.md`, README, `website/`) ? Contrôle le **PDF aplati** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiF "même version"`.
8. **Tests et mutations.** Chaque test prescrit rougit-il pour la bonne raison ? M1-M9 : tuée par un test nommé, ou
   déclarée équivalente / reportée avec la raison ? Un test qui passerait à vide (AC11 vacant, AC2 (iv) liste vide) — la
   fiche le dit-elle ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : les cinq points « Ce que le code établit » sont-ils établis au code ? L'argument contre la version
   déclarée (C-32-12-3) tient-il (version Cargo inchangée par une migration non breaking, P2-bis) ?
2. **La règle** : plancher et suffixe ferment-ils les deux cas visés ? Un cas légitime refusé (0.16.0 en 0.16.0 ;
   0.14.0 format 1) ? Un cas illégitime accepté et non nommé ?
3. **Périmètre et découpage** : cinq modules ? Migration réellement inutile ? `kesh-api` lit `MIGRATOR` sans dépendance
   nouvelle ? `parse_and_verify_with` compatible avec `spawn_blocking` et le générique `R` ?
4. **Faisabilité des tests**, **en priorité l'AC7 dans sa nouvelle place** (cf. *Attention particulière*) ; AC4 sur
   `test-schema` ; AC11 et sa matière (M7) ; AC1 (i) sans base.
5. **Décomptes** : recompte chaque nombre de la fiche et du registre (39 tables, 83 migrations, 8 exemptions « Hors
   fenêtre », 12 490 octets, 28 lignes du motif de l'AC17, 21 choix au registre, lignes citées) ; signale celui qui ne
   se retrouve pas.
6. **Cohérence interne** : AC1-AC21, T0-T7, registre C-32-12-1 à 21 (C-32-12-12 annoté par C-32-12-15), encadré de
   forme réduite, Change Log — contradiction, critère invérifiable, commande mal formée (exécute chaque `grep` de l'AC17).
7. **Ce que la fiche omet** : un site, un test existant qui rougira sans être nommé (Rust, Vitest, Playwright), une
   conséquence pour la recette visuelle, le CHANGELOG, la 32-5.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée ; pour chacun, dire s'il vient **de la remédiation P1** (`544c2821`) ou de la fiche
d'origine. ⛔ **La liste des axes exercés ET non exercés** (R1 à R8, F1 à F7, et l'*Attention particulière*) — sans
elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est
pas un défaut. Bilan par sévérité **recompté** sur la liste. Rapport écrit **d'un seul jet**, un seul fichier par
lentille : `/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p2-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p2-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire ; aucune redirection d'écriture (`>`, `tee`)
ailleurs que vers ton rapport. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `unzip` vers le disque (`unzip -l`
et `unzip -p` vers la sortie standard seulement), `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
`gh` en écriture. `pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame|ls-tree|tag --contains`,
`gh issue view`, `pdftotext … -`, `unzip -l|-p`.
