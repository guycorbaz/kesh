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

**Mise à jour `.env.example`** existant à la racine du repo (référencé dans `architecture.md:429`, fichier déjà tracké — 2.5 Ko avec des defaults insecure type `KESH_ADMIN_PASSWORD=changeme` ligne 20 et `KESH_JWT_SECRET=change-me-32-bytes-minimum-...` ligne 24). Remplacement des defaults insecure par des placeholders explicites `<GENERATE_ME: openssl rand -hex 32>` etc., ajout des variables manquantes (`KESH_LANG`, `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB`), commentaires pédagogiques homogénéisés. `KESH_TEST_MODE` **explicitement absent** avec avertissement. Le fichier reste committé (déjà tracké), `.env` reste ignoré (`.gitignore:14` + `.env.local:15` — vérifié pre-flight 2026-05-21).

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

3. **Given** `docker-compose.prod.yml`, **When** review, **Then** il déclare un réseau **externe** explicite : `networks: { frontend: { external: true } }` (nom `frontend` **confirmé** Pass 1 spec validate 2026-05-21 — réseau Docker existant sur le NAS de Guy, hébergeant l'interface externe). Le service `kesh-api` rejoint ce réseau (`networks: [frontend]`). Le port HTTP est exposé via `ports: "127.0.0.1:3000:3000"` (binding host-side restreint au loopback de l'hôte NAS — accès LAN privé via SSH tunnel v0.1.0, ou via reverse proxy DSM dans une future story v0.2+).

4. **Given** `docker-compose.prod.yml`, **When** review, **Then** il **ne contient pas** de bind-mount `./crates:ro` ni `./frontend:ro` (anti-pattern prod — uniquement docker-compose dev).

5. **Given** `docker-compose.prod.yml`, **When** review, **Then** il définit `restart: unless-stopped` sur `kesh-api`.

6. **Given** `docker-compose.prod.yml`, **When** review, **Then** il définit `logging` avec `driver: json-file` et `options: { max-size: 10m, max-file: 5 }` sur `kesh-api` (rotation logs Docker — empêche saturation disque NAS sur usage long-terme).

7. **Given** `docker-compose.prod.yml`, **When** review, **Then** il définit `mem_limit: 1g` sur `kesh-api` (protection runaway sur NAS partagé).

8. **Given** `docker-compose.prod.yml`, **When** review, **Then** `KESH_HOST=0.0.0.0` (bind interne au container — sans cela, le service Rust n'écouterait que sur la loopback du container et serait inaccessible depuis l'hôte NAS même avec port mapping correct). La restriction LAN privé v0.1.0 (décision D5) est appliquée **au niveau du port mapping host-side** via `ports: "127.0.0.1:3000:3000"` (AC #3), **pas au bind interne**. Cf. `docker-compose.dev.yml:17-23` qui documente ce pattern (loopback container vs loopback hôte).

9. **Given** `docker-compose.prod.yml`, **When** review, **Then** un healthcheck `curl -f http://localhost:3000/health` est défini (cohérent docker-compose.yml dev actuel).

### Fail-fast secrets dans `kesh-api/src/config.rs`

10. **Given** `KESH_JWT_SECRET` non défini dans l'environnement, **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::MissingVar("KESH_JWT_SECRET"))` et `kesh-api` exit avec code non-zero (comportement actuel ligne 376-377 — à **conserver**).

11. **Given** `KESH_JWT_SECRET` défini avec une longueur < 32 caractères, **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::WeakJwtSecret { actual_bytes })` (comportement actuel ligne 379-383 — à **conserver**).

12. **Given** `KESH_JWT_SECRET` contient la sous-chaîne `change-me`, **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::InsecureJwtSecret)` (nouveau variant) — **PAS** un `tracing::warn!` qui laisse passer (comportement actuel ligne 385-389 à **changer**).

