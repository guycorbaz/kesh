# Story 8-5a-bis: FR48 split + breaking POST /accept type discriminator (Q2)

Status: backlog

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
     (29 patches cumulés). Les sections §split-flow, §error-precedence-order
     sont transposées ici sans modification (sauf §split-flow step 5 où
     `bank_account.linked_account_id` → `bank_account.journal_account_id` automatique). -->

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

**Crate cible** : extension de `kesh-reconciliation` avec 1 nouveau module `split` (helper validateur balance + builder N+1 lignes). Le module `manual` existe déjà (livré 8-5a-base) et est composé par le helper split. Le module `rules` est livré par 8-5b.

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

   **Note** : `post_accept_filters_currency_mismatch` (ligne 1656) est `#[ignore]` (préservé pour Story 11 mono-CHF v0.1) — son body sera également patché pour cohérence post-un-ignore. **Total 16 sites à patcher (15 actifs + 1 ignored body placeholder)**.

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
   export async function splitTransaction(
       bankAccountId: number,
       bankTransactionId: number,
       splits: { counterpartyAccountId: number; amount: string; description: string }[],
       description?: string,
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
- 1 ligne banque agrégée (compte `bank_account_journal_id`, montant `tx.amount.abs()`, sign opposé à la majorité des splits selon sign(tx.amount)).
- N lignes contreparties (chaque `splits[i]` → 1 ligne, sign cohérent avec sign(tx.amount)).

**Rationale** : factorisation maximale du pattern. Le helper `build_journal_entry_for_counterparty` peut être appelé en boucle par `build_split_journal_entry` pour chaque ligne contrepartie, puis combiné en 1 NewJournalEntry à N+1 lignes. **Note implémentation** : le dev agent peut choisir de NE PAS littéralement composer (c'est-à-dire générer N NewJournalEntry indépendantes puis les fusionner) — c'est inefficace, donc préférer une implémentation directe à N+1 lignes dans `build_split_journal_entry` qui réutilise les sous-fonctions sign-resolver de manual.rs.

#### §validation-handler-side-split

**Ordre de validation pré-flight** (avant `with_account_lock`) :
1. Body validation Serde camelCase. `splits.len() >= 2 && splits.len() <= 50`.
2. `bankAccountId` cross-tenant : 404 `BANK_ACCOUNT_NOT_FOUND`.
3. `bank_account.journal_account_id IS Some(...)` : 412 `BANK_ACCOUNT_NOT_CONFIGURED`.
4. **Validation accounts batch** : pour chaque `splits[i].counterpartyAccountId`, vérifier `accounts::find_by_id_in_company` + `active=true`. Pattern : batch-load via une seule query `IN (...)` (cohérent `find_pending_by_ids` pattern reconciliation.rs:233) ou itération séquentielle si volume ≤ 50 (cap §split-flow inchangé). Tout split référençant un account inexistant ou archivé → `404 ACCOUNT_NOT_FOUND` body `details.missingAccountIds: [<ids>]` (camelCase JSON, list distincts triés).
5. `bankTransactionId` strictement pending : `find_strictly_pending_by_id_for_account` (helper 8-5a-base) → 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` si None.
6. `validate_split_balance(tx.amount, splits.iter().map(|s| s.amount))` → 400 `RECONCILIATION_SPLIT_IMBALANCE` si Err.

**Inside lock (advisory `with_account_lock`)** :
7. Re-fetch tx (TOCTOU defense pattern 8-4).
8. Resolve `entry_date` + `find_open_covering_date` → 409 `RECONCILIATION_FISCAL_YEAR_CLOSED`.
9. Build `NewJournalEntry` via `split::build_split_journal_entry`.
10. `journal_entries::create_in_tx`.
11. UPDATE bank_transactions optimistic lock → 409 si race.
12. Audit log `reconciliation.split_applied`.

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

**Décision Pass 3 Opus** : option (a) — enum typé `AcceptType { Invoice, Split }` (8-5a-bis livre 2 valeurs ; 8-5b ajoutera `Rule` ; `Manual` reporté v0.2 si demande utilisateur d'unifier /manual sur /accept). `String` libre rejeté car break diff-hostile + risque DoS.

**`AcceptProposalInput` après 8-5a-bis** :

```rust
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AcceptType {
    Invoice,
    Split,
    // Manual réservé v0.2 (route /manual standalone livrée 8-5a-base)
    // Rule réservé 8-5b
}

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
        // valueDate optional
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

**Note implémentation /accept type='split'** : 8-5a-bis peut soit (a) implémenter le flow `type='split'` complet dans `post_accept` (équivalent batch de `/split` standalone), soit (b) ne supporter que `type='invoice'` dans `/accept` et garder `/split` standalone. **Décision préférée 8-5a-bis** : option (a) — pour cohérence batch (le user peut accepter plusieurs proposals invoice + split en 1 POST). Si volume implémentation trop large, option (b) acceptable avec `type='split'` retourne 400 « non supporté v0.1, utiliser /reconciliation/split ».

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

**Plus 1 site ignored** (préservé pour Story 11) :
| # | Ligne | Test fonction parent | Notes |
|---|-------|---------------------|-------|
| 16 | dans `post_accept_filters_currency_mismatch` (1656) | body placeholder | `#[ignore]` Story 11 mono-CHF v0.1 — patch pour cohérence post-un-ignore |

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

100. **(Q2 — discriminator type obligatoire + migration tests 8-4)** Given un body proposal `{ bankTransactionId, invoiceId }` (legacy 8-4 sans `type`), When POST `/accept`, Then `400 Validation` avec message « champ `type` requis » (breaking change v0.1, pas de défaut). ET Given body `{ type: 'invoice', bankTransactionId, invoiceId }`, When POST `/accept`, Then flow 8-4 invoice exécuté (audit `reconciliation.accepted` + `invoice.paid`). ET Given le fichier `crates/kesh-api/tests/reconciliation_e2e.rs` (21 actifs + 1 ignored = 22 attributs `#[sqlx::test]`, vérification ground-truth Pass 3 Opus 4.7), When 8-5a-bis livré, Then les 15 sites POST /accept actifs + 1 site ignored ajoutent `type: 'invoice'` dans leur body et restent verts (régression non introduite). *Tests E2E HTTP : `accept_rejects_proposal_missing_type_discriminator` + `accept_with_explicit_invoice_type_runs_8_4_flow` + cargo `cargo test -p kesh-api --test reconciliation_e2e` 21 verts + 1 ignored.*

## Tasks / Subtasks

### T1. Helper `kesh-reconciliation::split::*` (AC #93-#96)

- [ ] T1.1 — Créer `crates/kesh-reconciliation/src/split.rs` (cf. §helper-split-signature) :
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

- [ ] T1.2 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod split;
  pub use split::{build_split_journal_entry, validate_split_balance, SplitDetail, SplitImbalance};
  ```

- [ ] T1.3 — Étendre `crates/kesh-reconciliation/src/errors.rs` (1 variant ajouté) :
  ```rust
  pub enum ReconciliationError {
      // ... 8-4 + 8-5a-base variants conservés (FiscalYearClosed)
      SplitImbalance { expected: Decimal, actual: Decimal, difference: Decimal },
  }
  ```

- [ ] T1.4 — Tests unit `kesh-reconciliation::split` (≥ 4) :
  1. `split_build_je_creates_n_plus_1_lines_for_debit_tx` (AC #93).
  2. `split_build_je_creates_n_plus_1_lines_for_credit_tx` (AC #94).
  3. `split_validate_balance_exact_match_ok` (AC #93).
  4. `split_validate_balance_imbalance_returns_error` (AC #95).

### T2. Route API `POST /api/v1/reconciliation/split` (AC #93-#99)

- [ ] T2.1 — Étendre `crates/kesh-api/src/routes/reconciliation.rs` avec handler `post_split` (cf. §validation-handler-side-split) :
  - Body `{ bankAccountId, bankTransactionId, splits: [...], valueDate? }` camelCase.
  - Pré-flight ordre §validation-handler-side-split étapes 1-6.
  - Inside lock : étapes 7-12.
  - **Différence majeure vs spec 8-5a unifiée** : pas de `bankLedgerAccountId` body. Résolu serveur-side via `bank_account.journal_account_id`.

- [ ] T2.2 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : ajouter `.route("/api/v1/reconciliation/split", post(routes::reconciliation::post_split))`.

- [ ] T2.3 — Étendre `crates/kesh-api/src/errors.rs` (variantes ajoutées/réutilisées) :
  - `AppError::ReconciliationSplitImbalance { expected, actual, difference }` → 400 `RECONCILIATION_SPLIT_IMBALANCE` body `details = { expected: '10700.00', actual: '10500.00', difference: '-200.00' }` (string Decimal cohérent AC #95).
  - `AppError::AccountNotFound { account_id, missing_account_ids: Option<Vec<i64>> }` (extension du variant 8-5a-zero/8-5a-base) → 404 `ACCOUNT_NOT_FOUND` body `{ error: { code, message, details: { accountId, missingAccountIds: [...] } } }` camelCase. `missingAccountIds` populated pour split (Vec d'ids invalides triés), single `accountId` pour manual.
  - `AppError::BankAccountNotConfigured` (réutiliser variant 8-5a-base).
  - `AppError::ReconciliationFiscalYearClosed` (réutiliser variant 8-5a-base).
  - `AppError::ReconciliationTransactionNotPending` (réutiliser variant 8-5a-base).
  - `AppError::ReconciliationOptimisticLockConflict` (réutiliser variant 8-4 / 8-5a-base).

### T3. Breaking change `POST /accept` discriminator type (AC #100)

- [ ] T3.1 — Modifier `post_accept` dans `crates/kesh-api/src/routes/reconciliation.rs` :
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
        },
    }
    ```
  - Match enum dans `accept_one` pour dispatcher vers flow `accept_invoice` (8-4 inchangé) ou `accept_split` (nouveau, équivalent batch de `/split`).
  - **Décision option (a) §note-implementation-accept** : implémenter `type='split'` dans `/accept` pour cohérence batch. Si volume trop large, fallback option (b) acceptable avec `type='split'` retourne 400 « non supporté v0.1, utiliser /reconciliation/split ».
  - Si `type` absent → 400 Validation message « champ `type` requis, valeurs acceptées v0.1 : ["invoice", "split"] » (Serde dispatch fail = 400/422 sérialisation, vérifier mappage).
  - Si `type` non reconnu → 400 Validation message « type inconnu ou non supporté : "<valeur>", valeurs acceptées v0.1 : ["invoice", "split"] ».

- [ ] T3.2 — Vérifier impact `Copy` : retirer `Copy` de `AcceptProposalInput` (F11'' Pass 3), ajouter `clone()` aux usages internes si nécessaire.

### T3.4. Migration tests E2E 8-4 existants (AC #100 part 3)

- [ ] T3.4 — Modifier `crates/kesh-api/tests/reconciliation_e2e.rs` (cf. §migration-21-tests) :
  - Patcher les 15 sites POST /accept actifs (lignes 757, 910, 1030, 1104, 1203, 1416, 1492, 1632, 1679, 1737, 1805, 1870, 1980, 2064, 2129) pour ajouter `type: 'invoice'` dans le body proposals[*].
  - Patcher 1 site ignored (dans `post_accept_filters_currency_mismatch` ligne 1656) pour cohérence post-un-ignore.
  - Vérifier `cargo test -p kesh-api --test reconciliation_e2e` retourne 21 verts + 1 ignored (= 22 attributs `#[sqlx::test]`).

### T3.5. Tests E2E HTTP nouveaux 8-5a-bis (AC #93-#100)

- [ ] T3.5 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_split_e2e.rs` *(nouveau fichier, ≥ 10 tests)* :
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

- [ ] T4.1 — Pas besoin de créer un nouveau helper : utiliser `fiscal_years::find_open_covering_date` existant (Story 3-7) avec `&mut tx_outer` passé depuis le handler `post_split` (cohérent 8-5a-base T4.1).

### T5. Frontend `TransactionSplitModal` + extensions (AC #93-#100 UI)

- [ ] T5.1 — Étendre `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
  ```ts
  // **Différence majeure vs spec 8-5a unifiée** : pas de `bankLedgerAccountId`.
  export async function splitTransaction(
      bankAccountId: number,
      bankTransactionId: number,
      splits: { counterpartyAccountId: number; amount: string; description: string }[],
      description?: string,
      valueDate?: string,
  ): Promise<{ bankTransactionId: number; journalEntryId: number }>;
  ```

- [ ] T5.2 — **Migration breaking change `acceptProposal`** : modifier `acceptProposal` pour ajouter `type: 'invoice'` explicite dans le body envoyé. Met à jour aussi `ReconciliationProposals.svelte` qui consomme. **Cohérent breaking Q2** :
  ```ts
  // Avant (8-4)
  body: { proposals: [{ bankTransactionId, invoiceId }] }
  // Après (8-5a-bis)
  body: { proposals: [{ type: 'invoice', bankTransactionId, invoiceId }] }
  ```

- [ ] T5.3 — Créer `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` :
  - Props : `bankTransaction`, `bankAccountId`.
  - Tableau splits éditable (ajout/suppression de ligne, min 2 max 50).
  - Sticker balance live computed `sum vs |tx.amount|` (vert si exact match, rouge sinon, avec différence affichée).
  - Bouton submit désactivé tant que balance ≠ exact OU `splits.length < 2`.
  - Gestion erreur `412 BANK_ACCOUNT_NOT_CONFIGURED` : message + lien `/bank-accounts`.

- [ ] T5.4 — Étendre `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - Pour chaque ligne tx avec `candidates: []` : 1 bouton « Éclater » (ouvre `TransactionSplitModal`). **Bouton « Affecter manuellement » déjà livré 8-5a-base** — coexistence à valider.
  - On modal success : refresh la liste.

- [ ] T5.5 — Tests Vitest (≥ 3-4) :
  1. `TransactionSplitModal: balance indicator updates live` (AC #93/#94).
  2. `TransactionSplitModal: submit disabled until balance exact match` (AC #93/#95).
  3. `acceptProposal sends type: 'invoice' in body` (régression breaking Q2).
  4. *(stretch)* `ReconciliationProposals: shows split button next to manual button for tx without candidate`.

### T6. i18n (AC implicite UI)

- [ ] T6.1 — Ajouter ~5 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixe strict `reconciliation-split-*`) :
  - `reconciliation-split-button-label`
  - `reconciliation-split-modal-title`
  - `reconciliation-split-balance-indicator`
  - `reconciliation-split-error-imbalance`
  - `reconciliation-split-success-toast`
  FR canonical.
- [ ] T6.2 — Traductions DE / IT / EN-CH — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [ ] T6.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T7. Tests E2E Playwright + a11y (AC #93-#99)

- [ ] T7.1 — Créer `frontend/tests/e2e/reconciliation-split.spec.ts` (≥ 1 actif) :
  1. `split end-to-end` : login Comptable, navigate `/reconciliation`, click « Éclater » sur tx -10700, ajouter 3 lignes (5000+4500+1200), vérifier balance indicator passe au vert, valider, vérifier toast succès + tx disparaît.

- [ ] T7.2 — Test a11y axe (AC #99) : 1 scénario sur la modal `TransactionSplitModal` ouvert — `expect(await new AxeBuilder().analyze()).toHaveNoViolations()`.

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
- `crates/kesh-db/src/repositories/accounts.rs` (utilise `find_by_id_in_company` + nouveau pattern batch)
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
- `crates/kesh-api/tests/reconciliation_e2e.rs` (migration 15 sites POST /accept + 1 ignored — T3.4)

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (~5 nouvelles clés `reconciliation-split-*` × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (extension `splitTransaction` + migration `acceptProposal` avec `type: 'invoice'`)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (extension bouton « Éclater »)
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts` *(nouveau, Vitest)*
- `frontend/tests/e2e/reconciliation-split.spec.ts` *(nouveau, Playwright)*

### Standards de test

- **Unit `kesh-reconciliation`** : `#[cfg(test)] mod tests` inline `split.rs`. ≥ 4 unit tests T1.4.
- **Intégration `kesh-db`** : pas de nouveau helper en 8-5a-bis (réutilisation 8-5a-base).
- **E2E HTTP `kesh-api`** : ≥ 10 nouveaux tests T3.5 + 15 sites migrés 8-4 (= 25 tests POST /accept actifs au final + 1 ignored). + 1 ou 2 tests accept-discriminator dans le fichier 8-4 existant.
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
| **2026-05-07** | Spec créée par re-split mécanique de 8-5a unifiée (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7). 8-5a-bis = FR48 split + breaking POST /accept Q2. **Différence majeure vs spec 8-5a unifiée** : le body POST `/split` n'inclut PAS `bankLedgerAccountId` — résolu serveur-side via `bank_account.journal_account_id` (foundation 8-5a-zero). Helper public `kesh-reconciliation::split::build_split_journal_entry` compose le pattern de `manual::build_journal_entry_for_counterparty` (8-5a-base). Migration nominale des 15 sites POST /accept dans `reconciliation_e2e.rs` (= 21 actifs + 1 ignored = 22 attributs `#[sqlx::test]` au total) pour Q2 breaking. 8 ACs (#93-#100). Tasks T1-T7. Path-dépendance bloquante : 8-5a-base `done`/merged. Status `8-5a-bis-split-breaking-accept: backlog`. | Claude (Opus 4.7 re-split workflow) |
