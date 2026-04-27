# Code Review Pass 5 Triage Report — Story 7-1

**Date:** 2026-04-27
**Reviewer LLM:** Claude Opus 4.7 (1M context) — cycle Opus → Sonnet → Haiku → Opus
**Review Layers:** Blind Hunter, Edge Case Hunter, Acceptance Auditor (3 fresh-context subagents in parallel)
**Review Mode:** FULL (spec + Pass 4 triage + diff `main..HEAD` Chunk 1)
**Chunk:** Code production + migrations + tests (14 fichiers, ~682 lignes)
**Total Findings Pre-Dedup:** 56 (Blind 20 + Edge 32 + Auditor 4)
**Total Findings Post-Dedup:** 35
**Total Findings Post-Reject:** 25
**Overall Verdict:** ⚠️ 1 CRITICAL régression + 5 HIGH non détectés en Pass 4 → **Pass 6 requise**

---

## Trend Pass 1 → 5

| Pass | LLM | CRITICAL | HIGH | MEDIUM | LOW | Total >LOW |
|------|-----|----------|------|--------|-----|-----------|
| 1 | Opus | (cf. pass-1) | | | | |
| 2 | Sonnet | (cf. pass-2-ready) | | | | |
| 3 | Haiku | (cf. pass-3-ready) | | | | |
| 4 | Opus/Sonnet | 2 | 6 | 10 | 0 | 18 |
| **5** | **Opus** | **1** | **5** | **5** | **6** | **11** |

Trend numérique : 18 > LOW (Pass 4) → 11 > LOW (Pass 5). Critère d'arrêt non atteint.

---

## Pass 4 Remediation Verification

| Finding | Pass 4 sev | Pass 5 status | Notes |
|---------|-----------|---------------|-------|
| C1 — INSERT IGNORE | CRITICAL | ✅ FIXED CORRECTLY | rows_affected check OK; pas de test ajouté |
| C2 — seed_demo uniqueness | CRITICAL | ✅ FIXED CORRECTLY | len()!=1 check OK; pas de test ajouté |
| H1 — Rollback handling | HIGH | ⚠️ FIXED + dead code (helper non câblé) → P12 |
| H2 — Deadlock 3 FOR UPDATE | HIGH | ❌ **NOT FIXED, NOT DOCUMENTED** → P2 |
| H3 — Account lookup timing | HIGH | ⚠️ FIXED mais retry sans sleep, probablement cosmétique → P7 |
| H4 — reset() step gating | HIGH | ⚠️ FIXED pour step≥7 mais brèche aux steps 3-6 → P4 |
| H5 — Migration backfill | HIGH | ❌ **REGRESSION: SIGNAL syntax invalide** → P1 (CRITICAL) |
| H6 — NULL check post-tx | HIGH | ✅ FIXED CORRECTLY | fetch_optional + Invariant OK |
| M2 — Frontend async cleanup | MEDIUM | ⚠️ FIXED via seq counter mais race window subsiste → P10 |
| M3 — Cryptic finalize msg | MEDIUM | ⚠️ Partiellement adressé (texte) mais user piégé → BS-1 |
| M4 — Idempotent stale snapshot | MEDIUM | ❌ Non adressé → P17 (LOW) |
| M5 — seed retry path | MEDIUM | ✅ Adressé via H3 |
| M6 — finalize/reset race | MEDIUM | ❌ Non adressé → P3 (escaladé HIGH) |
| M9 — Redundant step==8 logic | MEDIUM | ❌ Non adressé → defer |
| M10 — Frontend reset error UX | MEDIUM | ❌ Non adressé → P9 |
| M1, M7, M8 | MEDIUM | (defer per Pass 4 triage) |

---

## Findings Pass 5 — Consolidés

### 🔴 BAD SPEC (2)

