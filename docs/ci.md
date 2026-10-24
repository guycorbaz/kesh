# Pipeline CI/CD Kesh

Documentation de la pipeline GitHub Actions définie dans `.github/workflows/ci.yml` et `.github/workflows/release.yml`.

## Vue d'ensemble

La pipeline `ci.yml` se déclenche sur :

- **Pull request** vers `main` → 4 jobs (`backend`, `frontend`, `docker-build`, `e2e-smoke`)
- **Push** sur `main` (= merge de PR) → mêmes 4 jobs

Aucun filtre `paths:` : les quatre jobs tournent à chaque push, même sur un commit de documentation seule.

*(Jusqu'à la Story 31-2, ce document décrivait un job `e2e` complet qui n'avait jamais existé dans `ci.yml` — issue #577. Il décrit désormais le dépôt ; les décisions D-6-1-* relatives à ce job sont annotées « non retenu » en fin de document.)*

Aucune image Docker n'est publiée sur push `main` — la publication n'a lieu **que** sur push de tag `v*.*.*` (via `release.yml`). Le job `docker-build` du `ci.yml` valide les deux compose distribués (`docker compose config -q` sur `docker-compose.yml` et `docker-compose.prod.yml` — clé inconnue ou en double, interpolation mal formée) — `docker-compose.yml` dans les **deux sens** : il doit se lire avec des mots de passe MariaDB factices, et **refuser** sans eux en nommant `MARIADB_ROOT_PASSWORD` ou `MARIADB_PASSWORD` (Story 15-13a, #551) — puis build l'image en mode sanity (sans push) pour valider que le `Dockerfile` compile.

La pipeline `release.yml` se déclenche **uniquement** sur push de tag `v*.*.*` et publie une image Docker Hub SemVer + une GitHub Release.

### Schéma d'exécution

```
[backend]         [frontend]         [docker-build]   ← sanity, no push
                                           |
                                      [e2e-smoke]     ← image de la PR, une spec Playwright
```

| Job (nom affiché) | Trigger | Timeout | Dépendances | Vérification requise de `main` |
|---|---|---|---|---|
| `backend` (Backend (Rust)) | push, PR | 120 min | — | oui |
| `frontend` (Frontend (Svelte)) | push, PR | 20 min | — | oui |
| `docker-build` (Docker build (sanity)) | push, PR | 20 min | — | oui |
| `e2e-smoke` (E2E smoke (Playwright)) | push, PR | 25 min | `docker-build` | **non** — décision de Guy à prendre |
| `docker` (release) | push tag `v*.*.*` | (défaut) | — | — |

### Le smoke E2E — job `e2e-smoke` (Story 31-2, #577)

Il vérifie que **le harnais Playwright est vivant**, pas les fonctionnalités : une seule spec,
`frontend/tests/e2e/smoke.spec.ts`, deux tests — connexion par l'interface puis `GET /api/v1/auth/me` par
`authedApiContext` (200, nom `admin`), et la même route **sans** cookie (401 ; sans cette sonde négative, le 200
pourrait venir d'une route ouverte). C'est le mode d'échec de l'issue #285 : un contexte API sans cookie rendait 401
partout, et rien ne signalait une suite qui cessait de tourner. La **suite complète** reste locale, avant tout push
(`CLAUDE.md` § E2E).

Montage : connexion Docker Hub, `setup-buildx`, reconstruction de l'image de la PR depuis le cache `gha` laissé par
`docker-build` (`load: true`), puis `docker run --network=host` avec `KESH_TEST_MODE=true`, `KESH_HOST=127.0.0.1` (le
mode test refuse un bind non-loopback), `KESH_COOKIE_SECURE=false` (HTTP), `KESH_PORT=3000` et
`DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` — le service MariaDB du job, utilisateur `kesh` aux droits de
la production. Les migrations s'appliquent au démarrage du backend ; le seed est celui du backend
(`POST /api/v1/_test/seed`, appelé par `seedTestState`). Attente de `/health` en boucle (journal du conteneur affiché
si elle échoue), puis `npm ci`, `npx playwright install --with-deps chromium` et
`npx playwright test tests/e2e/smoke.spec.ts`. En cas d'échec : journal du backend, et `frontend/test-results/`
(traces) téléversé, rétention 3 jours.

Il exerce aussi, **sur chaque PR**, le chemin « image + utilisateur `kesh` + migrations au démarrage » que reprend le
smoke `/health` de `release.yml`. **Durée mesurée au premier run** (PR #702, 2026-10-23) : **16 min 29 s** — ≈ 10 min pour
`npx playwright install --with-deps chromium`, ≈ 5 min de construction de l'image depuis le cache `gha`, ≈ 4 s pour la
spec. Au-delà des 10 min visées : le cache des navigateurs (D-6-1-A) est le levier, à mettre en place.

**Repli** si le cache `gha` ne tient pas (reconstruction complète de l'image à chaque run) : un job qui compile
`kesh-api` et télécharge l'artefact `frontend-build` — écarté d'abord (C-31-2-2) parce qu'il ajoute un second
`cargo build` pour tester un binaire de débogage, qui n'est pas ce qu'on publie.

### L'utilisateur MariaDB de la CI : `kesh`, pas `root` (Story 31-2)

Le job `backend` applique `scripts/mariadb-init/01-dev-grants.sql` en `root` (étape « Apply dev grants », après
« Install mariadb-client », `-h 127.0.0.1`), puis emploie `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` :
`kesh` a exactement les droits du conteneur de dev — `ALL` sur `kesh`.*, `\_sqlx\_test%`.* et `kesh%`.*, aucun droit
global. Migrations et seed restent en `root` : c'est de l'outillage, pas le chemin de l'application. Les jobs
`e2e-smoke` et `release.yml` emploient le `kesh` que crée le service (`ALL` sur `kesh`.*, les droits de la production).

Pourquoi : une CI en `root` fait ce que faisait un conteneur de dev trop large — elle laisse passer un test qui dépend
d'un privilège que la production n'a pas (Stories 29-4 et 29-8, `INNODB_TRX`). Table des droits par environnement :
`docs/testing.md` § « Base de dev jetable ».

⚠️ **Le smoke de `release.yml` ne se vérifie qu'à la publication** : un échec y bloque la release avant le push de
l'image. Le risque est borné par `e2e-smoke`, qui exerce sur chaque PR la même image avec le même utilisateur. Premier
smoke de publication en utilisateur `kesh` (#698) : tag `v0.15.0`, 2026-10-24, **vert** (run 40080415340).

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

La CI ne joue que le smoke (`e2e-smoke`) ; la **suite complète** se joue en local avant tout push. Le montage —
backend `cargo run -p kesh-api` qui sert le frontend buildé, `KESH_TEST_MODE=true`, `KESH_COOKIE_SECURE=false`,
`KESH_BACKEND_URL` — est la recette de `docs/testing.md` § « Prérequis Playwright local », qui fait foi.

```bash
cd frontend
npx playwright install --with-deps chromium
npm run test:e2e                                   # suite complète
npx playwright test tests/e2e/smoke.spec.ts        # le seul smoke, comme en CI
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

Le job `docker-build` du `ci.yml` valide d'abord les deux compose distribués (`docker compose config -q` ; pour `docker-compose.yml`, avec des mots de passe MariaDB factices, puis l'échec **exigé** sans eux), puis build l'image en mode sanity (sans push) sur chaque PR/push afin de détecter au plus tôt toute régression du `Dockerfile`. L'image est jetée à la fin du runner. La **liste** des variables transmises par les compose est gardée par le test Rust `configuration_transmise` (job `backend`, Story 15-11a).

### Docker Hub : tirages authentifiés, secrets requis

Depuis la Story 31-2 (action A2 de la rétrospective de l'Epic 15), **tout tirage depuis Docker Hub est authentifié** par
les secrets `DOCKERHUB_USERNAME` et `DOCKERHUB_TOKEN`, configurés au niveau du dépôt GitHub (Settings → Secrets and
variables → Actions ; à renouveler chaque année ou en cas d'exposition). Auparavant, le service `mariadb:10.11` était
tiré **anonymement**, et un `429` de Docker Hub faisait échouer le job `backend` sans rapport avec le diff.

| Où | Ce qui est tiré | Comment c'est authentifié |
|---|---|---|
| `ci.yml`, job `backend` | service `mariadb:10.11` | `services.mariadb.credentials` |
| `ci.yml`, job `e2e-smoke` | service `mariadb:10.11` ; images de base du `Dockerfile` ; `moby/buildkit` | `credentials` + `docker/login-action` |
| `ci.yml`, job `docker-build` | images de base du `Dockerfile` (`rust`, `node`, `debian`) ; `moby/buildkit` | `docker/login-action` |
| `release.yml`, job `docker` | service `mariadb:10.11` ; images de base ; `moby/buildkit` ; puis le push | `credentials` + `docker/login-action` |

- **La connexion précède `docker/setup-buildx-action`** dans les trois jobs qui construisent : le builder
  `docker-container` tire `moby/buildkit` depuis Docker Hub à cette étape. ⚠️ Que ce tirage profite de la connexion est
  une **hypothèse** sur le comportement de l'action, à constater au journal du premier run (tirage authentifié ou non) —
  non vérifiée localement.
- **Les PR issues de forks et de Dependabot n'ont pas accès aux secrets** : elles échouent au démarrage du service ou à
  l'étape de connexion — échec **bruyant**, jamais un faux vert. Aucun contournement conditionnel n'est écrit (C-31-2-3 :
  dépôt tenu par un seul développeur, sans fork actif ni `dependabot.yml`). Si le dépôt accepte un jour des PR externes,
  le repli noté est `image: mirror.gcr.io/library/mariadb:10.11`, sans secret — non adopté, la provenance de l'image n'étant
  plus celle du Docker Hub officiel.
- **Un jeton invalide bloque les fusions.** La branche protégée `main` exige **trois** vérifications
  (`gh api repos/guycorbaz/kesh/branches/main/protection/required_status_checks` : `Backend (Rust)`, `Docker build
  (sanity)`, `Frontend (Svelte)`) ; les deux premières dépendent du jeton (service `credentials`, connexion). Un jeton
  expiré se voit donc à la première PR qui suit, par un rouge de démarrage — renouveler le secret, ne pas relancer.

## Timeouts et concurrency

### Timeouts par job

| Job | Timeout | Motivation |
|---|---|---|
| `backend` | 120 min | Compilation + `cargo test -j1 --test-threads=1` ; 19 min 14 s mesurées sur runner le 2026-08-16 (PR #311). La marge est délibérée : un timeout se dimensionne sur le pire cas (cf. commentaire de `ci.yml`) |
| `frontend` | 20 min | Build Vite + Vitest + svelte-check ≈ 5 min |
| `docker-build` | 20 min | Build multi-stage ≈ 8 min |
| `e2e-smoke` | 25 min | Image depuis le cache `gha` + installation de chromium + une spec ; 16 min 29 s au premier run (PR #702) |

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

Les migrations sont appliquées à **trois** endroits :

1. **CI explicite** dans le job `backend` (étape « Apply migrations to kesh DB ») : une boucle `mariadb <` en `root` sur `crates/kesh-db/migrations/*.sql`. **Requis** par les tests `kesh-db` qui utilisent `test_pool()` — connexion directe à la DB `kesh` (pas `sqlx::test` éphémère). ⚠️ Cette boucle ne remplit pas `_sqlx_migrations` : la base `kesh` du job ne sert qu'à ces tests, aucun backend n'y démarre.
2. **Démarrage du backend** : `MIGRATOR.run()` au démarrage de `kesh-api` (`crates/kesh-api/src/main.rs`) — c'est ainsi que le job `e2e-smoke` et le smoke de `release.yml` migrent leur base, comme en production.
3. **Tests `sqlx::test`** : SQLx applique les migrations (ou le squash `test-schema/`) sur chaque base éphémère `_sqlx_test_*`.

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

Le job `e2e-smoke` (comme le montage local) active `KESH_TEST_MODE=true` pour exposer `POST /api/v1/_test/seed` et ses presets (`fresh`, `with-company`/`post-onboarding`, `with-data`, …). Le helper Playwright `frontend/tests/e2e/helpers/test-state.ts` appelle cet endpoint avant chaque scénario pour garantir un état de DB reproductible.

Garde-fou sécurité : le backend refuse de démarrer si `KESH_TEST_MODE=true` **et** `KESH_HOST != 127.0.0.1` (cf. `ConfigError::TestModeWithPublicBind`). Loopback strict requis — `0.0.0.0` rejeté.

Le fail-fast avant les specs est le `globalSetup` de Playwright (`frontend/tests/e2e/global-setup.ts`), qui appelle `seedTestState('with-company')` une fois et liste les prérequis si l'endpoint ne répond pas ; aucune étape `curl` ne le précède dans le job.

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
| **D-6-1-A** | Cache des browsers Playwright (`~/.cache/ms-playwright` via `actions/cache@v4`) — économie ~50s par run e2e | LOW | Reporté — bénéfice marginal solo-dev ; à reconsidérer si `e2e-smoke` dépasse 10 min (Story 31-2) |
| **D-6-1-B** | Pinning des actions GitHub aux SHA commit (vs `@v4` etc.) — sécurité supply chain | LOW | Reporté — Dependabot peut automatiser |
| **D-6-1-C** | Couverture axe-core partielle (6/12 specs) — manquent `accounts`, `journal-entries`, `mode-expert`, `onboarding`, `onboarding-path-b`, `invoices_echeancier` | LOW | Follow-up opportuniste |
| **D-6-1-D** | Test axe `/invoices` valide l'empty state uniquement — l'état peuplé sera couvert par Story 6-4 (`seed_accounting_company`) | LOW | Partiellement résolu — Story 6-4 mergée, adoption dans le spec `invoices` à faire en follow-up |
| **D-6-1-E** | Tests Playwright bloqués post-seed (`continue-on-error: true` sur le step `Run Playwright`) | HAUTE | KF-007 / issue #19 — ⛔ blocant release v0.1, tracé pour Story 6-5 |

*(Annotation du 2026-10-23, Story 31-2 : D-6-1-A et D-6-1-E décrivent un job `e2e` complet en CI — **non retenu** : ce job n'a jamais figuré dans `ci.yml` (#577). La CI joue le seul smoke `e2e-smoke` ; D-6-1-A s'y applique, D-6-1-E est sans objet.)*
