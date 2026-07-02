# Story 12.5d: Frontend import + complétion + justificatif & documentation

Status: done

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
  - `POST /api/v1/imported-supplier-invoices/{id}/discard` → **204**. Rejets : **404** `IMPORTED_INVOICE_NOT_FOUND` (row absente/hors-company), **409** `IMPORT_NOT_PENDING_COMPLETION` (`details.currentStatus` — déjà complétée/écartée, race deux onglets).
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
   - **Toute autre erreur (M1)** : 500 (catastrophe DB/IO, ex. segfault pdfium L6), 401 (cookie expiré pendant un import long), réseau → `notifyError(err.message || 'Erreur inattendue lors de l\'import')`. Le `catch` n'avale jamais une erreur silencieusement : `if (isApiError(e) && e.code === 'INBOX_IMPORT_ALREADY_RUNNING') { ... } else { notifyError(...) }`.
   - Bouton désactivé + spinner pendant l'import (le run est synchrone, peut durer plusieurs secondes — pdfium) ; **réactivé dans un `finally`** (succès comme échec) pour permettre une nouvelle tentative.
   - Après import, la **liste `to_complete` est rechargée** (les nouvelles importées apparaissent).
   - **Map `errorCode → libellé FR**** (clés `imported-supplier-invoices-error-<code-kebab>`, dans le fichier de route — cf. AC8) couvrant les 10 codes : `UNSUPPORTED_FILE_TYPE`, `FILE_TOO_LARGE`, `SYMLINK_REJECTED`, `DUPLICATE`, `NO_QR_CODE_FOUND`, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR`, `FILE_READ_ERROR`, `FIELD_TOO_LONG`. Code inconnu → fallback générique « Échec de l'import (code) ».

### Liste & complétion

3. **Liste `to_complete`** (sur la même page ou onglet) : **chargée au montage** (`onMount` → `listImported('to_complete')`, état de chargement spinner/skeleton ; liste vide → message explicite « Aucune facture à compléter »). Pour chaque importée, afficher les coordonnées QR (créancier, IBAN, montant, référence, devise) + 3 actions : **Compléter**, **Écarter**, **Voir le justificatif**.
   - **Voir le justificatif** : download via `GET /imported-supplier-invoices/{id}/source-document` en **Pattern A** (`apiClient.getBlob` → blob → `triggerDownload`, gère 401-refresh — Pattern B `<a download>` ne gère PAS le refresh sur cookie httpOnly). **Erreurs (M4/L3)** : 404 `IMPORTED_INVOICE_NOT_FOUND` (race : row disparue depuis le chargement) → `notifyWarning` ; 410 `SOURCE_DOCUMENT_GONE` (fichier disque absent, L1/F7) → `notifyWarning` « fichier non restauré » ; réseau → `notifyError`.

