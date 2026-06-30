# Story 12.5d: Frontend import + complétion + justificatif & documentation

Status: ready-for-dev

<!-- Sous-story 4/4 (DERNIÈRE) de l'umbrella 12-5 (import répertoire factures, #194, cible v0.4). Périmètre : AC8 (frontend) + AC9 (doc) de l'umbrella. CONSOMME les 6 endpoints livrés par 12-5c (aucune logique transactionnelle en frontend). Les compteurs « en dur » (TABLES_TO_TRUNCATE, manifeste export, admin_full_export_e2e=31, migrations_upgrade_path=39, audit idempotence) ont DÉJÀ été faits en 12-5b → HORS scope 12-5d. -->

## Story

As a comptable PME utilisant Kesh,
I want un écran pour **déclencher l'import** du dossier de factures, voir le **rapport** (créées / échecs), **compléter** chaque facture importée (fournisseur + date + lignes) et **écarter** les non pertinentes, puis **retrouver le justificatif** depuis le détail de la facture,
so that je n'ai plus à ressaisir les coordonnées de paiement ni à classer les fichiers, et je retrouve la pièce d'origine en un clic.

## Contexte & source

- Sous-story **4/4 (dernière)** de l'umbrella **12-5** (#194). Cf. `12-5-import-repertoire-factures.md` AC8/AC9.
- **Dépend de 12-5c** (`12-5c-service-import-completion.md`, status `done`) qui livre les **6 endpoints** consommés ici (Comptable+, company-scopés) :
  - `POST /api/v1/inbox-import` → `{ accepted:[{importedSupplierInvoiceId, fileName}], failed:[{fileName, errorCode, details?}], warnings:[String] }` (HTTP 200) ; **409** `INBOX_IMPORT_ALREADY_RUNNING` si un import est déjà en cours.
  - `GET /api/v1/imported-supplier-invoices?status={to_complete|completed|discarded}` → `ImportedSupplierInvoice[]` (status **obligatoire**, sinon 400).
  - `POST /api/v1/imported-supplier-invoices/{id}/complete` (body camelCase : `contactId, invoiceDate, supplierInvoiceNumber?, dueDate?, lines:[{description, quantity, unitPrice, vatRate, expenseAccountId}]`) → `SupplierInvoiceResponse` (200). Rejets : **409** `IMPORT_NOT_PENDING_COMPLETION` (`details.currentStatus`), **404** `IMPORTED_INVOICE_NOT_FOUND`, **400** `CURRENCY_NOT_SUPPORTED` / `IBAN_REFERENCE_MISMATCH` / `AMOUNT_MISMATCH` (`details.expected`/`actual` pour AMOUNT_MISMATCH), + erreurs `create` (FISCAL_YEAR_INVALID, etc.).
  - `POST /api/v1/imported-supplier-invoices/{id}/discard` → **204**.
  - `GET /api/v1/imported-supplier-invoices/{id}/source-document` → binaire (avant complétion). 404 `SOURCE_DOCUMENT_NOT_FOUND` / 410 `SOURCE_DOCUMENT_GONE`.
  - `GET /api/v1/supplier-invoices/{id}/source-document` → binaire (après complétion). 404 si pas de justificatif importé (L5).
- **Clôt l'épopée 12-5** : après merge de cette sous-story, l'umbrella PR (12-5a..d) peut être ouverte (gate Guy).

## Périmètre (et hors-périmètre)

