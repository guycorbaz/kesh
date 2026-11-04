# Story v0172-session : la 0.17.2 — la session expire pendant une activité continue (#876)

Status: ready-for-dev

*Fiche sœur de `v0172-correctifs.md`, extraite le 2026-11-04 de sa sous-story A1 à la remédiation de la validation P2
(signal de découpage : sévérité stagnante, 12 MEDIUM puis 11, les défauts recyclés concentrés sur #876 ; déclaré au Project
Lead par l'orchestrateur). Worktree `/home/gcorbaz/devel/kesh-0172b`, branche `story/v0172-correctifs` (la même), partie de
`d7ec2ea2`. Clé courte : `v0172s` (port et bases de la story mère : 3173, `kesh_v0172b` / `kesh_e2e_v0172b`). Registre :
`v0172-choix-autonomes.md`, section `v0172s`, choix `C-v0172s-1` à `C-v0172s-14` (renumérotés par union depuis `C-v0172b-2`,
`3`, `4`, `5`, `18`, `23`, `24`, avec correspondance au registre). Issue : **#876** (bug, P2), en `refs` seulement. Numéros de
ligne de `d7ec2ea2`. Les passes P1 et P2 de validation ont été jouées sur la fiche mère (rapports
`kesh-gate-logs/v0172b-validate-p1.md`, `…-p2.md`) ; la prochaine passe de cette fiche est donc la **P3**.*

## Story

En tant que Comptable qui saisit une facture pendant vingt minutes, je ne veux pas être renvoyé à la connexion au moment
d'enregistrer ; en tant qu'utilisateur parti de son poste, je veux que la session se ferme après le temps d'inactivité
réglé ; en tant qu'utilisateur qui se déconnecte, je veux que la session soit réellement fermée.

## Contrainte

Celle de la fiche mère (§ *Contrainte*) : production réelle sur le NAS, aucun essai hors base jetable ; **aucune migration**
(88 migrations avant et après, `ls crates/kesh-db/migrations/*.sql | wc -l`) ; un **champ JSON additif** seulement ; retour à
la 0.17.1 sans restauration. Pas de P5, P6, P7.

**Modules** (règle de splitting) : (1) `crates/kesh-api/src/routes/auth.rs` et `routes/setup.rs` (routes d'authentification) ;
(2) `frontend/src/lib/shared/utils/api-client.ts` + module neuf `session-renewal.ts` ; (3) `frontend/src/lib/app/stores/auth.svelte.ts` ;
(4) les **trois** constructeurs du `LoginPayload` — `routes/login/+page.svelte:39-48` (type de la réponse et appel), 
`lib/features/setup/setup.api.ts:15, :53` (type `SetupAdminResponse` et appel), `api-client.ts:149` (succès de `doRefresh`,
depuis le corps de `/me`) — et `lib/features/admin-restore/AdminRestorePanel.svelte` (suspension après import, § 10) ;
(5) `frontend/vitest-setup.ts` ; manuels FR (utilisateur et administration). Cinq modules au sens de la règle (les
constructeurs du `LoginPayload` et l'appel de suspension sont des lignes d'appel du module de renouvellement).

## Ce que le code dit (d7ec2ea2)

La prémisse de l'issue (« à vérifier ») est **confirmée, et le défaut est plus net que l'issue ne le dit** : avec les
réglages par défaut, **le renouvellement sur 401 ne peut jamais réussir**, si bien qu'une session dure ≈ 15 minutes après la
connexion, **quelle que soit l'activité**.

1. **Deux durées de 15 minutes, comptées du même instant.** À la connexion, le jeton d'accès vaut `state.config.jwt_expiry`
   (`routes/auth.rs:244-250` ; défaut 15 min, `config.rs:688-708`) et le jeton de renouvellement reçoit `expires_at = now +
   refresh_inactivity` (`routes/auth.rs:254` ; défaut 15 min, `config.rs:735-755`). Les deux sont bornés **indépendamment**
   à [1, 1440] minutes. Même règle au renouvellement (`:422`), au changement de mot de passe (`:524`) et à l'assistant
   (`routes/setup.rs:288`). `.env.example:129, :160` et les deux `docker-compose*.yml` posent les mêmes 15/15.
2. **Le cookie d'accès disparaît à l'échéance du jeton.** `build_auth_cookies` (`routes/auth.rs:44-`) pose
   `kesh_access_token` avec `max_age = jwt_expiry` (`:54`) ; la requête suivante part sans jeton et reçoit 401 (le
   `leeway = 60` de `auth/jwt.rs:91` n'y change rien pour le cookie). Le cookie de renouvellement vit
   `refresh_token_max_lifetime` (30 jours, `:63-66`) : ce n'est **que** la durée du cookie ; la ligne en base expire à
   `now + refresh_inactivity`.
3. **Le frontend ne renouvelle qu'après ce 401.** `request` (`lib/shared/utils/api-client.ts:452-460`) et `requestRaw`
   (`:551-`) appellent `refreshTokens()` sur 401 ; aucun renouvellement préalable : `git grep -nE '\.expiresIn\b' --
   frontend/src ':!*.test.ts'` ne rend que des affectations et des validations de forme (`auth.svelte.ts:117, :245, :259`,
   `api-client.ts:115-170`, `setup.api.ts:53`, `login/+page.svelte:48`).
4. **Le renouvellement arrive trop tard.** `refresh` refuse un jeton dont `expires_at < now` (`routes/auth.rs:399`) ; le 401
   survient à `t₀ + jwt_expiry`, le jeton de renouvellement a expiré à `t₀ + refresh_inactivity` : avec 15 = 15, le `POST
   /auth/refresh` arrive **après** ⇒ `InvalidRefreshToken("token expired")` ⇒ `doRefresh` fait `clearSession()` puis
   `window.location.replace('/login?reason=session_expired')` (`api-client.ts:176-177`) — **la saisie en cours est perdue**.
5. **Le manuel promet le contraire** : `user-manual.tex:95` (« prolongée automatiquement à chaque action … Après 15 minutes sans
   activité ») ; `admin-manual.tex:111` (« compensée par rafraîchissement automatique côté frontend ») ; et le manuel
   d'administration se trompe ailleurs sur les cookies et les durées (§ *Manuel*, inventaire clos).
6. **Ce qui sonde le serveur sans l'utilisateur** : `BackupScheduleBanner.svelte:34` (`setInterval`) et
   `api-health.svelte.ts:102`. **Les requêtes ne sont pas une activité** (C-v0172s-2).
7. **Plusieurs onglets partagent les cookies**, et la rotation détecte le rejeu : un jeton déjà tourné présenté de nouveau
   (`revoked_reason = "rotation"`, `routes/auth.rs:379`) révoque **toutes** les sessions (`:380-389`). Le risque existe sur le
   chemin 401 ; le correctif le rend réel (C-v0172s-3).
8. **Tout échec de renouvellement efface la session** : `doRefresh` fait `clearSession()` + redirection sur erreur réseau
   (`api-client.ts:96-99`), sur réponse illisible (`:105-109`, `:115-118`) et sur **tout** statut non-OK (`:174-178`).
9. **La déconnexion ne passe par aucun verrou** : `logout()` (`auth.svelte.ts:166-187`) appelle `POST /api/v1/auth/logout`
   (`:169`) avec le cookie du moment ; le serveur révoque la seule ligne de ce cookie (`routes/auth.rs:306`,
   `refresh_tokens::revoke_by_token`, `WHERE token = ? AND revoked_at IS NULL`, `repositories/refresh_tokens.rs:115`). Un
   renouvellement en vol, commité avant, laisserait une ligne neuve **active** et ses cookies reposés (C-v0172s-10).
10. **Après un import complet, la session est invalidée mais l'écran reste** : `AdminRestorePanel.svelte` (rendu par
    `routes/(app)/admin/restore/+page.svelte:10` et `routes/onboarding/restore/+page.svelte:29`) garde le bilan (`imported`,
    `:37`, `:236-270` ; `unknownOutcome`, `:40`, `:272`) jusqu'au clic *Se reconnecter* (`relogin`, `:170-185`), et
    `admin-manual.tex:2322` le décrit (« n'est plus suivi d'une déconnexion immédiate … Prenez le temps de lire »). L'import
    remplace `refresh_tokens` : un renouvellement lancé sur ce bilan rendrait `401` (« unknown token », `routes/auth.rs:369-371`)
    et effacerait le bilan (C-v0172s-9).
11. **`hydrate()` pose `_currentUser = null` sans passer par `clearSession`** (`auth.svelte.ts:251`, `:265`, `:275`) ; et
    `/me` ne rend que des **restes** (`expires_in` calculé depuis `exp`, `routes/auth.rs:598`) : ni la durée réglée de
    l'inactivité ni celle du jeton d'accès ne sont connues après un rechargement.
12. **L'aperçu d'un document est un `<iframe>`** (`lib/features/imported-supplier-invoices/DocumentPreview.svelte:92`) : ses
    événements n'atteignent pas `window`.

## Conception retenue (C-v0172s-1 à 14)

**Le principe : la session se prolonge à l'entrée de l'utilisateur, au plus une fois par pas `T`, tant que le jeton de
renouvellement vit.** Pas de minuterie (C-v0172s-6).

- **Serveur** — champ additif `sessionExpiresIn` (secondes avant l'expiration du jeton de **renouvellement**) dans
  `LoginResponse` (`routes/auth.rs:135`, rendu par `login` et l'assistant `routes/setup.rs:308`), `RefreshResponse` (`:343`,
  `refresh` et `change_password`) — valeur `refresh_inactivity.num_seconds()` — et `MeResponse` (`:571`) — durée restante de
  la ligne du cookie `kesh_refresh_token` (chemin `/api/v1/auth`, `:63`), `max(0, expires_at − now)`, **`null`** si le cookie
  manque, est inconnu, révoqué ou expiré (`find_active_by_token`, `repositories/refresh_tokens.rs:79`). Aucune écriture
  (C-v0172s-5).
- **État du module** `lib/shared/utils/session-renewal.ts` : `lastRenewalAt` (horloge murale), `accessDeadline =
  armedAt + expiresIn`, `sessionDeadline = armedAt + sessionExpiresIn` (absent ⇒ `accessDeadline`), `lastInputAt`,
  `lastAttemptAt`. **Pas** `T = min(60 s, sessionExpiresIn/4, expiresIn/4)`, calculé **à chaque armement sur les valeurs
  reçues** : après `login` ou un renouvellement, ce sont les durées pleines (`N`, `jwt`) ; après `hydrate`, ce sont des
  **restes** (`/me`) — `T` est alors plus court, donc prudent, jusqu'au renouvellement suivant qui le recalcule (C-v0172s-6).
- **Armement et désarmement** : armé par `authState` après `login`, `hydrate` réussi et chaque renouvellement réussi (valeurs
  de la **réponse du refresh** ; le `/me` qui suit — `api-client.ts:149` — recale l'utilisateur et les restes) ; **désarmé à
  toute transition de `_currentUser` vers `null`** (`clearSession`, `logout`, les trois sites de `hydrate`, le message
  `auth-change` d'un autre onglet) ; **suspendu** par `AdminRestorePanel` dès l'envoi de l'import (C-v0172s-9) ; **inerte**
  dans Vitest tant qu'un test ne l'active pas (`vitest-setup.ts` → `disableSessionRenewalForTests()`).
- **Entrées** : écouteurs passifs `keydown`, `pointerdown`, `wheel`, `touchstart` sur `window`, et `visibilitychange` vers
  `visible`, posés à l'armement, retirés au désarmement (C-v0172s-2). **Les requêtes ne comptent pas** ; un geste dans
  l'aperçu `<iframe>` d'un document non plus (§ 12, déclaré au manuel, C-v0172s-12).
- **Règle à l'entrée** : `lastInputAt = now` ; si `now ≥ sessionDeadline`, **aucun** renouvellement (il serait refusé) ; sinon,
  si `now − max(lastRenewalAt, renewedAtPartagé) ≥ T` **et** `now − lastAttemptAt ≥ 10 s`, renouveler (sous le verrou).
- **Borne d'inactivité** (C-v0172s-8) — dernière entrée à `a`, dernier renouvellement à `r ∈ [a − T, a]` :
  - si `N ≥ jwt` : la session vit tant qu'une entrée peut renouveler, soit jusqu'à `r + N` ; elle prend fin **entre `N − T` et
    `N` après la dernière entrée** ;
  - si `N < jwt` : après `r + N` plus aucun renouvellement n'est possible, mais le jeton d'accès vit jusqu'à `r + jwt` et Kesh
    **ne coupe pas** la session avant (règle d'échec ci-dessous) ; elle prend fin **entre `jwt − T` et `jwt` après la dernière
    entrée renouvelante** — et, si l'utilisateur a laissé passer plus de `N` sans rien faire, à `r + jwt` même s'il travaille de
    nouveau entre-temps ;
  - en une formule : **entre `max(N, jwt) − T` et `max(N, jwt)`** après la dernière entrée qui a renouvelé ; avec les réglages
    par défaut, **entre 14 et 15 minutes** sans frappe ni clic.
- **Règle d'échec d'un renouvellement anticipé** (C-v0172s-7) : un `401 INVALID_REFRESH_TOKEN` n'efface la session **que si le
  jeton d'accès a expiré** (`now ≥ accessDeadline`) ; sinon le module se désarme pour ce jeton (`sessionDeadline = now`) et la
  session continue jusqu'à `accessDeadline` (la prochaine requête après cette heure rend 401, chemin ordinaire, et mène à la
  connexion) ; une erreur réseau, un 429 ou un 5xx **laissent la session** et notent `lastAttemptAt`.
- **Chemin 401 (requête rejetée)** : **module armé**, le renouvellement n'est tenté que si une entrée est **récente**
  (`now − lastInputAt < 10 s`, la requête en découle) ; sinon l'appel rejette `UNAUTHENTICATED`, sans renouvellement ni
  effacement. **Module non armé** (inerte dans Vitest, avant `hydrate`, après un `hydrate` en erreur) : comportement 0.17.1
  inchangé — c'est ce qui garde verts, **sans modification**, les six tests du chemin 401 de `api-client.test.ts` : `:205`
  (« refresh et retry sur 401 »), `:246` (« refresh échoué → clearSession + redirect login »), `:270` (« mutex de refresh »),
  `:327` (« refresh 200 sans expiresIn »), `:488` (« retry après refresh retourne 401 »), `:567` (« refresh fetch échoue
  (réseau) ») (C-v0172s-11).
- **Entre onglets** (C-v0172s-3) : `refreshTokens()` passe sous `navigator.locks.request('kesh:auth:refresh', …)` ; sous le
  verrou, si `localStorage['kesh:auth:renewedAt'] ≥` l'instant de la décision, un autre onglet a renouvelé : `GET /auth/me`
  au lieu du `POST` ; après succès, `renewedAt` écrit et message `{ type: 'auth-renewed' }` sur `authBroadcast` ; un onglet
  **désarmé** l'ignore. Sans `navigator.locks`, repli sur la comparaison de `renewedAt`.
- **Déconnexion** (C-v0172s-10) : `logout()` **désarme d'abord**, puis prend le **même verrou** `kesh:auth:refresh` — il attend
  donc un renouvellement en vol, de cet onglet ou d'un autre — et n'appelle `POST /auth/logout` qu'ensuite, avec le cookie
  courant ; la réponse d'un renouvellement arrivée après le désarmement ne réarme rien.
- **Après un import complet** (C-v0172s-9) : `AdminRestorePanel` appelle `sessionRenewal.suspend()` au moment où l'import
  part (`:156`) ; suspendu, le module n'écoute plus et le chemin 401 ne renouvelle pas ; *Se reconnecter* (`relogin`) reste le
  seul geste. Un import en échec (`imported` nul et `unknownOutcome` faux) **reprend** le module.
- **Écartées** (C-v0172s-1, 6, 8) : (a) une minuterie (première rédaction de la fiche mère : borne jusqu'à `2N − 1`, muette en
  veille) ; (b) relâcher l'expiration au serveur ; (c) prolonger à chaque requête ; (d) ne corriger que le manuel ; (e) pour
  `N < jwt`, effacer la session au premier refus de renouvellement — c'est couper un utilisateur dont le jeton d'accès vaut
  encore des minutes, ce que la 0.17.1 ne faisait pas (finding P1 de la validation P2) ; (f) pour `N < jwt`, couper la session
  à `r + N` côté navigateur — même effet.
- **Coûts déclarés** (C-v0172s-13) : au plus un `POST /auth/refresh` par pas `T` d'activité (une minute par défaut) et par
  utilisateur ; chacun ajoute une ligne à `refresh_tokens`, que seul le démarrage purge (au-delà de sept jours,
  `main.rs:230-233`, `delete_expired_and_revoked`) — environ 60 lignes par heure d'activité et par utilisateur entre deux
  redémarrages ; la table n'est **pas** dans l'export global (`exports/global.rs:215-217`, exclue comme secret), elle est dans
  le `.keshbackup` (`admin-manual.tex:1662`, « jetons de session »).
- **Changement de comportement déclaré** (C-v0172s-11) : avec 15/15, une requête de fond qui reçoit 401 **sans entrée récente**
  ne mène plus à `/login` : l'écran d'un utilisateur parti reste affiché jusqu'à sa première action, qui mène alors à la
  connexion (sa requête rend 401 avec une entrée récente ; le renouvellement échoue ; la session est effacée). Les données
  affichées restent celles qui l'étaient ; aucune requête neuve n'aboutit.

## Acceptance Criteria

**AC-1 — Le champ `sessionExpiresIn`.** `POST /api/v1/auth/login`, `POST /api/v1/auth/refresh`, `PUT /api/v1/auth/password` et
la fin de l'assistant rendent `sessionExpiresIn == refresh_inactivity.num_seconds()` (900 par défaut) ; `GET /api/v1/auth/me`
rend la durée restante du jeton du cookie (≤ 900 juste après la connexion ; ≤ 600 après avoir reculé `expires_at` de 300 s en
base), `null` sans cookie (en-tête `Bearer` seul), avec un cookie révoqué ou expiré. `expiresIn` inchangé. Fichier de tests :
celui des E2E d'authentification existant, nommé au Change Log après `git grep -ln 'auth/me' crates/kesh-api/tests`.

**AC-2 — La sémantique du serveur, figée.** Un jeton de renouvellement dont `expires_at` est passé est refusé (`401`,
`INVALID_REFRESH_TOKEN`) ; le test reste vert.

**AC-3 — Le renouvellement à l'entrée.** `session-renewal.test.ts` (Vitest ; horloge `vi.useFakeTimers({ toFake: ['Date'] })`
puis `vi.setSystemTime`, comme `bank-accounts-page.test.ts:702-718` ; aucun cas ne dépend d'une minuterie) — sauf mention,
`expiresIn = sessionExpiresIn = 900`, `T = 60 s` :
(a) une frappe toutes les 30 s pendant 40 minutes : `POST /api/v1/auth/refresh` au plus une fois par 60 s et au moins une fois
par 120 s ; aucune requête rejetée ;
(b) **une entrée puis plus rien** — (b1) renouvellement à t = 0, frappe à 0,5 min (pas de `POST`), puis une frappe à
15 min 01 s : **aucun** `POST` ; (b2) renouvellement à 0, frappe à 1 min ⇒ `POST` ; frappe à 15 min 30 s ⇒ `POST` ; variante sans
elle : frappe à 16 min 01 s ⇒ aucun `POST` ;
(c) une requête de fond chaque minute pendant 20 minutes, sans entrée : aucun `POST` ;
(d) `sessionExpiresIn = 120`, `expiresIn = 900` : `T = 30 s` ; une frappe toutes les 20 s renouvelle au moins toutes les 60 s ;
(e) `sessionExpiresIn` absent : `T` et les échéances sur `expiresIn` seul ;
(f) **veille** : l'horloge avance de 13 minutes d'un coup, puis une frappe ⇒ `POST` ; elle avance de 16 minutes, puis une frappe ⇒
aucun `POST` ;
(g) **retour sur l'onglet** : `visibilitychange` vers `visible` après 5 minutes sans entrée ⇒ `POST` ; vers `hidden` ⇒ rien ;
(h) **échec transitoire** : le `POST` anticipé rend `503` : session conservée, aucune redirection ; une frappe 2 s plus tard ⇒
**aucun** `POST` ; une frappe 10 s après l'échec ⇒ `POST`, qui réussit ; une erreur réseau : même chose ;
(i) **`N < jwt`** : `expiresIn = 900`, `sessionExpiresIn = 60` (`T = 15 s`) ; frappe à 2 min (la ligne a expiré à 1 min) :
**aucun** `POST` (garde `sessionDeadline`) ; `authState.isAuthenticated` reste vrai jusqu'à 15 min ; variante où le serveur rend
`401 INVALID_REFRESH_TOKEN` à un `POST` lancé à 50 s (horloges décalées) : session **conservée**, aucune redirection ; un `401
INVALID_REFRESH_TOKEN` à un `POST` lancé **après** `accessDeadline` ⇒ `clearSession()` et redirection `/login?reason=session_expired` ;
(j) **401 sans entrée récente** : `sessionExpiresIn = 3600`, `expiresIn = 900` ; une frappe à 30 s (pas de `POST`), puis rien ; à
16 min une requête de fond reçoit 401 ⇒ **aucun** `POST`, l'appel rejette `UNAUTHENTICATED`, la session n'est pas effacée ; une
frappe à 16 min 10 s, puis la requête d'un clic rendue 401 ⇒ un seul `POST`, la requête rejouée ;
(k) **désarmement** : après `clearSession()`, après un `hydrate()` qui rend 401, et après le message `auth-change` d'un autre
onglet, une frappe ne déclenche aucun `POST`, et un `auth-renewed` reçu est ignoré.

**AC-4 — Un seul renouvellement pour plusieurs onglets.** (a) Deux instances partageant `localStorage` et `navigator.locks`
simulés, une frappe dans chacune (horloge avancée d'au moins 1 ms entre les deux décisions, comparaison `≥`) : **un seul**
`POST` ; l'autre fait `GET /auth/me` ; (b) `auth-renewed` recale sans `POST` et compte pour `max(lastRenewalAt, renewedAt)` ;
(c) deux 401 simultanés dans deux onglets armés, entrée récente dans les deux : un seul `POST`, les deux requêtes rejouées une
fois ; (d) sans `navigator.locks` : (a) tient quand les deux décisions sont séquentielles (course résiduelle déclarée au Dev
Agent Record). Les six tests du chemin 401 de `api-client.test.ts` (§ *Conception*) restent verts **sans modification**.

**AC-5 — La déconnexion attend le renouvellement en vol.** Vitest : une frappe lance un `POST /auth/refresh` dont la réponse est
retenue ; `logout()` est appelé : `POST /auth/logout` ne part **qu'après** la réponse du refresh (ordre des appels lu au mock) ;
la réponse du refresh ne réarme pas le module (une frappe ensuite : aucun `POST`). Variante deux onglets : le renouvellement est
en vol dans l'onglet B, `logout()` dans l'onglet A attend le verrou.

**AC-6 — Le bilan d'un import complet reste lisible.** `AdminRestorePanel.test.ts` : module activé et armé ; l'import part
(`uploadFullImport` simulé) et réussit ; une frappe et une molette sur le bilan, deux minutes plus tard : **aucun** `POST
/auth/refresh`, aucune redirection, le bilan (`filesRestored`, `filesMissing`) reste affiché ; *Se reconnecter* mène à `/login`
(comportement existant). Même chose pour `unknownOutcome`. Un import refusé (erreur 4xx) reprend le module (une frappe ⇒
renouvellement si dû).

**AC-7 — Recette sur une installation de dev.** Sur `kesh_v0172b`, backend `KESH_JWT_EXPIRY_MINUTES=2`,
`KESH_REFRESH_INACTIVITY_MINUTES=2` : (a) taper dans un brouillon 6 minutes sans enregistrer, puis enregistrer : réussite ;
(b) une frappe puis 2 min 30 s sans rien, puis un clic : retour à la connexion ; une frappe puis 1 min 20 s, puis un clic : la
session vit ; (c) deux onglets actifs 6 minutes : aucune déconnexion, aucune ligne `token replay detected` au journal ;
(d) veille du poste (ou onglet masqué) 1 minute pendant une saisie, puis une frappe et *Enregistrer* : réussite ; (e) se
déconnecter en pleine saisie : `refresh_tokens` de l'utilisateur ne porte **aucune** ligne active ensuite (requête en lecture sur
`kesh_v0172b`). Déroulé au Dev Agent Record. Pas d'E2E (C-v0172s-4).

**AC-8 — Les manuels disent vrai.** Les sites du § *Manuel* sont corrigés, la borne écrite étant celle de la conception.

## Tests et mutations

| # | Test | Attendu avant correctif |
|---|---|---|
| T1 | `sessionExpiresIn` aux cinq réponses (AC-1) | rouge |
| T2 | refus d'un jeton expiré (AC-2) | vert |
| T3 | `session-renewal.test.ts` a–k (AC-3), `AC-4`, `AC-5` | **sans objet** : le module n'existe pas, le fichier ne se charge pas |
| T4 | les six tests du chemin 401 d'`api-client.test.ts` | verts avant et après, inchangés |
| T5 | `AdminRestorePanel.test.ts`, bilan lisible (AC-6) | rouge sur la seule assertion « aucun `POST` » une fois le module branché ; sans objet avant |

**Mutations nommées (P2)** — sous le verrou de gate ; après une restauration par `cp`/`mv`, toucher le fichier :
- **M1** — l'entrée n'appelle plus `refreshTokens()` : AC-3 (a), (f), (g) rougissent.
- **M2** — le pas `T` retiré : AC-3 (a) rougit.
- **M3** — les requêtes comptées comme entrée : AC-3 (c), (j) rougissent.
- **M4** — la comparaison `renewedAt` retirée : AC-4 (a), (c) rougissent.
- **M5** — `/me` rend `refresh_inactivity` au lieu du reste : AC-1 (`/me` ≤ 600) rougit.
- **M6** — la garde `now ≥ sessionDeadline` retirée : AC-3 (b1), (f) 2ᵉ partie, (i) 1ʳᵉ partie rougissent.
- **M7** — un 503 du renouvellement anticipé efface la session : AC-3 (h) rougit.
- **M8** — le délai de 10 s après un échec retiré : AC-3 (h) (« frappe 2 s plus tard ⇒ aucun `POST` ») rougit.
- **M9** — l'écouteur `visibilitychange` retiré : AC-3 (g) rougit.
- **M10** — la garde « entrée récente » du chemin 401 retirée : AC-3 (j) 1ʳᵉ partie rougit.
- **M11** — un `401 INVALID_REFRESH_TOKEN` efface la session même quand le jeton d'accès vit : AC-3 (i) variante rougit.
- **M12** — le désarmement retiré d'un site (`clearSession`, puis la branche 401 d'`hydrate`) : AC-3 (k) rougit.
- **M13** — `logout()` n'attend plus le verrou : AC-5 rougit.
- **M14** — `suspend()` retiré d'`AdminRestorePanel` : AC-6 rougit.
- **M15** — la garde « module non armé ⇒ comportement 0.17.1 » retirée du chemin 401 : T4 (`:205`) rougit.

## Tâches

1. T1 rouge sur `d7ec2ea2` ; `sessionExpiresIn` (serveur, cinq réponses).
2. `session-renewal.ts` et son câblage : `auth.svelte.ts` (armement ; désarmement à toute transition vers `null`), `api-client.ts`
   (`refreshTokens` sous le verrou, chemin anticipé distinct, garde 401 « entrée récente » module armé seulement), les trois
   constructeurs du `LoginPayload`, `logout()` sous le verrou, `AdminRestorePanel` (`suspend` / reprise),
   `disableSessionRenewalForTests()` dans `vitest-setup.ts` ; tests T3, T5 ; mutations M1 à M15.
3. AC-7 (recette) ; manuels et PDF (§ *Manuel*).
4. Gates : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh` (sous
   `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` et `mem-guard`), `cd frontend && npm run check && npm run
   lint-i18n-ownership && npm run test:unit && npm run build`, E2E complet au **dernier commit de code** (7 à 10 échecs
   attendus) ; base remise à zéro avant le gate complet. Aucun repository ni migration touché : gate ciblé permis en boucle.
5. Pas de contrôle visible neuf : pas de capture exigée (aucun texte visible nouveau, hors manuels).

## Manuel

PDF régénéré par `make fr` dans `docs/manual/`, contrôlé aplati (`pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '`,
puis `grep -F` de chaque phrase neuve **et** de chaque phrase retirée).

**`docs/manual/fr/user-manual.tex`** — `:95` récrit : la session se prolonge **tant que vous travaillez dans Kesh** (une frappe, un
clic, le retour sur l'onglet) ; elle prend fin **entre 14 et 15 minutes après votre dernière frappe ou votre dernier clic** (durée
réglable par l'administrateur) ; une saisie longue, même sans enregistrer, la garde ouverte ; **lire un document dans l'aperçu
d'une pièce importée ne compte pas** (faites un clic dans Kesh de temps à autre) ; une mise en veille du poste plus longue que
cette durée ferme la session ; plusieurs onglets partagent la même session ; la déconnexion ferme la session de tous les onglets.

**`docs/manual/fr/admin-manual.tex` — inventaire clos** (relevé par `grep -nE 'refresh|cookie|JWT|jeton d.accès|access token|KESH_JWT|INACTIVITY|SameSite|session'
docs/manual/fr/admin-manual.tex | grep -viE mcp`, 64 lignes lues une à une) : 53 hors classe — le secret JWT `:243, :245, :252,
:253, :577, :579, :599, :732, :1744, :1749, :1752, :2414, :2766, :2770, :2772, :2783, :2789, :2987, :3691`, le multi-tenant `:112,
:1500`, `KESH_COOKIE_SECURE` `:784, :1391, :1403, :1418, :1421, :1427, :1432` (exact), les sessions révoquées `:1192, :1361`, la
sauvegarde `:1662`, les sessions SSH/root `:2027, :2130, :2164`, les lignes sans rapport `:1655, :1665, :1988, :1991, :2050,
:2091, :2292, :2360, :2793, :2925, :3003, :3040, :3269, :3341, :3357, :3618`, les titres `:772, :2990, :2992` — et 11 au tableau :

| Site | Ce qu'il dit | Verdict |
|---|---|---|
| `:111` | « durée d'expiration courte (15 min) compensée par rafraîchissement automatique côté frontend » | faux jusqu'ici ; préciser « renouvelée à l'activité de l'utilisateur » |
| `:777` | `KESH_JWT_EXPIRY_MINUTES` « Durée access token » | exact ; ajouter « (et du cookie d'accès) » |
| `:778` | `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` « Durée absolue refresh token » | inexact : durée de vie du **cookie** (`routes/auth.rs:63-66`) |
| `:779` | `KESH_REFRESH_INACTIVITY_MINUTES` « Inactivité avant expiration refresh (sliding) » | à récrire : temps sans frappe ni clic avant la fin de la session ; la session prend fin entre `max(inactivité, durée du jeton d'accès) − 1 min` et `max(…)` après la dernière entrée |
| `:2322` | « Un import réussi n'est plus suivi d'une déconnexion immédiate … Prenez le temps de lire » | **reste vrai** grâce à la suspension (§ 10, AC-6) — à relire au PDF après correctif ; classé « sessions révoquées » par la première rédaction, reclassé ici (finding P4 de la validation P2) |
| `:2506` | « au plus la durée de session plus une minute » | à préciser : durée du **jeton d'accès** (`KESH_JWT_EXPIRY_MINUTES`) plus une minute |
| `:2995` | cookie d'accès « `HttpOnly; Secure; SameSite=Lax` » | faux : `SameSite=Strict` (`routes/auth.rs:52, :62`) |
| `:2996` | « refresh token (durée 30 jours par défaut) … hash SHA-256 (jamais stocké en clair) » | deux fois faux : 30 jours est la durée du cookie ; le jeton est stocké **tel quel** (`repositories/refresh_tokens.rs:30-34`) — texte corrigé ; **issue demandée** pour le stockage (C-v0172s-14) |
| `:2997` | rotation et trace | exact (`:379`) |
| `:2998` | « La déconnexion révoque tous les refresh tokens de la session » | à relire contre `revoke_by_token` (une ligne) : écrire « révoque le jeton de la session » si c'est le cas |
| `:3886` | glossaire « Refresh token — Token longue durée » | inexact : jeton qui renouvelle le jeton d'accès, valable l'inactivité réglée après chaque renouvellement |

## CHANGELOG

Dans la section commune `## [0.17.2] — Non publié`, sous `### Corrigé` :
- **La session ne se ferme plus pendant que vous travaillez** ([#876](https://github.com/guycorbaz/kesh/issues/876)) : elle n'était
  jamais prolongée — avec les réglages par défaut, elle prenait fin quinze minutes après la connexion, même en pleine saisie, et la
  saisie en cours était perdue. Elle se prolonge désormais tant que vous tapez ou cliquez, et se ferme entre 14 et 15 minutes après
  votre dernière frappe ou votre dernier clic.

`refs #876` aux commits ; aucun mot-clé de fermeture.

## Dev Notes

- **Pourquoi aucun test ne l'a vu** : les E2E durent moins de quinze minutes par spec ; le test du renouvellement
  (`api-client.test.ts:205-`) simule un refresh **qui réussit**.
- **Course résiduelle sans `navigator.locks`** : déclarée au Dev Agent Record, non résolue (Chromium, Firefox, Safari ≥ 15.4
  portent l'API).
- **Rechargement après expiration du jeton d'accès** : `hydrate()` garde son comportement (`auth.svelte.ts:261-`) ; avec 15/15 la
  session est alors morte.
- **Jamais de donnée réelle** ; recette sur `kesh_v0172b` seulement.

## Change Log

- 2026-11-04 — **fiche extraite** de `v0172-correctifs.md` (sous-story A1) à la remédiation de la **validation P2 (Opus) :
  0/0/11/12** de la fiche mère (rapport `kesh-gate-logs/v0172b-validate-p2.md`), sur signal de découpage (sévérité stagnante,
  défauts recyclés concentrés sur #876). Findings de la P2 qui concernent cette fiche, prémisse relue au code :
  **P1** (MEDIUM) appliqué — déroulé `N = 1`, `jwt = 15` refait : la garde « morte » porte désormais sur `sessionDeadline`, et un
  `401 INVALID_REFRESH_TOKEN` n'efface la session que jeton d'accès expiré ; la borne `max(N, jwt)` devient vraie et est écrite
  par cas (C-v0172s-8) ; AC-3 (i). **P2** (MEDIUM) appliqué — chemin 401 : entrée **récente** (< 10 s) exigée, module armé ;
  AC-3 (j) avec la frappe à 30 s. **P3** (MEDIUM) appliqué — lignes lues `api-client.test.ts:205, :246, :270, :327, :488, :567`
  (six tests 401 sans entrée) : la garde ne vaut que module armé ; les six tests restent verts inchangés (T4, M15). **P4**
  (MEDIUM) appliqué — `AdminRestorePanel.svelte:10, :37, :40, :156, :170-185` lus : suspension dès l'envoi, AC-6, `:2322` reclassé.
  **P5** (MEDIUM) appliqué — `auth.svelte.ts:169`, `routes/auth.rs:306`, `refresh_tokens.rs:115` lus : `logout()` désarme puis
  attend le verrou, AC-5, M13. LOW : **P12** appliqué (changement de comportement à 15/15 déclaré) ; **P13** appliqué (`T` sur les
  valeurs reçues, désarmement à toute transition vers `null`, `auth.svelte.ts:251, :265, :275`) ; **P14** appliqué (troisième
  constructeur `api-client.ts:149`, types `setup.api.ts:15`, `login/+page.svelte:39`) ; **P15** appliqué (T3 « sans objet avant »,
  frappe à +2 s, M8) ; **P16** **en partie réfuté** — la table `refresh_tokens` n'est **pas** dans l'export global
  (`exports/global.rs:215-217`, elle y est exclue comme secret) ; elle est dans le `.keshbackup` ; la croissance, elle, est déclarée
  (`main.rs:230-233`) ; **P17** appliqué (aperçu `<iframe>`, `DocumentPreview.svelte:92`, déclaré au manuel) ; **P21** pour la
  partie de cette fiche (C-v0172b-5, devenu C-v0172s-4 : horloge simulée, non minuteries). Valeurs grepées : `max(N, jwt)`,
  `minuterie`, `C-v0172b-2`, `C-v0172b-23`, `C-v0172b-24`, `AC-A` dans cette fiche. Prochaine : validation **P3** (Sonnet, complète,
  `v0172-session-validate-prompt-p3.md`).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
