# Code Review Pass 6 Triage Report — Story 7-1

**Date:** 2026-04-27
**Reviewer LLM:** Claude Sonnet 4.6 (cycle Opus → Sonnet → Haiku → Opus → Sonnet)
**Review Layers:** Blind Hunter, Edge Case Hunter, Acceptance Auditor (3 fresh-context Sonnet subagents)
**Review Mode:** FULL (spec + Pass 5 triage + diff `main..HEAD` Chunk 1 incl. Pass 5 patches)
**Chunk:** Code production + migrations + tests (16 fichiers, ~1019 lignes)
**Total Findings Pre-Dedup:** 30 (Blind 13 + Edge 13 + Auditor 4)
**Total Findings Post-Dedup:** 22
**Total Findings Post-Reject:** 18
**Overall Verdict:** ✅ Aucun CRITICAL ni HIGH après reconciliation. 3 MEDIUM + 9 LOW actionable. Pass 7 requise (3 MEDIUM > LOW).

---

## Trend Pass 1 → 6

| Pass | LLM | CRITICAL | HIGH | MEDIUM | LOW | Total >LOW |
|------|-----|----------|------|--------|-----|-----------|
| 4 | Opus/Sonnet | 2 | 6 | 10 | 0 | 18 |
| 5 | Opus | 1 | 5 | 5 | 6 | 11 |
| **6** | **Sonnet** | **0** | **0** | **3** | **9** | **3** |

Trend numérique : 18 → 11 → 3. Convergence forte. Budget restant : 2 passes.

---

## Reconciliation des verdicts contradictoires

Blind Hunter a flaggé 3 HIGH (F-01, F-02, F-04) que l'Acceptance Auditor a marqués comme correctement fixés. Reconciliation après lecture du code et du contexte v0.1 single-user :

| Finding | Blind | Auditor | Décision |
|---------|-------|---------|----------|
| **F-01** reset() lock libéré avant reset_demo | HIGH | "résiduel documenté #43" | Sous **single-user v0.1** la race est quasi-inreachable. **#43 ne couvre que le deadlock-retry middleware** (cross-table reverse-order), PAS le lock-and-release ici. → **LOW patch P6-L1** : clarifier le commentaire pour ne pas oversell la protection |
| **F-02** get_or_init_state race avec FOR UPDATE | HIGH | non flaggé | `fetch_one` retourne `Err`, pas panic. Lock auto-released sur conn drop. Acceptable v0.1. → **DEFER** |
| **F-04** seed_demo lock-and-release vs companies::update | HIGH | "résiduel documenté #43" | Mêmes raisons que F-01. companies::update gère NotFound proprement. → **LOW patch P6-L1** |

---

## Pass 5 Remediation Verification (Auditor)

Tous les patches P1-P17 de Pass 5 sont confirmés appliqués correctement. ACs 1-5 : PASS.

| Pass 5 finding | Pass 6 status |
|----------------|---------------|
| P1 (Migration SIGNAL) | ✅ FIXED CORRECTLY |
| P2 (H2 Pattern 5 doc) | ✅ FIXED CORRECTLY |
| P3 (reset TOCTOU) | ✅ FIXED with documented residual (cf. F-01) |
| P4 (reset is_demo gate) | ✅ FIXED CORRECTLY |
| P5 (ORDER BY company lock) | ✅ FIXED CORRECTLY |
| P6 (seed_demo races) | ✅ PARTIALLY FIXED with documented residual (cf. F-04) |
| P7 (retry backoff) | ✅ FIXED CORRECTLY |
| P8 (4 tests) | ✅ FIXED (gap N2 sur positive path env var) |
| P9-P17 | ✅ FIXED CORRECTLY |

---

## Findings Pass 6 — Consolidés

### 🛠️ PATCH (12 actionable)

#### MEDIUM (3)

**P6-M1 — `KESH_PRODUCTION_RESET` non documenté + parse strict silencieux**
- **Source:** Acceptance Auditor N1, Edge Case
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:214` + absent de `.env.example`, `docker-compose.dev.yml`, `docs/`
- **Problème:** seul `"1"` est accepté. `"true"` / `"yes"` est silencieusement rejeté avec la même erreur qu'un vrai block production. Aucune doc accessible aux ops.
- **Fix:** ajouter à `.env.example` (commenté avec explication), `docker-compose.dev.yml`, accepter `"1" | "true" | "yes"` case-insensitive.

**P6-M2 — `insert_with_defaults` rollback failure écrase le code d'erreur**
- **Source:** Blind F-05, Edge e6
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:248-251` (pool variant)
- **Problème:** `tx.rollback().await.map_err(map_db_error)?` propage l'erreur de rollback à la place de `InactiveOrInvalidAccounts`. La retry loop dans seed_demo ne match plus → bypass du retry sur cas transient.
- **Fix:** `tx.rollback().await.ok()` (best-effort, comme pattern P12 dans onboarding.rs).

