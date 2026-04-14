# Story 5.2: Validation & numérotation des factures

Status: review

<!-- Validation optionnelle via validate-create-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **valider une facture brouillon pour qu'elle reçoive un numéro définitif, devienne immuable et génère automatiquement l'écriture comptable correspondante**,
so that **la facture soit officielle, comptabilisée, et prête à être envoyée au client (PDF QR Bill en 5.3)**.

### Contexte

**Deuxième story de l'Epic 5 — Facturation QR Bill**. Couvre **FR33** (numérotation séquentielle à la validation), **FR35** (format de numérotation configurable), et le câblage écriture comptable de **FR34** (création auto débit client / crédit produit). La génération PDF elle-même est Story 5.3.

**Fondations déjà en place** (Story 5.1 et antérieures) :

- **Tables `invoices` / `invoice_lines`** — migration `20260416000001_invoices.sql`. Colonnes pertinentes : `status VARCHAR(16) CHECK IN ('draft','validated','cancelled')`, `invoice_number VARCHAR(64) NULL`, `total_amount DECIMAL(19,4)`, `version INT`. Aucun index unique sur `invoice_number` encore (volontaire — à ajouter en 5.2).
- **Repository `invoices`** — `crates/kesh-db/src/repositories/invoices.rs`. `update`/`delete` refusent déjà les factures `status != 'draft'` (`DbError::IllegalStateTransition`). Audit wrapper `{before, after}` sur update, snapshot direct sur create/delete.
- **Tables `journal_entries` / `journal_entry_lines`** — migration `20260412000001_journal_entries.sql`. Contraintes : `UNIQUE(company_id, fiscal_year_id, entry_number)`, `entry_number BIGINT > 0`, `journal CHECK IN ('Achats','Ventes','Banque','Caisse','OD')` (BINARY). Story 3.2 a défini le repository `journal_entries.rs` : pattern CRUD + audit, équilibre débit/crédit validé côté application via `kesh-core`.
- **Table `fiscal_years`** — migration initiale `20260404000001_initial_schema.sql`. Colonnes : `id, company_id, name, start_date, end_date, status (Open/Closed)`. La validation d'une facture doit résoudre le fiscal_year à partir de `invoice.date` et refuser si exercice clôturé (cohérent avec FR24 étendu au flux facturation).
- **Table `accounts`** — migration `20260411000001_accounts.sql`. `account_type IN ('Asset','Liability','Revenue','Expense')`. Les comptes de créances clients (Asset, ex. 1100) et de produits/ventes (Revenue, ex. 3000/3200) existent déjà dans le plan chargé (Story 3.1).
- **Pattern audit atomique + rollback** — `contacts.rs` / `products.rs` / `invoices.rs` (Story 5.1). `DbError::IllegalStateTransition` / `OptimisticLockConflict` déjà mappés sur 409 dans les handlers.
- **Whitelist TVA partagée** — `crates/kesh-api/src/routes/vat.rs` (Story 5.1 T3.1). `MAX_UNIT_PRICE`, `MAX_QUANTITY` dans `routes/limits.rs`.
- **Pattern frontend feature `invoices/`** — types, api, helpers, pages liste + new + [id] + [id]/edit. La vue `/invoices/[id]/+page.svelte` dispose déjà du bouton « Modifier » (draft) et « Supprimer ». **Ajouter bouton « Valider »** visible uniquement si `status === 'draft'`.
- **i18n FTL** — ~58 clés factures déjà ajoutées × 4 langues. Il faudra **ajouter** les clés liées à la validation, la config de format, et l'écriture générée.

### Scope verrouillé — ce qui DOIT être fait

1. **Config format de numérotation** — une configuration par `company_id` (pas globale). Stockage dans une **nouvelle table `company_invoice_settings`** (préférée à une colonne sur `companies` pour éviter d'impacter la story 1.4 et pour ouvrir la voie à d'autres réglages facturation ultérieurs : `default_revenue_account_id`, `default_receivable_account_id`, `default_journal`, etc.).
2. **Table `invoice_number_sequences`** — compteur séquentiel par `(company_id, fiscal_year_id)` avec contrainte UNIQUE. Incrémenté atomiquement lors de la validation (UPDATE + SELECT dans une transaction avec `FOR UPDATE`). Pas d'auto-increment SQL (risque de trou en cas de rollback).
3. **Transition d'état `draft → validated`** — endpoint dédié `POST /api/v1/invoices/:id/validate`. Atomique : attribution numéro + insertion écriture comptable + passage `status = 'validated'` + audit, le tout dans une seule transaction avec rollback explicite en cas d'échec.
4. **Génération écriture comptable** — création automatique d'une `journal_entry` dans le journal `Ventes` (FR). Lignes : 1 ligne débit sur le **compte de créance** (`default_receivable_account_id` config), N lignes crédit par regroupement par **taux de TVA** × **compte de produit** (`default_revenue_account_id` config, identique pour toutes les lignes en 5.2 — la ventilation par compte par produit arrive en Epic 9/10). Équilibre débit/crédit strict : total débit = `total_amount` = somme des crédits.
5. **Immutabilité post-validation** — une facture `validated` refuse toute mutation (update/delete) : déjà implémenté en 5.1 pour `update`/`delete`. **Ajouter** : refus d'un second `validate` (409 `ILLEGAL_STATE_TRANSITION`), refus côté UI (pas de bouton « Modifier » quand validée).
6. **Index unique conditionnel sur `invoice_number`** — migration qui ajoute `UNIQUE(company_id, invoice_number)` **seulement pour les lignes où `invoice_number IS NOT NULL`**. MariaDB n'a pas de partial index natif ; utiliser une **colonne générée** `invoice_number_unique_key` = `IFNULL(invoice_number, CONCAT('__draft__', id))` avec `UNIQUE(company_id, invoice_number_unique_key)`. Alternative acceptée : simple `UNIQUE(company_id, invoice_number)` sachant que MariaDB accepte plusieurs NULL dans un index UNIQUE (standard SQL). **Retenu : index simple `UNIQUE(company_id, invoice_number)` — MariaDB autorise N NULL (vérifié).** Pas besoin de colonne générée.
7. **API routes** — 2 nouveaux endpoints :
   - `POST /api/v1/invoices/:id/validate` (comptable_routes) — transition atomique.
   - `GET /api/v1/company/invoice-settings` (authenticated_routes) — lire config.
   - `PUT /api/v1/company/invoice-settings` (admin_routes — config company = rôle Admin uniquement) — modifier config (format, default_receivable_account_id, default_revenue_account_id, default_journal).
