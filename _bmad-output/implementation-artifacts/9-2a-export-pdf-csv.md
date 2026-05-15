# Story 9.2a: Export PDF & CSV par rapport

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a utilisateur du logiciel comptable Kesh,
I want exporter chacun des 4 rapports comptables (bilan, compte de résultat, balance des comptes, journaux) en PDF ou en CSV,
so that je puisse les partager avec mon fiduciaire, les archiver hors-ligne, ou les ré-importer dans Excel.

## Scope

Étend l'API publique stable `kesh_report::{BalanceSheet, IncomeStatement, TrialBalance, JournalReport}` livrée par Story 9-1 avec :
- 2 nouveaux modules `kesh-report::{pdf, csv}` (sérialiseurs purs, déterministes, byte-stable pour tests).
- 4 nouveaux endpoints HTTP `GET /api/v1/reports/{type}/export?format=pdf|csv` (Option B : routes séparées, pas d'extension `?format=` sur les routes JSON existantes — voir Decision §design-export-route).
- 4 boutons « Export PDF » + 4 boutons « Export CSV » dans la page `/reports` (1 paire par onglet, visible une fois le rapport généré).
- ~12 clés i18n `reports-export-*` × 4 locales (fr/de/it/en-CH).
- Audit log dédié `report.exported` (best-effort, pattern Story 9-1 `emit_report_audit`).

**Hors scope (livré par 9-2b)** : export ZIP global par table (souveraineté des données, FR68). Nouveau module `kesh-api/routes/exports.rs`, nouvelle entrée menu principal, `metadata.json` + hash SHA-256.

**Hors scope v0.1** : modèles documents personnalisables (FR81 — Epic 15), drill-down/recherche dans le PDF (FR70 — v0.2), aperçu inline avant download (UX nice-to-have v0.2).

## Acceptance Criteria

### Export PDF (FR67)

1. **Given** un utilisateur authentifié sur `/reports` avec un exercice sélectionné et un rapport (Bilan/Résultat/Balance/Journaux) généré, **When** l'utilisateur clique sur « Export PDF », **Then** le navigateur télécharge un fichier `.pdf` avec `Content-Type: application/pdf` et `Content-Disposition: attachment; filename="..."`.

2. **Given** un PDF généré, **When** ouvert dans n'importe quel lecteur PDF, **Then** les montants apparaissent au format suisse : apostrophe séparateur de milliers et point décimal (`1'234.56`), et les dates au format `dd.mm.yyyy` (FR67).

3. **Given** un PDF de bilan, **When** rendu, **Then** il contient l'en-tête (raison sociale de la company + période + exercice), les sections Actifs / Passifs / Capitaux propres avec totaux par classe de compte, et l'équation bilan vérifiée affichée en pied de page (`Total actifs = Total passifs + capitaux propres`).

4. **Given** un PDF de compte de résultat, **When** rendu, **Then** il contient l'en-tête, les sections Produits / Charges avec totaux, et le résultat net (bénéfice / perte) en pied de page.

5. **Given** un PDF de balance, **When** rendu, **Then** toutes les lignes ont 4 colonnes (Numéro, Compte, Débit, Crédit, Solde) avec totaux débit = totaux crédit en bas.

6. **Given** un PDF de journaux, **When** rendu (sans filtre `journal`), **Then** les 5 sections (Achats, Ventes, Banque, Caisse, OD) sont toujours présentes même vides — cohérent avec le DTO JSON Story 9-1 (Pass 1 ECH-09).

7. **Given** un PDF de journaux **with** filtre `journal=Ventes`, **When** rendu, **Then** seule la section Ventes apparaît, en-tête mentionne « Journal : Ventes ».

8. **Given** un rapport vide (aucune écriture dans la période), **When** export PDF, **Then** le PDF est généré quand même avec le message i18n `reports-error-no-entries-in-period` rendu au centre (cohérent avec UX-DR38 + helper `isReportEmpty` Story 9-1).

9. **Given** un dataset de référence (~1000 écritures), **When** génération PDF d'un rapport, **Then** la durée totale entre clic et début du download `< 3 secondes` (benchmark documenté en `crates/kesh-report/benches/`).

### Export CSV

10. **Given** un rapport généré, **When** l'utilisateur clique sur « Export CSV », **Then** le navigateur télécharge un fichier `.csv` avec `Content-Type: text/csv; charset=utf-8` et `Content-Disposition: attachment; filename="..."`.

11. **Given** un CSV, **When** ouvert dans Excel (CH/DE) ou LibreOffice, **Then** :
    - encodage UTF-8 avec BOM (`\u{FEFF}` en tête)
    - séparateur point-virgule `;` (PAS la virgule — convention Excel CH/DE)
    - fin de ligne `\r\n` (CRLF, RFC 4180)
    - montants au format ISO décimal point (`1234.56` — pas d'apostrophe en CSV, format machine-readable pour ré-import)
    - dates ISO 8601 (`2026-05-15` — machine-readable, pas le format affichage)
    - chaînes contenant `;`, `"`, ou retour-ligne sont entourées de `"` avec `"` interne doublé en `""` (RFC 4180).

12. **Given** un CSV de bilan, **When** rendu, **Then** colonnes : `Section;NumeroCompte;NomCompte;Solde` où `Section ∈ {Actifs, Passifs, CapitauxPropres}`. Ligne de total par section. Ligne finale `Total actifs;;;<somme>` + `Total passifs + capitaux propres;;;<somme>`.

13. **Given** un CSV de compte de résultat, **When** rendu, **Then** colonnes : `Section;NumeroCompte;NomCompte;Solde` où `Section ∈ {Produits, Charges}`. Total par section + ligne finale `ResultatNet;;;<somme>`.

14. **Given** un CSV de balance, **When** rendu, **Then** colonnes : `NumeroCompte;NomCompte;TotalDebit;TotalCredit;Solde`. Ligne finale totaux débit/crédit avec colonnes solde vide.

15. **Given** un CSV de journaux **sans** filtre, **When** rendu, **Then** colonnes : `Journal;DateEcriture;NumeroEcriture;Description;NumeroCompte;NomCompte;Debit;Credit`. Une ligne par `journal_entry_line`. Ordre par `Journal ASC, entry_date ASC, journal_entry_id ASC, line_index ASC`.

16. **Given** un CSV de journaux **with** filtre `journal=Ventes`, **When** rendu, **Then** seules les lignes du journal Ventes apparaissent, première colonne `Journal` reste populée (= "Ventes" partout).

17. **Given** un rapport vide, **When** export CSV, **Then** le CSV contient les en-têtes uniquement (1 ligne) + une ligne `# Aucune écriture dans la période` en commentaire ASCII (pas de BOM-issue).

### Frontend : intégration UI

18. **Given** la page `/reports` après génération réussie d'un rapport, **When** l'utilisateur regarde la zone de contrôles, **Then** 2 nouveaux boutons « Export PDF » et « Export CSV » apparaissent à droite du bouton « Générer » dans `ReportSelector.svelte` (ou dans le bandeau du tabpanel).

19. **Given** aucun rapport généré (avant `Générer`), **When** l'utilisateur regarde les boutons d'export, **Then** ils sont `disabled` (pattern cohérent `noFiscalYears` / `loading` Story 9-1 `ReportSelector`).

20. **Given** un exercice sans `fiscal_year` (preset `with-company-no-fy` Issue #90 ef07548), **When** l'utilisateur regarde les boutons d'export, **Then** ils sont `disabled` (cohérent AC #34 Story 9-1 — pas d'export possible sans exercice).

21. **Given** export PDF/CSV en cours, **When** l'utilisateur clique sur l'autre format, **Then** le second clic est ignoré jusqu'à fin du premier (`loading` flag partagé ou flag dédié `exporting` — au choix dev, justifier).

22. **Given** un nom de fichier suggéré au browser, **When** download déclenché, **Then** le format est `kesh-{type}-{companyShort}-{periodStart}_{periodEnd}.{ext}` où :
    - `{type}` ∈ {`bilan`, `compte-resultat`, `balance`, `journaux`} (localisé en français — i18n key `reports-filename-{type}`)
    - `{companyShort}` = slug ASCII de `company.name` (max 20 chars, fallback `company` si vide)
    - `{periodStart}` / `{periodEnd}` = `YYYY-MM-DD` (machine-readable pour tri filesystem)
    - `{ext}` = `pdf` ou `csv`
    - Exemple : `kesh-bilan-ci-test-company-2026-01-01_2026-12-31.pdf`

23. **Given** une erreur backend (500, 400, 401), **When** export échoue, **Then** le message d'erreur est affiché dans la zone d'alerte existante `errorMsg` (`+page.svelte` Story 9-1) avec format UX-DR38 — ce qui s'est passé + ce que l'utilisateur peut faire.

### Multi-tenant + sécurité

24. **Given** un utilisateur authentifié de company A, **When** il appelle `GET /api/v1/reports/balance-sheet/export?fiscalYearId=<id_de_company_B>`, **Then** 404 `FISCAL_YEAR_NOT_FOUND` (cohérent Story 9-1 — `ReportPeriod::resolve` joint sur `company_id`). Aucune fuite de données cross-tenant.

25. **Given** un utilisateur Consultation, **When** export PDF/CSV, **Then** réussit (lecture seule, cohérent avec rôles autorisés sur `GET /api/v1/reports/*` Story 9-1).

26. **Given** un utilisateur non authentifié, **When** export PDF/CSV, **Then** 401 (middleware auth standard).

27. **Given** le serveur backend, **When** export `?format=invalid` ou param manquant, **Then** 400 `VALIDATION_ERROR` JSON avec message listant les formats valides (`pdf`, `csv`).

### Audit + observabilité

28. **Given** un export PDF ou CSV réussi, **When** réponse 200 retournée, **Then** une ligne `audit_log` est insérée avec :
    - `action = 'report.exported'`
    - `entity_type = 'report'`
    - `entity_id = AUDIT_ENTITY_ID_NONE` (cohérent Story 9-1 — pas d'entité 1:1)
    - `details_json` incluant `reportType`, `format`, `fiscalYearId`, `periodStart`, `periodEnd`, `journalFilter` (si applicable)
    - Pattern best-effort : INSERT échec → `warn!` log + retour 200 (ne JAMAIS faire échouer le download — Story 9-1 ECH-15).

29. **Given** export en cours, **When** monitoring observe les logs structurés, **Then** un span tracing `report_export` est émis avec attributs `report_type`, `format`, `byte_size`, `duration_ms`.

### Performance + limites

30. **Given** un dataset large (10k écritures dans un journal), **When** export CSV journaux, **Then** la durée `< 5 secondes` (RAM bornée — streaming via `csv::Writer` sur réponse Axum, PAS d'allocation `Vec<String>` géante).

31. **Given** un dataset large, **When** export PDF, **Then** taille du fichier `< 5 MB` pour 10k écritures dans un journal (binding implicite : limite de pagination si nécessaire — voir §pagination-pdf §Limitations).

### Tests

32. **Given** la story est implémentée, **When** suite de tests exécutée, **Then** **≥ 16 tests E2E HTTP** dans `crates/kesh-api/tests/reports_export_e2e.rs` couvrent : (a) 4 endpoints PDF × 200 binary content, (b) 4 endpoints CSV × 200 text/csv content, (c) format invalid 400, (d) multi-tenant 404, (e) FY out of bounds 400, (f) rapport vide PDF + CSV success path, (g) auth 401, (h) RBAC Consultation 200.

33. **Given** la story est implémentée, **When** tests unit `kesh-report` exécutés, **Then** **≥ 12 tests** valident : (a) PDF byte signature commence par `%PDF-1.` (4 rapports × 1 = 4 tests), (b) CSV BOM + séparateur + CRLF (4 tests), (c) CSV escaping RFC 4180 (1 test), (d) format suisse montant + date dans PDF via grep regex sur le texte décompressé (3 tests).

34. **Given** la story est implémentée, **When** Vitest exécuté, **Then** **≥ 3 tests** sur `reports.api.ts` : (a) `getReportPdf(query)` retourne un `Blob`, (b) construction filename `kesh-bilan-...-2026-01-01_2026-12-31.pdf`, (c) erreur backend → erreur formatée UX-DR38.

35. **Given** la story est implémentée, **When** Playwright exécuté, **Then** **1 scénario actif** `reports-export-pdf.spec.ts` télécharge un PDF + assert byte signature `%PDF-1.` + filename pattern.

## Tasks / Subtasks

- [ ] **T1** Ajouter dépendances `printpdf` + `csv` à `crates/kesh-report/Cargo.toml` (AC: #1, #10)
  - [ ] T1.1 `printpdf = "0.7"` (cohérent `kesh-qrbill`, ne PAS upgrader sans coordination)
  - [ ] T1.2 `csv = "1.3"` (sérialiseur standard Rust, RFC 4180 compliant)
  - [ ] T1.3 Vérifier `cargo build -p kesh-report` clean après ajout

- [ ] **T2** Créer `crates/kesh-report/src/pdf.rs` — sérialiseur PDF pur (AC: #1-#9)
  - [ ] T2.1 Signature publique `pub fn render_balance_sheet_pdf(bs: &BalanceSheet, ctx: &PdfContext) -> Result<Vec<u8>, ReportError>` (× 4 fonctions, une par rapport)
  - [ ] T2.2 `PdfContext { company_name: String, locale: &str }` — porte les données i18n non incluses dans les DTOs (locale courante pour les libellés `Actifs`/`Aktiven`/etc.)
  - [ ] T2.3 Helpers privés `draw_header`, `draw_table_row`, `draw_totals_footer` (DRY — réutilisés par les 4 fonctions)
  - [ ] T2.4 Format suisse montants via helper local `format_swiss_amount(decimal: Decimal) -> String` (apostrophe séparateur + point décimal) — vérifier cohérence avec `kesh-i18n::format_amount` (DD-13, à confirmer en spec validate)
  - [ ] T2.5 Format dates `dd.mm.yyyy` via `NaiveDate::format("%d.%m.%Y")`
  - [ ] T2.6 Cas dégénéré rapport vide : afficher message centré, ne pas crasher (AC #8)

- [ ] **T3** Créer `crates/kesh-report/src/csv.rs` — sérialiseur CSV pur (AC: #10-#17)
  - [ ] T3.1 Signature publique `pub fn render_balance_sheet_csv<W: Write>(bs: &BalanceSheet, writer: W) -> Result<(), ReportError>` (× 4 fonctions, streaming-friendly)
  - [ ] T3.2 BOM en tête : `writer.write_all(b"\xef\xbb\xbf")?;` avant d'instancier `csv::WriterBuilder`
  - [ ] T3.3 `csv::WriterBuilder::new().delimiter(b';').terminator(csv::Terminator::CRLF).from_writer(writer)`
  - [ ] T3.4 Tests RFC 4180 escaping : un nom de compte contenant `;` doit être entouré de `"..."` (AC #11)
  - [ ] T3.5 Cas rapport vide : écrire seulement la ligne d'en-tête + commentaire ASCII (AC #17)

- [ ] **T4** Étendre `crates/kesh-report/src/lib.rs` exports + créer benchmark (AC: #1, #10, #9, #30, #31)
  - [ ] T4.1 `pub mod csv;` + `pub mod pdf;`
  - [ ] T4.2 Re-exports : `pub use pdf::{render_balance_sheet_pdf, render_income_statement_pdf, render_trial_balance_pdf, render_journal_report_pdf, PdfContext};` + symétrique pour CSV
  - [ ] T4.3 Créer `crates/kesh-report/benches/export.rs` avec criterion (à ajouter en dev-dep `criterion = "0.5"`) — bench les 4 rapports PDF + 4 CSV sur fixture 1000 écritures (AC #9 + #30 + #31)
  - [ ] T4.4 Bench documenté dans `Cargo.toml` : `[[bench]] name = "export" harness = false`

- [ ] **T5** Créer 4 nouveaux endpoints `kesh-api/src/routes/reports.rs` (AC: #1, #10, #22-#28)
  - [ ] T5.1 Ajouter `pub async fn export_balance_sheet`, `export_income_statement`, `export_trial_balance`, `export_journal_report` — signature `(State<AppState>, Extension<CurrentUser>, Query<ExportQuery>) -> Result<Response, AppError>` (réponse binaire, pas `Json<>`).
  - [ ] T5.2 Définir `struct ExportQuery { fiscal_year_id: i64, period_start: Option<NaiveDate>, period_end: Option<NaiveDate>, journal: Option<Journal>, format: ExportFormat }` avec `#[serde(rename_all = "camelCase")]`. `enum ExportFormat { Pdf, Csv }` avec `#[serde(rename_all = "lowercase")]`. Le champ `journal` reste `Option<Journal>` global (ignoré par les 3 premiers rapports — pattern Story 9-1 `JournalReportQuery`).
  - [ ] T5.3 Construire `Response` :
    - PDF : `axum::response::Response::builder().header(CONTENT_TYPE, "application/pdf").header(CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\"")).body(Body::from(pdf_bytes)).unwrap()`
    - CSV : idem avec `text/csv; charset=utf-8` + extension `.csv`. Body construit dans un `Vec<u8>` (le streaming Axum vers `csv::Writer` est possible mais ajoute de la complexité — accepté pour v0.1, voir L5)
  - [ ] T5.4 Filename helper `fn build_filename(report_type: &str, company_name: &str, period: &ReportPeriod, ext: &str) -> String` — slug ASCII via crate `slug = "0.1"` (à ajouter à kesh-api Cargo.toml), max 20 chars, fallback `"company"`.
  - [ ] T5.5 Audit log `report.exported` via helper modifié `emit_report_audit` (ajouter param `format: Option<&str>` à la signature existante, ou créer `emit_report_export_audit` séparé — au choix dev, justifier). Pattern best-effort identique Story 9-1.
  - [ ] T5.6 Validation `fiscal_year_id > 0` réutilisée (`validate_fiscal_year_id` helper Story 9-1).
  - [ ] T5.7 Multi-tenant via `ReportPeriod::resolve(&state.pool, current_user.company_id, ...)` — identique Story 9-1 (AC #24).

- [ ] **T6** Mount routes dans `crates/kesh-api/src/lib.rs` (AC: #1, #10, #26)
  - [ ] T6.1 4 nouvelles routes après celles de Story 9-1 ligne ~373 :
    - `.route("/api/v1/reports/balance-sheet/export", get(routes::reports::export_balance_sheet))`
    - `.route("/api/v1/reports/income-statement/export", get(routes::reports::export_income_statement))`
    - `.route("/api/v1/reports/trial-balance/export", get(routes::reports::export_trial_balance))`
    - `.route("/api/v1/reports/journals/export", get(routes::reports::export_journal_report))`
  - [ ] T6.2 Routes dans `authenticated_routes` (même bloc que Story 9-1) — auth middleware déjà appliqué.

- [ ] **T7** Étendre frontend `frontend/src/lib/features/reports/` (AC: #18-#23)
  - [ ] T7.1 Ajouter dans `reports.api.ts` :
    - `getReportExportUrl(type: ReportType, query: ReportQuery, format: 'pdf' | 'csv', journal?: string): string` (construit l'URL avec query string).
    - `downloadReport(type: ReportType, query: ReportQuery, format: 'pdf' | 'csv', filename: string): Promise<void>` (déclenche le download via fetch blob + lien `<a download>` éphémère, gère 401 redirect via `apiClient`).
    - `buildExportFilename(type: ReportType, companyName: string, period: { start: string; end: string }, format: 'pdf' | 'csv'): string` — implémente le pattern AC #22 + slug ASCII via regex `[^a-z0-9-]/g`.
  - [ ] T7.2 Modifier `ReportSelector.svelte` : ajouter 2 boutons `Export PDF` + `Export CSV` à droite du bouton « Générer ». Props : `onExportPdf: () => void` + `onExportCsv: () => void` + `canExport: boolean` (vrai si un rapport est généré ET pas en cours d'export).
  - [ ] T7.3 Modifier `+page.svelte` : ajouter state `companyName` (à charger via `/api/v1/companies/current` ou via `data.company` dans `+page.ts` — vérifier si endpoint existe Story 1-7+ ; sinon, fallback `'company'`).
  - [ ] T7.4 Handler `async function exportPdf()` + `exportCsv()` : appelle `downloadReport(activeTab, query, format, buildExportFilename(...))`, gère erreurs via `formatError` existant Story 9-1.
  - [ ] T7.5 Garde `canExport` : `false` si aucun rapport généré, `false` si `noFiscalYears`, `false` si `loading || exporting`.

- [ ] **T8** Ajouter clés i18n × 4 locales (fr/de/it/en-CH) — AC: #22, #23, #18-#21
  - [ ] T8.1 Clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` :
    - `reports-export-pdf-button = Export PDF`
    - `reports-export-csv-button = Export CSV`
    - `reports-export-loading = Génération du fichier…`
    - `reports-export-error-generic = Impossible d'exporter le rapport. Vérifiez votre connexion et réessayez.`
    - `reports-filename-balance-sheet = bilan`
    - `reports-filename-income-statement = compte-resultat`
    - `reports-filename-trial-balance = balance`
    - `reports-filename-journals = journaux`
    - `reports-pdf-header-period = Période`
    - `reports-pdf-empty-message = Aucune écriture dans la période sélectionnée.`
    - `reports-csv-comment-empty = # Aucune écriture dans la période`
  - [ ] T8.2 Idem pour `de-CH`, `it-CH`, `en-CH` (traductions de base, validation native v0.2 — L4 héritée)
  - [ ] T8.3 `npm run lint-i18n-ownership` PASS — toutes les clés `reports-export-*` et `reports-filename-*` appartiennent à `lib/features/reports/`
  - [ ] T8.4 `cargo run -p kesh-i18n --bin validate-locale-coverage` (si existant) — sinon validation manuelle clés présentes dans les 4 locales.

- [ ] **T9** Tests E2E HTTP `crates/kesh-api/tests/reports_export_e2e.rs` (AC: #32)
  - [ ] T9.1 16 tests minimum (voir AC #32 décomposition)
  - [ ] T9.2 Pattern fixture identique `reports_e2e.rs` Story 9-1 : `seed_accounting_company` + écritures via SQL bypass ou helper `kesh-db::test_fixtures::*` (pas de nouveau preset endpoint — réutilise `with-company` pour tests positifs et `with-company-no-fy` pour test « pas de fiscal_year »).
  - [ ] T9.3 Assertions content-type + content-disposition + premiers bytes pour PDF (`assert!(body.starts_with(b"%PDF-1."))`) et CSV (`assert_eq!(&body[..3], b"\xef\xbb\xbf")`).

- [ ] **T10** Tests unit `crates/kesh-report/src/{pdf,csv}.rs` + benchmark (AC: #33, #9, #30)
  - [ ] T10.1 12 tests unit minimum (voir AC #33 décomposition).
  - [ ] T10.2 Benchmark criterion `crates/kesh-report/benches/export.rs` exécuté localement avant push — résultats commités en commentaire dans le commit ou dans le story file `Completion Notes`.

- [ ] **T11** Tests frontend Vitest `frontend/src/lib/features/reports/reports.api.test.ts` (AC: #34)
  - [ ] T11.1 3 tests minimum :
    - `buildExportFilename` produit la string attendue pour les 4 types + slug edge cases (accents, espaces, longueur >20).
    - `downloadReport` mock `fetch` + assert appel `apiClient` correct + Blob retourné.
    - `downloadReport` rejette quand status 500 + message formaté.

- [ ] **T12** Playwright `frontend/tests/e2e/reports-export-pdf.spec.ts` (AC: #35)
  - [ ] T12.1 1 scénario : login → /reports → générer Bilan → cliquer « Export PDF » → assert download event Playwright + filename pattern + premier byte `%PDF-1.`
  - [ ] T12.2 Pas d'a11y test sur le bouton export (AC #18-#21 vérifiés via Vitest et tests E2E HTTP) — KF-027 #91 hors scope ne bloque pas.

- [ ] **T13** CI green + Test Locally First avant push (règle CLAUDE.md)
  - [ ] T13.1 `cargo fmt --all -- --check` clean
  - [ ] T13.2 `cargo build --workspace --all-targets` clean
  - [ ] T13.3 `cargo clippy --workspace --all-targets -- -D warnings` clean
  - [ ] T13.4 `cargo test --workspace -j1 -- --test-threads=1` — 100% pass sur les nouveaux tests + 0 régression sur Story 9-1 (`reports_e2e.rs` 28 tests + `report_aggregates.rs` 7 tests). Régressions résiduelles pré-existantes `config::tests::*` 20/24 documentées dans Completion Notes.
  - [ ] T13.5 `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` — clean (0 errors).
  - [ ] T13.6 Playwright `npx playwright test reports.spec.ts reports-export-pdf.spec.ts` — green (T12.4 Story 9-1 + nouveau scénario 9-2a). Note régressions a11y KF-027 #91 pré-existantes hors scope.

## Dev Notes

### Décisions de conception verrouillées

#### Decision §design-export-route — Routes séparées `/{type}/export`

**Choix v0.1** : 4 nouvelles routes `GET /api/v1/reports/{type}/export?format=pdf|csv` (Option B).

Rejeté : `GET /api/v1/reports/{type}?format=pdf|csv|json` (Option A, suggéré dans epic-9.md initial) car :
- Le type de réponse varie (JSON vs binaire) selon query param → `axum::response::Response` au lieu de `Json<Dto>` → impossible de typer le handler proprement avec `Result<Json<...>, AppError>` Story 9-1 pattern.
- Tests E2E plus simples avec routes séparées (un test = un content-type attendu, pas de matrice format × type).
- Frontend séparation propre : `getBalanceSheet` (JSON view) reste découplé de `getReportExportUrl` (download).

Trade-off accepté : duplication des handlers (4 × 2 = 8 fonctions au lieu de 4) — mitigé par helpers privés DRY (`build_filename`, `emit_report_export_audit`).

#### Decision §pdf-lib — printpdf 0.7

`kesh-qrbill::pdf.rs` utilise `printpdf 0.7` depuis Story 5-3 → réutilisé pour cohérence et réduction de la surface de dépendances workspace. Pas de migration vers `printpdf 0.8` ou alternative (`weasyprint`, `tectonic`) avant Epic 15 v0.2 (modèles documents personnalisables FR81).

Pattern à reproduire :
```rust
use printpdf::{PdfDocument, BuiltinFont, Mm, Color, Rgb, Point};
let (doc, page1, layer1) = PdfDocument::new("Bilan", Mm(210.0), Mm(297.0), "Layer 1"); // A4 portrait
let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
let current_layer = doc.get_page(page1).get_layer(layer1);
// ... draw operations
let bytes = doc.save_to_bytes()?;
```

#### Decision §csv-lib — `csv 1.3`

Standard Rust ecosystem, RFC 4180 compliant, support natif de `WriterBuilder::delimiter` + `terminator` + `quoting`. Pas de fork ad-hoc.

#### Decision §swiss-amount-format — helper local `kesh-report::pdf::format_swiss_amount`

Pas de dépendance directe sur `kesh-i18n` pour le formatage PDF — éviter le couplage circulaire et garder `kesh-report` testable en isolation (DD-14 pattern `kesh-qrbill`). Le helper réplique le format apostrophe + point décimal documenté dans `kesh-i18n::format_amount`. Test unit garantit la cohérence.

À confirmer en spec validate : Q5 — fauder un partage de helper via une nouvelle crate `kesh-format` ou accepter la mini-duplication ? Recommandation auteur : accepter la duplication v0.1, factoriser en Epic 15 si plusieurs crates dupliquent.

#### Decision §audit-action — `report.exported`

Distinct de `report.generated` (Story 9-1) pour séparer dans les requêtes audit :
- `report.generated` : génération JSON (view dans UI)
- `report.exported` : téléchargement fichier (PDF/CSV)

`details_json` inclut `format` discriminant.

#### Decision §filename-slug — crate `slug 0.1` ou regex inline ?

Préférence auteur : regex inline TypeScript-side seulement (`name.toLowerCase().normalize('NFD').replace(/[̀-ͯ]/g, '').replace(/[^a-z0-9-]/g, '-').slice(0, 20)`) — pas de nouvelle dépendance Rust.

Côté backend, le `Content-Disposition` filename utilise UTF-8 encoding RFC 5987 (`filename*=UTF-8''<percent-encoded>`) — pas besoin de slug ASCII strict. Test E2E HTTP assert le header est bien formé.

Si dépendance crate `slug` jugée nécessaire en review : `slug = "0.1"` (8K downloads/jour, pure Rust, MIT).

### Architecture compliance (architecture.md §17 + §11)

- `kesh-report` crate interne (decision #12) — étendue avec 2 modules `{pdf, csv}`. **Ne pas** déplacer `pdf.rs` vers `kesh-qrbill` (qrbill a un scope publishable indépendant DD-14).
- `kesh-i18n` transverse (decision #13) — utilisée par les libellés UI (`reports-export-*`), mais **PAS** par les sérialiseurs PDF/CSV (cf. §swiss-amount-format).
- Workspace Cargo (§11) — nouvelles deps `printpdf 0.7` (déjà workspace via qrbill) + `csv 1.3` (nouvelle).
- Multi-tenant `company_id` (Story 7-1 / KF-002) — pattern `current_user.company_id` strict, jamais bypass.

### Library / framework requirements

| Item | Version | Source | Justification |
|---|---|---|---|
| `printpdf` | 0.7 | `kesh-qrbill` precedent | Cohérence workspace, pas de churn dépendances v0.1 |
| `csv` | 1.3 | Nouvelle dep `kesh-report` | RFC 4180 compliant, standard Rust |
| `criterion` (dev) | 0.5 | Nouvelle dev-dep `kesh-report` | Benchmark `< 3s` AC #9 reproductible |
| `slug` | 0.1 (optionnel) | Nouvelle dep `kesh-api` si nécessaire | Voir §filename-slug |

### File structure

**Nouveaux fichiers :**
- `crates/kesh-report/src/pdf.rs` (~400 lignes, 4 fonctions publiques + helpers privés)
- `crates/kesh-report/src/csv.rs` (~250 lignes, 4 fonctions publiques + helpers privés)
- `crates/kesh-report/benches/export.rs` (~80 lignes, criterion)
- `crates/kesh-api/tests/reports_export_e2e.rs` (~400 lignes, 16 tests)
- `frontend/tests/e2e/reports-export-pdf.spec.ts` (~50 lignes)
- `frontend/src/lib/features/reports/reports.api.test.ts` (~80 lignes Vitest)

**Fichiers UPDATE (existants, ne PAS recréer) :**

- `crates/kesh-report/src/lib.rs` (~30 lignes existantes) — ajouter `pub mod pdf;` + `pub mod csv;` + re-exports
  - **Préserver** : tous les exports Story 9-1 (`BalanceSheet`, `IncomeStatement`, `TrialBalance`, `JournalReport`, `generate_*`, `ReportPeriod`)
  - **Modifier** : ajouter exports `render_*_pdf`, `render_*_csv`, `PdfContext`
- `crates/kesh-report/Cargo.toml` — ajouter `printpdf`, `csv`, `criterion` dev-dep, section `[[bench]]`
- `crates/kesh-api/src/routes/reports.rs` (~255 lignes existantes) — ajouter 4 handlers `export_*` + helpers privés
  - **Préserver** : `get_balance_sheet`, `get_income_statement`, `get_trial_balance`, `get_journal_report`, `validate_fiscal_year_id`, `emit_report_audit`, `ReportQuery`, `JournalReportQuery`
  - **Modifier** : ajouter `ExportQuery`, `ExportFormat`, 4 handlers `export_*`, helpers `build_filename` + audit
- `crates/kesh-api/src/lib.rs` ligne ~373 — 4 nouvelles routes après celles Story 9-1
- `frontend/src/lib/features/reports/reports.api.ts` (~117 lignes existantes) — ajouter `getReportExportUrl`, `downloadReport`, `buildExportFilename`
  - **Préserver** : `getBalanceSheet`, `getIncomeStatement`, `getTrialBalance`, `getJournalReport`, `isReportEmpty`, `formatSwissDate`, `formatReportAmount`, helpers `buildQuery`
- `frontend/src/lib/features/reports/ReportSelector.svelte` — ajouter 2 boutons + 2 props
- `frontend/src/routes/(app)/reports/+page.svelte` (~218 lignes existantes) — ajouter handlers `exportPdf`/`exportCsv` + state `exporting` + bind `onExportPdf`/`onExportCsv`
- 4 fichiers `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — ajouter 11 clés `reports-export-*` + `reports-filename-*`

### Testing standards

- **Tests d'intégration Rust** : `crates/kesh-api/tests/reports_export_e2e.rs` — pattern `reqwest` + `spawn_app` identique `reports_e2e.rs` Story 9-1. Helper `assert_pdf_response` + `assert_csv_response` à factoriser dans le fichier (pas dans `kesh-db::test_fixtures` — scope test seulement).
- **Tests unit Rust** : `crates/kesh-report/src/{pdf,csv}.rs` modules `#[cfg(test)] mod tests` en bas de chaque fichier. Fixtures `BalanceSheet`/`IncomeStatement`/etc. construites en code (pas de DB).
- **Tests frontend Vitest** : `frontend/src/lib/features/reports/reports.api.test.ts` (nouveau) — mock `fetch` via `vi.mock`.
- **Playwright** : `frontend/tests/e2e/reports-export-pdf.spec.ts` — pattern `await page.waitForEvent('download')` + `download.suggestedFilename()` + `download.path()` + `fs.readFile`.
- **Benchmarks criterion** : `crates/kesh-report/benches/export.rs` — pas en CI (compute coût), exécuté localement avant push, résultat commité en Completion Notes.

### Previous Story Intelligence (Story 9-1)

**Patterns à réutiliser** :
- `emit_report_audit` (best-effort, `tx.begin/insert_in_tx/commit`, `warn!` on error) — étendre avec param `format: &str` OU créer `emit_report_export_audit` séparé.
- `ReportPeriod::resolve(&pool, company_id, fiscal_year_id, period_start, period_end)` — réutiliser tel quel, retourne `ReportPeriod { start_date, end_date }` après vérification fiscal_year scoping + bornes.
- `validate_fiscal_year_id(i64) -> Result<(), AppError>` — réutiliser tel quel.
- Pattern `Query<ReportQuery>` + `Extension<CurrentUser>` + `State<AppState>` — identique pour les 4 nouveaux handlers.
- Multi-tenant via `current_user.company_id` partout, jamais via query param.

**Helpers publics API stable consommée** (lib.rs Story 9-1) :
- `BalanceSheet { period: ReportPeriod, assets: Vec<AccountBalance>, liabilities: Vec<AccountBalance>, equity_result: Decimal, ... }`
- `IncomeStatement { period, revenues: Vec<AccountBalance>, expenses: Vec<AccountBalance>, net_result: Decimal, ... }`
- `TrialBalance { period, rows: Vec<TrialBalanceRow> }`
- `JournalReport { period, journals: Vec<JournalSection>, journal_filter: Option<Journal> }`

Tous `Serialize` camelCase. Champs nommés stables — Story 9-2a peut les binder par name, pas par position.

**Patterns frontend à reproduire** :
- `+page.svelte` Story 9-1 utilise `$state` + `$effect` Svelte 5 runes.
- `formatError(err)` helper avec `isApiError` — réutiliser pour les erreurs d'export.
- ARIA tabs pattern avec `handleTabKeydown` — préserver, pas de changement.

**Régressions résiduelles pré-existantes documentées Story 9-1** (à hériter sans rouvrir dans 9-2a Completion Notes) :
- `kesh-api::config::tests::*` 20/24 fail local `.env` + `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true` collision.
- KF-027 #91 a11y `#bits-c1` DropdownMenu pré-existant — pas de scope 9-2a sauf si T7 introduit un nouveau DropdownMenu (très improbable, pattern boutons inline).

### Git intelligence (5 derniers commits sur main)

| Commit | Title | Pertinence 9-2a |
|---|---|---|
| `ef07548` | chore(test-infra): preset `with-company-no-fy` for AC #34 E2E (#92) | AC #20 — réutilise preset pour test « pas de fiscal_year empêche export » |
| `6495731` | Story 9-1: Rapports comptables (#89) | Foundation totale — API stable `kesh_report::*` consommée par 9-2a |
| `b3331dc` | review(9-1): Pass 2 Haiku 4.5 — STOP cycle convergence 0>LOW | Patterns review codifiés — préfigurer ground-truth grep pour patches 9-2a |
| `25b1b5a` | review(9-1): Pass 1 code review Sonnet 4.6 — 24 patches | 24 patches Sonnet — pattern à respecter (notamment ECH-15 audit best-effort, ECH-09 sections fixes journaux) |
| `425eae5` | feat(9-1): T12+T13 — Playwright + sprint-status review | Pattern Playwright + CI workflow déjà tunés pour Story 9-1 — réutilisables |

### Limitations connues v0.1

| # | Limite | Raison | Tracking dette |
|---|---|---|---|
| L1 | Pas d'aperçu inline avant download (PDF preview iframe ou similaire) | Hors scope v0.1, complexité UI vs valeur faible | v0.2 si demande utilisateur |
| L2 | Pas de personnalisation PDF (logo company, en-tête custom, fonts custom) | FR81 reporté Epic 15 | Epic 15 v0.2 |
| L3 | PDF font = Helvetica builtin (pas de support glyphs étendus) | Cohérent kesh-qrbill, pas d'embedding TTF v0.1 | Epic 15 si glyphs spéciaux nécessaires |
| L4 | Traductions DE/IT/EN basiques (machine ou auteur, pas natif speaker review) | Hérité Story 9-1 L4 | v0.2 review native speakers |
| L5 | CSV body en `Vec<u8>` en RAM (pas de streaming Axum body) | Simplicité v0.1, dataset référence ~1000 écritures = ~200 KB max | v0.2 si dataset très large observed |
| L6 | Pas de pagination PDF pour gros datasets (10k+ écritures dans un journal) | Mitigé par AC #31 (5 MB max sur 10k écritures, acceptable) ; pagination ajoutée si dépassement observé | Si rapport > 5 MB ou > 50 pages : story dédiée v0.2 |
| L7 | Pas d'horodatage signé / certificat sur le PDF (Swiss CO Art. 958f conformité partielle) | Recherche réglementaire R2 epic-9.md non encore complétée — décision audit-trail-only acceptée pour v0.1 | Si recherche R2 conclut signature requise : story Epic 14 ou 15 |
| L8 | `Content-Disposition` filename UTF-8 RFC 5987 — compatible navigateurs modernes (Chrome/Firefox/Safari) mais peut tronquer en IE11/Edge legacy | v0.1 ne supporte que navigateurs modernes (cohérent reste de l'app) | Pas de tracking — décision PRD |

### Risques & questions ouvertes (à clarifier en spec validate)

| # | Risque / question | À traiter |
|---|---|---|
| Q1 | **Helper `format_swiss_amount`** — local à `kesh-report::pdf` ou factorisé via nouvelle crate `kesh-format` ? Si plusieurs crates dupliquent le format (kesh-qrbill, kesh-report, frontend `formatSwissAmount`), création v0.2 d'une crate `kesh-format` pourrait être justifiée. | Spec validate Pass 1 — décision « accepter duplication v0.1 + L1 tracking dette » sauf objection majeure |
| Q2 | **Audit log signature** — étendre `emit_report_audit` avec param `format: Option<&str>` (backward compat) OU créer `emit_report_export_audit` séparé ? | Spec validate Pass 1 — proposer Option « créer fonction séparée » pour clarté + scope test |
| Q3 | **Endpoint exposing `companyName`** — `+page.svelte` a besoin de `companyName` pour construire le filename (AC #22). Vérifier si `/api/v1/companies/current` ou équivalent existe Story 1-7+. Sinon, fallback `'company'` documenté + dette L2 dédiée. | Spec validate Pass 1 — grep ground-truth `routes/companies.rs` |
| Q4 | **Test fixtures écritures** — pour les 16 tests E2E, faut-il créer un helper `seed_with_entries(fy_id, count)` dans `kesh-db::test_fixtures` ou inline SQL dans le fichier de test ? Story 9-1 a inline les écritures via SQL bypass dans `reports_e2e.rs`. | Spec validate Pass 1 — décision « inline cohérent Story 9-1, pas de nouvelle fixture publique » sauf si > 3 fichiers de test ont besoin du même seed |
| Q5 | **PDF pagination strategy** — si un rapport dépasse 1 page A4, faut-il (a) page break automatique géré par `printpdf::PdfPageReference::add_page`, (b) message « rapport tronqué » avec lien CSV, (c) crash propre ? AC #31 dit `< 5 MB` mais ne couvre pas multi-page. | Spec validate Pass 1 — proposer Option (a) par défaut, footer page X/Y |
| Q6 | **Performance benchmark dataset** — la fixture ~1000 écritures (AC #9) n'existe pas encore. À créer dans `kesh-report/benches/fixtures.rs` ou réutiliser une fixture existante ? | Spec validate Pass 1 — décision construire en code Rust (pas DB) via `BalanceSheet { assets: (0..N).map(...).collect(), ... }` |

### Project Structure Notes

Alignement avec `architecture.md` §11 (workspace) + §17 (FR65-FR68 → `kesh-report/` + `features/reports/`) :
- ✅ `kesh-report::{pdf, csv}` modules dans crate déjà existante (decision #12).
- ✅ Routes API dans `kesh-api/routes/reports.rs` (extension du fichier Story 9-1, pas de nouveau fichier).
- ✅ Frontend dans `lib/features/reports/` (extension `reports.api.ts` + `ReportSelector.svelte` + `+page.svelte`).
- ✅ i18n dans `kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — section `reports-*` étendue.

**Aucun conflit détecté.** Path-dependency soft sur Story 9-2b : si 9-2b démarre avant 9-2a merge, partage potentiel du module `csv` (cf. AC #32 epic-9.md). 9-2b attend 9-2a merge avant dev — convention Epic 8 (8-5a → 8-5b path-dep).

### References

- `_bmad-output/planning-artifacts/epic-9.md` §Story 9-2a (split rationale + ACs originaux)
- `_bmad-output/planning-artifacts/prd.md` FR65, FR66, FR67 (export PDF/CSV per report), UX-DR38 (messages d'erreur actionnables)
- `_bmad-output/planning-artifacts/architecture.md` decision #12 (kesh-report crate), decision #13 (kesh-i18n transverse), §11 (workspace Cargo), §17 (cartographie FR → modules)
- `_bmad-output/implementation-artifacts/9-1-rapports-comptables-bilan-resultat-balance-journaux.md` — API stable consommée, patterns review, ECH-15 audit best-effort, ECH-09 sections fixes journaux
- `crates/kesh-qrbill/src/pdf.rs` — précédent `printpdf 0.7` usage pattern
- `crates/kesh-report/src/lib.rs` — exports publics Story 9-1 (consommés par 9-2a)
- `crates/kesh-api/src/routes/reports.rs` — handlers Story 9-1 (étendus, pas remplacés)
- `frontend/src/lib/features/reports/reports.api.ts` + `ReportSelector.svelte` + `+page.svelte` — frontend Story 9-1 étendu
- Issue GitHub #91 (KF-027) — régression a11y `#bits-c1` pré-existante post-9-1, hors scope 9-2a
- Issue GitHub #90 (closed via PR #92 `ef07548`) — preset `with-company-no-fy` utilisé par AC #20 test

## Dev Agent Record

### Agent Model Used

(à remplir par dev-story)

### Debug Log References

(à remplir par dev-story)

### Completion Notes List

(à remplir par dev-story — inclure résultats benchmark criterion AC #9 + AC #30 + AC #31)

### File List

(à remplir par dev-story)
