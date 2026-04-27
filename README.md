# Kesh

[![CI](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml)
[![Release](https://github.com/guycorbaz/kesh/actions/workflows/release.yml/badge.svg)](https://github.com/guycorbaz/kesh/actions/workflows/release.yml)
[![License: EUPL 1.2](https://img.shields.io/badge/license-EUPL--1.2-blue.svg)](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
[![Rust](https://img.shields.io/badge/rust-1.85-orange.svg)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/svelte-5-ff3e00.svg)](https://svelte.dev/)

**Kesh** est un logiciel de comptabilité et de gestion pour indépendants, TPE et associations en Suisse. Gratuit, open source, auto-hébergé.

## Table des matières

- [Fonctionnalités](#fonctionnalités)
- [Pile technique](#pile-technique)
- [Démarrage rapide](#démarrage-rapide)
- [Structure du projet](#structure-du-projet)
- [Développement](#développement)
- [Tests](#tests)
- [Feuille de route](#feuille-de-route)
- [Contribuer](#contribuer)
- [Licence](#licence)

## Fonctionnalités

- **Comptabilité en partie double** — plan comptable suisse, écritures validées, audit log
- **Carnet d'adresses & catalogue produits** — contacts, conditions de paiement, TVA
- **Facturation QR Bill 2.2** — génération PDF conforme au standard suisse
- **Import bancaire CAMT.053 / CSV** — réconciliation automatique *(à venir)*
- **Paiements pain.001.001.03** — fichiers de paiement ISO 20022 *(à venir)*
- **TVA suisse** — calcul et rapports par période *(à venir)*
- **Multilingue** — FR, DE, IT, EN
- **Multi-utilisateurs** — RBAC avec rôles, JWT + refresh tokens, isolation multi-tenant par `company_id`

## Pile technique

- **Backend** : Rust 1.85 (édition 2024), Axum, SQLx
- **Frontend** : SvelteKit 2 + Svelte 5, TypeScript, Tailwind CSS 4
- **Base de données** : MariaDB 11.4
- **Déploiement** : Docker Compose (web app uniquement)
- **Tests** : `cargo test`, Vitest, Playwright

## Démarrage rapide

### Prérequis

- Rust ≥ 1.85 (installé automatiquement via `rust-toolchain.toml`)
- Node.js ≥ 20
- Docker + Docker Compose

### Installation

```bash
# 1. Cloner le repo
git clone https://github.com/guycorbaz/kesh.git
cd kesh

# 2. Démarrer MariaDB + backend (mode dev complet)
docker compose -f docker-compose.dev.yml up -d

# 3. Configurer l'environnement
cp .env.example .env
# Adapter les valeurs dans .env

# 4. Frontend (hot reload)
cd frontend
npm install
npm run dev
```

L'application est accessible sur http://localhost:5173 (frontend dev) et http://localhost:3000 (API).

### Image Docker (production)

Les images officielles sont publiées sur Docker Hub à chaque tag `v*.*.*` :

```bash
docker pull guycorbaz/kesh:latest
```

## Structure du projet

```
kesh/
├── crates/                  # Backend Rust (workspace multi-crates)
│   ├── kesh-core/           # Logique métier pure (types, validation)
│   ├── kesh-db/             # Persistance MariaDB, migrations
│   ├── kesh-api/            # Serveur HTTP Axum
│   ├── kesh-i18n/           # Internationalisation (Fluent)
│   ├── kesh-qrbill/         # Génération QR Bill 2.2
│   ├── kesh-payment/        # Fichiers pain.001
│   ├── kesh-import/         # Parseurs CAMT.053, CSV
│   ├── kesh-reconciliation/ # Rapprochement bancaire
│   ├── kesh-report/         # Bilan, résultat, balance
│   └── kesh-seed/           # Données d'amorçage
├── frontend/                # SvelteKit SPA
├── charts/                  # Plans comptables suisses
├── docs/                    # Documentation technique
└── .github/workflows/       # Pipelines CI/CD
```

## Architecture

### Multi-tenant (Story 6.2)

Kesh supporte plusieurs sociétés par instance via un modèle multi-tenant :

- **JWT claims** : chaque token contient `user_id`, `role`, et **`company_id`**
- **Scoping** : toutes les requêtes filtrent par `company_id` du JWT (défense en profondeur contre IDOR)
- **Onboarding** : création de la company lors de l'inscription (contrat Story 6.1)
- **Foreign Key** : `users.company_id` NOT NULL, FK vers `companies.id`

Chaque user est assigné à exactement une company. Le `company_id` est inclus au JWT à la connexion (story 1.5) et utilisé pour scoper tous les accès aux ressources (comptes, contacts, factures, écritures comptables, etc.).

## Développement

### Commandes utiles

```bash
# Backend
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# Frontend
cd frontend
npm run check          # svelte-check
npm run build          # build production
```

### Workflow Git

- Branche principale : `main`
- Les commits sur `main` déclenchent le pipeline CI (tests + build).
- Les tags `v*.*.*` déclenchent le pipeline Release (build et push Docker Hub).

## Tests

```bash
# Tests unitaires + intégration backend
DATABASE_URL='mysql://root:...@127.0.0.1:3306/kesh' \
  cargo test --workspace -- --test-threads=1

# Tests unitaires frontend
cd frontend && npm run test:unit

# Tests E2E Playwright
cd frontend && npm run test:e2e
```

> **Note** : les tests d'intégration SQLx créent des bases éphémères `_sqlx_test_*`. L'utilisateur DB doit avoir les droits `CREATE/DROP` sur `*.*` (en local, utiliser `root`).

## Feuille de route

Le projet suit une approche **BMAD** (Breakthrough Method of Agile AI-driven Development) avec une feuille de route structurée en epics :

| Version | Epics | Statut |
|---------|-------|--------|
| v0.1 | E1 Fondations & Authentification, E2 Onboarding & Configuration, E4 Carnet d'adresses & Catalogue, E5 Facturation QR Bill, E6 Qualité & CI/CD | ✅ Done |
| v0.1 | E3 Plan comptable & Écritures (ré-ouverte 2026-04-27 pour Story 3-7 gestion exercices comptables), E7 Technical Debt Closure | 🚧 En cours |
| v0.1 | E8 Import bancaire & Réconciliation, E9 Rapports & Exports, E10 Déploiement & Opérations | 📋 Backlog |
| v0.2 | E11 TVA Suisse, E12 Avoirs & Paiements (pain.001), E13 Budgets, E14 Clôture d'exercice, E15 Justificatifs & Lettrage | 📋 Backlog |

Détails : [PRD complet](_bmad-output/planning-artifacts/prd.md).

## Contribuer

Les contributions sont les bienvenues. Merci d'ouvrir une issue avant tout changement significatif pour en discuter.

- Respecter les règles de qualité du code (`CLAUDE.md`)
- Ajouter des tests pour toute nouvelle logique métier
- `cargo fmt` + `cargo clippy` doivent passer sans warning

## Licence

Distribué sous licence [EUPL 1.2](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).
