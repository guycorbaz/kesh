---
status: backlog
epic: 18
story: 18-1
type: umbrella
issue: 180
created: 2026-06-14
stepsCompleted: []
---

# Story 18-1 (UMBRELLA) — Comptabilisation TVA & Achats

> **UMBRELLA / story-parente.** Cette spec couvre le périmètre complet de l'**Issue #180** (story de
> suivi Epic 11 TVA). Elle sera **convergée par `bmad-create-story validate`** puis **splittée en
> sous-stories** 18-1a..N (pattern 17-3/17-4). La parente passera alors en statut `split` et servira de
> source de contexte ; le suivi se fera sur les sous-stories.
>
> **Décision Guy 2026-06-14** : nouvel **Epic 18 « Comptabilisation TVA & Achats »** dédié (Epic 11
> reste clos après rétro). Umbrella → split.

---

## Note de cadrage — ground-truth vérifié (4 agents Explore, 2026-06-14)

Toutes les affirmations ci-dessous sont vérifiées sur le code mergé (`main` @ `e08bd21`, post-11-2).
Les `fichier:ligne` sont la référence canonique pour la spec (anti-réinvention).

### Ce qui EXISTE déjà (réutiliser, ne pas réinventer)

| Brique | Emplacement | Note |
|--------|-------------|------|
| Enum `AccountType` (Asset/Liability/Revenue/Expense) | `crates/kesh-db/src/entities/account.rs:12-27` + miroir `crates/kesh-core/src/chart_of_accounts/mod.rs:15-31` | `VARCHAR(20)` PascalCase + CHECK BINARY. **Suffisant** : TVA récupérable = Asset, TVA due = Liability. Pas de nouveau variant requis. |
| Entité `Account` + table `accounts` | `account.rs:69-82` ; migration `20260411000001_accounts.sql:6-23` | `number`/`name`/`account_type`/`parent_id`/`active`/`version`. |
| CRUD comptes + `bulk_create_from_chart` | `crates/kesh-db/src/repositories/accounts.rs` | `bulk_create_from_chart(pool, company_id, ChartEntry[], lang)` amorce les plans. |
| Plans comptables suisses (PME/indépendant/association) | `crates/kesh-core/assets/charts/{pme,independant,association}.json` ; chargés via `load_chart()` `chart_of_accounts/mod.rs:71-85` | **Contiennent déjà** : `2200 TVA due` (Liability). ⚠️ voir §discrepancy ci-dessous. |
| Helper calcul TVA `line_vat_amount(base_ht, rate_percent) -> Decimal` | `crates/kesh-core/src/accounting/vat.rs:39-43` | Arrondi **par ligne** half-up (`Money::round_to_centimes`, `money.rs:66-71`, `MidpointAwayFromZero`). FR55. |
| Sélection taux temporel `find_for_category_at_date(pool, company_id, category, date)` | `crates/kesh-db/src/repositories/vat_rates.rs:89-108` | `valid_from <= date < valid_to`, déterministe par catégorie (Story 11-1). |
| `invoice_lines.vat_rate DECIMAL(5,2)` | entité `crates/kesh-db/src/entities/invoice.rs:38-51` ; migration `20260416000001_invoices.sql:41-61` | **Le taux est déjà stocké par ligne** — jamais comptabilisé, seulement lu par le rapport. |
| Validation facture → écriture | handler `crates/kesh-api/src/routes/invoices.rs:565-579` ; métier `crates/kesh-db/src/repositories/invoices.rs:921-1136` | Crée **2 lignes HT** : débit `default_receivable_account_id`, crédit `default_revenue_account_id`. |
| `company_invoice_settings` | entité `crates/kesh-db/src/entities/company_invoice_settings.rs:20-29` ; migration `20260419000003_company_invoice_settings.sql:35-51` | A `default_receivable_account_id` + `default_revenue_account_id`. **Pas de compte TVA.** |
| Équilibrage débit=crédit (3 niveaux) | core `crates/kesh-core/src/accounting/balance.rs:144-197` ; CHECK SQL `20260412000001_journal_entries.sql:46` ; double-check post-INSERT `crates/kesh-db/src/repositories/journal_entries.rs:199-216` | Tout ajout de ligne TVA doit rester équilibré (créance TTC = produit HT + TVA due). |
| Écriture manuelle (POST/PUT) | `crates/kesh-api/src/routes/journal_entries.rs:388-495` (create) + `499-599` (update) ; front `JournalEntryForm.svelte` (journal `Achats` par défaut) | **Seul moyen actuel de saisir une charge/achat.** |
| `VatReport` + génération | struct `crates/kesh-report/src/vat_report.rs:44-56` ; `generate()` `vat_report.rs:61-120` | `total_base_ht`/`total_vat_due`/`total_vat_recoverable`(=0)/`vat_balance`. Source = `invoice_lines` des factures `validated` (PAS le grand livre). |
| Routes rapport TVA | `crates/kesh-api/src/routes/reports.rs:540-574` (GET `/reports/vat`) + `576-639` (export PDF/CSV) | + rendu `pdf.rs render_vat_report_pdf` / `csv.rs render_vat_report_csv`. |
| Lecture grand livre / soldes par compte | `crates/kesh-report/src/trial_balance.rs:50-140` (`generate()`) | `SUM(debit)/SUM(credit)` par `account_id`, signe selon `account_type`. **Base de la réconciliation (e).** |
| Frontend | `accounts/+page.svelte` (plan comptable) ; `journal-entries/+page.svelte` + `JournalEntryForm.svelte` ; `reports/VatReportView.svelte` (note « récupérable à venir ») | |

