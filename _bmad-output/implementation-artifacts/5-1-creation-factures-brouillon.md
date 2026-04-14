# Story 5.1: Création de factures (brouillon)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **créer des factures en mode brouillon avec des lignes libres ou depuis le catalogue**,
so that **je puisse préparer la facturation de mes clients avant validation (Story 5.2) et génération PDF QR Bill (Story 5.3)**.

### Contexte

**Première story de l'Epic 5 — Facturation QR Bill**. Couvre **FR31** (lignes libres + catalogue) et **FR32** (CRUD brouillon) de l'Epic 5. Pose les fondations sur lesquelles 5.2 (validation + numérotation + écriture comptable) et 5.3 (PDF QR Bill) vont s'appuyer.

**Fondations déjà en place** (NE PAS refaire) :

- **Table `contacts`** — Story 4.1. Le champ `default_payment_terms VARCHAR(100) NULL` existe et est exposé en UI depuis Story 4.2. La facture référence un contact via `contact_id BIGINT NOT NULL` avec FK `ON DELETE RESTRICT`.
- **Table `products`** — Story 4.2. Les lignes de facture peuvent copier `name`, `unit_price`, `vat_rate` depuis un produit **sans FK** (voir décisions de conception ci-dessous).
- **Pattern Repository CRUD + audit log** — `contacts.rs` (Story 4.1) et `products.rs` (Story 4.2) sont les modèles canoniques. 6 fonctions, audit atomique avec rollback explicite, `XListQuery`/`XListResult`/`XSortBy` locaux, `escape_like` dupliqué. **À copier pour invoices**, avec adaptation pour la relation 1-N avec `invoice_lines`.
- **`rust_decimal::Decimal`** — déjà en dépendance. Features `serde-str` + `maths`. Utilisé par `journal_entries` (montants) et `products` (prix, TVA). **À réutiliser pour `quantity`, `unit_price`, `vat_rate`, `line_total`, `total_amount`.**
- **`ListResponse<T>`** — `routes/mod.rs:25`. **À réutiliser**, pas de nouveau type.
- **Tous les patterns frontend Stories 4.1/4.2** : `onMount` (pas `$effect`) pour lecture URL initiale, debounce 300ms, `notify*` helpers, `i18nMsg` canonical, dialog create/edit/delete/conflit 409, formatage `big.js` + apostrophe U+2019 via `formatSwissAmount`.
- **Whitelist TVA suisse** — définie dans `kesh-api/src/routes/products.rs` (constante `[0.00, 2.60, 3.80, 8.10]`). **À extraire en helper partagé `kesh-api/src/routes/vat.rs`** (`pub fn allowed_vat_rates() -> &'static [Decimal]`) pour éviter la duplication entre products et invoice_lines. **DRY** : n'importer qu'une source de vérité.
- **Sidebar nav** — pattern `+layout.svelte` navGroups. Ajouter « Factures ».
- **Aucun code `invoices` n'existe** — vérifié empiriquement. Tout à créer de zéro.

### Scope verrouillé — ce qui DOIT être fait

