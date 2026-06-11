# Story 17.4b: Couche email/SMTP + config (recovery)

Status: review

<!-- Extraite de la spec parente UMBRELLA 17-4 (`17-4-recovery-mot-de-passe.md`), validate CONVERGÉ 6 passes (trend > LOW 12→1→3→2→1→0). Contenu déjà adversarialement revu (Partie B : AC8-11, T-B1..T-B4, DC1/DC2/DC7/DC10). Re-validate optionnel. -->
<!-- DÉPEND de 17-4a (DONE) : consomme le refactor `AppState::new_for_tests` (stratégie anti-churn P4-1). SÉRIE, PAS parallélisable (P3 F2). BLOQUE 17-4c (endpoints). -->

## Story

As a **développeur posant la couche d'envoi d'email transactionnel de Kesh**,
I want **la config SMTP (env vars + fail-fast boot), un module `mail/` avec un trait `Mailer` injectable (`SmtpMailer` prod / `NoopMailer`+`MockMailer` test), le champ `mailer` sur `AppState`, et le variant d'erreur + les clés i18n de l'email**,
so that **17-4c puisse générer et envoyer le magic-link de reset via une abstraction testable (sans SMTP réel en CI), et que l'instance refuse de démarrer (fail-fast) si le recovery est activé sans config SMTP complète**.

## Contexte & cadrage

