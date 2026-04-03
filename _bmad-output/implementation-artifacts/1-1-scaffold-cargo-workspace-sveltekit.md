# Story 1.1 : Scaffold Cargo workspace & SvelteKit

Status: review

## Story

**As a** developpeur,
**I want** un workspace Cargo structure et un projet SvelteKit initialise,
**So that** je puisse commencer a developper sur des fondations propres.

## Criteres d'acceptation

1. **Given** workspace vide, **When** cargo init pour chaque crate (kesh-core, kesh-db, kesh-api, kesh-reconciliation, kesh-i18n, kesh-report, kesh-seed, kesh-import, kesh-payment, kesh-qrbill), **Then** Cargo.toml racine avec [workspace] members compile sans erreur
2. **Given** pas de frontend, **When** npx sv create frontend (Svelte 5, TypeScript, Playwright, adapter-static), **Then** SvelteKit demarre en mode dev avec ssr=false
3. **Given** workspace complet, **When** cargo build --workspace, **Then** tous les crates compilent sans erreur
4. **And** .gitignore, README.md, et structure de repertoires conforme a l'architecture
5. **And** docker-compose.dev.yml avec MariaDB locale pour le developpement (kesh-db testable des cette story)

## Taches / Sous-taches

### Tache 1 : Initialiser le workspace Cargo (AC-1, AC-3)

1.1. Creer le `Cargo.toml` racine avec la section `[workspace]` listant les 10 crates membres

1.2. Creer les repertoires pour chaque crate :
```bash
mkdir -p crates/{kesh-core,kesh-db,kesh-api,kesh-reconciliation,kesh-i18n,kesh-report,kesh-seed,kesh-import,kesh-payment,kesh-qrbill}
```

1.3. Executer `cargo init` pour chaque crate :
```bash
for crate in kesh-core kesh-db kesh-api kesh-reconciliation kesh-i18n kesh-report kesh-seed kesh-import kesh-payment kesh-qrbill; do
  cargo init "crates/$crate" --lib
done
```
Note : `kesh-api` et `kesh-seed` doivent avoir un `main.rs` (binaire) en plus de `lib.rs`.

1.4. Configurer chaque `Cargo.toml` de crate avec le nom correct et `edition = "2024"`

1.5. Creer les sous-repertoires internes de chaque crate conformement a la structure d'architecture (voir Notes de developpement)

1.6. Verifier : `cargo build --workspace` compile sans erreur

### Tache 2 : Initialiser le projet SvelteKit (AC-2)

2.1. Initialiser le frontend :
```bash
npx sv create frontend
```
Options a selectionner :
- Svelte 5
- TypeScript
- Playwright
- adapter-static

2.2. Configurer `svelte.config.js` avec `adapter-static` et les options SPA

2.3. Configurer le layout racine avec `ssr = false` et `prerender = false` dans `frontend/src/routes/+layout.ts`

2.4. Configurer `vite.config.ts` avec le proxy `/api/*` vers Axum (port 3000)

2.5. Installer et configurer Tailwind CSS v4 :
```bash
cd frontend && npx svelte-add@latest tailwindcss
```

> **Note Tailwind CSS v4 :** Tailwind v4 utilise une configuration basée sur CSS (`@config` / `@theme` dans `app.css`) au lieu de `tailwind.config.ts`. Ne pas créer manuellement un fichier `tailwind.config.ts` — suivre la sortie de `npx shadcn-svelte@latest init` qui peut en générer un pour la compatibilité shadcn.

2.6. Installer shadcn-svelte :
```bash
cd frontend && npx shadcn-svelte@latest init
```

> **Note :** Les composants spécifiques (Button, Input, Select, Table, etc.) seront importés dans les stories qui les utilisent (Story 1.10 pour le layout, Story 3.2 pour les formulaires, etc.)

2.7. Creer la structure de repertoires frontend (`lib/features/`, `lib/shared/`, `lib/app/`)

> **Note :** Créer uniquement les dossiers vides. Les fichiers (.svelte, .ts) seront créés dans les stories respectives.

2.8. Verifier : `npm run dev` demarre sans erreur avec ssr=false

### Tache 3 : Docker dev pour MariaDB (AC-5)

3.1. Creer `docker-compose.dev.yml` avec le service MariaDB (voir Notes de developpement pour la structure complete)

3.2. Verifier : `docker-compose -f docker-compose.dev.yml up -d` demarre MariaDB et le port 3306 est accessible

