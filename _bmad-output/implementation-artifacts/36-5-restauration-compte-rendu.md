# Story 36-5 : le compte rendu d'une restauration ne se perd plus sous un battement de session refusé (#931, KF-056)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-5`, branche
`story/36-5-restauration-compte-rendu`, par le workflow `bmad-create-story`. Release **0.17.8**, phase 1. **Aucune
migration**, aucun changement serveur ni d'API, aucune clé i18n. Code lu à la tête du worktree, **`4de0bcc7e`**. Étape 0 et
choix qui font foi : `e36-choix-autonomes-e360.md` (§ *Étape 0 — 36-5*, C-e360-8, C-e360-11) ; choix propres à la story :
`e36-choix-autonomes-36-5.md` (C-36-5-1 à C-36-5-7). Voie de base **1** (lignée B, après la 36-4). Validation : P1 et P2
remédiées (Change Log), passe suivante à lancer.*

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
  **Le code contredit le manuel, non l'inverse** ; seule s'y ajoute une demi-phrase sur le renvoi d'un écran laissé ouvert
  au bout de B (D6, depuis la validation P2).

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
frontend/src/routes/+layout.svelte:66:		// token) → `clearSession()` + `window.location.replace('/login?reason
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
| `routes/+layout.svelte:66` | **commentaire** (il décrit le chemin de `api-client.ts:202`) | hors du symptôme |

Les autres minuteries de l'application (`git grep -nE "setInterval\(" -- 'frontend/src/**/*.ts' 'frontend/src/**/*.svelte'`,
tests exclus) : `session-idle.svelte.ts:454` (le battement), `BackupScheduleBanner.svelte:34` (ci-dessus),
`api-health.svelte.ts:102` (`pollHealth`, sonde de `/health`, sans session : ne mène pas à `refreshTokens`). **Angle mort**
écrit : un `fetch` direct qui redirigerait de lui-même échapperait au premier grep s'il n'écrivait pas `/login` en clair ;
le second grep couvre `goto` et `redirect`. Aucun autre site.

## Décisions

