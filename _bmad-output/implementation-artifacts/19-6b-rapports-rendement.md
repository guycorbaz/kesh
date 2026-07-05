# Story 19.6b : Rapport « Rendement par projet »

Status: ready-for-dev

<!-- Sous-story 2/2 de l'umbrella 19-6 (SPLIT). Réutilise la fondation
     project_report (resolve_scope, ProjectPeriodMode) posée par 19-6a. Ajoute
     le 2ᵉ rapport « Rendement par projet » : coût investi vs revenus → résultat
     net + rendement %. RÉFÉRENCE : 19-6-rapports-analytiques.md + 19-6a. -->

## Story

As a comptable/indépendant PME utilisant Kesh,
I want un rapport **Rendement par projet** — coût investi (charges + actifs immobilisés), revenus, résultat net et rendement % — par sous-projet avec rollup, en mode exercice ou cumulé, exportable PDF/CSV,
so that j'analyse la performance de mes projets d'investissement à partir des écritures déjà taguées.

## Contexte & source

- **Umbrella 19-6** + **19-6a** (fondation mergée #207, sur main) : `project_report.rs` fournit `resolve_scope`, `ProjectReportScope`, `ProjectPeriodMode` (+ `je_filter`, `in_placeholders`, `bind_period`, `ordered_projects`), `ProjectInfo`, `ReportError::ProjectNotFound`. **Réutiliser tel quel.**
- Cette sous-story ajoute uniquement l'agrégation « rendement » + DTOs + PDF/CSV + 1 endpoint + onglet frontend. Miroir structurel de 19-6a.

## Décisions applicables (figées Guy 2026-07-04)

- **DC1** — classification par `accounts.account_type`. Signes : `Expense`/`Asset` = `debit − credit` ; `Revenue` = `credit − debit`.
- **DC3** — rollup 2 niveaux (racine + sous-projets ; sous-projet ciblé = lui seul). Sections par sous-projet + total.
- **DC4** — 2 modes exercice/cumulé (identiques 19-6a, même `ProjectPeriodMode`).
- **DC8** — **Coût investi** = Σ `Expense` + Σ `Asset` **dont `number NOT LIKE '10%'`** (exclut trésorerie banque/caisse taguée par propagation). **Revenus** = Σ `Revenue`. **Résultat net** = Revenus − Charges(`Expense`). **Rendement %** = `revenus / coût_investi × 100` si coût_investi > 0, sinon `null` (« — »).
- **DC7** — lecture, tous rôles, scopé company JWT.

## Acceptance Criteria

1. `generate_project_return(pool, company_id, scope, mode) -> Result<ProjectReturnReport, ReportError>` (`project_report.rs`) : une requête par scope avec agrégats **conditionnels par `account_type`** (SUM CASE) groupés par `jel.project_id`. Réutilise `mode.je_filter()` + `IN(scope.project_ids)` + `bind_period`. Formule DC8. DTOs camelCase :
   - `ProjectReturnReport { report_type: "project-return", project: ProjectInfo, mode, period_label, sections: Vec<ProjectReturnSection>, totals: ProjectReturnTotals }`.
   - `ProjectReturnSection { project: ProjectInfo, is_root: bool, cout_investi: Decimal, revenus: Decimal, resultat_net: Decimal, rendement_pct: Option<Decimal> }`.
   - `ProjectReturnTotals { cout_investi, revenus, resultat_net, rendement_pct: Option<Decimal> }` (rollup racine+enfants). Sections triées racine puis sous-projets par code. Une section n'est omise que si ses 3 montants sont nuls.
2. **Rendement %** : `rendement_pct = Some(revenus / cout_investi * 100)` si `cout_investi > 0`, sinon `None`. Arrondi 2 décimales (`round_dp(2)`). Idem au niveau totals.
3. Export PDF `render_project_return_pdf(report, ctx, labels)` (`pdf.rs`, labels dédiés `ProjectReturnPdfLabels` FR-CH) : en-tête projet+période, une ligne par section (sous-projet : coût investi | revenus | résultat net | rendement%), pied de total. Re-export lib.rs.
4. Export CSV `render_project_return_csv(report, w)` (`csv.rs`) : BOM + header `["Projet","SousProjet","CoutInvesti","Revenus","ResultatNet","RendementPct"]` + ligne total. Rendement formaté `xx.xx%` ou `—`. Re-export lib.rs.
5. API `GET /api/v1/reports/project-return` (JSON) + `.../export?format=pdf|csv`. Handlers calqués sur `get_project_expenses`/`export_project_expenses` (réutilise `ProjectReportQuery`, `resolve_project_report`, `project_mode_dates`, `build_project_export_response`, audit type `"project-return"`). Routes dans `authenticated_routes` **avant le `;`**. i18n slug `reports-filename-project-return = rendement-par-projet` (4 locales) + `resolve_type_slug`.
6. Frontend : DTO `ProjectReturnDto` + sous-types (`reports.types.ts`) ; `getProjectReturn` + `getProjectReportExportUrl('project-return', …)` + `downloadProjectReport` (généraliser le type param à `'project-expenses' | 'project-return'`) ; onglet « Rendement par projet » dans `reports/+page.svelte` (réutilise les contrôles projet/mode existants — factoriser si simple, sinon dupliquer le bloc) + `ProjectReturnView.svelte` (tableau par section : coût investi / revenus / résultat net / rendement%, ligne total). `data-testid project-return-*`. i18n fallback FR.
7. Tests intégration (`crates/kesh-report/tests/project_return.rs`, `#[sqlx::test]`) : (a) projet avec Expense + Asset immobilisé (compte 1500) + Revenue + une ligne **banque 1020 taguée** → coût investi **exclut** 1020 (DC8), résultat net = revenus − charges, rendement % correct ; (b) rendement null si coût investi = 0 (que des revenus) ; (c) rollup racine+enfants ; (d) mode cumulé ; (e) scope vide → totaux 0, rendement null.
8. Tests API HTTP (`crates/kesh-api/tests/`) : JSON shape + export PDF %PDF + CSV BOM + 404 projet inconnu. Frontend : unit `getProjectReturn` + `ProjectReturnView.test.ts` (rendu + rendement %/—). E2E : onglet Rendement (peut réutiliser/étendre `reports.spec.ts` — attention tab count passera 6→7).
9. Non-régression : rapports existants (dont 19-6a Dépenses) inchangés ; pas de migration ; export souveraineté inchangé. Quality gate **Test Locally First** complet (workspace serial), exit-codes vérifiés. CHANGELOG [Non publié] : entrée Rendement par projet.

## Tasks / Subtasks

- [ ] **T1 — Agrégation Rendement** (AC: 1-2) : `generate_project_return` + DTOs (SUM CASE par account_type, DC8 exclusion 10xx) + tests intégration (7 a-e).
- [ ] **T2 — Export PDF + CSV** (AC: 3-4) : `render_project_return_pdf` (labels dédiés) + `render_project_return_csv` + re-exports.
- [ ] **T3 — API** (AC: 5) : endpoint JSON + export, i18n slug 4 locales, audit.
- [ ] **T4 — Frontend** (AC: 6) : types + api + onglet + `ProjectReturnView` + i18n.
- [ ] **T5 — Tests + gate** (AC: 7-9) : API HTTP + frontend unit/E2E + CHANGELOG + Test Locally First.

## Dev Notes

**RÉFÉRENCE** : 19-6a (`crates/kesh-report/src/project_report.rs` — structure `generate_project_expenses` à cloner ; `routes/reports.rs` — `get_project_expenses`/`export_project_expenses`/`resolve_project_report`/`build_project_export_response`/`project_mode_dates` ; `frontend reports.api.ts`/`reports.types.ts`/`+page.svelte` onglet + contrôles projet/mode ; `ProjectExpensesView.svelte`). Umbrella 19-6 pour la cartographie kesh-report complète.

**SQL Rendement** (une requête, agrégats conditionnels) :
```sql
SELECT jel.project_id AS project_id,
  COALESCE(SUM(CASE WHEN a.account_type='Expense'
              OR (a.account_type='Asset' AND a.number NOT LIKE '10%')
            THEN jel.debit - jel.credit ELSE 0 END),0) AS cout_investi,
  COALESCE(SUM(CASE WHEN a.account_type='Revenue' THEN jel.credit - jel.debit ELSE 0 END),0) AS revenus,
  COALESCE(SUM(CASE WHEN a.account_type='Expense' THEN jel.debit - jel.credit ELSE 0 END),0) AS charges
FROM accounts a
INNER JOIN journal_entry_lines jel ON jel.account_id = a.id
INNER JOIN journal_entries je ON je.id = jel.entry_id
WHERE a.company_id = ? AND je.company_id = ? AND jel.project_id IN (...) {je_filter}
GROUP BY jel.project_id
```
Puis en Rust : `resultat_net = revenus - charges`, `rendement_pct = if cout_investi > 0 { Some((revenus/cout_investi*100).round_dp(2)) } else { None }`.

**Conventions** : multi-tenant company JWT (IDOR), DTOs camelCase, `AppError` 4xx, `ReportError`→mapping HTTP, PDF labels FR-CH, i18n fallback FR frontend, pas de `Date::now()` en lib (mode cumulé reçoit `today` du handler — déjà géré par `resolve_project_report`). Pas de migration. Branche `story/19-6b-rapports-rendement` (from main post-#207). Commit par étape BMAD. `Option<Decimal>` sérialise en `null` JSON quand `None` (rendement N/A).

**Pièges** : (1) `NOT LIKE '10%'` = exclusion trésorerie DC8 — tester avec une ligne 1020 taguée. (2) division rendement : garder `cout_investi > 0` (pas `!= 0` — un coût négatif n'a pas de sens de rendement). (3) `Option<Decimal>` : le frontend affiche « — » si null. (4) tab count E2E 6→7. (5) sérialisation Decimal scale (le SQL peut retourner scale 4 ; le rendement est round_dp(2)).

## Questions

Aucune — DC8 (coût investi hors 10xx), DC5 (HT côté Dépenses, sans objet ici) figés Guy.
