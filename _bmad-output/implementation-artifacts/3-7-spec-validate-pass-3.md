# Spec Validate Pass 3 — Story 3-7

**Date:** 2026-04-27
**Reviewer LLM:** orchestration Opus 4.7 + 3 reviewers Opus 4.7 (fresh context)
**Story file:** `3-7-gestion-exercices-comptables.md` (post Pass 2 commit `83e0d65`)
**Reviewer layers:** Disaster Prevention, AC Completeness, LLM Optimization
**Total findings raw:** 19
**Total findings post-dedup:** 17
**Verdict:** 4 HIGH + 8 MEDIUM + 5 LOW → Pass 4 requise

---

## Trend Pass 1 → 3

| Pass | LLM (reviewers) | CRITICAL | HIGH | MEDIUM | LOW | > LOW |
|------|-----------------|----------|------|--------|-----|-------|
| 1 | Sonnet × 3 | 2 | 9 | 11 | 3 | 22 |
| 2 | Haiku × 3 | 0 | 1 | 12 | 4 | 13 |
| **3** | **Opus × 3** | **0** | **4** | **8** | **5** | **12** |

Convergence ralentie (-8% Pass 2→3) car Opus a déterré 2 nouveaux bugs factuels (P3-H1, P3-H2) que Sonnet/Haiku avaient acceptés comme vrais. Cycle de remédiation Opus → Sonnet → Haiku → Opus → Sonnet pour Pass 4.

---

## Discussion technique avec Guy — Modèle métier validé

Guy a posé deux questions cruciales :

1. **Comment sait-on qu'une facture est dans le bon exercice comptable ?**
2. **Une facture émise fin d'année peut être payée sur l'exercice suivant.**

**Réponse vérifiée dans le code :**

- L'exercice est déterminé par la **date de l'écriture comptable (journal_entry)** créée à `validate_invoice`, pas la date de la facture elle-même. Le check `fiscal_years::find_open_covering_date(invoice.date)` est fait à la validation, puis le `journal_entries::create_in_tx` re-locke et re-vérifie le fy.
- `mark_as_paid` (Story 5-4) **ne crée AUCUN journal_entry** (vérifié `crates/kesh-db/src/repositories/invoices.rs:1069-1170`). Il fait juste `UPDATE invoices SET paid_at = ?` + audit log `invoice.paid`. Le journal_entry du paiement (Db Cash / Cr Receivable) sera créé par l'import bancaire futur (Story 8-4) basé sur la date de transaction CAMT.053.
- **Conséquence** : pas besoin de check fiscal_year à `mark_as_paid`. Une facture validée fin 2027 + payée en 2028 fonctionne sans contrainte (`paid_at` = juste un flag). Le Contexte de la story 3-7 ligne 19 est factuellement faux.

**Décision de scope (validée Guy)** : option α — retirer `mark_as_paid` du périmètre AC #22 + Contexte. Les seuls endpoints qui retournent une erreur fiscal_year en v0.1 sont `validate_invoice` (code `FISCAL_YEAR_INVALID`) et `journal_entries::create` (codes `NO_FISCAL_YEAR` ou `FISCAL_YEAR_CLOSED`).

---

## 🟠 HIGH (4)

### P3-H1 — AC #22 / Contexte / T5.9 référencent un code d'erreur inexistant + endpoint non concerné
- **Source:** Disaster Opus (vérification factuelle code)
- **Faits vérifiés** :
  - `validate_invoice` retourne `FISCAL_YEAR_INVALID` (errors.rs:472-478), **pas** `NO_FISCAL_YEAR`
  - `mark_as_paid` (invoices.rs:1069-1170) ne crée pas de journal_entry → **aucun** check fiscal_year
  - Seul `journal_entries::create` retourne `NO_FISCAL_YEAR` (errors.rs:~312) ou `FISCAL_YEAR_CLOSED`
- **Fix (option α)** : retirer `mark_as_paid` du Contexte (ligne 19) ; limiter AC #22 à 2 endpoints (validate_invoice + journal_entries::create) ; mapper les 3 codes possibles (`FISCAL_YEAR_INVALID` + `NO_FISCAL_YEAR` + `FISCAL_YEAR_CLOSED`) vers le même toast helper `notifyMissingFiscalYearOrFallback`.

### P3-H2 — `audit_log::find_by_action_and_entity_type` n'existe pas
- **Source:** Disaster Opus
- **Détail:** AC #14 (introduite Pass 2 HP2-M10) cite `audit_log::find_by_action_and_entity_type("fiscal_year.created", "fiscal_year").is_empty() == true`. Le repo `audit_log` n'expose que `insert_in_tx` et `find_by_entity`. Test ne compilera pas.
- **Fix:** utiliser `audit_log::find_by_entity('fiscal_year', fy_id).iter().any(|e| e.action == "fiscal_year.created") == false` (filtre en mémoire). Pas de nouvelle fn repo.

