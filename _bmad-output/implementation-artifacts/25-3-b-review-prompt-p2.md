# Prompt — passe 2 de `bmad-code-review`, Story 25-3-b

*Versionné le 2026-09-25. **Une** lentille en contexte frais (Haiku 4.5 — cycle Sonnet → Haiku →
Opus), sur deux périmètres : la remédiation de la passe 1, et l'**axe que la passe 1 a laissé
partiel**. ⚠️ Haiku : le diff est fourni **aplati en un seul fichier**.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-b-annuler-rapprochement`, tête `10776334`.
Fiche : `_bmad-output/implementation-artifacts/25-3-b-annuler-rapprochement.md` (Change Log, ligne
`review P1`). Code principal : `crates/kesh-db/src/repositories/reconciliation_cancel.rs`.

## Axe A — la remédiation de la passe 1

`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-b-p1-remediation.diff`
— l'ancienne `cancel_blocker(conn, company_id, bank_transaction_id)` (code mort) est supprimée ;
`blocker_for` devient `cancel_blocker(conn, company_id, &bt)`, appelée par `get_view` et
`cancel_in_tx`. Reste-t-il un appelant, un doc-comment, un texte de la fiche ou un test qui cite
l'ancienne forme ? Le comportement est-il strictement inchangé ?

## Axe B — l'ordre des verrous, contre les gestes que la passe 1 n'a pas comparés

`cancel_in_tx` verrouille, dans cet ordre : la transaction bancaire (`FOR UPDATE`) → la ligne
`invoice_settlements` qui porte l'écriture (lecture verrouillante de classement) → la facture →
l'écriture et son exercice (`journal_entries JOIN fiscal_years … FOR UPDATE`) → puis le socle
(`reverse_in_tx` / `cancel_settlement_in_tx` : écriture d'origine, exercice du jour
`find_open_covering_date … FOR UPDATE`, insertion). Le tout sous le verrou nommé du compte
bancaire (`with_account_lock`), avec rejeu sur 1213.

Pour **chacun** des gestes suivants, lis son code et dis quelles lignes il verrouille, dans quel
ordre, s'il prend le même verrou nommé, et si un **cycle** avec `cancel_in_tx` est possible — et,
s'il l'est, s'il est rejoué ou rend une erreur :
`routes/reconciliation.rs::post_split`, `accept_one_split`, `accept_one_rule`, `reject_batch`
(`post_reject`) ; `invoice_settlements_write::settle_invoice` ; la création d'un avoir
(`credit_notes`) ; `fiscal_years::close` et `reopen` ; `invoices::unvalidate`.

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** (code lu),
  correction. Pour tout CRITICAL ou HIGH affirmant qu'une chose est absente ou présente : la
  commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes exercés ET non exercés**, geste par geste pour l'axe B. Un « 0 finding »
  sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`,
tout `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`,
`cargo nextest`, `npm run`, `npx playwright`. Lecture, `grep`, `git show`/`log`/`diff` et
`cargo check` autorisés.
