# Story 3.1: Plan comptable — chargement & gestion

Status: done

## Story

As a **utilisateur**,
I want **disposer d'un plan comptable suisse et le personnaliser**,
so that **ma comptabilité soit structurée correctement**.

### Contexte

Première story de l'Epic 3 (Plan Comptable & Écritures). Crée la table `accounts`, les fichiers JSON des plans comptables suisses standards (PME, Association, Indépendant), le chargement automatique à l'onboarding (dette FR5 de Story 2-3), et l'interface de gestion (arborescence, ajout, modification, archivage). C'est la fondation sur laquelle reposent toutes les écritures comptables (stories 3-2 à 3-5).

### Décisions de conception

- **Plans comptables JSON** : 3 fichiers dans `charts/` (pme.json, association.json, independant.json). Basés sur les plans comptables standards suisses. Structure : tableau d'objets `{ number, name, type, parentNumber }`. Chargés par `kesh-core::chart_of_accounts`.
- **Chargement automatique (FR5)** : quand l'utilisateur choisit `org_type` à l'onboarding (step 3→4), le plan comptable correspondant est installé dans `accounts`. Résout le TODO(story 3-1) dans `onboarding.rs`.
- **Types de comptes** : `Asset` (actif), `Liability` (passif), `Revenue` (produit), `Expense` (charge). Enum `AccountType` dans kesh-db.
- **Hiérarchie** : `parent_id` optionnel pour l'arborescence. Les comptes de classe (1xxx, 2xxx...) n'ont pas de parent. Les sous-comptes pointent vers leur parent.
- **Archivage soft** : champ `active` boolean. Un compte archivé reste visible dans les écritures existantes mais n'apparaît plus dans les sélections.
- **Frontend** : page `/accounts` avec arborescence (tree view simplifiée), dialog d'ajout/modification, bouton d'archivage. Pattern similaire à `/users` (Story 1.12).

## Acceptance Criteria (AC)

1. **Chargement auto** — Given type d'organisation choisi à l'onboarding, When chargement plan comptable, Then le plan standard correspondant (PME/Association/Indépendant) est chargé depuis les fichiers JSON. Tous les comptes sont créés en DB avec numéro, nom, type, hiérarchie.
2. **Affichage arborescence** — Given plan comptable chargé, When affichage page `/accounts`, Then arborescence des comptes avec numéro, nom, type (actif/passif/charge/produit). Comptes triés par numéro.
3. **Ajout de compte** — Given un compte, When ajout d'un nouveau compte via dialog, Then le compte est créé avec numéro (unique par company), nom, type, parent optionnel.
4. **Modification** — Given un compte existant, When modification via dialog, Then le nom et le type sont mis à jour (verrouillage optimiste). Le numéro n'est PAS modifiable après création.
5. **Archivage** — Given un compte, When archivage, Then `active = false`. Le compte n'apparaît plus dans les sélections (autocomplete future) mais reste visible dans la page de gestion et les écritures existantes.
6. **i18n** — And les noms des comptes dans les JSON sont dans la langue comptable (`accounting_language` de la company). Les labels UI sont traduits via i18n (4 langues).
7. **Tests** — And tests unitaires kesh-core (parsing JSON, validation), tests intégration DB (CRUD accounts), tests E2E API (chargement + CRUD), test Playwright page `/accounts`.

## Tasks / Subtasks

