# Story 8-5a-zero: Foundation — `bank_account.journal_account_id` link

Status: ready-for-dev

<!-- Issue de re-split de Story 8-5a (`8-5a-reconciliation-manuelle-split.md`) le 2026-05-07 :
     Pass 3 validate Opus 4.7 sur 8-5a a révélé une dette architecturale critique F2''
     (`bank_account.journal_account_id` inexistant — anti-pattern UX `bankLedgerAccountId`
     dans le body POST /manual et /split). Décision Guy 2026-05-07 (option 3) :
     éviter la dette à la racine en re-splittant 8-5a en 3 sous-stories sur la frontière
     « foundation pose-pattern ».

     8-5a-zero (cette story) = foundation pure : ALTER TABLE + repo update + route PATCH +
     UI configuration. Aucune feature réconciliation. Démarre immédiatement.

     8-5a-base (FR45 manual match) ← path-dépendance sur 8-5a-zero (column existante).
     8-5a-bis  (FR48 split + breaking POST /accept Q2) ← path-dépendance sur 8-5a-base
                (helper public `manual::build_journal_entry_for_counterparty`).

     Voir `8-5a-reconciliation-manuelle-split.md` (status `archived-split-bis`) pour les
     décisions de conception détaillées validées sur 3 passes Sonnet→Haiku→Opus
     (29 patches cumulés). Les sections §audit-log-shapes, §rbac, §frontend-flow restent
     valables et seront transposées dans 8-5a-base et 8-5a-bis. -->

## Story

As a **administrateur Kesh (Path B production, configuration initiale ou évolution)**,
I want **lier chaque `bank_account` à un compte du plan comptable (typiquement classe 1 — Caisse banque 1020/1030) via une page de configuration dédiée**,
so that **lorsque j'utilise la réconciliation manuelle ou l'éclatement de transaction (livrés en 8-5a-base et 8-5a-bis), le compte comptable côté banque est résolu automatiquement par le serveur sans devoir le re-saisir à chaque fois — pattern UX cohérent avec `default_receivable_account_id` / `default_revenue_account_id` de Story 5-2**.

### Contexte

**Story 8-5a-zero = foundation du re-split 8-5a → 8-5a-zero / 8-5a-base / 8-5a-bis** (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7 sur 8-5a unifiée).

**Pourquoi 8-5a-zero en premier** : la spec 8-5a unifiée demandait `bankLedgerAccountId` dans le body POST `/manual` et `/split` parce que la table `bank_accounts` n'a **aucune colonne `journal_account_id`** dans le schéma v0.1 (vérifié migration `20260410000001_bank_accounts.sql` + entité `BankAccount` lignes 1-30). Cette dette architecturale F2'' a été détectée par Pass 3 Opus 4.7 (1M context, vérification ground truth) après Pass 1 Sonnet et Pass 2 Haiku qui l'ont manquée par biais d'auteur.

