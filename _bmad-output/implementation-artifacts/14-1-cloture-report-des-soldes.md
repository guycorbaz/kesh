# Story 14.1 : Clôture d'exercice & report à-nouveau — modèle temps réel

## Status

ready-for-dev

## Story

**As a** utilisateur de Kesh (indépendant, PME, association)
**I want** que mes soldes de bilan se **reportent automatiquement d'un exercice à l'autre** et que mon bilan reste juste en temps réel, sans devoir passer d'écritures de clôture manuelles,
**So that** ma comptabilité soit cohérente d'année en année (report à-nouveau) comme dans un logiciel moderne (Odoo/Flectra), tout en respectant l'immutabilité des exercices clos (CO art. 957-964).

## Contexte

Aujourd'hui, la clôture d'un exercice (`fiscal_years` `Open → Closed`) existe **uniquement comme verrou** (immutabilité + audit) ; elle **ne reporte aucun solde**. Tous les rapports (`balance_sheet`, `income_statement`, `trial_balance`) sont **silotés par exercice** (`WHERE je.fiscal_year_id = ? AND je.entry_date BETWEEN ? AND ?`) → **le bilan d'un nouvel exercice affiche zéro à l'actif/passif**. C'est le trou fonctionnel n°1.

### Décision d'architecture — modèle « temps réel virtuel » (Guy, 2026-07-21)

Modèle **Odoo/Flectra** : soldes **calculés en direct**, **sans écritures physiques** de clôture/à-nouveau.