**P6-M3 — Frontend `onboardingState.finalize()` autorise double-submit sur erreur**
- **Source:** Blind F-09
- **Location:** `frontend/src/lib/features/onboarding/onboarding.svelte.ts:142-149`
- **Problème:** `_loading = false` dans `finally` réactive le bouton submit immédiatement après une erreur. Double-clic possible avant que le toast ne soit visible.
- **Fix:** exposer un état `lastError` qui désactive submit jusqu'à action utilisateur, ou garder `_loading` true sur erreur jusqu'à un dismiss explicite.

#### LOW (9)

**P6-L1 — Comments dans reset()/seed_demo oversellent la protection des locks**
- **Source:** Blind F-01, F-04 (downgraded HIGH→LOW)
- **Fix:** clarifier que les locks couvrent seulement le gate-check / count-validation, pas l'opération destructive. Référencer #43 comme path à la pleine sérialisation.

**P6-L2 — Migration step order: index avant FK**
- **Source:** Blind F-08
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql`
- **Problème:** ordering NOT NULL → INDEX → FK. Si la migration plante entre INDEX et FK, schéma reste avec NOT NULL+INDEX mais sans contrainte référentielle.
- **Fix:** revenir à NOT NULL → FK → INDEX (l'index est créé implicitement par la FK avec InnoDB de toute façon).

**P6-L3 — Comment stale dans seed_demo après refactor P6**
- **Source:** Auditor N3
- **Location:** `crates/kesh-seed/src/lib.rs:138`
- **Fix:** remplacer `"creates its own company"` par `"updates the singleton company; concurrent calls serialized via FOR UPDATE"`.

**P6-L4 — Dead null-check dans finalize() après fail-fast en _in_tx**
- **Source:** Auditor N4, Edge e7
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:591-598`
- **Fix:** retirer le check (insert_with_defaults_in_tx fail-fast empêche déjà le NULL).

**P6-L5 — Pas de test pour le positive path de `KESH_PRODUCTION_RESET`**
- **Source:** Auditor N2
- **Location:** `crates/kesh-api/tests/onboarding_e2e.rs`
- **Fix:** ajouter `reset_allows_demo_at_step_5_when_env_var_set` avec `std::env::set_var` + teardown.

**P6-L6 — Ajouter FOR UPDATE sur le SELECT post-UPDATE de finalize (F-03 promu)**
- **Source:** Blind F-03 (defer→patch promotion)
- **Location:** `crates/kesh-api/src/routes/onboarding.rs` post-UPDATE SELECT in finalize
- **Problème:** SELECT après UPDATE sans FOR UPDATE. Sous default REPEATABLE READ c'est safe, mais sous READ COMMITTED tuning production ça pourrait retourner un snapshot intermédiaire.
- **Fix:** ajouter `FOR UPDATE` au SELECT pour uniformité avec le contrat de locking annoncé.

**P6-L7 — Frontend check stepCompleted avant goto (F-10 promu)**
- **Source:** Blind F-10 (defer→patch promotion)
- **Location:** `frontend/src/routes/onboarding/+page.svelte:108-122`
- **Problème:** `goto('/')` appelé inconditionnellement après finalize() résolu. Si backend retourne stepCompleted != 8 (bug ou race), user redirigé silencieusement vers app principale avec onboarding incomplet.
- **Fix:** `if (onboardingState.stepCompleted === 8) goto('/'); else toast.error(...)`.

**P6-L8 — Nouveau variant `OnboardingResetForbidden` distinct (F-12 promu)**
- **Source:** Blind F-12 (defer→patch promotion)
- **Location:** `crates/kesh-api/src/errors.rs` + `routes/onboarding.rs`
- **Problème:** 3 cas dans reset() partagent `OnboardingStepAlreadyCompleted`. Pour le cas (2) (production past step 2) et (3) (env var manquant), le client reçoit "déjà complété" alors qu'en réalité c'est un refus par policy.
- **Fix:** ajouter `AppError::OnboardingResetForbidden` (403 Forbidden), router les blocs production/env-var vers ce code, garder StepAlreadyCompleted pour step >= 7.

**P6-L9 — Standardiser le wrapping d'erreur dans finalize (F-13 promu)**
- **Source:** Blind F-13 (defer→patch promotion)
- **Location:** `crates/kesh-api/src/routes/onboarding.rs` finalize
- **Problème:** `AppError::Database(map_db_error(e))` construit explicitement à un endroit, alors que `?` + `From<DbError>` est utilisé partout ailleurs. Inconsistance.
- **Fix:** uniformiser via `.map_err(map_db_error)?` partout.

