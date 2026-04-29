# Code Review — Story 3.7 « Gestion des exercices comptables » — Pass 2

**Date** : 2026-04-28
**Reviewers** : Blind Hunter (Haiku), Edge Case Hunter (Haiku), Acceptance Auditor (Opus) — fenêtres fraîches, parallèles
**Source du diff** : commit `a43a568` (Story 3-7 impl + Pass 1 patches), 4865 lignes
**Mode** : `full` (avec spec + Pass 1 report)

## Synthèse

| Sévérité | Brut total | Après dédup/reject |
|----------|-----------:|-------------------:|
| CRITICAL | 0          | 0                  |
| HIGH     | 3          | 0                  |
| MEDIUM   | 9          | 1                  |
| LOW      | 18         | 8                  |
| **Total > LOW** | **12** | **1**          |

**Trend** : Pass 1 = 12 findings > LOW → Pass 2 = **1 finding > LOW** (-92%).

Findings rejetés (false positives ou non-issues) : 21 sur 30 bruts.

Per CLAUDE.md « Règle de remédiation » : 1 finding MEDIUM > LOW → soit Pass 3, soit reclassement explicite en dette technique documentée.

---

## CRITICAL

(aucun)

## HIGH

(aucun — Pass 2 confirme les patches Pass 1 sont solides sur les classes HIGH/CRITICAL)

## MEDIUM

### G1 — AC #22 : tests Playwright fallback toast restent `test.skip` (gap testing persistant)

- **Sources** : Acceptance Auditor D-4
- **Location** : `frontend/tests/e2e/fiscal-years.spec.ts:91-118`
- **Catégorie** : `defer` → reclasser en dette technique documentée (issue GitHub)
- **Détail** : Pass 1 F7 a remplacé les tests « annotations vides » par un `test.skip` honnête, mais l'AC #22 (fallback toast actionnable lorsque le backend retourne `FISCAL_YEAR_INVALID` / `NO_FISCAL_YEAR` / `FISCAL_YEAR_CLOSED`) n'a toujours **pas** de couverture E2E Playwright. Le wiring helper est partiellement validé statiquement par TypeScript (imports rigides), et les codes d'erreur backend sont couverts par les tests Rust E2E, mais le rendu réel du toast + la navigation vers `/settings/fiscal-years` ne sont jamais exercés dans un browser.
- **Impact** : Régression silencieuse possible si quelqu'un casse le wiring helper → JournalEntryForm/invoice page (suppression d'import, refactor du switch, etc.). Le compilateur ne détecte pas une suppression d'appel.
- **Recommandation** : créer une issue GitHub `[KF-NNN] AC #22 fiscal_year fallback toast — Playwright E2E coverage` avec `priority:medium` et `technical-debt`. Référencer dans Dev Notes section « Testing debt ». Idéalement résolu en réutilisant les fixtures de la Story 5.2 (validate_invoice end-to-end) qui peuvent forcer un état sans fiscal_year ouvert.

---

## LOW

### G2 — F1 catch `UniqueConstraintViolation` non narrow (spec future-proofing)

