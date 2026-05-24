---
epic: 9.5
title: "Technical Debt Closure"
version: v0.1
status: planning
sourceArtifact: _bmad-output/implementation-artifacts/epic-9-retro-2026-05-17.md (action items catégorie A)
relatedFRs: []
relatedDecisions:
  - "Politique « zero tech debt carry-forward » formalisée 2026-05-17 (memory `feedback_zero_tech_debt_carryforward`)"
  - "Pattern « Epic dédié cleanup » Epic 7 historique « Technical Debt Closure » (KF-001..007)"
crates:
  - kesh-api (modifs : `config.rs` fix env isolation tests, `routes/reports.rs` migrer `emit_report_export_audit` JSON snake_case)
  - frontend (modifs : a11y fixes, E2E test helpers refactor)
stories:
  - 9-5-1-kf-re-evaluation-closure
  - 9-5-2-code-consistency-fixes
  - 9-5-3-process-codification-claude-md
  - 9-5-4-swiss-co-research
---

# Epic 9.5 — Technical Debt Closure

## Vue d'ensemble

**Objectif :** Clore l'ensemble des dettes techniques catégorie A héritées des Epics 7/8/9 avant kickoff Epic 10 (TVA Suisse), conformément à la nouvelle politique projet « zero tech debt carry-forward » formalisée 2026-05-17 lors de la rétrospective Epic 9.

**Périmètre :** aucune feature nouvelle. 4 stories de cleanup focused : KF re-evaluation, code consistency, process codification, recherche réglementaire. ~13 items catégorie A enumerés en rétro Epic 9.

