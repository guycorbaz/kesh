# Prompt — revue de code P2 ciblée, Story 25-4-c3-b

*Versionné le 2026-09-30. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la seule
remédiation P1 : `git show 1e1aaebf` (un seul commit, diff aplati).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-c3-b-arrondi-au-centime.md`
(Change Log, entrée « Revue de code P1 »). Issue ouverte en route : `gh issue view 491`.

## Axes — tous obligatoires

1. **Le bras d'erreur de `settlement_journal_lines` au rapprochement** (`crates/kesh-api/src/routes/reconciliation.rs`,
   `grep -nF "INTERNAL_ERROR" crates/kesh-api/src/routes/reconciliation.rs`) : le code `INTERNAL_ERROR`
   est-il déjà émis ailleurs dans un `FailedProposal`, ou connu du frontend
   (`grep -rn "INTERNAL_ERROR" frontend/src`) ? Un code inconnu de l'écran s'affiche-t-il lisiblement
   (`ReconciliationProposals.svelte`, table des libellés d'erreur) ? Le `tracing::error!` fuit-il une donnée
   sensible ?
2. **Le test `a_revenue_rounding_account_receives_the_gap`** (`crates/kesh-api/tests/invoice_echeancier_e2e.rs`) :
   prouve-t-il ce qu'il dit ? Changer `account_type` et `number` d'un compte déjà désigné contourne-t-il une
   garde que l'écran appliquerait (le test mime-t-il un état atteignable) ?
3. **Le doc-comment ajouté** à `settlement_journal_lines` (`crates/kesh-db/src/repositories/invoice_settlements.rs`) :
   vrai ? (une écriture à deux lignes sur le même compte passe-t-elle `create_in_tx` —
   `grep -n "account_id" crates/kesh-db/src/repositories/journal_entries.rs | head -40` ; un contrôle de
   doublon existe-t-il ?)
4. **Les réfutations et reclassements de la P1**, écrits au Change Log : l'argument « R toujours pris en
   dernier » tient-il contre `grep -rn "FROM accounts.*FOR UPDATE\|UPDATE accounts" crates/kesh-db/src crates/kesh-api/src` ?
   L'issue #491 décrit-elle exactement le cycle ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (sortie de `grep -nF` ou code cité relu), correction.
⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git show`/`diff`/`log`, `gh issue view`,
`cargo check`.
