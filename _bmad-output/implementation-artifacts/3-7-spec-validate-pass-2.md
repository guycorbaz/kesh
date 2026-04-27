# Spec Validate Pass 2 — Story 3-7

**Date:** 2026-04-27
**Reviewer LLM:** orchestration Opus 4.7 + 3 reviewers Haiku 4.5 (fresh context)
**Story file:** `3-7-gestion-exercices-comptables.md` (post Pass 1 commit `1ccc481`)
**Reviewer layers:** Disaster Prevention, AC Completeness, LLM Optimization
**Total findings raw:** 20
**Total findings post-dedup:** 17
**Verdict:** 1 HIGH + 12 MEDIUM + 4 LOW → Pass 3 requise

---

## Trend Pass 1 → 2

| Pass | LLM (reviewers) | CRITICAL | HIGH | MEDIUM | LOW | > LOW |
|------|-----------------|----------|------|--------|-----|-------|
| 1 | Sonnet × 3 | 2 | 9 | 11 | 3 | 22 |
| **2** | **Haiku × 3** | **0** | **1** | **12** | **4** | **13** |

Convergence amorcée (-9 findings > LOW vs Pass 1). Pass 3 ciblera Opus (cycle Sonnet → Haiku → **Opus**). Budget : 2/8 utilisées.

---

## 🟠 HIGH (1)

### HP2-H1 — SQL `INSERT ... WHERE NOT EXISTS` manque `FROM dual`
- **Source:** Disaster F-01
- **Localisation:** T1.2 (`create_if_absent_in_tx`) + T3.1 snippet onboarding finalize
- **Détail:** Le pattern atomique `INSERT INTO t (cols) SELECT ?, ?, ?, ? WHERE NOT EXISTS (...)` peut être ambigu en MariaDB selon la version. Le pattern idiomatique stable est `SELECT ?, ?, ?, ? FROM dual WHERE NOT EXISTS (...)`.
- **Fix:** ajouter `FROM dual` dans les snippets SQL de T1.2 et T3.1.

---

## 🟡 MEDIUM (12)

### HP2-M1 — Cross-ref T1.7 → T1.12
- **Source:** Disaster F-02
- **Détail:** Section "Lock ordering" cite « T1.7 (nouvelle tâche) » pour la doc Pattern 5, mais T1.7 est `find_by_name`. La doc est en T1.12.
- **Fix:** corriger le cross-ref.

### HP2-M2 — Audit context de `insert_with_defaults_in_tx` flou (story 5-2)
- **Source:** Disaster F-03
- **Détail:** T1.9 dit que les tests doivent passer `user_id=1`. Mais `insert_with_defaults_in_tx` (de Story 5.2) n'écrit pas d'audit log — il est utilisé par finalize. Cette story ne refactor pas cette fn.
- **Fix:** clarifier que **seules les fns NOUVELLES (T1.1-T1.4)** ont une signature audit-aware ; les fns Story 5.2 restent inchangées (et n'auditent pas — c'est le caller qui audite via le wrapper finalize).

### HP2-M3 — AC #22 prérequis non vérifiés (cluster)
- **Sources:** Disaster F-04 + LLM F-07 + AC F-04
- **Détail:** L'AC #22 (fallback toast) suppose 3 choses : (a) `frontend/src/lib/features/invoices/` contient le handler de validate_invoice, (b) le code d'erreur `NO_FISCAL_YEAR` existe dans Story 5-2, (c) le toast doit-il aussi déclencher pour `journal_entries::create` et `mark_as_paid` ?
- **Fix:** T5.9 commence par `grep -rn "NO_FISCAL_YEAR" crates/kesh-api/` pour confirmer le code existe ; étendre AC #22 explicitement à `journal_entries::create` + `mark_as_paid` (mêmes invariants FR24).

### HP2-M4 — `Invariant("overlap")` / `"name-duplicate"` — string matching fragile
- **Source:** LLM F-01
- **Détail:** T1.1 prescrit `DbError::Invariant(String)` avec `"overlap"` ou `"name-duplicate"` comme discriminateur. Le handler T2.4 doit faire un match string par contenu — fragile si un futur Invariant utilise un préfixe similaire.
- **Fix:** introduire des constantes `pub const FY_OVERLAP_KEY: &str = "fiscal_year:overlap"` (préfixe namespacé) + ajouter un fallback `_ => AppError::Internal(...)` pour les Invariants non-fiscalyear.

### HP2-M5 — Responsabilité audit log dans `create_if_absent_in_tx`
- **Sources:** LLM F-02 + AC F-05
- **Détail:** T1.2 dit que la fn « inclut audit_log si créé », mais la responsabilité (helper vs caller) n'est pas explicite.
- **Fix:** documenter explicitement dans T1.2 : « le helper fait l'audit_log INSERT en interne SI rows_affected == 1. Le caller (finalize) ne fait RIEN d'audit pour ce path. »

### HP2-M6 — Manque clause transient errors dans T2.4
- **Source:** LLM F-03
- **Détail:** T2.4 mappe 5 cas d'erreur DB. Mais transient errors (pool exhausted, connection drop) tombent dans le catch-all implicite. Pas de log structuré.
- **Fix:** ajouter `_ => { tracing::error!("unexpected db error in create_fiscal_year: {e}"); AppError::Internal(...) }`.

