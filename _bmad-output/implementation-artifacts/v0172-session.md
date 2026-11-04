# Story v0172-session : la 0.17.3 — la durée d'inactivité réglée dans l'interface, exacte, remise à zéro par l'activité (#876, #891)

Status: ready-for-dev

*Fiche **reprise** le 2026-11-04 sur une décision du Project Lead qui fait tomber les trois prémisses de la conception validée de P3 à P6
(Change Log, entrée « Reprise ») : « J'aimerais que la durée d'inactivité soit configurée dans la partie admin de l'interface web et pas
par une variable d'environnement. J'aimerais que cette durée soit exactement celle configurée dans l'interface web et non pas un multiple
de celle-ci. J'aimerais que cette durée soit remise à zéro à chaque activité : mouvement de souris, etc. » Antérieurement : « 60 minutes :
15 est trop court : je me fais régulièrement éjecter alors que je suis en train de travailler et que je consulte des documents. » Le
changement d'exigence est porté par le **CR #891** (commentaire du 2026-11-04 : FR13 réécrit). Worktree `/home/gcorbaz/devel/kesh-0172s`,
branche `story/v0172-session`, tête de reprise `c547dc41`. Clé courte : `v0172s`. Registre : `v0172-choix-autonomes.md`, section
`v0172s` ; **en vigueur : C-v0172s-14, 17, 23, 24, 26 et C-v0172s-28 à 42** (C-1 à 13 : première conception, écartée ; C-15, 16, 20,
21, 22, 27 : remplacés à la reprise par C-28 et suivants ; C-18 : retiré ; C-19 et 25 : remplacés par C-26). Issues : **#876** (bug, P2),
**#891** (CR), en `refs` seulement. Numéros de ligne de `c547dc41` (le code applicatif est celui de `d7ec2ea2`, inchangé depuis). La
boucle de validation **repart à R1** (complète, Sonnet) : la conception est neuve, les passes P3 à P6 ont porté sur une autre.*

## Story

En tant que Comptable qui saisit une facture ou consulte des documents, je ne veux pas être renvoyé à la connexion tant que je travaille
dans Kesh ; en tant qu'utilisateur parti de son poste, je veux que la session se ferme **exactement** après la durée d'inactivité réglée ;
en tant qu'Administrateur, je veux régler cette durée **dans l'interface**, et que la restauration d'une sauvegarde complète ferme
réellement toutes les sessions.

## Contrainte

Production réelle sur le NAS du Project Lead (0.17.1, livres réels) : aucun essai hors base jetable (`kesh_v0172s`, `kesh_e2e_v0172s`, à
créer, migrations du worktree, seed `scripts/seed-dev-db.sql`, port 3174) ; aucune donnée réelle. **Une migration**, nécessaire et non
breaking (C-v0172s-40) : **88 → 89** (`ls crates/kesh-db/migrations/*.sql | wc -l`), DDL seul ; **le retour à la 0.17.1 (ou 0.17.2)
passe par la sauvegarde d'avant la mise à jour** — ou par le SQL qui retire la table **et** la ligne de `_sqlx_migrations` (§ *Migration
breaking policy*, P1). P2 sans objet (pas de bump `kesh_version_min_required`), P3 sans objet (aucun `DROP`/`RENAME`/`MODIFY`), **P5**
(ligne d'audit d'idempotence et compteurs recomptés), **P6** (couplage positionnel), **P7** (aucune écriture de données : rien à trier, le
détecteur le confirme), **P8** (ligne ajoutée à `migrations.sha384`). Aucune donnée réécrite hors des jetons de renouvellement : leur
échéance (`expires_at`) change de sens (§ *Conception*, 3), et la restauration complète les révoque (§ *Conception*, 8).

## Exigences de la reprise, et où chacune est tenue

