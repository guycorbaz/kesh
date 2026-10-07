# Prompt — validation P7, passe ciblée, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Une lentille (Haiku), contexte frais. Passe **ciblée** (CLAUDE.md § « La passe ciblée »,
décision D6) : braquée sur la seule remédiation de P5 et P6.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. Fiche :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md`. **Le diff à relire, aplati** :
`git diff 0a8e2ae6 c6819583 -- _bmad-output/implementation-artifacts/25-7-soldes-de-depart.md`. Lis ensuite l'AC 4,
l'AC 7, les Tasks, les Dev Notes et les Limites **dans leur état final** (le fichier, pas le diff).

## Lentille — Regression hunter

La remédiation a introduit `LOCK IN SHARE MODE`, un filtre `company_id` sur le verrou des comptes, un ordre des refus
par compte, `accounting::validate`, un entrelacement (5), une réécriture de la propriété (iii) et de la section
« Cycles », le premier exercice en `ORDER BY start_date`. Cherche **ce qu'elle a cassé ou laissé contradictoire** :

- une phrase de la fiche qui contredit une autre après la réécriture (AC 4 contre Tasks, Dev Notes, Limites, AC 7,
  Change Log) ; un reste de `FOR SHARE` dans une prescription (les mentions historiques du Change Log sont légitimes) ;
- l'entrelacement (5) : est-il réalisable et son assertion décidable ?
- `accounting::validate` : existe-t-il sous ce nom, avec quelle signature ? (`grep -rn "pub fn validate"
  crates/kesh-core/src`) ;
- le filtre `AND company_id = ?` : le plan reste-t-il sur `PRIMARY` ? (la mesure de P6 l'a établi : `range` sur
  `PRIMARY`) ;
- l'ordre des refus par compte : cohérent avec la table de l'AC 5 et les tests de l'AC 7 ?

Pour tout finding affirmant qu'un texte est présent ou absent : la sortie d'un `grep -nF`.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), ligne de la fiche, preuve, correction. ⛔ **La liste des axes
exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh` et `25-7-validate-p6-lockprobe.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/
`stash`, `sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, `docker`, toute
requête SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`.
