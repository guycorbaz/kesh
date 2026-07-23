# Story 14.3c : Présentation des fonds propres PAR RÔLE au bilan

## Status

ready-for-dev

## Story

**As a** utilisateur de Kesh (indépendant, PME, fiduciaire) consultant son bilan,
**I want** que la section **Capitaux propres** soit présentée **par rôle** (capital, autres fonds propres, report, résultat) et **distincte des dettes**,
**so that** mon bilan soit conforme à la structure légale (CO art. 959a) et lisible — sans que le report d'ouverture d'une migration se confonde avec le report calculé par Kesh.

## Contexte

### Ce que 14-1, 14-3a et 14-3b ont livré (socle)

- **14-1** (clôture & report à-nouveau, modèle temps réel virtuel) : le bilan expose deux lignes de fonds propres **calculées virtuellement** — `retained_earnings` (« Résultat reporté » = cumul P&L des exercices **strictement antérieurs** à `fy_start`) et `equity_result` (« Résultat de l'exercice » = P&L sur `[fy_start, as_of]`). Équation défensive `total_assets == total_liabilities + retained_earnings + equity_result` (`balance_sheet.rs:130`, garde `equation_holds`). Le hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS = ["2979","2800"]` a été **retiré** → tous les comptes de passif (capital/réserves/report physiques inclus) tombent aujourd'hui dans `liabilities`, indistinctement des dettes.
- **14-3a** (socle rôles) : colonne `accounts.role` (enum `AccountRole`, 10 valeurs) + `singleton_role` généré + `postable`. Backfill migration `2800→EquityCapital`, `2970→RetainedEarnings`, `2979→CurrentYearResult`. `CurrentYearResult` **non-postable** (résultat calculé) ; `RetainedEarnings` **reste postable** (report d'ouverture d'un migrant). Clés i18n `account-role-*` livrées **aux 4 locales**.
- **14-3b** (consommateurs A+C) : garde `postable` à la saisie manuelle + lookups facturation par rôle. **Ne touche pas le bilan.**

### Le problème que 14-3c résout (note léguée 14-3a:142-148)

Aujourd'hui les comptes de fonds propres **physiques** (capital 2800, réserves 2900, report 2970) sont **noyés dans la section Passifs**, mélangés aux vraies dettes. Il n'existe **aucune section « Capitaux propres » séparée au calcul** — seules les 2 lignes virtuelles sont injectées **au rendu** (PDF `pdf.rs:492-521`, CSV `csv.rs:120-136`, frontend `BalanceSheetView.svelte:137-154` dans le `<tfoot>` des passifs).

Présenter les fonds propres **par rôle** fait apparaître **la collision léguée** : un compte physique de rôle `RetainedEarnings` (ex-2970, solde d'ouverture posé par un migrant) se retrouve **côte à côte** avec la ligne **calculée** « Résultat reporté » (cumul P&L Kesh antérieur). Deux nombres différents sous des libellés quasi-identiques. **L'arithmétique est saine** (grandeurs disjointes, additionnées une fois, l'équation 14-1 tient) — c'est un problème de **lisibilité**.

### Ce qui n'est PAS dans 14-3c

- ❌ Écran de saisie des soldes d'ouverture / migration → **14-4** (bilan d'ouverture).
- ❌ Rendre `RetainedEarnings` non-postable (décision explicite 14-3a : reste postable pour la migration).
- ❌ Lever la limitation i18n PDF/CSV (L4/L11) — décision D3 ci-dessous.
- ❌ Toute nouvelle migration / nouvelle colonne DB (le champ `role` existe déjà).

## Décisions de conception (tranchées par Guy 2026-07-23, avant dev)

### D1 — Collision « Résultat reporté » : DISTINGUER nettement (pas fusionner)

Le compte **physique** de rôle `RetainedEarnings` (postings réels d'ouverture, cumulés depuis l'origine comme tout compte de passif) et la ligne **calculée** `retained_earnings` (cumul P&L Kesh `entry_date < fy_start`) sont **deux grandeurs disjointes** et restent **deux lignes distinctes** :

- **Ligne compte physique** : itemisée dans le groupe de rôle `RetainedEarnings`, sous son numéro + nom de compte (ex. « 2970 Bénéfice/perte reporté »), affichée **uniquement si son solde ≠ 0** (cohérent `HAVING balance != 0` existant). Sémantiquement = report d'ouverture / ajustements manuels.
- **Ligne calculée** : « Résultat reporté (calculé par Kesh) » — libellé **explicitement marqué calculé**, sans numéro de compte, valeur = `bs.retained_earnings`.

La présence/absence de numéro de compte + le mot « calculé » lèvent l'ambiguïté. **Ne jamais additionner ni fusionner** les deux dans une seule ligne.

**Corollaire symétrique** : le compte physique de rôle `CurrentYearResult` (2979) est **non-postable** depuis 14-3a → solde toujours nul → n'apparaît **jamais** dans la section (le `HAVING balance != 0` l'exclut). Seule la ligne calculée « Résultat de l'exercice » (`equity_result`) subsiste. **Pas de collision active** — mais le code de partition ne doit pas supposer que `CurrentYearResult` est absent (un migrant pourrait, avant que 14-3a le rende non-postable, avoir un solde legacy ; le `HAVING` gère, ne pas ajouter de garde spéciale).

### D2 — Extraire une section « Capitaux propres » dédiée (restructurer l'équation)

Les comptes de **rôle equity** sont **sortis** de `liabilities` vers une nouvelle section `equity` :

- **Rôles equity** = `EquityCapital`, `EquityOther`, `RetainedEarnings`, `CurrentYearResult`. Un compte de passif dont `role` ∈ ces 4 valeurs va dans `equity` ; un compte de passif de `role` NULL **ou** de rôle non-equity (ex. `Payable`, `VatPayable`, `VatSettlement`) **reste** dans `liabilities` (dettes réelles).
- `total_liabilities` = Σ des **dettes réelles seulement** (change de valeur — c'est voulu).
- Nouveau `total_equity` = Σ des comptes physiques de la section equity (**hors** lignes virtuelles).
- **Équation restructurée** (garde `equation_holds` adaptée) :
  `total_assets == total_liabilities + total_equity + retained_earnings + equity_result`.
  (Mathématiquement identique à 14-1 : on n'a fait que **déplacer** des comptes de `liabilities` vers `equity`, la somme totale passif+equity est inchangée.)
- **Ordre d'affichage de la section equity** (par rôle, ordre CO 959a al. 2 — du capital vers le résultat) :
  1. `EquityCapital` (comptes itemisés)
  2. `EquityOther` (comptes itemisés, **multi-valué** — plusieurs comptes possibles)
  3. `RetainedEarnings` (comptes physiques itemisés, si solde ≠ 0) puis **ligne calculée** « Résultat reporté (calculé) »
  4. **Ligne calculée** « Résultat de l'exercice »
  5. `Total capitaux propres` = `total_equity + retained_earnings + equity_result`

Conforme CO art. 959a (distinction capitaux étrangers / capitaux propres).

### D3 — i18n : hardcode FR-CH au PDF/CSV, i18n complète au frontend

- **PDF/CSV** : les nouveaux libellés (titres de sous-groupes de rôles, ligne « calculé ») sont **hardcodés FR-CH** dans `SectionLabels` (PDF) / littéraux (CSV) — **cohérent** avec les libellés equity déjà hardcodés là-bas, et respecte la limitation v0.1 **L4/L11** (i18n délibérément non branchée sur les sérialiseurs, `reports.rs:1066`). **Ne pas** lever L4/L11 dans cette story.
- **Frontend** : i18n **complète** via les clés `account-role-*` **déjà livrées aux 4 locales** (14-3a/b, `messages.ftl:162-173`) + éventuelles nouvelles clés `reports-*` pour les lignes calculées / le titre de section (voir T7).

## Acceptance Criteria

### A. Backend — propagation du rôle & partition equity/dettes

- **Given** la struct `AccountBalance` (`balance_sheet.rs:75-86`) et sa requête SQL (`balance_sheet.rs:186-198`), **When** on charge une section, **Then** `AccountBalance` gagne un champ `role: Option<AccountRole>` et le SELECT ajoute `a.role` (import `AccountRole` depuis `kesh_db::entities`).
- **Given** `fetch_cumulative_section(AccountType::Liability)`, **When** on agrège les passifs, **Then** les comptes dont `role` ∈ {`EquityCapital`, `EquityOther`, `RetainedEarnings`, `CurrentYearResult`} sont **partitionnés** dans une nouvelle section `equity: Vec<AccountBalance>` ; les autres (role NULL ou non-equity) restent dans `liabilities`. La partition se fait **par rôle**, **jamais par numéro**.
- **And** `total_liabilities` = Σ des `liabilities` (dettes seules) ; nouveau `total_equity` = Σ des comptes physiques de `equity`.
- **And** l'invariant de lookup est respecté : la partition opère sur des comptes déjà cumulés (`entry_date <= as_of`), pas de nouveau `WHERE role = ?` runtime ; si un tel lookup était introduit il **DOIT** porter `AND active = TRUE` (invariant 14-3a:140). *(Ici on filtre en mémoire le résultat d'une requête existante qui ne filtre pas `active` — préserver ce comportement : un compte equity archivé à solde non-nul reste affiché, cohérent avec les passifs.)*

### B. Backend — struct BalanceSheet & équation restructurée

- **Given** `BalanceSheet` (`balance_sheet.rs:53-68`), **When** on la sérialise, **Then** elle expose `equity: Vec<AccountBalance>` et `total_equity: Decimal` (camelCase JSON `equity` / `totalEquity`) en plus des champs existants (`retained_earnings`, `equity_result` conservés).
- **And** l'équation vérifiée devient `total_assets == total_liabilities + total_equity + retained_earnings + equity_result` ; `equation_holds` est recalculé sur cette base, `warn!` si faux (comportement 14-1 préservé, `balance_sheet.rs:130-140`).
- **And** la garde « rapport vide » (`is_empty`) est étendue : le rapport n'est vide que si assets **ET** liabilities **ET** equity **ET** retained_earnings **ET** equity_result sont tous nuls (ne pas masquer une section equity non-nulle).

### C. Export CSV — section Capitaux propres par rôle (FR-CH hardcode)

- **Given** `render_balance_sheet_csv` (`csv.rs:60-150`), **When** on exporte, **Then** une section `CapitauxPropres` liste les comptes physiques equity **groupés par rôle** (une ligne `Section;NumeroCompte;NomCompte;Solde` par compte, ordre D2), suivie des **2 lignes calculées** distinctes (« Résultat reporté (calculé) », « Résultat de l'exercice »), puis `Total capitaux propres`.
- **And** les comptes equity **ne figurent plus** dans la section `Passifs` (partition D2). `Total passifs` = dettes seules.
- **And** la ligne invariant finale reste correcte : `total_liabilities + total_equity + retained_earnings + equity_result`.
- **And** labels **hardcodés FR-CH** (littéraux inline, cohérent existant `csv.rs:120-136`).

### D. Export PDF — section Capitaux propres par rôle (FR-CH hardcode)

- **Given** `SectionLabels` (`pdf.rs:72-137`) et `render_balance_sheet_pdf` (`pdf.rs:445-538`), **When** on rend le PDF, **Then** la section « Capitaux propres » liste les comptes physiques equity groupés par rôle (via `draw_account_row`), sous des sous-titres de rôle hardcodés FR-CH (nouveaux champs `SectionLabels` : ex. `equity_capital_label`, `equity_other_label`, `retained_earnings_account_label`), suivis des 2 lignes calculées distinctes (`retained_result_label` renommé pour marquer « calculé », `equity_result_label` conservé).
- **And** les comptes equity **ne figurent plus** dans la section Passifs.
- **And** la pagination existante (`ensure_space_for_row` → `new_page`, `pdf.rs:273-277`) couvre les nouvelles lignes (croissance N pages OK — pas de `content_floor`/`TooManyLines`, notion étrangère aux rapports).
- **And** le pied équation devient `Total passifs + Capitaux propres` = `total_liabilities + total_equity + retained_earnings + equity_result`.

### E. Frontend — section Capitaux propres par rôle (i18n complète)

- **Given** `BalanceSheetView.svelte` (`:137-154`) et `reports.types.ts` (`AccountBalance:12-19`, `BalanceSheetDto:21-31`), **When** on affiche le bilan, **Then** `AccountBalance` TS gagne `role: AccountRole | null`, `BalanceSheetDto` gagne `equity: AccountBalance[]` + `totalEquity`, et une **section « Capitaux propres » dédiée** (titre via clé `reports-section-equity`, déjà présente mais inutilisée) affiche les comptes groupés par rôle + les 2 lignes calculées distinctes.
- **And** les libellés de rôles utilisent les clés `account-role-*` (via `accountRoleKey()` `accounts.types.ts:41-43`) — **4 locales déjà couvertes**. Les comptes equity **ne figurent plus** dans la table Passifs.
- **And** la distinction D1 est visible : compte physique `RetainedEarnings` sous son n°/nom, ligne calculée « Résultat reporté » marquée calculée (clé i18n distincte, cf. T7).

### F. Non-régression

- **Given** un plan **standard** (rôles equity backfillés 14-3a) sans solde physique sur 2970, **When** on affiche le bilan, **Then** la section Capitaux propres montre capital/réserves itemisés + les 2 lignes calculées ; l'équation tient ; **aucun double-comptage** (grandeurs disjointes).
- **And** un plan **renuméroté** (rôles equity sur des numéros ≠ 2800/2900/2970) : la présentation par rôle reste correcte (partition par `role`, pas par numéro).
- **And** l'équation `equation_holds` reste `true` sur tous les fixtures de test existants (le déplacement liabilities→equity ne change pas la somme totale).

### G. Tests

- **Backend** (`kesh-report` + `kesh-db/tests/report_aggregates.rs`) :
  - partition : compte de rôle equity → section `equity`, compte de dette (role NULL / Payable) → `liabilities` ; `total_liabilities` + `total_equity` = ancien `total_liabilities`.
  - équation restructurée tient (`total_assets == total_liabilities + total_equity + retained_earnings + equity_result`).
  - collision D1 : un compte physique `RetainedEarnings` à solde 50 000 + ligne calculée `retained_earnings` 5 000 → **deux lignes distinctes**, somme equity correcte, pas de fusion.
  - `CurrentYearResult` non-postable à solde nul → absent de la section (via `HAVING balance != 0`).
  - plan renuméroté : rôle equity sur numéro non-standard → présent dans `equity`.
  - le test existant `balance_sheet_counts_2979_in_liabilities` (`report_aggregates.rs:404-458`) est **mis à jour** : 2979 (rôle `CurrentYearResult`) va désormais dans `equity`, pas `liabilities` — renommer/ajuster l'assertion.
- **Frontend** (Vitest) : la section Capitaux propres affiche les comptes groupés par rôle, distinction physique/calculé visible, comptes equity absents de la table Passifs.

## Tasks / Subtasks

- [ ] **T1 — Backend : `AccountBalance.role` + SELECT `a.role`** (`balance_sheet.rs:75-86`, `:186-198`). Import `AccountRole` (`kesh_db::entities`). `role: Option<AccountRole>`, `#[sqlx]` decode.
- [ ] **T2 — Backend : partition equity/dettes** dans `generate` (`balance_sheet.rs:89-152`). Après `fetch_cumulative_section(Liability)`, partitionner par rôle equity ({EquityCapital, EquityOther, RetainedEarnings, CurrentYearResult}) → `equity` vs `liabilities`. Helper `is_equity_role(role) -> bool` (source de vérité unique de la liste des 4 rôles equity — pas de duplication).
- [ ] **T3 — Backend : struct `BalanceSheet` + équation** (`balance_sheet.rs:53-68`). Ajouter `equity: Vec<AccountBalance>`, `total_equity: Decimal`. Recalculer `equation_holds` sur l'équation restructurée. Étendre `is_empty` (garde rapport vide) à `equity`.
- [ ] **T4 — CSV** (`csv.rs:60-150`) : section `CapitauxPropres` par rôle + 2 lignes calculées distinctes + `Total capitaux propres` ; retirer equity de `Passifs` ; labels FR-CH inline.
- [ ] **T5 — PDF** (`pdf.rs:72-137` + `:445-538`) : nouveaux champs `SectionLabels` (sous-titres de rôles + libellé calculé) hardcodés FR-CH ; rendre la section equity par rôle via `draw_account_row` ; retirer equity des Passifs ; pied équation ajusté. Défauts `fr_ch_defaults` (`pdf.rs:107-137`).
- [ ] **T6 — Frontend** (`BalanceSheetView.svelte`, `reports.types.ts`) : `role` sur `AccountBalance` TS, `equity`/`totalEquity` sur `BalanceSheetDto`, section « Capitaux propres » dédiée groupée par rôle (clés `account-role-*` via `accountRoleKey`), retrait equity de la table Passifs.
- [ ] **T7 — i18n** : vérifier/ajouter les clés `reports-*` frontend nécessaires (titre section equity = `reports-section-equity` **déjà présente** ; ligne calculée « Résultat reporté (calculé) » = clé distincte de `reports-retained-earnings` si le mot « calculé » doit apparaître — décider une clé `reports-retained-earnings-calculated` ou réutiliser l'existante). **4 locales** synchronisées si nouvelle clé.
- [ ] **T8 — Tests** (cf. AC-G) : backend partition/équation/collision/renuméroté + mise à jour `balance_sheet_counts_2979_in_liabilities` ; Vitest frontend.
- [ ] **T9 — Doc-sync** : CHANGELOG (nouvelle présentation fonds propres par rôle au bilan) ; manuel utilisateur si section bilan documentée ; pas de nouvelle limitation attendue hors L (voir Limitations).

## Dev Notes

### Ancres ground-truth (vérifiées 2026-07-23 par 3 agents de cartographie)

**Backend calcul** :
- `crates/kesh-report/src/balance_sheet.rs` : `generate` (`:89-152`), `fetch_cumulative_section` (`:172-210`, SQL `:186-198`), `fetch_retained_earnings` (`:225-248`), struct `BalanceSheet` (`:53-68`), struct `AccountBalance` (`:75-86`, **sans `role`**), équation (`:130`), doc module (`:30-38` — hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS` **déjà retiré**).
- Les comptes equity physiques sont **tous `account_type = Liability`** (`pme.json:37-40` : 2800 EquityCapital, 2900 EquityOther, 2970 RetainedEarnings, 2979 CurrentYearResult). `accepts_account_type("Liability")` validé (`chart_of_accounts/mod.rs:632`).
- Enum `AccountRole` : `crates/kesh-db/src/entities/account.rs:89-181` (10 variants, `as_str` `:129-142`, `is_singleton` `:157-169`). Rôles equity : `EquityCapital`/`EquityOther` (non-singleton, multi), `RetainedEarnings`/`CurrentYearResult` (singleton). **Duplication assumée** avec `kesh_core::chart_of_accounts::AccountRole` (orphan rule) — garde-fou test `singleton_list_matches_sql_generation_expression`.

**Export** :
- CSV : `crates/kesh-report/src/csv.rs:60-150` (equity `:120-136`, 2 lignes FR hardcodées).
- PDF : `crates/kesh-report/src/pdf.rs` — `SectionLabels` (`:72-137`, `fr_ch_defaults` `:107-137`), rendu equity (`:492-521`), `draw_account_row`, pagination `ensure_space_for_row` (`:273-277`), garde vide `is_empty` (`:461-465`).
- Handler : `crates/kesh-api/src/routes/reports.rs` — `get_balance_sheet` (`:140`), `export_balance_sheet` (`:279-312`), `load_pdf_context` (`:1046-1069`, `fr_ch_default` `:1066` → **i18n non branchée, L4/L11**).

**Frontend** :
- `frontend/src/lib/features/reports/BalanceSheetView.svelte` : equity dans `<tfoot>` passifs (`:137-154`), pas de regroupement.
- `frontend/src/lib/features/reports/reports.types.ts` : `AccountBalance` (`:12-19`, **sans `role`**), `BalanceSheetDto` (`:21-31`).
- `frontend/src/lib/features/accounts/accounts.types.ts` : `AccountRole` TS (`:54`), `accountRoleKey()` (`:41-43`), `AccountResponse.role` (`:79`).
- i18n : `crates/kesh-i18n/locales/*/messages.ftl` — `account-role-*` (`:162-173`, **4 locales**), `reports-section-equity` (`:951`, **présente mais inutilisée**), `reports-retained-earnings` (`:989`), `reports-equity-result-*` (`:986-988`).

### Invariant légué de 14-1 (à ne pas perdre)

- **Pas de double-comptage par construction** (14-1:34) : en modèle virtuel on ne poste rien au résultat/report ; les postings physiques sur 2970/2800 sont des soldes d'ouverture pré-Kesh, **disjoints** du cumul P&L Kesh. Le déplacement liabilities→equity **ne change pas la somme** → l'équation tient trivialement. La garde `equation_holds` reste le filet.

### Pièges, par ordre de coût

1. **Ne pas casser l'équation** : `total_liabilities` **change de valeur** (perd les comptes equity). Tout code qui lisait `total_liabilities` comme « passifs + fonds propres » doit être audité (CSV/PDF/frontend pieds de page). La somme `total_liabilities + total_equity` = ancien `total_liabilities`.
2. **Source unique de `is_equity_role`** : la liste des 4 rôles equity ne doit exister **qu'à un endroit** (helper backend). Ne pas la dupliquer entre partition, CSV, PDF. Le frontend dérive du champ `role` par ligne (pas de liste en dur TS).
3. **Collision D1** : ne jamais fusionner compte physique `RetainedEarnings` et ligne calculée. Test discriminant obligatoire (AC-G).
4. **`HAVING balance != 0`** gère `CurrentYearResult` non-postable (solde nul → absent). Ne pas ajouter de garde spéciale par rôle.
5. **i18n PDF/CSV NON branchée** (L4/L11) : hardcode FR-CH assumé (D3). Ne pas tenter de brancher l'i18n serveur ici.
6. **Test existant à retourner** : `balance_sheet_counts_2979_in_liabilities` (`report_aggregates.rs:404-458`) asserte 2979 dans `liabilities` — **désormais faux** (va dans `equity`). Le mettre à jour, pas le supprimer.

### Hors scope (garde-fous)

- ❌ Écran de saisie soldes d'ouverture → 14-4.
- ❌ Rendre `RetainedEarnings` non-postable (14-3a décision explicite).
- ❌ Lever i18n PDF/CSV L4/L11 (D3).
- ❌ Nouvelle migration / colonne DB.
- ❌ Modifier le calcul des lignes virtuelles 14-1 (`fetch_retained_earnings`, `income_statement::generate`) — on ne fait que les **reclasser** dans la présentation.

### Limitations documentées (catégorie B)

- **L1 (héritée L4/L11)** — libellés PDF/CSV du bilan hardcodés FR-CH (i18n serveur non branchée). Les libellés de rôles au PDF/CSV héritent de cette limitation. Lever = story i18n-sérialiseurs dédiée (hors 14-3c).
- **L2 — distinction physique/calculé textuelle.** La distinction D1 repose sur la présence du n° de compte + le mot « calculé », pas sur une séparation visuelle forte. Suffisant v0.1 ; un regroupement visuel plus riche (encadré, sous-total « report total ») = amélioration future si demandé.

### References

- Conception : [Source: note léguée 14-3a:142-148 (collision), 14-1:34-35/50 (modèle virtuel + renvoi 14-3), 14-3b:24-28/177 (sortie du chantier B)]
- Norme : [Source: research-swiss-co-958f.md:440 — CO art. 959/959a/959b, distinction capitaux étrangers/propres]
- Conventions : [Source: CLAUDE.md § Test Locally First, § Review Iteration Rule, § Règle de commit]

## Dev Agent Record

### Agent Model Used
(à remplir par dev-story)

### Completion Notes List
(à remplir)

### File List
(à remplir)

## Change Log — create-story

Spec créée 2026-07-23 par cartographie ground-truth parallèle (3 agents : moteur backend bilan, couche export/frontend, contexte collision léguée). 3 décisions de conception tranchées par Guy avant dev (D1 distinguer, D2 section dédiée + équation restructurée, D3 hardcode FR-CH PDF/CSV + i18n frontend). Prochaine : `bmad-create-story validate` (boucle adversariale jusqu'à 0 > LOW).
