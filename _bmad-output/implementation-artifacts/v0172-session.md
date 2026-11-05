# Story v0172-session : la 0.17.3 — la durée d'inactivité réglée dans l'interface, exacte, remise à zéro par l'activité (#876, #891)

Status: done

*Fiche **reprise** le 2026-11-04 sur une décision du Project Lead qui fait tomber les trois prémisses de la conception validée de P3 à P6
(Change Log, entrée « Reprise ») : « J'aimerais que la durée d'inactivité soit configurée dans la partie admin de l'interface web et pas
par une variable d'environnement. J'aimerais que cette durée soit exactement celle configurée dans l'interface web et non pas un multiple
de celle-ci. J'aimerais que cette durée soit remise à zéro à chaque activité : mouvement de souris, etc. » Antérieurement : « 60 minutes :
15 est trop court : je me fais régulièrement éjecter alors que je suis en train de travailler et que je consulte des documents. » Le
changement d'exigence est porté par le **CR #891** (commentaire du 2026-11-04 : FR13 réécrit). Worktree `/home/gcorbaz/devel/kesh-0172s`,
branche `story/v0172-session`, tête de reprise `c547dc41`. Clé courte : `v0172s`. Registre : `v0172-choix-autonomes.md`, section
`v0172s` ; **en vigueur : C-v0172s-14, 17, 23, 24, 26 et C-v0172s-28 à 52** (C-1 à 13 : première conception, écartée ; C-15, 16, 20,
21, 22, 27 : remplacés à la reprise par C-28 et suivants ; C-18 : retiré ; C-19 et 25 : remplacés par C-26). Issues : **#876** (bug, P2),
**#891** (CR), **#895** (CR, hors périmètre : l'avertissement avant fermeture), en `refs` seulement. Numéros de ligne de `c547dc41` (le
code applicatif est celui de `d7ec2ea2`, inchangé depuis). La boucle de validation **est repartie à R1** (complète, Sonnet, 0/0/5/12,
remédiée), puis **R2** (complète, Opus, 0/0/3/12, remédiée) et **R3** (complète, Sonnet, 0/0/2/7,
remédiée), puis **R4** (ciblée, Opus, 0/0/2/6, remédiée) ; prochaine : **R5** (ciblée, Sonnet, sur `git diff ec83dcdd HEAD`).*

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
   `backup.rs` (table système exclue, commentaire `:33-37` et test `:851`) ; `test_fixtures.rs` (commentaire des tables système exclues du
   vidage, `:365-371`) ; `test-schema/0001_schema_squash.sql` **régénéré par `scripts/regen-test-schema.sh`** (fichier généré, « NE PAS
   ÉDITER ») ; `migrations.sha384` ; `tests/migrations_upgrade_path.rs` (`assert_eq!(total, 88)` → 89 **et** `total - 54` → `total - 55`,
   P6).
2. `crates/kesh-api` — `config.rs`, `main.rs` (`:245-251`), `auth/jwt.rs`, `middleware/auth.rs`, `routes/auth.rs`, `routes/setup.rs`,
   `routes/admin.rs` (révocation à l'import), `routes/session_settings.rs` (neuf) et `routes/mod.rs`, `lib.rs`, `audit_labels.rs`,
   `README.md` ; tests, dont les **gardes** qui changent : `tests/audit_route_registry.rs` (compteurs), `tests/admin_pat_denied_e2e.rs`
   (constructeurs du bloc admin), `tests/configuration_transmise.rs` (vert **sans modification**, C-v0172s-31).
3. `crates/kesh-i18n` — `locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
4. `frontend/src/lib/shared/utils` — `api-client.ts` (`tryRefresh()`, C-26) et son test.
5. `frontend/src/lib/app/stores` — `auth.svelte.ts` (champs d'inactivité de la charge utile, `hydrate()` → `tryRefresh()`) et
   `session-idle.svelte.ts` (neuf, le suivi d'activité), et leurs tests.
6. `frontend/src/lib/features/settings` — `settings.api.ts`, `settings.types.ts`, `SessionIdleSection.svelte` (neuf) et son test.
7. `frontend/src/lib/features/setup` — `setup.api.ts` (`:46-52`, un des trois constructeurs de `LoginPayload` : il transmet les champs
   d'inactivité de la réponse de l'assistant) ; un test neuf `setup.api.test.ts` (il n'en a pas).
8. `frontend/src/routes` — `+layout.svelte` (racine : démarrage du suivi), `(app)/settings/+page.svelte` (la section),
   `login/+page.svelte` (le message de fin de session, et le constructeur de `LoginPayload` de la connexion, `:35-50`).
9. Configuration livrée et textes : `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`, `docs/migrations-idempotence-audit.md`,
   manuels FR (`.tex` et PDF), `CHANGELOG.md`, `_bmad-output/planning-artifacts/{prd,epics}.md`, `docs/kesh-specifications.txt`.

**Dérogation à la règle de splitting** (C-v0172s-41, reformulée à la validation R1) : **neuf** modules, dont huit de code. **La règle n'est
pas satisfaite, et son exception ne s'applique pas à la lettre** : l'exception vise des cycles de dépendance ou des fusions intermédiaires
impossibles à tester ; ici, chaque moitié se teste seule (tests d'intégration du serveur ; Vitest avec `apiClient` simulé). Ce qui lie les
deux moitiés est la **publication** : le serveur seul (modules 1 à 3) fermerait la session N minutes après la connexion malgré l'activité,
puisque seul le battement prolonge l'échéance, et le navigateur seul n'aurait rien à appeler. L'alternative honnête serait deux fiches
fusionnées sans release entre elles ; elle est **écartée** pour ne pas recommencer la boucle de validation sur deux fiches couplées. **Déclarée au Project Lead le
2026-11-04**, par l'orchestrateur, dans sa conversation ; **le Project Lead n'a pas demandé de découpage**. *Atténuation* : le développement se fait en **deux lots ordonnés** sur la
branche — **lot S** (serveur : modules 1 à 3, avec la configuration livrée) puis **lot N** (navigateur : modules 4 à 8, avec les manuels, le
CHANGELOG et le PRD) — et **la revue de code se tient par lot** (un prompt de revue par lot, chacun sur trois à cinq modules). *Risque
accepté* : une validation de fiche sur neuf modules ; borné par la simplicité de chaque moitié (un réglage, une échéance, un battement ; un
module d'écoute sans verrou). Une modification de `repositories/` et une migration imposent le **gate complet en cours de boucle**
(`CLAUDE.md`, exception `kesh-db`).

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
   (`:38-45`) est construit en littéral à **cinq** endroits — deux en production (`middleware/auth.rs:156`, `auth/api_key.rs:204`) et trois
   en tests (`helpers.rs:111`, dans `#[cfg(test)] mod tests`, `:105-111` ; `services_motif.rs:881`, `services_rollout.rs:442`) ; `Claims` l'est dans **38 fichiers**
   hors de `jwt.rs` (`/usr/bin/grep -rlE "\bClaims \{" --include=*.rs crates | /usr/bin/grep -v src/auth/jwt.rs | wc -l`), tous des
   tests qui forgent un jeton par `jsonwebtoken::encode` ; `jwt::encode` n'est appelé que dans `src/` — quatre sites de production
   (`routes/auth.rs:244, :434, :536`, `routes/setup.rs:280`) et les tests en module de `middleware/auth.rs` et de `jwt.rs`. Le cookie d'accès vit `jwt_expiry` (`routes/auth.rs:54`), celui de renouvellement
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
   authentifié. `localStorage` sert à purger d'anciennes clés (`auth.svelte.ts:153-157`, `:178-182`), au mode d'affichage
   (`mode.svelte.ts:21`) et à l'état des sections de la barre latérale (`(app)/+layout.svelte:212`, préfixe `kesh:sidebar:`) — aucune clé
   `kesh:session:`.
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
   qu'aucune ligne n'existe (précédent `mcp_server_settings`) — la migration est du DDL pur, P7 n'a rien à trier, et le squash de test, régénéré
   par `scripts/regen-test-schema.sh`, n'y gagne que le `CREATE TABLE`. En-tête de migration sur le modèle de `20261024000001_backup_schedule.sql` (table système, non breaking,
   retour par la sauvegarde, tracked-by-sqlx). Table **système** : ajoutée à l'exclusion de `backup.rs` (commentaire `:33-37`, liste du
   test `:851`) ; ni exportée, ni restaurée, ni vidée par la réinitialisation de démonstration ; **une restauration garde le réglage de la
   destination**, et les sauvegardes 0.17.x restent importables. Dépôt `installation_settings` : `get(pool) -> InstallationSettings {
   session_idle_minutes, version }` (défaut si absente) ; `update_session_idle_minutes(pool, expected_version, user_id, minutes)` —
   version 0 ⇒ `INSERT` (doublon ⇒ conflit), sinon `UPDATE … WHERE id = 1 AND version = ?` (0 ligne ⇒ conflit), `version + 1`, et
   l'audit **`admin.session_settings_updated`** (`before`/`after`, `{ "sessionIdleMinutes": … }`) **dans la même transaction**
   (précédent `backup_schedule::update_settings`). **Une valeur inchangée n'écrit rien** — ni ligne, ni audit, ni version — comme le
   précédent (`backup_schedule.rs:176-178`) : à version 0, `PUT 60` rend `{ 60, version 0 }` sans créer de ligne. Constantes `SESSION_IDLE_MIN = 1`, `SESSION_IDLE_MAX = 1440`,
   `SESSION_IDLE_DEFAULT = 60`, miroir de la contrainte. **Bornes** : de 1 minute (plus court rendrait la saisie d'un seul champ risquée,
   et sert la recette) à 1440 (24 heures : une journée de travail ; au-delà, la session ne se fermerait plus en pratique, ce que FR13
   exclut) — ce sont aussi les bornes de la variable d'aujourd'hui, si bien qu'aucune valeur posée hier n'en sort.
