# Story 9.5-1b: Fix E2E infrastructure — KF #54 cascade 401 + KF #57 state/timing

Status: in-progress

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a mainteneur projet Kesh,
I want centraliser un helper d'authentification Bearer pour les appels `APIRequestContext` Playwright dans les helpers de seed E2E (KF #54), puis adresser test-par-test les 13 failures state/timing/redirect résiduelles (KF #57 — fiscal-years, mode-expert, onboarding, onboarding-path-b, homepage-settings, users, journal-entries),
so that la suite E2E redevient verte sur les 9 fichiers `.spec.ts` ciblés, la cascade 401 née de Story 6-5 (storage shift cookie → localStorage) est fermée définitivement, le pattern d'appel API depuis un test Playwright est documenté et réutilisable, et les KFs #54 + #57 peuvent être fermées via `closes #N` au merge.

## Scope

Story d'implémentation E2E **scopée infrastructure tests + helpers** (pas de modification du code applicatif Rust/Svelte hors `frontend/tests/`). Périmètre précis :

- **Helper centralisé** : `frontend/tests/e2e/helpers/test-state.ts` (extension — ajout fonction `authedApiContext(page)` et son utilitaire interne `readAccessTokenFromStorage(page)`).
- **9 fichiers `.spec.ts`** à patcher (refactor des helpers locaux + fix selectors/timing résiduels) :
  - `frontend/tests/e2e/invoices.spec.ts` — helpers `createContactViaApi`, `createProductViaApi` + cascade KF #54.
  - `frontend/tests/e2e/invoices_echeancier.spec.ts` — helper `createContact` + cascade KF #54.
  - `frontend/tests/e2e/journal-entries.spec.ts` — helper `getSeedAccountNumbers` + 12 failures KF #54 + 1 failure KF #57 (`journal-entries.spec.ts:404` tooltips pédagogiques).
  - `frontend/tests/e2e/fiscal-years.spec.ts` — KF #57 (`:44` toBeVisible + `:50` fiscal_year seedé + `:59` création/clôture 30s timeout).
  - `frontend/tests/e2e/mode-expert.spec.ts` — KF #57 (`:26` toggle data-mode 30s timeout + `:41` Ctrl+N redirect).
  - `frontend/tests/e2e/onboarding.spec.ts` — KF #57 (`:57` redirect démo reset + `:77` F5 reprise + `:119/:150` toBeEnabled Invoice Settings).
  - `frontend/tests/e2e/onboarding-path-b.spec.ts` — KF #57 (`:60` flux Path B 30s timeout).
  - `frontend/tests/e2e/homepage-settings.spec.ts` — KF #57 (`:43` 4 sections toBeVisible).
  - `frontend/tests/e2e/users.spec.ts` — KF #57 (`:44` liste users affichée).
- **0 fichier de production** modifié : aucun `.rs`, aucun `.svelte`, aucun `.ts` hors `frontend/tests/`.

**Hors scope 9-5-1b** :

- KF #55 (axe-core a11y 5 pages) — déférée à 9-5-1c.
- KF #91 (DropdownMenu wcag2a 4.1.2 nested-interactive) — déférée à 9-5-1c.
- KF #47 (Story 3-7 AC #22 fallback toast — `test.skip(true)` `fiscal-years.spec.ts:121`) — déférée à 9-5-1d (spec **distinct** du test E2E général que 9-5-1b touche aussi sur `fiscal-years.spec.ts`, voir Coordination ci-dessous).
- KF #50 (AC #29 race REPEATABLE READ déterministe `kf004_no_op_e2e.rs`) — déférée à 9-5-1d.
- Migration storage cookie → IndexedDB ou refactor `api-client.ts` — pas une dette, scope hors v0.1.
- Refactor `page.request.*` global remplacement par un proxy SvelteKit `:4173 → :3000` — option architecture envisagée mais hors scope (le helper local résout le problème sans toucher au build front).
- Réactivation E2E en CI bloquant — décision Epic 10+ après stabilisation v0.1.

## Acceptance Criteria

### Pré-flight environnement et baseline