13. **Given** `KESH_ADMIN_PASSWORD` non défini (variable d'env absente), **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::MissingVar("KESH_ADMIN_PASSWORD".into()))` — cohérent avec le pattern existant `KESH_JWT_SECRET` ligne 376-377 (default fallback `"changeme"` ligne 358 à **supprimer**). Le variant `EmptyAdminPassword` existant (ligne 362-364) reste utilisé pour le cas var définie mais composée de whitespace uniquement.

14. **Given** `KESH_ADMIN_PASSWORD` égal à `"changeme"` (ou n'importe quelle variante : `"Changeme"`, `"CHANGEME"`, leading/trailing whitespace), **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::InsecureAdminPassword)` (nouveau variant) — **PAS** un `tracing::warn!` (comportement actuel ligne 366-370 à **changer**). Vérification case-insensitive après trim.

15. **Given** `KESH_ADMIN_PASSWORD` défini avec une longueur < 12 caractères (après trim), **When** `Config::from_env()` est appelé, **Then** retourne `Err(ConfigError::WeakAdminPassword { actual_chars })` (nouveau variant).

16. **Given** chacun des cas 12, 14, 15 ci-dessus, **When** le binaire `kesh-api` boot, **Then** exit avec un code non-zero **et** émet un log explicite via `tracing::error!` qui guide l'utilisateur (e.g. « FATAL: KESH_JWT_SECRET contains 'change-me' placeholder — generate a real secret via `openssl rand -hex 32` and update your .env file »).

17. **Given** la suite de tests `cargo test -p kesh-api --lib config::tests`, **When** exécutée, **Then** au moins **4 nouveaux tests** couvrent les nouveaux cas : (a) JWT secret de **≥ 32 chars contenant `change-me`** (e.g. `"abcdefghij-change-me-abcdefghijabc"` 34 chars — la longueur évite que `WeakJwtSecret` ne court-circuite le check `InsecureJwtSecret`, l'ordre des checks ligne 379 puis 385 imposant que le secret passe d'abord le check longueur) → `InsecureJwtSecret`, (b) admin password `changeme` case-insensitive (lowercase + uppercase + variantes avec whitespace) → `InsecureAdminPassword`, (c) admin password < 12 chars → `WeakAdminPassword`, (d) admin password = 12 chars exactement → OK (limite basse). Les cas déjà couverts par les tests existants `config_rejects_missing_jwt_secret` (ligne 813), `config_rejects_weak_jwt_secret` (ligne 828), `config_rejects_empty_admin_password` (ligne 882), `config_rejects_whitespace_only_admin_password` (ligne 916) restent VERTS **après application des patches T1.2.1 (helper) + T1.2.2 (3 tests hors helper)** — la suppression du default fallback `"changeme"` (ligne 358) doit être validée par ces patches sinon `MissingVar("KESH_ADMIN_PASSWORD")` court-circuite les assertions originales. 0 régression sur la suite `config::tests` complète après application T1.2.1 + T1.2.2.

### `.env.example` (nouveau fichier racine)

18. **Given** la racine du repo, **When** `.env.example` est mis à jour (fichier existant — voir Scope), **Then** il documente **toutes** les variables d'environnement consommées par `crates/kesh-api/src/config.rs` : `DATABASE_URL`, `KESH_PORT`, `KESH_ADMIN_USERNAME`, `KESH_ADMIN_PASSWORD`, `KESH_JWT_SECRET`, `KESH_JWT_EXPIRY_MINUTES`, `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS`, `KESH_REFRESH_INACTIVITY_MINUTES`, `KESH_RATE_LIMIT_WINDOW_MINUTES`, `KESH_RATE_LIMIT_MAX_ATTEMPTS`, `KESH_RATE_LIMIT_BLOCK_MINUTES`, `KESH_LANG`, `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB`, `RUST_LOG` — **15 variables actives** définies + `KESH_HOST` **documentée en commentaire** (non-définie pour éviter override du compose prod, cf. T3.2 section Application ligne 162 + patch #13 Pass 2). Le `KESH_TEST_MODE` est aussi **explicitement absent** du `.env.example` avec un commentaire d'avertissement à la place : `# DO NOT SET KESH_TEST_MODE IN PRODUCTION — réservé exclusivement aux tests intégrés CI/dev. L'activation en prod désactive des garde-fous sécurité.`. Chaque variable active a un commentaire avant elle expliquant son rôle.

19. **Given** `.env.example`, **When** review, **Then** **aucune** valeur par défaut insecure n'apparaît : `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>`, `KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>`, `DATABASE_URL=<EDIT: mysql://kesh:<password>@<mariadb-host>:3306/kesh>`. Pas de `changeme`, pas de `change-me-32-bytes...`.

20. **Given** `.env.example`, **When** un utilisateur copie le fichier en `.env` puis tente `docker compose -f docker-compose.prod.yml up` sans rien modifier, **Then** `kesh-api` **ne démarre pas correctement** — soit un `ConfigError` fatal (si un placeholder rencontre un check existant, e.g. `KESH_ADMIN_PASSWORD` absent du `.env.example` → `MissingVar`), soit une erreur de connexion DB sur l'URL placeholder `<EDIT: mysql://...>` invalide. Le comportement voulu est : **aucun boot silencieux sur des valeurs non-fonctionnelles**. Pas obligatoirement un `ConfigError` spécifique (les placeholders `<GENERATE_ME: openssl rand -hex 32>` 35 chars passent `WeakJwtSecret`+`InsecureJwtSecret`, donc le fail-fast peut survenir en aval de Config — toujours avec exit non-zero et message log identifiable côté DB pool init).

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

- [ ] T1.1 — Ajouter 3 nouveaux variants à `enum ConfigError` (dans `kesh-api/src/config.rs`). **Sécurité IMPORTANTE** : les impl Display **ne loggent jamais la valeur du secret**, uniquement le pattern reconnu ou la longueur (pattern existant `WeakJwtSecret` ligne 43-50 montre `actual_bytes` count, pas la valeur). Le `tracing::error!(e.to_string())` consume le Display, donc toute valeur incluse fuiterait dans les logs Docker.
  - `InsecureJwtSecret` Display : « KESH_JWT_SECRET contient le placeholder `change-me`. Générer un vrai secret via : `openssl rand -hex 32` » — **pas** la valeur.
  - `InsecureAdminPassword` Display : « KESH_ADMIN_PASSWORD est la valeur par défaut `changeme` (case-insensitive). Changez-le avant la mise en production. » — **pas** la valeur (qui peut être `Changeme  ` avec whitespace).
  - `WeakAdminPassword { actual_chars: usize }` Display : « KESH_ADMIN_PASSWORD trop court : {actual_chars} chars, minimum 12. » — `actual_chars` est OK (juste le compte, pas le contenu).
- [ ] T1.2 — Modifier ligne 358 `let admin_password = env::var("KESH_ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into())` → **supprimer le fallback** et utiliser `.map_err(|_| ConfigError::MissingVar("KESH_ADMIN_PASSWORD".into()))?`. Plus de default `"changeme"`.
- [ ] T1.2.1 — **Mettre à jour `set_minimum_required()` helper de tests** (`crates/kesh-api/src/config.rs:740-745`) en ajoutant `env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");` dans le bloc `unsafe`. Sans cette mise à jour, les **24 tests** appelant `set_minimum_required()` (`grep -cE "^\s*set_minimum_required\(\);"` = 24) retournent `ConfigError::MissingVar("KESH_ADMIN_PASSWORD")` au lieu de `Ok(Config)` → régression garantie. Cette sous-tâche doit être exécutée **dans la même PR que T1.2**.
- [ ] T1.2.2 — **Patcher les ~3 tests hors helper** qui set manuellement leurs env vars sans passer par `set_minimum_required()` et qui n'incluent pas `KESH_ADMIN_PASSWORD` :
  - `config_rejects_missing_jwt_secret` (ligne 813) — ajouter `env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");` dans le bloc `unsafe` après la ligne 817. Sans ça : test fail car il attendait `MissingVar("KESH_JWT_SECRET")` mais reçoit `MissingVar("KESH_ADMIN_PASSWORD")` (ordre des checks `from_env()` post-T1.2 : DATABASE_URL → KESH_ADMIN_PASSWORD → KESH_JWT_SECRET, donc ADMIN_PASSWORD check intervient AVANT JWT check).
  - `config_rejects_weak_jwt_secret` (ligne 828) — même patch (ajout `KESH_ADMIN_PASSWORD`). Sans ça : test fail car il attendait `WeakJwtSecret` mais reçoit `MissingVar("KESH_ADMIN_PASSWORD")`.
  - `config_trims_admin_username` (ligne 899) — même patch. Sans ça : test fail `expect("should load")` panic car `from_env()` retourne `Err(MissingVar("KESH_ADMIN_PASSWORD"))`.
  - **Audit complémentaire recommandé** : exécuter `grep -nB2 "Config::from_env()" crates/kesh-api/src/config.rs` pour identifier tout autre test qui appelle `from_env()` sans set `KESH_ADMIN_PASSWORD` ni `set_minimum_required()`. Patcher tous les sites trouvés.
  - **Non affectés (à laisser intacts)** : `config_rejects_empty_admin_password` (882) et `config_rejects_whitespace_only_admin_password` (916) — ces tests set explicitement `KESH_ADMIN_PASSWORD=""` ou `"   "` → `env::var()` retourne `Ok("")` (pas `Err`), donc pas `MissingVar` mais bien `EmptyAdminPassword` qui est ce qu'ils attendent. ✓
- [ ] T1.3 — Modifier ligne 366-370 (warn `KESH_ADMIN_PASSWORD == "changeme"`) en `return Err(ConfigError::InsecureAdminPassword)` après comparaison case-insensitive sur la valeur trimée. Pattern recommandé : `if admin_password.trim().eq_ignore_ascii_case("changeme") { return Err(ConfigError::InsecureAdminPassword); }`.
- [ ] T1.4 — Ajouter check longueur après le check `eq_ignore_ascii_case` : `if admin_password.trim().chars().count() < 12 { return Err(ConfigError::WeakAdminPassword { actual_chars: admin_password.trim().chars().count() }); }`. Utiliser `.chars().count()` (pas `.len()`) pour compter les chars Unicode correctement.
- [ ] T1.5 — Modifier ligne 385-389 (warn `KESH_JWT_SECRET.contains("change-me")`) en `return Err(ConfigError::InsecureJwtSecret)`.
- [ ] T1.6 — Vérifier que `main.rs` propage bien le `ConfigError` en exit non-zero (pattern existant — `Config::from_env().map_err(|e| { tracing::error!(...); std::process::exit(1) })` ou équivalent). Si déjà en place pour les autres variants, no-op.
- [ ] T1.7 — Ajouter **4 nouveaux tests** unitaires dans le module `config::tests` (les cas « JWT empty/missing » et « admin password empty » sont déjà couverts par les tests existants `config_rejects_missing_jwt_secret` ligne 813, `config_rejects_weak_jwt_secret` ligne 828, `config_rejects_empty_admin_password` ligne 882, `config_rejects_whitespace_only_admin_password` ligne 916 — à patcher T1.2.2 pour les 2 premiers, à laisser intacts pour les 2 derniers) :
  - `config_rejects_jwt_secret_containing_change_me` — utiliser un secret de **≥ 32 chars** (e.g. `"abcdefghij-change-me-abcdefghijabc"` 34 chars) pour passer le check `WeakJwtSecret` ligne 379 et arriver au check `InsecureJwtSecret` ligne 385. Avec un secret < 32 chars comme `"change-me"` (10 chars), le test fail car il reçoit `WeakJwtSecret` au lieu de `InsecureJwtSecret`.
  - `config_rejects_admin_password_changeme_case_insensitive` (couvre lowercase + uppercase + leading/trailing whitespace en 1 test paramétrisé, OU 3 tests distincts)
  - `config_rejects_admin_password_short` (< 12 chars)
  - `config_accepts_admin_password_exactly_12_chars` (limite basse OK)

### T2: Création `docker-compose.prod.yml` (AC #1-9)

- [ ] T2.1 — Créer le fichier `docker-compose.prod.yml` à la racine du repo en s'inspirant de `docker-compose.yml` actuel (lignes 29-75 = service `kesh-api`) mais en retirant :
  - le bloc `depends_on: { mariadb: ... }` (ligne 36-38)
  - le bloc volume `./crates:/app/crates:ro` (ligne 67-68)
- [ ] T2.2 — Retirer le bloc `services: mariadb` entier (lignes 1-28 du compose actuel).
- [ ] T2.3 — Conserver `KESH_HOST: ${KESH_HOST:-0.0.0.0}` dans le compose prod (bind interne Docker — pattern identique compose dev). La restriction LAN privé v0.1.0 (D5) est appliquée via le port mapping host-side `127.0.0.1:3000:3000` (T2.6), pas au bind interne. Cf. `docker-compose.dev.yml:17-23` qui documente ce pattern.
- [ ] T2.4 — Modifier `DATABASE_URL` pour pointer vers une env var sans default vers le nom de service interne : `DATABASE_URL: ${DATABASE_URL}` (l'utilisateur fournit la valeur exacte dans son `.env`, on ne préfixe pas avec un hostname interne `mariadb:3306`).
- [ ] T2.5 — Note informative : le port 3306 (MariaDB) disparaît automatiquement avec la suppression du service mariadb en T2.2 — **aucune action requise** sur le service kesh-api (qui n'a pas et n'a jamais eu de port 3306 mappé). Cette sous-tâche est documentaire, ne pas chercher de ligne à éditer.
- [ ] T2.6 — Modifier le mapping ports de `"3000:3000"` en `"127.0.0.1:3000:3000"` (restriction host-side — port accessible uniquement via la loopback de l'hôte NAS, donc via SSH tunnel ou reverse proxy DSM local v0.2+). Décision D5 LAN privé v0.1.0 appliquée au niveau du port mapping host-side.
- [ ] T2.7 — Ajouter le bloc `networks: [frontend]` au service kesh-api + déclaration `networks: { frontend: { external: true } }` en bas du fichier. Nom `frontend` confirmé Pass 1 spec validate 2026-05-21 (réseau Docker existant NAS de Guy, hébergeant l'interface externe). Le service MariaDB Synology Package Center DSM doit être accessible depuis ce réseau (à valider via T5.2 smoke test — sinon `DATABASE_URL` devra utiliser l'IP LAN du NAS au lieu d'un hostname Docker).
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

- [ ] T3.1 — **Mettre à jour** le fichier `.env.example` existant à la racine du repo (**NE PAS** le recréer from scratch — partir du contenu actuel et patcher chirurgicalement). Au minimum, remplacer ligne 20 `KESH_ADMIN_PASSWORD=changeme` → `KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>` et ligne ~24 `KESH_JWT_SECRET=change-me-32-bytes-minimum-...` → `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>`. Vérifier ensuite l'exhaustivité des variables documentées (AC #18).
- [ ] T3.2 — Documenter chaque variable d'env consommée par `crates/kesh-api/src/config.rs` :
  - Section `# Base de données` : `DATABASE_URL=<EDIT: mysql://kesh:<password>@<mariadb-host>:3306/kesh>` avec commentaire « URL de connexion MariaDB 10.11+. Sur Synology DSM, pointer vers le hostname du service MariaDB sur le réseau Docker externe `frontend` (cf. AC #3 + T2.7). Si MariaDB tourne sur l'hôte NAS via Package Center DSM et que le réseau Docker n'a pas accès direct, utiliser l'IP LAN du NAS au lieu du hostname Docker. »
  - Section `# Application` : `KESH_PORT=3000`. **NE PAS définir `KESH_HOST=` dans `.env.example`** — le compose prod `docker-compose.prod.yml` force `KESH_HOST=0.0.0.0` (bind interne Docker requis sinon le container est inaccessible, cf. AC #8 + T2.3). Si l'utilisateur ajoute `KESH_HOST=127.0.0.1` dans son `.env` user-side, il override le compose prod et casse son install. Le default applicatif `127.0.0.1` du code (`config.rs:346`) reste valable pour exécution hors-container en dev local. **Action concrète T3.2** : laisser un commentaire pédagogique au lieu de la variable : `# KESH_HOST volontairement non défini ici — le compose prod le force à 0.0.0.0. Pour dev hors Docker, set explicitement KESH_HOST=127.0.0.1 dans votre .env local.`
  - Section `# Admin initial` : `KESH_ADMIN_USERNAME=admin`, `KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>` avec commentaire « Utilisé uniquement au tout premier boot avec table `users` vide. Le bootstrap est idempotent (cf. `bootstrap.rs:39-47`), donc une fois l'admin créé via UI, ces vars ne sont plus consultées. »
  - Section `# Authentification` : `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>` (`-hex 32` produit 32 bytes = 256 bits de random encodés en 64 caractères hex, bien au-delà du minimum 32 chars imposé) avec commentaire « Doit faire au moins 32 caractères et ne pas contenir 'change-me'. »
  - Section `# Session & rate limiting` : `KESH_JWT_EXPIRY_MINUTES=15`, `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS=30`, `KESH_REFRESH_INACTIVITY_MINUTES=15`, `KESH_RATE_LIMIT_WINDOW_MINUTES=15`, `KESH_RATE_LIMIT_MAX_ATTEMPTS=5`, `KESH_RATE_LIMIT_BLOCK_MINUTES=30`.
  - Section `# Logging` : `RUST_LOG=info`.
  - Section `# Localisation` : `KESH_LANG=fr` (locale par défaut interface — fr/de/it/en, FR75 PRD).
  - Section `# Politique mot de passe utilisateur` : `KESH_PASSWORD_MIN_LENGTH=8` (FR6 PRD — politique pour users créés via UI, distinct du `KESH_ADMIN_PASSWORD` 12 chars seedé via env).
  - Section `# Import bancaire` : `KESH_BANK_IMPORT_MAX_MB=10` (Story 8-1 limite upload bank statement).
  - Section `# Test mode (NE JAMAIS ACTIVER EN PRODUCTION)` : commenté/absent. Ajouter le bloc commentaire : `# DO NOT SET KESH_TEST_MODE IN PRODUCTION — réservé exclusivement aux tests intégrés CI/dev. L'activation en prod désactive des garde-fous sécurité (bind loopback, etc.).` Le commentaire est dans le fichier mais la variable n'est pas définie.
- [ ] T3.2.1 — **Traitement des variables legacy** déjà présentes dans `.env.example` mais hors des 16 listées AC #18 :
  - `MARIADB_ROOT_PASSWORD`, `MARIADB_DATABASE`, `MARIADB_USER`, `MARIADB_PASSWORD` (lignes 6-9 actuelles) — **CONSERVER** (utilisées par `docker-compose.yml` dev pour le service `mariadb` bundled qui reste l'outil dev primaire post-Story 10-1). Ajouter un commentaire de section : `# --- Service MariaDB bundled (DEV ONLY — docker-compose.yml + docker-compose.dev.yml) ---`. Section explicitement séparée des 16 variables Kesh + DATABASE_URL.
  - `COMPOSE_PROJECT_NAME` (ligne 49 actuelle) — **RETIRER** (low-value default Docker, pas utilisé par Kesh, juste un namespace cosmétique).
  - `KESH_PRODUCTION_RESET` (ligne 46 actuelle, commenté) — **CONSERVER** commenté (utilisé par `docker-compose.dev.yml:30` + `kesh-core/onboarding`, dette documentée KF-002). Section dev only.
- [ ] T3.3 — Vérifier `.gitignore` (AC #21) : `.env` ligne 14 + `.env.local` ligne 15 déjà présents — **aucune action requise** (vérifié pre-flight 2026-05-21 Pass 1).

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
- Helpers `env_lock()` + `reset_env()` (lignes 712-745) pour éviter contention env var entre tests parallèles (pattern projet — pas de crate `serial_test`).
- Helper `set_minimum_required()` (ligne 740-745) qui set actuellement `DATABASE_URL` + `KESH_JWT_SECRET`. À étendre par T1.2.1 pour ajouter `KESH_ADMIN_PASSWORD` (sinon ~23 tests cassent).
- Pattern de tests existant : `config_rejects_empty_admin_password` (ligne 882) + `config_rejects_whitespace_only_admin_password` (ligne 916) — modèles à suivre pour les 4 nouveaux tests.

Avant de coder les nouveaux tests, lire le module `config::tests` lignes 700-900 pour reprendre exactement le pattern :
```rust
let _guard = env_lock();
reset_env();
unsafe {
    env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
    env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
    env::set_var("KESH_ADMIN_PASSWORD", "<valeur testée>");
}
let result = Config::from_env();
assert!(matches!(result, Err(ConfigError::<variant_attendu>)));
```
Les tests qui touchent à `env::var` partagent un état global → la sérialisation via `env_lock()` est obligatoire.

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

### Audit complémentaire `KESH_ADMIN_PASSWORD` cross-crates (Pass 3 Opus insight #6)

Avant push (dans T5.1 Test Locally First), exécuter :

```sh
grep -rn "KESH_ADMIN_PASSWORD" crates --include="*.rs" 2>&1 | grep -v "src/config.rs"
```

→ identifier tout test d'intégration (`crates/*/tests/*.rs`) ou code source (`crates/*/src/*.rs` hors `config.rs`) qui set ou consume `KESH_ADMIN_PASSWORD` dans une valeur < 12 chars ou `"changeme"`. Si trouvé, patcher (mêmes patches que T1.2.2). Si rien trouvé : OK. Cet audit n'est pas couvert par T1.2.1 + T1.2.2 qui ciblent uniquement `kesh-api/src/config.rs`.

### Dette latente — compose dev `KESH_ADMIN_PASSWORD` défaut faible post-Story 10-1 (Pass 3 Opus insight #7 + Pass 4 Sonnet L-2)

**Deux fichiers compose dev affectés symétriquement** :
- `docker-compose.yml:49` (compose dev **primaire** pour les contributeurs) : `KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD:-changeme}`. Post-T1.3 (rejet case-insensitive `"changeme"`), tout dev qui lance `docker compose up` sans `.env` valide fera un fail-fast `InsecureAdminPassword`.
- `docker-compose.dev.yml:25` (compose dev alternatif) : `KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD:-admin}` (5 chars). Post-T1.4 (rejet `< 12 chars`), même symptôme — fail-fast `WeakAdminPassword`.

**Conséquence** : tout contributeur post-Story 10-1 qui lance un compose dev sans `.env` local valide rencontrera un fail-fast (au lieu d'un boot dev). C'est le comportement voulu par la sécurité globale, mais c'est une friction dev qu'il faut documenter.

**Pas une action Story 10-1** (les composes dev restent hors-scope du compose prod). À patcher dans une PR de suivi (e.g. story de cleanup post-Epic 10) en mettant `KESH_ADMIN_PASSWORD:-adminadminad12` (12+ chars dev-friendly) dans les **deux** composes — OU mieux, en documentant dans le `README.md` ou `CONTRIBUTING.md` que `.env` local doit être créé depuis `.env.example` avant tout `docker compose up`. La CI principale (`ci.yml`) ne lance pas le compose, donc pas de breakage CI.

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
| Q1 | ~~Nom du réseau Docker externe~~ — **RÉSOLU Pass 1 spec validate 2026-05-21** : nom = **`frontend`** (réseau Docker existant sur NAS de Guy, hébergeant l'interface externe). Figé dans AC #3, T2.7, scope. | — (résolu) |
| Q2 | Distinction `docker-compose.yml` vs `docker-compose.dev.yml` — pourquoi 2 compose dev distincts ? Si l'un est obsolète, le supprimer dans cette story. Sinon, documenter quand utiliser lequel. | Spec validate — investigation usage |
| Q3 | `mem_limit` exact pour kesh-api : 1g suffit-il pour les cas extrêmes (export ZIP global 50+ MB en mémoire — Story 9-2b L3 buffered RAM dette v0.2) ? Sinon, 2g et noter en docs. | Spec validate — estimation |
| Q4 | Faut-il aussi ajouter un `cpus: 2.0` resource limit ? Le bénéfice est faible sur NAS Synology single-app, l'effort est trivial. Décision : à inclure ou pas selon préférence Guy. | Spec validate — décision Guy |
| Q5 | Story 10-5 (httpOnly tokens) ajoutera-t-elle des cookies qui requièrent un domaine spécifique dans le compose (e.g. `KESH_COOKIE_DOMAIN`) ? Si oui, prévoir un placeholder dans `.env.example`. Sinon, ajouter en Story 10-5 plutôt qu'ici. | Spec validate Story 10-5 — à arbitrer en amont |

## Spec Validate Cycle

Cycle de revue adversariale CLAUDE.md §"Review Iteration Rule" — relance jusqu'à 0 finding > MEDIUM ou 8 passes.

### Pass 1 — Sonnet 4.6 (2026-05-21)

**Trend** : 11 findings bruts → 11 patches appliqués (3 CRITICAL + 2 HIGH + 4 MEDIUM + 2 LOW).

**Tous ground-truthed** par grep/Read avant application (CLAUDE.md §"Haiku-specific guardrails — grep ground-truth obligatoire", appliqué par hygiène défensive même sur Sonnet).

| # | Sév | Cat | Résumé | Patch |
|---|---|---|---|---|
| 1 | CRITICAL | REGR | `set_minimum_required()` ligne 740-745 ne set pas `KESH_ADMIN_PASSWORD`, ~23 tests cassent si fallback `"changeme"` supprimé | T1.2.1 ajoutée |
| 2 | CRITICAL | SPEC | AC #13 disait `EmptyAdminPassword`, T1.2 disait `MissingVar` (contradiction interne) | AC #13 unifié sur `MissingVar` (cohérent JWT pattern) |
| 3 | CRITICAL | SPEC | `KESH_HOST=127.0.0.1` dans container = inaccessible depuis hôte NAS même avec port mapping | AC #8 + T2.3 corrigés : `KESH_HOST=0.0.0.0` + ports `127.0.0.1:3000:3000` (restriction host-side) |
| 4 | HIGH | SPEC | AC #3 « ports absent » contredisait T2.6 « conserver 3000:3000 » | AC #3 + T2.6 alignés sur `ports: "127.0.0.1:3000:3000"` |
| 5 | HIGH | REINV | `.env.example` existe déjà (`changeme` ligne 20, `change-me-32-bytes-...` ligne 24) — story disait « créer » | Scope + T3.1 reformulés « mettre à jour » + T3.3 vérifié `.env` ligne 14 `.gitignore` |
| 6 | MEDIUM | SPEC | AC #18 omettait `KESH_LANG`, `KESH_TEST_MODE`, `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB` | AC #18 + T3.2 enrichis (16 vars + warning KESH_TEST_MODE) |
| 7 | MEDIUM | SPEC | Q1 nom réseau Docker non-résolu en status ready-for-dev | Résolu : nom = `frontend` (confirmé Guy 2026-05-21) — figé partout |
| 8 | MEDIUM | TEST | T1.7 listait 6 tests dont 2 existent déjà (`weak_jwt_secret` ligne 828, `empty_admin_password` ligne 882) | T1.7 + AC #17 réduits à 4 nouveaux tests, mention tests existants |
| 9 | MEDIUM | SPEC | T3.2 disait « `openssl rand -hex 32` = 32 chars hex = 16 bytes random » — faux | Corrigé : 64 chars hex = 32 bytes = 256 bits random |
| 10 | LOW | DOCS | Dev Notes mentionnait helper `with_env_vars` qui n'existe pas | Remplacé par pattern réel `env_lock() + reset_env() + env::set_var(...)` avec exemple code |
| 11 | LOW | DOCS | T2.5 redondant (port 3306 appartenait à service mariadb retiré par T2.2) | Reformulé en note documentaire (no-op action) |

**Critère d'arrêt CLAUDE.md** : 3 CRITICAL + 2 HIGH + 4 MEDIUM > LOW → relancer Pass 2 obligatoire.

### Pass 2 — Haiku 4.5 (2026-05-21)

**Trend** : 2 findings bruts → 2 patches appliqués (2 CRITICAL [REGR] = patches Pass 1 incomplets sur 2 emplacements de T3.2).

**Ground-truth check** : `grep -nF` exécuté sur les 2 claims CRITICAL Haiku → **0 hallucination détectée**, les 2 findings sont VALIDES (vrais oublis Sonnet Pass 1 patches #7 et patch #3 qui n'ont pas propagé à T3.2). Haiku a fait son job correctement, pas de pattern `feedback_haiku_review_diff_combined` (faux-positifs Haiku) cette fois.

| # | Sév | Cat | Résumé | Patch |
|---|---|---|---|---|
| 12 | CRITICAL | REGR | T3.2 ligne 155 commentaire DATABASE_URL référence encore `kesh-net` (oubli propagation patch #7) | Ligne 155 mise à jour : `frontend` + note alternative IP LAN NAS si réseau Docker n'a pas accès direct à MariaDB Package Center hôte |
| 13 | CRITICAL | REGR | T3.2 ligne 156 `.env.example` proposait `KESH_HOST=127.0.0.1` qui aurait override le `0.0.0.0` du compose prod (AC #8) → container inaccessible | Ligne 156 reformulée : `KESH_HOST` **non défini** dans `.env.example` (compose prod le force à `0.0.0.0`), commentaire pédagogique expliquant le contexte dev hors-container |

**Critère d'arrêt CLAUDE.md** : 2 CRITICAL + 0 HIGH + 0 MEDIUM > LOW → relancer Pass 3 obligatoire.

### Pass 3 — Opus 4.7 (2026-05-21)

**Trend** : 6 findings bruts → 5 patches appliqués + 1 finding déféré (5 LOW #19 drift `epic-10.md` `kesh-net` à propager post-merge, non-bloquant Story 10-1 per Opus recommendation).

**Ground-truth check** : `grep -nE` + `Read` direct des 3 tests CRITICAL #14 (Opus) ainsi que count `set_minimum_required` → **0 hallucination**. Opus a identifié un pattern transverse que Sonnet+Haiku ont raté (cf. CLAUDE.md §"Review Iteration Rule" : « Pour autant, la discipline grep ground-truth s'applique à tous les modèles par hygiène » — appliqué Pass 3 par défense en profondeur).

| # | Sév | Cat | Résumé | Patch |
|---|---|---|---|---|
| 14 | CRITICAL | REGR | T1.2.1 (Pass 1) couvre `set_minimum_required()` helper mais ~3 tests hors helper (`config_rejects_missing_jwt_secret` 813, `config_rejects_weak_jwt_secret` 828, `config_trims_admin_username` 899) set leurs env vars manuellement sans `KESH_ADMIN_PASSWORD` → fail post-T1.2. Sonnet+Haiku ratés. | T1.2.2 ajoutée : patch des 3 tests + audit complémentaire `grep -nB2 "Config::from_env()"` |
| 15 | MEDIUM | SPEC | T1.2.1 + AC #17 disaient « ~23 tests » — réel 24 (`grep -c "^\s*set_minimum_required\(\);"` = 24) | Texte corrigé en « 24 tests » |
| 16 | MEDIUM | SPEC | AC #17 + T1.7 cas (a) « JWT contenant `change-me` » ambigu — sans préciser longueur ≥ 32, le test peut catcher `WeakJwtSecret` au lieu de `InsecureJwtSecret` | AC #17 + T1.7 précisent ≥ 32 chars + donnent exemple concret `"abcdefghij-change-me-abcdefghijabc"` (34 chars) |
| 17 | MEDIUM | SPEC | `.env.example` contient `MARIADB_*` + `COMPOSE_PROJECT_NAME` + `KESH_PRODUCTION_RESET` hors des 16 listées AC #18 → choix arbitraire dev | T3.2.1 ajoutée : conserver `MARIADB_*` (compose dev), retirer `COMPOSE_PROJECT_NAME`, conserver `KESH_PRODUCTION_RESET` commenté |
| 18 | LOW | SEC | T1.1 ne précise pas que les Display ne loggent PAS la valeur du secret (risque fuite dans logs Docker) | T1.1 reformulée avec 3 exemples Display sans la valeur + rappel `tracing::error!(e.to_string())` consume Display |
| 19 | LOW | DOCS | `epic-10.md` mentionne encore `kesh-net` 5 fois (drift post-décision `frontend` Pass 1) — non-bloquant Story 10-1 | **DÉFÉRÉ** post-merge Story 10-1 vers Story 10-4 (manuel install qui rééditera la section) ou PR séparée. Per Opus recommendation |

**Analyse architecturale transverse (Opus-specific value)** :
- 4 patterns sains observés : fail-fast cohérent ConfigError, env_lock+reset_env pattern stable, bind loopback host-side D5 cohérent compose dev, idempotence bootstrap admin confirmée.
- 4 patterns douteux signalés : test `config_debug_hides_secrets` OK post-T1.5 (vérifié), audit `KESH_ADMIN_PASSWORD` cross-crates (insight #6, ajouté Dev Notes), dette latente `docker-compose.dev.yml:25` (insight #7, ajouté Dev Notes), `.env.example:24` actuel `KESH_JWT_SECRET=change-me-...` comportement fail-fast attendu post-T1.5 ✓.

**Critère d'arrêt CLAUDE.md** : 1 CRITICAL + 3 MEDIUM + 2 LOW (dont 1 déféré) > LOW → relancer Pass 4 obligatoire.

### Pass 4 — Sonnet 4.6 (2026-05-21)

**Trend** : 3 findings bruts → 3 patches appliqués (1 MEDIUM + 2 LOW). **Quasi-convergence**.

**Tous les 18 patches Pass 1+2+3 vérifiés** par Sonnet via grep+Read direct → **0 régression détectée**, tous OK. Tableau verdict complet dans le rapport Pass 4.

| # | Sév | Cat | Résumé | Patch |
|---|---|---|---|---|
| 20 | MEDIUM | SPEC | AC #20 promettait `ConfigError` (`InsecureJwtSecret`/`InsecureAdminPassword`/`MissingVar`) sur copie `.env.example` → `.env` sans modif. Mais placeholders `<GENERATE_ME: openssl rand -hex 32>` (35 chars, sans "change-me") passent les checks → exit non-zero via DB error mais pas via `ConfigError`. Tension architecturale AC #18 / #19 / #20 mutuellement exclusives. | AC #20 reformulé en Option A Sonnet : « ne démarre pas correctement » (soit `ConfigError`, soit DB error sur URL placeholder invalide). Objectif = aucun boot silencieux, pas exigence spécifique du variant. |
| 21 | LOW | SPEC | AC #18 disait "16 variables" mais T3.2 (patch #13 Pass 2) a retiré `KESH_HOST` du `.env.example` → 15 actives + 1 commentée | AC #18 reformulé en "15 variables actives + `KESH_HOST` documentée en commentaire" |
| 22 | LOW | DOCS | Dev Notes ne mentionnait que `docker-compose.dev.yml:25` (alt) mais oubliait `docker-compose.yml:49` (compose dev primaire) — même problème de défaut faible | Section « Dette latente » enrichie pour couvrir symétriquement les **2 composes dev**, avec recommandation patch + alternative `CONTRIBUTING.md` |

**Critère d'arrêt CLAUDE.md** : 0 CRITICAL + 0 HIGH + 0 MEDIUM post-patches Pass 4 (M-1 traité = MEDIUM résolu, L-1 + L-2 = LOW traités). **Probable convergence**.

### Pass 5 — Haiku 4.5 (2026-05-21) — CONVERGED ✅

**Trend** : **0 findings** (0 CRITICAL + 0 HIGH + 0 MEDIUM + 0 LOW).

**Méthode Haiku** : analyse adversariale de cohérence interne (AC ↔ Tasks ↔ Dev Notes) + comptage variables + verdict sur les 21 patches cumulés Pass 1-4. Aucun grep ground-truth nécessaire (pas de claim CRITICAL/HIGH affirmant existence/absence) — discipline `feedback_haiku_review_diff_combined` respectée par défaut (rien à hallucinationner sur 0 findings).

**Verdict détaillé Haiku** :
- AC #3 (réseau `frontend` external) ↔ T2.7 : ✅ alignés
- AC #8 (`KESH_HOST=0.0.0.0`) ↔ T2.3 + T2.6 : ✅ cohérents
- AC #13 (`MissingVar` pattern) ↔ T1.2 : ✅ unifié
- AC #17 + T1.7 (4 nouveaux tests) ↔ T1.2.1 + T1.2.2 : ✅ couverture complète tests existants + nouveaux
- AC #18 (15 variables actives + KESH_HOST commentée) ↔ T3.2 : ✅ post-patch #21 Pass 4
- AC #20 (fail-fast Option A) : ✅ tension architecturale résolue par patch #20 Pass 4
- Variables `.env.example` (T3.2 + T3.2.1) : ✅ 15 actives comptées, MARIADB_* conservées, COMPOSE_PROJECT_NAME retirée, KESH_PRODUCTION_RESET commentée
- T1.1 Display sans log valeur secret : ✅
- T4 MariaDB 10.11 alignement 4 fichiers : ✅ + audit grep T4.5
- Dev Notes dette latente symétrique composes dev : ✅

**Critère d'arrêt CLAUDE.md** : **0 findings > LOW → CONVERGED**. Cycle arrêté à Pass 5/8.

---

## 🎯 Cycle spec validate Story 10-1 — Bilan final

**Statut** : **CONVERGED** ✅ (Pass 5 Haiku 4.5, 2026-05-21).

**Métrique cycle** : **5 passes**, **21 patches appliqués + 1 déféré = 22 findings résolus**.

**Trend cumul** :
| Pass | LLM | Findings bruts | Patches appliqués | Délta |
|---|---|---|---|---|
| 1 | Sonnet 4.6 | 11 | 11 (3 CRIT + 2 HIGH + 4 MED + 2 LOW) | +11 |
| 2 | Haiku 4.5 | 2 | 2 (2 CRIT REGR) | +2 |
| 3 | Opus 4.7 | 6 | 5 + 1 déféré (1 CRIT + 3 MED + 1 LOW + 1 LOW déféré) | +5 |
| 4 | Sonnet 4.6 | 3 | 3 (1 MED + 2 LOW) | +3 |
| 5 | Haiku 4.5 | **0** | 0 | — |

**Insights LLM rotation** :
- **Sonnet 4.6 Pass 1** a posé le périmètre et identifié 11 findings cross-spec (excellent baseline).
- **Haiku 4.5 Pass 2** a catché 2 régressions par grep ground-truth strict — propagation Pass 1 patch #7 et patch #13 incomplètes sur T3.2 (zone éloignée des modifs Pass 1 immédiates).
- **Opus 4.7 Pass 3** a apporté une valeur transverse unique (CRITICAL #14 — 3 tests hors helper que Sonnet+Haiku n'ont pas catché), confirmant Insight I1 retro Epic 9 (Opus catches transverse patterns).
- **Sonnet 4.6 Pass 4** a clôt la tension architecturale AC #18/#19/#20 (patch #20) que les passes précédentes n'avaient pas vue.
- **Haiku 4.5 Pass 5** a confirmé la cohérence interne (re-cycle iter 2) avec 0 finding.

**1 finding déféré** :
- LOW #19 (Pass 3 Opus) — drift `epic-10.md` mentionne `kesh-net` 5 fois (à propager vers `frontend`). Non-bloquant Story 10-1. **À adresser** dans Story 10-4 (manuel install Synology qui rééditera la section « Réseau Docker externe »), OU PR de cohérence séparée post-merge Story 10-1.

**Story 10-1 prête pour `bmad-dev-story 10-1`** ✅ — l'implémenteur Rust+Docker peut démarrer avec confiance.

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