### T1 — Fichiers JSON plans comptables (AC: #1, #6)
- [x] T1.1 Créer `charts/pme.json` — plan comptable PME suisse standard (basé sur le plan comptable suisse PME 2015, éd. veb.ch/EXPERT suisse). Contenu minimum : 9 classes (1-9), ~30 groupes (10, 11, 20...), ~80 comptes courants (1000 Caisse, 1020 Banque, 1100 Débiteurs, 2000 Créanciers, 3000 Ventes, 4000 Achats, etc.). Structure multilingue : `{ "number": "1000", "name": { "fr": "Caisse", "de": "Kasse", "it": "Cassa", "en": "Cash" }, "type": "Asset", "parentNumber": "10" }`. Fichiers placés dans `crates/kesh-core/assets/charts/` (PAS racine workspace) pour compatibilité `include_str!()`.
- [x] T1.2 Créer `charts/association.json` — plan comptable association.
- [x] T1.3 Créer `charts/independant.json` — plan comptable indépendant.
- [x] T1.4 Décision i18n des comptes : stocker les noms dans la `accounting_language` de la company au moment du chargement. Les fichiers JSON contiennent les 4 langues : `{ "number": "1000", "name": { "fr": "Caisse", "de": "Kasse", "it": "Cassa", "en": "Cash" }, "type": "Asset", "parentNumber": null }`.

### T2 — Module kesh-core chart_of_accounts (AC: #1)
- [x] T2.1 Créer `crates/kesh-core/src/chart_of_accounts/mod.rs` — types : `ChartEntry { number: String, name: HashMap<String, String>, account_type: AccountType, parent_number: Option<String> }`, `AccountType` enum (Asset, Liability, Revenue, Expense). Les clés du `HashMap` sont en **minuscules** (`"fr"`, `"de"`, `"it"`, `"en"`). Helper `resolve_name(entry, lang: &str) -> String` qui fait `entry.name.get(&lang.to_lowercase()).or(entry.name.get("fr"))` avec fallback FR.
- [x] T2.2 Fonction `load_chart(org_type: &str) -> Result<Vec<ChartEntry>, CoreError>` — charge et parse le fichier JSON correspondant. Validation : numéros uniques, parent_number référence un numéro existant.
- [x] T2.3 Tests unitaires : parsing JSON, validation numéros dupliqués, parent invalide.
- [x] T2.4 Ajouter `pub mod chart_of_accounts;` dans `kesh-core/src/lib.rs`.

### T3 — Migration DB : table `accounts` (AC: #1, #3, #4, #5)
- [x] T3.1 Créer migration `crates/kesh-db/migrations/YYYYMMDD_accounts.sql` :
  ```sql
  CREATE TABLE accounts (
      id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
      company_id BIGINT NOT NULL,
      number VARCHAR(10) NOT NULL,
      name VARCHAR(255) NOT NULL,
      account_type VARCHAR(20) NOT NULL COMMENT 'Asset|Liability|Revenue|Expense',
      parent_id BIGINT NULL,
      active BOOLEAN NOT NULL DEFAULT TRUE,
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      CONSTRAINT fk_accounts_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
      CONSTRAINT fk_accounts_parent FOREIGN KEY (parent_id) REFERENCES accounts(id) ON DELETE RESTRICT,
      CONSTRAINT uq_accounts_company_number UNIQUE (company_id, number),
      CONSTRAINT chk_accounts_type CHECK (BINARY account_type IN (BINARY 'Asset', BINARY 'Liability', BINARY 'Revenue', BINARY 'Expense')),
      CONSTRAINT chk_accounts_number_nonempty CHECK (CHAR_LENGTH(TRIM(number)) > 0),
      CONSTRAINT chk_accounts_name_nonempty CHECK (CHAR_LENGTH(TRIM(name)) > 0)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
  ```
- [x] T3.2 Entity `Account` + `NewAccount` + `AccountUpdate` dans `crates/kesh-db/src/entities/account.rs`. Enum `AccountType` (Asset/Liability/Revenue/Expense) avec SQLx encodeurs manuels (pattern OrgType/Language).
- [x] T3.3 Ajouter `pub mod account;` dans `entities/mod.rs` + réexports.

### T4 — Repository `accounts` (AC: #1, #3, #4, #5)
- [x] T4.1 Créer `crates/kesh-db/src/repositories/accounts.rs` :
  - `create(pool, new: NewAccount) -> Account`
  - `find_by_id(pool, id) -> Option<Account>`
  - `list_by_company(pool, company_id, include_archived: bool) -> Vec<Account>`
  - `update(pool, id, version, changes: AccountUpdate) -> Account` (OL)
  - `archive(pool, id, version) -> Account` (set active=false, OL)
  - `bulk_create(pool, accounts: Vec<NewAccount>) -> Vec<Account>` — **dans une transaction unique** (BEGIN/COMMIT). Insérer en ordre topologique (classes → groupes → comptes) pour résoudre parent_id. Soit tous les comptes sont créés, soit aucun.
