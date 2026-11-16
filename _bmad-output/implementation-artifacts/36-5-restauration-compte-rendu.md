# Story 36-5 : le compte rendu d'une restauration ne se perd plus sous un battement de session refusé (#931, KF-056)

Status: done

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-5`, branche
`story/36-5-restauration-compte-rendu`, par le workflow `bmad-create-story`. Release **0.17.8**, phase 1. **Aucune
migration**, aucun changement serveur ni d'API, aucune clé i18n. Code lu à la tête du worktree, **`4de0bcc7e`**. Étape 0 et
choix qui font foi : `e36-choix-autonomes-e360.md` (§ *Étape 0 — 36-5*, C-e360-8, C-e360-11) ; choix propres à la story :
`e36-choix-autonomes-36-5.md` (C-36-5-1 à C-36-5-13, les quatre derniers au développement). Voie de base **1** (lignée
B, après la 36-4). Validation terminée en six passes (Change Log) ; développée le 2026-11-15, en revue.*

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

**Élargi à la validation P4 (V4-10)** : le symptôme est « la page **quitte** l'écran de restauration », vers `/login` ou
ailleurs. Détecteur sans filtre de destination :

```sh
$ git grep -nE "location\.(replace|assign|reload)|location\.href *=|goto\(|invalidateAll\(" -- frontend/src ':!*.test.ts' ':!*.spec.ts' | wc -l
63
```

Les 63 lignes ; celles qui sont rendues **avec** le panneau — modules partagés et layouts — sont (les autres sont des
pages de l'application ou de l'assistant, ou `/login`, `/setup`, la page d'erreur, jamais rendues avec lui ; filtre
`/usr/bin/grep -vE "routes/\(app\)/[a-z-]+/|components/invoices|mcp-prompts|features/setup|routes/login|routes/onboarding|\+error"`) :

```
frontend/src/lib/app/stores/session-idle.svelte.ts:407:	window.location.replace(`/login?reason=idle&minutes=${n ?? ''}`);
frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte:184:		window.location.replace('/login');
frontend/src/lib/shared/components/DemoBanner.svelte:28:			goto('/onboarding');
frontend/src/lib/shared/utils/api-client.ts:202:		window.location.replace('/login?reason=session_expired');
frontend/src/lib/shared/utils/api-client.ts:534:		window.location.replace('/setup');
frontend/src/lib/shared/utils/notify.ts:118:				void goto('/settings/fiscal-years');
frontend/src/routes/(app)/+layout.svelte:41:			import('$app/navigation').then(({ goto }) => goto('/journal-entries'));
frontend/src/routes/(app)/+layout.svelte:355:							window.location.replace('/login');
frontend/src/routes/+layout.svelte:66:		// token) → `clearSession()` + `window.location.replace('/login?reason
```

| Site neuf au tableau | Atteint sans geste après l'import ? | Sort |
|---|---|---|
| `api-client.ts:534` (`handle423Redirect`, 423 → `/setup`) | **non** : `users_exist` n'est remis à jour que par `/setup` (`git grep -n "users_exist" -- crates/kesh-api/src/routes` : `setup.rs`, `test_endpoints.rs`, aucun dans `admin.rs`) | hors du symptôme |
| `DemoBanner.svelte:28` | geste (`handleReset`, *Réinitialiser la démonstration*) | hors du symptôme |
| `notify.ts:118` | geste (`onClick` de l'action d'un toast) | hors du symptôme |
| `(app)/+layout.svelte:41` | geste (Ctrl+N en mode expert) | hors du symptôme |
Les autres minuteries de l'application (`git grep -nE "setInterval\(" -- 'frontend/src/**/*.ts' 'frontend/src/**/*.svelte'`,
tests exclus) : `session-idle.svelte.ts:454` (le battement), `BackupScheduleBanner.svelte:34` (ci-dessus),
`api-health.svelte.ts:102` (`pollHealth`, sonde de `/health`, sans session : ne mène pas à `refreshTokens`). **Angle mort**
écrit : un `fetch` direct qui redirigerait de lui-même échapperait au premier grep s'il n'écrivait pas `/login` en clair ;
le second grep couvre `goto` et `redirect`. Aucun autre site.

## Décisions

- **D1 — Un maintien de session pendant la restauration, porté par `authState`, lu par le chemin 401 du client**
  (C-36-5-1, amendé à la validation P1, V1-1 et V1-3). `auth.svelte.ts` gagne un état non réactif — l'**échéance** du
  maintien, `restoreHoldUntil` (ms, `null` sans maintien) — et deux méthodes, `holdForRestore(durationMs)` (pose ou
  réarme : `restoreHoldUntil = Date.now() + durationMs`, **remplace** toute échéance précédente, plus longue ou plus
  courte) et `releaseRestoreHold()`, avec **un accesseur public**, `get restoreHold(): boolean`, qui vaut exactement
  `restoreHoldUntil !== null && Date.now() < restoreHoldUntil` ;
  `restoreHoldUntil` reste privé au module. **Troisième méthode, ajoutée à la revue de code P1 (E1-2, C-36-5-14)** :
  `shortenRestoreHold(durationMs)` ramène l'échéance à au plus `maintenant + durationMs`, ne l'allonge jamais, ne pose
  rien sans maintien et ne lève pas ; le panneau l'appelle après un échec certain ou une issue inconnue. Le maintien
  expire seul (D3 fixe les durées). *(L'accesseur
  `restoreHoldDeadline` ajouté à la P3 pour D2 est **retiré** à la P4 : D2 simplifié ne lit plus l'échéance, C-36-5-9 ; les
  tests constatent les échéances par `restoreHold` à `échéance − 1` et à `échéance` ms.)*
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
  le panneau n'est pas monté ; les quatre autres lignes sont des commentaires. **Aucune connexion faite dans cet onglet
  n'hérite d'un maintien** (une connexion faite dans un **autre** onglet : Dev Notes, *Hors périmètre*, V4-8) : le maintien est un état de module, propre à l'onglet ; il est levé par `logout()` (le *Se
  reconnecter* du panneau, `:180`, et le menu *Déconnexion*) — **en première instruction, avant le `await fetch`**
  (`auth.svelte.ts:204-212` : `await fetch('/api/v1/auth/logout', …).catch(() => {});`, sans délai d'abandon : une
  requête de déconnexion pendue laisserait sinon le maintien posé ; la levée avant le `fetch` ne coûte rien, la suite de
  `logout()` menant de toute façon à `/login` ; décision de l'orchestrateur sur V3-7) —, par `clearSession()` (chemin 401 hors maintien, fermeture
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
- **D2 — Sous maintien, rien ne change avant l'échéance d'inactivité ; après elle, rien ne se ferme ni ne s'écrit
  localement** (simplifié à la validation P4, V4-1 et V4-2, décision de l'orchestrateur, C-36-5-9). **Le constat qui
  fonde D2** : le serveur ferme la session à `last + w` quoi qu'il arrive — le middleware refuse toute requête au-delà
  d'`idle_exp` (`middleware/auth.rs:156-161`, « au-delà, aucune requête n'est acceptée, quel que soit `exp` »), et
  l'échéance est écrite dans `refresh_tokens.expires_at` (`routes/auth.rs:25-33`), si bien que le renouvellement n'y
  survit pas non plus. Le maintien ne prolonge donc rien : il **diffère seulement le renvoi local à la connexion** —
  la redirection du chemin 401 (D1) et la fermeture d'inactivité. Dans `session-idle.svelte.ts`, avec `restoreHold` lu
  sur `authState` (seul membre du maintien que le module lit) :
  - **tant que `last + w − now > 0`** : le module se comporte **exactement comme aujourd'hui** — le battement part
    (`:345-348`), l'avertissement paraît une minute avant l'échéance (`:338-340`, formule inchangée), *Rester connecté*
    (`stayActive`, `:466-478`) écrit l'activité et l'efface ; un battement parti à temps prolonge la vraie session ;
  - **une fois `last + w` atteint, sous maintien** : `tick` ne ferme pas (`:331-333`) et met `warningRemainingMs` à
    `null` (la session est close au serveur, il n'y a rien à prolonger) ; `recordActivity` (`:247-258`) et `stayActive`
    **ne ferment pas et n'écrivent pas** l'activité (« une activité au-delà ne ranime pas la session », `:249`) ;
  - **à la levée ou à l'échéance du maintien**, le tic suivant ferme si l'échéance est passée, par le chemin
    d'aujourd'hui (`closeSession`, `/login?reason=idle&minutes=N`) : la fermeture est différée, jamais annulée.
  Hors maintien : rien ne change. Le battement **continue** pendant l'import, et son 401 éventuel, après l'invalidation
  ou après `last + w`, est absorbé par D1. Borne (D3, formulation unique) : « Le maintien est posé à la confirmation pour
  `importHoldMs(B)` ; au succès, son échéance est remplacée par `maintenant + B` ; après un échec certain ou une issue
  inconnue, son échéance est ramenée à au plus `maintenant + B`, jamais allongée, et le maintien est levé au premier
  geste, au démontage ou à cette échéance ; au 401 de l'import, il est levé aussitôt. Il ne dure donc jamais au-delà de `confirmation +
  max(importHoldMs(B), durée de l'import + B)`. » Au serveur, rien ne s'étend. *Écartées* : (a) suspendre aussi le battement pendant l'import — une
  restauration longue qui échoue trouverait la session close au serveur (C-36-5-2) ; (b) — rédaction de la P2 —
  calculer l'avertissement sur `max(last + w, échéance du maintien)` : il supprimait l'avertissement utile (avant la vraie
  fermeture, au serveur) et en montrait un inopérant, après elle (V4-1) ; (c) — rédaction de la P1 — éteindre
  l'avertissement dès la pose : *Rester connecté* n'était plus offert avant `last + w`.
- **D3 — Le panneau pose, borne et lève le maintien** (C-36-5-2, amendé à la validation P1, V1-3, V1-4, V1-9). Le
  maintien ne couvre que **deux phases** : l'import en cours (phase 1), et la lecture du compte rendu d'un import réussi
  (phase 2). **B** = la durée d'inactivité réglée — lue à la confirmation pour la phase 1, **relue au succès et à l'échec**
  (revue de code P1, E1-4, C-36-5-15) — (`sessionIdle.minutes`, `session-idle.svelte.ts:499-501`, en minutes ;
  60 si elle est inconnue, la valeur par défaut du serveur, manuel d'administration `:2651`).
  - **Borne de la phase 1 : `importHoldMs(B)`**, la fonction unique décrite plus bas (amendé aux validations
    P3, V3-1, décision de l'orchestrateur, C-36-5-8, et P4, V4-4). Lu au code : `api-client.ts:53`,
    `export const LONG_OPERATION_TIMEOUT_MS = 60 * 60_000;`, la durée que le client accorde à **chaque envoi** de l'import
    (`admin-restore.api.ts:46-48` ; le délai part à chaque `fetchWithTimeout`, `:319`, `setTimeout(() => controller.abort(), timeoutMs)`) ;
    un 401 renouvelé fait **un** rejeu, et un seul (`:476-485`, garde `!isRetry`), qui repart avec un délai neuf. L'import
    tient donc en au plus deux envois et un renouvellement : deux délais, plus une **marge** `RESTORE_HOLD_MARGIN_MS =
    60_000` (une minute, pour le renouvellement et la relecture de `/me` entre les deux envois ; fichier et hypothèse
    ci-dessous). Au-delà, l'import a fini en `TIMEOUT` (issue inconnue) avant l'échéance du maintien. Et B vaut de 1 à 1 440 minutes (`session-idle.svelte.ts:76-77`,
    `MIN_MINUTES = 1`, `MAX_MINUTES = 1_440`). Bornée par B seul (rédaction de la P1), la phase 1 expirait pendant un import
    plus long que B — dès une minute si l'Administrateur a réglé N = 1 — et le battement suivant rouvrait la KF-056. La
    constante est **importée** d'`api-client.ts` par le panneau, **jamais recopiée** (`git grep -n "60 \* 60_000" --
    frontend/src ':!*.test.ts'` → la seule ligne `:53`).
  - **Borne de la phase 2 : B.** *Pourquoi B* : c'est le temps qu'un écran inactif peut déjà rester ouvert sans fermeture,
    selon le réglage de l'Administrateur ; lire deux lignes de compte rendu en prend quelques secondes.
  - **Borne du maintien — formulation unique** (validation P5, V5-2 ; reprise telle quelle en D2 et dans *Ce qui doit
    rester vrai*) : « Le maintien est posé à la confirmation pour `importHoldMs(B)` ; au succès, son échéance est remplacée
    par `maintenant + B` ; après un échec certain ou une issue inconnue, son échéance est ramenée à au plus `maintenant +
    B`, jamais allongée, et le maintien est levé au premier geste, au démontage ou à cette échéance ; au 401 de l'import,
    il est levé aussitôt. Il ne dure donc jamais au-delà de `confirmation + max(importHoldMs(B), durée de l'import + B)`. »
    *(Récrite à la revue de code P1, E1-2 : « l'échéance de la confirmation est gardée » laissait un écran sans geste
    ouvert jusqu'à `confirmation + importHoldMs(B)`, au moins 121 min, après un échec rendu en quelques secondes. La borne
    finale est inchangée : `min(confirmation + importHoldMs(B), échec + B)` ne la dépasse pas.)*
  - **Où vivent la marge et la formule** (V5-3) : dans `frontend/src/lib/features/admin-restore/admin-restore.api.ts`, à
    côté d'`uploadFullImport` (qui emploie déjà `LONG_OPERATION_TIMEOUT_MS`, `:46-48`), deux exports :
    `export const RESTORE_HOLD_MARGIN_MS = 60_000;` et une **fonction unique**
    `export function importHoldMs(readingMs: number): number { return Math.max(readingMs, 2 * LONG_OPERATION_TIMEOUT_MS + RESTORE_HOLD_MARGIN_MS); }`
    (noms indicatifs) — l'expression du maximum n'est écrite **que là** ; le panneau et les tests l'appellent. B lui-même
    se calcule en un seul point du panneau (`(sessionIdle.minutes ?? 60) * 60_000`).
  - **Ce que la marge suppose** (V5-3, dit tel quel) : `doRefresh` (`api-client.ts:92`, `fetch('/api/v1/auth/refresh', …)`)
    et la relecture de `/me` (`:133`) n'ont **aucun délai d'abandon**. La marge d'une minute couvre un renouvellement et
    une relecture qui aboutissent en moins d'une minute ; c'est une hypothèse, non une borne. Au-delà, rien ne se perd
    **tant que le rejeu n'est pas parti** (l'import n'a pas encore atteint le serveur, aucune invalidation n'a eu lieu) ;
    la fenêtre exposée n'existe que si, après un renouvellement de plus d'une minute, le rejeu dure lui-même presque toute
    l'heure. Elle est acceptée, non étendue : la borner exigerait de réarmer le maintien depuis `api-client.ts`, pour un
    cas pathologique.
  - **confirmation** (`confirmImport`, `AdminRestorePanel.svelte:143`) : `authState.holdForRestore(importHoldMs(B))`
    **avant** l'appel à `uploadFullImport` (après `importing = true`, avant tout `await`) ;
  - **succès** : `authState.holdForRestore(B)` — le maintien est **réarmé** pour la lecture, l'heure de la phase 1 oubliée ; `sessionIdle.stop()` — la
    session n'existe plus, il n'y a rien à prolonger, et plus aucun battement ne part ; le panneau arme une minuterie de B
    qui, à l'échéance, termine la session comme le chemin 401 ordinaire : levée du maintien, `authState.clearSession()`,
    `window.location.replace('/login?reason=session_expired')`. Ces deux dernières instructions sont celles de
    `refreshTokens` (`api-client.ts:201-202`) : elles s'extraient dans une fonction exportée d'`api-client.ts`
    (`expireSession()`, nom indicatif), appelée par les deux sites (DRY). La minuterie est effacée au démontage.
    **Pas de double `expireSession()`** (V5-7, montré au code, sans garde ajoutée) : quand la minuterie s'exécute,
    `clearSession()` ne démonte pas le panneau — les layouts rendent la page sans condition (`(app)/+layout.svelte:453`,
    `{@render children()}` dans `<main>`, hors de tout bloc `{#if}` ; `onboarding/+layout.svelte:33` ; `routes/+layout.svelte:118`),
    et `location.replace` décharge la page sans lancer le nettoyage des composants ; inversement, un démontage d'abord
    efface la minuterie (AC5 (c), M22). Les deux appels ne peuvent donc se suivre ; et deux effacements resteraient sans
    gravité (`api-client.ts:195-196`) ;
  - **issue inconnue** (`unknownOutcome` vrai, `:158`) **et échec certain** (toute autre erreur que le 401 de l'import) :
    le message d'échec s'affiche ; **le maintien est levé au premier geste de l'utilisateur qui suit** — le premier
    `pointerdown` ou `keydown` sur la fenêtre (un écouteur posé une fois, en capture, par le panneau, retiré après usage,
    au démontage et au départ d'un nouvel import — revue P1, E1-1) —, ou au démontage, ou au plus tard à l'échéance,
    ramenée à au plus `maintenant + B` au moment de l'échec, jamais allongée (revue P1, E1-2 ; `shortenRestoreHold`)
    (validation P4, V4-5, forme la plus simple retenue : un seul geste, quel qu'il soit, vaut lecture du message). Tant qu'il dure, D2 fait que rien ne change pour
    une session vivante (battement, avertissement, *Rester connecté*), et qu'une session déjà échue ou invalidée ne renvoie
    pas à la connexion sous les yeux de l'utilisateur : le message reste lisible. Au geste, la levée rend le régime
    ordinaire — si la session est échue, le tic suivant ferme (`/login?reason=idle`) ; si l'import a abouti, le 401 suivant
    renvoie à la connexion, ce que le message demande déjà (« Reconnectez-vous pour vérifier l'état de l'installation »,
    `:100-103`) ; *Se reconnecter* est lui-même un geste. Le manuel (`:2342`, l'écran « affiche » le message et « propose
    Se reconnecter ») devient juste dans tous les cas ;
  - **l'import lui-même refusé en 401** (V2-5, conduite prudente) : sous maintien, un renouvellement échoué fait jeter
    `UNAUTHENTICATED` (`api-client.ts:487-492`) sans renvoi ; le `catch` du panneau le reconnaît (`isApiError(err) &&
    err.code === 'UNAUTHENTICATED'`) et, la session étant morte **avant** tout import, termine la session comme le chemin
    401 ordinaire : levée du maintien puis `expireSession()` (`/login?reason=session_expired`), au lieu d'afficher « Session
    expirée » (`:130-133`, `default: return err.message;`) avec un bouton d'import réactivé. C'est le comportement
    d'aujourd'hui pour ce cas ;
  - **pas d'avertissement avant la fin de la lecture** (phase 2) : la session est déjà morte au serveur ; *Rester connecté*
    n'aurait rien à prolonger, et il n'y a rien à enregistrer. Le manuel dit ce renvoi (D6) ;
  - **la levée différée se pose en `finally`** (V1-9, amendé à la P4) : le `finally` (`:162-164`) décide selon un
    indicateur local `keepHold`, posé à vrai **seulement** au succès : `if (!keepHold) armReleaseOnNextGesture();` (nom
    indicatif ; le 401 de l'import a déjà levé et quitté la page). Une exception levée dans le `catch` (`issueInconnue`,
    `importErrorMessage`, `toast.error`, `:158-161`) ne laisse donc jamais le maintien sans levée prévue ;
  - **le panneau quitte l'écran** (navigation dans l'application, lien *Retour à l'assistant*) : levée et effacement de la
    minuterie dans le nettoyage du composant (`$effect(() => () => …)` ou `onDestroy`), et un indicateur `destroyed` posé.
    **En phase 1** (import en vol) ou sans import : la requête suivante qui rend 401 renvoie à la connexion par le chemin
    ordinaire. **En phase 2** (succès constaté, session connue morte, module de session arrêté), conduite prudente
    (validation P3, V3-4) : le nettoyage appelle **`expireSession()`** après la levée — sinon le module resterait arrêté
    (l'effet du layout racine ne le relance que si `sessionUserId` change, `routes/+layout.svelte:33-42`) et l'application
    paraîtrait connectée jusqu'à la première requête. Exception : si `reconnecting` est vrai (*Se reconnecter* en cours,
    `:177-178`), le nettoyage n'appelle rien — `relogin` mène déjà à `/login`. *Se reconnecter* n'a rien à lever :
    `logout()` remet à zéro (D1), et la navigation recharge tout ;
  - **import encore en vol au démontage** (V1-4) : `uploadFullImport` n'a pas de signal d'annulation (`:156`), et
    l'import continue au serveur. Si sa réponse arrive après le démontage (`destroyed` vrai), le panneau **ne réarme pas**
    le maintien, **n'appelle pas** `sessionIdle.stop()` et **n'arme pas** la minuterie : l'utilisateur a quitté l'écran, il
    retrouve le régime ordinaire, y compris la redirection par le battement suivant si l'import a invalidé la session.
    Limite écrite, assumée : celui qui quitte l'écran pendant l'import ne voit pas le compte rendu (il ne l'aurait pas vu
    non plus aujourd'hui).
  *Écartées* : (a) lever le maintien dès la réponse reçue — c'est précisément après elle que le bandeau ou le battement
  renverraient à la connexion ; (a′) — rédaction des P1 à P3 — lever aussitôt à l'échec certain et à l'issue inconnue :
  une session échue pendant l'attente fermait au tic suivant et emportait le message (V4-5) ; (a″) attendre un clic sur
  un bouton du panneau seulement : une règle par bouton, plus de code pour le même effet ; (b) — première rédaction —
  garder le maintien sans borne après une issue inconnue :
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
  sauvegarde}. », ajouter (amendé à la P4, V4-9 : quitter l'écran du compte rendu mène aussi à la connexion, D3) :
  « Laissé ouvert, ou quitté, l'écran mène à la connexion --- au bout de la durée d'inactivité réglée
  (\S\ref{sec:duree-inactivite}) dans le premier cas, aussitôt dans le second. » Le `.tex` change dans la story ; **le PDF se régénère à l'intégration**, après l'union
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
   l'état est au module, et `login()` en tête du test suivant ne le nettoie plus. Précision (validation P3, V3-5) : dans
   `api-client.test.ts`, le `beforeEach` appelle déjà `authState.clearSession()` (`:80`), qui lève le maintien (D1) ; une
   fuite y serait donc masquée, et l'`afterEach` y est une ceinture de plus, non la preuve. La levée par `clearSession()`
   et par `logout()` se prouve dans `auth.svelte.test.ts` seul (AC6, M7′a, M7′b), jamais par des effets de bord
   d'`api-client.test.ts`.
2. **AC2 — Après un import réussi, le compte rendu reste** (E2E de D5, seconde partie ; `AdminRestorePanel.test.ts`) : aucun
   `POST /api/v1/auth/activity` ne part plus (compteur à zéro après 3 s d'activité), l'URL ne change pas ; au Vitest, le
   maintien est posé **avant** l'appel à `uploadFullImport` (ordre des appels des deux simulacres), réarmé au succès avec
   la durée B, `sessionIdle.stop` est appelé, et **`releaseRestoreHold` n'est pas appelé** après le succès (V4-7).
3. **AC3 — Échec certain ou issue inconnue : le message reste, puis tout reprend au premier geste** (`AdminRestorePanel.test.ts`,
   `session-idle.svelte.test.ts`) :
   (a) au panneau, pour une `ApiError` 400 (`INVALID_BACKUP_STRUCTURE`) puis, dans un second test, pour une erreur
   `TIMEOUT` ou un 504 (`isUnknownOutcome`) : `releaseRestoreHold` **n'est pas appelé** tant qu'aucun geste n'a lieu ; le
   message (`admin-restore-error`) est rendu ; à l'issue inconnue, *Se reconnecter* est proposé et le bouton d'import
   désactivé, comme aujourd'hui (`backup-restore-new-install.spec.ts:336-341`), et `sessionIdle.stop` n'est **pas** appelé ;
   un `keydown` (puis, dans un autre test, un `pointerdown`) sur la fenêtre appelle `releaseRestoreHold` **une fois** ; un
   second geste ne l'appelle plus (écouteur retiré) ; démonter sans geste l'appelle aussi. **Geste pendant l'import**
   (V5-1) : `uploadFullImport` pendante, `fireEvent.pointerDown(window)` puis `fireEvent.keyDown(window)` →
   `releaseRestoreHold` **non appelé** ; la promesse rejetée ensuite (400), le message rendu, un geste appelle
   `releaseRestoreHold` une fois — l'écouteur n'est armé qu'après l'affichage du message d'échec ou d'issue inconnue. La levée différée se pose dans
   le `finally` : constat de lecture à la revue, ligne citée — le cas « `toast.error` jette » n'est pas joué en Vitest
   (rejet non géré d'un `onclick` sans `await`, compté comme erreur avant comme après le correctif, V2-3) ;
   **ajouté à la revue de code P1** : (E1-1) après un échec certain, un nouvel import activé par `click` seul retire
   l'écouteur — un `keydown` ou un `pointerdown` pendant l'import, puis pendant la lecture du compte rendu, n'appelle
   pas `releaseRestoreHold` et `restoreHold` reste vrai ; (E1-2) B = 1 min, import de 90 s puis échec 400 :
   `shortenRestoreHold(60 000)` appelé, `restoreHold` vrai à `échec + 59 999`, faux à `échec + 60 000`, sans levée ;
   au store, `shortenRestoreHold` ramène l'échéance à au plus `maintenant + durée`, ne l'allonge jamais et ne pose rien
   sans maintien ;
   (b) au module de session (`authState.restoreHold` du simulacre réglé à vrai) : **avant** `last + w`, tout est
   d'aujourd'hui — le battement part, l'avertissement paraît à `last + w − 60 000` et vaut le temps restant, `stayActive()`
   l'efface et écrit `lastActivity` ; **à partir de** `last + w` : le `tick` ne ferme pas et `warningRemainingMs` vaut
   `null`, un `keydown` (`recordActivity`) et `stayActive()` ne ferment pas **et n'écrivent pas** `lastActivity` (valeur
   relue inchangée) ; puis, `restoreHold` passé à faux, le tic suivant ferme (`/login?reason=idle&minutes=N`,
   déconnexion au serveur d'abord, comme `:150-170` du test existant).
4. **AC4 — L'import refusé en 401 renvoie à la connexion** (V2-5 ; numéroté « AC4 bis » jusqu'à la P4, l'ancien AC4
   — issue inconnue — étant versé à l'AC3 ; `AdminRestorePanel.test.ts`) : `uploadFullImport`
   rejette `{ code: 'UNAUTHENTICATED', status: 401, message: 'Session expirée' }` → `releaseRestoreHold` puis `expireSession`
   appelés, dans cet ordre ; aucun `admin-restore-error` rendu.
5. **AC5 — Quitter le panneau lève le maintien** (`AdminRestorePanel.test.ts`) : (a) démonter le composant **sans import
   ou pendant l'import** : `releaseRestoreHold` appelé, `expireSession` non appelé ; une requête 401 qui suit (Vitest du
   client, maintien levé) renvoie à `/login?reason=session_expired` (corrigé à la P4, V4-3 : la rédaction de la P3 parlait
   d'un démontage « après un succès » sans redirection, ce que (c) contredit) ; (b) démonter **pendant**
   l'import (promesse de `uploadFullImport` pendante), puis la résoudre en succès : aucune exception, `holdForRestore`
   appelé une seule fois (à la confirmation), `sessionIdle.stop` jamais appelé, aucune minuterie armée, `expireSession`
   jamais appelé ; (c) démonter **en phase 2** (après un succès, `reconnecting` faux) : `releaseRestoreHold` puis
   `expireSession` appelés, dans cet ordre (V3-4) ; la minuterie de B est effacée : avancer ensuite de B, `expireSession`
   n'a été appelé **qu'une fois**, par le nettoyage ; (d) cliquer *Se reconnecter* puis démonter : `expireSession` n'est
   **pas** appelé.
5 bis. **AC5 bis — Le maintien est borné** (`auth.svelte.test.ts`, `api-client.test.ts`, `AdminRestorePanel.test.ts`,
   minuteries simulées) : (a) `holdForRestore(B)` : `restoreHold` vrai à `pose + B − 1` ms, faux à `pose + B` ; un 401
   dont le renouvellement échoue, à `pose + B`, efface et redirige comme hors maintien ; une seconde pose plus courte
   **remplace** la première (`holdForRestore(3 600 000)` puis `holdForRestore(60 000)` : vrai à `+59 999`, faux à
   `+60 000`) ; (b) au panneau, après un succès, la minuterie de B appelle `releaseRestoreHold` puis `expireSession` (le
   simulacre, V4-6), et rien avant B ; le contenu d'`expireSession` — `clearSession` puis
   `location.replace('/login?reason=session_expired')` — se prouve par un test propre dans `api-client.test.ts` ;
   (c) **les deux durées** : avec `sessionIdle.minutes = 5`, la confirmation pose
   `holdForRestore(importHoldMs(300 000))`, soit 7 260 000, et le
   succès `holdForRestore(300 000)` ; avec `minutes = null`, la confirmation pose 7 260 000 et le succès 3 600 000 ; les
   attentes se calculent à partir des constantes importées, jamais recopiées ; (d) **import plus long que B** :
   `sessionIdle.minutes = 1` (B = 60 000), vrai `authState` dans ce test, promesse d'`uploadFullImport` pendante,
   minuteries avancées de 61 000 ms puis de 3 600 000 ms : le maintien est toujours posé ; la promesse résolue en succès,
   le compte rendu (`admin-restore-success`, `admin-restore-files-restored`) s'affiche, et `restoreHold` vaut vrai à
   `+59 999` et faux à `+60 000` après le succès — la borne de la phase 1 n'a pas survécu à la fin de l'import.
6. **AC6 — La fermeture ordinaire est intacte (#876, #895)** : les tests existants de `session-idle.svelte.test.ts` et les
   tests 401 de `api-client.test.ts` (`:246-267`, `:327-343`, `:488`, `:567-581`) passent **sans modification de leurs
   corps ni de leurs assertions** — seuls les simulacres sont étendus (`session-idle.svelte.test.ts:17-19`,
   `AdminRestorePanel.test.ts:9-29`, § *Simulacres*), ce que le panneau exige (V5-5) ; hors
   maintien, le module de session se comporte comme aujourd'hui (couvert par ses tests existants) ; à
   `auth.svelte.test.ts`, `clearSession()` remet le maintien à zéro (`restoreHold` faux) ; `logout()` aussi, **dès avant
   la fin de sa requête** (`fetch` simulé par une promesse pendante : `restoreHold` déjà faux ; puis rejet de cette
   promesse : toujours faux, aucune exception) ; **`login()` ne le touche pas** (maintien posé, `login()`, `restoreHold`
   toujours vrai).
7. **AC7 — Preuve par répétition** (D4) : `repetition-avant.log` (60 exécutions, 30 par test, rouges comptés et nommés
   avec la fréquence observée de chaque test, ou leur absence dite) et `repetition-apres.log` (60 + 10 exécutions, zéro
   rouge), chacun avec son commit, dans `kesh-gate-logs/e36-5/`. Sans rouge avant, le Dev Agent Record écrit que la
   répétition « après » ne prouve rien par elle-même et que la preuve est celle du test de D5 (rouge avant, vert après).
8. **AC8 — Documentation** (D6) : la note KF-056 dans `docs/testing.md` ; `git grep -n "KF-056" -- docs/testing.md` la
   rend ; la phrase de D6 au `.tex` (`git grep -n "Laissé ouvert, ou quitté" -- docs/manual/fr/admin-manual.tex` → une
   ligne) ; le contrôle du PDF (« Prenez le temps de lire, puis cliquez ») rend toujours `1` ; le PDF régénéré à
   l'intégration rend aussi `1` pour « Laissé ouvert, ou quitté, l'écran mène à la connexion ».
9. **AC9 — Les deux chemins** : aucune ligne propre à une route n'est ajoutée ; le panneau est le seul à poser le
   maintien : les **sites d'appel**, lus à la main dans la sortie de `git grep -nE "holdForRestore\(" -- frontend/src
   ':!*.test.ts'`, sont la définition (`auth.svelte.ts`) et les deux appels du panneau (confirmation, succès) ; les lignes
   de commentaire qui nomment la méthode (en-têtes complétés par la story) sont écartées en les citant (validation P3,
   V3-3) ;
   `:140` (assistant) et `:262` (`/admin/restore`) font partie des deux répétitions.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, journal `kesh-gate-logs/e36-5/rouges.log`. Pour que les rouges portent sur le **comportement** et non sur
des exports absents, l'API du maintien (D1) est d'abord commitée **inerte** (`holdForRestore`/`releaseRestoreHold` sans
effet, `restoreHold` toujours `false` ; `expireSession` extraite d'emblée, sans changement de comportement) — patron de
la 35-7, AC2. Rouges au départ : AC1 (a) (Vitest du client : la redirection a lieu ; E2E : la page part à
`/login?reason=session_expired`) ; AC2 (Vitest du panneau : maintien jamais posé, `stop` jamais appelé ; E2E) ; AC3 (a)
(levée au geste jamais armée) et (b) pour la partie « à partir de `last + w` » (le module ferme) — sa partie « avant
`last + w` » est **verte au départ** (comportement d'aujourd'hui) et se dit telle ; ce que l'AC3 (a) garde d'aujourd'hui
(*Se reconnecter*, bouton désactivé) est vert, et se dit tel ; AC4 (le message « Session expirée » est rendu) ; AC5 ;
AC5 bis. AC1 (b) est **vert au départ** (le rejeu existe déjà) et se dit tel : il garde le rejeu contre M1′. AC1 bis :
rouge au départ (maintien inerte, la seconde requête redirige). AC6 :
« `login()` ne touche pas le maintien » est rouge sur l'API inerte (`restoreHold` toujours faux) ; « `logout()`/
`clearSession()` le remettent à zéro » est **vert au départ par construction, et se dit tel** — tué par M7′a et M7′b. Verts au
départ, et dits : les tests existants de l'AC6.

**Simulacres** (V1-5) : ceux d'`authState` de `AdminRestorePanel.test.ts:24-29` et de `session-idle.svelte.test.ts:17-19`
gagnent `holdForRestore`, `releaseRestoreHold` et `restoreHold` (celui du panneau aussi `shortenRestoreHold`, depuis la
revue de code P1 ; celui de `session-idle` n'en a pas besoin, le module ne l'appelle pas) — dans `session-idle.svelte.test.ts`, un accesseur que
le test règle (`get restoreHold() { return hold; }` sur une variable du fichier), seul membre du maintien que lit D2 ;
`restoreHoldDeadline` n'existe plus (V4-1) ; le simulacre du panneau n'a pas besoin de `clearSession` (V4-6) ;
`AdminRestorePanel.test.ts`
gagne un `vi.mock('$lib/app/stores/session-idle.svelte', () => ({ sessionIdle: { stop: vi.fn(), get minutes() { … } } }))`
— le module réel importerait `api-client` et poserait des minuteries — et un simulacre **partiel** d'`api-client`, qui
n'a aujourd'hui aucun `vi.mock` dans ce fichier (`AdminRestorePanel.test.ts:9-29` : `./admin-restore.api`, `i18n.svelte`,
`svelte-sonner`, `auth.svelte`) et dont le panneau emploie `isApiError` (`AdminRestorePanel.svelte:20`, `:105`) — un
simulacre entier ferait rougir les tests de messages d'erreur existants (V2-4) ; à écrire tel quel :
`vi.mock('$lib/shared/utils/api-client', async (orig) => ({ ...(await orig<typeof import('$lib/shared/utils/api-client')>()), expireSession: vi.fn() }));`
— `LONG_OPERATION_TIMEOUT_MS` y reste la vraie constante, que les tests importent pour leurs attentes au lieu de la
recopier ; `RESTORE_HOLD_MARGIN_MS` et `importHoldMs` s'importent d'`admin-restore.api.ts` (dont le `vi.mock` existant
devient partiel de la même façon, `uploadFullImport` restant un `vi.fn()`, faute de quoi le panneau importerait un
`importHoldMs` indéfini). Les tests qui touchent l'écouteur de geste (AC3 (a)) emploient
`fireEvent.keyDown(window, …)` / `fireEvent.pointerDown(window)`. Les tests de B et de la minuterie emploient `vi.useFakeTimers()`. Les tests de `logout()` simulent `fetch`
(`auth.svelte.ts:207`, `await fetch('/api/v1/auth/logout', …)`) par une promesse que le test résout ou rejette.

**Mutations** (code commité d'abord ; script et journaux dans `kesh-gate-logs/e36-5/mut/` ; restauration par
`git restore --worktree -- <fichier>` puis `touch`) :

| | Mutation | Tuée par |
|---|---|---|
| M1 | la garde du maintien retirée de `refreshTokens` (`clearSession` + redirection même sous maintien) | AC1 (a) (Vitest du client, E2E de D5) |
| M1′ | sous maintien, le renouvellement n'est plus tenté (rejet direct en `UNAUTHENTICATED`) | AC1 (b) (rejeu attendu, et `refresh` appelé en AC1 (a)) |
| M2 | `holdForRestore()` retiré de la confirmation | AC2 (Vitest du panneau), E2E de D5 |
| M3 | `holdForRestore()` déplacé après `await uploadFullImport(…)` | AC2 (ordre des appels) |
| M4 | la fermeture du `tick` non conditionnée au maintien | AC3 (b) (module de session, après `last + w`) |
| M5 | la levée au premier geste non armée dans le `finally` (échec certain, issue inconnue) | AC3 (a) (geste : `releaseRestoreHold` attendu) |
| M5″ | la levée faite aussitôt à l'échec certain ou à l'issue inconnue (rédaction des P1 à P3) | AC3 (a) (aucune levée avant le geste) |
| M5‴ | l'écouteur de geste non retiré après usage | AC3 (a) (second geste : pas de second appel) |
| ~~M5′~~ | *retirée à la validation P2 (V2-3)* : sans test qui la discrimine (rejet non géré, AC3) ; la place de la levée dans le `finally` est un constat de lecture de la revue | — |
| M6 | la levée au démontage retirée | AC5 (a) |
| M6′ | la garde `destroyed` retirée (succès après démontage) | AC5 (b) |
| M7 | `login()` remet le maintien à zéro (la forme de la première rédaction, V2-1) | AC1 bis (`api-client.test.ts` : après le renouvellement réussi, la seconde requête redirige) ; AC6 (`auth.svelte.test.ts`) |
| M7′a | `logout()` ne remet plus le maintien à zéro | AC6 (`auth.svelte.test.ts`, test de `logout()`, `fetch` simulé) |
| M7′b | `clearSession()` ne remet plus le maintien à zéro | AC6 (`auth.svelte.test.ts`, test de `clearSession()`) |
| M7″ | dans `logout()`, la levée déplacée **après** le `await fetch` | AC6 (`fetch` pendant : `restoreHold` encore vrai) |
| M8 | `sessionIdle.stop()` retiré du succès | AC2 (Vitest du panneau ; E2E de D5, seconde partie : compteur ≥ 1, cf. D5 — *prévision, non jouée en E2E : la mutation n'a été jouée qu'en Vitest, où AC2 la tue ; revue P1, A1-4*) |
| M9 | la garde du client appliquée **toujours** (maintien ignoré : jamais d'effacement ni de redirection après un renouvellement échoué) | AC6 — les tests 401 existants du client (`:246-267`, `:327-343`, `:567-581`) rougissent : la preuve que la fermeture ordinaire n'est pas affaiblie |
| M10 | la fermeture de `recordActivity` non conditionnée au maintien | AC3 (b) (`keydown` sous maintien, après `last + w`) |
| M10′ | sous maintien, après `last + w`, `recordActivity` écrit l'activité | AC3 (b) (`lastActivity` relue inchangée) |
| M11 | la fermeture de `stayActive` non conditionnée au maintien | AC3 (b) (`stayActive()` sous maintien, après `last + w`) |
| M11′ | sous maintien, après `last + w`, `stayActive` écrit l'activité | AC3 (b) (`lastActivity` inchangée) |
| M12 | sous maintien, l'avertissement montré après `last + w` (formule d'aujourd'hui laissée telle quelle, `remaining` négatif) | AC3 (b) (`warningRemainingMs` `null` après `last + w`) |
| M12′ | sous maintien, l'avertissement éteint avant `last + w` (rédaction de la P1) | AC3 (b) (avertissement attendu à `last + w − 60 000`) |
| M13 | l'accesseur `restoreHold` ignore l'échéance (maintien sans borne) | AC5 bis (a) |
| M14 | la minuterie de B retirée du succès | AC5 bis (b) |
| M16 | le cas `UNAUTHENTICATED` de l'import retiré du `catch` (message affiché, pas d'`expireSession`) | AC4 |
| M17 | la confirmation pose `holdForRestore(B)` au lieu de la borne de la phase 1 | AC5 bis (c) et (d) (B = 1 min, import de plus d'une minute : maintien levé avant la réponse) |
| M17′ | la borne de la phase 1 réduite à `max(B, LONG_OPERATION_TIMEOUT_MS)` (sans le rejeu ni la marge, rédaction de la P3) | AC5 bis (c) (7 260 000 attendu) |
| M18 | `holdForRestore` prend le maximum de l'échéance courante et de la nouvelle au lieu de la remplacer (l'heure survit au succès) | AC5 bis (a) (seconde pose plus courte) et (d) (échéance à `+60 000` après le succès) — *au développement, tuée par (a) seul : (d) ne la discrimine pas, la minuterie de B levant le maintien à `+60 000` (revue P1, A1-1)* |
| M19 | au démontage en phase 2, `expireSession()` retiré (levée seule) | AC5 (c) |
| M20 | au démontage, `expireSession()` appelé même quand `reconnecting` est vrai | AC5 (d) |
| M21 | `keepHold` non posé au succès (le `finally` arme la levée au geste après `holdForRestore(B)`) | AC2 (`releaseRestoreHold` non appelé après le succès, puis un geste : toujours pas) (V4-7) |
| M22 | la minuterie de B non effacée au démontage en phase 2 | AC5 (c) (`expireSession` appelé deux fois) |
| M23 | l'écouteur de geste armé à la confirmation (au début de `confirmImport`) au lieu du `finally` | AC3 (a), « geste pendant l'import » (`releaseRestoreHold` appelé pendant l'import) (V5-1) |
| M24 | le retrait de l'écouteur en tête de `confirmImport` supprimé (revue P1, E1-1) | AC3 (a), E1-1 (nouvel import par `click` seul : geste pendant l'import) |
| M25 | `shortenRestoreHold(B)` retiré du `finally` (l'échéance de la confirmation gardée, revue P1, E1-2) | AC3 (a), E1-2 (`restoreHold` faux à `échec + 60 000`) |
| M26 | `shortenRestoreHold` prend le maximum au lieu du minimum (l'échéance allongée) | AC3 (a), E1-2 au store (« n'allonge jamais ») |
| M27 | la garde `!destroyed` retirée du `finally` (revue P2, E2-1) | E2-1 (réponse tardive d'un panneau démonté : `shortenRestoreHold` non appelé, maintien de l'autre panneau intact) |
| M28 | au succès, B de la confirmation au lieu de B relu (`const reading = b`) | E2-2 (B relu au succès : `holdForRestore(60 000)`, fin à `+60 000`) |
| M29 | à l'échec, B de la confirmation au lieu de B relu | E2-2 (B relu à l'échec : `shortenRestoreHold(60 000)`) |
| M30 | au succès, le maintien réarmé AVANT la minuterie (B1-3) | E2-2 (ordre des appels : `setTimeout(…, B)` avant le second `holdForRestore`) |

*Retirées à la P4 (simplification de D2, V4-1)* : M15 (« maintien gardé à l'issue inconnue ») — l'issue inconnue garde
désormais le maintien jusqu'au premier geste, ce que tuent M5 et M5″. Recompte de la table : M1, M1′, M2, M3, M4, M5,
M5″, M5‴, M6, M6′, M7, M7′a, M7′b, M7″, M8, M9, M10, M10′, M11, M11′, M12, M12′, M13, M14, M16, M17, M17′, M18,
M19, M20, M21, M22 — 32 mutations à la P4 ; **33** depuis la P5, avec M23 (recompté : `/usr/bin/grep -c '^| M'` sur
la fiche) ; M5′ et M15 retirées.

Une mutation équivalente se dit au Dev Agent Record.

## Tâches

- [x] **Étape 0** : relire à la tête de la branche les lignes citées ci-dessus ; rejouer les deux greps de l'inventaire et
      celui des minuteries, sorties citées au Change Log ; **relire `docs/manual/fr/admin-manual.tex:2340` après la
      fusion de la 36-4**, seul fichier commun possible (Dev Notes, *Fichiers communs* ; V4-11), et y placer la phrase de
      D6 sans toucher aux lignes de la 36-4.
- [x] **Répétition avant** (D4) : bases `kesh_e2e_e365` recréées à neuf (migrations du worktree, seed), `npm run build`
      au commit de base, backend selon la recette E2E des consignes (port attribué par le prompt de développement),
      `KESH_TEST_MODE=true PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=… KESH_INBOX_DIR=… KESH_DOCUMENTS_DIR=…
      npx playwright test tests/e2e/backup-restore-new-install.spec.ts:140 tests/e2e/backup-restore-new-install.spec.ts:262 --repeat-each=30 --retries=0 --reporter=line`,
      sous le verrou de la voie 1 ; `frontend/test-results/` copié avant tout rejeu (traces des rouges).
- [x] API du maintien inerte (`auth.svelte.ts`) ; tests de l'AC1 à l'AC6 et E2E de D5 ; rouges consignés.
- [x] D1 (`auth.svelte.ts`, `api-client.ts`, dont l'extraction d'`expireSession`), D2 (`session-idle.svelte.ts`, commentaire d'en-tête `:20-22` complété :
      « aucune suspension » devient faux), D3 (`AdminRestorePanel.svelte`, commentaire d'en-tête `:9-12` complété) ;
      commentaire de `sendHeartbeat` `:363` récrit (le 401 n'a plus toujours redirigé).
- [x] Les 33 mutations de la table (M5′ et M15 retirées).
- [x] **Répétition après** (D4) ; note de `docs/testing.md` et demi-phrase de `admin-manual.tex:2340` (D6 ; PDF à
      l'intégration) ; une capture du compte rendu resté affiché après le
      battement forcé (`KESH_E2E_CAPTURES_DIR`, thème clair) dans `kesh-gate-logs/e36-5/captures/`.
- [x] Gates (ci-dessous) ; `Status: review` et clé `review` au `sprint-status.yaml` dans le même commit ; prompt de la
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
*Se reconnecter*, quittiez la page ou laissiez passer la durée d'inactivité réglée ; il pouvait disparaître aussitôt,
l'écran passant à la connexion avec « session expirée ». (refs #931)

*(Annoté le 2026-11-15, revue P1 du code d'intégration 0.17.8, E2/B2/A6 : « ou laissiez passer la durée d'inactivité
réglée » ajouté — la borne de D2/D3, récrite à la revue P1 de la story, que la note écrite avant n'avait pas suivie ; le
manuel la disait déjà, `admin-manual.tex:2340`.)*

## Dev Notes

### Hors périmètre

- Reporter le compte rendu après la reconnexion (piste de #931) : écartée, D1 (c).
- Les **autres onglets** du même navigateur sont renvoyés à la connexion à leur prochaine activité : c'est le comportement
  que le manuel décrit (`admin-manual.tex:2340`, « chacune prend fin à sa prochaine activité »), voulu.
- **Cet onglet, quand un autre onglet est renvoyé à la connexion** (V1-6) : la diffusion `auth-change` fait relancer
  `hydrate()`, dont la branche 401 met `_currentUser = null` **avant** `tryRefresh()` (`auth.svelte.ts:322-330`) ; le
  layout racine arrête alors le module de session (`routes/+layout.svelte:33-42`, `sessionUserId` nul), et les
  `isAdmin` dérivés basculent à faux (`BackupScheduleBanner.svelte:29`, `(app)/+layout.svelte:45`) : le compte rendu
  reste (aucune redirection), mais le menu d'administration disparaît autour de lui. Hors périmètre, sans test ; issue
  ouverte par l'orchestrateur : #1180.
- **Une connexion faite dans un autre onglet pendant la phase 2** (V4-8) : sa diffusion `auth-change` relance
  `hydrate()` dans cet onglet (`auth.svelte.ts:396-402`), qui, `/me` rendant 200 sur le cookie neuf, change
  l'identifiant ; l'effet de `routes/+layout.svelte:33-42` relance alors le module de session, arrêté par le panneau.
  Pendant au plus B, cet onglet-ci absorbe les 401 de la session neuve et ne la ferme pas pour inactivité au-delà de son
  échéance ; à B, la minuterie du panneau mène à `/login?reason=session_expired`, sans déconnecter l'autre onglet au
  serveur. Borné par B, sans écriture ; limite assumée, sans test ; **issue demandée à l'orchestrateur** (« on trace
  toujours »).
- L'angle mort de la 31-1b sur l'échec au `COMMIT` (`AdminRestorePanel.svelte:49-57`) : inchangé ; après un tel échec
  (500, échec « certain » pour l'écran), le maintien est levé au premier geste et un 401 renvoie alors à la connexion,
  comme aujourd'hui.
- **Le maintien n'a pas de propriétaire** (revue de code P3, P3-2 ; tracé en **#1193**) : `onDestroy` du panneau lève le
  maintien sans vérifier qu'il l'a posé ; deux panneaux vivants en même temps (cas que l'application ne rend pas : une
  seule route à la fois) pourraient se lever le maintien l'un l'autre. La garde E2-1 (`!destroyed` dans le `finally`)
  couvre la réponse tardive d'un panneau démonté ; le reste est hors périmètre, sans changement de code.

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
  rendu. Borne (D3, formulation unique) : « Le maintien est posé à la confirmation pour `importHoldMs(B)` ; au succès,
  son échéance est remplacée par `maintenant + B` ; après un échec certain ou une issue inconnue, son échéance est
  ramenée à au plus `maintenant + B`, jamais allongée, et le maintien est levé au premier geste, au démontage ou à cette
  échéance ; au 401 de l'import, il est levé aussitôt. Il ne dure donc jamais au-delà de `confirmation + max(importHoldMs(B), durée de
  l'import + B)`. » Il ne survit ni à la disparition du panneau, ni à `logout()`, ni à `clearSession()`, ni à un rechargement (état de module). **Un renouvellement réussi ne le
  lève jamais** (`login()` ne le touche pas, V2-1) ; aucune connexion faite dans cet onglet n'en hérite (D1).
- Le maintien ne prolonge rien au serveur et ne change rien avant `last + w` : il diffère seulement le renvoi local à la
  connexion (D2). Sous maintien, le renouvellement est
  toujours tenté.
- Aucune donnée réelle : fixtures synthétiques ; rien sous `~/kesh/`.

### Références

- Issue #931 (commentaire du 2026-11-13 : séquence relevée) ; Story 31-1b (AC19, C-31-1-39 : pas de redirection avant le
  clic) ; Story v0172-session (#876) et v0174-session-2 (#895) ; C-e360-8.
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS* ; § *E2E (Playwright)* ; `docs/testing.md` § *Les échecs attendus*.

## Dev Agent Record

*Développement du 2026-11-14 au 2026-11-15, agent Opus 5.5, worktree `kesh-36-5`, voie de base 1 (`kesh_e365`,
`kesh_e2e_e365`, `kesh_ci_e365`, port 3365). Journaux hors dépôt : `/home/gcorbaz/devel/kesh-gate-logs/e36-5/`.*

**Commits** (de `f43562b4d`, la fiche validée) : `58a9af582` E2E déterministe et étape 0 (V6-1) ; `29762b89a` API du
maintien inerte, `expireSession` extraite, tests de l'AC1 à l'AC6 (rouges) ; `69f18b23a` le correctif (D1, D2, D3) —
**dernier commit qui touche du code de production** ; `fe40d7bcc` puis `0e9da291c` l'E2E de D5 corrigé (C-36-5-13) —
**dernier commit qui touche du code**, un test ; `f4e6afd57` `docs/testing.md`, manuel, registre ; puis ce compte rendu.

**Fichiers de production** : `frontend/src/lib/app/stores/auth.svelte.ts` (état `_restoreHoldUntil`, `restoreHold`,
`holdForRestore`, `releaseRestoreHold` ; levée dans `clearSession()` et en première instruction de `logout()` ; `login()`
inchangé), `frontend/src/lib/shared/utils/api-client.ts` (`expireSession()` exportée ; garde `!authState.restoreHold` dans
`refreshTokens`, seul lieu de l'effacement et de la redirection, qui couvre `request` et `requestRaw`),
`frontend/src/lib/app/stores/session-idle.svelte.ts` (D2 dans `tick`, `recordActivity`, `stayActive` ; en-tête et
commentaire de `sendHeartbeat` complétés), `frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte` (D3),
`frontend/src/lib/features/admin-restore/admin-restore.api.ts` (`RESTORE_HOLD_MARGIN_MS`, `importHoldMs`). Aucun fichier
Rust, aucune migration, aucune clé i18n.

**Tests ajoutés** (recomptés `git show <ref>:<fichier> | grep -cE "^\s*(it|test)\("`, de `f43562b4d` à `HEAD`) :
Vitest **28** — `api-client.test.ts` 36 → 42 (+6), `auth.svelte.test.ts` 21 → 27 (+6), `session-idle.svelte.test.ts`
52 → 54 (+2), `AdminRestorePanel.test.ts` 15 → 29 (+14) ; E2E **1** — `backup-restore-new-install.spec.ts` 3 → 4 (le
test « KF-056 », `:381`). *(Annoté le 2026-11-15, revue P2, A2-4 : « `HEAD` » valait `1fbda93a7` à la date de ce
relevé. Recompté à `fb2012f1b`, même commande : `auth.svelte.test.ts` **29**, `AdminRestorePanel.test.ts` **35**, les
autres inchangés (42, 54, E2E 4) — Vitest **36** ajoutés au total de `f43562b4d` à `fb2012f1b` : 28 au développement,
4 à la remédiation P1, 4 à la remédiation P2.)* Les tests existants n'ont changé ni de corps ni d'assertion ; seuls les simulacres sont
étendus (`session-idle.svelte.test.ts` : `restoreHold` réglable, remis à faux au `beforeEach` ; `AdminRestorePanel.test.ts` :
simulacres partiels d'`admin-restore.api` et d'`api-client`, `sessionIdle` simulé, maintien délégué au vrai `authState`,
C-36-5-11).

**Rouges** (`rouges.log`, `rouges-vitest.log`, commit `29762b89a`) : Vitest ciblé des quatre fichiers, **20 rouges,
167 verts** (187), chacun sur une assertion de comportement. Verts au départ, et dits tels : AC1 (b), AC5 bis (a) et
AC5 (a) côté client, `expireSession`, `clearSession()`/`logout()` remettent à zéro (par construction), AC3 (b) « avant
l'échéance », AC5 (d) côté panneau. **E2E de D5 : rouge 3 sur 3 avant le correctif** (`repetition-avant-d5-r3.log`) à
l'attente du compte rendu, **après** que l'import a rendu 200 et le battement 401 ; traces (`test-results-avant-d5-r3/`) :
`frameUrl` `…/login?reason=session_expired`. ⚠️ **Les deux premiers rouges de ce test ne prouvaient rien** : par
`page.route` + `route.fetch()`, l'archive arrivait au serveur illisible (400 « Could not find EOCD »,
`backend-avant-d5.log`, `backend-avant-d5-r2.log`) ; le premier a d'abord été écrit au journal comme le rouge de la
KF-056, sur la foi d'une chaîne `login?reason=session_expired` trouvée dans la trace — elle venait du code servi.
L'annotation datée est dans `rouges.log` ; le test récrit (C-36-5-13) contrôle désormais le statut de l'import et du
battement avant l'affichage.

**Verts** : Vitest ciblé après le correctif (`verts-vitest.log`, `69f18b23a`) : 6 fichiers, **196/196** ;
`npm run check` 0 erreur, `lint-i18n-ownership` vert (`check-dev.log`, `lint-i18n-dev.log`).

**Mutations** (`mut/mutations.py`, `mut/resume.log`, `mut/M*.log` ; code commité avant, `69f18b23a` ; restauration par
`git restore --worktree` puis `touch`, `git status` propre après la campagne) : **33 sur 33 tuées**, chacune par le test
que la fiche nomme (diff et rouges dans chaque `M*.log`) ; M9 fait rougir trois tests 401 existants du client
(« refresh échoué → clearSession + redirect login », « refresh 200 sans expiresIn », « refresh fetch échoue (réseau) »)
plus deux neufs : la fermeture ordinaire n'est pas affaiblie. Aucune mutation équivalente. Les mutations sont
Vitest seulement ; la seconde partie de l'E2E (M8) n'a pas été mutée en E2E.

**Répétition (D4, AC7)** :
- *avant* (`repetition-avant.log`) : `:140` et `:262`, 30 passages chacun, **60 passés, 0 rouge** — frontend servi : le
  build d'un export de `29762b89a` (API inerte, comportement d'avant le correctif ; C-36-5-10), backend de la branche.
  **Fréquence observée : 0/30 pour `:140`, 0/30 pour `:262`.** La répétition ne reproduit donc pas la KF-056 ici ; sans
  rouge avant, la répétition « après » ne prouve rien par elle-même : **la preuve est celle du test de D5** (rouge 3 sur 3
  avant, vert 10 sur 10 après).
- *après* (`repetition-apres.log`, build de `69f18b23a`) : **60 passés, 0 rouge** ; test de D5 dix fois
  (`repetition-apres-d5-r2.log`, spec de `0e9da291c`) : **10 passés**. Les journaux `repetition-apres-d5.log` (10 rouges)
  et `repetition-avant-d5*.log` antérieurs sont ceux des versions défectueuses du test, gardés.
- Capture : `captures/j-compte-rendu-apres-battement-refuse.png` (thème clair : compte rendu, *Se reconnecter*, URL
  `/admin/restore` après le battement refusé).

**Gates au passage en revue** (bases remises à neuf ; commit `f4e6afd57`, code identique à `0e9da291c`) :
- `scripts/test-fast.sh --ci` sur `kesh_e365` : **5445 exécutés, 5445 passés, 13 ignorés** (`test-fast-ci.log`).
- `scripts/gate-ci.sh kesh_ci_e365` : **5452 passés, 0 échoué, 17 ignorés**, code 0 (`gate-ci.log`, `gate-ci.console.log`)
  — `textes_coherents.rs` compris.
- Frontend complet : `npm run check` 0 erreur, 29 avertissements préexistants, aucun dans les fichiers touchés
  (`front-check.log`) ; `lint-i18n-ownership` vert (`front-lint-i18n-ownership.log`) ; `test:unit` **234 fichiers,
  3459 tests** (`front-test-unit.log`) ; `npm run build` vert (`front-build.log`).
- **Non lancé** : l'E2E complet (à la clôture, consigne de l'Epic 36).

**AC9** : `git grep -nE "holdForRestore\(" -- frontend/src ':!*.test.ts'` → `auth.svelte.ts:151` (définition),
`AdminRestorePanel.svelte:221` (confirmation), `:235` (succès) ; aucune ligne de commentaire rendue. *(Annoté le
2026-11-15, revue P2, A2-4 : lignes valables à `69f18b23a`. À `fb2012f1b` : `auth.svelte.ts:155` (définition),
`AdminRestorePanel.svelte:230` (confirmation), `:254` (succès) ; et `shortenRestoreHold(` : `auth.svelte.ts:163`
(définition), `AdminRestorePanel.svelte:277` (le `finally`).)* **AC8** :
`git grep -n "KF-056" -- docs/testing.md` → `:431` ; `git grep -n "Laissé ouvert, ou quitté"` → `admin-manual.tex:2340` ;
PDF (non régénéré, D6) : « Prenez le temps de lire, puis cliquez » → `1`, la phrase neuve → `0` (attendu jusqu'à
l'intégration).

**Inventaire rejoué** (étape 0, puis après le correctif) : sorties identiques à la fiche, à une ligne près — le grep
`(goto|redirect)\(…'/login` rend aussi `SetupForm.svelte:27`, un **commentaire** (« `goto('/login')` s'exécute hors
contexte ») omis de la sortie copiée par la fiche ; hors du symptôme. Après le correctif, `api-client.ts:202` devient
`:219` (dans `expireSession`), appelée par `refreshTokens` (`:206`, sous la garde) et par le panneau (`:70`, `:106`),
deux sites atteints seulement à l'échéance de B ou au démontage, voulus (D3). Minuteries : inchangées, plus la
minuterie de lecture du panneau (`setTimeout`, D3).

**Écarts** : (1) D5 n'emploie pas `page.route` + `route.fetch()` pour retenir la réponse (C-36-5-13) ; (2) la
répétition « avant » sert un export du commit des rouges, non le worktree (C-36-5-10) ; (3) le PDF n'est pas régénéré
(D6, à l'intégration) ; (4) consigne Edit : aucun écart sur le code — les remplacements de `mut/mutations.py` sont ceux
des mutations, permis ; (5) la relecture de `admin-manual.tex:2340` « après la fusion de la 36-4 » n'a pas pu se faire :
la 36-4 n'est pas fusionnée ; la demi-phrase est insérée dans la seule ligne `:2340`, sans toucher à aucune autre ligne,
et l'union se vérifie à l'intégration.

**Issues à ouvrir** (demandées à l'orchestrateur) : la connexion faite dans un autre onglet pendant la phase 2 (Dev Notes,
*Hors périmètre*, V4-8 : cet onglet absorbe les 401 de la session neuve et ne la ferme pas pour inactivité, au plus B).

### Remédiation de la revue de code P1 (2026-11-15)

Revue P1 : B 0/0/0/3, E 0/1/1/2, A 0/0/0/5 (`kesh-gate-logs/e36-5/review-p1-{B,E,A}.md`). Prémisses relues au code avant
d'appliquer. Commits : `f142298e7` (code et tests, **dernier commit de code**), `85359e0f2` (fiche, manuel), puis ce compte
rendu (avec le registre C-36-5-14, C-36-5-15 et le prompt de la P2).

- **E1-1 (HIGH) = B1-1** — confirmé : `confirmImport` ne retirait pas l'écouteur d'un échec précédent (`grep -nF
  "disarmGesture"` : aucun appel dans `confirmImport`). Corrigé : `disarmGesture?.()` après la garde de ré-entrance, avant le
  maintien. Test : échec 400, nouvel import par `click` seul, `keydown`/`pointerdown` pendant l'import puis pendant la lecture :
  aucune levée, `restoreHold` vrai. Mutation **M24** tuée.
- **E1-2 (MEDIUM)** — confirmé : le `finally` gardait l'échéance de la confirmation (≥ 121 min). Corrigé :
  `authState.shortenRestoreHold(B)` (nouveau, `min(échéance, maintenant + B)`, rien sans maintien) dans le `finally` (C-36-5-14).
  Formulation unique de la borne récrite aux trois endroits (identiques, recompté), D3 « issue inconnue et échec certain »
  et AC3 (a) complétés ; la borne finale reste vraie. Tests : deux au store, un au panneau (B = 1 min, import de 90 s puis
  échec : vrai à `+59 999`, faux à `+60 000`). Mutations **M25** et **M26** tuées. Manuel : la phrase du succès ne change pas ;
  le paragraphe de l'issue inconnue (`:2342`) ne décrit pas de durée, inchangé.
- **E1-3 (LOW)** — gardé et écrit : la minuterie de lecture est armée en premier, et un commentaire dit pourquoi rien ne jette
  entre `keepHold = true` et la fin du `try` (une lecture, un `setTimeout`, une affectation, `stop()` qui retire des écouteurs).
- **E1-4 (LOW)** — B relu au succès et dans le `finally` (`readingMs()`), la phase 1 gardant la valeur de la confirmation
  (C-36-5-15) ; incise au manuel : « soixante minutes si l'écran ne la connaît pas encore ».
- **B1-2 (LOW)** — commentaire de `_restoreHoldUntil` : `hydrate()` ne lève pas le maintien, et pourquoi.
- **B1-3 (LOW)** — garanti par l'ordre : minuterie armée avant le maintien ; le cas d'une minuterie retardée par le
  navigateur est écrit au commentaire (un 401 entre-temps mène à la même connexion).
- **A1-1 (LOW)** — table annotée : M18 tuée par AC5 bis (a) seul.
- **A1-2 (LOW)** — les journaux « avant » sont annotés (commit du code servi : l'export de `29762b89a`) ; deux annotations
  portaient d'abord un commit faux (`69f18b23a` pour `-r2` et `-r3`), corrigées par une ligne ajoutée. `e2e-e36-5.sh` écrit
  désormais le commit du code servi et celui du worktree.
- **A1-3 (LOW)** — le rouge de l'E2E nomme la page (`compte rendu jamais affiché — page sur <URL>`). Écart au remède proposé :
  un contrôle d'URL placé **avant** la visibilité passerait, la redirection pouvant ne pas être encore faite ; l'URL est donc
  écrite dans le message d'échec de la visibilité.
- **A1-4 (LOW)** — M8 non jouée en E2E : la ligne de la table le dit.
- **A1-5 (LOW)** — les journaux de la remédiation portent leur commit en première ligne.

**Tests de la remédiation** (de `1fbda93a7` à `f142298e7`) : Vitest +4 (`auth.svelte.test.ts` 27 → 29,
`AdminRestorePanel.test.ts` 29 → 31) ; l'E2E modifié (message), non ajouté. **Mutations**, campagne 2 sur `f142298e7`
(`mut/resume.log`, `mut/campagne-2.console.log`) : **36 sur 36 tuées** (M1 à M23, plus M24 à M26 ; ancres de M5, M5″ et M14
mises à jour pour le `finally` en bloc et la minuterie de `reading`).

**Gates de la remédiation** (bases `kesh_e365` et `kesh_e2e_e365` remises à neuf ; commit `85359e0f2`, dernier commit de code
`f142298e7`, en première ligne de chaque journal) :
- Vitest ciblé : 5 fichiers, 194/194 (`r1-vitest-cible.log`, sur `f142298e7` non commité encore — avant le commit, mêmes sources).
  *(Annoté le 2026-11-15, revue P2, A2-3 : « commit en première ligne de chaque journal » est faux pour ce journal-là,
  lancé avant le commit ; une ligne datée l'écrit à sa fin.)*
- `scripts/test-fast.sh --ci` : **5445 passés, 13 ignorés** (`r1-test-fast-ci.log`).
- Frontend complet : `check` 0 erreur (29 avertissements préexistants) ; `lint-i18n-ownership` vert ; `test:unit` **234 fichiers,
  3463 tests** ; `build` vert (`r1-front-*.log`).
- Test de D5 dix fois sur ce build : **10 passés** (`repetition-apres-d5-r3.log`, test à `:381`).
- **Non lancés**, selon la consigne : `gate-ci` et l'E2E complet (clôture).

### Remédiation de la revue de code P2 (2026-11-15)

Revue P2, complète à deux lentilles : E 0/0/0/4, A 0/0/0/6 (`review-p2-{E,A}.md`) — rien au-dessus de LOW. Décision de
l'orchestrateur (« Tarir la source ») : les LOW se corrigent dans la story ; E2-1 touchant la production, une passe ciblée P3
relit ce correctif (`36-5-restauration-compte-rendu-review-prompt-p3-ciblee.md`). Prémisses relues au code. Commits :
`fb2012f1b` (code et tests, **dernier commit de code**), `eea1e0ea6` (fiche, manuel, `sprint-status.yaml`), puis ce compte
rendu.

- **E2-1** — confirmé (`AdminRestorePanel.svelte`, `finally` : le `return` sur `destroyed` laissait `keepHold` faux, et
  `shortenRestoreHold` touchait un maintien de module qui pouvait appartenir à un autre panneau). Corrigé :
  `if (!keepHold && !destroyed)`. Test : panneau A démonté pendant son import, panneau B sous maintien, la réponse de A
  arrive en échec — `shortenRestoreHold` non appelé, le maintien de B tient après une heure. **M27** tuée.
- **E2-2** — deux tests : B relu au succès (`minutes` 5 → 1 pendant l'import : `holdForRestore(60 000)`, fin à `+60 000`) et
  à l'échec (`shortenRestoreHold(60 000)`) ; **M28**, **M29** tuées. L'ordre minuterie puis maintien (B1-3) est observable par
  l'ordre des appels (espion sur `setTimeout`, appel de délai B avant le second `holdForRestore`) : test ajouté, **M30** tuée.
  Ce test fixe l'ordre, non une différence de comportement (à échéance égale sous horloge simulée, les deux ordres donnent
  le même résultat).
- **E2-3** — en-tête de `session-idle.svelte.ts` : « différée, jamais annulée » vaut pendant l'import et après un échec ;
  après un succès, le panneau arrête le module et termine la session au bout de B.
- **E2-4** — **rien à changer** : après un échec suivant un import plus long que B, le premier geste lève le maintien et le
  tic suivant ferme (`/login?reason=idle`) — c'est D2 (fermeture différée, jamais annulée), conforme à la fiche.
- **A2-1** — D1 nomme la troisième méthode, `shortenRestoreHold` ; *Simulacres* aussi.
- **A2-2** — la définition de B dit qu'il est lu à la confirmation pour la phase 1, relu au succès et à l'échec.
- **A2-3** — `r1-vitest-cible.log` (et `r2-vitest-cible.log`), lancés avant leur commit : une ligne datée l'écrit à leur fin ;
  annotation au Dev Agent Record, sans réécrire le relevé.
- **A2-4** — décompte des tests et lignes de l'AC9 annotés (valeurs à `fb2012f1b` : `auth.svelte.test.ts` 29,
  `AdminRestorePanel.test.ts` 35 ; **36** Vitest ajoutés au total), relevés d'origine laissés tels quels.
- **A2-5** — `sprint-status.yaml:1078` à jour.
- **A2-6** — demi-phrase ajoutée à `admin-manual.tex:2342` : « Laissé ouvert sans activité, cet écran mène à la connexion
  au plus tard au bout de la durée d'inactivité réglée. » Vraie au code : après une issue inconnue, le maintien est ramené à
  au plus `échec + B` ; sans activité, `last + w` est alors passé et le tic suivant ferme (ou, la session invalidée, le
  premier 401 mène à la connexion). PDF à l'intégration.

**Tests** de la remédiation P2 : Vitest +4 (`AdminRestorePanel.test.ts` 31 → 35). **Mutations**, campagne 3 sur `fb2012f1b`
(`mut/campagne-3.console.log`) : **40 sur 40 tuées** (M1 à M30).

**Gates** (bases `kesh_e365` et `kesh_e2e_e365` remises à neuf ; commit `eea1e0ea6`, dernier commit de code `fb2012f1b`, en
première ligne de chaque journal) : `scripts/test-fast.sh --ci` **5445 passés, 13 ignorés** (`r2-test-fast-ci.log`) ;
frontend complet — `check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` vert, `test:unit` **234 fichiers,
3467 tests**, `build` vert (`r2-front-*.log`) ; test de D5 dix fois sur ce build : **10 passés** (`repetition-apres-d5-r4.log`).
Vitest ciblé avant le commit : 163/163 (`r2-vitest-cible.log`, sans ligne de commit, annoté). **Non lancés** : `gate-ci` et
l'E2E complet (clôture).

### Remédiation de la passe ciblée P3 (2026-11-15)

Passe ciblée P3 (Haiku) : 0/0/1/3 (`review-p3-ciblee.md`) ; la garde E2-1 est jugée correcte. Prémisses relues au code.
Commit `68c949fa2` (manuel et test, **dernier commit de code** ; aucune ligne de production), puis ce compte rendu et le
prompt de la passe ciblée P4 (`36-5-restauration-compte-rendu-review-prompt-p4-ciblee.md`).

- **P3-1 (MEDIUM)** — confirmé : sans stockage local partagé, le module de session ne ferme rien de lui-même
  (`session-idle.svelte.ts`, `tick` : `if (w !== null && shared && now >= last + w)` ; en-tête : « ne ferme rien de lui-même »),
  si bien que l'écran d'une issue inconnue laissé ouvert ne mène pas toujours à la connexion. **Décision de l'orchestrateur** :
  la demi-phrase ajoutée pour A2-6 (finding facultatif) à `admin-manual.tex:2342` est **retirée** ; la ligne `:2342` est de
  nouveau celle de `f43562b4d`. La phrase de `:2340` (succès) ne dépend pas du stockage : la minuterie de lecture du panneau
  termine la session au bout de B.
- **P3-2 (LOW)** — tracé en **#1193** par l'orchestrateur ; cité aux Dev Notes (*Hors périmètre*), sans changement de code.
- **P3-3 (LOW)** — le test « (B1-3) » restaure son espion et démonte son panneau dans un `finally`.
- **P3-4 (LOW)** — les fichiers exécutés par `r1-vitest-cible.log` et `r2-vitest-cible.log` (cinq chacun) sont nommés par une
  ligne datée ajoutée à leur fin.

**Gates** (commit `68c949fa2` en première ligne) : Vitest ciblé `admin-restore/` 41/41 (`r3-vitest-cible.log`, avant le
commit) ; frontend complet — `check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` vert, `test:unit`
**234 fichiers, 3467 tests**, `build` vert (`r3-front-*.log`). **Non relancés** : `test-fast` (aucun fichier Rust ni `.ftl`
touché, consigne de l'orchestrateur), test de D5 (le code servi n'a pas changé) ; `gate-ci` et l'E2E complet à la clôture.

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
- 2026-11-14 — **Validation P3 (Sonnet) : 0/1/1/5 → remédiés** (`kesh-gate-logs/e36-5/validation-p3.md`), par l'agent de
  rédaction (Opus 5.5). Prémisses relues au code, toutes confirmées :
  - V3-1 (HIGH) : `api-client.ts:53` (`LONG_OPERATION_TIMEOUT_MS = 60 * 60_000`), `admin-restore.api.ts:46-48`,
    `session-idle.svelte.ts:76-77` (B de 1 à 1 440 min). Décision de l'orchestrateur, C-36-5-8 : phase 1 bornée par
    `max(B, LONG_OPERATION_TIMEOUT_MS)`, constante importée (seule définition : `git grep -n "60 \* 60_000"` → `:53`) ;
    phase 2 bornée par B ; la pose **remplace** l'échéance, si bien que l'heure ne survit ni au succès, ni à l'échec, ni à
    l'issue inconnue, ni au démontage. D1, D2, D3, AC5 bis (a) (c) (d), M17, M18, *Ce qui doit rester vrai*.
  - V3-2 (MEDIUM) : D1 ne nommait aucun accesseur de l'échéance. → `restoreHoldDeadline: number | null` public, porté
    par l'API inerte (`null`) et par les simulacres de `session-idle.svelte.test.ts` et du panneau ; D2, AC6, M12, M13 le
    citent.
  - V3-3 : AC9 parle des sites d'appel lus dans `git grep -nE "holdForRestore\("`, commentaires écartés en les citant.
  - V3-4 : conduite prudente — au démontage en phase 2 (session connue morte, module arrêté ; `routes/+layout.svelte:33-42`
    ne le relance pas), `expireSession()` après la levée, sauf si `reconnecting` (`AdminRestorePanel.svelte:177-178`) ;
    AC5 (c) (d), M19, M20.
  - V3-5 : `api-client.test.ts:80` (`authState.clearSession()` au `beforeEach`) : la levée par `clearSession`/`logout` se
    prouve dans `auth.svelte.test.ts` seul ; écrit en AC1 bis.
  - V3-6 : M7′ séparée en M7′a (`logout()`) et M7′b (`clearSession()`) ; `fetch` simulé pour `logout()`.
  - V3-7 : `auth.svelte.ts:204-212` (`await fetch(…).catch(() => {})`, sans délai d'abandon) : levée en **première
    instruction** de `logout()`, avant le `fetch` (décision de l'orchestrateur ; la lentille proposait « après ») ; AC6,
    M7″.
  Modules : inchangés (cinq, dont trois de code). Aucune migration.
- 2026-11-14 — **Validation P4 (Opus) : 0/0/3/8 → remédiés** (`kesh-gate-logs/e36-5/validation-p4.md`), par l'agent de
  rédaction (Opus 5.5), en **simplifiant D2** sur décision de l'orchestrateur (C-36-5-9 ; la boucle convergeait mal,
  chaque passe trouvant un défaut né de la remédiation précédente). Prémisses relues au code, toutes confirmées :
  - V4-1 (MEDIUM) : `middleware/auth.rs:156-161` (aucune requête au-delà d'`idle_exp`), `routes/auth.rs:25-33`
    (échéance écrite dans `refresh_tokens.expires_at`) : le serveur ferme à `last + w` quoi qu'il arrive ; le maintien ne
    diffère que le renvoi local. → D2 récrit : avant `last + w`, comportement d'aujourd'hui (battement, avertissement,
    *Rester connecté*) ; après, sous maintien, ni fermeture, ni écriture d'activité, ni avertissement ; fermeture à la
    levée ou à l'échéance du maintien. L'accesseur `restoreHoldDeadline` (P3) est retiré : plus rien ne le lit.
  - V4-2 (MEDIUM) : `session-idle.svelte.ts:247-258`, `:466-478` : AC3 et AC6 se contredisaient sur `stayActive()`.
    → AC3 (b) fixe les deux régimes ; la partie « avertissement » d'AC6 est retirée (sans objet) ; M10′, M11′ ; M12 et
    M12′ redéfinies.
  - V4-3 (MEDIUM) : AC5 (a), résidu de la P3, récrite (« sans import ou pendant l'import ») ; la minuterie de B effacée
    au démontage en phase 2 passe en AC5 (c) (M22).
  - V4-4 : `api-client.ts:319` (délai par envoi) et `:476-485` (un seul rejeu, `!isRetry`) : borne de la phase 1 =
    `max(B, 2 × LONG_OPERATION_TIMEOUT_MS + RESTORE_HOLD_MARGIN_MS)`, marge d'une minute écrite ; AC5 bis (c) à
    7 260 000 ms ; M17′.
  - V4-5 : à l'échec certain et à l'issue inconnue, levée au **premier geste** (`pointerdown` ou `keydown` sur la
    fenêtre), au démontage ou à l'échéance — forme la plus simple ; l'ancien AC4 (issue inconnue) est versé à l'AC3 (a),
    l'ancien AC4 bis devient AC4 ; M5, M5″, M5‴ ; M15 retirée (sans objet).
  - V4-6 : AC5 bis (b) attend `releaseRestoreHold` puis `expireSession` (simulacre) ; contenu d'`expireSession` prouvé au
    client ; `clearSession` retiré du simulacre du panneau.
  - V4-7 : M21 (`keepHold` non posé au succès), tuée par AC2 complété.
  - V4-8 : `auth.svelte.ts:396-402` : D1 dit « connexion faite dans cet onglet » ; la connexion dans un autre onglet
    pendant la phase 2 est écrite en *Hors périmètre*, issue demandée.
  - V4-9 : D6 : « Laissé ouvert, ou quitté, l'écran mène à la connexion … » ; AC8.
  - V4-10 : inventaire élargi sans filtre de destination (63 lignes ; `api-client.ts:534`, `DemoBanner.svelte:28`,
    `notify.ts:118`, `(app)/+layout.svelte:41` classés : aucun atteint sans geste).
  - V4-11 : étape 0 → relire `admin-manual.tex:2340` après la fusion de la 36-4 ; registre retouché (titre de C-36-5-5,
    marques « remplacé », ordre) ; la phrase du fichier d'epic (`:91-92`, « aucun fichier commun ») est laissée à
    l'orchestrateur.
  Recompte : **11 critères** (AC1, AC1 bis, AC2 à AC5, AC5 bis, AC6 à AC9) ; **32 mutations** (table, M5′ et M15
  retirées). Modules : inchangés (cinq, dont trois de code). Aucune migration.
- 2026-11-14 — **Validation P5 (Sonnet) : 0/0/1/6 → remédiés** (`kesh-gate-logs/e36-5/validation-p5.md`), par l'agent de
  rédaction (Opus 5.5), sans rien ajouter d'autre que ce que demandent les findings. Prémisses relues au code :
  - V5-1 (MEDIUM) : rien ne gardait un geste pendant l'import. → AC3 (a) « geste pendant l'import », M23 ; **33
    mutations** (`/usr/bin/grep -c '^| M'` → 33).
  - V5-2 : une seule formulation de la borne, écrite en D3 et reprise mot pour mot en D2 et dans *Ce qui doit rester
    vrai* ; juste au succès, à l'échec, à l'issue inconnue et au 401 de l'import.
  - V5-3 : `RESTORE_HOLD_MARGIN_MS` et la fonction unique `importHoldMs` exportées d'`admin-restore.api.ts` ;
    l'hypothèse sur `doRefresh` (`api-client.ts:92`) et `/me` (`:133`), sans délai d'abandon, écrite et acceptée, non
    étendue ; `vi.mock` d'`admin-restore.api` rendu partiel.
  - V5-4 : `sprint-status.yaml:1078` mis à jour.
  - V5-5 : AC6 « sans modification de leurs corps ni de leurs assertions », simulacres étendus.
  - V5-6 : *Retenu* de C-36-5-8 marqué « remplacé en partie par C-36-5-9 ».
  - V5-7 : montré au code qu'un double `expireSession()` ne peut arriver (`(app)/+layout.svelte:453`,
    `onboarding/+layout.svelte:33`, `routes/+layout.svelte:118` rendent la page sans condition ; `location.replace`
    décharge sans nettoyage) ; aucune garde ajoutée.
  Recompte : **11 critères**, **33 mutations**. Modules : inchangés. Aucune migration.
- 2026-11-14 — **Validation P6 ciblée (Haiku) : 0/0/0/1** (`kesh-gate-logs/e36-5/validation-p6-ciblee.md`). **Validation
  terminée en six passes**, sur une passe sans finding au-dessus de LOW. Trend (CRITICAL/HIGH/MEDIUM/LOW) : P1 Sonnet
  0/0/3/7 → P2 Opus 0/1/0/9 → P3 Sonnet 0/1/1/5 → P4 Opus 0/0/3/8 → P5 Sonnet 0/0/1/6 → P6 ciblée Haiku 0/0/0/1.
  Reclassement : aucun. Simplification de D2 décidée en P4 par l'orchestrateur (C-36-5-9).
- 2026-11-14 — **Étape 0 du développement** (Opus 5.5) : V6-1 (LOW) traité dans la fiche — ligne lue, D3 :
  « - **Borne de la phase 1 : `max(B, 2 × LONG_OPERATION_TIMEOUT_MS + RESTORE_HOLD_MARGIN_MS)`** » récrite
  « **Borne de la phase 1 : `importHoldMs(B)`**, la fonction unique décrite plus bas » ; l'expression du maximum n'est
  plus écrite qu'à la description d'`importHoldMs`. Greps de l'inventaire et des minuteries rejoués (Dev Agent Record,
  *Inventaire rejoué*) ; la 36-4 n'étant pas fusionnée, la phrase de D6 est placée dans la seule ligne `:2340`.
- 2026-11-15 — **Développement** (Opus 5.5) : D1, D2, D3, D6 livrés ; 28 tests Vitest et 1 E2E ; 20 rouges au départ,
  E2E de D5 rouge 3/3 avant et vert 10/10 après ; 33 mutations tuées sur 33 ; répétition 0 rouge avant (0/30, 0/30) et
  après ; `test-fast --ci`, `gate-ci`, frontend complet verts ; E2E complet à la clôture. Choix C-36-5-10 à C-36-5-13.
  Détail et journaux : Dev Agent Record. `Status: review`.
- 2026-11-15 — **Revue de code P1** (B, E, A) : 0/0/0/3, 0/1/1/2, 0/0/0/5 → **remédiée** (Opus 5.5) : E1-1 (HIGH) et E1-2
  (MEDIUM) corrigés au code (`f142298e7`), la borne du maintien récrite (« échéance ramenée à au plus `maintenant + B`,
  jamais allongée », C-36-5-14) ; les dix LOW traités (code, fiche, journaux, manuel). 4 tests Vitest, mutations M24 à M26 ;
  36/36 tuées. `test-fast --ci`, frontend complet, test de D5 ×10 verts. La règle de la borne changeant, la passe suivante
  est **complète, à deux lentilles** (E, A) : `36-5-restauration-compte-rendu-review-prompt-p2.md`.
- 2026-11-15 — **Revue de code P2** (E, A, passe complète) : 0/0/0/4, 0/0/0/6 — rien au-dessus de LOW. LOW corrigés dans la
  story (décision de l'orchestrateur, « Tarir la source ») : E2-1 en production (`fb2012f1b`), E2-2 (tests de B relu et de
  l'ordre), E2-3 (commentaire), A2-1 à A2-6 (fiche, annotations datées, `sprint-status.yaml`, manuel `:2342`) ; E2-4 sans
  changement. 4 tests Vitest, mutations M27 à M30 ; 40/40 tuées. `test-fast --ci`, frontend complet, test de D5 ×10 verts.
  Trend de la revue : P1 (B, E, A) 0/0/0/3, 0/1/1/2, 0/0/0/5 → P2 (E, A) 0/0/0/4, 0/0/0/6. Le correctif E2-1 touchant la
  production, la passe suivante est **ciblée** : `36-5-restauration-compte-rendu-review-prompt-p3-ciblee.md`.
- 2026-11-15 — **Passe ciblée P3** (Haiku) : 0/0/1/3. P3-1 (MEDIUM, manuel) : la demi-phrase d'A2-6 à `admin-manual.tex:2342`
  est **retirée** (décision de l'orchestrateur : fausse sans stockage local partagé ; A2-6 était facultatif). P3-2 tracé en
  #1193 (Dev Notes) ; P3-3 (test en `finally`) et P3-4 (annotations) traités. Aucune ligne de production touchée
  (`68c949fa2`) ; frontend complet vert. Trend : P1 0/1/1/10 (trois lentilles) → P2 0/0/0/10 (deux) → P3 ciblée 0/0/1/3. Le
  MEDIUM appelle une passe de plus : **P4 ciblée**, `36-5-restauration-compte-rendu-review-prompt-p4-ciblee.md`.
- 2026-11-15 — **Passe ciblée P4** (Sonnet) : 0/0/0/0 (`kesh-gate-logs/e36-5/review-p4-ciblee.md`). **Revue terminée.**
  Trend : P1 (B 0/0/0/3, E 0/1/1/2, A 0/0/0/5) → remédiés `f142298e7` → P2 (E 0/0/0/4, A 0/0/0/6) → remédiés `fb2012f1b` →
  P3 ciblée Haiku 0/0/1/3 → remédiés `68c949fa2` → P4 ciblée Sonnet 0/0/0/0. Modèles : Sonnet ou Opus aux passes complètes,
  Haiku et Sonnet aux ciblées. Reclassement : aucun ; issues tracées par l'orchestrateur : #1180, #1193.
- 2026-11-15 — **Clôture** (Opus 5.5), au dernier commit de code `68c949fa2` (journaux à `2686eb188`, commit en première
  ligne) : `scripts/gate-ci.sh kesh_ci_e365` **5452 passés, 0 échoué, 17 ignorés** (`clot-gate-ci.log`,
  `clot-gate-ci.console.log`) ; **E2E complet** sur `kesh_e2e_e365` recréée à neuf, port 3365 (`clot-e2e.log`, frontend
  servi : le build de `68c949fa2`) : **409 passés, 7 échoués, 18 ignorés** — les sept de la KF-029 (#97), nommés un par un
  contre `docs/testing.md` § *Les échecs attendus* (`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ;
  `onboarding.spec.ts:57`, `:77`, `:150`), aucune pollution ; `backup-restore-new-install.spec.ts:140`, `:262`, `:301` et le
  test de D5 (`:381`) verts ; port 3365 libre avant et après. `Status: done`.