**DANS 12-5d** :
- **Feature frontend `imported-supplier-invoices`** (types TS + module API) : wrappers des 6 endpoints + types miroir.
- **Écran « Importer le dossier »** (Comptable+) : bouton déclencheur → rapport batch rendu (créées, échecs avec `fileName` + `errorCode` **traduit**, warnings) ; gestion du **409** « déjà en cours ».
- **Liste des factures importées `to_complete`** + actions **Compléter** (formulaire) et **Écarter** (confirmation).
- **Formulaire de complétion** : coordonnées QR pré-affichées en lecture seule (créancier, IBAN, montant cible, référence) ; sélection fournisseur (`is_supplier`), date facture, n° + échéance optionnels, lignes (description/quantité/PU/TVA/compte de charge) ; mapping des rejets 400/409 en messages UX traduits.
- **Download justificatif** : « Voir le justificatif » sur chaque importée (avant complétion) ; lien **« Voir la facture d'origine »** sur le détail facture fournisseur (12-2, après complétion).
- **Sidebar** : entrée Comptable+ vers l'écran d'import.
- **i18n** : toutes les chaînes via `i18nMsg('clé', 'fallback FR')` + map `errorCode → libellé FR` pour le rapport.
- **Doc** : `.env.example` (5 vars inbox/documents), `docker-compose.yml` + `docker-compose.prod.yml` (volumes `/data/inbox` + `/data/documents` + env), `CHANGELOG.md` `[Non publié]`, `README.md` (Fonctionnalités + Feuille de route), `docs/manual/fr/admin-manual.tex` (section import + pdfium + vars). PDF manuel régénéré.
- **E2E Playwright** : `frontend/tests/e2e/inbox-import.spec.ts` (flux UI).

**HORS 12-5d** :
- Toute logique transactionnelle / décodage / sécurité (= 12-5c backend, `done`).
- Compteurs « en dur » (TABLES_TO_TRUNCATE, manifeste export, `admin_full_export_e2e`, `migrations_upgrade_path`, audit idempotence) — **DÉJÀ faits en 12-5b** (vérifié ground-truth).
- Création inline de fournisseur (L4 umbrella — sélection d'un contact `is_supplier` existant ; sinon l'utilisateur le crée via le flux Contacts puis revient).
- Macros version manuel `\keshVersion` (= gate release, pas cette story).

## Acceptance Criteria

### Module API & types