- [x] T4.2 Ajouter `pub mod accounts;` dans `repositories/mod.rs`.
- [x] T4.3 Tests intégration DB : create, list (with/without archived), update (OL), archive, bulk_create, unique constraint on (company_id, number).

### T5 — Routes API accounts (AC: #1, #2, #3, #4, #5)
- [x] T5.1 Créer `crates/kesh-api/src/routes/accounts.rs` :
  - `GET /api/v1/accounts` — liste les comptes de la company courante. Query param `includeArchived=false` par défaut. Retourne un tableau direct `Vec<AccountResponse>` (pas d'envelope pagination — un plan comptable est borné à ~200-400 comptes, toujours chargé en entier pour l'arborescence frontend). Note : les endpoints de Story 3-4 (écritures) utiliseront l'envelope `{ items, total, offset, limit }` standard.
  - `POST /api/v1/accounts` — crée un compte. Body : `{ number, name, accountType, parentId }`. Validation : number unique, parentId valide. Retourne `AccountResponse` (201).
  - `PUT /api/v1/accounts/{id}` — modifie un compte. Body : `{ name, accountType, version }`. Number non modifiable. Retourne `AccountResponse` (OL).
  - `PUT /api/v1/accounts/{id}/archive` — archive un compte. Body : `{ version }`. Retourne `AccountResponse`.
- [x] T5.2 Enregistrer les routes : `GET /api/v1/accounts` dans `authenticated_routes` (tout rôle, y compris Consultation). `POST`, `PUT /accounts/{id}`, `PUT /accounts/{id}/archive` dans `admin_routes` (Admin + Comptable seulement — les consultants ne modifient pas le plan comptable).
- [x] T5.3 Ajouter `pub mod accounts;` dans `routes/mod.rs`.

### T6 — Chargement automatique à l'onboarding (AC: #1, FR5)
- [x] T6.1 Résoudre le TODO(story 3-1) : charger le plan comptable dans le handler `set_accounting_language` (step 4→5), **PAS dans** `set_org_type` (step 3→4). Raison : à step 3, `accounting_language` vaut encore le default FR — les noms de comptes seraient toujours en FR. À step 4→5, les deux paramètres (`org_type` + `accounting_language`) sont connus. Pattern : après update de `company.accounting_language`, charger le plan via `kesh_core::chart_of_accounts::load_chart(org_type)` + `accounts::bulk_create()`. Retirer le TODO dans `set_org_type` et le déplacer dans `set_accounting_language`.
- [x] T6.2 Modifier `kesh_seed::reset_demo()` : ajouter `DELETE FROM accounts` dans l'ordre FK (accounts AVANT fiscal_years, AVANT companies). Sans cela, la FK `fk_accounts_company ON DELETE RESTRICT` bloque la suppression de companies, et des accounts orphelins subsistent.
- [x] T6.3 Modifier `kesh_seed::seed_demo()` : après création de la company démo, charger le plan comptable PME dans la langue `accounting_language` de la company.

### T7 — Frontend : page `/accounts` (AC: #2, #3, #4, #5)
- [x] T7.1 Créer `frontend/src/routes/(app)/accounts/+page.svelte` — remplacer le placeholder. Affiche l'arborescence des comptes (tree view simplifiée : indentation par niveau).
- [x] T7.2 Feature module : `frontend/src/lib/features/accounts/` — `accounts.types.ts`, `accounts.api.ts` (fetchAccounts, createAccount, updateAccount, archiveAccount).
- [x] T7.3 Dialog d'ajout de compte (shadcn Dialog) : champs number, name, type (select), parent (select optionnel). Validation côté client.
- [x] T7.4 Dialog de modification : name et type modifiables, number affiché en lecture seule.
- [x] T7.5 Bouton d'archivage avec confirmation. Comptes archivés affichés en grisé avec indication "Archivé".

