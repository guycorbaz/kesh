# Story 35-4 : *Rejeter* une ligne signalée « déjà comptabilisée » demande confirmation ; une confirmation ne survit plus à une relecture de la liste (#1023, #1024)

Status: done

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

- [x] Étape 0, sur `release/0.17.6` : relire les lignes citées ; **constat en navigateur** (Playwright) : confirmation ouverte,
      `reconciliation-show-older-btn.click({ timeout: 2000 })` échoue (« intercepts pointer events »), Tab ne sort pas du
      dialogue, Échap et clic hors referment ; consigner au registre `e354` (sert au commentaire de D5).
      **Montage** : le bouton n'est rendu qu'avec `hasMore && limit < PROPOSALS_MAX_LIMIT` (`ReconciliationProposals.svelte:727`),
      `PROPOSALS_DEFAULT_LIMIT = 100` (`reconciliation.api.ts:25`) : plus de cent transactions en attente sur le compte, montées
      comme dans `reconciliation-show-older.spec.ts` (relevé CAMT à `N` entrées, `:85-92`, IBAN de test partagé), **plus** une
      écriture non rapprochée du montant de l'une d'elles, qui la fait signaler (patron de
      `reconciliation-already-booked.spec.ts`), contre-passée au nettoyage (`:194-195`), l'import supprimé. Base jetable.
      **Si le constat diverge** (la liste se relit, ou le bouton reçoit le clic) : le commentaire de D5 n'est pas posté tel
      quel ; l'écart est écrit au registre `e354` et au rapport, et revient à l'orchestrateur.
- [x] Tests, rouges consignés ; composant, dialogue, clés ×4 et gardes i18n ; sites de l'ancien contrat ; E2E ; manuel ;
      CHANGELOG ; capture.
- [x] Gates : frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; backend complet
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

Claude Opus 5.5 (workflow `bmad-dev-story`, en autonomie), worktree `kesh-e354`, port 3404, bases `kesh_e354`,
`kesh_e2e_e354`, `kesh_ci_e354`.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e354/` : `rouges.log` (et `rouges-vitest.log`, `e2e-r0/`), `verts-vitest-1.log`,
`mutations.log` (et `mut/M1.log` … `mut/M9.log`), `make-user.log`, `user-manual-aplati.txt`, `gates.log` (`gate-build.log`,
`gate-test-fast.log`, `gate-ci.log`, `gate-ci-journal.log`, `front-*.log`), `e2e-r2/` à `e2e-r5/`, `captures/e354-confirmation-rejeter.png`.

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`).
- **Étape 0** : constat en Chromium sur le code de départ (`1ebe1240`) — la confirmation ouverte bloque la page : le
  bouton *Afficher aussi les plus anciennes* ne reçoit pas le clic (« intercepts pointer events »), Tab reste dans le
  dialogue (six pressions), aucune requête `…/reconciliation/proposals` ne part, Échap et le clic hors referment sans rien
  ouvrir. **Le constat confirme celui de la fiche** : le commentaire de fermeture de D5 peut être posté, la phrase entre
  crochets remplie ainsi (registre `e354`, « Constat de l'étape 0 »). Le constat reste sous test (second test de la spec
  neuve, C-e354-1).
- **D1–D4 livrés** : `requestReject` et `{ kind: 'reject' }` ; `runGesture` en `switch` exhaustif (`never`) ; le dialogue
  reçoit `gesture` et `selectedCount` (relevés dans `guard`, gardés pendant la transition de fermeture comme
  `confirmTransactions`) ; `load()` pose `pendingGesture = null` en tête, sans lecture préalable ; trois clés ×4 locales à
  côté de `…-confirm-*` ; `sitesTotal` 3026 → **3029**, recompté par la garde.
