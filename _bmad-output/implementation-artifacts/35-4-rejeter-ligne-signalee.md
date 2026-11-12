# Story 35-4 : *Rejeter* une ligne signalée « déjà comptabilisée » demande confirmation ; un geste confirmé sur une ligne partie ne se perd plus en silence (#1023, #1024)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration**, aucun changement serveur ni d'API. Code lu sur
**`release/0.17.6`** (`4e1c768a` ; `5e2a5818` ne diffère que d'une ligne du manuel, `user-manual.tex:2498` : les numéros
cités tiennent), qui porte les stories 34-4 (#828) et 34-11b. La branche de la story part de `release/0.17.6` (le worktree
du kickoff est sur `main`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -12, -13, -18) ; `e35-choix-autonomes-e354.md`
pour la story, à créer au premier choix. Validation P1 remédiée (Change Log).*

## Story

En tant que titulaire qui réconcilie ses relevés, quand une transaction est signalée « déjà comptabilisée », je veux qu'un
clic sur *Rejeter* me demande de confirmer, en me disant ce que le rejet fait, afin qu'un encaissement réel ne quitte pas la
liste sans que je l'aie voulu.

## Ce qui se lit au code (`release/0.17.6`)

Tout est dans `frontend/src/lib/features/reconciliation/` :

- **La garde existe pour trois gestes, pas pour *Rejeter*.** En-tête du composant (`ReconciliationProposals.svelte:44-48`) ;
  doc du type (`:283-287`, « Seuls *Accepter*, *Écriture manuelle* et *Éclater* passent par là : ils passeraient une
  écriture ») ; type `GuardedGesture` (`:288-291`) ; `guard` (`:296-303`) ; `runGesture` (`:305-309`, qui retombe sur
  *Éclater* par un `else`, `:308`) ; `requestAccept` (`:312-317`). `onReject` (`:428-445`) est branché **directement** sur le
  bouton (`onclick={onReject}`, `:485`).
- **Le signal** : `isAlreadyBooked(p)` (`already-booked.ts:108-110`) ; son en-tête (`:6-9`) ne nomme que les trois gestes. Le
  texte conseille de *rattacher* la ligne (`:47-102`). `AlreadyBooked` ne porte que des **nombres**
  (`reconciliation.types.ts:77-81` : `settlementCount`, `entryCount`, `matchedElsewhereCount`).
