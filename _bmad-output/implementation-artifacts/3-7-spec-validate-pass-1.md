# Spec Validate Pass 1 — Story 3-7

**Date:** 2026-04-27
**Reviewer LLM:** orchestration Opus 4.7 + 3 reviewers Sonnet 4.6 (fresh context)
**Story file:** `3-7-gestion-exercices-comptables.md`
**Reviewer layers:** Disaster Prevention, AC Completeness, LLM Optimization
**Total findings raw:** 31
**Total findings post-dedup:** 25
**Verdict:** ⚠️ 2 CRITICAL + 9 HIGH + 11 MEDIUM + 3 LOW → Pass 2 requise

---

## Trend Pass 1 → ?

| Pass | LLM (reviewers) | CRITICAL | HIGH | MEDIUM | LOW | > LOW |
|------|-----------------|----------|------|--------|-----|-------|
| **1** | **Sonnet × 3** | **2** | **9** | **11** | **3** | **22** |

Pass 2 ciblera Haiku (cycle Sonnet → Haiku). Budget : 8 passes max.

---

## 🔴 CRITICAL (2) — Bloquant compilation

### C-1 — `AppError::IllegalStateTransition` existe déjà → Task T7 entier mort
- **Source:** Disaster F-01, LLM F-01 (consensus)
- **Vérification:** `crates/kesh-api/src/errors.rs:440-447` mappe déjà `DbError::IllegalStateTransition` vers HTTP 409 + code `ILLEGAL_STATE_TRANSITION` via le variant `AppError::Database`.
- **Impact:** Si dev agent exécute T7, il crée un doublon qui casse le `match` exhaustif compilo.
- **Fix:** supprimer T7 entier (T7.1, T7.2, T7.3). Mettre à jour AC #11 pour référencer le mapping existant via `AppError::Database(DbError::IllegalStateTransition)`.

### C-2 — `finalize()` n'extrait pas `Extension<CurrentUser>` → T3.1 ne compile pas
- **Source:** Disaster F-02, LLM F-03 (consensus)
- **Vérification:** Signature actuelle `routes/onboarding.rs::finalize` = `(State(state): State<AppState>)`. Le snippet T3.1 utilise `current_user.user_id` qui n'existe pas dans le scope.
- **Impact:** Compilation impossible.
- **Fix:** promouvoir T3.3 en T3.0 prérequis. Ajouter `Extension(current_user): Extension<CurrentUser>` à la signature de finalize. Snippet T3.1 valide ensuite.

---

## 🟠 HIGH (9) — À fixer avant merge

### H-1 — T1.5 oublie callsites de `fiscal_years::create`
- **Source:** Disaster F-04
- **Détail:** Refactor signature de `create` pour accepter `user_id` casse au minimum `crates/kesh-api/src/routes/journal_entries.rs:1096` (vérifier) + ~9 callsites dans `crates/kesh-db/tests/fiscal_years_repository.rs`.
- **Fix:** étendre T1.5 avec liste exhaustive issue de `grep -rn "fiscal_years::create" crates/`. Inclure tests dans le périmètre du refactor.

### H-2 — Méthode HTTP : PATCH non importé
- **Source:** Disaster F-03
- **Détail:** Le projet utilise `PUT` pour les updates (pattern `lib.rs`). Story prescrit `PATCH`. Pas de précédent dans le codebase.
- **Fix:** trancher `PUT` (cohérence codebase). Mettre à jour T2.5 et AC #6/#7 pour utiliser `PUT /api/v1/fiscal-years/{id}`.

