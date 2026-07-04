# Story 19.6a : Fondation rapports projet + Dépenses par projet

Status: ready-for-dev

<!-- Sous-story 1/2 de l'umbrella 19-6 (SPLIT). Pose la fondation partagée
     (module project_report : scope racine+enfants, période 2 modes) + le 1er
     rapport « Dépenses par projet » bout-en-bout (agrégation + drill-down +
     PDF/CSV + API + onglet frontend). 19-6b (Rendement) réutilisera la fondation.
     RÉFÉRENCE ground-truth complète : 19-6-rapports-analytiques.md (Dev Notes). -->

## Story

As a comptable/indépendant PME utilisant Kesh,
I want un rapport **Dépenses par projet** — toutes les charges d'un projet (et de ses sous-projets), groupées par sous-projet puis par compte, avec drill-down jusqu'à l'écriture et export PDF/CSV, en mode exercice OU cumulé depuis l'origine,
so that je capte toutes les dépenses déductibles d'un projet de rénovation sans en oublier, prêtes pour ma déclaration fiscale.

## Contexte & source

- **Umbrella 19-6** (`_bmad-output/implementation-artifacts/19-6-rapports-analytiques.md`) — **RÉFÉRENCE ground-truth** : cartographie Explore complète (kesh-report, routes, pdf, csv, frontend, i18n) + décisions DC1-DC8 figées Guy 2026-07-04. Lire ses **Dev Notes** avant d'implémenter.
- Cette sous-story = **T1 (fondation) + T2 (Dépenses)** + API/frontend/tests pour ce seul rapport. Dépend de 19-1..19-5 (dimension `project_id` + données taguées, mergées/en PR).
- **Réutilise sans réinventer** : template agrégation `income_statement.rs::fetch_section` ; `pdf.rs` (`PdfBuilder`, `draw_header/draw_account_row/draw_totals_footer`, `VatPdfLabels` = précédent labels dédiés) ; `csv.rs` (`make_writer/write_bom/format_amount_iso`) ; `routes/reports.rs` (`get_income_statement`/`export_income_statement`, `load_pdf_context`, `build_export_response_with_locale`, `resolve_type_slug`) ; frontend `reports/+page.svelte` (tabs + Views) + `reports.api.ts` + `projects.api.ts::listProjects`.

## Décisions applicables (de l'umbrella, figées)

- **DC1** — classification par `accounts.account_type` (Expense = charge). Signe Expense = `debit − credit`.
- **DC3** — rollup 2 niveaux : rapport racine agrège racine + sous-projets (actifs+archivés) ; rapport sous-projet = lui seul. Rollup en Rust (`projects::list_by_company` + filtre `parent_id = root.id`).
- **DC4** — 2 modes : `fiscal_year` (filtre `je.fiscal_year_id = ?`, réutilise `ReportPeriod`) ou `cumulative` (pas de filtre exercice ; borne `je.entry_date` ∈ [`project.start_date` si Some, `project.end_date.unwrap_or(today)`]). `today` passé depuis le handler (pas de `Date::now()` en lib).
- **DC5** — Dépenses = HT (somme des lignes `Expense`).
- **DC7** — lecture, tous rôles, scopé `company_id` JWT.
- Drill-down = écritures inline expandables (pas de lien journal).

## Acceptance Criteria

### Fondation (kesh-report) — partagée avec 19-6b