### HP2-M7 — `validateFiscalYearForm` logique underspecified
- **Source:** LLM F-04
- **Détail:** T4.3 mentionne la fn mais pas la logique exacte. Risque que le dev implémente un check incomplet.
- **Fix:** spécifier signature + corps :
  ```ts
  function validateFiscalYearForm(input: CreateFiscalYearRequest): string | null {
      if (!input.name.trim()) return 'error-fiscal-year-name-empty';
      const start = new Date(input.startDate);
      const end = new Date(input.endDate);
      if (isNaN(start.getTime()) || isNaN(end.getTime())) return 'error-fiscal-year-dates-invalid';
      if (end <= start) return 'error-fiscal-year-dates-invalid';
      return null;
  }
  ```

### HP2-M8 — `list_by_company` ASC + reverse handler = surcoût
- **Source:** LLM F-05
- **Détail:** Pass 1 M-1 a décidé de garder ASC dans le repo et reverse côté handler par crainte de breaking change. Mais l'analyse réelle des callers montre que les seuls users actuels sont `find_open_covering_date` (qui ignore l'ordre, simple SELECT WHERE date BETWEEN) et la nouvelle UI list. Le breaking change n'existe pas.
- **Fix:** revenir à la décision Pass 1 originale → changer ORDER BY à `DESC` directement dans le repo. Plus simple, plus efficace.

### HP2-M9 — AC #20 manque test injection `companyId`
- **Source:** AC F-01
- **Détail:** Aucun test ne vérifie qu'un user de companyId=1 ne peut pas créer un fiscal_year pour companyId=2 en injectant `companyId: 2` dans le JSON body.
- **Fix:** ajouter test E2E `create_with_injected_company_id_ignored` : POST `{companyId: 999, name: "X", ...}` avec user companyId=1 → 201 + body.companyId == 1 (le backend doit forcer la valeur du JWT, ignorer le payload).

### HP2-M10 — T8.6 / AC #14 ne vérifient pas l'absence audit_log pour seed
- **Source:** AC F-02
- **Détail:** Story 3.5 a établi que seed_demo ne doit PAS auditer. T1.8 (`create_for_seed`) doit garantir cette propriété. Aucun test ne le vérifie.
- **Fix:** ajouter dans T8.6 + T1.11 : `assert audit_log::find_by_action_and_entity('fiscal_year.created', fy.id).is_empty() == true` après seed_demo.

### HP2-M11 — T3.2 ne vérifie pas l'audit log Path B finalize
- **Source:** AC F-07
- **Détail:** Path B finalize crée le fiscal_year via `create_if_absent_in_tx` qui doit auditer. T3.2 a un test « creates fiscal_year » mais pas d'assertion audit.
- **Fix:** ajouter assertion dans `path_b_finalize_creates_fiscal_year` : `audit_log::find_by_entity('fiscal_year', new_fy.id).len() == 1` + `action == 'fiscal_year.created'` + `user_id == admin.id`.

### HP2-M12 — `create_for_seed` algorithme vague
- **Source:** LLM F-08
- **Détail:** T1.8 dit « variante non-auditée » sans spécifier si elle valide overlap/name (comme `create()`) ou skip toutes les validations.
- **Fix:** expliciter dans T1.8 : « Pré-checks overlap et name-duplicate **identiques à `create()`** (les contraintes DB doivent être respectées même en seed) ; la SEULE différence est l'absence d'audit_log. Pas de paramètre `user_id`. Tx interne. Caller : `seed_demo`. »

---

## 🔵 LOW (4)

- **HP2-L1** (LLM F-06): T3.0 marqué « PRÉREQUIS » mais pas explicite dans le graphe → ajouter section « Prerequisites » ou note `# REQUIRES: T3.0` en tête de T3.1.
- **HP2-L2** (LLM F-09): file naming partiellement incohérent → re-vérifier que tous les fichiers TS frontend sont préfixés `fiscal-years.*` (pluriel) et le module Rust `fiscal_years.rs` (pluriel).
- **HP2-L3** (AC F-03): rôle « Lecteur » dans AC #12 — vérifier `crates/kesh-db/src/entities/user.rs` pour la valeur enum exacte (peut être `Lecteur`, `Consultation`, `Reader`, etc.).
- **HP2-L4** (AC F-06): test edge case `close_empty_fiscal_year` (sans écritures liées) → optionnel, ajouter si T1.11 a budget.

---

## Critère d'arrêt CLAUDE.md

13 findings > LOW → **Pass 3 requise** après remédiation. Cycle LLM : Haiku (Pass 2) → **Opus** (Pass 3) avec fenêtre fraîche.

Budget restant : 6 passes (2/8 utilisées).

---

## Phase de remédiation Pass 2

### Phase 1 — HIGH (1)
1. HP2-H1 : ajouter `FROM dual` dans les snippets SQL T1.2 + T3.1.

### Phase 2 — MEDIUM (12)
2-13. HP2-M1 à HP2-M12 selon table.

### Phase 3 — LOW (4)
14-17. Optionnel mais à appliquer pour cohérence.

### Phase 4 — Verify + commit + Pass 3
- Lecture critique post-remédiation.
- Commit + push.
- Lancer Pass 3 avec Opus × 3 reviewers.

---

**Pass 2 Triage Complete: 2026-04-27**
**Next:** Apply 17 patches → Pass 3 with Opus
