---
status: ready-for-dev
epic: 18
story: 18-1b
type: feature
parent: 18-1
issue: 180
created: 2026-06-15
depends_on: [18-1a]
stepsCompleted: []
---

# Story 18-1b — Comptabilisation TVA aux ventes

> Extraite de l'umbrella convergée [`18-1-comptabilisation-tva-achats.md`](18-1-comptabilisation-tva-achats.md)
> (validate 5 passes, DC1-DC9 figés). **Axe (b)** — la validation d'une facture de vente génère désormais
> une écriture comptable **avec la TVA due** (créance TTC), via un helper centralisé
> `generate_invoice_journal_lines`. Dépend de **18-1a** (`done` — comptes TVA + `default_vat_payable_account_id`).

## User Story

**En tant que** comptable/fiduciaire utilisant Kesh pour une PME suisse,
**je veux** que la validation d'une facture de vente comptabilise réellement la TVA due dans le grand livre
(une ligne de TVA due par taux, créance client en TTC),
**afin de** disposer d'un grand livre où le solde du compte TVA due reflète la TVA facturée — base du décompte
AFC (18-1d/e) et de la réconciliation rapport ↔ écritures.

## Contexte ground-truth (vérifié `main` @ `c74255a`, après 18-1a)

### Ce qui EXISTE (réutiliser, ne pas réinventer)

- **Comptabilisation actuelle** : `crates/kesh-db/src/repositories/invoices.rs::validate_invoice`
  (l.921-1136), **étape (7) l.1024-1051** : crée l'écriture dans la tx via `journal_entries::create_in_tx`
  avec **exactement 2 lignes** :
  - `débit créance = total` (`receivable_account_id`, l.1038-1042),
  - `crédit produit = total` (`revenue_account_id`, l.1043-1047),
  - où `total = invoice_before.total_amount` (l.1025). **Aucune ligne TVA aujourd'hui.**
- **`total_amount` = Σ `line_total` (HT)** — recalculé backend, source de vérité = lignes
  (`invoices.rs:11` + helper `compute_total` l.282-286 sur `compute_line_total(qty, unit_price)` l.278).
  Les `line_total` sont HT (pas de TVA dans `line_total`). **DC9 : `total_amount` reste HT, inchangé.**
- **Lignes facture** : `crates/kesh-db/src/entities/invoice.rs::InvoiceLine` — champs `vat_rate: Decimal`
  (l.48, taux en **pourcent**, ex. `8.10`) + `line_total: Decimal` (l.49, HT). Disponibles dans
  `validate_invoice` via `lines_before = fetch_lines(&mut tx, invoice_id)` (l.956, helper l.329).
- **Calcul TVA** : `crates/kesh-core/src/accounting/vat.rs::line_vat_amount(base_ht, rate_percent)`
  (l.39-43) — arrondi **half-up par ligne** (`Money::round_to_centimes`), gère le négatif (avoirs).
  `line_vat_amount(1000, 8.1) = 81.00` ; `line_vat_amount(1000, 0) = 0.00`. **DC7 = cet arrondi par ligne.**
- **Settings (posés en 18-1a)** : `company_invoice_settings.default_vat_payable_account_id: Option<i64>`
  (`entities/company_invoice_settings.rs:26`) = compte **TVA due** (`2200`). Lu dans la tx via
  `company_invoice_settings::get_or_create_default_in_tx` (déjà appelé l.959-960).
- **Erreur config** : `DbError::ConfigurationRequired(String)` (`errors.rs:72`, code HTTP
  `CONFIGURATION_REQUIRED` l.113) — déjà utilisée l.962-967 pour receivable/revenue manquants.
- **Création écriture** : `journal_entries::create_in_tx` (`repositories/journal_entries.rs:90`) — pose le
  `line_order` séquentiel (idx+1, l.181-191) et **re-vérifie l'équilibre** `SUM(debit)=SUM(credit)`
  applicatif après INSERT (l.199-216, défense en profondeur).
- **Type ligne** : `NewJournalEntryLine { account_id, debit, credit }` (`entities/journal_entry.rs:176`).

### Contrainte DB décisive (F-OPUS-1)