2. **L'écran et la route** (C-v0172s-30). `GET` et `PUT /api/v1/admin/session-settings` dans le bloc `admin_routes` (`lib.rs:255`,
   `require_not_pat` `:502`) : Administrateur seul, clés d'API refusées. `GET` ⇒ `{ sessionIdleMinutes, version, min: 1, max: 1440 }` ;
   `PUT { sessionIdleMinutes, version }` — corps lu par un extracteur déclaré par **`impl_validation_extractor!`**
   (`crates/kesh-api/src/extractors.rs`, C-v0172s-44), si bien que **tout** rejet du corps (clé absente, non entier comme `1.5`, JSON
   illisible) rend **400 `VALIDATION_ERROR`** et non le 422 natif d'Axum ; hors bornes ⇒ 400 aussi (rien d'écrit, aucun audit ; message
   i18n `admin-session-settings-error-out-of-domain`, repli égal à la valeur fr-CH, garde G9 de `textes_coherents.rs`), 409
   `OPTIMISTIC_LOCK_CONFLICT` sur version périmée, 200 et le nouvel état sinon. Inscrite au registre `audit_route_registry.rs` (statut
   tracé ; statut de rejeu recopié de l'entrée de `update_backup_schedule`, même patron) et au bloc `admin_routes` que lit
   `admin_pat_denied_e2e.rs` (§ *Registres et compteurs*, ci-dessous). L'écran : une section **« Session »** dans **Administration → Paramètres**
   (`/settings`), rendue pour l'Administrateur seul (`isAdmin`, `+page.svelte:38`) — un champ entier « Fermer la session après
   [ 60 ] minutes d'inactivité », bornes affichées, *Enregistrer*, message de succès, conflit de version traité comme l'e-mail de la
   société (`:61-80` : recharger et inviter à recommencer). Après enregistrement, la durée de **la session de l'Administrateur** est mise
   à jour aussitôt (`sessionIdle.setMinutes`), celle des autres sessions à leur prochaine activité (C-v0172s-38). Un Comptable ou une Consultation ne
   voit pas la section ; le `GET` leur rend 403.
3. **L'échéance, côté serveur : une seule valeur, exacte** (C-v0172s-32). `refresh_tokens.expires_at` **devient l'échéance
   d'inactivité** de la session (dernière activité + N), et le jeton d'accès en porte une copie dans une revendication **`idle_exp`**
   (secondes Unix). **Forme retenue (C-v0172s-43, validation R1)** : `Claims` **n'est pas modifiée** — les 38 fichiers de tests qui la
   construisent en littéral restent intacts. Une struct neuve `jwt::SessionClaims { #[serde(flatten)] pub claims: Claims, pub idle_exp:
   Option<i64> }` (`#[serde(default, skip_serializing_if = "Option::is_none")]` sur `idle_exp`, hors de `REQUIRED_SPEC_CLAIMS`) est ce
   que le middleware **décode** (`jwt::decode_session`, la seule fonction de décodage du chemin JWT ; `decode` devient `decode_session(..)
   .map(|s| s.claims)`, une seule implémentation) et ce que les sites de session **encodent**. Un jeton forgé par les tests (`Claims` seule)
   se décode avec `idle_exp = None`. Test unitaire de forme dans `jwt.rs` : aller-retour `SessionClaims` avec et sans `idle_exp`, et
   décodage d'un jeton `Claims` seul. **Le middleware contrôle `idle_exp` strictement**, sans la tolérance de 60 s : sur le chemin JWT,
   `if let Some(d) = s.idle_exp { if Utc::now().timestamp() >= d { 401 } }` ; `CurrentUser` reçoit `idle_exp: Option<i64>` (**cinq**
   littéraux à compléter : deux en production, trois en tests, § *Code* 3 ; `None` pour les clés d'API). **L'oubli est rendu impossible en
   production (C-v0172s-43, R1-5)** : `jwt::encode` (sans échéance) passe sous `#[cfg(test)]` — ses seuls appelants sont les tests en
   module de `jwt.rs` et de `middleware/auth.rs` ; les quatre sites de production (`routes/auth.rs:244, :434, :536`, `setup.rs:280`) et le
   battement passent par `jwt::encode_session(user_id, role, company_id, secret, lifetime, idle_exp: i64)`, dont l'échéance est
   **obligatoire** (pas d'`Option`). Sonde de contrôle au Dev Agent Record : `git grep -n "jwt::encode(" -- crates/kesh-api/src` ne rend
   que des lignes sous `#[cfg(test)]`. Écritures de l'échéance :
   - **connexion**, **assistant** (`setup.rs:288`) et **changement de mot de passe** (`auth.rs:524`) — gestes de l'utilisateur :
     `expires_at = now + N` (N lu par `installation_settings::get`), `idle_exp = expires_at` ;
   - **renouvellement** (`auth.rs:422`) : la ligne neuve **recopie** l'`expires_at` de l'ancienne — le renouvellement n'est pas une
     activité, il ne prolonge rien — et le jeton neuf porte `idle_exp` = cette même échéance ; le refus `expires_at < now` (`:399`)
     reste le contrôle ;
   - **battement d'activité** (§ 4) : `expires_at = now − âge + N`.
   Horodatages calculés par l'application (`Utc::now()`), jamais `NOW(3)` (précédent `backup_schedule.rs:16-19`). `encode_session` et
   le battement partagent une fonction `jwt::encode_session_claims(&SessionClaims, secret)` (pas de duplication).
   **Conséquence** : au-delà de l'échéance, aucune requête n'est acceptée — le jeton d'accès est refusé par `idle_exp` **quel que soit
   son `exp`**, le renouvellement par `expires_at < now`, le battement par les deux. Les jetons émis par une 0.17.x antérieure (sans
   `idle_exp`) restent acceptés jusqu'à leur `exp`, c'est-à-dire **au plus `KESH_JWT_EXPIRY_MINUTES`** (15 minutes par défaut, **8 heures**
   pour une installation qui a posé le contournement de #876) : la mise à jour ne déconnecte personne, et le CHANGELOG demande à qui a posé
   ce contournement de se reconnecter une fois après l'avoir retiré.
   *(Annoté le 2026-11-04, revue de code P1, S-A1 : faux pour les sessions du contournement. Leur ligne `refresh_tokens` porte
   l'ancienne inactivité glissante, connexion + 15 min, jamais prolongée tant que le jeton d'accès de 8 h vit ; le premier battement
   après la mise à jour trouve la ligne échue (`set_idle_deadline` : `expires_at > ?`) et rend 401, et le navigateur ferme la session.
   « Ne déconnecte personne » ne vaut que sans ce contournement ; avec lui, il faut se reconnecter une fois. Le code ne change pas
   — ranimer une ligne échue serait pire ; le manuel et le CHANGELOG le disent.)*
4. **Le battement d'activité** (C-v0172s-33). `POST /api/v1/auth/activity` (routeur protégé : jeton d'accès valide, `idle_exp` compris ;
   clés d'API refusées par `ensure_not_pat`), corps `{ "idleMs": <entier ≥ 0> }` (type `u64`, lu par un extracteur
   `impl_validation_extractor!` : `-1`, `1.5`, clé absente ⇒ **400 `VALIDATION_ERROR`**, C-v0172s-44) — **l'âge de la dernière activité**
   au moment de l'envoi, mesuré par le navigateur (§ 5) ; borné à `[0, N × 60 000]`. Sous `/api/v1/auth/`, la route reçoit le cookie de
   renouvellement, qui désigne la ligne de **cette** session. Une requête, une écriture : `refresh_tokens::set_idle_deadline(pool, token,
   user_id, deadline, now) -> bool` (`UPDATE refresh_tokens SET expires_at = ? WHERE token = ? AND user_id = ? AND revoked_at IS NULL AND
   expires_at > ?`) ; 0 ligne (cookie absent, ligne révoquée, expirée ou d'un autre utilisateur) ⇒ 401. Puis **un jeton d'accès réémis**
   avec les mêmes `sub`, `role`, `company_id` et **le même `exp`** que le jeton présenté, et le nouvel `idle_exp` — le battement ne
   prolonge pas le jeton d'accès, donc ni la fenêtre de rôle ou de société périmés (`middleware/auth.rs:56-71`), ni le rythme des
   renouvellements ; cookie d'accès d'âge `exp − now` — **si `exp − now ≤ 0`** (jeton présenté dans la tolérance de 60 s de `decode`), le
   jeton n'est **pas** réémis et aucun cookie d'accès n'est posé (un `Max-Age` nul supprimerait le cookie) : l'échéance est écrite en base,
   le prochain appel passera par le renouvellement. Réponse 200 `{ idleTimeoutMinutes, idleExpiresIn }` (secondes). **Aucune rotation,
   aucun jeton de renouvellement émis** : deux onglets qui battent ensemble ne déclenchent rien. Le battement passe par `apiClient.post` :
   sur 401 (jeton d'accès échu après une lecture de plus de 15 minutes), le chemin existant renouvelle et rejoue. Inscrit au registre
   `audit_route_registry.rs` comme route mutante **exemptée** : `("post", "auth::activity", Exempt("issue #876 — le battement d'activité
   n'écrit que l'échéance d'une session, comme le renouvellement"), SansEcritureAuJournal)`. Le motif nomme une issue, ce que
   `every_exemption_names_the_issue_that_follows_it` (`:1042`) exige (`motif.contains("issue #")`) ; `NoMatter` est exclu, la route
   écrivant `expires_at` ; le statut de rejeu est celui des quatre routes d'authentification exemptées (`:543`). `/me` et la réponse de connexion (`LoginResponse`, partagée par l'assistant, `setup.rs:307-314`)
   rendent aussi `idleTimeoutMinutes` et `idleExpiresIn` (le reste de `idle_exp`, `null` sans elle) ; la réponse du renouvellement aussi.
5. **Le navigateur : un module d'écoute, sans verrou** (C-v0172s-34, C-v0172s-35). `frontend/src/lib/app/stores/session-idle.svelte.ts`,
   démarré par le layout **racine** (`routes/+layout.svelte`) — il couvre donc `(app)` et `/onboarding`. **Clé d'abonnement**
   (C-v0172s-50, validation R3) : l'**identifiant** de l'utilisateur, non l'objet — `const sessionUserId = $derived(authState.currentUser?.userId
   ?? null)`, et un `$effect` qui ne lit que `sessionUserId` : non nul ⇒ `sessionIdle.start(…)`, dont **les arguments sont lus sous
   `untrack(…)`** (les deux champs de transit, que `login()` remplace à chaque renouvellement, ne doivent pas abonner l'effet ; `start()`
   ne doit pas non plus y lire de `$state` du module hors `untrack`) ; l'effet **rend `() => sessionIdle.stop()`** comme nettoyage, ce qui
   donne à la fois l'arrêt quand l'identifiant devient nul et l'arrêt puis le redémarrage sur un autre identifiant ; **`stop()` est
   idempotente** (la fermeture l'appelle avant `clearSession()`, puis le nettoyage l'appelle de nouveau). `authState.login()` remplace l'objet `_currentUser` à chaque renouvellement (`doRefresh`,
   `api-client.ts:149`) : un abonnement à l'objet arrêterait et redémarrerait le module toutes les 15 minutes ; à l'identifiant, non (le
   `$derived` d'une chaîne égale ne se propage pas). **Interface** (C-v0172s-50, R3-9) — N et l'échéance sont tenus par le **module**,
   non par `authState` :
   ```ts
   export const sessionIdle = {
     start(init: { idleTimeoutMinutes?: number; idleExpiresIn?: number | null }): void; // conciliation, écouteurs, minuterie
     stop(): void;                     // retire écouteurs et minuterie ; n'écrit rien
     setMinutes(n: number): void;      // N neuf (réponse d'un battement, ou enregistrement de l'écran)
     readonly minutes: number | null;  // N courant, lu par la fermeture (?minutes=)
   };
   ```
   `authState` ne porte que le **transit** des deux champs du `LoginPayload` (`idleTimeoutMinutes`, `idleExpiresIn`), que le layout passe à
   `start()` ; `SessionIdleSection` appelle `sessionIdle.setMinutes(n)` après un `PUT` réussi. Trois clés `localStorage` partagées par les onglets de l'origine :
   `kesh:session:lastActivity`, `kesh:session:reportedActivity`, `kesh:session:heartbeatAt` (millisecondes). Écouteurs passifs, en phase
   de capture, sur `window` : `keydown`, `pointerdown`, `pointermove` (**seulement si `screenX`/`screenY` diffèrent du dernier
   mouvement vu** : le navigateur émet des mouvements fictifs, à coordonnées inchangées, quand la page bouge sous un pointeur immobile — un
   bandeau mis à jour par une sonde de fond ne doit pas compter), `wheel`, `touchstart`, `focus` **dont la cible est `window` elle-même**
   (retour dans la fenêtre du navigateur ; un `.focus()` programmatique d'un élément, capturé sur `window`, a pour cible l'élément et ne
   compte pas), `dragover` et `drop` (pendant un glisser-déposer HTML5 — dépôt d'un relevé, `BankImportUpload.svelte:402-404` —, le
   navigateur émet `dragover`, non `pointermove`) ; sur `document` : `input` (capture : dictée, collage, saisie par un clavier virtuel ou une
   méthode de saisie qui n'émet pas de `keydown` utile) et `visibilitychange` vers `visible` ; `blur` de `window` quand le focus passe dans
   un `<iframe>` de la page (§ 6). **`scroll` n'est pas écouté** (C-v0172s-35, validation R2) : un contenu qui change au-dessus de la vue,
   ou qui raccourcit, émet `scroll` sans geste (ancrage de défilement, borne de `scrollTop`) — une sonde de fond le pourrait. **Ce que la
   fiche promet, et ne promet pas** (C-v0172s-49, validation R3) : le défilement **à la molette, au clavier ou au toucher** compte (par
   `wheel`, `keydown`, `touchstart`) ; le **glisser de la barre de défilement** ne compte **qu'à son début** (le `pointerdown` sur la
   barre, s'il parvient à la page ; les navigateurs ne remettent pas à la page les mouvements d'un glisser de barre, ce que la fiche
   n'établit pas autrement) — **écart déclaré**, sans portée pratique (un glisser dure quelques secondes) ; l'élan d'un défilement
   tactile, après le `touchstart`, n'est pas vu non plus. FR13, le manuel et le CHANGELOG disent donc « molette, clavier, souris,
   toucher », non « défilement ». *Écartée* : écouter `scroll` dans la seconde qui suit un geste — un mécanisme de plus pour le seul
   glisser de barre. Chaque événement : **d'abord** le contrôle de l'échéance (une activité au-delà de l'échéance ne ranime pas la session : elle la
   ferme), puis `lastActivity = now`, écrite au plus une fois par seconde. Une **minuterie d'une seconde** par onglet : (a) `deadline =
   max(lastActivity) + N × 60 000` ; `now ≥ deadline` ⇒ fin de session (§ 7) ; (b) sinon, si `lastActivity > reportedActivity` et `now −
   heartbeatAt ≥ 15 000`, l'onglet écrit `heartbeatAt = now` et `reportedActivity = lastActivity`, puis envoie le battement avec `idleMs =
   now − lastActivity` ; sur réponse, le module prend N du serveur (`sessionIdle.setMinutes`) ; sur échec (réseau, 5xx), `reportedActivity`
   reprend sa valeur précédente, `heartbeatAt` **garde** la sienne, si bien que l'envoi se retente **au pas de 15 s suivant** (non à la
   seconde suivante) ; sur 401 définitif, le chemin 401 a déjà effacé et redirigé. **Au démarrage** (connexion ou rechargement), le module
   concilie avec le serveur : si `idleExpiresIn` est un nombre, `serverLast = min(now, now + idleExpiresIn × 1000 − N × 60 000)` — **borné
   à maintenant** : une conciliation n'écrit **jamais** une activité future (avec un N appris plus petit que celui qui a calculé l'échéance
   du serveur, le calcul brut tomberait dans le futur et se partagerait à tous les onglets) —, `lastActivity = min(now, max(stockée,
   serverLast))`, et **`reportedActivity = min(now, max(reportedActivity stockée, serverLast))`** — concilié avec ce qui a **réellement**
   été rapporté au serveur (`serverLast`, que dit `idleExpiresIn`), jamais avec l'activité stockée : une activité notée dans les 15 s qui
   précèdent un rechargement ou l'ouverture d'un onglet reste « à rapporter », et le premier tic du module l'envoie (C-v0172s-51, R3-1) ; à la connexion `idleExpiresIn = N × 60`, donc `serverLast = now`.
   **Toute écriture** de `lastActivity` est de même bornée à `now`. **`heartbeatAt` absente ou non finie** ⇒ le battement est permis (elle vaut −∞ dans `now − heartbeatAt ≥ 15 000`, R5-3). **`lastActivity` absente ou non finie au tic** (clé effacée par un autre onglet ou par l'utilisateur) ⇒ le module reprend la dernière valeur finie qu'il a lue ou écrite (gardée en mémoire), ou `now` s'il n'en a aucune, et la **réécrit** dans la clé avant tout calcul : aucun calcul d'échéance ne se fait sur une valeur non finie (R5-4, étape 0 du développement). **Lecture des clés** (C-v0172s-52, validation R4) : une clé dont la
   valeur lue n'est pas un nombre fini (`Number.isFinite` faux : clé absente, `NaN`, texte) vaut **absente**, et **aucune écriture** dans
   une clé partagée ne se fait sans `Number.isFinite` vrai — un `NaN` écrit une fois rendrait `lastActivity > reportedActivity` toujours
   faux, dans tous les onglets, et la session se fermerait au serveur en pleine activité. Si `idleExpiresIn` est `null` (jeton émis avant
   la mise à jour, sans `idle_exp`) ou absent, `lastActivity` garde sa valeur stockée (bornée à `now`) et, à défaut, vaut `now` ;
   **`reportedActivity` garde sa valeur stockée si elle est finie ; sinon elle vaut 0 si `lastActivity` vient d'une valeur stockée** (cette activité part alors au premier tic) **et `lastActivity` (= `now`) s'il n'y en avait pas** (aucune activité, aucun battement : le chargement d'une page ne compte pas) — **et la valeur retenue est écrite dans la clé** (elle est finie), ce qui corrige une clé qui contenait `NaN` ou un texte (R5-1, R5-2, étape 0 du développement) —
   **aucun calcul sur `idleExpiresIn`** : en JavaScript `null × 1000` vaut 0 (l'échéance tomberait à `now`, fermeture immédiate) et `undefined × 1000` vaut `NaN`
   (jamais de fermeture). **N inconnu** (C-v0172s-47)
   (`idleTimeoutMinutes` absent, non entier ou hors `[1, 1440]` — les deux champs sont **facultatifs** dans `LoginPayload`,
   `idleTimeoutMinutes?: number; idleExpiresIn?: number | null`, pour que les **83** appels de `authState.login(` des tests restent valides,
   `git grep -o "authState.login(" -- 'frontend/src/**/*.test.ts' | wc -l`) : le module fait **un** `GET /api/v1/auth/me` et prend ses
   deux champs ; s'il échoue, il ne ferme rien de lui-même (le serveur fait foi) et retente au démarrage suivant. **Trois constructeurs**
   de `LoginPayload` transmettent les deux champs — `login/+page.svelte:35-50`, `setup.api.ts:46-52`, `doRefresh`/`tryRefresh`
   (`api-client.ts:132-157`, via `/me`) — et **`hydrate()`**, qui n'en construit pas mais affecte l'état directement
   (`auth.svelte.ts:254-259`), les lit aussi de `/me`. Le chargement d'une page ne compte pas comme activité. *Ce qui n'y est pas* : aucun `navigator.locks`, aucun
   `BroadcastChannel` neuf, aucun onglet maître, aucun module « armé », aucune suspension. Deux onglets qui envoient le même battement à
   la même seconde ne font qu'écrire deux fois la même échéance, à quelques millisecondes près. Si `localStorage` est indisponible
   (navigateur qui le refuse), le module garde ses trois valeurs en mémoire et **continue d'envoyer le battement** de l'activité de son
   onglet, mais **ne ferme rien de lui-même** (C-v0172s-45) : sans activité partagée, un onglet laissé sans geste fermerait — et, depuis la
   validation R1, **déconnecterait au serveur** (§ 7) — la session d'un autre onglet qui travaille. La fin de session vient alors du
   serveur : la première requête après l'échéance rend 401 et mène à la connexion (message de session expirée). **Écart déclaré**
   (validation R2, R2-15) : sans requête, l'écran reste affiché au-delà de l'échéance, jusqu'à la prochaine action — le serveur, lui, a
   refusé. Une fermeture locale sans déconnexion a été écartée pour rester simple (elle ramènerait à la connexion l'onglet inactif d'une
   session encore vivante). Déclaré au manuel d'administration.
6. **L'aperçu d'un document** (C-v0172s-36). Ce qui est observable, établi au code et à la spécification des événements : les
   événements de la visionneuse PDF d'un `<iframe>` (défilement, clic, clavier dans le PDF) **ne parviennent pas** à la page de Kesh (le
   document du cadre n'est pas celui de la page, et la visionneuse du navigateur n'expose pas les siens) ; la page voit le pointeur
   **entrer dans le cadre et en sortir** (`pointermove` jusqu'au bord, puis au retour), et le **clic qui donne le focus au cadre** (`blur`
   de `window` avec `document.activeElement` = l'`<iframe>`), qui compte. Une image (`<img>`) est dans la page : tout y compte. **Donc** :
   lire un PDF dans l'aperçu, sans geste hors du cadre, plus longtemps que N, ferme la session (avec le défaut de 60 minutes, une lecture
   continue d'une heure sans toucher la page). **Décision du Project Lead (2026-11-04, « 1, et éventuellement 3 »)** : la 0.17.3 garde ce
   comportement (C-v0172s-46) ; le manuel utilisateur l'écrit **en clair, avec le geste à faire** — pendant une longue lecture dans
   l'aperçu, sortir le pointeur du document, ou cliquer à côté, au moins une fois par durée réglée ; l'**avertissement avant fermeture avec
   « Rester connecté »** est **hors périmètre**, tracé par le **CR #895**, cité au manuel. *Écarté* : compter comme activité tout le temps
   où le focus reste dans le cadre — un poste quitté avec le PDF au premier plan ne se fermerait jamais, ce que FR13 interdit.
7. **La fin de session, et sa précision** (C-v0172s-37, C-v0172s-45). À l'échéance, chaque onglet (même instant, puisque `lastActivity`
   est partagée) arrête le module, **déconnecte la session au serveur et attend la réponse** — `await fetch('/api/v1/auth/logout', {
   method: 'POST', credentials: 'include', headers: { 'Content-Type': 'application/json' }, body: '{}', signal })`, le `signal` d'un
   `AbortController` qui abandonne au bout de **2 secondes** ; la route est publique (`lib.rs:1323`), lit le cookie de renouvellement,
   révoque la ligne (motif `logout`, `routes/auth.rs:306`), expire les deux cookies (`build_expired_auth_cookies`) et est idempotente
   (plusieurs onglets l'appellent ensemble sans dommage) —, puis, réponse reçue ou délai échu, appelle `authState.clearSession({ broadcast:
   false })` et `window.location.replace('/login?reason=idle&minutes=<N>')`. **Le marqueur est l'adresse elle-même** : `hydrate()`, sur
   `/login` avec `reason=idle`, ne fait **ni `/me` ni `tryRefresh()`** et laisse l'état nul (C-v0172s-48) — si bien qu'un rechargement de
   cette page ne rétablit rien, même si la déconnexion n'était pas arrivée ; `authState.logout()` n'est pas employée (elle diffuse
   `auth-change`). **Ce que la fermeture garantit, et ce qu'elle ne garantit pas** : sur la page de fermeture, aucune session n'est rétablie ;
   la session est **révoquée au serveur** dès que la déconnexion a abouti, ce qui est le cas ordinaire (2 secondes laissées) ; si elle n'a
   pas abouti (réseau coupé, serveur arrêté), la ligne n'est pas révoquée, et **naviguer ensuite vers une autre page** de Kesh peut retrouver
   la session tant que l'échéance du serveur n'est pas passée — ce qui n'arrive que si cette échéance est **plus tardive** que celle du
   navigateur (écart (c) ci-dessous) ; **écart déclaré**. Le cas d'école de l'écart (c) n'est **pas** celui d'un battement (après un
   battement, l'onglet, la ligne et le jeton portent la même échéance, calculée avec la même valeur) : c'est **la session de l'Administrateur
   qui enregistre un abaissement** (60 → 5 ; son module d'écoute prend 5 aussitôt par `sessionIdle.setMinutes`, § 2, alors que sa ligne garde l'échéance calculée avec 60
   jusqu'au battement suivant, au plus 15 s plus tard — le clic sur *Enregistrer* étant une activité) ou un onglet qui apprend la nouvelle
   valeur par `/me` au démarrage (conciliation bornée à maintenant, § 5). La page de connexion affiche
   **« Session fermée après N minutes d'inactivité. »** (clé `login-session-idle`, variable Fluent `$minutes` ; `minutes` absent ou non
   entier ⇒ le message existant de session expirée). ⚠️ **Sur `/login`, ce message s'affiche en français quelle que soit la langue** :
   les messages i18n ne se chargent qu'une fois authentifié (`routes/+layout.svelte:40-52`, l'endpoint `/api/v1/i18n/messages` est derrière
   `require_auth`), et le message voisin de session expirée est lui-même écrit en dur (`login/+page.svelte:92`) — limitation existante,
   déclarée ; la clé est tout de même portée par les quatre locales (pour le jour où l'endpoint sera public) et passe par `i18nMsg` avec
   le repli français. **Ce qui est perdu** : tout ce qui n'était pas enregistré, dans **tous** les onglets de Kesh du navigateur (formulaire
   en cours, brouillon non enregistré, filtre non appliqué) ; aucune alerte ne précède la fermeture — l'avertissement avec « Rester
   connecté » est le **CR #895**, hors de cette story. **Précision** : le
   navigateur ferme à `dernière activité + N` **à la seconde près** (activité notée au pas d'une seconde, minuterie d'une seconde) ; le
   serveur refuse **à partir de la même échéance, à la seconde près** (`idle_exp` en secondes entières, arrondi vers le bas ; l'échéance du serveur
   vaut celle du navigateur **à une seconde plus tôt près, et au délai de transmission du battement plus tard près** — le serveur calcule
   `now − idleMs + N` à la réception), parce que le battement lui transmet l'**âge exact** de la dernière activité. Quatre écarts, et seulement quatre :
   (a) l'activité est transmise au plus **15 secondes** après qu'elle a eu lieu — si le navigateur est fermé, mis en veille ou perd le
   réseau dans ces 15 secondes, le serveur ne l'a pas reçue et ferme jusqu'à 15 secondes plus tôt (l'utilisateur, alors, n'est plus là) ;
   un battement qui croise un renouvellement de la même session peut de même être perdu (la ligne neuve recopie l'échéance d'avant), écart
   borné aux mêmes 15 secondes et rattrapé au battement suivant ; (b) **un onglet en arrière-plan** : le navigateur ralentit ses minuteries
   (jusqu'à une fois par minute), si bien que l'écran de connexion peut n'apparaître qu'au retour sur l'onglet — **le serveur, lui, a
   refusé dès l'échéance**, et le retour sur l'onglet contrôle l'échéance avant de compter comme activité ; (c) **pendant la transition
   qui suit un changement du réglage** (C-v0172s-38) : chaque session applique la nouvelle valeur à sa prochaine activité ; entre-temps, une
   session déjà inactive garde l'échéance calculée avec l'ancienne — **abaissé**, un onglet qui a appris la nouvelle valeur ferme plus tôt
   que le serveur, et définitivement dès que la déconnexion aboutit (ci-dessus) ; **relevé**, le serveur ferme à l'ancienne échéance avant
   le navigateur, et la connexion s'affiche à la prochaine action avec le message de session expirée au lieu du message d'inactivité. Deux
   onglets qui n'ont pas appris la même valeur peuvent fermer à des instants différents pendant cette transition, qui dure au plus
   l'ancienne durée ; (d) **sous la seconde** (validation R2, R2-10, écrits plutôt que fermés) : un battement **rejoué** après un 401 (jeton
   d'accès échu) renvoie l'`idleMs` mesuré au premier envoi, et le serveur place l'échéance plus tard de la durée du renouvellement
   (d'ordinaire sous la seconde) ; une activité dans la **dernière seconde** avant l'échéance du serveur peut arriver après elle (pas de la
   minuterie et aller réseau) — le battement rend 401, le renouvellement est refusé, et l'utilisateur, pourtant actif à cette seconde, voit
   le message de session expirée. Aucun double, aucun arrondi à la durée du jeton d'accès : `KESH_JWT_EXPIRY_MINUTES` ne règle plus que le
   rythme des renouvellements.
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
   **session** renverrait à la connexion. *Nouveau* : sur `/login?reason=idle`, `hydrate()` ne tente **ni `/me` ni `tryRefresh()`** (§ 7,
   C-v0172s-48) ; ailleurs, après l'échéance de la session, `hydrate()` échoue (le renouvellement est refusé — ou la ligne a été révoquée par
   la déconnexion du § 7) et le layout mène à `/login` **sans** le message d'inactivité (le code du refus est le même pour tous les refus, et doit le rester,
   `all_refresh_error_codes_are_identical`) ; déclaré.
12. **`KESH_REFRESH_INACTIVITY_MINUTES` n'est plus lue du tout** (C-v0172s-31, **récrit à la validation R2**, R2-1). Aucune lecture,
   **aucun avertissement** : la forme la plus simple face à la garde `crates/kesh-api/tests/configuration_transmise.rs`. Cette garde
   **calcule** l'ensemble des variables lues depuis les appels `env_nonempty("…")` du code de production (`fn lues()`, `:265`) et exige :
   (T) que chaque variable lue soit transmise par les deux compose (sauf `EXCEPTIONS`, `:107-111`), et que toute clé transmise soit lue ;
   (E) que chaque variable lue ait sa ligne d'affectation dans `.env.example`, et que chaque affectation nomme une variable connue ;
   (F) que tout jeton `KESH_…` de `.env.example`, des deux compose, des quatre catalogues i18n, **des manuels français** et des littéraux de
   chaîne du code de production nomme une variable lue ou de l'hôte (`controle_fantomes`, `:1040` ; corpus `corpus_texte`, `:1062-1084`).
   Garder une lecture pour avertir aurait exigé une quatrième sorte d'exception, contrôlée dans les deux sens ; ne plus lire n'exige rien
   **à condition que le nom disparaisse de tout ce corpus** : retirée de `config.rs` (`:735-755`), de `.env.example` (`:159-160`), de
   `docker-compose.yml:85` et `docker-compose.prod.yml:94`, de `admin-manual.tex:779` et de `user-manual.tex:95`, et d'aucun texte neuf de
   ces fichiers — le manuel et le commentaire de `.env.example` parlent de « l'ancienne variable de durée d'inactivité », **sans la
   nommer** ; seuls le `CHANGELOG.md`, `crates/kesh-api/README.md` et `_bmad-output/` (hors corpus) la nomment. **La garde reste
   inchangée** et doit passer verte telle quelle ; ses cas exercés : (L) aucune lecture neuve hors des emplacements autorisés, (T) ni
   lecture ni transmission de la variable, (E) ni lecture ni affectation, (F) aucun jeton fantôme. `config.rs:1700`
   (`env::remove_var("KESH_REFRESH_INACTIVITY_MINUTES")`, nettoyage des tests de configuration) est retiré avec la lecture. **Conséquence
   déclarée** : une installation qui pose encore la variable (ancien `docker-compose`, `.env` recopié) démarre sans rien en dire et prend
   la valeur de l'écran (60 tant que rien n'est enregistré) ; seuls le CHANGELOG et la note de mise à jour le signalent. **Pas d'amorçage
   du réglage depuis la variable** : `.env.example:160` et les deux `docker-compose` posaient 15 ; amorcer donnerait 15 à presque toutes les
   installations, là où le Project Lead a fixé 60. Le champ `Config::refresh_inactivity` (`:207`, `:361`, `:510`, `:1186`, `:1635`) et le
   contrôle `main.rs:245-251` sont **retirés** ; les trois tests `config_refresh_inactivity_*` (`:2689-2721`) sont **retirés** (raison au
   Change Log). **Le paramètre de `from_fields_for_test` (`:450`) est gardé et ignoré** (renommé
   `_refresh_inactivity_ignored`, doc « ignoré depuis la 0.17.3 ; la durée vit en base ») et son assertion `:474-476` retirée : le retirer
   toucherait 78 fichiers de tests par édition manuelle (consigne 14), pour aucun comportement ; **issue demandée** pour le retirer en un
   lot mécanique. `KESH_JWT_EXPIRY_MINUTES` reste une variable
   (C-v0172s-42) ; **le contournement `KESH_JWT_EXPIRY_MINUTES=480` publié sur #876 est à retirer** : il ne prolonge plus la session
   (l'échéance d'inactivité s'applique quel que soit le jeton d'accès) et porterait à 8 heures la fenêtre d'un rôle ou d'une société
   périmés.
13. **Remplacé** (C-v0172s-28) : la règle du double (C-16, C-22), la borne « entre `N − jwt` et `N` après la dernière requête » (C-21),
   l'option `background` des lectures de fond (C-20 : sa propriété — **les sondes de fond ne comptent pas** — est désormais tenue par la
   conception : seul le battement prolonge l'échéance, aucune requête ne passe par le module d'écoute, et les événements qu'une mise à jour
   de l'affichage peut produire sans geste sont écartés — `pointermove` à coordonnées inchangées, `focus` d'un élément, `scroll` non
   écouté, § 5 ; le bandeau de sauvegarde et ses fichiers ne sont plus touchés), la dérogation à six modules (C-27). *Écartée* : la première conception (C-1 à 13).

## Acceptance Criteria

**AC-1 — Le réglage, son écran côté serveur, son audit.** `crates/kesh-db` (tests du dépôt, `#[sqlx::test(migrations = "./test-schema")]`) :
table vide ⇒ `get` rend `{ 60, version 0 }` ; `update(…, 0, user, 90)` ⇒ `{ 90, version 1 }` et **une** ligne d'audit
`admin.session_settings_updated` (`before.sessionIdleMinutes = 60`, `after = 90`) ; `update(…, 0, …)` une seconde fois ⇒ conflit ;
`update(…, 1, …)` ⇒ version 2 ; **valeur inchangée** — `update(…, 0, user, 60)` sur table vide ⇒ aucune ligne créée, aucun audit,
version 0 ; et, la ligne valant 90 en version 1, `update(…, 1, user, 90)` ⇒ aucun audit, version toujours 1 ; les contraintes refusent `id = 2`, `0` et `1441` (`INSERT`/`UPDATE` bruts,
précédent `backup_schedule.rs:350-362`). `crates/kesh-api/tests/session_settings_e2e.rs` (neuf) : `GET` Admin ⇒ 200 `{ sessionIdleMinutes:
60, version: 0, min: 1, max: 1440 }` ; Comptable et Consultation ⇒ 403 ; clé d'API ⇒ 403 ; `PUT 0`, `PUT 1441`, `PUT 1.5`, `PUT
"abc"`, corps sans `sessionIdleMinutes` ⇒ **400 `VALIDATION_ERROR`** (extracteur `impl_validation_extractor!`, jamais 422), rien d'écrit,
aucun audit ; `PUT 60` à version 0 ⇒ 200 `{ 60, version 0 }`, aucune ligne, aucun audit ; `PUT 30` ⇒ 200, audit ; version périmée ⇒ 409.
**Rouges avant correctif** (route et table absentes).

**AC-2 — La variable n'est plus lue.** `crates/kesh-api/tests/configuration_transmise.rs` **vert sans modification** (volets (L), (T),
(E), (F) : la variable n'est ni lue, ni transmise, ni affectée, ni nommée dans le corpus surveillé). Les trois tests
`config_refresh_inactivity_*` et la ligne `config.rs:1700` retirés, raison au Change Log. Sonde rendue au Dev Agent Record :
`git grep -nE 'refresh_inactivity|REFRESH_INACTIVITY' -- crates frontend scripts docs/manual .env.example docker-compose.yml
docker-compose.prod.yml` ne rend, après correctif, que le paramètre ignoré de `from_fields_for_test` (en minuscules) et
`crates/kesh-api/README.md` (hors corpus) — les quatre commentaires qui décrivent le champ disparu sont **mis à jour** (R3-6) :
`routes/auth.rs:30` (doc de `build_auth_cookies`) et `tests/auth_cookies_e2e.rs:249, :271, :274` (commentaires et message d'assertion :
« l'échéance d'inactivité de la ligne », non `refresh_inactivity`) ; le `CHANGELOG.md` la nomme. **Pas de rouge avant correctif** : c'est une garde, verte avant et après ; le retrait se prouve par
la sonde et par la garde.

**AC-3 — L'échéance exacte, côté serveur.** `crates/kesh-api/tests/session_idle_e2e.rs` (neuf), réglage posé par le dépôt :
(a) **connexion** avec N = 7 : la ligne porte `expires_at` à `now + 7 min` (± 2 s) et le jeton d'accès (décodé par le secret de test)
`idle_exp` = `expires_at` à la seconde ; (b) **middleware** : un jeton `encode_session` dont `exp` est dans 15 minutes et `idle_exp`
dans le passé d'une seconde ⇒ **401** sur `GET /api/v1/auth/me` ; `idle_exp` dans le futur ⇒ 200 ; un jeton sans `idle_exp`, forgé
comme les tests existants (littéral `Claims` et `jsonwebtoken::encode` : `jwt::encode` est sous `#[cfg(test)]`, invisible d'un test
d'intégration) ⇒ 200 (compatibilité) ; (c) **renouvellement** : ligne à `expires_at = now + 5 min` ⇒ `POST /auth/refresh` 200, la ligne neuve porte le
**même** `expires_at` (à la milliseconde) et le jeton neuf le même `idle_exp` ; (d) **battement** avec N = 7 et `idleMs = 10 000` : 200,
`expires_at` = `now − 10 s + 7 min` (± 2 s), réponse `idleTimeoutMinutes = 7`, cookie d'accès réémis dont `idle_exp` = la nouvelle
échéance ; `idleMs = 999 999 999` ⇒ borné (échéance = `now`, à ± 2 s) ; `idleMs = -1`, `idleMs = 1.5` et corps sans `idleMs` ⇒ **400
`VALIDATION_ERROR`** (extracteur, jamais 422) ; (e) le jeton présenté au battement est forgé par `encode_session` avec une durée de
**5 minutes** (distincte de `KESH_JWT_EXPIRY_MINUTES` = 15 de la configuration de test, pour que `exp ≠ now + jwt`) : le jeton réémis a
**le même `exp`** que lui ; un jeton
présenté dans la tolérance (`exp` passé de 30 s) ⇒ 200, échéance écrite en base, **aucun** `Set-Cookie` d'accès ; (f) cookie de renouvellement d'un **autre** utilisateur ⇒ 401, sa ligne inchangée ;
ligne révoquée ou expirée ⇒ 401 ; clé d'API ⇒ 403 ; (g) **au-delà de l'échéance** (ligne reculée d'une minute dans le passé, `idle_exp`
passé) : battement ⇒ 401, `POST /auth/refresh` ⇒ 401, aucune ligne prolongée ; (h) **assistant** (`POST /api/v1/setup/admin` sur base
vide, N = 7) : la ligne neuve à `now + 7 min`, le jeton d'accès porte `idle_exp` = `expires_at` à la seconde ; (i) **changement de mot de
passe** (`PUT /api/v1/auth/password`, N = 7) : de même pour la ligne et le jeton neufs ; (j) **un site, un cas** : chacun des cinq sites
d'émission (connexion, renouvellement, changement de mot de passe, assistant, battement) a son cas qui décode le jeton émis et exige
`idle_exp` présent et égal à l'échéance de la ligne. **Tests existants qui changent**, raison au Change Log : ceux
qui lisent l'`expires_at` d'une ligne neuve de renouvellement ou de connexion (`git grep -nE 'expires_at|minutes\(15\)' --
crates/kesh-api/tests/auth*.rs crates/kesh-api/tests/setup*.rs`, chaque site trié au Dev Agent Record) ; les tests de tolérance
(`middleware/auth.rs:352`, `auth_e2e.rs:776`) **restent verts sans modification** (jetons sans `idle_exp`), comme les 38 fichiers qui
forgent un `Claims` (`Claims` inchangée). **Rouges avant correctif** : (a) à (j).

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
pour `keydown`, `pointerdown`, `pointermove` (coordonnées neuves), `wheel`, `touchstart`, `visibilitychange` vers `visible`, `focus` sur
`window` (cible `window`), et `blur` de `window` avec un `<iframe>` focalisé — avancer 90 s, émettre l'événement, avancer 90 s : session
ouverte ; (2) **ce qui ne compte pas** : deux `pointermove` aux mêmes `screenX`/`screenY` — le second ; un `blur` sans `<iframe>`
focalisé ; un `scroll` sur un élément (non écouté) ; un `focus` dont la cible est un élément (`input.focus()` programmatique) ;
(3) **échéance exacte** : sans activité, à `N × 60 000 − 1 000` ms, `clearSession` non appelée ; à `N × 60 000` (+ le pas), **dans cet
ordre** : `fetch('/api/v1/auth/logout', …)` avec `method: 'POST'`, `credentials: 'include'` et un `signal`, **attendu** — `clearSession`
et `location.replace` ne sont appelés qu'après la résolution de ce `fetch` (simulé par une promesse résolue à la main) —, puis
`clearSession({ broadcast: false })` et `location.replace('/login?reason=idle&minutes=2')` ; (3 bis) `fetch` de déconnexion qui ne répond
pas : au bout de 2 000 ms le `signal` est abandonné et la redirection a lieu ; (4) **activité après
l'échéance** (minuterie arrêtée, événement à l'échéance + 500 ms) : fermeture, `lastActivity` non réécrite ; (5) **battement régulé** :
`pointermove` toutes les 100 ms pendant 60 s ⇒ au plus 5 battements ; aucune activité ⇒ aucun battement ; (5 bis) **`idleMs` exact**, sur la grille
d'une seconde de la minuterie (module démarré à t = 0 par une connexion, sans `heartbeatAt` stocké ; la conciliation écrit `lastActivity = 0`) : activité
à 1,5 s ⇒ battement au tic de 2 s, `idleMs = 500`, `heartbeatAt = 2 000` ; puis une seule autre activité à 10 s, puis rien ⇒ battement suivant au
tic de 17 s (15 s après le premier), `idleMs = 7 000` exactement ; (6) **un onglet vaut pour tous** : `localStorage.setItem('kesh:session:lastActivity', …)` par « un autre onglet »
repousse l'échéance de celui-ci ; `heartbeatAt` récent écrit par l'autre ⇒ pas de second battement dans les 15 s ; (7) réponse du
battement `idleTimeoutMinutes = 5` ⇒ l'échéance se calcule sur 5 ; (8) battement en échec réseau ⇒ pas de fermeture, nouvel envoi au pas
de 15 s suivant ; (9) **les sondes ne comptent pas** : un `apiClient.get` déclenché par minuterie ne modifie pas `lastActivity` ;
(10) `stop()` retire tous les écouteurs (un `keydown` après `stop()` n'écrit rien) et la minuterie ; (11) démarrage : `idleExpiresIn = 30`
avec N = 2 ⇒ `lastActivity = now − 90 s` (conciliation), fermeture à 30 s ; (12) démarrage avec `idleExpiresIn = null`, puis avec
`idleExpiresIn` absent : `lastActivity` = la valeur stockée, ou `now` sans valeur stockée — fermeture à `N × 60 000` après, jamais
`NaN` ; `reportedActivity` = la valeur stockée ; à défaut (clé absente, `"NaN"` ou texte), 0 si `lastActivity` était stockée, `now` sinon — **valeur écrite dans la clé** ; sans aucune valeur stockée, **aucun battement** au premier tic ; puis une
activité ⇒ un battement part au tic suivant (`heartbeatAt` absent) ; (12 bis) `lastActivity` effacée de `localStorage` en cours de session ⇒ au tic suivant, la clé est réécrite avec la dernière valeur connue du module (finie), et la fermeture a lieu à l'échéance attendue, jamais `NaN` (M30) ; aucune des trois clés ne contient jamais `NaN` (lecture de
`localStorage` après chaque tic) ; (13) `dragover`, `drop` et `input` (sur un
`<input>`, capture) comptent ; (14) **`localStorage` indisponible** (simulé : `setItem` qui lève) : battement envoyé sur activité, **aucune**
fermeture ni déconnexion à l'échéance locale ; (15) **N inconnu** (`idleTimeoutMinutes` absent) : un seul `GET /api/v1/auth/me`, ses champs
appliqués ; `/me` en échec ⇒ aucune fermeture ; (16) échec du battement : le suivant part à +15 s, non à +1 s ; (17) **jamais d'activité
future** : démarrage avec N = 1 et `idleExpiresIn = 3 000` (échéance calculée avec 60) ⇒ `lastActivity = now` (non `now + 59 min`), et
une valeur stockée dans le futur est ramenée à `now` ; (18) **activité non rapportée au démarrage**, sur la grille d'une seconde : `lastActivity`
stockée à t, `reportedActivity` stockée à t − 20 s, `heartbeatAt` absent ; démarrage d'un module à t + 1 s avec `idleExpiresIn` tel que
`serverLast = t − 20 s` ⇒ juste après le démarrage, `reportedActivity` stockée vaut t − 20 s ; un battement part au premier tic (t + 2 s),
`idleMs = 2 000` et `heartbeatAt = t + 2 000` exactement ; (19) **abonnement** (test du layout racine) : `authState.login` rappelé pendant la session avec le
même `userId` ⇒ `sessionIdle.start` **n'est pas** rappelé ; `userId` différent ⇒ `stop` puis `start` ; `clearSession` ⇒ `stop` ; un renouvellement (`login()` qui remplace les champs de transit) ne relance pas l'effet ; un second
`stop()` est sans effet. **Rouges avant correctif**
(le module n'existe pas). `setup.api.test.ts` (neuf) : `setupAdmin` transmet `idleTimeoutMinutes` et `idleExpiresIn` de la réponse à
`authState.login` ; le test de la page de connexion vérifie de même la connexion.

**AC-7 — Gardé : le renouvellement au démarrage** (ancien AC-5 bis). `auth.svelte.test.ts` : (1) `/me` 401, refresh 200, `/me` 200 ⇒
`currentUser` peuplé, **champs d'inactivité compris** ; (2) refresh 401 ⇒ `currentUser` nul, `location.replace` non appelé ; (3) un chemin
401 en vol et `hydrate()` ⇒ un seul `POST /auth/refresh` ; (4) **sur `/login?reason=idle`** (`window.location` simulé) : `hydrate()` ne
fait **aucun** `fetch` (ni `/me`, ni refresh) et laisse l'état nul ; sur `/login?reason=session_expired`, comportement ordinaire. Test
existant qui change : `auth.svelte.test.ts:147-163` (deux appels au lieu d'un). `api-client.test.ts` : les six tests du chemin 401
(`:205, :246, :270, :327, :488, :567`) restent verts. **Rouges avant correctif** : (1), (3), (4).

**AC-8 — L'écran.** `SessionIdleSection.test.ts` (neuf) : rend la valeur lue (60) et les bornes ; `0`, `1441`, `abc` ⇒ erreur de saisie,
aucun `PUT` ; `45` ⇒ `PUT { sessionIdleMinutes: 45, version }`, message de succès, `sessionIdle.setMinutes(45)` appelé ; 409 ⇒ rechargement et message de
conflit ; après l'enregistrement d'un abaissement, l'échéance de l'onglet suit la nouvelle valeur. `settings-page.test.ts` : la section
est rendue pour un Admin, **absente** pour un Comptable et une Consultation (aucun `GET` `/admin/session-settings` pour eux). **Test
existant qui change** : la fabrique `vi.mock('$lib/features/settings/settings.api', () => ({ … }))` de `settings-page.test.ts:20-26`
remplace le module entier ; elle reçoit les deux fonctions neuves (lecture et enregistrement du réglage), sans quoi les cas existants
rendus en Administrateur échouent. Captures (§ *Captures*). **Rouges avant correctif.**

**AC-9 — Le message de fin de session.** `login` (test de page) : `?reason=idle&minutes=60` ⇒ « Session fermée après 60 minutes
d'inactivité. » ; `?reason=idle&minutes=x` ⇒ le message existant de session expirée ; `?reason=session_expired` inchangé. Clé
`login-session-idle` dans les quatre locales, ownership vérifiée par `npm run lint-i18n-ownership`. **Ce test établit le texte français,
non une traduction** : sur `/login`, les messages i18n ne sont pas chargés (§ *Conception* 7). **Rouge avant correctif.**

**AC-10 — Recette sur une installation de dev** (`kesh_v0172s`, port 3174, `KESH_JWT_EXPIRY_MINUTES=1`, réglage porté à **2** minutes par
l'écran ; chronomètre, déroulé et heures au Dev Agent Record) : (a) bouger la souris sans cliquer pendant 3 min, puis enregistrer un
brouillon : réussite ; (b) ne rien faire : l'écran de connexion apparaît **à 2 min 00 s ± 2 s** de la dernière activité, avec le message ;
un `curl` rejouant le cookie d'accès capturé juste avant ⇒ 401 ; **recharger `/login`** : la page reste la page de connexion (la ligne est
révoquée, motif `logout`) ; (b bis) **le vrai cas de l'écart (c)** : l'Administrateur, dans **un seul onglet**, abaisse le réglage de 2
à 1 depuis l'écran et **ne fait plus rien** — l'onglet ferme à 1 min de son clic sur *Enregistrer* ; recharger `/login` : la page reste
la page de connexion (aucune requête `/me` ni refresh au journal du serveur) ; puis ouvrir `/` : connexion demandée (la ligne est révoquée,
motif `logout`, `SELECT revoked_reason FROM refresh_tokens ORDER BY id DESC LIMIT 1`) ; (c) deux onglets, activité dans le seul onglet B pendant 3 min : l'onglet
A reste connecté ; (d) activité continue pendant 90 s (jeton d'accès échu), puis recharger : connecté ; (e) aperçu PDF d'un document de
démonstration, défilement **dans** le cadre seulement : fermeture à 2 min (constat déclaré, C-v0172s-36) ; puis, même lecture avec une
sortie du pointeur hors du cadre toutes les 90 s : session ouverte (le geste que prescrit le manuel) ; (f) restauration complète d'un
export de la séance : plus aucune ligne active dans `refresh_tokens`, le réglage vaut toujours 2 ; (g) le journal d'audit montre
« Durée d'inactivité de la session modifiée » avec avant et après ; (h) backend démarré avec `KESH_REFRESH_INACTIVITY_MINUTES=15` :
il démarre sans rien en dire, et le réglage de l'écran s'applique.

**AC-11 — Les textes disent vrai.** § *Manuel* (PDF compris), § *Configuration livrée*, § *PRD*, § *CHANGELOG*.

## Tests et mutations

| # | Test | Attendu avant correctif |
|---|---|---|
| T1 | dépôt et route du réglage (AC-1) | rouge |
| T2 | la variable n'est plus lue : garde `configuration_transmise.rs` et sonde (AC-2) | verte (garde) |
| T3 | échéance serveur : connexion, middleware, renouvellement, battement (AC-3) | rouge |
| T4 | grâce de rotation (AC-4) | (a) rouge ; (b), (c) verts |
| T5 | restauration complète (AC-5) | rouge |
| T6 | suivi d'activité (AC-6) | rouge |
| T7 | renouvellement au démarrage (AC-7) | (1), (3), (4) rouges ; (2) vert |
| T8 | écran (AC-8) et message (AC-9) | rouges |

**Mutations nommées** — sous le verrou de gate ; après une restauration par `git checkout --`, toucher le fichier (cargo garde sinon le
binaire muté) :
- **M1** — le contrôle `idle_exp` retiré du middleware : T3 (b) rougit. **M1 bis** — contrôle avec la tolérance de 60 s : T3 (b) rougit.
- **M2** — le renouvellement prolonge (`now + N`) au lieu de recopier : T3 (c) rougit.
- **M3** — le battement ignore `idleMs` : T3 (d) rougit. **M3 bis** — sans borne haute : T3 (d) (cas 999 999 999) rougit.
- **M4** — le battement émet un jeton à `exp = now + jwt` : T3 (e) rougit (le jeton présenté dure 5 minutes, `jwt` vaut 15).
- **M5** — `set_idle_deadline` sans `user_id` : T3 (f) rougit.
- **M6** — grâce à 0 s : T4 (a) rougit ; **M6 bis** — grâce sans borne : T4 (b) rougit.
- **M7** — révocation de l'import retirée : T5 (a) rougit ; **M7 bis** — révocation sur le pool avant la transaction : T5 (b) et (a)
  rougissent.
- **M8** — le PUT sans contrôle de bornes : T1 rougit (la contrainte rend une erreur de base, non 400) ; **M8 bis** — audit retiré : T1
  rougit.
- **M9** — le module enregistre l'activité **avant** de contrôler l'échéance : T6 (4) rougit.
- **M10** — `pointermove` sans comparaison de coordonnées : T6 (2) rougit.
- **M11** — activité gardée en mémoire seulement (sans `localStorage`) : T6 (6) rougit.
- **M12** — battement sans régulation : T6 (5) rougit ; **M12 bis** — `idleMs` envoyé à 0 : T6 (5 bis) rougit (500 et 7 000 attendus).
- **M13** — `hydrate()` sans `tryRefresh()` : T7 (1) rougit ; **M13 bis** — `tryRefresh()` hors du mutex : T7 (3) rougit.
- **M14** — la section rendue sans condition de rôle : T8 (page) rougit.
- **M15** — l'assistant émet `idle_exp = i64::MAX` (au lieu de l'échéance de la ligne) : T3 (h) rougit ; **M15 bis** — de même au
  changement de mot de passe : T3 (i) rougit. *(Un site qui reviendrait à `jwt::encode` ne compile plus : `#[cfg(test)]`.)*
- **M16** — extracteur remplacé par `Json<T>` sur le `PUT` (puis sur le battement) : T1 (cas `1.5`, clé absente) et T3 (d) rougissent
  (422 au lieu de 400).
- **M17** — le battement pose le cookie même quand `exp − now ≤ 0` : T3 (e, cas de la tolérance) rougit.
- **M18** — la fermeture sans l'appel à `/auth/logout` : T6 (3) rougit.
- **M19** — repli sans `localStorage` qui ferme à l'échéance locale : T6 (14) rougit.
- **M20** — conciliation calculée sur `idleExpiresIn = null` : T6 (12) rougit.
- **M21** — `heartbeatAt` remis à sa valeur précédente sur échec : T6 (16) rougit.
- **M22** — valeur inchangée écrite (ligne et audit) : T1 (cas de la valeur inchangée) rougit.
- **M23** — la redirection n'attend pas la déconnexion (`fetch` lancé sans `await`) : T6 (3) rougit.
- **M24** — `hydrate()` ignore `reason=idle` : T7 (4) rougit.
- **M25** — conciliation non bornée à `now` : T6 (17) rougit.
- **M26** — `scroll` écouté de nouveau : T6 (2) rougit ; **M26 bis** — `focus` compté quelle que soit sa cible : T6 (2) rougit.
- **M27** — `reportedActivity` conciliée avec l'activité stockée : T6 (18) rougit.
- **M28** — le layout s'abonne à l'objet `currentUser` : T6 (19) rougit.
- **M29** — `reportedActivity` calculée avec `serverLast` indéfini quand `idleExpiresIn` est nul ou absent (`Math.max(stockée, NaN)`),
  sans garde `Number.isFinite` : T6 (12) rougit (`NaN` dans la clé, aucun battement après l'activité).
- **M30** — tic sans reprise de `lastActivity` absente (calcul direct sur la clé lue) : T6 (12 bis) rougit (échéance `NaN`, pas de fermeture).

## Tâches

**Lot S — serveur** (modules 1 à 3, configuration livrée)

1. Constater rouges T1, T2, T3, T4 (a), T5 sur `c547dc41` ; T4 (b), (c) verts.
2. `crates/kesh-db` : migration (en-tête : table système, non breaking, retour par la sauvegarde, tracked-by-sqlx, DDL seul) ; ligne de
   `docs/migrations-idempotence-audit.md` et compteurs **recomptés** (en-tête de section, ligne `Total`, partition) ; ligne de
   `migrations.sha384` ; squash `test-schema/0001_schema_squash.sql` **régénéré par `scripts/regen-test-schema.sh`**, jamais édité à la
   main (fichier marqué « GÉNÉRÉ, NE PAS ÉDITER » ; garde `test_schema_guard.rs`) ; exclusion de `backup.rs` et commentaire de
   `test_fixtures.rs:365-371` ; **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et chaque site inspecté (numéros de ligne ci-après **avant** l'insertion de la ligne de généalogie, qui décale de un ceux qui la suivent, R5-7)
   (`migrations_upgrade_path.rs:124-127`, `assert_eq!(total, 88, …)` → 89 avec sa ligne de généalogie, **et** `:217` `let
   n_before_upgrade_window = total - 54;` → `total - 55`, la frontière `total - N` restant constante, comme l'exige le commentaire
   `:138-142`, **et la prose du même fichier** (R3-7) : `:44, :46, :63, :64, :130, :138, :145, :148, :220, :289` — `54` → `55` là où il
   désigne la fenêtre, `88` → `89` dans la prose, la frontière `34` inchangée, `:148` (« porterait la frontière à 35 ») récrit pour 89 ; **ligne de généalogie** ajoutée après `:210`, au patron des
   précédentes : « Story v0172-session : 54 → 55, frontière inchangée (89 - 55 = 34). » ;
   contrôle `/usr/bin/grep -nE '\b(54|55|88|89)\b' crates/kesh-db/tests/migrations_upgrade_path.rs`, chaque occurrence restante justifiée
   au Dev Agent Record ; `audit_log_company_id_backfill.rs`, `lettering_documents_backfill.rs`, résolus par version) ; **P7** : `every_data_backfill_migration_is_triaged` vert sans entrée (DDL
   seul) ; dépôt `installation_settings` ; `refresh_tokens` (`set_idle_deadline`, `was_rotated_within`, `revoke_all_in_tx`).
3. `crates/kesh-api` : `jwt.rs` (`SessionClaims`, `decode_session`, `encode_session`, `encode_session_claims`, `encode` sous
   `#[cfg(test)]` ; `Claims` inchangée), `middleware/auth.rs` (décodage par `decode_session`, contrôle strict, `CurrentUser`),
   `routes/auth.rs` (connexion, renouvellement qui recopie, grâce, mot de passe, `/me`, battement), `routes/setup.rs`, `routes/admin.rs`
   (révocation), `routes/session_settings.rs`, deux extracteurs par `impl_validation_extractor!` (`extractors.rs`), `lib.rs`,
   `audit_labels.rs`, `config.rs` (lecture de la variable, `:1700` et trois tests retirés) et `main.rs` (C-v0172s-31) ; les **cinq**
   littéraux de `CurrentUser` (deux en production, trois en tests) ; les registres et compteurs du § *Registres et compteurs* ci-dessous ;
   `crates/kesh-api/README.md` (§ *Configuration livrée*).

   **Registres et compteurs** (validation R2, R2-2 ; chaque nombre **recompté à la source** au Dev Agent Record, non incrémenté de
   confiance — valeurs de départ relues le 2026-11-04) :
   - `tests/audit_route_registry.rs` : `LIB_ROUTES.len()` **142 → 144** (`:945-950`, message à compléter) ; `traced` **135 → 136**
     (`:959-984`, le `PUT` du réglage) ; `exempt` **4 → 5** (`:985`, message « 4 routes d'auth (#435) » à récrire : quatre de #435 et le
     battement, #876) ; `no_matter` 3 inchangé ; `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` **145 → 147** (`:991-995`) et `tout.len()`
     **145 → 147** (`:1379`) ; l'entrée du battement prescrite au § *Conception* 4 ; `every_exemption_names_the_issue_that_follows_it`
     (`:1042`) passe avec « issue #876 ».
   - `le_manuel_dit_le_compte_du_registre` (`:1004`) relit `admin-manual.tex` : la ligne `:3040` passe à « 136 des 144 routes à verbe
     mutant », « **cinq** routes exemptées --- les gestes de session, la connexion, la déconnexion, le renouvellement de session, le
     changement de son propre mot de passe (issue \#435) **et le battement d'activité (issue \#876)** --- » et « 136 + 5 + 3 = 144 »
     (§ *Manuel*, au tableau).
   - `tests/admin_pat_denied_e2e.rs` : « **50 constructeurs de méthode** » (`:46`) → **52** (le `GET` et le `PUT` du réglage, dans le bloc
     `admin_routes` entre les marqueurs `KESH-ADMIN-ROUTES-BEGIN/END`) ; la répartition `(14, 36)` (`:870-872`) → **(15, 37)** (le `GET`
     atteint la couche, le `PUT` est arrêté par le gate de portée d'une clé en lecture), avec la ligne de généalogie.
   - `tests/configuration_transmise.rs` : **inchangé**, vert (§ *Conception* 12).
4. `crates/kesh-i18n` : `admin-session-settings-error-out-of-domain`, `audit-log-action-admin-session-settings-updated`, et les clés du
   lot N, quatre locales (valeurs fr-CH = replis, G9).
5. Configuration livrée (§ *Configuration livrée*).
6. Mutations M1 à M8 bis, M15 à M17, M22 ; gate complet (repository et migration touchés) ; commit du lot ; **revue de code du lot S**.

**Lot N — navigateur** (modules 4 à 8, textes)

7. Constater rouges T6, T7 (1), (3), T8.
8. `api-client.ts` (`tryRefresh()`), `auth.svelte.ts` (`LoginPayload` : `idleTimeoutMinutes?: number`, `idleExpiresIn?: number | null`,
   **facultatifs** ; `hydrate()`) — et la lecture de `/me` dans `tryRefresh` (type en ligne de `doRefresh`, `api-client.ts:132-137`) ;
   `session-idle.svelte.ts` ; `routes/+layout.svelte` ; `settings.api.ts`, `settings.types.ts`, `SessionIdleSection.svelte`,
   `(app)/settings/+page.svelte` ; `login/+page.svelte` (message, et son constructeur de `LoginPayload`) ; `setup.api.ts` et
   `setup.api.test.ts`. **Les trois constructeurs de `LoginPayload`** (`login/+page.svelte:35-50`, `setup.api.ts:46-52`,
   `doRefresh`/`tryRefresh`) et `hydrate()` (affectation directe, `auth.svelte.ts:254-259`) transmettent les deux champs ; `hydrate()`
   reçoit aussi le court-circuit de `/login?reason=idle` (§ 7) ; sonde au Dev Agent Record : `git grep -nE "LoginPayload = \{|authState.login\(|_currentUser = \{" --
   frontend/src ':!*.test.ts'`. `settings-page.test.ts:20-26` : fabrique du simulacre complétée (AC-8). Vérifier qu'aucun test qui monte le layout racine ne laisse tourner la minuterie (`git grep -ln "routes/+layout"
   -- frontend/src`) et qu'aucune spec Playwright ne compte toutes les requêtes (`git grep -n "page.on('request'" -- frontend/tests`).
9. Manuels et PDF (§ *Manuel*), CHANGELOG, PRD et copies.
10. Mutations M9 à M14, M18 à M21, M23 à M30 ; recette AC-10 ; captures.
11. Gates (ci-dessous) ; **revue de code du lot N**.

**Gates** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh` (sous `flock
/home/gcorbaz/devel/kesh-gate-logs/.gate.lock` et `mem-guard`, base remise à zéro avant), `cd frontend && npm run check && npm run
lint-i18n-ownership && npm run test:unit && npm run build`, E2E complet au **dernier commit de code** (7 à 10 échecs attendus,
`docs/testing.md`), `make fr` dans `docs/manual/`. Ne déclarer que ce qui a tourné.

## Configuration livrée

- `.env.example:159-160` : la ligne `KESH_REFRESH_INACTIVITY_MINUTES=15` et son commentaire **retirés**, remplacés par : « # La durée
  d'inactivité de la session se règle dans Kesh (Administration → Paramètres, section Session ; 60 minutes par défaut). L'ancienne
  variable de durée d'inactivité n'est plus lue depuis la 0.17.3 : retirez-la si vous l'aviez recopiée. » — **sans nommer la
  variable** (garde (F) de `configuration_transmise.rs`, § *Conception* 12) ; `:128-130` : `KESH_JWT_EXPIRY_MINUTES` = « durée du jeton d'accès,
  donc rythme des renouvellements ; ne fixe pas la fin de session », `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` = « durée de vie du cookie de
  renouvellement, reposé à chaque renouvellement ».
- `docker-compose.yml:85`, `docker-compose.prod.yml:94` : la ligne retirée.
- `crates/kesh-api/README.md` : `:41` retirée (renvoi à l'écran), `:40` (« durée de vie du cookie de renouvellement, reposé à chaque
  renouvellement », non « lifetime absolu »), `:160` (`idle_exp`, contrôlé sans tolérance), `:161` (« échéance d'inactivité réglée en
  base, 60 min par défaut, prolongée par le battement `POST /api/v1/auth/activity` ; le renouvellement ne la prolonge pas »), `:179` (la
  durée par défaut est désormais de 60 minutes : le risque du jeton stocké en clair est plus grand — #890, C-v0172s-14), `:16`
  (`jwt.rs` : `SessionClaims` et `idle_exp`).

## PRD (CR #891)

- `_bmad-output/planning-artifacts/prd.md:391` : « FR13 : Le système renouvelle silencieusement la session et la termine après une durée
  d'inactivité **réglée par l'Administrateur dans l'interface** (60 minutes par défaut), **appliquée exactement** ; toute activité de
  l'utilisateur dans Kesh (clavier, souris — mouvement, clic, molette —, toucher, retour sur l'onglet) la remet à zéro — CR #891 ».
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
sed "s/’/'/g" | sed -E 's/([[:alpha:]])- ([[:alpha:]])/\1\2/g' | sed -E 's/© Équipe Kesh [0-9 ]+[0-9]{4}-[0-9]{2}-[0-9]{2} [^©]{0,120}Kesh
[0-9]+\.[0-9]+\.[0-9]+ //g'` (la dernière substitution retire le pied et l'en-tête de page — relevés le 2026-11-04 sous la forme « © Équipe
Kesh 7 2026-11-03 Premiers pas Kesh 0.17.1 » — qui s'insèrent au milieu d'une phrase coupée par un saut de page) — puis `grep -F` de chaque
phrase neuve et de chaque phrase retirée, **par fragments de moins d'une ligne** pour les paragraphes longs (`user-manual.tex:95` en est
un) : un fragment qui manque se relit au PDF, il ne se déclare pas absent sur la foi d'un seul `grep`.

**Inventaire, relevé insensible à la casse, recompté à la reprise** : `/usr/bin/grep -niE 'refresh|cookie|JWT|jeton d.accès|access
token|KESH_JWT|INACTIVITY|SameSite|session' docs/manual/fr/admin-manual.tex | wc -l` rend **82** (**83** à la tête de développement, après la fusion de la 0.17.2 : la ligne neuve `:2521`, au tableau) ; `/usr/bin/grep -niE
'refresh|cookie|JWT|jeton|session|reconnect|déconnect' docs/manual/fr/user-manual.tex | wc -l` rend **16** ; s'y ajoute
`/usr/bin/grep -niE 'inactivit' docs/manual/fr/*.tex` (trois lignes : `user-manual.tex:95`, `admin-manual.tex:779`, et
`admin-manual.tex:949`, hors classe — l'inactivité d'une connexion à la base, #863). Le classement de la fiche précédente (17 au tableau,
65 hors classe ; 3 et 13), **gardé à une ligne près**, est rappelé : hors classe côté administration — le secret JWT (`:243, :245, :252, :253, :577, :579,
:599, :732, :1744, :1749, :1752, :2414, :2780, :2784, :2786, :2797, :2803, :3001, :3705`), `KESH_COOKIE_SECURE` et HTTP-seul (`:784,
:1391, :1393, :1401, :1402, :1403, :1404, :1408, :1413, :1418, :1421, :1427, :1432, :1750, :1753, :2777, :2833`), le multi-tenant
(`:112, :1500`), la sauvegarde (`:1662`), SSH/root (`:2027, :2130, :2164`), MCP (`:1444, :2260, :3142, :3224, :3347, :3435`), sans
rapport (`:1655, :1665, :1988, :1991, :2050, :2091, :2807, :2939, :3017, :3283, :3355, :3371, :3632`), titres (`:772, :3004,
:3006`) — **`:3054` passe au tableau** (validation R2, R2-2 : elle dit le compte du registre des routes, que la story change ; classée
« sans rapport » à tort), d'où **82 = 18 au tableau + 64 hors classe** ; côté utilisateur — `:79, :114` (récupération), `:122` (cookies), `:313, :326, :3581, :3979` (clés API, écrans à venir), `:1697,
:1778, :2152` (« une autre session »), `:3491, :3563` (sauvegarde, `company_id`), `:3921` (glossaire du journal d'audit). **Seules les
prescriptions changent.**

| Site | Ce qu'il dit | Prescrit (reprise) |
|---|---|---|
| `user-manual.tex:95` | « La session reste active 15 minutes, prolongée … Après 15 minutes sans activité … `KESH_REFRESH_INACTIVITY_MINUTES` » | La session se ferme après une **durée d'inactivité réglée par l'administrateur** (60 minutes par défaut ; Administration → Paramètres, section Session). **Toute activité dans Kesh la remet à zéro** : frappe au clavier (y compris pour faire défiler), mouvement, clic ou molette de la souris, toucher, retour sur l'onglet de Kesh ; faire glisser la barre de défilement ne compte qu'à son début ; une activité dans un onglet vaut pour tous les onglets de Kesh du même navigateur. À l'échéance, Kesh affiche la page de connexion avec « Session fermée après N minutes d'inactivité » ; **ce qui n'était pas enregistré est perdu**, dans tous les onglets. Comptent aussi la dictée, le collage et le glisser-déposer d'un fichier. **Ne comptent pas** : la lecture d'un PDF dans l'aperçu — Kesh ne voit pas ce qui se passe dans le cadre du document ; seuls l'entrée et la sortie du pointeur, et le clic qui y entre, comptent. **Pendant une longue lecture dans l'aperçu, sortez le pointeur du document, ou cliquez à côté, au moins une fois par durée réglée** (une fois par heure avec le réglage par défaut) ; un avertissement avant la fermeture est demandé (CR #895). Ne comptent pas non plus le travail dans un autre programme ou un autre onglet, ni les mises à jour que Kesh fait seul. Le message s'affiche en français sur la page de connexion, quelle que soit la langue choisie. Recharger la page ou ouvrir un onglet ne déconnecte pas tant que la session vit. Rare : deux onglets qui renouvellent leur accès à la même seconde peuvent ramener l'un d'eux à la connexion. |
| `user-manual.tex:106` | après la récupération par courriel : « toutes vos sessions ouvertes … ont été déconnectées » | « … ne peuvent plus être renouvelées : chacune prend fin à sa prochaine activité, au plus tard à l'échéance de son jeton d'accès (15 minutes par défaut) » (`routes/auth.rs:969` ; le battement d'une ligne révoquée rend 401) |
| `user-manual.tex:119` | « Tous les refresh tokens de votre session actuelle sont révoqués » | la déconnexion révoque le jeton de renouvellement **de cette session** (`routes/auth.rs:306`) ; les autres appareils restent connectés |
| `admin-manual.tex:111` | « compensée par rafraîchissement automatique côté frontend » | « renouvelé à la première requête après son échéance ; la fin de session est fixée par la durée d'inactivité réglée dans l'interface (section *Durée d'inactivité de la session*) » |
| `:777` | `KESH_JWT_EXPIRY_MINUTES` « Durée access token » | « Durée du jeton d'accès (et de son cookie) : rythme des renouvellements ; ne fixe pas la fin de session » |
| `:778` | `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` « Durée absolue refresh token » | durée de vie du **cookie** de renouvellement, reposé à chaque renouvellement ; aucune durée absolue ne borne la session d'un utilisateur actif |
| `:779` | `KESH_REFRESH_INACTIVITY_MINUTES`, défaut `15` | ligne **retirée** du tableau, remplacée sous le tableau par : « Depuis la 0.17.3, **l'ancienne variable de durée d'inactivité n'est plus lue** (voir le CHANGELOG de la 0.17.3) : la durée se règle dans l'interface. Retirez-la de votre `.env` et de votre `docker-compose`. Si vous aviez posé `KESH_JWT_EXPIRY_MINUTES=480` pour contourner \#876, retirez-le aussi. » — **sans nommer la variable** : le manuel français est dans le corpus de la garde (F) de `configuration_transmise.rs`, où un nom de variable non lue est un fantôme (§ *Conception* 12) |
| `:3054` | « 135 des 142 routes à verbe mutant … quatre routes exemptées … (issue \#435) … 135 + 4 + 3 = 142 » | « 136 des 144 routes à verbe mutant … **cinq** routes exemptées --- les gestes de session, la connexion, la déconnexion, le renouvellement de session, le changement de son propre mot de passe (issue \#435) et le battement d'activité (issue \#876) --- … 136 + 5 + 3 = 144 » — chaque formule une fois, comme l'exige `le_manuel_dit_le_compte_du_registre` |
| `:1192`, `:1361` | révocation à la réinitialisation / au recovery : « déconnectées » | même précision que `user-manual.tex:106` |
| `:1446`, `:2360`, `:2506` | « au plus sa durée plus une minute » / « la durée de session plus une minute » | « au plus la durée du **jeton d'accès** (`KESH_JWT_EXPIRY_MINUTES`) plus une minute » — c'est lui qui porte la société (inchangé sur le fond) |
| `:2521` *(ajoutée à l'étape 0 du développement : écrite par la 0.17.2, § « Passer à la 0.17.2 »)* | « La session, non corrigée dans cette version » : nomme `KESH_JWT_EXPIRY_MINUTES=15` **et l'ancienne variable de durée d'inactivité**, annonce la correction pour la 0.17.3 et le contournement `KESH_JWT_EXPIRY_MINUTES=480` | récrite **sans nommer l'ancienne variable** (garde (F) de `configuration_transmise.rs`) : « Avec les réglages par défaut (jeton d'accès et durée d'inactivité de 15 minutes chacun) … (issue \#876). Corrigée en 0.17.3 (\S *Durée d'inactivité de la session*) : le contournement ci-dessous est à retirer en passant à la 0.17.3. » |
| `:2292`, `:2322` | « votre session est invalidée » ; « la session est déjà invalidée » | vrai après correctif ; ajouter « les sessions de tous les utilisateurs ne peuvent plus être renouvelées : chacune prend fin à sa prochaine activité, au plus tard à l'échéance de son jeton d'accès ; la vôtre, par *Se reconnecter*. La durée d'inactivité réglée n'est pas restaurée : celle de l'installation reste en vigueur. » |
| `:3009` | cookie d'accès « `HttpOnly; Secure; SameSite=Lax` » | `HttpOnly`, `Secure` sauf `KESH_COOKIE_SECURE=false`, `SameSite=Strict` (`routes/auth.rs:51-52, :61-62`) |
| `:3010` | « refresh token (durée 30 jours …) … hash SHA-256 (jamais stocké en clair) » | 30 jours : durée du cookie ; la ligne porte l'échéance d'inactivité ; jeton stocké **tel quel** (`refresh_tokens.rs:30-34`) — #890 |
| `:3011` | rotation et trace du rejeu | ajouter : la rotation n'a lieu qu'à l'échéance du jeton d'accès et **ne prolonge pas** la session ; un rejeu dans les 30 secondes d'une rotation est refusé sans révoquer les autres sessions ; l'onglet qui a perdu la course revient à la connexion |
| `:3012` | « La déconnexion révoque tous les refresh tokens de la session » | « révoque le jeton de renouvellement de la session » |
| `:3722` | Dépannage « Erreurs 401 en cascade » — « Refresh token expiré ou invalidé » | ajouter la cause ordinaire : session fermée par inactivité (la page de connexion le dit) ; un onglet en arrière-plan peut n'afficher la connexion qu'au retour |
| `:3900` | glossaire « Token longue durée » | jeton qui renouvelle le jeton d'accès ; sa ligne porte l'échéance d'inactivité de la session |
| **neuf**, après `:3004-3012` | — | sous-section **« Durée d'inactivité de la session »** : où (Administration → Paramètres, section Session, Administrateur seul) ; bornes 1 à 1440 minutes, défaut 60 ; **exacte** (« à la seconde près, sauf quatre écarts » : l'activité transmise au plus 15 s après ; l'onglet en arrière-plan ; la transition qui suit un changement du réglage ; deux écarts sous la seconde — rejeu d'un battement après un renouvellement, activité de la toute dernière seconde) ; ce qui compte et ce qui ne compte pas (renvoi au manuel utilisateur, aperçu PDF compris, avec le geste à faire et le CR \#895) ; le serveur refuse à l'échéance quel que soit le jeton d'accès ; à l'échéance, le navigateur **déconnecte** la session au serveur (il attend la réponse jusqu'à deux secondes) et la page de connexion ne la rétablit pas ; si la déconnexion n'a pas abouti (réseau coupé) et que l'échéance du serveur est plus tardive, revenir à une autre page de Kesh peut retrouver la session jusqu'à cette échéance ; un changement s'applique à chaque session à sa prochaine activité, une session déjà inactive garde son échéance — un abaissement peut donc fermer une session plus tôt que le serveur ne l'aurait fait ; tracé au journal d'audit ; non restauré par une restauration complète ; `localStorage` indisponible ⇒ activité non partagée entre onglets, c'est le serveur qui ferme (message de session expirée) et l'écran reste affiché jusqu'à la prochaine action ; l'ancienne variable de durée d'inactivité n'est plus lue (sans la nommer) ; après la mise à jour, les jetons émis avant restent valables jusqu'à `KESH_JWT_EXPIRY_MINUTES` |

*(Le tableau porte une ligne de plus que le relevé — la sous-section neuve —, qui ne change pas ses décomptes, portés par les lignes
existantes : 82 = 18 + 64 depuis la validation R2 (`:3054` reclassée), **83 = 19 + 64** depuis la fusion de la 0.17.2 (`:2521`), 16 = 3 + 13.)*

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
  la durée d'inactivité réglée, et **toute activité dans Kesh** — clavier, souris (mouvement, clic, molette), toucher, retour sur l'onglet — la remet à
  zéro, dans tous les onglets. À l'échéance, la page de connexion le dit ; ce qui n'était pas enregistré est perdu. La lecture d'un PDF dans
  l'aperçu, sans geste hors du document, ne compte pas comme activité : pendant une longue lecture, sortez le pointeur du document de temps
  en temps (un avertissement avant la fermeture est demandé : [#895](https://github.com/guycorbaz/kesh/issues/895)).
- Recharger la page ou ouvrir un onglet ne renvoie plus à la connexion tant que la session vit.

`### Modifié`
- **`KESH_REFRESH_INACTIVITY_MINUTES` n'est plus lue** : Kesh l'ignore sans rien en dire ; retirez-la de votre `.env` et de votre
  `docker-compose`. *Si vous aviez posé `KESH_JWT_EXPIRY_MINUTES=480` pour contourner #876, retirez-le, puis reconnectez-vous une fois :
  il ne prolonge plus la session et allongerait inutilement la durée de validité des droits d'un utilisateur ; les jetons émis avant la
  mise à jour restent valables jusqu'à huit heures.* *(Annoté le 2026-11-04, revue de code P1 : « huit heures » est faux, cf. S-A1
  au § 3 ; le CHANGELOG dit désormais qu'avec ce contournement il faut se reconnecter une fois, et une entrée « Un programme qui se
  connecte par mot de passe ne prolonge plus sa session » s'ajoute (S-E9).)*
- **Une restauration complète révoque les jetons de renouvellement restaurés** : aucune session, de la sauvegarde ou de l'installation, ne
  peut plus être renouvelée après l'import ; la durée d'inactivité réglée n'est pas restaurée. *(Ce n'est pas la correction d'un défaut
  publié : avec les réglages d'avant (15/15), ces jetons mouraient de fait avec le jeton d'accès.)*

En tête de section, l'avertissement de mise à jour : **une migration** (table système du réglage) — le retour à une version antérieure
(0.17.2 ou 0.17.1) passe par la sauvegarde complète d'avant la mise à jour (base, `.keshbackup`, dossier des documents). `refs #876`, `refs #891` aux commits ; aucun
mot-clé de fermeture. La note de mise à jour du manuel d'administration (*Passer à la 0.17.3*) relève de la préparation de la release
(orchestrateur) : elle reprend la migration, la variable ignorée et le contournement à retirer.

## Dev Notes

- **Pourquoi aucun test ne l'a vu** : les E2E durent moins de quinze minutes ; le test du renouvellement (`api-client.test.ts:205-`)
  simule un refresh qui réussit ; aucun test ne mesurait une inactivité.
- **Sécurité** : (a) le battement n'émet aucun jeton de renouvellement et ne prolonge pas `exp` — la fenêtre de rôle ou de société
  périmés reste celle du jeton d'accès ; (b) il exige le jeton d'accès **et** le cookie de renouvellement de la même session (même
  `user_id`) — rien de plus que ce qu'exige le renouvellement ; (c) `idleMs` ne peut que **raccourcir** l'échéance (borné à `[0, N × 60 000]` millisecondes) :
  un client ne peut pas obtenir plus qu'en battant réellement ; (d) une session reste vivante tant qu'un onglet produit des événements
  — un script injecté le pourrait aussi, mais il disposerait déjà de la session ; (e) **cas accepté** de la grâce (C-v0172s-17) : un
  attaquant qui présente le premier un jeton volé obtient un jeton neuf ; la victime qui présente ensuite le même jeton dans les 30 s
  reçoit 401 sans révocation générale ; au-delà, la détection de vol joue. (f) Les clés d'API et le serveur MCP n'ont pas de session :
  `idle_exp` absente, rien ne change pour eux.
- **Coût** : au plus un battement toutes les 15 s par navigateur actif — une lecture de réglage par clé primaire, un `UPDATE` sur
  `token` (clé unique `uq_refresh_tokens_token`, squash `:1182`), un encodage de jeton. Aucune ligne neuve (`refresh_tokens` ne grossit pas plus qu'aujourd'hui).
- **E2E** : le réglage vaut 60 (table vide) ; les specs bougent la souris et déclenchent des battements
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

- 2026-11-04 — **Validation R1 (Sonnet) : 0/0/5/12** (rapport `kesh-gate-logs/v0172s-validate-r1.md`, LOW numérotés R1-6 à R1-17),
  remédiée par l'agent de fiche (Opus). Prémisses relues au code avant application : `/usr/bin/grep -rlE "\bClaims \{"` (38 fichiers hors
  `jwt.rs`), `extractors.rs:1-55` (`impl_validation_extractor!` : 400 au lieu de 422), `auth.svelte.ts:137` et `:166-187` (`clearSession`
  ne touche pas les cookies `HttpOnly`, `logout` appelle la route), `setup.api.ts:46-52`, `jwt::encode` hors de `jwt.rs` (les seuls appels
  sont les tests en module de `middleware/auth.rs` et quatre sites de production), `CurrentUser {` (cinq littéraux),
  `scripts/regen-test-schema.sh` (« NE PAS ÉDITER »), `test_fixtures.rs:365-371`, `routes/+layout.svelte:40-52` et `login/+page.svelte:92`
  (i18n absente sur `/login`), `backup_schedule.rs:176-178`, `BankImportUpload.svelte:402-404`, et l'aplatissement réel du PDF (pied de
  page « © Équipe Kesh … Kesh 0.17.1 » au milieu des phrases). **Décision du Project Lead sur l'aperçu PDF** (« 1, et éventuellement 3 ») :
  comportement gardé, limite écrite en clair au manuel avec le geste à faire ; l'avertissement avant fermeture avec « Rester connecté » est
  hors périmètre, **CR #895**, cité à la fiche, au manuel et au CHANGELOG (C-v0172s-46).
  **MEDIUM** — **R1-1** appliqué : `Claims` inchangée ; `SessionClaims` (`#[serde(flatten)]`) décodée par le middleware et encodée par les
  sites de session ; les 38 fichiers de tests ne sont pas touchés (C-v0172s-43). **R1-2** appliqué : `impl_validation_extractor!` pour le
  `PUT` et le battement, 400 gardé, cas `1.5`, `"abc"` et clé absente ajoutés (AC-1, AC-3 d), M16 (C-v0172s-44). **R1-3** appliqué : la
  fermeture appelle `POST /auth/logout` (`keepalive`) avant `location.replace` — fin de session définitive, y compris après un abaissement
  du réglage ; troisième écart (transition après un changement) écrit au § 7 et au manuel ; le repli sans `localStorage` ne ferme plus de
  lui-même ; AC-6 (3), (14), AC-10 (b), (b bis), M18, M19 (C-v0172s-45). **R1-4** appliqué : `lib/features/setup` ajouté (module 7,
  `setup.api.ts` et un test neuf) ; les quatre constructeurs de `LoginPayload` nommés (§ 5, Tâche 8). **R1-5** appliqué : `jwt::encode` sous
  `#[cfg(test)]`, `encode_session` à échéance obligatoire, un cas par site d'émission (AC-3 h, i, j), M15, M15 bis (C-v0172s-43).
  **LOW** — **R1-6** appliqué (cinq littéraux de `CurrentUser`, trois en production, deux en tests) ; **R1-7** appliqué (`null` ⇒ aucune
  conciliation, AC-6 12, M20) ; **R1-8** appliqué (champs facultatifs ; N inconnu ⇒ un `GET /me`, AC-6 15, C-v0172s-47) ; **R1-9** appliqué
  (« jusqu'à `KESH_JWT_EXPIRY_MINUTES` », reconnexion après retrait du 480) ; **R1-10** appliqué (dérogation reformulée : l'exception ne
  s'applique pas à la lettre, deux fiches écartées, déclaration au Project Lead ; neuf modules) ; **R1-11** appliqué
  (`scripts/regen-test-schema.sh`, `test_fixtures.rs:365-371`, `migrations_upgrade_path.rs` 88 → 89) ; **R1-12** appliqué (`heartbeatAt`
  gardé, reprise à +15 s, AC-6 16, M21) ; **R1-13** appliqué (message en français sur `/login`, limitation déclarée, AC-9 ne prouve pas la
  traduction) ; **R1-14** appliqué (pas de cookie réémis si `exp − now ≤ 0`, AC-3 e, M17) ; **R1-15** appliqué (en-tête et pied de page
  retirés à l'aplatissement, fragments de moins d'une ligne) ; **R1-16** appliqué (`dragover`, `drop`, `input` ; AC-6 13) ; **R1-17**
  appliqué (valeur inchangée : rien d'écrit, AC-1, M22). Laissé : la citation `routes/auth.rs:51-52, :61-62`, relevée par le rapport sans
  être comptée, est juste à la relecture (`:51`, `:61` portent `Secure`, `:52`, `:62` `SameSite` — la ligne du tableau cite les deux). Valeurs grepées dans la fiche après remédiation : `deux écarts` (ne reste que
  l'historique), `sept littéraux`, `huit modules` (ne reste que l'entrée « Reprise »), `modules 4 à 7`, `encode_claims`, `quatre locales`
  (gardé, avec la limite écrite). Registre : C-v0172s-43 à 47 ajoutés, C-32, 34, 36, 37, 41 annotés. Prochaine : validation **R2** (Opus,
  complète, `v0172-session-validate-prompt-r2.md`).

- 2026-11-04 — **Validation R2 (Opus) : 0/0/3/12** (rapport `kesh-gate-logs/v0172s-validate-r2.md` ; six findings nés de la remédiation
  R1, neuf défauts d'origine), remédiée par l'agent de fiche (Opus). Prémisses relues au code avant application :
  `configuration_transmise.rs:1-60, :107-111, :265, :1006-1087` (volets L, T, E, F ; corpus des fantômes : `.env.example`, compose, i18n,
  manuels français, littéraux de production), `audit_route_registry.rs:543, :940-995, :1004, :1042, :1379`, `admin_pat_denied_e2e.rs:46,
  :870-872`, `admin-manual.tex:3054`, `migrations_upgrade_path.rs:120-142, :217`, `helpers.rs:105-111`, `config.rs:1700`,
  `settings-page.test.ts:20-26`, `mode.svelte.ts:21`, `(app)/+layout.svelte:212`, et le recompte des appels de `authState.login(` dans les
  tests (`git grep -o … -- 'frontend/*.test.ts' | wc -l` = 83).
  **MEDIUM** — **R2-1** appliqué, forme la plus simple : la variable **n'est plus lue du tout**, l'avertissement est retiré, son nom
  disparaît du corpus de la garde (manuel et `.env.example` disent « l'ancienne variable de durée d'inactivité ») ; `configuration_transmise.rs`
  passe **sans modification** ; AC-2 récrit en garde, T2 vert, `config.rs:1700` retiré (C-v0172s-31 récrit). **R2-2** appliqué : statut
  `Exempt("issue #876 — …")` du battement (le test exige `issue #`) ; compteurs prescrits et à recompter à la source — `LIB_ROUTES` 142 → 144,
  tracées 135 → 136, exemptées 4 → 5, 145 → 147 (`:991-995`, `:1379`), `admin_pat_denied_e2e.rs` 50 → 52 constructeurs et (14, 36) → (15, 37) ;
  `admin-manual.tex:3054` passée au tableau (82 = 18 + 64), « 136 des 144 », « cinq routes exemptées », « 136 + 5 + 3 = 144 » (§ *Tâches* 3,
  *Registres et compteurs*). **R2-3** appliqué : exemple corrigé (la session de l'Administrateur qui enregistre un abaissement, non un
  battement) ; la déconnexion est **attendue** (délai de 2 s) avant `location.replace` ; `hydrate()` ne tente ni `/me` ni `tryRefresh()`
  sur `/login?reason=idle` (C-v0172s-48) ; conciliation et écritures de `lastActivity` bornées à `now` ; ce qui reste non garanti (déconnexion
  non aboutie, échéance du serveur plus tardive, navigation vers une autre page) **écrit comme écart déclaré** ; AC-6 (3), (3 bis), (17),
  AC-7 (4), AC-10 (b bis) récrit sur la session de l'Administrateur, M23 à M25.
  **LOW** — **R2-4** appliqué (deux littéraux de `CurrentUser` en production, trois en tests, aux trois endroits) ; **R2-5** appliqué
  (`null` ⇒ 0, fermeture immédiate ; `undefined` ⇒ `NaN`) ; **R2-6** appliqué (83 appels dans les tests ; trois constructeurs et `hydrate()`
  qui affecte l'état ; sonde élargie à `_currentUser = {`) ; **R2-7** appliqué (corps de C-32 et C-41 récrits, et C-35, C-40, C-45) ;
  **R2-8** appliqué (`total - 54` → `total - 55`) ; **R2-9** appliqué (AC-6 5 bis : `idleMs = 5 000` exact ; AC-3 e : jeton présenté de
  5 minutes) ; **R2-10** appliqué par **déclaration** (écart (d), sous la seconde : rejeu après 401, activité de la dernière seconde) — le
  fermer demandait un second chemin d'envoi ; **R2-11** appliqué : `scroll` n'est plus écouté, `focus` ne compte que sur `window` (AC-6 2,
  M26, M26 bis) ; **R2-12** appliqué (`localStorage` sert aussi au mode et à la barre latérale) ; **R2-13** appliqué (« 0.17.2 ou 0.17.1 »
  au CHANGELOG et à C-40) ; **R2-14** appliqué (simulacre de `settings-page.test.ts:20-26`, AC-8 ; `config.rs:1700`, AC-2) ; **R2-15**
  appliqué par **déclaration** (sans `localStorage`, l'écran reste affiché jusqu'à la prochaine action ; écrit au § 5 et au manuel).
  Annotation : l'entrée « Validation R1 » ci-dessus dit « trois [littéraux] en production » et « quatre constructeurs » — faux à sa date,
  corrigés ici (R2-4, R2-6) ; l'entrée n'est pas réécrite. Valeurs grepées dans la fiche après remédiation : `trois en production`,
  `Trois écarts`, `sauf trois`, `89 appels`, `quatre constructeurs`, `avertissement au démarrage`, `legacy_inactivity`, `` `scroll` (capture``,
  `0.17.1 passe` — ne restent que les entrées historiques du Change Log. Registre : C-v0172s-48 ajouté ; C-31 récrit ; C-32, 35, 40, 41,
  45 corrigés. Prochaine : validation **R3** (Sonnet, complète, `v0172-session-validate-prompt-r3.md`).

- 2026-11-04 — **Validation R3 (Sonnet) : 0/0/2/7** (rapport `kesh-gate-logs/v0172s-validate-r3.md`), remédiée par l'agent de fiche
  (Opus). Prémisses relues avant application : § 5 de la fiche (conciliation), `api-client.ts:149` (`authState.login` à chaque
  renouvellement), `auth.svelte.ts` (`_currentUser` remplacé à chaque `login()`), `routes/auth.rs:30`, `auth_cookies_e2e.rs:249, :271,
  :274`, `migrations_upgrade_path.rs` (`/usr/bin/grep -nE "\b54\b|\b88\b"` : `:44, :46, :63, :64, :130, :138, :145, :148, :220, :289`),
  registre `:732`, `:890-894`.
  **MEDIUM** — **R3-1** appliqué : `reportedActivity` se concilie avec `serverLast` seulement, jamais avec l'activité stockée ; AC-6 (18),
  M27 (C-v0172s-51). **R3-2** : voie choisie, **ne plus promettre « défilement »** — FR13, manuel et CHANGELOG disent « clavier, souris
  (mouvement, clic, molette), toucher, retour sur l'onglet » ; le glisser de la barre ne compte qu'à son début, **écart déclaré** ; écouter
  `scroll` dans la seconde d'un geste est écarté, un mécanisme de plus pour ce seul cas (C-v0172s-49).
  **LOW** — **R3-3** appliqué : abonnement du layout à l'**identifiant** (`$derived(authState.currentUser?.userId ?? null)`), AC-6 (19),
  M28 (C-v0172s-50) ; **R3-4** appliqué (AC-6 5 bis sur la grille d'une seconde : 500, puis 6 000 au tic de 16 s ; M12 bis ajustée) ;
  **R3-5** appliqué (l'échéance du serveur à une seconde plus tôt près et au délai de transmission plus tard près) ; **R3-6** appliqué
  (sonde d'AC-2 juste : `routes/auth.rs:30` et `auth_cookies_e2e.rs:249, :271, :274` mis à jour ; phrase fausse des Dev Notes retirée) ;
  **R3-7** appliqué (prose de `migrations_upgrade_path.rs` aux dix lignes relevées, grep de contrôle `\b(54|55|88|89)\b`) ; **R3-8**
  appliqué (registre C-47 : 83 appels, trois constructeurs et `hydrate()` ; C-33 et Dev Notes : `[0, N × 60 000]` millisecondes) ;
  **R3-9** appliqué (interface `sessionIdle` : `start`, `stop`, `setMinutes`, `minutes` ; N tenu par le module, `authState` en transit ;
  AC-8 : `sessionIdle.setMinutes(45)` appelé).
  **Dérogation de splitting** : déclarée au Project Lead le 2026-11-04 par l'orchestrateur, dans sa conversation ; il n'a pas demandé de
  découpage (§ *Modules*, C-v0172s-41).
  Valeurs grepées dans la fiche après remédiation : `défilement` (ne reste que l'écart déclaré, l'iframe, la recette et l'ancien texte du
  manuel cité), `authState porte`, `[0, N]`, `ne pose plus la variable`, `t = 15 s`, `reportedActivity = min(now, max(stockée` ; dans le
  registre : `89 appels`, `quatre constructeurs`. Registre : C-v0172s-49 à 51 ajoutés ; C-33, 35, 41, 47 corrigés. Prochaine : validation
  **R4** (Opus, **ciblée** sur `git diff da16b94e HEAD`, `v0172-session-validate-prompt-r4.md`).

- 2026-11-04 — **Validation R4 (Opus, ciblée) : 0/0/2/6** (rapport `kesh-gate-logs/v0172s-validate-r4.md`, sur `git diff da16b94e
  ec83dcdd`), remédiée par l'agent de fiche (Opus). Prémisses relues : § 5 de la fiche (régulation, conciliation), `auth.svelte.ts:41,
  :113` (`userId` chaîne), `api-client.ts:149`, `migrations_upgrade_path.rs:192-210` (généalogie de la fenêtre), registre C-35.
  **MEDIUM** — **R4-1** appliqué : AC-6 (18) récrit sur la grille d'une seconde, sans « ≈ » : démarrage à t + 1 s, `heartbeatAt` absent,
  battement au premier tic (t + 2 s), `idleMs = 2 000`, `heartbeatAt = t + 2 000`. **R4-2** appliqué : branche « `idleExpiresIn` nul ou
  absent » de la conciliation — `reportedActivity` = valeur stockée finie, sinon 0 ; toute clé lue non finie vaut absente, aucune écriture
  sans `Number.isFinite` ; AC-6 (12) complété (deux variantes, battement après une activité, aucune clé à `NaN`) ; **M29** ajoutée
  (C-v0172s-52).
  **LOW** — **R4-3** appliqué (AC-6 5 bis : activité à 1,5 s ⇒ `idleMs = 500` au tic de 2 s ; activité à 10 s ⇒ `idleMs = 7 000` au tic
  de 17 s ; M12 bis ajustée) ; **R4-4** appliqué : la **valeur** grepée — qui tient N — dans toute la fiche (`authState`, « prennent les
  valeurs », `setMinutes`, « nouvelle valeur ») : `:162` et `:305` disent désormais `sessionIdle.setMinutes` et « son module d'écoute »,
  `:250` « le module prend N » ; restent `authState` comme transit (§ 5) et la phrase qui l'écarte (`non par authState`) ; **R4-5** appliqué
  (arguments de `start()` sous `untrack`, nettoyage `() => sessionIdle.stop()`, `stop()` idempotente ; AC-6 19 complété) ; **R4-6** appliqué
  (corps de C-35) ; **R4-7** sans objet pour la fiche : l'orchestrateur commente #891 lui-même ; **R4-8** appliqué (ligne de généalogie
  « Story v0172-session : 54 → 55, frontière inchangée (89 - 55 = 34). » après `:210`).
  Valeurs grepées après remédiation : `≈`, `0,5 s`, `6 000`, `tic de 16 s`, `son \`authState\``, `(\`authState\`)`. Registre :
  C-v0172s-52 ajouté ; C-35 corrigé. Prochaine : validation **R5** (Sonnet, **ciblée** sur `git diff ec83dcdd HEAD`,
  `v0172-session-validate-prompt-r5.md`).

- 2026-11-04 — **Validation R5 (Sonnet, ciblée) : 0/0/0/7** (rapport `kesh-gate-logs/v0172s-validate-r5.md`) : **validation terminée**
  (aucun finding au-dessus de LOW). **Étape 0 du développement** (Opus 5.5, mandat `v0173-session-dev.md`), après la fusion de `origin/main`
  (v0.17.2, `95e325e7`) dans la branche (commit `0b2e8ec9`) :
  **R5-1, R5-2** appliqués (§ 5 : `reportedActivity` à défaut vaut 0 si `lastActivity` était stockée, `now` sinon, et la valeur est écrite
  dans la clé ; AC-6 (12) aligné : sans valeur stockée, aucun battement) ; **R5-3** (`heartbeatAt` absente ⇒ battement permis) ; **R5-4**
  (`lastActivity` absente au tic ⇒ reprise de la dernière valeur finie du module, réécrite ; AC-6 (12 bis), **M30**) ; **R5-5** (« le module
  prend N du serveur ») ; **R5-6** (« puis une seule autre activité à 10 s ») ; **R5-7** (numéros de `migrations_upgrade_path.rs` avant
  l'insertion de la généalogie). **Numéros de ligne relus à la tête de fusion** (`git diff --stat c4a6f954 HEAD`, puis appariement ligne à
  ligne par `difflib` de chaque numéro cité) : `admin-manual.tex` décalé de **+14** à partir de `:2763` (ex. `:3040` → `:3054`, ligne lue :
  « La quasi-totalité des actions métier mutantes est enregistrée dans la table \texttt{audit\_log} » ; `:2995` → `:3009`, « L'access token
  (JWT court, 15 min par défaut) ») ; `user-manual.tex` : `:1689` → `:1697`, `:1733` → `:1778`, `:2084` → `:2152`, `:3414` → `:3491`,
  `:3486` → `:3563`, `:3504` → `:3581`, `:3843` → `:3921`, `:3901` → `:3979` ; `BankImportUpload.svelte:400-402` → `:402-404` ; aucun
  fichier de code serveur cité (`config.rs`, `routes/auth.rs`, `admin.rs`, `lib.rs`, `api-client.ts`, `auth.svelte.ts`, registres de
  tests) n'a changé. **Ligne neuve de l'inventaire** : `admin-manual.tex:2521` (§ *Passer à la 0.17.2*, « La session, non corrigée dans
  cette version »), qui **nomme l'ancienne variable** — elle serait un fantôme pour la garde (F) de `configuration_transmise.rs` : ajoutée au
  tableau (83 = 19 + 64). 88 migrations constatées, crates en 0.17.2 (non bumpés).

- 2026-11-04 — **Développement (Opus 5.5, autonomie)**, lots S puis N, commits `1dc94968` (lot S), `5392bd37` (lot N), `b7f02678`
  (verbe allemand, PDF). Rouges constatés avant chaque lot (`kesh-gate-logs/v0172s/rouges.log`) ; 42 mutations nommées jouées, toutes
  tuées par le test attendu (`mut/S-resume.log`, `mut/N-resume.log`) *(annoté à la revue de code P1 : **40**, 19 S + 21 N, recompté
  depuis les deux journaux)* ; gates complets, E2E complet, recette de durée et captures (Dev
  Agent Record). Choix du développement C-v0172s-53 à 58 au registre, dont deux **écarts à la fiche** : AC-2 (la garde
  `configuration_transmise.rs` change de deux nombres, C-54) et deux tests « n'ajoute aucune table » appliqués jusqu'à leur migration
  incluse (C-55) ; s'y ajoutent hors registre : `sans_ecriture` 117 → 119 (`audit_route_registry.rs`, compteur non prescrit par la
  fiche), HEAD discriminants 12 → 13 (`admin_pat_denied_e2e.rs`), G9 de `textes_coherents.rs` (7 → 8 entrées), `sitesTotal` 2813 →
  2825 (`i18n-keys.test.ts`), le verbe allemand « beenden » (garde `la_cloture_d_exercice_ne_parle_pas_comme_un_panneau`), et la note
  de la 0.17.2 du manuel d'administration récrite (`:2524`). AC-7 (3) est **vert avant correctif** (la fiche l'annonçait rouge) : sans
  renouvellement dans `hydrate()`, un seul `POST /auth/refresh` part de toute façon ; il garde le mutex, ce que M13 bis prouve.

- 2026-11-04 — **Revue de code P1 (Sonnet, trois lentilles par lot) et sa remédiation (Opus 5.5, autonomie)**. Rapports
  `kesh-gate-logs/v0172s-review-p1-{S,N}-{B,E,A}.md`. **Trend** (CRITICAL/HIGH/MEDIUM/LOW, recompté depuis les six rapports) :
  S-B 0/0/0/5, S-E 0/0/0/9, S-A 0/0/2/6 ; N-B 0/0/0/5, N-E 0/0/2/9, N-A 0/0/2/8 — **0/0/6/42** avant tri ; après tri de
  l'orchestrateur, **sept** constats au-dessus de LOW : S-B1/S-E1 **reclassé MEDIUM** (une erreur 500 au lieu d'un 409 est un
  P2), S-A1, S-A2 = N-A4 = N-E11 (décomptes), N-A1 = N-E1, N-E2, N-A2. Commits `ff20983d` (lot S), `ac666759` et `8d6561bc`
  (lot N, code), `d38fcd8b` (lot N, manuel et PDF). Chaque prémisse relue au code avant correctif :
  - **S-B1/S-E1** — lu : `installation_settings.rs` `"{SELECT_SQL} FOR UPDATE"` puis `INSERT … VALUES (1, ?, 1)`, seul
    `UniqueConstraintViolation` converti en conflit. **Rouge reproduit** avant correctif par un test de course déterministe
    (`concurrent_first_puts_yield_a_conflict_not_a_500` : `500 INTERNAL_ERROR`, `rem-p1-rouge-S1.log`) ; correctif : la route
    rejoue par `retry_on_deadlock` (C-v0172s-59), le rejeu rend 409 ; témoin du rejeu exigé.
  - **S-A1** — lu : `set_idle_deadline` `… AND expires_at > ?` ; une ligne de la 0.17.2 sous contournement porte connexion + 15 min.
    Prémisse vraie ; manuel (*Mise à jour*), CHANGELOG et fiche (§ 3, annotation datée) corrigés ; code inchangé.
  - **S-A2 / N-A4 / N-E11** — recomptés : `mut/S-resume.log` 19 `TUÉE`, `mut/N-resume.log` 21, **40** ; `rouges-S.log` : 18 rouges,
    dont `session_idle_e2e` **12** (4 + 12 + 1 + 1). Dev Agent Record (vivant) corrigé ; entrée « Développement » ci-dessus
    annotée, non réécrite ; `sprint-status.yaml` (note courante) corrigée.
  - **N-A1 / N-E1** — lu : le seul test d'arrêt ne tirait que `keydown`. Onze cas après `stop()` (table, témoin sans `stop()`
    chacun) et la minuterie comptée par `vi.getTimerCount()` (compte relatif : jsdom diffuse chaque écriture de `localStorage` par
    une minuterie à 0 ms — constaté, 5 minuterie de plus après `start()`) ; mutations `MN1` (`input`), `MN1bis` (`blur`), `MN2`
    (`clearInterval`) **tuées** par le cas attendu.
  - **N-E2** — lu : `readKey`/`writeKey`, `catch { shared = false; … }`, aucun test après la sonde. Deux cas (écriture, lecture
    refusées en cours de session) ; `MN3` (`writeKey`) tuée ; `MN4` (`readKey` qui rend « absente » sans basculer) **survivante au
    premier jeu** — le test a été renforcé (régulation du battement à 15 s gardée en repli), puis tuée ; `MN4bis` (`shared = false`
    retiré à la lettre : récursion infinie) tuée.
  - **N-A2** — AC-10 (e) **joué en navigateur** (Chromium 147, nouveau headless : le visualiseur PDF se rend, capture 07) :
    base jetable `kesh_v0172s`, réglage 2 min, fixture synthétique `scan-sans-qr.pdf` importée puis ouverte dans « Créer une
    facture » (`recette-e-r*.log`, script `scripts/recette-e-v0172s.mjs`). **Volet 1** — clic dans le document (un `blur` de la
    fenêtre, `activeElement = IFRAME`, activité comptée), puis molette **dans** le document toutes les 10 s : fermeture
    **120,67 s** (r1) et **120,55 s** (r2) après le clic, `lastActivity` jamais avancée par la molette, capture 08. **Volet 2** —
    le geste du manuel (`user-manual.tex:97`), pointeur sorti du document toutes les 90 s : session **ouverte 271,8 s** après le
    clic (réglage 120 s), chaque sortie comptée (`lastActivity` 20:30:09, 20:31:40, 20:33:10), `/me` 200 (r3). Le manuel dit
    vrai ; il n'est pas modifié sur ce point. **Constat neuf** (r2) : deux sorties au **même pixel** — la seconde n'a pas compté
    (le filtre des mouvements fictifs compare au dernier point vu) et la session s'est fermée ; un geste à la main ne repasse
    pratiquement pas par le même pixel, r3 varie le point de sortie. Tracé pour l'issue des LOW (Rec-e1). r1 a avorté au volet 2
    (second import du même fichier refusé comme doublon), corrigé dans le script.
  - **LOW corrigés** : S-A3 (doc de module du registre 144/147), S-A4 (paragraphe sous le tableau du README), S-A6 (C-v0172s-61),
    S-A7 (« ; » au lieu du second « ~: »), N-A7 (`doRefresh()`, C-58), S-B4/S-E8 (`ensure_not_pat` avant le corps, test et mutation
    `MS2` tuée), S-B5/S-E7 (`jwt::decode` sous `#[cfg(test)]` — `git grep` : aucun appelant hors du module de tests), S-E9 (manuel
    *Clients hors navigateur* et CHANGELOG ; lu : les clés d'API passent par `validate_pat`, sans `idle_exp` ni ligne
    `refresh_tokens`), N-B1/N-E4 (`trim()` et borne non finie, `MN5`, `MN6` tuées), N-B4/N-A9/N-E6 (bornes 1..1440 et singulier
    par `login-session-idle-one`, `MN7`, `MN8` tuées — **écart au mandat** : une clé `-one` et non un sélecteur Fluent, que
    `i18nMsg` n'évalue pas, `i18n.svelte.ts:22-27`, C-v0172s-60), N-B2/N-E8 (manuel utilisateur : quitter la fenêtre pendant que le
    document a la main « peut aussi compter, une fois » — **non éprouvé** : la recette ne simule pas un changement de fenêtre du
    système ; la phrase dit ce que le code ferait si le navigateur émet ce `blur`). S-A5 (pas de gate complet au commit du lot S) :
    constaté, le gate complet du lot S tel que livré est celui de `5392bd37` ; déclaré ici.
  - **LOW non corrigés** : seize points (S-B2/S-A8, S-B3, S-E2 à S-E6, N-B3/N-E7, N-B5, N-A3/N-E3, N-A5, N-A6, N-A8/N-E10, N-A10, N-E5,
    N-E9), plus le constat Rec-e1 de la recette, rédigés pour une issue (`kesh-gate-logs/v0172s/issue-low-p1.md`), ouverte
    par l'orchestrateur le 2026-11-05 : **#898**.
  - **Rejets** : aucun constat rejeté pour prémisse fausse.
  - **Gates** au dernier commit de code (`d38fcd8b`) : `test-fast.sh --ci` **4739/4739** (12 ignorés), `rem-p1-gate-backend.log`
    (2026-11-04 21:40) ; frontend `check` 0 erreur, `lint-i18n-ownership` vert, Vitest **2623/2623** (211 fichiers), `build`
    vert ; `make fr` stable. **E2E complet** : le run r2 a été interrompu vers le test 190 (OOM de la station, GNOME Shell tué
    le 2026-11-04 à 21:48:42) ; le r3, lancé sur une base non remise à neuf, a été arrêté au démarrage et ne compte pas ;
    **r4** (2026-11-05, orchestrateur, `kesh_e2e_v0172s` recréée vide, sous `/tmp/station-heavy.lock` et `.gate.lock`) :
    **343 verts, 7 échecs, 18 ignorés** (15,3 min) — les 7 échecs sont exactement ceux de la KF-029 (#97) listés à
    `docs/testing.md` (`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`,
    `:77`, `:150`) ; aucun autre (`kesh-gate-logs/v0172s/e2e-r4.log`).
- 2026-11-05 — **Revue de code P2 (Opus, trois lentilles par lot, sur le diff de la remédiation) : REVUE CLOSE.** Lot S
  (`11e5a38b..ff20983d`) : B 0/0/0/3, E 0/0/0/4, A 0/0/0/3 ; lot N (`ff20983d..d38fcd8b`) : B 0/0/0/2, E 0/0/0/2, A 0/0/0/2 —
  **aucun finding au-dessus de LOW**, 16 LOW (10 + 6). Tendance : P1 S 0/0/2 MEDIUM (+ S-B1 reclassé), N 0/0/4 → P2 0 MEDIUM.
  Rapports `kesh-gate-logs/v0172s-review-p2-{S,N}-{B,E,A}.md`, prompts `v0172-session-review-prompt-p2-{S,N}.md`.
  - **Corrigés par l'orchestrateur, texte seul** (manuels `.tex` et CHANGELOG ; aucune ligne de code, l'E2E r4 reste celui du
    dernier commit de code ; PDF régénérés à l'intégration de la release) : S2-E1/S2-A2 (« prend fin à sa première activité »
    → « peut prendre fin » : une session de moins de quinze minutes garde sa ligne vivante) ; S2-E4 (« la mise à jour ne
    déconnecte personne » retiré : aucune détection de version dans `svelte.config.js`, un onglet resté sur l'interface 0.17.2
    n'envoie pas de battement → « rechargez chaque onglet de Kesh resté ouvert », manuel d'administration et CHANGELOG) ; S2-E3
    (« seul le navigateur prolonge » → l'interface web, par une route qui exige le cookie de renouvellement, `activity_with`
    lu ; « la durée réglée à ce moment-là ») ; N2-B1 (proposition Alt-Tab du manuel utilisateur retirée : non éprouvée).
    Prémisses lues au code avant correction (`routes/auth.rs`, `activity` et `activity_with` ; `svelte.config.js`). Grep des
    formulations retirées sur le worktree hors `_bmad-output` : vide, sonde positive sur la formulation neuve (2 sites).
  - **LOW non corrigés** (S2-B1, S2-B2, S2-B3, S2-E2, S2-A1, S2-A3, N2-B2, N2-E1, N2-E2, N2-A1, N2-A2) : **#899**, ouverte par
    l'orchestrateur.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (autonomie complète, mandat `kesh-gate-logs/prompts/v0173-session-dev.md`), 2026-11-04.

### Completion Notes List

**Fusion de main** (`0b2e8ec9`) : `origin/main` (v0.17.2, `95e325e7`) fusionnée ; conflits sur `sprint-status.yaml` (lignes de main pour
v0172-annulees et v0172-correctifs, ligne de la branche pour v0172-session), le registre (union : sections de main, section `v0172s`
de la branche), `v0172-correctifs.md` (main), `v0172-session.md` (branche). Aucun marqueur restant ; 88 migrations ; crates en 0.17.2
(non bumpés : la release le fera).

**Étape 0** (`1e50fd03`) : les sept LOW de la validation R5 appliqués ; numéros de ligne relus après la fusion (Change Log).

**Rouges** (`kesh-gate-logs/v0172s/rouges.log`) — lot S, sur le code d'avant : 21 tests, **18 rouges** (T1 : 4 ; T3 : 12 ; T4 (a) ;
T5 (a) — 4 + 12 + 1 + 1 = 18, recompté depuis `rouges-S.log` à la revue de code P1 ; « T3 : 13 » écrit au développement),
**3 verts attendus** (T4 (b), (c) ; T5 (b)). Lot N, implémentation retirée (`git stash` des huit fichiers, deux fichiers neufs
mis de côté, restaurés ensuite) : 7 fichiers rouges, dont 10 tests rouges et les deux fichiers neufs (module absent) ; AC-7 (3) vert
avant (cf. Change Log).

**Mutations** (sous le verrou, restauration `git checkout --` puis `touch`) : lot S **19/19 tuées** (« 21/21 » écrit au développement, recompté à la revue de code P1 :
`mut/S-resume.log` porte 19 lignes `TUÉE` ; **40 mutations en tout**, 19 + 21) (M1, M1 bis, M2, M3, M3 bis, M4,
M5, M6, M6 bis, M7, M7 bis, M8, M8 bis, M15, M15 bis, M16, M16 bis — l'extracteur du battement —, M17, M22) ; lot N **21/21 tuées**
(M9 à M14, M13 bis, M12 bis, M18 à M21, M23 à M30, M26 bis). Chacune par le test attendu (`mut/M*.log`). M11 est jouée comme « lecture
en mémoire seulement » (la clé d'un autre onglet est ignorée) : tuée par AC-6 (6).

**Gates**, base remise à neuf avant chaque gate complet :
- `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest) au commit `5392bd37` : **4738 exécutés, 4737 verts, 1 rouge**
  (`kesh-i18n` `la_cloture_d_exercice_ne_parle_pas_comme_un_panneau` : « Sitzung schliessen ») — corrigé (`b7f02678`), `kesh-i18n` 41/41 ;
  gate complet rejoué au dernier commit de code (`b7f02678`, base remise à neuf) : **4738 exécutés, 4738 verts, 12 ignorés**
  (`gate-backend-2.log`).
- `cargo build --workspace --all-targets` : vert (`build-1.log`).
- Frontend : `npm run check` 0 erreur (29 avertissements, aucun dans les fichiers de la story), `lint-i18n-ownership` vert,
  `test:unit` **211 fichiers, 2586 tests verts**, `build` vert.
- `make fr` : trois PDF stables ; PDF aplatis contrôlés (phrases neuves présentes, `KESH_REFRESH_INACTIVITY`, « 135 des 142 » et « La
  session reste active 15 minutes » absents).
- **E2E complet** (port 3174, `kesh_e2e_v0172s`, `/tmp/kesh-e2e-v0172s-r1/`) : **343 passés, 7 échecs, 18 ignorés** ; les 7 échecs
  sont exactement les sept KF-029 (#97) de `docs/testing.md` (`mode-expert.spec.ts:26, :41`, `onboarding-path-b.spec.ts:65, :92`,
  `onboarding.spec.ts:57, :77, :150`). `ss` avant et après : 0 écoute sur 3174 ; backend arrêté par son PID.

**Recette de durée** (AC-10, `recette-r3.log`, base `kesh_v0172s` jetable, `KESH_JWT_EXPIRY_MINUTES=1`, l'ancienne variable posée à 15
— ignorée, volet (h)) : réglage porté à 2 par l'écran ; (a) mouvements de souris 3 min (19:31:56 → 19:34:57) puis `POST /contacts` 201,
`/me` 200 (`idleTimeoutMinutes = 2`) ; (d) 90 s d'activité puis rechargement : connecté ; (b) dernière activité 19:36:27.862, fermeture
19:38:28.416 — **120,55 s** (attendu 120 ± 2 s), message « Session fermée après 2 minutes d'inactivité. », `curl` avec l'ancien cookie
d'accès → **401**, dernière ligne `refresh_tokens` révoquée `logout`, rechargement de `/login` : reste sur la page ; (b bis) abaissement
2 → 1 à 19:38:30.798, fermeture 19:39:31.773 (**60,98 s**), `/` mène à la connexion, ligne `logout` ; (c) deux onglets, activité dans
B seul 3 min : A toujours connecté, `/me` 200. Deux essais préalables avortés par le script lui-même (r1 : mot de passe réécrit par le
bootstrap ; r2 : sélecteur du journal d'audit), corrigés dans `scripts/recette-v0172s.mjs`. **Non joués** au développement : (e) l'aperçu PDF et (f) la restauration complète (couverte par
`admin_full_import_e2e.rs`, T5). *(Revue de code P1, N-A2 : « (e) couvert par les tests du `blur` » surestimait — ces tests ne
couvrent que le clic qui entre, simulé. (e) a été **joué en navigateur** à la remédiation : fermeture à 120,67 s et 120,55 s
malgré la molette dans le document, session gardée 271,8 s par le geste du manuel ; détail au Change Log, entrée « Revue de code
P1 ». (f) reste non joué en navigateur ; C-v0172s-61.)*

**Captures** (`kesh-gate-logs/v0172s/captures/`, thème clair, base jetable) : `v0172s-01-parametres-session-60`,
`-02-saisie-hors-bornes-refusee`, `-03-enregistrement-reussi-2`, `-04-parametres-comptable-sans-session`,
`-05-connexion-session-fermee-2-min`, `-06-journal-audit-reglage` ; à la remédiation P1 : `-07-apercu-pdf` (le visualiseur PDF
rendu), `-08-apercu-pdf-fermeture`.

**Sondes** : `git grep -n "jwt::encode(" -- crates/kesh-api/src` ne rend que les tests en module de `middleware/auth.rs` (sous
`#[cfg(test)]`) ; `git grep -nE 'refresh_inactivity|REFRESH_INACTIVITY' -- crates frontend scripts docs/manual .env.example
docker-compose.yml docker-compose.prod.yml` ne rend que le paramètre ignoré de `from_fields_for_test` et `crates/kesh-api/README.md` ;
`LoginPayload` : trois constructeurs (`login/+page.svelte`, `setup.api.ts`, `doRefresh`) et `hydrate()` transmettent les deux champs ;
aucune spec Playwright ne compte toutes les requêtes (`homepage-reminders.spec.ts:78` filtre par URL) ; aucun test ne montait le layout
racine avant `root-layout-session.test.ts` (qui simule `sessionIdle`).

**Décomptes** (périmètre `1e50fd03` → tête) : 27 attributs de test Rust ajoutés, 3 retirés (`config_refresh_inactivity_*`) ; 44 `it(`
Vitest ajoutés.

**Remédiation de la revue de code P1** (Opus 5.5, mandat `kesh-gate-logs/prompts/v0173-session-rem-p1.md`) — commits `ff20983d`
(lot S), `ac666759`, `8d6561bc`, `d38fcd8b` (lot N). Détail des constats, prémisses lues et rejets au Change Log.
- **Rouge avant correctif** : `concurrent_first_puts_yield_a_conflict_not_a_500` → `500 INTERNAL_ERROR` (`rem-p1-rouge-S1.log`).
- **Mutations** (`kesh-gate-logs/v0172s/mut-rem-p1/`, script `scripts/mut-v0172s-rem-p1.py`, motif unique, restauration
  `git checkout --` puis `touch`) : `MS2`, `MN1`, `MN1bis`, `MN2`, `MN3`, `MN4` (survivante au premier jeu, tuée après renforcement
  du test), `MN4bis`, `MN5`, `MN6`, `MN7`, `MN8` — **11 tuées** sur 11 au dernier jeu, chacune par le cas attendu ; `MS1` (rejeu retiré)
  est le rouge avant correctif ci-dessus.
- **Décomptes** (périmètre `11e5a38b` → `d38fcd8b`) : 1 attribut de test Rust ajouté (le test de course ; `heartbeat_refuses_api_keys`
  étendu), 0 retiré ; 11 `it(` Vitest littéraux ajoutés (dont deux dans des boucles), 0 retiré ; `sitesTotal` 2825 → 2826 ; une clé
  i18n par locale (`login-session-idle-one`).
- **Recette (e)** : jouée (Change Log). **Non éprouvé** : le `blur` d'un changement de fenêtre du système (Alt-Tab) pendant que
  l'aperçu a le focus — la phrase du manuel utilisateur dit « peut aussi compter », ce que le code ferait si le navigateur émet ce
  `blur`.
- **Gates** au dernier commit de code : voir l'entrée « Revue de code P1 » du Change Log.

**Actions demandées à l'orchestrateur** : push et PR (`refs #876`, `refs #891` ; `closes` éventuel au titre de la PR seulement) ; issue
pour retirer en lot mécanique le paramètre ignoré `_refresh_inactivity_ignored` de `from_fields_for_test` (fiche § 12) ; revues de code
P1 par lot (`v0172-session-review-prompt-p1-S.md`, `-p1-N.md`, Sonnet, trois lentilles chacun).

### File List

Lot S : `crates/kesh-db/migrations/20261104000001_installation_settings.sql` (neuf), `crates/kesh-db/migrations.sha384`,
`crates/kesh-db/test-schema/0001_schema_squash.sql` (régénéré), `crates/kesh-db/src/repositories/{installation_settings.rs (neuf),
mod.rs, refresh_tokens.rs}`, `crates/kesh-db/src/{backup.rs, test_fixtures.rs}`, `crates/kesh-db/tests/migrations_upgrade_path.rs`,
`crates/kesh-api/src/{auth/jwt.rs, auth/api_key.rs, middleware/auth.rs, helpers.rs, config.rs, main.rs, lib.rs, audit_labels.rs,
routes/auth.rs, routes/setup.rs, routes/admin.rs, routes/session_settings.rs (neuf), routes/mod.rs}`, `crates/kesh-api/README.md`,
`crates/kesh-api/tests/{session_idle_e2e.rs (neuf), session_settings_e2e.rs (neuf), auth_e2e.rs, auth_cookies_e2e.rs,
admin_full_import_e2e.rs, admin_pat_denied_e2e.rs, audit_route_registry.rs, configuration_transmise.rs, textes_coherents.rs,
services_motif.rs, services_rollout.rs}`, `crates/kesh-i18n/locales/*/messages.ftl`, `.env.example`, `docker-compose.yml`,
`docker-compose.prod.yml`, `docs/migrations-idempotence-audit.md`, `docs/manual/fr/{admin,user}-manual.tex` (sites gardés).
Lot N : `frontend/src/lib/app/stores/{session-idle.svelte.ts (neuf), session-idle.svelte.test.ts (neuf), auth.svelte.ts,
auth.svelte.test.ts}`, `frontend/src/lib/shared/utils/api-client.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`,
`frontend/src/lib/features/settings/{SessionIdleSection.svelte (neuf), SessionIdleSection.test.ts (neuf), settings.api.ts,
settings.types.ts}`, `frontend/src/lib/features/setup/{setup.api.ts, setup.api.test.ts (neuf)}`, `frontend/src/routes/{+layout.svelte,
root-layout-session.test.ts (neuf), login/+page.svelte, login/login-page.test.ts (neuf), (app)/settings/+page.svelte,
(app)/settings/settings-page.test.ts}`, `crates/kesh-i18n/locales/*/messages.ftl`, `docs/manual/fr/*.tex` et `*.pdf`, `CHANGELOG.md`,
`_bmad-output/planning-artifacts/{prd,epics}.md`, `docs/kesh-specifications.txt`. Comptes rendus : cette fiche, le registre,
`sprint-status.yaml`, les deux prompts de revue.
