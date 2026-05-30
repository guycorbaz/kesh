# Pipeline CI/CD Kesh

Documentation de la pipeline GitHub Actions définie dans `.github/workflows/ci.yml` et `.github/workflows/release.yml`.

## Vue d'ensemble

La pipeline `ci.yml` se déclenche sur :

- **Pull request** vers `main` → 4 jobs (`backend`, `frontend`, `e2e`, `docker-build`)
- **Push** sur `main` (= merge de PR) → mêmes 4 jobs

Aucune image Docker n'est publiée sur push `main` — la publication n'a lieu **que** sur push de tag `v*.*.*` (via `release.yml`). Le job `docker-build` du `ci.yml` build l'image en mode sanity (sans push) pour valider que le `Dockerfile` compile.

La pipeline `release.yml` se déclenche **uniquement** sur push de tag `v*.*.*` et publie une image Docker Hub SemVer + une GitHub Release.

### Schéma d'exécution

```
[backend]         [frontend]         [docker-build]   ← sanity, no push
   |                  |
   +---------+--------+
             |
           [e2e]
```

| Job | Trigger | Timeout | Dépendances |
|---|---|---|---|
| `backend` | push, PR | 30 min | — |
| `frontend` | push, PR | 20 min | — |
| `e2e` | push, PR | 30 min | `backend`, `frontend` |
| `docker-build` (sanity, no push) | push, PR | 20 min | — |
| `docker` (release) | push tag `v*.*.*` | (défaut) | — |

## Reproduction locale

Toutes les commandes du CI peuvent être reproduites en local. Pré-requis : MariaDB 10.11 démarré et accessible via `DATABASE_URL` (alignement Story 10-1 D3 — parité prod NAS Synology Package Center DSM).

### Backend (Rust)

```bash
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1
```

**Pourquoi `-j1 --test-threads=1`** : les tests partagent un état global (i18n, locale d'erreurs) et l'isolation cross-binary SQLx exige une exécution séquentielle pour éviter `PoolTimedOut`. Cf. mémo projet `feedback_sqlx_mysql_gotchas`.

### Frontend (Svelte)

```bash
cd frontend
npm ci
npm run check       # svelte-check
npm run test:unit   # Vitest
npm run build       # Vite build
```

### E2E (Playwright)

Nécessite un backend démarré sur `localhost` (port défaut 80 ; override par `KESH_PORT` dans `.env`).

```bash
cd frontend
npx playwright install --with-deps chromium
npm run test:e2e
```

Locale Playwright forcée à `fr-CH` + TZ `Europe/Zurich` dans `playwright.config.ts` — ne pas modifier (évite la flakiness multi-locale, cf. D4 review pass 1 G2 D Story 5-4).

## Politique accessibilité (axe-core)

Chaque spec Playwright **majeur** doit inclure au moins un test `axe-core` qui assert `results.violations` est vide.

### Specs avec couverture axe (6/12)

| Spec | Page testée | Story d'origine |
|---|---|---|
| `auth.spec.ts` | `/login`, layout principal | 1.11 |
| `users.spec.ts` | `/users` (liste, formulaire) | 1.12 |
| `contacts.spec.ts` | `/contacts` | 6.1 |
| `products.spec.ts` | `/products` | 6.1 |
| `invoices.spec.ts` | `/invoices` (empty state — voir D-6-1-D) | 6.1 |
| `homepage-settings.spec.ts` | `/` (homepage) | 6.1 |

### Règle pour tout nouveau spec majeur

```ts
import AxeBuilder from '@axe-core/playwright';

test('axe-core sans violations sur <page>', async ({ page }) => {
    await page.goto('/<page>');
    await page.waitForLoadState('networkidle');
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
});
```

### Aucune exclusion AA sans KF documentée

`AxeBuilder` est appelé **sans** `.disableRules()`. Si une violation WCAG AA est détectée :

1. **Préféré** : corriger le composant fautif dans la story courante.
2. **Sinon** : créer une KF dans `docs/known-failures.md` + GitHub issue `known-failure` + ajouter `.disableRules(['violated-id'])` avec un commentaire pointant la KF.

Aucune tolérance silencieuse.

## Stratégie de publication Docker

La publication d'images Docker se fait **uniquement** sur push de tag SemVer `v*.*.*` (via `release.yml`). Aucune image n'est publiée sur push `main` — décision tracée dans le CR #17.

| Tag | Trigger | Workflow | Mutabilité |
|---|---|---|---|
| `:{version}` (ex `:0.1.0`) | push tag `v*.*.*` | `release.yml/docker` | Immuable — release SemVer |
| `:latest` | push tag `v*.*.*` | `release.yml/docker` | Mutable — toujours la dernière release SemVer |

Le job `docker-build` du `ci.yml` build l'image en mode sanity (sans push) sur chaque PR/push afin de détecter au plus tôt toute régression du `Dockerfile`. L'image est jetée à la fin du runner.

### Rotation des secrets Docker Hub

Les secrets `DOCKERHUB_USERNAME` et `DOCKERHUB_TOKEN` sont configurés au niveau du repo GitHub. À rotater annuellement ou en cas d'exposition. Vérification : Settings → Secrets and variables → Actions.

## Timeouts et concurrency

### Timeouts par job

| Job | Timeout | Motivation |
|---|---|---|
| `backend` | 30 min | Compilation + tests `-j1` ≈ 15 min observés en local, marge cold cache CI |
| `frontend` | 20 min | Build Vite + Vitest + svelte-check ≈ 5 min |
| `e2e` | 30 min | Release build backend + Playwright chromium ≈ 15 min |
| `docker-build` | 20 min | Build multi-stage ≈ 8 min |

Les timeouts évitent qu'un test gelé (typiquement `PoolTimedOut` SQLx cross-binary) mobilise un runner pendant 6 h (défaut GitHub Actions).

### Concurrency

```yaml
# ci.yml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

# release.yml
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false
```

- `ci.yml` annule le run précédent si un nouveau push survient sur la même ref (branche ou PR) — évite la file d'attente sur runners gratuits.
- `release.yml` garde `cancel-in-progress: false` — on ne veut **jamais** annuler une release SemVer en cours, même si deux tags partent en rafale (ex `v0.1.0` puis `v0.1.1`). Garantit publication séquentielle.

## Permissions

`ci.yml` déclare `permissions: { contents: read }` au niveau workflow (principle of least privilege). Aucun scope d'écriture nécessaire (pas de commit back, pas d'issue comment).

