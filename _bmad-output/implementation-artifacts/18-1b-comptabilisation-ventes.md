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
- **AC2** — Helper centralisé **`generate_invoice_journal_lines`** (mitigation couplage Epic 16, BH2-4) :
  une fonction pure (kesh-db ou kesh-core, cf. T-B1) prenant les lignes facture + les comptes de settings
  et retournant `Vec<NewJournalEntryLine>` (créance TTC + produit HT + lignes TVA par taux). `validate_invoice`
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
- **AC6** — **Déterminisme** : les lignes 2200 sont émises dans un **ordre stable** (trier par `vat_rate`
  croissant) pour un `line_order` reproductible (tests, audit, comparaison d'écritures `entries_equal`).
- **AC7** — **Audit** : le bloc audit `invoice.validated` (l.1088-1104) reflète les nouvelles lignes
  (il sérialise déjà `je.lines` génériquement → vérifier que les lignes 2200 apparaissent ; pas de code
  audit en dur à modifier a priori).
- **AC8** — **Non-régression (surface critique)** : les écritures existantes (factures déjà validées) sont
  intactes. Les **tests/fixtures existants qui valident des factures avec `vat_rate > 0`** doivent
  configurer `default_vat_payable_account_id` (sinon `CONFIGURATION_REQUIRED`). Le dev **DOIT** :
  - grep tous les call-sites de `validate_invoice` dans les tests (`crates/kesh-db/tests/`,
    `crates/kesh-api/tests/`) — au minimum `vat_report_e2e.rs` (valide des factures à 8.10/2.60),
    `reconciliation_e2e.rs`, `reconciliation_repository.rs`, `invoice_echeancier_e2e.rs`,
    `invoice_pdf_e2e.rs` ;
  - mettre à jour les fixtures pour configurer le compte TVA due **et** ajuster toute assertion sur le
    nombre/montant des lignes d'écriture (la créance passe HT→TTC, + N lignes 2200).
- **AC9** — **Tests** (T-B5) : (a) facture mono-taux 8.1 % → 3 lignes (créance TTC, produit HT, 1×2200) ;
  (b) facture `vat_rate=0`/exempt → **2 lignes, AUCUNE ligne 2200** (F10) ; (c) facture multi-taux
  (8.1 % + 2.6 %) → 2 lignes 2200 distinctes, montants par taux corrects ; (d) facture multi-taux
  (8.1 % + 0 %) → **1 seule** ligne 2200 (taux > 0 only) ; (e) **arrondi d'un taux > 0 donnant `0.00`**
  (base infime) → pas de ligne 2200 (F-OPUS-1) ; (f) `default_vat_payable_account_id` NULL + facture avec
  TVA → `ConfigurationRequired` ; (g) équilibre `SUM(debit)=SUM(credit)` vérifié ; (h) immunité au
  changement de taux (F-OPUS-6 : modifier `vat_rates` après validation ne change pas l'écriture).
- **AC10** — Quality gate « Test Locally First » vert (backend fmt/clippy/build/test serial + frontend
  inchangé sauf si une route/fixture le touche).

## Tasks (T-B1..T-B6)

- **T-B1** — Écrire le helper pur **`generate_invoice_journal_lines`** :
  signature pressentie (à finaliser selon emplacement) —
  `fn generate_invoice_journal_lines(lines: &[InvoiceLine], receivable_account_id: i64,
  revenue_account_id: i64, vat_payable_account_id: Option<i64>) -> Result<Vec<NewJournalEntryLine>, DbError>`.
  Logique : `total_ht = Σ line.line_total` ; `vat_by_rate` = map `vat_rate -> Σ line_vat_amount(line_total, vat_rate)` ;
  `total_vat = Σ vat_by_rate` ; si `total_vat > 0 && vat_payable_account_id.is_none()` →
  `ConfigurationRequired`. Émettre : créance `debit = total_ht + total_vat`, produit `credit = total_ht`,
  puis pour chaque `(rate, amount)` **trié par rate** avec `amount > 0` : ligne `credit = amount` sur le
  compte TVA due. (Préserver le comportement actuel si `total_ht > 0` ; les factures à total nul restent un
  cas pré-existant non couvert ici.) Documenter (`///`) le contrat + l'hypothèse F-OPUS-2 (pas d'avoir).
- **T-B2** — Brancher le helper dans `validate_invoice` étape (7) (`invoices.rs:1024-1051`) : remplacer la
  construction en dur des 2 lignes par l'appel au helper ; lire `settings.default_vat_payable_account_id` ;
  passer `lines_before`. Conserver `journal`, `entry_date`, `description`, l'ordre canonique et la tx.
- **T-B3** — Tests unitaires du helper (kesh-core/kesh-db, sans DB si pur) : AC9 (a)-(e) sur la composition
  des lignes (montants, nombre de lignes, ordre par taux, règle > 0), + équilibre.
- **T-B4** — Tests d'intégration `validate_invoice` (sqlx::test) : AC9 (a)-(h) bout-en-bout (écriture
  réellement insérée, contrainte `chk_jel_*` respectée, `ConfigurationRequired` quand compte TVA NULL).
- **T-B5** — **Non-régression** (AC8) : grep + mise à jour de **tous** les call-sites de `validate_invoice`
  en test qui utilisent `vat_rate > 0` (configurer le compte TVA due + corriger les assertions d'écriture).
- **T-B6** — Quality gate « Test Locally First » (AC10) + Change Log.

## Hors-scope (→ stories suivantes)

- Achats / impôt préalable (18-1c, DC3=B helper UI manuel). 18-1b ne touche **que** les ventes.
- Remplissage `total_vat_recoverable` / `vat_balance` (18-1d) et réconciliation (18-1e).
- **Avoirs / notes de crédit (Epic 12)** : contre-passation, PAS via ce helper (F-OPUS-2). Figé, non traité.
- **Epic 16 (compte produit par ligne)** : le helper est conçu pour l'accueillir, mais 18-1b garde **un seul
  compte produit** (`default_revenue_account_id`). Pas de produit par ligne ici.
- Factures à `total_amount = 0` (cas pré-existant : la contrainte `chk_jel_debit_credit_exclusive`
  empêche déjà une créance à 0 — comportement inchangé).

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

## Prochaine étape

`bmad-create-story validate 18-1b` (Pass 1 Sonnet 4.6) — cycle adversarial CLAUDE.md (rotation
Sonnet→Haiku→Opus→…, jusqu'à 0 finding > LOW ou 8 passes), puis `bmad-dev-story 18-1b` (Opus).
