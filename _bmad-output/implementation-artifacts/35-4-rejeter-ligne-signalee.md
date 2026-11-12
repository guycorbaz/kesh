# Story 35-4 : *Rejeter* une ligne signalée « déjà comptabilisée » demande confirmation ; un geste confirmé sur une ligne partie le dit (#1023, #1024)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration**, aucun changement serveur. Code lu sur **`release/0.17.6`**
(`4e1c768a`), qui porte les stories 34-4 (#828) et 34-11b. Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -12,
-13) ; `e35-choix-autonomes-e354.md` pour la story, à créer au premier choix. Validation à lancer par l'orchestrateur.*

## Story

En tant que titulaire qui réconcilie ses relevés, quand une transaction est signalée « déjà comptabilisée », je veux
qu'un clic sur *Rejeter* me demande de confirmer, en me disant ce que le rejet fait, afin qu'un encaissement réel ne
quitte pas la liste sans que je l'aie voulu ; et quand je confirme un geste sur une ligne qui a quitté la liste entre-temps,
je veux en être averti plutôt que de cliquer dans le vide.

## Ce qui se lit au code (`release/0.17.6`)

Tout est dans `frontend/src/lib/features/reconciliation/` :

- **La garde existe pour trois gestes, pas pour *Rejeter*.** `ReconciliationProposals.svelte:44-48` (en-tête) : « *Accepter
  (N)*, *Écriture manuelle* et *Éclater* demandent alors confirmation (`AlreadyBookedConfirmDialog`) ; *Rejeter* et
  *Rattacher à une écriture existante*, non. » Le type `GuardedGesture` (`:288-291`) n'a que `accept`, `manual`, `split` ;
  `guard` (`:296-303`) ouvre le dialogue si des lignes signalées sont concernées ; `requestAccept` (`:312-317`) relève les
  lignes signalées de la sélection. `onReject` (`:428-445`) est branché **directement** sur le bouton
  (`reconciliation-reject-btn`, `:484-488`) : aucun appel à `guard`.