### Tache 4 : Fichiers de configuration (AC-4)

4.1. Creer `.env.example` avec toutes les variables documentees (voir Notes de developpement)

4.2. Creer `.gitignore` avec les patterns Rust, Node, Docker, IDE

4.3. Creer `README.md` avec une description minimale du projet, les instructions de demarrage dev, et la licence

4.4. Creer le repertoire `charts/` avec les fichiers placeholder pour les plans comptables :
```
charts/pme.json
charts/association.json
charts/independant.json
```

4.5. Creer le repertoire `.github/workflows/` (vide pour le moment)

### Tache 5 : Validation finale (AC-1, AC-2, AC-3, AC-4, AC-5)

5.1. Verifier que `cargo build --workspace` compile sans erreur
5.2. Verifier que `npm run dev` (frontend) demarre correctement
5.3. Verifier que `docker-compose -f docker-compose.dev.yml up -d` demarre MariaDB
5.4. Verifier que la structure de repertoires correspond a l'architecture documentee
5.5. Verifier que `.gitignore` exclut correctement les artefacts de build
5.6. Commit initial avec tous les fichiers

## Notes de developpement

### Structure complete du repertoire

Voici l'arborescence complete du projet telle que definie dans l'architecture. Pour cette story, seuls les fichiers de scaffold sont crees (les fichiers de code metier seront crees dans les stories suivantes).

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
├── docker-compose.yml                  # kesh + mariadb (production)
├── docker-compose.dev.yml              # MariaDB locale (developpement)
├── Dockerfile                          # Multi-stage: build Rust + build Svelte → image finale
├── README.md
│
├── crates/
│   ├── kesh-core/                      # Logique metier pure, zero I/O
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/                  # Newtypes: Money, Iban, CheNumber, etc.
│   │       ├── accounting/             # Partie double, validation balance, journaux
│   │       ├── validation/             # Validations metier (IBAN, CHE, QR-IBAN)
│   │       ├── chart_of_accounts/      # Plans comptables, classes, types de comptes
│   │       └── errors.rs               # CoreError
│   │
│   ├── kesh-db/                        # Persistance MariaDB, repository pattern
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pool.rs                 # Configuration connexion SQLx
│   │   │   ├── repositories/           # Un module par entite
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
│   │   └── tests/                      # Tests d'integration DB
│   │
│   ├── kesh-api/                       # Serveur Axum, routes, auth, ServeDir
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                 # Point d'entree, setup serveur
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
│   ├── kesh-reconciliation/            # Reconciliation bancaire
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── matching.rs             # Algorithme de matching transactions
│   │       ├── rules.rs                # Regles d'affectation automatique
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
│   │       ├── income_statement.rs     # Compte de resultat
│   │       ├── trial_balance.rs        # Balance des comptes
│   │       ├── journal_report.rs       # Journaux
│   │       ├── pdf.rs                  # Generation PDF tabulaire
│   │       └── csv.rs                  # Export CSV
│   │
│   ├── kesh-seed/                      # Donnees de demo/test
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── demo_data.rs            # Donnees demo realistes
│   │       └── main.rs                 # CLI: cargo run -p kesh-seed
│   │
│   ├── kesh-import/                    # PUBLIABLE — Parseurs bancaires
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── camt053/               # Parseur CAMT.053 (multi-version)
│   │   │   ├── csv/                    # Parseur CSV generique (multi-encodage)
│   │   │   └── types.rs               # Types autonomes (pas de dependance kesh)
│   │   └── tests/
│   │       ├── fixtures/               # Fichiers de test SIX officiels
│   │       └── camt053_tests.rs
│   │
│   ├── kesh-payment/                   # PUBLIABLE — Generation pain.001
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pain001/               # Generateur pain.001 (multi-version)
│   │   │   ├── validation.rs           # Validation XSD
│   │   │   └── types.rs               # Types autonomes
│   │   └── tests/
│   │       └── fixtures/               # Schemas XSD SIX
│   │
│   └── kesh-qrbill/                    # PUBLIABLE — Swiss QR Bill
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── generator.rs            # Generation QR Code + mise en page
│       │   ├── pdf.rs                  # PDF pixel-perfect SIX (46x46mm)
│       │   ├── validation.rs           # Validation conformite SIX 2.2
│       │   └── types.rs               # Types autonomes
│       └── tests/
│           └── fixtures/
│
├── frontend/                           # SvelteKit SPA
│   ├── package.json
│   ├── svelte.config.js                # adapter-static, ssr=false
│   ├── vite.config.ts                  # Proxy /api/* → Axum en dev
│   ├── tsconfig.json
│   ├── tailwind.config.ts              # Peut être généré par shadcn-svelte init (voir note Tailwind v4)
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
│   │       │   ├── components/         # shadcn-svelte + composants partages
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