### P3-H3 — Inconsistance namespaced keys vs strings nues (4 endroits)
- **Source:** Disaster + LLM Opus (consensus)
- **Détail:** Pass 2 HP2-M4 a introduit `FY_OVERLAP_KEY = "fiscal_year:overlap"` etc. dans T1.1/T2.4, mais 4 endroits utilisent encore les bare strings : Décisions ligne 75-76, T1.3 ligne 194, T1.11 ligne 222, T2.5 lignes 269-270. Le match handler échouera silencieusement (`"name-empty"` ≠ `"fiscal_year:name-empty"`).
- **Fix:** sweep-replace les 4 endroits restants pour utiliser les valeurs `"fiscal_year:..."` (cohérent avec les constantes).

### P3-H4 — Dev Notes contredit Décisions de conception (lock ordering)
- **Source:** LLM HIGH + Disaster MEDIUM (escalé)
- **Détail:** Décisions ligne 87 dit « introduit DEUX nouveaux lock sites … T1.12: ajouter une entrée Pattern 5 ». Dev Notes ligne 425 dit « n'introduit pas de lock chains. Pas de modification à Pattern 5 nécessaire ». Direct contradiction.
- **Fix:** réécrire la phrase Dev Notes pour s'aligner avec Décisions + T1.12.

---

## 🟡 MEDIUM (8)

| # | Issue | Fix |
|---|-------|-----|
| **P3-M1** | Dev Notes ligne 433 référence encore « nouveau variant `AppError::IllegalStateTransition` » (leftover Pass 1 C-1 incomplet) | retirer la phrase, référencer le mapping existant errors.rs:440-447 |
| **P3-M2** | Source tree « # T1.1-T1.5 » mais le refactor est T1.1-T1.10 ; « # T1.6 tests » mais c'est T1.11 | sync les commentaires source-tree |
| **P3-M3** | T1.10 ORDER BY DESC casse silencieusement le test existant `fiscal_years_repository.rs:166-170` (test sur ASC) | mention dans T1.10 : « renommer/adapter le test existant pour vérifier DESC » |
| **P3-M4** | AC #12 / T2.8 utilisent `Lecteur` mais l'enum réel est `Consultation` (vérifié `chk_users_role`) | sweep-replace `Lecteur` → `Consultation` |
| **P3-M5** | Headings T1, T2, T5 omettent les ACs réellement implémentées (ex: T2 « (AC: #2-#11) » oublie #5, #12) | étendre les ranges dans les headings |
| **P3-M6** | AC #23 (DELETE 405) — pas de test E2E dans T2.8 | ajouter `test_delete_fiscal_year_returns_405` à AC #20 |
| **P3-M7** | AC #22 étendu à 3 endpoints Pass 2 mais T8 ne couvre que validate_invoice en E2E. Après α, étendre à journal_entries::create | ajouter scenario T8.7 pour journal_entries::create |
| **P3-M8** | AC #11 référence i18n key `error-fiscal-year-already-closed` mais aucun handler frontend ne map le code générique 409 `ILLEGAL_STATE_TRANSITION` vers cette clé spécifique | dans T5.x ajouter mapping basé sur l'URL endpoint (close → fiscal-year-specific key) |

---

## 🔵 LOW (5)

- **P3-L1** : Edge case « 2+ fiscal_years existants » lors du finalize idempotent — non testé (mais l'INSERT atomique le gère implicitement)
- **P3-L2** : T1 spans 14-16 ACs — heading très large (acceptable pour cette story massive)
- **P3-L3** : Audit log policy légèrement dupliquée Décisions vs Dev Notes
- **P3-L4** : T5.9 utilise dotted numbering 3 niveaux (T5.9.0, T5.9.1) — un peu lourd
- **P3-L5** : 66 traceability tags Pass 1/2 dans le spec — utiles pour audit, garder

---

## Critère d'arrêt CLAUDE.md

12 findings > LOW → **Pass 4 requise** après remédiation. Cycle LLM : Opus (Pass 3) → **Sonnet** (Pass 4) avec fenêtre fraîche.

Budget restant : 5 passes (3/8 utilisées).

---

## Phase de remédiation Pass 3

### Phase 1 — HIGH (4)
1. P3-H1 : appliquer α — corriger Contexte + AC #22 + T5.9 + T8.7
2. P3-H2 : remplacer fn audit_log inexistante
3. P3-H3 : sweep-replace namespaced keys
4. P3-H4 : réécrire phrase Dev Notes

### Phase 2 — MEDIUM (8)
5-12. P3-M1 à P3-M8

### Phase 3 — LOW (5)
13-17. Optionnels mais à appliquer pour cohérence

---

**Pass 3 Triage Complete: 2026-04-27**
**Next:** Apply 17 patches → Pass 4 with Sonnet