- **Le signal** : `isAlreadyBooked(p)` = la ligne porte un avertissement (`already-booked.ts:108-110`). Depuis la 34-11b,
  le texte conseille de *rattacher* la ligne (« Rattachez-la … ») ou, pour l'autre côté d'un virement, de la rattacher à
  l'écriture du virement (`already-booked.ts:47-102`) ; il ne conseille plus de la rejeter. Le signal se fonde sur le
  montant et le sens, jamais sur la date ni le libellé : un encaissement réel du même montant qu'une écriture non rapprochée
  est signalé (#1023).
- **Le dialogue** (`AlreadyBookedConfirmDialog.svelte`, 65 lignes) : titre, intro, liste des transactions
  (`describeTransaction`), *Annuler* / *Continuer quand même*. Son intro est écrite pour un geste **qui passe une écriture**
  (« Continuer passerait une écriture pour un mouvement qui est peut-être déjà aux livres : il serait compté deux fois »,
  clé `reconciliation-labels-already-booked-confirm-intro`, fr-CH `messages.ftl:1458`) : elle est fausse pour un rejet.
- **#1024 — un clic sans effet ni message.** `confirmGesture` (`:336-353`) : pour *Écriture manuelle* et *Éclater*, il
  relit la ligne dans la liste courante et sort par `if (!current) return;` (`:347`) si elle a disparu. Pour *Accepter*,
  `load()` vide la sélection (`selected = new Set()`, `:170`) et `onAccept` sort dès `selected.size === 0` (`:374`) : même
  silence (commentaire de #1024). `onReject` sortirait de même (`:429`).
- **Où dire un avis** : `errorMsg` remplace toute la liste (`{:else if errorMsg}`, branche du rendu) : un avis de geste
  sans objet ne doit pas passer par lui.
- **Un rejet se défait** : *Annuler le rejet* (`UnrejectTransactionAction.svelte`, Story 28-5b), dans le détail d'un import
  bancaire (`routes/(app)/bank-import/[id]/+page.svelte:207`), ouvert à l'Administrateur et au Comptable.
- **Tests existants** (`ReconciliationProposals.test.ts`) : `AC-6 Rejeter une ligne signalée ⇒ aucune confirmation (M6)`
  (`:1737-1743`) épingle l'ancien comportement ; `AC-6 Continuer ne rouvre pas le geste d'une ligne sortie de la liste`
  (`:1757-1781`) épingle le silence. E2E : `frontend/tests/e2e/reconciliation-already-booked.spec.ts` (`:72`).
- **Manuel** : `docs/manual/fr/user-manual.tex:3107` décrit la confirmation, le silence (« sans message (issue \#1024) ») et
  « *Rejeter* et *Rattacher à une écriture existante* ne demandent rien ».

## Décisions (C-e350-12, C-e350-13)

- **D1 — *Rejeter* passe par la garde**, comme les trois autres gestes, **pour les seules lignes signalées** de la
  sélection : une sélection sans ligne signalée rejette aussitôt, comme aujourd'hui (pas de friction ajoutée au cas
  courant). *Rattacher à une écriture existante* reste hors garde : c'est le geste que l'avertissement recommande.
- **D2 — Le dialogue dit le geste.** `AlreadyBookedConfirmDialog` reçoit le geste (`'write' | 'reject'`, ou un booléen) ;
  pour *Rejeter*, une intro propre : « Chaque transaction ci-dessous semble déjà comptabilisée. *Rejeter* la retire de la
  liste sans rien écrire : si c'est un mouvement réel qui n'est pas encore aux livres, il y manquera. Un rejet s'annule
  depuis le détail de son import (*Annuler le rejet*). » et un bouton *Rejeter quand même*. Les clés existantes servent
  toujours aux trois autres gestes. Le dialogue nomme les transactions (`describeTransaction`) ; il ne nomme pas l'écriture
  à laquelle la ligne ressemble : la réponse de `GET /reconciliation/proposals` n'en donne que le **nombre**
  (`alreadyBooked`, compteurs lus à `already-booked.ts:48-52`) — limite écrite au manuel, sans changement serveur.
- **D3 — Un geste confirmé sur une ligne partie le dit (#1024).** Quand *Continuer quand même* (ou *Rejeter quand même*)
  trouve la ligne absente de la liste courante, ou la sélection vidée par une relecture, un **avis** s'affiche au-dessus de
  la liste, sans la masquer : « Cette transaction n'est plus à traiter : elle a été rapprochée, rejetée ou modifiée entre-temps.
  La liste est à jour ; vérifiez-la avant de recommencer. » (une clé, au singulier et au pluriel par deux clés, sans
  sélecteur). Il s'efface au geste suivant. Rien n'est envoyé au serveur. Vaut pour les quatre gestes.
- **D4 — *Accepter* et *Rejeter* relisent la sélection confirmée** : le geste confirmé ne porte que sur les lignes encore
  présentes **et** encore sélectionnées ; si aucune ne l'est, l'avis de D3 ; si certaines seulement, le geste part pour
  celles-là, et l'avis nomme les autres. *(À trancher à l'étape 0 et au registre `e354` si la relecture ne garde pas la
  sélection : `load()` la vide aujourd'hui, `:170`, si bien qu'après une relecture il n'en reste aucune — le cas partiel ne
  se produit alors pas.)*

## Critères d'acceptation

1. **AC1 — *Rejeter*, sélection avec une ligne signalée** : le dialogue s'ouvre, nomme la ligne signalée (et elle seule si
   d'autres non signalées sont sélectionnées), avec l'intro et le bouton de D2 ; *Annuler* (et Échap) n'appelle pas
   `rejectProposals` ; *Rejeter quand même* l'appelle avec **toute** la sélection, comme aujourd'hui.
2. **AC2 — *Rejeter*, sélection sans ligne signalée** : aucun dialogue ; `rejectProposals` appelé aussitôt (inchangé).
3. **AC3 — Les trois autres gestes** gardent leur dialogue et son intro d'écriture (tests existants de la 34-4 verts).
4. **AC4 — #1024, *Écriture manuelle* et *Éclater*** : ligne sortie de la liste pendant la confirmation (montage du test
   `:1757`) → aucune modale, et l'avis de D3 est visible ; il disparaît au geste suivant.
5. **AC5 — #1024, *Accepter* et *Rejeter*** : liste relue pendant la confirmation (sélection vidée) → aucun appel au serveur,
   l'avis de D3 visible.
6. **AC6 — L'avis ne masque pas la liste** : les lignes restent affichées sous lui (contrairement à `errorMsg`).
7. **AC7 — Quatre langues.** Clés neuves dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, à côté de
   `reconciliation-labels-already-booked-confirm-*` (fr-CH `:1457-1460`, autres locales `:1388-1391`) ; aucun sélecteur ;
   les gardes i18n du frontend (`i18n-keys.test.ts`, `lint-i18n-ownership`) suivent ; `already-booked-i18n.test.ts` étendu
   si son objet couvre les clés du dialogue.
8. **AC8 — Tests existants réécrits** : `:1737-1743` (« aucune confirmation (M6) ») devient AC1 ; `:1757-1781` asserte en plus
   l'avis (AC4). L'en-tête du composant (`:44-48`) et celui du dialogue (`AlreadyBookedConfirmDialog.svelte:1-14`) disent le
   nouveau contrat.
9. **AC9 — E2E** : dans `reconciliation-already-booked.spec.ts`, une ligne signalée (montage existant, données
   synthétiques), cochée, *Rejeter* → le dialogue s'ouvre ; *Annuler* → la ligne reste ; *Rejeter quand même* → elle
   quitte la liste.
10. **AC10 — Manuel et CHANGELOG.** `user-manual.tex:3107` : *Rejeter* demande confirmation sur une ligne signalée, ce que
    le rejet fait et comment l'annuler ; la phrase « sans message (issue \#1024) » et « *Rejeter* … ne demandent rien »
    remplacées par le comportement nouveau ; la limite de D2 (le dialogue ne nomme pas l'écriture) écrite. PDF régénéré,
    contrôlé aplati. CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8).
11. **AC11 — Captures** (thème clair, base jetable, `kesh-gate-logs/e354/captures/`) : le dialogue de *Rejeter* ; l'avis
    de D3.

## Tests à écrire, et la preuve du rouge

Vitest de `ReconciliationProposals.test.ts` (AC1, AC2, AC4, AC5, AC6) et du dialogue : écrits d'abord, rouges sur le code de
départ (journal `kesh-gate-logs/e354/rouges.log`) ; AC3 vert avant et après. Le mock d'`i18nMsg` des tests du composant
rend un catalogue (le repli ne doit pas masquer une clé absente) — aucun état i18n global n'est en jeu côté frontend. E2E
de AC9, rouge sur le binaire de départ.

**Mutations** : M1 `guard` retiré de *Rejeter* (AC1) ; M2 *Rejeter* gardé même sans ligne signalée (AC2) ; M3 l'intro
d'écriture rendue pour *Rejeter* (AC1) ; M4 `if (!current) return;` sans avis (AC4) ; M5 l'avis posé dans `errorMsg` (AC6) ;
M6 l'avis non effacé au geste suivant (AC4).

## Tâches

- [ ] Étape 0 : relire les lignes citées ; vérifier qu'un relevé **saisi à la main** (#816) se retrouve aussi au détail
      d'un import, pour que « *Annuler le rejet* » y soit vrai (sinon, adapter la phrase de D2 et le dire au registre).
- [ ] Tests, rouges consignés ; composant, dialogue, clés ×4 ; manuel ; CHANGELOG ; captures ; E2E.
- [ ] Gates : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; backend complet
      (`test-fast.sh --ci`, les catalogues changent : `parity_between_locales`) ; E2E complet au dernier commit de code.

## Dev Notes

### Hors périmètre

- **Nommer l'écriture rapprochée** à laquelle la ligne ressemble : il faudrait l'exposer dans `alreadyBooked` (serveur,
  contrat d'API) ; à demander en issue si utile.
- **#1034** (écriture rapprochée des deux côtés), **#76**, **#993** : autres stories.

### Fichiers communs — règle d'union C-e350-8

`messages.ftl` ×4 (ici à côté de `reconciliation-labels-already-booked-confirm-*`), `user-manual.tex` (ici `:3107`),
`CHANGELOG.md`. Aucune autre story de l'epic ne touche `features/reconciliation/`.

### Ce qui doit rester vrai

- Rien n'est refusé ni fait d'office : le dialogue propose, l'utilisateur décide (« un appariement automatique propose, il
  ne crée jamais »). Aucun appel serveur nouveau. Fixtures synthétiques seulement.

### Références

- Issues #1023, #1024 (et son commentaire C4-1) ; Story 34-4 (#828), 34-11b ; Story 28-5b (*Annuler le rejet*).
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35). Validation non lancée.
