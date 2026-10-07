# Prompt — revue de code P5, passe ciblée, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Une lentille (Haiku), contexte frais. Passe **ciblée** (CLAUDE.md § « La passe ciblée »,
D6) : la boucle a convergé en P4 (0 au-dessus de LOW) ; sa remédiation touche du code de production.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. **Le diff à relire, aplati** :
`git diff 5ac148e8 b8b3d56e`. Fiche : `_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` (Change Log, entrée
« Revue de code P4 »).

## Lentille — Regression hunter

La remédiation a changé : la reconstruction de `complementRows` dans `load()` (`+page.svelte`, désormais
conditionnée au status obtenu) ; le texte de la confirmation (`{ $date }`) ; trois messages × 4 locales
(`error-opening-complement-invalid-amount`, `opening-balances-complete-confirm`,
`opening-balances-complete-unavailable-no-completable-account`) et le repli Rust du premier ; des tests Vitest et un
test e2e (`post_description_uses_company_accounting_language`). Cherche ce qu'elle a cassé :

- `load()` : un cas où la grille de complément garde des lignes **périmées** parce que le status a échoué (par
  exemple après une génération ou un changement d'état) ? Le premier chargement en échec ?
- Les messages : chaque variable (`{ $date }`) est-elle fournie par le code qui appelle la clé ? Les 4 locales
  ont-elles la même variable ? Le repli Svelte et le repli Rust sont-ils identiques au fr-CH (`grep -nF`) ?
- Les tests ajoutés prouvent-ils ce qu'ils nomment ?

Pour tout finding affirmant qu'un texte est présent ou absent : la sortie d'un `grep -nF`.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, preuve, correction. ⛔ **La liste des axes
exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`,
`npm`, `npx`, `docker`, `gh issue create`/`comment`/`edit`, aucune requête SQL. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`.