**Hors scope :** items catégorie B (limitations v0.1 documentées : Story 9-2a L13-L15, Story 9-2b L1-L18, Pass 1 deferred D1-D5, KF #76 multi-candidates UI 8-4) — gérés en parallèle via création GitHub Milestone `v0.2` + labelling (action hors story, voir §Action B parallèle).

**Provenance :** epic créé 2026-05-17 suite à clôture Epic 9 + décision Guy « pas de cumul de dette technique inter-epic ». Pattern référence : Epic 7 « Technical Debt Closure » historique (KF-001..007 fermées pré-Epic 8).

**Dépendances amont :** Epic 9 done (PR #94 mergée `35344c9`).

**Dépendances aval (Epic 10 TVA Suisse) :** Epic 10 **gelé** jusqu'à Epic 9.5 done. La JSON keys standardization (Story 9.5-2) impacte l'audit pattern `emit_*_audit` réutilisé Epic 10 pour `vat.calculated` / `vat.report_generated`.

---

## Catégorisation héritée retro Epic 9

| Catégorie | Définition | Action |
|---|---|---|
| **A — Vraie dette** | Bug latent, incohérence, action retro non-complétée, KF dormante ouverte | À fixer / clore dans Epic 9.5 |
| **B — Feature v0.2 légitime** | Limitation documentée avec scope explicite (`Lx` style) | Milestone v0.2 + labelling (parallèle, hors story) |
| **C — Decision design intentionnelle** | Pattern volontaire (e.g. tables EXCLUES, INNER JOIN FK garante) | Aucune action |

Total catégorie A : ~13 items répartis sur 4 stories.

---

## Stories

### Story 9.5-1 : KF re-evaluation + closure

**As a** mainteneur projet
**I want** vérifier que les 6 KFs GitHub Issues ouvertes sont encore actives, puis les fixer ou les fermer
**So that** le backlog KF reflète la réalité du code et qu'aucune dette ne traîne silencieusement

**KFs concernées (status 2026-05-17 — toutes OPEN sauf #70 closed) :**

| GitHub # | KF-NNN | Origine | Description courte |
|---|---|---|---|
| #47 | KF-019 | Story 3-7 | AC #22 Playwright E2E coverage gap (fallback toast) |
| #50 | KF-021 | Story 3-x | Test E2E déterministe pour AC #29 (race REPEATABLE READ no-op + mutation parallèle) |
| #54 | KF-022 | Story 7-x | E2E test helpers — cascade 401 sur API calls (createContact*, getAccountNumbers, etc.) |
| #55 | KF-023 | Story 7-x | E2E axe-core — violations a11y détectées sur 6 pages (login/contacts/homepage/invoices/products) |
| #57 | KF-025 | Story 7-x | E2E — failures state/timing/redirect dispersés (fiscal-years, mode-expert, onboarding, journal-entries) |
| #91 | KF-027 | Story 9-1 | E2E reports.spec.ts a11y — DropdownMenu.Trigger nested button violates wcag2a 4.1.2 |

**Critères d'acceptation :**

- **Given** chaque KF de la liste ci-dessus, **When** première étape obligatoire de la story, **Then** re-test du scénario (E2E run ou test ciblé) pour confirmer le bug encore reproductible.
- **Given** un KF dont le test passe maintenant (résolu par effet de bord), **When** verification, **Then** issue fermée avec commit message référençant la commit ou la story qui l'a résolu (`closes #N`).
- **Given** un KF encore actif, **When** fix appliqué, **Then** issue fermée avec test de régression ajouté.
- **And** chaque fix respecte le pattern `Test Locally First` (CLAUDE.md) avant push.
- **And** scoping multi-tenant `company_id` préservé sur chaque fix touchant des helpers API.
- **And** **aucun nouveau KF de sévérité > LOW** introduit par les fixes (vérifier via cycle review CLAUDE.md si patches non-triviaux).

**Effort estimé :** moyen — la re-évaluation initiale est rapide, mais 3-4 KFs E2E (#54, #55, #57) peuvent nécessiter des refactors de helpers de tests. KF #91 a11y DropdownMenu = recherche composant bits-ui spécifique.

**Path-dependency :** indépendant des autres stories Epic 9.5. Peut démarrer immédiatement après création epic.

**Décision split préventif appliquée 2026-05-18** : la règle CLAUDE.md §"Règle de splitting préventif" se déclenche (> 5 modules touchés : ~10-12 fichiers `.spec.ts` E2E + `tests/e2e/helpers/test-state.ts` + composant bits-ui DropdownMenu pour KF #91). Q3 ci-dessous anticipait précisément ce cas. Décomposition :

- **9.5-1a — Triage rapide** ✅ done 2026-05-18 : triage statique (grep code + git log + baseline logs diff) effectué. **Résultat : 0/6 KFs résolues par effet de bord depuis 2026-04-30**. Aucune sub-story annulée. Mapping résiduel finalisé ci-dessous.
- **9.5-1b — Fix E2E infrastructure** (KF #54 + #57) : fichiers scope précis : `frontend/tests/e2e/{invoices,invoices_echeancier,journal-entries,fiscal-years,mode-expert,onboarding,onboarding-path-b,homepage-settings,users}.spec.ts` + `tests/e2e/helpers/test-state.ts`. Root cause probable KF #54 : `page.request.*` calls sans Bearer header explicite (Story 6-5 localStorage shift). 2-3 passes attendues. **✅ review 2026-05-19** — KF #54 fixé 100% (0 occurrences 401), KFs #54 + #57 fermées via 2 commits closure dédiés. KF #57 split empirique post-fix en **KF-028 #96** (cascade-cleared post-KF #54 — UI navigation + backend 400 + UI refresh, 9 tests) + **KF-029 #97** (vrais résiduels — onboarding/mode-expert 30s timeouts + redirect + data-testid, 6 tests). Approche élargissement scope KF #57 → 2 KFs raffinées documentée pour rétrospective Epic 9.5.
- **9.5-1c — Fix a11y violations** (KF #55 + #91) : KF #55 audit a11y 5 pages (auth/contacts/homepage/invoices/products). KF #91 fix DropdownMenu.Trigger>Button nested-interactive dans `frontend/src/routes/(app)/+layout.svelte:136-144` — probable wrap custom retirant `<Button>` interne. Split possible 9-5-1c-quick + 9-5-1c-structural si > 100 violations résiduelles (R2 ci-dessous).
- **9.5-1d — Fix specific KFs** (KF #47 + #50) : KF #47 implémentation vrais tests Playwright AC#22 fallback toast (vs `test.skip(true, ...)` ligne 121 `fiscal-years.spec.ts`). KF #50 implémentation test déterministe race REPEATABLE READ via `tokio::join!` sur 2 pools distincts dans `kf004_no_op_e2e.rs`. Lien #49 KF-020 migration `SELECT FOR UPDATE` à arbitrer.

L'entrée `9-5-1-kf-re-evaluation-closure` dans `sprint-status.yaml` passe en status `split` (ne sera pas implémentée directement). Les ACs §Story 9.5-1 ci-dessus restent référencés via les sous-stories qui héritent du critère « 6 KFs fermées » comme condition de complétion globale.

**Note triage 9.5-1a** : aucun run E2E réel exécuté (mode static analysis uniquement — infra DB locale + browser stack coûteux à mettre en place pour confidence 100% sur des KFs déjà bien documentées). KFs #54/#55/#57 ont confidence ~90% encore actives. Un vrai run E2E avant implémentation 9-5-1b/c apporterait la confidence 100% mais n'est pas bloquant — les patches seront vérifiés au moment du dev par les tests E2E qu'ils corrigent.

---

### Story 9.5-2 : Code consistency fixes

**As a** mainteneur projet
**I want** standardiser les incohérences de code identifiées en rétro Epic 9 (régression locale `config::tests::*` + JSON keys `details_json` camelCase vs snake_case)
**So that** la base de code converge vers un pattern uniforme et que les développeurs futurs n'aient pas à choisir entre deux conventions

**Items à fixer :**

1. **`kesh-api::config::tests::*` 20/24 fail local** — collision `.env` `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true`. Régression locale traînée depuis Story 9-2a (documentée mais jamais fixée). Solution probable : isolation via `tempfile` ou env var stack scoped per-test (cf. Rust `serial_test` crate ou helper `with_env_vars`).

2. **`details_json` JSON keys incohérence** :
   - 9-2a `emit_report_export_audit` utilise camelCase (`reportType`, `format`, `fiscalYearId`, `periodStart`, `periodEnd`, `journalFilter`)
   - 9-2b `emit_global_export_audit` utilise snake_case (`company_id`, `byte_size`, `csv_count`, `fiscal_year_scope`, `duration_ms`) — décision Pass 1 review code-review 9-2b cohérent SQL JSON path `$.field_name`
   - **Décision projet** : snake_case standard pour `audit_log.details_json` JSON keys (cohérent SQL JSON paths future-proof, AC #23 spec 9-2b explicite).
   - **Action** : migrer `emit_report_export_audit` (9-2a) en snake_case + update assertions E2E `reports_export_e2e.rs` adjacentes + commit.

**Critères d'acceptation :**

- **Given** `cargo test -p kesh-api --lib config::tests::*`, **When** exécuté avec `.env` projet présent et `KESH_TEST_MODE=true` set, **Then** 24/24 tests passent (zero fail). Solution = isolation env explicite, pas dépendance ordre d'exécution.
- **Given** `emit_report_export_audit` après migration, **When** un export PDF/CSV est généré, **Then** `audit_log.details_json` contient les clés snake_case (`report_type`, `format`, `fiscal_year_id`, `period_start`, `period_end`, `journal_filter`).
- **Given** tests E2E `reports_export_e2e.rs::*audit*` après migration, **When** exécutés, **Then** 20/20 passent avec assertions snake_case mises à jour.
- **And** standardisation documentée : section §JSON keys convention dans CLAUDE.md ou `architecture.md` (snake_case standard pour `audit_log.details_json`, camelCase reste pour HTTP responses + frontend metadata.json — patterns existants préservés).
- **And** **0 régression** : 28/28 reports_e2e + 20/20 reports_export_e2e + 21/21 exports_global_e2e baselines préservées.

**Effort estimé :** faible. Fix `config::tests` est isolé. Migration `emit_report_export_audit` est ~5 keys × 1 fn + assertions tests adjacents.

**Path-dependency :** indépendant. Peut démarrer en parallèle de 9.5-1.

---

### Story 9.5-3 : Process codification CLAUDE.md

**As a** mainteneur projet
**I want** codifier dans CLAUDE.md les patterns de process découverts ou validés lors des Epics 7/8/9 mais pas encore documentés formellement
**So that** ces règles deviennent appliquées systématiquement et résistent au turnover développeurs

**Items à codifier :**

1. **« Haiku 4.5 grep ground-truth obligatoire »** — memory `feedback_haiku_review_diff_combined` actuellement persistée user-level. À promouvoir en section CLAUDE.md (règle projet) car validée 2× sur Epic 9 (2 hallucinations Pass 2 Haiku confirmées ground-truth). Placement suggéré : §"Review Iteration Rule" existante, sous-section "Haiku-specific guardrails".

2. **« AcceptedProposal batch pattern strict »** — Epic 8 retro action #6 non-codifiée. Pattern `FailedProposal per-proposal` (pas d'`AppError` global escalation) validé Epic 8 sur 8-4 + 8-5a-bis + 8-5b. Réutilisable Epic 11 (pain.001 paiements batch). Placement : nouvelle section §"Batch operations error handling" ou intégré dans architecture.md.

3. **« Zero tech debt carry-forward policy »** — décision Guy 2026-05-17 (cette rétro). Memory `feedback_zero_tech_debt_carryforward` à promouvoir en section CLAUDE.md formelle. Placement suggéré : nouvelle section §"Tech debt management" après §"Review Iteration Rule".

**Critères d'acceptation :**

- **Given** CLAUDE.md post-Story 9.5-3, **When** une nouvelle review Haiku est lancée, **Then** la règle "grep ground-truth avant flag CRITICAL/HIGH sur claims `await`/refs ligne précise" est citée explicitement et appliquée par l'orchestrateur.
- **Given** CLAUDE.md post-Story 9.5-3, **When** une opération batch (e.g. réconciliation accept multiple, future pain.001) est implémentée, **Then** le pattern `FailedProposal per-proposal` (pas d'`AppError` global escalation) est cité comme convention.
- **Given** CLAUDE.md post-Story 9.5-3, **When** une rétrospective d'epic est exécutée, **Then** la politique "zero tech debt carry-forward" est citée comme règle obligatoire de triage (catégories A/B/C).
- **And** chaque section ajoutée référence sa memory source pour traçabilité (e.g. `(cf. memory feedback_zero_tech_debt_carryforward)`).
- **And** les memories user-level concernées restent en place — la promotion vers CLAUDE.md project-level n'est pas une suppression.

**Effort estimé :** trivial. Pure documentation. 3 sections × ~20-40 lignes chacune.

**Path-dependency :** indépendant. **Recommandé de démarrer en premier** — débloque les autres stories en posant les règles process appliquées pendant leur cycle review.

---

### Story 9.5-4 : Recherche réglementaire Swiss CO Art. 957a/958f

**As a** mainteneur projet
**I want** conclure formellement la recherche réglementaire Swiss Code des Obligations Art. 957a (formats légaux balance/bilan/résultat) + Art. 958f (conservation 10 ans + audit trail signé)
**So that** la décision « audit-trail-only acceptée v0.1 » documentée dans 9-2b §L6 soit validée par recherche réglementaire concluante OU qu'une story dédiée Epic 14 soit ajoutée

**Provenance :**
- Epic 8 retro action #3 marquée partielle (« Recherche réglementaire Swiss CO Art. 957a — Guy »)
- Story 9-2b §L6 documente comme « audit-trail-only acceptée v0.1, story dédiée Epic 14 si signature requise »
- Pas de validation juridique formelle à ce jour. Risque conformité légale Suisse non-évalué.

**Critères d'acceptation :**

- **Given** documents externes Swiss CO Art. 957a + 958f + référentiels AFC + ECH-0058 (archivage électronique), **When** recherche effectuée, **Then** un document `_bmad-output/planning-artifacts/research-swiss-co-958f.md` produit synthétisant :
  - Champs obligatoires Art. 957a pour balance / bilan / compte de résultat (formats légaux acceptés)
  - Exigences Art. 958f pour conservation 10 ans + signature électronique qualifiée (si applicable PME)
  - Comparaison avec implémentation actuelle Kesh (Story 9-1 + 9-2a + 9-2b export ZIP)
  - Gap analysis explicite : ce qui manque pour conformité légale v1.0 stricte
- **Given** la recherche conclue, **When** Guy + Claude statuent, **Then** décision formelle prise parmi :
  - (a) Implémentation Kesh v0.1 conforme — pas de story additionnelle nécessaire
  - (b) Conformité v0.1 acceptée avec dette explicite — story v0.2 Epic 14 « Swiss CO 958f compliance » créée
  - (c) Conformité v0.1 insuffisante bloquante — story Epic 9.5-bis ajoutée
- **And** la décision met à jour 9-2b §L6 (et 9-2a si applicable) pour refléter le statut final.
- **And** le document de recherche est versionnable + relisible par un non-juriste (technique + interpretation, pas du texte légal brut).

**Effort estimé :** moyen — recherche réglementaire externe. Sources : seco.admin.ch (SECO Suisse), expertsuisse.ch, AFC (Administration fédérale des contributions), ordonnance OLICo (Loi sur les comptes), ECH-0058 archivage.

**Path-dependency :** indépendant des autres stories. Peut démarrer en parallèle.

---

## Action B parallèle (hors story)

**Créer GitHub Milestone `v0.2`** et labelliser les ~20 items catégorie B identifiés en rétro Epic 9 :

| Source | Items |
|---|---|
| **Story 9-2a limitations** | L13 PDF 10k > 5MB, L14 a11y `#bits-c1` (= KF #91 doublon, voir 9.5-1), L15 (à recheck) |
| **Story 9-2b limitations v0.1** | L1 import ZIP retour, L2 filtrage par exercice, L3 buffered RAM, L4 streaming HTTP, L5 traductions DE/IT/EN natives, L7 keshVersion CI/CD bump, L8 chiffrement ZIP, L10 sync→async, L11 perf 5000+, L12 metadata.json self-SHA, L13 UTC date filename, L14 fiscal_years MAX_LIMIT, L15 RAM peak bornage, L16 Language enum Unknown variant, L17 INNER JOIN audit, L18 AC#19 middleware validation |
| **Story 9-2b Pass 1 deferred** | D1 perf ORDER BY filesort, D2 zopfli license/size, D3 hex_encode allocs, D4 export_date timing, D5 aria-busy button |
| **KFs labellisées v0.2 si décidé** | #76 multi-candidates UI (Story 8-4) |

**Owner :** Guy
**Critère succès :** GitHub Milestone `v0.2` créé + ~20 items associés (issues existantes labellisées, ou nouvelles issues `enhancement` créées par bullet point identifiable du `deferred-work.md`).
**Path-dependency :** indépendant. Peut démarrer en parallèle des stories.

---

## Critères d'arrêt Epic 9.5

Epic considéré « done » quand :

- [ ] 4/4 stories avec status `done` dans `sprint-status.yaml`
- [ ] Les 6 KFs Epic 7/9 (#47, #50, #54, #55, #57, #91) sont **toutes fermées** sur GitHub (fix ou closed-as-resolved)
- [ ] `cargo test -p kesh-api --lib config::tests` → 24/24 passent (zero fail local)
- [ ] `audit_log.details_json` JSON keys uniformément snake_case dans `emit_report_export_audit` + `emit_global_export_audit` + `emit_report_audit` (cohérent §scope)
- [ ] CLAUDE.md contient les 3 nouvelles sections (Haiku grep ground-truth, AcceptedProposal batch pattern, Zero tech debt carry-forward)
- [x] Document `research-swiss-co-958f.md` produit + décision formelle (a/b/c) appliquée à 9-2a/9-2b si applicable — **DONE 2026-05-20** via Story 9-5-4 `bmad-dev-story` Opus 4.7 single-pass : verdict **(b) Dette explicite v0.2** confirmé par Guy via checkpoint élicitation T8.3, 9-2b §L6 + 9-2a §L7 mis à jour avec référence document de recherche, GitHub Issue `[Epic 14] Swiss CO 958f signature électronique qualifiée` créée (labels `enhancement` + `v0.2-milestone` + `legal-compliance` + `technical-debt`).
- [ ] GitHub Milestone `v0.2` créé + ~20 items catégorie B labellisés (Action B parallèle complétée)
- [ ] Rétrospective Epic 9.5 produite (status `done` dans `sprint-status.yaml`)
- [ ] 0 régression sur baselines existantes : 28/28 reports_e2e + 20/20 reports_export_e2e + 21/21 exports_global_e2e + autres E2E projet
- [ ] PR Epic 9.5 mergée sur `main` (cohérent pattern « avoid parallel PRs » memory `feedback_avoid_parallel_prs` — retro Epic 9.5 incluse dans PR dernière story)

---

## Risques & questions ouvertes

| # | Risque / question | À traiter dans |
|---|---|---|
| Q1 | Décision JSON keys standard `snake_case` confirmée (cf. §Story 9.5-2 item 2) — à valider Guy avant migration `emit_report_export_audit` | Story 9.5-2 spec validate |
| Q2 | Si 9.5-4 recherche conclue « conformité v0.1 insuffisante bloquante » (option c), Epic 9.5-bis ajoutée pourrait retarder Epic 10. Probabilité estimée faible (jurisprudence PME accepte audit_log + SHA-256 comme preuve d'intégrité), mais non-validée juridiquement à ce jour. | Story 9.5-4 |
| Q3 | KFs Epic 7 dormantes (#54, #55, #57) sont E2E test infrastructure — possibles dépendances cross-story complexes. Si plus de 2 KFs requièrent refactor de helpers communs, splitter 9.5-1 en sous-stories (9.5-1a tests infra + 9.5-1b a11y + 9.5-1c misc) à considérer. | Story 9.5-1 spec validate si effort dépasse seuil |
| Q4 | Memory promotion vers CLAUDE.md : risque de duplication doc (memory user-level + section CLAUDE.md project-level disent la même chose). Décision : CLAUDE.md = source de vérité projet, memories conservées pour traçabilité historique des décisions. | Story 9.5-3 |
| Q5 | KF #91 (DropdownMenu nested button) est lié au composant tier `bits-ui` (Svelte 5). Si fix nécessite contournement (e.g. wrapper custom), risque de friction UX. Alternative : labelliser v0.2 si pas critique pour merge Epic 9.5 — décision spec validate. | Story 9.5-1 spec validate |

---

## Références

- `_bmad-output/implementation-artifacts/epic-9-retro-2026-05-17.md` — rétrospective source
- `_bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md` — `emit_report_export_audit` camelCase original (à migrer)
- `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` — `emit_global_export_audit` snake_case déjà conforme + §L6 audit-trail-only + §L1-L18 catégorie B
- `_bmad-output/planning-artifacts/epic-9.md` — pattern epic référence (sections, frontmatter, stories)
- Memory `feedback_zero_tech_debt_carryforward` — politique projet
- Memory `feedback_haiku_review_diff_combined` — Haiku grep ground-truth pattern
- Memory `feedback_avoid_parallel_prs` — pattern PR retro incluse dans dernière story
- CLAUDE.md sections existantes : §"Review Iteration Rule" (pour ajout sous-section Haiku), §"Règle de commit et push" (pour ajout §"Tech debt management")
- GitHub Issues : #47, #50, #54, #55, #57, #91 (à fermer dans 9.5-1) — #76 (à labelliser v0.2 dans Action B)
- Historique : Epic 7 « Technical Debt Closure » (KF-001..007 fermées pré-Epic 8) — pattern référence
