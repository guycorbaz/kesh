# Prompt — revue de code P2, Story 25-6-a (le tableau de bord dit vrai)

*Versionné le 2026-10-03. Une passe (Opus), contexte frais, les trois lentilles. Trend : P1 0 CRITICAL/HIGH, 4 MED
(Sonnet ×3). La remédiation de P1 touche plusieurs modules (SQL `kesh-db`, trois composants, i18n, manuel) : protocole
complet, avec une attention particulière au diff de la remédiation.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-a-tableau-de-bord`. **Diff complet : `git diff 91ca244f 9f14ca58`** ;
**remédiation de P1 : `git diff 6489d6b7 9f14ca58`**. Fiche : `_bmad-output/implementation-artifacts/25-6-a-tableau-de-bord.md`
(AC 1–8, limites, Dev Agent Record, Change Log). Issues : `gh issue view 388`, `389`. Règles : `CLAUDE.md`.

## Ce que tu vérifies

1. **La remédiation de P1** :
   - le SQL de `list_by_company_with_balances` (`crates/kesh-db/src/repositories/bank_accounts.rs`) : la table `agg`
     désormais bornée aux comptes liés — compte les `?` contre les `.bind` (cinq) ; le résultat de `currentBalance` est-il
     inchangé pour TOUT compte lié, y compris un compte lié archivé avec `include_archived` faux ?
   - le test `seule_une_transaction_rapprochee_corrige` (`crates/kesh-db/tests/bank_account_statement_gap.rs`) tue-t-il
     vraiment la mutation « filtre `status` retiré » — les DEUX sous-requêtes portent-elles le filtre, et le test
     exerce-t-il celle qui compte ?
   - `BankAccountsCard.svelte` : l'état `error`, l'« écart non calculable » (sa condition est-elle juste dans tous les cas —
     compte non lié avec relevé, compte partagé sans relevé ?), le `{#if}`/`{:else}` bien fermé ;
   - `format.ts` : le `-0` évité — `Big(…).eq(0)` sur `-0.004` arrondi ?
   - la page (`routes/(app)/+page.svelte`) : la tuile bancaire en échec, puis un rechargement réussi ?
   - le reclassement de F1 (« plusieurs transactions sur une écriture, non atteignable ») : vérifie les cinq sites de
     `reconciliation.rs` cités au Change Log — lient-ils tous une écriture créée dans la même fonction ? Un autre
     chemin (`invoices.rs`, règlement de facture, 25-3) pose-t-il `matched_entry_id` ?
2. **Ce que P1 n'a pas couvert** : la page des comptes bancaires (`routes/(app)/bank-accounts/+page.svelte`) avec
   `currentBalance` en chaîne — tri, comparaisons, `formatBalance` ; tout autre lecteur de `BankAccountSummary` qui
   ferait de l'arithmétique sur `currentBalance` (`grep -rn "currentBalance" frontend/src`).
3. **Le manuel** — `docs/manual/fr/user-manual.tex` § *Tableau de bord*, *Solde comptable, relevé et écart*, *Import
   bancaire*, et les trois chemins « Administration → Comptes bancaires » ; **PDF aplati** vers `target/gate-logs/`.
4. **Recomptes** : `sitesTotal` 1767 (`frontend/src/lib/shared/i18n-keys.test.ts`, recompte `grep -o "i18nMsg("` des
   trois tuiles), parité des clés `homepage-*` (30 par locale) et `bank-accounts-labels-balance` dans les 4 locales.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, `grep -nF` pour toute
présence ou absence, calcul chiffré), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`, `node -e` sans effet de bord.
