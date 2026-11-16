# Prompt — revue de code P3 de la Story 36-18a, passe **ciblée** sur le seul correctif de la P2

Tu relis **un seul commit** de la Story 36-18a de Kesh (SvelteKit 5 runes), release 0.18.0 : `56cc62386`, qui corrige deux LOW
de la passe P2 ciblée (`/home/gcorbaz/devel/kesh-gate-logs/e36-18/review-p2-R.md`, R2 et R3). La règle du projet (« tarir la
source ») veut qu'un LOW corrigé dans la story rouvre la boucle sur une passe ciblée de ce seul correctif : c'est elle.

Worktree `/home/gcorbaz/devel/kesh-36-18`. Le diff, **aplati** : `git -C /home/gcorbaz/devel/kesh-36-18 show 56cc62386`.
Les commits suivants ne portent que la fiche, le registre et ce prompt. Modèle : Haiku, Sonnet ou Opus (passe ciblée) ;
contexte frais ; rapport en français.

Lis aussi : le rapport P2 (R2, R3) ; dans la fiche `_bmad-output/implementation-artifacts/36-18a-montants-ambigus-livres.md`,
le Dev Agent Record § *Remédiation de la revue P2* ; les journaux (lecture seule) `kesh-gate-logs/e36-18/rem2-*.log`,
`mut/M26.log` à `mut/M29.log`, `mut/resume.log`.

⛔ Rien sous `~/kesh/`, `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** Aucun fichier écrit hormis ton rapport ; **aucun fichier temporaire**, **aucune redirection** (`>`,
  `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null`). Un fichier créé par erreur se déclare.
- **Aucun `rm`**, ni `pkill`, `killall`, `kill` ; aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`…).
- **Rien qui écrit, compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`, `jq`, ni
  **`scripts/prepare-release.sh`**, ni aucun script du dépôt ou de `kesh-gate-logs`, ni `cargo`, `docker`, `mariadb`, `curl`,
  **`claude`**, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun réseau.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection), `cat`, `ls`, `wc`. ⚠️ `grep` est une fonction vers `ugrep` : `/usr/bin/grep` ou `git grep`. Toute affirmation
  de présence ou d'absence s'appuie sur une sortie `grep -nF` ou `sed -n` copiée.

## La lentille : chasseur de régressions du correctif

1. **R3 — `amountError`** (`frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte`) : chaque site qui posait
   un refus de montant le pose-t-il désormais dans `amountError`, et aucun autre refus ? `submit` et le scan remettent-ils
   `amountError` à vide ? Le bandeau `{formError || amountError}` : les deux peuvent-ils être non vides ensemble, et lequel
   l'emporte ? un refus serveur arrivé après un refus de montant ? `git grep -n "amountError\|formError" -- frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte`
   — un site oublié ? Le `data-testid="supplier-invoice-form-error"` garde-t-il ses lecteurs (tests, E2E) ?
2. **R2 — exclusivité débit/crédit** (`frontend/src/routes/(app)/settings/opening-balances/+page.svelte`) : `onDebitInput` /
   `onCreditInput` sur `.trim()`, désormais appelés par les deux grilles ; le type élargi (`{ debit; credit }`) ; un site qui teste
   encore la saisie brute (`git grep -n "!== ''" -- 'frontend/src/routes/(app)/settings/opening-balances/+page.svelte'`) ;
   « 0 » saisi efface-t-il toujours l'autre côté (comportement d'avant) ?
3. **Les tests neufs** (`SupplierInvoiceForm.test.ts`, `opening-balances-page.test.ts`) : chacun rougit-il pour la raison
   qu'il annonce (M26 à M29, `mut/M*.log`) ; titres conformes à ce que les tests font ; un test qui passe à vide ?
4. **Les comptes rendus** : la fiche (Dev Agent Record § *Remédiation de la revue P2*, Change Log), l'annotation de C-36-18-4
   au registre (R1), l'établissement de R4 (quelle spec a tourné au run r4) : chaque affirmation se retrouve-t-elle au
   journal ou à `git` ?

## Rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/e36-18/review-p3-R.md`. Findings numérotés (`R1`…) avec sévérité, lieu
(`fichier:ligne`), ligne lue copiée, preuve, remède ; **la liste des axes exercés ET non exercés** ; le décompte final recompté
depuis la liste. Un « 0 finding » non adossé à cette liste ne compte pas.
