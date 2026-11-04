# Story v0172-session : la 0.17.3 — la session expire pendant une activité continue (#876), et FR13 à 60 minutes (#891)

Status: ready-for-dev

*Fiche sœur de `v0172-correctifs.md`, extraite le 2026-11-04 de sa sous-story A1 (validation P2 de la fiche mère), conception
remplacée à la validation P3. **Visée pour la 0.17.3** sur décision du Project Lead du 2026-11-04 (Change Log), qui a aussi
fixé la durée : « 60 minutes : 15 est trop court : je me fais régulièrement éjecter alors que je suis en train de travailler et que je
consulte des documents. » Le changement d'exigence est porté par le **CR #891** (FR13 : 60 minutes au lieu de 15). Worktree
`/home/gcorbaz/devel/kesh-0172s`, branche `story/v0172-session`, partie de `1be0f701`. Clé courte : `v0172s`. Registre :
`v0172-choix-autonomes.md`, section `v0172s` ; **en vigueur : C-v0172s-14 à 17, 20 à 24, 26, 27** (C-v0172s-1 à 13 : première
conception, écartée ; C-v0172s-18 : retiré à la validation P4 ; C-v0172s-19 et 25 : remplacés à la validation P5 par C-v0172s-26). Issues : **#876** (bug,
P2), **#891** (CR), en `refs` seulement. Numéros de ligne de `1be0f701` (le code applicatif est celui de `d7ec2ea2`). La prochaine passe
de validation est la **P6**, complète ; la boucle approche du plafond de huit passes (`CLAUDE.md`).*

## Story

En tant que Comptable qui saisit une facture ou consulte des documents pendant vingt minutes, je ne veux pas être renvoyé à la connexion
au moment d'enregistrer ; en tant qu'utilisateur parti de son poste, je veux que la session se ferme après un temps d'inactivité connu ;
en tant qu'Administrateur qui restaure une sauvegarde complète, je veux que toutes les sessions soient réellement fermées.

## Contrainte

Production réelle sur le NAS du Project Lead : aucun essai hors base jetable (`kesh_v0172s`, à créer, migrations du worktree, seed
`scripts/seed-dev-db.sql`) ; **aucune migration** (88 avant et après, `ls crates/kesh-db/migrations/*.sql | wc -l`) ; aucune donnée
réécrite hors des jetons de renouvellement révoqués par une restauration complète (§ 9) ; retour à la version précédente sans restauration (même
schéma, jetons de même format). Pas de P5, P6, P7.

**Modules** (règle de splitting) : (1) `crates/kesh-api` — `config.rs` (défaut et règle de l'inactivité), `routes/auth.rs` (grâce de
rotation), `routes/admin.rs` (révocation après restauration complète) ; (2) `crates/kesh-db/src/repositories/refresh_tokens.rs` (deux
requêtes) ; (3) `frontend/src/lib/shared/utils/api-client.ts` (option `background`, `get(url, call?)`, `tryRefresh()`) et
`frontend/src/lib/app/stores/auth.svelte.ts` (`hydrate()`, un appel) ; (4)
`frontend/src/lib/features/admin-backup/` (`BackupScheduleBanner.svelte`, `backup-schedule.svelte.ts`, `backup-schedule.api.ts`, et leurs
tests) ; (5) la configuration livrée et sa documentation (`.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`,
`crates/kesh-api/README.md`), les manuels FR, le PRD (`_bmad-output/planning-artifacts/prd.md`, `epics.md`, CR #891) et
`docs/kesh-specifications.txt`. Une modification de `repositories/` impose le gate complet en cours de boucle (`CLAUDE.md`, exception
`kesh-db`).

**Dérogation règle de splitting** (C-v0172s-27) : au sens strict de la règle, le frontend touche deux modules de premier niveau
(`lib/shared/utils`, `lib/app/stores`), soit **six** en tout. *Justification* : le changement d'`auth.svelte.ts` est un appel
(`hydrate()` → `tryRefresh()`), indissociable de la fonction qu'il appelle ; séparer les deux en deux stories livrerait l'une sans l'autre
ou ferait dépendre l'une de l'autre au merge. *Risque accepté* : une revue sur six modules ; il est borné par la taille du changement
frontend (une fonction extraite de `doRefresh`, un appel, une option) et par le gate complet imposé par le repository.

## Ce que le code dit (1be0f701)

1. **Deux durées de 15 minutes, comptées du même instant.** Le jeton d'accès vaut `jwt_expiry` (`routes/auth.rs:244-250` ; défaut
   15 min, `config.rs:688-708`) ; le jeton de renouvellement reçoit `expires_at = now + refresh_inactivity` (`routes/auth.rs:254` ;
   défaut 15 min, `config.rs:735-755`, repli 15 hors bornes et sur valeur invalide, `:739-751`). Même règle au renouvellement (`:422`),
   au changement de mot de passe (`:524`) et à l'assistant (`routes/setup.rs:288`). La configuration livrée pose 15/15 :
   `.env.example:129, :160` (commentaires `:128`, `:130`, `:159`), `docker-compose.yml:76, :85`, `docker-compose.prod.yml:90, :94`.
   FR13 l'exige : `prd.md:391` « la termine après 15 minutes d'inactivité (configurable) », `prd.md:280`, `epics.md:63`, `:616`.
2. **Le cookie d'accès disparaît à l'échéance** (`build_auth_cookies`, `max_age = jwt_expiry`, `routes/auth.rs:54`) : la requête suivante
   part sans jeton et reçoit 401 du middleware (`middleware/auth.rs:73-`), avant tout traitement. Le cookie de renouvellement vit
   `refresh_token_max_lifetime` (30 jours, `:63-66`), reposé à chaque renouvellement ; drapeau `Secure` selon `KESH_COOKIE_SECURE`
   (`:51`, `:61`).
