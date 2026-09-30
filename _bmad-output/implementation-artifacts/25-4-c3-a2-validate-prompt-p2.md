# Prompt — validation P2 ciblée, Story 25-4-c3-a2 (le compte d'arrondi dans les plans livrés)

*Versionné le 2026-09-30. **Passe ciblée** : une lentille (Haiku 4.5), contexte frais, braquée sur la
remédiation P1 — `git show HEAD -- _bmad-output/implementation-artifacts/25-4-c3-a2-compte-arrondi-plans.md`
(contexte ; les numéros de ligne font foi dans les fichiers courants). Fiche, **état actuel** :
`_bmad-output/implementation-artifacts/25-4-c3-a2-compte-arrondi-plans.md`. Dépôt `/home/gcorbaz/devel/kesh`,
lecture seule.*

⚠️ La fiche décrit du travail **à faire** : juge l'exactitude de ce qu'elle affirme et la faisabilité de ce
qu'elle demande, pas l'état du code.

## Axes — tous obligatoires

1. **AC 3 tranché** : `crates/kesh-db/src/repositories/company_invoice_settings.rs` — les deux
   `insert_with_defaults*` peuvent-ils lire `companies.org_type` et appeler `load_chart` sans changer de
   signature ? Les recherches par rôle (`:288-318`) sont-elles bien `FOR UPDATE` ? `set_org_type`
   (`crates/kesh-api/src/routes/onboarding.rs:322`) n'est-il atteignable qu'à l'étape 3 ?
2. **AC 5** : `crates/kesh-db/tests/accounts_role_backfill.rs` dérive-t-il bien ses assertions de
   `load_chart` (`chart.len()`, `is_postable`) ? `crates/kesh-db/tests/closing_accounts_backfill.rs`
   n'appelle-t-il vraiment ni `load_chart` ni `bulk_create_from_chart` ?
3. **Où regarder** : `is_postable` est-il à `chart_of_accounts/mod.rs:340` ?
4. **Cohérence interne** : AC ↔ tâches, Change Log ; l'issue #488 existe-t-elle (`gh issue view 488`) ?

## Ce que tu rends

- **Findings** : sévérité, endroit exact, **preuve** : la commande exécutée **et sa sortie copiée**, ou
  l'extrait de code lu avec son numéro de ligne. Un finding sans preuve ne sera pas retenu ; un « 0
  finding » sans preuve non plus.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base
(`scripts/*`, `make`, `latexmk`, `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`/`switch`,
`sqlx migrate`, `cargo test`/`nextest`, `npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`).
Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `cargo check`.
