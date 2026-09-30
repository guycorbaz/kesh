# Prompt — validation P2 ciblée, Story 25-4-c3-a1 (le réglage du compte d'arrondi)

*Versionné le 2026-09-30. **Passe ciblée** : une lentille (Haiku 4.5), contexte frais, braquée sur la
remédiation P1 — `git show 06c075be -- _bmad-output/implementation-artifacts/25-4-c3-a1-reglage-compte-arrondi.md`
(contexte ; les numéros de ligne font foi dans les fichiers courants). Fiche :
`_bmad-output/implementation-artifacts/25-4-c3-a1-reglage-compte-arrondi.md`, **état actuel**. Dépôt
`/home/gcorbaz/devel/kesh`, lecture seule.*

⚠️ La fiche décrit du travail **à faire** : ne signale pas comme défaut que le code ne porte pas encore
ce qu'elle prévoit. Juge la fiche : exactitude de ce qu'elle affirme, faisabilité de ce qu'elle demande.

## Axes — tous obligatoires

1. **AC 5** : `crates/kesh-api/src/lib.rs:340-352` — est-il exact qu'aucune route ne supprime un compte
   (`grep -rn "delete" crates/kesh-api/src/routes/accounts.rs crates/kesh-db/src/repositories/accounts.rs`) ?
   Est-il exact que l'archivage d'un compte désigné comme compte TVA n'est gardé nulle part (lis la
   route et le dépôt d'archivage des comptes) ?
2. **AC 6** : `crates/kesh-db/src/backup.rs` — `export_table` et `column_constraints` lisent-ils bien
   `INFORMATION_SCHEMA` ? `check_schema_compat` accepte-t-il une colonne absente et facultative ? Le test
   `full_import_without_company_column_merges_archive_entries_as_null` existe-t-il à
   `crates/kesh-api/tests/admin_full_import_e2e.rs:2167` et utilise-t-il `strip_column` ?
3. **Dev Notes** : `migrations_upgrade_path.rs` porte-t-il `assert_eq!(total, 69` ?
   `docs/optimistic-locking-patterns.md:48` est-il bien l'énumération décrite ?
4. **Cohérence interne** : AC ↔ tâches, Change Log.

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
