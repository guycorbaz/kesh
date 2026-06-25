---
status: ready-for-dev
epic: 18
story: 18-1d
type: feature
parent: 18-1
issue: 180
created: 2026-06-25
depends_on: [18-1a, 18-1b, 18-1c]
baseline_commit: 85cde8d
stepsCompleted: []
---

# Story 18-1d — TVA récupérable réelle dans le VatReport

> Extraite de l'umbrella convergée [`18-1-comptabilisation-tva-achats.md`](18-1-comptabilisation-tva-achats.md).
> **Axe (d)** — **DC4** : remplir `total_vat_recoverable` (câblé à `0` aujourd'hui) depuis le **solde du
> compte impôt préalable au grand livre** + recalculer `vat_balance`. Le front retire la note « à venir ».
> Dépend de 18-1a (compte `1171` + `default_vat_recoverable_account_id`), 18-1b (TVA due comptabilisée),
> 18-1c (assistant achats qui alimente le solde `1171`).

## User Story

**En tant que** comptable/fiduciaire d'une PME suisse,
**je veux** que le rapport TVA affiche la **TVA récupérable réelle** (lue du grand livre) et le **solde net
dû à l'AFC**,
**afin de** disposer d'un décompte TVA complet (TVA due − TVA récupérable) sans calcul manuel externe.

## Contexte ground-truth (vérifié `main` @ `85cde8d`, après 18-1a/b/c)

### Ce qui EXISTE (réutiliser, ne pas réinventer)

- **`VatReport` + `generate()`** : `crates/kesh-report/src/vat_report.rs`.
  - Struct (l.46-56) : `period, rows, total_base_ht, total_vat_due, total_vat_recoverable, vat_balance`
    (camelCase serde). **Les champs `total_vat_recoverable` et `vat_balance` existent déjà** et sont
    sérialisés.
  - `generate(pool, company_id, period)` (l.61-122) : dérive la **TVA due** des `invoice_lines` des factures
    `validated` filtrées par `i.date BETWEEN period.start_date AND period.end_date` (l.69-79). À la fin
    (**l.109-110**) : `let total_vat_recoverable = Decimal::ZERO; let vat_balance = total_vat_due -
    total_vat_recoverable;`. **C'est exactement ces 2 lignes que 18-1d remplace.**
- **`ReportPeriod`** : porte `fiscal_year_id`, `start_date`, `end_date`. **`generate()` du VatReport n'utilise
  QUE `start_date`/`end_date`** (pas `fiscal_year_id`) — cohérent avec DC4.
- **Pattern d'agrégation de solde par compte** : `crates/kesh-report/src/trial_balance.rs:62-99`. Pour un
  compte **Asset** : `balance = SUM(debit) − SUM(credit)` (`CASE … WHEN account_type IN ('Asset','Expense')`,
  l.71-75). ⚠️ `trial_balance.generate()` filtre par `je.fiscal_year_id = ?` **ET** `je.entry_date BETWEEN`
  (l.82-83) → **NE PAS** réutiliser tel quel : DC4 impose le filtre **`entry_date` seul** (cf. DC4 ci-dessous).
- **Compte impôt préalable** : `company_invoice_settings.default_vat_recoverable_account_id: Option<i64>`
  (= compte `1171`, posé 18-1a). À lire pour ce `company_id`. Repo
  `crates/kesh-db/src/repositories/company_invoice_settings.rs` (`get_or_create_default` /
  `get_or_create_default_in_tx`). En lecture seule ici, un simple `SELECT default_vat_recoverable_account_id
  FROM company_invoice_settings WHERE company_id = ?` suffit (la row existe dès l'onboarding / lazy-create).
- **Front** : `frontend/src/lib/features/reports/VatReportView.svelte` **affiche DÉJÀ**
  `dto.totalVatRecoverable` (l.61) et `dto.vatBalance` (l.65). Le type `VatReportDto`
  (`reports.types.ts:104-105`) a déjà `totalVatDue`/`totalVatRecoverable`/`vatBalance` (strings décimales).
  **Seul changement front** : retirer la **note « à venir »** (`reports-vat-recoverable-note`, l.71-74).
- **Renderers PDF/CSV** : `render_vat_report_csv` (`csv.rs:354-359`, écrit déjà les records « TVA
  récupérable » `report.total_vat_recoverable` + « Solde » `report.vat_balance`) et `render_vat_report_pdf`
  (`pdf.rs:980-983`, libellés `vat_recoverable`/balance) **rendent déjà ces champs** depuis la struct.
  **Aucun changement dans le cas nominal** (≥1 ligne de vente) : ils s'allument automatiquement quand
  `generate()` remplit la valeur. ⚠️ **Exception** : les deux court-circuitent sur `report.rows.is_empty()`
  (`csv.rs:12`) → cf. AC10/F1 (cas « achats seuls »).
- **Route** : `GET /api/v1/reports/vat` (`routes/reports.rs`) appelle `vat_report::generate` — inchangée.

### Ce qui N'EXISTE PAS (à concevoir — minimal)

- La lecture du **solde du compte `1171`** sur la période n'est pas faite (`total_vat_recoverable = 0`).

## Décisions figées (héritées umbrella — NE PAS re-litiger)

- **DC4 (FIGÉ)** — `total_vat_recoverable` = **solde du compte `default_vat_recoverable_account_id` lu du
  grand livre**, agrégé en `SUM(debit) − SUM(credit)` (compte Asset, le préalable s'accumule au débit), avec
  le filtre **`je.entry_date BETWEEN period.start_date AND period.end_date` SANS `fiscal_year_id`** — pour
  rester cohérent avec le filtre `i.date BETWEEN` de la TVA due (`vat_report.rs:72-74`).
  - **F-OPUS-5 (hypothèse documentée)** : pas de chevauchement de `fiscal_years` sur une même date
    (invariant existant) ⇒ filtrer par `entry_date` seul est sûr pour un décompte **intra-exercice**. Un
    décompte **multi-exercice** (exercice non calendaire à cheval sur 2 fy) nécessiterait de réintroduire
    `fiscal_year_id` — **hors scope 18-1d**, à documenter dans le doc-comment.
- **DC4-bis (périmètre intégral — FIGÉ Pass 1 F2)** — le solde du compte récupérable est pris
  **intégralement** : **toute** écriture touchant `default_vat_recoverable_account_id` dans la période
  compte (achats via assistant 18-1c, OD manuelles, corrections, reclassements). **Pas d'isolation de
  périmètre**, contrairement à DC5 côté TVA due (qui isole le périmètre ventes du solde `2200`). Justification :
  `1171` (impôt préalable) est un compte **dédié** par convention 18-1a — toute écriture dessus EST, par
  construction comptable, de la TVA récupérable. Une écriture erronée pointant `1171` est une erreur de
  saisie utilisateur, pas un faux positif à filtrer. C'est l'**asymétrie volontaire** avec DC5 (le compte
  `2200` reçoit des écritures hors-ventes légitimes — auto-liquidation, régularisation AFC — d'où son
  isolation ; `1171` non).
- **DC4-ter (signe codé en dur — FIGÉ Pass 1 F4)** — la formule est `SUM(debit) − SUM(credit)` **codée en
  dur** (PAS un `CASE` sur `account_type` comme `trial_balance.rs:71-75`), car le compte d'impôt préalable
  est **toujours** un Asset par construction comptable. Un compte de type non-Asset configuré dans
  `default_vat_recoverable_account_id` relève d'une **erreur de configuration hors scope 18-1d** (le signe
  serait alors inattendu, mais c'est un cas dégénéré non géré v0.2, à documenter dans le doc-comment).
- **Compte non configuré** : si `default_vat_recoverable_account_id` est **`NULL`** (company pas encore
  configurée), `total_vat_recoverable = Decimal::ZERO` (comportement actuel inchangé, pas d'erreur).
- **`vat_balance` = `total_vat_due − total_vat_recoverable`** (inchangé dans la formule, mais récupérable
  n'est plus 0).
- **HORS SCOPE (→ 18-1e)** : la **réconciliation / cross-check** (DC5 : `reconciliation_delta`,
  `reconciliation_status`, isolation périmètre ventes du solde 2200, bandeau d'alerte front). 18-1d ne fait
  **que** remplir le récupérable + le solde net. **Ne pas** ajouter de champ de réconciliation ici.
- **Signe du solde** : le solde du préalable (Asset) est normalement **positif** (débits ≥ crédits). Le
  rapport l'affiche **tel quel** (une valeur négative — sur-corrections — reste possible et n'est pas
  bloquée ; documenter). `vat_balance` peut donc devenir négatif si récupérable > due (crédit d'impôt AFC) —
  comportement correct (l'AFC doit alors un remboursement).

## Acceptance Criteria

- **AC1** — `vat_report::generate` lit `default_vat_recoverable_account_id` du `company_invoice_settings` de
  la company. Si `Some(account_id)`, calcule `total_vat_recoverable` = `SUM(debit) − SUM(credit)` des
  `journal_entry_lines` de ce compte, filtrées par `je.company_id = company_id` **ET**
  `je.entry_date BETWEEN period.start_date AND period.end_date` (DC4). Si `None` → `Decimal::ZERO`.
- **AC2** — `vat_balance = total_vat_due − total_vat_recoverable` (recalculé avec la vraie valeur).
- **AC3** — **Anti-IDOR** : la requête de solde est scopée `je.company_id = ?` (jamais lire le solde d'une
  autre company). Cohérent avec le scoping existant de `generate` (l.63 « toutes les lignes scopées par
  company_id »).
- **AC4** — **Filtre période (DC4/F-OPUS-5)** : le filtre est `entry_date` seul (PAS `fiscal_year_id`). Une
  écriture sur `1171` **hors** de la fenêtre `[start, end]` ne compte pas ; une écriture dans la fenêtre
  compte, quel que soit son `fiscal_year_id`.
- **AC5** — **Compte non configuré** : `default_vat_recoverable_account_id == NULL` → `total_vat_recoverable
  = 0.00`, `vat_balance = total_vat_due` (régression nulle vs comportement actuel).
- **AC6** — **Front (AC9 umbrella)** : `VatReportView.svelte` **retire** la note « à venir »
  (`reports-vat-recoverable-note`). L'affichage de `totalVatRecoverable` et `vatBalance` (déjà présent) est
  conservé ; le libellé du solde reflète « solde net dû à l'AFC ». Pas d'autre changement front (les champs
  sont déjà câblés).
- **AC7** — **PDF/CSV** : les exports TVA (`render_vat_report_pdf`, `render_vat_report_csv`) affichent la
  **valeur réelle** du récupérable et du solde (vérifié par test que la valeur n'est plus `0` quand le compte
  a un solde). **Cas par défaut** (au moins une ligne de vente) : aucun code à changer (struct-driven).
- **AC10 — Période « achats seuls » (F1, HIGH) : le récupérable doit rester visible sans aucune vente.**
  Aujourd'hui le rapport est traité comme **vide** dès que `rows` (lignes de vente) est vide, à 3 endroits :
  front `isReportEmpty('vat')` (`reports.api.ts:107-109` = `vr.rows.length === 0` → bandeau « aucune écriture »,
  le `tfoot` n'est pas rendu) ; CSV `render_vat_report_csv` (court-circuit `if report.rows.is_empty()`,
  `csv.rs:12`) ; PDF idem. **Avant 18-1d, recoverable=0 toujours → ce court-circuit était correct ; 18-1d le
  casse** : une PME avec uniquement des achats sur la période (récupérable > 0, 0 facture validée) verrait
  `total_vat_recoverable > 0` calculé mais **rien affiché** (écran, CSV, PDF). **C'est le cas trimestre de
  rénovation sans loyer facturé.** **Correctif (obligatoire)** : redéfinir « rapport TVA vide » comme
  **`rows.is_empty() ET total_vat_recoverable == 0`** aux 3 endroits :
  - front `isReportEmpty('vat')` → `vr.rows.length === 0 && Number(vr.totalVatRecoverable) === 0` ;
  - CSV `render_vat_report_csv` → ne court-circuiter que si `rows.is_empty() && total_vat_recoverable.is_zero()`,
    sinon écrire quand même le récapitulatif (CA HT 0, TVA due 0, **TVA récupérable X**, **Solde -X**) ;
  - PDF `render_vat_report_pdf` → idem (rendre le bloc totaux si récupérable ≠ 0).
  Le `VatReportView.svelte` rend déjà le `tfoot` inconditionnellement — il suffit que le parent ne bascule
  pas sur le bandeau vide (corrigé par `isReportEmpty`). Test : « achats seuls, 0 vente → récupérable et
  solde affichés à l'écran, au CSV et au PDF » (AC8 cas (h)).
- **AC8** — **Tests** (intégration `#[sqlx::test]`, niveau `vat_report`) :
  - (a) achat avec TVA récupérable (écriture `D 1171 81.00 / …`) dans la période → `total_vat_recoverable ==
    81.00`, `vat_balance == total_vat_due − 81.00` ;
  - (b) plusieurs écritures sur `1171` (débits + un crédit de correction) → solde = `Σdebit − Σcredit` ;
  - (c) écriture sur `1171` **hors** période (entry_date avant start / après end) → **exclue** ; inclure un
    test de **bornes** : `entry_date == start_date` et `== end_date` → **inclus** (BETWEEN inclusif, cohérent
    `i.date BETWEEN ? AND ?` de la TVA due, F5) ;
  - (d) `default_vat_recoverable_account_id == NULL` (UPDATE du champ à NULL) → `total_vat_recoverable == 0` ;
  - (e) anti-IDOR : une écriture `1171` d'une autre company n'affecte pas le total ;
  - (f) `vat_balance` peut être négatif si récupérable > due (cas crédit d'impôt) ;
  - (g) cohérence : la TVA due (`invoice_lines`) reste inchangée (non-régression du calcul existant) ;
  - **(h) « achats seuls » (F1)** : écriture récupérable dans la période **sans aucune facture validée**
    (rows vide) → `total_vat_recoverable > 0`, `vat_balance < 0`, et le rapport n'est **PAS** traité comme
    vide (front `isReportEmpty` false ; CSV/PDF rendent le récapitulatif). Vérifier au moins le backend +
    `render_vat_report_csv` (la valeur récupérable apparaît dans le CSV malgré 0 ligne de vente).
- **AC9** — Quality gate « Test Locally First » : backend fmt/clippy/build/test serial + frontend
  check/lint-i18n/test:unit/build (impact front minime = retrait note).

## Tasks (T-D1..T-D5)

- **T-D1** — Helper de lecture du solde récupérable dans `vat_report.rs` (ou inline dans `generate`) :
  `SELECT COALESCE(SUM(jel.debit),0) - COALESCE(SUM(jel.credit),0) FROM journal_entry_lines jel INNER JOIN
  journal_entries je ON je.id = jel.entry_id WHERE je.company_id = ? AND jel.account_id = ? AND je.entry_date
  BETWEEN ? AND ?`. Retourne `Decimal` (COALESCE → 0 si aucune ligne). Doc `///` : DC4 + F-OPUS-5
  (entry_date seul, intra-exercice ; multi-exercice hors scope).
- **T-D2** — Brancher dans `generate` (remplacer l.109-110) : lire
  `default_vat_recoverable_account_id` (`SELECT … FROM company_invoice_settings WHERE company_id = ?`,
  `Option<i64>`) ; si `Some(id)` → appeler le helper T-D1 ; sinon `Decimal::ZERO`. Recalculer `vat_balance`.
  Conserver le scoping anti-IDOR `company_id`. **Mapper les erreurs DB des 2 requêtes via
  `kesh_db::errors::map_db_error`** (pattern existant `vat_report.rs:81`). **Mettre à jour le doc-comment du
  module** (`vat_report.rs:8-14` « TVA récupérable : 0.00 — aucune source ») **et le commentaire de struct**
  (l.52 « 0.00 en v0.2 ») devenus faux (F7).
- **T-D3** — Front : (a) retirer la note « à venir » de `VatReportView.svelte` (bloc complet `<p>…</p>`,
  **l.70-75**, clé `reports-vat-recoverable-note`) ; (b) **redéfinir `isReportEmpty('vat')`**
  (`reports.api.ts:107-109`) → `vr.rows.length === 0 && Number(vr.totalVatRecoverable) === 0` (F1/AC10).
  Le libellé du solde reste la clé i18n existante `reports-vat-balance` (`VatReportView.svelte:64`) — **ne
  pas changer la clé** (évite un impact `lint-i18n-ownership`) ; ajuster seulement le **fallback FR** si
  besoin (« Solde net dû à l'AFC ») sans renommer la clé (F8). Ne PAS toucher le tableau des taux.
- **T-D4** — Tests d'intégration `vat_report` (AC8 a-h). Réutiliser le pattern de fixtures `vat_report_e2e.rs`
  / `invoices_validate_vat.rs` (seed company + settings via fixture 18-1c). **Le compte
  `default_vat_recoverable_account_id` est `1000` (Caisse CI) dans la fixture (réutilisé, Story 18-1c)** —
  ⚠️ poster les écritures de test sur `seeded.accounts["1000"]`. **Garde-fou F3** : chaque `#[sqlx::test]`
  ayant sa DB éphémère, le test contrôle **toutes** les écritures ; ne créer **aucune** écriture parasite sur
  `1000` (ex. pas d'écriture de paiement Caisse) hors de celles que le test attend, sinon les assertions de
  solde (`== 81.00`) seraient faussées. La contrepartie des écritures de test ne doit PAS être `1000`
  (utiliser un autre compte, ex. `2000`). Pour le cas (d) NULL, `UPDATE company_invoice_settings SET
  default_vat_recoverable_account_id = NULL WHERE company_id = ?`.
- **T-D5** — Quality gate + Change Log.

## Hors-scope (→ stories suivantes)

- **Réconciliation rapport ↔ grand livre (DC5)** : `reconciliation_delta`, cross-check solde 2200 isolé
  périmètre ventes, bandeau d'alerte → **18-1e**.
- **Décompte multi-exercice** (exercice à cheval) : filtre `entry_date` seul suffit pour intra-exercice
  (F-OPUS-5) ; multi-exercice documenté comme limitation.
- **Ventilation du récupérable par taux** : le récupérable est un **solde de compte** (montant unique), pas
  ventilé par taux (l'Option B manuel ne porte pas le taux par ligne d'écriture). Le tableau des taux du
  rapport reste côté TVA due uniquement.

### Migration / doc

- **Aucune migration** (lecture seule du grand livre existant).
- **Doc-sync** (manuels décompte TVA) différée à **18-1f**.

## Risques

- **Filtre période (DC4)** : le piège est de réutiliser `trial_balance.generate()` tel quel (qui filtre par
  `fiscal_year_id`). 18-1d DOIT filtrer par `entry_date` seul pour matcher la TVA due. Test (c) verrouille.
- **Compte de test = `1000` réutilisé (fixture 18-1c)** : ne pas chercher un compte `1171` dans la fixture
  (elle réutilise `1000` comme récupérable). Les écritures de test doivent poster sur le compte pointé par
  `default_vat_recoverable_account_id` (= `1000`).
- **Signe Asset** : `SUM(debit) − SUM(credit)` (PAS l'inverse). Une inversion donnerait un récupérable
  négatif systématique. Le `CASE` de `trial_balance.rs:72-74` est la référence.
- **Cohérence avec 18-1e** : ne pas anticiper la réconciliation ici (pas de champ delta) — 18-1e ajoutera
  le cross-check sur la même `generate`.

## Prochaine étape

`bmad-create-story validate 18-1d` (rotation Sonnet→Haiku→Opus→…, contexte frais) avant la `dev-story`.

## Change Log

### `bmad-create-story validate 18-1d` — cycle adversarial (CLAUDE.md Review Iteration Rule)

| Passe | Modèle | Findings > LOW | Points clés |
|-------|--------|----------------|-------------|
| 1 | Sonnet 4.6 | 4 (2H+2M) | **Ground-truth 8/8 confirmé** (struct/generate l.109-110, ReportPeriod, formule Asset trial_balance, settings Option<i64>, front affiche déjà, renderers struct-driven, fixture compte 1000, invariant overlap fiscal_years réel). **F1 HIGH (vérifié orchestrateur)** : front `isReportEmpty('vat')` (`reports.api.ts:107`) + CSV (`csv.rs:12`) + PDF court-circuitent sur `rows.is_empty()` → période « achats seuls » (récupérable>0, 0 vente) = récupérable invisible → AC10 + cas test (h) : redéfinir vide = `rows vide ET recoverable==0`. **F2 HIGH** : périmètre solde non isolé → DC4-bis (prise intégrale délibérée, `1171` dédié, asymétrie volontaire avec DC5). **F4 MED** : signe `debit−credit` codé en dur → DC4-ter (Asset par construction). **F3 MED** : compte test `1000` Caisse → T-D4 garde-fou écritures parasites. LOW : bornes BETWEEN inclusives (test), doc-comment module à corriger, libellé i18n sans renommer clé, map_db_error, n° lignes csv:354/note l.70-75. |

**Trend findings > LOW** : Pass 1 (Sonnet) 4 (2H+2M). Prochaine : Pass 2 (Haiku) contexte frais.
