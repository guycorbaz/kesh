# Story 6.4 : Fixtures E2E déterministes (Rust + Playwright)

Status: in-progress

<!-- Note : story créée mid-cascade lors du dev de Story 6-1 (PR #16). Voir Change Log pour le contexte. Validation `validate-create-story` recommandée avant `dev-story`. -->

## Story

As a **développeur (Guy, mainteneur solo)**,
I want **des fixtures E2E déterministes pour Rust ET Playwright qui permettent à chaque suite de tests de partir d'un état comptable connu et reproductible**,
so that **les tests E2E ne soient ni dépendants de l'ordre d'exécution, ni du bricolage SQL inline, ni d'un état accumulé entre runs — débloquant Story 6-1 (CI gate verte) et fermant définitivement KF-001**.

### Contexte

**Story créée mid-cascade pendant Story 6-1** (PR #16, 2026-04-16). En activant la branch protection avec `E2E (Playwright)` comme required check, on a découvert que les tests Playwright ne peuvent pas tous passer dans la même session : certains exigent `onboarding_state.step_completed = 0` (specs `onboarding`, `onboarding-path-b`), d'autres exigent `step >= 3` + une company configurée (specs `accounts`, `contacts`, `products`, `invoices`, `journal-entries`, `users`, `homepage-settings`, `mode-expert`).

`onboarding_state` étant un **singleton system-wide** (1 seule row par DB), aucun pré-seed statique ne peut satisfaire les deux groupes simultanément.

**Origine planning** : Story 6-4 était initialement scopée pour les tests Rust intégration uniquement (`invoice_pdf_e2e`, `invoice_echeancier_e2e`) — voir `epics.md#Story-6.4`. La découverte du problème Playwright impose d'élargir le scope.

**Scope étendu vs epics.md** : on garde l'AC original (helper `seed_accounting_company` Rust + fermeture KF-001) ET on ajoute la couche Playwright (reset DB entre specs + helpers de seeding).

### Bloque actuellement

- **Story 6-1 PR #16** — 8 commits, 72/84 tests Playwright en échec, ne peut pas merger sous branch protection sans cette story
- **KF-001** (`invoice_pdf_e2e`) reste « closed » statiquement mais le bypass SQL existe encore dans `seed_validated_invoice_via_sql`

### État actuel

**Tests Rust** (10 fichiers dans `crates/kesh-api/tests/`) — utilisent `sqlx::test` (DB éphémère per-test, OK isolation) mais avec du bricolage SQL inline pour seeder l'état comptable :
- `invoice_pdf_e2e.rs` : `seed_validated_invoice` refondu avec INSERT SQL direct + UPDATE status (cf. KF-001)
- `invoice_echeancier_e2e.rs` : `create_validated_invoice_via_sql` similaire

**Tests Playwright** (12 fichiers dans `frontend/tests/e2e/`) — partagent **une seule MariaDB** entre toutes les specs :
- 4 specs admin (`accounts`, `contacts`, `products`, `invoices`, `journal-entries`, `users`, `homepage-settings`, `mode-expert`, `invoices_echeancier`) → expectent `/` après login = nécessite onboarding_state.step ≥ 3 + company configurée
- 3 specs onboarding (`onboarding`, `onboarding-path-b`, et le `Settings` d'`homepage-settings`) → expectent `/onboarding` après login = nécessite onboarding_state.step = 0
- `auth.spec.ts` n'a pas de dépendance d'état (login + axe)

**Pré-seed CI actuel (Story 6-1)** : insère 1 company + 2 accounts + 1 fiscal_year + 2 users (admin + changeme) — mais **n'insère pas onboarding_state**. Conséquence : default = step=0 = redirect /onboarding sur tous les login admin.

### Scope verrouillé — ce qui DOIT être fait

#### Volet 1 — Helper Rust `seed_accounting_company` (scope original epics.md)

1. **Crée un module `kesh-db::test_fixtures`** (alternative discutable : crate dédié `kesh-test-fixtures` — décision en T0)
2. **Helper `seed_accounting_company(pool: &MySqlPool) -> SeededCompany`** retourne struct avec `{ company_id, fiscal_year_id, admin_user_id, receivable_account_id, revenue_account_id, sales_journal_id }`
3. **Contenu seedé** :
   - 1 company (org_type Independant, langues FR/FR)
   - 1 fiscal_year 2020-2030 status `Open`
   - 1 user `Admin` (avec hash Argon2id réel ou pré-calculé)
   - Plan comptable minimal (au moins comptes 1000 Caisse, 1100 Banque, 2000 Capital, 3000 Ventes, 4000 Charges) — toggle pour seeder le plan KMU complet
   - 1 row `company_invoice_settings` avec `default_receivable_account_id` (compte 1100 ou similaire), `default_revenue_account_id` (compte 3000), `default_sales_journal` (`Banque` ou `Ventes`)
4. **Refactor des bypass SQL existants** dans `invoice_pdf_e2e.rs` + `invoice_echeancier_e2e.rs` :
   - Remplacer `seed_validated_invoice_via_sql` / `force_validate_via_sql` par : `seed_accounting_company(pool)` + appels normaux à `validate_invoice` route
   - **Plus aucun INSERT manuel ni UPDATE direct sur `invoices.status`** dans les tests
5. **Marquer KF-001 `closed` définitivement** (cf. `docs/known-failures.md`)

#### Volet 2 — Fixtures Playwright (scope ajouté par PR #16)

6. **Endpoint backend `/api/v1/_test/reset`** — **gated** par `KESH_TEST_MODE=true` (env var, refuse si non-set). Action : truncate toutes les tables sauf `_sqlx_migrations` puis re-seed minimal. Réponse `200 OK` avec body listant les rows seedées.
7. **Endpoint backend `/api/v1/_test/seed`** — POST avec body `{ "preset": "fresh" | "post-onboarding" | "with-company" | "with-data" }`. Chaque preset seede l'état approprié. Idempotent (truncate + insert).
8. **Helper Playwright `seedTestState(page, preset)`** — wrappe l'appel HTTP `/api/v1/_test/seed`, fail fast si endpoint pas exposé. Centralisé dans `frontend/tests/e2e/helpers/test-state.ts`.
9. **Convention par spec** :
   - `auth.spec.ts` → `seedTestState('with-company')` (login OK, layout chargé)
   - `accounts/contacts/products/invoices/journal-entries/users/homepage-settings/mode-expert.spec.ts` → `seedTestState('with-company')` (post-onboarding)
   - `onboarding/onboarding-path-b.spec.ts` → `seedTestState('fresh')` (step=0)
   - `invoices_echeancier.spec.ts` → `seedTestState('with-data')` (company + factures pré-existantes)
10. **Hook `test.beforeAll`** dans chaque spec : appelle `seedTestState(...)` avant le 1er test du fichier. Optionnellement `test.beforeEach` si certains tests pollutent (ex: création/archivage).
11. **CI : ajout de `KESH_TEST_MODE=true` dans le job e2e** + retrait du pré-seed SQL inline dans `ci.yml/e2e` (remplacé par `seedTestState`)
12. **Documentation `docs/testing.md`** :
    - Pattern Rust (helper + sqlx::test ephemeral)
    - Pattern Playwright (endpoint /api/v1/_test/* + helper seedTestState)
    - Liste des presets disponibles
    - Garanties de sécurité (gate `KESH_TEST_MODE`, refus en prod)

### Scope volontairement HORS story — décisions tranchées

- **Refonte des tests Rust hors `invoice_pdf_e2e` / `invoice_echeancier_e2e`** → garder leur structure existante. Le helper devient disponible mais migration progressive.
- **Multi-tenant scoping** (KF-002 / Story 6-2) → orthogonal, scope dédié.
- **Lint i18n** (Story 6-3) → orthogonal.
- **Refactor des tests Playwright pour cesser d'utiliser `changeme/changeme`** → décision : conserver les credentials existants (`admin/admin123`, `changeme/changeme`). Le seed CI doit créer les 2 users. Si simplification souhaitée → CR séparé.
- **Suppression de la dépendance des tests Playwright sur l'ordre d'exécution** → naturellement résolue par `beforeAll` qui reset l'état avant chaque spec.

### Décisions de conception

- **Endpoint `/api/v1/_test/*` vs CLI binaire** — endpoint HTTP est plus simple à invoquer depuis Playwright (pas besoin de spawn process), et le gate `KESH_TEST_MODE` empêche l'exposition prod (refus 404 ou 403 si non-set).

- **`KESH_TEST_MODE` env var vs feature flag Cargo** — env var permet de switch sans rebuild. Pertinent quand on lance le même binaire en CI (test mode) et en prod (mode normal). Cargo feature exigerait deux builds.

- **Truncate vs delete** — `TRUNCATE TABLE` est ~10x plus rapide et reset les `AUTO_INCREMENT`. Mais ne déclenche pas les triggers (pas de problème ici) et ignore les FK (utiliser `SET FOREIGN_KEY_CHECKS=0` autour). Choix : truncate avec FK désactivées le temps du reset.

- **Helper Playwright dans `frontend/tests/e2e/helpers/`** — pas dans `tests/e2e/` direct pour ne pas être détecté comme spec.

- **Presets nommés vs body riche** — préférer une enum (`fresh`, `post-onboarding`, `with-company`, `with-data`) pour limiter la combinatoire et éviter que chaque test invente son propre seed. Si besoin de flexibilité plus tard, ajouter un preset.

- **Pas de DB séparée par test Playwright** — coût trop élevé (Playwright tourne avec 2 workers en CI, multiplier les DBs deviendrait ingérable). On reset entre specs (= entre fichiers), pas entre tests individuels.

- **`onboarding_e2e.rs` Rust ne nécessite pas le helper** — il utilise `sqlx::test` éphémère et test la route onboarding directement.

- **Routing Playwright → backend (F1 review pass 1)** — Playwright tourne contre `:4173` (SvelteKit `preview`), backend sur `:3000`. Le helper `seedTestState` doit appeler le backend en URL **absolue** (`http://127.0.0.1:3000/api/v1/_test/seed`), via une `APIRequestContext` créée avec `request.newContext({ baseURL: 'http://127.0.0.1:3000' })`. Ne **pas** se reposer sur un proxy Vite en mode `preview` — comportement non documenté et fragile. La constante `BACKEND_URL` est extraite d'une env var Playwright (`process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000'`).

- **Propagation `KESH_TEST_MODE` : runtime branch, pas Cargo feature (F2 review pass 1)** — Décision tranchée : `Config::test_mode: bool` lu dans `build_router(state)`. Le merge des routes `/api/v1/_test/*` est conditionnel à `state.config.test_mode == true`. **Refus explicite** de `#[cfg(feature = "test-endpoints")]` car (a) deux builds = risque de drift, (b) test feature laissée active dans Dockerfile prod = catastrophe. La branche runtime garantit que la route n'existe simplement pas dans le router si `test_mode == false` — `404 Not Found` natif Axum.

- **Garde-fou staging : refus de démarrage si test_mode + non-loopback (F6 review pass 1, raffiné par N3 review pass 2)** — Au démarrage de `kesh-api`, si `config.test_mode == true` ET `config.host` n'est pas dans `{"127.0.0.1", "::1", "localhost"}`, le binaire **refuse de démarrer** avec un `tracing::error!` explicite. Évite qu'un opérateur déploie l'image CI en staging par erreur (var `KESH_TEST_MODE=true` héritée d'un `.env`) et expose `/api/v1/_test/seed` publiquement.

  **Décision pass 2 sur `0.0.0.0`** : `0.0.0.0` est **explicitement REJETÉ** (pas accepté comme alias loopback) car en environnement Docker en production le bind `0.0.0.0` expose réellement la route au réseau hôte. Conséquence pratique : **la CI et docker-compose.dev DOIVENT utiliser `KESH_HOST=127.0.0.1` quand `KESH_TEST_MODE=true`**. Ce choix strict prévient le scénario d'attaque : container Docker exposé via `-p 3000:3000` avec bind `0.0.0.0` à l'intérieur = port accessible depuis le réseau hôte = endpoint `/api/v1/_test/*` exposé.

  **Régression DX identifiée pass 3 (H1)** : aujourd'hui `docker-compose.dev.yml`, `.env.example` et `crates/kesh-api/src/config.rs` (défaut applicatif) utilisent `KESH_HOST=0.0.0.0`. Si un dev set juste `KESH_TEST_MODE=true` localement, le binaire refusera de démarrer. Décision tranchée pass 3 : **changer le défaut applicatif de `0.0.0.0` à `127.0.0.1`** (sécurité par défaut, opt-in explicite pour bind public en prod). Voir T7.6 pour les fichiers à mettre à jour.

- **`post-onboarding` vs `with-company` : différenciation explicite (F4 review pass 1)** — `post-onboarding` = état minimal post-onboarding (`onboarding_state.step_completed = 10` + company + admin + plan comptable + fiscal_year + company_invoice_settings + user `changeme`). `with-company` = **identique** à `post-onboarding` (alias sémantique pour les specs admin qui ne se soucient pas du nom). Décision : conserver les deux noms pour la lisibilité côté spec (`auth.spec.ts` peut documenter « with-company » plutôt que « post-onboarding » qui sème la confusion sur l'intention). Implémentation : un seul code path, deux strings de preset acceptés.

- **User `changeme` requis dans tous les presets sauf `fresh`** — `homepage-settings.spec.ts`, `onboarding.spec.ts`, `onboarding-path-b.spec.ts` utilisent `changeme/changeme`. Pour que les presets supportent ces specs, **`fresh` inclut le user `changeme` SEUL** (pas de company, pas d'admin), et `post-onboarding`/`with-company`/`with-data` incluent **les deux** users (`admin/admin123` + `changeme/changeme`).

### Dette technique acceptée — v0.2 ou plus tard

- **D-6-4-A** — Pas de reset entre tests individuels d'une même spec. Si un test pollue (création + archivage incomplet), le test suivant peut être affecté. Mitigation : convention de cleanup explicite dans chaque test, ou `test.beforeEach(seedTestState(...))` adopté progressivement si symptômes apparaissent.

- **D-6-4-B** — Endpoint `/api/v1/_test/*` pas couvert par tests d'intégration (chicken-and-egg : le helper teste lui-même). Smoke test manuel suffit pour MVP.

## Acceptance Criteria

1. **Given** le module `kesh-db::test_fixtures`, **When** importé, **Then** expose `seed_accounting_company(pool) -> SeededCompany` (struct avec tous les IDs nécessaires) ET un helper de cleanup associé.

2. **Given** un test Rust qui appelle `seed_accounting_company`, **When** exécuté, **Then** la DB contient : 1 company, 1 fiscal_year ouvert 2020-2030, ≥ 5 accounts (1000/1100/2000/3000/4000), **2 users Admin actifs** (`admin/admin123` ET `changeme/changeme`), 1 row `company_invoice_settings` avec `default_receivable_account_id` = id du compte 1100, `default_revenue_account_id` = id du compte 3000, `default_sales_journal` = `Ventes`.

3. **Given** `crates/kesh-api/tests/invoice_pdf_e2e.rs`, **When** grep `force_validate_via_sql\|UPDATE.*invoices.*status`, **Then** zéro occurrence (refactoré pour utiliser le helper + appels normaux `validate_invoice`).

4. **Given** `crates/kesh-api/tests/invoice_echeancier_e2e.rs`, **When** grep idem, **Then** zéro occurrence.

5. **Given** KF-001 dans `docs/known-failures.md`, **When** Story 6-4 done, **Then** entrée KF-001 amendée avec « status: closed (commit X) — bypass SQL retiré, helper `seed_accounting_company` utilisé ».

6. **Given** `KESH_TEST_MODE=false` (ou non-set), **When** appel `POST /api/v1/_test/reset` ou `/api/v1/_test/seed`, **Then** réponse `404 Not Found` (route non enregistrée par `build_router`) — **JAMAIS** exposée en prod.

6bis. **Given** `KESH_TEST_MODE=true` ET `KESH_HOST` ∉ `{127.0.0.1, ::1, localhost}`, **When** démarrage de `kesh-api`, **Then** le process **refuse de démarrer** avec exit code ≠ 0 et un log `tracing::error!` explicite (« KESH_TEST_MODE=true incompatible avec un bind public »). Garde-fou staging.

7. **Given** `KESH_TEST_MODE=true`, **When** `POST /api/v1/_test/seed` avec body `{"preset": "fresh"}`, **Then** DB truncate de toutes les tables sauf `_sqlx_migrations`, puis insert d'**un seul user `changeme/changeme`** (Admin, hash Argon2id réel). Aucune company, aucun account, aucun fiscal_year, aucune row `onboarding_state`. Réponse `200 OK`.

8. **Given** `KESH_TEST_MODE=true`, **When** `POST /api/v1/_test/seed` avec body `{"preset": "post-onboarding"}` (alias `with-company`), **Then** DB contient :
   - 2 users : `admin/admin123` + `changeme/changeme` (les deux Admin actifs)
   - 1 company `CI Test Company`, org_type `Independant`, langues FR/FR
   - `onboarding_state` singleton avec `step_completed = 10`, `is_demo = FALSE`, `ui_mode = 'guided'`
   - 1 fiscal_year 2020-2030 status `Open`
   - ≥ 5 accounts : 1000 Caisse (Asset), 1100 Banque (Asset), 2000 Capital (Liability), 3000 Ventes (Revenue), 4000 Charges (Expense)
   - 1 row `company_invoice_settings` : `default_receivable_account_id` = compte 1100, `default_revenue_account_id` = compte 3000, `default_sales_journal` = `Ventes`

9. **Given** `KESH_TEST_MODE=true`, **When** `POST /api/v1/_test/seed` avec body `{"preset": "with-company"}`, **Then** **strictement identique** à `post-onboarding` (alias sémantique). Le seul code path appliqué est celui décrit en AC #8.

10. **Given** `KESH_TEST_MODE=true`, **When** `POST /api/v1/_test/seed` avec body `{"preset": "with-data"}`, **Then** identique à `with-company` plus :
    - 1 contact `'CI Contact SA'` (type Entreprise, isClient = TRUE)
    - 1 product `'CI Product'` (vat_rate `8.10`, unit_price `100.00`)

    **Décision pass 3 (H3)** : **PAS de facture pré-seedée** dans ce preset. Vérification du code montre que `invoices_echeancier.spec.ts` crée ses propres factures dynamiquement via `daysFromToday()` et n'utilise jamais une facture pré-existante. Une facture pré-seedée :
    - serait du dead code (jamais référencée par les specs),
    - polluerait le tableau testé (date 2026-04-01 = passée → badge « En retard » qui interfère avec les assertions du golden path).

    Les specs qui ont besoin de factures les créent elles-mêmes via les routes normales (pattern préservé). Le preset `with-data` apporte juste contact + product pour éviter aux specs d'avoir à les créer aussi.

11. **Given** un appel à `/api/v1/_test/seed` avec body `{"preset": "invalid-name"}`, **Then** `400 Bad Request` avec message clair listant les presets valides.

12. **Given** `frontend/tests/e2e/helpers/test-state.ts`, **When** importé, **Then** expose `seedTestState(preset: 'fresh' | 'post-onboarding' | 'with-company' | 'with-data'): Promise<void>` qui crée son propre `APIRequestContext` ciblant `BACKEND_URL` (URL absolue du backend, par défaut `http://127.0.0.1:3000`) et fail si la requête HTTP retourne ≠ 200. **Note** : pas de paramètre `page` ni `request` — le helper est autonome (cf. F1 review pass 1 : évite la dépendance au proxy Vite preview).

13. **Given** chaque spec Playwright dans `frontend/tests/e2e/*.spec.ts`, **When** ouvert, **Then** le 1er bloc est `test.beforeAll(async () => { await seedTestState('<preset>'); });` avec le preset adapté à la spec (cf. liste §9 du scope) — **sauf** `onboarding.spec.ts` et `onboarding-path-b.spec.ts` qui utilisent `test.beforeEach` (cf. T6.5).

14. **Given** `npm run test:e2e -- --reporter=list` en local ou CI, **When** lancé, **Then** la sortie ne produit **aucune ligne `FAILED`** et aucun `.skip()` lié à l'état DB. Tous les specs passent.

14a. **Given** T6.2 done (8 specs admin + invoices_echeancier migrés), **When** `npm run test:e2e -- frontend/tests/e2e/{accounts,contacts,products,invoices,journal-entries,users,homepage-settings,mode-expert,invoices_echeancier}.spec.ts`, **Then** ces 9 specs passent localement.

14b. **Given** T6.4 done (specs onboarding migrés), **When** `npm run test:e2e -- frontend/tests/e2e/{onboarding,onboarding-path-b}.spec.ts`, **Then** ces 2 specs passent localement.

14c. **Given** T6.1 done (auth migré), **When** `npm run test:e2e -- frontend/tests/e2e/auth.spec.ts`, **Then** la spec passe localement (login + axe).

14d. **Given** T4.5 done (tests d'intégration de l'endpoint), **When** `cargo test -p kesh-api --test test_endpoints_e2e`, **Then** chaque preset (`fresh`, `post-onboarding`, `with-company`, `with-data`) testé via `POST /api/v1/_test/seed` retourne `200 OK` ET la DB contient exactement les rows attendues par les AC #7, #8, #9, #10 respectivement (assertion sur `SELECT COUNT(*)` par table). N9 pass 2 : couverture explicite pour éviter régression silencieuse de l'endpoint.

15. **Given** le job `e2e` de `ci.yml`, **When** inspecté, **Then** la step inline `mysql ... INSERT INTO users (admin, changeme) ...` est SUPPRIMÉE (remplacée par appels HTTP `seedTestState` dans Playwright). `KESH_TEST_MODE: "true"` ajouté à l'env du job.

16. **Given** `docs/testing.md`, **When** ouvert, **Then** il documente : (a) le pattern Rust avec `seed_accounting_company`, (b) le pattern Playwright avec `seedTestState`, (c) les 4 presets et leur usage, (d) la garantie sécurité du gate `KESH_TEST_MODE` (incluant le rejet de `0.0.0.0`), (e) un exemple de bout en bout pour chaque pattern, (f) **les prérequis pour lancer les tests Playwright en local** : backend démarré séparément avec `KESH_TEST_MODE=true` + `KESH_HOST=127.0.0.1` (NEW-H1 pass 4).

17. **Given** Story 6-1 PR #16 rebasée sur main après merge de Story 6-4, **When** la CI tourne, **Then** les 4 jobs passent vert (gate E2E inclus). 6-1 peut alors merger.

## Tasks / Subtasks

### T0 — Décision : crate `kesh-test-fixtures` vs module `kesh-db::test_fixtures` (AC: #1)

- [ ] T0.1 Trancher : module `kesh-db::test_fixtures` (plus simple, déjà dans le crate qui contient `MIGRATOR`) **OU** crate dédié `kesh-test-fixtures` (séparation cleaner, réutilisable hors kesh-db).
- [ ] T0.2 Décision documentée dans Dev Notes et appliquée pour la suite des tâches.

### T1 — Helper Rust `seed_accounting_company` (AC: #1, #2)

- [ ] T1.1 Créer le module/crate selon décision T0.
- [ ] T1.2 Implémenter `pub async fn seed_accounting_company(pool: &MySqlPool) -> Result<SeededCompany, Error>` :
  - Insert company (org_type=Independant, langues FR/FR)
  - Insert **2 users** : `admin/admin123` ET `changeme/changeme` (hash Argon2id réel via `crate::auth::password::hash_password`) — les deux Admin actifs
  - Insert fiscal_year 2020-2030 Open
  - Insert 5 accounts minimum : 1000 Caisse (Asset), 1100 Banque (Asset), 2000 Capital (Liability), 3000 Ventes (Revenue), 4000 Charges (Expense)
  - Insert company_invoice_settings (default_receivable_account_id=compte 1100, default_revenue_account_id=compte 3000, default_sales_journal=`Ventes`)
  - Retourner struct `SeededCompany { company_id, fiscal_year_id, admin_user_id, changeme_user_id, accounts: HashMap<&str, i64>, ... }`
- [ ] T1.3 Tests unitaires du helper (sqlx::test) : appeler le helper et vérifier que toutes les rows existent + FK cohérentes.

### T2 — Refactor `invoice_pdf_e2e.rs` (AC: #3)

- [ ] T2.1 Identifier toutes les utilisations de `seed_validated_invoice_via_sql` / `force_validate_via_sql` dans le fichier.
- [ ] T2.2 Remplacer chacune par : `let seeded = seed_accounting_company(&pool).await?;` puis flow normal (créer facture brouillon, ajouter lignes, appeler `validate_invoice` via la route normale).
- [ ] T2.3 Supprimer les fonctions de bypass SQL devenues mortes.
- [ ] T2.4 Lancer `cargo test -p kesh-api --test invoice_pdf_e2e` → 11/11 passent.

### T3 — Refactor `invoice_echeancier_e2e.rs` (AC: #4)

- [ ] T3.1 Idem T2 pour `create_validated_invoice_via_sql`.
- [ ] T3.2 Lancer `cargo test -p kesh-api --test invoice_echeancier_e2e` → tests passent.

### T4 — Endpoint `/api/v1/_test/seed` + gate `KESH_TEST_MODE` (AC: #6, #7, #8, #9, #10, #11)

- [ ] T4.1 Ajouter dans `crates/kesh-api/src/config.rs` :
  - Champ `pub test_mode: bool` dans `Config`, défaut `false`, parse via `env::var("KESH_TEST_MODE")` avec interprétation `"true"` / `"1"` → `true`, sinon `false`. Toute autre valeur → log warn + défaut `false`.
  - Builder `pub fn with_test_mode(mut self, v: bool) -> Self` (non-breaking pour les callers existants de `from_fields_for_test` / `make_test_config`).
  - Dans le constructeur principal `from_env` ou équivalent, **si `test_mode == true` ET `host` ∉ `{127.0.0.1, ::1, localhost}`** → retourner `Err(ConfigError::TestModeWithPublicBind)` (variant à ajouter à `ConfigError`). Caller (`main.rs`) print le message et exit 1.
- [ ] T4.2 Créer `crates/kesh-api/src/routes/test_endpoints.rs` :
  - **Branche runtime uniquement** (pas de `#[cfg(feature)]`) : la fonction `pub fn router() -> Router<AppState>` retourne le subrouter avec les 2 routes `POST /seed` et `POST /reset`.
  - Dans `crates/kesh-api/src/lib.rs::build_router`, ajouter à la fin :
    ```rust
    if state.config.test_mode {
        router = router.nest("/api/v1/_test", test_endpoints::router());
        tracing::warn!("KESH_TEST_MODE=true — /api/v1/_test/* exposé (DEV/CI ONLY)");
    }
    ```
- [ ] T4.3 **Avant** d'implémenter, valider l'inventaire des tables : `grep -h "CREATE TABLE " crates/kesh-db/migrations/*.sql | awk '{print $3}' | sort -u` doit lister toutes les tables présentes en DB. Comparer avec la liste de l'ordre truncate dans Dev Notes. **Si une table existe en DB mais pas dans la liste truncate** → l'ajouter au bon endroit (FK enfants → parents). Si une table listée n'existe pas → bug (refuser de proceeder, demander clarification).
- [ ] T4.3 Implémenter `POST /api/v1/_test/seed` :
  - Body : `{ "preset": "fresh" | "post-onboarding" | "with-company" | "with-data" }`
  - Pour chaque preset, exécuter une séquence d'opérations (truncate tables + insert via helpers Rust). Cf. AC #7-#10 pour le contenu détaillé de chaque preset.
  - Le preset `fresh` insère **uniquement** un user `changeme/changeme` (pas de company).
  - Les presets `post-onboarding` / `with-company` (alias) insèrent les **2 users** (`admin/admin123` + `changeme/changeme`) + state comptable complet + `onboarding_state.step_completed = 10`.
  - Le preset `with-data` = `with-company` + 1 contact `'CI Contact SA'` + 1 product `'CI Product'`. **Pas de facture pré-seedée** (cf. AC #10 et décision H3 review pass 3 — `invoices_echeancier.spec.ts` crée ses propres factures dynamiquement).
  - Réponse 200 avec liste des rows créées (debug, JSON).
- [ ] T4.4 Implémenter `POST /api/v1/_test/reset` (alias de `seed { preset: "fresh" }`).
- [ ] T4.5 Tests d'intégration : start kesh-api avec `KESH_TEST_MODE=true`, appeler les 4 presets, vérifier état DB.
- [ ] T4.6 Tests d'intégration : start kesh-api avec `KESH_TEST_MODE=false`, appeler `/api/v1/_test/seed` → 404.
- [ ] T4.7 Documentation inline dans `test_endpoints.rs` avec warning **« NEVER expose in production »**.

### T5 — Helper Playwright `seedTestState` (AC: #12)

- [ ] T5.1 Créer `frontend/tests/e2e/helpers/test-state.ts` avec **URL backend absolue** (le helper crée son propre `APIRequestContext` pointant sur le backend `:3000`, pas sur le frontend `:4173` exposé par Playwright `webServer`) :
  ```ts
  import { request as playwrightRequest, type APIRequestContext } from '@playwright/test';

  export type Preset = 'fresh' | 'post-onboarding' | 'with-company' | 'with-data';

  const BACKEND_URL = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000';

  export async function seedTestState(preset: Preset): Promise<void> {
    const ctx: APIRequestContext = await playwrightRequest.newContext({ baseURL: BACKEND_URL });
    try {
      const res = await ctx.post('/api/v1/_test/seed', { data: { preset } });
      if (!res.ok()) {
        throw new Error(
          `seedTestState(${preset}) failed: ${res.status()} ${res.statusText()} — ` +
          `KESH_TEST_MODE may not be enabled on backend ${BACKEND_URL}`
        );
      }
    } finally {
      await ctx.dispose();
    }
  }
  ```
  Le helper **ne dépend pas** de `request` injecté par Playwright (qui pointe sur `:4173`). Ainsi pas besoin de proxy Vite pour `/api/v1/*`.
- [ ] T5.2 ~~Tests unitaires Vitest~~ — supprimé (wrapper d'une ligne, valeur faible vs T4.5/T4.6 qui couvrent l'endpoint en intégration).
- [ ] T5.3 **Vérifier le routing en CI** : ajouter une step de smoke test dans `ci.yml/e2e` qui après le démarrage du backend appelle `curl -sf http://127.0.0.1:3000/api/v1/_test/seed -X POST -d '{"preset":"fresh"}' -H "Content-Type: application/json"` et fail si la réponse n'est pas `200 OK`. Garantit que `KESH_TEST_MODE` est correctement activé et que la route est joignable AVANT que les specs Playwright tournent.

### T6 — Adoption dans les 12 specs Playwright (AC: #13, #14)

- [ ] T6.1 `auth.spec.ts` → `test.beforeAll(async () => { await seedTestState('with-company'); })` (signature sans `request`, helper crée son propre context)
- [ ] T6.2 `accounts.spec.ts`, `contacts.spec.ts`, `products.spec.ts`, `invoices.spec.ts`, `journal-entries.spec.ts`, `users.spec.ts`, `homepage-settings.spec.ts`, `mode-expert.spec.ts` → `beforeAll: seedTestState('with-company')`
- [ ] T6.3 `invoices_echeancier.spec.ts` → `beforeAll: seedTestState('with-data')`
- [ ] T6.4 `onboarding.spec.ts`, `onboarding-path-b.spec.ts` → `beforeAll: seedTestState('fresh')`. **Note** : `onboarding.spec.ts` utilise actuellement `beforeEach` avec login `changeme/changeme` — remplacer ce `beforeEach` par un `beforeAll` qui seed `fresh` (le user `changeme` est inclus dans `fresh` per AC #7).
- [ ] T6.5 **Liste déterministe des specs nécessitant `beforeEach` plutôt que `beforeAll`** :
  - **`onboarding.spec.ts`** → `beforeEach: seedTestState('fresh')` car chaque test progresse `onboarding_state.step_completed` (mutation singleton irréversible dans le run)
  - **`onboarding-path-b.spec.ts`** → idem
  - **Tous les autres specs** → `beforeAll` suffit (ils utilisent des suffixes uniques `Date.now()` pour éviter les collisions de noms et leurs mutations sont scoped à des rows individuelles, pas au singleton)
  - Critère général : si un test mute un singleton (`onboarding_state`) ou supprime une row lue par un autre test du même fichier → `beforeEach`. Sinon → `beforeAll`.
- [ ] T6.6 Lancer `npm run test:e2e` localement → tous les specs passent (cf. AC #14, #14a, #14b, #14c).

### T7 — Mise à jour CI (AC: #15)

- [ ] T7.1 Dans `.github/workflows/ci.yml` job `e2e`, ajouter à l'env du job :
  - `KESH_TEST_MODE: "true"`
  - `KESH_HOST: "127.0.0.1"` (déjà présent — vérifier qu'il l'est bien et qu'il N'EST PAS `0.0.0.0`, sinon le garde-fou T4.1 refusera le démarrage)
- [ ] T7.2 Supprimer la step inline « Seed admin + changeme users (E2E auth) » du job `e2e` (Story 6-1) — devenue redondante avec `seedTestState` qui reset+reseed à chaque `beforeAll`.
- [ ] T7.3 Conserver les steps `cargo sqlx migrate run` (toujours utiles, garantit DB fraîchement migrée avant `seedTestState`).
- [ ] T7.4 Ajouter immédiatement après `Start backend` une **smoke test step** qui valide que l'endpoint test mode est joignable (cf. T5.3). URL via env var pour cohérence avec helper TS (N4 pass 2) :
  ```yaml
  - name: Smoke test /api/v1/_test/seed
    env:
      BACKEND_URL: ${{ env.KESH_BACKEND_URL || 'http://127.0.0.1:3000' }}
    run: |
      curl -sf -X POST "$BACKEND_URL/api/v1/_test/seed" \
        -H "Content-Type: application/json" \
        -d '{"preset":"fresh"}' \
        || (echo "FATAL: $BACKEND_URL/api/v1/_test/seed inaccessible — KESH_TEST_MODE not active?"; exit 1)
  ```
  **Note pass 2 (N7) raffinée pass 3 (H2)** : ce smoke test valide la connectivité endpoint via `curl`. Pour valider en plus que le helper TypeScript `seedTestState` fonctionne (signature, import `playwrightRequest.newContext`), un **`globalSetup` Playwright** (pas une spec) est ajouté dans `frontend/tests/e2e/global-setup.ts` qui appelle `seedTestState('with-company')` et vérifie l'absence d'exception. **Pourquoi pas une spec `_smoke.spec.ts`** : Playwright avec `workers ≥ 2` (cas par défaut sur ubuntu-latest 4 vCPU) parallélise les fichiers. Un spec qui fait `seedTestState('fresh')` tournerait en parallèle d'`accounts.spec.ts.beforeAll` qui veut `with-company` → race condition destructive. Un `globalSetup` tourne **une seule fois avant tous les workers** et garantit la séquentialité. Voir T7.7 pour l'implémentation.
- [ ] T7.5 Validation YAML (`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`).

- [ ] T7.7 **Création `globalSetup` Playwright** (H2 pass 3, message d'erreur amélioré pass 4 NEW-H1) :
  - Créer `frontend/tests/e2e/global-setup.ts` :
    ```ts
    import { seedTestState } from './helpers/test-state';
    async function globalSetup() {
      try { await seedTestState('with-company'); }
      catch (e) {
        console.error(
          '\n❌ FATAL: globalSetup failed.\n' +
          '   Vérifier que :\n' +
          '   1. Le backend kesh-api est démarré (cargo run -p kesh-api dans un terminal séparé)\n' +
          '   2. KESH_TEST_MODE=true est dans l\'env du backend\n' +
          '   3. KESH_HOST=127.0.0.1 (sinon refus démarrage T4.1)\n' +
          '   4. Le backend répond sur ' + (process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000') + '\n',
          e
        );
        throw e;
      }
    }
    export default globalSetup;
    ```
  - Modifier `frontend/playwright.config.ts` : `globalSetup: './tests/e2e/global-setup.ts'`.
  - **Préset `with-company`** (pas `fresh`) car cette state est non-destructive — le `beforeAll` de chaque spec re-seed son state propre par-dessus, sans surprise.
  - **Ne PAS créer** de spec `_smoke.spec.ts` (pas robuste avec workers parallèles, cf. note T7.4 raffinée pass 3).

- [ ] T7.6 **Mise à jour `KESH_HOST` défaut → `127.0.0.1`** (H1 pass 3) :
  - `crates/kesh-api/src/config.rs` : changer `host` défaut de `"0.0.0.0"` à `"127.0.0.1"` dans `from_env` + `from_fields_for_test`.
  - `docker-compose.dev.yml` : `KESH_HOST: "127.0.0.1"` (le port mapping `127.0.0.1:3000:3000` reste cohérent).
  - `.env.example` : `KESH_HOST=127.0.0.1` avec commentaire ajouté : `# Set to 0.0.0.0 ONLY for prod docker-compose where reverse proxy fronts the container`
  - `crates/kesh-api/README.md` : mettre à jour la doc de la variable (cf. M3 pass 3).
  - **Important** : ce changement est **non-breaking pour la CI Story 6-1** (le job `e2e` set explicitement `KESH_HOST: "127.0.0.1"`). Pour la prod, l'opérateur doit explicitement set `KESH_HOST=0.0.0.0` dans son `.env` ou docker-compose.prod.yml (à documenter en T8.3).

### T8 — Documentation `docs/testing.md` (AC: #16)

- [ ] T8.1 Créer `docs/testing.md` avec :
  - Vue d'ensemble : 2 patterns (Rust avec sqlx::test, Playwright avec seedTestState)
  - Section Rust : exemple d'utilisation de `seed_accounting_company`
  - **Section Prérequis Playwright local** (NEW-H1 pass 4) : explicitation que le backend `kesh-api` DOIT être démarré avant `npm run test:e2e` (ex: `cargo run -p kesh-api` dans un autre terminal, avec `KESH_TEST_MODE=true KESH_HOST=127.0.0.1` dans l'env). Documenter le pattern recommandé pour les devs : ajouter une ligne « `npm run test:e2e:full` » au `package.json` qui démarre le backend, attend `/health`, puis lance Playwright (optionnel mais recommandé).
  - Section Playwright : exemple `beforeAll: seedTestState`
  - Section Presets : tableau des 4 presets avec leur contenu (admin/admin123 + changeme/changeme dans tous sauf `fresh` qui contient `changeme` seul)
  - Section Sécurité : explication du gate `KESH_TEST_MODE`, pourquoi 404 si non-set, garantie qu'aucun risque prod, pourquoi `0.0.0.0` est rejeté (cf. H1 + N3)
  - Section Cleanup : convention si test polluant
- [ ] T8.2 Lien vers `docs/testing.md` ajouté dans `docs/ci.md` (section adéquate).
- [ ] T8.3 Mettre à jour `crates/kesh-api/README.md` (M3 pass 3) pour refléter le nouveau défaut `KESH_HOST=127.0.0.1` (au lieu de `0.0.0.0`). Ajouter une note : « Pour bind public en prod (reverse proxy en front), set `KESH_HOST=0.0.0.0` explicitement. »

### T9 — Fermeture KF-001 (AC: #5)

- [ ] T9.1 Dans `docs/known-failures.md`, amender l'entrée KF-001 :
  - Status passe de `closed` à `closed (vérifié post-Story-6.4)`
  - Ajout d'une ligne « Validation : aucun bypass SQL ne subsiste, helper `seed_accounting_company` utilisé »
- [ ] T9.2 Fermer définitivement l'issue GitHub `#7` (ou ajouter un commentaire de validation).

### T10 — Validation end-to-end (AC: #14, #17)

- [ ] T10.1 Push branche `story/6-4-fixtures-e2e-deterministes` → vérifier que les 4 jobs CI passent vert (dont E2E).
- [ ] T10.2 Merge sur main.
- [ ] T10.3 Rebase PR #16 (Story 6-1) sur le nouveau main → CI doit maintenant passer vert.
- [ ] T10.4 Mise à jour `sprint-status.yaml` : `6-4-fixtures-e2e-deterministes: ready-for-dev → done`.

## Dev Notes

### Fichiers à créer

- `crates/kesh-db/src/test_fixtures.rs` (ou crate `kesh-test-fixtures/` selon T0)
- `crates/kesh-api/src/routes/test_endpoints.rs`
- `frontend/tests/e2e/helpers/test-state.ts`
- `docs/testing.md`

### Fichiers à modifier

- `crates/kesh-db/src/lib.rs` — re-export du module test_fixtures
- `crates/kesh-api/src/config.rs` — ajout `test_mode: bool` + builder `with_test_mode()` non-breaking + variant `ConfigError::TestModeWithPublicBind` + check au démarrage
- `crates/kesh-api/src/lib.rs` — enregistrement conditionnel des routes /_test dans `build_router`
- `crates/kesh-api/src/main.rs` — gestion d'erreur `TestModeWithPublicBind` (print message + exit 1)
- `crates/kesh-api/tests/invoice_pdf_e2e.rs` — refactor seed
- `crates/kesh-api/tests/invoice_echeancier_e2e.rs` — refactor seed
- 12 specs Playwright (`frontend/tests/e2e/*.spec.ts`) — ajout `beforeAll: seedTestState` (sauf onboarding qui utilise `beforeEach`)
- `.github/workflows/ci.yml` — ajout `KESH_TEST_MODE: "true"` + retrait seed inline + smoke test endpoint
- `docs/known-failures.md` — amendement KF-001 (statut closed définitif post-helper)
- `docs/ci.md` — lien vers `docs/testing.md` + clarification que le seed inline est retiré

### Risques

- **Endpoint `/api/v1/_test/*` exposé accidentellement en prod** : sévérité CRITIQUE. Mitigation triple : (a) gate env var `KESH_TEST_MODE`, (b) refus 404 si non-set (pas 403 — moins discoverable), (c) test d'intégration explicite (AC #6) qui valide le 404.

- **Reset DB déclenché involontairement en cours de test** : sévérité moyenne. Mitigation : convention `beforeAll` (pas `beforeEach`) sauf si nécessaire ; et en CI, chaque spec a son propre `beforeAll` qui force l'état attendu.

- **Truncate avec FK désactivées peut laisser des orphelins** : si l'ordre de truncate est mauvais, les FK constraints reviennent activées avec des données incohérentes. Mitigation : truncate dans l'ordre inverse des dépendances + tests d'intégration des presets.

  **Ordre de truncate déterministe** (tables enfants → parents, FK désactivées globalement le temps du reset) :
  ```sql
  SET FOREIGN_KEY_CHECKS = 0;
  TRUNCATE TABLE invoice_lines;
  TRUNCATE TABLE journal_entry_lines;
  TRUNCATE TABLE invoices;
  TRUNCATE TABLE invoice_number_sequences;  -- N5 pass 2 : table créée par migration 20260417000001_invoice_validation.sql, présente à 100 %. Si une future PR retire la migration, ce truncate doit échouer fort (pas de fallback silencieux).
  TRUNCATE TABLE journal_entries;
  TRUNCATE TABLE audit_log;
  TRUNCATE TABLE company_invoice_settings;
  TRUNCATE TABLE bank_accounts;
  TRUNCATE TABLE accounts;            -- FK self-ref via parent_id
  TRUNCATE TABLE products;
  TRUNCATE TABLE contacts;
  TRUNCATE TABLE fiscal_years;
  TRUNCATE TABLE refresh_tokens;
  TRUNCATE TABLE onboarding_state;
  TRUNCATE TABLE users;
  TRUNCATE TABLE companies;
  -- NE PAS truncate _sqlx_migrations (sinon next migrate run réapplique tout)
  SET FOREIGN_KEY_CHECKS = 1;
  ```
  Inventaire à valider au début de T4.3 (`SHOW TABLES` runtime + grep des migrations) — si la liste évolue, mettre à jour.

- **Hash Argon2id du seed admin coûte ~50ms par appel `seed`** : pour un endpoint test, négligeable. Si performance pose problème, pré-calculer les hashes au startup et réutiliser.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story-6.4] — AC original (scope Rust uniquement)
- [Source: _bmad-output/implementation-artifacts/6-1-pipeline-ci-github-actions.md] — Story qui a déclenché la création de 6-4
- [Source: PR #16 — feedback_branch_protection_legacy.md] — Découverte de la contradiction d'état Playwright
- [Source: docs/known-failures.md#KF-001] — Bypass SQL à retirer
- [Source: crates/kesh-api/tests/invoice_pdf_e2e.rs:seed_validated_invoice] — Code à refactorer
- [Source: crates/kesh-api/tests/invoice_echeancier_e2e.rs:create_validated_invoice_via_sql] — Code à refactorer
- [Source: crates/kesh-db/migrations/20260409000001_onboarding_state.sql] — Schéma singleton à reset

## Dev Agent Record

### Agent Model Used

_(à remplir lors du dev)_

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Auteur | Modification |
|------|--------|--------------|
| 2026-04-16 | Claude Opus 4.6 (1M context) — pendant cascade Story 6-1 | Création du story file en mode draft. Scope élargi vs `epics.md#Story-6.4` original : ajout du volet Playwright (endpoint `/api/v1/_test/*` + helper `seedTestState`) découvert comme bloquant pour PR #16 Story 6-1. Volet Rust original (helper `seed_accounting_company` + fermeture KF-001) conservé. Story créée pour permettre option F : préparation 6-4 pendant que PR #16 est en pause. À valider via `validate-create-story` (passes adversariales recommandées) avant `dev-story`. |
| 2026-04-16 | Validation pass 1 — Claude Sonnet 4.6 (subagent fenêtre fraîche, orthogonal à l'auteur Opus) | 11 findings appliqués (3 CRITICAL + 4 HIGH + 4 MEDIUM, 3 LOW non appliqués). **CRITICAL** : (F1) routing Playwright→backend explicite via `BACKEND_URL` absolue dans helper + smoke test CI ; (F2) propagation `KESH_TEST_MODE` tranchée pour branche runtime dans `build_router` (refus `#[cfg(feature)]`) ; (F3) preset `with-data` raffiné pour inclure facture validée avec échéancier multi-échéances. **HIGH** : (F4) `post-onboarding`/`with-company` clarifiés comme alias sémantiques d'un seul code path ; (F5) user `changeme` ajouté à tous les presets sauf `fresh` qui le contient seul ; (F6) garde-fou staging — refus démarrage si `test_mode=true` + bind non-loopback (variant `ConfigError::TestModeWithPublicBind`) ; (F7) liste déterministe T6.5 (onboarding* en `beforeEach`, autres en `beforeAll`). **MEDIUM** : (F8) AC #14a/#14b/#14c intermédiaires ajoutés ; (F9) ordre de truncate explicite ajouté à Dev Notes ; (F10) builder `with_test_mode()` non-breaking spécifié pour `from_fields_for_test` ; (F11) `homepage-settings.spec.ts` confirmé `with-company` qui inclut `changeme`. 3 LOW non appliqués (Vitest peu pertinent, kesh-seed cosmétique, Change Log issue GitHub). **Recommandation pass 1** : 2e passe obligatoire avec LLM différent (Haiku ou Opus, pas Sonnet). |
| 2026-04-16 | Validation pass 2 — Claude Haiku 4.5 (subagent fenêtre fraîche, orthogonal Opus+Sonnet) | 9 findings appliqués (3 CRITICAL + 3 HIGH + 3 MEDIUM, 2 LOW non appliqués). **CRITICAL** : (N1) AC #2 amendé avec 2 users + champs `company_invoice_settings` détaillés ; (N2) AC #12 et #13 alignés sur signature réelle T5.1 (helper sans `page`/`request`, autonome via `BACKEND_URL`) ; (N3) `0.0.0.0` explicitement REJETÉ comme alias loopback (sécurité Docker) — CI doit utiliser `KESH_HOST=127.0.0.1`. **HIGH** : (N4) smoke test T7.4 utilise `KESH_BACKEND_URL` env var ; (N5) `invoice_number_sequences` ajouté à l'ordre de truncate (avec note de validation runtime) ; (N6) AC #2 inclut désormais champs `company_invoice_settings` complets. **MEDIUM** : (N7) ajout d'une smoke spec Playwright `_smoke.spec.ts` qui teste le helper TS ; (N8) valeurs preset `with-data` figées en dur (dates 2026-01-15/02-01/03-01/04-01) pour déterminisme ; (N9) AC #14d ajouté pour exiger tests d'intégration explicites de l'endpoint avec assertion par-preset. 2 LOW non appliqués (N10 exemple homepage-settings, N11 commentaire `_sqlx_migrations`). **Recommandation pass 2** : 3e passe obligatoire (LLM = Opus pour orthogonalité avec Sonnet/Haiku) avant `ready-for-dev`. |
| 2026-04-16 | Validation pass 3 — Claude Opus 4.6 (subagent fenêtre fraîche, orthogonal aux passes 1+2 par fenêtre) | 6 findings appliqués (3 HIGH + 3 MEDIUM, 3 LOW non appliqués). **HIGH** : (H1) défaut applicatif `KESH_HOST` changé `0.0.0.0` → `127.0.0.1` (sécurité par défaut, opt-in pour prod) + nouvelle T7.6 pour mettre à jour `docker-compose.dev.yml`, `.env.example`, `crates/kesh-api/README.md` ; (H2) `_smoke.spec.ts` REMPLACÉ par `globalSetup` Playwright (`frontend/tests/e2e/global-setup.ts`) — évite la race condition workers parallèles, T7.7 ajoutée ; (H3) AC #10 raffiné — **PAS de facture pré-seedée** dans preset `with-data` (vérifié dans `invoices_echeancier.spec.ts` : crée ses fixtures dynamiquement via `daysFromToday()`, ignorerait toute facture pré-existante et serait polluée par le badge « En retard »). **MEDIUM** : (M1) note défensive sur `invoice_number_sequences` clarifiée — table existe garantie, échec fort attendu si retirée ; (M2) T4.3 enrichie d'un grep `CREATE TABLE` préalable des migrations pour valider l'inventaire ; (M3) T8.3 ajoutée pour mettre à jour `crates/kesh-api/README.md`. 3 LOW non appliqués (L1 syntaxe GitHub Actions `||`, L2 décision T0 crate vs module, L3 `SELECT COUNT(*)` insuffisant). **Recommandation pass 3** : 4e passe obligatoire (LLM = Sonnet ou Haiku, rotation orthogonale). |
| 2026-04-16 | Validation pass 4 — Claude Sonnet 4.6 (subagent fenêtre fraîche, orthogonal à Opus pass 3) | 2 findings appliqués (1 HIGH + 1 MEDIUM, 3 LOW non appliqués). Trend convergence 14 → 11 → 9 → 2. **HIGH** : (NEW-H1) prérequis « backend démarré avant Playwright » manquant — patché : AC #16 amendé, T8.1 enrichie de la section « Prérequis Playwright local », globalSetup amélioré avec message d'erreur explicite listant les 4 conditions (backend up, KESH_TEST_MODE, KESH_HOST, BACKEND_URL). **MEDIUM** : (NEW-M1) T4.3 contredisait AC #10 post-H3 (mention facture pré-seedée alors qu'elle a été retirée) — patché : T4.3 alignée. 3 LOW non appliqués (NEW-L1 ordre numérotation T7.6/T7.7 cosmétique, NEW-L2 note rassurance `from_fields_for_test`, NEW-L3 test config_from_env_with_database_url à mettre à jour en T7.6). **Recommandation pass 4** : 5e passe obligatoire (LLM = Haiku, rotation Sonnet→Haiku→Opus→Sonnet→**Haiku**). |
| 2026-04-16 | Validation pass 5 — Claude Haiku 4.5 (subagent fenêtre fraîche, orthogonal à Sonnet pass 4) | **ZÉRO finding > LOW** ✅ Trend convergence final : 14 → 11 → 9 → 2 → **0**. Critère d'arrêt CLAUDE.md satisfait. Patches pass 4 (NEW-H1, NEW-M1) vérifiés appliqués sans régression. Cohérence globale ACs ↔ Tasks validée. Signature helper TS correcte. Doc défaut sécurisée (`KESH_HOST=127.0.0.1`). 3 observations LOW non bloquantes (clarté `fresh` vs `post-onboarding`, validité alias `with-company`, AC #14d détail `COUNT(*)`) — toutes acceptables comme polish post-implémentation si jamais. **Status : `draft` → `ready-for-dev`** ✅ Story validée par 5 passes adversariales (auteur Opus + Sonnet + Haiku + Opus + Sonnet + Haiku, rotation orthogonale complète). Prêt pour `bmad-dev-story`. |
