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

1. **Migration** (un seul fichier) :
   - `ALTER TABLE vat_rates ADD COLUMN version INT NOT NULL DEFAULT 0` (verrou optimiste pour les updates).
   - `ALTER TABLE vat_rates ADD COLUMN category VARCHAR(32) NOT NULL DEFAULT 'custom'` — discriminant **stable et EXTENSIBLE** de catégorie TVA (clé métier, indépendante du `label` d'affichage — permet de suivre « le taux normal » au fil des années). **PAS de contrainte `CHECK IN (liste fermée)`** : les autorités peuvent introduire de **nouvelles catégories officielles** sans migration de schéma — une nouvelle catégorie = une nouvelle valeur de clé. Seule contrainte : `CHECK (CHAR_LENGTH(TRIM(category)) > 0)` (non vide). Clés **réservées bien connues** (reconnues par l'app pour l'affichage i18n) : `normal`, `reduced`, `special`, `exempt` ; `custom` = catégorie libre admin ; toute autre clé future (ex. une nouvelle catégorie AFC) est acceptée et affichée via son `label`/sa clé brute.
   - **Backfill** des 4 taux seedés depuis leur `label` : `UPDATE vat_rates SET category = CASE label WHEN 'product-vat-normal' THEN 'normal' WHEN 'product-vat-reduced' THEN 'reduced' WHEN 'product-vat-special' THEN 'special' WHEN 'product-vat-exempt' THEN 'exempt' ELSE category END;`
   - **Index** `idx_vat_rates_company_category_active (company_id, category, active)` pour `find_for_category_at_date`.
   - Ligne dans `docs/migrations-idempotence-audit.md` (politique P5). **Non-breaking** (`ADD COLUMN … DEFAULT` ignoré par les anciens binaires → pas de bump `kesh_version_min_required`). Le seed `seed_default_swiss_rates_in_tx` devra aussi être étendu pour poser `category` à la création des nouvelles companies (cf. T1.3).
2. **Repository** (`vat_rates.rs`) — nouvelles fonctions calquées sur `repositories/bank_accounts.rs` :
   - `pub async fn create_for_company(tx, company_id, new) -> Result<VatRate, DbError>`
   - `pub async fn update_for_company(tx, company_id, id, fields, version) -> Result<VatRate, DbError>` (verrou optimiste + `SELECT … FOR UPDATE` ; en cas de conflit de version `rows_affected() == 0` → `return Err(DbError::OptimisticLockConflict)`, exactement comme `bank_accounts.rs:381`). Le handler propage via `AppError::Database(e)` → HTTP 409 (cf. AC#2).
   - `pub async fn deactivate_for_company(tx, company_id, id, version) -> Result<VatRate, DbError>` (soft : `active = FALSE` , même verrou optimiste ; **jamais de hard-delete** d'un taux potentiellement référencé par des écritures/factures historiques — audit-trail).
   - (signatures calquées sur `repositories/bank_accounts.rs` ; ne retourner que l'état final `VatRate` suffit — pas besoin du tuple `(avant, après)` de bank_accounts.)
   - **Vue historique** : réutiliser **`list_all_by_company()` existante** (NE PAS créer `list_all_for_company` — doublon de la fonction déjà utilisée par `exports/global.rs`).
   - `list_active_at_date(pool, company_id, date) -> Vec<VatRate>` — **tous** les taux en vigueur à une date (vue d'ensemble de la grille) : `valid_from <= date AND (valid_to IS NULL OR date < valid_to) AND active`, scopé `company_id`. **Sémantique** : `valid_from` inclusif, `valid_to` exclusif. Retourne un `Vec` (≤ une ligne par catégorie active).
   - `find_for_category_at_date(pool, company_id, category, date) -> Result<Option<VatRate>, DbError>` — **le** taux d'une catégorie donnée à une date : même prédicat + `category = ?`. **Déterministe** (au plus une ligne) grâce à l'invariant de non-chevauchement (ci-dessous). C'est la fonction que 11-2 consommera pour calculer la TVA d'une ligne (catégorie connue).
   - **Invariant de non-chevauchement par catégorie** : pour une `(company_id, category)` donnée, deux taux **actifs** ne peuvent PAS avoir des périodes `[valid_from, valid_to)` qui se chevauchent. MariaDB n'a pas de contrainte d'exclusion de plage → la validation se fait **en transaction sous advisory lock** (cf. ci-dessous) au moment du `create`/`update` : rejeter si un autre taux actif de la même catégorie chevauche la plage demandée.
   - **Advisory lock company REQUIS** (révision : la décision « pas d'advisory lock » de la passe 1 est annulée par l'ajout de `category`) : `kesh_db::repositories::bank_accounts::acquire_company_sentinel_lock(&mut tx, company_id)` (déjà `pub`, cf. Dev Notes) au début de chaque mutation, AVANT la vérification de chevauchement. Sans le lock, deux insertions concurrentes pourraient créer deux taux « normal » chevauchants.
3. **Routes API** (`vat.rs`) — handlers calqués sur `bank_accounts.rs` :
   - `POST   /api/v1/vat-rates` (création) → `vat_rate.created`
   - `PUT    /api/v1/vat-rates/{id}` (modification label / `valid_to` / `active`) → `vat_rate.updated`
   - `DELETE /api/v1/vat-rates/{id}` (désactivation soft) → `vat_rate.deactivated`
   - `GET    /api/v1/vat-rates?history=true` → liste complète (actifs + expirés) ; sans le param = comportement actuel (`active` seulement) **préservé** (non-régression des consommateurs existants : facturation, produits).
   - Guards : **rôle Administrateur** (FR54 « l'administrateur peut configurer ») + `assert_onboarding_complete`. Audit dans la même `tx` que la mutation.
4. **Changement de taux d'une catégorie au fil des années** (AC#5) — modélisé sans mutation destructive. **Ordre obligatoire : clôturer l'ancien D'ABORD, créer le nouveau ENSUITE** (sinon l'ancien `valid_to=NULL` chevauche le nouveau → rejet par l'invariant 4bis). Pour passer le **taux normal** de 7.7 % à 8.1 % au 2024-01-01 : (1) `PUT` l'ancien `normal` 7.70 → `valid_to = 2024-01-01` ; (2) `POST` un nouveau `normal` 8.10 `valid_from = 2024-01-01`. Comme `valid_to` est **exclusif**, les plages `[…, 2024-01-01)` et `[2024-01-01, …)` sont **adjacentes, non chevauchantes**. `find_for_category_at_date('normal', date)` renvoie alors 7.7 % avant 2024 et 8.1 % dès 2024. Le champ `rate` d'un taux existant n'est **pas** modifiable (valeur figée). L'action UI « changer le taux » exécute clôture-puis-création (idéalement dans une seule transaction atomique).
5. **Frontend** — page admin de gestion des taux TVA (`Paramètres` → `TVA`), calquée sur l'UI CRUD `bank_accounts` : taux **groupés par catégorie** (normal/réduit/spécial/exonéré/personnalisé), avec pour chaque catégorie le taux courant + l'historique (actifs/expirés). Formulaire création (choix de la **catégorie** dans l'enum + `rate` + `valid_from` + `valid_to` optionnel + `label` d'affichage optionnel), action « changer le taux » (crée le nouveau + clôt l'ancien), désactivation avec confirmation. Étendre la feature `vat-rates/`.
6. **i18n** — clés Fluent pour l'UI admin (titres, colonnes, **libellés de catégorie** `vat-category-{normal,reduced,special,exempt,custom}`, messages d'erreur, boutons) dans `fr-CH` (+ stubs DE/IT/EN selon `lint-i18n-ownership`).
7. **Validation** (per-requête, `AppError` ciblé) : `category` ∈ enum, `rate` 0..100 + `scale ≤ 2`, `valid_to > valid_from`, unicité `(company_id, rate, valid_from)` (déjà en DB), **non-chevauchement de validité au sein de la même catégorie active** (rejet si la plage demandée chevauche un autre taux actif de la même catégorie — vérifié sous advisory lock). `label` d'affichage optionnel (peut être vide → l'UI affiche le libellé de catégorie).
8. **Tests** : repository (CRUD, verrou optimiste, `list_active_at_date` aux bornes — veille de `valid_from` / jour de `valid_from` / jour de `valid_to`, taux inactif exclu), routes (RBAC admin, validation, audit émis, history vs active), E2E Playwright (créer un taux, clore l'ancien, vérifier l'historique).

### Hors scope (→ Story 11-2)

- **Calcul de la TVA** sur les lignes de facture/écriture, **arrondi commercial** au centime (FR55), **rapport TVA par période** AFC (FR56), export PDF/CSV du décompte. Tout cela est la Story 11-2.
- **FK `products.vat_rate → vat_rates`** : la non-liaison est un choix v0.1 intentionnel (validation asynchrone `verify_vat_rates_against_db`). Ne pas introduire la FK ici.
- **Helper d'arrondi commercial** (banker's vs commercial) : appartient à 11-2.

## Acceptance Criteria

1. **(FR54)** Un administrateur peut **créer** un taux TVA : `POST /api/v1/vat-rates` avec `category` (clé non vide ; clés réservées connues normal/reduced/special/exempt/custom, **mais valeurs nouvelles acceptées** — modèle extensible), `rate`, `valid_from`, `valid_to` (optionnel), `label` d'affichage (optionnel). Validation : `category` non vide (PAS de rejet sur « pas dans une liste fermée » — anticipe les nouvelles catégories officielles), `rate` 0..100 scale ≤ 2, `valid_to > valid_from`, unicité `(company_id, rate, valid_from)`, **non-chevauchement dans la catégorie** (AC#4bis). Événement audit `vat_rate.created` (détails incluant `category`) dans la même transaction, sous advisory lock company.
2. Un administrateur peut **modifier** un taux existant (`PUT /api/v1/vat-rates/{id}` : `label`, `valid_to`, `active`) avec **verrou optimiste** (`version`) — un conflit de version remonte `DbError::OptimisticLockConflict` (repo) ; le handler **propage `AppError::Database(e)`** (exactement comme `bank_accounts.rs:544/671`), rendu **HTTP 409 `OPTIMISTIC_LOCK_CONFLICT`** par le mapper central (`errors.rs:1645`, i18n existante `error-optimistic-lock`). **NE PAS** créer de clé i18n `vat_rate.conflict_version` ni re-mapper en `AppError::Validation` (400) — ce serait un mauvais code HTTP + une divergence du pattern. Le champ `rate` n'est **pas** modifiable (un taux a une valeur figée ; changer de taux = créer un nouveau). Audit `vat_rate.updated`.
3. **(AC historique)** Un administrateur peut **désactiver** un taux (`DELETE` soft → `active = FALSE`) sans suppression physique (préservation audit-trail). Audit `vat_rate.deactivated`. Un taux désactivé n'est plus proposé à la saisie mais reste visible dans l'historique et applicable aux opérations passées via les dates.
4. **(AC sélection temporelle par catégorie)** `find_for_category_at_date(company, category, date)` retourne **le** taux (déterministe, au plus un) de cette catégorie dont `valid_from <= date < valid_to` (ou `valid_to IS NULL`) et `active`. `list_active_at_date(company, date)` retourne **tous** les taux en vigueur (≤ un par catégorie). Testé aux bornes : la veille de `valid_from` → absent ; le jour de `valid_from` → présent ; le jour de `valid_to` → absent (exclusif).
4bis. **(AC non-chevauchement par catégorie)** La création/modification d'un taux est **rejetée** (erreur de validation per-requête) si elle introduit un chevauchement de période `[valid_from, valid_to)` avec un autre taux **actif de la même catégorie** (même `company_id`, en excluant la ligne en cours d'`UPDATE`). Deux plages **adjacentes** (`valid_to_A == valid_from_B`) ne se chevauchent **pas** (`valid_to` exclusif). Prédicat avec gestion `NULL` (=+∞) : cf. Dev Notes §Détection de chevauchement. Vérifié en transaction sous advisory lock company. Garantit le déterminisme de `find_for_category_at_date`.
5. **(AC changement de taux 7.7→8.1, catégorie normal)** Scénario : taux `category='normal'` 7.70 (`valid_from = 2023-…, valid_to = NULL`) ; l'admin **clôt d'abord** le 7.70 (`PUT valid_to = 2024-01-01`) **puis crée** un `normal` 8.10 (`valid_from = 2024-01-01`) — ordre imposé par l'invariant 4bis (l'inverse serait rejeté pour chevauchement). Vérification via `find_for_category_at_date('normal', date)` : au **2023-12-31** → 7.70 ; au **2024-01-01** → 8.10. Les deux coexistent dans l'historique (`?history=true`). Démontre que l'ancien taux reste applicable aux opérations antérieures, et que la **continuité par catégorie** est préservée année après année.
6. **(AC liste + historique)** `GET /api/v1/vat-rates` (sans param) renvoie les taux **actifs** (comportement actuel **inchangé** — non-régression facturation/produits). `GET /api/v1/vat-rates?history=true` renvoie la **liste complète** (actifs + inactifs/expirés) via `list_all_by_company`, triée **`valid_from DESC, rate ASC`** côté handler (la fonction repo trie `id ASC` ; ré-ordonner pour l'affichage admin), pour la vue admin.
7. **(RBAC)** Les mutations (POST/PUT/DELETE) sont réservées au **rôle Administrateur** (FR54) et requièrent l'onboarding complété (`assert_onboarding_complete`). Un rôle insuffisant → 403.
8. **(Migration)** Colonnes `version INT NOT NULL DEFAULT 0` **et** `category VARCHAR(32) NOT NULL DEFAULT 'custom'` (CHECK non-vide, **PAS** de liste fermée → extensible) ajoutées à `vat_rates` ; backfill `category` des 4 taux seedés depuis leur `label` ; index `(company_id, category, active)`. Non-breaking (pas de bump `kesh_version_min_required`). Ligne ajoutée à `docs/migrations-idempotence-audit.md` avec verdict + justification.
9. **(Frontend)** Page admin de gestion des taux TVA : taux **groupés par catégorie** (taux courant + historique actif/expiré par catégorie), création (choix catégorie + `rate` + dates + `label` optionnel), action « changer le taux » (crée + clôt), désactivation avec confirmation. Calquée sur l'UI CRUD `bank_accounts`. **Affichage** : la catégorie connue est affichée via sa clé i18n `vat-category-{key}` ; le `label` (s'il est renseigné) est affiché via `i18nMsg(label, label)` (clé seedée traduite, texte libre affiché brut) ; une catégorie inconnue/future s'affiche via sa clé brute ou son label. La réponse expose `version` (verrou optimiste) et `category`. États de chargement/erreur gérés. Pas d'API secure-context-only (déploiement HTTP LAN — cf. `feedback_no_secure_context_apis_http_lan`).
10. **(i18n)** Toutes les chaînes UI nouvelles ont des clés Fluent `fr-CH` + stubs DE/IT/EN ; `npm run lint-i18n-ownership` vert.
11. **(Tests)** Repository (CRUD, verrou optimiste, `list_active_at_date` aux bornes, taux inactif exclu), routes (RBAC, validation, audit, history vs active), E2E (créer/clore/historique). `cargo test --workspace` + `npm run test:unit` + build verts.
12. **(Préservation + seed)** Le seed onboarding (`seed_default_swiss_rates_in_tx` / `seed_default_swiss_rates`) est **étendu** pour poser `category` (`normal`/`reduced`/`special`/`exempt`) sur les 4 taux suisses créés à chaque nouvelle company (en plus de `version = 0` par défaut). Les consommateurs read-only existants (facturation, produits, `verify_vat_rates_against_db`, export global `list_all_by_company`) ne sont pas régressés (champs additifs).

## Tasks / Subtasks

- [ ] **T1 — Migration (`version` + `category`) + seed + audit idempotence** (AC #8, #12)
  - [ ] T1.1 `crates/kesh-db/migrations/YYYYMMDDXXXXXX_vat_rates_crud.sql` : `ADD COLUMN version INT NOT NULL DEFAULT 0` ; `ADD COLUMN category VARCHAR(32) NOT NULL DEFAULT 'custom'` + `CHECK (CHAR_LENGTH(TRIM(category)) > 0)` (PAS de liste fermée → extensible) ; backfill `category` des 4 taux seedés depuis `label` (CASE) ; `CREATE INDEX idx_vat_rates_company_category_active (company_id, category, active)` (**conserver** l'index existant `idx_vat_rates_company_active (company_id, active)` — il sert `list_active_for_company` sans filtre catégorie et n'est pas couvert en préfixe par le nouvel index ordonné).
  - [ ] T1.2 Ajouter la ligne dans `docs/migrations-idempotence-audit.md` (verdict + justification ; `ADD COLUMN` non-breaking → pas de bump `kesh_version_min_required`).
  - [ ] T1.3 Étendre le seed (`seed_default_swiss_rates` / `seed_default_swiss_rates_in_tx`, `repositories/vat_rates.rs`) pour poser `category` (`normal`/`reduced`/`special`/`exempt`) sur les 4 taux à la création des nouvelles companies.
- [ ] **T2 — Entité + repository CRUD** (AC #1-6, #4bis, #12)
  - [ ] T2.1 Étendre `entities/vat_rate.rs` : champs `version` (i32) et `category` (clé ; type Rust = enum `VatCategory { Normal, Reduced, Special, Exempt, Other(String) }` ou newtype string validé — **garder l'extensibilité** : impl avec fallback explicite `Other(String)` pour toute valeur inconnue, **PAS** un `#[derive(Deserialize)]` strict sans catch-all qui casserait sur une catégorie future ; en DB stocker/lire la clé brute `String` puis convertir à la frontière — `VatCategory` doit impl `sqlx::Type`/`Decode` ou être lu en `String`). Structs `UpdateVatRate` / payload create (avec `category`, `label` optionnel).
  - [ ] T2.1bis **⚠️ `VatRate` est `#[derive(sqlx::FromRow)]` → mapping par NOM de colonne.** Ajouter `category` (et `version`) au struct **OBLIGE** à ajouter ces colonnes à la SELECT-list des **3 fonctions existantes** : `list_all_by_company` (`vat_rates.rs:28`), `list_active_for_company` (`vat_rates.rs:48`), `find_active_by_rate` (`vat_rates.rs:70`) — sinon `query_as::<_, VatRate>` échoue **au runtime** (`ColumnNotFound`, invisible à `cargo build`, attrapé seulement par les tests DB). Idem toute nouvelle fonction.
  - [ ] T2.2 `repositories/vat_rates.rs` : `create_for_company`, `update_for_company` (optimistic lock + `FOR UPDATE`, calqué `bank_accounts`), `deactivate_for_company`, `list_active_at_date` (`Vec`), `find_for_category_at_date(company, category, date) -> Result<Option<VatRate>, DbError>`, et un helper de **détection de chevauchement** par catégorie (pour la validation 4bis). **Réutiliser `list_all_by_company()` existante** pour l'historique (pas de doublon). Préserver les fonctions existantes (`list_active_for_company`, `find_active_by_rate`, `list_all_by_company`, seed). Toutes les fonctions filtrent par `company_id` (IDOR).
- [ ] **T3 — Routes API + audit** (AC #1-3, #6, #7)
  - [ ] T3.1 `routes/vat.rs` : handlers POST/PUT/DELETE (guard admin + `assert_onboarding_complete`, **advisory lock company** en début de mutation), audit `vat_rate.{created,updated,deactivated}` (détails incluant `category`) dans la même `tx` (pattern `bank_accounts.rs`). Étendre `VatRateResponse` avec **`version`** et **`category`** (additifs — non-régression read-only ; `version` nécessaire au PUT optimiste).
  - [ ] T3.2 `GET /vat-rates?history=true` (liste complète via `list_all_by_company` existante) — param optionnel, défaut = comportement actuel inchangé (`list_active_for_company`). Avant de retourner en mode history, **ré-ordonner côté handler** par `valid_from DESC, rate ASC` (la fonction repo trie `id ASC`).
  - [ ] T3.3 Validation (`category` non vide, rate range/scale, dates, **non-chevauchement par catégorie** sous advisory lock) + mapping `AppError` i18n.
  - [ ] T3.4 Enregistrer POST/PUT/DELETE dans **`admin_routes`** de `crates/kesh-api/src/lib.rs` (~l.158-172, là où vivent `full_export`/`full_import`), **PAS** dans `comptable_routes` — écart **intentionnel** vs `bank_accounts` (qui est Comptable+) : les mutations TVA sont réservées Admin (FR54). Le `GET /vat-rates` reste dans les routes authentifiées existantes (inchangé).
- [ ] **T4 — Frontend page admin TVA** (AC #9, #10)
  - [ ] T4.1 Page + feature lib (calquée `bank_accounts` UI) : taux **groupés par catégorie** (courant + historique), création (choix catégorie + rate + dates + label optionnel), « changer le taux » (crée+clôt), édition `valid_to`/label, désactivation + confirmation. Affichage catégorie via `vat-category-{key}`, fallback clé brute pour catégorie inconnue.
  - [ ] T4.2 Guard de route (rôle admin), états chargement/erreur, pas d'API secure-context-only.
  - [ ] T4.3 i18n `fr-CH` + stubs DE/IT/EN, `lint-i18n-ownership` vert.
- [ ] **T5 — Tests** (AC #11)
  - [ ] T5.1 Repository : CRUD, verrou optimiste (conflit version → `DbError::OptimisticLockConflict`), `list_active_at_date` aux bornes (veille exclue / jour `valid_from` inclus / jour `valid_to` exclu) + cas multi-taux (4 catégories présentes au 2024-06-01), `find_for_category_at_date` déterministe (1 taux par catégorie/date ; scénario 7.7→8.1 catégorie normal), **rejet de chevauchement** dans une catégorie, catégorie inconnue/future acceptée (extensibilité), taux inactif exclu, `list_all_by_company` vs `list_active_for_company`.
  - [ ] T5.2 Routes : RBAC admin (403 si insuffisant), validation (catégorie, dates, chevauchement), audit émis (avec `category`), history vs active, non-régression GET sans param.
  - [ ] T5.3 E2E Playwright : créer un taux, clôturer l'ancien (`valid_to`), vérifier l'historique + que la saisie ne propose que les actifs.
- [ ] **T6 — Vérifs finales** (AC #11, #12)
  - [ ] T6.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (serial si touche kesh-db : `-j1 -- --test-threads=1`).
  - [ ] T6.2 `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build`.
  - [ ] T6.3 E2E (`npm run test:e2e`) — pré-requis MariaDB + seed + browsers (`PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`).

## Dev Notes

### Pattern de référence à suivre (NE PAS réinventer)

- **CRUD complet** : `crates/kesh-api/src/routes/bank_accounts.rs` + `crates/kesh-db/src/repositories/bank_accounts.rs` (Story v014-1). Reprendre : guard `assert_onboarding_complete`, validation, `update_for_company` avec verrou optimiste `version` + `SELECT … FOR UPDATE`, soft-delete (`active`), audit **dans la même transaction** que la mutation.
- **Advisory lock company — REQUIS** : avec l'ajout de `category` et son invariant de **non-chevauchement par catégorie** (4bis), `vat_rates` a désormais une contrainte cross-row (analogue à `is_primary` de `bank_accounts`). Chaque mutation acquiert le lock en début de transaction, AVANT la vérification de chevauchement — sans le lock, deux insertions concurrentes pourraient créer deux taux de même catégorie aux périodes chevauchantes. **`acquire_company_sentinel_lock` est DÉJÀ `pub`** dans `kesh_db::repositories::bank_accounts` (`bank_accounts.rs:588`) — helper générique company-scoped, déjà réutilisé hors-domaine (ex. `users.rs`). **L'appeler directement** : `kesh_db::repositories::bank_accounts::acquire_company_sentinel_lock(&mut tx, company_id)` — **rien à promouvoir/déplacer** (≠ `assert_onboarding_complete` qui, lui, est une fn *privée de la route* `bank_accounts.rs`). Alias neutre optionnel si le nommage gêne, non requis.

### Détection de chevauchement (prédicat canonique, F-CAT-3)

Deux taux actifs `(company_id, category)` chevauchent **ssi** : `new.valid_from < COALESCE(existing.valid_to, '9999-12-31') AND existing.valid_from < COALESCE(new.valid_to, '9999-12-31')` (en excluant `existing.id = new.id` sur un `UPDATE`). Le `COALESCE(valid_to, '9999-12-31')` gère `valid_to NULL` (=+∞) — sans lui, un test naïf `x < NULL` vaut NULL/faux en SQL et raterait le chevauchement (cas dominant : tous les taux seedés ont `valid_to NULL`). Test borne obligatoire (T5.1) : deux `normal` `valid_to NULL` → rejet ; deux plages adjacentes (`valid_to_A == valid_from_B`) → acceptées.
- **`assert_onboarding_complete`** est une fn **privée** de `bank_accounts.rs` (non exportée). Soit la dupliquer dans `vat.rs`, soit la promouvoir `pub(crate)` dans un **module partagé** (`routes/common.rs` ou équiv.) — la 2e option est préférable (DRY) mais éviter une dépendance directe `vat.rs → bank_accounts.rs` (couplage douteux entre deux domaines) ; à trancher au dev (acceptable de dupliquer si le refactor déborde).
- **Audit** : `kesh_db::repositories::audit_log::insert_in_tx(&mut tx, NewAuditLogEntry::from_current_user(&current_user, "vat_rate.created", "vat_rate", id, Some(details)))`. `details` = JSON (`label`, `rate`, `valid_from`, `valid_to`, et pour update : ancien/nouveau `valid_to`/`active`). **Import requis dans `routes/vat.rs`** : `use crate::audit::AuditActor;` — `from_current_user` est une méthode du trait `AuditActor` (défini dans `kesh-api/src/audit.rs`), pas de `NewAuditLogEntry` lui-même ; sans cet import, la compilation échoue (« no method named `from_current_user` »). Voir `bank_accounts.rs:27`.
- **rust_decimal** : `rust_decimal::Decimal`, sérialisé JSON en string (feature `serde-str`), DB `DECIMAL(5,2)`. Validation scale via `routes/limits.rs::scale_within(value, 2)` (VAT = scale 2, pas 4). **Jamais de f64.**
- **Frontend** : feature `frontend/src/lib/features/vat-rates/` déjà présente (read) ; l'étendre avec les appels mutate (calquer `bank-accounts` feature/page).

### Sémantique des dates (déjà figée en DB)

`valid_from` **inclusif**, `valid_to` **exclusif** (`chk_vat_rates_dates: valid_to IS NULL OR valid_to > valid_from`). `list_active_at_date` / `find_for_category_at_date` : `valid_from <= date AND (valid_to IS NULL OR date < valid_to) AND active = TRUE` (+ `category = ?` pour la seconde). Attention aux tests de bornes.

### Modèle de catégorie TVA — discriminant stable et EXTENSIBLE (décision Guy 2026-06-13)

Le `label` seul ne suffit pas à suivre « le taux normal » au fil des années (un admin pourrait saisir un label libre cassant la continuité). On introduit donc une colonne **`category`** = clé métier stable, **distincte du `label` d'affichage** :

- **Clés réservées connues** (reconnues pour l'affichage i18n `vat-category-{key}`) : `normal`, `reduced`, `special`, `exempt`. `custom` = libre admin.
- **EXTENSIBLE** (décision Guy) : les autorités peuvent introduire de **nouvelles catégories officielles** → la colonne est un `VARCHAR` SANS contrainte `CHECK IN (liste fermée)`. Une nouvelle catégorie = une nouvelle valeur de clé, **sans migration de schéma**. Côté Rust, modéliser avec un variant ouvert (`VatCategory::Other(String)`) ou un newtype string validé — une catégorie inconnue ne doit PAS faire échouer la désérialisation ni l'affichage (fallback : clé brute ou `label`). Une catégorie officielle future pourra être ajoutée par un simple seed/UPDATE.
- **Continuité temporelle par catégorie** : `find_for_category_at_date('normal', date)` suit la série (7.7 % avant 2024 → 8.1 % après), garantie déterministe par l'invariant de non-chevauchement (4bis).
- **Évolution possible (hors 11-1)** : si la gestion des catégories devient riche (libellés multilingues officiels, métadonnées), une table de référence `vat_categories` pourra être introduite plus tard ; 11-1 reste sur la clé `VARCHAR` (suffisant et extensible).

Cette colonne **débloque la Story 11-2** (calcul TVA : `find_for_category_at_date(catégorie de la ligne, date facture)`).

### RBAC

FR54 dit « l'**administrateur** peut configurer » → **tranché : rôle Administrateur** (`require_admin_role`, `middleware/rbac.rs:31`), mutations dans `admin_routes` (cf. T3.4). Écart intentionnel vs `bank_accounts` (Comptable+).

### Pièges connus

- **Non-régression GET** : des consommateurs existants (facturation, produits, `verify_vat_rates_against_db`) appellent `list_vat_rates` / `list_active_for_company`. Le `?history=true` doit être **opt-in** ; le défaut reste « actifs uniquement ».
- **Seed onboarding** : `version DEFAULT 0` couvre le cas version, MAIS le seed **doit être étendu** pour poser `category` explicitement (`normal`/`reduced`/`special`/`exempt`) — sinon les 4 taux des nouvelles companies tomberaient sur `category = 'custom'` (défaut) et la continuité par catégorie serait cassée. Cf. T1.3.
- **`VatRate` est `FromRow` par nom** : tout `SELECT` mappé sur `VatRate` (existant ou nouveau) DOIT lister **toutes** les colonnes du struct, `category` et `version` compris — sinon `ColumnNotFound` au runtime (cf. T2.1bis). Piège invisible à `cargo build`.
- **Pas de hard-delete** : un taux a pu servir à des factures/écritures historiques → soft-delete (`active = FALSE`) uniquement, jamais `DELETE FROM`.
- **Idempotence migration** : `ADD COLUMN` + backfill `UPDATE` + `CREATE INDEX` — ajouter au tableau `docs/migrations-idempotence-audit.md` (sinon finding MEDIUM en code review, politique P5). Le backfill `UPDATE … CASE` est idempotent (re-jouable sans effet de bord ; les valeurs déjà posées restent).
- **HTTP LAN** : pas d'API secure-context-only côté frontend (`crypto.randomUUID`/`subtle`/`clipboard`) — cf. `feedback_no_secure_context_apis_http_lan`, utiliser `$props.id()` pour les IDs DOM.
- **Affichage `category` vs `label`** : la **catégorie** (clé métier) pilote l'affichage principal via `vat-category-{key}` (`vat-category-normal`, …) ; le `label` devient un libellé d'affichage **optionnel** (les 4 taux seedés gardent leur `label` clé Fluent `product-vat-*` pour rétro-compat). Résolution frontend : afficher la catégorie connue via sa clé i18n ; si `label` renseigné, le résoudre via `i18nMsg(label, label)` (clé connue traduite, texte libre brut) ; **catégorie inconnue/future** → fallback sur la clé brute (extensibilité). Tester : catégorie connue traduite, catégorie future affichée en clé brute sans crash.

### Règle de splitting

Cette story touche ~4 modules (`kesh-db`, `kesh-api`, `frontend`, `kesh-i18n`) — sous le seuil de 5. **Scope élargi** par l'ajout de `category` (migration version+category+backfill+index+seed, repo avec non-chevauchement + advisory lock + `find_for_category_at_date`, frontend groupé par catégorie). Si l'implémentation déborde ou si une passe review re-boucle, splitter en **11-1a** (backend : migration + repo + routes + modèle catégorie) / **11-1b** (frontend groupé + i18n + E2E). Le découpage backend/frontend est net (l'API stabilise le contrat avant l'UI).

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

### Validate Pass 6 (Sonnet 4.6, 2026-06-13) — ✅ CONVERGÉ

**0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.** Confirmation finale : cohérence interne (12 ACs sans trou, mapping AC↔tâche complet), 0 résidu `find_applicable` hors Change Log, mapping 409 uniforme, `list_active_at_date` partout en `Vec`, et tous les claims ground-truth re-vérifiés exacts (`require_admin_role` rbac.rs:31, `list_all_by_company` vat_rates.rs:23, `VatRateResponse` sans `version`, `OptimisticLockConflict → 409` errors.rs:1645, `scale_within` limits.rs:31, `admin_routes` lib.rs:136-172, `i18nMsg`).

**Trend du cycle** : Pass 1 (Sonnet) 6 [2H/3M/1L] → Pass 2 (Haiku) 3 [3L] → Pass 3 (**Opus**) 6 [**1C**/2H/3L] → Pass 4 (Sonnet) 3 [1M/2L] → Pass 5 (Haiku) 1 [1M] → Pass 6 (Sonnet) **0**. Cycle Sonnet→Haiku→Opus→Sonnet→Haiku→Sonnet (LLM différent par passe). **Opus P3 a capté le CRITICAL de domaine** (cardinalité `find_applicable_for_date` / absence de catégorie) raté par Sonnet+Haiku. 0 hallucination Haiku (grep ground-truth systématique). Décision de design « discriminant catégorie » déférée à 11-2 (touche le modèle de données).

Status : **`ready-for-dev`**. Prochaine étape : `bmad-dev-story 11-1`.

### Re-scope — ajout du discriminant `category` (Guy, 2026-06-13, post-convergence)

Après convergence du cycle validate, Guy a tranché 2 points de modèle de données (en réponse au CRITICAL F-OPUS-1 + au constat « la TVA change d'année en année » + « les autorités peuvent créer de nouvelles catégories ») :

1. **Ajouter une colonne `category`** à `vat_rates` **dans 11-1** (pas déférée à 11-2) — discriminant métier stable, distinct du `label` d'affichage, permettant de suivre « le taux normal » comme une série temporelle (7.7 % → 8.1 %).
2. **Modèle extensible** : `category` = `VARCHAR(32)` **sans** `CHECK IN (liste fermée)` → les autorités peuvent introduire de nouvelles catégories officielles sans migration de schéma. Clés réservées connues (normal/reduced/special/exempt/custom) + ouverture (`VatCategory::Other(String)`).

**Impacts sur la spec** (déjà appliqués) : migration (T1) ajoute `category` + backfill + index + extension seed (T1.3) ; repo (T2) ajoute `find_for_category_at_date` (déterministe) + détection de chevauchement ; nouvel **AC#4bis** (non-chevauchement par catégorie) ; **advisory lock company REDEVIENT requis** (contrainte cross-row réintroduite — annule la décision « pas de lock » de Pass 1) ; frontend groupé par catégorie ; `VatRateResponse` expose `category` + `version`. La note « design déféré à 11-2 » est **supprimée** (résolu ici). Scope élargi → fallback split 11-1a/11-1b documenté.

⚠️ **Ce re-scope rouvre la validation** : une passe validate ciblée sur le delta `category` est requise avant dev (les 6 passes précédentes validaient le modèle sans catégorie).

### Validate delta-category Pass A (Opus 4.8, 2026-06-13) — NON convergé

Passe ciblée sur le delta `category` : **2 HIGH + 2 MEDIUM + 2 LOW**, tous nouveaux (interactions introduites par `category`), tous ground-truth, tous patchés :
- **F-CAT-1 HIGH** — `VatRate` est `#[derive(FromRow)]` par nom → ajouter `category`/`version` au struct **casse au runtime** les 3 SELECT existants (l.28/48/70) qui ne les listent pas (`ColumnNotFound`, invisible à `cargo build`). → T2.1bis + Piège.
- **F-CAT-2 HIGH** — contradiction d'ordre : AC#5 « crée puis clôt » est **infaisable** (l'ancien `valid_to=NULL` chevauche → rejet 4bis). → ordre inversé **clôturer-puis-créer** (plages adjacentes non chevauchantes, `valid_to` exclusif) dans Scope item 4 + AC#5 + AC#4bis.
- **F-CAT-3 MEDIUM** — prédicat de chevauchement avec `valid_to NULL` (=+∞) non spécifié (`x < NULL` = faux SQL) → prédicat canonique `COALESCE(valid_to,'9999-12-31')` figé en Dev Notes.
- **F-CAT-4 MEDIUM** — `acquire_company_sentinel_lock` est **déjà `pub`** dans `kesh-db` repositories (pas la couche route) et déjà partagé → corrigé « appeler directement, rien à promouvoir » (l'avertissement couplage ne visait que `assert_onboarding_complete`).
- **F-CAT-5/6 LOW** — mécanisme serde `VatCategory::Other` (catch-all), conserver l'index existant.

Vérifs positives : backfill `CASE label` exact (labels réels `product-vat-*`), seed T1.3 couvert, AC↔Task OK, non-régression backup `.keshbackup` (export dynamique INFORMATION_SCHEMA), modèle extensible sans contradiction. NON convergé → **Pass B requise** (Sonnet, contexte frais).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
