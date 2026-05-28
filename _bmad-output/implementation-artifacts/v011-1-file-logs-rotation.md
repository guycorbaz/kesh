# Story v011.1: Logs fichier avec rotation (Issue #119)

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a administrateur Kesh en production sur NAS Synology,
I want que kesh-api écrive ses logs dans un fichier avec rotation native (en plus de stdout/`docker logs`), configurable par variables d'environnement,
so that je puisse consulter et conserver les logs hors de la rétention `docker logs` (purgée à chaque `docker compose down/up`), les inclure dans le backup Hyper Backup co-localisé avec la DB et le `.env`, et faciliter le debugging des stories v011-2 (catch-22) et v011-3 (break-glass) en dev comme en prod.

## Scope

**Story-zéro infrastructure** de l'Epic Hotfix v0.1.1 (décision H9 : livrée en premier pour donner aux 3 stories suivantes une observabilité accessible hors `docker logs`).

Ajouter une **seconde sortie de logs fichier avec rotation native** via le crate `tracing-appender`, en gardant intacte la sortie stdout existante. La nouvelle sortie est **configurable par env vars** et **activée par défaut en prod** via un mount `./log/` dans `docker-compose.prod.yml`.

**Pas de schema change DB. Pas de nouvel endpoint. Pas de feature comptable.** Modification du bootstrap logging de `crates/kesh-api/src/main.rs` + ajout d'une config logging dans `crates/kesh-api/src/config.rs` + une dépendance Rust + doc + compose.