**Issue source :** [#122](https://github.com/guycorbaz/kesh/issues/122) (recovery production-grade, `v0.2-milestone`). Épopée 17-4, scope **cœur recovery seul** (décision Guy 2026-06-10). Spec complète + DC + sécurité : voir umbrella `17-4-recovery-mot-de-passe.md`.

**Position dans le split A–F (ordre SÉRIE) :** 17-4a (DONE) → **17-4b (ici)** → 17-4c (endpoints publics) → 17-4d (frontend) → 17-4e (tests) → 17-4f (doc). 17-4b **dépend de 17-4a** (refactor `new_for_tests`) et **n'est pas parallélisable** (P3 F2 : 18 fichiers de test partageraient `AppState`). **BLOQUE 17-4c** (qui consomme `Mailer` + la config SMTP).

**Greenfield (validate P1) :** aucune infrastructure email n'existe (`grep smtp/lettre/Message::builder` = 0 hit). Toute la couche d'envoi est à créer. Aucun moteur de templating (askama/tera absents) → emails formatés **inline** (`format!`) + i18n Fluent (DC10).

**Scope 17-4b (cette story) :**
- **T-B1** — Env vars `KESH_SMTP_*` (host/port/user/password/from/tls) + `KESH_FEATURE_FORGOT_PASSWORD` + `KESH_PUBLIC_BASE_URL` dans `config.rs`, avec `smtp_password` masqué dans `Debug`.
- **T-B2** — Fail-fast au boot : nouvelle `ConfigError` si feature on + config SMTP incomplète. Tests config.
- **T-B3** — Module `crates/kesh-api/src/mail/` : trait `Mailer` + `SmtpMailer` (lettre rustls) + `NoopMailer`/`MockMailer`. Champ `mailer: Arc<dyn Mailer>` sur `AppState`, **défauté dans le corps de `new_for_tests`** (signature inchangée) + ajouté manuellement aux **3 sites littéraux** (`main.rs`, `middleware/auth.rs:267`, `tests/setup_admin_e2e.rs:81`). Dép Cargo `lettre`.
- **T-B4** — `AppError::SmtpSendFailed` + clé i18n `error-smtp-send-failed` ×4 + **clés email** `email-password-reset-subject`/`email-password-reset-body` ×4 (rendu backend, P5).

**Hors scope 17-4b** (sous-stories suivantes) : endpoints `forgot/reset-password` + génération/validation token + rate-limit + `tokio::spawn` détaché + montage conditionnel des routes (17-4c) ; exposition `forgotPasswordEnabled` dans `/health` (DC9 — **T-C4**, Partie C) ; pages frontend (17-4d) ; tests E2E forgot/reset (17-4e) ; doc SMTP (17-4f). Le `Mailer` est **créé ici** mais **appelé** par 17-4c.

## Décisions de conception applicables (figées à l'umbrella, 6 passes)

- **DC1 — module `crates/kesh-api/src/mail/`** (PAS de crate `kesh-mail` : sur-ingénierie, cf. `kesh-payment` placeholder vide). Couplage fort config + i18n + `AppError` (tous dans kesh-api).
- **DC2 — lib `lettre` 0.11**, features `["tokio1-rustls-tls", "smtp-transport", "builder"]` — **rustls, PAS native-tls** (cohérence `sqlx` `runtime-tokio-rustls` `Cargo.toml:26` ; native-tls dupliquerait la stack TLS + dép OpenSSL système inutile). Async via runtime tokio `["full"]` déjà présent. *(Note dev : `lettre` rustls embarque ses roots webpki ; vérifier qu'aucun `rustls-native-certs` séparé n'est requis pour les SMTP à CA publique. La valeur exacte STARTTLS-vs-implicit-TLS de `KESH_SMTP_TLS` est à confirmer côté API `lettre` au dev.)*
- **DC7 — feature flag `KESH_FEATURE_FORGOT_PASSWORD` (défaut `false`)**. Si `false` : config SMTP optionnelle (pas de validation). Si `true` : config SMTP **complète requise au boot** (fail-fast). Parsing strict bool (`"true"`/`"1"`/empty, calque `KESH_TEST_MODE` `config.rs` strict-bool).
- **DC10 — email localisé inline** (`format!`) + i18n bundle Fluent global `config.locale` (`KESH_LANG`). **Pas de locale par-utilisateur** (colonne inexistante) → email dans la langue de l'instance.
- **DC12 / P4-1 — anti-churn `AppState`** : le champ `mailer` est défauté `Arc::new(NoopMailer)` **dans le corps de `new_for_tests`** (signature inchangée) → les ~33 call-sites de test migrés en 17-4a restent **intacts**. Seuls `main.rs`, `auth.rs:267`, `setup_admin_e2e.rs:81` (les 3 littéraux préservés en 17-4a) reçoivent le champ manuellement.

## Acceptance Criteria

> Numérotation continue de l'umbrella (AC8-11, Partie B). DC9/AC15-16 (`/health` + montage routes) sont **hors scope** (17-4c).

8. **Env vars SMTP** (`config.rs`, calque patterns `config.rs` opt-string trim+filter / strict-bool / int-borné) :
   - `KESH_SMTP_HOST` (opt string, trim+filter-empty),
   - `KESH_SMTP_PORT` (opt int **bornes [1, 65535]**, défaut **587**, parse+borne+warn),
   - `KESH_SMTP_USER` (opt string),
   - `KESH_SMTP_PASSWORD` (opt string — **masqué dans `Debug`** : champ privé + custom `Debug` `***`, calque `jwt_secret` `config.rs:156`/`:248`),
   - `KESH_SMTP_FROM` (opt string, validé email via `is_valid_email_simple` — `pub(crate)` depuis 17-4a, `routes/contacts.rs`),
   - `KESH_SMTP_TLS` (strict bool, défaut `true` — STARTTLS ; valeur exacte à confirmer côté `lettre`),
   - `KESH_PUBLIC_BASE_URL` (opt string, trim+filter-empty — base de l'URL du lien de reset, consommée par 17-4c),
   - `KESH_FEATURE_FORGOT_PASSWORD` (strict bool, défaut `false`, DC7).
   ⚠️ Ajouter aussi ces champs à `Config::from_fields_for_test` (`config.rs:305`) avec des **défauts** (SMTP `None`, feature `false`) **sans changer la signature publique** si possible (champs internes défautés dans le corps), pour ne pas churner `auth.rs:267` au-delà du champ `mailer`. *(Si la signature de `from_fields_for_test` doit changer, le documenter — c'est un builder test à 1-2 call-sites.)*

9. **Fail-fast boot (DC7)** : si `KESH_FEATURE_FORGOT_PASSWORD=true` mais qu'**une** des vars requises (`SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM`, `PUBLIC_BASE_URL`) est absente/vide, **ou** `SMTP_FROM` n'est pas un email valide → nouvelle variante `ConfigError` (`config.rs:14`) + `Config::from_env()` retourne `Err` → `main.rs:62` `std::process::exit(1)` avec message clair (calque les 7 fail-fast existants). Si `false` : SMTP optionnel, **aucune** validation. Tests config couvrant : feature off + SMTP vide → OK ; feature on + complet → OK ; feature on + 1 var manquante → Err ; feature on + SMTP_FROM invalide → Err.

10. **Module `mail/`** (DC1) :
    - `Mailer` **trait** `Send + Sync` : `async fn send_password_reset(&self, to: &str, reset_url: &str, locale: &str) -> Result<(), AppError>` (signature à figer au dev ; `locale` = `config.locale` pour le rendu Fluent, DC10).
    - `SmtpMailer` (impl lettre rustls DC2, lit la config SMTP — host/port/user/password/from/tls).
    - `NoopMailer` (no-op silencieux, défaut test/feature-off) + `MockMailer` (capture le mail en mémoire sans I/O réseau, pour les tests 17-4e — ex. `Arc<Mutex<Vec<...>>>`).
    - Champ `mailer: Arc<dyn Mailer>` ajouté à `AppState` (`lib.rs:42`), **défauté `Arc::new(NoopMailer)` dans le CORPS de `new_for_tests` (`lib.rs:60`, signature inchangée, P4-1)** + ajouté à `main.rs` (vrai `SmtpMailer` si feature on + config OK, sinon `NoopMailer`) + aux 2 littéraux-exception `middleware/auth.rs:267` et `tests/setup_admin_e2e.rs:81` (`Arc::new(NoopMailer)`). Champ `pub` pour que 17-4e mute `state.mailer = Arc::new(MockMailer)` post-construction.
    - **Clés Fluent de l'email** (rendu backend, distinctes des clés de page frontend 17-4d) : `email-password-reset-subject` + `email-password-reset-body` (placeholders `{ $resetUrl }` + `{ $ttlMinutes }`) **×4 locales** `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`, rendues via `config.locale` (DC10). Responsabilité **T-B3/T-B4** (P5), PAS 17-4d.

11. **`AppError`** (`errors.rs`) : variant `SmtpSendFailed(String)` mappé `500 SMTP_SEND_FAILED`, log `tracing::error!`, clé i18n `error-smtp-send-failed` ×4 locales. Pattern existant (enum + `IntoResponse` + clé `error-*`). *(Note : ce variant sera utilisé par 17-4c côté `SmtpMailer::send`, mais l'envoi y est fire-and-forget `tokio::spawn` détaché — l'erreur n'est que loggée, jamais propagée au client, DC4. Le variant existe ici pour que `Mailer::send_password_reset` ait un type de retour propre.)*

### Transverses

12. **Sécurité** : (a) `smtp_password` = secret → **masqué dans `Debug` de `Config`** (AC8, calque `jwt_secret`) ; ne jamais logger en clair. (b) `KESH_SMTP_TLS=true` par défaut (la doc 17-4f recommandera TLS car le `reset_url` contient le token brut → email en clair si SMTP non chiffré).

13. **Build/tests verts standalone** : 17-4b est mergeable indépendamment — le module `mail/` + la config SMTP ne référencent ni routes recovery, ni frontend. `NoopMailer` par défaut → aucun envoi réel. `cargo build --workspace --all-targets` + `cargo clippy -- -D warnings` + `cargo test --workspace` verts (Test Locally First). Comme la story ne touche **pas** `kesh-db`, le mode test parallèle suffit ; lancer le mode serial seulement si des tests d'intégration DB sont ajoutés.

## Tasks / Subtasks

- [x] **T-B1** Vars `KESH_SMTP_HOST/PORT/USER/PASSWORD/FROM/TLS` + `KESH_PUBLIC_BASE_URL` + `KESH_FEATURE_FORGOT_PASSWORD` dans `config.rs` (struct `:137` + parse `from_env` selon patterns existants). `smtp_password` champ privé + `Debug` custom `***` (`:248`). Défauter les nouveaux champs dans `from_fields_for_test` (`:305`) sans churn de signature si possible. (AC: 8, 12a)
- [x] **T-B2** Fail-fast boot : nouvelle `ConfigError` variant (`:14`) si `KESH_FEATURE_FORGOT_PASSWORD=true` + SMTP/PUBLIC_BASE_URL incomplet ou `SMTP_FROM` invalide ; branchée dans `from_env` (`main.rs:62` exit(1)). Tests config (4 cas : off/ok, on/ok, on/var-manquante, on/from-invalide). (AC: 9)
- [x] **T-B3** Module `crates/kesh-api/src/mail/{mod,smtp}.rs` (DC1) : trait `Mailer` (`Send+Sync`) + `SmtpMailer` (lettre rustls DC2, lit config SMTP) + `NoopMailer` + `MockMailer` (capture, test). Champ `mailer: Arc<dyn Mailer>` sur `AppState` (`lib.rs:42`), **défaut `Arc::new(NoopMailer)` dans le corps de `new_for_tests` (`lib.rs:60`)** ; ajout manuel à `main.rs:224` (selon feature) + `middleware/auth.rs:267` + `tests/setup_admin_e2e.rs:81`. Dép `lettre` 0.11 rustls dans `crates/kesh-api/Cargo.toml`. (AC: 10)
- [x] **T-B4** `AppError::SmtpSendFailed(String)` (`errors.rs`, `500 SMTP_SEND_FAILED` + `tracing::error!`) + clé `error-smtp-send-failed` ×4 + clés `email-password-reset-subject`/`email-password-reset-body` (placeholders `{ $resetUrl }`/`{ $ttlMinutes }`) ×4 dans les FTL. (AC: 10, 11)
- [x] **T-B5** Quality gate Test Locally First backend (fmt/build/clippy -D/test workspace). Parallèle suffit (pas de kesh-db touché) ; serial si ajout de tests intégration DB. (AC: 13)

## Dev Notes

### Ground-truth (état réel post-17-4a, vérifié 2026-06-10)

**Cargo & deps (`crates/kesh-api/Cargo.toml`) :**
- `tokio = { version = "1", features = ["full"] }` (:22) ; `sqlx … "runtime-tokio-rustls"` (:26) → **utiliser rustls pour `lettre`** (cohérence stack TLS, DC2). `uuid`/`chrono`/`sha2`/`argon2` présents. **`lettre` ABSENT** → à ajouter (`lettre = { version = "0.11", default-features = false, features = ["tokio1-rustls-tls", "smtp-transport", "builder"] }` — figer au dev, `default-features = false` pour exclure native-tls).

**Config & boot (`crates/kesh-api/src/config.rs`) :**
- `enum ConfigError` (`:14`) — ajouter le variant SMTP fail-fast ici. `pub struct Config` (`:137`). `jwt_secret: String` **privé** (`:156`) ; `impl Debug for Config` (`:236`) masque via `.field("jwt_secret", &"***")` (`:248`) → **même pattern pour `smtp_password`**. `rate_limit_max_attempts` etc. exemples de champs typés.
- `from_fields_for_test` (`:305`) — builder test (assert jwt_secret ≥32, rate_limit ∈ [1,100]…). Ajouter les champs SMTP défautés ; minimiser le churn (idéalement défauts dans le corps, signature stable).
- `from_env` (`:424+`) + 7 points fail-fast existants (`main.rs:62-68` `Config::from_env()` err → `exit(1)`). Patterns env : opt-string trim+filter, strict-bool (`KESH_TEST_MODE`), int-borné parse+warn. La validation SMTP s'ajoute dans `from_env`.
- Break-glass #121 (`auth/bootstrap.rs:219-302`) = fallback offline quand feature off — **NE PAS toucher** (vars d'env disjointes, coexistence sans interaction).

**AppState & anti-churn (`crates/kesh-api/src/lib.rs`) :**
- `AppState` (`:42`) actuel = `{ pool, config, rate_limiter, i18n, users_exist }` (5 champs). `new_for_tests(pool, config, rate_limiter, i18n)` (`:60`) défaute `users_exist=true` dans le corps. **17-4a a migré ~33 littéraux de test vers `new_for_tests`** → ajouter `mailer` en le défautant **dans le corps** de `new_for_tests` laisse ces 33 sites intacts (P4-1).
- 3 sites littéraux restants (à patcher manuellement) : `main.rs:224` (prod — vrai mailer selon feature), `middleware/auth.rs:267` (`test_state`, littéral avec `Config::from_fields_for_test` + tous les champs explicites), `tests/setup_admin_e2e.rs:81` (`users_exist` variable). Les 3 reçoivent `mailer: Arc::new(NoopMailer)` (ou vrai mailer pour main.rs).

**Email infra (greenfield) :**
- 0 code email. `AppError` (`errors.rs`) = enum + `IntoResponse` + i18n `t()`/`t_args()`, code SNAKE_CASE + clé `error-kebab`. `kesh-payment` = crate placeholder vide → **anti-pattern à ne pas imiter** (préférer module `mail/` in-kesh-api, DC1). `is_valid_email_simple` est `pub(crate)` depuis 17-4a (`routes/contacts.rs`) → réutilisable pour valider `SMTP_FROM`.
- i18n FTL `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` ; 17-4a y a ajouté `error-email-invalid` (modèle d'ajout multi-locale).

### Project Structure Notes

- **Nouveaux fichiers** : `crates/kesh-api/src/mail/mod.rs` (trait `Mailer` + `NoopMailer`/`MockMailer`), `crates/kesh-api/src/mail/smtp.rs` (`SmtpMailer` lettre). Déclarer `pub mod mail;` dans `lib.rs`.
- **Modifs** : `crates/kesh-api/Cargo.toml` (+lettre), `config.rs` (vars SMTP + ConfigError + Debug + from_fields_for_test), `lib.rs` (AppState.mailer + new_for_tests), `main.rs` (mailer selon feature), `middleware/auth.rs` (littéral test_state), `tests/setup_admin_e2e.rs` (littéral), `errors.rs` (SmtpSendFailed), FTL ×4 (error-smtp-send-failed + email-password-reset-subject/body).
- Aucune divergence structurelle — aligné conventions (token PAT 17-2a, RateLimiter login, public-route login, i18n multi-locale 17-4a).

### References

- [Source: umbrella `_bmad-output/implementation-artifacts/17-4-recovery-mot-de-passe.md` — Partie B AC8-11, T-B1..T-B4, DC1/DC2/DC7/DC10/DC12, convergée 6 passes]
- [Source: GitHub Issue #122 — recovery production-grade ; #121 break-glass conservé]
- [Source: crates/kesh-api/Cargo.toml:22,26 — tokio full + sqlx runtime-tokio-rustls (cohérence rustls DC2)]
- [Source: crates/kesh-api/src/config.rs:14,137,156,236,248,305 — ConfigError, Config struct, jwt_secret privé+Debug masqué, from_fields_for_test]
- [Source: crates/kesh-api/src/lib.rs:42,60 — AppState + new_for_tests (anti-churn P4-1)]
- [Source: crates/kesh-api/src/main.rs:224 ; src/middleware/auth.rs:267 ; tests/setup_admin_e2e.rs:81 — 3 littéraux AppState à patcher]
- [Source: crates/kesh-api/src/errors.rs — pattern AppError enum + IntoResponse + clé Fluent error-*]
- [Source: story 17-4a (DONE) — refactor new_for_tests, is_valid_email_simple pub(crate), modèle ajout i18n multi-locale]
- [Source: CLAUDE.md §Test Locally First]

## Dev Agent Record

### Agent Model Used

Opus 4.8 (run dev-story interrompu — session perdue avant finalisation du story file) ; reprise et finalisation Fable 5 (2026-06-10) : audit d'intégrité du code en working tree vs T-B1..T-B4, quality gate complet, complétion du Dev Agent Record.

### Debug Log References

- Run dev-story initial interrompu après écriture du code (T-B1..T-B4 complets en working tree, story file non mis à jour, rien de commité). Reprise : audit diff (541 insertions / 13 fichiers) — couverture intégrale des 4 tâches confirmée, puis quality gate.

### Completion Notes List

- Story extraite de l'umbrella 17-4 convergée 6 passes (2026-06-10). Partie B. Contenu déjà adversarialement revu (re-validate optionnel). Dépend de 17-4a (DONE).
- **T-B1** : 8 champs config (`smtp_host/port/user/password/from/tls`, `public_base_url`, `forgot_password_enabled`) + helpers partagés `opt_trimmed_env` / `parse_strict_bool` (nouveau variant `ConfigError::InvalidBoolValue`, pattern strict `KESH_COOKIE_SECURE`). `smtp_password` privé, masqué `***` dans `Debug`, accès via `Config::smtp_password()`. Défauts dans le corps de `from_fields_for_test` + `test_helpers` → zéro churn de signature (P4-1).
- **T-B2** : `ConfigError::IncompleteSmtpConfig { detail }` fail-fast dans `from_env` si feature on + var manquante OU `SMTP_FROM` invalide (`is_valid_email_simple` réutilisé de 17-4a). Module `smtp_config_tests` : 7 tests (off/ok, on/ok, on/var-manquante, on/from-invalide + port défaut/borné + strict-bool refusé).
- **T-B3** : module `mail/` in-kesh-api (DC1) — trait `Mailer` objet-safe **sans** `async_trait` (futures boxées `MailFuture`), `SmtpMailer` (lettre 0.11 rustls DC2, STARTTLS défaut / `builder_dangerous` si `KESH_SMTP_TLS=false`), `NoopMailer`, `MockMailer` (capture + variante `failing()` pour AC23-g SMTP-down). `AppState.mailer: Arc<dyn Mailer>` défauté `NoopMailer` dans le corps de `new_for_tests` (P4-1, 33 call-sites 17-4a intacts) + 3 littéraux patchés (main.rs prod-selon-feature, auth.rs test_state, setup_admin_e2e.rs). Const partagée `PASSWORD_RESET_TTL_MINUTES = 30` (DC8) consommée par 17-4c.
- **T-B4** : `AppError::SmtpSendFailed(String)` → 500 `SMTP_SEND_FAILED`, détail loggé `tracing::error!` jamais exposé ; clés `error-smtp-send-failed` + `email-password-reset-subject`/`email-password-reset-body` (placeholders `{ $resetUrl }`/`{ $ttlMinutes }`) dans les 4 FTL.
- **T-B5** : quality gate Test Locally First backend 4/4 vert — `cargo fmt --check` OK, `build --workspace --all-targets` OK, `clippy -D warnings` OK, `cargo test --workspace` OK (cf. Change Log).

### File List

- `crates/kesh-api/Cargo.toml` — dép `lettre` 0.11 (`default-features = false`, rustls)
- `Cargo.lock` — lockfile lettre + transitives
- `crates/kesh-api/src/config.rs` — 8 champs SMTP/recovery + 2 variants ConfigError + Debug masqué + fail-fast from_env + helpers + 7 tests
- `crates/kesh-api/src/mail/mod.rs` — **nouveau** : trait Mailer + NoopMailer + MockMailer + 3 tests unitaires
- `crates/kesh-api/src/mail/smtp.rs` — **nouveau** : SmtpMailer (lettre rustls, rendu Fluent DC10)
- `crates/kesh-api/src/lib.rs` — `pub mod mail` + champ `AppState.mailer` (défaut NoopMailer dans new_for_tests)
- `crates/kesh-api/src/main.rs` — construction mailer selon feature (SmtpMailer/NoopMailer) + logs boot
- `crates/kesh-api/src/errors.rs` — variant `SmtpSendFailed` + IntoResponse
- `crates/kesh-api/src/middleware/auth.rs` — littéral test_state + mailer NoopMailer
- `crates/kesh-api/tests/setup_admin_e2e.rs` — littéral spawn_app + mailer NoopMailer
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — 3 clés ×4 locales
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — 17-4b in-progress → review

## Change Log

- 2026-06-10 — dev-story 17-4b (Opus 4.8, interrompu ; reprise+finalisation Fable 5) : T-B1..T-B5 complets, 852 insertions / 17 fichiers au commit `b38a30c` (code + 7 tests config + story file + sprint-status + Cargo.lock + 4 FTL). Signature `Mailer::send_password_reset` figée à `locale: Locale` (enum) plutôt que `&str` (spec : « signature à figer au dev ») — cohérent avec `config.locale` et `I18nBundle::format`, évite un parse runtime. Quality gate backend 4/4 vert : fmt + build + clippy -D OK ; `cargo test --workspace` parallèle = 1019 verts hors kesh-db (21 échecs `kesh-db --lib` = contention parallélisme DB connue, `OptimisticLockConflict` sur tests repository sqlx — 17-4b ne touche pas kesh-db) ; re-run `cargo test -p kesh-db --lib -- --test-threads=1` (mode CI serial) = **187/187 verts, 0 régression**. Status `ready-for-dev → review`. Prochaine : `bmad-code-review 17-4b`.

### Pass 1 review (Sonnet 4.6) — 2026-06-11

3 reviewers parallèles (Blind Hunter / Edge Case Hunter / Acceptance Auditor), 13 findings bruts → 10 dédupliqués : **7 > LOW** (3 HIGH, 4 MEDIUM après merge) → patches appliqués, Pass 2 obligatoire (rotation → Haiku).

**Patches appliqués :**
- **BH-1 HIGH** — transport SMTP reconstruit à chaque envoi → `AsyncSmtpTransport` construit une seule fois dans `SmtpMailer::from_config` (résolution relay + TLS params + credentials au boot). Bénéfice secondaire : le password n'est plus stocké dans la struct (neutralise BH-2).
- **BH-6 MEDIUM** — `from_config` passe de `Option<Self>` à `Result<Self, String>` : l'échec d'init STARTTLS (légitimement possible) est distingué des vars absentes (inatteignables post-fail-fast) ; `main.rs` loggue le détail puis `exit(1)` (pattern boot existant).
- **E-1/A-1 MEDIUM** (edge+auditor) — `public_base_url` : `trim_end_matches('/')` + re-filter vide (exigence umbrella P4-4 omise par la spec-fille) + 2 tests (trailing slash, "///" → None).
- **E-2 MEDIUM** — garde boot `KESH_SMTP_HOST` contenant `:` (format `host:port` copié d'une doc → erreur SNI cryptique au 1er envoi sinon), exception IPv6 literal + 2 tests.
- **BH-4/E-4 MEDIUM** — `parse_strict_bool` trim avant comparaison (cohérence `opt_trimmed_env`) + test `" false "`.
- **BH-7 MEDIUM** — corps email multiline Fluent ×4 locales : URL isolée sur sa propre ligne (clickability clients mail), TTL remonté dans la phrase d'intro.
- **E-3 LOW** — message `IncompleteSmtpConfig` from-invalide précise « sans display-name ».
- **BH-8 LOW** — test Debug asserte la présence du masque `***` (pas seulement l'absence du secret).
- **BH-10 LOW** — `opt_trimmed_env` distingue `NotUnicode` (warn) de `NotPresent` (message fail-fast sinon trompeur).
- **A-2/A-3 LOW** — Change Log corrigé (852/17, note signature `Locale`).

**Dismissed :** BH-2 HIGH (claim « fuite Debug » — `SmtpMailer` n'a aucun impl `Debug`, pas de chemin de fuite ; rendu sans objet par BH-1), BH-5 MEDIUM (détail erreur lettre loggé serveur-only = pattern projet, pas de secret dans les erreurs lettre), BH-9 LOW (clone Vec test-only), exit(1) boot (pattern existant).
**Deferred :** BH-3 HIGH→reclassé design 17-4e (MockMailer capture-sur-échec : le test SMTP-down AC23-g vérifie le 200-toujours, pas la capture ; la story 17-4e adaptera si besoin — owner documenté ici).

Quality gate post-patch : fmt + clippy -D (kesh-api, kesh-i18n) verts ; tests kesh-i18n 21/21, config 69/69 (dont 6 nouveaux), mail 5/5.
