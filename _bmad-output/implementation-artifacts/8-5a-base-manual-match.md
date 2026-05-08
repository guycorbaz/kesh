# Story 8-5a-base: FR45 manual match (réconciliation manuelle)

Status: backlog

<!-- Issue de re-split de Story 8-5a (`8-5a-reconciliation-manuelle-split.md`) le 2026-05-07,
     post-Pass-3 validate Opus 4.7 qui a détecté la dette F2'' (`bank_account.journal_account_id`
     inexistant — anti-pattern UX `bankLedgerAccountId` dans body POST /manual).

     Décision Guy 2026-05-07 (option 3) : éviter la dette à la racine en re-splittant 8-5a
     en 3 sous-stories séquentielles.

     Path-dépendance bloquante :
     - 8-5a-zero (`8-5a-zero-bank-account-journal-link.md`) DOIT être `done`/merged sur main
       avant que 8-5a-base ne transitionne `backlog → ready-for-dev`. 8-5a-base lit
       `bank_account.journal_account_id` (column ajoutée par 8-5a-zero) pour résoudre
       serveur-side le ledger account banque sans body field `bankLedgerAccountId`.

     Voir `8-5a-reconciliation-manuelle-split.md` (status `archived-split-bis`) pour les
     décisions de conception détaillées validées sur 3 passes Sonnet→Haiku→Opus
     (29 patches cumulés). Les sections §audit-log-shapes, §rbac, §error-precedence-order
     sont transposées ici sans modification (sauf §manual-match-flow step 5 où
     `bank_account.linked_account_id` → `bank_account.journal_account_id` automatique). -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse, comptable interne ou fiduciaire)**,
I want **réconcilier manuellement les transactions bancaires sans candidate auto-matchée en sélectionnant un compte de contrepartie (charges/produits classes 5/6/7), sans devoir re-saisir le compte comptable banque (résolu serveur-side via le `journalAccountId` configuré sur le `bank_account` en 8-5a-zero)**,
so that **mon backlog de transactions `pending` se résorbe (frais bancaires, salaires individuels, intérêts) sans devoir attendre un moteur de règles, en bénéficiant de la configuration une-fois-par-bank-account du compte ledger banque (cohérent UX pattern `default_*_account_id` Story 5-2)**.

### Contexte

**Story 8-5a-base = sous-story FR45 manual match du re-split 8-5a → 8-5a-zero / 8-5a-base / 8-5a-bis** (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7).

**Pourquoi 8-5a-base après 8-5a-zero** : 8-5a-base consomme `bank_account.journal_account_id` (column ajoutée par 8-5a-zero) pour résoudre le compte ledger banque côté serveur, **sans body field `bankLedgerAccountId`**. Cela élimine la dette F2'' qui rendait la spec 8-5a unifiée non-convergente sur 3 passes adversariales.

**8-5a-base livre la valeur utilisateur immédiate :**
- **FR45** — création manuelle de contrepartie pour transactions inconnues (frais bancaires, intérêts, salaires individuels). UX simplifié : pas de re-sélection du compte banque à chaque tx.
- **Levée des limitations héritées 8-4** : L19 (matching journal_entries non-invoice), L20 (création écriture sans facture), L23 partielle (`auto_match_rejected_at` réversibilisé via manual-match RESET à `NULL`).
- **Helper public `kesh-reconciliation::manual::build_journal_entry_for_counterparty`** réutilisé par 8-5a-bis (via le helper split qui composera des manual sub-builds) et 8-5b (rules engine — flow `accept-with-rule`).

**8-5a-base ne livre PAS** :
- Foundation column `journal_account_id` (déjà 8-5a-zero — pré-requis).
- FR48 split (8-5a-bis).
- Breaking change `POST /accept` discriminator type (8-5a-bis, Q2).
- Helper `kesh-reconciliation::split::*` (8-5a-bis).
- Rules engine (8-5b).
- Suggestion ML automatique post-manual-match (FR46 originale, **reportée v0.2** — décision Guy Q5 2026-05-07).

**Status sprint** : `8-5a-base-manual-match: backlog` au moment de la création (2026-05-07). Transition vers `ready-for-dev` après que 8-5a-zero ait clos son cycle review (0 findings > LOW + merged main).

