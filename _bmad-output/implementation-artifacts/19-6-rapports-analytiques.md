# Story 19.6 : Rapports analytiques par projet (Dépenses + Rendement)

Status: ready-for-dev

<!-- Le PAYOFF d'Epic 19 : deux rapports lisant la dimension project_id posée
     par 19-1 et alimentée par 19-2..19-5. Fondation d'agrégation partagée
     (group by projet + account_type + rollup sous-projets + filtre exercice OU
     cumulé), puis 2 rapports au-dessus. Réutilise massivement l'infra
     kesh-report existante (income_statement = template exact). -->

## Story

As a comptable/indépendant PME utilisant Kesh,
I want deux rapports par projet — **Dépenses par projet** (toutes les charges d'un projet, sans en oublier) et **Rendement par projet** (coût investi vs revenus → résultat net + rendement %) — avec agrégation des sous-projets, choix exercice OU cumulé depuis l'origine, drill-down et export PDF/CSV,
so that je peux préparer mes déductions fiscales de rénovation et analyser le rendement de mes investissements à partir des écritures déjà taguées.

## Contexte & source

- **Epic 19** — design `_bmad-output/planning-artifacts/epic-19-analytique-projet-design.md` §4 (« Modèle de reporting — le vrai livrable ») + §2 D2 (rollup sous-projets), D3 (périmètre : charges + produits + bilan/actifs), D4 (exercice ET cumulé). Story 19-6, dépend de 19-1..19-5 (données à agréger — toutes mergées/en PR).
- **Déjà en place** (ne rien réinventer) :
  - dimension `journal_entry_lines.project_id` (19-1) + index `idx_jel_project` ; entité `projects` (2 niveaux, `parent_id`, `archived`, `start_date`/`end_date`) + repo `projects::list_by_company(pool, company_id, include_archived)` (`repositories/projects.rs:20`, tri racines→enfants `ORDER BY COALESCE(parent_id, id), parent_id IS NOT NULL, id`) + `get_for_company:43`.
  - **infra kesh-report complète** : modules par rapport (`generate(pool, company_id, period) -> Result<T, ReportError>` + `render_*_pdf` + `render_*_csv`), template exact `income_statement.rs` (classification Revenue/Expense, join lines→entries→accounts), `pdf.rs` (`PdfBuilder`, `draw_header/draw_account_row/draw_totals_footer`, `VatPdfLabels` = précédent labels dédiés), `csv.rs` (`make_writer`/`write_bom`/`format_amount_iso`, BOM UTF-8 + `;` + CRLF), `period.rs` (`ReportPeriod::resolve`).
  - routes `routes/reports.rs` : `GET /api/v1/reports/{type}` (JSON) + `GET /api/v1/reports/{type}/export` (PDF/CSV), `ReportQuery { fiscal_year_id, period_start, period_end }`, `load_pdf_context`, `build_export_response_with_locale`, `resolve_type_slug` (i18n `reports-filename-{type}`), audit `emit_report_audit`/`emit_report_export_audit`.
  - frontend `reports/+page.svelte` (tabs + DTO state + `generate()` switch + `*View.svelte` par rapport), `reports.api.ts` (`getIncomeStatement`, `downloadReport`/`triggerDownload`, `buildExportFilename`, `TYPE_SLUGS_FALLBACK`), `projects.api.ts::listProjects`.
- **Ce que fait cette story** : ajouter la **fondation projet-report** (période cumulée multi-exercices + rollup) + les 2 rapports (agrégation, API JSON, PDF, CSV) + les 2 écrans frontend + i18n.
- **Nouveauté d'infra** : aucun rapport cumulé multi-exercices n'existe (tous filtrent `je.fiscal_year_id = ?`). Le mode **cumulé** de 19-6 introduit une résolution de période sans filtre exercice, bornée par `entry_date` (dates du projet si présentes).

## Décisions de design (DC — lire avant les AC)

- **DC1 — Classification par `account_type`** [FIGÉ] : la nature d'une ligne taguée est donnée par `accounts.account_type` (join `journal_entry_lines.account_id → accounts.id`), **jamais** par le numéro de compte. Charge = `Expense`, Produit = `Revenue`, Actif (coût investi) = `Asset`. Signe : `Expense`/`Asset` = `debit − credit` ; `Revenue` = `credit − debit` (convention `income_statement`/`trial_balance`). Les lignes de contrepartie (banque, TVA, dette fournisseur, débiteur) taguées par propagation document-level (19-3/19-4/19-5) sont **naturellement exclues** des rapports car filtrées par `account_type` (Liability/Asset-banque hors périmètre charge/produit ; voir DC5 pour l'actif du rendement).
- **DC2 — Périmètre des rapports** [FIGÉ] :
  - **Dépenses par projet** : lignes de comptes `Expense` taguées, groupées par **sous-projet** puis par **compte**, avec total par sous-projet + total projet. Drill-down = lignes d'écriture contributrices par compte.
  - **Rendement par projet** : **Coût investi** = Σ lignes `Asset` + `Expense` taguées ; **Revenus** = Σ lignes `Revenue` taguées ; **Résultat net** = Revenus − Charges(`Expense`) ; **Rendement %** = Revenus / Coût investi (si Coût investi > 0, sinon `null`/« — »). Vue par sous-projet + rollup projet.