`release.yml` garde `contents: write` (nécessaire pour créer une GitHub Release via `softprops/action-gh-release`).

## Migrations SQLx

Les migrations sont appliquées à **quatre** endroits (idempotent) :

1. **CI explicit** dans le job `backend` : `cargo sqlx migrate run` (via `cargo-binstall` pour `sqlx-cli`). **Requis** par les tests `kesh-db` qui utilisent `test_pool()` — connexion directe à la DB `kesh` (pas `sqlx::test` éphémère).
2. **CI explicit** dans le job `e2e` : `cargo sqlx migrate run`. Auto-documente le flow et fail-fast clair vs échec opaque au démarrage backend.
3. **Backend startup** : `MIGRATOR.run()` au démarrage de `kesh-api` (`crates/kesh-api/src/main.rs`).
4. **Tests `sqlx::test`** : SQLx applique automatiquement les migrations sur chaque DB ephemeral `_sqlx_test_*`.

## Seed CI (job `backend`)

Le job `backend` injecte plusieurs rows dans la DB `kesh` après les migrations. Contenu aligné avec le helper `crates/kesh-db/src/test_fixtures.rs::seed_accounting_company` (Story 6-4) :

| Table | Row | Raison |
|---|---|---|
| `companies` | `'CI Seed Company'` (org_type `Independant`, langues `FR`/`FR`) | `kesh-db::repositories::*::tests::get_company_id()` exige ≥ 1 company |
| `users` | `'admin'` (role `Admin`, hash Argon2id de `admin123`) | `get_admin_user_id()` exige ≥ 1 user `Admin` |
| `accounts` × 5 | `1000/1100` (Asset), `2000` (Liability), `3000` (Revenue), `4000` (Expense) | `two_accounts()` exige ≥ 2 comptes actifs ; couverture des 4 types pour tests journal_entries |
| `fiscal_years` | `'Exercice CI 2020-2030'` (Open) | `journal_entries::tests::setup()` exige un exercice ouvert couvrant `today` |

