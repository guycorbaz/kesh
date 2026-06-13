# Story 11.1: Configuration des taux TVA (CRUD admin + sélection temporelle)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## ⚠️ Note de cadrage (ground-truth vérifié 2026-06-13)

**L'infrastructure de base des taux TVA existe déjà** (Story 7.2 / KF-003) — cette story l'**enrichit**, elle ne la construit pas de zéro. Déjà en place :

- Table **`vat_rates`** (`crates/kesh-db/migrations/20260428000001_vat_rates.sql`) : `id, company_id, label, rate DECIMAL(5,2), valid_from DATE, valid_to DATE NULL, active BOOL, created_at, updated_at`. Contraintes : `chk_vat_rates_rate_range (0..100)`, `chk_vat_rates_dates (valid_to > valid_from)`, `uq_vat_rates_company_rate_valid_from`.
- Entité **`crates/kesh-db/src/entities/vat_rate.rs`** (`VatRate`, `NewVatRate`).
- Repository **`crates/kesh-db/src/repositories/vat_rates.rs`** : `list_active_for_company()`, **`list_all_by_company()`** (déjà présente, Story 9-2b, sans filtre `active`, triée `id ASC`, utilisée par l'export global `exports/global.rs` — **couvre déjà la vue historique** : NE PAS créer de `list_all_for_company` doublon), `find_active_by_rate()`, `seed_default_swiss_rates()` + `seed_default_swiss_rates_in_tx()`.
- Routes read-only **`crates/kesh-api/src/routes/vat.rs`** : `list_vat_rates()` (GET), `verify_vat_rates_against_db()`, `VatRateResponse`.
- Frontend feature **`frontend/src/lib/features/vat-rates/`** (`listVatRates()`, store `getVatRates()`).
- **Pré-config onboarding** : 4 taux suisses 2024+ seedés atomiquement dans `finalize()` (`onboarding.rs`) — `normal 8.10`, `special 3.80`, `reduced 2.60`, `exempt 0.00`, `valid_from 2024-01-01`, `valid_to NULL`. i18n `product-vat-{normal,special,reduced,exempt}`.
- Tests : `crates/kesh-db/tests/vat_rates_repository.rs`.

**Ce qui MANQUE (le périmètre de cette story)** : aucun moyen pour un administrateur de **gérer** les taux après l'onboarding (création/modification/désactivation), pas de **sélection du taux applicable à une date** d'opération, pas de **vue historique** (les reads ne renvoient que `active = TRUE`), pas de colonne `version` (verrou optimiste), pas d'événements d'audit.

## Story

As a **administrateur d'une entreprise sur Kesh**,
I want **configurer les taux de TVA de mon entreprise (créer, modifier, désactiver) avec des dates de validité et un historique consultable**,
so that **les changements de taux suisses (ex. 7.7 % → 8.1 % en 2024) soient gérés correctement dans le temps, les anciennes opérations conservant leur taux d'époque**.

## Scope

**Cible** : enrichir l'infrastructure `vat_rates` existante avec un CRUD admin + sélection temporelle + historique. **Réutiliser strictement le pattern CRUD de référence `bank_accounts`** (Story v014-1) : guard onboarding, validation, verrou optimiste `version`, audit dans la même transaction.

### Dans le scope

1. **Migration** `ALTER TABLE vat_rates ADD COLUMN version INT NOT NULL DEFAULT 0` (verrou optimiste pour les updates) + ligne dans `docs/migrations-idempotence-audit.md` (politique P5). Non-breaking (`ADD COLUMN` → pas de bump `kesh_version_min_required`).
2. **Repository** (`vat_rates.rs`) — nouvelles fonctions calquées sur `repositories/bank_accounts.rs` :
   - `pub async fn create_for_company(tx, company_id, new) -> Result<VatRate, DbError>`
   - `pub async fn update_for_company(tx, company_id, id, fields, version) -> Result<VatRate, DbError>` (verrou optimiste + `SELECT … FOR UPDATE` ; en cas de conflit de version `rows_affected() == 0` → `return Err(DbError::OptimisticLockConflict)`, exactement comme `bank_accounts.rs:381`). Le handler propage via `AppError::Database(e)` → HTTP 409 (cf. AC#2).
   - `pub async fn deactivate_for_company(tx, company_id, id, version) -> Result<VatRate, DbError>` (soft : `active = FALSE` , même verrou optimiste ; **jamais de hard-delete** d'un taux potentiellement référencé par des écritures/factures historiques — audit-trail).
   - (signatures calquées sur `repositories/bank_accounts.rs` ; ne retourner que l'état final `VatRate` suffit — pas besoin du tuple `(avant, après)` de bank_accounts.)
   - **Vue historique** : réutiliser **`list_all_by_company()` existante** (NE PAS créer `list_all_for_company` — doublon de la fonction déjà utilisée par `exports/global.rs`).
   - `list_active_at_date(pool, company_id, date) -> Vec<VatRate>` — **tous** les taux en vigueur à une date : `valid_from <= date AND (valid_to IS NULL OR date < valid_to) AND active`, scopé `company_id`. **Sémantique** : `valid_from` inclusif, `valid_to` exclusif (cohérent `chk_vat_rates_dates`). (nouvelle fonction — aucun équivalent n'existe.) **Retourne un `Vec`, PAS un `Option`** : à une date donnée, plusieurs taux coexistent (la grille suisse — normal/réduit/spécial/exonéré — est multi-taux simultanément ; cf. les 4 taux seedés tous `valid_from 2024-01-01`). La **sélection d'un taux unique par catégorie** (« le taux normal au 2024-06-01 ») n'est PAS possible avec le schéma actuel (pas de colonne `category` — le seul discriminant est `label`) et relève de la **Story 11-2** (calcul, où la ligne porte sa catégorie). Voir §"Décision de design déférée à 11-2" ci-dessous.
3. **Routes API** (`vat.rs`) — handlers calqués sur `bank_accounts.rs` :
   - `POST   /api/v1/vat-rates` (création) → `vat_rate.created`
   - `PUT    /api/v1/vat-rates/{id}` (modification label / `valid_to` / `active`) → `vat_rate.updated`
   - `DELETE /api/v1/vat-rates/{id}` (désactivation soft) → `vat_rate.deactivated`
   - `GET    /api/v1/vat-rates?history=true` → liste complète (actifs + expirés) ; sans le param = comportement actuel (`active` seulement) **préservé** (non-régression des consommateurs existants : facturation, produits).
   - Guards : **rôle Administrateur** (FR54 « l'administrateur peut configurer ») + `assert_onboarding_complete`. Audit dans la même `tx` que la mutation.
4. **Changement de taux dans le temps** (AC#5) — modélisé sans mutation destructive : pour passer 7.7 % → 8.1 %, l'admin **crée un nouveau taux** (`valid_from` = date de bascule) et **clôt l'ancien** (`PUT` `valid_to` = date de bascule). Les opérations antérieures conservent l'ancien taux via `list_active_at_date`. Le formulaire UI guide ce flux (pas d'édition du champ `rate` d'un taux existant — un taux a une valeur figée ; on en crée un nouveau).
5. **Frontend** — page admin de gestion des taux TVA (`Paramètres` → `TVA` ou `Administration`), calquée sur l'UI CRUD `bank_accounts` : liste avec colonne actif/expiré + historique, formulaire création, édition `valid_to`/label, désactivation avec confirmation. Réutiliser la feature `vat-rates/`.
6. **i18n** — clés Fluent pour l'UI admin (titres, colonnes, messages d'erreur, libellés boutons) dans `fr-CH` (+ stubs DE/IT/EN selon convention `lint-i18n-ownership`).
7. **Validation** (par taux, pattern `FailedProposal`/AppError selon convention) : `rate` 0..100 + `scale ≤ 2`, `valid_to > valid_from`, unicité `(company_id, rate, valid_from)` (déjà en DB), label non vide. Erreurs business per-requête (pas batch ici → `AppError` ciblé acceptable car requête single-resource).
8. **Tests** : repository (CRUD, verrou optimiste, `list_active_at_date` aux bornes — veille de `valid_from` / jour de `valid_from` / jour de `valid_to`, taux inactif exclu), routes (RBAC admin, validation, audit émis, history vs active), E2E Playwright (créer un taux, clore l'ancien, vérifier l'historique).

### Hors scope (→ Story 11-2)

- **Calcul de la TVA** sur les lignes de facture/écriture, **arrondi commercial** au centime (FR55), **rapport TVA par période** AFC (FR56), export PDF/CSV du décompte. Tout cela est la Story 11-2.
- **FK `products.vat_rate → vat_rates`** : la non-liaison est un choix v0.1 intentionnel (validation asynchrone `verify_vat_rates_against_db`). Ne pas introduire la FK ici.
- **Helper d'arrondi commercial** (banker's vs commercial) : appartient à 11-2.

## Acceptance Criteria

1. **(FR54)** Un administrateur peut **créer** un taux TVA : `POST /api/v1/vat-rates` avec `label`, `rate`, `valid_from`, `valid_to` (optionnel). Validation : `rate` 0..100 scale ≤ 2, `valid_to > valid_from`, label non vide, unicité `(company_id, rate, valid_from)`. Événement audit `vat_rate.created` dans la même transaction.
2. Un administrateur peut **modifier** un taux existant (`PUT /api/v1/vat-rates/{id}` : `label`, `valid_to`, `active`) avec **verrou optimiste** (`version`) — un conflit de version remonte `DbError::OptimisticLockConflict` (repo) ; le handler **propage `AppError::Database(e)`** (exactement comme `bank_accounts.rs:544/671`), rendu **HTTP 409 `OPTIMISTIC_LOCK_CONFLICT`** par le mapper central (`errors.rs:1645`, i18n existante `error-optimistic-lock`). **NE PAS** créer de clé i18n `vat_rate.conflict_version` ni re-mapper en `AppError::Validation` (400) — ce serait un mauvais code HTTP + une divergence du pattern. Le champ `rate` n'est **pas** modifiable (un taux a une valeur figée ; changer de taux = créer un nouveau). Audit `vat_rate.updated`.
3. **(AC historique)** Un administrateur peut **désactiver** un taux (`DELETE` soft → `active = FALSE`) sans suppression physique (préservation audit-trail). Audit `vat_rate.deactivated`. Un taux désactivé n'est plus proposé à la saisie mais reste visible dans l'historique et applicable aux opérations passées via les dates.
4. **(AC validité temporelle)** `list_active_at_date(company, date)` retourne **tous** les taux dont `valid_from <= date < valid_to` (ou `valid_to IS NULL`) et `active`. Testé aux bornes : la veille de `valid_from` → taux absent de la liste ; le jour de `valid_from` → présent ; le jour de `valid_to` → absent (exclusif). (La sélection d'**un** taux par catégorie est hors-scope 11-1 → 11-2.)
5. **(AC changement de taux 7.7→8.1)** Scénario : partant d'un taux 7.70 `valid_from = 2023-…, valid_to = NULL`, l'admin crée un taux 8.10 `valid_from = 2024-01-01` puis clôture le 7.70 (`PUT valid_to = 2024-01-01`). Vérification via `list_active_at_date` : au **2023-12-31**, la liste contient 7.70 et **pas** 8.10 ; au **2024-01-01**, contient 8.10 et **pas** 7.70. Les deux coexistent dans l'historique (`?history=true`). Démontre que l'ancien taux reste applicable aux opérations antérieures.
6. **(AC liste + historique)** `GET /api/v1/vat-rates` (sans param) renvoie les taux **actifs** (comportement actuel **inchangé** — non-régression facturation/produits). `GET /api/v1/vat-rates?history=true` renvoie la **liste complète** (actifs + inactifs/expirés) via `list_all_by_company`, triée **`valid_from DESC, rate ASC`** côté handler (la fonction repo trie `id ASC` ; ré-ordonner pour l'affichage admin), pour la vue admin.
7. **(RBAC)** Les mutations (POST/PUT/DELETE) sont réservées au **rôle Administrateur** (FR54) et requièrent l'onboarding complété (`assert_onboarding_complete`). Un rôle insuffisant → 403.
8. **(Migration)** Colonne `version INT NOT NULL DEFAULT 0` ajoutée à `vat_rates` (non-breaking, pas de bump `kesh_version_min_required`). Ligne ajoutée à `docs/migrations-idempotence-audit.md` avec verdict + justification.
9. **(Frontend)** Page admin de gestion des taux TVA : liste (actif/expiré/historique), création (label **texte libre**), édition `valid_to`/label, désactivation avec confirmation. Calquée sur l'UI CRUD `bank_accounts`. Les labels sont affichés via `i18nMsg(label, label)` (clés seedées traduites, texte libre admin affiché brut — cf. §Pièges). La réponse expose `version` (verrou optimiste). États de chargement/erreur gérés. Pas d'API secure-context-only (déploiement HTTP LAN — cf. `feedback_no_secure_context_apis_http_lan`).
10. **(i18n)** Toutes les chaînes UI nouvelles ont des clés Fluent `fr-CH` + stubs DE/IT/EN ; `npm run lint-i18n-ownership` vert.
11. **(Tests)** Repository (CRUD, verrou optimiste, `list_active_at_date` aux bornes, taux inactif exclu), routes (RBAC, validation, audit, history vs active), E2E (créer/clore/historique). `cargo test --workspace` + `npm run test:unit` + build verts.
12. **(Préservation)** Le seed onboarding existant (`seed_default_swiss_rates_in_tx`) et les consommateurs read-only existants (facturation, produits, `verify_vat_rates_against_db`) ne sont pas régressés. Le seed pose `version = 0` (défaut).

## Tasks / Subtasks

- [ ] **T1 — Migration `version` + audit idempotence** (AC #8)
  - [ ] T1.1 `crates/kesh-db/migrations/YYYYMMDDXXXXXX_vat_rates_version.sql` : `ALTER TABLE vat_rates ADD COLUMN version INT NOT NULL DEFAULT 0;` (+ index si pertinent pour `list_all_by_company`/`list_active_at_date`).
  - [ ] T1.2 Ajouter la ligne correspondante dans `docs/migrations-idempotence-audit.md` (verdict + justification ; `ADD COLUMN` non-breaking → pas de bump `kesh_version_min_required`).
- [ ] **T2 — Entité + repository CRUD** (AC #1-6, #12)
  - [ ] T2.1 Étendre `entities/vat_rate.rs` : champ `version`, structs `UpdateVatRate` / payload create.
  - [ ] T2.2 `repositories/vat_rates.rs` : `create_for_company`, `update_for_company` (optimistic lock + `FOR UPDATE`, calqué `bank_accounts`), `deactivate_for_company`, `list_active_at_date` (retourne `Vec`, scopé `company_id`). **Réutiliser `list_all_by_company()` existante** pour l'historique (ne PAS créer de doublon). Préserver les fonctions existantes (`list_active_for_company`, `find_active_by_rate`, `list_all_by_company`, seed). Toutes les fonctions filtrent par `company_id` (défense IDOR).
- [ ] **T3 — Routes API + audit** (AC #1-3, #6, #7)
  - [ ] T3.1 `routes/vat.rs` : handlers POST/PUT/DELETE (guard admin + `assert_onboarding_complete`), audit `vat_rate.{created,updated,deactivated}` dans la même `tx` (pattern `bank_accounts.rs`). Étendre `VatRateResponse` avec le champ **`version`** (additif — non-régression pour les consommateurs read-only ; nécessaire au client pour le PUT optimiste, comme `bank_accounts` expose `version`).
  - [ ] T3.2 `GET /vat-rates?history=true` (liste complète via `list_all_by_company` existante) — param optionnel, défaut = comportement actuel inchangé (`list_active_for_company`). Avant de retourner en mode history, **ré-ordonner côté handler** par `valid_from DESC, rate ASC` (la fonction repo trie `id ASC`).
  - [ ] T3.3 Validation (rate range/scale, dates, label) + mapping `AppError` i18n.
  - [ ] T3.4 Enregistrer POST/PUT/DELETE dans **`admin_routes`** de `crates/kesh-api/src/lib.rs` (~l.158-172, là où vivent `full_export`/`full_import`), **PAS** dans `comptable_routes` — écart **intentionnel** vs `bank_accounts` (qui est Comptable+) : les mutations TVA sont réservées Admin (FR54). Le `GET /vat-rates` reste dans les routes authentifiées existantes (inchangé).
- [ ] **T4 — Frontend page admin TVA** (AC #9, #10)
  - [ ] T4.1 Page + feature lib (calquée `bank_accounts` UI) : liste actif/historique, création, édition `valid_to`/label, désactivation + confirmation.
  - [ ] T4.2 Guard de route (rôle admin), états chargement/erreur, pas d'API secure-context-only.
  - [ ] T4.3 i18n `fr-CH` + stubs DE/IT/EN, `lint-i18n-ownership` vert.
- [ ] **T5 — Tests** (AC #11)
  - [ ] T5.1 Repository : CRUD, verrou optimiste (conflit version → `DbError::OptimisticLockConflict`), `list_active_at_date` aux bornes (veille de `valid_from` exclue / jour de `valid_from` inclus / jour de `valid_to` exclu) + cas multi-taux (4 taux seedés tous présents au 2024-06-01), taux inactif exclu, `list_all_by_company` vs `list_active_for_company`.
  - [ ] T5.2 Routes : RBAC admin (403 si insuffisant), validation, audit émis, history vs active, non-régression GET sans param.
  - [ ] T5.3 E2E Playwright : créer un taux, clôturer l'ancien (`valid_to`), vérifier l'historique + que la saisie ne propose que les actifs.
- [ ] **T6 — Vérifs finales** (AC #11, #12)
  - [ ] T6.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (serial si touche kesh-db : `-j1 -- --test-threads=1`).
  - [ ] T6.2 `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build`.
  - [ ] T6.3 E2E (`npm run test:e2e`) — pré-requis MariaDB + seed + browsers (`PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`).

## Dev Notes

### Pattern de référence à suivre (NE PAS réinventer)

- **CRUD complet** : `crates/kesh-api/src/routes/bank_accounts.rs` + `crates/kesh-db/src/repositories/bank_accounts.rs` (Story v014-1). Reprendre : guard `assert_onboarding_complete`, validation, `update_for_company` avec verrou optimiste `version` + `SELECT … FOR UPDATE`, soft-delete (`active`), audit **dans la même transaction** que la mutation.
- **Advisory lock company — NON requis pour vat_rates** : `bank_accounts` utilise `acquire_company_sentinel_lock` à cause de la contrainte cross-row `is_primary` (un seul compte primaire par company). Les `vat_rates` n'ont **pas** de contrainte cross-row analogue → le `SELECT … FOR UPDATE` sur la row individuelle suffit pour les updates ; les créations sont protégées par `uq_vat_rates_company_rate_valid_from`. Ne PAS ajouter d'advisory lock.
- **`assert_onboarding_complete`** est une fn **privée** de `bank_accounts.rs` (non exportée). Soit la dupliquer dans `vat.rs`, soit la promouvoir `pub(crate)` dans un **module partagé** (`routes/common.rs` ou équiv.) — la 2e option est préférable (DRY) mais éviter une dépendance directe `vat.rs → bank_accounts.rs` (couplage douteux entre deux domaines) ; à trancher au dev (acceptable de dupliquer si le refactor déborde).
- **Audit** : `kesh_db::repositories::audit_log::insert_in_tx(&mut tx, NewAuditLogEntry::from_current_user(&current_user, "vat_rate.created", "vat_rate", id, Some(details)))`. `details` = JSON (`label`, `rate`, `valid_from`, `valid_to`, et pour update : ancien/nouveau `valid_to`/`active`). **Import requis dans `routes/vat.rs`** : `use crate::audit::AuditActor;` — `from_current_user` est une méthode du trait `AuditActor` (défini dans `kesh-api/src/audit.rs`), pas de `NewAuditLogEntry` lui-même ; sans cet import, la compilation échoue (« no method named `from_current_user` »). Voir `bank_accounts.rs:27`.
- **rust_decimal** : `rust_decimal::Decimal`, sérialisé JSON en string (feature `serde-str`), DB `DECIMAL(5,2)`. Validation scale via `routes/limits.rs::scale_within(value, 2)` (VAT = scale 2, pas 4). **Jamais de f64.**
- **Frontend** : feature `frontend/src/lib/features/vat-rates/` déjà présente (read) ; l'étendre avec les appels mutate (calquer `bank-accounts` feature/page).

### Sémantique des dates (déjà figée en DB)

`valid_from` **inclusif**, `valid_to` **exclusif** (`chk_vat_rates_dates: valid_to IS NULL OR valid_to > valid_from`). `list_active_at_date` : `valid_from <= date AND (valid_to IS NULL OR date < valid_to) AND active = TRUE`. Attention aux tests de bornes.

### Décision de design déférée à 11-2 — discriminant de catégorie TVA

Le schéma `vat_rates` n'a **pas de colonne `category`** : les 4 taux suisses (normal/réduit/spécial/exonéré) se distinguent uniquement par leur `label` (clé i18n pour les seedés). Tant que tous partagent `valid_from 2024-01-01, valid_to NULL`, ils coexistent — il n'existe donc PAS de notion déterministe de « **le** taux applicable au 2024-06-01 » sans préciser la catégorie. 11-1 livre donc une requête **multi-taux** (`list_active_at_date -> Vec`) et la gestion temporelle (création/clôture par dates). La **sélection d'un taux unique par catégorie** (nécessaire au calcul TVA sur une ligne de facture) relève de **Story 11-2**, qui devra trancher : (a) ajouter une colonne `category` enum à `vat_rates`, ou (b) sélectionner par `(rate, date)` puisque la ligne porte déjà son taux. **À décider au kickoff 11-2** (touche le modèle de données → arbitrage Project Lead). 11-1 ne préjuge pas de ce choix.

### RBAC

FR54 dit « l'**administrateur** peut configurer ». Guard = rôle Administrateur sur les mutations. Vérifier le helper RBAC exact dans `crates/kesh-api/src/lib.rs` (ex. `require_admin_role` ; cf. `require_comptable_role` pour les clés API). Si une décision « Comptable+ » est préférée, la trancher en spec validate.

### Pièges connus

- **Non-régression GET** : des consommateurs existants (facturation, produits, `verify_vat_rates_against_db`) appellent `list_vat_rates` / `list_active_for_company`. Le `?history=true` doit être **opt-in** ; le défaut reste « actifs uniquement ».
- **Seed onboarding** : `seed_default_swiss_rates_in_tx` insère sans `version` explicite → la colonne `version DEFAULT 0` couvre ce cas (pas de modif du seed nécessaire, mais le vérifier).
- **Pas de hard-delete** : un taux a pu servir à des factures/écritures historiques → soft-delete (`active = FALSE`) uniquement, jamais `DELETE FROM`.
- **Idempotence migration** : `ADD COLUMN` simple — ajouter au tableau `docs/migrations-idempotence-audit.md` (sinon finding MEDIUM en code review, politique P5).
- **HTTP LAN** : pas d'API secure-context-only côté frontend (`crypto.randomUUID`/`subtle`/`clipboard`) — cf. `feedback_no_secure_context_apis_http_lan`, utiliser `$props.id()` pour les IDs DOM.
- **Dualité `label` clé-i18n vs texte libre** : les 4 taux **seedés** ont un `label` = clé Fluent (`product-vat-normal`, …) résolue côté frontend par `i18nMsg(label, fallback)` (`frontend/src/lib/shared/utils/i18n.svelte.ts:15` : `_messages[key] || fallback`). Un taux **créé par l'admin** (AC#1) porte un `label` **texte libre** (PAS une clé). Règle figée : le frontend résout via **`i18nMsg(label, label)`** (fallback = label brut) → les clés connues sont traduites, le texte libre s'affiche tel quel. **Documenter dans l'UI** qu'un label admin ne doit pas usurper une clé Fluent (`product-vat-*`). Tester les deux cas (clé seedée traduite + texte libre affiché brut).

### Règle de splitting

Cette story touche ~4 modules (`kesh-db`, `kesh-api`, `frontend`, `kesh-i18n`) — sous le seuil de 5. Si `bmad-create-story validate` boucle au-delà de 4 passes sans converger, splitter en 11-1a (backend migration+repo+routes) / 11-1b (frontend+i18n+E2E).

### Project Structure Notes

- Migration : `crates/kesh-db/migrations/` (nommage `YYYYMMDDXXXXXX_vat_rates_version.sql`).
- Backend : `crates/kesh-db/{entities,repositories}/vat_rate(s).{rs}`, `crates/kesh-api/src/routes/vat.rs`.
- Frontend : `frontend/src/lib/features/vat-rates/` + route page admin.
- i18n : `crates/kesh-i18n/locales/*/messages.ftl`.

### References

- [Source: epics.md#Epic 10 (=11) Story 10.1 — Configuration des taux TVA, FR54]
- [Source: crates/kesh-db/migrations/20260428000001_vat_rates.sql — schéma existant]
- [Source: crates/kesh-db/src/repositories/vat_rates.rs — fonctions existantes + seed]
- [Source: crates/kesh-api/src/routes/vat.rs — routes read-only existantes]
- [Source: crates/kesh-api/src/routes/bank_accounts.rs — pattern CRUD de référence (Story v014-1)]
- [Source: crates/kesh-api/src/routes/onboarding.rs — seed atomique finalize()]
- [Source: docs/migrations-idempotence-audit.md — politique P5]
- [Source: CLAUDE.md — Migration breaking policy, Batch API conventions, Test Locally First]

## Change Log

### Create-story (2026-06-13)

Spec créée (Opus 4.8) comme enrichissement de l'infra `vat_rates` existante (Story 7.2/KF-003). 12 ACs. Pattern de référence `bank_accounts`. Hors-scope = calcul+rapport (11-2).

### Validate Pass 1 (Sonnet 4.6, 2026-06-13)

0 CRITICAL, 2 HIGH, 3 MEDIUM, 1 LOW — tous vérifiés ground-truth, tous patchés :
- **F1 HIGH** — `list_all_by_company()` existe déjà (Story 9-2b, utilisée par `exports/global.rs`) → la spec aurait fait créer un doublon `list_all_for_company` → corrigé : réutiliser l'existante (note cadrage + T2.2 + T3.2 + AC#6).
- **F2 HIGH** — RBAC : `bank_accounts` = Comptable+, mais mutations TVA = Admin (FR54) → T3.4 explicite « enregistrer dans `admin_routes`, PAS `comptable_routes`, écart intentionnel ».
- **F3 MEDIUM** — note de cadrage omettait `list_all_by_company` → ajoutée.
- **F4 MEDIUM** — `from_current_user` = méthode du trait `AuditActor` → import `use crate::audit::AuditActor;` documenté.
- **F5 MEDIUM** — advisory lock non tranché → décidé NON requis (pas de contrainte cross-row type `is_primary`).
- **F6 LOW** — tri `?history=true` → précisé `valid_from DESC, rate ASC`.

Validé exact ground-truth : table/contraintes `vat_rates`, fonctions repo existantes, routes read-only, `require_admin_role` (rbac.rs:31), `scale_within` (limits.rs:31), seed `finalize()`. Prochaine : Pass 2 (Haiku).

### Validate Pass 2 (Haiku 4.5, 2026-06-13)

**0 CRITICAL/HIGH/MEDIUM, 3 LOW, 0 hallucination.** Critère d'arrêt (0 > LOW) atteint. Ground-truth re-vérifié exhaustivement (admin_routes l.136-172, `list_all_by_company` trie id ASC, `require_admin_role`, `AuditActor`, contraintes unicité/dates, feature frontend `vat-rates/` 4 fichiers, sémantique dates cohérente, pas de violation unicité par PUT valid_to). 3 LOW patchés : signatures repo complètes + `DbError::OptimisticLockConflict` (T2.2), type erreur conflit nommé (AC#2), tri history côté handler (T3.2). Prochaine : Pass 3 (Opus) — sécurité code feature.

### Validate Pass 3 (Opus 4.8, 2026-06-13) — NON convergé

**1 CRITICAL + 2 HIGH + 3 LOW** — enjeux de **2nd ordre / domaine comptable** ratés par Sonnet+Haiku (la passe Opus paie, comme en 11-0). Tous vérifiés ground-truth, tous patchés :
- **F-OPUS-1 CRITICAL** — `find_applicable_for_date -> Option` est de **cardinalité indéfinie** : les 4 taux suisses seedés matchent tous la même date (pas de colonne `category` ; vérifié migration). → renommé `list_active_at_date -> Vec` (validité temporelle déterministe) ; **sélection mono-taux par catégorie déférée à 11-2** (où la ligne porte sa catégorie) + note de design « ajouter colonne category OU sélectionner par (rate,date) » à trancher au kickoff 11-2 (touche le modèle de données → Project Lead). AC#4/AC#5 reformulés.
- **F-OPUS-2 HIGH** — AC#2 (patch Pass 2) mappait le conflit optimiste en `AppError::Validation` (400) + clé i18n inventée, alors que `bank_accounts` **propage `AppError::Database(e)` → HTTP 409 `OPTIMISTIC_LOCK_CONFLICT`** (errors.rs:1645, i18n `error-optimistic-lock`). Corrigé (annule la mauvaise direction du LOW Pass 2).
- **F-OPUS-3 HIGH** — dualité `label` clé-i18n (seedés) vs texte libre (admin-créés) non résolue → règle figée `i18nMsg(label, label)` + test 2 cas (Pièges + AC#9).
- **F-OPUS-4/5/6 LOW** — `version` ajouté à `VatRateResponse` (T3.1), module partagé pour `assert_onboarding_complete` (Dev Notes), note étanchéité scope.

NON convergé (1C+2H > LOW) → **Pass 4 requise** (Sonnet, contexte frais). *(Multi-tenant scoping + migration version/lignes existantes vérifiés sains par Opus — pas de finding.)*

### Validate Pass 4 (Sonnet 4.6, 2026-06-13) — NON convergé (renommage incomplet)

**0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW** — tous patchés. P4-1 (MEDIUM) : le renommage `find_applicable_for_date → list_active_at_date` (Pass 3) était **incomplet** — 4 occurrences résiduelles dans des sections normatives (Scope item 4 & 8, AC#11, Dev Notes sémantique dates) contredisant T2.2/T5.1/AC#4-5. Corrigées + terminologie bornes alignée. P4-2 LOW : Scope item 4 annoté `(AC#3)` → `(AC#5)`. P4-3 LOW : chemin `i18n.svelte.ts` complété. Mapping 409 et règle `i18nMsg` re-vérifiés exacts. NON convergé (1 MEDIUM) → **Pass 5 requise** (Haiku, contexte frais).

### Validate Pass 5 (Haiku 4.5, 2026-06-13) — NON convergé (1 dernier résidu)

**0 CRITICAL/HIGH, 1 MEDIUM, 0 LOW, 0 hallucination.** Vérification de convergence en 9 points tous OK sauf P5-M1 : une dernière occurrence **abrégée** `find_applicable` (ligne 77, T1.1, dans une parenthèse sur l'indexation) ratée par le grep Pass 4 (qui cherchait la forme complète). Corrigée → `list_active_at_date`. Grep final : 0 résidu hors Change Log. → Pass 6 (Sonnet) de confirmation.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