**Hors scope** (autres stories de l'epic) :
- Fix catch-22 onboarding → Story v011-2.
- Break-glass admin reset → Story v011-3.
- Port défaut 80 → Story v011-4.
- Centralisation/agrégation de logs (ELK, Loki, syslog distant) → hors v0.1.x (over-engineering pour un déploiement solo NAS).

## Contexte technique de départ (lu 2026-05-28)

**Setup logging actuel** — `crates/kesh-api/src/main.rs:34-37` :

```rust
// 1. Logging
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    .init();
```

Subscriber **mono-layer** (stdout uniquement), initialisé à l'**étape 1**, **AVANT** `Config::from_env()` (étape 2, ligne 40). Les erreurs de config (ligne 43, `tracing::error!("Erreur de configuration: {}", e)`) sont donc actuellement capturées par le subscriber stdout déjà en place.

**⚠️ Guardrail d'ordonnancement principal de cette story** : le layer fichier a besoin des valeurs `KESH_LOG_FILE_*`. Si on déplace naïvement l'init du subscriber après `Config::from_env()`, les erreurs fatales de config (JWT secret absent, etc.) émises *avant* l'init ne seraient plus capturées. Voir Dev Notes §"Ordonnancement boot" pour l'approche retenue.

**Dépendances actuelles** — `crates/kesh-api/Cargo.toml:27-28` :

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

Pas de `tracing-appender`. Pas de feature `json` sur `tracing-subscriber` (requise pour `KESH_LOG_FILE_FORMAT=json`).

## Acceptance Criteria

### Dépendances (AC #1)

1. **Given** `crates/kesh-api/Cargo.toml`, **When** review, **Then** :
   - `tracing-appender = "0.2"` est ajouté à `[dependencies]` (dernière version stable `0.2.3`, compatible `tracing-subscriber 0.3`).
   - La dépendance `tracing-subscriber` gagne la feature `json` : `features = ["env-filter", "json"]` (requise pour le format JSON, AC #5).

### Configuration logging (AC #2-6)

2. **Given** `crates/kesh-api/src/config.rs`, **When** review, **Then** une configuration logging est lue depuis l'environnement avec les 4 variables suivantes, leurs defaults et validations :

   | Variable | Type | Default | Validation |
   |---|---|---|---|
   | `KESH_LOG_FILE_PATH` | `Option<String>` (chemin fichier complet) | `None` (= logs fichier désactivés) | chaîne vide ou absente → `None` (opt-out explicite) |
   | `KESH_LOG_FILE_ROTATION` | enum `daily`/`hourly`/`never` | `daily` | valeur inconnue → `warn!` + fallback `daily` |
   | `KESH_LOG_FILE_MAX_FILES` | `usize` | `7` | `0` ou non-numérique → `warn!` + fallback `7` |
   | `KESH_LOG_FILE_FORMAT` | enum `pretty`/`json` | `pretty` | valeur inconnue → `warn!` + fallback `pretty` |

   Le pattern de parsing (match sur `env::var`, fallback sur valeur invalide, jamais de panic) suit le pattern existant `KESH_PORT` / `KESH_BANK_IMPORT_MAX_MB` (`config.rs:360-375`, `config.rs:642+`). **Nuance d'ordonnancement (cf. Dev Notes §"Ordonnancement boot")** : contrairement à `KESH_PORT` (parsé après l'init du subscriber), `LogConfig::from_env()` est appelé **avant** l'init du subscriber. Il ne doit donc PAS émettre `tracing::warn!` directement (perdu silencieusement). Il **collecte** les messages de fallback dans un `Vec<String>` retourné au caller, qui les **rejoue via `tracing::warn!` après `.init()`** (signature recommandée : `LogConfig::from_env() -> (LogConfig, Vec<String>)` — pas de `Result` car le parsing est infaillible par construction, tout est fallback).

3. **Given** `KESH_LOG_FILE_PATH` absent OU chaîne vide, **When** boot, **Then** le layer fichier n'est **pas** créé (`None`), le comportement est identique à aujourd'hui (stdout uniquement), aucun fichier n'est ouvert, aucune erreur. Garantit la **rétro-compatibilité totale** : un déploiement v0.1.0 qui n'a pas la var continue à logger en stdout exactement comme avant.

4. **Given** `KESH_LOG_FILE_ROTATION` valant `daily`, `hourly`, ou `never` (case-insensitive), **When** parsing, **Then** mappé respectivement vers `tracing_appender::rolling::Rotation::DAILY`, `HOURLY`, `NEVER`. Toute autre valeur → `tracing::warn!("KESH_LOG_FILE_ROTATION='{}' invalide, fallback 'daily'", v)` + `DAILY`.

5. **Given** `KESH_LOG_FILE_FORMAT` valant `pretty` ou `json` (case-insensitive), **When** parsing, **Then** le layer fichier utilise respectivement le format human-readable (`fmt::layer()` sans `.json()`) ou JSON structuré (`fmt::layer().json()`). Le layer fichier émet **sans ANSI** (`.with_ansi(false)`) dans les deux cas (un fichier ne doit jamais contenir de codes couleur terminal).

6. **Given** la config logging, **When** ses champs sont parsés, **Then** des tests unitaires couvrent : defaults (toutes vars absentes → `path=None, rotation=daily, max_files=7, format=pretty`), chaque valeur valide de chaque enum, et le fallback sur valeur invalide (rotation/format/max_files), via un helper de parsing pur testable sans I/O fichier (cf. pattern de testabilité `config.rs`).

### Subscriber multi-layer (AC #7-9)

7. **Given** `crates/kesh-api/src/main.rs`, **When** review, **Then** le subscriber est restructuré de `tracing_subscriber::fmt().init()` (mono-layer) vers `tracing_subscriber::registry()` composé de :
   - le même `EnvFilter` qu'aujourd'hui (`try_from_default_env().unwrap_or_else(|_| "info".into())`),
   - un **layer stdout** reproduisant le comportement actuel (préservation stricte — même format human-readable par défaut sur stdout),
   - un **layer fichier conditionnel** (`Option`) : `Some` si `KESH_LOG_FILE_PATH` est défini, `None` sinon (un `Option<Layer>` est lui-même un `Layer` no-op si `None` — pattern natif `tracing-subscriber`).

8. **Given** `KESH_LOG_FILE_PATH=/var/log/kesh/kesh.log`, **When** le serveur démarre, **Then** :
   - Le chemin est décomposé via `std::path::Path` : `dir = parent()`, `prefix = file_stem()`, `suffix = extension()`. Pour `/var/log/kesh/kesh.log` → `dir=/var/log/kesh`, `prefix=kesh`, `suffix=log`. Cas limites à gérer explicitement : `extension()` = `None` (ex. `.../kesh`) → `suffix=""` (valide pour tracing-appender) ; `parent()` = `None` ou vide (ex. `kesh.log` sans dir) → fallback dir `.` (cwd) ; chemin se terminant par `/` ou `file_stem()` vide → chemin invalide → dégradation gracieuse AC #9 (stdout-only + message d'erreur).
   - `dir`, `prefix`, `suffix` sont passés à `RollingFileAppender::builder().rotation(...).filename_prefix(prefix).filename_suffix(suffix).max_log_files(...).build(dir)`.
   - Le writer est wrappé par `tracing_appender::non_blocking(appender)` qui retourne `(NonBlocking, WorkerGuard)`.
   - **Le `WorkerGuard` est conservé vivant pour toute la durée du programme** (lié à une variable dans `main`, p.ex. `let _log_guard = ...;` qui n'est PAS droppée avant `axum::serve`). Sans cela, le writer non-bloquant flush partiellement et des logs sont perdus à l'arrêt. **C'est le gotcha le plus important de `tracing-appender`.**

9. **Given** l'init du subscriber, **When** l'ouverture du fichier de log échoue (répertoire inexistant non créable, permissions insuffisantes), **Then** le serveur **ne crashe pas** : il log un `tracing::error!` (ou `eprintln!` si avant subscriber, cf. Dev Notes) explicite « impossible d'ouvrir le fichier de log <path>: <err> — logs fichier désactivés, stdout conservé » et **continue en stdout-only** (dégradation gracieuse — les logs fichier sont une commodité, pas une dépendance critique au boot).

### Tests (AC #10-11)

10. **Given** les helpers de parsing config logging, **When** `cargo test -p kesh-api`, **Then** les tests unitaires de l'AC #6 passent.

11. **Given** un test d'intégration, **When** un subscriber est configuré avec un `KESH_LOG_FILE_PATH` pointant vers un `tempfile`/répertoire temporaire + quelques `tracing::info!`/`error!` émis, **Then** après flush (drop explicite du guard ou `tracing_appender` flush), le fichier de log existe et contient les entrées émises (assertion sur le contenu). Le test ne dépend d'aucun service externe (pas de DB). Si l'isolation du global subscriber rend le test fragile, encapsuler la construction des layers dans une fonction pure retournant les layers/un writer testable et tester cette fonction (cf. Dev Notes §"Testabilité").

### Infra & déploiement (AC #12-14)

12. **Given** `docker-compose.prod.yml`, **When** review, **Then** :
    - un mount `./log:/var/log/kesh` est ajouté au service `kesh-api` (volume host relatif au cwd du compose, co-localisé avec `.env` et la DB pour le scope unique Hyper Backup — décision H4),
    - `KESH_LOG_FILE_PATH: ${KESH_LOG_FILE_PATH:-/var/log/kesh/kesh.log}` est ajouté à la section `environment`,
    - les commentaires d'en-tête du fichier documentent le nouveau mount + l'opt-out (`KESH_LOG_FILE_PATH=""` pour stdout pur),
    - le plafond `logging: json-file max-size/max-file` existant (stdout Docker) est **conservé** (les deux sorties coexistent).

13. **Given** `.gitignore`, **When** review, **Then** une entrée `log/` (et/ou `*.log`) est ajoutée pour ne **jamais** committer de fichiers de log. **Given** `docker-compose.dev.yml`, **When** review, **Then** l'alignement dev est cohérent (au minimum la var documentée ; le mount dev est optionnel selon Dev Notes).

14. **Given** `.env.example`, **When** review, **Then** une nouvelle section « Logs fichier » documente les 4 variables avec exemples commentés, le comportement par défaut (activé en prod via le compose), l'opt-out, et un **warning sur les permissions du volume host** (le container tourne en root → fichiers owned `root` côté NAS ; voir Q4).

### Documentation (AC #15-16)

15. **Given** `docs/manual/fr/admin-manual.tex`, **When** review, **Then** une sous-section « Configuration des logs fichier » est ajoutée dans le § Configuration : explique les 4 vars, l'emplacement par défaut, la rotation, le format JSON pour ingestion outillée, et la consultation côté NAS (`tail -f ./log/kesh.log`). Le PDF est régénéré (`latexmk -xelatex` dans `docs/manual/fr/`) et committé (convention projet PR #102).

16. **Given** `CHANGELOG.md` (qui contient aujourd'hui `## [Non publié]` vide + `## [0.1.0]`), **When** review, **Then** la section `## [Non publié]` est renommée `## [0.1.1] — <date>` (convention Keep a Changelog) et reçoit une entrée `Added` documentant les logs fichier avec rotation. **Ne pas** créer de section `[0.1.1]` dupliquée ni laisser un `[Non publié]` vide en double. (Les stories v011-2/3/4 ajouteront leurs entrées dans cette même section `[0.1.1]`.)

### Qualité (AC #17)

17. **Given** la branche de la story, **When** la série « Test Locally First » backend (`cargo fmt --check` + `build` + `clippy -D warnings` + `test --workspace`) et frontend (no-op, pas de changement frontend) tourne, **Then** tout est vert, **0 régression** sur les baselines existantes (cargo test workspace, 262 Vitest, E2E Playwright non impactés). La CI est verte.

## Tasks / Subtasks

- [ ] **T1 — Dépendances** (AC: #1)
  - [ ] Ajouter `tracing-appender = "0.2"` à `crates/kesh-api/Cargo.toml`.
  - [ ] Ajouter la feature `json` à `tracing-subscriber`.
  - [ ] `cargo build -p kesh-api` confirme la résolution.
- [ ] **T2 — Config logging** (AC: #2-6)
  - [ ] Ajouter une struct `LogConfig` (ou champs équivalents) + parsing `from_env()` dans `config.rs`, avec les 4 vars, defaults et fallbacks `warn!`.
  - [ ] Enum interne `LogRotation { Daily, Hourly, Never }` + `LogFormat { Pretty, Json }` avec parsing case-insensitive.
  - [ ] Helper de parsing pur (sans lecture env directe) pour testabilité.
  - [ ] Tests unitaires defaults + valeurs valides + fallbacks (AC #6).
- [ ] **T3 — Construction des layers** (AC: #7-9)
  - [ ] Fonction qui construit le layer fichier `Option` depuis `LogConfig` : `RollingFileAppender::builder()` + `non_blocking()` → `(layer, Option<WorkerGuard>)`.
  - [ ] Gestion d'erreur d'ouverture fichier → dégradation gracieuse stdout-only (AC #9).
  - [ ] Mapping rotation/format → API tracing-appender / fmt::layer (`.json()`, `.with_ansi(false)`).
- [ ] **T4 — Bootstrap main.rs** (AC: #7-8)
  - [ ] Restructurer `main.rs` : lire `LogConfig` tôt, init `registry().with(filter).with(stdout_layer).with(file_layer)`, **conserver le `WorkerGuard`** dans `main` (cf. §"Ordonnancement boot").
  - [ ] Vérifier que les erreurs de config (étape 2) restent loggées.
- [ ] **T5 — Test d'intégration fichier** (AC: #11)
  - [ ] Ajouter `tempfile = "3"` à `[dev-dependencies]` de `crates/kesh-api/Cargo.toml` (confirmé absent au 2026-05-28).
  - [ ] Test : layers construits avec un path temporaire → émission → flush → assertion contenu fichier.
- [ ] **T6 — Infra** (AC: #12-14)
  - [ ] `docker-compose.prod.yml` : mount `./log:/var/log/kesh` + env `KESH_LOG_FILE_PATH` + commentaires.
  - [ ] `docker-compose.dev.yml` : alignement (var documentée).
  - [ ] `.gitignore` : `log/`.
  - [ ] `.env.example` : section « Logs fichier » + warning perms.
- [ ] **T7 — Doc** (AC: #15-16)
  - [ ] `admin-manual.tex` sous-section + régénération PDF.
  - [ ] `CHANGELOG.md` entrée `Added` sous `[0.1.1]`.
- [ ] **T8 — Quality gate** (AC: #17)
  - [ ] Série Test Locally First backend complète, 0 régression.

### Review Findings (Pass 1 Sonnet 4.6 — 2026-05-28) — CYCLE CONVERGÉ

**Trend** : Pass 1 = 0C/0H/0M/**2 LOW** → critère d'arrêt CLAUDE.md §"Review Iteration Rule" atteint dès la passe 1 (uniquement LOW). Aucune passe supplémentaire requise.

- [x] [Review][Patch] **R2 LOW** `decompose_path` rejette un chemin se terminant par séparateur (`/` ou `\`) : sans ce guard, `Path::file_stem` renverrait le nom du répertoire (`/var/log/kesh/` → stem `kesh`, parent `/var/log`) et écrirait le log un niveau trop haut → `None` → dégradation gracieuse stdout-only + test `decompose_trailing_slash_is_invalid` [`crates/kesh-api/src/logging.rs:decompose_path`]
- [x] [Review][Patch] **R1 LOW** Note de limitation v0.1 ajoutée : les `std::process::exit(1)` des paths d'erreur fatale de boot ne droppent pas `_log_guard` → les derniers logs fichier bufferisés (non-bloquants) peuvent être perdus ; stdout (`docker logs`) reste fiable pour ces erreurs — acceptable v0.1 [`crates/kesh-api/src/main.rs`]

## Dev Notes

### Ordonnancement boot (le point critique)

Le subscriber est aujourd'hui init **avant** `Config::from_env()`. Le layer fichier a besoin de `KESH_LOG_FILE_*`. **Approche recommandée** (le dev peut diverger s'il justifie) :

1. Lire **uniquement** la config logging tôt via un `LogConfig::from_env()` léger et **infaillible** (jamais de panic/exit — fallbacks `warn!`), indépendant de la validation lourde de `Config::from_env()` (qui, elle, fait fail-fast sur JWT secret etc.).
2. Init le `registry()` (stdout + file conditionnel) avec cette `LogConfig`.
3. **Ensuite** `Config::from_env()` — ses erreurs fatales (ligne 43) sont désormais capturées par le subscriber déjà en place (stdout + éventuellement fichier).

Conséquence : les `warn!` de fallback de `LogConfig` (rotation/format invalides) sont émis *pendant* l'init du subscriber. Soit on les émet juste après l'init (le subscriber est up), soit on collecte les warnings et on les rejoue après init. Ne PAS émettre de `tracing::*` avant que le subscriber soit installé (ils seraient perdus silencieusement). Pour les erreurs d'ouverture de fichier (AC #9) qui surviennent *pendant* la construction des layers (donc avant `.init()`), utiliser `eprintln!` (stderr) comme fallback, puis continuer stdout-only.

### tracing-appender 0.2.3 — API (vérifié web 2026-05-28)

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let appender = RollingFileAppender::builder()
    .rotation(Rotation::DAILY)          // ou HOURLY / NEVER
    .filename_prefix("kesh")            // → fichiers "kesh.2026-05-28" etc.
    .filename_suffix("log")
    .max_log_files(7)                   // garde les 7 plus récents
    .build("/var/log/kesh")?;           // Result<_, InitError> — gérer l'erreur (AC #9)

let (non_blocking, guard) = tracing_appender::non_blocking(appender);
// `guard` (WorkerGuard) DOIT rester vivant tout le programme — sinon perte de logs au flush.
```

- Décomposer `KESH_LOG_FILE_PATH` (`/var/log/kesh/kesh.log`) en `dir = /var/log/kesh`, `prefix = kesh`, `suffix = log` pour le builder. Documenter clairement le mapping (le builder ne prend pas un chemin de fichier unique mais dir+prefix+suffix ; avec rotation `NEVER`, le fichier reste `kesh.log`).
- `Rotation::NEVER` + `max_log_files` : `max_log_files` n'est honoré que quand la rotation crée de **nouveaux** fichiers. Avec `NEVER`, un fichier unique est réutilisé indéfiniment et `max_log_files` est **silencieusement ignoré** (aucun ancien fichier à élaguer). Le documenter dans le commentaire `KESH_LOG_FILE_MAX_FILES` de `.env.example` (AC #14) et ne PAS écrire de test attendant un plafonnement du nombre de fichiers avec `NEVER`.

### Layers tracing-subscriber

```rust
use tracing_subscriber::{prelude::*, fmt, EnvFilter, registry};

let file_layer = log_cfg.file_path.as_ref().map(|_| {
    let layer = fmt::layer().with_writer(non_blocking).with_ansi(false);
    match log_cfg.format { LogFormat::Json => layer.json().boxed(), LogFormat::Pretty => layer.boxed() }
});

registry()
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    .with(fmt::layer())          // stdout — comportement actuel préservé
    .with(file_layer)            // Option<Layer> = no-op si None
    .init();
```

- Le `.json()` exige la feature `json` de `tracing-subscriber` (AC #1).
- `.boxed()` (type erasure via `Layer::boxed`) si les deux branches du `match` ont des types différents — sinon erreur de type sur le `match`.
- **Préserver le comportement stdout** : le `fmt::layer()` par défaut reproduit le rendu de `tracing_subscriber::fmt()`. Vérifier visuellement qu'un `docker logs` reste lisible comme avant.

### Testabilité

Le global subscriber ne peut être init qu'une fois par process → tests fragiles si on appelle `.init()`. Préférer tester (a) le **parsing** `LogConfig` (pur, AC #6) et (b) pour l'AC #11, construire un appender vers un répertoire temporaire et écrire via le `NonBlocking` writer directement (ou un subscriber local via `tracing::subscriber::with_default(...)` scoped au test), puis flush (drop du guard) et lire le fichier. `tempfile` est probablement déjà une dev-dependency — vérifier `crates/kesh-api/Cargo.toml [dev-dependencies]` avant d'ajouter.

### Volume Docker & permissions (Q4 epic)

Le container `kesh-api` tourne en **root** (pas de directive `USER` dans le `Dockerfile`). Les fichiers écrits dans `/var/log/kesh` (→ `./log/` host) seront owned `root:root` côté NAS. À vérifier en prod réelle Guy : Hyper Backup lit-il correctement des fichiers root ? (Probablement oui, le daemon backup tourne en root.) Documenter dans `.env.example` (AC #14) que le répertoire `./log/` est créé au premier boot avec owner root. Ne PAS tenter de changer l'UID du process dans cette story (hors scope ; le run-as-non-root est une dette sécurité distincte éventuelle).

### Project Structure Notes

- Fichiers **UPDATE** : `crates/kesh-api/src/main.rs` (bootstrap logging), `crates/kesh-api/src/config.rs` (LogConfig), `crates/kesh-api/Cargo.toml` (deps), `docker-compose.prod.yml`, `docker-compose.dev.yml`, `.gitignore`, `.env.example`, `docs/manual/fr/admin-manual.tex` (+ PDF), `CHANGELOG.md`.
- Fichiers **NEW** : éventuellement un test d'intégration `crates/kesh-api/tests/logging_file.rs` (ou un module `#[cfg(test)]` dans config.rs pour le parsing).
- Aucune migration DB, aucun nouveau crate workspace.

### References

- [Source: _bmad-output/planning-artifacts/epic-hotfix-v0.1.1.md#Story v011-1] — scope, ACs high-level, décisions H4/H9, Q4.
- [Source: crates/kesh-api/src/main.rs:34-37] — setup logging actuel (mono-layer stdout, avant config).
- [Source: crates/kesh-api/Cargo.toml:27-28] — deps tracing actuelles (pas d'appender, pas de feature json).
- [Source: crates/kesh-api/src/config.rs:360-375] — pattern de parsing env var avec fallback `warn!` (modèle pour KESH_LOG_FILE_*).
- [Source: docs.rs/tracing-appender/0.2.3] — API RollingFileAppender::builder + non_blocking + WorkerGuard (vérifié web 2026-05-28).
- [Source: docker-compose.prod.yml] — service kesh-api, section environment + logging json-file existant + commentaires d'en-tête.
- [Source: CLAUDE.md#Test Locally First] — checks backend obligatoires avant push.
- Issue GitHub #119 — demande initiale logs fichier rotation.

## Change Log

### Spec validate (cycle convergé en 2 passes — 2026-05-28)

Boucle adversariale LLMs rotatifs, contexte frais par passe (CLAUDE.md Review Iteration Rule) :

| Passe | LLM | Findings | Détail |
|---|---|---|---|
| 1 | Sonnet 4.6 | 2 MEDIUM + 3 LOW | F1 warn! avant subscriber → collecte+replay post-init ; F2 algo décomposition path + cas limites ; F3 NEVER ignore max_log_files ; F4 tempfile dev-dep ; F5 CHANGELOG rename [Non publié]→[0.1.1] |
| 2 | Haiku 4.5 | **0 > LOW** (1 LOW) | signature `LogConfig::from_env() -> (LogConfig, Vec<String>)` précisée. Discipline grep ground-truth appliquée, 0 faux-positif. |

**Trend** : passe 1 = 5 findings (2M+3L) → passe 2 = 1 finding (0 > LOW). Critère d'arrêt atteint (uniquement LOW). Spec `ready-for-dev` confirmée. Prochaine étape : `dev-story`.

### Code-review (cycle convergé en 1 passe Sonnet — 2026-05-28)

Boucle adversariale post-`dev-story` (CLAUDE.md §"Review Iteration Rule") :

| Passe | LLM | Findings | Patches |
|---|---|---|---|
| 1 | Sonnet 4.6 | 0C / 0H / 0M / **2 LOW** | R2 guard trailing-slash `decompose_path` + test ; R1 note limitation flush `process::exit(1)` |

**Trend** : passe 1 = 2 findings (0 > LOW) → critère d'arrêt atteint dès la passe 1. Patches commités `fb43d50`. Détail des findings : cf. §"Review Findings (Pass 1 Sonnet 4.6)" sous Tasks / Subtasks. Status `review` (bump `done` au merge de la PR, convention projet `done` = mergé). Prochaine étape : Test Locally First complet (T8) + git push + PR.

## Dev Agent Record

### Agent Model Used

Opus 4.7 (1M context) — dev-story single-pass orchestré 2026-05-28.

### Debug Log References

- `cargo fmt --all -- --check` : clean (après auto-format de 4 tests log_config).
- `cargo clippy --workspace --all-targets -- -D warnings` : clean (0 warning).
- `cargo test -p kesh-api --lib` : 189 passed / 0 failed (dont 14 nouveaux : 8 `config::tests::log_config_*` + 6 `logging::tests::*` incl. test fichier d'intégration `file_appender_writes_entries`).
- Échecs `kesh-db` en run groupé (`cargo test --workspace`) : environnementaux (état DB dev partagé, non réinitialisé comme en CI) — `test_filter_by_search` passe en isolation. Aucun code kesh-db touché par cette story → non régression. Verts en CI (DB seedée + `-j1 --test-threads=1`).

### Completion Notes List

- T1 deps : `tracing-appender = "0.2"` + feature `json` sur `tracing-subscriber` + `tempfile` dev-dep.
- T2 `LogConfig` (séparé de `Config`, parsing infaillible, warnings collectés) + 8 tests parsing purs.
- T3/T4 `crates/kesh-api/src/logging.rs` : `decompose_path`, `build_appender` (NEVER ignore `max_log_files`), `init_tracing` (registry stdout + fichier conditionnel `Option<BoxedLayer>`, `with_ansi(false)`, `json()` selon format), `WorkerGuard` retourné `#[must_use]`.
- main.rs : `dotenvy::dotenv()` AVANT `LogConfig::from_env()`, `_log_guard` gardé vivant, warnings rejoués post-init.
- T5 test fichier via subscriber local scoped (`with_default`) + flush sur drop guard.
- T6 infra : mount `./log:/var/log/kesh` + 4 env vars prod compose, alignement dev (off par défaut), `.gitignore log/`, `.env.example` section + warning perms root.
- T7 doc : `admin-manual.tex` §"Configuration des logs fichier" + PDF régénéré (xelatex), `CHANGELOG.md` [0.1.1] Ajouts.

### File List

- `crates/kesh-api/Cargo.toml` (deps tracing-appender + json + tempfile)
- `crates/kesh-api/src/config.rs` (LogConfig + enums + parsing + tests)
- `crates/kesh-api/src/logging.rs` (NEW — layers + init)
- `crates/kesh-api/src/lib.rs` (`pub mod logging`)
- `crates/kesh-api/src/main.rs` (bootstrap logging restructuré)
- `Cargo.lock` (tracing-appender 0.2.5 + tempfile 3)
- `docker-compose.prod.yml` (mount + env logs)
- `docker-compose.dev.yml` (env logs, off par défaut)
- `.gitignore` (`log/`)
- `.env.example` (section Logs fichier)
- `docs/manual/fr/admin-manual.tex` + `admin-manual.pdf`
- `CHANGELOG.md` ([0.1.1] Ajouts)
