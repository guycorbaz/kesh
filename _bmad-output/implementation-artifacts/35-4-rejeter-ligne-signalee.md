# Story 35-4 : *Rejeter* une ligne signalée « déjà comptabilisée » demande confirmation ; une confirmation ne survit plus à une relecture de la liste (#1023, #1024)

Status: in-progress

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**. **Aucune migration**, aucun changement serveur ni d'API. Code lu sur
**`release/0.17.6`** (`4e1c768a` ; `5e2a5818` ne diffère que d'une ligne du manuel, `user-manual.tex:2498` : les numéros
cités tiennent), qui porte les stories 34-4 (#828) et 34-11b. La branche de la story part de `release/0.17.6` (le worktree
du kickoff est sur `main`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -12, -13, -18, -20) ;
`e35-choix-autonomes-e354.md` pour la story, à créer au premier choix. Validations P1, P2 et P3 remédiées (Change Log).*

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
  transactions périmées ; *Continuer* ne fait alors rien (`:347`, `:374`, `:429`). Le test `:1757-1777` joue ce chemin en
  cliquant derrière la modale (jsdom). Dans un navigateur, d'après le code, la liste ne se relit pas pendant la confirmation :
  `load()` n'a pour appelants que l'`$effect` (`limit`, `bankAccountId`, ce dernier remonté par `{#key}`), `showOlder` (sous la
  modale) et les fins de gestes (`:240`, `:252`, `:280`, `:420`, `:439`), pendant lesquelles les boutons sont `disabled={busy}` ;
  aucun minuteur ni écouteur de visibilité.
