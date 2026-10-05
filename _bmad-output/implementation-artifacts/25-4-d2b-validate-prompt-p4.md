# Prompt — validation P4 ciblée, Story 25-4-d2b

*Versionné le 2026-10-02. **Une lentille** (Haiku), contexte frais — passe ciblée sur la remédiation de P3. Trend :
P1 2H/3M/2L → P2 9M/8L → P3 1M/1L.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2b-bouton-solder-le-reste`. **Diff à relire : `git diff 895fed4e 211052a4`**
(la fiche `_bmad-output/implementation-artifacts/25-4-d2b-bouton-solder-le-reste.md`). La fiche décrit du travail **à
faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Lentille — regression hunter

1. AC 2, la gestion des refus : lis **tout** le passage (du « refus : » jusqu'à la fin de l'AC 2). Les cas
   `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION`, `INVALID_INPUT` « déjà payée » / « rien à solder », et la règle
   « après toute relecture » forment-ils maintenant une partition **sans contradiction** : pour chaque cas, le dialogue
   se ferme-t-il ou reste-t-il ouvert, sans ambiguïté ?
2. AC 7, le montage de l'E2E : la référence `crates/kesh-api/src/routes/company_invoice_settings.rs:83-91` existe-t-elle
   et nomme-t-elle bien les trois champs obligatoires cités ?
3. Le Change Log de la fiche : l'entrée P3 dit-elle ce que le diff fait ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, `grep -nF` pour toute présence ou absence). ⛔ La liste
des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`.