| # | Exigence (prompt de reprise) | Tenue par |
|---|---|---|
| 1 | Réglage dans l'écran d'administration, en base, défaut 60, bornes justifiées, Administrateur seul, audit ; sort de `KESH_REFRESH_INACTIVITY_MINUTES` | Conception 1, 2 ; AC-1, AC-2, AC-8 |
| 2 | Durée exacte, sans double ni arrondi au jeton d'accès ; précision écrite | Conception 3, 4, 7 ; AC-3, AC-6 |
| 3 | Remise à zéro à chaque activité, un onglet vaut pour tous, aperçu PDF établi, sondes exclues | Conception 5, 6 ; AC-6 |
| 4 | Fin de session : le serveur refuse au-delà de N, le navigateur ramène à la connexion à l'échéance, saisie perdue dite | Conception 3, 7 ; AC-3, AC-6, AC-9 |
| 5 | Simplicité : aucun verrou, module armé ni suspension | Conception 5 (« ce qui n'y est pas ») ; C-v0172s-34 |
| 6 | Splitting : modules comptés, dérogation écrite | § *Modules* ; C-v0172s-41 |

## Modules (règle de splitting)

1. `crates/kesh-db` — migration `20261104000001_installation_settings.sql` ; `repositories/installation_settings.rs` (neuf) et
   `repositories/mod.rs` ; `repositories/refresh_tokens.rs` (`set_idle_deadline`, `was_rotated_within`, `revoke_all_in_tx`) ;
   `backup.rs` (table système exclue, commentaire `:33-37` et test `:851`) ; `test-schema/0001_schema_squash.sql` ; `migrations.sha384`.
2. `crates/kesh-api` — `config.rs`, `main.rs` (`:245-251`), `auth/jwt.rs`, `middleware/auth.rs`, `routes/auth.rs`, `routes/setup.rs`,
   `routes/admin.rs` (révocation à l'import), `routes/session_settings.rs` (neuf) et `routes/mod.rs`, `lib.rs`, `audit_labels.rs`,
   `README.md` ; tests.
3. `crates/kesh-i18n` — `locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
4. `frontend/src/lib/shared/utils` — `api-client.ts` (`tryRefresh()`, C-26) et son test.
5. `frontend/src/lib/app/stores` — `auth.svelte.ts` (champs d'inactivité de la charge utile, `hydrate()` → `tryRefresh()`) et
   `session-idle.svelte.ts` (neuf, le suivi d'activité), et leurs tests.
6. `frontend/src/lib/features/settings` — `settings.api.ts`, `settings.types.ts`, `SessionIdleSection.svelte` (neuf) et son test.
7. `frontend/src/routes` — `+layout.svelte` (racine : démarrage du suivi), `(app)/settings/+page.svelte` (la section),
   `login/+page.svelte` (le message de fin de session).
8. Configuration livrée et textes : `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`, `docs/migrations-idempotence-audit.md`,
   manuels FR (`.tex` et PDF), `CHANGELOG.md`, `_bmad-output/planning-artifacts/{prd,epics}.md`, `docs/kesh-specifications.txt`.

**Dérogation à la règle de splitting** (C-v0172s-41) : **huit** modules, dont sept de code. *Justification* : le serveur seul (modules 1 à
3) fermerait la session N minutes après la connexion quelle que soit l'activité, puisque seul le battement du navigateur prolonge
l'échéance ; le navigateur seul n'a rien à appeler. Les deux moitiés ne se publient pas l'une sans l'autre. *Atténuation* : le
développement se fait en **deux lots ordonnés** sur la branche — **lot S** (serveur : modules 1 à 3, avec la configuration livrée) puis
**lot N** (navigateur : modules 4 à 7, avec les manuels, le CHANGELOG et le PRD) — et **la revue de code se tient par lot** (un prompt de
revue par lot, chacun sur trois ou quatre modules). *Risque accepté* : une validation de fiche sur huit modules ; il est borné par la
simplicité de chaque moitié (un réglage, une échéance, un battement ; un module d'écoute sans verrou). Une modification de `repositories/`
et une migration imposent le **gate complet en cours de boucle** (`CLAUDE.md`, exception `kesh-db`).

## Ce que le code dit (c547dc41)

1. **L'inactivité vient d'une variable d'environnement.** `KESH_REFRESH_INACTIVITY_MINUTES`, défaut 15, bornes `[1, 1440]`, repli 15
   (`config.rs:735-755`) ; champ `Config::refresh_inactivity` (`:207`) ; paramètre de `from_fields_for_test` (`:450`, assertion
   `(0, 24 h]` `:474-476`), passé par 78 fichiers (`git grep -l from_fields_for_test -- crates | wc -l`) ; `make_test_config` (`:1635`) ;
   tests `config_refresh_inactivity_*` (`:2689-2721`) ; contrôle de cohérence au démarrage (`main.rs:245-251`). Configuration livrée :
   `.env.example:160` (commentaire `:159`), `docker-compose.yml:85`, `docker-compose.prod.yml:94` (`${…:-15}` : la variable est **toujours
   posée** dans un conteneur lancé par ces fichiers) ; `crates/kesh-api/README.md:41, :161, :179`.
2. **Elle est comptée depuis le dernier renouvellement, non depuis l'activité.** Connexion : ligne `refresh_tokens` à `now + inactivité`
   (`routes/auth.rs:254`) ; renouvellement : rotation (`:418`) et ligne neuve à `now + inactivité` (`:422`) ; changement de mot de passe
   (`:524`) ; assistant (`routes/setup.rs:288`). Rien d'autre ne touche `expires_at`.
3. **Le jeton d'accès ne porte aucune échéance d'inactivité.** `Claims` = `sub`, `role`, `company_id`, `iat`, `exp` (`auth/jwt.rs:40-52`) ;
   `decode` tolère 60 s (`:91`) ; le middleware ne lit pas la base sur le chemin JWT (`middleware/auth.rs:146-163`) ; `CurrentUser`
   (`:38-45`) est construit en littéral à cinq endroits (`api_key.rs` ×2, `helpers.rs`, `middleware/auth.rs` ×2) et dans deux tests
   (`services_motif.rs`, `services_rollout.rs`). Le cookie d'accès vit `jwt_expiry` (`routes/auth.rs:54`), celui de renouvellement
   `refresh_token_max_lifetime` sur le chemin `/api/v1/auth` (`:57-66`) : **une route sous `/api/v1/auth/` reçoit les deux cookies**.
4. **Le chemin 401 existe et se rejoue** : `request`/`requestRaw` appellent `refreshTokens()` (mutex d'onglet `refreshPromise`,
   `api-client.ts:67-69`, `:185-192`), qui efface et redirige vers `/login?reason=session_expired` sur échec (`:96-98`, `:106-108`,
   `:115-117`, `:176-177`), puis rejouent une fois.
5. **Deux onglets qui renouvellent ensemble déclenchent la détection de vol** : un jeton déjà tourné présenté de nouveau ⇒
   `revoke_all_for_user(…, "theft_detected")` (`routes/auth.rs:379-389`).
6. **Aucun écouteur d'activité.** `git grep -nE "addEventListener\('(keydown|pointermove|mousemove|pointerdown)" -- frontend/src`
   ne rend aucun site hors tests ; `<svelte:window onkeydown>` n'existe que pour des raccourcis clavier (`JournalEntryForm.svelte:305`,
   `(app)/+layout.svelte:293`, `(app)/journal-entries/+page.svelte:328`), sans rapport avec la session ; l'unique lecture périodique authentifiée est celle du bandeau
   de sauvegarde (`BackupScheduleBanner.svelte:33-34`, toutes les 30 min) ; `api-health.svelte.ts:102` interroge `/health`, non
   authentifié. `localStorage` n'est employé que pour purger d'anciennes clés (`auth.svelte.ts:153-157`, `:178-182`).
7. **L'aperçu des documents** est un `<iframe>` d'URL `blob:` pour un PDF (`DocumentPreview.svelte:92-97`), une `<img>` pour une image
   (`:85-90`) ; le PDF y est rendu par la visionneuse du navigateur.
8. **La restauration complète remplace `refresh_tokens` sans révoquer** (`routes/admin.rs:180-182`, `restore_tables_in_tx` `:422-425`) :
   les lignes actives de la sauvegarde sont réinsérées telles quelles (inchangé depuis la fiche précédente, § 9 de l'ancienne rédaction).
9. **Un précédent de réglage d'installation existe** : `_kesh_backup_schedule` (Story 31-1c) — table **système** (préfixe `_kesh_`), hors
   de `TABLES_TO_TRUNCATE` (`backup.rs:33-37`, test `backup_inventory_matches_schema` `:846-860`), donc ni exportée ni restaurée, sans quoi
   **toute sauvegarde antérieure deviendrait inimportable** (le contrôle de couverture exige l'égalité des inventaires de tables) ; réglage
   par `PUT` avec verrou optimiste et audit `admin.backup_schedule_updated` dans la même transaction (`repositories/backup_schedule.rs:173-240`) ;
   horodatages liés par l'application, jamais `NOW(3)` (`:16-19`). Un second précédent, `mcp_server_settings` (32-5a), rend **les défauts
   du code** tant qu'aucune ligne n'est enregistrée.
10. **L'écran Paramètres** (`/settings`, menu Administration → Paramètres, `routes/(app)/+layout.svelte:132`) porte déjà des sections
   modifiables par l'Administrateur seul (`(app)/settings/+page.svelte:38` `isAdmin`, `:346`, `:414`) ; ses appels vivent dans
   `lib/features/settings/settings.api.ts`.

## Conception retenue

1. **Le réglage, en base, dans une table système** (C-v0172s-29). Migration `20261104000001_installation_settings.sql` :
   `CREATE TABLE _kesh_installation_settings (id TINYINT UNSIGNED NOT NULL PRIMARY KEY DEFAULT 1, session_idle_minutes INT NOT NULL
   DEFAULT 60, version INT NOT NULL DEFAULT 1, updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
   CONSTRAINT chk_kesh_installation_settings_single_row CHECK (id = 1), CONSTRAINT chk_kesh_installation_settings_idle CHECK
   (session_idle_minutes BETWEEN 1 AND 1440))`, **sans `INSERT`** : la lecture rend le défaut du code (**60**, version 0) tant
   qu'aucune ligne n'existe (précédent `mcp_server_settings`) — la migration est du DDL pur, P7 n'a rien à trier, et le squash de test n'a
   besoin que du `CREATE TABLE`. En-tête de migration sur le modèle de `20261024000001_backup_schedule.sql` (table système, non breaking,
   retour par la sauvegarde, tracked-by-sqlx). Table **système** : ajoutée à l'exclusion de `backup.rs` (commentaire `:33-37`, liste du
   test `:851`) ; ni exportée, ni restaurée, ni vidée par la réinitialisation de démonstration ; **une restauration garde le réglage de la
   destination**, et les sauvegardes 0.17.x restent importables. Dépôt `installation_settings` : `get(pool) -> InstallationSettings {
   session_idle_minutes, version }` (défaut si absente) ; `update_session_idle_minutes(pool, expected_version, user_id, minutes)` —
   version 0 ⇒ `INSERT` (doublon ⇒ conflit), sinon `UPDATE … WHERE id = 1 AND version = ?` (0 ligne ⇒ conflit), `version + 1`, et
   l'audit **`admin.session_settings_updated`** (`before`/`after`, `{ "sessionIdleMinutes": … }`) **dans la même transaction**
   (précédent `backup_schedule::update_settings`). Constantes `SESSION_IDLE_MIN = 1`, `SESSION_IDLE_MAX = 1440`,
   `SESSION_IDLE_DEFAULT = 60`, miroir de la contrainte. **Bornes** : de 1 minute (plus court rendrait la saisie d'un seul champ risquée,
   et sert la recette) à 1440 (24 heures : une journée de travail ; au-delà, la session ne se fermerait plus en pratique, ce que FR13
   exclut) — ce sont aussi les bornes de la variable d'aujourd'hui, si bien qu'aucune valeur posée hier n'en sort.
2. **L'écran et la route** (C-v0172s-30). `GET` et `PUT /api/v1/admin/session-settings` dans le bloc `admin_routes` (`lib.rs:255`,
   `require_not_pat` `:502`) : Administrateur seul, clés d'API refusées. `GET` ⇒ `{ sessionIdleMinutes, version, min: 1, max: 1440 }` ;
   `PUT { sessionIdleMinutes, version }` ⇒ 400 `VALIDATION_ERROR` hors bornes ou non entier (rien d'écrit, aucun audit ; message
   i18n `admin-session-settings-error-out-of-domain`, repli égal à la valeur fr-CH, garde G9 de `textes_coherents.rs`), 409
   `OPTIMISTIC_LOCK_CONFLICT` sur version périmée, 200 et le nouvel état sinon. Inscrite au registre `audit_route_registry.rs` (statut
   audité) et à la liste de `admin_pat_denied_e2e`. L'écran : une section **« Session »** dans **Administration → Paramètres**
   (`/settings`), rendue pour l'Administrateur seul (`isAdmin`, `+page.svelte:38`) — un champ entier « Fermer la session après
   [ 60 ] minutes d'inactivité », bornes affichées, *Enregistrer*, message de succès, conflit de version traité comme l'e-mail de la
   société (`:61-80` : recharger et inviter à recommencer). Après enregistrement, la durée de **la session de l'Administrateur** est mise
   à jour aussitôt (`authState`), celle des autres sessions à leur prochaine activité (C-v0172s-38). Un Comptable ou une Consultation ne
   voit pas la section ; le `GET` leur rend 403.
3. **L'échéance, côté serveur : une seule valeur, exacte** (C-v0172s-32). `refresh_tokens.expires_at` **devient l'échéance
   d'inactivité** de la session (dernière activité + N), et le jeton d'accès en porte une copie dans une revendication **facultative
   `idle_exp`** (secondes Unix ; `#[serde(default)] Option<i64>` dans `Claims`, hors de `REQUIRED_SPEC_CLAIMS`). **Le middleware la
   contrôle strictement**, sans la tolérance de 60 s de `decode` : sur le chemin JWT, `if let Some(d) = claims.idle_exp { if
   Utc::now().timestamp() >= d { 401 } }` ; `CurrentUser` reçoit `idle_exp: Option<i64>` (sept littéraux à compléter, § *Code* 3, `None`
   pour les clés d'API). Écritures de l'échéance :
   - **connexion**, **assistant** (`setup.rs:288`) et **changement de mot de passe** (`auth.rs:524`) — gestes de l'utilisateur :
     `expires_at = now + N` (N lu par `installation_settings::get`), `idle_exp = expires_at` ;
   - **renouvellement** (`auth.rs:422`) : la ligne neuve **recopie** l'`expires_at` de l'ancienne — le renouvellement n'est pas une
     activité, il ne prolonge rien — et le jeton neuf porte `idle_exp` = cette même échéance ; le refus `expires_at < now` (`:399`)
     reste le contrôle ;
   - **battement d'activité** (§ 4) : `expires_at = now − âge + N`.
   Horodatages calculés par l'application (`Utc::now()`), jamais `NOW(3)` (précédent `backup_schedule.rs:16-19`). `jwt::encode` garde sa
   signature (jetons sans `idle_exp`, employés par les tests) ; une fonction `jwt::encode_session(user_id, role, company_id, secret,
   lifetime, idle_exp)` et une fonction `jwt::encode_claims(&Claims, secret)` partagée (pas de duplication) servent les sites de session.
   **Conséquence** : au-delà de l'échéance, aucune requête n'est acceptée — le jeton d'accès est refusé par `idle_exp` **quel que soit
   son `exp`**, le renouvellement par `expires_at < now`, le battement par les deux. Les jetons émis par une 0.17.x antérieure (sans
   `idle_exp`) restent acceptés jusqu'à leur `exp` (15 minutes au plus) : la mise à jour ne déconnecte personne.
4. **Le battement d'activité** (C-v0172s-33). `POST /api/v1/auth/activity` (routeur protégé : jeton d'accès valide, `idle_exp` compris ;
   clés d'API refusées par `ensure_not_pat`), corps `{ "idleMs": <entier ≥ 0> }` — **l'âge de la dernière activité** au moment de
   l'envoi, mesuré par le navigateur (§ 5) ; borné à `[0, N × 60 000]`. Sous `/api/v1/auth/`, la route reçoit le cookie de
   renouvellement, qui désigne la ligne de **cette** session. Une requête, une écriture : `refresh_tokens::set_idle_deadline(pool, token,
   user_id, deadline, now) -> bool` (`UPDATE refresh_tokens SET expires_at = ? WHERE token = ? AND user_id = ? AND revoked_at IS NULL AND
   expires_at > ?`) ; 0 ligne (cookie absent, ligne révoquée, expirée ou d'un autre utilisateur) ⇒ 401. Puis **un jeton d'accès réémis**
   avec les mêmes `sub`, `role`, `company_id` et **le même `exp`** que le jeton présenté, et le nouvel `idle_exp` — le battement ne
   prolonge pas le jeton d'accès, donc ni la fenêtre de rôle ou de société périmés (`middleware/auth.rs:56-71`), ni le rythme des
   renouvellements ; cookie d'accès d'âge `exp − now`. Réponse 200 `{ idleTimeoutMinutes, idleExpiresIn }` (secondes). **Aucune rotation,
   aucun jeton de renouvellement émis** : deux onglets qui battent ensemble ne déclenchent rien. Le battement passe par `apiClient.post` :
   sur 401 (jeton d'accès échu après une lecture de plus de 15 minutes), le chemin existant renouvelle et rejoue. Inscrit au registre
   `audit_route_registry.rs` comme route mutante **non auditée**, avec la raison (ni donnée comptable ni réglage : l'échéance d'une
   session, comme le renouvellement). `/me` et la réponse de connexion (`LoginResponse`, partagée par l'assistant, `setup.rs:307-314`)
   rendent aussi `idleTimeoutMinutes` et `idleExpiresIn` (le reste de `idle_exp`, `null` sans elle) ; la réponse du renouvellement aussi.
5. **Le navigateur : un module d'écoute, sans verrou** (C-v0172s-34, C-v0172s-35). `frontend/src/lib/app/stores/session-idle.svelte.ts`,
   démarré par le layout **racine** (`routes/+layout.svelte`) quand `authState.currentUser` devient non nul — il couvre donc `(app)` et
   `/onboarding` — et arrêté quand il redevient nul. Trois clés `localStorage` partagées par les onglets de l'origine :
   `kesh:session:lastActivity`, `kesh:session:reportedActivity`, `kesh:session:heartbeatAt` (millisecondes). Écouteurs passifs, en phase
   de capture, sur `window` : `keydown`, `pointerdown`, `pointermove` (**seulement si `screenX`/`screenY` diffèrent du dernier
   mouvement vu** : le navigateur émet des mouvements fictifs, à coordonnées inchangées, quand la page bouge sous un pointeur immobile — un
   bandeau mis à jour par une sonde de fond ne doit pas compter), `wheel`, `touchstart`, `focus` ; sur `document` : `scroll` (capture, pour
   tout élément défilant) et `visibilitychange` vers `visible` ; `blur` de `window` quand le focus passe dans un `<iframe>` de la page
   (§ 6). Chaque événement : **d'abord** le contrôle de l'échéance (une activité au-delà de l'échéance ne ranime pas la session : elle la
   ferme), puis `lastActivity = now`, écrite au plus une fois par seconde. Une **minuterie d'une seconde** par onglet : (a) `deadline =
   max(lastActivity) + N × 60 000` ; `now ≥ deadline` ⇒ fin de session (§ 7) ; (b) sinon, si `lastActivity > reportedActivity` et `now −
   heartbeatAt ≥ 15 000`, l'onglet écrit `heartbeatAt = now` et `reportedActivity = lastActivity`, puis envoie le battement avec `idleMs =
   now − lastActivity` ; sur réponse, `N` et l'échéance prennent les valeurs du serveur ; sur échec (réseau, 5xx), `reportedActivity`
   reprend sa valeur précédente et l'envoi se retente au pas suivant ; sur 401 définitif, le chemin 401 a déjà effacé et redirigé. **Au
   démarrage** (connexion ou rechargement), le module concilie avec le serveur : `serverLast = now + idleExpiresIn × 1000 − N × 60 000`,
   `lastActivity = max(stockée, serverLast)`, `reportedActivity = max(stockée, serverLast)` — à la connexion `idleExpiresIn = N × 60`,
   donc `serverLast = now`. Le chargement d'une page ne compte pas comme activité. *Ce qui n'y est pas* : aucun `navigator.locks`, aucun
   `BroadcastChannel` neuf, aucun onglet maître, aucun module « armé », aucune suspension. Deux onglets qui envoient le même battement à
   la même seconde ne font qu'écrire deux fois la même échéance, à quelques millisecondes près. Si `localStorage` est indisponible
   (navigateur qui le refuse), le module garde ses trois valeurs en mémoire : l'activité ne se partage plus entre onglets, et un onglet
   laissé sans geste se ferme à sa propre échéance pendant qu'un autre travaille (déclaré au manuel d'administration).
6. **L'aperçu d'un document** (C-v0172s-36). Ce qui est observable, établi au code et à la spécification des événements : les
   événements de la visionneuse PDF d'un `<iframe>` (défilement, clic, clavier dans le PDF) **ne parviennent pas** à la page de Kesh (le
   document du cadre n'est pas celui de la page, et la visionneuse du navigateur n'expose pas les siens) ; la page voit le pointeur
   **entrer dans le cadre et en sortir** (`pointermove` jusqu'au bord, puis au retour), et le **clic qui donne le focus au cadre** (`blur`
   de `window` avec `document.activeElement` = l'`<iframe>`), qui compte. Une image (`<img>`) est dans la page : tout y compte. **Donc** :
   lire un PDF dans l'aperçu, sans geste hors du cadre, plus longtemps que N, ferme la session (avec le défaut de 60 minutes, une lecture
   continue d'une heure sans toucher la page). *Écarté* : compter comme activité tout le temps où le focus reste dans le cadre — un poste
   quitté avec le PDF au premier plan ne se fermerait jamais, ce que FR13 interdit. Déclaré au manuel utilisateur.
7. **La fin de session, et sa précision** (C-v0172s-37). À l'échéance, chaque onglet (même instant, puisque `lastActivity` est partagée)
   arrête le module, appelle `authState.clearSession({ broadcast: false })` et `window.location.replace('/login?reason=idle&minutes=<N>')` ;
   la page de connexion affiche **« Session fermée après N minutes d'inactivité. »** (clé `login-session-idle`, variable Fluent `$minutes`,
   quatre locales ; `minutes` absent ou non entier ⇒ le message existant de session expirée). **Ce qui est perdu** : tout ce qui n'était
   pas enregistré, dans **tous** les onglets de Kesh du navigateur (formulaire en cours, brouillon non enregistré, filtre non appliqué) ;
   aucune alerte ne précède la fermeture (C-v0172s-37 : une alerte « encore une minute » serait un CR distinct). **Précision** : le
   navigateur ferme à `dernière activité + N` **à la seconde près** (activité notée au pas d'une seconde, minuterie d'une seconde) ; le
   serveur refuse **à partir de la même échéance, à la seconde près** (`idle_exp` en secondes entières, arrondi vers le bas : jamais plus
   tard que l'échéance), parce que le battement lui transmet l'**âge exact** de la dernière activité. Deux écarts, et seulement deux :
   (a) l'activité est transmise au plus **15 secondes** après qu'elle a eu lieu — si le navigateur est fermé, mis en veille ou perd le
   réseau dans ces 15 secondes, le serveur ne l'a pas reçue et ferme jusqu'à 15 secondes plus tôt (l'utilisateur, alors, n'est plus là) ;
   un battement qui croise un renouvellement de la même session peut de même être perdu (la ligne neuve recopie l'échéance d'avant), écart
   borné aux mêmes 15 secondes et rattrapé au battement suivant ; (b) **un onglet en arrière-plan** : le navigateur ralentit ses minuteries
   (jusqu'à une fois par minute), si bien que l'écran de connexion peut n'apparaître qu'au retour sur l'onglet — **le serveur, lui, a
   refusé dès l'échéance**, et le retour sur l'onglet contrôle l'échéance avant de compter comme activité. Aucun double, aucun arrondi à la
   durée du jeton d'accès : `KESH_JWT_EXPIRY_MINUTES` ne règle plus que le rythme des renouvellements.
8. **Gardé : la restauration complète révoque toutes les sessions** (C-v0172s-23, inchangé). Dans la transaction d'import, juste avant le
   `SELECT MIN(id) FROM users WHERE role = 'Admin'` (`routes/admin.rs:488-492`) — après `restore_tables_in_tx` (`:422-425`), la garde de
   comptage (`:436`), les rejeux 5-bis (`:452-`) et 5-ter (`:464-`), avant le refus « aucun compte Admin » (`:493-497`), la réparation
   5-quater, l'audit et le commit — `refresh_tokens::revoke_all_in_tx(&mut tx, "password_change")` (`UPDATE refresh_tokens SET revoked_at
   = NOW(3), revoked_reason = ? WHERE revoked_at IS NULL`). Motif `"password_change"` : la contrainte `chk_refresh_tokens_revoked_reason`
   (`20260406000001_refresh_tokens_revoked_reason.sql` ; squash `:1187`) n'admet que cinq valeurs ; précédents `routes/auth.rs:964-969` et
   `auth/bootstrap.rs:355-364` ; la distinction est portée par l'audit `admin.full_import` ; `"admin_disable"` écarté. Sonde des lecteurs
   du motif : `git grep -nE '"(logout|rotation|password_change|admin_disable|theft_detected)"' -- crates/kesh-api/src` (seul `auth.rs:379`
   compare, à `"rotation"`). Aucun traitement de la transaction ne lit `refresh_tokens` (`git grep -n refresh_tokens -- crates/kesh-db/src
   crates/kesh-api/src/routes/admin.rs`). Commentaire `admin.rs:180-182` mis à jour. *Pourquoi toujours nécessaire* : une ligne restaurée
   porte l'échéance d'inactivité de l'export ; tant qu'elle n'est pas passée, son cookie se renouvellerait. Avec la révocation, le premier
   battement ou la première requête après l'échéance du jeton d'accès rend 401 ; `sessionInvalidated: true` (`admin.rs:267`) est vrai.
9. **Gardé : la grâce de rotation** (C-v0172s-17, inchangé). Dans `refresh`, avant la détection de vol (`:379`), un jeton révoqué pour
   `"rotation"` depuis moins de 30 secondes rend `401 INVALID_REFRESH_TOKEN` **sans** `revoke_all_for_user`, mesuré à l'horloge de la base
   (`refresh_tokens::was_rotated_within(pool, token, secs)` : `… revoked_reason = 'rotation' AND revoked_at > NOW(3) - INTERVAL ? SECOND`,
   la même horloge qui a écrit `revoked_at = NOW(3)`, `refresh_tokens.rs:114`). La rotation n'a lieu qu'à l'échéance du jeton d'accès
   (toutes les 15 minutes d'une session active) ; deux onglets peuvent encore y présenter le même jeton.
10. **Gardé : la course entre onglets, résiduelle** (C-v0172s-24, inchangé sur le fond). Si deux onglets envoient leur `POST
   /auth/refresh` avec le même jeton avant que la réponse du premier n'ait reposé les cookies, le second reçoit 401 et **cet onglet**
   revient à la connexion (motif `session_expired`) ; l'autre et les autres appareils continuent. Le battement ne l'aggrave pas (aucune
   rotation) ; il la raréfie même, une seule requête par 15 secondes partant de tous les onglets réunis.
11. **Gardé : le renouvellement au démarrage, forme minimale** (C-v0172s-26, inchangé). `tryRefresh(): Promise<boolean>` extrait de
   `doRefresh` (`api-client.ts:85-178`), sous le même mutex (`refreshPromise`), sans effacement ni redirection sur échec ; le chemin 401
   efface et redirige lui-même (N requêtes rejetées sur un renouvellement échoué : N effacements, sans gravité) ; branche « refresh réussi,
   `/me` en échec » gardée (`true`). `hydrate()` (`auth.svelte.ts:209-316`, branche 401 `:261-267`), sur 401 de `/me`, appelle une fois
   `tryRefresh()` (import statique : le cycle `auth.svelte.ts:17` → `vat-rates` → `api-client.ts:15` existe déjà et reste bénin), juge le
   succès sur `authState.currentUser !== null`, et n'efface ni ne redirige ni ne diffuse sur échec (`hooks.client.ts:22` l'appelle sur
   toute page, `/login` comprise). Sans lui, un rechargement après l'échéance du **jeton d'accès** (15 minutes) mais avant celle de la
   **session** renverrait à la connexion. *Nouveau* : après l'échéance de la session, `hydrate()` échoue (le renouvellement est refusé) et
   le layout mène à `/login` **sans** le message d'inactivité (le code du refus est le même pour tous les refus, et doit le rester,
   `all_refresh_error_codes_are_identical`) ; déclaré.
12. **`KESH_REFRESH_INACTIVITY_MINUTES` est ignorée** (C-v0172s-31). `from_env` ne la lit plus comme durée ; si elle est posée et non
   vide, un avertissement au démarrage : « KESH_REFRESH_INACTIVITY_MINUTES est ignorée depuis la 0.17.3 : la durée d'inactivité se règle
   dans Administration → Paramètres, section Session. Retirez la ligne de votre .env ou de votre docker-compose. » — texte produit par une
   fonction pure `legacy_inactivity_warning(valeur: Option<&str>) -> Option<String>`, testée. **Pas d'amorçage du réglage depuis la
   variable** : `.env.example:160` et les deux `docker-compose` posaient 15 ; amorcer donnerait 15 à presque toutes les installations, là
   où le Project Lead a fixé 60. Le champ `Config::refresh_inactivity` (`:207`, `:361`, `:510`, `:1186`, `:1635`) et le contrôle
   `main.rs:245-251` sont **retirés** ; les trois tests `config_refresh_inactivity_*` (`:2689-2721`) sont remplacés par ceux de
   l'avertissement (raison au Change Log). **Le paramètre de `from_fields_for_test` (`:450`) est gardé et ignoré** (renommé
   `_refresh_inactivity_ignored`, doc « ignoré depuis la 0.17.3 ; la durée vit en base ») et son assertion `:474-476` retirée : le retirer
   toucherait 78 fichiers de tests par édition manuelle (consigne 14), pour aucun comportement ; **issue demandée** pour le retirer en un
   lot mécanique. Retirée de `.env.example` (`:159-160`, remplacée par un commentaire qui renvoie à l'écran), de `docker-compose.yml:85` et
   `docker-compose.prod.yml:94`. Une installation qui la pose encore (ancien `docker-compose`, `.env` recopié) démarre normalement, avec
   l'avertissement, et prend la valeur de l'écran (60 tant que rien n'est enregistré). `KESH_JWT_EXPIRY_MINUTES` reste une variable
   (C-v0172s-42) ; **le contournement `KESH_JWT_EXPIRY_MINUTES=480` publié sur #876 est à retirer** : il ne prolonge plus la session
   (l'échéance d'inactivité s'applique quel que soit le jeton d'accès) et porterait à 8 heures la fenêtre d'un rôle ou d'une société
   périmés.
13. **Remplacé** (C-v0172s-28) : la règle du double (C-16, C-22), la borne « entre `N − jwt` et `N` après la dernière requête » (C-21),
   l'option `background` des lectures de fond (C-20 : sa propriété — **les sondes de fond ne comptent pas** — est désormais tenue **par
   construction**, seul le battement prolongeant l'échéance, et aucune requête ne passant par le module d'écoute ; le bandeau de sauvegarde
   et ses fichiers ne sont plus touchés), la dérogation à six modules (C-27). *Écartée* : la première conception (C-1 à 13).

## Acceptance Criteria

**AC-1 — Le réglage, son écran côté serveur, son audit.** `crates/kesh-db` (tests du dépôt, `#[sqlx::test(migrations = "./test-schema")]`) :
table vide ⇒ `get` rend `{ 60, version 0 }` ; `update(…, 0, user, 90)` ⇒ `{ 90, version 1 }` et **une** ligne d'audit
`admin.session_settings_updated` (`before.sessionIdleMinutes = 60`, `after = 90`) ; `update(…, 0, …)` une seconde fois ⇒ conflit ;
`update(…, 1, …)` ⇒ version 2 ; les contraintes refusent `id = 2`, `0` et `1441` (`INSERT`/`UPDATE` bruts, précédent
`backup_schedule.rs:350-362`). `crates/kesh-api/tests/session_settings_e2e.rs` (neuf) : `GET` Admin ⇒ 200 `{ sessionIdleMinutes: 60,
version: 0, min: 1, max: 1440 }` ; Comptable et Consultation ⇒ 403 ; clé d'API ⇒ 403 ; `PUT 0`, `PUT 1441`, `PUT 1.5` ⇒ 400, rien
d'écrit, aucun audit ; `PUT 30` ⇒ 200, audit ; version périmée ⇒ 409. **Rouges avant correctif** (route et table absentes).

**AC-2 — La variable ignorée.** Tests unitaires de `legacy_inactivity_warning` : `None` ⇒ `None` ; `Some("")` ⇒ `None` ; `Some("15")` ⇒
message qui nomme la variable et l'écran ; `from_env` avec la variable posée démarre (la durée n'est plus un champ de `Config`). Les trois
tests `config_refresh_inactivity_*` retirés, raison au Change Log ; sonde `git grep -nE 'refresh_inactivity|REFRESH_INACTIVITY' -- crates
frontend scripts .env.example docker-compose*.yml` rendue au Dev Agent Record, chaque site restant justifié (le paramètre ignoré, le
message d'avertissement, le CHANGELOG historique). **Rouge avant correctif** (la fonction n'existe pas).

**AC-3 — L'échéance exacte, côté serveur.** `crates/kesh-api/tests/session_idle_e2e.rs` (neuf), réglage posé par le dépôt :
(a) **connexion** avec N = 7 : la ligne porte `expires_at` à `now + 7 min` (± 2 s) et le jeton d'accès (décodé par le secret de test)
`idle_exp` = `expires_at` à la seconde ; (b) **middleware** : un jeton `encode_session` dont `exp` est dans 15 minutes et `idle_exp`
dans le passé d'une seconde ⇒ **401** sur `GET /api/v1/auth/me` ; `idle_exp` dans le futur ⇒ 200 ; un jeton `jwt::encode` sans `idle_exp`
⇒ 200 (compatibilité) ; (c) **renouvellement** : ligne à `expires_at = now + 5 min` ⇒ `POST /auth/refresh` 200, la ligne neuve porte le
**même** `expires_at` (à la milliseconde) et le jeton neuf le même `idle_exp` ; (d) **battement** avec N = 7 et `idleMs = 10 000` : 200,
`expires_at` = `now − 10 s + 7 min` (± 2 s), réponse `idleTimeoutMinutes = 7`, cookie d'accès réémis dont `idle_exp` = la nouvelle
échéance ; `idleMs = 999 999 999` ⇒ borné (échéance = `now`, à ± 2 s) ; `idleMs = -1` ⇒ 400 (désérialisation) ; (e) le jeton réémis par le
battement a **le même `exp`** que le jeton présenté ; (f) cookie de renouvellement d'un **autre** utilisateur ⇒ 401, sa ligne inchangée ;
ligne révoquée ou expirée ⇒ 401 ; clé d'API ⇒ 403 ; (g) **au-delà de l'échéance** (ligne reculée d'une minute dans le passé, `idle_exp`
passé) : battement ⇒ 401, `POST /auth/refresh` ⇒ 401, aucune ligne prolongée. **Tests existants qui changent**, raison au Change Log : ceux
qui lisent l'`expires_at` d'une ligne neuve de renouvellement ou de connexion (`git grep -nE 'expires_at|minutes\(15\)' --
crates/kesh-api/tests/auth*.rs crates/kesh-api/tests/setup*.rs`, chaque site trié au Dev Agent Record) ; les tests de tolérance
(`middleware/auth.rs:352`, `auth_e2e.rs:776`) **restent verts sans modification** (jetons sans `idle_exp`). **Rouges avant correctif** :
(a) à (g).

**AC-4 — Gardé : la grâce de rotation** (ancien AC-3). `auth_e2e.rs` : (a) **nouveau** — refresh, puis rejeu aussitôt de l'ancien jeton :
`401 INVALID_REFRESH_TOKEN`, la ligne neuve reste active ; (b) `refresh_replay_after_rotation_revokes_all` (`:1066-1116`) **modifié** :
avant le rejeu, `UPDATE refresh_tokens SET revoked_at = NOW(3) - INTERVAL 60 SECOND WHERE token = ?` ; détection de vol inchangée ;
(c) `all_refresh_error_codes_are_identical` (`:1647-`) : cas 3 par la grâce, **cas 4** ajouté (rejeu reculé de 60 s, chemin du vol), les
quatre codes identiques. **Rouge avant correctif** : (a) ; (b), (c) verts avant et après.

**AC-5 — Gardé : la restauration complète révoque tout** (ancien AC-4). `admin_full_import_e2e.rs` : (a) export fait après une connexion,
puis import : `SELECT COUNT(*) FROM refresh_tokens WHERE revoked_at IS NULL` = 0 ; les lignes auparavant actives portent
`revoked_reason = 'password_change'`, les autres gardent leur motif ; un `POST /auth/refresh` et un `POST /auth/activity` avec le cookie
d'une ligne restaurée rendent 401 ; (b) archive **sans Admin** (refus `admin.rs:493-497`, après le point de révocation) : l'import échoue,
la transaction est annulée, les lignes actives de la destination restent actives. **Et** : le réglage `_kesh_installation_settings` de
la destination (posé à 30 avant l'export, à 45 avant l'import) vaut **45** après l'import (table système, non restaurée).
`sessionInvalidated: true` inchangé (`admin_full_import_e2e.rs:326`, `admin_backup_e2e.rs:814`). **Rouges avant correctif** : (a) ; le
volet du réglage aussi (table absente).

**AC-6 — Le suivi d'activité, dans le navigateur.** `session-idle.svelte.test.ts` (neuf ; `vi.useFakeTimers()`, qui simule `Date` et
`setInterval` ; `localStorage` de jsdom vidé à chaque cas ; `apiClient.post` simulé), N = 2 : (1) **chaque activité remet à zéro** :
pour `keydown`, `pointerdown`, `pointermove` (coordonnées neuves), `wheel`, `touchstart`, `scroll` (sur un élément, capture),
`visibilitychange` vers `visible`, `focus`, et `blur` de `window` avec un `<iframe>` focalisé — avancer 90 s, émettre l'événement, avancer
90 s : session ouverte ; (2) **mouvement fictif** : deux `pointermove` aux mêmes `screenX`/`screenY` — le second ne compte pas ; un `blur`
sans `<iframe>` focalisé ne compte pas ; (3) **échéance exacte** : sans activité, à `N × 60 000 − 1 000` ms, `clearSession` non appelée ;
à `N × 60 000` (+ le pas), `clearSession({ broadcast: false })` et `location.replace('/login?reason=idle&minutes=2')` ; (4) **activité après
l'échéance** (minuterie arrêtée, événement à l'échéance + 500 ms) : fermeture, `lastActivity` non réécrite ; (5) **battement régulé** :
`pointermove` toutes les 100 ms pendant 60 s ⇒ au plus 5 battements, chacun avec `idleMs = now − lastActivity` (≤ 1 000) ; aucune activité
⇒ aucun battement ; (6) **un onglet vaut pour tous** : `localStorage.setItem('kesh:session:lastActivity', …)` par « un autre onglet »
repousse l'échéance de celui-ci ; `heartbeatAt` récent écrit par l'autre ⇒ pas de second battement dans les 15 s ; (7) réponse du
battement `idleTimeoutMinutes = 5` ⇒ l'échéance se calcule sur 5 ; (8) battement en échec réseau ⇒ pas de fermeture, nouvel envoi au pas
de 15 s suivant ; (9) **les sondes ne comptent pas** : un `apiClient.get` déclenché par minuterie ne modifie pas `lastActivity` ;
(10) `stop()` retire tous les écouteurs (un `keydown` après `stop()` n'écrit rien) et la minuterie ; (11) démarrage : `idleExpiresIn = 30`
avec N = 2 ⇒ `lastActivity = now − 90 s` (conciliation), fermeture à 30 s. **Rouges avant correctif** (le module n'existe pas).

**AC-7 — Gardé : le renouvellement au démarrage** (ancien AC-5 bis). `auth.svelte.test.ts` : (1) `/me` 401, refresh 200, `/me` 200 ⇒
`currentUser` peuplé, **champs d'inactivité compris** ; (2) refresh 401 ⇒ `currentUser` nul, `location.replace` non appelé ; (3) un chemin
401 en vol et `hydrate()` ⇒ un seul `POST /auth/refresh`. Test existant qui change : `auth.svelte.test.ts:147-163` (deux appels au lieu
d'un). `api-client.test.ts` : les six tests du chemin 401 (`:205, :246, :270, :327, :488, :567`) restent verts. **Rouges avant correctif** :
(1), (3).

**AC-8 — L'écran.** `SessionIdleSection.test.ts` (neuf) : rend la valeur lue (60) et les bornes ; `0`, `1441`, `abc` ⇒ erreur de saisie,
aucun `PUT` ; `45` ⇒ `PUT { sessionIdleMinutes: 45, version }`, message de succès, `authState` porte 45 ; 409 ⇒ rechargement et message de
conflit. `settings-page.test.ts` : la section est rendue pour un Admin, **absente** pour un Comptable et une Consultation (aucun `GET`
`/admin/session-settings` pour eux). Captures (§ *Captures*). **Rouges avant correctif.**

**AC-9 — Le message de fin de session.** `login` (test de page) : `?reason=idle&minutes=60` ⇒ « Session fermée après 60 minutes
d'inactivité. » ; `?reason=idle&minutes=x` ⇒ le message existant de session expirée ; `?reason=session_expired` inchangé. Clé
`login-session-idle` dans les quatre locales, ownership vérifiée par `npm run lint-i18n-ownership`. **Rouge avant correctif.**

**AC-10 — Recette sur une installation de dev** (`kesh_v0172s`, port 3174, `KESH_JWT_EXPIRY_MINUTES=1`, réglage porté à **2** minutes par
l'écran ; chronomètre, déroulé et heures au Dev Agent Record) : (a) bouger la souris sans cliquer pendant 3 min, puis enregistrer un
brouillon : réussite ; (b) ne rien faire : l'écran de connexion apparaît **à 2 min 00 s ± 2 s** de la dernière activité, avec le message ;
un `curl` rejouant le cookie d'accès capturé juste avant ⇒ 401 ; (c) deux onglets, activité dans le seul onglet B pendant 3 min : l'onglet
A reste connecté ; (d) activité continue pendant 90 s (jeton d'accès échu), puis recharger : connecté ; (e) aperçu PDF d'un document de
démonstration, défilement **dans** le cadre seulement : fermeture à 2 min (constat déclaré, C-v0172s-36) ; (f) restauration complète d'un
export de la séance : plus aucune ligne active dans `refresh_tokens`, le réglage vaut toujours 2 ; (g) le journal d'audit montre
« Durée d'inactivité de la session modifiée » avec avant et après ; (h) backend démarré avec `KESH_REFRESH_INACTIVITY_MINUTES=15` :
l'avertissement apparaît, le réglage de l'écran s'applique.

**AC-11 — Les textes disent vrai.** § *Manuel* (PDF compris), § *Configuration livrée*, § *PRD*, § *CHANGELOG*.

## Tests et mutations

| # | Test | Attendu avant correctif |
|---|---|---|
| T1 | dépôt et route du réglage (AC-1) | rouge |
| T2 | avertissement de la variable (AC-2) | rouge |
| T3 | échéance serveur : connexion, middleware, renouvellement, battement (AC-3) | rouge |
| T4 | grâce de rotation (AC-4) | (a) rouge ; (b), (c) verts |
| T5 | restauration complète (AC-5) | rouge |
| T6 | suivi d'activité (AC-6) | rouge |
| T7 | renouvellement au démarrage (AC-7) | (1), (3) rouges ; (2) vert |
| T8 | écran (AC-8) et message (AC-9) | rouges |

**Mutations nommées** — sous le verrou de gate ; après une restauration par `git checkout --`, toucher le fichier (cargo garde sinon le
binaire muté) :
- **M1** — le contrôle `idle_exp` retiré du middleware : T3 (b) rougit. **M1 bis** — contrôle avec la tolérance de 60 s : T3 (b) rougit.
- **M2** — le renouvellement prolonge (`now + N`) au lieu de recopier : T3 (c) rougit.
- **M3** — le battement ignore `idleMs` : T3 (d) rougit. **M3 bis** — sans borne haute : T3 (d) (cas 999 999 999) rougit.
- **M4** — le battement émet un jeton à `exp = now + jwt` : T3 (e) rougit.
- **M5** — `set_idle_deadline` sans `user_id` : T3 (f) rougit.
- **M6** — grâce à 0 s : T4 (a) rougit ; **M6 bis** — grâce sans borne : T4 (b) rougit.
- **M7** — révocation de l'import retirée : T5 (a) rougit ; **M7 bis** — révocation sur le pool avant la transaction : T5 (b) et (a)
  rougissent.
- **M8** — le PUT sans contrôle de bornes : T1 rougit (la contrainte rend une erreur de base, non 400) ; **M8 bis** — audit retiré : T1
  rougit.
- **M9** — le module enregistre l'activité **avant** de contrôler l'échéance : T6 (4) rougit.
- **M10** — `pointermove` sans comparaison de coordonnées : T6 (2) rougit.
- **M11** — activité gardée en mémoire seulement (sans `localStorage`) : T6 (6) rougit.
- **M12** — battement sans régulation : T6 (5) rougit ; **M12 bis** — `idleMs` envoyé à 0 : T6 (5) rougit.
- **M13** — `hydrate()` sans `tryRefresh()` : T7 (1) rougit ; **M13 bis** — `tryRefresh()` hors du mutex : T7 (3) rougit.
- **M14** — la section rendue sans condition de rôle : T8 (page) rougit.

## Tâches

**Lot S — serveur** (modules 1 à 3, configuration livrée)

1. Constater rouges T1, T2, T3, T4 (a), T5 sur `c547dc41` ; T4 (b), (c) verts.
2. `crates/kesh-db` : migration (en-tête : table système, non breaking, retour par la sauvegarde, tracked-by-sqlx, DDL seul) ; ligne de
   `docs/migrations-idempotence-audit.md` et compteurs **recomptés** (en-tête de section, ligne `Total`, partition) ; ligne de
   `migrations.sha384` ; `CREATE TABLE` au squash `test-schema/0001_schema_squash.sql` (garde `test_schema_guard.rs`) ; exclusion de
   `backup.rs` ; **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et chaque site inspecté (dont
   `assert_eq!(total, …)` de `migrations_upgrade_path.rs`) ; **P7** : `every_data_backfill_migration_is_triaged` vert sans entrée (DDL
   seul) ; dépôt `installation_settings` ; `refresh_tokens` (`set_idle_deadline`, `was_rotated_within`, `revoke_all_in_tx`).
3. `crates/kesh-api` : `jwt.rs` (`idle_exp`, `encode_session`, `encode_claims`), `middleware/auth.rs` (contrôle strict, `CurrentUser`),
   `routes/auth.rs` (connexion, renouvellement qui recopie, grâce, mot de passe, `/me`, battement), `routes/setup.rs`, `routes/admin.rs`
   (révocation), `routes/session_settings.rs`, `lib.rs`, `audit_labels.rs`, `config.rs` et `main.rs` (C-v0172s-31) ; les sept littéraux
   de `CurrentUser` ; registres `audit_route_registry.rs` et `admin_pat_denied_e2e` ; `crates/kesh-api/README.md` (§ *Configuration
   livrée*).
4. `crates/kesh-i18n` : `admin-session-settings-error-out-of-domain`, `audit-log-action-admin-session-settings-updated`, et les clés du
   lot N, quatre locales (valeurs fr-CH = replis, G9).
5. Configuration livrée (§ *Configuration livrée*).
6. Mutations M1 à M8 bis ; gate complet (repository et migration touchés) ; commit du lot ; **revue de code du lot S**.

**Lot N — navigateur** (modules 4 à 7, textes)

7. Constater rouges T6, T7 (1), (3), T8.
8. `api-client.ts` (`tryRefresh()`), `auth.svelte.ts` (`LoginPayload` : `idleTimeoutMinutes`, `idleExpiresIn` ; `hydrate()`) — et la
   lecture de `/me` dans `tryRefresh` (type en ligne de `doRefresh`, `api-client.ts:132-137`) ; `session-idle.svelte.ts` ;
   `routes/+layout.svelte` ; `settings.api.ts`, `settings.types.ts`, `SessionIdleSection.svelte`, `(app)/settings/+page.svelte` ;
   `login/+page.svelte`. Vérifier qu'aucun test qui monte le layout racine ne laisse tourner la minuterie (`git grep -ln "routes/+layout"
   -- frontend/src`) et qu'aucune spec Playwright ne compte toutes les requêtes (`git grep -n "page.on('request'" -- frontend/tests`).
9. Manuels et PDF (§ *Manuel*), CHANGELOG, PRD et copies.
10. Mutations M9 à M14 ; recette AC-10 ; captures.
11. Gates (ci-dessous) ; **revue de code du lot N**.

**Gates** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh` (sous `flock
/home/gcorbaz/devel/kesh-gate-logs/.gate.lock` et `mem-guard`, base remise à zéro avant), `cd frontend && npm run check && npm run
lint-i18n-ownership && npm run test:unit && npm run build`, E2E complet au **dernier commit de code** (7 à 10 échecs attendus,
`docs/testing.md`), `make fr` dans `docs/manual/`. Ne déclarer que ce qui a tourné.

## Configuration livrée

- `.env.example:159-160` : la ligne `KESH_REFRESH_INACTIVITY_MINUTES=15` et son commentaire **retirés**, remplacés par : « # La durée
  d'inactivité de la session se règle dans Kesh (Administration → Paramètres, section Session ; 60 minutes par défaut). La variable
  KESH_REFRESH_INACTIVITY_MINUTES est ignorée depuis la 0.17.3. » ; `:128-130` : `KESH_JWT_EXPIRY_MINUTES` = « durée du jeton d'accès,
  donc rythme des renouvellements ; ne fixe pas la fin de session », `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` = « durée de vie du cookie de
  renouvellement, reposé à chaque renouvellement ».
- `docker-compose.yml:85`, `docker-compose.prod.yml:94` : la ligne retirée.
- `crates/kesh-api/README.md` : `:41` retirée (renvoi à l'écran), `:40` (« durée de vie du cookie de renouvellement, reposé à chaque
  renouvellement », non « lifetime absolu »), `:160` (`idle_exp`, contrôlé sans tolérance), `:161` (« échéance d'inactivité réglée en
  base, 60 min par défaut, prolongée par le battement `POST /api/v1/auth/activity` ; le renouvellement ne la prolonge pas »), `:179` (la
  durée par défaut est désormais de 60 minutes : le risque du jeton stocké en clair est plus grand — #890, C-v0172s-14), `:16`
  (`jwt.rs` : `idle_exp`).

## PRD (CR #891)

- `_bmad-output/planning-artifacts/prd.md:391` : « FR13 : Le système renouvelle silencieusement la session et la termine après une durée
  d'inactivité **réglée par l'Administrateur dans l'interface** (60 minutes par défaut), **appliquée exactement** ; toute activité de
  l'utilisateur dans Kesh (clavier, souris, défilement, toucher, retour sur l'onglet) la remet à zéro — CR #891 ».
- `prd.md:280` : « Authentification JWT avec refresh silencieux, fin de session après la durée d'inactivité réglée dans l'interface (60 min
  par défaut, CR #891) ».
- `epics.md:63` : à l'identique de `prd.md:391`.
- `epics.md:616` : « **Given** aucune activité pendant la durée réglée par l'Administrateur (60 minutes par défaut, Administration →
  Paramètres), **When** une requête ou un renouvellement, **Then** refus 401 et retour à la connexion (CR #891) ».
- `docs/kesh-specifications.txt` (non engendré : `git log --format=%s -3 -- docs/kesh-specifications.txt` rend des commits d'epics, aucun
  générateur) : `:278`, `:389`, `:2144`, `:2690` mis à jour de même, avec le renvoi à #891 ; hors classe (inchangés) : `:44`, `:71`,
  `:219`, `:374`, `:1439`, `:1453`, `:2129`, `:2625` (installation et prise en main en 15 minutes), `:392`, `:2147`, `:2691`
  (limitation des connexions), `:806`, `:2362`, `:2679` (durée du jeton d'accès, inchangée).
- Le développeur greppe la valeur (`/usr/bin/grep -nE '\b15 min(utes)?\b' _bmad-output/planning-artifacts/{prd,epics}.md
  docs/kesh-specifications.txt`) et trie (fenêtre de limitation des connexions, `epics.md:617`, hors classe).

## Manuel

PDF régénéré par `make fr` dans `docs/manual/`, contrôlé aplati — `pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' ' |
sed "s/’/'/g" | sed -E 's/([[:alpha:]])- ([[:alpha:]])/\1\2/g'` — puis `grep -F` de chaque phrase neuve et de chaque phrase retirée.

**Inventaire, relevé insensible à la casse, recompté à la reprise** : `/usr/bin/grep -niE 'refresh|cookie|JWT|jeton d.accès|access
token|KESH_JWT|INACTIVITY|SameSite|session' docs/manual/fr/admin-manual.tex | wc -l` rend **82** ; `/usr/bin/grep -niE
'refresh|cookie|JWT|jeton|session|reconnect|déconnect' docs/manual/fr/user-manual.tex | wc -l` rend **16** ; s'y ajoute
`/usr/bin/grep -niE 'inactivit' docs/manual/fr/*.tex` (trois lignes : `user-manual.tex:95`, `admin-manual.tex:779`, et
`admin-manual.tex:949`, hors classe — l'inactivité d'une connexion à la base, #863). Le classement de la fiche précédente (17 au tableau,
65 hors classe ; 3 et 13), **gardé**, est rappelé : hors classe côté administration — le secret JWT (`:243, :245, :252, :253, :577, :579,
:599, :732, :1744, :1749, :1752, :2414, :2766, :2770, :2772, :2783, :2789, :2987, :3691`), `KESH_COOKIE_SECURE` et HTTP-seul (`:784,
:1391, :1393, :1401, :1402, :1403, :1404, :1408, :1413, :1418, :1421, :1427, :1432, :1750, :1753, :2763, :2819`), le multi-tenant
(`:112, :1500`), la sauvegarde (`:1662`), SSH/root (`:2027, :2130, :2164`), MCP (`:1444, :2260, :3128, :3210, :3333, :3421`), sans
rapport (`:1655, :1665, :1988, :1991, :2050, :2091, :2793, :2925, :3003, :3040, :3269, :3341, :3357, :3618`), titres (`:772, :2990,
:2992`) ; côté utilisateur — `:79, :114` (récupération), `:122` (cookies), `:313, :326, :3504, :3901` (clés API, écrans à venir), `:1689,
:1733, :2084` (« une autre session »), `:3414, :3486` (sauvegarde, `company_id`), `:3843` (glossaire du journal d'audit). **Seules les
prescriptions changent.**

| Site | Ce qu'il dit | Prescrit (reprise) |
|---|---|---|
| `user-manual.tex:95` | « La session reste active 15 minutes, prolongée … Après 15 minutes sans activité … `KESH_REFRESH_INACTIVITY_MINUTES` » | La session se ferme après une **durée d'inactivité réglée par l'administrateur** (60 minutes par défaut ; Administration → Paramètres, section Session). **Toute activité dans Kesh la remet à zéro** : frappe au clavier, mouvement, clic ou molette de la souris, défilement, toucher, retour sur l'onglet de Kesh ; une activité dans un onglet vaut pour tous les onglets de Kesh du même navigateur. À l'échéance, Kesh affiche la page de connexion avec « Session fermée après N minutes d'inactivité » ; **ce qui n'était pas enregistré est perdu**, dans tous les onglets. **Ne comptent pas** : la lecture d'un PDF dans l'aperçu (Kesh ne voit pas ce qui se passe dans le cadre du document ; seuls l'entrée et la sortie du pointeur, et le clic qui y entre, comptent), le travail dans un autre programme ou un autre onglet, les mises à jour que Kesh fait seul. Recharger la page ou ouvrir un onglet ne déconnecte pas tant que la session vit. Rare : deux onglets qui renouvellent leur accès à la même seconde peuvent ramener l'un d'eux à la connexion. |
| `user-manual.tex:106` | après la récupération par courriel : « toutes vos sessions ouvertes … ont été déconnectées » | « … ne peuvent plus être renouvelées : chacune prend fin à sa prochaine activité, au plus tard à l'échéance de son jeton d'accès (15 minutes par défaut) » (`routes/auth.rs:969` ; le battement d'une ligne révoquée rend 401) |
| `user-manual.tex:119` | « Tous les refresh tokens de votre session actuelle sont révoqués » | la déconnexion révoque le jeton de renouvellement **de cette session** (`routes/auth.rs:306`) ; les autres appareils restent connectés |
| `admin-manual.tex:111` | « compensée par rafraîchissement automatique côté frontend » | « renouvelé à la première requête après son échéance ; la fin de session est fixée par la durée d'inactivité réglée dans l'interface (section *Durée d'inactivité de la session*) » |
| `:777` | `KESH_JWT_EXPIRY_MINUTES` « Durée access token » | « Durée du jeton d'accès (et de son cookie) : rythme des renouvellements ; ne fixe pas la fin de session » |
| `:778` | `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` « Durée absolue refresh token » | durée de vie du **cookie** de renouvellement, reposé à chaque renouvellement ; aucune durée absolue ne borne la session d'un utilisateur actif |
| `:779` | `KESH_REFRESH_INACTIVITY_MINUTES`, défaut `15` | ligne **retirée** du tableau, remplacée sous le tableau par : « `KESH_REFRESH_INACTIVITY_MINUTES` est **ignorée** depuis la 0.17.3 (avertissement au démarrage) : la durée d'inactivité se règle dans l'interface. Retirez-la de votre `.env` et de votre `docker-compose`. Si vous aviez posé `KESH_JWT_EXPIRY_MINUTES=480` pour contourner #876, retirez-le aussi. » |
| `:1192`, `:1361` | révocation à la réinitialisation / au recovery : « déconnectées » | même précision que `user-manual.tex:106` |
| `:1446`, `:2360`, `:2506` | « au plus sa durée plus une minute » / « la durée de session plus une minute » | « au plus la durée du **jeton d'accès** (`KESH_JWT_EXPIRY_MINUTES`) plus une minute » — c'est lui qui porte la société (inchangé sur le fond) |
| `:2292`, `:2322` | « votre session est invalidée » ; « la session est déjà invalidée » | vrai après correctif ; ajouter « les sessions de tous les utilisateurs ne peuvent plus être renouvelées : chacune prend fin à sa prochaine activité, au plus tard à l'échéance de son jeton d'accès ; la vôtre, par *Se reconnecter*. La durée d'inactivité réglée n'est pas restaurée : celle de l'installation reste en vigueur. » |
| `:2995` | cookie d'accès « `HttpOnly; Secure; SameSite=Lax` » | `HttpOnly`, `Secure` sauf `KESH_COOKIE_SECURE=false`, `SameSite=Strict` (`routes/auth.rs:51-52, :61-62`) |
| `:2996` | « refresh token (durée 30 jours …) … hash SHA-256 (jamais stocké en clair) » | 30 jours : durée du cookie ; la ligne porte l'échéance d'inactivité ; jeton stocké **tel quel** (`refresh_tokens.rs:30-34`) — #890 |
| `:2997` | rotation et trace du rejeu | ajouter : la rotation n'a lieu qu'à l'échéance du jeton d'accès et **ne prolonge pas** la session ; un rejeu dans les 30 secondes d'une rotation est refusé sans révoquer les autres sessions ; l'onglet qui a perdu la course revient à la connexion |
| `:2998` | « La déconnexion révoque tous les refresh tokens de la session » | « révoque le jeton de renouvellement de la session » |
| `:3708` | Dépannage « Erreurs 401 en cascade » — « Refresh token expiré ou invalidé » | ajouter la cause ordinaire : session fermée par inactivité (la page de connexion le dit) ; un onglet en arrière-plan peut n'afficher la connexion qu'au retour |
| `:3886` | glossaire « Token longue durée » | jeton qui renouvelle le jeton d'accès ; sa ligne porte l'échéance d'inactivité de la session |
| **neuf**, après `:2990-2998` | — | sous-section **« Durée d'inactivité de la session »** : où (Administration → Paramètres, section Session, Administrateur seul) ; bornes 1 à 1440 minutes, défaut 60 ; **exacte** (« à la seconde près, sauf deux écarts » : l'activité transmise au plus 15 s après ; l'onglet en arrière-plan) ; ce qui compte et ce qui ne compte pas (renvoi au manuel utilisateur, aperçu PDF compris) ; le serveur refuse à l'échéance quel que soit le jeton d'accès ; un changement s'applique à chaque session à sa prochaine activité, une session déjà inactive garde son échéance ; tracé au journal d'audit ; non restauré par une restauration complète ; `localStorage` indisponible ⇒ activité non partagée entre onglets ; la variable ignorée |

*(Le classement ajoute une ligne au tableau — la sous-section neuve — sans changer les décomptes du relevé, qui portent sur les lignes
existantes : 82 = 17 + 65, 16 = 3 + 13.)*

## Captures

Thème clair, base de démonstration jetable (`kesh_v0172s`), dans `/home/gcorbaz/devel/kesh-gate-logs/v0172s/captures/` : (1) Paramètres,
section Session, vue Administrateur (60) ; (2) saisie hors bornes refusée ; (3) enregistrement réussi ; (4) Paramètres vu par un Comptable
(section absente) ; (5) page de connexion « Session fermée après 2 minutes d'inactivité » ; (6) journal d'audit, l'entrée du réglage. Jointes
au Dev Agent Record et aux prompts de revue de code.

## CHANGELOG

Section `## [0.17.3] — Non publié` (à créer en tête si elle n'existe pas) :

`### Ajouté`
- **La durée d'inactivité de la session se règle dans Kesh** ([#891](https://github.com/guycorbaz/kesh/issues/891)) :
  Administration → Paramètres, section Session, de 1 minute à 24 heures, **60 minutes par défaut**. Le changement est tracé au journal
  d'audit.

`### Corrigé`
- **La session ne se ferme plus pendant que vous travaillez** ([#876](https://github.com/guycorbaz/kesh/issues/876)) : elle prenait fin
  quinze minutes après la connexion, même en pleine saisie, et la saisie en cours était perdue. Elle se ferme désormais **exactement** après
  la durée d'inactivité réglée, et **toute activité dans Kesh** — clavier, souris, défilement, toucher, retour sur l'onglet — la remet à
  zéro, dans tous les onglets. À l'échéance, la page de connexion le dit ; ce qui n'était pas enregistré est perdu. La lecture d'un PDF dans
  l'aperçu, sans geste hors du document, ne compte pas comme activité.
- Recharger la page ou ouvrir un onglet ne renvoie plus à la connexion tant que la session vit.

`### Modifié`
- **`KESH_REFRESH_INACTIVITY_MINUTES` est ignorée** : un avertissement l'indique au démarrage ; retirez-la de votre `.env` et de votre
  `docker-compose`. *Si vous aviez posé `KESH_JWT_EXPIRY_MINUTES=480` pour contourner #876, retirez-le : il ne prolonge plus la session et
  allongerait inutilement la durée de validité des droits d'un utilisateur.*
- **Une restauration complète révoque les jetons de renouvellement restaurés** : aucune session, de la sauvegarde ou de l'installation, ne
  peut plus être renouvelée après l'import ; la durée d'inactivité réglée n'est pas restaurée. *(Ce n'est pas la correction d'un défaut
  publié : avec les réglages d'avant (15/15), ces jetons mouraient de fait avec le jeton d'accès.)*

En tête de section, l'avertissement de mise à jour : **une migration** (table système du réglage) — le retour à la 0.17.1 passe par la
sauvegarde complète d'avant la mise à jour (base, `.keshbackup`, dossier des documents). `refs #876`, `refs #891` aux commits ; aucun
mot-clé de fermeture. La note de mise à jour du manuel d'administration (*Passer à la 0.17.3*) relève de la préparation de la release
(orchestrateur) : elle reprend la migration, la variable ignorée et le contournement à retirer.

## Dev Notes

- **Pourquoi aucun test ne l'a vu** : les E2E durent moins de quinze minutes ; le test du renouvellement (`api-client.test.ts:205-`)
  simule un refresh qui réussit ; aucun test ne mesurait une inactivité.
- **Sécurité** : (a) le battement n'émet aucun jeton de renouvellement et ne prolonge pas `exp` — la fenêtre de rôle ou de société
  périmés reste celle du jeton d'accès ; (b) il exige le jeton d'accès **et** le cookie de renouvellement de la même session (même
  `user_id`) — rien de plus que ce qu'exige le renouvellement ; (c) `idleMs` ne peut que **raccourcir** l'échéance (borné à `[0, N]`) :
  un client ne peut pas obtenir plus qu'en battant réellement ; (d) une session reste vivante tant qu'un onglet produit des événements
  — un script injecté le pourrait aussi, mais il disposerait déjà de la session ; (e) **cas accepté** de la grâce (C-v0172s-17) : un
  attaquant qui présente le premier un jeton volé obtient un jeton neuf ; la victime qui présente ensuite le même jeton dans les 30 s
  reçoit 401 sans révocation générale ; au-delà, la détection de vol joue. (f) Les clés d'API et le serveur MCP n'ont pas de session :
  `idle_exp` absente, rien ne change pour eux.
- **Coût** : au plus un battement toutes les 15 s par navigateur actif — une lecture de réglage par clé primaire, un `UPDATE` sur
  `token` (clé unique `uq_refresh_tokens_token`, squash `:1182`), un encodage de jeton. Aucune ligne neuve (`refresh_tokens` ne grossit pas plus qu'aujourd'hui).
- **E2E** : le backend des E2E ne pose plus la variable ; le réglage vaut 60 ; les specs bougent la souris et déclenchent des battements
  (sans effet sur leurs assertions, à vérifier : Tâche 8).
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

- 2026-11-04 — **Validation P6 (Opus) : 0/0/3/13** (rapport `kesh-gate-logs/v0172s-validate-p6.md`), sixième passe ; remédiation
  **textuelle** : aucun mécanisme ajouté, deux assertions et deux mutations retirées. **Signal de non-convergence (règle de splitting,
  second critère), constaté** : sévérité P3 MEDIUM → P4 MEDIUM → P5 **HIGH** → P6 MEDIUM ; le HIGH de P5 (motif `'full_import'` refusé par
  la contrainte CHECK) est né de la révocation **ajoutée par la remédiation P4** : c'est du **recyclage**, au sens de l'amendement D5. Le
  signal est déclaré au Project Lead par l'orchestrateur. Décision déjà prise et rappelée : #876 **sortie de la 0.17.2** pour la 0.17.3, et
  conception **simplifiée** (validation P3 : fenêtre d'inactivité, chemin 401 existant ; validation P4 : retraits). Prémisses relues au
  code. **P1** (MEDIUM) appliqué — commentaire de #876 lu (« Contournement d'ici là : `KESH_JWT_EXPIRY_MINUTES=480` ») : retrait du
  contournement écrit au CHANGELOG, au manuel `:779` et dans la raison du § 2 (960 minutes) ; libellé de l'avertissement quand la variable
  est absente. **P2** (MEDIUM) appliqué par **retrait** — `auth.svelte.ts:30-37` lu (`authBroadcast` nul sans `BroadcastChannel`) : les
  assertions « `postMessage` non appelé » et M7 bis, M7 ter retirées, sans sonde neuve. **P3** (MEDIUM) appliqué — le signal ci-dessus.
  LOW : **P4** M4 sans « le cas 4 de (c) » ; **P5** `README.md:40` ajouté ; **P6** import **statique** (cycle déjà existant et bénin,
  `auth.svelte.ts:17` → `vat-rates` → `api-client.ts:15`), raison fausse retirée ; **P7** `auth.svelte.test.ts:147-163` nommé (compte d'appels
  à deux) ; **P8** N effacements écrits, sans gravité ; **P9** `hydrate()` juge le succès sur `currentUser !== null` (le chemin 401 garde
  `true` et rejoue) ; **P10** « deux requêtes quasi simultanées dans deux onglets — deux gestes, ou plusieurs onglets rouverts ensemble » ;
  **P11** `kesh-specifications.txt:44, :219, :1453` hors classe ; **P12** #890 cité (manuel `:2996`, registre C-14) ; **P13** sonde
  élargie à `crates/kesh-db/src` (réparation 5-quater comprise) ; **P14** libellé de FR13 gardé (décision du Project Lead), signalé ;
  **P15** « avec les réglages par défaut (15/15) » ; **P16** `hydrate()` cité `:209-316`, branche 401 `:261-267`. Valeurs grepées : `480`,
  `postMessage`, `M7 bis`, `M7 ter`, `cas 4 de (c)`, `importé dynamiquement`, `:229-265`, `issue demandée`, `deux gestes`. Prochaine :
  validation **P7** (Sonnet, **ciblée**, `v0172-session-validate-prompt-p7.md`, sur `git diff d556bc25 HEAD`) ; il reste P7 et P8.

- 2026-11-04 — **Reprise de la fiche (Opus, autonomie)**, sur la décision du Project Lead du même jour (commentaire de #891) : la durée
  d'inactivité se règle **dans l'interface d'administration**, s'applique **exactement** (aucun multiple) et se **remet à zéro à chaque
  activité** (souris, clavier…). Les trois prémisses de la conception validée de P3 à P6 tombent : inactivité mesurée aux requêtes,
  réglée par variable d'environnement, portée au double du jeton d'accès. La validation P7 ciblée prévue
  (`v0172-session-validate-prompt-p7.md`) **n'a pas lieu** : elle portait sur une conception remplacée ; la boucle repart à **R1**
  (complète, Sonnet, `v0172-session-validate-prompt-r1.md`).
  **Conception retenue** (C-v0172s-28 à 42) : réglage dans une table **système** `_kesh_installation_settings` (migration DDL seule, sans
  ligne initiale, défaut du code 60, bornes 1 à 1440 ; ni exportée ni restaurée — les sauvegardes 0.17.x restent importables), écran
  dans Administration → Paramètres (section Session, Administrateur seul), route `GET`/`PUT /api/v1/admin/session-settings`, audit
  `admin.session_settings_updated` ; `refresh_tokens.expires_at` devient l'échéance d'inactivité, copiée dans une revendication `idle_exp`
  du jeton d'accès que le middleware contrôle **sans tolérance** ; le renouvellement **recopie** l'échéance ; un **battement**
  `POST /api/v1/auth/activity { idleMs }` (au plus un toutes les 15 s, âge exact de la dernière activité, aucune rotation, `exp`
  inchangé) ; côté navigateur, un module d'écoute (`session-idle.svelte.ts`) partagé entre onglets par `localStorage`, une minuterie d'une
  seconde, **aucun verrou** ; fin de session à l'échéance avec « Session fermée après N minutes d'inactivité ». Précision écrite : à la
  seconde près, sauf l'activité non transmise dans les 15 s qui précèdent un arrêt du navigateur, et l'affichage différé d'un onglet en
  arrière-plan (le serveur refusant, lui, dès l'échéance). `KESH_REFRESH_INACTIVITY_MINUTES` **ignorée** avec avertissement, retirée des
  gabarits, sans amorçage (les gabarits posaient 15).
  **Gardé, et dit section par section** : la révocation à l'import complet avec `"password_change"` (§ 8, AC-5, M7 — inchangés, un volet
  ajouté : le réglage de la destination survit) ; la grâce de rotation (§ 9, AC-4, M6) ; la course résiduelle entre onglets (§ 10) ; le
  renouvellement au démarrage minimal (§ 11, AC-7, M13) ; les inventaires des manuels (82 = 17 + 65, 16 = 3 + 13, **recomptés** ce jour
  par les mêmes relevés ; seules les prescriptions changent, et une sous-section neuve s'ajoute) ; la mise à jour de FR13 et de ses copies
  (texte récrit sur le commentaire de #891) ; le tri de `docs/kesh-specifications.txt`. **Remplacé** : la règle du double (C-16, C-22), la
  borne `N − jwt` (C-21), l'option `background` (C-20 : propriété tenue par construction, le bandeau de sauvegarde n'est plus touché), la
  dérogation à six modules (C-27, remplacée par C-41 : huit modules, deux lots ordonnés, revue par lot). **Migration** : une, non
  breaking, sans bump (88 → 89) — la reprise rend la story dépendante de la sauvegarde pour le retour à la 0.17.1, ce que l'ancienne fiche
  évitait (C-v0172s-40). Valeurs grepées dans cette fiche après rédaction : `0.17.2` (ne reste que l'historique et le retour possible
  vers une 0.17.2), `KESH_REFRESH_INACTIVITY_MINUTES` (ne reste que l'ignorance et son avertissement), `double`, `background`,
  `45 et 60`, `aucune migration`. Prochaine : validation **R1** (Sonnet, complète).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