### Ce qui N'EXISTE PAS (à concevoir)

1. **Aucune entité « facture d'achat » / « dépense » / « expense »** (`crates/kesh-db/src/entities/mod.rs:1-56` : seule `Invoice` = ventes). Les achats se saisissent **uniquement en écriture manuelle**.
2. **Aucun compte TVA dans `company_invoice_settings`** (ni due, ni récupérable, ni décompte).
3. **Aucune ligne TVA générée** lors de la validation facture (HT seul).
4. **`total_vat_recoverable` n'a aucune source de données** — câblé `Decimal::ZERO` (`vat_report.rs:109-110`).
5. **Le rapport TVA n'est pas réconcilié au grand livre** — il dérive la TVA due des `invoice_lines`, pas du solde du compte 2200.

### ⚠️ Discrepancy terminologique CRITIQUE à trancher (plans comptables)

Les agents ont trouvé dans les 3 plans suisses :

| N° | Libellé seedé (FR) | Type | Réalité comptable suisse |
|----|--------------------|------|--------------------------|
| `1170` | « Impôt anticipé à récupérer » (Verrechnungssteuer) | Asset | ❌ = **impôt anticipé** (withholding 35 % sur dividendes/intérêts), **PAS** la TVA récupérable. |
| `2200` | « TVA due » | Liability | ✅ correct (TVA collectée sur ventes). |
| `2201` | « Impôt anticipé dû » | Liability | ❌ = impôt anticipé dû, **PAS** le compte de décompte TVA. |

**Conséquence** : il **manque** les comptes TVA standard du plan PME suisse :
- **Impôt préalable** (TVA récupérable sur achats) — typiquement `1170`/`1171` dans le plan PME réel (matériel/marchandises/prestations + investissements/charges).
- **Compte de décompte TVA** (`2201` ou `2206` selon plan) — le solde net dû à l'AFC.

→ **DC1** : faut-il corriger les libellés/numéros existants ET/OU ajouter les comptes manquants dans les 3 plans `.json` ? (impact migration des installations existantes — comptes seedés déjà en prod NAS).

---

## User Story

