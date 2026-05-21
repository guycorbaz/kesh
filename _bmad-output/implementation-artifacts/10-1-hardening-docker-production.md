# Story 10.1: Hardening Docker production

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a administrateur Kesh déployant sur NAS Synology,
I want un livrable Docker production durci (`docker-compose.prod.yml` séparé sans service `mariadb` bundled, fail-fast au boot si secrets default/faibles, log rotation + resource limits, `.env.example` sans placeholder insecure, alignement MariaDB 10.11 dans toutes les configurations dev/CI),
so that je puisse mettre Kesh en production sur mon NAS sans configuration sécurité supplémentaire, sans risque de saturation disque/mémoire à long terme, et sans risque de drift entre la version MariaDB testée en CI et celle utilisée en production.

## Scope

Cette story livre le **fichier `docker-compose.prod.yml`** distinct du compose dev, qui constitue le livrable principal de la release v0.1.0. Le service `mariadb` n'y est **pas** bundled — l'utilisateur fournit sa propre instance MariaDB 10.11+ (sur NAS Synology Package Center DSM dans le cas de Guy). Le réseau Docker est **externe** (`external: true`), créé par l'utilisateur avant le `docker compose up` (décision D9 epic-10.md).

**Renforcement boot** dans `crates/kesh-api/src/config.rs` : refuser le démarrage du serveur si `KESH_JWT_SECRET` ou `KESH_ADMIN_PASSWORD` contiennent un placeholder ou sont trop faibles. Les warnings existants (lignes 366-370 et 385-389 actuelles) deviennent des `ConfigError` qui font exit le binaire (vs `tracing::warn!` qui laisse passer).

**Création `.env.example`** à la racine du repo (référencé dans `architecture.md:429` mais inexistant à ce jour) avec toutes les variables d'env documentées, **aucun default insecure** (placeholders explicites `<GENERATE_ME: openssl rand -base64 32>` etc.), et commentaires pédagogiques.

**Alignement MariaDB 10.11 partout** (décision D3 epic-10.md) :
- `docker-compose.yml:4` `mariadb:11-jammy` → `mariadb:10.11`
- `docker-compose.dev.yml:43` `mariadb:11.4` → `mariadb:10.11`
- `.github/workflows/ci.yml:27` `mariadb:11.4` → `mariadb:10.11`
- Commentaire migration `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:28` ajusté pour refléter le nouveau pin.

Cette story n'ajoute **aucune feature comptable** et ne touche aucun code métier — c'est exclusivement du hardening infra/config.