### H-3 — `find_by_id` + check company_id = Anti-Pattern 4
- **Source:** Disaster F-05
- **Détail:** Le pattern fetch-then-check est explicitement listé comme Anti-Pattern dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md`. Le bon pattern est query avec `WHERE company_id = ? AND id = ?` directement.
- **Fix:** ajouter `fiscal_years::find_by_id_in_company(pool, company_id, id)` au repo. Utiliser dans T2.3, T2.5, T2.6. Retirer le check applicatif post-fetch.

### H-4 — `list_by_company().is_empty()` dans finalize = TOCTOU race
- **Source:** Disaster F-06
- **Détail:** T3.1 fait le check `is_empty()` hors tx FOR UPDATE → race classique avec finalize/seed_demo concurrents. Contredit la rigueur Pattern 5 établie en Story 7-1 P3-P6.
- **Fix:** utiliser `INSERT INTO fiscal_years (...) SELECT ... FROM dual WHERE NOT EXISTS (SELECT 1 FROM fiscal_years WHERE company_id = ?)` (insert-if-not-exists atomique) OU acquérir un lock `SELECT FOR UPDATE` dans la même tx que `insert_with_defaults_in_tx`. Préférer la première approche (single SQL roundtrip).

### H-5 — 2 UNIQUE constraints → mapping `UniqueConstraintViolation` non distinguable
- **Source:** Disaster F-07
- **Détail:** Schéma a `uq_fiscal_years_company_name` ET `uq_fiscal_years_company_start_date`. Le handler doit distinguer ces 2 cas pour mapper vers `error-fiscal-year-overlap` vs `error-fiscal-year-name-duplicate`.
- **Fix:** inspecter `MySqlDatabaseError::message` (contient le constraint name) ou pré-checker l'unicité côté handler avec `find_by_name(company_id, name)` + `find_by_start_date(company_id, start_date)` avant l'INSERT (avec FOR UPDATE pour éviter race). Préférer pré-check pour clarté.

### H-6 — Détection overlap réelle de plages dates manquante
- **Source:** AC F-02
- **Détail:** epics.md:854 exige « non-chevauchement avec exercices existants ». UNIQUE(company_id, start_date) ne couvre PAS le cas Jan-Dec 2027 + Jul 2027-Jun 2028 (start_date différents).
- **Fix:** ajouter check `EXISTS (SELECT 1 FROM fiscal_years WHERE company_id = ? AND start_date <= ? AND end_date >= ?)` (overlap d'intervalles fermés) dans `create()` avant INSERT, dans la même tx avec FOR UPDATE. Ajouter AC #4-bis et test.

### H-7 — Contradiction signature `update_name` (3 endroits)
- **Source:** AC F-04 + LLM F-02
- **Détail:** Section "Scope" liste `(pool, id, expected_version, user_id, new_name)`, T1.3 liste `(pool, user_id, id, new_name)` (sans version), Décisions disent "pas d'optimistic locking".
- **Fix:** harmoniser en `pub async fn update_name(pool: &MySqlPool, user_id: i64, id: i64, new_name: String) -> Result<FiscalYear, DbError>`. Supprimer toute mention de `expected_version`.

### H-8 — AC #20 manque test IDOR multi-tenant
- **Source:** AC F-07
- **Détail:** T2.3 et Dev Notes mentionnent multi-tenant scoping mais AC #20 ne liste pas le test « GET /fiscal-years/{id_autre_company} → 404 ».
- **Fix:** ajouter à AC #20 : `test_get_fiscal_year_other_company_returns_404` + `test_update_fiscal_year_other_company_returns_404` + `test_close_fiscal_year_other_company_returns_404`.

### H-9 — AC manquante : toast "Créez d'abord un exercice"
- **Source:** AC F-01
- **Détail:** epics.md:858 exige toast actionnable + lien si validate_invoice échoue par absence de fiscal_year. Auto-create T3.1 couvre Path B mais pas le cas Path A reset+re-validate ou le cas exercice supprimé manuellement.
- **Fix:** ajouter AC #22 — « Given une tentative de validate_invoice avec aucun fiscal_year ouvert couvrant la date, When error 400 NO_FISCAL_YEAR, Then toast actionnable côté frontend `Créez d'abord un exercice comptable dans Paramètres → Exercices` avec lien `/settings/fiscal-years` ». Tâche T5.9 frontend handler.

---

## 🟡 MEDIUM (11) — À fixer dans Pass 1 remediation