---

### 📦 DEFER (10)

| Finding | Catégorie | Justification |
|---------|-----------|---------------|
| F-02 (pool race read+lock) | concurrency | Acceptable single-user v0.1. Tracker via #43 |
| F-06 (insert_with_defaults double-commit fragility) | code structure | Pas de bug; refactor v0.2 (M1 pré-existant) |
| F-07 (retry loop on permanent failure) | operational | 150ms surcoût acceptable; transient/permanent indistinguishable au type level |
| F-11 (settingsSeq sans `$state`) | clarity | Comportement correct |
| e1 (pool exhaustion entre 2 acquisitions) | overload | Acceptable cas dégradé |
| e10 (display:none + remount) | UX rare | Acceptable |
| e12 (back-button pendant async sequence) | UX | Idempotent backend gère |
| e13 (count_active_by_role multi-tenant drift) | théorique | D8 pré-existant |
| D1-D8 (Pass 5 deferred) | pré-existants | Inchangé |

### ❌ REJECT (4)

| Finding | Raison |
|---------|--------|
| e2 (KESH_PRODUCTION_RESET skip à step==2) | Intentionnel — step 2 = pré-branch A/B, reset OK |
| e3 (Invariant → 500) | Correct: `len() != 1` vraiment Internal, pas user-fixable |
| e8 (commit failure → partial state) | Idempotent retry géré |
| e11 (handleSkipBank non-ApiError) | Branché correctement |

---

## Classification Summary

| Catégorie | Count | Severity | Action |
|-----------|-------|----------|--------|
| **patch — CRITICAL** | 0 | — | — |
| **patch — HIGH** | 0 | — | — |
| **patch — MEDIUM** | 3 | P6-M1..M3 | Fix immédiat |
| **patch — LOW** | 9 | P6-L1..L9 | Cleanup + correctness uniformity |
| **defer** | 10 | — | Tracker via #43 ou pré-existants |
| **reject** | 4 | — | Bruit |

**Total actionable (`patch`): 12**

---

## Critère d'arrêt — non atteint

3 MEDIUM > LOW restants → Pass 7 requise (cycle: Sonnet → **Haiku**, fenêtre fraîche). Budget restant : 2 passes (6/8 utilisées).

Les 3 MEDIUMs sont des fixes ciblés de 30 min total. Ne pas les défer.

---

## Issues GitHub identifiées comme implémentées

Contrôle parallèle mené pendant Pass 6 (sur demande user) :

| Issue | Status après Pass 5 | Action |
|-------|---------------------|--------|
| **#17** retirer docker-publish-main du ci.yml | ✅ Implémenté (grep vide dans ci.yml) | À fermer |
| **#30** 32 i18n key-ownership violations | ✅ Implémenté (Story 6-3, lint script PASS) | À fermer |
| **#40** Restrict /reset endpoint post-completion | ✅ Implémenté par Pass 5 P3+P4 (+2 tests) | À fermer (note status code 400 vs 403 originally) |
| **#41** httpOnly token storage | 🟡 Partiellement (doc créé, impl backend pas faite) | Garder ouverte |
| **#31** Backend test failure post-migration | 🟡 Migration retouchée (Pass 5 P1), à retester | À retester localement |

---

## Remediation Path

### Phase 1: MEDIUM (P6-M1, M2, M3) — bloque Pass 7
1. P6-M1 : doc + parse helper (15 min)
2. P6-M2 : `.ok()` sur rollback (5 min)
3. P6-M3 : frontend double-submit guard (15 min)

### Phase 2: LOW (P6-L1..L9) — cleanup + correctness
4. P6-L1 : comments lock scope (10 min)
5. P6-L2 : migration step order (5 min)
6. P6-L3 : seed_demo comment (2 min)
7. P6-L4 : dead null-check (5 min)
8. P6-L5 : test env var positive path (15 min)
9. P6-L6 : FOR UPDATE post-UPDATE SELECT (5 min)
10. P6-L7 : frontend stepCompleted check (5 min)
11. P6-L8 : OnboardingResetForbidden variant (15 min)
12. P6-L9 : finalize error wrapping (5 min)

### Phase 3: Issues
13. Fermer #17, #30, #40 avec références aux commits/passes.

### Phase 4: Verify + commit + Pass 7
14. cargo check + cargo test
15. Commit Pass 6 patches
16. `bmad-code-review` Pass 7 avec **Haiku** (cycle Opus → Sonnet → **Haiku** → Opus → Sonnet → Haiku).

---

**Triage Complete: 2026-04-27**
**Review Readiness:** ⚠️ Remediation requise avant Pass 7 (3 MEDIUM + 9 LOW)