- **Sources** : Blind Hunter HIGH-1 (rétrogradé) + Edge Case Hunter #1
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:198-205` (post-Pass 1)
- **Catégorie** : patch optionnel (cosmétique + observabilité)
- **Détail** : Le catch dans `create_if_absent_in_tx` intercepte **toute** `UniqueConstraintViolation` et retourne `Ok(None)`. Aujourd'hui le schema fiscal_years n'a que 2 UNIQUE constraints (`uq_fiscal_years_company_name`, `uq_fiscal_years_company_start_date`), tous deux sur `(company_id, X)` — donc l'idempotence est sémantiquement correcte. Mais si quelqu'un ajoute une 3e UNIQUE constraint (ex. sur un futur champ `external_id`), elle serait silencieusement traitée comme idempotente.
- **Patch proposé** : ajouter un `tracing::warn!` quand le catch fire pour observabilité. Optionnellement narrow sur les noms de constraint connues. Petit, low-risk.

### G3 — Spec drift : signatures `update_name` / `close` enrichies par F2 non répercutées dans la spec

- **Sources** : Acceptance Auditor D-6
- **Location** : `_bmad-output/implementation-artifacts/3-7-gestion-exercices-comptables.md` (T1.3 ligne 199, T1.4 ligne 204)
- **Catégorie** : `bad_spec` (spec drift)
- **Détail** : T1.3 documente `update_name(pool, user_id, id, new_name)`, T1.4 documente `close(pool, user_id, id)`. Le patch F2 a ajouté `company_id` à ces deux signatures pour défense en profondeur multi-tenant. L'implémentation et les tests sont corrects, mais la spec n'a pas été mise à jour pour refléter le nouveau contrat.
- **Patch proposé** : mettre à jour T1.3, T1.4 et la section "Décisions de conception" pour documenter la signature finale + référence à Pass 1 F2.

### G4 — Spec drift : i18n keys list (T6.1) ne reflète pas les 2 nouvelles clés F3+F5

- **Sources** : Acceptance Auditor D-2 + D-5
- **Location** : `_bmad-output/implementation-artifacts/3-7-gestion-exercices-comptables.md` (T6.1 lignes 416-429)
- **Catégorie** : `bad_spec` (spec drift)
- **Détail** : T6.1 liste ~26 clés. Les patches Pass 1 ont ajouté :
  - `error-fiscal-year-name-too-long` (F3)
  - `error-fiscal-year-closed-for-date` (F5)
  Total réel : 28 clés × 4 locales = 112 entrées (vs ~104 documentées).
- **Patch proposé** : actualiser T6.1 + AC #19 + AC #22 dans la spec.

### G5 — Spec drift : AC #4 mentionne « toast rouge » mais F12 a retiré le toast pour validation backend

- **Sources** : Acceptance Auditor D-1
- **Location** : `_bmad-output/implementation-artifacts/3-7-gestion-exercices-comptables.md` (AC #4 ligne 104)
- **Catégorie** : `bad_spec` (spec drift)
- **Détail** : AC #4 dit « Le toast rouge s'affiche, la modale reste ouverte avec les valeurs saisies pour correction. ». Le patch F12 a explicitement choisi single error surface (inline-only) pour les erreurs de validation backend (modale reste ouverte avec erreur inline visible). Le toast subsiste pour les erreurs inattendues (réseau, etc.).
- **Patch proposé** : actualiser AC #4 (et AC #8 par symétrie) pour refléter la décision F12 — préciser « inline pour validation backend, toast pour erreurs inattendues ».

### G6 — Test E2E `update_name_other_company_returns_404` n'exerce pas le scoping repo F2

- **Sources** : Acceptance Auditor D-3
- **Location** : `crates/kesh-api/tests/fiscal_years_e2e.rs` (test homonyme)
- **Catégorie** : `defer` (informationnel)
- **Détail** : Ce test bloque dès le pre-check `find_by_id_in_company` du handler → 404 retourné AVANT que `update_name` ne soit appelé. Donc le scoping repo F2 n'est PAS exercé par ce test E2E. Heureusement, deux tests dédiés F2 existent (`update_name_repo_rejects_cross_tenant` côté E2E et repo).
- **Recommandation** : aucune action — la couverture F2 est correcte via les tests dédiés. Renommage cosmétique optionnel.

### G7 — `find_open_covering_date` non modifiée par F14 (mais déjà filtrée par status='Open')

- **Sources** : Acceptance Auditor D-9
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:397-414`
- **Catégorie** : `reject` (false positive — déjà filtrée)
- **Détail** : `find_open_covering_date` filtre déjà sur `status = 'Open'`, donc ajouter `ORDER BY status DESC` est redondant. Le patch F14 (ORDER BY déterministe) ne s'applique légitimement qu'à `find_covering_date`. Pas d'action.

### G8 — Helpers privés repo non documentés dans la spec

