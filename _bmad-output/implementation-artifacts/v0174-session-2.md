# Story v0174-session-2 : la 0.17.4, côté navigateur — avertir avant la fermeture pour inactivité, avec « Rester connecté » (#895), et les LOW navigateur de la 0.17.3 (#898, #899)

Status: ready-for-dev

*Sous-fiche de `v0174-session.md` (contexte, triage des LOW, issues demandées), qui fait foi pour ce qu'elle porte.
Branche `story/v0174-session`, worktree `/home/gcorbaz/devel/kesh-0174s`, port **3177**, bases `kesh_v0174s` /
`kesh_e2e_v0174s`. Registre `v0174-choix-autonomes-v0174s.md` : **C-v0174s-10 à 15, 17, 18 et 20** s'appliquent ici. Numéros de ligne
de `7bd4d5c0`. Se développe **après** la `v0174-session-1` (même branche) ; aucune dépendance de code.*

## Story

En tant qu'utilisateur qui lit un long document dans l'aperçu, ou qui a laissé Kesh ouvert un moment, je veux être
averti une minute avant que ma session ne se ferme, et pouvoir la garder d'un clic ; en tant qu'utilisateur parti de son
poste, je veux que la session se ferme quand même à l'échéance.

## Contrainte

Aucune migration, aucune route, **aucun réglage neuf** (mandat ; C-v0174s-10). L'avis ne change ni l'échéance ni sa
précision (0.17.3, `admin-manual.tex:3039`) : il n'est qu'un affichage et une activité de plus. Production réelle :
essais sur bases jetables, captures sur base de démonstration.

## Ce que le code dit

- Le module (`frontend/src/lib/app/stores/session-idle.svelte.ts`) calcule l'échéance à chaque tic (`tick`, `:283-300` :
  `now >= last + w` ⇒ `closeSession`), ne ferme rien sans stockage partagé (`deadlineReached`, `:177-181`, `shared`), et
  expose `start`, `stop`, `setMinutes`, `minutes` (`:372-420`). `minutes` est déjà un `$state` (`:143`) ; rien d'autre de
  l'état n'est réactif.
- Il est démarré par le layout racine (`routes/+layout.svelte:31-41`), qui rend aussi `DegradedBanner` et le `Toaster`
  (`:105`, `:108`) ; le test `routes/root-layout-session.test.ts` couvre l'abonnement.
- Le focus : une `focus` de cible `window` compte comme activité (`:245-247`) — la fenêtre la reçoit quand le focus
  revient d'un `<iframe>` vers la page (C-v0174s-11) ; un `blur` vers un `<iframe>` compte aussi (`:249-252`).
- Les dialogues (bits-ui `Dialog`, `components/ui/dialog/dialog-content.svelte`, superposition `Dialog.Overlay`,
  contenu `z-50`) se ferment par défaut sur un geste hors de leur contenu (`interactOutsideBehavior` employé en `'close'`
  à `WriteOffDialog.svelte:127`, `DeleteBankImportAction.svelte:132`, `McpSendDetail.svelte:346`).
- `i18nMsg(clé, repli)` (`$lib/shared/utils/i18n.svelte`), patron `DegradedBanner.svelte` ; les replis
  Svelte sont gardés par `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`, `i18n-un-repli-par-cle.test.ts` et
  `i18n-entrees-a-variables.test.ts` (G9 de `textes_coherents.rs` ne lit que les replis Rust — « (G9) » écrit ici à la
  fiche était faux, validation P1, F10) ; `frontend/src/lib/shared/i18n-keys.test.ts` inventorie les sites d'appel (ses
  compteurs bougent avec un composant neuf).
- Playwright `^1.59.1` (`frontend/package.json:24`) : l'horloge simulée `page.clock` est disponible. Aucune spec E2E de
  la session n'existe.

## Conception

### 1. Le module : un état d'avertissement, et `stayActive()` (C-v0174s-10, C-v0174s-12, C-v0174s-14)

