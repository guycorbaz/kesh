# Story 8-5a-bis: FR48 split + breaking POST /accept type discriminator (Q2)

Status: review

<!-- Issue de re-split de Story 8-5a (`8-5a-reconciliation-manuelle-split.md`) le 2026-05-07,
     post-Pass-3 validate Opus 4.7 qui a détecté la dette F2'' (`bank_account.journal_account_id`
     inexistant — anti-pattern UX `bankLedgerAccountId` dans body POST /split).

     Décision Guy 2026-05-07 (option 3) : éviter la dette à la racine en re-splittant 8-5a
     en 3 sous-stories séquentielles.

     Path-dépendance bloquante :
     - 8-5a-zero (`8-5a-zero-bank-account-journal-link.md`) DOIT être `done`/merged sur main
     - 8-5a-base (`8-5a-base-manual-match.md`) DOIT être `done`/merged sur main avant que
       8-5a-bis ne transitionne `backlog → ready-for-dev`. 8-5a-bis :
       (a) consomme `bank_account.journal_account_id` (foundation 8-5a-zero) pour résoudre
           serveur-side le ledger account banque sans body field `bankLedgerAccountId`,
       (b) réutilise le helper public `kesh-reconciliation::manual::build_journal_entry_for_counterparty`
           livré par 8-5a-base (composé pour générer la ligne banque + N lignes contrepartie
           dans le helper split),
       (c) porte le breaking change Q2 `POST /accept` discriminator type='invoice' obligatoire
           (migration de tous les tests E2E HTTP 8-4 actifs).

     Voir `8-5a-reconciliation-manuelle-split.md` (status `archived-split-bis`) pour les
     décisions de conception détaillées validées sur 3 passes Sonnet→Haiku→Opus
     (29 patches cumulés). La section §split-flow est transposée ici sans modification
     (sauf step 5 où `bank_account.linked_account_id` → `bank_account.journal_account_id`
     automatique). M1''' Pass 3 Opus : §error-precedence-order de la spec archivée n'est
     PAS transposée ici — l'ordre de précédence des erreurs est porté par la liste
     ordonnée §validation-handler-side-split (steps 1-13 explicites). -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse, comptable interne ou fiduciaire)**,
I want **éclater une transaction bancaire agrégée (paiement multi-salaires + charges sociales sur 1 paiement, ou encaissement multi-source) en N+1 lignes de journal_entry, sans devoir re-saisir le compte comptable banque (résolu serveur-side via 8-5a-zero), avec une garantie de balance Decimal exact entre la transaction et la somme des splits**,
so that **les transactions agrégées soient comptablement décomposées proprement, et que le breaking change `POST /accept` discriminator type (Q2) soit effectif dans le système — préparant 8-5b (rules engine) qui ajoutera type='rule'**.

### Contexte

**Story 8-5a-bis = sous-story FR48 split + Q2 breaking change du re-split 8-5a → 8-5a-zero / 8-5a-base / 8-5a-bis** (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7).

**Pourquoi 8-5a-bis après 8-5a-base** :
1. 8-5a-bis consomme `bank_account.journal_account_id` (foundation 8-5a-zero) pour résoudre serveur-side le compte ledger banque côté serveur, **sans body field `bankLedgerAccountId`**.
2. 8-5a-bis compose le helper `manual::build_journal_entry_for_counterparty` livré par 8-5a-base (pour la cohérence de signature, le helper split factorise N appels manual + 1 ligne banque agrégée).
3. 8-5a-bis porte le breaking change Q2 (POST /accept discriminator obligatoire), ce qui nécessite la migration de **tous** les 21 tests E2E HTTP 8-4 actifs (via 15 sites POST /accept dans `reconciliation_e2e.rs`). Ce breaking change est porté par 8-5a-bis (et pas 8-5a-base) parce que :
   - Le breaking change est conceptuellement lié à la prolifération des types de proposals (manual / split / rule) — 8-5a-base introduit /manual standalone (pas via /accept), donc pas besoin de breaking 8-5a-base.
   - 8-5b (rules engine) ajoutera `type='rule'` à /accept — c'est le 2e usage du discriminator.
   - 8-5a-bis devient le moment naturel pour poser le breaking, regroupant split + accept type='split' (livré ici) + préparation type='rule' (8-5b).

**8-5a-bis livre la valeur utilisateur immédiate :**
- **FR48** — éclatement de transaction agrégée en N imputations (salaires multiples + charges sociales sur 1 paiement, ou encaissement multi-source).
- **Breaking change Q2** — `POST /accept` discriminator `type` obligatoire (préparation 8-5b rule type).
- **Helper public `kesh-reconciliation::split::*`** réutilisé par le frontend pour validation balance live.

**8-5a-bis ne livre PAS** :
- Foundation column `journal_account_id` (déjà 8-5a-zero — pré-requis).
- FR45 manual match (déjà 8-5a-base — pré-requis).
- Rules engine (8-5b).
- Suggestion ML automatique (FR46 reportée v0.2 — Q5).

**Status sprint** : `8-5a-bis-split-breaking-accept: backlog` au moment de la création (2026-05-07). Transition vers `ready-for-dev` après que 8-5a-base ait clos son cycle review (0 findings > LOW + merged main).

**Pré-requis closed (au moment du démarrage 8-5a-bis)** :
- ✅ **Story 8-5a-zero** — column `bank_account.journal_account_id`, route PATCH, UI configuration.
- ✅ **Story 8-5a-base** — helper public `kesh-reconciliation::manual::build_journal_entry_for_counterparty`, helper repo `find_strictly_pending_by_id_for_account`, route POST `/manual`, audit `reconciliation.manual_matched`.
- ✅ Story 8-4 — `kesh-reconciliation` crate base + `with_account_lock` + audit log helpers.
- ✅ Story 6-2 — multi-tenant scoping pattern KF-002 Pattern 1.
- ✅ Story 5-2 — `journal_entries::create_in_tx`.
- ✅ Story 3-7 — `fiscal_years::find_open_covering_date`.

**Crate cible** : extension de `kesh-reconciliation` avec 1 nouveau module `split` (helper validateur balance + builder N+1 lignes). Le module `manual` existe déjà (livré 8-5a-base) — 8-5a-bis **réutilise le pattern sign-aware** du helper manual sans le composer littéralement (cf. Pass 2 H1 + §helper-split-signature : implémentation directe à N+1 lignes pour éviter N appels manual + fusion). Le module `rules` est livré par 8-5b.

### Scope verrouillé — ce qui est livré par 8-5a-bis

1. **Éclatement de transaction agrégée (FR48)** — nouvelle route `POST /api/v1/reconciliation/split` (sub-router `comptable_routes`).

   **Body simplifié post-8-5a-zero** :
   ```json
   {
     "bankAccountId": 17,
     "bankTransactionId": 42,
     "splits": [
       { "counterpartyAccountId": 5000, "amount": "5000", "description": "Salaire Alice" },
       { "counterpartyAccountId": 5000, "amount": "4500", "description": "Salaire Bob" },
       { "counterpartyAccountId": 5700, "amount": "1200", "description": "Charges" }
     ],
     "valueDate": "2026-05-31"
   }
   ```
   **Pas de `bankLedgerAccountId`** — résolu serveur-side via `bank_accounts::find_by_id_for_company(...).journal_account_id` (foundation 8-5a-zero). Si NULL → 412 `BANK_ACCOUNT_NOT_CONFIGURED`.

   **Validation balance** : `sum(splits[*].amount) === bankTransaction.amount.abs()` (Decimal exact, **pas de tolérance**, validation backend ET frontend live).
   
   **Signe des `splits[*].amount`** (C1 Pass 2 Haiku + C1''' Pass 3 Opus) : tous les montants des splits **DOIVENT être strictement positifs** (`> 0`, montants absolus, indépendants du signe `tx.amount`). Ex. : `tx.amount = -10700` (sortie cash) → `splits = [5000, 4500, 1200]` (tous strictement positifs). Le signe débit/crédit est déterminé par `sign(tx.amount)` au niveau du handler, pas par les montants individuels. Validation pré-flight DOIT rejeter **tout** `splits[i].amount <= 0` avec 400 Validation.

   **Pourquoi `> 0` strict (pas `>= 0`) — décision Pass 3 Opus C1'''** : `splits[i].amount = 0` créerait une ligne `journal_entry_lines` 0/0 (debit=0 ET credit=0) sémantiquement vide. `journal_entries::create_in_tx` ne vérifie PAS `debit > 0 OR credit > 0` (cf. `crates/kesh-db/src/repositories/journal_entries.rs:115-119` qui ne valide que `lines.is_empty()` + balance globale step 6 ligne 212). Une ligne 0/0 passerait donc la balance check mais pollurait les comptes. Le strict `> 0` aligne split sur la précondition `tx.amount != 0` du flow manual (cf. `manual.rs:74-78` `assert!(!tx.amount.is_zero())`).

   **Atomicité** : 1 SEULE `journal_entry` à N+1 lignes (1 ligne banque au montant total + N lignes contreparties), créée atomiquement avec UPDATE `bank_transactions` via `journal_entries::create_in_tx`.

2. **Helper `kesh-reconciliation::split::*`** :
   ```rust
   /// Détail d'un split pour `build_split_journal_entry` (type-safe).
   pub struct SplitDetail {
       pub account_id: i64,
       pub amount: Decimal,
       pub description: String,
   }

   /// Variante N+1 lignes pour split (FR48). Pure (zéro I/O).
   /// Compose le pattern de `manual::build_journal_entry_for_counterparty` pour
   /// générer 1 ligne banque agrégée + N lignes contrepartie.
   pub fn build_split_journal_entry(
       tx: &BankTransaction,
       bank_account_journal_id: i64,
       splits: &[SplitDetail],
       description: String,
       entry_date: NaiveDate,
   ) -> NewJournalEntry

   /// Vérifie sum(splits[*].amount) == tx.amount.abs() Decimal exact.
   pub fn validate_split_balance(tx_amount: Decimal, splits: &[Decimal]) -> Result<(), SplitImbalance>

   pub struct SplitImbalance {
       pub expected: Decimal,
       pub actual: Decimal,
       pub difference: Decimal,
   }
   ```

   **Précondition** : `validate_split_balance` ne valide PAS `splits.len() >= 2` ni `splits.len() <= 50` — le caller (handler `post_split`) DOIT appliquer ces 2 contraintes AVANT d'appeler le helper (cohérent F7'' Pass 3 décision). `tx_amount` est passé brut (signed), `validate_split_balance` applique `.abs()` en interne.

   **Description top-level JE (M3''' Pass 3 Opus)** : le helper `build_split_journal_entry` prend un paramètre `description: String` qui devient `NewJournalEntry.description`. Le body POST `/split` n'a PAS de `description` top-level (chaque split line a sa propre `description`). **Décision Pass 3 Opus** : le handler construit la string top-level comme suit :
   ```rust
   // Construction handler-side (post_split + accept_one_split) :
   let description = format!(
       "Éclatement transaction agrégée ({} lignes)",
       splits.len()
   );
   ```
   Pas de i18n key dédiée v0.1 (admin/audit context, pas user-facing direct). Si le user a besoin de personnaliser, v0.2 ajoutera un body field `description?: string` top-level + clé i18n. Le tracking des descriptions per-line reste dans les `journal_entry_lines` (champs implicites via repository v0.2 — out-of-scope 8-5a-bis).

   **Conversion `SplitImbalance` → `ReconciliationError` (M5''' Pass 3 Opus)** : la pure helper `validate_split_balance` retourne `Result<(), SplitImbalance>` (struct local au module split). Le variant `ReconciliationError::SplitImbalance` est struct-like avec les mêmes champs. **Recommandation** : implémenter `From<SplitImbalance> for ReconciliationError` :
   ```rust
   impl From<SplitImbalance> for ReconciliationError {
       fn from(e: SplitImbalance) -> Self {
           ReconciliationError::SplitImbalance {
               expected: e.expected,
               actual: e.actual,
               difference: e.difference,
           }
       }
   }
   ```
   Permet `validate_split_balance(...)?` direct dans la closure `with_account_lock`. Sans ce `From`, le handler doit `.map_err(|e: SplitImbalance| ReconciliationError::SplitImbalance { ... })` à chaque call-site — verbeux.

3. **Breaking change `POST /api/v1/reconciliation/accept` (Q2 décision Guy 2026-05-07)** — le request body proposals[*] a désormais un discriminator `type` **obligatoire** :
   - `type: 'invoice'` (8-4 héritée) — `{ type: 'invoice', bankTransactionId, invoiceId }`
   - `type: 'split'` (8-5a-bis, équivalent batch de la route `/split` standalone) — `{ type: 'split', bankTransactionId, splits: [...] }`
   - `type: 'manual'` (8-5a-base, équivalent batch de `/manual` standalone) — `{ type: 'manual', bankTransactionId, counterpartyAccountId, description?, valueDate? }`. **Note** : 8-5a-base livre `/manual` route standalone, pas via /accept. **Décision design 8-5a-bis** : ajouter `type: 'manual'` comme valeur acceptée mais non implémentée (cohérent §note-implementation-accept) — réservé futur, retourne 400 si non encore supporté.
   - `type: 'rule'` (8-5b — non livré 8-5a-bis, mais le discriminator est posé pour préparation).

   **Pas de backward-compat** : si `type` absent → `400 Validation` avec message « champ `type` requis, valeurs acceptées v0.1 : ["invoice", "split"] » (Kesh pas en prod, breaking change accepté par décision Guy Q2). Si `type` présent mais non reconnu (`'manual'`, `'rule'` en 8-5a-bis v0.1) → `400 Validation` avec message « type inconnu ou non supporté : "<valeur>", valeurs acceptées v0.1 : ["invoice", "split"] ».

4. **Migration tests E2E HTTP 8-4 existants** (AC #99 / T3.4 — détaillée §migration-21-tests) :
   Le fichier `crates/kesh-api/tests/reconciliation_e2e.rs` contient **21 tests actifs + 1 ignored = 22 attributs `#[sqlx::test]` au total** (vérifié `awk` Pass 3 Opus). 15 de ces tests font un POST `/accept` (vérifié `grep` 2026-05-07 — 15 sites). Les tests doivent ajouter `type: 'invoice'` explicite dans tous les bodies POST /accept (Q2 breaking).

   **Liste exacte des 15 sites POST /accept à patcher** (lignes vérifiées 2026-05-07) :
   1. ligne 757 — dans `post_accept_reconciles_transaction_and_invoice`
   2. ligne 910 — dans `post_accept_handles_partial_failure`
   3. ligne 1030 — dans `post_accept_rejects_unvalidated_or_paid_invoice`
   4. ligne 1104 — dans `post_accept_does_not_leak_cross_tenant_invoice`
   5. ligne 1203 — dans `post_accept_returns_409_on_account_lock_contention`
   6. ligne 1416 — dans `reconciliation_routes_require_comptable_role`
   7. ligne 1492 — dans `post_reject_after_accept_returns_already_reconciled_failed` (setup `accept` avant `reject`)
   8. ligne 1632 — dans `post_accept_filters_signed_amount`
   9. ligne 1679 — dans `post_accept_returns_404_on_cross_tenant_bank_account`
   10. ligne 1737 — dans `post_accept_returns_400_on_cross_account_proposal`
   11. ligne 1805 — dans `post_accept_rejects_payment_date_before_invoice_date`
   12. ligne 1870 — dans `post_accept_emits_dual_audit_invoice_paid`
   13. ligne 1980 — dans `post_accept_skips_non_chf_transaction`
   14. ligne 2064 — dans `post_accept_rejects_zero_score_match`
   15. ligne 2129 — dans `post_accept_rejects_payment_date_outside_window`

   **Note** : `post_accept_filters_currency_mismatch` (ligne 1656) est `#[ignore]` (préservé pour Story 11 mono-CHF v0.1) — son corps fait `panic!("placeholder")` sans appel `POST /accept`, donc **rien à patcher**. Cf. §migration-21-tests « Pas de 16ème site (F6 Pass 1) » pour le détail. **Total 15 sites à patcher** (15 actifs ; 1 ignored sans appel `/accept`).

   Pour chaque site, ajouter `type: 'invoice'` dans le body :
   ```rust
   // Avant (8-4)
   .json(&json!({ "proposals": [{ "bankTransactionId": tx_id, "invoiceId": inv_id }] }))
   // Après (8-5a-bis)
   .json(&json!({ "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }] }))
   ```

5. **Audit log action `reconciliation.split_applied`** (Q4a — action distincte, cohérent décision Guy 2026-05-07) :
   ```json
   {
     "bank_transaction_id": 42,
     "splits": [
       { "counterparty_account_id": 5000, "amount": "5000.00", "description": "Salaire Alice" },
       { "counterparty_account_id": 5000, "amount": "4500.00", "description": "Salaire Bob" },
       { "counterparty_account_id": 5700, "amount": "1200.00", "description": "Charges" }
     ],
     "total_amount": "10700.00",
     "journal_entry_id": 999,
     "value_date": "2026-05-31",
     "was_previously_rejected": false
   }
   ```
   100% snake_case top-level (cohérent F4'' Pass 3 décision Opus). Pas de sous-objets typés camelCase. **Note clés sub-objets `splits[i]`** : également snake_case (cohérent §audit-log-shape 8-5a-base).

6. **Frontend extensions** :
   - Composant `TransactionSplitModal.svelte` (nouveau) : tableau de splits éditable (ajout/suppression de ligne, min 2 max 50) + indicateur balance live (sum vs `|tx.amount|`, vert si exact match, rouge sinon, submit désactivé tant que balance ≠ exact). **Pas de dropdown ledger account banque** — résolu serveur-side via 8-5a-zero.
   - Extension de `ReconciliationProposals.svelte` (héritée 8-4 + 8-5a-base) : 1 bouton supplémentaire par ligne tx sans candidate : « Éclater » (ouvre `TransactionSplitModal`).
   - **Migration** `acceptProposal` : ajouter `type: 'invoice'` explicite dans le body envoyé via `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (cohérent breaking Q2). Met à jour aussi `ReconciliationProposals.svelte` qui consomme.

7. **API client frontend** : nouvelle fonction `splitTransaction` :
   ```ts
   // F5 Pass 1 : description?: string top-level retiré (sans contrepartie body Rust).
   export async function splitTransaction(
       bankAccountId: number,
       bankTransactionId: number,
       splits: { counterpartyAccountId: number; amount: string; description: string }[],
       valueDate?: string,
   ): Promise<{ bankTransactionId: number; journalEntryId: number }>;
   ```
   **Pas de `bankLedgerAccountId`** — résolu serveur-side. Différence majeure vs spec 8-5a unifiée.

8. **i18n** : ~5 nouvelles clés `reconciliation-split-*` × 4 locales fr/de/it/en-CH. **Pas** les clés `reconciliation-rules-*` (8-5b).

9. **Tests** :
   - Unit `kesh-reconciliation::split` (≥ 4 cas : N+1 lignes credit + debit, balance valid + invalid).
   - Integration `kesh-db` (≥ 1 sqlx pour helper si nécessaire — réutilise principalement helpers 8-4 + 8-5a-base).
   - E2E HTTP `kesh-api` :
     - **Nouveaux tests 8-5a-bis** (≥ 10) : 8 split (AC #93-#99) + 2 accept-discriminator (AC #100).
     - **Migration 21 tests actifs 8-4** : ajouter `type: 'invoice'` dans 15 sites POST /accept (cf. §migration-21-tests).
   - Vitest (≥ 3-4 : split modal balance live + button + api migration).
   - Playwright (≥ 1 actif + 1 a11y).

10. **Sync** sprint-status — pas de KF/CR pré-tracée.

**HORS scope 8-5a-bis (→ 8-5b / v0.2) :**

- Table `reconciliation_rules` + repos + migrations (8-5b T1+T2)
- Routes CRUD `/api/v1/reconciliation/rules` (8-5b T4)
- Engine d'application des règles dans GET /proposals + POST /accept type='rule' (8-5b)
- Audit log actions `reconciliation_rule.{created,updated,deleted,applied}` (8-5b)
- Page frontend `/reconciliation/rules` + composant `RuleFormModal` (8-5b)
- Suggestion ML « voulez-vous créer une règle ? » (reportée v0.2 — décision Guy Q5)
- Auto-acceptation des règles à fort score (reporté v0.2)
- Annulation de réconciliation (reporté v0.2 L45)
- Liaison split → invoices multiples (reporté v0.2 L44)

### Décisions de conception

#### §split-flow

**Validation balance Decimal exact** : `sum(splits[*].amount) == tx.amount.abs()` strict (pas de tolérance). Le helper `validate_split_balance` retourne `Err(SplitImbalance)` mappé `400 RECONCILIATION_SPLIT_IMBALANCE` body :
```json
{
  "error": {
    "code": "RECONCILIATION_SPLIT_IMBALANCE",
    "message": "...",
    "details": {
      "expected": "10700.00",
      "actual": "10500.00",
      "difference": "-200.00"
    }
  }
}
```

**Min/max splits** : `2 <= splits.len() <= 50`. Validation handler-side AVANT `validate_split_balance` (cohérent F7'' Pass 3 précondition longueur).
- `< 2` → 400 Validation « splits doit contenir ≥ 2 lignes — utilisez /manual pour 1 ligne ».
- `> 50` → 400 Validation « splits ≤ 50 lignes ».

**Pas de table `bank_transaction_splits` séparée** : la décomposition est portée par la `journal_entry` à N+1 lignes (SSOT comptable, pas de duplication). Cohérent §split-flow spec 8-5a unifiée.

**Atomicité** : 1 seule transaction DB pour : (a) UPDATE `bank_transactions.status='reconciled'`, (b) INSERT `journal_entries` à N+1 lignes via `create_in_tx`, (c) audit log `reconciliation.split_applied` 1 entrée + audit log `journal_entry.created` (émis automatiquement par `create_in_tx`).

#### §helper-split-signature

**Décision** : `build_split_journal_entry` factorise le pattern `manual::build_journal_entry_for_counterparty` (8-5a-base) :
- 1 ligne banque agrégée (compte `bank_account_journal_id`, montant `tx.amount.abs()`, sign opposé aux N lignes contreparties — la banque est crédit pour `tx.amount < 0` (sortie cash) et débit pour `tx.amount > 0` (entrée cash) ; cohérent partie double).
- N lignes contreparties (chaque `splits[i]` → 1 ligne, sign cohérent avec `sign(tx.amount)` — toutes débit pour sortie cash, toutes crédit pour entrée cash).

**Rationale** : factorisation maximale du pattern. Le helper `build_journal_entry_for_counterparty` peut être appelé en boucle par `build_split_journal_entry` pour chaque ligne contrepartie, puis combiné en 1 NewJournalEntry à N+1 lignes. **Note implémentation** : le dev agent peut choisir de NE PAS littéralement composer (c'est-à-dire générer N NewJournalEntry indépendantes puis les fusionner) — c'est inefficace, donc préférer une implémentation directe à N+1 lignes dans `build_split_journal_entry` qui réutilise les sous-fonctions sign-resolver de manual.rs.

#### §validation-handler-side-split

**Ordre de validation pré-flight** (avant `with_account_lock`) :
1. Body validation Serde camelCase. `splits.len() >= 2 && splits.len() <= 50`.
1bis. **Validation longueur description (M4''' Pass 3 Opus)** : pour chaque `splits[i]`, vérifier `splits[i].description.chars().count() <= 200`. Toute description > 200 chars → 400 Validation « splits[i].description trop longue (max 200 caractères) ». Cohérent avec `MAX_MANUAL_DESCRIPTION_LEN = 200` (reconciliation.rs:1074) + cap frontend ligne 353 modal. Defense-in-depth backend : le frontend cap mais un client API direct pourrait bypass.
2. **Validation sign splits (C1 Pass 2 + C1''' Pass 3 Opus — strict `> 0`)** : pour chaque `splits[i]`, vérifier `splits[i].amount > Decimal::ZERO`. Tout montant négatif OU zéro → 400 Validation « splits[i].amount doit être strictement positif (> 0) ». Cf. §scope point 1 (« Signe des splits[*].amount ») pour rationale ligne 0/0 dans journal_entry.
3. `bankAccountId` cross-tenant : 404 `BANK_ACCOUNT_NOT_FOUND`.
4. `bank_account.journal_account_id IS Some(...)` : 412 `BANK_ACCOUNT_NOT_CONFIGURED`.
5. **Validation accounts batch** : pour chaque `splits[i].counterpartyAccountId`, vérifier `accounts::find_by_id_in_company` + `active=true`. Pattern : batch-load via une seule query `IN (...)` (cohérent `find_pending_by_ids` pattern reconciliation.rs:233) ou itération séquentielle si volume ≤ 50 (cap §split-flow inchangé). Tout split référençant un account inexistant ou archivé → `404 ACCOUNT_NOT_FOUND` body `details.missingAccountIds: [<ids>]` (camelCase JSON, list distincts triés).
6. `bankTransactionId` strictement pending : `find_strictly_pending_by_id_for_account` (helper 8-5a-base) → 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` si None.
6bis. **(M2''' Pass 3 Opus) Pré-validation `tx.amount != 0`** : si `tx.amount.is_zero()` → 400 `Validation` message `"zero_amount_transaction"` (cohérent post_manual step 4bis reconciliation.rs:1218-1224). Sans cette précondition, une tx d'amount 0 + splits=[0,0] passerait `validate_split_balance` (sum=0=abs(0)) et créerait N+1 lignes 0/0 dans le JE (sémantiquement vides, polluent les comptes). Note : avec C1''' (`splits[i].amount > 0` strict), splits=[0,0] est déjà rejeté step 2 → mais step 6bis est defense-in-depth pour le cas où step 2 serait bypassed par bug futur ou variant non-prévu.
7. `validate_split_balance(tx.amount, &splits.iter().map(|s| s.amount).collect::<Vec<_>>())` → 400 `RECONCILIATION_SPLIT_IMBALANCE` si Err. **(F7 Pass 1 : `validate_split_balance` attend `&[Decimal]`, pas un `Iterator` — il faut `.collect()` avant passage.)**

