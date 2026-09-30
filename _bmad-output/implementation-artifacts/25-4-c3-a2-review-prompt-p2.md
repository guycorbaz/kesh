# Prompt — revue de code P2 ciblée, Story 25-4-c3-a2 (le compte d'arrondi dans les plans livrés)

*Versionné le 2026-09-30. **Passe ciblée** : une lentille (Haiku 4.5), contexte frais, braquée sur la
remédiation P1 — `git show HEAD` de la branche `story/25-4-c3-a2-compte-arrondi-plans`, écrit dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/c3a2-p2.diff`.
Dépôt `/home/gcorbaz/devel/kesh`, lecture seule.*

## Axes — tous obligatoires

1. **Le refus du rôle sur l'entrée marquée** (`crates/kesh-core/src/chart_of_accounts/mod.rs`,
   `validate_chart`) : les trois plans livrés passent-ils toujours (le `6940` ne porte aucun rôle —
   vérifie dans les JSON) ? Le test neuf prouve-t-il ce qu'il annonce ?
2. **Les types liés par l'enum** (`crates/kesh-db/src/repositories/company_invoice_settings.rs`,
   `rounding_account_from_chart`) : `AccountType` s'encode-t-il en MySQL vers la même chaîne que la colonne
   `accounts.account_type` (lis `crates/kesh-db/src/entities/account.rs`, `impl Encode`) ? Le nombre de `?`
   et de `.bind()` concorde-t-il ?
3. **Le `tracing::warn!`** : champs et syntaxe corrects, `tracing` est-il une dépendance de `kesh-db` ?
4. **Le doc-comment** : dit-il vrai sur le comportement (absence → `None`, erreur SQL → remonte) ?

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