**Notes** :
- Le hash Argon2id est pré-calculé (cf. `ADMIN_PASSWORD_HASH` dans `test_fixtures.rs`) pour que les tests E2E qui loguent `admin/admin123` trouvent le user.
- Le fiscal_year couvre 2020-2030 délibérément large pour tolérer la dérive d'horloge entre runs CI.
- Le bootstrap admin de `kesh-api` ne peut pas remplir ce rôle car il crée un user mais **pas** de company (la company passe par le flow onboarding, hors scope d'un seed CI).

## Fixtures E2E déterministes (Story 6-4)

Le job `e2e` active `KESH_TEST_MODE=true` pour exposer `POST /api/v1/_test/seed` avec 4 presets (`fresh`, `with-company`/`post-onboarding`, `with-data`). Le helper Playwright `frontend/tests/e2e/helpers/test-state.ts` appelle cet endpoint avant chaque scénario pour garantir un état de DB reproductible.

Garde-fou sécurité : le backend refuse de démarrer si `KESH_TEST_MODE=true` **et** `KESH_HOST != 127.0.0.1` (cf. `ConfigError::TestModeWithPublicBind`). Loopback strict requis — `0.0.0.0` rejeté.

Un smoke test `curl POST /api/v1/_test/seed` s'exécute avant Playwright pour fail-fast si la config CI est cassée.

## Décision MariaDB 10.11 (Story 10-1 D3 — parité prod NAS)

**Mise à jour 2026-05-21** (Story 10-1 D3) : alignement à `mariadb:10.11` partout (dev + CI + prod) pour parité avec la version Package Center Synology DSM 7.x du NAS de Guy (production v0.1.0). Compat MariaDB ≥ 10.6 confirmée par sanity check pre-flight (26 migrations passent sur 10.11 vierge, dont la VIRTUAL+UNIQUE column de `reconciliation_rules` Story 8-5b).

**Historique** : la pipeline CI utilisait antérieurement MariaDB 11.4 (Story 6-1 décision 2026-04-15). Le passage à 10.11 dans Story 10-1 vise à éviter tout drift entre la version testée en CI et celle utilisée en prod sur le NAS. `architecture.md` impose toujours « MariaDB 10.6+ » comme minimum — 10.11 respecte le minimum avec marge.

### Justification mono-version 10.11 (Story 10-2 AC #20)

La pipeline CI utilise **un seul service MariaDB** en version 10.11 — aucune matrice `strategy.matrix.mariadb-version` n'est introduite. Trois raisons :

- **Cible prod unique** : le NAS Synology Package Center DSM ≥ 7.2 ne propose que MariaDB 10.x stable (10.11 spécifiquement). Une matrice qui inclurait 11.x ferait passer des tests sur un moteur que personne ne tournera en prod, sans gain de couverture.
- **Bugs 10.11-specific seraient masqués par 11** : si une feature dépend d'un comportement spécifique à MariaDB 11, elle passerait sur la branche matrice 11 mais casserait silencieusement sur 10.11 — sans matrice, le test 10.11 attrape immédiatement.
- **Compat upstream MariaDB ≥ 10.6 documentée** : la migration `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:27-28` documente la compat 10.6+ pour les opérateurs hors NAS Synology qui voudraient tourner sur 10.6/10.7/10.8/10.9/10.10. Cette compat **n'est pas testée par la CI projet** — c'est de la documentation, pas une garantie.

## Dette technique CI

| ID | Description | Sévérité | Statut |
|---|---|---|---|
| **D-6-1-A** | Cache des browsers Playwright (`~/.cache/ms-playwright` via `actions/cache@v4`) — économie ~50s par run e2e | LOW | Reporté — bénéfice marginal solo-dev |
| **D-6-1-B** | Pinning des actions GitHub aux SHA commit (vs `@v4` etc.) — sécurité supply chain | LOW | Reporté — Dependabot peut automatiser |
| **D-6-1-C** | Couverture axe-core partielle (6/12 specs) — manquent `accounts`, `journal-entries`, `mode-expert`, `onboarding`, `onboarding-path-b`, `invoices_echeancier` | LOW | Follow-up opportuniste |
| **D-6-1-D** | Test axe `/invoices` valide l'empty state uniquement — l'état peuplé sera couvert par Story 6-4 (`seed_accounting_company`) | LOW | Partiellement résolu — Story 6-4 mergée, adoption dans le spec `invoices` à faire en follow-up |
| **D-6-1-E** | Tests Playwright bloqués post-seed (`continue-on-error: true` sur le step `Run Playwright`) | HAUTE | KF-007 / issue #19 — ⛔ blocant release v0.1, tracé pour Story 6-5 |