**Hors scope** (couverts par d'autres stories Epic 10) :
- Migrations idempotence + `_kesh_version` table + downgrade protection au boot — Story 10-2.
- Résilience frontend si DB inaccessible — Story 10-3.
- Manuel install Synology + procédure backup DSM + CHANGELOG.md — Story 10-4.
- Tokens cookies httpOnly + endpoint `/me` + retrait localStorage — Story 10-5.
- `release.yml` smoke test post-build — amélioration parallèle à la fin d'Epic 10.

## Acceptance Criteria

### docker-compose.prod.yml (nouveau fichier)

1. **Given** la racine du repo, **When** un fichier `docker-compose.prod.yml` est créé, **Then** il contient **un seul service** `kesh-api` (pas de service `mariadb` ni autre).

2. **Given** `docker-compose.prod.yml`, **When** `docker compose -f docker-compose.prod.yml config` est exécuté, **Then** la validation YAML PASS et aucune référence à un service `mariadb` interne n'apparaît dans la sortie.

3. **Given** `docker-compose.prod.yml`, **When** review, **Then** il déclare un réseau **externe** explicite : `networks: { kesh-net: { external: true } }` (nom `kesh-net` ou équivalent, à valider Q1 spec validate). Le service `kesh-api` rejoint ce réseau. Aucun port host n'est exposé (`ports:` absent ou commenté).

4. **Given** `docker-compose.prod.yml`, **When** review, **Then** il **ne contient pas** de bind-mount `./crates:ro` ni `./frontend:ro` (anti-pattern prod — uniquement docker-compose dev).

5. **Given** `docker-compose.prod.yml`, **When** review, **Then** il définit `restart: unless-stopped` sur `kesh-api`.

6. **Given** `docker-compose.prod.yml`, **When** review, **Then** il définit `logging` avec `driver: json-file` et `options: { max-size: 10m, max-file: 5 }` sur `kesh-api` (rotation logs Docker — empêche saturation disque NAS sur usage long-terme).

7. **Given** `docker-compose.prod.yml`, **When** review, **Then** il définit `mem_limit: 1g` sur `kesh-api` (protection runaway sur NAS partagé).

8. **Given** `docker-compose.prod.yml`, **When** review, **Then** `KESH_HOST` est `127.0.0.1` par défaut (loopback — HTTP only LAN privé v0.1.0, décision D5 epic-10.md). Pas de `0.0.0.0` puisque pas de reverse proxy en v0.1.0.

9. **Given** `docker-compose.prod.yml`, **When** review, **Then** un healthcheck `curl -f http://localhost:3000/health` est défini (cohérent docker-compose.yml dev actuel).

### Fail-fast secrets dans `kesh-api/src/config.rs`

10. **Given** `KESH_JWT_SECRET` non défini dans l'environnement, **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::MissingVar("KESH_JWT_SECRET"))` et `kesh-api` exit avec code non-zero (comportement actuel ligne 376-377 — à **conserver**).

11. **Given** `KESH_JWT_SECRET` défini avec une longueur < 32 caractères, **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::WeakJwtSecret { actual_bytes })` (comportement actuel ligne 379-383 — à **conserver**).

12. **Given** `KESH_JWT_SECRET` contient la sous-chaîne `change-me`, **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::InsecureJwtSecret)` (nouveau variant) — **PAS** un `tracing::warn!` qui laisse passer (comportement actuel ligne 385-389 à **changer**).

13. **Given** `KESH_ADMIN_PASSWORD` non défini, **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::EmptyAdminPassword)` (default actuel `"changeme"` à **supprimer** ligne 358 — plus de fallback vers default insecure).

14. **Given** `KESH_ADMIN_PASSWORD` égal à `"changeme"` (ou n'importe quelle variante : `"Changeme"`, `"CHANGEME"`, leading/trailing whitespace), **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::InsecureAdminPassword)` (nouveau variant) — **PAS** un `tracing::warn!` (comportement actuel ligne 366-370 à **changer**). Vérification case-insensitive après trim.

15. **Given** `KESH_ADMIN_PASSWORD` défini avec une longueur < 12 caractères (après trim), **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::WeakAdminPassword { actual_chars })` (nouveau variant).

16. **Given** chacun des cas 12, 14, 15 ci-dessus, **When** le binaire `kesh-api` boot, **Then** exit avec un code non-zero **et** émet un log explicite via `tracing::error!` qui guide l'utilisateur (e.g. « FATAL: KESH_JWT_SECRET contains 'change-me' placeholder — generate a real secret via `openssl rand -hex 32` and update your .env file »).

17. **Given** la suite de tests `cargo test -p kesh-api --lib config::tests`, **When** exécutée, **Then** au moins **6 nouveaux tests** couvrent : (a) JWT secret vide, (b) JWT secret < 32, (c) JWT secret contenant `change-me`, (d) admin password absent/empty, (e) admin password `changeme` case-insensitive, (f) admin password < 12 chars. Et 0 régression sur les tests `config::tests` existants.

### `.env.example` (nouveau fichier racine)

18. **Given** la racine du repo, **When** `.env.example` est créé, **Then** il contient **toutes** les variables d'environnement documentées dans `crates/kesh-api/src/config.rs` (`KESH_PORT`, `KESH_HOST`, `KESH_ADMIN_USERNAME`, `KESH_ADMIN_PASSWORD`, `KESH_JWT_SECRET`, `KESH_JWT_EXPIRY_MINUTES`, `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS`, `KESH_REFRESH_INACTIVITY_MINUTES`, `KESH_RATE_LIMIT_*`, `RUST_LOG`, `DATABASE_URL`) avec un commentaire avant chaque variable expliquant son rôle.

19. **Given** `.env.example`, **When** review, **Then** **aucune** valeur par défaut insecure n'apparaît : `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>`, `KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>`, `DATABASE_URL=<EDIT: mysql://kesh:<password>@<mariadb-host>:3306/kesh>`. Pas de `changeme`, pas de `change-me-32-bytes...`.

