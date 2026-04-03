---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-04-02'
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - docs/kesh-prd-v0.2.md
  - docs/change_request.md
workflowType: 'architecture'
project_name: 'kesh'
user_name: 'Guy'
date: '2026-04-02'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Analyse du Contexte Projet

### Vue d'ensemble des Exigences

**Exigences fonctionnelles :**
89 exigences fonctionnelles (FR1-FR89) réparties en 22 domaines. Le cœur architectural se concentre sur :
- Moteur comptable en partie double avec intégrité absolue (FR20-FR24, FR60-FR62)
- Import/parsing de standards bancaires suisses ISO 20022 (FR42-FR53)
- Génération de documents conformes SIX : QR Bill 2.2, pain.001.001.09.ch.03 (FR31-FR41)
- RBAC multi-utilisateurs avec verrouillage optimiste (FR9-FR17)
- Réconciliation bancaire semi-automatique avec règles évolutives (FR44-FR49)
- Internationalisation 4 langues avec formats suisses natifs (FR75-FR76)

Phasage : v0.1 couvre le cœur comptable + facturation QR Bill + import CAMT + réconciliation. v0.2 ajoute TVA, budgets, avoirs, pain.001, clôture, justificatifs.

**Exigences non-fonctionnelles :**
- Performance : pages < 300ms, import 200 transactions < 2s, PDF < 3s
- Sécurité : argon2/bcrypt, JWT + refresh silencieux, rate limiting, TLS (nginx)
- Intégrité : balance toujours juste, zéro perte de données, arithmétique décimale exacte
- Conformité : CO suisse (art. 957-964), standards SIX, TVA AFC
- Accessibilité : inspiré WCAG AA sans contrainte stricte
- Déploiement : docker-compose, image < 100 Mo, logs stdout/stderr

**Échelle & Complexité :**
- Domaine principal : Web app full-stack (SPA Svelte + API REST Axum)
- Niveau de complexité : Haute — standards bancaires ISO 20022, conformité comptable suisse, TVA multi-taux avec historique
- Composants architecturaux estimés : 10 crates backend + frontend SPA
- Charge : 2-5 utilisateurs simultanés par instance (auto-hébergé)

### Contraintes & Dépendances Techniques

- **Stack imposée** : Rust (Axum), Svelte, MariaDB 10.6+, Docker
- **Arithmétique** : `rust_decimal` exclusivement, jamais de f64 pour les montants
- **Standards SIX** : QR Bill 2.2 (dimensions 46×46mm, position A4, police, croix suisse), pain.001.001.09.ch.03 (validation XSD, SPS 2026), CAMT.053.001.04
- **Encodage** : UTF-8 + ISO-8859-1 (import CSV, détection automatique)
- **Configuration** : variables d'environnement
- **Licence** : EUPL 1.2
- **Web app uniquement** : pas de Tauri (décision PRD final)

### Préoccupations Transversales Identifiées

1. **Intégrité comptable** — Validation débit=crédit sur chaque écriture, immutabilité post-clôture, `rust_decimal` partout. Traverse : saisie, import, réconciliation, clôture, rapports.
2. **Internationalisation** — 4 langues (FR/DE/IT/EN), Fluent `.ftl`, backend expose les traductions. Traverse : toute l'UI, les PDF générés, les messages d'erreur, l'aide contextuelle.
3. **Formats suisses** — Apostrophe séparateur de milliers (`1'234.56`), dates `dd.mm.yyyy`. Traverse : affichage UI, PDF, exports CSV.
4. **Authentification & Autorisation** — JWT + refresh, RBAC 3 rôles, rate limiting. Traverse : toutes les routes API, toutes les vues frontend.
5. **Validation métier** — IBAN/QR-IBAN, IDE CHE (checksum), XSD pain.001, conformité QR Bill SIX. Traverse : carnet d'adresses, facturation, paiements.
6. **Audit & Conservation** — CO suisse exige 10 ans de conservation. Écritures verrouillées post-clôture, pas de suppression définitive. Traverse : toute modification de données comptables.
7. **Gestion d'erreurs structurée** — Import partiel avec listing détaillé, doublons, conflit de version (optimistic locking), session expirée. Traverse : import, saisie concurrente, UI.

### Décisions Architecturales Préliminaires (issues de la revue collaborative)