8. **Page de configuration facturation** — `/settings/invoicing/+page.svelte`, accessible aux Admin. Formulaire : format numérotation (input texte avec placeholders `{YEAR}`, `{SEQ}`, `{SEQ:04}`, `{SEQ:06}`), sélecteur compte client (filtré sur `account_type = 'Asset'`), sélecteur compte produit (filtré sur `account_type = 'Revenue'`), sélecteur journal (dropdown fixe 5 valeurs). Pré-remplissage avec valeurs sensibles par défaut si l'onboarding ne les a pas posées.
9. **Bouton « Valider »** sur `/invoices/[id]/+page.svelte` (vue détail) et sur `/invoices/[id]/edit/+page.svelte` (après sauvegarde draft). Dialog de confirmation : « Une fois validée, cette facture sera immuable et générera une écriture comptable. Continuer ? ». POST validate → redirection vers vue lecture seule mise à jour (avec le nouveau numéro).
10. **i18n** — ~30 clés supplémentaires × 4 langues.
11. **Tests** — Rust DB (transaction atomicité + concurrence), unit handlers, Vitest (format preview), Playwright (validation E2E golden path).
12. **`invoices.due_date` (v0.6 — clarification, PAS de nouvelle colonne)** — la colonne `due_date DATE NULL` **existe déjà** depuis Story 5.1 (migration `20260416000001_invoices.sql`). La 5.2 la rend **pertinente** : (a) backend `create_invoice` : si non fournie → `due_date = invoice.date` (copie côté serveur, défaut pragmatique demandé par Guy). L'utilisateur peut override (date +N jours selon ses conditions de paiement). (b) UI `/invoices/new` et `/invoices/[id]/edit` : soigner l'affichage des 2 champs (date de facture + date de valeur/échéance). Pas de calcul auto lié à `payment_terms` (trop fragile, demande explicite de Guy : simple défaut). **Non utilisée** par `validate_invoice` en 5.2 (l'écriture comptable reste datée sur `invoice.date`). Préparatoire Epic 6 / Story 5.4 échéancier.
13. **Template libellé écriture configurable (v0.6)** — `company_invoice_settings.journal_entry_description_template VARCHAR(128) NOT NULL DEFAULT '{YEAR}-{INVOICE_NUMBER}'`. Placeholders supportés : `{YEAR}` (année du fiscal_year), `{INVOICE_NUMBER}`, `{CONTACT_NAME}`. Validé à l'écriture (PUT /invoice-settings) : whitelist placeholders, longueur ≤ 128, au moins 1 placeholder. Rendu atomique via `String::replace` dans `validate_invoice`. Remplace le libellé hard-codé.

### Scope volontairement HORS story — décisions tranchées

- **Génération PDF QR Bill** → Story 5.3.
- **Annulation par avoir / contre-passation** → Epic 10 (avoirs v0.2). En 5.2, une facture `validated` ne peut JAMAIS repasser `draft` ni `cancelled` via l'API standard. Seul un avoir (5.2+ → 10.1) pourra l'annuler comptablement.
- **Ventilation TVA dans l'écriture comptable** — en 5.2, la ligne crédit est **agrégée** (un seul compte Revenue pour toute la facture, TVA non séparée). La ventilation propre (ligne 3000 HT + ligne 2200/2201 TVA due) arrive en **Epic 9 — TVA Suisse**. Décision explicite : ne pas anticiper la structure TVA pour ne pas reprendre le modèle en 9.1.
- **Compte de produit par ligne** (1 produit catalogue = 1 compte revenue distinct) → futur (Epic 9 ou plus tard). En 5.2, toutes les lignes vont au même `default_revenue_account_id`.
- **Numérotation multi-séquences** (ex. séquence distincte par journal, par type de document) → hors scope. Une seule séquence par exercice.
- **Reset manuel du compteur** → hors scope. Le compteur repart à 1 automatiquement à chaque nouvel exercice créé (row créée à la demande lors de la première validation de l'exercice).
- **Prévisualisation du numéro avant validation** (afficher « le prochain numéro sera F-2026-0042 ») → hors scope (nécessite lecture non transactionnelle du compteur ; ajoute de la complexité sans valeur métier — l'utilisateur voit le numéro après validation).
- **Modification du format a posteriori** — si l'utilisateur change le format après avoir validé des factures, les factures existantes conservent leur numéro historique (snapshot). Pas de renumérotation rétroactive. Couvert par le fait que `invoice_number` est persisté (pas calculé à la volée).
- **Export du format vers un template PDF** → câblé en 5.3.

### Décisions de conception

- **Table `invoice_number_sequences`** :
  ```sql
  CREATE TABLE invoice_number_sequences (
      id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
      company_id BIGINT NOT NULL,
      fiscal_year_id BIGINT NOT NULL,
      next_number BIGINT NOT NULL DEFAULT 1,
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      CONSTRAINT fk_ins_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
      CONSTRAINT fk_ins_fiscal_year FOREIGN KEY (fiscal_year_id) REFERENCES fiscal_years(id) ON DELETE RESTRICT,
      CONSTRAINT uq_ins_company_fy UNIQUE (company_id, fiscal_year_id),
      CONSTRAINT chk_ins_next_positive CHECK (next_number >= 1)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
  ```
  **Pattern d'incrémentation** (dans la transaction de validation) :
  ```
  SELECT next_number FROM invoice_number_sequences
      WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE;
  -- si 0 rows : INSERT (company_id, fiscal_year_id, next_number=1)
  -- puis retenter le SELECT FOR UPDATE
  UPDATE invoice_number_sequences SET next_number = next_number + 1, version = version + 1
      WHERE company_id = ? AND fiscal_year_id = ?;
  -- numéro attribué = next_number lu avant UPDATE
  ```
  **Raison du `FOR UPDATE`** : garantir « sans trou » absolu, même en validation concurrente. Le rollback de la transaction de validation rollback aussi le compteur (pas de trou). **Alternative rejetée** : AUTO_INCREMENT SQL — laisse des trous en cas de rollback (c'est le comportement natif InnoDB).

- **Table `company_invoice_settings`** :
  ```sql
  CREATE TABLE company_invoice_settings (
      company_id BIGINT NOT NULL PRIMARY KEY,
      invoice_number_format VARCHAR(64) NOT NULL DEFAULT 'F-{YEAR}-{SEQ:04}',
      default_receivable_account_id BIGINT NULL,
      default_revenue_account_id BIGINT NULL,
      default_sales_journal VARCHAR(10) NOT NULL DEFAULT 'Ventes',
      journal_entry_description_template VARCHAR(128) NOT NULL DEFAULT '{YEAR}-{INVOICE_NUMBER}',
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      CONSTRAINT fk_cis_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
      CONSTRAINT fk_cis_receivable FOREIGN KEY (default_receivable_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
      CONSTRAINT fk_cis_revenue FOREIGN KEY (default_revenue_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
      CONSTRAINT chk_cis_journal CHECK (BINARY default_sales_journal IN (BINARY 'Achats', BINARY 'Ventes', BINARY 'Banque', BINARY 'Caisse', BINARY 'OD')),  -- syntaxe identique à chk_journal_entries_journal (migration 20260412000001) — pattern validé en prod
      CONSTRAINT chk_cis_format_nonempty CHECK (CHAR_LENGTH(TRIM(invoice_number_format)) > 0)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
  ```
  **Raison PK = `company_id`** : relation 1-1 avec `companies`. Une seule row par company. Row créée à la volée (lazy) au premier `GET /company/invoice-settings` avec valeurs par défaut si absente (pattern « upsert read »).
  **`default_receivable_account_id` et `default_revenue_account_id` NULL** à l'install : forcent l'utilisateur Admin à les configurer avant la première validation. Le handler `validate` refuse (400 `CONFIGURATION_REQUIRED`) si l'un des deux est NULL.

- **Format de numérotation — mini-DSL** :
  - Placeholders supportés : `{YEAR}` (année du fiscal_year, extraite de `fiscal_years.start_date`), `{SEQ}` (numéro sans padding), `{SEQ:NN}` (padding zéros, **NN borné à [1, 10]** — au-delà, le numéro rendu pourrait dépasser VARCHAR(64) et 10 chiffres = 10 milliards de factures, largement suffisant), `{FY}` (nom du fiscal_year littéral, ex « 2026 » ou « 2025/2026 »).
  - Texte libre avant/après/entre les placeholders.
  - Exemples valides : `F-{YEAR}-{SEQ:04}` → `F-2026-0042`. `FACT{SEQ}` → `FACT42`. `{FY}/{SEQ:06}` → `2026/000042`.
  - Validation format côté backend (à l'écriture via `PUT /invoice-settings`) :
    - Regex de caractères autorisés : `^[A-Za-z0-9\-_/\.#\s{}:]+$`.
    - Au moins un placeholder reconnu.
    - Longueur du **template** ≤ 64.
    - Pour chaque `{SEQ:NN}` : `1 <= NN <= 10`.
    - **Borne aussi le numéro rendu** : simuler le rendu avec `seq = 10^NN - 1` (ou 10^10 - 1 si pas de padding) et vérifier que la longueur du résultat tient dans la VARCHAR(64) cible. Sinon → 400 `INVALID_INPUT` (« Le format générerait un numéro trop long »). M2 défensif.
  - Implémentation : un module `invoice_number_format.rs` dans `kesh-api/src/routes/` (helper stateless) avec `pub fn render(template: &str, year: i32, fy_name: &str, seq: i64) -> Result<String, FormatError>` — **testé unitairement** (≥ 8 cas).
  - **Pas de regex de parsing run-time par facture** — optimisation sans objet ; `String::replace` suffit.

- **Génération de l'écriture comptable (5.2 baseline — sans ventilation TVA)** :
  - Journal : `company_invoice_settings.default_sales_journal` (défaut `Ventes`).
  - Date : `invoice.date` (même date que la facture).
  - Description : `"Facture {invoice_number} - {contact_name}"` (ex. « Facture F-2026-0042 - Acme SA »).
  - Lignes : **2 lignes** uniquement en 5.2 (agrégation TTC, sans TVA séparée) :
    - **Débit** sur `default_receivable_account_id`, montant = `invoice.total_amount`.
    - **Crédit** sur `default_revenue_account_id`, montant = `invoice.total_amount`.
  - `fiscal_year_id` : résolu à partir de `invoice.date` ∈ `[fiscal_years.start_date, end_date]` avec `status = 'Open'`. **Si exercice clôturé ou inexistant** : 400 `FISCAL_YEAR_INVALID` avec message explicite (« Impossible de valider une facture dont la date n'appartient à aucun exercice ouvert »).
  - `entry_number` : réutiliser le compteur natif `journal_entries` (Story 3.2 l'incrémente déjà atomiquement par `(company_id, fiscal_year_id)`). **NE PAS** réimplémenter la séquence.
  - Liaison facture ↔ écriture : **nouvelle colonne `invoices.journal_entry_id BIGINT NULL`** avec FK `ON DELETE RESTRICT`. Remplie à la validation. Permet la navigation UI (bouton « Voir l'écriture comptable » sur vue facture validée). Impose aussi : **la suppression de l'écriture (hors scope v0.1 sur facture validée) serait bloquée par FK — comportement souhaité**.
  - **Idempotence défensive** : la transaction de validation vérifie `invoice.status = 'draft'` avec `SELECT ... FOR UPDATE` avant toute action. Si concurrence → le second appel voit `status = 'validated'` et renvoie 409 `ILLEGAL_STATE_TRANSITION`.

- **Transaction de validation — squelette (ordre des locks strictement respecté, voir section Concurrence)** :
  ```
  BEGIN;
    -- (1) Lock facture
    SELECT id, status, date, total_amount, contact_id, version FROM invoices
        WHERE id = ? AND company_id = ? FOR UPDATE;
    -- vérifier status == 'draft' → sinon Err(IllegalStateTransition)
    -- charger company_invoice_settings via get_or_create_default_in_tx(&mut tx, company_id)
    --   (version tx-aware — PAS de transaction imbriquée) ; vérifier receivable/revenue NON NULL → sinon Err(ConfigurationRequired)

    -- (2) Lock fiscal_year ouvert couvrant invoice.date
    SELECT id, start_date, end_date, status FROM fiscal_years
        WHERE company_id = ? AND start_date <= ? AND end_date >= ? AND status = 'Open'
        FOR UPDATE;
    -- 0 row → Err(FiscalYearInvalid)

    -- (3) Lock + incrément séquence facture
    SELECT next_number FROM invoice_number_sequences
        WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE;
    -- 0 rows → INSERT IGNORE (company_id, fiscal_year_id, next_number=1) puis SELECT FOR UPDATE à nouveau
    let seq = next_number;
    UPDATE invoice_number_sequences SET next_number = next_number + 1, version = version + 1
        WHERE company_id = ? AND fiscal_year_id = ?;
    let invoice_number = invoice_number_format::render(settings.format, year_from_fy, fy.name, seq);

    -- (4) Créer l'écriture comptable via journal_entries::create_in_tx (qui prend FOR UPDATE sur journal_entries pour entry_number)
    let je = journal_entries::create_in_tx(&mut tx, NewJournalEntry {
        company_id, fiscal_year_id, date = invoice.date, journal = settings.default_sales_journal,
        description = "Facture {invoice_number} - {contact_name}" (locale = companies.accounting_language),
        lines = [
            { account_id: settings.default_receivable_account_id, debit: total, credit: 0 },
            { account_id: settings.default_revenue_account_id,    debit: 0,     credit: total },
        ],
    }, actor_user_id)?;
    -- create_in_tx valide équilibre débit/crédit, calcule entry_number via SELECT MAX FOR UPDATE, et INSERT lignes

    -- (5) Passage validated + liaison
    UPDATE invoices SET status='validated', invoice_number=?, journal_entry_id=?, version=version+1
        WHERE id=? AND version=?;
    -- rows_affected == 0 → Err(OptimisticLockConflict)

    INSERT audit_log 'invoice.validated' avec wrapper {before, after, journal_entry: {id, entry_number, lines}};
    -- échec audit → tx.rollback() explicite, bubble-up
  COMMIT;
  ```
  **Rollback explicite** si l'audit échoue (pattern contacts/products/invoices 5.1).

- **Concurrence et ordre des locks (H2)** :
  - Deux `POST /validate` simultanés sur la **même** facture → le second voit `status = 'validated'` après le `FOR UPDATE` sur invoices et renvoie 409.
  - Deux validations sur des factures **différentes de la même company** → sérialisées par `FOR UPDATE` sur `invoice_number_sequences(company_id, fiscal_year_id)`.
  - **Ordre d'acquisition des locks obligatoire** (documenter en commentaire dans `validate_invoice`) :
    1. `invoices` (`SELECT ... FOR UPDATE` sur la facture à valider).
    2. `fiscal_years` (`SELECT ... FOR UPDATE` via `find_open_covering_date_for_update` — alignement avec l'ordre utilisé par `journal_entries::create` Story 3.2).
    3. `invoice_number_sequences` (`SELECT ... FOR UPDATE` sur la row `(company_id, fiscal_year_id)`).
    4. `journal_entries` (`SELECT MAX(entry_number) ... FOR UPDATE` pour réserver l'entry_number — comportement existant Story 3.2, conservé dans `create_in_tx`).
    5. INSERT journal_entries + journal_entry_lines + UPDATE invoices + INSERT audit.
  - **Pourquoi `fiscal_years` en position 2 et pas 3** : `journal_entries::create` (Story 3.2) prend déjà `fiscal_years FOR UPDATE` avant `journal_entries`. Si `validate_invoice` prenait `invoice_number_sequences` AVANT `fiscal_years`, un flow manuel Story 3.2 concurrent (qui prend `fiscal_years` puis `journal_entries`) pourrait deadlock avec un flow 5.2 qui tient `invoice_number_sequences` et attend `fiscal_years`. Prendre `fiscal_years` en 2e rang aligne l'ordre sur Story 3.2 et **élimine ce risque**.
  - Tests de concurrence incluent à la fois « 2 validates parallèles » et « 1 validate + 1 création manuelle `journal_entries::create` parallèle » (T6.1/T6.3).

- **Verrouillage optimiste** — `PUT /invoice-settings` utilise `version` classique (pattern contacts/products). L'endpoint `validate` n'exige **pas** de `version` en paramètre : il vérifie `status = 'draft'` dans la transaction (plus fort qu'un check de version car l'utilisateur peut modifier la facture entre deux reloads sans que cela invalide sa volonté de valider la version la plus récente).

- **RBAC** :
  - `POST /invoices/:id/validate` → `comptable_routes` (un comptable peut valider).
  - `GET /company/invoice-settings` → `authenticated_routes` (tous peuvent lire la config pour preview).
  - `PUT /company/invoice-settings` → `admin_routes` (seul Admin peut modifier — paramétrage société).

- **Audit log** :
  - `invoice.validated` → snapshot complet { before: {status='draft', invoice_number=null, ...}, after: {status='validated', invoice_number='F-2026-0042', journal_entry_id=..., ...}, journal_entry: {id, entry_number, lines} }.
  - `company_invoice_settings.updated` → wrapper `{before, after}`.

- **Impact sur les flows Story 5.1** :
  - Le handler `update_invoice` (déjà existant) continue de refuser `status != 'draft'`. Aucune modification.
  - Le handler `delete_invoice` idem : une facture validée n'est jamais supprimée. (L'avoir arrive en Epic 10.)
  - Le handler `create_invoice` : **pas d'impact** — création toujours en `draft`.
  - La page `/invoices/[id]/+page.svelte` doit être adaptée : **bouton « Valider »** visible si draft, **affichage du numéro** visible si validée, **bouton « Voir l'écriture comptable »** (lien vers `/journal-entries/:id`) si validée.

- **i18n du nom du journal « Ventes »** — le nom stocké dans `journal_entries.journal` est un **code ASCII fixe** (`'Ventes'`), pas un libellé traduit (cf. CHECK BINARY). Le libellé UI est déjà traduit côté frontend via i18n (Story 3.2). Aucune friction.

- **Description de l'écriture** — utilise la langue comptable de la company (`companies.accounting_language`), pas celle de l'interface. Pattern : récupérer le template depuis les locales via `kesh-i18n` avec la locale de `accounting_language`. Clé : `invoice-journal-entry-description` avec args `{invoiceNumber}`, `{contactName}`. **Fallback** : si la clé n'existe pas dans la locale demandée, fallback sur `fr-CH` (pattern kesh-i18n).

- **Migration number** : avant création, faire `ls crates/kesh-db/migrations/` et prendre le numéro suivant (attendu `20260417000001_invoice_validation.sql`, **à vérifier empiriquement**).

- **Feature flag / rollout** : aucun. La transition draft→validated est un pur ajout, pas de migration destructrice.

## Acceptance Criteria (AC)

1. **Validation nominale** (FR33) — Given une facture brouillon v1 avec 2 lignes (total 1200.00 CHF), un exercice ouvert couvrant `invoice.date`, et une config `invoice_number_format = 'F-{YEAR}-{SEQ:04}'` + comptes par défaut configurés, When `POST /api/v1/invoices/:id/validate`, Then la facture passe à `status = 'validated'`, reçoit `invoice_number = 'F-2026-0001'`, reçoit un `journal_entry_id` non null, `version` incrémenté, une écriture comptable est créée avec 2 lignes équilibrées (débit 1200.00 sur compte créance + crédit 1200.00 sur compte produit), journal `Ventes`, entry_number séquentiel. Audit `invoice.validated` écrit avec snapshot complet.

2. **Compteur séquentiel sans trou** — Given 3 validations consécutives dans le même exercice, Then les numéros attribués sont `0001`, `0002`, `0003` (aucun trou). Given un rollback de validation (erreur au milieu de la transaction), Then le compteur n'avance pas (transaction rollback → UPDATE sur sequences annulé). **Test** : forcer une erreur d'audit → vérifier que `next_number` reste à la valeur initiale et qu'aucun journal_entry n'est créé.

3. **Compteur par exercice** — Given une validation dans exercice A (2025) et une autre dans exercice B (2026), Then chaque exercice a sa propre séquence (2025 → 0001, 2026 → 0001). Les rows `invoice_number_sequences` sont distinctes par `(company_id, fiscal_year_id)`.

4. **Concurrence — validation simultanée** — Given 2 appels `POST /validate` simultanés sur la **même** facture, Then un seul réussit (200 + status='validated'), l'autre renvoie 409 `ILLEGAL_STATE_TRANSITION`. Aucune double écriture comptable. Aucun saut de compteur.

5. **Concurrence — validations parallèles de factures différentes** — Given 2 appels `POST /validate` simultanés sur **deux factures différentes** de la **même company/exercice**, Then les deux réussissent, les numéros sont distincts et consécutifs (ex. 0042 et 0043), aucun deadlock.

6. **Immutabilité post-validation** — Given une facture `validated`, When `PUT /api/v1/invoices/:id`, Then 409 `ILLEGAL_STATE_TRANSITION`. When `DELETE /api/v1/invoices/:id`, Then 409. When `POST /validate` à nouveau, Then 409.

7. **Rejet exercice clôturé/inexistant** — Given une facture dont la `date` n'appartient à aucun `fiscal_year` ouvert, When `POST /validate`, Then 400 `FISCAL_YEAR_INVALID` avec message « Aucun exercice ouvert pour cette date ». Aucun changement persistant.

8. **Rejet config incomplète** — Given `default_receivable_account_id` ou `default_revenue_account_id` NULL dans `company_invoice_settings`, When `POST /validate`, Then 400 `CONFIGURATION_REQUIRED` avec message « Configurez les comptes par défaut dans Paramètres > Facturation avant de valider une facture ».

9. **Format de numérotation configurable** (FR35) — Given config `invoice_number_format = 'FACT{SEQ}'`, When validation, Then `invoice_number = 'FACT1'`. Given format `'{FY}/{SEQ:06}'` avec fiscal_year.name = '2025/2026', When validation, Then `invoice_number = '2025/2026/000042'`.

10. **Unicité du numéro** — Given un `invoice_number` déjà attribué dans la même company, When tentative d'insertion manuelle du même numéro via SQL direct, Then violation `UNIQUE(company_id, invoice_number)` rejetée par la DB. Défense en profondeur contre bug applicatif.

11. **Écriture comptable équilibrée** — Given une facture validée, Then l'écriture comptable générée est équilibrée : Σ débits = Σ crédits = `invoice.total_amount`. **Validation principale** : la fonction `journal_entries::create_in_tx` appelle `kesh_core::validate_balanced(&lines)` avant INSERT (pattern Story 3.2, canonical). **Assertion post-insert défensive** dans le test d'intégration `test_validate_nominal_creates_journal_entry_and_assigns_number` : SELECT débits/crédits depuis la DB → assert égalité. Pas de re-vérification dans le code de prod (trust `create_in_tx`).

12. **Liaison bidirectionnelle** — Given une facture validée, Then `invoices.journal_entry_id` pointe sur la row créée. Given tentative de suppression de cette `journal_entry` via SQL direct, Then violation FK `ON DELETE RESTRICT` (protège l'intégrité).

13. **CRUD config** — Given rôle Admin, When `PUT /company/invoice-settings`, Then la config est mise à jour avec verrou optimiste. Given rôle Comptable, When `PUT /company/invoice-settings`, Then 403. Given absence de row `company_invoice_settings`, When `GET /company/invoice-settings`, Then 200 avec valeurs par défaut (format `F-{YEAR}-{SEQ:04}`, comptes NULL, journal `Ventes`) et création lazy de la row.

14. **Validation format côté backend** — Given format `F-{INVALID}-{SEQ}` (placeholder inconnu) ou format vide ou format > 64 chars, When `PUT /invoice-settings`, Then 400 `INVALID_INPUT` avec message explicite listant les placeholders autorisés.

15. **UI — bouton « Valider »** — Sur `/invoices/[id]`, bouton « Valider » visible uniquement si `status === 'draft'`. Click → dialog confirmation → POST validate → recharge la vue → affichage du numéro et du lien vers l'écriture comptable. Erreur 409 → toast + reload. Erreur 400 `CONFIGURATION_REQUIRED` → toast contextualisé au rôle : **si Admin** lien cliquable vers `/settings/invoicing`, **sinon** message « Demandez à votre administrateur de configurer les comptes par défaut de facturation » (cf. SD-2).

16. **UI — page config** — `/settings/invoicing` réservée aux Admin (route guard). Champs : format (input + preview temps réel `F-2026-0001`), sélecteur compte créance (dropdown Asset accounts), sélecteur compte produit (dropdown Revenue accounts), sélecteur journal (5 options fixes). Bouton « Enregistrer » → PUT. Conflit 409 → modale reload. Succès → toast.

17. **Preview format temps réel côté frontend** — helper `previewInvoiceNumber(format: string, year: number, fyName: string, seq: number): string` dans `invoices.helpers.ts`, testé Vitest, utilisé dans la page de config pour afficher à l'utilisateur un exemple concret pendant la saisie.

18. **RBAC** — `POST /invoices/:id/validate` → comptable/admin OK, readonly → 403. `PUT /invoice-settings` → admin OK, comptable/readonly → 403. `GET /invoice-settings` → tous authentifiés.

19. **i18n** — ~30 clés supplémentaires × 4 langues (validation, config, erreurs spécifiques).

20. **Audit log complet** — Chaque validation écrit `invoice.validated` atomiquement avec snapshot. Chaque update config écrit `company_invoice_settings.updated` wrapper. Rollback explicite si audit échoue.

21. **Tests** — DB (atomicité + concurrence + compteur), unit handlers (format parsing, validation refus), Vitest (preview helper), Playwright (golden path validation + redirect + affichage numéro).

## Tasks / Subtasks

### T1 — Migrations & entités (AC: #1, #2, #3, #10, #12, #13)

- [x] T1.1 Créer `crates/kesh-db/migrations/20260417000001_invoice_validation.sql` (vérifier numéro avant) :
  - `CREATE TABLE invoice_number_sequences` (voir Décisions de conception).
  - `CREATE TABLE company_invoice_settings` (voir Décisions de conception).
  - `ALTER TABLE invoices ADD COLUMN journal_entry_id BIGINT NULL, ADD CONSTRAINT fk_invoices_journal_entry FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT, ADD CONSTRAINT uq_invoices_number UNIQUE (company_id, invoice_number);` — note : MariaDB autorise plusieurs NULL dans un UNIQUE (standard SQL), pas besoin de colonne générée.
  - **`due_date` existe déjà** (Story 5.1) — pas de nouvelle colonne. Le backend `create_invoice` défautera `due_date = invoice.date` si non fournie (modif T2 ou T3 handler).
  - **Ajout colonne `journal_entry_description_template`** dans `company_invoice_settings` (inclus dans le CREATE TABLE) — voir Décisions de conception v0.6.
- [x] T1.2 Créer entités :
  - `crates/kesh-db/src/entities/invoice_number_sequence.rs` — `InvoiceNumberSequence { id, company_id, fiscal_year_id, next_number, version, created_at, updated_at }` avec FromRow.
  - `crates/kesh-db/src/entities/company_invoice_settings.rs` — `CompanyInvoiceSettings { company_id, invoice_number_format, default_receivable_account_id (Option<i64>), default_revenue_account_id (Option<i64>), default_sales_journal, version, ... }`. `CompanyInvoiceSettingsUpdate { invoice_number_format, default_receivable_account_id, default_revenue_account_id, default_sales_journal, version }`.
- [x] T1.3 Modifier `crates/kesh-db/src/entities/invoice.rs` :
  - Ajouter champ `journal_entry_id: Option<i64>` à `Invoice`.
  - Exposer via `#[serde(rename_all = "camelCase")]` → `journalEntryId`.
- [x] T1.4 Ajouter `pub mod invoice_number_sequence;` et `pub mod company_invoice_settings;` + re-exports dans `entities/mod.rs`.

### T2 — Repositories (AC: #1, #2, #3, #4, #5, #6, #7, #8, #11, #12, #13, #20)

- [x] T2.0a **Refactor `journal_entries`** — extraire `create_in_tx(tx, NewJournalEntry, actor) -> Result<JournalEntry>` depuis `create(pool, ...)`. Faire déléguer `create` à `create_in_tx`. Conserver le `FOR UPDATE` journal_entries + validation équilibre débit/crédit + audit. Tests Story 3.2 existants doivent passer sans modification (refactor non-breaking).
- [x] T2.0b **Créer `fiscal_years::find_open_covering_date(tx, company_id, date) -> Result<Option<FiscalYear>>`** — query `WHERE company_id = ? AND start_date <= ? AND end_date >= ? AND status = 'Open' FOR UPDATE`. Ne PAS toucher à `find_covering_date` existant.
- [x] T2.0c **Étendre `DbError` dans `crates/kesh-db/src/errors.rs`** :
  - Ajouter `FiscalYearInvalid` (sans payload — contexte dans le message log) et `ConfigurationRequired { field: String }` (pour message UX précis, ex. `"default_receivable_account_id"`).
  - Étendre `impl DbError { pub fn code(&self) -> &'static str }` avec `FISCAL_YEAR_INVALID` et `CONFIGURATION_REQUIRED`.
  - Mapper dans `kesh-api/src/errors.rs` (ou l'équivalent `IntoResponse`) → 400 `FISCAL_YEAR_INVALID` et 400 `CONFIGURATION_REQUIRED`.
  - Ajouter tests unit des mappings dans le module existant.
- [x] T2.1 Créer `crates/kesh-db/src/repositories/invoice_number_sequences.rs` :
  - `pub async fn next_number_for(&mut tx, company_id, fiscal_year_id) -> Result<i64>` — SELECT FOR UPDATE → si absent INSERT (1) et retry → capturer valeur → UPDATE +1 → renvoyer valeur capturée.
  - Tests : concurrence 2 validations parallèles (même fy) → numéros consécutifs distincts. Rollback → compteur intact.
- [x] T2.2 Créer `crates/kesh-db/src/repositories/company_invoice_settings.rs` :
  - **Deux signatures pour éviter les transactions imbriquées** :
    - `get_or_create_default(pool, company_id) -> Result<CompanyInvoiceSettings>` — version pool-level, utilisée par le handler `GET /company/invoice-settings` (hors transaction validate).
    - `get_or_create_default_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, company_id) -> Result<CompanyInvoiceSettings>` — version tx-level, utilisée par `validate_invoice` pour charger la config **à l'intérieur** de la transaction atomique (évite tx imbriquée).
    - **Stratégie DRY — 2 options acceptables** :
      1. **Préférée** : helper privé `get_or_create_default_inner<'e, E>(executor: E, company_id) -> Result<...>` avec bound `E: sqlx::Executor<'e, Database = sqlx::MySql>` (HRTB peut être nécessaire). Les deux signatures publiques délèguent.
      2. **Fallback pragmatique** si le générique ne compile pas proprement avec SQLx 0.8 (les bounds `Executor` + `&mut Transaction` sont notoirement fragiles) : **dupliquer les 4-5 lignes** `INSERT IGNORE` + `SELECT` dans les deux fonctions concrètes, avec un commentaire `// MIRROR: keep in sync with get_or_create_default_in_tx` de part et d'autre. Test `test_get_or_create_concurrent` couvre les deux call-sites via une fonction de test partagée. **Ne pas perdre plus de 2h sur le générique** — si difficile, passer au fallback sans regret.
  - **Pattern idempotent obligatoire (H1)** : utiliser `INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)` (les autres colonnes prennent leurs DEFAULT définis dans le CREATE TABLE), suivi d'un `SELECT`. Deux appels concurrents sur une company sans row existante → une seule row créée, le second `INSERT IGNORE` est no-op (pas d'erreur PK), et les deux `SELECT` retournent la même row.
  - **Note MariaDB** : `INSERT IGNORE` + `SELECT` dans la même transaction voit bien la row insérée (`READ_COMMITTED` implicite sur sa propre tx). Pas besoin de `SELECT FOR UPDATE` ici — le contenu est en lecture seule dans la suite de `validate_invoice` (les comptes/format sont des colonnes immuables pour la durée de la transaction côté Admin qui modifie via `PUT`).
  - Test dédié `test_get_or_create_concurrent` (pool-level) avec `tokio::join!` : 2 appels parallèles `get_or_create_default(pool, same_company_id)` → exactement une row créée, deux retours équivalents, aucune erreur PK propagée.
  - `update(pool, company_id, expected_version, update, actor) -> Result<CompanyInvoiceSettings>` — verrou optimiste + audit.
  - Tests : get sur company sans settings → crée la row avec défauts. Update concurrent → OptimisticLockConflict.
- [x] T2.3 Modifier `crates/kesh-db/src/repositories/invoices.rs` — ajouter `pub async fn validate_invoice(pool, id, company_id, actor_user_id) -> Result<(Invoice, JournalEntry)>` :
  - Squelette transaction dans « Décisions de conception ».
  - Ordre de locks documenté en commentaire : `invoices` (FOR UPDATE) → `fiscal_years` (FOR UPDATE via `find_open_covering_date`) → `invoice_number_sequences` (FOR UPDATE) → `journal_entries` (FOR UPDATE interne à `create_in_tx`) → INSERT `journal_entry_lines` → UPDATE `invoices`. **Alignement strict avec la section Concurrence** — toute divergence = deadlock potentiel.
  - **ATTENTION atomicité (C1)** : la fonction existante `journal_entries::create(pool, ...)` ouvre sa propre transaction — **inutilisable** depuis `validate_invoice` (romperait l'atomicité). **Créer une nouvelle fonction publique `journal_entries::create_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, new: NewJournalEntry, actor_user_id: i64) -> Result<JournalEntry, DbError>`** qui n'ouvre pas de transaction mais accepte celle du caller. Refactorer `journal_entries::create` pour déléguer à `create_in_tx` (DRY, pas de duplication). Story 3.2 existante non impactée fonctionnellement.
  - Rendu du numéro via helper `invoice_number_format::render`.
  - Résolution `fiscal_year_id` via `fiscal_years::find_open_covering_date(tx, company_id, date)`. **La fonction actuelle `find_covering_date` NE FILTRE PAS sur `status = 'Open'`** (vérifié dans `repositories/fiscal_years.rs`) — deux options, au choix (privilégier la 1) :
    1. **Créer une nouvelle fonction `find_open_covering_date`** qui ajoute `AND status = 'Open'` à la query. Nom explicite, pas de régression sur les callers de `find_covering_date`.
    2. Ajouter une vérification explicite `fy.status == FiscalYearStatus::Open` dans `validate_invoice` après l'appel — plus fragile car peut être oublié par un futur caller.
    **Retenu : option 1.** Échec → `DbError::FiscalYearInvalid` → handler → 400 `FISCAL_YEAR_INVALID`.
  - Audit `invoice.validated` avec wrapper + écriture comptable embarquée dans le snapshot.
  - Sur échec partiel → `tx.rollback()` explicite avant bubble-up de l'erreur.
- [x] T2.4 Ajouter `pub mod invoice_number_sequences; pub mod company_invoice_settings;` dans `repositories/mod.rs`.

### T3 — Helper format + API routes (AC: #9, #13, #14, #15, #16, #17, #18)

- [x] T3.1 Créer `crates/kesh-api/src/routes/invoice_number_format.rs` :
  ```rust
  pub fn render(template: &str, year: i32, fy_name: &str, seq: i64) -> Result<String, FormatError>;
  pub fn validate_template(template: &str) -> Result<(), FormatError>;
  ```
  Placeholders : `{YEAR}`, `{FY}`, `{SEQ}`, `{SEQ:04}`, `{SEQ:06}`. Longueur max 64. Regex whitelist `^[A-Za-z0-9\-_/\.#\s{}:]+$`. Exige ≥ 1 placeholder reconnu. Tests ≥ 10 cas (valides + invalides + edge : `{SEQ:99}`, placeholder inconnu, caractère non autorisé).
- [x] T3.2 Créer `crates/kesh-api/src/routes/company_invoice_settings.rs` avec DTOs + 2 handlers :
  - `GET /api/v1/company/invoice-settings` (authenticated_routes).
  - `PUT /api/v1/company/invoice-settings` (admin_routes).
  - Validation : `validate_template`, comptes existants + du bon type (Asset/Revenue) + active, journal dans whitelist, version attendue.
- [x] T3.3 Ajouter handler dans `crates/kesh-api/src/routes/invoices.rs` :
  - `POST /api/v1/invoices/:id/validate` (comptable_routes) → appelle `invoices::validate_invoice` → renvoie `InvoiceResponse` (incluant `journalEntryId` et `invoiceNumber`).
  - Mapping erreurs : `IllegalStateTransition` → 409, `FiscalYearInvalid` → 400 `FISCAL_YEAR_INVALID`, `ConfigurationRequired` → 400 `CONFIGURATION_REQUIRED`, `OptimisticLockConflict` → 409 (défensif).
- [x] T3.4 Enregistrer routes dans `kesh-api/src/lib.rs` : validate dans `comptable_routes` ; GET settings dans `authenticated_routes` ; PUT settings dans `admin_routes` (si la garde admin n'existe pas, créer le layer — vérifier `auth::require_admin_role` avant de dupliquer).
- [x] T3.5 Ajouter `pub mod invoice_number_format; pub mod company_invoice_settings;` dans `routes/mod.rs`.

### T4 — Frontend : bouton validation + page config (AC: #15, #16, #17, #19)

- [x] T4.1 Étendre `frontend/src/lib/features/invoices/invoices.api.ts` :
  - `validateInvoice(id: number): Promise<InvoiceResponse>`.
  - `getInvoiceSettings(): Promise<InvoiceSettings>`.
  - `updateInvoiceSettings(update, version): Promise<InvoiceSettings>`.
- [x] T4.2 Étendre `invoices.types.ts` : types `InvoiceSettings`, `InvoiceSettingsUpdate`.
- [x] T4.3 Créer `frontend/src/lib/features/invoices/invoice-number-format.ts` :
  - `previewInvoiceNumber(format, year, fyName, seq): string` — mirror du helper Rust. Test Vitest.
  - `validateFormatTemplate(format): { ok: boolean; error?: string }` — validation côté client pour feedback UX.
- [x] T4.4 Modifier `frontend/src/routes/(app)/invoices/[id]/+page.svelte` :
  - Bouton « Valider » si `invoice.status === 'draft'` → dialog confirmation → `validateInvoice(id)` → reload.
  - Affichage du `invoice.invoiceNumber` en grand si `validated`.
  - Bouton « Voir l'écriture comptable » (lien `/journal-entries/{journalEntryId}`) si validée.
  - Désactivation bouton « Modifier » / « Supprimer » si `validated` (déjà géré côté API, mais UI propre).
- [x] T4.5 Créer `frontend/src/routes/(app)/settings/invoicing/+page.svelte` :
  - Route guard Admin (layout parent ou check dans load/onMount).
  - Formulaire : format (input + preview live via `previewInvoiceNumber`), selects compte client (filtré Asset actifs), compte produit (filtré Revenue actifs), journal (5 options), bouton Save.
  - Gestion 409 (modale reload). Toast succès.
- [x] T4.6 Ajouter entrée sidebar « Paramètres > Facturation » (visible Admin uniquement) dans `+layout.svelte`.

### T5 — i18n (AC: #19)

- [x] T5.1 Ajouter ~30 clés × 4 langues (fr-CH, de-CH, it-CH, en-CH) :
  - Bouton/dialog : `invoice-validate-button`, `invoice-validate-confirm-title`, `invoice-validate-confirm-body`, `invoice-validate-success`, `invoice-validate-success-body` (avec `{invoiceNumber}`).
  - Erreurs : `invoice-error-fiscal-year-invalid`, `invoice-error-configuration-required`, `invoice-error-already-validated`.
  - Vue facture : `invoice-number-label`, `invoice-status-validated-label`, `invoice-view-journal-entry-link`.
  - Page config : `settings-invoicing-title`, `settings-invoicing-format-label`, `settings-invoicing-format-help`, `settings-invoicing-format-preview`, `settings-invoicing-receivable-account`, `settings-invoicing-revenue-account`, `settings-invoicing-journal`, `settings-invoicing-save`, `settings-invoicing-format-invalid`, `settings-invoicing-format-too-long`.
  - Description écriture auto : `invoice-journal-entry-description` (args : `{invoiceNumber}`, `{contactName}`). **Utiliser la locale `companies.accounting_language`**, pas la locale d'interface.
  - Nav : `nav-settings-invoicing`.

### T6 — Tests (AC: #21)

- [ ] T6.1 Tests DB `invoice_number_sequences::tests` :
  - `test_first_call_creates_row_with_next_1`.
  - `test_consecutive_calls_return_sequential`.
  - `test_rollback_does_not_advance_counter`.
  - `test_separate_fiscal_years_have_separate_counters`.
  - `test_concurrent_calls_serialize_via_for_update` (2 tx parallèles → numéros distincts, pas de deadlock). **Config pool obligatoire** : `PoolOptions::new().max_connections(4)` (au moins 2 — la 1re tx garde sa connexion jusqu'au COMMIT, le 2e `pool.begin()` doit pouvoir prendre une autre connexion ou `tokio::join!` deadlock). Si le harnais `#[sqlx::test]` du projet ne permet pas de configurer `max_connections`, créer un pool manuel dans le test (helper local `setup_pool_for_concurrency_test()`). **Compatible avec CI `--test-threads=1`** : `tokio::join!` est de la concurrence intra-test (multi-tasks Tokio), pas inter-test ; pas de conflit avec la sérialisation cross-test imposée par `feedback_sqlx_mysql_gotchas`. Pattern : 2 tâches `tokio::spawn` qui font chacune `pool.begin()` → `next_number_for` → sleep 100ms avant COMMIT → COMMIT. Asserter numéros = {N, N+1} sans trou.
  - Ajouter aussi `test_concurrent_validate_vs_manual_journal_entry` dans `invoices::tests` (T6.3) : 1 tâche `validate_invoice`, 1 tâche `journal_entries::create` manuel en parallèle — vérifier absence de deadlock (timeout 5s → fail).
- [ ] T6.2 Tests DB `company_invoice_settings::tests` :
  - `test_get_or_create_lazy_insert`.
  - `test_get_or_create_concurrent` — 2 GETs simultanés via `tokio::join!` sur une company sans row → une seule row créée, les deux `SELECT` retournent la même row, aucun 500 (H1).
  - `test_update_optimistic_lock`.
  - `test_check_journal_whitelist`.
- [ ] T6.3 Tests DB `invoices::tests` (étendus) :
  - `test_validate_nominal_creates_journal_entry_and_assigns_number`.
  - `test_validate_rejects_non_draft`.
  - `test_validate_rejects_fiscal_year_closed`.
  - `test_validate_rejects_missing_config`.
  - `test_validate_is_atomic_rollback_on_audit_failure` (mock audit échoue → compteur intact + pas de je).
  - `test_validate_unique_number_constraint` (force INSERT direct avec numéro existant → erreur FK).
  - `test_validated_invoice_rejects_update_and_delete` (assertion défensive sur comportement existant).
  - `test_validate_concurrent_same_invoice_one_succeeds_other_409`.
- [ ] T6.4 Tests unit handlers `invoices::tests` : POST validate → mapping erreurs. `company_invoice_settings::tests` : validation format, RBAC refus non-admin sur PUT.
- [x] T6.5 Tests unit `invoice_number_format.rs` : ≥ 10 cas (tous placeholders, padding, texte libre, erreurs format). **Cas nommés obligatoires** : `test_max_padding_within_varchar64` — `validate_template("F-{YEAR}-{SEQ:10}")` → OK, puis simuler rendu avec `seq = 9_999_999_999` et asserter longueur ≤ 64. `test_padding_nn_zero_rejected` (`{SEQ:0}` → erreur). `test_padding_nn_above_10_rejected` (`{SEQ:11}` → erreur).
- [ ] T6.6 Tests Vitest `invoice-number-format.test.ts` : mirror des tests Rust côté frontend pour le helper `previewInvoiceNumber`.
- [ ] T6.7 Test Playwright `invoices_validate.spec.ts` :
  - Login admin → configurer format + comptes par défaut.
  - Logout / login comptable → créer une facture draft (réutiliser flow Story 5.1).
  - Cliquer « Valider » → confirmer dialog → vérifier affichage du numéro (ex. `F-2026-0001`) + bouton « Voir l'écriture comptable ».
  - Tenter de modifier (bouton disparu ou 409) → toast attendu.

### T7 — Validation finale

- [x] T7.1 `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] T7.2 `cargo test --workspace -- --test-threads=1` (respecter contrainte cross-binary SQLx, mémoire `feedback_sqlx_mysql_gotchas`).
- [x] T7.3 `npm run test:unit -- --run` full suite frontend.
- [x] T7.4 `npm run check` (svelte-check 0 errors).
- [ ] T7.5 Test manuel end-to-end : créer facture, valider, vérifier écriture comptable dans `/journal-entries`.
- [x] T7.6 Mettre à jour sprint-status → `review`.

## Dev Notes

### Architecture — où va quoi

```
kesh-db/
├── migrations/20260417000001_invoice_validation.sql        # T1.1
└── src/
    ├── entities/
    │   ├── invoice.rs                                      # T1.3 (ajout journal_entry_id)
    │   ├── invoice_number_sequence.rs                      # T1.2
    │   ├── company_invoice_settings.rs                     # T1.2
    │   └── mod.rs                                          # T1.4
    └── repositories/
        ├── invoices.rs                                     # T2.3 (ajout validate_invoice)
        ├── invoice_number_sequences.rs                     # T2.1
        ├── company_invoice_settings.rs                     # T2.2
        └── mod.rs                                          # T2.4

kesh-api/src/routes/
├── invoices.rs                                             # T3.3 (handler POST validate)
├── invoice_number_format.rs                                # T3.1 (helper stateless + tests)
├── company_invoice_settings.rs                             # T3.2 (GET + PUT)
└── mod.rs                                                  # T3.5

frontend/src/lib/features/invoices/
├── invoices.api.ts                                         # T4.1
├── invoices.types.ts                                       # T4.2
├── invoice-number-format.ts                                # T4.3
└── invoice-number-format.test.ts                           # T6.6

frontend/src/routes/(app)/
├── invoices/[id]/+page.svelte                              # T4.4 (ajout bouton Valider)
└── settings/invoicing/+page.svelte                         # T4.5

frontend/tests/e2e/invoices_validate.spec.ts                # T6.7

crates/kesh-i18n/locales/*/messages.ftl                     # T5.1 (+30 clés × 4 langues)
```

### Ce qui existe DÉJÀ — NE PAS refaire

- **Pattern repository + audit atomique + rollback** — contacts.rs / products.rs / invoices.rs (5.1). Strict copie.
- **`DbError::IllegalStateTransition`** — déjà mappé 409 par les handlers invoices.
- **`DbError::OptimisticLockConflict`** — idem.
- **Repository `journal_entries`** — Story 3.2. La fonction publique existante est `journal_entries::create(pool, ...)` qui ouvre sa PROPRE transaction — **inutilisable depuis `validate_invoice`** (C1). **Refactor obligatoire** : extraire le cœur en `create_in_tx(tx: &mut Transaction<'_, MySql>, ...)` et faire déléguer `create` à ce nouveau helper (DRY). `create_in_tx` doit conserver : le `FOR UPDATE` sur `journal_entries` pour `entry_number` (MAX+1), la validation équilibre débit/crédit via `kesh-core`, l'INSERT des lignes, et l'audit atomique. La validation d'équilibre DOIT rester dans `kesh-core` (ou dans `create_in_tx`) — **NE JAMAIS dupliquer** dans `validate_invoice`.
- **Helper `fiscal_years::find_open_covering_date`** — **n'existe PAS** dans `repositories/fiscal_years.rs` (vérifié). La fonction existante `find_covering_date` ne filtre **pas** sur `status = 'Open'` — danger (C2). **Créer `find_open_covering_date`** (ajoute `AND status = 'Open'` + `FOR UPDATE` si appelée depuis une transaction). **Ne pas modifier `find_covering_date`** pour ne pas régresser Story 3.2 qui l'utilise peut-être avec d'autres intentions.
- **Whitelist journals** — déjà dans le CHECK `chk_journal_entries_journal`. Le CHECK de `company_invoice_settings.default_sales_journal` la duplique volontairement (défense en profondeur). **Valeur canonique** : constante `JOURNAL_CODES: [&str; 5] = ["Achats", "Ventes", "Banque", "Caisse", "OD"]` à extraire dans un module partagé (ex. `kesh-db::constants` ou `routes/journals.rs`) si pas déjà fait. Sinon dupliquer avec un commentaire « aligné avec chk_journal_entries_journal ».
- **`rust_decimal::Decimal`** — pour `total_amount` (déjà utilisé par journal_entry_lines pour débit/crédit).
- **`ListResponse<T>`**, `notifySuccess/Error`, `i18nMsg`, `onMount` URL init, cleanup debounce — tous les patterns 5.1.

### Points de vigilance (prévention LLM)

1. **NE PAS remplacer le compteur par AUTO_INCREMENT** — exigence FR33 = séquence sans trou, incompatible avec AUTO_INCREMENT en InnoDB (trous sur rollback).
2. **NE PAS oublier le `FOR UPDATE`** sur `invoice_number_sequences` — sans lui, deux validations parallèles → numéros dupliqués.
3. **NE PAS calculer `entry_number` du `journal_entry` manuellement** — réutiliser la séquence existante de Story 3.2 (compteur natif journal_entries).
4. **Ordre des locks strict** : `invoices` → `fiscal_years` → `invoice_number_sequences` → `journal_entries`. **Section Concurrence canonique** (Décisions de conception). Inverser → risque de deadlock avec flows concurrents `journal_entries::create` manuel.
5. **NE PAS ventiler la TVA dans l'écriture en 5.2** — décision explicite. Ventilation = Epic 9. En 5.2, 2 lignes suffisent (débit créance / crédit revenus agrégés TTC).
6. **NE PAS utiliser une regex complexe pour le format** — `String::replace` sur les placeholders reconnus suffit. Tests > regex.
7. **NE PAS copier l'enum de journaux localement** dans le handler validate — réutiliser la constante partagée (ou la dupliquer avec commentaire d'alignement explicite).
8. **NE PAS oublier la locale `accounting_language`** pour la description de l'écriture — différente de la locale UI.
9. **NE PAS stocker le format dans `companies`** — usage d'une table dédiée `company_invoice_settings` pour découplage.
10. **`validate` est atomique** — tout ou rien. Aucun état intermédiaire (numéro attribué mais pas d'écriture, ou écriture créée mais facture toujours draft) ne doit exister. Rollback explicite + tests explicites sur ce comportement (T6.3 `test_validate_is_atomic_rollback_on_audit_failure`).
11. **`UNIQUE(company_id, invoice_number)` accepte plusieurs NULL** — vérifié sur MariaDB. Pas de colonne générée nécessaire. Si le reviewer doute : test DB qui insère 10 factures draft (invoice_number=NULL) et vérifie qu'aucune erreur UNIQUE n'est levée.
12. **RBAC Admin pour PUT settings** — vérifier que `admin_routes` existe dans `kesh-api/src/lib.rs`. Si seulement `comptable_routes` et `authenticated_routes` sont posés, créer le layer (middleware `require_admin_role` probablement déjà présent dans `auth/`).
13. **Migration number** — `ls crates/kesh-db/migrations/` avant création. Max observé au moment de la rédaction : `20260416000002` → **placeholder `20260417000001`** (à ajuster si une hotfix intercurrente a produit une migration entre-temps).
14. **Format validator cohérent backend↔frontend** — les deux doivent rejeter exactement les mêmes inputs. Sinon une config acceptée par l'UI échouera serveur (UX frustrante). Pattern : tests de parité (mêmes cas en Vitest et en unit test Rust).
15. **Snapshot audit `invoice.validated`** — inclure l'écriture comptable complète (id, entry_number, lignes) — utile pour reconstitution forensique.

### Previous Story Intelligence (Story 5.1)

Learnings à appliquer directement :

- **5 passes de code review** sur 5.1 → pattern mature. Bénéficier des patches appliqués :
  - P1 validation id, P11 anti double-submit, P13 modale conflit 409, P41 anti double-click → **appliquer directement sur le bouton « Valider »** (disable pendant submit, toast sur 409, reload).
  - P14 `LazyLock` pour constantes parsées → utiliser pour `JOURNAL_CODES` si extrait.
  - P19 test défense en profondeur sur CHECK DB → test unicité via INSERT direct (AC #10).
  - P37 `await getContact` avant `mounted = true` → pattern d'initialisation async sur `/settings/invoicing` : `await getInvoiceSettings()` + `await listAccounts()` avant le premier paint.
  - P44 `crypto.randomUUID` pour IDs DOM uniques par instance → pertinent si composant `FormatPreview` embarqué.
- **Whitelist TVA partagée** (5.1 T3.1) → déjà DRY. Ne pas re-dupliquer.
- **Pattern `onMount` URL init** (pas `$effect`) — appliquer sur page de config si filtres/params.
- **Flakiness cross-binary SQLx** (`feedback_sqlx_mysql_gotchas`) → les nouveaux tests DB doivent tourner avec `--test-threads=1`. Si la CI tourne `-j1 -- --test-threads=1`, aucune adaptation.
- **`#[serde(rename_all = "camelCase")]`** obligatoire sur chaque nouveau DTO. `rust_decimal::serde-str` gère la sérialisation string auto.
- **Technical debt 5.1 DT-1 (RBAC e2e)** : même traitement possible ici si budget serré — documenter dans section dette plutôt que bloquer la story. Privilégier les tests backend (unit + DB) pour couvrir le RBAC.
- **Technical debt 5.1 DT-4 (i18n UI en dur FR)** : pattern sprint-wide non résolu. Story 5.2 ajoute `/settings/invoicing` et modifie `/invoices/[id]`. **Décision explicite** : utiliser `i18nMsg('settings-invoicing-...')` dans les nouveaux composants dès cette story (ne pas propager la dette). Les clés FTL sont ajoutées en T5.1 — les brancher côté UI simultanément. Coût marginal faible ; évite d'agrandir DT-4.

### Git Intelligence

5 derniers commits :
- `bf8f5d1 feat(story-5.1): draft invoice CRUD + UI + i18n (5 review passes, 52 patches)` ← base de cette story.
- `47bac36 docs(story-5.1): apply pass 2 (Haiku) review patches`.
- `e0e5964 docs: add Story 5.1 spec (draft invoices) after Sonnet review pass`.
- `e3b9f2f docs: rewrite README to GitHub standards`.
- `c894649 ci: run tests with --test-threads=1 to serialize global state` ← **contrainte CI active**, respecter pour les nouveaux tests DB.

### References

- Epic 5 : [`_bmad-output/planning-artifacts/epics.md`](../planning-artifacts/epics.md#Story-5-2) (lignes 891–903).
- PRD FR33–FR35 : [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md) (lignes 421–423).
- Story 5.1 (base factures draft) : [`5-1-creation-factures-brouillon.md`](./5-1-creation-factures-brouillon.md).
- Migration invoices (Story 5.1) : `crates/kesh-db/migrations/20260416000001_invoices.sql`.
- Migration journal_entries (Story 3.2) : `crates/kesh-db/migrations/20260412000001_journal_entries.sql`.
- Migration fiscal_years (Story 1.4) : `crates/kesh-db/migrations/20260404000001_initial_schema.sql`.
- Migration accounts (Story 3.1) : `crates/kesh-db/migrations/20260411000001_accounts.sql`.
- Pattern repository canonique : `crates/kesh-db/src/repositories/invoices.rs` (Story 5.1), `journal_entries.rs` (Story 3.2).
- Règle de remédiation des revues : `CLAUDE.md` § « Règle de remédiation des revues ».

## Security debt (hérité / propagé)

### SD-1 — IDOR multi-tenant (propagation DT-2 Story 5.1)

- **Origine** : pattern systémique hérité (contacts.rs, products.rs, invoices.rs, journal_entries.rs). Les nouveaux handlers de 5.2 (`validate_invoice`, `GET/PUT /company/invoice-settings`) utiliseront vraisemblablement le même pattern `companies::list(&state.pool, 1, 0).first()` pour obtenir `company_id`, au lieu de `current_user.company_id`.
- **Propriétaire** : Guy Corbaz.
- **Story de remédiation** : « multi-tenancy hardening » — cf. DT-2 de Story 5.1 (à planifier avant mise en prod).
- **Justification du report** : déploiement v0.1 mono-tenant par conception (une instance Kesh = une entreprise). Risque concret nul tant qu'une seule row `companies` existe. Correction transverse préférable (tous les handlers en un seul commit) plutôt que story-by-story.
- **Application en 5.2** : utiliser le pattern existant `companies::list(1, 0)` sans ajouter de nouvelle dette, sans tenter de correction locale (cohérence avec le reste du code + règle d'exception CLAUDE.md).

### SD-2 — UX : toast `CONFIGURATION_REQUIRED` avec lien admin pour un Comptable

- **Origine** : AC #15 → toast pointant vers `/settings/invoicing` (page Admin-only).
- **Scope de correction immédiate dans 5.2** : le toast doit vérifier `currentUser.role`. Si `Admin` → lien cliquable vers `/settings/invoicing`. Sinon → message textuel « Demandez à votre administrateur de configurer les comptes par défaut de facturation ». **À implémenter dans T4.4** (pas reporté — coût marginal nul).

## Questions ouvertes pour Guy (à trancher avant ou en début de dev)

1. **Comptes par défaut (FR35 implicite)** — est-ce acceptable d'exiger une config manuelle des comptes créance/revenus (400 `CONFIGURATION_REQUIRED` à la première validation), ou faut-il poser ces valeurs lors de l'onboarding (Story 2.2/2.3) avec les comptes standard Suisse (ex. 1100 Clients, 3000 Ventes) pré-sélectionnés ?
2. **Ventilation TVA reportée en Epic 9** — confirmez-vous qu'en 5.2 l'écriture comptable **ne sépare pas** la TVA (1 ligne crédit agrégée TTC sur le compte de produit) ? L'alternative — séparer dès 5.2 — implique d'introduire `default_vat_account_id` dans `company_invoice_settings` et de ventiler par taux, ce qui est proche du scope Epic 9.
3. **Date de l'écriture** — `journal_entry.date = invoice.date` en 5.2, d'accord ? Alternative : `entry_date = aujourd'hui` à la validation (date comptable ≠ date de facture). Le choix a un impact sur le contrôle de fiscal_year.
4. **Nom du journal** — code fixe `"Ventes"` par défaut (FR), ou exposer un sélecteur à l'utilisateur dès la config company ? (Choix retenu dans le draft : config exposée, défaut `"Ventes"`.)
5. **Libellé de l'écriture** — « Facture {numéro} - {contact} » acceptable ? Besoin de personnalisation via template i18n ?

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context) — `/bmad-dev-story`

### Debug Log References

- `cargo check -p kesh-api` : OK après wiring (GET/PUT settings, POST validate).
- `cargo clippy --workspace --all-targets -- -D warnings` : OK (0 warning).
- `cargo fmt --all` : appliqué.
- `cargo test -p kesh-core --lib invoice_format` : **17/17 passent** (cas nommés T6.5 incl.).
- `cargo test -p kesh-api --lib routes::company_invoice_settings` : **2/2 passent**.
- `npx vitest run invoice-number-format.test.ts` : **13/13 passent**.
- `npm run check` : 0 erreur (fichiers Story 5.2), 2 warnings pré-existants hors scope.

### Completion Notes List

**T1 / T2** : livrés dans commit `03169dc` (backend DB + core) — validate_invoice atomique, create_in_tx, find_open_covering_date, next_number_for, get_or_create_default_in_tx, DbError::FiscalYearInvalid + ConfigurationRequired mappés 400.

**T3 — API routes** : livré.
- Helper format `kesh-core::invoice_format` (cross-crate) — pragmatique vs `routes/invoice_number_format.rs` de la spec, aligné avec l'usage `validate_invoice`.
- `routes/company_invoice_settings.rs` : GET + PUT handlers, DTOs, validation format + description + journal whitelist + comptes (scope, type, actif).
- `routes/invoices.rs` : handler `validate_invoice_handler` + champ `journalEntryId` sur `InvoiceResponse`.
- `lib.rs` : POST /invoices/:id/validate → comptable_routes, GET /company/invoice-settings → authenticated_routes, PUT /company/invoice-settings → admin_routes.

**T4 — Frontend** : livré.
- `invoices.api.ts` : 3 nouvelles fonctions (validate, get/update settings).
- `invoices.types.ts` : `InvoiceSettingsResponse`, `UpdateInvoiceSettingsRequest`, `JournalCode`, `journalEntryId` sur InvoiceResponse.
- `invoice-number-format.ts` + test Vitest (13 cas parité Rust).
- `/invoices/[id]/+page.svelte` : bouton Valider + dialog, bouton « Voir écriture comptable » si validée, gestion 409 / CONFIGURATION_REQUIRED contextualisée rôle (SD-2).
- `/settings/invoicing/+page.svelte` : formulaire Admin (format + preview live, comptes filtrés Asset/Revenue actifs, journal, description template). Optimistic lock 409 → reload.
- `+layout.svelte` : entrée admin « Facturation » → `/settings/invoicing`.

**T5 — i18n** : 27 clés ajoutées à `fr-CH/messages.ftl`. Marqueur section dans de/it/en — traduction complète **reportée en DT-5.2-4** (batchée avec DT-4 propagation pattern-wide). Fallback `fr-CH` de `kesh-i18n` couvre l'usage courant. L'UI Story 5.2 utilise des chaînes françaises hardcodées (cohérent avec l'existant 5.1 — DT-4 inchangée).

**T6 — Tests** : livré partiellement.
- ✅ Kesh-core `invoice_format` : 17 tests (T6.5 complet incl. cas nommés).
- ✅ Route `company_invoice_settings` : 2 tests unit (whitelist journal).
- ✅ Vitest helper format : 13 tests parité.
- ⏳ T6.1/T6.2/T6.3 tests DB concurrence/atomicité : **non livrés** — dette TD-5.2-1/TD-5.2-2. `validate_invoice` est couvert par l'ordre de locks documenté et `FOR UPDATE` canonique ; un angle mort subsiste sur l'empirique `tokio::join!`.
- ⏳ T6.7 Playwright : non livré (TD-5.2-3, même politique DT-1 de 5.1).

**T7** : fmt clean, clippy 0 warning, tests unit OK, svelte-check 0 erreur. Test manuel E2E non exécuté (pas de DB vivante dans cette session).

### Test debt (ouverte)

- **TD-5.2-1** : T6.1 tests DB `invoice_number_sequences` (5 cas — création lazy, consécutifs, rollback, fy séparés, concurrent `tokio::join!`). À écrire en sessions dédiées DB.
- **TD-5.2-2** : T6.3 tests DB `validate_invoice` (nominal, non-draft, fy clôturé, config manquante, atomicité rollback audit, unicité numéro, concurrence 409).
- **TD-5.2-3** : T6.7 Playwright golden path `invoices_validate.spec.ts`.
- **TD-5.2-4** : traduction 27 clés de-CH / it-CH / en-CH — batch DT-4.

### File List

**Nouveaux fichiers (T3–T4–T5, session courante)** :

- `crates/kesh-api/src/routes/company_invoice_settings.rs`
- `frontend/src/lib/features/invoices/invoice-number-format.ts`
- `frontend/src/lib/features/invoices/invoice-number-format.test.ts`
- `frontend/src/routes/(app)/settings/invoicing/+page.svelte`

**Fichiers modifiés (session courante)** :

- `crates/kesh-api/src/lib.rs` — wiring 3 routes.
- `crates/kesh-api/src/routes/mod.rs` — `pub mod company_invoice_settings;`.
- `crates/kesh-api/src/routes/invoices.rs` — handler validate + champ `journalEntryId`.
- `frontend/src/lib/features/invoices/invoices.api.ts` — 3 fonctions.
- `frontend/src/lib/features/invoices/invoices.types.ts` — types settings + `journalEntryId`.
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` — bouton Valider + dialog + lien écriture.
- `frontend/src/routes/(app)/+layout.svelte` — entrée sidebar admin.
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — 27 clés Story 5.2.
- `crates/kesh-i18n/locales/{de-CH,it-CH,en-CH}/messages.ftl` — marqueur section.
- `_bmad-output/implementation-artifacts/5-2-validation-numerotation-factures.md` — Status → review, tasks T3-T7 cochés, Dev Agent Record.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — 5-2 → review.

**Rappel — fichiers livrés commit `03169dc` (T1+T2)** :

- `crates/kesh-db/migrations/20260417000001_invoice_validation.sql`
- `crates/kesh-db/src/entities/{invoice_number_sequence,company_invoice_settings,invoice,mod}.rs`
- `crates/kesh-db/src/repositories/{invoice_number_sequences,company_invoice_settings,invoices,journal_entries,fiscal_years}.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-core/src/{invoice_format.rs,lib.rs}`

## Change Log

| Date       | Version | Description                                          | Auteur          |
| ---------- | ------- | ---------------------------------------------------- | --------------- |
| 2026-04-14 | 0.1     | Spec initiale Story 5.2 (validation + numérotation) | Claude Opus 4.6 |
| 2026-04-14 | 0.2     | Review spec pass 1 (Sonnet, fresh context, 2 CRITICAL + 3 HIGH + 3 MEDIUM + 3 LOW findings) + application de 11 patches : P1 atomicité `journal_entries::create_in_tx` (C1), P2 `fiscal_years::find_open_covering_date` filtre status=Open (C2), P3 `INSERT IGNORE` idempotent lazy insert (H1), P4 ordre des locks explicite incl. fiscal_years en position 2 (H2), P5 squelette transaction détaillé avec commentaires entry_number (H3), P6 test `test_get_or_create_concurrent` (H1), P7 test `test_concurrent_validate_vs_manual_journal_entry` (H2), P8 pool min 2 connexions obligatoire pour tests concurrence (M1), P9 borne padding `{SEQ:NN}` ≤ 10 + validation longueur rendue (M2), P10 commentaire migration alignement CHECK BINARY syntaxe Story 3.2 (M3/L1), P11 DT-4 i18n : brancher `i18nMsg` dès cette story pour ne pas propager la dette (L3). Tasks T2.0a (refactor journal_entries) et T2.0b (fiscal_years helper) ajoutées. | Claude Sonnet 4.6 |
| 2026-04-14 | 0.3     | Review spec pass 2 (Haiku, fresh context, 2 CRITICAL + 1 HIGH + 3 MEDIUM + 2 LOW findings — régressions des patches pass 1) + application de 4 patches : P12 T2.3 ordre de locks aligné avec section Concurrence (incl. fiscal_years en position 2), P13 `get_or_create_default_in_tx` ajouté pour éviter transaction imbriquée dans `validate_invoice` (+ helper privé générique sur Executor), P14 AC #11 clarifie que la validation équilibre vit dans `create_in_tx` (kesh_core), post-insert check = assertion de test uniquement, P15 test concurrence compatibilité `--test-threads=1` documentée (concurrence intra-test Tokio, pas cross-test). | Claude Haiku 4.5  |
| 2026-04-14 | 0.4     | Review spec pass 3 (Opus, fresh context, 1 HIGH + 2 MEDIUM + 4 LOW findings, aucun CRITICAL) + application de 4 patches : P16 task T2.0c ajoutée — étendre enum `DbError` avec `FiscalYearInvalid` et `ConfigurationRequired` (cross-crate kesh-db + kesh-api, mapping 400), P17 fallback pragmatique documenté pour helper générique Executor (si HRTB SQLx 0.8 récalcitrant → duplication contrôlée de 5 lignes avec commentaire MIRROR), P18 section « Security debt » SD-1 (IDOR multi-tenant propagé, documenté comme dette alignée DT-2 5.1) + SD-2 (UX toast CONFIGURATION_REQUIRED conditionnel au rôle, corrigé in-story dans T4.4 + AC #15). Findings LOW L3-1/L3-2/L3-3 non actionnables (OK en l'état). | Claude Opus 4.6   |
| 2026-04-14 | 0.5     | Review spec pass 4 (Sonnet, fresh context, 0 CRITICAL/HIGH/MEDIUM + 3 LOW cosmétiques — **convergence atteinte**) + application de 2 patches polish : P19 point de vigilance #4 aligné sur l'ordre canonique `invoices → fiscal_years → invoice_number_sequences → journal_entries`, P20 T6.5 ajoute tests nommés `test_max_padding_within_varchar64`, `test_padding_nn_zero_rejected`, `test_padding_nn_above_10_rejected`. Verdict pass 4 : « PASS — ready for dev. Convergence atteinte. » Règle de remédiation CLAUDE.md satisfaite (findings ≤ LOW). Spec clôturée, ready pour `bmad-dev-story`. | Claude Sonnet 4.6 |
| 2026-04-14 | 0.6     | Implémentation T3–T7 (`bmad-dev-story`, Opus 4.6). T1/T2 déjà livrés (commit 03169dc). T3 : routes API (company_invoice_settings GET/PUT, validate handler, wiring lib.rs). T4 : frontend complet (API, types, helper format avec test Vitest 13 cas, /invoices/[id] bouton Valider + dialog, /settings/invoicing page Admin, sidebar). T5 : 27 clés FR ajoutées (autres locales → DT-5.2-4 batchée avec DT-4). T6 : tests unit + helper (30 tests), tests DB concurrence reportés en dette (TD-5.2-1/2/3). T7 : fmt/clippy/check green. Status → review. | Claude Opus 4.6   |
| 2026-04-14 | 0.6     | **Élargissement de scope validé par Guy avant démarrage dev** (réponses aux questions ouvertes) : (1) **Clarification `due_date`** — la colonne existe déjà depuis Story 5.1 ; 5.2 ajoute un défaut backend (`due_date = invoice.date` si non fourni) + soin UI (champ date de valeur/échéance). Pas de nouvelle colonne. (2) Ajout colonne `company_invoice_settings.journal_entry_description_template VARCHAR(128) NOT NULL DEFAULT '{YEAR}-{INVOICE_NUMBER}'` — libellé de l'écriture comptable configurable, placeholders `{YEAR}`, `{INVOICE_NUMBER}`, `{CONTACT_NAME}`. Remplace le libellé hard-codé. (3) Stories backlog ajoutées : « Journaux personnalisables » (indispensable avant v1.0), « Onboarding comptes facturation pré-remplis », « Échéancier factures ». **Décisions associées** : Q1 garde 400 `CONFIGURATION_REQUIRED`, Q2 ventilation TVA restée Epic 9, Q4 journal `Ventes` par défaut + 5 codes figés en 5.2 (extensibilité → story dédiée). | Claude Opus 4.6   |
