# Kesh

Logiciel de comptabilité et de gestion pour indépendants, TPE et associations en Suisse.

Gratuit, open source (EUPL 1.2), auto-hébergé.

## Démarrage rapide (développement)

### Prérequis

- Rust >= 1.85 (édition 2024)
- Node.js >= 20
- Docker + Docker Compose

### Installation

```bash
# 1. Cloner le repo
git clone https://github.com/gcorbaz/kesh.git
cd kesh

# 2. Démarrer MariaDB
docker compose -f docker-compose.dev.yml up -d

# 3. Configurer l'environnement
cp .env.example .env
# Adapter les valeurs dans .env

# 4. Backend
cargo build --workspace

# 5. Frontend
cd frontend
npm install
npm run dev
```

### Structure du projet

```
kesh/
├── crates/              # Backend Rust (10 crates)
│   ├── kesh-core/       # Logique métier pure
│   ├── kesh-db/         # Persistance MariaDB
│   ├── kesh-api/        # Serveur Axum
│   └── ...
├── frontend/            # SvelteKit SPA
├── charts/              # Plans comptables suisses
└── docs/                # Documentation
```

## Licence

[EUPL 1.2](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
