---
status: ready-for-dev
epic: 18
story: 18-1e
type: feature
parent: 18-1
issue: 180
created: 2026-06-25
depends_on: [18-1a, 18-1b, 18-1d]
baseline_commit: 7264994
stepsCompleted: []
---

# Story 18-1e — Réconciliation rapport TVA ↔ grand livre

> Extraite de l'umbrella convergée [`18-1-comptabilisation-tva-achats.md`](18-1-comptabilisation-tva-achats.md).
> **Axe (e)** — **DC5** : cross-check de la **TVA due dérivée des `invoice_lines`** (total de référence
> traçable) contre le **solde du compte TVA due (`2200`) au grand livre, isolé au périmètre ventes**. Le
> rapport expose `reconciliation_delta` + `reconciliation_status` ; le front affiche un **bandeau d'alerte
> INFO non bloquant** quand l'écart ≠ 0. Satisfait l'AC epic « les montants correspondent aux écritures »
> (lève AC#7ter de 11-2).
> Dépend de 18-1a (compte `2200` + `default_vat_payable_account_id`), 18-1b (TVA due réellement comptabilisée
> via `generate_invoice_journal_lines` + lien `invoices.journal_entry_id`), 18-1d (struct `VatReport` enrichie
> + branchement `generate`).

## User Story

**En tant que** comptable/fiduciaire d'une PME suisse,
**je veux** que le rapport TVA **réconcilie** la TVA due calculée depuis les factures avec le solde réel du
compte TVA due au grand livre, et me **signale tout écart** (typiquement une écriture validée modifiée à la
main),
**afin de** garantir que « les montants du décompte correspondent aux écritures comptables » avant de
transmettre le décompte à l'AFC.

## Contexte ground-truth (vérifié `main` @ `7264994`, après 18-1a/b/c/d)

### Ce qui EXISTE (réutiliser, ne pas réinventer)

- **`VatReport` + `generate()`** : `crates/kesh-report/src/vat_report.rs`.
  - Struct (l.56-67, serde `camelCase`) : `period, rows, total_base_ht, total_vat_due,
    total_vat_recoverable, vat_balance`. **18-1e ajoute 2 champs** `reconciliation_delta` +
    `reconciliation_status` — sérialisation auto via le `#[serde(rename_all = "camelCase")]` existant.
  - `generate(pool, company_id, period)` (l.72-148) :
    - **TVA due dérivée** des `invoice_lines` (l.79-119) : `SELECT il.vat_rate, il.line_total … WHERE
      i.company_id = ? AND i.status = 'validated' AND i.date BETWEEN ? AND ?` → arrondi par ligne
      (`line_vat_amount`, FR55) accumulé par taux dans un `BTreeMap` → `total_vat_due` (l.119). **C'est le
      total de référence traçable (DC5).**
    - **TVA récupérable** (l.124-137) : lit `default_vat_recoverable_account_id` puis helper
      `recoverable_balance` (l.156-178). `vat_balance = total_vat_due − total_vat_recoverable` (l.138).
    - Retour `VatReport { … }` (l.140-147). **C'est ici que 18-1e calcule et ajoute le delta** (juste avant
      la construction du retour).
  - **Helper `recoverable_balance` (l.156-178)** : modèle exact de la requête de solde de compte scopée
    company. **18-1e ajoute un helper jumeau `due_account_balance_sales_scope`** (voir T-E1) sur le compte
    `default_vat_payable_account_id`, restreint au périmètre ventes.