- **Le cas « autre onglet »** (la ligne a changé ailleurs pendant que cette page était ouverte), **geste par geste**
  (validation P3, relu sur `reconciliation.rs`) :
  - *Accepter* et *Rejeter* sont des lots (`{ accepted|rejected, failed }`) : une ligne déjà rapprochée, déjà rejetée ou
    supprimée est refusée dans `failed[]` (`classify_batch_transaction`, `:1527-1545`, appelée par *Accepter* `:1743` et
    *Rejeter* `:4530`, `:4706` : `BANK_TRANSACTION_NOT_FOUND`,
    `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED` ; pour *Rejeter*, relecture verrouillée
    `:4697-4720`) et dite dans *Échecs partiels*. La spec `reconciliation-deleted-import.spec.ts:152-166` en joue **un** cas :
    *Rejeter* sur une transaction dont l'import a été supprimé (`BANK_TRANSACTION_NOT_FOUND`) ; aucune spec ne joue un rejet
    concurrent ;
  - *Écriture manuelle* et *Éclater* sont **unitaires** (`ManualMatchResponse`, `SplitResponse`,
    `reconciliation.types.ts:166-175` ; routes `/manual`, `post_manual_once` `:5036`, et `/split`, `post_split_once`
    `:5496`) : une ligne déjà rapprochée est refusée en conflit 409 (`find_strictly_pending_by_id_for_account` →
    `OptimisticLockConflict`, `:5066-5080` et `:5556-5563` ; `UPDATE … WHERE status = 'pending' AND version = ?`, `:5137-5164` et
    `:5601-5619`) et le refus s'affiche **dans la modale** (`ManualMatchModal.svelte:139-151`,
    `TransactionSplitModal.svelte:234-239`, `errorMsg = errorMessageOf(e)`) ; une ligne **rejetée** ailleurs n'est **pas**
    refusée : le geste passe son écriture et lève le rejet (`auto_match_rejected_at = NULL`, `:5140`, `:5605` ; tracé à
    l'audit, `was_previously_rejected`, `:5087`, `:5569`) — comportement d'aujourd'hui, que la story ne change pas.
- **Un rejet se défait** : *Annuler le rejet*, au détail de l'import (`routes/(app)/bank-import/[id]/+page.svelte:207`), relevé
  saisi à la main compris (`:30`, `isManualImport`). Rejeter écrit `auto_match_rejected_at` et une trace d'audit, mais **ne
  passe aucune écriture** ; le manuel dit « sans la passer en comptabilité » (`user-manual.tex:3097`). Les libellés du bouton
  de la rangée : fr « Rejeter », de « Ablehnen », it « Rifiutare », en « Reject » (`reconciliation-actions-reject`).
- **Tests existants** : `ReconciliationProposals.test.ts` — le mock d'`i18nMsg` rend le **repli** (`:62-67`) ; `:1737-1743`
  (« AC-6 Rejeter une ligne signalée ⇒ aucune confirmation (M6) » : rougit par construction après D1, **remplacé** par AC1 ; son
  « (M6) » est la mutation de la 34-4, non le M6 de cette fiche) ; `:1757-1777` (la ligne sortie ; le `describe` se ferme à
  `:1778`) ; `:1533` (commentaire). `already-booked-i18n.test.ts` : liste
  fermée `CLES` (`:25-40`, `:51-53`), comparaison des replis (`:47`), `{ $count }` exigé pour les `-other` (`:60`) ; « huit
  clés » (`:12`), « dix clés » (`:51`). `i18n-keys.test.ts:972` : `sitesTotal: 3026` ; `i18n-entrees-a-variables.test.ts` exige
  des clés littérales.
- **E2E qui cliquent *Rejeter*** : `reconciliation-deleted-import.spec.ts:166` et `reconciliation-reject-unmatched.spec.ts:166`
  (compte partagé `CH4431999123000889012`). E2E de la 34-4 : `reconciliation-already-booked.spec.ts:72` (rattachement
  `:175-188`, nettoyage `:194-195`). `rejected_count` : `crates/kesh-db/src/repositories/bank_imports.rs:435`, calcul `:610`.
- **Manuel** : `user-manual.tex:3049-3050` (« \emph{Accepter}, \emph{Écriture manuelle} et \emph{Éclater} demandent
  confirmation », trois gestes) ; `:3097` (puce *Rejeter*, à compléter) ; `:3107` (« sans message (issue \#1024) » ; « *Rejeter* et
  *Rattacher à une écriture existante* ne demandent rien »).

## Décisions (C-e350-12, -18, -20)

- **D1 — *Rejeter* passe par la garde** si la sélection contient une ligne signalée ; sinon le rejet part aussitôt. Un
  `requestReject` remplace `onclick={onReject}` (`:485`) ; `GuardedGesture` gagne `{ kind: 'reject' }` ; `confirmGesture`
  l'exécute aussitôt, comme *Accepter* ; `runGesture` devient un `switch` exhaustif (`never`), sans repli. *Rattacher* reste
  hors garde.
- **D2 — Le dialogue dit le geste et sa portée.** Il reçoit `gesture: 'write' | 'reject'` et `selectedCount` (le nombre de
  cochées, pris au moment de `guard`, comme `confirmTransactions`). Pour *Rejeter*, en texte brut (le catalogue n'a pas de
  balisage), avec l'apostrophe typographique `’` du bloc voisin (`…-confirm-intro`, fr-CH `messages.ftl:1458`) :
  - intro : « Chaque transaction ci-dessous semble déjà comptabilisée. Rejeter retire les { $count } transactions cochées de
    la liste sans passer d’écriture : si l’une est un mouvement réel qui n’est pas encore aux livres, il y manquera ; si elle y
    est déjà, rattachez-la plutôt (« Rattacher à une écriture existante »). Un rejet s’annule depuis le détail de son import
    (« Annuler le rejet »). » — deux clés, `-one` (sans `{ $count }`, « la transaction cochée ») et `-other`. Les deux libellés
    cités reprennent, **dans chaque locale**, ceux des clés `reconciliation-link-button-label` et
    `reconciliation-unreject-button` (fr-CH `:1557`, `:925` ; de, it, en `:1487`, `:872` : « Einer bestehenden Buchung
    zuordnen » / « Ablehnung aufheben », « Collega a una scrittura esistente » / « Annulla il rifiuto », « Link to an existing
    entry » / « Undo rejection ») ;
  - bouton : « Rejeter quand même » (de « Trotzdem ablehnen », it « Rifiuta comunque », en « Reject anyway ») ; **même**
    `data-testid` que pour les autres gestes, `already-booked-confirm-continue`.
  Il ne nomme pas l'écriture ressemblante : **#1052**.
- **D3 — #1024 : une relecture referme la confirmation** (C-e350-20, remplace l'avis de C-e350-18). `load()` pose
  `pendingGesture = null;` en tête, **sans lire `pendingGesture` auparavant** (pas de `if (pendingGesture) …`) : `load()` est
  appelé par un `$effect`, qui suit ce que la fonction lit avant son premier `await` (`:164-165`) ; une lecture ferait de
  chaque `guard` le déclencheur d'une relecture, qui refermerait aussitôt la confirmation (AC1 le verrait).
- **D3 (suite) — ce que la relecture laisse.** `confirmTransactions` n'est pas vidé (le dialogue garde sa liste pendant sa
  transition de fermeture). Une confirmation ouverte sur des lignes périmées **ne peut jamais** être validée ; il n'y a plus de clic sans
  effet à signaler, donc **aucun avis**, aucune clé, aucune région `role="status"`. `if (!current) return;` (`:347`) et les
  gardes `selected.size === 0` restent, défensives. Les findings de l'avis (V2-1, V2-9) sont sans objet.
- **D4 — Clés**, sous le préfixe `reconciliation-labels-already-booked-` : `…-confirm-intro-reject-one`,
  `…-confirm-intro-reject-other`, `…-confirm-reject`. Trois sites `i18nMsg` littéraux : `sitesTotal` passe de 3026 à
  **3029**, recompté à la source (C-e350-8, C-e350-18).
- **D5 — Fermeture de #1024** : la PR porte `refs #1024` ; l'orchestrateur ferme l'issue à la main avec ce commentaire,
  **à condition que le constat de l'étape 0 le confirme** ; s'il diverge, le commentaire n'est pas posté tel quel et l'écart
  revient à l'orchestrateur. Les phrases entre crochets se remplissent du constat consigné au registre `e354` :
  > Livré dans la 0.17.7 (story 35-4) : une relecture de la liste referme la confirmation « déjà comptabilisée » ouverte, si
  > bien qu'elle ne peut plus être validée sur des lignes qui ont changé (`load()` pose `pendingGesture = null`). Le clic sans
  > effet ni message n'est donc plus possible. [Constat de l'étape 0 : dans un navigateur, la liste ne se relit pas pendant
  > que la confirmation est ouverte — le bouton *Afficher aussi les plus anciennes* ne reçoit pas le clic, Tab ne sort pas du dialogue, Échap et
  > le clic hors valent *Annuler* ; le chemin de l'issue n'était atteignable qu'en test (jsdom).] Quand la ligne a changé
  > dans un autre onglet, le serveur garde ses réponses d'aujourd'hui, geste par geste : *Accepter* et *Rejeter* refusent la
  > ligne déjà rapprochée, déjà rejetée ou supprimée, et le disent dans *Échecs partiels* ; *Écriture manuelle* et *Éclater*
  > refusent la ligne déjà rapprochée et le disent dans leur fenêtre ; sur une ligne rejetée ailleurs, ces deux gestes passent
  > leur écriture et lèvent le rejet, tracé à l'audit. Le même effet sur *Accepter* (commentaire C4-1) est couvert par le
  > même changement.

## Critères d'acceptation

1. **AC1 — *Rejeter*, sélection avec une ligne signalée** (Vitest) : le dialogue s'ouvre, nomme la ligne signalée (elle
   seule si d'autres, non signalées, sont cochées), avec l'intro et le bouton de D2 et le nombre de **cochées** (`-one` si
   une, `-other` sinon) ; *Annuler*, Échap et le X n'appellent pas `rejectProposals` (Vitest, comme les tests de la 34-4,
   `:1699`, `:1729`) ; le **clic hors** se constate à l'étape 0 ou à l'E2E (AC7), aucun test de `features/reconciliation`
   ne le simulant en jsdom — toute fermeture passe par le seul `onOpenChange={(v) => !v && onCancel()}`
   (`AlreadyBookedConfirmDialog.svelte:31`) ; *Rejeter quand même* l'appelle avec **toute** la sélection.