20. **Given** `.env.example`, **When** un utilisateur copie le fichier en `.env` puis tente `docker compose -f docker-compose.prod.yml up` sans rien modifier, **Then** `kesh-api` exit immédiatement avec un message d'erreur identifiable (`InsecureJwtSecret` ou `InsecureAdminPassword` ou `MissingVar`). Le fail-fast est l'effet voulu — pas de boot accidentel sur defaults.

21. **Given** `.gitignore` du repo, **When** review, **Then** `.env` est listé (le fichier user-side ne doit jamais être committé). `.env.example` reste committé. *(à vérifier — si déjà présent, no-op)*

### Alignment MariaDB 10.11 (3 fichiers + 1 commentaire migration)

22. **Given** `docker-compose.yml:4` (dev compose racine), **When** review, **Then** `image: mariadb:10.11` (au lieu de `mariadb:11-jammy`).

23. **Given** `docker-compose.dev.yml:43`, **When** review, **Then** `image: mariadb:10.11` (au lieu de `mariadb:11.4`).

24. **Given** `.github/workflows/ci.yml:27`, **When** review, **Then** `image: mariadb:10.11` (au lieu de `mariadb:11.4`).

25. **Given** `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:28`, **When** review, **Then** le commentaire mentionne « Docker Compose pin `mariadb:10.11`, OK » (au lieu de `mariadb:11-jammy`). Le pré-requis `MariaDB ≥ 10.6` reste documenté (10.11 ≥ 10.6, donc cohérent).

26. **Given** la suite CI lancée après les modifs ci-dessus (sur la PR Story 10-1), **When** le job `Backend (Rust)` exécute `cargo test --workspace -j1 -- --test-threads=1` contre le service `mariadb:10.11`, **Then** tous les tests Rust passent (validation pratique compat MariaDB 10.11 — déjà confirmée par sanity check pre-flight 2026-05-21 sur DB vierge).

### Validation end-to-end

27. **Given** workflow complet `Test Locally First` (CLAUDE.md) sur la branche `chore/story-10-1-spec` ou `story/10-1-*`, **When** exécuté avant push, **Then** les 4 commandes Backend Rust passent (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) et les 4 commandes Frontend passent (`npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`).

28. **Given** `docker-compose.prod.yml` finalisé + `.env` user-side valide créé localement (avec `KESH_JWT_SECRET=$(openssl rand -hex 32)` + `KESH_ADMIN_PASSWORD=$(openssl rand -base64 24)` + `DATABASE_URL` pointant vers le `kesh-mariadb` actuel ou une MariaDB 10.11 ad-hoc), **When** `docker compose -f docker-compose.prod.yml up -d` localement, **Then** le container `kesh-api` démarre sans erreur + `curl http://127.0.0.1:3000/health` retourne 200 + body `{ status: "ok", db: true }`.

29. **And** 0 régression sur les baselines : 250+ tests Vitest + cargo test workspace + 76 Playwright E2E préservés.