**En tant que** comptable/fiduciaire utilisant Kesh pour une PME suisse,
**je veux** que la TVA soit réellement comptabilisée dans le plan comptable (TVA due sur ventes + TVA
récupérable / impôt préalable sur achats) et que le rapport TVA par période reflète le **solde net dû à
l'AFC** réconcilié avec le grand livre,
**afin de** produire un décompte TVA correct et auditable où « les montants correspondent aux écritures
comptables » (AC epic, partiellement déféré en 11-2 via AC#7ter).

---

## Périmètre (Issue #180, axes a–e) → frontières de split pressenties

> Le split définitif est **acté au `validate`** (pattern 17-3/17-4). Frontières pressenties :

- **18-1a — Comptes TVA (story-zéro / fondation)** : DC1 résolu — comptes TVA dans les plans + champs
  `default_vat_payable_account_id` / `default_vat_recoverable_account_id` (+ décompte) dans
  `company_invoice_settings` + migration + UI config. **Pose le socle pour b/c/d/e.** Axe (a).
- **18-1b — Comptabilisation TVA aux ventes** : `validate_invoice()` génère la ligne TVA due (créance
  devient TTC). Réutilise `line_vat_amount` + `vat_rate` déjà stocké. Axe (b).
- **18-1c — Achats avec TVA (impôt préalable)** : **DC3=B** — helper UI d'écriture manuelle assistée
  (journal `Achats`) pré-remplissant la ligne d'impôt préalable depuis un taux TVA. Réutilise
  `POST /journal-entries` + `line_vat_amount` + `find_for_category_at_date`. Pas de nouvelle entité. Axe (c).
- **18-1d — TVA récupérable + solde net dans `VatReport`** : **DC4** — remplir `total_vat_recoverable`
  depuis le **solde du compte impôt préalable au grand livre** (`trial_balance`-like) + `vat_balance`. Axe (d).
- **18-1e — Réconciliation rapport ↔ grand livre** : cross-check TVA dérivée vs solde comptes TVA via
  `trial_balance`-like ; satisfaire l'AC epic « montants correspondent aux écritures ». Axe (e).
- **18-1f — Tests E2E + doc** : manuel user/admin (décompte TVA), CHANGELOG, README.

Ordre : **a (story-zéro) → b // c parallélisables → d (dépend b+c) → e (dépend d) → f**.

---

## Acceptance Criteria (groupés — détaillés au split)

**Axe (a) — Comptes TVA**
- AC1 — (DC1=corriger+migrer) les comptes TVA corrects (impôt préalable distinct de l'impôt anticipé,
  TVA due, compte de décompte) existent dans les 3 plans `.json`, libellés conformes à la terminologie
  TVA suisse (FR/DE/IT).
- AC2 — `company_invoice_settings` expose les comptes TVA par défaut (due + récupérable + décompte),
  configurables via l'UI admin, avec migration non-breaking (DC8).
- AC3 — migration des installations existantes : ajustement prudent des comptes déjà seedés (ne pas
  casser un compte portant des écritures) + doc de la procédure.

**Axe (b) — Comptabilisation ventes**
- AC4 — la validation d'une facture de vente génère une écriture **équilibrée TTC** : débit créance
  (TTC), crédit produit (HT), crédit TVA due (compte 2200) par taux ou agrégé (DC2).
- AC5 — la TVA comptabilisée = `Σ line_vat_amount(line_total, vat_rate)` (cohérence avec le rapport
  11-2, même arrondi par ligne).
- AC6 — non-régression : factures sans TVA (taux 0 / exempt) → pas de ligne TVA ; écritures existantes
  intactes.

**Axe (c) — Achats**
- AC7 — (DC3=B) un helper UI d'écriture manuelle assistée (journal `Achats`) permet de saisir un achat
  avec TVA récupérable : la ligne d'impôt préalable est pré-remplie depuis un taux TVA, l'écriture reste
  équilibrée et postée sur le compte impôt préalable.

**Axe (d) — Rapport**
- AC8 — `total_vat_recoverable` et `vat_balance` reflètent la TVA récupérable réelle (source DC4).
- AC9 — le `VatReportView` retire la note « à venir » et affiche le solde net dû à l'AFC.

**Axe (e) — Réconciliation**
- AC10 — le rapport TVA réconcilie avec les soldes des comptes TVA du grand livre (cross-check ou
  source unique), satisfaisant l'AC epic « les montants correspondent aux écritures » (lève AC#7ter de 11-2).

**Axe (f) — Tests & doc**
- AC11 — tests d'intégration (comptabilisation ventes + achats + réconciliation) + E2E.
- AC12 — manuels user/admin + CHANGELOG + README synchronisés (politique CLAUDE.md).

---

## Décisions de conception (DC) — figées vs à trancher au validate

- **DC1 (FIGÉ — Guy 2026-06-14) — Corriger les plans + migrer les données existantes.** Corriger
  libellés/numéros dans les 3 `.json` (impôt préalable distinct de l'impôt anticipé + compte de
  décompte TVA) **ET** fournir une migration qui ajuste les comptes déjà seedés des installations
  existantes (dont prod NAS). La migration suit DC8 (non-breaking si possible + audit idempotence). À
  concevoir au détail dans 18-1a (story-zéro) : stratégie d'ajustement data prudente (ne pas casser des
  comptes ayant déjà des écritures).
- **DC2 (à trancher au validate)** — Ligne TVA vente : **une ligne 2200 agrégée par facture** vs **une
  ligne par taux**. (Le rapport 11-2 groupe par taux ; l'écriture peut rester agrégée tant que le total
  réconcilie.) Décision mineure, tranchée au validate.
