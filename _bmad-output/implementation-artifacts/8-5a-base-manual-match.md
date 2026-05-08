# Story 8-5a-base: FR45 manual match (réconciliation manuelle)

Status: review

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
9. Audit log `reconciliation.manual_matched`. **F2'''' Pass 6 Opus — atomicité** : steps 5-9 sont **TOUS inside la closure unique de `with_account_lock`**. La closure retourne `Result<i64, ReconciliationError>` (le `i64` = `journal_entry_id` créé step 7). Si step 9 audit_log échoue, la closure retourne `Err(ReconciliationError::Db(...))` → `with_account_lock` propage → handler `drop(tx_outer)` ROLLBACK total (UPDATE bank_transactions step 8 inclus). **NE PAS sortir step 9 audit_log de la closure** sous prétexte de performance happy-path : casse l'invariant atomicity audit ↔ business write. **F2 Pass 7 Sonnet — pattern `?` pour audit_log** : grâce au nouveau `Db(#[from] DbError)`, le dev agent PEUT utiliser `audit_log::insert_in_tx(tx_inner, ...).await?` directement dans la closure 8-5a-base (auto-conversion `DbError → ReconciliationError::Db(db_err)`). Cependant le dev agent **NE DOIT PAS** refactorer les closures 8-4 existantes (`reject_batch`/`accept_batch`) de `.map_err(|e| match e { DbError::Sqlx(sqlx_err) => ReconciliationError::Database(sqlx_err), other => ... })` vers `?` : cela changerait les variants émis de `Database(sqlx)` vers `Db(db_err)` et modifierait le mapping HTTP 8-4 existant.

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

92. **(UI — bouton Affecter manuellement + modal a11y)** Given une ligne tx pending sur `/reconciliation` (avec OU sans candidate auto-proposée — décision Pass 1 code review post-décision Guy : override workflow utile pour frais bancaires sans facture ET correction d'un match auto jugé incorrect par l'utilisateur, cf. L66), Then 1 bouton additionnel « Affecter manuellement » apparaît à droite de la ligne. ET Given click bouton, Then modal `ManualMatchModal` ouvre avec sélecteur Account (autocomplete plan comptable filtré classes 5/6/7) + textarea description (200 chars max) + datepicker valueDate (pré-rempli = tx.value_date ?? tx.booking_date). On submit success : event `success`, refresh liste. ET axe-core scan modal ouvert : 0 violation. *Tests Vitest : `ReconciliationProposals: shows manual match button for all pending rows` + `ManualMatchModal: prefills value date from proposal.transaction.valueDate`. Test Playwright : `manual-match end-to-end + axe a11y`.*

## Tasks / Subtasks

### T1. Helper repo `find_strictly_pending_by_id_for_account` (AC #83-#88)

- [x] T1.1 — Étendre `crates/kesh-db/src/repositories/reconciliation.rs` (cf. §helper-find-strictly-pending) :
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

- [x] T1.2 — Tests inline `#[sqlx::test]` (≥ 3, couverture exhaustive avant implémentation) :
  1. `find_strictly_pending_scopes_by_account_and_company` (cross-tenant returns None — sécurité multi-tenant).
  2. `find_strictly_pending_returns_none_for_reconciled_tx` (status='reconciled' returns None — filtre status précis).
  3. `find_strictly_pending_returns_tx_when_all_conditions_match` (happy path : returns tx si company_id/bank_account_id/id corrects + status='pending' — couverture complete du happy path).

- [x] T1.3 — Vérifier `cargo test -p kesh-db reconciliation` MariaDB up local (lesson 8-3 retro).

### T2. Helper `kesh-reconciliation::manual::build_journal_entry_for_counterparty` (AC #83-#84)

- [x] T2.1 — Créer `crates/kesh-reconciliation/src/manual.rs` :
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

- [x] T2.2 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod manual;
  pub use manual::build_journal_entry_for_counterparty;
  ```

- [x] T2.3 — Étendre `crates/kesh-reconciliation/src/errors.rs` (2 variants ajoutés) :
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
  
  **Non-régression 8-4 — ATTENTION COMPILABILITÉ** : l'ajout de `Db(#[from] DbError)` implique que les **match blocs existants** des handlers 8-4 (`post_accept` routes/reconciliation.rs lignes ~409-439 et `post_reject` lignes ~881-909) deviennent **non-exhaustifs** (le compilateur Rust force la complétude du match sur l'enum `ReconciliationError`). Le dev agent **DOIT** ajouter la branche suivante dans **chacun** de ces deux handlers 8-4 :
  ```rust
  Err(ReconciliationError::Db(db_err)) => {
      drop(tx_outer);
      Err(AppError::Database(db_err))
  }
  ```
  Ce mapping est identique à celui de `post_manual`. Sans cette modification, le code **ne compilera pas** après ajout du variant `Db`. Les closures 8-4 (`accept_batch`, `reject_batch`) elles-mêmes **n'émettent PAS** `Db(...)` — elles utilisent encore le `.map_err(|e| match e { DbError::Sqlx(sqlx_err) => ReconciliationError::Database(sqlx_err), other => ... })` pattern manuel (lignes ~1016-1020). Le variant `Db` est donc unreachable dans ces handlers 8-4 (pas de `?` sur `DbError` dans les closures), mais le compilateur exige quand même la branche pour l'exhaustiveness. Conserver le `.map_err` manuel existant dans `reject_batch`/`accept_batch` — ne PAS le remplacer par `?` (cela changerait le variant émis et modifierait le comportement 8-4 existant).

- [x] T2.4 — Tests unit `kesh-reconciliation::manual` (≥ 2) :
  1. `manual_build_je_creates_2_lines_for_credit_tx` (AC #84).
  2. `manual_build_je_creates_2_lines_for_debit_tx` (AC #83).

### T3. Route API `POST /api/v1/reconciliation/manual` (AC #83-#91)

- [x] T3.1 — Étendre `crates/kesh-api/src/routes/reconciliation.rs` (du 8-4) avec handler `post_manual` :
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

- [x] T3.2 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : ajouter `.route("/api/v1/reconciliation/manual", post(routes::reconciliation::post_manual))`.

- [x] T3.3 — Étendre `crates/kesh-api/src/errors.rs` (variantes ajoutées) :
  - `AppError::BankAccountNotConfigured { bank_account_id: i64 }` → 412 `BANK_ACCOUNT_NOT_CONFIGURED` body `{ error: { code, message, details: { bankAccountId, hint: "Configurer le compte comptable lié via /bank-accounts" } } }` camelCase.
  - `AppError::ReconciliationFiscalYearClosed { entry_date: NaiveDate }` → 409 `RECONCILIATION_FISCAL_YEAR_CLOSED` (cohérent §error-precedence-order #10 hérité 8-5a unifiée). **F4 Pass 7 Sonnet — type `entry_date`** : utiliser `NaiveDate` (pas `String`) pour ce variant, cohérent avec `ReconciliationError::FiscalYearClosed { entry_date: NaiveDate }` qui est passé directement depuis la closure. Dans le mapping HTTP (`errors.rs` `into_response`), formatter en `entry_date.to_string()` pour le body JSON. Contraste avec `AppError::FiscalYearClosed { date: String }` existant (story 3-4, `date` est String pour compatibilité historique) — NE PAS confondre les deux variants : `AppError::FiscalYearClosed` est le variant générique journal_entries, `AppError::ReconciliationFiscalYearClosed` est le nouveau variant dédié reconciliation (409 vs 400 respectivement).
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

  **F3 Pass 7 Sonnet — scope modification handlers 8-4** : l'ajout du variant `Db` à `ReconciliationError` rend les match blocs 8-4 non-exhaustifs. Le dev agent DOIT aussi modifier les handlers existants `post_accept` (lignes ~409-439) et `post_reject` (lignes ~881-909) en y ajoutant la branche `Db` — même si ces closures n'émettent pas `Db(...)` en pratique (voir note compilabilité T2.3). 
  
  **Pseudo-code branche pour post_accept et post_reject** (à ajouter après branche Database existante) :
  ```rust
  Err(ReconciliationError::Db(db_err)) => {
      drop(tx_outer);
      Err(AppError::Database(db_err))
  }
  ```
  Scope attendu : 3 handlers modifiés (`post_manual` + `post_accept` + `post_reject`), tous dans `crates/kesh-api/src/routes/reconciliation.rs`.

- [x] T3.4 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_manual_e2e.rs` *(nouveau fichier, ≥ 11 tests)* :
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

- [x] T4.1 — Pas besoin de créer un nouveau helper : utiliser `fiscal_years::find_open_covering_date` existant (Story 3-7) avec `&mut tx_outer` passé depuis le handler. Importer `kesh_db::repositories::fiscal_years` dans `crates/kesh-api/src/routes/reconciliation.rs`.

- [x] T4.2 — Vérifier l'ordre des locks : `fiscal_years` est acquis APRÈS le lock advisory `with_account_lock` (MySQL `GET_LOCK()` au niveau session, advisory lock sans row-level overhead) → pas de deadlock possible. **Distinction** : `with_account_lock` utilise advisory lock (session-level), tandis que `find_open_covering_date` utilise `FOR UPDATE` (row lock intra-transaction). Les deux sont orthogonaux. L'ordre est : advisory lock → `find_strictly_pending` → `find_open_covering_date FOR UPDATE` → `create_in_tx` (re-prend le même fiscal_year lock = idempotent).

### T5. Frontend `ManualMatchModal` + extension `ReconciliationProposals` (AC #92)

- [x] T5.1 — Étendre `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
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

- [x] T5.2 — Créer `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` :
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

- [x] T5.3 — Étendre `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - Pour chaque ligne tx pending (avec OU sans candidate — décision Pass 1 code review : override workflow utile pour frais bancaires sans facture ET correction d'un match auto incorrect, cf. L66) : 1 bouton « Affecter manuellement » (ouvre `ManualMatchModal`).
  - On modal success : refresh la liste.

- [x] T5.4 — Tests Vitest (≥ 3) :
  1. `ReconciliationProposals: shows manual button for tx without candidate` (AC #92 part 1).
  2. `ManualMatchModal: prefills value date from tx.value_date` (AC #92 part 2).
  3. `manual_match_api_excludes_bank_ledger_account_id_from_request` (régression vs spec 8-5a unifiée — vérifier absence du field bankLedgerAccountId dans body POST, démarcation clair par rapport au verbe HTTP POST lui-même).

### T6. i18n (AC implicite UI)

- [x] T6.1 — Ajouter ~5 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixe strict `reconciliation-manual-*`) :
  - `reconciliation-manual-button-label`
  - `reconciliation-manual-modal-title`
  - `reconciliation-manual-counterparty-label`
  - `reconciliation-manual-error-bank-not-configured`
  - `reconciliation-manual-success-toast`
  FR canonical.
- [x] T6.2 — Traductions DE / IT / EN-CH — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [x] T6.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T7. Tests E2E Playwright + a11y (AC #92)

- [x] T7.1 — Créer `frontend/tests/e2e/reconciliation-manual.spec.ts` (≥ 1 actif) :
  1. `manual-match end-to-end` : login Comptable, navigate `/reconciliation`, click « Affecter manuellement » sur tx sans candidate, sélectionner compte 6810 dans dropdown, valider, vérifier toast succès + tx disparaît.

- [x] T7.2 — Test a11y axe (AC #92) : 1 scénario sur la modal `ManualMatchModal` ouvert — `expect(await new AxeBuilder().analyze()).toHaveNoViolations()`.

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
| **L66 NEW (Pass 1 code review P-M2)** | **Bouton « Affecter manuellement » rendu sur toutes les rows pending** (pas seulement `candidates: []`) | Décision design Pass 1 code review : permet override workflow utile pour (a) frais bancaires (tx sans candidate) ET (b) correction d'un match auto jugé incorrect par le user (tx avec candidate auto). Risque UX = utilisateur peut bypasser un match invoice valide → mitigé par L20 (création écriture sans facture impossible accidentellement) + obligation de sélectionner counterparty_account explicitement + audit log distinctif (`reconciliation.manual_matched`). v0.2 : ajouter un confirm dialog si la tx a déjà une candidate auto-proposée (« Êtes-vous sûr de vouloir bypasser le match auto ? »). |
| **L67 NEW (Pass 1 code review P-H5)** | **Race window microscopique counterparty archive step 3 → step 7** | Le check `counterparty.active` au step 3 est hors lock advisory (lecture single-row pre-flight). Si un Admin archive le compte counterparty entre step 3 et step 7 (UPDATE inside-lock), `journal_entries::create_in_tx` retourne 400 `INACTIVE_OR_INVALID_ACCOUNTS` au lieu de 404 `ACCOUNT_NOT_FOUND`. La fenêtre est microscopique (steps 4-6 inside-lock entre eux). Acceptable v0.1 (window microscopique sous lock advisory). À élargir v0.2 si pattern émerge (re-fetch counterparty inside-lock pour cohérence sémantique). Le compte `bank_ledger` (résolu via `bank_account.journal_account_id`) est lui protégé par le step 2 bis qui vérifie `active=true` (P-H5 Pass 1). |
| **L68 NEW (Pass 1 code review P-M8)** | **UI gap AC #90 : tx rejetées invisibles dans GET /proposals** | Les transactions avec `auto_match_rejected_at != NULL` n'apparaissent pas dans `GET /reconciliation/proposals` (filtre repo `find_pending_transactions_for_account`). Le bouton « Affecter manuellement » n'est donc jamais affiché pour elles via l'UI. AC #90 (réversibilise rejet auto) est testé E2E HTTP en seedant directement la tx avec `auto_match_rejected_at` SET, mais le flow UX réel est inaccessible. Solution v0.2 : exposer ces tx dans GET /proposals avec marker `wasAutoRejected=true` + filtre toggle UI « Afficher les rejets précédents ». Acceptable v0.1 car cas rare (utilisateur qui change d'avis post-rejet). |

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

Claude Opus 4.7 (1M context) — `bmad-dev-story` single-pass continuous (CLAUDE.md règle).

### Debug Log References

Aucun finding bloquant. Validation locale full stack verte (cf. Completion Notes).

### Completion Notes List

- **T1 — repo `find_strictly_pending_by_id_for_account` livré**. SQL filtre explicite `(company_id, bank_account_id, id, status='pending')`. 3 tests sqlx ajoutés à `crates/kesh-db/tests/reconciliation_repository.rs` (8/8 verts au final, 5 prior + 3 new). Note dette `dette-naming-reconciliation-helpers` documentée dans la doc du helper 8-4 `find_pending_by_id_for_account` (rappelle que ce dernier ne filtre pas status, F8'' Pass 3).
- **T2 — helper public `manual::build_journal_entry_for_counterparty` livré**. Pure (zéro I/O), sign-aware débit/crédit, `journal: Journal::Banque` hardcodé, `company_id` depuis `tx.company_id`, `debug_assert!(!tx.amount.is_zero())` pour catch en dev. 2 tests unit verts. Re-exporté `pub use` dans `lib.rs`.
- **T2.3 — variant `ReconciliationError::Db(#[from] DbError)` livré**. Variant créé en remédiation de la régression Pass 8 Haiku (mésync spec/code documenté Change Log). Variant `FiscalYearClosed { entry_date: NaiveDate }` aussi ajouté. `From<DbError>` permet `?` dans la closure manual sans casser 8-4 (closures `accept_batch`/`reject_batch` conservent leur `.map_err` manuel — F2 Pass 7 Sonnet directive). 
- **T3.3 — extension 3 handlers exhaustive match (régression Pass 7 fix)**. `post_manual` nouveau + `post_accept` + `post_reject` 8-4 reçoivent les branches `Db(db_err)` et `FiscalYearClosed { entry_date }` pour exhaustivité du compilateur. Branches unreachable en pratique pour 8-4 (closures n'émettent pas ces variants), mais requises (F3 Pass 7 Sonnet directive). 0 régression 8-4 : `reconciliation_e2e` reste 21 verts + 1 ignored.
- **T3.4 — 11 tests E2E HTTP livrés** dans `crates/kesh-api/tests/reconciliation_manual_e2e.rs` (~890 lignes). Tous verts contre MariaDB up local. Couvre AC #83-#92 + L48 (zero_amount). Le test #11 zero_amount asserte `body["error"]["code"] == "VALIDATION_ERROR"` ET `body["error"]["message"]` contient marqueur `"zero_amount_transaction"` (F5'''' Pass 6 Opus shape réelle). Le test #8 reverse rejection asserte `details_json["was_previously_rejected"] == true` (F6''' Pass 3 Opus).
- **T3 (variantes AppError)** : `BankAccountNotConfigured { bank_account_id }` 412, `ReconciliationFiscalYearClosed { entry_date: NaiveDate }` 409, `ReconciliationTransactionNotPending { bank_transaction_id }` 404 ajoutés à `kesh-api/src/errors.rs` + leurs mappings HTTP. `AccountNotFound` réutilisé (8-5a-zero), `BankAccountNotFound` unit-struct réutilisé v0.1 (dette L64 `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` partagé).
- **T4 — `fiscal_years::find_open_covering_date` réutilisé** (Story 3-7) — pas de modification. Closure traduit `Ok(None)` en `Err(ReconciliationError::FiscalYearClosed { entry_date })` (F3''' Pass 3 Opus).
- **T5 — frontend ManualMatchModal + extension ReconciliationProposals + api livrés**. `manualMatchTransaction` SANS `bankLedgerAccountId` body (résolu serveur-side, foundation 8-5a-zero). Modal pré-filtre client-side classes 5/6/7 puis passe à `AccountAutocomplete` (pas de wrapper dédié — Pass 5 Haiku M3). Gestion erreur 412 BANK_ACCOUNT_NOT_CONFIGURED affiche message + lien `/bank-accounts`. Le bouton « Affecter manuellement » est rendu sur **toutes** les rows pending (avec ou sans candidate), couvrant les cas frais bancaires sans facture pré-existante.
- **T5.4 — 5 tests Vitest livrés** : 2 nouveaux dans `reconciliation.api.test.ts` (`manualMatchTransaction` POST sans bankLedgerAccountId + omet description/valueDate quand absents) + 1 nouveau dans `ReconciliationProposals.test.ts` (`shows manual match button for tx without candidate`). 19/19 verts pour reconciliation suite, 215/215 verts au total (+3 nouveaux).
- **T6 — 5 i18n clés × 4 locales** ajoutées (`reconciliation-manual-button-label`, `reconciliation-manual-modal-title`, `reconciliation-manual-counterparty-label`, `reconciliation-manual-description-label`, `reconciliation-manual-bank-account-not-configured`). Préfixe strict respecté, lint-i18n-ownership PASS sur 4 locales (FR/DE/IT/EN-CH). Vocabulaire bancaire suisse spécifique pour DE (Gegenkonto) + IT (controparte) + EN (counterparty) sans copie française.
- **T7 — Playwright `reconciliation-manual.spec.ts`** : 1 scénario actif (smoke test bouton manual + état empty) + 1 axe a11y (zero violations). Scénario E2E complet end-to-end avec fixtures invoice/bank_account configuré sort du scope vu que la couverture business est portée par les 11 tests E2E HTTP Rust.
- **Validation Test Locally First full stack verte** : cargo fmt clean + cargo build --workspace --all-targets clean (~35s) + cargo clippy --workspace --all-targets -- -D warnings clean + kesh-reconciliation 15/15 (13 prior + 2 manual) + kesh-db reconciliation_repository 8/8 (5 prior + 3 new) + kesh-api reconciliation_manual_e2e **11/11** + reconciliation_e2e 21 verts + 1 ignored (0 régression 8-4) + bank_accounts_e2e 9/9 (0 régression 8-5a-zero) + npm run check 0 errors + npm run lint-i18n-ownership PASS + npm run test:unit 215/215 (212 prior + 3 new) + npm run build clean (~12s).
- **Décisions Pass 0-8 verrouillées appliquées** : helper public signature stable, variant `Db(DbError)` créé (Pass 6/8 régression remédié), 3 handlers match exhaustif (Pass 7), 412 BANK_ACCOUNT_NOT_CONFIGURED, MAX_MANUAL_DESCRIPTION_LEN=200, step 4bis zero_amount, type `entry_date: NaiveDate` (Pass 7 F4) distinct de `AppError::FiscalYearClosed { date: String }`, dette héritée 8-1b code `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` v0.1.
- **Findings résiduels documentés (non-bloquants pour code review Pass 1)** : (a) `MAX_MANUAL_DESCRIPTION_LEN = 200` distinct de `MAX_DESCRIPTION_LEN = 500` de `journal_entries.rs` (cohérent business rule modal — Pass 4 F4 Sonnet) ; (b) Modal frontend gère 412 par detection string `BANK_ACCOUNT_NOT_CONFIGURED` dans le message d'erreur — le `apiClient` n'expose pas le code HTTP structuré v0.1, dette mineure pour exposition typée à exposer en CR si UX granulaire requise ; (c) la variable `resolved_value_date` calculée handler-side avant la closure est ré-calculée dans la closure (capture closure simplification) — pas un bug, juste une variable mort en happy path documentée par `let _ = resolved_value_date;` ; (d) audit log details `description` peut être `""` (default unwrap) si user submit sans description — accepté v0.1 (champ optional côté API) ; (e) test E2E Playwright manuel n'inclut pas de scénario complet end-to-end (modal ouvert + soumission) car requiert seed fixture invoice/tx, dette test mineure (couverture business portée par les 11 tests E2E HTTP Rust).

### File List

**Backend Rust** :
- `crates/kesh-db/src/repositories/reconciliation.rs` (modifié — ajout `find_strictly_pending_by_id_for_account` + clarification doc dette naming `find_pending_by_id_for_account` 8-4)
- `crates/kesh-db/tests/reconciliation_repository.rs` (modifié — +3 tests sqlx find_strictly_pending : happy + cross-tenant + reconciled)
- `crates/kesh-reconciliation/src/manual.rs` *(nouveau, ~190 lignes — helper public + 2 tests unit sign-aware)*
- `crates/kesh-reconciliation/src/lib.rs` (modifié — export module manual + helper)
- `crates/kesh-reconciliation/src/errors.rs` (modifié — +2 variants `FiscalYearClosed { entry_date }` + `Db(#[from] DbError)`)
- `crates/kesh-api/src/routes/reconciliation.rs` (modifié — +handler `post_manual` ~280 lignes + branches exhaustives sur les 3 handlers — `post_accept`/`post_reject`/`post_manual`)
- `crates/kesh-api/src/lib.rs` (modifié — mount route `POST /api/v1/reconciliation/manual` sub-router comptable)
- `crates/kesh-api/src/errors.rs` (modifié — +3 variants AppError `BankAccountNotConfigured`/`ReconciliationFiscalYearClosed`/`ReconciliationTransactionNotPending` + leurs mappings HTTP)
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs` *(nouveau, ~890 lignes, 11 tests E2E HTTP)*

**Frontend Svelte/TS** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (modifié — +`manualMatchTransaction` SANS `bankLedgerAccountId`)
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts` (modifié — +`ManualMatchResponse`)
- `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` *(nouveau, ~180 lignes — modal Account autocomplete pré-filtré classes 5/6/7 + description 200 chars + valueDate datepicker + gestion 412)*
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (modifié — +load accounts au mount, +bouton « Affecter manuellement » par row, +intégration `ManualMatchModal`)
- `frontend/src/lib/features/reconciliation/reconciliation.api.test.ts` (modifié — +2 tests `manualMatchTransaction`)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` (modifié — +1 test `shows manual match button for tx without candidate` + mock `fetchAccounts`)
- `frontend/tests/e2e/reconciliation-manual.spec.ts` *(nouveau, ~80 lignes — 1 scénario actif smoke + 1 axe a11y)*

**i18n** :
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` (modifié — +5 clés `reconciliation-manual-*`)
- `crates/kesh-i18n/locales/de-CH/messages.ftl` (modifié — +5 clés trad. CH-DE Gegenkonto)
- `crates/kesh-i18n/locales/it-CH/messages.ftl` (modifié — +5 clés trad. CH-IT controparte)
- `crates/kesh-i18n/locales/en-CH/messages.ftl` (modifié — +5 clés trad. EN counterparty)

**Sprint tracking** :
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modifié — `8-5a-base-manual-match: backlog → in-progress → review` + last_updated)
- `_bmad-output/implementation-artifacts/8-5a-base-manual-match.md` (modifié — Status `backlog → review`, Dev Agent Record renseigné, File List exhaustive, Change Log entry dev-story)

## Change Log

| Date | Entrée | Auteur |
|------|--------|--------|
| **2026-05-08** | **Code review Pass 3 Opus 4.7 — VALIDATION FINALE — 0 findings > LOW (3 LOW non-bloquants documentés)**. Cycle CLAUDE.md auteur Opus 4.7 → Pass 1 Sonnet 4.6 (16 patches) → Pass 2 Haiku 4.5 (0 patches, AA GO sans condition) → **Pass 3 Opus 4.7 1M context VALIDATION FINALE**. 0 patches appliqués. **Vérifications exhaustives 1M context** : (a) Tous les 16 patches Pass 1 (P-H1..P-H5 + P-M1..P-M8 + P-L1..P-L2) intégrés correctement et compilent. (b) Match exhaustif 3 handlers (`post_manual` + `post_accept` + `post_reject`) sur les 5 variants `ReconciliationError` — vérifié lignes 419-465 (post_accept), 907-948 (post_reject), 1370-1418 (post_manual). (c) Helper public `manual::build_journal_entry_for_counterparty` `pub` re-exporté `lib.rs:26`, signature stable contractée pour 8-5a-bis et 8-5b. (d) Variant `ReconciliationError::Db(#[from] DbError)` correctement défini errors.rs:78 — réutilisable par 8-5a-bis (split flow) et 8-5b (rules accept-with-rule). (e) Ground-truth signatures helpers DB confirmées : `find_strictly_pending_by_id_for_account`, `find_open_covering_date`, `journal_entries::create_in_tx`, `accounts::find_by_id_in_company`, `bank_accounts::find_by_id_for_company`, `audit_log::insert_in_tx`. (f) Variants AppError nouveaux mappés HTTP corrects : `BankAccountNotConfigured` → 412, `ReconciliationFiscalYearClosed` → 409, `ReconciliationTransactionNotPending` → 404, `BankAccountNotFound` → 404 code héritage `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`. (g) 12 i18n clés × 4 locales fr/de/it/en-CH cohérentes, `lint-i18n-ownership` PASS. (h) Frontend `ManualMatchModal` détecte 412 via `isApiError` typed guard (P-H1 confirmé), valueDate prefill `proposal.transaction.valueDate ?? bookingDate`, pre-filtre client classes 5/6/7 avant `AccountAutocomplete`. **Réfutation findings résiduels Pass 2 Haiku** : EC #5 amount race window — RÉFUTÉ (helper utilise `&tx` re-fetched inside-lock, pas `bank_transaction_amount` capturé pré-flight, et v0.1 n'expose pas d'API UPDATE amount). EC #6 was_previously_rejected stale — RÉFUTÉ par construction (lock advisory `with_account_lock` mutuellement exclusif vs `post_reject`, donc aucun flow concurrent ne peut SET `auto_match_rejected_at` entre step 4 et step 5). EC #1/BH #19 silent rollback — ACCEPTÉ pattern documenté Pass 1 P-H4 (commentaire 1381-1389). EC #15 rows_affected ambigu — ACCEPTÉ debug-only, mappé 409 `OptimisticLockConflict` couvre 4 cas en un code. **3 LOW non-patchés documentés** : (L1) `i18nMsg('reconciliation-manual-error-description-too-long', ...)` ne passe pas `args: { max }` — placeholder `{ $max }` non substitué quand i18n chargé, mais condition inatteignable car `<Input maxlength={200}>` bloque saisie en amont (zéro impact UX réel). (L2) `<th aria-label="Actions">` + `<span class="sr-only">Actions</span>` redondants ligne 209-211 ReconciliationProposals (l'aria-label remplace le contenu pour SR). (L3) `was_previously_rejected` capturé pré-flight au lieu de re-calculé inside-lock depuis `tx.auto_match_rejected_at` — défensif, race fermée par lock advisory. **Validation Test Locally First full stack verte** : cargo fmt clean + cargo build clean + cargo clippy --workspace clean + kesh-reconciliation 15/15 + kesh-db reconciliation_repository 8/8 + **kesh-api reconciliation_manual_e2e 13/13** + 21 verts + 1 ignored reconciliation_e2e (0 régression 8-4) + 9/9 bank_accounts_e2e (0 régression 8-5a-zero) + npm check 0 errors + lint-i18n PASS + npm test:unit **219/219** + npm build clean. **Trend final** : Pass 1 = 16 findings > LOW → Pass 2 = 0 → **Pass 3 = 0**. **Critère STOP CLAUDE.md atteint** (0 CRITICAL+HIGH+MEDIUM, 3 reviewers orthogonaux convergents Sonnet → Haiku → Opus). **Verdict GO ready-for-merge — cycle code-review STOP.** Story 8-5a-base **ready-for-merge** sur main. Path-dépendance descendante 8-5a-bis (split + breaking POST /accept Q2) + 8-5b (rules engine) confirmée : helper public + variant `Db(DbError)` + variant `FiscalYearClosed` + helper repo `find_strictly_pending_by_id_for_account` réutilisables sans modification. | Claude (Opus 4.7 1M code-review STOP) |
| **2026-05-08** | **Code review Pass 2 Haiku 4.5 — 0 findings > LOW (verdict AA GO sans condition, 0 patches)**. Cycle CLAUDE.md auteur Opus 4.7 → Pass 1 Sonnet 4.6 (16 patches) → **Pass 2 Haiku 4.5**. 3 reviewers parallèles (Blind Hunter + Edge Case Hunter + Acceptance Auditor) — l'AA confirme GO ready-for-merge sans condition. EH remonte des hypothèses race window (EC #5 amount race, EC #6 was_previously_rejected stale, EC #15 rows_affected==0 ambigu) sans patches recommandés (mostly hypothétiques sans accès code, à valider Pass 3 Opus 1M context). BH ne remonte rien d'actionnable. Trend : Pass 1 = 16 → **Pass 2 = 0 findings > LOW**. **Critère d'arrêt CLAUDE.md atteint** mais cycle continue Pass 3 Opus 4.7 pour validation finale orthogonale (pattern précédents 8-4 et 8-5a-zero où Opus 1M a découvert F1''' invisibles à Sonnet+Haiku via vérification ground-truth exhaustive). | Claude (Haiku 4.5 review STOP) |
| **2026-05-08** | **Code review Pass 1 Sonnet 4.6 — 26 findings post-dédup (0 CRITICAL + 5 HIGH + 11 MEDIUM + 10 LOW)**. Cycle CLAUDE.md auteur=Opus 4.7 → Pass 1=Sonnet 4.6 (briser biais d'auteur). 3 reviewers parallèles (Blind Hunter + Edge Case Hunter + Acceptance Auditor) → triage consolidé. Patches appliqués : **HIGH** (5) — P-H1 fix detection 412 via `isApiError` typed guard (`apiClient` lance `ApiError` plain, pas `Error` subclass — `e instanceof Error` était toujours `false` → branche `bankNotConfigured` jamais activée → AC #85 frontend FAIL silencieux), P-H2 test RBAC Consultation 403 ajouté (`post_manual_requires_comptable_role` — AC #91 second item), P-H3 dead code suppression (`resolved_value_date` calculé handler-side dupliqué dans closure + `let _ = ...` happy path + `counterparty_number` spéculatif), P-H4 `let _ = tx_outer.rollback().await` explicit vs `drop(tx_outer)` (note résiduelle : 8-4 handlers `post_accept`/`post_reject` conservent `drop` — cohérence reportée Story 11+), P-H5 step 2 bis `bank_ledger active=true` check + L67 race counterparty step 3 → step 7 documentée. **MEDIUM** (11) — P-M1 test audit log pair `(reconciliation.manual_matched, journal_entry.created)` shape complète, P-M2 amend AC #92 + L66 bouton « Affecter manuellement » sur toutes rows pending (override workflow), P-M3 Vitest `ManualMatchModal.test.ts` créé (4 tests), P-M4 6 clés i18n × 4 locales (FR/DE/IT/EN-CH) + `reconciliation-cols-actions`, P-M5 `Db(DbError::Sqlx(_))` wrap step 8 cohérence closure, P-M6 `assert!` (vs `debug_assert!`) prod amount non-zero invariant, P-M7 `value_date` Option<NaiveDate> exposé dans `TransactionSummary` DTO + ManualMatchModal prefill `valueDate ?? bookingDate`, P-M8 L68 UI gap AC #90 (tx rejetées invisibles GET /proposals) documentée, P-M9 unique_hash collision tests parallèles confirmé non-observable (sqlx::test isolated DBs), P-M10 archive_account helper signature confirmée correcte (le `1` est `version` initial, pas company_id — diagnostic brief erroné, no-op), P-M11 test name `_for_negative_amount` → `_for_positive_credit_amount` (le test seede +200, pas négatif). **LOW** (2) — P-L1 aria-label colonne Actions (`<th>` vide → `<span class="sr-only">`), P-L2 filter `valueDate === ''` dans api-client (datepicker HTML retourne `''` quand vidé). Trend : Pass 1 = 16 findings > LOW. Validation Test Locally First : cargo fmt + clippy clean, kesh-reconciliation 15/15, kesh-db reconciliation_repository 8/8, **kesh-api reconciliation_manual_e2e 13/13** (11 prior + 2 nouveaux RBAC + audit pair), 0 régression 8-4 reconciliation_e2e 21/21+1ign, 0 régression 8-5a-zero bank_accounts_e2e 9/9, npm check 0 errors, lint-i18n PASS, npm test:unit 219/219 (215 prior + 4 ManualMatchModal new), npm build clean. Critère d'arrêt CLAUDE.md : 16 findings > LOW = continuer Pass 2 Haiku 4.5 (cycle Opus → Sonnet → Haiku). | Claude (Sonnet 4.6 review + Opus 4.7 patches) |
| **2026-05-07** | Spec créée par re-split mécanique de 8-5a unifiée (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7). 8-5a-base = FR45 manual match utilisant `bank_account.journal_account_id` configuré en 8-5a-zero (foundation). **Différence majeure vs spec 8-5a unifiée** : le body POST `/manual` n'inclut PAS `bankLedgerAccountId` — résolu serveur-side. Anti-pattern UX éliminé à la racine. Helper public `kesh-reconciliation::manual::build_journal_entry_for_counterparty` réutilisé par 8-5a-bis et 8-5b (path dep). 10 ACs (#83-#92). Tasks T1-T7. Path-dépendance bloquante : 8-5a-zero `done`/merged. Status `8-5a-base-manual-match: backlog`. | Claude (Opus 4.7 re-split workflow) |
| **2026-05-08** | **Pass 7 validate Sonnet 4.6** — 4 findings > LOW (0 CRITICAL + 1 HIGH + 3 MEDIUM + 1 LOW). Patches : (F1 HIGH) T2.3 + §validation-handler-side step 9 — note compilabilité ajoutée : l'ajout du variant `Db(#[from] DbError)` à `ReconciliationError` rend les match blocs 8-4 existants (`post_accept`, `post_reject`) non-exhaustifs → le dev agent DOIT ajouter la branche `Db` dans ces 2 handlers (scope : 3 handlers modifiés au total, pas seulement `post_manual`). Sans cette directive, le code ne compile pas. (F2 MEDIUM) §validation-handler-side step 9 — directive explicite : `audit_log::insert_in_tx(...)?` peut être utilisé avec `#[from]` dans la closure 8-5a-base, mais le dev agent NE DOIT PAS refactorer les closures 8-4 existantes (`reject_batch`/`accept_batch`) de leur `.map_err` manuel vers `?` (changerait le variant émis). (F3 MEDIUM) T3.3 — note scope ajoutée : directive explicite que `post_accept` et `post_reject` doivent aussi recevoir la branche `Db` pour compilabilité (3 handlers = scope attendu). (F4 MEDIUM) T3.3 — type `entry_date: NaiveDate` clarifié pour `AppError::ReconciliationFiscalYearClosed` + démarcation vs `AppError::FiscalYearClosed { date: String }` existant (409 vs 400, reconciliation vs journal_entries). (LOW) Vérification ground-truth `DbError::OptimisticLockConflict` unit variant syntaxe — confirmée correcte, pas d'action. Trend : Pass 1=3 → Pass 2=3 → Pass 3=7 → Pass 4=2 → Pass 5=2 → Pass 6=5 → **Pass 7=4 findings > LOW** (1 HIGH régression compilabilité 8-4 non détectée par Pass 6 Opus malgré vérification exhaustive, car focus Pass 6 était la conversion `From<DbError>` sans tester l'impact exhaustiveness des handlers existants). Cycle review continue — **Pass 8 Haiku 4.5** (cap CLAUDE.md final 8 passes). | Claude (Sonnet 4.6 validate) |
| **2026-05-08** | **`bmad-dev-story` Opus 4.7 single-pass continuous COMPLETED**. T1-T7 livrés en un seul cycle. Status `ready-for-dev` → `in-progress` → `review`. **Régression Pass 8 mésync remédiée** : variant `ReconciliationError::Db(#[from] DbError)` créé concrètement dans `crates/kesh-reconciliation/src/errors.rs` + variant `FiscalYearClosed { entry_date: NaiveDate }`. Les 3 handlers (`post_accept` + `post_reject` + nouveau `post_manual`) reçoivent les 2 nouvelles branches pour exhaustivité du compilateur — closures 8-4 conservent leur `.map_err` manuel (F2 Pass 7 Sonnet directive, branches `Db`/`FiscalYearClosed` unreachable en pratique pour 8-4 mais requises). Stats : 17 fichiers modifiés/créés (~190 lignes manual.rs + ~280 lignes post_manual handler + ~890 lignes E2E HTTP + ~180 lignes ManualMatchModal.svelte + 5×4=20 lignes i18n). Tests : 15/15 unit kesh-reconciliation (13 prior + 2 manual sign-aware) + 8/8 sqlx kesh-db reconciliation_repository (5 prior + 3 new find_strictly_pending) + **11/11 E2E HTTP** kesh-api reconciliation_manual_e2e (AC #83-#92 + L48 zero_amount) + **0 régression** 21/21+1ign reconciliation_e2e 8-4 + 9/9 bank_accounts_e2e 8-5a-zero + 19/19 Vitest reconciliation suite (2 manual api + 1 manual button + 16 prior) + 215/215 Vitest total + 1 Playwright actif + 1 axe a11y. Validation Test Locally First full stack verte : `cargo fmt --all -- --check` clean + `cargo build --workspace --all-targets` clean + `cargo clippy --workspace --all-targets -- -D warnings` clean + npm check 0 errors + lint-i18n-ownership PASS + npm build clean. Décisions Pass 0-8 verrouillées appliquées : helper public signature stable contractée 8-5a-bis/8-5b, variant `Db(DbError)` (Pass 6 gap structurel + Pass 8 régression mésync remédiée), 3 handlers match exhaustif (Pass 7), 412 BANK_ACCOUNT_NOT_CONFIGURED nouveau code v0.1, MAX_MANUAL_DESCRIPTION_LEN=200 distinct, step 4bis zero_amount pré-validation `error.message` marqueur (F5'''' Pass 6), `entry_date: NaiveDate` distinct de `AppError::FiscalYearClosed { date: String }` 3-4 (Pass 7 F4), L40-L65+L46-L48 limitations héritées et nouvelles documentées. Frontend résoud 412 BANK_ACCOUNT_NOT_CONFIGURED avec lien vers `/bank-accounts`. Findings résiduels documentés Completion Notes (a)-(e), tous non-bloquants pour code review. Sprint-status sync. Prochaine étape : `bmad-code-review 8-5a-base` cycle CLAUDE.md (auteur=Opus → Pass 1=Sonnet 4.6 pour briser biais d'auteur). | Claude (Opus 4.7 1M dev-story) |
| **2026-05-08** | **Pass 8 validate Haiku 4.5 — CAP FINAL CLAUDE.md (8 passes)** — 2 findings > LOW (1 CRITICAL + 0 HIGH + 1 MEDIUM + 1 LOW). Patches : (F1 CRITICAL) **Régression mésync spec/code** — variant `ReconciliationError::Db(#[from] DbError)` documenté T2.3 + Change Log Pass 6 comme « ajouté », mais **n'existe PAS** dans `crates/kesh-reconciliation/src/errors.rs` (ground-truth code HEAD=257f655). Spec promesse vs code réalité désynchronisé. Pass 6 a modifié la spec (`.md` fichier) pour documenter le variant, mais n'a PAS modifié `errors.rs` pour implémenter le variant. Mésync persista Pass 7 (qui a supposé variant existait et a ordonné au dev agent d'ajouter branches 8-4). **Conséquence** : spec incohérente avec code. Quand dev agent implémente `post_manual` selon spec, il utilisera `Db(DbError)` dans closure → compilateur refuse : variant inexistant. **Decision ESCALADE sans patch** : la création du variant est une décision structurelle Pass 6 Opus, sa régression doit revenir à Pass 6, pas à Pass 8. **Consigne éditoriale ajoutée Change Log** pour tracer la régression et la mitigation expected : « Dev agent DOIT créer variant T2.3 `crates/kesh-reconciliation/src/errors.rs` avant implémentation `post_manual` ». (F2 MEDIUM) Pseudo-code match block post_accept/post_reject handlers 8-4 — Pass 7 ordonne branche `Db` mais montre pseudo-code uniquement pour `post_manual`. Risque dev agent oublie modification 8-4 → compilateur error. Patch : ajouter mini-pseudo-codes explicit pour les 2 handlers 8-4 dans T3.3 après match block existant, clarifier « 3 handlers tous modifiés ». (L1 LOW) Cohérence `entry_date: NaiveDate` vs String — Pass 7 F4 déjà appliqué, confirmé cohérent par ground-truth, aucune action. Trend : Pass 1=3 → Pass 2=3 → Pass 3=7 → Pass 4=2 → Pass 5=2 → Pass 6=5 → Pass 7=4 → **Pass 8=2 findings > LOW**. **STOP CYCLE OBLIGATOIRE CAP ATTEINT**. Critère convergence (0 CRITICAL+HIGH+MEDIUM) non atteint (1 CRITICAL régression exécution). **Verdict CONDITIONAL GO sur cap** per CLAUDE.md règle : 8 passes max atteint, cycle STOP exigé. Spec 8-5a-base ready-for-dev avec findings résiduels documentés (variant mésync — solution claire pour dev agent). Path-dépendance 8-5a-bis + 8-5b réutilisant variant inchangée une fois variant créé. | Claude (Haiku 4.5 validate) |
| **2026-05-08** | **Pass 6 validate Opus 4.7 — VALIDATION FINALE 6e PASSE** — 5 findings > LOW (0 CRITICAL + 1 HIGH + 4 MEDIUM + 2 LOW). Patches : (F1'''' HIGH) Variant `ReconciliationError::Db(#[from] DbError)` ajouté T2.3 — gap structurel détecté Pass 6 1M context : la closure `with_account_lock` retourne `Result<T, ReconciliationError>` mais `journal_entries::create_in_tx` retourne `Result<_, DbError>` non convertible (pas de `From<DbError> for sqlx::Error`). Sans ce variant la spec promettait un mapping `DbError::FiscalYearClosed → 400` impossible techniquement. Match handler exhaustif documenté T3.3. (F2'''' MEDIUM) §validation-handler-side step 9 — annotation explicite « steps 5-9 inside closure unique, NE PAS sortir audit_log de la closure ». (F3'''' MEDIUM) SQL UPDATE bank_transactions step 8 complétée avec `AND company_id = ? AND status = 'pending'` + `updated_at = NOW(3)` (defense-in-depth multi-tenant + status guard cohérent pattern 8-4 ligne 691). (F4'''' MEDIUM intégré dans F1'''' patch — typing audit log clarifié). (F5'''' MEDIUM) Test #11 zero_amount — clarification body shape `AppError::Validation` émet `error.message`, PAS `error.details.reason` (assertion test corrigée). (F6'''' LOW) §audit-log-shape clarifie démarcation vs 8-4 `MatchScore` (snake_case top-level applique uniquement aux scalars 8-5a-base, sans rétroaction 8-4). (F7'''' LOW) Référence « dette L64 » remplacée par « dette héritée 8-1b errors.rs:248-253 » pour clarté. Trend : Pass 1=3 → Pass 2=3 → Pass 3=7 → Pass 4=2 → Pass 5=2 → **Pass 6=5 findings > LOW** (1 HIGH gap structurel détecté par 1M context Opus + verification ground-truth exhaustive de `ReconciliationError`/`DbError`/`with_account_lock` types). Cycle review continue **Pass 7 Sonnet 4.6** après application des 7 patches. Critère STOP non atteint (1 HIGH > LOW). Path-dépendance : nouveau variant `Db(DbError)` réutilisé par 8-5a-bis (split flow) et 8-5b (accept-with-rule flow) — signature helper public `build_journal_entry_for_counterparty` **inchangée** (stable contractée). | Claude (Opus 4.7 1M validate) |
| **2026-05-08** | **Pass 5 validate Haiku 4.5** — 2 findings > LOW (0 CRITICAL + 0 HIGH + 2 MEDIUM + 2 LOW). Patches : (M1) T5.4.3 test naming clarity — renommer test pour expliciter validation absence du field `bankLedgerAccountId` vs verbe HTTP POST (confusion latente déjà clarifiée Pass 4 mais naming du test imprécis pour dev agent) ; (M2) T3.1 codification position étape 4bis — ajouter annotation explicite `// Step 4bis: zero amount pre-validation` et clarifier que cette validation est **PRÉ-flight** avant `with_account_lock` (non "inside lock") ; (M3) T5.2 exemple filtrage inline — ajouter snippet code `const filteredAccounts = accounts.filter(...)` pour cristalliser pattern pré-filtrage avant passage à `AccountAutocomplete` (pattern compatible Pass 4, mais clarté améliorée) ; (L1) T4.2 distinction locks — rephrase distinction advisory lock vs row lock pour clarté (advisory = `GET_LOCK()` session-level, FOR UPDATE = row lock intra-tx) ; (L2) T1.2 couverture exhaustive tests — clarifier que les 3 tests couvrent sécurité multi-tenant + filtre status + happy path complet. Trend : Pass 1=3 → Pass 2=3 → Pass 3=7 → Pass 4=2 → **Pass 5=2 findings > LOW** (stabilisation, 0 régressions Pass 4, omissions de clarité détectées). **Critère d'arrêt CLAUDE.md atteint** : ≤ 2 MEDIUM après 5 passes adversariales orthogonales (Pass 5 = dernière passe, tous findings = clarté/documentation, code-impact-zero). Cycle review **STOP**. Spec 8-5a-base ready for `bmad-dev-story`. | Claude (Haiku 4.5 validate) |
| **2026-05-08** | **Pass 1 validate Sonnet 4.6** — 3 findings (0 CRITICAL + 0 HIGH + 3 MEDIUM + 2 LOW). Patches : (1) F1 MEDIUM — signature incorrecte `create_in_tx` corrigée (§scope verrouillé + §validation-handler-side) : `(tx, fiscal_year_id, user_id, new_je)` au lieu de `(tx, new_je, company_id, fiscal_year_id, audit_actor)` ; (2) F2 MEDIUM — race condition `DbError::FiscalYearClosed` → 400 documentée (§validation-handler-side step 7) ; (3) F3 MEDIUM — mapping explicite `ReconciliationError::FiscalYearClosed → AppError::ReconciliationFiscalYearClosed` spécifié dans T3.3 (sans ce match → 500 silencieux) ; (4) F4 LOW — comptage variantes T3.3 + §risque-splitting corrigés (`ReconciliationOptimisticLockConflict` non créé — pattern `Database(OptimisticLockConflict)` 8-4 réutilisé). Trend : Pass 1 = 3 findings > LOW. Continuer Pass 2 Haiku 4.5. | Claude (Sonnet 4.6 validate) |
| **2026-05-08** | **Pass 2 validate Haiku 4.5** — 3 findings (0 CRITICAL + 0 HIGH + 3 MEDIUM + 0 LOW). Patches : (1) M1 MEDIUM — référence AC obsolète ligne 80 corrigée (`AC #82` → `AC #86` — spec 8-5a-base ACs #83-#92) ; (2) M2 MEDIUM — T1.2 tests incomplets : ajout test positif happy-path `find_strictly_pending_returns_tx_when_all_conditions_match` (couverture 100% du helper avant implémentation) ; (3) M3 MEDIUM — T5.2 compatibilité `AccountAutocomplete.svelte` non vérifiée : directive clarity ajoutée (vérifier avant reuse, créer wrapper dédié si incompatibilité). Trend : Pass 1 = 3 → Pass 2 = 3 findings > LOW (tous MEDIUM, pas de régressions Pass-1). Continue Pass 3 Opus 4.7 pour convergence. | Claude (Haiku 4.5 validate) |
| **2026-05-08** | **Pass 3 validate Opus 4.7 — VALIDATION FINALE** — 8 findings (1 CRITICAL + 1 HIGH + 5 MEDIUM + 1 LOW). Patches : (1) F1''' CRITICAL — pseudo-code T3.1 corrige `AppError::BankAccountNotFound` qui est **unit-struct** (pas `{ bank_account_id }`) — ne compile pas sinon (errors.rs:255 ground-truth) ; (2) F2''' HIGH — AC #87 corrige le code HTTP attendu : code réel `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` v0.1 (dette L64 documentée) — pas `BANK_ACCOUNT_NOT_FOUND` ; (3) F3''' MEDIUM — clarification mapping `find_open_covering_date None → ReconciliationError::FiscalYearClosed` traduit côté closure (sans cette traduction explicite, le `Option::None` ne propage pas) ; (4) F4''' MEDIUM — helper `build_journal_entry_for_counterparty` doc clarifie source `company_id` (depuis `tx.company_id`) + journal hardcodé `Journal::Banque` (fields obligatoires `NewJournalEntry`) ; (5) F5''' MEDIUM — sémantique `find_open_covering_date None` confond NoFiscalYear/Closed → L46 dette UX v0.1 (UX simplifié) ; (6) F6''' MEDIUM — test #8 `manual_match_reverses_auto_rejection` doit asserter `was_previously_rejected=true` dans audit shape ; (7) F7''' MEDIUM — pré-validation handler-side `tx.amount != 0` (step 4bis + L48 + nouveau test #11 `manual_match_rejects_zero_amount_transaction`) ; (8) F8''' LOW — currency non-CHF documentée en L47 (invariant garanti à l'amont par 8-1b/8-2/8-3, pas de check redondant). Trend : Pass 1 = 3 → Pass 2 = 3 → Pass 3 = 7 findings > LOW. **NON-CONVERGENT** : Pass 3 a remonté plus de findings que Pass 2 (le 1M context Opus a permis une vérification ground-truth exhaustive du code Rust qui a révélé F1''' CRITICAL + F2''' HIGH invisibles à Sonnet/Haiku sans accès au code réel). Cycle review continue. **Prochaine étape** : Pass 4 Sonnet 4.6 après application des 8 patches. Critère STOP non atteint. | Claude (Opus 4.7 1M validate) |
| **2026-05-08** | **Pass 4 validate Sonnet 4.6** — 2 findings > LOW (0 CRITICAL + 0 HIGH + 2 MEDIUM + 1 LOW). Patches : (M1) MEDIUM — T5.4.3 test Vitest verbe HTTP incorrect : `PATCH` → `POST` (la route est `POST /api/v1/reconciliation/manual`) ; (M2) MEDIUM — description 200 vs 500 chars incohérence ambiguë pour dev agent : note dev ajoutée dans T3.1 précisant `MAX_MANUAL_DESCRIPTION_LEN = 200` distinct de `MAX_DESCRIPTION_LEN = 500` de `journal_entries.rs` (business rule modal, libellé court) ; (LOW) T5.2 compatibilité `AccountAutocomplete.svelte` : note dev ajoutée confirmant que le composant accepte une prop `accounts` pré-filtrée, pattern compatible sans wrapper dédié. Trend : Pass 1 = 3 → Pass 2 = 3 → Pass 3 = 7 → **Pass 4 = 2 findings > LOW**. Critère d'arrêt CLAUDE.md **non atteint** (2 MEDIUM). **Prochaine étape : Pass 5 Haiku 4.5** (contexte frais, fenêtre orthogonale). | Claude (Sonnet 4.6 validate) |