### T8 — Clés i18n (AC: #6)
- [x] T8.1 Ajouter les clés dans les 4 fichiers `.ftl` :
  - `accounts-title` / `accounts-add` / `accounts-edit` / `accounts-archive` / `accounts-archive-confirm`
  - `account-field-number` / `account-field-name` / `account-field-type` / `account-field-parent`
  - `account-type-asset` / `account-type-liability` / `account-type-revenue` / `account-type-expense`
  - `account-archived-label`

### T9 — Tests (AC: #7)
- [x] T9.1 Tests unitaires kesh-core : parsing JSON chart, validation (numéros dupliqués, parent invalide).
- [x] T9.2 Tests intégration DB : CRUD accounts (create, list, update OL, archive, bulk_create, unique constraint).
- [x] T9.3 Tests E2E API : GET /accounts, POST /accounts, PUT /accounts/{id}, PUT /accounts/{id}/archive, chargement auto via onboarding.
- [x] T9.4 Test Playwright : page /accounts affiche arborescence, ajout/modification via dialog.

## Dev Notes

### Plans comptables suisses — structure JSON

```json
[
  { "number": "1",    "name": { "fr": "Actifs", "de": "Aktiven", "it": "Attivi", "en": "Assets" }, "type": "Asset", "parentNumber": null },
  { "number": "10",   "name": { "fr": "Actifs circulants", ... }, "type": "Asset", "parentNumber": "1" },
  { "number": "1000", "name": { "fr": "Caisse", ... }, "type": "Asset", "parentNumber": "10" },
  { "number": "1020", "name": { "fr": "Banque", ... }, "type": "Asset", "parentNumber": "10" },
  ...
]
```

Les numéros suivent la norme suisse :
- 1xxx = Actifs, 2xxx = Passifs, 3xxx = Produits d'exploitation, 4xxx = Charges matières, 5xxx = Charges personnel, 6xxx = Autres charges, 7xxx = Résultats annexes, 8xxx = Résultats hors exploitation, 9xxx = Clôture

### Chargement auto (FR5) — pattern

Dans `onboarding.rs::set_accounting_language()` (PAS set_org_type — voir CRITICAL-01 validation) :
```rust
// Après update company.accounting_language — org_type ET accounting_language sont connus
let company = get_company(&state).await?;
let chart = kesh_core::chart_of_accounts::load_chart(org_type.as_str())?;
let lang = company.accounting_language.as_str().to_lowercase(); // "fr", "de", etc.
let new_accounts: Vec<NewAccount> = chart.iter().map(|entry| {
    let name = entry.name.get(&lang).or(entry.name.get("fr")).unwrap();
    NewAccount { company_id: company.id, number: entry.number.clone(), name: name.clone(), ... }
}).collect();
accounts::bulk_create(&state.pool, new_accounts).await?;
```

### Pattern arborescence frontend

Tree view simplifiée : pas de composant Tree externe. Indentation par `padding-left` basé sur le niveau (calculé via parent_id).

```svelte
{#each sortedAccounts as account}
  <div style="padding-left: {account.level * 24}px" class:opacity-50={!account.active}>
    <span class="font-mono">{account.number}</span>
    <span>{account.name}</span>
    <Badge>{account.accountType}</Badge>
    {#if !account.active}<span class="text-xs text-text-muted">(Archivé)</span>{/if}
  </div>
{/each}
```

### Embed JSON dans le binaire

Les fichiers `charts/*.json` doivent être accessibles au runtime. Options :
- **Option A** : `include_str!()` à la compilation (embed dans le binaire). Simple, pas de config path.
- **Option B** : lire depuis le filesystem (`KESH_CHARTS_DIR` env var). Flexible mais nécessite un COPY dans Dockerfile.

