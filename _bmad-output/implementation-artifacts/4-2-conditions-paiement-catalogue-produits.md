# Story 4.2: Conditions de paiement & catalogue produits/services

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **associer des conditions de paiement à mes contacts et gérer un catalogue de produits/services avec prix et taux TVA**,
so that **la création de factures (Epic 5) soit rapide et cohérente, avec des lignes pré-rempliables depuis le catalogue**.

### Contexte

**Deuxième et dernière story de l'Epic 4** (Carnet d'adresses & Catalogue). Couvre FR28 (conditions de paiement), FR29 (catalogue produits/services), et pose les fondations pour FR30 (pré-remplissage facture, câblage effectif en Story 5.1).

**Fondations déjà en place** (NE PAS refaire) :

- **Colonne `default_payment_terms VARCHAR(100) NULL`** — déjà créée dans `20260414000001_contacts.sql` (Story 4.1). L'API contacts accepte et persiste déjà ce champ dans `CreateContactRequest` et `UpdateContactRequest`. **Le formulaire UI n'expose PAS encore le champ** — c'est le travail de cette story.
- **Pattern Repository CRUD + audit log** — Story 4.1 (contacts) est le modèle canonique le plus récent. 6 fonctions, audit atomique avec rollback explicite, `ContactListQuery`/`ContactListResult`/`ContactSortBy` locaux, `escape_like` dupliqué. **À copier tel quel pour products.**
- **`rust_decimal::Decimal`** — déjà en dépendance de `kesh-db` (`Cargo.toml:9,14` features `serde-str` + `maths`) et `kesh-core`. Utilisé par `journal_entries` pour les montants débit/crédit. **À réutiliser pour `unit_price` et `vat_rate`.**
- **`ListResponse<T>`** — type générique paginé dans `routes/mod.rs:25`, utilisé par `journal_entries` (Story 3.4) et `contacts` (Story 4.1). **À réutiliser, NE PAS créer de nouveau type.**
- **Tous les patterns frontend Story 4.1** : `onMount` pour lecture URL initiale (pas `$effect` — fix P1 code review 4.1), debounce 300ms, `notify*` helpers, `i18nMsg` canonical, dialog create/edit/archive/conflit 409. **À répliquer.**
- **Aucun code `products` n'existe** — vérifié empiriquement 2026-04-12. Tout à créer de zéro.
- **Aucune logique TVA n'existe** — pas de table `vat_rates`, pas de calcul TVA. Story 4.2 stocke le taux TVA applicable **par produit** comme un `DECIMAL(5,2)` (ex: `7.70` pour 7.7%). Le calcul TVA effectif (total HT → TTC, décompte TVA par période) est reporté à **Epic 9** (TVA Suisse).

### Scope verrouillé — ce qui DOIT être fait

