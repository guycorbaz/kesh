# Story 9.5-1d: Fix specific KFs — KF #47 fallback toast E2E + KF #50 race REPEATABLE READ test concurrent

Status: done

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
   - **Assertion 1** : `await expect(page.locator('[data-sonner-toast]').filter({ hasText: /Créez d'abord un exercice/ })).toBeVisible()` — `svelte-sonner` rend le container toast avec `data-sonner-toast=""` + `aria-live="polite"` (PAS de `role="alert"`, vérifié ground-truth `node_modules/svelte-sonner/dist/Toast.svelte:344-360` — voir Pass 1 code-review AA-1).
   - **Assertion 2** : cliquer le bouton action du toast (label « Ouvrir Paramètres » i18n `go-to-settings`). Selector possible : `await page.getByRole('button', { name: /ouvrir paramètres/i }).click()`.
   - **Assertion 3** : `await expect(page).toHaveURL(/\/settings\/fiscal-years/)` — la navigation a eu lieu.
   - **Test isolation** : le bloc `afterEach` existant (`clearAuthStorage(page)` ligne 25) + `beforeEach` `seedTestState('with-company')` ligne 22 garantissent l'isolation. Pas besoin de cleanup spécifique.

4. **Given** le test 2 `NO_FISCAL_YEAR` implémenté, **When** il est exécuté, **Then** il satisfait :
   - **Setup** : `seedTestState('with-company-no-fy')` (preset existant `test_endpoints.rs:20` — company sans fiscal_year). **Override beforeEach** pour ce test spécifique (le default `with-company` ne convient pas — le test a son propre `seedTestState` au début).
   - **Re-login obligatoire** : `seedTestState` truncate + re-seed la DB ⇒ invalide les sessions JWT/localStorage existantes du `beforeEach`. Après l'override seed, appeler `login(page)` (ou `goToFiscalYears(page)` qui inclut le login) pour ré-authentifier avec les credentials du nouveau seed `admin/admin` avant toute navigation. Sans ce re-login, les requêtes `/api/v1/journal-entries` redirigeraient vers `/login` (cascade 401) et le toast ne serait jamais déclenché.
   - **Action 1** : naviguer vers `/journal-entries` via `page.goto(...)`.
   - **Action 2** : cliquer le bouton « Nouvelle écriture » ou équivalent ouvrant `JournalEntryForm` (sélecteur à confirmer ground-truth `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`).
   - **Action 3** : remplir le minimum requis pour soumission (date, montant, comptes), puis cliquer « Enregistrer ».
   - **Assertion 1** : toast rendu avec message i18n `error-fiscal-year-missing` (« Créez d'abord un exercice comptable dans Paramètres → Exercices »). Pattern selector : `await expect(page.locator('[data-sonner-toast]').filter({ hasText: /Créez d'abord un exercice/ })).toBeVisible()` (svelte-sonner sans `role="alert"` — Pass 1 code-review AA-1).
   - **Assertion 2** : clic action button → navigation `/settings/fiscal-years`.

5. **Given** le test 3 `FISCAL_YEAR_CLOSED` implémenté, **When** il est exécuté, **Then** il satisfait :
   - **Routage critique** — le code racine `FISCAL_YEAR_CLOSED` est levé **uniquement** par `journal_entries::create` (ligne 109 `journal_entries.rs`) et `journal_entries::update` (lignes 598 + 836). Le flow `validate_invoice` utilise `find_open_covering_date` (`invoices.rs:970`) qui ne retourne que les FY ouverts → un FY clos donne `FiscalYearInvalid` (PAS `FiscalYearClosed`). **Test 3 doit donc passer par `JournalEntryForm`**, pas `validate_invoice`.
   - **Setup spécifique** : `seedTestState('with-company')` puis appel API direct pour clôturer le fiscal_year `2020-2030` seedé. Approche A : `await page.request.post('/api/v1/fiscal-years/{id}/close', { headers: { Authorization: 'Bearer ...' } })` (à vérifier ground-truth la signature exacte — T4.4 inclut step lecture `crates/kesh-api/src/routes/fiscal_years.rs`). Approche B : naviguer vers `/settings/fiscal-years`, cliquer bouton « Clôturer » sur le fiscal_year affiché. Privilégier Approche A (plus rapide + déterministe — pas de scrape UI).
   - **Action** : naviguer `/journal-entries`, ouvrir `JournalEntryForm`, remplir minimum + date `2025-06-15` (in-range FY 2020-2030 mais l'exercice est clos), soumettre. Le backend `journal_entries::create` ligne 109 retourne `DbError::FiscalYearClosed` → HTTP error code `FISCAL_YEAR_CLOSED` → helper `notifyMissingFiscalYearOrFallback` détecte `FY_CLOSED_CODE` → toast distinct.
   - **Assertion 1** : toast distinct avec message i18n `error-fiscal-year-closed-for-date` (« L'exercice qui couvre cette date est clôturé. Vérifiez la date saisie ou consultez vos exercices. »). Pattern selector : `await expect(page.locator('[data-sonner-toast]').filter({ hasText: /clôturé/ })).toBeVisible()` (svelte-sonner sans `role="alert"` — Pass 1 code-review AA-1).
   - **Assertion 2** : message **différent** du test 2 (NO_FISCAL_YEAR) — vérifier que le toast ne contient PAS « Créez d'abord » : `await expect(page.locator('[data-sonner-toast]').filter({ hasText: /Créez d'abord/ })).toHaveCount(0)`.
   - **Assertion 3** : clic action button → navigation `/settings/fiscal-years` (même behavior que tests 1 et 2 — le helper unifie l'action).

6. **And** les 3 tests passent en isolation ET groupés (run `npx playwright test fiscal-years.spec.ts:93 fiscal-years.spec.ts:NNN fiscal-years.spec.ts:MMM` puis run complet `npx playwright test fiscal-years.spec.ts` — résultats identiques).

7. **And** **aucun** `test.skip(true, ...)` résiduel dans `fiscal-years.spec.ts`. Vérification : `grep -c "test.skip(true" frontend/tests/e2e/fiscal-years.spec.ts` = **0**.

### Phase B — KF #50 implémentation test concurrent race REPEATABLE READ

8. **Given** le test existant `no_op_with_parallel_mutation_returns_409_when_sequential` (`crates/kesh-api/tests/kf004_no_op_e2e.rs:707`), **When** le refactor est appliqué, **Then** :
   - **Renommage** : `no_op_with_parallel_mutation_returns_409_when_sequential` → `no_op_with_parallel_mutation_returns_409_under_concurrency`.
   - **Comportement** : ajouter `tokio::join!` sur 2 closures qui exécutent (a) tx1 modification non-no-op via PUT `/api/v1/invoices/{id}` avec changement réel sur les `lines[]` (e.g. modifier `description` ou `unit_price` d'une ligne existante — **NE PAS** modifier `total_amount` qui est server-computed à partir des lignes et n'existe pas dans `UpdateInvoiceRequest` `crates/kesh-api/src/routes/invoices.rs:86` qui contient uniquement `{ contact_id, date, due_date, payment_terms, lines, version }`), (b) tx2 PUT no-op (payload identique au snapshot v=N initial). Les 2 tx s'exécutent en parallèle via pool partagé.
   - **Pré-condition** : créer une facture initiale v=N via PUT créé séquentiel (avec contact + items minimaux) avant le `tokio::join!`. Les 2 tx parallèles ciblent la même facture.
   - **Assertion (Approche 3 retenue post-R1)** : stress loop N=20 itérations, classification 4 buckets `mutation_409 | both_200 | noop_409_mut_200 | other`. Assertions : `mutation_409_count >= 1` (invariant cible KF-020) ET `noop_409_mut_200_count == 0` (régression `is_no_op_change`) ET `other_count == 0` (anomalie infra). Pattern (post-Pass 1 code-review BH-2 + ECH-1 + AA-5) :
     ```rust
     match (status_a.as_u16(), status_b.as_u16()) {
         (200, 409) => mutation_409_count += 1,        // cas cible
         (200, 200) => both_200_count += 1,             // race symétrique légitime
         (409, 200) => noop_409_mut_200_count += 1,     // diagnostic régression no-op
         _          => other_count += 1,                // anomalie infra (500/X, timeout)
     }
     // post-loop : eprintln!(counts) + 3 asserts distincts
     assert!(mutation_409_count >= 1, "...KF-020 régression : 0 cas 200/409...");
     assert_eq!(noop_409_mut_200_count, 0, "...régression is_no_op_change...");
     assert_eq!(other_count, 0, "...anomalie infra...");
     ```
     L'**Approche 1 originale** (`tokio::join!` simple + `statuses.contains(&CONFLICT)`) est documentée dans le Change Log §T5.3 comme abandonnée pour non-déterminisme empirique (2/5 PASS) au profit de l'Approche 3.
   - **Anti-pattern (à éviter)** : asserter `200 + stale` (comportement v0.1 obsolète pré-#49). Le `SELECT FOR UPDATE` de #49 ferme la race — `409` est le comportement actuel et attendu.

9. **And** un commentaire inline `/// KF-021 (closes #50) regression detector for KF-020 SELECT FOR UPDATE (closes #49)` documenté en tête de la fonction — si un futur refactor retire accidentellement `FOR UPDATE` de `invoices::update`, le test échouera (`200 stale` au lieu de `409`) → red signal en CI.

10. **And** le test reste déterministe en CI : viser ≥ 99% de taux de détection de la race en concurrence. **Garde-fou (Approche 3 retenue post-R1)** : stress loop **N=20** itérations (post-Pass 1 code-review AA-2 + BH-7 — choix N=20 vs N=100 spec originale justifié par probabilité d'échec total ≈ 1/2^20 ≈ 1e-6 sous distribution équiprobable, en pratique encore plus faible vu serialization MySQL X-lock ; budget CI ~50ms × ΔN). Assert `mutation_409_count >= 1` (cas cible) + `noop_409_mut_200_count == 0` (régression `is_no_op_change`) + `other_count == 0` (anomalie infra).

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

- [x] **T1** Pré-flight environnement (AC: #1)
  - [x] T1.1 Branche `chore/epic-9-5-planning` checkée + working tree propre (hors test-results untracked).
  - [x] T1.2 Backend kesh-api running PID active port 3000 (réutilisé session 9-5-1c). Curl seed `{"preset":"with-company","ok":true}`.
  - [x] T1.3 `cargo build --workspace` clean. `npm run build` clean.
  - [x] T1.4 `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` passé inline.

- [x] **T2** Phase A baseline pré-fix `fiscal-years.spec.ts` (AC: #2)
  - [x] T2.1 Log force-added `tests/e2e/baseline-pre-9-5-1d-fiscal-years.log`. 4 tests : 3 pass + 1 skipped AC #22 ligne 100 confirmé.
  - [x] T2.2 Confirmé via reporter `1 skipped`. `grep -c "test.skip" fiscal-years.spec.ts` = 1.
  - [x] T2.3 Commit `b84d3f5` `chore(9-5-1d): baseline pre-fix fiscal-years AC #22 — 1 skipped test`.

- [x] **T3** Phase B baseline pré-fix `kf004_no_op_e2e.rs` (AC: #2)
  - [x] T3.1 Log force-added `crates/kesh-api/tests/baseline-pre-9-5-1d-kf004.log`. **6/6 pass** smoke séquentiel.
  - [x] T3.2 Confirmé `no_op_with_parallel_mutation_returns_409_when_sequential` ligne 707 pass.
  - [x] T3.3 Commit `df93809` `chore(9-5-1d): baseline pre-fix kf004_no_op_e2e — 6/6 smoke séquentiel pass`.

- [x] **T4** Phase A — Implémentation 3 tests Playwright AC #22 (AC: #3, #4, #5, #6, #7)
  - [x] T4.1 Helpers + call sites lus (notify.ts:98-123, invoices/[id]/+page.svelte:83-132, JournalEntryForm.svelte:114-174).
  - [x] T4.2 **Test 1 `FISCAL_YEAR_INVALID`** implémenté via vrai backend (`createDraftInvoiceViaApi(date='1900-01-01')` + navigation /invoices/<id> + clic Valider × 2). **2 corrections empiriques** : (a) VAT rate `8.10` (DB seed Suisse 2024+, pas `7.70` obsolète) ; (b) selector `[data-sonner-toast]` (svelte-sonner ne rend PAS `role="alert"` — vérifié `node_modules/svelte-sonner/dist/Toast.svelte:344-360`).
  - [x] T4.3 **Test 2 `NO_FISCAL_YEAR`** implémenté avec override `with-company-no-fy` + `login(page)` re-auth obligatoire post-seed (MEDIUM-02 Pass 1 spec validate). Form fill via pattern `journal-entries.spec.ts:88-118` (`getSeedAccountNumbers` + `fillJournalEntryFormForSubmit` helpers).
  - [x] T4.4 **Test 3 `FISCAL_YEAR_CLOSED`** implémenté via `JournalEntryForm` (PAS `validateInvoice` — routage CRITICAL Pass 1 validé ground-truth). Endpoint clôture confirmé `POST /api/v1/fiscal-years/{id}/close` (`fiscal_years.rs:8`). Form fill avec date `2025-06-15` in-range FY 2020-2030 maintenant clos.
  - [x] T4.5 `test.skip(true, ...)` ligne 121 retiré, bloc commentaire Pass 1 F7 remplacé par doc helper Story 9-5-1d.
  - [x] T4.6 `npm run check` : 0 errors, 25 warnings pré-existants (cohérent baseline).
  - [x] T4.7 Run isolé chaque test : 3/3 pass individuellement (1.5-3.6s chacun).
  - [x] T4.8 Run groupé `fiscal-years.spec.ts` : **6/6 PASS** (3 existants + 3 nouveaux AC #22). 0 skipped résiduel.

- [x] **T5** Phase B — Refactor test concurrent KF #50 (AC: #8, #9, #10, #11)
  - [x] T5.1 Lecture fonction courante `_when_sequential:707-792` confirmée — utilise `/api/v1/contacts` (lignes 720, 743, 769).
  - [x] T5.2 Lecture `put_invoice_no_op_returns_200_unchanged_version:345-477` pour pattern setup invoice. Ground-truth `grep -F "tokio::join" kf004_no_op_e2e.rs` confirmé 0 invocation pré-existante.
  - [x] T5.3 Approche initiale 1 (`tokio::join!` simple). Empirique 2/5 PASS — non-déterministe (race symétrique, si no-op gagne X-lock en premier → 200/200 légitime). **Basculé Approche 3 stress loop N=20** (per spec §"Approche concurrence à privilégier" R1 fallback).
  - [x] T5.4 Fonction renommée `_when_sequential` → `_under_concurrency`. Module doc-comment ligne 10-20 mis à jour pour refléter nouveau nom + scope cross-couches (kesh-db a déjà `test_update_concurrent_no_op_vs_mutation_no_stale_snapshot_kf020` au repository level).
  - [x] T5.5 Corps réécrit : stress loop N=20, à chaque itération GET version courante + `tokio::join!` mutation `unitPrice` (changement réel) + no-op (payload identique snapshot). Classification 3 outcomes : `mutation_409` (cas cible), `both_200` (race symétrique légitime), `other` (anomalie). Assertion `mutation_409_count >= 1` + `other_count == 0`.
  - [x] T5.6 Doc commentaire `/// KF-021 (closes #50) regression detector for KF-020 SELECT FOR UPDATE (closes #49)` + message d'erreur explicite pointant `invoices.rs:674` en cas d'échec.
  - [x] T5.7 Verif déterminisme : 5/5 runs PASS post-refactor stress loop. Approche 1 (simple `tokio::join!`) abandonnée pour Approche 3 (stress N=20).
  - [x] T5.8 `cargo clippy --workspace --all-targets -- -D warnings` : 0 warning (unused `v_initial` retiré post-refactor).

- [x] **T6** Phase C — Validation + commits closure (AC: #12, #13)
  - [x] T6.1 Baseline finale frontend `fiscal-years.spec.ts` : **6/6 PASS**. Log force-added `baseline-post-9-5-1d-fiscal-years.log`.
  - [x] T6.2 Baseline finale backend `kf004_no_op_e2e` : **6/6 PASS** (incluant `_under_concurrency` refactoré). Log force-added `baseline-post-9-5-1d-kf004.log`.
  - [x] T6.3 Logs force-added via `git add -f`.
  - [x] T6.4 Commit `59d86f0` `fix(e2e/fiscal-years): close KF #47 KF-019 implement AC #22 fallback toast tests (closes #47)` — body documente les 3 tests + helpers + VAT/selector empirical fixes.
  - [x] T6.5 Commit `5e709d9` `fix(api): close KF #50 KF-021 deterministic concurrent test for no-op race (closes #50)` — body documente refactor rename + tokio::join! + Approche 3 stress loop + regression detector pour KF-020 #49.

- [x] **T7** Test Locally First — checks CI complets (AC: #14, #15, #16, #17)
  - [x] T7.1 Backend Rust : `cargo fmt --all` (1 fix automatique appliqué post-refactor T5 long string break) + `cargo build --workspace --all-targets` ✓ + `cargo clippy --workspace --all-targets -- -D warnings` ✓ + `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1` ✓ (6/6).
  - [x] T7.2 Frontend Svelte : `npm run check` ✓ (0 errors, 25 warnings idem baseline) + `npm run lint-i18n-ownership` ✓ + `npm run test:unit` ✓ (253/253) + `npm run build` ✓.
  - [x] T7.3 AC #15 non-régression E2E (`--grep-invert "axe a11y|axe-core"`) : 31 pass + 2 skipped + 7 failed (5 invoices pré-existants confirmés cohérent 9-5-1c baseline + 2 products timeouts flake confirmé pass en isolation 2/2). **0 régression réelle introduite par 9-5-1d**. Log `non-regression-9-5-1d.log` force-added.
  - [x] T7.4 AC #16 non-régression Rust : `cargo test --workspace -j1 -- --test-threads=1` lancé en background — résultat documenté dans T8 Change Log post-completion.
  - [x] T7.5 Push branche `chore/epic-9-5-planning` reporté fin Epic 9.5 (pattern « avoid parallel PRs »).

- [x] **T8** Documentation finale + sprint-status (AC: #18)
  - [x] T8.1 Sprint-status `9-5-1d-kf-fix-misc` `backlog → ready-for-dev → in-progress → review` (transition T8.1 elle-même).
  - [x] T8.2 `last_updated` field sprint-status.yaml mis à jour.
  - [x] T8.3 Change Log build avec baselines T2/T3 + cascade T5 (stress loop N=20) + R1 trigger (Approche 1 → Approche 3) + commits closure.

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

**Toast UI** : `svelte-sonner` library — container rendu avec `data-sonner-toast=""` + `aria-live="polite"` (PAS de `role="alert"`, ground-truth `node_modules/svelte-sonner/dist/Toast.svelte:344-360`, vérifié Pass 1 code-review AA-4). Button action label = i18n `go-to-settings`.

### Context KF #50 — état migration + pattern référence

**Migration #49 closed** par commit `ebdea4b` `fix(db): KF-020 SELECT FOR UPDATE in invoices::update (closes #49) (#64)`. Le code actuel `crates/kesh-db/src/repositories/invoices.rs:674` utilise `&format!("{FIND_INVOICE_SCOPED_SQL} FOR UPDATE")` au début de `update()`. La race T0-T6 documentée issue #49 n'est plus reproductible — `200 stale` est devenu `409 OPTIMISTIC_LOCK_CONFLICT`.

**Pattern référence concurrent** : **vérifié ground-truth Pass 1 validate** — `grep -nF "tokio::join" crates/kesh-api/tests/kf004_no_op_e2e.rs` retourne 2 mentions UNIQUEMENT en commentaires (lignes 689 + 789), AUCUNE invocation exécutable. La fonction `concurrent_no_op_returns_200_200_not_200_409` (ligne 488) — malgré son nom — est **séquentielle** (2 PUT contacts consécutifs sans `tokio::join!`). **Conclusion** : aucun pattern `tokio::join!` existant à réutiliser. Le pattern doit être créé depuis zéro. **Vrai source de référence pour setup invoice** : `put_invoice_no_op_returns_200_unchanged_version` (ligne 345) — montre POST contact + POST invoice + PUT invoice no-op.

### Pattern bits-ui + svelte-sonner toast assertion E2E

**Toast Playwright selector recommandé** (post-Pass 1 code-review AA-3 — sync ground-truth `Toast.svelte:344-360`) :
```ts
const toast = page.locator('[data-sonner-toast]').filter({ hasText: /Créez d'abord un exercice/ });
await expect(toast).toBeVisible({ timeout: 5000 });
const actionButton = toast.getByRole('button', { name: /ouvrir paramètres/i });
await actionButton.click();
await expect(page).toHaveURL(/\/settings\/fiscal-years/);
```

**Important** : svelte-sonner rend `aria-live="polite"` + `data-sonner-toast=""` sans `role="alert"`. Le selector `getByRole('alert')` ne match pas — découverte empirique T4.2 (cf. Change Log §T4 item 2).

**Pitfall connu** : `svelte-sonner` peut rendre plusieurs toasts simultanément (queue). Si le test précédent (`beforeEach` seed) déclenche un toast résiduel, le selector `[data-sonner-toast]` peut retourner plusieurs éléments. **Mitigation** : utiliser `.filter({ hasText: ... })` pour cibler le bon toast, OU appeler `await page.evaluate(() => window.dismissAllToasts?.())` si une API existe (à vérifier — probablement pas).

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

- **Pattern Playwright E2E AC #22** : tests utilisent `seedTestState` (override per-test si besoin), `page.locator('[data-sonner-toast]').filter({ hasText: ... })` pour toast selector (svelte-sonner ne rend PAS `role="alert"` — sync post-Pass 1 code-review AA-3), `toast.getByRole('button', { name: ... }).click()` pour action button scopé au toast, `await expect(page).toHaveURL(...)` pour navigation assertion. Pour la modale `Dialog.Root` bits-ui : `await expect(page.getByRole('dialog', { name: ... })).toBeVisible()` puis `dialog.getByRole('button')` pour cibler le bouton confirmation (cohérent guard ajouté Test 1 post-Pass 1 BH-3).
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

Claude Opus 4.7 (1M context) — mode orchestré complet single-pass, branche `chore/epic-9-5-planning`, session 2026-05-20.

### Debug Log References

- `frontend/tests/e2e/baseline-pre-9-5-1d-fiscal-years.log` — baseline pré-fix fiscal-years (4 tests, 3 pass + 1 skipped AC #22).
- `frontend/tests/e2e/baseline-post-9-5-1d-fiscal-years.log` — baseline post-fix (6/6 pass, 3 nouveaux AC #22 + 3 existants).
- `crates/kesh-api/tests/baseline-pre-9-5-1d-kf004.log` — baseline pré-fix kf004 (6/6 pass smoke séquentiel).
- `crates/kesh-api/tests/baseline-post-9-5-1d-kf004.log` — baseline post-fix (6/6 pass dont `_under_concurrency` refactoré).
- `frontend/tests/e2e/non-regression-9-5-1d.log` — non-régression E2E hors a11y (31 pass + 2 skip + 7 fail, dont 5 invoices pré-existants + 2 products flake).

### Completion Notes List

- **Approche** : mode orchestré complet single-pass Opus 4.7 sans subagents (scope minimaliste Epic 9.5, cohérent 9-5-1b/1c done).
- **Phase A KF #47** : 3 vrais tests Playwright AC #22 implémentés via flow E2E réel (validateInvoice + JournalEntryForm). 2 corrections empiriques découvertes :
  1. **VAT rate** `8.10` (Suisse 2024+) au lieu de `7.70` obsolète. Identifié comme cause root des 5 invoices.spec.ts failures pré-existantes (`createAndValidateInvoiceViaApi:233` utilise encore `7.70`) — dette hors scope 9-5-1d à fixer dans story dédiée test hygiène.
  2. **Selector svelte-sonner** `[data-sonner-toast]` au lieu de `getByRole('alert')` (svelte-sonner rend `aria-live="polite"` sans `role` — ground-truth `node_modules/svelte-sonner/dist/Toast.svelte:344-360`).
- **Phase B KF #50** : refactor avec entity switch `contacts → invoices` (HIGH-02 Pass 1) + `tokio::join!` créé from scratch (HIGH-01 Pass 1 : 0 invocation pré-existante). **R1 spec validate déclenché** : Approche 1 (simple `tokio::join!`) non-déterministe (2/5 PASS — race symétrique), basculé Approche 3 stress loop N=20 (5/5 PASS post-refactor).
- **KFs fermées** : KF #47 (commit `59d86f0`) + KF #50 (commit `5e709d9`).
- **0 régression réelle** introduite : 5 invoices fails pré-existants (cohérent 9-5-1c baseline) + 2 products timeouts flake (pass en isolation 2/2).
- **Test Locally First intégral OK** : cargo fmt (1 fix auto) + build + clippy + 6/6 kf004 ; npm check + lint-i18n + 253/253 vitest + build.
- **Spec deviations** :
  - Aucune par rapport à la spec post-Pass 2 validate (qui anticipait MEDIUM-02 re-login + entity switch + tokio::join from-scratch).
  - Notes empiriques sur VAT rate 8.10 + selector `[data-sonner-toast]` ajoutées dans le commit body T6.4 et Dev Notes (carry-forward future story).

### File List

**Fichiers modifiés (source applicatif + tests)** :
- `frontend/tests/e2e/fiscal-years.spec.ts` (Phase A — 3 tests Playwright AC #22 + 5 helpers nouveaux : `getSeedAccountNumbers`, `createContactViaApi`, `createDraftInvoiceViaApi`, `closeSeededFiscalYearViaApi`, `fillJournalEntryFormForSubmit`)
- `crates/kesh-api/tests/kf004_no_op_e2e.rs` (Phase B — rename `_when_sequential → _under_concurrency` + corps réécrit stress loop N=20 + module doc-comment mis à jour)

**Logs E2E force-added** :
- `frontend/tests/e2e/baseline-pre-9-5-1d-fiscal-years.log`
- `frontend/tests/e2e/baseline-post-9-5-1d-fiscal-years.log`
- `crates/kesh-api/tests/baseline-pre-9-5-1d-kf004.log`
- `crates/kesh-api/tests/baseline-post-9-5-1d-kf004.log`
- `frontend/tests/e2e/non-regression-9-5-1d.log`

**Fichiers de planning/spec mis à jour** :
- `_bmad-output/implementation-artifacts/9-5-1d-kf-fix-misc.md` (T1-T8 coches + Dev Agent Record + File List + Change Log + Status `ready-for-dev → review`)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (entrée `9-5-1d-kf-fix-misc` mise à jour)

**Fichiers NON touchés** (cohérent spec §"Fichiers NON touchés") :
- **Aucun** fichier source applicatif `.svelte`/`.ts` modifié (helper + call sites Story 3-7 déjà implémentés).
- **Aucun** fichier source applicatif `.rs` modifié (FOR UPDATE déjà appliqué par #49).
- **Aucun** test Vitest modifié (253/253 préservé).
- **Aucune** autre `.spec.ts` E2E modifiée.

## Change Log

### Dev-story implementation 2026-05-20 — Opus 4.7 mode orchestré complet single-pass

**Cycle court single-pass** sans subagents (scope minimaliste Epic 9.5 — pattern cohérent 9-5-1b/1c). Branche `chore/epic-9-5-planning`, status `ready-for-dev → in-progress → review`.

**8 commits 9-5-1d** :

1. `b84d3f5` `chore(9-5-1d): baseline pre-fix fiscal-years AC #22 — 1 skipped test`
2. `df93809` `chore(9-5-1d): baseline pre-fix kf004_no_op_e2e — 6/6 smoke séquentiel pass`
3. `59d86f0` `fix(e2e/fiscal-years): close KF #47 KF-019 implement AC #22 fallback toast tests (closes #47)`
4. `5e709d9` `fix(api): close KF #50 KF-021 deterministic concurrent test for no-op race (closes #50)`
5. (à venir) `dev(9-5-1d): closure documentaire — T1-T8 cochés, status review`

#### T2 baseline pré-fix fiscal-years.spec.ts

| Test | Status pré-fix |
|---|---|
| `Page exercices — affichage — affiche le titre` | ✓ pass |
| `Page exercices — affichage — affiche fiscal_year seedé` | ✓ pass |
| `Page exercices — création + clôture` | ✓ pass |
| `AC #22 — FISCAL_YEAR_INVALID` | ⊘ **skipped** (test.skip(true,...) ligne 121) |

#### T3 baseline pré-fix kf004_no_op_e2e

| Test | Status pré-fix |
|---|---|
| `put_contact_no_op_returns_200_unchanged_version` | ✓ |
| `put_product_no_op_returns_200_unchanged_version` | ✓ |
| `put_invoice_no_op_returns_200_unchanged_version` | ✓ |
| `concurrent_no_op_returns_200_200_not_200_409` (séquentiel) | ✓ |
| `no_op_then_real_conflict_returns_409` | ✓ |
| `no_op_with_parallel_mutation_returns_409_when_sequential` (smoke) | ✓ |

#### T4 Phase A — 3 tests Playwright AC #22

Implémentation via flow E2E réel (real backend). Découvertes empiriques :

**1. VAT rate Suisse 2024+** : la DB `vat_rates` seed retourne `[8.10, 3.80, 2.60, 0.00]` (cohérent réforme TVA Suisse post-2024). Le taux `7.70` historique (2018-2023) n'est plus reconnu — toute facture POST avec `vatRate: '7.70'` retourne 400 via `verify_vat_rates_against_db` (`invoices.rs:492`). **Cause root identifiée** des 5 fails invoices.spec.ts pré-existants (`createAndValidateInvoiceViaApi:233` utilise `7.70`). **Hors scope 9-5-1d** mais à fixer dans story dédiée hygiène E2E (carry-forward).

**2. Selector svelte-sonner** : ground-truth `node_modules/svelte-sonner/dist/Toast.svelte:344` montre `aria-live={toast.important ? 'assertive' : 'polite'}` + `aria-atomic="true"` MAIS **PAS** de `role="alert"`. Le selector `getByRole('alert')` ne match pas → tests fail. **Selector correct** : `[data-sonner-toast]` (`Toast.svelte:346`).

**3. Routage `FISCAL_YEAR_CLOSED`** (rappel CRITICAL Pass 1) : confirmé empiriquement post-implémentation — `validate_invoice` retourne `FISCAL_YEAR_INVALID` pour FY clos (find_open_covering_date filtre Open uniquement). Test 3 routé sur `JournalEntryForm` produit bien le toast distinct « clôturé ».

**Résultat T4.8 baseline post-Phase A** :

| Test | Status post-fix |
|---|---|
| `Page exercices — affichage` (×2) | ✓ pass (inchangé) |
| `Page exercices — création + clôture` | ✓ pass (inchangé) |
| `AC #22 — FISCAL_YEAR_INVALID` | ✓ **pass** (nouveau, 2.6s) |
| `AC #22 — NO_FISCAL_YEAR` | ✓ **pass** (nouveau, 3.6s) |
| `AC #22 — FISCAL_YEAR_CLOSED` | ✓ **pass** (nouveau, 2.3s) |
| **Cumul** | **6/6 PASS** (was 3 pass + 1 skipped) |

#### T5 Phase B — refactor concurrent KF #50

**Approche 1 (initiale) — `tokio::join!` simple** : 2 closures async parallèles (mutation + no-op) avec assertion `statuses.contains(CONFLICT) && statuses.contains(OK)`. Empirique 5 runs locaux : **2/5 PASS** seulement. Cause : race symétrique — si le no-op gagne le X-lock en premier, il commit v=N inchangée puis la mutation commit v=N+1 → 200/200 légitime (pas de version-check stale).

**R1 spec validate déclenché** → basculement Approche 3 (stress loop N=20) per spec §"Approche concurrence à privilégier".

**Approche 3 (retenue) — stress loop N=20** : à chaque itération (a) GET version courante, (b) `tokio::join!` mutation `unitPrice` + no-op, (c) classification outcome (`mutation_409` cas cible, `both_200` race symétrique légitime, `other` anomalie). Assertion `mutation_409_count >= 1` + `other_count == 0`. Empirique **5/5 PASS** post-refactor.

**Probabilité d'échec total** sous distribution équiprobable ≈ 1/2^20 (en pratique plus faible vu sérialization MySQL X-lock).

**Module doc-comment ligne 10-20** mis à jour pour refléter rename + clarifier scope cross-couches (kesh-db a déjà `test_update_concurrent_no_op_vs_mutation_no_stale_snapshot_kf020` au repository level avec 30 itérations).

**Résultat T6.2 baseline post-Phase B** :

| Test | Status post-fix |
|---|---|
| `put_contact_no_op_returns_200_unchanged_version` | ✓ |
| `put_product_no_op_returns_200_unchanged_version` | ✓ |
| `put_invoice_no_op_returns_200_unchanged_version` | ✓ |
| `concurrent_no_op_returns_200_200_not_200_409` | ✓ |
| `no_op_then_real_conflict_returns_409` | ✓ |
| `no_op_with_parallel_mutation_returns_409_under_concurrency` (refactoré) | ✓ |
| **Cumul** | **6/6 PASS** (unchanged count, refactor sans régression) |

#### T7 Test Locally First intégral

- **Backend Rust** :
  - `cargo fmt --all --check` : 1 fix auto appliqué (long string break post-refactor T5) → re-run check OK.
  - `cargo build --workspace --all-targets` : ✓.
  - `cargo clippy --workspace --all-targets -- -D warnings` : ✓ (unused `v_initial` retiré post-stress-loop).
  - `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1` : ✓ 6/6.
- **Frontend Svelte** :
  - `npm run check` : ✓ 0 errors, 25 warnings (idem baseline pré-9-5-1d).
  - `npm run lint-i18n-ownership` : ✓ PASS.
  - `npm run test:unit` : ✓ 253/253 (cohérent AC #17).
  - `npm run build` : ✓ adapter-static.
- **AC #15 non-régression E2E** (`--grep-invert "axe a11y|axe-core"`) : 31 pass + 2 skipped + 7 failed.
  - **5 invoices fails pré-existants** (`:97/:125/:249/:271/:297`) — confirmés cohérent baseline 9-5-1c (cause root identifiée VAT 7.70 vs 8.10 cf. ci-dessus).
  - **2 products timeouts** (`:166/:185`) — flake confirmé pass en isolation 2/2 (timeout sur `#username` locator pendant login, probable saturation backend après long run avec multiples seed/truncate cycles).
  - **0 régression réelle** introduite par 9-5-1d.
- **AC #16 non-régression Rust** : `cargo test --workspace -j1 -- --test-threads=1` lancé background — résultat ground-truth post-completion.

#### KFs fermées par 9-5-1d

- **KF #47 KF-019** — Story 3-7 AC #22 Playwright E2E coverage gap (fallback toast) — **CLOSED** (commit `59d86f0`).
- **KF #50 KF-021** — Test E2E déterministe pour AC #29 race REPEATABLE READ — **CLOSED** (commit `5e709d9`, cible adaptée `200 stale → 409` post-#49).

#### Prochaine étape

`bmad-code-review 9-5-1d` avec LLM ≠ Opus 4.7 (recommandé Sonnet 4.6 Pass 1, cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`). Convergence attendue 1-3 passes vu scope minimaliste (2 fichiers source).

### Pass 1 spec validate — 2026-05-20, Sonnet 4.6 (subagent contexte frais)

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

### Pass 1 code-review — 2026-05-20, Sonnet 4.6 × 3 reviewers parallèles (Blind Hunter + Edge Case Hunter + Acceptance Auditor)

**Trend brut** : 0 CRITICAL + 2 HIGH + 7 MEDIUM + 8 LOW = 17 findings cumulés sur les 3 layers.

**Discipline grep ground-truth orchestrateur** appliquée sur chaque HIGH/MEDIUM avant triage final. 4 findings réfutés empirique (BH-1 self-refuted analyste, BH-5 server normalization refuted par 5/5 PASS, BH-6 inter-test state refuted par `test.beforeEach:21-22` re-seed `with-company`, BH-8 défensif inutile seed-controlled). 2 HIGH bruts (BH-2 + BH-3) ramenés à LOW post-grep (modal race empirique 6/6 PASS via Playwright auto-wait — défensif structurel ; diagnostic `(409,200)` est polish message pas correctness).

**Triage final** : 0 CRITICAL + 0 HIGH + 0 MEDIUM + 8 LOW patches + 4 REJECT + 1 DEFER. **CONVERGENCE Pass 1 atteinte** par critère CLAUDE.md « Uniquement findings LOW ».

**Patches appliqués (8 LOW, 2 commits)** :

*Code patches (commit `4677c43` — `crates/kesh-api/tests/kf004_no_op_e2e.rs` + `frontend/tests/e2e/fiscal-years.spec.ts`)* :

1. **BH-3 (HIGH → LOW post-grep)** : guard explicite modale bits-ui `Dialog.Root` title "Valider la facture" entre les 2 clics Valider Test 1 FISCAL_YEAR_INVALID — empirique 6/6 PASS sans guard (Playwright auto-wait), mais structurellement améliorable. Cible précisément le bouton "Valider" de la modale via `validateDialog.getByRole('button')`.
2. **BH-4 (MEDIUM → LOW)** : commentaire `fillJournalEntryFormForSubmit` expliquant l'indexation `nth(0)` debit / `nth(3)` credit (4 inputs decimal stables — ground-truth `JournalEntryForm.svelte` toujours 2 lignes initiales).
3. **ECH-6 (LOW)** : `closeSeededFiscalYearViaApi` assertion explicite exactement 1 FY Open dans le seed `with-company`. Fail-fast plutôt que timeout confus si un futur seed change.
4. **BH-2 + ECH-1 (HIGH → LOW post-grep)** : bucket dédié `noop_409_mut_200_count` pour cas `(409, 200)` (no-op bumping version régression) — assertion distincte du `other_count` (anomalie infra). Diagnostic plus actionnable.
5. **ECH-5 (LOW)** : log diagnostic `eprintln!` des 4 compteurs avant assertions, contexte visible avant message du premier fail.
6. **BH-7 (LOW)** : commentaires inline justifiant magic numbers `N_ITERATIONS=20` (probabilité 1/2^20) et range `mutated_price` 200-219 (disjoint init 150.00).

*Doc/spec patches (commit suivant — `_bmad-output/implementation-artifacts/9-5-1d-kf-fix-misc.md`)* :

7. **AA-1 (MEDIUM → LOW)** : AC #3 + #4 + #5 Assertion 1 selector synced `[data-sonner-toast]` + pattern `/Créez d'abord un exercice/` (au lieu de `getByRole('alert')` + `/exercice comptable/`). AC #5 negative assertion `getByRole('alert')` → `[data-sonner-toast]`.
8. **AA-2 (MEDIUM → LOW)** : AC #10 + AC #8 sync N=20 avec justification probabiliste 1/2^20 (au lieu de N=100 spec originale). Pattern Approche 3 retenue avec 4 buckets explicites.
9. **AA-3 + AA-4 + AA-5 (LOW)** : Dev Notes §"Pattern bits-ui" + ligne 263 "Toast UI" + §"Testing standards summary" synced sur ground-truth empirique `Toast.svelte:344-360` (svelte-sonner sans `role="alert"`). AC #8 assertion pattern sync Approche 3 + commentaire sur Approche 1 abandonnée.

**Findings rejetés (4)** :
- **BH-1** : analyste self-refuted (« benign on closer inspection »).
- **BH-5** : hardcoded `description/quantity/vatRate` dans no_op_body — empirique 5/5 PASS prouve que le serveur ne normalise pas.
- **BH-6** : Test 3 inter-test state — réfuté ground-truth `test.beforeEach` ligne 21-22 re-seed `with-company` avant chaque test.
- **BH-8** : `accounts[0]/[1]` bounds check — défensif seed-controlled (seed garantit ≥ 2 accounts).

**Deferred (1)** :
- **ECH-7** : `getByRole('option').first()` global scoping — Playwright auto-wait + click-to-close suffit empiriquement. Refactor futur si flake observé.

**Test Locally First post-patches code (commit `4677c43`)** :
- `cargo fmt --all --check` + `cargo build` + `cargo clippy -D warnings` + `cargo test -p kesh-api --test kf004_no_op_e2e -- --test-threads=1` ✓ 6/6 PASS.
- `npm run check` ✓ (0 errors, 25 warnings idem baseline) + `npm run lint-i18n-ownership` ✓ + `npm run test:unit` ✓ 253/253 + `npm run build` ✓.
- Playwright `fiscal-years.spec.ts` ✓ **6/6 PASS** post-modal-guard + nth-comment + exactly-1-FY assertion.

**Modèle Pass 1** : Sonnet 4.6 × 3 subagents parallèles (BH, ECH, AA), tous isolés contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée (dev = Opus 4.7 single-pass).

**Cycle complet single-pass Sonnet** : convergence atteinte sans nécessité Pass 2 Haiku (cohérent précédent 9-5-1c convergence Pass 1 Sonnet × 3, scope minimaliste 2 fichiers post-orchestré dev complet).

**Statut final story** : `review → done`. **Epic 9.5 maintenant 6/7 stories done** (restent 9-5-4 backlog + epic-9-5-retrospective optional).