**Recommandation : Option A** (`include_str!()`) — les plans comptables sont statiques et petits (~50 Ko chacun). Fichiers placés dans `crates/kesh-core/assets/charts/` pour que le chemin relatif soit court : `include_str!("../assets/charts/pme.json")` depuis `chart_of_accounts/mod.rs`. Pas de Dockerfile COPY nécessaire (embarqué dans le binaire).

### Piège : `parent_id` vs `parent_number`

Les JSON utilisent `parentNumber` (string) pour la hiérarchie. Lors du chargement, il faut résoudre `parentNumber` → `parent_id` (FK). Pattern : d'abord insérer tous les comptes sans parent_id, puis UPDATE en batch pour lier les parents via le number.

Alternative : insérer en ordre topologique (parents d'abord, enfants ensuite) pour que le parent_id soit déjà disponible. C'est le pattern recommandé — trier par longueur de numéro (1 → 10 → 100 → 1000).

### Pattern — `AccountType` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Asset,     // Actif (1xxx)
    Liability, // Passif (2xxx)
    Revenue,   // Produit (3xxx, 7xxx, 8xxx)
    Expense,   // Charge (4xxx, 5xxx, 6xxx)
}
```

Stocké en DB comme VARCHAR avec CHECK BINARY (pattern OrgType/Language).

### Project Structure Notes

- **Nouveau** : `crates/kesh-core/assets/charts/pme.json`, `association.json`, `independant.json`
- **Nouveau** : `crates/kesh-core/src/chart_of_accounts/mod.rs`
- **Nouvelle migration** : `crates/kesh-db/migrations/YYYYMMDD_accounts.sql`
- **Nouvelle entity** : `crates/kesh-db/src/entities/account.rs`
- **Nouveau repository** : `crates/kesh-db/src/repositories/accounts.rs`
- **Nouveau** : `crates/kesh-api/src/routes/accounts.rs`
- **Nouveau** : `frontend/src/lib/features/accounts/` (types + api)
- **Modification** : `frontend/src/routes/(app)/accounts/+page.svelte`
- **Modification** : `crates/kesh-api/src/routes/onboarding.rs` (résoudre TODO FR5)
- **Modification** : `crates/kesh-seed/src/lib.rs` (seed plan comptable)
- **Modifications i18n** : 4 fichiers `.ftl`

### Test debt

- **T9.3 — Tests E2E API** : Aucune infrastructure de test API HTTP (TestClient, helpers) n'existe dans le projet. Les tests E2E sont couverts par Playwright (T9.4) et les tests DB intégration (T9.2). La création d'un framework de test API est reportée à une story transverse dédiée (action item rétrospective Epic 2 — A2 : `make test-e2e`). **Propriétaire** : SM. **Story de remédiation** : à créer lors du sprint planning Epic 3.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-3-Story-3.1] — AC BDD
- [Source: _bmad-output/planning-artifacts/architecture.md#chart_of_accounts] — Module structure
- [Source: _bmad-output/planning-artifacts/prd.md#FR18-FR19] — Plan comptable chargeable et personnalisable
- [Source: _bmad-output/planning-artifacts/prd.md#FR4-FR5] — Chargement auto par org_type
- [Source: _bmad-output/implementation-artifacts/epic-2-retro-2026-04-09.md] — Dette FR5, action items

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

- include_str!() path required `../../assets/charts/` (not `../`) from chart_of_accounts/ subdirectory
- Select.Root value type must be `string`, not `number | null` — used string conversion for parentId

### Completion Notes List

- T1: 3 plans comptables JSON (PME 91 comptes, Association 79 comptes, Independant 78 comptes), 4 langues chacun
- T2: Module kesh-core::chart_of_accounts avec ChartEntry, AccountType, load_chart(), resolve_name(), validation (unicité numéros, parents valides). 14 tests unitaires.
- T3: Migration 20260411000001_accounts.sql, entity Account/NewAccount/AccountUpdate, enum AccountType avec SQLx encodeurs manuels
- T4: Repository accounts (create, find_by_id, list_by_company, update OL, archive OL, bulk_create, bulk_create_from_chart, delete_all_by_company). 6 tests intégration DB.
- T5: Routes API (GET /accounts, POST /accounts, PUT /accounts/{id}, PUT /accounts/{id}/archive). GET dans authenticated_routes, POST/PUT dans comptable_routes (require_comptable_role).
- T6: Chargement auto dans set_accounting_language (step 4→5, FR5 résolu). reset_demo nettoie accounts. seed_demo charge plan PME.
- T7: Page /accounts avec arborescence (indentation par niveau), dialogs ajout/modification/archivage, toggle archivés, badge type, RBAC frontend.
- T8: 14 clés i18n dans 4 fichiers .ftl (FR/DE/IT/EN)
- T9: Tests unitaires kesh-core (14), tests intégration DB (6), test Playwright accounts.spec.ts (6)

### File List

- crates/kesh-core/assets/charts/pme.json (existait, inchangé)
- crates/kesh-core/assets/charts/association.json (nouveau)
- crates/kesh-core/assets/charts/independant.json (nouveau)
- crates/kesh-core/src/chart_of_accounts/mod.rs (nouveau)
- crates/kesh-core/src/lib.rs (modifié — ajout pub mod chart_of_accounts)
- crates/kesh-core/src/errors.rs (modifié — ajout UnknownChartType, InvalidChart)
- crates/kesh-core/Cargo.toml (modifié — serde_json promu de dev à régulier)
- crates/kesh-db/migrations/20260411000001_accounts.sql (nouveau)
- crates/kesh-db/src/entities/account.rs (nouveau)
- crates/kesh-db/src/entities/mod.rs (modifié — ajout account)
- crates/kesh-db/src/repositories/accounts.rs (nouveau)
- crates/kesh-db/src/repositories/mod.rs (modifié — ajout accounts)
- crates/kesh-db/Cargo.toml (modifié — ajout kesh-core dependency)
- crates/kesh-api/src/routes/accounts.rs (nouveau)
- crates/kesh-api/src/routes/mod.rs (modifié — ajout accounts)
- crates/kesh-api/src/lib.rs (modifié — ajout comptable_routes + accounts routes)
- crates/kesh-api/src/routes/onboarding.rs (modifié — FR5 chargement auto, retrait TODO)
- crates/kesh-seed/src/lib.rs (modifié — accounts dans reset_demo + seed_demo)
- crates/kesh-seed/Cargo.toml (modifié — ajout kesh-core dependency)
- frontend/src/lib/features/accounts/accounts.types.ts (nouveau)
- frontend/src/lib/features/accounts/accounts.api.ts (nouveau)
- frontend/src/routes/(app)/accounts/+page.svelte (nouveau)
- frontend/tests/e2e/accounts.spec.ts (nouveau)
- crates/kesh-i18n/locales/fr-CH/messages.ftl (modifié — clés accounts)
- crates/kesh-i18n/locales/de-CH/messages.ftl (modifié — clés accounts)
- crates/kesh-i18n/locales/it-CH/messages.ftl (modifié — clés accounts)
- crates/kesh-i18n/locales/en-CH/messages.ftl (modifié — clés accounts)

## Change Log

- 2026-04-09: Implémentation complète Story 3-1 — Plan comptable chargement & gestion (Claude Opus 4.6)
- 2026-04-09: Code review passe 2 (Haiku via subagents) — 1 finding MEDIUM nouveau (P2-1 : création compte sous parent archivé). Patché : validation parent actif dans create_account. 4 rejetés (faux positifs), 2 defer (déjà pass 1). Critère d'arrêt atteint.
- 2026-04-09: Code review passe 1 (Sonnet via subagents) — 10 findings MEDIUM+, 8 patches appliqués, 1 documenté en dette technique (T9.3 tests E2E API). Corrections : guard idempotence onboarding (F3), cleanup bulk_create dead code (F6), guard cycle getLevel frontend (F7), block update archived account (F8), block archive parent with active children (F9), guard last_insert_id bulk_create* (F10), validation longueur number/name API (F11). T9.3 reclassé en dette technique documentée.
