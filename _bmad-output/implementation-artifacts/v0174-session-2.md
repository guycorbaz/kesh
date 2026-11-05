# Story v0174-session-2 : la 0.17.4, côté navigateur — avertir avant la fermeture pour inactivité, avec « Rester connecté » (#895), et les LOW navigateur de la 0.17.3 (#898, #899)

Status: ready-for-dev

*Sous-fiche de `v0174-session.md` (contexte, triage des LOW, issues demandées), qui fait foi pour ce qu'elle porte.
Branche `story/v0174-session`, worktree `/home/gcorbaz/devel/kesh-0174s`, port **3177**, bases `kesh_v0174s` /
`kesh_e2e_v0174s`. Registre `v0174-choix-autonomes-v0174s.md` : **C-v0174s-10 à 15** s'appliquent ici. Numéros de ligne
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
- `i18nMsg(clé, repli)` (`$lib/shared/utils/i18n.svelte`), patron `DegradedBanner.svelte` ; la garde
  `crates/kesh-api/tests/textes_coherents.rs` (G9) exige un repli égal à la valeur fr-CH ; `frontend/src/lib/shared/i18n-keys.test.ts`
  inventorie les sites d'appel (ses compteurs bougent avec un composant neuf).
- Playwright `^1.59.1` (`frontend/package.json:24`) : l'horloge simulée `page.clock` est disponible. Aucune spec E2E de
  la session n'existe.

## Conception

### 1. Le module : un état d'avertissement, et `stayActive()` (C-v0174s-10, C-v0174s-12, C-v0174s-14)

- Constante exportée `WARNING_LEAD_MS = 60_000` ; délai effectif `lead = min(WARNING_LEAD_MS, w / 2)` (`w = N × 60 000`).
- État réactif `warningRemainingMs = $state<number | null>(null)`, écrit **par `tick` seulement** : après le contrôle de
  fermeture, si `shared` et `w !== null` et `last + w − now ≤ lead`, il vaut `last + w − now` (> 0) ; sinon `null`.
  `stop()` le remet à `null`. Exposé en lecture : `get warningRemainingMs(): number | null`. Sans stockage partagé, il
  reste `null` (C-v0174s-14). Une activité dans un autre onglet (clé `lastActivity` repoussée) le remet à `null` au tic
  suivant, sans mécanisme neuf.
