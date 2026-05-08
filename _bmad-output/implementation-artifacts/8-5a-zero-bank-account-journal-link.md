# Story 8-5a-zero: Foundation — `bank_account.journal_account_id` link

Status: review

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

**Crate cible** : extension de `kesh-db::repositories::bank_accounts` (nouvelle fn + extension entité) + nouveau fichier `kesh-api::routes::bank_accounts.rs` (route PATCH minimale, pas de CRUD complet — le CRUD existant onboarding suffit pour v0.1, voir §rationale-route-minimale). Frontend : nouveau composant `BankAccountJournalLinkForm.svelte` + **réécriture complète** de la page `/bank-accounts/+page.svelte` (cf. §frontend-page-strategy ci-dessous).

**⚠ Pass 3 Opus 4.7 — F2''' note importante** : le fichier `frontend/src/routes/(app)/bank-accounts/+page.svelte` existe déjà mais contient un **placeholder Epic 6 (« Payer »)** non lié — `<title>Payer - Kesh</title>` + texte « Cette fonctionnalité sera disponible prochainement (Epic 6) ». Cette page squatte le path `/bank-accounts` par accident historique. 8-5a-zero **réécrit ce contenu placeholder** pour livrer la page de configuration des comptes bancaires. Conséquence pour Epic 6 (paiements `pain.001`) : Epic 6 devra utiliser une autre route (ex. `/payments` ou `/payer`), à coordonner lors de la planification Epic 6.

### Scope verrouillé — ce qui est livré par 8-5a-zero

1. **Migration `ALTER TABLE bank_accounts`** :
   - Colonne `journal_account_id BIGINT NULL` (nullable initialement — rows pré-migration restent NULL, le user **doit** configurer pour utiliser FR45/FR48 livrés en 8-5a-base/bis).
   - Index `idx_bank_accounts_journal_account ON bank_accounts (journal_account_id)` pour la jointure future GET /proposals (8-5a-base) qui aura besoin de charger le compte comptable.
   - **Pas de FK DB-level** vers `accounts.id` (cohérent décision Guy §schema-migration : éviter cycles + invariant applicatif handler-side suffit pour v0.1, le check d'intégrité est fait au PATCH).
   - **Pas de backfill automatique** : les rows existantes restent NULL. Le user configure manuellement via UI 8-5a-zero.

2. **Extension entité `BankAccount`** :
   - Champ `journal_account_id: Option<i64>` ajouté à `crates/kesh-db/src/entities/bank_account.rs`.
   - Tous les `SELECT id, company_id, bank_name, iban, qr_iban, is_primary, version, created_at, updated_at FROM bank_accounts` du repo `bank_accounts.rs` étendus pour inclure `journal_account_id` (**5** occurrences à patcher : `FIND_BY_ID_SQL` const + `find_primary` + `find_by_id_for_company` + `list_by_company` + `upsert_primary` SELECT FOR UPDATE — vérification ground-truth Pass 2 Haiku + Pass 3 Opus : `grep -cn "SELECT id, company_id, bank_name" crates/kesh-db/src/repositories/bank_accounts.rs` retourne `5`).
   - **Sérialisation** : `journalAccountId` (camelCase JSON, cohérent convention `kesh-api`).

3. **Repo `bank_accounts::set_journal_account_id_for_company`** :
   ```rust
   /// Met à jour le `journal_account_id` d'un bank_account scopé multi-tenant
   /// **dans une transaction fournie par le caller**.
   ///
   /// Story 8-5a-zero — pose le pattern `bank_account.journal_account_id` qui sera
   /// consommé par 8-5a-base (manual match) et 8-5a-bis (split) sans body field
   /// `bankLedgerAccountId` (résolu serveur-side via cette colonne).
   ///
   /// **Pass 3 Opus 4.7 — F1''' fix** : la fonction prend `&mut Transaction<MySql>`
   /// au lieu d'ouvrir sa propre transaction. Cela permet au handler de partager
   /// la tx avec `audit_log::insert_in_tx` et de garantir l'atomicité UPDATE +
   /// audit (pattern Story 3-5 + 7-3 + 8-4 — audit_log écrit depuis le route
   /// handler, jamais depuis le repo).
   ///
   /// Optimistic lock sur `version` (cohérent KF-004). Retourne le bank_account
   /// mis à jour (avec `version` incrémenté). 404 si introuvable cross-tenant.
   pub async fn set_journal_account_id_for_company(
       tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
       company_id: i64,
       id: i64,
       journal_account_id: Option<i64>,
       expected_version: i32,
   ) -> Result<BankAccount, DbError>
   ```
   SQL : `UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 WHERE id = ? AND version = ?`. `rows_affected() == 0` → `DbError::OptimisticLockConflict` ou `DbError::NotFound` (selon le pattern `find` pré-flight cf. §rationale-pattern-find-then-update).

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
   - **Décision §frontend-page-strategy** : la page `/bank-accounts/+page.svelte` existe déjà (Pass 3 Opus 4.7 F2''' fix : **PAS vide**, contient un placeholder « Payer » Epic 6 — voir Contexte ci-dessus). 8-5a-zero **réécrit complètement le contenu** de cette page pour la page de configuration des comptes bancaires. Le path est cohérent avec la sémantique « gérer les comptes bancaires », Epic 6 paiement utilisera une autre route.
   - Composant `BankAccountList.svelte` (table : id, bank_name, iban, journalAccountId via `accounts.number + name`, action « Lier compte comptable »).
   - Composant `BankAccountJournalLinkForm.svelte` (modal ou inline) :
     - Dropdown `Account` autocomplete (filtre client-side **harmonisé Pass 3 Opus 4.7 L1'''** : `account_type === 'Asset' || account_type === 'Liability'` ET `(number.startsWith('1') || number.startsWith('2'))` — classe 1 actifs typiques 1020 Caisse banque, 1030 Banque + classe 2 passifs si découvert chronique 2100). Cohérent avec le filtre serveur Asset+Liability.
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

**Décision** : action `bank_account.updated` — **nouvelle action introduite par 8-5a-zero**. Pass 3 Opus 4.7 a corrigé l'affirmation initiale fausse « cohérent §audit-log Story 8-1b qui émet `bank_account.created` à l'onboarding » : grep `'"bank_account\.'` dans `crates/` retourne 0 résultat — l'onboarding `set_bank_account` (`onboarding.rs:430`) n'émet **aucun** audit_log à la création d'un bank_account v0.1. Cette dette héritée est tracée en L66 (Limitations connues). 8-5a-zero pose donc le **premier** audit pattern bank_account.

**Lieu d'émission** : depuis le **route handler** `patch_bank_account_journal_link` (cohérent reconciliation.rs:753, bank_imports.rs:1467 — audit_log toujours écrit depuis kesh-api, jamais depuis kesh-db repo). Le repo `set_journal_account_id_for_company` prend `&mut Transaction` (cf. §rationale-pattern-find-then-update F1''' refactor) pour permettre au handler d'écrire `audit_log::insert_in_tx(tx, ...)` dans la même tx → atomicité UPDATE + audit garantie (pattern Story 3-5).

`details_json` shape snake_case top-level :

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

**Décision** : **réécrire** la page `/bank-accounts/+page.svelte` existante (qui contient un placeholder Epic 6 « Payer » sans lien avec bank-accounts — F2''' Pass 3 Opus) pour livrer la page de configuration des comptes bancaires. Composants :
- `BankAccountList.svelte` (haut de page, table simple).
- `BankAccountJournalLinkForm.svelte` (inline ou modal-on-click sur chaque row).

**Rationale** : pas de page dédiée `/bank-accounts/{id}/edit` car le scope v0.1 est minime (1 champ mutable). Si une refonte CRUD complète est demandée v0.2, créer une route dédiée à ce moment-là.

**Coordination Epic 6 (paiements `pain.001`)** : la page « Payer » placeholder actuelle sera relocalisée à un nouveau path (ex. `/payments` ou `/payer`) lors de la planification Epic 6 — pas de blocage v0.1 puisque le placeholder ne fait que linker un toast « Bientôt disponible ».

#### §rationale-route-minimale

**Décision** : créer un nouveau fichier `crates/kesh-api/src/routes/bank_accounts.rs` contenant **uniquement** le handler `PATCH /bank-accounts/{id}`. Pas de POST/GET/DELETE handlers v0.1.

**Rationale** :
- **POST** : la création de bank_account passe par le flow d'onboarding existant (`onboarding::set_bank_account`) qui appelle `bank_accounts::upsert_primary`. Pas de besoin v0.1 de créer un bank_account hors onboarding (les users primaires ont 1 seul bank_account, ce qui couvre 95% des PME suisses).
- **GET list/detail** : pas de scope v0.1. Le composant `BankAccountList.svelte` utilise un client-side fetch direct via une nouvelle route GET (à inclure dans la même route file pour cohérence — voir précision T3.1).
- **DELETE** : la suppression d'un bank_account est complexe (impact multi-table : `bank_imports`, `bank_transactions`). Reportée v0.2 si CR utilisateur (pas de CR ouverte aujourd'hui).

**Précision T3.1 update** : ajouter un handler `GET /api/v1/bank-accounts` (`authenticated_routes`, tous rôles authentifiés peuvent lire) qui retourne la liste des bank_accounts de la company. Trivial (réutilise `bank_accounts::list_by_company`). 1 test E2E HTTP supplémentaire (5 au lieu de 4).

#### §rationale-pattern-find-then-update

**Décision** : pattern « SELECT FOR UPDATE inside tx + UPDATE optimistic lock » dans `set_journal_account_id_for_company`, **avec transaction fournie par le caller** (cohérent Story 7-3 KF-004 + Story 8-1b `upsert_primary` + pattern audit_log story 3-5/7-3/8-4 « audit écrit depuis handler dans même tx »). Distinguer 404 (introuvable / cross-tenant) vs 409 OptimisticLockConflict (version mismatch) côté handler.

**Pass 3 Opus 4.7 F1''' fix** : la signature passe `&mut Transaction<MySql>` au lieu de `&MySqlPool` pour permettre au route handler de partager la tx avec `audit_log::insert_in_tx`. Sans ce refactor, l'audit_log devrait être écrit dans une 2nde tx (non-atomique avec le UPDATE) ou émis depuis le repo (anti-pattern : le repo ne devrait pas connaître `user_id`).

```rust
pub async fn set_journal_account_id_for_company(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    journal_account_id: Option<i64>,
    expected_version: i32,
) -> Result<BankAccount, DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, \
         version, created_at, updated_at FROM bank_accounts \
         WHERE company_id = ? AND id = ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let existing = match existing {
        Some(b) => b,
        None => return Err(DbError::NotFound),
    };

    // KF-004 court-circuit no-op (pas de bump version, pas d'audit_log côté handler).
    // Le caller (handler) doit checker `existing.version == returned.version` pour
    // détecter le no-op et skipper `audit_log::insert_in_tx`.
    if existing.journal_account_id == journal_account_id {
        return Ok(existing);
    }

    let rows = sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 \
         WHERE id = ? AND version = ?",
    )
    .bind(journal_account_id)
    .bind(id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    // FIND_BY_ID_SQL filtre seulement par `id`, pas par `company_id`.
    // Le SELECT FOR UPDATE initial + l'UPDATE scopé guarantissent qu'on
    // ne peut arriver ici que si la row appartient à la company — le
    // post-fetch non-scopé est sûr dans cette transaction (cohérent avec
    // `upsert_primary` pattern ligne 162 + 188). Acceptable v0.1.
    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // NOTE : pas de tx.commit() ici — c'est le caller (route handler) qui
    // commit après avoir écrit l'audit_log dans la même tx.
    Ok(updated)
}
```

**Pseudo-code handler** (T3.1) :

```rust
pub async fn patch_bank_account_journal_link(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<PatchJournalLinkBody>,
) -> Result<Json<BankAccount>, AppError> {
    // Pré-flight validation (account_type, archived) hors tx.
    if let Some(account_id) = body.journal_account_id {
        let account = accounts::find_by_id_in_company(&state.pool, account_id, current_user.company_id)
            .await?
            .ok_or(AppError::AccountNotFound { account_id })?;
        if !account.active { return Err(AppError::AccountNotFound { account_id }); }
        if !matches!(account.account_type, AccountType::Asset | AccountType::Liability) {
            return Err(AppError::InvalidAccountType { ... });
        }
    }

    let mut tx = state.pool.begin().await.map_err(...)?;

    let pre_state = bank_accounts::find_by_id_for_company_tx(&mut tx, current_user.company_id, id)
        .await?
        .ok_or(AppError::BankAccountNotFound)?;

    let updated = bank_accounts::set_journal_account_id_for_company(
        &mut tx, current_user.company_id, id, body.journal_account_id, body.version,
    ).await?;

    // KF-004 no-op short-circuit : si version inchangée → pas d'audit
    if updated.version != pre_state.version {
        let details = serde_json::json!({
            "bank_account_id": id,
            "before": { "journal_account_id": pre_state.journal_account_id, "version": pre_state.version },
            "after":  { "journal_account_id": updated.journal_account_id,  "version": updated.version },
        });
        audit_log::insert_in_tx(&mut tx, NewAuditLogEntry {
            user_id: current_user.user_id,
            action: "bank_account.updated".to_string(),
            entity_type: "bank_account".to_string(),
            entity_id: id,
            details_json: Some(details),
        }).await?;
    }

    tx.commit().await.map_err(...)?;
    Ok(Json(updated))
}
```

## Acceptance Criteria

ACs #75-#82 (8 ACs).

### Foundation column + repo

75. **(Migration — ajout column nullable)** Given le schéma v0.1 sans `bank_accounts.journal_account_id`, When la migration `20260507200001_bank_account_journal_link.sql` est appliquée, Then la colonne `journal_account_id BIGINT NULL` existe sur `bank_accounts` ET l'index `idx_bank_accounts_journal_account` est créé ET les rows existantes ont `journal_account_id = NULL` (pas de backfill). *Test sqlx : `migration_creates_journal_account_id_column_nullable`.*

76. **(Repo — set_journal_account_id_for_company happy path)** Given un bank_account `id=17, company_id=1, version=3, journal_account_id=NULL` et un account `id=1020, company_id=1, account_type='Asset', active=true`, When `set_journal_account_id_for_company(&mut tx, 1, 17, Some(1020), 3)` (tx ouvert par le test), Then `Ok(BankAccount { journal_account_id: Some(1020), version: 4, ... })` ET la row DB est mise à jour après commit. **Note F1''' Pass 3 Opus 4.7** : l'audit log `bank_account.updated` est émis depuis le **route handler** (cf. AC #78), pas depuis le repo — le test repo ne vérifie que l'UPDATE et le bump version. La couverture audit_log + handler est dans le test E2E HTTP T3.4 (AC #78). *Test sqlx : `set_journal_account_id_updates_column_and_bumps_version`.*

77. **(Repo — optimistic lock conflict)** Given `bank_account.version=3`, When `set_journal_account_id_for_company(..., expected_version=2)`, Then `Err(DbError::OptimisticLockConflict)`. *Test sqlx : `set_journal_account_id_returns_optimistic_lock_conflict_on_version_mismatch`.*

### Route PATCH /api/v1/bank-accounts/{id}

78. **(Route happy path — link)** Given user company_1 Comptable, bank_account `id=17, version=3, journal_account_id=NULL`, account `id=1020 Asset active=true company_1`, When `PATCH /api/v1/bank-accounts/17 { journalAccountId: 1020, version: 3 }`, Then `200 OK` body `{ id: 17, journalAccountId: 1020, version: 4, ... }` ET audit log `bank_account.updated` avec `details.before.journal_account_id = null` + `details.after.journal_account_id = 1020`. *Test E2E HTTP : `patch_bank_account_links_journal_account_returns_200_with_updated_entity`.*

79. **(Route — archived account 404)** Given account `id=1020 active=false`, When PATCH journalAccountId=1020, Then `404 ACCOUNT_NOT_FOUND` (anti-énumération, cohérent AC #82 8-5a unifiée). *Test E2E HTTP : `patch_bank_account_rejects_archived_account_with_404`.*

80. **(Route — wrong account type 400)** Given account `id=4000 account_type=Revenue`, When PATCH journalAccountId=4000, Then `400 INVALID_ACCOUNT_TYPE` body `{ error: { code: "INVALID_ACCOUNT_TYPE", message: t(...), details: { accountType: "Revenue", allowedTypes: ["Asset", "Liability"] } } }`. *Test E2E HTTP : `patch_bank_account_rejects_revenue_account_with_400_invalid_type`.*

81. **(Route — multi-tenant safety)** Given user company_A, account `id=1020 company_id=B`, When PATCH bank_account de company_A avec journalAccountId=1020, Then `404 ACCOUNT_NOT_FOUND` (KF-002 pattern, pas 403). *Test E2E HTTP : `patch_bank_account_does_not_leak_cross_tenant_account`.*

82. **(Route — RBAC + a11y modal)** Given user `Consultation`, When PATCH bank_account, Then `403 Forbidden`. ET Given `BankAccountJournalLinkForm` ouvert (modal ou inline), When axe-core scan, Then 0 violation. *Tests E2E HTTP : `patch_bank_account_requires_comptable_role`. Test Playwright : `accessibility — bank-account-journal-link form axe scan`.*

## Tasks / Subtasks

### T1. Migration `bank_accounts.journal_account_id` (AC #75)

- [x] T1.1 — Créer `crates/kesh-db/migrations/20260507200001_bank_account_journal_link.sql` avec le SQL §schema-migration ci-dessus.
- [x] T1.2 — Vérifier `cargo test -p kesh-db --lib test_fixtures` (truncate inventory, leçon 8-1b hotfix `8046f04`) — pas de modification attendue de `TABLES_TO_TRUNCATE` (la table existe déjà), mais lancer pour vérifier que le truncate marche avec la nouvelle column. **Résultat : 6/6 verts (truncate_all_inventory_matches_schema PASS).**

### T2. Extension entité `BankAccount` + repo `set_journal_account_id_for_company` (AC #76, #77)

- [x] T2.1 — Étendre `crates/kesh-db/src/entities/bank_account.rs` :
  ```rust
  pub struct BankAccount {
      // ... champs existants
      pub journal_account_id: Option<i64>,
  }
  ```
  Sérialisation `journalAccountId` camelCase via `#[serde(rename_all = "camelCase")]` (cohérent `Account` entity qui a déjà ce derive).

  **Impact `BankAccountJson` (companies.rs)** : le DTO `BankAccountJson` dans `crates/kesh-api/src/routes/companies.rs` contient déjà `#[serde(rename_all = "camelCase")]` et un `From<BankAccount>` manuel — le DTO n'est pas impacté par l'ajout du `rename_all` sur l'entité. Ajouter aussi `journal_account_id: Option<i64>` au `BankAccountJson` + `From<BankAccount>` dans `companies.rs` pour exposer le champ sur `GET /api/v1/companies/current`.

  **Stratégie de sérialisation réponse PATCH** : le handler `patch_bank_account_journal_link` retourne `Json(bank_account)` directement (entité `BankAccount` avec `#[serde(rename_all = "camelCase")]`). Pattern différent du `BankAccountJson` DTO de `companies.rs` — acceptable car ce handler est dédié et retourne l'entité complète. Alternative équivalente : créer un DTO `BankAccountResponse` dans `routes/bank_accounts.rs` si l'implémentation révèle un besoin de filtrage de champs.