`crates/kesh-db/migrations/20260412000001_journal_entries.sql:46-48` :
```sql
CONSTRAINT chk_jel_debit_credit_exclusive CHECK ((debit = 0 AND credit > 0) OR (debit > 0 AND credit = 0)),
CONSTRAINT chk_jel_debit_nonneg  CHECK (debit  >= 0),
CONSTRAINT chk_jel_credit_nonneg CHECK (credit >= 0),
```
⇒ **chaque ligne doit avoir debit>0 XOR credit>0** (un côté strictement positif, l'autre exactement 0).
**Une ligne TVA due à `credit = 0.00` ferait ÉCHOUER l'INSERT.** D'où la règle positive AC6 : n'émettre
la ligne 2200 d'un taux que si son montant TVA **agrégé par taux est strictement > 0**.

## Décisions figées (héritées umbrella — NE PAS re-litiger)

- **DC2** — l'écriture émet **une ligne TVA due (`2200`) PAR TAUX présent** (pas une ligne agrégée).
  Groupement par `vat_rate`, montant = Σ des `line_vat_amount` (arrondis par ligne) des lignes de ce taux.
- **DC7** — arrondi TVA = `line_vat_amount` half-up **par ligne** (réutiliser `vat.rs:39`, ne pas
  réarrondir la somme).
- **DC9** — `invoices.total_amount` reste **HT** (inchangé). Le **TTC n'est PAS persisté** ; il est calculé
  à la volée pour la ligne créance : `débit_créance_TTC = total_ht + Σ(line_vat_amount)`.
- **DC6** — pas de nouveau variant `AccountType`.
- **F-OPUS-1** — règle positive : ligne 2200 émise **seulement si montant agrégé du taux > 0** (AC6).
- **F-OPUS-6** — la TVA est calculée sur le `vat_rate` **snapshoté dans `invoice_lines`** (taux au moment de
  la validation), JAMAIS re-lookupé via `find_for_category_at_date` → immunité aux changements de taux
  postérieurs (factures déjà validées inchangées).
- **F-OPUS-2 (avoirs Epic 12, HORS SCOPE mais figé)** : `line_vat_amount` autorise un négatif, MAIS
  `chk_jel_*_nonneg` interdit debit/crédit négatif. Le helper part de l'hypothèse **`line_total >= 0`**.
  Les avoirs (Epic 12) passeront par **contre-passation** (swap débit↔crédit), PAS un montant négatif →
  **ne pas réutiliser le helper tel quel** pour les avoirs. Documenté, hors 18-1b.
- **F-OPUS-7** — la génération TVA s'insère dans la **transaction existante** de `validate_invoice`, sans
  nouveau lock, ordre canonique préservé.

## Acceptance Criteria

- **AC1** — La validation d'une facture de vente génère une écriture **équilibrée** :
  - `débit créance` (`default_receivable_account_id`) `= total_ht + Σ(line_vat_amount(line_total, vat_rate))` (TTC) ;
  - `crédit produit` (`default_revenue_account_id`) `= total_ht` (= `total_amount`, HT, DC9) ;
  - **N lignes `crédit TVA due`** (`default_vat_payable_account_id`), **une par taux** dont le montant agrégé
    `> 0` (DC2 + F-OPUS-1).
  L'équilibre est garanti **par construction** : le débit créance somme exactement les mêmes
  `line_vat_amount` (arrondis par ligne) que la somme des crédits 2200. La double-vérification
  `SUM(debit)=SUM(credit)` de `create_in_tx` (l.199-216) doit passer.
  - **Exemple chiffré (2 taux)** : ligne A `1000.00` HT @ `8.10 %`, ligne B `500.00` HT @ `2.60 %` →
    `line_vat_amount(1000, 8.1) = 81.00`, `line_vat_amount(500, 2.6) = 13.00` ; `total_ht = 1500.00`,
    `total_vat = 94.00`. Écriture (ordre canonique) :
    `D créance 1594.00` / `C produit 1500.00` / `C 2200@2.60 13.00` / `C 2200@8.10 81.00`.
    `SUM(debit) = 1594.00 = SUM(credit) = 1500.00 + 13.00 + 81.00`. ✓
- **AC2** — Helper centralisé **`generate_invoice_journal_lines`** (mitigation couplage Epic 16, BH2-4) :
  une fonction **sans I/O** (pas d'accès base, testable en `#[test]` standard sans `#[sqlx::test]`) prenant
  les lignes facture + les comptes de settings et retournant `Result<Vec<NewJournalEntryLine>, DbError>`
  (créance TTC + produit HT + lignes TVA par taux). **Emplacement = `kesh-db`** (PAS `kesh-core` : `InvoiceLine`,
  `NewJournalEntryLine` et `DbError` sont des types `kesh-db`, et `kesh-db → kesh-core` est déjà établi → un
  helper dans `kesh-core` créerait une dépendance circulaire impossible). Le placer dans
  `crates/kesh-db/src/repositories/invoices.rs` (fonction privée) à côté de son appelant. `validate_invoice`
  étape (7) **appelle ce helper** au lieu de construire les 2 lignes en dur. Epic 16 (compte produit par
  ligne) s'y branchera sans 2e refactor de `validate_invoice`.
- **AC3** — TVA comptabilisée `= Σ line_vat_amount(line_total, vat_rate)` (cohérence rapport 11-2,
  même arrondi par ligne DC7). **Invariant (F-OPUS-6)** : utiliser le **taux snapshoté sur
  `invoice_lines.vat_rate`**, JAMAIS un re-lookup `find_for_category_at_date` → immunité aux changements de
  taux postérieurs (factures déjà validées inchangées).
- **AC4** — **Règle de génération (F-OPUS-1)** : une ligne 2200 n'est émise que si le montant TVA **agrégé
  par taux est strictement > 0**. Donc :
  - facture entièrement à taux 0/exempt → **aucune ligne 2200** (créance = produit = HT, comme aujourd'hui) ;
  - un taux dont l'arrondi agrégé tombe à `0.00` → **pas de ligne 2200** pour ce taux ;
  - facture multi-taux (ex. 8.1 % + 0 %) → ligne 2200 **pour le taux > 0 uniquement**.
- **AC5** — **Config requise** : si au moins une ligne 2200 doit être émise (TVA agrégée > 0) **et**
  `default_vat_payable_account_id` est `NULL` → `DbError::ConfigurationRequired("default_vat_payable_account_id")`
  (HTTP 400 `CONFIGURATION_REQUIRED`), cohérent avec receivable/revenue (l.962-967). Si la facture n'a
  **aucune** TVA > 0, le compte TVA due **n'est pas requis** (validation passe sans lui).
  - **Note (compte archivé, vérifié ground-truth)** : `validate_invoice` n'utilise PAS la validation de
    `PUT /company/invoice-settings` (type/actif) — il lit le row via `get_or_create_default_in_tx`. Si
    `default_vat_payable_account_id` pointe vers un compte **inactif/archivé**, le helper émet la ligne 2200
    et c'est `create_in_tx` (étape 2, `journal_entries.rs:129-142`, filtre `active = TRUE`) qui rejette avec
    `DbError::InactiveOrInvalidAccounts` (vérifié ground-truth `errors.rs:1693-1694` → HTTP **400**, code
    `INACTIVE_OR_INVALID_ACCOUNTS`). Les **deux** erreurs sont HTTP **400** ; ce qui les distingue est le
    **code** : `CONFIGURATION_REQUIRED` (compte NULL, non configuré) vs `INACTIVE_OR_INVALID_ACCOUNTS`
    (compte configuré mais archivé). Comportement acceptable v0.1 (identique à receivable/revenue) —
    **documenter** cette distinction de code dans le doc-comment du helper et la section Risques.
- **AC6** — **Déterminisme** : les lignes 2200 sont émises dans un **ordre stable** — trier par `vat_rate`
  croissant (`Decimal::cmp`, ordre naturel). Les taux dont le montant agrégé == 0 sont **déjà exclus**
  (règle AC4) avant le tri → le tri ne porte que sur les taux émis (montant > 0). `line_order` reproductible
  pour tests/audit/comparaison `entries_equal`.
- **AC7** — **Audit** : le bloc audit `invoice.validated` (l.1088-1104) sérialise les lignes de l'écriture
  sous la clé **`journalEntry.lines`** (l.1096-1102, depuis `je.lines` re-fetché par `create_in_tx` après
  INSERT). Vérifier que les lignes 2200 apparaissent bien dans `journalEntry.lines` (PAS dans `before/after.*`
  qui sérialisent les lignes **facture** `lines_before`). **Aucun code audit en dur à modifier** : la
  sérialisation de `je.lines` est déjà générique.
- **AC8** — **Non-régression (surface critique)** : les écritures existantes (factures déjà validées) sont
  intactes. Tout test qui appelle réellement `validate_invoice` sur une facture avec `vat_rate > 0` doit
  désormais configurer `default_vat_payable_account_id` (sinon `CONFIGURATION_REQUIRED`). Le dev **DOIT** :
  - **Mettre à jour la fixture partagée** `crates/kesh-db/src/test_fixtures.rs::seed_accounting_company`
    **et** `seed_accounting_company_no_fy` (vérifié ground-truth : leur `INSERT INTO company_invoice_settings`
    n'a que `default_receivable/revenue_account_id` ; le plan CI minimal seede 5 comptes 1000/1100/2000/3000/4000).
    **APPROCHE RECOMMANDÉE (vérifiée Pass 3 Opus)** : **réutiliser le compte Liability existant `2000`** comme
    `default_vat_payable_account_id` dans l'`INSERT` de `company_invoice_settings` — **PAS** d'ajout d'un 6e
    compte. `create_in_tx` ne vérifie qu'`active=TRUE` + `company_id` (PAS le type — la validation de type vit
    dans la route `PUT` que la fixture bypasse), donc `2000` (Liability, actif) est un compte TVA due valide
    pour les écritures de test. **Pourquoi** : ajouter un compte `2200` ferait passer le compteur de comptes
    de **5 → 6** et casserait **toutes** les assertions de compteur absolu (vérifié ground-truth, à NE PAS
    rater) : `test_endpoints_e2e.rs:201,244,272,303,426` (`("accounts", 5)` ×5),
    `exports_global_e2e.rs:765` (`("accounts.csv", 5)`) + commentaires l.9/l.727, et les auto-tests de la
    fixture `test_fixtures.rs:505-506` (`account_count == 5` + `seeded.accounts.len() == 5`) et `:671`.
    Réutiliser `2000` **évite tout ce périmètre** (compteur inchangé). Mettre à jour uniquement l'auto-test
    qui vérifie le contenu de CIS si nécessaire (ajouter l'assertion `default_vat_payable_account_id`).
  - **Call-sites réels** de `validate_invoice` avec `vat_rate > 0` (vérifiés ground-truth) :
    `crates/kesh-api/tests/vat_report_e2e.rs` (8.10/2.60/8.00), `invoice_echeancier_e2e.rs` (8.10),
    `invoice_pdf_e2e.rs` (7.70). Avec la fixture corrigée, ces validations **réussissent** ; mettre à jour
    toute assertion sur le **montant de la créance** (HT→TTC) ou le **nombre de lignes d'écriture** (+ N
    lignes 2200) si elles en font.
  - **NE PAS** toucher `reconciliation_e2e.rs` ni `crates/kesh-db/tests/reconciliation_repository.rs` :
    vérifié ground-truth, ils **bypassent** `validate_invoice` via INSERT SQL direct
    (`reconciliation_e2e.rs:274` « bypass validate_invoice pipeline », `reconciliation_repository.rs:140`) —
    aucune mise à jour nécessaire, ne pas douter du grep.
  - **Gate grep exhaustif (avant AC10 vert)** : exécuter `grep -rln "validate_invoice" crates/*/tests crates/kesh-db/src`
    et croiser avec les fixtures à `vat_rate > 0` — pour qu'un test ajouté entre l'écriture de la spec et le
    dev ne soit pas manqué. La liste ci-dessus est vérifiée @ `c74255a`.
  - **Surface Playwright** : `frontend/tests/e2e/invoices.spec.ts:~240` valide une facture (`vatRate '7.70'`)
    via l'endpoint HTTP en s'appuyant sur `seedTestState('with-company')` → `test_endpoints.rs` →
    `seed_accounting_company` — **implicitement couvert** par le fix fixture (le seed config désormais
    `default_vat_payable_account_id`). Vérifier qu'aucune assertion Playwright ne porte sur le montant de
    créance/écriture (le total affiché reste `total_amount` HT, inchangé). Lancer `npm run test:e2e` si la
    suite tourne en local (Test Locally First).
- **AC9** — **Tests** (T-B3 unitaires helper + T-B4 intégration) : (a) facture mono-taux 8.1 % → 3 lignes
  (créance TTC, produit HT, 1×2200) ; (b) facture `vat_rate=0`/exempt → **2 lignes, AUCUNE ligne 2200**
  (AC4) ; (c) facture multi-taux (8.1 % + 2.6 %) → 2 lignes 2200 distinctes, **ordonnées par taux croissant**
  (2.60 avant 8.10), montants par taux corrects ; (d) facture multi-taux (8.1 % + 0 %) → **1 seule** ligne
  2200 (taux > 0 only) ; (e) **arrondi d'un taux > 0 donnant `0.00`** (1 ligne base infime) → pas de ligne
  2200 (F-OPUS-1) ; (e2) **DC7 — somme des montants arrondis par ligne** : 2 lignes au même taux 8.1 %,
  bases 0.07 + 0.07 → `round(0.00567)=0.01` ×2 → agrégat `0.02` → **une** ligne 2200 à `credit 0.02`
  (vérifie qu'on somme les montants par ligne, PAS qu'on arrondit une fois la base agrégée) ; (f)
  `default_vat_payable_account_id` NULL + facture avec TVA > 0 → `ConfigurationRequired` ; (f2) compte TVA
  configuré mais **archivé** + facture avec TVA → `InactiveOrInvalidAccounts` (400, code distinct) ; (g) équilibre
  `SUM(debit)=SUM(credit)` vérifié (la double-vérif de `create_in_tx` passe) ; (h) immunité au changement de
  taux (F-OPUS-6 : modifier `vat_rates` après validation ne change pas l'écriture — le snapshot
  `invoice_lines.vat_rate` est utilisé). **Taux historiques** (ex. `7.70 %` pré-2024) traités à l'identique
  (groupement/arrondi par taux snapshoté, aucune validation contre `vat_rates`).
- **AC10** — Quality gate « Test Locally First » vert (backend fmt/clippy/build/test serial + frontend
  inchangé sauf si une route/fixture le touche).

## Tasks (T-B1..T-B6)

- **T-B1** — Écrire le helper sans I/O **`generate_invoice_journal_lines`** (privé `kesh-db`) :
  signature — `fn generate_invoice_journal_lines(lines: &[InvoiceLine], receivable_account_id: i64,
  revenue_account_id: i64, vat_payable_account_id: Option<i64>) -> Result<Vec<NewJournalEntryLine>, DbError>`.
  **Prend `&[InvoiceLine]`** (type entité DB retourné par `fetch_lines`), PAS `&[NewInvoiceLine]` (ne pas
  copier par analogie avec `compute_total` qui opère sur `NewInvoiceLine`).
  Logique : `total_ht = Σ line.line_total` ; `vat_by_rate` = map `vat_rate -> Σ line_vat_amount(line_total, vat_rate)`
  (somme des montants **déjà arrondis par ligne**, DC7) ; `total_vat = Σ vat_by_rate`. Si
  `total_vat > 0 && vat_payable_account_id.is_none()` → `ConfigurationRequired("default_vat_payable_account_id")`.
  Émettre dans cet **ordre canonique** : (0) créance `debit = total_ht + total_vat` (line_order 1) ; (1) produit
  `credit = total_ht` (line_order 2) ; (2..N) pour chaque `(rate, amount)` **trié par `rate` croissant
  (`Decimal::cmp`)** avec `amount > 0` : ligne `credit = amount` sur le compte TVA due (line_order 3..).
  - **Équilibre par construction (clé)** : `total_vat = Σ vat_by_rate` (calculé AVANT le filtre `> 0`) est
    **égal** à `Σ_lines line_vat_amount(line_total, vat_rate)` — les deux formulations sont équivalentes car
    `vat_by_rate[r] >= 0` pour tout `r` (hypothèse F-OPUS-2 `line_total >= 0`), et un taux filtré (montant
    agrégé == 0) contribue 0 au débit ET 0 aux crédits. Le débit créance somme donc exactement les **mêmes**
    montants arrondis par ligne que la somme des crédits (produit + lignes 2200). NE PAS réarrondir
    `total_vat` séparément.
  - **Contrat `total_ht = 0`** : le helper retourne créance `debit=0` + produit `credit=0` (comportement
    actuel inchangé) ; `create_in_tx` rejettera via `chk_jel_debit_credit_exclusive` (cas pré-existant des
    factures à total nul, **non géré applicativement** — dette v0.1 cohérente avec l'existant). Le helper ne
    lève PAS d'erreur propre pour ce cas.
  - **Garde F-OPUS-2** : ajouter `debug_assert!(line.line_total >= Decimal::ZERO, ...)` en tête de boucle
    (les avoirs Epic 12 passeront par contre-passation, hors-scope ; en release la contrainte DB
    `chk_jel_credit_nonneg` reste le garde-fou ultime). NE PAS ajouter de `return Err` (chemin non exercé en 18-1b).
  - **Clé de groupement `Decimal` (vérifié Pass 3 Opus)** : grouper par `vat_rate` brut est sûr —
    `rust_decimal::Decimal` a un `Eq`/`Hash` **insensibles à l'échelle** (`dec!(8.1) == dec!(8.10)`, même
    clé `HashMap`/`BTreeMap`). NE PAS `.normalize()` la clé. Préférer un **`BTreeMap<Decimal, Decimal>`**
    (comme `vat_report.rs:86`) : itération ASC native → satisfait l'ordre AC6 sans étape de tri explicite,
    et structure parallèle au rapport 11-2 (facilite la réconciliation 18-1e). L'équilibre exact tient
    même si débit (échelle 4) et crédits 2200 (échelle 2) diffèrent d'échelle car la comparaison
    `SUM(debit) != SUM(credit)` de `create_in_tx` est **value-based** (échelle ignorée).
  Documenter (`///`) le contrat complet + l'hypothèse F-OPUS-2.
- **T-B2** — Brancher le helper dans `validate_invoice` étape (7) (`invoices.rs:1024-1051`) : remplacer la
  construction en dur des 2 lignes par l'appel au helper. Passer **directement** `settings.default_vat_payable_account_id`
  (déjà un `Option<i64>`, lu l.959-960) en 4e argument — la décision `ConfigurationRequired` vit **dans** le
  helper (pas de check NULL anticipé au call-site, contrairement à receivable/revenue qui restent requis
  inconditionnellement l.962-967). Passer `&lines_before` (déjà fetché l.956, sûr : aucune modif de
  `invoice_lines` dans la tx avant ce point). Conserver `journal`, `entry_date`, `description`, l'ordre
  canonique et la tx (pas de nouveau lock, F-OPUS-7).
- **T-B3** — Tests unitaires du helper (`#[test]` standard `kesh-db`, sans DB — le helper est sans I/O,
  toutes ces branches sont testables sur le `Vec` retourné) : AC9 (a) mono-taux, **(b) zéro-TVA → 2 entrées,
  aucun account_id == vat_payable**, (c) ordre multi-taux, **(d) 8.1 %+0 % → 3 entrées, 1 seule ligne 2200**,
  (e) arrondi→0, (e2) DC7 somme-par-ligne, (f) `ConfigurationRequired`, + équilibre débit=crédit sur la
  composition retournée. (Les cas (b)/(d) sont la branche de suppression zéro-taux F-OPUS-1 — couverture
  unitaire en plus de T-B4.)
- **T-B4** — Tests d'intégration `validate_invoice` (`#[sqlx::test]`) : AC9 bout-en-bout — notamment (b)
  zéro-TVA → 2 lignes, (d) multi-taux dont 0 %, (f) `ConfigurationRequired` (compte NULL), (f2) compte
  archivé → `InactiveOrInvalidAccounts`, (g) écriture réellement insérée (contraintes `chk_jel_*` OK), (h)
  immunité au changement de taux.
- **T-B5** — **Non-régression** (AC8) : (a) mettre à jour la **fixture partagée** `test_fixtures.rs`
  (`seed_accounting_company` + `seed_accounting_company_no_fy`) en **réutilisant le compte `2000`** comme
  `default_vat_payable_account_id` dans l'`INSERT` CIS (PAS de 6e compte → compteur inchangé, aucune
  assertion `("accounts", 5)` cassée) ; (b) corriger les assertions de **créance/lignes d'écriture** des
  call-sites réels (`vat_report_e2e`, `invoice_echeancier_e2e`, `invoice_pdf_e2e`) ; (c) NE PAS toucher
  `reconciliation_e2e`/`reconciliation_repository` (bypassent `validate_invoice`) ; (d) gate grep exhaustif
  `validate_invoice` avant de déclarer le quality gate vert.
- **T-B6** — Quality gate « Test Locally First » (AC10) + Change Log.

## Hors-scope (→ stories suivantes)

- Achats / impôt préalable (18-1c, DC3=B helper UI manuel). 18-1b ne touche **que** les ventes.
- Remplissage `total_vat_recoverable` / `vat_balance` (18-1d) et réconciliation (18-1e).
- **Avoirs / notes de crédit (Epic 12)** : contre-passation, PAS via ce helper (F-OPUS-2). Figé, non traité.
- **Epic 16 (compte produit par ligne)** : le helper est conçu pour l'accueillir, mais 18-1b garde **un seul
  compte produit** (`default_revenue_account_id`). Pas de produit par ligne ici.
- Factures à `total_amount = 0` (cas pré-existant : la contrainte `chk_jel_debit_credit_exclusive`
  empêche déjà une créance à 0 — comportement inchangé).

### Migration / doc / frontend (vérifié Pass 3 Opus)

- **Aucune migration** — 18-1b ne change que la génération d'écriture au runtime, pas le schéma.
- **Aucun impact frontend/i18n** — `validate_invoice_handler` (`routes/invoices.rs:575-578`) retourne
  `InvoiceResponse::from_parts(invoice, lines)` (facture + lignes facture seulement) ; `journal_entry.lines`
  n'est **pas** exposé en HTTP, aucun consommateur ne lit le nombre de lignes d'écriture côté client.
- **Doc-sync (manuels / CHANGELOG) différée à 18-1f** (split umbrella) — mi-epic, aucun changement visible
  utilisateur tant que le décompte AFC n'est pas livré.

## Risques

- **Surface de régression `CONFIGURATION_REQUIRED` (AC5/AC8)** : c'est le principal piège. Toute facture
  avec TVA > 0 validée par un test/fixture **sans** `default_vat_payable_account_id` configuré échouera
  désormais. Le dev doit balayer exhaustivement les call-sites (T-B5) avant de déclarer le gate vert —
  sinon des suites e2e vertes deviennent rouges (ex. `vat_report_e2e`).
- **Couplage Epic 16** (umbrella) : centraliser dans le helper évite un 2e refactor de `validate_invoice`.
  Garder la signature du helper extensible (le produit par ligne viendra brancher un compte produit par
  ligne au lieu d'un compte unique).
- **Équilibre par construction** : ne PAS réarrondir `Σ vat` séparément du débit créance — sommer les
  **mêmes** `line_vat_amount` arrondis par ligne des deux côtés (DC7), sinon écart d'1 centime → INSERT
  rejeté par le re-check `create_in_tx`.
- **Réconciliation bancaire (pré-existant, HORS SCOPE — vérifié ground-truth)** : `find_unpaid_invoices_for_window`
  (`reconciliation.rs:91`) matche `total_amount BETWEEN tx_amount ± tolérance`. `total_amount` reste **HT**
  (DC9, inchangé par 18-1b). Le virement bancaire est TTC. **18-1b ne change RIEN à ce comportement** (ni
  `total_amount`, ni la query de réconciliation) — l'éventuel écart HT↔TTC sur le matching est **pré-existant
  et orthogonal** (dépend de ce que la facture montre au client, hors scope 18-1b). Aucune action en 18-1b ;
  noté pour mémoire (une future story pourrait persister/exposer le TTC pour le matching).
- **Compte TVA due archivé (vérifié ground-truth `errors.rs:1693`)** : remonte `InactiveOrInvalidAccounts`
  — **HTTP 400** (comme `CONFIGURATION_REQUIRED`) ; seul le **code** diffère — cf. AC5. Documenté pour le support.

## Prochaine étape

`bmad-create-story validate 18-1b` — cycle adversarial CLAUDE.md (rotation Sonnet→Haiku→Opus→…, jusqu'à
0 finding > LOW ou 8 passes), puis `bmad-dev-story 18-1b` (Opus).

## Change Log

### `bmad-create-story validate 18-1b` — cycle adversarial (CLAUDE.md Review Iteration Rule)

| Passe | Modèle | Findings > LOW | Points clés |
|-------|--------|----------------|-------------|
| 1 | Sonnet 4.6 | 5 (1C↓+2H+2M restants après triage) | Ground-truth : **11/11 claims CONFIRMÉES** (file:line exacts). Patches : HIGH#2 fixture partagée `seed_accounting_company` à mettre à jour (compte 2200 + `default_vat_payable`) ; M2 faux positifs `reconciliation_e2e`/`_repository` (bypassent validate_invoice) retirés de la surface régression ; M1 helper en `kesh-db` pas `kesh-core` (dép circulaire) + signature `&[InvoiceLine]` ; HIGH#3 compte 2200 archivé → `InactiveOrInvalidAccounts` documenté ; M3/M4 contrat helper (équilibre par construction, total_ht=0, garde F-OPUS-2) ; M5 test DC7 somme-par-ligne ajouté ; M6 audit `journalEntry.lines` ; L2 tri `Decimal::cmp` ; L7 taux legacy. **CRITICAL réconciliation HT↔TTC down-classé** : pré-existant/orthogonal (18-1b ne touche pas total_amount, DC9) → note Risques. |
| 2 | Haiku 4.5 | 1 MEDIUM réel | **Correction factuelle ground-truth** : `InactiveOrInvalidAccounts` → HTTP **400** (`errors.rs:1693`), PAS 422 (erreur introduite Pass 1) — les 2 erreurs sont 400, seul le **code** diffère. Ground-truth Pass 1 re-confirmé 11/11 + fixture/bypass/call-sites OK. Clarifications : exemple chiffré 2-taux (équilibre 1594=1500+13+81) ajouté AC1 ; passage direct `Option` au helper (T-B2) ; T-B3/T-B4 sous-cas explicités. Reste Haiku (C1 « ambiguïté signature », H1/H2/M1-6) = formulations déjà couvertes, non bloquantes. |
| 3 | Opus 4.8 | 1 HIGH (catch-architectural) | Architectural + numeric reviewers : design **prouvé sound** (0>LOW) — cross-company défendu 2 couches (`validate_account` write + `create_in_tx` post filtre company/active), `Decimal` Eq/Hash **insensible à l'échelle** (`8.1`==`8.10` même clé, NE PAS `.normalize()`), équilibre exact prouvé sur exemples adversariaux, DC2↔`VatReport` 1:1, aucun consommateur 2-lignes. **HIGH completeness** : ajouter compte `2200` aux fixtures partagées casserait **~8 assertions de compteur absolu** (`test_endpoints_e2e.rs` ×5, `exports_global_e2e.rs:765`, auto-tests `test_fixtures.rs:505/506/671`) → **fix : réutiliser le compte `2000` existant** comme TVA due dans la fixture (compteur inchangé, `create_in_tx` ne vérifie pas le type). Patches : approche fixture réécrite + gate grep exhaustif ; note `Decimal`/`BTreeMap` (T-B1) ; note no-migration/no-frontend/doc-différée. |
| 4 | Sonnet 4.6 | 1 MEDIUM | **Tous les patches Pass 3 VÉRIFIÉS ground-truth** : compte `2000` = Liability dans les 2 fixtures (`test_fixtures.rs:127,233`), `create_in_tx` sans check de type (l.127-142), aucun test n'asserte le solde de 2000 après validate, 8 sites de compteur exacts, 3 call-sites OK, aucun manqué. MEDIUM : T-B3 omettait les cas unitaires (b) zéro-TVA + (d) 8.1 %+0 % (branche suppression F-OPUS-1 testable sans DB) → ajoutés. LOW patchés : F-OPUS-6 ajouté aux Décisions figées, surface Playwright `invoices.spec.ts:240` notée (implicitement couverte par fix fixture), réf orpheline `F10`→`AC4`. |
