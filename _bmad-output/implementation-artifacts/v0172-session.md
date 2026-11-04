# Story v0172-session : la 0.17.2 — la session expire pendant une activité continue (#876)

Status: ready-for-dev

*Fiche sœur de `v0172-correctifs.md`, extraite le 2026-11-04 de sa sous-story A1 à la remédiation de la validation P2 (signal de
découpage : sévérité stagnante, défauts recyclés concentrés sur #876). **Conception remplacée à la remédiation de la validation
P3** (Sonnet, 0/0/7/10) : le renouvellement anticipé à l'entrée de l'utilisateur, ses verrous entre onglets, son module « armé »
et sa suspension pendant l'import sont **abandonnés** au profit d'une conception plus simple — **la fenêtre d'inactivité du jeton
de renouvellement plus longue que la vie du jeton d'accès**, qui laisse fonctionner le chemin existant (401, renouvellement, rejeu)
— C-v0172s-15. Worktree `/home/gcorbaz/devel/kesh-0172b`, branche `story/v0172-correctifs`, partie de `d7ec2ea2`. Clé courte :
`v0172s` (port et bases de la story mère : 3173, `kesh_v0172b` / `kesh_e2e_v0172b`). Registre : `v0172-choix-autonomes.md`, section
`v0172s` ; **en vigueur : C-v0172s-14 à C-v0172s-21** (C-v0172s-1 à 13 : conception écartée, historique). Issue : **#876** (bug, P2),
en `refs` seulement. Numéros de ligne de `d7ec2ea2`. La prochaine passe de validation est la **P4**, **complète** (la conception
change).*

## Story

En tant que Comptable qui saisit une facture pendant vingt minutes, je ne veux pas être renvoyé à la connexion au moment
d'enregistrer ; en tant qu'utilisateur parti de son poste, je veux que la session se ferme après un temps d'inactivité connu.

## Contrainte

Celle de la fiche mère : production réelle sur le NAS, aucun essai hors base jetable ; **aucune migration** (88 migrations avant
et après, `ls crates/kesh-db/migrations/*.sql | wc -l`) ; aucune donnée réécrite ; retour à la 0.17.1 sans restauration (la 0.17.1
relit la même configuration, et les jetons ont le même format). Pas de P5, P6, P7.

**Modules** (règle de splitting) : (1) `crates/kesh-api` — `config.rs` (défaut et règle de l'inactivité) et `routes/auth.rs`
(grâce de rotation) ; (2) `frontend/src/lib/shared/utils/api-client.ts` (repli après un refus de renouvellement, requêtes de fond) ;
(3) `frontend/src/lib/app/stores/auth.svelte.ts` (renouvellement silencieux au démarrage) ; (4) `frontend/src/lib/features/admin-backup/`
(`backup-schedule.svelte.ts` / `backup-schedule.api.ts` : la lecture périodique marquée « de fond ») ; (5) la configuration livrée
(`.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`) et les manuels FR. **Cinq** — la dérogation que demandait la
conception précédente (huit modules, finding P7 de la validation P3) n'a plus d'objet.

## Ce que le code dit (d7ec2ea2)

1. **Deux durées de 15 minutes, comptées du même instant.** Le jeton d'accès vaut `jwt_expiry` (`routes/auth.rs:244-250` ; défaut
   15 min, `config.rs:688-708`) ; le jeton de renouvellement reçoit `expires_at = now + refresh_inactivity` (`routes/auth.rs:254` ;
   défaut 15 min, `config.rs:735-755`, repli 15 hors bornes et sur valeur invalide, `:739-751`). Même règle au renouvellement (`:422`),
   au changement de mot de passe (`:524`) et à l'assistant (`routes/setup.rs:288`). La configuration livrée pose les mêmes 15/15 :
   `.env.example:129, :160` (commentaires `:128`, `:130`, `:159`), `docker-compose.yml:76, :85`, `docker-compose.prod.yml:90, :94`
   (`${KESH_REFRESH_INACTIVITY_MINUTES:-15}`).
2. **Le cookie d'accès disparaît à l'échéance** (`build_auth_cookies`, `max_age = jwt_expiry`, `routes/auth.rs:54`) : la requête
   suivante part sans jeton et reçoit 401 du middleware (`middleware/auth.rs:73-`), avant tout traitement. Le cookie de
   renouvellement vit `refresh_token_max_lifetime` (30 jours, `:63-66`), reposé à chaque renouvellement.
3. **Le chemin 401 existe et se rejoue** : `request` (`lib/shared/utils/api-client.ts:452-460`) et `requestRaw` (`:551-`) appellent
   `refreshTokens()` (mutex d'onglet, `refreshPromise`, `:185-192` : deux requêtes rejetées dans un onglet ne font qu'un
   renouvellement), puis **rejouent** la requête une fois (`isRetry`, `:458-461`).
4. **Le renouvellement glisse** : chaque `POST /auth/refresh` réussi révoque l'ancienne ligne (`:417`, `revoked_reason = "rotation"`)
   et en crée une neuve à `now + refresh_inactivity` (`:421-432`) — l'inactivité se compte depuis le **dernier renouvellement**.
5. **Le défaut** : avec 15 = 15, le 401 survient à `t₀ + 15` et la ligne a expiré à `t₀ + 15` (+ quelques ms) ; `refresh` la refuse
   (`expires_at < now`, `:399`) ⇒ `doRefresh` efface la session et redirige (`api-client.ts:174-178`) — saisie perdue. Le chemin 401
   est juste ; ce sont les **durées** qui le condamnent : il réussit dès que `refresh_inactivity > jwt_expiry`.
6. **Le rejeu ne perd pas la saisie, et un `POST` rejoué est sûr** : le 401 du jeton d'accès vient du middleware, **avant** le handler
   (§ 2) : rien n'a été écrit, la requête est rejouée à l'identique (`request<T>(url, options, true, call)`, `:460`), et l'état du
   formulaire, côté navigateur, n'a pas bougé. *Limite* : un handler qui rendrait lui-même 401 hors des routes exclues
   (`AUTH_EXCLUDED_URLS`, `:28`) — `PUT /api/v1/auth/password` sur mot de passe courant faux (`InvalidCredentials`) — déclenche aussi
   un renouvellement puis un rejeu refusé : comportement 0.17.1, inchangé, sans écriture (le refus précède `update_password`). Un corps
   non rejouable (flux) n'est employé, à la connaissance de la fiche, par aucun appel : le développeur le vérifie (`git grep -nE
   'ReadableStream|duplex' -- frontend/src`).
7. **Deux onglets aujourd'hui** : ils partagent les cookies. S'ils reçoivent 401 dans le même intervalle (deux requêtes parties avant
   que la réponse du premier renouvellement n'ait reposé les cookies), le second présente un jeton déjà tourné ⇒
   `revoked_reason == "rotation"` (`:379`) ⇒ `revoke_all_for_user(…, "theft_detected")` (`:380-389`) : **toutes** les sessions de
   l'utilisateur sont révoquées. Ce cas ne se produit pas en 0.17.1 (le renouvellement échoue toujours avant) ; il deviendrait réel
   avec la correction des durées (C-v0172s-17).
8. **Le démarrage d'un onglet ne renouvelle pas** : `hydrate()` lit `GET /auth/me` par `fetch` direct (`auth.svelte.ts:229`) ; sur 401,
   il pose `_currentUser = null` (`:261-265`) sans tenter de renouvellement — un rechargement après l'échéance du jeton d'accès, jeton de
   renouvellement encore valable, renvoie à la connexion.
9. **Une seule lecture périodique authentifiée** : `BackupScheduleBanner.svelte:27, :34` (`REFRESH_MS = 30 min`, `setInterval` →
   `refreshBackupScheduleState`, `backup-schedule.svelte.ts:49`, `fetchBackupSchedule` sur `GET /api/v1/admin/backup-schedule`).
   L'autre minuterie, `api-health.svelte.ts:102`, interroge `/health`, non authentifié. `git grep -nE 'setInterval\(' -- frontend/src
   ':!*.test.ts'` ne rend que ces deux lignes. Une lecture de fond qui, sur 401, renouvellerait la session la garderait ouverte
   indéfiniment sur un écran d'administration abandonné.

## Conception retenue (C-v0172s-14 à 21)

**Le principe : la fenêtre d'inactivité du jeton de renouvellement dépasse la vie du jeton d'accès ; le chemin 401 existant fait le
reste.**

1. **Le défaut de configuration** (C-v0172s-16) : `KESH_REFRESH_INACTIVITY_MINUTES` passe de 15 à **60** par défaut (`config.rs:735-755`,
   y compris les deux replis « hors borne » et « invalide »), dans `.env.example:160`, `docker-compose.yml:85` et
   `docker-compose.prod.yml:94` (`:-60`). `KESH_JWT_EXPIRY_MINUTES` reste à 15.
2. **La règle** `refresh_inactivity ≥ 2 × jwt_expiry` (C-v0172s-16) : si la valeur lue est plus petite, Kesh **démarre quand même**,
   écrit un avertissement (`KESH_REFRESH_INACTIVITY_MINUTES=15 inférieur au double de KESH_JWT_EXPIRY_MINUTES=15 : 30 minutes
   appliquées`) et applique `2 × jwt_expiry`. Refuser le démarrage casserait une installation dont le `.env` a été recopié du gabarit
   0.17.1 (`KESH_REFRESH_INACTIVITY_MINUTES=15`, `.env.example:160`). La borne haute existante (1440) ne s'applique pas à la valeur
   portée au double (un `jwt` de 1440 donne 2880, dit dans l'avertissement). `from_fields_for_test` (`config.rs:1633-1635`) passe aussi
   à 60 minutes d'inactivité.
3. **La grâce de rotation** (C-v0172s-17) : dans `refresh`, un jeton révoqué pour `"rotation"` **depuis moins de 30 secondes**
   (`token.revoked_at`, écrit par `NOW(3)`, `repositories/refresh_tokens.rs:111-116`) rend `401 INVALID_REFRESH_TOKEN` (« token
   recently rotated ») **sans** `revoke_all_for_user` ; au-delà, détection de vol inchangée. Aucun jeton n'est émis dans ce cas.
4. **Le repli du navigateur après un refus** (C-v0172s-18) : dans `doRefresh`, un `401` au `POST /auth/refresh` est suivi d'**un**
   `GET /auth/me` ; s'il rend 200, un autre onglet a renouvelé et reposé les cookies : le renouvellement est tenu pour réussi
   (`authState.login(…, { broadcast: false })` comme au succès, `:149-160`), la requête d'origine est rejouée ; sinon, effacement et
   redirection, comme aujourd'hui. Aucun verrou, aucun `localStorage`.
5. **Le démarrage d'un onglet** (C-v0172s-19) : `hydrate()`, sur un 401 de `/me`, tente **un** renouvellement **silencieux** (`POST
   /auth/refresh` ; succès ⇒ second `GET /auth/me` ; échec ⇒ état nul, **sans** redirection — la page de connexion appelle aussi
   `hydrate()`, une redirection y bouclerait).
6. **Les lectures de fond** (C-v0172s-20) : `CallOptions` (`api-client.ts:59-64`) reçoit `background?: boolean` ; une requête de fond
   qui reçoit 401 **ne renouvelle pas** : elle rejette `UNAUTHENTICATED`, sans effacement ni redirection. La lecture périodique du
   bandeau de sauvegarde (appel de la minuterie seulement — la première lecture, au montage, reste ordinaire) est marquée
   `background: true` ; `refreshBackupScheduleState` garde son état précédent sur erreur (`:53-57`).
7. **La borne d'inactivité qui en résulte** (C-v0172s-21) : appelons *requête* toute requête API de l'utilisateur (non de fond). La
   dernière à `a` ; le dernier renouvellement à `r ≤ a`. Une requête faite pendant que le jeton d'accès vit (`a < r + jwt`) ne prolonge
   rien ; une requête après (`a ≥ r + jwt`) renouvelle. La session prend donc fin **entre `N − jwt` et `N` après la dernière requête**
   (`N` = inactivité appliquée) : avec les nouveaux défauts (60/15), **entre 45 et 60 minutes** ; avec un `.env` 0.17.1 à 15/15, porté à
   30, **entre 15 et 30 minutes**. Une saisie sans requête (un long formulaire) est sûre tant qu'elle dure moins de `N − jwt`.
   **Aucune durée absolue** ne borne la session d'un utilisateur actif : chaque renouvellement crée une ligne neuve et repose un cookie
   de `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` (finding P13 de la validation P3) — comme en 0.17.1 pour qui avait `N > jwt` ; déclaré au
   manuel.
8. **Écarté** (C-v0172s-15) : la conception des validations P1 à P3 — renouvellement anticipé à chaque entrée de l'utilisateur, pas `T`,
   verrou `navigator.locks` (absent en HTTP-LAN, contexte non sécurisé : finding P3), `renewedAt` partagé, module « armé » et « inerte »
   sous Vitest (finding P1 : un fichier d'amorçage qui importe le module défait les `vi.mock`), suspension pendant l'import complet
   (finding P5), déconnexion sous verrou (finding P4). Elle donnait une borne plus serrée (« 14 à 15 minutes après la dernière frappe »),
   mais trois passes de suite y ont trouvé des MEDIUM nés de sa mécanique, et elle comptait huit modules. La conception retenue renonce
   à compter les frappes : l'inactivité se mesure aux requêtes, avec une marge `jwt`.

## Acceptance Criteria

**AC-1 — Le défaut et la règle de configuration.** Tests unitaires de `config.rs` : sans variable, `refresh_inactivity == 60 min` ; avec
`KESH_REFRESH_INACTIVITY_MINUTES=90`, 90 ; avec `=15` et `KESH_JWT_EXPIRY_MINUTES=15`, **30** et un avertissement ; avec `=5` et jwt 2,
5 (≥ 4) ; hors borne (`9999`) ou invalide, 60 (puis la règle). **Tests existants à mettre à jour**, raison au Change Log :
`config_refresh_inactivity_defaults_to_15_min` (`config.rs:2689-2696`, 15 ⇒ 60, et son nom), `config_refresh_inactivity_respects_env`
(`:2698-2708`, valeur 5 avec jwt 15 : poser jwt à 2, ou attendre 30), `config_refresh_inactivity_out_of_bounds_falls_back`
(`:2711-2721`, 15 ⇒ 60). Le développeur greppe les autres dépendances au défaut (`git grep -nE 'refresh_inactivity|minutes\(15\)' --
crates/kesh-api`) et les nomme au Change Log. **Rouges avant correctif** : défaut 60 et règle du double.

**AC-2 — Le renouvellement réussit après l'échéance du jeton d'accès.** Test d'intégration dans `crates/kesh-api/tests/auth_e2e.rs`
(le fichier qui porte les réponses de `login`, `refresh` et `password`, p. ex. `:238`, `assert_eq!(body["expiresIn"], 900)`), configuration
de test (jwt 15, inactivité 60) : connexion ; la ligne de renouvellement porte `expires_at ≈ now + 60 min` ; on recule cet `expires_at` de
16 minutes (la ligne reste valable 44 minutes, comme seize minutes après la connexion) et un `POST /auth/refresh` **réussit** (200, nouvelle
ligne à `now + 60`). Variante 15/15 (configuration forcée par `from_fields_for_test`) : règle du double ⇒ 30, le même appel réussit.
**Rouge avant correctif** (défaut 15 : la ligne a expiré).

**AC-3 — La grâce de rotation.** Test d'intégration (`auth_e2e.rs`) : deux `POST /auth/refresh` avec le **même** cookie ; avant le second, on
pose `revoked_at = now − 5 s` sur la ligne tournée : le second rend `401 INVALID_REFRESH_TOKEN` et **les sessions de l'utilisateur ne sont
pas révoquées** (la ligne neuve du premier reste active) ; avec `revoked_at = now − 60 s` : détection de vol, toutes les lignes révoquées
(test existant de la détection s'il y en a un — le développeur le cherche par `git grep -n 'theft_detected' crates/kesh-api/tests` —,
sinon ajouté). **Rouge avant correctif** pour le premier cas.

**AC-4 — Le repli après un refus.** `api-client.test.ts` : une requête reçoit 401, `POST /auth/refresh` rend 401, `GET /auth/me` rend 200
⇒ pas d'effacement, pas de redirection, la requête est rejouée et réussit ; `GET /auth/me` rend 401 ⇒ `clearSession()` et redirection
(comportement existant). Les tests existants du chemin 401 (`:205`, `:246`, `:270`, `:327`, `:488`, `:567`) : le développeur les rejoue ;
ceux dont la séquence de simulacres suppose qu'un 401 au refresh est **immédiatement** suivi de l'effacement reçoivent le simulacre du
`GET /auth/me` (401) — liste exacte et raison au Change Log. **Rouge avant correctif** (premier cas).

**AC-5 — Deux onglets.** `api-client.test.ts` (deux appels concurrents) : premier `POST /auth/refresh` 200, second 401 « recently rotated »
puis `GET /auth/me` 200 ⇒ les deux requêtes d'origine rejouées et réussies, aucune redirection.

**AC-6 — Le démarrage d'un onglet.** `auth.svelte.test.ts` : `hydrate()`, `GET /auth/me` 401, `POST /auth/refresh` 200, `GET /auth/me`
200 ⇒ `currentUser` peuplé ; refresh 401 ⇒ `currentUser` nul, **aucune** redirection (`window.location.replace` non appelé). Les tests
existants d'`hydrate` qui attendent un seul `fetch` sur 401 reçoivent le simulacre du refresh — nommés au Change Log. **Rouge avant
correctif** (premier cas).

**AC-7 — Les lectures de fond.** `api-client.test.ts` : une requête `background: true` reçoit 401 ⇒ **aucun** `POST /auth/refresh`, rejet
`UNAUTHENTICATED`, pas d'effacement ; sans l'option, renouvellement (existant). Test du module `backup-schedule.svelte.ts` (simulacre
d'`apiClient`) : l'appel de la minuterie passe `background: true`, la lecture du montage non.

**AC-8 — Recette sur une installation de dev.** `kesh_v0172b`, backend `KESH_JWT_EXPIRY_MINUTES=1`, `KESH_REFRESH_INACTIVITY_MINUTES=3` :
(a) taper dans un brouillon 2 minutes sans enregistrer, puis enregistrer : réussite (401, renouvellement, rejeu — vu au journal du
backend : `refresh successful`) ; (b) ne rien faire 3 min 30 s, puis un clic : retour à la connexion ; (c) deux onglets, une action dans
chacun à quelques secondes d'intervalle juste après l'échéance du jeton d'accès : aucune déconnexion, aucune ligne `token replay
detected` ; (d) recharger la page 2 minutes après la dernière action : la session vit (renouvellement silencieux) ; (e) backend sans
variable : pas d'avertissement, `refresh_inactivity` vaut 60 (journal de démarrage) ; avec 15/15 : l'avertissement apparaît ;
(f) un `.env` copié du nouveau `.env.example` démarre sans avertissement. Déroulé au Dev Agent Record.

**AC-9 — Les manuels et la configuration livrée disent vrai** (§ *Manuel*, PDF compris ; commentaires de `.env.example`).

## Tests et mutations

| # | Test | Attendu avant correctif |
|---|---|---|
| T1 | config : défaut 60, règle du double (AC-1) | rouge |
| T2 | renouvellement après l'échéance de l'accès (AC-2) | rouge |
| T3 | grâce de rotation (AC-3) | rouge (1ᵉʳ cas), vert (2ᵈ) |
| T4 | repli `/me` après refus (AC-4), deux onglets (AC-5) | rouges |
| T5 | `hydrate` renouvelle (AC-6) | rouge (1ᵉʳ cas) |
| T6 | requêtes de fond (AC-7) | rouge |

**Mutations nommées (P2)** — sous le verrou de gate ; après une restauration par `cp`/`mv`, toucher le fichier :
- **M1** — défaut rétabli à 15 : T1 et T2 rougissent.
- **M2** — règle du double retirée : T1 (cas 15/15) et la variante 15/15 de T2 rougissent.
- **M3** — grâce portée à 0 s : T3 (1ᵉʳ cas) rougit.
- **M4** — grâce sans borne (aucune détection de vol) : T3 (2ᵈ cas) rougit.
- **M5** — le `GET /auth/me` de repli retiré : T4 rougit.
- **M6** — `hydrate` sans renouvellement silencieux : T5 rougit ; **M6 bis** — le renouvellement silencieux redirige sur échec : T5
  (« aucune redirection ») rougit.
- **M7** — l'option `background` ignorée : T6 rougit.

## Tâches

1. T1–T6 rouges sur `d7ec2ea2` (constatés et notés).
2. Serveur : `config.rs` (défaut, règle du double, avertissement, `from_fields_for_test`), `routes/auth.rs` (grâce de 30 s) ; tests.
3. Frontend : `api-client.ts` (`background`, repli `/me`), `auth.svelte.ts` (`hydrate`), `backup-schedule.svelte.ts` (appel de fond) ;
   tests ; mutations M1–M7.
4. Configuration livrée : `.env.example:160` (valeur 60) et ses commentaires — `:159` (« inactivité, comptée depuis le dernier
   renouvellement, au moins le double de `KESH_JWT_EXPIRY_MINUTES` ; défaut 60 ») et `:130` (« durée de vie du **cookie** de
   renouvellement ; aucune durée absolue pour un utilisateur actif ») (finding P17) ; `docker-compose.yml:85`, `docker-compose.prod.yml:94`.
5. AC-8 (recette) ; manuels et PDF ; CHANGELOG.
6. Gates : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh` (sous
   `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` et `mem-guard`), `cd frontend && npm run check && npm run lint-i18n-ownership &&
   npm run test:unit && npm run build`, E2E complet au **dernier commit de code** (7 à 10 échecs attendus) ; base remise à zéro avant le
   gate complet. Le changement du défaut touche toutes les suites qui construisent une configuration : gate complet au commit du serveur.

## Manuel

PDF régénéré par `make fr` dans `docs/manual/`, contrôlé aplati **et apostrophes normalisées** — `pdftotext docs/manual/fr/<manuel>.pdf - |
tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g"` — puis `grep -F` de chaque phrase neuve et de chaque phrase retirée.

**Inventaire, relevé insensible à la casse** (finding P6 de la validation P3) :
`grep -niE 'refresh|cookie|JWT|jeton d.accès|access token|KESH_JWT|INACTIVITY|SameSite|session' docs/manual/fr/admin-manual.tex` rend
**82** lignes : **75** sans « MCP » et **7** qui en portent (`:1444, :1446, :2260, :3128, :3210, :3333, :3421`, relues : seule `:1446`
parle de durée de session — le filtre `grep -viE mcp` du relevé précédent l'écartait) ; `grep -niE
'refresh|cookie|JWT|jeton|session|reconnect|déconnect' docs/manual/fr/user-manual.tex` rend **16** lignes (`:79, :95, :106, :114, :119,
:122, :313, :326, :1689, :1733, :2084, :3414, :3486, :3504, :3843, :3901`).

**`user-manual.tex`** — au tableau : `:95` et `:119` ; hors classe, les 14 autres (`:79, :114` récupération du mot de passe ; `:106`
déconnexion de toutes les sessions au changement de mot de passe, exact ; `:122` cookies `HttpOnly Secure`, exact ; `:313, :326, :3504,
:3901` clés API et écrans à venir ; `:1689, :1733, :2084` « une autre session » au sens d'un autre utilisateur ou onglet ; `:3414, :3486`
sauvegarde et `company_id` ; `:3843` glossaire du journal d'audit).

**`admin-manual.tex`** — au tableau (13) : `:111, :777, :778, :779, :1446, :2360, :2506, :2995, :2996, :2997, :2998, :3708, :3886` ; hors
classe (69) : le secret JWT (`:243, :245, :252, :253, :577, :579, :599, :732, :1744, :1749, :1752, :2414, :2766, :2770, :2772, :2783,
:2789, :2987, :3691`), `KESH_COOKIE_SECURE` et HTTP-seul (`:784, :1391, :1393, :1401, :1402, :1403, :1404, :1408, :1413, :1418, :1421,
:1427, :1432, :1750, :1753, :2763, :2819`), le multi-tenant (`:112, :1500`), les sessions révoquées (`:1192, :1361, :2322`), la sauvegarde
(`:1662`), les sessions SSH/root (`:2027, :2130, :2164`), les MCP (`:1444, :2260, :3128, :3210, :3333, :3421`), les lignes sans rapport
(`:1655, :1665, :1988, :1991, :2050, :2091, :2292, :2793, :2925, :3003, :3040, :3269, :3341, :3357, :3618`), les titres (`:772, :2990,
:2992`). (`:2322` — le bilan d'un import complet reste lisible — reste vrai : la conception retenue ne renouvelle qu'à une requête, et le
bilan n'en fait pas avant *Se reconnecter* ; le développeur le vérifie à la recette.)

| Site | Ce qu'il dit | Prescrit |
|---|---|---|
| `user-manual.tex:95` | « La session reste active 15 minutes, prolongée automatiquement à chaque action … Après 15 minutes sans activité » | la session se prolonge tant que vous travaillez dans Kesh ; elle prend fin, avec les réglages par défaut, **entre 45 et 60 minutes après votre dernière action qui interroge le serveur** (ouvrir un écran, enregistrer, chercher) ; une saisie longue sans enregistrer est sûre tant qu'elle dure moins de 45 minutes ; un rechargement de la page ne déconnecte pas tant que la session vit ; la durée se règle par `KESH_REFRESH_INACTIVITY_MINUTES` (l'administrateur, au moins le double de la durée du jeton d'accès) |
| `user-manual.tex:119` | « Tous les refresh tokens de votre session actuelle sont révoqués » | la déconnexion révoque le jeton de renouvellement **de cette session** (`routes/auth.rs:306`, `revoke_by_token`) ; les autres appareils restent connectés |
| `admin-manual.tex:111` | « durée d'expiration courte (15 min) compensée par rafraîchissement automatique côté frontend » | exact après correctif ; préciser « renouvelé à la première requête après son échéance » |
| `:777` | `KESH_JWT_EXPIRY_MINUTES` « Durée access token » | ajouter « (et du cookie d'accès) » |
| `:778` | `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` « Durée absolue refresh token » | durée de vie du **cookie** de renouvellement (`routes/auth.rs:63-66`), reposé à chaque renouvellement : **aucune durée absolue** ne borne la session d'un utilisateur actif |
| `:779` | `KESH_REFRESH_INACTIVITY_MINUTES` « Inactivité avant expiration refresh (sliding) » ; défaut `15` | défaut **60** ; au moins le double de `KESH_JWT_EXPIRY_MINUTES` (sinon porté au double, avertissement au démarrage) ; la session prend fin entre `inactivité − durée du jeton d'accès` et `inactivité` après la dernière requête de l'utilisateur ; un `.env` recopié de la 0.17.1 (15) donne 30 |
| `:1446`, `:2360`, `:2506` | « au plus sa durée plus une minute » / « la durée de session plus une minute » | « au plus la durée du **jeton d'accès** (`KESH_JWT_EXPIRY_MINUTES`) plus une minute » — c'est lui qui porte la société |
| `:2995` | cookie d'accès « `SameSite=Lax` » | `SameSite=Strict` (`routes/auth.rs:52, :62`) |
| `:2996` | « refresh token (durée 30 jours par défaut) … hash SHA-256 (jamais stocké en clair) » | 30 jours : durée du cookie ; la ligne expire après l'inactivité ; jeton stocké **tel quel** (`repositories/refresh_tokens.rs:30-34`) — **issue demandée** (C-v0172s-14) |
| `:2997` | rotation et trace pour la détection du rejeu | ajouter la grâce : un rejeu dans les 30 secondes d'une rotation (deux onglets) est refusé sans révoquer les autres sessions |
| `:2998` | « La déconnexion révoque tous les refresh tokens de la session » | « révoque le jeton de renouvellement de la session » |
| `:3708` | Dépannage « Erreurs 401 en cascade » — « Cause : Refresh token expiré ou invalidé » | ajouter la cause ordinaire : session expirée par inactivité (aucune requête de l'utilisateur depuis plus de `KESH_REFRESH_INACTIVITY_MINUTES`) ; un bandeau de fond peut alors rester affiché jusqu'à la première action, qui mène à la connexion |
| `:3886` | glossaire « Token longue durée » | jeton qui renouvelle le jeton d'accès, valable l'inactivité réglée après chaque renouvellement |

## CHANGELOG

Dans la section commune `## [0.17.2] — Non publié`, sous `### Corrigé` :
- **La session ne se ferme plus pendant que vous travaillez** ([#876](https://github.com/guycorbaz/kesh/issues/876)) : avec les réglages
  par défaut, elle prenait fin quinze minutes après la connexion, même en pleine saisie, et la saisie en cours était perdue. La durée
  d'inactivité passe à **60 minutes** par défaut : la session se ferme alors entre 45 et 60 minutes après votre dernière action. *Un
  `.env` recopié d'une version antérieure, qui pose `KESH_REFRESH_INACTIVITY_MINUTES=15`, est porté à 30 minutes (au moins le double de
  la durée du jeton d'accès), avec un avertissement au démarrage ; pour 60 minutes, retirez la ligne ou mettez-la à 60.*

`refs #876` aux commits ; aucun mot-clé de fermeture. La note de mise à jour (manuel d'administration, *Passer à la 0.17.2*) relève de la
préparation de la release (orchestrateur) : elle reprend la phrase en italique.

## Dev Notes

- **Pourquoi aucun test ne l'a vu** : les E2E durent moins de quinze minutes ; le test du renouvellement (`api-client.test.ts:205-`) simule
  un refresh **qui réussit**.
- **Sécurité** : l'inactivité tolérée passe de 15 minutes (en fait : la session mourait 15 minutes après la connexion) à 45-60 minutes par
  défaut — c'est le prix de la correction ; l'administrateur règle la valeur. La grâce de 30 s n'émet aucun jeton : un attaquant qui rejoue
  un jeton volé dans ce délai reçoit 401 ; seule la détection (révocation de toutes les sessions) est suspendue pour ce rejeu.
- **E2E** : le backend des E2E ne pose pas la variable (recette `CLAUDE.md`) : il prendra 60 ; aucune spec ne dépend de 15 (le
  développeur le vérifie : `git grep -n 'INACTIVITY' frontend/tests`).
- **Jamais de donnée réelle** ; recette sur `kesh_v0172b` seulement.

## Change Log

- 2026-11-04 — **fiche extraite** de `v0172-correctifs.md` (sous-story A1) à la remédiation de la validation P2 (Opus) : 0/0/11/12 de la
  fiche mère ; findings P1 à P5, P12 à P17 de cette passe traités sur la conception d'alors (renouvellement à l'entrée de l'utilisateur).
  Prochaine : validation P3 (Sonnet, complète).
- 2026-11-04 — **Validation P3 (Sonnet) : 0/0/7/10** (rapport `kesh-gate-logs/v0172s-validate-p3.md`). **Conception remplacée**, sur
  instruction de l'orchestrateur, après évaluation au code d'une conception plus simple : une fenêtre d'inactivité plus longue que le jeton
  d'accès suffit au chemin existant. Lu : `routes/auth.rs:254, :399, :417, :421-432` (le renouvellement glisse), `:377-389` (rejeu ⇒ vol),
  `api-client.ts:185-192` (mutex d'onglet), `:452-461` (rejeu une fois), `middleware/auth.rs:73-` (le 401 précède le handler : rejeu sûr),
  `auth.svelte.ts:229, :261-265` (`hydrate` ne renouvelle pas), `BackupScheduleBanner.svelte:27, :34` (seule lecture périodique
  authentifiée), `config.rs:735-755`, `:2689-2721`, `.env.example:128-130, :159-160`, `docker-compose*.yml`. **Findings sur la mécanique
  écartée — sans objet, chacun relu** : P1 (amorçage Vitest), P2 (`renewedAt`), P3 (`navigator.locks` absent en HTTP : la conception
  retenue n'a aucun verrou ; la course entre onglets est fermée par la grâce du serveur et le repli `/me`, en HTTP comme en HTTPS), P4
  (déconnexion sous verrou), P5 (état suspendu), P8 (`renewedAt` futur), P9 (`T`, désarmement, `sessionExpiresIn`), P10 (événements
  d'entrée), P11 (états du module), P15 (`<iframe>`), P16 (effacement selon l'horloge locale). **Appliqués** : **P6** (MEDIUM) —
  inventaire relevé en `grep -niE` (82 + 16 lignes) ; `admin-manual.tex:3708`, `:2360`, `:1446` (écartée par le filtre « mcp » du relevé
  précédent) et `user-manual.tex:119` au tableau ; **P7** (MEDIUM) — sans objet sous la conception retenue (cinq modules, § *Contrainte*) ;
  **P12** (LOW) — puce du CHANGELOG récrite (« avec les réglages par défaut », sans « jamais prolongée ») ; **P13** (LOW) — `:95` nomme
  la variable ; « aucune durée absolue » déclaré (§ 7, `:778`) ; **P14** (LOW) — fichiers de tests nommés (`auth_e2e.rs`) ; **P17** (LOW)
  — commentaires `.env.example:130, :159` à récrire (Tâche 4). Registre : C-v0172s-1 à 13 écartés (historique), C-v0172s-14 (issue du jeton
  en clair) garde, C-v0172s-15 à 21 neufs. Valeurs grepées dans cette fiche : `sessionExpiresIn`, `navigator.locks`, `renewedAt`, `armé`,
  `suspend`, `14 et 15 minutes` (n'apparaissent plus qu'au § 8 *Écarté* et au Change Log). Prochaine : validation **P4** (Opus,
  **complète**, `v0172-session-validate-prompt-p4.md`).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
