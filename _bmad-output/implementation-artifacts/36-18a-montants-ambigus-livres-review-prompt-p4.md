# Prompt — revue de code P4 de la Story 36-18a, passe **ciblée** sur le seul correctif de la P3

Tu relis **un seul commit** de la Story 36-18a de Kesh (SvelteKit 5 runes), release 0.18.0 : `c123e0f66`, qui corrige le LOW
R1 de la passe P3 ciblée (`/home/gcorbaz/devel/kesh-gate-logs/e36-18/review-p3-R.md`) : le refus « Une ligne n'est pas au
centime » ne s'effaçait plus à la correction du prix depuis `56cc62386` ; il est désormais posé dans `amountError`, le refus
de montant que la saisie du prix efface. Le LOW R2 de la P3 (un seul `amountError` pour plusieurs champs) n'est **pas**
corrigé : il part en issue, sur décision de l'orchestrateur — ne le relève pas de nouveau.

Worktree `/home/gcorbaz/devel/kesh-36-18`. Le diff, **aplati** : `git -C /home/gcorbaz/devel/kesh-36-18 show c123e0f66`. Les
commits suivants ne portent que la fiche et ce prompt. Modèle : Haiku, Sonnet ou Opus (passe ciblée) ; contexte frais ;
rapport en français.

Lis aussi : le rapport P3 ; dans la fiche `_bmad-output/implementation-artifacts/36-18a-montants-ambigus-livres.md`, le Dev
Agent Record § *Remédiation de la revue P3* ; les journaux (lecture seule) `kesh-gate-logs/e36-18/rem3-*.log`, `mut/M30.log`.

⛔ Rien sous `~/kesh/`, `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** Aucun fichier écrit hormis ton rapport ; **aucun fichier temporaire**, **aucune redirection** (`>`,
  `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null`). Un fichier créé par erreur se déclare.
- **Aucun `rm`**, ni `pkill`, `killall`, `kill` ; aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`, `jq`, ni
  **`scripts/prepare-release.sh`**, ni aucun script du dépôt ou de `kesh-gate-logs`, ni `cargo`, `docker`, `mariadb`, `curl`,
  **`claude`**, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun réseau.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection), `cat`, `ls`, `wc`. ⚠️ `grep` est une fonction vers `ugrep` : `/usr/bin/grep` ou `git grep`. Toute affirmation
  de présence ou d'absence s'appuie sur une sortie `grep -nF` ou `sed -n` copiée.

## La lentille : chasseur de régressions du correctif

1. Le refus de centime posé dans `amountError` (`SupplierInvoiceForm.svelte`, garde de `submit`) : l'ordre des gardes de
   `submit` est-il inchangé ? `formError` et `amountError` peuvent-ils être non vides ensemble après ce changement ? le
   bandeau (`formError || amountError`) montre-t-il toujours le refus de centime au clic (test « au clic, affiche le message
   DÉDIÉ », `SupplierInvoiceForm.test.ts`) ? La saisie de la **quantité**, qui lève aussi ce refus, ne l'efface pas : dis-le
   s'il le faut, sans le compter comme régression (comportement d'avant `56cc62386` : la quantité ne l'effaçait pas non plus —
   vérifie-le par `git show 56cc62386~1:frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte`).
2. Le commentaire d'`amountError` : dit-il vrai du code, refus par refus ?
3. Le test neuf (« le refus « pas au centime » s'efface à la saisie du prix (M30) ») : rougit-il pour la raison annoncée
   (`mut/M30.log`) ? passe-t-il à vide ?
4. Les comptes rendus (Dev Agent Record § *Remédiation de la revue P3*, Change Log) : chaque chiffre se retrouve-t-il au journal
   cité, à la tête citée ?

## Rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/e36-18/review-p4-R.md`. Findings numérotés (`R1`…) avec sévérité, lieu
(`fichier:ligne`), ligne lue copiée, preuve, remède ; **la liste des axes exercés ET non exercés** ; le décompte final recompté
depuis la liste. Un « 0 finding » non adossé à cette liste ne compte pas.
