# Story 14.1 : Clôture d'exercice & report à-nouveau — modèle temps réel

## Status

ready-for-dev

## Story

**As a** utilisateur de Kesh (indépendant, PME, association)
**I want** que mes soldes de bilan se **reportent automatiquement d'un exercice à l'autre** et que mon bilan reste juste en temps réel, sans devoir passer d'écritures de clôture manuelles,
**So that** ma comptabilité soit cohérente d'année en année (report à-nouveau) comme dans un logiciel moderne (Odoo/Flectra), tout en respectant l'immutabilité des exercices clos (CO art. 957-964).

## Contexte

Aujourd'hui, la clôture d'un exercice (`fiscal_years` `Open → Closed`) existe **uniquement comme verrou** : elle rend les écritures de l'exercice immuables (`DbError::FiscalYearClosed`) et journalise l'audit, mais **ne génère aucune écriture** et **ne reporte aucun solde**. Conséquence : tous les rapports (`balance_sheet`, `income_statement`, `trial_balance`) sont **silotés par exercice** (`WHERE je.fiscal_year_id = ? AND je.entry_date BETWEEN ? AND ?`) → **le bilan d'un nouvel exercice affiche zéro à l'actif/passif** tant qu'aucune écriture d'à-nouveau n'y est saisie. C'est le trou fonctionnel n°1.

### Décision d'architecture — modèle « temps réel virtuel » (tranché avec Guy, 2026-07-21)

On adopte le modèle **Odoo/Flectra** : les soldes sont **calculés en direct** depuis les écritures, **sans écritures physiques de clôture ni d'à-nououveau**.

- **Comptes de bilan** (Asset + Liability, classes 1-2, capitaux propres compris) : **cumulatifs depuis l'origine** — leur solde au bilan d'un exercice = somme de **toutes** les écritures ≤ date d'arrêté, **tous exercices confondus**. C'est le « report à-nouveau » **calculé**.
- **Comptes de résultat** (Revenue + Expense, classes 3-9) : **par période** (inchangé) — le compte de résultat reste borné à l'exercice.
- **Résultat de l'exercice** : déjà calculé à la volée (`balance_sheet.rs:72` `equity_result = income.net_result`).
- **Résultat reporté** : **calculé** = cumul des résultats nets des exercices **antérieurs** (nouveau).
- **Clôture** = verrou de période (existe déjà) ; **aucune** écriture auto-générée dans le grand livre.

### Décision performance — v1 virtuel pur + couture snapshot (tranché avec Guy, 2026-07-21)

La somme « depuis l'origine » croît avec l'historique. À l'échelle PME (quelques milliers de lignes/an, dizaines/centaines de milliers sur 10-15 ans), une agrégation **servie par index** est de l'ordre de la milliseconde — non-problème (Odoo/Flectra font pareil à plus grande échelle). Donc :

