# Prompt — validation P3, Story 25-6-a (le tableau de bord dit vrai)

*Versionné le 2026-10-03. Une passe (Sonnet), contexte frais. Trend : P1 4H/7M (Sonnet ×3) → P2 1H/6M/8L (Opus). La
remédiation de P2 change une **règle métier** (correction de l'écart par la date de valeur, arbitrée par Guy) : la passe
porte sur la fiche entière, avec une attention particulière au diff.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-a-tableau-de-bord`. **Fiche :**
`_bmad-output/implementation-artifacts/25-6-a-tableau-de-bord.md` (tête `761f144d`). Remédiation de P2 :
`git diff 917b5242 761f144d`. Issues : `gh issue view 388`, `389`. Règles : `CLAUDE.md`. La fiche décrit du travail **à
faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Ce que tu vérifies

1. **La correction de date de valeur (AC 4)** — refais le calcul, chiffré :
   - le montant de la transaction (`bank_transactions.amount`, signé comme le relevé) est-il bien égal au mouvement que
     l'écriture de rapprochement porte sur le compte lié (`journal_entry_lines` du compte, dans `matched_entry_id`) ?
     Lis les chemins qui posent `matched_entry_id` (`grep -n "matched_entry_id = ?" crates/kesh-api/src/routes/reconciliation.rs`)
     — rapprochement d'une facture (avec écart d'arrondi, 25-4-c3), éclatement (split), règle, rapprochement manuel :
     la ligne du compte bancaire vaut-elle **toujours** `amount` ? Sinon, la correction est fausse ;
   - une transaction rapprochée **puis** son rapprochement annulé (25-3-b) : `matched_entry_id` et `status` reviennent-ils
     à l'état non rapproché ? l'écriture annulée et sa contre-passation, datées comment, faussent-elles la correction ?
   - un rapprochement qui solde **plusieurs** transactions par une seule écriture, ou l'inverse ?
2. **Cohérence interne** : la fiche dit-elle la même chose partout (AC 1 à 8, tâches, Dev Notes, limites, *Modules*,
   Change Log) ? Les clés i18n neuves nommées sont-elles cohérentes avec les AC qui les emploient ?
3. **Les références ajoutées** existent-elles et disent-elles ce qu'affirme la fiche : `camt053/mod.rs:541`,
   `reconciliation.rs:1958`, `:2295`, `kesh-reconciliation/src/manual.rs:35`, `csv/parser.rs:342`, `vat-purchase.ts:12`,
   `lint-i18n-ownership.js:151`, `homepage-settings.spec.ts:65-70`, `homepage-reminders.spec.ts:19`, `:59-88` ?
4. **Ce que les passes précédentes n'ont pas regardé** : la performance de la requête par compte bancaire (une correction
   qui joint `bank_transactions` à `journal_entries`, index disponibles) ; le cas d'un compte bancaire lié **changé** de
   compte de grand livre après des rapprochements.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute présence
ou absence, calcul chiffré), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`.
