# Prompt — validation P4 ciblée, Story 25-4-c3-b

*Versionné le 2026-09-30. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la seule
remédiation P3 : `git show b1853bb4` (un seul commit, diff aplati).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-c3-b-arrondi-au-centime.md`.
Issue : `gh issue view 490`. ⚠️ La fiche décrit du travail **à faire** : ne reproche ni au code ni aux
manuels de ne pas encore le porter (un tel finding sera rejeté).

## Axes — tous obligatoires

1. **Chaque phrase ajoutée par `b1853bb4`** est-elle vraie contre le code ? En particulier : la garde
   actuelle refuse-t-elle 10.008 sur 10.0050 en 400 (`invoice_settlements_write.rs:165-170`,
   `crates/kesh-api/src/errors.rs`, branche `InvalidInput`) ; big.js (`frontend/node_modules/big.js`) n'a-t-il
   pas de `.dp()` ; `new Big("1e-3").toString()` rend-il « 0.001 » (essai `node -e` dans le scratchpad) ;
   `invoice_settlements.amount` est-il `DECIMAL(19,4)` (migrations).
2. **Contradictions** : une phrase ajoutée contredit-elle une autre phrase de la fiche ? Greppe `into_parts`,
   `normalize`, `décimales`, `garde`, `mutation`.
3. **Tests de l'AC 7** : chaque test, tel qu'écrit maintenant, est-il soit rouge avant le patch, soit
   explicitement déclaré garde de mutation ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité `fichier:ligne`), correction.
⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`, `npm run`, `npx playwright`,
`gh issue create`/`comment`/`edit`). Expériences uniquement dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`.
