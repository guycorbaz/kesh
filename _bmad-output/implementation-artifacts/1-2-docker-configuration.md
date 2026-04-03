# Story 1.2 : Docker & configuration

Status: ready-for-dev

## Story

**As a** administrateur,
**I want** installer Kesh via docker-compose en une commande,
**So that** l'application soit opérationnelle en moins de 15 minutes.

## Critères d'acceptation

1. **Given** docker-compose.dev.yml avec 2 containers (kesh + mariadb), **When** docker-compose up, **Then** les 2 containers démarrent et le healthcheck passe au vert
2. **Given** .env.example documenté, **When** l'admin copie .env.example vers .env et configure KESH_ADMIN_PASSWORD et KESH_PORT, **Then** l'application démarre avec ces valeurs
3. **Given** Dockerfile multi-stage (build Rust + build Svelte → image finale), **When** docker build, **Then** l'image résultante pèse moins de 100 Mo
4. **Given** application démarrée, **When** GET /health, **Then** réponse 200 avec statut de connexion DB
5. **And** les logs Docker affichent l'URL de l'application au démarrage

## Tâches / Sous-tâches

### Tâche 1 : Serveur Axum minimal avec healthcheck (AC-4, AC-5)

1.1. Ajouter les dépendances dans `crates/kesh-api/Cargo.toml` :
- `axum = "0.8"`
- `tokio = { version = "1", features = ["full"] }`
- `tower-http = { version = "0.6", features = ["fs", "cors", "trace"] }`
- `dotenvy = "0.15"`
- `sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "mysql"] }`
- `tracing = "0.1"`
- `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`
- `serde = { version = "1", features = ["derive"] }`
- `serde_json = "1"`

1.2. Créer `crates/kesh-api/src/config.rs` :
- Struct `Config` avec champs : `database_url`, `port`, `admin_username`, `admin_password`
- Chargement via `dotenvy::dotenv().ok()` puis `std::env::var()`
- Valeurs par défaut : `KESH_PORT=3000`
- Validation : `DATABASE_URL` obligatoire, log warning si `KESH_ADMIN_PASSWORD` = "changeme"

1.3. Créer `crates/kesh-api/src/routes/health.rs` :
- Route `GET /health`
- Réponse JSON : `{ "status": "ok", "database": "connected" }` (200) ou `{ "status": "degraded", "database": "disconnected", "error": "..." }` (503)
- Le healthcheck tente un `SELECT 1` via le pool SQLx
- Si pas de pool (DB inaccessible au démarrage), retourner 503

Structure des modules dans kesh-api/src/ :
- main.rs : point d'entrée, déclare `mod config; mod routes;`
- config.rs : struct Config + ConfigError + from_env()
- routes/mod.rs : déclare `pub mod health;`, fonction router()
- routes/health.rs : handler healthcheck

