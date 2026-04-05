# kesh-api

Serveur HTTP Axum de Kesh. Expose l'API REST `/api/v1/*`, le healthcheck
`/health`, et sert le frontend SvelteKit en SPA via `ServeDir`.

## Structure

```
src/
├── main.rs          # Point d'entrée : logging, pool, migrations, bootstrap, serve
├── lib.rs           # AppState + build_router (exposé aux tests d'intégration)
├── config.rs        # Config::from_env + validation + Debug masquant les secrets
├── errors.rs        # AppError + IntoResponse exhaustif vers JSON structuré
├── auth/
│   ├── password.rs  # Argon2id hash/verify + dummy_verify (timing-attack)
│   ├── jwt.rs       # Claims HS256 + encode/decode (leeway 60s)
│   └── bootstrap.rs # ensure_admin_user (FR3 : admin auto au démarrage)
├── middleware/
│   └── auth.rs      # require_auth (from_fn_with_state) + CurrentUser
└── routes/
    ├── health.rs    # GET /health (DB check)
    └── auth.rs      # POST /api/v1/auth/{login,logout}
tests/
└── auth_e2e.rs      # Tests E2E spawn_app avec reqwest
```

## Variables d'environnement

| Variable | Obligatoire ? | Défaut | Description |
|---|---|---|---|
| `DATABASE_URL` | **oui** | — | URL MariaDB, format `mysql://user:pass@host:port/db` |
| `KESH_JWT_SECRET` | **oui** | — | Clé HS256, **≥ 32 bytes**. Pas de valeur par défaut. |
| `KESH_PORT` | non | `3000` | Port HTTP |
| `KESH_HOST` | non | `0.0.0.0` | Interface d'écoute |
| `KESH_ADMIN_USERNAME` | non | `admin` | Username du compte admin bootstrap (FR3) |
| `KESH_ADMIN_PASSWORD` | non | `changeme` | Mot de passe admin bootstrap (logué en warning s'il vaut `changeme`) |
| `KESH_JWT_EXPIRY_MINUTES` | non | `15` | Durée de vie de l'access token, borné `[1, 1440]` |
| `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS` | non | `30` | Lifetime absolu du refresh token, borné `[1, 365]` |
| `KESH_STATIC_DIR` | non | `frontend/build` | Répertoire du SPA SvelteKit buildé |
| `RUST_LOG` | non | `info` | Filtre `tracing_subscriber::EnvFilter` |

### Génération d'un `KESH_JWT_SECRET` valide

```bash
openssl rand -hex 32
```

Produit une chaîne hex de 64 caractères = 32 bytes d'entropie. **Ne jamais
committer un vrai secret dans le repo.** Le fichier `.env.example` contient
une valeur factice `change-me-32-bytes-minimum-secret-generate-with-openssl-rand-hex-32`
qui est explicitement rejetée au démarrage via un warning.

### Comportement au démarrage (story 1.5)

1. **Logging** initialisé (`tracing_subscriber` + `EnvFilter`).
2. **Config** chargée depuis l'environnement. Erreur fatale (`exit 1`) si :
   - `DATABASE_URL` manquante
   - `KESH_JWT_SECRET` manquante
   - `KESH_JWT_SECRET` < 32 bytes (`ConfigError::WeakJwtSecret`)
3. **Pool MariaDB** créé. **Erreur fatale si la DB est indisponible** — l'authentification ne peut pas fonctionner sans DB (revirement partiel de la story 1.2 qui démarrait en mode dégradé).
4. **Migrations** appliquées via `kesh_db::MIGRATOR.run(&pool)`. Cela inclut automatiquement toutes les migrations du crate `kesh-db`.
5. **Bootstrap admin** (`ensure_admin_user`) : si la table `users` est vide, un compte admin est créé à partir de `KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD`. Idempotent, tolérant aux races (voir Dev Notes story 1.5).
6. **Serveur** bind sur `{KESH_HOST}:{KESH_PORT}` et sert les routes.

Après démarrage, le healthcheck `/health` continue de retourner 503 en cas
de perte de connexion DB — seule la phase de démarrage est stricte.

## API

### `POST /api/v1/auth/login`

```json
// Request
{ "username": "admin", "password": "changeme" }

// Response 200
{
  "accessToken": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "refreshToken": "0a0f5c91-3df7-4d11-b661-ced79f0fa9ec",
  "expiresIn": 900
}
```

Erreurs :
- `400 VALIDATION_ERROR` — username/password vide
- `401 INVALID_CREDENTIALS` — user inconnu, mot de passe incorrect, ou compte inactif (même code par design anti-enumeration)

### `POST /api/v1/auth/logout`

```json
// Request
{ "refreshToken": "0a0f5c91-..." }
```

Réponse : `204 No Content` **dans tous les cas** (idempotent). N'exige pas
de JWT valide — un client avec un access token expiré doit pouvoir
invalider sa session.

### `GET /health`

Réponse :
- `200 { "status": "ok", "database": "connected" }` — DB joignable
- `503 { "status": "degraded", "database": "disconnected" }` — DB perdue après démarrage

## Authentification — flux

- **Access token** : JWT HS256, `sub` (user_id String), `role`, `iat`, `exp`. Durée 15 min par défaut. `leeway = 60s` au decode pour absorber le clock drift NTP.
- **Refresh token** : UUID v4 opaque persisté en base (`refresh_tokens`). Lifetime absolu 30 jours par défaut. La sliding expiration « 15 min d'inactivité » sera ajoutée en story 1.6.
- **Middleware `require_auth`** : pattern Axum 0.8 `from_fn_with_state` qui décode le JWT et injecte un `CurrentUser { user_id, role }` dans les `Extensions` de la requête. Les handlers protégés le récupèrent via `Extension<CurrentUser>`.
- **Bootstrap admin** : au premier démarrage sur une DB vide, un compte `KESH_ADMIN_USERNAME` avec mot de passe `KESH_ADMIN_PASSWORD` est créé. Log explicite « CHANGEZ LE MOT DE PASSE ».

## Tests

```bash
# Tests unitaires + intégration (nécessite MariaDB pour les tests `#[sqlx::test]`)
DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh' cargo test -p kesh-api

# Tests E2E uniquement
DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3306/kesh' cargo test -p kesh-api --test auth_e2e
```

Voir `crates/kesh-db/README.md` pour les prérequis DB (privilèges `CREATE DATABASE` nécessaires pour `#[sqlx::test]`) et la section « Flakiness connue » sur les tests workspace parallèles.

## Dettes techniques documentées (story 1.5)

1. **Argon2 sync dans les handlers async** — blocage des workers tokio ~50 ms par login. À wrapper dans `tokio::task::spawn_blocking` post-MVP. Le rate limiting de la story 1.6 est un prérequis dur avant toute mise en production.
2. **`refresh_tokens.token` stocké en clair** — remplacement par `SHA-256(token)` + rotation à chaque refresh planifié pour la story 1.6.
3. **Migrations au démarrage** — `MIGRATOR.run()` seulement, la détection de version/rollback sophistiquée reste pour la story 8.2.
