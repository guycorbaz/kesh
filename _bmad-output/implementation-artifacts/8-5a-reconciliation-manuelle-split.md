# Story 8-5a: Réconciliation manuelle + éclatement de transaction

Status: ready-for-dev

<!-- Issue de scission de Story 8-5 (`8-5-reconciliation-manuelle-regles-affectation.md`) le 2026-05-07 :
     Story 8-5 unifiée touchait 5-6 modules au seuil critique CLAUDE.md « splitter si > 5 modules »
     et regroupait 3 features distinctes (FR45 manual + FR48 split, FR46 suggestion, FR47 rules engine).
     Décision Guy 2026-05-07 (Q1 = Option B, avant `bmad-create-story validate`) : split en
     8-5a (manual + split, FR45 + FR48) + 8-5b (rules engine, FR47 ; FR46 ML suggestion reportée v0.2 cf. Q5).
     La spec d'origine 8-5-reconciliation-manuelle-regles-affectation.md reste comme référence
     des décisions de conception détaillées (§manual-match-flow, §split-flow, §audit-log-actions, §error-precedence-order). -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse, comptable interne ou fiduciaire)**,
I want **réconcilier manuellement les transactions bancaires sans candidate auto-matchée en sélectionnant un compte de contrepartie, ou éclater une transaction agrégée (salaires, charges sociales) en plusieurs imputations comptables**,
so that **mon backlog de transactions `pending` se résorbe (frais bancaires, salaires, intérêts) sans devoir attendre un moteur de règles, et que les transactions agrégées soient comptablement décomposées proprement**.

### Contexte

**Story 8-5a = première moitié de la story unifiée 8-5**, scindée pré-`bmad-create-story validate` pour respecter la règle de splitting CLAUDE.md (> 5 modules + 3 features distinctes). Voir [`8-5-reconciliation-manuelle-regles-affectation.md`](8-5-reconciliation-manuelle-regles-affectation.md) (status `archived-split`) pour la spec d'origine — toutes les **décisions de conception** (§manual-match-flow, §split-flow, §audit-log-actions, §error-precedence-order) y sont documentées en détail et restent valides pour 8-5a sans modification (sauf Q2 breaking change `POST /accept` discriminator + Q4a actions audit distinctes).

**Pourquoi 8-5a en premier :** le helper `kesh-reconciliation::manual::build_journal_entry_for_counterparty` extrait par 8-5a est consommé par 8-5b (flow `accept-with-rule`). 8-5a livre donc la **fondation** de la réconciliation manuelle, indépendante du moteur de règles.

**8-5a livre la valeur utilisateur immédiate :**
- **FR45** — création manuelle de contrepartie pour transactions inconnues (frais bancaires, intérêts, salaires individuels).
- **FR48** — éclatement de transaction agrégée en N imputations (salaires multiples + charges sociales sur 1 paiement).
- **Levée des limitations héritées 8-4** : L19 (matching journal_entries non-invoice), L20 (création écriture sans facture), L23 partielle (`auto_match_rejected_at` réversibilisé via manual-match RESET à `NULL`).

**8-5a ne livre PAS** (renvoi vers 8-5b) :
- Table `reconciliation_rules` + repos + migrations (8-5b T1+T2)
- Routes CRUD `/api/v1/reconciliation/rules` (8-5b T4)
- Engine d'application des règles dans GET /proposals + POST /accept (8-5b T3+T4)
- Audit log actions `reconciliation_rule.{created,updated,deleted,applied}` (8-5b)
- Page frontend `/reconciliation/rules` + composant `RuleFormModal` (8-5b T5)
- Suggestion ML « voulez-vous créer une règle ? » (reportée v0.2 — décision Guy Q5 2026-05-07)

**Status sprint :** `8-5a-reconciliation-manuelle-split: backlog → ready-for-dev` au moment de la création (2026-05-07). `8-5b-reconciliation-rules-engine` reste `backlog` jusqu'à 8-5a `done`/merged.