30. **And** la PR Story 10-1 est mergée sur `main` (squash) avec `closes` aucune Issue GitHub (Story 10-1 n'adresse pas d'Issue spécifique — Issue #41 sera fermée par Story 10-5).

## Tasks / Subtasks

### T1: Fail-fast secrets dans `kesh-api/src/config.rs` (AC #10-17)

- [ ] T1.1 — Ajouter 3 nouveaux variants à `enum ConfigError` (dans `kesh-api/src/config.rs`) :
  - `InsecureJwtSecret` (avec impl Display message clair)
  - `InsecureAdminPassword` (avec impl Display message clair)
  - `WeakAdminPassword { actual_chars: usize }` (avec impl Display)
- [ ] T1.2 — Modifier ligne 358 `let admin_password = env::var("KESH_ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into())` → **supprimer le fallback** et utiliser `.map_err(|_| ConfigError::MissingVar("KESH_ADMIN_PASSWORD".into()))?` (ou similaire). Plus de default `"changeme"`.
- [ ] T1.3 — Modifier ligne 366-370 (warn `KESH_ADMIN_PASSWORD == "changeme"`) en `return Err(ConfigError::InsecureAdminPassword)` après comparaison case-insensitive sur la valeur trimée. Pattern recommandé : `if admin_password.trim().eq_ignore_ascii_case("changeme") { return Err(ConfigError::InsecureAdminPassword); }`.
- [ ] T1.4 — Ajouter check longueur après le check `eq_ignore_ascii_case` : `if admin_password.trim().chars().count() < 12 { return Err(ConfigError::WeakAdminPassword { actual_chars: admin_password.trim().chars().count() }); }`. Utiliser `.chars().count()` (pas `.len()`) pour compter les chars Unicode correctement.
- [ ] T1.5 — Modifier ligne 385-389 (warn `KESH_JWT_SECRET.contains("change-me")`) en `return Err(ConfigError::InsecureJwtSecret)`.
- [ ] T1.6 — Vérifier que `main.rs` propage bien le `ConfigError` en exit non-zero (pattern existant — `Config::from_env().map_err(|e| { tracing::error!(...); std::process::exit(1) })` ou équivalent). Si déjà en place pour les autres variants, no-op.
- [ ] T1.7 — Ajouter 6 tests unitaires dans le module `config::tests` :
  - `config_rejects_jwt_secret_containing_change_me`
  - `config_rejects_admin_password_changeme_lowercase`
  - `config_rejects_admin_password_changeme_uppercase`
  - `config_rejects_admin_password_changeme_with_whitespace`
  - `config_rejects_admin_password_short` (< 12 chars)
  - `config_accepts_admin_password_exactly_12_chars` (limite basse OK)

### T2: Création `docker-compose.prod.yml` (AC #1-9)

- [ ] T2.1 — Créer le fichier `docker-compose.prod.yml` à la racine du repo en s'inspirant de `docker-compose.yml` actuel (lignes 29-75 = service `kesh-api`) mais en retirant :
  - le bloc `depends_on: { mariadb: ... }` (ligne 36-38)
  - le bloc volume `./crates:/app/crates:ro` (ligne 67-68)
- [ ] T2.2 — Retirer le bloc `services: mariadb` entier (lignes 1-28 du compose actuel).
- [ ] T2.3 — Modifier `KESH_HOST: ${KESH_HOST:-127.0.0.1}` (au lieu de `0.0.0.0` — décision D5 LAN privé).
- [ ] T2.4 — Modifier `DATABASE_URL` pour pointer vers une env var sans default vers le nom de service interne : `DATABASE_URL: ${DATABASE_URL}` (l'utilisateur fournit la valeur exacte dans son `.env`, on ne préfixe pas avec un hostname interne `mariadb:3306`).
- [ ] T2.5 — Retirer le mapping de ports `3306:3306` (le service mariadb n'est plus interne).
- [ ] T2.6 — Conserver le port `3000:3000` pour kesh-api (à exposer pour le reverse proxy DSM futur OU pour l'accès direct LAN v0.1.0).
- [ ] T2.7 — Ajouter le bloc `networks: kesh-net` au service kesh-api + déclaration `networks: { kesh-net: { external: true } }` en bas du fichier.
- [ ] T2.8 — Ajouter le bloc `logging` (AC #6) :
  ```yaml
  logging:
    driver: json-file
    options:
      max-size: 10m
      max-file: 5
  ```
- [ ] T2.9 — Ajouter `mem_limit: 1g` au service kesh-api (AC #7).
- [ ] T2.10 — Conserver `restart: unless-stopped` + `healthcheck` (déjà présents dans le compose dev, à copier).
- [ ] T2.11 — Tester localement `docker compose -f docker-compose.prod.yml config` (validation YAML structurelle).

### T3: Création `.env.example` (AC #18-21)

- [ ] T3.1 — Créer `.env.example` à la racine du repo (pas dans un sous-dossier).
- [ ] T3.2 — Documenter chaque variable d'env consommée par `crates/kesh-api/src/config.rs` :
  - Section `# Base de données` : `DATABASE_URL=<EDIT: mysql://kesh:<password>@<mariadb-host>:3306/kesh>` avec commentaire « URL de connexion MariaDB 10.11+. Sur Synology DSM, pointer vers le hostname du service MariaDB sur le réseau Docker externe `kesh-net`. »
  - Section `# Application` : `KESH_PORT=3000`, `KESH_HOST=127.0.0.1` (commenter que `0.0.0.0` serait pour reverse proxy futur v0.2+).
  - Section `# Admin initial` : `KESH_ADMIN_USERNAME=admin`, `KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>` avec commentaire « Utilisé uniquement au tout premier boot avec table `users` vide. Le bootstrap est idempotent (cf. `bootstrap.rs:39-47`), donc une fois l'admin créé via UI, ces vars ne sont plus consultées. »
  - Section `# Authentification` : `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>` (32 chars hex = 16 bytes random) avec commentaire « Doit faire au moins 32 caractères et ne pas contenir 'change-me'. »
  - Section `# Session & rate limiting` : `KESH_JWT_EXPIRY_MINUTES=15`, `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS=30`, `KESH_REFRESH_INACTIVITY_MINUTES=15`, `KESH_RATE_LIMIT_WINDOW_MINUTES=15`, `KESH_RATE_LIMIT_MAX_ATTEMPTS=5`, `KESH_RATE_LIMIT_BLOCK_MINUTES=30`.
  - Section `# Logging` : `RUST_LOG=info`.
- [ ] T3.3 — Vérifier `.gitignore` (AC #21) : si `.env` y est déjà, no-op. Sinon ajouter la ligne `.env`.

### T4: Alignment MariaDB 10.11 (AC #22-26)

- [ ] T4.1 — Edit `docker-compose.yml:4` : `image: mariadb:11-jammy` → `image: mariadb:10.11`.
- [ ] T4.2 — Edit `docker-compose.dev.yml:43` : `image: mariadb:11.4` → `image: mariadb:10.11`.
- [ ] T4.3 — Edit `.github/workflows/ci.yml:27` : `image: mariadb:11.4` → `image: mariadb:10.11`.
- [ ] T4.4 — Edit `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:28` : `(Docker Compose pin `mariadb:11-jammy`, OK).` → `(Docker Compose pin `mariadb:10.11`, OK — requis MariaDB ≥ 10.6).`.
- [ ] T4.5 — Vérifier qu'il n'y a pas d'autres références hardcodées à `mariadb:11` ailleurs dans le repo : `grep -rn "mariadb:11\|mariadb-11" --include="*.yml" --include="*.yaml" --include="*.sql" --include="*.md"`. Update si trouvé (sauf archives historiques `_bmad-output/implementation-artifacts/*.md` qui peuvent rester non-modifiées — ce sont des artefacts de stories passées).

### T5: Validation end-to-end (AC #27-29)

- [ ] T5.1 — Exécuter `Test Locally First` complet (AC #27) :
  ```sh
  cargo fmt --all -- --check
  cargo build --workspace --all-targets
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cd frontend
  npm run check
  npm run lint-i18n-ownership
  npm run test:unit
  npm run build
  ```
- [ ] T5.2 — Tester `docker compose -f docker-compose.prod.yml up -d` localement avec un `.env` valide (AC #28). Vérifier `/health` 200. Vérifier que les logs respectent la rotation (créer un volume de logs verbeux et observer la taille).
- [ ] T5.3 — Vérifier que la CI passe sur la PR Story 10-1 — en particulier le job `Backend (Rust)` avec `mariadb:10.11` (AC #26).

## Dev Notes

### Module touché

- `crates/kesh-api/src/config.rs` (modification 1 module — fail-fast logic)
- `docker-compose.prod.yml` (nouveau fichier, infra)
- `docker-compose.yml`, `docker-compose.dev.yml` (modif ligne image MariaDB)
- `.github/workflows/ci.yml` (modif ligne image MariaDB)
- `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql` (modif commentaire ligne 28 — pas de modif SQL)
- `.env.example` (nouveau fichier)
- `.gitignore` (vérification ou ajout `.env`)

**Total modules** : 1 crate Rust (`kesh-api`) + 1 crate dépendante (`kesh-db` — commentaire seul) + 5 fichiers infra/CI = **2 crates / 7 fichiers**. Sous le seuil 5 modules de la règle de splitting CLAUDE.md.

### Pattern fail-fast `ConfigError` à suivre

Le code actuel `kesh-api/src/config.rs` ligne 376-389 montre déjà 3 patterns fail-fast en place pour `KESH_JWT_SECRET` :

```rust
let jwt_secret = env::var("KESH_JWT_SECRET")
    .map_err(|_| ConfigError::MissingVar("KESH_JWT_SECRET".into()))?;

if jwt_secret.len() < 32 {
    return Err(ConfigError::WeakJwtSecret { actual_bytes: jwt_secret.len() });
}

if jwt_secret.contains("change-me") {
    tracing::warn!("KESH_JWT_SECRET contient 'change-me' — générez ...");  // <-- À CHANGER en `return Err(...)`
}
```

→ Le 3e check devient `return Err(ConfigError::InsecureJwtSecret)`. Pattern strictement identique aux 2 premiers, juste remplacer le `warn!` par un `return`.

Idem pour `KESH_ADMIN_PASSWORD` ligne 358-370 :

```rust
let admin_password = env::var("KESH_ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into());  // <-- SUPPRIMER fallback

if admin_password.trim().is_empty() {
    return Err(ConfigError::EmptyAdminPassword);  // <-- CONSERVER
}

if admin_password == "changeme" {
    tracing::warn!("KESH_ADMIN_PASSWORD est 'changeme' — ...");  // <-- À CHANGER en `return Err(...)` case-insensitive trimmé
}
```

### Pattern test `config::tests` à suivre

Le module `config::tests` (`kesh-api/src/config.rs:670-880+`) utilise déjà :
- `make_test_config(admin_username, admin_password)` helper (`config.rs:674`) pour bypass env vars dans les tests.
- `serial_test` crate (probablement, à vérifier) pour éviter contention env var entre tests parallèles.
- Pattern de tests existant : `config_rejects_empty_admin_password` (ligne 882) — modèle à suivre pour les 6 nouveaux tests.

Avant de coder les nouveaux tests, lire le module `config::tests` complet pour reprendre exactement le helper `with_env_vars` ou équivalent (probablement) et reproduire le style. Les tests qui touchent à `env::var` partagent un état global → besoin de serialisation.

### Spec mariadb image tag

`mariadb:10.11` est un **rolling tag** qui pointe vers la dernière patch release de la série 10.11 (e.g. 10.11.11 actuel, 10.11.12 plus tard). Cohérent avec la version exacte du NAS de Guy (10.11.11) et offre les patches sécurité au fil de l'eau. **Ne pas pinner à 10.11.11 exact** — la stabilité de l'API 10.11.x est garantie par MariaDB upstream (politique semver-like sur les minor releases).

Pour la prod NAS, Guy utilise le Package Center DSM qui livre une version 10.11 spécifique mise à jour par Synology. Aucun lien avec le tag Docker — l'alignement Docker → DSM est sur la **série 10.11**, pas la version exacte.

### Convention compose dev vs prod

Convention adoptée dans cette story :
- `docker-compose.yml` (racine) = **dev compose** avec MariaDB bundled + bind-mount + port DB exposé. Reste l'outil de dev primaire pour les contributeurs.
- `docker-compose.prod.yml` (nouveau) = **prod compose** sans MariaDB bundled + sans bind-mount + sans port DB. Livrable release v0.1.0.
- `docker-compose.dev.yml` (déjà existant, ligne 43) = compose alternative dev encore utilisée par certaines workflows BMAD — à aligner aussi mais pas à fusionner.

Distinction `docker-compose.yml` vs `docker-compose.dev.yml` : à investiguer Story 10-1 spec validate (cohérence usage cross-team) → Q2 ci-dessous.

### Sécurité — pourquoi 12 chars minimum admin password

L'AC #15 impose 12 caractères minimum sur `KESH_ADMIN_PASSWORD`. Justification :
- OWASP minimum recommend = 12 chars (NIST SP 800-63B).
- `openssl rand -base64 24` produit 32 caractères ASCII URL-safe — bien au-delà.
- Cohérent avec `KESH_JWT_SECRET` qui impose 32 chars (mais le JWT secret est un secret machine, le password est un secret humain → seuil plus bas pour l'utilisabilité).

À ne **pas** confondre avec FR6 « politique de mot de passe utilisateur » qui s'applique aux users créés via UI (Story 1-7) — c'est une autre politique. Ici on parle uniquement de l'admin initial seedé via env.

### Pattern bootstrap.rs idempotent

Référencé dans epic-10.md décision D10 + AC #18 commentaire `.env.example` : `crates/kesh-api/src/auth/bootstrap.rs:39-47` lit `SELECT COUNT(*) FROM users` et **return early** si users existent. 4 tests prouvent l'idempotence : `bootstrap_creates_admin_on_empty_db`, `bootstrap_is_idempotent_on_repeated_calls`, `bootstrap_skips_if_users_already_exist`, `bootstrap_skips_silently_when_no_company_exists`.

→ Le `.env.example` peut documenter ce comportement pour rassurer l'utilisateur que laisser `KESH_ADMIN_PASSWORD` dans `.env` après le 1er boot ne réseed pas l'admin (cf. T3.2).

### Test Locally First obligatoire (CLAUDE.md)

Cette story touche `.github/workflows/ci.yml` ET `docker-compose.yml` → la CI **va re-runner** et utiliser `mariadb:10.11`. Si une migration ou un test ne passait que sur MariaDB 11 par accident, on le verra à la PR. Le sanity check pre-flight 2026-05-21 a déjà confirmé que les 26 migrations passent sur 10.11 vierge → confidence haute, mais le `cargo test --workspace` complet en CI est la vraie validation.

### Project Structure Notes

Story 10-1 ne crée pas de nouveau module Rust ni de nouvelle route HTTP. Les fichiers modifiés respectent la structure existante :
- `crates/kesh-api/src/config.rs` (module existant)
- Racine du repo pour `docker-compose.prod.yml` + `.env.example` (cohérent avec `docker-compose.yml` + `docker-compose.dev.yml` existants)
- `.github/workflows/ci.yml` (existant)
- `crates/kesh-db/migrations/*.sql` (modif commentaire d'une migration existante — pas de nouvelle migration créée)

Aucun conflit avec la structure unifiée du projet documentée dans `architecture.md` lignes 428-440 (qui prévoit `.env.example` mais ne l'a jamais créé).

## Questions ouvertes (spec validate)

| # | Question | Statut |
|---|---|---|
| Q1 | Nom du réseau Docker externe — `kesh-net` ? `kesh` ? `mariadb-net` ? À aligner avec ce que Guy a déjà créé sur son NAS. | Spec validate — Guy fournit le nom exact |
| Q2 | Distinction `docker-compose.yml` vs `docker-compose.dev.yml` — pourquoi 2 compose dev distincts ? Si l'un est obsolète, le supprimer dans cette story. Sinon, documenter quand utiliser lequel. | Spec validate — investigation usage |
| Q3 | `mem_limit` exact pour kesh-api : 1g suffit-il pour les cas extrêmes (export ZIP global 50+ MB en mémoire — Story 9-2b L3 buffered RAM dette v0.2) ? Sinon, 2g et noter en docs. | Spec validate — estimation |
| Q4 | Faut-il aussi ajouter un `cpus: 2.0` resource limit ? Le bénéfice est faible sur NAS Synology single-app, l'effort est trivial. Décision : à inclure ou pas selon préférence Guy. | Spec validate — décision Guy |
| Q5 | Story 10-5 (httpOnly tokens) ajoutera-t-elle des cookies qui requièrent un domaine spécifique dans le compose (e.g. `KESH_COOKIE_DOMAIN`) ? Si oui, prévoir un placeholder dans `.env.example`. Sinon, ajouter en Story 10-5 plutôt qu'ici. | Spec validate Story 10-5 — à arbitrer en amont |

## Dev Agent Record

### Agent Model Used

À renseigner par dev agent au début de l'implémentation.

### Debug Log References

### Completion Notes List

### File List

À renseigner par dev agent à la fin de l'implémentation. Liste prévisionnelle :

- `crates/kesh-api/src/config.rs` (M) — fail-fast secrets + 6 tests nouveaux
- `docker-compose.prod.yml` (A) — fichier nouveau
- `docker-compose.yml` (M) — image MariaDB 10.11
- `docker-compose.dev.yml` (M) — image MariaDB 10.11
- `.github/workflows/ci.yml` (M) — image MariaDB 10.11
- `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql` (M) — commentaire ligne 28
- `.env.example` (A) — fichier nouveau
- `.gitignore` (M si nécessaire) — ajout `.env` si absent

## References

- `_bmad-output/planning-artifacts/epic-10.md` — Story 10-1 ACs source (décisions D1-D11, périmètre, critères d'arrêt)
- `_bmad-output/planning-artifacts/prd.md:374-376` — FR1, FR2, FR3 (config par env, install < 15 min)
- `_bmad-output/planning-artifacts/prd.md:495` — FR77 (docker-compose 2 containers, reverse proxy optionnel — note : v0.1.0 = HTTP only LAN privé, D5)
- `_bmad-output/planning-artifacts/architecture.md:41` — image < 100 Mo, logs stdout/stderr (déjà respecté par Dockerfile actuel)
- `_bmad-output/planning-artifacts/architecture.md:51` — stack imposée MariaDB **10.6+** (confirmation officielle compat, cohérent décision D3 align 10.11)
- `_bmad-output/planning-artifacts/architecture.md:170,185,193` — convention « 2 containers kesh + mariadb » (ajusté en v0.1.0 : kesh seul dans le compose prod, mariadb externe NAS)
- `_bmad-output/planning-artifacts/architecture.md:252` — `tracing` crate stdout/stderr (logs Docker-friendly, à conserver)
- `_bmad-output/planning-artifacts/architecture.md:429` — `.env.example` mentionné dans la structure prévue (à créer cette story, n'existe pas à ce jour)
- `crates/kesh-api/src/config.rs:340-389` — code actuel de `Config::from_env()` à modifier (suppression default `changeme` ligne 358, conversion warn → error lignes 366-370 + 385-389)
- `crates/kesh-api/src/config.rs:674-882` — pattern tests `config::tests` à suivre (`make_test_config` helper + `config_rejects_empty_admin_password` modèle de référence)
- `crates/kesh-api/src/auth/bootstrap.rs:39-47` — idempotence bootstrap confirmée + 4 tests
- `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:27-28` — commentaire MariaDB version à ajuster
- `docker-compose.yml` (racine) — base de référence pour créer `docker-compose.prod.yml` (en retirant mariadb + bind-mount + port DB)
- `.github/workflows/ci.yml:27-40` — services CI MariaDB (à aligner)
- `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` — story précédente, pattern de référence (As-a/Scope/AC/Tasks/Dev Notes/References)
- CLAUDE.md §"Test Locally First" — 4 checks Backend Rust + 4 checks Frontend obligatoires avant push
- CLAUDE.md §"Règle de splitting préventif" — Story 10-1 sous le seuil 5 modules (audit cette story : 2 crates touchés)
- CLAUDE.md §"Issue Tracking Rule" — Story 10-1 ne ferme aucune Issue GitHub (pas de KF affectée, Issue #41 réservée Story 10-5)
- Memory `project-session-state-2026-05-20-end` — action items pré-Epic 10
- Decisions pre-flight Epic 10 (D1-D11) — voir `epic-10.md` §Décisions clés