**Pré-requis closed (au moment du démarrage 8-5a-base)** :
- ✅ **Story 8-5a-zero** — column `bank_account.journal_account_id`, repo `set_journal_account_id_for_company`, route PATCH `/api/v1/bank-accounts/{id}`, UI configuration (page `/bank-accounts`).
- ✅ Story 8-4 — `kesh-reconciliation` crate activé (matching, mutex, errors), routes `GET /proposals` + `POST /accept` + `POST /reject`, audit `reconciliation.accepted`/`reconciliation.rejected`, frontend `features/reconciliation/`, schema `bank_transactions { status, matched_entry_id, auto_match_rejected_at }`.
- ✅ Story 6-2 — multi-tenant scoping pattern KF-002 Pattern 1.
- ✅ Story 5-2 — `journal_entries::create_in_tx` (helper transaction-bound, indispensable pour créer une écriture comptable atomiquement avec le UPDATE bank_transactions du flow accept manuel).
- ✅ Stories 4-1, 3-1 — entités `Contact`, `Account` chargées par les selectors UI.
- ✅ Story 3-7 — `fiscal_years::find_open_covering_date` (résolution `fiscal_year_id` à partir d'une `entry_date`).

**Crate cible** : extension de `kesh-reconciliation` avec 1 nouveau module `manual` (helper public `build_journal_entry_for_counterparty`). Le module `split` est livré par 8-5a-bis. Le module `rules` est livré par 8-5b.

### Scope verrouillé — ce qui est livré par 8-5a-base

1. **Création manuelle de contrepartie (FR45)** — nouvelle route `POST /api/v1/reconciliation/manual` (sub-router `comptable_routes`).

   **Body simplifié post-8-5a-zero** :
   ```json
   {
     "bankAccountId": 17,
     "bankTransactionId": 42,
     "counterpartyAccountId": 6810,
     "description": "Frais TWINT mai",
     "valueDate": "2026-05-15"
   }
   ```
   **Pas de `bankLedgerAccountId`** — résolu serveur-side via `bank_accounts::find_by_id_for_company(...).journal_account_id`. Si NULL → 412 `BANK_ACCOUNT_NOT_CONFIGURED` (le user doit configurer en 8-5a-zero avant d'utiliser 8-5a-base).

   **Flow** :
   - Pré-flight ownership : `bank_accounts::find_by_id_for_company(pool, company_id, bankAccountId)` → 404 (variant unit-struct `AppError::BankAccountNotFound`, code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` v0.1 — dette de naming héritée 8-1b documentée `errors.rs:248-253`, F7'''' Pass 6 Opus clarification numéro de référence) si None.
   - Vérification configuration : `bank_account.journal_account_id` doit être `Some(journal_account_id)`. Si `None` → 412 `BANK_ACCOUNT_NOT_CONFIGURED` body `{ error: { code: "BANK_ACCOUNT_NOT_CONFIGURED", message: t(...), details: { bankAccountId, hint: "Configurer le compte comptable lié via /bank-accounts" } } }`.
   - Validation `counterpartyAccountId` : `accounts::find_by_id_in_company` + check `active=true` → 404 `ACCOUNT_NOT_FOUND` (anti-énumération, AC #86 hérité).
   - **Note** : pas de check `account_type` strict pour `counterpartyAccountId` (peut être Asset/Liability/Revenue/Expense — flexible v0.1, le user choisit). Frontend filtre client-side classes 5/6/7 par UX, pas d'invariant serveur.
   - Charge tx via `find_strictly_pending_by_id_for_account` (helper créé en 8-5a-base T1) → 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` si None (status ≠ pending ou cross-account/cross-tenant).
   - Acquérir `with_account_lock(tx_outer, company_id, bank_account_id, 5)` (advisory lock 8-4 réutilisé).
   - Inside lock : re-fetch tx (TOCTOU defense), re-vérifier status='pending'.
   - Résoudre `entry_date` = `valueDate ?? tx.booking_date`.
   - `fiscal_years::find_open_covering_date(tx, company_id, entry_date)` → 409 `RECONCILIATION_FISCAL_YEAR_CLOSED` si None.
   - Construire `NewJournalEntry` via `manual::build_journal_entry_for_counterparty(...)` (helper pure §helper-signature ci-dessous).
   - `journal_entries::create_in_tx(tx, fiscal_year_id, user_id, new_je)` → atomicité (émet audit `journal_entry.created` automatiquement). **Signature réelle** : `(tx: &mut Transaction<'_, MySql>, fiscal_year_id: i64, user_id: i64, new: NewJournalEntry)` — le `company_id` est dans `new.company_id`, pas un paramètre séparé.
   - UPDATE `bank_transactions SET status='reconciled', matched_entry_id=<new_je_id>, auto_match_rejected_at=NULL, updated_at=NOW(3), version=version+1 WHERE id=? AND company_id=? AND status='pending' AND version=?` (optimistic lock cohérent P3-H1 8-4 + defense-in-depth multi-tenant + status guard, cf. F3'''' Pass 6 Opus §validation-handler-side step 8 pour la SQL complète).
   - Audit log `reconciliation.manual_matched` action distincte (Q4a, cohérent §audit-log-shapes 8-5a unifiée).
   - Response : `200 OK` body `{ bankTransactionId: 42, journalEntryId: 999 }`.

2. **Helper `kesh-reconciliation::manual::build_journal_entry_for_counterparty`** (signature stable — réutilisé par 8-5a-bis et 8-5b) :
   ```rust
   /// Construit une `NewJournalEntry` à 2 lignes pour réconciliation manuelle.
   /// Pure (zéro I/O). Sign-aware : sign de `tx.amount` détermine côté débit/crédit.
   /// **Helper public, signature stable contractée pour 8-5a-bis et 8-5b**.
   ///
   /// Inputs :
   /// - `tx` : la `BankTransaction` à matcher (status='pending').
   /// - `bank_account_journal_id` : le compte comptable banque résolu via
   ///   `bank_account.journal_account_id` (foundation 8-5a-zero).
   /// - `counterparty_account_id` : le compte de contrepartie choisi par l'utilisateur.
   /// - `description` : description de la journal_entry (200 chars max, validation handler).
   /// - `entry_date` : `valueDate ?? tx.booking_date` (résolu côté handler).
   ///
   /// Output : `NewJournalEntry` avec 2 lignes :
   /// - Ligne 1 (banque) : `bank_account_journal_id`, débit ou crédit selon sign(tx.amount).
   /// - Ligne 2 (contrepartie) : `counterparty_account_id`, opposé sign.
   pub fn build_journal_entry_for_counterparty(
       tx: &BankTransaction,
       bank_account_journal_id: i64,
       counterparty_account_id: i64,
       description: String,
       entry_date: NaiveDate,
   ) -> NewJournalEntry
   ```

3. **Helper repo `find_strictly_pending_by_id_for_account`** (extension `kesh-db::repositories::reconciliation`) :
   - Filtre explicite `status = 'pending'` (vs `find_pending_by_id_for_account` 8-4 qui ne filtre pas — F8'' Pass 3).
   - Multi-tenant scoped `(company_id, bank_account_id, id)`.
   - Utilisé par `/manual` (8-5a-base) et `/split` (8-5a-bis).

4. **Audit log action `reconciliation.manual_matched`** (Q4a — action distincte, cohérent décision Guy 2026-05-07) :
   ```json
   {
     "bank_transaction_id": 42,
     "counterparty_account_id": 6810,
     "journal_entry_id": 999,
     "amount": "-150.00",
     "description": "Frais TWINT mai",
     "value_date": "2026-05-15",
     "was_previously_rejected": false
   }
   ```
   `was_previously_rejected = true` si tx avait `auto_match_rejected_at != NULL` avant le match (cas reverse rejet auto). 100% snake_case top-level (cohérent F4'' Pass 3 décision).

5. **Levée explicite des limitations héritées 8-4** :
   - **L19** (matching journal_entries non-invoice) → **LEVÉE** : FR45 manual crée une journal_entry directement sans facture pré-existante.
   - **L20** (création écriture sans facture) → **LEVÉE** : `journal_entries::create_in_tx` invoqué par manual sans dépendance à `invoice.journal_entry_id`.
   - **L23** (`auto_match_rejected_at` non réversible) → **PARTIELLEMENT LEVÉE** : Manual RESET `auto_match_rejected_at=NULL`. Bouton « Annuler le rejet » UI v0.1 toujours absent — l'utilisateur doit créer une écriture explicite.

6. **Frontend extensions** :
   - Composant `ManualMatchModal.svelte` (nouveau) : sélecteur `Account` (autocomplete plan comptable, filtré client-side `account.number.startsWith('5') || ...startsWith('6') || ...startsWith('7')`) + textarea description (200 chars max) + datepicker valueDate (pré-rempli `tx.value_date ?? tx.booking_date`). **Pas de dropdown ledger account banque** — résolu serveur-side.
   - Extension de `ReconciliationProposals.svelte` (héritée 8-4) : 1 bouton supplémentaire par ligne tx sans candidate : « Affecter manuellement » (ouvre `ManualMatchModal`). **Pas encore de bouton « Éclater »** — c'est 8-5a-bis.

7. **API client frontend** : nouvelle fonction `manualMatchTransaction` dans `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
   ```ts
   export async function manualMatchTransaction(
       bankAccountId: number,
       bankTransactionId: number,
       counterpartyAccountId: number,
       description?: string,
       valueDate?: string,
   ): Promise<{ bankTransactionId: number; journalEntryId: number }>;
   ```
   **Pas de `bankLedgerAccountId`** — résolu serveur-side. Différence majeure vs spec 8-5a unifiée.

8. **i18n** : ~5 nouvelles clés `reconciliation-manual-*` × 4 locales fr/de/it/en-CH. **Pas** les clés `reconciliation-split-*` (8-5a-bis) ni `reconciliation-rules-*` (8-5b).

9. **Tests** :
   - Unit `kesh-reconciliation::manual` (≥ 2 cas : sign-aware build_je for credit + debit).
   - Integration `kesh-db::repositories::reconciliation::find_strictly_pending_by_id_for_account` (≥ 2 sqlx multi-tenant + status filter).
   - E2E HTTP `kesh-api` (≥ 10 tests : AC #83-#92).
   - Vitest (≥ 3-4 : modal + button + api).
   - Playwright (≥ 1 actif + 1 a11y).

10. **Sync** sprint-status — pas de KF/CR pré-tracée.

**HORS scope 8-5a-base (→ 8-5a-bis / 8-5b / v0.2) :**

- Route `POST /reconciliation/split` (8-5a-bis).
- Helper `kesh-reconciliation::split::*` (8-5a-bis).
- Breaking change `POST /accept` discriminator type obligatoire (8-5a-bis, Q2).
- Migration des 21 tests E2E HTTP 8-4 actifs (8-5a-bis, Q2).
- Bouton « Éclater » dans `ReconciliationProposals.svelte` (8-5a-bis).
- Composant `TransactionSplitModal.svelte` (8-5a-bis).
- Table `reconciliation_rules` + CRUD + application (8-5b).
- Suggestion ML automatique post-manual-match (FR46 reportée v0.2 — Q5).
- Annulation de réconciliation (reportée v0.2, L45).

### Décisions de conception

#### §helper-signature

**Décision** : signature de `manual::build_journal_entry_for_counterparty` stable, contractée pour 8-5a-bis et 8-5b. Aucune évolution sans CR explicite après merge 8-5a-base.

```rust
pub fn build_journal_entry_for_counterparty(
    tx: &BankTransaction,                // tx.company_id et tx.amount sont consommés
    bank_account_journal_id: i64,        // résolu serveur-side via bank_account.journal_account_id
    counterparty_account_id: i64,
    description: String,
    entry_date: NaiveDate,
) -> NewJournalEntry
```

**Rationale** :
- **Pure** (zéro I/O) : tests unitaires faciles, pas de mock DB.
- **Sign-aware** : si `tx.amount < 0` (débit titulaire = sortie cash) → la ligne banque est en CRÉDIT (compte d'actif diminue) et la contrepartie en DÉBIT (charge augmente). Si `tx.amount > 0` (crédit titulaire = entrée cash) → inverse.
- **F4''' Pass 3 Opus — fields obligatoires `NewJournalEntry`** : la struct `NewJournalEntry` (cf. `kesh-db/src/entities/journal_entry.rs:166`) exige `company_id`, `entry_date`, `journal: Journal`, `description`, `lines: Vec<NewJournalEntryLine>`. Le helper :
  - `company_id` : récupéré depuis `tx.company_id`.
  - `journal` : hardcodé `Journal::Banque` (cohérent flow réconciliation ; toute opération bank_transaction tombe dans le journal Banque).
  - `entry_date`, `description` : passés en paramètres.
  - `lines` : 2 lignes calculées (sign-aware, banque + contrepartie).
- **F7''' Pass 3 Opus — précondition `tx.amount != 0`** : le helper assume `tx.amount != 0`. Si `tx.amount == 0`, la sign-aware logic est ambiguë (`abs(0) = 0` → 2 lignes débit/crédit 0/0 valides en partie double mais sémantiquement vides). Le handler **doit pré-valider** `tx.amount != Decimal::ZERO` avec 400 `VALIDATION_ERROR { reason: "zero_amount_transaction" }` (cf. T3.1 step 4bis). Le helper peut soit panic (`debug_assert`) soit accepter — choix : `debug_assert!(!tx.amount.is_zero())` pour catch en dev, accepter en prod (pas de UB).
- **Pas de SplitDetail** : c'est la signature simple à 2 lignes. Le helper N+1 lignes est dans le module `split` (8-5a-bis).

#### §validation-handler-side

**Ordre de validation pré-flight** (avant `with_account_lock`, cohérent §error-precedence-order de la spec 8-5a unifiée) :
1. `bankAccountId` cross-tenant : `bank_accounts::find_by_id_for_company(pool, company_id, bankAccountId)` → 404 `BANK_ACCOUNT_NOT_FOUND`.
2. **NOUVEAU 8-5a-base** : `bank_account.journal_account_id IS Some(...)` → 412 `BANK_ACCOUNT_NOT_CONFIGURED` sinon (ce check remplace la validation `bankLedgerAccountId` body de la spec 8-5a unifiée).
3. `counterpartyAccountId` cross-tenant + `active=true` : `accounts::find_by_id_in_company` + check → 404 `ACCOUNT_NOT_FOUND`.
4. `bankTransactionId` strictement pending + cross-tenant + cross-account : `find_strictly_pending_by_id_for_account` → 404 `RECONCILIATION_TRANSACTION_NOT_PENDING`.
4bis. **F7''' Pass 3 Opus — pré-validation `tx.amount != 0`** : si `tx.amount == Decimal::ZERO`, retourner 400 `AppError::Validation("zero_amount_transaction".into())` (cf. L48). Évite que le helper `build_journal_entry_for_counterparty` produise 2 lignes 0/0 sémantiquement vides.

**Inside lock (advisory `with_account_lock`)** :
5. Re-fetch tx (TOCTOU defense pattern 8-4).
6. Resolve `entry_date` + `find_open_covering_date(tx_inside_lock, company_id, entry_date)` → si `None`, retourner `Err(ReconciliationError::FiscalYearClosed { entry_date })` depuis la closure du lock. Le handler externe traduit en 409 `RECONCILIATION_FISCAL_YEAR_CLOSED`. **F5''' Pass 3 Opus — sémantique du `None`** : `find_open_covering_date` retourne `None` à la fois pour « aucun exercice n'existe pour cette date » (NoFiscalYear) et « l'exercice existe mais est `Closed` ». La spec mappe les deux en 409 `RECONCILIATION_FISCAL_YEAR_CLOSED` v0.1 (UX simplifié — utilisateur cible reçoit le même message « réconciliation impossible, vérifier l'exercice comptable »). Différenciation reportée v0.2 (cf. L46 ci-dessous) si UX granulaire requise.
7. `journal_entries::create_in_tx(tx_inside_lock, fiscal_year_id, user_id, new_je)`. **Note race** : si `create_in_tx` retourne `DbError::FiscalYearClosed` (exercice clos entre step 6 et step 7), ce cas remonte via la closure en `Err(ReconciliationError::Db(DbError::FiscalYearClosed))` (cf. T2.3 nouveau variant `Db(#[from] DbError)`) → handler match `Err(ReconciliationError::Db(db_err)) => Err(AppError::Database(db_err))` → mapping HTTP `AppError::Database(DbError::FiscalYearClosed)` → 400 `FISCAL_YEAR_CLOSED` (pas 409). Acceptable v0.1 — cas pathologique (clôture concurrent rarissime sous lock advisory). Documenter dans une note Dev si différenciation nécessaire v0.2.
8. UPDATE bank_transactions optimistic lock → 409 via `AppError::Database(DbError::OptimisticLockConflict)` si race (pattern 8-4 réutilisé — pas de variant `ReconciliationOptimisticLockConflict` dédié, `Database(OptimisticLockConflict)` → 409 `OPTIMISTIC_LOCK_CONFLICT` couvre ce cas). **F3'''' Pass 6 Opus — SQL UPDATE complète** (defense-in-depth multi-tenant + status guard, cohérent pattern 8-4 ligne 691) :
   ```sql
   UPDATE bank_transactions 
   SET status = 'reconciled', matched_entry_id = ?, auto_match_rejected_at = NULL, 
       updated_at = NOW(3), version = version + 1 
   WHERE id = ? AND company_id = ? AND status = 'pending' AND version = ?
   ```
   Si `rows_affected() == 0` → mapper en `Err(ReconciliationError::Db(DbError::OptimisticLockConflict))` (couvre race version + race status `pending → reconciled` par autre flow concurrent).
9. Audit log `reconciliation.manual_matched`. **F2'''' Pass 6 Opus — atomicité** : steps 5-9 sont **TOUS inside la closure unique de `with_account_lock`**. La closure retourne `Result<i64, ReconciliationError>` (le `i64` = `journal_entry_id` créé step 7). Si step 9 audit_log échoue, la closure retourne `Err(ReconciliationError::Db(...))` → `with_account_lock` propage → handler `drop(tx_outer)` ROLLBACK total (UPDATE bank_transactions step 8 inclus). **NE PAS sortir step 9 audit_log de la closure** sous prétexte de performance happy-path : casse l'invariant atomicity audit ↔ business write.

#### §audit-log-shape

100% snake_case top-level **dans 8-5a-base** (cohérent F4'' Pass 3 décision Opus). Le helper `build_journal_entry_for_counterparty` produit une `NewJournalEntry` à 2 lignes scalaires sans struct typée à serializer — donc tous les fields de `details_json` sont scalaires (i64, Decimal stringified, NaiveDate stringified, bool, String). **F6'''' Pass 6 Opus — démarcation vs 8-4** : 8-4 sérialise `"score": MatchScore { ... }` (struct typée serde camelCase via Serialize derive). 8-5a-base n'utilise pas de struct typée — la règle « pas de sous-objet camelCase » s'applique uniquement aux scalars de 8-5a-base, sans rétroaction sur 8-4 existant.

```json
{
  "bank_transaction_id": 42,
  "counterparty_account_id": 6810,
  "journal_entry_id": 999,
  "amount": "-150.00",
  "description": "Frais TWINT mai",
  "value_date": "2026-05-15",
  "was_previously_rejected": false
}
```

#### §rbac

Sub-router `comptable_routes` (Comptable+). `Consultation` retourne 403 (cohérent toutes mutations reconciliation 8-4).

#### §frontend-flow

**Modal `ManualMatchModal.svelte`** ouvert depuis `ReconciliationProposals.svelte` :
- Click bouton « Affecter manuellement » sur une ligne tx pending sans candidate auto-matchée.
- Modal fields :
  - Sélecteur `Account` autocomplete (filtre client-side classes 5/6/7 via `account.number.startsWith('5/6/7')`). Réutiliser `AccountAutocomplete.svelte` de `features/journal-entries/` si compatible.
  - Textarea description (200 chars max, placeholder « Frais bancaires mai »).
  - Datepicker `valueDate` pré-rempli (`tx.value_date ?? tx.booking_date`).
- Submit → `manualMatchTransaction(...)` → event `success` → refresh liste proposals.

**Pas de bouton « Éclater »** dans 8-5a-base. C'est ajouté en 8-5a-bis.

#### §helper-find-strictly-pending

**Décision** : ajouter `find_strictly_pending_by_id_for_account` dans `crates/kesh-db/src/repositories/reconciliation.rs` (pas dans `bank_transactions.rs`), avec filtre explicite `status = 'pending'`. Le helper existant 8-4 `find_pending_by_id_for_account` est conservé pour 8-4 (son absence de filtre status est intentionnelle pour `accept_one` step 4 qui vérifie le status séparément).

Rationale : **F8'' Pass 3 Opus** — le nom 8-4 est trompeur (ne filtre pas status). Le nouveau helper clarifie. Renommer le helper 8-4 est tracé en `dette-naming-reconciliation-helpers` (issue GitHub à créer post-merge si dette persistante, non-bloquant).

```rust
/// Charge une transaction **strictement** `pending` par id, scopée tenant + compte.
/// Utilisé par /manual (8-5a-base) et /split (8-5a-bis) pour pré-flight ownership
/// AVANT lock ET inside lock.
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

## Acceptance Criteria

ACs #83-#92 (10 ACs).

### Création manuelle de contrepartie (FR45)

83. **(FR45 — happy paiement débit)** Given une `bank_transaction` `pending` débit `-150.00 CHF` sur `bank_account_id=17` ET `bank_account.journal_account_id=1020` configuré (8-5a-zero) ET un compte `6810 Frais bancaires` actif, When `POST /api/v1/reconciliation/manual { bankAccountId: 17, bankTransactionId: 42, counterpartyAccountId: 6810, description: "Frais TWINT mai" }`, Then `200 OK` body `{ bankTransactionId: 42, journalEntryId: 999 }` ET `journal_entries` table contient 1 nouvelle entry à 2 lignes (1020 crédit 150.00 + 6810 débit 150.00) ET `bank_transactions.status='reconciled'`, `matched_entry_id=999`, `auto_match_rejected_at=NULL`. *Test E2E HTTP : `manual_match_creates_journal_entry_for_debit_transaction`.*

84. **(FR45 — happy encaissement crédit)** Given tx pending crédit `+200.00 CHF`, compte contrepartie `7510 Intérêts bancaires`, When manual-match, Then journal_entry à 2 lignes (1020 débit 200 + 7510 crédit 200). *Test E2E HTTP : `manual_match_creates_journal_entry_for_credit_transaction`.*

85. **(FR45 — bank_account non configuré 412)** Given un `bank_account.journal_account_id IS NULL` (cas user qui n'a pas configuré en 8-5a-zero), When POST `/manual`, Then `412 Precondition Failed` body `{ error: { code: "BANK_ACCOUNT_NOT_CONFIGURED", message: t(...), details: { bankAccountId: 17, hint: "Configurer le compte comptable lié via /bank-accounts" } } }`. *Test E2E HTTP : `manual_match_rejects_unconfigured_bank_account_with_412`.*

86. **(FR45 — multi-tenant safety counterparty)** Given user company_A POST manual avec `counterpartyAccountId` appartenant à company_B, Then `404 ACCOUNT_NOT_FOUND` (KF-002 pattern, pas 403). *Test E2E HTTP : `manual_match_does_not_leak_cross_tenant_account`.*

87. **(FR45 — multi-tenant safety bank_account)** Given user company_A POST manual avec `bankAccountId` appartenant à company_B, Then `404` body `error.code = "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND"` (dette naming L64 héritée 8-1b — code partagé v0.1 entre `bank-imports` et `bank-accounts` parce que `AppError::BankAccountNotFound` est un unit-struct unique). v0.2 : renommer en `BANK_ACCOUNT_NOT_FOUND`. *Test E2E HTTP : `manual_match_returns_404_on_cross_tenant_bank_account`.*

88. **(FR45 — already reconciled idempotency)** Given tx déjà `reconciled` (matched_entry_id != NULL), When POST manual, Then `404 RECONCILIATION_TRANSACTION_NOT_PENDING` (cohérent `find_strictly_pending_by_id_for_account` retourne None car status != 'pending' — pas 409 puisque le helper ne distingue pas les causes). *Test E2E HTTP : `manual_match_rejects_already_reconciled_transaction`.*

89. **(FR45 — fiscal year closed)** Given `entry_date` qui tombe dans un exercice fiscal `Closed`, When manual-match, Then `409 RECONCILIATION_FISCAL_YEAR_CLOSED`. *Test E2E HTTP : `manual_match_rejects_closed_fiscal_year`.*

90. **(FR45 — réversibilise rejet auto, lève L23)** Given tx `pending` avec `auto_match_rejected_at != NULL` (rejetée 8-4), When POST manual, Then `200 OK` ET tx update `status='reconciled'`, `auto_match_rejected_at=NULL`, audit `details.was_previously_rejected=true`. *Test E2E HTTP : `manual_match_reverses_auto_rejection`.*

91. **(FR45 — RBAC Comptable+ + audit log canonique)** Given user `Consultation`, When POST manual, Then `403 Forbidden`. ET Given POST manual happy par Comptable, When commit, Then audit_log contient 2 entrées : `(action='reconciliation.manual_matched', entity_type='bank_transaction', entity_id=42, details = { bank_transaction_id, counterparty_account_id, journal_entry_id, amount, description, value_date, was_previously_rejected })` ET `(action='journal_entry.created', entity_type='journal_entry', entity_id=999)` (émis par `journal_entries::create_in_tx`, héritage Story 3-2). *Tests E2E HTTP : `manual_match_requires_comptable_role` + `manual_match_emits_audit_log_pair`.*

### UI frontend extensions (manual seulement)

92. **(UI — bouton Affecter manuellement + modal a11y)** Given une ligne tx pending sans candidate sur `/reconciliation`, Then 1 bouton additionnel « Affecter manuellement » apparaît à droite de la ligne. ET Given click bouton, Then modal `ManualMatchModal` ouvre avec sélecteur Account (autocomplete plan comptable filtré classes 5/6/7) + textarea description (200 chars max) + datepicker valueDate (pré-rempli = tx.value_date ?? tx.booking_date). On submit success : event `success`, refresh liste. ET axe-core scan modal ouvert : 0 violation. *Tests Vitest : `ReconciliationProposals: shows manual button for tx without candidate` + `manual_match_modal_renders_with_prefilled_fields`. Test Playwright : `manual-match end-to-end + axe a11y`.*

## Tasks / Subtasks

### T1. Helper repo `find_strictly_pending_by_id_for_account` (AC #83-#88)

- [ ] T1.1 — Étendre `crates/kesh-db/src/repositories/reconciliation.rs` (cf. §helper-find-strictly-pending) :
  ```rust
  pub async fn find_strictly_pending_by_id_for_account<'e, E>(
      executor: E,
      company_id: i64,
      bank_account_id: i64,
      id: i64,
  ) -> Result<Option<BankTransaction>, DbError>
  where E: sqlx::Executor<'e, Database = MySql>,
  ```
  SQL : `WHERE company_id = ? AND bank_account_id = ? AND id = ? AND status = 'pending'`.

- [ ] T1.2 — Tests inline `#[sqlx::test]` (≥ 3, couverture exhaustive avant implémentation) :
  1. `find_strictly_pending_scopes_by_account_and_company` (cross-tenant returns None — sécurité multi-tenant).
  2. `find_strictly_pending_returns_none_for_reconciled_tx` (status='reconciled' returns None — filtre status précis).
  3. `find_strictly_pending_returns_tx_when_all_conditions_match` (happy path : returns tx si company_id/bank_account_id/id corrects + status='pending' — couverture complete du happy path).

- [ ] T1.3 — Vérifier `cargo test -p kesh-db reconciliation` MariaDB up local (lesson 8-3 retro).

### T2. Helper `kesh-reconciliation::manual::build_journal_entry_for_counterparty` (AC #83-#84)

- [ ] T2.1 — Créer `crates/kesh-reconciliation/src/manual.rs` :
  ```rust
  use kesh_db::entities::journal_entry::{NewJournalEntry, NewJournalEntryLine};
  use kesh_db::entities::journal_entry::Journal;
  use kesh_db::entities::bank_transaction::BankTransaction;
  use rust_decimal::Decimal;
  use chrono::NaiveDate;

  /// Construit une `NewJournalEntry` à 2 lignes pour réconciliation manuelle.
  /// Pure (zéro I/O). Sign-aware : sign de `tx.amount` détermine débit/crédit.
  /// **Helper public, signature stable contractée pour 8-5a-bis et 8-5b**.
  pub fn build_journal_entry_for_counterparty(
      tx: &BankTransaction,
      bank_account_journal_id: i64,
      counterparty_account_id: i64,
      description: String,
      entry_date: NaiveDate,
  ) -> NewJournalEntry { ... }
  ```

- [ ] T2.2 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod manual;
  pub use manual::build_journal_entry_for_counterparty;
  ```

- [ ] T2.3 — Étendre `crates/kesh-reconciliation/src/errors.rs` (2 variants ajoutés) :
  ```rust
  pub enum ReconciliationError {
      // ... 8-4 variants conservés (AccountLocked, LockReleaseFailed, Database(#[from] sqlx::Error))
      FiscalYearClosed { entry_date: NaiveDate },
      /// **F1'''' Pass 6 Opus** — wrapper typé pour `DbError` (variants
      /// `FiscalYearClosed`, `OptimisticLockConflict`, `InactiveOrInvalidAccounts`,
      /// `Invariant`, etc.). Ground-truth : `kesh-db::errors::DbError` est un
      /// enum distinct de `sqlx::Error` (pas `From<DbError> for sqlx::Error`).
      /// Sans ce variant, la closure `with_account_lock` (qui retourne
      /// `Result<T, ReconciliationError>`) ne peut PAS bubbler typé un
      /// `DbError::FiscalYearClosed` (race step 6 → step 7) ni un
      /// `DbError::OptimisticLockConflict` (UPDATE bank_transactions step 8).
      ///
      /// Le handler `post_manual` (kesh-api) match ce variant en
      /// `match lock_result { Err(ReconciliationError::Db(db_err)) => Err(AppError::Database(db_err)), ... }`
      /// — préserve la fidélité du `DbError` jusqu'au mapping HTTP final.
      ///
      /// **Path-dep** : réutilisé par 8-5a-bis (split flow) et 8-5b
      /// (accept-with-rule flow) pour la même raison.
      Db(#[from] kesh_db::errors::DbError),
      // (SplitImbalance reporté en 8-5a-bis)
      // (BankAccountNotConfigured handler-side dans kesh-api uniquement)
  }
  ```
  
  **Pourquoi ce variant `Db(DbError)` séparé de `Database(sqlx::Error)`** :
  - 8-4 conserve son flow inchangé (`Database(#[from] sqlx::Error)` reste la voie pour les erreurs sqlx brutes hors helpers `kesh-db`).
  - Les helpers `kesh-db` (qui retournent `Result<_, DbError>`) bubblent maintenant via `Db(#[from] DbError)` dans la closure. Le `?` opérator fonctionne en transparent : `journal_entries::create_in_tx(...)?` compile.
  - Le handler distingue `Database(sqlx)` → 500 catch-all et `Db(db_err)` → mapping fin via `AppError::Database(db_err)` (qui sait mapper `DbError::FiscalYearClosed` → 400 et `DbError::OptimisticLockConflict` → 409 etc.).
  
  **Non-régression 8-4** : 8-4 n'utilise PAS de helper `kesh-db` qui retourne `DbError` dans la closure `with_account_lock` (8-4 fait du SQL inline + `FailedProposal` per-proposal). Donc le nouveau variant `Db` n'impacte pas le flow 8-4 existant.

- [ ] T2.4 — Tests unit `kesh-reconciliation::manual` (≥ 2) :
  1. `manual_build_je_creates_2_lines_for_credit_tx` (AC #84).
  2. `manual_build_je_creates_2_lines_for_debit_tx` (AC #83).

### T3. Route API `POST /api/v1/reconciliation/manual` (AC #83-#91)

- [ ] T3.1 — Étendre `crates/kesh-api/src/routes/reconciliation.rs` (du 8-4) avec handler `post_manual` :
  - Validation Serde body `{ bankAccountId, bankTransactionId, counterpartyAccountId, description?, valueDate? }` camelCase.
  - **Note dev — description max length** : la spec et le frontend fixent 200 chars max pour le field `description`. Le code existant dans `routes/journal_entries.rs` utilise `MAX_DESCRIPTION_LEN = 500`. 8-5a-base utilise **200** chars (business rule modal, libellé court UX). Définir `const MAX_MANUAL_DESCRIPTION_LEN: usize = 200;` dans `routes/reconciliation.rs` et valider avec `AppError::Validation(format!("description trop longue (max {MAX_MANUAL_DESCRIPTION_LEN} caractères)"))`. Ne pas réutiliser la constante 500 de `journal_entries.rs` (limites distinctes).
  - Pré-flight ordre §validation-handler-side ci-dessus. **Important** : l'étape 4bis `tx.amount != 0` est une validation **PRÉ-flight** (avant le `with_account_lock`), à insérer immédiatement après la validation `counterpartyAccountId` étape 3 — cf. §validation-handler-side step 4bis pour le positionnement exact.
  - **Différence majeure vs spec 8-5a unifiée** : pas de `bankLedgerAccountId` body. Résolution serveur-side :
    ```rust
    // F1''' Pass 3 Opus : `AppError::BankAccountNotFound` est unit-struct
    // (pas `{ bank_account_id }`) — dette L64 documentée. Code HTTP réel
    // retourné = `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (cf. errors.rs ligne 255 +
    // tests bank_accounts_e2e.rs ligne 518). v0.2 : renommer en
    // `BANK_ACCOUNT_NOT_FOUND` (breaking client).
    let bank_account = bank_accounts::find_by_id_for_company(&state.pool, company_id, body.bank_account_id)
        .await?
        .ok_or(AppError::BankAccountNotFound)?;
    let journal_account_id = bank_account.journal_account_id
        .ok_or(AppError::BankAccountNotConfigured { bank_account_id: body.bank_account_id })?;
    ```
  - Inside lock : flow 5-9 §validation-handler-side. **Mapping `find_open_covering_date` None → `ReconciliationError::FiscalYearClosed`** : la closure passée à `with_account_lock` doit traduire le `None` retourné par `fiscal_years::find_open_covering_date` en `Err(ReconciliationError::FiscalYearClosed { entry_date })`. Sans cette traduction explicite, le `None` est un `Ok(None)` côté DB qui ne propage pas en erreur (F3''' Pass 3 Opus).

- [ ] T3.2 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : ajouter `.route("/api/v1/reconciliation/manual", post(routes::reconciliation::post_manual))`.

- [ ] T3.3 — Étendre `crates/kesh-api/src/errors.rs` (variantes ajoutées) :
  - `AppError::BankAccountNotConfigured { bank_account_id: i64 }` → 412 `BANK_ACCOUNT_NOT_CONFIGURED` body `{ error: { code, message, details: { bankAccountId, hint: "Configurer le compte comptable lié via /bank-accounts" } } }` camelCase.
  - `AppError::ReconciliationFiscalYearClosed { entry_date: NaiveDate }` → 409 `RECONCILIATION_FISCAL_YEAR_CLOSED` (cohérent §error-precedence-order #10 hérité 8-5a unifiée).
  - `AppError::ReconciliationTransactionNotPending { bank_transaction_id }` → 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` (helper `find_strictly_pending_by_id_for_account` retourne None — couvre cas pending/reconciled/cross-account/cross-tenant en un seul code).
  - `AppError::AccountNotFound { account_id: i64 }` → 404 `ACCOUNT_NOT_FOUND` (réutiliser variant 8-5a-zero T3.3 si déjà créé).
  - `AppError::BankAccountNotFound` (réutiliser variant 8-1b/8-5a-zero existant — **F1''' Pass 3 Opus : unit-struct, PAS `{ bank_account_id }`**, code HTTP réel `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` v0.1, dette L64).
  - `AppError::ReconciliationOptimisticLockConflict` **— NE PAS CRÉER** : le variant n'existe pas en 8-4. Le pattern 8-4 réutilise `AppError::Database(DbError::OptimisticLockConflict)` → 409 `OPTIMISTIC_LOCK_CONFLICT`. 8-5a-base doit faire de même pour l'UPDATE bank_transactions step 8 (mapper le `rows_affected() == 0` sur `AppError::Database(DbError::OptimisticLockConflict)`).
  - **Mapping `ReconciliationError::FiscalYearClosed` → `AppError`** : le handler DOIT matcher ce cas explicitement dans le `match lock_result { ... }` block (pattern 8-4). Exemple :
    ```rust
    Err(ReconciliationError::FiscalYearClosed { entry_date }) => {
        drop(tx_outer);
        Err(AppError::ReconciliationFiscalYearClosed { entry_date })
    }
    ```
    Sans ce match explicite, le cas tomberait dans `ReconciliationError::Database(e)` → 500 Internal Error (silencieux incorrect).
  
  - **F1'''' Pass 6 Opus — Mapping `ReconciliationError::Db(DbError)` → `AppError`** : le handler match aussi explicitement le nouveau variant `Db` (cf. T2.3) pour préserver la fidélité du `DbError` :
    ```rust
    Err(ReconciliationError::Db(db_err)) => {
        drop(tx_outer);
        Err(AppError::Database(db_err))
    }
    ```
    Le mapping `AppError::Database(DbError) → HTTP` est déjà géré dans `errors.rs:922+` (sous-match exhaustif) :
    - `DbError::FiscalYearClosed` → 400 `FISCAL_YEAR_CLOSED` (race step 6 → step 7).
    - `DbError::OptimisticLockConflict` → 409 `OPTIMISTIC_LOCK_CONFLICT` (race UPDATE bank_transactions step 8).
    - `DbError::Invariant(_)`, `DbError::Sqlx(_)`, etc. → 500 `INTERNAL_ERROR`.
    
    **Match block complet du handler** :
    ```rust
    match lock_result {
        Ok(journal_entry_id) => {
            tx_outer.commit().await.map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(Json(ManualMatchResponse { bank_transaction_id, journal_entry_id }))
        }
        Err(ReconciliationError::AccountLocked { bank_account_id, timeout_secs }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationAccountLocked { bank_account_id, timeout_secs })
        }
        Err(ReconciliationError::LockReleaseFailed { bank_account_id, .. }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationLockReleaseFailed { bank_account_id })
        }
        Err(ReconciliationError::FiscalYearClosed { entry_date }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationFiscalYearClosed { entry_date })
        }
        Err(ReconciliationError::Db(db_err)) => {
            drop(tx_outer);
            Err(AppError::Database(db_err))
        }
        Err(ReconciliationError::Database(e)) => {
            drop(tx_outer);
            Err(AppError::Database(DbError::Sqlx(e)))
        }
    }
    ```
    Match exhaustif sur tous les variants `ReconciliationError` (le compilateur Rust force la complétude).

- [ ] T3.4 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_manual_e2e.rs` *(nouveau fichier, ≥ 11 tests)* :
  1. `manual_match_creates_journal_entry_for_debit_transaction` (AC #83).
  2. `manual_match_creates_journal_entry_for_credit_transaction` (AC #84).
  3. `manual_match_rejects_unconfigured_bank_account_with_412` (AC #85, **nouveau 8-5a-base**).
  4. `manual_match_does_not_leak_cross_tenant_account` (AC #86).
  5. `manual_match_returns_404_on_cross_tenant_bank_account` (AC #87).
  6. `manual_match_rejects_already_reconciled_transaction` (AC #88).
  7. `manual_match_rejects_closed_fiscal_year` (AC #89).
  8. `manual_match_reverses_auto_rejection` (AC #90).
  9. `manual_match_requires_comptable_role` (AC #91 RBAC).
  10. `manual_match_emits_audit_log_pair` (AC #91 audit, **assertion sur `was_previously_rejected=false`** ET shape complète `details_json` : `bank_transaction_id`, `counterparty_account_id`, `journal_entry_id`, `amount`, `description`, `value_date` snake_case top-level).
  11. **F7''' Pass 3 Opus** — `manual_match_rejects_zero_amount_transaction` (L48) : tx avec `amount = 0.00` → 400 `VALIDATION_ERROR`. **F5'''' Pass 6 Opus — shape response body** : `AppError::Validation(msg)` à `errors.rs:418-420` émet `build_response(BAD_REQUEST, "VALIDATION_ERROR", &msg)` qui produit body `{ error: { code: "VALIDATION_ERROR", message: msg } }` — **PAS** de field `details.reason`. Le test doit asserter sur `body["error"]["code"] == "VALIDATION_ERROR"` ET `body["error"]["message"]` contient le marqueur `"zero_amount_transaction"` (utiliser `AppError::Validation("zero_amount_transaction".into())` pour que le marqueur soit dans le message). Si différenciation client requise v0.2 → créer un variant typé `AppError::ValidationWithReason { reason: &'static str }` qui émet `details: { reason }` (pattern 8-4 `BankCsvEmptyFile` errors.rs:868) — non bloquant v0.1.

  **F6''' Pass 3 Opus** — Le test #8 `manual_match_reverses_auto_rejection` (AC #90) DOIT inclure une assertion explicite sur le détail audit log `was_previously_rejected=true` (pas seulement le UPDATE bank_transactions `auto_match_rejected_at=NULL` côté row). Sans cette assertion, le test ne couvre pas la branche audit shape distinctive.

### T4. Helper `fiscal_years::find_open_covering_date` — vérification signature (AC #89)

- [ ] T4.1 — Pas besoin de créer un nouveau helper : utiliser `fiscal_years::find_open_covering_date` existant (Story 3-7) avec `&mut tx_outer` passé depuis le handler. Importer `kesh_db::repositories::fiscal_years` dans `crates/kesh-api/src/routes/reconciliation.rs`.

- [ ] T4.2 — Vérifier l'ordre des locks : `fiscal_years` est acquis APRÈS le lock advisory `with_account_lock` (MySQL `GET_LOCK()` au niveau session, advisory lock sans row-level overhead) → pas de deadlock possible. **Distinction** : `with_account_lock` utilise advisory lock (session-level), tandis que `find_open_covering_date` utilise `FOR UPDATE` (row lock intra-transaction). Les deux sont orthogonaux. L'ordre est : advisory lock → `find_strictly_pending` → `find_open_covering_date FOR UPDATE` → `create_in_tx` (re-prend le même fiscal_year lock = idempotent).

### T5. Frontend `ManualMatchModal` + extension `ReconciliationProposals` (AC #92)

- [ ] T5.1 — Étendre `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
  ```ts
  // **Différence majeure vs spec 8-5a unifiée** : pas de `bankLedgerAccountId`.
  // Résolu serveur-side via bank_account.journal_account_id (foundation 8-5a-zero).
  export async function manualMatchTransaction(
      bankAccountId: number,
      bankTransactionId: number,
      counterpartyAccountId: number,
      description?: string,
      valueDate?: string,
  ): Promise<{ bankTransactionId: number; journalEntryId: number }>;
  ```

- [ ] T5.2 — Créer `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` :
  - Props : `bankTransaction`, `bankAccountId`.
  - Sélecteur `Account` autocomplete (filtre client-side classes 5/6/7 via `account.number.startsWith('5/6/7')`). **Vérifier d'abord** que le composant `AccountAutocomplete.svelte` existe dans `frontend/src/lib/features/journal-entries/` et est compatible (accepte les options de filtrage). Créer un wrapper ou composant dédié `ManualMatchAccountSelector` si incompatibilité. **Note Pass 4 Sonnet** : le composant `AccountAutocomplete.svelte` accepte une prop `accounts: AccountResponse[]` pré-filtrée (examinée lors de la validate Pass 4). Pattern compatible : le `ManualMatchModal` doit filtrer **client-side** avant la prop :
    ```ts
    const filteredAccounts = accounts.filter(a => 
      ['5','6','7'].some(c => a.number.startsWith(c))
    );
    // puis passer filteredAccounts à <AccountAutocomplete accounts={filteredAccounts} />
    ```
    Aucun wrapper dédié n'est nécessaire. Pas de modification du composant existant.
  - Textarea description (200 chars max).
  - Datepicker `valueDate` pré-rempli (`tx.value_date ?? tx.booking_date`).
  - On submit : `manualMatchTransaction(...)` + dispatch event `success`.
  - Gestion erreur `412 BANK_ACCOUNT_NOT_CONFIGURED` : afficher message + lien vers `/bank-accounts` pour configuration (UX guide).

- [ ] T5.3 — Étendre `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - Pour chaque ligne tx avec `candidates: []` : 1 bouton « Affecter manuellement » (ouvre `ManualMatchModal`).
  - On modal success : refresh la liste.

- [ ] T5.4 — Tests Vitest (≥ 3) :
  1. `ReconciliationProposals: shows manual button for tx without candidate` (AC #92 part 1).
  2. `ManualMatchModal: prefills value date from tx.value_date` (AC #92 part 2).
  3. `manual_match_api_excludes_bank_ledger_account_id_from_request` (régression vs spec 8-5a unifiée — vérifier absence du field bankLedgerAccountId dans body POST, démarcation clair par rapport au verbe HTTP POST lui-même).

### T6. i18n (AC implicite UI)

- [ ] T6.1 — Ajouter ~5 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixe strict `reconciliation-manual-*`) :
  - `reconciliation-manual-button-label`
  - `reconciliation-manual-modal-title`
  - `reconciliation-manual-counterparty-label`
  - `reconciliation-manual-error-bank-not-configured`
  - `reconciliation-manual-success-toast`
  FR canonical.
- [ ] T6.2 — Traductions DE / IT / EN-CH — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [ ] T6.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T7. Tests E2E Playwright + a11y (AC #92)

- [ ] T7.1 — Créer `frontend/tests/e2e/reconciliation-manual.spec.ts` (≥ 1 actif) :
  1. `manual-match end-to-end` : login Comptable, navigate `/reconciliation`, click « Affecter manuellement » sur tx sans candidate, sélectionner compte 6810 dans dropdown, valider, vérifier toast succès + tx disparaît.

- [ ] T7.2 — Test a11y axe (AC #92) : 1 scénario sur la modal `ManualMatchModal` ouvert — `expect(await new AxeBuilder().analyze()).toHaveNoViolations()`.

## Risque de splitting

**Modules touchés** :
1. `crates/kesh-db/src/repositories/reconciliation.rs` (1 nouvelle fn `find_strictly_pending_by_id_for_account`).
2. `crates/kesh-reconciliation/src/manual.rs` *(nouveau)*.
3. `crates/kesh-reconciliation/src/lib.rs` + `errors.rs` (extension : 2 variants `FiscalYearClosed` + `Db(DbError)` — F1'''' Pass 6 Opus).
4. `crates/kesh-api/src/routes/reconciliation.rs` (1 nouveau handler `post_manual`).
5. `crates/kesh-api/src/errors.rs` (4 variants actifs : `BankAccountNotConfigured` nouveau, `ReconciliationFiscalYearClosed` nouveau, `ReconciliationTransactionNotPending` nouveau, `AccountNotFound` réutilisé — `BankAccountNotFound` existe déjà). `ReconciliationOptimisticLockConflict` **non créé** (réutilise `Database(DbError::OptimisticLockConflict)` pattern 8-4).
6. `crates/kesh-i18n` (5 clés × 4 locales).
7. `frontend/src/lib/features/reconciliation` (extension `reconciliation.api.ts` + `ReconciliationProposals.svelte` + nouveau `ManualMatchModal.svelte`).

**Total : 7 modules**. Au-dessus du seuil CLAUDE.md « splitter si > 5 modules ». **Pas de re-split** car (a) le scope est cohérent autour d'un seul flow (FR45 manual), (b) les patterns sont acquis 8-4, (c) volume estimé ~500-600 lignes spec + ~800-1000 lignes code = bien en-dessous du seuil 1500 lignes 8-4 retro.

**Aucune dérogation nécessaire**.

## Dev Notes

### API surface livrée 8-1b/8-2/8-3/8-4/8-5a-zero — patterns à réutiliser

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403.
- **Audit log atomique** : helper `audit_log::insert_in_tx(tx, NewAuditLogEntry { ... })`. Action `reconciliation.manual_matched` distincte (Q4a).
- **Erreurs structurées** : `AppError::*` typé. Body camelCase JSON.
- **i18n key ownership** : préfixe strict, kebab-case, lint-i18n-ownership pass (Story 6-3).
- **`rust_decimal::Decimal`** : Decimal exact partout pour amounts.
- **Repository pattern + sqlx** : Executor générique `<E: Executor>` (pattern 8-3 / 8-4).
- **Advisory lock per-account** : `with_account_lock(tx, company_id, bank_account_id, 5)` réutilisé pour manual (8-5a-bis split aussi).
- **`journal_entries::create_in_tx`** : helper Story 5-2, accepte tx ouverte par caller, ne commit pas. Émet audit `journal_entry.created` automatiquement.
- **`fiscal_years::find_open_covering_date`** : helper Story 3-7, indispensable pour résoudre `fiscal_year_id` à partir d'une `entry_date`.
- **`bank_account.journal_account_id`** : column livrée par 8-5a-zero — résolu serveur-side, pas de body field client.

### Lessons leçons des stories précédentes

- **8-4 retro** (cycle 4 passes review pour 5 modules / ~2200 lignes) : 8-5a-base découpée à ~600 lignes spec + ~1000 lignes code pour viser ≤ 2 passes review.
- **8-5a unifiée Pass 3 Opus** (dette F2'' détectée) : élimination à la racine via 8-5a-zero plutôt que body field `bankLedgerAccountId` propagé. Cohérence UX `default_*_account_id` Story 5-2.
- **5-2 leçon** (`create_in_tx` pour atomicité) : la nouvelle route manual **doit** utiliser `create_in_tx` plutôt que `create` (qui ouvre sa propre tx, incompatible avec la tx du `with_account_lock`).
- **8-4 patch P3-H1** (optimistic lock UPDATE bank_transactions `AND version = ?`) : appliquer **systématiquement** sur l'UPDATE bank_transactions de 8-5a-base pour défense-in-depth.

### Patterns architecturaux à respecter

- **Pas de dépendance circulaire** : `kesh-reconciliation → kesh-core, kesh-db` (cohérent 8-4). Le module `manual` consomme `kesh_db::entities::BankTransaction` et `kesh_db::entities::journal_entry::NewJournalEntry`.
- **Cohérence audit log snake_case top-level** : `details_json` 100% snake_case (cohérent F4'' Pass 3 décision Opus). Aucun sous-objet typé camelCase dans 8-5a-base.
- **Pas d'`f64` pour montants** : `Decimal` partout (`tx.amount`).
- **Tests : éviter le coupling temporel** : utiliser des dates fixes dans les seeds (`NaiveDate::from_ymd_opt(2026, 5, 15)`).
- **`auto_match_rejected_at=NULL` au manual-match** : indispensable pour éviter qu'une tx manual-matched apparaisse comme « rejetée + matched » (état incohérent).

### Source tree à toucher

**DB** :
- `crates/kesh-db/src/repositories/reconciliation.rs` (ajout `find_strictly_pending_by_id_for_account` — T1.1)
- `crates/kesh-db/src/repositories/fiscal_years.rs` (utiliser `find_open_covering_date` existant Story 3-7 — T4, pas de modification)
- `crates/kesh-db/src/repositories/accounts.rs` (utiliser `find_by_id_in_company` existant — T3.1, pas de modification)
- `crates/kesh-db/src/repositories/bank_accounts.rs` (utiliser `find_by_id_for_company` étendu par 8-5a-zero — T3.1, pas de modification)

**Backend `kesh-reconciliation`** :
- `crates/kesh-reconciliation/Cargo.toml` (deps inchangées)
- `crates/kesh-reconciliation/src/lib.rs` (refactor — module `manual` ajouté ; `split` reporté 8-5a-bis ; `rules` reporté 8-5b)
- `crates/kesh-reconciliation/src/manual.rs` *(nouveau, pure, helper réutilisé par 8-5a-bis et 8-5b)*
- `crates/kesh-reconciliation/src/errors.rs` (2 variants ajoutés : `FiscalYearClosed { entry_date }` + `Db(#[from] DbError)` — F1'''' Pass 6 Opus)

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation.rs` (extension : `post_manual` handler)
- `crates/kesh-api/src/lib.rs` (mount route)
- `crates/kesh-api/src/errors.rs` (5 nouvelles/réutilisées variantes)
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs` *(nouveau, ≥ 10 tests)*

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (~5 nouvelles clés `reconciliation-manual-*` × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (extension `manualMatchTransaction`, **pas de `bankLedgerAccountId`**)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (extension bouton « Affecter manuellement »)
- `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/ManualMatchModal.test.ts` *(nouveau, Vitest)*
- `frontend/tests/e2e/reconciliation-manual.spec.ts` *(nouveau, Playwright)*

### Standards de test

- **Unit `kesh-reconciliation`** : `#[cfg(test)] mod tests` inline `manual.rs`. ≥ 2 unit tests T2.4.
- **Intégration `kesh-db`** : `#[sqlx::test]`. ≥ 2 tests T1.2.
- **E2E HTTP `kesh-api`** : helper `spawn_app(pool)` (pattern 8-1b/8-2/8-3/8-4). ≥ 10 nouveaux tests T3.4.
- **Vitest frontend** : `npm run test:unit -- reconciliation`. ≥ 3 tests T5.4.
- **Playwright** : `frontend/tests/e2e/reconciliation-manual.spec.ts`. ≥ 1 actif + 1 a11y.

### Checklist locale avant push

```sh
# Backend (cf. CLAUDE.md « Test Locally First »)
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB up requis

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # T6.3
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reconciliation-manual.spec.ts
```

### Limitations connues v0.1 (sous-ensemble 8-5a-base)

| # | Limitation | Justification |
|---|---|---|
| L19 (héritée 8-4) | **LEVÉE** | FR45 manual crée une journal_entry directement sans facture pré-existante (compte de contrepartie librement choisi). |
| L20 (héritée 8-4) | **LEVÉE** | `journal_entries::create_in_tx` invoqué par manual. Pas de dépendance à `invoice.journal_entry_id`. |
| L23 (héritée 8-4) | **PARTIELLEMENT LEVÉE** | Manual RESET `auto_match_rejected_at=NULL` (cas « rejet → manual reverse »). Bouton « Annuler le rejet » UI v0.1 toujours absent — l'utilisateur doit explicitement créer une écriture. v0.2 : bouton dédié. |
| L21 (héritée 8-4) | **NON LEVÉE v0.1** | Reportée v0.2. Workaround 8-5a-base : manual avec compte « Différence de paiement » + invoice toujours dans `paid_at IS NULL`. |
| L18 (héritée 8-4) | **NON LEVÉE v0.1** | Pas de seuil auto-accept. Reportée v0.2. |
| **(post-Q5)** | Suggestion ML automatique post-manual non livrée | Décision Guy Q5 2026-05-07 : la response `POST /manual` ne retourne pas de `ruleSuggestion`. L'utilisateur crée ses rules manuellement via `/reconciliation/rules` (livré par 8-5b). Suggestion ML potentielle Story 8-5c v0.2 ou Epic 11+. |
| **L46 NEW (F5''' Pass 3 Opus)** | **NON DIFFÉRENCIATION** `NoFiscalYear` vs `FiscalYearClosed` dans le flow manual | `fiscal_years::find_open_covering_date` retourne `Option<FiscalYear>` (None unifié pour les 2 cas). Le handler manual mappe `None` → 409 `RECONCILIATION_FISCAL_YEAR_CLOSED` sans distinguer si l'exercice n'existe pas vs s'il existe mais est `Closed`. UX simplifiée v0.1 (user cible reçoit le même message « réconciliation impossible »). Différenciation v0.2 : ajouter un helper `find_any_covering_date` qui retourne le statut. |
| **L47 NEW (F8''' Pass 3 Opus)** | **Currency `tx.currency != "CHF"` non validée explicitement par 8-5a-base** | L'invariant `tx.currency == "CHF"` est garanti à l'amont par les imports 8-1b (CAMT.053) + 8-2 (CSV) + 8-3 (dedup) qui rejettent les non-CHF en 422 `BANK_IMPORT_UNSUPPORTED_CURRENCY`. Le handler `/manual` n'ajoute pas de check redondant. Si une tx non-CHF se faufile (jamais en v0.1), le UPDATE/INSERT journal_entry passera mais représente la transaction comme CHF (faux comptablement). Risque accepté v0.1, détection upstream incontournable. v0.2 : check belt-and-suspenders dans le handler manual + 422 `UNSUPPORTED_CURRENCY`. |
| **L48 NEW (F7''' Pass 3 Opus)** | **`tx.amount == 0` validation handler-side** | Le handler `/manual` doit pré-valider `tx.amount != Decimal::ZERO` (step 4bis §validation-handler-side, code 400 `VALIDATION_ERROR`, marqueur `"zero_amount_transaction"` dans `error.message` via `AppError::Validation("zero_amount_transaction".into())` — F5'''' Pass 6 Opus). Sinon le helper `build_journal_entry_for_counterparty` produirait 2 lignes débit/crédit 0/0 (sémantiquement vides, pollue les comptes). v0.2 : promouvoir cette validation au niveau import (rejeter à l'amont) + variant typé `AppError::ValidationWithReason { reason }` pour exposer le marqueur dans `details.reason` plutôt que `error.message`. |

### Risques et points d'attention pour le dev agent

1. **Path-dépendance 8-5a-zero** : ce 8-5a-base **ne peut démarrer** que si 8-5a-zero est `done`/merged sur main (column `bank_account.journal_account_id` doit exister DB-side ET dans entité Rust). Si non-respecté, les tests E2E HTTP passent sur fixtures sans configuration → cas dégradé `412 BANK_ACCOUNT_NOT_CONFIGURED` couvert mais le happy path AC #83-#84 cassent.

2. **Helper `manual::build_journal_entry_for_counterparty` réutilisé par 8-5a-bis et 8-5b** : ne pas le rendre privé. Le marquer `pub` dans `lib.rs` re-exports. La signature ne doit pas changer après merge 8-5a-base (8-5a-bis et 8-5b dépendent de la stabilité d'API). Si une évolution est nécessaire, elle doit faire l'objet d'un CR explicite.

3. **412 vs 404 vs 409 ordre** : le code 412 `BANK_ACCOUNT_NOT_CONFIGURED` est nouveau dans le système Kesh. Vérifier que `crates/kesh-api/src/errors.rs` mappe correctement vers HTTP 412 (Precondition Failed). Cohérence §error-precedence-order : 1. RBAC 403 → 2. ValidationBody 400 → 3. NotFound 404 → 4. **NEW Precondition 412** → 5. ConflictBusinessLogic 409.

4. **Test E2E HTTP volume** : 10 nouveaux tests minimum. Pas de dette test acceptable cette fois (lessons 8-4 retro). Sécurité multi-tenant + RBAC + non-régression sont **incontournables**.

5. **Suppression de la suggestion ML (Q5)** : ne pas implémenter la fonction `suggest_rule` ni l'objet `ruleSuggestion` dans la response. C'est explicitement out-of-scope 8-5a-base (et out-of-scope 8-5b aussi, reporté v0.2).

### Références

- [`8-5a-reconciliation-manuelle-split.md`](8-5a-reconciliation-manuelle-split.md) — spec d'origine `archived-split-bis` (référence des décisions de conception détaillées).
- [`8-5a-zero-bank-account-journal-link.md`](8-5a-zero-bank-account-journal-link.md) — pré-requis foundation column `journal_account_id`.
- [`8-5a-bis-split-breaking-accept.md`](8-5a-bis-split-breaking-accept.md) — sous-story FR48 split + breaking POST /accept Q2 (path-dep 8-5a-base).
- [`8-5b-reconciliation-rules-engine.md`](8-5b-reconciliation-rules-engine.md) — rules engine (path-dep 8-5a-bis).
- [`epic-8.md`](../planning-artifacts/epic-8.md) — Story 8-5 ACs originaux (FR45-48).
- [`prd.md`](../planning-artifacts/prd.md) §FR45-48 lignes 439-442.
- [`8-4-reconciliation-matching-automatique.md`](8-4-reconciliation-matching-automatique.md) — patterns repo + mutex + audit + savepoint à réutiliser.
- [`architecture.md`](../planning-artifacts/architecture.md) §11.5 (kesh-reconciliation), §17 (FR42-FR53 mapping).
- [Story 5-2 `journal_entries::create_in_tx`](../../crates/kesh-db/src/repositories/journal_entries.rs) — helper transaction-bound.
- [Story 3-7 `fiscal_years::find_open_covering_date`](../../crates/kesh-db/src/repositories/fiscal_years.rs) — résolution `fiscal_year_id` from `entry_date`.

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
| **2026-05-07** | Spec créée par re-split mécanique de 8-5a unifiée (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7). 8-5a-base = FR45 manual match utilisant `bank_account.journal_account_id` configuré en 8-5a-zero (foundation). **Différence majeure vs spec 8-5a unifiée** : le body POST `/manual` n'inclut PAS `bankLedgerAccountId` — résolu serveur-side. Anti-pattern UX éliminé à la racine. Helper public `kesh-reconciliation::manual::build_journal_entry_for_counterparty` réutilisé par 8-5a-bis et 8-5b (path dep). 10 ACs (#83-#92). Tasks T1-T7. Path-dépendance bloquante : 8-5a-zero `done`/merged. Status `8-5a-base-manual-match: backlog`. | Claude (Opus 4.7 re-split workflow) |
| **2026-05-08** | **Pass 6 validate Opus 4.7 — VALIDATION FINALE 6e PASSE** — 5 findings > LOW (0 CRITICAL + 1 HIGH + 4 MEDIUM + 2 LOW). Patches : (F1'''' HIGH) Variant `ReconciliationError::Db(#[from] DbError)` ajouté T2.3 — gap structurel détecté Pass 6 1M context : la closure `with_account_lock` retourne `Result<T, ReconciliationError>` mais `journal_entries::create_in_tx` retourne `Result<_, DbError>` non convertible (pas de `From<DbError> for sqlx::Error`). Sans ce variant la spec promettait un mapping `DbError::FiscalYearClosed → 400` impossible techniquement. Match handler exhaustif documenté T3.3. (F2'''' MEDIUM) §validation-handler-side step 9 — annotation explicite « steps 5-9 inside closure unique, NE PAS sortir audit_log de la closure ». (F3'''' MEDIUM) SQL UPDATE bank_transactions step 8 complétée avec `AND company_id = ? AND status = 'pending'` + `updated_at = NOW(3)` (defense-in-depth multi-tenant + status guard cohérent pattern 8-4 ligne 691). (F4'''' MEDIUM intégré dans F1'''' patch — typing audit log clarifié). (F5'''' MEDIUM) Test #11 zero_amount — clarification body shape `AppError::Validation` émet `error.message`, PAS `error.details.reason` (assertion test corrigée). (F6'''' LOW) §audit-log-shape clarifie démarcation vs 8-4 `MatchScore` (snake_case top-level applique uniquement aux scalars 8-5a-base, sans rétroaction 8-4). (F7'''' LOW) Référence « dette L64 » remplacée par « dette héritée 8-1b errors.rs:248-253 » pour clarté. Trend : Pass 1=3 → Pass 2=3 → Pass 3=7 → Pass 4=2 → Pass 5=2 → **Pass 6=5 findings > LOW** (1 HIGH gap structurel détecté par 1M context Opus + verification ground-truth exhaustive de `ReconciliationError`/`DbError`/`with_account_lock` types). Cycle review continue **Pass 7 Sonnet 4.6** après application des 7 patches. Critère STOP non atteint (1 HIGH > LOW). Path-dépendance : nouveau variant `Db(DbError)` réutilisé par 8-5a-bis (split flow) et 8-5b (accept-with-rule flow) — signature helper public `build_journal_entry_for_counterparty` **inchangée** (stable contractée). | Claude (Opus 4.7 1M validate) |
| **2026-05-08** | **Pass 5 validate Haiku 4.5** — 2 findings > LOW (0 CRITICAL + 0 HIGH + 2 MEDIUM + 2 LOW). Patches : (M1) T5.4.3 test naming clarity — renommer test pour expliciter validation absence du field `bankLedgerAccountId` vs verbe HTTP POST (confusion latente déjà clarifiée Pass 4 mais naming du test imprécis pour dev agent) ; (M2) T3.1 codification position étape 4bis — ajouter annotation explicite `// Step 4bis: zero amount pre-validation` et clarifier que cette validation est **PRÉ-flight** avant `with_account_lock` (non "inside lock") ; (M3) T5.2 exemple filtrage inline — ajouter snippet code `const filteredAccounts = accounts.filter(...)` pour cristalliser pattern pré-filtrage avant passage à `AccountAutocomplete` (pattern compatible Pass 4, mais clarté améliorée) ; (L1) T4.2 distinction locks — rephrase distinction advisory lock vs row lock pour clarté (advisory = `GET_LOCK()` session-level, FOR UPDATE = row lock intra-tx) ; (L2) T1.2 couverture exhaustive tests — clarifier que les 3 tests couvrent sécurité multi-tenant + filtre status + happy path complet. Trend : Pass 1=3 → Pass 2=3 → Pass 3=7 → Pass 4=2 → **Pass 5=2 findings > LOW** (stabilisation, 0 régressions Pass 4, omissions de clarité détectées). **Critère d'arrêt CLAUDE.md atteint** : ≤ 2 MEDIUM après 5 passes adversariales orthogonales (Pass 5 = dernière passe, tous findings = clarté/documentation, code-impact-zero). Cycle review **STOP**. Spec 8-5a-base ready for `bmad-dev-story`. | Claude (Haiku 4.5 validate) |
| **2026-05-08** | **Pass 1 validate Sonnet 4.6** — 3 findings (0 CRITICAL + 0 HIGH + 3 MEDIUM + 2 LOW). Patches : (1) F1 MEDIUM — signature incorrecte `create_in_tx` corrigée (§scope verrouillé + §validation-handler-side) : `(tx, fiscal_year_id, user_id, new_je)` au lieu de `(tx, new_je, company_id, fiscal_year_id, audit_actor)` ; (2) F2 MEDIUM — race condition `DbError::FiscalYearClosed` → 400 documentée (§validation-handler-side step 7) ; (3) F3 MEDIUM — mapping explicite `ReconciliationError::FiscalYearClosed → AppError::ReconciliationFiscalYearClosed` spécifié dans T3.3 (sans ce match → 500 silencieux) ; (4) F4 LOW — comptage variantes T3.3 + §risque-splitting corrigés (`ReconciliationOptimisticLockConflict` non créé — pattern `Database(OptimisticLockConflict)` 8-4 réutilisé). Trend : Pass 1 = 3 findings > LOW. Continuer Pass 2 Haiku 4.5. | Claude (Sonnet 4.6 validate) |
| **2026-05-08** | **Pass 2 validate Haiku 4.5** — 3 findings (0 CRITICAL + 0 HIGH + 3 MEDIUM + 0 LOW). Patches : (1) M1 MEDIUM — référence AC obsolète ligne 80 corrigée (`AC #82` → `AC #86` — spec 8-5a-base ACs #83-#92) ; (2) M2 MEDIUM — T1.2 tests incomplets : ajout test positif happy-path `find_strictly_pending_returns_tx_when_all_conditions_match` (couverture 100% du helper avant implémentation) ; (3) M3 MEDIUM — T5.2 compatibilité `AccountAutocomplete.svelte` non vérifiée : directive clarity ajoutée (vérifier avant reuse, créer wrapper dédié si incompatibilité). Trend : Pass 1 = 3 → Pass 2 = 3 findings > LOW (tous MEDIUM, pas de régressions Pass-1). Continue Pass 3 Opus 4.7 pour convergence. | Claude (Haiku 4.5 validate) |
| **2026-05-08** | **Pass 3 validate Opus 4.7 — VALIDATION FINALE** — 8 findings (1 CRITICAL + 1 HIGH + 5 MEDIUM + 1 LOW). Patches : (1) F1''' CRITICAL — pseudo-code T3.1 corrige `AppError::BankAccountNotFound` qui est **unit-struct** (pas `{ bank_account_id }`) — ne compile pas sinon (errors.rs:255 ground-truth) ; (2) F2''' HIGH — AC #87 corrige le code HTTP attendu : code réel `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` v0.1 (dette L64 documentée) — pas `BANK_ACCOUNT_NOT_FOUND` ; (3) F3''' MEDIUM — clarification mapping `find_open_covering_date None → ReconciliationError::FiscalYearClosed` traduit côté closure (sans cette traduction explicite, le `Option::None` ne propage pas) ; (4) F4''' MEDIUM — helper `build_journal_entry_for_counterparty` doc clarifie source `company_id` (depuis `tx.company_id`) + journal hardcodé `Journal::Banque` (fields obligatoires `NewJournalEntry`) ; (5) F5''' MEDIUM — sémantique `find_open_covering_date None` confond NoFiscalYear/Closed → L46 dette UX v0.1 (UX simplifié) ; (6) F6''' MEDIUM — test #8 `manual_match_reverses_auto_rejection` doit asserter `was_previously_rejected=true` dans audit shape ; (7) F7''' MEDIUM — pré-validation handler-side `tx.amount != 0` (step 4bis + L48 + nouveau test #11 `manual_match_rejects_zero_amount_transaction`) ; (8) F8''' LOW — currency non-CHF documentée en L47 (invariant garanti à l'amont par 8-1b/8-2/8-3, pas de check redondant). Trend : Pass 1 = 3 → Pass 2 = 3 → Pass 3 = 7 findings > LOW. **NON-CONVERGENT** : Pass 3 a remonté plus de findings que Pass 2 (le 1M context Opus a permis une vérification ground-truth exhaustive du code Rust qui a révélé F1''' CRITICAL + F2''' HIGH invisibles à Sonnet/Haiku sans accès au code réel). Cycle review continue. **Prochaine étape** : Pass 4 Sonnet 4.6 après application des 8 patches. Critère STOP non atteint. | Claude (Opus 4.7 1M validate) |
| **2026-05-08** | **Pass 4 validate Sonnet 4.6** — 2 findings > LOW (0 CRITICAL + 0 HIGH + 2 MEDIUM + 1 LOW). Patches : (M1) MEDIUM — T5.4.3 test Vitest verbe HTTP incorrect : `PATCH` → `POST` (la route est `POST /api/v1/reconciliation/manual`) ; (M2) MEDIUM — description 200 vs 500 chars incohérence ambiguë pour dev agent : note dev ajoutée dans T3.1 précisant `MAX_MANUAL_DESCRIPTION_LEN = 200` distinct de `MAX_DESCRIPTION_LEN = 500` de `journal_entries.rs` (business rule modal, libellé court) ; (LOW) T5.2 compatibilité `AccountAutocomplete.svelte` : note dev ajoutée confirmant que le composant accepte une prop `accounts` pré-filtrée, pattern compatible sans wrapper dédié. Trend : Pass 1 = 3 → Pass 2 = 3 → Pass 3 = 7 → **Pass 4 = 2 findings > LOW**. Critère d'arrêt CLAUDE.md **non atteint** (2 MEDIUM). **Prochaine étape : Pass 5 Haiku 4.5** (contexte frais, fenêtre orthogonale). | Claude (Sonnet 4.6 validate) |
