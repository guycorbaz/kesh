# Prompt — revue de code P1 de la Story 32-12a (trois lentilles B, E, A ; Sonnet)

Tu relis le développement de la **Story 32-12a** de Kesh (comptabilité suisse) — *la règle
d'inventaire et ses textes : une sauvegarde antérieure reste importable* (refs #722) —, dans le
worktree `/home/gcorbaz/devel/kesh-32-12a`, branche `story/32-12a`, partie de `story/32-12`
(`9ae93f4c`, la fiche validée). **Contexte frais** : tu ne connais rien du développement hors de ce
que le dépôt montre. Langue du rapport : français. Modèle : **Sonnet**. Une seule lentille par
instance : celle que l'orchestrateur te nomme (B, E ou A).

**Le diff à relire, aplati** (un seul diff, jamais la suite des commits) :

```sh
git -C /home/gcorbaz/devel/kesh-32-12a diff story/32-12 HEAD
git -C /home/gcorbaz/devel/kesh-32-12a diff --stat story/32-12 HEAD
```

**La spécification** : `_bmad-output/implementation-artifacts/32-12-import-sauvegarde-anterieure.md`
— les AC portés par la 32-12a (**AC1-AC7, AC9-AC11, AC15-AC17, AC18a, AC19a**), les Dev Notes
(*Inventaire de la classe*, *Contrat*, *Fichiers touchés*), et le **Dev Agent Record § 32-12a**, qui
déclare ce qui a tourné. Le registre des choix : `epic-32-choix-autonomes-3212.md`. Les règles :
`CLAUDE.md` (dont § *Migration breaking policy*, **P5, P6, P7 et P8**) et
`docs/agents-orchestration.md` § 6.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucune
  redirection** (`>`, `>>`, `tee`), pas même vers un fichier « jetable » ; **aucun fichier
  temporaire**, ni dans `/tmp` ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- **Aucune commande qui écrit, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`,
  ni `make`, ni `latexmk`, ni `python3`, ni `sqlx`, ni `docker`, ni `curl` ; **ni
  `scripts/prepare-release.sh`**, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun
  script du dépôt. Aucun `git` qui écrit (`commit`, `checkout`, `switch`, `stash`, `tag`, `push`,
  `reset`, `rebase`, `add`). Aucune commande `gh`.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `wc`, `sort`,
  `unzip -l` et `unzip -p` **vers la sortie standard** (pour lire la fixture), en pipes seulement.
- **Toute affirmation de présence ou d'absence** dans le code s'appuie sur une sortie `grep -nF`
  (fixed-string) **copiée** dans le rapport ; un bloc multi-ligne, sur `grep -nFA <N>` ou `sed -n`.

## Lentille B — Blind Hunter (le diff seul, sans la fiche)

Lis le diff aplati **sans** ouvrir la fiche, comme un relecteur qui ne sait pas ce qui était voulu :

1. `kesh-db/src/backup.rs` : `check_backup_inventory` — l'ordre des contrôles (`Inconnue`, puis
   `Exigee`, puis `HorsOrdre`), le tri des absentes (version, puis rang), le choix de la
   `presente` de `HorsOrdre`, une table présente sans date, une table présente datée au plus tard
   au plancher ; `table_creators` / `table_creating_migrations` (`OnceLock`, minimum, dédoublonnage
   d'une même migration). Un `panic!` ou un `unwrap` atteignable en production ?
2. `kesh-db/src/post_restore.rs` : `created_application_table` (casse, accents graves,
   `IF NOT EXISTS`, nom isolé, `;` et `(` collés), passage de `split_statements` et `normalize` en
   `pub(crate)`, retrait de la fonction de test qui calculait la fenêtre, les deux tests P7 dont la
   fenêtre est désormais `IMPORT_FLOOR_MIGRATION`, `post_floor_write_targets` (cibles après
   `INTO`/`FROM`/`UPDATE`, modificateurs, table système, DDL `ON UPDATE CURRENT_TIMESTAMP`).
3. `kesh-api/src/admin_backup/import.rs` : `parse_and_verify_with`, le passage d'une erreur
   d'inventaire à `InvalidBackupStructure`, `tables_absent` et le `Debug` manuel ; les tests AC7
   (`fixture_sans_table`, entrée dossier `files/`, `include_bytes!` du bon chemin).
4. Tests : chaque test neuf peut-il rougir ? Un test qui passe à vide (boucle sur une liste vide
   sans garde de non-vacuité, assertion tautologique) ? `post_floor_table_seeds_are_replayed` est
   déclaré vacant : sa détection est-elle réellement exercée ailleurs ?

## Lentille E — Edge Case Hunter (chemins et bornes)

Parcours chaque branche et chaque borne, et ne rapporte que ce qui n'est pas traité :

1. **Règle** : inventaire complet, une/plusieurs absentes postérieures, deux tables nées dans la
   même migration (une absente, les deux absentes), une absente égale au plancher, une table
   inconnue **et** une table exigée absente dans le même fichier, manifeste vide, inventaire
   injecté vide. Le plancher **égal** à la plus récente créatrice (cas actuel) : la règle tolère-t-
   elle quelque chose aujourd'hui en production ? (`InventoryRules::production()`.)
2. **Import** : une sauvegarde **format 1** (fixture 0.14.0) et **format 2** (fixture 0.15.0)
   passent-elles toujours ? Une table tolérée absente : l'inventaire des noms d'entrées du format 2
   (`attendus`), `verifier_coherence`, `referenced_files`, `check_schema_compat`, `restore_body`,
   `replay_post_restore_backfills` / `missing_sentinels` (table entière absente) — chacun se
   comporte-t-il correctement avec une table manquante ? Le compte `tables_restored` et la garde
   de cohérence du handler (`crates/kesh-api/src/routes/admin.rs`, **non touché**) restent-ils
   justes ?
3. **`post_restore`** : P7 — la fenêtre au plancher rend-elle permissif ou trop strict l'un des
   deux tests, aujourd'hui et une fois qu'une table postérieure existera ? Le compteur `checked`
   (8) tient-il ? Une migration de version égale au plancher ?
4. **P5, P6, P8** : le diff ajoute-t-il ou modifie-t-il un fichier `crates/kesh-db/migrations/*.sql`
   (`git diff --stat story/32-12 HEAD -- crates/kesh-db/migrations`) ? Touche-t-il
   `docs/migrations-idempotence-audit.md` ? Un site `migrations.len()` / `apply_migrations_up_to`
   est-il affecté (`git grep -n "migrations.len()\|apply_migrations_up_to"`) ?
5. **Fixture** (`crates/kesh-api/tests/fixtures/keshbackup-0.15.0-format2.keshbackup`) : `unzip -l`,
   `unzip -p … manifest.json` — format 2, `keshVersion` `0.15.0`, 39 tables, deux pièces ; aucune
   donnée réelle (noms, IBAN : exemples SIX seulement).

## Lentille A — Acceptance Auditor (la fiche contre le code)

Pour **chaque** AC de la 32-12a (AC1-AC7, AC9-AC11, AC15-AC17, AC18a, AC19a) : tenu, partiel ou non
tenu, avec la ligne de code ou de test qui le prouve (`grep -nF`). En particulier :

1. AC1 : les noms exacts (`created_application_table -> Option<&'static str>`, `table_creators`,
   `table_creating_migrations`, `normalize` et `split_statements` en `pub(crate)`), le retrait de
   la fonction de test, les tests (i) et (ii), le doc-comment de la reconnaissance (dangers de
   `split_statements`, `migrations_contain_no_block_comment` cité).
2. AC2 : le doc-comment d'`IMPORT_FLOOR_MIGRATION` nomme-t-il **une à une** les quatre migrations
   publiées ? `TOLEREES_ECRITES` / `tolerees_calculees` / `exigees` ; (i) à (iv).
3. AC3 : signature, ordre défini, les dix cas de test de la fiche, un à un.
4. AC5 : messages `Exigee` et `Inconnue` **mot pour mot** inchangés, message `HorsOrdre` exact ;
   `tables_absent` lu par le `Debug` manuel ; `lib.rs` inchangé (aucune ré-exportation).
5. AC9 : chaque site de `post_restore.rs` énuméré par l'AC9 est-il récrit, et chaque relevé daté
   que l'AC9 dit garder est-il gardé tel quel ? Le compteur `checked` reste-t-il à 8 ?
6. AC16 : chaque texte énuméré, dont les **deux** paragraphes de `CLAUDE.md` P7 ; les quatre
   migrations publiées **non touchées** (P8) ; `post_restore.rs` (justification de l'exemption
   `20260917000001`) **non touché** (confié à la 32-5).
7. **AC17** : relance **toi-même** les cinq greps et la valeur, **tels qu'écrits** dans l'AC17,
   depuis la racine du worktree, suivis de `| wc -l` (pipe, aucun fichier) ; compare les nombres à
   ceux que le Dev Agent Record déclare **après patch**, et confronte chaque ligne au tableau des
   sorts qu'il consigne. Toute ligne qui n'y figure pas est un site neuf : finding.
8. AC18a / AC19a : le Dev Agent Record déclare-t-il des gates et des mutations **exécutés**, avec
   leurs chiffres et journaux (`/home/gcorbaz/devel/kesh-gate-logs/3212a-*.log`, lecture seule) ?
   Chaque mutation a-t-elle un tueur **nommé** qui existe dans le code ? M6 (et M8 le cas échéant)
   déclarée équivalente avec sa raison ? Les tests ajoutés sont-ils recomptés aux deux bornes
   (`git show story/32-12:<f> | grep -c '#\[test\]\|#\[sqlx::test'` contre `HEAD`) ?
9. **Manuel** : `docs/manual/fr/admin-manual.tex` n'est pas touché par la 32-12a (sort *Récrit par la
   32-12b*). La 32-12a le rend-elle faux pour autant ? (lis § *Importer*, l. 2242 et suivantes, et
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers la sortie standard).

## Rapport

Écris ton rapport — et rien d'autre, nulle part — dans
`/home/gcorbaz/devel/kesh-gate-logs/3212a-review-p1-<B|E|A>.md` (selon ta lentille) :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sa sévérité (CRITICAL / HIGH / MEDIUM /
  LOW), `fichier:ligne`, preuve (**sortie `grep -nF` ou `sed -n` copiée**), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été**, avec la raison — un
  « 0 finding » sans cette liste ne compte pas comme passe ;
- tout écart de protocole de ta part (commande interdite lancée, fichier créé) déclaré.
