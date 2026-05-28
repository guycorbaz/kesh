# Story v011.2: Fix catch-22 onboarding fresh-install (Issue #120)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **nouvel utilisateur qui installe Kesh v0.1.x sur un fresh deploy (DB vide)**,
I want que `docker compose up` me permette de **me loguer avec l'admin du `.env` puis de compléter l'onboarding** sans connaître de workaround SQL,
so that je puisse créer ma company et commencer à utiliser Kesh en < 15 min (FR1), sans le **catch-22 bloquant** où l'admin n'est jamais créé tant qu'aucune company n'existe et où la company ne peut être créée que via un wizard exigeant une authentification.

## Scope

**Severity : catégorie A bloquante** (Issue #120). Sans ce fix, **aucun** nouvel utilisateur ne peut compléter l'install v0.1.0.

**Approche retenue : Option A complète** (décision Guy 2026-05-28, epic décision H2) — bootstrap crée simultanément une **company stub** (`is_stub = TRUE`) **ET l'admin** du `.env` quand la DB est vide, plus un marqueur `is_stub` explicite exposé au frontend pour un nudge de renommage.

**Dans le scope :**
- Migration `companies.is_stub BOOLEAN NOT NULL DEFAULT FALSE` (non-breaking).
- `Company` entity + `NewCompany`/repository : champ `is_stub` propagé sur **tous** les SELECT/INSERT (sinon sqlx casse au runtime — cf. §"Contexte technique").
- `bootstrap.rs` : sur DB vide (`company_count == 0 AND user_count == 0`), créer company stub `is_stub=TRUE` + admin attaché.
- `/api/v1/onboarding/state` expose `isStub` (lu sur la company courante).
- `is_stub` repassé à `FALSE` quand l'utilisateur renseigne ses vraies coordonnées (`set_coordinates`, step 5→6).
- Frontend : type `OnboardingState.isStub`, store, et **bannière de nudge** « Votre entreprise a un nom provisoire » tant que `isStub == true`.
- Tests unitaires bootstrap (fresh / idempotent / partial-state) + cycle `is_stub` + E2E Playwright fresh-install.
- Docs : CHANGELOG `Fixed`, `migrations-idempotence-audit.md`.

**Hors scope :**
- Recovery password production-grade (Issue #122, v0.2+).
- Break-glass admin reset (Story v011-3 séparée).
- Port défaut 80 (Story v011-4 séparée).
- Blocage **strict** de l'accès app tant que `is_stub` (cf. Q2 — l'app force déjà l'onboarding via `step_completed < 6`, le nudge `is_stub` est additif non-bloquant v0.1).

## Contexte technique de départ (lu 2026-05-28 — ground-truth)

> ⚠️ Le pseudo-code de l'epic (`INSERT INTO companies (name, language, is_stub, ...)`) ne reflète **pas** le schéma réel. Source de vérité ci-dessous.

### Le catch-22 (Issue #120)

1. `kesh-api` boot → `bootstrap::ensure_admin_user` (`crates/kesh-api/src/auth/bootstrap.rs:24`).
2. Sur DB vide : `company_count == 0` → **skip création admin** (`bootstrap.rs:32-37`, `return Ok(())`).
3. Frontend : `(app)/+layout.ts:10` ET `onboarding/+layout.ts:10` redirigent vers `/login` si `!authState.isAuthenticated`.
4. Login impossible (admin jamais créé) → **deadlock absolu**.

### Mécanisme onboarding existant (à réutiliser, NE PAS réinventer)

- L'app **force déjà** l'onboarding : `frontend/src/routes/(app)/+layout.ts:40-46` redirige vers `/onboarding` si `step < 3` (demo) / `step < 6` (prod).
- Le wizard **crée déjà** une company placeholder à l'étape 1 : `onboarding.rs:786` `ensure_company_with_language` → branche `None` INSERT `name="(en cours de configuration)"`, `address="-"`, `org_type=Independant`, `accounting_language=Fr` (`onboarding.rs:804-815`).
- La progression est suivie dans `onboarding_state.step_completed` (table singleton, entité `kesh_db::entities::OnboardingState`).
- Les steps remplissent progressivement la company : org_type (step 3→4 `onboarding.rs:287`), accounting_language (4→5 `onboarding.rs:321`), name+address (5→6 `set_coordinates` `onboarding.rs:377`).

→ **Conséquence design** : une fois l'admin créé par bootstrap, tout l'existant prend le relais. `is_stub` est un marqueur **additif** (nudge renommage), pas le mécanisme de routage (qui reste `step_completed`).

### Schéma `companies` réel (`crates/kesh-db/migrations/20260404000001_initial_schema.sql`)

Colonnes NOT NULL avec CHECK : `name` (CHECK length>0), `address` (CHECK length>0), `org_type` (CHECK in `Independant|Association|Pme`), `accounting_language` + `instance_language` (CHAR(2) CHECK in `FR|DE|IT|EN`). `ide_number` NULL ok. `version`/`created_at`/`updated_at` gérés DB.
→ Le stub DOIT satisfaire **toutes** ces contraintes (notamment `address` non-vide).

### `Config` (`crates/kesh-api/src/config.rs:123`)

Champs disponibles : `admin_username` (l.127), `admin_password` (l.128). **Pas de champ `lang`/`language`** → le stub utilise des langues par défaut (`FR`/`FR`) que le wizard remplacera (cohérent avec `ensure_company_with_language` qui hardcode `Language::Fr` à l.811).

### Sites à modifier si `is_stub` ajouté à l'entité `Company` (FromRow)

`sqlx::query_as::<_, Company>` mappe par nom de colonne → **tout** SELECT mappant `Company` sans `is_stub` casse au runtime (`no column found for name: is_stub`). Les 6 sites SELECT complets `Company` :

1. `crates/kesh-db/src/repositories/companies.rs:17` `FIND_BY_ID_SQL`
2. `crates/kesh-db/src/repositories/companies.rs:21` `LIST_SQL`
3. `crates/kesh-seed/src/lib.rs:89-90`
4. `crates/kesh-api/src/routes/onboarding.rs:630-631` (finalize, company lock)
5. `crates/kesh-api/src/routes/onboarding.rs:794-795` (`ensure_company_with_language`)
6. `crates/kesh-api/src/routes/onboarding.rs:843-844` `COMPANY_SELECT_FOR_UPDATE`

Sites INSERT company (le DEFAULT FALSE couvre les tests/repo ; explicite uniquement où stub) :
- `companies.rs:33` repository `create` → DEFAULT FALSE (non-stub) ✓ inchangé.
- `onboarding.rs:805` placeholder INSERT → set `is_stub=TRUE` (cohérence).
- `bootstrap.rs` nouvel INSERT stub → `is_stub=TRUE`.
- INSERTs de test (`test_fixtures.rs`, `*_e2e.rs`, `users_repository.rs`) → DEFAULT FALSE s'applique, inchangés.

### Frontend onboarding

- `frontend/src/lib/features/onboarding/onboarding.types.ts` : `OnboardingState { stepCompleted, isDemo, uiMode }`.
- `frontend/src/lib/features/onboarding/onboarding.svelte.ts` : store runes `_state`, `fetchState()` (l.40).
- `frontend/src/lib/features/onboarding/onboarding.api.ts` : appels API.

## Acceptance Criteria

### Migration & DB (AC #1-3)

- [ ] **AC #1** Migration `crates/kesh-db/migrations/<timestamp>_companies_is_stub.sql` : `ALTER TABLE companies ADD COLUMN is_stub BOOLEAN NOT NULL DEFAULT FALSE;`. Non-breaking (ADD COLUMN avec default → anciens binaires l'ignorent) → **pas** de bump `kesh_version_min_required` (CLAUDE.md migration breaking policy, epic H8).
- [ ] **AC #2** Ligne ajoutée à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx` ou justification — CLAUDE.md P5).
- [ ] **AC #3** Entité `kesh_db::entities::Company` gagne `pub is_stub: bool` ; les **6 sites SELECT** (cf. §Contexte) ajoutent `is_stub` à leur liste de colonnes.

### Bootstrap backend (AC #4-7)

- [ ] **AC #4** `bootstrap::ensure_admin_user` : si `company_count == 0 AND user_count == 0`, crée une company stub (`name="Setup en cours"`, `address="À configurer"`, `org_type=Independant`, `accounting_language=Fr`, `instance_language=Fr`, `is_stub=TRUE`) PUIS l'admin attaché à cette company. Valeurs placeholder satisfaisant les CHECK NOT NULL.
- [ ] **AC #5** Idempotence préservée : appels répétés ne dupliquent ni company ni admin ; si `user_count > 0` → skip (inchangé) ; si `company_count > 0 AND user_count == 0` (partial state, ex. company créée par wizard mais admin pas bootstrappé) → créer l'admin sur la company existante (comportement actuel préservé, ne PAS recréer de stub).
- [ ] **AC #6** Tolérance race (TOCTOU) préservée : `DbError::UniqueConstraintViolation` sur l'INSERT admin reste un succès silencieux ; le sanity-check post-insert (`bootstrap.rs:98-119`) reste non-fatal.
- [ ] **AC #7** Log `tracing::info!` explicite après création stub+admin (id company stub + username, rappel de compléter l'onboarding + CHANGER LE MOT DE PASSE).

### Exposition état onboarding (AC #8-10)

- [ ] **AC #8** `OnboardingResponse` (`onboarding.rs:58`) gagne `is_stub: bool` (serde camelCase → `isStub`). Valeur lue sur la company courante (première company ; `false` si aucune company).
- [ ] **AC #9** `get_state` (`onboarding.rs:75`) renseigne `isStub` depuis la company (sans casser la signature `From<OnboardingState>` — fetch company séparé dans le handler, pas dans le `From`).
- [ ] **AC #10** `set_coordinates` (step 5→6) repasse `is_stub=FALSE` dans le même UPDATE que name/address/ide_number (`onboarding.rs:928`). Après cet UPDATE, `isStub` vaut `false` dans les réponses suivantes.

### Frontend (AC #11-13)

- [ ] **AC #11** `OnboardingState` (types + store) gagne `isStub: boolean` ; `fetchState()` et les POST le propagent.
- [ ] **AC #12** Bannière/notice non-bloquante visible tant que `isStub == true` (texte i18n « Votre entreprise a un nom provisoire — complétez vos coordonnées »), affichée dans le flux onboarding. Disparaît dès `isStub == false`.
- [ ] **AC #13** Le routage existant (`(app)/+layout.ts` force `/onboarding` si `step < 6`) **n'est pas régressé** — `isStub` est additif, ne modifie pas les seuils de redirection (Q2 : non-bloquant v0.1).

### Tests (AC #14-16)

- [ ] **AC #14** Tests unitaires `bootstrap.rs` (sqlx::test) : (a) fresh install DB vide → 1 company `is_stub=TRUE` + 1 admin ; (b) idempotent (2 appels → pas de doublon) ; (c) partial state company-sans-user → admin créé sur company existante, pas de stub ; (d) `users` existants → skip. Le test existant `bootstrap_skips_silently_when_no_company_exists` (`bootstrap.rs:255`) est **mis à jour** (sur DB vide on crée désormais stub+admin, plus de skip).
- [ ] **AC #15** Test du cycle `is_stub` : après `set_coordinates`, la company a `is_stub=false`.
- [ ] **AC #16** Test E2E Playwright `fresh-install` : DB vide → login admin `.env` → wizard prod complet → company renommée → `isStub` false → app accessible. (Réutiliser les helpers E2E existants ; cf. `feedback_avoid_parallel_prs` pour le bundling PR.)

### Docs & Qualité (AC #17-18)

- [ ] **AC #17** `CHANGELOG.md` `[0.1.1]` section `Fixed` : entrée détaillée du catch-22 + `closes #120`.
- [ ] **AC #18** Série Test Locally First backend + frontend complète, 0 régression sur baselines (cargo test workspace, Vitest, E2E Playwright existants).

## Tasks / Subtasks

- [ ] **T1 — Migration + entité** (AC #1-3)
  - [ ] Créer `migrations/<ts>_companies_is_stub.sql` (`ADD COLUMN is_stub BOOLEAN NOT NULL DEFAULT FALSE`).
  - [ ] `docs/migrations-idempotence-audit.md` : nouvelle ligne.
  - [ ] `entities/company.rs` : `pub is_stub: bool` sur `Company` (PAS sur `NewCompany`/`CompanyUpdate` — géré via SQL explicite/DEFAULT).
  - [ ] Mettre à jour les 6 SELECT `Company` (companies.rs ×2, kesh-seed l.89, onboarding.rs l.630/794/843).
- [ ] **T2 — Bootstrap stub+admin** (AC #4-7)
  - [ ] Refactor `ensure_admin_user` : nouvelle branche `company_count == 0 && user_count == 0` → INSERT stub (`is_stub=TRUE`) + admin. Factoriser les valeurs placeholder (helper partagé avec `ensure_company_with_language` si DRY net, sinon constantes locales documentées).
  - [ ] Préserver branches existantes : `user_count > 0` skip ; `company_count > 0 && user_count == 0` → admin sur company existante ; race `UniqueConstraintViolation`.
- [ ] **T3 — Exposition isStub** (AC #8-10)
  - [ ] `OnboardingResponse` + `is_stub` field, handler `get_state` fetch company is_stub.
  - [ ] `set_coordinates` UPDATE inclut `is_stub = FALSE`.
- [ ] **T4 — Frontend** (AC #11-13)
  - [ ] `onboarding.types.ts` + store + api : `isStub`.
  - [ ] Bannière i18n non-bloquante (clés FR/DE/IT/EN — vérifier ownership i18n `npm run lint-i18n-ownership`).
- [ ] **T5 — Tests** (AC #14-16)
  - [ ] Tests unitaires bootstrap (4 cas) + maj `bootstrap_skips_silently_when_no_company_exists`.
  - [ ] Test cycle is_stub.
  - [ ] E2E Playwright fresh-install.
- [ ] **T6 — Docs + quality gate** (AC #17-18)
  - [ ] CHANGELOG Fixed + `closes #120`.
  - [ ] Série Test Locally First complète.

## Dev Notes

### Patterns à respecter (ground-truth)

- **Placeholder cohérent** : aligner les valeurs stub bootstrap avec `ensure_company_with_language` (`onboarding.rs:808-812`). Aujourd'hui le wizard utilise `"(en cours de configuration)"` / `"-"`. Le bootstrap peut utiliser `"Setup en cours"` / `"À configurer"` ; **décision spec** : harmoniser sur une seule paire de constantes partagées pour éviter la divergence (DRY). À trancher Pass 1.
- **Optimistic locking** : tous les UPDATE company passent par SELECT FOR UPDATE + `version = version + 1 WHERE version = ?` (cf. `onboarding.rs` helpers). Le bootstrap, lui, tourne au boot mono-process avant l'ouverture du serveur HTTP — pas de contention HTTP, mais garder la tolérance `UniqueConstraintViolation`.
- **`is_stub` n'est PAS le mécanisme de routage** — `onboarding_state.step_completed` l'est (layouts existants). `is_stub` est un nudge additif. Ne pas dupliquer la logique de complétion.

### Réconciliation bootstrap ↔ ensure_company_with_language

Sur fresh install Option A : bootstrap crée la company (id=1, `is_stub=TRUE`) au boot. Au 1er login → `(app)/+layout` fetch state (init step 0) → redirect `/onboarding` → step 0 `set_language` → `ensure_company_with_language` trouve la company (branche `Some`) → UPDATE `instance_language` (ne touche pas `is_stub`). Steps suivants remplissent org_type/accounting_language puis `set_coordinates` repasse `is_stub=FALSE`. **Pas de doublon de company.**

### Migration breaking policy (CLAUDE.md)

`ADD COLUMN is_stub BOOLEAN NOT NULL DEFAULT FALSE` = **non-breaking** (anciens binaires ignorent la colonne ; rows existantes prennent `FALSE`). → pas de bump `kesh_version_min_required` (epic H8). Audit idempotence `docs/migrations-idempotence-audit.md` obligatoire (P5).

### Règle de splitting préventif (CLAUDE.md)

Cette story touche ~5 modules de 1er niveau (kesh-db, kesh-api/auth, kesh-api/routes/onboarding, frontend/features/onboarding, frontend wizard) — **au seuil** (>5 déclenche le split obligatoire). Maintenue en story unique. **Garde-fou** : si `bmad-create-story validate` boucle > 4 passes sans converger, splitter en v011-2a (backend : migration + entity + bootstrap + exposition isStub + tests) / v011-2b (frontend bannière + E2E) avant dev-story.

### Questions ouvertes (à trancher en spec validate Pass 1 — cf. epic Q1/Q2)

- **Q-A (epic Q2)** : bannière `is_stub` strictement bloquante (interdit l'accès app) ou non-bloquante (nudge) ? **Proposition spec : non-bloquante v0.1** (l'app force déjà l'onboarding via `step_completed`, un blocage strict supplémentaire serait redondant et risquerait de piéger un user en demo path). À confirmer Guy.
- **Q-B** : valeurs placeholder harmonisées (bootstrap vs `ensure_company_with_language`) — une seule paire de constantes partagées ? **Proposition : oui, DRY.**
- **Q-C (epic Q1)** : `is_stub` rétro-compat — les déploiements v0.1.0 existants (seul Guy, qui sera reset) auront `is_stub=FALSE`. Aucun impact runtime (les vraies companies déjà nommées ne sont pas stub). Confirmé non-problématique.

## Change Log

### Create-story (2026-05-28)

Story créée par `bmad-create-story v011-2` (Opus 4.7). Analyse ground-truth exhaustive : Issue #120, `bootstrap.rs`, schéma `companies` réel (≠ pseudo-code epic), mécanisme onboarding existant (`ensure_company_with_language` + `step_completed` + layouts forçant le wizard), 6 sites SELECT `Company`, `config.rs` (pas de champ `lang`). **Scope tranché par Guy : Option A complète** (is_stub explicite) malgré la redondance partielle identifiée avec `step_completed`. Status `ready-for-dev`. Prochaine étape : `bmad-create-story validate v011-2` Pass 1 (Sonnet 4.6, cycle CLAUDE.md Review Iteration Rule).

## Dev Agent Record

### Agent Model Used

_(à remplir au dev-story)_

### Debug Log References

_(à remplir au dev-story)_

### Completion Notes List

_(à remplir au dev-story)_

### File List

_(à remplir au dev-story)_