**Inside lock (advisory `with_account_lock`)** :
8. Re-fetch tx (TOCTOU defense pattern 8-4).
9. Resolve `entry_date` + `find_open_covering_date` → 409 `RECONCILIATION_FISCAL_YEAR_CLOSED`.
10. Build `NewJournalEntry` via `split::build_split_journal_entry`.
11. `journal_entries::create_in_tx`.
12. UPDATE bank_transactions optimistic lock + **reset `auto_match_rejected_at = NULL`** (M3 Pass 2) → 409 si race.
13. Audit log `reconciliation.split_applied`.

#### §audit-log-shape

100% snake_case top-level + sub-objects. Cohérent F4'' Pass 3 décision Opus.

```json
{
  "bank_transaction_id": 42,
  "splits": [
    { "counterparty_account_id": 5000, "amount": "5000.00", "description": "Salaire Alice" },
    { "counterparty_account_id": 5000, "amount": "4500.00", "description": "Salaire Bob" },
    { "counterparty_account_id": 5700, "amount": "1200.00", "description": "Charges" }
  ],
  "total_amount": "10700.00",
  "journal_entry_id": 999,
  "value_date": "2026-05-31",
  "was_previously_rejected": false
}
```

#### §note-implementation-accept

**Décision Pass 3 Opus + F2''' Pass 3 Opus (clarification)** : enum tagged `AcceptProposalInput` (auto-discriminator Serde via `#[serde(tag = "type")]`). 8-5a-bis livre 2 variantes (`Invoice`, `Split`) ; 8-5b ajoutera `Rule` ; `Manual` reporté v0.2 si demande utilisateur d'unifier /manual sur /accept. `String` libre rejeté car break diff-hostile + risque DoS.

**Pas d'enum `AcceptType` séparé (F8 Pass 1 confirmé Pass 3)** : le dispatch se fait directement par pattern matching sur les variantes `AcceptProposalInput`. Un enum `AcceptType { Invoice, Split }` parallèle serait dead code (Clippy `-D warnings` → CI rouge) — ne PAS créer.

**`AcceptProposalInput` après 8-5a-bis** :

```rust
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AcceptProposalInput {
    #[serde(rename = "invoice")]
    Invoice {
        bank_transaction_id: i64,
        invoice_id: i64,
    },
    #[serde(rename = "split")]
    Split {
        bank_transaction_id: i64,
        splits: Vec<SplitProposalLine>,
        // M6''' Pass 3 Opus : `value_date` optional (cohérence avec /split standalone).
        // Si absent → handler default = `body.value_date.or(tx.value_date).unwrap_or(tx.booking_date)`
        // (3 couches cohérent post_manual reconciliation.rs:1281).
        #[serde(default)]
        value_date: Option<chrono::NaiveDate>,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SplitProposalLine {
    pub counterparty_account_id: i64,
    pub amount: Decimal,
    pub description: String,
}
```

**Impact `Copy`** : F11'' Pass 3 — `AcceptProposalInput` perd `Copy` car `Vec` (et `Decimal` interne) n'est pas `Copy`. Solution : retirer `Copy`, ajouter `clone()` aux usages internes. Le test `accept_rejects_proposal_missing_type_discriminator` couvre serde rejection (Serde rejette si `type` absent → 400/422 sérialisation).

**F3''' Pass 3 Opus — Refactor `accept_one` non trivial (~270 lignes touchées)** : `accept_one` (reconciliation.rs:528-918) accède **29 fois** aux champs `proposal.bank_transaction_id` et `proposal.invoice_id` (vérifié `grep -c "proposal\.\(bank_transaction_id\|invoice_id\)" reconciliation.rs` = 29). Avec l'enum tagged, ces accès directs cassent (les variantes n'ont pas la même shape).

**Pattern recommandé Pass 3 Opus** : extraire un destructure top-level + dispatch :
```rust
async fn accept_one(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    proposal: &AcceptProposalInput,
    batch_size: i64,
) -> Result<AcceptedProposal, FailedProposal> {
    match proposal {
        AcceptProposalInput::Invoice { bank_transaction_id, invoice_id } => {
            accept_one_invoice(tx, company_id, bank_account_id, user_id, *bank_transaction_id, *invoice_id, batch_size).await
        }
        AcceptProposalInput::Split { bank_transaction_id, splits, value_date } => {
            accept_one_split(tx, company_id, bank_account_id, user_id, *bank_transaction_id, splits, *value_date, batch_size).await
        }
    }
}
```

Puis `accept_one_invoice` = corps actuel d'`accept_one` (sans changement, juste passage par paramètre vs `proposal.bank_transaction_id`). `accept_one_split` = nouveau helper équivalent `/split` standalone (validation balance + N+1 lignes JE + audit `reconciliation.split_applied`). Coût refactor estimé : ~50 lignes touchées dans `accept_one_invoice` (29 sites de remplacement `proposal.X` → `X`) + ~150 lignes nouvelles `accept_one_split` (équivalent batch de `post_split` body).

**H1 Pass 4 — Résolution `bank_account.journal_account_id` dans `accept_one_split` (inside lock)** : `accept_one_split` reçoit `bank_account_id` mais doit résoudre `bank_account.journal_account_id` pour appeler `split::build_split_journal_entry`. Décision : faire le lookup `bank_accounts::find_by_id_for_company` **inside la closure** `accept_one_split` (SELECT non-mutant, OK à l'intérieur du lock advisory). Si `journal_account_id IS NULL` → retourner `FailedProposal { bank_transaction_id, error_code: "BANK_ACCOUNT_NOT_CONFIGURED", details: None }` (cohérent pattern `FailedProposal` per-proposal, PAS un `AppError::BankAccountNotConfigured` global qui casserait le batch loop). Signature recommandée `accept_one_split` inchangée (pas de paramètre `bank_account_journal_id` supplémentaire — le lookup est encapsulé dans la fonction). Pattern :

```rust
async fn accept_one_split(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    bank_transaction_id: i64,
    splits: &[SplitProposalLine],
    value_date: Option<chrono::NaiveDate>,
    batch_size: i64,
) -> Result<AcceptedProposal, FailedProposal> {
    // Résolution journal_account_id inside lock (SELECT non-mutant)
    let bank_account = match bank_accounts::find_by_id_for_company(&mut **tx, company_id, bank_account_id).await {
        Ok(Some(ba)) => ba,
        _ => return Err(FailedProposal { bank_transaction_id, error_code: "BANK_ACCOUNT_NOT_FOUND".to_string(), details: None }),
    };
    let bank_account_journal_id = match bank_account.journal_account_id {
        Some(id) => id,
        None => return Err(FailedProposal { bank_transaction_id, error_code: "BANK_ACCOUNT_NOT_CONFIGURED".to_string(), details: None }),
    };
    // ... suite identique au flow post_split inside lock (steps 8-13)
}
```

**Validations inside `accept_one_split` à répliquer (M2 Pass 4)** : Les validations pré-flight du body JSON (steps 1, 1bis, 2) sont garanties par le custom extractor AcceptBody + dispatch Serde → `SplitProposalLine` (camelCase). `accept_one_split` DOIT néanmoins répliquer :
- **Step 1bis** (description ≤ 200 chars) : oui — un client API peut envoyer n'importe quel JSON valide.
- **Step 2** (amount > 0 strict) : oui — même raison.
- **Step 6bis** (tx.amount != 0) : oui — après TOCTOU re-fetch inside lock, `tx.amount` est immutable mais le check est defense-in-depth.
- **Step 7** (`validate_split_balance`) : oui — obligatoire.
Ces validations retournent `FailedProposal { error_code: "VALIDATION_ERROR" | "RECONCILIATION_SPLIT_IMBALANCE", details: Some(serde_json::json!({ "reason": "..." })) }` (pas des `AppError` globaux).

**Note implémentation /accept type='split'** (H4 Pass 2) : 8-5a-bis peut soit (a) implémenter le flow `type='split'` complet dans `post_accept` (équivalent batch de `/split` standalone), soit (b) ne supporter que `type='invoice'` dans `/accept` et garder `/split` standalone. **Décision préférée 8-5a-bis** : option (a) — pour cohérence batch (le user peut accepter plusieurs proposals invoice + split en 1 POST) et pour éviter asymmetry UX (discriminator accepte `type='split'` au Serde mais retourne 400 métier). Si volume implémentation trop large (> 100 lignes code), option (b) acceptable avec `type='split'` retourne 400 « non supporté v0.1, utiliser /reconciliation/split ».

#### §rbac

Sub-router `comptable_routes` (Comptable+). `Consultation` retourne 403 (cohérent toutes mutations reconciliation 8-4 + 8-5a-base).

#### §frontend-flow

**Modal `TransactionSplitModal.svelte`** ouvert depuis `ReconciliationProposals.svelte` :
- Click bouton « Éclater » sur une ligne tx pending sans candidate auto-matchée.
- Modal fields :
  - Tableau splits éditable :
    - Chaque ligne : sélecteur Account autocomplete (filtré client-side classes 5/6/7) + input amount Decimal + textarea description (200 chars).
    - Boutons « + ajouter ligne » / « - retirer ligne » (min 2, max 50).
  - Sticker balance live computed `sum vs |tx.amount|` :
    - Vert si `sum === |tx.amount|` (exact match Decimal).
    - Rouge sinon, avec différence affichée.
  - Bouton submit désactivé tant que balance ≠ exact OU `splits.length < 2`.
- Submit → `splitTransaction(...)` → event `success` → refresh liste proposals.
- Gestion erreur `412 BANK_ACCOUNT_NOT_CONFIGURED` : afficher message + lien vers `/bank-accounts`.

**Migration `acceptProposal`** : modifier dans `reconciliation.api.ts` pour ajouter `type: 'invoice'` explicite dans le body envoyé par `acceptProposal`. Met à jour aussi `ReconciliationProposals.svelte` qui consomme. Cohérent breaking Q2.

#### §migration-21-tests

**Liste exacte des 15 sites POST /accept à patcher** (vérifiée 2026-05-07 via `grep -nE "\.post.*reconciliation/accept" reconciliation_e2e.rs`) :

| # | Ligne | Test fonction parent | Notes |
|---|-------|---------------------|-------|
| 1 | 757 | `post_accept_reconciles_transaction_and_invoice` | Happy path 8-4 |
| 2 | 910 | `post_accept_handles_partial_failure` | Partial success batch |
| 3 | 1030 | `post_accept_rejects_unvalidated_or_paid_invoice` | Invoice state guard |
| 4 | 1104 | `post_accept_does_not_leak_cross_tenant_invoice` | Cross-tenant 404 |
| 5 | 1203 | `post_accept_returns_409_on_account_lock_contention` | Advisory lock |
| 6 | 1416 | `reconciliation_routes_require_comptable_role` | RBAC test (multi-route) |
| 7 | 1492 | `post_reject_after_accept_returns_already_reconciled_failed` | Setup `accept` avant `reject` |
| 8 | 1632 | `post_accept_filters_signed_amount` | Sign filter |
| 9 | 1679 | `post_accept_returns_404_on_cross_tenant_bank_account` | Cross-tenant bank_account |
| 10 | 1737 | `post_accept_returns_400_on_cross_account_proposal` | Cross-account proposal |
| 11 | 1805 | `post_accept_rejects_payment_date_before_invoice_date` | Date guard before |
| 12 | 1870 | `post_accept_emits_dual_audit_invoice_paid` | Audit log dual |
| 13 | 1980 | `post_accept_skips_non_chf_transaction` | Currency guard |
| 14 | 2064 | `post_accept_rejects_zero_score_match` | Score guard |
| 15 | 2129 | `post_accept_rejects_payment_date_outside_window` | Date guard outside |

**Pas de 16ème site (F6 Pass 1)** : le test ignoré `post_accept_filters_currency_mismatch` ne contient **aucun** appel `POST /accept` — il fait immédiatement `panic!("placeholder...")`. Il n'y a que **15 sites** à patcher (ground-truth `grep -c "reconciliation/accept" reconciliation_e2e.rs` = 15). La mention précédente « 16 sites (15 actifs + 1 ignored body placeholder) » était incorrecte.

**Pattern de patch** :
```rust
// AVANT (8-4)
.json(&json!({ "proposals": [{ "bankTransactionId": tx_id, "invoiceId": inv_id }] }))