1. **Given** un workspace Kesh à jour avec `main` `35344c9` + branche `chore/epic-9-5-planning` checkée + commit BMAD upgrade `c6f9444` présent localement (cf. PR #95), **When** la story démarre, **Then** prérequis confirmés : `cargo build --workspace` clean, `cd frontend && npm install && npm run build` clean, MariaDB démarré + migrations appliquées + seed CI inline (cf. T1.4 procédure exacte — bloc SQL `.github/workflows/ci.yml:127-163`), Playwright Chromium installé via `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright install chromium` (memory `reference_playwright_ubuntu26`). Backend kesh-api démarré en mode test : `KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_STATIC_DIR=frontend/build cargo run -p kesh-api &`.

2. **Given** la fenêtre Test Locally First (CLAUDE.md), **When** la baseline E2E pré-fix est capturée, **Then** un fichier `frontend/tests/e2e/baseline-pre-9-5-1b.log` est créé via `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright test invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts --reporter=list 2>&1 | tee tests/e2e/baseline-pre-9-5-1b.log` (depuis `frontend/`). Cette baseline doit reproduire **≥ 6 failures `createContact*` failed: 401** (KF #54 reproductible) **et ≥ 8 failures state/timing/redirect** (KF #57 reproductible). Si l'une de ces conditions n'est PAS satisfaite (e.g. KF déjà résolue par effet de bord depuis le triage 9-5-1a), réviser la story (réduction scope ou fermeture sans patch) avant d'aller plus loin.

### Helper authedApiContext (fondation KF #54)

3. **Given** la nécessité d'attacher le Bearer token aux appels `APIRequestContext` Playwright dans les helpers de seed, **When** la fonction `authedApiContext(page)` est ajoutée à `frontend/tests/e2e/helpers/test-state.ts`, **Then** elle satisfait :
   - **Signature** : `export async function authedApiContext(page: Page): Promise<APIRequestContext>`.
   - **Lecture token** : extraction de `localStorage.getItem('kesh:auth:accessToken')` via `page.evaluate(() => localStorage.getItem(...))` — clés cohérentes avec les constantes existantes lignes 36-38 de `test-state.ts` et `auth.svelte.ts:53-55`.
   - **Garde-fou pré-condition** : si token absent (`null`/`''`/`undefined`), throw avec message explicite `'authedApiContext: no accessToken in localStorage — call login(page) before this helper'` (anti-pattern « silencieux 401 » — l'erreur dit pourquoi).
   - **Construction context** : `playwrightRequest.newContext({ baseURL: resolveBackendUrl(), extraHTTPHeaders: { Authorization: \`Bearer ${token}\` } })` — réutilise `resolveBackendUrl()` existant ligne 47 (DRY).
   - **Pas de dispose interne** : le caller est responsable du `await ctx.dispose()` (cohérent avec le pattern try/finally de `seedTestState` lignes 73-85).

4. **Given** la pré-condition « token présent en localStorage », **When** un test unitaire Vitest est ajouté dans `frontend/tests/e2e/helpers/test-state.test.ts` (créer le fichier si absent), **Then** il couvre :
   - Cas nominal : mock `page.evaluate` retourne `'tok-123'` → `authedApiContext` retourne un context (vérifier via mock `playwrightRequest.newContext` que le `Authorization: Bearer tok-123` est dans `extraHTTPHeaders`).
   - Cas erreur : `page.evaluate` retourne `null` → throw avec message exact `'authedApiContext: no accessToken in localStorage — call login(page) before this helper'`.
   - Cas erreur : `page.evaluate` retourne `''` → throw avec même message (validation `!token` couvre les 3 cas).

   **Note** : tests unitaires Vitest sur helpers d'orchestration Playwright sont permissifs (l'objectif est de fixer le contrat de l'API, pas de tester Playwright lui-même).

### Refactor helpers cascade 401 (KF #54 — 3 fichiers)

5. **Given** `frontend/tests/e2e/invoices.spec.ts`, **When** les **4 helpers locaux** suivants sont refactorés pour utiliser `authedApiContext`, **Then** chacun satisfait les critères ci-dessous :
   - `createContactViaApi` (définition lignes 36-49 — `page.request.post` ligne 37).
   - `createProductViaApi` (définition lignes 51-63 — `page.request.post` ligne 57).
   - `createContactWithAddressViaApi` (définition à la ligne 176 — call sites aux lignes 225, 242, 268 — utilise `page.request.post` ligne 180).
   - `createAndValidateInvoiceViaApi` (définition à la ligne 194 — utilise `page.request.post` aux lignes 199 et 217 ; **non listée dans KF #54** car cascade-hidden derrière `createContactWithAddressViaApi`, mais sera la prochaine source de 401 après fix des 3 autres).

   Critères communs aux 4 helpers :
   - Chaque helper instancie un context via `const ctx = await authedApiContext(page)` puis appelle `ctx.post('/api/v1/...', { data: {...} })` (au lieu de `page.request.post(...)`).
   - `try/finally` enveloppe l'appel pour `await ctx.dispose()` (parallélisme cohérent avec `seedTestState`).
   - Aucun changement de signature externe (l'appelant continue d'invoquer `await createContactViaApi(page, name)` sans modification).

6. **Given** `frontend/tests/e2e/invoices_echeancier.spec.ts`, **When** les **2 helpers locaux** sont refactorés, **Then** :
   - `createContactViaApi` (ligne 41 — `page.request.post` ligne 45).
   - `createAndValidateInvoice` (ligne 58 — `page.request.post` aux lignes 65 et 78).
   - Mêmes critères que AC #5 : utilisation `authedApiContext` + `try/finally` + dispose.

7. **Given** `frontend/tests/e2e/journal-entries.spec.ts`, **When** le helper `getSeedAccountNumbers` (lignes 42-56 environ) est refactoré, **Then** mêmes critères + adaptation à la signature GET (`ctx.get('/api/v1/accounts?includeArchived=false')`).

### Validation cascade 401 résolue

8. **Given** les patches AC #5/6/7 appliqués, **When** `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright test invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts --reporter=list` est exécuté depuis `frontend/`, **Then** :
   - **0 failure** avec un message contenant `401 Unauthorized` OU `createContact*VialApi failed: 401` OU `createProduct.*failed: 401` OU `create invoice failed: 401` OU `expect(received).toBeTruthy()` originant de `helpers/test-state.ts` ligne `authedApiContext`.
   - Le compte de tests passants augmente d'au moins **+18** vs baseline (`tests/e2e/baseline-pre-9-5-1b.log`) — couvre 5 failures `invoices.spec.ts` (lignes 87/112/220/236/259) + 1 failure `invoices_echeancier.spec.ts` (ligne 90) + 12 failures `journal-entries.spec.ts` documentées dans KF #54 = 18 combinés.
   - **Garde anti-faux-positif** : si l'augmentation < 18 mais 0 failure 401 résiduel, vérifier qu'aucun test n'est passé en `test.skip()` masquant le succès. `grep "test.skip" tests/e2e/{invoices,invoices_echeancier,journal-entries}.spec.ts | wc -l` — le compte doit être identique entre baseline et post-fix (sinon documenter pourquoi un test a été skippé).

### Fix résiduel state/timing (KF #57 — 6 fichiers)

9. **Given** la baseline post-AC #8 (après fix KF #54), **When** un re-run E2E sur les 7 fichiers KF #57 est exécuté (`fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts journal-entries.spec.ts`), **Then** une **catégorisation triée par root cause** est documentée dans le Change Log :
   - **Cascade-cleared** : failures KF #57 résolues automatiquement par fix KF #54 (le seedTestState 401 cascadait sur ces tests). Nombre attendu : 0 à 5 (incertain — la baseline 9-5-1a a observé 9 occurrences mais sans isoler les vrais state/timing des cascades).
   - **Seed preset gaps** : failures dues à preset `with-company` ne créant pas une row attendue (e.g. `fiscal-years.spec.ts:50` attend fiscal_year 2020-2030 mais le preset n'en crée pas un). Solution : ajuster preset backend (`crates/kesh-api/src/test_endpoints.rs` ou équivalent — hors scope si > 1 changement de signature, dans ce cas split en sous-story).
   - **Brittle selectors** : `getByText` ambigu ou non-stable. Solution : migrer vers `getByTestId` (ajout `data-testid` côté composant Svelte si nécessaire — autorisé en 9-5-1b si limité à 1-2 composants).
   - **Timing/wait gaps** : `toBeVisible` sans `waitForLoadState('networkidle')` préalable, ou `toHaveURL` sans `waitForURL`. Solution : ajouter wait approprié.
   - **Brittle interactions** : keyboard shortcut (`Ctrl+N` mode-expert:41) ou hover (`journal-entries.spec.ts:404` tooltips) qui ne déclenchent pas sur la page non-stabilisée. Solution : `waitForLoadState` + `page.waitForTimeout(100)` (anti-flake mais limité — éviter > 200ms).

10. **Given** la catégorisation AC #9, **When** chaque failure est patchée individuellement, **Then** chaque patch est commité séparément avec message descriptif `fix(e2e/<spec-name>): <root-cause-short>` (1 commit par root-cause, pas un commit géant — préserve `git log` lisible). Si plusieurs failures partagent la même root-cause exact (e.g. 3 timings dans onboarding.spec.ts), elles peuvent être groupées en un commit unique avec body listant les lignes touchées.

11. **And** pour chaque failure catégorisée « seed preset gaps », si le fix nécessite une modification du backend Rust (`test_endpoints.rs` ou seed handler), **escalader la décision** : soit (a) modifier backend dans 9-5-1b (autorisé si <30 lignes Rust et 0 nouvelle API), soit (b) marquer la failure comme « déférée 9-5-1d ou 9-5-1e » et documenter dans le Change Log. **Critère** : si le changement backend introduit une nouvelle route ou une nouvelle table seed → défère ; si c'est juste ajouter une row dans un preset existant → faire en 9-5-1b.

12. **And** pour chaque failure catégorisée « brittle interactions » qui résiste à `waitForLoadState + waitForTimeout 200ms`, l'inscrire dans une **nouvelle KF GitHub Issues catégorie A** (avec template `known_failure.yml`, label `known-failure` + `technical-debt`) plutôt que de la fixer par un `waitForTimeout` long (>500ms) qui ajoute du flakiness latent. Documenter dans Change Log le numéro de KF créée + scope déféré.

### Validation finale + closure

13. **Given** AC #5 à #12 satisfaits, **When** la commande complète E2E sur les 9 fichiers est exécutée (cf. AC #2 mais après fix), **Then** :
    - **0 failure** restante sur la suite (ou exclusivement des failures hors scope 9-5-1b explicitement référencées — i.e. KF #47 `fiscal-years.spec.ts:121` AC #22 fallback toast qui reste skipped, et tout autre `test.skip` pré-existant non touché).
    - Le compte de pass + skip = 100% du compte total de tests (aucun fail/timeout).

14. **And** un fichier baseline post-fix est créé : `frontend/tests/e2e/baseline-post-9-5-1b.log` via re-run avec la même commande qu'AC #2. Conservé pour traçabilité dans la branche (sera supprimé via `.gitignore` après merge si politique projet — cf. la baseline `baseline-post-7-5.log` actuellement versionnée).

15. **And** les KFs GitHub #54 + #57 sont fermées par 2 commits **dédiés** (1 par KF, pas un commit géant qui ferme les 2) :
    - `fix(e2e): close KF #54 KF-022 cascade 401 helpers via authedApiContext (closes #54)` — body cite les 3 fichiers refactorés + le diff de `test-state.ts`.
    - `fix(e2e): close KF #57 KF-025 state/timing/redirect (closes #57)` — body cite la catégorisation + les commits individuels de fix associés.

### Test Locally First + non-régression

16. **Given** la story 9-5-1b a un impact sur les tests E2E ainsi que sur le helper `test-state.ts` (et **potentiellement** un seed backend Rust si AC #11 escalade vers (a)), **When** un commit est créé, **Then** la règle CLAUDE.md `Test Locally First` s'applique **intégralement** :
    - Backend (Rust) — 4 checks obligatoires : `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (pour modifs backend AC #11(a) uniquement).
    - Frontend (Svelte) — 4 checks obligatoires : `cd frontend && npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`.
    - E2E (Playwright) — l'execution sur les 9 specs EST le travail de la story (AC #13), donc baseline post-fix sert d'évidence Test Locally First.
    - **Exception** : commits intermédiaires sur le helper ou sur 1 seul spec file peuvent sauter la suite complète E2E (lancer uniquement le spec touché). Le check complet est obligatoire **avant** le commit final qui ferme `closes #54` + `closes #57`.

17. **And** aucune régression sur les autres tests E2E hors scope 9-5-1b : `auth.spec.ts contacts.spec.ts homepage.spec.ts products.spec.ts reports.spec.ts` doivent **rester verts** (vs `baseline-post-7-5.log`) — vérification rapide par re-run.

18. **And** aucune régression `cargo test --workspace` : compte de tests passants identique pré et post 9-5-1b (à l'exception des tests Rust éventuellement ajoutés par AC #11(a) si seed preset modifié).

## Tasks / Subtasks

- [x] **T1** Pré-flight environnement (AC: #1)
  - [x] T1.1 Vérifier branche `chore/epic-9-5-planning` checkée + working tree propre vs `chore/bmad-upgrade-6.6.0` (PR #95 en attente merge).
  - [x] T1.2 `cargo build --workspace` propre.
  - [x] T1.3 `cd frontend && npm install` (si modifs `package.json` depuis dernière fois) + `npm run build` propre.
  - [x] T1.4 Démarrer MariaDB + migrations + seed CI inline : (a) `docker compose -f docker-compose.dev.yml up -d db` ; (b) appliquer migrations via `KESH_TEST_MODE=true cargo run -p kesh-api` au démarrage ; (c) appliquer bloc SQL « Seed CI fixtures » de `.github/workflows/ci.yml:127-163` (company + admin + fiscal_year + accounts minimum) si non auto-créé par le seed endpoint.
  - [x] T1.5 Installer Playwright Chromium : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright install chromium` (memory `reference_playwright_ubuntu26`).
  - [x] T1.6 Démarrer backend mode test : `KESH_TEST_MODE=true KESH_HOST=127.0.0.1 KESH_STATIC_DIR=frontend/build cargo run -p kesh-api &` (background) + sanity check `curl -fsS http://127.0.0.1:3000/healthz`.
  - [x] T1.7 Exporter `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` une fois en session pour les T2-T8 (sinon Playwright refuse de démarrer sur Ubuntu 26.04 ≥ 1.49).

- [x] **T2** Baseline pré-fix E2E + confirmation KFs reproductibles (AC: #2)
  - [x] T2.1 Depuis `frontend/`, exécuter `npx playwright test invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts --reporter=list 2>&1 | tee tests/e2e/baseline-pre-9-5-1b.log`.
  - [x] T2.2 Confirmer ≥ 6 failures `createContact*VialApi failed: 401` OU `401 Unauthorized` (KF #54 reproductible) + ≥ 8 failures `Test timeout` / `toBeVisible.*fail` / `toHaveURL.*fail` (KF #57 reproductible). **Résultat empirique** : 27 failed / 22 passed / 7 skipped (3.0min). 18 cascade 401 confirmées (6 invoices/_echeancier + 12 journal-entries) + 7 state/timing/redirect KF #57 (mode-expert ×2 + onboarding-path-b ×2 + onboarding ×2 + journal-entries:404 tooltips) + 2 a11y hors scope (homepage-settings:61, invoices:77 — KF #55 déférées 9-5-1c). KF #54 et #57 reproductibles ✓.
  - [x] T2.3 Si confirmé : `git add tests/e2e/baseline-pre-9-5-1b.log && git commit -m "chore(9-5-1b): baseline pre-fix E2E — KF #54 + #57 reproductibles"`. Sinon : escalader auprès de Guy (KFs déjà résolues → fermer sans patch).

- [x] **T3** Helper `authedApiContext` dans `test-state.ts` (AC: #3, #4)
  - [x] T3.1 Éditer `frontend/tests/e2e/helpers/test-state.ts` — ajouter import `Page` from `@playwright/test` (vérifier non-déjà-importé).
  - [x] T3.2 Ajouter constante `STORAGE_KEY_ACCESS_TOKEN` réutilisée (déjà présente ligne 36 — pas de duplication).
  - [x] T3.3 Ajouter fonction `readAccessTokenFromStorage(page: Page): Promise<string | null>` (utilitaire interne — extrait via `page.evaluate`).
  - [x] T3.4 Ajouter fonction publique `export async function authedApiContext(page: Page): Promise<APIRequestContext>` suivant la spec AC #3 (résolution token, garde-fou null/empty, construction context avec `extraHTTPHeaders: { Authorization: Bearer <token> }`).
  - [x] T3.5 **Avant** de créer le fichier Vitest, mettre à jour `frontend/vite.config.ts` pour inclure les tests sous `tests/` dans le glob Vitest : `include: ['src/**/*.test.ts', 'tests/**/*.test.ts']` (sans cette modification, `npm run test:unit` ignore silencieusement le nouveau test — vérifié par lecture directe de `vite.config.ts:32`).
  - [x] T3.6 Créer `frontend/tests/e2e/helpers/test-state.test.ts` (Vitest) avec 3 cas de test AC #4 (mock `playwrightRequest.newContext` via `vi.mock`, mock `page.evaluate` via stub). **Note dev** : utiliser `vi.hoisted` pour déclarer `newContextMock` au même niveau que `vi.mock` (sinon ReferenceError hoist).
  - [x] T3.7 Lancer (a) `cd frontend && npm run test:unit -- tests/e2e/helpers/test-state.test.ts` : 3/3 pass attendu (chemin explicite, robuste au glob) **et** (b) `cd frontend && npm run test:unit 2>&1 | grep "test-state.test"` : confirmer que le nouveau fichier est bien découvert par le glob mis à jour en T3.5. **Résultat empirique** : 3/3 pass (754ms) ; suite complète 253 tests pass (27 fichiers, +3 nouveaux).
  - [x] T3.8 Commit `feat(e2e): add authedApiContext helper for Bearer-authed API calls (refs #54)` — inclut la modif `vite.config.ts` + helper + tests Vitest.

- [x] **T4** Refactor `invoices.spec.ts` — 4 helpers + 1 call site PDF (AC: #5)
  - [x] T4.1 Identifier helpers locaux à patcher : `grep -n "page.request" frontend/tests/e2e/invoices.spec.ts` (attendu : **6 occurrences** aux lignes 37, 57, 180, 199, 217, 232 — les 4 helpers `createContactViaApi`/`createProductViaApi`/`createContactWithAddressViaApi`/`createAndValidateInvoiceViaApi` à refactorer ; la ligne 232 est une assertion de test PDF directe — laisser en place si le test passe après refactor des 4 helpers, sinon la traiter comme un 5ᵉ call site à wrapper en `authedApiContext`).
  - [x] T4.2 Refactor chaque helper : pattern `const ctx = await authedApiContext(page); try { const res = await ctx.post(...); ...; } finally { await ctx.dispose(); }`. **Extension** : la ligne 232 (`page.request.get('/api/v1/invoices/{id}/pdf')` dans le test PDF golden path) refactorée aussi en proactif — sans Bearer le 401 aurait persisté.
  - [x] T4.3 Vérifier aucun changement de signature externe — les call sites doivent rester inchangés.
  - [x] T4.4 `npx playwright test invoices.spec.ts --reporter=list` : tests qui échouaient en 401 doivent maintenant passer (sauf failures non-401 hors scope KF #54). **Résultat empirique** : 1 passed / 6 failed / 0 skipped (15.4s). **0 occurrences 401** ✓ (Bearer fix actif). Failures résiduelles : 1 a11y `:87` (KF #55 hors scope) + 2 combobox strict-mode `:97/:122` (KF #57 cascade-cleared, T8 catégorisation) + 3 PDF tests `:243/:265/:288` qui passent maintenant à un nouveau stade `create invoice failed: 400` (validation backend — KF #57 cascade-cleared aussi).
  - [x] T4.5 Commit `fix(e2e/invoices): use authedApiContext in 4 helper functions (refs #54)`.

- [ ] **T5** Refactor `invoices_echeancier.spec.ts` — 2 helpers (AC: #6)
  - [ ] T5.1 Identifier helpers via `grep -n "page.request" frontend/tests/e2e/invoices_echeancier.spec.ts` (attendu : **3 occurrences** aux lignes 45, 65, 78 — les helpers `createContactViaApi` (ligne 41, post ligne 45) et `createAndValidateInvoice` (ligne 58, 2 posts lignes 65 + 78) à refactorer).
  - [ ] T5.2 Refactor identique T4.2 — les **2 helpers** doivent être patchés (pas seulement `createContactViaApi`, sinon cascade 401 se déplace sur `createAndValidateInvoice`).
  - [ ] T5.3 `npx playwright test invoices_echeancier.spec.ts --reporter=list` : tests 401 → pass.
  - [ ] T5.4 Commit `fix(e2e/invoices_echeancier): use authedApiContext in 2 helper functions (refs #54)`.

- [ ] **T6** Refactor `journal-entries.spec.ts` (AC: #7)
  - [ ] T6.1 Identifier helper via `grep -n "page.request" frontend/tests/e2e/journal-entries.spec.ts` (attendu : `getSeedAccountNumbers` ligne ~42-56 utilisant GET).
  - [ ] T6.2 Refactor : pattern `const ctx = await authedApiContext(page); try { const res = await ctx.get('/api/v1/accounts?includeArchived=false'); ...; } finally { await ctx.dispose(); }`.
  - [ ] T6.3 `npx playwright test journal-entries.spec.ts --reporter=list` : 12 failures cascade 401 → pass (peut révéler des failures KF #57 résiduelles non-cascade — documenter dans T7).
  - [ ] T6.4 Commit `fix(e2e/journal-entries): use authedApiContext in getSeedAccountNumbers helper (refs #54)`.

- [ ] **T7** Validation cascade 401 résolue (AC: #8)
  - [ ] T7.1 Re-run combiné : `npx playwright test invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts --reporter=list 2>&1 | tee tests/e2e/post-kf54-9-5-1b.log` (depuis `frontend/`).
  - [ ] T7.2 `grep -cE "createContact.*failed: 401|createProduct.*failed: 401|create invoice failed: 401|401 Unauthorized|authedApiContext: no accessToken" tests/e2e/post-kf54-9-5-1b.log` : 0 attendu (pattern élargi pour couvrir les 4 helpers `invoices.spec.ts` + les 2 helpers `invoices_echeancier.spec.ts`).
  - [ ] T7.3 Compter le delta pass count : `grep -c "✓" tests/e2e/post-kf54-9-5-1b.log` vs `grep -c "✓" tests/e2e/baseline-pre-9-5-1b.log` → delta ≥ +18 attendu.
  - [ ] T7.4 Anti-faux-positif (**depuis la racine du repo**, pas depuis `frontend/`) : `grep -c "test.skip" frontend/tests/e2e/invoices.spec.ts frontend/tests/e2e/invoices_echeancier.spec.ts frontend/tests/e2e/journal-entries.spec.ts` pré-fix vs post-fix → identique. (Chemins explicites au lieu d'expansion brace `{...}` pour éviter ambiguïté working-dir.)
  - [ ] T7.5 Si conditions T7.2 + T7.3 + T7.4 OK : `git add tests/e2e/post-kf54-9-5-1b.log && git commit -m "chore(9-5-1b): KF #54 cascade 401 cleared — baseline post-kf54 attached"`.

- [ ] **T8** Catégorisation + fix résiduel KF #57 (AC: #9, #10, #11, #12)
  - [ ] T8.1 Run E2E sur les 7 fichiers KF #57 : `npx playwright test fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts journal-entries.spec.ts --reporter=list 2>&1 | tee tests/e2e/post-kf54-kf57-9-5-1b.log`.
  - [ ] T8.2 Construire le tableau de catégorisation (AC #9) — 5 colonnes : `Test path:line`, `Root cause category`, `Fix approach`, `LoC estimate`, `Decision (fix-here / defer-1d / new-KF)`.
  - [ ] T8.3 Pour chaque entrée « Cascade-cleared » : vérifier que le test passe maintenant (re-run individuel) — pas de patch nécessaire. Documenter dans Change Log.
  - [ ] T8.4 Pour chaque entrée « Seed preset gaps » : si modification backend < 30 LoC Rust et 0 nouvelle API → patch dans 9-5-1b avec commit `fix(seed): add <row> to preset <name> (refs #57)`. Sinon → marquer « déféré 9-5-1d » + documenter scope.
  - [ ] T8.5 Pour chaque entrée « Brittle selectors » : migrer `getByText` → `getByTestId` (ajouter `data-testid` côté composant Svelte si nécessaire — limité à 1-2 composants max). Commit `fix(e2e/<spec>): replace brittle getByText selectors with getByTestId (refs #57)`.
  - [ ] T8.6 Pour chaque entrée « Timing/wait gaps » : ajouter `await page.waitForLoadState('networkidle')` ou `await page.waitForURL(...)` selon contexte. Commit `fix(e2e/<spec>): add waitForLoadState before <assertion> (refs #57)`.
  - [ ] T8.7 Pour chaque entrée « Brittle interactions » résistant à `waitForLoadState + waitForTimeout(200)` : créer KF GitHub via `gh issue create --template known_failure.yml` avec scope précis + lien commit. Documenter dans Change Log « KF #NN créée pour <test> — déférée hors 9-5-1b ».
  - [ ] T8.8 Re-run E2E sur les 7 fichiers après tous les fix résiduels : `npx playwright test fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts journal-entries.spec.ts --reporter=list 2>&1 | tee tests/e2e/post-fix-9-5-1b.log`. Toutes les failures non-déférées doivent passer.

- [ ] **T9** Validation finale 9 fichiers + closure (AC: #13, #14, #15)
  - [ ] T9.1 Run complet 9 fichiers : `npx playwright test invoices.spec.ts invoices_echeancier.spec.ts journal-entries.spec.ts fiscal-years.spec.ts mode-expert.spec.ts onboarding.spec.ts onboarding-path-b.spec.ts homepage-settings.spec.ts users.spec.ts --reporter=list 2>&1 | tee tests/e2e/baseline-post-9-5-1b.log`.
  - [ ] T9.2 Vérifier : 0 failure (hors `test.skip` pré-existants — KF #47 `fiscal-years.spec.ts:121` AC #22 doit rester skipped, c'est attendu, ne pas le débloquer ici).
  - [ ] T9.3 Commit baseline `git add tests/e2e/baseline-post-9-5-1b.log && git commit -m "chore(9-5-1b): baseline post-fix E2E — KF #54 + #57 fermées"`.
  - [ ] T9.4 Commit closure KF #54 dédié (cf. AC #15) : `git commit --allow-empty -m "fix(e2e): close KF #54 KF-022 cascade 401 helpers via authedApiContext\n\n<body avec listing 3 fichiers refactorés + extrait test-state.ts authedApiContext>\n\ncloses #54"`.
  - [ ] T9.5 Commit closure KF #57 dédié : `git commit --allow-empty -m "fix(e2e): close KF #57 KF-025 state/timing/redirect\n\n<body avec catégorisation finale + listing commits T8.4-T8.7>\n\ncloses #57"`.
  - [ ] T9.6 Aucune régression sur les specs hors scope : `npx playwright test auth.spec.ts contacts.spec.ts homepage.spec.ts products.spec.ts reports.spec.ts --reporter=list` → identique au `baseline-post-7-5.log` (AC #17).

- [ ] **T10** Test Locally First — checks CI complets (AC: #16, #18)
  - [ ] T10.1 Backend Rust : `cargo fmt --all -- --check` + `cargo build --workspace --all-targets` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test --workspace` (mode parallèle local OK, le mode CI `-j1 --test-threads=1` n'est requis qu'en cas de touche `kesh-db`).
  - [ ] T10.2 Frontend Svelte : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` (4 checks doivent tous passer).
  - [ ] T10.3 Sanity check : `cargo test --workspace 2>&1 | grep -E "test result|test_count"` — comptes inchangés pré/post 9-5-1b sauf si AC #11(a) backend seed modifié.
  - [ ] T10.4 Si tout OK : push branche `chore/epic-9-5-planning` (déjà en mode commit après les T2-T9).

- [ ] **T11** Documentation finale + sprint-status (AC: #15 + cohérence orchestration)
  - [ ] T11.1 Mise à jour `_bmad-output/implementation-artifacts/sprint-status.yaml` : entrée `9-5-1b-kf-fix-infra: in-progress` (au start T2) puis `in-progress → review` (après T9) puis `review → done` (après code-review converge).
  - [ ] T11.2 Mise à jour `_bmad-output/planning-artifacts/epic-9-5.md` : section « Décision split préventif appliquée 2026-05-18 » — ajouter ligne « 9.5-1b done <date> — KFs #54 + #57 fermées, baseline-post-9-5-1b.log attaché ».
  - [ ] T11.3 Si nouvelle KF GitHub créée en T8.7 : ajouter référence dans le Dev Notes de cette story + mention dans rétrospective Epic 9.5 finale.

## Dev Notes

### Cadrage scope minimal — pattern Story 9-5-1a/2/3

Cette story 9-5-1b suit la même discipline que les sous-stories Epic 9.5 précédentes : **scope minimaliste, file-list explicite, anti-pattern Story 7-1 historique** (4 passes spec validate sur scope > 5 modules). Ici les fichiers touchés sont limités à `frontend/tests/e2e/**` plus une éventuelle modif backend Rust < 30 LoC seulement si AC #11(a) escalade. Le critère projet « 2-3 passes attendues » documenté dans `epic-9-5.md:86` est cohérent avec cette discipline.

### Root cause KF #54 — `page.request.*` n'hérite pas de localStorage

Confirmé par lecture directe de `frontend/src/lib/shared/utils/api-client.ts:104-126` (ligne 122 : `headers['Authorization'] = \`Bearer ${authState.accessToken}\`` n'est injecté QUE pour les fetch via `api-client.ts`) et `frontend/src/lib/app/stores/auth.svelte.ts:53-55` (storage keys `kesh:auth:accessToken`).

`page.request.*` est un client HTTP **raw Playwright** qui partage le cookie store de la page mais **pas** le localStorage / Svelte state. Le Story 6-5 KF-007 closure (commit `765af6d`) a migré le storage cookie → localStorage. Cette migration a coupé l'injection automatique du token pour les call sites `page.request.post()` qui dépendaient implicitement des cookies. Les 18 tests E2E impactés depuis ~2026-04-30 (baseline pre-7-5).

**Pattern correct** post-fix : `const ctx = await authedApiContext(page); await ctx.post(...)` — où `authedApiContext` lit localStorage via `page.evaluate` et configure un `APIRequestContext` avec `extraHTTPHeaders: { Authorization: \`Bearer ${token}\` }`.

### Pourquoi pas de proxy SvelteKit `:4173 → :3000`

Une alternative architecturale serait d'ajouter un proxy SvelteKit `vite.config.ts` ou middleware pour que `:4173/api/v1/*` proxy vers `:3000`, ce qui ferait que `page.request.post('/api/v1/contacts')` hérite naturellement du cookie. **Rejetée** car :

1. Modifie le build front pour un besoin de test uniquement (smell — production build pollué pour les tests).
2. Le helper `authedApiContext` est isolé dans `tests/e2e/helpers/`, scope tests-only — cohérent avec la séparation déjà adoptée par `seedTestState` (qui crée aussi son propre `APIRequestContext` ligne 73).
3. Cookie-based auth est un anti-pattern v0.1 — le projet a explicitement migré vers Bearer token localStorage (Story 6-5 KF-007). Re-introduire le cookie côté tests cassera l'invariant.

### Root cause KF #57 — heterogeneous, à catégoriser empiriquement

Le KF #57 GitHub body identifie 4 causes probables (timeouts, `toBeVisible` fail, `toHaveURL` fail, brittle selectors). Une partie pourrait être **cascade KF #54** (seedTestState pass mais helper créé contact échoue 401 → test suivant voit empty state → `toBeVisible` fail sur contact name). Le T8.1 mesure empiriquement combien de failures KF #57 sont résolues par effet de bord après fix KF #54 — l'ordre des fix (KF #54 d'abord, KF #57 après) est volontaire.

### Risque R1 — seed preset modifications

Si le triage T8 révèle qu'un fix KF #57 nécessite de modifier le preset backend (`with-company` doit créer un fiscal_year par défaut e.g.), la décision AC #11 trace une ligne claire : `< 30 LoC Rust + 0 nouvelle API → fix-here`, sinon défère 9-5-1d.

Le seed handler backend est probablement dans `crates/kesh-api/src/routes/_test.rs` ou `kesh-api/src/test_endpoints.rs` (à confirmer). Si un preset doit accepter un nouveau paramètre query (e.g. `?withFiscalYear=true`), c'est **par construction** une nouvelle API → défère.

### Risque R2 — flakiness `waitForTimeout` (anti-pattern)

L'AC #12 interdit explicitement les `waitForTimeout > 500ms` comme « patch » pour stabiliser un test brittle. Raison : un test qui passe après `waitForTimeout(1000)` ajoute latence + flakiness latente (la cause racine n'est pas adressée, le délai masque). Le bon réflexe est `waitForLoadState`, `waitForURL`, `waitForResponse`, ou — si vraiment l'interaction est non-déterministe — déférer en KF avec scope précis pour investigation future.

### Coordination avec 9-5-1d (KF #47)

`frontend/tests/e2e/fiscal-years.spec.ts:121` contient un `test.skip(true, ...)` dont le message exact est :

```
test.skip(
    true,
    "Skipped — helper testé via TS compile-time + tests backend ; un vrai E2E nécessite un setup form complexe (validate_invoice). Voir Story 5.2 e2e + JournalEntryForm wiring."
);
```

(Le message a évolué — 9-5-1a Dev Notes en référait sous une forme paraphrasée. La sémantique reste : test E2E AC #22 déféré, KF #47 le tracke.) 9-5-1b touche `fiscal-years.spec.ts` aux lignes `:44`, `:50`, `:59` (KF #57 issues), **mais ne touche PAS** le bloc à la ligne `:121` (KF #47 — déférée 9-5-1d). Les éditions de 9-5-1b sur ce fichier doivent préserver le `test.skip` ligne 121 — vérifier explicitement après le dernier commit T8 ou T9 via `grep -nF "test.skip" frontend/tests/e2e/fiscal-years.spec.ts` (flag `-F` fixed-string obligatoire car le message contient des caractères regex `.`/`-`) : doit retourner la ligne 121 toujours présente.

### Memory carries

- **`reference_playwright_ubuntu26`** : sur Ubuntu 26.04 obligatoire `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` pour install + run E2E (limitation upstream Playwright ≤ 1.49). Exporté une fois en session pour toutes les T2-T9.
- **`feedback_haiku_review_diff_combined`** : pour les passes de review code à venir (3 fichiers spec patches + helper test-state.ts), discipline grep ground-truth obligatoire si reviewer Haiku — surfaces touchées multi-commit fréquentes (T4/T5/T6 = 3 commits successifs sur fichiers adjacents).
- **`feedback_avoid_parallel_prs`** : ne pas créer de PR séparée pour 9-5-1b — continuer sur la branche `chore/epic-9-5-planning` (groupera 9-5-3 + 9-5-1a + 9-5-2 + 9-5-1b + 9-5-1c/d + retro Epic 9.5).

### Project Structure Notes

- **Fichiers édités par 9-5-1b** :
  - `frontend/tests/e2e/helpers/test-state.ts` — ajout `authedApiContext` + utilitaire `readAccessTokenFromStorage` (~30 LoC).
  - `frontend/tests/e2e/helpers/test-state.test.ts` — **nouveau fichier** Vitest (3 tests, ~50 LoC).
  - `frontend/tests/e2e/invoices.spec.ts` — refactor 2-3 helpers locaux (LoC nets ≈ 0, mais ~20 lignes touchées).
  - `frontend/tests/e2e/invoices_echeancier.spec.ts` — refactor 1-2 helpers (~10 lignes touchées).
  - `frontend/tests/e2e/journal-entries.spec.ts` — refactor 1 helper + fix KF #57 ligne 404 (~15 lignes touchées).
  - `frontend/tests/e2e/{fiscal-years,mode-expert,onboarding,onboarding-path-b,homepage-settings,users}.spec.ts` — fix KF #57 résiduel (LoC variable selon catégorisation T8, attendu ~5-30 par fichier).
  - `frontend/tests/e2e/baseline-pre-9-5-1b.log` + `baseline-post-9-5-1b.log` — fichiers de baseline traçabilité.
  - `_bmad-output/implementation-artifacts/9-5-1b-kf-fix-infra.md` (cette spec) — Change Log final.
  - `_bmad-output/implementation-artifacts/sprint-status.yaml` — statut entry.
  - `_bmad-output/planning-artifacts/epic-9-5.md` — section split mise à jour avec done date.

- **Fichiers NON touchés** :
  - **Aucun** fichier `.rs` modifié hors scope AC #11(a) (escalade explicite limitée à < 30 LoC).
  - **Aucun** fichier `.svelte` modifié sauf cas AC #8 brittle selectors → ajout `data-testid` (limité 1-2 composants).
  - `frontend/src/**` reste intact (sauf `data-testid` ajouts ciblés).
  - `frontend/tests/e2e/{auth,contacts,homepage,products,reports}.spec.ts` non touchés.
  - `frontend/tests/e2e/fiscal-years.spec.ts:121` (test.skip KF #47) préservé.

- **GitHub Issues** : 2 fermées (`closes #54` + `closes #57`) via commits dédiés T9.4 + T9.5. 0 à 1 nouvelle KF créée si AC #12 déclenchée pour brittle interactions résistantes.

### Testing standards summary

- **Pattern E2E Bearer-auth** : centralisation via `authedApiContext(page)` dans `helpers/test-state.ts`. Tout futur helper de seed E2E qui appelle l'API authentifiée doit utiliser ce pattern (pas `page.request.*` direct).
- **Pattern test unitaire helpers Playwright** : Vitest avec mock `vi.mock('@playwright/test', () => ({...}))` — cf. `test-state.test.ts` créé en T3.5. Convention : tester le contrat (signatures + gardes-fous), pas l'intégration Playwright.
- **Pattern catégorisation failures** : tableau Markdown 5 colonnes (cf. AC #9) — réutilisable pour futures stories de stabilisation E2E.
- **Convention `data-testid`** : si ajout nécessaire (T8.5), suivre le pattern `data-testid="<feature>-<role>"` (e.g. `data-testid="invoice-row"`, `data-testid="contact-name-cell"`). Pas de hash, pas d'index.

### Estimation effort

- **T1 (pré-flight)** : 15-30 min (setup workspace + Playwright install + backend up).
- **T2 (baseline pre-fix)** : 5-10 min (run + commit).
- **T3 (helper + tests unitaires)** : 30-45 min (code + Vitest + run).
- **T4/T5/T6 (refactor 3 spec files)** : 20-30 min chacun = ~75 min.
- **T7 (validation cascade)** : 10 min.
- **T8 (KF #57 catégorisation + fix résiduel)** : **variable selon résultat T8.1** — typiquement 1-3 heures selon nombre de failures hors-cascade. Limite supérieure : si > 8 failures hors-cascade ET résistantes aux patches simples, défère ≥ 4 vers nouvelles KFs (AC #12) plutôt que tirer la story.
- **T9-T11 (validation finale + Test Locally First + doc)** : 30-45 min.
- **Total** : ~3-5 heures de dev pur + 2-3 passes review.

### Memory référencées

- `reference_playwright_ubuntu26`, `feedback_haiku_review_diff_combined`, `feedback_avoid_parallel_prs` (cf. supra).
- `project_session_state_2026_05_18_end` (sera mise à jour post-9-5-1b done).

### References

- [Source: _bmad-output/planning-artifacts/epic-9-5.md#Story-9.5-1] — spec parent epic, ACs hérités §"Critères d'acceptation" + scope finalisé split.
- [Source: _bmad-output/implementation-artifacts/9-5-1a-kf-triage.md] — triage statique amont (verdict 6/6 KFs actives, scope sub-stories).
- [Source: frontend/tests/e2e/helpers/test-state.ts:71-86] — pattern `APIRequestContext` existant (seedTestState).
- [Source: frontend/tests/e2e/helpers/test-state.ts:36-38] — constantes `STORAGE_KEY_ACCESS_TOKEN` cohérentes avec auth.svelte.ts.
- [Source: frontend/src/lib/app/stores/auth.svelte.ts:53-55] — storage keys canoniques `kesh:auth:accessToken/refreshToken/expiresIn`.
- [Source: frontend/src/lib/shared/utils/api-client.ts:104-126] — pattern injection `Authorization: Bearer` côté app SvelteKit.
- [Source: frontend/tests/e2e/invoices.spec.ts:36-49] — anti-pattern actuel `page.request.post` sans Bearer.
- [Source: frontend/tests/e2e/baseline-post-7-5.log] — baseline empirique (32 failed, 52 passed, 9 skipped post-Story 7-5).
- [GitHub Issue #54 KF-022] — cascade 401 helpers (~18 tests).
- [GitHub Issue #57 KF-025] — state/timing/redirect dispersés (~13 tests).
- [Commit 765af6d] — Story 6-5 KF-007 closure (storage shift cookie → localStorage, origine KF #54).
- [Source: CLAUDE.md§Test Locally First] — checks CI obligatoires backend + frontend.
- [Source: CLAUDE.md§Review Iteration Rule] — cycle review 2-3 passes attendues, LLM différent par passe.
- [Source: CLAUDE.md§Règle de splitting préventif] — discipline file-list explicite.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Pass 1 spec validate — 2026-05-19, Sonnet 4.6 (subagent contexte frais)

**Verdict trend** : 0 CRITICAL + 2 HIGH + 2 MEDIUM + 3 LOW = 7 findings (Convergence : NON).

**Discipline grep ground-truth Sonnet** appliquée — 16/16 ground-truth verifications positives (every CRITICAL/HIGH claim verified by `Read` direct sur fichier source ou `grep -nF` avant d'être levée). Aucun faux-positif détecté.

**Patches appliqués (7/7 — tous patchables sans defer)** :

1. **HIGH-01 — `vite.config.ts:32` include pattern** : le test Vitest proposé `frontend/tests/e2e/helpers/test-state.test.ts` tombe hors du glob `include: ['src/**/*.test.ts']` → `npm run test:unit` l'ignorerait silencieusement. **Patch** : T3.5 ajouté pour étendre le glob à `['src/**/*.test.ts', 'tests/**/*.test.ts']` *avant* de créer le fichier de test ; T3.7 dédoublé pour vérifier (a) exécution explicite + (b) découverte par le glob mis à jour.

2. **HIGH-02 — `createAndValidateInvoiceViaApi` manquant du refactor** : ligne 194 de `invoices.spec.ts` utilise `page.request.post` (lignes 199 + 217) et est appelée immédiatement après `createContactWithAddressViaApi` aux lignes 225/242/268. **Cascade-hidden** dans KF #54 mais réémerge après fix des autres helpers. **Patch** : AC #5 réécrit pour lister explicitement les **4 helpers** (createContactViaApi/createProductViaApi/createContactWithAddressViaApi/createAndValidateInvoiceViaApi avec lignes exactes) ; T4.1 mis à jour avec 6 occurrences attendues lignes 37/57/180/199/217/232 ; T7.2 grep élargi à `create invoice failed: 401|createProduct.*failed: 401` ; AC #8 message d'erreur élargi.

3. **MEDIUM-01 — Ambiguïté « ou » AC #6** : `invoices_echeancier.spec.ts` a **2 helpers** (createContactViaApi ligne 41 + createAndValidateInvoice ligne 58) qui utilisent tous deux `page.request.post`. **Patch** : AC #6 réécrit pour énumérer les 2 helpers ; T5.1 mis à jour avec 3 occurrences attendues lignes 45/65/78 ; T5.2 clarifié sur la nécessité de patcher les 2 (sinon cascade 401 se déplace sur le second).

4. **MEDIUM-02 — Message `test.skip` faussé dans Dev Notes** : le message cité (« AC #22 fallback toast - deferred to KF #47 ») est paraphrasé — le message réel mentionne « helper testé via TS compile-time + tests backend ; un vrai E2E nécessite un setup form complexe (validate_invoice). Voir Story 5.2 e2e + JournalEntryForm wiring. ». **Patch** : Dev Notes §Coordination avec 9-5-1d cite maintenant le message textuel exact + précise l'usage de `grep -nF` (fixed-string) pour vérification ground-truth future.

5. **LOW-01 — Compte ambigu « 6 failures invoices »** : ambigu entre `invoices.spec.ts` seul vs invoices + invoices_echeancier combinés. **Patch** : AC #8 explicite « 5 failures `invoices.spec.ts` (lignes 87/112/220/236/259) + 1 failure `invoices_echeancier.spec.ts` (ligne 90) + 12 failures `journal-entries.spec.ts` = 18 combinés ».

6. **LOW-02 — « ligne ~220 » pointe call site vs définition** : confusion possible entre call site (ligne 225) et définition de fonction (ligne 176). **Patch** : AC #5 précise « définition à la ligne 176 — call sites aux lignes 225, 242, 268 ».

7. **LOW-03 — Expansion brace `{...}` shell ambiguë** : `grep ... frontend/tests/e2e/{invoices,invoices_echeancier,journal-entries}.spec.ts` peut s'exécuter depuis `frontend/` ou racine selon contexte, expansion brace pas garantie portable. **Patch** : T7.4 reformulé avec working dir explicite (« depuis la racine du repo ») + chemins complets sans expansion brace.

**Recommandation Sonnet** : Pass 2 Haiku 4.5 avec discipline grep ground-truth obligatoire (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`).

**Modèle Pass 1** : Sonnet 4.6 (subagent isolé, contexte frais — spec créée par Opus 4.7, règle CLAUDE.md `LLM différent passe précédente` respectée).

### Pass 2 spec validate — 2026-05-19, Haiku 4.5 (subagent contexte frais)

**Verdict trend brut** : 1 CRITICAL + 0 HIGH + 0 MEDIUM + 1 LOW = 2 findings (Convergence brute : NON).
**Verdict effectif après dismiss faux-positif** : 0 CRITICAL + 0 HIGH + 0 MEDIUM + 1 LOW (accepté tel quel) = **CONVERGENCE PASS 2** (critère CLAUDE.md `Uniquement findings LOW` atteint).

**Discipline grep ground-truth Haiku** : 9/9 vérifications positives — pré-conditions du patch validées (commit `35344c9` HEAD main + `c6f9444` sur `chore/bmad-upgrade-6.6.0`), AC numbering 1-18 sans gap, line numbers `invoices.spec.ts` 37/57/180/199/217/232 + `invoices_echeancier.spec.ts` 45/65/78 confirmés, message `test.skip` `fiscal-years.spec.ts:121` correct, 3 memory files cités présents.

**Finding C1 dismissed comme faux-positif Haiku** (per CLAUDE.md §"Haiku-specific guardrails") :

- **C1 (CRITICAL) Haiku allégué** : « HIGH-01 patch NOT applied — `vite.config.ts:32` still missing `'tests/**/*.test.ts'` ». Ground-truth Haiku : `sed -n '32p' frontend/vite.config.ts` → `include: ['src/**/*.test.ts'],` (seul `src` pattern). **Conclusion Haiku** : régression Pass 1.
- **Réfutation ground-truth orchestrateur** :
  - `git show 9c2cef6 --stat` → 1 fichier modifié uniquement (`9-5-1b-kf-fix-infra.md`, +64/-20). **Le commit Pass 1 ne touche PAS `frontend/vite.config.ts`**.
  - `grep -nF "vite.config.ts" 9-5-1b-kf-fix-infra.md` → ligne 151 contient « T3.5 **Avant** de créer le fichier Vitest, mettre à jour `frontend/vite.config.ts` ... » — c'est une **consigne de tâche pour le futur dev**, pas un patch appliqué.
  - Status story = `ready-for-dev`. **Aucun fichier source modifié à ce stade par construction** (le code patches arrivent en dev-story, pas en spec validate).
- **Cause Haiku** : confusion classique « patch dans la spec qui ajoute une tâche » vs « patch dans le code source ». Variante du symptôme documenté CLAUDE.md §"Haiku-specific guardrails" (Haiku confond la nature du patch — méta-spec vs code).
- **Action** : dismiss C1. Aucun patch nécessaire. `vite.config.ts` sera modifié par le dev en T3.5 lors de `bmad-dev-story 9-5-1b`.

**Finding L1 accepté tel quel** (LOW polish) :

- **L1 (LOW)** : AC #10 commit grouping ambiguity sur « la même root-cause exact ». Haiku conclut elle-même « Not a blocker. No action required in spec » — la flexibilité est intentionnelle (T8 est empirique). Pas de patch.

**Trend cumulé spec validate** : Pass 1 Sonnet 0C+2H+2M+3L → 7 patches → Pass 2 Haiku 1C(dismissed)+0H+0M+1L(accepté) → **Convergence**. Cycle short (2 passes) — cohérent avec discipline « scope minimaliste » Epic 9.5.

**Modèle Pass 2** : Haiku 4.5 (subagent isolé, contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée Sonnet → Haiku).

**Statut final spec** : `ready-for-dev` confirmé. Prête pour `bmad-dev-story 9-5-1b`.