- **Comptes de bilan** (Asset + Liability, capitaux propres compris) : **cumulatifs depuis l'origine** — solde = Σ de **toutes** les écritures `entry_date ≤ date d'arrêté`, tous exercices confondus. Report à-nouveau **calculé**.
- **Comptes de résultat** (Revenue + Expense) : **par période** (inchangé).
- **Résultat de l'exercice** : calculé à la volée (`income.net_result`).
- **Résultat reporté** : **calculé** = cumul des résultats nets des exercices **strictement antérieurs**.
- **Clôture** = verrou (existant) ; **aucune** écriture auto-générée.

### Décision CRITIQUE (validate Pass 1) — supprimer le hardcode de numéros ; rôles = 14-3

Le code exclut aujourd'hui `EQUITY_RESULT_ACCOUNT_NUMBERS = ["2979","2800"]` de `total_liabilities` (`balance_sheet.rs:22-27`) pour éviter le double-comptage du résultat calculé. **C'est un piège** : `2800` est le **capital** (vraie equity), et il est **exclu**. Un utilisateur **migrant** qui pose ses capitaux propres d'ouverture sur `2800` (ou toute écriture réelle vers un compte exclu) → montant **exclu des passifs ET absent du « résultat reporté » (dérivé du P&L)** → **les capitaux propres migrés disparaissent, l'équation casse** pour le persona cible de E14. (NB : `2970` « report à nouveau » n'est PAS dans la liste ; le défaut vient de l'exclusion aveugle par numéro, quel qu'il soit.)

**Décision (Guy, 2026-07-21) — chaque compte a un rôle explicite, le numéro ne sert JAMAIS à déduire le rôle** :

- **14-1 supprime le hardcode par numéros.** L'equity est **entièrement virtuelle** : on compte **tous** les comptes de passif cumulés (capital, réserves, report à nouveau inclus, **sans exclusion par numéro**) + on ajoute deux **lignes calculées** « Résultat de l'exercice » et « Résultat reporté ». Aucune magie de numéro.
- **Pas de double-comptage par construction** : en modèle virtuel, on **ne poste rien** au résultat/report (l'app calcule). Les postings réels à `2970`/`2800` (capital, migration) sont des soldes d'ouverture pré-Kesh, **disjoints** du cumul P&L Kesh → comptés une seule fois. Garde-fou `equation_holds` = filet si l'utilisateur passe malgré tout une écriture de clôture manuelle (déconseillé, cf. Dev Notes).
- **Durcissement = Story 14-3** (rôles configurables sur `accounts`) : rendra le compte de résultat **non-postable** et permettra une présentation des fonds propres **par rôle** (chart-agnostic), remplaçant définitivement toute logique par numéro.

### Décision performance — v1 virtuel pur + couture snapshot (Guy, 2026-07-21)

À l'échelle PME, l'agrégation « depuis l'origine » **servie par index** est de l'ordre de la milliseconde (Odoo/Flectra font pareil à plus grande échelle). Donc :

- **v1 : virtuel pur.**
- **Couture obligatoire** : isoler les soldes d'ouverture derrière **un seul point** (`opening_balances(pool, company_id, as_of)` OU une borne SQL unique `entry_date <= end_date`) pour brancher plus tard un **snapshot de soldes de clôture** (1 ligne/compte/exercice clos, **définitif via l'immutabilité**) **sans changer modèle ni UX**. Snapshot **non** implémenté ici — issue `enhancement`+`performance` à créer.
- **Interaction réouverture (14-2)** : un snapshot d'exercice **rouvert** devra être invalidé/recalculé (déclenché par `fiscal_year.reopened`). En v1 virtuel pur, rien à invalider. À documenter dans l'issue snapshot.
- **Index — DÉJÀ EN PLACE** (validate Pass 1) : `idx_journal_entries_company_date (company_id, entry_date DESC)` + `idx_jel_entry (entry_id)` existent (`migrations/20260412000001_journal_entries.sql:33,50`). → **pas de migration** ; juste **vérifier via `EXPLAIN`** que le plan est index-served.

### Hors scope (garde-fous)

- ❌ **Écritures physiques** de clôture / à-nououveau : le modèle virtuel les élimine.
- ❌ **Hardcode de numéros de comptes** : supprimé ici ; rôles = **14-3**.
- ❌ **Bilan d'ouverture / soldes de départ éditables** : **Story 14-4** (dépend de 14-3). En v1 de 14-1, un migrant peut poser une écriture OD sur ses comptes de capitaux propres réels (comptés correctement une fois le hardcode retiré) — mais l'écran dédié = 14-4.
- ❌ **Affectation du résultat assistée** (CO 958) : v1 = virtuel. Écriture d'affectation assistée = story future.
- ❌ **Réouverture / dégigeage** : **Story 14-2**.
- ❌ **Snapshot** matérialisé, **date de verrouillage** globale : évolutions futures.
- ❌ Transitoires (#232), amortissements (#222) : chantiers séparés.

## Acceptance Criteria

### A. Report à-nououveau virtuel — bilan cumulatif cross-exercice (`kesh-report::balance_sheet`)

- **Given** un exercice N clos avec des soldes actif/passif non nuls, **When** je génère le **bilan** de l'exercice N+1 à une **date d'arrêté** donnée, **Then** chaque compte de **bilan** (Asset/Liability) affiche son solde **cumulé depuis l'origine** = Σ(débit−crédit selon sens) de **toutes** les écritures `entry_date ≤ date d'arrêté`, **tous exercices confondus**.
- **And** pour le bilan, **`period_start` (borne basse) devient SANS EFFET sur TOUTES les lignes** — actif, passif **ET** le split fonds propres. Seule la **date d'arrêté = `period_end`** compte pour l'actif/passif ; le split « résultat de l'exercice / reporté » est ancré à la **borne d'EXERCICE `fy_start`** (début de l'exercice courant), **jamais** à `period_start`. Ainsi une requête à date d'arrêté en cours d'année ne déplace **pas** le split (validate Pass 2 HIGH). Documenter ce changement de sémantique de l'API.
- **And** le compte de **résultat** (`income_statement`) reste **borné à la période** (Revenue/Expense par exercice).
- **And** aucun compte à solde nul n'est listé (`HAVING balance != 0` conservé).

### B. Résultat de l'exercice + résultat reporté (fonds propres, sans hardcode)

- **Given** des exercices avec résultats, **When** je consulte les fonds propres du bilan courant, **Then** le bilan expose **deux lignes calculées** distinctes :
  - **« Résultat de l'exercice »** = résultat net **de l'exercice courant** = P&L sur `[fy_start, date d'arrêté]` (**ancré à `fy_start`, PAS à `period_start`** — validate Pass 2). Pour une date d'arrêté en fin d'exercice = `income.net_result` de l'exercice ; pour une date en cours d'année = résultat **year-to-date** ;
  - **« Résultat reporté »** = **cumul des résultats nets AVANT `fy_start`** = P&L `entry_date < fy_start`, `account_type IN (Revenue, Expense)` (nouveau champ `retained_earnings`).
- **And** `total_liabilities` compte **tous** les comptes de passif cumulés **sans aucune exclusion par numéro** (capital/réserves/report inclus) — le hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS` est **retiré** de la logique.
- **And** l'équation tient cross-exercice : `total_assets == total_liabilities + retained_earnings + equity_result`.

### C. Équilibre en début d'exercice sans écriture

- **Given** un exercice N+1 **sans aucune écriture**, **When** je génère son bilan, **Then** il **équilibre** et reflète les soldes de clôture de N, **sans** aucune écriture d'à-nououveau.

### D. Premier exercice (cas dégénéré)

- **Given** le **tout premier exercice** d'une société (aucun exercice antérieur), **When** je génère son bilan, **Then** `retained_earnings == 0` (aucun résultat antérieur — pas de `NULL`/panic) et le calcul dégénère exactement au comportement mono-exercice actuel (régression zéro).

### E. Aucune écriture auto-générée à la clôture

- **Given** le modèle temps réel, **When** je clôture un exercice, **Then** la clôture reste un **pur verrou + audit** (inchangé) — **aucune** écriture créée.

### F. Compte de résultat & balance de vérification

- Le **compte de résultat** reste **par période** (inchangé).
- **Décision figée (validate Pass 1)** : la **balance de vérification** (`trial_balance`) reste **par période** (c'est un outil de contrôle de saisie de l'exercice, pas une photo cumulative). **And** l'UI/doc indique clairement que le total par compte de `trial_balance` (mouvement de période) **n'est pas comparable** au solde cumulé du même compte au bilan — pour éviter un faux « bug » perçu.

### G. Couture perf — point unique + index vérifié

- Le calcul des soldes cumulés DOIT passer par **un seul point** réutilisable (helper/borne unique, cf. §perf), pour permettre un snapshot futur **sans toucher API/UX**.
- **And** vérifier via `EXPLAIN` que l'agrégation est **index-served** (les index existent déjà — pas de migration attendue). Si `EXPLAIN` révèle un scan, créer l'index manquant (migration `ADD INDEX` non-breaking + audit idempotence P5).
- **And** créer une issue GitHub `enhancement`+`performance` « Snapshot des soldes de clôture ».

### H. Rendus & exports (blast radius — validate Pass 1)

- **API** `GET /api/v1/reports/balance-sheet` : ajouter `retainedEarnings` (contrat **rétro-compatible**, ajout de champ).
- **CSV** (`kesh-report/src/csv.rs`) : la ligne fonds propres et le total `total_liab_eq` (`csv.rs:~122`) DOIVENT inclure `retained_earnings` (sinon **bilan exporté déséquilibré**).
- **PDF** (`kesh-report/src/pdf.rs:~513`) : idem `total_liab_eq`.
- **Constructions `BalanceSheet { … }`** (le champ `retained_earnings` non-`Option` casse la compilation) : mettre à jour les littéraux dans `csv.rs` (tests), `pdf.rs` (tests), et `benches/export.rs`.
- **Frontend** `BalanceSheetView.svelte` : afficher « Résultat reporté » + « Résultat de l'exercice » ; libellé **« Perte reportée »** si `retained_earnings < 0` (i18n FR/DE/IT/EN). L'équation affichée tient.

### I. Tests & gate

- Tests unit `kesh-report` avec **fixture numérique explicite et arithmétiquement close**, p.ex. : FY2025 actifs 15 000 / passifs 10 000 / résultat net **5 000** (reste vivant dans les comptes Revenue/Expense — aucune écriture de clôture) ; FY2026 une écriture **+200 de produit** (débit actif 200 / crédit produit 200) → **attendus** : actifs cumulés **15 200**, `retained_earnings` **5 000**, `equity_result` **200**, équation `15 200 == 10 000 + 5 000 + 200` → `equation_holds == true`. Couvrir aussi : (a) 1er exercice (`retained_earnings==0`), (b) **résultat reporté négatif** (pertes cumulées), (c) migrant posant des capitaux d'ouverture sur un compte de report/capital → `equation_holds==true`, (d) **date d'arrêté en cours d'année** : les 2 lignes fonds propres sont **identiques** à l'appel plein-exercice (ancrage `fy_start`, AC-A), (e) invariant `entry_date ∈ exercice` (Dev Note 4).
- **Tests encodant l'ANCIENNE exclusion — à réécrire/supprimer** (validate Pass 2 HIGH, sinon gate rouge) :
  - `crates/kesh-db/tests/report_aggregates.rs` Test 6 `balance_sheet_excludes_2979_from_liabilities` (~L402-451) — asserte que 2979 est **exclu** ; à inverser (2979 posté → **compté** dans les passifs, l'equity restant juste via le split calculé).
  - `crates/kesh-report/src/balance_sheet.rs` test inline `equity_result_constants_present` (~L162-167) — asserte l'existence de la const ; **supprimer** (la const `EQUITY_RESULT_ACCOUNT_NUMBERS` est **retirée entièrement**, pas laissée en code mort).
  - `crates/kesh-api/tests/reports_e2e.rs:1335` `balance_sheet_empty_period_returns_zero_totals_equation_holds` — prémisse (requête intra-exercice → totaux nuls) **contredit** le bilan cumulatif ; réécrire selon la nouvelle sémantique (`period_start` sans effet) ou superséder.
- Test E2E `reports_e2e` : 2 exercices (N avec activité, N+1 vide) → bilan N+1 non nul + équilibré.
- Gate complet (backend fmt/clippy/test + frontend check/unit) vert. Doc CHANGELOG + README (E14 en cours).

## Tasks / Subtasks

- [ ] **T1** `kesh-report` : point unique de calcul des soldes cumulés (borne `entry_date <= end_date`, `period_start` sans effet pour bilan) — AC-A/G.
- [ ] **T2** `balance_sheet` : `fetch_section` Asset/Liability cumulatif **sans exclusion par numéro** ; ajouter `retained_earnings` (cumul P&L antérieur, **date-borné** `entry_date < period.start_date`) ; réviser `equation_holds` — AC-A/B/D/H.
- [ ] **T3** `trial_balance` reste par période (+ note UI) ; `income_statement` inchangé — AC-F.
- [ ] **T4** `EXPLAIN` sur l'agrégation (index déjà présents) ; index seulement si scan — AC-G.
- [ ] **T5** exports : `csv.rs` + `pdf.rs` (formules `total_liab_eq`) + littéraux `BalanceSheet` (csv/pdf tests + `benches/export.rs`) — AC-H.
- [ ] **T6** API `reports` : `retainedEarnings` (rétro-compat) — AC-H.
- [ ] **T7** Frontend `BalanceSheetView` : résultat reporté + résultat de l'exercice + libellé perte + i18n 4 langues — AC-H.
- [ ] **T8** Tests unit (fixtures numériques + 1er exercice + perte + migrant) + réécriture `reports_e2e:1335` + E2E 2 exercices — AC-I.
- [ ] **T9** Issue snapshot + doc CHANGELOG/README + gate complet — AC-G/I.

## Dev Notes

### Pièges, par ordre de coût

1. **Ne PAS générer d'écritures.** Tout est calculé.
2. **Retrait du hardcode** `EQUITY_RESULT_ACCOUNT_NUMBERS` : ne PAS le remplacer par un autre hardcode. Equity = tous les passifs cumulés + 2 lignes calculées. Le durcissement par rôles = 14-3.
3. **`period_start` sans effet pour le bilan** — c'est le changement de sémantique qui casse `reports_e2e:1335` ; assumé (AC-A, AC-I).
4. **Ancrage à `fy_start` (borne d'exercice), pas `period_start`** (validate Pass 2 HIGH) : `equity_result` = P&L sur `[fy_start, end_date]` ; `retained_earnings` = P&L `entry_date < fy_start`, `account_type IN (Revenue, Expense)`. `fy_start` = `fiscal_years.start_date` de l'exercice courant (résolu par `find_covering_date` ou par le `fiscal_year_id` de la requête). Calculé **du même point unique** que les soldes d'ouverture — **PAS** en itérant les lignes `fiscal_years` (gaps entre exercices + exercices simultanément `Open` permis ; la borne date est suffisante et non-ambiguë).
   - **Invariant dont dépend l'équation** (validate Pass 2 MEDIUM) : l'actif/passif cumulés ignorent `fiscal_year_id` mais `equity_result` le garde (via `income_statement`) → l'équation ne tient que si tout `entry_date` d'une écriture tombe **dans** son `fiscal_year_id`. `update` l'impose (`journal_entries.rs:671` `DateOutsideFiscalYear`) mais **`create_in_tx` NON** (`:51` « pré-validé par le caller »). **Tâche** : soit ajouter un test confirmant qu'aucune écriture ne peut être créée hors bornes de son exercice, soit ajouter la garde défensive symétrique à `create_in_tx`. Défaut v1 : test de garde + `equation_holds` comme filet.
5. **Ne pas passer d'écritures de clôture manuelles** en modèle virtuel (l'app calcule le résultat) — sinon double-comptage avec `retained_earnings`. `equation_holds` (warning) reste le filet. À documenter côté utilisateur.
6. **Sens des soldes** : Asset = `débit − crédit` ; Liability = `crédit − débit` (`trial_balance.rs:70-73`, `balance_sheet.rs:135-141`).
7. **Quirk typage** `9100`/`9200` = `Expense`, `2979`/`2800` = `Liability` (`pme.json`) : neutralisé tant qu'on ne poste rien dessus. Ne pas s'appuyer dessus.

### Contrats backend (ground-truth — ancres corrigées validate Pass 1)

- `balance_sheet.rs` : `generate:61` ; `fetch_section` SQL, filtre `je.fiscal_year_id = ? AND entry_date BETWEEN` aux **lignes 135-137** ; `let equity_result = income.net_result;` **ligne 75** (pas 72) ; `let equation_holds = ...;` **ligne 77** (pas 76) ; `EQUITY_RESULT_ACCOUNT_NUMBERS:27` (à **retirer** de la logique) ; struct `BalanceSheet` champs 32-40 (ajouter `retained_earnings`).
- `income_statement.rs:30` `generate`, `:40` `net_result = total_revenues − total_expenses` = résultat de l'exercice.
- `trial_balance.rs:51` `generate`, sens des soldes **:70-73**.
- `period.rs` : `ReportPeriod { fiscal_year_id, start_date, end_date }`.
- Exports : `csv.rs:117` (ligne equity), `csv.rs:122` (`total_liab_eq`) ; `pdf.rs:497`/`pdf.rs:513`. Littéraux `BalanceSheet` : `csv.rs` tests ~604-753, `pdf.rs` tests ~1374/1738, `benches/export.rs:65`.
- `account.rs:11` : `AccountType = Asset|Liability|Revenue|Expense` (pas de type Equity). Plan PME : classe 1→Asset, 2→Liability, 3→Revenue, 4-9→Expense (`kesh-core/assets/charts/pme.json`).
- `fiscal_years.rs:568` `close` (pur flip, **inchangé**), `:422` `find_covering_date`, `:491` `find_overlapping`, `:600` « réouverture interdite » (changée par 14-2). Immutabilité écritures : `journal_entries.rs` guards blocs 110/659/916 (throws 122/661/917). Index : `20260412000001_journal_entries.sql:33` (`idx_journal_entries_company_date`), `:50` (`idx_jel_entry`).
- Test à réécrire : `reports_e2e.rs:1335`.

### Contrats frontend (ground-truth)

- `reports.api.ts:33` `getBalanceSheet`, `reports.types.ts` `BalanceSheetDto`, `BalanceSheetView.svelte` (rend `equityResult`/`totalLiabilities`/`equationHolds`). Page `routes/(app)/reports/+page.svelte`.

### Leçon de review à appliquer dès le dev

- **Un patch = un test** (`feedback_review_patch_needs_test`) — calcul financier → tests numériques précis avant done.
- **Fix structurel > incrémental** : l'équation du bilan tient par construction.

### Références

- Epic 14 (ex-« Epic 13 : Clôture d'Exercice » `epics.md`) — ACs FR60/FR61/FR62.
- Issue #232 (bouclement) — **reformulée** par le modèle temps réel ; mettre à jour.
- Stories liées : **14-2** (réouvrir/dégiger), **14-3** (rôles de comptes, durcit 14-1), **14-4** (bilan d'ouverture, dépend de 14-3).
- Cartographie (2 agents Explore, 2026-07-21) + validate Pass 1 (2 reviewers Sonnet, 2026-07-21).

## Change Log — validate

### Pass 1 (Sonnet ×2 : ancres/faisabilité + comptable/AC, 2026-07-21) — 1 CRITICAL + 3 HIGH + 4 MEDIUM + LOW → patchés

- **CRITICAL** (comptable) — hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS` exclut 2800/2970 → capitaux propres migrés disparaissent → équation cassée. **Résolu** : retrait du hardcode, equity virtuelle, décision « rôles explicites » (14-3). Vérité-terrain confirmée (`balance_sheet.rs:22-27`).
- **HIGH** (ancres) — sémantique `period_start` du bilan cumulatif non spécifiée + test `reports_e2e:1335` cassé. **Résolu** : AC-A (`period_start` sans effet) + AC-I (réécriture du test).
- **HIGH** (ancres) — rendus CSV/PDF + 4 littéraux `BalanceSheet` (dont `benches/export.rs`) hors scope → bilans exportés déséquilibrés / non-compilation. **Résolu** : AC-H + T5.
- **HIGH** (comptable) — « bilan d'ouverture hors scope » non sûr (même cause que le CRITICAL). **Résolu** : hardcode retiré + 14-4 en scope épic.
- **MEDIUM** — décision `trial_balance` (→ figée par période, AC-F) ; AC 1er exercice (→ AC-D) ; règle date-bornée du cumul (→ Dev Note 4) ; 2 dérives d'ancres (75/77, → corrigées).
- **LOW** — exemple numérique (→ AC-I) ; libellé perte reportée (→ AC-H) ; ranges d'ancres imprécis (→ corrigés).
- **Faisabilité (positif)** : modèle jugé sain par les 2 ; **index déjà présents** → pas de migration (AC-G).

### Pass 2 (Opus, contexte frais, 2026-07-21) — cœur CONFIRMÉ sain (équation = identité partie double, 0 double-comptage) ; 2 HIGH + 1 MEDIUM → patchés

- **HIGH** — blast-radius tests incomplet : `report_aggregates.rs` Test 6 (`balance_sheet_excludes_2979_from_liabilities`) + inline `balance_sheet.rs:162` (`equity_result_constants_present`) encodent l'ancienne exclusion → gate rouge. **Résolu** : AC-I liste ces tests à inverser/supprimer + const retirée entièrement.
- **HIGH** — `period_start` sans effet **contredisait** le split equity (résultat exercice vs reporté dépendait de `period_start` → mauvais chiffres silencieux en date d'arrêté mi-année, équation tenant quand même). **Résolu** : ancrage du split à `fy_start` (AC-A/B, Dev Note 4) + test (d).
- **MEDIUM** — l'équation dépend de l'invariant `entry_date ∈ exercice`, non imposé par `create_in_tx` (seul `update`). **Résolu** : Dev Note 4 (test de garde / garde défensive) + test (e).
- **LOW** — fixture rendue arithmétiquement close (+200 produit → `equity_result` 200) ; prose `2970`→`2979` corrigée ; dérives d'ancres mineures (fetch_section WHERE L136-137, trial_balance CASE L71-74).
- **Trend** : Pass 1 (1 CRIT + 3 HIGH + 4 MED) → Pass 2 (0 CRIT, 2 HIGH + 1 MED, cœur confirmé sain) → patchés. **Pass 3 requise** (HIGH trouvés), LLM différent (Haiku), contexte frais.

### Pass 3 (Haiku, contexte frais, 2026-07-21) — grep ground-truth complet → **CONVERGÉ (0 > LOW)**

- Blast-radius tests **complet** (grep : aucun autre test référençant l'exclusion/`2979`/`2800` non couvert).
- Ancres **toutes justes** (equity_result:75, equation_holds:77, csv 117/122, pdf 497/513, benches:65, report_aggregates Test 6, inline:162).
- Ancrage `fy_start` **implémentable** (via `ReportPeriod::resolve`/`find_by_id_in_company`).
- AC **cohérentes** (A vs B), fixture **arithmétiquement close** (15 200 = 10 000 + 5 000 + 200), aucun « decide in dev ».
- Restent 2 LOW cosmétiques (libellé « Perte reportée » déjà en AC-H, pédagogie period_start) — sans risque gate.

### Décision — validate

**CONVERGÉ en 3 passes** (Sonnet×2 → Opus → Haiku, rotation LLM + contexte frais + grep ground-truth). Critère d'arrêt Review Iteration Rule atteint : **0 CRITICAL/HIGH/MEDIUM**. Cœur du modèle prouvé sain (identité partie double, 0 double-comptage). Spec `ready-for-dev` confirmée. Prochaine étape : `bmad-dev-story`.

## Dev Agent Record

_(à compléter par `bmad-dev-story`)_
