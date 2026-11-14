# Story 36-5 : le compte rendu d'une restauration ne se perd plus sous un battement de session refusé (#931, KF-056)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-5`, branche
`story/36-5-restauration-compte-rendu`, par le workflow `bmad-create-story`. Release **0.17.8**, phase 1. **Aucune
migration**, aucun changement serveur ni d'API, aucune clé i18n. Code lu à la tête du worktree, **`4de0bcc7e`**. Étape 0 et
choix qui font foi : `e36-choix-autonomes-e360.md` (§ *Étape 0 — 36-5*, C-e360-8, C-e360-11) ; choix propres à la story :
`e36-choix-autonomes-36-5.md` (C-36-5-1 à C-36-5-6). Voie de base **1** (lignée B, après la 36-4). Validation non lancée.*

## Story

En tant qu'Administrateur qui restaure une installation — depuis *Administration → Restaurer / Importer* ou depuis
l'assistant d'une installation neuve —, je veux que le compte rendu de l'import (pièces restaurées, pièces introuvables)
reste à l'écran jusqu'à ce que je clique *Se reconnecter* ou que je quitte la page, afin de lire, avant d'être renvoyé à la
connexion, combien de pièces je dois aller rechercher — sans que la fermeture ordinaire de session (#876, #895) en soit
affaiblie.

## Ce qui se lit au code (`4de0bcc7e`)

**La cause** (commentaire de #931 du 2026-11-13, traces Playwright `kesh-gate-logs/e355a0/`) : l'import invalide la session
au serveur ; un battement d'activité parti entre ce moment et l'arrivée de la réponse rend 401 ; le chemin 401 du client
efface la session et quitte la page. Lignes lues :

- **Le battement démarre au layout racine, donc sur les deux pages** — `frontend/src/routes/+layout.svelte:25-26` : « le
  suivi d'activité de la session, pour `(app)` comme pour `/onboarding` » ; `:33-42` : `$effect` qui appelle
  `sessionIdle.start({...})` dès que `sessionUserId` n'est pas nul, et `return () => sessionIdle.stop();`.
- **Le battement** — `frontend/src/lib/app/stores/session-idle.svelte.ts` : `tick` (`:326-349`) ferme la session à
  l'échéance (`:331-333`, `void closeSession();`), calcule l'avertissement (`:338-340`), puis, si
  `!(last > rep) || now - hb < HEARTBEAT_MIN_INTERVAL_MS` est faux (`:345`), envoie `void sendHeartbeat(now - last, rep, generation);`
  (`:348`). `sendHeartbeat` (`:351-377`) passe par `apiClient.post<IdleFields>('/api/v1/auth/activity', …)` (`:354`) ; son
  `catch` dit lui-même : « Un 401 définitif a déjà effacé la session et redirigé (chemin 401) » (`:363`). Le clic sur
  *Confirmer le remplacement* est une activité (`pointerdown`, `WINDOW_EVENTS`, `:299`) : il rend un battement dû au tic
  suivant si le dernier date de plus de 15 s (`HEARTBEAT_MIN_INTERVAL_MS = 15_000`, `:63`). La fermeture d'inactivité passe
  aussi par `recordActivity` (`:250-252`) et `stayActive` (`:470-472`), puis `closeSession` (`:379-409`) :
  `window.location.replace(\`/login?reason=idle&minutes=${n ?? ''}\`);` (`:407`).
- **Le serveur refuse le battement après l'import** — `crates/kesh-api/src/routes/admin.rs:191-194` : « L'import remplace
  les `refresh_tokens` ⇒ sessions destination invalidées (`sessionInvalidated: true`) ; […] il **révoque** dans sa
  transaction les lignes restaurées encore actives » ; `crates/kesh-api/src/routes/auth.rs:795-796` : « cookie absent, ligne
  d'un autre utilisateur, révoquée ou échue ⇒ 401 ». Le jeton d'accès, lui, est sans état
  (`crates/kesh-api/src/middleware/auth.rs:150` : `let session = jwt::decode_session(&token, …)?;`) : les autres routes
  l'acceptent jusqu'à son `exp`, d'où un renvoi plus tardif possible (bandeau des sauvegardes, plus bas).
- **Le chemin 401 du client** — `frontend/src/lib/shared/utils/api-client.ts:198-205` :
  ```ts
  async function refreshTokens(): Promise<boolean> {
  	const refreshed = await sharedRefresh();
  	if (!refreshed) {
  		authState.clearSession();
  		window.location.replace('/login?reason=session_expired');
  ```
  appelé par `request` (`:476-493`, `const refreshed = await refreshTokens();` à `:481`) et `requestRaw` (`:573-587`, `:578`).
  Après l'import, `/auth/refresh` rend 401 (jeton révoqué) : effacement et redirection.
- **Le panneau** — `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte` : `confirmImport` (`:143-165`) pose
  `importing = true;` (`:148`) puis `imported = await uploadFullImport(selectedFile);` (`:156`) ; en cas d'erreur,
  `unknownOutcome = issueInconnue(err);` (`:158`) ; le compte rendu est rendu sous `{#if imported}` (`:236-270`,
  `data-testid="admin-restore-files-restored"` `:250`, `"admin-restore-files-missing"` `:261`) ; `relogin` (`:176-185`)
  appelle `authState.logout()` puis `window.location.replace('/login');` (`:184`). Le commentaire d'en-tête promet (`:9-11`) :
  « Succès → avis de ce que la restauration a rendu, puis déconnexion + redirection /login au clic « Se reconnecter » ».
  L'appel : `admin-restore.api.ts:43-50`, `apiClient.postFormData<FullImportResponse>('/api/v1/admin/full-import', form, { timeoutMs: LONG_OPERATION_TIMEOUT_MS, retry: false })`.
- **Les deux chemins atteints** — le même composant sert `/admin/restore` (`frontend/src/routes/(app)/admin/restore/+page.svelte:7-10`,
  `<AdminRestorePanel />`) et l'assistant d'une installation neuve (`frontend/src/routes/onboarding/restore/+page.svelte:11`,
  `:29`, `<AdminRestorePanel />`). Les deux specs qui rougissent : `frontend/tests/e2e/backup-restore-new-install.spec.ts:140`
  (assistant, `:203` `toHaveURL(/\/onboarding\/restore$/)`, assertion du compte rendu `:212-217`) et `:262` (`/admin/restore`,
  `:282`, assertion `:286-294`). Le défaut produit atteint donc **les deux** écrans, et un utilisateur réel.
- **L'autre onglet** — un autre onglet du même navigateur, renvoyé à la connexion par son propre battement, diffuse
  `auth-change` (`frontend/src/lib/app/stores/auth.svelte.ts:198-201`) ; cet onglet-ci relance alors `hydrate()`, dont la
  branche 401 efface l'identité **sans rediriger** (`:318-333` : « UN renouvellement, sous le mutex du chemin 401, sans
  effacement ni redirection ni diffusion sur échec »). Il ne perd pas son compte rendu par ce chemin.
- **KF-056 dans la documentation** — `git grep -n "KF-056\|#931\|backup-restore-new-install" -- docs/testing.md` : vide ;
  la liste nominative de `docs/testing.md` § *Les échecs attendus* (`:372`, tableau `:396-402`) ne porte que les sept de la
  KF-029 ; la KF-056 y était tolérée **implicitement**, dans les « 0 à 2 de pollution » (`:431`). La mémoire des échecs
  attendus (`e2e-echecs-attendus.md`) ne la cite pas (`/usr/bin/grep -n "KF-056\|931\|backup-restore"` : sortie vide, code 1).
- **Le manuel** — `docs/manual/fr/admin-manual.tex:2340` décrit déjà le comportement voulu : « Un import réussi n'est plus
  suivi d'une déconnexion immédiate […] Prenez le temps de lire, puis cliquez \textbf{« Se reconnecter »} » ; le PDF le porte
  (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'Prenez le temps de lire, puis cliquez'`
  → `1`). L'assistant : `:2450` (« Restaurer une sauvegarde depuis l'assistant »). Le manuel utilisateur ne décrit pas
  l'écran (`git grep -n -i "Se reconnecter\|introuvables après la restauration" -- docs/manual/fr/user-manual.tex` : vide).
  **Le code contredit le manuel, non l'inverse** : rien à y changer (D6).

### Inventaire par le symptôme : tout ce qui renvoie à la connexion sans geste de l'utilisateur

Symptôme : « la page quitte l'écran de restauration pour `/login` alors que l'utilisateur n'a cliqué ni *Se reconnecter*
ni *Déconnexion* ». Détecteur par lecture, parti des **redirections vers `/login`**, quelle que soit leur forme
(`location.replace`, `goto`, `redirect`) :

```sh
$ git grep -n "location.replace\|location.href *=" -- frontend/src ':!*.test.ts' | /usr/bin/grep -i "login"
frontend/src/lib/app/stores/session-idle.svelte.ts:407:	window.location.replace(`/login?reason=idle&minutes=${n ?? ''}`);
frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte:184:		window.location.replace('/login');
frontend/src/lib/shared/utils/api-client.ts:202:		window.location.replace('/login?reason=session_expired');
frontend/src/routes/(app)/+layout.svelte:355:							window.location.replace('/login');
$ git grep -nE "(goto|redirect)\([^)]*'/login" -- frontend/src ':!*.test.ts'
frontend/src/lib/features/setup/SetupForm.svelte:82:						void goto('/login');
frontend/src/routes/(app)/+layout.ts:11:		throw redirect(302, '/login');
frontend/src/routes/onboarding/+layout.ts:11:		throw redirect(302, '/login');
```

**Sonde positive** : le site connu de la KF-056, `api-client.ts:202`, est rendu. Tri, ligne lue :

| Site | Atteint après l'import ? | Sort |
|---|---|---|
| `api-client.ts:202` (`refreshTokens`, appelé par `:481` et `:578`) | **oui** : tout 401 d'une requête de l'onglet — le battement, puis, après l'`exp` du jeton d'accès, toute requête de fond (`BackupScheduleBanner.svelte:34`, `setInterval(() => void refreshBackupScheduleState(), REFRESH_MS)`, `REFRESH_MS = 30 * 60_000` `:27`, rendu par `(app)/+layout.svelte:373`, donc sur `/admin/restore` seulement) | **dans le périmètre** (D1) |
| `session-idle.svelte.ts:407` (`closeSession`, à l'échéance d'inactivité) | **oui**, si l'utilisateur reste inactif N minutes pendant ou après l'import | **dans le périmètre** (D2) |
| `AdminRestorePanel.svelte:184` (`relogin`) | geste de l'utilisateur (*Se reconnecter*) | voulu, inchangé |
| `(app)/+layout.svelte:355` (menu *Déconnexion*, `onclick`) | geste de l'utilisateur | voulu, inchangé |
| `(app)/+layout.ts:11`, `onboarding/+layout.ts:11` (`load`) | seulement à une **navigation** : l'utilisateur quitte la page | hors du symptôme |
| `SetupForm.svelte:82` | page `/setup`, jamais rendue avec le panneau | hors du symptôme |

Les autres minuteries de l'application (`git grep -nE "setInterval\(" -- 'frontend/src/**/*.ts' 'frontend/src/**/*.svelte'`,
tests exclus) : `session-idle.svelte.ts:454` (le battement), `BackupScheduleBanner.svelte:34` (ci-dessus),
`api-health.svelte.ts:102` (`pollHealth`, sonde de `/health`, sans session : ne mène pas à `refreshTokens`). **Angle mort**
écrit : un `fetch` direct qui redirigerait de lui-même échapperait au premier grep s'il n'écrivait pas `/login` en clair ;
le second grep couvre `goto` et `redirect`. Aucun autre site.

## Décisions

- **D1 — Un maintien de session pendant la restauration, porté par `authState`, lu par le chemin 401 du client**
  (C-36-5-1). `auth.svelte.ts` gagne un état non réactif `restoreHold` (booléen) et deux méthodes,
  `holdForRestore()` et `releaseRestoreHold()`, avec un accesseur `get restoreHold()`. `login()` et `logout()` le remettent
  à `false`. Sous maintien, `refreshTokens` (`api-client.ts:198`) — ou ses deux appelants `:481` et `:578`, au choix du
  développeur, sans duplication — **ne renouvelle pas, n'efface pas, ne redirige pas** : la requête rejetée jette
  l'`ApiError` `UNAUTHENTICATED` existante (`:487-492`, `:582-587`). Pourquoi `authState` : `api-client.ts` (`:15`) et
  `session-idle.svelte.ts` (`:51`) l'importent déjà — aucun cycle neuf. *Écartées* : (a) ignorer **tout** 401 du battement
  (C-e360-8 (b) : affaiblirait #876) ; (b) arrêter seulement le module de session (`sessionIdle.stop()`) au clic : laisse
  le bandeau des sauvegardes rediriger après l'`exp` du jeton d'accès, et un battement déjà parti au moment du clic ; (c)
  garder le compte rendu en `sessionStorage` et le montrer après la reconnexion (piste de #931) : surface plus large, et
  l'utilisateur le lirait sur l'installation restaurée, hors de son contexte ; (d) corriger la spec seule (C-e360-8 (a)).
- **D2 — Sous maintien, le module de session note l'activité et bat, mais ne ferme rien et n'avertit pas.** Dans
  `session-idle.svelte.ts`, `tick` (`:326`) ne ferme pas (`:331-333`) et met `warningRemainingMs` à `null` tant que
  `authState.restoreHold` est vrai ; `recordActivity` (`:250-252`) et `stayActive` (`:470-472`) ne ferment pas non plus. Le
  battement **continue** pendant l'import : il prolonge la session au serveur tant que l'import n'a pas abouti (un import
  long qui échoue retrouve une session vivante), et son 401 éventuel, après l'invalidation, est absorbé par D1. À la levée
  du maintien, le tic suivant ferme si l'échéance est passée : la fermeture est **différée**, jamais annulée. Le serveur
  garde son contrôle strict d'`idle_exp` (`middleware/auth.rs:156-161`), que rien ici ne touche. *Écartée* : suspendre aussi
  le battement pendant l'import — une restauration longue qui échoue trouverait la session close au serveur (C-36-5-2).
- **D3 — Le panneau pose et lève le maintien** (C-36-5-2). Dans `confirmImport` (`AdminRestorePanel.svelte:143`) :
  `authState.holdForRestore()` **avant** l'appel à `uploadFullImport` (après `importing = true`, avant tout `await`).
  - **succès** : le maintien reste ; le panneau appelle `sessionIdle.stop()` — la session n'existe plus, il n'y a rien à
    prolonger ni à fermer, et plus aucun battement ne part ;
  - **issue inconnue** (`unknownOutcome` vrai, `:158`) : le maintien reste (l'installation est peut-être remplacée) ; le
    module de session continue (la session est peut-être vivante) ; *Se reconnecter* tranche, comme aujourd'hui ;
  - **échec certain** (toute autre erreur) : `authState.releaseRestoreHold()` dans le `catch` ; tout reprend comme avant ;
  - **le panneau quitte l'écran** (navigation dans l'application, lien *Retour à l'assistant*) : levée dans le nettoyage du
    composant (`$effect(() => () => …)` ou `onDestroy`). La requête suivante qui rend 401 renvoie alors à la connexion par
    le chemin ordinaire. *Se reconnecter* n'a rien à lever : `logout()` remet à zéro (D1), et la navigation recharge tout.
  *Écartée* : lever le maintien dès la réponse reçue — c'est précisément après elle que le bandeau ou le battement
  renverraient à la connexion.
- **D4 — La preuve par répétition, N = 30** (C-e360-8, C-36-5-4). Avant le correctif, sur le code de `4de0bcc7e` (ou la
  branche, correctif mis de côté) : `backup-restore-new-install.spec.ts:140` **et** `:262`, `--repeat-each=30 --retries=0`
  chacun (60 exécutions), **au moins un rouge attendu**, journal `kesh-gate-logs/e36-5/repetition-avant.log`. Fréquence
  relevée sur #931 : `:262` rouge une fois sur trois passages isolés, `:140` trois sur trois verts isolés (deux relevés) :
  P(aucun rouge en 30 passages de `:262`) ≈ (2/3)^30 < 10⁻⁵ si la fréquence est celle-là. **Si aucun rouge ne paraît**, le
  dire au Dev Agent Record tel quel, sans relancer jusqu'à en obtenir un : la reproduction repose alors sur le test
  déterministe de D5, dont le rouge est obligatoire. Après le correctif : les mêmes 60, plus le test de D5 `--repeat-each=10`,
  **zéro rouge**, `repetition-apres.log`. *Écartée* : N = 20 (le minimum de C-e360-8) — la fréquence de `:140` seule est
  inconnue, et dix passages de plus de chaque côté coûtent peu.
- **D5 — Un E2E déterministe qui rejoue la course** (C-36-5-3). Dans `backup-restore-new-install.spec.ts`, un test neuf,
  **à la fin du fichier** (les lignes `:140` et `:262` de D4 ne bougent pas), sur `/admin/restore` : export, puis import
  dont la réponse est **retenue** par `page.route('**/api/v1/admin/full-import', …)` — `await route.fetch()` (le serveur a
  invalidé la session), puis, avant `route.fulfill({ response })` : `localStorage.removeItem('kesh:session:heartbeatAt')`
  (`KEY_HEARTBEAT_AT`, `session-idle.svelte.ts:58`) par `page.evaluate`, deux `page.mouse.move` à des coordonnées
  distinctes, et l'attente d'au moins un `POST /api/v1/auth/activity` répondu (`page.waitForResponse`, statut 401). Puis le
  compte rendu est attendu, l'URL est toujours `/admin/restore`. Ensuite, compte rendu affiché : nouveau
  `removeItem('kesh:session:heartbeatAt')`, nouveaux mouvements, un compteur des requêtes `**/api/v1/auth/activity` posé à
  zéro, une attente bornée de 3 s (trois tics de la minuterie d'une seconde, commentée comme telle : on prouve une
  **absence**), puis : compteur à zéro, URL inchangée, compte rendu visible. Avant le correctif : le battement de l'étape
  retenue rend 401, le client redirige, le test rougit à coup sûr. ⚠️ `route.fulfill` doit être enveloppé d'un
  `try/catch` : avant le correctif, la page est déjà partie quand il s'exécute. *Écartées* : (a) `page.clock` pour avancer
  la minuterie — il fausse aussi `Date.now` dans tout le module de session et dans les dates de l'écran ; (b) le même test
  sur l'assistant : `/setup` et l'assistant à chaque passage, pour le même composant ; le chemin de l'assistant est couvert
  par la répétition de `:140` et par les tests Vitest du panneau, indépendants de la route.
- **D6 — Manuel inchangé ; `docs/testing.md` reçoit la note de la KF-056** (C-36-5-5). Le manuel d'administration
  (`:2340`) décrit déjà le comportement que le correctif rétablit ; le PDF n'a pas à être régénéré. Dans `docs/testing.md`,
  après la note historique de la KF-052 (`:417`), une note « KF-056 (#931), corrigée par la Story 36-5 » : la cause, la
  preuve (les deux journaux de D4, le test de D5), et ce qui n'est pas établi (zéro rouge en N passages est compatible avec
  un défaut plus rare). La phrase des « 0 à 2 de pollution » (`:431`) n'est pas modifiée : elle ne nomme pas la KF-056.

## Critères d'acceptation

1. **AC1 — Pendant l'import, un 401 ne renvoie pas à la connexion** (E2E de D5, première partie ; `api-client.test.ts`) :
   sous maintien, une requête qui rend 401 jette `UNAUTHENTICATED` **sans** appel à `/api/v1/auth/refresh`, **sans**
   `clearSession`, **sans** `location.replace` ; à l'E2E, le battement 401 de l'étape retenue laisse la page sur
   `/admin/restore` et le compte rendu s'affiche.
2. **AC2 — Après un import réussi, le compte rendu reste** (E2E de D5, seconde partie ; `AdminRestorePanel.test.ts`) : aucun
   `POST /api/v1/auth/activity` ne part plus (compteur à zéro après 3 s d'activité), l'URL ne change pas ; au Vitest, le
   maintien est posé **avant** l'appel à `uploadFullImport` (ordre des appels des deux simulacres), reste posé au succès, et
   `sessionIdle.stop` est appelé.
3. **AC3 — Échec certain : tout reprend** (`AdminRestorePanel.test.ts`, `session-idle.svelte.test.ts`) : pour une `ApiError`
   400 (`INVALID_BACKUP_STRUCTURE`), le maintien est levé dans le `catch` ; au module de session, une échéance passée sous
   maintien ne ferme rien, puis ferme au tic qui suit la levée (`/login?reason=idle&minutes=N`, déconnexion au serveur
   d'abord, comme `:150-170` du test existant).
4. **AC4 — Issue inconnue : le maintien reste** (`AdminRestorePanel.test.ts`) : pour une erreur `TIMEOUT` ou un 504
   (`isUnknownOutcome`), le maintien reste posé, *Se reconnecter* est proposé et le bouton d'import désactivé, comme
   aujourd'hui (`backup-restore-new-install.spec.ts:336-341`).
5. **AC5 — Quitter le panneau lève le maintien** (`AdminRestorePanel.test.ts`) : démonter le composant après un succès
   appelle `releaseRestoreHold` ; une requête 401 qui suit (Vitest du client) renvoie à `/login?reason=session_expired`.
6. **AC6 — La fermeture ordinaire est intacte (#876, #895)** : les tests existants de `session-idle.svelte.test.ts` et les
   tests 401 de `api-client.test.ts` (`:246-267`, `:327-343`, `:488`, `:567-581`) passent **sans modification** ; au module
   de session, sous maintien, l'avertissement vaut `null`, puis réapparaît au tic qui suit la levée si l'échéance est à
   moins d'une minute ; à `auth.svelte.test.ts`, `login()` et `logout()` remettent le maintien à `false`.
7. **AC7 — Preuve par répétition** (D4) : `repetition-avant.log` (60 exécutions, rouges comptés et nommés, ou leur absence
   dite) et `repetition-apres.log` (60 + 10 exécutions, zéro rouge), chacun avec son commit, dans `kesh-gate-logs/e36-5/`.
8. **AC8 — Documentation** (D6) : la note KF-056 dans `docs/testing.md` ; `git grep -n "KF-056" -- docs/testing.md` la
   rend ; le contrôle du PDF du manuel d'administration rend toujours `1`.
9. **AC9 — Les deux chemins** : aucune ligne propre à une route n'est ajoutée ; le panneau est le seul à poser le
   maintien (`git grep -n "holdForRestore" -- frontend/src ':!*.test.ts'` → l'appel du panneau et la définition seuls) ;
   `:140` (assistant) et `:262` (`/admin/restore`) font partie des deux répétitions.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, journal `kesh-gate-logs/e36-5/rouges.log`. Pour que les rouges portent sur le **comportement** et non sur
des exports absents, l'API du maintien (D1) est d'abord commitée **inerte** (`holdForRestore`/`releaseRestoreHold` sans
effet, `restoreHold` toujours `false`) — patron de la 35-7, AC2. Rouges au départ : AC1 (Vitest du client : la redirection
a lieu ; E2E : la page part à `/login?reason=session_expired`) ; AC2 (Vitest du panneau : maintien jamais posé, `stop`
jamais appelé ; E2E) ; AC3 (le maintien n'est jamais levé ; la fermeture n'est pas différée) ; AC5 ; AC6 (le maintien
inerte : `login()`/`logout()` passent par construction — ce test-là est **vert au départ, et se dit tel** ; il est tué par
M7). Verts au départ, et dits : AC4 (l'issue inconnue garde aujourd'hui *Se reconnecter*), les tests existants de
l'AC6. Les simulacres d'`authState` de `AdminRestorePanel.test.ts:24-29` et de `session-idle.svelte.test.ts:17-19` gagnent
les membres neufs.

**Mutations** (code commité d'abord ; script et journaux dans `kesh-gate-logs/e36-5/mut/` ; restauration par
`git restore --worktree -- <fichier>` puis `touch`) :

| | Mutation | Tuée par |
|---|---|---|
| M1 | la garde du maintien retirée du chemin 401 du client | AC1 (Vitest du client, E2E de D5) |
| M2 | `holdForRestore()` retiré du panneau | AC2 (Vitest du panneau), E2E de D5 |
| M3 | `holdForRestore()` déplacé après `await uploadFullImport(…)` | AC2 (ordre des appels) |
| M4 | la fermeture du `tick` non conditionnée au maintien | AC3 (module de session) |
| M5 | `releaseRestoreHold()` retiré du `catch` d'échec certain | AC3 (panneau) |
| M6 | la levée au démontage retirée | AC5 |
| M7 | `login()` (puis `logout()`) ne remet plus le maintien à zéro | AC6 (`auth.svelte.test.ts`) |
| M8 | `sessionIdle.stop()` retiré du succès | AC2 (Vitest du panneau ; E2E de D5, compteur) |
| M9 | la garde du client appliquée **toujours** (maintien ignoré, 401 absorbé en toute circonstance) | AC6 — les tests 401 existants du client rougissent : c'est la preuve que la fermeture ordinaire n'est pas affaiblie |

Une mutation équivalente se dit au Dev Agent Record.

## Tâches

- [ ] **Étape 0** : relire à la tête de la branche les lignes citées ci-dessus ; rejouer les deux greps de l'inventaire et
      celui des minuteries, sorties citées au Change Log ; vérifier qu'aucun fichier n'est commun avec la 36-4 fusionnée
      (D-notes, *Fichiers communs*).
- [ ] **Répétition avant** (D4) : bases `kesh_e2e_e365` recréées à neuf (migrations du worktree, seed), `npm run build`
      au commit de base, backend selon la recette E2E des consignes (port attribué par le prompt de développement),
      `KESH_TEST_MODE=true PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=… KESH_INBOX_DIR=… KESH_DOCUMENTS_DIR=…
      npx playwright test tests/e2e/backup-restore-new-install.spec.ts:140 tests/e2e/backup-restore-new-install.spec.ts:262 --repeat-each=30 --retries=0 --reporter=line`,
      sous le verrou de la voie 1 ; `frontend/test-results/` copié avant tout rejeu (traces des rouges).
- [ ] API du maintien inerte (`auth.svelte.ts`) ; tests de l'AC1 à l'AC6 et E2E de D5 ; rouges consignés.
- [ ] D1 (`auth.svelte.ts`, `api-client.ts`), D2 (`session-idle.svelte.ts`, commentaire d'en-tête `:20-22` complété :
      « aucune suspension » devient faux), D3 (`AdminRestorePanel.svelte`, commentaire d'en-tête `:9-12` complété) ;
      commentaire de `sendHeartbeat` `:363` récrit (le 401 n'a plus toujours redirigé).
- [ ] Mutations M1 à M9.
- [ ] **Répétition après** (D4) ; note de `docs/testing.md` (D6) ; une capture du compte rendu resté affiché après le
      battement forcé (`KESH_E2E_CAPTURES_DIR`, thème clair) dans `kesh-gate-logs/e36-5/captures/`.
- [ ] Gates (ci-dessous) ; `Status: review` et clé `review` au `sprint-status.yaml` dans le même commit ; prompt de la
      revue P1 à côté de la fiche.

**Gates** (consignes `e35-dev-commun.md` § *Quels gates, quand*, voie **1**) : en cours de développement, frontend ciblé
(`npm run check`, `npm run lint-i18n-ownership`, Vitest des quatre fichiers de tests touchés) ; au passage en `review`,
frontend complet (`check`, `lint-i18n-ownership`, `test:unit`, `build`), puis `scripts/test-fast.sh --ci` et
`scripts/gate-ci.sh kesh_ci_e365` (aucun fichier Rust n'est touché ; la table les exige, et un compte rendu qui s'en
dispenserait le dirait) ; aux remédiations, frontend complet ; à la clôture, `gate-ci` **et** E2E complet sur
`kesh_e2e_e365` recréée, au dernier commit de code, échecs nommés un par un contre `docs/testing.md`. Une commande lourde à
la fois (E7). Les deux répétitions ne remplacent pas l'E2E complet de la clôture.

## Note de version

*Corrigé* — Après une restauration (*Administration → Restaurer / Importer*, ou depuis l'assistant d'une installation
neuve), le compte rendu de l'import — pièces restaurées, pièces introuvables — reste affiché jusqu'à ce que vous cliquiez
*Se reconnecter* ou quittiez la page ; il pouvait disparaître aussitôt, l'écran passant à la connexion avec « session
expirée ». (refs #931)

## Dev Notes

### Hors périmètre

- Reporter le compte rendu après la reconnexion (piste de #931) : écartée, D1 (c).
- Les **autres onglets** du même navigateur sont renvoyés à la connexion à leur prochaine activité : c'est le comportement
  que le manuel décrit (`admin-manual.tex:2340`, « chacune prend fin à sa prochaine activité »), voulu.
- L'angle mort de la 31-1b sur l'échec au `COMMIT` (`AdminRestorePanel.svelte:49-57`) : inchangé ; après un tel échec
  (500, échec « certain » pour l'écran), le maintien est levé et un 401 renvoie à la connexion, comme aujourd'hui.

### Fichiers communs avec les autres stories de la phase 1

Tableau du fichier d'epic : la 36-5 n'y partage que `messages.ftl` ×4 (« possible ») et `user-manual.tex` (« possible ») —
**elle ne touche ni l'un ni l'autre** (aucune clé i18n, D6). La 36-4 touche `kesh-db`, `kesh-api`, leurs tests,
`docs/api-external.md` et les manuels ; la 36-5 touche `frontend/src/lib/app/stores/{auth,session-idle}.svelte.ts`,
`frontend/src/lib/shared/utils/api-client.ts`, `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte`, leurs
tests, `frontend/tests/e2e/backup-restore-new-install.spec.ts` et `docs/testing.md` : **aucun fichier commun**. 36-1 à
36-3 ne touchent aucun de ces fichiers (fichier d'epic, colonne *Fichiers principaux*). `CHANGELOG.md` : non écrit
(C-e360-11), la note ci-dessus le remplace. `i18n-keys.test.ts` (`sitesTotal`) : aucun `i18nMsg(` ajouté.

### Modules touchés (règle de splitting)

`frontend/src/lib/app/stores`, `frontend/src/lib/shared/utils`, `frontend/src/lib/features/admin-restore`,
`frontend/tests/e2e`, `docs` : **cinq**, dont trois de code. La règle (plus de cinq) ne s'applique pas.

### Ce qui doit rester vrai

- La session se ferme exactement à l'échéance réglée hors restauration (#876) ; l'avertissement paraît une minute avant
  (#895) ; un 401 hors maintien renvoie à `/login?reason=session_expired`. Le serveur ne change pas.
- Le maintien n'est posé que par le panneau de restauration, et ne survit ni à sa disparition, ni à `login()`/`logout()`,
  ni à un rechargement (état de module).
- Aucune donnée réelle : fixtures synthétiques ; rien sous `~/kesh/`.

### Références

- Issue #931 (commentaire du 2026-11-13 : séquence relevée) ; Story 31-1b (AC19, C-31-1-39 : pas de redirection avant le
  clic) ; Story v0172-session (#876) et v0174-session-2 (#895) ; C-e360-8.
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS* ; § *E2E (Playwright)* ; `docs/testing.md` § *Les échecs attendus*.

## Dev Agent Record

## Change Log

- 2026-11-14 — Fiche rédigée (Epic 36, phase 1), sur `4de0bcc7e`. Choix propres : `e36-choix-autonomes-36-5.md`
  (C-36-5-1 à C-36-5-6). Validation non lancée.