- **DC3 (FIGÉ — Guy 2026-06-14) — Option B : écriture manuelle assistée.** PAS de nouvelle entité
  `PurchaseInvoice`. Les achats avec TVA récupérable se saisissent via `POST /journal-entries` (existe
  déjà, journal `Achats`) + un **helper UI** qui pré-remplit la ligne d'impôt préalable depuis un taux
  TVA (`line_vat_amount` + `find_for_category_at_date`). Exemple cible :
  `D charge 6xxx 1000.00 / D impôt préalable 1170 81.00 / C fournisseur 1081.00`. Epic 18 reste léger.
  **Pas d'ajout à `TABLES_TO_TRUNCATE`** (aucune nouvelle table d'entité). Issue #180 « (ou lignes TVA
  mappées) » couverte.
- **DC4 (FIGÉ par conséquence de DC3=B)** — `total_vat_recoverable` = **solde du compte impôt préalable
  lu du grand livre** (agrégation `trial_balance`-like sur le compte récupérable, période du rapport).
  Pas d'agrégation d'entité achats (n'existe pas en Option B).
- **DC5 (à trancher au validate, fortement contraint par DC3=B)** — Réconciliation : TVA due conserve
  sa dérivation `invoice_lines` (pour la ventilation par taux du décompte) **+ cross-check** contre le
  solde du compte 2200 du grand livre (détection d'écart vente comptabilisée vs facturée) ; TVA
  récupérable vient du grand livre (DC4). Le détail (cross-check affiché vs source unique) est tranché
  au validate. Satisfait l'AC epic « montants correspondent aux écritures » (lève AC#7ter de 11-2).
- **DC6 (figé)** — Pas de nouveau variant `AccountType` (Asset/Liability suffisent).
- **DC7 (figé)** — Arrondi TVA = `line_vat_amount` half-up par ligne (cohérence 11-2, FR55).
- **DC8 (figé)** — Toute migration suit la politique CLAUDE.md (non-breaking par défaut + ligne audit
  idempotence `docs/migrations-idempotence-audit.md` + bump `kesh_version_min_required` si breaking).

---

## Hors-scope / limitations (à documenter)

- Décompte TVA officiel AFC (formulaire e-décompte / export ESTV) — hors v0.2, candidat story future.
- Méthode des taux de la dette fiscale nette (TDFN) — hors scope (méthode effective seule).
- Multi-période / corrections de décomptes antérieurs — hors scope initial.

---

## Couplages & risques

- **Epic 16 (Facturation avancée — compte produit par ligne, backlog)** : la comptabilisation par taux
  (DC2) et le compte produit par ligne (#152) touchent la même fonction `validate_invoice()`. Coordonner
  pour éviter deux refactors successifs du même code. **Risque de conflit** — à noter au split.
- **Migration prod NAS** (DC1) : des comptes TVA mal libellés sont déjà seedés chez Guy → toute
  correction de plan doit gérer l'existant (pas seulement le seed des nouvelles installs).
- **`TABLES_TO_TRUNCATE` (export/import 17-3)** : toute nouvelle table (PurchaseInvoice en Option A)
  doit y être ajoutée sinon test schéma rouge (cf. couplage 17-3/17-4).

---

## Références ground-truth (fichier:ligne) — anti-réinvention

Voir le tableau « Ce qui EXISTE déjà » ci-dessus. Points d'ancrage prioritaires pour le dev :
- Comptabilisation : `crates/kesh-db/src/repositories/invoices.rs:921-1136` (étape 7, l.1024-1051).
- Calcul TVA : `crates/kesh-core/src/accounting/vat.rs:39-43`.
- Rapport : `crates/kesh-report/src/vat_report.rs:44-120` ; routes `reports.rs:540-639`.
- Grand livre : `crates/kesh-report/src/trial_balance.rs:50-140`.
- Settings : `company_invoice_settings.rs:20-29` + migration `20260419000003`.
- Plans : `crates/kesh-core/assets/charts/{pme,independant,association}.json`.

---

## Prochaine étape

`bmad-create-story validate 18-1` Pass 1 (Sonnet 4.6) — cycle adversarial CLAUDE.md (rotation
Sonnet→Haiku→Opus→…, jusqu'à 0 finding > LOW ou 8 passes). Objectifs du validate :
1. **DC1 + DC3 + DC4 déjà FIGÉS par Guy** (2026-06-14) : DC1 corriger plans + migration data ; DC3
   Option B écriture manuelle assistée ; DC4 récupérable lu du grand livre. Ne PAS les re-litiger.
2. Trancher **DC2** (ligne TVA agrégée vs par taux) + **DC5** (détail réconciliation cross-check vs
   source unique).
3. Acter le **split** 18-1a..f définitif.
4. Vérifier la non-régression du flux `validate_invoice` et la cohérence rapport↔comptabilisation.
