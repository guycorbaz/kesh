# Prompt — validation P4 ciblée, Story 25-6-a

*Versionné le 2026-10-03. **Une lentille** (Haiku), contexte frais — passe ciblée sur la remédiation de P3. Trend :
P1 4H/7M → P2 1H/6M → P3 2M/4L.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-a-tableau-de-bord`. **Diff à relire, un seul commit, aplati :
`git diff 27ef4413 71f0d3d1`** (la fiche `_bmad-output/implementation-artifacts/25-6-a-tableau-de-bord.md`). La fiche
décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Lentille — regression hunter

1. **Résidus de l'ancienne règle** : la correction somme désormais la **ligne de l'écriture sur le compte lié**, plus
   `bank_transactions.amount`. Reste-t-il une phrase de la fiche qui parle de sommer `amount` pour la correction ?
   `grep -nF "amount" <fiche>` — lis chaque occurrence et dis si elle concerne la correction.
2. **Le sens de la correction**, chiffré : relevé au 30 (`CLBD` 1000), une transaction rapprochée de +200 comptabilisée
   le 30, écriture le 31 portant `débit 200` sur le compte lié. Solde comptable au 30 = 800 ; correction = ? ; écart = ?
   Puis le cas inverse (comptabilisée le 31, écriture le 30). Puis un lien changé : l'écriture porte sur l'ancien compte —
   que vaut `SUM(jel.debit − jel.credit)` sur le compte lié actuel ?
3. **Cohérence** : les cas de test ajoutés (annulation, lien changé, chemin facture, arrondi négatif) contredisent-ils une
   limite assumée ou un autre AC ? La limite « résidu après annulation » est-elle compatible avec le test « un
   rapprochement annulé : hors de la correction » ?
4. **Références ajoutées** : `camt053/mod.rs:336`, `reconciliation.rs:1270`, `reconciliation_cancel.rs:331`,
   `bank_accounts.rs` (`set_journal_account_id_for_company`) — existent-elles et disent-elles ce qu'affirme la fiche ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve obligatoire : la commande ET sa sortie, ou le calcul** — un finding sans preuve
sera rejeté, un « 0 finding » sans les preuves des quatre vérifications aussi. ⛔ La liste des axes exercés ET non
exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`.