**BS-1 — AC 3 fallback UI manquant : utilisateur piégé en step 7**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:170-180` (Validation error path)
- **Problème:** Si `accounts 1100/3000` manquent au finalize, user voit erreur "ajoutez ces comptes" mais (a) `step_completed=7` figé, (b) reset bloqué (production, step>2), (c) pas de route UI pour ajouter accounts pendant onboarding. État non-récupérable.
- **Source:** Blind#19
- **Action:** créer issue GitHub `KF-002-CR-001` (CR — fallback UI accounts pendant onboarding) OU implémenter auto-création accounts manquants dans `bulk_create_from_chart`.

**BS-2 — Pre-submit revalidation TOCTOU côté frontend**
- **Location:** `frontend/src/lib/components/invoices/InvoiceForm.svelte:244-270`
- **Problème:** Le pattern "F5: Revalidate settings" est TOCTOU déguisé. Correct fix = validation côté serveur dans `POST /invoices`.
- **Source:** Blind#11, Edge e30
- **Action:** créer issue GitHub `KF-002-CR-002` (CR — server-side account validation in POST /invoices).

---

### 🛠️ PATCH (17)

#### CRITICAL (1)

**P1 — Migration H5 SIGNAL syntax invalide (régression introduite par Pass 4)**
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql:19-24`
- **Problème:** `SELECT CASE WHEN ... THEN SIGNAL ...` au top-level. `SIGNAL` n'est valide que dans compound statement (BEGIN…END). La migration échouera au parse sur fresh DB.
- **Source:** Blind#4 (CRITICAL), Edge e10/e12, Auditor R-P5-001
- **Fix:** wrapper dans BEGIN…END via `CREATE PROCEDURE` temporaire + CALL + DROP PROCEDURE, OU forcer erreur via INSERT/SELECT impossible, OU se reposer sur la contrainte NOT NULL et accepter le message cryptique.
- **Blocking:** YES (toute fresh deploy cassée)

#### HIGH (5)

**P2 — H2 deadlock 3 FOR UPDATE sequentiels — non adressé ni documenté**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:458-505`
- **Problème:** Lock order onboarding_state → company → accounts. Pas de doc. Commit `704913d` mentionne "deferred to documentation pass" mais aucun doc trouvé.
- **Source:** Auditor B-H2, Edge e36
- **Fix:** documenter ordre global de verrouillage dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md` + commentaire en tête de finalize() + créer GitHub issue KF-002-H-002 pour stabilisation v0.2 (cf. règle Exception CLAUDE.md → doit être docs + owner + remediation story).

**P3 — reset() TOCTOU race vs finalize() concurrente**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:159-183`
- **Problème:** reset() lit non-locked puis appelle reset_demo. finalize() concurrente peut flipper step à 8 entre les deux → wipe production tenant.
- **Source:** Blind#1, Edge e2
- **Fix:** wrapper reset() dans tx, SELECT FOR UPDATE sur onboarding_state, re-check, puis reset_demo dans la même tx (ou commit du lock juste avant).

**P4 — reset() trusts is_demo flag aux steps 3-6**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:50-53`
- **Problème:** H4 ne bloque que step≥7. Tenant production avec is_demo corrompu reste resettable aux steps 3-6.
- **Source:** Blind#13
- **Fix:** ajouter check ENV var `KESH_ENVIRONMENT == "demo"` OU tightening du gate (block reset si step > 0 sauf demo explicite).

