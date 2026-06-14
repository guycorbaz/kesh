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
- **Impôt préalable** (TVA récupérable sur achats / Vorsteuer) — distinct de l'impôt anticipé `1170`.
- **Compte de décompte TVA** — le solde net dû à l'AFC.

**Asymétrie ground-truthée des 3 plans** (grep `charts/*.json`, 2026-06-14 — Pass 1 validate) :
| Compte | `pme.json` | `independant.json` | `association.json` |
|--------|-----------|--------------------|--------------------|
| `1170` Impôt anticipé (Asset) | ✅ l.10 | ✅ l.9 | ✅ l.9 |
| `2200` TVA due (Liability) | ✅ l.27 | ✅ l.25 | ✅ l.23 |
| `2201` Impôt anticipé dû (Liability) | ✅ l.28 | ❌ **absent** | ❌ **absent** |

→ **DC1 (FIGÉ, voir §DC ci-dessous)** : **ajouter** de nouveaux comptes (impôt préalable + décompte TVA),
**ne JAMAIS renommer** `1170`/`2201` (= impôt anticipé / Verrechnungssteuer, sémantique distincte, peut
porter des écritures réelles de withholding 35 %). Migration **différenciée par `org_type`** (2201 n'existe
que dans `pme`). Numéros choisis sans collision (cf. DC1).

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
**Conditionné par DC2 (BH2-1)** : DC2 est tranché au validate **avant le split définitif**. Si DC2 = ligne
agrégée, 18-1a→18-1b sont en série étroite (structure simple) ; si DC2 = par taux, 18-1b s'étale mais
18-1c démarre sans attendre. Le helper de génération de lignes (BH2-4) est posé en 18-1b quelle que soit
l'option.

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
- AC4 — la validation d'une facture de vente génère une écriture **équilibrée** :
  `débit créance = Σ(line_total) + Σ(line_vat_amount(line_total, vat_rate))` (TTC) ;
  `crédit produit = Σ(line_total)` (HT) ; `crédit TVA due (2200) = Σ(line_vat_amount(...))` par taux ou
  agrégé (DC2). `total_amount` reste HT (DC9). Débit = crédit garanti (3 niveaux existants).
- AC5 — la TVA comptabilisée = `Σ line_vat_amount(line_total, vat_rate)` (cohérence avec le rapport
  11-2, même arrondi half-up par ligne).
- AC6 — non-régression : factures **sans TVA (taux 0 / exempt)** → **aucune ligne compte 2200** générée
  (seulement créance + produit) ; écritures existantes intactes.

**Axe (c) — Achats**
- AC7 — (DC3=B) un helper UI d'écriture manuelle assistée (journal `Achats`) permet de saisir un achat
  avec TVA récupérable : la ligne d'impôt préalable est pré-remplie depuis un taux TVA, l'écriture reste
  équilibrée et postée sur le compte impôt préalable.

**Axe (d) — Rapport**
- AC8 — `total_vat_recoverable` et `vat_balance` reflètent la TVA récupérable réelle (source DC4).
- AC9 — le `VatReportView` retire la note « à venir » et affiche le solde net dû à l'AFC.

**Axe (e) — Réconciliation**
- AC10 — le rapport TVA réconcilie avec les soldes des comptes TVA du grand livre et expose
  `reconciliation_delta: Decimal` + `reconciliation_status: "ok" | "delta"` (DC5) ; un écart ≠ 0
  (écriture validée modifiée à la main) déclenche un bandeau d'alerte frontend, sans bloquer le rapport.
  Satisfait l'AC epic « les montants correspondent aux écritures » (lève AC#7ter de 11-2).

**Axe (f) — Tests & doc**
- AC11 — tests d'intégration (comptabilisation ventes + achats + réconciliation) + E2E, **dont un test
  explicite** : facture `vat_rate=0`/exempt → `journal_entry_lines` ne contient AUCUNE ligne sur le
  compte 2200 (F10) ; et un test de l'écart de réconciliation `delta ≠ 0`.
- AC12 — manuels user/admin + CHANGELOG + README synchronisés (politique CLAUDE.md).

---

## Décisions de conception (DC) — figées vs à trancher au validate