- Constante exportée `WARNING_LEAD_MS = 60_000` ; délai effectif `lead = min(WARNING_LEAD_MS, w / 2)` (`w = N × 60 000`).
- État réactif `warningRemainingMs = $state<number | null>(null)`, écrit **par `tick` seulement**, **juste après** le
  contrôle de fermeture (`:288-291`) et **avant** le `return` de `heartbeatInFlight` (`:292`) — pour un utilisateur inactif,
  `tick` sort à `:296` (`!(last > rep)`) : un calcul placé en fin de fonction ne s'exécuterait jamais (validation P1, F8) —
  si `shared` et `w !== null` et `last + w − now ≤ lead`, il vaut `last + w − now` (> 0) ; sinon `null`.
  `stop()` le remet à `null`. Exposé en lecture : `get warningRemainingMs(): number | null`. Sans stockage partagé, il
  reste `null` (C-v0174s-14). Une activité dans un autre onglet (clé `lastActivity` repoussée) le remet à `null` au tic
  suivant, sans mécanisme neuf.
- `stayActive(): void` — l'activité explicite du bouton : le même contrôle préalable de l'échéance que `recordActivity`
  (au-delà, elle ferme), puis `lastActivity = now` **sans** la régulation d'une seconde (`lastWriteAt` mis à jour),
  `memLast = now`, et `warningRemainingMs = null` aussitôt (sans attendre le tic). Le battement part au tic suivant par le
  chemin existant (`last > rep`, `heartbeatAt` vieux de plus de 15 s puisque l'utilisateur était inactif).
- Rien d'autre ne change dans le calcul de l'échéance ni dans la fermeture.

### 2. Le composant `SessionIdleWarning.svelte` (C-v0174s-11, C-v0174s-13, C-v0174s-15)

`frontend/src/lib/shared/components/SessionIdleWarning.svelte`, **composant à propriétés** — `remainingMs: number` et
`onStay: () => void` —, que `routes/+layout.svelte` rend à côté de `DegradedBanner` **seulement** si
`typeof sessionIdle.warningRemainingMs === 'number'`, avec `remainingMs={sessionIdle.warningRemainingMs}` et
`onStay={() => sessionIdle.stayActive()}` (validation P1, F7 : le test du composant pilote ses propriétés par
`rerender`, sans getter simulé non réactif ; et la condition `typeof` ne rend rien quand un simulacre du module n'expose pas
le champ — `!== null` rendrait l'avis, `undefined !== null`).

- **Contenu** : une phrase statique dans un élément `role="alert"` — `session-idle-warning-announce` « Votre session va
  bientôt se fermer faute d'activité : cliquez sur « Rester connecté », ou n'importe où dans Kesh hors du document
  affiché. » (validation P1, F12 : dire **comment** agir ; validation P2, P2-3 : **sans promettre la touche**, inopérante
  quand le focus est dans l'aperçu PDF — ses touches ne parviennent pas à la page, `v0172-session.md:291-293` ; seul le
  clic dans la page vaut dans le cas que l'avis vise) —, un compte à rebours visible `aria-hidden="true"` —
  `session-idle-warning-countdown` « Fermeture dans { $remaining }. » où `$remaining` est `m:ss` (`0:59`, arrondi au
  supérieur à la seconde) —, et un `<button type="button">` `session-idle-warning-stay` « Rester connecté », qui appelle
  `onStay` sur `click`. Attributs de test `data-testid="session-idle-warning"`,
  `"session-idle-warning-countdown"`, `"session-idle-warning-stay"`.
- **Aucun déplacement programmatique du focus** (ni `autofocus`, ni `.focus()`, ni prise de focus d'une primitive) :
  ramener le focus depuis l'aperçu PDF ferait recevoir à la fenêtre un `focus` compté comme activité, et l'avis se
  renouvellerait seul (C-v0174s-11).
- **Au-dessus d'un dialogue, sans le fermer** (C-v0174s-13, **récrit** à la validation P1, F3 ; établi dans
  `frontend/node_modules/bits-ui` **2.16.5**) : l'avis est visible et cliquable au-dessus de la superposition d'un dialogue
  modal ouvert, et un clic sur « Rester connecté » **ne ferme pas** ce dialogue ni n'en perd la saisie. **Pas de
  primitive bits-ui** : `Dialog.Content` est toujours `aria-modal="true"` (`bits/dialog/dialog.svelte.js:265-266`), ce
  qui contredirait C-v0174s-11. Le moyen, en trois propriétés d'un élément ordinaire :
  1. **Le geste n'atteint pas `document`** : la couche du dialogue décide de se fermer sur un `pointerdown` de `document`
     en **bouillonnement**, après l'avoir marqué « intercepté » en **capture** (`bits/utilities/dismissible-layer/use-dismissable-layer.svelte.js:84`,
     `:90`, `:110` : `#isAnyEventIntercepted()` ⇒ aucune fermeture). La racine de l'avis pose donc un écouteur
     **natif** `pointerdown` en bouillonnement qui appelle `stopPropagation()` — par `addEventListener` dans une
     attache (`{@attach}`) ou un `$effect`, **non** par `onpointerdown` (Svelte 5 délègue cet événement à la racine de
     l'application : l'arrêt aurait lieu plus haut, à établir ; l'écouteur natif lève le doute). Le module d'inactivité,
     en capture sur `window`, a déjà noté l'activité avant cet arrêt.
  2. **`pointer-events: auto`** sur la racine de l'avis : un `Dialog.Content` à `preventScroll` vrai (défaut,
     `bits/dialog/components/dialog-content.svelte:25`, `:85` — branche `{:else}`, celle qu'emploie Kesh, qui ne passe pas
     `child` ; validation P2, P2-7) monte `ScrollLock`, qui pose
     `document.body.style.pointerEvents = "none"` (`internal/body-scroll-lock.svelte.js:129`) — sans `auto`, l'avis ne
     recevrait aucun clic. *(La lentille F écrivait « pas de `pointer-events: none` posé par `Dialog` en 2.16.5 » : faux,
     rejeté.)*
  3. **`z-index` strictement supérieur à 50** (`z-[60]`) : la superposition (`dialog-overlay.svelte`, `fixed inset-0
     z-50`) et le contenu (`z-50`) sont montés par un portail après le layout ; à égalité, le dialogue gagnerait.
  Échap n'est pas touché (il reste au dialogue ; c'est aussi une activité). **Clavier sous un dialogue** : le piège à focus
  du dialogue (`trapFocus` vrai par défaut) enferme la tabulation ; le bouton n'est pas atteignable au **Tab** tant
  qu'un dialogue modal est ouvert — **écart déclaré**, tracé par **#912**. Un clic sur le bouton déplace le focus hors du dialogue, que
  le piège ramène : sans effet sur la saisie (AC-4 (c) le prouve). Si l'étape 0 constate qu'un des trois points ne tient
  pas en Chromium réel : l'écart est écrit au Change Log et au manuel, l'issue demandée, et l'AC-4 (c) est retirée avec
  motif.
- **Forme** : bandeau ou carte fixe en haut, centré, thème clair, contraste AA (jetons de couleur existants — la leçon de
  #640 : vérifier que le jeton employé est **défini**, `components/ui/jetons-non-definis.test.ts`), visible au-dessus de
  l'aperçu PDF (`<iframe>`) sans le masquer entièrement.
- **i18n** : trois clés aux quatre catalogues (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), replis
  Svelte égaux à la valeur fr-CH. de-CH « Ihre Sitzung wird wegen Inaktivität bald beendet: Klicken Sie auf
  «Angemeldet bleiben» oder irgendwo in Kesh ausserhalb des angezeigten Dokuments. » / « Ende in { $remaining }. » /
  « Angemeldet bleiben » (« beendet », comme
  `login-session-idle` de-CH, `messages.ftl:2229` — validation P1, F10) ; it-CH « La sessione sta per essere chiusa per
  inattività: cliccate su «Resta connesso» o in un punto qualsiasi di Kesh fuori dal documento visualizzato. » / « Chiusura tra { $remaining }. » / « Resta connesso » ;
  en-CH « Your session will soon be closed due to inactivity: click “Stay signed in”, or anywhere in Kesh outside the
  displayed document. » /
  « Closing in { $remaining }. » / « Stay signed in » (relus par le développement ; orthographe suisse : `ss`).

### 3. LOW navigateur de #898 et #899 (fiche parente, § *Triage*)

1. **N-A3/N-E3** — un cas court par ligne dans `session-idle.svelte.test.ts` : la régulation d'écriture
   (`ACTIVITY_WRITE_THROTTLE_MS`, `:61` : dix `keydown` en 500 ms ⇒ **une** écriture de `lastActivity`, compte des
   `setItem`) ; `generation` (`:147`, une réponse de `/me` résolue après `stop()` n'écrit rien ; une réponse de battement
   résolue après `stop()` ne change pas `minutes`) ; `heartbeatInFlight` (`:150`, un battement qui met 20 s à répondre ⇒
   aucun second envoi pendant ce temps) ; `isValidMinutes` (`:79`, réponse `{ idleTimeoutMinutes: 0 }` du battement ou de
   `/me` ⇒ `minutes` inchangé).
2. **N-A8/N-E10** — le cas (9) (« une requête de fond ne modifie pas lastActivity ») réécrit en garde de principe (il
   vérifie qu'`apiClient` n'est pas importé pour écouter : commentaire et assertion sur les écouteurs posés, non un
   rouge impossible), l'assertion `toBe(480_000 > 470_000 ? 470_000 : 0)` remplacée par `toBe(470_000)`, et les cas de
   `currentLast` démarrés à `vi.setSystemTime(1_000_000)` (un `??` muté en `||` rougit alors).
3. **N-E5** — `sendHeartbeat`, branche d'échec (`:315`) : si `previousReported` n'est pas fini, `removeItem` de
   `KEY_REPORTED_ACTIVITY` (et `memory.delete` en repli) au lieu d'un `writeKey` qui ne fait rien ; l'activité redevient
   « à rapporter » et repart au pas de 15 s suivant.
4. **Rec-e1** — `pointerout` sur `window` (capture, passif) : si `relatedTarget` est nul **ou** un
   `HTMLIFrameElement` (validation P2, P2-13), `lastPointer = null` ; la sortie suivante compte, même au même pixel. Le
   pointeur qui passe de la page sur le cadre a pour `relatedTarget` l'`<iframe>` (un nœud de la page) ou `null` selon le
   navigateur : les deux cas sont couverts, rien n'est à constater. Un `pointerout` entre deux éléments ordinaires de la
   page ne remet rien à zéro. Ajouté à `addListeners`/`removeListeners`. **Ce n'est pas un écouteur d'activité** : le garde
   « onze écouteurs » (`session-idle.svelte.test.ts:458`, `:493-494`, `toHaveLength(11)`) reste à **onze** — il porte sur
   les écouteurs d'activité, chacun avec son témoin de fermeture —, et son commentaire le dit ; `pointerout` a ses propres
   cas (Vitest : `relatedTarget` nul, puis `<iframe>`, puis un `<div>`). **Pas de garde sur son retrait par `stop()`** :
   un `pointerout` resté posé ne fait que remettre `lastPointer` à `null`, sans effet observable, puisque `pointermove`
   est retiré et que `start()` remet `lastPointer` à `null` (validation P2, P2-5 ; W13 retirée).

## Acceptance Criteria

**AC-1 — L'avertissement, au bon moment.** `session-idle.svelte.test.ts` (horloge simulée, `localStorage` de jsdom
vidé, N = 2, démarrage à `vi.setSystemTime(1_000_000)`) : à `w − 60 001` ms d'inactivité, `warningRemainingMs` est
`null` ; au tic de `w − 60 000`, il vaut `60 000` ; il décroît d'une seconde par tic ; à l'échéance, la fermeture a lieu
comme en 0.17.3 et `warningRemainingMs` revient à `null`. N = 1 : l'avis apparaît à 30 s de l'échéance. Sans stockage
partagé (`setItem` qui lève) : jamais d'avis. **Rouge avant correctif.**

**AC-2 — Toute activité, et « Rester connecté », l'effacent.** (a) `stayActive()` pendant l'avis ⇒ `warningRemainingMs`
`null` aussitôt, `lastActivity = now`, un battement part au tic suivant avec `idleMs` ≤ 1 000 ; (b) un `keydown` pendant
l'avis ⇒ l'avis disparaît au tic suivant ; (c) « un autre onglet » qui écrit `lastActivity = now` ⇒ l'avis disparaît
au tic suivant ; (d) `stayActive()` appelée **après** l'échéance (minuterie arrêtée) ⇒ fermeture, `lastActivity` non
réécrite ; (e) dix `stayActive()` en 500 ms ⇒ dix écritures permises (pas de régulation), aucune erreur. **Rouge avant
correctif.**

**AC-3 — Le composant.** `SessionIdleWarning.test.ts` (neuf ; récrit à la validation P2, P2-2, sur les propriétés) :
`render(SessionIdleWarning, { remainingMs: 59_400, onStay })` ⇒ texte visible « Fermeture dans 1:00. » (arrondi au
supérieur) ; `rerender({ remainingMs: 58_400 })` ⇒ « 0:59 », et le texte de l'élément `role="alert"` (phrase statique)
**est inchangé** ; le compte à rebours est `aria-hidden="true"` ; un clic sur « Rester connecté » appelle `onStay`
**une** fois ; `document.activeElement` est **inchangé** par le rendu (un `<input>` focalisé avant le reste). Le cas
« rien n'est rendu » appartient au layout seul. Le test du layout racine
(`root-layout-session.test.ts`) : son simulacre de `sessionIdle` (`:11-16`) gagne `warningRemainingMs` (`null`, puis un
nombre) et `stayActive` ; l'avis est rendu si et seulement si le champ est un nombre ; un champ absent du simulacre ne
rend rien. **Et** un `pointerdown` émis sur l'avis **n'atteint pas** un écouteur posé sur `document` en bouillonnement
(espion), alors qu'un écouteur de `window` en capture le voit (le module note l'activité) — C-v0174s-13 (1) ; la racine
porte `pointer-events: auto` et un `z-index` > 50 (classes contrôlées). **Rouge avant correctif.**

