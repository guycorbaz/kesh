# Prompt — revue de code P2 ciblée, Story 25-4-c3-a1 (le réglage du compte d'arrondi)

*Versionné le 2026-09-30. **Passe ciblée** : une lentille (Haiku 4.5), contexte frais, braquée sur la
remédiation P1 — `git show aa76eb2c`, écrit dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/c3a1-p2.diff`.
Dépôt `/home/gcorbaz/devel/kesh`, lecture seule. Fiche : `_bmad-output/implementation-artifacts/25-4-c3-a1-reglage-compte-arrondi.md`.*

## Axes — tous obligatoires

1. **La validation conditionnelle** (`crates/kesh-api/src/routes/company_invoice_settings.rs`, fichier
   courant) : `default_rounding_account_id` n'est validé que s'il diffère de `current`. Cherche un chemin
   où un compte **invalide** est désormais accepté à tort : `current` lu hors transaction avant le
   `update` — une course entre deux `PUT` peut-elle faire enregistrer un compte jamais validé ? Le verrou
   optimiste (`version`) le rattrape-t-il ? Un compte d'une **autre société** peut-il passer ainsi ?
2. **Le test neuf** `settings_unchanged_archived_rounding_account_does_not_block_other_changes`
   (`crates/kesh-api/tests/idor_multi_tenant_e2e.rs`) : prouve-t-il ce qu'il annonce ?
3. **Le Playwright** (`frontend/tests/e2e/settings-rounding-account.spec.ts`) : le `finally` peut-il
   lui-même échouer et masquer l'erreur d'origine ? Laisse-t-il la base propre dans tous les cas ?
4. **`double_option` en `pub(crate)`** : les deux sites l'utilisent-ils toujours
   (`grep -rn "double_option" crates/kesh-api/src`) ?

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