| # | Décision | Choix | Justification |
|---|---|---|---|
| 1 | Séparation core/db | Stricte — `kesh-core` sans I/O | Testabilité, évolutivité, logique métier pure |
| 2 | Versioning parseurs SIX | Multi-version pour CAMT.053/pain.001, dernière version seulement pour QR Bill | Les banques ne migrent pas toutes en même temps ; SIX impose une date de bascule pour QR Bill |
| 3 | Concurrence imports bancaires | Blocage (mutex par compte bancaire) | Simplicité, pas de réconciliation de conflits pour 2-5 utilisateurs |
| 4 | Phasage architectural | Architecture complète dès v0.1 | Éviter les refactors lourds, interfaces v0.2 définies sans implémentation |
| 5 | Extension stocks (CR-003) | Points d'extension prévus dès v0.1 | Trait/interface pour entités gérées, sans code mort |
| 6 | Crates autonomes | Structure indépendante (zéro dépendance), publication remise à plus tard | Bonne architecture sans contraintes de publication prématurées |
| 7 | Types des crates publiables | Types autonomes par crate, conversion via From/Into côté Kesh | Indépendance totale entre crates |
| 8 | Documentation | Complète dès le jour 1 — `///` Rust, `//!` modules, JSDoc Svelte | Hygiène, maintenabilité, attractivité open source |
| 9 | Module de réconciliation | `kesh-reconciliation` dédié — matching, règles d'affectation, mutex | Un module par domaine métier, éviter les modules gigantesques |
| 10 | Glossaire comptable | PDF statique avec how-to (v0.2) + tooltips courts (clés Fluent) dans l'UI | Ne pas surcharger l'interface, tout en aidant les non-comptables |
| 11 | Seed rechargeable | `kesh-seed` crate séparée, passe par `kesh-core` → `kesh-db`, contourne `kesh-api` | Validations métier préservées, pas besoin de serveur HTTP |
| 12 | Génération PDF | Chaque crate gère son propre PDF — `kesh-qrbill` (pixel-perfect SIX) et `kesh-report` (tabulaire) séparés | Besoins trop différents pour une couche partagée |
| 13 | `kesh-i18n` | Crate séparée — chargement Fluent + formatage suisse (montants, dates) | Dépendance transversale utilisée par report, api, qrbill |
| 14 | Stratégie de test | 3 niveaux : unitaires (core seul), intégration (seed+db), E2E Playwright (api complet seedé) | Périmètres clairs, zéro ambiguïté |

### Structure Workspace Cargo

```
Crates publiables (indépendants)       Crates Kesh (internes)
┌──────────────────┐                   ┌────────────────────┐
│ kesh-import      │───────────────┐   │ kesh-core          │
│ kesh-payment     │───────────┐   ├──►│ kesh-db            │
│ kesh-qrbill      │───────┐   │   │   │ kesh-api           │
└──────────────────┘       │   │   │   │ kesh-reconciliation │
                           ▼   ▼   ▼   │ kesh-i18n          │
                      Conversions via   │ kesh-report        │
                      From/Into dans    │ kesh-seed          │
                      kesh-core/api     └────────────────────┘
```

### Stratégie de Test

```
Tests unitaires        : kesh-core seul (logique pure, pas d'I/O)
Tests d'intégration    : kesh-seed + kesh-db (données réalistes en base)
Tests E2E (Playwright) : kesh-api complet avec base seedée
```

## Évaluation Starter Template

### Domaine Technologique Principal

Full-stack : Rust (Axum) backend + Svelte SPA frontend, déployé via Docker.

### Versions Vérifiées (avril 2026)

| Technologie | Version | Usage |
|---|---|---|
| Axum | 0.8.x | Framework HTTP Tokio |
| SQLx | 0.8.6 | Driver MySQL/MariaDB async |
| rust_decimal | 1.39.0 | Arithmétique financière exacte |
| fluent-bundle | 0.16.x | Internationalisation Fluent |
| tower-http | 0.5.x | Middleware CORS, compression, tracing, ServeDir |
| Svelte | 5.55.x | Framework réactif |
| SvelteKit | 2.55.x | Framework applicatif (mode SPA) |
| Playwright | 1.57.x | Tests E2E |

### Options Considérées

**Frontend :**
- **Option A — SvelteKit SPA (`adapter-static`)** ✅ : routing intégré, layouts, error pages, build Vite, Playwright intégrable. Sortie = fichiers statiques.
- **Option B — Svelte pur + Vite** ❌ : trop minimal, il faudrait réinventer routing et layouts.

**Backend :**
- **Cargo workspace manuel** ✅ : pas de starter template pertinent pour un workspace multi-crates custom. Scaffold manuel = norme Rust.

### Sélection Retenue

| Composant | Choix | Initialisation |
|---|---|---|
| Frontend | SvelteKit SPA (`adapter-static`, `ssr=false`) | `npx sv create frontend` |
| Backend | Cargo workspace manuel (10 crates) | `Cargo.toml` workspace + `cargo init` par crate |

### Commandes d'Initialisation

```bash
# Frontend — SvelteKit SPA
npx sv create frontend
# → Svelte 5, TypeScript, Playwright, adapter-static
# → ssr = false dans root layout

# Backend — Cargo workspace
mkdir -p crates/{kesh-core,kesh-db,kesh-api,kesh-reconciliation,kesh-i18n,kesh-report,kesh-seed,kesh-import,kesh-payment,kesh-qrbill}
# → Cargo.toml racine avec [workspace] members
# → cargo init pour chaque crate
```

### Architecture de Serving

```
Prod :  Navigateur → (nginx TLS optionnel) → Axum :3000 (SPA + API /api/v1/*)
Dev :   Navigateur → Vite :5173 (hot reload) → proxy /api/* → Axum :3000
```

