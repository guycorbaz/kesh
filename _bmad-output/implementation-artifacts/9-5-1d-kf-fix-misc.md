# Story 9.5-1d: Fix specific KFs — KF #47 fallback toast E2E + KF #50 race REPEATABLE READ test concurrent

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a mainteneur projet Kesh,
I want fermer les 2 KFs spécifiques résiduelles d'Epic 9.5 — **KF #47 (KF-019)** en remplaçant le `test.skip(true, ...)` placeholder `frontend/tests/e2e/fiscal-years.spec.ts:121` par 3 vrais tests Playwright qui exercent le helper `notifyMissingFiscalYearOrFallback` via les call sites instrumentés (validate_invoice pour `FISCAL_YEAR_INVALID` + `FISCAL_YEAR_CLOSED`, JournalEntryForm pour `NO_FISCAL_YEAR`) et **KF #50 (KF-021)** en remplaçant le smoke-test séquentiel `no_op_with_parallel_mutation_returns_409_when_sequential` (`crates/kesh-api/tests/kf004_no_op_e2e.rs:707`) par un test concurrent réel `tokio::join!` sur 2 `MySqlPool` distincts qui exercise la fenêtre de race et asserte **`409 OPTIMISTIC_LOCK_CONFLICT`** (comportement post-#49 closed par commit `ebdea4b`, NON plus `200 stale` qui était le comportement v0.1 obsolète),
so that les 2 dernières KFs Epic 9.5 sub-story 1 sont fermées (`closes #47` + `closes #50`), la dette de couverture Playwright AC #22 (toast actionnable fallback fiscal_year) est éliminée, le pattern de test concurrent multi-pool sert de régression detector pour la migration `SELECT FOR UPDATE` (#49 closed), et Epic 9.5 sub-story 1 est complète (4/4 sub-stories done : 9-5-1a triage + 9-5-1b infra + 9-5-1c a11y + 9-5-1d misc).

## Scope

Story d'implémentation **dual-backend-frontend scopée tests E2E + tests intégration Rust**. Périmètre précis :

### KF #47 (KF-019) — Playwright AC #22 fallback toast

