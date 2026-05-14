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

- **9-1 (cette story)** livre la **génération de données** : 4 modules de calcul dans la crate `kesh-report` (actuellement coquille vide, cf. `crates/kesh-report/src/lib.rs`), 4 routes HTTP qui retournent du **JSON structuré** uniquement, et la page frontend qui rend ces données en HTML/Tailwind avec les formats suisses (apostrophe, dd.mm.yyyy) via `kesh-i18n`.
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

1. **Crate `kesh-report` instanciée (FR65)** — `Cargo.toml` avec deps `chrono`, `rust_decimal`, `serde`, `thiserror`, `sqlx` (feature `mysql` + `chrono` + `rust_decimal` + `runtime-tokio-rustls`), `kesh-core`, `kesh-db`, `kesh-i18n`. Structure :

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

2. **Module `period.rs` (commun, T1)** — type `ReportPeriod { start_date: NaiveDate, end_date: NaiveDate, fiscal_year_id: i64 }` + helper `from_query(pool, company_id, fiscal_year_id, period_start?, period_end?)` qui :
   - Vérifie que `fiscal_year_id` appartient à `company_id` (404 sinon).
   - Si `period_start`/`period_end` absents → utilise `(fy.start_date, fy.end_date)` complet.
   - Si présents → valide que l'intervalle est inclus dans `[fy.start_date, fy.end_date]` (400 `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` sinon).
   - Valide `period_start <= period_end` (400 `REPORT_PERIOD_INVALID` sinon).