3. **Le chemin 401 existe et se rejoue** : `request` (`api-client.ts:452-460`) et `requestRaw` (`:551-`) appellent `refreshTokens()`
   (mutex d'onglet, `refreshPromise`, `:185-192`), puis **rejouent** la requête une fois (`isRetry`, `:458-461`).
4. **Le renouvellement glisse** : chaque `POST /auth/refresh` réussi révoque l'ancienne ligne (`:418`, `revoke_by_token(…, "rotation")`) et
   en crée une neuve à `now + refresh_inactivity` (`:421-432`).
5. **Le défaut** : avec 15 = 15, le 401 survient à `t₀ + 15`, la ligne a expiré à `t₀ + 15` ; `refresh` la refuse (`expires_at < now`,
   `:399`) ⇒ `doRefresh` efface la session et redirige (`api-client.ts:174-178`) — saisie perdue. Le chemin réussit dès que
   `refresh_inactivity > jwt_expiry`.
6. **Le rejeu est sûr** : le 401 du jeton d'accès précède le handler (§ 2) ; la requête est rejouée à l'identique (`:460`). *Limite* :
   `PUT /api/v1/auth/password` sur mot de passe courant faux rend 401 du handler, renouvelle puis rejoue un refus — comportement 0.17.1,
   sans écriture. Le développeur vérifie qu'aucun corps non rejouable n'est employé (`git grep -nE 'ReadableStream|duplex' -- frontend/src`).
7. **Deux onglets** : s'ils reçoivent 401 dans le même intervalle, le second présente un jeton déjà tourné ⇒ `revoked_reason ==
   "rotation"` (`:379`) ⇒ `revoke_all_for_user(…, "theft_detected")` (`:380-389`) : **toutes** les sessions de l'utilisateur, tous
   appareils, sont révoquées. Le test `refresh_replay_after_rotation_revokes_all` (`crates/kesh-api/tests/auth_e2e.rs:1066-1116`) fige ce
   comportement en rejouant l'ancien jeton **aussitôt** après la rotation ; le cas 3 de `all_refresh_error_codes_are_identical`
   (`:1647-`, « Case 3: revoked token (rotate, then re-present) », `:1681`) aussi. Sonde qui les trouve : `git grep -nEi
   'replay|rotation|theft' -- crates/kesh-api/tests/auth_e2e.rs` rend `:1046, :1056, :1063, :1066, :1068, :1085, :1115` (le motif
   `theft_detected` seul ne rend rien).
8. **Une seule lecture périodique authentifiée** : `BackupScheduleBanner.svelte:33` (lecture au montage) et `:34` (`setInterval`, toutes
   les 30 min, `REFRESH_MS`, `:27`) appellent la même `refreshBackupScheduleState()` (`backup-schedule.svelte.ts:49`), qui appelle
   `fetchBackupSchedule()` (`backup-schedule.api.ts:82`), qui appelle `apiClient.get` (`api-client.ts:575`, `get<T>(url)` — **sans**
   options). L'autre minuterie (`api-health.svelte.ts:102`) interroge `/health`, non authentifié.
9. **La restauration complète remplace `refresh_tokens` sans révoquer** : `routes/admin.rs:180-182` (« L'import remplace les
   `refresh_tokens` ⇒ sessions destination invalidées »), `restore_tables_in_tx` (`:422-425`), puis le rejeu des backfills (`:441-`) et
   l'audit dans la même transaction. Les lignes de la sauvegarde sont réinsérées telles quelles, actives comprises. En 0.17.1, avec 15/15,
   elles mouraient de fait avec le cookie d'accès ; avec 60/15, une ligne de la sauvegarde dont le jeton est encore celui d'un cookie
   (aucune rotation depuis l'export) se renouvelle : la session — et des sessions d'autres appareils, même déconnectées après l'export —
   **survit** à l'import. `admin-manual.tex:2292`, `:2322` (« la session est déjà invalidée ») et le corps `sessionInvalidated: true`
   (`routes/admin.rs:267`) deviendraient faux.

## Conception retenue

1. **Le défaut** (C-v0172s-16, CR #891) : `KESH_REFRESH_INACTIVITY_MINUTES` passe de 15 à **60** (`config.rs:735-755`, les deux replis
   compris ; `.env.example:160` ; `docker-compose.yml:85`, `docker-compose.prod.yml:94`, `:-60`). `KESH_JWT_EXPIRY_MINUTES` reste à 15.
2. **La règle du double** (C-v0172s-16, C-v0172s-22) : une **fonction unique** `effective_refresh_inactivity(jwt: TimeDelta, lue:
   TimeDelta) -> TimeDelta` rend `max(lue, 2 × jwt)` ; appelée par `from_env` **seulement**, après la lecture et la validation de bornes
   de la valeur lue (l'avertissement : `KESH_REFRESH_INACTIVITY_MINUTES=15 inférieur au double de KESH_JWT_EXPIRY_MINUTES=15 : 30 minutes
   appliquées`). `from_fields_for_test` (`config.rs:443-510`) **n'est pas** modifié : ses ~80 appelants des tests d'intégration
   (`auth_e2e.rs:63-65` et suivants, qui passent 15/15 explicitement) gardent leurs valeurs, et son assertion `≤ 24 h` (`:474-476`)
   porte toujours sur une valeur passée, jamais doublée. `make_test_config` (`config.rs:1617-1635`, `#[cfg(test)]`) non plus.
3. **La grâce de rotation** (C-v0172s-17) : dans `refresh`, avant la détection de vol (`:379`), un jeton révoqué pour `"rotation"`
   **depuis moins de 30 secondes** rend `401 INVALID_REFRESH_TOKEN` **sans** `revoke_all_for_user`. L'âge se calcule **à l'horloge de la
   base** (une seule horloge : celle qui a écrit `revoked_at = NOW(3)`, `refresh_tokens.rs:114`) : une fonction de dépôt
   `refresh_tokens::was_rotated_within(pool, token, secs) -> bool` (`… WHERE token = ? AND revoked_reason = 'rotation' AND revoked_at >
   NOW(3) - INTERVAL ? SECOND`). Le code d'erreur reste celui de tous les refus (`all_refresh_error_codes_are_identical`).
4. **Les lectures de fond** (C-v0172s-20) : `CallOptions` (`api-client.ts:59-64`) reçoit `background?: boolean`, conservé au rejeu ;
   une requête de fond qui reçoit 401 **ne renouvelle pas** : elle rejette `UNAUTHENTICATED`, sans effacement ni redirection.
   `apiClient.get` devient `get<T>(url, call?: CallOptions)` (`:575`) ; `fetchBackupSchedule(call?)` (`backup-schedule.api.ts:82`) le
   transmet ; `refreshBackupScheduleState(call?)` (`backup-schedule.svelte.ts:49`) aussi ; **le site de la minuterie**
   (`BackupScheduleBanner.svelte:34`) passe `{ background: true }`, celui du montage (`:33`) rien. Sans cela, la lecture de 30 minutes,
   sous une inactivité de 60, renouvellerait indéfiniment une session laissée sur un écran d'administration.
5. **La restauration complète révoque toutes les sessions** (C-v0172s-23) : dans la transaction d'import, **juste avant** le
   `SELECT MIN(id) FROM users WHERE role = 'Admin'` (`routes/admin.rs:488-492`) — donc après `restore_tables_in_tx` (`:422-425`), la garde
   de comptage (`:436`), le rejeu des backfills 5-bis (`:452-`) et le backfill 5-ter (`:464-`), et avant le refus « aucun compte Admin »
   (`:493-497`), la réparation 5-quater, l'audit et le commit — `refresh_tokens::revoke_all_in_tx(&mut tx, "password_change")`
   (`UPDATE refresh_tokens SET revoked_at = NOW(3), revoked_reason = ? WHERE revoked_at IS NULL`) ; le commit les emporte avec le reste.
   **Le motif** est `"password_change"`, non un motif neuf : la contrainte `chk_refresh_tokens_revoked_reason`
   (`20260406000001_refresh_tokens_revoked_reason.sql` ; `test-schema/0001_schema_squash.sql:1187`) n'admet que `logout`, `rotation`,
   `password_change`, `admin_disable`, `theft_detected`, et un motif neuf exigerait une migration (`ALTER … CHECK`), que la story exclut.
   Le précédent du dépôt a tranché deux fois de même : la réinitialisation du mot de passe par courriel (`routes/auth.rs:964-969`) et le
   recours d'urgence (`auth/bootstrap.rs:355-364`) réutilisent `"password_change"` et portent la distinction dans `audit_log.action`.
   Ici aussi : l'import **remplace** les empreintes de mot de passe par celles de la sauvegarde — c'est sémantiquement un changement de
   mot de passe pour chaque compte ; la distinction est portée par l'audit `admin.full_import`. `"admin_disable"` est écarté : il désigne
   la désactivation d'**un** compte (`routes/users.rs:366, :442`) et le lire après un import égarerait. Sonde des appelants :
   `git grep -nE '"(logout|rotation|password_change|admin_disable|theft_detected)"' -- crates/kesh-api/src` (`auth.rs:306, :382, :418, :515,
   :969`, `bootstrap.rs:364`, `users.rs:366, :442, :493`) ; aucun ne lit le motif hors `auth.rs:379` (`== "rotation"`). Un cookie d'une
   ligne de la sauvegarde rend donc `401` (« token revoked » : motif ≠ `rotation`, pas de détection de vol). `sessionInvalidated: true`
   redevient vrai. Aucun des traitements de la transaction ne lit `refresh_tokens` (le développeur le vérifie : `git grep -n
   refresh_tokens crates/kesh-db/src/post_restore.rs crates/kesh-db/src/backfill.rs crates/kesh-api/src/routes/admin.rs`), si bien que
   l'ordre ne change rien à leur résultat. Le commentaire `routes/admin.rs:180-182` est mis à jour (« l'import remplace puis **révoque**
   les `refresh_tokens` »).
