# Prompt — revue de code P3 ciblée, Story 25-4-d2b

*Versionné le 2026-10-02. Une lentille (Haiku), contexte frais — passe ciblée sur la remédiation de P2. Trend : P1
1H/3M/6L (Sonnet ×3) → P2 2M/4L (Opus), dont un MEDIUM était une régression du correctif de P1.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2b-bouton-solder-le-reste`. **Diff à revoir : `git diff d9e1c164 8f6d4487`**
(un commit, à lire **aplati** : c'est le seul diff à considérer). Fiche :
`_bmad-output/implementation-artifacts/25-4-d2b-bouton-solder-le-reste.md` (Change Log, entrée « P2 »).

## Lentille — regression hunter

La remédiation :
1. `frontend/src/routes/(app)/invoices/[id]/+page.svelte` : garde `if (!o && writeOffSubmitting) return` **retirée** de
   `onOpenChange` du dialogue de solde ; relecture de la facture (`getInvoice`) après `validateInvoice` ;
2. `frontend/src/lib/features/invoices/WriteOffDialog.svelte` : message « reste inconnu » affiché quand `amountDue` est
   `null` ;
3. `frontend/src/lib/features/invoices/invoice-helpers.ts` : `formatExactAmount` dans un `try/catch` ;
4. `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` : résumé au format exact ;
5. une clé i18n reformulée (4 locales), deux assertions de test.

Vérifie, avec un `grep -nF` pour toute présence ou absence affirmée :
- la relecture après validation : casse-t-elle un test ou un comportement existant de la validation (notification,
  `invoice-validate-*`, état des autres boutons) ? En cas d'échec de la relecture, la fiche reste-t-elle cohérente ?
- sans la garde du parent, une fermeture pendant l'envoi est-elle encore empêchée (par le dialogue lui-même) ?
- la clé `invoices-write-off-error-unknown-due` a-t-elle la même valeur en fr-CH que le repli du code ?
- le résumé de l'échéancier : `summary.unpaidTotal` / `overdueTotal` peuvent-ils être `null` ou vides ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie). ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`.
