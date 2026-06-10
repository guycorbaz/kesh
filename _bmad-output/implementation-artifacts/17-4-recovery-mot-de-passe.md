# Story 17.4: Recovery de mot de passe self-service (forgot-password email magic-link)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->
<!-- STORY UMBRELLA (parente) — split quasi-certain au `bmad-create-story validate` (cf. CLAUDE.md §Règle de splitting préventif : >5 modules — kesh-db migration/repo, kesh-api config/mail/routes, frontend pages/i18n, doc). Découpage pressenti 17-4a..17-4f aligné sur les Parties A–F ci-dessous. Le contrat token + format de l'email (transverse backend↔frontend) est conçu d'un bloc ici ; les frontières sous-story se cristallisent au validate (pattern 17-2 / 17-3). -->

## Story

As a **utilisateur d'une installation Kesh (admin solo ou user multi-tenant) qui a oublié son mot de passe**,
I want **demander une réinitialisation via un lien magique reçu par email, et définir un nouveau mot de passe depuis une page publique**,
so that **je récupère l'accès à mon compte en autonomie, sans accès SSH/Docker au NAS ni intervention d'un autre admin**.

## Contexte & cadrage (à lire avant tout)

**Issue source :** [#122](https://github.com/guycorbaz/kesh/issues/122) (`enhancement`, `triage`, `v0.2-milestone`). « Recovery mot de passe production-grade (forgot-password email + alternatives) ». Scope déclaré dans l'issue : **Large (>3 days, likely needs a PRD/spec)**.

**⚠️ Décision de périmètre (Guy, 2026-06-10) — CŒUR RECOVERY SEUL pour 17-4 :**

L'issue #122 énumère **6 composantes**. Cette épopée 17-4 ne livre que le **cœur shippable** ; les autres sont **explicitement reportées** (catégorie B, limitations documentées ci-dessous, à re-trier en rétro Epic 17 / backlog v0.2+) :

| # | Composante #122 | 17-4 ? | Note |
|---|---|---|---|
| 1 | Forgot-password email (magic-link) | ✅ **IN** | Cœur |
| 2 | Config SMTP (env vars + fail-fast boot) | ✅ **IN** | Cœur |
| 3 | Champ `email` sur `users` (migration + wizard + UI) | ✅ **IN** | Prérequis du match recovery |
| 4 | Notifications email (password changed, login nouvelle IP, anti-bruteforce) | ❌ **OUT → L1** | Sauf l'email magic-link lui-même (composante 1). Le mail « votre mot de passe a été changé » = L1 v0.2+. |
| 5 | 2FA TOTP | ❌ **OUT → L2** | Épopée dédiée v0.2+/v0.3. |
| 6 | Account lockout / throttling avancé | ⚠️ **PARTIEL** | Le **rate-limiting anti-énumération** de l'endpoint forgot-password EST dans le cœur (réutilise `RateLimiter` existant). Le lockout multi-facteur avancé (reuse-token = compromis → alerte, throttle 24h glissant) = **L3** v0.2+. |
| — | Alternatives (SSO/OAuth, recovery codes) | ❌ **OUT → L4** | Hors scope, stories séparées éventuelles. |

**Compatibilité break-glass #121 (CONSERVÉE) :** le mécanisme `KESH_ADMIN_RESET` (Issue #121, livré v0.1.1, `auth/bootstrap.rs:219-302` case 5) **reste le fallback offline** quand SMTP n'est pas configuré (`KESH_FEATURE_FORGOT_PASSWORD=false`). Les deux systèmes coexistent sans interaction (vars d'env disjointes, actions audit distinctes). 17-4 **ne touche pas** le break-glass.

**⚠️ Découverte ground-truth critique (anti-réinvention) :**
- **Aucune infrastructure email/SMTP n'existe** dans le workspace (grep `smtp`/`lettre`/`Message::builder` = 0 hit). Greenfield : pas de code à refactorer, mais **toute la couche envoi est à créer** (lib `lettre` à ajouter, cf. DC2).
- **Aucune colonne `email` sur `users`** (`migrations/20260404000001_initial_schema.sql:24-37` + `20260419000002_users_company_id.sql` ; entité `User` `crates/kesh-db/src/entities/user.rs:108`). Le seul `email` existant est sur `contacts` (donnée métier, pas auth). → migration `ADD COLUMN email` à créer.
- **Aucun moteur de templating** (askama/tera/handlebars/minijinja absents). Les emails seront **formatés inline** (`format!`) avec i18n Fluent (cf. DC10).
- Brique **token bearer hashé SHA-256** déjà éprouvée par les **clés PAT 17-2a** (`api_keys.key_hash CHAR(64)`, OsRng + SHA-256 indexé, DC1) → **réutiliser le pattern** pour `password_reset_tokens.token_hash` (DC3), ne PAS réinventer.
- `RateLimiter` custom par IP existe déjà (`crates/kesh-api/src/middleware/rate_limit.rs:35`, utilisé par login) → **réutiliser** pour l'anti-énumération forgot-password (DC5), ne PAS ajouter `tower_governor`.

**Migrations DB :** 17-4 introduit **2 migrations NON-breaking** (cf. CLAUDE.md §Migration breaking policy) :
- `ADD COLUMN email VARCHAR(255) NULL` sur `users` → **non-breaking** (nullable, ignoré par anciens binaires). Pas de bump `kesh_version_min_required`.
- `CREATE TABLE password_reset_tokens` → **non-breaking** (nouvelle table). Pas de bump.
- ⇒ **2 lignes à ajouter** dans `docs/migrations-idempotence-audit.md` (verdict `yes` si `IF NOT EXISTS`, sinon `tracked-by-sqlx`) — sinon finding MEDIUM en code-review (P5 garde-fou).

**⚠️ Couplage 17-3 export/import (épopée sœur, mergée PR #171) :**
- L'export `.keshbackup` sérialise dynamiquement les `columnNames` → la **nouvelle colonne `users.email` est incluse automatiquement** (aucun code à toucher). La compat cross-version est gérée par le double-check colonnes 17-3 AC12c.
- La **nouvelle table `password_reset_tokens` DOIT être ajoutée à `TABLES_TO_TRUNCATE`** (`crates/kesh-db/src/backup.rs:34`, enfant de `users`) — **sinon le test `backup_inventory_matches_schema` (`backup.rs:567`) échoue** (schéma ≠ liste hardcodée). Tâche explicite T-A4. (Position : après `refresh_tokens`, autre table éphémère enfant de `users`, avant `onboarding_state`.)

## Décision de split (pressentie — à confirmer au validate Pass 1)

Découpage **A–F**, 6 sous-stories (pattern 17-3) :
- **17-4a** DB foundation + champ email backend — **story-zéro** : pose la migration `email`, la table `password_reset_tokens`, les entités/repos (`find_by_email`, repo tokens), l'ajout `email` aux DTO `setup/admin` + `users` CRUD, l'ajout à `TABLES_TO_TRUNCATE`. **Doit merger en premier** (tout en dépend).
- **17-4b** Couche email/SMTP + config — module `mail/` (DC1), lib `lettre` (DC2), env vars `KESH_SMTP_*` + `KESH_FEATURE_FORGOT_PASSWORD` + `KESH_PUBLIC_BASE_URL`, fail-fast boot, **trait `Mailer` injectable** (testabilité, no-SMTP en test). Parallélisable avec 17-4a (ne dépend pas de la DB).
- **17-4c** Backend endpoints — `POST /api/v1/auth/forgot-password` + `POST /api/v1/auth/reset-password` (routes **publiques**), génération/validation token, rate-limit anti-énum, audit, exposition feature-flag (`/health` ou endpoint config public). Dépend de 17-4a + 17-4b.
- **17-4d** Frontend — pages publiques `/forgot-password` + `/reset-password?token=`, champ `email` dans `SetupForm` + dialogues users CRUD, i18n ×4, lien « mot de passe oublié ? » conditionnel sur le login. Dépend de 17-4c (contrat).
- **17-4e** Tests E2E/intégration — happy path, token expiré, token réutilisé, user inexistant (anti-énum), SMTP down, reset → invalidation refresh tokens.
- **17-4f** Doc — manuel admin (config SMTP), manuel user (workflow recovery), CHANGELOG, README, `.env.example`.

**17-4a et 17-4b parallélisables** ; 17-4c est le point de jointure.

## Décisions de conception (DC)

> Les DC marquées **FIGÉE** sont tranchées ; les **À TRANCHER** sont à valider/affiner au `validate`.

- **DC1 — Emplacement de la couche email** *(À TRANCHER — recommandation : module `crates/kesh-api/src/mail/` PLUTÔT qu'une nouvelle crate `kesh-mail`)*. Rationale recommandation : un seul type d'email transactionnel (magic-link), couplage fort à `config` + i18n + `AppError` (tous dans kesh-api) ; une crate dédiée = sur-ingénierie (cf. `kesh-payment` placeholder vide). Si le validate juge la réutilisabilité/test-isolation suffisante → crate. **Frontière exacte décidée au validate.**
- **DC2 — Lib SMTP : `lettre` 0.11** *(FIGÉE)*, features `["tokio1-native-tls", "smtp-transport", "builder"]` (async via le runtime tokio `["full"]` déjà présent). De-facto standard Rust, maintenu. **+1 dépendance Cargo** (assumée, justifiée — aucune alternative in-tree). Pas de `reqwest` (pas d'API HTTP tierce ; SMTP direct).
- **DC3 — Stockage token : SHA-256 hashé indexé** *(FIGÉE, calque PAT 17-2a DC1)*. Token brut = **32 octets OsRng** encodés base64url (réutilise `OsRng` réexporté via `argon2`, pattern 17-2a — **0 nouvelle dép**). Stocké en DB **uniquement** sous forme `SHA-256` hex `CHAR(64)` (`token_hash`, `UNIQUE`). Le brut ne vit que dans l'URL de l'email. Rationale : une fuite DB ne doit pas permettre la prise de contrôle de compte (le token est un bearer credential). SHA-256 suffit (entropie 256 bits, pas besoin d'Argon2 per-lookup — cf. 17-2a DC1).
- **DC4 — Anti-énumération : `POST /forgot-password` retourne TOUJOURS `200`** *(FIGÉE)* avec un corps générique (« Si un compte correspond, un email a été envoyé »), **que l'utilisateur existe ou non**. Jamais de signal distinguant existant/inexistant (ni status, ni timing grossier, ni message). Travail constant (lookup + éventuel envoi en tâche détachée). L'énumération de masse est bridée par le rate-limit DC5.
- **DC5 — Rate-limiting : réutiliser `RateLimiter` par IP** *(FIGÉE)*. Une **instance distincte** dédiée à forgot-password (seuils propres, ex. 5/15min, configurables ou défauts), montée comme l'instance login (`crates/kesh-api/src/middleware/rate_limit.rs`). Dépassement → `429`. Pas de `tower_governor`.
- **DC6 — Match utilisateur** *(À TRANCHER au validate — proposition)*. `users.email` est **nullable et NON-unique** (multi-tenant : deux users de companies distinctes peuvent partager un email). Proposition : (a) si l'input ne contient pas `@` → lookup par `username` (contrainte `UNIQUE`, ≤1 résultat) ; (b) si l'input contient `@` → lookup par `email` ; si **exactement 1** match → procéder, si **0 ou >1** → traiter comme « pas de match » (aucun email, mais `200` anti-énum DC4). Un compte sans email renseigné est **non-recouvrable par ce flux** (→ break-glass #121). **À affiner au validate** (faut-il une contrainte `UNIQUE(email)` partielle ? non recommandé en multi-tenant).
- **DC7 — Feature flag `KESH_FEATURE_FORGOT_PASSWORD` (défaut `false`)** *(FIGÉE)*. Si `false` : les routes `/forgot-password` + `/reset-password` ne sont **pas montées** (404) et le lien frontend est masqué (cf. DC9) ; recovery = break-glass #121. Si `true` : la config SMTP **complète est requise au boot** (fail-fast, cf. AC). Parsing strict bool (`"true"`/`"1"`/empty, pattern `KESH_TEST_MODE` `config.rs:701`).
- **DC8 — Token : usage unique + TTL 30 min + révocation refresh** *(FIGÉE)*. `expires_at = now + 30min` (chrono `TimeDelta::minutes(30)`, pattern `jwt.rs:73`). Au reset réussi : `used_at = NOW(3)` (consommation), `users::update_password`, **puis `refresh_tokens::revoke_all_for_user(user_id, "password_reset")`** (nouvelle raison whitelistée). Un token déjà `used_at IS NOT NULL` ou `expires_at < now` → `400 INVALID_OR_EXPIRED_TOKEN`. (Reuse-detection avancée = L3.)
- **DC9 — Exposition du feature-flag au frontend** *(À TRANCHER — proposition : étendre `GET /health`)*. Le login doit afficher le lien « mot de passe oublié ? » seulement si la feature est active. Proposition : ajouter `forgotPasswordEnabled: bool` au corps de `/health` (déjà public, déjà consommé par le frontend pour la version). Alternative : endpoint `/api/v1/config` public. **Décidé au validate.**
- **DC10 — Localisation de l'email** *(FIGÉE pour le cœur)*. Contenu email formaté **inline** (`format!`) + i18n via le bundle Fluent global `config.locale` (`KESH_LANG`). **Pas de locale par-utilisateur** (la colonne n'existe pas) → email dans la langue de l'instance. (Locale per-user = v0.3, noté L1-adjacent.)
- **DC11 — `password_reset_tokens` : FK `ON DELETE CASCADE`** *(FIGÉE)*. Différent des `api_keys` (`RESTRICT`) car les tokens sont éphémères/secrets sans valeur d'audit propre (l'audit du reset vit dans `audit_log`, FK `RESTRICT` séparée). Supprimer un user purge ses tokens pendants.
- **DC12 — Migrations non-breaking** *(FIGÉE)*. `ADD COLUMN email` nullable + `CREATE TABLE` → pas de bump `kesh_version_min_required` (cf. §Migration breaking policy P1/P3). 2 lignes idempotence-audit (P5).

## Acceptance Criteria

> ACs groupés par **Partie A–F** (= frontières de split). Numérotation continue pour traçabilité.

### Partie A — DB foundation + champ email backend (story-zéro)

1. **Migration `email`** : `ALTER TABLE users ADD COLUMN IF NOT EXISTS email VARCHAR(255) NULL` (dialecte MariaDB, style `migrations/20260419000001_invoice_paid_at.sql`). Index non-unique `idx_users_email (email)` pour le lookup recovery. **Backward-compatible** (existants → `email = NULL`). Entité `User` (`entities/user.rs:108`) + `COLUMNS`/`FromRow` étendus de `pub email: Option<String>`.
2. **Migration `password_reset_tokens`** : `CREATE TABLE password_reset_tokens (id BIGINT AUTO_INCREMENT PK, user_id BIGINT NOT NULL, token_hash CHAR(64) NOT NULL, expires_at DATETIME(3) NOT NULL, used_at DATETIME(3) NULL, created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3), CONSTRAINT fk_prt_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE, CONSTRAINT uq_prt_token_hash UNIQUE (token_hash), INDEX idx_prt_user (user_id), INDEX idx_prt_expires (expires_at)) ENGINE=InnoDB … utf8mb4_unicode_ci` (style `migrations/20260605000001_api_keys.sql`). Entité `PasswordResetToken` + repo dédié (cf. AC4).
3. **Repo users** : ajouter `find_by_email(pool, email) -> Result<Vec<User>, DbError>` (retourne **Vec** car non-unique, DC6) à `crates/kesh-db/src/repositories/users.rs`. `update_password` (déjà existant, `users.rs:270`) **réutilisé tel quel** au reset.
4. **Repo password_reset_tokens** (`crates/kesh-db/src/repositories/password_reset_tokens.rs`, nouveau) : `create(pool, user_id, token_hash, expires_at)`, `find_valid_by_hash(pool, token_hash) -> Option<PasswordResetToken>` (filtre `used_at IS NULL AND expires_at > NOW(3)`), `mark_used(pool, id)`, `invalidate_all_for_user(pool, user_id)` (optionnel : invalider les tokens pendants quand un nouveau est demandé). Insertions/updates **paramétrés** (sqlx `query`/`bind`).
5. **Champ `email` plumbing backend (sans UI)** : DTO `setup/admin` (`routes/setup.rs`) et `users` CRUD (`routes/users.rs` create/update) acceptent un `email: Option<String>` optionnel, validé par le helper email existant (`is_valid_email_simple` réutilisé de `routes/contacts.rs`) si présent (non-vide ⇒ format valide, sinon `400 VALIDATION_ERROR`). `UserResponse` expose `email`. **Le wizard onboarding pose l'email de l'admin initial** (plumbing backend ici, champ UI en 17-4d).
6. **`TABLES_TO_TRUNCATE`** : ajouter `"password_reset_tokens"` (`crates/kesh-db/src/backup.rs:34`, après `"refresh_tokens"`), pour que la table soit incluse dans export/import 17-3 et que `backup_inventory_matches_schema` (`backup.rs:567`) reste vert. **Sans cet ajout, le test échoue.**
7. **Idempotence-audit** : 2 lignes ajoutées à `docs/migrations-idempotence-audit.md` (les 2 nouvelles migrations + verdict + justification).

### Partie B — Couche email/SMTP + config

8. **Env vars SMTP** (`config.rs`, patterns `config.rs:467` string / `:701` bool / `:747` int borné) :
   - `KESH_SMTP_HOST` (opt string, trim+filter-empty),
   - `KESH_SMTP_PORT` (opt int **bornes [1, 65535]**, défaut **587**, parse+borne+warn),
   - `KESH_SMTP_USER` (opt string),
   - `KESH_SMTP_PASSWORD` (opt string — **masqué dans `Debug`** : champ privé ou custom `Debug`, calque `jwt_secret` `config.rs:156`),
   - `KESH_SMTP_FROM` (opt string, validé email via `is_valid_email_simple`),
   - `KESH_SMTP_TLS` (strict bool, défaut `true` — STARTTLS ; *valeur exacte STARTTLS-vs-implicit à confirmer côté `lettre` au dev*),
   - `KESH_PUBLIC_BASE_URL` (opt string, ex. `https://kesh.example.com` — **requis si feature on**, sert à construire le lien magique absolu de l'email),
   - `KESH_FEATURE_FORGOT_PASSWORD` (strict bool, défaut `false`, DC7).
9. **Fail-fast boot (DC7)** : si `KESH_FEATURE_FORGOT_PASSWORD=true` mais qu'**une** des vars requises (`SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM`, `PUBLIC_BASE_URL`) est absente/vide, ou `SMTP_FROM` n'est pas un email valide → nouvelle `ConfigError` variant + `Config::from_env()` retourne `Err` → `main.rs:62` `std::process::exit(1)` avec message clair. Si `false` : SMTP optionnel, pas de validation.
10. **Module `mail/`** (DC1) : `Mailer` **trait** (`async fn send_password_reset(&self, to: &str, reset_url: &str, locale) -> Result<(), AppError>`) + impl `SmtpMailer` (lettre, lit la config SMTP) + impl `NoopMailer`/`MockMailer` (tests, capture le mail sans I/O réseau). Le `Mailer` est porté par `AppState` (Arc dyn). Contenu email formaté inline + i18n Fluent (DC10).
11. **`AppError`** : variant `SmtpSendFailed(String)` (mappé `500 SMTP_SEND_FAILED`, log `tracing::error!`, i18n `error-smtp-send-failed`) + `ForgotPasswordDisabled` si nécessaire (cf. DC7 — route non montée préférée à un variant). Pattern `errors.rs` (enum + `IntoResponse` + clé Fluent `error-*` ×4 locales).

### Partie C — Backend endpoints publics

12. **`POST /api/v1/auth/forgot-password`** (route **publique**, montée à côté de `/api/v1/auth/login` `lib.rs:493`, **seulement si `KESH_FEATURE_FORGOT_PASSWORD=true`**) : corps `{ identifier: string }` (username ou email). **Rate-limited par IP (DC5)** → `429` si dépassement. Lookup (DC6) ; si match unique → génère token brut (32o OsRng base64url), stocke `SHA-256` (`password_reset_tokens`), construit `reset_url = {KESH_PUBLIC_BASE_URL}/reset-password?token={brut}`, envoie l'email via `Mailer` (idéalement en tâche détachée pour timing constant). **Retourne TOUJOURS `200`** corps générique (DC4), que le user existe ou non. Audit `auth.password_reset_requested` (`entity_type` `user`, `details_json` : `identifier_kind` username|email, `matched` bool — **interne, jamais renvoyé au client**) **uniquement si match** (un non-match ne crée pas d'entrée user_id).
13. **`POST /api/v1/auth/reset-password`** (route **publique**, même gating) : corps `{ token: string, new_password: string }`. Hash SHA-256 du `token` reçu → `find_valid_by_hash` (filtre `used_at IS NULL AND expires_at > NOW(3)`). Si absent/expiré/utilisé → `400 INVALID_OR_EXPIRED_TOKEN` (générique, pas de fuite). Valide le nouveau mot de passe (`password::validate_password` + `config.password_min_length`, `auth/password.rs:113`). **Dans une transaction** : `mark_used(token.id)` + `update_password(user_id, hash)` + audit `auth.password_reset_completed` (`NewAuditLogEntry::user(user_id, …)`) ; **après commit** : `refresh_tokens::revoke_all_for_user(user_id, "password_reset")` (raison à whitelister dans la contrainte `chk_refresh_tokens_revoked_reason` — **migration de contrainte à vérifier**, cf. Dev Notes). Retourne `200` succès.
14. **Whitelisting raison révocation** : `"password_reset"` ajoutée à la contrainte CHECK `chk_refresh_tokens_revoked_reason` (`migrations/20260406000001_*`). ⚠️ **Modifier une contrainte CHECK existante** = `DROP CONSTRAINT` + `ADD CONSTRAINT` (vérifier non-breaking : élargir l'ensemble de valeurs autorisées est non-breaking pour un ancien binaire qui n'émet jamais cette valeur ; documenter idempotence). *(Alternative sans migration : réutiliser `"password_change"` existant — à trancher au validate ; le distinguo audit reste via `audit_log.action`.)*
15. **Exposition feature-flag (DC9)** : `GET /health` (ou endpoint public dédié) expose `forgotPasswordEnabled: bool` pour que le frontend conditionne l'affichage du lien.
16. **Sécurité endpoints** : routes **hors** middleware `require_auth` (publiques, pré-login). Pas de scope PAT (non concerné, ce sont des routes publiques non authentifiées). Cohérence anti-énum DC4 sur les **deux** endpoints (reset-password ne distingue pas token-inexistant de token-expiré).

### Partie D — Frontend

17. **Page publique `/forgot-password`** (`frontend/src/routes/forgot-password/+page.svelte`, **pas** sous `(app)`, pas de guard auth — pattern `/login`) : un champ « nom d'utilisateur ou email », bouton « Envoyer le lien de réinitialisation », POST `auth-recovery.api.ts → requestPasswordReset(identifier)`. Affiche **toujours** le message générique de succès (anti-énum DC4), gestion `429`.
18. **Page publique `/reset-password?token=`** (`frontend/src/routes/reset-password/+page.svelte`, publique) : lit le `token` du query param, champs « nouveau mot de passe » + « confirmer », POST `resetPassword(token, newPassword)`. Succès → message + redirection `/login`. Erreurs typées : `400` token invalide/expiré → message « lien invalide ou expiré », `429`, validation mdp. **HTTP-LAN safe** : `$props.id()` pour IDs DOM, aucune API secure-context-only (cf. `feedback_no_secure_context_apis_http_lan`, bugs #143/#145).
19. **Feature API module** `frontend/src/lib/features/auth-recovery/` : `auth-recovery.api.ts` (`requestPasswordReset`, `resetPassword`) + `.types.ts` + tests (pattern `contacts.api.ts` / `setup.api.ts`).
20. **Lien « Mot de passe oublié ? »** sur `/login` (`routes/login/+page.svelte`), **conditionné** sur `forgotPasswordEnabled` (DC9, lu via health/config au chargement). Masqué si feature off.
21. **Champ `email` dans `SetupForm`** (`lib/features/setup/SetupForm.svelte`, après les champs password ; `setup.api.ts:setupAdmin` signature étendue `email`) **et** dans les dialogues create/edit de `routes/(app)/users/+page.svelte` (state `createEmail`/`editEmail`, POST/PUT body étendus). Validation format email côté UI (présence `@`) + serveur (AC5).
22. **i18n ×4 locales** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`) : clés `forgot-password-*`, `reset-password-*`, `user-field-email`/`user-email-invalid`, respectant `lint-i18n-ownership` (feature-scoped ou namespace global `error-*`). `npm run lint-i18n-ownership` PASS.

### Partie E — Tests E2E / intégration

23. **Tests d'intégration Rust** (backend, `crates/kesh-api/tests/`) couvrant : (a) **happy path** forgot → token créé → reset → login avec nouveau mdp OK + anciens refresh tokens révoqués ; (b) **token expiré** → `400` ; (c) **token réutilisé** (`used_at` set) → `400` ; (d) **user inexistant** → `200` générique + **aucun** token créé (anti-énum) ; (e) **email non renseigné** → `200` + aucun email ; (f) **rate-limit** dépassé → `429` ; (g) **SMTP down** (via `MockMailer` qui erreure) → le `forgot-password` reste `200` côté client (l'échec d'envoi est loggé serveur, ne révèle rien) **OU** `500` selon décision timing — *à trancher au validate* (anti-énum vs feedback erreur). Utiliser `MockMailer` (DC10/AC10) — **aucun envoi SMTP réel en CI**.
24. **E2E Playwright** (frontend, `frontend/e2e/`) : parcours UI forgot-password (saisie → message succès) + reset-password (token mocké → nouveau mdp → redirection login). Le lien email réel n'est pas vérifiable sans SMTP → le token est injecté via seed/API de test. *(Vérification de réception email réelle sur 3+ providers [Gmail/Postfix/Mailgun/Synology Mail] = test manuel hors-CI documenté, AC #122 — noté dette de validation manuelle, pas bloquant CI.)*

### Partie F — Documentation

25. **Manuel admin LaTeX FR** (`docs/manual/fr/admin-manual.tex` + PDF régénéré `latexmk -xelatex`) : nouvelle section « Configuration du recovery par email (SMTP) » — vars `KESH_SMTP_*` + `KESH_FEATURE_FORGOT_PASSWORD` + `KESH_PUBLIC_BASE_URL`, exemples (Gmail SMTP, Postfix local, Synology Mail Server), **et** rappel du fallback break-glass #121 quand SMTP indisponible.
26. **Manuel user LaTeX FR** (`docs/manual/fr/user-manual.tex` + PDF) : workflow « J'ai oublié mon mot de passe » côté utilisateur final (demande → email → reset).
27. **`.env.example`** : nouvelle section « Recovery par email (SMTP) » avec les 8 vars commentées + warning (secret SMTP password) + note « si non configuré, recovery = break-glass `KESH_ADMIN_RESET` cf. section Recovery ».
28. **`CHANGELOG.md`** (`Added`) + **`README.md`** (« Feuille de route » + « Fonctionnalités », retirer `(à venir)` recovery, statut Epic 17) pour v0.2.x.

### Transverses (toutes parties)

29. **Sécurité** : (a) token bearer hashé DC3, jamais en clair en DB ; (b) anti-énumération DC4 sur les 2 endpoints ; (c) rate-limit DC5 ; (d) email = secret SMTP password masqué dans logs (AC8) ; (e) le `reset_url` contient le token brut → l'email transite en clair si SMTP non-TLS : la doc recommande TLS (`KESH_SMTP_TLS=true`). (f) Pas de CSRF nouveau : endpoints publics non authentifiés, pas de cookie de session impliqué.
30. **i18n ownership** : `lint-i18n-ownership` PASS (clés `forgot-password-*`/`reset-password-*` feature-scoped).
31. **HTTP-LAN safe** : aucune API secure-context-only en runtime sur les pages publiques (souvent servies en HTTP LAN). `$props.id()` pour IDs DOM (cf. `feedback_no_secure_context_apis_http_lan`).
32. **Compat break-glass #121** : 17-4 ne modifie pas `auth/bootstrap.rs`. Test : break-glass `KESH_ADMIN_RESET` fonctionne toujours avec `KESH_FEATURE_FORGOT_PASSWORD=false`.

## Tasks / Subtasks

> Tâches groupées par Partie (A–F). Au split, chaque groupe devient une sous-story (17-4a…17-4f). Le **story-zéro** est la **Partie A** (migration + repos + `TABLES_TO_TRUNCATE`, prérequis de tout).

### Partie A — DB foundation (story-foundation)

- [ ] **T-A1** Migration `ADD COLUMN email` sur `users` (+ index `idx_users_email`), style `IF NOT EXISTS`. Étendre entité `User` (`pub email: Option<String>`) + `COLUMNS`/`FromRow` + tous les `SELECT` users impactés. (AC: 1)
- [ ] **T-A2** Migration `CREATE TABLE password_reset_tokens` (DC11 CASCADE). Entité `PasswordResetToken`. (AC: 2)
- [ ] **T-A3** Repo `find_by_email` (Vec) + nouveau repo `password_reset_tokens` (create/find_valid_by_hash/mark_used/invalidate_all_for_user), paramétrés. Tests unit repo. (AC: 3, 4)
- [ ] **T-A4** `TABLES_TO_TRUNCATE += "password_reset_tokens"` (`backup.rs:34`) + vérifier `backup_inventory_matches_schema` vert. (AC: 6)
- [ ] **T-A5** Plumbing `email` dans DTO `setup/admin` + `users` create/update + `UserResponse` + validation `is_valid_email_simple`. Tests. (AC: 5)
- [ ] **T-A6** 2 lignes `docs/migrations-idempotence-audit.md`. (AC: 7)

### Partie B — Couche email/SMTP + config

- [ ] **T-B1** Vars `KESH_SMTP_*` + `KESH_FEATURE_FORGOT_PASSWORD` + `KESH_PUBLIC_BASE_URL` dans `config.rs` (+ champ password masqué Debug). (AC: 8)
- [ ] **T-B2** Fail-fast boot (nouvelle `ConfigError`) si feature on + SMTP incomplet. Tests config. (AC: 9)
- [ ] **T-B3** Module `mail/` (DC1) : trait `Mailer` + `SmtpMailer` (lettre) + `MockMailer` (test). Câbler dans `AppState`. Dép `lettre` (DC2). (AC: 10)
- [ ] **T-B4** `AppError::SmtpSendFailed` + i18n `error-smtp-send-failed` ×4. (AC: 11)

### Partie C — Backend endpoints

- [ ] **T-C1** `POST /auth/forgot-password` (public, gated feature) : rate-limit, lookup DC6, génération token OsRng+SHA-256, envoi Mailer, `200` anti-énum, audit. (AC: 12)
- [ ] **T-C2** `POST /auth/reset-password` (public, gated) : validate token (hash+TTL+used), validate mdp, tx (mark_used + update_password + audit), revoke refresh post-commit. (AC: 13)
- [ ] **T-C3** Raison `"password_reset"` : migration contrainte CHECK refresh_tokens **OU** réutiliser `"password_change"` (trancher validate). (AC: 14)
- [ ] **T-C4** `forgotPasswordEnabled` dans `/health` (DC9). Montage conditionnel des routes selon feature flag (`lib.rs`). (AC: 15, 16)

### Partie D — Frontend

- [ ] **T-D1** Feature `lib/features/auth-recovery/` (api+types+tests). (AC: 19)
- [ ] **T-D2** Page `/forgot-password` (publique, runes Svelte 5). (AC: 17)
- [ ] **T-D3** Page `/reset-password` (publique, token query, HTTP-LAN safe). (AC: 18, 31)
- [ ] **T-D4** Lien conditionnel sur `/login` (DC9). (AC: 20)
- [ ] **T-D5** Champ `email` dans `SetupForm` + dialogues users CRUD. (AC: 21)
- [ ] **T-D6** i18n `forgot-password-*`/`reset-password-*`/`user-*` ×4 + `lint-i18n-ownership` PASS. (AC: 22, 30)

### Partie E — Tests

- [ ] **T-E1** Tests intégration Rust (happy/expiré/réutilisé/inexistant/no-email/rate-limit/SMTP-down) avec `MockMailer`. (AC: 23)
- [ ] **T-E2** E2E Playwright forgot + reset (token injecté). (AC: 24)

### Partie F — Doc

- [ ] **T-F1** Manuel admin SMTP + PDF. (AC: 25)
- [ ] **T-F2** Manuel user workflow recovery + PDF. (AC: 26)
- [ ] **T-F3** `.env.example` section SMTP. (AC: 27)
- [ ] **T-F4** CHANGELOG + README. (AC: 28)

## Dev Notes

### Architecture & ground-truth (cartographie 2026-06-10, 5 agents Explore)

**Auth & password** (`feedback`: ne PAS réinventer) :
- Hash : `crates/kesh-api/src/auth/password.rs` — `hash_password_async(String) -> Result<String, AppError>` (:49), `verify_password_async` (:56), `validate_password(pwd, min_length)` (:113). **Toujours la version async** dans les handlers.
- Users repo : `crates/kesh-db/src/repositories/users.rs` — `find_by_username` (:156), `find_by_id` (:128), `update_password(pool, user_id, hash)` (:270, incrémente `version`, **ne révoque pas** les tokens, c'est au caller). `find_by_email` **à créer**.
- Entité `User` `crates/kesh-db/src/entities/user.rs:108` : `{ id, username, password_hash, role, active, company_id, version, created_at, updated_at }` — **pas d'`email`**. `Role` enum (:18) `{ Admin, Comptable, Consultation }`.
- Refresh tokens : `crates/kesh-db/src/repositories/refresh_tokens.rs` — `revoke_all_for_user(pool, user_id, reason) -> u64` (:134). Raisons whitelistées par contrainte CHECK (migration `20260406000001`) : `logout`/`rotation`/`password_change`/`admin_disable`/`theft_detected`. → ajouter `password_reset` (AC14/T-C3) ou réutiliser `password_change`.
- Reset admin existant (référence) : `routes/users.rs:264` `reset_password` (admin-only, `require_admin_role`) — fait validate+hash+update_password+revoke_all(`"password_change"`). Le flux self-service réutilise les mêmes briques **sans** auth admin.
- Login : `routes/auth.rs:179` — émet JWT (`auth/jwt.rs:60` `encode`, HS256) + refresh token (uuid v4), `build_auth_cookies` (:43, HttpOnly SameSite=Strict). Rate-limit IP (:188).
- Routeur : `lib.rs:493-509` routes **publiques** (`/health`, `/api/v1/auth/login|logout|refresh`, `/setup/admin`). Protected = merge admin/comptable/authenticated sous `require_auth` (:484). **Ajouter `/api/v1/auth/forgot-password` + `/reset-password` dans le bloc public**, montage conditionnel selon feature flag.
- `RateLimiter` `middleware/rate_limit.rs:35` (custom, par IP, `check_rate_limit`/`record_failed_attempt`/`reset`). Config `rate_limit_max_attempts` (défaut 5), `rate_limit_window` (15min), `rate_limit_block_duration` (30min).

**Config & boot** :
- `config.rs` struct (:137-233) + `from_env` (:424-855). Patterns : opt-string trim+filter `config.rs:467` (KESH_ADMIN_USERNAME) ; strict-bool `:701` (KESH_TEST_MODE) ; int-borné parse+warn `:747` (KESH_ADMIN_EXPORT_INMEM_MB [1,2048]). `jwt_secret` champ **privé** (:156) → calque pour masquer `smtp_password`.
- Fail-fast : `main.rs:62-68` `Config::from_env()` err → `exit(1)`. 6 autres points (DB, downgrade, migrations, bootstrap, i18n, bind). Ajouter la validation SMTP dans `from_env`.
- Break-glass #121 : `auth/bootstrap.rs:219-302` case 5 (matrice 6 cas docstring :1-20). **NE PAS toucher.** Audit action `"admin_break_glass_reset"` (:254).
- `.env.example` : sections A-L (:1-203). Pas de section SMTP. Admin recovery doc dans docstring `KESH_ADMIN_PASSWORD` (:58-82).
- `Cargo.toml` kesh-api : `tokio` `["full"]`, `uuid 1 ["v4","serde"]`, `chrono 0.4`, `argon2` (réexporte `OsRng`), `sha2`. **Pas** de `lettre`/email. `reqwest` en dev-dep seulement.

**Migrations & audit** :
- Dir `crates/kesh-db/migrations/` (31 migrations, `YYYYMMDDHHMMSS_name.sql`). Template ADD COLUMN : `20260419000001_invoice_paid_at.sql:19` (`IF NOT EXISTS`). Template CREATE TABLE : `20260605000001_api_keys.sql:18` (FK/INDEX/CHECK, `token_hash CHAR(64)`, `uq_…_key_hash UNIQUE`). users : `20260404000001_initial_schema.sql:24`.
- `_kesh_version` : `version.rs:222` `check_downgrade_protection` (avant migrate), `:310` `record_boot_version`. ADD COLUMN nullable + CREATE TABLE = non-breaking (DC12).
- Audit : entité `entities/audit_log.rs:95` (`AuditLogEntry { user_id FK RESTRICT, action, entity_type, entity_id, details_json, actor_type, actor_api_key_id }`), `ActorType` enum (:25). Constructeurs `NewAuditLogEntry::user(...)` (:136), `::from_current_user` (`audit.rs:18`). Insert : `repositories/audit_log.rs:29` `insert_in_tx(tx, new)`. Actions = **strings libres** → `auth.password_reset_requested` / `auth.password_reset_completed`. **FK `audit_log.user_id → users(id)` NOT NULL** : pour `forgot-password` non-matché, **ne pas** créer d'entrée (pas de user_id valide).
- `TABLES_TO_TRUNCATE` `backup.rs:34` (22 tables, enfants→parents). **Ajouter `password_reset_tokens`** (AC6). Test `backup_inventory_matches_schema` `backup.rs:567` auto-fail sinon.

**Email infra (greenfield)** :
- 0 code email. `uuid` server-side OK (`auth.rs`, `setup.rs` `Uuid::new_v4().to_string()`). `chrono` `Utc::now() + TimeDelta::minutes(30)` (pattern `jwt.rs:73`, `main.rs:167`). `AppError` `errors.rs` (enum + `IntoResponse` + `t()`/`t_args()` i18n, code SNAKE_CASE + clé `error-kebab`). Crates workspace : 10 (kesh-core/db/api/i18n/report/reconciliation/seed/import/payment/qrbill). `kesh-payment` = placeholder vide (anti-pattern à ne pas imiter pour le mail — préférer module `kesh-api/src/mail/`, DC1).

**Frontend** :
- Public routes : hors `(app)`. `/login` (`routes/login/+page.svelte`, POST `/api/v1/auth/login`, pas de guard), `/setup` (`routes/setup/+layout.ts:21` public). Guard auth = `routes/(app)/+layout.ts:9`. → `/forgot-password` + `/reset-password` au **même niveau que login** (pas de guard).
- `SetupForm` `lib/features/setup/SetupForm.svelte` (champs username/password/confirm, insérer email). `setup.api.ts:32` `setupAdmin(username, password)` → +email.
- Users admin `routes/(app)/users/+page.svelte` (guard Admin `+page.ts:7`), dialogues create (:351)/edit (:398)/reset-password (:455). Ajouter email create/edit.
- i18n FTL `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`, conso `i18nMsg(key, fallback)` (`lib/shared/utils/i18n.svelte.ts:14`). Lint ownership `scripts/lint-i18n-ownership.js:79` (feature-scoped sauf namespaces globaux error/tooltip/common/mode/shortcut/demo).
- HTTP-LAN : `$props.id()` (pas `crypto.randomUUID`, cf. `ContactPicker.svelte:25`), `copyToClipboard` fallback `execCommand` (`lib/shared/utils/clipboard.ts:17`). Bugs #143/#145.
- Feature api pattern : `lib/features/contacts/contacts.api.ts:1` (apiClient.post/put), `setup.api.ts:1` (public).

### Project Structure Notes

- Nouveaux fichiers backend : `crates/kesh-db/migrations/{ts}_users_email.sql`, `{ts}_password_reset_tokens.sql`, `crates/kesh-db/src/entities/password_reset_token.rs`, `crates/kesh-db/src/repositories/password_reset_tokens.rs`, `crates/kesh-api/src/mail/{mod,smtp,template}.rs`, handlers dans `routes/auth.rs` (étendu).
- Nouveaux fichiers frontend : `routes/forgot-password/+page.svelte`, `routes/reset-password/+page.svelte`, `lib/features/auth-recovery/{auth-recovery.api,auth-recovery.types,*.test}.ts`.
- Modifs : `config.rs`, `lib.rs` (routes), `errors.rs`, `entities/user.rs` + repos users, `backup.rs` (TABLES_TO_TRUNCATE), `routes/{setup,users,health}.rs`, `SetupForm.svelte`, `users/+page.svelte`, `login/+page.svelte`, FTL ×4, `.env.example`, manuels LaTeX, CHANGELOG, README, idempotence-audit.
- **Aligné** sur les conventions existantes (token hashé PAT 17-2a, RateLimiter login, audit string-action, public-route login, HTTP-LAN). Aucune divergence structurelle.

### References

- [Source: GitHub Issue #122 — Recovery mot de passe production-grade]
- [Source: GitHub Issue #121 — Break-glass KESH_ADMIN_RESET (fallback conservé)]
- [Source: crates/kesh-api/src/auth/password.rs:49,56,113 — hash/verify/validate]
- [Source: crates/kesh-db/src/repositories/users.rs:156,270 — find_by_username, update_password]
- [Source: crates/kesh-db/src/repositories/refresh_tokens.rs:134 — revoke_all_for_user]
- [Source: crates/kesh-api/src/config.rs:467,701,747 — patterns env vars]
- [Source: crates/kesh-api/src/auth/bootstrap.rs:219-302 — break-glass #121]
- [Source: crates/kesh-db/migrations/20260605000001_api_keys.sql:18 — template CREATE TABLE + token_hash CHAR(64)]
- [Source: crates/kesh-db/src/backup.rs:34,567 — TABLES_TO_TRUNCATE + test schéma]
- [Source: crates/kesh-db/src/entities/audit_log.rs:95 ; crates/kesh-api/src/audit.rs:18 — audit]
- [Source: crates/kesh-api/src/lib.rs:484,493 — routeur public/protégé]
- [Source: frontend/src/routes/login/+page.svelte ; routes/setup/+layout.ts:21 — public routes]
- [Source: crates/kesh-i18n/locales/*/messages.ftl ; frontend/scripts/lint-i18n-ownership.js:79 — i18n]
- [Source: CLAUDE.md §Règle de splitting préventif, §Migration breaking policy, §Issue Tracking]
- [Source: memory feedback_no_secure_context_apis_http_lan — bugs #143/#145]
- [Source: memory project_17_2a_review_done — pattern PAT token SHA-256 (DC3)]

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story — Opus 4.8 recommandé : flux cross-crate config↔mail↔routes + sécurité anti-énum non-mécaniques)

### Debug Log References

### Completion Notes List

- Spec umbrella créée 2026-06-10 (Opus 4.8, 5 agents Explore : auth/password, config/boot+break-glass, migrations/audit, frontend public/i18n, email greenfield). Scope **cœur recovery seul** (décision Guy). Structure umbrella→split (pattern 17-2/17-3). Prochaine : `bmad-create-story validate 17-4` Pass 1 Sonnet 4.6 (split A–F décidé/affiné au validate ; trancher DC1/DC6/DC9/AC14/AC23-g).

### File List
