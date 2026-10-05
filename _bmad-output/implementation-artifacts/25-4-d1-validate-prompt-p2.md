# Prompt — validation P2 ciblée, Story 25-4-d1

*Versionné le 2026-10-01. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la remédiation P1
(entrée « Validation P1 » du Change Log).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-d1-comptes-de-solde.md`.
⚠️ La fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **Le double marqueur interdit** : la règle est-elle complète ? Une entrée marquée `roundingDifference` ET une
   nature ; deux natures sur une même entrée (impossible par le type `Option<WriteOffNature>`, mais la fiche le
   dit-elle ?) ; un compte marqué qui porterait aussi un rôle.
2. **Les messages préservés** : les quatre tests cités (`crates/kesh-core/src/chart_of_accounts/mod.rs:994, :1008,
   :1019, :1038`) existent-ils et assertent-ils bien sur ces messages (`sed -n`) ?
3. **Le test de comptage à créer** : les bornes citées (`mod.rs:425, :439, :458`) et `accounts_role_backfill.rs:148-150`
   disent-elles ce que la fiche affirme ?
4. **Contradiction** : une phrase de la fiche dit-elle encore qu'un test de comptage existe, ou contredit-elle le
   double marqueur ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (sortie de `grep -nF` / `sed -n`), correction. ⛔ La liste des axes exercés
ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`, `make`,
`latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed -n`, `git show`/`diff`/`log`.