Le coût d'éliminer la dette à la racine (8-5a-zero) plutôt que la traîner dans `bankLedgerAccountId` body → 8-5a-base + 8-5a-bis :
- **UX** : le user configure 1 fois par bank_account au lieu de re-fournir le ledger account à chaque transaction (frontend a besoin de l'autocomplete classe 1, qui se simplifie ici).
- **Cohérence** : pattern `default_*_account_id` Story 5-2 (configuration explicite par entité) plutôt que body field anti-pattern.
- **Future-proof** : 8-5a-bis split + 8-5b rules engine héritent du résolu serveur-side, pas de dette technique propagée.

**Décision Guy 2026-05-07 verrouillée** : option 3 (re-split). Trend non-convergent 8-5a unifiée Pass 1 = 5 → Pass 2 = 6 → Pass 3 = 7 findings > LOW signait le gap structurel propagé à chaque passe. Le splitting est plus défensif que de tracer la dette en v0.2.

**8-5a-zero livre la foundation suivante (aucune feature réconciliation) :**
1. **Migration** : `ALTER TABLE bank_accounts ADD COLUMN journal_account_id BIGINT NULL` + index pour join performance.
2. **Repo** : nouvelle fonction `bank_accounts::set_journal_account_id_for_company(...)` + extension `find_by_id_for_company` retourne `journal_account_id`.
3. **Route API** : `PATCH /api/v1/bank-accounts/{id}` — endpoint dédié pour mettre à jour `journalAccountId`.
4. **Frontend** : page `/bank-accounts` (création nouvelle ou extension de la page existante minimale) avec dropdown account classe 1 (actifs).
5. **i18n + tests** : 5 clés × 4 locales + ~12 tests cumulés (sqlx + E2E HTTP + Vitest + Playwright).

**8-5a-zero ne livre PAS** :
- FR45 manual match (8-5a-base)
- FR48 split (8-5a-bis)
- Breaking change `POST /accept` discriminator type (8-5a-bis, Q2)
- Helper `kesh-reconciliation::manual::build_journal_entry_for_counterparty` (8-5a-base)
- Helper `kesh-reconciliation::split::*` (8-5a-bis)

**Status sprint** : `8-5a-zero-bank-account-journal-link: ready-for-dev` au moment de la création (2026-05-07). `8-5a-base` et `8-5a-bis` restent `backlog` jusqu'à 8-5a-zero `done`/merged.

**Pré-requis closed** :
- ✅ Story 8-1a/b/2/3/4 mergées sur main (foundation Epic 8 complète : import CAMT.053 + CSV + dedup + matching auto).
- ✅ Story 6-2 — multi-tenant scoping pattern KF-002 Pattern 1.
- ✅ Story 3-1 — entité `Account` chargée + `accounts::find_by_id_in_company` (helper réutilisé pour validation).
- ✅ Story 1-8 — RBAC sub-router pattern (Comptable+ pour mutations).
- ✅ Story 3-5 — audit log canonique (`audit_log::insert_in_tx`).

**Crate cible** : extension de `kesh-db::repositories::bank_accounts` (nouvelle fn + extension entité) + nouveau fichier `kesh-api::routes::bank_accounts.rs` (route PATCH minimale, pas de CRUD complet — le CRUD existant onboarding suffit pour v0.1, voir §rationale-route-minimale). Frontend : nouveau composant `BankAccountJournalLinkForm.svelte` ou extension de la page bank-accounts existante.

### Scope verrouillé — ce qui est livré par 8-5a-zero

1. **Migration `ALTER TABLE bank_accounts`** :
   - Colonne `journal_account_id BIGINT NULL` (nullable initialement — rows pré-migration restent NULL, le user **doit** configurer pour utiliser FR45/FR48 livrés en 8-5a-base/bis).
   - Index `idx_bank_accounts_journal_account ON bank_accounts (journal_account_id)` pour la jointure future GET /proposals (8-5a-base) qui aura besoin de charger le compte comptable.
   - **Pas de FK DB-level** vers `accounts.id` (cohérent décision Guy §schema-migration : éviter cycles + invariant applicatif handler-side suffit pour v0.1, le check d'intégrité est fait au PATCH).
   - **Pas de backfill automatique** : les rows existantes restent NULL. Le user configure manuellement via UI 8-5a-zero.

2. **Extension entité `BankAccount`** :
   - Champ `journal_account_id: Option<i64>` ajouté à `crates/kesh-db/src/entities/bank_account.rs`.
   - Tous les `SELECT id, company_id, bank_name, iban, qr_iban, is_primary, version, created_at, updated_at FROM bank_accounts` du repo `bank_accounts.rs` étendus pour inclure `journal_account_id` (4 occurrences au minimum à patcher : `FIND_BY_ID_SQL` const + `find_primary` + `list_by_company` + `upsert_primary` SELECT FOR UPDATE).
   - **Sérialisation** : `journalAccountId` (camelCase JSON, cohérent convention `kesh-api`).

3. **Repo `bank_accounts::set_journal_account_id_for_company`** :
   ```rust
   /// Met à jour le `journal_account_id` d'un bank_account scopé multi-tenant.
   ///
   /// Story 8-5a-zero — pose le pattern `bank_account.journal_account_id` qui sera
   /// consommé par 8-5a-base (manual match) et 8-5a-bis (split) sans body field
   /// `bankLedgerAccountId` (résolu serveur-side via cette colonne).
   ///
   /// Optimistic lock sur `version` (cohérent KF-004). Retourne le bank_account
   /// mis à jour (avec `version` incrémenté). 404 si introuvable cross-tenant.
   pub async fn set_journal_account_id_for_company(
       pool: &MySqlPool,
       company_id: i64,
       id: i64,
       journal_account_id: Option<i64>,
       expected_version: i32,
   ) -> Result<BankAccount, DbError>
   ```
   SQL : `UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 WHERE company_id = ? AND id = ? AND version = ?`. `rows_affected() == 0` → `DbError::OptimisticLockConflict` ou `DbError::NotFound` (selon le pattern `find` pré-flight cf. §rationale-pattern-find-then-update).

4. **Route API `PATCH /api/v1/bank-accounts/{id}`** :
   - Sub-router `comptable_routes` (RBAC Comptable+, pattern 8-1b T6.3).
   - Body : `{ journalAccountId: number | null, version: number }` (champ unique mutable v0.1 — pas de patch pour `bank_name`/`iban` puisque l'onboarding existant gère, voir §rationale-route-minimale).
   - Validation handler-side :
     - `version >= 1` (i32, validation Serde).
     - Si `journalAccountId !== null` :
       - `journalAccountId > 0` (i64, validation body).
       - `accounts::find_by_id_in_company(pool, journalAccountId, company_id)` retourne `Some(Account)` (404 `ACCOUNT_NOT_FOUND` sinon).
       - `account.active == true` (404 `ACCOUNT_NOT_FOUND` sinon — anti-énumération).
       - `account.account_type == AccountType::Asset || account.account_type == AccountType::Liability` (400 `INVALID_ACCOUNT_TYPE` sinon — un compte bancaire doit être un actif/passif comptable, jamais Revenue/Expense).
       - **Note classe** : la spec 8-5a unifiée notait que l'entité `Account` n'a pas de champ `class` direct (seul `account_type: AccountType { Asset, Liability, Revenue, Expense }`). Le contrôle de classe stricte (« classe 1 » vs « classe 2 ») est reporté à un check `number.starts_with("1")` côté frontend (filtre dropdown UX) — pas d'invariant serveur fort v0.1 (pattern cohérent avec validation invoice 5-2 qui se contente du `account_type` côté backend).
     - Si `journalAccountId === null` : permis (le user peut « délier » un compte).
   - Audit log : action `bank_account.updated` avec `details_json` snake_case top-level :
     ```json
     {
       "bank_account_id": 17,
       "before": { "journal_account_id": null, "version": 3 },
       "after": { "journal_account_id": 1020, "version": 4 }
     }
     ```
   - Response : `200 OK` avec body `BankAccount` mis à jour (camelCase JSON — `journalAccountId`).
   - Erreurs typées : `404 BANK_ACCOUNT_NOT_FOUND` (cross-tenant ou inexistant), `404 ACCOUNT_NOT_FOUND` (journalAccountId invalide ou archivé), `400 INVALID_ACCOUNT_TYPE` (Revenue/Expense), `409 OPTIMISTIC_LOCK_CONFLICT` (version mismatch).

5. **Nouveau fichier `crates/kesh-api/src/routes/bank_accounts.rs`** *(non existant — voir §rationale-route-minimale)* :
   - 1 handler `patch_bank_account_journal_link`.
   - Mounting dans `crates/kesh-api/src/lib.rs` sous `comptable_routes`.

6. **Frontend extensions** :
   - **Décision §frontend-page-strategy** : la page `/bank-accounts/+page.svelte` existe déjà (vide v0.1, voir résultat `ls frontend/src/routes/(app)/bank-accounts/`). 8-5a-zero **étend cette page** plutôt que d'en créer une nouvelle (cohérent splitting préventif : pas de double-route bank-accounts).
   - Composant `BankAccountList.svelte` (table : id, bank_name, iban, journalAccountId via `accounts.number + name`, action « Lier compte comptable »).
   - Composant `BankAccountJournalLinkForm.svelte` (modal ou inline) :
     - Dropdown `Account` autocomplete (filtre client-side `account_type === 'Asset' || account_type === 'Liability'` ET `number.startsWith('1')` pour UX classe 1 — actifs typiques 1020 Caisse banque, 1030 Banque, etc.).
     - Bouton « Lier » (PATCH avec `journalAccountId`) ou « Délier » (PATCH avec `journalAccountId: null`).
   - **API client** : extension `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` (nouveau fichier) avec `listBankAccounts()` (GET) et `updateBankAccountJournalLink(id, journalAccountId, version)`.

7. **i18n** : 5 nouvelles clés × 4 locales :
   - `bank-accounts-labels-page-title`
   - `bank-accounts-labels-journal-account-id`
   - `bank-accounts-actions-link-account`
   - `bank-accounts-actions-unlink-account`
   - `bank-accounts-errors-invalid-account-type`
   FR canonical, traductions DE/IT/EN-CH non-copies (lesson 8-2 H13).

8. **Tests** :
   - **Unit** : N/A (pas de helper pure dans 8-5a-zero — la validation est handler-side dans kesh-api, couverte par tests E2E HTTP).
   - **Integration `kesh-db`** : 4-5 tests `#[sqlx::test]` :
     1. `set_journal_account_id_updates_column_and_bumps_version` (happy path).
     2. `set_journal_account_id_returns_optimistic_lock_conflict_on_version_mismatch`.
     3. `set_journal_account_id_does_not_leak_cross_tenant` (company_A bank_account, company_B caller → ne trouve pas).
     4. `set_journal_account_id_to_null_unlinks_successfully`.
     5. `find_by_id_for_company_returns_journal_account_id_when_set` (extension entité validée).
   - **E2E HTTP `kesh-api`** : 3-4 tests dans nouveau fichier `crates/kesh-api/tests/bank_accounts_e2e.rs` :
     1. `patch_bank_account_links_journal_account_returns_200_with_updated_entity` (AC #75 happy).
     2. `patch_bank_account_rejects_archived_account_with_404` (AC #78).
     3. `patch_bank_account_rejects_revenue_account_with_400_invalid_type` (AC #79).
     4. `patch_bank_account_does_not_leak_cross_tenant_account` (AC #80, KF-002 pattern).
   - **Vitest frontend** : 2-3 tests :
     1. `BankAccountJournalLinkForm: filters dropdown to asset accounts class 1`.
     2. `BankAccountJournalLinkForm: disables submit on no change`.
     3. `bank-accounts.api: PATCH sends correct body shape`.
   - **Playwright** : 1-2 actifs :
     1. `bank-account-journal-link.spec.ts` : login Comptable, navigate `/bank-accounts`, click « Lier compte comptable » sur un bank_account avec `journalAccountId === null`, sélectionner « 1020 Caisse banque », valider, vérifier toast succès + refresh liste affiche le compte lié.
     2. axe a11y scan sur la modal/form (AC #82).

9. **Sync** sprint-status — pas d'autre action 8-5a-zero (pas de KF/CR pré-tracée).

**HORS scope 8-5a-zero (→ 8-5a-base / 8-5a-bis / v0.2) :**

- Routes `POST /reconciliation/manual` ou `/split` — 8-5a-base / 8-5a-bis.
- Helper `kesh-reconciliation::manual::*` ou `::split::*` — 8-5a-base / 8-5a-bis.
- Migration tests E2E HTTP 8-4 (21 actifs avec `type: 'invoice'`) — 8-5a-bis (Q2 breaking change).
- Page `/reconciliation/rules` ou table `reconciliation_rules` — 8-5b.
- POST/DELETE pour `bank_accounts` — pas de scope v0.1 (l'onboarding gère la création initiale, le DELETE est tracé en CR potentielle si demande utilisateur).
- Configuration multi-bank_account (un user peut avoir plusieurs bank_accounts, chacun avec son propre `journal_account_id` — 8-5a-zero **supporte** cela via la route PATCH par id, mais la création de bank_accounts secondaires hors onboarding n'est pas couverte v0.1).
- Page d'audit log dédiée `bank_account.updated` — l'audit log existant `audit_log` couvre, pas d'UI dédiée v0.1.

### Décisions de conception

#### §schema-migration

**Décision** : `journal_account_id BIGINT NULL` (nullable initialement) + pas de FK DB-level.

**Rationale** :
- **Nullable initial** : permet le `ALTER TABLE` non-bloquant en ALGORITHM=INSTANT (cohérent migration 8-4 pattern `auto_match_rejected_at NULL`). Pas de backfill automatique — les rows existantes héritent NULL et le user **doit** configurer pour utiliser FR45/FR48 (validation 412 PRECONDITION_FAILED `BANK_ACCOUNT_NOT_CONFIGURED` côté handler 8-5a-base/bis quand `bank_account.journal_account_id IS NULL`).
- **Pas de FK DB-level** : la table `accounts` est tenant-scoped via `company_id`, et la FK `bank_accounts.journal_account_id REFERENCES accounts(id)` ne peut **pas** garantir la cohérence company sans CHECK trigger (pattern non supporté MariaDB 10.x sans DELIMITER). L'invariant `bank_account.company_id == account.company_id` est appliqué côté handler PATCH (check `accounts::find_by_id_in_company(pool, journal_account_id, company_id)` retourne `Some`). FK à ajouter v0.2 si MariaDB 10.5+ avec CHECK plus strict.

**SQL migration** :
```sql
-- Migration : ajouter `journal_account_id` à bank_accounts pour lier au plan comptable.
-- Story 8-5a-zero — foundation pour FR45 manual match (8-5a-base) et FR48 split (8-5a-bis).
--
-- ALGORITHM=INSTANT évite la copie complète de la table sur MariaDB 10.3+ (instant
-- ADD COLUMN nullable). LOCK=NONE garantit la concurrent DML pendant la migration.
ALTER TABLE bank_accounts
    ADD COLUMN journal_account_id BIGINT NULL AFTER qr_iban,
    ALGORITHM=INSTANT, LOCK=NONE;

-- Index pour les jointures futures GET /proposals (8-5a-base) qui chargent
-- le compte comptable lié au bank_account pour résoudre serveur-side le
-- ledger account du flow manual/split.
CREATE INDEX IF NOT EXISTS idx_bank_accounts_journal_account
    ON bank_accounts (journal_account_id);
```

#### §validation-account-type

**Décision** : valider côté handler que `account.account_type IN (Asset, Liability)` — un compte bancaire ne peut être lié qu'à un compte d'actif (typique 1020 Caisse, 1030 Banque) ou de passif (rarement, e.g. découvert chronique 2100). **Refus** : Revenue/Expense (400 `INVALID_ACCOUNT_TYPE`).

**Rationale** : cohérent avec le sens comptable. Un debit de bank_account génère un crédit côté ledger account banque (compte d'actif diminue) ; impossible avec un Revenue/Expense.

**Note frontend** : le dropdown filtre côté client `account_type === 'Asset' || account_type === 'Liability'` ET `number.startsWith('1') || number.startsWith('2')` pour l'UX (montrer principalement classe 1, optionnellement classe 2). Le filtre serveur reste sur `account_type` (Account.number n'est pas filtré côté backend pour éviter la complexité, l'UX frontend suffit).

#### §audit-log

**Décision** : action `bank_account.updated` (cohérent §audit-log Story 8-1b qui émet `bank_account.created` à l'onboarding). `details_json` shape snake_case top-level :

```json
{
  "bank_account_id": 17,
  "before": { "journal_account_id": null, "version": 3 },
  "after":  { "journal_account_id": 1020, "version": 4 }
}
```

**Rationale** : pattern `before/after` cohérent avec audit log Story 7-3 (KF-004 `is_no_op_change` court-circuit + sub-objets). Si le PATCH est un no-op (même `journal_account_id` que l'existant), le handler court-circuite **avant** d'écrire l'audit log (cohérent KF-004 — pas d'audit pour des changements vides).

#### §rbac

**Décision** : route `PATCH /api/v1/bank-accounts/{id}` sous `comptable_routes` (Comptable+). `Consultation` retourne 403 (cohérent toutes les mutations bank_*).

#### §frontend-flow

**Décision** : étendre la page `/bank-accounts/+page.svelte` existante (vide v0.1) plutôt que créer un sous-flow. Composants :
- `BankAccountList.svelte` (haut de page, table simple).
- `BankAccountJournalLinkForm.svelte` (inline ou modal-on-click sur chaque row).

**Rationale** : pas de page dédiée `/bank-accounts/{id}/edit` car le scope v0.1 est minime (1 champ mutable). Si une refonte CRUD complète est demandée v0.2, créer une route dédiée à ce moment-là.

#### §rationale-route-minimale

**Décision** : créer un nouveau fichier `crates/kesh-api/src/routes/bank_accounts.rs` contenant **uniquement** le handler `PATCH /bank-accounts/{id}`. Pas de POST/GET/DELETE handlers v0.1.

**Rationale** :
- **POST** : la création de bank_account passe par le flow d'onboarding existant (`onboarding::set_bank_account`) qui appelle `bank_accounts::upsert_primary`. Pas de besoin v0.1 de créer un bank_account hors onboarding (les users primaires ont 1 seul bank_account, ce qui couvre 95% des PME suisses).
- **GET list/detail** : pas de scope v0.1. Le composant `BankAccountList.svelte` utilise un client-side fetch direct via une nouvelle route GET (à inclure dans la même route file pour cohérence — voir précision T3.1).
- **DELETE** : la suppression d'un bank_account est complexe (impact multi-table : `bank_imports`, `bank_transactions`). Reportée v0.2 si CR utilisateur (pas de CR ouverte aujourd'hui).

**Précision T3.1 update** : ajouter un handler `GET /api/v1/bank-accounts` (`authenticated_routes`, tous rôles authentifiés peuvent lire) qui retourne la liste des bank_accounts de la company. Trivial (réutilise `bank_accounts::list_by_company`). 1 test E2E HTTP supplémentaire (5 au lieu de 4).

#### §rationale-pattern-find-then-update

**Décision** : pattern « SELECT FOR UPDATE inside tx + UPDATE optimistic lock » dans `set_journal_account_id_for_company` (cohérent Story 7-3 KF-004 + Story 8-1b `upsert_primary`). Distinguer 404 (introuvable / cross-tenant) vs 409 OptimisticLockConflict (version mismatch) côté handler.

```rust
pub async fn set_journal_account_id_for_company(...) -> Result<BankAccount, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT ... FROM bank_accounts WHERE company_id = ? AND id = ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let existing = match existing {
        Some(b) => b,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
    };

    // KF-004 court-circuit no-op
    if existing.journal_account_id == journal_account_id {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(existing);
    }

    let rows = sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 \
         WHERE id = ? AND version = ?",
    )
    .bind(journal_account_id)
    .bind(id)
    .bind(expected_version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(updated)
}
```

## Acceptance Criteria

ACs #75-#82 (8 ACs).

### Foundation column + repo

75. **(Migration — ajout column nullable)** Given le schéma v0.1 sans `bank_accounts.journal_account_id`, When la migration `20260507200001_bank_account_journal_link.sql` est appliquée, Then la colonne `journal_account_id BIGINT NULL` existe sur `bank_accounts` ET l'index `idx_bank_accounts_journal_account` est créé ET les rows existantes ont `journal_account_id = NULL` (pas de backfill). *Test sqlx : `migration_creates_journal_account_id_column_nullable`.*

76. **(Repo — set_journal_account_id_for_company happy path)** Given un bank_account `id=17, company_id=1, version=3, journal_account_id=NULL` et un account `id=1020, company_id=1, account_type='Asset', active=true`, When `set_journal_account_id_for_company(pool, 1, 17, Some(1020), 3)`, Then `Ok(BankAccount { journal_account_id: Some(1020), version: 4, ... })` ET la row DB est mise à jour ET l'audit log `bank_account.updated` est émis. *Test sqlx : `set_journal_account_id_updates_column_and_bumps_version`.*

77. **(Repo — optimistic lock conflict)** Given `bank_account.version=3`, When `set_journal_account_id_for_company(..., expected_version=2)`, Then `Err(DbError::OptimisticLockConflict)`. *Test sqlx : `set_journal_account_id_returns_optimistic_lock_conflict_on_version_mismatch`.*

### Route PATCH /api/v1/bank-accounts/{id}

78. **(Route happy path — link)** Given user company_1 Comptable, bank_account `id=17, version=3, journal_account_id=NULL`, account `id=1020 Asset active=true company_1`, When `PATCH /api/v1/bank-accounts/17 { journalAccountId: 1020, version: 3 }`, Then `200 OK` body `{ id: 17, journalAccountId: 1020, version: 4, ... }` ET audit log `bank_account.updated` avec `details.before.journal_account_id = null` + `details.after.journal_account_id = 1020`. *Test E2E HTTP : `patch_bank_account_links_journal_account_returns_200_with_updated_entity`.*

79. **(Route — archived account 404)** Given account `id=1020 active=false`, When PATCH journalAccountId=1020, Then `404 ACCOUNT_NOT_FOUND` (anti-énumération, cohérent AC #82 8-5a unifiée). *Test E2E HTTP : `patch_bank_account_rejects_archived_account_with_404`.*

80. **(Route — wrong account type 400)** Given account `id=4000 account_type=Revenue`, When PATCH journalAccountId=4000, Then `400 INVALID_ACCOUNT_TYPE` body `{ error: { code: "INVALID_ACCOUNT_TYPE", message: t(...), details: { accountType: "Revenue", allowedTypes: ["Asset", "Liability"] } } }`. *Test E2E HTTP : `patch_bank_account_rejects_revenue_account_with_400_invalid_type`.*

81. **(Route — multi-tenant safety)** Given user company_A, account `id=1020 company_id=B`, When PATCH bank_account de company_A avec journalAccountId=1020, Then `404 ACCOUNT_NOT_FOUND` (KF-002 pattern, pas 403). *Test E2E HTTP : `patch_bank_account_does_not_leak_cross_tenant_account`.*

82. **(Route — RBAC + a11y modal)** Given user `Consultation`, When PATCH bank_account, Then `403 Forbidden`. ET Given `BankAccountJournalLinkForm` ouvert (modal ou inline), When axe-core scan, Then 0 violation. *Tests E2E HTTP : `patch_bank_account_requires_comptable_role`. Test Playwright : `accessibility — bank-account-journal-link form axe scan`.*

## Tasks / Subtasks

### T1. Migration `bank_accounts.journal_account_id` (AC #75)

- [ ] T1.1 — Créer `crates/kesh-db/migrations/20260507200001_bank_account_journal_link.sql` avec le SQL §schema-migration ci-dessus.
- [ ] T1.2 — Vérifier `cargo test -p kesh-db --lib test_fixtures` (truncate inventory, leçon 8-1b hotfix `8046f04`) — pas de modification attendue de `TABLES_TO_TRUNCATE` (la table existe déjà), mais lancer pour vérifier que le truncate marche avec la nouvelle column.

### T2. Extension entité `BankAccount` + repo `set_journal_account_id_for_company` (AC #76, #77)

- [ ] T2.1 — Étendre `crates/kesh-db/src/entities/bank_account.rs` :
  ```rust
  pub struct BankAccount {
      // ... champs existants
      pub journal_account_id: Option<i64>,
  }
  ```
  Sérialisation `journalAccountId` camelCase via `#[serde(rename_all = "camelCase")]` (cohérent `Account` entity).

- [ ] T2.2 — Patcher les 4 SELECT SQL dans `crates/kesh-db/src/repositories/bank_accounts.rs` pour inclure `journal_account_id` :
  - `FIND_BY_ID_SQL` constante.
  - `find_primary` SELECT.
  - `find_by_id_for_company` SELECT.
  - `list_by_company` SELECT.
  - `upsert_primary` SELECT FOR UPDATE.
  - **Précaution** : `upsert_primary` ne met PAS à jour `journal_account_id` (il reste préservé sur l'UPDATE existing — vérifier que la branche `Some(account)` ne touche pas la colonne, et que la branche `None` (INSERT) n'inclut pas la colonne dans VALUES — laisse à NULL par défaut DB).

- [ ] T2.3 — Ajouter `set_journal_account_id_for_company` dans `bank_accounts.rs` (cf. §rationale-pattern-find-then-update ci-dessus).

- [ ] T2.4 — Tests `#[sqlx::test]` (≥ 5) :
  1. `set_journal_account_id_updates_column_and_bumps_version` (AC #76).
  2. `set_journal_account_id_returns_optimistic_lock_conflict_on_version_mismatch` (AC #77).
  3. `set_journal_account_id_does_not_leak_cross_tenant`.
  4. `set_journal_account_id_to_null_unlinks_successfully`.
  5. `find_by_id_for_company_returns_journal_account_id_when_set` (régression entité).

- [ ] T2.5 — Vérifier `cargo test -p kesh-db bank_accounts` MariaDB up local (lesson 8-3 retro).

### T3. Route API `PATCH /api/v1/bank-accounts/{id}` + GET /api/v1/bank-accounts (AC #78-#82)

- [ ] T3.1 — Créer `crates/kesh-api/src/routes/bank_accounts.rs` :
  - Handler `patch_bank_account_journal_link` :
    - Validation Serde body `{ journalAccountId: Option<i64>, version: i32 }` camelCase.
    - Fetch existing via `bank_accounts::find_by_id_for_company(pool, company_id, id)` → 404 `BANK_ACCOUNT_NOT_FOUND` si None.
    - Si `journalAccountId.is_some()` :
      - `accounts::find_by_id_in_company(pool, journal_account_id, company_id)` → 404 `ACCOUNT_NOT_FOUND` si None.
      - Check `account.active == true` → 404 `ACCOUNT_NOT_FOUND` si false (anti-énumération).
      - Check `account.account_type IN (Asset, Liability)` → 400 `INVALID_ACCOUNT_TYPE` sinon.
    - Appel `bank_accounts::set_journal_account_id_for_company(pool, company_id, id, journal_account_id, version)` :
      - `Err(DbError::OptimisticLockConflict)` → 409 `OPTIMISTIC_LOCK_CONFLICT`.
      - `Err(DbError::NotFound)` → 404 `BANK_ACCOUNT_NOT_FOUND` (race entre find + update).
    - Audit log `bank_account.updated` avec `details_json` shape §audit-log.
    - Response 200 OK avec `BankAccount` mis à jour.
  - Handler `list_bank_accounts` (GET, authenticated tous rôles) :
    - Réutilise `bank_accounts::list_by_company(pool, company_id)`.
    - Response 200 OK avec `Vec<BankAccount>` JSON.

- [ ] T3.2 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : `.route("/api/v1/bank-accounts/{id}", patch(routes::bank_accounts::patch_bank_account_journal_link))`.
  - `authenticated_routes` : `.route("/api/v1/bank-accounts", get(routes::bank_accounts::list_bank_accounts))`.
  - Déclarer `pub mod bank_accounts;` dans `routes/mod.rs`.

- [ ] T3.3 — Étendre `crates/kesh-api/src/errors.rs` :
  - `AppError::BankAccountNotFound { bank_account_id: i64 }` → 404 `BANK_ACCOUNT_NOT_FOUND` (réutiliser variant 8-1b si déjà existant — vérifier).
  - `AppError::AccountNotFound { account_id: i64 }` → 404 `ACCOUNT_NOT_FOUND` (variant à créer si inexistant — F10'' Pass 3 8-5a confirmait absence). Body `{ error: { code, message, details: { accountId } } }` camelCase.
  - `AppError::InvalidAccountType { account_id, account_type, allowed_types }` → 400 `INVALID_ACCOUNT_TYPE` (variant à créer).
  - `AppError::OptimisticLockConflict` (variant existant — vérifier réutilisation) → 409.

- [ ] T3.4 — Tests E2E HTTP `crates/kesh-api/tests/bank_accounts_e2e.rs` *(nouveau fichier, ≥ 5 tests)* :
  1. `patch_bank_account_links_journal_account_returns_200_with_updated_entity` (AC #78).
  2. `patch_bank_account_rejects_archived_account_with_404` (AC #79).
  3. `patch_bank_account_rejects_revenue_account_with_400_invalid_type` (AC #80).
  4. `patch_bank_account_does_not_leak_cross_tenant_account` (AC #81).
  5. `patch_bank_account_requires_comptable_role` (AC #82).
  6. *(stretch)* `list_bank_accounts_returns_journal_account_id_when_set`.

### T4. Frontend page `/bank-accounts` extension (AC #78-#82 UI)

- [ ] T4.1 — Créer `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` :
  ```ts
  export interface BankAccountSummary {
      id: number;
      bankName: string;
      iban: string;
      qrIban: string | null;
      isPrimary: boolean;
      journalAccountId: number | null;
      version: number;
  }

  export async function listBankAccounts(): Promise<BankAccountSummary[]>;

  export async function updateBankAccountJournalLink(
      id: number,
      journalAccountId: number | null,
      version: number,
  ): Promise<BankAccountSummary>;
  ```

- [ ] T4.2 — Créer `frontend/src/lib/features/bank-accounts/BankAccountList.svelte` :
  - Table : Bank name, IBAN, Compte comptable lié (number + name via fetch accounts), Action « Lier » / « Délier ».
  - Bouton click ouvre `BankAccountJournalLinkForm` (modal ou inline).

- [ ] T4.3 — Créer `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte` :
  - Props : `bankAccount: BankAccountSummary`, `accounts: Account[]` (chargé via API existante `accounts::list_by_company`).
  - Dropdown filtré client-side : `account_type === 'Asset' || account_type === 'Liability'` ET `(number.startsWith('1') || number.startsWith('2'))` (UX classe 1/2). Réutiliser `AccountAutocomplete.svelte` si compatible.
  - Bouton « Lier » (PATCH avec `journalAccountId`) ou « Délier » (PATCH avec `journalAccountId: null`).
  - On submit success : event `success`, refresh liste.

- [ ] T4.4 — Étendre `frontend/src/routes/(app)/bank-accounts/+page.svelte` :
  - Mount `BankAccountList`.
  - Chargement initial via `listBankAccounts()` + `accounts::list_by_company()`.

- [ ] T4.5 — Tests Vitest (≥ 2-3) :
  1. `BankAccountJournalLinkForm: filters dropdown to asset accounts class 1`.
  2. `BankAccountJournalLinkForm: disables submit on no change` (no-op KF-004 cohérent).
  3. `bank-accounts.api: PATCH sends correct body shape with version`.

### T5. i18n (AC implicite UI)

- [ ] T5.1 — Ajouter 5 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` :
  - `bank-accounts-labels-page-title = Comptes bancaires`
  - `bank-accounts-labels-journal-account-id = Compte comptable lié`
  - `bank-accounts-actions-link-account = Lier au plan comptable`
  - `bank-accounts-actions-unlink-account = Délier`
  - `bank-accounts-errors-invalid-account-type = Type de compte invalide (Actif ou Passif requis)`
  FR canonical.
- [ ] T5.2 — Traductions DE / IT / EN-CH — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse (DE : « Verbindung mit Kontorahmen »).
- [ ] T5.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T6. Tests E2E Playwright + a11y (AC #82)

- [ ] T6.1 — Créer `frontend/tests/e2e/bank-account-journal-link.spec.ts` (≥ 1 actif) :
  1. `bank-account journal link end-to-end` : login Comptable, navigate `/bank-accounts`, click « Lier » sur un bank_account avec `journalAccountId === null`, sélectionner « 1020 Caisse banque » dans dropdown, valider, vérifier toast succès + le compte lié apparaît dans la liste.

- [ ] T6.2 — Test a11y axe (AC #82) : 1 scénario sur la modal/form ouvert — `expect(await new AxeBuilder().analyze()).toHaveNoViolations()`.

## Risque de splitting

**Modules touchés** :
1. `crates/kesh-db/migrations` (1 migration triviale).
2. `crates/kesh-db/src/entities/bank_account.rs` (1 champ ajouté).
3. `crates/kesh-db/src/repositories/bank_accounts.rs` (1 nouvelle fn + 4 SELECT SQL patches).
4. `crates/kesh-api/src/routes/bank_accounts.rs` (1 nouveau fichier, 2 handlers).
5. `crates/kesh-api/src/errors.rs` (3 variants).
6. `crates/kesh-i18n` (5 clés × 4 locales).
7. `frontend/src/lib/features/bank-accounts` (3 fichiers).
8. `frontend/src/routes/(app)/bank-accounts/+page.svelte` (extension).

**Total : 8 modules**. Au-dessus du seuil CLAUDE.md « splitter si > 5 modules ». **Pas de re-split** car (a) le scope est mécanique (migration + repo + route + UI), aucune logique métier complexe, (b) les modules sont tous sur le même chemin de dépendance (DB → API → Frontend), pas de cross-cutting.

**Volume estimé** : ~400-500 lignes spec actuelle + ~600-800 lignes code/tests à implémenter = bien en-dessous du seuil 1500 lignes 8-4 retro.

**Aucune dérogation nécessaire**.

## Dev Notes

### API surface livrée Epic 8 — patterns à réutiliser

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403.
- **Audit log atomique** : helper `audit_log::insert_in_tx(tx, NewAuditLogEntry { ... })`. Pattern `before/after` pour les `*.updated` actions (Story 7-3).
- **Erreurs structurées** : `AppError::*` typé (préféré sur `Custom`). Body camelCase JSON.
- **i18n key ownership** : préfixe strict, kebab-case, lint-i18n-ownership pass (Story 6-3).
- **Repository pattern + sqlx** : Executor générique `<E: Executor>` (pattern 8-3 / 8-4).
- **Optimistic lock** : `version` column + UPDATE `... AND version = ?` + 409 sur `rows_affected() == 0` (cohérent KF-004).
- **No-op short-circuit** : `is_no_op_change(...)` court-circuit AVANT toute mutation (cohérent KF-004 + bank_accounts existant).

### Patterns architecturaux à respecter

- **Pas de FK DB-level cross-tenant** : `bank_accounts.journal_account_id` reste sans FK explicite (l'invariant company match est handler-side).
- **`account_type` validation** : Asset/Liability autorisés, Revenue/Expense rejetés. Pas de check `number` strict côté backend (UX frontend suffit pour v0.1).
- **Body PATCH minimal** : un seul champ mutable (`journalAccountId`) v0.1. Si besoin futur de patcher `bank_name` / `iban` hors onboarding, créer un handler dédié (pas de mega-PATCH multi-fields ambigu).

### Source tree à toucher

**DB** :
- `crates/kesh-db/migrations/20260507200001_bank_account_journal_link.sql` *(nouveau)*
- `crates/kesh-db/src/entities/bank_account.rs` (champ `journal_account_id` ajouté + `#[serde(rename_all = "camelCase")]`)
- `crates/kesh-db/src/repositories/bank_accounts.rs` (4 SELECT patches + nouvelle fn `set_journal_account_id_for_company` + tests inline)

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/bank_accounts.rs` *(nouveau, 2 handlers)*
- `crates/kesh-api/src/routes/mod.rs` (ajout `pub mod bank_accounts`)
- `crates/kesh-api/src/lib.rs` (mounting routes)
- `crates/kesh-api/src/errors.rs` (3 variants : `BankAccountNotFound` réutilisé/créé, `AccountNotFound`, `InvalidAccountType`)
- `crates/kesh-api/tests/bank_accounts_e2e.rs` *(nouveau, ≥ 5 tests)*

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (5 nouvelles clés `bank-accounts-*` × 4 locales)

**Frontend** :
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` *(nouveau)*
- `frontend/src/lib/features/bank-accounts/BankAccountList.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.test.ts` *(nouveau, Vitest)*
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.test.ts` *(nouveau, Vitest)*
- `frontend/src/routes/(app)/bank-accounts/+page.svelte` (extension)
- `frontend/tests/e2e/bank-account-journal-link.spec.ts` *(nouveau, Playwright)*

### Standards de test

- **Intégration `kesh-db`** : `#[sqlx::test]`. ≥ 5 tests T2.4.
- **E2E HTTP `kesh-api`** : helper `spawn_app(pool)` (pattern 8-1b/8-2/8-3/8-4). ≥ 5 nouveaux tests T3.4.
- **Vitest frontend** : `npm run test:unit -- bank-accounts`. ≥ 2-3 tests T4.5.
- **Playwright** : `frontend/tests/e2e/bank-account-journal-link.spec.ts`. ≥ 1 actif + 1 a11y.

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
npm run lint-i18n-ownership
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- bank-account-journal-link.spec.ts
```

### Limitations connues v0.1 (8-5a-zero)

| # | Limitation | Justification |
|---|---|---|
| L60 | Pas de FK DB-level `bank_accounts.journal_account_id REFERENCES accounts(id)` | Cohérence company impossible à garantir sans CHECK trigger MariaDB DELIMITER (non supporté ici). Invariant handler-side suffit v0.1. v0.2 si MariaDB 10.5+ ou refactor schema-level. |
| L61 | Pas de validation classe stricte côté backend (juste `account_type IN (Asset, Liability)`) | L'entité `Account` n'a pas de champ `class` direct. Le filtre `number.startsWith('1')` est UX frontend uniquement. Risque résiduel : un user motivé peut PATCH avec un compte 2100 (passif) qui n'est pas un compte bancaire stricto sensu. Acceptable v0.1, traçable en CR si remontée. |
| L62 | Pas de DELETE `bank_account` ni de POST hors onboarding | v0.1 : un user a 1 bank_account principal géré par onboarding. Multi-bank_account et suppression reportés v0.2. |
| L63 | Pas de configuration multi-bank_account autour de FR45/FR48 v0.1 | Si un user a 2 bank_accounts, il **doit** configurer `journal_account_id` sur les **deux** pour utiliser la réconciliation manuelle/split sur les deux. Pas de fallback automatique. |

### Risques et points d'attention pour le dev agent

1. **Migration sqlx test_fixtures** : la nouvelle column ne change pas `TABLES_TO_TRUNCATE` (pas de nouvelle table) MAIS le `truncate_all_inventory_matches_schema` doit confirmer que `bank_accounts` est toujours dans la liste avec la nouvelle column. Lancer `cargo test -p kesh-db --lib test_fixtures` MariaDB up local avant push (lesson 8-1b hotfix `8046f04`).

2. **Patches SELECT SQL bank_accounts.rs** : 4 occurrences à patcher (cf. T2.2). Oublier une seule cassera la déserialisation `BankAccount` partout. Vérifier avec un grep `SELECT id, company_id, bank_name` dans `repositories/bank_accounts.rs`.

3. **`upsert_primary` ne touche pas `journal_account_id`** : la branche `Some(account)` UPDATE ne mentionne PAS `journal_account_id` (préservation), la branche `None` INSERT ne mentionne PAS `journal_account_id` (laisse NULL par défaut DB). Le cas `journal_account_id` est mutable uniquement via `set_journal_account_id_for_company`, pas via re-onboarding.

4. **Path-dépendance 8-5a-base/bis** : ce 8-5a-zero **doit être stable** (review-closed avec 0 findings > LOW + merged main) avant de démarrer 8-5a-base. La signature de `set_journal_account_id_for_company` peut évoluer après merge si CR explicite, mais les contracts JSON `journalAccountId` (camelCase, body PATCH, audit log shape) sont **stables** (8-5a-base et 8-5a-bis liront `bank_account.journal_account_id` directement, pas via un body field).

5. **Frontend `AccountAutocomplete.svelte`** : si le composant existe déjà dans `features/journal-entries/` (cohérent avec spec 8-5a unifiée T5.3), le réutiliser. Sinon, créer un composant minimaliste (pas un mega-autocomplete v0.1).

### Références

- [`8-5a-reconciliation-manuelle-split.md`](8-5a-reconciliation-manuelle-split.md) — spec d'origine `archived-split-bis` (référence des décisions de conception détaillées + finding F2'' Pass 3 Opus).
- [`8-5a-base-manual-match.md`](8-5a-base-manual-match.md) — sous-story FR45 manual match consommatrice de `bank_account.journal_account_id`.
- [`8-5a-bis-split-breaking-accept.md`](8-5a-bis-split-breaking-accept.md) — sous-story FR48 split + breaking POST /accept Q2.
- [Story 7-3 KF-004](../../crates/kesh-db/src/repositories/bank_accounts.rs:109) — pattern `is_no_op_change` court-circuit.
- [Story 8-1b T6.3](../../crates/kesh-db/src/repositories/bank_accounts.rs:74) — `find_by_id_for_company` cross-tenant safety.
- [Story 5-2](../../crates/kesh-db/src/repositories/journal_entries.rs) — pattern `default_*_account_id` sur entités (cohérence UX).
- [`epic-8.md`](../planning-artifacts/epic-8.md) — contexte FR45-48.
- [`prd.md`](../planning-artifacts/prd.md) §FR45-48 lignes 439-442.

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
| **2026-05-07** | Spec créée par re-split mécanique de 8-5a unifiée (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7). 8-5a-zero = foundation pure : ALTER TABLE `bank_accounts.journal_account_id` + repo + route PATCH + UI configuration. Aucune feature réconciliation. Path-dépendance 8-5a-base et 8-5a-bis sur `bank_account.journal_account_id` documentée. 8 ACs (#75-#82). Tasks T1-T6. Status `8-5a-zero-bank-account-journal-link: ready-for-dev`. Élimine la dette F2'' Pass 3 Opus (anti-pattern UX `bankLedgerAccountId` dans body POST /manual et /split). | Claude (Opus 4.7 re-split workflow) |
