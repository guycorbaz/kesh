# Story 9.2a: Export PDF & CSV par rapport

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a utilisateur du logiciel comptable Kesh,
I want exporter chacun des 4 rapports comptables (bilan, compte de résultat, balance des comptes, journaux) en PDF ou en CSV,
so that je puisse les partager avec mon fiduciaire, les archiver hors-ligne, ou les ré-importer dans Excel.

## Scope

Étend l'API publique stable `kesh_report::{BalanceSheet, IncomeStatement, TrialBalance, JournalReport}` livrée par Story 9-1 avec :
- 2 nouveaux modules `kesh-report::{pdf, csv}` (sérialiseurs purs, déterministes, byte-stable pour tests). Nouveau variant `ReportError::PdfGeneration(String)` mappé `AppError::PdfGenerationFailed(s)` (variant existant `errors.rs:184`, code HTTP 500, i18n key dédié — Pass 4 ECH4-L1 sync).
- 4 nouveaux endpoints HTTP `GET /api/v1/reports/{type}/export?format=pdf|csv` (Option B : routes séparées, pas d'extension `?format=` sur les routes JSON existantes — voir Decision §design-export-route). `format` parsé `Option<String>` puis validé handler-side → 400 VALIDATION_ERROR (pas 422 Axum).
- 4 boutons « Export PDF » + 4 boutons « Export CSV » dans la page `/reports` (1 paire par onglet, visible une fois le rapport généré). Flag `exporting` dédié (pas partagé `loading`).
- **10 clés i18n** (`reports-export-*` + `reports-filename-*` + `reports-pdf-*`) × 4 locales (fr/de/it/en-CH).
- Audit log dédié `report.exported` (best-effort, nouvelle fn `emit_report_export_audit` séparée — ne modifie PAS `emit_report_audit` Story 9-1).

**Hors scope (livré par 9-2b)** : export ZIP global par table (souveraineté des données, FR68). Nouveau module `kesh-api/routes/exports.rs`, nouvelle entrée menu principal, `metadata.json` + hash SHA-256.

**Hors scope v0.1** : modèles documents personnalisables (FR81 — Epic 15), drill-down/recherche dans le PDF (FR70 — v0.2), aperçu inline avant download (UX nice-to-have v0.2).

## Acceptance Criteria

### Export PDF (FR67)

1. **Given** un utilisateur authentifié sur `/reports` avec un exercice sélectionné et un rapport (Bilan/Résultat/Balance/Journaux) généré, **When** l'utilisateur clique sur « Export PDF », **Then** le navigateur télécharge un fichier `.pdf` avec `Content-Type: application/pdf` et `Content-Disposition: attachment; filename="..."`.

2. **Given** un PDF généré, **When** ouvert dans n'importe quel lecteur PDF, **Then** les montants apparaissent au format suisse : apostrophe séparateur de milliers et point décimal (`1'234.56`), et les dates au format `dd.mm.yyyy` (FR67).

3. **Given** un PDF de bilan, **When** rendu, **Then** il contient l'en-tête (raison sociale de la company + période + exercice), les sections **Actifs** et **Passifs** énumérant les comptes avec totaux par compte, et la section **Capitaux propres** réduite à **une seule ligne synthétique « Résultat de l'exercice : `<equity_result>` »** (Pass 3 BH3-H2 ground-truth `balance_sheet.rs:30-40` — `BalanceSheet` DTO Story 9-1 expose `equity_result: Decimal` scalaire, pas `Vec<AccountBalance>` ; pas d'`AccountType::Equity` v0.1). L'équation bilan vérifiée (`Total actifs = Total passifs + equity_result`) est affichée en pied de page (cohérent §swiss-amount-format Decision §design-export-route). Refactor multi-lignes équité → CR Epic 14 (clôture) ou Epic 15 si feedback fiduciaire.

4. **Given** un PDF de compte de résultat, **When** rendu, **Then** il contient l'en-tête, les sections Produits / Charges avec totaux, et le résultat net (bénéfice / perte) en pied de page.

5. **Given** un PDF de balance, **When** rendu, **Then** toutes les lignes ont **5 colonnes** (Numéro, Compte, Débit, Crédit, Solde) avec totaux débit = totaux crédit en bas (Pass 1 BH-L1 + AA-H1 — typo « 4 » → « 5 »).

6. **Given** un PDF de journaux, **When** rendu (sans filtre `journal`), **Then** les 5 sections (Achats, Ventes, Banque, Caisse, OD) sont toujours présentes même vides — cohérent avec le DTO JSON Story 9-1 (Pass 1 ECH-09).

7. **Given** un PDF de journaux **with** filtre `journal=Ventes`, **When** rendu, **Then** seule la section Ventes apparaît, en-tête mentionne « Journal : Ventes ».

8. **Given** un rapport vide (aucune écriture dans la période), **When** export PDF, **Then** le PDF est généré quand même avec le message `ctx.empty_message` (i18n key `reports-pdf-empty-message`) rendu au centre. Le handler résout la clé via `kesh-i18n` AVANT d'appeler `render_*_pdf` — `kesh-report` ne dépend pas de `kesh-i18n` (Pass 1 AA-M1 — Architecture decision §swiss-amount-format respectée).

9. **Given** un dataset de référence (~1000 écritures), **When** génération PDF d'un rapport, **Then** la durée de **rendering pur** mesurée via criterion bench `< 500 ms` sur dataset 1000 écritures (Pass 1 AA-L1 + ECH-M3 — clarification scope : le benchmark mesure le rendering seul, pas le round-trip réseau ; le budget total client→download `< 3s` est documenté comme cible CI mais pas asserted via test automatisé v0.1).

### Export CSV

10. **Given** un rapport généré, **When** l'utilisateur clique sur « Export CSV », **Then** le navigateur télécharge un fichier `.csv` avec `Content-Type: text/csv; charset=utf-8` et `Content-Disposition: attachment; filename="..."`.

11. **Given** un CSV, **When** ouvert dans Excel (CH/DE) ou LibreOffice, **Then** :
    - encodage UTF-8 avec BOM (`\u{FEFF}` en tête)
    - séparateur point-virgule `;` (PAS la virgule — convention Excel CH/DE)
    - fin de ligne `\r\n` (CRLF, RFC 4180)
    - montants au format ISO décimal point (`1234.56` — pas d'apostrophe en CSV, format machine-readable pour ré-import)
    - dates ISO 8601 (`2026-05-15` — machine-readable, pas le format affichage)
    - chaînes contenant `;`, `"`, ou retour-ligne sont entourées de `"` avec `"` interne doublé en `""` (RFC 4180).

12. **Given** un CSV de bilan, **When** rendu, **Then** colonnes : `Section;NumeroCompte;NomCompte;Solde` où `Section ∈ {Actifs, Passifs, CapitauxPropres}`. Ligne de total par section pour Actifs et Passifs. **La section `CapitauxPropres` ne contient qu'UNE ligne** (Pass 3 BH3-H2) : `CapitauxPropres;;Résultat de l'exercice;<equity_result>` (NomCompte = "Résultat de l'exercice" en clair, NumeroCompte vide car pas de compte du plan comptable derrière — `BalanceSheet::equity_result` est scalaire). Ligne finale `Total actifs;;;<somme>` + `Total passifs + capitaux propres;;;<somme>`. **Invariant hérité Story 9-1** : les deux totaux finaux sont toujours égaux (calcul `BalanceSheet::equity_result`) — pas de ligne de vérification explicite, l'égalité est implicite (Pass 1 ECH-M4).

13. **Given** un CSV de compte de résultat, **When** rendu, **Then** colonnes : `Section;NumeroCompte;NomCompte;Solde` où `Section ∈ {Produits, Charges}`. Total par section + ligne finale `ResultatNet;;;<somme>`.

14. **Given** un CSV de balance, **When** rendu, **Then** colonnes : `NumeroCompte;NomCompte;TotalDebit;TotalCredit;Solde`. Ligne finale totaux débit/crédit avec colonnes solde vide.

15. **Given** un CSV de journaux **sans** filtre, **When** rendu, **Then** colonnes : `Journal;DateEcriture;NumeroEcriture;Description;NumeroCompte;NomCompte;Debit;Credit`. Une ligne par `journal_entry_line`. Ordre par `Journal ASC, entry_date ASC, journal_entry_id ASC, line_index ASC`.

16. **Given** un CSV de journaux **with** filtre `journal=Ventes`, **When** rendu, **Then** seules les lignes du journal Ventes apparaissent, première colonne `Journal` reste populée (= "Ventes" partout).

17. **Given** un rapport vide, **When** export CSV, **Then** le CSV contient **uniquement la ligne d'en-tête** (1 ligne, pas de data rows) — pas de comment `#` car non RFC 4180 (Pass 1 ECH-M1). Le frontend détecte le cas vide en comptant les lignes du blob téléchargé.

### Frontend : intégration UI

18. **Given** la page `/reports` après génération réussie d'un rapport, **When** l'utilisateur regarde la zone de contrôles, **Then** 2 nouveaux boutons « Export PDF » et « Export CSV » apparaissent à droite du bouton « Générer » dans `ReportSelector.svelte` (ou dans le bandeau du tabpanel).

19. **Given** aucun rapport généré (avant `Générer`), **When** l'utilisateur regarde les boutons d'export, **Then** ils sont `disabled` (pattern cohérent `noFiscalYears` / `loading` Story 9-1 `ReportSelector`).

20. **Given** un exercice sans `fiscal_year` (preset `with-company-no-fy` Issue #90 ef07548), **When** l'utilisateur regarde les boutons d'export, **Then** ils sont `disabled` (cohérent AC #34 Story 9-1 — pas d'export possible sans exercice).

21. **Given** export PDF/CSV en cours, **When** l'utilisateur clique sur l'autre format, **Then** le second clic est ignoré jusqu'à fin du premier via **flag dédié `exporting`** (PAS partagé avec `loading`, cf. AC #36 + Pass 2 AA2-C1 — décision verrouillée Pass 1 ECH-H2, alternative « shared loading » retirée).

22. **Given** un nom de fichier suggéré au browser, **When** download déclenché, **Then** le format est `kesh-{typeSlug}-{companyShort}-{periodStart}_{periodEnd}.{ext}` où :
    - `{typeSlug}` = valeur de la clé i18n `reports-filename-{reportType}` où `reportType ∈ {balance-sheet, income-statement, trial-balance, journals}` (clé en slug anglais, valeur localisée — fr-CH = `bilan`/`compte-resultat`/`balance`/`journaux`, de-CH = `bilanz`/`erfolgsrechnung`/etc.) — Pass 1 AA-H3 résolu.
    - `{companyShort}` = slug ASCII de `company.name` via `buildExportFilename` (NFD strip diacritics + lowercase + non-alphanum → `-` + collapse repeated `-` + truncate 20 chars + strip trailing `-`). Fallback `company` si slug résultant est vide.
    - `{periodStart}` / `{periodEnd}` = `YYYY-MM-DD` (machine-readable pour tri filesystem). Si `period_start`/`end` étaient `None` à l'API (donc résolus depuis le FY par `ReportPeriod::resolve`), le filename utilise les bornes effectives du FY.
    - `{ext}` = `pdf` ou `csv`
    - Exemple : `kesh-bilan-ci-test-company-2026-01-01_2026-12-31.pdf`

23. **Given** une erreur backend (500, 400, 401), **When** export échoue, **Then** le message d'erreur est affiché dans la zone d'alerte existante `errorMsg` (`+page.svelte` Story 9-1) avec format UX-DR38 — ce qui s'est passé + ce que l'utilisateur peut faire.

### Multi-tenant + sécurité

24. **Given** un utilisateur authentifié de company A, **When** il appelle `GET /api/v1/reports/balance-sheet/export?fiscalYearId=<id_de_company_B>`, **Then** 404 `FISCAL_YEAR_NOT_FOUND` (cohérent Story 9-1 — `ReportPeriod::resolve` joint sur `company_id`). Aucune fuite de données cross-tenant.

25. **Given** un utilisateur Consultation, **When** export PDF/CSV, **Then** réussit (lecture seule, cohérent avec rôles autorisés sur `GET /api/v1/reports/*` Story 9-1).

26. **Given** un utilisateur non authentifié, **When** export PDF/CSV, **Then** 401 (middleware auth standard).

27. **Given** le serveur backend, **When** export `?format=invalid`, `?format=` absent, `?format=PDF` (uppercase) ou `?format=Csv` (mixed case), OU `?fiscalYearId` absent ou ≤ 0, OU `?periodStart`/`periodEnd` en dehors des bornes de l'exercice, **Then** 400 `VALIDATION_ERROR` JSON avec message listant la cause (Pass 1 BH-H2 + ECH-H4 + AA-L4 + Pass 2 ECH2-H1 — **strict lowercase enforced**, pas de case-insensitive matching). Le `format` est `Option<String>` parsé serde (jamais 422 Axum), validé handler-side via `match query.format.as_deref() { Some("pdf") => Pdf, Some("csv") => Csv, _ => Err(AppError::Validation(...)) }`. Les autres params suivent le pattern Story 9-1 (`validate_fiscal_year_id` + `ReportPeriod::resolve`).

### Audit + observabilité

28. **Given** un export PDF ou CSV réussi, **When** réponse 200 retournée, **Then** une ligne `audit_log` est insérée avec :
    - `action = 'report.exported'`
    - `entity_type = 'report'`
    - `entity_id = AUDIT_ENTITY_ID_NONE` (cohérent Story 9-1 — pas d'entité 1:1)
    - `details_json` incluant `reportType`, `format`, `fiscalYearId`, `periodStart`, `periodEnd`, `journalFilter` (si applicable)
    - Pattern best-effort : INSERT échec → `warn!` log + retour 200 (ne JAMAIS faire échouer le download — Story 9-1 ECH-15).

29. **Given** export en cours, **When** monitoring observe les logs structurés, **Then** un span tracing `report_export` est émis avec attributs `report_type`, `format`, `byte_size`, `duration_ms`.

### Performance + limites

30. **Given** un dataset large (10k écritures dans un journal), **When** export CSV journaux, **Then** la durée de rendering pur mesurée via criterion bench `< 5 secondes` (Pass 1 AA-C1 corrigée — RAM bornée via `csv::Writer` vers un `Vec<u8>` final, pas de `Vec<String>` par ligne, mais **PAS de streaming Axum body v0.1** cf. L5).

31. **Given** un dataset large, **When** export PDF, **Then** taille du fichier `< 5 MB` pour 10k écritures dans un journal (binding implicite : limite de pagination si nécessaire — voir §pagination-pdf §Limitations).

### Tests

32. **Given** la story est implémentée, **When** suite de tests exécutée, **Then** **≥ 16 tests E2E HTTP** dans `crates/kesh-api/tests/reports_export_e2e.rs` couvrent (Pass 1 AA-M2 + Pass 2 AA2-H2 — décomposition corrigée à 16 exact, stratégie empty report mise à jour) : (a) 4 endpoints PDF × 200 binary content, (b) 4 endpoints CSV × 200 text/csv content, (c) format invalid 400 VALIDATION_ERROR (incl. `?format=PDF` uppercase rejet), (d) multi-tenant 404, (e) FY out of bounds 400 (period **avant ou après** FY 2020-2030, e.g. `2019-12-31` ou `2031-01-01`), (f) rapport vide PDF + CSV success path (2 tests : **`with-company` seedé SANS aucune écriture journal_entries** + période dans FY 2020-2030 → reports retournent vides — pas via période 2099 qui est out-of-FY et déclencherait 400), (g) auth 401, (h) RBAC Consultation 200 PDF + Consultation 200 CSV (2 tests). Total : 4+4+1+1+1+2+1+2 = **16**.

33. **Given** la story est implémentée, **When** tests unit `kesh-report` exécutés, **Then** **≥ 12 tests** valident : (a) PDF byte signature commence par `%PDF-1.` (4 rapports × 1 = 4 tests), (b) CSV BOM + séparateur + CRLF (4 tests), (c) CSV escaping RFC 4180 (1 test), (d) format suisse montant + date dans PDF via grep regex sur le texte décompressé (3 tests).

34. **Given** la story est implémentée, **When** Vitest exécuté, **Then** **≥ 3 tests** sur `reports.api.ts` : (a) `downloadReport(type, query, 'pdf', filename)` appelle `fetch` avec l'URL correcte construite via `getReportExportUrl` et retourne via Blob (mock fetch) — Pass 1 AA-H2 fn name correct ; (b) `buildExportFilename` produit la string attendue pour les 4 types + edge cases (accents `Müller AG` → `muller-ag`, longueur >20 truncate + strip trailing `-`, slug vide → fallback `company`) ; (c) erreur backend (mock 500) → exception formatée propagée à `+page.svelte` `formatError`.

35. **Given** la story est implémentée, **When** Playwright exécuté, **Then** **1 scénario actif** `reports-export-pdf.spec.ts` télécharge un PDF + assert byte signature `%PDF-1.` + filename pattern. **(Pass 1 ECH-M5)** Utiliser `await download.saveAs('/tmp/kesh-test-9-2a.pdf')` puis `fs.readFile()` — pas `download.path()` qui peut retourner `null`.

36. **(Nouvelle AC Pass 1 ECH-H2)** **Given** un export PDF/CSV en vol, **When** l'utilisateur change `selectedFiscalYearId` côté frontend, **Then** : (a) l'export en vol cible toujours l'ancien FY (closure captures `query` au moment du clic) ; (b) le flag `exporting` est **reset à `false` dans le `finally`** du handler `exportPdf()`/`exportCsv()` quelle que soit l'issue (success ou catch error) ; (c) si erreur, `errorMsg` est mis à jour même si l'UI affiche le nouveau FY (acceptable v0.1, documenté L9). **Le flag `exporting` est dédié et NE partage PAS `loading`** (qui contrôle uniquement `generate()`).

## Tasks / Subtasks

- [ ] **T1** Ajouter dépendances `printpdf` + `csv` à `crates/kesh-report/Cargo.toml` (AC: #1, #10)
  - [ ] T1.1 `printpdf = "0.7"` (cohérent `kesh-qrbill`, ne PAS upgrader sans coordination)
  - [ ] T1.2 `csv = "1.3"` (sérialiseur standard Rust, RFC 4180 compliant)
  - [ ] T1.3 Vérifier `cargo build -p kesh-report` clean après ajout

- [ ] **T2** Créer `crates/kesh-report/src/pdf.rs` — sérialiseur PDF pur (AC: #1-#9, #29, #31 — Pass 4 AA4-LOW3 ajout AC #31 pour T2.3a pagination)
  - [ ] T2.0 **(Pass 1 ECH-C2 + Pass 3 ECH3-H2)** Ajouter `PdfGeneration(String)` à `ReportError` (`crates/kesh-report/src/errors.rs`) — **mappé `AppError::PdfGenerationFailed(s)`** (variant existant `crates/kesh-api/src/errors.rs:184`, cohérent avec `invoice_pdf.rs` Story 5-3 + i18n key `error-pdf-generation-failed`, code HTTP 500, message client utile). **PAS `AppError::Internal`** (Pass 3 ECH3-H2 invalide la dismission Pass 2 ECH2-C1 : ECH2-C1 disait « mapping déjà documentée » mais T2.0 disait `Internal`, alors que le variant dédié `PdfGenerationFailed` existe déjà avec son IntoResponse branch). Sans ça, `printpdf::save_to_bytes() -> Result<_, PrintpdfError>` ne peut être propagé en `Result<Vec<u8>, ReportError>` ET on perd l'i18n message UX existant.
  - [ ] T2.1 Signature publique `pub fn render_balance_sheet_pdf(bs: &BalanceSheet, ctx: &PdfContext) -> Result<Vec<u8>, ReportError>` (× 4 fonctions, une par rapport)
  - [ ] T2.2 **(Pass 3 ECH3-C1 + BH3-M2)** `PdfContext { company_name: String, locale: String, empty_message: String, journal_filter_label: Option<String>, section_labels: SectionLabels }` — tous champs owned `String` (pas `&'static str` qui forcerait `Box::leak` ou consts). `locale` est **`String` (e.g. `"fr-CH"`)** dérivé côté handler via `SELECT accounting_language FROM companies WHERE id = ?` (1 query DB ajoutée par export, perf acceptable < 5ms). `empty_message`, `journal_filter_label` (`Some("Ventes")` si filtre), `section_labels` (struct local avec `actifs`, `passifs`, `capitaux_propres`, `produits`, `charges`, etc. — chargés depuis kesh-i18n côté handler) sont tous résolus AVANT d'appeler `render_*_pdf`. `kesh-report::pdf` reste découplé de `kesh-i18n` (DD-14 pattern `kesh-qrbill`).
  - [ ] T2.3 Helpers privés `draw_header`, `draw_table_row`, `draw_totals_footer` (DRY — réutilisés par les 4 fonctions)
  - [ ] T2.3a **(Pass 2 ECH2-M3 + Pass 3 ECH3-M3 constants)** Pagination automatique. **Constants centralisées** (Pass 3 ECH3-M3 — sans ça dev devine) en haut de `pdf.rs` :
    ```rust
    const PAGE_WIDTH_MM: f32 = 210.0;   // A4 portrait
    const PAGE_HEIGHT_MM: f32 = 297.0;
    const MARGIN_TOP_MM: f32 = 20.0;
    const MARGIN_BOTTOM_MM: f32 = 25.0;
    const FONT_SIZE_PT: f32 = 10.0;
    const LINE_HEIGHT_MM: f32 = 4.2;    // ≈ 12pt at 10pt font
    // → utile ≈ 252mm / 4.2mm ≈ 60 rows/page A4
    ```
    Tracker `cursor_y` au fil des `draw_table_row`. Quand `cursor_y - LINE_HEIGHT_MM < MARGIN_BOTTOM_MM`, appeler `let (new_page_idx, new_layer_idx) = doc.add_page(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), format!("Layer {}", page_num));` et reset `cursor_y = PAGE_HEIGHT_MM - MARGIN_TOP_MM` (header redrawn condensed). Helper `draw_footer(layer, page_num, total_pages)` ajouté (footer X/Y). **Test** : benchmark 10k écritures journal_report → assert PDF byte size `< 5 MB` (AC #31) ET pagination déclenchée (count pages > 1 via inspection PDF bytes).
  - [ ] T2.4 Format suisse montants via helper local `format_swiss_amount(decimal: Decimal) -> String` (apostrophe séparateur + point décimal, **toujours 2 décimales** `format!("{:.2}", ...)`, gestion signe négatif avec `-` en préfixe). Cohérence à vérifier avec `kesh-i18n::format_money` (Pass 1 BH-H3 — nom correct, pas `format_amount`) — accepté duplication v0.1 (cf. Decision §swiss-amount-format + Q1).
  - [ ] T2.5 Format dates `dd.mm.yyyy` via `NaiveDate::format("%d.%m.%Y")`
  - [ ] T2.6 Cas dégénéré rapport vide : afficher `ctx.empty_message` centré, ne pas crasher (AC #8 patché Pass 1 AA-M1 — i18n résolu côté handler, **PAS** dépendance kesh-i18n dans kesh-report)
  - [ ] T2.7 Format négatifs : tester explicitement `format_swiss_amount(Decimal::from_str("-1234.56"))` → `"-1'234.56"` (Pass 1 ECH-L1).

- [ ] **T3** Créer `crates/kesh-report/src/csv.rs` — sérialiseur CSV pur (AC: #10-#17)
  - [ ] T3.1 Signature publique `pub fn render_balance_sheet_csv<W: Write>(bs: &BalanceSheet, writer: W) -> Result<(), ReportError>` (× 4 fonctions, streaming-friendly)
  - [ ] T3.2 BOM en tête : `writer.write_all(b"\xef\xbb\xbf")?;` avant d'instancier `csv::WriterBuilder`
  - [ ] T3.3 `csv::WriterBuilder::new().delimiter(b';').terminator(csv::Terminator::CRLF).from_writer(writer)`
  - [ ] T3.4 Tests RFC 4180 escaping : (a) un nom de compte contenant `;` doit être entouré de `"..."` ; (b) un nom contenant `"` doit doubler le `"` → `""` ; (c) un nom contenant `\n` ou `\r\n` doit être entouré de `"..."` (Pass 1 ECH-L4). Le crate `csv 1.3` gère (b) et (c) automatiquement, juste tester.
  - [ ] T3.5 Cas rapport vide : écrire **uniquement la ligne d'en-tête, aucune ligne suivante** (Pass 1 ECH-M1 — drop le `# Aucune écriture` qui n'est pas RFC 4180 et serait affiché en data row par Excel). Le frontend peut détecter le cas vide via la présence d'une seule ligne dans le blob téléchargé.
  - [ ] T3.6 Format décimal : **toujours 2 décimales** via `format!("{:.2}", amount)` — `Decimal::ZERO` → `"0.00"` (Pass 1 ECH-H5 — sinon Excel auto-typing colonne mixte casse).

- [ ] **T4** Étendre `crates/kesh-report/src/lib.rs` exports + créer benchmark (AC: #1, #10, #9, #30, #31)
  - [ ] T4.1 `pub mod csv;` + `pub mod pdf;`
  - [ ] T4.2 Re-exports : `pub use pdf::{render_balance_sheet_pdf, render_income_statement_pdf, render_trial_balance_pdf, render_journal_report_pdf, PdfContext};` + symétrique pour CSV
  - [ ] T4.3 **(Pass 1 AA-M4)** Créer `crates/kesh-report/benches/export.rs` avec criterion (à ajouter en dev-dep `criterion = "0.5"`) — bench les 4 rapports PDF + 4 CSV sur **2 fixtures** : 1000 écritures (AC #9) et 10000 écritures (AC #30 + #31). Fixtures construites en code Rust (pas DB) via factories `make_balance_sheet(n_accounts: usize)` etc. — accepted Q6.
  - [ ] T4.4 Bench documenté dans `Cargo.toml` : `[[bench]] name = "export" harness = false`

- [ ] **T5** Créer 4 nouveaux endpoints `kesh-api/src/routes/reports.rs` (AC: #1, #10, #22-#29)
  - [ ] T5.1 Ajouter `pub async fn export_balance_sheet`, `export_income_statement`, `export_trial_balance`, `export_journal_report` — signature `(State<AppState>, Extension<CurrentUser>, Query<ExportQuery>) -> Result<Response, AppError>` (réponse binaire, pas `Json<>`).
  - [ ] T5.2 **(Pass 1 BH-H2)** Définir `struct ExportQuery { fiscal_year_id: i64, period_start: Option<NaiveDate>, period_end: Option<NaiveDate>, journal: Option<Journal>, format: Option<String> }` avec `#[serde(rename_all = "camelCase")]`. **Le champ `format` est `Option<String>` pas `enum`** — la validation est faite handler-side via `validate_format(&query.format)? -> ExportFormat` qui retourne `AppError::Validation("format manquant ou invalide, attendu pdf|csv")` (400 JSON cohérent). Évite le 422 Axum par défaut sur deserialization enum failure. Le champ `journal` reste `Option<Journal>` global (ignoré par les 3 premiers rapports — pattern Story 9-1 `JournalReportQuery`).
  - [ ] T5.3 **(Pass 1 AA-M3 + ECH-M2)** Construire `Response` :
    - PDF : `axum::response::Response::builder().header(CONTENT_TYPE, "application/pdf").header(CONTENT_DISPOSITION, build_content_disposition(filename)?).body(Body::from(pdf_bytes)).unwrap()`. Helper `build_content_disposition` retourne **les deux formes** : `attachment; filename="ascii_fallback"; filename*=UTF-8''<percent-encoded>` (RFC 5987 + ASCII fallback). Sans ça, `HeaderValue::from_str` panic sur un company name `"Müller AG"` (caractères non-ISO-8859-1).
    - CSV : idem avec `text/csv; charset=utf-8` + extension `.csv`. Body construit dans un `Vec<u8>` (cf. L5 — streaming Axum hors scope v0.1).
  - [ ] T5.4 **(Pass 1 BH-M1 + ECH-C1 + AA-M3 + Pass 3 BH3-M3 + Pass 4 ECH4-L3)** Filename helper `fn build_filename(type_slug: &str, company_name: &str, period: &ReportPeriod, ext: &str) -> String` — slug ASCII via **regex inline** (`[^a-z0-9-]` → `-` puis `-+$` → `""`), max 20 chars, **fallback `"report"` pour `type_slug` vide post-slug** (cas CJK future locale) **et fallback `"company"` pour `company_name` vide post-slug**. **Slug appliqué à `type_slug` ET `company_name`** (Pass 3 BH3-M3 — defense in depth). **Pas de crate `slug`** ajoutée à kesh-api/Cargo.toml. Le slug ASCII est la valeur du `filename=` ; le `filename*=UTF-8''<percent-encoded>` utilise le nom original UTF-8.
  - [ ] T5.5 **(Pass 1 BH-M2)** Audit log `report.exported` via **nouvelle fonction séparée** `emit_report_export_audit(pool, user_id, report_type, format, fiscal_year_id, period_start, period_end, journal_filter)` — **PAS** modification de la signature existante `emit_report_audit` (qui briserait les 4 callers Story 9-1). Pattern best-effort identique.
  - [ ] T5.6 Validation `fiscal_year_id > 0` réutilisée (`validate_fiscal_year_id` helper Story 9-1).
  - [ ] T5.7 Multi-tenant via `ReportPeriod::resolve(&state.pool, current_user.company_id, ...)` — identique Story 9-1 (AC #24).
  - [ ] T5.8 **(Pass 1 AA-H4 + Pass 3 BH3-M1 syntax fixed)** Instrumenter les 4 handlers avec le pattern correct `tracing` Rust :
    ```rust
    let span = tracing::info_span!(
        "report_export",
        report_type = %report_type,
        format = %format,
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();
    /* ... fetch report + render PDF/CSV → body: Vec<u8> ... */
    span.record("byte_size", body.len());
    span.record("duration_ms", start.elapsed().as_millis() as u64);
    ```
    **Pass 3 BH3-M1** : `info_span!` évalue les field values à la création du span, donc `byte_size = body.len()` directement ne compile pas (body inconnu). Pattern correct = `tracing::field::Empty` placeholder + `span.record()` après le travail.

- [ ] **T6** Mount routes dans `crates/kesh-api/src/lib.rs` (AC: #1, #10, #26)
  - [ ] T6.1 **(Pass 1 BH-H1)** Insérer les 4 nouvelles routes **AVANT le `;` de fermeture** de `let authenticated_routes = Router::new()...;` (ligne ~373 actuelle), en chaînant `.route()` sur la dernière route Story 9-1. **Ne PAS** insérer après le `;` — ça créerait des routes orphelines hors `authenticated_routes` → 401 silencieusement bypass → IDOR cross-tenant critique :
    - `.route("/api/v1/reports/balance-sheet/export", get(routes::reports::export_balance_sheet))`
    - `.route("/api/v1/reports/income-statement/export", get(routes::reports::export_income_statement))`
    - `.route("/api/v1/reports/trial-balance/export", get(routes::reports::export_trial_balance))`
    - `.route("/api/v1/reports/journals/export", get(routes::reports::export_journal_report))`
  - [ ] T6.2 Vérification anti-régression : après merge, `grep -A1 "authenticated_routes = Router" crates/kesh-api/src/lib.rs | head -50` doit montrer les 4 routes export AVANT le `;`. Si test E2E AC #26 (auth 401) échoue avec 200 sur n'importe quel endpoint export, c'est ce bug.

- [ ] **T7** Étendre frontend `frontend/src/lib/features/reports/` (AC: #18-#23)
  - [ ] T7.1 Ajouter dans `reports.api.ts` :
    - `getReportExportUrl(type: ReportType, query: ReportQuery, format: 'pdf' | 'csv', journal?: string): string` (construit l'URL avec query string).
    - `downloadReport(type: ReportType, query: ReportQuery, format: 'pdf' | 'csv', filename: string): Promise<void>` (déclenche le download via fetch blob + lien `<a download>` éphémère, gère 401 redirect via `apiClient`).
    - `buildExportFilename(type: ReportType, companyName: string, period: { start: string; end: string }, format: 'pdf' | 'csv'): string` — implémente le pattern AC #22 + slug ASCII via regex. **Post-process slug** : `.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().replace(/[^a-z0-9-]/g, '-').replace(/-+/g, '-').slice(0, 20).replace(/-+$/, '')` (Pass 1 ECH-H3 — trailing hyphens supprimés après truncate).
  - [ ] T7.2 Modifier `ReportSelector.svelte` : ajouter 2 boutons `Export PDF` + `Export CSV` à droite du bouton « Générer ». Props : `onExportPdf: () => void` + `onExportCsv: () => void` + `canExport: boolean` (vrai si un rapport est généré ET pas en cours d'export).
  - [ ] T7.3 **(Pass 1 ECH-H1 + BH-M3 + AA-H5 — Q3 RÉSOLU + Pass 2 ECH2-H2 + Pass 3 ECH3-H1)** Modifier `+page.ts` (existant) pour charger `companyName` en parallèle de `fiscalYears` via **`fetchCompanyCurrent()` existant dans `lib/features/settings/settings.api.ts`** (Pass 3 ECH3-H1 ground-truth — helper déjà créé Story 1-7+, **PAS** de nouveau fichier `lib/features/companies/companies.api.ts` à créer, qui dupliquerait). Endpoint backend confirmé `GET /api/v1/companies/current` à `companies.rs:75` retournant `CompanyCurrentResponse { company: { name, ... }, ... }`. Import : `import { fetchCompanyCurrent } from '$lib/features/settings/settings.api';`. Couplage cross-feature `reports → settings` accepté v0.1 (alternative : extraire dans `lib/shared/api/` → refactor cosmétique reporté). Extension de `PageData` : `interface PageData { fiscalYears: FiscalYearResponse[]; companyName: string }`. **Gestion erreurs complète** (Pass 2 ECH2-H2) :
    - `401` → re-throw (`+layout.ts` auth guard redirect login).
    - `403` → impossible en théorie (Consultation peut lire companies), mais si rencontré → fallback `companyName = 'company'` + log warn.
    - `500`, network timeout, autres → fallback `companyName = 'company'` (filename dégradé mais page utilisable, cohérent avec pattern Story 9-1 fiscalYears `[]` fallback).
    - **Pas de throw** non-401 (sinon page crash sur fluctuation backend transitoire — pattern différent du fiscalYears Story 9-1 qui propage parce que c'est critique pour la fonctionnalité ; ici companyName est cosmétique filename, dégradé acceptable).
  - [ ] T7.4 Handler `async function exportPdf()` + `exportCsv()` : appelle `downloadReport(activeTab, query, format, buildExportFilename(...))`, gère erreurs via `formatError` existant Story 9-1. **(Pass 1 ECH-H2)** Utiliser **flag `exporting` dédié** (pas partagé avec `loading`), toujours reset dans `finally`. Pas de race guard `genSeq` — les exports sont fire-and-forget côté state (le blob déclenche le download navigateur, pas d'écriture state critique).
  - [ ] T7.5 Garde `canExport` : `false` si aucun rapport généré, `false` si `noFiscalYears`, `false` si `loading || exporting`.
  - [ ] T7.6 **(Pass 1 ECH-H2 follow-up)** Si l'utilisateur change `selectedFiscalYearId` pendant un export en vol : l'export en vol cible toujours l'ancien FY (closure captures `query` au moment du clic). Au retour : (a) succès → blob téléchargé avec filename portant l'ancien FY (correct UX) ; (b) erreur → message affiché dans `errorMsg` même si UI affiche le nouveau FY (acceptable v0.1, documenté dette L9 ci-dessous).

- [ ] **T8** Ajouter clés i18n × 4 locales (fr/de/it/en-CH) — AC: #22, #23, #18-#21
  - [ ] T8.1 **(Pass 1 AA-H3)** Clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` — **les suffixes i18n utilisent les slugs anglais cohérents avec `ReportType` côté TS (`balance-sheet`, `income-statement`, etc.)**. Le **filename** rendu reste en français (FR locale) — c'est la valeur de la clé, pas la clé elle-même :
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
  - [ ] T8.2 Idem pour `de-CH`, `it-CH`, `en-CH` (traductions de base, validation native v0.2 — L4 héritée). DE filename ex : `bilanz`, `erfolgsrechnung`, `kontensaldenliste`, `journale`. IT/EN équivalents.
  - [ ] T8.3 **(Pass 1 AA-L3)** `npm run lint-i18n-ownership` PASS — toutes les clés `reports-export-*`, `reports-filename-*`, `reports-pdf-*` appartiennent à `lib/features/reports/`. Le préfixe `reports-csv-*` n'est plus utilisé (suite à T3.5 patché Pass 1 ECH-M1, le `csv-comment-empty` est supprimé).
  - [ ] T8.4 `cargo run -p kesh-i18n --bin validate-locale-coverage` (si existant) — sinon validation manuelle 10 clés présentes dans les 4 locales (Pass 1 BH-L3 — 10 clés exactement, plus 11 ni 12).

- [ ] **T9** Tests E2E HTTP `crates/kesh-api/tests/reports_export_e2e.rs` (AC: #32)
  - [ ] T9.1 **16 tests minimum** (voir AC #32 décomposition Pass 1 patchée — `(a)4+(b)4+(c)1+(d)1+(e)1+(f)2+(g)1+(h)2 = 16`).
  - [ ] T9.2 **(Pass 1 AA-M5 + Pass 2 AA2-H2)** Pattern fixture identique `reports_e2e.rs` Story 9-1 : `seed_accounting_company` + écritures via SQL bypass. **Stratégie spécifique** :
    - **Tests positifs PDF/CSV** : `with-company` + insertion de 3-5 écritures via SQL (cohérent Story 9-1 reports_e2e.rs). Période query incluse dans FY 2020-2030.
    - **Test rapport vide (AC #32(f))** : `with-company` seedé **SANS aucune insertion `journal_entries`** + query `?fiscalYearId=<id>&periodStart=2026-01-01&periodEnd=2026-12-31` (période DANS FY 2020-2030 mais aucune écriture en DB). Les 4 rapports retournent vides → AC #8 (PDF empty_message) et AC #17 (CSV header-only). **NE PAS utiliser période 2099** qui est out-of-FY (FY est 2020-2030) → déclencherait 400 PERIOD_OUT_OF_FY au lieu d'empty report (Pass 2 AA2-H2 ground-truth `test_fixtures.rs:114` → FY 2020-2030).
    - **Test multi-tenant 404 (AC #32(d))** : créer un 2e seed via 2e appel `seed_accounting_company_extra` ou fixture inline 2 companies.
    - **Test auth 401 (AC #32(g))** : pas de Bearer token, **request GET** (les 4 endpoints export sont GET, cf. T6.1 — Pass 4 BH4-L3 coquille « POST » corrigée) sur 1 endpoint export → 401 (test isolé, pas obligation de tester les 4 endpoints — middleware auth identique sur authenticated_routes router).
    - **Test RBAC Consultation (AC #32(h))** : créer 1 user role Consultation + login → PDF + CSV success path = 2 tests.
  - [ ] T9.3 Assertions content-type + content-disposition + premiers bytes pour PDF (`assert!(body.starts_with(b"%PDF-1."))`) et CSV (`assert_eq!(&body[..3], b"\xef\xbb\xbf")`).
  - [ ] T9.4 **(Pass 1 ECH-M2)** Test content-disposition avec company name non-ASCII (e.g. `"Müller AG"`) — assert header bien formé, pas de `HeaderValue` panic. Réutilise `seed_accounting_company` mais override `companies.name` via SQL `UPDATE`.

- [ ] **T10** Tests unit `crates/kesh-report/src/{pdf,csv}.rs` + benchmark (AC: #33, #9, #30)
  - [ ] T10.1 12 tests unit minimum (voir AC #33 décomposition).
  - [ ] T10.2 Benchmark criterion `crates/kesh-report/benches/export.rs` exécuté localement avant push — résultats commités en commentaire dans le commit ou dans le story file `Completion Notes`.

- [ ] **T11** Tests frontend Vitest `frontend/src/lib/features/reports/reports.api.test.ts` (AC: #34)
  - [ ] T11.1 **3 tests minimum** (Pass 1 ECH-H3 patché — strip trailing hyphens explicitement testé) :
    - `buildExportFilename` produit la string attendue pour les 4 types + slug edge cases : accents `Müller AG` → `muller-ag`, espaces multiples `Kesh ---   SA` → `kesh-sa`, longueur >20 truncate `acme-sa-fribourg-extension-long` → `acme-sa-fribourg-ext` (puis strip trailing `-` si applicable), nom vide → fallback `company`, nom chinois `北京公司` → fallback `company`.
    - `downloadReport` mock `fetch` + assert appel `apiClient` correct + Blob retourné.
    - `downloadReport` rejette quand status 500 + message formaté.

- [ ] **T12** Playwright `frontend/tests/e2e/reports-export-pdf.spec.ts` (AC: #35)
  - [ ] T12.1 **(Pass 1 ECH-M5)** 1 scénario : login → /reports → générer Bilan → cliquer « Export PDF » → `const download = await page.waitForEvent('download')` → `await download.saveAs('/tmp/kesh-test-9-2a.pdf')` → `fs.readFile('/tmp/kesh-test-9-2a.pdf')` → assert premier byte `%PDF-1.` + assert `download.suggestedFilename()` match pattern `/^kesh-bilan-.*-\d{4}-\d{2}-\d{2}_\d{4}-\d{2}-\d{2}\.pdf$/`.
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
| `criterion` (dev) | 0.5 | Nouvelle dev-dep `kesh-report` | Benchmark `< 500ms` AC #9 reproductible |
| `percent-encoding` | 2.x | **(Pass 1 AA-M3)** Réutilisée dans workspace via `reqwest`/`url` (vérifier pas besoin d'ajout explicite Cargo.toml) | RFC 5987 percent-encoding pour `Content-Disposition filename*=UTF-8''<...>` |
| ~~`slug`~~ | ~~0.1~~ | **(Pass 1 BH-M1 + ECH-C1)** Retirée — regex inline backend + frontend cohérents | Pas de nouvelle dépendance |

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
- `crates/kesh-api/src/errors.rs` — **(Pass 4 ECH4-M1)** ajouter bras `ReportError::PdfGeneration(s) => AppError::PdfGenerationFailed(s)` dans le `match err` de `From<ReportError> for AppError` (`errors.rs:476`). Sans ça, build échoue E0004 non-exhaustive match dès T2.0 commit. **Préserver** : 5 bras existants (`Db`, `FiscalYearNotFound`, `PeriodInvalid`, `PeriodOutOfFiscalYear`, `TrialBalanceUnbalanced`).
- `frontend/src/lib/features/reports/reports.api.ts` (~117 lignes existantes) — ajouter `getReportExportUrl`, `downloadReport`, `buildExportFilename`
  - **Préserver** : `getBalanceSheet`, `getIncomeStatement`, `getTrialBalance`, `getJournalReport`, `isReportEmpty`, `formatSwissDate`, `formatReportAmount`, helpers `buildQuery`
- `frontend/src/lib/features/reports/ReportSelector.svelte` — ajouter 2 boutons + 2 props
- `frontend/src/routes/(app)/reports/+page.svelte` (~218 lignes existantes) — ajouter handlers `exportPdf`/`exportCsv` + state `exporting` + bind `onExportPdf`/`onExportCsv`
- 4 fichiers `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — ajouter **10 clés** : 4× `reports-export-*` + 4× `reports-filename-*` + 2× `reports-pdf-*` (Pass 4 BH4-L2 + AA4-LOW1 + ECH4-L2 sync, total cohérent avec T8.1 + T8.4 + Scope §line 19).

### Testing standards

- **Tests d'intégration Rust** : `crates/kesh-api/tests/reports_export_e2e.rs` — pattern `reqwest` + `spawn_app` identique `reports_e2e.rs` Story 9-1. Helper `assert_pdf_response` + `assert_csv_response` à factoriser dans le fichier (pas dans `kesh-db::test_fixtures` — scope test seulement).
- **Tests unit Rust** : `crates/kesh-report/src/{pdf,csv}.rs` modules `#[cfg(test)] mod tests` en bas de chaque fichier. Fixtures `BalanceSheet`/`IncomeStatement`/etc. construites en code (pas de DB).
- **Tests frontend Vitest** : `frontend/src/lib/features/reports/reports.api.test.ts` (nouveau) — mock `fetch` via `vi.mock`.
- **Playwright** : `frontend/tests/e2e/reports-export-pdf.spec.ts` — pattern `await page.waitForEvent('download')` + `download.suggestedFilename()` + `await download.saveAs('/tmp/...')` + `fs.readFile(filepath)` (Pass 2 BH2-H1 — **PAS** `download.path()` qui peut retourner `null` selon config Playwright, cohérent T12.1).
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
- `JournalReport { period: ReportPeriod, journals: Vec<JournalSection>, grand_total_debit: Decimal, grand_total_credit: Decimal }` — **Pass 1 BH-C1 ground-truth `journal_report.rs:21-26`** : `journal_filter` n'est **PAS** un champ du struct, c'est uniquement un paramètre de `generate_journal_report(..., journal: Option<Journal>)`. Le label du filtre pour le header PDF doit être passé via `PdfContext.journal_filter_label` (cf. T2.2 patché).

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
| L6 | **(Pass 3 BH3-H1 corrigée)** Pagination PDF automatique **implémentée** v0.1 (T2.3a) : page break à `cursor_y < MARGIN_BOTTOM_MM (25mm)` + footer `page X/Y`. Pas de TOC ni breaks intelligents par section (e.g., couper milieu de section Actifs) | Pagination cosmétique uniquement v0.1 — section breaks intelligents + TOC → v0.2 si feedback | v0.2 si > 50 pages observé sur dataset réel |
| L7 | Pas d'horodatage signé / certificat sur le PDF (Swiss CO Art. 958f conformité partielle) | Recherche réglementaire R2 epic-9.md non encore complétée — décision audit-trail-only acceptée pour v0.1 | Si recherche R2 conclut signature requise : story Epic 14 ou 15 |
| L8 | `Content-Disposition` filename UTF-8 RFC 5987 — compatible navigateurs modernes (Chrome/Firefox/Safari) mais peut tronquer en IE11/Edge legacy | v0.1 ne supporte que navigateurs modernes (cohérent reste de l'app) | Pas de tracking — décision PRD |
| L9 | **Export en vol + changement de FY** : si l'utilisateur change `selectedFiscalYearId` pendant un export en vol, l'export complète sur l'ancien FY (closure capture). En cas d'erreur, `errorMsg` est mis à jour même si l'UI affiche le nouveau FY → confusion UX possible (Pass 1 ECH-H2). | Acceptable v0.1 : exports rapides (~500ms benchmark) limitent la fenêtre. Mitigation v0.2 si feedback utilisateur : ajouter un toast contextualisé « Export bilan FY 2025 terminé » au lieu de mettre à jour `errorMsg` partagé. | v0.2 si flake observé |
| L10 | **(Pass 3 ECH3-M1)** Duplicate query param `?format=pdf&format=csv` : serde_urlencoded (Axum default) retient la **dernière** valeur silencieusement → l'export retourne le format `csv`. Pas de 400 distinct. | Comportement par défaut du parser, faible probabilité (user n'écrit pas manuellement les URLs), pas de risque sécurité. | v0.2 si rapporté |
| L11 | **(Pass 3 ECH3-M2)** PDF Helvetica builtin ne supporte que Latin-1 ish. Si `company.name` contient Cyrillique/CJK (`Газпром`, `北京公司`, etc.), le PDF rend des cases vides ou caractères de remplacement à la place. AC #3 (« en-tête raison sociale ») visuellement dégradé. | Dégradation gracieuse acceptée v0.1 (cohérent L3) ; le filename reste utilisable via slug ASCII (T5.4) ; v0.2 si demande explicite tenants non-Latin avec embedding TTF (`printpdf::PdfFont::from_bytes`). | Epic 15 si demande |
| L12 | **(Pass 3 BH3-L1)** Wording T7.3 « 401 → re-throw » diverge subtilement du pattern Story 9-1 `+page.ts:18-22` qui `return { fiscalYears: [] }` sur 401. Les deux fonctionnent (api-client redirige déjà), mais inconsistance dans le même fichier. | Acceptable v0.1 : sémantique identique en pratique (redirect login déclenché par api-client). Refactor cosmétique → si revue manuelle insiste. | LOW |

### Risques & questions ouvertes — RÉSOLUS Pass 1

Toutes les Q1-Q6 ont été tranchées en Pass 1 Sonnet 4.6 (cycle CLAUDE.md). Verdict :

| # | Question | Décision Pass 1 | Référence patch |
|---|---|---|---|
| Q1 | Helper `format_swiss_amount` local ou factorisé `kesh-format` ? | **Accepté duplication v0.1**. Helper local à `kesh-report::pdf`, pas de dépendance kesh-i18n (Decision §swiss-amount-format). Si > 3 crates dupliquent → story Epic 15 v0.2. **Pass 1 BH-H3 fix incluse** : nom correct `kesh-i18n::format_money` (pas `format_amount`). | T2.4 |
| Q2 | `emit_report_audit` modifié ou nouveau `emit_report_export_audit` ? | **Nouveau fn séparé** mandaté. Modifier la signature existante briserait les 4 callers Story 9-1 → 28 E2E tests reports_e2e.rs cassent (Pass 1 BH-M2). | T5.5 |
| Q3 | Endpoint `companyName` existe ? | **Oui, confirmé `companies.rs:75`** (Pass 1 ECH-H1 + BH-M3 + AA-H5 ground-truth). Chargé via `+page.ts` load function en parallèle de `fiscalYears`, extension `PageData`. Fallback `'company'` uniquement si `data.companyName === ''` (pathologique). | T7.3 |
| Q4 | Fixtures écritures helper public ou inline SQL ? | **Inline SQL** cohérent Story 9-1 `reports_e2e.rs`. Pas de nouveau preset endpoint. Stratégie spécifique : `with-company` + insertions SQL ad-hoc + tests rapport vide via **seed SANS écritures + période DANS FY 2020-2030** (Pass 2 AA2-H2 invalidant la stratégie 2099 hors FY — Pass 4 BH4-L1+AA4-LOW2+ECH4 trail sync). | T9.2 + AC #32(f) |
| Q5 | PDF pagination strategy multi-page ? | **Option (a) page break automatique** via `doc.add_page()` quand cursor Y approche 25mm du bas. Footer `page X/Y` dans helper `draw_footer`. AC #31 (< 5 MB pour 10k entries) sécurise contre fichiers absurdes. | T2.3 + AC #31 |
| Q6 | Benchmark dataset construction code vs DB ? | **Code Rust pur** via factories `make_balance_sheet(n_accounts: usize)` etc. — pas de DB pour bench, isolation des effets DB (latence/buffer pool). 2 fixtures : 1000 + 10000 écritures (Pass 1 AA-M4). | T4.3 |

**Pass 1 = 0 question résiduelle.** Toutes verrouillées par grep/Read ground-truth.

### Pass 1 clarifications — codes findings traités

Section synthèse pour traçabilité review iteration CLAUDE.md.

| Code | Finding | Sévérité Pass 1 | Patch |
|---|---|---|---|
| BH-C1 | `JournalReport` struct ground-truth (pas de `journal_filter` field, mais `grand_total_debit/credit`) | CRITICAL | Dev Notes API stable corrigé + `PdfContext.journal_filter_label` ajouté T2.2 + ECH-H8 résolu en synergie |
| BH-H1 | T6.1 « après ligne 373 » placerait routes hors `authenticated_routes` (IDOR) | HIGH | T6.1 réécrit + T6.2 vérification grep ajoutée |
| BH-H2 | `?format=invalid` retourne 422 Axum pas 400 VALIDATION_ERROR | HIGH | T5.2 `format: Option<String>` + validation handler-side → AC #27 patché (couvre aussi ECH-H4) |
| BH-H3 | `kesh-i18n::format_amount` n'existe pas (vraie fn `format_money`) | HIGH | T2.4 corrigé |
| BH-M1 | Contradiction slug crate vs regex inline | MEDIUM (escaladé CRITICAL via ECH-C1) | T5.4 : regex inline, pas de crate `slug` |
| BH-M2 | Modifier `emit_report_audit` casse 4 callers Story 9-1 | MEDIUM | T5.5 : nouvelle fonction `emit_report_export_audit` mandatée |
| BH-M3 | Q3 companyName ambigu — endpoint existe | MEDIUM | Voir Q3 ci-dessus + T7.3 |
| BH-L1 | AC #5 typo « 4 colonnes » → 5 | LOW (escaladé HIGH via AA-H1) | AC #5 patché |
| BH-L2 | lib.rs ~30 → 28 lignes | LOW | File structure note non patchée (cosmétique pure) |
| BH-L3 | Scope « ~12 clés » → 10 | LOW | T8.4 patché « 10 clés » |
| ECH-C1 | Filename slug divergence backend vs frontend vs Decision | CRITICAL | T5.4 + T7.1 + Decision §filename-slug harmonisés (regex inline + RFC 5987 backend) |
| ECH-C2 | `ReportError::PdfGeneration` variant manquant | CRITICAL | T2.0 nouveau (variant + From mapping) |
| ECH-H1 | Q3 endpoint companyName | HIGH (merge BH-M3 + AA-H5) | Voir Q3 + T7.3 |
| ECH-H2 | `exporting` flag race condition | HIGH | AC #36 nouvelle + T7.4 flag dédié + T7.6 closure capture documentée |
| ECH-H3 | Slug trailing hyphens post-truncate | HIGH | T7.1 regex `replace(/-+$/, '')` + T11.1 test edge cases élargi |
| ECH-H4 | Missing `fiscalYearId` → 422 pas 400 | HIGH (merge BH-H2) | AC #27 patché |
| ECH-H5 | Decimal `0` vs `0.00` format Excel | HIGH | T3.6 nouveau `format!("{:.2}", ...)` |
| ECH-H6 | `PdfContext` pas de `journal_filter` info | HIGH (merge BH-C1) | T2.2 `journal_filter_label: Option<String>` ajouté |
| ECH-M1 | Empty CSV `#` comment non RFC 4180 | MEDIUM | T3.5 + AC #17 patché : header seul, pas de comment |
| ECH-M2 | Content-Disposition RFC 5987 manquant | MEDIUM (merge AA-M3) | T5.3 helper `build_content_disposition` filename + filename* |
| ECH-M3 | Performance SLA ambiguë | MEDIUM (merge AA-L1) | AC #9 clarifié rendering pur < 500ms |
| ECH-M4 | PDF/CSV equity verification inconsistance | MEDIUM | AC #12 note invariant implicite |
| ECH-M5 | Playwright `download.path()` may be null | MEDIUM | T12.1 `download.saveAs()` mandaté |
| ECH-L1 | Test format swiss montant négatif | LOW | T2.7 nouveau test négatif |
| ECH-L2 | Risque clash i18n keys `reports-filename-balance-sheet` vs `reports-balance-sheet` | LOW | Acceptable, lint-i18n-ownership couvre (T8.3) |
| ECH-L3 | `details_json` journalFilter null consistency | LOW | Reporté Pass 2 ou dev-story (cosmétique audit) |
| ECH-L4 | Test CSV escape `\n` dans champ | LOW (escaladé) | T3.4 (c) ajouté |
| ECH-L5 | CSV `Vec<u8>` Content-Length note | LOW | Acceptable, hyper chunked transfer fallback |
| AA-C1 | AC #30 streaming claim contradicte Vec<u8> | CRITICAL | AC #30 réécrit |
| AA-H1 | AC #5 « 4 colonnes » → 5 | HIGH (merge BH-L1) | AC #5 patché |
| AA-H2 | AC #34(a) `getReportPdf` fn inexistante | HIGH | AC #34(a) → `downloadReport` |
| AA-H3 | i18n key naming FR slug vs EN slug contradiction | HIGH | AC #22 + T8.1 harmonisés : clé EN slug, valeur localisée FR/DE/IT/EN |
| AA-H4 | AC #29 tracing span orphan task | HIGH | T5.8 nouveau + T5 AC liste mise à jour |
| AA-H5 | Q3 endpoint companyName | HIGH (merge ECH-H1) | Voir Q3 |
| AA-M1 | AC #8 i18n key viole Decision §swiss-amount-format | MEDIUM | AC #8 patché : `ctx.empty_message` résolu handler-side |
| AA-M2 | AC #32 décomp sums 15 not 16 | MEDIUM | AC #32 décompo corrigée 16 exact (h split en 2) |
| AA-M3 | Content-Disposition simple vs RFC 5987 | MEDIUM (merge ECH-M2) | T5.3 patché |
| AA-M4 | Bench dataset 1000 vs AC #30 10k | MEDIUM | T4.3 patché 2 fixtures 1000 + 10000 |
| AA-M5 | Empty report fixture strategy manquante | MEDIUM | T9.2 stratégie `period=2099-01-01/2099-12-31` |
| AA-L1 | Performance threshold ambigu | LOW (merge ECH-M3) | AC #9 patché |
| AA-L2 | « ~12 clés » → 10 | LOW (merge BH-L3) | Voir BH-L3 |
| AA-L3 | Lint-i18n ne couvre pas `reports-pdf-*` `reports-csv-*` | LOW | T8.3 patché : `reports-pdf-*` ajouté ; `reports-csv-*` plus utilisé (T3.5 drop comment) |
| AA-L4 | AC #32(e) « FY out of bounds 400 » sans AC déclarant | LOW | AC #27 patché (couvre ce cas) |

**Trend Pass 1** : 43 findings bruts → 34 distincts post-dedup → **31 patches appliqués** (Option A : C+H+M) + 3 LOW résiduels acceptés (L1 cosmétique, L2 cosmétique, ECH-L3 audit cosmétique reporté).

### Pass 2 Haiku 4.5 clarifications — codes findings traités

3 reviewers Haiku 4.5 parallèles fresh-context. Pattern mémoire `feedback_haiku_review_diff_combined` confirmé : 10/19 findings dismissed post-ground-truth (53% faux-positifs Haiku).

| Code | Finding | Sévérité Pass 2 | Verdict |
|---|---|---|---|
| BH2-H1 | Testing standards section ~344 référence `download.path()` mais T12.1 mandate `saveAs()` | HIGH | **PATCHÉ** — Testing standards updated |
| ECH2-C1 | `ReportError::PdfGeneration` mapping missing | CRITICAL Haiku | **DISMISSED** — T2.0 dit explicitement « mappé `AppError::Internal` (500) dans `From<ReportError> for AppError` », la mapping est déjà documentée |
| ECH2-C2 | Exporting flag leak on navigation | CRITICAL Haiku | **DISMISSED** — Svelte 5 `$state` component-scoped, `finally` sur composant détruit = no-op silencieux, pas de leak réel |
| ECH2-H1 | Format case-sensitivity undefined | HIGH | **PATCHÉ** — AC #27 + T5.6 explicit lowercase strict, test case `?format=PDF` rejet |
| ECH2-H2 | SSR + companyName fetch non-401 error handling | HIGH | **PATCHÉ** — T7.3 spec explicit fallback `'company'` pour 403/500/network, throw 401 seul |
| ECH2-H3 | RFC 5987 percent-encoding round-trip test | HIGH | **DEFER LOW** — T9.4 « bien formé » acceptable v0.1, round-trip decode optionnel (test ajouté si HEADER panic observé en dev-story) |
| ECH2-M1 | Empty CSV detection « 1 ligne » ambigu split | MEDIUM | **DEFER LOW** — AC #17 + T3.5 clair (header seul, byte count via blob size). Frontend détection n'est PAS dans le scope 9-2a (juste afficher le download) |
| ECH2-M2 | T6.2 grep n'assure pas que test E2E AC #26 tourne | MEDIUM | **DISMISSED** — T13.4 mandate `cargo test --workspace -j1` clean, incluant `reports_export_e2e.rs`. Si test 401 échoue, CI rouge. Pas besoin d'AC supplémentaire |
| ECH2-M3 | PDF pagination strategy incomplete (pas de sub-task) | MEDIUM | **PATCHÉ** — T2.3a nouvelle sub-task page break + footer X/Y + test 10k size |
| ECH2-M4 | Audit log silent failure compliance | MEDIUM | **DEFER LOW** — cohérent pattern Story 9-1 best-effort, monitoring séparé hors scope 9-2a (Epic 9 retro) |
| AA2-C1 | AC #21 « au choix dev » contredit AC #36 mandate dedicated flag | CRITICAL Haiku | **PATCHÉ** comme HIGH (faux CRITICAL — c'est une contradiction de wording, pas un bug bloquant). AC #21 réécrit |
| AA2-H1 | AC #8 `reports-pdf-empty-message` clarification | HIGH | **DISMISSED** — T3.5 patché Pass 1 = CSV header-only sans message. Clé `reports-pdf-empty-message` correctement nommée (PDF seul) |
| AA2-H2 | AC #32(f) stratégie 2099 out-of-FY | HIGH | **PATCHÉ** — AC #32 + T9.2 réécrits : empty report via seed sans écritures + période DANS FY 2020-2030 |
| AA2-M1 | T4.3 factory API ambiguity n_accounts vs n_lines | MEDIUM | **DEFER LOW** — détail implémentation, le dev clarifiera (criterion factories sont prototype interne) |
| AA2-M2 | AC #34 explanation | MEDIUM | **SELF-DISMISSED** par AA2 reviewer |
| AA2-M3 | T5.8 tracing duration_ms timing pattern | MEDIUM | **DEFER LOW** — pseudo-code T5.8 acceptable, dev choisit pattern `Instant::now() + .elapsed()` standard Rust |
| AA2-L1 | AC #22 typeSlug encoding détail | LOW | **SELF-DISMISSED** par AA2 reviewer |
| AA2-L2 | `?format=` empty string handling | LOW | **DEFER LOW** — T5.6 validation match Option<String> couvre None et Some("") en même return Err |
| ECH2-L1 | T11.1 exemple math `acme-sa-fribourg-ext` (20 chars) | LOW | **DEFER LOW** — exemple cosmétique, le dev fera l'arithmétique en test |
| ECH2-L2 | CSV trailing CRLF test scope | LOW | **DEFER LOW** — AC #33(b) « 4 tests CSV BOM + séparateur + CRLF » suffisant, dev raffinera |

**Bilan Pass 2 post-ground-truth** :
- 19 bruts → 4 dismissed (Haiku faux-positifs 21%) + 9 self/cross-dismissed (47% deferred LOW ou cosmétique) → **6 patches appliqués** (3 HIGH + 3 MEDIUM légitimes).
- Trend > LOW : Pass 1 = 31 → **Pass 2 = 6** (-81%).

### Pass 3 Opus 4.7 clarifications — pattern 8-5b retro CONFIRMÉ

3 reviewers Opus 4.7 parallèles fresh-context — pattern Story 8-5b retro confirmé : **Opus brise consensus Sonnet+Haiku par ground-truth grep aggressif**. 2 dismissions Pass 2 invalidées (ECH2-C1 wrong target variant, T2.3a sans update L6).

| Code | Finding | Sévérité Pass 3 | Verdict |
|---|---|---|---|
| BH3-H1 | L6 contradicte T2.3a — Pass 2 patched pagination sans updater Limitations | HIGH | **PATCHÉ** — L6 réécrite « pagination implémentée v0.1 » |
| BH3-H2 | AC #3 + #12 décrivent CapitauxPropres avec « totaux par classe » mais `BalanceSheet` DTO Story 9-1 a `equity_result: Decimal` scalaire, pas `Vec<AccountBalance>` | HIGH | **PATCHÉ** — AC #3 + #12 clarifient « section CapitauxPropres = 1 ligne synthétique Résultat de l'exercice » |
| BH3-M1 | T5.8 `tracing::info_span!` pseudo-code ne compile pas (`body.len()` inconnu à création span) | MEDIUM | **PATCHÉ** — T5.8 réécrite avec pattern `tracing::field::Empty` + `span.record()` correct |
| BH3-M2 | `PdfContext.locale: &'static str` force `Box::leak` ou consts | MEDIUM (bundled C1) | **PATCHÉ** — locale `String` owned (cf. C1 ECH3-C1) |
| BH3-M3 | Non-ASCII translation values briseraient filename header si future locale ajoutée | MEDIUM | **PATCHÉ** — T5.4 slug regex sur full filename (type_slug + company_name) |
| BH3-L1 | T7.3 401 re-throw diverge Story 9-1 pattern fallback | LOW | **DEFER L12** — sémantique équivalente en pratique |
| BH3-L2 | Pas de test smoke audit `report.exported` row INSERTed | LOW | **DEFER LOW** — pattern best-effort Story 9-1 hérité |
| ECH3-C1 | `PdfContext.locale` source non spécifiée (Accept-Language ? DB ?) | CRITICAL | **PATCHÉ** (bundled BH3-M2) — T2.2 réécrite : locale via SELECT `companies.accounting_language` côté handler, 1 query DB ≤ 5ms, pattern documenté |
| ECH3-H1 | T7.3 mandate créer `lib/features/companies/companies.api.ts:getCurrentCompany` mais `fetchCompanyCurrent` existe déjà dans `lib/features/settings/settings.api.ts` | HIGH | **PATCHÉ** — T7.3 réécrite : réutiliser `fetchCompanyCurrent`, pas créer duplicate |
| ECH3-H2 | T2.0 mappe vers `AppError::Internal` mais variant dédié `AppError::PdfGenerationFailed(s)` existe déjà à `errors.rs:184` avec i18n. **Pass 2 ECH2-C1 dismissed à tort** | HIGH | **PATCHÉ** — T2.0 corrigée : map vers `PdfGenerationFailed` cohérent invoice_pdf |
| ECH3-H3 | T6.1 devrait mentionner explicit que routes export DOIVENT être dans `authenticated_routes` pas admin/comptable | HIGH (marginal) | **DEFER LOW** — T6.1 dit déjà « avant le `;` de `let authenticated_routes = ...` », explicite enough. AC #32(h) Consultation test attraperait régression |
| ECH3-M1 | Duplicate `?format=pdf&format=csv` silent last-wins | MEDIUM | **PATCHÉ L10** — documenté comme dette acceptable v0.1 |
| ECH3-M2 | Asian/Cyrillic company name → glyph missing rendu PDF | MEDIUM | **PATCHÉ L11** — dégradation gracieuse documentée, v0.2 si demande |
| ECH3-M3 | T2.3a pagination « Y < 25mm » sans constants (MARGIN, FONT_SIZE, LINE_HEIGHT) | MEDIUM | **PATCHÉ** — T2.3a constants centralisées en haut de `pdf.rs` |
| ECH3-L1 | Spec ne dit pas si `companyName` + `fiscalYears` chargés en `Promise.all` ou séquentiel | LOW | **DEFER LOW** — impact perf marginal |
| ECH3-L2 | `details_json` audit log pas de `companyId` | LOW | **DEFER LOW** — pré-existant pattern Story 9-1 |
| AA3-L1 | Pass 2 patch trail mislabels T5.6 (vraie task T5.2) | LOW | **DEFER LOW** — cosmétique trail |
| AA3-L2 | T2 header AC mapping incomplet pour T2.3a (devrait inclure AC #31) | LOW | **DEFER LOW** — traceability matrix |
| AA3-L3 | Deferred LOWs trackés dans patch trail pas dans Limitations | LOW | **DEFER LOW** — convention pré-existante |

**Bilan Pass 3 post-ground-truth** :
- 19 bruts → 15 distincts post-dedup → **10 patches appliqués** (1 CRITICAL + 4 HIGH + 5 MEDIUM légitimes) + 8 LOW deferred (cosmétiques + dette tracée v0.2).
- Trend > LOW : Pass 1 = 31 → Pass 2 = 6 → **Pass 3 = 10** (régression apparente mais qualité réelle améliorée — Opus a invalidé 2 dismissions Pass 2 et trouvé 1 CRITICAL gap conceptuel sur la locale source).

**Critère arrêt CLAUDE.md NON atteint** — Pass 4 Sonnet 4.6 obligatoire (cycle Sonnet → Haiku → Opus → Sonnet, briser biais Opus auteur Pass 3 + valider 10 patches sans régression). Budget 3/8 passes consommé. **Splitting préventif évalué** : Pass 3 régression numérique mais bug-type findings (pas conceptuels) → split 9-2a non justifié, Pass 4 devrait converger sur 0 > LOW si patches Pass 3 propres.

### Pass 4 Sonnet 4.6 clarifications — CONVERGENCE ATTEINTE

3 reviewers Sonnet 4.6 parallèles fresh-context (BH4+ECH4+AA4) post-Pass-3 Opus. **Tous 3 reviewers GO**. Pattern 8-5b retro respecté : Sonnet brise biais Opus Pass 3 author.

| Code | Finding | Sévérité Pass 4 | Verdict |
|---|---|---|---|
| ECH4-M1 | File Structure UPDATE list manque `crates/kesh-api/src/errors.rs` (où ajouter `From<ReportError>` arm) | MEDIUM | **PATCHÉ** — UPDATE list ajoute errors.rs avec note exhaustive match E0004. ECH4 lui-même note « compiler-guardrailed → reclassement LOW acceptable » |
| BH4-L1+AA4-LOW2+ECH4-L1 | Stale references à « 2099 » dans Q4 row + « AppError::Internal » dans Scope §line 16 + « 11 clés » dans File Structure | LOW (3 codes mergés) | **PATCHÉ** — Scope §line 16 sync `PdfGenerationFailed`, Q4 row sync stratégie empty-report DANS FY, File Structure « 11 → 10 clés » |
| BH4-L3 | T9.2 ligne 244 dit « POST » mais routes export sont GET | LOW | **PATCHÉ** — T9.2 « request GET » |
| AA4-LOW3 | T2 header AC mapping manque AC #31 | LOW | **PATCHÉ** — T2 header `(AC: #1-#9, #29, #31)` |
| ECH4-L3 | type_slug fallback empty post-slug non spécifié | LOW | **PATCHÉ** — T5.4 fallback `"report"` pour type_slug vide, `"company"` pour company_name vide |
| BH4-L2 + AA4-LOW1 + ECH4-L2 | Variantes du « 11 vs 10 clés » | LOW (merge ci-dessus) | **PATCHÉ** unique |

**Pass 3 patches verification** : **10/10 ✓** confirmés ground-truth par les 3 reviewers Pass 4 (`PdfGenerationFailed` variant `errors.rs:184`, `BalanceSheet::equity_result` scalaire, `fetchCompanyCurrent` dans `settings.api.ts:4`, `accounting_language` field `company.rs:150`, `printpdf 0.7` + `tracing 0.1` versions ground-truth correctes).

**Bilan Pass 4 post-ground-truth** :
- 10 bruts → 7 distincts post-dedup → **7 patches appliqués** (1 MEDIUM cosmetic spec gap + 6 LOW typos / wording stale) → **0 > LOW post-patch**.
- Trend > LOW : Pass 1 = 31 → Pass 2 = 6 → Pass 3 = 10 → **Pass 4 = 1 (essentiellement 0 post-patch)**.

### 🎯 CONVERGENCE ATTEINTE — Cycle review STOP

**Critère arrêt CLAUDE.md ATTEINT Pass 4** : « Uniquement des findings de sévérité LOW » → tous patchés. Le 1 MEDIUM (ECH4-M1) est reclassement LOW justifié — Rust exhaustive match E0004 compiler-guardrail rend l'oubli impossible côté implémentation (le dev ne peut pas ship un build qui casse).

**Budget consommé : 4/8 passes.** Marge restante 4 passes inutilisée.

**Cycle final** : Sonnet (auteur Pass 1) → Haiku Pass 2 → Opus Pass 3 (brise consensus, invalide 2 dismissions Pass 2) → Sonnet Pass 4 (brise biais Opus, confirme convergence). Pattern 8-5b retro pleinement respecté.

**Verdict final : `GO ready-for-dev` FERME**. Spec ~1300 lignes, 36 ACs, 13 tasks (T1-T13 + sub-tasks T2.0/T2.3a/T2.7/T3.6/T5.8/T6.2/T7.6/T9.4 ajoutés au fil des passes). Le dev a une spec complète avec :
- Ground-truth verified (kesh-report API contracts, AppError variants, fetchCompanyCurrent, accounting_language, FY 2020-2030, printpdf 0.7, tracing 0.1)
- Decisions verrouillées (R1-R8 résolues, Q1-Q6 fermées, L1-L12 documentées)
- Test budget chiffré (16 E2E HTTP + 12 unit + 3 Vitest + 1 Playwright = 32 tests minimum)
- 4 passes de patches cumulés (31 + 6 + 10 + 7 = 54 patches au total)

**Path-dep aval** : 9-2b (export global ZIP) peut démarrer dès que 9-2a est mergée (réutilise `kesh-report::csv` serializer + filename helpers + audit pattern).

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