2. **AC2 — *Rejeter*, sélection sans ligne signalée** : aucun dialogue ; `rejectProposals` appelé aussitôt — non-régression,
   **verte au départ**, tuée par M2.
3. **AC3 — Les trois autres gestes** gardent leur dialogue et son intro d'écriture (tests de la 34-4 verts, **hors**
   `:1737-1743`, remplacé par AC1).
4. **AC4 — Une relecture referme la confirmation** (Vitest, montage de `:1757-1777` réécrit) : confirmation ouverte, relecture
   de la liste → le dialogue n'est plus ouvert, aucune modale ni aucun appel **de geste** (`acceptProposals`,
   `rejectProposals`, modale *Écriture manuelle* ou *Éclater* ; le `getProposals` de la relecture est attendu) ; vrai pour
   les quatre gestes gardés. Rouge au départ, pour les trois gestes de la 34-4 : le dialogue réapparaît ouvert après la
   relecture ; pour *Rejeter*, rouge par l'absence du dialogue (AC1), et la preuve propre à D3 est M4.
5. **AC5 — Quatre langues et gardes i18n.** Les clés de D4 dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`, à côté de
   `…-confirm-*` (fr-CH `:1457-1460`, autres `:1388-1391`), sans balisage ; `CLES` étendu (obligatoire), « huit clés » (`:12`) et
   « dix clés » (`:51`) recomptés ; l'assertion de `{ $count }` (`:60`) couvre `…-confirm-intro-reject-other` ; replis égaux au
   fr-CH ; `sitesTotal` à 3029, recompté.
6. **AC6 — Sites de l'ancien contrat**, recomptés par la commande vérifiée en validation P2 :
   `git grep -nE 'ne demandent rien|sans message|ne passent pas par lui|passeraient une écriture|Rejeter\*? et \*?Rattacher|\\emph\{Écriture manuelle\} et|dix clés|huit clés' -- frontend/src/lib/features/reconciliation docs/manual/fr/user-manual.tex`
   **Sortie réelle** sur `release/0.17.6` (`5e2a5818`, relevée en validation P3), onze lignes :
   - **sites** : `user-manual.tex:3049` (l'énumération à trois gestes, `:3049-3050` : « \emph{Accepter}, \emph{Écriture
     manuelle} et \emph{Éclater} demandent confirmation » → y ajouter *Rejeter*) ; `user-manual.tex:3107` ;
     `AlreadyBookedConfirmDialog.svelte:9`, `:10` ; `ReconciliationProposals.svelte:47`, `:286` ;
     `already-booked-i18n.test.ts:12`, `:51` ;
   - **faux positifs, écartés** : `user-manual.tex:860` (« sans message », saisie d'une date) ; `user-manual.tex:2780` (« sans message »,
     lignes à ignorer d'un import CSV) ; `ReconciliationProposals.svelte:214` (« ignoré sans message », chargement d'une ligne).

   Plus, que la commande ne voit pas : l'en-tête **entier** de `AlreadyBookedConfirmDialog.svelte` (`:1-14` : `:2-3`
   « confirmation avant un geste qui passerait une écriture », `:5-7` ouvert par les trois gestes et « propose *Continuer quand
   même* », `:12-13` le contrat des props, que D2 change) ; `user-manual.tex:3097` (à **compléter**) ;
   `ReconciliationProposals.svelte:326-331` (doc de `confirmGesture`, sur deux lignes) ; `ReconciliationProposals.test.ts:1533` ;
   `already-booked.ts:6-9`. Chacun corrigé ou laissé avec sa raison au Change Log. **Retiré de l'inventaire** (validation P3) :
   le commentaire `ReconciliationProposals.svelte:804-806` (« tous les refus de « Rejeter » portent sur une transaction sortie
   de la liste ») parle du bilan `failed[]`, que la story ne change pas.
7. **AC7 — E2E.**
   - **Neuf**, distinct de celui de la 34-4, même montage : ligne signalée cochée, *Rejeter* → dialogue ; *Annuler* → la ligne
     reste ; *Rejeter quand même* → elle quitte la liste. Nettoyage : l'écriture reste non rattachée, contre-passée comme
     aujourd'hui (`:194-195`) ; l'import portant une ligne rejetée se supprime (`rejected_count`,
     `kesh-db/src/repositories/bank_imports.rs:435`, `:610`).
   - **Existants qui rejettent** : `reconciliation-deleted-import.spec.ts:166` et `reconciliation-reject-unmatched.spec.ts:166`
     assertent d'abord l'absence d'`already-booked-warning` sur leurs lignes.
8. **AC8 — Manuel et CHANGELOG.** `user-manual.tex:3049-3050` (*Rejeter* ajouté à l'énumération des gestes qui demandent
   confirmation) ; `:3097` (la confirmation sur une ligne signalée) ; `:3107` (*Rejeter* demande
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
      **Montage** : le bouton n'est rendu qu'avec `hasMore && limit < PROPOSALS_MAX_LIMIT` (`ReconciliationProposals.svelte:727`),
      `PROPOSALS_DEFAULT_LIMIT = 100` (`reconciliation.api.ts:25`) : plus de cent transactions en attente sur le compte, montées
      comme dans `reconciliation-show-older.spec.ts` (relevé CAMT à `N` entrées, `:85-92`, IBAN de test partagé), **plus** une
      écriture non rapprochée du montant de l'une d'elles, qui la fait signaler (patron de
      `reconciliation-already-booked.spec.ts`), contre-passée au nettoyage (`:194-195`), l'import supprimé. Base jetable.
      **Si le constat diverge** (la liste se relit, ou le bouton reçoit le clic) : le commentaire de D5 n'est pas posté tel
      quel ; l'écart est écrit au registre `e354` et au rapport, et revient à l'orchestrateur.
- [ ] Tests, rouges consignés ; composant, dialogue, clés ×4 et gardes i18n ; sites de l'ancien contrat ; E2E ; manuel ;
      CHANGELOG ; capture.
- [ ] Gates : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; backend complet
      (`test-fast.sh --ci`, les catalogues changent) ; E2E complet au dernier commit de code.

## Dev Notes

### Hors périmètre

- **#1052** : nommer l'écriture ressemblante (contrat d'API).
- **#1034**, **#76**, **#993** : autres stories.

### Fichiers communs — règle d'union C-e350-8

`messages.ftl` ×4 (ici à côté de `…-confirm-*`), `user-manual.tex` (ici `:3049-3050`, `:3097`, `:3107`), `CHANGELOG.md`,
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
- 2026-11-12 — **Validation P3** (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW (`kesh-gate-logs/e35-4-validation-p3.md`).
  Série 9 → 3 → 2. Remédiée par l'agent de relève (Opus 5.5). Prémisses relues sur `release/0.17.6` (`5e2a5818`) : la commande
  d'AC6 **exécutée** (onze lignes) ; `AlreadyBookedConfirmDialog.svelte:1-14`, `:31` ; `user-manual.tex:3047-3051`, `:2780` ;
  `reconciliation.rs:1527-1545`, `:4697-4720`, `post_manual_once` `:5036-5164`, `post_split_once` `:5496-5619` ;
  `ManualMatchModal.svelte:139-151`, `TransactionSplitModal.svelte:234-239` ; `reconciliation-deleted-import.spec.ts:148-166` ;
  `ReconciliationProposals.svelte:159-181`, `:727`, `:802-806` ; tests `:1699`, `:1729`, `:1737-1743`, `:1755-1778` ;
  `messages.ftl` (`:925`, `:1557` fr-CH ; `:872`, `:1487` de, it, en ; apostrophes du bloc `:1450-1462`). Toutes confirmées ;
  V3-2 **plus large que le rapport** : sur une ligne rejetée dans un autre onglet, *Écriture manuelle* et *Éclater* ne sont pas
  refusés — le serveur passe l'écriture et lève le rejet (`:5140`, `:5605`), tracé à l'audit.
  - V3-1 (MEDIUM) : sortie réelle écrite dans AC6, trois faux positifs nommés et écartés (`:860`, `:2780`, `Proposals.svelte:214`) ;
    ajoutés `user-manual.tex:3049-3050` (AC6, AC8, « Ce qui se lit », fichiers communs, registre C-e350-8) et l'en-tête entier
    du dialogue (`:1-14`).
  - V3-2 (MEDIUM) : « Ce qui se lit au code » dit le cas « autre onglet » geste par geste ; la spec citée l'est pour ce qu'elle
    joue (import supprimé) ; le commentaire de fermeture de #1024 (D5) récrit, geste par geste ; registre C-e350-18 annoté.
  - LOW : V3-3 (montage de l'étape 0, conduite si le constat diverge, phrase du constat entre crochets en D5) ; V3-4
    (`:1737-1743` remplacé par AC1, AC3 l'exclut, plage `:1757-1777`) ; V3-5 (rouge de *Rejeter* en AC4, « aucun appel de
    geste ») ; V3-6 (écriture inconditionnelle, D3) ; V3-7 (libellés cités tirés des deux clés dans chaque locale, `’`) ;
    V3-8 (clic hors à l'étape 0 ou à l'E2E, Vitest pour *Annuler*, Échap, X) ; V3-9 (`ReconciliationProposals.svelte:804-806`
    retiré de l'inventaire, avec sa raison) ; V3-10 (`sprint-status.yaml` : commentaire de la clé mis à jour).
  - Aucun choix nouveau au registre (les corrections sont des faits relus au code).
  - Symptômes greppés (fiche, registre, fichier d'epic) : `1757-1781`, « autre onglet », `failed[]`, « elle rend », `:805` —
    remplacés dans la fiche, annotés au registre (C-e350-18) ; fichier d'epic : rien.
- 2026-11-12 — **Validation P4, passe ciblée** (Sonnet) sur la remédiation P3 : 0 finding au-dessus de LOW, 1 LOW
  (`kesh-gate-logs/e35-4-validation-p4-ciblee.md`). V4-1 appliqué : la puce D3 coupée en deux (« D3 (suite) »), sans
  changement de fond.
- 2026-11-12 — **Validation terminée.** Série des findings au-dessus de LOW : P1 9 MEDIUM → P2 3 → P3 2 → P4 0 (CRITICAL et
  HIGH : 0 à chaque passe). Modèles : Opus (P1), Sonnet (P2), Opus (P3), Sonnet en passe ciblée (P4). La boucle se clôt sur
  une passe sans finding au-dessus de LOW ; la remédiation de V4-1 ne touche que la rédaction. Choix retenus : **C-e350-18**
  (point (3), `sitesTotal` sous la règle d'union ; points (1) remplacé et (2) l'issue #1052) et **C-e350-20** (une relecture
  referme la confirmation). Statut : `ready-for-dev`.
