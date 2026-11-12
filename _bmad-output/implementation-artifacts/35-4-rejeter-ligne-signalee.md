# Story 35-4 : *Rejeter* une ligne signalée « déjà comptabilisée » demande confirmation ; une confirmation ne survit plus à une relecture de la liste (#1023, #1024)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration**, aucun changement serveur ni d'API. Code lu sur
**`release/0.17.6`** (`4e1c768a` ; `5e2a5818` ne diffère que d'une ligne du manuel, `user-manual.tex:2498` : les numéros
cités tiennent), qui porte les stories 34-4 (#828) et 34-11b. La branche de la story part de `release/0.17.6` (le worktree
du kickoff est sur `main`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -12, -13, -18, -20) ;
`e35-choix-autonomes-e354.md` pour la story, à créer au premier choix. Validations P1 et P2 remédiées (Change Log).*

## Story

En tant que titulaire qui réconcilie ses relevés, quand une transaction est signalée « déjà comptabilisée », je veux qu'un
clic sur *Rejeter* me demande de confirmer, en me disant ce que le rejet fait, afin qu'un encaissement réel ne quitte pas la
liste sans que je l'aie voulu ; et je veux qu'une confirmation ne puisse jamais être validée sur des lignes qui ne sont plus
celles de la liste.

## Ce qui se lit au code (`release/0.17.6`)

Tout est dans `frontend/src/lib/features/reconciliation/` :

- **La garde existe pour trois gestes, pas pour *Rejeter*.** En-tête du composant (`ReconciliationProposals.svelte:44-48`) ;
  doc du type (`:283-287`) ; type `GuardedGesture` (`:288-291`) ; `guard` (`:296-303`) ; `runGesture` (`:305-309`, repli sur
  *Éclater* par un `else`, `:308`) ; `requestAccept` (`:312-317`) ; `confirmGesture` (`:336-353` : *Accepter* s'exécute
  aussitôt, `:343-346` ; *Écriture manuelle* et *Éclater* relisent leur ligne et sortent par `if (!current) return;`, `:347`).
  `onReject` (`:428-445`) est branché **directement** sur le bouton (`onclick={onReject}`, `:485`). Tous ces boutons sont
  `disabled={busy}` ; seules les cases à cocher ne le sont pas.
- **Le signal** : `isAlreadyBooked(p)` (`already-booked.ts:108-110`) ; son en-tête (`:6-9`) ne nomme que les trois gestes.
  `AlreadyBooked` ne porte que des **nombres** (`reconciliation.types.ts:77-81`).
- **Le dialogue** (`AlreadyBookedConfirmDialog.svelte`, 65 lignes) : en-tête `:1-14` (écarte *Rejeter* « que l'avertissement
  recommande », `:9-10`, faux depuis la 34-11b) ; intro d'écriture (clé `…-confirm-intro`, fr-CH `messages.ftl:1458`) ;
  boutons `already-booked-confirm-cancel` / `already-booked-confirm-continue`. Composant bits-ui : focus piégé, clic hors =
  fermeture, bouton X ; toute fermeture vaut *Annuler*. Il vit **dans** la branche `{:else}` de la liste (`:796-800`,
  `open={pendingGesture !== null}`).
- **#1024 — la cause du silence.** `load()` (`:159-178`) ne touche ni `pendingGesture` ni `confirmTransactions` et vide la
  sélection (`:170`) ; `loading = true` démonte la branche de la liste, dialogue compris, puis la remonte **ouvert**, avec des
  transactions périmées ; *Continuer* ne fait alors rien (`:347`, `:374`, `:429`). Le test `:1757-1781` joue ce chemin en
  cliquant derrière la modale (jsdom). Dans un navigateur, d'après le code, la liste ne se relit pas pendant la confirmation :
  `load()` n'a pour appelants que l'`$effect` (`limit`, `bankAccountId`, ce dernier remonté par `{#key}`), `showOlder` (sous la
  modale) et les fins de gestes (`:240`, `:252`, `:280`, `:420`, `:439`), pendant lesquelles les boutons sont `disabled={busy}` ;
  aucun minuteur ni écouteur de visibilité. Le cas « autre onglet » aboutit à `failed[]` côté serveur
  (`RECONCILIATION_ALREADY_REJECTED`, `reconciliation.rs:1540`), joué en navigateur par
  `reconciliation-deleted-import.spec.ts:152-166`.