// APRÈS (8-5a-bis)
.json(&json!({ "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": inv_id }] }))
```

**Validation** : `cargo test -p kesh-api --test reconciliation_e2e` doit retourner **21 verts + 1 ignored** (= 22 attributs `#[sqlx::test]`) après patch (régression non introduite). Vérification ground-truth Pass 3 Opus 4.7 : `awk '/^#\[sqlx::test\]/' reconciliation_e2e.rs | wc -l` = 22 (non-ignored = 21).

## Acceptance Criteria

ACs #93-#100 (8 ACs).

### Éclatement de transaction agrégée (FR48)

93. **(FR48 — split happy paiement)** Given tx pending débit `-10700.00` sur `bank_account_id=17` ET `bank_account.journal_account_id=1020` configuré (8-5a-zero) ET les comptes 5000 et 5700 actifs, body `{ bankAccountId: 17, bankTransactionId: 42, splits: [{ counterpartyAccountId: 5000, amount: '5000', description: 'Salaire Alice' }, { counterpartyAccountId: 5000, amount: '4500', description: 'Salaire Bob' }, { counterpartyAccountId: 5700, amount: '1200', description: 'Charges' }] }`, When POST `/split`, Then `200 OK` body `{ bankTransactionId: 42, journalEntryId: 999 }` ET 1 journal_entry à 4 lignes (1020 crédit 10700 + 5000 débit 5000 + 5000 débit 4500 + 5700 débit 1200) ET `bank_transactions.status='reconciled'`, `auto_match_rejected_at=NULL`. *Test E2E HTTP : `split_creates_journal_entry_with_n_plus_1_lines`.*