- `stayActive(): void` — l'activité explicite du bouton : le même contrôle préalable de l'échéance que `recordActivity`
  (au-delà, elle ferme), puis `lastActivity = now` **sans** la régulation d'une seconde (`lastWriteAt` mis à jour),
  `memLast = now`, et `warningRemainingMs = null` aussitôt (sans attendre le tic). Le battement part au tic suivant par le
  chemin existant (`last > rep`, `heartbeatAt` vieux de plus de 15 s puisque l'utilisateur était inactif).
- Rien d'autre ne change dans le calcul de l'échéance ni dans la fermeture.

### 2. Le composant `SessionIdleWarning.svelte` (C-v0174s-11, C-v0174s-13, C-v0174s-15)

`frontend/src/lib/shared/components/SessionIdleWarning.svelte`, rendu par `routes/+layout.svelte` à côté de
`DegradedBanner`, **seulement** si `sessionIdle.warningRemainingMs !== null`.

- **Contenu** : une phrase statique dans un élément `role="alert"` — `session-idle-warning-announce` « Votre session va
  bientôt se fermer faute d'activité. » —, un compte à rebours visible `aria-hidden="true"` —
  `session-idle-warning-countdown` « Fermeture dans { $remaining }. » où `$remaining` est `m:ss` (`0:59`, arrondi au
  supérieur à la seconde) —, et un `<button type="button">` `session-idle-warning-stay` « Rester connecté », qui appelle
  `sessionIdle.stayActive()` sur `click`. Attributs de test `data-testid="session-idle-warning"`,
  `"session-idle-warning-countdown"`, `"session-idle-warning-stay"`.
- **Aucun déplacement programmatique du focus** (ni `autofocus`, ni `.focus()`, ni prise de focus d'une primitive) :
  ramener le focus depuis l'aperçu PDF ferait recevoir à la fenêtre un `focus` compté comme activité, et l'avis se
  renouvellerait seul (C-v0174s-11).
- **Au-dessus d'un dialogue, sans le fermer** (C-v0174s-13) : l'avis est visible et cliquable au-dessus de la
  superposition d'un dialogue modal ouvert, et un clic sur « Rester connecté » **ne ferme pas** ce dialogue ni n'en perd
  la saisie. **Étape 0** : établir le moyen — piste à éprouver d'abord, une primitive bits-ui empilée au-dessus de la
  pile des couches (non modale, `trapFocus={false}`, `preventScroll={false}`, prise de focus à l'ouverture empêchée,
  `interactOutsideBehavior="ignore"`, `escapeKeydownBehavior="ignore"`), dont un clic ne fait pas réagir les couches
  inférieures ; à défaut, un autre moyen simple (et dit au Change Log). Si aucun moyen simple n'existe : l'écart est écrit
  (manuel : « bougez la souris dans Kesh, hors du dialogue »), l'issue demandée, et l'AC-4 (c) est retirée avec motif.
- **Forme** : bandeau ou carte fixe en haut, centré, thème clair, contraste AA (jetons de couleur existants — la leçon de
  #640 : vérifier que le jeton employé est **défini**, `components/ui/jetons-non-definis.test.ts`), visible au-dessus de
  l'aperçu PDF (`<iframe>`) sans le masquer entièrement.
- **i18n** : trois clés aux quatre catalogues (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), replis
  égaux au fr-CH (G9). de-CH « Ihre Sitzung wird wegen Inaktivität bald geschlossen. » / « Schliessung in { $remaining }. » /
  « Angemeldet bleiben » ; it-CH « La sessione sta per chiudersi per inattività. » / « Chiusura tra { $remaining }. » /
  « Resta connesso » ; en-CH « Your session will soon close due to inactivity. » / « Closing in { $remaining }. » /
  « Stay signed in » (relus par le développement ; orthographe suisse : `ss`).

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
4. **Rec-e1** — `pointerout` sur `window` (capture, passif) : si `relatedTarget` est nul (le pointeur quitte la page,
   pour un `<iframe>` ou hors de la fenêtre), `lastPointer = null` ; la sortie suivante compte, même au même pixel.
   Ajouté à `addListeners`/`removeListeners`.

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

**AC-3 — Le composant.** `SessionIdleWarning.test.ts` (neuf) : `warningRemainingMs = 59_400` ⇒ texte visible
« Fermeture dans 1:00. » (arrondi au supérieur), puis `0:59` au tic suivant ; l'élément `role="alert"` porte la phrase
statique et **son texte ne change pas** d'un tic à l'autre ; le compte à rebours est `aria-hidden="true"` ; un clic sur
« Rester connecté » appelle `stayActive` **une** fois ; `document.activeElement` est **inchangé** par l'apparition de
l'avis (un `<input>` focalisé avant le reste) ; `warningRemainingMs = null` ⇒ rien n'est rendu. Le test du layout racine
(`root-layout-session.test.ts`) : l'avis n'est rendu que pendant une session. **Rouge avant correctif.**

**AC-4 — E2E** (`frontend/tests/e2e/session-idle-warning.spec.ts`, neuf ; N = 2 posé par `PUT
/api/v1/admin/session-settings` dans la spec, remis à 60 en fin ; `page.clock.install()` **avant** la connexion, pour
simuler `Date` et les minuteries de la page) : (a) connexion, aucune activité, `page.clock.fastForward` jusqu'à
`w − 60 s` ⇒ l'avis est visible ; clic sur « Rester connecté » ⇒ l'avis disparaît, et au-delà de l'ancienne échéance la
page est toujours la même (pas de `/login`) ; (b) aucune activité jusqu'à l'échéance ⇒ `/login?reason=idle&minutes=2`
(comme en 0.17.3) ; (c) un dialogue ouvert (par exemple celui de passage en perte d'une facture, `WriteOffDialog`, ou un
autre dialogue de la démonstration qui porte un champ), un texte saisi dedans, avis affiché, clic sur « Rester
connecté » ⇒ le dialogue est **toujours ouvert** et le texte saisi intact (C-v0174s-13) ; (d) la page porte un aperçu PDF
(document de démonstration, `DocumentPreview`) : l'avis est visible au-dessus. L'horloge simulée ne touche pas le
serveur : si le battement du (a) est refusé parce que le serveur n'a pas avancé, c'est attendu ; établir à l'étape 0
que la spec reste juste (le serveur, dont l'échéance n'est pas atteinte en temps réel, accepte le battement). Les
échecs connus de la suite (`docs/testing.md` § *Les échecs attendus*) restent les seuls. **Rouges avant correctif.**

**AC-5 — LOW navigateur.** Les points 1 à 4 du § *Conception* 3, chacun avec son test, constatés au Dev Agent Record.

**AC-6 — i18n.** Trois clés `session-idle-warning-*` aux quatre catalogues, replis égaux au fr-CH ;
`cargo test -p kesh-i18n` (parité des locales), `textes_coherents.rs` (G9) et `i18n-keys.test.ts` (compteurs mis à
jour, raison au Change Log) verts ; `npm run lint-i18n-ownership` vert.

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
| W10 | Rec-e1 : `lastPointer = null` retiré | AC-5 (4) |
| W11 | régulation d'une seconde retirée de `recordActivity` | AC-5 (1) |
| W12 | contrôle `gen === generation` retiré de `learnFromMe` | AC-5 (1) |

Chaque mutation : motif vérifié unique, Vitest ciblé sous les deux verrous, restauration `git checkout --` puis `touch`.

## Tâches

1. **Étape 0** — relire les lignes citées ; établir le moyen de l'avis au-dessus d'un dialogue (C-v0174s-13) et la
   justesse de l'AC-4 sous `page.clock` ; écart ⇒ Change Log, et issue demandée s'il le faut.
2. Tests rouges (AC-1 à AC-4), constatés.
3. Module (`warningRemainingMs`, `stayActive`, LOW 3 et 4), composant, layout, i18n.
4. Tests (AC-1 à AC-6), LOW 1 et 2.
5. Manuels (`.tex`), puis les **deux** PDF régénérés (`make fr` dans `docs/manual/`, sous les deux verrous — ce `make`
   reprend aussi les textes de la `v0174-session-1`) et contrôlés aplatis ; CHANGELOG.
6. Mutations W1 à W12.
7. Gates : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ;
   `scripts/test-fast.sh --ci` (i18n et `textes_coherents.rs`) ; **E2E complet** au dernier commit de code — chacun sous
   les deux verrous ; recette et captures.

## Captures (`/home/gcorbaz/devel/kesh-gate-logs/v0174s/captures/`, thème clair, au viewport — leçon N-A10)

`v0174s-01-avis.png` (avis sur une page ordinaire, compte à rebours lisible) ; `v0174s-02-avis-apercu-pdf.png` (au-dessus
de l'aperçu) ; `v0174s-03-avis-dialogue.png` (au-dessus d'un dialogue ouvert avec saisie) ; `v0174s-04-apres-clic.png`
(avis disparu, dialogue intact) ; `v0174s-05-fermeture.png` (page de connexion, message d'inactivité — prise **après** le
rendu, non 65 ms après le changement d'adresse, leçon N2-A1). Jointes au Dev Agent Record et au prompt de revue.

## Manuel

- **Utilisateur** (`docs/manual/fr/user-manual.tex:95-97`) : après la phrase de l'échéance, « **Une minute avant**,
  Kesh affiche un avertissement avec un compte à rebours et le bouton **Rester connecté** : un clic — ou toute activité
  dans Kesh — garde la session ouverte. » ; le passage de l'aperçu PDF se **simplifie** : « La lecture d'un PDF dans
  l'aperçu ne compte pas (Kesh ne voit pas ce qui se passe dans le document) : pendant une longue lecture, cliquez sur
  **Rester connecté** quand l'avertissement apparaît. » — le renvoi « (CR \#895) » et la consigne « au moins une fois par
  durée réglée » sont retirés ; le reste du paragraphe est gardé. Si l'étape 0 a écrit un écart (C-v0174s-13), il est dit
  ici.
- **Administration** (`admin-manual.tex:3043`, « Ce qui compte ») : remplacer « (le geste à faire est au manuel
  utilisateur ; un avertissement avant la fermeture est demandé, CR \#895) » par « (une minute avant l'échéance, un
  avertissement propose de rester connecté ; détail au manuel utilisateur) » ; `:3047` (`localStorage`) : ajouter « et
  aucun avertissement ne précède la fermeture ». La précision (« à la seconde près, sauf quatre écarts », `:3039`) est
  inchangée.
- Sonde du PDF aplati (les deux manuels) : « 895 » absent, « Rester connecté » présent.

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

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