1. **Migration `invoices` + `invoice_lines`** — deux tables liées par FK avec `ON DELETE CASCADE` (suppression d'une facture brouillon supprime ses lignes).
2. **Entités Rust** — `Invoice`, `NewInvoice`, `InvoiceUpdate`, `InvoiceLine`, `NewInvoiceLine`, `InvoiceLineUpdate`.
3. **Repository `invoices`** — CRUD facture brouillon + gestion atomique des lignes (replace-all sur update). Audit log sur chaque mutation (`invoice.created`, `invoice.updated`, `invoice.deleted`).
4. **API routes `/api/v1/invoices`** — 5 handlers : `list`, `get`, `create`, `update`, `delete`. GETs dans `authenticated_routes`, mutations dans `comptable_routes`.
5. **Frontend feature `invoices`** — page `/invoices` liste + page `/invoices/new` et `/invoices/:id/edit` (formulaire complexe multi-lignes). Sélecteur contact (dropdown cherchable), sélecteur produit par ligne (ou ligne libre), calcul total en temps réel.
6. **i18n** — ~50 nouvelles clés × 4 langues.
7. **Tests** — Rust DB + unit handler + Vitest + Playwright.

### Scope volontairement HORS story — décisions tranchées

- **Validation de facture / numérotation séquentielle** → **Story 5.2**. En 5.1, `status` est toujours `draft`, `invoice_number` reste `NULL`.
- **Écriture comptable automatique** (débit client / crédit produit) → **Story 5.2**.
- **Génération PDF QR Bill** → **Story 5.3**.
- **Calcul TVA agrégé / décompte par période** → **Epic 9** (TVA Suisse). En 5.1, chaque ligne stocke son taux TVA mais aucun calcul « TVA totale par taux » n'est exposé.
- **Avoirs / notes de crédit** → **Epic 10**.
- **Gestion multi-devises** → hors v0.1. `total_amount` implicitement en CHF.
- **Acomptes / paiements partiels** → hors scope 5.1 (apparaît en Epic 10 / v0.2).
- **Envoi par email** → hors v0.1.
- **Templates de factures personnalisables** → hors v0.1. Un seul template PDF (câblé en 5.3).
- **Duplication de facture** (« nouvelle à partir de ») → nice-to-have, **reporté** (à évaluer en fin d'Epic 5).
- **Exercice comptable / cut-off par période** → la date de facture est libre en 5.1. La contrainte « cohérence avec exercice ouvert » arrive en Epic 12 (Clôture).

### Décisions de conception

- **Lignes de facture indépendantes du catalogue (pas de FK `product_id`)** — chaque ligne stocke en dur `description`, `quantity`, `unit_price`, `vat_rate` au moment de la création. **Raison** : la modification ou l'archivage d'un produit catalogue NE DOIT PAS rétroactivement altérer une facture existante. Copie snapshot. Le catalogue n'est qu'un accélérateur de saisie. Ce pattern est explicitement énoncé en Epic 5 (« lignes facture » = données propres). **Alternative rejetée** : FK nullable `product_id` → introduit un couplage fragile (archive produit → facture incohérente) sans valeur ajoutée pour v0.1.

- **`total_amount` calculé côté backend et stocké en DB** — `total_amount DECIMAL(19,4) NOT NULL` persisté à chaque `create`/`update`. **Raison** : évite de recalculer au chargement (pour la liste paginée qui affiche le total), et sert de point de vérification lors de la validation 5.2 et la génération PDF 5.3. Le backend recalcule `total_amount` à partir des lignes à chaque mutation (**single source of truth = lignes**). Le frontend peut afficher un total temps réel pendant la saisie, mais la valeur persistée est celle recalculée par le backend. **`line_total` par ligne est également stocké** (`quantity × unit_price`, sans TVA en 5.1 — la ventilation TVA arrive en 5.2). Idempotence du recalcul validée en tests.

- **Statut par enum texte** — colonne `status VARCHAR(16) NOT NULL DEFAULT 'draft'` avec CHECK `status IN ('draft', 'validated', 'cancelled')`. Story 5.1 ne manipule que `draft`. Stories 5.2 et 10.1 (avoirs) introduiront les transitions. **Pattern identique à `journal_entries.status`**. Pas d'enum SQLx (cf. piège documenté dans `feedback_sqlx_mysql_gotchas`).

- **`invoice_number VARCHAR(64) NULL`** — reste NULL en brouillon. Story 5.2 attribuera une valeur non-null au moment de la validation avec contrainte `UNIQUE(company_id, invoice_number) WHERE invoice_number IS NOT NULL`. **En 5.1, pas d'index unique encore** — on ajoutera l'index conditionnel (MariaDB : via colonne générée + unique) en 5.2. **Ne pas anticiper** — chaque story fait ses propres changements schéma.

- **`date` et `due_date` en `DATE` (pas DATETIME)** — une facture a une date calendaire, pas une horodate. `date NOT NULL`, `due_date NULL` (l'utilisateur peut remplir plus tard ou selon les conditions de paiement). Pas de calcul automatique de `due_date` en 5.1 (nice-to-have pour 5.2 ou ultérieur).

- **`payment_terms VARCHAR(255) NULL`** — copié au moment de la création depuis `contact.default_payment_terms` (snapshot), modifiable par l'utilisateur dans le formulaire facture. Même logique snapshot que les lignes : modifier les conditions par défaut d'un contact NE DOIT PAS affecter les factures passées.

- **Suppression d'une facture brouillon = hard delete** — pas de soft-delete (pas de colonne `active`). **Raison** : une facture brouillon n'a pas d'existence comptable, elle peut disparaître complètement. Les factures validées (5.2) ne seront JAMAIS supprimables (seul un avoir l'annule). L'audit log conserve la trace (`invoice.deleted` + snapshot complet) — c'est suffisant.

- **`FOREIGN KEY invoice_lines.invoice_id REFERENCES invoices(id) ON DELETE CASCADE`** — cohérent avec hard-delete brouillon : supprimer la facture supprime ses lignes. **Pas `ON DELETE RESTRICT`**. Pattern identique à journal_entries/journal_entry_lines.

- **Impact de `FK invoices.contact_id ON DELETE RESTRICT` sur les contacts** — une fois cette migration déployée, **la suppression d'un contact qui a au moins une facture renverra une erreur FK violation**. Le handler `DELETE /api/v1/contacts/:id` (Story 4.1) doit mapper `DbError::ForeignKeyConstraintViolation` (ou équivalent SQLx MariaDB) → 409 avec message i18n « Impossible de supprimer ce contact : il a des factures liées ». Ajouter un test d'intégration `test_delete_contact_rejected_when_has_invoices` dans `contacts::tests` au moment de cette story. Ne pas considérer ce comportement comme un bug de la Story 4.1 — c'est une évolution normale liée à l'introduction des factures.

- **Replace-all sur update des lignes** — à chaque `update_invoice`, le repository **supprime toutes les lignes existantes puis insère les nouvelles** dans une transaction. **Raison** : simplifie énormément la logique (pas de diff LCS ligne à ligne), cohérent avec l'ergonomie UI (l'utilisateur réorganise ses lignes librement dans le dialog). **Contrepartie** : les IDs de ligne changent à chaque update — c'est acceptable car aucune entité externe n'y référence (en 5.1 comme en 5.2). À réévaluer en Epic 10 si les avoirs référencent des lignes spécifiques. Test d'intégration explicite sur ce comportement.

- **Verrouillage optimiste sur la facture (`version`)** — pattern identique contacts/products. `UPDATE ... WHERE id = ? AND version = ? → 0 rows → 409 OPTIMISTIC_LOCK_CONFLICT`. Pas de version par ligne (les lignes sont remplacées en bloc).

- **`total_amount` calculé côté backend** — `total_amount = Σ (quantity × unit_price)` sur toutes les lignes, **sans TVA** en 5.1 (la ventilation TVA arrive en 5.2). Stocké en `DECIMAL(19,4)`. Le frontend peut afficher le total en temps réel pendant la saisie (preview), mais la valeur persistée est recalculée par le backend à chaque mutation.

- **Contrainte « au moins une ligne »** — CHECK côté application (handler), pas DB (plus simple). Message 400 `INVALID_INPUT` : « Une facture doit contenir au moins une ligne ». Le handler refuse les payloads avec `lines = []`.

- **Contrainte `quantity > 0`** — CHECK DB (`chk_invoice_lines_quantity_positive CHECK (quantity > 0)`) + validation handler. Pas de quantités négatives en 5.1 (les avoirs / notes de crédit arriveront en Epic 10 avec un type distinct).

- **Whitelist TVA partagée** — extraire `allowed_vat_rates()` de `routes/products.rs` vers `routes/vat.rs` (nouveau module `pub` dans `routes/mod.rs`). Réutilisation dans le handler `invoices`. **DRY strict** — aucune duplication de la liste. Message d'erreur identique.

- **Pagination liste factures** — offset/limit standard. Tri par défaut : `date DESC, id DESC` (les plus récentes en tête). Tri cliquable sur : date, total_amount, contact_name (join). Pas de tri sur `invoice_number` en 5.1 (tous NULL).

- **Recherche** — LIKE sur `invoice_number` (futur), `payment_terms`, et `contact.name` via JOIN. Debounce 300ms côté frontend.

- **Filtres** — `status` (draft/validated/cancelled — en 5.1 seul `draft` existe mais le filtre est prêt), `contact_id` (optionnel), `date_from`/`date_to` (range).

- **Endpoint GET lecture** — `GET /api/v1/invoices/:id` renvoie la facture AVEC ses lignes triées par `position ASC`. Pas d'endpoint séparé pour les lignes (elles n'existent pas indépendamment d'une facture).

- **Champ `position INT NOT NULL` sur `invoice_lines`** — ordre explicite (l'utilisateur peut réordonner). Pas d'auto-increment global, mais séquence par facture (0, 1, 2, …). Le backend réattribue les positions à chaque update (stable).

- **Frontend formulaire = page dédiée, pas dialog** — contrairement à contacts/products. **Raison** : le formulaire facture est complexe (entête + tableau de lignes dynamique + totaux) et bénéficie d'un espace écran complet. Routes : `/invoices` (liste), `/invoices/new` (création), `/invoices/:id/edit` (édition, draft only), `/invoices/:id` (vue lecture seule). Le bouton « Supprimer » est un dialog de confirmation sur la liste ET sur la page détail.

- **Tableau de lignes en Svelte 5** — `$state` array de lignes. Boutons « Ajouter ligne libre » / « Ajouter depuis catalogue » (ouvre un sélecteur produit dans un dialog imbriqué). Bouton suppression par ligne. Validation front : au moins 1 ligne, chaque ligne : description non vide, quantity > 0, unit_price ≥ 0, vat_rate dans la whitelist.

- **Sélecteur contact** — dropdown cherchable (`Combobox` ou équivalent Svelte). Appelle `listContacts({ search, limit: 50 })` à chaque frappe (debounced). Affiche nom + IDE en option. À la sélection, pré-remplit `payment_terms` depuis `contact.defaultPaymentTerms`.

- **Sélecteur produit (depuis catalogue)** — dialog avec liste produits (appelle `listProducts({ search, includeArchived: false })`). À la sélection, ajoute une ligne avec `description = product.name` (+ description produit en dessous si présente), `quantity = 1`, `unit_price = product.unitPrice`, `vat_rate = product.vatRate`. L'utilisateur peut ensuite ajuster quantity/prix avant sauvegarde.

- **Audit log** — `invoice.created`, `invoice.updated`, `invoice.deleted`. Snapshot inclut l'entête **et** toutes les lignes (tableau). Convention : `created`/`deleted` = snapshot direct, `updated` = wrapper `{before, after}` avec les deux versions complètes (entête + lignes).

- **RBAC** — GETs (liste, détail) dans `authenticated_routes` (accessible aux lecteurs). Mutations (create, update, delete) dans `comptable_routes`. Pattern contacts/products.

- **Gestion 409 `ILLEGAL_STATE_TRANSITION`** — si l'utilisateur tente d'éditer/supprimer une facture `status != 'draft'`. En 5.1 c'est défensif (aucune facture n'est validée), mais la vérification est déjà implémentée pour faciliter 5.2.

## Acceptance Criteria (AC)

1. **Création facture brouillon nominale** (FR31, FR32) — Given un contact existant et un produit catalogue, When l'utilisateur crée une facture avec 1 ligne depuis le catalogue + 1 ligne libre, Then la facture est créée avec `status = 'draft'`, `invoice_number = NULL`, `version = 1`, 2 lignes persistées avec `position = 0, 1`, `total_amount = Σ (qty × unit_price)`, et une entrée audit `invoice.created` avec snapshot complet.

2. **Lignes depuis catalogue copient le snapshot** — Given un produit P1 avec `unit_price = 150.00`, `vat_rate = 8.10`, When ajout d'une ligne depuis ce produit, Then la ligne contient `description = P1.name`, `unit_price = 150.00`, `vat_rate = 8.10` (snapshot). When P1 est modifié après (prix passe à 200.00), Then la ligne de la facture existante conserve `unit_price = 150.00` (immutabilité).

3. **Lignes libres** — Given la création d'une facture, When l'utilisateur ajoute une ligne libre avec `description = "Conseil stratégie"`, `quantity = 4.5`, `unit_price = 200.00`, `vat_rate = 8.10`, Then la ligne est persistée avec `line_total = 900.0000` (4.5 × 200.00).

4. **Modification brouillon avec verrouillage optimiste** — Given une facture brouillon v1 avec 2 lignes, When l'utilisateur remplace les lignes par 3 nouvelles, Then `version` passe à 2, anciennes lignes supprimées, 3 nouvelles insérées avec `position = 0, 1, 2`, audit `invoice.updated` avec wrapper `{before, after}`. When conflit de version, Then 409 `OPTIMISTIC_LOCK_CONFLICT` et rollback.

5. **Suppression brouillon** — Given une facture brouillon, When suppression, Then la facture et ses lignes sont supprimées (cascade), audit `invoice.deleted` avec snapshot complet. Given une facture `validated` ou `cancelled` (statut défensif pour 5.2), When tentative suppression, Then 409 `ILLEGAL_STATE_TRANSITION`.

6. **Au moins une ligne requise** — Given la création/modification avec `lines = []`, Then 400 `INVALID_INPUT` avec message i18n « Une facture doit contenir au moins une ligne ».

7. **Validation TVA whitelist** (partagée avec products) — Given une ligne avec `vat_rate = 99.99`, Then 400 « Taux TVA non autorisé. Valeurs acceptées : 0.00%, 2.60%, 3.80%, 8.10% ».

8. **Validation quantité positive** — Given une ligne avec `quantity = 0` ou `quantity = -1`, Then 400 « La quantité doit être strictement positive ». CHECK DB en défense en profondeur.

9. **Validation contact existant** — Given un `contact_id` inexistant ou appartenant à une autre company, Then 400 `INVALID_INPUT` (pas 500).

10. **Pré-remplissage `payment_terms` depuis contact** (FR28 câblage) — Given un contact avec `default_payment_terms = "30 jours net"`, When création d'une facture pour ce contact, Then le champ `payment_terms` du formulaire est pré-rempli avec « 30 jours net ». L'utilisateur peut modifier avant enregistrement. La valeur persistée est celle saisie au moment de l'enregistrement (snapshot).

11. **Liste paginée + tri + filtres** — Given 30 factures, When chargement `/invoices`, Then 20 premières triées par `date DESC, id DESC`. Tri cliquable sur date, total_amount, contact_name. Filtres : `status`, `contact_id`, `date_from`/`date_to`. Pagination offset/limit. URL reflète l'état.

12. **Recherche debouncée** — Given la liste, When saisie dans le champ recherche, Then après 300ms debounce, filtre LIKE sur `invoice_number` (futur), `payment_terms`, `contact.name` via JOIN.

13. **URL state préservé après reload** — Given état filtré/paginé/trié, When refresh, Then état restauré depuis les query params.

14. **RBAC** — GETs (`authenticated_routes`), mutations (`comptable_routes`). Un utilisateur `readonly` peut lister/voir, pas créer/modifier/supprimer.

15. **i18n** — Tous labels, messages, erreurs, options internationalisés × 4 langues (fr-CH, de-CH, it-CH, en-CH).

16. **Audit log complet** — 3 mutations (create, update, delete) écrivent audit atomiquement. Rollback explicite en cas d'échec audit (pattern contacts/products).

17. **Notifications** — `notifySuccess`/`notifyError` pour tous les feedbacks utilisateur.

18. **Formatage suisse** — Tous les montants (`unit_price`, `line_total`, `total_amount`) affichés avec `formatSwissAmount` (apostrophe U+2019, 2 décimales CHF). Helper réutilisé, pas dupliqué.

19. **Tests** — Rust DB + unit handler + Vitest + Playwright.

## Tasks / Subtasks

### T1 — Migration & entités (AC: #1, #2, #3, #5, #8)

- [ ] T1.1 Créer `crates/kesh-db/migrations/20260416000001_invoices.sql` (vérifier le numéro avant — doit être > `20260415000001`) :
  ```sql
  CREATE TABLE invoices (
      id BIGINT NOT NULL AUTO_INCREMENT,
      company_id BIGINT NOT NULL,
      contact_id BIGINT NOT NULL,
      invoice_number VARCHAR(64) NULL,
      status VARCHAR(16) NOT NULL DEFAULT 'draft',
      date DATE NOT NULL,
      due_date DATE NULL,
      payment_terms VARCHAR(255) NULL,
      total_amount DECIMAL(19,4) NOT NULL DEFAULT 0,
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      PRIMARY KEY (id),
      CONSTRAINT fk_invoices_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
      CONSTRAINT fk_invoices_contact FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE RESTRICT,
      CONSTRAINT chk_invoices_status CHECK (status IN ('draft', 'validated', 'cancelled')),
      CONSTRAINT chk_invoices_total_non_negative CHECK (total_amount >= 0),
      INDEX idx_invoices_company_status (company_id, status),
      INDEX idx_invoices_company_date (company_id, date),
      INDEX idx_invoices_contact (contact_id)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

  CREATE TABLE invoice_lines (
      id BIGINT NOT NULL AUTO_INCREMENT,
      invoice_id BIGINT NOT NULL,
      position INT NOT NULL,
      description VARCHAR(1000) NOT NULL,
      quantity DECIMAL(19,4) NOT NULL,
      unit_price DECIMAL(19,4) NOT NULL,
      vat_rate DECIMAL(5,2) NOT NULL,
      line_total DECIMAL(19,4) NOT NULL,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      PRIMARY KEY (id),
      CONSTRAINT fk_invoice_lines_invoice FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
      CONSTRAINT chk_invoice_lines_quantity_positive CHECK (quantity > 0),
      CONSTRAINT chk_invoice_lines_unit_price_non_negative CHECK (unit_price >= 0),
      CONSTRAINT chk_invoice_lines_vat_rate_range CHECK (vat_rate >= 0 AND vat_rate <= 100),
      CONSTRAINT chk_invoice_lines_description_not_empty CHECK (CHAR_LENGTH(TRIM(description)) > 0),
      CONSTRAINT uq_invoice_lines_position UNIQUE (invoice_id, position),
      INDEX idx_invoice_lines_invoice (invoice_id)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
  ```
  **Note** : la clause `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci` est OBLIGATOIRE (cf. pattern products) — sinon MariaDB 11.x divergerait avec `uca1400_ai_ci`.

- [ ] T1.2 Créer `crates/kesh-db/src/entities/invoice.rs` :
  - `pub struct Invoice { id, company_id, contact_id, invoice_number (Option<String>), status (String), date (chrono::NaiveDate), due_date (Option<NaiveDate>), payment_terms (Option<String>), total_amount (Decimal), version, created_at, updated_at }` avec `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]` + `#[serde(rename_all = "camelCase")]`.
  - `pub struct NewInvoice { company_id, contact_id, date, due_date, payment_terms, lines (Vec<NewInvoiceLine>) }`.
  - `pub struct InvoiceUpdate { contact_id, date, due_date, payment_terms, lines (Vec<NewInvoiceLine>) }`.
  - `pub struct InvoiceLine { id, invoice_id, position, description, quantity, unit_price, vat_rate, line_total, created_at }` + FromRow.
  - `pub struct NewInvoiceLine { description, quantity, unit_price, vat_rate }` (pas de `position` — calculée par le repository, pas de `line_total` — calculé par le repository).
- [ ] T1.3 Ajouter `pub mod invoice;` + re-exports dans `entities/mod.rs`.

### T2 — Repository `invoices` (AC: #1, #2, #3, #4, #5, #6, #8, #16)

- [ ] T2.1 Créer `crates/kesh-db/src/repositories/invoices.rs` — pattern contacts/products, avec spécificités :
  - `invoice_snapshot_json(&Invoice, &[InvoiceLine]) -> serde_json::Value` — entête + lignes (tableau `[{position, description, quantity, unitPrice, vatRate, lineTotal}]`), montants en string décimal via `to_string()`.
  - `escape_like` — dupliquer.
  - `InvoiceSortBy { Date, TotalAmount, ContactName, CreatedAt }`.
  - `InvoiceListQuery { search, status, contact_id, date_from, date_to, sort_by, sort_direction, limit, offset }`.
  - `InvoiceListResult { items (Vec<InvoiceListItem>), total, offset, limit }`. **`InvoiceListItem`** = projection légère sans les lignes (juste entête + `contact_name` via JOIN) pour optimiser la liste.
  - Fonctions principales : `create(pool, NewInvoice, actor_user_id) -> Invoice`, `find_by_id_with_lines(pool, id, company_id) -> (Invoice, Vec<InvoiceLine>)`, `list_by_company_paginated`, `update(pool, id, company_id, expected_version, InvoiceUpdate, actor) -> Invoice`, `delete(pool, id, company_id, actor)`.
  - **`create`** : transaction → INSERT invoice → calculer `total_amount` = Σ (qty × unit_price) → UPDATE invoices SET total_amount → INSERT lines (avec `position = 0..N`, `line_total = qty × unit_price`) → INSERT audit log → commit. **Rollback explicite** si audit échoue.
  - **`update`** : **pattern aligné avec `products.rs`** — SELECT initial optimiste (sans `FOR UPDATE`) pour charger l'entité courante + construire le snapshot `before`. Vérifier `status == 'draft'` dans le repository → sinon `DbError::IllegalStateTransition` (pattern identique à la vérification `active` dans products.rs). Puis transaction : DELETE invoice_lines WHERE invoice_id → INSERT nouvelles lignes → `UPDATE invoices SET ..., version = version + 1 WHERE id = ? AND version = ?` → si `rows_affected == 0` → `DbError::OptimisticLockConflict` → audit wrapper `{before, after}` → commit. **NE PAS** utiliser `SELECT ... FOR UPDATE` dans `update` (cohérence avec products.rs).
  - **`delete`** : transaction → **`SELECT ... FOR UPDATE`** (justifié ici pour garantir l'atomicité snapshot + vérification statut + suppression) → vérifier `status == 'draft'` → snapshot avant suppression → DELETE invoices (CASCADE supprime les lignes) → audit `invoice.deleted` → commit.
  - **`list_by_company_paginated`** : JOIN contacts (alias `c`) : `FROM invoices i INNER JOIN contacts c ON c.id = i.contact_id`. SELECT inclut `c.name AS contact_name`. `InvoiceSortBy::ContactName.as_sql_column()` retourne littéralement `"c.name"` (pas `"contact_name"` — l'alias dans ORDER BY fonctionne sur MariaDB mais une colonne qualifiée est plus robuste). Whitelist SQL stricte par enum variant, pas de string concat. `push_where_clauses` avec filtres status/contact_id/date range + search LIKE sur `i.payment_terms` et `c.name`.
- [ ] T2.2 Ajouter `pub mod invoices;` dans `repositories/mod.rs`.

### T3 — API routes `/api/v1/invoices` (AC: #1, #4, #5, #6, #7, #8, #9, #11, #12, #14, #17)

- [ ] T3.1 Créer `crates/kesh-api/src/routes/vat.rs` — extraire `allowed_vat_rates()` depuis `products.rs` :
  ```rust
  use rust_decimal::Decimal;
  use std::sync::LazyLock;

  static ALLOWED_VAT_RATES: LazyLock<Vec<Decimal>> = LazyLock::new(|| {
      ["0.00", "2.60", "3.80", "8.10"]
          .iter()
          .map(|s| Decimal::from_str(s).expect("vat rate literal must parse"))
          .collect()
  });

  pub fn allowed_vat_rates() -> &'static [Decimal] {
      &ALLOWED_VAT_RATES
  }

  pub fn validate_vat_rate(rate: &Decimal) -> bool {
      allowed_vat_rates().contains(rate)
  }
  ```
  Mettre à jour `products.rs` pour importer depuis `crate::routes::vat` (DRY). Refactor non-breaking.
- [ ] T3.2 Créer `crates/kesh-api/src/routes/invoices.rs` avec DTOs + 5 handlers :
  - `CreateInvoiceRequest { contactId, date, dueDate, paymentTerms, lines: Vec<CreateInvoiceLineRequest> }`.
  - `UpdateInvoiceRequest { contactId, date, dueDate, paymentTerms, lines: Vec<CreateInvoiceLineRequest>, version }`.
  - `CreateInvoiceLineRequest { description, quantity, unitPrice, vatRate }`.
  - `InvoiceResponse` (full, avec lignes), `InvoiceListItemResponse` (léger, sans lignes, avec `contactName`).
  - `ListInvoicesQuery { search, status, contactId, dateFrom, dateTo, sortBy, sortDirection, limit, offset }`.
  - Handlers : `list_invoices`, `get_invoice` (avec lines), `create_invoice`, `update_invoice`, `delete_invoice`.
  - **Validation request** :
    - `lines.is_empty()` → 400 `INVALID_INPUT` (« Au moins une ligne requise »).
    - Pour chaque ligne :
      - `description` : non-vide (après trim), ≤ 1000 chars, **normalisé NFC** via `unicode_normalization::UnicodeNormalization::nfc()` avant persistance (cohérence avec products.rs qui normalise `name`/`description` en NFC pour éviter les collisions CHECK/LIKE entre formes composée/décomposée — obligatoire pour les saisies macOS qui produisent du NFD).
      - `quantity > Decimal::ZERO`, `quantity <= 1_000_000` (plafond anti-overflow `qty × unit_price`), **scale ≤ 4** (sinon 400 — évite troncature silencieuse en `DECIMAL(19,4)`).
      - `unit_price >= Decimal::ZERO`, `unit_price <= 1_000_000_000` (même plafond que products — réutiliser la constante `MAX_UNIT_PRICE` via import depuis `routes::limits` ou équivalent ; si pas encore extrait, **extraire `MAX_UNIT_PRICE` et la validation de scale dans un module partagé `routes/limits.rs` dès cette story** et refactorer products pour l'importer). **Scale ≤ 4**.
      - `vat::validate_vat_rate(&line.vat_rate)` — **NE PAS valider le scale de `vat_rate`** : `Decimal::eq` ignore le scale, donc `8.1 == 8.10 == 8.100` côté comparaison whitelist (cohérent avec products.rs).
    - `payment_terms` : si `Some`, normaliser NFC, ≤ 255 chars, trim.
  - **Validation contact** : vérifier que `contact_id` appartient à la même `company_id` ET que `contact.active == true` → sinon 400 `INVALID_INPUT` (« Contact introuvable »).
  - Mapping erreurs : `DbError::IllegalStateTransition` → 409 `ILLEGAL_STATE_TRANSITION`, `DbError::OptimisticLockConflict` → 409 `OPTIMISTIC_LOCK_CONFLICT`.
- [ ] T3.3 Enregistrer routes : GETs dans `authenticated_routes`, mutations dans `comptable_routes`.
- [ ] T3.4 Ajouter `pub mod invoices; pub mod vat;` dans `routes/mod.rs`.

### T4 — Frontend feature `invoices` (AC: #1, #2, #3, #4, #5, #10, #11, #12, #13, #15, #17, #18)

- [ ] T4.1 Créer `frontend/src/lib/features/invoices/` avec :
  - `invoices.types.ts` — DTOs TypeScript alignés sur l'API (`InvoiceResponse`, `InvoiceListItemResponse`, `CreateInvoiceRequest`, `UpdateInvoiceRequest`, `InvoiceLine`, `CreateInvoiceLineRequest`, `ListInvoicesQuery`).
  - `invoices.api.ts` — 5 fonctions (listInvoices, getInvoice, createInvoice, updateInvoice, deleteInvoice).
  - `invoice-helpers.ts` — `computeLineTotal(qty: string, price: string): string` (via big.js), `computeInvoiceTotal(lines): string`, `formatInvoiceTotal` qui délègue à `formatSwissAmount` (DRY, **pas de duplication**).
  - `invoice-helpers.test.ts` — tests Vitest précision décimale (ex: `0.1 + 0.2 === "0.30"`).
- [ ] T4.2 Créer `/invoices/+page.svelte` — page liste (table contactName, date, total, status, actions). Filtres : search, status select, contact combobox, date range. `onMount` URL init. Debounce 300ms. `formatSwissAmount` pour total_amount.
- [ ] T4.3 Créer `/invoices/new/+page.svelte` et `/invoices/[id]/edit/+page.svelte` — formulaire commun (composant partagé `InvoiceForm.svelte`) avec :
  - Entête : contact combobox (`ContactPicker.svelte`), date, due_date, payment_terms (préfill depuis contact).
  - Tableau lignes `$state<Line[]>` : boutons « Ajouter ligne libre » + « Ajouter depuis catalogue ». Par ligne : description (input), quantity, unit_price (inputmode="decimal"), vat_rate (select whitelist), line_total (calculé, readonly), bouton supprimer.
  - Total en bas : `computeInvoiceTotal(lines)` affiché via `formatSwissAmount`.
  - Validation front avant submit : ≥ 1 ligne, chaque ligne valide, contact sélectionné.
  - Submit → POST/PUT → redirect `/invoices` avec toast success.
  - Erreur 409 `OPTIMISTIC_LOCK_CONFLICT` → modale reload.
- [ ] T4.4 Créer `ProductPicker.svelte` (dialog imbriqué) — liste produits cherchable (appelle `listProducts`), sélection → callback qui ajoute une ligne au formulaire avec snapshot complet.
- [ ] T4.5 Créer `ContactPicker.svelte` (combobox) — dropdown cherchable appelant `listContacts({ search, limit: 50 })` debounced.
- [ ] T4.6 Créer `/invoices/[id]/+page.svelte` — vue lecture seule avec bouton « Modifier » (si draft) et « Supprimer » (dialog de confirmation).
- [ ] T4.7 Ajouter lien sidebar « Factures » dans `+layout.svelte` navGroups.

### T5 — Clés i18n (AC: #15)

- [ ] T5.1 Ajouter ~50 clés × 4 langues :
  - Nav : `nav-invoices`.
  - Pages : `invoices-page-title`, `invoice-new-title`, `invoice-edit-title`, `invoice-view-title`.
  - Labels entête : `invoice-form-contact`, `invoice-form-date`, `invoice-form-due-date`, `invoice-form-payment-terms`, `invoice-form-status`, `invoice-form-number`.
  - Labels lignes : `invoice-line-description`, `invoice-line-quantity`, `invoice-line-unit-price`, `invoice-line-vat-rate`, `invoice-line-total`, `invoice-line-actions`, `invoice-add-free-line`, `invoice-add-from-catalog`.
  - Table liste : `invoice-col-date`, `invoice-col-contact`, `invoice-col-number`, `invoice-col-status`, `invoice-col-total`, `invoice-col-actions`.
  - Statuts : `invoice-status-draft`, `invoice-status-validated`, `invoice-status-cancelled`.
  - Filtres/boutons : `invoice-filter-search`, `invoice-filter-status-all`, `invoice-filter-contact-all`, `invoice-filter-date-from`, `invoice-filter-date-to`, `invoice-new-button`, `invoice-edit-button`, `invoice-delete-button`.
  - Totaux : `invoice-subtotal`, `invoice-total`.
  - Messages : `invoice-empty-list`, `invoice-created-success`, `invoice-updated-success`, `invoice-deleted-success`, `invoice-delete-confirm-title`, `invoice-delete-confirm-body`, `invoice-conflict-title`, `invoice-conflict-body`.
  - Erreurs : `invoice-error-no-lines`, `invoice-error-contact-required`, `invoice-error-contact-invalid`, `invoice-error-quantity-positive`, `invoice-error-description-required`, `invoice-error-vat-invalid` (réutilisable depuis products), `invoice-error-illegal-state`.
  - ProductPicker : `invoice-product-picker-title`, `invoice-product-picker-search`, `invoice-product-picker-empty`.
  - ContactPicker : `invoice-contact-picker-placeholder`, `invoice-contact-picker-empty`.

### T6 — Tests (AC: #19)

- [ ] T6.0 **Helpers fixtures** dans `invoices::tests` (module privé) : `create_test_contact(pool, company_id) -> (i64, Contact)` (nom préfixé `"TestInvoiceContact_"` + uuid court, IDE optionnel null, active=true) et `create_test_product(pool, company_id) -> (i64, Product)` (nom préfixé `"TestInvoiceProduct_"`, unit_price/vat_rate fixés). `cleanup_test_invoices(pool, company_id)` : `DELETE FROM invoices WHERE company_id = ? AND id IN (SELECT ...)` — isoler via un préfixe sur `payment_terms` (`"TestInvoice_"`) ou via les IDs retournés par les helpers (plus fiable). Pattern strictement calqué sur `cleanup_test_products` dans products.rs.
- [ ] T6.1 Tests d'intégration DB `invoices::tests` — pattern contacts/products :
  - `test_create_with_lines_computes_total` — vérifie `total_amount` et `line_total` après INSERT.
  - `test_create_writes_audit_log` — audit atomique avec snapshot entête + lignes.
  - `test_update_replaces_all_lines` — 2 lignes initiales, update avec 3 nouvelles, vérifier anciennes supprimées et nouvelles positions (0,1,2).
  - `test_update_optimistic_lock_conflict`.
  - `test_update_rejects_non_draft` — modifier à la main status='validated' via direct SQL, puis update → `IllegalStateTransition`.
  - `test_update_writes_audit_log_wrapper` — `{before, after}` contenant les deux versions complètes.
  - `test_delete_cascades_lines` — DELETE FK CASCADE vérifié empiriquement.
  - `test_delete_rejects_non_draft`.
  - `test_delete_writes_audit_log`.
  - `test_list_filters_by_status_and_date_range`.
  - `test_list_orders_by_date_desc_by_default`.
  - `test_find_by_id_returns_lines_ordered_by_position`.
  - `test_db_rejects_quantity_zero_via_direct_insert` — CHECK defense in depth.
  - `test_db_rejects_invalid_status_via_direct_update`.
  - **`test_delete_contact_rejected_when_has_invoices`** — ce test vit dans `contacts::tests` (ou dans un fichier d'intégration dédié) : créer un contact, créer une facture pour ce contact, tenter `DELETE /api/v1/contacts/:id` → vérifier 409. Assure qu'on n'introduit pas de régression sur Story 4.1 par la FK `ON DELETE RESTRICT`.
- [ ] T6.2 Tests unit handlers `invoices::tests` (module `#[cfg(test)]`) : validation `lines = []`, validation TVA whitelist (réutilisation `vat::validate_vat_rate`), validation contact_id cross-company.
- [ ] T6.3 Tests Vitest `invoice-helpers.test.ts` : `computeLineTotal` précision, `computeInvoiceTotal` avec 3 lignes, arrondis.
- [ ] T6.4 Tests Playwright `invoices.spec.ts` :
  - Création facture avec 1 ligne libre + 1 ligne catalogue → vérifier total et persistance après reload.
  - Modification : ajouter/supprimer une ligne → total mis à jour.
  - Suppression brouillon.
  - Filtre par contact.
  - URL state après refresh.
  - Pré-remplissage `payment_terms` depuis contact.

### T7 — Validation finale

- [ ] T7.1 `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] T7.2 `cargo check --workspace --tests`.
- [ ] T7.3 `npm run test:unit` (full suite frontend).
- [ ] T7.4 `npm run check` (svelte-check, 0 errors).
- [ ] T7.5 Test manuel : créer une facture, reload, vérifier que tout est intact.
- [ ] T7.6 Mettre à jour sprint-status → `review`.

## Dev Notes

### Architecture — où va quoi

```
kesh-db/
├── migrations/20260416000001_invoices.sql               # T1.1
└── src/
    ├── entities/
    │   ├── invoice.rs                                   # T1.2
    │   └── mod.rs                                       # T1.3
    └── repositories/
        ├── invoices.rs                                  # T2 (CRUD + audit + tests)
        └── mod.rs                                       # T2.2

kesh-api/src/routes/
├── vat.rs                                               # T3.1 (helper partagé DRY)
├── invoices.rs                                          # T3.2 (5 handlers + tests unit)
├── products.rs                                          # refactor T3.1 (import vat)
└── mod.rs                                               # T3.4

frontend/src/lib/features/invoices/
├── invoices.types.ts                                    # T4.1
├── invoices.api.ts                                      # T4.1
├── invoice-helpers.ts                                   # T4.1
└── invoice-helpers.test.ts                              # T4.1

frontend/src/lib/components/
├── ContactPicker.svelte                                 # T4.5
└── ProductPicker.svelte                                 # T4.4

frontend/src/lib/components/invoices/
└── InvoiceForm.svelte                                   # T4.3 (composant partagé)

frontend/src/routes/(app)/invoices/
├── +page.svelte                                         # T4.2 liste
├── new/+page.svelte                                     # T4.3
├── [id]/+page.svelte                                    # T4.6 vue lecture
└── [id]/edit/+page.svelte                               # T4.3

frontend/tests/e2e/invoices.spec.ts                      # T6.4
```

### Ce qui existe DÉJÀ — NE PAS refaire

- **`rust_decimal::Decimal`** — dépendance et features déjà configurées.
- **`ListResponse<T>`** — `routes/mod.rs:25`, utilisé par journal_entries/contacts/products.
- **Pattern audit atomique** — `contacts.rs`/`products.rs` sont les modèles. Rollback explicite si `audit_log::insert` échoue.
- **Whitelist TVA** — dans `products.rs`, à extraire en `vat.rs` (T3.1) et réutiliser.
- **`escape_like`** — dupliqué dans contacts/products. Dupliquer une 3e fois OU extraire en helper `repositories/shared.rs` au choix du dev (faire le choix cohérent avec la politique DRY du projet — si extraction, mettre à jour contacts/products aussi dans ce commit).
- **`formatSwissAmount`** — dans `balance.ts` (Story 3.2) ou `$lib/shared/utils/format-decimal.ts` si déjà extrait par 4.2. Réutiliser tel quel.
- **`notifySuccess`/`notifyError`** — helpers globaux.
- **`i18nMsg` canonical** — pattern établi depuis Story 2.1.
- **Pattern `onMount` URL init** (pas `$effect`) — fix P1 code review 4.1, appliqué en 4.2.
- **Pattern cleanup debounce dans `onMount` return** — P6 code review 4.2.
- **Type `Combobox`** — vérifier si composant UI existe déjà (Melt UI / bits-ui via shadcn-svelte). Sinon implémenter un `ContactPicker` simple : `<input>` + liste déroulante absolute-positioned, navigation flèches.

### Points de vigilance (prévention LLM)

1. **NE PAS ajouter de FK `product_id` aux lignes** — décision de conception explicite (snapshot immuable). Les lignes stockent `description`/`unit_price`/`vat_rate` en dur.
2. **NE PAS utiliser l'enum SQLx pour `status`** — cf. `feedback_sqlx_mysql_gotchas` (mapping manuel, laisser `status: String` avec CHECK DB).
3. **NE PAS calculer `total_amount` côté frontend pour la persistance** — le backend est la source de vérité. Le frontend affiche en live, mais le backend recalcule à chaque mutation.
4. **NE PAS dupliquer la whitelist TVA** — extraire en `vat.rs` (T3.1), refactorer products en conséquence.
5. **NE PAS utiliser `Intl.NumberFormat('de-CH')`** pour les montants — perte de précision sur gros nombres. Utiliser `formatSwissAmount` (big.js + apostrophe U+2019).
6. **NE PAS skipper le test `test_update_replaces_all_lines`** — vérifie le comportement replace-all (risque de régression si futur dev passe à un diff LCS).
7. **NE PAS oublier le rollback explicite** si audit échoue — pattern contacts.rs, sinon audit incohérent avec base.
8. **`FOR UPDATE` uniquement dans `delete`** (atomicité snapshot + check statut + suppression). Pour `update`, utiliser le pattern optimiste `rows_affected == 0` comme products.rs — **pas de `FOR UPDATE`**. Ne pas confondre les deux cas.
9. **Migration number** — avant de créer, faire `ls crates/kesh-db/migrations/` et prendre le numéro suivant (devrait être `20260416000001`, mais à vérifier).
10. **ENGINE+CHARSET+COLLATE** obligatoire sur les 2 CREATE TABLE — sinon MariaDB 11 diverge.
11. **Test Playwright** — isoler correctement les DB (chaque test e2e a sa propre DB via `#[sqlx::test]` côté backend, ou base fresh via seed au setup).
12. **Validation contact cross-company** — refuser `contact_id` appartenant à une autre company. Sinon 400 explicite, pas 500.
13. **`chrono::NaiveDate` vs `DateTime`** — `date` et `due_date` sont `NaiveDate`. Sérialisation JSON : ISO 8601 (`"2026-04-14"`). Vérifier `serde` + `chrono` features si besoin.

### Previous Story Intelligence (Story 4.2)

Learnings à appliquer :

- **3 passes de code review (Opus → Sonnet → Haiku)** ont trouvé 22 patches (P1–P22) sur 4.2. Les plus pertinents pour 5.1 :
  - **P1** : `onMount` pour lecture URL initiale (**pas `$effect`**) — éviter boucles infinies.
  - **P6** : cleanup debounce dans `onMount` return — éviter fuite mémoire.
  - **P11** : validation TVA en `Decimal` natif (pas string-to-string) — réutiliser directement `vat::validate_vat_rate`.
  - **P14** : `LazyLock` pour constantes parsées (pas `lazy_static`).
  - **P19** : test de défense en profondeur sur CHECK DB (direct INSERT bypass handler).
- **Naming snake_case SQL → camelCase JSON** via `#[serde(rename_all = "camelCase")]` — **obligatoire** sur chaque DTO.
- **`rust_decimal` feature `serde-str`** gère auto la sérialisation string → NE PAS manuellement `to_string()` dans les `From<Entity>` impl.
- **Flakiness cross-binary SQLx** (memory `feedback_sqlx_mysql_gotchas`) → CI tourne avec `-j1 -- --test-threads=1`. Les nouveaux tests d'invoices doivent respecter ce modèle.

### References

- Epic & stories source : [`_bmad-output/planning-artifacts/epics.md#Epic-5`](../planning-artifacts/epics.md) (lignes 871–920).
- PRD FR31–FR35 : [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md).
- Architecture — structure `kesh-db`/`kesh-api` : [`architecture.md`](../planning-artifacts/architecture.md#Crates) (lignes 440–490).
- Pattern repository canonique : `crates/kesh-db/src/repositories/contacts.rs` (Story 4.1), `products.rs` (Story 4.2).
- Pattern route + DTO : `crates/kesh-api/src/routes/products.rs` (Story 4.2).
- Pattern feature frontend : `frontend/src/lib/features/products/`, `frontend/src/routes/(app)/products/+page.svelte`.
- Whitelist TVA source : `crates/kesh-api/src/routes/products.rs` (à refactorer).
- Règle de remédiation revues : `CLAUDE.md` § « Règle de remédiation des revues ».

## Dev Agent Record

### Agent Model Used

_TBD after implementation_

### Debug Log References

### Completion Notes List

### File List