- **Axum sert tout** : fichiers statiques SPA via `tower-http::ServeDir` + API REST
- **TLS non géré par Kesh** : HTTP pur, TLS = infrastructure (nginx/Traefik/Caddy)
- **DB inaccessible** : le frontend est servi quand même, message d'erreur côté client
- **Docker-compose** : 2 containers (kesh + mariadb), nginx optionnel

### Outils Recommandés

- `cargo-nextest` — exécution de tests plus rapide
- `cargo-deny` — audit licences et sécurité
- `cargo-udeps` — détection de dépendances inutilisées

### Décisions Architecturales (étape 3)

| # | Décision | Choix | Justification |
|---|---|---|---|
| 15 | TLS | Pas géré par Kesh — HTTP pur | TLS = problème d'infrastructure, pas d'application |
| 16 | Serving frontend | Axum sert tout via `tower-http::ServeDir` | Source unique, pas de nginx requis |
| 17 | DB inaccessible | Frontend servi, message d'erreur client | L'utilisateur voit toujours quelque chose |
| 18 | Docker-compose | 2 containers : kesh + mariadb | Simplifié vs 3 containers avec nginx |
| 19 | Dev workflow | Vite dev server + proxy vers Axum | Hot reload frontend sans rebuild |

**Note :** L'initialisation du projet (workspace Cargo + SvelteKit) sera la première story d'implémentation.

**Note PRD :** Les points suivants devront être mis à jour dans le PRD :
1. Axum sert le frontend en production (plus nginx par défaut)
2. nginx optionnel (TLS/reverse proxy uniquement)
3. docker-compose simplifié à 2 containers
4. Le frontend s'affiche même si la DB est inaccessible

## Décisions Architecturales Principales

### Analyse des Priorités