94. **(FR48 — split happy encaissement)** Given tx pending crédit `+5000.00`, body `splits: [{ counterpartyAccountId: 7510, amount: '3000', description: 'Intérêts' }, { counterpartyAccountId: 6900, amount: '2000', description: 'Remboursement frais' }]`, When POST `/split`, Then journal_entry à 3 lignes (1020 débit 5000 + 7510 crédit 3000 + 6900 crédit 2000). *Test E2E HTTP : `split_creates_journal_entry_for_credit_transaction`.*

95. **(FR48 — split balance violation)** Given tx `-10700.00`, splits sum=10500 (200 missing), When POST split, Then `400 RECONCILIATION_SPLIT_IMBALANCE` body `details = { expected: '10700.00', actual: '10500.00', difference: '-200.00' }`. *Test E2E HTTP : `split_rejects_imbalanced_payload`.*

96. **(FR48 — split min 2 lignes + max 50)** Given splits.len=1, When POST, Then `400 Validation` (« splits doit contenir ≥ 2 lignes »). ET Given splits.len=51, When POST, Then `400 Validation` (« splits ≤ 50 lignes »). *Tests E2E HTTP : `split_rejects_single_line_payload` + `split_rejects_too_many_lines`.*

97. **(FR48 — split bank_account non configuré 412)** Given `bank_account.journal_account_id IS NULL`, When POST split, Then `412 BANK_ACCOUNT_NOT_CONFIGURED`. *Test E2E HTTP : `split_rejects_unconfigured_bank_account_with_412`.*

98. **(FR48 — split multi-tenant safety + RBAC)** Given un `splits[i].counterpartyAccountId` appartient à company_B, When POST, Then `404 ACCOUNT_NOT_FOUND` body `details.missingAccountIds = [<id>]` (camelCase JSON, cohérent convention kesh-api). ET Given user `Consultation`, When POST split, Then `403 Forbidden`. *Tests E2E HTTP : `split_does_not_leak_cross_tenant_account` + `split_requires_comptable_role`.*

99. **(FR48 — split audit log + already reconciled)** Given POST split happy 3 lignes (comptes 5000, 5000, 5700), When commit, Then audit_log contient 2 entrées : `(action='reconciliation.split_applied', entity_type='bank_transaction', entity_id=tx.id, details = { bank_transaction_id, splits: [{ counterparty_account_id, amount, description }, ...], total_amount: '10700.00', journal_entry_id, value_date, was_previously_rejected })` (structure cohérente §audit-log-shape, snake_case top-level + sub-objects) ET `(action='journal_entry.created', entity_type='journal_entry', entity_id=<new_je_id>)` émis par `journal_entries::create_in_tx`. ET Given tx déjà `reconciled`, When POST split, Then `404 RECONCILIATION_TRANSACTION_NOT_PENDING`. *Tests E2E HTTP : `split_emits_audit_log` + `split_rejects_already_reconciled`.*

### Breaking change `POST /accept` discriminator type (Q2 décision Guy)