- **Fichier modifié** : `frontend/tests/e2e/fiscal-years.spec.ts` (1 fichier — bloc `test.describe('AC #22 — fallback toast actionnable')` lignes 93-126).
- **Tests à implémenter** : 3 vrais tests Playwright remplaçant le `test.skip(true, ...)` ligne 121 :
  1. **`FISCAL_YEAR_INVALID`** via flow `validate_invoice` avec date facture **hors** plage `with-company` fiscal_year seedé (2020-2030 → utiliser `1900-01-01`). Helper instrumenté `frontend/src/routes/(app)/invoices/[id]/+page.svelte:94-96` via `notifyMissingFiscalYearOrFallback`. Vérifier toast rendu + clic « Ouvrir Paramètres » → `/settings/fiscal-years`.
  2. **`NO_FISCAL_YEAR`** via flow `JournalEntryForm` avec preset `with-company-no-fy` (preset existant `crates/kesh-api/src/routes/test_endpoints.rs:20`). Helper instrumenté `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:140-141`. Vérifier toast « Créez d'abord un exercice » + clic → `/settings/fiscal-years`.
  3. **`FISCAL_YEAR_CLOSED`** via flow `JournalEntryForm` (PAS `validate_invoice` — vérifié ground-truth : `validate_invoice` utilise `find_open_covering_date` qui produit `FiscalYearInvalid` pour un FY clos, le code `FiscalYearClosed` n'est levé que par `journal_entries::create/update` lignes 109/598/836). Setup : clôturer le fiscal_year `with-company` seedé (2020-2030) via API, puis soumettre une écriture via `JournalEntryForm` avec date in-range (e.g. `2025-06-15`). Vérifier toast distinct « L'exercice qui couvre cette date est clôturé… » (différent du message NO_FISCAL_YEAR).

- **0 modification source applicative** (helper `notifyMissingFiscalYearOrFallback` `frontend/src/lib/shared/utils/notify.ts:98-123` déjà implémenté + 2 call sites instrumentés depuis Story 3-7).

### KF #50 (KF-021) — Test concurrent race REPEATABLE READ

- **Fichier modifié** : `crates/kesh-api/tests/kf004_no_op_e2e.rs` (1 fichier — fonction `no_op_with_parallel_mutation_returns_409_when_sequential` ligne 707-792).
- **Refactor approche** :
  - **Renommer** `no_op_with_parallel_mutation_returns_409_when_sequential` → `no_op_with_parallel_mutation_returns_409_under_concurrency` (le smoke séquentiel devient un vrai test concurrent).
  - **Ajouter** un test concurrent réel utilisant `tokio::join!` sur 2 `MySqlPool` distincts (via `pool.clone()` ne suffit PAS — chaque tâche doit avoir SA connection, idéalement via 2 pools séparés ou 2 `pool.acquire()` parallèles) qui exercise la fenêtre T0-T6 documentée issue #49 KF-020.
  - **Assertion mise à jour** : asserter **`409 OPTIMISTIC_LOCK_CONFLICT`** (comportement post-#49 fix `ebdea4b`). NON plus `200 + stale snapshot` qui était le comportement v0.1 obsolète documenté dans la spec originale KF #50.
  - **Documenter** dans le code (commentaire inline ou doc rust ///) que le test sert de **régression detector** pour la migration `SELECT FOR UPDATE` (#49 closed) — si le `FOR UPDATE` était accidentellement retiré, le test reviendrait à `200 stale` (failed assert 409) → red signal.

- **Changement d'entité critique** : le test courant `no_op_with_parallel_mutation_returns_409_when_sequential` (`kf004_no_op_e2e.rs:707-792`) opère sur `/api/v1/contacts` (lignes 720, 743, 769). Le refactor **doit changer l'entité de `contacts` à `invoices`** pour cibler le `SELECT FOR UPDATE` de la migration #49 (l'entité `contacts` ne dispose PAS de `FOR UPDATE`, c'est spécifique à `invoices::update` `kesh-db/src/repositories/invoices.rs:674`). Pattern de setup invoice à réutiliser : voir `put_invoice_no_op_returns_200_unchanged_version` (`kf004_no_op_e2e.rs:345`) qui montre le setup complet (`create_seeded_company` fournit déjà un fiscal_year + 2 users `alice`/`bob` ; il reste à POST `/api/v1/contacts` puis POST `/api/v1/invoices` avec lignes).

- **Pattern référence `tokio::join!`** — **vérifié ground-truth Pass 1 : ZÉRO invocation `tokio::join!` dans `kf004_no_op_e2e.rs`**. Les 2 mentions du fichier (lignes 689 + 789) sont en **commentaires** uniquement, pas en code exécutable. Conclusion : le pattern concurrent doit être **créé depuis zéro** dans cette story, pas adapté d'un existant. La fonction `concurrent_no_op_returns_200_200_not_200_409` (ligne 488) — malgré son nom — est **séquentielle** (2 PUT contacts consécutifs sans `tokio::join!`). Source de référence pour le setup invoice : `put_invoice_no_op_returns_200_unchanged_version` (ligne 345).

- **Approche concurrence à privilégier** (à arbitrer T5) :
  - **Approche 1 (recommandée)** : `tokio::join!` sur 2 closures async, chacune utilisant le même `pool` partagé (les connections sont acquises automatiquement par les helpers `app.client.put(...)` via `reqwest::Client` qui partage le pool HTTP — la concurrence DB se produit côté `kesh-db/repositories/invoices.rs` via `pool.acquire()` interne à `update()`). 1 pool partagé suffit ; les 2 tx parallèles se sérialisent au niveau du `SELECT FOR UPDATE` X-lock.
  - **Approche 2 (fallback)** : 2 `MySqlPool::connect` distincts pointant sur la même DB. Plus lourd setup mais isolation complète.
  - **Approche 3 (dégradé)** : stress test en boucle N=100 itérations qui valide *présence* d'au moins 1 cas `409 OPTIMISTIC_LOCK_CONFLICT` (cf. issue #50 description). Last-resort si Approches 1+2 sont non-déterministes en CI.

- **0 modification kesh-db** (`SELECT FOR UPDATE` déjà appliqué `crates/kesh-db/src/repositories/invoices.rs:674` par commit `ebdea4b` `fix(db): KF-020 SELECT FOR UPDATE in invoices::update (closes #49)`).

### Hors scope 9-5-1d (par construction post-9-5-1a triage)

- KF #54 (cascade 401) — ✅ fermée 9-5-1b commit `93c36e1`.
- KF #55 (axe-core 6 pages) — ✅ fermée 9-5-1c commit `2babd2f` (4 résiduels login known v0.1).
- KF #57 (state/timing/redirect) — ✅ fermée 9-5-1b commit `c30a344`, split en KF-028 #96 + KF-029 #97 nouvelles.
- KF #91 (DropdownMenu nested-interactive) — ✅ fermée 9-5-1c commit `0e84fa2`.
- Audit autres tests E2E hors AC #22 + race no-op — hors scope.
- Migration `SELECT FOR UPDATE` autres entités (`contacts`, `products`, etc.) — hors scope. Si une issue prod émerge, ouvrir issue dédiée par entité (cf. issue #49 §Remediation story).

## Acceptance Criteria

### Pré-flight environnement et baseline

1. **Given** un workspace Kesh à jour avec `main` `35344c9` + branche `chore/epic-9-5-planning` checkée (HEAD `3bb4486` post-9-5-1c done), **When** la story démarre, **Then** prérequis confirmés : `cargo build --workspace` clean, `cd frontend && npm install && npm run build` clean, MariaDB démarré + migrations appliquées, Playwright Chromium installé via `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npx playwright install chromium` (memory `reference_playwright_ubuntu26`), backend kesh-api démarré KESH_TEST_MODE=true + KESH_HOST=127.0.0.1 (override `.env` qui a KESH_HOST=0.0.0.0). Si backend déjà running depuis 9-5-1c → réutiliser (PID 3081990 si session continue, sinon redémarrer).

2. **Given** la baseline pré-fix, **When** les 2 commandes baseline sont exécutées :
   - `cd frontend && PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test fiscal-years.spec.ts --reporter=list 2>&1 | tee tests/e2e/baseline-pre-9-5-1d-fiscal-years.log`
   - `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1 2>&1 | tee crates/kesh-api/tests/baseline-pre-9-5-1d-kf004.log`
   
   **Then** confirmer (a) le test `FISCAL_YEAR_INVALID déclenche le toast actionnable` est `skipped` (`test.skip(true, ...)` ligne 121) — vérifié dans le rapport Playwright `1 skipped` sur le bloc AC #22 ; (b) le test `no_op_with_parallel_mutation_returns_409_when_sequential` passe en mode séquentiel (smoke). **Hint diagnostic** : `grep -c "test.skip" frontend/tests/e2e/fiscal-years.spec.ts` doit retourner ≥ 1 (la ligne 121). `grep -nE "fn no_op_with_parallel_mutation_returns_409_when_sequential" crates/kesh-api/tests/kf004_no_op_e2e.rs` doit retourner ligne 707.

### Phase A — KF #47 implémentation 3 tests Playwright AC #22

3. **Given** le test 1 `FISCAL_YEAR_INVALID` implémenté, **When** il est exécuté, **Then** il satisfait :
   - **Setup** : `seedTestState('with-company')` (fiscal_year 2020-2030 ouvert).
   - **Action 1** : créer une facture (POST `/api/v1/invoices`) avec champs minimaux (contact, items) — peu importe les détails business sauf que la date d'émission est **hors plage** `2020-2030`. Suggestion : `issueDate: '1900-01-01'`. Stocker l'`id` de la facture créée.
   - **Action 2** : naviguer vers `/invoices/<id>` via `page.goto(...)`.
   - **Action 3** : cliquer le bouton « Valider la facture » (sélecteur à adapter selon le composant — probablement `data-testid="invoice-validate-button"` ou `getByRole('button', { name: /valider/i })`).
   - **Assertion 1** : `await expect(page.getByRole('alert').filter({ hasText: /exercice comptable/ })).toBeVisible()` — le toast `svelte-sonner` est rendu avec `role="alert"`.
   - **Assertion 2** : cliquer le bouton action du toast (label « Ouvrir Paramètres » i18n `go-to-settings`). Selector possible : `await page.getByRole('button', { name: /ouvrir paramètres/i }).click()`.
   - **Assertion 3** : `await expect(page).toHaveURL(/\/settings\/fiscal-years/)` — la navigation a eu lieu.
   - **Test isolation** : le bloc `afterEach` existant (`clearAuthStorage(page)` ligne 25) + `beforeEach` `seedTestState('with-company')` ligne 22 garantissent l'isolation. Pas besoin de cleanup spécifique.

4. **Given** le test 2 `NO_FISCAL_YEAR` implémenté, **When** il est exécuté, **Then** il satisfait :
   - **Setup** : `seedTestState('with-company-no-fy')` (preset existant `test_endpoints.rs:20` — company sans fiscal_year). **Override beforeEach** pour ce test spécifique (le default `with-company` ne convient pas — le test a son propre `seedTestState` au début).
   - **Re-login obligatoire** : `seedTestState` truncate + re-seed la DB ⇒ invalide les sessions JWT/localStorage existantes du `beforeEach`. Après l'override seed, appeler `login(page)` (ou `goToFiscalYears(page)` qui inclut le login) pour ré-authentifier avec les credentials du nouveau seed `admin/admin` avant toute navigation. Sans ce re-login, les requêtes `/api/v1/journal-entries` redirigeraient vers `/login` (cascade 401) et le toast ne serait jamais déclenché.
   - **Action 1** : naviguer vers `/journal-entries` via `page.goto(...)`.
   - **Action 2** : cliquer le bouton « Nouvelle écriture » ou équivalent ouvrant `JournalEntryForm` (sélecteur à confirmer ground-truth `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`).
   - **Action 3** : remplir le minimum requis pour soumission (date, montant, comptes), puis cliquer « Enregistrer ».
   - **Assertion 1** : toast rendu avec message i18n `error-fiscal-year-missing` (« Créez d'abord un exercice comptable dans Paramètres → Exercices »). Pattern selector : `await expect(page.getByRole('alert').filter({ hasText: /Créez d'abord un exercice/ })).toBeVisible()`.
   - **Assertion 2** : clic action button → navigation `/settings/fiscal-years`.

5. **Given** le test 3 `FISCAL_YEAR_CLOSED` implémenté, **When** il est exécuté, **Then** il satisfait :
   - **Routage critique** — le code racine `FISCAL_YEAR_CLOSED` est levé **uniquement** par `journal_entries::create` (ligne 109 `journal_entries.rs`) et `journal_entries::update` (lignes 598 + 836). Le flow `validate_invoice` utilise `find_open_covering_date` (`invoices.rs:970`) qui ne retourne que les FY ouverts → un FY clos donne `FiscalYearInvalid` (PAS `FiscalYearClosed`). **Test 3 doit donc passer par `JournalEntryForm`**, pas `validate_invoice`.
   - **Setup spécifique** : `seedTestState('with-company')` puis appel API direct pour clôturer le fiscal_year `2020-2030` seedé. Approche A : `await page.request.post('/api/v1/fiscal-years/{id}/close', { headers: { Authorization: 'Bearer ...' } })` (à vérifier ground-truth la signature exacte — T4.4 inclut step lecture `crates/kesh-api/src/routes/fiscal_years.rs`). Approche B : naviguer vers `/settings/fiscal-years`, cliquer bouton « Clôturer » sur le fiscal_year affiché. Privilégier Approche A (plus rapide + déterministe — pas de scrape UI).
   - **Action** : naviguer `/journal-entries`, ouvrir `JournalEntryForm`, remplir minimum + date `2025-06-15` (in-range FY 2020-2030 mais l'exercice est clos), soumettre. Le backend `journal_entries::create` ligne 109 retourne `DbError::FiscalYearClosed` → HTTP error code `FISCAL_YEAR_CLOSED` → helper `notifyMissingFiscalYearOrFallback` détecte `FY_CLOSED_CODE` → toast distinct.
   - **Assertion 1** : toast distinct avec message i18n `error-fiscal-year-closed-for-date` (« L'exercice qui couvre cette date est clôturé. Vérifiez la date saisie ou consultez vos exercices. »). Pattern selector : `await expect(page.getByRole('alert').filter({ hasText: /clôturé/ })).toBeVisible()`.
   - **Assertion 2** : message **différent** du test 2 (NO_FISCAL_YEAR) — vérifier que le toast ne contient PAS « Créez d'abord » : `await expect(page.getByRole('alert').filter({ hasText: /Créez d'abord/ })).toHaveCount(0)`.
   - **Assertion 3** : clic action button → navigation `/settings/fiscal-years` (même behavior que tests 1 et 2 — le helper unifie l'action).

6. **And** les 3 tests passent en isolation ET groupés (run `npx playwright test fiscal-years.spec.ts:93 fiscal-years.spec.ts:NNN fiscal-years.spec.ts:MMM` puis run complet `npx playwright test fiscal-years.spec.ts` — résultats identiques).

7. **And** **aucun** `test.skip(true, ...)` résiduel dans `fiscal-years.spec.ts`. Vérification : `grep -c "test.skip(true" frontend/tests/e2e/fiscal-years.spec.ts` = **0**.

### Phase B — KF #50 implémentation test concurrent race REPEATABLE READ

8. **Given** le test existant `no_op_with_parallel_mutation_returns_409_when_sequential` (`crates/kesh-api/tests/kf004_no_op_e2e.rs:707`), **When** le refactor est appliqué, **Then** :
   - **Renommage** : `no_op_with_parallel_mutation_returns_409_when_sequential` → `no_op_with_parallel_mutation_returns_409_under_concurrency`.
   - **Comportement** : ajouter `tokio::join!` sur 2 closures qui exécutent (a) tx1 modification non-no-op via PUT `/api/v1/invoices/{id}` avec changement réel sur les `lines[]` (e.g. modifier `description` ou `unit_price` d'une ligne existante — **NE PAS** modifier `total_amount` qui est server-computed à partir des lignes et n'existe pas dans `UpdateInvoiceRequest` `crates/kesh-api/src/routes/invoices.rs:86` qui contient uniquement `{ contact_id, date, due_date, payment_terms, lines, version }`), (b) tx2 PUT no-op (payload identique au snapshot v=N initial). Les 2 tx s'exécutent en parallèle via pool partagé.
   - **Pré-condition** : créer une facture initiale v=N via PUT créé séquentiel (avec contact + items minimaux) avant le `tokio::join!`. Les 2 tx parallèles ciblent la même facture.
   - **Assertion mise à jour** : asserter que **au moins l'un** des 2 retours est `409 OPTIMISTIC_LOCK_CONFLICT` (le perdant de la race). L'autre retourne `200 OK` (le gagnant). Pattern :
     ```rust
     let (resp_a, resp_b) = tokio::join!(tx_a, tx_b);
     let statuses = [resp_a.status(), resp_b.status()];
     assert!(statuses.contains(&StatusCode::CONFLICT), "expected at least one 409, got {:?}", statuses);
     assert!(statuses.contains(&StatusCode::OK), "expected at least one 200, got {:?}", statuses);
     ```
   - **Anti-pattern (à éviter)** : asserter `200 + stale` (comportement v0.1 obsolète pré-#49). Le `SELECT FOR UPDATE` de #49 ferme la race — `409` est le comportement actuel et attendu.

9. **And** un commentaire inline `/// KF-021 (closes #50) regression detector for KF-020 SELECT FOR UPDATE (closes #49)` documenté en tête de la fonction — si un futur refactor retire accidentellement `FOR UPDATE` de `invoices::update`, le test échouera (`200 stale` au lieu de `409`) → red signal en CI.

10. **And** le test reste déterministe en CI : viser ≥ 99% de taux de détection de la race en concurrence. **Garde-fou** : si non-déterminisme observé après 5 runs locaux (i.e. test pass partiel < 5/5), basculer sur Approche 3 (stress loop N=100 + assert présence ≥ 1 cas 409) documentée Scope §"Approche concurrence à privilégier".

11. **And** le test ne dépend PAS d'un ordre spécifique des 2 tx (race symétrique — le `tokio::join!` non-déterministe doit produire `409` peu importe quelle tx gagne la course au `FOR UPDATE` X-lock).

### Phase C — Validation + closure

12. **Given** Phase A + Phase B complétées, **When** les baselines finales sont prises :
    - `cd frontend && npx playwright test fiscal-years.spec.ts --reporter=list 2>&1 | tee tests/e2e/baseline-post-9-5-1d-fiscal-years.log`
    - `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1 2>&1 | tee crates/kesh-api/tests/baseline-post-9-5-1d-kf004.log`
    
    **Then** :
    - Frontend : `fiscal-years.spec.ts` tous tests pass (incluant les 3 nouveaux AC #22 + les tests existants `Page exercices — affichage` + `création + clôture`). **0 test skipped** (sauf legacy `test.skip(condition, ...)` si présent pour autres raisons).
    - Backend : `kf004_no_op_e2e` tests tous pass, incluant le test renommé `no_op_with_parallel_mutation_returns_409_under_concurrency`.

13. **And** 2 commits closure dédiés :
    - `fix(e2e/fiscal-years): close KF #47 KF-019 implement AC #22 fallback toast tests (closes #47)` — body documente les 3 tests (`FISCAL_YEAR_INVALID` + `NO_FISCAL_YEAR` + `FISCAL_YEAR_CLOSED`) + les call sites helper testés.
    - `fix(api): close KF #50 KF-021 deterministic concurrent test for no-op race (closes #50)` — body documente le refactor rename + `tokio::join!` + assertion 409 + regression detector pour KF-020 #49.

### Test Locally First + non-régression

14. **Given** la story 9-5-1d touche 1 fichier `.spec.ts` (Playwright) + 1 fichier `.rs` (test Rust) — pas de modif source applicative, **When** un commit est créé, **Then** la règle CLAUDE.md `Test Locally First` s'applique avec scope adapté :
    - Backend Rust : `cargo fmt --all -- --check` + `cargo build --workspace --all-targets` + `cargo clippy --workspace --all-targets -- -D warnings` + **`cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1`** (test serial obligatoire pour intégration DB cohérent CI `ci.yml`).
    - Frontend Svelte : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` — les 4 checks obligatoires.
    - E2E : la suite de tests modifiée EST le travail de la story. Run de la suite complète `fiscal-years.spec.ts` sert d'évidence Test Locally First sur le scope.

15. **And** aucune régression sur les autres tests E2E hors scope 9-5-1d : run rapide `npx playwright test auth.spec.ts contacts.spec.ts homepage-settings.spec.ts invoices.spec.ts products.spec.ts reports.spec.ts users.spec.ts --grep-invert "axe a11y|axe-core"` (cohérent baseline 9-5-1c T10.3). Les 5 fails invoices pré-existants (`invoices.spec.ts:97/125/249/271/297`) restent pré-existants — pas de nouvelle régression introduite.

16. **And** aucune régression sur les autres tests Rust : `cargo test --workspace -j1 -- --test-threads=1` (cohérent CI) — 0 nouveau fail introduit par les modifs de `kf004_no_op_e2e.rs`.

17. **And** `npm run test:unit` reste à 253/253 pass post-changes (aucun test Vitest touché — la story ne modifie pas la source applicative frontend ni les utilities).

### Closure GitHub Issues + sprint-status

18. **Given** Phase A + B + C complétées, **When** la story est marquée done, **Then** :
    - **KF #47** : fermée systématiquement par commit AC #13 (`closes #47`).
    - **KF #50** : fermée systématiquement par commit AC #13 (`closes #50`).
    - `sprint-status.yaml` : `9-5-1d-kf-fix-misc` `backlog → ready-for-dev → in-progress → review → done`. Epic 9.5 progression `5/7 → 6/7 stories done` (restent 9-5-4 backlog + epic-9-5-retrospective optional).

## Tasks / Subtasks

- [ ] **T1** Pré-flight environnement (AC: #1)
  - [ ] T1.1 Vérifier branche `chore/epic-9-5-planning` checkée + working tree propre (hors test-results untracked attendus).
  - [ ] T1.2 Backend kesh-api running KESH_TEST_MODE=true + KESH_HOST=127.0.0.1 (réutiliser si déjà up depuis 9-5-1c, sinon redémarrer). Sanity check `curl -fsS http://127.0.0.1:3000/api/v1/_test/seed -X POST -H 'Content-Type: application/json' -d '{"preset":"with-company"}'` → `{"preset":"with-company","ok":true}`. **Important** : ce curl truncate + re-seed la DB volontairement pour repartir d'un état déterministe.
  - [ ] T1.3 `cargo build --workspace` propre. `cd frontend && npm run build` propre.
  - [ ] T1.4 `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` passé inline aux commandes Playwright.

- [ ] **T2** Phase A baseline pré-fix `fiscal-years.spec.ts` (AC: #2)
  - [ ] T2.1 Run `cd frontend && PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test fiscal-years.spec.ts --reporter=list 2>&1 | tee tests/e2e/baseline-pre-9-5-1d-fiscal-years.log`.
  - [ ] T2.2 Confirmer `FISCAL_YEAR_INVALID déclenche le toast actionnable` ligne 100-125 est skipped (`1 skipped` dans le reporter). Vérifier `grep -c "test.skip" frontend/tests/e2e/fiscal-years.spec.ts` ≥ 1.
  - [ ] T2.3 `git add -f tests/e2e/baseline-pre-9-5-1d-fiscal-years.log` (cohérent précédent 9-5-1b/c force-add). Commit `chore(9-5-1d): baseline pre-fix fiscal-years AC #22 — 1 skipped test`.

- [ ] **T3** Phase B baseline pré-fix `kf004_no_op_e2e.rs` (AC: #2)
  - [ ] T3.1 Run `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1 2>&1 | tee crates/kesh-api/tests/baseline-pre-9-5-1d-kf004.log`.
  - [ ] T3.2 Confirmer `no_op_with_parallel_mutation_returns_409_when_sequential` pass en smoke séquentiel. Vérifier `grep -nE "fn no_op_with_parallel_mutation_returns_409_when_sequential" crates/kesh-api/tests/kf004_no_op_e2e.rs` retourne ligne 707.
  - [ ] T3.3 `git add -f crates/kesh-api/tests/baseline-pre-9-5-1d-kf004.log` + commit `chore(9-5-1d): baseline pre-fix kf004_no_op_e2e — smoke séquentiel pass`.

- [ ] **T4** Phase A — Implémentation 3 tests Playwright AC #22 (AC: #3, #4, #5, #6, #7)
  - [ ] T4.1 Lire `frontend/src/lib/shared/utils/notify.ts:98-123` pour comprendre exactement les messages i18n et le label action du toast. Lire `frontend/src/routes/(app)/invoices/[id]/+page.svelte:83-130` pour comprendre le flow `validateInvoice` + integration `notifyMissingFiscalYearOrFallback`. Lire `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:140-145` pour comprendre le flow `JournalEntryForm` + intégration helper.
  - [ ] T4.2 **Test 1 `FISCAL_YEAR_INVALID`** : créer test via flow `validateInvoice` end-to-end (POST `/api/v1/invoices` avec `issueDate: '1900-01-01'` → naviguer `/invoices/<id>` → clic « Valider »). Vérifier toast `role="alert"` avec message « Créez d'abord un exercice » + clic action → `/settings/fiscal-years`. Sélecteurs à adapter selon le DOM réel.
  - [ ] T4.3 **Test 2 `NO_FISCAL_YEAR`** : override `seedTestState('with-company-no-fy')` au début du test (le default beforeEach utilise `with-company`). **Re-login obligatoire après l'override seed** : appeler `login(page)` (helper projet) pour ré-authentifier — sans ça, les requêtes API tombent en cascade 401 et le toast n'est jamais déclenché (le re-seed truncate les sessions). Flow JournalEntryForm — naviguer `/journal-entries`, ouvrir form, remplir minimum, soumettre. Vérifier toast distinct + clic action.
  - [ ] T4.4 **Test 3 `FISCAL_YEAR_CLOSED`** : preset `with-company`, puis appel API direct pour clôturer fiscal_year — **vérifier l'endpoint exact dans `crates/kesh-api/src/routes/fiscal_years.rs` avant** (probablement `POST /api/v1/fiscal-years/{id}/close` ou `PATCH` selon convention projet). **Routage critique** : `FISCAL_YEAR_CLOSED` n'est PAS levé par `validate_invoice` (qui retourne `FISCAL_YEAR_INVALID` via `find_open_covering_date` `invoices.rs:970`). `FISCAL_YEAR_CLOSED` est levé **uniquement** par `journal_entries::create` (`journal_entries.rs:109`) + `journal_entries::update` (`journal_entries.rs:598` + `:836`). **Test 3 doit donc utiliser le flow `JournalEntryForm`**, pas `validateInvoice`. Soumettre une écriture avec date in-range (e.g. `2025-06-15`) après clôture du FY. Vérifier toast distinct « clôturé » (≠ test 2 « Créez d'abord »).
  - [ ] T4.5 Retirer le `test.skip(true, ...)` ligne 121 et le bloc commentaire qui le précède (lignes 94-99 — l'explication Pass 1 F7 n'est plus pertinente une fois les vrais tests implémentés). Préserver le `test.describe('AC #22 — fallback toast actionnable')` + restructurer le bloc avec les 3 sous-tests.
  - [ ] T4.6 `npm run check` : 0 errors Svelte (25 warnings pré-existants acceptables — pas de nouveau warning sur le test modifié).
  - [ ] T4.7 Run isolé chaque test : `npx playwright test fiscal-years.spec.ts:<line> --reporter=list` pour chacun (vérification isolation).
  - [ ] T4.8 Run groupé : `npx playwright test fiscal-years.spec.ts --reporter=list` — confirmer 3 tests AC #22 pass + tests existants pass (affichage + création/clôture).

- [ ] **T5** Phase B — Refactor test concurrent KF #50 (AC: #8, #9, #10, #11)
  - [ ] T5.1 Lire `crates/kesh-api/tests/kf004_no_op_e2e.rs:707-792` (fonction `no_op_with_parallel_mutation_returns_409_when_sequential`) pour comprendre le setup actuel — **important** : la fonction utilise `/api/v1/contacts` (lignes 720, 743, 769), pas `/api/v1/invoices`. Le refactor doit **changer l'entité** à `invoices` (cf. Scope §"Changement d'entité critique").
  - [ ] T5.2 Lire `crates/kesh-api/tests/kf004_no_op_e2e.rs:345-485` (fonction `put_invoice_no_op_returns_200_unchanged_version`) — c'est le **vrai** pattern de référence pour le setup invoice (POST `/api/v1/contacts` puis POST `/api/v1/invoices` avec `lines`). La fonction `concurrent_no_op_returns_200_200_not_200_409` (ligne 488) est **séquentielle** malgré son nom (2 PUT consécutifs sans `tokio::join!`) — vérifié ground-truth : `grep -nF "tokio::join" kf004_no_op_e2e.rs` retourne uniquement 2 mentions en commentaires (lignes 689, 789), aucune invocation. **Conclusion** : le pattern `tokio::join!` doit être **créé depuis zéro** dans cette story.
  - [ ] T5.3 Choisir l'**approche concurrence** parmi Scope §"Approche concurrence à privilégier" (privilégier Approche 1 : `tokio::join!` sur 2 closures async — le `pool` partagé suffit, les helpers `app.client.put(...)` via `reqwest::Client` gèrent la concurrence HTTP, et `kesh-db/repositories/invoices.rs::update()` gère sa propre `pool.acquire()` interne).
  - [ ] T5.4 **Renommer** la fonction : `no_op_with_parallel_mutation_returns_409_when_sequential` → `no_op_with_parallel_mutation_returns_409_under_concurrency`. Mettre à jour tout commentaire/doc associé (incluant le module doc-comment ligne 13-19 qui réfère au test séquentiel).
  - [ ] T5.5 **Réécrire le corps complet** : (a) setup — `create_seeded_company` (déjà fait, fournit `company_id` + fiscal_year), puis créer 2 users alice/bob (déjà fait), POST `/api/v1/contacts` (1 contact pour la facture), POST `/api/v1/invoices` (1 facture initiale v=N avec 1 ou 2 lignes via `CreateInvoiceLineRequest` — voir `put_invoice_no_op_returns_200_unchanged_version:345` pour le pattern exact JSON) ; (b) concurrent — `let (resp_a, resp_b) = tokio::join!(tx_a, tx_b)` où `tx_a` modifie réellement la facture (e.g. changer `description` ou `unit_price` d'une ligne dans `lines[]` — **pas** `total_amount` qui est server-computed et n'existe pas dans `UpdateInvoiceRequest` `routes/invoices.rs:86`) et `tx_b` envoie un no-op (payload identique au snapshot v=N initial). Asserter `statuses.contains(&CONFLICT) && statuses.contains(&OK)`.
  - [ ] T5.6 **Doc commentaire** : ajouter `/// KF-021 (closes #50) regression detector for KF-020 SELECT FOR UPDATE (closes #49)` en tête de la fonction. Préserver les comments T0-T6 existants si pertinents (adapter pour décrire la nouvelle race window post-#49).
  - [ ] T5.7 **Verif déterminisme** : exécuter le test 5 fois localement (`for i in 1 2 3 4 5; do cargo test ... no_op_with_parallel_mutation_returns_409_under_concurrency; done`). Si tous 5 pass → déterministe ≥ 99%, OK. Si < 5/5 pass → basculer sur Approche 3 (stress loop) — modifier le test pour boucler N=100 itérations et asserter `count(409) >= 1`. Documenter le choix final dans le commentaire.
  - [ ] T5.8 `cargo clippy --workspace --all-targets -- -D warnings` : 0 warning (les nouveaux unused imports `tokio::join` ou similaire doivent être proprement gérés).

- [ ] **T6** Phase C — Validation + commits closure (AC: #12, #13)
  - [ ] T6.1 Baseline finale frontend : `cd frontend && PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test fiscal-years.spec.ts --reporter=list 2>&1 | tee tests/e2e/baseline-post-9-5-1d-fiscal-years.log`. Confirmer 3 nouveaux tests AC #22 pass + tests existants pass + **0 skipped** sur AC #22.
  - [ ] T6.2 Baseline finale backend : `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1 2>&1 | tee crates/kesh-api/tests/baseline-post-9-5-1d-kf004.log`. Confirmer `no_op_with_parallel_mutation_returns_409_under_concurrency` pass + autres tests pass.
  - [ ] T6.3 `git add -f tests/e2e/baseline-post-9-5-1d-fiscal-years.log crates/kesh-api/tests/baseline-post-9-5-1d-kf004.log`.
  - [ ] T6.4 Commit 1 KF #47 : `fix(e2e/fiscal-years): close KF #47 KF-019 implement AC #22 fallback toast tests (closes #47)` — body cite les 3 tests + call sites helper testés + baseline post log.
  - [ ] T6.5 Commit 2 KF #50 : `fix(api): close KF #50 KF-021 deterministic concurrent test for no-op race (closes #50)` — body cite le refactor rename + `tokio::join!` + assertion 409 + regression detector pour KF-020 (#49).

- [ ] **T7** Test Locally First — checks CI complets (AC: #14, #15, #16, #17)
  - [ ] T7.1 Backend Rust : `cargo fmt --all -- --check` + `cargo build --workspace --all-targets` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1` (les 4 checks Rust obligatoires pour cette story qui modifie un test Rust).
  - [ ] T7.2 Frontend Svelte : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` (4 checks doivent tous passer, 25 warnings pré-existants acceptables).
  - [ ] T7.3 AC #15 non-régression E2E : `cd frontend && PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test auth.spec.ts contacts.spec.ts homepage-settings.spec.ts invoices.spec.ts products.spec.ts reports.spec.ts users.spec.ts --grep-invert "axe a11y|axe-core" --reporter=list 2>&1 | tee tests/e2e/non-regression-9-5-1d.log`. Comparer aux baselines 9-5-1c : aucune **nouvelle** régression introduite (les 5 fails invoices pré-existants restent pré-existants).
  - [ ] T7.4 AC #16 non-régression Rust : `cargo test --workspace -j1 -- --test-threads=1` — 0 nouveau fail introduit. Si MariaDB pas démarré localement, ce check peut être skippé (les tests intégration nécessitent MariaDB). En revanche, `cargo test --workspace` (parallel, unit only) doit pass.
  - [ ] T7.5 Push branche `chore/epic-9-5-planning` reporté à fin Epic 9.5 (pattern « avoid parallel PRs »).

- [ ] **T8** Documentation finale + sprint-status (AC: #18)
  - [ ] T8.1 Mise à jour `_bmad-output/implementation-artifacts/sprint-status.yaml` : entrée `9-5-1d-kf-fix-misc` → `in-progress` (start T2/T3) → `review` (après T7) → `done` (après code-review converge).
  - [ ] T8.2 Update `last_updated` field sprint-status.yaml header.
  - [ ] T8.3 Build doc Change Log avec : (a) baselines T2 + T3 + post T6.1 + T6.2, (b) approche concurrence retenue T5.3 + résultat verif déterminisme T5.7, (c) commits closure T6.4 + T6.5.

## Dev Notes

### Cadrage scope minimal — pattern Epic 9.5 sub-stories

Cette story 9-5-1d suit la discipline Epic 9.5 (scope minimaliste, file-list explicite, anti-pattern Story 7-1 historique). Ici le scope est strictement **2 fichiers** :
- `frontend/tests/e2e/fiscal-years.spec.ts` (KF #47)
- `crates/kesh-api/tests/kf004_no_op_e2e.rs` (KF #50)

Aucun fichier source applicatif modifié (helper `notify.ts` + call sites déjà implémentés Story 3-7 ; `SELECT FOR UPDATE` `invoices.rs` déjà appliqué commit `ebdea4b` #49). 2 KFs indépendantes — pas de cycle dépendance entre Phase A et Phase B.

### Context KF #47 — état helper + call sites

**Helper** : `frontend/src/lib/shared/utils/notify.ts:98-123`
```ts
export function notifyMissingFiscalYearOrFallback(err: ApiError): boolean {
    const isMissing = (FY_MISSING_CODES as readonly string[]).includes(err.code);
    const isClosed = err.code === FY_CLOSED_CODE;
    if (!isMissing && !isClosed) return false;

    const message = isClosed
        ? i18nMsg('error-fiscal-year-closed-for-date', "L'exercice qui couvre cette date est clôturé. Vérifiez la date saisie ou consultez vos exercices.")
        : i18nMsg('error-fiscal-year-missing', "Créez d'abord un exercice comptable dans Paramètres → Exercices");

    toast.error(message, {
        duration: ERROR_DURATION,
        action: {
            label: i18nMsg('go-to-settings', 'Ouvrir Paramètres'),
            onClick: () => void goto('/settings/fiscal-years'),
        }
    });
    return true;
}
```

**Call sites instrumentés** :
- `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:140-141`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte:94-96`

**Codes erreur backend** (ground-truth `notify.ts:71-72`) :
- `FY_MISSING_CODES = ['FISCAL_YEAR_INVALID', 'NO_FISCAL_YEAR'] as const`.
- `FY_CLOSED_CODE = 'FISCAL_YEAR_CLOSED'`.
- L'ordre dans le tableau est sans importance fonctionnelle (`.includes()` est order-agnostic ligne 99).

**Toast UI** : `svelte-sonner` library, role `alert` sur le container rendu, button action label = i18n `go-to-settings`.

### Context KF #50 — état migration + pattern référence

**Migration #49 closed** par commit `ebdea4b` `fix(db): KF-020 SELECT FOR UPDATE in invoices::update (closes #49) (#64)`. Le code actuel `crates/kesh-db/src/repositories/invoices.rs:674` utilise `&format!("{FIND_INVOICE_SCOPED_SQL} FOR UPDATE")` au début de `update()`. La race T0-T6 documentée issue #49 n'est plus reproductible — `200 stale` est devenu `409 OPTIMISTIC_LOCK_CONFLICT`.

**Pattern référence concurrent** : **vérifié ground-truth Pass 1 validate** — `grep -nF "tokio::join" crates/kesh-api/tests/kf004_no_op_e2e.rs` retourne 2 mentions UNIQUEMENT en commentaires (lignes 689 + 789), AUCUNE invocation exécutable. La fonction `concurrent_no_op_returns_200_200_not_200_409` (ligne 488) — malgré son nom — est **séquentielle** (2 PUT contacts consécutifs sans `tokio::join!`). **Conclusion** : aucun pattern `tokio::join!` existant à réutiliser. Le pattern doit être créé depuis zéro. **Vrai source de référence pour setup invoice** : `put_invoice_no_op_returns_200_unchanged_version` (ligne 345) — montre POST contact + POST invoice + PUT invoice no-op.

### Pattern bits-ui + svelte-sonner toast assertion E2E

**Toast Playwright selector recommandé** :
```ts
const alertLocator = page.getByRole('alert').filter({ hasText: /exercice comptable/ });
await expect(alertLocator).toBeVisible();
const actionButton = alertLocator.getByRole('button', { name: /ouvrir paramètres/i });
await actionButton.click();
await expect(page).toHaveURL(/\/settings\/fiscal-years/);
```

**Pitfall connu** : `svelte-sonner` peut rendre plusieurs toasts simultanément (queue). Si le test précédent (`beforeEach` seed) déclenche un toast résiduel, le selector `getByRole('alert')` peut retourner plusieurs éléments. **Mitigation** : utiliser `.filter({ hasText: ... })` pour cibler le bon toast, OU appeler `await page.evaluate(() => window.dismissAllToasts?.())` si une API existe (à vérifier — probablement pas).

### Cascade KF #50 sur autres tests kf004_no_op_e2e

Les autres tests dans `kf004_no_op_e2e.rs` (5 tests : `put_contact_no_op_*`, `put_product_no_op_*`, `put_invoice_no_op_*`, `concurrent_no_op_*`, `no_op_then_real_conflict_*`) **ne sont PAS impactés** par le refactor du 6ème test. Ils continuent de tester leur scénario spécifique. Le renommage et la réécriture de `no_op_with_parallel_mutation_returns_409_when_sequential` est isolé.

### Risque R1 — non-déterminisme du test concurrent

Le `tokio::join!` sur 2 closures async ne garantit pas que les 2 tx s'interleavent exactement comme la race T0-T6 décrit. MariaDB InnoDB ordonne les transactions selon les locks acquis, et le `SELECT FOR UPDATE` du fix #49 sérialise nécessairement les 2 tx (l'une attend le X-lock de l'autre). Conséquence : le test peut être déterministe (la 2e tx voit toujours v=N+1 et retourne 409) OU non-déterministe (en théorie, si MariaDB ordonne tx_b avant tx_a, on a `200 + 200` au lieu de `200 + 409`).

**Mitigation Approche 3 (stress loop)** : si Approche 1 (`tokio::join!` simple) est non-déterministe en CI, basculer sur une boucle N=100 itérations qui asserte `count(409) >= 1`. Au moins une itération doit déclencher le `409` post-FOR UPDATE.

**Argument déterminisme attendu** : la fonction `update()` post-#49 fait `SELECT FOR UPDATE` étape 1, **inconditionnellement**. La 2e tx attend le X-lock. Quand le X-lock se libère (tx_a commit avec v=N+1), tx_b ré-SELECT et lit v=N+1, puis compare avec son payload v=N → mismatch → **409**. Le déterminisme post-FOR UPDATE est élevé (~99-100%). Approche 1 devrait suffire.

### Risque R2 — KF #50 cible obsolète

La spec originale KF #50 (issue #50) demandait d'asserter `200 + stale`. Cette spec est **obsolète post-#49 closed**. Le présent story 9-5-1d adapte la cible à `409`. Si un reviewer (humain ou LLM) consulte l'issue #50 GitHub et compare avec notre implémentation, il peut conclure à une déviation. **Mitigation** : documenter clairement dans le commit body T6.5 que la cible a évolué de `200 stale` (v0.1 obsolète) à `409` (post-#49 fix) — référencer commit `ebdea4b` qui a fermé #49.

### Memory carries

- `reference_playwright_ubuntu26` : obligatoire `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (déjà appliqué T1.4).
- `feedback_haiku_review_diff_combined` : discipline grep ground-truth obligatoire pour les passes Haiku review. Mitigation appliquée par diff aplati en code-review (`git diff base..HEAD`).
- `feedback_avoid_parallel_prs` : pas de PR séparée — branche `chore/epic-9-5-planning` cumul Epic 9.5.

### Project Structure Notes

- **Fichiers édités par 9-5-1d** :
  - `frontend/tests/e2e/fiscal-years.spec.ts` (Phase A — remplace `test.skip(true, ...)` par 3 vrais tests AC #22, ~80-120 LoC ajoutées).
  - `crates/kesh-api/tests/kf004_no_op_e2e.rs` (Phase B — rename + refactor `no_op_with_parallel_mutation_returns_409_when_sequential`, ~30-60 LoC modifiées).
  - `_bmad-output/implementation-artifacts/9-5-1d-kf-fix-misc.md` (cette spec, Change Log final).
  - `_bmad-output/implementation-artifacts/sprint-status.yaml` (statut entry).
  - `frontend/tests/e2e/baseline-pre-9-5-1d-fiscal-years.log` + `baseline-post-9-5-1d-fiscal-years.log` (force-added).
  - `crates/kesh-api/tests/baseline-pre-9-5-1d-kf004.log` + `baseline-post-9-5-1d-kf004.log` (force-added).

- **Fichiers NON touchés** :
  - **Aucun** fichier source applicatif `.svelte`/`.ts` modifié (helper + call sites Story 3-7 déjà implémentés).
  - **Aucun** fichier source applicatif `.rs` modifié (FOR UPDATE migration #49 déjà appliqué).
  - **Aucune** autre `.spec.ts` E2E modifiée (5 fails invoices pré-existants restent pré-existants).
  - **Aucun** test Vitest modifié.

- **GitHub Issues** :
  - **#47 fermée** systématiquement par T6.4 (`closes #47`).
  - **#50 fermée** systématiquement par T6.5 (`closes #50`).

### Testing standards summary

- **Pattern Playwright E2E AC #22** : tests utilisent `seedTestState` (override per-test si besoin), `page.getByRole('alert').filter({ hasText: ... })` pour toast selector, `page.getByRole('button', { name: ... }).click()` pour action button, `await expect(page).toHaveURL(...)` pour navigation assertion. Pattern cohérent avec autres specs E2E projet (auth, invoices, etc.).
- **Pattern Rust integration test** : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` + `truncate_all(&pool)` + `create_seeded_company` + `spawn_app` + `login` (helpers existants). Pour le concurrent test : `tokio::join!` sur 2 closures async qui partagent le `pool` (chaque task obtient sa `pool.acquire()` automatiquement).
- **Test isolation** : `truncate_all` au début de chaque test garantit DB clean. Pas de cross-pollution entre tests.

### Estimation effort

- **T1 (pré-flight)** : 10 min (backend déjà running depuis 9-5-1c si session continue).
- **T2-T3 (baselines)** : 15 min.
- **T4 (KF #47 — 3 tests Playwright)** : 2-3h (lecture call sites + écriture tests + verif isolation).
- **T5 (KF #50 — refactor concurrent)** : 1-2h (lecture pattern référence + réécriture + verif déterminisme).
- **T6 (validation + commits)** : 20 min.
- **T7 (Test Locally First)** : 30 min.
- **T8 (doc finale)** : 15 min.
- **Total** : ~4-6h.

### References

- [Source: _bmad-output/planning-artifacts/epic-9-5.md#Story-9.5-1] — spec parent (point 9.5-1d ligne 88).
- [Source: _bmad-output/implementation-artifacts/9-5-1a-kf-triage.md] — triage amont (mapping KF #47 + #50 → 9-5-1d misc).
- [Source: _bmad-output/implementation-artifacts/9-5-1b-kf-fix-infra.md] — pattern dev-story mode orchestré complet réutilisable + baselines force-add precedent.
- [Source: _bmad-output/implementation-artifacts/9-5-1c-kf-fix-a11y.md] — pattern code-review Pass 1 single-pass converged.
- [Source: frontend/src/lib/shared/utils/notify.ts:98-123] — helper `notifyMissingFiscalYearOrFallback` (testé indirectement par les 3 tests Playwright).
- [Source: frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:140-141] — call site `NO_FISCAL_YEAR`.
- [Source: frontend/src/routes/(app)/invoices/[id]/+page.svelte:94-96] — call site `FISCAL_YEAR_INVALID` + `FISCAL_YEAR_CLOSED`.
- [Source: frontend/tests/e2e/fiscal-years.spec.ts:93-126] — code cible KF #47 (bloc `test.describe('AC #22')` + `test.skip(true, ...)` ligne 121).
- [Source: crates/kesh-api/tests/kf004_no_op_e2e.rs:706-792] — code cible KF #50 (`no_op_with_parallel_mutation_returns_409_when_sequential`).
- [Source: crates/kesh-api/tests/kf004_no_op_e2e.rs:487-577] — pattern référence concurrent existant `concurrent_no_op_returns_200_200_not_200_409`.
- [Source: crates/kesh-db/src/repositories/invoices.rs:674] — `SELECT FOR UPDATE` appliqué par commit `ebdea4b` (#49 closed).
- [Source: crates/kesh-api/src/routes/test_endpoints.rs:20] — preset `with-company-no-fy` existant pour NO_FISCAL_YEAR case.
- [GitHub Issue #47 KF-019] — Story 3-7 AC #22 Playwright E2E coverage gap.
- [GitHub Issue #50 KF-021] — Test E2E déterministe pour AC #29 race REPEATABLE READ.
- [GitHub Issue #49 KF-020] — invoices::update SELECT FOR UPDATE (CLOSED par commit `ebdea4b`).
- [Source: CLAUDE.md§Test Locally First] — checks CI obligatoires.
- [Source: CLAUDE.md§Review Iteration Rule] — cycle review 2-3 passes attendues, LLM différent par passe.
- [Source: CLAUDE.md§Règle de splitting préventif] — discipline file-list explicite (ici 2 fichiers, pas de split nécessaire).

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Pass 1 spec validate — 2026-05-20, Sonnet 4.6 (subagent contexte frais)

**Verdict trend brut** : 1 CRITICAL + 2 HIGH + 2 MEDIUM + 2 LOW = 7 findings (Convergence : NON).

**Discipline grep ground-truth Sonnet** : 5/5 patches majeurs grep-vérifiés par le reviewer + cross-checked par l'orchestrateur (chaque CRITICAL/HIGH/MEDIUM vérifié par `grep -nF` ou `Read` direct des fichiers cités).

**Patches appliqués (5/7 — 2 LOW dismissed)** :

1. **CRITICAL — Test 3 `FISCAL_YEAR_CLOSED` routage incorrect** : `validate_invoice` utilise `find_open_covering_date` (`invoices.rs:970`) qui retourne `FiscalYearInvalid` (PAS `FiscalYearClosed`) pour un FY clos. `FiscalYearClosed` est levé uniquement par `journal_entries::create` (`journal_entries.rs:109`) + `journal_entries::update` (`journal_entries.rs:598 + :836`). **Patch** : AC #5 + T4.4 + Scope §"KF #47" reroutés sur flow `JournalEntryForm` (journal_entries::create) avec submit écriture comptable après clôture FY. Le toast distinct « clôturé » devient testable. Justification ground-truth : `grep -n "FiscalYearClosed" crates/kesh-db/src/repositories/*.rs` → 5 hits dans journal_entries.rs uniquement, 0 dans invoices.rs.

2. **HIGH-01 — Pattern référence `tokio::join!` inexistant dans `kf004_no_op_e2e.rs`** : la spec affirmait que `concurrent_no_op_returns_200_200_not_200_409` (ligne 488) utilisait `tokio::join!` comme pattern référence. Ground-truth : `grep -nF "tokio::join" crates/kesh-api/tests/kf004_no_op_e2e.rs` → 2 mentions uniquement en commentaires (lignes 689, 789), 0 invocation exécutable. La fonction référencée est séquentielle (2 PUT consécutifs). **Patch** : Scope §"Pattern référence" + Dev Notes §"Context KF #50" + T5.2 mis à jour pour clarifier : (a) aucun pattern `tokio::join!` existant à réutiliser, le pattern doit être créé from scratch ; (b) vrai source de référence pour setup invoice = `put_invoice_no_op_returns_200_unchanged_version` (`kf004_no_op_e2e.rs:345`).

3. **HIGH-02 — KF #50 entity switch `contacts` → `invoices` non explicite** : le test courant (`no_op_with_parallel_mutation_returns_409_when_sequential`) utilise `/api/v1/contacts` (`kf004_no_op_e2e.rs:720, 743, 769`), mais le SELECT FOR UPDATE (#49) est appliqué uniquement à `invoices::update`. Le refactor doit changer l'entité de contacts à invoices, ce qui implique réécriture complète du setup (POST contact + POST invoice avec lignes). **Patch** : Scope §"Refactor approche" + T5.5 mis à jour pour expliciter le changement d'entité + référence `put_invoice_no_op_returns_200_unchanged_version:345` pour le setup pattern.

4. **MEDIUM-01 — `total_amount` non settable via `UpdateInvoiceRequest`** : la spec disait « modifier `total_amount` » comme mutation réelle. Ground-truth : `crates/kesh-api/src/routes/invoices.rs:86-95` montre `UpdateInvoiceRequest { contact_id, date, due_date, payment_terms, lines, version }` — pas de `total_amount` (computed server-side from `lines`). **Patch** : AC #8 + T5.5 remplacés « modifier `total_amount` » par « changer `description` ou `unit_price` d'une ligne dans `lines[]` ».

5. **MEDIUM-02 — Test 2 NO_FISCAL_YEAR re-login après seed override** : la spec instruisait `seedTestState('with-company-no-fy')` mid-test mais n'incluait pas l'étape `login(page)` requise après le truncate + re-seed (invalide les sessions JWT). Sans re-login, cascade 401 et le toast NO_FISCAL_YEAR n'est jamais déclenché. **Patch** : AC #4 + T4.3 ajoutent l'étape `login(page)` obligatoire après l'override seed avec justification cascade 401.

**Findings dismissed (2 LOW)** :

- **LOW-01 — Dev Notes inline code block compacté** : observation cosmétique sur le compactage du `notifyMissingFiscalYearOrFallback` dans la section Dev Notes (les lignes `i18nMsg(...)` multi-lignes sont compactées). Pas d'impact implémentation. **Dismiss**.
- **LOW-02 — Sprint-status progression `5/7 → 6/7` à vérifier** : observation que l'implémenteur doit confirmer le compteur réel au démarrage de la story (cohérent rapide). Pas un défaut de spec — vérification routine T1. **Dismiss**.

**Cross-verification orchestrateur ground-truth** (avant application patches) :

```
grep -n "find_open_covering_date" crates/kesh-db/src/repositories/invoices.rs
→ 906:/// 2. fiscal_years::find_open_covering_date
→ 970:        let fy = fiscal_years::find_open_covering_date(...)

grep -n "FiscalYearClosed" crates/kesh-db/src/repositories/journal_entries.rs
→ 109, 514, 598, 798, 836, 1180, 1181, 1997 (8 hits dans journal_entries.rs)
grep -n "FiscalYearClosed" crates/kesh-db/src/repositories/invoices.rs
→ 0 hit dans invoices.rs ✓ (confirme CRITICAL Sonnet)

grep -F "tokio::join" crates/kesh-api/tests/kf004_no_op_e2e.rs
→ 689:/// concurrente (`tokio::join!` ou ...)   ← commentaire
→ 789:        "exécution séquentielle ... (... tokio::join concurrent ...)   ← commentaire
0 invocation exécutable ✓ (confirme HIGH-01 Sonnet)

sed -n '707,792p' crates/kesh-api/tests/kf004_no_op_e2e.rs | grep "api/v1/"
→ 3 occurrences "api/v1/contacts" lignes 720, 743, 769 ✓ (confirme HIGH-02 Sonnet)

cat crates/kesh-api/src/routes/invoices.rs lignes 86-95
→ struct UpdateInvoiceRequest { contact_id, date, due_date, payment_terms, lines, version }
0 champ total_amount ✓ (confirme MEDIUM-01 Sonnet)
```

**Recommandation Sonnet** : Pass 2 Haiku 4.5 avec discipline grep ground-truth obligatoire (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`). Vérifier propagation patches Pass 1 + chercher inconsistances résiduelles AC↔T mapping post-patches CRITICAL.

**Modèle Pass 1** : Sonnet 4.6 (subagent isolé contexte frais — spec créée par Opus 4.7, règle CLAUDE.md `LLM différent passe précédente` respectée).

### Pass 2 spec validate — 2026-05-20, Haiku 4.5 (subagent contexte frais)

**Verdict trend brut** : 0 CRITICAL + 0 HIGH + 0 MEDIUM + 1 LOW = 1 finding (Convergence : **OUI** — critère CLAUDE.md « Uniquement findings LOW » atteint).

**Discipline grep ground-truth Haiku** appliquée — 11/11 verifications positives. Toutes les 5 patches Pass 1 (1C+2H+2M) confirmées propagées correctement par Haiku via Read direct du spec post-Pass 1. **Aucune hallucination Haiku** observée sur ce cycle (la mitigation diff aplati cross-modèle n'a pas eu de cas pathologique à traiter — la spec est un fichier unique, pas un diff multi-commit).

**Ground-truths cross-verified par Haiku** :
- ✓ `validateInvoice` route inchangée pour Test 1 (FY_INVALID), Test 3 routé sur JournalEntryForm.
- ✓ `tokio::join!` claim retiré, pattern référence corrigée vers `put_invoice_no_op_returns_200_unchanged_version:345`.
- ✓ Entity switch `contacts` → `invoices` documenté explicitement Scope + T5.5.
- ✓ `total_amount` remplacé par `description`/`unit_price` (ground-truth `UpdateInvoiceRequest:86-95`).
- ✓ `login(page)` step ajouté AC #4 + T4.3 avec justification cascade 401.
- ✓ `FY_MISSING_CODES` + `FY_CLOSED_CODE` existent (`notify.ts:71-72`).
- ✓ Helper `login(page)` défini localement `fiscal-years.spec.ts:29`.
- ✓ Preset `with-company-no-fy` existe (`test_endpoints.rs:20`).
- ✓ Endpoint `POST /api/v1/fiscal-years/{id}/close` confirmé.
- ✓ Effort estimation §"Estimation effort" reste raisonnable post-patches (T5 2-3h cohérent).

**Patch appliqué (1 LOW polish)** :

1. **LOW-01 — `FY_MISSING_CODES` ordre cosmétique** : Dev Notes claimait `['NO_FISCAL_YEAR', 'FISCAL_YEAR_INVALID']` mais ground-truth `notify.ts:71` montre `['FISCAL_YEAR_INVALID', 'NO_FISCAL_YEAR'] as const`. Impact fonctionnel **nul** (`includes()` ligne 99 est order-agnostic), polish documentation. **Patch** : aligné Dev Notes sur l'ordre source réel + note explicite « ordre sans importance fonctionnelle ».

**Trend cumul cycle 2-passes** :
- Pass 1 Sonnet 4.6 : 1C+2H+2M+2L = 7 findings → 5 patches + 2 LOW dismissed.
- Pass 2 Haiku 4.5 : 0C+0H+0M+1L = 1 finding → 1 LOW polish → **0 résiduel**.
- **Total : 6 patches sur 2 passes. Cycle court (Sonnet → Haiku) cohérent 9-5-1b spec validate done en 2 passes.**

**Cycle complet `Sonnet → Haiku`** : convergence atteinte sans nécessité Opus Pass 3 (scope minimaliste 2 fichiers — pas de subtilité architecturale qui requiert Opus). Pattern cohérent avec retro Epic 9 Insight I1 « Opus catches subtle stuff » qui s'applique aux scopes complexes — pas le cas ici.

**Modèle Pass 2** : Claude Haiku 4.5 (subagent isolé, contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée Sonnet → Haiku).

**Statut final spec** : `ready-for-dev` confirmé. Prête pour `bmad-dev-story 9-5-1d` (mode orchestré complet attendu cohérent 9-5-1b/c, LLM recommandé Opus 4.7 ou Sonnet 4.6 — différent de Pass 2 Haiku).