- **Comptabilisation TVA due aux ventes (18-1b)** : `crates/kesh-db/src/repositories/invoices.rs`.
  - `validate_invoice` (l.1019-1232) crée l'écriture de vente via `generate_invoice_journal_lines`
    (l.933-995) : créance TTC (débit) / produit HT (crédit) / **N lignes crédit TVA due par taux** sur le
    compte `settings.default_vat_payable_account_id` (= `2200`), montants > 0 uniquement (DC2/F-OPUS-1).
  - **Lien facture → écriture** : `UPDATE invoices SET … journal_entry_id = ?` (l.1156-1169). Colonne
    `invoices.journal_entry_id BIGINT NULL` FK → `journal_entries.id`
    (`20260417000001_invoice_validation.sql:53-57`). **Lien unidirectionnel** : il n'y a **pas** de colonne
    `invoice_id` sur `journal_entry_lines`. Pour isoler les lignes 2200 « ventes » : passer par
    `invoices.journal_entry_id` (factures `validated`).
  - L'écriture de vente porte `entry_date = invoice.date` (`validate_invoice` l.~1137) et `journal =
    settings.default_sales_journal` (**configurable**, défaut `'Ventes'`, migration `20260417000001`).
    ⚠️ **Donc filtrer sur le libellé `journal = 'Ventes'` est FRAGILE** (le journal des ventes est un
    réglage modifiable) → **DC5-iso ci-dessous impose l'isolation par le lien facture validée**, pas par le
    libellé.
- **Schéma grand livre** (`20260412000001_journal_entries.sql`) :
  - `journal_entries(id, company_id, fiscal_year_id, entry_number, entry_date DATE, journal VARCHAR(10)
    CHECK Achats|Ventes|Banque|Caisse|OD, description, version, …)`.
  - `journal_entry_lines(id, entry_id FK→journal_entries ON DELETE CASCADE, account_id FK→accounts,
    line_order, debit DECIMAL(19,4) ≥0, credit DECIMAL(19,4) ≥0)`. CHECK exclusivité débit/crédit.
- **Compte TVA due configuré** : `company_invoice_settings.default_vat_payable_account_id: Option<i64>`
  (= `2200`, posé 18-1a). Repo `crates/kesh-db/src/repositories/company_invoice_settings.rs`. En lecture
  seule ici : un `SELECT default_vat_payable_account_id FROM company_invoice_settings WHERE company_id = ?`
  suffit (row lazy-créée à l'onboarding ; déjà lu de la même façon pour le récupérable en 18-1d, l.124-137).
- **Front** : `frontend/src/lib/features/reports/VatReportView.svelte`.
  - Props `dto: VatReportDto` (l.7-10), dérivée `empty = isReportEmpty('vat', dto)` (l.11), structure
    `{#if empty}{:else}<table>…<tfoot>…` (l.27-69) : `tfoot` lignes CA HT / TVA récupérable / Solde
    (l.52-66). `</section>` l.69-71. **Point d'insertion du bandeau** : juste après la fermeture du bloc
    `{#if empty}{:else}` (l.68) et avant `</section>`, OU en tête de section — rendu **inconditionnel** (le
    bandeau doit s'afficher même si `empty`, cf. AC9).
  - `VatReportDto` : `reports.types.ts:100-107` (`totalBaseHt`/`totalVatDue`/`totalVatRecoverable`/
    `vatBalance` strings). **18-1e ajoute `reconciliationDelta: string` + `reconciliationStatus: 'ok' |
    'delta'`** après l.106.
  - `isReportEmpty('vat')` : `reports.api.ts:107-113` (`rows.length === 0 && Number(totalVatRecoverable)
    === 0`). **NE PAS toucher** (le delta n'entre pas dans la définition de « vide » — voir DC5-empty).
  - **Pattern d'alerte inline** (PAS de composant wrapper réutilisable pour les alertes contextuelles) :
    `<p class="rounded bg-{couleur}-50 p-3 text-sm text-{couleur}-900" role="alert">…</p>` — modèle
    `reports-equation-warning` dans `BalanceSheetView.svelte:128` (`bg-red-50`). Pour un bandeau **INFO non
    bloquant** (DC5), préférer une teinte d'avertissement (`amber`/`yellow`), distincte du rouge d'erreur.
- **i18n** : 4 locales `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`. Clés
  `reports-vat-*` (fr-CH l.892-897 : `reports-vat-recoverable`, `reports-vat-balance`, …). Pattern
  `reports-{section}-{nom}` kebab-case. **18-1e ajoute des clés** `reports-vat-reconciliation-*` dans les
  **4** locales (parité obligatoire, `lint-i18n-ownership`).
- **Renderers** : `render_vat_report_csv` (`csv.rs:316-368`, bloc récap l.345-364 termine par « Solde ») et
  `render_vat_report_pdf` (`pdf.rs:934-989`, bloc totaux l.979-986 via `draw_totals_footer`, labels
  `VatPdfLabels` l.879-912). **18-1e ajoute une ligne « écart de réconciliation » après « Solde »** dans les
  deux. `VatPdfLabels::fr_ch_defaults()` (utilisé `reports.rs:611`) doit recevoir le(s) libellé(s).
- **Routes** : `GET /api/v1/reports/vat` (`routes/reports.rs:544-574`) et export PDF/CSV
  (`reports.rs:577-639`) appellent `generate_vat_report` puis sérialisent / rendent. **Aucun changement de
  route** (champs additifs sérialisés auto ; les renderers consomment la struct enrichie).

### Ce qui N'EXISTE PAS (à concevoir — minimal)

- Le **solde du compte `2200` isolé au périmètre ventes** (lignes liées à une facture validée de la période)
  n'est calculé nulle part.
- Le **delta de réconciliation** et son **statut** (champs struct + calcul dans `generate`).
- Le **bandeau d'alerte front** + la **ligne réconciliation** dans CSV/PDF.

## Décisions figées (héritées umbrella DC5 — NE PAS re-litiger)

- **DC5 (FIGÉ Pass 3 Opus — CROSS-CHECK AFFICHÉ, 2 sources)** — La TVA due garde sa dérivation
  `invoice_lines` comme **total de référence traçable par le client** ; on **cross-check** contre le solde
  du compte `2200` du grand livre. La TVA récupérable (18-1d) reste source unique grand livre (DC4-bis, pas
  de cross-check). **Le delta porte uniquement sur la TVA due.** Une source unique masquerait silencieusement
  les divergences (écriture validée modifiée à la main) que seul le cross-check révèle.
- **DC5-iso (FIGÉ — F-OPUS-4 ; raffiné par ground-truth `default_sales_journal` configurable)** — le solde
  `2200` de référence pour le delta **isole le périmètre ventes par le LIEN FACTURE VALIDÉE**, pas par le
  libellé `journal`. Concrètement : ne sommer que les lignes `journal_entry_lines` sur le compte
  `default_vat_payable_account_id` dont l'`entry_id` appartient à une facture **`validated`** de la période
  (`invoices.journal_entry_id`, `i.date BETWEEN start AND end`). **Justification** : (1) `journal = 'Ventes'`
  est un réglage modifiable (`default_sales_journal`) → un filtre sur le libellé serait faux si l'utilisateur
  change le journal des ventes ; (2) le lien `journal_entry_id` désigne **exactement** l'écriture générée par
  `validate_invoice`, donc le périmètre est strict et symétrique à la dérivation `invoice_lines` (même
  ensemble de factures). **Conséquence (voulue)** : une écriture **manuelle légitime** sur `2200`
  (auto-liquidation, régularisation AFC, OD) qui n'est **pas** liée à une facture validée est **exclue** du
  solde de référence → **pas de faux positif** (F-OPUS-4 satisfait). Le résiduel manuel hors-ventes sur `2200`
  est **hors scope** du delta (documenté comme « écart informatif à investiguer manuellement », non calculé
  v0.2).
- **DC5-delta (FIGÉ — BH2-2, signe + seuil)** —
  `reconciliation_delta := total_vat_due (dérivé invoice_lines) − solde_2200_ventes (grand livre)`.
  - **Signe** : positif = TVA facturée **>** TVA comptabilisée sur `2200` (une ligne 2200 a été réduite/
    supprimée à la main) ; négatif = comptabilisée **>** facturée (une ligne 2200 a été gonflée à la main).
  - **Solde 2200 ventes** = `SUM(credit) − SUM(debit)` des lignes ciblées (la TVA due est portée au
    **crédit** de `2200` ; signe Liability codé en dur — voir DC5-signe).
  - **Seuil d'alerte** : `reconciliation_status = "delta"` si `|reconciliation_delta| >= Decimal::new(1, 2)`
    (= `0.01`, 1 centime), sinon `"ok"` (sous le centime = arrondi numérique négligeable). Le rapport **n'est
    jamais bloqué**. ⚠️ **NE PAS utiliser `dec!(0.01)`** : la macro `dec!` (`rust_decimal_macros`) est en
    `[dev-dependencies]` uniquement (`kesh-report/Cargo.toml:24`) et n'est `use`-ée que dans `#[cfg(test)]`
    (`vat_report.rs:183`) → indisponible dans le code de production `generate()`. Utiliser
    `Decimal::new(1, 2)` (pattern existant `kesh-core/src/bank_imports.rs:260` pour le même seuil 1 centime).
- **DC5-signe (FIGÉ — cohérent DC4-ter)** — le solde 2200 ventes est `SUM(credit) − SUM(debit)` **codé en
  dur** (PAS un `CASE` sur `account_type`), car `2200` est **toujours** un compte Liability par construction
  comptable (TVA collectée au crédit). Un compte non-Liability configuré dans
  `default_vat_payable_account_id` relève d'une erreur de configuration hors scope (cas dégénéré non géré
  v0.2, à documenter dans le doc-comment du helper).
- **DC5-null (FIGÉ)** — si `default_vat_payable_account_id == NULL` (TVA due jamais configurée → aucune ligne
  `2200` n'a pu être générée), **rien à réconcilier** : `reconciliation_delta = Decimal::ZERO`,
  `reconciliation_status = "ok"`. Pas d'erreur, pas de bandeau. Symétrique du `None → 0` du récupérable
  (18-1d, l.136).
- **DC5-period (FIGÉ — cohérent DC4)** — le solde 2200 ventes est filtré par la **même fenêtre** que la TVA
  due dérivée : factures `validated` avec `i.date BETWEEN start AND end`. On passe par la jointure `invoices`
  (qui porte `status` + `date` + `journal_entry_id`), donc **pas** besoin d'un filtre `je.entry_date`
  séparé (l'écriture porte de toute façon `entry_date = invoice.date`). Cela garantit que les deux côtés du
  delta couvrent **exactement le même ensemble de factures** → en l'absence d'édition manuelle, delta == 0
  par construction.
- **DC5-empty (FIGÉ)** — le delta **n'entre pas** dans la définition de « rapport vide » (`isReportEmpty`,
  18-1d) : on **ne touche pas** `reports.api.ts:107-113`. En pratique, un rapport « vide » (0 vente + 0
  récupérable) a `total_vat_due = 0` et aucune ligne 2200 liée → delta = 0, status "ok", pas de bandeau —
  donc rien à afficher de toute façon. Le bandeau de réconciliation est rendu **inconditionnellement par
  rapport au flag `empty`** mais ne s'affiche que si `reconciliationStatus === "delta"` (AC9).
- **`reconciliation_status` = `String`** (valeurs `"ok"` / `"delta"`), **pas un nouvel enum** : minimise la
  surface (le front compare `=== "delta"`, le CSV/PDF affichent un libellé). Calculé en dur dans `generate`.

## Acceptance Criteria

- **AC1 — Solde 2200 ventes (helper)** — `generate` calcule le solde du compte
  `default_vat_payable_account_id` **isolé au périmètre ventes** :
  `SUM(jel.credit) − SUM(jel.debit)` des `journal_entry_lines` où `jel.account_id =
  default_vat_payable_account_id` **ET** l'écriture est référencée par une facture `validated` de la période
  (`i.company_id = ? AND i.status = 'validated' AND i.date BETWEEN ? AND ? AND i.journal_entry_id =
  jel.entry_id`). Scopé `company_id` (anti-IDOR). `COALESCE(…, 0)` → 0 si aucune ligne. Signe codé en dur
  (DC5-signe).
- **AC2 — Delta + statut** — `reconciliation_delta = total_vat_due − solde_2200_ventes` (DC5-delta) ;
  `reconciliation_status = "delta"` si `|delta| >= 0.01`, sinon `"ok"`. Les 2 champs sont ajoutés à la struct
  `VatReport` (serde `camelCase` → `reconciliationDelta` / `reconciliationStatus`) et sérialisés
  automatiquement par la route JSON existante (`reports.rs:544-574`, aucun changement de route).
- **AC3 — Compte non configuré (DC5-null)** — `default_vat_payable_account_id == NULL` → `reconciliation_delta
  = 0.00`, `reconciliation_status = "ok"` (pas d'erreur, pas de bandeau). Régression nulle.
- **AC4 — Cas nominal sans édition manuelle** — pour une période où toutes les factures validées ont leur
  écriture intacte, `solde_2200_ventes == total_vat_due` ⇒ `delta == 0`, `status == "ok"`. (Invariant par
  construction : même ensemble de factures, mêmes montants par taux.)
- **AC5 — Détection d'édition manuelle (cœur DC5)** — si une écriture liée à une facture validée a été
  modifiée via `PUT /journal-entries` de sorte que la ligne `2200` ne corresponde plus à la TVA facturée,
  `delta != 0` et `status == "delta"`. C'est la divergence que le cross-check révèle.
- **AC6 — Isolation périmètre ventes (F-OPUS-4)** — une **écriture manuelle sur `2200` non liée à une facture
  validée** (OD, auto-liquidation) **n'affecte PAS** `reconciliation_delta` (elle est exclue du solde de
  référence). Aucun faux positif.
- **AC7 — Anti-IDOR** — le solde 2200 ventes est scopé `i.company_id = ?` (et `jel`/`je` via la jointure) :
  une facture/écriture d'une autre company n'entre jamais dans le delta.
- **AC8 — Renderers CSV/PDF** — `render_vat_report_csv` et `render_vat_report_pdf` affichent, **après la ligne
  « Solde »**, une ligne « Écart de réconciliation » avec la valeur de `reconciliation_delta` (et,
  optionnellement, le statut). `VatPdfLabels::fr_ch_defaults()` reçoit le(s) nouveau(x) libellé(s). Les
  gardes « achats seuls » de 18-1d (`rows.is_empty() && total_vat_recoverable.is_zero()`) restent inchangées.
- **AC9 — Front : bandeau d'alerte INFO non bloquant** — `VatReportView.svelte` affiche un bandeau (teinte
  avertissement, `role="alert"`, inline Tailwind façon `reports-equation-warning`) **si et seulement si**
  `dto.reconciliationStatus === 'delta'`. Le message indique l'écart (`reconciliationDelta` formaté via
  `formatReportAmount`) et oriente vers « vérifier les écritures validées modifiées manuellement ». Le
  bandeau **ne bloque pas** le rapport (pas de gate export, le `tfoot` reste affiché). `VatReportDto` gagne
  `reconciliationDelta` + `reconciliationStatus`. **`isReportEmpty` inchangé** (DC5-empty).
- **AC10 — i18n** — clés `reports-vat-reconciliation-*` (au moins : libellé écart pour CSV/PDF/front +
  message du bandeau) présentes dans les **4** locales (fr/de/it/en), parité `lint-i18n-ownership`. Libellé
  écart FR ≈ « Écart de réconciliation » ; message bandeau FR ≈ « Le décompte ne correspond pas aux écritures
  comptables (écart : { $delta }). Vérifiez les écritures validées modifiées manuellement. ». ⚠️ **F4 —
  l'interpolation FTL exige `{ $delta }` (avec `$`)** : la regex de substitution (`i18n.svelte.ts:17`)
  matche `\{\s*\$(\w+)\s*\}`. Écrire `{delta}` sans `$` → variable jamais substituée, `{delta}` littéral
  affiché (défaut silencieux). Cf. clés existantes `error-entry-unbalanced` (`{ $debit }`/`{ $credit }`),
  `vat-purchase-description` (`{ $rate }`).
- **AC11 — Tests** (intégration `#[sqlx::test]` niveau `kesh-report`, + E2E si pertinent) :
  - (a) **nominal delta 0** : ≥1 facture validée avec TVA, écriture intacte → `delta == 0`, `status == "ok"` ;
  - (b) **édition manuelle (delta ≠ 0)** : facture validée puis UPDATE de la ligne `2200` de son écriture
    (réduire le crédit) → `delta == écart attendu`, `status == "delta"` (AC5) ;
  - (c) **isolation (AC6/F-OPUS-4)** : écriture manuelle sur `2200` **non liée** à une facture validée (OD)
    → **n'affecte pas** le delta (delta reste 0 si factures intactes) ;
  - (d) **compte NULL (AC3)** : `default_vat_payable_account_id = NULL` → `delta == 0`, `status == "ok"` ;
  - (e) **anti-IDOR (AC7)** : facture/écriture `2200` d'une autre company n'affecte pas le delta ;
  - (f) **seuil** : un écart `< 0.01` (sous le centime) → `status == "ok"` ; un écart `>= 0.01` → `"delta"` ;
  - (g) **non-régression** : `total_vat_due` / `total_vat_recoverable` / `vat_balance` inchangés (les 7 tests
    `vat_report_recoverable.rs` + les E2E `vat_report_e2e.rs` restent verts) ;
  - (h) **renderer** : `render_vat_report_csv` contient la ligne « Écart de réconciliation » avec la valeur
    (cas delta ≠ 0).
- **AC12 — Quality gate « Test Locally First »** — backend `fmt`/`clippy -D warnings`/`build`/`test` (serial
  si `kesh-db`/intégration) + frontend `check`/`lint-i18n-ownership`/`test:unit`/`build`.

## Tasks (T-E1..T-E6)

- **T-E1 — Helper solde 2200 ventes** (`vat_report.rs`, jumeau de `recoverable_balance` l.156-178) :
  `due_account_balance_sales_scope(pool, company_id, account_id, period) -> Result<Decimal, ReportError>` :
  ```sql
  SELECT COALESCE(SUM(jel.credit), 0) - COALESCE(SUM(jel.debit), 0)
  FROM journal_entry_lines jel
  INNER JOIN invoices i
    ON i.journal_entry_id = jel.entry_id
  WHERE i.company_id = ?
    AND i.status = 'validated'
    AND i.date BETWEEN ? AND ?
    AND jel.account_id = ?
  ```
  Signe `credit − debit` codé en dur (DC5-signe, Liability). `map_db_error` (pattern `vat_report.rs:81`).
  Doc `///` : DC5-iso (lien facture validée, pas libellé journal), DC5-signe, DC5-period (même fenêtre que la
  due dérivée), périmètre = ventes uniquement (résiduel manuel hors-scope).
  > **Note** : la jointure `invoices i ON i.journal_entry_id = jel.entry_id` est l'isolation « lien facture
  > validée ». Elle exclut nativement (i) les OD manuelles sur 2200 sans facture, (ii) les factures non
  > validées, (iii) les autres companies (via `i.company_id`). Pas de filtre `journal` textuel (F-OPUS-4 +
  > robustesse `default_sales_journal` configurable).
- **T-E2 — Champs struct + calcul dans `generate`** (`vat_report.rs`) :
  - Ajouter `pub reconciliation_delta: Decimal` + `pub reconciliation_status: String` à `VatReport`
    (l.56-67). Doc `///` sur chaque champ (delta = due dérivé − solde 2200 ventes ; status "ok"/"delta",
    seuil 0.01).
  - ⚠️ **F1 (HIGH) — mettre à jour TOUS les sites de construction littérale `VatReport { … }`** (sinon
    `cargo build --workspace --all-targets` rouge — Rust exige tous les champs présents). `grep -rn
    "VatReport {" crates/` donne **4 sites** : (1) `vat_report.rs:140` (`generate`, le vrai) ; (2)
    `vat_report.rs:211` (`fn aggregate()` sous `#[cfg(test)]`) ; (3) `pdf.rs:1230` (`fixture_vat(empty)`) ;
    (4) `pdf.rs:1273` (test `vat_report_pdf_recoverable_only_renders`). Les sites 2-4 (tests) reçoivent
    `reconciliation_delta: Decimal::ZERO, reconciliation_status: "ok".to_string()`. Re-grep avant de figer
    (le code a pu bouger).
  - Dans `generate`, **avant** la construction du `VatReport { … }` (l.140) : lire
    `default_vat_payable_account_id` (`SELECT … FROM company_invoice_settings WHERE company_id = ?`,
    `Option<i64>` — réutiliser/factoriser le pattern de lecture déjà présent l.124-137 pour le récupérable ;
    on peut lire les **deux** comptes dans la même requête `SELECT default_vat_payable_account_id,
    default_vat_recoverable_account_id …` pour éviter 2 allers-retours). Si `Some(id)` → helper T-E1, sinon
    `Decimal::ZERO`. Calculer `delta = total_vat_due − solde` ; `status = if delta.abs() >= Decimal::new(1, 2)
    { "delta".to_string() } else { "ok".to_string() }` (⚠️ **PAS `dec!(0.01)`** — voir DC5-delta, macro
    dev-only). Mapper l'erreur DB via `map_db_error`.
  - `reconciliation_delta`/`reconciliation_status` ajoutés au `VatReport { … }` retourné (site 1).
- **T-E3 — Renderers CSV/PDF** (AC8) :
  - **CSV** `render_vat_report_csv` (`csv.rs:345-364`) : après la ligne « Solde » (l.363-364), écrire une
    ligne `["Écart de réconciliation", "", report.reconciliation_delta…]` (libellé i18n FR par défaut,
    cohérent avec les libellés en clair existants du CSV). Garde « achats seuls » 18-1d inchangée.
  - **PDF** `render_vat_report_pdf` (`pdf.rs:979-986`) : après « Solde » (l.986), un appel
    `draw_totals_footer(... reconciliation_delta ...)`. Enrichir `VatPdfLabels` (l.879-912) d'un champ
    `reconciliation_delta_label: String` + valeur dans `fr_ch_defaults()`.
- **T-E4 — Front** (`VatReportView.svelte`, `reports.types.ts`) :
  - `reports.types.ts:100-107` : ajouter `reconciliationDelta: string;` + `reconciliationStatus: 'ok' |
    'delta';` après l.106.
  - `VatReportView.svelte` : bandeau inline rendu **inconditionnellement vs `empty`** mais conditionné à
    `dto.reconciliationStatus === 'delta'` — p.ex. en tête de section (après l'en-tête, avant le `{#if
    empty}` qui est à **l.69**, `</table>` l.68, `</section>` l.70). ⚠️ **F3 — signature `i18nMsg` exacte**
    (`i18n.svelte.ts:14` : `i18nMsg(key: string, fallback: string, args?: Record<string, string | number>)`)
    — le 2e argument est le **fallback string** (PAS l'objet args), l'interpolation se fait au 3e arg avec la
    syntaxe `{ $var }` (regex `i18n.svelte.ts:17`). Exemple correct :
    ```svelte
    {#if dto.reconciliationStatus === 'delta'}
      <p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="alert">
        {i18nMsg('reports-vat-reconciliation-warning',
          'Le décompte ne correspond pas aux écritures comptables (écart : { $delta }). Vérifiez les écritures validées modifiées manuellement.',
          { delta: formatReportAmount(dto.reconciliationDelta) })}
      </p>
    {/if}
    ```
    (cf. patterns `settings/api-keys/+page.svelte:92`, `VatPurchaseAssistant.svelte:113`.) **NE PAS** toucher
    `isReportEmpty` (`reports.api.ts`) ni le `tfoot`.
- **T-E5 — Tests** (AC11 a-h) : nouveau `crates/kesh-report/tests/vat_report_reconciliation.rs`
  (`#[sqlx::test]`). Réutiliser le pattern de fixtures de `vat_report_recoverable.rs` / `vat_report_e2e.rs`
  (seed company + `company_invoice_settings` + comptes + facture validée). ⚠️ **F5 — créer un contact AVANT
  toute facture** : `NewInvoice` exige un `contact_id: i64` obligatoire (`entities/invoice.rs:67-74`) et
  `seed_accounting_company` n'en crée aucun. Après `seed_accounting_company`, appeler
  `seed_contact_and_product(&pool, seeded.company_id).await.unwrap()` (`test_fixtures.rs:464-487`, publique)
  pour obtenir un `contact_id`, puis `NewInvoice { contact_id, … }` → `invoices::create` → `validate_invoice`
  (qui pose `journal_entry_id`). **Atteignabilité confirmée (ground-truth)** : `kesh-db` est une dépendance
  **normale** de `kesh-report` (`kesh-report/Cargo.toml` `[dependencies] kesh-db = { path = "../kesh-db" }`)
  et `invoices::create` (`invoices.rs:343`) + `invoices::validate_invoice` (`invoices.rs:1019`) sont **`pub`**
  → directement appelables depuis un test `kesh-report`. **Helper de référence à répliquer** :
  `create_validated_invoice` (`crates/kesh-api/tests/vat_report_e2e.rs:144-172`, appelle `invoices::create`
  puis `invoices::validate_invoice` **directement, PAS via HTTP**) + `seed_contact` (`:120-141`). Copier ce
  pattern dans le nouveau fichier de test. ⚠️ **Pré-requis config** : `validate_invoice` d'une facture à
  `vat_rate > 0` exige `default_vat_payable_account_id` configuré (sinon `DbError::ConfigurationRequired`,
  `invoices.rs:980-981`) → la fixture doit UPDATE `company_invoice_settings` (`default_receivable_account_id`
  + `default_revenue_account_id` + `default_vat_payable_account_id`, compte `2000` réutilisé par convention
  fixtures 18-1b/d) AVANT de valider. Points de vigilance :
  - pour (b)/(e), **valider** une facture (via le repo/route qui crée l'écriture + pose `journal_entry_id`),
    puis **UPDATE** directement la ligne `2200` de l'écriture pour simuler l'édition manuelle ;
  - pour (c), créer une écriture manuelle (`journal_entries::create…`) sur `2200` **sans** la lier à une
    facture (aucune facture ne pointe son `journal_entry_id`) → vérifier qu'elle n'entre pas dans le delta ;
  - chaque `#[sqlx::test]` a sa DB éphémère → contrôler **toutes** les écritures, pas d'écriture parasite
    sur `2200` hors de celles attendues ;
  - vérifier le **seuil** (f) avec un écart de `0.005` (→ "ok") et `0.01` (→ "delta").
  - 1 assertion renderer (h) : `render_vat_report_csv` contient « Écart de réconciliation » + la valeur.
- **T-E6 — Quality gate + Change Log** (AC12). Doc-sync manuels/CHANGELOG **différée à 18-1f** (politique
  umbrella). Vérifier `lint-i18n-ownership` (parité 4 locales).

## Hors-scope (→ stories suivantes / limitations documentées)

- **Résiduel manuel hors-ventes sur `2200`** (auto-liquidation, régularisation AFC, OD non liées à une
  facture) : **exclu** du delta (DC5-iso). Non calculé/affiché v0.2 — l'utilisateur l'investigue via le grand
  livre. Documenter comme limitation dans le doc-comment du helper T-E1.
- **Réconciliation du compte récupérable `1171`** : pas de cross-check (source unique grand livre, DC4-bis).
  Le delta porte **uniquement** sur la TVA due.
- **Ventilation du delta par taux** : le delta est un scalaire global (pas par taux). Suffisant pour signaler
  une divergence ; le détail s'investigue au grand livre.
- **Décompte multi-exercice** : héritage F-OPUS-5 (période clampée intra-exercice, `ReportPeriod::resolve`).
  Le filtre par `i.date BETWEEN` reste cohérent.
- **Doc-sync** (manuels user/admin décompte TVA + bandeau réconciliation, CHANGELOG, README) → **18-1f**.

## Risques

- **Filtre périmètre (DC5-iso)** : le piège est de filtrer sur `journal = 'Ventes'` (libellé configurable) au
  lieu du lien `invoices.journal_entry_id`. Test (c) verrouille l'isolation (OD manuelle exclue) ; un filtre
  textuel ferait échouer (c) si l'utilisateur a un journal de ventes renommé, **et** inclurait à tort des OD
  classées « Ventes ». **Toujours passer par la jointure `invoices`.**
- **Signe (DC5-signe)** : la TVA due est au **crédit** de `2200` → solde ventes = `credit − debit` (inverse
  du récupérable Asset `debit − credit` de 18-1d). Une inversion donnerait un delta = `2 × due`
  systématique. Le test (a) (delta 0 nominal) verrouille le signe.
- **Cohérence des deux côtés du delta** : la due dérivée filtre `i.status='validated' AND i.date BETWEEN` ; le
  solde 2200 ventes doit filtrer **le même ensemble** (DC5-period) sinon delta ≠ 0 parasite. La jointure
  `invoices` partagée garantit l'alignement.
- **Non-régression sérialisation** : les tests existants (`vat_report_e2e.rs` ~8, `vat_report_recoverable.rs`
  7) asseront **par champ** / via `contains()` → l'ajout de 2 champs + 1 ligne CSV ne les casse pas (vérifié
  ground-truth). Mais **rajouter** un champ à la struct impose de mettre à jour **toute construction littérale
  de `VatReport`** (sinon erreur de compilation) — il n'y a qu'un site (`generate`), mais vérifier d'éventuels
  builders de test.
- **Bandeau toujours visible** : le rendre **hors** de la branche `{:else}` du `{#if empty}` (sinon il
  disparaîtrait sur un rapport « vide ») — bien que DC5-empty implique qu'un rapport vide a delta 0 (donc pas
  de bandeau), garder le rendu structurellement indépendant de `empty`.

## Prochaine étape

`bmad-create-story validate 18-1e` Pass 1 (Sonnet 4.6) — cycle adversarial CLAUDE.md (rotation
Sonnet→Haiku→Opus→…, contexte frais, grep ground-truth) jusqu'à 0 finding > LOW ou 8 passes. Puis
`bmad-dev-story 18-1e` (Opus). Dernière sous-story restante ensuite : **18-1f** (tests E2E + doc).

## Change Log

### `bmad-create-story validate 18-1e` — cycle adversarial (CLAUDE.md Review Iteration Rule)

| Passe | Modèle | Findings > LOW | Points clés |
|-------|--------|----------------|-------------|
| 1 | Sonnet 4.6 | 5 (2H+3M) | **Ground-truth complet** (struct l.56-67, generate l.72-148, recoverable_balance l.156-178, SQL jointure `invoices i ON i.journal_entry_id = jel.entry_id` confirmée colonne `jel.entry_id`, signe credit−debit cohérent DC5-signe, anti-IDOR via `i.company_id`, cohérence delta=0 nominale prouvée par construction, garde CSV/PDF inchangée, isReportEmpty inchangé, 4 locales). **F1 HIGH** : 3 struct-literals `VatReport{}` orphelins (`vat_report.rs:211` test `aggregate`, `pdf.rs:1230` `fixture_vat`, `pdf.rs:1273` test recoverable) → ajout 2 champs casse compilation `--all-targets` → T-E2 liste les 4 sites. **F2 HIGH** : `dec!(0.01)` indispo en prod (`rust_decimal_macros` dev-dep only `Cargo.toml:24`, `use` sous `#[cfg(test)]` `vat_report.rs:183`) → `Decimal::new(1, 2)` (pattern `bank_imports.rs:260`). **F3 MED** : signature `i18nMsg(key, fallback, args)` (`i18n.svelte.ts:14`) — exemple corrigé (fallback en 2e arg). **F4 MED** : interpolation FTL `{ $delta }` (avec `$`, regex `i18n.svelte.ts:17`) pas `{delta}` → sinon défaut silencieux. **F5 MED** : `NewInvoice.contact_id` obligatoire + `seed_accounting_company` ne crée pas de contact → T-E5 + `seed_contact_and_product` ; **risque archi tranché par orchestrateur** : `kesh-db` dep normale de `kesh-report` + `invoices::create`/`validate_invoice` `pub` → test reste en `kesh-report`, répliquer helper `create_validated_invoice` (`vat_report_e2e.rs:144`). LOW : n° lignes front décalés de 1 (F6, corrigé). **Tous patchés.** |