**P5 — finalize() lock company `LIMIT 1` sans `ORDER BY`**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:489-505`
- **Problème:** SELECT FROM companies LIMIT 1 FOR UPDATE non-déterministe. Mono-tenant safe mais multi-tenant futur → mauvaise company lockée.
- **Source:** Blind#14, Edge e7
- **Fix:** ajouter `ORDER BY id LIMIT 1`, OU mieux : utiliser `WHERE id = ?` avec company_id de la session onboarding.

**P6 — seed_demo() race conditions multiples**
- **Location:** `crates/kesh-seed/src/lib.rs:71-105 + 120-121 + 163-164`
- **Problème:** (a) `companies::list` sans FOR UPDATE → reset_demo concurrent peut DELETE; (b) `bulk_create_from_chart` commit own tx → orphaned accounts si seed_demo abort après; (c) `onboarding::update_step` optimistic version → step pas update si version change pendant seed.
- **Source:** Blind#15, Edge e20/e22/e26
- **Fix:** wrapper seed_demo dans une seule tx avec FOR UPDATE sur onboarding_state ET companies; `bulk_create_from_chart_in_tx`; commit final unique. Élimine aussi P7 (race H3).

#### MEDIUM (5)

**P7 — H3 retry loop sans backoff (probablement cosmétique)**
- **Location:** `crates/kesh-seed/src/lib.rs:139-161`
- **Problème:** 3 retries en busy-loop sans sleep. Si cause = REPEATABLE READ snapshot freshness, ne mitige pas. Si cause permanente (chart sans 1100/3000), 3 round-trips inutiles.
- **Source:** Blind#2, Edge e23, Auditor R-P5-004
- **Fix:** factoriser via P6 (preferred — élimine la race), ou ajouter `tokio::time::sleep(Duration::from_millis(50))` entre retries.

**P8 — Aucune nouvelle couverture de test pour fix paths**
- **Location:** `crates/kesh-db/tests/`, `crates/kesh-api/tests/`
- **Problème:** C1, C2, H4, H6 — zéro test ajouté. Le seul test modifié couvre la nouvelle sémantique de rejet, pas les remediation paths.
- **Source:** Auditor R-P5-006
- **Fix:** ajouter au minimum 4 tests Rust intégration (un par finding critique fixé).

**P9 — Frontend gestion d'erreur finalize() insuffisante**
- **Location:** `frontend/src/routes/onboarding/+page.svelte:108-122`, `frontend/src/lib/features/onboarding/onboarding.svelte.ts:142-148`
- **Problème:** Si setBankAccount succeed puis finalize() throw, user voit toast générique "Erreur lors de la sauvegarde du compte bancaire" et reste à /onboarding step 7.
- **Source:** Blind#12, Edge e33-e35, Auditor M10
- **Fix:** distinguer error codes (VALIDATION, ONBOARDING_STEP_ALREADY_COMPLETED) et afficher message actionnable; redirect vers /accounts si validation accounts échoue.

**P10 — InvoiceForm $effect loadingSettings race window**
- **Location:** `frontend/src/lib/components/invoices/InvoiceForm.svelte:127-150`
- **Problème:** loadingSettings=true posé dans IIFE async, pas synchrone. Fenêtre où loadingSettings reste false du run précédent → user peut cliquer Submit pendant re-fetch.
- **Source:** Blind#10, Edge e29/e32
- **Fix:** poser loadingSettings=true synchrone avant l'IIFE.

**P11 — kesh_seed errors mapped to 500 instead of 422**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:144`
- **Problème:** Toute erreur seed_demo → AppError::Internal (500). InactiveOrInvalidAccounts devrait être 422.
- **Source:** Edge e28
- **Fix:** match SeedError::Db(InactiveOrInvalidAccounts) → AppError::Validation, sinon Internal.

#### LOW (6)

**P12 — `best_effort_rollback` helper dead code**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:9-16, 28-33`
- **Source:** Blind#7, Edge e1, Auditor R-P5-002
- **Fix:** câbler tous les sites via `best_effort_rollback(tx).await`, OU supprimer le helper.

**P13 — Migration commentaires "Step 4" en double**
- **Location:** `crates/kesh-db/migrations/20260419000002_users_company_id.sql:26 et 34`
- **Source:** Blind#5, Edge e13, Auditor R-P5-003
- **Fix:** renuméroter Step 3 (validation) → Step 4 (NOT NULL) → Step 5 (index) → Step 6 (FK).

**P14 — Test name `test_insert_with_defaults_handles_missing_accounts` trompeur**
- **Location:** `crates/kesh-db/tests/company_invoice_settings_repository.rs:64-77`
- **Source:** Blind#16
- **Fix:** renommer en `test_insert_with_defaults_rejects_missing_accounts`.

**P15 — finalize() vérifie version optimiste à l'intérieur d'un FOR UPDATE**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:165-180`
- **Source:** Blind#18
- **Fix:** retirer `AND version = ?` ou retourner DbError::Invariant au lieu de OptimisticLockConflict.

**P16 — insert_with_defaults dead defense + risque FK obsolète**
- **Location:** `crates/kesh-db/src/repositories/company_invoice_settings.rs:270-285`
- **Source:** Blind#8
- **Fix:** retirer le NULL re-check (mort) ou le remplacer par JOIN validant accounts.active=TRUE.

**P17 — finalize() idempotent step==8 retourne snapshot locked-time**
- **Location:** `crates/kesh-api/src/routes/onboarding.rs:481-484`
- **Source:** Blind#6/#20, Edge e6, Auditor R-P5-005
- **Fix:** `tx.rollback().await.ok()` au lieu de tx.commit() sur ce path read-only; documenter le snapshot.

---

### 📦 DEFER (8)