- **v1 : virtuel pur** (somme depuis l'origine).
- **Couture obligatoire** : isoler le calcul des soldes d'ouverture derrière **un seul helper** `opening_balances(pool, company_id, as_of)` (voir AC-F), pour pouvoir brancher plus tard un **snapshot de soldes de clôture** (1 ligne/compte/exercice clos, rendu **définitif par l'immutabilité de l'exercice clos**) **sans changer le modèle de données ni l'UX**. Le snapshot n'est PAS implémenté ici — c'est une évolution perf documentée (créer une issue `enhancement` + `performance`).
- **Interaction réouverture (Story 14-2)** : un exercice peut être **rouvert** (dégiger, Story 14-2). En v1 virtuel pur, rien à invalider (tout est live). Mais le futur snapshot devra **invalider/recalculer** le snapshot d'un exercice rouvert — l'invalidation sera déclenchée par l'événement `fiscal_year.reopened`. À documenter dans l'issue snapshot.
- **Prérequis** : vérifier que l'agrégation est servie par index (cf. AC-G) ; l'ajouter sinon. C'est le vrai levier, pas l'architecture.

### Hors scope (garde-fous — NE PAS faire ici)

- ❌ **Aucune écriture physique** de clôture (résultat→capitaux) ni d'à-nououveau : c'est précisément ce que le modèle temps réel élimine.
- ❌ **Écriture d'affectation du résultat assistée** (CO 958, décision d'AG) : v1 = affectation **entièrement virtuelle** (le résultat des exercices antérieurs s'agrège en « résultat reporté » calculé). L'écriture d'affectation assistée = story future si besoin de tracer une décision d'AG.
- ❌ **Bilan d'ouverture éditable** (saisie des soldes de départ pour une migration depuis un autre logiciel) : v1 = l'utilisateur saisit une **écriture manuelle OD** datée au 1er jour de son 1er exercice (fonctionne déjà avec `journal_entries`). Un écran dédié = story future (14-2 éventuelle).
- ❌ **Snapshot** matérialisé (perf) : évolution future, seulement la **couture** est posée ici.
- ❌ **Réouverture / dégigeage d'un exercice** clos par erreur : **Story 14-2 dédiée** (Admin-only + motif obligatoire + audit `fiscal_year.reopened` + garde-fou d'ordre « interdit si exercice postérieur clos »). Change la décision actuelle « réouverture interdite » (`fiscal_years.rs:600`) — modèle Odoo (verrou réversible **tracé**). Hors 14-1.
- ❌ **Date de verrouillage ajustable** façon Odoo (lock date globale) : le statut `Closed` par exercice suffit en v1.
- ❌ Transitoires / régularisations (#232), amortissements (#222) : chantiers séparés.

## Acceptance Criteria

### A. Report à-nououveau virtuel — bilan cumulatif cross-exercice (`kesh-report::balance_sheet`)

- **Given** un exercice N clos avec des soldes actif/passif non nuls, **When** je génère le **bilan** de l'exercice N+1 (à n'importe quelle date d'arrêté de N+1), **Then** chaque compte de **bilan** (Asset/Liability) affiche son solde **cumulé depuis l'origine** = Σ(débit−crédit selon sens) de **toutes** les écritures dont `entry_date ≤ date d'arrêté`, **tous exercices confondus** — et non les seules écritures de N+1.
- **And** le compte de **résultat** (`income_statement`) reste **borné à la période** de l'exercice demandé (aucun cumul cross-exercice pour Revenue/Expense).
- **And** aucun compte à solde nul n'est listé (`HAVING balance != 0` conservé).

### B. Résultat reporté (fonds propres)

- **Given** des exercices antérieurs clos ou non avec des résultats nets, **When** je consulte les capitaux propres du bilan de l'exercice courant, **Then** le bilan distingue :
  - **« Résultat de l'exercice »** = résultat net de l'exercice courant (inchangé, `income.net_result`) ;
  - **« Résultat reporté »** = **cumul des résultats nets des exercices strictement antérieurs** à l'exercice courant (nouveau champ).
- **And** l'équation du bilan tient **cross-exercice** : `total_assets == total_liabilities + résultat_reporté + résultat_de_l_exercice` (à la place de l'actuel `== total_liabilities + equity_result`).

### C. Équilibre en début d'exercice sans écriture d'à-nououveau

- **Given** un exercice N+1 **sans aucune écriture** encore saisie, **When** je génère son bilan, **Then** il **équilibre** et reflète les soldes de clôture de N (actif = passif + fonds propres), **sans** qu'aucune écriture d'à-nououveau n'ait été postée.

### D. Aucune écriture auto-générée

- **Given** le modèle temps réel, **When** je clôture un exercice (`POST /fiscal-years/{id}/close`), **Then** le comportement de clôture reste un **pur verrou + audit** (inchangé) et **aucune** écriture de clôture / à-nououveau n'est créée dans le grand livre.

### E. Compte de résultat & trial balance

- **Given** l'exercice courant, **When** je génère le **compte de résultat**, **Then** il reste **par période** (inchangé).
- **Décision** : la **balance de vérification** (`trial_balance`) — préciser en dev si elle passe en mode cumulatif « as-of » pour les comptes de bilan (cohérence avec le bilan) OU reste par période. Défaut proposé : **garder `trial_balance` par période** (c'est un outil de contrôle de saisie de l'exercice) ; documenter la décision.

### F. Couture perf — helper unique `opening_balances`

- Le calcul des **soldes d'ouverture** (cumul des comptes de bilan **avant** le début de la période) DOIT être isolé dans **un seul point** réutilisable, p.ex. `opening_balances(pool, company_id, as_of: NaiveDate) -> HashMap<i64, Decimal>` (ou intégré au SQL du bilan via une borne `entry_date <= end_date` unique), de sorte qu'un **snapshot** matérialisé puisse le remplacer plus tard **sans toucher l'API HTTP ni la vue**.
- **And** créer une issue GitHub `enhancement` + `performance` « Snapshot des soldes de clôture pour borner le report à-nououveau » référencée dans les Dev Notes.