### Les 10 crates et leur role

| Crate | Type | Description |
|---|---|---|
| `kesh-core` | Interne | Logique metier pure, zero I/O. Types forts (newtypes), validation metier, comptabilite en partie double, plan comptable. Aucune dependance sur les autres crates internes. |
| `kesh-db` | Interne | Persistance MariaDB via SQLx. Repository pattern (un module par entite). Migrations SQL versionnees. Depend de `kesh-core`. |
| `kesh-api` | Interne (binaire) | Serveur Axum : routes REST `/api/v1/*`, middleware JWT/RBAC/rate-limiting, ServeDir pour le SPA, gestion d'erreurs AppError. Depend de `kesh-core`, `kesh-db`, `kesh-i18n`, `kesh-reconciliation`. |
| `kesh-reconciliation` | Interne | Reconciliation bancaire : algorithme de matching, regles d'affectation automatique, mutex par compte. Depend de `kesh-core`, `kesh-db`. |
| `kesh-i18n` | Interne | Internationalisation : chargement fichiers Fluent `.ftl`, formatage montants/dates suisses. |
| `kesh-report` | Interne | Generation de rapports comptables (bilan, compte de resultat, balance, journaux) en PDF tabulaire et CSV. Depend de `kesh-core`, `kesh-db`, `kesh-i18n`. |
| `kesh-seed` | Interne (binaire) | Donnees de demo/test. CLI executable (`cargo run -p kesh-seed`). Passe par `kesh-core` → `kesh-db`, contourne `kesh-api`. Depend de `kesh-core`, `kesh-db`. |
| `kesh-import` | **Publiable** | Parseurs bancaires CAMT.053 (multi-version) et CSV (multi-encodage). Types autonomes, zero dependance interne Kesh. |
| `kesh-payment` | **Publiable** | Generateur pain.001 (multi-version), validation XSD. Types autonomes, zero dependance interne Kesh. |
| `kesh-qrbill` | **Publiable** | Swiss QR Bill conforme SIX 2.2. Generation QR Code, PDF pixel-perfect (46x46mm). Types autonomes, zero dependance interne Kesh. |

### Notes de structure projet

**Separation crates publiables vs internes :**

Les 3 crates publiables (`kesh-import`, `kesh-payment`, `kesh-qrbill`) sont completement independantes : elles n'ont aucune dependance sur les autres crates du workspace. Elles definissent leurs propres types dans `types.rs`. La conversion entre ces types et les types de `kesh-core` se fait via des implementations `From/Into` situees cote `kesh-core` ou `kesh-api`.

Les 7 crates internes forment un graphe de dependances :

```
kesh-core (zero dependance interne)
    ↑
kesh-db (depend de kesh-core)
    ↑
kesh-reconciliation (depend de kesh-core, kesh-db)
kesh-report (depend de kesh-core, kesh-db, kesh-i18n)
kesh-seed (depend de kesh-core, kesh-db)
kesh-api (depend de kesh-core, kesh-db, kesh-i18n, kesh-reconciliation)
```

### Configuration du Cargo.toml racine

```toml
[workspace]
members = [
    "crates/kesh-core",
    "crates/kesh-db",
    "crates/kesh-api",
    "crates/kesh-reconciliation",
    "crates/kesh-i18n",
    "crates/kesh-report",
    "crates/kesh-seed",
    "crates/kesh-import",
    "crates/kesh-payment",
    "crates/kesh-qrbill",
]
resolver = "3"

[workspace.package]
edition = "2024"
license = "EUPL-1.2"
repository = "https://github.com/guy/kesh"
```

**Version Rust minimale :** 1.85+ (requis pour `edition = "2024"`)

Créer un fichier `rust-toolchain.toml` à la racine du projet :

```toml
[toolchain]
channel = "stable"
```

### Configuration SvelteKit

**svelte.config.js :**
```javascript
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: vitePreprocess(),
    kit: {
        adapter: adapter({
            fallback: 'index.html'  // SPA mode — toutes les routes vers index.html
        })
    }
};

export default config;
```

**frontend/src/routes/+layout.ts :**
```typescript
export const ssr = false;
export const prerender = false;
```