- **Un rejet se défait** : *Annuler le rejet*, au détail de l'import (`routes/(app)/bank-import/[id]/+page.svelte:207`), relevé
  saisi à la main compris (`:30`, `isManualImport`). Rejeter écrit `auto_match_rejected_at` et une trace d'audit, mais **ne
  passe aucune écriture** ; le manuel dit « sans la passer en comptabilité » (`user-manual.tex:3097`). Les libellés du bouton
  de la rangée : fr « Rejeter », de « Ablehnen », it « Rifiutare », en « Reject » (`reconciliation-actions-reject`).
- **Tests existants** : `ReconciliationProposals.test.ts` — le mock d'`i18nMsg` rend le **repli** (`:62-67`) ; `:1737-1743`
  (« aucune confirmation (M6) ») ; `:1757-1781` (la ligne sortie) ; `:1533` (commentaire). `already-booked-i18n.test.ts` : liste
  fermée `CLES` (`:25-40`, `:51-53`), comparaison des replis (`:47`), `{ $count }` exigé pour les `-other` (`:60`) ; « huit
  clés » (`:12`), « dix clés » (`:51`). `i18n-keys.test.ts:972` : `sitesTotal: 3026` ; `i18n-entrees-a-variables.test.ts` exige
  des clés littérales.
- **E2E qui cliquent *Rejeter*** : `reconciliation-deleted-import.spec.ts:166` et `reconciliation-reject-unmatched.spec.ts:166`
  (compte partagé `CH4431999123000889012`). E2E de la 34-4 : `reconciliation-already-booked.spec.ts:72` (rattachement
  `:175-188`, nettoyage `:194-195`). `rejected_count` : `crates/kesh-db/src/repositories/bank_imports.rs:435`, calcul `:610`.
- **Manuel** : `user-manual.tex:3097` (puce *Rejeter*, à compléter) ; `:3107` (« sans message (issue \#1024) » ; « *Rejeter* et
  *Rattacher à une écriture existante* ne demandent rien »).

## Décisions (C-e350-12, -18, -20)

- **D1 — *Rejeter* passe par la garde** si la sélection contient une ligne signalée ; sinon le rejet part aussitôt. Un
  `requestReject` remplace `onclick={onReject}` (`:485`) ; `GuardedGesture` gagne `{ kind: 'reject' }` ; `confirmGesture`
  l'exécute aussitôt, comme *Accepter* ; `runGesture` devient un `switch` exhaustif (`never`), sans repli. *Rattacher* reste
  hors garde.
- **D2 — Le dialogue dit le geste et sa portée.** Il reçoit `gesture: 'write' | 'reject'` et `selectedCount` (le nombre de
  cochées, pris au moment de `guard`, comme `confirmTransactions`). Pour *Rejeter*, en texte brut (le catalogue n'a pas de
  balisage) :
  - intro : « Chaque transaction ci-dessous semble déjà comptabilisée. Rejeter retire les { $count } transactions cochées de
    la liste sans passer d'écriture : si l'une est un mouvement réel qui n'est pas encore aux livres, il y manquera ; si elle y
    est déjà, rattachez-la plutôt (« Rattacher à une écriture existante »). Un rejet s'annule depuis le détail de son import
    (« Annuler le rejet »). » — deux clés, `-one` (sans `{ $count }`, « la transaction cochée ») et `-other` ;
  - bouton : « Rejeter quand même » (de « Trotzdem ablehnen », it « Rifiuta comunque », en « Reject anyway ») ; **même**
    `data-testid` que pour les autres gestes, `already-booked-confirm-continue`.
  Il ne nomme pas l'écriture ressemblante : **#1052**.
- **D3 — #1024 : une relecture referme la confirmation** (C-e350-20, remplace l'avis de C-e350-18). `load()` pose
  `pendingGesture = null` en tête ; `confirmTransactions` n'est pas vidé (le dialogue garde sa liste pendant sa transition de
  fermeture). Une confirmation ouverte sur des lignes périmées **ne peut jamais** être validée ; il n'y a plus de clic sans
  effet à signaler, donc **aucun avis**, aucune clé, aucune région `role="status"`. `if (!current) return;` (`:347`) et les
  gardes `selected.size === 0` restent, défensives. Les findings de l'avis (V2-1, V2-9) sont sans objet.
- **D4 — Clés**, sous le préfixe `reconciliation-labels-already-booked-` : `…-confirm-intro-reject-one`,
  `…-confirm-intro-reject-other`, `…-confirm-reject`. Trois sites `i18nMsg` littéraux : `sitesTotal` passe de 3026 à
  **3029**, recompté à la source (C-e350-8, C-e350-18).