6. **La borne d'inactivité** (C-v0172s-21) : *requête* = toute requête API de l'utilisateur (non de fond). La dernière à `a`, le dernier
   renouvellement à `r ≤ a`. Une requête faite pendant que le jeton d'accès vit ne prolonge rien ; une requête après renouvelle. La
   session prend fin **entre `N − jwt` et `N` après la dernière requête** (`N` = inactivité appliquée) : défauts 60/15, **entre 45 et
   60 minutes** ; un `.env` 0.17.x à 15/15, porté à 30, **entre 15 et 30 minutes**. Une saisie ou une lecture sans requête est sûre tant
   qu'elle dure moins de `N − jwt`. **Aucune durée absolue** ne borne la session d'un utilisateur actif (chaque renouvellement repose un
   cookie de `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS`). Pour retrouver une session courte : réduire **les deux** valeurs (p. ex.
   `KESH_JWT_EXPIRY_MINUTES=7`, `KESH_REFRESH_INACTIVITY_MINUTES=15`, soit 8 à 15 minutes).
7. **La course entre onglets, réduite et résiduelle** (C-v0172s-24) : la grâce supprime la révocation de **toutes** les sessions ; elle
   ne sauve pas l'onglet perdant. Si deux onglets envoient leur `POST /auth/refresh` avec le même jeton avant que la réponse du premier
   n'ait reposé les cookies — deux requêtes de l'utilisateur, dans deux onglets, à quelques millisecondes d'intervalle, juste après
   l'échéance du jeton d'accès —, le second reçoit 401 et **cet onglet** revient à la connexion (sa saisie non enregistrée est perdue) ;
   l'autre onglet et les autres appareils continuent. La fiche ne la déclare **pas** fermée. Les lectures de fond ne renouvelant plus
   (§ 4), il faut deux gestes de l'utilisateur quasi simultanés. Écrite au manuel utilisateur (`:95`) et d'administration (`:2997`).
8. **Retiré à la validation P4** (C-v0172s-18) : le **repli `GET /auth/me`** après un refus de renouvellement — un seul `/me` ne fermait
   pas la course (le 401 du second peut précéder le `Set-Cookie` du premier) ; la grâce suffit à l'essentiel, le reste est déclaré au § 7.
8 bis. **Le renouvellement au démarrage, sous forme minimale** (C-v0172s-26, rétabli à la validation P5) : sans lui, un rechargement
   ou un nouvel onglet renvoie à la connexion **dès l'échéance du jeton d'accès posé au dernier renouvellement** — parfois quelques
   secondes après la dernière action (connexion à 09:00, travail jusqu'à 09:14 sans renouvellement, F5 à 09:16 : `/me` sans cookie
   d'accès, page de connexion), c'est-à-dire le symptôme même du CR #891. Forme retenue : `api-client.ts` extrait de `doRefresh` une
   fonction **`tryRefresh(): Promise<boolean>`**, sous le **même mutex d'onglet** (`refreshPromise`, `:67-69`, `:185-192`), qui fait le
   `POST /auth/refresh` puis le `GET /auth/me` et pose l'état par `authState.login(…, { broadcast: false })` en cas de succès, et rend
   `false` **sans rien effacer ni rediriger** en cas d'échec ; le chemin 401 de `request`/`requestRaw` appelle `tryRefresh()` et, sur
   `false`, fait lui-même `clearSession()` et la redirection (comportement actuel, inchangé). `hydrate()` (`auth.svelte.ts:229-265`), sur un
   401 de `/me`, appelle **une fois** `tryRefresh()` — importé dynamiquement (`await import('$lib/shared/utils/api-client')`), pour ne pas
   créer à l'initialisation l'import circulaire `api-client` ↔ `auth.svelte` — ; succès : l'état est posé ; échec : état nul, **sans**
   `clearSession()` diffusé, **sans** redirection (`hooks.client.ts:22` appelle `hydrate()` au démarrage de toute page, `/login`
   compris). **Pourquoi P13 à P15 de la validation P4 ne reviennent pas** : aucune issue d'`hydrate()` ne diffuse `auth-change`, donc
   aucun onglet ne relance celui d'un autre (P13), et le mutex partagé interdit deux renouvellements concurrents dans un onglet ; le
   cookie périmé (P14) — un jeton déjà tourné dont la réponse a été perdue — serait de toute façon présenté au premier 401 du chemin
   existant, avec la même conséquence (détection de vol, reconnexion sur tous les appareils) : `hydrate()` ne l'expose pas davantage ;
   la raison écrite pour l'absence de redirection est désormais la bonne (P15 : c'est `hooks.client.ts`, non la page de connexion, qui
   appelle `hydrate()`).