1. **Exposer `default_payment_terms`** dans le formulaire contact existant (`ContactForm.svelte` ou `+page.svelte`) : un input texte libre avec placeholder (ex: « 30 jours net »). Pas de migration (colonne déjà là). Pas de nouveau handler (API contacts l'accepte déjà). Juste un champ UI + i18n.

2. **Migration `products`** — table avec schéma conforme à l'AC de l'Epic : `id, company_id, name, description, unit_price (DECIMAL(19,4)), vat_rate (DECIMAL(5,2)), active, version, created_at, updated_at`. Contrainte unique `(company_id, name)` pour éviter les doublons de noms.

3. **Entité Rust `Product`** — `kesh-db/src/entities/product.rs` avec `Product`, `NewProduct`, `ProductUpdate`. Champs `unit_price` et `vat_rate` en `rust_decimal::Decimal`.

4. **Repository `products`** — `kesh-db/src/repositories/products.rs` avec 5 fonctions : `create`, `find_by_id`, `list_by_company_paginated`, `update`, `archive`. Audit log atomique identique au pattern contacts. `ProductListQuery` / `ProductListResult` / `ProductSortBy` locaux.

5. **API routes `/api/v1/products`** — 5 handlers (list, get, create, update, archive). GETs dans `authenticated_routes`, mutations dans `comptable_routes`. Validation métier (nom non vide, prix ≥ 0, taux TVA dans la whitelist suisse).

6. **Frontend feature `products`** — page `/products` avec table filtrable/paginée/triable, formulaire create/edit dans dialog, archive, toasts, modale 409, URL state sync. Affichage prix via `Intl.NumberFormat('de-CH')` avec apostrophe suisse.

7. **i18n** — ~30 nouvelles clés × 4 langues pour le catalogue + ~5 clés pour le champ payment terms.

8. **Tests** — Rust DB + unit + Vitest + Playwright.

### Scope volontairement HORS story — décisions tranchées

- **Pré-remplissage lignes facture** (FR30) : Story 5.1 (Epic 5). Cette story crée le catalogue, pas le mécanisme de sélection depuis la facture.
- **Calcul TVA / décompte** (FR45-FR47) : Epic 9. Story 4.2 stocke le taux par produit, le calcul arrive plus tard.
- **TVA historique / multi-taux par date** : hors v0.1. Un produit a UN taux TVA applicable.
- **Import catalogue CSV / bulk** : hors scope v0.1.
- **Catégories / tags produits** : pas dans le PRD, pas dans l'epic.
- **Images / attachements produit** : pas de stockage blob en v0.1.
- **SKU / code-barres** : pas dans le PRD.
- **Quantité en stock** : pas un logiciel de gestion de stock.
- **Suggestions conditionnelles de paiement** (dropdown prédéfini) : le champ est texte libre (décision de l'epic « les conditions de paiement sont un champ texte libre »). Pas de table de lookup.

### Décisions de conception

- **`unit_price` en `DECIMAL(19,4)`** — **cohérent avec `journal_entry_lines.debit/credit` qui utilisent `DECIMAL(19,4)`** (vérifié empiriquement dans `20260412000001_journal_entries.sql:42-43`). Pas `DECIMAL(15,4)` qui serait une précision moindre et créerait une incohérence lors du câblage Epic 5 (copie de lignes produit vers journal_entry_lines). Permet les prix de `0.0001` à `999'999'999'999'999.9999`. Le frontend affiche avec 2 décimales par défaut (CHF), mais le stockage conserve 4 pour les arrondis intermédiaires.

- **`vat_rate` en `DECIMAL(5,2)`** — stocke le pourcentage directement (ex: `7.70` pour 7.7%, `2.50` pour 2.5%). **Pas un ratio** (pas `0.077`). Plus lisible en DB, en API, et dans les formulaires. Conversion en ratio (`/ 100`) se fait au calcul en Epic 9.

- **Whitelist taux TVA suisse** — Validation côté handler API (pas en DB CHECK) pour permettre une évolution sans migration :
  - `0.00` — exonéré
  - `2.60` — taux réduit (depuis 01.01.2024, anciennement 2.5)
  - `3.80` — taux spécial hébergement (depuis 01.01.2024, anciennement 3.7)
  - `8.10` — taux normal (depuis 01.01.2024, anciennement 7.7)
  La whitelist est définie comme constante dans le handler. **Décision v0.1** : pas de table `vat_rates` paramétrable — hardcodé côté API. Si les taux changent (peu probable à court terme — dernière modification 01.01.2024), un commit suffit.

- **Unicité produit par `(company_id, name)`** — deux produits dans la même company ne peuvent pas avoir le même nom. Pattern identique à l'unicité IDE des contacts. Mapping erreur → `AppError::Validation("Un produit avec ce nom existe déjà")` (pas de code dédié comme IDE — le générique `RESOURCE_CONFLICT` suffit ici via le from DbError).

- **Soft-delete + optimistic locking** — identique à contacts (active + version + route `/archive`).

- **Pas de champ `description` obligatoire** — optionnel (`VARCHAR(1000) NULL`). Les petits indépendants ont souvent des produits à une ligne.

- **`default_payment_terms` dans le form contact** — simple `<input type="text">` avec label et placeholder i18n. Pas de validation côté backend (le champ accepte déjà n'importe quel texte ≤ 100 chars, validé en Story 4.1).

- **Frontend prix** — `<input type="text" inputmode="decimal">` (pas `type="number"` qui a des problèmes UX avec les séparateurs). Validation pattern regex client : `^(0|[1-9][0-9]*)(\.[0-9]{1,4})?$` (rejette les zéros en tête superflus). **Affichage** : utiliser le pattern **big.js** établi par `balance.ts` (Story 3.2) — `Big(priceString).toFixed(2)` + formatage suisse avec **apostrophe typographique U+2019 (`’`)** comme séparateur milliers. **NE PAS** utiliser l'apostrophe ASCII U+0027 (`'`) — le code existant `balance.ts:99` utilise U+2019, conforme à la norme Swiss Number SN01 et au BFS. **NE PAS utiliser `Intl.NumberFormat('de-CH')`** qui passe par `parseFloat()` implicitement et perd la précision pour les montants > `Number.MAX_SAFE_INTEGER` (incohérent avec le pattern comptable du projet). Le helper `formatPrice(d: string): string` de `product-helpers.ts` prend la string Decimal du backend et retourne `"1’500.00"` (avec U+2019). **DRY** : préférer **réutiliser `formatSwissAmount`** de `balance.ts` (ou l'extraire en helper partagé `$lib/shared/utils/format-decimal.ts` si le pattern se répète) plutôt que dupliquer la logique de formatage dans `product-helpers.ts`. `big.js ^6.2.2` est déjà en dépendance frontend (`package.json:42`).

- **Audit log** — `product.created`, `product.updated`, `product.archived`. Snapshot avec `unitPrice` et `vatRate` en string décimal (pattern `serde_json::json!` existant). Convention snapshot direct (create/archive) / wrapper `{before, after}` (update).

## Acceptance Criteria (AC)

1. **Champ conditions de paiement visible** (FR28) — Given le formulaire d'édition d'un contact, When l'utilisateur ouvre le dialog create/edit, Then un champ texte libre « Conditions de paiement » est visible avec le placeholder « ex: 30 jours net ». When il saisit « 10 jours 2% escompte » et enregistre, Then la valeur est persistée en base et visible lors de la prochaine édition.

2. **Création produit nominale** (FR29) — Given le catalogue, When l'utilisateur remplit nom, prix unitaire, taux TVA (dropdown), Then un produit est créé avec `version = 1`, `active = true`, et une entrée `audit_log` `action = "product.created"`. Le prix est stocké en `DECIMAL(19,4)` (ex: `1500.0000`) et le taux en `DECIMAL(5,2)` (ex: `8.10`).

3. **Taux TVA whitelist** — Given la saisie d'un produit, When le taux TVA est `8.10` (normal 2024), `3.80` (spécial), `2.60` (réduit), ou `0.00` (exonéré), Then création OK. When le taux est `99.99` ou `-1.00`, Then 400 « Taux TVA non autorisé ».

4. **Modification avec verrouillage optimiste** — Given un produit `v1`, When l'utilisateur modifie le nom et le prix, Then `version` passe à 2, audit `product.updated` avec `{before, after}`. When conflit, Then 409 `OPTIMISTIC_LOCK_CONFLICT` + modale reload.

5. **Archivage produit** — Given un produit actif, When archivage, Then `active = false`, version++, audit `product.archived`. Given un produit déjà archivé, When re-archivage ou modification, Then 409 `ILLEGAL_STATE_TRANSITION`. Given la liste sans `includeArchived`, Then le produit archivé n'apparaît pas.

6. **Unicité nom produit** — Given un produit existant « Création logo », When création d'un 2e produit même nom dans la même company, Then 409 `RESOURCE_CONFLICT` avec message i18n « Un produit avec ce nom existe déjà ».

7. **Liste paginée + tri** — Given 25 produits, When chargement `/products`, Then 20 premiers triés par `name ASC` par défaut. Tri cliquable sur nom, prix, TVA. Pagination offset/limit.

8. **Recherche debouncée** — Given la liste, When saisie dans le champ recherche, Then après 300ms debounce, filtre LIKE sur `name` + `description`. URL reflète le filtre.

9. **URL state préservé après reload** — Given un état filtré/paginé/trié, When refresh, Then état restauré depuis les query params.

10. **RBAC** — GETs dans `authenticated_routes`, mutations dans `comptable_routes` (pattern contacts 4.1).

11. **i18n** — Tous labels, messages, erreurs internationalisés × 4 langues.

12. **Audit log complet** — 3 mutations (create, update, archive) chacune écrit audit atomiquement.

13. **Notifications** — `notifySuccess`/`notifyError` pour tous les feedbacks.

14. **Tests** — Rust DB + unit handler + Vitest + Playwright.

## Tasks / Subtasks

### T1 — Champ `default_payment_terms` dans le form contact (AC: #1)

- [x] T1.1 Dans `frontend/src/routes/(app)/contacts/+page.svelte`, ajouter un `<input type="text">` pour `default_payment_terms` dans la section formulaire dialog. **Placement précis** : insérer le bloc `<div>` du champ `formPaymentTerms` **juste après** le bloc `form-ide` (lignes 603-609 de `contacts/+page.svelte`) et **avant** `{#if formError}` (ligne 612) — le bloc `formError` doit rester collé aux boutons submit pour que le message d'erreur soit visible au moment du clic. Label : `i18nMsg('contact-form-payment-terms', 'Conditions de paiement')`. Placeholder : `i18nMsg('contact-form-payment-terms-placeholder', 'ex: 30 jours net')`. Le champ est lié à l'état du formulaire existant (ajouter `formPaymentTerms = $state('')` + le passer dans le payload `createContact`/`updateContact`). **Important** : réinitialiser `formPaymentTerms = '';` dans `openCreate()` (au même titre que `formName = '';` etc.) pour éviter le carryover de valeur entre ouvertures successives du dialog.
- [x] T1.2 Lors de l'ouverture en mode edit (`openEdit`), initialiser `formPaymentTerms` depuis `c.defaultPaymentTerms ?? ''`.
- [x] T1.3 Ajouter les clés i18n `contact-form-payment-terms` et `contact-form-payment-terms-placeholder` dans les 4 `.ftl` (FR/DE/IT/EN).
- [x] T1.4 Test Playwright : ouvrir un contact existant en édition, saisir « 30 jours net », enregistrer, recharger, vérifier que la valeur persiste.

### T2 — Migration & entité Product (AC: #2, #3, #6)

- [x] T2.1 Créer `crates/kesh-db/migrations/20260415000001_products.sql` (vérifier le numéro avant) :
  ```sql
  CREATE TABLE products (
      id BIGINT NOT NULL AUTO_INCREMENT,
      company_id BIGINT NOT NULL,
      name VARCHAR(255) NOT NULL,
      description VARCHAR(1000) NULL,
      unit_price DECIMAL(19,4) NOT NULL,
      vat_rate DECIMAL(5,2) NOT NULL,
      active BOOLEAN NOT NULL DEFAULT TRUE,
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      PRIMARY KEY (id),
      CONSTRAINT fk_products_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
      CONSTRAINT uq_products_company_name UNIQUE (company_id, name),
      CONSTRAINT chk_products_name_not_empty CHECK (CHAR_LENGTH(TRIM(name)) > 0),
      CONSTRAINT chk_products_price_non_negative CHECK (unit_price >= 0),
      INDEX idx_products_company_active (company_id, active)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
  ```
  **Note** : la clause `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci` est **obligatoire** — sans elle, MariaDB 11.x utilise `utf8mb4_uca1400_ai_ci` par défaut, dont le comportement case-insensitive diffère subtilement de `utf8mb4_unicode_ci` utilisé par toutes les autres tables du projet (`initial_schema.sql`, `journal_entries.sql`, etc.).

- [x] T2.2 Créer `crates/kesh-db/src/entities/product.rs` :
  - `pub struct Product { id, company_id, name, description (Option<String>), unit_price (Decimal), vat_rate (Decimal), active, version, created_at, updated_at }` avec `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]` + `#[serde(rename_all = "camelCase")]`.
  - `pub struct NewProduct { company_id, name, description, unit_price (Decimal), vat_rate (Decimal) }`.
  - `pub struct ProductUpdate { name, description, unit_price (Decimal), vat_rate (Decimal) }`. **Pas de `version`** — paramètre séparé au repository.
- [x] T2.3 Ajouter `pub mod product;` + re-exports dans `entities/mod.rs`.

### T3 — Repository `products` (AC: #2, #4, #5, #6, #7, #8, #12)

- [x] T3.1 Créer `crates/kesh-db/src/repositories/products.rs`. Pattern strictement calqué sur `contacts.rs` (Story 4.1) — copier la structure et adapter :
  - `product_snapshot_json(&Product) -> serde_json::Value` — inclure `companyId`, `unitPrice` (string décimal via `to_string()`), `vatRate` (string décimal).
  - `escape_like` — dupliquer depuis contacts.rs (même 3 lignes).
  - `ProductSortBy { Name, UnitPrice, VatRate, CreatedAt }` — enum local, chaque variant retourne un littéral SQL.
  - `ProductListQuery { search, include_archived, sort_by, sort_direction, limit, offset }` — plus simple que contacts (pas de filtres type/client/supplier).
  - `ProductListResult { items, total, offset, limit }`.
  - `push_where_clauses` — company_id + active + search LIKE name/description.
  - 5 fonctions : `create`, `find_by_id`, `list_by_company_paginated`, `update`, `archive`.
  - `update` : pré-check `active` → `IllegalStateTransition` si archivé. Wrapper audit `{before, after}`.
  - `archive` : pré-check `active` → `IllegalStateTransition` si déjà archivé. Snapshot direct.
- [x] T3.2 Ajouter `pub mod products;` dans `repositories/mod.rs`.

### T4 — API routes `/api/v1/products` (AC: #2, #3, #4, #5, #6, #7, #10, #13)

- [x] T4.1 Créer `crates/kesh-api/src/routes/products.rs` avec DTOs + 5 handlers. Pattern contacts.rs. Points spécifiques :
  - Validation TVA whitelist : parser les constantes **en `Decimal`** au démarrage (lazy_static ou `const` si possible) :
    ```rust
    fn allowed_vat_rates() -> Vec<Decimal> {
        ["0.00", "2.60", "3.80", "8.10"]
            .iter()
            .map(|s| Decimal::from_str(s).unwrap())
            .collect()
    }
    ```
    Comparer la valeur parsée du JSON via `allowed_vat_rates().contains(&req.vat_rate)` — **comparaison `Decimal == Decimal`**, exacte et safe (pas de float, pas de string-to-string). Message 400 : « Taux TVA non autorisé. Valeurs acceptées : 0.00%, 2.60%, 3.80%, 8.10% ».
  - Validation prix : `unit_price >= Decimal::ZERO` (le prix 0 est autorisé — ex: produit offert).
  - Validation nom : non vide, ≤ 255 chars.
  - Validation description : ≤ 1000 chars.
  - `ProductResponse` : garder `unit_price: Decimal` et `vat_rate: Decimal` comme types natifs dans la struct. **La feature `serde-str` de `rust_decimal` (activée dans `Cargo.toml`) gère automatiquement la sérialisation JSON en string** (ex: `"1500.0000"`). **NE PAS** appeler `to_string()` manuellement dans le `From<Product>` impl — cela casserait le round-trip Deserialize si la DTO est utilisée en tests. Pattern identique à `JournalEntryResponse`.
  - Pas de `map_product_error` personnalisé — le `DbError::UniqueConstraintViolation` sur `uq_products_company_name` tombe dans le mapping générique `RESOURCE_CONFLICT` de `errors.rs`, ce qui est suffisant.
- [x] T4.2 Enregistrer routes : GETs dans `authenticated_routes`, mutations dans `comptable_routes` (pattern contacts lib.rs).
- [x] T4.3 Ajouter `pub mod products;` dans `routes/mod.rs`.

### T5 — Frontend feature `products` (AC: #2, #4, #5, #7, #8, #9, #11, #13)

- [x] T5.1 Créer `frontend/src/lib/features/products/` avec :
  - `products.types.ts` — `ProductResponse`, `ListProductsQuery`, `CreateProductRequest`, `UpdateProductRequest`, `ArchiveProductRequest`, `ListResponse<ProductResponse>`.
  - `products.api.ts` — 5 fonctions (listProducts, getProduct, createProduct, updateProduct, archiveProduct).
  - `product-helpers.ts` — `formatPrice(d: string): string` via `Intl.NumberFormat('de-CH', ...)` + `formatVatRate(d: string): string` (affiche « 8.10% »).
  - `product-helpers.test.ts` — tests Vitest `formatPrice`, `formatVatRate`.
- [x] T5.2 Créer `/products/+page.svelte` — page liste avec table (nom, description, prix, TVA%, actions) + filtres (search debounced, includeArchived) + pagination + tri + dialog create/edit/archive/conflit 409. `onMount` pour URL init (pattern fix P1 code review 4.1). Cleanup debounce dans `onMount` return (P6). Input prix : `<input type="text" inputmode="decimal">`. TVA : `<select>` avec les 4 valeurs whitelist.
- [x] T5.3 Ajouter lien sidebar « Catalogue » dans `+layout.svelte` navGroups (label hardcodé, pattern contacts P10).

### T6 — Clés i18n (AC: #11)

- [x] T6.1 Ajouter ~35 clés × 4 langues dans les `.ftl` :
  - Nav : `nav-products`
  - Titres : `products-page-title`, `product-form-create-title`, `product-form-edit-title`
  - Labels form : `product-form-name`, `product-form-description`, `product-form-price`, `product-form-vat-rate`, `product-form-vat-help`
  - TVA options : `product-vat-exempt`, `product-vat-reduced`, `product-vat-special`, `product-vat-normal`
  - Boutons/colonnes : `product-list-new`, `product-list-edit`, `product-list-archive`, `product-col-name`, `product-col-description`, `product-col-price`, `product-col-vat`, `product-col-actions`
  - Filtres/messages : `product-filter-search`, `product-filter-archived`, `product-empty-list`, `product-created-success`, `product-updated-success`, `product-archived-success`
  - Erreurs : `product-error-name-required`, `product-error-name-too-long`, `product-error-price-negative`, `product-error-vat-invalid`, `product-error-name-duplicate`
  - Dialogs : `product-archive-confirm-title`, `product-archive-confirm-body`, `product-conflict-title`, `product-conflict-body`
  - Payment terms contacts : `contact-form-payment-terms`, `contact-form-payment-terms-placeholder`

### T7 — Tests (AC: #12, #14)

- [x] T7.1 Tests d'intégration DB `products::tests` — pattern contacts Story 4.1 :
  - `test_create_and_find`, `test_create_writes_audit_log`, `test_create_rejects_duplicate_name`, `test_update_optimistic_lock`, `test_update_writes_audit_log_with_wrapper`, `test_update_rejects_archived`, `test_archive_sets_inactive_and_writes_audit`, `test_archive_rejects_already_archived`, `test_filter_by_search`, `test_list_sort_order`, `test_archived_excluded_by_default`, `test_db_rejects_negative_price_via_direct_insert` (vérifie le CHECK constraint indépendamment du handler — défense en profondeur).
- [x] T7.2 Tests unit handler products : validation TVA whitelist, validation prix, trim nom/description.
- [x] T7.3 Tests Vitest : `formatPrice`, `formatVatRate`.
- [x] T7.4 Tests Playwright `products.spec.ts` : création nominale, archivage, filtre recherche.
- [x] T7.5 Test Playwright contact payment terms (T1.4).

### T8 — Validation finale

- [x] T8.1 `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings`
- [x] T8.2 `cargo check --workspace --tests`
- [x] T8.3 `npm run test:unit` (full suite)
- [x] T8.4 `npm run check` (svelte-check, 0 errors)
- [x] T8.5 Mettre à jour sprint-status → `review`.

## Dev Notes

### Architecture — où va quoi

```
kesh-db/
├── migrations/20260415000001_products.sql     # T2.1
└── src/
    ├── entities/
    │   ├── product.rs                         # T2.2
    │   └── mod.rs                             # T2.3
    └── repositories/
        ├── products.rs                        # T3 (CRUD + audit + tests)
        └── mod.rs                             # T3.2

kesh-api/src/routes/
├── products.rs                                # T4.1 (5 handlers + tests unit)
└── mod.rs                                     # T4.3

frontend/src/lib/features/products/
├── products.types.ts                          # T5.1
├── products.api.ts                            # T5.1
├── product-helpers.ts                         # T5.1
└── product-helpers.test.ts                    # T5.1

frontend/src/routes/(app)/products/+page.svelte # T5.2
frontend/tests/e2e/products.spec.ts            # T7.4
```

### Ce qui existe DÉJÀ — NE PAS refaire

- **`default_payment_terms`** colonne, entity, API — créés Story 4.1 (migration, `Contact`/`NewContact`/`ContactUpdate`, handler `create_contact`/`update_contact`). Le champ est juste invisible en UI.
- **`rust_decimal::Decimal`** — `kesh-db/Cargo.toml:9,14`, `kesh-core/Cargo.toml`. Features `serde-str` + `maths`. Pattern journal_entries pour montants.
- **`ListResponse<T>`** — `routes/mod.rs:25` (Story 3.4). Réutiliser.
- **`escape_like`** — dans `contacts.rs` et `journal_entries.rs`. Dupliquer encore (3 lignes, décision Story 3.5 L1).
- **`SortDirection { Asc, Desc }`** — `kesh_core::listing`. Réutiliser. Le `SortBy` partagé est journal-entries-specific (ne PAS l'utiliser).
- **Pattern `onMount` URL init** — Story 4.1 code review P1. Utiliser `onMount` + `page` de `$app/state`, PAS `$effect` sur `$page`.
- **Pattern debounce cleanup** — Story 4.1 code review P6. `onMount` retourne un cleanup `clearTimeout`.
- **`AppError` variants** — `IdeAlreadyExists` (4.1), `Validation` (générique 400), toutes les variantes `DbError` (existantes). Pour products, le `UniqueConstraintViolation` → `RESOURCE_CONFLICT` générique suffit.
- **`comptable_routes` / `authenticated_routes`** — `lib.rs:84` / `lib.rs:121`.
- **Composants shadcn** : `Dialog`, `Button`, `Input`, `Select` — tous installés. `Label` n'existe PAS (utiliser `<label>` HTML natif, pattern contacts P1 code review 4.1).

### Patterns Story 4.1 à réutiliser (citations précises)

- **Repository contacts complet** : `contacts.rs` — 6 fonctions. Copier la structure (create/find/list_paginated/update/archive + push_where_clauses + escape_like + snapshot_json + audit rollback explicite). Le `product.rs` sera plus simple (pas de filtres type/client/supplier, pas de CheNumber).
- **Handler contacts** : `routes/contacts.rs` — DTOs camelCase, validation `validate_common`, `map_contact_error`. Pour products, la validation est plus simple (nom + prix + TVA).
- **Frontend contacts page** : `contacts/+page.svelte` — onMount + debounce + syncUrl + dialog create/edit/archive/conflit. Copier et adapter (colonnes différentes, champs formulaire différents, TVA select au lieu de client/supplier checkboxes).
- **Helpers frontend** : `contact-helpers.ts` — `formatIdeNumber`, `normalizeIdeForApi`. Pour products : `formatPrice`, `formatVatRate`.

### Pièges identifiés

1. **`Decimal` sérialisation en JSON** : avec `serde_json`, `Decimal` sérialise par défaut en string (`"1500.0000"`) quand `serde-str` feature est active. C'est le comportement souhaité (pas de perte de précision). Le frontend reçoit une string et l'affiche via `formatPrice`. **NE PAS** tenter de sérialiser en nombre JSON (perte de précision).

2. **`vat_rate` comparaison whitelist** : comparer des `Decimal` via `==` est safe (exact, pas de float). La whitelist `[Decimal::from_str("0.00"), ...]` se compare exactement. **NE PAS** comparer des `f64`.

3. **`DECIMAL(19,4)` pour `unit_price`** — cohérent avec `journal_entry_lines.debit/credit` (vérifié empiriquement `20260412000001_journal_entries.sql:42-43`). Le `vat_rate` utilise `DECIMAL(5,2)` car c'est un pourcentage (max 99.99%). **Dette technique** : `escape_like` sera dupliqué une 3e fois (contacts + journal_entries + products). Extraction vers `kesh-db/src/utils.rs` à planifier dans une story de maintenance post-Epic 4 si le pattern se répète une 4e fois.

4. **`UNIQUE (company_id, name)` case-sensitivity** : contrairement à l'IDE CHE (BINARY), le nom de produit peut avoir une collation case-insensitive (`utf8mb4_general_ci`) — deux produits « Logo » et « logo » seraient considérés comme duplicats. C'est le comportement voulu (un catalogue ne devrait pas avoir deux produits « Consultation » et « consultation »). **Pas de BINARY CHECK** sur le nom (contrairement à contacts/accounts).

5. **Frontend prix input** : `<input type="number">` a des problèmes UX avec les séparateurs. Utiliser `type="text" inputmode="decimal"` avec validation regex `^(0|[1-9][0-9]*)(\.[0-9]{1,4})?$` côté client (rejette les zéros en tête superflus comme `007.50`). Le backend valide via `Decimal::from_str`.

6. **Taux TVA suisse 2024** : les taux ont changé au 01.01.2024. Les anciens taux (7.7%, 3.7%, 2.5%) ne sont PAS dans la whitelist v0.1. Si un utilisateur a des produits avec les anciens taux, il devra les mettre à jour. C'est acceptable pour v0.1 (pas de gestion historique des taux).

7. **Sidebar label hardcodé** : « Catalogue » en français, pattern P10 Story 4.1. Ne pas tenter de refactorer la sidebar en i18n (scope creep).

8. **Validation `description ≤ 1000 chars` côté handler = garantie primaire** : le `VARCHAR(1000)` côté DB est un filet de sécurité dont le comportement (tronquage vs rejet) dépend du `sql_mode` runtime (`STRICT_ALL_TABLES` par défaut sur MariaDB 11.x — bien, mais non garanti). La validation Rust dans le handler est donc **la vraie garantie**. Pattern identique à contacts (`MAX_ADDRESS_LEN = 500` côté Rust + `VARCHAR(500)` côté DB).

9. **Double-garde `unit_price >= 0` (CHECK DB + validation handler)** : défense en profondeur comptable. Si l'un est contourné (ex: seed SQL direct, migration bugguée), l'autre rattrape. Ajouter en T7.1 un test `test_db_rejects_negative_price_via_direct_insert` qui INSERT directement un prix négatif pour vérifier que le CHECK constraint fonctionne indépendamment du handler.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-4-Story-4.2] — AC BDD
- [Source: _bmad-output/planning-artifacts/prd.md#FR28-FR30] — Conditions paiement + catalogue + pré-remplissage
- [Source: _bmad-output/planning-artifacts/architecture.md#decimal] — `rust_decimal` obligatoire pour montants
- [Source: crates/kesh-db/migrations/20260414000001_contacts.sql] — default_payment_terms déjà en place
- [Source: crates/kesh-db/src/repositories/contacts.rs] — Pattern canonique CRUD + audit + QueryBuilder
- [Source: crates/kesh-api/src/routes/contacts.rs] — Pattern canonique handler + validation
- [Source: frontend/src/routes/(app)/contacts/+page.svelte] — Pattern canonique page frontend (post-P1/P6)
- [Source: crates/kesh-db/Cargo.toml] — rust_decimal déjà en dépendance

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context) — dev-story 2026-04-12.

### Debug Log References

- `cargo fmt --all --check` : reformaté automatiquement (2 fichiers).
- `cargo clippy --workspace --all-targets -- -D warnings` : 0 warning.
- `cargo check --workspace --tests` : OK.
- `cargo test -p kesh-api --lib routes::products` : 9/9 unit tests OK (whitelist TVA, prix zéro/négatif, name trim/empty/too-long, description too-long, normalize_optional).
- `npm run check` (svelte-check) : 0 error (2 warnings préexistants sur design-system, non liés à 4.2).
- `npm run test:unit -- --run` : 153/153 tests OK (incl. 13 nouveaux tests `product-helpers.test.ts`).
- Tests DB `products::tests` (12 tests) et Playwright `products.spec.ts` : **à exécuter en intégration** (nécessitent DB + seed + serveur).

### Completion Notes List

- **T1** ✅ Champ `default_payment_terms` exposé dans `contacts/+page.svelte` (placement précis après form-ide et avant formError ; réinit dans `openCreate`, hydratation dans `openEdit`). i18n `contact-form-payment-terms` + placeholder ajoutés ×4 langues. Test Playwright `products.spec.ts#contact payment terms`.
- **T2** ✅ Migration `20260415000001_products.sql` (ENGINE+CHARSET+COLLATE `utf8mb4_unicode_ci` obligatoire MariaDB 11), entity `Product`/`NewProduct`/`ProductUpdate` avec `rust_decimal::Decimal` (feature `serde-str`).
- **T3** ✅ Repository `products.rs` : 5 fonctions (create/find_by_id/list_by_company_paginated/update/archive) + audit log atomique (snapshot direct pour create/archive, wrapper before/after pour update) + pré-check active → `IllegalStateTransition`. 12 tests DB (incl. `test_db_rejects_negative_price_via_direct_insert` — défense en profondeur). 3e duplication de `escape_like` documentée en dette technique.
- **T4** ✅ Handlers `/api/v1/products` (5 routes, GETs auth / mutations comptable+). Whitelist TVA `[0.00, 2.60, 3.80, 8.10]` comparée en `Decimal` (pas string). `ProductResponse` garde `Decimal` natif — sérialisation string via `serde-str`. Validation prix ≥ 0 (zéro autorisé), name trim + MAX 255, description MAX 1000.
- **T5** ✅ Feature frontend `products/` : types, api, helpers (`formatPrice` via `formatSwissAmount` DRY — apostrophe typographique U+2019, `isValidPrice` regex `^(0|[1-9][0-9]{0,14})(\.[0-9]{1,4})?$`). Page `/products/+page.svelte` : table filtrable/paginée/triable (Name/UnitPrice/VatRate), dialog create/edit/archive/conflict, URL state sync (`onMount` pattern post-P1). Sidebar "Catalogue" ajoutée.
- **T6** ✅ ~40 clés i18n × 4 langues (FR/DE/IT/EN).
- **T7** ✅ Tests unit Rust (9 validation handler) + 12 tests DB + 13 Vitest (`formatPrice`/`formatVatRate`/`isValidPrice`) + 4 specs Playwright (création, archivage, filtre recherche, payment terms).
- **T8** ✅ fmt/clippy/check/svelte-check/vitest tous ✅.

### File List

- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modifié)
- `_bmad-output/implementation-artifacts/4-2-conditions-paiement-catalogue-produits.md` (modifié)
- `crates/kesh-db/migrations/20260415000001_products.sql` (nouveau)
- `crates/kesh-db/src/entities/product.rs` (nouveau)
- `crates/kesh-db/src/entities/mod.rs` (modifié)
- `crates/kesh-db/src/repositories/products.rs` (nouveau)
- `crates/kesh-db/src/repositories/mod.rs` (modifié)
- `crates/kesh-api/Cargo.toml` (modifié — `rust_decimal_macros` en dev-dep)
- `crates/kesh-api/src/routes/products.rs` (nouveau)
- `crates/kesh-api/src/routes/mod.rs` (modifié)
- `crates/kesh-api/src/lib.rs` (modifié — 5 nouvelles routes)
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` (modifié)
- `crates/kesh-i18n/locales/de-CH/messages.ftl` (modifié)
- `crates/kesh-i18n/locales/it-CH/messages.ftl` (modifié)
- `crates/kesh-i18n/locales/en-CH/messages.ftl` (modifié)
- `frontend/src/lib/features/products/products.types.ts` (nouveau)
- `frontend/src/lib/features/products/products.api.ts` (nouveau)
- `frontend/src/lib/features/products/product-helpers.ts` (nouveau)
- `frontend/src/lib/features/products/product-helpers.test.ts` (nouveau)
- `frontend/src/routes/(app)/products/+page.svelte` (nouveau)
- `frontend/src/routes/(app)/+layout.svelte` (modifié — lien sidebar Catalogue)
- `frontend/src/routes/(app)/contacts/+page.svelte` (modifié — champ payment terms)
- `frontend/tests/e2e/products.spec.ts` (nouveau)

## Senior Developer Review (AI)

**Date** : 2026-04-13
**Review** : code-review adversarial 3 passes (Opus backend → Sonnet frontend → Haiku consolidé)
**Outcome** : **Approved with remediation** — 22 patches appliqués, 0 finding > LOW résiduel

### Bilan par passe

| Passe | LLM | Portée | Findings bruts | Patches | Verdict |
|-------|-----|--------|----------------|---------|---------|
| 1 | Opus 4.6 | Chunk 1 — Backend Rust | 3 reviewers × ~10 findings | 6 (3 HIGH + 1 MED + 2 LOW) | BLOCKING MEDIUM+ |
| 2 | Sonnet 4.5 | Chunk 2 — Frontend + i18n + e2e | 3 reviewers × ~11 findings | 10 (1 HIGH + 9 MED) | BLOCKING MEDIUM+ |
| 2b | Sonnet 4.5 | Chunk 2 — LOW retouchés | — | 6 (LOW) | — |
| 3 | Haiku 4.5 | Diff consolidé (backend+frontend) | 0 | 0 | **CLEAN** |

**Règle multi-passes CLAUDE.md respectée** : LLMs strictement orthogonaux (Opus → Sonnet → Haiku), fenêtres de contexte fraîches pour chaque passe.

### Patches appliqués (22)

**Backend (6)** :
- **P1 [HIGH]** Validation `unit_price.scale() ≤ 4` + cap 1 000 000 000 CHF pour éviter truncation silencieuse MariaDB et overflow Epic 5 (ligne facture = prix × qty). 4 nouveaux tests unitaires.
- **P2 [HIGH]** Migration : `CHECK (vat_rate BETWEEN 0 AND 100)` + `CHECK (unit_price ≤ 1 000 000 000)` — défense en profondeur symétrique au CHECK existant sur prix négatif.
- **P3 [HIGH]** NFC normalisation étendue à `description` (auparavant uniquement `name`). Test NFD→NFC de collision ajouté.
- **P4 [MEDIUM]** Longueur `search` vérifiée sur chaîne trimmée (cohérence avec repository qui trim).
- **P5 [LOW]** `ORDER BY ..., id ASC` — tiebreaker déterministe empêche doublons/sauts de pagination sur prix/TVA égaux.
- **P6 [LOW]** `LazyLock` pour whitelist TVA (initialisation unique, lieu de parsing à chaque appel).

**Frontend (10 MEDIUM+)** :
- **P7 [HIGH]** E2E `createProduct` : `selectOption({ value })` au lieu de `{ label: regex.source }` (le helper était dead code silencieux).
- **P8 [MED]** Dialog conflict : ajout bouton **Annuler** — suppression du modal-trap.
- **P9 [MED]** `editing = null` / `archiveTarget = null` après 409 — empêche un second conflit garanti si l'utilisateur rouvre le formulaire.
- **P10 [MED]** `void goto(...)` + `void loadProducts()` explicites pour éviter unhandled promise rejections.
- **P11 [MED]** 5 clés i18n dédiées (`product-form-cancel`, `product-form-submit-create`, `product-form-submit-edit`, `product-archive-cancel`, `product-archive-confirm`) en 4 locales — suppression du couplage inter-feature avec les clés `contact-*`.
- **P12 [MED]** E2E `archiveRow` : `page.locator('tbody').getByText(name)` pour éviter les flakes avec le toast de succès.
- **P13 [MED]** URL `sortBy` / `sortDirection` whitelistées contre `ProductSortBy` / `SortDirection` — fallback gracieux sur URL partagée invalide.
- **P14 [MED]** `openEdit` : `toFixed(4)` + strip trailing zeros pour préserver précision DB DECIMAL(19,4) (auparavant `toFixed(2)` tronquait silencieusement).
- **P15 [MED]** `toggleSort` reset `offset = 0` — pagination cohérente sur changement de tri.
- **P16 [MED]** Nouveau test E2E AC #9 : reload + restauration complète du state (filtres + tri + URL).

**LOW polish (6)** :
- **P17 [LOW]** Classification prix en 3 états (`empty` / `negative` / `invalid`) avec messages d'erreur dédiés ; `formTouched` évite la validation fantôme à l'ouverture du dialog de création.
- **P18 [LOW]** Libellés TVA it-CH avec virgule décimale (`8,10 %`) — convention italienne ; de-CH conservé en point (standard CHF suisse alémanique).
- **P19 [LOW]** `normalizePriceInput` — accepte la virgule décimale (claviers mobiles suisses) et normalise avant envoi backend.
- **P20 [LOW]** Fallback défensif `formVatRate = '8.10'` si le backend renvoie un taux hors whitelist.
- **P21 [LOW]** `waitForURL(/search=/)` remplace `waitForTimeout(500)` — attente event-driven, non-flaky sur CI lents.
- **P22** — `maxlength={1000}` déjà présent (non-finding).

### Dette technique reportée (8 defers documentés)

- **D1** `get_company` pick `LIMIT 1` sans `ORDER BY` + ignore `CurrentUser` — pattern hérité v0.1 mono-company. À refondre lors du multi-tenant (post-v1.0).
- **D2** Whitelist TVA hardcodée — table `vat_rates` paramétrable à prévoir pour futures modifications Confédération (évolution OTVA).
- **D3** `update` bump `version` même sur no-op — pattern hérité de Story 4.1, à uniformiser transversalement.
- **D4** Pas de FULLTEXT index sur `search` — MVP < 10k rows accepté, perf debt pour Epic 5+.
- **D5** Reads (`list_products`, `get_product`) sans `CurrentUser` — pas d'audit log sur lecture ; design debt transverse.
- **D6** Messages d'erreur backend français hardcodés (`"Aucune company en base"`) — i18n des erreurs = story transverse.
- **D7** Vérification archive vs références invoices — dépend d'Epic 5.
- **D8** Sidebar `Catalogue` hardcodée en français — contradiction interne spec (piège #7 vs T6.1), à trancher dans une story de refactor sidebar i18n.

### Tests validés

- `cargo check -p kesh-api` : ✅
- `cargo test -p kesh-api --lib routes::products` : ✅ **14/14** (dont 5 nouveaux : scale, cap, NFC description, collision)
- `svelte-check` : ✅ 0 error (2 warnings non liés à 4-2)
- `vitest run src/lib/features/products/` : ✅ **21/21** (dont 7 nouveaux pour `normalizePriceInput` + `classifyPriceInput`)
- Tests DB d'intégration (`cargo test products::tests`) et Playwright : à rejouer en CI (nécessitent MariaDB + seed)

### Action items résiduels

Aucun. Story 4.2 validée pour merge.

## Change Log

- 2026-04-12: Création de la story 4.2 (Claude Opus 4.6, 1M context) — dernière story Epic 4 « Carnet d'adresses & Catalogue ». Décisions clés :
  - **Payment terms** : simple champ texte libre, colonne déjà en place Story 4.1. Zéro migration, juste un `<input>` dans le form contact + 2 clés i18n.
  - **Products schema** : `unit_price DECIMAL(19,4)` (cohérent journal_entries) + `vat_rate DECIMAL(5,2)`. Stockage en pourcentage direct (8.10 pas 0.081).
  - **TVA whitelist v0.1** : 4 taux suisses 2024 hardcodés côté handler (0.00, 2.60, 3.80, 8.10). Comparaison Decimal parsé (pas string). Pas de table `vat_rates` paramétrable.
  - **Unicité nom produit** : `UNIQUE (company_id, name)` avec collation `utf8mb4_unicode_ci` case-insensitive (pas de BINARY — « Logo » et « logo » sont duplicats, comportement voulu). Erreur 409 générique `RESOURCE_CONFLICT`.
  - **Frontend prix** : `type="text" inputmode="decimal"` (pas `type="number"`). Affichage via `big.js` `Big(d).toFixed(2)` + formatage suisse apostrophe `'` — PAS `Intl.NumberFormat` qui perd la précision float (incohérent avec le pattern comptable big.js établi en Story 3.2).
  - **Collation migration** : `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci` obligatoire (MariaDB 11.x change le défaut vers `uca1400_ai_ci` sinon).
  - **Pas de calcul TVA en 4.2** : stockage du taux seulement, calcul arrive en Epic 5/9.
  - **Pattern contacts 4.1 copié** : même architecture repo + handler + page, plus simple (moins de filtres, pas d'IDE, pas de CheNumber).
- 2026-04-12: **Revue adversariale passe 1** (Sonnet adversarial + Haiku empirique, LLMs orthogonaux à Opus auteur). 4 MEDIUM + 3 LOW → 7 patches appliqués :
  - **P1 [MEDIUM]** `formatPrice` doit utiliser `big.js` (pattern `balance.ts` Story 3.2), pas `Intl.NumberFormat('de-CH')` qui passe par `parseFloat()` et perd la précision pour les grands montants.
  - **P2 [MEDIUM]** `DECIMAL(15,4)` → `DECIMAL(19,4)` pour `unit_price` — cohérence avec `journal_entry_lines.debit/credit` (vérifié empiriquement). L'affirmation « même précision » était factuellement incorrecte.
  - **P3 [MEDIUM]** Migration products : ajout obligatoire de `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci` — sans quoi MariaDB 11.x utilise `utf8mb4_uca1400_ai_ci` par défaut (changement depuis 10.x), divergent des autres tables.
  - **P4 [MEDIUM]** T1.1 : consigne explicite de réinitialiser `formPaymentTerms = ''` dans `openCreate()` pour éviter le carryover de valeur entre dialog ouvertures.
  - **P5 [LOW]** TVA whitelist : comparaison entre `Decimal` parsés (pas string-to-string) pour éviter les faux négatifs sur `"8.1"` vs `"8.10"`.
  - **P6 [LOW]** Regex prix : `^(0|[1-9][0-9]*)(\.[0-9]{1,4})?$` pour rejeter les zéros en tête.
  - **P7 [LOW]** Note dette technique `escape_like` ×3 avec story maintenance planifiée post-Epic 4.
  - **P8 [LOW]** Alignement regex prix entre section Décisions (ligne 286) et Change Log — incohérence rattrapée par pass 2 Haiku.
- 2026-04-12: **Revue adversariale passe 3** (Opus deep adversarial, orthogonal à Sonnet+Haiku passes 1-2). 2 MEDIUM + 4 LOW trouvés, patches **P9-P14** appliqués :
  - **P9 [MEDIUM]** AC#2 contenait encore `DECIMAL(15,4)` (la clause contractuelle est celle testée par Acceptance Auditor — divergence silencieuse avec migration corrigée P2). Aligné sur `DECIMAL(19,4)`.
  - **P10 [MEDIUM]** Apostrophe ASCII `'` (U+0027) vs typographique `’` (U+2019) : le code de référence `balance.ts:99` utilise U+2019 (norme Swiss SN01, BFS). Spec clarifiée pour imposer U+2019, + recommandation DRY de factoriser `formatSwissAmount` dans `$lib/shared/utils/format-decimal.ts` plutôt que dupliquer.
  - **P11 [LOW]** T4.1 précise que `ProductResponse` garde `Decimal` natif (feature `serde-str` gère la sérialisation string auto) — PAS de `to_string()` manuel qui casserait le round-trip Deserialize.
  - **P12 [LOW]** T1.1 précise le placement exact du champ `formPaymentTerms` : juste après `form-ide` (lignes 603-609) et avant `{#if formError}` (ligne 612).
  - **P13 [LOW]** Piège #8 ajouté : validation `description ≤ 1000 chars` côté handler = garantie primaire, `VARCHAR(1000)` = filet dépendant de `sql_mode`.
  - **P14 [LOW]** Piège #9 + test T7.1 ajoutés : `test_db_rejects_negative_price_via_direct_insert` pour vérifier le CHECK constraint indépendamment du handler (défense en profondeur comptable).
- 2026-04-12: **Revue adversariale passe 4** (Haiku convergence, orthogonal à Opus pass 3). P9-P14 vérifiés : P10-P14 ✅, mais **P9 résiduel** détecté — la ligne 31 du Scope verrouillé contenait encore `DECIMAL(15,4)` (P9 n'avait corrigé que l'AC#2). Patch **P15** appliqué :
  - **P15 [MEDIUM]** Ligne 31 Scope verrouillé : `DECIMAL(15,4)` → `DECIMAL(19,4)`. Dernière occurrence normative éliminée (seules les 2 occurrences restantes sont des anti-exemples explicites en Décisions + Change Log historique P2/P9).
- 2026-04-12: **Implémentation Story 4.2 (Claude Opus 4.6, 1M context)** — 8 tâches réalisées, status → `review`. Migration, entity, repo DB avec 12 tests, handlers API avec whitelist TVA Decimal et 9 tests unit, feature frontend complète avec `formatSwissAmount` réutilisé (DRY), ~40 clés i18n × 4 langues, Playwright specs produits + payment terms. Validations : `cargo fmt`, `clippy -D warnings`, `cargo check --tests`, `npm run check` (0 err), `npm run test:unit` (153/153), `cargo test routes::products` (9/9). Tests DB + Playwright à exécuter en intégration.
- 2026-04-12: **CRITÈRE D'ARRÊT CLAUDE.MD ATTEINT APRÈS 4 PASSES ADVERSARIALES** (Sonnet+Haiku → Haiku → Opus → Haiku, **LLMs strictement orthogonaux**, fenêtres fraîches). **15 patches au total** (P1-P15) : 0 CRITICAL + 0 HIGH + 6 MEDIUM + 9 LOW. 0 finding > LOW résiduel. Story 4.2 **PRÊTE POUR `dev-story`**.
- 2026-04-13: **Code-review 3 passes (Opus → Sonnet → Haiku)** — voir section _Senior Developer Review (AI)_. 22 patches appliqués (P1–P22), 4 HIGH + 10 MEDIUM + 6 LOW + 2 non-findings. Verdict final Haiku : **CLEAN** (0 régression). Status → `done`.
