# Story v014-1: CRUD bank_accounts post-onboarding + sidebar collapsible + restructuration UX (Issue #138)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **utilisateur de Kesh qui a terminé l'onboarding initial**,
I want **pouvoir gérer mes comptes bancaires (créer, éditer, archiver) après l'onboarding via une page UI dédiée et une navigation sidebar découvrable**,
so that je puisse ajouter un compte bancaire que j'avais oublié au wizard, modifier un IBAN saisi par erreur, archiver un compte fermé, et accéder à toutes les pages de configuration de Kesh (plan comptable, exercices, profils CSV bancaires, règles d'affectation) sans avoir à connaître les URLs par cœur.

## Scope

**Severity : bug catégorie A bloquant + UX rework majeur découverts en dogfooding live v0.1.3 sur prod NAS Synology Guy (2026-05-31).** L'absence de CRUD `bank_accounts` post-onboarding bloque l'émission de factures QR Bill (le QR Bill exige un IBAN primary configuré). 4 pages frontend sont orphelines de la sidebar (accessibles uniquement par URL directe). La nomenclature « Payer » dans la sidebar est trompeuse. Le widget « Comptes bancaires » de la page d'accueil pousse une action de configuration au lieu d'afficher de la valeur quotidienne (solde).

**Cible release** : v0.1.4 hotfix consolidé. **Closes** Issue #138.

### Backend — CRUD bank_accounts post-onboarding (gap critique bloquant)

Aujourd'hui le seul endpoint qui crée un `bank_account` est `POST /api/v1/onboarding/bank-account` (step 6→7 du wizard, refuse les appels post-onboarding via `if current.step_completed != 6 → return OnboardingStepAlreadyCompleted`). Aucun `POST /api/v1/bank-accounts` standalone n'existe. Le `PATCH /api/v1/bank-accounts/{id}` existant (Story 8-5a-zero) ne modifie QUE `journal_account_id`.

**Nouveaux endpoints** :

1. **`POST /api/v1/bank-accounts`** (Comptable+ — cohérent `comptable_routes` Story 8-5a-zero PATCH). Body `{ bankName, iban, qrIban?, isPrimary?, journalAccountId? }`. Réutilise les validations de `routes/onboarding.rs::set_bank_account` (lignes 470-489) : `bank_name.trim()` non-vide, IBAN via `kesh_core::types::Iban::new`, QR-IBAN via `kesh_core::types::QrIban::new` si fourni. Si `journalAccountId` fourni : pré-flight Account exists + actif + Asset|Liability (cohérent PATCH existant lignes 115-148). Si `isPrimary=true` ET un primary existe déjà : retourner 409 `BANK_ACCOUNT_PRIMARY_ALREADY_EXISTS` (vs. silencieux qui pourrait perdre le primary précédent). Retourne 201 + body `BankAccount`. Audit log `bank_account.created` émis depuis le handler (same tx).

2. **`PUT /api/v1/bank-accounts/{id}`** (Comptable+). Body `{ bankName, iban, qrIban?, isPrimary?, journalAccountId?, version }`. Édite tous les champs métier (vs. PATCH actuel qui ne touche que `journal_account_id`). Optimistic lock via `version`. Si `isPrimary=true` et un autre compte est primary : transition primary (mettre old_primary à false, set new à true) — atomique tx. Audit log `bank_account.updated` avec before/after JSON.