1. Nouveau module `crates/kesh-report/src/project_report.rs`, enregistré `lib.rs` (module + re-exports). Contient la **fondation partagée** :
   - `ProjectReportScope { root: ProjectInfo, subprojects: Vec<ProjectInfo>, project_ids: Vec<i64> }` où `ProjectInfo = { id, code, name }`. Construit par `resolve_scope(pool, company_id, project_id) -> Result<ProjectReportScope, ReportError>` : charge le projet (`projects::get_for_company`), si inconnu → `ReportError` mappé 404 (nouveau variant `ProjectNotFound { project_id }`) ; si racine (`parent_id IS NULL`) → scope = racine + enfants (`projects::list_by_company` filtré `parent_id == root.id`, archivés inclus) ; si sous-projet → scope = lui seul. `project_ids` = tous les ids du scope.
   - `enum ProjectPeriodMode { FiscalYear { period: ReportPeriod }, Cumulative { start: Option<NaiveDate>, end: NaiveDate } }` + `period_label(&self) -> String` (ex. « Exercice 2026 » ou « Cumulé jusqu'au 04.07.2026 »). Un helper interne `where_clause_and_binds` factorisant le filtre `je` (fiscal_year vs entry_date range) réutilisable par les 2 rapports.
2. Nouveau variant `ReportError::ProjectNotFound { project_id: i64 }` (`errors.rs`) mappé 404 côté handler (`AppError`).

### Rapport « Dépenses par projet »

3. `generate_project_expenses(pool, company_id, scope, mode) -> Result<ProjectExpensesReport, ReportError>` : agrège les lignes `Expense` taguées `jel.project_id IN (scope.project_ids)`, groupées par (sous-projet, compte), signe `debit − credit`. SQL calqué `income_statement::fetch_section` + `AND jel.project_id IN (?,…)` (placeholders dynamiques) + `jel.project_id` au SELECT/GROUP BY + filtre `je` selon mode. DTO camelCase :
   - `ProjectExpensesReport { report_type: "project-expenses", project: ProjectInfo, mode: String, period_label: String, sections: Vec<ProjectExpenseSection>, grand_total: Decimal }`.
   - `ProjectExpenseSection { project: ProjectInfo, is_root: bool, rows: Vec<ExpenseAccountRow>, subtotal: Decimal }` (une section par projet du scope ayant ≥ 1 ligne ; sections triées racine d'abord puis sous-projets par code).
   - `ExpenseAccountRow { account_id, account_number, account_name, amount, entries: Vec<ProjectEntryRef> }` (drill-down AC4).
4. **Drill-down** (AC4) : `entries` = écritures contributrices du (sous-projet, compte) : `ProjectEntryRef { entry_id, entry_number, entry_date, description, amount }` (`amount` = `debit − credit` de la/les ligne(s) de ce compte dans cette écriture pour ce projet). Une requête détail (join `journal_entries`, filtre Expense + `project_id IN (...)` + mode), agrégée en Rust par (project_id, account_id, entry_id). Triée par date puis entry_number.
5. Export PDF `render_project_expenses_pdf(report, ctx, labels) -> Result<Vec<u8>, ReportError>` (`pdf.rs`) : `PdfBuilder::new`, `draw_header` (titre « Dépenses par projet — {code} {name} » + `period_label`), guard vide → message, par section : ligne titre sous-projet + `draw_account_row` par compte + `draw_totals_footer` sous-total, puis total général. Labels dédiés `ProjectExpensesPdfLabels` (précédent `VatPdfLabels`) + défauts FR-CH. Re-export `lib.rs`.
6. Export CSV `render_project_expenses_csv(report, w)` (`csv.rs`) : BOM + header `["Projet","SousProjet","NumeroCompte","NomCompte","Montant"]` + une ligne par (section, compte) + lignes sous-total/total. `format_amount_iso`. Re-export `lib.rs`.

### API (kesh-api)

7. `ProjectReportQuery { project_id: i64, mode: ProjectModeParam (fiscal_year|cumulative, serde), fiscal_year_id: Option<i64>, period_start: Option<NaiveDate>, period_end: Option<NaiveDate> }` (camelCase) dans `routes/reports.rs`. Validation : `mode=fiscal_year` sans `fiscalYearId` → 400 `AppError::Validation` ; `project_id <= 0` → 400. Résolution mode : fiscal_year → `ReportPeriod::resolve` puis `ProjectPeriodMode::FiscalYear` ; cumulative → `ProjectPeriodMode::Cumulative { start: project.start_date, end: project.end_date.unwrap_or(today) }` avec `today = chrono::Utc::now().date_naive()` (résolu handler-side).
8. `GET /api/v1/reports/project-expenses` (JSON) + `GET /api/v1/reports/project-expenses/export?format=pdf|csv`. Handlers calqués `get_income_statement`/`export_income_statement` : `resolve_scope` → mode → `generate_project_expenses` → audit best-effort (`emit_report_audit`/`emit_report_export_audit`, type `"project-expenses"`) → export via `build_export_response_with_locale`. Enregistrés `lib.rs` **avant le `;`** d'`authenticated_routes` (anti-IDOR warning `lib.rs:611`). Projet inconnu → 404, cross-company → 404.
9. i18n slug `reports-filename-project-expenses` = `depenses-par-projet` dans les **5 locales** (`crates/kesh-i18n/locales/*/messages.ftl`, section `reports-filename-*`) + `resolve_type_slug` gère le nouveau type. Frontend `TYPE_SLUGS_FALLBACK` idem.

### Frontend

10. `reports.types.ts` : DTO `ProjectExpensesReport` + sous-types (miroir camelCase). `reports.api.ts` : `getProjectExpenses(query)` (via `buildQuery` + `apiClient.get`), URL export via `getReportExportUrl`, slug fallback.
11. Onglet « Dépenses par projet » dans `reports/+page.svelte` (tab + DTO `$state` + `generate()` switch + `exportReport`). Sélecteurs : **projet** (`listProjects()`, arbre 2 niveaux — copie pattern 19-x), **mode** (Exercice → sélecteur `fiscalYearId` existant ; Cumulé), bouton Générer + export PDF/CSV.
12. Composant `ProjectExpensesView.svelte` : tableau groupé par section (sous-projet) → compte ; lignes de compte **expandables** vers les écritures (drill-down AC4) ; sous-totaux par section + total général. Montants formatés suisses (apostrophe milliers). `data-testid` cohérents (`project-expenses-*`). i18n fallback FR inline.

### Tests / non-régression / qualité

13. Tests agrégation (`crates/kesh-api/tests/` ou `crates/kesh-report/tests/`, `#[sqlx::test(migrator)]`, helper `make_project` + seed écritures taguées) : (a) 2 sous-projets, lignes Expense taguées mixtes → sections + sous-totaux + total ; **lignes de contrepartie taguées (banque/TVA/dette) exclues** (DC1) ; (b) mode cumulé traverse 2 exercices (2 fiscal_years) ; (c) rollup : rapport racine inclut enfants, rapport sous-projet = lui seul ; (d) drill-down : entries contributrices correctes (n°, date, montant) ; (e) projet inconnu → 404, cross-company → 404 ; (f) scope sans ligne taguée → rapport vide (sections vides, grand_total 0), pas d'erreur.
14. Tests API : JSON + export PDF (signature `%PDF`) + CSV (BOM + header) ; validation mode/fiscalYearId (400) ; multi-tenant (projet autre company → 404).
15. Non-régression : les 5 rapports existants inchangés ; pas de nouvelle migration ; export souveraineté/backup inchangé. Frontend : unit `reports.api` (query 2 modes) + View (rendu sections + expand). E2E Playwright : générer Dépenses par projet (projet + mode) → assertion tableau + (optionnel) export. Scoper les `getByRole('option')`.
16. Quality gate **Test Locally First** complet, exit-codes vérifiés (pas de pipe grep), workspace serial (kesh-report/kesh-db touchés → `cargo test --workspace -j1 -- --test-threads=1`). CHANGELOG [Non publié] : entrée rapport Dépenses par projet.

## Tasks / Subtasks

- [x] **T1 — Fondation kesh-report** (AC: 1-2) : `project_report.rs` (`ProjectReportScope` + `resolve_scope`, `ProjectPeriodMode` + `period_label` + `where_clause_and_binds`), `ReportError::ProjectNotFound`, re-exports lib.rs.
- [x] **T2 — Agrégation Dépenses + drill-down** (AC: 3-4) : `generate_project_expenses` + DTOs + requête détail drill-down + tests agrégation (13 a-f).
- [x] **T3 — Export PDF + CSV** (AC: 5-6) : `render_project_expenses_pdf` (labels dédiés) + `render_project_expenses_csv` + re-exports.
- [x] **T4 — API** (AC: 7-9) : `ProjectReportQuery`, 2 endpoints, mapping `ProjectNotFound`→404, i18n slug 5 locales, audit, enregistrement lib.rs avant `;`.
- [ ] **T5 — Frontend** (AC: 10-12) : types + api + onglet + `ProjectExpensesView` (drill-down expandable) + sélecteurs projet/mode + i18n.
- [ ] **T6 — Tests + gate** (AC: 13-16) : API tests + non-régression + frontend unit/E2E + CHANGELOG + Test Locally First.

## Dev Notes

**RÉFÉRENCE PRINCIPALE** : `_bmad-output/implementation-artifacts/19-6-rapports-analytiques.md` § Dev Notes (cartographie Explore complète file:line — income_statement.rs:55, period.rs:78, pdf.rs:217/329/387/416/879, csv.rs:29/47/138, reports.rs:168/339/662/730/695/807, lib.rs:594-636/611, frontend reports/+page.svelte + reports.api.ts + i18n messages.ftl:936) + Pièges connus (IN dynamique, mode cumulé sans fiscal_year, rollup sous-projet ciblé, exclusion contrepartie via account_type, E2E option scoping).

**Confirmés cette session** :
- `income_statement.rs::fetch_section` (SQL template + `AccountBalance` DTO `balance_sheet.rs:47-58` avec `#[sqlx(rename="number/name")]`).
- `ReportPeriod::resolve` (`period.rs:78`) + `resolve_dates` (asymétrique 4 cas).
- `ReportError` (`errors.rs`) : variants `Db/FiscalYearNotFound/PeriodInvalid/PeriodOutOfFiscalYear/PdfGeneration/CsvGeneration` → **ajouter `ProjectNotFound`**.
- `lib.rs` re-exports pattern (modules + `generate_*` + `render_*_pdf/csv` + labels).
- `projects::list_by_company(pool, company_id, include_archived)` (racines→enfants), `get_for_company`.

**Conventions** : multi-tenant `company_id` JWT (IDOR), DTOs camelCase, `AppError` 4xx (jamais 500 métier), `ReportError`→mapping HTTP, PDF labels FR-CH défaut, i18n fallback FR frontend, pas de `Date::now()` en lib (passer `today` handler-side), pas de `unreachable!()` sur match account_type (défaut → erreur). Pas de migration. Branche `story/19-6-rapports-analytiques` (créée depuis main). Commit par étape BMAD.

**Pièges critiques** :
1. `IN (...)` dynamique : placeholders selon `scope.project_ids.len()`, binds ordonnés (comme `validate_taggable_in_tx`). Scope jamais vide.
2. Mode cumulé : PAS de filtre `fiscal_year_id` ; borne `entry_date` ∈ [start?, end]. `today` passé en param.
3. Sous-projet ciblé → scope = lui seul (pas de remontée parent).
4. Filtre `account_type = 'Expense'` exclut mécaniquement banque/TVA/dette taguées (test AC13a).
5. E2E : scoper `getByRole('option')` (collision selects).

## Questions

Aucune — DC5 (HT), DC8 (hors 10xx, s'applique à 19-6b), drill-down inline : tous figés Guy 2026-07-04 (cf. umbrella).

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 — dev-story orchestré inline (run autonome, session longue).

### État au 2026-07-04 (BACKEND DONE, frontend + review à finir en contexte frais)

**FAIT (committé `c59aeb1`)** — backend bout-en-bout, testé :
- **T1** fondation `crates/kesh-report/src/project_report.rs` : `ProjectInfo`, `ProjectReportScope` + `resolve_scope` (racine→enfants, sous-projet→lui seul), `ProjectPeriodMode {FiscalYear, Cumulative}` + `period_label`/`as_str`/`je_filter` (fragment SQL + binds), `in_placeholders`. `ReportError::ProjectNotFound` (errors.rs) → mappé 404 (`kesh-api/errors.rs` : `AppError::Database(DbError::NotFound)`). Re-exports `lib.rs`.
- **T2** `generate_project_expenses` : 2 requêtes (agrégat par (project_id, compte) signe debit−credit + détail drill-down par écriture), assemblage sections en Rust (racine puis enfants par code, sections vides omises), `IN(...)` dynamique + `bind_period`. DTOs camelCase (`ProjectExpensesReport/ProjectExpenseSection/ExpenseAccountRow/ProjectEntryRef`).
- **T3** `render_project_expenses_pdf` + `ProjectExpensesPdfLabels` (pdf.rs, `draw_expense_row`) + `render_project_expenses_csv` (csv.rs, BOM/;/CRLF). Re-exports lib.rs.
- **T4** API `routes/reports.rs` : `ProjectReportQuery` + `ProjectModeParam` (fiscal_year|cumulative), `resolve_project_report` (valide project_id>0 + fiscalYearId requis si fiscal_year, résout mode ; cumulé `today = Utc::now().date_naive()`), handlers `get_project_expenses` + `export_project_expenses` (+ `build_project_export_response` dates-based, `project_mode_dates`), audit `emit_project_report_audit`/`_export_audit`. Routes enregistrées `lib.rs` AVANT le `;`. i18n slug `reports-filename-project-expenses` (4 locales fr/de/it/en).
- **Tests** `crates/kesh-report/tests/project_expenses.rs` : 4/4 verts (rollup+drill-down+exclusion contrepartie, cumulé cross-FY, 404, scope vide). fmt + clippy 0 (kesh-report + kesh-api).

**RESTE (contexte frais)** :
- **T5 frontend** : `reports.types.ts` DTO `ProjectExpensesReport` + `reports.api.ts` `getProjectExpenses`/export + onglet dans `reports/+page.svelte` (⚠️ query différente : sélecteur projet `listProjects()` + sélecteur mode exercice/cumulé, pas seulement `fiscalYearId` — le machinery `ReportType`/`ReportQuery` existant est typé pour fiscalYearId, prévoir une intégration soignée ou un état dédié) + `ProjectExpensesView.svelte` (sections + drill-down expandable) + `TYPE_SLUGS_FALLBACK` `project-expenses: 'depenses-par-projet'`.
- **T6 tests** : API HTTP e2e (`crates/kesh-api/tests/` : JSON + export PDF %PDF + CSV BOM + validation mode/fiscalYearId 400 + multi-tenant 404) ; frontend unit (`reports.api` query 2 modes) + View ; E2E Playwright ; CHANGELOG [Non publié] ; gate Test Locally First workspace serial.
- **Code-review** 2 passes (dev Opus → Sonnet → Haiku) jusqu'à 0>LOW.

### File List (à jour, backend)

- crates/kesh-report/src/project_report.rs (nouveau)
- crates/kesh-report/src/lib.rs (module + re-exports)
- crates/kesh-report/src/errors.rs (ProjectNotFound)
- crates/kesh-report/src/pdf.rs (ProjectExpensesPdfLabels + render_project_expenses_pdf + draw_expense_row)
- crates/kesh-report/src/csv.rs (render_project_expenses_csv)
- crates/kesh-report/tests/project_expenses.rs (nouveau, 4 tests)
- crates/kesh-api/src/routes/reports.rs (ProjectReportQuery + 2 handlers + helpers + audit)
- crates/kesh-api/src/errors.rs (ReportError::ProjectNotFound → 404)
- crates/kesh-api/src/lib.rs (2 routes)
- crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl (slug)

## Change Log

- **Dev backend (2026-07-04, Opus 4.8)** : T1-T4 + tests intégration agrégation (4/4). Backend Dépenses par projet complet (fondation partagée réutilisable par 19-6b, agrégation + drill-down + PDF/CSV + API 2 endpoints + i18n). Frontend (T5) + tests HTTP/E2E (T6) + code-review à compléter en contexte frais (session longue — split qualité). Commit `c59aeb1`.
