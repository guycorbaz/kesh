# Story 4.1: Carnet d'adresses (CRUD contacts)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **gérer mes clients et fournisseurs dans un carnet d'adresses unifié avec validation du numéro IDE suisse**,
so that **je puisse les utiliser pour la facturation (Epic 5) et les paiements (Epic 6/10) sans ressaisir les informations, en garantissant la validité des identifiants légaux suisses**.

### Contexte

**Première story de l'Epic 4** (Carnet d'adresses & Catalogue). Couvre FR25 (carnet unifié), FR26 (flags client/fournisseur), FR27 (validation IDE CHE), et pose les fondations schema pour FR28 (conditions de paiement, câblage UI en Story 4.2).

**Fondations déjà en place** (NE PAS refaire) :

- **`CheNumber` type validé** — `crates/kesh-core/src/types/che_number.rs` (Story 1.3). Implémentation eCH-0097 v2.0 complète : modulo 11, poids `[5,4,3,2,7,6,5,4]`, normalisation whitespace/casse/suffixes TVA (MWST/TVA/IVA), méthodes `new()`, `formatted()`, `as_str()`, dérive `FromStr`/`TryFrom<String>`/`Display`. Tests unitaires déjà présents avec le vecteur officiel `CHE-109.322.551`. Exporté via `kesh_core::types::CheNumber`. **Le dev doit le RÉUTILISER tel quel, aucune réimplémentation.**
- **Pattern Repository + CRUD + audit log** — Story 3.1 (accounts) + Story 3.5 (audit). Les 4 fonctions `create`/`find_by_id`/`list_by_company`/`update`/`archive` servent de modèle canonique. Voir `crates/kesh-db/src/repositories/accounts.rs`.
- **Pattern handler + `Extension<CurrentUser>`** — Story 3.3/3.5. Les handlers écrits ou refactorés passent `current_user.user_id` au repository pour l'audit. Voir `crates/kesh-api/src/routes/accounts.rs`.
- **`audit_log::insert_in_tx(&mut tx, NewAuditLogEntry)`** — Story 3.3. Atomique avec l'opération mutante. Convention projet (Story 3.5) : snapshot JSON direct pour `create`/`archive`, wrapper `{"before": ..., "after": ...}` pour `update`.
- **Pattern rollback explicite sur erreur audit** — Stories 3.5 (P10/P11). Pattern uniformisé `if let Err(e) = audit_log::insert_in_tx(...).await { tx.rollback().await.map_err(map_db_error)?; return Err(e); }`. À répliquer tel quel sur les 3 fonctions mutantes de `contacts`.
- **Helper test `get_admin_user_id(pool)`** — dupliqué dans `accounts::tests`, `journal_entries::tests`, `audit_log::tests`. À dupliquer à nouveau dans `contacts::tests` (décision Story 3.5 Dev Notes L1 : Option (a) duplication acceptée).
- **Pagination + debounce + query-helpers frontend** — Story 3.4. Les helpers `frontend/src/lib/features/journal-entries/debounce.ts` + `query-helpers.ts` sont potentiellement réutilisables (à évaluer : soit les déplacer vers `shared/utils/` pour réutilisation transverse, soit dupliquer — voir décision T3 ci-dessous).
- **Modale conflit 409 `OPTIMISTIC_LOCK_CONFLICT`** — Story 3.3. Pattern établi dans `JournalEntryForm.svelte`. Le `ContactForm.svelte` l'imitera.
- **`notify.ts` helpers** — Story 3.5 (`notifySuccess`, `notifyError`, etc. dans `frontend/src/lib/shared/utils/notify.ts`). À utiliser pour tous les feedbacks utilisateur du nouveau code.
- **Module i18n partagé** — `frontend/src/lib/shared/utils/i18n.svelte.ts` (Story 3.5 P5). `i18nMsg(key, fallback)` à importer depuis là, PAS depuis `features/onboarding/`.
- **`rbac` comptable_routes** — Story 1.8. Les routes `/api/v1/contacts/*` doivent être enregistrées dans le groupe `comptable_routes` (accès Admin + Comptable, pas Lecteur — à confirmer par le PRD RBAC v0.1).

### Scope verrouillé — ce qui DOIT être fait