3. **`DELETE /api/v1/bank-accounts/{id}`** (Comptable+). Soft-delete via flag `archived` (NOUVELLE colonne migration, défaut `false`) — préserve audit + traçabilité. Refus 412 `BANK_ACCOUNT_HAS_TRANSACTIONS` si des `bank_transactions` existent sur ce compte (toutes statuts — pas de colonne `archived` sur `bank_transactions`, voir AC#8). Refus 412 `BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY` si le compte est primary ET au moins 1 autre compte non-archivé existe (forcer transition primary avant suppression — sinon l'instance n'aurait plus de primary). Si compte primary unique + pas de transactions : autoriser archivage (l'utilisateur peut tout casser s'il veut, on warn juste). Audit log `bank_account.archived`.

**Pas un PATCH générique multi-fields** (cf. comment ligne 38-40 de `bank_accounts.rs` actuel : « créer un handler dédié plutôt qu'un mega-PATCH »). On garde le PATCH actuel (`PATCH /bank-accounts/{id}` pour `journal_account_id` uniquement) **et** on ajoute le PUT pour l'édition complète. 2 endpoints distincts, scope clair.

**Audit log** : 3 nouveaux events `bank_account.created` / `bank_account.updated` / `bank_account.archived` (cohérent Story 8-5a-zero TODO L65 — à clôturer ici). `entity_type = "bank_account"`, `entity_id = bank_account.id`, `details_json` = snapshot before/after pour update, snapshot pour create/archive. **Note FINDING-10 Pass 1** : l'événement DELETE émet `bank_account.archived` (et **pas** `bank_account.deleted`) — terme précis pour un soft-delete, évite d'induire en erreur les auditeurs CO Art. 958f qui consulteraient l'audit log pour voir un compte « supprimé » encore présent en DB.

**Tests intégration backend** :
- `crates/kesh-api/tests/bank_accounts_e2e.rs` étendu (le fichier existe Story 8-5a-zero) : ajouter tests POST happy path + IBAN invalide + QR-IBAN invalide + primary collision 409 + PUT happy path + DELETE refus 412 si transactions + DELETE happy path archive + RBAC Consultation rejeté 403.
- Tests audit log : vérifier `bank_account.created/updated/deleted` events présents avec bons champs.

### Backend — migration DB `bank_accounts.archived`

Nouvelle migration `crates/kesh-db/migrations/<timestamp>_bank_accounts_archived.sql` :

```sql
ALTER TABLE bank_accounts
    ADD COLUMN archived BOOLEAN NOT NULL DEFAULT FALSE,
    ALGORITHM=INSTANT, LOCK=NONE;

CREATE INDEX idx_bank_accounts_company_active
    ON bank_accounts (company_id, archived);
```

Non-breaking (`ADD COLUMN nullable... DEFAULT FALSE`, anciens binaires ignorent). Pas de bump `kesh_version_min_required` requis (P3 CLAUDE.md). Audit doc à ajouter dans `docs/migrations-idempotence-audit.md` (P5 CLAUDE.md).

Repository `crates/kesh-db/src/repositories/bank_accounts.rs` mis à jour :
- `list_by_company()` : ajouter `AND archived = FALSE` au WHERE (filtrer archivés par défaut). Nouveau paramètre `include_archived: bool` (défaut `false`) pour le frontend qui veut un toggle « Afficher les archivés ».
- Nouvelle fonction `update(pool, id, company_id, NewBankAccount, expected_version)` pour PUT — transaction avec SELECT FOR UPDATE + UPDATE + retour `(updated, before)` cohérent pattern `set_journal_account_id_for_company`.
- Nouvelle fonction `archive(pool, id, company_id, expected_version)` pour DELETE soft.
- Vérification primary collision dans `create()` et `update()` (sélection FOR UPDATE sur is_primary=TRUE existant).

### Frontend — page `/bank-accounts` CRUD complet

Aujourd'hui (Story 8-5a-zero) : liste read-only des comptes + modal pour lier `journal_account_id` à un compte du plan comptable.

**Étendre** (`frontend/src/routes/(app)/bank-accounts/+page.svelte` + `frontend/src/lib/features/bank-accounts/`) :

- Bouton **« Nouveau compte bancaire »** en haut de la page → modal de création.
- Modal création : champs `bankName` (text), `iban` (text + masque format CH..), `qrIban` (text optionnel), `isPrimary` (switch boolean), `journalAccountId` (select dropdown filtré classes 1/2 Asset|Liability actifs). Validation côté client basique + propagation message d'erreur backend.
- Modal édition (`PUT`) : remplace ou complète l'édition actuelle. Bouton ✏️ par ligne → modal complet (tous les champs éditables). Garde optimistic-lock via `version`.
- Bouton 📦 archive par ligne → confirm dialog avec préavis si transactions existent. Appel `DELETE /api/v1/bank-accounts/{id}`. Si 412 → toast message backend (« contient X transactions, archiver-les d'abord »).
- Toggle **« Afficher les archivés »** en haut (cohérent pattern `/accounts` Story 3.1 ligne ~30 de `+page.svelte`).
- **Tooltip ou helper text** inline expliquant le lien `journalAccountId` : « Lie ce compte bancaire à un compte du plan comptable (typiquement 1020 Caisse, 1030 Banque). Permet à la réconciliation automatique (FR47) de créer les écritures vers le bon compte. Modifiable plus tard. »

**Wrapper API** `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` (le fichier existe Story 8-5a-zero) : ajouter `createBankAccount(payload)`, `updateBankAccount(id, payload)`, `archiveBankAccount(id, expectedVersion)`.

### Frontend — sidebar collapsible + restructuration sections

`frontend/src/routes/(app)/+layout.svelte` lignes 43-71 (`navGroups`).

**Restructuration** :

```js
const navGroups: Array<{ label: string | null; items: NavItem[]; defaultExpanded: boolean }> = [
    {
        label: 'Quotidien',
        defaultExpanded: true,
        items: [
            { i18nKey: 'nav-home', fallback: 'Accueil', href: '/' },
            { i18nKey: 'nav-contacts', fallback: "Carnet d'adresses", href: '/contacts' },
            { i18nKey: 'nav-products', fallback: 'Catalogue', href: '/products' },
            { i18nKey: 'nav-invoices', fallback: 'Facturer', href: '/invoices' },
            { i18nKey: 'nav-invoicing-due-dates', fallback: 'Échéancier', href: '/invoices/due-dates' },
            { label: 'Importer', href: '/bank-import' },
        ],
    },
    {
        label: 'Mensuel',
        defaultExpanded: true,
        // Note FINDING-12 Pass 1 : items Mensuel conservés en `label:` hardcodé FR
        // (cohérent avec l'existant pré-v014-1). Migration i18n complète reportée
        // v0.2 — hors scope du hotfix UX v0.1.4.
        items: [
            { label: 'Écritures', href: '/journal-entries' },
            { label: 'Réconciliation', href: '/reconciliation' },
            { label: 'Rapports', href: '/reports' },
        ],
    },
    {
        label: 'Administration',
        defaultExpanded: false,
        items: [
            { i18nKey: 'nav-accounts', fallback: 'Plan comptable', href: '/accounts' },
            { i18nKey: 'nav-fiscal-years', fallback: 'Exercices comptables', href: '/settings/fiscal-years' },
            { i18nKey: 'nav-bank-accounts', fallback: 'Comptes bancaires', href: '/bank-accounts' },
            { i18nKey: 'nav-bank-profiles', fallback: 'Profils bancaires', href: '/bank-import/profiles' },
            { i18nKey: 'nav-reconciliation-rules', fallback: 'Règles d\'affectation', href: '/reconciliation/rules' },
            { i18nKey: 'nav-export-global', fallback: 'Export global', href: '/export' },
            { i18nKey: 'nav-settings', fallback: 'Paramètres', href: '/settings' },
        ],
    },
];

// Admin-only items appended to "Administration" group dynamically
// Note FINDING-7 Pass 1 : conservés en `label:` hardcodé FR (cohérent avec le pattern existant
// des items Mensuel — i18n complète pour ces 2 entrées reportée v0.2 pour éviter d'étendre
// le scope i18n keys de AC#21 au-delà des 5 entrées Administration découvertes orphelines).
const adminOnlyItems: NavItem[] = [
    { label: 'Utilisateurs', href: '/users' },
    { label: 'Facturation', href: '/settings/invoicing' },
];
```

**Notes critiques** :
- Entrée « Payer » du groupe Quotidien **supprimée** (nom trompeur — pointait vers `/bank-accounts` qui est config référentielle, pas paiement).
- Groupe « Administration » **fusionne** :
  - Les 4 nouvelles entrées orphelines (Plan comptable, Exercices, Profils bancaires, Règles d'affectation).
  - L'ex-« Payer » renommé en « Comptes bancaires » (page CRUD étendue).
  - Les ex-`label: null` items (Export global, Paramètres) déplacés.
  - Les admin-only items (Utilisateurs, Facturation) **fusionnés dans Administration** au lieu d'un groupe séparé `ADMINISTRATION` (DRY : un seul groupe admin/config).

**Collapse** : implémentation via `<details>`/`<summary>` HTML natif.

```svelte
{#each navGroups as group}
    <details open={isGroupExpanded(group)} ontoggle={(e) => persistGroupState(group.label, e.currentTarget.open)}>
        <summary class="...">
            {group.label}
            <ChevronDown class="ml-auto h-4 w-4 transition-transform" />
        </summary>
        <ul>
            {#each group.items as item}
                <li><a href={item.href}>{item.label}</a></li>
            {/each}
            {#if group.label === 'Administration' && isAdmin}
                {#each adminOnlyItems as item}
                    <li><a href={item.href}>{item.label}</a></li>
                {/each}
            {/if}
        </ul>
    </details>
{/each}
```

**Persistence** via `localStorage` (**SSR-safe obligatoire** — cohérent pattern `frontend/src/lib/app/stores/mode.svelte.ts:21,30`) :
- Clés : `kesh:sidebar:expanded:quotidien`, `kesh:sidebar:expanded:mensuel`, `kesh:sidebar:expanded:administration`.
- Valeurs : `"true"` / `"false"`.
- **Lecture** : utiliser `typeof localStorage !== 'undefined' ? localStorage.getItem(key) : null` (ne pas crasher en SSR / hydration phase).
- **Écriture sur `ontoggle`** : guard `if (typeof localStorage !== 'undefined') { localStorage.setItem(key, ...) }`.
- Fallback à `group.defaultExpanded` si clé absente ou côté serveur.
- Helper functions `isGroupExpanded(group): boolean` + `persistGroupState(label, open)` appliquant ces guards.

**Accessibilité** : `<details>`/`<summary>` est natif a11y (ARIA expanded géré par le browser, focus keyboard, lecteurs d'écran). Pas de custom ARIA nécessaire. CSS animation chevron via `details[open] summary svg { transform: rotate(180deg); }`.

**i18n** : nouvelles clés `nav-accounts`, `nav-fiscal-years`, `nav-bank-accounts`, `nav-bank-profiles`, `nav-reconciliation-rules`, `nav-administration` (label groupe), `nav-quotidien`, `nav-mensuel` dans les 4 locales `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`.

### Frontend — page `/settings/+page.svelte`

Aujourd'hui (lignes 86-105) : section « Comptes bancaires » avec bouton « Modifier » → `notYet()` toast « Édition bientôt disponible ».

**Remplacer** par :
- Section listant les comptes bancaires (read-only, comme aujourd'hui).
- Bouton **« Gérer dans Administration → Comptes bancaires »** → `href="/bank-accounts"` (au lieu de toast).
- Texte d'aide en bas : « Pour ajouter, modifier ou archiver un compte bancaire, utilisez la page dédiée Administration → Comptes bancaires. »

Supprime le `notYet()` toast et la fonction `notYet()` (ligne 27-29) si elle n'est plus utilisée ailleurs (vérifier `grep -n notYet`).

### Frontend — widget « Comptes bancaires » page d'accueil

`frontend/src/routes/(app)/+page.svelte` lignes 77-102.

**Comportement actuel** :
- Aucun compte → widget affiché avec message « Aucun compte bancaire configuré. Ajoutez votre compte pour importer vos relevés. » + bouton « Configurer » vers `/settings`.
- Comptes existent → liste nom banque + IBAN + « Aucune transaction importée ».
- Bouton « Configurer » toujours présent.

**Refactor** :

| État | Affichage |
|------|-----------|
| **Aucun compte** | Widget **retiré complètement** du DOM (`{#if bankAccounts.length > 0}` enveloppe tout le widget). |
| **≥ 1 compte** | Widget viewer **sans CTA configuration** : pour chaque compte, afficher (a) nom banque, (b) IBAN raccourci (4 derniers chiffres : `••••2957`), (c) **solde actuel** (calculé serveur-side via `SUM(debit) - SUM(credit)` sur `journal_entry_lines` du `account_id` lié via `bank_accounts.journal_account_id`), (d) date dernière transaction si dispo. **Ligne de pied** : « Total liquidités : X CHF » (somme des soldes). |

**Backend support requis** : nouvelle route ou extension de `GET /api/v1/bank-accounts` pour retourner les soldes calculés.

**Décision** : étendre le payload `BankAccount` retourné par `GET /api/v1/bank-accounts` avec un champ `currentBalance: Decimal` calculé côté backend. **Important** : le calcul n'est fait que si `journal_account_id IS NOT NULL` (sinon le solde est `None`). Implémentation : query SQL avec LEFT JOIN sur `journal_entry_lines` + GROUP BY. Performance OK v0.1 (max ~10 comptes par company, query rapide).

**Alternative** : nouveau endpoint `GET /api/v1/bank-accounts/balances` qui retourne `[{ bankAccountId, balance, lastTransactionDate }]`. Moins de coupling, mais 2 requêtes pour le frontend. **Recommandation Pass 1 spec validate** : étendre `GET /api/v1/bank-accounts` (1 endpoint, moins de churn).

**Limitation v0.1** : si le `journal_account_id` n'est pas configuré sur un compte (cas par défaut au moment de la création post-onboarding tant que l'utilisateur n'a pas lié), `currentBalance = null` → affichage « Solde non disponible — lier le compte au plan comptable pour afficher le solde » avec petit lien vers `/bank-accounts`. UX gap acceptable v0.1.

### Documentation

- **`docs/user-guide/fr/getting-started.md`** : nouvelle section après §3 « Configurer l'exercice comptable 2026 » :

  **§3 bis — Lier ses comptes bancaires au plan comptable**
  - Pourquoi : permet à la réconciliation automatique (FR47) de créer les écritures vers le bon compte 1020/1030, et à la page d'accueil d'afficher les soldes courants.
  - Comment : Administration → Comptes bancaires → édition du compte → champ « Compte du plan comptable » → dropdown filtré classes 1 et 2 (Asset/Liability actifs). Typiquement choisir `1030 Banque` pour un compte courant, `1020 Caisse` pour la petite caisse.
  - Si l'utilisateur a plusieurs comptes courants distincts (BCV + PostFinance par ex.) : créer des **sous-comptes auxiliaires** sous 1030 (ex. `1030.001 BCV CHF`, `1030.002 PostFinance épargne`) via Plan comptable, puis lier chaque bank_account à son sous-compte respectif.

- **`docs/manual/fr/admin-manual.tex`** : pas de changement majeur (l'admin manual est focalisé sur le déploiement, pas l'usage métier). Éventuellement une mention en passant si pertinent dans la section « Configuration de la company ».

- **`CHANGELOG.md`** : nouvelle section `## [0.1.4] — Non publié`.

### Tests E2E à adapter

- **Tests Playwright qui cliquent sur sidebar entries** (`frontend/tests/e2e/*.spec.ts`) : vérifier que les sélecteurs survivent au passage à `<details>`/`<summary>`. Les liens sont toujours dans le DOM même quand le groupe est `<details>` fermé (juste cachés visuellement) → `page.click()` devrait marcher. Mais certains tests utilisent `page.getByRole('link', { name: '...' })` qui exige visibilité — il faudra peut-être ouvrir le `<details>` programmatiquement via `page.locator('summary').click()` d'abord. À auditer fichier par fichier.

- **Nouveaux tests Playwright** (`frontend/tests/e2e/bank-accounts-crud.spec.ts` NEW) :
  - Création compte bancaire happy path (depuis `/bank-accounts`).
  - Création avec IBAN invalide → message d'erreur.
  - Création avec QR-IBAN invalide → message d'erreur.
  - Édition d'un compte existant.
  - Archivage d'un compte sans transactions.
  - Tentative archivage compte primary → blocage avec message explicite.
  - Toggle « Afficher les archivés ».
  - Navigation depuis sidebar Administration → Comptes bancaires.

- **Nouveau test Playwright sidebar** (`frontend/tests/e2e/sidebar-navigation.spec.ts` NEW) : vérifier que les 4 ex-pages-orphelines sont atteignables via clic sidebar (`/accounts`, `/bank-import/profiles`, `/reconciliation/rules`, `/settings/fiscal-years`). Vérifier persistence localStorage (toggle expand → reload → état préservé).

### Hors scope (v0.2+)

- **Drag-and-drop ordering** des comptes bancaires (tri actuel : `ORDER BY is_primary DESC, id`).
- **Import CSV de plan comptable custom** (l'UI actuelle ne le permet pas, déjà documenté limitation L2 `getting-started.md`).
- **Multi-tenant fiduciaire** (1 utilisateur Kesh = 1 company v0.1).
- **Page « Payer » réelle** (génération `pain.001` paiements bancaires automatisés) → Epic 12.
- **Refactor adresse structurée** (Issue à créer suite à Q précédente Guy 19:00 — pain.001/QR Bill Structured Address).

## Contexte technique (ground-truth post-v011-5 + v0.1.3, vérifié 2026-05-31)

### Backend Rust — `crates/kesh-api/src/routes/bank_accounts.rs` (existing 211 lignes)

État actuel (Story 8-5a-zero) :
- `list_bank_accounts()` ligne 91-99 : `GET /api/v1/bank-accounts`, multi-tenant via `current_user.company_id`.
- `patch_bank_account_journal_link()` ligne 101-211 : `PATCH /api/v1/bank-accounts/{id}`, modifie UNIQUEMENT `journal_account_id`, valide Account exists + Asset|Liability, optimistic lock via `version`, audit `bank_account.updated` dans même tx, court-circuit no-op KF-004.
- `PatchJournalLinkBodyExtractor` (lignes 58-89) : extracteur custom qui convertit serde rejections en `AppError::Validation` (pattern réutilisable pour POST/PUT).

**Ajouts v014-1** :
- `create_bank_account()` handler — `POST /api/v1/bank-accounts`, Comptable+. Validation IBAN/QR-IBAN/primary collision. Audit `bank_account.created`.
- `update_bank_account()` handler — `PUT /api/v1/bank-accounts/{id}`, Comptable+. Édition complète tous champs. Optimistic lock. Audit `bank_account.updated` (étend l'usage du même action que le PATCH actuel — distinguer dans `details_json.trigger` = `"full_update"` vs `"journal_account_link"`).
- `archive_bank_account()` handler — `DELETE /api/v1/bank-accounts/{id}`, Comptable+. Soft-delete via `archived=TRUE`. Guards : 412 si transactions existent, 412 si primary unique. Audit `bank_account.archived`.
- `CreateBankAccountBodyExtractor`, `UpdateBankAccountBodyExtractor` (pattern Pass 1 P-H1 code review Story 8-5a-zero).

### Backend Rust — `crates/kesh-db/src/repositories/bank_accounts.rs` (existing 350+ lignes)

État actuel (Story 8-5a-zero) :
- `create(pool, NewBankAccount)` ligne 13-48 : INSERT simple + SELECT (transaction).
- `find_primary(pool, company_id)` ligne 51-63.
- `find_by_id_for_company(pool, company_id, id)` ligne 75-89 : KF-002 multi-tenant scoping.
- `list_by_company(pool, company_id)` ligne 92-104.
- `upsert_primary(pool, NewBankAccount)` : upsert idempotent pour onboarding.
- `set_journal_account_id_for_company(tx, ...)` : transaction-bound, retourne `(updated, before)`.
- `is_no_op_change(existing, new)` helper KF-004.

**Ajouts v014-1** :
- **MANDATORY (FINDING-1 Pass 1)** : Mettre à jour **toutes** les requêtes SELECT explicites de ce fichier (`FIND_BY_ID_SQL` ligne 9 + les autres lignes 56-57, 80-82, 96-98, 124-125, 253-255, plus les nouvelles queries `update_for_company`, `archive_for_company`) pour inclure la colonne `archived` dans la liste des colonnes sélectionnées. Sinon `sqlx::FromRow` crashera à runtime avec `ColumnNotFound("archived")` (aucun `#[sqlx(default)]` sur l'entité `BankAccount`). Également : ajouter le champ `pub archived: bool` à l'entité `BankAccount` dans `crates/kesh-db/src/entities/bank_account.rs`.
- `update_for_company(tx, company_id, id, new: NewBankAccount, expected_version) -> Result<(BankAccount, BankAccount), DbError>` : transaction-bound, SELECT FOR UPDATE + UPDATE + retour `(updated, before)`. Vérifie optimistic lock. Vérifie primary collision (si new.is_primary=true ET un autre compte primary existe → la transaction délègue à `transition_primary` helper qui flip l'ancien à false).
- `archive_for_company(tx, company_id, id, expected_version) -> Result<BankAccount, DbError>` : SELECT FOR UPDATE + UPDATE archived=TRUE + retour entity. Vérifie optimistic lock.
- `count_transactions_for_bank_account(pool, bank_account_id, company_id) -> i64` : helper pour guard 412.
- `list_by_company` étendue avec paramètre `include_archived: bool` (filter `WHERE archived = FALSE` par défaut).

### Backend Rust — `crates/kesh-api/src/lib.rs` (mount routes)

État actuel ligne 250-260 :
```rust
.route(
    "/api/v1/bank-accounts/{id}",
    patch(routes::bank_accounts::patch_bank_account_journal_link),
)
```

Et ligne 380-395 :
```rust
.route(
    "/api/v1/bank-accounts",
    get(routes::bank_accounts::list_bank_accounts),
)
```

**Ajouts** :
- Ajouter `post(routes::bank_accounts::create_bank_account)` au routeur `comptable_routes` (mutations) sur `/api/v1/bank-accounts`.
- Ajouter `.put(routes::bank_accounts::update_bank_account).delete(routes::bank_accounts::archive_bank_account)` à la chaîne existant sur `/api/v1/bank-accounts/{id}` dans `comptable_routes`. Le PATCH reste tel quel (scope distinct).

### Backend Rust — `crates/kesh-api/src/errors.rs` (variants à ajouter)

Nouveaux variants `AppError` :
- `BankAccountPrimaryAlreadyExists` → 409 Conflict, code `BANK_ACCOUNT_PRIMARY_ALREADY_EXISTS`.
- `BankAccountHasTransactions { transaction_count: i64 }` → 412 Precondition Failed, code `BANK_ACCOUNT_HAS_TRANSACTIONS`, body inclut `details.transactionCount`.
- `BankAccountCannotArchivePrimary` → 412 Precondition Failed, code `BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY`, message « Le compte principal ne peut pas être archivé tant qu'un autre compte non-archivé existe. Définissez d'abord un autre compte comme principal, puis archivez celui-ci. »

**i18n keys correspondantes pour les 4 locales FR/DE/IT/EN** (cohérent pattern projet `errors.rs:980,997` `bank-accounts-errors-account-not-found` / `bank-accounts-errors-invalid-account-type`) :
- `bank-accounts-errors-primary-already-exists`
- `bank-accounts-errors-has-transactions`
- `bank-accounts-errors-cannot-archive-primary`

### Frontend — `frontend/src/routes/(app)/+layout.svelte`

Structure actuelle (lignes 43-71) — voir extrait dans la section « Frontend — sidebar » ci-dessus pour la structure cible.

**Patch** :
- Remplacer la `const navGroups` actuelle par la nouvelle (3 groupes : Quotidien, Mensuel, Administration). Garder le type `NavItem` existant.
- Supprimer la `const adminNavItems` actuelle et les fusionner dans `Administration` via `{#if isAdmin}` block dans le `{#each}`.
- Ajouter helper functions `isGroupExpanded(group)` + `persistGroupState(label, open)` avec accès `localStorage`.
- Remplacer le rendu `{#each navGroups as group}` par le wrapping `<details><summary>...</summary><ul>...</ul></details>`.
- Garder les classes Tailwind existantes pour la cohérence visuelle.

**Vérification post-patch** :
- Mode Guest (non-authentifié) : sidebar masquée probablement par `(app)/+layout.ts` redirect → pas d'impact.
- Mode Consultation (lecture seule) : pas d'accès admin → les items Administration restent visibles mais pas Utilisateurs/Facturation (filtre `isAdmin`).
- Mode Admin : tous les items visibles dans Administration (les 7 standards + 2 admin-only fusionnés).

### Frontend — `frontend/src/routes/(app)/+page.svelte` (homepage widget)

État actuel lignes 77-102 (voir extrait spec). 3 widgets actuels probablement (factures, comptes, autres) — à vérifier scope complet de la page.

**Patch** :
- Envelopper tout le bloc widget « Comptes bancaires » dans `{#if bankLoaded && bankAccounts.length > 0}` → widget retiré si aucun compte.
- Retirer le bouton « Configurer » (lignes 99-101).
- Étendre l'affichage pour chaque compte : nom banque + IBAN raccourci (slice 4 derniers chars) + solde + date dernière transaction.
- Ajouter une ligne de pied « Total liquidités » sommant tous les soldes.
- **Ground-truth FINDING-5 Pass 1** : `bankAccounts` est actuellement chargé via `fetchCompanyCurrent()` (settings API, retourne `BankAccountJson` sans `currentBalance`). T9 doit **remplacer** par `listBankAccounts()` (depuis `bank-accounts.api.ts` — nom réel, pas `fetchBankAccounts`) qui retournera `BankAccountSummary` étendu avec `currentBalance`. Adapter les types côté composant (remplacer `BankAccountJson` par `BankAccountSummary`).

### Frontend — `frontend/src/lib/features/bank-accounts/`

Fichiers existants Story 8-5a-zero (**ground-truth FINDING-8 Pass 1** : pas de `bank-accounts.types.ts` séparé — les types `BankAccountSummary` + `AccountResponse` sont définis inline dans `bank-accounts.api.ts`) :
- `bank-accounts.api.ts` : `listBankAccounts()` (nom réel — **pas** `fetchBankAccounts`), `updateBankAccountJournalLink(id, payload)`, types `BankAccountSummary` + `AccountResponse` inline.
- `BankAccountList.svelte`, `BankAccountJournalLinkForm.svelte`.

**Ajouts v014-1** :
- `bank-accounts.api.ts` étendu (mêmes conventions : types inline dans le même fichier) :
  - Fonctions : `createBankAccount(payload)`, `updateBankAccount(id, payload)`, `archiveBankAccount(id, expectedVersion)`.
  - Types inline : `NewBankAccountPayload`, `UpdateBankAccountPayload` (camelCase pour serde matching), extension `BankAccountSummary` avec `currentBalance: number | null` ou `string | null` selon décision de précision Decimal (cf. FINDING-13).
- **Ne pas créer** de fichier `bank-accounts.types.ts` séparé (n'existe pas et n'est pas le pattern du module).
- Composants Svelte nouveaux dans `frontend/src/lib/features/bank-accounts/` : `CreateBankAccountModal.svelte`, `EditBankAccountModal.svelte`, `ArchiveBankAccountConfirmDialog.svelte`.

## Acceptance Criteria

### Backend — CRUD endpoints (AC #1-12)

- [ ] **AC #1** Migration `bank_accounts.archived BOOLEAN NOT NULL DEFAULT FALSE` ajoutée. Non-breaking. Audit `docs/migrations-idempotence-audit.md` mis à jour (P5 CLAUDE.md).
- [ ] **AC #2** `POST /api/v1/bank-accounts` route montée dans `comptable_routes` (`lib.rs`). RBAC Comptable+ requis (test 403 pour Consultation).
- [ ] **AC #3** `POST /api/v1/bank-accounts` body validation : `bankName.trim()` non-vide, IBAN format via `kesh_core::types::Iban::new`, QR-IBAN format via `kesh_core::types::QrIban::new` si fourni. Si `isPrimary=true` ET un primary existe déjà : 409 `BANK_ACCOUNT_PRIMARY_ALREADY_EXISTS`. Si `journalAccountId` fourni : Account exists + actif + Asset|Liability (réutilise pré-flight PATCH lignes 115-148).
- [ ] **AC #4** `POST /api/v1/bank-accounts` happy path retourne 201 + body `BankAccount` complet. Audit log `bank_account.created` émis en transaction avec INSERT.
- [ ] **AC #5** `PUT /api/v1/bank-accounts/{id}` route montée. RBAC Comptable+. Body avec tous les champs métier + `version` optimistic lock.
- [ ] **AC #6** `PUT /api/v1/bank-accounts/{id}` édition complète tous champs. Transition primary atomique si `isPrimary` change. Optimistic lock 409 si version stale. Audit log `bank_account.updated` avec `details_json.trigger = "full_update"` + before/after snapshot.
- [ ] **AC #7** `DELETE /api/v1/bank-accounts/{id}` route montée. RBAC Comptable+. Soft-delete via `archived=TRUE`.
- [ ] **AC #8** `DELETE` refuse 412 `BANK_ACCOUNT_HAS_TRANSACTIONS` si des `bank_transactions` existent sur ce compte (toutes statuts confondus — `pending` ET `reconciled`). Condition SQL : `SELECT COUNT(*) FROM bank_transactions WHERE bank_account_id = ? AND company_id = ?`. Note : `bank_transactions` n'a pas de colonne `archived` — le terme « non-archivées » dans le Scope §Backend était impropre. Refus inconditionnel dès qu'au moins une transaction existe (auditabilité CO Art. 958f — l'archivage d'un compte avec historique reconcilé serait problématique pour les rapports). Body inclut `details.transactionCount`.
- [ ] **AC #9** `DELETE` refuse 412 `BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY` si le compte est primary ET au moins 1 autre compte non-archivé existe.
- [ ] **AC #10** `DELETE` autorise l'archivage du primary unique (cas dégénéré — l'utilisateur sait ce qu'il fait). Audit log `bank_account.archived`.
- [ ] **AC #11** `PATCH /api/v1/bank-accounts/{id}` existant **inchangé** (scope strict `journal_account_id`). Le PUT n'écrase pas le PATCH.
- [ ] **AC #12** Repository `bank_accounts::list_by_company` étendu avec paramètre `include_archived: bool` (défaut `false`). Le handler `GET /api/v1/bank-accounts` accepte query param `?includeArchived=true` (défaut false).

### Backend — solde calculé (AC #13-15)

- [ ] **AC #13** Route `GET /api/v1/bank-accounts` retourne pour chaque compte un champ `currentBalance: Option<Decimal>` (camelCase serde). Si `journal_account_id IS NOT NULL` : calcul `SUM(debit) - SUM(credit)` sur `journal_entry_lines` du `account_id` lié. Si NULL : `currentBalance: null`.
- [ ] **AC #14** Performance acceptable v0.1 (~10 comptes max par company, query avec LEFT JOIN + GROUP BY). Pas de cache nécessaire v0.1.
- [ ] **AC #15** Tests unit ou intégration : compte sans journal_account_id → balance null. Compte avec journal_account_id + 3 écritures (2 débit 100, 1 crédit 30) → balance 170.

### Frontend — sidebar restructurée + collapsible (AC #16-21)

- [ ] **AC #16** `(app)/+layout.svelte` `navGroups` restructuré : 3 groupes (Quotidien, Mensuel, Administration) avec items selon spec ci-dessus. Entrée « Payer » supprimée. Entrée « Comptes bancaires » ajoutée à Administration. Pages orphelines `/accounts`, `/settings/fiscal-years`, `/bank-import/profiles`, `/reconciliation/rules` ajoutées à Administration. `/export` et `/settings` déplacés dans Administration.
- [ ] **AC #17** Admin-only items (`/users`, `/settings/invoicing`) fusionnés dans le groupe Administration via `{#if isAdmin}` block (au lieu d'un groupe séparé).
- [ ] **AC #18** Implémentation `<details>`/`<summary>` HTML natif pour chaque groupe. État par défaut : QUOTIDIEN + MENSUEL `open`, ADMINISTRATION fermé.
- [ ] **AC #19** Chevron `▾` / `▸` SVG (Lucide `ChevronDown`) à droite du label, rotation CSS via `details[open] summary svg { transform: rotate(180deg); }`.
- [ ] **AC #20** Persistence localStorage : 3 clés `kesh:sidebar:expanded:{quotidien,mensuel,administration}`. Lecture au mount, écriture sur `ontoggle`. Fallback à `defaultExpanded` si clé absente.
- [ ] **AC #21** i18n FR/DE/IT/EN ajoutées pour les 8 entrées sidebar principales : `nav-accounts`, `nav-fiscal-years`, `nav-bank-accounts`, `nav-bank-profiles`, `nav-reconciliation-rules`, `nav-administration`, `nav-quotidien`, `nav-mensuel`. **Note** : les 2 admin-only items (`/users` « Utilisateurs », `/settings/invoicing` « Facturation ») et les 3 items Mensuel restent en `label:` hardcodé FR (cohérent pattern existant — i18n complète reportée v0.2, hors scope du hotfix UX v0.1.4).

### Frontend — page `/bank-accounts` CRUD complet (AC #22-27)

- [ ] **AC #22** Bouton « Nouveau compte bancaire » en haut de la page → ouvre modal création.
- [ ] **AC #23** Modal création : champs bankName, IBAN, QR-IBAN optionnel, isPrimary switch, journalAccountId dropdown filtré classes 1/2 Asset/Liability actifs. Validation client (format basique) + propagation message d'erreur backend i18n.
- [ ] **AC #24** Modal édition (PUT) : remplace l'édition actuelle limitée à `journalAccountId`. Tous les champs éditables. Optimistic lock via version.
- [ ] **AC #25** Bouton archive 📦 par ligne → confirm dialog. Appel `DELETE`. Si 412 transactions → toast affiche `transactionCount`. Si 412 primary → toast explique procédure.
- [ ] **AC #26** Toggle « Afficher les archivés » en haut. Appel `GET /bank-accounts?includeArchived=true`. Lignes archivées affichées avec style désaturé.
- [ ] **AC #27** Tooltip ou helper text inline sur le champ `journalAccountId` : « Lie ce compte bancaire à un compte du plan comptable (typiquement 1020 Caisse, 1030 Banque). Permet à la réconciliation automatique de créer les écritures vers le bon compte. Modifiable plus tard. »

### Frontend — page `/settings/+page.svelte` (AC #28)

- [ ] **AC #28** Section « Comptes bancaires » : remplacer bouton « Modifier » (qui appelait `notYet()`) par lien `href="/bank-accounts"` avec texte « Gérer dans Administration → Comptes bancaires ». Texte d'aide en bas de la section. Supprimer fonction `notYet()` si plus utilisée ailleurs.

### Frontend — widget homepage (AC #29-30)

- [ ] **AC #29** `(app)/+page.svelte` widget « Comptes bancaires » : `{#if bankLoaded && bankAccounts.length > 0}` enveloppe tout le widget → retrait complet si aucun compte. Bouton « Configurer » supprimé.
- [ ] **AC #30** Widget viewer : pour chaque compte affiche `bankName`, IBAN raccourci (4 derniers chars : `••••2957`), `currentBalance` formatté CHF, date dernière transaction si disponible. Ligne de pied « Total liquidités » sommant les soldes. Si `currentBalance === null` (journal_account_id absent) : affiche « Solde non disponible — lier au plan comptable » + petit lien vers `/bank-accounts`.

### Tests E2E (AC #31-33)

- [ ] **AC #31** Tests Playwright existants qui cliquent sur la sidebar adaptés : ouvrir programmatiquement le `<details>` `Administration` via `page.locator('details summary:has-text("Administration")').click()` avant de cliquer sur un item Administration. **Cas spécifiques à adapter** :
  - `frontend/tests/e2e/users.spec.ts` (test `admin voit le lien Utilisateurs dans le sidebar` ligne 41) : ajouter un clic sur `details summary:has-text("Administration")` avant l'assertion `toBeVisible()` sur `nav-link-users` — un enfant d'un `<details>` fermé est dans le DOM mais **pas visible** (hidden via UA stylesheet) et fera échouer `toBeVisible()` (FINDING-14 Pass 1).
  - `frontend/tests/e2e/homepage-settings.spec.ts` (test « affiche 3 widgets sur la page d'accueil » lignes 33-38) : le widget `homepage-card-bank-accounts` est conditionnel (`{#if bankAccounts.length > 0}`) après AC#29. Le seed `with-company` (`seed_accounting_company`) ne crée aucun `bank_account`. **Dépendance confirmée grep** : ligne 37 du test contient `await expect(page.locator('[data-testid="homepage-card-bank-accounts"]')).toBeVisible();` — assertion ferme de visibilité du widget. Options : **(a) FORTEMENT RECOMMANDÉ — étendre le seed pour créer un bank_account** (préserve la visibilité attendue et la couverture du test ligne 37 sans modification du test), **(b)** mettre à jour le test pour asserter l'absence du widget (cassure ligne 37 acceptée, à reformuler en `toBeHidden()` ou `.count()===0`). Décision dev-story, mais (a) fortement préférée — évite régression du test ligne 37.
- [ ] **AC #32** Nouveau `frontend/tests/e2e/bank-accounts-crud.spec.ts` : 7 tests minimum (création happy path, IBAN invalide, QR-IBAN invalide, édition, archivage sans transactions, archivage avec transactions blocage 412, toggle archivés).
- [ ] **AC #33** Nouveau `frontend/tests/e2e/sidebar-navigation.spec.ts` : navigation vers les 5 entrées Administration (Plan comptable, Exercices, Comptes bancaires, Profils bancaires, Règles d'affectation) via clic sidebar. Test persistence localStorage : toggle expand Administration → reload page → état préservé.

### Documentation (AC #34-35)

- [ ] **AC #34** **CRÉER** `docs/user-guide/fr/getting-started.md` (nouveau fichier — **le répertoire `docs/user-guide/fr/` n'existe pas encore et doit être créé aussi**). Structure minimale : §1 vue d'ensemble, §2 connexion, §3 exercice comptable, **§3 bis** « Lier ses comptes bancaires au plan comptable » expliquant le rationale + procédure step-by-step + cas multi-comptes via sous-comptes auxiliaires (cf. Scope §Documentation pour le contenu détaillé de §3 bis).
- [ ] **AC #35** `CHANGELOG.md` : section `## [0.1.4] — Non publié` avec entrées standardisées [Keep a Changelog](https://keepachangelog.com/) :
  - **Added** : POST/PUT/DELETE `/api/v1/bank-accounts` (CRUD post-onboarding) ; sidebar collapsible `<details>`/`<summary>` avec persistence localStorage ; page `/bank-accounts` CRUD complet ; solde calculé `currentBalance` dans payload GET ; 5 entrées sidebar `nav-accounts/fiscal-years/bank-accounts/bank-profiles/reconciliation-rules` (pages plus accessibles via navigation).
  - **Changed** : widget homepage « Comptes bancaires » affiche les soldes au lieu d'un CTA configuration ; entrée sidebar « Payer » renommée en « Comptes bancaires » (sous Administration) ; structure sidebar réorganisée en 3 groupes (Quotidien/Mensuel/Administration).
  - **Removed** : bouton « Modifier » sur `/settings` § Comptes bancaires (remplacé par lien vers `/bank-accounts` AC#28) ; fonction `notYet()` supprimée si plus utilisée.
  - **Fixed** : pages orphelines (`/accounts`, `/bank-import/profiles`, `/reconciliation/rules`, `/settings/fiscal-years`) ajoutées à la sidebar (Issue #138).

  **`README.md`** : ajouter ligne `v0.1.4 (hotfix) | CRUD bank_accounts post-onboarding + sidebar collapsible + restructuration UX | 🚧 En cours` (puis `✅ Done` au release) dans le tableau « Feuille de route » (section §Feuille de route, après la ligne v0.1.3) — cohérent CLAUDE.md §« Synchroniser le planning du README à chaque commit ».

### Quality gate (AC #36)

- [ ] **AC #36** Test Locally First complet vert : `cargo fmt + clippy --workspace --all-targets -- -D warnings + build + test --workspace -j1 -- --test-threads=1` ; `npm run check + lint-i18n-ownership + test:unit + build` ; Playwright `bank-accounts-crud.spec.ts` + `sidebar-navigation.spec.ts` PASS local.

## Tasks / Subtasks

- [ ] **T1 — Migration DB + repository extension** (AC #1, #12)
  - [ ] Migration `bank_accounts.archived` BOOLEAN.
  - [ ] Audit `docs/migrations-idempotence-audit.md` mis à jour.
  - [ ] Repository `list_by_company` étendu avec `include_archived` param.
  - [ ] Nouvelle fonction `update_for_company(tx, ..., expected_version)`.
  - [ ] Nouvelle fonction `archive_for_company(tx, ..., expected_version)`.
  - [ ] Helper `count_transactions_for_bank_account(pool, bank_account_id, company_id)`.
  - [ ] Helper transition primary (flip old primary à false dans même tx que set new à true).
- [ ] **T2 — POST /api/v1/bank-accounts** (AC #2, #3, #4)
  - [ ] Handler `create_bank_account()` + `CreateBankAccountBodyExtractor`.
  - [ ] Variant `AppError::BankAccountPrimaryAlreadyExists` + i18n keys 4 locales.
  - [ ] Mount route dans `comptable_routes`.
  - [ ] Audit log `bank_account.created` dans transaction INSERT.
  - [ ] Tests intégration `bank_accounts_e2e.rs` : happy path, IBAN invalide, QR-IBAN invalide, primary collision 409, RBAC 403.
- [ ] **T3 — PUT /api/v1/bank-accounts/{id}** (AC #5, #6)
  - [ ] Handler `update_bank_account()` + `UpdateBankAccountBodyExtractor`.
  - [ ] Mount route (chaîne avec PATCH/DELETE existants).
  - [ ] Audit log `bank_account.updated` avec `details_json.trigger = "full_update"`.
  - [ ] Transition primary atomique.
  - [ ] Tests intégration : happy path, version stale 409, primary transition, RBAC 403.
- [ ] **T4 — DELETE /api/v1/bank-accounts/{id}** (AC #7-10)
  - [ ] Handler `archive_bank_account()`.
  - [ ] Variants `AppError::BankAccountHasTransactions{ transaction_count }` + `AppError::BankAccountCannotArchivePrimary` + i18n keys 4 locales.
  - [ ] Mount route.
  - [ ] Audit log `bank_account.archived`.
  - [ ] Tests intégration : happy path soft-delete, refus 412 transactions, refus 412 primary, archivage primary unique OK.
- [ ] **T5 — Solde calculé** (AC #13, #14, #15)
  - [ ] Repository : nouvelle fonction `list_by_company_with_balances(pool, company_id, include_archived) -> Vec<BankAccountWithBalance>` avec LEFT JOIN sur `journal_entry_lines`.
  - [ ] Handler `list_bank_accounts()` retourne le payload étendu.
  - [ ] Frontend : interface `BankAccountSummary` dans `bank-accounts.api.ts` (nom réel — **pas** `BankAccountResponse`) étendue avec `currentBalance: number | null`. **Conversion explicite obligatoire (FINDING-6 Pass 2)** : le backend Rust sérialise `rust_decimal::Decimal` en **string** via feature `serde-str` (cf. `crates/kesh-api/Cargo.toml:39`, pattern documenté `routes/products.rs:95` et `routes/vat.rs:109`). Donc le JSON reçu contient `{ "currentBalance": "1234.56" }` (ou `null`). Ajouter dans `listBankAccounts()` un helper de transformation : `currentBalance = item.currentBalance == null ? null : Number(item.currentBalance)`. Si la précision CHF 4 décimales est critique pour des calculs (au-delà de l'affichage), envisager une lib Decimal côté TS (`decimal.js`) — pour v0.1 (affichage uniquement, ≤ 2 décimales CHF), `Number()` suffit. Adapter `listBankAccounts()` pour retourner le payload avec conversion appliquée.
  - [ ] Tests : compte sans journal_account_id → null, avec écritures → solde correct.
- [ ] **T6 — Sidebar collapsible + restructuration** (AC #16-21)
  - [ ] Refactor `(app)/+layout.svelte` `navGroups`.
  - [ ] Implémentation `<details>`/`<summary>` + chevron CSS.
  - [ ] Persistence localStorage 3 clés + helpers `isGroupExpanded` / `persistGroupState`.
  - [ ] Fusion admin-only items dans groupe Administration via `{#if isAdmin}`.
  - [ ] 8 nouvelles clés i18n FR/DE/IT/EN.
- [ ] **T7 — Page /bank-accounts CRUD complet** (AC #22-27)
  - [ ] `CreateBankAccountModal.svelte` nouveau composant.
  - [ ] `EditBankAccountModal.svelte` (étend le modal édition actuel).
  - [ ] `ArchiveBankAccountConfirmDialog.svelte` nouveau composant.
  - [ ] Wrapper API `createBankAccount`, `updateBankAccount`, `archiveBankAccount`.
  - [ ] Bouton « Nouveau compte bancaire » + toggle « Afficher les archivés ».
  - [ ] Tooltip helper text pour `journalAccountId`.
- [ ] **T8 — Page /settings/+page.svelte** (AC #28)
  - [ ] Remplacer bouton « Modifier » `notYet()` par lien `/bank-accounts`.
  - [ ] Supprimer fonction `notYet()` si plus utilisée.
- [ ] **T9 — Widget homepage** (AC #29, #30)
  - [ ] Envelopper widget bancaires dans `{#if bankAccounts.length > 0}`.
  - [ ] Retirer bouton « Configurer ».
  - [ ] Affichage solde + IBAN raccourci + ligne total liquidités.
  - [ ] Gérer cas `currentBalance === null` avec message + lien.
- [ ] **T10 — Tests E2E Playwright** (AC #31-33)
  - [ ] Audit tests existants `frontend/tests/e2e/*.spec.ts` pour adaptation `<details>` Administration.
  - [ ] Nouveau `bank-accounts-crud.spec.ts` (7 tests minimum).
  - [ ] Nouveau `sidebar-navigation.spec.ts` (5 routes Administration + persistence localStorage).
- [ ] **T11 — Documentation + CHANGELOG** (AC #34, #35)
  - [ ] `docs/user-guide/fr/getting-started.md` : §3 bis ajouté.
  - [ ] `CHANGELOG.md` : section `[0.1.4] — Non publié`.
- [ ] **T12 — Quality gate** (AC #36)
  - [ ] Test Locally First série complète backend + frontend + Playwright.
  - [ ] Sprint-status v014-1 in-progress → review.

## Dev Notes

### Patterns à respecter (ground-truth code)

- **Extracteur custom** (`PatchJournalLinkBodyExtractor` ligne 58-89) : pattern pour POST/PUT bodies. Convertit serde rejections en `AppError::Validation` (400 standard Kesh) au lieu du 422 Axum natif. Réutiliser pour `CreateBankAccountBodyExtractor` + `UpdateBankAccountBodyExtractor`.
- **Audit log dans handler, jamais repo** : pattern Story 3-5 / 7-3 / 8-4 / 8-5a-zero. Le repo retourne `(updated, before)` ; le handler décide d'émettre l'audit log dans la même transaction via `audit_log::insert_in_tx(&mut tx, NewAuditLogEntry { ... })`.
- **Multi-tenant scoping (KF-002)** : toutes les queries scope `WHERE company_id = ?` strict. Anti-énumération : 404 (pas 403) sur cross-tenant. Pattern dans `find_by_id_for_company` ligne 75-89.
- **Optimistic lock** : champ `version` sur les tables mutables, bump à chaque UPDATE, rejet 409 `OptimisticLockConflict` si version client stale. Pattern dans `set_journal_account_id_for_company` (PATCH actuel).
- **No-op KF-004** : court-circuit si les valeurs ne changent pas (helper `is_no_op_change` existant). Évite bump version inutile + audit log spurious.
- **Validation IBAN** : `kesh_core::types::Iban::new(&body.iban)` retourne Result. Pattern dans `routes/onboarding.rs:478`.
- **Validation QR-IBAN** : `kesh_core::types::QrIban::new` retourne Result. Pattern dans `routes/onboarding.rs:484`.

### Pattern primary transition (référence)

L'invariant `is_primary` doit être unique par company (au plus 1 primary). Quand un POST/PUT set `is_primary=true` :
1. SELECT FOR UPDATE des comptes primary de la company (devrait y en avoir 0 ou 1).
2. Si un primary existe et ≠ celui qu'on modifie : UPDATE old_primary set is_primary=FALSE dans même tx.
3. INSERT/UPDATE new account set is_primary=TRUE.
4. Audit log entry pour les deux changements (old_primary démoté + new_primary promu).

**Décision Pass 1 (à trancher en spec validate)** : transition silencieuse (le user souhaite changer le primary, OK) OU explicite (refuser 409 et exiger un PUT explicite sur l'ancien primary d'abord) ?

**Recommandation** : silencieux mais avec audit log explicite (`details_json.trigger = "primary_transition"` sur l'ancien). UX user-friendly.

### Limitations documentées v0.1 (catégorie B)

- **L1 — Pas de soft-delete `archived=true → false` (restoration)** v0.1. Une fois archivé, le compte ne peut pas être ré-activé via UI. Workaround : SQL direct. Story Epic 11+ pour ajouter UI restoration.
- **L2 — Pas de bulk-archive ni bulk-edit** : opérations 1-par-1 v0.1.
- **L3 — Solde calculé non-cached** : recalculé à chaque GET. Acceptable v0.1 (max ~10 comptes), v0.2 si volume devient gros.
- **L4 — `currentBalance: null` UX message « lier au plan comptable »** : sous-optimal — idéalement on devrait pré-suggérer un compte par défaut (1030 Banque) à la création du compte bancaire. Story Epic 11+ pour auto-suggestion intelligente.

### Test Locally First (CLAUDE.md)

- Backend : `cargo fmt + clippy --workspace --all-targets -- -D warnings + build + test --workspace -j1 -- --test-threads=1`. Migration ajoutée → tests sqlx vont auto-apply, vérifier qu'aucune migration existante ne casse.
- Frontend : `npm run check + lint-i18n-ownership + test:unit + build`. Nouvelles clés i18n → lint i18n-ownership doit passer (i18n keys définies dans 4 locales, ownership feature/setup ou app/sidebar selon où placé).
- E2E Playwright : `bank-accounts-crud.spec.ts` + `sidebar-navigation.spec.ts` + tests existants adaptés (s'assurer qu'ils restent verts avec `<details>` sidebar).

### Règle de splitting préventif (CLAUDE.md)

Story touche **~12-15 fichiers** :
- Backend : `bank_accounts.rs` route, `bank_accounts.rs` repo, `lib.rs` mount, `errors.rs` variants, migration SQL, audit doc.
- Frontend : `+layout.svelte` sidebar, `bank-accounts/+page.svelte` page, `bank-accounts/` features (3 modals + API + types), `+page.svelte` homepage widget, `settings/+page.svelte`, 4 locales i18n.
- Doc : `getting-started.md`, `CHANGELOG.md`.
- Tests : 2 nouveaux e2e spec + audit tests existants.

Au-dessus du seuil > 5 modules. **Cohésion forte** (toutes les modifs servent le même but : CRUD bank_accounts post-onboarding + sidebar UX rework). Pas un rollout mécanique mais une feature cohérente.

→ **Maintenue en story unique**. Soupape : si `bmad-create-story validate` boucle > 4 passes sans converger, splitter en v014-1a (backend CRUD + tests intégration) + v014-1b (frontend sidebar + page CRUD + homepage + tests E2E). Frontière nette backend/frontend.

### Migration breaking policy (CLAUDE.md)

Migration `bank_accounts.archived BOOLEAN NOT NULL DEFAULT FALSE` est **non-breaking** (P3 : ADD COLUMN nullable... DEFAULT). Anciens binaires v0.1.3 ignorent le nouveau champ. Pas de bump `kesh_version_min_required`. Audit `docs/migrations-idempotence-audit.md` mis à jour (P5).

### Issue Tracking Rule (CLAUDE.md)

- Closes Issue #138 au merge.
- Pas de nouvelles KFs créées si tout passe. Si découvertes en cours de dev, créer via GitHub Issue (pas tracking local).

## Dev Agent Record

### Agent Model Used

_(à remplir au dev-story)_

### Debug Log References

_(à remplir au dev-story)_

### Completion Notes List

_(à remplir au dev-story)_

### File List

_(à remplir au dev-story)_

## Change Log

### Pass 2 Haiku 4.5 spec validate (2026-05-31)

**Modèle** : Haiku 4.5 (general-purpose sub-agent, fenêtre contexte fraîche — discipline CLAUDE.md grep ground-truth NON-NÉGOCIABLE pour CRIT/HIGH).
**Total findings remontés** : 8 — 1 CRITICAL + 3 HIGH + 3 MEDIUM + 1 LOW.
**Verdict** : `CONTINUE_TO_PASS_3` (post-triage 4 findings > LOW restants).

**Triage orchestrateur post-grep ground-truth** :

- **F1 CRITICAL DISMISSED** (doublon Pass 1) : « SELECT de bank_accounts manquent colonne archived » — déjà documenté MANDATORY ligne 273 du story file par F1 Pass 1. Haiku n'a pas remarqué que la spec post-Pass 1 contient déjà la directive. Grep ground-truth orchestrateur confirme `MANDATORY (FINDING-1 Pass 1)` présent ligne 273. Faux-positif typique Haiku « contexte combiné » (cf. CLAUDE.md §Haiku-specific guardrails).
- **F2 HIGH appliqué** : 3 i18n keys explicites pour les variants AppError (`bank-accounts-errors-primary-already-exists` / `-has-transactions` / `-cannot-archive-primary`). Confirmé ground-truth pattern `bank-accounts-errors-*` ligne 980, 997 `errors.rs`.
- **F3 HIGH → reclassé MEDIUM, appliqué** : SSR-safe localStorage. Confirmé pattern `typeof localStorage !== 'undefined'` ligne 21, 30 `mode.svelte.ts`. Reclassé car dev expérimenté l'appliquerait spontanément.
- **F4 HIGH → reclassé MEDIUM, appliqué** : AC#31 dépendance ligne 37 `homepage-settings.spec.ts` confirmée grep (`toBeVisible()` ferme sur `homepage-card-bank-accounts`). Option (a) « FORTEMENT RECOMMANDÉ » au lieu de « recommandé ». Amélioration éditoriale (pas architecturale).
- **F5 MEDIUM → reclassé LOW, appliqué** : AC#21 note explicite que admin-only items + Mensuel items restent `label:` hardcodé v0.1 (cohérent pattern existant).
- **F6 MEDIUM appliqué** : Decimal serde-str conversion. Confirmé ground-truth `serde-str` feature 5 occurrences `Cargo.toml` + 2 doc patterns (`products.rs:95`, `vat.rs:109`). T5 explicite la step `Number(item.currentBalance)` avec note v0.2 si précision Decimal.js nécessaire.
- **F7 LOW appliqué** : AC#35 sections Keep a Changelog (`Added/Changed/Removed/Fixed`) explicites.
- **F8 LOW skipped** : note spéculative CSS chevron timing — non-actionnable, no patch.

**Bilan patches Pass 2** : 6 appliqués / 1 dismissed (F1) / 1 skipped (F8). Trend : Pass 1 14 findings → Pass 2 8 findings (dont 1 doublon Pass 1) → décroissance attendue, convergence en cours.

**Prochaine étape** : Pass 3 Opus 4.7 (fenêtre contexte fraîche, recherche d'angles architecturaux cross-fichiers ratés par Sonnet+Haiku — pattern empirique CLAUDE.md §Review Iteration Rule).

### Pass 1 Sonnet 4.6 spec validate (2026-05-31)

**Modèle** : Sonnet 4.6 (general-purpose sub-agent, fenêtre contexte fraîche).
**Total findings** : 14 — 1 CRITICAL + 6 HIGH + 3 MEDIUM + 4 LOW.
**Verdict** : `CONTINUE_TO_PASS_2` (≥1 finding > LOW).

**Patches appliqués** (14/14) :

- **FINDING-1 CRITICAL** — toutes les requêtes SELECT existantes de `crates/kesh-db/src/repositories/bank_accounts.rs` (FIND_BY_ID_SQL + 6 autres) doivent inclure la colonne `archived` ; sinon `sqlx::FromRow` crashera runtime avec `ColumnNotFound("archived")`. Field `pub archived: bool` à ajouter à l'entité `BankAccount`.
- **FINDING-2 HIGH** — message variant `BankAccountCannotArchivePrimary` inversé sémantiquement (« tant qu'il est l'unique compte » → « tant qu'un autre compte non-archivé existe »).
- **FINDING-3 HIGH** — code error Scope ligne 29 `BANK_ACCOUNT_IS_PRIMARY` → `BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY` (alignement avec AC#9 + variants section).
- **FINDING-4 HIGH** — AC#8 condition SQL clarifiée : `bank_transactions` n'a pas de colonne `archived`, count toutes statuts (`pending` + `reconciled`). Terme « non-archivées » corrigé dans Scope §3 aussi.
- **FINDING-5 HIGH** — `fetchBankAccounts()` n'existe pas → `listBankAccounts()` (nom réel). Homepage charge actuellement via `fetchCompanyCurrent()` (settings API) à remplacer par `listBankAccounts()` dans T9.
- **FINDING-6 HIGH** — test `homepage-settings.spec.ts` cassera après AC#29 (widget conditionnel `{#if bankAccounts.length > 0}` + seed `with-company` ne crée pas de bank_account). AC#31 étendu avec 2 options : étendre seed OU adapter test pour absence du widget.
- **FINDING-7 HIGH** — navGroups Administration utilisent `i18nKey:` + `fallback:` au lieu de `label:` hardcodé pour les 5 entrées listées dans AC#21 (cohérence pattern Quotidien). Admin-only items conservés en `label:` (i18n reportée v0.2 hors scope).
- **FINDING-8 MEDIUM** — `bank-accounts.types.ts` n'existe pas → types inline dans `bank-accounts.api.ts` (pattern actuel du module). Ne pas créer le fichier.
- **FINDING-9 MEDIUM** — AC#34 explicite que `docs/user-guide/fr/getting-started.md` ET son répertoire parent doivent être **créés** (n'existent pas encore).
- **FINDING-10 MEDIUM** — audit log `bank_account.deleted` → `bank_account.archived` (5 occurrences). Terme précis pour soft-delete + cohérent CO Art. 958f auditabilité.
- **FINDING-11 MEDIUM** — AC#35 étendu : `README.md` Feuille de route doit recevoir ligne v0.1.4 (CLAUDE.md §Synchroniser planning README).
- **FINDING-12 LOW** — items Mensuel sans `i18nKey:` documentés explicitement comme choix conscient (hors scope hotfix).
- **FINDING-13 LOW** — T5 type frontend `BankAccountResponse` → `BankAccountSummary` (nom réel) avec note précision Decimal/Number.
- **FINDING-14 LOW** — couvert par FINDING-6 (AC#31 mentionne explicitement `users.spec.ts` cas `nav-link-users` + `toBeVisible()` qui exige groupe Administration ouvert).

**Ground-truth vérifications** (par sub-agent Pass 1) : 7 fichiers lus pour confirmer les findings (`bank_accounts.rs` route+repo, `errors.rs`, `bank-accounts.api.ts`, homepage `+page.svelte`, `test_fixtures.rs`, migration `bank_imports.sql`, `homepage-settings.spec.ts`).

**Prochaine étape** : `bmad-create-story validate` Pass 2 Haiku 4.5 (fenêtre contexte fraîche, cycle CLAUDE.md Review Iteration Rule — appliquer discipline grep ground-truth obligatoire pour tout finding CRITICAL/HIGH affirmant absence d'un code attendu ou présence d'un anti-pattern non-corrigé).

### Create-story (2026-05-31)

Story créée directement (pas via cycle BMAD `bmad-create-story` standard car pas d'epic-X.md formel pour v0.1.4 — source = Issue #138 comment consolidé post-dogfooding live v0.1.3 sur prod NAS Synology Guy).

Scope consolidé suite à 4 rounds de retours dogfooding 2026-05-31 :
1. **Round 1 (cookies HTTP-only)** → résolu v0.1.3 publié.
2. **Round 2 (pages orphelines sidebar)** → cette story v014-1.
3. **Round 3 (CRUD bank_accounts post-onboarding manquant)** → cette story v014-1.
4. **Round 4 (sidebar collapsible + restructuration sections + widget homepage solde)** → cette story v014-1.

Décision Guy 2026-05-31 19:00 : tout corriger avant de continuer le test, puis re-onboarding from-scratch avec compte bancaire ajouté + ajout d'autres comptes post-onboarding pour valider le flow CRUD complet.

36 ACs sur 11 sections (Backend CRUD + solde + sidebar + page CRUD + settings + homepage + tests E2E + doc + quality gate). Status `ready-for-dev`. Prochaine étape : **`bmad-create-story validate v014-1-bank-crud-and-sidebar-ux`** (Pass 1 Sonnet 4.6, cycle CLAUDE.md Review Iteration Rule).