- **DC1 (FIGÉ — Guy 2026-06-14 ; stratégie précisée Pass 1) — AJOUTER les comptes TVA, ne JAMAIS
  renommer l'existant.**
  - **Ajouter** dans les 3 `.json` : un compte **impôt préalable** (TVA récupérable, Asset) + un compte
    **décompte TVA** (Liability). Numéros à finaliser en 18-1a **sans collision** (p.ex. `1171` pour
    l'impôt préalable — `1170` pris ; `2206` pour le décompte — `2201` pris dans `pme`). Le dev 18-1a
    vérifie la non-collision sur les 3 plans avant de figer les numéros.
  - **Ne PAS renommer** `1170` ni `2201` (= impôt anticipé / Verrechnungssteuer — sémantique distincte,
    peut porter des écritures réelles). F2/F7.
  - **Libellés canoniques du nouveau compte impôt préalable** (F6) : FR « Impôt préalable », DE
    « Vorsteuer », IT « Imposta precedente », EN « Input VAT ». (NE PAS copier les libellés de `1170`.)
  - **Migration data existant** : `INSERT … SELECT` idempotent **par company**, différencié par
    `org_type` si nécessaire, ne touchant aucun compte existant :
    `INSERT INTO accounts (company_id, number, name, account_type, parent_id, active, version, …)
     SELECT c.id, '<num>', '<libellé locale company>', '<type>', <parent>, … FROM companies c
     WHERE NOT EXISTS (SELECT 1 FROM accounts a WHERE a.company_id=c.id AND a.number='<num>')`.
    Suit DC8 (non-breaking + ligne `docs/migrations-idempotence-audit.md`).
- **DC2 (à trancher au validate)** — Ligne TVA vente : **une ligne 2200 agrégée par facture** vs **une
  ligne par taux**. (Le rapport 11-2 groupe par taux ; l'écriture peut rester agrégée tant que le total
  réconcilie.) **Couplage Epic 16 (F8)** : « par taux » structure naturellement l'écriture pour
  accueillir le compte produit par ligne (#152) sans 2e refactor de `validate_invoice` ; « agrégée »
  imposerait un refactor anticipé. À pondérer au validate.
- **DC3 (FIGÉ — Guy 2026-06-14) — Option B : écriture manuelle assistée.** PAS de nouvelle entité
  `PurchaseInvoice`. Les achats avec TVA récupérable se saisissent via `POST /journal-entries` (existe
  déjà, journal `Achats`) + un **helper UI** qui pré-remplit la ligne d'impôt préalable depuis un taux
  TVA (`line_vat_amount` + `find_for_category_at_date`). Exemple cible :
  `D charge 6xxx 1000.00 / D impôt préalable 1170 81.00 / C fournisseur 1081.00`. Epic 18 reste léger.
  **Pas d'ajout à `TABLES_TO_TRUNCATE`** (aucune nouvelle table d'entité). Issue #180 « (ou lignes TVA
  mappées) » couverte.
- **DC4 (FIGÉ par conséquence de DC3=B ; filtre précisé Pass 1 F9)** — `total_vat_recoverable` =
  **solde du compte impôt préalable lu du grand livre**. **Le filtre doit être
  `entry_date BETWEEN start_date AND end_date` (sans contrainte `fiscal_year_id`)** pour rester cohérent
  avec le filtre de `VatReport` (`i.date BETWEEN ? AND ?`, `vat_report.rs:72-74`) — `trial_balance.generate()`
  filtre par `fiscal_year_id` (`trial_balance.rs:83`), donc réutiliser sa logique d'agrégation mais PAS
  son filtre période tel quel. Pas d'agrégation d'entité achats (n'existe pas en Option B).
- **DC5 (à trancher au validate, fortement contraint par DC3=B ; écart précisé Pass 1 F5)** —
  Réconciliation : TVA due conserve sa dérivation `invoice_lines` (ventilation par taux) **+ cross-check**
  contre le solde du compte 2200 du grand livre ; TVA récupérable vient du grand livre (DC4).
  **Comportement en cas d'écart (obligatoire)** : le rapport expose `reconciliation_delta: Decimal` +
  `reconciliation_status: "ok" | "delta"` (delta ≠ 0 possible si une écriture validée a été modifiée
  manuellement via `PUT /journal-entries`). Le frontend affiche un bandeau d'alerte si `delta ≠ 0`. Le
  rapport n'est jamais bloqué. Satisfait l'AC epic « montants correspondent aux écritures » (lève AC#7ter
  de 11-2). Le choix « cross-check affiché » vs « source unique grand livre » est tranché au validate.
  **Définition du delta (BH2-2)** : `delta := total_vat_due_dérivé(invoice_lines) − solde_compte_2200(grand_livre)`
  (positif = TVA facturée > comptabilisée). **Seuil d'alerte** : `|delta| >= 0.01` (1 centime — sous ce
  seuil, arrondi numérique négligeable, `status = "ok"`). **Source affichée comme total de référence** :
  la TVA due dérivée des factures (traçable par le client) ; le solde 2200 et le delta sont affichés en
  note d'info. Bandeau frontend **INFO non bloquant** (pas de gate export), orientant vers « vérifier les
  écritures validées modifiées manuellement ».
- **DC9 (FIGÉ Pass 1 F1/F4) — Sémantique de `invoices.total_amount`** : reste **HT** (inchangé, lu
  partout comme HT). Le **TTC n'est PAS persisté** sur la facture ; il est calculé à la volée dans
  `validate_invoice` pour la ligne créance :
  `débit_créance_TTC = Σ(line_total) + Σ(line_vat_amount(line_total, vat_rate))`. **Pas de changement de
  sémantique de colonne, pas de migration de `total_amount`** (évite un breaking sur toute la codebase
  qui lit `total_amount` comme HT).
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
  **Mitigation (BH2-4)** : 18-1b **centralise la génération des lignes** d'écriture dans un helper
  `generate_invoice_journal_lines(invoice_lines, settings, …) -> Vec<NewJournalEntryLine>` (créance TTC +
  produit HT + TVA due). Epic 16 (compte produit par ligne) s'y branche ensuite **sans 2e refactor** de
  `validate_invoice`. Documenter l'ordre de merge prévu au kickoff Epic 16.
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