9. **Écartée** (C-v0172s-15) : la première conception (renouvellement anticipé à chaque entrée, verrous, module armé, suspension).

## Acceptance Criteria

**AC-1 — Le défaut et la règle.** Tests unitaires (`config.rs`) : sans variable, `refresh_inactivity == 60 min` ; `=90` ⇒ 90 ; `=15` avec
`KESH_JWT_EXPIRY_MINUTES=15` ⇒ **30** et avertissement ; `=5` avec jwt 2 ⇒ 5 ; hors borne (`9999`) ou invalide ⇒ 60. Tests de
`effective_refresh_inactivity` : (15, 15) ⇒ 30 ; (15, 60) ⇒ 60 ; (1440, 1440) ⇒ 2880. **Tests existants à mettre à jour**, raison au
Change Log : `config_refresh_inactivity_defaults_to_15_min` (`config.rs:2689-2696`, ⇒ 60, et son nom),
`config_refresh_inactivity_respects_env` (`:2698-2708`, valeur 5 avec jwt 15 : poser jwt à 2), `config_refresh_inactivity_out_of_bounds_falls_back`
(`:2711-2721`, ⇒ 60) ; autres dépendances au défaut cherchées par `git grep -nE 'refresh_inactivity|minutes\(15\)' -- crates/kesh-api/src`
et nommées. **Rouges avant correctif** : défaut 60, règle du double.

**AC-2 — Le renouvellement réussit après l'échéance du jeton d'accès (garde).** `auth_e2e.rs`, configuration de test jwt 15, inactivité
60 (passées à `from_fields_for_test`) : connexion ; la ligne porte `expires_at ≈ now + 60 min` ; on recule son `expires_at` de 16 minutes
(il reste 44 minutes) ; `POST /auth/refresh` **réussit** (200, ligne neuve à `now + 60`). **Vert avant et après** : ce test garde le
chemin, il ne prouve pas le défaut — c'est AC-1 (le défaut lu par `from_env`) qui rougit avant correctif.