4. **Formulaire de complétion** (**Dialog**, pattern création de `supplier-invoices/+page.svelte` — DC L6) :
   - **Pré-affichage lecture seule** des coordonnées QR (guide l'utilisateur) : nom créancier, IBAN, **montant cible** (`staging.amount` — la somme TTC des lignes devra l'égaler exactement), référence. **Avertissement sous-centime (DC-d3)** : si `staging.amount` a plus de 2 décimales significatives → bandeau « montant QR à sous-centimes, impossible à atteindre — recommandation : Écarter ».
   - **Champs éditables** : sélection fournisseur via **`<select>` simple** chargé `listContacts({ isSupplier: true, limit: 200 })` (PAS `ContactPicker` — qui n'expose pas de filtre `isSupplier` et vit dans `$lib/components/invoices/` ; le `<select>` est le pattern ground-truth de `supplier-invoices/+page.svelte`, plus bas risque — C5/A2), `invoiceDate` (requis), `supplierInvoiceNumber?`, `dueDate?`, **lignes** (description, quantité, PU HT, taux TVA via `listVatRates`, compte de charge `Expense` actif via `fetchAccounts()` filtré). Décimaux saisis en `type="text" inputmode="decimal"`, calculs via **`big.js`** (`computeLineTotal`), affichage via `formatSwissAmount`.
   - **Total TTC live** affiché (F-OPUS-1) — **calculé en parité EXACTE avec le backend** : `Σ TTC = Σ(quantity×unit_price pleine précision) + Σ lineVatAmount(line_total, vat_rate)` en **réutilisant le helper `lineVatAmount` de `frontend/src/lib/features/journal-entries/vat-purchase.ts:36`** (documenté parité exacte avec `kesh_core::accounting::vat::line_vat_amount`, arrondi centime HALF_UP par ligne). **NE PAS** utiliser `computeLineTotal` seul (= HT, sans TVA → mismatch sur toute facture taxée). L'indicateur « Σ TTC ≠ cible » compare le **`Σ TTC` pleine précision** (pas l'affichage arrondi 2 déc.) à `staging.amount` pour éviter un faux-vert (le backend exige l'égalité exacte pleine précision F-OPUS-2 12-5c). **Le bouton Valider reste actif vis-à-vis de la réconciliation montant** (autorité = backend `AMOUNT_MISMATCH`, jamais bloquée côté UI — DC-d3).
   - **Validation client structurelle avant soumission (M3)** : bouton Valider **désactivé uniquement** si (a) aucun fournisseur sélectionné, (b) 0 ligne, ou (c) une ligne a `description` vide (NOT NULL + CHECK DB) — indication visuelle sur le champ fautif. **La réconciliation montant n'est JAMAIS une condition de désactivation** (DC-d3 : actif vis-à-vis du montant, désactivé seulement sur invalidité structurelle).
   - Soumission → `completeImport(id, body)` → succès `notifySuccess` (option : lien toast « Voir la facture créée » → `/supplier-invoices/{newId}`, id de la `SupplierInvoiceResponse`) + **retrait de la ligne complétée de la liste, l'utilisateur RESTE sur le worklist d'import** (F-OPUS-2 : PAS de redirection automatique — l'import est un flux **batch** ; éjecter vers le détail après chaque complétion ferait perdre la place dans les N-1 factures restantes).
   - **Mapping rejets → messages UX traduits** (via `isApiError` + `err.code`) : `CURRENCY_NOT_SUPPORTED` (« Devise non supportée (CHF uniquement) »), `IBAN_REFERENCE_MISMATCH` (« Incohérence IBAN/référence QRR »), `AMOUNT_MISMATCH` (« Le total des lignes (`details.actual`) ≠ montant QR (`details.expected`) »), `IMPORT_NOT_PENDING_COMPLETION` (« Déjà complétée/écartée »), `IMPORTED_INVOICE_NOT_FOUND` (« Facture importée introuvable »), `FISCAL_YEAR_INVALID` (« Aucun exercice ouvert pour cette date »), fallback `err.message`.

5. **Écarter** : confirmation (Dialog) → `discardImport(id)` (204) → retrait de la liste + `notifySuccess`. Texte : « Le fichier justificatif reste conservé. » **Erreurs (M5/A1)** : `catch` mappant 409 `IMPORT_NOT_PENDING_COMPLETION` (« Déjà complétée/écartée par une autre session ») + 404 `IMPORTED_INVOICE_NOT_FOUND` (« Facture importée introuvable ») + fallback `notifyError(err.message || 'Impossible d\'écarter')` ; rechargement de la liste sur conflit pour resynchroniser.

### Justificatif sur le détail facture

6. **Détail facture fournisseur** (`/(app)/supplier-invoices/[id]/+page.svelte`) : lien **« Voir la facture d'origine »** → download via `GET /supplier-invoices/{id}/source-document` (**Pattern A** `getBlob` + ancre éphémère, gère 401-refresh). **DC-d1 FIGÉ** : le lien est **toujours rendu** (pas de champ backend `hasSourceDocument` — 12-5c `done`, cf. DC-d1). Au clic : **404 `SOURCE_DOCUMENT_NOT_FOUND`** (facture directe 12-2, L5) → **`notifyInfo`** « Cette facture n'a pas de justificatif importé » ; **410 `SOURCE_DOCUMENT_GONE`** (fichier non restauré, L1/F7) → **`notifyWarning`** « Justificatif non restauré » ; réseau → `notifyError`. Jamais d'erreur bloquante.

### Navigation & i18n

7. **Sidebar** (`(app)/+layout.svelte`) : entrée vers `/supplier-invoices/import` (groupe « Quotidien », pattern existant — visible tous rôles côté sidebar, accès réel gardé Comptable+ backend + `+page.ts`). `data-testid` auto `nav-link-supplier-invoices-import`.

8. **i18n** : 100% des chaînes via `i18nMsg('clé', 'fallback FR')` (système runtime custom, `i18nMsg` de `i18n.svelte.ts` — PAS de `.ftl`/paraglide). **Préfixe unique `imported-supplier-invoices-`** pour TOUTES les clés (y compris la map errorCode → `imported-supplier-invoices-error-<code-kebab>`). **Garde-fou `lint-i18n-ownership` (C3)** : le linter (`frontend/scripts/lint-i18n-ownership.js`) exige que les fichiers d'un feature folder `src/lib/features/X/` n'utilisent QUE des clés préfixées `X-`. Donc : (a) le **feature module** `imported-supplier-invoices/` ne contient QUE types + API (AUCUN `i18nMsg`) ; (b) tous les appels `i18nMsg(...)` ET la map errorCode vivent dans les **fichiers de route** `src/routes/(app)/supplier-invoices/import/+page.svelte` (les routes ne sont pas soumises au check d'ownership feature, mais le préfixe unique évite toute ambiguïté). Les valeurs FR sont le **fallback inline** — qui EST le système pour les chaînes de page (F-OPUS-4 : `i18nMsg(key, fallback)` renvoie `_messages[key] || fallback` ; précédent 12-2/12-3 : ~1 clé sur 37 seulement est enregistrée dans le catalogue FTL `crates/kesh-i18n/locales/*/messages.ftl`, le reste vit en fallback inline). **AUCUNE tâche FTL/backend i18n requise** pour 12-5d (le dropdown DE/IT/EN est désactivé, Story 2.1) — ne PAS enregistrer les nouvelles clés dans les `.ftl` (divergerait du précédent). Le fallback FR provisoire suffit ; les locales DE/IT/EN viendront avec l'activation du sélecteur (v0.2+).

### Documentation

9. **`.env.example`** : nouvelle section « Import de factures depuis un dossier (#194) » avec les 5 vars + défauts + note volumes : `KESH_DOCUMENTS_DIR` (`/data/documents`), `KESH_INBOX_DIR` (`/data/inbox`), `KESH_INBOX_MAX_FILE_BYTES` (26214400 = 25 Mo), `KESH_INBOX_MAX_FILES_PER_RUN` (200), `KESH_INBOX_MAX_PDF_PAGES` (20). Ajout après la section `KESH_ADMIN_EXPORT_INMEM_MB`.

10. **`docker-compose.yml` + `docker-compose.prod.yml`** : dans `kesh-api`, ajouter `environment` (`KESH_DOCUMENTS_DIR`/`KESH_INBOX_DIR` + 3 MAX_) + `volumes` (`/data/inbox` + `/data/documents`). Prod : volumes **nommés** (`kesh-documents:`, `kesh-inbox:`) déclarés dans la section `volumes:` (persistance). Dev : bind mounts host (`./inbox:/data/inbox`, `./documents:/data/documents`) acceptables. Cohérent avec le mapping du manuel admin.

11. **`CHANGELOG.md`** `[Non publié]` → `### Ajouté` : bullet « Import de factures fournisseurs depuis un dossier (#194) » en termes utilisateur (bouton « Importer le dossier », détection Swiss QR, staging « à compléter », justificatif récupérable).

12. **`README.md`** : section « Fonctionnalités » — retirer/mettre à jour les mentions « *(scan QR-facture à venir)* » (lignes ~31-32) ; ajouter l'import-répertoire. « Feuille de route » — refléter l'import dans la ligne v0.x correspondante (✓ / 🚧).

12-bis. **`website/` GitHub Pages (C2)** — la checklist pré-push CLAUDE.md impose de vérifier `website/index.html` + `website/roadmap.html` à chaque push de PR. **`website/roadmap.html`** (~l.241) décrit encore E12 de façon périmée (« Credits & Payments (pain.001)... » antérieur à 12-1/12-2/12-5) : **mettre à jour** la description E12 pour refléter avoirs + factures fournisseurs + **import répertoire**. Vérifier `website/index.html` : pas d'over-claim/under-claim sur l'import. (Le déploiement Pages est auto sur push `main` touchant `website/**`.)

13. **`docs/manual/fr/admin-manual.tex`** : (a) entrées des 5 vars dans la table « Variables optionnelles » (~l.659) ; (b) nouvelle sous-section `\subsection{Import de factures depuis un dossier}` : configuration `KESH_INBOX_DIR` sur NAS Synology (dossier partagé), caveat canonicalisation symlink `/data`→`/volume1`, persistance `KESH_DOCUMENTS_DIR`, réglage `KESH_INBOX_MAX_PDF_PAGES`, limitation **L6** (pdfium segfault PDF malformé tue le process) + **L8** (inbox non partitionné par tenant, Issue #199). **Régénérer le PDF** (`make fr` dans `docs/manual/` ; **NE PAS** bumper `\keshVersion` — gate release).

### Qualité

14. **E2E Playwright** `frontend/tests/e2e/inbox-import.spec.ts` (pattern `supplier-invoices.spec.ts`) — flux **UI**, **seed via DC-d4** (fixture PNG déposée dans `KESH_INBOX_DIR` du serveur de test puis import réel ; PNG→rxing, pas de pdfium) :
    - login → navigation vers `/supplier-invoices/import` → clic « Importer le dossier » → **rapport rendu** (inbox vide → 0 créées sans crash ; avec fixture → ≥1 créée).
    - après import de la fixture, la **liste `to_complete`** affiche l'importée → ouverture du formulaire de complétion → remplissage (fournisseur seedé, date, 1 ligne au montant cible) → soumission → statut passe `completed` / disparaît de la liste.
    - **écarter** une importée → disparaît + `notifySuccess`.
    - **« Voir le justificatif »** sur une importée → assert download déclenché (le fichier vient d'être archivé par l'import réel → pas de 410).
    - **Validation client** : bouton Valider désactivé si pas de fournisseur / 0 ligne (assert).
    - **Fallback (DC-d4)** : si le harness CI ne peut pas partager `KESH_INBOX_DIR` (chemin non inscriptible par le runner), marquer les scénarios dépendant de l'import réel `test.skip(reason explicite)` ; le flux complétion reste couvert par Vitest composant + 12-5c intégration. NE PAS étendre le preset Rust `seedTestState` (modif backend).
    - Pré-requis E2E : `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (Ubuntu 26.04+, cf. memory) + MariaDB + seed CI + `KESH_INBOX_DIR` exporté vers un dossier inscriptible (DC-d4) + fixture `frontend/tests/e2e/fixtures/spc_*.png` pré-commitée (générable via le writer rxing, cf. `inbox_import_e2e.rs` 12-5c `qr_png_from_payload`).

15. **Quality gate Test Locally First — exit code vérifié** :
    - Frontend : `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build`.
    - E2E : `cd frontend && npm run test:e2e` (avec override Playwright).
    - Backend : si aucun fichier Rust touché (12-5d = frontend+doc), la suite backend reste verte par construction ; lancer `cargo build --workspace` par sécurité si un type partagé bouge. **PAS** `cargo test | grep` (exit code masqué, cf. `feedback_cargo_test_pipe_masks_exit`).
    - 0 régression `npm run test:unit` (333 vitest baseline) + `svelte-check` 0 erreur.

## Décisions de conception (DC) — FIGÉES (validate Pass 1)

- **DC-d1 [✅ FIGÉ — always-render + 404/410 gracieux, AUCUNE modif backend]** : le lien « Voir la facture d'origine » est **toujours rendu** sur le détail facture fournisseur ; au clic, un 404 `SOURCE_DOCUMENT_NOT_FOUND` (facture directe 12-2 sans justificatif, L5) → `notifyInfo` « Cette facture n'a pas de justificatif importé » ; un 410 `SOURCE_DOCUMENT_GONE` → `notifyWarning` « Justificatif non restauré ». **L'alternative `hasSourceDocument: bool` est REJETÉE** : elle rouvrirait 12-5c (status `done`, review convergé, 1634 tests verts) → violation de la frontière frontend-only + contrat STOP-modif-architecturale. UX accepté v0.4 : sur les factures manuelles (majorité en début de vie PME), le lien clique « à vide » avec un `notifyInfo` non bloquant. *(⚠️ Gate Guy : si tu préfères le rendu conditionnel via champ backend, ce serait une story de suivi sur 12-2/12-5c, pas 12-5d.)*
- **DC-d2 [✅ FIGÉ — `/supplier-invoices/import`]** : route de l'écran import+liste, sous le domaine existant `supplier-invoices/`.
- **DC-d3 [✅ FIGÉ — bouton Valider actif vis-à-vis du montant, désactivé seulement sur invalidité structurelle]** : le formulaire **pré-affiche le montant QR cible** et indique visuellement l'écart `Σ TTC ≠ cible`. **Le bouton Valider n'est JAMAIS désactivé par un écart de montant** (la règle d'égalité centime-exacte F-OPUS-2 est tranchée par le backend `AMOUNT_MISMATCH`, jamais dupliquée côté UI) ; il est désactivé **uniquement** sur invalidité structurelle (pas de fournisseur / 0 ligne / `description` vide — M3). `Σ TTC` est calculé en parité backend via `lineVatAmount` (F-OPUS-1, cf. AC4). **Cas sous-centime** (QR tiers non conforme SIX 2.2, `amount` à >2 décimales non nulles) : si `staging.amount` a plus de 2 décimales significatives, afficher un **avertissement distinct** « Le montant QR contient des sous-centimes (X) — impossible à atteindre par des lignes centime-exactes ; recommandation : Écarter cette facture ».
- **DC-d4 [✅ FIGÉ — E2E round-trip via fixture PNG dans l'inbox, AUCUNE modif backend]** : le test E2E crée la row staging en **exécutant le vrai import** — le setup E2E dépose un **fichier fixture QR PNG pré-commité** (`frontend/tests/e2e/fixtures/spc_*.png`, image → `rxing`, **pas de pdfium**) dans le `KESH_INBOX_DIR` du serveur de test (écriture Node `fs`, même hôte que le serveur en CI), puis clique « Importer ». **Mécanisme concret** : le serveur backend E2E tourne via `from_env` (PAS `from_fields_for_test` dont le défaut `/tmp/kesh-inbox-test` ne concerne que les tests unitaires/intégration) → le harness E2E (script de démarrage / `webServer` Playwright) **doit exporter `KESH_INBOX_DIR`** vers un dossier inscriptible (ex. `/tmp/kesh-e2e-inbox`), et le test écrit la fixture dans ce même chemin (lu via `process.env.KESH_INBOX_DIR`). Le pipeline d'import reste couvert par les 16 tests d'intégration 12-5c — l'E2E valide les **flux UI**. **Fallback documenté** : si le harness CI ne peut pas exposer/partager le chemin inbox, le scénario « compléter une importée » est `test.skip(reason)` explicite et la complétion reste couverte par Vitest (composant) + 12-5c. **PAS** d'extension du preset Rust `seedTestState` (= modif backend, rejetée).
- **DC-d5 [✅ FIGÉ — pas de redirection, worklist persistant]** : la liste des importées est **chargée au montage** (`onMount` → `listImported('to_complete')`) et la ligne traitée est **retirée** (ou la liste rechargée) après complétion/écart — **l'utilisateur reste sur l'écran `/supplier-invoices/import`** (cohérent F-OPUS-2 : flux batch, pas de redirection auto vers le détail). Pas de store global réactif inter-pages pour v0.4.

## Tasks / Subtasks

- [x] **T-d1 — Feature module** (AC1) : `frontend/src/lib/features/imported-supplier-invoices/{imported-supplier-invoices.types.ts, imported-supplier-invoices.api.ts}` — types miroir (décimaux `string`) + 6 wrappers `apiClient`. AUCUN `i18nMsg` ici (lint-i18n-ownership).
- [x] **T-d2 — Écran import + rapport** (AC2) : route `(app)/supplier-invoices/import/+page.svelte` + `+page.ts` (guard Comptable+) — bouton « Importer le dossier », rapport `{accepted, failed, warnings}`, map errorCode traduite, 409 discriminé, fallback `notifyError`, spinner+finally.
- [x] **T-d3 — Liste `to_complete` + download pré-complétion** (AC3) : `onMount` `listImported('to_complete')`, état vide, 3 actions par ligne ; « Voir le justificatif » Pattern A + erreurs 404/410.
- [x] **T-d4 — Formulaire complétion** (AC4, DC-d3) : Dialog, pré-affichage QR lecture seule + avertissement sous-centime, `<select>` fournisseur `isSupplier`, lignes (desc/qty/PU/TVA/compte), **Σ TTC live via `lineVatAmount`** comparé pleine précision, validation structurelle (bouton désactivé), mapping rejets 400/409/404, pas de redirection (reste worklist).
- [x] **T-d5 — Écarter** (AC5) : Dialog confirm → `discardImport` 204 + retrait ; erreurs 409/404 + fallback.
- [x] **T-d6 — Lien justificatif détail** (AC6, DC-d1) : `supplier-invoices/[id]/+page.svelte` lien « Voir la facture d'origine » always-render Pattern A, 404→notifyInfo / 410→notifyWarning.
- [x] **T-d7 — Sidebar + i18n** (AC7/AC8) : entrée nav `quotidien` vers `/supplier-invoices/import` ; toutes clés `imported-supplier-invoices-*` (fallback FR inline, dans les fichiers de route), 0 tâche FTL.
- [x] **T-d8 — Doc** (AC9-13 + AC12-bis) : `.env.example` (5 vars), `docker-compose.yml` + `docker-compose.prod.yml` (volumes+env), `CHANGELOG.md`, `README.md`, `website/roadmap.html`+`index.html`, `docs/manual/fr/admin-manual.tex` (+ régénérer PDF `make fr`, NE PAS bumper `\keshVersion`).
- [x] **T-d9 — E2E + quality gate** (AC14/AC15) : `frontend/tests/e2e/inbox-import.spec.ts` + fixture `frontend/tests/e2e/fixtures/spc_*.png` + câblage `KESH_INBOX_DIR` (DC-d4). Gate exit-code : `npm run check && lint-i18n-ownership && test:unit && build` + `test:e2e`.

## Dev Notes

### Ground-truth frontend (vérifié 2026-06-30, 2 agents Explore)

**Routing & guards** :
- Pages authentifiées sous `frontend/src/routes/(app)/`. `supplier-invoices/+page.svelte` (liste) + `supplier-invoices/[id]/+page.svelte` (détail) **existent**.
- Guard rôle = `+page.ts` `load()` : `import { browser } from '$app/environment'; import { redirect } from '@sveltejs/kit'; import { authState } from '$lib/app/stores/auth.svelte';` + `export const ssr = false;` + `if (browser && role !== 'Admin' && role !== 'Comptable') throw redirect(302, '/');`. (Pattern Admin-only dans `users/+page.ts`.) `Role = 'Admin' | 'Comptable' | 'Consultation'` (`$lib/shared/types/user.ts`).
- `(app)/+layout.ts` garde déjà `isAuthenticated`.

**API client** (`$lib/shared/utils/api-client.ts`) : `apiClient.{get,getBlob,post,postFormData,put,patch,delete}`. Auth = cookie httpOnly (`credentials:'include'`, refresh transparent sur 401). Erreurs = `{error:{code,message,details}}` → `ApiError{code,message,details?,status}` ; `isApiError(err)` type guard. Exemple POST : `apiClient.post('/api/v1/supplier-invoices', req)` dans `supplier-invoices.api.ts`.

**Formulaire lignes** (`supplier-invoices/+page.svelte`) : `select` fournisseur (`suppliers` chargés via `listContacts({ isSupplier:true, limit:200 })`), grille `grid-cols-12` par ligne (description col-4, quantity, unitPrice, vatRate `select` via `listVatRates()`, expenseAccount `select` via `fetchAccounts()` filtré `accountType==='Expense' && active`). Lignes init en **strings** (`quantity:'1', unitPrice:'', vatRate:'0', expenseAccountId:0`). `onMount` + `Promise.all` pour charger contacts/comptes/taux.

**Décimaux** : strings backend. Calc HT via `big.js` (`computeLineTotal(qty, unitPrice)` de **`$lib/features/invoices/invoice-helpers.ts`** → `new Big(qty).times(unitPrice).round(4, ROUND_HALF_UP).toFixed(4)`). **TVA par ligne (parité backend, F-OPUS-1)** : `lineVatAmount(line_total, vat_rate)` de **`frontend/src/lib/features/journal-entries/vat-purchase.ts:36`** — documenté parité EXACTE avec `kesh_core::accounting::vat::line_vat_amount` (arrondi centime HALF_UP). Le `Σ TTC live` du formulaire de complétion = `Σ computeLineTotal` + `Σ lineVatAmount(line_total, rate)`, comparé pleine précision à `staging.amount`. Affichage `formatSwissAmount(new Big(d))` → `1'234.50`. Input `type="text" inputmode="decimal"`, regex `^\d{1,15}([.,]\d{1,4})?$`.

**i18n** (`$lib/shared/utils/i18n.svelte.ts`) : `i18nMsg(key, fallbackFR, args?)`, substitution `{ $var }`. Messages servis par `/api/v1/i18n/messages` selon locale. FR = fallback inline. DE/IT/EN désactivés (dropdown disabled, Story 2.1).

**Download authentifié** : Pattern A (recommandé, gère 401) `apiClient.getBlob(url)` → `blob()` → ancre éphémère `triggerDownload` (cf. `exports.api.ts` / `reports.api.ts`). `parseContentDispositionFilename` existe dans **`exports.api.ts:52`** (et `admin-backup.api.ts`), **PAS** dans `reports.api.ts` (qui code le filename en dur) — importer/recopier depuis `exports.api.ts` pour extraire le nom du header `Content-Disposition`. Pattern B (simple) ancre `<a href={url} download={name}>` (cf. `payment-batches/[id]/+page.svelte` pain.001) — ne gère PAS le 401-refresh.

**Sidebar** (`(app)/+layout.svelte`) : `navGroups` (quotidien/mensuel/administration). `supplier-invoices` + `payment-batches` déjà dans « quotidien » (non restreints sidebar). `isAdmin = $derived(role==='Admin')`. Ajouter l'entrée import dans `quotidien.items`. `navTestid(href)` → `nav-link-...`.

**Composants UI** (`$lib/components/ui/`) : shadcn/bits-ui — `Button` (variant default/outline/ghost/destructive, size sm), `Input`, `Dialog.{Root,Content,Header,Title,Description,Footer,Trigger}`, `Select` (mais pages utilisent `<select>` brut). `notify.ts` : `notifySuccess/Error/Warning/Info`. Icônes `@lucide/svelte`. `ContactPicker` (à **`$lib/components/invoices/ContactPicker.svelte`** — PAS `ui/` ; combobox ARIA, props `selected/onSelect/placeholder`, n'expose PAS de filtre `isSupplier`) → **NE PAS l'utiliser** ; garder le `<select>` simple chargé `listContacts({ isSupplier:true })` (pattern ground-truth `supplier-invoices/+page.svelte`, plus bas risque — DC C5). `BankImportUpload.svelte` = analogue le plus proche pour « déclencher + afficher rapport batch ».

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

### Validate Pass 1 (Sonnet 4.6, 3 couches : fidélité ground-truth / complétude AC / conventions-faisabilité), 2026-06-30
Trend > LOW : **~9** (2 HIGH + 7 MEDIUM). Layer fidélité : spec **fidèle** sur TOUS les contrats critiques (6 endpoints, paths, HTTP status, error codes, `CompleteImportRequest` shape camelCase, status strings, `AMOUNT_MISMATCH details.expected/actual`, champs entité camelCase, compteurs doc déjà faits 12-5b → scope « hors 12-5d » légitime). Patches :
- **H1** — DC-d1 figé : « always-render + 404/410 gracieux », alternative backend `hasSourceDocument` **rejetée** (rouvrirait 12-5c `done` → viole frontière frontend-only). Flag gate Guy.
- **H2/C1** — DC-d4 figé : E2E round-trip via fixture PNG déposée dans `KESH_INBOX_DIR` du serveur de test (PNG→rxing, pas pdfium, pas de modif backend) ; fallback `test.skip` documenté si harness ne partage pas le chemin.
- **DC-d2/d3/d5 figés** : route `/supplier-invoices/import` ; bouton Valider toujours actif (autorité backend) + avertissement sous-centime → recommandation Écarter ; liste chargée `onMount` + rechargée.
- **M1** — fallback `notifyError` pour toute erreur non-409 de `/inbox-import` (500/401/réseau) + bouton réactivé en `finally`.
- **M3** — validation client formulaire complétion (fournisseur requis, ≥1 ligne, `description` non vide) → bouton désactivé.
- **M4/L3** — error-handling download pré-complétion (404/410/réseau) + Pattern A obligatoire.
- **M5/A1** — error-handling discard (404 `IMPORTED_INVOICE_NOT_FOUND` / 409 `IMPORT_NOT_PENDING_COMPLETION` + fallback).
- **C2** — ajout AC12-bis : `website/roadmap.html` (description E12 périmée) + `website/index.html` (checklist pré-push CLAUDE.md).
- **C3** — i18n : préfixe unique `imported-supplier-invoices-*`, `i18nMsg` dans les fichiers de route (PAS le feature module) → `lint-i18n-ownership` vert.
- **LOW** — chemin `ContactPicker` (`invoices/` pas `ui/`, + pin `<select>`), `parseContentDispositionFilename` dans `exports.api.ts` (pas `reports.api.ts`), notifyInfo/Warning mapping, Dialog (pas inline), redirect post-complétion recommandée, E2E « Voir le justificatif ».

Pass 2 : Haiku 4.5 (contexte frais).

### Validate Pass 2 (Haiku 4.5, 2 couches : fidélité+conventions / complétude+cohérence), 2026-06-30
Trend > LOW : **1 MEDIUM** (après dismiss). Layer fidélité : spec **100% fidèle** confirmée (6 endpoints/shapes/codes, scripts npm, règle `lint-i18n-ownership` `keyBelongsToFeature` vérifiée, fichiers cités tous présents). Layer complétude : **READY-FOR-DEV, 0 CRITICAL/HIGH**, toutes AC umbrella couvertes, tous DC figés, 0 contradiction. Patches :
- **M (réf L2 erronée + scope redirect)** — la note « (L2, à implémenter) » référençait une limitation inexistante en 12-5d ; clarifié « redirection vers `/supplier-invoices/{newId}` (UX standard, à implémenter) ».
- **LOW (DC-d4 chemin E2E)** — Haiku confirme la faisabilité + précise : serveur E2E via `from_env` (≠ défaut unit-test `/tmp/kesh-inbox-test`) → le harness doit exporter `KESH_INBOX_DIR` vers un dossier inscriptible (ex. `/tmp/kesh-e2e-inbox`), documenté dans DC-d4.
- **LOW (chemin fixtures)** — corrigé `frontend/tests/e2e/fixtures/` (pas `frontend/tests/fixtures/`, vérifié ground-truth).
- **LOW (note « (L4) » AC14)** — clarifiée.

**Faux-positif Haiku réfuté (garde-fou CLAUDE.md)** : « 🔴 CRITICAL — .env.example/docker-compose/CHANGELOG/README/manuel/website NON IMPLÉMENTÉS ». Haiku a confondu **travail prescrit par la spec (AC9-13 + AC12-bis)** avec **défaut d'implémentation** : ces items SONT les tâches que le dev doit faire, donc absents au moment de la spec — la spec les liste correctement en scope. Réfuté : aucune action (ce sont des AC, pas des trous).

Pass 3 : Opus 4.8 (axe architecture/UX/cohérence cross-AC).

### Validate Pass 3 (Opus 4.8, deep architectural / cross-artefact / parité numérique), 2026-06-30
Trend > LOW : **2 MEDIUM + 2 LOW** (catches que Sonnet+Haiku ne tracent pas — cross-artefact spec↔Rust↔TS, contradiction cross-AC, parité numérique). Vérifiés sains en deep-trace : DC-d1 (page route `supplier-invoices/[id]`, pas de composant partagé, 0 download préexistant → intégration propre), frontière scope (0 modif 12-5c), parcours e2e (import lent : spinner+finally+409 discriminé+M1 ok). Patches :
- **F-OPUS-1 (MEDIUM↗)** — le « Total TTC live » : seul helper nommé = `computeLineTotal` (**HT seul**) → indicateur mismatch sur toute facture taxée + risque faux-vert (front arrondit 4 déc, backend exige égalité pleine précision). Patch : calcul TTC via `lineVatAmount` (`journal-entries/vat-purchase.ts:36`, parité EXACTE `line_vat_amount` confirmée ground-truth), comparé pleine précision à `staging.amount`.
- **F-OPUS-2 (MEDIUM)** — **contradiction** AC4 « redirection /supplier-invoices/{newId} » vs DC-d5 « recharger la liste » (mutuellement exclusifs) + la redirection casse le worklist batch. Patch : **suppression de la redirection** ; l'utilisateur reste sur le worklist, la ligne complétée est retirée (option toast « Voir la facture créée »). DC-d5 mis à jour.
- **F-OPUS-3 (LOW)** — « bouton TOUJOURS actif » (DC-d3) vs « désactivé si... » (M3) : reformulé « actif vis-à-vis du montant, désactivé seulement sur invalidité structurelle ».
- **F-OPUS-4 (LOW)** — AC8 « DE/IT/EN gérées backend » trompeur : Opus trace que l'inline fallback EST le système (précédent 12-2/12-3 : ~1/37 clés FTL) → la spec a raison de ne pas ajouter de tâche FTL ; rationale corrigé (AUCUNE tâche FTL, dropdown DE/IT/EN désactivé).

Pass 4 : Sonnet 4.6 (convergence).

### Validate Pass 4 (Sonnet 4.6, convergence), 2026-06-30 — CONVERGÉ
**Verdict : PRÊT POUR DEV — 0 finding CRITICAL/HIGH/MEDIUM.** Les 4 patches Opus vérifiés cohérents ground-truth : F-OPUS-1 (`lineVatAmount(ht, ratePercent)` confirmé `vat-purchase.ts:36`, usage cohérent), F-OPUS-2 (0 résidu « redirection » contradictoire — seuls subsistent le guard SvelteKit `redirect(302)` et le Change Log historique), F-OPUS-3 (DC-d3 + AC4/M3 cohérents), F-OPUS-4 (AC8 sans tâche FTL, cohérent Dev Notes). 0 DC `à ratifier` (les 5 `[✅ FIGÉ]` ; seul `⚠️ Gate Guy` DC-d1 = gate architectural délibéré, pas un défaut). Tous les helpers load-bearing confirmés (`parseContentDispositionFilename` `exports.api.ts:52`, `listContacts/listVatRates/fetchAccounts`, `apiClient.getBlob`, `notify*`, `lint-i18n-ownership`). 0 contradiction AC↔AC / AC↔DC. 2 LOW patchés (chemin import `computeLineTotal`, shorthand Change Log).

### Dev-story — implémentation (Opus 4.8, 2026-06-30)
Run unique T-d1→T-d9. **Quality gate frontend exit-code vérifié** : `npm run check` (0 err), `npm run lint-i18n-ownership` (PASS), `npm run test:unit` (340/340), `npm run build` (OK). **E2E `inbox-import.spec.ts` exécuté EN RÉEL 2/2** contre un backend monté (DB isolée `kesh_e2e`, `KESH_INBOX_DIR=/tmp/kesh-e2e-inbox`) — round-trip complet import→liste→download→complétion validé. 1 bug attrapé en E2E : `each_key_duplicate` sur `{#each vatRates}` (seed 4 taux catégorie `custom`) → clé `(r.id)`. Aucun fichier Rust touché → suite backend inchangée. Status `review`. Prochaine : `bmad-code-review 12-5d` (LLM ≠ Opus).

### Code-review Pass 1 (Sonnet 4.6, 3 couches // Blind/Edge/Acceptance), 2026-06-30
Trend > LOW : **1 HIGH + 8 MEDIUM**. Acceptance Auditor : implémentation **fidèle** (tous AC/DC respectés, modulo findings ci-dessous). Patches :
- **EC1/BH2 (HIGH)** — `structurallyInvalid()` ne vérifiait pas `expenseAccountId===0` → bouton actif sans compte → erreur backend brute. Patch : validation structurelle complète (compte ≠ 0 + `isValidAmount(quantity/unitPrice)` + `invoiceDate` non vide + description) ; bouton désactivé sur invalidité structurelle uniquement (jamais le montant, DC-d3).
- **EC2/BH1 (MEDIUM)** — `runImport` : un échec de `reloadList` APRÈS un import réussi effaçait le rapport (`report=null`). Patch : try/catch séparés (le rapport survit, warning si reload échoue).
- **EC3 (MEDIUM)** — état de formulaire partagé entre lignes : compléter rowA pendant que rowB est ouvert. Patch : `if (completingId === id)` au succès + boutons Compléter/Écarter `disabled={saving}`.
- **EC4/EC5 (MEDIUM)** — quantité/PU non-numériques ou date vidée → erreur backend brute. Patch : couverts par la validation structurelle (`isValidAmount` + date).
- **EC6/BH7 (MEDIUM)** — `await reloadList()` dans un `catch` de `discard` → rejection non gérée. Patch : helper `safeReloadList()` (catch interne).
- **M1 (MEDIUM)** — test E2E `discard` manquant. Patch : 2e fixture `spc_e2e_discard.png` (montant 55.00, hash distinct) + test « écarter → disparaît » (dialog accept).
- **M2 (MEDIUM)** — assert bouton désactivé manquant. Patch : `expect(imported-complete-submit).toBeDisabled()` sans fournisseur dans le round-trip.
- **M3 (MEDIUM)** — `docker-compose.prod.yml` bind mounts vs spec « volumes nommés ». **Déviation ASSUMÉE + justifiée** : sur NAS l'utilisateur DOIT déposer ses factures dans l'inbox via File Station → un volume nommé (stockage interne Docker) rendrait l'inbox inaccessible. Les bind mounts `./inbox`/`./documents` (calqués sur `./log`, scope Hyper Backup) sont **nécessaires à l'usabilité**. Dev compose garde des volumes nommés (acceptable, cf. spec). Documenté ici.
- **LOW patchés** : clés `{#each report.warnings}`/`{#each report.failed}` → index (anti `each_key_duplicate` — même classe de bug que celui attrapé en dev sur vatRates) ; `notifyInfo` inutilisé retiré (page import) ; L1 rename 4 clés détail → `imported-supplier-invoices-*` (AC8) ; boutons `disabled`.
- **LOW laissés (documentés)** : `{#each fLines as line, idx (idx)}` (index UNIQUE → pas de crash, seul recyclage DOM cosmétique, parité ground-truth `supplier-invoices`) ; fallback filename sans extension (le backend envoie toujours `Content-Disposition`) ; L2 `\subsubsection` (confirmé CORRECT pour la hiérarchie LaTeX réelle par l'Auditor) ; onMount `Promise.all` masque la liste si un référentiel échoue (référentiels rarement en échec).

Gate frontend : check 0 err + lint-i18n PASS + 340 test:unit + build ; **E2E 3/3 réels** (round-trip + discard + disabled-assert). Pass 2 : Haiku 4.5.

### Code-review Pass 2 (Haiku 4.5, 2 couches : correctness+spec / edge+accept), 2026-06-30 — CONVERGÉ
**0 CRITICAL/HIGH/MEDIUM sur les 2 couches.** Tous les patches Pass 1 (EC1-EC6, BH1/BH2/BH7, M1/M2/M3) vérifiés présents ground-truth. Tous AC1-AC15 + DC-d1..d5 conformes ; parité TTC `lineVatAmount` pleine précision, validation structurelle complète, `safeReloadList`, boutons `disabled`, docker prod bind-mounts justifié, E2E 3 tests. **Aucune hallucination CRITICAL/HIGH Haiku** (rien à grep-réfuter). LOW : (a) clés index `report.failed`/`warnings` — acceptable (arrays immuables, décidé Pass 1) ; (b) `|| fContactId === null` « redondant » — en réalité **nécessaire au narrowing TS** (`structurallyInvalid()` n'informe pas le compilateur), pas d'action ; (c) `spc_e2e_discard.png` manquant en File List → **ajouté**.

### Synthèse du cycle code-review
Trend > LOW : **Pass 1 (Sonnet) 9 (1 HIGH + 8 MEDIUM) → Pass 2 (Haiku) 0**. 2 passes, rotation Sonnet→Haiku (2 modèles). Catch majeur Pass 1 : cluster **validation client incomplète** (`expenseAccountId===0`, montants/date non validés → erreurs backend brutes) + robustesse (report effacé, form partagé, rejection non gérée). Convergence en 2 passes (story UI frontend sans surface d'intégrité transactionnelle type 12-5c → pas de 3ᵉ passe Opus nécessaire ; **E2E 3/3 réels** = filet de sécurité fort). 1 déviation assumée justifiée (docker prod bind-mounts pour usabilité NAS File Station). **Épopée 12-5 COMPLÈTE 4/4** (a/b/c/d). Prochaine : PR umbrella 12-5.

### Synthèse du cycle validate
Trend > LOW : **Pass 1 (Sonnet) ~9 → Pass 2 (Haiku) 1 → Pass 3 (Opus) 2 → Pass 4 (Sonnet) 0**. 4 passes, rotation Sonnet→Haiku→Opus→Sonnet (cycle complet). Catches majeurs : **Pass 1** (DC non figés → figés ; error-handling exhaustif des 6 endpoints ; i18n prefix lint ; website checklist) ; **Pass 3 Opus** (parité numérique TTC `lineVatAmount` vs `computeLineTotal` HT-seul ; contradiction redirect vs worklist batch). 1 faux-positif Haiku réfuté (« docs CRITICAL non implémentés » = travail prescrit AC9). Convergence au seuil 4 passes → **pas de re-split** (règle splitting : boucle *au-delà* de 4 sans converger ; ici convergé À 4). Backend 12-5c `done` non touché (frontière frontend+doc respectée). **Prêt pour `bmad-dev-story 12-5d`** (Opus recommandé). Reco dev : AC1 (feature module) → AC8 i18n keys → AC2-7 (pages) → AC9-13 doc → AC14 E2E.

## Dev Agent Record

### Agent Model Used

Opus 4.8 (1M) — dev-story orchestré, run unique T-d1→T-d9. Quality gate frontend
exit-code vérifié + **E2E exécuté en réel** contre un backend monté (DB isolée `kesh_e2e`).

### Debug Log References

- **E2E `each_key_duplicate`** : le formulaire de complétion ne s'ouvrait pas en E2E. `pageerror` = `svelte.dev/e/each_key_duplicate`. Cause : le seed `with-company` a **4 taux TVA tous catégorie `custom`** (id 1-4) → `{#each vatRates as r (r.category)}` produit 4 clés `custom` dupliquées → crash runtime du rendu. Fix : clé `(r.id)`. *(Bug latent identique dans `supplier-invoices/+page.svelte` ground-truth — son formulaire n'est jamais ouvert en E2E, hors scope ici.)*
- Run E2E réel : backend démarré `cargo run` (debug) port 8788, `KESH_TEST_MODE=true`, `DATABASE_URL=…/kesh_e2e` (DB isolée, jamais la DB dev), `KESH_INBOX_DIR=/tmp/kesh-e2e-inbox`, `KESH_STATIC_DIR=frontend/build` ; `KESH_BACKEND_URL` + override Playwright Ubuntu → **2/2 verts**.

### Completion Notes List

- **T-d1 (feature module)** — `imported-supplier-invoices/{types,api}` : 6 wrappers `apiClient`, downloads **Pattern A** (`getBlob` + ancre éphémère, gère 401-refresh) avec `parseContentDispositionFilename` réimporté d'`exports.api.ts`. Décimaux `string`. AUCUN `i18nMsg` dans le feature module (lint-i18n-ownership).
- **T-d2/3 (écran import + liste)** — page `/supplier-invoices/import` + guard `+page.ts` Comptable+ : bouton import (spinner + `finally`), rapport batch (map errorCode→FR clés `imported-supplier-invoices-error-*` dans le fichier de route), 409 discriminé + fallback `notifyError` ; liste `to_complete` `onMount`, état vide, download justificatif 404/410.
- **T-d4 (complétion)** — formulaire inline par ligne (pattern ground-truth `supplier-invoices/+page.svelte` qui est inline, pas Dialog ; déviation cosmétique assumée vs spec DC L6) : `<select>` fournisseur `isSupplier`, lignes, **Σ TTC live via `lineVatAmount` (parité backend, comparé pleine précision)** + avertissement sous-centime, validation structurelle (bouton désactivé si pas de fournisseur/0 ligne/desc vide ; jamais sur le montant — DC-d3), mapping rejets `CURRENCY_NOT_SUPPORTED`/`IBAN_REFERENCE_MISMATCH`/`AMOUNT_MISMATCH`/`IMPORT_NOT_PENDING_COMPLETION`/`IMPORTED_INVOICE_NOT_FOUND`/`FISCAL_YEAR_INVALID`, **reste sur le worklist** (F-OPUS-2, pas de redirection).
- **T-d5 (écarter)** — confirm → `discardImport` 204 + retrait ; erreurs 409/404 + reload + fallback.
- **T-d6 (justificatif détail)** — `supplier-invoices/[id]` lien « Voir la facture d'origine » always-render (DC-d1), 404→`notifyInfo` / 410→`notifyWarning`.
- **T-d7 (sidebar/i18n)** — entrée `nav-supplier-invoices-import` ; clés `imported-supplier-invoices-*` (fallback FR inline, lint PASS).
- **T-d8 (doc)** — `.env.example` (5 vars) ; `docker-compose.yml` (env + volumes nommés) + `docker-compose.prod.yml` (env + bind mounts host `./inbox`/`./documents`) ; `CHANGELOG` ; `README` (Fonctionnalités ✓ + roadmap E12) ; `website/roadmap.html` (E12 rafraîchi) + `index.html` vérifié (pas de claim erroné) ; `admin-manual.tex` (5 vars + section import Synology/pdfium L6/inbox-#199) + **PDF régénéré** (`\keshVersion` NON bumpé).
- **T-d9 (E2E + gate)** — `inbox-import.spec.ts` (guard + import-vide + round-trip réel) + fixture `spc_e2e_invoice.png`. **Gate** : `check` 0 err, `lint-i18n-ownership` PASS, `test:unit` 340/340, `build` OK ; **E2E 2/2 réels**.
- **Compteurs en dur** : DÉJÀ faits 12-5b (TABLES_TO_TRUNCATE, manifeste, `admin_full_export_e2e=31`, `migrations_upgrade_path=39`, idempotence) — aucun fichier Rust touché par 12-5d → suite backend inchangée.

### File List

**Créés** :
- `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.types.ts`
- `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.api.ts`
- `frontend/src/routes/(app)/supplier-invoices/import/+page.ts`
- `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`
- `frontend/tests/e2e/inbox-import.spec.ts`
- `frontend/tests/e2e/fixtures/spc_e2e_invoice.png`
- `frontend/tests/e2e/fixtures/spc_e2e_discard.png` (code-review Pass 1 M1)

**Modifiés** :
- `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte` (lien justificatif)
- `frontend/src/routes/(app)/+layout.svelte` (entrée sidebar)
- `.env.example`, `docker-compose.yml`, `docker-compose.prod.yml`, `CHANGELOG.md`, `README.md`
- `website/roadmap.html`
- `docs/manual/fr/admin-manual.tex` + `docs/manual/fr/admin-manual.pdf`
