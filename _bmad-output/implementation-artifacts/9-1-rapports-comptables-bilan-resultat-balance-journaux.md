# Story 9-1: Rapports comptables (bilan, compte de résultat, balance des comptes, journaux)

Status: ready-for-dev

<!-- Première story de l'Epic 9 « Rapports & Exports ». Livre les 4 générateurs de rapports comptables
     dans la nouvelle crate `kesh-report` + les 4 routes HTTP `GET /api/v1/reports/{type}` qui exposent les données
     structurées en JSON. Le rendu PDF + l'export CSV (FR66-FR68) sont livrés par la Story 9-2.

     Source de scope : `_bmad-output/planning-artifacts/epic-9.md` §Stories → Story 9-1.
     ACs de réference : FR65 (PRD ligne 474) + équation bilan + balance débit/crédit + journaux par catégorie. -->

## Story

As a **utilisateur Kesh (Marc indépendant graphiste, Sophie trésorière association, Lisa fiduciaire)**,
I want **générer mes 4 rapports comptables réglementaires (bilan, compte de résultat, balance des comptes, journaux) à partir des écritures comptables validées d'un exercice donné, filtrables par période, avec affichage immédiat dans l'interface**,
so that **je puisse vérifier ma situation financière à n'importe quel moment, préparer ma clôture d'exercice, et fournir un état comptable conforme aux exigences de mon fiduciaire — sans dépendre d'un export PDF/CSV (livré séparément en Story 9-2)**.

### Contexte

**Story 9-1 = première story de l'Epic 9 « Rapports & Exports »**, créée 2026-05-14 après clôture Epic 8 (« Import Bancaire & Réconciliation »). Voir [`epic-9.md`](../planning-artifacts/epic-9.md) §Stories pour la spec d'épic.

**Séparation 9-1 / 9-2 (verrouillée par epic-9.md)** :

- **9-1 (cette story)** livre la **génération de données** : 4 modules de calcul dans la crate `kesh-report` (actuellement coquille vide, cf. `crates/kesh-report/src/lib.rs`), 4 routes HTTP qui retournent du **JSON structuré** uniquement, et la page frontend qui rend ces données en HTML/Tailwind avec les formats suisses (apostrophe, dd.mm.yyyy) **côté frontend** (helper `formatSwissAmount` depuis `$lib/features/journal-entries/balance.ts`). **Pass 2 BH2-02** : `kesh-i18n` n'est PAS dépendance de `kesh-report` en 9-1 (formatage Rust reporté 9-2 PDF).
- **9-2 (suivante)** livre **PDF + CSV + export global** : rendu PDF tabulaire (lib à confirmer, `printpdf 0.7` candidat par défaut car déjà utilisé par `kesh-qrbill`), export CSV par rapport, et export ZIP global par table (FR66-FR68).

**Cette séparation est délibérée** : elle permet à 9-1 d'être revue/mergée sans dépendre du choix de la librairie PDF (R3 epic-9.md) ni de la performance du rendu (R1). Le JSON est la **surface stable** pour 9-2 et pour les Epics 11 (TVA), 13 (Budgets), 14 (Clôture) qui réutilisent les agrégations.