- **DC3 — Rollup 2 niveaux** [FIGÉ] : un rapport sur un projet **racine** agrège le projet **et ses sous-projets actifs+archivés** (l'historique reste lisible). `list_by_company` fournit racines+enfants ; le rollup se fait **en Rust** (`BTreeMap` par sous-projet, précédent `vat_report.rs`), filtrant les `project_id IN (racine, enfants…)`. Un rapport sur un **sous-projet** ne montre que lui (pas de parent). Les résultats sont groupés par sous-projet (une section « projet lui-même » + une section par sous-projet).
- **DC4 — Deux modes de période** [FIGÉ] :
  - `mode=fiscal_year` : `fiscalYearId` requis → filtre `je.fiscal_year_id = ?` (réutilise `ReportPeriod::resolve`, borne dates optionnelles `periodStart`/`periodEnd`).
  - `mode=cumulative` : **pas** de filtre exercice → toutes les écritures taguées du projet(+enfants), bornées par `je.entry_date` entre `project.start_date` (si défini, sinon aucune borne basse) et `project.end_date` (si défini, sinon aujourd'hui). Nouvelle résolution `ProjectReportPeriod` (pas d'exercice unique). Traverse les clôtures d'exercice.
- **DC5 — TVA dans les dépenses : HT** [FIGÉ — Guy 2026-07-04] : « Dépenses par projet » somme les lignes `Expense` = **montants HT** (la TVA récupérable est sur un compte `Asset`, hors périmètre `Expense`). Pour un non-assujetti, pas de TVA récupérable séparée → HT = TTC de fait. Simple et cohérent.
- **DC8 — Coût investi exclut la trésorerie** [FIGÉ — Guy 2026-07-04] : le « Coût investi » du Rendement = Σ lignes `Expense` + Σ lignes `Asset` **dont le numéro de compte ne commence PAS par `10`** (exclut banque/caisse classe 10xx). Rationale : la ligne banque d'un règlement, taguée par propagation document-level (19-3/19-4/19-5), ne doit pas gonfler le coût investi ; seuls les vrais comptes d'immobilisation/actif d'investissement (14xx/15xx…) comptent. SQL : `(a.account_type = 'Expense') OR (a.account_type = 'Asset' AND a.number NOT LIKE '10%')`. Testé (AC16c + piège 4).
- **DC6 — Nommage « Projet »** [FIGÉ] : terme « Projet » (déjà utilisé UI 19-1..19-5). Rapports nommés « Dépenses par projet » / « Rendement par projet ».
- **DC7 — RBAC** [FIGÉ] : rapports en lecture, **tous rôles** (comme les autres rapports, `authenticated_routes`). Scopé `company_id` du JWT (anti-IDOR).

## Acceptance Criteria

### Fondation (kesh-report + période cumulée)

1. Nouveau module `crates/kesh-report/src/project_report.rs` (ou `analytic_report.rs`) exposant la fondation partagée : résolution de la liste des `project_id` à agréger (racine + enfants via `projects::list_by_company` filtré, en Rust) ; résolution de période à deux modes (`ProjectReportMode::FiscalYear { period: ReportPeriod }` | `ProjectReportMode::Cumulative { start: Option<NaiveDate>, end: NaiveDate }`). Enregistré dans `lib.rs` (module + re-exports).
2. Type `ProjectReportScope` = `{ root: Project, subprojects: Vec<Project>, project_ids: Vec<i64> }` construit depuis `projects::get_for_company` + filtre enfants (`parent_id = root.id`). Si le projet ciblé est un sous-projet → scope = lui seul. Projet inconnu/cross-company → `ReportError` mappé 404.

### Rapport « Dépenses par projet »

3. `generate_project_expenses(pool, company_id, scope, period_mode) -> Result<ProjectExpensesReport, ReportError>` : agrège les lignes `Expense` taguées `project_id IN (scope.project_ids)`, groupées par (sous-projet, compte), signe `debit − credit`. SQL calqué `income_statement::fetch_section` + `AND jel.project_id IN (...)` + `je` filtré selon le mode (fiscal_year OU entry_date range). DTO : `{ report_type, project: {id,code,name}, mode, period_label, sections: Vec<ProjectSection>, grand_total }` où `ProjectSection = { project: {id,code,name}, is_root, rows: Vec<AccountAmount>, subtotal }` et `AccountAmount = { account_id, account_number, account_name, amount, entry_count }`. camelCase Serialize.
4. **Drill-down** : chaque `AccountAmount` porte `entries: Vec<ProjectEntryRef>` (`{ entry_id, entry_number, entry_date, description, amount }`) — les écritures contributrices (une requête détail par scope, jointe entries, filtrée `Expense` + `project_id IN (...)`, triée date). Permet l'expansion UI ligne-à-écriture (design « drill-down jusqu'à l'écriture »).
5. Export PDF `render_project_expenses_pdf(report, ctx, labels) -> Result<Vec<u8>, ReportError>` : `PdfBuilder` + en-tête (nom projet + mode/période) + par section (sous-projet) une ligne titre + `draw_account_row` par compte + `draw_totals_footer` sous-total, puis total général. Labels dédiés `ProjectReportPdfLabels` (précédent `VatPdfLabels`), défauts FR-CH.
6. Export CSV `render_project_expenses_csv(report, w)` : BOM + header `["Projet","SousProjet","NumeroCompte","NomCompte","Montant"]` + une ligne par (sous-projet, compte) + lignes total. `format_amount_iso`.

### Rapport « Rendement par projet »

7. `generate_project_return(pool, company_id, scope, period_mode) -> Result<ProjectReturnReport, ReportError>` : par sous-projet, calcule **coûtInvesti** (Σ `Expense` + Σ `Asset` **hors classe 10xx** — DC8, signe `debit−credit`), **revenus** (Σ `Revenue`, signe `credit−debit`), **résultatNet** (revenus − charges `Expense`), **rendementPct** (`revenus / coûtInvesti` si > 0, sinon `null`). Rollup total projet. DTO `{ report_type, project, mode, period_label, sections: Vec<ProjectReturnSection>, totals: {cout_investi, revenus, resultat_net, rendement_pct} }`.
8. Export PDF `render_project_return_pdf` + CSV `render_project_return_csv` (header `["Projet","SousProjet","CoutInvesti","Revenus","ResultatNet","RendementPct"]`). Rendement % formaté (ex. `12.50%` ou `—` si null).

### API (kesh-api)

9. Query struct `ProjectReportQuery { project_id: i64, mode: ProjectReportModeParam (fiscal_year|cumulative), fiscal_year_id: Option<i64> (requis si fiscal_year), period_start: Option<NaiveDate>, period_end: Option<NaiveDate> }` (camelCase). Validation : `mode=fiscal_year` sans `fiscalYearId` → 400 ; `project_id <= 0` → 400.
10. `GET /api/v1/reports/project-expenses` (JSON) + `GET /api/v1/reports/project-expenses/export?format=pdf|csv` ; `GET /api/v1/reports/project-return` (JSON) + `.../export`. Handlers calqués `get_income_statement`/`export_income_statement` : résout scope + période, génère, audit best-effort, export via `build_export_response_with_locale`. Enregistrés dans `lib.rs` **avant** le `;` d'`authenticated_routes` (anti-IDOR, cf. warning `lib.rs:611`). Projet inconnu → 404.
11. i18n filename slugs : clés `reports-filename-project-expenses` = `depenses-par-projet` et `reports-filename-project-return` = `rendement-par-projet` dans les **5 locales** (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-US,...}/messages.ftl`, section `reports-filename-*` ~ligne 936 fr) + `resolve_type_slug` gère les nouveaux types.

### Frontend

12. `reports.types.ts` : DTOs TS `ProjectExpensesReport` / `ProjectReturnReport` (miroir camelCase). `reports.api.ts` : `getProjectExpenses(query)` / `getProjectReturn(query)` (via `buildQuery` + `apiClient.get`), URLs export via `getReportExportUrl`, `TYPE_SLUGS_FALLBACK` + les 2 nouveaux slugs.
13. Deux onglets dans `reports/+page.svelte` (tabs `:240` + DTO `$state` + `generate()` switch + `exportReport`). Chaque onglet : **sélecteur de projet** (`listProjects()`, arbre 2 niveaux — copie pattern 19-x) + **sélecteur de mode** (Exercice → sélecteur `fiscalYearId` ; Cumulé) + bouton Générer + export PDF/CSV (`downloadReport`).
14. Deux composants `ProjectExpensesView.svelte` / `ProjectReturnView.svelte` : Dépenses = tableau groupé par sous-projet → compte, lignes de compte **expandables** vers les écritures (drill-down AC4), sous-totaux + total. Rendement = tableau par sous-projet (coût investi / revenus / résultat net / rendement %) + ligne total. Montants formatés suisses (apostrophe milliers). `data-testid` cohérents. i18n fallback FR inline.
15. Entrée menu : les rapports apparaissent dans la page **Rapports** existante (nouveaux onglets) — pas de nouvelle route. (Design mentionne « Menu Mensuel → Rapports » : c'est la page `reports` actuelle.)

### Tests / non-régression / qualité

16. Tests agrégation (`crates/kesh-report/tests/` ou `crates/kesh-api/tests/` — `#[sqlx::test(migrator)]`) : (a) Dépenses — projet avec 2 sous-projets, écritures Expense taguées mixtes → sections + sous-totaux + total corrects ; lignes de contrepartie taguées (banque/TVA) **exclues** (DC1) ; (b) mode cumulé traverse 2 exercices ; (c) Rendement — coût investi (`Expense` + `Asset` hors 10xx, DC8) + revenus + résultat net + rendement % ; **une ligne banque 10xx taguée est exclue du coût investi** ; rendement null si coût investi = 0 ; (d) rollup : rapport racine inclut enfants, rapport sous-projet = lui seul ; (e) drill-down : entries contributrices correctes ; (f) projet inconnu/cross-company → 404 ; (g) scope vide (aucune ligne taguée) → rapport vide non-erreur.
17. Tests API (`crates/kesh-api/tests/`) : endpoints JSON + export PDF (signature %PDF) + CSV (BOM + header) pour les 2 rapports ; validation mode/fiscalYearId ; multi-tenant (projet d'une autre company → 404).
18. Non-régression : les 5 rapports existants inchangés (income_statement etc.) ; `ReportError` variants réutilisés ou étendus proprement ; pas de nouvelle migration. Export souveraineté/backup inchangé.
19. Frontend : unit `reports.api` (sérialisation query 2 modes) + Views (rendu sections). E2E Playwright : générer Dépenses par projet (sélecteur projet + mode) → assertion tableau + export. Scoper les `getByRole('option')` (leçon 19-2/19-4/19-5).
20. Quality gate **Test Locally First** complet, exit-codes vérifiés (pas de pipe grep — `feedback_cargo_test_pipe_masks_exit`), workspace serial si kesh-db/kesh-report touchés. CHANGELOG [Non publié] : entrée rapports analytiques. E2E réel si le harness le permet.

## Règle de splitting (escape-hatch)

Cette story est large (2 rapports × agrégation+API+PDF+CSV+frontend+i18n, ~6 aires). Elle reste **unique** car les 2 rapports partagent ~70% d'infra (scope, période, rollup, PDF/CSV builders) et la fondation `income_statement` est un template éprouvé → revue file-by-file possible. **Si `bmad-create-story validate` boucle > 4 passes OU si dev-story sature**, splitter en **19-6a (fondation + Dépenses, chemin rénovations)** + **19-6b (Rendement, réutilise la fondation)** — ordre du design « chemin minimal 19-1→19-3→19-6 ». Documenter la dérogation le cas échéant.

## Tasks / Subtasks

- [ ] **T1 — Fondation kesh-report** (AC: 1-2) : module `project_report.rs` (scope racine+enfants, période 2 modes `ProjectReportMode`), re-exports `lib.rs`, `ReportError` étendu si besoin (ProjectNotFound → 404).
- [ ] **T2 — Rapport Dépenses** (AC: 3-6) : `generate_project_expenses` + DTOs + drill-down + `render_project_expenses_pdf/csv` + labels dédiés.
- [ ] **T3 — Rapport Rendement** (AC: 7-8) : `generate_project_return` + DTOs + `render_project_return_pdf/csv`.
- [ ] **T4 — API** (AC: 9-11) : `ProjectReportQuery`, 4 endpoints (2 JSON + 2 export), enregistrement lib.rs avant `;`, i18n slugs 5 locales, audit.
- [ ] **T5 — Frontend** (AC: 12-15) : types + api fns + 2 onglets + 2 Views (drill-down expandable Dépenses) + sélecteurs projet/mode/exercice + i18n.
- [ ] **T6 — Tests + gate** (AC: 16-20) : tests agrégation (a-g) + API + non-régression + frontend unit/E2E + CHANGELOG + Test Locally First.

## Dev Notes

### Ground-truth (cartographie Explore 2026-07-04)

**Données**
- `journal_entry_lines` (`entities/journal_entry.rs:142-153`) : `id, entry_id, account_id, line_order, debit, credit, project_id: Option<i64>`. Pas de date/exercice sur la ligne → join `entry_id → journal_entries` (`:126-139` : `fiscal_year_id: i64`, `entry_date: NaiveDate`, `journal`, `company_id`).
- `accounts` (`entities/account.rs:71-82`) : `account_type: AccountType` (`enum {Asset, Liability, Revenue, Expense}` `:12-17`, PascalCase, CHECK BINARY). Classification par `account_type`, PAS le numéro.
- `projects` (`entities/project.rs:16-30`, migration `20260702000001`) : `parent_id: Option<i64>`, `archived`, `start_date`/`end_date: Option<NaiveDate>`. Repo `projects::list_by_company:20` (racines→enfants), `get_for_company:43`, `has_children:125`. Pas de fetch récursif → 2 niveaux, filtrer `parent_id = root.id`.
- `fiscal_years` : `find_covering_date` (`repositories/fiscal_years.rs:422`), `list_by_company:541` (multi-exercices pour le cumulé), `find_by_id_in_company:393`.

**Template agrégation** — `crates/kesh-report/src/income_statement.rs:55-96` (`fetch_section`) : `sign_expr` par account_type + `SELECT ... FROM accounts a INNER JOIN journal_entry_lines jel ON jel.account_id=a.id INNER JOIN journal_entries je ON je.id=jel.entry_id WHERE a.company_id=? AND a.account_type=? AND je.company_id=? AND je.fiscal_year_id=? AND je.entry_date BETWEEN ? AND ? GROUP BY a.id,... HAVING balance!=0`. **Ajouter** `AND jel.project_id IN (?,?,...)` + inclure `jel.project_id` au GROUP BY (pour la ventilation par sous-projet). Binds dynamiques pour le `IN`.
- Ligne déjà sélectionnant `project_id` scopé company : `journal_entries.rs:983-992` (`list_all_lines_by_company`).
- Signe canonique : `trial_balance.rs:63-97` `CASE WHEN a.account_type IN ('Asset','Expense') THEN debit-credit ELSE credit-debit`.
- Rollup en Rust (BTreeMap) : précédent `vat_report.rs:108-117`.

**PDF** — `crates/kesh-report/src/pdf.rs` : `PdfBuilder:217` (`new`, `write_line:277`, `ensure_space_for_row:270`, `draw_footers:302`, `finalize:317`), `draw_header:329`, `draw_account_row:387`, `draw_totals_footer:416`, `format_swiss_amount:160`, `format_swiss_date:187`. Labels : `PdfContext:55` + `SectionLabels:73` + `fr_ch_defaults:107` ; **précédent labels dédiés `VatPdfLabels:879`** (passer une struct `ProjectReportPdfLabels` en 3e arg). printpdf 0.7, Helvetica builtin (pas de TTF).

**CSV** — `crates/kesh-report/src/csv.rs` : `make_writer:29` (`;` + CRLF), `write_bom:47` (BOM `[0xEF,0xBB,0xBF]`), `format_amount_iso:38` (`{:.2}`). Modèle `render_income_statement_csv:138`. Wrap handler `render_csv_to_vec` (`reports.rs:715`).

**Routes** — `crates/kesh-api/src/routes/reports.rs` : `ReportQuery:51`, `get_income_statement:168`, `export_income_statement:339`, `ReportPeriod::resolve` (`period.rs:78`), `load_pdf_context:662`, `build_export_response_with_locale:730`, `build_filename:757`, `resolve_type_slug:695`, audit `emit_report_audit:807`/`emit_report_export_audit:863`. Enregistrement `lib.rs:594-636` (⚠️ avant le `;`, warning `:611`). `projects` router existe déjà (`routes/projects.rs`).

**Frontend** — `frontend/src/routes/(app)/reports/+page.svelte` (tabs `:240`, DTO state `:54`, `generate():102`, render View `:329`, `exportReport:191`, `canExport:168`). `reports.api.ts` (`getIncomeStatement:37`, `buildQuery:18`, `getReportExportUrl:145`, `buildExportFilename:176`, `TYPE_SLUGS_FALLBACK:192`, `downloadReport:235`, `triggerDownload:256`). Views existants = modèles (`IncomeStatementView.svelte`). Projets : `projects.api.ts::listProjects:16`. i18n filename : `crates/kesh-i18n/locales/fr-CH/messages.ftl:936` (`reports-filename-*`) + 4 autres locales + fallback map.

### Conventions projet

- Multi-tenant scopé `company_id` du JWT (IDOR) ; DTOs camelCase ; `AppError` 4xx typée (jamais 500 sur erreur métier) ; `ReportError` → mapping HTTP existant ; i18n fallback FR inline frontend ; PDF labels FR-CH par défaut (i18n PDF différée v0.2, cohérent existants) ; pas de `unreachable!()` sur match account_type ouvert (défaut → `ReportError`). Commit par étape BMAD, branche `story/19-6-rapports-analytiques` (créée depuis main, indépendante des PR 19-5 #206).
- **Pas de migration** (lecture seule sur schéma existant). Rien à faire idempotence-audit / breaking policy / backup manifeste.

### Pièges connus

1. **`IN (...)` dynamique** : construire les placeholders `?,?,…` selon `scope.project_ids.len()` et binder dans l'ordre (comme `validate_taggable_in_tx`). Scope jamais vide (au moins la racine).
2. **Mode cumulé** : NE PAS filtrer `fiscal_year_id`. Borne basse = `project.start_date` (si `Some`), borne haute = `project.end_date.unwrap_or(today)`. `today` via un paramètre (kesh-report n'a pas d'horloge — passer `NaiveDate` depuis le handler, cohérent no-`Date::now()` en lib).
3. **Rollup sous-projet ciblé** : si le `project_id` demandé a `parent_id = Some(...)` (c'est un sous-projet), scope = lui seul (pas de remontée au parent). Rapport racine = racine + enfants.
4. **Lignes de contrepartie taguées** : le tag document-level (19-3/19-4/19-5) tague TOUTES les lignes (charge + TVA + banque/dette). Le filtre `account_type = 'Expense'` (Dépenses) exclut mécaniquement banque/dette/TVA. Vérifier en test (AC16a). Pour le Rendement, **DC8 FIGÉ** : coût investi = `Expense` + `Asset` **hors classe 10xx** (`a.number NOT LIKE '10%'`) → la ligne banque taguée d'un règlement n'est PAS comptée. Test AC16c avec une ligne banque taguée pour prouver l'exclusion.
5. **E2E** : scoper les sélecteurs option (collision select projet ↔ autres selects), `KESH_COOKIE_SECURE=false` local.

### References

- [Source: epic-19-analytique-projet-design.md §4 (reporting), §2 D2/D3/D4]
- [Source: 19-2..19-5 story files — helper validate_taggable_in_tx, dimension project_id, propagation]
- [Source: crates/kesh-report/src/income_statement.rs — template agrégation par account_type]
- [Source: crates/kesh-api/src/routes/reports.rs — pattern handler JSON + export PDF/CSV]

## Questions pour Guy — TRANCHÉES (2026-07-04)

1. **DC5 — TVA dans Dépenses** → **HT** (somme des lignes Expense). ✅
2. **DC8 — Coût investi & trésorerie** → **Exclure la trésorerie (classe 10xx)** : coût investi = `Expense` + `Asset` `NOT LIKE '10%'`. ✅
3. **Drill-down** → **écritures inline expandables** (AC4), pas de lien journal (hors scope v1). ✅
