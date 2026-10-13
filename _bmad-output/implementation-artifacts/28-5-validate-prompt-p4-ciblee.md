# Prompt — validation P4 ciblée de la spec, Story 28-5 (28-5a serveur / 28-5b écran)

*Versionné le 2026-10-13. Passe ciblée (`CLAUDE.md` § « La passe ciblée ») : **une seule lentille, Haiku**, contexte
frais, en lecture seule, braquée sur la **seule remédiation P3** de la fiche. La P3 était déjà une passe ciblée Haiku ;
sa remédiation est petite (une mutation requalifiée, un numéro de ligne, une nuance de registre) et ne touche aucun
code.*

Dépôt `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5`. **Objet — un diff unique, aplati** (un seul commit,
`981ea9df`, sur `8e875312`) :

```sh
git diff 8e875312 981ea9df -- _bmad-output/
```

Trois fichiers : la fiche `_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md`, le registre
`epic-28-choix-autonomes.md` (un seul hunk, *Retenu* de C-28-5-4) et `sprint-status.yaml` (clé `28-5a-…`). Lis le diff,
puis ouvre chaque fichier **autour des hunks** pour relire le texte final (les numéros de ligne d'un hunk sont ceux du
fichier d'après). Rapport remédié : `/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p3-ciblee.md` (MEDIUM P3-1 ; LOW
P3-2, P3-3).

⚠️ **Erreur de catégorie à ne pas commettre.** La story n'est **pas développée** : tu relis une SPÉCIFICATION. Une
route, un test (B1, B10…), un doc-comment ou un texte de manuel que la fiche **prescrit** et qui n'existe pas encore
dans le code n'est **PAS** un défaut — c'est le travail à faire. Un défaut est : une contradiction entre deux endroits
du texte, une prescription inexécutable ou fausse au code **actuel**, un fait cité faux (numéro de ligne, définition de
colonne, nom de fonction), un décompte faux, ou un site oublié. Vérifie ce point avant chaque finding.

⚠️ **Grep ground-truth obligatoire.** Toute affirmation de présence ou d'absence s'appuie sur une sortie de
`grep -nF "<chaîne>" <fichier>` (ou `sed -n 'A,Bp'`) **copiée dans le rapport**. Pas de finding CRITICAL ou HIGH
« le patch n'a pas été appliqué » sans cette sortie.

## Lentille unique — chasseur de régressions de la remédiation P3

1. **La preuve d'équivalence de M17 est-elle vraie ?** La fiche cite
   `crates/kesh-db/migrations/20260504000001_bank_imports.sql:74` et le squash
   `crates/kesh-db/test-schema/0001_schema_squash.sql` : `sed -n '74p'` de la migration porte-t-il bien
   `ON UPDATE CURRENT_TIMESTAMP(3)` sur `updated_at` de `bank_transactions` (et non d'une autre table) ? Même
   définition dans le squash, pour `bank_transactions` ? Une migration **ultérieure** (`grep -rnF "bank_transactions"
   crates/kesh-db/migrations/`) a-t-elle modifié cette colonne (`MODIFY`/`CHANGE`) et retiré l'`ON UPDATE` ?
   L'`UPDATE` prescrit (Séquence de la route, point 4) change-t-il bien au moins une autre colonne
   (`auto_match_rejected_at`, `version`), condition pour que MariaDB renouvelle la colonne ? Une `version = version + 1`
   suffit-elle ? Le moteur renouvelle-t-il la colonne **même si** `NOW(3)` est affecté explicitement (cas non muté) ?
2. **Cohérence de la requalification.** M17 est-elle dite **équivalente** (et non « tuée ») à **tous** les sites :
   AC12 (a), tableau des mutations, T6a, `sprint-status.yaml` (28-5a), note de B1 ? Reste-t-il, hors historique, une
   phrase qui lui fait « tuer » ou « rougir » un test, ou qui compte M17 parmi les mutations tuées ? Greppe `\bM17\b`
   sur la fiche, le registre, l'epic et `sprint-status.yaml`, et triage chaque occurrence (historique / vivante).
   L'entrée **P2** du Change Log est-elle **intacte** (aucune ligne réécrite, `git diff` à l'appui) ?
3. **B1 et son assertion `updated_at`.** La fiche dit que l'assertion « postérieur à la valeur forcée » ne prouve que
   l'écriture de la ligne. Est-ce exact, et B1 prescrit-il toujours un montage exécutable (forçage de `updated_at` dans
   le passé par un `UPDATE` direct : ce forçage-là, lui, déclenche-t-il l'`ON UPDATE` et écrase-t-il la valeur
   voulue ? Si la requête de forçage ne fixe pas `updated_at` explicitement, la valeur forcée est-elle perdue ?).
   Les comptes `journal_entries` / `invoice_settlements` de B1 sont-ils intacts ?
4. **Décomptes de la partie a.** M1-M17 (17 lignes du tableau), dont combien déclarées équivalentes en partie a
   (M13, M14, M17 : trois) ? « M1-M8 et M15 » = 9 tuées : cohérent avec le tableau ? Aucun chiffre contradictoire
   dans l'AC12 (a), T6a, le Change Log P3 et `sprint-status.yaml` ? Recompte `grep -cE "^\| M[0-9]+ "` sur la fiche.
5. **P3-2 : la ligne `docs/api-external.md`.** `grep -nF "GET /api/v1/bank-imports/{id}" docs/api-external.md` rend-il
   574 ? Reste-t-il un `575` désignant cette ligne (`grep -nE "\b575\b"` sur fiche, registre, epic, sprint) ? La
   plage `docs/api-external.md:534-575` (références de la fiche) est-elle encore juste, ou à réaligner ?
6. **P3-3 : registre.** « (nuancé, C-28-5-30) » est-il en fin de phrase du *Retenu* de C-28-5-4, sans casser la
   phrase ? Le décompte `grep -c "^### C-28-5-"` vaut-il toujours 30 ?
7. **Aucun code touché.** `git diff --stat 8e875312 981ea9df` ne montre que des fichiers sous `_bmad-output/` ?
   (Condition de clôture de la boucle : la remédiation ne touche aucune ligne de production.)

## Ce que tu rends

Findings numérotés (CRITICAL / HIGH / MEDIUM / LOW), chacun avec `fichier:ligne`, **origine** (« remédiation P3 » ou
« antérieur »), **preuve** (sortie de `grep -nF` ou `sed -n` copiée), correction proposée. ⛔ **Liste des axes exercés
ET non exercés** (avec, pour chacun, ce qui a réellement été lu ou exécuté) : un « 0 finding » sans elle ne compte pas
comme passe. Rapport complet dans **`/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p4-ciblee.md`** ; dernier
message : le chemin, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris **aucun** fichier hors de `/home/gcorbaz/devel/kesh-gate-logs/` (le rapport seul). Aucune commande qui écrit
dans le dépôt ou dans une base, ni qui compile ou exécute quoi que ce soit : `scripts/*` (dont
**`scripts/prepare-release.sh`** et `scripts/test-fast.sh`), `make`, `latexmk`, `cargo`, `sqlx`, `npm`, `npx`,
`docker`, `mariadb`/`mysql`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`push`/`tag`, `gh` en
écriture. **Aucun interpréteur** : ni `python`/`python3`, ni `node`, ni `perl`, ni `ruby`, ni `awk`, ni
`bash -c`/`sh -c` de script. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `tr`, `cut`, `head`, `tail`,
`git log`/`show`/`diff`/`grep`.