100. **(Q2 — discriminator type obligatoire + migration tests 8-4)** Given un body proposal `{ bankTransactionId, invoiceId }` (legacy 8-4 sans `type`), When POST `/accept`, Then `400 Validation` avec message « champ `type` requis » (breaking change v0.1, pas de défaut). ET Given body `{ type: 'invoice', bankTransactionId, invoiceId }`, When POST `/accept`, Then flow 8-4 invoice exécuté (audit `reconciliation.accepted` + `invoice.paid`). ET Given le fichier `crates/kesh-api/tests/reconciliation_e2e.rs` (21 actifs + 1 ignored = 22 attributs `#[sqlx::test]`, vérification ground-truth Pass 3 Opus 4.7), When 8-5a-bis livré, Then les **15 sites POST /accept actifs** ajoutent `type: 'invoice'` dans leur body et restent verts (régression non introduite ; le test ignored `post_accept_filters_currency_mismatch` n'a aucun appel `/accept` à patcher — F6 Pass 1). *Tests E2E HTTP : `accept_rejects_proposal_missing_type_discriminator` + `accept_with_explicit_invoice_type_runs_8_4_flow` + cargo `cargo test -p kesh-api --test reconciliation_e2e` 21 verts + 1 ignored.*

## Tasks / Subtasks

### T1. Helper `kesh-reconciliation::split::*` (AC #93-#96)

- [x] T1.1 — Créer `crates/kesh-reconciliation/src/split.rs` (cf. §helper-split-signature) :
  ```rust
  use kesh_db::entities::journal_entry::{NewJournalEntry, NewJournalEntryLine};
  use kesh_db::entities::bank_transaction::BankTransaction;
  use rust_decimal::Decimal;
  use chrono::NaiveDate;

  /// Détail d'un split pour `build_split_journal_entry` (type-safe).
  pub struct SplitDetail {
      pub account_id: i64,
      pub amount: Decimal,
      pub description: String,
  }

  /// Variante N+1 lignes pour split (FR48). Pure (zéro I/O).
  /// Compose le pattern de `manual::build_journal_entry_for_counterparty` (8-5a-base).
  pub fn build_split_journal_entry(
      tx: &BankTransaction,
      bank_account_journal_id: i64,
      splits: &[SplitDetail],
      description: String,
      entry_date: NaiveDate,
  ) -> NewJournalEntry { ... }

  /// Vérifie sum(splits[*].amount) == tx.amount.abs() Decimal exact.
  /// Précondition : caller DOIT vérifier splits.len() ∈ [2, 50] AVANT.
  /// `tx_amount` signed brut ; .abs() appliqué en interne.
  pub fn validate_split_balance(tx_amount: Decimal, splits: &[Decimal]) -> Result<(), SplitImbalance> {
      let sum: Decimal = splits.iter().sum();
      let expected = tx_amount.abs();
      if sum != expected {
          return Err(SplitImbalance { expected, actual: sum, difference: sum - expected });
      }
      Ok(())
  }

  pub struct SplitImbalance {
      pub expected: Decimal,
      pub actual: Decimal,
      pub difference: Decimal,
  }
  ```

- [x] T1.2 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod split;
  pub use split::{build_split_journal_entry, validate_split_balance, SplitDetail, SplitImbalance};
  ```

- [x] T1.3 — Étendre `crates/kesh-reconciliation/src/errors.rs` (1 variant ajouté + impl From manuel) :
  ```rust
  pub enum ReconciliationError {
      // ... 8-4 + 8-5a-base variants conservés (FiscalYearClosed)
      SplitImbalance { expected: Decimal, actual: Decimal, difference: Decimal },
  }
  ```

  **M3 Pass 4 — `impl From<SplitImbalance> for ReconciliationError` doit être écrit manuellement** (prescrit M5''' Pass 3, §helper-split-signature). Avec `thiserror`, `#[from]` ne s'applique pas sur un struct-variant multi-champs — il requiert un newtype variant. La conversion DOIT donc être une impl manuelle dans `errors.rs` (ou dans `split.rs`) :

  ```rust
  // Dans crates/kesh-reconciliation/src/errors.rs (ou split.rs)
  use crate::split::SplitImbalance;
  
  impl From<SplitImbalance> for ReconciliationError {
      fn from(e: SplitImbalance) -> Self {
          ReconciliationError::SplitImbalance {
              expected: e.expected,
              actual: e.actual,
              difference: e.difference,
          }
      }
  }
  ```

  Sans cette impl, `validate_split_balance(...)?` dans la closure `with_account_lock` (qui retourne `Result<_, ReconciliationError>`) ne compile pas — le compilateur dira « trait `From<SplitImbalance>` is not implemented for `ReconciliationError` ». Le dev agent DOIT l'écrire explicitement (ne pas chercher un `#[from]` thiserror sur le struct-variant).

- [x] T1.4 — Tests unit `kesh-reconciliation::split` (≥ 4) :
  1. `split_build_je_creates_n_plus_1_lines_for_debit_tx` (AC #93).
  2. `split_build_je_creates_n_plus_1_lines_for_credit_tx` (AC #94).
  3. `split_validate_balance_exact_match_ok` (AC #93).
  4. `split_validate_balance_imbalance_returns_error` (AC #95).

  **Note** : tests des préconditions (`splits[i].amount > 0` strict C1''' + `tx.amount != 0` step 6bis M2''' + length max 200 chars M4''') sont couverts au niveau handler/E2E HTTP (T3.4), pas au niveau helper pur (qui ne valide ni longueur description, ni signe individuel — par convention §helper-split-signature précondition handler-side).

### T2. Route API `POST /api/v1/reconciliation/split` (AC #93-#99)

- [x] T2.1 — Étendre `crates/kesh-api/src/routes/reconciliation.rs` avec handler `post_split` (cf. §validation-handler-side-split) :
  - Body `{ bankAccountId, bankTransactionId, splits: [...], valueDate? }` camelCase.
  - **Défaut valueDate** (M2 Pass 2 + F4''' Pass 3 Opus — 3 couches cohérent ground-truth `post_manual` reconciliation.rs:1281) : `entry_date = body.value_date.or(tx.value_date).unwrap_or(tx.booking_date)` :
    1. Si `body.value_date` présent → utiliser cette valeur.
    2. Sinon, si `tx.value_date` présent (banque a fourni une date de valeur distincte du booking) → utiliser cette valeur.
    3. Sinon → `tx.booking_date` (toujours présent, NOT NULL).
    
    **Asymmetry Pass 2 corrigée** : Pass 2 M4 disait juste « défaut = `tx.booking_date` » (2 couches), mais le code réel `post_manual` ligne 1281 a 3 couches incluant `tx.value_date`. Sans cette correction, split divergerait du flow manual sur les tx avec value_date distincte du booking_date (pratique courante banques suisses).
  - Pré-flight ordre §validation-handler-side-split étapes 1-7 (étape 2 ajoutée Pass 2).
  - Inside lock : étapes 8-13 (numérotation décalée).
  - **Différence majeure vs spec 8-5a unifiée** : pas de `bankLedgerAccountId` body. Résolu serveur-side via `bank_account.journal_account_id`.

- [x] T2.2 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : ajouter `.route("/api/v1/reconciliation/split", post(routes::reconciliation::post_split))`.

- [x] T2.3 — Étendre `crates/kesh-api/src/errors.rs` (variantes ajoutées/réutilisées) :
  - `AppError::ReconciliationSplitImbalance { expected, actual, difference }` → 400 `RECONCILIATION_SPLIT_IMBALANCE` body `details = { expected: '10700.00', actual: '10500.00', difference: '-200.00' }` (string Decimal cohérent AC #95).
  - `AppError::AccountNotFound { account_id, missing_account_ids: Option<Vec<i64>> }` (extension du variant 8-5a-zero/8-5a-base) → 404 `ACCOUNT_NOT_FOUND` body `{ error: { code, message, details: { accountId, missingAccountIds: [...] } } }` camelCase. `missingAccountIds` populated pour split (Vec d'ids invalides triés), single `accountId` pour manual/bank-accounts.

    **Convention split `account_id` quand plusieurs missing** : pour le cas split avec `missing_account_ids: Some(vec)`, `account_id` est le **premier** id de `vec` trié (cohérence avec `details.accountId` singular du variant 8-5a-base). Le frontend lit `missingAccountIds` (array) pour afficher tous les IDs invalides ; `accountId` (singular) sert de fallback.

    **MIGRATION REQUISE (F2 Pass 1 + F1''' Pass 3 Opus — 3 callsites, ground-truth `grep -rn "AppError::AccountNotFound"` 2026-05-07)** : l'extension casse la compilation de **3** callsites existants (pas 2 — F1''' Pass 3 ground-truth correctif) :
    - `crates/kesh-api/src/routes/reconciliation.rs:1199` (post_manual step 3 counterparty inactive) : `AppError::AccountNotFound { account_id }` → ajouter `missing_account_ids: None`.
    - `crates/kesh-api/src/routes/bank_accounts.rs:124` (patch_bank_account_journal_link account introuvable) : idem.
    - `crates/kesh-api/src/routes/bank_accounts.rs:128` (patch_bank_account_journal_link account archivé KF-002 anti-énumération) : idem. **Manqué Pass 1 F2** — confirmé Pass 3 Opus 1M context.
    - `crates/kesh-api/src/errors.rs:750` `IntoResponse` match arm : le destructuring `AppError::AccountNotFound { account_id }` → `AppError::AccountNotFound { account_id, missing_account_ids }`. Body adapté :
      ```rust
      let mut details = serde_json::json!({ "accountId": account_id });
      if let Some(ids) = missing_account_ids {
          details["missingAccountIds"] = serde_json::json!(ids);
      }
      ```
      Préserve la rétro-compat clients qui lisent `details.accountId` (manual + bank-accounts cases).
  - `AppError::BankAccountNotConfigured` (réutiliser variant 8-5a-base).
  - `AppError::ReconciliationFiscalYearClosed` (réutiliser variant 8-5a-base).
  - `AppError::ReconciliationTransactionNotPending` (réutiliser variant 8-5a-base).
  - **Pas de variant `ReconciliationOptimisticLockConflict`** (F1 Pass 1) : ce variant n'existe pas dans `errors.rs`. L'optimistic lock remonte via `ReconciliationError::Db(DbError::OptimisticLockConflict)` → `AppError::Database(DbError::OptimisticLockConflict)` → 409 `OPTIMISTIC_LOCK_CONFLICT` (cf. `reconciliation.rs:1277,1337` pattern 8-5a-base). Le match bloc `post_split` doit inclure `Err(ReconciliationError::Db(db_err)) => Err(AppError::Database(db_err))` (cohérent `post_manual` lignes 1410-1413).

### T3. Breaking change `POST /accept` discriminator type (AC #100)

- [x] T3.1 — Modifier `post_accept` dans `crates/kesh-api/src/routes/reconciliation.rs` :
  - Refactorer `AcceptProposalInput` en enum tagged (§note-implementation-accept) :
    ```rust
    #[derive(Debug, Deserialize, Clone)]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum AcceptProposalInput {
        #[serde(rename = "invoice")]
        Invoice {
            bank_transaction_id: i64,
            invoice_id: i64,
        },
        #[serde(rename = "split")]
        Split {
            bank_transaction_id: i64,
            splits: Vec<SplitProposalLine>,
            // M6''' Pass 3 Opus : valueDate optional cohérent avec /split standalone.
            #[serde(default)]
            value_date: Option<chrono::NaiveDate>,
        },
    }
    ```
  - **Retirer `AcceptType` enum (F8 Pass 1)** : l'enum `AcceptType { Invoice, Split }` défini dans §note-implementation-accept est dead code — le dispatch se fait via pattern matching sur `AcceptProposalInput`. Ne pas créer ce type ; Clippy `-D warnings` signale `dead_code` → CI rouge.
  - Match enum dans `accept_one` pour dispatcher vers flow `accept_invoice` (8-4 inchangé) ou `accept_split` (nouveau, équivalent batch de `/split`).
  - **Décision option (a) §note-implementation-accept** : implémenter `type='split'` dans `/accept` pour cohérence batch. Si volume trop large, fallback option (b) acceptable avec `type='split'` retourne 400 « non supporté v0.1, utiliser /reconciliation/split ».
  - **Gestion 400 pour type absent/inconnu (F9 Pass 1)** : par défaut, `axum::Json` + `serde(tag)` retourne 422 Unprocessable Entity sur désérialisation échouée. Pour retourner 400 `Validation` avec le message spécifié, implémenter un custom extractor `AcceptBodyExtractor` sur le modèle de `PatchJournalLinkBodyExtractor` (`crates/kesh-api/src/routes/bank_accounts.rs:55-89`) :
    ```rust
    // Pattern : impl FromRequest pour wrapper AcceptBody, catch JsonRejection → AppError::Validation
    // Si `type` absent → 400 message « champ `type` requis, valeurs acceptées v0.1 : ["invoice", "split"] »
    // Si `type` non reconnu → Serde retourne JsonDataError → même custom 400
    ```
    Ce custom extractor est **requis** pour satisfaire AC #100 (`accept_rejects_proposal_missing_type_discriminator` → 400 pas 422).

- [x] T3.2 — Vérifier impact `Copy` : retirer `Copy` de `AcceptProposalInput` (F11'' Pass 3), ajouter `clone()` aux usages internes si nécessaire.

### T3.3. Migration tests E2E 8-4 existants (AC #100 part 3)

- [x] T3.3 — Modifier `crates/kesh-api/tests/reconciliation_e2e.rs` (cf. §migration-21-tests) :
  - Patcher les **15 sites POST /accept actifs** (lignes 757, 910, 1030, 1104, 1203, 1416, 1492, 1632, 1679, 1737, 1805, 1870, 1980, 2064, 2129) pour ajouter `type: 'invoice'` dans le body proposals[*].
  - **Pas de site ignoré à patcher (F6 Pass 1)** : le test ignoré `post_accept_filters_currency_mismatch` ne contient aucun appel `POST /accept`.
  - Vérifier `cargo test -p kesh-api --test reconciliation_e2e` retourne **21 verts + 1 ignored** (= 22 attributs `#[sqlx::test(migrator)]` au total, 1 ignoré).

### T3.4. Tests E2E HTTP nouveaux 8-5a-bis (AC #93-#100)

- [x] T3.4 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_split_e2e.rs` *(nouveau fichier, ≥ 10 tests)* :
  1. `split_creates_journal_entry_with_n_plus_1_lines` (AC #93).
  2. `split_creates_journal_entry_for_credit_transaction` (AC #94).
  3. `split_rejects_imbalanced_payload` (AC #95).
  4. `split_rejects_single_line_payload` (AC #96 part 1).
  5. `split_rejects_too_many_lines` (AC #96 part 2).
  6. `split_rejects_unconfigured_bank_account_with_412` (AC #97).
  7. `split_does_not_leak_cross_tenant_account` (AC #98 part 1).
  8. `split_requires_comptable_role` (AC #98 part 2).
  9. `split_emits_audit_log` (AC #99 part 1).
  10. `split_rejects_already_reconciled` (AC #99 part 2).

  ET dans `crates/kesh-api/tests/reconciliation_e2e.rs` (extension du fichier existant) :

  11. `accept_rejects_proposal_missing_type_discriminator` (AC #100 part 1).
  12. `accept_with_explicit_invoice_type_runs_8_4_flow` (AC #100 part 2 — peut être couvert par les 15 tests migrés AC #100 part 3, à apprécier).

### T4. Helper `fiscal_years::find_open_covering_date` — réutilisé tel quel (AC #99)

- [x] T4.1 — Pas besoin de créer un nouveau helper : utiliser `fiscal_years::find_open_covering_date` existant (Story 3-7) avec `&mut tx_outer` passé depuis le handler `post_split` (cohérent 8-5a-base T4.1).

### T5. Frontend `TransactionSplitModal` + extensions (AC #93-#100 UI)

- [x] T5.1 — Étendre `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
  ```ts
  // **Différence majeure vs spec 8-5a unifiée** : pas de `bankLedgerAccountId`.
  // **F5 Pass 1** : le paramètre `description?: string` top-level a été retiré —
  // il n'a pas de contrepartie dans le body Rust (SplitBody ne définit pas de description
  // top-level). Chaque split line a sa propre description.
  export async function splitTransaction(
      bankAccountId: number,
      bankTransactionId: number,
      splits: { counterpartyAccountId: number; amount: string; description: string }[],
      valueDate?: string,
  ): Promise<{ bankTransactionId: number; journalEntryId: number }>;
  ```

- [x] T5.2 — **Migration breaking change `acceptProposal`** : modifier `acceptProposal` pour ajouter `type: 'invoice'` explicite dans le body envoyé. Met à jour aussi `ReconciliationProposals.svelte` qui consomme. **Cohérent breaking Q2** :
  ```ts
  // Avant (8-4)
  body: { proposals: [{ bankTransactionId, invoiceId }] }
  // Après (8-5a-bis)
  body: { proposals: [{ type: 'invoice', bankTransactionId, invoiceId }] }
  ```
  **MIGRATION TYPE REQUISE (F3 Pass 1)** : mettre à jour `frontend/src/lib/features/reconciliation/reconciliation.types.ts` :
  - `AcceptProposalInput` → discriminated union TypeScript :
    ```ts
    export type AcceptProposalInput =
      | { type: 'invoice'; bankTransactionId: number; invoiceId: number }
      | { type: 'split'; bankTransactionId: number; splits: SplitProposalLine[] };
    export interface SplitProposalLine {
      counterpartyAccountId: number;
      amount: string;
      description: string;
    }
    ```
  **MIGRATION TEST VITEST REQUISE (F4 Pass 1)** : mettre à jour `reconciliation.api.test.ts` (repérer la ligne avec `grep -n 'expect(body.proposals)' reconciliation.api.test.ts`) :
  - Changer `expect(body.proposals).toEqual([{ bankTransactionId: 1, invoiceId: 2 }])` → `expect(body.proposals).toEqual([{ type: 'invoice', bankTransactionId: 1, invoiceId: 2 }])`. (L1 Pass 4 : le numéro `~69` est approximatif, grep est plus fiable).

- [x] T5.3 — Créer `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` :
  - Props : `bankTransaction`, `bankAccountId`.
  - Tableau splits éditable (ajout/suppression de ligne, min 2 max 50).
  - Sticker balance live computed `sum vs |tx.amount|` (vert si exact match, rouge sinon, avec différence affichée).
  - Bouton submit désactivé tant que balance ≠ exact OU `splits.length < 2`.
  - Gestion erreur `412 BANK_ACCOUNT_NOT_CONFIGURED` : message + lien `/bank-accounts`.

- [x] T5.4 — Étendre `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - Pour chaque ligne tx avec `candidates: []` : 1 bouton « Éclater » (ouvre `TransactionSplitModal`). **Bouton « Affecter manuellement » déjà livré 8-5a-base** — coexistence à valider.
  - On modal success : refresh la liste.

- [x] T5.5 — Tests Vitest (≥ 3-4) :
  1. `TransactionSplitModal: balance indicator updates live` (AC #93/#94).
  2. `TransactionSplitModal: submit disabled until balance exact match` (AC #93/#95).
  3. `acceptProposal sends type: 'invoice' in body` (régression breaking Q2).
  4. *(stretch)* `ReconciliationProposals: shows split button next to manual button for tx without candidate`.

### T6. i18n (AC implicite UI)

- [x] T6.1 — Ajouter ~5 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixe strict `reconciliation-split-*`) :
  - `reconciliation-split-button-label`
  - `reconciliation-split-modal-title`
  - `reconciliation-split-balance-indicator`
  - `reconciliation-split-error-imbalance`
  - `reconciliation-split-success-toast`
  FR canonical.
- [x] T6.2 — Traductions DE / IT / EN-CH — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [x] T6.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T7. Tests E2E Playwright + a11y (AC #93-#99)

- [x] T7.1 — Créer `frontend/tests/e2e/reconciliation-split.spec.ts` (≥ 1 actif) :
  1. `split end-to-end` : login Comptable, navigate `/reconciliation`, click « Éclater » sur tx -10700, ajouter 3 lignes (5000+4500+1200), vérifier balance indicator passe au vert, valider, vérifier toast succès + tx disparaît.

- [x] T7.2 — Test a11y axe (AC #99) : 1 scénario sur la modal `TransactionSplitModal` ouvert — `expect(await new AxeBuilder().analyze()).toHaveNoViolations()`.

## Risque de splitting

**Modules touchés** :
1. `crates/kesh-reconciliation/src/split.rs` *(nouveau)*.
2. `crates/kesh-reconciliation/src/lib.rs` + `errors.rs` (extension `SplitImbalance`).
3. `crates/kesh-api/src/routes/reconciliation.rs` (1 nouveau handler `post_split` + refactor `post_accept` discriminator).
4. `crates/kesh-api/src/errors.rs` (1 nouveau variant `ReconciliationSplitImbalance` + extension `AccountNotFound` avec `missing_account_ids`).
5. `crates/kesh-api/tests/reconciliation_e2e.rs` (migration 15 sites POST /accept) + `crates/kesh-api/tests/reconciliation_split_e2e.rs` *(nouveau)*.
6. `crates/kesh-i18n` (5 clés × 4 locales).
7. `frontend/src/lib/features/reconciliation` (extension `reconciliation.api.ts` + `ReconciliationProposals.svelte` + nouveau `TransactionSplitModal.svelte`).

**Total : 7 modules**. Au-dessus du seuil CLAUDE.md « splitter si > 5 modules ». **Pas de re-split** car (a) le scope est cohérent autour d'un seul flow (FR48 split + Q2 breaking accept), (b) les patterns sont acquis 8-4/8-5a-base, (c) volume estimé ~600-700 lignes spec + ~1000-1200 lignes code (incluant migration 15 tests) = légèrement supérieur 8-5a-base mais bien en-dessous du seuil 1500 lignes 8-4 retro.

**Note breaking change Q2** : la migration des 15 sites POST /accept dans `reconciliation_e2e.rs` est mécanique (regex find/replace). Pas de risque architectural — juste de l'attention au détail.

**Aucune dérogation nécessaire**.

## Dev Notes

### API surface livrée 8-1b/8-2/8-3/8-4/8-5a-zero/8-5a-base — patterns à réutiliser

- **Multi-tenant scoping** (KF-002 Pattern 1).
- **Audit log atomique** : `audit_log::insert_in_tx`. Action `reconciliation.split_applied` distincte (Q4a).
- **Erreurs structurées** : `AppError::*` typé. Body camelCase JSON.
- **i18n key ownership** : préfixe strict `reconciliation-split-*` (Story 6-3).
- **`rust_decimal::Decimal`** : Decimal exact partout pour amounts. Validateur `validate_split_balance` utilise `==` strict.
- **Repository pattern + sqlx** : Executor générique `<E: Executor>`.
- **Advisory lock per-account** : `with_account_lock(tx, company_id, bank_account_id, 5)` réutilisé pour split (cohérent 8-5a-base manual).
- **`journal_entries::create_in_tx`** : helper Story 5-2.
- **`fiscal_years::find_open_covering_date`** : helper Story 3-7.
- **`bank_account.journal_account_id`** : column livrée 8-5a-zero — résolu serveur-side.
- **`manual::build_journal_entry_for_counterparty`** : helper Story 8-5a-base, signature stable. Composé par `split::build_split_journal_entry` pour cohérence pattern.
- **`find_strictly_pending_by_id_for_account`** : helper Story 8-5a-base, partagé par /manual et /split.

### Lessons leçons des stories précédentes

- **8-4 retro** : 8-5a-bis découpée à ~700 lignes spec + ~1200 lignes code pour viser ≤ 2 passes review.
- **8-5a unifiée Pass 3 Opus** : élimination dette F2'' à la racine via 8-5a-zero. Cohérence UX `default_*_account_id`.
- **5-2 leçon** (`create_in_tx`) : route split utilise `create_in_tx` plutôt que `create`.
- **8-4 patch P3-H1** (optimistic lock UPDATE bank_transactions `AND version = ?`) : appliquer systématiquement dans 8-5a-bis.
- **Q2 décision Guy 2026-05-07** : breaking change /accept — Kesh pas en prod, accepté. Migration des 15 sites de test 8-4 fait partie du scope 8-5a-bis.

### Patterns architecturaux à respecter

- **Pas de dépendance circulaire** : `kesh-reconciliation → kesh-core, kesh-db`.
- **Cohérence audit log snake_case top-level + sub-objects** : `details_json` 100% snake_case (cohérent F4'' Pass 3).
- **Pas d'`f64` pour montants** : `Decimal` partout.
- **Tests : éviter le coupling temporel** : dates fixes dans seeds.
- **`auto_match_rejected_at=NULL` au split** : indispensable pour éviter état incohérent.

### Source tree à toucher

**DB** : (pas de modification — tous les helpers réutilisés)
- `crates/kesh-db/src/repositories/reconciliation.rs` (utilise `find_strictly_pending_by_id_for_account` 8-5a-base)
- `crates/kesh-db/src/repositories/fiscal_years.rs` (utilise `find_open_covering_date`)
- `crates/kesh-db/src/repositories/accounts.rs` (utilise `find_by_id_in_company` en itération séquentielle cap 50 — L3 Pass 1 : pas de nouveau helper kesh-db)
- `crates/kesh-db/src/repositories/bank_accounts.rs` (utilise `find_by_id_for_company` étendu 8-5a-zero)

**Backend `kesh-reconciliation`** :
- `crates/kesh-reconciliation/src/split.rs` *(nouveau, pure helper + balance validator)*
- `crates/kesh-reconciliation/src/lib.rs` (ajout `pub mod split` + re-exports)
- `crates/kesh-reconciliation/src/errors.rs` (1 variant ajouté `SplitImbalance`)

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation.rs` (extension `post_split` + refactor `post_accept` discriminator)
- `crates/kesh-api/src/lib.rs` (mount route /split)
- `crates/kesh-api/src/errors.rs` (1 nouveau variant + extension)
- `crates/kesh-api/tests/reconciliation_split_e2e.rs` *(nouveau, ≥ 10 tests)*
- `crates/kesh-api/tests/reconciliation_e2e.rs` (migration 15 sites POST /accept — T3.3 ; le test ignored n'a pas d'appel /accept à patcher)

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (~5 nouvelles clés `reconciliation-split-*` × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (extension `splitTransaction` + migration `acceptProposal` avec `type: 'invoice'`)
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts` **(F3 Pass 1 : mise à jour `AcceptProposalInput` → discriminated union + ajout `SplitProposalLine`)**
- `frontend/src/lib/features/reconciliation/reconciliation.api.test.ts` **(F4 Pass 1 : migration test `acceptProposals` ligne ~69 pour ajouter `type: 'invoice'`)**
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (extension bouton « Éclater »)
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts` *(nouveau, Vitest)*
- `frontend/tests/e2e/reconciliation-split.spec.ts` *(nouveau, Playwright)*

### Standards de test

- **Unit `kesh-reconciliation`** : `#[cfg(test)] mod tests` inline `split.rs`. ≥ 4 unit tests T1.4.
- **Intégration `kesh-db`** : pas de nouveau helper en 8-5a-bis (réutilisation 8-5a-base).
- **E2E HTTP `kesh-api`** : ≥ 10 nouveaux tests T3.4 (split + 1-2 accept-discriminator) + 15 sites POST /accept migrés `type: 'invoice'` dans 8-4 existant (= 21 tests actifs `reconciliation_e2e.rs` post-migration + 10+ tests dans `reconciliation_split_e2e.rs` nouveau ; 1 ignored placeholder inchangé).
- **Vitest frontend** : ≥ 3-4 tests T5.5.
- **Playwright** : ≥ 1 actif + 1 a11y T7.

### Checklist locale avant push

```sh
# Backend (cf. CLAUDE.md « Test Locally First »)
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB up requis (incl. 21 tests 8-4 migrés)

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # T6.3
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reconciliation-split.spec.ts
```

### Limitations connues v0.1 (8-5a-bis)

| # | Limitation | Justification |
|---|---|---|
| L44 | Split ne lie pas vers des invoices multiples | v0.1 : split crée un journal_entry à N+1 lignes pointant sur des comptes de contrepartie type frais/produits. Lier 1 split vers N invoices clients (paiement multi-factures) reporté v0.2. Workaround v0.1 : utiliser N `manual` séparés (anti-pattern). |
| L45 | Annulation de réconciliation non disponible | v0.1 : pas d'undo `accept` / `manual` / `split`. v0.2 : route `POST /reconciliation/revert/{bankTransactionId}`. |
| L47 | Multi-currency split non supporté | Tous les splits dans la même currency que la tx (CHF v0.1 mono-CHF cohérent L38 héritée 8-4). Reporté Story 11. |
| **(post-Q5)** | Suggestion ML automatique post-split non livrée | Décision Guy Q5 2026-05-07 : pas de `ruleSuggestion` dans response. L'utilisateur crée ses rules via `/reconciliation/rules` (8-5b). |
| **(post-Q2)** | Pas de migration progressive type discriminator | Breaking change v0.1 — Kesh pas en prod, accepté par décision Guy. Si Kesh atteint prod plus tard avec multi-version API, refactor en optional + warning header. |

### Risques et points d'attention pour le dev agent

1. **Path-dépendance 8-5a-base** : ce 8-5a-bis **ne peut démarrer** que si 8-5a-base est `done`/merged sur main (helper `manual::build_journal_entry_for_counterparty` doit exister + helper `find_strictly_pending_by_id_for_account` + variant `BankAccountNotConfigured` + `ReconciliationFiscalYearClosed` + `ReconciliationTransactionNotPending`). Sinon les tests E2E HTTP `split_rejects_*` qui testent ces erreurs ne compilent pas.

2. **AccountNotFound migration — 3 callsites (F1''' Pass 3 Opus, M1 Pass 4 Sonnet)** : T2.3 ligne 567 liste **3 callsites** à patcher (`reconciliation.rs:1199` + `bank_accounts.rs:124` + `bank_accounts.rs:128`). Ground-truth vérifié Pass 4 Sonnet 4.6 : `grep -rn "AppError::AccountNotFound"` = 3 occurrences exactes. ~~Pass 2 H2 affirmait « exactement 2 »~~ — ce chiffre était incorrect et a été corrigé en Pass 3 F1''' (Opus). Le dev agent NE DOIT PAS s'étonner de trouver 3 callsites : c'est la valeur correcte.

2. **Breaking change `POST /accept` (Q2)** : si le dev agent oublie de migrer les **15 sites POST /accept** actifs E2E HTTP 8-4 existants (AC #100 + T3.4), CI rouge en local + remote. **Vérification systématique** : `cargo test -p kesh-api --test reconciliation_e2e` doit retourner 21 verts + 1 ignored après patch (= 22 attributs `#[sqlx::test]`).

3. **`AcceptProposalInput` perd `Copy`** : F11'' Pass 3 — le refactor enum tagged + `Vec<SplitProposalLine>` casse `Copy`. Ajouter `clone()` aux usages internes. Vérifier qu'aucun caller n'attend `Copy` (ex. `enumerate().map(|(i, p)| ...)` qui prend ownership).

4. **Helper `split::build_split_journal_entry` ré-utilisation manual** : préférer une implémentation directe à N+1 lignes (pas une composition littérale qui appelerait N fois manual). Le rationale §helper-split-signature est de factoriser le **pattern** sign-aware, pas l'algorithme exact.

5. **Test E2E HTTP volume cumulé** : 10 nouveaux tests 8-5a-bis + 15 sites migrés 8-4 = ~25 sites de test à maintenir. Pas de dette test acceptable (lessons 8-4 retro). Sécurité multi-tenant + RBAC + non-régression sont **incontournables**.

6. **Suppression de la suggestion ML (Q5)** : ne pas implémenter `suggest_rule` ni `ruleSuggestion`. Out-of-scope 8-5a-bis (et 8-5b aussi, reporté v0.2).

7. **`type='manual'` dans `/accept`** : décidé NON-implémenté v0.1 (route `/manual` standalone livrée 8-5a-base couvre le cas use-case). Le discriminator `type='manual'` n'est PAS inclus dans `AcceptType` enum 8-5a-bis (Serde rejette 400). Si demande utilisateur pour unifier batch, créer un CR explicite.

### Références

- [`8-5a-reconciliation-manuelle-split.md`](8-5a-reconciliation-manuelle-split.md) — spec d'origine `archived-split-bis`.
- [`8-5a-zero-bank-account-journal-link.md`](8-5a-zero-bank-account-journal-link.md) — pré-requis foundation column.
- [`8-5a-base-manual-match.md`](8-5a-base-manual-match.md) — pré-requis helper manual + helper repo + variants AppError.
- [`8-5b-reconciliation-rules-engine.md`](8-5b-reconciliation-rules-engine.md) — rules engine (path-dep 8-5a-bis pour breaking POST /accept type='rule').
- [`epic-8.md`](../planning-artifacts/epic-8.md) — Story 8-5 ACs originaux (FR45-48).
- [`prd.md`](../planning-artifacts/prd.md) §FR48 ligne 442.
- [`8-4-reconciliation-matching-automatique.md`](8-4-reconciliation-matching-automatique.md) — patterns à réutiliser + 21 tests E2E HTTP à migrer.

## Dev Agent Record

### Agent Model Used

Claude Opus 4.7 (1M context) — single-pass continuous dev-story 2026-05-12.

### Debug Log References

- Pass 1 build fail : `error[E0004] non-exhaustive patterns: Err(ReconciliationError::SplitImbalance { .. }) not covered` sur 3 match blocks (`post_accept` ligne 419, `post_reject` ligne 907, `post_manual` ligne 1371) post-ajout du variant `SplitImbalance`. Résolu en ajoutant branches exhaustives mappant vers `AppError::ReconciliationSplitImbalance` (unreachable en pratique pour `accept_batch` / `reject_batch` / `post_manual` mais requises par compilateur).
- Pass 1 E2E tests 15 fails après refactor enum tagged : status 400 inattendu sur tous les sites POST /accept migrés `type: "invoice"`. Cause racine : `#[serde(tag = "type", rename_all = "camelCase")]` enum-level n'applique pas le `rename_all` aux champs des variants struct. Fix : déplacer `rename_all = "camelCase"` au niveau de chaque variant `#[serde(rename = "invoice", rename_all = "camelCase")]`. Trend : 15 fails → 0 fails Pass 2.
- Pass 1 split_e2e tests 4 fails sur 10 :
  - 2 fails sur colonne `journal_entry_id` introuvable dans `journal_entry_lines` (vrai nom : `entry_id`). Fix : rename SQL queries.
  - 1 fail audit log read : `details_json` est `JSON` (BLOB), pas `VARCHAR`. Fix : utiliser `Value` direct via `sqlx::query_as` au lieu de `String` + `serde_json::from_str`.
  - 1 fail balance imbalance : `actual` retourné "10500" au lieu de "10500.00" (Decimal sum garde la scale du plus petit operand : "5000" scale 0 → sum scale 0). Fix : sender JSON `"5000.00"` etc. pour preserver scale 2 (test data alignment, le backend ne force pas la scale).
- helpers `bank_accounts::find_by_id_for_company` et `accounts::find_by_id_in_company` ne sont **pas Executor-generic** (signature `&MySqlPool`). Inside `accept_one_split` (qui reçoit un `&mut Transaction`), les SELECTs sont **inlinés** (cohérent pattern step 8 UPDATE inline dans `accept_one_invoice`). kesh-db extension à un Executor-generic pattern reportée Story 11+ (hors scope 8-5a-bis).

### Completion Notes List

**Régression Pass 8 mésync remédiée** :
- Variant `ReconciliationError::SplitImbalance` créé concrètement dans `crates/kesh-reconciliation/src/errors.rs` (Pass 4 documenté en spec mais jamais ajouté à errors.rs avant cette session).
- `impl From<SplitImbalance> for ReconciliationError` manuel dans `split.rs` (thiserror `#[from]` inapplicable sur struct-variant multi-champs cf. T1.3 M3 Pass 4).
- 3 match blocks (`post_accept` / `post_reject` / `post_manual`) reçoivent la branche `SplitImbalance` pour exhaustivité du compilateur (unreachable en pratique).

**T1-T7 livrés single-pass Opus 4.7 :**
- T1 helper `kesh-reconciliation::split` : `SplitDetail` + `build_split_journal_entry` (N+1 lignes sign-aware) + `validate_split_balance` (Decimal exact) + `SplitImbalance` + `impl From` → 5 tests unit verts (debit + credit + balance valid + balance imbalance + balance excess regression).
- T2 route `POST /api/v1/reconciliation/split` : handler `post_split` avec 13 steps validation §validation-handler-side-split (steps 1, 1bis, 2 surface ; steps 3-7 pré-flight ownership/account/balance ; steps 8-13 inside `with_account_lock` advisory). Routes sub-router `comptable_routes` (RBAC Comptable+).
- T3 breaking Q2 `AcceptProposalInput` enum tagged Serde (Invoice + Split variants) + custom extractor `AcceptBodyExtractor` (400 Validation au lieu de 422 Axum natif). Refactor `accept_one` en dispatch top-level → `accept_one_invoice` (corps 8-4 inchangé, 29 sites `proposal.X` → params plain i64) + `accept_one_split` (nouveau, équivalent batch de `/split` standalone, lookup `journal_account_id` inside lock per-proposal H1 Pass 4).
- T3.3 migration 19 occurrences `{ "bankTransactionId": ..., "invoiceId": ... }` → `{ "type": "invoice", ... }` via `sed -i` (15 sites POST /accept actifs + 4 sites batch dans mêmes tests).
- T3.4 fichier nouveau `crates/kesh-api/tests/reconciliation_split_e2e.rs` (10 tests E2E HTTP, **10/10 verts**) + 2 tests discriminator nouveaux dans `reconciliation_e2e.rs`.
- T5 frontend : `reconciliation.types.ts` discriminated union TS + `SplitProposalLine`. `reconciliation.api.ts` `splitTransaction()` + migration `acceptProposals` body. `ReconciliationProposals.svelte` bouton « Éclater » + state `splitOpen`/`splitProposal`/`onSplitSuccess`. `TransactionSplitModal.svelte` nouveau (tableau éditable, balance indicator live, min 2 max 50, gestion 412). 4 tests Vitest nouveaux.
- T6 i18n 5 clés `reconciliation-split-*` × 4 locales (fr/de/it/en-CH), traductions natives (pas copies).
- T7 Playwright `reconciliation-split.spec.ts` 1 scénario empty state + 1 axe a11y.

**Tests verts** :
- kesh-reconciliation : 20/20 unit (+ 5 nouveaux split).
- reconciliation_split_e2e : **10/10 E2E HTTP** avec MariaDB up.
- reconciliation_e2e : **23/23 E2E HTTP + 1 ignored** post-migration 15 sites (régression Q2 vérifiée).
- reconciliation_manual_e2e : 13/13 (0 régression 8-5a-base).
- bank_accounts_e2e : 9/9 (0 régression 8-5a-zero).
- Vitest frontend : **223/223** (215 prior + 8 nouveaux dont 4 TransactionSplitModal).

**Validation Test Locally First full stack verte** :
- `cargo fmt --all -- --check` clean.
- `cargo build --workspace --all-targets` clean (~1m).
- `cargo clippy --workspace --all-targets -- -D warnings` clean (~3s post-cache).
- `cargo test -p kesh-reconciliation --lib` 20/20.
- `cargo test -p kesh-api --test reconciliation_split_e2e -- --test-threads=1` 10/10 (~19s).
- `cargo test -p kesh-api --test reconciliation_e2e -- --test-threads=1` 23/23 + 1 ignored (~45s).
- `cargo test -p kesh-api --test reconciliation_manual_e2e -- --test-threads=1` 13/13.
- `cargo test -p kesh-api --test bank_accounts_e2e -- --test-threads=1` 9/9.
- `npm run check` 0 errors 16 warnings (pré-existants pas reliés).
- `npm run lint-i18n-ownership` PASS.
- `npm run test:unit` 223/223 (~5s).
- `npm run build` clean.

**Findings résiduels non-bloquants documentés** :
- (a) Pattern SELECT inline dans `accept_one_split` (4 occurrences : journal_account lookup, bank ledger active check, batch counterparty active check ×N) — équivalent fonctionnel des helpers `bank_accounts::find_by_id_for_company` et `accounts::find_by_id_in_company` qui prennent `&MySqlPool` non-Executor-generic. Dette de refactoring kesh-db reportée Story 11+ (extension Executor-generic des 2 helpers).
- (b) `accept_one_split` n'émet **pas** d'audit `journal_entry.created` séparé : `journal_entries::create_in_tx` l'émet automatiquement (vérifié AC #99 part 1 test `split_emits_audit_log` qui asserte `je_audit_count = 1`).
- (c) `AcceptedProposal.invoice_id = 0` sentinel pour Split variant (la structure commune avec Invoice flow). Si frontend v0.2 doit distinguer Invoice vs Split au niveau de la response `AcceptResponse`, refactor en enum tagged response. v0.1 le frontend traite uniformément.
- (d) `make_new_tx` dans `reconciliation_split_e2e.rs` omet `counterparty_name` (signature 7 args vs 8 dans `reconciliation_e2e.rs`). Non-bloquant : pas de test 8-5a-bis qui dépend de counterparty_name.

**Lève L19/L20/L21 héritées 8-4** : split crée écriture sans facture pré-existante (L19), pas de matching journal_entries non-invoice nécessaire (L20), pas de paiement partiel (L21).

**Path-dépendance descendante 8-5b débloquée** :
- Helper `split::build_split_journal_entry` signature stable contractée.
- Variant `AcceptProposalInput::Rule` à ajouter par 8-5b (enum tagged déjà posé).
- 15 sites POST /accept migrés type='invoice' — pas de re-migration nécessaire pour 8-5b.

### File List

**Backend `kesh-reconciliation`** :
- `crates/kesh-reconciliation/src/split.rs` *(nouveau, ~390 lignes incluant tests)*
- `crates/kesh-reconciliation/src/lib.rs` (ajout `pub mod split` + re-exports `SplitDetail`/`SplitImbalance`/`build_split_journal_entry`/`validate_split_balance`)
- `crates/kesh-reconciliation/src/errors.rs` (1 variant `SplitImbalance` ajouté + import `rust_decimal::Decimal`)

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation.rs` (~300 lignes ajoutées : refactor enum tagged `AcceptProposalInput` + impl `bank_transaction_id()` + struct `SplitProposalLine` + `AcceptBodyExtractor` + handler `post_split` + helpers `accept_one_invoice` (renommage) + `accept_one_split` (nouveau) + 3 branches exhaustives `SplitImbalance` dans match blocks 8-4/8-5a-base)
- `crates/kesh-api/src/lib.rs` (mount route POST `/api/v1/reconciliation/split`)
- `crates/kesh-api/src/errors.rs` (variant `AccountNotFound` étendu avec `missing_account_ids: Option<Vec<i64>>` + 3 callsites migrés + nouveau variant `ReconciliationSplitImbalance` + mapping IntoResponse)
- `crates/kesh-api/src/routes/bank_accounts.rs` (2 callsites `AccountNotFound` migrés vers nouvelle signature)
- `crates/kesh-api/tests/reconciliation_split_e2e.rs` *(nouveau, ~950 lignes, 10 tests)*
- `crates/kesh-api/tests/reconciliation_e2e.rs` (migration 19 occurrences `{ "bankTransactionId":` → `{ "type": "invoice", "bankTransactionId":` + 2 tests discriminator ajoutés en fin de fichier)

**i18n** :
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` (5 clés `reconciliation-split-*` ajoutées)
- `crates/kesh-i18n/locales/de-CH/messages.ftl` (idem, traductions DE)
- `crates/kesh-i18n/locales/it-CH/messages.ftl` (idem, traductions IT)
- `crates/kesh-i18n/locales/en-CH/messages.ftl` (idem, traductions EN)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts` (`AcceptProposalInput` discriminated union TS + `SplitProposalLine` + `SplitResponse`)
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (`splitTransaction()` + import `SplitProposalLine`/`SplitResponse`)
- `frontend/src/lib/features/reconciliation/reconciliation.api.test.ts` (test `acceptProposals` mis à jour avec `type: 'invoice'`)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (state `splitOpen`/`splitProposal` + `openSplit`/`onSplitSuccess` + bouton « Éclater » par row + import `TransactionSplitModal` + injection modal + `type: 'invoice' as const` dans `items` du flow accept)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` (mise à jour assertion `acceptProposals` payload avec `type: 'invoice'`)
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` *(nouveau, ~290 lignes)*
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts` *(nouveau, 4 tests Vitest)*
- `frontend/tests/e2e/reconciliation-split.spec.ts` *(nouveau, 2 tests Playwright : structure + axe a11y)*

**Spec / sprint-status** :
- `_bmad-output/implementation-artifacts/8-5a-bis-split-breaking-accept.md` (Status `ready-for-dev` → `in-progress` → `review` + checkboxes Tasks/Subtasks marquées [x] + Dev Agent Record + File List + Change Log)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (`8-5a-bis-split-breaking-accept: ready-for-dev → in-progress → review` + bump `last_updated`)

## Change Log

| Date | Entrée | Auteur |
|------|--------|--------|
| **2026-05-07** | Spec créée par re-split mécanique de 8-5a unifiée (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7). 8-5a-bis = FR48 split + breaking POST /accept Q2. **Différence majeure vs spec 8-5a unifiée** : le body POST `/split` n'inclut PAS `bankLedgerAccountId` — résolu serveur-side via `bank_account.journal_account_id` (foundation 8-5a-zero). Helper public `kesh-reconciliation::split::build_split_journal_entry` compose le pattern de `manual::build_journal_entry_for_counterparty` (8-5a-base). Migration nominale des 15 sites POST /accept dans `reconciliation_e2e.rs` (= 21 actifs + 1 ignored = 22 attributs `#[sqlx::test]` au total) pour Q2 breaking. 8 ACs (#93-#100). Tasks T1-T7. Path-dépendance bloquante : 8-5a-base `done`/merged. Status `8-5a-bis-split-breaking-accept: backlog`. | Claude (Opus 4.7 re-split workflow) |
| **2026-05-07** | **Pass 1 validate Sonnet 4.6** — 9 findings (0 CRITICAL + 2 HIGH + 5 MEDIUM + 2 LOW). Patches appliqués : [F1] retire `ReconciliationOptimisticLockConflict` inexistant T2.3, remplace par pattern `Db(DbError::OptimisticLockConflict)` réel. [F2] T2.3 : migration callsites `AccountNotFound` extension `missing_account_ids`. [F3] T5.2 + source tree : ajoute `reconciliation.types.ts` mise à jour discriminated union. [F4] T5.2 : directive migration test Vitest existant ligne ~69. [F5] T5.1 + §7 : retire `description?: string` top-level sans contrepartie Rust de `splitTransaction`. [F6] §migration-21-tests + T3.3 (ex-T3.4) : corrige 16 → 15 sites (test ignoré = 0 appel `/accept`). [F7] §validation step 6 : corrige call-site `validate_split_balance` Iterator → `collect::<Vec<_>>()`. [F8] T3.1 : retire `AcceptType` dead code (Clippy `-D warnings`). [F9] T3.1 : prescrit custom extractor pour 400 vs 422 serde. [LOW L2] renommage T3.4→T3.3/T3.5→T3.4. [LOW L3] source tree contradiction "nouveau batch" → "itération séquentielle". Trend : Pass 1 = 7 findings > LOW. Prochaine étape Pass 2 Haiku 4.5. | Claude (Sonnet 4.6 validate) |
| **2026-05-07** | **Pass 2 validate Haiku 4.5** — 6 findings (1 CRITICAL + 4 HIGH + 6 MEDIUM + 2 LOW). Patches appliqués : [C1] Ajoute clarification signe `splits[*].amount` toujours >= 0 + ajout validation step 2 pré-flight handler (montants négatifs → 400). [H1] Clarification duplication sign logic — recommandation implémentation directe (non-composition littérale). [H2] Ground-truth vérification AccountNotFound 2 callsites (Pass 1 affirme). [H3] Custom extractor AcceptBodyExtractor pattern valid (non-issue si texte dur). [H4] Option (a) type='split' dans /accept recommandée si < 100 lignes (cohérence batch UX). [M1] Iterator→Vec (Pass 1 patché, confirmé OK). [M2] Validation `splits[*].amount >= 0` ajoutée step 2. [M3] `auto_match_rejected_at = NULL` lors UPDATE (step 12 renumerotée). [M4] `valueDate` défaut = `tx.booking_date` (cohérent manual). [M5] ≥10 tests E2E flexible (OK). [M6] Ground-truth AccountNotFound migration. [LOW L1] Max 50 splits justifiée (OK, pas change). [LOW L2] Nomenclature asymétrie (mineur, OK). Trend : 1 CRITICAL + 4 HIGH + 6 MEDIUM = 11 findings > LOW. Recommendation : réappliquer patches et relancer Pass 3 Opus si scope implémentation large. | Claude (Haiku 4.5 validate) |
| **2026-05-09** | **Pass 3 validate Opus 4.7 — VALIDATION FINALE** — 11 findings > LOW (1 CRITICAL + 4 HIGH + 6 MEDIUM + 4 LOW). Patches appliqués : [C1'''] Pass 2 a introduit contradiction §scope vs step 2 validation (`> 0` strict vs `>= 0`) — harmonisé sur strict `> 0` pour empêcher lignes JE 0/0 sémantiquement vides (ground-truth `journal_entries::create_in_tx` ne valide pas `debit > 0 OR credit > 0`). [F1'''] AccountNotFound migration : ground-truth = **3 callsites** (124, 128, 1199), pas 2 — `bank_accounts.rs:128` manqué Pass 1+2 ; mapping IntoResponse détaillé pour `Some(vec)` vs `None`. [F2'''] §note-implementation-accept nettoyée (suppression dead `AcceptType` enum séparé, conservation seulement `AcceptProposalInput` tagged enum, cohérent T3.1 F8). [F3'''] Refactor `accept_one` ground-truth : 29 sites internes accèdent `proposal.bank_transaction_id`/`proposal.invoice_id` — pattern dispatch `match proposal { Invoice {...} => accept_one_invoice(...), Split {...} => accept_one_split(...) }` détaillé. [F4'''] `valueDate` 3-couches `body.value_date.or(tx.value_date).unwrap_or(tx.booking_date)` cohérent post_manual reconciliation.rs:1281 (Pass 2 M4 disait 2 couches, manquait `tx.value_date`). [M1'''] §error-precedence-order référencée mais inexistante — note de transposition retirée. [M2'''] Ajout step 6bis pré-flight `tx.amount != 0` (defense-in-depth manquant côté split). [M3'''] Spécification handler-side construction `description` top-level JE (`format!("Éclatement transaction agrégée ({} lignes)", splits.len())`). [M4'''] Validation longueur description split max 200 chars step 1bis (defense-in-depth backend). [M5'''] Conversion `SplitImbalance` (struct) → `ReconciliationError::SplitImbalance` (variant) via `impl From<SplitImbalance>` recommandé. [M6'''] Ajout `value_date: Option<NaiveDate>` au variant `Split` de `AcceptProposalInput` (Pass 2 avait commentaire `// valueDate optional` sans le champ). [LOW] Nettoyage références stale "16 sites / 22 verts" → "15 sites / 21 verts". [LOW] Wording « sign opposé à la majorité » → « opposé aux N splits ». [LOW] Wording « manual composé par helper split » → « pattern réutilisé sans composition littérale » (cohérent Pass 2 H1). Trend : Pass 1 = 7 → Pass 2 = 11 → Pass 3 = 11 findings > LOW. **STOP cycle 8-passes CLAUDE.md ATTEINT** (3 passes Sonnet→Haiku→Opus, max budget 8 atteint avec marge ; passes additionnelles probablement convergence sur LOW only). Recommandation Guy : **CONDITIONAL GO ready-for-dev** — Pass 3 a appliqué tous les patches actionnables ; recommande Pass 4 Sonnet 4.6 pour vérification finale orthogonale OU continuer en dev-story si budget contraint. Path-dep 8-5b : helper `split::build_split_journal_entry` signature stable contractée + variant `AppError::AccountNotFound` extension cohérente avec futur batch validation 8-5b. | Claude (Opus 4.7 1M validate — VALIDATION FINALE) |
| **2026-05-09** | **Pass 4 validate Sonnet 4.6** — 4 findings (0 CRITICAL + 1 HIGH + 2 MEDIUM + 1 LOW). Ground-truth confirmés : 3 callsites AccountNotFound ✓, 29 sites accept_one ✓, 15 sites POST /accept ✓, 22 attributs sqlx::test (21 actifs + 1 ignored) ✓, 3 couches valueDate ✓. Patches appliqués : [H1] Résolution `bank_account.journal_account_id` inside `accept_one_split` : pattern lookup SELECT inside lock + mapping `FailedProposal { error_code: "BANK_ACCOUNT_NOT_CONFIGURED" }` per-proposal (pas AppError global). Liste validations à répliquer inside `accept_one_split` (steps 1bis + 2 + 6bis + 7 via FailedProposal). [M1] Dev Note §risques item 2 : stale claim « exactement 2 callsites » remplacé par directive correcte « 3 callsites, conforme F1''' Pass 3 ». [M2] Validations inside `accept_one_split` explicitement listées dans §note-implementation-accept (section H1 patch). [M3] T1.3 : `impl From<SplitImbalance> for ReconciliationError` doit être manuel (thiserror `#[from]` inapplicable sur struct-variant multi-champs) — code snippet ajouté dans T1.3. [L1] Vitest test locator `~69` → `grep -n 'expect(body.proposals)'` (robustesse si fichier bouge). Trend : Pass 1 = 7 → Pass 2 = 11 → Pass 3 = 11 → Pass 4 = 3 findings > LOW. **STOP cycle — 0 CRITICAL, 1 HIGH résolu, 2 MEDIUM résolus** — convergence atteinte. Recommandation : **GO ready-for-dev**. | Claude (Sonnet 4.6 validate) |
| **2026-05-12** | **bmad-dev-story Opus 4.7 single-pass continuous COMPLETED**. T1-T7 livrés. Status `ready-for-dev` → `in-progress` → `review`. Stats : 14 fichiers modifiés/créés. **Régression Pass 8 mésync remédiée** : variant `ReconciliationError::SplitImbalance` créé concrètement (Pass 4 documenté en spec mais jamais ajouté à errors.rs avant cette session) + `impl From<SplitImbalance>` manuel. 3 match blocks (`post_accept` / `post_reject` / `post_manual`) reçoivent la branche `SplitImbalance` pour exhaustivité (unreachable en pratique). T1 helper `kesh-reconciliation::split` (build_split_journal_entry N+1 lignes sign-aware + validate_split_balance Decimal exact) → 5 tests unit verts. T2 route POST /split (13 steps validation, sub-router comptable, audit `reconciliation.split_applied` snake_case top-level + sub-objects). T3 breaking Q2 `AcceptProposalInput` enum tagged Serde + `AcceptBodyExtractor` 400 (vs 422 Axum natif) + dispatch `accept_one` → `accept_one_invoice` + `accept_one_split` (29 sites `proposal.X` refactorés en params plain i64 ; lookup `journal_account_id` inside lock per-proposal H1 Pass 4). T3.3 migration 19 occurrences POST /accept body `type: 'invoice'` via `sed -i`. T3.4 fichier `reconciliation_split_e2e.rs` nouveau (10 tests E2E HTTP) + 2 tests discriminator dans `reconciliation_e2e.rs`. T5 frontend `TransactionSplitModal.svelte` (tableau éditable balance live indicator min 2 max 50) + bouton `Éclater` dans `ReconciliationProposals.svelte` + migration `acceptProposals` body avec discriminated union TS + `splitTransaction` API client. T6 i18n 5 clés `reconciliation-split-*` × 4 locales (lint-i18n-ownership PASS). T7 Playwright `reconciliation-split.spec.ts` (1 scénario empty + 1 axe a11y). Tests : **20/20 unit kesh-reconciliation** (5 nouveaux split) + **10/10 E2E HTTP reconciliation_split_e2e** + **23/23 + 1 ignored reconciliation_e2e** (post-migration 15 sites, 0 régression Q2) + 13/13 reconciliation_manual_e2e (0 régression 8-5a-base) + 9/9 bank_accounts_e2e (0 régression 8-5a-zero) + 223/223 Vitest frontend (215 prior + 8 nouveaux). Validation Test Locally First full stack verte : `cargo fmt --all -- --check` clean + `cargo build --workspace --all-targets` clean + `cargo clippy --workspace --all-targets -- -D warnings` clean + `npm run check` 0 errors + `npm run lint-i18n-ownership` PASS + `npm run build` clean. **Findings résiduels non-bloquants** (cf. Completion Notes) : (a) SELECT inline dans `accept_one_split` car helpers `bank_accounts/accounts` non Executor-generic — dette kesh-db refactor Story 11+ ; (b) `accept_one_split` n'émet pas `journal_entry.created` séparé (émis par `create_in_tx`) ; (c) `AcceptedProposal.invoice_id = 0` sentinel pour Split variant ; (d) `make_new_tx` signature 7 args dans split_e2e (vs 8 dans reconciliation_e2e). Lève L19/L20/L21 héritées 8-4. Path-dep descendante 8-5b débloquée (helper signature stable + variant Rule à ajouter). Prochaine étape : `bmad-code-review 8-5a-bis` cycle CLAUDE.md (auteur=Opus → Pass 1=Sonnet 4.6 pour briser biais d'auteur). | Claude (Opus 4.7 dev-story) |
| **2026-05-12** | **Pass 1 code-review Sonnet 4.6** — 34 findings bruts (Blind Hunter 19 + Edge Case Hunter 10 + Acceptance Auditor 5) → 30 distincts post-dédup. Verdict Acceptance Auditor : **CONDITIONAL GO** (1 MEDIUM AA-F1 + 4 LOW). 7 findings > LOW résiduels. **6 patches appliqués** : [P1 HIGH ECH-01] validation `splits[i].amount.scale() <= 2` step 2 (post_split + post_accept Split + accept_one_split defense-in-depth) — évite 500 DATABASE_ERROR sur INSERT `journal_entry_lines.debit DECIMAL(19,4)` en strict mode MariaDB ; [P2 HIGH ECH-02] guard `splits[i].counterpartyAccountId != bank_ledger_account_id` (post_split step 5 + accept_one_split step c) — évite JE self-referential balance-sheet no-op ; [P3 MEDIUM AA-F1] nouveau test E2E HTTP `accept_with_explicit_split_type_runs_split_flow` reconciliation_e2e.rs — coverage du flow `POST /accept type='split'` exerçant `accept_one_split` end-to-end ; [P4 MEDIUM BH-M6] retirer i18n key `reconciliation-split-success-toast` × 4 locales (jamais utilisée, cohérent ManualMatchModal sans toast) ; [P5 LOW BH-M4+ECH-07+AA-F3 merged] normaliser scale 2 décimales via `Decimal::rescale(2)` dans audit log `splits[*].amount` + RECONCILIATION_SPLIT_IMBALANCE error body (expected/actual/difference) — évite "10500" vs "10700.00" inconsistency ; [P6 LOW BH-L2] retirer dead code `_unused_naive_date_time` + import `NaiveDateTime` inutile. **Defers tracés dette tech v0.1 non-bloquants** : BH-H1 HIGH (invoice_id sentinel Split non-visible v0.1), BH-H2+ECH-03 HIGH/MEDIUM (TOCTOU balance pre-lock low-prob), BH-H4+H5+ECH-09 HIGH (frontend parseFloat+1e-6 cross-cutting v0.2 decimal.js), ECH-05 MEDIUM (batch size cap pré-existant 8-4 v0.2), BH-L3 LOW (SplitLineInput vs SplitProposalLine accepté noms par contexte), AA-F2 LOW (axe test n'ouvre pas modal — pattern 8-5a-base). Tests post-patches : 10/10 split_e2e + 24/24+1ign reconciliation_e2e (P3 nouveau test inclus) + 13/13 manual + 9/9 bank_accounts + 223/223 Vitest. Validation Test Locally First full stack verte. Trend : Pass 1 = 7 findings > LOW → 0 HIGH + 0 MEDIUM post-patches. | Claude (Sonnet 4.6 code-review) |
| **2026-05-12** | **Pass 2 code-review Haiku 4.5 — VALIDATION FINALE** — 25 findings bruts (Blind Hunter 8 + Edge Case Hunter 12 + Acceptance Auditor 5). Triage : **tous dismissés** post-vérification ground-truth. Faux positifs notables Blind Hunter Haiku : BH2-1 "missing scale() in post_split step 2" et BH2-2 "missing != bank_ledger guard in post_split step 5" — vérifiés présents lignes 2120 + 2172 du diff (Haiku a mal indexé le diff combiné des 2 commits). Autres BH2/ECH2 findings : duplicates Pass 1 defers, theoretical edges non-buggy, ou observations correctes mais déjà tracées Completion Notes (a). Verdict Acceptance Auditor Haiku : **`GO ready-for-merge`** — 0 CRITICAL / 0 HIGH / 0 MEDIUM, tous AC #93-#100 SATISFIED, scale normalization §audit-log-shape consistent, breaking Q2 AC #100 migration complète. Trend cycle complet : Pass 1 = 7 findings > LOW → Pass 2 = **0 finding > LOW post-triage**. **Critère d'arrêt CLAUDE.md ATTEINT Pass 2** — pas besoin de Pass 3 Opus. Cycle review STOP. Prochaine étape : push + ouverture PR vers main. | Claude (Haiku 4.5 code-review) |
