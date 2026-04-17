# Tests — patterns et fixtures

Ce document décrit les deux patterns de tests utilisés dans Kesh (Rust intégration et Playwright E2E), la plomberie des fixtures déterministes introduite par la Story 6.4, et les prérequis pour lancer chaque suite en local.

## Vue d'ensemble

| Niveau | Framework | Fixtures | Localisation |
|---|---|---|---|
| Unitaires Rust | `cargo test` | aucunes (logique pure) | `crates/*/src/**/tests` |
| Intégration Rust | `cargo test` + `sqlx::test` | `kesh_db::test_fixtures::seed_accounting_company` | `crates/kesh-api/tests/*_e2e.rs` |
| Unitaires frontend | Vitest | mocks | `frontend/tests/**/*.test.ts` |
| E2E Playwright | `@playwright/test` | endpoint `POST /api/v1/_test/seed` via `seedTestState` | `frontend/tests/e2e/*.spec.ts` |

La Story 6.4 a unifié deux patterns disparates (bypass SQL ad-hoc en Rust, absence totale de reset DB en Playwright) en une seule couche de fixtures partagée par les deux.

## Pattern Rust : `seed_accounting_company`

Chaque test intégration backend démarre d'une DB éphémère (fournie par `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`) puis seede l'état comptable via le helper :

```rust
use kesh_db::test_fixtures::seed_accounting_company;
use kesh_db::repositories::invoices;

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn my_invoice_flow(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    // seeded : company_id, fiscal_year_id, admin_user_id, changeme_user_id,
    //          accounts: HashMap<"1000"|"1100"|"2000"|"3000"|"4000", i64>

    let (invoice, _) = invoices::create(&pool, seeded.admin_user_id, ...).await.unwrap();
    // Valider via le flow normal (pas d'UPDATE SQL direct — KF-001 closed).
    invoices::validate_invoice(&pool, seeded.company_id, invoice.id, seeded.admin_user_id)
        .await.expect("validate");
}
```

**Ce que le helper crée** :

- 1 `companies` `'CI Test Company'`, org_type `Independant`, langues FR/FR, adresse 2 lignes (`'Test Address 1\n1000 Lausanne'` — QR Bill exige line1/line2).
- 2 `users` Admin actifs : `admin/admin123` et `changeme/changeme` (hashes Argon2id pré-calculés dans `ADMIN_PASSWORD_HASH` / `CHANGEME_PASSWORD_HASH`).
- 1 `fiscal_years` 2020-2030 `Open`.
- 5 `accounts` minimaux : 1000 Caisse (Asset), 1100 Banque (Asset), 2000 Capital (Liability), 3000 Ventes (Revenue), 4000 Charges (Expense).
- 1 `company_invoice_settings` avec `default_receivable_account_id` = 1100, `default_revenue_account_id` = 3000, `default_sales_journal` = `Ventes`.

Les helpers associés (`truncate_all`, `seed_changeme_user_only`, `mark_onboarding_complete`, `seed_contact_and_product`) exposent les briques utilisées par l'endpoint runtime ci-dessous.

Lancer la suite :

```bash
# DB MariaDB de dev démarrée (docker compose -f docker-compose.dev.yml up -d mariadb)
DATABASE_URL="mysql://root:kesh_dev_root@127.0.0.1:3306/kesh" cargo test -p kesh-api --tests
```

## Pattern Playwright : `seedTestState` via endpoint gated

Playwright partage une seule DB MariaDB entre toutes les specs — pas d'équivalent de `sqlx::test` éphémère per-test. La solution Story 6.4 : un endpoint **runtime** `POST /api/v1/_test/seed` qui truncate la DB puis re-seed, exposé **uniquement si `KESH_TEST_MODE=true`** dans l'env du backend.

```ts
// frontend/tests/e2e/homepage-settings.spec.ts
import { test, expect } from '@playwright/test';
import { seedTestState } from './helpers/test-state';

test.beforeAll(async () => {
  await seedTestState('with-company');
});

test('homepage affiche la company seedée', async ({ page }) => {
  await page.goto('/login');
  await page.fill('#username', 'admin');
  await page.fill('#password', 'admin123');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/');
});
```

### Les 4 presets

| Preset | Contenu | Usage typique |
|---|---|---|
| `fresh` | DB vidée, user `changeme/changeme` seul | `onboarding.spec.ts`, `onboarding-path-b.spec.ts` |
| `post-onboarding` | fresh + `admin/admin123` + company + fiscal_year + 5 accounts + company_invoice_settings + `onboarding_state.step_completed = 10` | alias sémantique de `with-company` |
| `with-company` | **identique** à `post-onboarding` (même code path) | toutes les specs admin (`auth`, `accounts`, `contacts`, `products`, `invoices`, `journal-entries`, `users`, `homepage-settings`, `mode-expert`) |
| `with-data` | `with-company` + 1 contact `'CI Contact SA'` + 1 product `'CI Product'` | `invoices_echeancier.spec.ts` (les factures sont créées dynamiquement par le spec — **pas de facture pré-seedée**) |