3. **Module `balance_sheet.rs` (T2, FR65 bilan)** :
   - Fonction publique `pub async fn generate(pool: &MySqlPool, company_id: i64, period: &ReportPeriod) -> Result<BalanceSheet, ReportError>`.
   - Output struct `BalanceSheet { period, assets: Vec<AccountBalance>, liabilities: Vec<AccountBalance>, equity_result: Decimal, total_assets: Decimal, total_liabilities: Decimal, equation_holds: bool }`.
   - Algorithme : agrège `SUM(debit) - SUM(credit)` pour les comptes `account_type IN ('Asset','Liability')` filtrés par `entry_date BETWEEN period.start_date AND period.end_date AND fiscal_year_id = period.fiscal_year_id AND company_id = ?`. Pour les passifs, le solde « naturel » est `credit - debit`. `equity_result = total_revenues - total_expenses` calculé conjointement (résultat de l'exercice intégré aux capitaux propres au bilan v0.1 — Q1 décision pré-spec à trancher).
   - **Équation bilan** : `equation_holds = (total_assets == total_liabilities + equity_result)`. Tolérance 0 (rust_decimal exact, pas de tolérance float).
   - AC #1 — équation bilan vérifiée dans tous les cas du test fixture.

4. **Module `income_statement.rs` (T3, FR65 compte de résultat)** :
   - Fonction publique `pub async fn generate(pool, company_id, period) -> Result<IncomeStatement, ReportError>`.
   - Output struct `IncomeStatement { period, revenues: Vec<AccountBalance>, expenses: Vec<AccountBalance>, total_revenues: Decimal, total_expenses: Decimal, net_result: Decimal }`.
   - Algorithme : agrège par compte pour `account_type IN ('Revenue','Expense')`. Revenue : `credit - debit`. Expense : `debit - credit`. `net_result = total_revenues - total_expenses`.

5. **Module `trial_balance.rs` (T4, FR65 balance des comptes)** :
   - Fonction publique `pub async fn generate(pool, company_id, period) -> Result<TrialBalance, ReportError>`.
   - Output struct `TrialBalance { period, rows: Vec<TrialBalanceRow>, total_debit: Decimal, total_credit: Decimal, balanced: bool }`.
   - `TrialBalanceRow { account_number: String, account_name: String, account_type: AccountType, total_debit: Decimal, total_credit: Decimal, balance: Decimal }` (balance = solde net signé selon convention type).
   - Algorithme : 1 ligne par compte du plan comptable de la company (tous types, **y compris archivés** si écritures dans la période — Q2 décision pré-spec : afficher archivés Active=false avec marqueur si non-zéro). `total_debit = SUM(debit)`, `total_credit = SUM(credit)` agrégés sur la période.
   - **Invariant** : `balanced = (total_debit == total_credit)`. Si false → erreur métier `ReportError::TrialBalanceUnbalanced` (théoriquement impossible si `journal_entries` est correct, mais défense en profondeur — log `error!` + retour HTTP 500).
   - AC #3 — `total_debit == total_credit` testé sur fixture multi-écritures.

6. **Module `journal_report.rs` (T5, FR65 journaux)** :
   - Fonction publique `pub async fn generate(pool, company_id, period, journal_filter: Option<Journal>) -> Result<JournalReport, ReportError>`.
   - Output struct `JournalReport { period, journals: Vec<JournalSection>, grand_total_debit: Decimal, grand_total_credit: Decimal }`.
   - `JournalSection { journal: Journal, entries: Vec<JournalEntryRow>, section_total_debit: Decimal, section_total_credit: Decimal }`.
   - `JournalEntryRow { entry_id, entry_number, entry_date, description, lines: Vec<JournalEntryLineRow> }` avec `JournalEntryLineRow { account_number, account_name, debit, credit }`.
   - Si `journal_filter = None` → toutes les sections (Achats, Ventes, Banque, Caisse, OD) dans cet ordre fixe.
   - Si `journal_filter = Some(j)` → 1 seule section.
   - Algorithme : SELECT `journal_entries je JOIN journal_entry_lines jel ON jel.entry_id = je.id JOIN accounts a ON a.id = jel.account_id` filtré par company + période + (optionnel) journal, ORDER BY `je.journal, je.entry_date, je.entry_number, jel.line_order`.

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
   - `reports.api.ts` — 4 fetchers `getBalanceSheet(params)`, `getIncomeStatement(params)`, `getTrialBalance(params)`, `getJournalReport(params)`.
   - `ReportSelector.svelte` — UI : sélecteur d'exercice (chargé via `/api/v1/fiscal-years`), date range picker (period start/end optionnels), bouton « Générer ».
   - `BalanceSheetView.svelte`, `IncomeStatementView.svelte`, `TrialBalanceView.svelte`, `JournalReportView.svelte` — 4 vues qui affichent le rapport en tableau HTML Tailwind, montants formatés via `formatMoney` (Intl.NumberFormat équivalent suisse — cf. `frontend/src/lib/shared/utils/formatting.ts`).
   - `routes/(app)/reports/+page.svelte` — page hôte qui mount `ReportSelector` + chargement à la demande des 4 vues selon onglet actif.
   - `routes/(app)/reports/+page.ts` — load function pour charger la liste des fiscal_years.
   - Pas de bouton « Télécharger PDF » ni « Export CSV » v0.1 (livré 9-2). Tracker LXX explicitement.

10. **i18n (T9)** — **~30 nouvelles clés Fluent** × 4 locales (fr/de/it/en-CH) sous le préfixe `reports-` :
    - Labels rapports : `reports-balance-sheet`, `reports-income-statement`, `reports-trial-balance`, `reports-journals` (4 clés).
    - Colonnes : `reports-column-account-number`, `reports-column-account-name`, `reports-column-debit`, `reports-column-credit`, `reports-column-balance`, `reports-column-entry-date`, `reports-column-description` (7 clés).
    - Sections bilan : `reports-section-assets`, `reports-section-liabilities`, `reports-section-equity`, `reports-section-revenues`, `reports-section-expenses` (5 clés).
    - Totaux : `reports-total-assets`, `reports-total-liabilities`, `reports-total-revenues`, `reports-total-expenses`, `reports-total-debit`, `reports-total-credit`, `reports-net-result`, `reports-grand-total` (8 clés).
    - Filtres : `reports-filter-period`, `reports-filter-fiscal-year`, `reports-filter-journal`, `reports-button-generate` (4 clés).
    - Erreurs UX : `reports-error-no-entries-in-period`, `reports-error-period-out-of-fiscal-year` (2 clés UX-DR38).
    - **Total cible : 30 clés**. Lint `npm run lint-i18n-ownership` doit passer. Cf. §Risques R4.

11. **Audit log (T7 — décision Q3 pré-spec)** — option par défaut (à confirmer validate) : 1 action `report.generated` émise par chaque GET réussi avec `details_json = { reportType: 'balance-sheet'|'income-statement'|'trial-balance'|'journals', fiscalYearId, periodStart, periodEnd, journalFilter? }`. Justification : CO Art. 958f audit trail des consultations comptables. **Alternative** : audit uniquement sur génération PDF/CSV (= en Story 9-2). Décision Q3 à trancher validate Pass 1.

12. **Tests** :
    - **Unit `kesh-report`** (`#[cfg(test)] mod tests` inline dans chaque module) — ≥ 16 tests : balance_sheet (4 : équation, archivés, période partielle, multi-fiscal-year-isolation), income_statement (3 : résultat net positif/négatif/zéro), trial_balance (4 : balanced=true, balanced=false ground truth, account ordering, totaux), journal_report (5 : 5 journaux, filter Achats, ordering, vide, multi-line entries).
    - **Intégration `kesh-db`** (`#[sqlx::test]` dans nouveau fichier `crates/kesh-db/tests/report_aggregates.rs`) — ≥ 6 tests : multi-tenant cross-company (4 cross-tenant 0-rows), period filter (1 boundary), fiscal_year isolation (1).
    - **E2E HTTP `kesh-api`** (`crates/kesh-api/tests/reports_e2e.rs` nouveau) — ≥ 20 tests : 4 endpoints × { happy path, 401 unauth, 400 validation, 404 cross-tenant, 400 period out of FY } = 20 minimum.
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

**Décision par défaut spec** : **OUI**, action `report.generated` avec `entity_type = 'report'`, `entity_id = NULL` (pas d'entité concrète), `details_json = { reportType, fiscalYearId, periodStart, periodEnd, journalFilter? }`. Justification :
- CO Art. 958f audit trail des consultations.
- Coût : 1 INSERT audit_log par request (négligeable).
- Permet retracer « qui a vu quel rapport quand » pour conformité fiduciaire.

**Alternative** : audit uniquement sur PDF/CSV (9-2). **Refusée** car la consultation JSON est la première brique de l'audit trail — l'export est un sur-coût après.

**À valider validate Pass 1** : si recherche réglementaire R2 epic-9.md indique que seul l'export persistant compte comme « consultation soumise à audit », bascule sur 9-2 et tracer LXX dette.

**Pattern d'implémentation** : audit émis **après** succès du SELECT (pas avant — pas d'audit sur erreur 400/404/500). Insert audit dans une transaction dédiée (1 INSERT atomique). Si l'INSERT audit échoue → log `warn!`, ne pas faire échouer la réponse rapport (best-effort audit pour la consultation, distinct de l'audit transactionnel des mutations comptables qui DOIT bloquer la mutation si l'audit échoue).

#### Q4 — Format de date dans les query params HTTP

**Question** : `periodStart` / `periodEnd` en query param HTTP : format ISO 8601 (`YYYY-MM-DD`) ou suisse (`dd.mm.yyyy`) ?

**Décision** : **ISO 8601 (`YYYY-MM-DD`) côté API**. Justification :
- Architecture.md ligne 355 : « Dates : ISO 8601 (`2026-04-02`...) ».
- Frontend convertit `dd.mm.yyyy` (saisie UI) → `YYYY-MM-DD` (envoi API) — pattern KE déjà standard pour invoices, journal_entries.
- Évite ambiguïté locale (FR/DE/EN).
- **Affichage** dans les rapports : `dd.mm.yyyy` via `kesh-i18n::format_date` (cf. §i18n).

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

#[derive(Debug, Clone)]
pub struct ReportPeriod {
    pub fiscal_year_id: i64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl ReportPeriod {
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

**Sérialisation `Decimal`** : utiliser `serde_with::DisplayFromStr` ou `#[serde(with = "rust_decimal::serde::str")]` pour sortir en string `"1234.56"` côté JSON (architecture.md ligne 356). Vérifier que `rust_decimal` feature `serde-with-str` est activée dans `Cargo.toml`.

### §audit-shapes — Pattern audit log 9-1

Une seule nouvelle action audit : `report.generated`.

```rust
NewAuditLogEntry {
    user_id: current_user.id,
    action: "report.generated".to_string(),
    entity_type: "report".to_string(),
    entity_id: None,                              // Pas d'entité concrète
    details_json: serde_json::json!({
        "reportType": "balance-sheet",            // ou income-statement, trial-balance, journals
        "fiscalYearId": 12,
        "periodStart": "2026-01-01",              // ISO 8601 string
        "periodEnd": "2026-12-31",
        "journalFilter": null                     // ou "Achats", "Ventes", etc. pour journals
    }),
}
```

**Atomicité** : audit est inséré dans une transaction dédiée (1 INSERT) après le SELECT métier réussi. Si l'INSERT audit échoue → `warn!` + retour HTTP 200 quand même (best-effort consultation audit, **distinct du pattern transactionnel** des mutations comptables où audit DOIT bloquer la mutation).

### §error-shapes — Mapping `ReportError → AppError`

| `ReportError` variant | `AppError` variant | HTTP | Code métier exposé |
|---|---|---|---|
| `Db(e)` | `AppError::Db(e)` | 500 | `INTERNAL` |
| `FiscalYearNotFound` | `AppError::ReportFiscalYearNotFound` | 404 | `FISCAL_YEAR_NOT_FOUND` |
| `PeriodInvalid { reason }` | `AppError::Validation { reason }` | 400 | `VALIDATION` |
| `PeriodOutOfFiscalYear { .. }` | `AppError::ReportPeriodOutOfFiscalYear { fy_start, fy_end, requested_start, requested_end }` | 400 | `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` |
| `TrialBalanceUnbalanced { total_debit, total_credit }` | `AppError::Internal` (log error!) | 500 | `INTERNAL` |

4 nouveaux variants `AppError` à ajouter dans `crates/kesh-api/src/errors.rs` :

- `ReportFiscalYearNotFound`
- `ReportPeriodInvalid { reason: String }` (alias `Validation` ok aussi)
- `ReportPeriodOutOfFiscalYear { fy_start: NaiveDate, fy_end: NaiveDate, requested_start: NaiveDate, requested_end: NaiveDate }`

`From<ReportError> for AppError` impl factorise le mapping.

### §i18n-keys — 30 nouvelles clés `reports-*`

Cible spec : **30 clés** × 4 locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`).

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

# Erreurs UX (2)
reports-error-no-entries-in-period = Aucune écriture dans la période sélectionnée. Modifiez les dates ou choisissez un autre exercice.
reports-error-period-out-of-fiscal-year = La période sélectionnée dépasse les bornes de l'exercice. Choisissez une période entre { $fyStart } et { $fyEnd }.
```

**Traductions DE/IT/EN-CH** : version basique livrée par dev-story, traducteur officiel reporté v0.2 (cohérent limitation L51 héritée 8-5b). Marquer `# TODO official translation` en commentaire Fluent pour DE/IT/EN-CH.

### §performance — Cible v0.1

**Cible** : génération JSON d'un rapport < **500 ms** sur dataset de référence (~1000 écritures × 2-5 lignes = ~3000 lines totales). Pas de cible PDF en 9-1 (livré 9-2 avec cible `< 3s` epic-9.md §critères-arrêt).

**Mesure** : test E2E `reports_perf_smoke` (1 test marqué `#[ignore]` par défaut, activable `--ignored`) qui mesure le temps de `generate_balance_sheet` sur seed CI (à enrichir si nécessaire). **Pas de strict pass/fail v0.1** — observation only, dette R1 si > 500 ms.

**Optimisations v0.1** :

- 1 seul SELECT par rapport (pas de N+1). Utiliser JOINs.
- Index existants : `idx_journal_entries_company_date` + `idx_journal_entries_fiscal_year` + `idx_jel_entry` + `idx_jel_account` (cf. migration 20260412 ligne 33-34, 50-51). Vérifier `EXPLAIN` au dev-story.
- Pas de cache v0.1 (DB-direct).

**Limites connues** (LXX dette si dépassées) : > 10k écritures par exercice → JSON > 5 Mo, latence potentielle > 1s. Acceptable v0.1 → v0.2 pagination/streaming via Story 9-X.

## Acceptance Criteria

### Crate `kesh-report` (FR65)

1. **AC #1 — Bilan, équation comptable** — Given un exercice avec écritures validées équilibrées, when `GET /api/v1/reports/balance-sheet?fiscalYearId={fy}` est appelé, then la response JSON contient `assets`, `liabilities`, `equityResult`, `totalAssets`, `totalLiabilities`, `equationHolds: true` et `totalAssets == totalLiabilities + equityResult` (égalité Decimal exacte).

2. **AC #2 — Bilan, ordre des classes** — Given un bilan généré, when les `assets` et `liabilities` sont énumérés, then chaque liste est triée par `accountNumber` ASC (ordre lexical, ex. `1000` avant `1010`, `2000` avant `2100`).

3. **AC #3 — Compte de résultat** — Given un exercice avec écritures sur des comptes Revenue/Expense, when `GET /reports/income-statement?fiscalYearId={fy}`, then `netResult == totalRevenues - totalExpenses` (Decimal exact) et chaque section est triée par `accountNumber` ASC.

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

13. **AC #13 — Période out of fiscal year** — Given un exercice 2026-01-01 → 2026-12-31, when GET rapport avec `periodEnd=2027-01-15` (hors borne fy), then response 400 avec error code `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` et details `{ fyEnd: "2026-12-31", requestedEnd: "2027-01-15" }`.

14. **AC #14 — Période inversée** — Given `periodStart=2026-06-30&periodEnd=2026-04-01`, when GET rapport, then 400 `Validation` avec message FR « periodStart doit être ≤ periodEnd ».

15. **AC #15 — Multi-exercices isolation** — Given une company avec exercices `fy1=2025` et `fy2=2026`, when `GET /reports/balance-sheet?fiscalYearId={fy1}`, then seules les écritures avec `fiscal_year_id = fy1` sont agrégées (aucune écriture fy2 ne contamine le bilan fy1).

### Multi-tenant (KF-002 Pattern 1)

16. **AC #16 — Cross-tenant fiscal_year** — Given un user authentifié de company A et un `fiscal_year_id` qui appartient à company B, when GET rapport avec ce `fiscalYearId`, then response 404 `FISCAL_YEAR_NOT_FOUND` (jamais 403 — pattern KF-002).

17. **AC #17 — Cross-tenant aggregation** — Given 2 companies avec écritures, when user company A GET trial-balance, then aucune écriture company B n'apparaît dans `rows` (filtre `WHERE company_id = ?` systématique).

18. **AC #18 — Audit log scoped** — Given un audit `report.generated` émis par user A, when `SELECT FROM audit_log WHERE user_id = userA`, then l'audit appartient à company A via user (héritage tenant standard).

### Validation params

19. **AC #19 — fiscalYearId obligatoire** — Given GET rapport sans `fiscalYearId` query param, then response 400 `Validation` avec message FR « fiscalYearId est obligatoire ».

20. **AC #20 — fiscalYearId malformé** — Given GET avec `fiscalYearId=abc`, then 400 `Validation` (parse i64 échoué).

21. **AC #21 — Date malformée** — Given `periodStart=2026/01/15` (format non-ISO), then 400 `Validation` avec message FR « periodStart doit être au format YYYY-MM-DD ».

22. **AC #22 — Journal enum invalide** — Given `GET /reports/journals?journal=Salaires`, then 400 `Validation` avec message FR « journal doit être l'un de : Achats, Ventes, Banque, Caisse, OD ».

### RBAC

23. **AC #23 — Authenticated read** — Given un user avec rôle Consultation (lecture seule, pas d'écriture), when GET rapport, then response 200 (rôle Consultation autorisé en lecture).

24. **AC #24 — Unauthenticated 401** — Given GET rapport sans header Authorization, then response 401.

### Audit log (Q3 décision)

25. **AC #25 — Audit émis sur succès** — Given GET rapport authentifié et succès 200, when query `audit_log WHERE action = 'report.generated' AND user_id = ?`, then 1 row existe avec `details_json` contenant `{ reportType, fiscalYearId, periodStart, periodEnd, journalFilter? }`.

26. **AC #26 — Pas d'audit sur erreur 400/404** — Given GET rapport avec params invalides (400) ou cross-tenant (404), when query audit_log, then **aucune** row `report.generated` n'est créée pour cette tentative.

### Frontend

27. **AC #27 — Page `/reports` accessible** — Given user authentifié sur frontend, when navigation vers `/reports`, then la page charge sans erreur, affiche `ReportSelector` avec dropdown exercices et 4 onglets (Bilan, Résultat, Balance, Journaux).

28. **AC #28 — Génération via UI** — Given user sur `/reports` avec un exercice sélectionné, when click bouton « Générer » sur l'onglet « Bilan », then la vue `BalanceSheetView` se charge avec les données du JSON formatées (montants apostrophe `1'234.56`, dates `dd.mm.yyyy`).

29. **AC #29 — Message UX si période vide** — Given GET rapport sur une période sans écriture (response avec listes vides), when frontend rend, then affichage du message « Aucune écriture dans la période sélectionnée. Modifiez les dates ou choisissez un autre exercice. » (clé i18n `reports-error-no-entries-in-period`) — cohérent UX-DR38.

30. **AC #30 — Pas de bouton PDF/CSV v0.1** — Given page `/reports` rendue, when inspection UI, then **aucun** bouton « Télécharger PDF » ni « Export CSV » n'est présent (tracé LXX, livré 9-2).

## Tasks / Subtasks

### T1. Foundation `kesh-report` — Cargo + period + errors (AC implicite scaffolding)

- [ ] T1.1 — Étendre `crates/kesh-report/Cargo.toml` avec deps : `chrono = { workspace = true, features = ["serde"] }`, `rust_decimal = { workspace = true, features = ["serde-with-str", "maths"] }`, `serde = { workspace = true, features = ["derive"] }`, `thiserror = { workspace = true }`, `sqlx = { workspace = true, features = ["mysql", "chrono", "rust_decimal", "runtime-tokio-rustls"] }`, `tracing = { workspace = true }`, dépendances internes `kesh-core = { workspace = true }`, `kesh-db = { workspace = true }`. **Vérifier `workspace.dependencies` Cargo.toml root** — ajouter ce qui manque.
- [ ] T1.2 — Refactor `crates/kesh-report/src/lib.rs` : suppression du `//! Crate placeholder...`, déclaration des 5 modules + `pub use` exports (cf. §rust-types).
- [ ] T1.3 — Créer `crates/kesh-report/src/errors.rs` avec `ReportError` enum (cf. §error-shapes) + impl `From<DbError>` + `From<ReportError> for AppError` (mais l'impl `for AppError` reste dans `kesh-api/src/errors.rs` pour éviter dépendance cyclique).
- [ ] T1.4 — Créer `crates/kesh-report/src/period.rs` avec struct `ReportPeriod` + méthode `resolve(pool, company_id, fiscal_year_id, period_start?, period_end?)` qui :
  - Récupère `fiscal_years` via `kesh_db::repositories::fiscal_years::find_by_id_in_company(pool, fiscal_year_id, company_id)`.
  - Retourne `ReportError::FiscalYearNotFound` si None.
  - Si `period_start`/`period_end` absents → `(fy.start_date, fy.end_date)`.
  - Valide bornes incluses dans fy, ordre `start ≤ end`. Erreurs : `PeriodInvalid` ou `PeriodOutOfFiscalYear`.
- [ ] T1.5 — 4 unit tests `period::tests::*` : default_period, partial_period, period_out_of_fy_end, period_inversed.

### T2. Module `balance_sheet.rs` (AC #1, #2)

- [ ] T2.1 — Créer struct `BalanceSheet` + `AccountBalance` (cf. §rust-types) + `pub async fn generate(pool, company_id, period) -> Result<BalanceSheet, ReportError>`.
- [ ] T2.2 — SQL : 1 query qui agrège par `account_id` filtré par `account_type IN ('Asset','Liability')` ET `entry_date BETWEEN period.start AND period.end` ET `fiscal_year_id = period.fiscal_year_id` ET `company_id = ?`. Inclut comptes archivés avec écritures (cf. Q2). Calcule `SUM(debit) - SUM(credit)` (sign convention par type).
- [ ] T2.3 — Calculer `equity_result` via appel `income_statement::generate(pool, company_id, period)` ou query inline (préférer factorisation : helper privé `compute_net_result(pool, company_id, period)` partagé entre balance_sheet et income_statement).
- [ ] T2.4 — Vérifier `equation_holds = (total_assets == total_liabilities + equity_result)`. Ne pas retourner erreur si false (defense in depth : le frontend affichera un badge rouge). Logger `warn!` si false.
- [ ] T2.5 — 4 unit tests inline `#[cfg(test)] mod tests` : equation_balance_sums (montants connus, équation vérifiée), partial_period_excludes_outside_entries, archived_account_with_entries_appears, ordering_by_account_number.

### T3. Module `income_statement.rs` (AC #3)

- [ ] T3.1 — Struct `IncomeStatement` + `generate(pool, company_id, period) -> Result<...>`.
- [ ] T3.2 — SQL agrégation `account_type IN ('Revenue','Expense')` (sign convention : Revenue = `credit - debit`, Expense = `debit - credit`).
- [ ] T3.3 — `net_result = total_revenues - total_expenses`.
- [ ] T3.4 — 3 unit tests : net_result_positive, net_result_negative, net_result_zero.

### T4. Module `trial_balance.rs` (AC #4, #5, #6)

- [ ] T4.1 — Struct `TrialBalance` + `TrialBalanceRow` + `generate(...)`.
- [ ] T4.2 — SQL : SELECT par compte avec `LEFT JOIN journal_entry_lines` filtré période. Inclure comptes actifs **OU** archivés avec écritures dans la période (Q2). Calculer `SUM(debit)`, `SUM(credit)`, `balance` signé selon `account_type`.
- [ ] T4.3 — Vérifier `total_debit == total_credit`. Si non → `ReportError::TrialBalanceUnbalanced { total_debit, total_credit }` + log `error!`.
- [ ] T4.4 — 4 unit tests : balanced_true_on_valid_seed, archived_with_entries_appears_with_marker, archived_without_entries_excluded, totals_match_aggregation.

### T5. Module `journal_report.rs` (AC #7, #8, #9, #10)

- [ ] T5.1 — Struct `JournalReport`, `JournalSection`, `JournalEntryRow`, `JournalEntryLineRow` + `generate(pool, company_id, period, journal_filter: Option<Journal>) -> Result<...>`.
- [ ] T5.2 — SQL : SELECT `journal_entries JOIN journal_entry_lines JOIN accounts` filtré company + fiscal_year + période + (optionnel) journal. ORDER BY `je.journal, je.entry_date, je.entry_number, jel.line_order`.
- [ ] T5.3 — Grouper en mémoire par `journal` (5 sections fixes Achats, Ventes, Banque, Caisse, OD si `journal_filter = None`, sinon 1 section).
- [ ] T5.4 — Calculer `section_total_debit`, `section_total_credit`, `grand_total_debit`, `grand_total_credit`.
- [ ] T5.5 — 5 unit tests : all_journals_present (5 sections vides ou non), filter_achats_only_returns_one_section, ordering_chronological_then_entry_number, line_order_preserved, empty_period_returns_empty_sections.

### T6. Routes API `kesh-api/routes/reports.rs` (AC #11-#24, #25-#26)

- [ ] T6.1 — Créer `crates/kesh-api/src/routes/reports.rs` avec 4 handlers : `get_balance_sheet`, `get_income_statement`, `get_trial_balance`, `get_journal_report`.
- [ ] T6.2 — Query params extractor : déclarer struct `ReportQuery { fiscal_year_id: i64, period_start: Option<NaiveDate>, period_end: Option<NaiveDate> }` (camelCase via `#[serde(rename_all = "camelCase")]`). Pour `journals` : `JournalReportQuery extends ReportQuery + journal: Option<Journal>`.
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
- [ ] T6.7 — Étendre `Cargo.toml` `kesh-api` : ajouter `kesh-report = { workspace = true }`.

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
- [ ] T8.7 — 4 Vitest tests `reports.api.test.ts` + `BalanceSheetView.test.ts` (formatage suisse, période par défaut, équation visualisée).

### T9. i18n (AC #29 + ownership lint)

- [ ] T9.1 — Ajouter 30 clés `reports-*` dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (canonical, cf. §i18n-keys).
- [ ] T9.2 — Ajouter les mêmes 30 clés en DE/IT/EN-CH avec traduction basique + commentaire `# TODO official translation` (cohérent L51 héritée 8-5b).
- [ ] T9.3 — Lint `npm run lint-i18n-ownership` → PASS. Si échec → vérifier préfixe `reports-` et propriétaire `features/reports/`.

### T10. Tests E2E HTTP `kesh-api` (AC #11-#26)

- [ ] T10.1 — Créer `crates/kesh-api/tests/reports_e2e.rs` avec helper `spawn_app` (pattern 8-5b). Seed minimal : 1 company + 1 fiscal_year ouvert + ~5 comptes (1 Asset, 1 Liability, 1 Revenue, 1 Expense, 1 Equity) + ~3 écritures multi-journaux + 1 compte archivé avec écriture.
- [ ] T10.2 — **20 tests minimum** :
  1. `balance_sheet_returns_balanced_assets_liabilities` (AC #1)
  2. `balance_sheet_orders_accounts_by_number` (AC #2)
  3. `income_statement_computes_net_result` (AC #3)
  4. `trial_balance_total_debit_equals_total_credit` (AC #4)
  5. `trial_balance_includes_archived_account_with_entries` (AC #5)
  6. `trial_balance_excludes_archived_without_entries` (AC #6)
  7. `journals_returns_five_sections_in_order` (AC #7)
  8. `journals_filter_achats_returns_one_section` (AC #8)
  9. `journals_orders_entries_chronologically` (AC #9)
  10. `journals_preserves_line_order` (AC #10)
  11. `default_period_uses_fiscal_year_full_range` (AC #11)
  12. `partial_period_excludes_outside_entries` (AC #12)
  13. `period_end_out_of_fy_returns_400` (AC #13)
  14. `period_inversed_returns_400` (AC #14)
  15. `multi_fiscal_years_isolation` (AC #15)
  16. `cross_tenant_fiscal_year_returns_404` (AC #16)
  17. `cross_tenant_aggregation_filtered_by_company` (AC #17)
  18. `fiscal_year_id_missing_returns_400` (AC #19)
  19. `unauthenticated_returns_401` (AC #24)
  20. `report_generated_audit_emitted_on_success` (AC #25)
  + (bonus) `report_generated_audit_not_emitted_on_400_404` (AC #26).
- [ ] T10.3 — Vérifier passe `cargo test -p kesh-api --test reports_e2e -- --test-threads=1` MariaDB up. Tolérance 0 régression sur les tests existants `cargo test --workspace`.

### T11. Tests d'intégration `kesh-db` (multi-tenant, agrégats)

- [ ] T11.1 — Créer `crates/kesh-db/tests/report_aggregates.rs` (6 tests sqlx) — multi-tenant strict, période bounds, fiscal_year isolation. Pattern `#[sqlx::test]` héritée 8-5b.

### T12. Playwright E2E + a11y (AC #27, #28)

- [ ] T12.1 — Créer `frontend/tests/e2e/reports.spec.ts` : login → navigation `/reports` → sélection exercice seed → onglet « Bilan » → click « Générer » → assertion présence `Total actifs` + montant attendu formaté apostrophe.
- [ ] T12.2 — 1 axe a11y scan zero violations sur la page `/reports` rendue.
- [ ] T12.3 — Sur Ubuntu 26.04+ : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reports.spec.ts`.

### T13. Sync sprint-status + README

- [ ] T13.1 — `_bmad-output/implementation-artifacts/sprint-status.yaml` : `9-1-rapports-comptables-bilan-resultat-balance-journaux: ready-for-dev → in-progress` (au moment de `bmad-dev-story`), puis `in-progress → review` (au commit dev-story), puis `review → done` (au merge post-code-review).
- [ ] T13.2 — README.md : vérifier section « Feuille de route » — Epic 9 statut `🚧 En cours` au démarrage 9-1 (épisode déjà tagué `🚧` lors de la création epic-9.md commit `95d7bc3`). Aucun changement v0.1 features tant que 9-2 (export) n'est pas livré.

## Dev Notes

### API surface existante à réutiliser (livré Epics 1-8)

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403. Source : `kesh-db/src/repositories/journal_entries.rs:55`+, `kesh-db/src/repositories/accounts.rs:108`.
- **`fiscal_years::find_by_id_in_company(pool, id, company_id)`** : `crates/kesh-db/src/repositories/fiscal_years.rs:399`. Retourne `Result<Option<FiscalYear>, DbError>`. Utilisé pour résoudre + vérifier multi-tenant en T1.4.
- **`audit_log::insert_in_tx(tx, NewAuditLogEntry)`** : `crates/kesh-db/src/repositories/audit_log.rs` (ligne 26+). Atomique avec transaction caller. Pour Q3 audit `report.generated`, ouvrir une mini-tx dédiée (consultation read-only, pas de pattern transactionnel mutation).
- **`kesh_i18n::format_money(&Decimal)` + `format_date(&NaiveDate)`** : `crates/kesh-i18n/src/formatting.rs:16, 38`. **Apostrophe U+2019** (typographique, pas `'`). Dates `dd.mm.yyyy`. Tests existants (15+ cas).
- **`AccountType`** enum : variantes `Asset|Liability|Revenue|Expense` (cf. `crates/kesh-db/migrations/20260411000001_accounts.sql` ligne 11 + entité Rust associée — vérifier `kesh-db/src/entities/account.rs`).
- **`Journal`** enum : variantes `Achats|Ventes|Banque|Caisse|OD` (cf. `crates/kesh-db/migrations/20260412000001_journal_entries.sql` ligne 30 + `kesh-db/src/entities/journal_entry.rs`).
- **`CurrentUser` extractor** : pattern Axum dans `kesh-api/src/extractors.rs`. Expose `user.id`, `user.company_id`, `user.role`. À utiliser dans les handlers T6.3.
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
- **Pas de dépendance circulaire** : `kesh-report` dépend de `kesh-core`, `kesh-db`, `kesh-i18n`. **Pas d'inverse**. `kesh-api` dépend de `kesh-report`.
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
- `crates/kesh-api/tests/reports_e2e.rs` *(nouveau, T10 — ≥ 20 tests)*

**Crate `kesh-db`** :
- `crates/kesh-db/tests/report_aggregates.rs` *(nouveau, T11 — ≥ 6 tests sqlx)*

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
- **Intégration `kesh-db`** (T11) : `#[sqlx::test]` dans `crates/kesh-db/tests/report_aggregates.rs`. **≥ 6 tests** sqlx. Seed via `kesh-seed` ou inline INSERT.
- **E2E HTTP `kesh-api`** (T10) : `spawn_app(pool)` pattern hérité 8-5b. **≥ 20 tests** (énumérés T10.2).
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

## Change Log

| Date | Entrée | Auteur |
|------|--------|--------|
| **2026-05-14** | **`bmad-create-story 9-1` Opus 4.7 — spec initiale ready-for-dev** | Claude (Opus 4.7 — create-story) |
