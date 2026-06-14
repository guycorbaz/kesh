# Story 11.2: Calcul TVA & rapport par période (TVA due / vente)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## ⚠️ Note de cadrage (ground-truth vérifié 2026-06-14)

Cette story livre **le calcul de la TVA par ligne (arrondi commercial, FR55)** et **un rapport TVA par période exportable PDF/CSV (FR56)**. Elle **réutilise** massivement l'infrastructure existante : arrondi `Money`, crate `kesh-report` (Epic 9), pattern d'export `reports.rs`, sélection temporelle `vat_rates` (Story 11-1). **Aucun changement de schéma** (calcul à la volée, décision Guy 2026-06-14).

### Ce qui existe déjà (à réutiliser, NE PAS réinventer)

- **Arrondi commercial** : `kesh_core::types::Money::round_to_centimes()` (`crates/kesh-core/src/types/money.rs:66`) = `round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)` (half-up). **C'est exactement FR55** (123.455 → 123.46). Testé (`money.rs:220-235`). `rust_decimal = "1.41"` features `serde-str` + `maths` (`kesh-core/Cargo.toml:10`). **Jamais de f64.**
- **Infra rapport + export PDF/CSV** (Epic 9, Stories 9-1/9-2a) — crate `crates/kesh-report/` :
  - 4 rapports existants (`balance_sheet`, `income_statement`, `trial_balance`, `journal_report`) — chacun = `pub struct` `Serialize` camelCase + `pub async fn generate(pool, company_id, &ReportPeriod) -> Result<_, ReportError>` (modèle : `income_statement.rs:30-50`).
  - `ReportPeriod::resolve(pool, company_id, fiscal_year_id, period_start, period_end)` (`period.rs:74`) — résout la fenêtre + valide les bornes dans l'exercice.
  - PDF : `printpdf = "0.7"`, `PdfContext` (`pdf.rs:54`) + `SectionLabels` (`pdf.rs:72`, **`pub` mais champs obligatoires** — cf. M5 ci-dessous), `PdfBuilder` (`pdf.rs:216`, **privé** — pagination A4, Helvetica builtin, footer page X/Y) + helpers `draw_*` privés, `format_swiss_amount` (`1'234.56`) / `format_swiss_date` (`15.01.2026`). ⚠️ `PdfBuilder` et les `draw_*` étant privés, `render_vat_report_pdf` **DOIT** être écrit dans `pdf.rs` (même fichier). Erreur `ReportError::PdfGeneration` → HTTP 500 `AppError::PdfGenerationFailed`.
  - CSV : `csv = "1.3"`, BOM UTF-8, délimiteur `;`, terminator CRLF, montants ISO 2 décimales (`csv.rs:24-39`). Erreur `ReportError::CsvGeneration` → HTTP 500 `AppError::CsvGenerationFailed`.
  - Re-exports : `crates/kesh-report/src/lib.rs:16-43`.
- **Routes export** (`crates/kesh-api/src/routes/reports.rs`) — pattern par rapport : `get_*` (JSON on-screen) + `export_*` (binaire). Helpers réutilisables :
  - `validate_format(&query.format)` + `validate_fiscal_year_id(...)` ;
  - `ExportQuery { fiscal_year_id, period_start: Option<NaiveDate>, period_end: Option<NaiveDate>, journal: Option<Journal>, format: Option<String> }` (`reports.rs:76-84`) ;
  - `load_pdf_context(pool, company_id) -> (PdfContext, company_name)` (locale BCP-47 depuis `companies.accounting_language`) ;
  - `render_csv_to_vec(|w| ...)` ;
  - `resolve_type_slug(state, locale, report_type)` (slug i18n `reports-filename-{type}`) ;
  - `build_export_response_with_locale(format, body, type_slug, company_name, period, locale)` (Content-Type + Content-Disposition RFC 5987) ;
  - `emit_report_audit(...)` (`reports.rs:700`, `report.generated`) / `emit_report_export_audit(...)` (`reports.rs:756`, `report.exported`) — best-effort, non bloquant, détails JSON snake_case. ⚠️ **Ces deux fn sont privées (`async fn`, non `pub`)** : les nouveaux handlers `get_vat_report`/`export_vat_report` **DOIVENT** être ajoutés dans `reports.rs` (même fichier) — ne pas créer un nouveau module route. Modèle d'export complet : `export_income_statement` (`reports.rs:337-398`).
- **Enregistrement routes** : les 8 routes rapports sont dans **`authenticated_routes`** (tous rôles authentifiés, lecture seule, FR65) — `lib.rs:478-513`. ⚠️ Les routes export DOIVENT rester **avant le `;`** de `authenticated_routes` (sinon orphelines hors guards → 401 bypass → IDOR cross-tenant, cf. commentaire BH-H1 `lib.rs:495-497`).
- **Frontend rapports** (`frontend/src/routes/(app)/reports/+page.svelte`, 333 lignes ; `frontend/src/lib/features/reports/`) : 4 onglets + sélecteur d'exercice + plage de dates, flags `loading`/`exporting` séparés, garde anti-race `genSeq`. Helpers `reports.api.ts` : `downloadReport(type, query, format, filename)`, `buildExportFilename(...)`, `triggerDownload(blob, filename)` via `<a download>` éphémère (**HTTP-LAN-safe**, pas d'API secure-context). Lien nav : `+layout.svelte:75` (`{ label: 'Rapports', href: '/reports' }`).
- **i18n rapports** : clés `reports-filename-*` (`crates/kesh-i18n/locales/fr-CH/messages.ftl:898-901`), mapping langue `map_language_to_bcp47` (`util.rs`).
- **Sélection temporelle TVA** (Story 11-1) : `vat_rates::find_for_category_at_date(pool, company_id, category, date)` (`repositories/vat_rates.rs:87`) — déterministe. Voir §Données pour son rôle (limité) dans cette story.

### Ce qui MANQUE (le périmètre de cette story)

1. **Aucun calcul de TVA** n'existe : `compute_line_total` (`repositories/invoices.rs:277-279`) ne calcule que `quantity × unit_price` (HT, arrondi 4 déc.) ; aucune fonction `base × rate / 100`. Le helper de calcul TVA par ligne (FR55) est à créer.
2. **Aucun rapport TVA** : la crate `kesh-report` n'a pas de module `vat_report`.
3. **Aucune route ni page** de rapport TVA.

### Gap structurel assumé (décision Guy 2026-06-14) — TVA récupérable déférée

⚠️ **Les écritures comptables ne tracent PAS la TVA** : à la validation d'une facture, seules **2 lignes** sont créées (débit client / crédit produit, `invoices.rs` validate ~l.1024-1051) avec le montant **HT** (`total_amount` = somme des `line_total`, jamais de TTC). Il n'existe **pas de comptes TVA** dans le plan comptable (`AccountType` = Asset/Liability/Revenue/Expense uniquement), **pas de saisie d'achats avec TVA**, et les `journal_entry_lines` n'ont **aucun champ TVA**.