### G. Performance — index

- **Given** l'agrégation cumulative, **When** on l'exécute, **Then** elle est **servie par index** : vérifier qu'un index couvre le motif `journal_entries(company_id, entry_date)` + jointure `journal_entry_lines(entry_id, account_id)`. Ajouter l'index manquant via migration **non-breaking** (`ADD INDEX`) si nécessaire (audit idempotence P5 + doc).

### H. Chart-agnostique — comptes de capitaux propres / résultat

- Le hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS = ["2979","2800"]` (`balance_sheet.rs:27`) est **Sterchi-PME-spécifique** (flag L70). Le calcul du **résultat reporté** ne doit PAS re-hardcoder des numéros par plan : dériver le résultat reporté du **cumul des Revenue/Expense des exercices antérieurs** (indépendant du plan), pas d'un compte 2970 nommé. Documenter comment l'ancien exclude-list interagit avec le nouveau champ (éviter double-comptage).

### I. Routes HTTP & Frontend

- **API** : `GET /api/v1/reports/balance-sheet` expose les nouveaux champs (`retainedEarnings` / `résultat reporté` en plus de `equityResult`). Contrat rétro-compatible (ajout de champs).
- **Frontend** : `BalanceSheetView.svelte` affiche « Résultat reporté » + « Résultat de l'exercice » dans les fonds propres ; l'équation affichée tient. i18n FR/DE/IT/EN des libellés.

### J. Tests & gate

- Tests unitaires `kesh-report` : (a) bilan N+1 cumule les soldes de N (report à-nououveau), (b) résultat reporté = cumul des résultats antérieurs, (c) équilibre cross-exercice en début d'exercice sans écriture, (d) compte de résultat reste par période.
- Test E2E `reports_e2e` : 2 exercices (N clos avec activité, N+1 vide) → bilan N+1 non nul + équilibré.
- Gate complet (backend fmt/clippy/test + frontend check/unit) vert. Doc CHANGELOG + README roadmap (E14 en cours).

## Tasks / Subtasks

- [ ] **T1** `kesh-report` : introduire le calcul cumulatif des comptes de bilan (helper `opening_balances` OU borne SQL unique `entry_date <= end_date` cross-exercice) — AC-A, AC-F.
- [ ] **T2** `kesh-report::balance_sheet` : `fetch_section` (Asset/Liability) en mode cumulatif ; ajouter `retained_earnings` (cumul résultats antérieurs) ; réviser `equation_holds` (`== total_liabilities + retained_earnings + equity_result`) — AC-A/B/C/H.
- [ ] **T3** vérifier `income_statement` & décider `trial_balance` (défaut : par période) — AC-E ; documenter.
- [ ] **T4** index de perf (vérifier/ajouter migration `ADD INDEX` non-breaking + audit idempotence P5) — AC-G.
- [ ] **T5** API `reports` : exposer `retainedEarnings` (rétro-compat) — AC-I.
- [ ] **T6** Frontend `BalanceSheetView` : afficher résultat reporté + résultat de l'exercice + i18n 4 langues — AC-I.
- [ ] **T7** Tests unit `kesh-report` + E2E `reports_e2e` 2 exercices — AC-J.
- [ ] **T8** Issue GitHub `enhancement`+`performance` snapshot + doc CHANGELOG/README + gate complet — AC-F/J.

## Dev Notes

### Pièges, par ordre de coût

1. **Ne PAS générer d'écritures** (clôture/à-nououveau). Tout est calculé. Le grand livre ne contient que les écritures métier.
2. **Sens des soldes** : Asset = `débit − crédit` ; Liability = `crédit − débit` (cf. `trial_balance.rs:71-75`, `balance_sheet.rs:106-141`). Le cumul cross-exercice conserve ce sens.
3. **Double-comptage du résultat** : aujourd'hui `equity_result` est calculé du compte de résultat **et** `EQUITY_RESULT_ACCOUNT_NUMBERS` exclut 2979/2800 des passifs pour éviter le double-count. En ajoutant `retained_earnings` (cumul Revenue/Expense antérieurs), s'assurer que les comptes de résultat cumulés **ne** repassent **pas** dans les passifs (les Revenue/Expense ne sont pas des Liability, donc a priori pas de collision — mais valider avec un plan réel + le cas des comptes 9xxx typés `Expense`, cf. quirk ci-dessous).
4. **Quirk de typage** : `9100`/`9200` (Compte de résultat / Bilan de clôture) sont typés `Expense` dans le plan PME ; `2979` (Résultat de l'exercice) est `Liability`. En modèle temps réel **on ne poste rien** sur ces comptes → le quirk est neutralisé tant qu'aucune écriture n'y va. Ne PAS s'appuyer dessus.
5. **Zéro-amount lines interdites** en base (`chk_jel_debit_credit_exclusive`) — sans objet ici (aucune écriture générée).
6. **Chart-agnostique** : dériver le résultat reporté du **cumul des classes de résultat** des exercices antérieurs, pas d'un numéro de compte hardcodé (AC-H).

### Contrats backend (ground-truth, à ne pas re-deviner)

- `kesh-report/src/balance_sheet.rs` : `generate:61`, `fetch_section` SQL `:106-141` (filtre `je.fiscal_year_id = ? AND entry_date BETWEEN`), `equity_result = income.net_result:72`, `equation_holds:76`, `EQUITY_RESULT_ACCOUNT_NUMBERS:27` (Sterchi-PME, flag L70). Struct `BalanceSheet` (period, assets, liabilities, total_assets, total_liabilities, equity_result, equation_holds).
- `kesh-report/src/income_statement.rs` : `generate:30`, `net_result = total_revenues − total_expenses:40` = **le** résultat de l'exercice.
- `kesh-report/src/trial_balance.rs` : `generate:51`, sens des soldes `:71-75` — **meilleur candidat de réutilisation** pour un solde cumulé par compte.
- `kesh-report/src/period.rs` : `ReportPeriod { fiscal_year_id, start_date, end_date }`.
- `kesh-db` comptes : `AccountType` = `Asset | Liability | Revenue | Expense` (`entities/account.rs:11`) — **pas de type Equity** (les capitaux propres sont des `Liability`). Bilan = Asset+Liability ; Résultat = Revenue+Expense. Plan PME : classe 1→Asset, 2→Liability, 3→Revenue, 4-9→Expense (`kesh-core/assets/charts/pme.json`).
- `fiscal_years` : `close:568` (pur flip Open→Closed + audit, **inchangé**), `find_covering_date:422`, `find_overlapping:491`. Plusieurs exercices coexistent, pas de contrainte « exercice N-1 doit être clos ». Immutabilité des écritures d'un exercice clos : `journal_entries.rs:110/659/916` (`FiscalYearClosed`).
- Écritures (référence pour comprendre les soldes, PAS pour en générer) : `journal_entry_lines(account_id, debit, credit, project_id)` ; `journal_entries(company_id, fiscal_year_id, entry_date, journal)`.

### Contrats frontend (ground-truth)

- `frontend/src/lib/features/reports/reports.api.ts` : `getBalanceSheet → /api/v1/reports/balance-sheet:33`. Types `reports.types.ts` (`BalanceSheetDto`). Vue `BalanceSheetView.svelte`. Page `routes/(app)/reports/+page.svelte`.

### Leçon de review à appliquer dès le dev

- **Un patch = un test** (memory `feedback_review_patch_needs_test`). Le calcul de solde cumulé est financier → tests unitaires numériques précis (report à-nououveau, résultat reporté, équilibre) **avant** de considérer une tâche done.
- **Fix structurel > incrémental** sur les invariants d'équilibre (memory) : l'équation du bilan doit tenir par construction, pas par rustines.

### Références

- Epic 14 (ex-« Epic 13 : Clôture d'Exercice » dans `epics.md`) — ACs FR60/FR61/FR62.
- Issue #232 (bouclement) — **reformulée** par le modèle temps réel : plus d'« assistant d'écritures de clôture », mais consolidation temps réel + report à-nououveau virtuel. Mettre à jour #232 en conséquence.
- Cartographie complète (2 agents Explore, 2026-07-21) : moteur comptable + cycle fiscal_year.

## Questions sauvegardées (pour validate/Guy)

1. **`trial_balance`** cumulatif « as-of » ou par période ? (défaut proposé : par période — cf. AC-E).
2. **Résultat reporté** : le montrer comme **une ligne agrégée** dans les fonds propres, ou ventilé ? (défaut : une ligne « Résultat reporté »).
3. **Affectation du résultat assistée** (CO 958) : confirmée **hors v1** (virtuel) — OK ?

## Change Log — validate

_(à compléter par les passes `bmad-create-story validate`)_

## Dev Agent Record

_(à compléter par `bmad-dev-story`)_