- **Sources** : Acceptance Auditor D-8
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:545-585` (helpers privés)
- **Catégorie** : `reject` (info only)
- **Détail** : `snapshot_json`, `build_audit_entry`, `insert_fiscal_year_in_tx`, `fetch_fiscal_year_in_tx` sont des helpers DRY non mentionnés dans la spec. Pas une déviation négative — bonne pratique.

### G9 — F14 ORDER BY collation fragility (status DESC)

- **Sources** : Edge Case Hunter #6
- **Location** : `crates/kesh-db/src/repositories/fiscal_years.rs:373` (`ORDER BY status DESC, start_date DESC`)
- **Catégorie** : `defer` (théorique)
- **Détail** : `'Closed' < 'Open'` lexicographiquement sous `utf8mb4_unicode_ci` (C=67, O=79), donc DESC met Open en premier. Logiquement correct mais dépend de l'ordre lexical. Une migration future de collation pourrait altérer ce comportement.
- **Patch alternatif** (non-appliqué) : utiliser `ORDER BY CASE WHEN status='Open' THEN 0 ELSE 1 END, start_date DESC` pour ordre explicite. Coût supplémentaire mineur, gain de clarté + future-proof. **Defer** v0.2 ou ne pas faire — la doc Pattern 5 peut noter l'invariant.

---

## Findings rejetés (false positives ou bruit)

| # | Source | Raison du reject |
|---|--------|------------------|
| R1 | Blind HIGH-2 (handler+repo redondant) | Defense en profondeur intentionnelle (cf. F2 patch) — pas un bug |
| R2 | Blind HIGH-3 (frontend ne trim pas) | **False positive** — le code Svelte dans `submitCreate`/`submitRename` fait bien `createForm.name.trim()` avant l'envoi (vérifié l. 99 et l. 137 de la page) |
| R3 | Blind MED-4 (i18n interpolation) | **False positive** — vérifié : `i18nMsg('fiscal-year-close-confirmation-body', ..., { name: closeTarget.name })` passe bien les args |
| R4 | Blind MED-5 (AC #22 tests vides) | Doublon de G1 (Auditor D-4) |
| R5 | Blind MED-6 (update handler ne trim pas) | **Repris par F8 patch** : trim centralisé dans `update_name` repo (l. 254) — comportement cohérent. Pas de bug |
| R6 | Blind MED-7 (F14 untested) | Test serait nécessaire mais le scénario (deux fiscal_years couvrant la même date) est interdit par UNIQUE constraints — défense en profondeur uniquement |
| R7 | Blind MED-8 (F4 comment confusing) | Le commentaire est volontairement explicite, lisibilité acceptable |
| R8 | Edge #2 (company_id ≤ 0) | **False positive** — `company_id` vient toujours du JWT (CurrentUser), validé à l'auth, garanti > 0 |
| R9 | Edge #3 + #7 (chars vs UTF-8 bytes) | **False positive** — `VARCHAR(50)` en MariaDB utf8mb4 = 50 **caractères** (pas bytes). Vérifié dans la migration `20260404000001_initial_schema.sql:50`. `chars().count()` est correct |
| R10 | Edge #4 (i18nMsg sans args) | **False positive** — vérifié : seul `fiscal-year-close-confirmation-body` a `{ $name }`, et le caller passe bien `args` |
| R11 | Edge #5 (closeSubmitting reste true) | **False positive** — `closeSubmitting = false` est dans le `finally` AVANT `await loadFiscalYears()`. Vérifié l. 195 |
| R12 | Edge #8 (currentYearDefaults timezone) | Doublon de F15 (Pass 1 defer v0.2) |
| R13 | Edge #9 (TOCTOU create) | **False positive** — `find_overlapping FOR UPDATE` + `find_by_name FOR UPDATE` dans la même tx empêchent le TOCTOU. Le UNIQUE constraint DB est filet de sécurité. Lié à F10 (deadlock retry v0.2) |
| R14 | Edge #10 (test setup FK) | **False positive** — `#[sqlx::test(migrator = ...)]` applique toutes les migrations, `audit_log` table prête |
| R15 | Blind LOW-9 (find_by_name no isolated test) | Couvert indirectement par les tests `test_create_rejects_duplicate_name` + `test_update_name_rejects_duplicate_name` |
| R16 | Blind LOW-10 (last_insert_id doc) | Polish — informationnel |
| R17 | Blind LOW-11 (i18n parity visual) | Vérifié : 4 locales contiennent les 28 mêmes clés `fiscal-year-*` + `error-fiscal-year-*` |

---

## Recommandation suite

**Trend** : 12 → 1 finding > LOW (-92%) après Pass 1+2.

**Décision proposée — STOP iteration après Pass 2** avec les actions suivantes :

1. **G1** (Playwright AC #22) → reclasser en **dette technique documentée** :
   - Créer une **GitHub issue** `[KF-NNN] AC #22 fiscal_year fallback toast — Playwright E2E coverage` (template `known_failure.yml`, labels `known-failure` + `technical-debt` + `priority:medium`).
   - Ajouter section « Testing debt » dans Dev Notes du story file 3-7 référençant l'issue.
   - Cohérent avec CLAUDE.md « Exception : MEDIUM+ reclassé en dette documentée … compte comme résolu pour cette itération ».

2. **G3 + G4 + G5** (spec drift) → mise à jour de la spec pour réconcilier :
   - T1.3 / T1.4 signatures avec `company_id`
   - T6.1 keys list avec les 2 nouvelles clés F3+F5
   - AC #4 + AC #8 wording « inline pour validation, toast pour fallback inattendu »

3. **G2** (F1 narrowing) → patch optionnel : ajouter `tracing::warn!` quand le catch fire pour observabilité future.

Si toutes ces actions sont prises, le ratio convergé est **0 finding > LOW non documenté** → critère d'arrêt CLAUDE.md atteint.

**Alternative** : Pass 3 avec un 4ᵉ LLM (Sonnet retour, ou Haiku/Opus différent) pour validation supplémentaire. Coût LLM élevé pour 1 seul MEDIUM connu et reclasable. Pas recommandé.

## Reviewers

| Layer | Modèle | Findings bruts | Findings retenus après dédup |
|-------|--------|----------------|------------------------------|
| Blind Hunter | Haiku | 11 | 1 (1 doublon Auditor + 9 reject) |
| Edge Case Hunter | Haiku | 10 | 1 (1 doublon Pass 1 defer + 8 reject) |
| Acceptance Auditor | Opus | 9 | 7 retenus (D-4 MED, D-1/D-2/D-3/D-5/D-6 LOW spec drift, D-9 reject, D-8 info) |

**Total brut** : 30. **Total après triage** : 9 (1 MEDIUM + 8 LOW). **Reject** : 17 + 4 doublons = 21.

---

**Pass 2 status** : 1 finding MEDIUM (D-4 / G1) reclassable en dette → critère d'arrêt CLAUDE.md atteint après reclassement. **Convergence Pass 1 → Pass 2 : 12 → 1 (-92%)**.