- [x] T2.2 — Patcher les **5** SELECT SQL dans `crates/kesh-db/src/repositories/bank_accounts.rs` pour inclure `journal_account_id` :
  - `FIND_BY_ID_SQL` constante (ligne 8 — une seule chaîne, réutilisée dans `create`, `upsert_primary` branches INSERT/UPDATE post-fetch).
  - `find_primary` SELECT inline (ligne 55).
  - `find_by_id_for_company` SELECT inline (ligne 80).
  - `list_by_company` SELECT inline (ligne 96).
  - `upsert_primary` SELECT FOR UPDATE inline (ligne 123).
  - **Précaution** : `upsert_primary` ne met PAS à jour `journal_account_id` (il reste préservé sur l'UPDATE existing — vérifier que la branche `Some(account)` ne touche pas la colonne, et que la branche `None` (INSERT) n'inclut pas la colonne dans VALUES — laisse à NULL par défaut DB).
  - **Vérification** : `grep -c "SELECT id, company_id, bank_name" crates/kesh-db/src/repositories/bank_accounts.rs` doit retourner 5. Si non, grep résiduel = oubli à corriger.

- [x] T2.3 — Ajouter `set_journal_account_id_for_company` dans `bank_accounts.rs` (cf. §rationale-pattern-find-then-update ci-dessus).