- **D5 — Fermeture de #1024** : la PR porte `refs #1024` ; l'orchestrateur ferme l'issue à la main avec ce commentaire :
  > Livré dans la 0.17.7 (story 35-4) : une relecture de la liste referme la confirmation « déjà comptabilisée » ouverte, si
  > bien qu'elle ne peut plus être validée sur des lignes qui ont changé (`load()` pose `pendingGesture = null`). Le clic sans
  > effet ni message n'est donc plus possible. Constat de l'étape 0 : dans un navigateur, la liste ne se relit pas pendant
  > que la confirmation est ouverte (modale qui piège le focus, clic hors = *Annuler*) ; le chemin de l'issue n'était
  > atteignable qu'en test (jsdom). Le cas « autre onglet » est rendu par le serveur dans `failed[]`, comme avant. Le même
  > effet sur *Accepter* (commentaire C4-1) est couvert par le même changement.

## Critères d'acceptation

1. **AC1 — *Rejeter*, sélection avec une ligne signalée** (Vitest) : le dialogue s'ouvre, nomme la ligne signalée (elle
   seule si d'autres, non signalées, sont cochées), avec l'intro et le bouton de D2 et le nombre de **cochées** (`-one` si
   une, `-other` sinon) ; *Annuler*, Échap, le X et le clic hors n'appellent pas `rejectProposals` ; *Rejeter quand même*
   l'appelle avec **toute** la sélection.
2. **AC2 — *Rejeter*, sélection sans ligne signalée** : aucun dialogue ; `rejectProposals` appelé aussitôt — non-régression,
   **verte au départ**, tuée par M2.
3. **AC3 — Les trois autres gestes** gardent leur dialogue et son intro d'écriture (tests de la 34-4 verts).
4. **AC4 — Une relecture referme la confirmation** (Vitest, montage de `:1757-1781` réécrit) : confirmation ouverte, relecture
   de la liste → le dialogue n'est plus ouvert, aucune modale ni aucun appel serveur ; vrai pour les quatre gestes gardés.
   Rouge au départ : le dialogue réapparaît ouvert après la relecture.
5. **AC5 — Quatre langues et gardes i18n.** Les clés de D4 dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, à côté de
   `…-confirm-*` (fr-CH `:1457-1460`, autres `:1388-1391`), sans balisage ; `CLES` étendu (obligatoire), « huit clés » (`:12`) et
   « dix clés » (`:51`) recomptés ; l'assertion de `{ $count }` (`:60`) couvre `…-confirm-intro-reject-other` ; replis égaux au
   fr-CH ; `sitesTotal` à 3029, recompté.
6. **AC6 — Sites de l'ancien contrat**, recomptés par la commande vérifiée en validation P2 :
   `git grep -nE 'ne demandent rien|sans message|ne passent pas par lui|passeraient une écriture|Rejeter\*? et \*?Rattacher|\\emph\{Écriture manuelle\} et|dix clés|huit clés' -- frontend/src/lib/features/reconciliation docs/manual/fr/user-manual.tex`
   (elle rend `user-manual.tex:3107`, `AlreadyBookedConfirmDialog.svelte:9-10`, `ReconciliationProposals.svelte:47`, `:286`, et les
   deux tests), plus, que la commande ne voit pas : `user-manual.tex:3097` (à **compléter**), `ReconciliationProposals.svelte:326-331`
   (doc de `confirmGesture`, sur deux lignes), `ReconciliationProposals.test.ts:1533` et le commentaire de `:805` (« tous les refus de
   *Rejeter* »), `already-booked.ts:6-9`. Chacun corrigé ou laissé avec sa raison au Change Log.
7. **AC7 — E2E.**
   - **Neuf**, distinct de celui de la 34-4, même montage : ligne signalée cochée, *Rejeter* → dialogue ; *Annuler* → la ligne
     reste ; *Rejeter quand même* → elle quitte la liste. Nettoyage : l'écriture reste non rattachée, contre-passée comme
     aujourd'hui (`:194-195`) ; l'import portant une ligne rejetée se supprime (`rejected_count`,
     `kesh-db/src/repositories/bank_imports.rs:435`, `:610`).
   - **Existants qui rejettent** : `reconciliation-deleted-import.spec.ts:166` et `reconciliation-reject-unmatched.spec.ts:166`
     assertent d'abord l'absence d'`already-booked-warning` sur leurs lignes.