**AC-3 — La grâce de rotation.** `auth_e2e.rs` : (a) **nouveau** — refresh, puis rejeu **aussitôt** de l'ancien jeton : `401
INVALID_REFRESH_TOKEN`, et la ligne neuve du premier refresh reste **active** (`revoked_at IS NULL`) ; (b) **`refresh_replay_after_rotation_revokes_all`
(`:1066-1116`) modifié** : avant le rejeu, `UPDATE refresh_tokens SET revoked_at = NOW(3) - INTERVAL 60 SECOND WHERE token = ?` (l'ancien
jeton) ; détection de vol inchangée (toutes les lignes révoquées) — raison au Change Log ; (c) `all_refresh_error_codes_are_identical`
(`:1647-`) : son cas 3 passe désormais par la grâce ; un **cas 4** (même rejeu, `revoked_at` reculé de 60 s, chemin du vol) est ajouté, et
les quatre codes restent identiques. **Rouges avant correctif** : (a) ; (b) et (c) verts avant, et doivent le rester après.

**AC-4 — La restauration complète révoque tout.** `admin_full_import_e2e.rs` : (a) avant l'import, une ligne active de `refresh_tokens`
présente **dans la sauvegarde** (export fait après une connexion, puis import de cet export) ; après l'import : `SELECT COUNT(*) FROM
refresh_tokens WHERE revoked_at IS NULL` = 0 ; les lignes **auparavant actives** portent `revoked_reason = 'password_change'` (les lignes
déjà révoquées gardent leur motif d'origine) — l'import réussit, ce qui prouve que le motif passe la contrainte CHECK ; un `POST
/auth/refresh` avec le cookie de cette ligne rend 401 ; (b) **échec après la révocation** : une archive dont la table `users` ne contient
**aucun** Admin (le refus `admin.rs:493-497`, levé après le point de révocation) — l'import échoue (`AdminFullImportFailed`), la
transaction est annulée, et les lignes actives de la **destination** (la session du test, connectée avant l'import) sont **toujours
actives**, aucune n'est révoquée. `sessionInvalidated: true` inchangé (`admin_full_import_e2e.rs:326`, `admin_backup_e2e.rs:814`). **Rouge avant
correctif** (la ligne restaurée est active).

**AC-5 — Les lectures de fond.** `api-client.test.ts` : `background: true` + 401 ⇒ **aucun** `POST /auth/refresh`, rejet
`UNAUTHENTICATED`, pas d'effacement ni de redirection ; sans l'option, renouvellement (existant). `BackupScheduleBanner.test.ts`
(minuteries simulées) : la lecture du montage appelle `fetchBackupSchedule` **sans** `background`, celle qui suit `REFRESH_MS` **avec**
`{ background: true }`. `backup-schedule.api.test.ts` : un cas neuf, `fetchBackupSchedule({ background: true })` transmet l'option à
`apiClient.get` ; **l'assertion existante `:56-57`** (`toHaveBeenNthCalledWith(1, '/api/v1/admin/backup-schedule')`) devient
`toHaveBeenNthCalledWith(1, '/api/v1/admin/backup-schedule', undefined)` (le second argument est désormais transmis, absent) — raison au
Change Log. Les autres appelants (`AutoBackupSection.svelte:145, :213`) inchangés. **Rouges avant correctif.**

**AC-5 bis — Le renouvellement au démarrage.** `auth.svelte.test.ts` : (1) `hydrate()`, `GET /auth/me` 401, `POST /auth/refresh` 200,
`GET /auth/me` 200 ⇒ `currentUser` peuplé, `authBroadcast.postMessage` **non appelé** ; (2) refresh 401 ⇒ `currentUser` nul,
`window.location.replace` **non appelé**, `postMessage` non appelé ; (3) concurrent : un chemin 401 en vol dans l'onglet et `hydrate()` ⇒
**un seul** `POST /auth/refresh`. Les tests existants d'`hydrate` qui attendent un seul `fetch` sur 401 reçoivent le simulacre du refresh
(401) — nommés au Change Log. `api-client.test.ts` : les six tests du chemin 401 (`:205, :246, :270, :327, :488, :567`) restent verts
(l'effacement et la redirection passent de `doRefresh` à l'appelant, comportement identique). **Rouges avant correctif** : (1), (3).

**AC-6 — Recette sur une installation de dev.** `kesh_v0172s`, backend `KESH_JWT_EXPIRY_MINUTES=1`, `KESH_REFRESH_INACTIVITY_MINUTES=3` :
(a) taper dans un brouillon **1 min 30 s** sans enregistrer, puis enregistrer : réussite (au journal : `refresh successful`) ; (b) rien
pendant 3 min 30 s, puis un clic : retour à la connexion ; (c) deux onglets ouverts, une action dans chacun à quelques secondes
d'intervalle après l'échéance du jeton d'accès : **les deux restent connectés** (le cas n'exerce **pas** la grâce — le second onglet
trouve déjà le cookie neuf ; la grâce est établie par AC-3) ; (c bis) une action, puis **recharger la page** 1 min 30 s plus tard (jeton
d'accès échu, session vivante) : la page se recharge connectée (renouvellement au démarrage) ; (d) restauration complète d'un export fait
dans la séance : après l'import, `refresh_tokens` n'a plus de ligne active (requête en lecture sur `kesh_v0172s`) ; (e) la durée
appliquée se lit sur une ligne neuve, **arrondie en secondes** (`created_at` est posé par la base à l'`INSERT`, quelques millisecondes
après le calcul d'`expires_at`, `routes/auth.rs:254-262` : une troncature en minutes rendrait `N − 1`) : `SELECT
ROUND(TIMESTAMPDIFF(SECOND, created_at, expires_at) / 60) FROM refresh_tokens ORDER BY id DESC LIMIT 1` rend 3 ; avec
`KESH_REFRESH_INACTIVITY_MINUTES=1` et jwt 1, l'avertissement apparaît au démarrage et la requête rend 2. Déroulé au Dev
Agent Record.

**AC-7 — Les textes disent vrai.** Le § *Manuel* (PDF compris), les commentaires de `.env.example`, le PRD et les epics (§ *PRD*).

## Tests et mutations

| # | Test | Attendu avant correctif |
|---|---|---|
| T1 | config : défaut 60, règle du double (AC-1) | rouge |
| T2 | renouvellement après l'échéance de l'accès (AC-2) | vert (garde) |
| T3 | grâce de rotation (AC-3) | (a) rouge ; (b), (c) verts |
| T4 | restauration complète (AC-4) | rouge |
| T5 | lectures de fond (AC-5) | rouge |
| T6 | renouvellement au démarrage (AC-5 bis) | (1), (3) rouges ; (2) vert |

**Mutations nommées (P2)** — sous le verrou de gate ; après une restauration par `cp`/`mv`, toucher le fichier :
- **M1** — défaut rétabli à 15 : T1 rougit.
- **M2** — `effective_refresh_inactivity` rend la valeur lue : T1 (cas 15/15 et tests de la fonction) rougit.
- **M3** — grâce portée à 0 s : T3 (a) rougit.
- **M4** — grâce sans borne : T3 (b) et le cas 4 de (c) rougissent.
- **M5** — la révocation après restauration retirée : T4 (a) rougit ; **M5 bis** — la révocation déplacée **avant l'ouverture** de la
  transaction d'import, sur le pool (aucun verrou n'est encore tenu : la mutation s'exécute sans attente mutuelle) : T4 (b) rougit (les
  lignes de la destination sont révoquées alors que l'import échoue), et T4 (a) aussi (les lignes restaurées sont réinsérées actives).
- **M7** — `hydrate()` sans `tryRefresh()` : T6 (1ᵉʳ cas) rougit ; **M7 bis** — succès posé par `login()` **avec** diffusion : T6
  (« `postMessage` non appelé ») rougit ; **M7 ter** — échec suivi de `clearSession()` diffusé ou d'une redirection : T6 (2ᵈ cas) rougit ;
  **M7 quater** — `tryRefresh()` hors du mutex : T6 (cas concurrent) rougit.
- **M6** — l'option `background` ignorée dans `request` : T5 (`api-client.test.ts`) rougit ; **M6 bis** — l'argument retiré au site de
  la minuterie (`BackupScheduleBanner.svelte:34`) : T5 (`BackupScheduleBanner.test.ts`) rougit.

## Tâches

1. T1, T3 (a), T4, T5 rouges sur `1be0f701` (constatés et notés) ; T2 vert.
2. Serveur : `config.rs` (défaut, `effective_refresh_inactivity`, avertissement), `routes/auth.rs` (grâce), `routes/admin.rs`
   (révocation dans la transaction d'import), `repositories/refresh_tokens.rs` (`was_rotated_within`, `revoke_all_in_tx`) ; tests, dont
   la mise à jour de `refresh_replay_after_rotation_revokes_all` et le cas 4.
3. Frontend : `api-client.ts` (`background`, `get(url, call?)`, `tryRefresh()` extrait de `doRefresh`), `auth.svelte.ts` (`hydrate()`),
   `backup-schedule.api.ts`, `backup-schedule.svelte.ts`, `BackupScheduleBanner.svelte:34` ; tests.
4. Configuration livrée : `.env.example:160` (60) et ses commentaires `:159` (« inactivité, comptée depuis le dernier renouvellement, au
   moins le double de `KESH_JWT_EXPIRY_MINUTES` ; défaut 60 ») et `:130` (« durée de vie du **cookie** de renouvellement ; aucune durée
   absolue pour un utilisateur actif ») ; `docker-compose.yml:85`, `docker-compose.prod.yml:94` ; `crates/kesh-api/README.md:41` (défaut
   60, au moins le double du jeton d'accès), `:161` (« 60 min d'inactivité par défaut »), `:179` (« sliding expiration de 60 min par
   défaut » : le risque du jeton stocké en clair, que justifiait une durée courte, est plus grand — C-v0172s-14) ; `:160` reste vrai.
5. PRD (§ *PRD*) ; manuels et PDF ; CHANGELOG.
6. Mutations M1–M7 quater ; AC-6 (recette).
7. Gates : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh` (sous
   `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` et `mem-guard`), `cd frontend && npm run check && npm run lint-i18n-ownership &&
   npm run test:unit && npm run build`, E2E complet au **dernier commit de code** (7 à 10 échecs attendus) ; base remise à zéro avant
   chaque gate complet. **Gate complet même en cours de boucle** : la story touche un repository (`CLAUDE.md`, exception `kesh-db`).

## PRD (CR #891)

- `_bmad-output/planning-artifacts/prd.md:391` : « FR13 : Le système renouvelle silencieusement la session et la termine après **60
  minutes** d'inactivité (configurable ; au moins le double de la durée du jeton d'accès) — CR #891 ».