1.4. Réécrire `crates/kesh-api/src/main.rs` :
- Charger la configuration via `config.rs`
- Initialiser `tracing_subscriber` avec filtre `RUST_LOG` (défaut : `info`)
- Créer le pool SQLx MariaDB (avec gestion d'erreur gracieuse si DB indisponible)
- Construire le routeur Axum : routes `/health` + fallback ServeDir (prêt pour le SPA)
- Bind sur `0.0.0.0:{KESH_PORT}`
- Logger l'URL au démarrage : `tracing::info!("Kesh démarré sur http://0.0.0.0:{port}")`

1.5. Créer `crates/kesh-api/src/routes/mod.rs` :
- `pub mod health;`

1.6. Vérifier : `cargo build -p kesh-api` compile sans erreur

### Tâche 2 : Configuration .env (AC-2)

2.1. Mettre à jour `.env.example` pour s'assurer que toutes les variables utilisées par `config.rs` sont documentées :
```bash
# === Kesh — Configuration ===
# Copier ce fichier vers .env et adapter les valeurs

# --- Base de données ---
DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh
MARIADB_ROOT_PASSWORD=kesh_dev_root
MARIADB_DATABASE=kesh
MARIADB_USER=kesh
MARIADB_PASSWORD=kesh_dev

# --- Application ---
KESH_PORT=3000
KESH_HOST=0.0.0.0

# --- Compte admin initial (FR3) ---
KESH_ADMIN_USERNAME=admin
KESH_ADMIN_PASSWORD=changeme

# --- Docker ---
COMPOSE_PROJECT_NAME=kesh

# --- Logging ---
RUST_LOG=info

# --- Variables ajoutées dans les stories suivantes ---
# Auth (Story 1.5-1.6): KESH_JWT_SECRET, KESH_JWT_EXPIRY_MINUTES, etc.
# i18n (Story 7.x): KESH_LANG
```

2.2. S'assurer que `DATABASE_URL` utilise `127.0.0.1` (pas `localhost`) pour cohérence avec le port-binding docker-compose.dev.yml

### Tâche 3 : Dockerfile multi-stage (AC-3)

3.1. Créer `Dockerfile` à la racine du projet :

```dockerfile
# --- Stage 1 : Build Rust ---
FROM rust:1.85-bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo build --release -p kesh-api

# --- Stage 2 : Build Frontend ---
FROM node:22-bookworm-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# --- Stage 3 : Image finale ---
FROM debian:bookworm-slim AS runtime
# curl nécessaire pour le healthcheck Docker
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/kesh-api ./kesh-api
# strip réduit la taille du binaire (~30-50%), sécurise la contrainte < 100 Mo
RUN strip /app/kesh-api
COPY --from=frontend-builder /app/frontend/build ./static
ENV KESH_STATIC_DIR=/app/static
EXPOSE 3000
CMD ["./kesh-api"]
```

**Notes importantes :**
- `debian:bookworm-slim` plutot que `alpine` pour eviter les problemes de compilation musl avec SQLx
- Si l'image depasse 100 Mo, envisager `cargo-chef` pour le cache des dependances (optimisation Story 8.1)
- Le binaire Rust en release est typiquement 10-20 Mo, les fichiers statiques SPA < 5 Mo, l'image de base ~80 Mo
- Alternative : utiliser `FROM scratch` ou `FROM gcr.io/distroless/cc-debian12` pour image encore plus legere (Story 8.1)

3.2. Créer `.dockerignore` :
```
target/
frontend/node_modules/
frontend/.svelte-kit/
frontend/build/
.env
.env.*
.git/
*.md
_bmad/
_bmad-output/
design-artifacts/
docs/
charts/
```

3.3. Vérifier : `docker build -t kesh:dev .` construit l'image sans erreur
3.4. Vérifier : `docker image ls kesh:dev` — taille < 100 Mo (AC-3)

### Tâche 4 : Mise à jour docker-compose.dev.yml (AC-1)

> **Note importante :** Le `docker-compose.dev.yml` passe de "MariaDB seule" (Story 1.1) à "stack complète" (MariaDB + kesh-api). Pour le développement quotidien avec hot reload, le développeur continue d'utiliser `cargo run` directement + MariaDB via compose. Le service kesh dans le compose est un "mode test intégration" pour valider le Dockerfile et la configuration Docker.

4.1. Ajouter le service `kesh` au `docker-compose.dev.yml` existant :

```yaml
services:
  kesh:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: kesh-dev
    restart: unless-stopped
    ports:
      - "127.0.0.1:3000:3000"
    environment:
      DATABASE_URL: mysql://kesh:kesh_dev@mariadb:3306/kesh
      KESH_PORT: 3000
      KESH_HOST: 0.0.0.0
      KESH_ADMIN_USERNAME: ${KESH_ADMIN_USERNAME:-admin}
      KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD:-changeme}
      RUST_LOG: ${RUST_LOG:-info}
    depends_on:
      mariadb:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 15s

  mariadb:
    # ... (configuration existante inchangée)
```

**Notes :**
- `DATABASE_URL` dans le container utilise `mariadb` (nom du service Docker) comme hostname, pas `127.0.0.1`
- `depends_on` avec `condition: service_healthy` pour attendre que MariaDB soit prêt
- Le port est bindé sur `127.0.0.1:3000` (même pattern que MariaDB pour la sécurité)
- Le healthcheck Docker utilise curl — il faudra ajouter `curl` dans l'image ou utiliser un healthcheck alternatif (wget ou binaire custom). Alternative : ne pas mettre de healthcheck Docker sur le container kesh en dev, utiliser uniquement le endpoint `/health` manuellement

4.2. **Alternative au healthcheck Docker (recommandée pour dev)** : Plutôt que d'installer curl dans l'image, utiliser un script ou simplement ne pas définir de healthcheck sur le service kesh en dev (le `depends_on: mariadb: condition: service_healthy` suffit pour l'ordonnancement). Le healthcheck `/health` reste accessible manuellement via `curl http://127.0.0.1:3000/health`.

4.3. Vérifier : `docker compose -f docker-compose.dev.yml up` démarre les 2 containers
4.4. Vérifier : `curl http://127.0.0.1:3000/health` retourne 200

### Tâche 5 : Log URL au démarrage (AC-5)

5.1. Dans `main.rs`, après le bind du serveur, afficher :
```
INFO kesh_api: Kesh démarré sur http://0.0.0.0:3000
INFO kesh_api: Healthcheck : http://0.0.0.0:3000/health
INFO kesh_api: Base de données : connectée (ou : indisponible)
```

5.2. Utiliser `tracing::info!` pour que les messages apparaissent dans les logs Docker (`stdout`)

### Tâche 6 : Validation finale (AC-1 à AC-5)

6.1. `cargo build -p kesh-api` — compile sans erreur
6.2. `cargo test -p kesh-api` — tests passent (si tests unitaires ajoutés)
6.2.1. Ajouter un test unitaire minimal dans kesh-api pour Config::from_env() :
- Test avec DATABASE_URL défini → Config créé avec valeurs par défaut correctes
- Test sans DATABASE_URL → erreur ConfigError::MissingVar
6.3. `docker build -t kesh:dev .` — build réussi
6.4. `docker image ls kesh:dev` — taille < 100 Mo
6.5. `docker compose -f docker-compose.dev.yml up -d` — 2 containers démarrent
6.6. `curl http://127.0.0.1:3000/health` — retourne `{"status":"ok","database":"connected"}` (200)
6.7. `docker compose -f docker-compose.dev.yml logs kesh` — contient l'URL au démarrage
6.8. Tester avec `.env` modifié (port, mot de passe) — l'application utilise les nouvelles valeurs

## Notes de développement

### Architecture de serving

```
Prod :  Navigateur → (nginx TLS optionnel) → Axum :3000 (SPA + API /api/v1/*)
Dev :   Navigateur → Vite :5173 (hot reload) → proxy /api/* → Axum :3000
```

En dev, deux modes de travail possibles :
1. **Mode Vite (frontend dev)** : `npm run dev` sur le frontend (port 5173) + `cargo run -p kesh-api` (port 3000). Le proxy Vite redirige `/api/*` vers Axum. Hot reload frontend.
2. **Mode Docker (test intégration)** : `docker compose -f docker-compose.dev.yml up`. Tout tourne en containers. Pas de hot reload, mais simule l'environnement de production.

### Configuration via dotenvy

Le crate `dotenvy` remplace `dotenv` (qui est abandonné). Pattern de chargement :

```rust
use dotenvy::dotenv;
use std::env;

#[derive(Debug)]
pub enum ConfigError {
    MissingVar(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVar(var) => write!(f, "Variable d'environnement manquante: {}", var),
        }
    }
}

impl std::error::Error for ConfigError {}

pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub host: String,
    pub admin_username: String,
    pub admin_password: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok(); // Charge .env si présent, silencieux sinon
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            port: env::var("KESH_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| ConfigError::Invalid("KESH_PORT"))?,
            host: env::var("KESH_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            admin_username: env::var("KESH_ADMIN_USERNAME")
                .unwrap_or_else(|_| "admin".to_string()),
            admin_password: env::var("KESH_ADMIN_PASSWORD")
                .unwrap_or_else(|_| "changeme".to_string()),
        })
    }
}
```

- `DATABASE_URL` est obligatoire — le serveur ne démarre pas sans.
- Les autres variables ont des valeurs par défaut raisonnables pour le dev.
- En production (Docker), les variables sont passées via `environment:` dans docker-compose.

### Healthcheck endpoint /health

Spécification du endpoint :

```
GET /health
Content-Type: application/json

# Tout OK (200)
{
    "status": "ok",
    "database": "connected"
}

# DB inaccessible (503)
{
    "status": "degraded",
    "database": "disconnected",
    "error": "Connection refused"
}
```

Implémentation :
```rust
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn health(
    State(pool): State<Option<MySqlPool>>,
) -> impl IntoResponse {
    match &pool {
        Some(pool) => {
            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({"status": "ok", "database": "connected"})),
                ),
                Err(e) => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({
                        "status": "degraded",
                        "database": "disconnected",
                        "error": e.to_string()
                    })),
                ),
            }
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "degraded",
                "database": "disconnected",
                "error": "Pool not initialized"
            })),
        ),
    }
}
```

**Note importante :** Le pool SQLx est wrappé dans `Option<MySqlPool>` pour gérer le cas où la DB est inaccessible au démarrage. L'application démarre quand même (FR89 — le frontend doit être servi même sans DB).

### Dockerfile multi-stage

Trois stages :
1. **rust-builder** : compile le binaire Rust en mode release. Image `rust:1.85-bookworm` (~1.5 Go, jetée après build).
2. **frontend-builder** : build le SPA SvelteKit via `npm run build`. Sortie dans `frontend/build/` (adapter-static).
3. **runtime** : image finale `debian:bookworm-slim` (~80 Mo) avec uniquement le binaire + les fichiers statiques.

Budget taille image finale :
- `debian:bookworm-slim` : ~80 Mo
- Binaire Rust release (stripped) : ~10-15 Mo
- Fichiers statiques SPA : ~2-5 Mo
- **Total estimé : ~95 Mo** — proche de la limite de 100 Mo

Si la taille dépasse 100 Mo, options :
- `strip` le binaire dans le Dockerfile : `RUN strip /app/target/release/kesh-api`
- Utiliser `RUSTFLAGS="-C link-arg=-s"` pour strip à la compilation
- Passer à `gcr.io/distroless/cc-debian12` (~20 Mo) — Story 8.1
- Utiliser `cargo-chef` pour optimiser le cache Docker — Story 8.1

### tower-http::ServeDir pour les fichiers statiques

```rust
use tower_http::services::{ServeDir, ServeFile};

// Le SPA SvelteKit est construit dans frontend/build/
// En Docker, copié dans /app/static/
let static_dir = env::var("KESH_STATIC_DIR")
    .unwrap_or_else(|_| "frontend/build".to_string());

let app = Router::new()
    .route("/health", get(health::health))
    // Routes API futures : .nest("/api/v1", api_routes)
    .fallback_service(ServeDir::new(&static_dir).fallback(
        ServeFile::new(format!("{}/index.html", static_dir))
    ));
```

- `ServeDir` sert les fichiers statiques (JS, CSS, HTML)
- `fallback` vers `index.html` pour le routing SPA (toute URL non-API retourne le SPA)
- Variable `KESH_STATIC_DIR` permet de configurer le chemin (différent en dev vs Docker)

### Dépendances Rust (versions de l'architecture)

| Crate | Version | Usage dans cette story |
|---|---|---|
| axum | 0.8.x | Framework HTTP, routing, extractors |
| tokio | 1.x | Runtime async (features: full) |
| tower-http | 0.6.x | ServeDir, CORS, tracing middleware |
| dotenvy | 0.15.x | Chargement fichier .env |
| sqlx | 0.8.6 | Pool MySQL, healthcheck `SELECT 1` |
| tracing | 0.1.x | Macros de logging structuré |
| tracing-subscriber | 0.3.x | Subscriber avec filtre env (RUST_LOG) |
| serde | 1.x | Sérialisation (derive) |
| serde_json | 1.x | Réponses JSON du healthcheck |

**Note :** L'architecture spécifie `tower-http 0.5.x`, mais Axum 0.8 requiert `tower-http 0.6.x`. Utiliser la version compatible avec Axum 0.8.

### Variables d'environnement (état actuel + ajouts)

| Variable | Obligatoire | Défaut | Description |
|---|---|---|---|
| `DATABASE_URL` | Oui | — | URL de connexion MariaDB |
| `KESH_PORT` | Non | `3000` | Port d'écoute HTTP |
| `KESH_HOST` | Non | `0.0.0.0` | Adresse de bind |
| `KESH_ADMIN_USERNAME` | Non | `admin` | Nom du compte admin initial (FR3) |
| `KESH_ADMIN_PASSWORD` | Non | `changeme` | Mot de passe admin initial (FR3) |
| `KESH_STATIC_DIR` | Non | `frontend/build` | Chemin des fichiers statiques SPA |
| `RUST_LOG` | Non | `info` | Niveau de log (tracing) |
| `MARIADB_ROOT_PASSWORD` | Docker | `kesh_dev_root` | Mot de passe root MariaDB |
| `MARIADB_DATABASE` | Docker | `kesh` | Nom de la base de données |
| `MARIADB_USER` | Docker | `kesh` | Utilisateur MariaDB |
| `MARIADB_PASSWORD` | Docker | `kesh_dev` | Mot de passe utilisateur MariaDB |
| `COMPOSE_PROJECT_NAME` | Docker | `kesh` | Nom du projet Docker Compose |

### Docker-compose.dev.yml : état actuel et modifications

**État actuel (Story 1.1)** : Un seul service `mariadb`, port bindé sur `127.0.0.1:3306`.

**Modifications Story 1.2** : Ajout du service `kesh` qui build le Dockerfile et se connecte à MariaDB via le réseau Docker interne.

**Point important** : Dans le container Docker, `DATABASE_URL` utilise `mariadb` comme hostname (nom du service), pas `127.0.0.1`. En dehors de Docker (dev local), c'est `127.0.0.1:3306`.

### Délimitation avec Story 8.1 (Docker production)

Cette story (1.2) crée :
- Un Dockerfile fonctionnel (multi-stage, image < 100 Mo)
- Le service kesh dans docker-compose.dev.yml
- Le endpoint /health

Story 8.1 créera :
- `docker-compose.yml` (production, sans build context, image depuis registry)
- Optimisations Dockerfile (cargo-chef, distroless, strip)
- Configuration TLS/nginx optionnel
- Volumes de données persistants
- Logs stdout/stderr conformes Docker (déjà en place via tracing)

### Notes de la story précédente (Story 1.1)

- `rust-toolchain.toml` fixe la version à `1.85.0` (requis pour `edition = "2024"`)
- MariaDB 11.4 LTS (support jusqu'en 2029)
- Port MariaDB bindé sur `127.0.0.1:3306` (pas exposé sur toutes les interfaces)
- `adapter-auto` a été retiré au profit de `adapter-static` (SPA)
- Le workspace contient 10 crates, tous compilent avec `cargo build --workspace`
- Le frontend SvelteKit démarre en mode dev avec `npm run dev`
- Variables MariaDB utilisent le préfixe `MARIADB_` (pas `MYSQL_`) — convention MariaDB 11.4
- `kesh-api` a actuellement un `main.rs` placeholder (`println!("Hello, world!")`)
- `kesh-api/Cargo.toml` n'a aucune dépendance — tout est à ajouter dans cette story
- Pas de `lib.rs` dans kesh-api (uniquement `main.rs`) — créer `config.rs` et `routes/` comme modules

### Références

- [Source : architecture.md — Section "Architecture de Serving"]
- [Source : architecture.md — Section "Décisions Architecturales" #16-#19]
- [Source : architecture.md — Section "Infrastructure & Déploiement"]
- [Source : architecture.md — Section "Versions Vérifiées"]
- [Source : architecture.md — Section "Frontières Architecturales"]
- [Source : architecture.md — Section "Structure Complète du Répertoire"]
- [Source : epics.md — Section "Story 1.2 : Docker & configuration"]
- [Source : epics.md — Section "Story 8.1 : Docker-compose & Dockerfile production"]
- [Source : prd.md — FR1 (docker-compose < 15 min)]
- [Source : prd.md — FR2 (configuration via variables d'environnement)]
- [Source : prd.md — FR3 (compte admin initial via env)]
- [Source : prd.md — FR8 (endpoint healthcheck /health)]
- [Source : prd.md — FR89 (frontend servi même si DB inaccessible)]

## Dev Agent Record

### Agent Model Used
(à remplir par l'agent de développement)

### Debug Log References
(à remplir par l'agent de développement)

### Completion Notes List
(à remplir par l'agent de développement)

### File List
(à remplir par l'agent de développement)