8. **AC8 — Manuel et CHANGELOG.** `user-manual.tex:3097` (la confirmation sur une ligne signalée) ; `:3107` (*Rejeter* demande
   confirmation ; « sans message (issue \#1024) » remplacé par : une relecture referme la confirmation) ; la limite de D2 avec
   #1052. PDF régénéré ; contrôle au PDF aplati : absence de « sans message (issue #1024) », présence de « Rejeter quand même ».
   CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*, un point par issue (C-e350-8) ; celui de #1023 cite #1052.
9. **AC9 — Captures** (thème clair, base jetable, `kesh-gate-logs/e354/captures/`) : le dialogue de *Rejeter*.

## Tests à écrire, et la preuve du rouge

Vitest de `ReconciliationProposals.test.ts` (AC1, AC4) et du dialogue : rouges au départ (journal
`kesh-gate-logs/e354/rouges.log`) ; AC2 et AC3 sont des non-régressions vertes. Le mock d'`i18nMsg` rend le **repli** : les
textes s'y lisent sur les replis ; l'égalité des replis au catalogue tient par `already-booked-i18n.test.ts`, l'existence des
clés par `i18n-keys.test.ts`. E2E de AC7, rouge sur le binaire de départ.

**Mutations** : M1 `guard` retiré de *Rejeter* (AC1) ; M2 *Rejeter* gardé même sans ligne signalée (AC2) ; M3 l'intro d'écriture
rendue pour *Rejeter* (AC1) ; M4 `pendingGesture = null` retiré de `load()` (AC4) ; M5 `rejectProposals` appelé avec les seules
lignes signalées (AC1) ; M6 `selected` entier passé à `confirmTransactions` (AC1, le dialogue nomme des lignes non signalées) ;
M7 le compte de l'intro pris sur les signalées (AC1) ; M8 `-one` et `-other` inversés (AC1) ; M9 la branche `reject` de
`runGesture` qui ouvre *Éclater* (AC1).

## Tâches

- [ ] Étape 0, sur `release/0.17.6` : relire les lignes citées ; **constat en navigateur** (Playwright) : confirmation ouverte,
      `reconciliation-show-older-btn.click({ timeout: 2000 })` échoue (« intercepts pointer events »), Tab ne sort pas du
      dialogue, Échap et clic hors referment ; consigner au registre `e354` (sert au commentaire de D5).
- [ ] Tests, rouges consignés ; composant, dialogue, clés ×4 et gardes i18n ; sites de l'ancien contrat ; E2E ; manuel ;
      CHANGELOG ; capture.
- [ ] Gates : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; backend complet
      (`test-fast.sh --ci`, les catalogues changent) ; E2E complet au dernier commit de code.

## Dev Notes

### Hors périmètre

- **#1052** : nommer l'écriture ressemblante (contrat d'API).
- **#1034**, **#76**, **#993** : autres stories.

### Fichiers communs — règle d'union C-e350-8

`messages.ftl` ×4 (ici à côté de `…-confirm-*`), `user-manual.tex` (ici `:3097`, `:3107`), `CHANGELOG.md`,
`frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`, recompté à la source). Aucune autre story ne touche
`features/reconciliation/` ni les specs E2E de la réconciliation.

### Ce qui doit rester vrai

- Rien n'est refusé ni fait d'office. Aucun appel serveur nouveau. Fixtures synthétiques seulement.

### Références

- Issues #1023, #1024 (et son commentaire C4-1), #1052 ; Story 34-4 (#828), 34-11b ; Story 28-5b ; validations P1 et P2
  `kesh-gate-logs/e35-4-validation-p*.md`.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Priorités des défauts*.

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
- 2026-11-12 — **Validation P2** (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW (`kesh-gate-logs/e35-4-validation-p2.md`).
  Série 9 → 3. Prémisses relues ; aucune réfutée (`load()` `:159-178` ne touche pas `pendingGesture` ; dialogue dans la branche
  de la liste `:796-800`). Appliqués :
  - V2-2 : **la relecture referme la confirmation** (D3, C-e350-20), solution la plus simple et la plus sûre : une
    confirmation périmée ne peut plus être validée ; l'avis de la P1 est retiré ; fermeture de #1024 à la main, commentaire
    écrit (D5), la PR porte `refs #1024` ;
  - V2-1 : sans objet (plus d'avis) ;
  - V2-3 : commande de recompte vérifiée, qui trouve le manuel ; sites qu'elle ne voit pas nommés à part (AC6) ;
  - LOW : V2-4 (protocole de l'étape 0) ; V2-5 (interface du dialogue, `selectedCount`, `kind: 'reject'`, testid réemployé,
    `requestReject`) ; V2-6 (texte brut, « sans passer d'écriture », termes des locales, `{ $count }` du `-other`) ; V2-7
    (mutations M6–M9) ; V2-8 (`:3097` au registre, `sitesTotal` 3029, `bank_imports.rs` du dépôt) ; V2-9 (sans objet).
  - Symptômes greppés (fiche, registre, fichier d'epic) : « avis », `role="status"`, « garde défensive », « `…-stale` » —
    restes retirés hors des entrées passées du Change Log et des registres annotés.
