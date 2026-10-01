# Prompt — validation P4 ciblée, Story 25-4-d2a (l'écriture de solde et son annulation)

*Versionné le 2026-10-01. **Une lentille** (Haiku), contexte frais — passe ciblée sur la seule remédiation de P3
(cf. `CLAUDE.md` § « La passe ciblée »). Trend : P1 2H/4M/2L → P2 3H/1M/9L → P3 3M.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2a-ecriture-de-solde`. **Diff à relire : `git diff d74dc46c fdb53058`**
(une seule modification de la fiche, `_bmad-output/implementation-artifacts/25-4-d2a-ecriture-de-solde.md`). La fiche
décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Lentille — regression hunter

La remédiation :
1. place le nouveau motif de refus d'annulation (« un solde existe sur la facture ») **juste après** le rang 1
   `InvoiceCredited`, avant la queue de `settlement_entry_cancel_blocker` ;
2. ajoute côté frontend le code du motif à `InvoiceSettlementCancelCode` et son message, la story passant à **cinq
   modules** ;
3. ajoute deux tests (rang ; rapprochement d'une facture soldée refusé).

Vérifie :
- les références citées : `crates/kesh-api/src/routes/invoices.rs:1351-1364`,
  `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts:58-62`, `frontend/src/lib/features/invoices/settlement-cancel.ts`,
  `crates/kesh-db/src/repositories/reconciliation.rs:119-127` — existent-elles et disent-elles ce que la fiche affirme ?
- le code du motif côté frontend : où se trouve la table qui le traduit (le motif est-il dans la partie **commune**
  partagée avec le fournisseur, ou dans la partie propre aux factures client) ? Le test de libellés en dur
  (`frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`) et `sitesTotal` (`i18n-keys.test.ts`) bougeront-ils, et la fiche
  le dit-elle ?
- la fiche est-elle restée cohérente avec elle-même (les « Modules », les « Ce qu'il ne faut pas faire », les tâches
  et l'AC 6 disent-ils la même chose) ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie). Pour tout finding affirmant qu'un code est absent ou
présent : la sortie d'un `grep -nF`. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`.