1. **Feature `frontend/src/lib/features/imported-supplier-invoices/`** (calque `supplier-invoices/`) : `imported-supplier-invoices.types.ts` (types miroir : `ImportedSupplierInvoice`, `InboxImportReport`, `AcceptedFile`, `FailedFile`, `CompleteImportRequest`, `CompleteImportLineRequest`) + `imported-supplier-invoices.api.ts` (6 wrappers via `apiClient`). Décimaux = **`string`** (rust_decimal sérialisé string, cf. ground-truth). Le `status` du query param est **obligatoire** (le wrapper `listImported(status)` l'exige, pas de défaut implicite).

### Écran import + rapport

2. **Page `/(app)/supplier-invoices/import/+page.svelte`** (Comptable+ via `+page.ts` guard, pattern `users/+page.ts` adapté `role !== 'Admin' && role !== 'Comptable'`) : bouton **« Importer le dossier »** → `triggerInboxImport()` → affiche le **rapport** :
   - `accepted.length` créées (message succès) ;
   - `failed[]` listées avec `fileName` + `errorCode` **traduit** (map dédiée, cf. AC8) + `details` éventuels ;
   - `warnings[]` affichés (bandeau info) ;
   - **409 `INBOX_IMPORT_ALREADY_RUNNING`** → message « Un import est déjà en cours, réessayez » (distinct d'un rapport partiel — le `catch` discrimine via `isApiError(e) && e.code === 'INBOX_IMPORT_ALREADY_RUNNING'`).
   - Bouton désactivé + spinner pendant l'import (le run est synchrone, peut durer plusieurs secondes — pdfium).
   - Après import, la **liste `to_complete` est rechargée** (les nouvelles importées apparaissent).
   - **Map `errorCode → libellé FR**** (i18n) couvrant les 10 codes : `UNSUPPORTED_FILE_TYPE`, `FILE_TOO_LARGE`, `SYMLINK_REJECTED`, `DUPLICATE`, `NO_QR_CODE_FOUND`, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR`, `FILE_READ_ERROR`, `FIELD_TOO_LONG`. Code inconnu → fallback générique « Échec de l'import (code) ».

### Liste & complétion

3. **Liste `to_complete`** (sur la même page ou onglet) : pour chaque importée, afficher les coordonnées QR (créancier, IBAN, montant, référence, devise) + 3 actions : **Compléter**, **Écarter**, **Voir le justificatif** (download via `imported source-document`).

4. **Formulaire de complétion** (inline ou Dialog, pattern `supplier-invoices/+page.svelte`) :
   - **Pré-affichage lecture seule** des coordonnées QR (guide l'utilisateur) : nom créancier, IBAN, **montant cible** (`staging.amount` — la somme TTC des lignes devra l'égaler exactement), référence.
   - **Champs éditables** : sélection fournisseur (contacts `is_supplier=true`, `listContacts({ isSupplier: true })`), `invoiceDate` (requis), `supplierInvoiceNumber?`, `dueDate?`, **lignes** (description, quantité, PU HT, taux TVA via `listVatRates`, compte de charge `Expense` actif). Décimaux saisis en `type="text" inputmode="decimal"`, calculs via **`big.js`** (`computeLineTotal`), affichage via `formatSwissAmount`.
   - **Total TTC live** affiché ; si `staging.amount` présent, indicateur visuel quand `Σ TTC ≠ montant cible` (le backend rejette `AMOUNT_MISMATCH` — l'UI guide AVANT soumission mais ne duplique pas la règle d'autorité).
   - Soumission → `completeImport(id, body)` → succès `notifySuccess` + retrait de la liste + (option) redirection vers `/supplier-invoices/{newId}`.
   - **Mapping rejets → messages UX traduits** (via `isApiError` + `err.code`) : `CURRENCY_NOT_SUPPORTED` (« Devise non supportée (CHF uniquement) »), `IBAN_REFERENCE_MISMATCH` (« Incohérence IBAN/référence QRR »), `AMOUNT_MISMATCH` (« Le total des lignes (`details.actual`) ≠ montant QR (`details.expected`) »), `IMPORT_NOT_PENDING_COMPLETION` (« Déjà complétée/écartée »), `FISCAL_YEAR_INVALID` (« Aucun exercice ouvert pour cette date »), fallback `err.message`.

5. **Écarter** : confirmation (Dialog ou `confirm`) → `discardImport(id)` (204) → retrait de la liste + `notifySuccess`. Texte : « Le fichier justificatif reste conservé. »

### Justificatif sur le détail facture

6. **Détail facture fournisseur** (`/(app)/supplier-invoices/[id]/+page.svelte`) : lien **« Voir la facture d'origine »** → download via `GET /supplier-invoices/{id}/source-document` (Pattern A `getBlob` + ancre éphémère, gère 401-refresh). **DC-d1 (à ratifier validate)** : le lien est **toujours rendu** ; au clic, un 404 `SOURCE_DOCUMENT_NOT_FOUND` (facture créée directement 12-2, L5) ou 410 `SOURCE_DOCUMENT_GONE` (fichier non restauré, L1/F7) → `notifyInfo`/`notifyWarning` traduit, **PAS** d'erreur bloquante. Évite un champ backend `hasSourceDocument` (modif de 12-5c `done`). *(Alternative envisageable en validate : ajouter `hasSourceDocument: bool` à `SupplierInvoiceResponse` pour un rendu conditionnel — déviation backend, à trancher.)*

### Navigation & i18n

7. **Sidebar** (`(app)/+layout.svelte`) : entrée vers `/supplier-invoices/import` (groupe « Quotidien », pattern existant — visible tous rôles côté sidebar, accès réel gardé Comptable+ backend + `+page.ts`). `data-testid` auto `nav-link-supplier-invoices-import`.

8. **i18n** : 100% des chaînes via `i18nMsg('clé', 'fallback FR')` (système runtime custom, `i18nMsg` de `i18n.svelte.ts` — PAS de `.ftl`/paraglide). Clés préfixées `import-` / `imported-supplier-invoices-`. Les valeurs FR sont le fallback inline (les locales DE/IT/EN sont gérées côté backend `/api/v1/i18n/messages`, valeurs FR provisoires acceptées — cohérent politique projet). Map `errorCode → libellé` = clés `import-error-<code-kebab>`.

### Documentation

9. **`.env.example`** : nouvelle section « Import de factures depuis un dossier (#194) » avec les 5 vars + défauts + note volumes : `KESH_DOCUMENTS_DIR` (`/data/documents`), `KESH_INBOX_DIR` (`/data/inbox`), `KESH_INBOX_MAX_FILE_BYTES` (26214400 = 25 Mo), `KESH_INBOX_MAX_FILES_PER_RUN` (200), `KESH_INBOX_MAX_PDF_PAGES` (20). Ajout après la section `KESH_ADMIN_EXPORT_INMEM_MB`.

10. **`docker-compose.yml` + `docker-compose.prod.yml`** : dans `kesh-api`, ajouter `environment` (`KESH_DOCUMENTS_DIR`/`KESH_INBOX_DIR` + 3 MAX_) + `volumes` (`/data/inbox` + `/data/documents`). Prod : volumes **nommés** (`kesh-documents:`, `kesh-inbox:`) déclarés dans la section `volumes:` (persistance). Dev : bind mounts host (`./inbox:/data/inbox`, `./documents:/data/documents`) acceptables. Cohérent avec le mapping du manuel admin.

11. **`CHANGELOG.md`** `[Non publié]` → `### Ajouté` : bullet « Import de factures fournisseurs depuis un dossier (#194) » en termes utilisateur (bouton « Importer le dossier », détection Swiss QR, staging « à compléter », justificatif récupérable).

12. **`README.md`** : section « Fonctionnalités » — retirer/mettre à jour les mentions « *(scan QR-facture à venir)* » (lignes ~31-32) ; ajouter l'import-répertoire. « Feuille de route » — refléter l'import dans la ligne v0.x correspondante (✓ / 🚧).

13. **`docs/manual/fr/admin-manual.tex`** : (a) entrées des 5 vars dans la table « Variables optionnelles » (~l.659) ; (b) nouvelle sous-section `\subsection{Import de factures depuis un dossier}` : configuration `KESH_INBOX_DIR` sur NAS Synology (dossier partagé), caveat canonicalisation symlink `/data`→`/volume1`, persistance `KESH_DOCUMENTS_DIR`, réglage `KESH_INBOX_MAX_PDF_PAGES`, limitation **L6** (pdfium segfault PDF malformé tue le process) + **L8** (inbox non partitionné par tenant, Issue #199). **Régénérer le PDF** (`make fr` dans `docs/manual/` ; **NE PAS** bumper `\keshVersion` — gate release).

### Qualité

14. **E2E Playwright** `frontend/tests/e2e/inbox-import.spec.ts` (pattern `supplier-invoices.spec.ts`) — flux **UI** :
    - login → navigation vers `/supplier-invoices/import` → clic « Importer le dossier » → rapport rendu (au minimum le bandeau résultat, inbox vide → 0 créées sans crash).
    - liste `to_complete` + ouverture formulaire complétion + soumission (sur une importée **seedée**) → statut passe `completed` / disparaît de la liste.
    - écarter une importée → disparaît.
    - **DC-d4 (mécanisme de seed E2E, à ratifier validate)** : seeder une row `imported_supplier_invoices` (preset `seedTestState` étendu OU dépôt d'un fichier fixture QR PNG dans `KESH_INBOX_DIR` du serveur de test avant déclenchement). Le pipeline d'import lui-même est **déjà couvert** par les 16 tests d'intégration 12-5c — l'E2E 12-5d couvre les **flux UI**, pas le décodage.
    - Pré-requis E2E : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (Ubuntu 26.04+, cf. memory) + MariaDB + seed CI.

15. **Quality gate Test Locally First — exit code vérifié** :
    - Frontend : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build`.
    - E2E : `cd frontend && npm run test:e2e` (avec override Playwright).
    - Backend : si aucun fichier Rust touché (12-5d = frontend+doc), la suite backend reste verte par construction ; lancer `cargo build --workspace` par sécurité si un type partagé bouge. **PAS** `cargo test | grep` (exit code masqué, cf. `feedback_cargo_test_pipe_masks_exit`).
    - 0 régression `npm run test:unit` (333 vitest baseline) + `svelte-check` 0 erreur.

## Décisions de conception (DC) — à figer en validate

- **DC-d1** — visibilité « Voir la facture d'origine » : **toujours rendu + 404/410 gracieux** (défaut, pas de modif backend 12-5c). Alternative : champ `hasSourceDocument` backend (déviation). → trancher validate/Guy.
- **DC-d2** — route de l'écran : **`/supplier-invoices/import`** (sous-route proche du domaine). Alternative `/imported-supplier-invoices`. → figer validate.
- **DC-d3** — le formulaire **pré-affiche le montant QR cible** et indique visuellement l'écart, mais **ne duplique pas** la règle de réconciliation exacte (autorité = backend `AMOUNT_MISMATCH`). L'égalité est centime-exacte côté backend (F-OPUS-2) ; l'UI guide sans bloquer la soumission (le backend tranche).
- **DC-d4** — mécanisme de seed E2E (fixture inbox vs preset DB). → figer validate.
- **DC-d5** — la liste des importées est rechargée après import/complétion/écart (pas de store global réactif inter-pages nécessaire pour v0.4).

## Dev Notes

### Ground-truth frontend (vérifié 2026-06-30, 2 agents Explore)

**Routing & guards** :
- Pages authentifiées sous `frontend/src/routes/(app)/`. `supplier-invoices/+page.svelte` (liste) + `supplier-invoices/[id]/+page.svelte` (détail) **existent**.
- Guard rôle = `+page.ts` `load()` : `import { browser } from '$app/environment'; import { redirect } from '@sveltejs/kit'; import { authState } from '$lib/app/stores/auth.svelte';` + `export const ssr = false;` + `if (browser && role !== 'Admin' && role !== 'Comptable') throw redirect(302, '/');`. (Pattern Admin-only dans `users/+page.ts`.) `Role = 'Admin' | 'Comptable' | 'Consultation'` (`$lib/shared/types/user.ts`).
- `(app)/+layout.ts` garde déjà `isAuthenticated`.

**API client** (`$lib/shared/utils/api-client.ts`) : `apiClient.{get,getBlob,post,postFormData,put,patch,delete}`. Auth = cookie httpOnly (`credentials:'include'`, refresh transparent sur 401). Erreurs = `{error:{code,message,details}}` → `ApiError{code,message,details?,status}` ; `isApiError(err)` type guard. Exemple POST : `apiClient.post('/api/v1/supplier-invoices', req)` dans `supplier-invoices.api.ts`.

**Formulaire lignes** (`supplier-invoices/+page.svelte`) : `select` fournisseur (`suppliers` chargés via `listContacts({ isSupplier:true, limit:200 })`), grille `grid-cols-12` par ligne (description col-4, quantity, unitPrice, vatRate `select` via `listVatRates()`, expenseAccount `select` via `fetchAccounts()` filtré `accountType==='Expense' && active`). Lignes init en **strings** (`quantity:'1', unitPrice:'', vatRate:'0', expenseAccountId:0`). `onMount` + `Promise.all` pour charger contacts/comptes/taux.

**Décimaux** : strings backend. Calc via `big.js` (`computeLineTotal(qty, unitPrice)` → `new Big(qty).times(unitPrice).round(4, ROUND_HALF_UP).toFixed(4)`). Affichage `formatSwissAmount(new Big(d))` → `1'234.50`. Input `type="text" inputmode="decimal"`, regex `^\d{1,15}([.,]\d{1,4})?$`.

**i18n** (`$lib/shared/utils/i18n.svelte.ts`) : `i18nMsg(key, fallbackFR, args?)`, substitution `{ $var }`. Messages servis par `/api/v1/i18n/messages` selon locale. FR = fallback inline. DE/IT/EN désactivés (dropdown disabled, Story 2.1).

**Download authentifié** : Pattern A (recommandé, gère 401) `apiClient.getBlob(url)` → `blob()` → ancre éphémère `triggerDownload` (cf. `reports.api.ts` / `exports.api.ts`, filename via `parseContentDispositionFilename`). Pattern B (simple) ancre `<a href={url} download={name}>` (cf. `payment-batches/[id]/+page.svelte` pain.001).

**Sidebar** (`(app)/+layout.svelte`) : `navGroups` (quotidien/mensuel/administration). `supplier-invoices` + `payment-batches` déjà dans « quotidien » (non restreints sidebar). `isAdmin = $derived(role==='Admin')`. Ajouter l'entrée import dans `quotidien.items`. `navTestid(href)` → `nav-link-...`.

**Composants UI** (`$lib/components/ui/`) : shadcn/bits-ui — `Button` (variant default/outline/ghost/destructive, size sm), `Input`, `Dialog.{Root,Content,Header,Title,Description,Footer,Trigger}`, `Select` (mais pages utilisent `<select>` brut). `notify.ts` : `notifySuccess/Error/Warning/Info`. Icônes `@lucide/svelte`. `ContactPicker` (combobox ARIA, props `selected/onSelect/placeholder`, n'expose PAS de filtre `isSupplier` → soit étendre, soit garder le `<select>` simple chargé `isSupplier:true`). `BankImportUpload.svelte` = analogue le plus proche pour « déclencher + afficher rapport batch ».

**Tokens CSS** : `bg-primary/text-primary`, `text-text-muted`, `text-destructive/text-error`, `border-border`, `bg-surface/-alt/-hover`, `bg-warning-soft/border-warning`, `bg-error-soft/border-error`.

**Gaps à créer** (pas d'équivalent existant) : feature module `imported-supplier-invoices` (types+api), page `/supplier-invoices/import`, guard Comptable+ (1er du genre), entrée sidebar Comptable+ (ou via `items` + RBAC backend), lien justificatif sur détail (pas de champ `hasSourceDocument`).

### Ground-truth doc/compteurs (vérifié 2026-06-30)

- **DÉJÀ FAIT en 12-5b** (HORS scope 12-5d) : `TABLES_TO_TRUNCATE` (`backup.rs:41`, `imported_supplier_invoices` avant `supplier_invoices`/`companies`), manifeste export (dynamique via `TABLES_TO_TRUNCATE`, `admin_backup/export.rs:54`), `admin_full_export_e2e` (`data_count==31`), `migrations_upgrade_path` (`total==39`, `total-16`), `docs/migrations-idempotence-audit.md` (row + total 39).
- **À FAIRE en 12-5d** : `.env.example` (0 mention inbox/documents actuellement), `docker-compose.yml` (volumes lignes 66-68, pas de inbox/documents) + `docker-compose.prod.yml` (volumes lignes 110-114, idem), `CHANGELOG.md` (`[Non publié]` sans mention import), `README.md` (lignes 31-32 « scan QR-facture à venir » + roadmap), `admin-manual.tex` (table vars l.659 sans inbox, pas de section pdfium).
- **E2E** : specs sous `frontend/tests/e2e/`. Modèle `supplier-invoices.spec.ts` / `payment-batches.spec.ts` : `seedTestState('with-company')` (beforeAll), `clearAuthStorage` (afterEach), helper `login(page)` (`#username`/`#password` `admin`/`admin123`, attend `/`).

### Conventions
- **Pas de logique métier en frontend** : le frontend consomme les endpoints 12-5c, ne réimplémente PAS la réconciliation/atomicité.
- **Test Locally First** exit code vérifié (frontend check/lint-i18n/test:unit/build + e2e ; PAS `| grep`).
- **Multi-tenant** : tous les endpoints sont déjà company-scopés backend ; le frontend n'ajoute pas de scoping.
- **Branche** : `story/12-5-import-repertoire-factures` (umbrella, déjà sur la branche).
- **Contrat d'autonomie** : STOP si modification architecturale. L'ajout éventuel de `hasSourceDocument` (DC-d1 alternative) toucherait le backend 12-5c `done` → relève d'une décision (validate/Guy), pas auto-appliqué.

### Limitations (héritées umbrella)
- **L4** — création inline fournisseur hors scope (sélection contact `is_supplier` existant).
- **L5** — `GET .../source-document` 404 pour une facture créée directement 12-2 (sans import). DC-d1 gère gracieusement.
- **L6** — pdfium natif non sandboxé (segfault PDF malformé tue le process). Documenté manuel admin.
- **L8** — inbox non partitionné par tenant (Issue #199, v0.4-milestone). Documenté manuel admin.

### References
- [Source: 12-5-import-repertoire-factures.md] — AC8 (frontend) + AC9 (doc/tests).
- [Source: 12-5c-service-import-completion.md] — contrats des 6 endpoints, codes d'erreur, statuts.
- [Source: frontend ground-truth] — `supplier-invoices/+page.svelte`, `api-client.ts`, `i18n.svelte.ts`, `(app)/+layout.svelte`, `users/+page.ts`, `reports.api.ts`.

## Change Log

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