**Conséquence** : la **TVA due (vente)** est calculable depuis les factures de vente validées ; la **TVA récupérable (achats / impôt préalable)** n'a **aucune source de données** dans le modèle actuel.

**Décision (Guy 2026-06-14)** : cette story livre la **TVA due seule**. Le rapport est néanmoins **structuré au format décompte** dès maintenant — les lignes/totaux **TVA récupérable** et **solde** existent dans le modèle (`VatReport`), le PDF et le CSV, **à `0.00`** pour l'instant. La **TVA récupérable sur achats sera implémentée dans une story de suivi dédiée** (elle requiert le modèle de données manquant : saisie d'achats avec TVA OU comptes TVA + mapping). Ce gap est tracé comme **dette catégorie B** (limitation v0.2 avec story de remédiation planifiée — politique zero-tech-debt) + Issue GitHub `enhancement` à créer (cf. §Données / Issue Tracking Rule). **Pas de TVA récupérable inventée ni dérivée de façon fragile.**

## Story

As a **utilisateur d'une entreprise sur Kesh assujettie à la TVA**,
I want **que la TVA de chaque ligne de facture soit calculée avec l'arrondi commercial suisse, et pouvoir générer un rapport TVA par période (trimestriel/semestriel) exportable en PDF et CSV**,
so that **je puisse préparer mon décompte TVA (chiffre d'affaires et TVA due par taux) en cohérence avec mes factures de vente comptabilisées**.

## Scope

**Cible** : (1) un **helper de calcul TVA par ligne** réutilisable (FR55, arrondi commercial) ; (2) un **rapport TVA par période** (`kesh-report::vat_report`) agrégeant les factures de vente validées **par taux**, calculé **à la volée** (read-only, aucun changement de schéma) ; (3) **routes JSON + export PDF/CSV** calquées sur `reports.rs` ; (4) **page/onglet frontend** calqué sur la page Rapports.

### Dans le scope

1. **Helper de calcul TVA par ligne** (FR55) — nouveau module `crates/kesh-core/src/accounting/vat.rs` (le module `accounting` est de la logique métier pure sans I/O — `kesh-core/src/lib.rs`) :
   - `pub fn line_vat_amount(base_ht: Decimal, rate_percent: Decimal) -> Decimal` = `Money::new(base_ht * rate_percent / dec!(100)).round_to_centimes().amount()` — **réutilise `Money::round_to_centimes`** (DRY, half-up `MidpointAwayFromZero`). **Jamais de f64.**
   - ⚠️ **`rate_percent` est en unité POURCENT** (ex. `8.1` pour 8.1 %, comme `invoice_lines.vat_rate DECIMAL(5,2)` qui stocke `8.10` — `invoice.rs:48`), **PAS** en décimal (`0.081`). La division `/ 100` en dépend ; un appelant qui passerait `0.081` produirait une erreur silencieuse (`× 0.00081`). Le documenter dans `///` et le verrouiller par un test : `line_vat_amount(dec!(100), dec!(8.1)) == dec!(8.10)`.
   - Doc `///` : FR55, arrondi commercial AFC au centime, **par ligne** (cf. invariant ci-dessous).
   - Exporter via `kesh_core::accounting`.
2. **Rapport TVA** — nouveau module `crates/kesh-report/src/vat_report.rs` (calqué `income_statement.rs`) :
   - `pub struct VatReport { period: ReportPeriod, rows: Vec<VatReportRow>, total_base_ht: Decimal, total_vat_due: Decimal, total_vat_recoverable: Decimal, vat_balance: Decimal }` (`Serialize` camelCase).
   - `pub struct VatReportRow { rate: Decimal, category: Option<String>, base_ht: Decimal, vat_due: Decimal }` — **une ligne par taux** présent dans les ventes de la période.
   - `pub async fn generate(pool, company_id, &ReportPeriod) -> Result<VatReport, ReportError>` :
     - SQL : sélectionner les **lignes** (granularité ligne, **PAS** d'agrégation SQL — l'arrondi par ligne se fait en Rust) des factures **`status = 'validated'`** dont `invoices.date BETWEEN period.start_date AND period.end_date` et `invoices.company_id = ?` (anti-IDOR) : `SELECT il.vat_rate, il.line_total FROM invoice_lines il INNER JOIN invoices i ON i.id = il.invoice_id WHERE i.company_id = ? AND i.status = 'validated' AND i.date BETWEEN ? AND ?` (ordre `il.vat_rate` optionnel ; tri final fait en Rust). ⚠️ **Pas de `GROUP BY`/`SUM` en SQL** : sommer en SQL empêcherait l'arrondi par ligne (cf. invariant).
     - **Invariant FR55 — arrondir PAR LIGNE puis sommer** (en Rust) : itérer les lignes, pour chaque ligne `vat = accounting::vat::line_vat_amount(line_total, vat_rate)` ; accumuler dans une map `vat_rate -> (base_ht += line_total, vat_due += vat)`. ⚠️ **NE PAS** sommer les bases puis arrondir une seule fois (résultat ≠, non conforme AFC). Test obligatoire (cf. AC#3).
     - `total_base_ht` = Σ `base_ht` ; `total_vat_due` = Σ `vat_due` ; `total_vat_recoverable = Decimal::ZERO` (gap structurel, cf. note de cadrage) ; `vat_balance = total_vat_due - total_vat_recoverable`.
     - Tri des `rows` par `rate ASC` (stable, testable au niveau source comme `income_statement.rs:136`).
     - `category` (annotation d'affichage **best-effort**, optionnelle) : laisser `None` en 11-2 (le grouping primaire est **par taux**, standard du décompte ; l'inférence taux→catégorie est ambiguë et non requise). Documenter dans Dev Notes. *(Ne PAS consommer `find_for_category_at_date` ici : elle résout catégorie→taux, l'inverse n'est pas nécessaire au rapport.)*
     - Re-exports dans `crates/kesh-report/src/lib.rs` (`VatReport`, `VatReportRow`, `generate as generate_vat_report`).
3. **Sérialiseurs PDF/CSV** — `crates/kesh-report/src/pdf.rs` + `csv.rs` (calquer les renderers existants) :
   - `render_vat_report_pdf(&VatReport, &PdfContext, &VatPdfLabels) -> Result<Vec<u8>, ReportError>` : tableau par taux (Taux, CA HT, TVA due) + totaux + lignes **TVA récupérable** et **Solde** (à 0.00). Réutiliser `PdfBuilder`, `format_swiss_amount`, l'en-tête période/société. ⚠️ **NE PAS étendre `SectionLabels`** (struct `pub` à champs obligatoires consommé par les 4 rapports existants → toute extension force la modif de `fr_ch_defaults()`/`load_pdf_context` et risque la régression AC#12). À la place, **créer un struct dédié `VatPdfLabels`** (libellés TVA i18n) passé en **paramètre additionnel** à `render_vat_report_pdf` — `SectionLabels` reste intact.
   - `render_vat_report_csv<W: Write>(&VatReport, W) -> Result<(), ReportError>` : BOM UTF-8 + `;` + CRLF. En-tête : `Taux;ChiffreAffairesHT;TVADue` ; une ligne par taux (`8.10;12000.00;972.00`) ; puis lignes récapitulatives sur la même structure de colonnes (label en 1re colonne, montant en colonne TVA) : `Total TVA due;;<totalVatDue>`, `TVA récupérable;;0.00`, `Solde;;<vatBalance>` (pattern « total » des renderers existants, ex. `render_balance_sheet_csv` écrit `Total actifs;;;<montant>`). Rapport vide = en-têtes seuls (cf. `csv.rs:68-70`).
   - Re-exports `lib.rs`.
4. **Routes API** (`crates/kesh-api/src/routes/reports.rs`) — calquées `get_income_statement` / `export_income_statement` :
   - `GET /api/v1/reports/vat` → JSON `VatReport` (on-screen). Audit `report.generated` (`report_type = "vat"`).
   - `GET /api/v1/reports/vat/export?format=pdf|csv` → binaire. Audit `report.exported` (`report_type = "vat"`).
   - ⚠️ **Deux structs de query distincts** (pattern établi, `reports.rs`) : `get_vat_report` utilise `Query<ReportQuery>` (`reports.rs:136/169/202` — **PAS** de `format`, **PAS** de `validate_format`) ; `export_vat_report` utilise `Query<ExportQuery>` (`reports.rs:275/340/404/468` + `validate_format`). Réutiliser `validate_fiscal_year_id`, `ReportPeriod::resolve`, `load_pdf_context`, `render_csv_to_vec`, `resolve_type_slug(state, locale, "vat")`, `build_export_response_with_locale`.
   - Enregistrer les 2 routes dans **`authenticated_routes`** (tous rôles, lecture seule FR65), **avant le `;`** (anti-IDOR, cf. `lib.rs:495-497`). Scoping `current_user.company_id`.
5. **Frontend** — étendre la feature `reports/` + la page Rapports (réutilisation maximale) :
   - Ajouter un **onglet « TVA »** (5e onglet) à `frontend/src/routes/(app)/reports/+page.svelte` OU une page dédiée `(app)/reports/vat/` — privilégier l'onglet (réutilise sélecteur d'exercice, plage de dates, flags `loading`/`exporting`, `genSeq`, boutons PDF/CSV). Vue `VatReportView.svelte` (tableau par taux + totaux + récupérable/solde à 0).
   - `reports.api.ts` : ajouter `ReportType` `'vat'` (slug fallback `decompte-tva`), endpoint `/api/v1/reports/vat[/export]`, types dans `reports.types.ts` (`VatReport`, `VatReportRow`).
   - Montants `rust_decimal` reçus en **string** (serde-str) — formater à l'affichage, ne pas parser en `number` (perte de précision). **Pas d'API secure-context** (HTTP LAN).
6. **i18n** — clés Fluent `fr-CH` : titre rapport TVA, en-têtes colonnes (Taux, Chiffre d'affaires HT, TVA due, TVA récupérable, Solde), libellés PDF, **clé Fluent `reports-filename-vat`** de valeur `decompte-tva` (le slug de filename — résolu via `resolve_type_slug`, comme `reports-filename-balance-sheet = bilan` `fr-CH/messages.ftl:898`), message « TVA récupérable disponible dans une version ultérieure » (note d'affichage). Stubs DE/IT/EN selon `lint-i18n-ownership` (fallback FR accepté, cohérent projet).
7. **Tests** :
   - **Unitaire** `accounting::vat` : `line_vat_amount` (cas standard, midpoint half-up 123.455→123.46, montant nul, taux 0 %, négatif/avoir).
   - **Unitaire** `vat_report` : invariant **arrondi par ligne ≠ arrondi global** (cas où la différence est observable), tri par taux, totaux, `vat_balance` = due − 0.
   - **Intégration routes** (`crates/kesh-api/tests/`) : JSON + export PDF/CSV (statut 200, Content-Type, Content-Disposition), période hors exercice → 400, format invalide → 400, anti-IDOR cross-tenant (entreprise A ne voit pas les factures de B), seules les factures `validated` comptent (draft/cancelled exclues), date hors période exclue.
   - **Frontend** : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`. E2E Playwright (générer + télécharger) — **déférable** si redondant avec les tests d'intégration API (gap LOW, cf. précédent 11-1).

### Hors scope (story de suivi dédiée)

- **Comptes TVA dans le plan comptable — direction confirmée (Guy 2026-06-14) : « le compte TVA sera à implémenter ».** C'est la pièce de modèle de données manquante qui débloque tout le reste : un type/des comptes TVA dédiés (TVA due/collectée, TVA récupérable/impôt préalable, compte de décompte) dans le plan comptable. Sa mise en place est le préalable de la TVA récupérable **et** de la comptabilisation correcte de la TVA. **Story de suivi dédiée** (Issue GitHub `enhancement` à créer en T6.1).
- **TVA récupérable sur achats (impôt préalable)** — dépend des comptes TVA ci-dessus + d'une source d'achats avec TVA (saisie de factures d'achat OU lignes TVA mappées). **Story de suivi.** En 11-2, la TVA récupérable et le solde figurent dans le rapport **à 0.00** (structure prête à les recevoir).
- **Comptabilisation de la TVA dans les écritures** (lignes TVA à la validation facture sur les comptes TVA, réconciliation rapport↔grand livre AFC) — refactor du moteur comptable, dépend des comptes TVA. Hors scope 11-2.
- **Formulaire AFC officiel** (mise en page exacte du décompte officiel, e-TVA) — le rapport livré est un récapitulatif TVA par taux conforme FR56, pas le formulaire officiel.
- **Persistance de `vat_amount`/`category` sur les lignes** — décision « calcul à la volée » (Guy 2026-06-14) ; pas de migration.
- **Pré-remplissage du taux à la saisie facture via `find_for_category_at_date`** (assistant catégorie→taux) — hors scope.

## Acceptance Criteria

1. **(FR55 — calcul par ligne)** `kesh_core::accounting::vat::line_vat_amount(base_ht, rate_percent)` retourne `base_ht × rate_percent / 100` arrondi au centime en **arrondi commercial** (`MidpointAwayFromZero`, via `Money::round_to_centimes`). `rate_percent` est en **unité pourcent** (`8.1` = 8.1 %, pas `0.081`). Cas prouvés par test : `line_vat_amount(100, 8.1) == 8.10` (unité) et `123.455 → 123.46` (half-up). Calcul en `rust_decimal::Decimal`, **jamais de f64**.
2. **(FR56 — rapport par période)** `GET /api/v1/reports/vat?fiscalYearId=…[&periodStart=…&periodEnd=…]` retourne un `VatReport` JSON : pour chaque **taux** présent dans les factures de vente **validées** de la période, une ligne `{ rate, category, baseHt, vatDue }` (le grouping est **par taux** ; `category` reste `None` en 11-2 — l'inférence taux→catégorie est déférée à la story de suivi comptes TVA) ; plus `totalBaseHt`, `totalVatDue`, `totalVatRecoverable` (= `0.00`), `vatBalance` (= `totalVatDue − totalVatRecoverable`). La période est résolue par `ReportPeriod::resolve` (bornes validées dans l'exercice).
3. **(FR55 — arrondi par ligne, pas global)** Le `vat_due` par taux est la **somme des TVA arrondies ligne par ligne** (`Σ round_to_centimes(line_total × rate/100)`), **non** l'arrondi de la base agrégée. **Vecteur de test divergent fourni** (à utiliser tel quel) : 3 lignes de `line_total = 0.10` au même taux `8.00 %` → arrondi **par ligne** = `3 × round_to_centimes(0.008) = 3 × 0.01 = 0.03` ; arrondi **global** = `round_to_centimes(0.30 × 8/100) = round_to_centimes(0.024) = 0.02`. Le test vérifie que `vat_due == 0.03` (méthode par ligne), prouvant que l'implémentation ne fait PAS l'arrondi global.
4. **(Export PDF/CSV)** `GET /api/v1/reports/vat/export?format=pdf|csv&fiscalYearId=…` renvoie le rapport : PDF (`application/pdf`, en-tête société/période, tableau par taux + totaux + lignes TVA récupérable/Solde à 0.00) et CSV (`text/csv; charset=utf-8`, BOM UTF-8, `;`, CRLF, colonnes `Taux;ChiffreAffairesHT;TVADue` + totaux + récupérable + solde). `Content-Disposition` avec filename localisé (slug `decompte-tva`) RFC 5987. Format invalide → 400 ; échec génération → 500 (`AppError::PdfGenerationFailed` / `CsvGenerationFailed`).
5. **(Source de données — vente validée uniquement)** Seules les factures `status = 'validated'` avec `date` dans la période sont agrégées (les `draft`/`cancelled` sont exclues). La base par taux est la somme des `line_total` (**HT**). Vérifié par test.
6. **(Anti-IDOR / multi-tenant)** Toutes les requêtes filtrent par `current_user.company_id` ; une entreprise ne voit jamais les factures d'une autre. Test cross-tenant.
7. **(RBAC / FR65)** Les routes TVA sont en **lecture seule, accessibles à tous les rôles authentifiés** (cohérent avec les 4 rapports Epic 9 — `lib.rs:478`), enregistrées dans `authenticated_routes` **avant le `;`** (anti-bypass guards). Onboarding non requis au-delà du contexte authentifié standard des rapports.
7bis. **(TVA récupérable déférée — gap tracé)** Le rapport expose `totalVatRecoverable = 0.00` et `vatBalance = totalVatDue`. Le PDF/CSV affichent explicitement les lignes « TVA récupérable » et « Solde » (à 0). Le gap est documenté (dette cat. B + Issue GitHub `enhancement` créée pour la story de suivi achats). **Aucune valeur de récupérable inventée.**
7ter. **(Réconciliation écritures — partiellement déférée, décision tracée)** L'AC epic « les montants correspondent aux écritures comptables » (epics.md:1247) n'est **que partiellement** satisfiable en v0.2 : la **base HT par taux** réconcilie avec les crédits du compte produit (l'écriture poste le HT — `repositories/invoices.rs:1025` `let total = invoice_before.total_amount;`), mais la **TVA due ne correspond à AUCUNE écriture** car la TVA n'est jamais comptabilisée (pas de ligne TVA, pas de compte TVA — note de cadrage). Cette partie de l'AC epic est **explicitement déférée** à la story de suivi « comptes TVA » (propriétaire : Guy ; tracée dans l'Issue T6.1). 11-2 ne sur-revendique pas la réconciliation : la TVA due est **dérivée des lignes de facture**, pas lue du grand livre. À documenter dans le rapport (note d'affichage) et la story.
8. **(Audit)** Génération JSON → `report.generated` (`report_type = "vat"`) ; export → `report.exported` (`report_type = "vat"`, `format`). Via `emit_report_audit` / `emit_report_export_audit` (best-effort, non bloquant, snake_case).
9. **(Frontend)** Un onglet/page « TVA » dans Rapports : sélection exercice + plage de dates (réutilisant les contrôles existants), tableau par taux (CA HT, TVA due) + totaux + récupérable/solde (0), boutons export PDF/CSV (flag `exporting` séparé, garde anti-race). Montants `Decimal` reçus en string, formatés sans parsing `number`. Pas d'API secure-context (HTTP LAN). États chargement/erreur gérés.
10. **(i18n)** Toutes les chaînes UI/PDF nouvelles ont des clés Fluent `fr-CH` (+ clé `reports-filename-vat` de valeur `decompte-tva`) ; `npm run lint-i18n-ownership` vert. Stubs DE/IT/EN (fallback FR accepté).
11. **(Tests)** Unitaires (`accounting::vat`, `vat_report` invariant arrondi par ligne) + intégration routes (JSON, PDF, CSV, période hors exercice, format invalide, IDOR, validated-only) verts. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (serial si touche kesh-db), `cd frontend && npm run check && lint-i18n-ownership && test:unit && build` verts.
12. **(Non-régression)** Les 4 rapports Epic 9 (routes, export, frontend) restent intacts ; `PdfContext` et `SectionLabels` ne sont **pas modifiés** (les libellés TVA vivent dans un struct dédié `VatPdfLabels`, pas dans `SectionLabels`). `total_amount` HT des factures et le calcul `compute_line_total` ne sont **pas** modifiés. Côté frontend, l'ajout de l'onglet TVA n'altère pas le comportement des 4 onglets existants.

## Tasks / Subtasks

- [ ] **T1 — Helper de calcul TVA par ligne** (AC #1, #3)
  - [ ] T1.1 Nouveau module `crates/kesh-core/src/accounting/vat.rs` : `pub fn line_vat_amount(base_ht: Decimal, rate_percent: Decimal) -> Decimal` (`base × rate / 100` puis `Money::round_to_centimes`). Doc `///` (FR55, AFC, par ligne). Déclarer le sous-module dans `accounting/mod.rs` + re-export.
  - [ ] T1.2 Tests unitaires : standard, half-up `123.455→123.46`, montant 0, taux 0 %, négatif (avoir).
- [ ] **T2 — Module `vat_report` (génération)** (AC #2, #3, #5, #6, #7bis)
  - [ ] T2.1 `crates/kesh-report/src/vat_report.rs` : structs `VatReport` / `VatReportRow` (`Serialize` camelCase) + `pub async fn generate(pool, company_id, &ReportPeriod)`. SQL : lignes des factures `validated`, `date` dans la période, scopé `company_id` (anti-IDOR). Agrégation **arrondi par ligne** via `accounting::vat::line_vat_amount`, accumulée par `vat_rate`. `total_vat_recoverable = ZERO`, `vat_balance = due - 0`. Tri `rate ASC`.
  - [ ] T2.2 Re-exports `crates/kesh-report/src/lib.rs` (`VatReport`, `VatReportRow`, `generate_vat_report`).
  - [ ] T2.3 Tests unitaires : invariant arrondi par ligne ≠ global (**vecteur AC#3** : 3×`0.10` @ `8.00 %` → `vat_due == 0.03`, pas `0.02`), tri par taux, totaux, `vat_balance == total_vat_due` (récupérable 0), `category == None` pour toutes les lignes (L1). Test source-level du tri stable (style `income_statement.rs:136`).
  - [ ] T2.4 Mettre à jour le doc-comment de `find_for_category_at_date` (`crates/kesh-db/src/repositories/vat_rates.rs` ~l.84-86) qui affirme à tort « fonction que 11-2 consommera pour calculer la TVA d'une ligne » → préciser « consommée par l'assistant de saisie facture (catégorie→taux) ; le rapport TVA 11-2 groupe par `vat_rate` snapshoté et n'en a pas besoin » (OPUS-4, évite un finding de commentaire périmé en review future).
- [ ] **T3 — Sérialiseurs PDF + CSV** (AC #4, #7bis)
  - [ ] T3.1 `pdf.rs` (même fichier — `PdfBuilder` privé) : `render_vat_report_pdf(&VatReport, &PdfContext, &VatPdfLabels)` (tableau par taux + totaux + lignes récupérable/solde à 0.00), réutilise `PdfBuilder`/`format_swiss_amount`/en-tête. Libellés TVA via **struct dédié `VatPdfLabels`** (NE PAS toucher `SectionLabels` — non-régression AC#12).
  - [ ] T3.2 `csv.rs` : `render_vat_report_csv` (BOM/`;`/CRLF, colonnes `Taux;ChiffreAffairesHT;TVADue` + totaux + récupérable + solde). Répliquer (pas appeler — pas de helper partagé) la garde `if rows.is_empty()` = en-têtes seuls, comme chaque renderer existant (`render_balance_sheet_csv` etc.).
  - [ ] T3.3 Re-exports `lib.rs`.
- [ ] **T4 — Routes API + audit** (AC #2, #4, #6, #7, #8)
  - [ ] T4.1 `routes/reports.rs` (**même fichier** — `emit_report_*` privés) : `get_vat_report` (`Query<ReportQuery>`, **sans** `validate_format`, audit `report.generated`, calqué `get_income_statement` `reports.rs:166`) + `export_vat_report` (`Query<ExportQuery>` + `validate_format`, PDF/CSV, audit `report.exported`, calqué `export_income_statement` `reports.rs:337`). Réutiliser `validate_fiscal_year_id`, `ReportPeriod::resolve`, `load_pdf_context`, `render_csv_to_vec`, `resolve_type_slug(_, _, "vat")`, `build_export_response_with_locale`.
  - [ ] T4.2 Enregistrer `GET /api/v1/reports/vat` + `/api/v1/reports/vat/export` dans **`authenticated_routes`** (`lib.rs`), **avant le `;`** (anti-IDOR).
  - [ ] T4.3 Tests intégration : JSON, PDF, CSV, période hors exercice (400), **période chevauchant 2 exercices → 400** (OPUS-2), format invalide (400), IDOR cross-tenant, **validated-only** (seeder une facture `cancelled` **via SQL direct** — aucun endpoint ne crée de `cancelled`, cf. `repositories/invoices.rs:2520`), date hors période exclue, **ligne 0 %/exempt → ligne de rapport avec `base_ht > 0` et `vat_due = 0.00`** (OPUS-5).
- [ ] **T5 — Frontend onglet/page TVA** (AC #9, #10)
  - [ ] T5.1 ⚠️ **Ajouter `'vat'` à `ReportType` (`reports.types.ts:92`) casse l'exhaustivité TypeScript en plusieurs points — TOUS à mettre à jour sinon `npm run check`/`build` rouge** :
    - `reports.types.ts` : `ReportType` + `'vat'`, types `VatReport`/`VatReportRow`.
    - `reports.api.ts` : `TYPE_SLUGS_FALLBACK: Record<ReportType, string>` (`:174`) → ajouter `vat: 'decompte-tva'` ; `isReportEmpty` (`:68`) → **ajouter `| VatReport` au type union du paramètre `dto`** (`:70-75`, sinon l'appel `isReportEmpty('vat', vatReport)` est rejeté par TS → `npm run check` rouge) **ET** `case 'vat'` dans le switch (`:79`, `rows.length === 0`) ; endpoints `getReportUrl`/`getReportExportUrl` (et tout `switch`/`Record` indexé par `ReportType`).
    - `+page.svelte` : tableau `tabs` (`:228`) → 5e entrée ; `generate()` switch (`:108`) → `case 'vat'` ; `activeReportPeriod()` switch (`:143`) → `case 'vat': return vatReport?.period ?? null` (reproduire le `?? null` des cas existants — ne pas laisser tomber dans un retour implicite `undefined`) ; branche template d'affichage ; nouvelle variable d'état `vatReport`.
    - `VatReportView.svelte` (nouveau) : tableau par taux + totaux + récupérable/solde 0.
    - Réutilise les contrôles période + boutons export existants (flag `exporting`, garde anti-race `genSeq`).
  - [ ] T5.2 Montants string (pas de parsing number), états chargement/erreur, pas d'API secure-context. Guard route = cohérent rapports existants.
  - [ ] T5.3 i18n `fr-CH` (titre, colonnes, libellés PDF, `reports-filename-vat`, **note d'affichage réconciliation/récupérable — AC#7bis/#7ter**) + stubs DE/IT/EN, `lint-i18n-ownership` vert.
- [ ] **T6 — Dette tracée + vérifs finales** (AC #7bis, #11, #12)
  - [ ] T6.1 **(AC#7bis/#7ter)** Créer l'Issue GitHub `enhancement` « Comptes TVA + comptabilisation TVA + TVA récupérable sur achats » (template `feature_request.yml`, label `v0.2-milestone` — épic en cours v0.2) — story de suivi (cf. Issue Tracking Rule). Scope listé = (a) comptes TVA dans le plan comptable (direction confirmée Guy), (b) lignes TVA à la comptabilisation des factures, (c) source d'achats avec TVA, (d) TVA récupérable dans le rapport (remplit `totalVatRecoverable`/`vatBalance`), (e) réconciliation rapport↔grand livre AFC. Lier la story 11-2 dans la description (« débloque le côté récupérable de `VatReport` »). Référencer le n° d'Issue dans le Change Log. ⚠️ Création = action sortante : confirmer avec Guy avant de la créer (ou laisser le n° en TODO si non confirmé au dev).
  - [ ] T6.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (serial `-j1 -- --test-threads=1` si kesh-db/intégration).
  - [ ] T6.3 `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build`.
  - [ ] T6.4 E2E (`npm run test:e2e`) — déférable si redondant avec T4.3 (gap LOW à documenter, précédent 11-1).

## Dev Notes

### Patterns de référence (NE PAS réinventer)

- **Rapport + export** : copier `income_statement.rs` (`generate`) + `export_income_statement` (`reports.rs:337-398`). Même squelette : `validate_format` → `validate_fiscal_year_id` → `ReportPeriod::resolve` → `generate_*` → `load_pdf_context` → `match format { Pdf => render_*_pdf, Csv => render_csv_to_vec(...) }` → `emit_report_export_audit` → `resolve_type_slug` → `build_export_response_with_locale`.
- **Arrondi** : `Money::round_to_centimes()` (`money.rs:66`) est la **seule** source d'arrondi commercial — l'envelopper, ne pas réimplémenter `round_dp_with_strategy`.
- **PDF/CSV** : réutiliser `PdfBuilder`, `format_swiss_amount`, `format_swiss_date`, `make_writer`/BOM (`csv.rs:24-39`).

### Invariant FR55 — arrondir par ligne PUIS sommer (critique)

L'AFC exige l'arrondi de la TVA **par ligne**. Donc :
```
vat_due[rate] = Σ_lignes  round_to_centimes(line_total × rate / 100)
```
et **NON** `round_to_centimes( (Σ line_total) × rate / 100 )`. Les deux divergent dès que ≥2 lignes ont une partie fractionnaire au-delà du centime. Le test T2.3 doit exhiber un cas divergent (ex. trois lignes à `33.335 × 8.1%`) et prouver que l'implémentation suit l'arrondi par ligne.

### Données — d'où vient la TVA due, et le rôle (limité) de `find_for_category_at_date`

- **Source** : `invoice_lines` (`vat_rate DECIMAL(5,2)`, `line_total DECIMAL(19,4)` = HT) jointes à `invoices` (`status`, `date`, `company_id`, `total_amount` HT). Les lignes **snapshotent** `vat_rate` à la création (la modif d'un produit catalogue n'altère pas une facture — `invoice.rs:1-8`). Le rapport groupe **par `vat_rate`** stocké : pas besoin de re-résoudre la catégorie.
- **Base de calcul TVA = le `line_total` persisté (DECIMAL 19,4)**, PAS un recalcul `quantity × unit_price`. `line_total` est déjà arrondi à 4 décimales à la création (`compute_line_total = (qty*unit_price).round_dp(4)`, `repositories/invoices.rs:278`). La chaîne 4 déc. → ×taux → arrondi 2 déc. est un **double-arrondi intentionnel et AFC-acceptable** (l'erreur intermédiaire ≤ 5×10⁻⁵/ligne, négligeable au centime). Les vecteurs de test utilisent des valeurs `line_total` directes (reproductibles). NE PAS re-dériver la base depuis `quantity × unit_price`.
- **Merge par taux & 0 %/exempt** : `vat_rates.category` étant ouvert, deux catégories peuvent partager un même taux numérique à une date — elles **fusionnent dans une seule ligne** du rapport. C'est **intentionnel et conforme au décompte AFC** (qui est par taux, pas par catégorie interne). Une ligne à **0 %/exempt** apparaît comme une **ligne du rapport** (`base_ht > 0`, `vat_due = 0.00`) — le chiffre d'affaires exonéré DOIT figurer (exigence AFC). Tester au niveau ligne de rapport (cf. T4.3).
- **`find_for_category_at_date` (Story 11-1)** résout **catégorie → taux à une date** (assistant de saisie). Le rapport fait l'inverse (il a déjà le taux). **Ne pas l'utiliser ici.** L'annotation `category` du rapport reste `None` en 11-2 (inférence taux→catégorie ambiguë, non requise — le décompte standard est par taux).
- **TVA récupérable** : aucune source (pas d'achats/TVA dans le modèle, cf. note de cadrage). → `0.00`, déférée à une story de suivi. **Issue GitHub `enhancement` à créer (T6.1)** + dette cat. B documentée. NE PAS dériver de récupérable depuis les comptes de charges (fragile : aucun taux TVA attaché aux écritures).

### Période & exercice

- `ReportPeriod::resolve(pool, company_id, fiscal_year_id, period_start, period_end)` valide les bornes (start ≤ end, **chaque borne dans `[fy_start, fy_end]`** — `period.rs:33`, sinon `PeriodOutOfFiscalYear` → 400). Le SQL VAT filtre les **factures** par `i.date BETWEEN period.start_date AND period.end_date` (les factures n'ont pas de `fiscal_year_id` direct ; la fenêtre de dates suffit et reste cohérente avec l'exercice résolu). FR56 « trimestriel/semestriel » = l'utilisateur choisit la plage (presets trimestre/semestre côté UI = nice-to-have, non bloquant).
- ⚠️ **Limitation documentée v0.2 (OPUS-2)** : la période TVA est **bornée à un seul exercice** (bornage strict avec rejet `PeriodOutOfFiscalYear` → 400 par `ReportPeriod::resolve`, pas de correction silencieuse). Une entreprise dont l'exercice n'est **pas calendaire** (ex. juillet→juin) **ne peut pas** demander en un seul appel un **trimestre TVA calendaire** (ex. Q1 janv-mars) qui chevauche deux exercices → 400. Acceptable v0.2 pour les exercices calendaires (cas dominant) ; amélioration future possible (les factures n'ayant pas de `fiscal_year_id`, un futur endpoint pourrait résoudre la période par dates seules sans clamping FY). **Test** : période chevauchant la frontière d'exercice → 400 (T4.3).

### RBAC

Lecture seule, **tous rôles authentifiés** (FR65), comme les 4 rapports Epic 9 (`lib.rs:478`). ⚠️ Écart **intentionnel** vs la story 11-1 (mutations TVA = Admin) : ici on **lit** un rapport, pas on configure des taux. Routes dans `authenticated_routes` avant le `;`.

### Pièges connus

- **Arrondi global vs par ligne** : cf. invariant ci-dessus — le piège #1 de cette story.
- **f64 interdit** : tout en `Decimal`. Sérialisation JSON en **string** (serde-str) ; le frontend ne doit pas `parseFloat` (perte précision) — formater la string.
- **IDOR** : `company_id` sur **toutes** les requêtes ; routes export avant le `;` de `authenticated_routes` (sinon bypass guards, cf. `lib.rs:495-497`).
- **Factures non validées** : exclure `draft`/`cancelled` (seules `validated` sont comptabilisées). Test obligatoire.
- **Non-régression PdfContext/SectionLabels** : NE PAS toucher `SectionLabels` (struct `pub` à champs obligatoires consommé par les 4 rapports → toute extension casse `fr_ch_defaults()` + `load_pdf_context`). Les libellés TVA vont dans un struct dédié `VatPdfLabels` passé en paramètre à `render_vat_report_pdf`.
- **`total_amount` est HT** : ne pas confondre avec un TTC ; le rapport calcule la TVA en sus, ne la lit pas depuis `total_amount`.

### Règle de splitting (décision : story unique d'abord)

Cette story touche ~5 modules (`kesh-core`, `kesh-report`, `kesh-api`, `frontend`, `kesh-i18n`) — au seuil. Décision Guy 2026-06-14 : **tenter en une story**. **Fallback** si `validate` reboucle >4 passes ou si l'implémentation déborde : splitter **11-2a** (backend : `kesh-core` calc + `kesh-report` generate + tests unitaires/intégration) / **11-2b** (export PDF/CSV restants + frontend + i18n + E2E). Le contrat API se stabilise avant l'UI.

### Project Structure Notes

- `crates/kesh-core/src/accounting/vat.rs` (nouveau) + `accounting/mod.rs` (export).
- `crates/kesh-report/src/vat_report.rs` (nouveau) + `pdf.rs`/`csv.rs`/`lib.rs` (modifiés, additifs).
- `crates/kesh-api/src/routes/reports.rs` (modifié — 2 handlers) + `lib.rs` (modifié — 2 routes) + `crates/kesh-api/tests/` (nouveaux/étendus).
- `crates/kesh-i18n/locales/*/messages.ftl` (modifiés).
- `frontend/src/lib/features/reports/{reports.types.ts,reports.api.ts,VatReportView.svelte}` + `routes/(app)/reports/+page.svelte` (modifiés).

### References

- [Source: epics.md#Epic 10 (=11) Story 10.2 — Calcul TVA & rapport par période, FR55/FR56]
- [Source: crates/kesh-core/src/types/money.rs:66 — round_to_centimes (arrondi commercial AFC)]
- [Source: crates/kesh-report/src/income_statement.rs — modèle generate]
- [Source: crates/kesh-report/src/{pdf.rs,csv.rs,period.rs,lib.rs} — infra export Epic 9]
- [Source: crates/kesh-api/src/routes/reports.rs:337-398 — pattern export (income statement) ; get_* (ReportQuery) reports.rs:133-265 ; helpers privés emit_report_audit:700 / emit_report_export_audit:756]
- [Source: crates/kesh-api/src/lib.rs:478-520 — enregistrement routes rapports (authenticated_routes, anti-IDOR)]
- [Source: crates/kesh-db/src/entities/invoice.rs — Invoice/InvoiceLine (vat_rate, line_total HT, status, date)]
- [Source: crates/kesh-db/src/repositories/invoices.rs:277-279 — compute_line_total (HT, non modifié)]
- [Source: crates/kesh-db/src/repositories/vat_rates.rs:87 — find_for_category_at_date (non consommée ici)]
- [Source: frontend/src/routes/(app)/reports/+page.svelte + lib/features/reports/ — UI rapports + downloadReport]
- [Source: 11-1-configuration-taux-tva.md — infra vat_rates, catégorie extensible, gap structurel TVA écritures]
- [Source: CLAUDE.md — zero tech debt carry-forward, Issue Tracking Rule, Test Locally First, no secure-context APIs HTTP LAN]

## Change Log

### Create-story (Opus 4.8, 2026-06-14)

Spec créée. Périmètre tranché par Guy (3 décisions, AskUserQuestion) : **(1) TVA due seule** (vente) — la TVA récupérable sur achats n'a aucune source de données (pas de saisie achats/TVA, pas de comptes TVA, écritures sans TVA) → déférée à une **story de suivi** (Issue GitHub `enhancement` à créer en T6.1, dette cat. B), le rapport étant structuré au format décompte (récupérable/solde présents à 0) ; **(2) calcul à la volée** (read-only, aucun changement de schéma) ; **(3) story unique d'abord** (fallback split 11-2a/11-2b documenté).

Réutilise : `Money::round_to_centimes` (FR55), crate `kesh-report` + pattern `reports.rs` (Epic 9), `ReportPeriod::resolve`, frontend `downloadReport`. Nouveau : helper `accounting::vat::line_vat_amount`, module `kesh-report::vat_report`, renderers PDF/CSV, routes JSON+export, onglet frontend TVA. Invariant critique souligné : **arrondi TVA par ligne puis somme** (≠ arrondi global, conformité AFC). 12 ACs (+ 7bis). Ground-truth vérifié : `Money` (money.rs:66), infra report (income_statement/pdf/csv/period/lib), routes (reports.rs:337-398 export, get_* ReportQuery, lib.rs:478-520), modèle facture HT (invoice.rs, invoices.rs:277-279), gap écritures sans TVA confirmé.

Prochaine étape : `bmad-create-story validate 11-2` (Pass 1 Sonnet, cycle CLAUDE.md) — porter une attention au périmètre TVA récupérable et à l'invariant d'arrondi.

### Validate Pass 1 (Sonnet 4.6, 2026-06-14) — NON convergé

**0 CRITICAL, 3 HIGH, 5 MEDIUM, 7 LOW** — tous vérifiés ground-truth (grep -nF), tous patchés :
- **H1 HIGH** — `get_vat_report` doit utiliser `Query<ReportQuery>` (pas `ExportQuery`) : les `get_*` existants utilisent `ReportQuery` (`reports.rs:136/169/202`), seuls les `export_*` utilisent `ExportQuery` (`:275/340/404/468`). Scope item 4 + T4.1 corrigés (2 structs distincts, `validate_format` réservé à l'export).
- **H2 HIGH** — ajouter `'vat'` à `ReportType` (`reports.types.ts:92`) casse l'exhaustivité TS : `TYPE_SLUGS_FALLBACK: Record<ReportType>` (`reports.api.ts:174`) + `isReportEmpty` switch (`:79`) → `npm run check` rouge. T5.1 énumère désormais tous les points.
- **H3 HIGH** — idem côté `+page.svelte` : `tabs` (`:228`), `generate()` switch (`:108`), `activeReportPeriod()` switch (`:143`, retourner `null` pas `undefined`), variable `vatReport`. T5.1 énumère.
- **M1 MEDIUM** — réf `reports.rs:272-398` = `export_balance_sheet`+`export_income_statement` ; le modèle exact est `export_income_statement` `reports.rs:337-398`. Corrigé (References + Change Log).
- **M3 MEDIUM** — `emit_report_audit`/`emit_report_export_audit` privés (`reports.rs:700/756`) → handlers à écrire dans `reports.rs`. Précisé.
- **M4 MEDIUM** — `PdfBuilder` privé (`pdf.rs:216`) → `render_vat_report_pdf` dans `pdf.rs`. Précisé.
- **M5 MEDIUM** — `SectionLabels` (`pdf.rs:72`) `pub` à champs obligatoires → NE PAS l'étendre (régression AC#12) ; struct dédié `VatPdfLabels` en paramètre. Scope item 3 + T3.1 + AC#12 + Dev Notes corrigés.
- **M7 MEDIUM** — ambiguïté unité `rate_percent` (% vs décimal) → figé « unité pourcent `8.1` » + test `line_vat_amount(100, 8.1) == 8.10`. AC#1 + Scope item 1 + T1.1.
- **M6 + LOW (L1)** — garde CSV vide à répliquer par renderer ; `money.rs:61-71` → `money.rs:66` (uniformisé). Patchés.

NON convergé (3H/5M) → **Pass 2 requise** (Haiku, contexte frais, fichier final aplati — cf. guardrail Haiku CLAUDE.md). *(Invariant arrondi par ligne, périmètre TVA récupérable déférée, et gap structurel écritures sans TVA confirmés sains par Sonnet — pas de finding.)*

### Validate Pass 2 (Haiku 4.5, 2026-06-14) — NON convergé

**0 CRITICAL, 0 HIGH, 4 MEDIUM, 3 LOW, 0 hallucination** — Haiku a re-vérifié tout le ground-truth (Money:66, ReportPeriod:78, PdfBuilder:216 privé, SectionLabels:72 pub champs obligatoires, emit_report_*:700/756 privés, export_income_statement:337-398, routes lib.rs:478-520, ReportType:92, TYPE_SLUGS_FALLBACK:174, vat_rate invoice.rs:48, validate_format) — tous exacts, aucun faux-positif. Findings = clarté/complétude, tous patchés :
- **M1 MEDIUM** — SQL de `vat_report::generate` : préciser **pas de `GROUP BY`/`SUM` en SQL** (sinon casse l'arrondi par ligne) ; requête figée + agrégation en Rust dans une map `vat_rate -> (base, vat)`. T2.1.
- **M2 MEDIUM** — format CSV des lignes récapitulatives (récupérable/solde) non spécifié → figé : `Total TVA due;;<x>`, `TVA récupérable;;0.00`, `Solde;;<x>` (pattern « total » des renderers). Scope item 3.
- **M3 MEDIUM** — vecteur de test divergent absent (et l'exemple proposé par Haiku ne divergeait pas) → **vecteur correct fourni** : 3×`0.10` @ `8.00 %` → par ligne `0.03`, global `0.02`. AC#3 + T2.3.
- **M4/L1 MEDIUM/LOW** — `category == None` n'était que dans Dev Notes → remonté en AC#2 + test T2.3.
- **L2 LOW** — formulation `activeReportPeriod()` ré-écrite (le code existant retourne déjà `null` via `?? null` ; reproduire le pattern). T5.1.
- **L3 LOW** — vocabulaire clé Fluent vs slug clarifié (`reports-filename-vat` = clé, `decompte-tva` = valeur). Scope item 6 + AC#10.
- **L4 LOW** — scope Issue GitHub précisé (label `v0.2-milestone`, ACs a-e, lien story 11-2, confirmation Guy avant création). T6.1.

NON convergé (4M) → **Pass 3 requise** (Opus, contexte frais — la passe Opus capte les enjeux de 2nd ordre/domaine comptable, comme en 11-0/11-1). Cycle Sonnet→Haiku→Opus.

### Validate Pass 3 (Opus 4.8, 2026-06-14) — NON convergé

**0 CRITICAL, 1 HIGH, 2 MEDIUM, 2 LOW** — enjeux de **domaine comptable / 2nd ordre** ratés par Sonnet+Haiku, tous ground-truth, tous patchés :
- **OPUS-1 HIGH** — l'AC epic « les montants correspondent aux écritures comptables » (epics.md:1247) est **structurellement insatisfiable** pour la TVA (jamais comptabilisée — `invoices.rs:1025` poste le HT seul) et n'était pas reconnue comme décision déférée. → **AC#7ter** ajouté : base HT réconcilie avec le compte produit, TVA due **dérivée des lignes** (pas du grand livre), réconciliation TVA **explicitement déférée** story comptes TVA (propriétaire Guy, Issue T6.1). Pas de sur-revendication.
- **OPUS-2 MEDIUM** — période TVA bornée à 1 exercice (`period.rs:33` clamping) : un exercice non-calendaire ne peut pas demander un trimestre TVA calendaire chevauchant 2 FY → 400. **Limitation documentée v0.2** + test 400 (Dev Notes Période + T4.3).
- **OPUS-3 MEDIUM** — base TVA = `line_total` persisté (DECIMAL 19,4, déjà arrondi 4 déc. `invoices.rs:278`), PAS un recalcul `qty×unit_price` ; double-arrondi intentionnel/AFC-acceptable. Figé en Dev Notes (reproductibilité des vecteurs de test).
- **OPUS-4 LOW** — doc-comment `find_for_category_at_date` (vat_rates.rs:84-86) périmé (« 11-2 consommera ») → T2.4 le corrige.
- **OPUS-5 LOW** — merge par taux (catégories partageant un taux) + lignes 0 %/exempt : intentionnel et conforme décompte AFC (par taux) ; le CA exonéré DOIT figurer (ligne `base>0, vat=0`). Documenté + test report-row (T4.3).

Vérifs positives Opus (aucun finding) : pas d'avoirs/négatifs (`chk_invoice_lines_quantity_positive`, `line_total ≥ 0`), `cancelled` jamais créé par API (seed SQL requis pour le test), IDOR sain (`invoice_lines` sans `company_id`, scope via join `i.company_id`), arrondi `MidpointAwayFromZero` + vecteur divergent valides, tous les chemins/lignes exacts.

NON convergé (1H/2M) → **Pass 4 requise** (Sonnet, contexte frais). Cycle Sonnet→Haiku→Opus→Sonnet.

### Validate Pass 4 (Sonnet 4.6, 2026-06-14) — NON convergé

**0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW** — passe de cohérence/convergence ; patches Pass 3 re-vérifiés cohérents (AC↔Task OK, ground-truth Opus reconfirmé : `repositories/invoices.rs:1025` total_amount, `period.rs:33` bornage, `chk_invoice_lines_quantity_positive`, `invoice_lines` sans `company_id`). Tous patchés :
- **CONV-M1 MEDIUM** — T5.1 omettait la mise à jour du **type union du paramètre `dto` de `isReportEmpty`** (`reports.api.ts:70-75`) → l'appel `isReportEmpty('vat', vatReport)` aurait cassé `npm run check` (AC#11). Ajouté « `| VatReport` au type union ».
- **CONV-L1 LOW** — AC#7ter sans mapping de tâche → `(AC#7bis/#7ter)` ajouté à T5.3 + T6.1.
- **CONV-L2 LOW** — réfs ambiguës `invoices.rs` (2 fichiers homonymes) → qualifiées `repositories/invoices.rs` (AC#7ter, T4.3, Dev Notes).
- **CONV-L3 LOW** — « clamping » imprécis → « bornage strict avec rejet `PeriodOutOfFiscalYear` → 400 ».

NON convergé (1 MEDIUM) → **Pass 5 requise** (Haiku, contexte frais). Cycle Sonnet→Haiku→Opus→Sonnet→Haiku.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