- **Le dialogue** (`AlreadyBookedConfirmDialog.svelte`, 65 lignes) : son en-tête (`:1-14`) écarte *Rejeter* « que
  l'avertissement recommande » (`:9-10`) — faux depuis la 34-11b ; son intro (clé `reconciliation-labels-already-booked-
  confirm-intro`, fr-CH `messages.ftl:1458`) parle d'une écriture passée deux fois. Composant bits-ui : focus piégé
  (`dialog-content.svelte:24`, `trapFocus = true`), clic hors du dialogue = fermeture (`dismissible-layer.svelte:8`,
  `interactOutsideBehavior = "close"`), bouton X (`showCloseButton = true`) ; toute fermeture vaut *Annuler*.
- **#1024 — le silence.** `confirmGesture` (`:336-353`) sort par `if (!current) return;` (`:347`) ; `load()` vide la sélection
  (`:170`) et `onAccept` sort dès `selected.size === 0` (`:374`) ; `onReject` aussi (`:429`). **Mais dans un navigateur**, la
  liste ne se relit pas pendant que la confirmation est ouverte (V1-1 de la validation P1, établi par lecture) : `load()`
  n'est relancé que par l'`$effect` qui suit `limit` et `bankAccountId` (`:180-182`) ; *Afficher aussi les plus anciennes*
  (`:189-191`) est **sous** la modale, qui piège le focus et se ferme au clic hors d'elle ; `bankAccountId` est remonté par
  `{#key}`. Le test qui fonde #1024 (`ReconciliationProposals.test.ts:1767-1768`) clique derrière la modale, ce que jsdom
  permet. Le cas « autre onglet » de l'issue ne retire pas la ligne de la liste du client : le geste part et le serveur
  répond par `failed[]`, déjà affiché (`user-manual.tex:3111`). **#1024 n'est donc atteignable qu'en jsdom, ou par un futur
  déclencheur de relecture** — à constater à l'étape 0 dans un navigateur.
- **Où les messages s'affichent** : `errorMsg` remplace la liste (branche `{:else if errorMsg}`, `:461`) ; la liste, les
  boutons et le dialogue vivent dans la branche `{:else}` de `{#if loading}…` (`:457-801`) ; `reconciliation-success` est
  hors de cette chaîne (`:451-456`) ; une région `role="status"` présente dès le rendu existe déjà pour un autre avis
  (`:502-509`, C-29-9-21).
- **Un rejet se défait** : *Annuler le rejet* (`UnrejectTransactionAction.svelte`, Story 28-5b), au détail de l'import
  (`routes/(app)/bank-import/[id]/+page.svelte:207`) — y compris pour un relevé **saisi à la main**, qui a son détail d'import
  (`:30`, `isManualImport` ; `reconciliation-already-booked.spec.ts:120-124` ; `user-manual.tex:3097`). Rejet et annulation
  sont du même rôle (`lib.rs:882`, `:909`, `:1013`).
- **Tests existants** : `ReconciliationProposals.test.ts` — le mock d'`i18nMsg` rend le **repli** (`:62-67`) ;
  `AC-6 Rejeter une ligne signalée ⇒ aucune confirmation (M6)` (`:1737-1743`) ; `AC-6 Continuer ne rouvre pas le geste d'une
  ligne sortie de la liste` (`:1757-1781`) ; les Vitest qui rejettent (`:310`, `:363`, `:432`, `:741`, `:1081`) ont
  `alreadyBooked: null` (`:121`, `:141`). `already-booked-i18n.test.ts` tient une **liste fermée** des clés du préfixe
  `reconciliation-labels-already-booked-` (`CLES`, `:25-40` ; `toEqual([...CLES].sort())`, `:51-53`) et compare les replis au
  fr-CH (`:47`) ; son commentaire dit « Les huit clés » (`:12`) et son test « les dix clés » (`:51`).
  `frontend/src/lib/shared/i18n-keys.test.ts:972` : `sitesTotal: 3026`, compte exact des sites `i18nMsg(`.
- **E2E qui cliquent *Rejeter*** (`git grep -n reconciliation-reject-btn frontend/tests/e2e`) :
  `reconciliation-deleted-import.spec.ts:166` (montants 100.00 à 499.99, `:120`) et `reconciliation-reject-unmatched.spec.ts:166`
  (90 000 à 98 999.99, `:94`), sur le compte partagé `CH4431999123000889012`. E2E de la 34-4 :
  `reconciliation-already-booked.spec.ts:72` (finit par un rattachement, `:175-188` ; nettoyage `:194-195`).
- **Manuel** : `user-manual.tex:3097` (puce *Rejeter*, sans confirmation) et `:3107` (« sans message (issue \#1024) » ;
  « *Rejeter* et *Rattacher à une écriture existante* ne demandent rien »).

## Décisions (C-e350-12, C-e350-13, C-e350-18)

- **D1 — *Rejeter* passe par la garde**, comme les trois autres gestes, **si la sélection contient une ligne signalée** ;
  sinon le rejet part aussitôt, comme aujourd'hui. *Rattacher à une écriture existante* reste hors garde : c'est le geste que
  l'avertissement recommande. `runGesture` devient un `switch` exhaustif (`never`), sans repli sur *Éclater* (V1-15).
- **D2 — Le dialogue dit le geste et sa portée.** `AlreadyBookedConfirmDialog` reçoit le geste ; pour *Rejeter*, une intro
  et un bouton propres :
  - intro : « Chaque transaction ci-dessous semble déjà comptabilisée. *Rejeter* retire **les { $count } transactions
    cochées** de la liste sans rien écrire : si l'une est un mouvement réel qui n'est pas encore aux livres, il y manquera ;
    si elle y est déjà, rattachez-la plutôt. Un rejet s'annule depuis le détail de son import (*Annuler le rejet*). » — sans
    sélecteur : deux clés, `-one` et `-other`, comme `already-booked.ts` le fait (V1-11, V1-14) ;
  - bouton : *Rejeter quand même*.
  Il nomme les transactions signalées (`describeTransaction`), non l'écriture à laquelle elles ressemblent : le serveur n'en
  donne que le nombre. Cette part de #1023 part en issue (C-e350-18).
- **D3 — #1024 : une garde défensive, sans promesse inatteignable** (C-e350-13, révisé par C-e350-18). Si la confirmation
  trouve la ligne absente de la liste courante, ou la sélection vide, un **avis** s'affiche : « Rien n'a été fait : la liste a
  changé depuis l'ouverture de la confirmation ; vérifiez-la avant de recommencer. » — sans nommer de cause, que le client
  ne connaît pas. Une seule forme (aucun lot partiel : une relecture vide toute la sélection, `:170`). Place : **hors** de la
  chaîne `{#if loading}`, sous `reconciliation-success`, dans une région `role="status"` présente dès le rendu (sur le modèle
  de `:502-509`). Il s'efface à **tout** geste de la rangée ou d'une ligne (case à cocher, *Accepter*, *Rejeter*, *Écriture
  manuelle*, *Éclater*, *Rattacher*, *Afficher aussi les plus anciennes*) et à toute relecture `load()`. Testé en Vitest ;
  l'étape 0 constate dans un navigateur si un chemin réel y mène (registre `e354`) : **s'il n'y en a pas**, aucune capture de
  l'avis, et le manuel ne le décrit pas (il retire seulement la phrase du silence) ; la PR dit que #1024 se ferme par une garde
  défensive.
- **D4 — Clés**, toutes sous le préfixe `reconciliation-labels-already-booked-` (le test de liste fermée et la comparaison des
  replis les couvrent) : `…-confirm-intro-reject-one`, `…-confirm-intro-reject-other`, `…-confirm-reject`, `…-stale`.

## Critères d'acceptation

1. **AC1 — *Rejeter*, sélection avec une ligne signalée** (Vitest) : le dialogue s'ouvre, nomme la ligne signalée (elle seule
   si d'autres, non signalées, sont cochées), avec l'intro et le bouton de D2 (le compte des cochées dedans) ; *Annuler*,
   Échap, le X et le clic hors du dialogue n'appellent pas `rejectProposals` ; *Rejeter quand même* l'appelle avec **toute**
   la sélection.
2. **AC2 — *Rejeter*, sélection sans ligne signalée** : aucun dialogue ; `rejectProposals` appelé aussitôt — non-régression,
   **verte au départ**, tuée par M2.
3. **AC3 — Les trois autres gestes** gardent leur dialogue et son intro d'écriture (tests de la 34-4 verts).
4. **AC4 — Garde défensive, *Écriture manuelle* et *Éclater*** (Vitest, montage de `:1757`) : aucune modale, l'avis de D3
   visible ; il s'efface au geste suivant — un test par famille de geste (rangée, ligne, *Afficher aussi*, relecture).
5. **AC5 — Garde défensive, *Accepter* et *Rejeter*** (Vitest) : sélection vidée pendant la confirmation → aucun appel au
   serveur, l'avis visible.
6. **AC6 — Place de l'avis** : il est hors de la branche de la liste et reste visible pendant un chargement ; sa région
   `role="status"` existe avant qu'il paraisse. Rouge au départ par l'assertion de présence de l'avis.
7. **AC7 — Quatre langues et gardes i18n.** Les clés de D4 dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, à côté de
   `reconciliation-labels-already-booked-confirm-*` (fr-CH `:1457-1460`, autres `:1388-1391`) ; **`CLES`** de
   `already-booked-i18n.test.ts` étendu (obligatoire), son commentaire (`:12`) et son titre (`:51`) recomptés ; replis égaux
   au fr-CH par ce test ; `i18n-keys.test.ts` (`sitesTotal`) recompté à la source (C-e350-8).
8. **AC8 — Sites de l'ancien contrat**, recomptés par
   `git grep -nE 'Écriture manuelle\*? et \*?Éclater|Rejeter\*? et \*?Rattacher|dix clés|huit clés' -- frontend/src/lib/features/reconciliation docs/manual/fr/user-manual.tex`,
   chacun corrigé ou laissé avec sa raison au Change Log : au moins `ReconciliationProposals.svelte:44-48`, `:283-287`,
   `:326-331` ; `already-booked.ts:6-9` ; `AlreadyBookedConfirmDialog.svelte:1-14` ; `already-booked-i18n.test.ts:12`, `:51` ;
   `user-manual.tex:3097`, `:3107`. Tests existants : `:1737-1743` devient AC1 ; `:1757-1781` asserte en plus l'avis (AC4).
9. **AC9 — E2E.**
   - **Neuf** : un test **distinct** de celui de la 34-4, sur le même montage (`reconciliation-already-booked.spec.ts`,
     données synthétiques) : ligne signalée cochée, *Rejeter* → dialogue ; *Annuler* → la ligne reste ; *Rejeter quand même*
     → elle quitte la liste. Nettoyage : l'écriture reste non rattachée, donc contre-passée comme aujourd'hui (`:194-195`) ;
     l'import portant une ligne rejetée se supprime (`bank_imports.rs:434`, `rejected_count`).
   - **Existants qui rejettent** (V1-6) : `reconciliation-deleted-import.spec.ts:166` et
     `reconciliation-reject-unmatched.spec.ts:166` **assertent d'abord l'absence** d'`already-booked-warning` sur leurs lignes,
     pour échouer lisiblement si une écriture d'une autre spec les signale, plutôt que de rester bloqués sur le dialogue.
10. **AC10 — Manuel et CHANGELOG.** `user-manual.tex:3097` (puce *Rejeter* : confirmation sur une ligne signalée) et `:3107`
    (*Rejeter* demande confirmation ; la phrase « sans message (issue \#1024) » **retirée**, sans décrire l'avis si l'étape 0
    ne trouve aucun chemin réel ; la limite de D2 écrite, avec le numéro de l'issue). PDF régénéré, contrôlé aplati.
    CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8) ; celui de #1023 cite l'issue de D2.
11. **AC11 — Captures** (thème clair, base jetable, `kesh-gate-logs/e354/captures/`) : le dialogue de *Rejeter*. L'avis de D3
    seulement si l'étape 0 en trouve un chemin réel.

## Tests à écrire, et la preuve du rouge

Vitest de `ReconciliationProposals.test.ts` (AC1, AC4, AC5, AC6) et du dialogue : rouges sur le code de départ (journal
`kesh-gate-logs/e354/rouges.log`) ; **AC2 et AC3 sont des non-régressions, vertes avant et après**. Le mock d'`i18nMsg` du
composant rend le **repli** (`:62-67`) : les textes s'y lisent sur les replis ; l'égalité des replis au catalogue tient par
`already-booked-i18n.test.ts`, l'existence des clés par `i18n-keys.test.ts` — ce sont les deux filets (V1-3). Aucun état
i18n global côté frontend. E2E de AC9, rouge sur le binaire de départ.

**Mutations** : M1 `guard` retiré de *Rejeter* (AC1) ; M2 *Rejeter* gardé même sans ligne signalée (AC2) ; M3 l'intro
d'écriture rendue pour *Rejeter* (AC1, lue sur le repli) ; M4 `if (!current) return;` sans avis (AC4) ; M5 l'avis posé dans
`errorMsg` ou dans la branche de la liste (AC6) ; M6 l'avis non effacé à un geste (AC4, une par famille) ; M7 l'avis absent
pour *Accepter*/*Rejeter* à sélection vide (AC5) ; M8 `rejectProposals` appelé avec les seules lignes signalées (AC1).

## Tâches

- [ ] Étape 0 : relire les lignes citées sur `release/0.17.6` ; **constater dans un navigateur** si un chemin relit la liste
      pendant la confirmation (D3) et l'écrire au registre `e354`.
- [ ] Tests, rouges consignés ; composant, dialogue, clés ×4 et gardes i18n ; sites de l'ancien contrat ; E2E ; manuel ;
      CHANGELOG ; captures.
- [ ] Gates : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; backend complet
      (`test-fast.sh --ci`, les catalogues changent : `parity_between_locales`) ; E2E complet au dernier commit de code.

## Dev Notes

### Hors périmètre

- **Nommer l'écriture ressemblante** dans le dialogue : issue à ouvrir par l'orchestrateur (C-e350-18), citée au CHANGELOG et
  dans la PR ; #1023 se ferme avec cette limite dite.
- **#1034**, **#76**, **#993** : autres stories.

### Fichiers communs — règle d'union C-e350-8

`messages.ftl` ×4 (ici à côté de `reconciliation-labels-already-booked-confirm-*`), `user-manual.tex` (ici `:3097`, `:3107`),
`CHANGELOG.md`, `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`, recompté à la source). Aucune autre story de l'epic
ne touche `features/reconciliation/` ni les specs E2E de la réconciliation.

### Ce qui doit rester vrai

- Rien n'est refusé ni fait d'office : le dialogue propose, l'utilisateur décide (« un appariement automatique propose, il ne
  crée jamais »). Aucun appel serveur nouveau. Fixtures synthétiques seulement.

### Références

- Issues #1023, #1024 (et son commentaire C4-1) ; Story 34-4 (#828), 34-11b ; Story 28-5b (*Annuler le rejet*) ;
  validation P1 `kesh-gate-logs/e35-4-validation-p1.md`.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Priorités des défauts* (« on trace
  toujours »).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35). Validation non lancée.
- 2026-11-12 — **Validation P1** (Opus) : 0 CRITICAL, 0 HIGH, 9 MEDIUM, 6 LOW (`kesh-gate-logs/e35-4-validation-p1.md`).
  Prémisses relues sur `release/0.17.6` ; aucune réfutée, V1-1 tenu pour établi **par lecture** et renvoyé à l'étape 0 pour
  le constat dans un navigateur. Appliqués :
  - V1-1 : #1024 réduit à une garde défensive testée en Vitest ; ni capture ni description de l'avis sans chemin réel ;
    l'avis ne nomme pas de cause (D3, C-e350-18) ;
  - V1-2 : clés nommées sous le préfixe (D4), forme unique, cas partiel retiré, `CLES` étendu obligatoirement ;
  - V1-3 : le mock rend le repli ; les deux filets nommés ;
  - V1-4 : AC2 non-régression verte ; M7, M8 ;
  - V1-5 : liste fermée des gestes qui effacent l'avis, place hors de la branche de la liste, région `role="status"` ;
  - V1-6 : les deux E2E qui rejettent assertent l'absence de signal ;
  - V1-7 : `i18n-keys.test.ts` (`sitesTotal`) ajouté à la règle d'union, pour toutes les stories (registre) ;
  - V1-8 : l'écriture ressemblante part en issue (C-e350-18, texte rédigé pour l'orchestrateur) ;
  - V1-9 : inventaire recompté des sites de l'ancien contrat (AC8), `:3097` au manuel ;
  - LOW : V1-10 (relevé manuel au détail d'import : vérifié, étape 0 allégée) ; V1-11 (X, clic hors, portée du lot) ;
    V1-12 (E2E distinct, nettoyage) ; V1-13 (commit, branche de départ) ; V1-14 (« rattachez-la plutôt ») ; V1-15 (`switch`
    exhaustif).
  - Symptômes greppés dans la fiche, le registre et le fichier d'epic : « un geste confirmé sur une ligne partie le dit »,
    « l'avis nomme les autres », « rapprochée, rejetée ou modifiée », « rend un catalogue » — remplacés.