**Décisions critiques (bloquent l'implémentation) :**
- Data architecture, authentification, structure API, serving frontend

**Décisions importantes (façonnent l'architecture) :**
- Composants UI, state management, logging, CI/CD

**Décisions différées (post-MVP) :**
- Documentation API formelle (OpenAPI), permissions granulaires, scaling horizontal

### Data Architecture

| Décision | Choix | Justification |
|---|---|---|
| Accès base de données | SQLx direct + repository pattern dans `kesh-db` | Contrôle total sur le SQL, requêtes comptables complexes. Migration vers PostgreSQL possible en réécrivant les repositories sans toucher core/api |
| Migrations | `sqlx migrate`, fichiers versionnés dans `crates/kesh-db/migrations/` | Zéro perte de données en cas de migration |
| Cache | Aucun | Simplicité. 2-5 utilisateurs, réévaluation si problème de performance |
| Validation des données | Types forts dans `kesh-core` (newtypes pour IBAN, CHE, montants) + validation API | Intégrité garantie à deux niveaux : types métier + validation HTTP |

### Authentification & Sécurité

| Décision | Choix | Justification |
|---|---|---|
| JWT | `jsonwebtoken` crate | Mature, stable |
| Hashing mots de passe | Argon2id | Standard actuel, résistant GPU |
| Token structure | Access token JWT (~15 min) + Refresh token UUID opaque en base | Révocation immédiate possible (désactivation, changement MdP) |
| RBAC | Hiérarchique : Consultation < Comptable < Admin | Simple, chaque rôle hérite du précédent. Point d'extension prévu pour permissions granulaires post-MVP |
| Rate limiting | `/api/v1/auth/login` uniquement, compteur en mémoire par IP, middleware tower | Le plus simple possible |

### API & Communication

| Décision | Choix | Justification |
|---|---|---|
| Design API | REST, préfixe `/api/v1/`, routes kebab-case | Convention standard |
| Pagination | offset/limit | Simple, suffisant pour le volume |
| Format d'erreur | Structuré avec code métier + message + details | Le frontend peut afficher des messages traduits |
| Documentation API | Pas de documentation formelle MVP, le code fait foi | Réévaluation post-MVP |
| Sérialisation | `serde` / `serde_json`, champs JSON en camelCase | `#[serde(rename_all = "camelCase")]` — convention frontend JS |

### Frontend Architecture

| Décision | Choix | Justification |
|---|---|---|
| State management | Svelte stores natifs (writable, derived) | Simple, intégré, suffisant pour cette taille |
| Composants UI | shadcn-svelte (Svelte 5, Tailwind CSS v4) | Code dans le projet, modifiable, accessible, pas de lock-in |
| Communication API | `fetch` natif + wrapper léger (JWT, refresh, erreurs) | Zéro dépendance externe |
| Formatage montants/dates | Frontend via `Intl.NumberFormat` / `Intl.DateTimeFormat` (locale `de-CH`) | API navigateur native, zéro dépendance, apostrophe suisse native |

### Infrastructure & Déploiement

| Décision | Choix | Justification |
|---|---|---|
| CI/CD | GitHub Actions | Standard open source, repo sur GitHub |
| Logging | `tracing` crate (écosystème Tokio/Axum), stdout/stderr | Logs structurés, conforme Docker |
| Configuration | Variables d'environnement via `dotenvy` + fichier `.env` (dev et prod) | Mécanisme unique partout |
| Scaling | Pas de scaling horizontal. Instances autonomes (1 instance = 1 base) | 2-5 utilisateurs max par instance, même en PME |

### Analyse d'Impact des Décisions

**Séquence d'implémentation :**
1. Scaffold workspace Cargo + SvelteKit SPA
2. `kesh-core` — types forts, newtypes, validation métier
3. `kesh-db` — repository pattern, migrations SQLx, schéma initial
4. `kesh-api` — serveur Axum, auth JWT, RBAC, ServeDir
5. Frontend — SvelteKit, shadcn-svelte, wrapper fetch, stores
6. Crates métier — import, qrbill, reconciliation, etc.

**Dépendances croisées :**
- `kesh-core` n'a aucune dépendance sur les autres crates internes
- `kesh-db` dépend de `kesh-core` (types métier)
- `kesh-api` dépend de `kesh-core`, `kesh-db`, `kesh-i18n`, `kesh-reconciliation`
- `kesh-reconciliation` dépend de `kesh-core`, `kesh-db`
- `kesh-report` dépend de `kesh-core`, `kesh-db`, `kesh-i18n`
- `kesh-seed` dépend de `kesh-core`, `kesh-db`
- Crates publiables (`kesh-import`, `kesh-payment`, `kesh-qrbill`) : zéro dépendance interne

## Patterns d'Implémentation & Règles de Cohérence

### Points de Conflit Identifiés

12 catégories où des agents AI pourraient diverger, toutes traitées ci-dessous.

### Naming Patterns

**Base de données :**

| Élément | Convention | Exemple |
|---|---|---|
| Tables | snake_case, pluriel | `journal_entries`, `import_rules` |
| Colonnes | snake_case | `company_id`, `fiscal_year_start` |
| Clés étrangères | `{table_singulier}_id` | `account_id`, `entry_id` |
| Index | `idx_{table}_{colonnes}` | `idx_accounts_company_id` |
| Contraintes unique | `uq_{table}_{colonnes}` | `uq_users_email` |

**API REST :**

| Élément | Convention | Exemple |
|---|---|---|
| Routes | kebab-case, pluriel | `/api/v1/journal-entries` |
| Paramètres route | `:id` | `/api/v1/accounts/:id` |
| Query params | camelCase | `?companyId=1&pageSize=50` |

**Code Rust :**

| Élément | Convention | Exemple |
|---|---|---|
| Structs/Enums | PascalCase | `JournalEntry`, `AccountType` |
| Fonctions/méthodes | snake_case | `create_entry`, `validate_iban` |
| Modules | snake_case | `journal_entries`, `bank_import` |
| Fichiers | snake_case | `journal_entries.rs` |
| Newtypes | PascalCase | `Iban(String)`, `CheNumber(String)`, `Money(Decimal)` |

**Code Svelte/TypeScript :**

| Élément | Convention | Exemple |
|---|---|---|
| Composants | PascalCase | `JournalEntryForm.svelte` |
| Fichiers routes SvelteKit | kebab-case | `routes/journal-entries/+page.svelte` |
| Fonctions/variables | camelCase | `fetchEntries()`, `accountList` |
| Types/interfaces | PascalCase | `JournalEntry`, `ApiError` |
| Stores | camelCase | `currentCompany`, `journalEntries` |

### Structure Patterns

**Organisation des tests :**
- Rust : tests unitaires co-localisés (`#[cfg(test)] mod tests`), tests d'intégration dans `crates/{crate}/tests/`
- Svelte : tests co-localisés (`Component.test.ts` à côté de `Component.svelte`)
- E2E Playwright : `frontend/tests/e2e/`

**Organisation frontend : par feature**

```
frontend/src/lib/
├── features/
│   ├── journal-entries/    # composants, stores, types, API
│   ├── invoicing/
│   ├── bank-import/
│   ├── reconciliation/
│   ├── accounts/
│   ├── contacts/
│   └── reports/
├── shared/                 # composants partagés, utils, types communs
└── app/                    # layout, auth, navigation, config
```

### Format Patterns

**Réponses API :**
- Succès lecture : donnée directe (pas de wrapper)
- Succès liste : `{ "items": [...], "total": 42, "offset": 0, "limit": 50 }`
- Succès création : 201 + ressource créée
- Succès suppression : 204 (pas de body)
- Erreur : `{ "error": { "code": "ENTRY_UNBALANCED", "message": "...", "details": {...} } }`

**Conventions de données JSON :**
- Champs : camelCase (`companyId`, `fiscalYearStart`)
- Dates : ISO 8601 (`"2026-04-02"`, `"2026-04-02T14:30:00Z"`)
- Montants : string décimal (`"1234.56"`) — jamais de float
- Champs vides : omis (pas de `"field": null`), sauf si la distinction vide/absent est sémantique

**Codes HTTP :**

| Situation | Code |
|---|---|
| Succès lecture | 200 |
| Création réussie | 201 |
| Suppression réussie | 204 |
| Validation échouée | 400 |
| Non authentifié | 401 |
| Non autorisé (RBAC) | 403 |
| Ressource non trouvée | 404 |
| Conflit (optimistic lock) | 409 |
| Rate limited | 429 |
| Erreur serveur | 500 |

### Communication & Process Patterns

**Gestion des erreurs Rust :**
- `AppError` centralisé dans `kesh-api` → `IntoResponse` Axum
- Types d'erreur par crate (`CoreError`, `DbError`) → conversion via `From<T>` vers `AppError`
- Logging `tracing` côté serveur, message traduit côté client, jamais de stack trace exposée

**Gestion des erreurs Frontend :**
- Wrapper `fetch` intercepte toutes les erreurs
- 401 → refresh automatique, si échec → redirection login
- 409 → modal conflit de version
- 400 → détails de validation sous les champs
- 500 → banner d'erreur générique

**Loading states Frontend :**
- Variable `loading` booléenne par requête dans le composant
- Skeleton/spinner local (pas de spinner global)
- Boutons désactivés pendant la soumission

**Logging Rust :**

| Niveau | Usage |
|---|---|
| `error!` | Erreur inattendue, bug, panique rattrapée |
| `warn!` | Situation anormale mais gérée (import partiel, doublon) |
| `info!` | Événements métier (login, clôture, import terminé) |
| `debug!` | Détails techniques (requêtes SQL, durées) |
| `trace!` | Très verbeux, dev uniquement |

**Verrouillage optimiste :**
- Champ `version` (integer) sur chaque entité modifiable
- Frontend envoie `version` dans PUT/PATCH
- `version` en base ≠ reçue → 409 Conflict
- Pattern uniforme : écritures, comptes, contacts, factures

### Règles Obligatoires pour Tous les Agents AI

1. **Jamais de f64 pour les montants** — `rust_decimal::Decimal` exclusivement
2. **Toute écriture comptable doit être équilibrée** — validation dans `kesh-core` avant persistance
3. **Tout code public documenté** — `///` Rust, JSDoc Svelte
4. **Tests unitaires pour toute logique métier** — pas de code métier sans test
5. **Nommage conforme aux conventions ci-dessus** — vérifiable par revue de code
6. **Erreurs structurées avec code métier** — jamais de string d'erreur en dur côté frontend
7. **Verrouillage optimiste sur toute entité modifiable** — champ `version` systématique

## Structure du Projet & Frontières

### Structure Complète du Répertoire

```
kesh/
├── .github/
│   └── workflows/
│       ├── ci.yml                      # Build, tests, clippy, fmt
│       └── release.yml                 # Build Docker image
├── .env.example                        # Template variables d'environnement
├── .gitignore
├── Cargo.toml                          # Workspace root
├── Cargo.lock
├── docker-compose.yml                  # kesh + mariadb
├── Dockerfile                          # Multi-stage: build Rust + build Svelte → image finale
├── README.md
│
├── crates/
│   ├── kesh-core/                      # Logique métier pure, zéro I/O
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/                  # Newtypes: Money, Iban, CheNumber, etc.
│   │       ├── accounting/             # Partie double, validation balance, journaux
│   │       ├── validation/             # Validations métier (IBAN, CHE, QR-IBAN)
│   │       ├── chart_of_accounts/      # Plans comptables, classes, types de comptes
│   │       └── errors.rs               # CoreError
│   │
│   ├── kesh-db/                        # Persistance MariaDB, repository pattern
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pool.rs                 # Configuration connexion SQLx
│   │   │   ├── repositories/           # Un module par entité
│   │   │   │   ├── accounts.rs
│   │   │   │   ├── journal_entries.rs
│   │   │   │   ├── invoices.rs
│   │   │   │   ├── contacts.rs
│   │   │   │   ├── users.rs
│   │   │   │   ├── import_rules.rs
│   │   │   │   ├── bank_accounts.rs
│   │   │   │   └── companies.rs
│   │   │   └── errors.rs              # DbError
│   │   ├── migrations/                 # sqlx migrate
│   │   └── tests/                      # Tests d'intégration DB
│   │
│   ├── kesh-api/                       # Serveur Axum, routes, auth, ServeDir
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                 # Point d'entrée, setup serveur
│   │       ├── config.rs               # Chargement .env via dotenvy
│   │       ├── routes/                 # Un module par domaine
│   │       │   ├── auth.rs             # Login, refresh, logout
│   │       │   ├── accounts.rs
│   │       │   ├── journal_entries.rs
│   │       │   ├── invoices.rs
│   │       │   ├── contacts.rs
│   │       │   ├── bank_imports.rs
│   │       │   ├── reconciliation.rs
│   │       │   ├── reports.rs
│   │       │   ├── users.rs
│   │       │   ├── companies.rs
│   │       │   ├── bank_accounts.rs
│   │       │   └── health.rs           # /health endpoint
│   │       ├── middleware/
│   │       │   ├── auth.rs             # JWT extraction, RBAC
│   │       │   └── rate_limit.rs       # Rate limiting login
│   │       ├── extractors.rs           # Axum extractors (CurrentUser, Role)
│   │       ├── errors.rs               # AppError → IntoResponse
│   │       └── static_files.rs         # ServeDir frontend SPA
│   │
│   ├── kesh-reconciliation/            # Réconciliation bancaire
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── matching.rs             # Algorithme de matching transactions
│   │       ├── rules.rs                # Règles d'affectation automatique
│   │       ├── mutex.rs                # Mutex par compte bancaire
│   │       └── errors.rs
│   │
│   ├── kesh-i18n/                      # Internationalisation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── loader.rs              # Chargement fichiers .ftl
│   │   │   └── formatting.rs          # Montants suisses, dates
│   │   └── locales/
│   │       ├── fr-CH/
│   │       ├── de-CH/
│   │       ├── it-CH/
│   │       └── en-CH/
│   │
│   ├── kesh-report/                    # Rapports comptables → PDF/CSV
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── balance_sheet.rs        # Bilan
│   │       ├── income_statement.rs     # Compte de résultat
│   │       ├── trial_balance.rs        # Balance des comptes
│   │       ├── journal_report.rs       # Journaux
│   │       ├── pdf.rs                  # Génération PDF tabulaire
│   │       └── csv.rs                  # Export CSV
│   │
│   ├── kesh-seed/                      # Données de démo/test
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── demo_data.rs            # Données démo réalistes
│   │       └── main.rs                 # CLI: cargo run -p kesh-seed
│   │
│   ├── kesh-import/                    # PUBLIABLE — Parseurs bancaires
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── camt053/               # Parseur CAMT.053 (multi-version)
│   │   │   ├── csv/                    # Parseur CSV générique (multi-encodage)
│   │   │   └── types.rs               # Types autonomes (pas de dépendance kesh)
│   │   └── tests/
│   │       ├── fixtures/               # Fichiers de test SIX officiels
│   │       └── camt053_tests.rs
│   │
│   ├── kesh-payment/                   # PUBLIABLE — Génération pain.001
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pain001/               # Générateur pain.001 (multi-version)
│   │   │   ├── validation.rs           # Validation XSD
│   │   │   └── types.rs               # Types autonomes
│   │   └── tests/
│   │       └── fixtures/               # Schémas XSD SIX
│   │
│   └── kesh-qrbill/                    # PUBLIABLE — Swiss QR Bill
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── generator.rs            # Génération QR Code + mise en page
│       │   ├── pdf.rs                  # PDF pixel-perfect SIX (46×46mm)
│       │   ├── validation.rs           # Validation conformité SIX 2.2
│       │   └── types.rs               # Types autonomes
│       └── tests/
│           └── fixtures/
│
├── frontend/                           # SvelteKit SPA
│   ├── package.json
│   ├── svelte.config.js                # adapter-static, ssr=false
│   ├── vite.config.ts                  # Proxy /api/* → Axum en dev
│   ├── tsconfig.json
│   ├── tailwind.config.ts
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css                     # Tailwind imports
│   │   ├── routes/
│   │   │   ├── +layout.svelte          # Layout principal (nav, auth check)
│   │   │   ├── +layout.ts             # ssr=false, prerender=false
│   │   │   ├── +page.svelte            # Page d'accueil post-login
│   │   │   ├── +error.svelte           # Page d'erreur (DB down, etc.)
│   │   │   ├── login/
│   │   │   ├── accounts/
│   │   │   ├── journal-entries/
│   │   │   ├── invoices/
│   │   │   ├── contacts/
│   │   │   ├── bank-import/
│   │   │   ├── reconciliation/
│   │   │   ├── reports/
│   │   │   ├── bank-accounts/
│   │   │   └── settings/               # Users, company, config
│   │   └── lib/
│   │       ├── features/
│   │       │   ├── journal-entries/
│   │       │   │   ├── JournalEntryForm.svelte
│   │       │   │   ├── JournalEntryList.svelte
│   │       │   │   ├── journal-entries.store.ts
│   │       │   │   ├── journal-entries.api.ts
│   │       │   │   ├── journal-entries.types.ts
│   │       │   │   └── JournalEntryForm.test.ts
│   │       │   ├── invoicing/
│   │       │   ├── bank-import/
│   │       │   ├── reconciliation/
│   │       │   ├── accounts/
│   │       │   ├── contacts/
│   │       │   ├── reports/
│   │       │   └── auth/
│   │       ├── shared/
│   │       │   ├── components/         # shadcn-svelte + composants partagés
│   │       │   ├── utils/
│   │       │   │   ├── api-client.ts   # Wrapper fetch (JWT, refresh, erreurs)
│   │       │   │   └── formatting.ts   # Intl.NumberFormat, Intl.DateTimeFormat
│   │       │   └── types/
│   │       │       ├── api.ts          # ApiError, PaginatedResponse
│   │       │       └── common.ts
│   │       └── app/
│   │           ├── stores/             # Stores globaux (currentUser, locale)
│   │           └── config.ts
│   ├── static/                         # Assets statiques (favicon, etc.)
│   └── tests/
│       └── e2e/                        # Playwright
│           ├── auth.spec.ts
│           ├── journal-entries.spec.ts
│           ├── invoicing.spec.ts
│           ├── bank-import.spec.ts
│           └── reconciliation.spec.ts
│
└── charts/                             # Plans comptables standards
    ├── pme.json
    ├── association.json
    └── independant.json
```

### Mapping Exigences → Structure

| Domaine FR | Backend | Frontend |
|---|---|---|
| FR1-FR8 Configuration/Onboarding | `kesh-api/routes/companies.rs`, `kesh-api/routes/health.rs` | `routes/settings/`, `features/auth/` |
| FR9-FR17 Utilisateurs/Sécurité | `kesh-api/middleware/auth.rs`, `kesh-api/routes/auth.rs`, `kesh-api/routes/users.rs` | `features/auth/`, `shared/utils/api-client.ts` |
| FR18-FR24 Plan comptable/Écritures | `kesh-core/accounting/`, `kesh-core/chart_of_accounts/`, `kesh-db/repositories/` | `features/accounts/`, `features/journal-entries/` |
| FR25-FR28 Contacts | `kesh-db/repositories/contacts.rs` | `features/contacts/` |
| FR29-FR30 Catalogue | `kesh-db/repositories/` (ajout) | `features/invoicing/` (catalogue intégré) |
| FR31-FR38 Facturation | `kesh-qrbill/`, `kesh-api/routes/invoices.rs` | `features/invoicing/` |
| FR39-FR41 Paiements (v0.2) | `kesh-payment/` | `features/` (ajout v0.2) |
| FR42-FR53 Import/Réconciliation | `kesh-import/`, `kesh-reconciliation/`, `kesh-api/routes/bank_imports.rs` | `features/bank-import/`, `features/reconciliation/` |
| FR54-FR56 TVA (v0.2) | `kesh-core/` (ajout v0.2) | `features/` (ajout v0.2) |
| FR65-FR70 Rapports | `kesh-report/` | `features/reports/` |
| FR75-FR76 i18n | `kesh-i18n/` | `shared/utils/formatting.ts` |
| FR77-FR80 Déploiement | `Dockerfile`, `docker-compose.yml`, `kesh-seed/` | — |

### Frontières Architecturales

```
┌─────────────────────────────────────────────────┐
│                   Navigateur                     │
│  ┌───────────────────────────────────────────┐  │
│  │  SvelteKit SPA (shadcn-svelte, stores)    │  │
│  │  └── api-client.ts (fetch + JWT)          │  │
│  └──────────────────┬────────────────────────┘  │
└─────────────────────┼───────────────────────────┘
                      │ HTTP JSON /api/v1/*
┌─────────────────────┼───────────────────────────┐
│ kesh-api (Axum)     │                           │
│  ├── middleware (JWT, RBAC, rate limit)          │
│  ├── routes → appelle kesh-core + kesh-db       │
│  ├── ServeDir (SPA statique)                    │
│  └── AppError (toutes erreurs → JSON)           │
├─────────────────────────────────────────────────┤
│ kesh-core          │ kesh-reconciliation        │
│  (logique pure)    │  (matching, rules, mutex)  │
│  (types, valid.)   │                            │
├─────────────────────────────────────────────────┤
│ kesh-db (repository pattern, SQLx)              │
│  └── migrations/                                │
├─────────────────────────────────────────────────┤
│ MariaDB                                         │
└─────────────────────────────────────────────────┘

Crates publiables (indépendants, zéro dépendance) :
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ kesh-import  │ │ kesh-payment │ │ kesh-qrbill  │
│ CAMT.053/CSV │ │ pain.001     │ │ QR Bill SIX  │
└──────────────┘ └──────────────┘ └──────────────┘
      ↕ From/Into        ↕ From/Into      ↕ From/Into
      kesh-core/api      kesh-core/api    kesh-core/api
```

### Flux de Données

```
Import bancaire :
  Fichier CAMT.053 → kesh-import (parse) → From/Into → kesh-core (validation)
  → kesh-reconciliation (matching, mutex) → kesh-db (persistance)

Facturation :
  Frontend (saisie) → kesh-api (validation) → kesh-core (logique facture)
  → kesh-db (persistance) → kesh-qrbill (génération PDF) → Frontend (téléchargement)

Rapports :
  Frontend (demande) → kesh-api → kesh-db (requêtes agrégées)
  → kesh-report (mise en forme) → kesh-i18n (formatage) → PDF/CSV → Frontend
```

## Résultats de Validation de l'Architecture

### Validation de Cohérence ✅

**Compatibilité des décisions :**
- Axum 0.8.x + tower-http 0.5.x + SQLx 0.8.x + tokio : écosystème Tokio cohérent
- SvelteKit 2.x + Svelte 5 + shadcn-svelte + Tailwind CSS v4 : compatibilité vérifiée
- `rust_decimal` 1.39 + SQLx MariaDB : support natif du type Decimal
- `fluent-bundle` 0.16 : stable, pas de dépendance conflictuelle
- snake_case DB ↔ camelCase JSON ↔ snake_case Rust : conversions gérées par `serde(rename_all)`

**Cohérence des patterns :**
- Repository pattern ↔ SQLx direct : cohérent
- Feature-based frontend ↔ routes SvelteKit par domaine : aligné
- Naming conventions cohérentes à travers toutes les couches

**Alignement structure :**
- Chaque domaine FR a un répertoire backend ET frontend correspondant
- Les frontières crate/module respectent la séparation des responsabilités
- Les crates publiables sont effectivement indépendantes (zéro dépendance vérifiée)

### Validation de Couverture des Exigences ✅

**Exigences fonctionnelles v0.1 :** 100% couvertes
**Exigences fonctionnelles v0.2 :** 3 gaps mineurs, non bloquants, extensibles :
- FR57-FR59 Budgets : ajout `budgets.rs` dans `kesh-db` + logique dans `kesh-core`
- FR63-FR64 Justificatifs : ajout `attachments.rs` dans `kesh-db` (filesystem, volume Docker dédié)
- FR81 Modèles documents : personnalisation templates dans `kesh-report`

**Exigences non-fonctionnelles :** Toutes couvertes
- Performance : SQLx async, parseurs Rust natifs
- Sécurité : argon2id, JWT + refresh, RBAC, rate limiting, TLS = infra
- Intégrité : `kesh-core` types forts, `rust_decimal`, validation balance
- Accessibilité : shadcn-svelte (ARIA, navigation clavier)
- Déploiement : Docker multi-stage, image < 100 Mo, logs stdout/stderr

### Validation de Préparation à l'Implémentation ✅

**Complétude des décisions :** 19 décisions architecturales + 14 décisions préliminaires, toutes documentées avec justifications et versions vérifiées.

**Complétude de la structure :** Arborescence complète (~80 fichiers/dossiers), mapping FR → répertoires explicite.

**Complétude des patterns :** Naming (DB, API, Rust, Svelte/TS), formats (API, JSON, HTTP), process (erreurs, loading, logging, verrouillage) — tous couverts. 7 règles obligatoires pour les agents AI.

### Checklist de Complétude

**✅ Analyse des exigences**
- [x] Contexte projet analysé en profondeur
- [x] Échelle et complexité évaluées
- [x] Contraintes techniques identifiées
- [x] Préoccupations transversales mappées

**✅ Décisions architecturales**
- [x] Décisions critiques documentées avec versions
- [x] Stack technologique entièrement spécifiée
- [x] Patterns d'intégration définis
- [x] Considérations de performance adressées

**✅ Patterns d'implémentation**
- [x] Conventions de nommage établies
- [x] Patterns de structure définis
- [x] Patterns de communication spécifiés
- [x] Patterns de processus documentés

**✅ Structure du projet**
- [x] Structure complète des répertoires définie
- [x] Frontières des composants établies
- [x] Points d'intégration mappés
- [x] Mapping exigences → structure complet

### Évaluation de Préparation

**Statut global :** PRÊT POUR L'IMPLÉMENTATION

**Niveau de confiance :** Élevé

**Forces clés :**
- Séparation nette des responsabilités (10 crates, chacune un domaine)
- Crates publiables indépendantes (potentiel communautaire)
- Patterns explicites pour éviter les divergences entre agents AI
- Architecture complète v0.2 prévue sans code mort en v0.1
- Décisions issues de revues collaboratives multi-perspectives (Party Mode)

**Points à surveiller :**
- Performance sans cache (à mesurer en conditions réelles)
- Complexité du workspace Cargo (10 crates = temps de compilation)
- Réconciliation bancaire (algorithme de matching à affiner avec données réelles)

**Gaps v0.2 identifiés (non bloquants) :**
- Budgets : repository + logique core à ajouter
- Justificatifs : stockage filesystem (volume Docker dédié), métadonnées dans kesh-db
- Modèles documents : personnalisation templates PDF

### Consignes pour les Agents AI

- Suivre toutes les décisions architecturales exactement comme documentées
- Utiliser les patterns d'implémentation de manière cohérente
- Respecter la structure du projet et les frontières entre crates
- Se référer à ce document pour toute question architecturale
- Ne jamais utiliser de f64 pour les montants — `rust_decimal::Decimal` exclusivement

### Modifications PRD à Reporter

1. Axum sert le frontend en production (plus nginx par défaut)
2. nginx optionnel (TLS/reverse proxy uniquement)
3. docker-compose simplifié à 2 containers (kesh + mariadb)
4. Le frontend s'affiche même si la DB est inaccessible
