# Revue de code P5 ciblée — Story 34-8 (#938) : la remédiation de la passe ciblée P4

**Passe ciblée, une seule lentille, confiée à Sonnet**, en contexte frais. Elle ne relit **que** la remédiation de la P4
ciblée (Haiku), qui a rendu 1 MEDIUM (C-1, régression de la remédiation P3) et 2 LOW.

## Périmètre

- Worktree : `/home/gcorbaz/devel/kesh-e348`, branche `story/34-8-fournisseur-relecture-apres-commit`.
- **Le diff à relire, aplati** : `git -C /home/gcorbaz/devel/kesh-e348 diff b01229f6..c0ec8440` (le commit de code de la
  remédiation, `c0ec8440`). Pour le contexte : `b01229f6` (le test qui tue M10b, juste avant).
- Rapport P4 : `/home/gcorbaz/devel/kesh-gate-logs/e348-review-p4-ciblee.md`.
- Mutations : `/home/gcorbaz/devel/kesh-gate-logs/e348/mut/SYNTHESE.log` (M10b, M10c, M13, M14) et leurs `.log`.

## Interdits

Lecture seule. Aucune commande qui écrit, compile, exécute un test ou lance un script (`cargo`, `npm`, `npx`,
`scripts/*.sh`, `scripts/prepare-release.sh` compris), aucun `git` qui écrit, aucune redirection ni fichier temporaire.
Permis : `git diff`, `git show`, `git log`, `git grep`, `/usr/bin/grep`, `sed -n`, `cat`. Toute affirmation de présence ou
d'absence s'appuie sur une sortie `grep -nF` copiée, prise **au fichier** (non au diff). Le seul fichier écrit est ton
rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e348-review-p5-ciblee.md` : findings numérotés `S-n`, sévérité, preuve, correction
proposée. Puis **la liste des axes exercés et celle des axes non exercés**.

## La lentille : « la remédiation a-t-elle introduit le défaut suivant ? »

Dans `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte` :

1. **`reloadAfterRefusal`** ne vide plus les refus des blocs de statut que si `invoice.status` a changé
   (`statusBefore`). Le **statut** est-il le bon critère ? Un bloc peut-il changer sans que le statut change (par exemple
   le bloc d'annulation, `cancellable`, ou le bloc du règlement, `settlementCancellable`), laissant un refus devenu faux ?
   Inversement, un statut qui change peut-il garder un refus encore juste ?
2. Le cas `invoice` nul avant la relecture (`statusBefore` indéfini) : chargement initial en échec, puis relecture.
3. Les appelants de `reloadAfterRefusal` (issue inconnue, refus périmé, document) : le changement de condition change-t-il
   l'un d'eux autrement que prévu ? M10 (`clearBlockErrors` dans `clearCycleErrors`) tient-il toujours ?
4. **`detach()`** : `detachError = ''` après la confirmation (C-2). Un refus d'avant reste-t-il affiché à tort pendant
   l'envoi suivant ?
5. **Le commentaire de `clearCycleErrors`** (C-3) dit-il vrai de chaque geste ?
6. **Les tests neufs** (`supplier-unknown-outcome-page.test.ts` : « refus périmé relu au même statut », « confirmation
   refusée garde le refus d'avant ») mordent-ils pour la bonne raison (M13, M14) ? La première forme de M14 a survécu : la
   justification de `SYNTHESE.log` tient-elle ?

Un « 0 finding » ne compte que s'il est adossé à la liste des axes exercés.