**vite.config.ts :**
```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [sveltekit()],
    server: {
        proxy: {
            '/api': {
                target: 'http://localhost:3000',
                changeOrigin: true
            }
        }
    }
});
```

### Configuration docker-compose.dev.yml

> **Note :** Ce fichier `docker-compose.dev.yml` contient uniquement MariaDB pour le développement local. Le `docker-compose.yml` de production (2 containers : kesh + mariadb) sera créé dans la Story 8.1.

> **MariaDB 11.4 LTS** (support jusqu'en 2029), remplace 10.6 (fin de vie juillet 2026).

```yaml
services:
  mariadb:
    image: mariadb:11.4
    container_name: kesh-mariadb-dev
    ports:
      - "3306:3306"
    environment:
      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD:-kesh_dev_root}
      MYSQL_DATABASE: ${MYSQL_DATABASE:-kesh_dev}
      MYSQL_USER: ${MYSQL_USER:-kesh}
      MYSQL_PASSWORD: ${MYSQL_PASSWORD:-kesh_dev_password}
    volumes:
      - kesh-mariadb-data:/var/lib/mysql
    healthcheck:
      test: ["CMD", "healthcheck.sh", "--connect", "--innodb_initialized"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  kesh-mariadb-data:
```

### Variables d'environnement (.env.example)

```bash
# =============================================================================
# Kesh — Variables d'environnement
# Copier ce fichier vers .env et adapter les valeurs
# =============================================================================

# --- Base de donnees ---
DATABASE_URL=mysql://kesh:kesh_dev_password@localhost:3306/kesh_dev
MYSQL_ROOT_PASSWORD=kesh_dev_root
MYSQL_DATABASE=kesh_dev
MYSQL_USER=kesh
MYSQL_PASSWORD=kesh_dev_password

# --- Application ---
KESH_PORT=3000

# --- Compte admin initial (FR3) ---
KESH_ADMIN_USERNAME=admin
KESH_ADMIN_PASSWORD=changeme_before_production

# --- Docker ---
COMPOSE_PROJECT_NAME=kesh

# Variables supplémentaires ajoutées dans les stories suivantes (auth, i18n, logging)
# --- Authentification (Story 2.x) ---
# KESH_JWT_SECRET=change_this_to_a_random_secret_in_production
# # Durée de vie de l'access token JWT (court — le refresh token renouvelle silencieusement)
# KESH_JWT_EXPIRY_MINUTES=15
# # Durée de vie du refresh token (le refresh token doit durer plus longtemps que l'access token)
# KESH_REFRESH_TOKEN_EXPIRY_DAYS=7
# KESH_RATE_LIMIT_MAX_ATTEMPTS=5
# KESH_RATE_LIMIT_WINDOW_MINUTES=15
# KESH_RATE_LIMIT_BLOCK_MINUTES=30
# KESH_MIN_PASSWORD_LENGTH=8
# KESH_HOST=0.0.0.0
#
# --- Internationalisation (Story 7.x) ---
# KESH_LANG=fr
#
# --- Logging ---
# RUST_LOG=info
```

### .gitignore

```gitignore
# Rust
/target/
**/*.rs.bk
Cargo.lock
!Cargo.lock

# Node / Frontend
frontend/node_modules/
frontend/.svelte-kit/
frontend/build/
frontend/dist/

# Environnement
.env
.env.local
.env.production

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# Docker
docker-compose.override.yml

# OS
.DS_Store
Thumbs.db

# Logs
*.log
```

Note : `Cargo.lock` est conserve dans le repo (bonne pratique pour les applications Rust, contrairement aux bibliotheques).

### Conventions de nommage

**Base de donnees :**

| Element | Convention | Exemple |
|---|---|---|
| Tables | snake_case, pluriel | `journal_entries`, `import_rules` |
| Colonnes | snake_case | `company_id`, `fiscal_year_start` |
| Cles etrangeres | `{table_singulier}_id` | `account_id`, `entry_id` |
| Index | `idx_{table}_{colonnes}` | `idx_accounts_company_id` |
| Contraintes unique | `uq_{table}_{colonnes}` | `uq_users_email` |

**API REST :**

| Element | Convention | Exemple |
|---|---|---|
| Routes | kebab-case, pluriel | `/api/v1/journal-entries` |
| Parametres route | `:id` | `/api/v1/accounts/:id` |
| Query params | camelCase | `?companyId=1&pageSize=50` |

**Code Rust :**

| Element | Convention | Exemple |
|---|---|---|
| Structs/Enums | PascalCase | `JournalEntry`, `AccountType` |
| Fonctions/methodes | snake_case | `create_entry`, `validate_iban` |
| Modules | snake_case | `journal_entries`, `bank_import` |
| Fichiers | snake_case | `journal_entries.rs` |
| Newtypes | PascalCase | `Iban(String)`, `CheNumber(String)`, `Money(Decimal)` |

**Code Svelte/TypeScript :**

| Element | Convention | Exemple |
|---|---|---|
| Composants | PascalCase | `JournalEntryForm.svelte` |
| Fichiers routes SvelteKit | kebab-case | `routes/journal-entries/+page.svelte` |
| Fonctions/variables | camelCase | `fetchEntries()`, `accountList` |
| Types/interfaces | PascalCase | `JournalEntry`, `ApiError` |
| Stores | camelCase | `currentCompany`, `journalEntries` |

**JSON API :**

| Element | Convention | Exemple |
|---|---|---|
| Champs | camelCase | `companyId`, `fiscalYearStart` |
| Dates | ISO 8601 | `"2026-04-02"` |
| Montants | string decimale | `"1234.56"` (jamais de float) |

**Serde Rust → JSON :** Utiliser `#[serde(rename_all = "camelCase")]` sur toutes les structs API pour la conversion automatique snake_case → camelCase.

### Outils recommandes

- **cargo-nextest** : execution de tests plus rapide que `cargo test`
- **cargo-deny** : audit licences et securite des dependances
- **cargo-udeps** : detection de dependances inutilisees

Installation :
```bash
cargo install cargo-nextest cargo-deny cargo-udeps
```

### Versions technologiques verifiees (avril 2026)

| Technologie | Version | Usage |
|---|---|---|
| Axum | 0.8.x | Framework HTTP Tokio |
| SQLx | 0.8.6 | Driver MySQL/MariaDB async |
| rust_decimal | 1.39.0 | Arithmetique financiere exacte |
| fluent-bundle | 0.16.x | Internationalisation Fluent |
| tower-http | 0.5.x | Middleware CORS, compression, tracing, ServeDir |
| Svelte | 5.55.x | Framework reactif |
| SvelteKit | 2.55.x | Framework applicatif (mode SPA) |
| Playwright | 1.57.x | Tests E2E |
| MariaDB | 11.4 LTS | Base de donnees (support jusqu'en 2029) |

### Organisation des tests

- **Rust** : tests unitaires co-localises (`#[cfg(test)] mod tests`), tests d'integration dans `crates/{crate}/tests/`
- **Svelte** : tests co-localises (`Component.test.ts` a cote de `Component.svelte`)
- **E2E Playwright** : `frontend/tests/e2e/`

### Architecture de serving

```
Prod :  Navigateur → (nginx TLS optionnel) → Axum :3000 (SPA + API /api/v1/*)
Dev :   Navigateur → Vite :5173 (hot reload) → proxy /api/* → Axum :3000
```

- Axum sert tout en production : fichiers statiques SPA via `tower-http::ServeDir` + API REST
- TLS non gere par Kesh : HTTP pur, TLS = infrastructure (nginx/Traefik/Caddy)
- Docker-compose : 2 containers (kesh + mariadb), nginx optionnel

### Initialisation shadcn-svelte

Pour cette story, seul `npx shadcn-svelte@latest init` est executé. Les composants spécifiques (Button, Input, Select, Table, Dialog, Toast, Tooltip, DropdownMenu) seront importés dans les stories qui les utilisent (Story 1.10 pour le layout, Story 3.2 pour les formulaires, etc.)

### References

- [Source: architecture.md - Section "Structure Complete du Repertoire"]
- [Source: architecture.md - Section "Evaluation Starter Template" et "Commandes d'Initialisation"]
- [Source: architecture.md - Section "Naming Patterns"]
- [Source: architecture.md - Section "Outils Recommandes"]
- [Source: architecture.md - Section "Dependances croisees"]
- [Source: architecture.md - Section "Frontieres Architecturales"]
- [Source: epics.md - Section "Story 1.1 : Scaffold Cargo workspace & SvelteKit"]
- [Source: prd.md - FR1 (docker-compose < 15 min), FR2 (env vars), FR3 (admin via env)]

## Dev Agent Record

### Agent Model Used
(a remplir par l'agent de developpement)

### Debug Log References

### Completion Notes List

### File List