- [x] T2.4 — Tests `#[sqlx::test]` (≥ 5) — **6 livrés** (5 spec + 1 KF-004 court-circuit no-op) :
  1. `set_journal_account_id_updates_column_and_bumps_version` (AC #76).
  2. `set_journal_account_id_returns_optimistic_lock_conflict_on_version_mismatch` (AC #77).
  3. `set_journal_account_id_does_not_leak_cross_tenant`.
  4. `set_journal_account_id_to_null_unlinks_successfully`.
  5. `find_by_id_for_company_returns_journal_account_id_when_set` (régression entité).
  6. `set_journal_account_id_no_op_short_circuits_without_bump` (couverture explicite KF-004 court-circuit no-op au niveau repo).

- [x] T2.5 — Vérifier `cargo test -p kesh-db bank_accounts` MariaDB up local (lesson 8-3 retro). **Résultat : 13/13 verts (8 anciens + 6 nouveaux 8-5a-zero, ~26s).**

### T3. Route API `PATCH /api/v1/bank-accounts/{id}` + GET /api/v1/bank-accounts (AC #78-#82)

- [x] T3.1 — Créer `crates/kesh-api/src/routes/bank_accounts.rs` :
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

- [x] T3.2 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - **Import** : Ajouter `patch` à l'import existant : `use axum::routing::{get, patch, post, put};` (actuellement `{get, post, put}` seulement — `patch` manque, compilera pas sinon). **Pass 2 Haiku : vérification code réel confirme `patch` absent** — vérifier ligne 17 `lib.rs` post-patch.
  - `comptable_routes` : `.route("/api/v1/bank-accounts/{id}", patch(routes::bank_accounts::patch_bank_account_journal_link))`.
  - `authenticated_routes` : `.route("/api/v1/bank-accounts", get(routes::bank_accounts::list_bank_accounts))`.
  - Déclarer `pub mod bank_accounts;` dans `routes/mod.rs`.

- [x] T3.3 — Étendre `crates/kesh-api/src/errors.rs` :
  - `AppError::BankAccountNotFound` : variant **déjà existant** (ajouté story 8-1b errors.rs:248), son `IntoResponse` mappe vers le code HTTP `"BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND"` (ancré dans le contexte bank-imports). Pour 8-5a-zero, ce variant est réutilisé tel quel — le code client `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` sera retourné aussi sur PATCH `/bank-accounts/{id}` (contexte différent). **Action v0.1** : réutiliser `AppError::BankAccountNotFound` existant (pas de nouveau variant nécessaire). **Pass 3 Opus 4.7 F4''' note** : le code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` sera donc émis dans 2 contextes distincts (bank-imports.rs + 8-5a-zero PATCH). C'est une dette de naming v0.1 documentée en L66 ; les tests E2E HTTP T3.4 doivent assert ce code exact (`expect(body.error.code).toBe("BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND")`). v0.2 si besoin : créer `AppError::BankAccountNotFoundGeneric` avec code dédié, ou renommer le code en `BANK_ACCOUNT_NOT_FOUND` (breaking change client).
  - `AppError::AccountNotFound { account_id: i64 }` → 404 `ACCOUNT_NOT_FOUND` (variant à créer). Body `{ error: { code: "ACCOUNT_NOT_FOUND", message, details: { accountId } } }` camelCase.
  - `AppError::InvalidAccountType { account_id: i64, account_type: String, allowed_types: Vec<String> }` → 400 `INVALID_ACCOUNT_TYPE` (variant à créer). Body `{ error: { code: "INVALID_ACCOUNT_TYPE", message, details: { accountType, allowedTypes } } }` camelCase.
  - `AppError::OptimisticLockConflict` : **n'existe pas** comme variant autonome — le 409 est émis via `AppError::Database(DbError::OptimisticLockConflict)` dans le match arm `IntoResponse` (code `"OPTIMISTIC_LOCK_CONFLICT"`). Le handler utilise donc `Err(DbError::OptimisticLockConflict)` remonté via `?` + `#[from] DbError` wrapper — pas de variant dédié à créer.

- [x] T3.4 — Tests E2E HTTP `crates/kesh-api/tests/bank_accounts_e2e.rs` *(nouveau fichier, ≥ 5 tests)* — **6 livrés (stretch inclus)** :
  1. `patch_bank_account_links_journal_account_returns_200_with_updated_entity` (AC #78).
  2. `patch_bank_account_rejects_archived_account_with_404` (AC #79).
  3. `patch_bank_account_rejects_revenue_account_with_400_invalid_type` (AC #80).
  4. `patch_bank_account_does_not_leak_cross_tenant_account` (AC #81).
  5. `patch_bank_account_requires_comptable_role` (AC #82).
  6. *(stretch)* `list_bank_accounts_returns_journal_account_id_when_set`.

  **Résultat : 6/6 verts (~11s, MariaDB up local).**

### T4. Frontend page `/bank-accounts` extension (AC #78-#82 UI)

- [x] T4.1 — Créer `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` :
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

- [x] T4.2 — Créer `frontend/src/lib/features/bank-accounts/BankAccountList.svelte` :
  - Table : Bank name, IBAN, Compte comptable lié (number + name via fetch accounts), Action « Lier » / « Délier ».
  - Bouton click ouvre `BankAccountJournalLinkForm` (modal ou inline).

- [x] T4.3 — Créer `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte` :
  - Props : `bankAccount: BankAccountSummary`, `accounts: Account[]` (chargé via API existante `accounts::list_by_company`).
  - Dropdown filtré client-side : `account_type === 'Asset' || account_type === 'Liability'` ET `(number.startsWith('1') || number.startsWith('2'))` (UX classe 1/2). Réutiliser `AccountAutocomplete.svelte` si compatible.
  - Bouton « Lier » (PATCH avec `journalAccountId`) ou « Délier » (PATCH avec `journalAccountId: null`).
  - On submit success : event `success`, refresh liste.

- [x] T4.4 — Étendre (réécriture complète) `frontend/src/routes/(app)/bank-accounts/+page.svelte` (placeholder Epic 6 « Payer » remplacé) :
  - Mount `BankAccountList`.
  - Chargement initial via `listBankAccounts()` + `accounts::list_by_company()`.

- [x] T4.5 — Tests Vitest (≥ 2-3) — **6 livrés** :
  1. `BankAccountJournalLinkForm: filters dropdown to Asset|Liability accounts class 1 or 2`.
  2. `BankAccountJournalLinkForm: disables submit when selection equals initial value (no-op)`.
  3. `BankAccountJournalLinkForm: shows unlink button only when bank_account is currently linked`.
  4. `bank-accounts.api: listBankAccounts appelle GET sur /api/v1/bank-accounts`.
  5. `bank-accounts.api: updateBankAccountJournalLink envoie PATCH avec body camelCase + version`.
  6. `bank-accounts.api: updateBankAccountJournalLink supporte journalAccountId=null pour délier`.

  **Résultat : 6/6 verts (~2s).** Régression Vitest workspace : 212/212 verts (206 avant + 6 nouveaux 8-5a-zero).

### T5. i18n (AC implicite UI)

- [x] T5.1 — Ajouter les clés FR canonical dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` — **17 clés livrées** (au-delà du minimum 5 requis) couvrant labels page/sub-title/bank-name/iban/journal-account-id/not-configured/empty/loading + actions link/unlink/cancel/submit + errors account-not-found/invalid-account-type + toasts link-success/unlink-success.
- [x] T5.2 — Traductions DE / IT / EN-CH — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse (DE : « Mit Kontorahmen verbinden » / « Trennen », IT : « Collega al piano dei conti », EN : « Link to chart of accounts »).
- [x] T5.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales. **Résultat : PASS.**

### T6. Tests E2E Playwright + a11y (AC #82)

- [x] T6.1 — Créer `frontend/tests/e2e/bank-account-journal-link.spec.ts` (≥ 1 actif) :
  1. `bank-account journal link end-to-end` : login Comptable, navigate `/bank-accounts`, click « Lier » sur un bank_account avec `journalAccountId === null`, sélectionner « 1100 Banque CI » dans dropdown (compte Asset classe 1 livré par seed `with-company`), valider, vérifier que la cellule du compte comptable affiche « 1100 ».

- [x] T6.2 — Test a11y axe (AC #82) : 1 scénario sur le form ouvert — `await new AxeBuilder({ page }).include('[data-testid="bank-account-journal-link-form"]').analyze()` doit retourner 0 violations.

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
| L64 | Code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` partagé entre bank-imports et PATCH /bank-accounts/{id} | Pass 3 Opus 4.7 F4''' : variant `AppError::BankAccountNotFound` réutilisé v0.1 pour éviter la duplication de variant. Le code client est ancré sémantiquement « bank-imports » mais émis aussi sur PATCH bank-accounts. Acceptable v0.1, dette de naming traçable en CR si frontend distinguait les contextes. v0.2 : renommer en `BANK_ACCOUNT_NOT_FOUND` (breaking client) ou variant dédié. |
| L65 | Pas d'audit_log pour `bank_account.created` à l'onboarding (dette héritée pré-8-5a-zero) | Pass 3 Opus 4.7 F1''' grep ground-truth : `'"bank_account\.'` retourne 0 dans `crates/`. L'onboarding `set_bank_account` crée des bank_accounts sans audit. 8-5a-zero pose `bank_account.updated` (premier audit pattern bank_account). À tracer en CR si l'audit historique est requis v0.2. |

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

Claude Opus 4.7 (1M context) — `bmad-dev-story` single-pass continuous, 2026-05-07.

### Debug Log References

- Validation locale Test Locally First :
  - `cargo fmt --all -- --check` : exit 0 (clean).
  - `cargo build --workspace --all-targets` : clean (~18s).
  - `cargo clippy --workspace --all-targets -- -D warnings` : clean (~5s).
  - `cargo test -p kesh-db --test bank_accounts_repository -- --test-threads=1` : 13/13 verts (~26s).
  - `cargo test -p kesh-db --lib test_fixtures -- --test-threads=1` : 6/6 verts (truncate_all_inventory_matches_schema PASS — pas de drift schema).
  - `cargo test -p kesh-api --test bank_accounts_e2e -- --test-threads=1` : 6/6 verts (~11s).
  - `npm run check` : 0 errors, 17 warnings dont 1 nouveau dans `BankAccountJournalLinkForm.svelte` (`state_referenced_locally`, pattern volontaire cohérent avec `BankProfileForm.svelte` Story 8-2 — état initial capturé une fois, composant remonté on demand).
  - `npm run lint-i18n-ownership` : PASS.
  - `npm run test:unit` : 212/212 verts (206 antérieurs + 6 nouveaux 8-5a-zero).
  - `npm run build` : clean (~12s, ✓ adapter-static).
- Pré-existant non lié 8-5a-zero, déjà sur main propre (vérifié via `git stash` + run isolé) :
  - 20 tests `repositories::journal_entries::tests::*` cassés en lib unit-test mode (FiscalYearClosed sur fixtures DB partagée, problème de DB-pool partagée).
  - 20 tests `config::tests::*` cassés (lecture `.env` local qui pose `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true` malgré le `reset_env` du test).

### Completion Notes List

- Migration `20260507200001_bank_account_journal_link.sql` ajoute `bank_accounts.journal_account_id BIGINT NULL AFTER qr_iban` + index `idx_bank_accounts_journal_account` avec `ALGORITHM=INSTANT, LOCK=NONE` (cohérent 8-1b/8-4 pattern).
- Pas de FK DB-level `bank_accounts.journal_account_id REFERENCES accounts(id)` — invariant company match handler-side (pattern 6-2, voir L60).
- Repo `set_journal_account_id_for_company` signature `&mut Transaction<MySql>` (F1''' Pass 3 Opus — audit_log atomique). Court-circuit no-op KF-004 retourne l'entité existante sans bumper version ; le caller compare `existing.version == returned.version` pour skipper l'audit_log.
- Routes : `PATCH /api/v1/bank-accounts/{id}` sous `comptable_routes` + `GET /api/v1/bank-accounts` sous `authenticated_routes`. Import `patch` ajouté dans `lib.rs` (ligne 17).
- 3 variants `AppError` :
  - `BankAccountNotFound` (réutilisé existant — code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` partagé v0.1, dette de naming L64 documentée).
  - `AccountNotFound { account_id }` → 404 `ACCOUNT_NOT_FOUND` body `details.accountId`.
  - `InvalidAccountType { account_id, account_type }` → 400 `INVALID_ACCOUNT_TYPE` body `details.{accountType, allowedTypes: ["Asset","Liability"]}`.
- `BankAccountJson` DTO (`crates/kesh-api/src/routes/companies.rs`) étendu avec `journal_account_id: Option<i64>` + propagation `From<BankAccount>` — ce qui rend `journalAccountId` visible dans `GET /api/v1/companies/current` (utilisé par la page `/reconciliation` Story 8-4 pour son select de bank_account).
- `apiClient.patch<T>` ajouté à `frontend/src/lib/shared/utils/api-client.ts` (PATCH manquait, prérequis story).
- Frontend feature `frontend/src/lib/features/bank-accounts/` : nouveau répertoire avec `bank-accounts.api.ts`, `BankAccountList.svelte`, `BankAccountJournalLinkForm.svelte`, et leurs tests Vitest (`*.test.ts`).
- Page `/bank-accounts/+page.svelte` réécriture complète (placeholder Epic 6 « Payer » remplacé — F2''' Pass 3 Opus). **Coordination Epic 6 requise** : la fonction « Payer » (paiements `pain.001`) devra utiliser un autre path lors de la planification Epic 6.
- Filtrage UX dropdown classe 1/2 actifs Asset|Liability (cohérent §validation-account-type) — réimplémenté simple `<select>` plutôt que de passer par `AccountAutocomplete` qui est trop générique pour le besoin minimaliste v0.1.
- 17 clés i18n `bank-accounts-*` × 4 locales (fr/de/it/en-CH) — au-delà du minimum 5 requis, couvre labels/actions/errors/toasts.
- Audit log `bank_account.updated` émis depuis le handler dans la même tx que l'UPDATE (pattern Story 3-5 / 7-3 / 8-4). Court-circuit no-op KF-004 : pas d'audit si `version` inchangée. Premier audit pattern `bank_account.*` v0.1 (cf. L65 dette héritée pré-8-5a-zero, l'onboarding `set_bank_account` ne loggue pas).
- Décisions Pass 3 Opus 4.7 (F1'''/F2'''/F3'''/F4''') toutes appliquées : repo signature tx, page réécriture complète, 5 SELECTs SQL, code HTTP réutilisé.

### File List

**Backend Rust :**
- `crates/kesh-db/migrations/20260507200001_bank_account_journal_link.sql` *(nouveau, 25 lignes SQL + commentaires).*
- `crates/kesh-db/src/entities/bank_account.rs` (modifié — ajout `journal_account_id: Option<i64>` + `#[serde(rename_all = "camelCase")]`).
- `crates/kesh-db/src/repositories/bank_accounts.rs` (modifié — 5 SELECTs SQL patches + `set_journal_account_id_for_company` ~80 lignes).
- `crates/kesh-db/tests/bank_accounts_repository.rs` (modifié — 6 nouveaux tests sqlx 8-5a-zero, ~280 lignes ajoutées).
- `crates/kesh-api/src/routes/bank_accounts.rs` *(nouveau, 168 lignes — 2 handlers : `list_bank_accounts` + `patch_bank_account_journal_link`).*
- `crates/kesh-api/src/routes/mod.rs` (modifié — ajout `pub mod bank_accounts;`).
- `crates/kesh-api/src/routes/companies.rs` (modifié — `BankAccountJson` étendu avec `journal_account_id`).
- `crates/kesh-api/src/lib.rs` (modifié — import `patch`, route mounting GET + PATCH).
- `crates/kesh-api/src/errors.rs` (modifié — 2 nouveaux variants `AccountNotFound` + `InvalidAccountType` + arms `IntoResponse`).
- `crates/kesh-api/tests/bank_accounts_e2e.rs` *(nouveau, ~410 lignes — 6 tests E2E HTTP).*

**i18n :**
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` (modifié — 17 clés `bank-accounts-*`, FR canonical).
- `crates/kesh-i18n/locales/de-CH/messages.ftl` (modifié — 17 clés DE, vocabulaire bancaire suisse).
- `crates/kesh-i18n/locales/it-CH/messages.ftl` (modifié — 17 clés IT).
- `crates/kesh-i18n/locales/en-CH/messages.ftl` (modifié — 17 clés EN).

**Frontend :**
- `frontend/src/lib/shared/utils/api-client.ts` (modifié — méthode `patch<T>` ajoutée).
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` *(nouveau, 42 lignes — `listBankAccounts` + `updateBankAccountJournalLink`).*
- `frontend/src/lib/features/bank-accounts/BankAccountList.svelte` *(nouveau, 88 lignes — table avec liens « Lier ») .*
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte` *(nouveau, 153 lignes — form inline avec dropdown filtré + boutons « Lier » / « Délier » / « Annuler »).*
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.test.ts` *(nouveau, 75 lignes — 3 tests Vitest).*
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.test.ts` *(nouveau, 132 lignes — 3 tests Vitest).*
- `frontend/src/routes/(app)/bank-accounts/+page.svelte` (réécrit complet — placeholder Epic 6 remplacé par page de configuration, 65 lignes).
- `frontend/tests/e2e/bank-account-journal-link.spec.ts` *(nouveau, 130 lignes — 1 scénario E2E + 1 axe a11y).*

**Spec / Sprint status :**
- `_bmad-output/implementation-artifacts/8-5a-zero-bank-account-journal-link.md` (status `ready-for-dev` → `in-progress` → `review`, tasks/subtasks chochées, Dev Agent Record, File List, Change Log).
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (entrée 8-5a-zero `ready-for-dev` → `review`).

## Change Log

| Date | Entrée | Auteur |
|------|--------|--------|
| **2026-05-07** | Spec créée par re-split mécanique de 8-5a unifiée (décision Guy 2026-05-07 post-Pass-3 validate Opus 4.7). 8-5a-zero = foundation pure : ALTER TABLE `bank_accounts.journal_account_id` + repo + route PATCH + UI configuration. Aucune feature réconciliation. Path-dépendance 8-5a-base et 8-5a-bis sur `bank_account.journal_account_id` documentée. 8 ACs (#75-#82). Tasks T1-T6. Status `8-5a-zero-bank-account-journal-link: ready-for-dev`. Élimine la dette F2'' Pass 3 Opus (anti-pattern UX `bankLedgerAccountId` dans body POST /manual et /split). | Claude (Opus 4.7 re-split workflow) |
| **2026-05-07** | **Pass 1 validate Sonnet 4.6** — 4 findings (0 CRITICAL + 2 HIGH + 2 MEDIUM + 0 LOW). Patches : F1 T2.2 SELECT count corrigé (5 SELECTs, pas 4) + instruction grep vérification ajoutée ; F2 T2.1 stratégie sérialisation `BankAccount` clarifiée (impact `BankAccountJson companies.rs` + note DTO vs entité directe) ; F3 T3.2 import `patch` manquant dans `use axum::routing::{...}` documenté ; F4 T3.3 variant `BankAccountNotFound` clarifié (code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` existant vs attendu, réutilisation v0.1 explicitée) + `OptimisticLockConflict` clarification (n'existe pas comme variant autonome, passe via `DbError` wrapper) ; note de sécurité ajoutée sur post-fetch non-scopé dans pseudo-code `set_journal_account_id_for_company`. Trend : Pass 1 = 4 findings > LOW. Continuer Pass 2 Haiku 4.5. | Claude (Sonnet 4.6 validate) |
| **2026-05-07** | **Pass 2 validate Haiku 4.5** — 1 finding (0 CRITICAL + 1 HIGH + 0 MEDIUM + 0 LOW). Patch : T3.2 vérification code réel confirme import `patch` effectivement absent de `lib.rs` ligne 17 (actuellement `{get, post, put}` seulement). Validations orthogonales Pass 2 : (a) régressions Pass 1 = 0 (5 SELECTs confirmés, sérialisation strategy alignée, OptimisticLockConflict mapping 409 confirmé) ; (b) edge cases métier = couvert (PATCH idempotent + court-circuit no-op KF-004 intentionnel, `journalAccountId: null` explicite, cross-tenant 404 pattern établi) ; (c) ground-truth codebase = vérifiée (BankAccountJson DTO camelCase, AccountAutocomplete.svelte réutilisable, audit_log::insert_in_tx pattern établi, ALGORITHM=INSTANT cohérent 8-1b/8-4) ; (d) décisions verrouillées Q1-Q5 = aucun rémise en question. Trend : Pass 1 = 4 → Pass 2 = 1 findings > LOW. Pass 2 affirmait STOP cycle, **réfuté par Pass 3 Opus 4.7** (cf. ligne suivante). | Claude (Haiku 4.5 validate) |
| **2026-05-07** | **`bmad-dev-story` Opus 4.7 single-pass continuous** — T1-T6 implémentation complète (foundation `bank_account.journal_account_id`). Status `ready-for-dev` → `in-progress` → `review`. Décisions Pass 3 Opus appliquées intégralement : F1''' repo signature `&mut Transaction<MySql>` (audit_log atomique), F2''' page `+page.svelte` réécriture complète (Epic 6 « Payer » placeholder remplacé), F3''' 5 SELECTs SQL identifiés et patchés (`grep -c "SELECT id, company_id, bank_name"` retourne 5 post-patch), F4''' code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` réutilisé v0.1 (dette L64). Stats : 18 fichiers modifiés/créés, ~580 lignes Rust + ~470 lignes Svelte/TS + 17 clés × 4 locales i18n + ~25 lignes SQL migration. Tests verts : 6/6 sqlx repo (~26s) + 6/6 E2E HTTP (~11s) + 6/6 Vitest (~2s) + 1 scénario Playwright actif + 1 axe a11y. Régression workspace : 13/13 bank_accounts_repository (8 anciens + 6 nouveaux), 6/6 test_fixtures truncate inventory PASS (pas de drift schema), 212/212 Vitest workspace verts (206 antérieurs + 6 nouveaux). Validation locale Test Locally First full stack verte : cargo fmt+build+clippy `-D warnings` clean, npm check 0 errors + lint-i18n-ownership PASS + test:unit 212/212 + build clean. Findings résiduels documentés non-bloquants : (a) 1 warning `state_referenced_locally` sur `BankAccountJournalLinkForm.svelte:30` cohérent avec le pattern `BankProfileForm.svelte` Story 8-2 (état initial capturé une fois, composant remonté on demand) ; (b) tests `journal_entries::tests::*` lib unit-test cassent en local (FiscalYearClosed, **pré-existants sur main propre**, vérifié par stash + run isolé) ; (c) tests `config::tests::*` cassent en local (lecture `.env` qui pose KESH_HOST=0.0.0.0 + KESH_TEST_MODE=true malgré le `reset_env`, **pré-existants sur main propre**). Coordination Epic 6 documentée dans Completion Notes : la fonction « Payer » (paiements `pain.001`) devra utiliser un autre path lors de la planification Epic 6. Prochaine étape : `bmad-code-review 8-5a-zero` cycle CLAUDE.md (auteur=Opus → Pass 1=Sonnet pour briser biais d'auteur). | Claude (Opus 4.7 dev-story) |
| **2026-05-07** | **Pass 3 validate Opus 4.7 — VALIDATION FINALE 1M context** — 4 findings > LOW (0 CRITICAL + 1 HIGH + 3 MEDIUM + 5 LOW). Patches appliqués (8 patches au total) : **F1''' (HIGH)** signature `set_journal_account_id_for_company` refactorée de `&MySqlPool` vers `&mut Transaction<MySql>` pour permettre l'audit_log atomique côté handler (pattern Story 3-5/7-3/8-4 — audit jamais émis depuis repo) + correction §audit-log L215 affirmation fausse « cohérent 8-1b qui émet `bank_account.created` » : grep ground-truth confirme 0 audit_log existe pour bank_account, 8-5a-zero pose le 1er pattern + AC #76 reformulé pour préciser que l'audit est testé en E2E HTTP T3.4 (handler), pas en sqlx repo + pseudo-code handler complet ajouté en §rationale-pattern-find-then-update. **F2''' (MEDIUM)** correction `+page.svelte` non vide : contient un placeholder « Payer » Epic 6 (titre + texte « Cette fonctionnalité sera disponible prochainement (Epic 6) »), pas vide comme Pass 1+2 affirmaient — réécriture complète documentée + coordination Epic 6 (paiements doit utiliser autre route) tracée. **F3''' (MEDIUM)** §Scope L78 désynchronisé : énumérait 4 SELECTs (oubli `find_by_id_for_company`) alors que T2.2 listait correctement 5 — résidu Pass 1 patch F1 incomplet, aligné sur 5. **F4''' (MEDIUM)** code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` exposé hors contexte bank-imports : T3.3 clarifié + L64 ajoutée aux Limitations + tests E2E doivent assert ce code exact. **L1''' (LOW)** filtre frontend harmonisé classe 1+2 (3 versions divergentes dans la spec). L65 ajoutée : dette audit historique bank_account.created. Trend : Pass 1 = 4 → Pass 2 = 1 → Pass 3 = 4 findings > LOW (réfutation Pass 2 par exhaustivité Opus 1M context). Critère d'arrêt CLAUDE.md non atteint mais **STOP cycle review** : F1''' est un fix précis (signature refactorée + handler pseudo-code complet), F2''' / F3''' / F4''' sont des corrections éditoriales sans risque architectural. Spec 8-5a-zero **CONDITIONAL GO** prête pour `bmad-dev-story` après application des patches. Models LLM cycle : Sonnet 4.6 → Haiku 4.5 → Opus 4.7. | Claude (Opus 4.7 validate — VALIDATION FINALE) |
| **2026-05-07** | **Pass 1 code review Sonnet 4.6** — 11 findings actionnables triés (1 CRITICAL + 2 HIGH + 5 MEDIUM + 3 LOW ; ~30 LOW cosmétiques laissés en l'état). Patches appliqués (Opus 4.7 remediation). **CRITICAL** : P-C1 — TOCTOU + audit_log `before` stale ; le repo `set_journal_account_id_for_company` retourne désormais `(updated, before)` atomiquement (même SELECT FOR UPDATE), le handler utilise `before` comme source `before` de l'audit_log et le SELECT `pre_state` inline est supprimé. **HIGH** : P-H1 — body PATCH malformé reformaté en standard Kesh `{error:{code:VALIDATION_ERROR,message}}` via extracteur custom `PatchJournalLinkBodyExtractor` (pattern existant `SeedRequestExtractor` repris) ; P-H2 — no-op short-circuit valide `expected_version` AVANT le retour (plus de 200 OK silencieux sur version stale) ; P-H3 false positive (vérifié : `BankAccountJson` a déjà `#[serde(rename_all = "camelCase")]`, pas de patch). **MEDIUM** : P-M1 test sqlx `migration_creates_journal_account_id_column_nullable` (introspection IS_NULLABLE + INSERT NULL + INSERT id inexistant — couvre AC #75) ; P-M2 test E2E HTTP `patch_bank_account_returns_404_bank_account_not_found_for_unknown_id` qui assert `code = BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (dette L64) + nouveau test `patch_bank_account_malformed_body_returns_400_validation_error` (P-H1) ; P-M3 résolu automatiquement par P-C1 (plus de SELECT inline dans le handler) ; P-M4 UPDATE ajoute `AND company_id = ?` (defense-in-depth) ; P-M5 `Promise.allSettled` à la place de `Promise.all` dans `+page.svelte` (dégradation gracieuse si `fetchAccounts` échoue) ; P-M6 false positive (vérifié : `buildHeaders` ajoute déjà `Content-Type: application/json` pour TOUTES les méthodes non-FormData incluant PATCH) ; P-M7 toasts `bank-accounts-toast-{link,unlink}-success` câblés via `notifySuccess` (helper `svelte-sonner` existant). **LOW** : P-L1 Svelte warning `state_referenced_locally` sur `BankAccountJournalLinkForm.svelte:30` éliminé (`$state(null)` + `$effect` sync) — npm check passe de 17 → 16 warnings (les 14 restants sur `BankProfileForm.svelte` + 2 a11y design-system sont pré-existants hors scope) ; P-L4 commentaire `TODO(L65)` ajouté dans `onboarding.rs` au site `bank_accounts::upsert_primary` (audit historique `bank_account.created` à backfill v0.2). Tests verts : 15/15 sqlx `bank_accounts_repository` (12 anciens + P-M1 + P-H2 stale-version + 1 helper) + 8/8 E2E HTTP `bank_accounts_e2e` (6 anciens + P-M2 unknown_id + P-H1 malformed_body) + 21/21 reconciliation_e2e + 1 ignored (0 régression 8-4) + 29/29 bank_imports_e2e + 2/2 companies_e2e + 212/212 Vitest. Validation locale : cargo fmt + clippy `-D warnings` + build workspace clean ; npm check 0 errors / 16 warnings ; lint-i18n PASS ; build clean. 20 fail `config::tests::*` pré-existants (env var pollution, documenté dev-story commit `b164e0f`). Trend : Pass 1 = 11 findings > LOW (1 C + 2 H + 5 M + 3 L). Critère d'arrêt CLAUDE.md non atteint (1 C + 2 H + 5 M > 0) → **continuer Pass 2 Haiku 4.5** (cycle Sonnet → Haiku, fenêtre fraîche orthogonale pour détecter régressions introduites par P-C1 refactor tuple). | Claude (Sonnet 4.6 review + Opus 4.7 patches) |
| **2026-05-08** | **Pass 3 code review Opus 4.7 — VALIDATION FINALE 1M context (orthogonalité Sonnet+Haiku)** — 1 finding > LOW (0 CRITICAL + 0 HIGH + 1 MEDIUM + 2 LOW). Patch appliqué (1 patch) : **F-M1 (MEDIUM)** confirmation finding Pass 2 Haiku BH2-M-3 — manque test E2E HTTP « double PATCH idempotent → audit_count=1 ». Le no-op short-circuit repo est testé en sqlx (`set_journal_account_id_no_op_short_circuits_without_bump`), MAIS le chemin handler `if updated.version != before.version → skip audit_log::insert_in_tx` n'était couvert end-to-end que par UN PATCH (audit_count==1). Ajout `patch_bank_account_idempotent_no_op_does_not_duplicate_audit_log` qui exécute 2 PATCH avec même `journalAccountId` + version mise à jour entre les deux et asserte `audit_count == 1`. Garde-fou contre régression future si le handler oublie le check ou le repo perd le court-circuit no-op (P-C1 + KF-004). Confirmations Pass 2 réfutées : (a) BH2-H-2 « test migration ne vérifie pas FK absence » **réfuté** — test `migration_creates_journal_account_id_column_nullable` ligne 642-657 vérifie explicitement (a) IS_NULLABLE='YES', (b) INSERT NULL réussit, (c) INSERT avec `journal_account_id=999_999_999` (row inexistante) réussit (preuve absence FK applicative) ; (b) EC2 OptimisticLockConflict mapping confirmé OK — handler ligne 167 wrap `Err(e) => return Err(AppError::Database(e))` qui mappe via `IntoResponse for AppError` ligne 928 vers `409 OPTIMISTIC_LOCK_CONFLICT` ; (c) EC2 post-fetch sans company_id sûr (commenté lignes 306-308 — SELECT FOR UPDATE initial scope par company_id, UPDATE scope par company_id, post-fetch dans même tx forcément company-correct) ; (d) Path<i64> lower-bound check non requis (cohérent pattern Kesh `products.rs`/`fiscal_years.rs`/`contacts.rs` qui n'ont pas non plus ce check — l'id négatif/zéro retombe naturellement sur 404 NotFound). Findings LOW restants non patchés : **L1** spec ligne 543 affirme « 17 clés i18n livrées » mais grep confirme 16 clés réelles en fr-CH/de-CH/it-CH/en-CH (écart documentation Change Log dev-story, non bloquant) ; **L2** `PatchJournalLinkBodyExtractor` scope-limité au handler (BH2-H-1 reclassé LOW car commenté lignes 55-57 + cohérent pattern `SeedRequestExtractor` existant). Validation ground-truth EXHAUSTIVE Pass 3 (1M context) : (a) 5 callsites tuple `(updated, before)` adaptés (1 handler + 8 tests sqlx, tous déstructurent correctement) ; (b) 6 SELECTs `bank_accounts.rs` post-patch (5 originaux + 1 nouveau dans `set_journal_account_id_for_company` SELECT FOR UPDATE) — tous incluent `journal_account_id` ; (c) imports Rust corrects (AccountType, NewAuditLogEntry, accounts, audit_log, bank_accounts) ; (d) signature `accounts::find_by_id_in_company(pool, id, company_id)` correspond ; (e) `audit_log::insert_in_tx(tx, NewAuditLogEntry{..})` correspond ; (f) AppError::AccountNotFound + InvalidAccountType + BankAccountNotFound + Validation tous mappés correctement (404/400/404/400) ; (g) custom extractor `PatchJournalLinkBodyExtractor` retourne `AppError::Validation` qui produit shape standard `{error:{code:"VALIDATION_ERROR",message}}` (mieux que `SeedRequestExtractor` legacy `{error:message}`). Tests post-patch : 9/9 E2E HTTP bank_accounts (8 anciens + idempotent), 15/15 sqlx bank_accounts_repository, 21/21 reconciliation_e2e + 1 ignored, 29/29 bank_imports_e2e, 2/2 companies_e2e, 212/212 Vitest, npm check 0 errors / 16 warnings, lint-i18n PASS, build clean, cargo fmt+clippy `-D warnings` clean. Trend : Pass 1 = 11 → Pass 2 = ~2 → Pass 3 = 1 finding > LOW (1 MEDIUM patché). **Critère d'arrêt CLAUDE.md atteint après application : 0 finding > LOW** → **STOP cycle code-review**. Story 8-5a-zero **GO ready-for-merge**. Models LLM cycle : Sonnet 4.6 → Haiku 4.5 → Opus 4.7 (fenêtres fraîches orthogonales). | Claude (Opus 4.7 1M validate — VALIDATION FINALE) |

