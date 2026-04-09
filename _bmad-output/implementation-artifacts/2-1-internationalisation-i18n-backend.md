# Story 2.1: Internationalisation (i18n) backend

Status: done

## Story

As a **utilisateur**,
I want **utiliser Kesh dans ma langue (FR/DE/IT/EN) avec formatage suisse**,
so that **l'interface et les données soient présentées dans ma langue avec les conventions régionales suisses**.

### Contexte

C'est la première story de l'Epic 2 (Onboarding & Configuration). Elle pose les fondations i18n que toutes les stories suivantes utiliseront. Le scope est **backend uniquement** : crate `kesh-i18n` + intégration dans `kesh-api`. Le frontend i18n (extraction des strings hardcodées) sera traité progressivement dans les stories 2.2→2.5.

### Décisions de conception

- **Fluent (.ftl)** comme format de traduction — choix architecture (ADR #13). Mozilla Fluent gère naturellement la pluralisation, le genre et les arguments nommés.
- **Langue par instance** via `KESH_LANG` (env) — FR par défaut. L'API expose les traductions dans cette langue. La langue par utilisateur (profil) sera ajoutée ultérieurement (Story 2.4+).
- **Formatage suisse** : apostrophe typographique `'` (U+2019) comme séparateur de milliers (`1'234.56`), dates `dd.mm.yyyy`. Implémenté dans `kesh-i18n`, pas dans Fluent (Fluent n'est pas conçu pour le formatage numérique).
- **Messages d'erreur API** : restent en codes SCREAMING_SNAKE_CASE (`INVALID_CREDENTIALS`, etc.). Le champ `message` passe de français hardcodé à traduction Fluent via la langue instance. Les codes sont la source de vérité côté client.
- **Frontend** : cette story n'extrait PAS les strings frontend. Elle expose un endpoint `/api/v1/i18n/messages` qui retourne les traductions pour la langue courante, prêt pour le frontend dans les stories suivantes.

## Acceptance Criteria (AC)

1. **Chargement Fluent** — Given kesh-i18n configuré, When chargement des fichiers `.ftl` pour FR/DE/IT/EN, Then toutes les chaînes UI sont disponibles dans les 4 langues. Fallback : si une clé manque dans la locale demandée, utiliser FR-CH.
2. **Langue instance via env** — Given `KESH_LANG=de` (ou fr/it/en, défaut fr), When démarrage de l'API, Then les messages d'erreur et l'endpoint i18n utilisent la langue configurée.
3. **Formatage montants suisse** — Given un montant `1234.56`, When formaté via `kesh-i18n::format_money`, Then résultat `1'234.56` (apostrophe typographique U+2019). Négatif : `-1'234.56`. Zéro : `0.00`. Arrondi CHF : `1'234.55` (centimes, pas millièmes).
4. **Formatage dates suisse** — Given une date `2026-04-03`, When formatée via `kesh-i18n::format_date`, Then résultat `03.04.2026` (dd.mm.yyyy). Format identique pour les 4 langues (convention suisse).
5. **Organisation fichiers .ftl** — And les fichiers `.ftl` sont organisés : `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
6. **Tests unitaires** — And tests couvrant : formatage montants (positif, négatif, zéro, grands nombres) × 4 langues, formatage dates × 4 langues, chargement Fluent avec fallback, résolution de clés manquantes.
7. **Endpoint i18n** — Given un admin connecté, When `GET /api/v1/i18n/messages`, Then retourne `{ locale: "fr-CH", messages: { "key": "value", ... } }` dans la langue instance.
8. **Messages d'erreur traduits** — Given `KESH_LANG=de`, When une erreur API se produit, Then le champ `message` est en allemand (ex: "Ungültige Anmeldedaten" au lieu de "Identifiants invalides"). Le champ `code` reste `INVALID_CREDENTIALS`.

## Tasks / Subtasks

### T1 — Crate kesh-i18n : dépendances et structure (AC: #1, #5)
- [x] T1.1 Ajouter `fluent-bundle = "0.16"`, `fluent-syntax = "0.11"`, `intl-memoizer = "0.5"` dans `crates/kesh-i18n/Cargo.toml`. Ajouter `rust_decimal = "1"` et `chrono = "0.4"` pour les fonctions de formatage. Ajouter `thiserror = "2"` pour les erreurs. `rust_decimal_macros` en dev-dependencies.
- [x] T1.2 Créer `crates/kesh-i18n/src/lib.rs` : module public `loader`, `formatting`, `error`, type `Locale` enum (FrCh, DeCh, ItCh, EnCh) avec `impl From<&str>` et `impl Display`. Réexport de `FluentArgs`.
- [x] T1.3 Créer `crates/kesh-i18n/src/error.rs` : `I18nError` enum (FluentParse, MissingResource, Io).

### T2 — Loader Fluent (AC: #1, #5)
- [x] T2.1 Créer `crates/kesh-i18n/src/loader.rs` : struct `I18nBundle` contenant un `HashMap<Locale, ConcurrentBundle>` (FluentBundle concurrent pour Send+Sync) + `keys: HashMap<Locale, Vec<String>>` (FluentBundle 0.16 n'expose pas d'itérateur).
- [x] T2.2 Implémenter `I18nBundle::load(locales_dir: &Path) -> Result<Self, I18nError>` : charge les fichiers `{locale}/messages.ftl`, extrait les clés via fluent-syntax AST.
- [x] T2.3 Implémenter `I18nBundle::format(&self, locale: &Locale, key: &str, args: Option<&FluentArgs>) -> String` : résout un message. Fallback vers FrCh si la clé manque dans la locale demandée. Retourne la clé brute si absente partout.
- [x] T2.4 Implémenter `I18nBundle::all_messages(&self, locale: &Locale) -> HashMap<String, String>` : retourne toutes les paires clé/valeur avec fallback FR-CH.

### T3 — Fichiers .ftl initiaux (AC: #5, #8)
- [x] T3.1 Créer `crates/kesh-i18n/locales/fr-CH/messages.ftl` avec les messages d'erreur API actuels.
- [x] T3.2 Créer `crates/kesh-i18n/locales/de-CH/messages.ftl` — traductions allemandes.
- [x] T3.3 Créer `crates/kesh-i18n/locales/it-CH/messages.ftl` — traductions italiennes.
- [x] T3.4 Créer `crates/kesh-i18n/locales/en-CH/messages.ftl` — traductions anglaises.
- [x] T3.5 16 clés par locale : error-invalid-credentials, error-unauthenticated, error-forbidden, error-not-found, error-conflict, error-optimistic-lock, error-rate-limited, error-service-unavailable, error-validation, error-cannot-disable-self, error-cannot-disable-last-admin, error-invalid-refresh-token, error-username-empty, error-username-too-long, error-internal, error-foreign-key, error-check-constraint, error-illegal-state.

### T4 — Formatage suisse (AC: #3, #4)
- [x] T4.1 Créer `crates/kesh-i18n/src/formatting.rs` : `pub fn format_money(amount: &Decimal) -> String` — apostrophe typographique U+2019, 2 décimales toujours, signe `-` pour négatifs.
- [x] T4.2 `pub fn format_date(date: &NaiveDate) -> String` — format `dd.mm.yyyy`.
- [x] T4.3 `pub fn format_datetime(dt: &NaiveDateTime) -> String` — format `dd.mm.yyyy HH:MM`.

### T5 — Intégration kesh-api (AC: #2, #7, #8)
- [x] T5.1 Ajouter `kesh-i18n` comme dépendance de `kesh-api` dans Cargo.toml.
- [x] T5.2 Ajouter `KESH_LANG` dans `Config` (env, défaut "fr", mapping vers `Locale`).
- [x] T5.3 Charger `I18nBundle` au démarrage dans `main.rs` (via `KESH_LOCALES_DIR` ou auto-détection), l'ajouter à `AppState`. Initialiser `init_error_i18n` pour les messages d'erreur globaux.
- [x] T5.4 Modifier `AppError` dans `errors.rs` : `OnceLock` global pour i18n, helper `t(key, default)` pour résolution avec fallback. Tous les messages hardcodés remplacés par clés Fluent.
- [x] T5.5 Créer `routes/i18n.rs` : handler `GET /api/v1/i18n/messages` (authentifié, tout rôle) retournant `{ locale: "fr-CH", messages: { ... } }`.
- [x] T5.6 Enregistrer la route dans `build_router()` sous `authenticated_routes`.

### T6 — Tests (AC: #6)
- [x] T6.1 Tests unitaires `kesh-i18n` : `format_money` — 9 tests (positif, négatif, zéro, below 1000, large, centimes, rounding, exact 1000, million).
- [x] T6.2 Tests unitaires : `format_date` (3 tests) et `format_datetime` (2 tests).
- [x] T6.3 Tests unitaires : `I18nBundle::load` (succès), `format` (clé existante FR, clé existante DE, fallback FR, clé absente → clé brute).
- [x] T6.4 Tests unitaires : `all_messages` retourne toutes les clés, `format_with_args` interpole.
- [x] T6.5 Tests intégration `kesh-api` : `GET /api/v1/i18n/messages` retourne 200 + JSON avec locale et messages (+ test 401 sans auth).
- [x] T6.6 Tests intégration : erreur API (login échoué) retourne le message dans la langue instance.
- [x] T6.7 Tests config : `KESH_LANG` mapping Locale::from — couvert par tests unitaires lib.rs.

## Dev Notes

### Architecture kesh-i18n (ADR #13)

```
crates/kesh-i18n/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Réexports + Locale enum
│   ├── loader.rs       # I18nBundle (FluentBundle wrapper)
│   ├── formatting.rs   # format_money, format_date, format_datetime
│   └── error.rs        # I18nError
└── locales/
    ├── fr-CH/messages.ftl
    ├── de-CH/messages.ftl
    ├── it-CH/messages.ftl
    └── en-CH/messages.ftl
```

### État existant du codebase

- **kesh-i18n** : crate placeholder (lib.rs vide, répertoires locales avec `.gitkeep`)
- **kesh-db** : `Language` enum déjà défini (`Fr`, `De`, `It`, `En`) dans `entities/company.rs`. Company a `accounting_language` et `instance_language`.
- **kesh-api/errors.rs** : 14 messages d'erreur hardcodés en français à migrer vers Fluent.
- **kesh-api/routes/users.rs** : 2 messages de validation hardcodés en français.
- **Frontend** : sélecteur de langue dans le header (Story 1.10) avec items `disabled` + commentaire "Story 2.1". NE PAS toucher le frontend dans cette story.

### Dépendances croisées

```
kesh-i18n (cette story)
├── fluent-bundle 0.16
├── fluent-syntax 0.11
├── rust_decimal 1 (workspace)
├── chrono 0.4 (workspace)
└── thiserror 2 (workspace)

Consommateurs (futures stories) :
├── kesh-api (cette story : messages erreur + endpoint)
├── kesh-report (Epic 7 : formatage PDF)
└── kesh-qrbill (Epic 5 : formatage QR Bill)
```

### Formatage suisse — Spécifications exactes

| Type | Exemple | Règles |
|------|---------|--------|
| Montant positif | `1'234.56` | Apostrophe U+2019, 2 décimales, point décimal |
| Montant négatif | `-1'234.56` | Signe `-` préfixé |
| Montant zéro | `0.00` | Pas d'apostrophe |
| Montant < 1000 | `999.00` | Pas d'apostrophe |
| Date | `03.04.2026` | dd.mm.yyyy, zéro-padding |
| DateTime | `03.04.2026 14:30` | dd.mm.yyyy HH:MM |

### Fluent .ftl — Syntaxe de référence

```ftl
# Commentaire
error-invalid-credentials = Identifiants invalides
error-not-found = Ressource introuvable
error-username-too-long = Le nom d'utilisateur ne doit pas dépasser { $max } caractères
```

Arguments nommés avec `{ $variable }`. Pas besoin de pluralisation pour les messages d'erreur.

### Patterns établis (Epic 1)

- **Config** : `env_var_with_default!` ou `std::env::var` dans `Config::from_env()`. Warn + fallback si invalide (pattern Story 1.7).
- **AppState** : struct `AppState { pool, config, rate_limiter }` — ajouter `i18n: Arc<I18nBundle>`.
- **Routes** : `Router::new().route(...)` dans `build_router()`. Routes authentifiées via `.merge(authenticated_routes)`.
- **Erreurs** : `AppError` enum dans `errors.rs`, impl `IntoResponse`. Chaque variante a un `code` et un `message`.
- **Tests intégration** : `spawn_app()` pattern avec `reqwest::Client`, assertions sur status + JSON body.

### Piège critique — Locale vs Language

Le codebase a déjà un enum `Language` dans `kesh-db` (Fr, De, It, En) pour la DB. La story 2.1 crée un enum `Locale` dans `kesh-i18n` (FrCh, DeCh, ItCh, EnCh) pour Fluent. Les deux doivent coexister. Implémenter `From<Language> for Locale` dans kesh-i18n (ajouter kesh-db ou kesh-core comme dépendance, ou dupliquer le mapping — préférer le mapping simple `From<&str>`).

### Messages d'erreur à migrer (kesh-api/errors.rs)

| Code | FR actuel | Clé Fluent |
|------|-----------|------------|
| INVALID_CREDENTIALS | Identifiants invalides | error-invalid-credentials |
| UNAUTHENTICATED | Non authentifié | error-unauthenticated |
| FORBIDDEN | Accès interdit | error-forbidden |
| NOT_FOUND | Ressource introuvable | error-not-found |
| OPTIMISTIC_LOCK_CONFLICT | Conflit de version — la ressource a été modifiée | error-optimistic-lock |
| RESOURCE_CONFLICT | Ressource déjà existante | error-conflict |
| SERVICE_UNAVAILABLE | Service temporairement indisponible | error-service-unavailable |
| RATE_LIMITED | Trop de tentatives | error-rate-limited |
| INVALID_REFRESH_TOKEN | Session expirée | error-invalid-refresh-token |
| VALIDATION_ERROR | (dynamique) | error-validation |
| CANNOT_DISABLE_SELF | Impossible de désactiver son propre compte | error-cannot-disable-self |
| CANNOT_DISABLE_LAST_ADMIN | Impossible de désactiver le dernier administrateur | error-cannot-disable-last-admin |

Plus dans `routes/users.rs` :
| Message FR | Clé Fluent |
|-----------|------------|
| Le nom d'utilisateur ne peut pas être vide | error-username-empty |
| Le nom d'utilisateur ne doit pas dépasser 64 caractères | error-username-too-long |

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#ADR-13] — kesh-i18n crate design
- [Source: _bmad-output/planning-artifacts/prd.md#FR75-FR76] — Multilingue + séparation langue comptable/interface
- [Source: _bmad-output/planning-artifacts/epics.md#Epic-2-Story-2.1] — AC BDD
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Onboarding] — Choix de langue UX

## Dev Agent Record

### Agent Model Used
Opus 4.6

### Completion Notes
- ✅ T1 : Crate kesh-i18n structurée (Locale enum, error, formatting, loader) avec FluentBundle concurrent (Send+Sync)
- ✅ T2 : Loader Fluent avec fallback FR-CH, extraction des clés via fluent-syntax AST (FluentBundle 0.16 n'a pas d'itérateur)
- ✅ T3 : 4 fichiers .ftl (fr-CH, de-CH, it-CH, en-CH), 18 clés chacun couvrant tous les messages d'erreur API
- ✅ T4 : Formatage suisse (apostrophe U+2019, dd.mm.yyyy), 14 tests unitaires
- ✅ T5 : Intégration kesh-api (Config.locale, AppState.i18n, OnceLock pour erreurs, endpoint /api/v1/i18n/messages, migration messages users.rs)
- ✅ T6 : 21 tests unitaires kesh-i18n + 3 tests E2E kesh-api (endpoint + auth + erreur traduite)
- Piège résolu : FluentBundle standard n'est pas Send+Sync → FluentBundle::new_concurrent + intl-memoizer

### File List

#### New Files
- `crates/kesh-i18n/src/lib.rs` — Module principal, Locale enum, réexports
- `crates/kesh-i18n/src/error.rs` — I18nError enum
- `crates/kesh-i18n/src/loader.rs` — I18nBundle (chargement Fluent + résolution + fallback)
- `crates/kesh-i18n/src/formatting.rs` — format_money, format_date, format_datetime (suisse)
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — Traductions françaises
- `crates/kesh-i18n/locales/de-CH/messages.ftl` — Traductions allemandes
- `crates/kesh-i18n/locales/it-CH/messages.ftl` — Traductions italiennes
- `crates/kesh-i18n/locales/en-CH/messages.ftl` — Traductions anglaises
- `crates/kesh-api/src/routes/i18n.rs` — Handler GET /api/v1/i18n/messages
- `crates/kesh-api/tests/i18n_e2e.rs` — Tests E2E i18n

#### Modified Files
- `crates/kesh-i18n/Cargo.toml` — Dépendances fluent-bundle, fluent-syntax, intl-memoizer, rust_decimal, chrono, thiserror
- `crates/kesh-api/Cargo.toml` — Ajout dépendance kesh-i18n
- `crates/kesh-api/src/lib.rs` — AppState + i18n field, route i18n enregistrée
- `crates/kesh-api/src/config.rs` — Config.locale (KESH_LANG env), from_fields_for_test, make_test_config
- `crates/kesh-api/src/errors.rs` — OnceLock i18n global, helper t(), messages traduits via Fluent
- `crates/kesh-api/src/routes/mod.rs` — pub mod i18n
- `crates/kesh-api/src/routes/users.rs` — Messages validation via i18n
- `crates/kesh-api/src/middleware/auth.rs` — Test helper AppState + i18n
- `crates/kesh-api/tests/auth_e2e.rs` — AppState + i18n
- `crates/kesh-api/tests/rbac_e2e.rs` — AppState + i18n
- `crates/kesh-api/tests/users_e2e.rs` — AppState + i18n
- `Dockerfile` — COPY locales + KESH_LOCALES_DIR env

## Change Log

| Date | Passe | Modèle | Findings | Patches |
|------|-------|--------|----------|---------|
| 2026-04-08 | Implémentation | Opus 4.6 | — | T1-T6 complètes, 21 tests kesh-i18n + 3 E2E, build OK |
| 2026-04-08 | Code review passe 1 | Sonnet 4.6 (3 agents) | 7 patch, 3 defer, 5 rejetés | P1: -0.00 guard, P2: log Fluent errors, P3: init_error_i18n warning, P4: locale inconnue warning, P5: with_locale builder + test E2E locale DE, P6: test fallback corrigé, P7: rounding MidpointAwayFromZero |
| 2026-04-08 | Code review passe 2 | Haiku 4.5 (2 agents) | 1 patch, 2 rejetés | P8: OnceLock → RwLock pour permettre tests multi-locales (test DE flaky avec OnceLock) |
