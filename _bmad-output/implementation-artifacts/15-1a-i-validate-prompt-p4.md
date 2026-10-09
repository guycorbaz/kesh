# Prompt — validation P4 de la spec, Story 15-1a-i

*Versionné le 2026-10-09. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** : `_bmad-output/implementation-artifacts/15-1a-i-marque-du-lettrage.md`. La dernière remédiation est le commit `2d3c4b41` (`git show 2d3c4b41`) — lis le Change Log de la fiche.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #518, #543.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Code de référence : `origin/main` = `5e4bec50`. **La 15-1a vient d'être découpée (C124)** : index `15-1a-socle-lettrage.md` (historique des passes), sous-fiches `15-1a-i-marque-du-lettrage.md` et `15-1a-ii-gardes-du-lettrage.md` — relis les deux pour la frontière. Prérequis `15-12a-cloture-dans-l-ordre.md` (C119), sœur `15-12b-filet-sous-un-bilan-clos.md`. Choix C90, C94, C101 à C106, C113 à C118, C124 à C127. Vérifie que rien n'est perdu au découpage (contre `git show c9cf51f8:_bmad-output/implementation-artifacts/15-1a-socle-lettrage.md`). **Axes prioritaires** : la preuve (α)-(β) de C125 (exercices verrouillés un par un par clé primaire, bornés à ceux qui portent une ligne du groupe ; postérieur clos lu sans verrou par `find_later_closed` — une lecture périmée peut-elle laisser passer un lettrage qu'il faudrait refuser ?) ; la vue REPEATABLE READ ouverte par la lecture (a) avant les verrous (F3-4) ; le test NOWAIT et la mutation « trier par id » ; la règle des périodes de C113 ; le bump et l'en-tête de migration (C101, C115, P2-bis, P7, P8) ; le manuel, PDF aplati compris.
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la dernière remédiation (`git show 2d3c4b41`) a-t-elle introduit des défauts ? Contradictions entre AC, tâches, Dev Notes et choix du registre ; frontière avec les fiches sœurs ; faits recopiés sans vérification (numéros de ligne, fonctions, codes, clés i18n) — revérifie-les au code (`grep -nF`) ; recompte des AC et tâches ; implémentable sans deviner ?
- **F — Full-scope adversary** : la conception elle-même. Les chemins de code qui écrivent ou lisent ce que la story
  change (inventorier les sites NON résolus, pas énumérer ceux qui marchent) ; transactions, verrous, ordre des
  contrôles ; tests existants qui changeront de sens ; frontend qui lit les codes ou champs touchés (`grep -rn` dans
  `frontend/src`) ; i18n 4 locales et replis ; le **manuel** (`docs/manual/fr/*.tex` **et le PDF aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`) ;
  `docs/api-external.md` ; CHANGELOG ; la règle de découpage (§ « Règle de splitting préventif », amendement D5).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et
sortie, ou code cité relu), correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la
sortie d'un `grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-1a-i-p4-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