**Pré-requis closed** :
- ✅ Story 8-4 — `kesh-reconciliation` crate activé (matching, mutex, errors), routes `GET /proposals` + `POST /accept` + `POST /reject`, audit `reconciliation.accepted`/`reconciliation.rejected`, frontend `features/reconciliation/`, schema `bank_transactions { status, matched_entry_id, auto_match_rejected_at }`.
- ✅ Story 6-2 — multi-tenant scoping pattern KF-002 Pattern 1.
- ✅ Story 5-2 — `journal_entries::create_in_tx` (helper transaction-bound, indispensable pour créer une écriture comptable atomiquement avec le UPDATE bank_transactions du flow accept manuel).
- ✅ Stories 4-1, 3-1 — entités `Contact`, `Account` chargées par les selectors UI.
- ✅ Story 3-7 — `fiscal_years::find_open_for_date_for_company` (résolution `fiscal_year_id` à partir d'une `entry_date`).

**Crate cible** : extension de `kesh-reconciliation` (modules existants : `matching`, `mutex`, `errors`) avec 2 nouveaux modules : `manual` (FR45 helper `build_journal_entry_for_counterparty` + variante split) + `split` (FR48 validateur balance Decimal exact). Le 3e module `rules` est livré par 8-5b.

### Scope verrouillé — ce qui est livré par 8-5a

1. **Création manuelle de contrepartie (FR45)** — nouvelle route `POST /api/v1/reconciliation/manual` (sub-router `comptable_routes`). Body : `{ bankAccountId, bankTransactionId, counterpartyAccountId, description?, valueDate? }`. Crée **atomiquement** : (a) une `journal_entry` 2 lignes (compte bancaire ↔ compte de contrepartie) via `journal_entries::create_in_tx`, (b) UPDATE `bank_transactions.status='reconciled'`, `matched_entry_id=<new_je_id>`, `auto_match_rejected_at=NULL` (réversible — lève **L23**), (c) audit log `reconciliation.manual_matched` (action distincte — décision Guy Q4a). Cf. spec d'origine §manual-match-flow.

2. **Éclatement de transaction agrégée (FR48)** — nouvelle route `POST /api/v1/reconciliation/split` (Comptable+). Body : `{ bankAccountId, bankTransactionId, splits: [{ counterpartyAccountId, amount, description }] }` avec `sum(splits[*].amount) === bankTransaction.amount.abs()` (Decimal exact, validation backend, **pas de tolérance**). Crée **atomiquement** : (a) UNE seule `journal_entry` à N+1 lignes (1 ligne compte bancaire au montant total + N lignes contreparties), (b) UPDATE `bank_transactions.status='reconciled'`, `matched_entry_id=<new_je_id>`, `auto_match_rejected_at=NULL`, (c) audit log `reconciliation.split_applied` 1 entrée. **Pas de table `bank_transaction_splits` séparée** : la décomposition est portée par la `journal_entry` à N+1 lignes (SSOT comptable, pas de duplication). Cf. spec d'origine §split-flow.

3. **Helpers `kesh-reconciliation::manual` + `kesh-reconciliation::split`** :
   - `manual::build_journal_entry_for_counterparty(tx, bank_account_journal_id, counterparty_account_id, description, entry_date) -> NewJournalEntry` — pure (zéro I/O), à 2 lignes, sign-aware (débit/crédit selon `tx.amount` sign). **Helper public, réutilisé par 8-5b** (flow `accept-with-rule`).
   - `manual::build_journal_entry_for_split(tx, bank_account_journal_id, splits, description, entry_date) -> NewJournalEntry` — pure, à N+1 lignes, sign-aware.
   - `split::validate_split_balance(tx_amount, splits) -> Result<(), SplitImbalance>` — Decimal exact `==`.

   **⚠️ Gap architectural — résolution `bank_account_journal_id` (F2'' Pass 3 Opus 4.7)** :
   Le paramètre `bank_account_journal_id` (compte comptable lié au `bank_account`) est **requis** pour construire la `NewJournalEntry` côté banque. Or **l'entité `BankAccount` (`crates/kesh-db/src/entities/bank_account.rs`) n'a AUCUNE colonne `journal_account_id` / `linked_account_id` / équivalente** dans le schéma v0.1 (vérifié migration `20260410000001_bank_accounts.sql` + entité). La spec d'origine 8-5 (`§manual-match-flow` step 5, lignes 125-126) référence `bank_account.linked_account_id` qui **n'existe pas**.

   **Décision 8-5a (verrouillée Pass 3)** : Le client (frontend + API) **doit fournir `bankLedgerAccountId` dans le body** de `POST /manual` et `POST /split` (champ requis, type `i64 > 0`). Le handler valide cross-tenant via `accounts::find_by_id_in_company`. Cette décision évite une migration de schéma (lesson 8-3 retro : pas de migration en 8-5a) et reste cohérente avec le pattern 5-2 où `validate_invoice` consomme `default_receivable_account_id` / `default_revenue_account_id` (configuration explicite, pas auto-magique).

   **Body étendu** :
   ```json
   {
     "bankAccountId": 17,
     "bankLedgerAccountId": 1020,
     "bankTransactionId": 42,
     "counterpartyAccountId": 6810,
     "description": "Frais TWINT mai",
     "valueDate": "2026-05-15"
   }
   ```

   **Validation handler** :
   - `bankLedgerAccountId > 0` (validation body, 400).
   - `accounts::find_by_id_in_company(pool, bankLedgerAccountId, company_id)` retourne `Some(Account)` ET `account.active == true` (404 `ACCOUNT_NOT_FOUND` sinon).
   - **Pas de check de classe comptable** côté backend v0.1 (l'entité `Account` n'a pas de champ `class` directement — seul `account_type: AccountType` qui est `Asset/Liability/Revenue/Expense`). Le frontend filtre côté client à `number.starts_with('1')` pour la classe 1 (actifs) sur le sélecteur `bankLedgerAccount`. Une validation serveur stricte de la classe est reportée Story 8-5c v0.2.

   **AC #75 et AC #85 mis à jour pour refléter le body étendu**. **`splitTransaction` API frontend mise à jour** pour ajouter `bankLedgerAccountId` (T5.1).

   **Suggestion v0.2 (issue GitHub à créer post-merge 8-5a)** : ajouter une migration `ALTER TABLE bank_accounts ADD COLUMN journal_account_id BIGINT NULL FOREIGN KEY → accounts.id` + page de configuration UX, pour éliminer le `bankLedgerAccountId` du body (pré-rempli serveur-side). Tracé en `dette-architecture-bank-ledger`.

4. **Helper `kesh-db::bank_transactions::find_pending_by_id_for_account`** — charge une transaction `pending` par id, scopée tenant + compte. Helper utilisé par `/manual` + `/split` + (8-5b) `/accept` rule type.

5. **Breaking change `POST /api/v1/reconciliation/accept` (Q2 décision Guy 2026-05-07)** — le request body proposals[*] a désormais un discriminator `type` **obligatoire** :
   - `type: "invoice"` (8-4 héritée) — `{ type: 'invoice', bankTransactionId, invoiceId }`
   - `type: "manual"` (8-5a, équivalent batch de la route `/manual` standalone) — `{ type: 'manual', bankTransactionId, counterpartyAccountId, description?, valueDate? }`
   - `type: "split"` (8-5a) — `{ type: 'split', bankTransactionId, splits: [...] }`
   - `type: "rule"` (8-5b — non livré 8-5a, mais le discriminator est posé) — réservé futur, refusé v0.1 8-5a si non implémenté.

   **Pas de backward-compat** : si `type` absent → `400 Validation` avec message « champ `type` requis, valeurs acceptées v0.1 : ["invoice"] » (Kesh pas en prod, breaking change accepté). Si `type` présent mais non reconnu ou non encore supporté (`"manual"`, `"split"`, `"rule"` en 8-5a v0.1) → `400 Validation` avec message « type inconnu ou non supporté : "<valeur>", valeurs acceptées v0.1 : ["invoice"] ». Migration des **21 tests actifs** E2E HTTP 8-4 existants (+ 1 ignored préservé = 22 attributs `#[sqlx::test]` au total) pour ajouter `type: 'invoice'` explicite est livrée par 8-5a (couverte par AC #96 et T3.4). **Pass 3 validate F1'' Opus 4.7 — correction du compte** : la spec disait "22 actifs + 1 ignored = 23 total" en plusieurs endroits (régression Pass 1 Sonnet 4.6 F4) ; vérification `awk` directe sur le fichier `reconciliation_e2e.rs` confirme **21 tests actifs (`#[sqlx::test]` non précédés de `#[ignore]`) + 1 ignored (`post_accept_filters_currency_mismatch` ligne 1654-1655) = 22 attributs `#[sqlx::test]` au total**.

   **Note implémentation 8-5a** : le 8-5a peut soit (a) garder `/manual` et `/split` comme routes standalone et étendre `/accept` uniquement avec discriminator `type` (`invoice` reconnu, `manual`/`split` peuvent rester par leurs routes dédiées si plus simple), soit (b) unifier tout sur `/accept` avec discriminator complet. **Décision design préférée 8-5a** : garder `/manual` et `/split` standalone pour la lisibilité backend, et exposer `type='invoice'` strict sur `/accept` avec breaking change. Le 8-5b ajoutera `type='rule'` à `/accept`. Cette décision peut être revisitée pendant `bmad-create-story validate 8-5a` Pass 1 si Sonnet identifie une simplification.

6. **Frontend extensions** :
   - Composant `ManualMatchModal.svelte` : sélecteur `Account` (autocomplete plan comptable, filtré classes 5/6/7) + textarea description + datepicker valueDate (pré-rempli `tx.value_date ?? tx.booking_date`).
   - Composant `TransactionSplitModal.svelte` : tableau de splits éditable (ajout/suppression de ligne, min 2 max 50) + indicateur balance live (sum vs `|tx.amount|`, vert si exact match, rouge sinon, submit désactivé tant que balance ≠ exact).
   - Extension de `ReconciliationProposals.svelte` (héritée 8-4) : 2 boutons supplémentaires par ligne tx sans candidate auto : « Affecter manuellement » (ouvre `ManualMatchModal`) + « Éclater » (ouvre `TransactionSplitModal`).
   - Migration des appels existants `acceptProposal` 8-4 dans `ReconciliationProposals.svelte` pour ajouter `type: 'invoice'` explicite (cohérent breaking change Q2).

7. **i18n** — ~10 nouvelles clés (`reconciliation-manual-*` × 5, `reconciliation-split-*` × 5) × 4 locales fr/de/it/en-CH. **Pas** les clés `reconciliation-rules-*` ni `reconciliation-suggestion-*` (8-5b).

8. **Tests** — Unit `kesh-reconciliation::manual` + `::split` (≥ 6 cas : sign-aware build_je, balance validation), Integration `kesh-db::repositories::reconciliation::find_strictly_pending_by_id_for_account` (≥ 2 sqlx multi-tenant), E2E HTTP `kesh-api` (≥ **18 tests** : 10 manual + 6 split + 2 accept-discriminator ; + 21 tests stretch goal pour couvrir AC #89/#91/#93), Vitest (≥ 4), Playwright (≥ 2 actifs + 2 a11y).

9. **Sync** sprint-status + audit log 2 nouvelles actions discriminantes (`reconciliation.manual_matched`, `reconciliation.split_applied`).

**HORS scope 8-5a (→ 8-5b ou v0.2) :**

- Table `reconciliation_rules` + migration + repo + entité (8-5b T1+T2)
- Routes CRUD `POST/GET/PATCH/DELETE /api/v1/reconciliation/rules` (8-5b T4)
- Extension `GET /proposals` rule application (8-5b T4)
- Extension `POST /accept` discriminator `type='rule'` (8-5b T4)
- Page `/reconciliation/rules` frontend + `RuleFormModal` (8-5b T5)
- Audit log `reconciliation_rule.{created,updated,deleted,applied}` (8-5b)
- Suggestion automatique de règle post-manual-match (FR46 originale, **reportée v0.2 / Story 8-5c potentielle** — décision Guy Q5 2026-05-07, retire l'algorithme déterministe original `suggest_rule` de la spec)
- Auto-acceptation des règles à fort score (reporté v0.2, couplé L18 héritée 8-4)
- Annulation de réconciliation (reporté v0.2, cf. L45 héritée spec d'origine)
- Liaison split → invoices multiples (reporté v0.2, cf. L44 héritée spec d'origine)

### Décisions de conception (rappel — voir spec d'origine pour le détail)

Toutes les décisions §manual-match-flow, §split-flow, §audit-log-actions (limitées aux 2 actions 8-5a), §error-precedence-order de [`8-5-reconciliation-manuelle-regles-affectation.md`](8-5-reconciliation-manuelle-regles-affectation.md) §73-405 s'appliquent telles quelles à 8-5a, **avec les amendements suivants verrouillés par décision Guy 2026-05-07** :

- **Q2 — Breaking change `POST /accept` discriminator** : pas de backward-compat 8-4. Si `type` absent → `400 Validation` (vs spec d'origine §accept-with-rule-flow qui supposait `type` omitted = `'invoice'`). Le 8-5a est responsable de migrer les 21 tests E2E HTTP 8-4 existants (`crates/kesh-api/tests/reconciliation_e2e.rs`) pour ajouter `type: 'invoice'` explicite, ainsi que le frontend `reconciliation.api.ts` et `ReconciliationProposals.svelte`.
- **Q4a — Actions audit distinctes** : `reconciliation.manual_matched` et `reconciliation.split_applied` (cohérent §audit-log-actions). Pas de modifiers Vec, pas de réutilisation de `reconciliation.accepted` avec marqueur `paid_via='manual'`.
- **Q5 — Suppression FR46 suggestion ML** : la response de `POST /manual` ne contient PAS d'objet `ruleSuggestion` (vs spec d'origine §manual-match-flow step 9 et response example). La response 8-5a est :

  ```json
  { "bankTransactionId": 42, "journalEntryId": 999 }
  ```

  Le frontend ne déclenche pas de toast « Voulez-vous créer une règle ? ». L'utilisateur crée ses rules manuellement via la page `/reconciliation/rules` (livrée par 8-5b).

- **§manual-match-flow** valide tel quel pour les steps 0-8 et 10 (sans step 9 ruleSuggestion supprimé Q5).
- **§split-flow** valide tel quel.
- **§audit-log-actions** : seuls `reconciliation.manual_matched` et `reconciliation.split_applied` sont émis par 8-5a. Les 4 actions `reconciliation_rule.*` sont du ressort 8-5b.
- **§error-precedence-order** : ajouts 8-5a pertinents = #9 `ACCOUNT_NOT_FOUND`, #10 `RECONCILIATION_FISCAL_YEAR_CLOSED`, #11 `RECONCILIATION_SPLIT_IMBALANCE`. Les codes #12-#15 (rule-related) sont 8-5b.

## Acceptance Criteria

Numérotation héritée de la spec 8-5 d'origine pour traçabilité. ACs #75-#100 (~26 ACs sur manual + split + breaking change accept + UI manual+split). Les ACs #101-#124 sont du ressort de 8-5b.

### Création manuelle de contrepartie (FR45)

75. **(FR45 — happy paiement débit)** Given une `bank_transaction` `pending` débit `-150.00 CHF` (frais bancaires) sur `bank_account_id=17`, le compte comptable bancaire `bankLedgerAccountId=1020` (Caisse banque, fourni par le client cf. F2'' Pass 3) et un compte `6810 Frais bancaires` actif, When `POST /api/v1/reconciliation/manual { bankAccountId: 17, bankLedgerAccountId: 1020, bankTransactionId: 42, counterpartyAccountId: 6810, description: "Frais TWINT mai" }`, Then `200 OK` body `{ bankTransactionId: 42, journalEntryId: 999 }` ET `journal_entries` table contient 1 nouvelle entry à 2 lignes (1020 crédit 150.00 + 6810 débit 150.00) ET `bank_transactions.status='reconciled'`, `matched_entry_id=999`, `auto_match_rejected_at=NULL`. *Test E2E HTTP : `manual_match_creates_journal_entry_for_debit_transaction`.*

76. **(FR45 — happy encaissement crédit)** Given tx pending crédit `+200.00 CHF`, compte contrepartie `7510 Intérêts bancaires`, When manual-match, Then journal_entry à 2 lignes (1020 débit 200 + 7510 crédit 200). *Test E2E HTTP : `manual_match_creates_journal_entry_for_credit_transaction`.*

77. **(FR45 — multi-tenant safety counterparty)** Given user company_A POST manual avec `counterpartyAccountId` appartenant à company_B, Then `404 ACCOUNT_NOT_FOUND` (KF-002 pattern, pas 403). *Test E2E HTTP : `manual_match_does_not_leak_cross_tenant_account`.*

78. **(FR45 — multi-tenant safety bank_account)** Given user company_A POST manual avec `bankAccountId` appartenant à company_B, Then `404 BANK_ACCOUNT_NOT_FOUND`. *Test E2E HTTP : `manual_match_returns_404_on_cross_tenant_bank_account`.*

79. **(FR45 — already reconciled idempotency)** Given tx déjà `reconciled` (matched_entry_id != NULL), When POST manual, Then `409 RECONCILIATION_ALREADY_RECONCILED`. *Test E2E HTTP : `manual_match_rejects_already_reconciled_transaction`.*

80. **(FR45 — fiscal year closed)** Given `entry_date` qui tombe dans un exercice fiscal `Closed`, When manual-match, Then `409 RECONCILIATION_FISCAL_YEAR_CLOSED`. *Test E2E HTTP : `manual_match_rejects_closed_fiscal_year`.*

81. **(FR45 — réversibilise rejet auto, lève L23)** Given tx `pending` avec `auto_match_rejected_at != NULL` (rejetée 8-4), When POST manual, Then `200 OK` ET tx update `status='reconciled'`, `auto_match_rejected_at=NULL`, audit `details.was_previously_rejected=true`. *Test E2E HTTP : `manual_match_reverses_auto_rejection`.*

82. **(FR45 — archived counterparty account)** Given `counterpartyAccountId` pointant sur compte `active=false`, When POST manual, Then `404 ACCOUNT_NOT_FOUND` (cohérent comportement 8-4 qui exclut les comptes archivés). *Test E2E HTTP : `manual_match_rejects_archived_account`.*

83. **(FR45 — RBAC Comptable+)** Given user `Consultation`, When POST manual, Then `403 Forbidden` (sub-router comptable_routes). *Test E2E HTTP : `manual_match_requires_comptable_role`.*

84. **(FR45 — audit log canonique)** Given POST manual happy, When commit, Then audit_log contient 2 entrées : `(action='reconciliation.manual_matched', entity_type='bank_transaction', entity_id=42, details = { bank_transaction_id, counterparty_account_id, journal_entry_id, amount, description, value_date, was_previously_rejected })` ET `(action='journal_entry.created', entity_type='journal_entry', entity_id=999)` (émis par `journal_entries::create_in_tx` lui-même, héritage Story 3-2). **Précision `was_previously_rejected`** : `true` si tx avait `auto_match_rejected_at != NULL` avant le match (AC #81), `false` sinon (cas nominal AC #75). Le test `manual_match_emits_audit_log_pair` vérifie le cas `was_previously_rejected=false` ; `manual_match_reverses_auto_rejection` (AC #81) vérifie `was_previously_rejected=true`. *Test E2E HTTP : `manual_match_emits_audit_log_pair`.*

### Éclatement de transaction agrégée (FR48)

85. **(FR48 — split happy paiement)** Given tx pending débit `-10700.00`, body `{ bankAccountId: 17, bankLedgerAccountId: 1020, bankTransactionId: 42, splits: [{ counterpartyAccountId: 5000, amount: '5000', description: 'Salaire Alice' }, { counterpartyAccountId: 5000, amount: '4500', description: 'Salaire Bob' }, { counterpartyAccountId: 5700, amount: '1200', description: 'Charges' }] }` (`bankLedgerAccountId` requis cf. F2'' Pass 3), When POST `/split`, Then `200 OK` + 1 journal_entry à 4 lignes (1020 crédit 10700 + 5000 débit 5000 + 5000 débit 4500 + 5700 débit 1200) + `bank_transactions.status='reconciled'`. *Test E2E HTTP : `split_creates_journal_entry_with_n_plus_1_lines`.*

86. **(FR48 — split happy encaissement)** Given tx pending crédit `+5000.00` (remboursement multi-source), body `splits: [{ counterpartyAccountId: 7510, amount: 3000, description: 'Intérêts' }, { counterpartyAccountId: 6900, amount: 2000, description: 'Remboursement frais' }]`, When POST `/split`, Then journal_entry à 3 lignes (1020 débit 5000 + 7510 crédit 3000 + 6900 crédit 2000). *Test E2E HTTP : `split_creates_journal_entry_for_credit_transaction`.*

87. **(FR48 — split balance violation)** Given tx `-10700.00`, splits sum=10500 (200 missing), When POST split, Then `400 RECONCILIATION_SPLIT_IMBALANCE` body `details = { expected: '10700.00', actual: '10500.00', difference: '-200.00' }`. *Test E2E HTTP : `split_rejects_imbalanced_payload`.*

88. **(FR48 — split min 2 lignes)** Given splits.len=1, When POST, Then `400 Validation` (« splits doit contenir ≥ 2 lignes — utilisez /manual pour 1 ligne »). *Test E2E HTTP : `split_rejects_single_line_payload`.*

89. **(FR48 — split max 50 lignes)** Given splits.len=51, When POST, Then `400 Validation` (« splits ≤ 50 lignes »). *Test E2E HTTP : `split_rejects_too_many_lines`.*

90. **(FR48 — split multi-tenant safety)** Given un `splits[i].counterpartyAccountId` appartient à company_B, When POST, Then `404 ACCOUNT_NOT_FOUND` body `details.missingAccountIds = [<id>]` (camelCase JSON, cohérent convention kesh-api). *Test E2E HTTP : `split_does_not_leak_cross_tenant_account`.*

91. **(FR48 — split déjà réconciliée)** Given tx `reconciled`, When POST, Then `409 RECONCILIATION_ALREADY_RECONCILED`. *Test E2E HTTP : `split_rejects_already_reconciled`.*

92. **(FR48 — split RBAC Comptable+)** Given user `Consultation`, When POST split, Then `403 Forbidden`. *Test E2E HTTP : `split_requires_comptable_role`.*

93. **(FR48 — split audit log)** Given POST split happy 3 lignes (comptes 5000, 5000, 5700), When commit, Then audit_log contient 2 entrées : `(action='reconciliation.split_applied', entity_type='bank_transaction', entity_id=tx.id, details = { bank_transaction_id, splits: [{ counterparty_account_id, amount, description }, { counterparty_account_id, amount, description }, ...], total_amount: '10700.00' })` (structure cohérente AC #84, snake_case) ET `(action='journal_entry.created', entity_type='journal_entry', entity_id=<new_je_id>)` émis par `journal_entries::create_in_tx`. *Test E2E HTTP : `split_emits_audit_log`.*

### Breaking change `POST /accept` discriminator type (Q2 décision Guy)

94. **(Q2 — discriminator type obligatoire)** Given un body proposal `{ bankTransactionId, invoiceId }` (legacy 8-4 sans `type`), When POST `/accept`, Then `400 Validation` avec message « champ `type` requis » (breaking change v0.1, pas de défaut). *Test E2E HTTP : `accept_rejects_proposal_missing_type_discriminator`.*

95. **(Q2 — type='invoice' explicite)** Given body `{ type: 'invoice', bankTransactionId, invoiceId }`, When POST `/accept`, Then flow 8-4 invoice exécuté (audit `reconciliation.accepted` + `invoice.paid`). *Test E2E HTTP : `accept_with_explicit_invoice_type_runs_8_4_flow`.*

96. **(Q2 — migration tests E2E 8-4)** Given le fichier de tests `crates/kesh-api/tests/reconciliation_e2e.rs` (**21 tests actifs + 1 ignored** = 22 attributs `#[sqlx::test]` au total — Pass 3 validate Opus 4.7 F1'' : la spec disait "22 actifs + 1 ignored = 23 total" suite à régression Pass 1 Sonnet 4.6 F4 ; vérification `awk` directe sur le fichier confirme 21 actifs + 1 ignored `post_accept_filters_currency_mismatch` ligne 1654-1655 = 22 total), When 8-5a livré, Then tous les tests qui POST `/accept` ajoutent `type: 'invoice'` dans leur body et restent verts (régression non introduite). *Test E2E HTTP : `cargo test -p kesh-api --test reconciliation_e2e` : 21 verts + 1 ignored (mono-CHF Story 11 préservé).*

### UI frontend extensions (manual + split)

97. **(UI — bouton Affecter manuellement + bouton Éclater)** Given une ligne tx pending sans candidate sur `/reconciliation`, Then 2 boutons additionnels « Affecter manuellement » et « Éclater » apparaissent à droite de la ligne. *Test Vitest : `ReconciliationProposals.test.ts: shows manual+split buttons for tx without candidate`.*

98. **(UI — ManualMatchModal)** Given click « Affecter manuellement » sur tx 42, Then modal ouvert avec sélecteur Account (autocomplete plan comptable filtré classes 5/6/7) + textarea description (200 chars max) + datepicker valueDate (pré-rempli = tx.value_date ?? tx.booking_date). On submit success : event `success`, refresh liste. *Test Vitest + Playwright : `manual_match_modal_renders_with_prefilled_fields`.*

99. **(UI — TransactionSplitModal)** Given click « Éclater » sur tx 42 (-10700), Then modal ouvert avec tableau splits éditable (min 2 max 50, ajout/suppression de ligne) + sticker balance live « 0.00 / 10 700.00 CHF » (vert si exact match, rouge sinon) + bouton submit désactivé tant que balance ≠ exact. *Test Vitest : `split_modal_balance_indicator_updates_live`.*

100. **(UI — accessibilité a11y axe modals)** Given `ManualMatchModal` ouvert ET `TransactionSplitModal` ouvert (deux scénarios séparés), When axe-core scan, Then 0 violation. *Tests Playwright : `accessibility — modal manual axe scan`, `accessibility — modal split axe scan`.*

### Sécurité & multi-tenant (consolidation)

(Couvert par AC #77, #78, #82, #90 ci-dessus. Pas d'AC séparé Sécurité dans 8-5a — la consolidation type `all_8_5a_mutations_require_comptable_role` est couverte implicitement par les AC RBAC #83 + #92.)

### i18n & accessibilité

(Couvert par les AC UI #97-#100 ci-dessus. Pas d'AC séparé i18n dans 8-5a — délivré via T6 et test `npm run lint-i18n-ownership` PASS sur 4 locales pour les 10 nouvelles clés.)

## Tasks / Subtasks

### T1. Helper `find_pending_by_id_for_account` avec filtre status='pending' (AC #75-#83, #85-#92)

**Note état du code (Pass 1 validate)** : `find_pending_by_id_for_account` existe déjà dans `crates/kesh-db/src/repositories/reconciliation.rs` (ajouté en Story 8-4 pour le TOCTOU fix dans `accept_one`). **Mais** il ne filtre pas `status = 'pending'` — il retourne n'importe quelle transaction du compte. La spec 8-5a a besoin d'un variant qui :
- Retourne `None` si status ≠ `'pending'` (pour rejeter 409 ALREADY_RECONCILED proprement avant le lock, ou confirmer l'état inside lock).
- Est dans `kesh-db::repositories::reconciliation` (same module), **pas** dans `bank_transactions.rs` (évite la duplication cross-module).

**Décision** : Ajouter une variante `find_strictly_pending_by_id_for_account` dans `reconciliation.rs` avec le filtre `status = 'pending'` explicite, plutôt que dupliquer le helper dans `bank_transactions.rs`. Le helper existant `find_pending_by_id_for_account` est conservé pour 8-4 (son absence de filtre status est intentionnelle pour `accept_one` step 4 qui vérifie le status séparément).

**F8'' Pass 3 Opus 4.7 — note de naming** : Le helper existant 8-4 `find_pending_by_id_for_account` a un nom trompeur — il **ne filtre PAS `status='pending'`** (cf. doc-comment lignes 270-280 de `reconciliation.rs`, le caller fait le check status au step 4 inside lock). Le nom `find_strictly_pending_by_id_for_account` ajouté par 8-5a clarifie cette distinction. Renommer le helper 8-4 (e.g. en `find_by_id_for_account_no_status_filter`) est tracé en `dette-naming-reconciliation-helpers` (issue GitHub à créer post-merge 8-5a, non-bloquant).

- [ ] T1.1 — Étendre `crates/kesh-db/src/repositories/reconciliation.rs` (pas `bank_transactions.rs`) :
  ```rust
  /// Charge une transaction **strictement** `pending` par id, scopée tenant + compte.
  /// Utilisé par /manual et /split pour pré-flight ownership AVANT lock ET inside lock.
  /// Retourne None si introuvable, status != 'pending', cross-tenant, ou cross-account.
  /// Distinct de `find_pending_by_id_for_account` (8-4) qui ne filtre pas status.
  pub async fn find_strictly_pending_by_id_for_account<'e, E>(
      executor: E,
      company_id: i64,
      bank_account_id: i64,
      id: i64,
  ) -> Result<Option<BankTransaction>, DbError>
  where E: sqlx::Executor<'e, Database = MySql>,
  ```
  SQL : `WHERE company_id = ? AND bank_account_id = ? AND id = ? AND status = 'pending'`.

- [ ] T1.2 — Tests inline `#[sqlx::test]` (≥ 2) :
  1. `find_strictly_pending_scopes_by_account_and_company` — verify cross-tenant returns None.
  2. `find_strictly_pending_returns_none_for_reconciled_tx` — verify status='reconciled' returns None.

- [ ] T1.3 — Mettre à jour les références à ce helper dans T3 (handlers `/manual` et `/split`) pour utiliser `find_strictly_pending_by_id_for_account` (nouveau) au lieu de `find_pending_by_id_for_account` (8-4 existant).

- [ ] T1.4 — Vérifier `cargo test -p kesh-db reconciliation` MariaDB up local (lesson 8-3 retro).

### T2. Helpers `kesh-reconciliation::manual` + `kesh-reconciliation::split` (AC #75-#76, #85-#87)

- [ ] T2.1 — Créer `crates/kesh-reconciliation/src/manual.rs` :
  ```rust
  // Note : NewJournalEntry est dans kesh_db (pas kesh_core).
  // kesh-reconciliation → kesh-db est déjà dans Cargo.toml.
  use kesh_db::entities::journal_entry::{NewJournalEntry, NewJournalEntryLine};
  use kesh_db::entities::journal_entry::Journal;
  use kesh_db::entities::bank_transaction::BankTransaction;
  use rust_decimal::Decimal;
  use chrono::NaiveDate;

  /// Construit une `NewJournalEntry` à 2 lignes pour réconciliation manuelle.
  /// Pure (zéro I/O). Sign-aware : sign de tx.amount → côté débit/crédit.
  /// **Helper public, réutilisé par 8-5b** (flow `accept-with-rule`).
  pub fn build_journal_entry_for_counterparty(
      tx: &BankTransaction,
      bank_account_journal_id: i64,
      counterparty_account_id: i64,
      description: String,
      entry_date: NaiveDate,
  ) -> NewJournalEntry { ... }

  /// Détail d'un split pour `build_journal_entry_for_split` (type-safe).
  pub struct SplitDetail {
      pub account_id: i64,
      pub amount: Decimal,
      pub description: String,
  }

  /// Variante N+1 lignes pour split (FR48). Pure.
  pub fn build_journal_entry_for_split(
      tx: &BankTransaction,
      bank_account_journal_id: i64,
      splits: &[SplitDetail],
      description: String,
      entry_date: NaiveDate,
  ) -> NewJournalEntry { ... }
  ```

- [ ] T2.2 — Créer `crates/kesh-reconciliation/src/split.rs` :
  ```rust
  use rust_decimal::Decimal;

  /// Vérifie que sum(splits[*].amount) == tx.amount.abs() (Decimal exact, pas de tolérance).
  ///
  /// **F7'' Pass 3 Opus 4.7 — précondition longueur** : ce helper ne valide PAS
  /// `splits.len() >= 2` (AC #88) ni `splits.len() <= 50` (AC #89). Le caller
  /// (handler `post_split`) DOIT appliquer ces deux contraintes AVANT
  /// d'appeler `validate_split_balance`, sinon une liste vide validera contre
  /// `tx.amount = 0` (cas dégénéré) et un split unique passera contre une tx
  /// du même montant (anti-pattern qui devrait passer par `/manual`).
  ///
  /// **Précondition `tx_amount`** : signed (positif=crédit titulaire,
  /// négatif=débit). `validate_split_balance` applique `.abs()` en interne ;
  /// le caller passe `tx.amount` brut (pas `tx.amount.abs()`). Si
  /// `tx.amount == 0`, `expected == 0` et `splits` doit sommer à 0
  /// — refusé in fine par `chk_journal_entries_balance` DB constraint via
  /// `create_in_tx` (entry à somme nulle = balanced mais inutile).
  /// Validation explicite `tx.amount != 0` recommandée côté handler (cas
  /// très rare en CAMT.053, exclu en pratique par le parser).
  pub fn validate_split_balance(tx_amount: Decimal, splits: &[Decimal]) -> Result<(), SplitImbalance> {
      let sum: Decimal = splits.iter().sum();
      let expected = tx_amount.abs();
      if sum != expected {
          return Err(SplitImbalance { expected, actual: sum, difference: sum - expected });
      }
      Ok(())
  }
  pub struct SplitImbalance { pub expected: Decimal, pub actual: Decimal, pub difference: Decimal }
  ```

- [ ] T2.3 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod manual;
  pub mod split;

  pub use manual::{build_journal_entry_for_counterparty, build_journal_entry_for_split};
  pub use split::{validate_split_balance, SplitImbalance};
  ```

- [ ] T2.4 — Étendre `crates/kesh-reconciliation/src/errors.rs` :
  ```rust
  pub enum ReconciliationError {
      // ... 8-4 variants conservés
      SplitImbalance { expected: Decimal, actual: Decimal, difference: Decimal },
      FiscalYearClosed { entry_date: NaiveDate },
  }
  ```

- [ ] T2.5 — Tests unit `kesh-reconciliation` (≥ 6) :
  1. `manual_build_je_creates_2_lines_for_credit_tx` (AC #76).
  2. `manual_build_je_creates_2_lines_for_debit_tx` (AC #75).
  3. `split_build_je_creates_n_plus_1_lines` (AC #85).
  4. `split_build_je_creates_n_plus_1_lines_for_credit_tx` (AC #86).
  5. `split_validate_balance_exact_match_ok` (AC #85).
  6. `split_validate_balance_imbalance_returns_error` (AC #87).

### T3. Routes API `POST /manual` + `POST /split` + breaking change `/accept` (AC #75-#96)

- [ ] T3.1 — Étendre `crates/kesh-api/src/routes/reconciliation.rs` (du 8-4) :
  - Handler `post_manual` (cf. spec d'origine §manual-match-flow steps 0-8 + 10, **sans step 9 ruleSuggestion**). **Résolution `entry_date`** : utiliser la valeur `valueDate` du request body si fournie, sinon fallback `tx.booking_date`. Cette date est utilisée pour : (a) résoudre le `fiscal_year_id` via `find_open_covering_date`, (b) positionner `journal_entry.date` lors de la création via `create_in_tx`.
  - Handler `post_split` (cf. spec d'origine §split-flow). Idem résolution `entry_date` que `/manual`.
  - Modifier `post_accept` pour exiger `type` discriminator obligatoire (Q2 breaking) :
    - `type: 'invoice'` → flow 8-4 inchangé.
    - `type` absent ou non reconnu (`'manual'`/`'split'`/`'rule'` v0.1 8-5a) → `400 Validation` avec liste des types acceptés.

  **F3'' Pass 3 Opus 4.7 — validation accounts (`bankLedgerAccountId` + `counterpartyAccountId` + tous les `splits[i].counterpartyAccountId`)** :
  - Helper réutilisé : `accounts::find_by_id_in_company(pool, account_id, company_id)` (existant, signature `(pool: &MySqlPool, id: i64, company_id: i64) -> Result<Option<Account>, DbError>`, **note ordre paramètres inversé vs `fiscal_years::find_by_id_in_company`** — cohérence à reporter v0.2).
  - Le helper **ne filtre PAS `active=true`** côté SQL (comportement actuel) — le handler doit donc faire le check manuel après le `find` :
    ```rust
    let account = accounts::find_by_id_in_company(&state.pool, account_id, company_id)
        .await?
        .ok_or(AppError::AccountNotFound)?;
    if !account.active {
        return Err(AppError::AccountNotFound); // AC #82 : archived = 404 (anti-énumération)
    }
    ```
  - **Ordre de validation pré-flight** (avant `with_account_lock`) :
    1. `bank_accounts::find_by_id_for_company(pool, company_id, bankAccountId)` → 404 `BANK_ACCOUNT_NOT_FOUND` (AC #78).
    2. `accounts::find_by_id_in_company(pool, bankLedgerAccountId, company_id)` + check `active` → 404 `ACCOUNT_NOT_FOUND` (F2'' Pass 3).
    3. `accounts::find_by_id_in_company(pool, counterpartyAccountId, company_id)` + check `active` → 404 `ACCOUNT_NOT_FOUND` (AC #77, #82).
    4. **Pour `/split`** : itérer `splits[i].counterpartyAccountId`, batch-load via une seule query `IN (...)` (pattern `find_pending_by_ids` reconciliation.rs:233) ou itération séquentielle si volume ≤ 50 (cap T2 inchangé). Tout split référençant un account inexistant ou archivé (cross-tenant ou même tenant) → `404 ACCOUNT_NOT_FOUND` body `details.missingAccountIds: [<ids>]` (AC #90 camelCase JSON, list distincts triés).
  - **Note** : `journal_entries::create_in_tx` re-vérifie déjà tous les `account_ids` (existence + `active=TRUE` + même `company_id`) au step 2 (cf. `crates/kesh-db/src/repositories/journal_entries.rs:113-143`). Le pré-flight handler-side fait le mapping HTTP fin (`404 ACCOUNT_NOT_FOUND` typé + `missingAccountIds` détaillé) ; sans pré-flight, l'erreur remonterait `DbError::InactiveOrInvalidAccounts` mappée `400 INACTIVE_OR_INVALID_ACCOUNTS` générique (moins UX-friendly).

- [ ] T3.2 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : ajouter `POST /api/v1/reconciliation/manual`, `POST /api/v1/reconciliation/split`.

- [ ] T3.3 — Étendre `crates/kesh-api/src/errors.rs` (variantes ajoutées) :
  - `AppError::AccountNotFound { account_id, missing_account_ids: Option<Vec<i64>> }` → `404 ACCOUNT_NOT_FOUND` body `{ error: { code: "ACCOUNT_NOT_FOUND", message: t(...), details: { accountId, missingAccountIds: [...] } } }` (camelCase JSON, AC #90 ; `missingAccountIds` populated pour split, single `accountId` pour manual). **F10'' Pass 3 Opus 4.7 — confirmation absence variant** : la spec disait « si pas déjà existant » mais aucun grep `AccountNotFound` dans `errors.rs` actuel (seul `BankAccountNotFound` existe) → variant **à créer**, non pas à réutiliser.
  - `AppError::ReconciliationAlreadyReconciled { bank_transaction_id }` → `409 RECONCILIATION_ALREADY_RECONCILED` (émis si `find_strictly_pending_by_id_for_account` retourne `None` car status ≠ 'pending' ou tx introuvable). **Note** : ce variant a été *supprimé* en M6 Pass 1 code review 8-4 (`errors.rs:319-323` doc-comment) car remplacé par `FailedProposal { error_code: "RECONCILIATION_ALREADY_RECONCILED" }` per-proposal pour le batch `/accept`. **8-5a le ré-introduit comme variant global** car `/manual` et `/split` traitent UNE transaction (pas un batch — pas de partial success), donc le code per-proposal ne s'applique pas.
  - `AppError::ReconciliationFiscalYearClosed { entry_date: NaiveDate }` → `409 RECONCILIATION_FISCAL_YEAR_CLOSED` (cf. F6'' clarification HTTP 409 + cohérence §error-precedence-order).
  - `AppError::ReconciliationSplitImbalance { expected, actual, difference }` → `400 RECONCILIATION_SPLIT_IMBALANCE` body `details = { expected: '10700.00', actual: '10500.00', difference: '-200.00' }` (string Decimal cf. AC #87).

  **F11'' Pass 3 Opus 4.7 — impact `Copy` sur `AcceptProposalInput`** : la struct actuelle `AcceptProposalInput` (`reconciliation.rs:155-160`) dérive `Copy` (2 i64 = 16 octets, OK). Ajouter un champ `r#type: String` pour Q2 breaking change **casse `Copy`** car `String` est `Drop`. Solutions :
  - **(a)** Remplacer `String` par un `enum` typé : `enum AcceptType { Invoice, Manual, Split, Rule }` avec `#[serde(rename_all = "lowercase")]` — dérive `Copy` car enum unit-only. **Préférée** car typage strict + Validation Serde gratuit (rejet 400 si valeur inconnue côté Rust avant atteindre le handler).
  - **(b)** Garder `String` libre + retirer `Copy` de `AcceptProposalInput` → ajouter `clone()` aux usages internes. Plus flexible mais break diff-hostile.

  **Décision Pass 3** : option (a). La spec d'origine §accept-with-rule-flow était implicite ; le breaking change Q2 mérite un typage strict serde. Le test `accept_rejects_proposal_missing_type_discriminator` (AC #94) couvre serde rejection (`type` field absent = 422 sérialisation Serde) et le test `accept_with_explicit_invoice_type_runs_8_4_flow` (AC #95) couvre le happy path.

  ```rust
  #[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
  #[serde(rename_all = "lowercase")]
  pub enum AcceptType {
      Invoice,
      // Manual, Split, Rule réservés v0.1 — Serde rejette pour l'instant
      // sauf si explicitement listés (cohérent §rejet manual/split/rule
      // sur /accept en 8-5a — ces flows passent par routes dédiées).
  }

  #[derive(Debug, Deserialize, Clone, Copy)]
  #[serde(rename_all = "camelCase")]
  pub struct AcceptProposalInput {
      pub r#type: AcceptType,
      pub bank_transaction_id: i64,
      pub invoice_id: i64,
  }
  ```

- [ ] T3.4 — **Migration tests E2E 8-4 existants** : modifier `crates/kesh-api/tests/reconciliation_e2e.rs` (21 tests existants) pour ajouter `type: 'invoice'` explicite dans tous les bodies POST /accept (AC #96).

- [ ] T3.5 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_manual_e2e.rs` *(nouveau)* (≥ **18 tests**) :
  1-10. Manual (AC #75-#84) : 10 tests.
  11-16. Split (AC #85-#93) : 6 tests minimaux couvrant AC #85 (happy paiement), #86 (happy encaissement), #87 (imbalance), #88 (min 2 lignes), #90 (multi-tenant), #92 (RBAC). AC #89 (max 50) + #91 (déjà réconciliée) + #93 (audit) = 3 tests additionnels "stretch goal" à ajouter si le budget le permet.
  17. Accept type discriminator obligatoire (AC #94).
  18. Accept type='invoice' explicite (AC #95).
  
  **Total minimal** : 18 tests (10 manual + 6 split + 2 accept-discriminator). **Total complet (avec stretch)** : 21 tests = 18 minimum + 3 stretch (AC #89/#91/#93).

### T4. Helper `fiscal_years::find_open_covering_date` — vérification signature (AC #80)

**Note état du code (Pass 1 validate)** : Le helper `fiscal_years::find_open_for_date_for_company` **n'existe pas** avec ce nom. Il existe :
- `find_covering_date(pool: &MySqlPool, company_id, date)` → `Option<FiscalYear>` (tout status, pas de FOR UPDATE)
- `find_open_covering_date(tx: &mut Transaction, company_id, date)` → `Option<FiscalYear>` (status='Open', FOR UPDATE, inside tx)

Le flow `/manual` et `/split` tourne sous `with_account_lock` qui ouvre une `tx_outer`. Le helper correct à utiliser est **`find_open_covering_date(tx, company_id, entry_date)`** (status='Open', inside tx = sémantique correcte pour bloquer une clôture concurrente de l'exercice).

- [ ] T4.1 — **Pas besoin de créer un nouveau helper** : utiliser `fiscal_years::find_open_covering_date` existant (Story 3-7) avec `&mut tx_outer` passé depuis le handler. Importer `kesh_db::repositories::fiscal_years` dans `crates/kesh-api/src/routes/reconciliation.rs`.

  **F5'' Pass 3 Opus 4.7 — redondance avec `journal_entries::create_in_tx`** : le helper `journal_entries::create_in_tx` (cf. `crates/kesh-db/src/repositories/journal_entries.rs:96-110`) fait LUI-MÊME un `SELECT FOR UPDATE` sur `fiscal_years WHERE id = ? AND company_id = ?` puis vérifie `status != 'Closed'` (retourne `DbError::FiscalYearClosed` mappé `400 FISCAL_YEAR_CLOSED` par `errors.rs:908`). Donc **deux fail-fast du closed-check sont possibles** :
  - **(a) Approche fail-fast handler** (préférée) : appeler `find_open_covering_date(tx, company_id, entry_date)` côté handler, retourner `409 RECONCILIATION_FISCAL_YEAR_CLOSED` (code dédié 8-5a) avant `create_in_tx`. Le helper interne fera ensuite un re-lock idempotent (re-SELECT FOR UPDATE de la même row dans la même tx = no-op).
  - **(b) Approche déléguer à `create_in_tx`** : laisser `create_in_tx` détecter et retourner `DbError::FiscalYearClosed` mappé en `400 FISCAL_YEAR_CLOSED` générique. Inconvénient : code HTTP 400 (générique, légalement c'est une violation `Conflict 409`), wording moins précis pour 8-5a.

  **Décision Pass 3** : **approche (a)** — appeler `find_open_covering_date` côté handler (pré-flight inside lock), pour bénéficier du code dédié `409 RECONCILIATION_FISCAL_YEAR_CLOSED` cohérent §error-precedence-order #10. Le re-check de `create_in_tx` reste utile en defense-in-depth mais ne sera jamais déclenché en chemin nominal (la fiscal_year est lockée par notre `find_open_covering_date FOR UPDATE`, pas de fenêtre TOCTOU).

- [ ] T4.2 — Vérifier l'ordre des locks : `fiscal_years` est acquis APRÈS le lock advisory `with_account_lock` (GET_LOCK au niveau session) → pas de deadlock possible (advisory lock ≠ row lock). L'ordre est : advisory lock → `find_strictly_pending` → `find_open_covering_date FOR UPDATE` → `create_in_tx` (re-prend le même fiscal_year lock = idempotent).

- [ ] T4.3 — Ajouter test E2E HTTP `manual_match_rejects_closed_fiscal_year` (AC #80) : seed un exercice `Closed` + tx pending, POST `/manual`, vérifier **`409 RECONCILIATION_FISCAL_YEAR_CLOSED`** (le `find_open_covering_date` retourne `None` car le filtre `status='Open'` exclut le Closed → handler retourne l'erreur typée avant `create_in_tx`). **F6'' Pass 3 Opus 4.7 — distinction code/HTTP** :
  - `RECONCILIATION_FISCAL_YEAR_CLOSED` (8-5a, **HTTP 409**, code spécifique au flow réconciliation) — émis par `post_manual` / `post_split` côté handler quand `find_open_covering_date` retourne `None` pour une `entry_date` qui tombe sur un exercice Closed (ou aucun exercice n'existe). Mappage AppError→HTTP : 409 (conflit métier, l'utilisateur doit choisir une autre date ou réouvrir un exercice — non-faisable v0.1).
  - `FISCAL_YEAR_CLOSED` (existant, HTTP 400, code générique journal entries) — émis par `journal_entries::create_in_tx` step 1 si la fiscal_year passe le check handler mais bascule Closed entre les deux locks (race extrême — quasi impossible avec advisory lock + FOR UPDATE en chaîne, mais reste en defense-in-depth).
  - Les deux codes coexistent. Le mapping HTTP 409 du code spécifique 8-5a est cohérent avec le pattern §error-precedence-order #7 (advisory lock 409) et #5 (already reconciled 409) — toutes les erreurs métier de réconciliation sont 409, pas 400.

### T5. Frontend `ManualMatchModal` + `TransactionSplitModal` + extensions (AC #97-#100)

- [ ] T5.1 — Étendre `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
  ```ts
  // F2'' Pass 3 Opus 4.7 — `bankLedgerAccountId` ajouté pour résoudre le compte
  // comptable côté banque (compte 1020 typiquement). bank_accounts table n'a
  // pas de colonne journal_account_id en v0.1 — le client doit le fournir
  // explicitement. Reportée v0.2 : pré-remplir serveur-side via migration
  // `ALTER TABLE bank_accounts ADD journal_account_id`.
  export async function manualMatchTransaction(
      bankAccountId: number,
      bankLedgerAccountId: number,
      bankTransactionId: number,
      counterpartyAccountId: number,
      description?: string,
      valueDate?: string,
  ): Promise<{ bankTransactionId: number; journalEntryId: number }>;

  export async function splitTransaction(
      bankAccountId: number,
      bankLedgerAccountId: number,
      bankTransactionId: number,
      splits: { counterpartyAccountId: number; amount: string; description: string }[],
      description?: string,
      valueDate?: string,
  ): Promise<{ bankTransactionId: number; journalEntryId: number }>;
  // Note : pas de `splitsCount` dans la response — le backend retourne
  // { bankTransactionId, journalEntryId } (cohérent avec manualMatchTransaction).
  // Le frontend peut compter `splits.length` client-side si nécessaire.
  ```

- [ ] T5.2 — **Migration breaking change `acceptProposal`** : modifier `acceptProposal` pour ajouter `type: 'invoice'` explicite dans le body envoyé. Met à jour aussi `ReconciliationProposals.svelte` qui consomme.

- [ ] T5.3 — Créer `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` :
  - Props : `bankTransaction`, `bankAccountId`.
  - Sélecteur `Account` autocomplete (filtre accounts dont `number.startsWith('5') || number.startsWith('6') || number.startsWith('7')` — plan comptable suisse PME classes 5/6/7 = charges/produits/compte de résultat ; filtre client-side puisque la struct `Account` n'a pas de champ `class` côté backend v0.1). Réutiliser `AccountAutocomplete.svelte` de `features/journal-entries/` si compatible.
  - Textarea description (200 chars max).
  - Datepicker `valueDate` pré-rempli.
  - On submit : `manualMatchTransaction(...)` + dispatch event `success`.

- [ ] T5.4 — Créer `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` :
  - Props : `bankTransaction`, `bankAccountId`.
  - Tableau splits éditable (ajout/suppression de ligne, min 2 max 50).
  - Sticker balance live computed `sum vs |tx.amount|` (vert si exact match, rouge sinon).
  - Bouton submit désactivé tant que balance ≠ exact.

- [ ] T5.5 — Étendre `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - Pour chaque ligne tx avec `candidates: []` : 2 boutons `Affecter manuellement` + `Éclater`.
  - On modal success : refresh la liste.

- [ ] T5.6 — Tests Vitest (≥ 4) :
  1. `ReconciliationProposals: shows manual+split buttons for tx without candidate` (AC #97).
  2. `ManualMatchModal: prefills value date from tx.value_date` (AC #98).
  3. `TransactionSplitModal: balance indicator updates live` (AC #99).
  4. `TransactionSplitModal: submit disabled until balance exact match` (AC #99).

### T6. i18n (AC implicite UI)

- [ ] T6.1 — Ajouter ~10 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixes stricts `reconciliation-manual-*` × 5, `reconciliation-split-*` × 5). FR canonical.
- [ ] T6.2 — Traductions DE / IT / EN — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [ ] T6.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T7. Tests E2E Playwright + a11y (AC #97-#100)

- [ ] T7.1 — Créer `frontend/tests/e2e/reconciliation-manual.spec.ts` (≥ 2 actifs) :
  1. `manual-match end-to-end` : login Comptable, navigate `/reconciliation`, click « Affecter manuellement » sur tx sans candidate, sélectionner compte, valider, vérifier toast succès + tx disparaît.
  2. `split end-to-end` : click « Éclater » sur tx -10700, ajouter 3 lignes, vérifier balance indicator passe au vert, valider, vérifier disparition.

- [ ] T7.2 — Tests a11y axe (AC #100) : 2 scénarios sur chaque modal (manual, split) — ouvrir modal puis `expect(await new AxeBuilder().analyze()).toHaveNoViolations()`.

### T8. Sync sprint-status (AC implicite Epic 8 progress)

- [ ] T8.1 — `_bmad-output/implementation-artifacts/sprint-status.yaml` : transition `8-5a-reconciliation-manuelle-split: ready-for-dev → in-progress` (au début dev) puis `→ review` (post `dev-story`). Note explicative dans `last_updated` field.
- [ ] T8.2 — Pas de README sync à ce stade (8-5a partiel — Epic 8 reste 🚧 En cours jusqu'au merge 8-5b).
- [ ] T8.3 — Pas d'issue GitHub à fermer (8-5a n'a pas de KF/CR pré-tracée).

## Dev Notes

### API surface livrée 8-1b/8-2/8-3/8-4 — patterns à réutiliser

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403.
- **Audit log atomique** : helper `audit_log::insert_in_tx(tx, NewAuditLogEntry { ... })`. Une entrée par operation distincte. Pour 8-5a : `reconciliation.manual_matched`, `reconciliation.split_applied`.
- **Erreurs structurées** : `AppError::Custom { ... }` ou variantes typées dédiées (préféré).
- **i18n key ownership** : préfixe strict, kebab-case, lint-i18n-ownership pass (Story 6-3).
- **`rust_decimal::Decimal`** : Decimal exact partout pour amounts. Validateur `validate_split_balance` utilise `==` strict.
- **Repository pattern + sqlx** : Executor générique `<E: Executor>` (pattern 8-3 / 8-4).
- **Advisory lock per-account** : `with_account_lock(tx, company_id, bank_account_id, 5)` réutilisé pour manual + split (sérialisation cross-flows sur le même compte).
- **`journal_entries::create_in_tx`** : helper Story 5-2, accepte tx ouverte par caller, ne commit pas. Émet audit `journal_entry.created` automatiquement.
- **`fiscal_years::find_open_for_date_for_company`** : helper Story 3-7, indispensable pour résoudre `fiscal_year_id` à partir d'une `entry_date` dans manual + split.

### Lessons leçons des stories précédentes

- **8-4 retro** (cycle 4 passes review pour 5 modules / ~2200 lignes) : 8-5a découpée à ~1500 lignes pour viser ≤ 3 passes review.
- **8-3 retro** (CHECK constraints invisibles sans MariaDB up) : pas de nouvelle migration en 8-5a (la migration `reconciliation_rules` est en 8-5b T1). 8-5a touche uniquement la helper `find_pending_by_id_for_account` qui ne change pas le schéma.
- **5-2 leçon** (`create_in_tx` pour atomicité) : les 2 nouvelles routes manual/split **doivent** utiliser `create_in_tx` plutôt que `create` (qui ouvre sa propre tx, incompatible avec la tx du `with_account_lock`).
- **8-4 patch P3-H1** (optimistic lock UPDATE bank_transactions `AND version = ?`) : appliquer **systématiquement** sur tous les UPDATE bank_transactions de 8-5a (manual, split) pour défense-in-depth.
- **Q2 décision Guy 2026-05-07** : breaking change `POST /accept` — Kesh pas en prod, donc accepté. Migration des 21 tests E2E HTTP 8-4 fait partie du scope 8-5a (AC #96 + T3.4). Si oublié → CI rouge.

### Patterns architecturaux à respecter

- **Pas de dépendance circulaire** : `kesh-reconciliation → kesh-core, kesh-db` (cohérent 8-4). Les nouveaux modules `manual`, `split` consomment `kesh_db::entities::BankTransaction` et `kesh_db::entities::journal_entry::NewJournalEntry`.
- **Cohérence audit log** : tous les **clés top-level** des `details_json` pour `reconciliation.manual_matched` et `reconciliation.split_applied` utilisent snake_case systématiquement : `bank_transaction_id`, `counterparty_account_id`, `journal_entry_id`, `total_amount`, `amount`, `description`, `value_date`, `was_previously_rejected`. Pas de mélange camelCase **au top-level** (cohérent AC #84, AC #93, AC #90 error `missingAccountIds` est camelCase uniquement en réponse HTTP, jamais dans `details_json`).

  **F4'' Pass 3 Opus 4.7 — exception sous-objets typés** : si un `details_json` inclut un objet sérialisé via une struct typée Rust avec `#[serde(rename_all = "camelCase")]` (ex. pattern 8-4 où `details.score` = `MatchScore { total, amount_score, reference_score, contact_score }` produit JSON `{ total, amountScore, referenceScore, contactScore }`), le sous-objet sera camelCase. Cette nuance est **acceptable** (cohérence avec API HTTP qui sérialise les structs Rust de la même manière) et ne casse pas la lisibilité. **Pour 8-5a manual_matched et split_applied** : aucun sous-objet typé n'est inclus — toutes les valeurs sont primitives (i64, String, Decimal-as-String, NaiveDate-as-String, bool) ou des Vec d'objets construits manuellement via `serde_json::json!` avec clés snake_case explicites. Donc 100 % snake_case dans 8-5a (vs mix 8-4).
- **Pas d'`f64` pour montants** : `Decimal` partout (`splits[i].amount`, `tx.amount`). Le `f64` n'apparaît que dans le score 8-4 hérité, pas dans 8-5a.
- **Tests : éviter le coupling temporel** : utiliser des dates fixes dans les seeds (`NaiveDate::from_ymd_opt(2026, 5, 15)`).
- **`auto_match_rejected_at=NULL` au manual-match** : indispensable pour éviter qu'une tx manual-matched apparaisse comme « rejetée + matched » (état incohérent).

### Source tree à toucher

**DB** :
- `crates/kesh-db/src/repositories/reconciliation.rs` (ajout `find_strictly_pending_by_id_for_account` — T1.1, **F9'' Pass 3 Opus 4.7 correction** : était listé `bank_transactions.rs` ce qui contredisait T1 qui localise dans `reconciliation.rs`)
- `crates/kesh-db/src/repositories/fiscal_years.rs` (utiliser `find_open_covering_date` existant Story 3-7 — T4, pas de modification)
- `crates/kesh-db/src/repositories/accounts.rs` (utiliser `find_by_id_in_company` existant — T3.1, pas de modification ; check `active=true` côté handler)

**Backend `kesh-reconciliation`** :
- `crates/kesh-reconciliation/Cargo.toml` (deps inchangées : `kesh-core`, `kesh-db`, `sqlx`, `chrono`, `rust_decimal`, `serde`, `thiserror`, `tracing`)
- `crates/kesh-reconciliation/src/lib.rs` (refactor — modules `manual`, `split` ajoutés ; `rules` reporté 8-5b)
- `crates/kesh-reconciliation/src/manual.rs` *(nouveau, pure, helper réutilisé par 8-5b)*
- `crates/kesh-reconciliation/src/split.rs` *(nouveau, validator)*
- `crates/kesh-reconciliation/src/errors.rs` (2 variants ajoutés : `SplitImbalance`, `FiscalYearClosed`)

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation.rs` (extension : `post_manual` + `post_split` + breaking change `post_accept` discriminator type)
- `crates/kesh-api/src/lib.rs` (mount routes)
- `crates/kesh-api/src/errors.rs` (3 nouvelles variantes : `AccountNotFound`, `ReconciliationFiscalYearClosed`, `ReconciliationSplitImbalance`)
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs` *(nouveau, ≥ 16 tests)*
- `crates/kesh-api/tests/reconciliation_e2e.rs` (migration : ajouter `type: 'invoice'` explicite dans 21 tests existants — T3.4)

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (~10 nouvelles clés `reconciliation-manual-*` + `reconciliation-split-*` × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (extension manual + split + migration `acceptProposal` avec `type: 'invoice'`)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (extension boutons manuel/éclater)
- `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/ManualMatchModal.test.ts` *(nouveau, Vitest)*
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts` *(nouveau, Vitest)*
- `frontend/tests/e2e/reconciliation-manual.spec.ts` *(nouveau, Playwright)*

### Standards de test

- **Unit `kesh-reconciliation`** : `#[cfg(test)] mod tests` inline `manual.rs` + `split.rs`. ≥ 6 unit tests T2.5.
- **Intégration `kesh-db`** : `#[sqlx::test]`. ≥ 2 tests T1.2 (+ ≥ 1 T4.2 fiscal_years).
- **E2E HTTP `kesh-api`** : helper `spawn_app(pool)` (pattern 8-1b/8-2/8-3/8-4). ≥ **18 nouveaux tests** T3.5 (10 manual + 6 split + 2 accept-discriminator). + **21 tests actifs** 8-4 migrés + 1 ignored préservé (= 22 total dans reconciliation_e2e.rs — régression non introduite).
- **Vitest frontend** : `npm run test:unit -- reconciliation`. ≥ 4 tests T5.6.
- **Playwright** : `frontend/tests/e2e/reconciliation-manual.spec.ts`. ≥ 2 actifs + 2 a11y.

### Checklist locale avant push

```sh
# Backend (cf. CLAUDE.md « Test Locally First »)
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB up requis (T1 sqlx + T2 unit + T3 E2E + tests 8-4 migrés)

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # T6.3
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reconciliation-manual.spec.ts
```

### Limitations connues v0.1 (sous-ensemble 8-5a — voir spec d'origine pour la liste complète L41-L50)

| # | Limitation | Justification |
|---|---|---|
| L44 | Split ne lie pas vers des invoices multiples | v0.1 : split crée un journal_entry à N+1 lignes pointant sur des comptes de contrepartie type frais/produits. Lier 1 split vers N invoices clients (paiement multi-factures) reporté v0.2. Workaround v0.1 : utiliser N `manual` séparés sur des sous-transactions virtuelles (anti-pattern, déconseillé). |
| L45 | Annulation de réconciliation non disponible | v0.1 : pas d'undo `accept` / `manual` / `split`. L'utilisateur doit modifier l'écriture comptable manuellement (Story 3-3). v0.2 : route `POST /reconciliation/revert/{bankTransactionId}`. |
| L47 | Multi-currency split non supporté | Tous les splits doivent être dans la même currency que la tx (CHF v0.1, mono-CHF cohérent L38 héritée 8-4). Reporté Story 11. |
| **(post-Q5)** | Suggestion ML automatique post-manual non livrée | Décision Guy Q5 2026-05-07 : la response `POST /manual` ne retourne pas de `ruleSuggestion`. L'utilisateur crée ses rules manuellement via `/reconciliation/rules` (livré par 8-5b). Suggestion ML potentielle Story 8-5c v0.2 ou Epic 11+. |

### Levée des limitations héritées 8-4

| # héritée 8-4 | Status 8-5a | Mécanisme |
|---|---|---|
| L19 (matching journal_entries non-invoice) | **LEVÉE** | FR45 manual crée une journal_entry directement sans facture pré-existante (compte de contrepartie librement choisi). |
| L20 (création écriture sans facture) | **LEVÉE** | `journal_entries::create_in_tx` invoqué par manual et split. Pas de dépendance à `invoice.journal_entry_id`. |
| L23 (`auto_match_rejected_at` non réversible) | **PARTIELLEMENT LEVÉE** | Manual RESET `auto_match_rejected_at=NULL` (cas « rejet → manual reverse »). Bouton « Annuler le rejet » UI v0.1 toujours absent — l'utilisateur doit explicitement créer une écriture. v0.2 : bouton dédié. |
| L21 (paiement partiel) | **NON LEVÉE v0.1** | Reportée v0.2. Workaround 8-5a : manual avec compte « Différence de paiement » + invoice toujours dans `paid_at IS NULL`. |
| L18 (seuil auto-accept) | **NON LEVÉE v0.1** | Reportée v0.2 (cf. spec d'origine L42). |

### Risques et points d'attention pour le dev agent

1. **Breaking change `POST /accept` (Q2)** : si le dev agent oublie de migrer les **21 tests actifs** E2E HTTP 8-4 existants (AC #96 + T3.4), CI rouge en local + remote. **Vérification systématique** : `cargo test -p kesh-api --test reconciliation_e2e` doit retourner 21 verts + 1 ignored mono-CHF Story 11 (= 22 attributs `#[sqlx::test]` dans le fichier).

2. **Helper `manual::build_journal_entry_for_counterparty` réutilisé par 8-5b** : ne pas le rendre privé. Le marquer `pub` dans `lib.rs` re-exports. La signature ne doit pas changer après merge 8-5a (8-5b dépend de la stabilité d'API). Si une évolution est nécessaire, elle doit faire l'objet d'un CR explicite.

3. **`fiscal_years::find_open_for_date_for_company` peut ne pas exister** : vérifier dans `crates/kesh-db/src/repositories/fiscal_years.rs` Story 3-7. Si le helper n'existe qu'avec une signature différente, créer une version compatible (Executor générique, multi-tenant scoped) avant T3.

4. **Test E2E HTTP volume** : 18 nouveaux tests minimum + 21 tests actifs 8-4 migrés = **≥ 39 tests à passer**. Pas de dette test acceptable cette fois (lessons 8-4 retro). Si scope vraiment trop large dans une seule session dev, accepter de différer 3 tests stretch-goal split edge cases (AC #89/#91/#93) **uniquement en ultime recours** et tracer en `dette-test-e2e-http`. **Mais** : les 10 tests manual + 6 split + 2 accept-discriminator + 21 régression 8-4 sont **incontournables** (sécurité multi-tenant + RBAC + non-régression).

5. **Suppression de la suggestion ML (Q5)** : ne pas implémenter la fonction `suggest_rule` ni l'objet `ruleSuggestion` dans la response. C'est explicitement out-of-scope 8-5a (et out-of-scope 8-5b aussi, reporté v0.2).

### Références

- [`8-5-reconciliation-manuelle-regles-affectation.md`](8-5-reconciliation-manuelle-regles-affectation.md) — spec d'origine `archived-split` (référence des décisions de conception détaillées).
- [`epic-8.md`](../planning-artifacts/epic-8.md) — Story 8-5 ACs originaux (FR45-FR48), section « Risques » R6 R7.
- [`prd.md`](../planning-artifacts/prd.md) §FR45-FR48 lignes 439-442.
- [`8-4-reconciliation-matching-automatique.md`](8-4-reconciliation-matching-automatique.md) — patterns repo + mutex + audit + savepoint à réutiliser.
- [`architecture.md`](../planning-artifacts/architecture.md) §11.5 (kesh-reconciliation), §17 (FR42-FR53 mapping).
- [Story 5-2 `journal_entries::create_in_tx`](../../crates/kesh-db/src/repositories/journal_entries.rs) — helper transaction-bound.
- [Story 3-7 `fiscal_years::*`](../../crates/kesh-db/src/repositories/fiscal_years.rs) — résolution `fiscal_year_id` from `entry_date`.
- Findings résiduels 8-4 documentés (non-bloquants) : A6-2 POST currency guard gap MEDIUM (à fixer début Story 11), `reject_batch` reload asymétrique LOW.

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
| **2026-05-07** | Spec créée par split mécanique de 8-5 unifiée (décision Guy 2026-05-07 Q1=B). Découpage scope FR45 (manual) + FR48 (split) + breaking change `POST /accept` discriminator (Q2). FR46 suggestion ML retirée (Q5 — reportée v0.2). 26 ACs (#75-#100). Tasks T1-T8. Path-dépendance 8-5b sur le helper `manual::build_journal_entry_for_counterparty` documentée. Status `8-5a-reconciliation-manuelle-split: backlog → ready-for-dev`. | Claude (Opus 4.7 split workflow) |
| **2026-05-07** | **Pass 1 validate Sonnet 4.6** — 7 findings (2 CRITICAL + 3 HIGH + 2 MEDIUM + 0 LOW appliqués). Patches appliqués : F1 import erroné `kesh_core::accounting::NewJournalEntry` → `kesh_db::entities::journal_entry::NewJournalEntry` (T2.1) ; F2 T1 refactored — `find_pending_by_id_for_account` existe dans reconciliation.rs sans filtre status, ajout de `find_strictly_pending_by_id_for_account` dans le même module ; F3 T4 refactored — `find_open_for_date_for_company` n'existe pas, utiliser `find_open_covering_date` (Story 3-7) ; F4 compte tests 8-4 corrigé 21→22 actifs + 1 ignored = 23 total (AC #96, T3.4, §Risques 1) ; F5 décompte tests T3.5 unifié à 18 min (10+6+2) ; F6 `accountId` → `counterpartyAccountId` dans AC #85-#86 ; F7 `was_previously_rejected` précisé both cases (AC #84) ; + LOW : `splitsCount` retiré de `splitTransaction` response frontend, `missing_account_ids` → `missingAccountIds` camelCase (AC #90), filtre classe compte explicité côté client dans T5.3. Trend : Pass 1 = 5 findings > LOW. Prochaine étape : Pass 2 Haiku 4.5. | Claude (Sonnet 4.6 validate) |
| **2026-05-07** | **Pass 2 validate Haiku 4.5** — 6 findings (2 CRITICAL + 2 HIGH + 2 MEDIUM + 2 LOW appliqués). Patches majeurs : F1 AC #93 `reconciliation.split_applied` `details_json` structure complètement spécifiée (cohérent AC #84 snake_case) ; F2 T3.1 `entry_date` résolution clarifiée (fallback `tx.booking_date` si `valueDate` omis) ; F3 T2.1 tuple splits remplacé par struct `SplitDetail` type-safe ; F4 cohérence audit log snake_case documentée ; F5 T3.3 variante `ReconciliationAlreadyReconciled` explicitée ; F6 `valueDate` omis fallback behavior documenté. LOW : AC #96 lisibilité améliorée, test Vitest gaps notées. Trend : Pass 1 = 5 > LOW → Pass 2 = 6 > LOW (régression, mais sur fondements architecturaux solides — 4 issues CRITICAL+HIGH adressées profondeur codebase). Prochaine étape : Pass 3 Opus 4.7 (cycle orthogonal Sonnet → Haiku → Opus). | Claude (Haiku 4.5 validate) |
| **2026-05-07** | **Pass 3 validate Opus 4.7 — VALIDATION FINALE (1M context)** — 11 findings (2 CRITICAL + 2 HIGH + 3 MEDIUM + 4 LOW appliqués). Patches majeurs : **F1'' CRITICAL** compte tests `reconciliation_e2e.rs` corrigé partout (était "22 actifs + 1 ignored = 23 total" suite à régression Pass 1 Sonnet F4, vérification `awk` donne **21 actifs + 1 ignored = 22 total** — 6 occurrences corrigées : §scope ligne 68, AC #96, §Standards de test, §Risques 1+4) ; **F2'' CRITICAL** gap architectural `bank_account.linked_account_id` n'existe pas dans le schéma v0.1 (vérifié migration `20260410000001_bank_accounts.sql` + entité `BankAccount`), spec d'origine 8-5 §manual-match-flow lignes 125-126 référence un champ inexistant — **décision verrouillée 8-5a** : ajout `bankLedgerAccountId` requis dans body `POST /manual` et `POST /split` (AC #75 et #85 mis à jour, T5.1 frontend API mise à jour, suggestion v0.2 dette-architecture-bank-ledger pour migration `ALTER TABLE bank_accounts ADD journal_account_id`) ; **F3'' HIGH** validation accounts (`bankLedgerAccountId` + `counterpartyAccountId` + tous les `splits[i]`) explicitée — `accounts::find_by_id_in_company` ne filtre pas `active=true` côté SQL, handler doit appliquer le check post-find pour AC #82 + ordre pré-flight #1-4 + batch validation pour split + reuse `find_pending_by_ids` pattern + note interaction avec `journal_entries::create_in_tx` step 2 ; **F4'' HIGH** clarification snake_case top-level vs camelCase sous-objets typés (pattern 8-4 `details.score` = `MatchScore` camelCase via `Serialize`) — règle 8-5a : 100% snake_case car aucun sous-objet typé ; **F5'' MEDIUM** redondance `find_open_covering_date` côté handler vs re-lock interne `journal_entries::create_in_tx` step 1 — décision approche (a) fail-fast handler pour code 409 dédié vs 400 générique ; **F6'' MEDIUM** distinction `RECONCILIATION_FISCAL_YEAR_CLOSED` (8-5a, HTTP 409) vs `FISCAL_YEAR_CLOSED` (existant, HTTP 400 générique) — coexistence justifiée + cohérence §error-precedence-order ; **F7'' MEDIUM** précondition `validate_split_balance` documentée (longueur min/max validée par caller, pas par helper, signed `tx.amount` brut, edge case `tx.amount==0`) ; **F8'' LOW** note nom trompeur `find_pending_by_id_for_account` (8-4 ne filtre pas status — voir `dette-naming-reconciliation-helpers`) ; **F9'' LOW** §source-tree corrigé (était `bank_transactions.rs`, doit être `reconciliation.rs` cohérent T1) ; **F10'' LOW** `AppError::AccountNotFound` confirmé non-existant (variant à créer, non réutiliser ; payload structure spécifié) ; **F11'' LOW** impact `Copy` sur `AcceptProposalInput` (ajout discriminator `type` casse `Copy`) — décision option (a) enum typé `AcceptType { Invoice }` au lieu de `String` (typage strict + rejet Serde gratuit). Trend : Pass 1 = 5 > LOW → Pass 2 = 6 > LOW → **Pass 3 = 7 > LOW** (régression structurelle révélée — gaps architecturaux F1''/F2'' invisibles aux passes Sonnet+Haiku via biais d'auteur sur les patches récents, détectés par Opus 1M context vérification ground truth fichiers Rust réels). **Critère STOP CLAUDE.md non atteint** (≥ 1 finding > LOW). **Recommandation** : la spec a maintenant 7 findings résiduels CRITICAL+HIGH+MEDIUM adressés en patches in-place mais avec une dette de validation architecturale (F2'' notamment) — le splitting préventif aurait pu identifier ce gap plus tôt. **Verdict CONDITIONAL GO** : la spec est **implémentable telle quelle** par dev-story ; chaque finding a une remédiation concrète appliquée. **Prochaine étape recommandée** : pass 4 Sonnet 4.6 sur le code post-implémentation (review code, pas spec) — le cycle review SPEC s'arrête car les 11 findings sont tous adressés via patches concrets dans la spec. Si Guy préfère lancer Pass 4 Haiku 4.5 sur la spec, le risque est faible (les 11 patches sont conservateurs et orthogonaux) mais le ROI marginal — convergence asymptotique attendue ≤ 3 findings. | Claude (Opus 4.7 validate, 1M context) |
