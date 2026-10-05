# Prompt — revue de code P3 ciblée, Story 25-6-a

*Versionné le 2026-10-03. **Une lentille** (Haiku), contexte frais — passe ciblée sur la remédiation de P2. Trend :
P1 4 MED (Sonnet ×3) → P2 2 MED (Opus). La remédiation de P2 ne touche, en production, qu'une chaîne de repli Svelte ;
le reste est tests, AC et manuel.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-a-tableau-de-bord`. **Diff à relire, un seul commit, aplati :
`git show 4f94afd9`.**

## Lentille — regression hunter

1. **Le filtre `status` sur ses deux copies** : `grep -nF "bt.status = 'reconciled'" crates/kesh-db/src/repositories/bank_accounts.rs`
   — deux lignes ? Le test `seule_une_transaction_rapprochee_corrige_le_miroir`
   (`crates/kesh-db/tests/bank_account_statement_gap.rs`) monte-t-il exactement le cas de la SECONDE sous-requête
   (`booked_after_entered_before` : `booking_date > period_to`, écriture `≤ period_to`) ? Calcule ce qu'il attend, avec et
   sans le filtre.
2. **L'AC 4 amendé** (`_bmad-output/implementation-artifacts/25-6-a-tableau-de-bord.md`) : dit-il la même chose que le
   code (`bank_accounts.rs`, branche `(None, Some(r))`) et que le test `sans_compte_lie_le_releve_reste` ? Reste-t-il dans
   la fiche une phrase qui dit encore les trois champs `null` sans compte lié ?
3. **Le test du câblage** (`frontend/src/routes/(app)/homepage-page.test.ts`) : mocke-t-il exactement les modules que la
   page importe (`grep -n "import" "frontend/src/routes/(app)/+page.svelte"`) ? Prouverait-il quelque chose si
   `listBankAccounts` réussissait ?
4. **Résidus** : `grep -rn "Deux limites\|'Solde')" docs/manual/fr/user-manual.tex "frontend/src/routes/(app)/bank-accounts/+page.svelte"`.

## Ce que tu rends

Findings avec sévérité, endroit, **preuve obligatoire : la commande ET sa sortie, ou le calcul** — un finding sans
preuve sera rejeté, un « 0 finding » sans les preuves des quatre vérifications aussi. ⛔ La liste des axes exercés ET non
exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`.