- **AC6, sites de l'ancien contrat** — commande de la fiche rejouée après correction : restent `user-manual.tex:860`,
  `:2780` et `ReconciliationProposals.svelte:226` (les trois faux positifs, le dernier décalé de `:214` par les ajouts),
  et `user-manual.tex:3049`, `:3107`, qui portent désormais *Rejeter* (le motif `\emph{Écriture manuelle} et` y reste
  légitime). Corrigés : en-têtes de `AlreadyBookedConfirmDialog.svelte` (`:1-14`, entier), de
  `ReconciliationProposals.svelte` (`:44-48`), doc de `GuardedGesture` et de `confirmGesture`, `already-booked.ts:6-9`,
  `already-booked-i18n.test.ts` (« huit clés », « dix clés » → treize), commentaire de `ReconciliationProposals.test.ts:1533`.
- **Manuel** : `:3049-3050`, `:3097`, `:3107` ; PDF régénéré, contrôlé aplati : « sans message (issue #1024) » 0,
  « Rejeter quand même » 2, « Rejeter aussi, depuis la version 0.17.7 » 1, « referme une confirmation ouverte » 1,
  « issue #1052 » 1. **CHANGELOG** : `## [0.17.7] — Non publié`, *Corrigé*, un point par issue, celui de #1023 cite #1052.
- **Consigne Edit** : le code, les catalogues, le manuel et le CHANGELOG ont été modifiés par l'outil Edit seulement. Un
  script Python a servi aux seules mutations (motif vérifié unique, restauration `git checkout --` + `touch`) et à corriger
  ce script lui-même (hors dépôt). Aucun écart.

### File List

- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (modifié)
- `frontend/src/lib/features/reconciliation/AlreadyBookedConfirmDialog.svelte` (modifié)
- `frontend/src/lib/features/reconciliation/already-booked.ts` (modifié, en-tête)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` (modifié)
- `frontend/src/lib/features/reconciliation/already-booked-i18n.test.ts` (modifié)
- `frontend/src/lib/shared/i18n-keys.test.ts` (modifié, `sitesTotal`)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (modifiés)
- `frontend/tests/e2e/reconciliation-already-booked-reject.spec.ts` (neuf)
- `frontend/tests/e2e/helpers/camt-synthetic.ts` (neuf, extrait de la spec de la 29-9)
- `frontend/tests/e2e/reconciliation-show-older.spec.ts`, `reconciliation-deleted-import.spec.ts`,
  `reconciliation-reject-unmatched.spec.ts` (modifiés)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf` (modifiés)
- `CHANGELOG.md` (modifié)
- `_bmad-output/implementation-artifacts/e35-choix-autonomes-e354.md` (neuf), `sprint-status.yaml`, cette fiche

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
- 2026-11-13 — **Développement** (Opus 5.5, `bmad-dev-story`, en autonomie). Quatre commits sur `1ebe1240` : `45a0ec3b`
  (tests rouges, étape 0), `b2795d2c` (D1–D4, dernier commit de code de production), `ec3c9e35` (manuel, PDF, CHANGELOG),
  `16105cba` (spec E2E : clic hors réessayé, C-e354-2 — dernier commit de code).
  - **Étape 0** : le constat **confirme** la fiche (registre `e354`) ; le commentaire de fermeture de #1024 (D5) peut
    être posté, la phrase entre crochets remplie par ce constat.
  - **Rouges** (`kesh-gate-logs/e354/rouges.log`) : Vitest 12 rouges, 71 verts — AC1 (5) et AC4/*Rejeter* faute de
    dialogue ; AC4 *Accepter*, *Écriture manuelle*, *Éclater* parce que le dialogue réapparaît ouvert après la relecture ;
    AC3 (3) sur le seul `data-testid` neuf `already-booked-confirm-intro` (non-régression du texte verte) ; AC2 vert. E2E
    neuve rouge sur le binaire de départ (AC7), le test de l'étape 0 vert.
  - **Mutations** M1–M9 de la fiche jouées sur `ReconciliationProposals.test.ts` : **neuf tuées**
    (`kesh-gate-logs/e354/mutations.log`). ⚠️ Un premier tirage les déclarait toutes « tuées » sur une sortie
    « No test files found » (chemin de test relatif à la mauvaise racine) : le script a été corrigé pour exiger un test
    rouge nommé, et le tirage rejoué ; seul le second compte.
  - **Gates complets**, sur `ec3c9e35` (code de production final) : `cargo build --workspace --all-targets` vert ;
    `scripts/test-fast.sh --ci` **5307 passés, 13 ignorés** ; `scripts/gate-ci.sh kesh_ci_e354` **5314 passés, 0 échoué,
    17 ignorés** ; frontend : `check` 0 erreur (29 avertissements, aucun dans un fichier touché), `lint-i18n-ownership`
    PASS, `test:unit` **3197 tests, 232 fichiers**, `build` vert. Après `16105cba` (spec E2E seule) : `npm run check`
    rejoué, 0 erreur ; spec rejouée seule ×5, **10/10** ; **E2E complet** (`e2e-r5`) : **406 passés, 7 échoués,
    17 ignorés** — les sept de la KF-029 (#97) : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
    `onboarding.spec.ts:57`, `:77`, `:150`. Le run précédent (`e2e-r2`, sur `ec3c9e35`) en rendait 18 : les sept, le clic
    hors de la spec neuve (C-e354-2), et dix `supplier-documents` / `supplier-invoice-*` en `inbox-import: 503` et
    `INTERNAL_ERROR` du relevé saisi, nés de `sqlx: pool timed out` au backend pendant que d'autres gates chargeaient
    MariaDB ; aucun ne revient au run `e2e-r5`.
  - **Choix** : C-e354-1 (le constat de l'étape 0 devient un test E2E ; relevé synthétique partagé), C-e354-2 (clic hors
    réessayé dans l'E2E).
  - **Capture** : `kesh-gate-logs/e354/captures/e354-confirmation-rejeter.png` (thème clair). Le guillemet « y tombe seul
    en fin de ligne : le catalogue n'emploie aucune espace insécable — issue demandée, hors périmètre.
  - Statut : `review`.
- **2026-11-13 — Revue de code, close en P1** (prompt versionné `35-4-rejeter-ligne-signalee-review-prompt-p1.md`, `d4500c93` ;
  diff aplati `1ebe1240..16105cba` ; rapports hors dépôt `kesh-gate-logs/e354-review-p1-{B,E,A}.md`).
  - Trend : **P1 (Opus, trois lentilles) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 17 LOW** (B 5, E 5, A 7) → boucle close, aucune
    remédiation de code.
  - **M10 jouée par l'orchestrateur** (E4/A7 : la mutation propre à D3 manquait) : `load()` mutée en
    `if (pendingGesture) pendingGesture = null;`, `npx vitest run …/ReconciliationProposals.test.ts` sous verrou frontend →
    **tuée** (exit 1, quinze tests rouges dont AC1 et AC4 ; journal `kesh-gate-logs/e354/mut-m10.log`) ; fichier restauré par
    `git checkout --` puis `touch`, `git status --porcelain` vide. « AC1 le verrait » (D3) est désormais constaté.
  - LOW, une issue par problème : #1086 (guillemets sans espace insécable, demandé au développement : A6), #1087 (manuel
    `:3097`, « Elle » sans antécédent), #1088 (garde `if (!current) return;` non exercée), #1089 (sélection modifiée derrière la
    modale, jsdom ; B2 et E1), #1090 (`cliquerHors` réessayé ; B3 et E5), #1091 (`toContainText('1')`), #1092 (en-tête
    d'`already-booked.ts`), #1093 (bornes du compte non testées), #1094 (double clic, Échap pendant l'envoi, `failed[]` partiel).
  - Annotations datées du compte rendu de développement (A1, A2, A3, A5), sans réécrire son texte : `e2e-r2` portait aussi
    « 2 did not run » ; l'attribution des `pool timed out` à la charge des autres gates est une inférence, non une ligne de
    journal ; 167 captures d'autres specs sont déposées dans `e354/captures/` (`KESH_E2E_CAPTURES_DIR` posé pour toute la
    suite) ; « code de production final » désigne `ec3c9e35`, dernier commit qui touche du code de production avec le manuel,
    le dernier commit de code étant `16105cba` (spec E2E seule).
  - Statut : `done`.