### Choix `beforeAll` vs `beforeEach`

| Cas | Hook | Raison |
|---|---|---|
| State singleton muté par chaque test (ex: `onboarding_state.step_completed`) | `beforeEach` | Chaque test doit re-partir de zéro, la mutation est irréversible dans le run. |
| Mutations scopées à des rows individuelles (contact, product, etc.) | `beforeAll` | Les tests utilisent des suffixes uniques (`Date.now()`) — pas de collision. |

## Sécurité : le gate `KESH_TEST_MODE`

L'endpoint `/api/v1/_test/*` est une porte ouverte sur la DB complète (truncate + re-seed). Sa sécurité repose sur **trois couches** :

1. **Gate runtime dans `build_router`** : les routes ne sont montées que si `config.test_mode == true`. Une requête POST vers `/api/v1/_test/seed` avec `test_mode=false` retombe sur le fallback `ServeDir` → `404 Not Found` ou `405 Method Not Allowed` (jamais 200). Vérifié par les tests `test_endpoints_e2e::seed_endpoint_not_available_when_test_mode_off`.
2. **Refus de démarrage si bind non-loopback** : `Config::from_env()` retourne `ConfigError::TestModeWithPublicBind` si `KESH_TEST_MODE=true` **et** `KESH_HOST ∉ {127.0.0.1, ::1, localhost}`. Le binaire exit 1 avec un message explicite avant même d'écouter.
3. **`0.0.0.0` explicitement rejeté** : pas accepté comme alias loopback. Raison : en Docker avec `-p 3000:3000`, un container qui bind `0.0.0.0` en interne expose le port sur le réseau hôte — l'endpoint `/api/v1/_test/*` deviendrait accessible publiquement. La CI et `docker-compose.dev.yml` **doivent** utiliser `KESH_HOST=127.0.0.1` quand `KESH_TEST_MODE=true`.

Le défaut applicatif de `KESH_HOST` est passé de `0.0.0.0` à `127.0.0.1` (Story 6.4 T7.6) — sécurité par défaut, opt-in explicite pour bind public en prod.

## Prérequis Playwright local

Le `webServer` de `playwright.config.ts` démarre uniquement **le frontend** (`npm run preview` sur `:4173`). Le backend `kesh-api` **n'est pas lancé automatiquement** — il faut le démarrer séparément.

**Recipe minimale** (dans 2 terminaux) :

```bash
# Terminal 1 : DB + backend
docker compose -f docker-compose.dev.yml up -d mariadb
cd /path/to/kesh
KESH_TEST_MODE=true KESH_HOST=127.0.0.1 \
  DATABASE_URL="mysql://root:kesh_dev_root@127.0.0.1:3306/kesh" \
  KESH_JWT_SECRET="dev-secret-at-least-32-bytes-long-for-testing" \
  cargo run -p kesh-api

# Terminal 2 : Playwright
cd frontend
npm run test:e2e
```

Le `globalSetup` Playwright (`tests/e2e/global-setup.ts`) appelle `seedTestState('with-company')` une seule fois avant tous les workers — si le backend est éteint ou `KESH_TEST_MODE=false`, il throw avec un message listant les 4 prérequis (backend up, `KESH_TEST_MODE`, `KESH_HOST` loopback, `KESH_BACKEND_URL`).

Pour surcharger l'URL backend (ex: tests contre un kesh-api distant) :

```bash
KESH_BACKEND_URL=http://localhost:3001 npm run test:e2e
```

## Cleanup entre tests (dette technique acceptée)

- **Pas de reset entre tests individuels d'une même spec** (dette `D-6-4-A`). Si un test pollue (création + archivage incomplet), le test suivant peut être affecté. Mitigation : convention de cleanup explicite dans chaque test, ou adoption progressive de `test.beforeEach(seedTestState(...))` si symptômes apparaissent.
- **Pas de tests d'intégration de l'endpoint depuis le helper TypeScript** (dette `D-6-4-B`). Chicken-and-egg : le helper teste lui-même. La couverture Rust (`test_endpoints_e2e`) + le `globalSetup` Playwright suffisent pour détecter une régression.

## Références

- Story 6.4 : `_bmad-output/implementation-artifacts/6-4-fixtures-e2e-deterministes.md`
- Helper Rust : `crates/kesh-db/src/test_fixtures.rs`
- Endpoint runtime : `crates/kesh-api/src/routes/test_endpoints.rs`
- Helper Playwright : `frontend/tests/e2e/helpers/test-state.ts`
- Tests d'intégration de l'endpoint : `crates/kesh-api/tests/test_endpoints_e2e.rs`
- CI : `.github/workflows/ci.yml` (job `e2e`, step `Smoke test /api/v1/_test/seed`)