- `prd.md:280` : « expiration **60 min** d'inactivité » (même renvoi).
- `epics.md:63` : la reprise de FR13, à l'identique de `prd.md:391`.
- `epics.md:616` : « **Given** inactivité de **60 minutes** (configurable via env, au moins le double de la durée du jeton d'accès) … »
  (renvoi à #891).
- Le développeur greppe la valeur (`grep -nE '\b15 min(utes)?\b' _bmad-output/planning-artifacts/{prd,epics}.md`) et trie les
  occurrences (fenêtre de limitation des connexions, `epics.md:617`, hors classe).
- **`docs/kesh-specifications.txt`**, copie des spécifications : `:278` (« expiration 15 min d'inactivité »), `:389` et `:2144` (FR13),
  `:2690` (story 1.6) — **mises à jour** de même, avec le renvoi à #891 ; hors classe : `:71`, `:374`, `:1439`, `:2129`, `:2625` (installation
  et prise en main en 15 minutes), `:392`, `:2147`, `:2691` (limitation des connexions), `:806`, `:2362`, `:2679` (durée du jeton d'accès,
  inchangée). Si le fichier est engendré (le développeur le vérifie : `git log --format=%s -5 -- docs/kesh-specifications.txt`), le
  regénérer plutôt que l'éditer.

## Manuel

PDF régénéré par `make fr` dans `docs/manual/`, contrôlé aplati — `pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' ' |
sed "s/’/'/g" | sed -E 's/([[:alpha:]])- ([[:alpha:]])/\1\2/g'` — puis `grep -F` de chaque phrase neuve et de chaque phrase retirée (la
seconde substitution recolle les césures ; en cas de doute, un fragment court).

**Inventaire, relevé insensible à la casse** : `grep -niE 'refresh|cookie|JWT|jeton d.accès|access token|KESH_JWT|INACTIVITY|SameSite|session'
docs/manual/fr/admin-manual.tex` rend **82** lignes (75 sans « MCP », 7 avec : `:1444, :1446, :2260, :3128, :3210, :3333, :3421` ; seule
`:1446` parle de durée de session) ; `grep -niE 'refresh|cookie|JWT|jeton|session|reconnect|déconnect' docs/manual/fr/user-manual.tex`
rend **16** lignes.

**`user-manual.tex`** — au tableau : `:95`, `:106`, `:119` ; hors classe, les 13 autres (`:79, :114` récupération du mot de passe ; `:122`
cookies `HttpOnly Secure`, exact sous `KESH_COOKIE_SECURE=true` ; `:313, :326, :3504, :3901` clés API et écrans à venir ; `:1689, :1733,
:2084` « une autre session » au sens d'un autre utilisateur ou onglet ; `:3414, :3486` sauvegarde et `company_id` ; `:3843` glossaire du
journal d'audit).

**`admin-manual.tex`** — au tableau (17) : `:111, :777, :778, :779, :1192, :1361, :1446, :2292, :2322, :2360, :2506, :2995, :2996, :2997,
:2998, :3708, :3886` (`:1192` et `:1361` partagent une ligne du tableau, même verdict) ; hors classe (65) : le secret JWT (`:243, :245,
:252, :253, :577, :579, :599, :732, :1744, :1749, :1752, :2414, :2766, :2770, :2772, :2783, :2789, :2987, :3691`), `KESH_COOKIE_SECURE` et
HTTP-seul (`:784, :1391, :1393, :1401, :1402, :1403, :1404, :1408, :1413, :1418, :1421, :1427, :1432, :1750, :1753, :2763, :2819`), le
multi-tenant (`:112, :1500`), la sauvegarde (`:1662`), les sessions SSH/root (`:2027, :2130, :2164`), les MCP (`:1444, :2260, :3128,
:3210, :3333, :3421`), les lignes sans rapport (`:1655, :1665, :1988, :1991, :2050, :2091, :2793, :2925, :3003, :3040, :3269, :3341,
:3357, :3618`), les titres (`:772, :2990, :2992`). *(82 = 17 + 65 ; l'état précédent, 13 / 69, recompté juste par la P4, devient 17 / 65 par le
passage au tableau de `:1192`, `:1361`, `:2292` et `:2322`.)*

| Site | Ce qu'il dit | Prescrit |
|---|---|---|
| `user-manual.tex:95` | « La session reste active 15 minutes, prolongée automatiquement à chaque action … Après 15 minutes sans activité » | la session se prolonge tant que vous travaillez dans Kesh ; elle prend fin, avec les réglages par défaut, **entre 45 et 60 minutes après votre dernière action qui interroge le serveur** (ouvrir un écran, enregistrer, chercher) ; une saisie ou une lecture sans enregistrer est sûre tant qu'elle dure moins de 45 minutes ; recharger la page ou ouvrir un nouvel onglet ne déconnecte pas tant que la session vit ; deux onglets qui interrogent le serveur à la même seconde, juste après l'échéance du jeton d'accès, peuvent ramener **l'un** d'eux à la connexion (rare) ; la durée se règle par `KESH_REFRESH_INACTIVITY_MINUTES` |
| `user-manual.tex:106` | après la **récupération** par courriel : « toutes vos sessions ouvertes … ont été déconnectées » | « … ne peuvent plus être renouvelées : elles prennent fin au plus tard après la durée du jeton d'accès (15 minutes par défaut) » (`routes/auth.rs:969`) |
| `user-manual.tex:119` | « Tous les refresh tokens de votre session actuelle sont révoqués » | la déconnexion révoque le jeton de renouvellement **de cette session** (`routes/auth.rs:306`, `revoke_by_token`) ; les autres appareils restent connectés |
| `admin-manual.tex:111` | « compensée par rafraîchissement automatique côté frontend » | « renouvelé à la première requête après son échéance » |
| `:777` | `KESH_JWT_EXPIRY_MINUTES` « Durée access token » | ajouter « (et du cookie d'accès) » |
| `:778` | `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` « Durée absolue refresh token » | durée de vie du **cookie** de renouvellement (`routes/auth.rs:63-66`), reposé à chaque renouvellement : **aucune durée absolue** ne borne la session d'un utilisateur actif |
| `:779` | `KESH_REFRESH_INACTIVITY_MINUTES` ; défaut `15` | défaut **60** ; au moins le double de `KESH_JWT_EXPIRY_MINUTES` (sinon porté au double, avertissement au démarrage) ; fin de session entre `inactivité − jeton d'accès` et `inactivité` après la dernière requête ; un `.env` antérieur à 15 donne 30 ; session courte : réduire les deux (p. ex. 7 et 15) |
| `:1192`, `:1361` | révocation des sessions à la réinitialisation / au recovery : « déconnectées » | même précision que `user-manual.tex:106` |
| `:1446`, `:2360`, `:2506` | « au plus sa durée plus une minute » / « la durée de session plus une minute » | « au plus la durée du **jeton d'accès** (`KESH_JWT_EXPIRY_MINUTES`) plus une minute » — c'est lui qui porte la société |
| `:2292`, `:2322` | « votre session est invalidée » ; « la session est déjà invalidée » | **vrai après correctif** : l'import révoque tous les jetons de renouvellement (§ 5 de la conception, C-v0172s-23) ; ajouter « les sessions de tous les utilisateurs ne peuvent plus être renouvelées : elles prennent fin au plus tard à l'échéance de leur jeton d'accès (15 minutes par défaut) ; la vôtre, par le bouton *Se reconnecter* » — cohérent avec `:2360` |
| `:2995` | cookie d'accès « `HttpOnly; Secure; SameSite=Lax` » | `HttpOnly`, `Secure` sauf `KESH_COOKIE_SECURE=false`, `SameSite=Strict` (`routes/auth.rs:51-52, :61-62`) |
| `:2996` | « refresh token (durée 30 jours par défaut) … hash SHA-256 (jamais stocké en clair) » | 30 jours : durée du cookie ; la ligne expire après l'inactivité ; jeton stocké **tel quel** (`repositories/refresh_tokens.rs:30-34`) — **issue demandée** (C-v0172s-14) |
| `:2997` | rotation et trace pour la détection du rejeu | ajouter la grâce : un rejeu dans les 30 secondes d'une rotation est refusé sans révoquer les autres sessions ; l'onglet dont le renouvellement a perdu la course (deux onglets, la même seconde, juste après l'échéance du jeton d'accès) revient à la connexion |
| `:2998` | « La déconnexion révoque tous les refresh tokens de la session » | « révoque le jeton de renouvellement de la session » |
| `:3708` | Dépannage « Erreurs 401 en cascade » — « Cause : Refresh token expiré ou invalidé » | ajouter la cause ordinaire : session expirée par inactivité ; un bandeau de fond peut rester affiché jusqu'à la première action, qui mène à la connexion |
| `:3886` | glossaire « Token longue durée » | jeton qui renouvelle le jeton d'accès, valable l'inactivité réglée après chaque renouvellement |

## CHANGELOG

Dans la section de la **0.17.3** (`## [0.17.3] — Non publié`, à créer en tête si elle n'existe pas), sous `### Corrigé` :
- **La session ne se ferme plus pendant que vous travaillez** ([#876](https://github.com/guycorbaz/kesh/issues/876)) : avec les réglages
  par défaut, elle prenait fin quinze minutes après la connexion, même en pleine saisie, et la saisie en cours était perdue. La durée
  d'inactivité passe à **60 minutes** par défaut ([#891](https://github.com/guycorbaz/kesh/issues/891)) : la session se ferme alors entre
  45 et 60 minutes après votre dernière action qui interroge le serveur (ouvrir un écran, enregistrer, chercher). *Un `.env` recopié
  d'une version antérieure, qui pose `KESH_REFRESH_INACTIVITY_MINUTES=15`, est porté à 30 minutes (au moins le double de la durée du
  jeton d'accès), avec un avertissement au démarrage ; pour 60 minutes, retirez la ligne ou mettez-la à 60.*
- Recharger la page ou ouvrir un nouvel onglet ne renvoie plus à la connexion tant que la session vit.

Sous `### Modifié` :
- **Une restauration complète révoque les jetons de renouvellement restaurés** : aucune session, de la sauvegarde ou de l'installation,
  ne peut plus être renouvelée après l'import. *(Ce n'est pas la correction d'un défaut publié : avec l'ancienne durée de 15 minutes, ces
  jetons mouraient de fait avec le jeton d'accès ; la durée de 60 minutes rendait nécessaire de les révoquer.)*

`refs #876`, `refs #891` aux commits ; aucun mot-clé de fermeture. La note de mise à jour (manuel d'administration, *Passer à la 0.17.3*)
relève de la préparation de la release (orchestrateur) : elle reprend la phrase en italique.

## Dev Notes

- **Pourquoi aucun test ne l'a vu** : les E2E durent moins de quinze minutes ; le test du renouvellement (`api-client.test.ts:205-`) simule
  un refresh **qui réussit**.
- **Sécurité** : l'inactivité tolérée passe à 45-60 minutes par défaut — décision du Project Lead (CR #891). La grâce de 30 s n'émet aucun
  jeton. **Cas accepté** (C-v0172s-17) : un attaquant qui présente le premier un jeton volé obtient un jeton neuf ; la victime qui présente
  ensuite le même jeton dans les 30 s reçoit 401 **sans** révocation générale — elle est renvoyée à la connexion et l'attaquant garde une
  session ; au-delà de 30 s, la détection de vol joue comme aujourd'hui. Le risque est borné à cette fenêtre.
- **E2E** : le backend des E2E ne pose pas la variable : il prendra 60 ; le développeur vérifie qu'aucune spec n'en dépend
  (`git grep -n 'INACTIVITY' frontend/tests`).
- **Jamais de donnée réelle** ; recette sur `kesh_v0172s` seulement.

## Change Log

- 2026-11-04 — fiche extraite de `v0172-correctifs.md` (sous-story A1) à la remédiation de la validation P2 (Opus) de la fiche mère.
- 2026-11-04 — validation P3 (Sonnet) : 0/0/7/10 ; conception remplacée (fenêtre d'inactivité plus longue que le jeton d'accès).
  *Annotation (validation P4) : cette entrée citait `:417` pour la révocation par rotation ; la ligne est `routes/auth.rs:418`.*
- 2026-11-04 — **Validation P4 (Opus) : 0/0/7/13** (rapport `kesh-gate-logs/v0172s-validate-p4.md`), remédiée dans le worktree
  `kesh-0172s` (branche `story/v0172-session`), après les décisions du Project Lead du même jour : **#876 sort de la 0.17.2 pour une
  0.17.3**, et l'inactivité vaut **60 minutes** (CR #891). Prémisses relues au code avant application.
  **P1** (MEDIUM) appliqué — sonde positive `git grep -nEi 'replay|rotation|theft' -- crates/kesh-api/tests/auth_e2e.rs` (`:1066, :1068,
  :1085, :1115` …) : `refresh_replay_after_rotation_revokes_all` modifié (rejeu hors grâce), cas 3 d'`all_refresh_error_codes_are_identical`
  passé par la grâce, cas 4 ajouté (AC-3). **P2** (MEDIUM) appliqué — `config.rs:443-510` (`from_fields_for_test`, paramètre) et
  `:1617-1635` (`make_test_config`) lus : la règle vit dans une fonction unique appelée par `from_env` seul ; les ~80 appelants de
  `from_fields_for_test` inchangés (C-v0172s-22). **P3** (MEDIUM) appliqué — AC-2 déclaré garde (vert avant), le rouge porté par AC-1 ; M1
  corrigée. **P4** (MEDIUM) appliqué — `BackupScheduleBanner.svelte:33-34`, `api-client.ts:575` lus : `get(url, call?)`,
  `fetchBackupSchedule(call?)`, site de la minuterie nommé, M6 bis. **P5** (MEDIUM) appliqué — la course est **réduite et résiduelle**,
  écrite comme telle (§ 7, C-v0172s-24) ; le repli `/me`, qui ne la fermait pas, est **retiré**. **P6** (MEDIUM) appliqué, sur décision du
  Project Lead — `routes/admin.rs:180-182, :422-425` lus : la restauration complète révoque toutes les lignes dans sa transaction (§ 9,
  AC-4, M5, C-v0172s-23). **P7** (MEDIUM) appliqué — CR #891 ; FR13 mis à jour (`prd.md:391`, `:280`, `epics.md:63`, `:616`, § *PRD*).
  LOW : **P8** — `:418` (annotation datée ci-dessus) ; **P9** — durée appliquée observée par `TIMESTAMPDIFF` sur une ligne neuve (AC-6 e) ;
  **P10** — 1 min 30 s (AC-6 a) ; **P11** — âge de la rotation à l'horloge de la base (`was_rotated_within`) ; **P12** — cas où
  l'attaquant tourne le premier, déclaré (Dev Notes, C-v0172s-17) ; **P13**, **P14**, **P15** — **sans objet** : le renouvellement
  silencieux d'`hydrate()` est **retiré** (§ 8) ; **P16** — `Secure` conditionnel à `:2995` ; **P17** — `user-manual.tex:106`
  reclassé (récupération par courriel), `:1192`, `:1361` au tableau ; **P18** — C-v0172s-14 annoté au registre ; **P19** — trace de la
  conception écartée dans la fiche **mère** (`v0172-correctifs.md:50`, « champ JSON ») : **non corrigée ici**, la fiche mère étant en
  développement dans un autre worktree — demandée à l'orchestrateur ; **P20** — puce du CHANGELOG alignée sur le manuel.
  **Retraits** (simplicité, instruction de l'orchestrateur) : le repli `GET /auth/me` (C-v0172s-18) et le renouvellement silencieux
  d'`hydrate()` (C-v0172s-19), avec leurs AC (anciens AC-4 à AC-6) et mutations (M5, M6, M6 bis anciennes) ; conséquences déclarées
  (§ 7, § 8, manuel `:95`, C-v0172s-25). **Version** : toute mention de la 0.17.2 retirée ; CHANGELOG en 0.17.3. Valeurs grepées dans
  cette fiche : `0.17.2`, `[0.17.2]`, `/auth/me`, `hydrate`, `:417`, `theft_detected`, `kesh_v0172b`. Prochaine : validation **P5**
  (Sonnet, complète, `v0172-session-validate-prompt-p5.md`).

- 2026-11-04 — **Validation P5 (Sonnet) : 0/1/4/9** (rapport `kesh-gate-logs/v0172s-validate-p5.md`). Prémisses relues au code.
  **P1** (HIGH) appliqué — `20260406000001_refresh_tokens_revoked_reason.sql` et `test-schema/0001_schema_squash.sql:1187` (CHECK à cinq
  valeurs), `routes/auth.rs:964-969`, `auth/bootstrap.rs:355-364` lus ; sonde des appelants (`git grep -nE '"(logout|rotation|password_change|admin_disable|theft_detected)"'`) :
  motif **`"password_change"`**, sans migration, sur le précédent du dépôt, distinction portée par l'audit ; `"admin_disable"` écarté ;
  § 5, AC-4, C-23 récrits. **P2** (MEDIUM) appliqué — `admin.rs:436, :452, :464, :488-497` lus : révocation placée juste avant le `SELECT
  MIN(id)` ; déclencheur décidable de l'échec après révocation : archive sans Admin (AC-4 b) ; M5 bis récrite (révocation sur le pool
  **avant** la transaction : ne s'auto-bloque pas). **P3** (MEDIUM) appliqué, sur instruction de l'orchestrateur — le renouvellement au
  démarrage est **rétabli** sous forme minimale (§ 8 bis : `tryRefresh()` sous le mutex, aucune diffusion, aucune redirection ;
  AC-5 bis, M7 à M7 quater ; C-v0172s-26 remplace C-19 et C-25) ; la raison pour laquelle P13 à P15 de la P4 ne reviennent pas est écrite ;
  dérogation de splitting écrite (six modules, C-v0172s-27). **P4** (MEDIUM) appliqué — `routes/auth.rs:254-262` lu : arrondi en
  secondes (AC-6 e). **P5** (MEDIUM) appliqué — `crates/kesh-api/README.md:41, :161, :179` lus : ajoutés au module (5) et à la Tâche 4.
  LOW : **P6** appliqué (position fixée, seul argument « aucun lecteur de `refresh_tokens` », commentaire `admin.rs:180-182`) ; **P7**
  appliqué (`:2292`, `:2322` : « ne peuvent plus être renouvelées … au plus tard à l'échéance du jeton d'accès ») ; **P8** appliqué (puce
  sous *Modifié*, avec la raison) ; **P9** appliqué (course résiduelle au manuel `:95` et `:2997`) ; **P10** appliqué au registre (en-tête
  de section et « Retenu » de C-v0172s-15) ; **P11** appliqué (`docs/kesh-specifications.txt:278, :389, :2144, :2690`, et le tri des
  autres occurrences) ; **P12** — la phrase de la fiche mère est déjà corrigée par l'agent de développement (instruction de
  l'orchestrateur) ; les PDF des manuels régénérés par les deux fiches sont un point de **fusion** pour l'orchestrateur (régénérer après
  la fusion des `.tex`) ; **P13** appliqué (assertion `backup-schedule.api.test.ts:56-57` nommée) ; **P14** appliqué (AC-6 c récrit, il
  n'exerce pas la grâce ; c bis ajouté pour le rechargement). Valeurs grepées dans cette fiche : `full_import` (ne reste que l'action
  d'audit `admin.full_import`), `TIMESTAMPDIFF(MINUTE`, `plus de 15 minutes après`, `C-v0172s-25`, `M6 bis` (Tâche 6). Prochaine :
  validation **P6** (Opus, complète, `v0172-session-validate-prompt-p6.md`) — sixième passe : **dès celle-ci, l'orchestrateur ne remédie
  plus lui-même** (`CLAUDE.md`), et la boucle approche du plafond de huit.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