- **D1 — Un maintien de session pendant la restauration, porté par `authState`, lu par le chemin 401 du client**
  (C-36-5-1, amendé à la validation P1, V1-1 et V1-3). `auth.svelte.ts` gagne un état non réactif — l'**échéance** du
  maintien, `restoreHoldUntil` (ms, `null` sans maintien) — et deux méthodes, `holdForRestore(durationMs)` (pose ou
  réarme : `restoreHoldUntil = Date.now() + durationMs`) et `releaseRestoreHold()`, avec un accesseur `get restoreHold()`
  qui vaut vrai **seulement tant que `Date.now() < restoreHoldUntil`** : le maintien expire seul (D3 fixe la durée).
  **Qui le lève** (amendé à la validation P2, V2-1 ; C-36-5-7) : `releaseRestoreHold()` (le panneau), `logout()` et
  `clearSession()` — **jamais `login()`**. Raison, lue au code : le renouvellement réussi passe par `doRefresh`, qui appelle
  `authState.login({…}, { broadcast: false })` (`api-client.ts:148-158`) — « `authState.login()` remplace l'objet à chaque
  renouvellement » (`routes/+layout.svelte:27-28`) ; si `login()` levait le maintien, un renouvellement réussi pendant
  l'import (jeton d'accès expiré à l'envoi, ou au fil d'un import long) l'effacerait, et le battement suivant rouvrirait la
  KF-056. Les appelants de `authState.login(` (`git grep -n "authState.login(" -- frontend/src ':!*.test.ts'`, sortie
  copiée, commentaires compris) :
  ```
  frontend/src/lib/features/setup/setup.api.ts:73:	authState.login(payload);
  frontend/src/lib/shared/utils/api-client.ts:83: * - Pas d'appel à `authState.login(...)` (qui voulait stocker les tokens en
  frontend/src/lib/shared/utils/api-client.ts:125:		// - success path : `authState.login(payload, {broadcast: false})` qui
  frontend/src/lib/shared/utils/api-client.ts:148:				authState.login(
  frontend/src/routes/+layout.svelte:27:	// qu'à l'IDENTIFIANT de l'utilisateur : `authState.login()` remplace l'objet
  frontend/src/routes/login/+page.svelte:51:			// signature `authState.login()` change pour un objet payload
  frontend/src/routes/login/+page.svelte:66:			authState.login({
  ```
  Trois appels réels : `api-client.ts:148` (renouvellement — ne doit pas lever) ; `setup.api.ts:73` (création du premier
  administrateur, page `/setup`) et `login/+page.svelte:66` (connexion), deux connexions réelles, rendues sur des pages où
  le panneau n'est pas monté ; les quatre autres lignes sont des commentaires. **Aucune connexion d'un autre utilisateur
  n'hérite d'un maintien** : le maintien est un état de module, propre à l'onglet ; il est levé par `logout()` (le *Se
  reconnecter* du panneau, `:180`, et le menu *Déconnexion*), par `clearSession()` (chemin 401 hors maintien, fermeture
  d'inactivité `session-idle.svelte.ts:406`), par le démontage du panneau — or atteindre `/login` ou `/setup` démonte le
  panneau, que ce soit par `location.replace` (rechargement complet : module neuf) ou par la redirection SPA d'un `load`
  (`(app)/+layout.ts:11`) — et il expire seul à B (D3) ; la fermeture du navigateur perd l'état de module. *Option
  écartée* (b) : `login()` lève le maintien sauf sur une option propre au renouvellement — un appelant futur qui oublierait
  l'option rouvrirait le défaut en silence ; la forme retenue n'a pas de cas oubliable, et chaque chemin vers une connexion
  réelle passe déjà par une levée. Sous maintien, le chemin 401 du client **tente toujours le
  renouvellement** (`sharedRefresh()`, `api-client.ts:184-191`) : réussi, la requête est rejouée comme aujourd'hui
  (`:482-485`, `:579-580`) ; **échoué, il n'efface pas la session et ne redirige pas** — `clearSession` et
  `location.replace` (`:200-203`) sont seuls suspendus — et la requête rejetée jette l'`ApiError` `UNAUTHENTICATED`
  existante (`:487-492`, `:582-587`). La garde se place dans `refreshTokens` (`:198-205`), seul lieu des deux effets, sans
  duplication. Ce que cela préserve, lu au code : **l'import envoyé avec un jeton d'accès expiré réussit encore** —
  `postFormData` passe par `request` (`:638-640`, `return request<T>(url, { method: 'POST', body: form }, false, call);`),
  dont le chemin 401 renouvelle puis rejoue avec les mêmes `options`, `FormData` compris (`:484`) ; `retry: false` ne borne
  que `fetchWithRetry` (`:388`), non ce rejeu. Après l'import, le renouvellement échoue sans trace parasite : les lignes
  restaurées sont révoquées au motif `password_change` (`admin.rs:194-196`), et `routes/auth.rs:491-492` (« Révoqué par logout,
  password_change, etc. → 401 simple ») rend « token revoked » sans passer par la détection de vol (`:467-489`, dans la
  branche `if token.revoked_reason.as_deref() == Some("rotation")` de `:449`) ; une ligne absente rend 401 de même. C'est
  le chemin qu'exécute déjà le défaut. Pourquoi `authState` : `api-client.ts` (`:15`) et
  `session-idle.svelte.ts` (`:51`) l'importent déjà — aucun cycle neuf. *Écartées* : (a) ignorer **tout** 401 du battement
  (C-e360-8 (b) : affaiblirait #876) ; (b) arrêter seulement le module de session (`sessionIdle.stop()`) au clic : laisse
  le bandeau des sauvegardes rediriger après l'`exp` du jeton d'accès, et un battement déjà parti au moment du clic ; (c)
  garder le compte rendu en `sessionStorage` et le montrer après la reconnexion (piste de #931) : surface plus large, et
  l'utilisateur le lirait sur l'installation restaurée, hors de son contexte ; (d) corriger la spec seule (C-e360-8 (a)) ;
  (e) — première rédaction de cette fiche — ne pas renouveler sous maintien : l'import envoyé avec un jeton d'accès expiré
  échouait, et le battement d'un import long cessait de prolonger la session dès l'expiration du jeton (V1-1).
- **D2 — Sous maintien, le module de session note l'activité et bat, ne ferme rien, et avertit avant la fin du
  maintien.** Dans `session-idle.svelte.ts`, `tick` (`:326`) ne ferme pas (`:331-333`) tant que `authState.restoreHold` est
  vrai ; `recordActivity` (`:250-252`) et `stayActive` (`:470-472`) ne ferment pas non plus. **L'avertissement** (#895)
  se calcule sur le moment où la fermeture aura réellement lieu (amendé à la validation P2, V2-6, conduite prudente) :
  sous maintien, `fin = max(last + w, restoreHoldUntil)` au lieu de `last + w`, et `warningRemainingMs = fin − now` dès que
  `fin − now ≤ min(WARNING_LEAD_MS, w / 2)` (`:338-340`, même seuil). Un Administrateur inactif pendant un import plus long
  que B voit donc l'avertissement une minute avant que le maintien n'expire, et *Rester connecté* (`stayActive`) repousse
  `last` : à l'expiration, l'échéance ordinaire n'est pas atteinte, rien ne se ferme. Hors maintien, la formule est celle
  d'aujourd'hui. *Écartée* : éteindre l'avertissement sous maintien (première rédaction) — à l'expiration, l'échéance était
  déjà passée et la fermeture tombait sans avertissement. Le
  battement **continue** pendant l'import : il prolonge la session au serveur tant que l'import n'a pas abouti (un import
  long qui échoue retrouve une session vivante — vrai même après l'expiration du jeton d'accès, puisque D1 renouvelle
  encore), et son 401 éventuel, après l'invalidation, est absorbé par D1. À la levée ou à l'expiration du maintien, le tic
  suivant ferme si l'échéance est passée : la fermeture est **différée**, jamais annulée ; chacune des deux phases de D3
  est bornée par B, si bien que l'écran reste ouvert au plus la durée de l'import plus B après la confirmation (V2-10) — au
  serveur, rien ne s'étend (`middleware/auth.rs:156-161`). Le serveur
  garde son contrôle strict d'`idle_exp` (`middleware/auth.rs:156-161`), que rien ici ne touche. *Écartée* : suspendre aussi
  le battement pendant l'import — une restauration longue qui échoue trouverait la session close au serveur (C-36-5-2).
- **D3 — Le panneau pose, borne et lève le maintien** (C-36-5-2, amendé à la validation P1, V1-3, V1-4, V1-9). Le
  maintien ne couvre que **deux phases** : l'import en cours, et la lecture du compte rendu d'un import réussi ; chacune est
  bornée par **B = la durée d'inactivité réglée** (`sessionIdle.minutes`, `session-idle.svelte.ts:499-501`, en minutes ;
  60 si elle est inconnue, la valeur par défaut du serveur, manuel d'administration `:2651`). *Pourquoi B* : c'est le temps
  qu'un écran inactif peut déjà rester ouvert sans fermeture, selon le réglage de l'Administrateur ; lire deux lignes de
  compte rendu en prend quelques secondes ; un import qui dure plus de B est rare (base de Guy petite) et se termine alors
  sous le régime ordinaire — la fermeture locale, si l'utilisateur est inactif, et le chemin 401 ordinaire.
  - **confirmation** (`confirmImport`, `AdminRestorePanel.svelte:143`) : `authState.holdForRestore(B)` **avant** l'appel à
    `uploadFullImport` (après `importing = true`, avant tout `await`) ;
  - **succès** : `authState.holdForRestore(B)` — le maintien est **réarmé** pour la lecture ; `sessionIdle.stop()` — la
    session n'existe plus, il n'y a rien à prolonger, et plus aucun battement ne part ; le panneau arme une minuterie de B
    qui, à l'échéance, termine la session comme le chemin 401 ordinaire : levée du maintien, `authState.clearSession()`,
    `window.location.replace('/login?reason=session_expired')`. Ces deux dernières instructions sont celles de
    `refreshTokens` (`api-client.ts:201-202`) : elles s'extraient dans une fonction exportée d'`api-client.ts`
    (`expireSession()`, nom indicatif), appelée par les deux sites (DRY). La minuterie est effacée au démontage ;
  - **issue inconnue** (`unknownOutcome` vrai, `:158`) : **le maintien est levé** — la fermeture d'inactivité et le
    chemin 401 reprennent. Si l'import a abouti, le battement suivant renvoie à la connexion, ce que le message demande
    déjà (« Reconnectez-vous pour vérifier l'état de l'installation », `:100-103`) ; s'il n'a pas eu lieu, la session vit et
    rien ne change. C'est le comportement d'aujourd'hui pour ce cas : le manuel (`:2342`) reste juste ;
  - **échec certain** (toute autre erreur) : maintien levé ;
  - **l'import lui-même refusé en 401** (V2-5, conduite prudente) : sous maintien, un renouvellement échoué fait jeter
    `UNAUTHENTICATED` (`api-client.ts:487-492`) sans renvoi ; le `catch` du panneau le reconnaît (`isApiError(err) &&
    err.code === 'UNAUTHENTICATED'`) et, la session étant morte **avant** tout import, termine la session comme le chemin
    401 ordinaire : levée du maintien puis `expireSession()` (`/login?reason=session_expired`), au lieu d'afficher « Session
    expirée » (`:130-133`, `default: return err.message;`) avec un bouton d'import réactivé. C'est le comportement
    d'aujourd'hui pour ce cas ;
  - **pas d'avertissement avant la fin de la lecture** (phase 2) : la session est déjà morte au serveur ; *Rester connecté*
    n'aurait rien à prolonger, et il n'y a rien à enregistrer. Le manuel dit ce renvoi (D6) ;
  - **levée en `finally`** (V1-9) : un indicateur local `keepHold`, posé à vrai **seulement** au succès, décide dans le
    `finally` (`:162-164`) : `if (!keepHold) authState.releaseRestoreHold();`. Une exception levée dans le `catch`
    (`issueInconnue`, `importErrorMessage`, `toast.error`, `:158-161`) ne laisse donc jamais le maintien posé ;
  - **le panneau quitte l'écran** (navigation dans l'application, lien *Retour à l'assistant*) : levée et effacement de la
    minuterie dans le nettoyage du composant (`$effect(() => () => …)` ou `onDestroy`), et un indicateur `destroyed` posé.
    La requête suivante qui rend 401 renvoie alors à la connexion par le chemin ordinaire. *Se reconnecter* n'a rien à
    lever : `logout()` remet à zéro (D1), et la navigation recharge tout ;
  - **import encore en vol au démontage** (V1-4) : `uploadFullImport` n'a pas de signal d'annulation (`:156`), et
    l'import continue au serveur. Si sa réponse arrive après le démontage (`destroyed` vrai), le panneau **ne réarme pas**
    le maintien, **n'appelle pas** `sessionIdle.stop()` et **n'arme pas** la minuterie : l'utilisateur a quitté l'écran, il
    retrouve le régime ordinaire, y compris la redirection par le battement suivant si l'import a invalidé la session.
    Limite écrite, assumée : celui qui quitte l'écran pendant l'import ne voit pas le compte rendu (il ne l'aurait pas vu
    non plus aujourd'hui).
  *Écartées* : (a) lever le maintien dès la réponse reçue — c'est précisément après elle que le bandeau ou le battement
  renverraient à la connexion ; (b) — première rédaction — garder le maintien sans borne après une issue inconnue :
  l'écran d'administration restait ouvert indéfiniment, sans fermeture ni avertissement (V1-3) ; (c) une borne fixe plus
  courte (15 min, l'`exp` par défaut du jeton d'accès) : sans rapport avec le réglage que l'Administrateur a choisi pour
  un écran inactif.
- **D4 — La preuve par répétition, N = 30** (C-e360-8, C-36-5-4). Avant le correctif, sur le code de `4de0bcc7e` (ou la
  branche, correctif mis de côté) : `backup-restore-new-install.spec.ts:140` **et** `:262`, `--repeat-each=30 --retries=0`
  chacun (60 exécutions), **au moins un rouge attendu**, journal `kesh-gate-logs/e36-5/repetition-avant.log`. Fréquence
  relevée sur #931 : `:262` rouge une fois sur trois passages isolés, `:140` trois sur trois verts isolés (deux relevés) :
  P(aucun rouge en 30 passages de `:262`) ≈ (2/3)^30 ≈ 5·10⁻⁶ **si** la fréquence est celle-là — mais elle est estimée
  sur **trois** passages, et son intervalle est très large (V1-10) : la mesure « avant » de 30 passages par test, faite au
  développement, est la première estimation qui vaille, et le Dev Agent Record en donne la fréquence observée (rouges sur
  passages, pour chaque test). **Si aucun rouge ne paraît**, le dire tel quel, sans relancer jusqu'à en obtenir un : la
  répétition « après » ne prouve alors **rien par elle-même** (zéro rouge avant, zéro après), et la preuve repose sur le
  seul test déterministe de D5, dont le rouge avant correctif est obligatoire. Après le correctif : les mêmes 60, plus le test de D5 `--repeat-each=10`,
  **zéro rouge**, `repetition-apres.log`. *Écartée* : N = 20 (le minimum de C-e360-8) — la fréquence de `:140` seule est
  inconnue, et dix passages de plus de chaque côté coûtent peu.
- **D5 — Un E2E déterministe qui rejoue la course** (C-36-5-3). Dans `backup-restore-new-install.spec.ts`, un test neuf,
  **à la fin du fichier** (les lignes `:140` et `:262` de D4 ne bougent pas), sur `/admin/restore` : export, puis import
  dont la réponse est **retenue** par `page.route('**/api/v1/admin/full-import', …)` — `await route.fetch()` (le serveur a
  invalidé la session), puis, avant `route.fulfill({ response })` : `await page.waitForTimeout(1_100)` — **obligatoire** et
  commentée : `recordActivity` ignore toute activité à moins de `ACTIVITY_WRITE_THROTTLE_MS = 1_000` ms de la précédente
  (`session-idle.svelte.ts:67`, `:254`), et le clic *Confirmer* vient d'en écrire une ; sans cette attente, un import court
  laisse les mouvements suivants sans effet, aucun battement ne part et le test rougit **après** le correctif, ou rougit
  avant pour la mauvaise raison (V1-2) — ; puis `localStorage.removeItem('kesh:session:heartbeatAt')`
  (`KEY_HEARTBEAT_AT`, `session-idle.svelte.ts:58`) par `page.evaluate`, deux `page.mouse.move` à des coordonnées
  distinctes, et l'attente d'au moins un `POST /api/v1/auth/activity` répondu (`page.waitForResponse`, statut 401) — la
  réponse de l'import est ainsi retenue plus d'une seconde, quelle que soit la durée réelle de l'import. Puis le compte
  rendu est attendu, l'URL est toujours `/admin/restore`. Ensuite, compte rendu affiché : de nouveau
  `waitForTimeout(1_100)` (même raison), `removeItem('kesh:session:heartbeatAt')`, nouveaux mouvements, un compteur des
  requêtes `**/api/v1/auth/activity` posé à zéro **avant** ces mouvements, une attente bornée de 3 s (trois tics de la
  minuterie d'une seconde, commentée comme telle : on prouve une **absence**), puis : compteur à zéro, URL inchangée,
  compte rendu visible. **M8 est tuée** par cette seconde partie : sans `sessionIdle.stop()`, le module tourne encore ;
  l'échec du premier battement a remis `reportedActivity` à sa valeur d'avant (`:367-373`), les mouvements écrivent une
  activité neuve hors régulation, `heartbeatAt` est retirée : le tic suivant envoie un battement, le compteur vaut 1 au
  moins. Avant le correctif : le battement de l'étape retenue rend 401, le renouvellement échoue, le client redirige, le
  test rougit à coup sûr. ⚠️ `route.fulfill` doit être enveloppé d'un
  `try/catch` : avant le correctif, la page est déjà partie quand il s'exécute. *Écartées* : (a) `page.clock` pour avancer
  la minuterie — il fausse aussi `Date.now` dans tout le module de session et dans les dates de l'écran ; (b) le même test
  sur l'assistant : `/setup` et l'assistant à chaque passage, pour le même composant ; le chemin de l'assistant est couvert
  par la répétition de `:140` et par les tests Vitest du panneau, indépendants de la route.
- **D6 — Une demi-phrase au manuel d'administration ; `docs/testing.md` reçoit la note de la KF-056** (C-36-5-5, amendé à
  la validation P2, V2-7). Le manuel d'administration (`:2340`) décrit déjà le comportement que le correctif rétablit
  (« Prenez le temps de lire, puis cliquez « Se reconnecter » ») ; ce qui est neuf et visible, c'est le renvoi à la
  connexion d'un écran laissé ouvert au bout de B (D3). Après « Prenez le temps de lire, puis cliquez \textbf{« Se
  reconnecter »} : la session est déjà invalidée, et l'écran de connexion attend les identifiants \textbf{de la
  sauvegarde}. », ajouter : « Laissé ouvert, l'écran y revient de lui-même au bout de la durée d'inactivité réglée
  (\S\ref{sec:duree-inactivite}). » Le `.tex` change dans la story ; **le PDF se régénère à l'intégration**, après l'union
  avec la 36-4 (qui touche aussi ce manuel, Dev Notes), et se contrôle aplati. L'issue inconnue (`:2342`) garde le
  comportement d'aujourd'hui (D3). Dans `docs/testing.md`, une note « KF-056 (#931), corrigée par la
  Story 36-5 », placée **après les deux paragraphes que la note de la KF-052 annonce** (`:417` finit par « Les deux
  paragraphes suivants restent tels qu'écrits à leur date. » ; ces paragraphes sont `:419-422` et `:424-429`), donc juste
  avant `⇒ **Le compte attendu…**` (`:431`) — une insertion après `:417` casserait cette référence (V1-7). Contenu : la
  cause, la preuve (les deux journaux de D4, le test de D5), et ce qui n'est pas établi (zéro rouge en N passages est
  compatible avec un défaut plus rare). Elle dit aussi que la KF-056 ne figurait dans aucune liste : qu'elle ait été
  absorbée par les « 0 à 2 de pollution » (`:431`) est une **inférence** (ses rouges, quand ils sont tombés dans une suite
  complète, ont pu être comptés là), que rien au dépôt ne confirme. La phrase de `:431` n'est pas modifiée.

## Critères d'acceptation

1. **AC1 — Pendant l'import, un 401 ne renvoie pas à la connexion** (E2E de D5, première partie ; `api-client.test.ts`) :
   (a) sous maintien, une requête qui rend 401 puis dont le renouvellement échoue jette `UNAUTHENTICATED` **sans**
   `clearSession` et **sans** `location.replace` — le `POST /api/v1/auth/refresh` est bien tenté ; (b) sous maintien, 401
   puis renouvellement réussi : la requête est **rejouée** et rend sa réponse (rien n'est absorbé) — en particulier
   `postFormData` rejoue le même `FormData` (import envoyé avec un jeton d'accès expiré) ; à l'E2E, le battement 401 de
   l'étape retenue laisse la page sur `/admin/restore` et le compte rendu s'affiche.
1 bis. **AC1 bis — Un renouvellement réussi ne lève pas le maintien** (V2-1 ; `api-client.test.ts`, vrai `authState`) :
   sous maintien, une requête rend 401, `/auth/refresh` rend 200, `/auth/me` rend 200 (donc `authState.login(…)` est
   appelé, `api-client.ts:148`), la requête est rejouée ; **ensuite** `authState.restoreHold` est toujours vrai, et une
   seconde requête qui rend 401, renouvellement refusé, jette `UNAUTHENTICATED` **sans** `clearSession` ni
   `location.replace`. Les tests qui posent un maintien le lèvent en `afterEach` (`authState.releaseRestoreHold()`) :
   l'état est au module, et `login()` en tête du test suivant ne le nettoie plus.
2. **AC2 — Après un import réussi, le compte rendu reste** (E2E de D5, seconde partie ; `AdminRestorePanel.test.ts`) : aucun
   `POST /api/v1/auth/activity` ne part plus (compteur à zéro après 3 s d'activité), l'URL ne change pas ; au Vitest, le
   maintien est posé **avant** l'appel à `uploadFullImport` (ordre des appels des deux simulacres), réarmé au succès avec
   la durée B, et `sessionIdle.stop` est appelé.
3. **AC3 — Échec certain : tout reprend** (`AdminRestorePanel.test.ts`, `session-idle.svelte.test.ts`) : pour une `ApiError`
   400 (`INVALID_BACKUP_STRUCTURE`), le maintien est levé ; la levée se fait dans le `finally` (constat de lecture à la
   revue, ligne citée : le cas « `toast.error` jette » n'est **pas** joué en Vitest — l'exception sortirait d'un `onclick`
   sans `await`, en rejet non géré que Vitest compte comme erreur avant comme après le correctif, et le test ne
   discriminerait rien ; V2-3) ; au module de session, une échéance passée sous maintien ne ferme
   rien — ni au `tick`, ni à `recordActivity` (un `keydown`), ni à `stayActive()` —, puis ferme au tic qui suit la levée
   (`/login?reason=idle&minutes=N`, déconnexion au serveur d'abord, comme `:150-170` du test existant).
4. **AC4 — Issue inconnue : la fermeture ordinaire reprend** (`AdminRestorePanel.test.ts`) : pour une erreur `TIMEOUT` ou
   un 504 (`isUnknownOutcome`), le maintien est **levé**, `sessionIdle.stop` n'est **pas** appelé, *Se reconnecter* est
   proposé et le bouton d'import désactivé, comme aujourd'hui (`backup-restore-new-install.spec.ts:336-341`).
4 bis. **AC4 bis — L'import refusé en 401 renvoie à la connexion** (V2-5 ; `AdminRestorePanel.test.ts`) : `uploadFullImport`
   rejette `{ code: 'UNAUTHENTICATED', status: 401, message: 'Session expirée' }` → `releaseRestoreHold` puis `expireSession`
   appelés, dans cet ordre ; aucun `admin-restore-error` rendu.
5. **AC5 — Quitter le panneau lève le maintien** (`AdminRestorePanel.test.ts`) : (a) démonter le composant après un succès
   appelle `releaseRestoreHold` et efface la minuterie de B (avancer les minuteries de B ensuite : aucune redirection) ;
   une requête 401 qui suit (Vitest du client) renvoie à `/login?reason=session_expired` ; (b) démonter **pendant**
   l'import (promesse de `uploadFullImport` pendante), puis la résoudre en succès : aucune exception, `holdForRestore`
   appelé une seule fois (à la confirmation), `sessionIdle.stop` jamais appelé, aucune minuterie armée.
5 bis. **AC5 bis — Le maintien est borné** (`auth.svelte.test.ts`, `api-client.test.ts`, `AdminRestorePanel.test.ts`,
   minuteries simulées) : (a) `holdForRestore(B)` : `restoreHold` vrai à `B − 1` ms, faux à `B` ; un 401 dont le
   renouvellement échoue, à `B`, efface et redirige comme hors maintien ; (b) au panneau, après un succès, la minuterie de
   B appelle `releaseRestoreHold`, `clearSession` et `location.replace('/login?reason=session_expired')` (par la fonction
   extraite), et rien avant B ; (c) B vaut `sessionIdle.minutes × 60 000` ms, et 60 minutes quand `minutes` est `null`.
6. **AC6 — La fermeture ordinaire est intacte (#876, #895)** : les tests existants de `session-idle.svelte.test.ts` et les
   tests 401 de `api-client.test.ts` (`:246-267`, `:327-343`, `:488`, `:567-581`) passent **sans modification** ; au module
   de session, sous maintien, l'avertissement suit D2 : avec `last + w` passé et `restoreHoldUntil − now = 30 000` ms, il
   vaut 30 000 ; `stayActive()` l'efface, et à l'expiration du maintien rien ne se ferme (l'échéance ordinaire a été
   repoussée) ; sans `stayActive`, la fermeture tombe au tic qui suit l'expiration, l'avertissement ayant paru avant ; à
   `auth.svelte.test.ts`, `logout()` et `clearSession()` remettent le maintien à zéro (`restoreHold` faux), **`login()` ne
   le touche pas** (maintien posé, `login()`, `restoreHold` toujours vrai).
7. **AC7 — Preuve par répétition** (D4) : `repetition-avant.log` (60 exécutions, 30 par test, rouges comptés et nommés
   avec la fréquence observée de chaque test, ou leur absence dite) et `repetition-apres.log` (60 + 10 exécutions, zéro
   rouge), chacun avec son commit, dans `kesh-gate-logs/e36-5/`. Sans rouge avant, le Dev Agent Record écrit que la
   répétition « après » ne prouve rien par elle-même et que la preuve est celle du test de D5 (rouge avant, vert après).
8. **AC8 — Documentation** (D6) : la note KF-056 dans `docs/testing.md` ; `git grep -n "KF-056" -- docs/testing.md` la
   rend ; la demi-phrase de D6 au `.tex` (`git grep -n "y revient de lui-même" -- docs/manual/fr/admin-manual.tex` → une
   ligne) ; le contrôle du PDF (« Prenez le temps de lire, puis cliquez ») rend toujours `1` ; le PDF régénéré à
   l'intégration rend aussi `1` pour « y revient de lui-même au bout de la durée d'inactivité réglée ».
9. **AC9 — Les deux chemins** : aucune ligne propre à une route n'est ajoutée ; le panneau est le seul à poser le
   maintien (`git grep -n "holdForRestore" -- frontend/src ':!*.test.ts'` → la définition et les deux appels du panneau, confirmation et succès, seuls) ;
   `:140` (assistant) et `:262` (`/admin/restore`) font partie des deux répétitions.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, journal `kesh-gate-logs/e36-5/rouges.log`. Pour que les rouges portent sur le **comportement** et non sur
des exports absents, l'API du maintien (D1) est d'abord commitée **inerte** (`holdForRestore`/`releaseRestoreHold` sans
effet, `restoreHold` toujours `false`) — patron de la 35-7, AC2. Rouges au départ : AC1 (Vitest du client : la redirection
a lieu ; E2E : la page part à `/login?reason=session_expired`) ; AC2 (Vitest du panneau : maintien jamais posé, `stop`
jamais appelé ; E2E) ; AC3 (le maintien n'est jamais levé ; la fermeture n'est pas différée) ; AC4 (levée jamais
appelée ; ce qui y est d'aujourd'hui — *Se reconnecter*, bouton désactivé — est vert, et se dit tel) ; AC5 ; AC5 bis ;
AC1 (b) est **vert au départ** (le rejeu existe déjà) et se dit tel : il garde le rejeu contre M1′. AC1 bis : rouge au
départ (maintien inerte, la seconde requête redirige). AC4 bis : rouge (le message « Session expirée » est rendu). AC6 :
« `login()` ne touche pas le maintien » est rouge sur l'API inerte (`restoreHold` toujours faux) ; « `logout()`/
`clearSession()` le remettent à zéro » est **vert au départ par construction, et se dit tel** — tué par M7′. Verts au
départ, et dits : les tests existants de l'AC6.

**Simulacres** (V1-5) : ceux d'`authState` de `AdminRestorePanel.test.ts:24-29` et de `session-idle.svelte.test.ts:17-19`
gagnent `holdForRestore`, `releaseRestoreHold`, `restoreHold` (et `clearSession` pour le panneau) ; `AdminRestorePanel.test.ts`
gagne un `vi.mock('$lib/app/stores/session-idle.svelte', () => ({ sessionIdle: { stop: vi.fn(), get minutes() { … } } }))`
— le module réel importerait `api-client` et poserait des minuteries — et un simulacre **partiel** d'`api-client`, qui
n'a aujourd'hui aucun `vi.mock` dans ce fichier (`AdminRestorePanel.test.ts:9-29` : `./admin-restore.api`, `i18n.svelte`,
`svelte-sonner`, `auth.svelte`) et dont le panneau emploie `isApiError` (`AdminRestorePanel.svelte:20`, `:105`) — un
simulacre entier ferait rougir les tests de messages d'erreur existants (V2-4) ; à écrire tel quel :
`vi.mock('$lib/shared/utils/api-client', async (orig) => ({ ...(await orig<typeof import('$lib/shared/utils/api-client')>()), expireSession: vi.fn() }));`. Les tests de B et de la minuterie emploient
`vi.useFakeTimers()`.

**Mutations** (code commité d'abord ; script et journaux dans `kesh-gate-logs/e36-5/mut/` ; restauration par
`git restore --worktree -- <fichier>` puis `touch`) :

| | Mutation | Tuée par |
|---|---|---|
| M1 | la garde du maintien retirée de `refreshTokens` (`clearSession` + redirection même sous maintien) | AC1 (a) (Vitest du client, E2E de D5) |
| M1′ | sous maintien, le renouvellement n'est plus tenté (rejet direct en `UNAUTHENTICATED`) | AC1 (b) (rejeu attendu, et `refresh` appelé en AC1 (a)) |
| M2 | `holdForRestore()` retiré de la confirmation | AC2 (Vitest du panneau), E2E de D5 |
| M3 | `holdForRestore()` déplacé après `await uploadFullImport(…)` | AC2 (ordre des appels) |
| M4 | la fermeture du `tick` non conditionnée au maintien | AC3 (module de session) |
| M5 | `releaseRestoreHold()` retiré du `finally` (échec certain) | AC3 (panneau) |
| ~~M5′~~ | *retirée à la validation P2 (V2-3)* : sans test qui la discrimine (rejet non géré, AC3) ; la place de la levée dans le `finally` est un constat de lecture de la revue | — |
| M6 | la levée au démontage retirée | AC5 (a) |
| M6′ | la garde `destroyed` retirée (succès après démontage) | AC5 (b) |
| M7 | `login()` remet le maintien à zéro (la forme de la première rédaction, V2-1) | AC1 bis (`api-client.test.ts` : après le renouvellement réussi, la seconde requête redirige) ; AC6 (`auth.svelte.test.ts`) |
| M7′ | `logout()` (puis `clearSession()`) ne remet plus le maintien à zéro | AC6 (`auth.svelte.test.ts`) |
| M8 | `sessionIdle.stop()` retiré du succès | AC2 (Vitest du panneau ; E2E de D5, seconde partie : compteur ≥ 1, cf. D5) |
| M9 | la garde du client appliquée **toujours** (maintien ignoré : jamais d'effacement ni de redirection après un renouvellement échoué) | AC6 — les tests 401 existants du client (`:246-267`, `:327-343`, `:567-581`) rougissent : la preuve que la fermeture ordinaire n'est pas affaiblie |
| M10 | la fermeture de `recordActivity` non conditionnée au maintien | AC3 (`keydown` sous maintien, échéance passée) |
| M11 | la fermeture de `stayActive` non conditionnée au maintien | AC3 (`stayActive()` sous maintien) |
| M12 | sous maintien, l'avertissement calculé sur `last + w` (formule d'aujourd'hui) au lieu de `max(last + w, restoreHoldUntil)` | AC6 (avertissement à 30 000 ms de l'expiration, `last + w` passé : il vaudrait un nombre négatif ou paraîtrait trop tôt) |
| M12′ | sous maintien, l'avertissement éteint (`null`, première rédaction) | AC6 (avertissement attendu à 30 000) |
| M13 | l'accesseur `restoreHold` ignore l'échéance (maintien sans borne) | AC5 bis (a) |
| M14 | la minuterie de B retirée du succès | AC5 bis (b) |
| M15 | le maintien gardé à l'issue inconnue (levée retirée pour ce cas) | AC4 |
| M16 | le cas `UNAUTHENTICATED` de l'import retiré du `catch` (message affiché, pas d'`expireSession`) | AC4 bis |

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
- [ ] D1 (`auth.svelte.ts`, `api-client.ts`, dont l'extraction d'`expireSession`), D2 (`session-idle.svelte.ts`, commentaire d'en-tête `:20-22` complété :
      « aucune suspension » devient faux), D3 (`AdminRestorePanel.svelte`, commentaire d'en-tête `:9-12` complété) ;
      commentaire de `sendHeartbeat` `:363` récrit (le 401 n'a plus toujours redirigé).
- [ ] Mutations M1 à M16 (avec M1′, M6′, M7′, M12′ ; M5′ retirée).
- [ ] **Répétition après** (D4) ; note de `docs/testing.md` et demi-phrase de `admin-manual.tex:2340` (D6 ; PDF à
      l'intégration) ; une capture du compte rendu resté affiché après le
      battement forcé (`KESH_E2E_CAPTURES_DIR`, thème clair) dans `kesh-gate-logs/e36-5/captures/`.
- [ ] Gates (ci-dessous) ; `Status: review` et clé `review` au `sprint-status.yaml` dans le même commit ; prompt de la
      revue P1 à côté de la fiche.

**Gates** (consignes `e35-dev-commun.md` § *Quels gates, quand*, voie **1**) : en cours de développement, frontend ciblé
(`npm run check`, `npm run lint-i18n-ownership`, Vitest des quatre fichiers de tests touchés) ; au passage en `review`,
frontend complet (`check`, `lint-i18n-ownership`, `test:unit`, `build`), puis `scripts/test-fast.sh --ci` et
`scripts/gate-ci.sh kesh_ci_e365` — **maintenus bien qu'aucun fichier Rust ne soit touché**, parce qu'un test Rust lit
les sources du frontend : `crates/kesh-api/tests/textes_coherents.rs:593` (`sources.extend(fichiers_sous("frontend/src", …`)
contrôle les textes de tous les `.svelte` et `.ts` hors tests ; un commentaire ou un texte neuf des quatre fichiers touchés
peut le faire rougir sans que Vitest ni l'E2E le voient (validation P1, axe 4) ; aux remédiations,
`scripts/test-fast.sh --ci` (bases remises à neuf) **et** frontend complet (`e35-dev-commun.md:12` ; même raison,
`textes_coherents.rs:593`, V2-2) ; à la clôture, `gate-ci` **et** E2E complet sur
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
- **Cet onglet, quand un autre onglet est renvoyé à la connexion** (V1-6) : la diffusion `auth-change` fait relancer
  `hydrate()`, dont la branche 401 met `_currentUser = null` **avant** `tryRefresh()` (`auth.svelte.ts:322-330`) ; le
  layout racine arrête alors le module de session (`routes/+layout.svelte:33-42`, `sessionUserId` nul), et les
  `isAdmin` dérivés basculent à faux (`BackupScheduleBanner.svelte:29`, `(app)/+layout.svelte:45`) : le compte rendu
  reste (aucune redirection), mais le menu d'administration disparaît autour de lui. Hors périmètre, sans test ; **issue
  demandée à l'orchestrateur** (une par problème, `technical-debt` ou `bug` P4 à son appréciation : comportement visible
  mineur).
- L'angle mort de la 31-1b sur l'échec au `COMMIT` (`AdminRestorePanel.svelte:49-57`) : inchangé ; après un tel échec
  (500, échec « certain » pour l'écran), le maintien est levé et un 401 renvoie à la connexion, comme aujourd'hui.

### Fichiers communs avec les autres stories de la phase 1

Tableau du fichier d'epic : la 36-5 n'y partage que `messages.ftl` ×4 (« possible ») et `user-manual.tex` (« possible ») —
**elle ne touche ni l'un ni l'autre** (aucune clé i18n, D6). La 36-4 touche `kesh-db`, `kesh-api`, leurs tests,
`docs/api-external.md` et les manuels ; la 36-5 touche `frontend/src/lib/app/stores/{auth,session-idle}.svelte.ts`,
`frontend/src/lib/shared/utils/api-client.ts`, `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte`, leurs
tests, `frontend/tests/e2e/backup-restore-new-install.spec.ts`, `docs/testing.md` et — depuis la validation P2 (V2-7) —
`docs/manual/fr/admin-manual.tex` (`:2340`, une demi-phrase). **Un fichier commun possible avec la 36-4** : le manuel
d'administration, si elle y écrit la phrase de l'attribution (fichier d'epic : « 36-4 (attribution, avec
`admin-manual.tex`) ») — paragraphes distincts (la 36-5 ne touche que `:2340`, section de la restauration) ; celle qui
part la seconde relit la ligne après la fusion de la première ; **le PDF se régénère après l'union**, à l'intégration, et
se contrôle aplati. Aucun autre fichier commun. 36-1 à
36-3 ne touchent aucun de ces fichiers (fichier d'epic, colonne *Fichiers principaux*). `CHANGELOG.md` : non écrit
(C-e360-11), la note ci-dessus le remplace. `i18n-keys.test.ts` (`sitesTotal`) : aucun `i18nMsg(` ajouté.

### Modules touchés (règle de splitting)

`frontend/src/lib/app/stores`, `frontend/src/lib/shared/utils`, `frontend/src/lib/features/admin-restore`,
`frontend/tests/e2e`, `docs` : **cinq**, dont trois de code. La règle (plus de cinq) ne s'applique pas.

### Ce qui doit rester vrai

- La session se ferme exactement à l'échéance réglée hors restauration (#876) ; l'avertissement paraît une minute avant
  (#895) ; un 401 hors maintien renvoie à `/login?reason=session_expired`. Le serveur ne change pas.
- Le maintien n'est posé que par le panneau de restauration, ne couvre que l'import en cours et la lecture du compte
  rendu, chacune bornée par B (la durée d'inactivité réglée) — au total, l'écran reste ouvert au plus la durée de l'import
  plus B après la confirmation —, et ne survit ni à sa disparition, ni à une issue inconnue, ni à `logout()`, ni à
  `clearSession()`, ni à un rechargement (état de module). **Un renouvellement réussi ne le lève jamais** (`login()` ne le
  touche pas, V2-1) ; aucune connexion d'un autre utilisateur n'en hérite (D1). Sous maintien, le renouvellement est
  toujours tenté.
- Aucune donnée réelle : fixtures synthétiques ; rien sous `~/kesh/`.

### Références

- Issue #931 (commentaire du 2026-11-13 : séquence relevée) ; Story 31-1b (AC19, C-31-1-39 : pas de redirection avant le
  clic) ; Story v0172-session (#876) et v0174-session-2 (#895) ; C-e360-8.
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS* ; § *E2E (Playwright)* ; `docs/testing.md` § *Les échecs attendus*.

## Dev Agent Record

## Change Log

- 2026-11-14 — Fiche rédigée (Epic 36, phase 1), sur `4de0bcc7e`. Choix propres : `e36-choix-autonomes-36-5.md`
  (C-36-5-1 à C-36-5-6). Validation non lancée.
- 2026-11-14 — **Validation P1 (Sonnet) : 0/0/3/7 → remédiés** (`kesh-gate-logs/e36-5/validation-p1.md`), par l'agent de
  rédaction (Opus 5.5). Prémisses relues au code avant d'appliquer, toutes confirmées :
  - V1-1 (MEDIUM) : `api-client.ts:198-205` et `:481-485` (le chemin 401 seul renouvelle ; la première rédaction le
    supprimait sous maintien) ; `postFormData` `:638-640` passe par `request`, dont le rejeu renvoie le même `FormData`
    (`:484`) ; `routes/auth.rs:449`, `:467-489`, `:491-492` (pas de détection de vol pour une ligne révoquée en
    `password_change`, `admin.rs:194-196`). → D1 récrit (renouvellement toujours tenté, seuls `clearSession` et la
    redirection suspendus), D2 complété, AC1 (a)/(b), M1, M1′, M9.
  - V1-3 (MEDIUM) : `AdminRestorePanel.svelte:158` (issue inconnue, maintien sans borne). → option prudente : maintien
    limité à l'import en cours et à la lecture du compte rendu, borné par B = durée d'inactivité réglée (60 min si
    inconnue), levé à l'issue inconnue ; à B après un succès, fin de session comme le chemin 401 (fonction extraite).
    D3, AC4, AC5 bis, M13 à M15 ; C-36-5-2 amendé.
  - V1-2 (MEDIUM) : `session-idle.svelte.ts:67` (`ACTIVITY_WRITE_THROTTLE_MS = 1_000`), `:254`. → D5 : attente de
    1,1 s avant chaque jeu de mouvements ; M8 tuée par la seconde partie (raisonnement écrit en D5, `:367-373`).
  - LOW : V1-4 (succès après démontage, garde `destroyed`, AC5 (b), M6′) ; V1-5 (simulacre de `sessionIdle` et de la
    fonction extraite) ; V1-6 (autre onglet : hors périmètre écrit, issue demandée) ; V1-7 (`docs/testing.md:417` annonce
    « les deux paragraphes suivants », `:419-429` : note placée avant `:431` ; « absorbée par la pollution » dite
    inférence) ; V1-8 (M10 à M12) ; V1-9 (levée en `finally`, indicateur `keepHold`, M5′) ; V1-10 (fréquence estimée sur
    trois passages : la mesure « avant » à 30 passages par test donne la première estimation ; AC7 complété).
  - Gates : `test-fast --ci` et `gate-ci` maintenus au passage en revue, motivés par `textes_coherents.rs:593`.
  - Constat pour l'epic : C-e360-8 demande de « retirer KF-056 des échecs tolérés », mais elle ne figure dans aucune liste
    (`docs/testing.md`, mémoire `e2e-echecs-attendus.md`) : il n'y a pas de ligne à supprimer ; D6 ajoute la note.
  Modules : inchangés (cinq, dont trois de code). Aucune migration.
- 2026-11-14 — **Validation P2 (Opus) : 0/1/0/9 → remédiés** (`kesh-gate-logs/e36-5/validation-p2.md`), par l'agent de
  rédaction (Opus 5.5). Prémisses relues au code :
  - V2-1 (HIGH) : `api-client.ts:148-158` (`authState.login({…}, { broadcast: false })` au succès de `doRefresh`),
    `routes/+layout.svelte:27-28`, `auth.svelte.ts:144-163` (`login()`). Confirmé : la remédiation P1 (renouvellement
    toujours tenté) faisait lever le maintien par `login()`. → option (a), C-36-5-7 : `login()` ne touche plus le
    maintien ; seuls `releaseRestoreHold()`, `logout()` et `clearSession()` le lèvent. Appelants de `authState.login(`
    greppés (sortie copiée en D1) : trois appels réels, dont deux connexions réelles sur des pages où le panneau n'est pas
    monté ; héritage par un autre utilisateur écarté, chemin par chemin, en D1. D1, AC1 bis (neuf), AC6, M7 (redéfinie :
    tuée par AC1 bis), M7′, *Ce qui doit rester vrai* ; levée en `afterEach` des tests qui posent un maintien.
  - V2-2 : gates de remédiation = `test-fast --ci` et frontend complet (`e35-dev-commun.md:12`).
  - V2-3 : `AdminRestorePanel.svelte:157-164` (aucun `catch` englobant, `onclick` sans `await`) : le cas « `toast.error`
    jette » quitte l'AC3, M5′ retirée, la place de la levée dans le `finally` devient un constat de lecture.
  - V2-4 : `AdminRestorePanel.test.ts:9-29` sans `vi.mock` d'`api-client`, `isApiError` employé (`AdminRestorePanel.svelte:20`,
    `:105`) : simulacre partiel écrit tel quel.
  - V2-5 : `api-client.ts:486-492`, `AdminRestorePanel.svelte:130-133` : conduite prudente — l'import refusé en 401 lève le
    maintien et termine la session (`expireSession`), comme aujourd'hui ; AC4 bis, M16.
  - V2-6 : `session-idle.svelte.ts:331-340` : conduite prudente — sous maintien, l'avertissement se calcule sur
    `max(last + w, restoreHoldUntil)` ; il paraît avant l'expiration ; D2, AC6, M12 (redéfinie), M12′. Phase de lecture :
    pas d'avertissement, motivé en D3 (session déjà morte).
  - V2-7 : `admin-manual.tex:2340` : demi-phrase sur le renvoi au bout de la durée d'inactivité réglée (lien
    `sec:duree-inactivite`, `:3170`) ; `.tex` dans la story, PDF à l'intégration ; fichier commun possible avec la 36-4
    écrit aux Dev Notes ; AC8.
  - V2-8 : en-tête de la fiche mis à jour ; *Retenu* contredits du registre (C-36-5-1, C-36-5-2) marqués « remplacé ».
  - V2-9 : sortie du premier grep complétée (`routes/+layout.svelte:66`, commentaire, classé au tableau).
  - V2-10 : « n'excède jamais B » récrit : chaque phase bornée par B, au total la durée de l'import plus B.