**AC-4 — E2E** (`frontend/tests/e2e/session-idle-warning.spec.ts`, neuf ; N = 2 posé par `GET` puis `PUT
/api/v1/admin/session-settings` (le `PUT` exige la `version` lue), **remis à 60 dans un `afterAll`/`finally`** — un échec
de la spec ne doit pas laisser N = 2 au reste de la suite (`workers: 1`, `playwright.config.ts:53`) ; `page.clock.install()` **avant** la connexion, pour
simuler `Date` et les minuteries de la page) : (a) connexion, aucune activité, `page.clock.fastForward` jusqu'à
`w − 60 s` ⇒ l'avis est visible ; clic sur « Rester connecté » ⇒ l'avis disparaît, et au-delà de l'ancienne échéance la
page est toujours la même (pas de `/login`) ; (b) aucune activité jusqu'à l'échéance ⇒ `/login?reason=idle&minutes=2`
(comme en 0.17.3) — **témoin de non-régression, vert avant correctif** (validation P1, F4) ; (c) un dialogue ouvert qui
porte un **champ de texte** (`WriteOffDialog` n'en a pas — radios et date, `WriteOffDialog.svelte:207`, F6 : choisir à
l'étape 0 un dialogue atteignable avec la démonstration et le nommer au Change Log — candidats : un dialogue de
`McpPromptDetail.svelte` à `textarea`, `SendEmailDialog.svelte` si le SMTP de la recette suffit — ou, à défaut, saisir la
date de `WriteOffDialog` et vérifier qu'elle est intacte), une valeur saisie dedans, avis affiché, clic sur « Rester
connecté » ⇒ le dialogue est **toujours ouvert** et la valeur intacte (C-v0174s-13) ; (d) un aperçu PDF ouvert — obtenu
comme le fait `frontend/tests/e2e/supplier-documents.spec.ts` (dépôt dans la boîte d'entrée, `inboxHandshake` `:19`,
`:62`, `deposit(...)`, `PDF_SANS_QR` `:11`, `:123` ; exige `KESH_INBOX_DIR` des deux côtés) : il n'existe **aucun** document
de démonstration (F5) — : l'avis est visible au-dessus, et le **clic** sur « Rester connecté » réussit (l'actionnabilité
de Playwright prouve que le cadre ne le masque pas ; le rendu du PDF par Chromium sans tête n'est pas contrôlé, seul
l'`<iframe>`). Sous `page.clock` (F16) : attendre la réponse du battement (`waitForResponse('**/api/v1/auth/activity')`)
avant un nouveau `fastForward`, et ne jamais avancer l'horloge pendant une requête en vol (la minuterie de 30 s
d'`apiClient`, `api-client.ts:319`, se déclencherait). L'horloge simulée ne touche pas le
serveur : si le battement du (a) est refusé parce que le serveur n'a pas avancé, c'est attendu ; établir à l'étape 0
que la spec reste juste (le serveur, dont l'échéance n'est pas atteinte en temps réel, accepte le battement). Les
échecs connus de la suite (`docs/testing.md` § *Les échecs attendus*) restent les seuls. **Rouges avant correctif** :
(a), (c), (d).

**AC-5 — LOW navigateur.** Les points 1 à 4 du § *Conception* 3, chacun avec son test, constatés au Dev Agent Record.

**AC-6 — i18n.** Trois clés `session-idle-warning-*` aux quatre catalogues, replis égaux au fr-CH ; `cargo test -p
kesh-i18n` (parité des locales) vert ; les gardes Svelte (validation P1, F10 : G9 de `textes_coherents.rs` — `:19`, `:711` — ne lit
que les replis **Rust**) — `frontend/src/lib/shared/i18n-keys.test.ts` (compteurs mis à jour, raison au Change Log),
`i18n-entrees-a-variables.test.ts` (le site passe `{ remaining }` à `session-idle-warning-countdown`),
`i18n-un-repli-par-cle.test.ts`, `i18n-repli-divergent-actif.test.ts` — vertes. `npm run lint-i18n-ownership` reste vert,
**par construction** pour ce composant (le script ne balaie que `lib/features`, `lint-i18n-ownership.js:17`) : ce n'est
pas un contrôle de l'avis.

**AC-7 — Les textes disent vrai.** § *Manuel* (PDF compris), § *CHANGELOG*.

**AC-8 — Recette et captures** (§ *Captures*), base jetable, N = 2 réglé **par l'écran** (leçon N-A6),
`KESH_JWT_EXPIRY_MINUTES` ≥ 2 pour tout contrôle 401 (leçon N-A5) : (a) l'avis apparaît à 1 min 00 s ± 2 s de la
dernière activité ; (b) clic ⇒ session gardée ; (c) sans clic ⇒ fermeture à 2 min 00 s ± 2 s ; (d) lecture d'un PDF dans
l'aperçu, pointeur et défilement **dans** le document : l'avis apparaît au-dessus, un clic sur le bouton garde la
session ; (e) deux onglets : l'avis apparaît dans les deux, un clic dans l'un l'efface dans l'autre au tic suivant ;
(f) un dialogue ouvert avec une saisie : clic ⇒ dialogue et saisie intacts. Chronomètre et heures au Dev Agent Record.

## Tests et mutations

| # | Mutation | Tuée par |
|---|---|---|
| W1 | condition d'apparition inversée, ou `lead` à 0 | AC-1 |
| W2 | `min(…, w / 2)` retiré (avis dès l'activité pour N = 1) | AC-1 (N = 1 : 30 s) |
| W3 | avis rendu sans stockage partagé | AC-1 |
| W4 | `stayActive` ne note pas l'activité (seulement `warningRemainingMs = null`) | AC-2 (a) : l'avis revient au tic suivant, fermeture à l'ancienne échéance |
| W5 | `stayActive` sans contrôle préalable de l'échéance | AC-2 (d) |
| W6 | compte à rebours dans l'élément `role="alert"` | AC-3 (texte de l'alerte inchangé) |
| W7 | `.focus()` sur le bouton à l'apparition | AC-3 (`activeElement` inchangé) |
| W8 | `stop()` ne remet pas `warningRemainingMs` à `null` | AC-1 (après la fermeture) |
| W9 | N-E5 : `removeItem` remplacé par l'ancien `writeKey` | AC-5 (3) |
| W10 | Rec-e1 : `lastPointer = null` retiré, puis le cas `HTMLIFrameElement` retiré (deux mutations) | AC-5 (4) (cas `null`, puis cas `<iframe>`) |
| W11 | régulation d'une seconde retirée de `recordActivity` | AC-5 (1) |
| W12 | contrôle `gen === generation` retiré de `learnFromMe` | AC-5 (1) |
| W14 | `stopPropagation()` retiré de l'écouteur natif de l'avis | AC-3 (l'espion de `document` le voit) ; AC-4 (c) |
| W15 | `pointer-events: auto` retiré | AC-3 (classe) ; AC-4 (c) (clic impossible sous `ScrollLock`) |

Chaque mutation : motif vérifié unique, Vitest ciblé sous les deux verrous, restauration `git checkout --` puis `touch`.

## Tâches

1. **Étape 0** — relire les lignes citées (et `node_modules/bits-ui` aux lignes de C-v0174s-13) ; éprouver les trois
   propriétés de l'avis au-dessus d'un dialogue (C-v0174s-13), choisir le dialogue à champ de l'AC-4 (c), établir la
   justesse de l'AC-4 sous `page.clock` ; écart ⇒ Change Log, et issue demandée s'il le faut.
2. Tests rouges (AC-1 à AC-3 ; AC-4 (a), (c), (d)), constatés.
3. Module (`warningRemainingMs`, `stayActive`, LOW 3 et 4), composant, layout, i18n.
4. Tests (AC-1 à AC-6), LOW 1 et 2.
5. Manuels (`.tex`), puis les **deux** PDF régénérés (`make fr` dans `docs/manual/`, sous les deux verrous — ce `make`
   reprend aussi les textes de la `v0174-session-1`) et contrôlés aplatis ; CHANGELOG.
6. Mutations W1 à W12, W14 et W15 (W13 retirée à la validation P2 ; numéros gardés).
7. Gates : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ;
   `scripts/test-fast.sh --ci` (i18n et `textes_coherents.rs`) ; **E2E complet** au dernier commit de code — chacun sous
   les deux verrous ; recette et captures.

## Captures (`/home/gcorbaz/devel/kesh-gate-logs/v0174s/captures/`, thème clair, au viewport — leçon N-A10)

`v0174s-01-avis.png` (avis sur une page ordinaire, compte à rebours lisible) ; `v0174s-02-avis-apercu-pdf.png` (au-dessus
de l'aperçu) ; `v0174s-03-avis-dialogue.png` (au-dessus d'un dialogue ouvert avec saisie) ; `v0174s-04-apres-clic.png`
(avis disparu, dialogue intact) ; `v0174s-05-fermeture.png` (page de connexion, message d'inactivité — prise **après** le
rendu, non 65 ms après le changement d'adresse, leçon N2-A1). Jointes au Dev Agent Record et au prompt de revue.

## Manuel

- **Utilisateur** (`docs/manual/fr/user-manual.tex:95-97`) : après la phrase de l'échéance, « **Environ une minute
  avant** (trente secondes si la durée réglée est d'une minute), Kesh affiche un avertissement avec un compte à rebours et
  le bouton **Rester connecté** : un clic sur ce bouton, ou n'importe où dans Kesh hors du document affiché, garde la
  session ouverte. » (validation P2, P2-3 : pas de promesse de touche) ;
  au `:97`, **seule la phrase en gras** (« Pendant une longue lecture dans l'aperçu, sortez le pointeur du document, ou
  cliquez à côté, au moins une fois par durée réglée … ; un avertissement avant la fermeture est demandé (CR \#895) ») est
  remplacée par « **Pendant une longue lecture dans l'aperçu, cliquez sur Rester connecté quand l'avertissement
  apparaît.** » — la phrase précédente du même paragraphe, qui dit déjà que Kesh ne voit pas ce qui se passe dans le cadre,
  est gardée (pas de doublon, validation P1, F11 (2)) ; le reste du paragraphe est gardé. Si l'étape 0 a écrit un écart (C-v0174s-13), il est dit
  ici.
- **Administration** (`admin-manual.tex:3043`, « Ce qui compte ») : remplacer « (le geste à faire est au manuel
  utilisateur ; un avertissement avant la fermeture est demandé, CR \#895) » par « (une minute avant l'échéance --- trente
  secondes pour une durée d'une minute ---, un avertissement propose de rester connecté ; détail au manuel utilisateur) » ; `:3047` (`localStorage`) : ajouter « et
  aucun avertissement ne précède la fermeture ». La précision (« à la seconde près, sauf quatre écarts », `:3039`) est
  inchangée.
- **Sous-section « Passer à la 0.17.4 »** (créée par la `v0174-session-1`) : la part de cette sous-fiche (fiche parente,
  § *Textes*, point 3) — « Ce qui change à l'écran : un avertissement une minute avant la fermeture pour inactivité, avec
  *Rester connecté* ».
- `docs/kesh-specifications.txt` (`:2690` environ, à côté de la ligne d'acceptation du CR #891 ajoutée par la 0.17.3) : une
  ligne d'acceptation pour l'avis (CR #895) — validation P1, F11 (4).
- Sonde du PDF aplati (les deux manuels) : « 895 » absent, « Rester connecté » présent. Les **deux** PDF sont régénérés
  par cette sous-fiche (`make fr`).

## CHANGELOG (`## [0.17.4] — Non publié`)

- **Ajouté** — « **Un avertissement avant la fermeture pour inactivité** ([#895](…/issues/895)) : une minute avant
  l'échéance, Kesh affiche un compte à rebours et un bouton **Rester connecté**. Il couvre la lecture d'un PDF dans
  l'aperçu, où Kesh ne voit pas vos gestes. »
- **Corrigé** — « Après un battement d'activité en échec, l'activité est de nouveau transmise au serveur ; ressortir du
  document affiché par le même point de l'écran compte comme activité (refs #898). »

## Dev Notes

- Le module reste « sans verrou, sans onglet maître, sans suspension » (C-v0172s-34) : l'avis lit l'état que chaque tic
  calcule déjà.
- `stayActive` n'envoie rien lui-même (C-v0174s-12) : un second chemin d'envoi contournerait la régulation entre onglets.
- Un avis visible dans un onglet en arrière-plan peut avoir un compte à rebours en retard (minuteries ralenties) : sans
  conséquence, l'échéance est calculée sur l'horloge, non sur le compte.
- Ne pas employer `role="alertdialog"` (prise de focus) ni une région `aria-live` sur le compte à rebours (C-v0174s-11).

## Change Log

- 2026-11-05 — Sous-fiche créée (Opus 5.5, autonomie), avec la parente. Prochaine : validation P1 (Sonnet, R et F).
- 2026-11-05 — Validation P1 remédiée (détail au Change Log de la parente) : F3 (C-v0174s-13 récrit, établi dans bits-ui
  2.16.5 ; prémisse « pas de `pointer-events: none` » rejetée), F4, F5, F6, F7 (composant à propriétés, condition
  `typeof`), F8, F9 (douzième écouteur, W13, prémisse `relatedTarget` à constater), F10, F11, F12 (phrase annoncée, clavier
  sous dialogue déclaré), F13 (issue demandée), F16 ; W13 à W15 ajoutées. Prochaine : validation P2 (Opus, complète).
- 2026-11-05 — Validation P2 remédiée (détail au Change Log de la parente) : P2-2 (AC-3 sur les propriétés), P2-3
  (annonce et manuels sans promesse de touche, quatre locales), P2-5 (W13 retirée, garde à onze écouteurs d'activité),
  P2-7 (`:85`), P2-9, P2-13 (`relatedTarget` nul ou `<iframe>`, plus de constat), P2-14 (à #912). Prochaine :
  validation P3 (Sonnet, complète).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