- **D1 — M1:** Code duplication insert_with_defaults / _in_tx (~60 lignes) — pré-existant, deferred v0.2.
- **D2 — M7:** CASCADE DELETE sans audit trail — dette compliance, future story.
- **D3 — M8:** docker-compose curl healthcheck — couvert par Chunk 2, hors scope ici.
- **D4 — M9:** Logique step==8 early-exit redondante (M9 Pass 4 silencieusement skip).
- **D5:** Migration backfill `ORDER BY id LIMIT 1` (assume mono-tenant). À adresser quand multi-tenant company.
- **D6:** `step_completed in 0..=6` retourne OnboardingStepAlreadyCompleted (sémantique).
- **D7:** InactiveOrInvalidAccounts ne distingue pas receivable vs revenue.
- **D8:** count_active_by_role_in_company race théorique — pré-existant.

---

### ❌ REJECT (10)

- Edge e3 (reset rejette step≥7 même si is_demo=true) — comportement correct
- Edge e4 (finalize on demo lock held) — déjà géré inline
- Edge e8 (i32 version overflow après 2^31 updates) — théorique
- Edge e9 (tx commit failure semantics) — géré
- Edge e14 (UNIQUE constraint accounts) — schéma, hors scope
- Edge e17 (idempotency check sans rollback) — faux positif (pool variant n'a pas de tx; in_tx variant rollback dans caller, vérifié)
- Edge e18 (caller doit rollback) — vérifié OK
- Edge e21 (list.len()!=1 leaks side effects) — actuellement aucun side effect avant
- Edge e24/e25 (match arm syntax, off-by-one max_retries) — mineurs
- Edge e27 (FOREIGN_KEY_CHECKS=0 leak) — connection dédiée vérifiée
- Edge e31 (account ID stale après reset+re-seed) — phantom theoretical

---

## Classification Summary

| Catégorie | Count | Severity | Action |
|-----------|-------|----------|--------|
| **bad_spec** | 2 | — | Issues GitHub à créer |
| **patch — CRITICAL** | 1 | P1 | Bloque merge — fix immédiat |
| **patch — HIGH** | 5 | P2-P6 | À fixer avant merge sauf defer documenté |
| **patch — MEDIUM** | 5 | P7-P11 | À fixer avant merge si capacité |
| **patch — LOW** | 6 | P12-P17 | Cosmétique, à fixer au passage |
| **defer** | 8 | — | Pré-existants, tracker GitHub |
| **reject** | 10 | — | Bruit |

**Total actionable (`patch` + `bad_spec`): 19**

---

## Critère d'arrêt — non atteint

Selon CLAUDE.md, on relance Pass N+1 tant qu'il y a au moins un finding > LOW (CRITICAL/HIGH/MEDIUM). Pass 5 = **1 CRITICAL + 5 HIGH + 5 MEDIUM = 11 findings > LOW** → **Pass 6 nécessaire après remédiation**.

Budget restant : 8 - 5 = **3 passes**.

---

## Remediation Path

### Phase 1: BLOCKING (P1)
1. P1 — Migration H5 SIGNAL fix (15 min)

### Phase 2: HIGH (P2-P6)
2. P2 — Documenter H2 deadlock + GitHub issue (15 min)
3. P3 — reset() TOCTOU fix (30 min)
4. P4 — reset() blast radius reduction (20 min)
5. P5 — finalize ORDER BY (5 min)
6. P6 — seed_demo race fix — refactor (60-90 min, élimine aussi P7)

### Phase 3: MEDIUM (P7-P11)
7. P7 — H3 backoff (résolu par P6)
8. P8 — Tests C1/C2/H4/H6 (60 min)
9. P9 — Frontend finalize error handling (20 min)
10. P10 — loadingSettings sync (10 min)
11. P11 — Map InactiveOrInvalidAccounts → 422 (15 min)

### Phase 4: LOW (P12-P17)
12. P12-P17 — Cosmetic cleanup (30-60 min total)

### Phase 5: bad_spec (BS-1, BS-2)
13. BS-1, BS-2 — créer issues GitHub `KF-002-CR-001` et `KF-002-CR-002`

### Phase 6: Verify
14. cargo check + cargo test
15. Commit Pass 5 patches
16. Push (si demandé) puis Pass 6 avec Sonnet

---

**Triage Complete: 2026-04-27**
**Review Readiness:** ⚠️ Remediation required before merge (1 critical + 5 high + 5 medium)