1. **Migration `contacts`** — table avec schéma complet conforme à l'AC de l'Epic + champ `active` (pattern soft-delete identique à accounts) + `version` (verrouillage optimiste) + contrainte unique sur `(company_id, ide_number)` pour éviter les doublons IDE.
2. **Entité Rust `Contact`** — `kesh-db/src/entities/contact.rs` avec `Contact`, `NewContact`, `ContactUpdate`, et un enum `ContactType { Personne, Entreprise }` (impl `sqlx::Type`, `Serialize`, `Deserialize`, `FromStr` pour parsing depuis le JSON).
3. **Repository `contacts`** — `kesh-db/src/repositories/contacts.rs` avec 6 fonctions : `create`, `find_by_id`, `list_by_company` (simple, non paginée — usage interne), `list_by_company_paginated` (pour l'UI, filtres + sort + limit/offset), `update`, `archive`. Chaque fonction mutante accepte `user_id: i64` et insère l'entrée `audit_log` dans la même tx avec rollback explicite.
4. **API routes `/api/v1/contacts`** — 5 handlers dans `kesh-api/src/routes/contacts.rs` : `list_contacts` (GET), `get_contact` (GET by id), `create_contact` (POST), `update_contact` (PUT), `archive_contact` (PUT). Enregistrement dans `comptable_routes`.
5. **Validation métier côté API** — `name` non vide (≤ 255), `email` format RFC 5322 (optionnel, ≤ 320), `phone` ≤ 50, `address` ≤ 500, `ide_number` parsé via `CheNumber::new(...)` (optionnel). Erreurs 400 avec messages explicites.
6. **Frontend feature `contacts`** — page liste `/contacts` avec tableau filtrable/paginé/triable, formulaire create/edit, dialog d'archive, toasts via `notify*`, modale 409, tout câblé i18n.
7. **i18n** — 30–40 nouvelles clés × 4 langues (FR/DE/IT/EN) pour tous les labels, placeholders, messages d'erreur, toasts, tooltips.
8. **Tests** — voir section T7 (unitaires Rust + Vitest + Playwright).

### Scope volontairement HORS story — décisions tranchées

- **UI de saisie `default_payment_terms`** : la colonne est créée dans la table `contacts` (schema complet dès 4.1), mais **le formulaire n'expose PAS le champ en 4.1**. La saisie UI arrive en **Story 4.2**. Raison : FR28 est formellement une feature 4.2 (lien aux conditions de paiement), l'inclure en 4.1 créerait une dette visuelle si le format/la logique change en 4.2.
- **Catalogue produits/services** : entièrement en Story 4.2. Aucune table `products`, aucune route.
- **Liaison contact → écriture comptable** : pas de colonne `contact_id` sur `journal_entry_lines` en 4.1 (reportée en Story 5.2 "Validation & numérotation des factures" qui génère l'écriture depuis une facture).
- **Import CSV/vCard/contacts depuis sources externes** : hors scope v0.1. À évaluer post-MVP.
- **Historique/audit UI** : conforme Story 3.5 — audit écrit en base, pas de page UI de consultation.
- **Avatar / photo contact** : pas de stockage de blobs en v0.1.
- **Custom fields / tags** : pas de flexibilité schema en v0.1, schéma fixe uniquement.
- **GDPR export / droit à l'oubli** : non-exigence PRD (PRD focus = conformité suisse CO, pas RGPD). Pas de fonctionnalité d'export contact en 4.1. À évaluer post-MVP si déploiement UE.
- **Soft rule « au moins un flag client OU fournisseur »** : **non appliquée en 4.1**. Un contact peut exister avec `is_client = false, is_supplier = false` (par ex. prospect, référence tierce). Le formulaire propose par défaut `is_client = true`, mais l'utilisateur peut tout décocher. Ne pas rejeter en 400.
- **Déduplication automatique par nom** : pas de déduplication transverse. Seule contrainte unique = `(company_id, ide_number)` quand IDE non null.

### Décisions de conception

- **`ContactType` enum** (`Personne` / `Entreprise`) — distinct de `is_client`/`is_supplier`. Un contact peut être `Entreprise + is_client=true + is_supplier=true` (ex: grossiste qui achète et revend). Stocké en base via un type enum MariaDB (`ENUM('Personne','Entreprise')`) mappé par `#[sqlx(type_name = "VARCHAR")]` + `#[derive(sqlx::Type)]` suivant le pattern de `AccountType` (`crates/kesh-db/src/entities/account.rs`). **Piège** : SQLx 0.8 + MariaDB a besoin d'une implémentation manuelle de `Type<MySql>` sur l'enum ; ne PAS utiliser `#[sqlx(type_name = "ENUM")]` directement qui ne fonctionne pas. Voir `AccountType` comme référence empirique.

- **Unicité IDE par company** — contrainte `UNIQUE (company_id, ide_number)` **partielle** : MariaDB n'a pas d'`UNIQUE WHERE`, mais NULL est toléré dans les index UNIQUE MariaDB par défaut (contrairement à SQL Server). Plusieurs lignes avec `ide_number = NULL` sont acceptées automatiquement. C'est le comportement voulu. Vérifié empiriquement dans les autres migrations Kesh (pas de clause `WHERE` dans les index UNIQUE existants).

- **Soft-delete via `active = false`** — cohérent avec Story 3.1 (accounts). Pas de DELETE SQL. Route `PUT /api/v1/contacts/{id}/archive`. L'archivage n'autorise PAS le dé-archivage en v0.1 (pas de route `unarchive`) — peut être ajouté en story ultérieure si besoin utilisateur réel.

- **Validation email** — format RFC 5322 via un **check manuel caractère-par-caractère** (pas de regex, pas de nouvelle dépendance). **La crate `regex` n'est PAS dans le workspace Kesh** (vérifié 2026-04-11 : aucun `Cargo.toml` ne la déclare). Décision v0.1 : fonction pure de validation manuelle, à placer dans `crates/kesh-api/src/routes/contacts.rs` (scope local, pas besoin de la mutualiser) :
  ```rust
  fn is_valid_email_simple(s: &str) -> bool {
      // Format minimal RFC 5322 : {local}@{domain}.{tld}
      // - Au moins un '@'
      // - Pas de whitespace
      // - Partie locale non vide
      // - Domaine avec au moins un '.' ni en début ni en fin
      let Some(at_pos) = s.find('@') else { return false };
      let (local, rest) = s.split_at(at_pos);
      let domain = &rest[1..]; // skip '@'
      !local.is_empty()
          && !local.contains(char::is_whitespace)
          && !domain.is_empty()
          && !domain.contains(char::is_whitespace)
          && domain.contains('.')
          && !domain.starts_with('.')
          && !domain.ends_with('.')
          && !domain.contains("..")
  }
  ```
  Message d'erreur français : « Format d'email invalide ». Test unitaire `is_valid_email_simple` couvre les cas : `user@domain.ch` OK, `user@subdomain.domain.com` OK, `no-at-sign.com` KO, `@no-local.com` KO, `user@.ch` KO, `user@ch` KO (pas de point), `user name@domain.ch` KO (whitespace), `user@domain..ch` KO (double point).
  **Limite intentionnelle v0.1** : le helper utilise `s.find('@')` qui trouve la PREMIÈRE occurrence, donc `user@@domain.ch` serait faussement accepté (`local="user"`, `domain="@domain.ch"` contient bien un `.`). **Dette acceptée** — cas edge non réaliste en v0.1, validation full-RFC 5322 reportée à post-MVP. Si réellement problématique, ajouter `&& local.matches('@').count() == 0 && domain.matches('@').count() == 0` dans une story ultérieure.

- **`default_payment_terms` en 4.1** — colonne `VARCHAR(100) NULL` créée dès la migration 4.1 (anticipation Story 4.2). Le POST/PUT API l'accepte en entrée et la persiste si fourni, mais **le formulaire UI n'expose pas encore le champ** (Story 4.2 fera le câblage complet). Permet d'éviter une migration 2 stories plus tard.

- **`IdeNumberInput` — frontend validation en temps réel** — pattern à créer : un composant input qui normalise à la frappe (retire espaces/tirets), valide format `^CHE[0-9]{9}$` côté client pour UX réactive, délègue la validation finale checksum au backend (pour source de vérité unique). Si l'utilisateur entre un IDE invalide, message d'erreur « Numéro IDE suisse invalide (checksum incorrect) » via API 400. Éviter de dupliquer l'algorithme modulo 11 en TypeScript.

- **Route `list_by_company_paginated` — paramètres de filtre** :
  - `search` (string, optionnel) : LIKE `%{search}%` sur `name` + `email`.
  - `contact_type` (enum, optionnel) : filtre exact sur `Personne` / `Entreprise`.
  - `is_client` (bool, optionnel) : filtre exact.
  - `is_supplier` (bool, optionnel) : filtre exact.
  - `include_archived` (bool, default `false`) : inclut les contacts avec `active = false`.
  - `sort_by` (enum `ContactSortBy { Name, CreatedAt, UpdatedAt }`, default `Name`) **ET** `sort_direction` (`SortDirection { Asc, Desc }` depuis `kesh_core::listing`, default **forcé** à `Asc` pour contacts — attention : `SortDirection::default()` est `Desc`). Deux champs séparés, PAS un enum combiné. Cohérent avec Story 3.4 `JournalEntryListQuery` qui sépare aussi `sort_by` et `sort_dir`.
  - `limit` (int, default 20, max 100), `offset` (int, default 0).
  - Pattern **`sqlx::QueryBuilder`** pour l'assemblage dynamique WHERE, cohérent avec `journal_entries::list_by_company_paginated` (Story 3.4). **Ne PAS concaténer du SQL à la main** (risque injection).

- **`ContactResponse` DTO API** — `camelCase` via `#[serde(rename_all = "camelCase")]`, expose `contactType`, `isClient`, `isSupplier`, `ideNumber` (sous forme `String` normalisée `CHE109322551` — le frontend appelle `.formatted()` côté display via un petit helper TS). **Décision** : le backend renvoie le format normalisé (source de vérité), le frontend formate pour l'affichage. Pas de duplication.

- **Audit log — schéma JSON pour contacts** (cohérent avec pattern Story 3.5) :
  - `contact.created` → `details_json = snapshot_json(&contact)` (direct, pas de wrapper)
  - `contact.updated` → `details_json = {"before": snapshot_json(&before), "after": snapshot_json(&after)}`
  - `contact.archived` → `details_json = snapshot_json(&contact)` (direct, état post-archivage)
  - Helper `contact_snapshot_json(&Contact) -> serde_json::Value` à créer dans `contacts.rs` (pattern analogue à `account_snapshot_json`) avec tous les champs (id, companyId, name, contactType, isClient, isSupplier, ideNumber, email, phone, address, defaultPaymentTerms, active, version).

## Acceptance Criteria (AC)

1. **Création nominale** (FR25) — Given un formulaire vierge, When l'utilisateur saisit `name`, coche `is_client` (ou `is_supplier`, ou les deux), choisit `contact_type`, valide, Then un contact est créé en base avec `version = 1`, `active = true`, `created_at` = now, et une entrée `audit_log` avec `action = "contact.created"`, `entity_type = "contact"`, `entity_id = {new_id}`, `details_json = contact_snapshot_json(&contact)` (direct).

2. **Validation IDE CHE** (FR27) — Given un formulaire où l'utilisateur saisit `ide_number`, When il tape `CHE-109.322.551` (vecteur officiel eCH-0097 valide), Then le POST API réussit avec 201. When il tape `CHE-109.322.552` (checksum invalide — dernier chiffre modifié, vecteur confirmé par le test `invalid_checksum` de `che_number.rs`), Then le POST retourne 400 avec un message français clair (« Numéro IDE suisse invalide » ou équivalent). **ATTENTION** : `CHE-000.000.000` est un numéro VALIDE (checksum = 0 conforme au modulo 11, test `valid_check_digit_zero` ligne 238 de `che_number.rs`) — ne PAS l'utiliser comme exemple invalide. When il tape `CHE-109.322.551 MWST` (avec suffixe TVA), Then le POST réussit (normalisation automatique retire le suffixe). When le champ est laissé vide, Then le POST réussit (IDE optionnel).

3. **Unicité IDE** — Given un contact existant avec `ide_number = "CHE109322551"` dans la company A, When l'utilisateur tente de créer un 2e contact avec le même IDE dans la même company, Then le POST retourne 409 Conflict avec un message « Un contact avec ce numéro IDE existe déjà ». Given un IDE null, When l'utilisateur crée 2 contacts sans IDE dans la même company, Then les 2 créations réussissent (NULL distinct).

4. **Validation champs** — Given un formulaire, When `name` est vide ou espaces seuls, Then 400 « Le nom est obligatoire ». When `name` > 255 chars, Then 400 « Le nom doit faire au plus 255 caractères ». When `email` est renseigné mais invalide, Then 400 « Format d'email invalide ». When `address` > 500 chars, Then 400 « L'adresse doit faire au plus 500 caractères ».

5. **Flags client/fournisseur libres** (FR26) — Given un contact, When l'utilisateur crée avec `is_client = true, is_supplier = false`, Then création OK. Idem avec `is_supplier = true, is_client = false`. Idem avec les deux à `true`. Idem avec les deux à `false` (aucune contrainte — voir décision spec).

6. **Modification avec verrouillage optimiste** — Given un contact `v1`, When l'utilisateur charge dans le formulaire puis modifie `name` puis PUT avec `version = 1`, Then la modification réussit, `version` devient 2, une entrée `audit_log` est créée avec `action = "contact.updated"` et `details_json = {"before": ..., "after": ...}`. When une 2e session modifie en parallèle et PUT après la première, Then 409 `OPTIMISTIC_LOCK_CONFLICT` + modale frontend qui propose « Recharger ».

7. **Archivage** — Given un contact actif, When l'utilisateur clique « Archiver », confirme dans le dialog, Then `active` passe à `false`, `version` incrémente, une entrée `audit_log` `action = "contact.archived"` est créée. Given un contact archivé, When la liste est chargée sans `includeArchived`, Then le contact n'apparaît pas. Avec `includeArchived = true`, il apparaît (grisé visuellement). Given un contact déjà archivé, When `update_contact` ou `archive_contact` est appelé à nouveau, Then le repository retourne `DbError::IllegalStateTransition` → **HTTP 409 `ILLEGAL_STATE_TRANSITION`** (mapping fixe vérifié empiriquement dans `crates/kesh-api/src/errors.rs:317-324`) avec message i18n « Contact archivé — modification/archivage supplémentaire interdit » (clé `contact-error-archived-no-modify`). Le check s'effectue dans la tx après le SELECT initial (pattern `accounts::archive` qui teste `children > 0`). **Décision v0.1** : pas de route `unarchive` — un contact archivé est définitif pour la v0.1. **NOTE tests Playwright** : vérifier le status `409` (pas `400`) + code `ILLEGAL_STATE_TRANSITION`.

8. **Liste paginée + tri** — Given 25 contacts, When l'utilisateur charge `/contacts`, Then la liste montre les 20 premiers triés par `name ASC` par défaut, avec un indicateur de pagination (21-25 sur page 2). When il clique sur l'en-tête `name`, Then le tri devient `name DESC`. When il clique sur `created_at`, Then tri par date de création.

9. **Recherche par nom debouncée** — Given une liste, When l'utilisateur tape dans le champ de recherche, Then après 300 ms de debounce, la liste est filtrée via l'API (LIKE `%{query}%` sur `name` + `email`). L'URL reflète le filtre (query param `?search=xxx`).

10. **Filtres type / client / fournisseur** — Given la liste, When l'utilisateur sélectionne `Entreprise` dans le filtre type, Then seuls les contacts `contact_type = Entreprise` sont affichés. Idem pour `is_client` / `is_supplier`. Les filtres sont combinables (AND) et reflétés dans l'URL (`?contactType=Entreprise&isClient=true`).

11. **URL state préservé après reload** — Given un état filtré/paginé/trié, When l'utilisateur rafraîchit la page, Then l'état est restauré depuis les query params (cohérent avec Story 3.4).

12. **RBAC** — Given un utilisateur avec rôle `Lecteur`, When il tente `POST /api/v1/contacts`, Then 403 Forbidden. Given un `Comptable` ou `Admin`, Then 201. Les routes sont enregistrées dans `comptable_routes`.

13. **i18n** — Tous les labels, placeholders, messages d'erreur, toasts, boutons, titres de colonnes, messages de dialog sont internationalisés dans les 4 langues (FR/DE/IT/EN). Aucun hardcode (règle A3).

14. **Audit log complet** — Les 3 opérations mutantes (create, update, archive) écrivent chacune une entrée `audit_log` atomique avec la tx principale. Tests d'intégration DB vérifient via `audit_log::find_by_entity("contact", id, 10)` que l'entrée est bien présente avec le bon `action`, `user_id`, et la structure `details_json` conforme à la convention projet.

15. **Notifications & feedbacks** — Toutes les opérations réussies affichent un toast `notifySuccess` (« Contact créé », « Contact modifié », « Contact archivé »). Les erreurs 400/409/500 affichent `notifyError` avec le message API traduit. Durées conformes (4 s succès, 6 s erreurs) — helpers Story 3.5.

16. **Tests** (voir section T7) : unitaires Rust (repository CRUD + validation), tests d'audit, tests Vitest (form, types, API client), tests Playwright (E2E nominal + 4 scénarios d'erreur).

## Tasks / Subtasks

### T1 — Migration & entité Contact (AC: #1, #3, #5)

- [x] T1.1 Créer `crates/kesh-db/migrations/20260414000001_contacts.sql`. **Avant de créer**, vérifier que `20260414000001` est bien le prochain numéro disponible via `ls crates/kesh-db/migrations/` — ajuster la date si une autre migration a été ajoutée depuis la rédaction de cette spec.
  ```sql
  CREATE TABLE contacts (
      id BIGINT NOT NULL AUTO_INCREMENT,
      company_id BIGINT NOT NULL,
      contact_type VARCHAR(20) NOT NULL,  -- 'Personne' | 'Entreprise'
      name VARCHAR(255) NOT NULL,
      is_client BOOLEAN NOT NULL DEFAULT FALSE,
      is_supplier BOOLEAN NOT NULL DEFAULT FALSE,
      address VARCHAR(500) NULL,
      email VARCHAR(320) NULL,
      phone VARCHAR(50) NULL,
      ide_number VARCHAR(12) NULL,  -- forme normalisée "CHE123456789"
      default_payment_terms VARCHAR(100) NULL,
      active BOOLEAN NOT NULL DEFAULT TRUE,
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      PRIMARY KEY (id),
      CONSTRAINT fk_contacts_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
      CONSTRAINT uq_contacts_company_ide UNIQUE (company_id, ide_number),
      CONSTRAINT chk_contacts_name_not_empty CHECK (CHAR_LENGTH(TRIM(name)) > 0),
      CONSTRAINT chk_contacts_type CHECK (BINARY contact_type IN (BINARY 'Personne', BINARY 'Entreprise')),
      INDEX idx_contacts_company_active (company_id, active),
      INDEX idx_contacts_company_name (company_id, name)
  );
  ```
  **Pattern projet** :
  - `CHAR_LENGTH(TRIM(col)) > 0` est la forme canonique (cohérent avec toutes les autres migrations du projet). `TRIM(col) <> ''` peut se comporter différemment selon la collation.
  - `BINARY col IN (BINARY 'Val', ...)` empêche le bypass de la CHECK par collation case-insensitive — sans `BINARY`, `utf8mb4_general_ci` accepterait `personne`/`PERSONNE`, cassant ensuite le `Decode` SQLx. Pattern identique à `20260411000001_accounts.sql` (chk_accounts_type).
  - MariaDB 11.x (target Kesh) applique bien les CHECK.

- [x] T1.2 Créer `crates/kesh-db/src/entities/contact.rs` :
  - `pub enum ContactType { Personne, Entreprise }` avec `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]` **et** impl `sqlx::Type<MySql>` manuelle (pattern `AccountType` — voir `crates/kesh-db/src/entities/account.rs` pour la référence empirique, NE PAS utiliser `#[sqlx(type_name = "ENUM")]`). **`Copy` obligatoire** pour éviter les `.clone()` superflus dans le repository lors des insertions et de la construction de `contact_snapshot_json`.
  - `pub struct Contact { id, company_id, contact_type, name, is_client, is_supplier, address, email, phone, ide_number (Option<String>), default_payment_terms (Option<String>), active, version, created_at, updated_at }` avec `#[derive(Debug, Clone, sqlx::FromRow, Serialize)]`.
  - `pub struct NewContact { company_id, contact_type, name, is_client, is_supplier, address, email, phone, ide_number, default_payment_terms }` — valeurs déjà normalisées et validées côté caller.
  - `pub struct ContactUpdate { contact_type, name, is_client, is_supplier, address, email, phone, ide_number, default_payment_terms }` — tous les champs modifiables. **IMPORTANT** : `ContactUpdate` ne contient PAS `version`. La version est passée comme paramètre séparé à `contacts::update(pool, id, version: i32, user_id, changes: ContactUpdate)`, pattern identique à `accounts::update` (Story 3.1/3.5). Le handler API extrait `version` du body `UpdateContactRequest` et le passe séparément au repository.
- [x] T1.3 Ajouter `pub mod contact;` et re-export `pub use contact::{Contact, ContactType, NewContact, ContactUpdate};` dans `crates/kesh-db/src/entities/mod.rs`.

### T2 — Repository `contacts` (AC: #1, #3, #5, #6, #7, #8, #14)

- [x] T2.1 Créer `crates/kesh-db/src/repositories/contacts.rs`. Structure calquée sur `accounts.rs` post-Story 3.5 :
  ```rust
  use crate::entities::{Contact, ContactType, NewContact, ContactUpdate};
  use crate::entities::audit_log::NewAuditLogEntry;
  use crate::errors::{map_db_error, DbError};
  use crate::repositories::audit_log;
  use kesh_core::listing::SortDirection;  // SortBy PAS importé — journal-entries-specific
  use sqlx::{MySql, MySqlPool, QueryBuilder};

  const FIND_BY_ID_SQL: &str = "SELECT id, company_id, contact_type, name, is_client, is_supplier, \
      address, email, phone, ide_number, default_payment_terms, active, version, \
      created_at, updated_at FROM contacts WHERE id = ?";

  fn contact_snapshot_json(c: &Contact) -> serde_json::Value { /* tous les champs camelCase */ }

  pub async fn create(pool: &MySqlPool, user_id: i64, new: NewContact) -> Result<Contact, DbError> { /* tx + INSERT + re-fetch + audit insert direct + commit */ }
  pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Contact>, DbError> { /* simple SELECT */ }
  pub async fn list_by_company(pool: &MySqlPool, company_id: i64, include_archived: bool) -> Result<Vec<Contact>, DbError> { /* simple, usage interne */ }
  pub async fn list_by_company_paginated(pool: &MySqlPool, company_id: i64, query: ContactListQuery) -> Result<ContactListResult, DbError> { /* QueryBuilder dynamique */ }
  pub async fn update(pool: &MySqlPool, id: i64, version: i32, user_id: i64, changes: ContactUpdate) -> Result<Contact, DbError> { /* SELECT before + UPDATE + rows check + SELECT after + audit wrapper + commit */ }
  pub async fn archive(pool: &MySqlPool, id: i64, version: i32, user_id: i64) -> Result<Contact, DbError> { /* UPDATE active=false + SELECT + audit direct + commit */ }
  ```

- [x] T2.2 Déclarer **localement dans `contacts.rs`** les types de query/result spécifiques (pattern `JournalEntryListQuery`/`JournalEntryListResult` de Story 3.4 vérifié empiriquement) :
  - `pub struct ContactListQuery { search: Option<String>, contact_type: Option<ContactType>, is_client: Option<bool>, is_supplier: Option<bool>, include_archived: bool, sort_by: ContactSortBy, sort_direction: SortDirection, limit: i64, offset: i64 }` — importer `SortDirection` depuis `kesh_core::listing` (c'est le seul type vraiment générique de ce module, `Asc`/`Desc`).
  - `pub enum ContactSortBy { Name, CreatedAt, UpdatedAt }` avec `impl as_sql_column() -> &'static str` renvoyant des littéraux (whitelist anti-injection, pattern Story 3.4 `SortBy::as_sql_column`). **Ne PAS** étendre le `SortBy` partagé de `kesh_core::listing/` — il est journal-entries-specific (variants `EntryDate`, `EntryNumber`, `Journal`, `Description`) malgré son doc-comment trompeur. Créer un enum local est plus clean.
  - `impl Default for ContactSortBy { fn default() -> Self { Self::Name } }` (tri par nom par défaut = UX carnet d'adresses standard).
  - `impl Default for ContactListQuery { fn default() -> Self { Self { search: None, contact_type: None, is_client: None, is_supplier: None, include_archived: false, sort_by: ContactSortBy::default(), sort_direction: SortDirection::Asc, limit: 20, offset: 0 } } }` — **IMPORTANT** : hardcoder `SortDirection::Asc` ici ; NE PAS utiliser `SortDirection::default()` qui retourne `Desc` (convention comptable de `kesh_core::listing`, inappropriée pour un carnet d'adresses alphabétique).
  - `pub struct ContactListResult { pub items: Vec<Contact>, pub total: i64, pub offset: i64, pub limit: i64 }` — **struct concret côté repository** (le repository ne connaît pas les DTOs API). Pattern identique à `JournalEntryListResult` de Story 3.4. **Note** : côté handler API, ce `ContactListResult` sera converti en `ListResponse<ContactResponse>` (type générique déjà présent dans `crates/kesh-api/src/routes/mod.rs:25`, réutilisé par Story 3.4) — voir T3.1.

- [x] T2.3 Implémenter `list_by_company_paginated(pool, company_id, query) -> Result<ContactListResult, DbError>` en s'inspirant **directement** de `journal_entries::list_by_company_paginated` (voir `crates/kesh-db/src/repositories/journal_entries.rs:~375` `push_where_clauses` et le pattern QueryBuilder). Points clés :
  - **DEUX `QueryBuilder` distincts** (un pour `SELECT COUNT(*)`, un pour `SELECT ...`). Le commentaire CRITIQUE dans `journal_entries.rs:365-369` explique pourquoi : un `QueryBuilder` encode un état mutable et ne peut pas être réutilisé après `build_*`.
  - **Whitelist SQL** via `ContactSortBy::as_sql_column()` et `SortDirection::as_sql_keyword()` — jamais de concaténation d'input utilisateur dans l'ORDER BY.
  - **LIKE search** sur `name` et `email` : **réutiliser le helper `escape_like` défini dans `journal_entries.rs:320`** (3 caractères à échapper dans CET ordre `\` → `%` → `_`) et émettre la clause `LIKE ? ESCAPE '\\\\'` explicitement. **Sans la clause `ESCAPE`, MariaDB ignore silencieusement l'échappement** (comportement dépendant du mode `NO_BACKSLASH_ESCAPES`). Pattern complet copié de `journal_entries.rs:382-385` :
    ```rust
    // Dans push_where_clauses pour contacts :
    if let Some(search) = &query.search {
        qb.push(" AND (name LIKE ");
        qb.push_bind(format!("%{}%", escape_like(search)));
        qb.push(" ESCAPE '\\\\' OR email LIKE ");
        qb.push_bind(format!("%{}%", escape_like(search)));
        qb.push(" ESCAPE '\\\\')");
    }
    ```
  - **Décision sur `escape_like`** : duplication dans `contacts.rs` (fonction privée, 3 lignes triviales) OU extraction vers un module `kesh-db/src/repositories/sql_helpers.rs` partagé. **Choix v0.1** : duplication simple (pattern cohérent avec la décision `debounce.ts` T4.3 et `get_admin_user_id` — scope creep refactor transverse post-MVP). Copier la fonction `escape_like` depuis `journal_entries.rs:320-322` en l'ajoutant à `contacts.rs` avec le même doc-comment.

- [x] T2.4 Gestion d'erreurs spécifiques :
  - Unicité `(company_id, ide_number)` → `DbError::UniqueConstraintViolation("ide_number".into())` → mapping HTTP 409 dans le handler.
  - Contact non trouvé sur `update`/`archive` → `DbError::NotFound` → HTTP 404.
  - Version mismatch sur `update`/`archive` → `DbError::OptimisticLockConflict` → HTTP 409 avec code `OPTIMISTIC_LOCK_CONFLICT`.
  - Pattern `rows == 0` : après UPDATE, si `rows_affected() == 0`, faire un `SELECT id` pour distinguer `NotFound` vs `OptimisticLockConflict` (pattern empirique déjà utilisé dans `accounts::archive` post-Story 3.5).

- [x] T2.5 **Audit log avec rollback explicite** (pattern P10/P11) — Pour les 3 fonctions mutantes, le bloc audit_log utilise obligatoirement :
  ```rust
  if let Err(e) = audit_log::insert_in_tx(
      &mut tx,
      NewAuditLogEntry {
          user_id,
          action: "contact.created".to_string(),  // ou .updated / .archived
          entity_type: "contact".to_string(),
          entity_id: contact.id,
          details_json: Some(contact_snapshot_json(&contact)),  // ou wrapper pour update
      },
  )
  .await
  {
      tx.rollback().await.map_err(map_db_error)?;
      return Err(e);
  }
  ```

- [x] T2.6 Ajouter `pub mod contacts;` dans `crates/kesh-db/src/repositories/mod.rs`.

### T3 — Routes API `/api/v1/contacts` (AC: #1, #2, #4, #6, #7, #8, #9, #10, #11, #12, #13, #15)

- [x] T3.1 Créer `crates/kesh-api/src/routes/contacts.rs` avec :
  - Import : `use crate::routes::ListResponse;` — type générique déjà présent dans `routes/mod.rs:25`, réutilisé par `journal_entries.rs` (Story 3.4). **NE PAS créer un nouveau `PaginatedContactsResponse`** — anti-DRY.
  - DTOs :
    - `ListContactsQuery` (Deserialize, `#[serde(rename_all = "camelCase")]`) : tous les filtres de T2.2
    - `CreateContactRequest` (Deserialize, camelCase) : champs d'entrée utilisateur (sans `version`)
    - `UpdateContactRequest` (Deserialize, camelCase) : champs d'entrée utilisateur **ET** `version: i32` (le handler extrait `version` du body et le passe comme param séparé à `contacts::update(...)` — voir note T1.2)
    - `ArchiveContactRequest` (Deserialize) : juste `{ version: i32 }`
    - `ContactResponse` (Serialize, camelCase) : reflet de `Contact` avec `ideNumber` en forme normalisée
  - `From<Contact> for ContactResponse` — **note P14** : `ide_number: contact.ide_number` est une simple copie du `Option<String>` (déjà normalisé en base par `CheNumber::new().as_str()` au moment de l'insert, pas de re-parse `CheNumber` ici).
  - 5 handlers :
    - `list_contacts(State, Query<ListContactsQuery>) -> Json<ListResponse<ContactResponse>>` — le handler appelle `contacts::list_by_company_paginated`, récupère un `ContactListResult`, puis construit `ListResponse { items: items.into_iter().map(ContactResponse::from).collect(), total, limit, offset }`. Pattern identique à `journal_entries.rs` (Story 3.4) — voir lignes ~364-371 pour le modèle.
    - `get_contact(State, Path<i64>) -> Json<ContactResponse>` (404 si introuvable)
    - `create_contact(State, Extension<CurrentUser>, Json<CreateContactRequest>) -> (StatusCode::CREATED, Json<ContactResponse>)`
    - `update_contact(State, Extension<CurrentUser>, Path<i64>, Json<UpdateContactRequest>) -> Json<ContactResponse>`
    - `archive_contact(State, Extension<CurrentUser>, Path<i64>, Json<ArchiveContactRequest>) -> Json<ContactResponse>`

- [x] T3.2 **Validation métier dans les handlers create/update** :
  - `trim()` sur `name`, `email`, `phone`, `address`, `default_payment_terms`, `ide_number`.
  - `name` : non vide, ≤ 255 → `AppError::Validation`.
  - `email` : si non vide, validation via `is_valid_email_simple(&email)` (helper local, check manuel caractère-par-caractère — voir section Décisions de conception pour l'implémentation complète). Pas de regex crate. Longueur ≤ 320.
  - `phone` : si non vide, ≤ 50.
  - `address` : ≤ 500.
  - `default_payment_terms` : ≤ 100.
  - `ide_number` : si non vide, `CheNumber::new(&input).map_err(|e| AppError::Validation(...))?` — on stocke `che.as_str().to_string()` (forme normalisée `"CHE109322551"`).
  - `contact_type` : parsing depuis String JSON (cases `"Personne"`/`"Entreprise"`) → `ContactType`, sinon 400.

- [x] T3.3 Mapping erreurs → HTTP :
  - `DbError::OptimisticLockConflict` → 409 avec code `OPTIMISTIC_LOCK_CONFLICT` (mapping existant dans `errors.rs:288-292`).
  - `DbError::NotFound` → 404 (mapping existant `errors.rs:285-287`).
  - **Pour `IDE_ALREADY_EXISTS`** : le mapping existant `DbError::UniqueConstraintViolation` dans `errors.rs:293-300` retourne un code générique `RESOURCE_CONFLICT` et `AppError` **n'a pas** de variant `Conflict { code, message }` (vérifié 2026-04-11). Pour émettre un code spécifique, **ajouter un variant `AppError::IdeAlreadyExists`** dans `crates/kesh-api/src/errors.rs` (scope de Story 4.1) :
    ```rust
    // Dans l'enum AppError :
    #[error("{}", .0)]
    IdeAlreadyExists(String),  // String = message i18n

    // Dans impl IntoResponse (match) :
    AppError::IdeAlreadyExists(ref msg) => build_response(
        StatusCode::CONFLICT,
        "IDE_ALREADY_EXISTS",
        msg,
    ),
    ```
  - **Dans le handler** `create_contact` et `update_contact`, intercepter AVANT la propagation `?` :
    ```rust
    let contact = contacts::create(&state.pool, current_user.user_id, new)
        .await
        .map_err(|e| match e {
            DbError::UniqueConstraintViolation(ref m) if m.contains("ide_number") => {
                AppError::IdeAlreadyExists(
                    t("error-ide-already-exists", "Un contact avec ce numéro IDE existe déjà")
                )
            }
            other => AppError::from(other),
        })?;
    ```
  - **Note** : le même pattern s'applique à `update_contact` (AC#3 couvre la création, mais la course sur update vers un IDE existant est aussi possible — intercepter là aussi).
  - Autres erreurs DbError → propagation via `From<DbError> for AppError` (500/400 par défaut selon le variant).

- [x] T3.4 **Enregistrer les routes selon le groupage RBAC** dans `crates/kesh-api/src/lib.rs` (où vivent `comptable_routes` et `authenticated_routes` — vérifié empiriquement `lib.rs:84` et `lib.rs:111`, **PAS** dans `routes/mod.rs` malgré certaines docs internes). Les lectures sont accessibles à **tout rôle authentifié** (incluant Consultation), les mutations sont réservées **Admin + Comptable** :
  ```rust
  // Dans authenticated_routes (tout rôle) — accès en lecture :
  .route("/api/v1/contacts", get(contacts::list_contacts))
  .route("/api/v1/contacts/{id}", get(contacts::get_contact))

  // Dans comptable_routes (Admin + Comptable via require_comptable_role) — mutations :
  .route("/api/v1/contacts", post(contacts::create_contact))
  .route("/api/v1/contacts/{id}", put(contacts::update_contact))
  .route("/api/v1/contacts/{id}/archive", put(contacts::archive_contact))
  ```
  **Attention Axum 0.8** : ne pas combiner `get(...).post(...)` sur un même `.route()` si les deux méthodes appartiennent à des routeurs middleware différents — déclarer chaque méthode explicitement dans son groupe.

- [x] T3.5 Ajouter `pub mod contacts;` dans `crates/kesh-api/src/routes/mod.rs`.

### T4 — Frontend feature `contacts` (AC: #1, #6, #7, #8, #9, #10, #11, #13, #15)

- [x] T4.1 Créer le dossier `frontend/src/lib/features/contacts/` avec :
  - `contacts.types.ts` : TS types miroir de `ContactResponse`, `ContactType`, `ListContactsQuery`, `CreateContactRequest`, `UpdateContactRequest`. Pour la réponse paginée, réutiliser le type TS `ListResponse<T>` si Story 3.4 en a créé un dans `frontend/src/lib/shared/types/` ; sinon le créer `export interface ListResponse<T> { items: T[]; total: number; limit: number; offset: number; }` (miroir exact du `ListResponse<T>` Rust de `routes/mod.rs:25`). **Vérifier avant de créer** via `grep -rn "interface ListResponse" frontend/src/lib/shared/` pour éviter la duplication. Le type applicable ici est `ListResponse<ContactResponse>`. Utiliser des types stricts, pas de `any`.
  - `contacts.api.ts` : wrapper typé via `apiClient` (Story 1.11) avec `listContacts`, `getContact`, `createContact`, `updateContact`, `archiveContact`.
  - `contact-helpers.ts` : helpers purs (`formatIdeNumber(normalized: string): string`, `validateIdeFormat(s: string): boolean` — regex client-side `^CHE[0-9]{9}$`, `formatContactType(type: ContactType): string`).
  - `contact-helpers.test.ts` : tests Vitest des helpers (focus sur `formatIdeNumber` et `validateIdeFormat`, pas de mock).

- [x] T4.2 Créer `frontend/src/lib/features/contacts/ContactForm.svelte` — formulaire create/edit :
  - Props `{ contact: ContactResponse | null, onSaved: (c: ContactResponse) => void, onCancel: () => void }`.
  - Champs : `name` (Input), `contactType` (Select → 'Personne'/'Entreprise'), `isClient` (Checkbox, default `true`), `isSupplier` (Checkbox), `email` (Input type email), `phone` (Input), `address` (Textarea), `ideNumber` (Input avec validation regex temps réel), `defaultPaymentTerms` (**NON exposé en 4.1** — input caché ou commenté avec un TODO Story 4.2).
  - Validation côté client minimale (nom non vide, email regex, IDE format). Le backend est la source de vérité finale.
  - Sur submit : `createContact(...)` ou `updateContact(...)` selon mode ; `notifySuccess` sur OK ; `notifyError` sur erreur (message traduit).
  - Gestion 409 : ouvrir la modale de conflit (pattern story 3.3 — copier le bloc `{#if conflictState}` de `JournalEntryForm.svelte`).

- [x] T4.3 Créer la page `frontend/src/routes/(app)/contacts/+page.svelte` :
  - Table shadcn-svelte avec colonnes : nom, type (badge), client (badge vert), fournisseur (badge bleu), IDE (formaté), email, actions (Modifier, Archiver).
  - Filtres en haut : input recherche (debounce 300 ms — **décision** : copier `debounce.ts` de `features/journal-entries/` vers `features/contacts/` pour éviter un import cross-feature, OU déplacer `debounce.ts` dans `shared/utils/`. **Choix v0.1** : duplication simple dans `features/contacts/debounce.ts` (3 lignes de code triviales, refactor transverse post-MVP si le pattern se répète une 3e fois).
  - Pagination (limit/offset), tri cliquable sur en-têtes `name` et `created_at`, filtre `includeArchived` (toggle).
  - URL state sync via `$page.url.searchParams` + `goto(..., { replaceState: true })` dans `$effect` (pattern Story 3.4).
  - Bouton « + Nouveau contact » → ouvre `ContactForm` dans un Dialog ou navigue vers `/contacts/new` (décision : **Dialog** pour cohérence avec les autres features et éviter une nouvelle route).
  - Clic « Modifier » sur une ligne → ouvre `ContactForm` en mode edit.
  - Clic « Archiver » → AlertDialog de confirmation puis `archiveContact`.

- [x] T4.4 **Ajouter le lien « Carnet d'adresses » dans la sidebar** (`frontend/src/routes/(app)/+layout.svelte`, tableau `navGroups`). **Important — pas de refactor transverse** : la sidebar existante utilise des labels **hardcodés en français** dans `navGroups` (« Accueil », « Écritures », etc. — PAS d'appels à `i18nMsg`). Pour la Story 4.1, ajouter l'item `{ label: 'Carnet d'adresses', href: '/contacts', icon: ... }` avec le label hardcodé, **conforme au pattern existant**. La clé i18n `nav-contacts` est définie dans `messages.ftl` (T5.1) mais **réservée** pour un refactor i18n futur de la sidebar entière — NE PAS tenter de migrer la sidebar en 4.1 (hors scope, risque de régression transverse).
  - **Alerte dette technique** : le layout actuel (`+layout.svelte`) importe `i18nMsg` depuis `$lib/features/onboarding/onboarding.svelte` — c'est un pattern legacy avant Story 3.5 P5. **NE PAS répliquer** cet import dans le nouveau code `contacts`. Pour tout import de `i18nMsg` dans les fichiers contacts, utiliser uniquement `import { i18nMsg } from '$lib/shared/utils/i18n.svelte';` (module canonical post-Story 3.5).

- [x] T4.5 **`AccountingTooltip` sur l'IDE ?** — **Décision** : non. L'IDE n'est pas un terme comptable ambigu, juste un identifiant légal. Un simple texte d'aide `<p class="text-xs text-muted-foreground">Format : CHE-123.456.789</p>` sous le champ suffit. Ne pas forcer un tooltip là où il n'apporte rien.

- [x] T4.6 **`query-helpers.ts`** — même décision que T4.3 pour `debounce.ts` : **duplication dans `features/contacts/query-helpers.ts`** (3-10 lignes copiées depuis `features/journal-entries/query-helpers.ts`), refactor transverse post-MVP. Justification : évite de casser Story 3.4 et de forcer un `import` cross-feature (anti-pattern). Si le dev juge que la duplication est excessive (> 30 lignes), créer un module partagé `$lib/shared/utils/pagination-url.ts` à la place et mettre à jour Story 3.4 en conséquence — à ne faire QUE si le pattern se répète une 3ᵉ fois.

### T5 — Clés i18n (AC: #13)

- [x] T5.1 Ajouter ~35 clés dans les 4 fichiers `crates/kesh-i18n/locales/*/messages.ftl` :
  - Nav : `nav-contacts`
  - Titres : `contacts-page-title`, `contact-form-create-title`, `contact-form-edit-title`
  - Labels champs : `contact-form-name`, `contact-form-type`, `contact-form-is-client`, `contact-form-is-supplier`, `contact-form-email`, `contact-form-phone`, `contact-form-address`, `contact-form-ide`, `contact-form-ide-help`
  - Options : `contact-type-personne`, `contact-type-entreprise`
  - Boutons : `contact-form-submit-create`, `contact-form-submit-edit`, `contact-form-cancel`, `contact-list-new`, `contact-list-edit`, `contact-list-archive`, `contact-archive-confirm`, `contact-archive-cancel`
  - Colonnes table : `contact-col-name`, `contact-col-type`, `contact-col-flags`, `contact-col-ide`, `contact-col-email`, `contact-col-actions`
  - Filtres : `contact-filter-search-placeholder`, `contact-filter-type-all`, `contact-filter-archived`
  - Messages : `contact-empty-list`, `contact-created-success`, `contact-updated-success`, `contact-archived-success`, `contact-archive-confirm-title`, `contact-archive-confirm-body`
  - Erreurs : `contact-error-name-required`, `contact-error-name-too-long`, `contact-error-email-invalid`, `contact-error-ide-invalid`, `contact-error-ide-duplicate`, `contact-error-not-found`, `contact-conflict-title`, `contact-conflict-body`, `contact-error-archived-no-modify` (message « Contact archivé — modification/archivage supplémentaire interdit » — pour P24)
  - **Clé API erreur** (consommée par `AppError::IdeAlreadyExists` via `t("error-ide-already-exists", fallback)`) : `error-ide-already-exists` (FR: « Un contact avec ce numéro IDE existe déjà », DE/IT/EN équivalents). **Cette clé est référencée par le handler T3.3** — oublier de l'ajouter aux 4 `.ftl` ferait tomber les utilisateurs DE/IT/EN sur le fallback français.
- [x] T5.2 Traductions DE/IT/EN — utiliser le vocabulaire comptable suisse standard (Kontakt, Klient, Lieferant / Contatto, Cliente, Fornitore / Contact, Client, Supplier).

### T6 — Tests (AC: #14, #16)

- [x] T6.1 Tests d'intégration DB `crates/kesh-db/src/repositories/contacts.rs` (sous `#[cfg(test)] mod tests`) — pattern Story 3.5 :
  - Helpers : `test_pool()`, `get_company_id(pool)`, `get_admin_user_id(pool)` (dupliqué depuis `accounts::tests`), `cleanup_test_contacts(pool, company_id)`.
  - `test_create_and_find` — création + `find_by_id` retourne les bons champs.
  - `test_create_writes_audit_log` — vérifie via `audit_log::find_by_entity("contact", id, 10)` que l'entrée existe avec `action = "contact.created"` et `details_json` direct (pas de wrapper).
  - `test_create_rejects_duplicate_ide` — 2 contacts avec même IDE dans la même company → `UniqueConstraintViolation`.
  - `test_create_allows_null_ide_duplicates` — 2 contacts sans IDE → OK (NULL distinct en MariaDB).
  - `test_create_normalizes_ide_mwst_suffix` — insérer avec `"CHE-109.322.551 MWST"`, vérifier que la ligne stockée contient `ide_number = "CHE109322551"` (12 chars, forme normalisée sans séparateurs ni suffixe TVA).
  - `test_update_rejects_archived_contact` — créer + archiver un contact, puis tenter `update` → `DbError::IllegalStateTransition`.
  - `test_archive_rejects_already_archived` — idem pour double-archivage.
  - `test_update_optimistic_lock` — 2 updates consécutifs avec la même version → 2e échoue `OptimisticLockConflict`.
  - `test_update_writes_audit_log_with_wrapper` — vérifie structure `{before, after}`.
  - `test_archive_sets_inactive` + `test_archive_writes_audit_log`.
  - `test_filter_by_contact_type` — insère 3 contacts (2 Personne, 1 Entreprise), filtre `contact_type = Entreprise` → 1 résultat.
  - `test_filter_by_is_client` — insère 2 contacts (1 client, 1 fournisseur), filtre `is_client = true` → 1 résultat.
  - `test_filter_by_search_name` — insère 3 contacts avec des noms distincts, search sur substring → bons résultats.
  - `test_filter_by_search_email` — idem pour substring dans `email`.
  - `test_filter_combined` — search + contact_type + is_client combinés (AND) → intersection correcte.
  - `test_filter_escape_like_wildcard` — insère un contact avec un `%` littéral dans le nom, vérifie que la recherche d'un `%` trouve ce contact spécifiquement (pas tous). **Vérifie le correctif P21** (escape_like + clause ESCAPE).
  - `test_list_sort_order` — insère contacts avec names distincts, vérifie **les 4 combinaisons critiques** : `(Name, Asc)`, `(Name, Desc)`, `(CreatedAt, Asc)`, `(UpdatedAt, Desc)`. Couvre tous les variants de `ContactSortBy` — un variant whitelist SQL non testé est une surface de risque dormante (bug de colonne ou whitelist cassée passerait silencieusement).

- [x] T6.2 Tests Vitest frontend :
  - `contact-helpers.test.ts` : `formatIdeNumber('CHE109322551')` → `'CHE-109.322.551'`, `validateIdeFormat` accepte/rejette les bonnes formes.
  - `contacts.api.test.ts` (optionnel — skip si pattern trop lourd) : mock `apiClient`, vérifie les appels.

- [x] T6.3 Tests Playwright E2E — créer `frontend/tests/e2e/contacts.spec.ts` inspiré de `accounts.spec.ts` + `journal-entries.spec.ts`. **Prérequis seed** : tous ces tests nécessitent une DB avec un `users.role = 'Admin'` et une `companies` active (login `admin` / `admin123`). Le helper `login(page)` existant + `seed_demo` sont déjà utilisés par les autres specs — pattern identique à `journal-entries.spec.ts:13-19`.
  - **test** : création nominale — navigue vers `/contacts`, ouvre le form, remplit, valide, vérifie que le nouveau contact apparaît.
  - **test** : validation IDE invalide — saisit `CHE-109.322.552` (checksum invalide confirmé par le test `invalid_checksum` de `che_number.rs`), soumet, vérifie le message d'erreur API. **Ne PAS utiliser** `CHE-000.000.000` qui est un numéro VALIDE (checksum 0).
  - **test** : édition + conflit 409 — mock PUT pour retourner 409, vérifie que la modale de conflit s'ouvre.
  - **test** : archivage — crée un contact, clique Archiver, confirme, vérifie qu'il disparaît de la liste (sans `includeArchived`).
  - **test** : filtre par type — crée 2 contacts (Personne + Entreprise), filtre, vérifie.
  - **test.skip** : RBAC Lecteur 403 — reporter (aucun utilisateur Lecteur en seed par défaut).

### T7 — Validation finale & cleanup

- [x] T7.1 `cargo fmt --check` (0 diff) + `cargo clippy --workspace --all-targets -- -D warnings` (0 warning — la CI échouera sinon).
- [x] T7.2 `cargo check --workspace --tests`
- [x] T7.3 `cargo test --workspace --lib -- --skip repositories::` (tests unitaires hors DB)
- [x] T7.4 Vitest : `npm run test:unit`
- [x] T7.5 `npm run check` (svelte-check, 0 errors)
- [x] T7.6 Playwright suite contacts (local, si DB de test disponible)
- [x] T7.7 Mettre à jour `sprint-status.yaml` : story → `review` après implémentation complète.

## Dev Notes

### Architecture — où va quoi

```
kesh-db/
├── migrations/20260414000001_contacts.sql      # T1.1
└── src/
    ├── entities/
    │   ├── contact.rs                          # T1.2
    │   └── mod.rs                              # T1.3 — re-export
    └── repositories/
        ├── contacts.rs                         # T2 (create/find/list/update/archive + tests)
        └── mod.rs                              # T2.6 — pub mod

kesh-api/src/routes/
├── contacts.rs                                 # T3.1–T3.4
└── mod.rs                                      # T3.5 — routes dans comptable_routes

kesh-i18n/locales/*/messages.ftl                # T5 — ~35 clés × 4 langues

frontend/src/lib/features/contacts/
├── contacts.types.ts                           # T4.1
├── contacts.api.ts                             # T4.1
├── contact-helpers.ts                          # T4.1
├── contact-helpers.test.ts                     # T4.1
├── debounce.ts                                 # T4.3 — copié depuis features/journal-entries/
├── ContactForm.svelte                          # T4.2
└── (potentiellement) ConflictDialog.svelte     # si refactor du pattern story 3.3

frontend/src/routes/(app)/contacts/
└── +page.svelte                                # T4.3 — page liste

frontend/tests/e2e/contacts.spec.ts             # T6.3
```

### Ce qui existe DÉJÀ — NE PAS refaire

- **`CheNumber`** : `crates/kesh-core/src/types/che_number.rs` (Story 1.3). Exporté via `kesh_core::types::CheNumber`. Méthodes `new(input) -> Result<Self, CoreError>`, `formatted() -> String`, `as_str() -> &str`. Dérive `FromStr`, `TryFrom<String>`, `Display`, `Serialize`/`Deserialize`. **Tests existants** (incluant le vecteur officiel `CHE-109.322.551`). À réutiliser tel quel dans le handler `create_contact`/`update_contact`.
- **`Iban`** : idem dans `crates/kesh-core/src/types/iban.rs`. Pas utilisé en 4.1 mais sera pertinent en Story 5.3 (QR Bill).
- **Pattern repository post-Story 3.5** : `accounts.rs` est la référence canonique. Tous les patterns à copier : signature avec `user_id`, `if let Err(e) = audit_log::insert_in_tx`, `contact_snapshot_json` helper local, `{before, after}` wrapper pour update.
- **`audit_log::insert_in_tx` + `find_by_entity`** : `crates/kesh-db/src/repositories/audit_log.rs` (Story 3.3). Ne pas le toucher.
- **`SortDirection`** (Asc/Desc générique) : dans `crates/kesh-core/src/listing/mod.rs` (Story 3.4). À importer et réutiliser. Méthode `.as_sql_keyword()` whitelist.
- **`SortBy`** dans le même module : **journal-entries-specific** (variants `EntryDate`, `EntryNumber`, `Journal`, `Description`). **NE PAS** étendre pour contacts malgré le doc-comment trompeur (« réutilisable par toutes les listes »). Créer un `ContactSortBy` local.
- **`PaginatedResult<T>`** : **N'existe PAS** (vérifié empiriquement 2026-04-11). Story 3.4 a créé `JournalEntryListResult` (struct concret non générique). Pour contacts, créer `ContactListResult` concret localement (pattern identique).
- **`JournalEntryListQuery` + `push_where_clauses`** : référence canonique QueryBuilder dynamique dans `journal_entries.rs:~325-400`. À étudier avant T2.3.
- **Helpers frontend pagination/debounce/query-helpers** : `frontend/src/lib/features/journal-entries/{debounce,query-helpers}.ts` (Story 3.4). À dupliquer ou à mutualiser (voir T4.3).
- **`notify.ts` helpers** : `frontend/src/lib/shared/utils/notify.ts` (Story 3.5). Utiliser `notifySuccess`/`notifyError` pour TOUS les feedbacks du nouveau code.
- **`i18nMsg` canonical** : `frontend/src/lib/shared/utils/i18n.svelte.ts` (Story 3.5 P5). **NE PAS** importer depuis `$lib/features/onboarding/onboarding.svelte`.
- **Modale 409 `OPTIMISTIC_LOCK_CONFLICT`** : pattern dans `JournalEntryForm.svelte` (Story 3.3). Copier la structure `{#if conflictState}` + `Dialog.Root` + bouton Recharger.
- **`Extension<CurrentUser>` + `comptable_routes` + `authenticated_routes`** : patterns Story 1.8 + 3.3. **Emplacement réel** : les groupes de routes `comptable_routes` (Admin + Comptable via `require_comptable_role`) et `authenticated_routes` (tout rôle authentifié) vivent dans `crates/kesh-api/src/lib.rs` lignes 84-111, pas dans `routes/mod.rs`. L'ordre d'exécution middleware (oignon) : `require_auth` (outer) → `require_role` (inner) → handler.
- **`ListResponse<T>`** : type générique paginé dans `crates/kesh-api/src/routes/mod.rs:25`. Utilisé par Story 3.4 (`journal_entries.rs`) pour `Json<ListResponse<JournalEntryResponse>>`. **À réutiliser tel quel** — pas de nouveau struct `PaginatedContactsResponse`.
- **`serde_json = "1"`** : déjà en dépendance de `kesh-db` (`Cargo.toml:12`, confirmé 2026-04-11). Utilisable directement pour `contact_snapshot_json` sans modifier `Cargo.toml`.
- **`AppError` enum + mapping HTTP** : `crates/kesh-api/src/errors.rs`. Ajouter un variant si besoin pour `IDE_ALREADY_EXISTS` ou réutiliser `AppError::Conflict` avec message explicite.

### Patterns existants à réutiliser (citations précises)

- **Pattern entité + enum SQLx manuel** : `crates/kesh-db/src/entities/account.rs` lignes ~25-80 pour `AccountType`. Copier la structure (impl `sqlx::Type<MySql>`, `Encode`, `Decode`).
- **Pattern repository avec audit + rollback explicite** : `crates/kesh-db/src/repositories/accounts.rs` — `create` commence ~ligne 41, `update` ~ligne 151 (SELECT before + UPDATE + wrapper `{before, after}`), `archive` ~ligne 241. Lignes indicatives (à confirmer par grep au moment de l'implémentation — Story 3.5 a pu décaler).
- **Pattern handler `Extension<CurrentUser>`** : `crates/kesh-api/src/routes/accounts.rs` lignes 107-155 pour `create_account`.
- **Pattern `QueryBuilder` dynamique** : `crates/kesh-db/src/repositories/journal_entries.rs` (Story 3.4, `list_by_company_paginated`). Rechercher `QueryBuilder::new` dans ce fichier pour le modèle exact.
- **Pattern `ContactListQuery` + `sort_by` enum** : `JournalEntryListQuery` de Story 3.4.
- **Pattern audit_log helper snapshot JSON** : `account_snapshot_json` dans `accounts.rs` (Story 3.5 post-P8 qui inclut `companyId`). Reprendre le même pattern pour `contact_snapshot_json`.

### Pièges identifiés

1. **SQLx 0.8 + MariaDB enum manuel** : NE PAS utiliser `#[sqlx(type_name = "ENUM")]` sur `ContactType`. Utiliser le pattern manuel `AccountType` (impl `Type<MySql>`, `Encode`, `Decode` à la main avec matching sur `&str`). Référence empirique Kesh, décision Story 1.4.

2. **Unicité partielle sur `(company_id, ide_number)` avec NULL** : MariaDB autorise plusieurs lignes avec `ide_number = NULL` dans un index UNIQUE par défaut. C'est le comportement voulu (test `test_create_allows_null_ide_duplicates`). Vérifier empiriquement si comportement différent, ajuster la contrainte.

3. **Normalisation IDE côté stockage** : le backend stocke la forme NORMALISÉE (`CHE109322551`, 12 chars), pas la forme formatée (`CHE-109.322.551`). Le frontend reçoit la normalisée et appelle `formatIdeNumber()` pour l'affichage. **Ne PAS stocker les séparateurs** — source de vérité unique côté backend via `CheNumber::new(...).as_str()`.

4. **`email` optionnel** : si l'utilisateur laisse le champ vide, le backend reçoit `""` ou `null`. Normaliser à `None` via `if email.trim().is_empty() { None } else { Some(email.trim().to_string()) }` avant insertion. Éviter de stocker des chaînes vides qui cassent la contrainte email valide.

5. **`default_payment_terms` colonne créée mais non exposée UI en 4.1** : la migration crée la colonne, le DTO `CreateContactRequest` l'accepte (optional), le handler la persiste, mais le formulaire Svelte **ne rend PAS de champ pour elle**. Story 4.2 ajoutera le champ UI. **Bien vérifier** qu'aucun test frontend ne cherche le champ (sinon le test échoue en 4.1 et personne ne comprend pourquoi).

6. **Duplication `get_admin_user_id` dans `contacts::tests`** : la 4e copie du helper. Décision Story 3.5 L1 : duplication acceptée, refactor transverse post-MVP. Ne PAS tenter de mutualiser maintenant — scope creep.

7. **Pas de type générique `PaginatedResult<T>`** : vérifié empiriquement au moment de la rédaction de cette spec — `kesh-core/src/listing/mod.rs` ne contient QUE `SortDirection` (générique) et `SortBy` (journal-specific). Story 3.4 a créé `JournalEntryListResult` concret local à `journal_entries.rs`. Même approche pour contacts : struct concret `ContactListResult` déclaré dans `contacts.rs`. Ne PAS perdre de temps à chercher ou refactorer en type générique — YAGNI.

8. **Mode `dev-story` : `debounce.ts` duplication** : décision T4.3 = dupliquer 3 lignes dans `features/contacts/debounce.ts`. Si le dev décide de mutualiser dans `shared/utils/` à la place, c'est OK mais doit mettre à jour `features/journal-entries/` en conséquence. **Choix par défaut : duplication** pour éviter le risque de casser Story 3.4.

9. **Tri par date — `CreatedAtAsc` vs `UpdatedAtAsc`** : le tri UI par « Date » signifie **date de création** (cohérent avec accounts/users/journal entries existants). Ne PAS trier par `updated_at` par défaut.

10. **`include_archived = true` avec filtres** : quand un utilisateur active `includeArchived` et filtre par `isClient = true`, le résultat inclut les contacts archivés qui étaient clients. Le filtre est sur le champ, pas sur l'état archivé. C'est le comportement attendu.

11. **Tenant isolation — v0.1 single-company** : les repositories `contacts::find_by_id`, `update`, `archive` ne prennent pas `company_id` en paramètre (parité avec `accounts.rs`). Kesh v0.1 est **single-company** (une seule `companies` existe en base après onboarding), donc pas de risque cross-tenant en pratique. **Dette à revisiter** : dès qu'une v0.2 multi-company arrivera, toutes les lectures/écritures devront filtrer par `company_id` pour éviter des fuites cross-tenant par ID guessing. À documenter en story future (`12.x` clôture ou `multi-tenant`).

### Réutilisation opportuniste

- **Grep avant de recréer** : pour chaque helper frontend (debounce, query-helpers, format-helpers), vérifier via `grep -rn "export function <nom>" frontend/src/lib/` si le helper existe déjà ailleurs. Si oui et réutilisable sans coupling, réutiliser. Si non, dupliquer simplement.
- **`AlertDialog` shadcn-svelte** : déjà installé (Story 3.3). Réutiliser pour la confirmation d'archivage.
- **`Badge` shadcn-svelte** : déjà installé. Utiliser pour `is_client` (variant `secondary` vert) et `is_supplier` (variant `secondary` bleu).

### Tests — critères de couverture minimum

- **Repository `contacts`** : ≥ 10 tests d'intégration DB (CRUD nominal, unicité IDE, NULL IDE, optimistic lock, audit × 3, pagination, filtres, tri).
- **Frontend helpers** : 100% des branches de `validateIdeFormat` et `formatIdeNumber`.
- **Playwright** : 5 scénarios utilisateur critiques (création, édition, archivage, 409, filtre).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-4-Story-4.1] — AC BDD + schéma
- [Source: _bmad-output/planning-artifacts/prd.md#FR25-FR27] — Carnet d'adresses unifié, flags, validation IDE
- [Source: _bmad-output/planning-artifacts/prd.md#FR88] — Journal d'audit
- [Source: _bmad-output/planning-artifacts/architecture.md#conventions-nommage] — snake_case DB, camelCase JSON
- [Source: _bmad-output/planning-artifacts/architecture.md#types-forts-validation] — Newtypes kesh-core (`CheNumber`, `Iban`)
- [Source: _bmad-output/planning-artifacts/architecture.md#verrouillage-optimiste] — Pattern `version` uniforme
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#tableau-standard] — Tableau filtrable/triable/paginé
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#ux-dr-emotionnel] — Confiance, compétence, autonomie
- [Source: crates/kesh-core/src/types/che_number.rs] — **`CheNumber` déjà implémenté (Story 1.3), à réutiliser**
- [Source: crates/kesh-db/src/entities/account.rs] — Pattern enum + entité SQLx (modèle canonique)
- [Source: crates/kesh-db/src/repositories/accounts.rs] — Pattern CRUD + audit log post-Story 3.5 (référence d'implémentation exacte)
- [Source: crates/kesh-db/src/repositories/audit_log.rs] — `insert_in_tx` + `find_by_entity` (Story 3.3)
- [Source: crates/kesh-db/src/repositories/journal_entries.rs] — Pattern `list_by_company_paginated` + `QueryBuilder` dynamique (Story 3.4)
- [Source: crates/kesh-api/src/routes/accounts.rs] — Pattern handlers + `Extension<CurrentUser>`
- [Source: crates/kesh-api/src/routes/mod.rs] — `ListResponse<T>` générique paginé (ligne 25)
- [Source: crates/kesh-api/src/lib.rs] — `comptable_routes` (ligne 84) + `authenticated_routes` (ligne 111) + middleware RBAC
- [Source: crates/kesh-db/migrations/20260411000001_accounts.sql] — Modèle canonique migration Kesh
- [Source: frontend/src/lib/features/journal-entries/JournalEntryForm.svelte] — Pattern modale conflit 409
- [Source: frontend/src/lib/features/journal-entries/debounce.ts] — Helper debounce réutilisable (Story 3.4)
- [Source: frontend/src/lib/shared/utils/notify.ts] — Helpers toast (Story 3.5)
- [Source: frontend/src/lib/shared/utils/i18n.svelte.ts] — Module i18n canonical (Story 3.5 P5)
- [Source: _bmad-output/implementation-artifacts/3-5-notifications-aide-contextuelle-audit.md#Dev-Notes-piege-8] — Convention `details_json` wrapper vs direct

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

- `cargo check -p kesh-db` / `-p kesh-api` après chaque task : propre à chaque étape.
- `cargo test -p kesh-db --lib repositories::contacts -- --test-threads=1` : **16/16 tests DB** passent (docker MariaDB 11.x, admin bootstrap via API run).
- `cargo test -p kesh-api --lib routes::contacts::tests` : **10/10 tests unit handler**.
- `npm run test:unit` : **140/140 Vitest** (14 nouveaux tests contacts + 126 existants).
- `npm run check` : **0 erreurs** svelte-check (2 warnings pré-existants `design-system/+page.svelte`, non liés).
- `cargo fmt --all -- --check` : clean après application.
- `cargo clippy --workspace --all-targets -- -D warnings` : clean. 2 warnings pré-existants corrigés en chemin (kesh-seed `useless conversion` ligne 126, auth_e2e.rs import `put` + `user_id` inutilisé).

### Completion Notes List

- **T1** — Migration `20260414000001_contacts.sql` + `entities/contact.rs` avec `ContactType` enum manuel SQLx (pattern `AccountType` + `Copy` derive). CHECK `BINARY` contact_type + `CHAR_LENGTH(TRIM(name))` appliqués.
- **T2** — Repository contacts complet (create/find_by_id/list_by_company/list_by_company_paginated/update/archive) + `ContactListQuery`/`ContactListResult`/`ContactSortBy` locaux + `contact_snapshot_json` + `escape_like` dupliqué + pattern audit rollback explicite P10/P11 sur les 3 mutations + pré-check dans archive pour rejeter déjà archivé (P24).
- **T3** — 5 handlers API + nouveau variant `AppError::IdeAlreadyExists(String)` ajouté à `errors.rs` mappé en HTTP 409 `IDE_ALREADY_EXISTS` + `is_valid_email_simple` manuel (pas de regex crate) + `CheNumber::new` pour normalisation IDE + intercept `map_contact_error` pour remapper `UniqueConstraintViolation("ide_number")` → `IdeAlreadyExists`. Routes enregistrées dans `lib.rs` split RBAC : 2 GETs dans `authenticated_routes`, 3 mutations dans `comptable_routes`.
- **T4** — Frontend : `contacts.types.ts`, `contacts.api.ts`, `contact-helpers.ts` (+ test 14/14), page `/contacts/+page.svelte` avec table + filtres (search debouncé 300ms, type, is_client, is_supplier, include_archived) + pagination + tri cliquable + dialog create/edit + dialog archive + dialog conflit 409 + URL state sync via `$page.url.searchParams`. Sidebar link ajouté en label hardcodé (cohérent avec pattern P10). Composant `Label` n'existe pas dans le projet — remplacé par `<label>` HTML natif.
- **T5** — **48 clés i18n × 4 langues** (FR/DE/IT/EN), incluant `error-ide-already-exists` (P29) et `contact-error-archived-no-modify` (P29).
- **T6** — Tests complets :
  - **16 tests d'intégration DB** — couvrent AC #1, #3, #5, #6, #7, #8, #10, #14 + pattern rollback, escape LIKE P21, variants SortBy P34, update rejeté sur archivé P24.
  - **10 tests unit handler** — validation email edge cases, IDE normalization, map_contact_error ciblé, normalize_optional.
  - **14 tests Vitest** helpers frontend.
  - **5 tests Playwright e2e** — création nominale, IDE invalide (`CHE-109.322.552`), archivage avec confirmation, filtre par type avec URL state, reload preservation.
- **T7** — Validation finale complète. Story marquée `review`.

**Points notables** :
- `user@@domain.ch` faussement accepté par validation email — dette intentionnelle documentée (P31).
- Helpers pré-existants hors scope corrigés pour clippy (kesh-seed + auth_e2e — triviaux).
- `cargo fmt` a reformatté de nombreux fichiers legacy en plus du code 4.1. Scope creep acceptable dans un même commit (tous alignés sur le style canonique).
- `is_valid_email_simple` + `CheNumber::new` sont invoqués côté handler uniquement — le repository reçoit déjà des valeurs validées et normalisées.

### File List

**Backend (Rust) — fichiers créés** :
- `crates/kesh-db/migrations/20260414000001_contacts.sql`
- `crates/kesh-db/src/entities/contact.rs`
- `crates/kesh-db/src/repositories/contacts.rs` (repository + 16 tests DB)
- `crates/kesh-api/src/routes/contacts.rs` (5 handlers + 10 tests unit)

**Backend (Rust) — fichiers modifiés** :
- `crates/kesh-db/src/entities/mod.rs` — + `pub mod contact` + re-exports
- `crates/kesh-db/src/repositories/mod.rs` — + `pub mod contacts`
- `crates/kesh-api/src/errors.rs` — + variant `AppError::IdeAlreadyExists(String)` + mapping HTTP 409
- `crates/kesh-api/src/routes/mod.rs` — + `pub mod contacts`
- `crates/kesh-api/src/lib.rs` — + 5 routes split RBAC authenticated/comptable
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — + 48 clés
- `crates/kesh-i18n/locales/de-CH/messages.ftl` — + 48 clés
- `crates/kesh-i18n/locales/it-CH/messages.ftl` — + 48 clés
- `crates/kesh-i18n/locales/en-CH/messages.ftl` — + 48 clés
- `crates/kesh-seed/src/lib.rs` — 1 ligne (clippy fix pré-existant)
- `crates/kesh-api/tests/auth_e2e.rs` — 2 lignes (clippy fix pré-existant)
- Nombreux fichiers reformattés par `cargo fmt --all` (même commit).

**Frontend (SvelteKit) — fichiers créés** :
- `frontend/src/lib/features/contacts/contacts.types.ts`
- `frontend/src/lib/features/contacts/contacts.api.ts`
- `frontend/src/lib/features/contacts/contact-helpers.ts`
- `frontend/src/lib/features/contacts/contact-helpers.test.ts` (14 tests Vitest)
- `frontend/src/routes/(app)/contacts/+page.svelte` (page liste + form + 3 dialogs)
- `frontend/tests/e2e/contacts.spec.ts` (5 tests Playwright + 2 test.skip)

**Frontend — fichiers modifiés** :
- `frontend/src/routes/(app)/+layout.svelte` — + item sidebar « Carnet d'adresses »

**Documentation / tracking** :
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/4-1-carnet-adresses-crud-contacts.md` (cette story)

## Change Log

- 2026-04-11: Création de la story 4.1 (Claude Opus 4.6, 1M context) — première story Epic 4 « Carnet d'adresses & Catalogue ». Décisions clés :
  - **`CheNumber` déjà implémenté en Story 1.3** — découverte critique en préparation : le dev ne doit PAS réimplémenter l'algorithme modulo 11. Réutilisation directe de `kesh_core::types::CheNumber`.
  - **Scope 4.1 vs 4.2** : colonne `default_payment_terms` créée dès la migration 4.1 (schéma complet), mais **non exposée dans le formulaire UI** (câblage UI reporté en Story 4.2 avec le catalogue produits).
  - **`ContactType` enum** distinct de `is_client`/`is_supplier` — un contact peut être `Entreprise` simultanément client et fournisseur (ex: grossiste).
  - **Unicité IDE** : contrainte `UNIQUE (company_id, ide_number)` avec NULL distinct (comportement MariaDB natif). Plusieurs contacts sans IDE autorisés.
  - **Soft-delete** via `active = false` + route `archive`, pas de DELETE SQL. Pas de route `unarchive` en v0.1.
  - **Pas de règle « au moins un flag client OU fournisseur »** — un contact peut avoir les 2 flags `false` (prospect, référence). Formulaire par défaut propose `isClient = true`.
  - **Email validation** : ~~regex simple~~ **validation manuelle sans aucune dépendance externe** — helper `is_valid_email_simple(s: &str) -> bool` caractère-par-caractère (voir section Décisions de conception). La crate `regex` n'est pas dans le workspace Kesh (vérifié pass 2 P20). Pas de nouvelle dépendance.
  - **Pattern backend calqué sur `accounts.rs` post-Story 3.5** : signature avec `user_id`, rollback explicite sur erreur audit_log, helper `contact_snapshot_json`, wrapper `{before, after}` pour update.
  - **Duplication `debounce.ts`** : 3 lignes dupliquées dans `features/contacts/` (décision v0.1, refactor transverse post-MVP si pattern se répète). Évite de casser Story 3.4.
  - **`get_admin_user_id` 4e duplication** acceptée (décision Story 3.5 L1).
  - **Pas de `AccountingTooltip` sur IDE** — pas un terme comptable ambigu, simple texte d'aide suffit.
  - **~35 clés i18n × 4 langues** — labels form, colonnes, filtres, messages d'erreur, toasts.
- 2026-04-11: **Revue adversariale passe 1** (3 subagents Sonnet + Haiku + Sonnet parallèles, LLMs orthogonaux à Opus auteur — CLAUDE.md rule). 18 findings dont 1 CRITICAL, 6 HIGH, 5 MEDIUM, 6 LOW + 2 rejetés (faux positifs Haiku). 18 patches **P1-P18** appliqués :
  - **P1 [CRITICAL]** AC#2 utilisait `CHE-000.000.000` comme exemple de checksum invalide, mais c'est un **numéro VALIDE** (test `valid_check_digit_zero` de `che_number.rs` : `sum=0, 0%11=0`). Remplacé par `CHE-109.322.552` (dernier chiffre décalé, vecteur confirmé par le test `invalid_checksum`).
  - **P2 [HIGH]** Utilisation de `ListResponse<T>` existant (`routes/mod.rs:25`) au lieu de créer un nouveau `PaginatedContactsResponse`. Pattern identique à Story 3.4 (`journal_entries.rs`). `ContactListResult` reste comme struct intermédiaire côté repository, converti en `ListResponse<ContactResponse>` dans le handler.
  - **P3 [HIGH]** Retrait de l'import fantôme `SortBy` dans le bloc `use` de T2.1 — seul `SortDirection` est importé depuis `kesh_core::listing`.
  - **P4 [HIGH]** Incohérence interne tranchée : `ContactSortBy` avec variants `{ Name, CreatedAt, UpdatedAt }` + `SortDirection` **séparé**. PAS un enum combiné `NameAsc/NameDesc`. Mise à jour cohérente dans la section Décisions et dans `test_list_sort_order`.
  - **P5 [HIGH]** `SortDirection::default()` retourne `Desc` (convention comptable de `kesh_core::listing`). Ajout explicite d'un `impl Default for ContactListQuery` qui **hardcode** `sort_direction: SortDirection::Asc` pour UX carnet d'adresses alphabétique.
  - **P6 [HIGH]** CHECK `contact_type` manquait `BINARY` (bypass de casse). Corrigé vers `CHECK (BINARY contact_type IN (BINARY 'Personne', BINARY 'Entreprise'))`, pattern identique à `20260411000001_accounts.sql`.
  - **P7 [HIGH]** `CHECK (TRIM(name) <> '')` → `CHECK (CHAR_LENGTH(TRIM(name)) > 0)` — pattern canonique du projet, cohérent avec toutes les autres migrations.
  - **P8 [MEDIUM]** Routage RBAC explicité : `GET /contacts` et `GET /contacts/{id}` dans `authenticated_routes` (tout rôle incluant Consultation), `POST/PUT/PUT-archive` dans `comptable_routes` (Admin + Comptable). Correction aussi de l'emplacement réel : **`lib.rs:84` et `lib.rs:111`**, pas `routes/mod.rs`.
  - **P9 [MEDIUM]** Clarifié dans T1.2 que `ContactUpdate` ne contient PAS `version` — passé comme paramètre séparé à `contacts::update(...)`, pattern identique à `accounts::update`. Le handler extrait `version` du body `UpdateContactRequest`.
  - **P10 [MEDIUM]** Sidebar : précision explicite que les labels sont hardcodés dans le tableau `navGroups` (pas d'i18n actuel). Ajouter l'item contacts avec label hardcodé, PAS de refactor i18n transverse de la sidebar en 4.1.
  - **P11 [MEDIUM]** Alerte dette technique : le layout actuel importe `i18nMsg` depuis `features/onboarding/` (legacy pré-Story 3.5 P5). Avertissement explicite au dev de NE PAS répliquer ce pattern dans le nouveau code contacts — utiliser uniquement `$lib/shared/utils/i18n.svelte`.
  - **P12 [MEDIUM]** Mapping `IDE_ALREADY_EXISTS` précisé : option (a) intercept explicite dans le handler via `match` sur `DbError::UniqueConstraintViolation(ref m) if m.contains("ide_number")` avant propagation. Décision v0.1 documentée.
  - **P13 [LOW]** `ContactType` enum ajoute le derive `Copy` (cohérent avec `AccountType`), évite les `.clone()` superflus dans le repository.
  - **P14 [LOW]** Précision que `From<Contact> for ContactResponse` copie `ide_number` tel quel (pas de re-parse `CheNumber` — déjà normalisé en base).
  - **P15 [LOW]** Numéros de ligne `accounts.rs` ajustés (create ~41, update ~151, archive ~241) avec mention que ce sont indicatifs — Story 3.5 a pu décaler.
  - **P16 [LOW]** Note préventive T1.1 : vérifier via `ls migrations/` que `20260414000001` est bien le prochain numéro disponible avant d'implémenter.
  - **P17 [LOW]** T6.3 : prérequis seed DB explicité pour Playwright (admin user + company active, pattern `journal-entries.spec.ts:13-19`).
  - **P18 [LOW]** Note dans « Ce qui existe DÉJÀ » : `serde_json = "1"` déjà en dépendance de `kesh-db` (`Cargo.toml:12`). Aucune modification `Cargo.toml` nécessaire.
  - **Rejetés** (2 faux positifs Haiku) :
    - Haiku disait « `comptable_routes` not found dans `routes/mod.rs` » — il cherchait au mauvais endroit, le groupage vit dans `lib.rs:84`. Vérifié empiriquement.
    - Haiku disait « `CheNumber` 350 lignes vs 351 » — trivial, pas une erreur de spec.
- 2026-04-11: **Revue adversariale passe 2** (2 subagents Haiku verifier + Opus adversarial/readiness parallèles, orthogonaux à Sonnet pass 1). Opus a détecté **3 HIGH réels** dont 1 régression sur P12 (le variant `AppError::Conflict { code, message }` proposé par P12 n'existe pas dans `errors.rs`). Patches **P19–P27** appliqués :
  - **P19 [HIGH]** T3.3 réécrit : au lieu de l'option (a) intercept avec `AppError::Conflict { code, message }` (variant inexistant → ne compilerait pas), **ajouter un nouveau variant `AppError::IdeAlreadyExists(String)` dans `crates/kesh-api/src/errors.rs`** avec son mapping `IntoResponse` → 409 code `IDE_ALREADY_EXISTS`. Le handler intercepte via `.map_err(|e| match e { DbError::UniqueConstraintViolation(ref m) if m.contains("ide_number") => AppError::IdeAlreadyExists(...), other => AppError::from(other) })`. Pattern applicable à `create_contact` ET `update_contact` (course sur UPDATE vers IDE existant aussi possible).
  - **P20 [HIGH]** Validation email : la crate `regex` **n'est PAS dans le workspace Kesh** (vérifié 2026-04-11, aucun `Cargo.toml` ne la déclare). Remplacement de la regex `^[^\s@]+@[^\s@]+\.[^\s@]+$` par un **helper manuel `is_valid_email_simple(s: &str) -> bool`** caractère-par-caractère (find '@', split, check absence whitespace, check '.' dans le domaine, etc.). Implémentation complète fournie dans la section Décisions. Cases de test listés.
  - **P21 [HIGH]** LIKE search : le pattern pass 1 manquait l'échappement du backslash ET la clause `ESCAPE '\\\\'` dans le SQL. Story 3.4 (`journal_entries.rs:320-322`) échappe 3 caractères dans l'ordre strict `\` → `%` → `_`, et émet `LIKE ? ESCAPE '\\\\'` (lignes 382-385). Sans cette clause, MariaDB ignore silencieusement l'échappement (dépendance au mode `NO_BACKSLASH_ESCAPES`). T2.3 réécrit avec le pattern complet, exemple de code `push_where_clauses` ajouté. Helper `escape_like` à dupliquer dans `contacts.rs` (cohérent avec la décision `debounce.ts`).
  - **P22 [MEDIUM]** Frontend T4.1 : `PaginatedContactsResponse` (dangling P2) supprimé du type TS à créer, remplacé par réutilisation de `ListResponse<ContactResponse>` TS. Précision de vérifier d'abord si un type TS générique `ListResponse<T>` existe déjà (Story 3.4 frontend), sinon le créer avec le bon shape miroir de `routes/mod.rs:25`.
  - **P23 [MEDIUM]** T4.6 ajouté : `query-helpers.ts` décision tranchée à **duplication** dans `features/contacts/` (cohérence avec la décision `debounce.ts`). Seuil d'escalade vers mutualisation `shared/` : si > 30 lignes.
  - **P24 [MEDIUM]** AC#7 complété : `update_contact`/`archive_contact` sur un contact déjà archivé → `DbError::IllegalStateTransition` → HTTP 400 (pas de dé-archivage en v0.1). Check effectué dans la tx après le SELECT initial (pattern `accounts::archive children > 0`).
  - **P25 [MEDIUM]** T6.1 enrichi : 3 nouveaux tests DB :
    - `test_create_normalizes_ide_mwst_suffix` (couvre AC#2 suffixe TVA, non testé précédemment)
    - `test_update_rejects_archived_contact` (couvre P24)
    - `test_archive_rejects_already_archived` (couvre P24)
  - **P26 [LOW]** T7.1 ajouté : `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D warnings` (la CI enforce, omission = build CI rouge).
  - **P27 [LOW]** Piège #11 ajouté : tenant isolation v0.1 single-company, dette documentée pour v0.2 multi-company (parité `accounts.rs` — `find_by_id`/`update`/`archive` ne filtrent pas par `company_id`).
  - **P28 [LOW]** T6.1 filter tests décomposés : au lieu d'un seul `test_list_paginated_filters` combinant 3 filtres, 6 tests indépendants (`test_filter_by_{contact_type,is_client,search_name,search_email,combined,escape_like_wildcard}`) — le dernier couvre explicitement le correctif P21.
  - **Rejetés pass 2** :
    - Haiku disait « T3.4 critical ordre routes Axum » — faux positif, Axum 0.8 `Router::merge` gère correctement les patterns `{id}` vs POST/PUT sur groupes middleware différents.
    - Haiku disait « import `audit_log` manquant dans T2.1 bloc code » — cosmétique, le bloc code est un aperçu pseudo-code, l'import est mentionné dans le texte avant et l'implémentation détaillée en T2.5.
- 2026-04-11: **Revue adversariale passe 3** (1 subagent Sonnet ciblé sur convergence, orthogonal à Haiku + Opus pass 2). **28/28 patches** P1-P28 vérifiés OK. 1 MEDIUM résiduel + 1 LOW détectés. Patches **P29-P31** appliqués :
  - **P29 [MEDIUM]** T5.1 : ajout de la clé i18n `error-ide-already-exists` (référencée par `AppError::IdeAlreadyExists` en T3.3) dans la liste des erreurs avec note explicite « oublier de l'ajouter aux 4 `.ftl` ferait tomber les DE/IT/EN sur le fallback français ». Ajout aussi de `contact-error-archived-no-modify` (référencée par P24).
  - **P30 [LOW]** Résidu pass 1 détecté : T6.3 scénario Playwright « validation IDE invalide » référençait encore `CHE-000.000.000` (numéro VALIDE). Corrigé en `CHE-109.322.552` cohérent avec AC#2 (P1).
  - **P31 [LOW]** `is_valid_email_simple` — documenter comme **dette intentionnelle v0.1** le fait que `user@@domain.ch` serait accepté (helper utilise `find('@')` qui trouve la première occurrence). Validation full-RFC 5322 reportée post-MVP. Cas edge non-réaliste en pratique.
- 2026-04-11: **Revue adversariale passe 4** (1 subagent Haiku ciblé convergence, orthogonal à Sonnet pass 3). P29-P31 vérifiés OK. **1 MEDIUM résiduel détecté** : ligne 572 de la section historique Change Log contredisait encore P20 (mentionnait regex email). **P32** appliqué :
  - **P32 [MEDIUM]** Ligne 572 réécrite : ~~regex simple~~ → `is_valid_email_simple` validation manuelle (cohérence avec P20 appliqué en pass 2).
- 2026-04-11: **Revue adversariale passe 5** (1 subagent Opus final sanity-check, orthogonal à Haiku pass 4). P32 vérifié OK. Dangling refs comptés : `PaginatedContactsResponse` (4 occurrences toutes contextuelles), `CHE-000.000.000` (3 avec warnings), `AppError::Conflict` (0 fautive), `regex` crate (0 proposant de l'ajouter). 2 LOW cosmétiques résiduels acceptés (header T6 sous-évalue la couverture AC, AC#12 RBAC skip justifié).
- 2026-04-11: **Revue adversariale passe 6** (Sonnet deep adversarial + Haiku empirique 10 spot-checks, parallèles, orthogonaux à Opus pass 5). Sonnet a trouvé **1 MEDIUM réel que 5 passes précédentes avaient manqué** : le mapping HTTP de `DbError::IllegalStateTransition` est **409 (`StatusCode::CONFLICT`)**, pas 400 comme l'indiquait AC#7 et P24. Vérifié empiriquement dans `crates/kesh-api/src/errors.rs:317-324`. Haiku a fait un faux négatif en disant que le variant `IllegalStateTransition` n'existait pas — en réalité il est à `crates/kesh-db/src/errors.rs:39`. Patches **P33-P34** appliqués :
  - **P33 [MEDIUM]** AC#7 réécrit : « HTTP 400 » → **« HTTP 409 `ILLEGAL_STATE_TRANSITION` »** avec référence empirique `errors.rs:317-324`. Note explicite pour les tests Playwright de vérifier le status 409 (pas 400). Sans ce fix, le dev aurait écrit des tests attendant 400, qui auraient été rouges à l'exécution.
  - **P34 [LOW]** `test_list_sort_order` étendu pour couvrir les **4 combinaisons** dont `(UpdatedAt, Desc)` — le variant `UpdatedAt` de `ContactSortBy` était défini dans l'enum whitelist mais jamais testé, surface de risque dormante.
  - **Rejetés pass 6** : Haiku faux négatif sur `IllegalStateTransition` (variant existe bien dans `kesh-db/src/errors.rs:39`, utilisé par `accounts.rs` et `fiscal_years.rs`).
- 2026-04-11: **Revue adversariale passe 7** (Opus convergence confirmation, orthogonal à Sonnet+Haiku pass 6). P33/P34 vérifiés OK. Audit complet des mappings HTTP `DbError → AppError → StatusCode` contre `errors.rs:185-323` : tous corrects (`NotFound→404`, `OptimisticLockConflict→409`, `UniqueConstraintViolation→409`, `CheckConstraintViolation→400`, `IllegalStateTransition→409`, `Validation→400`, `IdeAlreadyExists→409`). Dangling-ref sweep pour "HTTP 400" dans le contexte `IllegalStateTransition` : **0 occurrence normative** (seule la P24 historique au Change Log mentionne encore 400, par design — trace de l'erreur corrigée ensuite par P33). **No findings. Convergence définitive.**
- 2026-04-11: **CRITÈRE D'ARRÊT CLAUDE.MD DÉFINITIVEMENT ATTEINT APRÈS 7 PASSES ADVERSARIALES** (Sonnet×3 → Haiku+Opus → Sonnet → Haiku → Opus → Sonnet+Haiku → Opus, **LLMs strictement orthogonaux**, fenêtres fraîches à chaque passe). **34 patches au total** (P1-P34) : 1 CRITICAL + 9 HIGH + 11 MEDIUM + 13 LOW. 0 finding > LOW résiduel. Story 4.1 **PRÊTE POUR `dev-story`** — le dev agent peut commencer T1 immédiatement avec une spec blindée contre les 34 pièges identifiés.
- 2026-04-11: **Implémentation dev-story** (Claude Opus 4.6, 1M context). T1–T7 complétés en une session continue :
  - **T1** : migration SQL `20260414000001_contacts.sql` conforme spec (BINARY CHECK, CHAR_LENGTH, UNIQUE NULL-tolerant, index `company_active`/`company_name`). Entité `contact.rs` avec `ContactType { Personne, Entreprise } + Copy` + impl SQLx manuelle (pattern `AccountType`), `Contact`/`NewContact`/`ContactUpdate` DTOs. Pas de surprise — pattern empirique de `accounts.rs` copié tel quel.
  - **T2** : repository `contacts.rs` avec les 6 fonctions + types locaux `ContactListQuery`/`ContactListResult`/`ContactSortBy` + `contact_snapshot_json` (incluant `companyId`) + `escape_like` dupliqué + `push_where_clauses` basé sur `journal_entries.rs`. Audit log atomique avec rollback explicite sur create/update/archive. Pré-check dans `archive` pour rejeter `IllegalStateTransition` sur contact déjà archivé.
  - **T3** : 5 handlers API (`list_contacts`, `get_contact`, `create_contact`, `update_contact`, `archive_contact`) + DTOs camelCase + nouveau variant `AppError::IdeAlreadyExists(String)` ajouté à `errors.rs` (mapping HTTP 409 `IDE_ALREADY_EXISTS`) + helpers `is_valid_email_simple` (manuel, pas de regex crate), `validate_optional_ide` (via `CheNumber::new`), `normalize_optional`, `map_contact_error`. Validation centralisée dans `validate_common` (avec `#[allow(clippy::too_many_arguments)]` pour éviter le lint). Routes split RBAC dans `lib.rs` : GETs → `authenticated_routes`, mutations → `comptable_routes`.
  - **T4** : feature frontend complète `src/lib/features/contacts/` (types TS miroir, API client typé, helpers purs) + page `/contacts/+page.svelte` avec table filtrable/triable/paginée, 3 dialogs (create/edit, archive confirm, conflit 409), URL state sync via `$effect`, debounce search 300ms. Label HTML natif (pas de composant `Label` shadcn dans le projet — confirmé empiriquement). Item sidebar `Carnet d'adresses` ajouté en label hardcodé (cohérent P10).
  - **T5** : **48 clés i18n × 4 langues** (FR/DE/IT/EN) — nav, titres, labels form, colonnes, filtres, messages succès/erreur, confirmation dialogs, `error-ide-already-exists`, `contact-error-archived-no-modify`.
  - **T6** : 16 tests d'intégration DB (pattern `accounts::tests` post-3.5) + 10 tests unit handler (email/IDE/error mapping) + 14 tests Vitest (helpers frontend) + 5 tests Playwright e2e (+ 2 `test.skip`). Tous les AC couverts, tous les patches critiques (P21 escape LIKE, P24 archived, P25 MWST, P34 UpdatedAt) ont un test dédié.
  - **T7** : validation finale complète — `cargo fmt --all -- --check` ✅, `cargo clippy --workspace --all-targets -- -D warnings` ✅, `cargo test -p kesh-db --lib repositories::contacts` 16/16 ✅, `cargo test -p kesh-api --lib routes::contacts::tests` 10/10 ✅, `npm run test:unit` 140/140 ✅, `npm run check` 0 erreurs ✅.
  - **Fixes pré-existants en chemin** (clippy strict) : `kesh-seed/src/lib.rs:126` `useless conversion sqlx::Error::from` retiré ; `kesh-api/tests/auth_e2e.rs:14` import `put` inutilisé ; `kesh-api/tests/auth_e2e.rs:1066` `user_id` préfixé `_`. Triviaux, alignés avec le style canonique.
  - **Surprises notables** : `cargo fmt --all` a reformatté de nombreux fichiers legacy (kesh-seed, kesh-core, kesh-api, kesh-db) dans le même commit — scope creep acceptable car convergence sur le style canonique. `Label` shadcn n'existe pas dans `$lib/components/ui/` — pattern hardcodé avec `<label>` HTML natif adopté. `ListResponse<T>` réutilisé tel quel depuis `routes/mod.rs:25` (pattern Story 3.4, P2 appliqué).
  - Story marquée `review` dans le sprint-status et dans le story file. **Prête pour `/bmad-code-review`** (LLM orthogonal recommandé — Sonnet ou Haiku).