| # | Source | Issue | Fix |
|---|--------|-------|-----|
| **M-1** | Disaster F-08 + LLM F-08 | ORDER BY ASC→DESC peut casser tests existants | grep callers `list_by_company`, garder ASC en repo + sort DESC côté handler T2.2 |
| **M-2** | Disaster F-09 | AC #17 référence `closed_at` colonne inexistante | utiliser `updated_at` (auto-mis à jour par MariaDB ON UPDATE) |
| **M-3** | Disaster F-10 + AC F-08 | Rename post-Closed pas justifié | ajouter dans Décisions : « rename post-close autorisé pour corriger un libellé sans toucher aux dates ; aucune contrainte CO ne l'interdit » + test E2E rename Closed |
| **M-4** | Disaster F-11 | `routes/mod.rs` absent du Source tree | ajouter `crates/kesh-api/src/routes/mod.rs # MOD register` à la liste |
| **M-5** | LLM F-05 | Signature `create_in_tx` incomplète | écrire `pub async fn create_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, user_id: i64, new: NewFiscalYear) -> Result<FiscalYear, DbError>` + commenter le double-deref `&mut **tx` |
| **M-6** | LLM F-06 | Naming incohérent helper | uniformiser en `fiscal-years.helpers.ts` (préfixe pluriel = nom de feature dir) |
| **M-7** | LLM F-07 | `best_effort_rollback` recommandé alors que les handlers fiscal_years n'ouvrent pas de tx | retirer la mention dans Dev Notes (la tx est dans la fn repo, pas le handler) |
| **M-8** | LLM F-09 | AC #20 sans payloads ni setup user Lecteur | ajouter exemples concrets de payloads (CreateFiscalYearRequest JSON), référencer le helper `spawn_app` + comment créer un user Lecteur dans test |
| **M-9** | AC F-05 | "No new lock site" contredit T1.3 (FOR UPDATE in update_name) | T1.3 doit utiliser `SELECT before FOR UPDATE` puis update; ce nouveau site doit être ajouté à Pattern 5 doc T1.7 (nouveau) |
| **M-10** | AC F-06 | AC #14 test `demo_path_creates_fiscal_year` mais aucune tâche | ajouter T8.6 — test Playwright Path A vérifie auto-create existant |
| **M-11** | AC F-03 | Pas d'AC pour delete attempt | ajouter AC #23 — DELETE /fiscal-years/{id} retourne 405 Method Not Allowed (ou pas de route enregistrée du tout — vérifier comportement axum) |

---

## 🔵 LOW (3)

- **L-1** (LLM F-10): Vagueness dans T2.2 / T3.3 / T8.2 (« probablement », « vérifier que », « réutilise »). Durcir le wording avec actions concrètes.
- **L-2** (AC F-08): Renommage post-Close — ajouter test E2E explicite si M-3 retient l'autorisation.
- **L-3** (AC F-09): UX décision « opt-in vs transparent » d'epics.md:859 — la story tranche transparent (auto-create) ; ajouter une phrase explicite dans Décisions pour clore ce point.

---

## ❌ REJECT (1)

- **L-F04 (LLM-optim)** : "UNIQUE on `name` doesn't exist" — **FAUX POSITIF**. Schéma a bien `CONSTRAINT uq_fiscal_years_company_name UNIQUE (company_id, name)` (vérifié dans `crates/kesh-db/migrations/20260404000001_initial_schema.sql`). Le reviewer s'est trompé sur ce point précis.

---

## Critère d'arrêt CLAUDE.md

22 findings > LOW → **Pass 2 requise** après remédiation. Cycle LLM : Sonnet (Pass 1) → **Haiku** (Pass 2).

Budget restant : 7 passes (1/8 utilisée).

---

## Phase de remédiation Pass 1

### Phase 1 — CRITICAL (2 fixes)
1. C-1 : supprimer T7 entier ; AC #11 référence variant existant.
2. C-2 : promouvoir T3.3 en T3.0 ; ajouter `Extension<CurrentUser>` à signature finalize.

### Phase 2 — HIGH (9 fixes)
3-11. H-1 à H-9 selon table ci-dessus.

### Phase 3 — MEDIUM (11 fixes)
12-22. M-1 à M-11 selon table.

### Phase 4 — LOW (3)
23-25. Cosmétique.

### Phase 5 — Verify + commit + Pass 2
- Lecture critique de la spec post-remédiation.
- Commit + push.
- Lancer Pass 2 avec Haiku × 3 reviewers.

---

**Pass 1 Triage Complete: 2026-04-27**
**Next:** Apply 25 patches → save report → Pass 2 with Haiku