**Crate cible** : extension de [`crates/kesh-report`](../../crates/kesh-report/) (coquille vide actuellement — `lib.rs` contient un seul commentaire `//! Crate placeholder — implementation in subsequent stories.` et `Cargo.toml` n'a aucune dep). 9-1 instancie 4 modules métier purs + entités + Cargo deps.

**Dépendances closed (au démarrage 9-1)** :

- ✅ Story 3-1 — `accounts` (account_type ∈ `Asset|Liability|Revenue|Expense`).
- ✅ Story 3-2 — `journal_entries` + `journal_entry_lines` (avec `journal ∈ Achats|Ventes|Banque|Caisse|OD`, `fiscal_year_id`, `entry_date`, `debit`/`credit` DECIMAL(19,4)).
- ✅ Story 3-7 — `fiscal_years` (status `Open|Closed`, helpers `find_by_id_in_company`, `list_by_company`).
- ✅ Story 2-1 — `kesh-i18n` (`format_money` apostrophe U+2019, `format_date` dd.mm.yyyy, 4 locales FR/DE/IT/EN-CH).
- ✅ Story 7-1 (KF-002) — pattern multi-tenant scoping (`company_id` filtré dans toutes les requêtes, cross-tenant = 404).
- ✅ Story 5-2 — pattern audit log atomique (`audit_log::insert_in_tx`).
- ✅ Story 6-3 — i18n key ownership lint (`npm run lint-i18n-ownership`).

**Aucun blocker connu**. Verdict rétro Epic 8 §🔍 Readiness Epic 9 : « aucune dette architecturale d'Epic 8 ne bloque Epic 9 ».

**Recherche réglementaire Swiss CO Art. 957a (R2 epic-9.md, action item #3 retro Epic 8)** :

R2 est documentée dans `epic-9.md` comme « à faire avant spec validate 9-1 — owner Guy ». Cette spec liste les **questions ouvertes Q1-Q5** à trancher à la passe `bmad-create-story validate` (avec sources réglementaires citées si Guy a fait la recherche entre-temps). Voir §Décisions de conception → Q1-Q5 ci-dessous + §Risques R2-R7.

### Scope verrouillé — ce qui est livré par 9-1

1. **Crate `kesh-report` instanciée (FR65)** — `Cargo.toml` avec deps **versions directes** (Pass 1 BH-03 + Pass 2 BH2-02 : pas de `kesh-i18n` en 9-1) : `chrono`, `rust_decimal`, `serde`, `thiserror`, `sqlx` (feature `mysql` + `chrono` + `rust_decimal` + `runtime-tokio-rustls`), `kesh-core`, `kesh-db`. Structure :

   ```
   crates/kesh-report/
   ├── Cargo.toml                    *(extension : ajout deps)*
   └── src/
       ├── lib.rs                    *(refactor : pub use modules)*
       ├── balance_sheet.rs          *(nouveau, T2 — Bilan)*
       ├── income_statement.rs       *(nouveau, T3 — Compte de résultat)*
       ├── trial_balance.rs          *(nouveau, T4 — Balance des comptes)*
       ├── journal_report.rs         *(nouveau, T5 — Journaux)*
       ├── period.rs                 *(nouveau, T1 — ReportPeriod commun)*
       └── errors.rs                 *(nouveau, T1 — ReportError)*
   ```

2. **Module `period.rs` (commun, T1)** — type `ReportPeriod { fiscal_year_id: i64, start_date: NaiveDate, end_date: NaiveDate }` + helper `ReportPeriod::resolve(pool, company_id, fiscal_year_id, period_start?, period_end?)` (Pass 1 BH-08 : nom unique `resolve`, le nom `from_query` est retiré) qui :
   - Vérifie que `fiscal_year_id` appartient à `company_id` via `fiscal_years::find_by_id_in_company(pool, company_id, fiscal_year_id)` (Pass 1 BH-02 : ordre params `(pool, company_id, id)` cohérent avec `crates/kesh-db/src/repositories/fiscal_years.rs:399`). 404 `FISCAL_YEAR_NOT_FOUND` si None.
   - **Pass 1 ECH-02** : période asymétrique acceptée — table de résolution :
     - `(None, None)` → `(fy.start_date, fy.end_date)` (exercice complet).
     - `(Some(s), None)` → `(s, fy.end_date)` (depuis s jusqu'à fin fy).
     - `(None, Some(e))` → `(fy.start_date, e)` (du début fy jusqu'à e).
     - `(Some(s), Some(e))` → `(s, e)` après validations.
   - Valide bornes incluses dans fy : `s ≥ fy.start_date` ET `e ≤ fy.end_date` (400 `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` sinon).
   - Valide `s ≤ e` (400 `REPORT_PERIOD_INVALID` sinon — message error code, pas string FR — cf. Pass 1 AA-05).
   - **Pass 1 ECH-06 + Pass 3 BH3-01** : la validation `fiscal_year_id > 0` se fait **handler-side** (T6.3) avant l'appel à `resolve` — un check explicite `if fiscal_year_id <= 0 { return Err(AppError::Validation(format!("fiscalYearId must be > 0"))); }` (note Pass 3 BH3-01 : `AppError::Validation(String)` est **tuple variant** existant, PAS struct variant — cohérent pattern `journal_entries.rs:259`). Ne pas s'appuyer sur le serde i64 parsing qui accepte 0 et négatifs sans erreur.

3. **Module `balance_sheet.rs` (T2, FR65 bilan)** :
   - Fonction publique `pub async fn generate(pool: &MySqlPool, company_id: i64, period: &ReportPeriod) -> Result<BalanceSheet, ReportError>`.
   - Output struct `BalanceSheet { period, assets: Vec<AccountBalance>, liabilities: Vec<AccountBalance>, equity_result: Decimal, total_assets: Decimal, total_liabilities: Decimal, equation_holds: bool }`.
   - Algorithme : agrège **`COALESCE(SUM(debit), 0) - COALESCE(SUM(credit), 0)`** (Pass 1 ECH-01 : SUM sur 0 ligne retourne NULL en SQL — `COALESCE` à 0 protège du type mismatch sqlx) pour les comptes `account_type IN ('Asset','Liability')` filtrés par `entry_date BETWEEN period.start_date AND period.end_date AND fiscal_year_id = period.fiscal_year_id AND company_id = ?`. Pour les passifs, le solde « naturel » est `credit - debit`. `equity_result = total_revenues - total_expenses` calculé conjointement.
   - **Pass 1 ECH-04** : la table `accounts.account_type` (CHECK BINARY) n'a **PAS** de variant `Equity` — les comptes de capitaux propres réels (compte 2800/2979 Sterchi PME) sont déclarés en `Liability`. **L'équation v0.1 admet que `total_liabilities` inclut les fonds propres permanents** (report à nouveau, capital social, etc.) ET `equity_result` ne représente **que** le résultat de l'exercice courant. Cette présentation est **approximative pré-clôture** : la clôture Epic 14 produira l'écriture qui transfère `equity_result` vers le compte 2979 (résultat reporté). **Documenter explicitement dans le rendu frontend** : « Résultat de l'exercice (avant clôture) ». L52-bis dette tracée.
   - **Équation bilan** : `equation_holds = (total_assets == total_liabilities + equity_result)`. Tolérance 0 (rust_decimal exact, pas de tolérance float). **Pass 1 ECH-04 nuance** : sur dataset équilibré avec capitaux propres en `Liability` + résultat séparé `equity_result`, l'équation tient mathématiquement (les soldes des comptes de capitaux propres permanents sont déjà dans `total_liabilities`). Vérifier ground-truth via fixture seed T2.5.
   - **Pass 1 ECH-01 cas vide** : si la période n'a aucune écriture, retourne `BalanceSheet { assets: vec![], liabilities: vec![], total_assets: 0, total_liabilities: 0, equity_result: 0, equation_holds: true }` (0 == 0 + 0). Pas d'erreur.
   - **Pass 2 AA2-11 — règle d'inclusion archived (cohérente trial_balance Pass 1 ECH-03)** :
     - Comptes Asset/Liability **actifs sans écriture dans la période** : **exclus** des listes `assets`/`liabilities` (épure visuelle bilan — différent de trial_balance qui les inclut). Justification : le bilan affiche les positions financières effectives ; un compte actif sans solde n'est pas une position.
     - Comptes Asset/Liability **archivés avec écritures dans la période** : **inclus** avec `active: false` (cohérent CO 957a conservation 10 ans).
     - Comptes Asset/Liability **archivés sans écriture dans la période** : exclus.
     - SQL T2.2 utilise donc `WHERE jel.account_id IN (SELECT id FROM accounts WHERE company_id = ? AND account_type IN ('Asset','Liability'))` puis agrège — les comptes sans écriture sortent naturellement de l'agrégation. Filtre archived `(active=true OR EXISTS écritures)` appliqué pour décider d'**afficher ou non** un compte avec écritures non-zéro.
   - **Pass 3 ECH3-01 — exclusion comptes Equity-like du total_liabilities (CRITICAL fonctionnel)** :
     Les plans comptables seed (`crates/kesh-core/assets/charts/{pme,independant,association}.json`) contiennent le compte **2979 « Résultat de l'exercice »** type `Liability`. Si un user saisit une écriture vers 2979 pré-clôture (rien ne l'interdit DB-side v0.1), `total_liabilities` inclut le solde 2979 ET `equity_result` est calculé indépendamment → **double-comptage qui casse l'équation bilan**.
     **Solution v0.1** : exclure ces comptes du calcul `total_liabilities` au niveau du SELECT SQL :
     ```rust
     // crates/kesh-report/src/balance_sheet.rs
     /// Numéros de comptes qui représentent sémantiquement de l'equity-result
     /// et qui sont calculés séparément via `equity_result` (PAS via SUM Liability).
     /// Plan Sterchi PME standard : 2979 (Bénéfice/Perte exercice), 2800 (Bénéfice/Perte reporté).
     pub const EQUITY_RESULT_ACCOUNT_NUMBERS: &[&str] = &["2979", "2800"];
     ```
     Le SQL T2.2 ajoute `AND a.number NOT IN ('2979','2800')` au filtre des passifs. Test T2.5 dédié `manual_entry_to_2979_does_not_double_count_in_equity_result` (fixture : écriture vers 2979 + écritures Revenue/Expense, vérifier `equation_holds: true`).
     L65-bis dette tracée : si une PME utilise un plan comptable non-Sterchi avec d'autres numéros pour le résultat, configurable via `kesh-core/chart_of_accounts` v0.2 (CR si signalé).
   - AC #1 — équation bilan vérifiée dans tous les cas du test fixture.

4. **Module `income_statement.rs` (T3, FR65 compte de résultat)** :
   - Fonction publique `pub async fn generate(pool, company_id, period) -> Result<IncomeStatement, ReportError>`.
   - Output struct `IncomeStatement { period, revenues: Vec<AccountBalance>, expenses: Vec<AccountBalance>, total_revenues: Decimal, total_expenses: Decimal, net_result: Decimal }`.
   - Algorithme : agrège par compte pour `account_type IN ('Revenue','Expense')` avec **`COALESCE(SUM(...), 0)`** (Pass 1 ECH-01). Revenue : `credit - debit`. Expense : `debit - credit`. `net_result = total_revenues - total_expenses`.
   - **Pass 1 AA-01** : SQL `ORDER BY a.account_number ASC` (tri stable comme balance_sheet, cohérent AC #3).

5. **Module `trial_balance.rs` (T4, FR65 balance des comptes)** :
   - Fonction publique `pub async fn generate(pool, company_id, period) -> Result<TrialBalance, ReportError>`.
   - Output struct `TrialBalance { period, rows: Vec<TrialBalanceRow>, total_debit: Decimal, total_credit: Decimal, balanced: bool }`.
   - `TrialBalanceRow { account_id: i64, account_number: String, account_name: String, account_type: AccountType, active: bool, total_debit: Decimal, total_credit: Decimal, balance: Decimal }` (balance = solde net signé selon convention type).
   - **Pass 1 ECH-03 — règle d'inclusion (3 cas)** :
     - Comptes **actifs** (`active=true`) : **inclus dans tous les cas**, même sans écriture dans la période (affichés avec `total_debit=0, total_credit=0, balance=0`). Justification : convention CO 957a balance comptable = vue complète du plan comptable, pas seulement les comptes mouvementés.
     - Comptes **archivés avec écritures dans la période** : inclus avec `active: false` (marqueur).
     - Comptes **archivés sans écriture dans la période** : exclus (épure visuelle).
   - **Pass 1 ECH-14 — hiérarchie `parent_id`** : v0.1 traite **tous les comptes comme des feuilles** (pas de sous-totaux par groupe parent). Si un compte parent a été mouvementé directement (anomalie de plan comptable), il apparaît avec ses montants ; sous-totaux par groupe parent → v0.2 (drill-down).
   - SQL : `WHERE a.company_id = ? AND (a.active = TRUE OR EXISTS (SELECT 1 FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id WHERE jel.account_id = a.id AND je.company_id = ? AND je.entry_date BETWEEN ? AND ? AND je.fiscal_year_id = ?))`. Agrégation `COALESCE(SUM(jel.debit), 0)` / `COALESCE(SUM(jel.credit), 0)` (Pass 1 ECH-01). `ORDER BY a.account_number ASC` (tri stable AC #2 + AC #3).
   - **Invariant** : `balanced = (total_debit == total_credit)`. Si false → erreur métier `ReportError::TrialBalanceUnbalanced` (théoriquement impossible si `journal_entries` est correct, mais défense en profondeur — log `error!` + retour HTTP 500).
   - AC #4 — `total_debit == total_credit` testé sur fixture multi-écritures.

6. **Module `journal_report.rs` (T5, FR65 journaux)** :
   - **Pass 1 BH-10** : `use kesh_db::entities::journal_entry::Journal` (PAS `kesh_core::accounting::Journal` — manque traits sqlx).
   - Fonction publique `pub async fn generate(pool, company_id, period, journal_filter: Option<Journal>) -> Result<JournalReport, ReportError>`.
   - Output struct `JournalReport { period, journals: Vec<JournalSection>, grand_total_debit: Decimal, grand_total_credit: Decimal }`.
   - `JournalSection { journal: Journal, entries: Vec<JournalEntryRow>, section_total_debit: Decimal, section_total_credit: Decimal }`.
   - `JournalEntryRow { entry_id, entry_number, entry_date, description, lines: Vec<JournalEntryLineRow> }` avec `JournalEntryLineRow { account_id, account_number, account_name, debit, credit, line_order }`.
   - **Pass 1 ECH-05** : si `journal_filter = None` → **TOUJOURS 5 sections** (Achats, Ventes, Banque, Caisse, OD) dans cet ordre fixe, **même si un journal n'a aucune écriture** dans la période (la section apparaît avec `entries: vec![], section_total_debit: 0, section_total_credit: 0`). Le frontend rend 5 onglets stables ; pas de gap visuel.
   - Si `journal_filter = Some(j)` → 1 seule section.
   - Algorithme : SELECT `journal_entries je JOIN journal_entry_lines jel ON jel.entry_id = je.id JOIN accounts a ON a.id = jel.account_id` filtré par company + période + (optionnel) journal, **avec `COALESCE(SUM(...), 0)`** côté agrégats `section_total_debit`/`section_total_credit`/`grand_total_*` (Pass 1 ECH-01). ORDER BY `je.journal, je.entry_date ASC, je.entry_number ASC, jel.line_order ASC`. **Groupement applicatif** côté Rust pour construire les 5 sections fixes (initialiser un `BTreeMap<Journal, JournalSection>` puis itérer les rows).

7. **Module `errors.rs` (T1)** — `pub enum ReportError { Db(DbError), FiscalYearNotFound, PeriodInvalid { reason: String }, PeriodOutOfFiscalYear { fy_start, fy_end, requested_start, requested_end }, TrialBalanceUnbalanced { total_debit, total_credit } }` avec impl `From<DbError>`.

8. **Routes API `kesh-api/routes/reports.rs` (nouveau, T6)** :
   - `GET /api/v1/reports/balance-sheet?fiscalYearId={id}&periodStart={dd.mm.yyyy|YYYY-MM-DD}&periodEnd={...}` — retourne JSON `BalanceSheetDto` (camelCase serde rename_all).
   - `GET /api/v1/reports/income-statement?fiscalYearId&periodStart&periodEnd` — `IncomeStatementDto`.
   - `GET /api/v1/reports/trial-balance?fiscalYearId&periodStart&periodEnd` — `TrialBalanceDto`.
   - `GET /api/v1/reports/journals?fiscalYearId&periodStart&periodEnd&journal={Achats|Ventes|Banque|Caisse|OD}` — `JournalReportDto`. `journal` query param optionnel.
   - **RBAC** : tous les rôles authentifiés (`authenticated_routes` dans `lib.rs`) peuvent lire (consultation seule, pas de mutation).
   - **Multi-tenant** : `company_id = current_user.company_id` extrait du JWT (pattern KF-002 Pattern 1). **Toutes les requêtes filtrent par `company_id`** — cross-tenant = 404.
   - **Validation query params** : `fiscalYearId` obligatoire i64 > 0 (400 `Validation` sinon). Dates en `YYYY-MM-DD` ISO 8601 (cf. architecture.md §Format Patterns ligne 355). Si parsing échoue → 400 `Validation`. Si `periodStart`/`periodEnd` absents → période complète de l'exercice.
   - **Pas de pagination v0.1** : les rapports retournent **tout** sur la période. Cf. §Risques R1.
   - **Pas de side-effects** : ces endpoints sont en lecture pure, **aucune écriture DB** sauf audit log si Q3 décide « audit chaque génération ».

9. **Frontend `frontend/src/lib/features/reports/` + routes (T8)** :
   - `reports.types.ts` — types TypeScript miroir des DTO (BalanceSheetDto, IncomeStatementDto, TrialBalanceDto, JournalReportDto, AccountBalance, etc.) avec montants `string` (cf. architecture.md ligne 356 : « Montants : string décimal, jamais de float »).
   - `reports.api.ts` — 4 fetchers `getBalanceSheet(params)`, `getIncomeStatement(params)`, `getTrialBalance(params)`, `getJournalReport(params)` + helper `isReportEmpty(reportType, dto)` (Pass 1 AA-11 + BH-11) qui retourne `true` selon la condition unifiée par type :
     - `BalanceSheet` : `assets.length === 0 && liabilities.length === 0`
     - `IncomeStatement` : `revenues.length === 0 && expenses.length === 0`
     - `TrialBalance` : `rows.length === 0`
     - `JournalReport` : `journals.every(j => j.entries.length === 0)`
   - **Pass 1 BH-06 — formatage suisse côté frontend** : utiliser **`formatSwissAmount(big: Big)`** depuis `$lib/features/journal-entries/balance.ts:93` (PAS `formatting.ts` — fichier inexistant). Pour la cohérence Pattern, créer `$lib/features/reports/reports-helpers.ts` qui réexporte `formatSwissAmount` + `formatSwissDate` (à créer si nécessaire — wrapper Intl.DateTimeFormat ou manuel `dd.mm.yyyy`).
   - `ReportSelector.svelte` — UI : sélecteur d'exercice (chargé via `GET /api/v1/fiscal-years` → **route existante** livrée Story 3-7, vérifiée Pass 2 AA2-03 dans `crates/kesh-api/src/lib.rs:321-322` mountée dans `authenticated_routes`), date range picker (period start/end optionnels), bouton « Générer ». **Pass 1 ECH-12 + AC #34** : si la liste fiscal_years est vide, dropdown vide + bouton `disabled` + message `reports-error-no-fiscal-year-available`.
   - `BalanceSheetView.svelte`, `IncomeStatementView.svelte`, `TrialBalanceView.svelte`, `JournalReportView.svelte` — 4 vues qui affichent le rapport en tableau HTML Tailwind, montants formatés via `formatSwissAmount` (apostrophe `1'234.56`) et dates via wrapper `dd.mm.yyyy`.
   - **Pass 1 ECH-11 — equity_result négatif** : `BalanceSheetView` affiche `equityResult` dans une section dédiée « Résultat de l'exercice (avant clôture) ». Si négatif → libellé « Perte de l'exercice », couleur rouge (Tailwind `text-red-600`). Si positif → « Bénéfice de l'exercice », vert. Si zéro → neutre. Ne PAS confondre avec total liabilities.
   - **Pass 1 ECH-22 — formatage variables Fluent** : pour le message `reports-error-period-out-of-fiscal-year`, le composant Svelte qui consomme l'erreur 400 reçoit `error.details.fyStart` / `fyEnd` en ISO 8601 (`"2026-12-31"`). Avant passage à `$t('reports-error-period-out-of-fiscal-year', { fyStart: ..., fyEnd: ... })`, **convertir** via `formatSwissDate` pour afficher `"31.12.2026"`. Documenté dans T8.3.
   - `routes/(app)/reports/+page.svelte` — page hôte qui mount `ReportSelector` + chargement à la demande des 4 vues selon onglet actif. **Pass 1 AC #33** : exactement 4 éléments `role="tab"` rendus avec labels i18n.
   - `routes/(app)/reports/+page.ts` — load function pour charger la liste des fiscal_years.
   - Pas de bouton « Télécharger PDF » ni « Export CSV » v0.1 (livré 9-2). Tracé L52.

10. **i18n (T9)** — **~30 nouvelles clés Fluent** × 4 locales (fr/de/it/en-CH) sous le préfixe `reports-` :
    - Labels rapports : `reports-balance-sheet`, `reports-income-statement`, `reports-trial-balance`, `reports-journals` (4 clés).
    - Colonnes : `reports-column-account-number`, `reports-column-account-name`, `reports-column-debit`, `reports-column-credit`, `reports-column-balance`, `reports-column-entry-date`, `reports-column-description` (7 clés).
    - Sections bilan : `reports-section-assets`, `reports-section-liabilities`, `reports-section-equity`, `reports-section-revenues`, `reports-section-expenses` (5 clés).
    - Totaux : `reports-total-assets`, `reports-total-liabilities`, `reports-total-revenues`, `reports-total-expenses`, `reports-total-debit`, `reports-total-credit`, `reports-net-result`, `reports-grand-total` (8 clés).
    - Filtres : `reports-filter-period`, `reports-filter-fiscal-year`, `reports-filter-journal`, `reports-button-generate` (4 clés).
    - Erreurs UX : `reports-error-no-entries-in-period`, `reports-error-period-out-of-fiscal-year`, `reports-error-no-fiscal-year-available` (3 clés UX-DR38 — Pass 1 ECH-12 ajoute la 3e pour AC #34).
    - Section résultat (Pass 1 ECH-11) : `reports-equity-result-profit`, `reports-equity-result-loss`, `reports-equity-result-section-title` (3 clés).
    - **Total cible : 34 clés** (Pass 1 : 30 → 34). Lint `npm run lint-i18n-ownership` doit passer. Cf. §Risques R4. **Pass 1 ECH-19** : chaque clé IT/DE/EN-CH **DOIT** avoir une valeur texte (traduction basique acceptable + commentaire `# TODO official translation` en ligne précédente). Une clé sans valeur fait que Fluent retourne la clé brute à l'UI (`reports-balance-sheet` au lieu de "Bilanz").

11. **Audit log (T7 — décision Q3 pré-spec)** — option par défaut (à confirmer validate) : 1 action `report.generated` émise par chaque GET réussi avec `details_json = { reportType: 'balance-sheet'|'income-statement'|'trial-balance'|'journals', fiscalYearId, periodStart, periodEnd, journalFilter? }`. Justification : CO Art. 958f audit trail des consultations comptables. **Alternative** : audit uniquement sur génération PDF/CSV (= en Story 9-2). Décision Q3 à trancher validate Pass 1.

12. **Tests** :
    - **Unit `kesh-report`** (`#[cfg(test)] mod tests` inline dans chaque module) — ≥ 16 tests : balance_sheet (4 : équation, archivés, période partielle, multi-fiscal-year-isolation), income_statement (3 : résultat net positif/négatif/zéro), trial_balance (4 : balanced=true, balanced=false ground truth, account ordering, totaux), journal_report (5 : 5 journaux, filter Achats, ordering, vide, multi-line entries).
    - **Intégration `kesh-db`** (`#[sqlx::test]` dans nouveau fichier `crates/kesh-db/tests/report_aggregates.rs`) — ≥ 7 tests : multi-tenant cross-company (4 cross-tenant 0-rows), period filter (1 boundary), fiscal_year isolation (1).
    - **E2E HTTP `kesh-api`** (`crates/kesh-api/tests/reports_e2e.rs` nouveau) — ≥ 28 tests : 4 endpoints × { happy path, 401 unauth, 400 validation, 404 cross-tenant, 400 period out of FY } = 20 minimum.
    - **Vitest frontend** (`frontend/src/lib/features/reports/*.test.ts`) — ≥ 4 tests : ReportSelector validation, BalanceSheetView formatting montants, formatage suisse apostrophe, période par défaut = exercice complet.
    - **Playwright E2E** (`frontend/tests/e2e/reports.spec.ts` nouveau) — ≥ 1 actif (générer bilan sur exercice seed) + 1 axe a11y scan zero violations sur la page `/reports`.

**HORS scope 9-1 (→ Story 9-2 ou v0.2 ou jamais) :**

- **Export PDF de chaque rapport** → Story 9-2 (FR66, FR67).
- **Export CSV de chaque rapport** → Story 9-2 (FR66).
- **Export global ZIP par table** → Story 9-2 (FR68).
- **Bouton « Télécharger » dans le menu principal** → Story 9-2.
- **FR69 (recherche écritures par montant/libellé/numéro/date)** — non scope 9-1. Mappé `kesh-report/` dans architecture.md §17 mais sémantiquement plus proche d'Epic 3 (saisie écritures). **Décision Guy en epic-9.md R5** : laisser FR69 hors Epic 9, tracker via CR si re-scopé.
- **FR70 (rapport personnalisable / drill-down)** — non scope v0.1. Reporté v0.2/Epic 15 sauf demande explicite (R6 epic-9.md).
- **FR81 (personnalisation modèles : logo, footer)** — Epic 15 v0.2.
- **Rapports TVA** — Epic 11.
- **Comparatif budget vs réalisé** — Epic 13.
- **Rapport de clôture / report des soldes** — Epic 14.
- **Drill-down depuis bilan/résultat vers les écritures** — v0.2 (UX souhaitable mais pas v0.1).
- **Multi-currency reports** — v0.2 (cohérent L38 héritée 8-4 mono-CHF).
- **Pagination ou streaming pour très gros exercices > 50k lignes** — v0.2 (R1 epic-9.md). En v0.1 on charge tout en mémoire et retourne en un seul JSON. Si exercice > 10k écritures observé en prod → CR.

### Décisions de conception

#### Q1 — Affichage du résultat de l'exercice au bilan (impact §balance-sheet)

**Question** : au bilan v0.1, le « résultat de l'exercice en cours » (= net_result du compte de résultat) doit-il être :

- **Option A — Affiché comme ligne dédiée** dans une section « Capitaux propres » à droite du bilan (`equity_result` champ dédié du struct `BalanceSheet`). L'équation devient `total_assets = total_liabilities + equity_result`.
- **Option B — Confondu dans les passifs sous une ligne « Bénéfice/Perte de l'exercice »** (compte 2979 standard Sterchi). Équation devient `total_assets = total_liabilities` (le résultat est dans total_liabilities).
- **Option C — Non affiché v0.1** (rapporté manuellement par l'utilisateur via écriture de clôture Epic 14). Équation `total_assets = total_liabilities` brut, vérification fiscale uniquement post-clôture.

**Décision par défaut spec** : **Option A** (ligne `equity_result` séparée + équation vérifiée incluant `equity_result`). Justification :
- Lisible : l'utilisateur Marc/Sophie voit immédiatement son résultat en bas du bilan.
- N'exige pas d'écriture de clôture pour produire un bilan en cours d'exercice.
- Compatible Epic 14 (la clôture pourra reverser le `net_result` vers le compte 2979 réel via une écriture).

**À confirmer validate Pass 1** : si Guy a fait la recherche R2 epic-9.md (Swiss CO Art. 957a + Sterchi PME), valider que la présentation respecte les exigences fiduciaire. Sinon, garder Option A et créer GitHub Issue CR si la recherche fait apparaître un format légal différent.

#### Q2 — Comptes archivés (active=false) avec écritures dans la période

**Question** : la balance des comptes (`trial_balance`) doit-elle afficher les comptes archivés qui ont des écritures dans la période ?

**Décision** : **Oui, affichés** avec un marqueur visuel (`active: false` exposé dans `TrialBalanceRow`). Justification :
- CO Art. 957-964 conservation 10 ans — un compte archivé garde son historique comptable.
- Un solde non-zéro sur un compte archivé doit être signalé (anomalie possible : compte archivé en cours d'exercice sans solde de clôture).
- Le frontend affiche une ligne grisée + badge « Archivé ».
- Les comptes archivés **sans aucune écriture dans la période** ne sont **pas** affichés (épure visuelle).

**Filter SQL** : `WHERE a.company_id = ? AND (a.active = TRUE OR EXISTS (SELECT 1 FROM journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id WHERE jel.account_id = a.id AND je.company_id = ? AND je.entry_date BETWEEN ? AND ? AND je.fiscal_year_id = ?))`. Cf. T4 SQL spec.

#### Q3 — Audit log sur génération de rapport (R7 + retro Epic 8 action item)

**Question** : chaque appel `GET /api/v1/reports/{type}` doit-il créer une ligne audit log ?

**Décision validate Pass 1 — TRANCHÉE** (Pass 1 AA-13) : **OUI**, action `report.generated` avec `entity_type = 'report'`, `entity_id = AUDIT_ENTITY_ID_NONE (0)` (sentinelle — Pass 1 BH-01), `details_json = { reportType, fiscalYearId, periodStart, periodEnd, journalFilter? }`. Justification :
- CO Art. 958f audit trail des consultations.
- Coût : 1 INSERT audit_log par request (négligeable).
- Permet retracer « qui a vu quel rapport quand » pour conformité fiduciaire.

**Alternative refusée** : audit uniquement sur PDF/CSV (9-2). La consultation JSON est la première brique de l'audit trail — l'export PDF est un sur-coût après.

**Décision révisable post-validate** : si la recherche réglementaire R2 epic-9.md (owner Guy) révèle que seul l'export persistant compte comme « consultation soumise à audit », créer un CR GitHub explicite et basculer en 9-2 (pas de modification silencieuse).

**Pattern d'implémentation** : audit émis **après** succès du SELECT (pas avant — pas d'audit sur erreur 400/404/500). Insert audit dans une transaction dédiée (1 INSERT atomique). Si l'INSERT audit échoue → log `warn!`, ne pas faire échouer la réponse rapport (best-effort audit pour la consultation, distinct de l'audit transactionnel des mutations comptables qui DOIT bloquer la mutation si l'audit échoue).

#### Q4 — Format de date dans les query params HTTP

**Question** : `periodStart` / `periodEnd` en query param HTTP : format ISO 8601 (`YYYY-MM-DD`) ou suisse (`dd.mm.yyyy`) ?

**Décision** : **ISO 8601 (`YYYY-MM-DD`) côté API**. Justification :
- Architecture.md ligne 355 : « Dates : ISO 8601 (`2026-04-02`...) ».
- Frontend convertit `dd.mm.yyyy` (saisie UI) → `YYYY-MM-DD` (envoi API) — pattern KE déjà standard pour invoices, journal_entries.
- Évite ambiguïté locale (FR/DE/EN).
- **Affichage** dans les rapports : `dd.mm.yyyy` via wrapper frontend `formatSwissDate` (Pass 2 BH2-02 — pas de `kesh-i18n::format_date` côté Rust en 9-1).

#### Q5 — Ordre des colonnes balance des comptes (compatibilité fiduciaire)

**Question** : ordre standard ? Convention Sterchi PME = `Numéro | Nom | Débit | Crédit | Solde` ou `... | Solde débiteur | Solde créditeur` séparés ?

**Décision par défaut spec** : **Colonnes uniques `Débit | Crédit | Solde`** (un compte a un solde signé). Justification :
- Plus compact.
- Le frontend peut séparer visuellement par couleur (vert/rouge) selon le signe.
- Si fiduciaire exige le format double-colonne soldes → CR post-validate (re-scopable v0.2 ou via FR81 Epic 15 modèles documents).

**À valider validate Pass 1** : si R2 recherche Sterchi PME → format double-colonne, basculer en 2 colonnes soldes.

### §api-routes — Conventions endpoints

Toutes les 4 routes suivent ce pattern strict, hérité des conventions architecture.md §Naming Patterns (kebab-case routes, camelCase query params) :

**Endpoint** : `GET /api/v1/reports/{type}` où `{type} ∈ {balance-sheet, income-statement, trial-balance, journals}`.

**Query params** :

| Param | Type | Obligatoire | Validation | Erreur si invalide |
|---|---|---|---|---|
| `fiscalYearId` | i64 > 0 | OUI | parse i64, > 0, FK valide pour `company_id` | 400 `Validation` ou 404 `FiscalYearNotFound` |
| `periodStart` | YYYY-MM-DD | non | parse `NaiveDate`, ≥ `fy.start_date` | 400 `Validation` ou 400 `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` |
| `periodEnd` | YYYY-MM-DD | non | parse `NaiveDate`, ≤ `fy.end_date`, `≥ periodStart` | 400 idem |
| `journal` (journals only) | enum Achats\|Ventes\|Banque\|Caisse\|OD | non | parse via `Journal::from_str` | 400 `Validation` |

**RBAC** : `authenticated_routes` (Admin + Comptable + Consultation). Pas de mutation possible → consultation seule.

**Multi-tenant** : `company_id = current_user.company_id` extrait du JWT par l'extractor `CurrentUser`. **Toutes les requêtes filtrent par `company_id`** systématiquement. Cross-tenant `fiscal_year_id` = 404 (pattern KF-002 Pattern 1).

**Codes HTTP** :

| Situation | Code |
|---|---|
| Succès | 200 + JSON body |
| Auth manquante / JWT invalide | 401 |
| Validation (parse query, period out of FY, etc.) | 400 |
| fiscal_year_id inexistant pour ce tenant | 404 `FiscalYearNotFound` |
| Trial balance unbalanced (DB corrompue) | 500 `Internal` + log error |
| DB down | 500 |

**Response shape** : JSON direct (pas d'enveloppe), camelCase via `#[serde(rename_all = "camelCase")]`.

### §rust-types — Types publics `kesh-report`

```rust
// crates/kesh-report/src/lib.rs

pub mod balance_sheet;
pub mod errors;
pub mod income_statement;
pub mod journal_report;
pub mod period;
pub mod trial_balance;

pub use balance_sheet::{BalanceSheet, generate as generate_balance_sheet};
pub use errors::ReportError;
pub use income_statement::{IncomeStatement, generate as generate_income_statement};
pub use journal_report::{JournalReport, JournalSection, generate as generate_journal_report};
pub use period::ReportPeriod;
pub use trial_balance::{TrialBalance, TrialBalanceRow, generate as generate_trial_balance};

// crates/kesh-report/src/period.rs

use chrono::NaiveDate;
use serde::Serialize;

// Pass 1 BH-05 : Serialize requis car BalanceSheet/IncomeStatement/TrialBalance/JournalReport
// dérivent Serialize et contiennent `pub period: ReportPeriod`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPeriod {
    pub fiscal_year_id: i64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl ReportPeriod {
    /// Résout la période effective d'un rapport.
    ///
    /// Pass 1 ECH-02 : si **un seul** de `period_start`/`period_end` est fourni,
    /// l'autre est dérivé des bornes de l'exercice (asymétrie OK, pas d'erreur 400).
    /// Si les deux sont absents → période = exercice complet.
    /// Si les deux sont présents → bornes valides + ordre `start ≤ end`.
    ///
    /// Pass 1 BH-02 : signature `find_by_id_in_company(pool, company_id, id)`
    /// (PAS `(pool, id, company_id)` — voir `crates/kesh-db/src/repositories/fiscal_years.rs:399`).
    pub async fn resolve(
        pool: &sqlx::MySqlPool,
        company_id: i64,
        fiscal_year_id: i64,
        period_start: Option<NaiveDate>,
        period_end: Option<NaiveDate>,
    ) -> Result<Self, ReportError> { /* ... */ }
}

// crates/kesh-report/src/balance_sheet.rs

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheet {
    pub period: ReportPeriod,
    pub assets: Vec<AccountBalance>,
    pub liabilities: Vec<AccountBalance>,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub equity_result: Decimal,
    pub equation_holds: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub account_id: i64,
    pub account_number: String,
    pub account_name: String,
    pub account_type: kesh_db::entities::AccountType,  // Pass 1 BH-15 : cohérence avec TrialBalanceRow
    pub active: bool,
    pub balance: Decimal,
}

pub async fn generate(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
) -> Result<BalanceSheet, ReportError> { /* ... */ }
```

(Patterns équivalents pour `income_statement.rs`, `trial_balance.rs`, `journal_report.rs` — voir §scope §3-§6.)

**Pass 1 BH-10** : pour `journal_report.rs`, **utiliser `Journal` de `kesh-db`** (`use kesh_db::entities::journal_entry::Journal`) qui a les traits `Type<MySql>`, `Encode<MySql>`, `Decode<MySql>` requis par sqlx. **NE PAS utiliser** `kesh_core::accounting::Journal` (enum pur sans traits sqlx — `crates/kesh-core/src/accounting/balance.rs:25`).

**Sérialisation `Decimal`** (Pass 1 BH-04) : utiliser `#[serde(with = "rust_decimal::serde::str")]` (sans wrapper externe). Ce path est activé par la feature `serde-str` de `rust_decimal = "1.41"` (cf. T1.1 ci-dessus). **NE PAS utiliser** `serde_with::DisplayFromStr` ni la feature `serde-with-str` (inexistante dans `rust_decimal`).

### §audit-shapes — Pattern audit log 9-1

Une seule nouvelle action audit : `report.generated`.

**Pass 1 BH-01 / ECH-09 — sentinelle `entity_id`** : le schéma DB `audit_log.entity_id BIGINT NOT NULL` (cf. `crates/kesh-db/migrations/20260413000001_audit_log.sql:18`) + struct `NewAuditLogEntry.entity_id: i64` (non-Option, cf. `crates/kesh-db/src/entities/audit_log.rs`) → impossible de passer `None`. **Décision** : utiliser **la valeur sentinelle `0`** (BIGINT positif, distinct de tout `entity_id` réel car `AUTO_INCREMENT` commence à 1). **Pas de migration DB v0.1** (audit_log conservation 10 ans CO 957-964 — éviter ALTER COLUMN sur table d'audit). Définir constante publique :

```rust
// Pass 2 ECH2-01 + ECH2-10 + AA2-04 : définir dans `crates/kesh-db/src/entities/audit_log.rs`
// (PAS dans kesh-report ni kesh-api) pour réutilisabilité trans-crate v0.2+ et éviter
// la duplication. Toutes les futures actions audit sans entité concrète (Epic 14 clôture,
// Epic 15 exports, etc.) utiliseront cette même constante.
/// Sentinelle `entity_id` pour les actions audit sans entité concrète
/// (rapports, consultations agrégées, exports, etc.).
///
/// Garantie d'unicité sémantique : les `id` réels d'entités sont en `AUTO_INCREMENT`
/// qui démarre à 1 — `0` ne correspond à aucune entité réelle.
///
/// **IMPORTANT — utilisation correcte** : pour distinguer plusieurs actions audit
/// avec `entity_id = 0`, **toujours filtrer sur la combinaison `(entity_type, entity_id)`**,
/// jamais sur `entity_id` seul. Exemple :
///   SELECT * FROM audit_log WHERE entity_type = 'report' AND entity_id = AUDIT_ENTITY_ID_NONE;
pub const AUDIT_ENTITY_ID_NONE: i64 = 0;
```

**Vérification ground-truth** (Pass 2 ECH2-01) : `audit_log.id BIGINT AUTO_INCREMENT PRIMARY KEY` dans migration `20260413000001` — démarre bien à 1, donc `entity_id = 0` ne peut jamais correspondre à une row existante. Pas de risque de collision avec entités réelles.

**Pass 3 ECH3-02 — re-export obligatoire dans `entities/mod.rs`** :

Pour que les consumers puissent utiliser `use kesh_db::entities::AUDIT_ENTITY_ID_NONE`, ajouter au Source tree §kesh-db le fichier `crates/kesh-db/src/entities/mod.rs` (modif 1 ligne) :

```rust
// crates/kesh-db/src/entities/mod.rs (modification)
pub use audit_log::{AuditLogEntry, NewAuditLogEntry, AUDIT_ENTITY_ID_NONE};  // Pass 3 ECH3-02 : re-export
```

Sans cette ligne, le compilateur force `use kesh_db::entities::audit_log::AUDIT_ENTITY_ID_NONE` (chemin pleine submodule) — fonctionnel mais incohérent avec le pattern `AuditLogEntry`/`NewAuditLogEntry` déjà re-exportés au niveau `entities::*`.

**Future-proof** : si Epic 14 (clôture) émet `audit.action = 'fiscal_year.closed'` avec `entity_type = 'fiscal_year'` ET `entity_id = fy.id` (entité réelle), aucune collision. Si Epic 15 (exports) émet `audit.action = 'export.zip.generated'` avec `entity_type = 'export'` ET `entity_id = AUDIT_ENTITY_ID_NONE`, le filtre par `entity_type` distingue parfaitement des audits rapports. Pattern propre, scalable.

Shape de l'audit row :

```rust
NewAuditLogEntry {
    user_id: current_user.user_id,
    action: "report.generated".to_string(),
    entity_type: "report".to_string(),
    entity_id: AUDIT_ENTITY_ID_NONE,              // Pass 1 BH-01 : sentinelle 0
    details_json: Some(serde_json::json!({
        "reportType": "balance-sheet",            // ou income-statement, trial-balance, journals
        "fiscalYearId": 12,
        "periodStart": "2026-01-01",              // ISO 8601 string
        "periodEnd": "2026-12-31",
        "journalFilter": null                     // ou "Achats", "Ventes", etc. pour journals
    })),
}
```

**Pass 1 AA-03 / BH-14 — multi-tenant scoping de l'audit** : `audit_log` **n'a PAS de colonne `company_id`** (cf. migration `20260413000001`). Le scope tenant se fait via `user_id` → `users.company_id`. AC #18 (reformulé) teste cette propriété via JOIN `audit_log JOIN users ON audit_log.user_id = users.id WHERE users.company_id = ?`. **La sécurité multi-tenant est garantie côté lecture** par les routes qui filtrent toujours par `company_id` extrait du JWT — un user company B ne peut pas lire l'audit company A même si la table audit_log elle-même n'a pas de scope direct.

**Atomicité best-effort (Pass 1 ECH-15)** : audit inséré dans une **mini-transaction dédiée** (1 INSERT) **après** le SELECT métier réussi. Le pattern code OBLIGATOIRE (T7.1) :

```rust
// Dans le handler (après generate_balance_sheet réussi) :
let report = kesh_report::generate_balance_sheet(&state.pool, company_id, &period).await?;

// Pass 1 ECH-15 : best-effort — match exhaustif, JAMAIS de `?` qui bubble-up.
match emit_report_audit(
    &state.pool,
    current_user.user_id,
    "balance-sheet",
    period.fiscal_year_id,
    period.start_date,
    period.end_date,
    None,
).await {
    Ok(_) => {}
    Err(e) => tracing::warn!(error = ?e, "audit insert failed (report.generated) — non-blocking"),
}

Ok(Json(report))
```

**Pass 1 AA-04 — stabilité tests E2E AC #25** : pour éviter le flake (audit best-effort + test assert 1 row), le test E2E `report_generated_audit_emitted_on_success` (T10.2 #20) doit :
- Utiliser `spawn_app(pool)` avec **pool de connexion test ≥ 4** (suffisant pour SELECT métier + INSERT audit simultanés).
- Lire l'audit via `SELECT FROM audit_log WHERE user_id = ? AND action = 'report.generated' ORDER BY id DESC LIMIT 1` (pas de race avec d'autres tests parallèles si --test-threads=1).
- Si le test devient flake malgré ces précautions → marquer L62 dette test et créer KF GitHub.

### §error-shapes — Mapping `ReportError → AppError`

| `ReportError` variant | `AppError` variant | HTTP | Code métier exposé |
|---|---|---|---|
| `Db(e)` | `AppError::Db(e)` | 500 | `INTERNAL_ERROR` |
| `FiscalYearNotFound` | `AppError::ReportFiscalYearNotFound` | 404 | `FISCAL_YEAR_NOT_FOUND` |
| `PeriodInvalid { reason }` | `AppError::Validation(reason)` (tuple variant) | 400 | `VALIDATION_ERROR` |
| `PeriodOutOfFiscalYear { .. }` | `AppError::ReportPeriodOutOfFiscalYear { fy_start, fy_end, requested_start, requested_end }` (struct variant nouveau) | 400 | `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` |
| `TrialBalanceUnbalanced { total_debit, total_credit }` | `AppError::Internal` (log error!) | 500 | `INTERNAL_ERROR` |

3 nouveaux variants `AppError` à ajouter dans `crates/kesh-api/src/errors.rs` :

- `ReportFiscalYearNotFound`
- `ReportPeriodOutOfFiscalYear { fy_start: NaiveDate, fy_end: NaiveDate, requested_start: NaiveDate, requested_end: NaiveDate }`
- (`PeriodInvalid { reason }` réutilise le variant existant `AppError::Validation(String)` **tuple variant** — Pass 3 BH3-01 ground-truth `crates/kesh-api/src/errors.rs:65-66`)

`From<ReportError> for AppError` impl factorise le mapping.

#### Shape JSON body 400 `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` (Pass 1 AA-02 + BH-09 + Pass 2 AA2-01 + Pass 3 BH3-03)

**Pass 3 BH3-03 — divergence assumée du pattern ErrorBody standard** : le `ErrorBody { error: ErrorDetail }` + `ErrorDetail { code, message }` existant (`crates/kesh-api/src/errors.rs:469-478`) **n'a PAS de champ `details`**. Refactor lourd écarté (hors scope 9-1). **Solution adoptée** : ce variant **UNIQUEMENT** émet un body JSON ad-hoc via `serde_json::json!(...)` qui inclut `details`, **divergent** du pattern `build_response`. Documenter explicitement dans le code que c'est intentionnel + tracer L68 dette « refactor `ErrorBody` pour supporter `details: Option<Value>` v0.2 ».

**Convention camelCase** cohérente avec architecture.md §Format Patterns ligne 355 :

```json
{
  "error": {
    "code": "REPORT_PERIOD_OUT_OF_FISCAL_YEAR",
    "message": "La période sélectionnée dépasse les bornes de l'exercice.",
    "details": {
      "fyStart": "2026-01-01",
      "fyEnd": "2026-12-31",
      "requestedStart": "2026-01-01",
      "requestedEnd": "2027-01-15"
    }
  }
}
```

Les 4 champs (`fyStart`, `fyEnd`, `requestedStart`, `requestedEnd`) sont **toujours présents** (camelCase ISO 8601 string, **jamais omis ni null** — Pass 2 AA2-05). Le test AC #13 (T10.2 #13) vérifie ground-truth les **4 champs**.

**Pass 2 AA2-01 — IntoResponse mapping snake_case Rust → camelCase JSON (CRITICAL)** :

Le variant `AppError::ReportPeriodOutOfFiscalYear { fy_start, fy_end, requested_start, requested_end }` (snake_case Rust) doit être sérialisé en `{ fyStart, fyEnd, requestedStart, requestedEnd }` (camelCase JSON). **Pattern recommandé** : utiliser un DTO intermédiaire avec `#[serde(rename_all = "camelCase")]` dans `kesh-api/src/errors.rs::IntoResponse impl` :

```rust
// crates/kesh-api/src/errors.rs (extension T6.4) — Pass 3 BH3-03 + BH3-07

use chrono::NaiveDate;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PeriodOutOfFyDetails {
    fy_start: NaiveDate,         // sérialisé "fyStart"
    fy_end: NaiveDate,           // sérialisé "fyEnd"
    requested_start: NaiveDate,  // sérialisé "requestedStart"
    requested_end: NaiveDate,    // sérialisé "requestedEnd"
}

// Extension de l'impl IntoResponse existante pour AppError (pattern ad-hoc divergent du build_response standard)
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            // Pass 3 BH3-03 : variant unique avec body JSON ad-hoc (divergent du build_response standard
            // car ErrorBody existant n'a pas de champ `details`). Documenter L68 dette refactor v0.2.
            AppError::ReportPeriodOutOfFiscalYear { fy_start, fy_end, requested_start, requested_end } => {
                let details = PeriodOutOfFyDetails { fy_start, fy_end, requested_start, requested_end };
                let body = serde_json::json!({
                    "error": {
                        "code": "REPORT_PERIOD_OUT_OF_FISCAL_YEAR",
                        "message": "La période sélectionnée dépasse les bornes de l'exercice.",
                        "details": details,
                    }
                });
                (StatusCode::BAD_REQUEST, axum::Json(body)).into_response()
            }
            // ReportFiscalYearNotFound { fiscal_year_id } : mono-champ, pas besoin de DTO
            AppError::ReportFiscalYearNotFound { fiscal_year_id } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "FISCAL_YEAR_NOT_FOUND",
                        "message": "Exercice comptable introuvable pour cette company.",
                        "details": { "fiscalYearId": fiscal_year_id },
                    }
                });
                (StatusCode::NOT_FOUND, axum::Json(body)).into_response()
            }
            // Tous les autres variants (Validation, Db, etc.) : pattern existant build_response (ErrorBody sans details)
            // ... (cohérent journal_entries.rs etc.)
        }
    }
}
```

**Justification** : sans ce DTO intermédiaire, `serde_json::json!(...)` directement à partir des champs Rust snake_case produirait `{ fy_start, fy_end, ... }` en JSON — incohérent avec la convention architecture.md camelCase et casse AC #13.

**Pattern pour les 3 nouveaux AppError variants** (Pass 3 BH3-03 clarification) :
- `ReportPeriodOutOfFiscalYear { fy_start, fy_end, requested_start, requested_end }` : **ad-hoc JSON via `serde_json::json!()` + DTO intermédiaire camelCase** (divergent build_response).
- `ReportFiscalYearNotFound { fiscal_year_id }` : ad-hoc JSON minimal (1 champ déjà camelCase).
- `Validation(String)` (tuple variant existant — Pass 3 BH3-01) : **PAS modifié** — utilise le pattern existant `build_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &msg)` (`crates/kesh-api/src/errors.rs:513`). Le `String` interne devient le `message`. Pas de `details`.

**Pass 3 BH3-07 — pattern emit_report_audit (T7.1)** :

```rust
// crates/kesh-api/src/routes/reports.rs

async fn emit_report_audit(
    pool: &sqlx::MySqlPool,
    user_id: i64,
    report_type: &str,
    fiscal_year_id: i64,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
    journal_filter: Option<&str>,
) -> Result<(), kesh_db::errors::DbError> {
    let mut tx = pool.begin().await?;
    kesh_db::repositories::audit_log::insert_in_tx(
        &mut tx,
        kesh_db::entities::audit_log::NewAuditLogEntry {
            user_id,
            action: "report.generated".to_string(),
            entity_type: "report".to_string(),
            entity_id: kesh_db::entities::AUDIT_ENTITY_ID_NONE,  // Pass 3 ECH3-02 : re-export entities/mod.rs
            details_json: Some(serde_json::json!({
                "reportType": report_type,
                "fiscalYearId": fiscal_year_id,
                "periodStart": period_start.format("%Y-%m-%d").to_string(),
                "periodEnd": period_end.format("%Y-%m-%d").to_string(),
                "journalFilter": journal_filter,
            })),
        },
    ).await?;
    tx.commit().await?;
    Ok(())
}
```

#### Shape JSON body 400 `VALIDATION` (cohérent stories antérieures)

```json
{
  "error": {
    "code": "VALIDATION",
    "message": "<message technique pour debugging, non destiné UX final>",
    "details": { "reason": "<raison structurée, ex. 'fiscalYearId > 0'>" }
  }
}
```

**Pass 1 AA-05** : les ACs `#14`, `#19`, `#20`, `#21`, `#22` ne doivent **PAS** asserter sur un texte FR précis dans la response. **Asserter uniquement sur `error.code = "VALIDATION_ERROR"` + le `details.reason` structuré.** Le formatage UX user-friendly se fait **côté frontend** via Fluent (clés `reports-error-*` existantes ou clés génériques validation déjà disponibles `error-validation-*`). Cohérent pattern Epic 8.

#### Shape JSON body 404 `FISCAL_YEAR_NOT_FOUND`

```json
{
  "error": {
    "code": "FISCAL_YEAR_NOT_FOUND",
    "message": "Exercice comptable introuvable pour cette company.",
    "details": { "fiscalYearId": 42 }
  }
}
```

### §api-routes-validation — Mécanisme de validation params (Pass 1 ECH-06 + ECH-13)

Dans chaque handler (T6.3), **avant** d'appeler `ReportPeriod::resolve` :

```rust
// Pass 1 ECH-06 : check > 0 explicite (serde i64 accepte 0 et négatifs sans erreur)
if query.fiscal_year_id <= 0 {
    return Err(AppError::Validation(format!("fiscalYearId must be > 0")));  // Pass 3 BH3-01 tuple variant
}
```

**Pass 1 ECH-13 + Pass 3 BH3-12 — overflow i64 et `QueryRejection` (design)** :

Axum 0.8 `Query<T>` rejection par défaut retourne **`text/plain` 400** (cf. pattern `journal_entries.rs:240-244` documenté « comportement par défaut intentionnel »). Si on adopte ce default :
- Les ACs #19-#22 ne peuvent PAS asserter `error.code = "VALIDATION_ERROR"` car le body est text/plain (pas JSON).
- Les tests E2E doivent asserter `status = 400` + `content-type contains "text/plain"` (cohérent stories antérieures).

**Décision Pass 3 BH3-12 — adopter le default Axum** : pas de `handle_query_rejection` custom. Cohérent avec le pattern existant des autres endpoints. **Reformuler ACs #19-#22** : assertion sur **`status = 400`** uniquement, pas sur `error.code` (le body est text/plain, pas JSON). L68 dette tracée pour uniformisation JSON v0.2.

Pour les **erreurs métier internes** (validation post-parsing, ex. `fiscalYearId <= 0`), le handler retourne explicitement `AppError::Validation(format!("..."))` qui passe par `build_response` → `"VALIDATION_ERROR"` JSON (AC #20-bis testable).

**Pass 1 ECH-08 — case-sensitivity `journal` query param** : le `Journal` enum de `kesh-db` dérive `Deserialize` avec les variants exacts `Achats|Ventes|Banque|Caisse|OD` (PascalCase, sensible à la casse). `journal=achats` → 400 `Validation`. AC #22 message du `details.reason` doit inclure les 5 valeurs acceptées avec leur casse : `"journal must be one of: Achats, Ventes, Banque, Caisse, OD (case-sensitive)"`.

**Pass 1 ECH-20 — param dupliqué (`journal=Achats&journal=Ventes`)** : Axum `Query<T>` via `serde_qs` prend la **dernière** occurrence. Comportement déterministe non-bloquant ; ne pas chercher à le « corriger » v0.1.

### §i18n-keys — 34 nouvelles clés `reports-*`

Cible spec : **34 clés** × 4 locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`) — Pass 3 BH3-10 propagation correcte.

Liste nominale (référence — la conformité ownership est vérifiée par `npm run lint-i18n-ownership`) :

```fluent
# Labels rapports (4)
reports-balance-sheet = Bilan
reports-income-statement = Compte de résultat
reports-trial-balance = Balance des comptes
reports-journals = Journaux

# Colonnes (7)
reports-column-account-number = N° de compte
reports-column-account-name = Intitulé
reports-column-debit = Débit
reports-column-credit = Crédit
reports-column-balance = Solde
reports-column-entry-date = Date
reports-column-description = Libellé

# Sections (5)
reports-section-assets = Actifs
reports-section-liabilities = Passifs
reports-section-equity = Capitaux propres
reports-section-revenues = Produits
reports-section-expenses = Charges

# Totaux (8)
reports-total-assets = Total actifs
reports-total-liabilities = Total passifs
reports-total-revenues = Total produits
reports-total-expenses = Total charges
reports-total-debit = Total débit
reports-total-credit = Total crédit
reports-net-result = Résultat net
reports-grand-total = Total général

# Filtres (4)
reports-filter-period = Période
reports-filter-fiscal-year = Exercice
reports-filter-journal = Journal
reports-button-generate = Générer

# Erreurs UX (3 — Pass 2 BH2-01)
reports-error-no-entries-in-period = Aucune écriture dans la période sélectionnée. Modifiez les dates ou choisissez un autre exercice.
reports-error-period-out-of-fiscal-year = La période sélectionnée dépasse les bornes de l'exercice. Choisissez une période entre { $fyStart } et { $fyEnd }.
reports-error-no-fiscal-year-available = Aucun exercice comptable disponible. Créez un exercice avant de générer des rapports.

# Section résultat de l'exercice (3 — Pass 1 ECH-11 + Pass 2 BH2-01)
reports-equity-result-section-title = Résultat de l'exercice (avant clôture)
reports-equity-result-profit = Bénéfice de l'exercice
reports-equity-result-loss = Perte de l'exercice
```

**Décompte ground-truth** : 4 labels rapports + 7 colonnes + 5 sections + 8 totaux + 4 filtres + 3 erreurs UX + 3 equity-result = **34 clés** × 4 locales.

**Traductions DE/IT/EN-CH** : version basique livrée par dev-story, traducteur officiel reporté v0.2 (cohérent limitation L51 héritée 8-5b). Marquer `# TODO official translation` en commentaire Fluent pour DE/IT/EN-CH.

### §performance — Cible v0.1

**Cible** : génération JSON d'un rapport < **500 ms** sur dataset de référence (~1000 écritures × 2-5 lignes = ~3000 lines totales). Pas de cible PDF en 9-1 (livré 9-2 avec cible `< 3s` epic-9.md §critères-arrêt).

**Mesure** : test E2E `reports_perf_smoke` (1 test marqué `#[ignore]` par défaut, activable `--ignored`) qui mesure le temps de `generate_balance_sheet` sur seed CI (à enrichir si nécessaire). **Pas de strict pass/fail v0.1** — observation only, dette R1 si > 500 ms. **Pass 1 AA-15** : le test perf_smoke (#[ignore]) **DOIT être activé manuellement** sur le dataset seed CI avant la soumission code review. Le reviewer doit vérifier le log de timing et s'assurer qu'aucun N+1 query n'a été introduit (`EXPLAIN` sur les SQL agrégats au minimum sur balance_sheet).

**Optimisations v0.1** :

- 1 seul SELECT par rapport (pas de N+1). Utiliser JOINs.
- Index existants : `idx_journal_entries_company_date` + `idx_journal_entries_fiscal_year` + `idx_jel_entry` + `idx_jel_account` (cf. migration 20260412 ligne 33-34, 50-51). Vérifier `EXPLAIN` au dev-story.
- Pas de cache v0.1 (DB-direct).

**Limites connues** (LXX dette si dépassées) : > 10k écritures par exercice → JSON > 5 Mo, latence potentielle > 1s. Acceptable v0.1 → v0.2 pagination/streaming via Story 9-X.

## Acceptance Criteria

### Crate `kesh-report` (FR65)

1. **AC #1 — Bilan, équation comptable** — Given un exercice avec écritures validées équilibrées, when `GET /api/v1/reports/balance-sheet?fiscalYearId={fy}` est appelé, then la response JSON contient `assets`, `liabilities`, `equityResult`, `totalAssets`, `totalLiabilities`, `equationHolds: true` et `totalAssets == totalLiabilities + equityResult` (égalité Decimal exacte).

2. **AC #2 — Bilan, ordre des classes** — Given un bilan généré, when les `assets` et `liabilities` sont énumérés, then chaque liste est triée par `accountNumber` ASC (ordre lexical, ex. `1000` avant `1010`, `2000` avant `2100`).

3. **AC #3 — Compte de résultat** — Given un exercice avec écritures sur des comptes Revenue/Expense, when `GET /reports/income-statement?fiscalYearId={fy}`, then `netResult == totalRevenues - totalExpenses` (Decimal exact) et chaque section (`revenues`, `expenses`) est triée par `accountNumber` ASC. **Pass 1 AA-01** : le tri `accountNumber ASC` est garanti par `ORDER BY a.account_number ASC` dans la query T3.2 et testé via T3.4 test `net_result_ordering_by_account_number`.

4. **AC #4 — Balance des comptes, équilibre** — Given un exercice avec écritures validées, when `GET /reports/trial-balance?fiscalYearId={fy}`, then `totalDebit == totalCredit` (Decimal exact) et `balanced: true`.

5. **AC #5 — Balance avec compte archivé non-zéro** — Given un compte archivé (`active=false`) avec des écritures dans la période, when GET trial-balance, then le compte apparaît dans `rows` avec `active: false` et son solde calculé.

6. **AC #6 — Balance sans compte archivé zéro** — Given un compte archivé sans aucune écriture dans la période, when GET trial-balance, then le compte n'apparaît PAS dans `rows`.

7. **AC #7 — Journaux, 5 sections par défaut** — Given un exercice avec écritures dans les 5 journaux (Achats, Ventes, Banque, Caisse, OD), when `GET /reports/journals?fiscalYearId={fy}`, then la response contient `journals: [...]` avec exactement 5 `JournalSection` dans cet ordre fixe : Achats, Ventes, Banque, Caisse, OD.

8. **AC #8 — Journaux, filtre par journal** — Given un exercice multi-journaux, when `GET /reports/journals?fiscalYearId={fy}&journal=Achats`, then la response contient 1 seule `JournalSection` de journal `Achats`.

9. **AC #9 — Journaux, ordre des entries** — Given un journal avec plusieurs écritures, when GET journals, then `entries` est trié par `entryDate ASC, entryNumber ASC` (chronologique strict).

10. **AC #10 — Journaux, ordre des lignes intra-écriture** — Given une écriture multi-lignes, when GET journals, then `lines` est trié par `lineOrder ASC` (préserve la saisie utilisateur).

### Filtrage période & exercice (FR65)

11. **AC #11 — Période par défaut = exercice complet** — Given `periodStart` et `periodEnd` absents en query, when GET rapport, then la période utilisée est `[fy.start_date, fy.end_date]` (exercice entier).

12. **AC #12 — Période partielle** — Given `periodStart=2026-04-01&periodEnd=2026-06-30` (un trimestre dans l'exercice), when GET rapport, then seules les écritures avec `entry_date BETWEEN 2026-04-01 AND 2026-06-30` sont incluses dans les agrégations.

13. **AC #13 — Période out of fiscal year** — Given un exercice 2026-01-01 → 2026-12-31, when GET rapport avec `periodEnd=2027-01-15` (hors borne fy), then response 400 avec `error.code = "REPORT_PERIOD_OUT_OF_FISCAL_YEAR"` et **les 4 champs** `error.details = { fyStart: "2026-01-01", fyEnd: "2026-12-31", requestedStart: "2026-01-01", requestedEnd: "2027-01-15" }` (Pass 1 AA-02 : shape JSON précise, cf. §error-shapes).

14. **AC #14 — Période inversée** — Given `periodStart=2026-06-30&periodEnd=2026-04-01`, when GET rapport, then response 400 avec `error.code = "VALIDATION_ERROR"` et `error.details.reason` non-vide (texte technique structurel, Pass 1 AA-05 — assertion sur le code uniquement, pas le texte). Le frontend mappe ce code sur la clé Fluent UX appropriée.

15. **AC #15 — Multi-exercices isolation** — Given une company avec exercices `fy1=2025` et `fy2=2026`, when `GET /reports/balance-sheet?fiscalYearId={fy1}`, then seules les écritures avec `fiscal_year_id = fy1` sont agrégées (aucune écriture fy2 ne contamine le bilan fy1).

### Multi-tenant (KF-002 Pattern 1)

16. **AC #16 — Cross-tenant fiscal_year** — Given un user authentifié de company A et un `fiscal_year_id` qui appartient à company B, when GET rapport avec ce `fiscalYearId`, then response 404 `FISCAL_YEAR_NOT_FOUND` (jamais 403 — pattern KF-002).

17. **AC #17 — Cross-tenant aggregation** — Given 2 companies avec écritures, when user company A GET trial-balance, then aucune écriture company B n'apparaît dans `rows` (filtre `WHERE company_id = ?` systématique).

18. **AC #18 — Audit log scoped via user** — Given un audit `report.generated` émis par user A (company A), when query SQL `SELECT al.* FROM audit_log al JOIN users u ON al.user_id = u.id WHERE al.action = 'report.generated' AND u.company_id = ?` avec `companyId = A`, then la row audit existe pour user A et **n'apparaît pas** quand le filtre est `companyId = B` (Pass 1 AA-03 + BH-14 : `audit_log` n'a PAS de colonne `company_id` directe — le scope tenant se fait via JOIN `users`).

### Validation params

19. **AC #19 — fiscalYearId obligatoire** — Given GET rapport sans `fiscalYearId` query param, then response **status = 400** (body `text/plain` Axum default — Pass 3 BH3-12). Pas d'assertion sur `error.code` JSON.

20. **AC #20 — fiscalYearId malformé** — Given GET avec `fiscalYearId=abc` (non-numérique), then status = 400 (body `text/plain` Axum default).

20-bis. **AC #20-bis — fiscalYearId ≤ 0** (Pass 1 ECH-06 + Pass 3 BH3-12) — Given GET avec `fiscalYearId=0` OU `fiscalYearId=-1`, then 400 avec **`content-type: application/json`** (validation post-parsing handler-side, passe par `build_response`) et `error.code = "VALIDATION_ERROR"`, `error.message` contient `"fiscalYearId must be > 0"`. Distinct des ACs #19/#20 qui sont rejetés en parsing (text/plain).

21. **AC #21 — Date malformée** — Given `periodStart=2026/01/15` (format non-ISO) OU `periodStart=2026-02-30` (date inexistante, Pass 1 ECH-07), then status = 400 (body `text/plain` Axum default — Pass 3 BH3-12).

22. **AC #22 — Journal enum invalide** — Given `GET /reports/journals?journal=Salaires` OU `journal=achats` (mauvaise casse, Pass 1 ECH-08), then status = 400 (body `text/plain` Axum default — Pass 3 BH3-12).

### RBAC

23. **AC #23 — Authenticated read (3 rôles)** — Given un user authentifié avec rôle `Admin`, `Comptable` OU `Consultation`, when GET rapport, then response 200 dans les 3 cas. Test E2E couvre **au minimum le rôle Consultation** (AC #32 spécifique, le plus restrictif — si Consultation peut lire, Admin et Comptable peuvent aussi via la même route `authenticated_routes`). Pass 2 AA2-02 : AC #32 est le cas concret testé de AC #23 générique. **Pas de doublon** : #23 énonce la couverture des 3 rôles, #32 fournit l'assertion test concrète.

24. **AC #24 — Unauthenticated 401** — Given GET rapport sans header Authorization, then response 401.

### Audit log (Q3 décision)

25. **AC #25 — Audit émis sur succès** — Given GET rapport authentifié et succès 200 (pool test ≥ 4 connexions, cf. Pass 1 AA-04 §audit-shapes), when query `audit_log WHERE action = 'report.generated' AND user_id = ? ORDER BY id DESC LIMIT 1`, then 1 row existe avec `entity_type = 'report'`, `entity_id = AUDIT_ENTITY_ID_NONE (0)`, `details_json` JSON contenant `{ reportType, fiscalYearId, periodStart, periodEnd, journalFilter }` (Pass 1 BH-01 : sentinelle `entity_id = 0`).

26. **AC #26 — Pas d'audit sur erreur 400/404** — Given GET rapport avec params invalides (400) ou cross-tenant (404), when query audit_log, then **aucune** row `report.generated` n'est créée pour cette tentative.

### Frontend

27. **AC #27 — Page `/reports` accessible** — Given user authentifié sur frontend, when navigation vers `/reports`, then la page charge sans erreur, affiche `ReportSelector` avec dropdown exercices et 4 onglets (Bilan, Résultat, Balance, Journaux).

28. **AC #28 — Génération via UI** — Given user sur `/reports` avec un exercice sélectionné, when click bouton « Générer » sur l'onglet « Bilan », then la vue `BalanceSheetView` se charge avec les données du JSON formatées (montants apostrophe `1'234.56`, dates `dd.mm.yyyy`).

29. **AC #29 — Message UX si rapport vide** — Given GET rapport sur une période sans aucune écriture (response 200 avec `BalanceSheet.assets == [] && liabilities == []`, OU `IncomeStatement.revenues == [] && expenses == []`, OU `TrialBalance.rows == []`, OU `JournalReport.journals.every(j => j.entries == [])`), when frontend rend la vue correspondante, then affichage du message i18n `reports-error-no-entries-in-period` à la place du tableau (Pass 1 AA-11 + BH-11 : condition unifiée via helper `isReportEmpty(reportType, dto)` dans `reports.api.ts`). UX-DR38 respecté.

30. **AC #30 — Pas de bouton PDF/CSV v0.1** — Given page `/reports` rendue, when inspection UI, then **aucun** bouton « Télécharger PDF » ni « Export CSV » n'est présent (tracé LXX, livré 9-2).

### ACs additionnels Pass 1 (Pass 1 BH-12, AA-09, AA-10, ECH-12)

31. **AC #31 — Period effectif dans la réponse** (Pass 1 BH-12) — Given GET rapport avec `fiscalYearId=X` SANS `periodStart`/`periodEnd`, when response 200, then `response.period.fiscalYearId === X`, `response.period.startDate === fy.startDate` (ISO 8601), `response.period.endDate === fy.endDate`. Permet aux consumers 9-2 / Epic 14 de connaître la période effective sans re-requête `/fiscal-years`.

32. **AC #32 — Rôle Consultation autorisé en lecture** (Pass 1 AA-09) — Given un user JWT avec rôle `Consultation` (lecture seule, pas d'Admin ni Comptable), when GET `/api/v1/reports/balance-sheet?fiscalYearId={fy}`, then response 200 avec body BalanceSheetDto valide. Vérifie que la route est bien dans `authenticated_routes` (et pas `comptable_routes`).

33. **AC #33 — 4 onglets visibles sur `/reports`** (Pass 1 AA-10) — Given user authentifié sur `/reports`, when page rendue, then exactement **4** éléments `role="tab"` présents avec labels i18n `reports-balance-sheet`, `reports-income-statement`, `reports-trial-balance`, `reports-journals` (assertion Playwright `expect(page.getByRole('tab')).toHaveCount(4)`).

34. **AC #34 — Company sans exercice : bouton Générer désactivé** (Pass 1 ECH-12) — Given user authentifié d'une company qui n'a aucun `fiscal_year` (cas onboarding incomplet), when navigation `/reports`, then dropdown exercices vide + bouton « Générer » `disabled` + message i18n `reports-error-no-fiscal-year-available` affiché ("Aucun exercice comptable disponible. Créez un exercice avant de générer des rapports.").

## Tasks / Subtasks

### T1. Foundation `kesh-report` — Cargo + period + errors (AC implicite scaffolding)

- [ ] T1.1 — Étendre `crates/kesh-report/Cargo.toml` avec deps **versions directes** (Pass 1 BH-03 : le workspace root n'a PAS de section `[workspace.dependencies]` ; toutes les crates utilisent des versions directes alignées sur `kesh-db/Cargo.toml`). Bloc final :
  ```toml
  [dependencies]
  kesh-core = { path = "../kesh-core" }
  kesh-db = { path = "../kesh-db" }
  sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "mysql", "chrono", "rust_decimal", "macros"] }
  chrono = { version = "0.4", features = ["serde"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  thiserror = "2"
  rust_decimal = { version = "1.41", features = ["serde-str", "maths"] }
  tracing = "0.1"

  [dev-dependencies]
  rust_decimal_macros = "1.40"
  ```
  **Pass 1 BH-04** : feature `rust_decimal` = `"serde-str"` (PAS `"serde-with-str"` — inexistante). Active `rust_decimal::serde::str` pour sérialiser en string `"1234.56"`.
  **Pass 1 BH-07** : `kesh-i18n` **n'est PAS dépendance de `kesh-report` en 9-1** (aucun appel runtime — formatage suisse délégué au frontend). Ajout `kesh-i18n` reporté Story 9-2 (génération PDF côté Rust). Aucune fonction `kesh_i18n::format_money/format_date` n'est appelée par les 4 modules 9-1.
- [ ] T1.2 — Refactor `crates/kesh-report/src/lib.rs` : suppression du `//! Crate placeholder...`, déclaration des 5 modules + `pub use` exports (cf. §rust-types).
- [ ] T1.3 — Créer `crates/kesh-report/src/errors.rs` avec `ReportError` enum (cf. §error-shapes) + impl `From<DbError>` + `From<ReportError> for AppError` (mais l'impl `for AppError` reste dans `kesh-api/src/errors.rs` pour éviter dépendance cyclique).
- [ ] T1.4 — Créer `crates/kesh-report/src/period.rs` avec struct `ReportPeriod` (derive `Debug, Clone, Serialize` + `#[serde(rename_all = "camelCase")]`) + méthode `resolve(pool, company_id, fiscal_year_id, period_start?, period_end?)` qui :
  - Récupère `fiscal_years` via `kesh_db::repositories::fiscal_years::find_by_id_in_company(pool, company_id, fiscal_year_id)` (Pass 1 BH-02 : ordre params `(pool, company_id, id)`).
  - Retourne `ReportError::FiscalYearNotFound` si None.
  - Applique la table de résolution asymétrique (Pass 1 ECH-02, cf. §scope §2).
  - Valide bornes incluses dans fy, ordre `start ≤ end`. Erreurs : `PeriodInvalid` ou `PeriodOutOfFiscalYear`.
- [ ] T1.5 — 7 unit tests `period::tests::*` : default_period, partial_period_both, partial_period_start_only (Pass 1 ECH-02), partial_period_end_only (Pass 1 ECH-02), period_out_of_fy_end, period_out_of_fy_start, period_inversed, period_same_day_is_valid (Pass 1 ECH-18).

### T2. Module `balance_sheet.rs` (AC #1, #2)

- [ ] T2.1 — Créer struct `BalanceSheet` + `AccountBalance` (cf. §rust-types) + `pub async fn generate(pool, company_id, period) -> Result<BalanceSheet, ReportError>`.
- [ ] T2.2 — SQL : 1 query qui agrège par `account_id` filtré par `a.account_type IN ('Asset','Liability')` ET `je.entry_date BETWEEN ? AND ?` ET `je.fiscal_year_id = ?` ET `a.company_id = ? AND je.company_id = ?`. **Pass 3 BH3-14** : préfixer **TOUTES** les colonnes par leur alias de table (`je.entry_date`, `je.fiscal_year_id`, `je.company_id`, `a.account_type`, `a.account_number`, `jel.debit`, `jel.credit`) pour éviter toute ambiguïté JOIN (cohérent T4.2 + T5.2 déjà préfixés). Inclut comptes archivés avec écritures (cf. Q2 + Pass 2 AA2-11). Calcule `COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0)` (sign convention par type, Pass 1 ECH-01).
- [ ] T2.3 — Calculer `equity_result` via **appel direct à `income_statement::generate(pool, company_id, period)`** depuis `balance_sheet.rs`, puis extraire `.net_result` (Pass 1 BH-13 : pas de helper `compute_net_result` partagé — YAGNI v0.1 ; 2 queries SQL distinctes mais simples). Si optimisation perf requise post-merge (R1 dette) → factoriser en `aggregates.rs` interne v0.2.
- [ ] T2.4 — Vérifier `equation_holds = (total_assets == total_liabilities + equity_result)`. Ne pas retourner erreur si false (defense in depth : le frontend affichera un badge rouge). Logger `warn!` si false.
- [ ] T2.5 — 5 unit tests inline `#[cfg(test)] mod tests` : equation_balance_sums, partial_period_excludes_outside_entries, archived_account_with_entries_appears, ordering_by_account_number, **empty_period_returns_zero_totals_equation_holds** (Pass 1 ECH-01).

### T3. Module `income_statement.rs` (AC #3)

- [ ] T3.1 — Struct `IncomeStatement` + `generate(pool, company_id, period) -> Result<...>`.
- [ ] T3.2 — SQL agrégation `account_type IN ('Revenue','Expense')` (sign convention : Revenue = `credit - debit`, Expense = `debit - credit`).
- [ ] T3.3 — `net_result = total_revenues - total_expenses`.
- [ ] T3.4 — 4 unit tests : net_result_positive, net_result_negative, net_result_zero, **net_result_ordering_by_account_number** (Pass 1 AA-01).

### T4. Module `trial_balance.rs` (AC #4, #5, #6)

- [ ] T4.1 — Struct `TrialBalance` + `TrialBalanceRow` + `generate(...)`.
- [ ] T4.2 — SQL : SELECT par compte avec `LEFT JOIN journal_entry_lines` filtré période. Inclure comptes actifs **OU** archivés avec écritures dans la période (Q2). Calculer `SUM(debit)`, `SUM(credit)`, `balance` signé selon `account_type`.
- [ ] T4.3 — Vérifier `total_debit == total_credit`. Si non → `ReportError::TrialBalanceUnbalanced { total_debit, total_credit }` + log `error!`.
- [ ] T4.4 — 5 unit tests : balanced_true_on_valid_seed, archived_with_entries_appears_with_marker, archived_without_entries_excluded, totals_match_aggregation, **active_account_without_entries_appears_with_zero_balance** (Pass 1 ECH-03).

### T5. Module `journal_report.rs` (AC #7, #8, #9, #10)

- [ ] T5.1 — Struct `JournalReport`, `JournalSection`, `JournalEntryRow`, `JournalEntryLineRow` + `generate(pool, company_id, period, journal_filter: Option<Journal>) -> Result<...>`.
- [ ] T5.2 — SQL : SELECT `journal_entries JOIN journal_entry_lines JOIN accounts` filtré company + fiscal_year + période + (optionnel) journal. ORDER BY `je.journal, je.entry_date, je.entry_number, jel.line_order`.
- [ ] T5.3 — Grouper en mémoire par `journal` (5 sections fixes Achats, Ventes, Banque, Caisse, OD si `journal_filter = None`, sinon 1 section).
- [ ] T5.4 — Calculer `section_total_debit`, `section_total_credit`, `grand_total_debit`, `grand_total_credit`.
- [ ] T5.5 — 6 unit tests : all_journals_present (5 sections vides ou non), filter_achats_only_returns_one_section, ordering_chronological_then_entry_number, line_order_preserved, empty_period_returns_empty_sections, **filter_none_with_missing_journal_returns_5_sections_with_empty_one** (Pass 1 BH-18 + ECH-05).

### T6. Routes API `kesh-api/routes/reports.rs` (AC #11-#24, #25-#26)

- [ ] T6.1 — Créer `crates/kesh-api/src/routes/reports.rs` avec 4 handlers : `get_balance_sheet`, `get_income_statement`, `get_trial_balance`, `get_journal_report`.
- [ ] T6.2 — Query params extractor (Pass 2 AA2-06 + Pass 3 BH3-13 — derives explicites + champs dupliqués obligatoires) :
  ```rust
  use serde::Deserialize;
  use chrono::NaiveDate;

  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]  // Pass 2 AA2-06 : sans ce derive, query params seraient snake_case en URL
  pub struct ReportQuery {
      pub fiscal_year_id: i64,             // URL : ?fiscalYearId=42
      pub period_start: Option<NaiveDate>, // URL : ?periodStart=2026-01-01
      pub period_end: Option<NaiveDate>,   // URL : ?periodEnd=2026-12-31
  }

  // Pass 3 BH3-13 : `#[serde(flatten)]` NE FONCTIONNE PAS avec serde_urlencoded (Axum 0.8 default).
  // Bug serde_urlencoded #33 — champs flattenés invisibles au parser. Dupliquer les champs
  // manuellement (cohérent pattern Axum recommandé).
  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct JournalReportQuery {
      pub fiscal_year_id: i64,             // dupliqué de ReportQuery
      pub period_start: Option<NaiveDate>,
      pub period_end: Option<NaiveDate>,
      pub journal: Option<kesh_db::entities::journal_entry::Journal>,  // URL : ?journal=Achats (case-sensitive)
  }

  impl JournalReportQuery {
      /// Helper pour réutiliser le code commun de résolution ReportPeriod.
      pub fn as_report_query(&self) -> ReportQuery {
          ReportQuery {
              fiscal_year_id: self.fiscal_year_id,
              period_start: self.period_start,
              period_end: self.period_end,
          }
      }
  }
  ```
- [ ] T6.3 — Chaque handler :
  1. Extrait `CurrentUser` (JWT middleware) → `company_id`.
  2. Parse query params, valide (`fiscal_year_id > 0`).
  3. Appelle `ReportPeriod::resolve(pool, company_id, ...)`.
  4. Appelle `kesh_report::generate_balance_sheet(...)` (ou autre).
  5. Sur succès : insert audit log `report.generated` (transaction dédiée, best-effort).
  6. Retourne `Json(BalanceSheetDto)` avec status 200.
- [ ] T6.4 — Étendre `crates/kesh-api/src/errors.rs` avec 3 variants `AppError::ReportFiscalYearNotFound`, `AppError::ReportPeriodOutOfFiscalYear { fy_start, fy_end, requested_start, requested_end }`, `AppError::ReportPeriodInvalid { reason }` + `IntoResponse` mapping (cf. §error-shapes).
- [ ] T6.5 — Étendre `crates/kesh-api/src/routes/mod.rs` avec `pub mod reports;`.
- [ ] T6.6 — Étendre `crates/kesh-api/src/lib.rs` `authenticated_routes` avec 4 routes :
  ```rust
  .route("/api/v1/reports/balance-sheet", get(routes::reports::get_balance_sheet))
  .route("/api/v1/reports/income-statement", get(routes::reports::get_income_statement))
  .route("/api/v1/reports/trial-balance", get(routes::reports::get_trial_balance))
  .route("/api/v1/reports/journals", get(routes::reports::get_journal_report))
  ```
- [ ] T6.7 — Étendre `Cargo.toml` `kesh-api` : ajouter `kesh-report = { path = "../kesh-report" }` (Pass 3 BH3-08 : `workspace = true` ne compile pas — root Cargo.toml n'a pas de `[workspace.dependencies]`).

### T7. Audit log (AC #25, #26)

- [ ] T7.1 — Définir helper privé dans `reports.rs` : `async fn emit_report_audit(pool, user_id, report_type, fiscal_year_id, period, journal_filter?) -> ()`. Best-effort : log `warn!` si l'INSERT audit échoue, **ne pas faire échouer la response**.
- [ ] T7.2 — Appelé **après** le SELECT métier réussi (pas avant). Pas d'appel sur path d'erreur 400/404/500.
- [ ] T7.3 — Action `report.generated`, `entity_type = "report"`, `entity_id = None`, `details_json` (cf. §audit-shapes).

### T8. Frontend `features/reports/` (AC #27, #28, #29, #30)

- [ ] T8.1 — Créer `frontend/src/lib/features/reports/reports.types.ts` avec types DTO (BalanceSheetDto, IncomeStatementDto, TrialBalanceDto, JournalReportDto, AccountBalance, TrialBalanceRow, JournalSection, etc.). Montants `string`, dates `string` ISO 8601.
- [ ] T8.2 — Créer `reports.api.ts` avec 4 fetchers (utilise `api-client.ts` wrapper fetch JWT).
- [ ] T8.3 — Créer `ReportSelector.svelte` : dropdown exercices (load via `/api/v1/fiscal-years`), 2 date inputs (period start/end optionnels, format `dd.mm.yyyy` saisie → conversion `YYYY-MM-DD` envoi API), bouton « Générer ».
- [ ] T8.4 — Créer 4 vues : `BalanceSheetView.svelte`, `IncomeStatementView.svelte`, `TrialBalanceView.svelte`, `JournalReportView.svelte`. Affichage Tailwind sobre. Montants via `formatMoney` (apostrophe). Dates via `formatDate` (dd.mm.yyyy).
- [ ] T8.5 — Créer `frontend/src/routes/(app)/reports/+page.svelte` avec 4 onglets (Bilan, Résultat, Balance, Journaux) + chargement à la demande de la vue active.
- [ ] T8.6 — Créer `+page.ts` load function : récupère la liste des fiscal_years.
- [ ] T8.7 — **5 Vitest tests** : `reports.api.test.ts` (formatage suisse, période par défaut, helper `isReportEmpty`) + `BalanceSheetView.test.ts` (équation visualisée, **coloration `equity_result` rouge/vert/neutre selon signe** — Pass 3 BH3-19 : test dédié assertion CSS `text-red-600` pour perte / `text-green-600` pour bénéfice / neutre pour zéro).

### T9. i18n (AC #29 + ownership lint)

- [ ] T9.1 — Ajouter **34 clés** `reports-*` dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (canonical, cf. §i18n-keys — Pass 3 BH3-10).
- [ ] T9.2 — Ajouter les mêmes **34 clés** en DE/IT/EN-CH avec traduction basique + commentaire `# TODO official translation` (cohérent L51 héritée 8-5b).
- [ ] T9.3 — Lint `npm run lint-i18n-ownership` → PASS. Si échec → vérifier préfixe `reports-` et propriétaire `features/reports/`.

### T10. Tests E2E HTTP `kesh-api` (AC #11-#26)

- [ ] T10.1 — Créer `crates/kesh-api/tests/reports_e2e.rs` avec helper `spawn_app` (pattern 8-5b). Seed minimal : 1 company + 1 fiscal_year ouvert + 5 comptes (1 `Asset` ex. 1000, 1 `Liability` ex. 2000, 1 `Liability` ex. 3000 *« fonds propres permanents »* — Pass 3 BH3-11 : **PAS de variant `Equity`** dans le CHECK constraint `account_type IN ('Asset','Liability','Revenue','Expense')`, donc fonds propres sémantiques en `Liability`), 1 `Revenue` ex. 4000, 1 `Expense` ex. 5000) + ~3 écritures multi-journaux équilibrées + 1 compte archivé `Asset` ex. 1090 avec écriture. **Pool ≥ 4 connexions** (Pass 2 ECH2-07 + L62).
- [ ] T10.2 — **≥ 28 tests** (Pass 1 — count enrichi de 20 → 28 pour couvrir les ACs orphelins AA-06/07/08/09/10) :
  1. `balance_sheet_returns_balanced_assets_liabilities` (AC #1)
  2. `balance_sheet_orders_accounts_by_number` (AC #2)
  3. `income_statement_computes_net_result_and_orders_by_account_number` (AC #3 — Pass 1 AA-01 inclut le check ordering)
  4. `trial_balance_total_debit_equals_total_credit` (AC #4)
  5. `trial_balance_includes_archived_account_with_entries` (AC #5)
  6. `trial_balance_excludes_archived_without_entries` (AC #6)
  7. `journals_returns_five_sections_in_order` (AC #7) — **Pass 1 ECH-05** : fixture seed avec écritures dans **3 journaux seulement** (Achats, Banque, OD) mais assertion 5 sections présentes (Ventes et Caisse vides avec `entries: []`).
  8. `journals_filter_achats_returns_one_section` (AC #8)
  9. `journals_orders_entries_chronologically` (AC #9)
  10. `journals_preserves_line_order` (AC #10)
  11. `default_period_uses_fiscal_year_full_range_all_endpoints` (AC #11 + AC #31) — **Pass 1 BH-12 + Pass 2 AA2-09** : assertion `response.period.startDate == fy.start_date && response.period.endDate == fy.end_date && response.period.fiscalYearId == fy.id` testée **sur les 4 endpoints** (balance-sheet, income-statement, trial-balance, journals) — 4 sous-tests groupés ou 1 test parameterized.
  12. `partial_period_excludes_outside_entries` (AC #12)
  13. `period_end_out_of_fy_returns_400_with_four_details_fields` (AC #13) — **Pass 1 AA-02** : assertion JSON body 400 contient les **4 champs** `error.details.{fyStart, fyEnd, requestedStart, requestedEnd}`.
  14. `period_inversed_returns_400_validation_code` (AC #14) — **Pass 1 AA-05** : assertion sur `error.code = "VALIDATION_ERROR"`, pas sur texte FR.
  15. `multi_fiscal_years_isolation` (AC #15)
  16. `cross_tenant_fiscal_year_returns_404` (AC #16)
  17. `cross_tenant_aggregation_filtered_by_company` (AC #17)
  18. `audit_log_scoped_to_company_via_user_join` (AC #18 — Pass 1 AA-03 reformulé : JOIN `users` pour vérifier le scope).
  19. `fiscal_year_id_missing_returns_400` (AC #19)
  20. `fiscal_year_id_malformed_returns_400` (AC #20 — Pass 1 AA-06 : `fiscalYearId=abc`).
  21. `fiscal_year_id_zero_or_negative_returns_400` (AC #20-bis — Pass 1 ECH-06 : 2 sub-cases `=0` et `=-1`).
  22. `date_malformed_returns_400` (AC #21 — Pass 1 AA-07 : 2 sub-cases `2026/01/15` et `2026-02-30`).
  23. `journal_enum_invalid_returns_400_with_accepted_values` (AC #22 — Pass 1 AA-08 + ECH-08 : 2 sub-cases `journal=Salaires` et `journal=achats` lowercase).
  24. `consultation_role_can_read_reports` (AC #32 — Pass 1 AA-09 : JWT avec rôle `Consultation` → 200).
  25. `unauthenticated_returns_401` (AC #24)
  26. `report_generated_audit_emitted_on_success_with_sentinel_entity_id` (AC #25 — Pass 1 BH-01 : assertion `entity_id == 0`).
  27. `report_generated_audit_not_emitted_on_400_404` (AC #26).
  28. `balance_sheet_empty_period_returns_zero_totals_equation_holds` (Pass 1 ECH-01 : cas vide).
- [ ] T10.3 — Vérifier passe `cargo test -p kesh-api --test reports_e2e -- --test-threads=1` MariaDB up. Tolérance 0 régression sur les tests existants `cargo test --workspace`.

### T11. Tests d'intégration `kesh-db` (multi-tenant, agrégats SQL)

**Pass 1 AA-12 — clarification distinction T10 vs T11** : T11 teste les **agrégats SQL** au niveau repo/kesh-db (correctness des `COALESCE(SUM(...), 0)`, des `BETWEEN` bounds, des `WHERE company_id = ?` ground-truth). T10 teste les **endpoints HTTP** end-to-end (parsing JWT, response shape, codes HTTP). **Pas de redondance** : T11 isole les SQL avec seed inline minimal, T10 vérifie l'intégration complète depuis JWT auth. Les deux niveaux protègent des classes de régression différentes.

- [ ] T11.1 — Créer `crates/kesh-db/tests/report_aggregates.rs` (≥ 7 tests sqlx) — multi-tenant strict, période bounds inclusives, fiscal_year isolation, SUM sur 0 ligne (COALESCE), exercice non-calendaire (Pass 1 ECH-16). Pattern `#[sqlx::test]` héritée 8-5b.
  **Pass 2 AA2-12 — seed structure obligatoire** (sinon `fiscal_year_non_calendar_isolation` peut passer vacuously) :
  ```rust
  // Fixture minimale par test sqlx (inline ou via helper module dans tests/):
  // - 1 company (ex. id=1)
  // - 1 fiscal_year **NON-CALENDAIRE** : start_date=2026-07-01, end_date=2027-06-30, status='Open'
  // - 5 comptes (Pass 3 BH3-11 : tous CHECK enum 4 valeurs, PAS d'Equity) :
  //     1000 (Asset, Banque), 2000 (Liability, Fournisseurs),
  //     3000 (Liability, Capital social — fonds propres permanents sémantique mais stockés Liability),
  //     4000 (Revenue, Ventes), 5000 (Expense, Achats). Tous actifs.
  // - 1 compte archivé avec écriture : 1090 Asset active=false avec 1 ligne écriture dans la période.
  // - 3 écritures journal_entries (Pass 3 BH3-11 : énumération explicite) :
  //     E1 (Achats, 2026-09-15) : 2 lignes — débit 5000=1000.00, crédit 2000=1000.00
  //     E2 (Banque, 2026-12-31) : 2 lignes — débit 1000=500.00, crédit 4000=500.00
  //     E3 (OD, 2027-03-15)     : 2 lignes — débit 1090=250.00 (archivé), crédit 2000=250.00
  // Total 3 écritures × 2 lignes = 6 lignes, équilibrées, fy non-calendaire respecté.
  ```
  Permet de vérifier ground-truth :
  - `fiscal_year_non_calendar_isolation` : un test paire crée fy2 calendaire 2027-01-01→2027-12-31 ; les écritures fy1 (juillet-juin) ne contaminent pas fy2.
  - `archived_account_with_entries_appears` : compte 1090 archivé doit apparaître dans trial_balance.
  - Etc.

### T12. Playwright E2E + a11y (AC #27, #28, #33, #34)

- [ ] T12.1 — Créer `frontend/tests/e2e/reports.spec.ts` : login → navigation `/reports` → **assertion `expect(page.getByRole('tab')).toHaveCount(4)`** (Pass 1 AA-10 + AC #33) → sélection exercice seed → onglet « Bilan » → click « Générer » → assertion présence `Total actifs` + montant attendu formaté apostrophe.
- [ ] T12.2 — 1 axe a11y scan zero violations sur la page `/reports` rendue.
- [ ] T12.3 — Sur Ubuntu 26.04+ : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reports.spec.ts`.
- [ ] T12.4 — Test scénario `company_without_fiscal_year_disables_generate_button` (Pass 1 ECH-12 + AC #34) : seed une 2e company sans `fiscal_year` → login user de cette company → navigation `/reports` → assertion bouton « Générer » `disabled` + message `reports-error-no-fiscal-year-available` visible.

### T13. Sync sprint-status + README + CI

- [ ] T13.1 — `_bmad-output/implementation-artifacts/sprint-status.yaml` : `9-1-rapports-comptables-bilan-resultat-balance-journaux: ready-for-dev → in-progress` (au moment de `bmad-dev-story`), puis `in-progress → review` (au commit dev-story), puis `review → done` (au merge post-code-review).
- [ ] T13.2 — README.md : vérifier section « Feuille de route » — Epic 9 statut `🚧 En cours` au démarrage 9-1 (épisode déjà tagué `🚧` lors de la création epic-9.md commit `95d7bc3`). Aucun changement v0.1 features tant que 9-2 (export) n'est pas livré.
- [ ] T13.3 — `.github/workflows/ci.yml` (Pass 3 BH3-18 + Pass 2 ECH2-07 + L62) : ajouter env var `SQLX_MAX_CONNECTIONS: "4"` au step `Backend (Rust)` pour garantir le pool ≥ 4 pendant `cargo test -- --test-threads=1` (audit best-effort + SELECT métier concurrents sur la même request). Si déjà présent (autres stories Epic 8), aucune modification.

## Dev Notes

### API surface existante à réutiliser (livré Epics 1-8)

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403. Source : `kesh-db/src/repositories/journal_entries.rs:55`+, `kesh-db/src/repositories/accounts.rs:108`.
- **`fiscal_years::find_by_id_in_company(pool, id, company_id)`** : `crates/kesh-db/src/repositories/fiscal_years.rs:399`. Retourne `Result<Option<FiscalYear>, DbError>`. Utilisé pour résoudre + vérifier multi-tenant en T1.4.
- **`audit_log::insert_in_tx(tx, NewAuditLogEntry)`** : `crates/kesh-db/src/repositories/audit_log.rs` (ligne 26+). Atomique avec transaction caller. Pour Q3 audit `report.generated`, ouvrir une mini-tx dédiée (consultation read-only, pas de pattern transactionnel mutation).
- **`kesh_i18n::format_money(&Decimal)` + `format_date(&NaiveDate)`** : `crates/kesh-i18n/src/formatting.rs:16, 38`. **Apostrophe U+2019** (typographique, pas `'`). Dates `dd.mm.yyyy`. Tests existants (15+ cas).
- **`AccountType`** enum : variantes `Asset|Liability|Revenue|Expense` (cf. `crates/kesh-db/migrations/20260411000001_accounts.sql` ligne 11 + entité Rust associée — vérifier `kesh-db/src/entities/account.rs`).
- **`Journal`** enum : variantes `Achats|Ventes|Banque|Caisse|OD` (cf. `crates/kesh-db/migrations/20260412000001_journal_entries.sql` ligne 30 + `kesh-db/src/entities/journal_entry.rs`).
- **`CurrentUser` extension** (Pass 3 BH3-06 : PAS un extractor custom — pattern Axum `Extension<T>` injecté par middleware `require_auth`) : défini dans `crates/kesh-api/src/middleware/auth.rs:29` comme `pub struct CurrentUser { pub user_id: i64, pub role: Role, pub company_id: i64 }` (champs **non-Option** : Pass 3 BH3-16 — R12 KF-002 résolu Story 6.2). Usage dans handlers : `pub async fn get_balance_sheet(State(state): State<AppState>, Extension(current_user): Extension<CurrentUser>, Query(query): Query<ReportQuery>) -> Result<Json<BalanceSheet>, AppError> { ... }`. Pattern hérité de `fiscal_years.rs:153-156`. **Accéder via `current_user.user_id` + `current_user.company_id`** (Pass 3 BH3-05).
- **Convention JSON camelCase** : `#[serde(rename_all = "camelCase")]` sur chaque DTO (architecture.md §Format Patterns).
- **Decimal serde** : utiliser feature `serde-with-str` de `rust_decimal` pour sérialiser en string (`"1234.56"`).

### Lessons des stories précédentes (Epic 8 retro)

- **Pattern audit append-only** (cf. audit_log.rs ligne 1-5 commentaire « Pas de méthode `delete` ») : audit est inamovible, CO Art. 957-964 conservation 10 ans. Insertion uniquement.
- **8-5b lesson Q5 (suggestion ML hors scope)** : pas d'invention de feature non demandée. Pour 9-1, **pas de drill-down, pas de personnalisation, pas de comparaison budget** — strictement les 4 rapports requis par FR65.
- **8-5b lesson migration UNIQUE partiel** : pas applicable ici (pas de nouvelle table 9-1, juste agrégations sur tables existantes). Pas de migration DB en 9-1 (sauf si validate Pass 1 décide audit avec une nouvelle table `report_generations` — peu probable, audit_log suffit).
- **8-5a-bis lesson breaking change discriminator** : non applicable (9-1 ajoute des routes, ne modifie aucun contrat existant).
- **8-4 retro découpage scope** : 9-1 fait ~2500-3000 lignes nettes (4 modules × ~300 lignes + 4 routes × ~150 + frontend + tests). Acceptable, comparable à 8-4 (~2200 lignes pour 5 modules). **Splitting safety-net** : si validate diverge > 4 passes, splitter en 9-1a (balance_sheet + income_statement, le cœur) + 9-1b (trial_balance) + 9-1c (journal_report) — cohérent règle CLAUDE.md « profondeur d'incertitude ». Documenter alors dans le story file avant re-spec.
- **Test Locally First (CLAUDE.md)** : 9-1 ajoute des tests sqlx (T11) qui nécessitent MariaDB up local. Avant push CI : `cargo test --workspace -j1 -- --test-threads=1` au minimum sur les modules touchés.
- **Path-dépendance v0.2** : la surface JSON livrée par 9-1 (BalanceSheet, IncomeStatement, TrialBalance, JournalReport) sera réutilisée par 9-2 (PDF/CSV rendering) et par Epic 14 (clôture utilise `BalanceSheet` + `IncomeStatement`). **Garder l'API publique stable** (helper pattern Story 8-5a-base). Tout breaking change post-9-1 → CR explicite.

### Patterns architecturaux à respecter

- **Pas de `f64`** : `rust_decimal::Decimal` partout. `equation_holds = (a == b + c)` égalité Decimal exacte, pas de tolérance.
- **Pas de dépendance circulaire** : `kesh-report` dépend de `kesh-core`, `kesh-db` (Pass 2 BH2-02 : PAS `kesh-i18n` en 9-1 — formatage frontend uniquement). **Pas d'inverse**. `kesh-api` dépend de `kesh-report`.
- **Repository pattern strict** : les SQL queries des modules `kesh-report` peuvent rester inline dans le module (pattern OK pour des agrégations ad-hoc), mais si Story 9-2 ou Epic 14 réutilise la même agrégation → extraire dans `kesh-db/repositories/` au moment du besoin (YAGNI v0.1).
- **Naming patterns** (architecture.md §Naming Patterns) :
  - Routes kebab-case : `/reports/balance-sheet`, `/reports/income-statement`, `/reports/trial-balance`, `/reports/journals`.
  - Query params camelCase : `fiscalYearId`, `periodStart`, `periodEnd`.
  - Code Rust snake_case : `balance_sheet`, `income_statement`, `trial_balance`, `journal_report`.
  - Frontend kebab-case routes : `routes/(app)/reports/+page.svelte`.
- **Codes HTTP** : 200 succès, 400 validation, 401 auth, 404 cross-tenant ou FY not found, 500 invariant break (trial unbalanced). Pas de 403 (multi-tenant = 404).
- **i18n key ownership** : préfixe `reports-`, propriétaire `features/reports/`. Lint vérifie qu'aucune autre feature n'utilise ces clés.

### Source tree à toucher

**Crate `kesh-report` (extension de coquille vide)** :
- `crates/kesh-report/Cargo.toml` (T1.1 — ajout deps)
- `crates/kesh-report/src/lib.rs` (T1.2 — refactor placeholder → vrais modules)
- `crates/kesh-report/src/errors.rs` *(nouveau, T1.3)*
- `crates/kesh-report/src/period.rs` *(nouveau, T1.4)*
- `crates/kesh-report/src/balance_sheet.rs` *(nouveau, T2)*
- `crates/kesh-report/src/income_statement.rs` *(nouveau, T3)*
- `crates/kesh-report/src/trial_balance.rs` *(nouveau, T4)*
- `crates/kesh-report/src/journal_report.rs` *(nouveau, T5)*

**Crate `kesh-api`** :
- `crates/kesh-api/Cargo.toml` (T6.7 — ajout `kesh-report` dep)
- `crates/kesh-api/src/routes/reports.rs` *(nouveau, T6.1-T6.3)*
- `crates/kesh-api/src/routes/mod.rs` (T6.5 — `pub mod reports`)
- `crates/kesh-api/src/lib.rs` (T6.6 — 4 routes ajoutées dans `authenticated_routes`)
- `crates/kesh-api/src/errors.rs` (T6.4 — 3 nouveaux variants AppError + IntoResponse)
- `crates/kesh-api/tests/reports_e2e.rs` *(nouveau, T10 — ≥ 28 tests, Pass 2 BH2-03)*

**Crate `kesh-db`** :
- `crates/kesh-db/tests/report_aggregates.rs` *(nouveau, T11 — ≥ 7 tests sqlx, Pass 2 BH2-04)*
- `crates/kesh-db/src/entities/mod.rs` *(modif 1 ligne, Pass 3 ECH3-02 — re-export `AUDIT_ENTITY_ID_NONE`)*
- `crates/kesh-db/src/entities/audit_log.rs` *(modif — ajout `pub const AUDIT_ENTITY_ID_NONE: i64 = 0` Pass 2 ECH2-01)*

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (T9 — 30 nouvelles clés × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reports/reports.types.ts` *(nouveau, T8.1)*
- `frontend/src/lib/features/reports/reports.api.ts` *(nouveau, T8.2)*
- `frontend/src/lib/features/reports/ReportSelector.svelte` *(nouveau, T8.3)*
- `frontend/src/lib/features/reports/BalanceSheetView.svelte` *(nouveau, T8.4)*
- `frontend/src/lib/features/reports/IncomeStatementView.svelte` *(nouveau, T8.4)*
- `frontend/src/lib/features/reports/TrialBalanceView.svelte` *(nouveau, T8.4)*
- `frontend/src/lib/features/reports/JournalReportView.svelte` *(nouveau, T8.4)*
- `frontend/src/lib/features/reports/reports.api.test.ts` *(nouveau, T8.7)*
- `frontend/src/lib/features/reports/BalanceSheetView.test.ts` *(nouveau, T8.7)*
- `frontend/src/routes/(app)/reports/+page.svelte` *(nouveau, T8.5)*
- `frontend/src/routes/(app)/reports/+page.ts` *(nouveau, T8.6)*
- `frontend/tests/e2e/reports.spec.ts` *(nouveau, T12.1)*

**Aucune migration DB en 9-1** (Q3 audit utilise table `audit_log` existante).

### Standards de test

- **Unit `kesh-report`** (T2.5, T3.4, T4.4, T5.5) : `#[cfg(test)] mod tests` inline dans chaque module. **≥ 16 tests** total. Utilise une mini-fixture en-mémoire si possible (Decimal pur, pas d'I/O) OU `#[sqlx::test]` si l'agrégation est plus claire à tester DB-side.
- **Intégration `kesh-db`** (T11) : `#[sqlx::test]` dans `crates/kesh-db/tests/report_aggregates.rs`. **≥ 7 tests** sqlx (Pass 2 BH2-04). Seed via `kesh-seed` ou inline INSERT.
- **E2E HTTP `kesh-api`** (T10) : `spawn_app(pool)` pattern hérité 8-5b. **≥ 28 tests** (énumérés T10.2 — Pass 2 BH2-03).
- **Vitest frontend** (T8.7) : `npm run test:unit -- reports`. **≥ 4 tests**.
- **Playwright** (T12) : `frontend/tests/e2e/reports.spec.ts`. **≥ 1 actif + 1 a11y**.

### Checklist locale avant push

```sh
# Backend (CLAUDE.md « Test Locally First »)
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB up requis (T10 E2E + T11 sqlx)

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # T9.3
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reports.spec.ts
```

### Limitations connues v0.1 (LXX 9-1)

| # | Limitation | Justification |
|---|---|---|
| L52 | Pas d'export PDF/CSV en 9-1 | Livré 9-2 (FR66, FR67). Évite couplage 9-1 avec choix librairie PDF (R3 epic-9.md). |
| L53 | Pas de drill-down depuis bilan/résultat vers les écritures | v0.2 UX. Pour v0.1 l'utilisateur consulte le journal séparément. |
| L54 | Pas de pagination ni streaming pour très gros exercices > 50k lignes | v0.2 (R1 epic-9.md). Charge tout en mémoire, latence négligeable < 10k écritures. Si > 10k observé → CR. |
| L55 | Pas de comparatif inter-exercices (bilan année N vs année N-1) | v0.2. v0.1 = un seul exercice à la fois. |
| L56 | Pas de personnalisation visuelle (logo, footer, format légal alternatif) | FR81 Epic 15. |
| L57 | Traductions DE/IT/EN-CH basiques (TODO official translation) | Cohérent L51 héritée 8-5b. Reporté v0.2. |
| L58 | Pas d'audit log côté frontend (UI consultation log) | Audit DB only v0.1. UI v0.2. |
| L59 | Le résultat de l'exercice affiché au bilan (`equity_result`) est calculé en mémoire sans écriture de clôture comptable réelle | Q1 Option A v0.1. La clôture (Epic 14) générera l'écriture 2979 réelle. v0.1 affichage informatif. |
| L60 | Single-currency (CHF) | Hérité L38 héritée 8-4. Multi-currency Epic 11+. |
| L61 | Présentation `equity_result` séparé du `total_liabilities` au bilan (Q1 Option A) | Pass 1 ECH-04 : table `accounts.account_type` n'a pas de variant `Equity` ; les fonds propres permanents sont en `Liability`. L'équation tient mais le rendu UI doit préciser « avant clôture ». Vrai bilan post-clôture Epic 14. |
| L62 | Pool de connexion DB test ≥ 4 pour AC #25 (audit best-effort + SELECT métier concurrent) | Pass 1 AA-04 + Pass 2 ECH2-07 : sans pool ≥ 4, race INSERT audit possible. **CI doit configurer `SQLX_MAX_CONNECTIONS=4` au minimum** (env var dans `.github/workflows/ci.yml` step de test DB). Dev-story T10.1 doit vérifier le helper `spawn_app(pool)` accepte un paramètre pool_size ≥ 4 OU utilise la default sqlx (qui est ≥ 8). KF post-merge si flake observé CI. |
| L63 | Pas de hiérarchie `parent_id` exploitée v0.1 (pas de sous-totaux par groupe parent) | Pass 1 ECH-14 : tous les comptes traités comme feuilles. Drill-down + subtotaux v0.2. |
| L64 | `report.generated` audit best-effort (warn! sur INSERT failure, retour 200) | Pass 1 Q3 décidé + ECH-15 pattern code documenté. CO Art. 958f satisfait pour le cas nominal ; failure mode = consultation non auditée mais data retournée intacte. KF si CO inspection révèle non-conformité. |
| L65 | `balance_sheet` exécute 2 queries SQL (balance + income_statement pour equity_result) non-atomiques | Pass 2 BH2-10 / R17 : sous READ COMMITTED, divergence possible ≤ centimes si écriture concurrente. Acceptable v0.1, optimisation query unifiée OU snapshot REPEATABLE READ → v0.2. |
| L66 | Period inversion via résolution asymétrique : si `(None, Some(e))` avec `e < fy.start_date` | Pass 2 ECH2-02 : résolution donne `(fy.start_date, e)` avec `start > end` → check `start ≤ end` rejette en 400 `VALIDATION`. Message error pourrait être plus explicite. KF UX v0.2. |
| L67 | Ambiguïté timezone des dates en query params API | Pass 2 ECH2-04 : `entry_date` stocké `DATE` (jour calendaire sans TZ). API attend ISO `YYYY-MM-DD` interprété comme date calendaire pure (pas datetime). Pas de conversion TZ backend ; frontend affiche en local user. Documenté en §api-routes. |
| L68 | Body 400 hétérogène : `text/plain` (parse query) vs `application/json` (validation handler) | Pass 3 BH3-12 : Axum `Query<T>` default rejection retourne `text/plain` (cohérent stories antérieures). ACs #19/#20/#21/#22 testés sur status 400 uniquement. AC #20-bis (validation post-parsing) teste JSON `VALIDATION_ERROR`. Uniformisation JSON v0.2 nécessite handler `Query<T, R>` custom. |
| L69 | Schema `ErrorBody`/`ErrorDetail` n'a pas de champ `details` — variant unique `ReportPeriodOutOfFiscalYear` utilise body JSON ad-hoc | Pass 3 BH3-03 : pattern divergent (cohérent v0.1, dette refactor v0.2). Le variant émet directement `serde_json::json!()` au lieu de passer par `build_response`. Documenté §error-shapes. |
| L70 | Comptes 2979/2800 (résultat exercice / report à nouveau Sterchi PME) **exclus** de `total_liabilities` au bilan v0.1 | Pass 3 ECH3-01 : évite double-comptage avec `equity_result` calculé séparément. Const `EQUITY_RESULT_ACCOUNT_NUMBERS = &["2979", "2800"]` dans `kesh-report::balance_sheet`. Plans comptables custom (hors Sterchi) → CR v0.2 si numéros différents. |

### Risques et points d'attention pour le dev agent

1. **Q1 décision « affichage résultat au bilan »** : décision spec par défaut Option A. Si Pass 1 validate diverge → re-trancher. **Ne pas implémenter Options B ou C sans CR explicite**.

2. **Q2 archived account dans trial_balance** : le SQL T4.2 doit gérer les 2 cas (actif OR archivé avec écritures). Test AC #5 + #6 vérifie strictement. Attention au piège : `LEFT JOIN` ne suffit pas → besoin de `EXISTS` subquery sur `journal_entry_lines` pour archived comptes.

3. **Q3 audit sur consultation** : décision spec = audit émis. **Best-effort** (warn! sur INSERT failure, ne pas faire échouer la response). Pattern distinct de l'audit transactionnel des mutations. AC #25 + #26 vérifient.

4. **R2 conformité CO 957a** : la recherche réglementaire (action item retro Epic 8 owner Guy) doit avoir été faite **avant validate Pass 1**. Si elle ne l'est pas, valider quand même avec Options par défaut + créer GitHub Issue CR pour formats légaux post-recherche.

5. **Equation bilan = égalité Decimal exacte** : `equation_holds = (total_assets == total_liabilities + equity_result)`. Pas de tolérance. Si Decimal arithmétique cause un mismatch (théoriquement impossible — les `Decimal` sont exacts), c'est un bug DB ou un bug dans la logique d'agrégation → log error! + frontend affichera un badge rouge.

6. **Trial balance unbalanced** : si `total_debit != total_credit` côté agrégation, c'est un invariant cassé (toutes les écritures `journal_entries` sont équilibrées par construction — voir `journal_entries.rs:201` « SUM(debit) = SUM(credit) » post-INSERT check). Si l'agrégation cross-période casse l'invariant, c'est un bug → `ReportError::TrialBalanceUnbalanced` + log error! + HTTP 500. **Ne pas masquer**.

7. **Audit log emit on success only** : `report.generated` audit insert se fait **après** le SELECT métier réussi. **Pas d'audit sur 400/404/500** (AC #26). Le pattern : `let result = generate(...).await?; emit_audit(...); Ok(Json(result))`.

8. **Multi-tenant strict** : `company_id = current_user.company_id` extrait du JWT. **Toutes les requêtes SQL filtrent par `company_id` ET `fiscal_year_id` ET (selon module) account.company_id**. Cross-tenant attaqué via path traversal `fiscalYearId={fyOfCompanyB}` → 404 (pattern KF-002 Pattern 1). Test AC #16 vérifie ground-truth.

9. **Ordre stable des sections journal** : pour `journal_report`, l'ordre `Achats, Ventes, Banque, Caisse, OD` est **fixe** (cf. AC #7). Ne **pas** trier alphabétiquement. Convention métier suisse.

10. **Période out of fiscal_year edge cases** : tester `periodStart < fy.start_date` ET `periodEnd > fy.end_date` ET `periodStart > fy.end_date`. Test AC #13 vérifie au moins le cas `periodEnd > fy.end_date`. Ajouter cas symétriques si validate le demande.

11. **Path-dépendance 9-2** : 9-2 consommera `kesh_report::BalanceSheet`, `IncomeStatement`, `TrialBalance`, `JournalReport` pour générer PDF/CSV. **Garder ces types `pub` dans `lib.rs` + docstring `///` complète** (pattern helper public 8-5a-base). Aucun breaking change post-merge 9-1 sans CR.

12. **Pas de splitting préventif** : la story 9-1 touche 6 modules top-level (kesh-report, kesh-api/routes/reports, kesh-api/errors, kesh-api/lib, frontend/features/reports, frontend/routes/reports + i18n). Borderline règle CLAUDE.md (> 5 modules). Mais : modules **tous nouveaux** (pas de refactor cross-cutting), pattern **uniforme** entre les 4 rapports, **pas de breaking change**. **Pas de split préventif justifié.** Splitting safety-net (cf. §Lessons) : si validate diverge > 4 passes → split en 9-1a/b/c.

13. **Format d'erreur UX** : tous les messages d'erreur frontend doivent dire « ce qui s'est passé ET ce que l'utilisateur peut faire » (UX-DR38). Clés `reports-error-period-out-of-fiscal-year` + `reports-error-no-entries-in-period` formulées ainsi (cf. §i18n-keys).

14. **i18n DE/IT/EN-CH traductions basiques** : reporter `# TODO official translation` (cohérent L51 8-5b). Le dev-story crée des traductions « plausible » FR-DE-IT-EN (« Bilan » → « Bilanz », « Bilancio », « Balance Sheet », etc.) qui sont suffisamment correctes pour passer la lint et permettre une visualisation UI plausible. Traducteur officiel v0.2.

### R9 (Pass 1 BH-17) — KF-022 E2E helpers 401 partiels (epic-9.md mention)

KF-022 (GitHub #54) — E2E helpers d'authentification 401 partiels. Story 9-1 ajoute des routes `/api/v1/reports/*` ; si les helpers Playwright 401 sont flaky, T12 peut flake. **Action dev-story** : vérifier que `reports.spec.ts` utilise la même procédure de login que les autres tests E2E stables post-Epic 8 (ex. `bank-account-journal-link.spec.ts`). Si flake observé → tracer KF dérivée + corriger en post-merge.

### R10 (Pass 1 ECH-10) — Cohérence de lecture sous READ COMMITTED

MariaDB InnoDB default = `READ COMMITTED`. Pendant un SELECT rapport, une nouvelle `journal_entry` insérée par un autre user (réconciliation, saisie manuelle) en transaction atomique : **invisible** au SELECT en cours tant que la tx n'est pas committée. Le pattern `journal_entries::create_in_tx` (cf. `crates/kesh-db/src/repositories/journal_entries.rs:55+`) garantit que les écritures sont **toujours visibles complètes** (en-tête + lignes équilibrées) ou pas du tout. **Conclusion** : `READ COMMITTED` est safe v0.1, pas besoin de `REPEATABLE READ`. Documenter dans Dev Notes.

### R11 (Pass 1 ECH-16) — Exercices non-calendaires (juillet-juin) isolation

Un exercice fiscal suisse peut s'étendre 2025-07-01 → 2026-06-30. Le double filtre `entry_date BETWEEN ? AND ? AND fiscal_year_id = ?` protège correctement, mais le dev agent ne doit **PAS** omettre l'un des deux filtres. Test T11 dédié `fiscal_year_non_calendar_isolation` à ajouter.

### R12 — ARCHIVÉ (Pass 3 BH3-16)

KF-002 historique résolu Story 6.2. `CurrentUser.company_id: i64` (non-Option, `middleware/auth.rs:32`) + `User.company_id: i64` (`entities/user.rs:117`) — non-nullable depuis Epic 6. Plus de risque NULL v0.1. R12 archivée.

### R13 (Pass 1 ECH-17) — Écritures avec 50+ lignes

Aucune contrainte DB ne limite `journal_entry_lines` par écriture. v0.1 retourne la totalité dans `JournalEntryRow.lines`. Si > 50 lignes par écriture observé en prod → CR pour pagination intra-écriture v0.2.

### R14 (Pass 2 ECH2-06 + Pass 3 BH3-15) — Precision DECIMAL(19,4) vs `rust_decimal`

`journal_entry_lines.{debit,credit}` stocke `DECIMAL(19,4)` (19 digits totaux − 4 décimales = **15 digits avant virgule = max ~999 billions CHF** ≈ 10^15, **PAS 99 billions** — correction Pass 3 BH3-15). Les agrégations `SUM(debit)` côté SQL préservent exactement la précision en MariaDB InnoDB (pas de cast Float). `rust_decimal::Decimal` côté Rust gère arbitraire jusqu'à ~28 chiffres significatifs. **Aucun risque d'arrondi v0.1** pour les datasets attendus (CHF privé/PME ≤ quelques millions). Vérification via T11 test « insérer ligne `debit=99999999.9999` → SUM exact retourné ». Test optionnel.

### R15 (Pass 2 ECH2-08) — `equity_result` calcul édge NULL

Si l'INSERT `SUM(...)` retourne `NULL` (0 lignes correspondantes), `COALESCE(SUM(...), 0)` ramène à `Decimal::ZERO`. **L'ordre des `COALESCE`** est important : utiliser `COALESCE(SUM(debit), 0) - COALESCE(SUM(credit), 0)` (pas `COALESCE(SUM(debit) - SUM(credit), 0)` — édge cases NULL différents). Cf. §scope §3-§6 SQL explicite.

### R16 (Pass 2 ECH2-09) — Audit log INSERT partiellement échoué

Si `audit_log::insert_in_tx` retourne `DbError::Invariant("rows_affected == 0")` (cf. `audit_log.rs:42-50`), le `warn!` affiche l'erreur **mais l'INSERT a peut-être eu lieu** (avant un trigger qui l'a annulé). Pattern best-effort : on log et continue. **Garantie atomique** : l'INSERT audit est dans une transaction dédiée auto-commit ; si la transaction rollback (erreur), aucune row partielle n'est créée. Aucune action additionnelle requise v0.1.

### R17 (Pass 2 BH2-10) — `equity_result` 2 queries SQL non-atomiques

Le calcul `balance_sheet.equity_result` appelle `income_statement::generate` qui exécute 1 query SQL distincte du balance_sheet (2 queries indépendantes). Sous `READ COMMITTED`, si une nouvelle écriture est committée entre les 2 queries, `equity_result` et `total_liabilities` peuvent voir des snapshots légèrement différents (divergence ≤ quelques centimes en pratique). **Acceptable v0.1** — documenter en L65 comme dette traçable. **Optimisation v0.2** : query SQL unifiée OU transaction snapshot REPEATABLE READ.

### Références

- [`epic-9.md`](../planning-artifacts/epic-9.md) — spec de l'épic (Story 9-1 §Stories, R1-R8 risques, action items retro Epic 8).
- [`epics.md`](../planning-artifacts/epics.md) lignes 1104-1137 — section legacy « Epic 8 : Rapports & Exports » (avant renumérotage Epic 8 → 9), drift connu non corrigé (CR-009 #61 pattern).
- [`prd.md`](../planning-artifacts/prd.md) ligne 474-478 — FR65, FR66, FR67, FR68. Ligne 189 — CO art. 957-964. Ligne 200 — formats suisses.
- [`architecture.md`](../planning-artifacts/architecture.md) §3 décisions #12 (kesh-report dédiée) et #13 (kesh-i18n transversale), §11.5 workspace Cargo, §17 cartographie FR65-FR70, §Format Patterns ligne 344, §Naming Patterns ligne 281, §Frontières Architecturales ligne 645.
- [`ux-design-specification.md`](../planning-artifacts/ux-design-specification.md) §43 personas (Marc, Sophie, Lisa), §380-386 « Export global en un clic » (pour 9-2, mais conceptualise la souveraineté qui inspire 9-1 aussi), §UX-DR38 messages erreur actionnables.
- [`8-5b-reconciliation-rules-engine.md`](8-5b-reconciliation-rules-engine.md) — patterns hérités : repository Executor générique, audit_log atomique, multi-tenant scoping, helper public stable. Cycle review 2 passes (Sonnet → Haiku), valeur de référence pour 9-1.
- [`crates/kesh-report/`](../../crates/kesh-report/) — coquille vide actuelle à instancier.
- [`crates/kesh-i18n/src/formatting.rs`](../../crates/kesh-i18n/src/formatting.rs) — `format_money`, `format_date` à réutiliser.
- [`crates/kesh-db/src/repositories/fiscal_years.rs:399`](../../crates/kesh-db/src/repositories/fiscal_years.rs) — `find_by_id_in_company`.
- [`crates/kesh-db/src/repositories/audit_log.rs:26`](../../crates/kesh-db/src/repositories/audit_log.rs) — `insert_in_tx`.
- [`crates/kesh-db/migrations/20260411000001_accounts.sql`](../../crates/kesh-db/migrations/20260411000001_accounts.sql) — schéma accounts (account_type, active).
- [`crates/kesh-db/migrations/20260412000001_journal_entries.sql`](../../crates/kesh-db/migrations/20260412000001_journal_entries.sql) — schéma journal_entries + journal_entry_lines.
- [`crates/kesh-qrbill/Cargo.toml`](../../crates/kesh-qrbill/Cargo.toml) — `printpdf 0.7` candidat 9-2 (référence pour future décision R3 epic-9.md).

## Dev Agent Record

### Agent Model Used

À renseigner par le dev agent au moment de l'implémentation.

### Debug Log References

(à compléter par dev-story)

### Completion Notes List

(à compléter par dev-story)

### File List

(à compléter par dev-story)

## §pass-1-clarifications — Synthèse des décisions Pass 1 Sonnet 4.6

Référence rapide pour le dev agent. Tous les `Pass 1 <CODE>` dans la spec ci-dessus pointent ici.

| Code | Sujet | Décision verrouillée |
|---|---|---|
| **BH-01 / ECH-09** | `audit_log.entity_id` NOT NULL | Sentinelle `AUDIT_ENTITY_ID_NONE: i64 = 0`. Pas de migration DB. |
| **BH-02** | Ordre params `find_by_id_in_company` | `(pool, company_id, id)` — cf. `fiscal_years.rs:399` |
| **BH-03** | Cargo deps workspace | Versions directes alignées sur `kesh-db/Cargo.toml` (pas de `[workspace.dependencies]` root) |
| **BH-04** | Feature `rust_decimal` serde | `"serde-str"` (PAS `"serde-with-str"`). Module path `rust_decimal::serde::str` |
| **BH-05** | `ReportPeriod` derives | `Debug, Clone, Serialize` + `#[serde(rename_all = "camelCase")]` |
| **BH-06** | Formatage suisse frontend | `formatSwissAmount` depuis `$lib/features/journal-entries/balance.ts:93` (PAS `formatting.ts`) |
| **BH-07** | `kesh-i18n` dep 9-1 | **PAS** dépendance — ajoutée en 9-2 pour PDF |
| **BH-08** | Nom helper `period.rs` | `resolve` (unique — pas `from_query`) |
| **BH-10** | `Journal` enum | `kesh_db::entities::journal_entry::Journal` (sqlx traits) |
| **BH-12** | Period dans response | Toujours présente (`response.period.{startDate, endDate, fiscalYearId}`) ; AC #31 vérifie |
| **BH-13** | Helper `compute_net_result` | Pas de helper — appel direct `income_statement::generate(...).net_result` |
| **BH-15** | `AccountBalance.account_type` | Ajouté pour cohérence avec `TrialBalanceRow` |
| **ECH-01** | SUM sur 0 ligne | `COALESCE(SUM(debit), 0)` / `COALESCE(SUM(credit), 0)` dans tous les agrégats SQL |
| **ECH-02** | Période asymétrique | Table de résolution 4 cas (None/Some × 2). Asymétrie OK. |
| **ECH-03** | Comptes actifs sans écriture | Inclus avec `balance: 0` dans trial_balance (CO 957a vue complète) |
| **ECH-04** | Equity en `Liability` | v0.1 : fonds propres permanents dans `total_liabilities` + `equity_result` = résultat exercice seul. L61. |
| **ECH-05** | Journaux vides | 5 sections **toujours** présentes même vides (`entries: []`) si `journal_filter = None` |
| **ECH-06** | `fiscalYearId > 0` validation | Check handler-side explicite `if fiscal_year_id <= 0 → AppError::Validation` |
| **ECH-08** | `journal` case-sensitive | Casse exacte `Achats|Ventes|Banque|Caisse|OD` ; AC #22 message inclut les 5 valeurs |
| **ECH-10** | Cohérence READ COMMITTED | Safe v0.1 (atomicité `journal_entries::create_in_tx` garantit visibilité complète) |
| **ECH-11** | `equity_result` négatif | Section dédiée frontend « Perte de l'exercice » rouge ; positif → vert ; zéro → neutre |
| **ECH-12** | FY list vide UI | Bouton « Générer » `disabled` + message `reports-error-no-fiscal-year-available` (AC #34) |
| **ECH-13** | Overflow i64 | Géré via Axum `QueryRejection` → mappé `AppError::Validation` |
| **ECH-14** | Hiérarchie `parent_id` | Non exploitée v0.1 (tous comptes traités comme feuilles). L63. |
| **ECH-15** | Pattern audit best-effort | `match emit_report_audit(...).await { Err(e) => warn!(...), Ok(_) => {} }` (PAS `?`) |
| **ECH-16** | Exercice non-calendaire | Test T11 `fiscal_year_non_calendar_isolation` |
| **ECH-17** | Écritures > 50 lignes | Aucune limite v0.1, CR si observé. L63. |
| **ECH-18** | Période même jour | Autorisée (BETWEEN inclusif). Test T1.5 `period_same_day_is_valid` |
| **ECH-19** | Traductions IT/DE/EN-CH | Chaque clé DOIT avoir valeur texte (pas que `# TODO`). Fluent retourne clé brute si valeur absente |
| **ECH-20** | Param `journal` dupliqué | Axum `Query` prend la dernière occurrence (déterministe). Pas d'action |
| **ECH-21** | `current_user.company_id` NULL | Garde dans extractor (héritée 7-1). Vérifier au dev-story |
| **ECH-22** | Variables Fluent dates | Frontend convertit ISO → `dd.mm.yyyy` via `formatSwissDate` avant passage à Fluent |
| **AA-01** | ORDER BY income_statement | `ORDER BY a.account_number ASC` ajouté T3.2 + test T3.4 |
| **AA-02 / BH-09** | Shape JSON body 400 | 4 champs `{ fyStart, fyEnd, requestedStart, requestedEnd }` camelCase ; AC #13 assert les 4 |
| **AA-03 / BH-14** | AC #18 audit scoping | Reformulé : JOIN `users` requis (audit_log n'a pas `company_id` direct) |
| **AA-04** | AC #25 flake test E2E | Pool test ≥ 4 connexions ; tracé L62 |
| **AA-05** | Messages erreur API | Asserter sur `error.code` (`VALIDATION`, etc.), PAS sur texte FR. UX final via Fluent frontend |
| **AA-06, AA-07, AA-08, AA-09, AA-10** | Tests T10.2 manquants | T10.2 augmenté 20 → 28 tests (ACs orphelins couverts) |
| **AA-11 / BH-11** | AC #29 condition unifiée | Helper `isReportEmpty(reportType, dto)` dans `reports.api.ts` ; 4 conditions par type |
| **AA-12** | T11 vs T10.2 redondance | Non-redondant : T11 SQL agrégats kesh-db, T10 HTTP end-to-end. Documenté |
| **AA-13** | Q3 audit ambiguïté | TRANCHÉE : audit en 9-1 best-effort. CR si R2 recherche révèle conflit |
| **AA-15** | Perf strict pass/fail | Test `#[ignore]` activable manuellement + check `EXPLAIN` en code review |
| **BH-17** | R9 KF-022 E2E helpers | Surveillé au dev-story ; même procédure login que `bank-account-journal-link.spec.ts` |
| **BH-18** | Test journal section vide | Test T5.5 `filter_none_with_missing_journal_returns_5_sections_with_empty_one` |
| **BH-19** | Warning `rename_all` query params | Noté dans T6.2 — sans `rename_all`, query params seraient snake_case en URL |

### Pass 2 Haiku 4.5 — codes additionnels

| Code | Sujet | Décision verrouillée |
|---|---|---|
| **AA2-01** (CRITICAL) | Mapping snake_case → camelCase IntoResponse | DTO intermédiaire `#[serde(rename_all = "camelCase")]` dans `kesh-api/src/errors.rs` (snippet T6.4 fourni) |
| **BH2-01** | 4 clés i18n manquantes bloc Fluent | Bloc Fluent §i18n-keys complété 34 clés (4 labels + 7 colonnes + 5 sections + 8 totaux + 4 filtres + 3 erreurs UX + 3 equity-result) |
| **BH2-02** | Refs stale `kesh-i18n` dep `kesh-report` | 4 sites corrigés (ligne 24, 49, Dev Notes Patterns, §Affichage) — `kesh-i18n` reporté 9-2 |
| **BH2-03** | Counts tests T10 stale (20 vs 28) | Tous les sites synchronisés à ≥ 28 tests |
| **BH2-04** | Counts tests T11 stale (6 vs 7) | Tous les sites synchronisés à ≥ 7 tests sqlx |
| **AA2-02** | Doublon apparent AC #23 vs #32 | AC #23 reformulé : couverture 3 rôles ; AC #32 = cas concret Consultation testé (le plus restrictif) |
| **AA2-03** | AC #34 dépendance route externe | `GET /api/v1/fiscal-years` confirmé existant dans `crates/kesh-api/src/lib.rs:321-322` (Story 3-7) |
| **AA2-06** | `serde(rename_all)` explicite manquant T6.2 | Code snippet complet ReportQuery + JournalReportQuery avec derives obligatoires |
| **AA2-09** | AC #31 period dans response — toutes routes | Test T10.2 #11 renommé `default_period_uses_fiscal_year_full_range_all_endpoints` |
| **AA2-11** | balance_sheet règle archived rules | Explicite §3 : actifs sans écriture exclus (épure bilan), archived avec écritures inclus |
| **AA2-12** | T11 seed structure non détaillée | Fixture obligatoire documentée : 1 company + 1 fy non-calendaire + 5 comptes + 1 archivé + 3 écritures multi-journaux |
| **ECH2-01 / ECH2-10 / AA2-04** | Sentinelle entity_id=0 scope | Déplacée dans `kesh-db/src/entities/audit_log.rs` (trans-crate). Filtre toujours sur `(entity_type, entity_id)` jamais `entity_id` seul. Future-proof |
| **ECH2-02** | Période inversée asymétrique (None, Some(e<start)) | Géré par check `s ≤ e` après résolution → 400 `VALIDATION`. L66 dette UX message v0.2 |
| **ECH2-04** | TZ ambiguïté dates ISO | DATE calendar-day pure (pas datetime), aucune conversion TZ backend. L67 documenté |
| **ECH2-06** | DECIMAL(19,4) precision | R14 : précision préservée v0.1 jusqu'à ~99 billions CHF (au-delà non testé) |
| **ECH2-07** | Pool DB ≥ 4 connexions CI | L62 enrichi : env var `SQLX_MAX_CONNECTIONS=4` à ajouter `.github/workflows/ci.yml` |
| **ECH2-08** | `equity_result` NULL edge | R15 : ordre `COALESCE` explicite — `COALESCE(SUM(debit), 0) - COALESCE(SUM(credit), 0)`, pas l'inverse |
| **ECH2-09** | Audit partial INSERT | R16 : `audit_log::insert_in_tx` atomique (rollback si erreur), pas de row partielle possible |
| **BH2-10** | 2 queries SQL non-atomiques (balance + income) | R17 + L65 dette traçable v0.2 query unifiée OU snapshot REPEATABLE READ |
| **BH2-05** | AC #3 ordering passif | Assertion test explicite (T3.4 `net_result_ordering_by_account_number`) — pas de patch supplémentaire requis |
| **AA2-05** | 4 champs présence non-garantie JSON | DTO `#[serde(rename_all = "camelCase")]` sans `#[serde(skip_serializing_if)]` → champs toujours présents |
| **AA2-07** | Mapping test → sub-cases unclear | T10.2 tests #20-#28 enrichis : « 2 sub-cases » documentées (e.g., `date_malformed` couvre `2026/01/15` ET `2026-02-30`) |
| **AA2-08, AA2-10, AA2-13, AA2-14, AA2-15, ECH2-11..15** | LOW résiduels | Documentation incrémentale ou edges peu probables — non bloquants dev-story |

## Change Log

| Date | Entrée | Auteur |
|------|--------|--------|
| **2026-05-14** | **`bmad-create-story validate 9-1` Pass 3 Opus 4.7 — 3 reviewers parallèles fresh-context — BRISE BIAIS CONVERGENT Pass 1+2** — 50+ findings bruts (BH3 20 + ECH3 15 + AA3 6) → 25+ distincts post-dedup. **VERDICT DIVERGENT INTER-REVIEWERS** : Acceptance Auditor Opus = GO sans condition (rubber-stamp biais convergent), Edge Case Hunter Opus = CONDITIONAL GO (2 HIGH ground-truth), **Blind Hunter Opus = NO-GO avec 6 CRITICAL ground-truth ratés Pass 1+2**. Le pattern documenté 8-5b retro confirmé : Sonnet+Haiku ont rubber-stampé les snippets de code sans grep ground-truth, Opus fresh-context a brisé le biais en lisant le code merged. **Findings CRITICAL Pass 3 BH ground-truth (6)** : (1) BH3-01 `AppError::Validation` est tuple variant `Validation(String)` (errors.rs:65-66), spec utilise syntaxe struct → ne compile pas ; (2) BH3-02 code erreur réel `"VALIDATION_ERROR"` pas `"VALIDATION"` (errors.rs:513), 10+ tests E2E existants confirment ; (3) BH3-03 `ErrorBody { error: ErrorDetail { code, message } }` (errors.rs:469-478) n'a PAS de champ `details` — schema AC #13 nécessite refactor ou body JSON ad-hoc divergent ; (4) BH3-04 `"INTERNAL_ERROR"` pas `"INTERNAL"` (errors.rs:544) ; (5) BH3-05 `current_user.user_id` pas `current_user.id` (auth.rs:32) ; (6) BH3-06 `CurrentUser` dans `middleware/auth.rs:29` PAS `extractors.rs` (fichier inexistant), pattern `Extension<CurrentUser>` injecté par middleware. **HIGH (10 post-dedup)** : BH3-07 emit_report_audit wrap pool.begin/commit manquant ; BH3-08 `kesh-report = { workspace = true }` ne compile pas → `path` ; BH3-09 sqlx feature `json` clarifier ; BH3-10 T9.1/T9.2 stale 30 vs 34 clés ; BH3-11 seed type 'Equity' viole CHECK constraint → Liability explicite ; BH3-12 Axum Query rejection default `text/plain` casse ACs #19-#22 → reformuler status uniquement ; BH3-13 `#[serde(flatten)]` ne marche pas avec `serde_urlencoded` → dupliquer champs JournalReportQuery ; BH3-14 SQL T2.2 manque préfixe `je.` ambiguité JOIN ; ECH3-01 compte 2979 « Résultat exercice » dans plans comptables seed (PME/indépendant/association) double-compte avec equity_result → exclusion via const `EQUITY_RESULT_ACCOUNT_NUMBERS = &["2979", "2800"]` ; ECH3-02 `AUDIT_ENTITY_ID_NONE` non re-exporté dans `entities/mod.rs`. **MEDIUM (6)** : BH3-15 R14 magnitude DECIMAL(19,4) ~10^15 pas 99 billions ; BH3-16 R12 archivé (KF-002 résolu 6.2) ; BH3-17 AC #1 fragile au seed ; BH3-18 T13.3 ajout `SQLX_MAX_CONNECTIONS=4` à `.github/workflows/ci.yml` ; BH3-19 T8.7 test coloration equity_result rouge/vert dédié ; BH3-20 helper Playwright seed company-without-fy documenté. Décision Guy Option A — **~22 patches CRITICAL + HIGH + MEDIUM appliqués** : substitutions globales tuple variant + VALIDATION_ERROR + INTERNAL_ERROR + user_id + middleware/auth.rs + workspace → path + 30 → 34 i18n keys ; refactor §error-shapes shape JSON ad-hoc divergent ErrorBody documenté (L69) ; snippet IntoResponse complet 3 variants (ReportPeriodOutOfFiscalYear + ReportFiscalYearNotFound + Validation pattern existant) ; T6.2 ReportQuery + JournalReportQuery sans flatten (champs dupliqués + helper as_report_query) ; T2.2 SQL préfixe `je.` systématique ; T10.1/T11.1 seed Equity → Liability explicite + énumération 3 écritures équilibrées ; balance_sheet exclusion 2979/2800 + const EQUITY_RESULT_ACCOUNT_NUMBERS + test T2.5 dédié manual_entry_to_2979_does_not_double_count ; AUDIT_ENTITY_ID_NONE re-export entities/mod.rs (1 ligne) ; emit_report_audit pattern complet pool.begin/insert_in_tx/commit ; ACs #19/#20/#21/#22 reformulés status = 400 (text/plain Axum default), AC #20-bis distinct JSON validation handler ; T13.3 modif ci.yml ; T8.7 test coloration equity ; R12 archivée + R14 magnitude corrigée + L65-L70 dettes tracées. Spec ~1300 → ~1500 lignes (+200 nettes). 34 ACs (33 nominaux + #20-bis sub) inchangés. Trend complet : Pass 1 = 17+ > LOW → Pass 2 = 17 > LOW (defauts complémentaires Haiku) → **Pass 3 = 20 > LOW (Opus BH brise biais)** → estimé 0-3 HIGH post-Pass-3-patch. **Critère arrêt CLAUDE.md NON encore atteint** — Pass 4 obligatoire (cycle Opus → Sonnet 4.6, briser biais Opus auteur Pass 3 + valider que les ~22 patches Pass 3 n'introduisent pas de régression). Budget 3/8 passes consommé. Splitting préventif évalué : pas déclenché (convergence en cours, 4 passes max sans split = règle CLAUDE.md respectée). | Claude (Opus 4.7 validate Pass 3 — 3 reviewers parallèles fresh-context) |
| **2026-05-14** | **`bmad-create-story validate 9-1` Pass 2 Haiku 4.5 — 3 reviewers parallèles fresh-context COMPLETED** — 38 findings bruts (BH2 8 + ECH2 15 + AA2 15) → ~28 distincts post-dedup inter-reviewers (collisions sentinelle ECH2-01/AA2-04/ECH2-10, validation handler ECH2-05/AA2-06, mapping camelCase AA2-01). Triage : 1 CRITICAL (AA2-01 IntoResponse mapping snake_case Rust → camelCase JSON non documenté pour AppError::ReportPeriodOutOfFiscalYear — test AC #13 fail certain sans correction), 6 HIGH (BH2-01 4 clés i18n manquantes bloc Fluent + BH2-02 3 refs stale kesh-i18n + BH2-03 counts T10 stale + AA2-03 dép externe `/api/v1/fiscal-years` + AA2-06 serde rename_all explicite + ECH2-07 pool CI), ~10 MEDIUM (sentinelle scope, AC #23/#32 doublon, balance_sheet vs trial_balance archived rules, T11 seed, AC #31 4 endpoints, BH2-04 T11 count, BH2-10 2 queries non-atomiques, ECH2-06 precision, ECH2-08 NULL equity, ECH2-09 audit partial), ~12 LOW. Verdicts Auditor CONDITIONAL GO + Hunter CONDITIONAL GO + ECH CONDITIONAL GO. Décision Guy Option A — **~19 patches CRITICAL+HIGH+MEDIUM appliqués** : (1) IntoResponse snippet camelCase DTO intermédiaire dans §error-shapes (CRITICAL AA2-01 résout) ; (2) bloc Fluent §i18n-keys complété 34 clés (BH2-01) ; (3) 4 refs `kesh-i18n` corrigées (ligne 24, 49, Dev Notes, §Affichage) — confirmation pas dep `kesh-report` v0.1 ; (4) counts T10 `≥ 28` + T11 `≥ 7` synchronisés à tous les sites ; (5) T6.2 code snippet ReportQuery + JournalReportQuery avec `#[derive(Deserialize)] #[serde(rename_all = "camelCase")]` explicite ; (6) AC #34 vérif route `GET /api/v1/fiscal-years` existante `kesh-api/src/lib.rs:321-322` (Story 3-7) ; (7) L62 enrichi pool CI `SQLX_MAX_CONNECTIONS=4` env var requis ; (8) AC #23 reformulé couverture 3 rôles (cas concret AC #32 Consultation) ; (9) balance_sheet règle archived rules explicite (différent trial_balance) ; (10) sentinelle scope kesh-db/src/entities/audit_log.rs (trans-crate future-proof Epic 14+15) + règle filtre `(entity_type, entity_id)` jamais `entity_id` seul ; (11) T11.1 seed structure obligatoire (1 fy non-calendaire + 5 comptes + 1 archivé + 3 écritures multi-journaux) ; (12) AC #31 test T10.2 #11 toutes routes (4 endpoints assertés) ; (13) R14 precision DECIMAL(19,4) + R15 ordre COALESCE explicite + R16 audit atomic + R17 2 queries non-atomiques + L65-L67 dettes tracées ; (14) §pass-1-clarifications enrichi avec 23 codes Pass 2 (AA2-01-15 + BH2-01-19 + ECH2-01-22 condensés). Spec 1095 → ~1300 lignes (+200 nettes), 34 ACs inchangés, sous-tâches affinées. Trend Pass 1 = 17+ > LOW → Pass 2 pré-patch = 17 > LOW → **Pass 2 post-patch estimé 0-2 HIGH résiduel** (à confirmer Pass 3 Opus 4.7). **Critère arrêt CLAUDE.md NON encore atteint** — Pass 3 obligatoire (cycle CLAUDE.md Sonnet → Haiku → Opus, briser biais Haiku + valider patches sans régression). Budget 2/8 passes consommé. | Claude (Haiku 4.5 validate Pass 2 — 3 reviewers parallèles fresh-context) |
| **2026-05-14** | **`bmad-create-story validate 9-1` Pass 1 Sonnet 4.6 — 3 reviewers parallèles fresh-context COMPLETED** — 56 findings bruts (BH 19 + ECH 22 + AA 15) → 54 distincts post-dedup (2 doublons : BH-01/ECH-09 audit entity_id NOT NULL + BH-09/AA-02 shape JSON body error). Triage : 3 CRITICAL (BH-01 audit entity_id, BH-02 ordre params find_by_id_in_company, BH-03 workspace deps), 14 HIGH, ~24 MEDIUM, ~12 LOW. Verdict Acceptance Auditor : CONDITIONAL GO. Verdict Blind Hunter : NO-GO sur 3 CRITICAL. Décision Guy Option A — **~41 patches CRITICAL + HIGH + MEDIUM appliqués** : (T1.1) Cargo deps versions directes alignées kesh-db (sqlx="0.8", rust_decimal="1.41" feature serde-str, chrono="0.4", thiserror="2", tracing="0.1") + retrait kesh-i18n dep ; (§rust-types) Serialize sur ReportPeriod + account_type ajouté à AccountBalance + Journal de kesh-db explicite ; (§scope/T1.4) `resolve` unique nom + signature find_by_id_in_company(pool, company_id, id) + table de résolution asymétrique 4 cas + check `> 0` handler-side ; (§scope §3-§6) COALESCE(SUM(...), 0) partout + équation bilan v0.1 avec Equity-in-Liability documentée + comptes actifs sans écriture inclus + 5 sections journaux toujours présentes + ORDER BY accountNumber sur income_statement + hiérarchie parent_id non exploitée v0.1 ; (§audit-shapes) sentinelle AUDIT_ENTITY_ID_NONE=0 + pattern code best-effort match (pas de `?`) + JOIN users pour scope multi-tenant ; (§error-shapes) shape JSON 4 champs camelCase + mécanisme validation > 0 + mapping rejection Query<T> + case-sensitivity Journal documenté + param dupliqué déterministe ; (ACs) 30 → 34 ACs (#31 period dans response, #32 rôle Consultation, #33 4 onglets Playwright, #34 FY vide UI disabled) + ACs #3/#13/#14/#18/#19-22/#25/#29 reformulés avec assertions précises ; (T10.2) 20 → 28 tests E2E HTTP énumérés (tests #20-#28 ajoutés : malformed/zero-negative/date-invalid/journal-invalid/consultation/audit-sentinel/empty-period) ; (T11) ≥ 7 tests sqlx avec fiscal_year_non_calendar_isolation + clarification non-redondance T10 ; (T12) 4 onglets assertion + scénario FY vide ; (frontend) helper isReportEmpty unifié + formatSwissAmount (PAS formatting.ts) + section equity_result frontend rouge/vert + conversion ISO→dd.mm.yyyy Fluent ; (i18n) 30 → 34 clés (+ no-fiscal-year-available + equity-result × 3) + règle valeur obligatoire pour chaque locale ; (§Risques) R9 KF-022 + R10 READ COMMITTED + R11 non-calendaire + R12 company_id NULL + R13 écritures longues ; (§Limitations) L52 → L52+L61+L62+L63+L64 ; (Q3 trail décidée définitivement : audit 9-1 best-effort, CR si R2 conflit) ; (T2.3 BH-13 helper compute_net_result remplacé par appel direct income_statement::generate) ; (§performance perf_smoke manuel obligatoire code review). Spec 831 → ~1100 lignes, 30 → 34 ACs, 52 → ~60 sous-tâches. Trend Pass 1 = 17+ findings > LOW pré-patch. **Critère arrêt CLAUDE.md NON encore atteint** — Pass 2 obligatoire (Haiku 4.5 cycle Sonnet → Haiku, briser biais Sonnet auteur Pass 1 + valider que les ~41 patches n'introduisent pas de régression). Budget 1/8 passes. | Claude (Sonnet 4.6 validate Pass 1 — 3 reviewers parallèles fresh-context) |
| **2026-05-14** | **`bmad-create-story 9-1` Opus 4.7 — spec initiale ready-for-dev** | Claude (Opus 4.7 — create-story) |
