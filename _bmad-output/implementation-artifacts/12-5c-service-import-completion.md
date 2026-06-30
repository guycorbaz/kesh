# Story 12.5c: Service d'import + endpoints + complétion atomique

Status: ready-for-dev

<!-- Sous-story 3/4 de l'umbrella 12-5 (validate convergé 6 passes). Périmètre T4 : service de lecture inbox + sécurité + verrou de run + rapport batch FailedProposal + refactor DC6 `create_in_tx` + endpoints liste/complétion/écarter/download. Dépend de 12-5a (parse_spc_payload) ET 12-5b (qr_decode, document_storage, entité/repo imported_supplier_invoices, KESH_DOCUMENTS_DIR). NE COUVRE PAS le frontend ni la doc (12-5d). Absorbe les 2 dettes déférées en code-review 12-5b (validation status HTTP, map_db_error 1406/1264). -->

## Story

As a comptable PME utilisant Kesh sur mon NAS,
I want déclencher l'import du dossier de factures (lecture inbox → décodage QR → staging + archivage du fichier), puis **compléter** chaque facture importée (fournisseur, date, lignes) de façon **atomique et sûre**,
so that les factures fournisseurs entrent dans la comptabilité sans ressaisie des coordonnées de paiement, sans double écriture comptable, et avec leur justificatif récupérable.

## Contexte & source

- Sous-story **3/4** de l'umbrella **12-5** (import répertoire factures, issue **#194**, cible v0.4). Cf. `12-5-import-repertoire-factures.md` (AC2/AC3/AC4/AC5/AC6/AC7/AC10, DC6, schéma, Dev Notes ground-truth).
- **Dépend de 12-5a** (`parse_spc_payload`, `ScannedQrBill` dans `kesh-qrbill`, commit `85a235b`) **et 12-5b** (décodage + entité + stockage, mergé sur la branche umbrella) : ce module **consomme** `qr_decode::{decode_spc_from_image_bytes, decode_spc_from_pdf_bytes}`, `document_storage::{store_document, read_document}`, le repository `imported_supplier_invoices` et le mapping `NewImportedSupplierInvoice::from_scanned`.
- **Débloque 12-5d** (frontend + doc) qui **consomme** les endpoints livrés ici (aucune logique transactionnelle en frontend).
- **Absorbe 2 dettes déférées de la code-review 12-5b** (cf. §Dettes héritées 12-5b).

## Périmètre (et hors-périmètre)

**DANS 12-5c (T4)** :
- **Config** : `KESH_INBOX_DIR` (défaut `/data/inbox`) + caps `KESH_INBOX_MAX_FILE_BYTES` (défaut 25 Mo), `KESH_INBOX_MAX_FILES_PER_RUN` (défaut 200), `KESH_INBOX_MAX_PDF_PAGES` (défaut 20, câblé à `DecodeConfig.max_pages` 12-5b).
- **Service d'import** : lecture inbox sur déclenchement manuel (DC2) + **verrou de run** (sérialisation, F6) + court-circuit doublon hash (réactive `discarded`, F5) + boucle décodage (12-5b) + création staging + archivage fichier (12-5b) + suppression inbox au succès / déplacement `failed/` à l'échec + rapport batch `{accepted, failed, warnings}` (HTTP 200, pattern FailedProposal).
- **Sécurité d'ingestion** (AC4) : symlink rejeté, anti-traversal (canonicalisation sous racine), liste blanche d'extensions + **magic bytes**, taille max, stabilité fichier (en cours d'écriture), caps pdfium.
- **Refactor DC6** : extraire `supplier_invoices::create_in_tx(&mut tx, …)` (miroir de `pay_in_tx`) sans changer le comportement de `create` (pool-owning) existant.
- **Endpoints** (Comptable+, company-scopés) : `POST .../inbox-import`, `GET .../imported-supplier-invoices?status=`, `POST .../imported-supplier-invoices/{id}/complete` (siège DC6 + réconciliation F2 + routage IBAN F9 + validation devise), `POST .../imported-supplier-invoices/{id}/discard`, `GET .../imported-supplier-invoices/{id}/source-document`, `GET .../supplier-invoices/{id}/source-document`.
- **Dettes 12-5b absorbées** : validation `status` à la frontière HTTP (liste) ; mapping `map_db_error` des codes 1406/1264 → variant typé pour échec par-fichier propre.
- **Compteurs/tests** : aucune nouvelle table (la table est livrée 12-5b) ; tests d'intégration import + complétion + sécurité + download.

**HORS 12-5c** (→ 12-5d) :
- Tout le frontend (écran import, rapport UI, liste/complétion/écarter UI, lien « Voir la facture d'origine »).
- Doc utilisateur/admin, `.env.example`, `docker-compose.yml` volumes, CHANGELOG, README (12-5d, sauf si une env var introduite ici impose une note minimale — alors la note minimale `.env.example` est tolérée ici).
- E2E Playwright (12-5d, après l'UI).

## Acceptance Criteria

### Service d'import & rapport batch

1. **Config inbox** : `KESH_INBOX_DIR` (pattern `from_env` comme `KESH_DOCUMENTS_DIR` 12-5b / `KESH_ADMIN_BACKUP_DIR`, défaut **`/data/inbox`**) + `KESH_INBOX_MAX_FILE_BYTES` (défaut 25*1024*1024), `KESH_INBOX_MAX_FILES_PER_RUN` (défaut 200), `KESH_INBOX_MAX_PDF_PAGES` (défaut 20). Champs ajoutés à `Config`, lus dans `from_env` **ET** mis à jour dans `from_fields_for_test` (`config.rs:~388`, champ `documents_dir` assigné `:~466`) + `make_test_config` (`config.rs:~1328`) — **2 fonctions** à éditer, sinon échec de compilation des ~33 call-sites de tests en aval (défauts ex. `inbox_dir: "/tmp/kesh-inbox-test"`). Le wiring `KESH_INBOX_MAX_PDF_PAGES` → `DecodeConfig { max_pages, .. }` (12-5b) est fait **ici** (AC3/L6 12-5b délégait ce câblage à 12-5c).
   **Canonicalisation (anti-traversal robuste, F-NAS)** : la comparaison « chemin sous la racine » exige que la racine soit **canonicalisée**. Sur NAS Synology, `/data/inbox` est souvent un symlink vers `/volume1/.../inbox` → `canonicalize(entry)` (`/volume1/...`) ne commencerait PAS par `/data/inbox` brut → **tous les fichiers légitimes échoueraient** l'anti-traversal. Donc : canonicaliser `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` **une fois** (au démarrage si le dossier existe — sinon `create_dir_all` puis canonicalize, ou canonicalisation lazy au 1ᵉʳ run) et stocker la forme résolue ; l'anti-traversal compare `canonicalize(entry).starts_with(racine_canonicalisée)`.

2. **`POST /api/v1/inbox-import`** (Comptable+) : lit `KESH_INBOX_DIR`, traite chaque entrée, retourne **HTTP 200** avec body `{ accepted: [...], failed: [...], warnings: [...] }` (pattern FailedProposal CLAUDE.md — **aucune** erreur per-fichier n'escalade en `AppError` global ; exceptions globales = 401/403 RBAC, 409 run déjà en cours, 500 catastrophe DB/IO).
   - `accepted[]` : `{ imported_supplier_invoice_id: i64, file_name: String }` (staging créé).
   - `failed[]` : `{ file_name: String, error_code: String (constante), details: Option<serde_json::Value> }` — **identifiant business = `file_name`**, JAMAIS d'index positionnel ; `error_code` = constante canonique (jamais `format!`).
   - `warnings: Vec<String>` : avertissements non liés à un fichier précis. **Borné** : **un seul** message de troncature `MAX_FILES_PER_RUN` (pas un par fichier excédentaire) ; un message par fichier instable, borné par `MAX_FILES_PER_RUN`.
   - **Catalogue `error_code`** (constantes) : `UNSUPPORTED_FILE_TYPE`, `FILE_TOO_LARGE`, `SYMLINK_REJECTED`, `DUPLICATE`, `NO_QR_CODE_FOUND`, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR`, `FILE_READ_ERROR`, **`FIELD_TOO_LONG`** (champ QR tiers hors bornes SIX 2.2 dépassant la largeur de colonne — dette D2, distinct de `INVALID_SPC_PAYLOAD` qui = payload non-SPC). Mapping `DecodeError` (12-5b) → `error_code` : `ImageDecode`→**`UNSUPPORTED_FILE_TYPE`** (le magic-bytes a déjà filtré les non-images en step 4 ; un `ImageDecode` à ce stade = format reconnu mais corrompu/non décodable, sémantiquement « type non exploitable » ; ne pas mapper en `FILE_READ_ERROR` qui est réservé aux I/O — le contenu a déjà été lu), `PdfRender`→`PDF_RENDER_ERROR`, `InvalidSpcPayload`→`INVALID_SPC_PAYLOAD`, `InvalidIban`→`INVALID_IBAN` ; absence de QR SPC (`Ok(None)`)→`NO_QR_CODE_FOUND` ; `DbError` issu de 1406/1264 à l'INSERT staging (D2)→`FIELD_TOO_LONG`.

3. **Boucle de traitement par fichier** : `read_dir(KESH_INBOX_DIR)` **non récursif** (pas de descente dans les sous-dossiers). Pour chaque entrée (jusqu'à `MAX_FILES_PER_RUN`) :
   0. **Filtrage type d'entrée** : si l'entrée n'est **ni un fichier régulier ni un symlink** (répertoire — dont `failed/` lui-même créé dans l'inbox —, socket, FIFO, device) → **silencieusement ignorée** (ni `accepted`, ni `failed`, ni `warning`). Évite que `failed/` génère un `UNSUPPORTED_FILE_TYPE` à chaque run + un `rename` invalide (EINVAL). Si `symlink_metadata()` retourne `Err` (entrée disparue en race) → ignorée silencieusement (warning facultatif), pas de `failed[]`.
   1. **Symlink** : `symlink_metadata().is_symlink()` → `SYMLINK_REJECTED` (jamais suivi/ouvert), déplacé `failed/`.
   2. **Taille** : `std::fs::metadata(path)?.len()` (stat, **PAS** de lecture du contenu) > `MAX_FILE_BYTES` → `FILE_TOO_LARGE`. Le check taille **précède toute lecture** `fs::read`/`File::open` du contenu — sinon un fichier de plusieurs Go chargé en RAM avant le check = DoS mémoire.
   3. **Stabilité** : taille + mtime identiques sur 2 lectures espacées d'un court délai ; si instable → **ignoré ce tour** (ni `accepted` ni `failed`, warning), retenté au prochain déclenchement. Lecture impossible → `FILE_READ_ERROR`.
   4. **Type** : extension ∈ liste blanche (`pdf`,`png`,`jpg`,`jpeg`) **ET** magic bytes cohérents (PDF `%PDF`, PNG/JPEG signatures) ; sinon `UNSUPPORTED_FILE_TYPE`.
   5. **Hash SHA-256** calculé **avant** décodage → court-circuit doublon (AC6) : si `(company_id, file_hash)` matche une row `to_complete`/`completed` → `DUPLICATE` (déplacé `failed/`) ; si matche une row `discarded` → **réactivation `to_complete`** via `reactivate_to_complete` (qui remet aussi `supplier_invoice_id=NULL`, cf. Dev Notes) (F5, `accepted`, fichier inbox supprimé).
   6. **Décodage** (12-5b) : PDF → `decode_spc_from_pdf_bytes(bytes, DecodeConfig{ max_pages: KESH_INBOX_MAX_PDF_PAGES, .. })` ; image → `decode_spc_from_image_bytes(bytes)`. `Ok(Some)` → suite ; `Ok(None)` → `NO_QR_CODE_FOUND` ; `Err(DecodeError)` → `error_code` mappé.
   7. **Archivage** : `store_document(documents_dir, bytes, ext, original_filename, mime)` (12-5b) → `DocumentMeta`.
   8. **Staging** : `NewImportedSupplierInvoice::from_scanned(company_id, &scanned, doc)` → `imported_supplier_invoices::create`. Violation `UNIQUE (company_id, file_hash)` (race) → `DUPLICATE` per-fichier (HTTP 200, **PAS** d'`AppError` 500). **Échec `create` non-DUPLICATE** (ex. 1406 → `FIELD_TOO_LONG`, ou erreur DB) : le fichier vient d'être archivé (step 7) mais n'aura pas de row → **nettoyer le fichier archivé** (best-effort `remove_file(storage_path)`) pour éviter un orphelin sur disque, OU accepter l'orphelin borné (le stockage est content-addressed `{sha256hex}.{ext}`, donc un ré-import du même fichier réécrit le même chemin — orphelin idempotent, non cumulatif ; documenter le choix). Le fichier inbox part en `failed/`.
   9. **Succès** → suppression du fichier inbox (tolérer `ENOENT`, idempotent). **Échec** → déplacement vers `failed/`. Le dossier `failed/` est créé (`create_dir_all`) **une fois avant la boucle** (plus robuste qu'à la volée). **Nommage anti-collision** (localisé à `failed/` ; les fichiers archivés en `KESH_DOCUMENTS_DIR` gardent le standard `{sha256hex}.{ext}` de 12-5b) : la cible dans `failed/` = **`{stem}_{8 premiers hex du sha256}.{ext}`** où `{stem}` est extrait via `Path::file_stem()` (filename seul, jamais les composants de chemin — un `original_filename` malveillant `../../x.pdf` ne doit pas s'échapper). `std::fs::rename` écrase atomiquement une cible homonyme sur POSIX → le suffixe hash évite qu'un 2ᵉ fichier de même nom écrase le 1ᵉʳ. Anti-traversal sur le nom (jamais de composant `/`/`..`).

4. **Sécurité** (AC4 umbrella) : chemins via env (admin) ; canonicalisation `KESH_INBOX_DIR`/`failed/`/`KESH_DOCUMENTS_DIR` (chemin résolu reste sous la racine, cf. AC1) ; jamais d'écriture hors `KESH_DOCUMENTS_DIR`/`failed/` ; nom archivé = `{sha256hex}.{ext}` (12-5b, jamais le nom d'origine). **Anti-TOCTOU symlink** : le check `is_symlink()` (step 1) et l'ouverture effective (steps 3-4) sont séparés → un attaquant local pourrait substituer un symlink entre les deux. Atténuation : ouvrir le fichier avec **`O_NOFOLLOW`** (`OpenOptionsExt::custom_flags(libc::O_NOFOLLOW)` sur Linux → `ELOOP` si symlink au moment de l'ouverture) plutôt que `fs::read` aveugle ; à défaut, re-vérifier `is_symlink()` juste avant lecture. Risque modéré (inbox semi-contrôlée admin, exploitation = présence locale sur le NAS) mais documenté. **Caps pdfium** (12-5b expose `DecodeConfig`) : `max_pages` câblé ici ; **L6** (segfault natif tue le process) acceptée v0.4.

5. **Verrou de run (F6)** : deux déclenchements concurrents (2 onglets / double-clic) → sérialisés. Si un import est déjà en cours → **HTTP 409** (comportement FIGÉ, PAS de warning+no-op — le frontend 12-5d doit pouvoir distinguer « rapport d'import » d'un succès partiel vs « refus 409 » dans son `catch`). Variant à créer : **`AppError::InboxImportAlreadyRunning` → HTTP 409, `error_code` `INBOX_IMPORT_ALREADY_RUNNING`** (aucun variant `RunAlreadyInProgress`/équivalent n'existe dans `errors.rs` — grep 0). **Impératifs d'implémentation (NON négociables, sinon fuite de verrou)** :
   - **Option A — `GET_LOCK` MariaDB** : `GET_LOCK` tient le verrou **sur la connexion qui l'a acquis**. Le pool sqlx recycle les connexions → utiliser `pool.acquire()` pour obtenir une **`PoolConnection` dédiée**, exécuter `SELECT GET_LOCK('kesh_inbox_import', 0)` dessus, la **garder vivante pendant TOUT le run**, faire `RELEASE_LOCK` sur **cette même connexion**, puis la drop. NE PAS exécuter GET_LOCK via `pool.execute()` (la connexion repart au pool, le lock fuit).
   - **Option B — `AtomicBool`** : `compare_exchange(false, true)` pour acquérir ; **RAII guard obligatoire** (`struct RunGuard<'a>(&'a AtomicBool); impl Drop { store(false) }`) pour relâcher le flag sur **tout** chemin de sortie (early-return `?`, panic de task). Sans guard Drop, un early-return laisse le flag `true` → tous les imports suivants reçoivent 409 jusqu'au redémarrage. Champ `inbox_import_running: Arc<AtomicBool>` à ajouter à `AppState` (`lib.rs`) avec défaut `false` dans `new_for_tests`/`from_fields_for_test` (~33 sites tests, cf. pattern `users_exist`).
   - Documenter le choix retenu (A recommandé : pas d'impact `AppState`/tests).

### Refactor DC6 & complétion atomique

6. **Refactor `supplier_invoices::create_in_tx`** : extraire le cœur de `create` (`repositories/supplier_invoices.rs:212-406`, qui `pool.begin()` l.239 / `tx.commit()` l.406) en **`create_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, new: NewSupplierInvoice, user_id: i64) -> Result<SupplierInvoiceWithLines, DbError>`** — miroir du vrai `create(pool, new, user_id)` (`supplier_invoices.rs:212-216`) dont `company_id` est destructuré depuis `new.company_id` (PAS un param séparé — ne pas dupliquer la source). Même esprit que `pay_in_tx` (prend `&mut tx`, aucun `begin`/`commit` interne ; retourne `SupplierInvoiceWithLines`). **Type retour = `SupplierInvoiceWithLines` (kesh-db), PAS `SupplierInvoiceResponse`** : `SupplierInvoiceResponse` vit dans `kesh-api` (`routes/supplier_invoices.rs:61`) et `kesh-db` **ne peut pas en dépendre** (cycle Cargo). Le handler HTTP construit la réponse via `SupplierInvoiceResponse::from_parts(inv.invoice, inv.lines)` (pattern `supplier_invoices.rs:303`). `create` (pool-owning) **conserve son comportement** (appelle `create_in_tx` puis `commit`, comme `pay` appelle `pay_in_tx`). Aucune régression des tests 12-2.

7. **`POST /api/v1/imported-supplier-invoices/{id}/complete`** (Comptable+, company-scopé) — corps JSON camelCase :
   ```json
   { "contactId": i64, "invoiceDate": "YYYY-MM-DD",
     "supplierInvoiceNumber": "string|null", "dueDate": "YYYY-MM-DD|null",
     "lines": [{ "description": "string", "quantity": "Decimal",
                 "unitPrice": "Decimal (HT)", "vatRate": "Decimal", "expenseAccountId": i64 }] }
   ```
   **Complétion atomique (DC6)** dans **une seule transaction** :
   1. `SELECT … FROM imported_supplier_invoices WHERE id=? AND company_id=? FOR UPDATE` (garde anti-double + anti-IDOR).
   2. Si `status != 'to_complete'` → abandon, **HTTP 409 Conflict** avec corps `{ errorCode: "IMPORT_NOT_PENDING_COMPLETION", details: { currentStatus: "<status>" } }` (permet au frontend de distinguer « déjà complétée/écartée » d'une erreur serveur 500). Aucune écriture.
   3. **Validation devise — CHF UNIQUEMENT en v0.4 (décision Guy 2026-06-30, F-OPUS-1)** : `staging.currency == "CHF"` sinon **rejet** « Devise non supportée en v0.4 (CHF uniquement) ». **Rationale** : la devise n'est PAS persistée sur `supplier_invoices` (L7) ET le générateur pain.001 (12-3) hardcode `Ccy="CHF"` (`pain001/mod.rs:25`) → une facture EUR entrant dans un lot pain.001 ferait **payer la banque en CHF** (mislabel silencieux). Accepter EUR sans devise persistée + propagée serait un défaut d'intégrité de paiement. **Déroge à l'umbrella AC7.3/P4-M4 (∈{CHF,EUR})** — restriction assumée v0.4 ; support EUR = follow-up (pain.001 multi-devise + colonne `currency` sur `supplier_invoices`), cf. L7 mis à jour. **NE PAS** ajouter de champ `currency` à `NewSupplierInvoice`/migration 12-2 (L7).
   4. **Routage IBAN (F9)** : `staging.is_qr_iban=true` → `NewSupplierInvoice.creditor_qr_iban` + **exiger `staging.reference_type='QRR'`** (sinon rejet métier). **Cas inverse** : `is_qr_iban=false` **+** `reference_type='QRR'` = combinaison invalide per SIX SPC 2.2 (QRR ↔ QR-IBAN seulement) → **rejet métier** « QRR exige un QR-IBAN ». Sinon `is_qr_iban=false` → `creditor_iban`. **Pourquoi valider ici** : `payment_batches.rs:613` ne valide PAS cette cohérence en aval (il route `PaymentReference::Qrr(reference.unwrap_or_default())` sans vérifier le type d'IBAN) → un pain.001 invalide serait généré ; la validation doit donc être faite **en amont** à la complétion. Note : si 12-5a `parse_spc_payload` garantit déjà la cohérence IBAN↔référence, documenter l'invariant et la validation ici devient une défense en profondeur.
   5. **Mapping `payment_reference` (C5-1)** : `staging.reference_value` si `reference_type ∈ {QRR,SCOR}`, sinon `None`.
   6. **Réconciliation montant (F2)** : si `staging.amount` présent → exiger l'**égalité EXACTE** `Σ lignes TTC == staging.amount` (pleine précision, **PAS** une tolérance `round2`) sinon bloquer « Le total des lignes (X) ne correspond pas au montant du QR (Y) ». **Pourquoi exact et non `round2` (F-OPUS-2)** : `total_amount` est stocké en `DECIMAL(19,4)` pleine précision par `create_in_tx`/`generate_purchase_journal_lines` (`line_total = quantity*unit_price` non arrondi au centime). Une tolérance `round2` laisserait passer `Σ TTC = 99.9999` contre `staging.amount = 100.00` → `total_amount` stocké (99.9999) ≠ `staging.amount` ≠ `expected_payment_amount` (100.00) : trois valeurs en désaccord, résidu non réconciliable, et pain.001 (qui arrondit l'affichage à 2 déc.) paierait 100.00 alors que le grand livre boucle à 99.9999. L'égalité exacte **rejette** les lignes à produit sous-centime (le message pousse l'utilisateur à saisir des lignes centime-exactes) et garantit `total_amount == staging.amount == expected_payment_amount` **par construction**. **Le `Σ lignes TTC` DOIT être calculé exactement comme `create_in_tx`/`generate_purchase_journal_lines`** — TVA via `kesh_core::accounting::vat::line_vat_amount(line_total, vat_rate)` (arrondie par ligne, HALF_UP/centime, FR55 AFC), idéalement le même helper. Renseigner `NewSupplierInvoice.expected_payment_amount = staging.amount`. Si `staging.amount` absent → pas de contrôle. **Note (faux-négatifs acceptés, hors-SPC)** : (a) un QR tiers calculé en arrondi TVA global (≠ par ligne AFC) peut différer d'un centime ; (b) un QR tiers **non conforme SIX 2.2** dont `amount` porte >2 décimales non nulles (ex. `100.001`) ne pourra jamais matcher un `Σ TTC` centime-exact (`parse_spc_payload` 12-5a fait `Decimal::from_str` sans borner l'échelle). Dans les deux cas → blocage ; comportement attendu (l'utilisateur écarte la facture et la saisit manuellement, ou ressaisit ses lignes pour matcher le total QR autoritatif), pas un bug. Portée quasi-nulle (toute banque suisse valide le QR avant impression).
   7. `create_in_tx(&mut tx, …)` (AC6, retourne `SupplierInvoiceWithLines`) → `UPDATE imported_supplier_invoices SET status='completed', supplier_invoice_id=? WHERE id=?` → `commit`.
   - Le **handler HTTP** retourne **`SupplierInvoiceResponse`** (construit via `SupplierInvoiceResponse::from_parts(...)` à partir du `SupplierInvoiceWithLines` retourné par `create_in_tx`, identique à `POST /api/v1/supplier-invoices`). `create_in_tx` lui-même retourne `SupplierInvoiceWithLines` (kesh-db).
   - **Rejets métier des steps 3/4/6** (pré-`create_in_tx`, pas un `FailedProposal` car requête mono-item) → **HTTP 400** via `AppError::Validation(...)` (rend `VALIDATION_ERROR`, `errors.rs:726`) avec un `errorCode` canonique distinct pour que 12-5d guide l'utilisateur sans parser le message : step 3 devise → `CURRENCY_NOT_SUPPORTED` ; step 4 IBAN/QRR → `IBAN_REFERENCE_MISMATCH` ; step 6 réconciliation → `AMOUNT_MISMATCH`. (step 2 statut → 409 `IMPORT_NOT_PENDING_COMPLETION`, déjà figé.)
   - **Échec de `create_in_tx`** : rollback → staging **reste `to_complete`**, message UX explicite, **aucune écriture comptable partielle**. `DbError::FiscalYearInvalid` → « Aucun exercice fiscal ouvert couvrant cette date » ; montant ≤ 0 → « Le montant doit être positif ».

8. **`POST /api/v1/imported-supplier-invoices/{id}/discard`** (Comptable+, company-scopé) : transition `to_complete` → `discarded` (FOR UPDATE, garde statut). Le fichier archivé est **conservé** (pas de suppression du justificatif v0.4).

### Endpoints liste & download

9. **`GET /api/v1/imported-supplier-invoices?status={to_complete|completed|discarded}`** (Comptable+, company-scopé) : liste filtrée par statut. **`status` est obligatoire** (pas de défaut côté API ; l'affichage par défaut `to_complete` est une décision frontend 12-5d, pas une valeur API implicite). **Validation du `status`** à la frontière HTTP (dette D1 absorbée) : `status` absent ou hors-domaine `{to_complete,completed,discarded}` → **400 Bad Request** via `AppError::Validation(...)` (existe `errors.rs:66`), PAS une liste vide silencieuse. Consomme `repositories::imported_supplier_invoices::list_by_status` (scopé `company_id`).

10. **Download du justificatif** (Comptable+, company-scopé, anti-IDOR `WHERE company_id = {current}`) :
    - `GET /api/v1/imported-supplier-invoices/{id}/source-document` — avant complétion : résout via `imported_supplier_invoices::find_by_id_scoped(company_id, id)` (anti-IDOR explicite).
    - `GET /api/v1/supplier-invoices/{id}/source-document` — après complétion (résout `imported_supplier_invoices WHERE supplier_invoice_id = {id} AND company_id = {current}`).
    - Lit via `document_storage::read_document(documents_dir, storage_path)` (12-5b). Headers `Content-Type` (= `mime_type` stocké) + `Content-Disposition: attachment; filename="{original_filename}"`.
    - **Sémantique d'absence, JAMAIS 500** : **row absente** (facture sans justificatif, L5) → **404 Not Found** « facture sans justificatif stocké » ; **`ReadDocumentError::NotFound`** (row présente mais fichier disque absent, ex. restore métadonnée-seule L1, F7) → **410 Gone** « justificatif non restauré ». `ReadDocumentError::InvalidPath` → 500 (corruption interne, ne devrait jamais arriver). Test dédié (métadonnée sans fichier disque → 410 ; row absente → 404 ; jamais 500).

### Qualité

11. **Tests** (intégration `#[sqlx::test]` + service) : import (dossier temp avec fixtures PNG/PDF → staging + fichiers archivés + rapport ; échecs `failed[]` par catégorie) ; sécurité (symlink, traversal, type refusé/magic-bytes, doublon hash scopé company, réactivation `discarded`, taille max, **entrée répertoire ignorée** sans pollution `failed[]`) ; verrou de run (2e appel concurrent → 409) ; **complétion atomique** (`to_complete`→`completed` + facture `open` + `supplier_invoice_id` lié ; échec exercice fermé → reste `to_complete`, pas d'écriture ; réconciliation montant KO → bloqué ; routage QR-IBAN⇒QRR + cas invalide `is_qr_iban=false`+QRR rejeté ; **devise EUR rejetée (CHF only v0.4, F-OPUS-1)** ; **réconciliation : lignes à produit sous-centime `quantity=3,unit_price=33.3333` → bloqué** (égalité exacte, F-OPUS-2)) ; `discard` ; download (404 fichier absent, anti-IDOR cross-company, Content-Disposition) ; **D1 (dette) : `GET .../imported-supplier-invoices?status=invalid` → 400 `AppError::Validation` ; sans `status` → 400 ; `?status=to_complete` → 200 + liste scopée `company_id`** ; **D2 (dette) : fixture avec `creditor_name` > 70 chars → import retourne `failed[{ file_name, error_code: "FIELD_TOO_LONG" }]`, HTTP 200 (PAS 500)** — test de non-régression de la résolution D2. Refactor `create_in_tx` : 0 régression des tests 12-2.

- **Fixtures QR pour les tests d'intégration** : le service d'import vit dans `kesh-api` ; les tests d'intégration `crates/kesh-api/tests/*.rs` **n'ont PAS accès** au helper `#[cfg(test)]` `render_spc_qr_png` de la lib (`qr_decode.rs`, privé au crate-test de la lib). Stratégie : (a) un **helper de génération PNG QR** réutilisable — soit `pub(crate)` exposé hors `#[cfg(test)]`, soit ré-implémenté dans un module commun de tests (`tests/common/`), construisant un payload SPC via `kesh_qrbill::build_payload` puis encodé en PNG via `rxing::…::QRCodeWriter` (rxing est désormais dép runtime de kesh-api) ; OU (b) **fixtures pré-générées commitées** dans `crates/kesh-api/tests/fixtures/` (`spc_*.png`, `spc_*.pdf`). Le PDF QR est plus délicat (pas de writer PDF trivial) → fixture pré-générée commitée recommandée pour le cas PDF (test `#[ignore]` si pdfium absent, cf. 12-5b AC5). Documenter le choix au dev.

12. **Quality gate Test Locally First — exit code vérifié** (PAS `cargo test | grep`) : `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo build --workspace --all-targets` + `cargo test --workspace -j1 -- --test-threads=1`. 0 régression.

## Dettes héritées 12-5b (à absorber ici)

- **D1 — validation `status` (list_by_status)** : le repository `list_by_status` (12-5b) ne valide pas le `status` (retourne liste vide sur valeur hors-domaine). **Ici (AC9)** : valider le query param à la frontière HTTP → 400 sur valeur invalide. (Contrat documenté en doc de `list_by_status` 12-5b.)
- **D2 — `map_db_error` 1406/1264** : un champ QR tiers sur-long (nom > 70, message > 140, etc.) fait échouer l'INSERT staging avec un `DbError` générique opaque. Aujourd'hui `map_db_error` (`errors.rs:151`) ne gère que 1062/1451/1452/4025/3819 ; les codes MariaDB **1406** (Data too long) et **1264** (Out of range) tombent dans le variant de repli **`DbError::Sqlx(sqlx::Error)`** (`errors.rs:163` — il n'existe PAS de variant `DbError::Other`). **Ici** : ajouter un variant typé `DbError::DataLengthOrRange(String)` mappé depuis 1406/1264, exploité par le service d'import pour un **échec par-fichier propre** (`failed[]` avec `error_code` **`FIELD_TOO_LONG`**) au lieu d'un 500 global. **Garde-fou** : vérifier qu'aucun test existant n'asserte `DbError::Sqlx(_)` comme comportement attendu pour 1406/1264 (grep ground-truth `crates/kesh-db/` : 0 référence à 1406/1264 dans les tests aujourd'hui → ajout sûr).

## DC6 — Complétion atomique (rappel umbrella, figé)

`repositories::supplier_invoices::create` **possède** le pool et **commit sa propre transaction** → une complétion naïve `create()` (COMMIT) puis `UPDATE staging` laisse une fenêtre non-atomique (crash entre les deux → double facture + double écriture comptable à la re-complétion). **Solution figée** : `create_in_tx(&mut tx, …)` (refactor, AC6) exécuté dans la même transaction que le `SELECT … FOR UPDATE` du staging + l'`UPDATE status='completed'`. Atomique : pas de double facture possible. `create` (pool-owning) reste pour les appels directs 12-2.

## Tasks / Subtasks

- [ ] **T4.1 — Config inbox** : `KESH_INBOX_DIR` (`/data/inbox`) + `KESH_INBOX_MAX_FILE_BYTES`/`MAX_FILES_PER_RUN`/`MAX_PDF_PAGES` dans `Config` + `from_env` (pattern `documents_dir` 12-5b). Câbler `MAX_PDF_PAGES` → `DecodeConfig.max_pages`.
- [ ] **T4.2 — Refactor DC6** : extraire `supplier_invoices::create_in_tx(&mut tx, …)` (miroir `pay_in_tx`), `create` délègue + commit. 0 régression tests 12-2.
- [ ] **T4.3 — Dette D2** : `DbError` variant pour MariaDB 1406/1264 + mapping `map_db_error` ; vérifier non-régression.
- [ ] **T4.4 — Service d'import** : module `inbox_import` (kesh-api) — lecture inbox, verrou run, sécurité (symlink/taille/stabilité/type+magic), hash+doublon (réactive `discarded`), décodage 12-5b, archivage 12-5b, staging, suppression/`failed/`, rapport `{accepted, failed, warnings}`. Constantes `error_code`.
- [ ] **T4.5 — Endpoints** : `routes/imported_supplier_invoices.rs` (nouveau) — `POST /inbox-import`, `GET /imported-supplier-invoices?status=` (valide status, dette D1), `POST /{id}/complete` (DC6 + F2 + F9 + devise), `POST /{id}/discard`, `GET /{id}/source-document` ; + `GET /supplier-invoices/{id}/source-document` (routes/supplier_invoices.rs). **Déclarer** le module dans `routes/mod.rs` (`pub mod imported_supplier_invoices`) **ET câbler** les routes dans **`lib.rs::build_router::comptable_routes`** (l.186-375, pattern supplier_invoices l.237-247) — l'enregistrement Axum réel est dans `lib.rs`, pas `routes/mod.rs` ; un module déclaré mais non câblé compile mais retourne 404. Note routeur : `POST /inbox-import` et `/imported-supplier-invoices/*` sont à des préfixes distincts mais peuvent vivre dans le même module (deux `.route()` enregistrés à leurs chemins respectifs dans `comptable_routes`). RBAC Comptable+ (`require_comptable_role` middleware, `lib.rs:373`) + company-scope partout.
- [ ] **T4.6 — Tests** : intégration import + sécurité + verrou + complétion atomique + réconciliation + download (404/IDOR) + non-régression `create_in_tx`. Quality gate exit-code vérifié.

## Dev Notes

### Ground-truth (vérifié 2026-06-29)

**API 12-5b consommée** (déjà mergée sur la branche umbrella) :
- `crates/kesh-api/src/qr_decode.rs` : `decode_spc_from_image_bytes(bytes: &[u8]) -> Result<Option<ScannedQrBill>, DecodeError>` ; `decode_spc_from_pdf_bytes(bytes: &[u8], cfg: DecodeConfig) -> Result<…>` ; `DecodeConfig { max_pages, max_dimension }` (Default 20/2000) ; `DecodeError::{ImageDecode(String), PdfRender(String), InvalidSpcPayload(QrBillError), InvalidIban(String)}`. **Sérialisation pdfium déjà gérée** (`PDFIUM_LOCK` statique) — mais le **verrou de run** AC5 reste nécessaire (sémantique métier : un seul import à la fois, pas seulement un seul appel pdfium).
- `crates/kesh-api/src/document_storage.rs` : `store_document(documents_dir: &Path, bytes: &[u8], ext: &str, original_filename: &str, mime_type: &str) -> io::Result<DocumentMeta>` (refuse 0-octet + ext non sûre) ; `read_document(documents_dir, storage_path) -> Result<Vec<u8>, ReadDocumentError>` (`NotFound`/`InvalidPath`/`Io`) ; `mime_for_ext(ext) -> &'static str`.
- `crates/kesh-db/src/entities/imported_supplier_invoice.rs` : `NewImportedSupplierInvoice::from_scanned(company_id, &ScannedQrBill, DocumentMeta)` (mapping complet, `address_type` normalisé {K,S}) ; `DocumentMeta { storage_path, original_filename, sha256, mime_type, byte_size }`.
- `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` : `create`, `find_by_id_scoped(company_id, id)`, `find_by_company_hash(company_id, file_hash)`, `list_by_status(company_id, status)` — tous scopés `company_id`. **`list_by_status` ne valide pas `status`** (dette D1 à traiter à la frontière HTTP). **Fonctions UPDATE à AJOUTER** (le repo 12-5b n'expose aucun UPDATE) : `reactivate_to_complete(&mut tx, company_id, id)` (réactivation `discarded`→`to_complete`, AC3 step 5) ; `mark_completed(&mut tx, company_id, id, supplier_invoice_id)` (complétion, AC7 step 7) ; **`find_by_supplier_invoice_id_scoped(pool/exec, company_id, supplier_invoice_id)`** (résolution du justificatif via `GET /supplier-invoices/{id}/source-document`, AC10) — toutes `tx`/exec-aware (appelées dans la transaction de complétion/import) et scopées `company_id`. **`reactivate_to_complete` DOIT aussi remettre `supplier_invoice_id = NULL`** (`UPDATE … SET status='to_complete', supplier_invoice_id=NULL WHERE id=? AND company_id=?`) : une row `discarded` peut porter un `supplier_invoice_id` résiduel d'une complétion antérieure ; sans le reset, la réactivation laisserait un pointeur vers une ancienne facture qui serait ensuite écrasé à la prochaine complétion (donnée incohérente).
- **Cross-module** : le handler `/complete` (`routes/imported_supplier_invoices.rs`) importe `SupplierInvoiceResponse` depuis `crate::routes::supplier_invoices` (`use crate::routes::supplier_invoices::SupplierInvoiceResponse;`) — ne pas redéfinir (DRY). **`SupplierInvoiceResponse::from_parts` est aujourd'hui `fn` privée (`routes/supplier_invoices.rs:85`)** → la passer **`pub fn`** pour l'appeler depuis le nouveau module (changement mécanique 1 mot, pas architectural).
- `crates/kesh-api/src/config.rs` : `documents_dir` (défaut `/data/documents`) ajouté 12-5b — reproduire le pattern pour `KESH_INBOX_DIR` (défaut `/data/inbox`).

**Cibles de refactor / extension** :
- `crates/kesh-db/src/repositories/supplier_invoices.rs:212-406` `create` (pool-owning, `pool.begin()` l.239, `tx.commit()` l.406) → extraire `create_in_tx`. **Patron exact** : `pay` (l.457) délègue à `pay_in_tx` (l.490). `find_open_covering_date(&mut tx, …)` l.296, `company_invoice_settings::get_or_create_default_in_tx` l.302 déjà tx-aware.
- `crates/kesh-db/src/errors.rs:151` `map_db_error` `match my_err.number()` gère 1062/1451/1452 → ajouter 1406/1264 (dette D2).
- `crates/kesh-api/src/routes/supplier_invoices.rs` : handlers `list/get/create/pay/cancel` (l.216/234/247/286/310) — **aucun handler complétion ni download** → ajouter `source-document` ici + nouveau module `imported_supplier_invoices`.
- `crates/kesh-api/src/routes/mod.rs:28` `pub mod supplier_invoices` → ajouter `pub mod imported_supplier_invoices` + nest la route.
- `crates/kesh-db/src/entities/supplier_invoice.rs` : `NewSupplierInvoice { company_id, contact_id, supplier_invoice_number, invoice_date: NaiveDate, due_date, creditor_iban, creditor_qr_iban, payment_reference, expected_payment_amount, lines: Vec<NewSupplierInvoiceLine> }`. `NewSupplierInvoiceLine { description (NOT NULL non-vide), quantity, unit_price, vat_rate, expense_account_id }`. **Aucun champ `currency`** (L7 — devise validée service-level).

### Conventions
- **Pattern batch FailedProposal** (CLAUDE.md) : `{accepted, failed, warnings}`, HTTP 200, `file_name` identifiant business, `error_code` constante (jamais `format!`), `details` JSON. Exceptions globales : 401/403 RBAC, 409 run en cours, 500 catastrophe.
- **Migration policy** : aucune nouvelle migration ici (table livrée 12-5b). Si un `DbError` variant est ajouté (D2), pas d'impact migration.
- **Sécurité filesystem** : env (admin), canonicalisation sous racine, symlink rejeté, liste blanche + magic bytes, taille max, stabilité fichier.
- **Multi-tenant** : tout finder/endpoint scopé `company_id` du user courant (anti-IDOR, KF-002).
- **Test Locally First** exit code vérifié (PAS `cargo test | grep`, cf. `feedback_cargo_test_pipe_masks_exit`).
- **Branche** : `story/12-5-import-repertoire-factures` (umbrella).
- **Contrat d'autonomie** : STOP si modification architecturale (le refactor `create_in_tx` est prévu par DC6 figé, donc dans le périmètre).

### Limitations (héritées umbrella)
- **L1** — binaires justificatifs hors `.keshbackup` (métadonnées seules, DC5). Le download après restore métadonnée-seule → 404 (F7). Epic 14.
- **L4** — création inline fournisseur hors scope (sélection contact `is_supplier` existant). UI 12-5d.
- **L5** — seules les factures importées via 12-5 ont un justificatif → `GET .../source-document` 404 pour une facture créée directement 12-2. Epic 14.
- **L6** — pdfium natif non sandboxé ; segfault PDF malformé tue le process (caps en atténuation). Accepté v0.4.
- **L7** — devise non persistée sur `supplier_invoices` ; **v0.4 = CHF uniquement** (EUR rejeté à la complétion, F-OPUS-1/décision Guy 2026-06-30). Support EUR = follow-up nécessitant (a) colonne `currency` sur `supplier_invoices` (évolution 12-2) ET (b) propagation devise à pain.001 (`pain001/mod.rs:25` hardcode `Ccy="CHF"`). Tant que ces deux manquent, accepter EUR = risque de paiement mislabelé CHF.

### References
- [Source: 12-5-import-repertoire-factures.md] — AC2/AC3/AC4/AC5/AC6/AC7/AC10, DC6, schéma, Dev Notes, §Découpage.
- [Source: 12-5b-decodage-entite-stockage.md] — API qr_decode/document_storage/entité/repo consommée + dettes déférées D1/D2.
- [Source: supplier_invoices.rs:212/457/490] — `create` owns-tx, patron `pay_in_tx` pour `create_in_tx`. [errors.rs:151] — map_db_error.
- [Source: routes/supplier_invoices.rs / routes/mod.rs] — enregistrement endpoints.

## Change Log

### Pass 1 — validate (Sonnet 4.6, 3 couches : fidélité API / edge-sécurité-comptable / complétude-conventions), 2026-06-30
Trend > LOW : **~13** (2 HIGH + 11 MEDIUM). Layer A a confirmé par table ground-truth que **toutes** les signatures API 12-5b/12-2 citées sont exactes (fondation solide). Patches appliqués :
- **H1** (Layer C/A) — `create_in_tx` retourne `SupplierInvoiceWithLines` (kesh-db), PAS `SupplierInvoiceResponse` (kesh-api → cycle Cargo) ; handler construit la réponse via `from_parts`.
- **H2** (Layer B) — verrou run : **HTTP 409 figé** + variant `AppError::InboxImportAlreadyRunning` ; impératifs anti-fuite (Option A `GET_LOCK` sur `pool.acquire()` connexion dédiée ; Option B `AtomicBool` + RAII guard Drop).
- **M (filtrage entrées)** — entrées non-fichier (dont `failed/`) silencieusement ignorées (anti-pollution rapport), `symlink_metadata` Err géré.
- **M (collision `failed/`)** — nommage `{stem}_{hash8}.{ext}` (rename POSIX écrase sinon) + création `failed/`.
- **M (réconciliation)** — Σ TTC via `kesh_core::accounting::vat::line_vat_amount` (arrondi par ligne) = `create_in_tx`, anti-drift centime pain.001/QRR.
- **M (canonicalisation NAS)** — canonicaliser `KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR` au chargement (symlink `/data`→`/volume1` Synology casserait l'anti-traversal).
- **M (TOCTOU)** — `O_NOFOLLOW` à l'ouverture.
- **M (`FIELD_TOO_LONG`)** — error_code dédié au catalogue + mapping D2.
- **M (D2 `DbError::Sqlx`)** — `Other` n'existe pas, repli = `Sqlx` ; variant `DataLengthOrRange` depuis 1406/1264 ; garde-fou grep corrigé.
- **M (test D2)** — test non-régression `creditor_name>70` → `FIELD_TOO_LONG` HTTP 200 (AC11).
- **M (`lib.rs::build_router`)** — câblage Axum réel (pas `routes/mod.rs`), pattern l.237-247.
- **M (`is_qr_iban=false`+QRR)** — rejet métier ; `payment_batches.rs:613` ne valide pas en aval.
- **LOW** — `from_fields_for_test`/`make_test_config`, fns repo UPDATE (`reactivate_to_complete`/`mark_completed`), import `SupplierInvoiceResponse`, bornes `warnings`.

### Pass 2 — validate (Haiku 4.5, 3 couches, discipline grep ground-truth), 2026-06-30
Trend > LOW : **6 MEDIUM** réels (après dismiss). Patches Pass 1 tous confirmés en place par grep. Patches Pass 2 :
- **from_parts pub** (Layer A, grep `routes/supplier_invoices.rs:85` `fn` privée) — passer `pub fn` pour appel cross-module.
- **réactivation → `supplier_invoice_id=NULL`** (Layer B) — `reactivate_to_complete` reset le pointeur résiduel (sinon référence vers ancienne facture).
- **taille via `metadata().len()` avant lecture** (Layer B) — anti-DoS mémoire (pas de `fs::read` d'un fichier géant avant le check).
- **`complete` sur staging non-`to_complete` → HTTP 409** + `errorCode IMPORT_NOT_PENDING_COMPLETION` (Layer B).
- **stratégie fixtures QR** (Layer C) — helper `pub(crate)`/`tests/common` via rxing writer OU fixtures commitées `tests/fixtures/` (le helper `#[cfg(test)]` 12-5b n'est pas accessible aux tests d'intégration).
- **mapping `ImageDecode`→`UNSUPPORTED_FILE_TYPE`** (Layer C) — pinné (magic-bytes a déjà filtré ; format reconnu mais corrompu).
- **LOW** : orphelin archivage (nettoyage best-effort/idempotent content-addressed), anti-IDOR explicite 1er download (`find_by_id_scoped`), 404 (row absente) vs 410 (fichier absent), `status` obligatoire, `file_stem()` anti-traversal, `failed/` créé avant boucle.

**Dismiss (grep ground-truth)** : `AppError::Validation` « n'existe pas » (Layer A M-3) **réfuté** — existe `errors.rs:66` ; `DbError::DataLengthOrRange`/`description` non-vide = travail prescrit D2 / comportement hérité 12-2 (hors scope), pas des trous.

### Pass 3 — validate (Opus 4.8, 3 couches, axe architecture/intégrité comptable), 2026-06-30
Layer A (cross-crate) + Layer C (complétude) **CONVERGÉ 0>LOW** (tables ground-truth toutes vertes : `line_vat_amount` accessible kesh-api via kesh-core, type retour kesh-db correct, pas de sous-tx, catalogue error_code complet, couverture T4 totale). **Layer B (intégrité comptable) catch 2 MEDIUM** ratés par Sonnet+Haiku — pattern « Opus catch architectural » :
- **F-OPUS-1** (MEDIUM↗HIGH) — EUR accepté à la complétion mais pain.001 hardcode `Ccy="CHF"` (`pain001/mod.rs:25`) + devise non persistée (L7) → facture EUR mislabelée CHF au paiement. **Décision Guy 2026-06-30 (AskUserQuestion) : CHF UNIQUEMENT en v0.4** (EUR rejeté). Déroge umbrella ∈{CHF,EUR} ; support EUR = follow-up (L7 mis à jour).
- **F-OPUS-2** (MEDIUM) — réconciliation `round2` tolérait un sous-centime alors que `total_amount` est stocké `DECIMAL(19,4)` pleine précision → 3 valeurs en désaccord. **Patch : égalité EXACTE `Σ TTC == staging.amount`** (rejette les lignes sous-centime, garantit `total_amount==staging.amount==expected_payment_amount` par construction).
- **LOW** (Layer A) : signature `create_in_tx(tx, new, user_id)` (company_id depuis `new`, pas param séparé) ; ajout finder `find_by_supplier_invoice_id_scoped` ; dérive refs `from_fields_for_test:~388`.

Splitting préventif : **pas de re-split** (convergence en 3 passes < seuil 4 ; 2 crates ≈ 5 modules ; split casserait la cohésion DC6). Reco dev : T4.2/T4.3 (refactor create_in_tx + D2) en tête, tests verts, avant T4.4/T4.5.

### Pass 4 — validate (Sonnet 4.6, 3 couches, convergence), 2026-06-30
Layer A + Layer B **CONVERGÉ 0>LOW** (ground-truth tous exacts : `pain001/mod.rs:25` CCY="CHF", `line_vat_amount` vat.rs:39, `from_parts:85` privée, `AppError::Validation`→400 errors.rs:726 ; F-OPUS-1/F-OPUS-2 cohérents sur toutes les sections, aucune mention résiduelle `round2`/EUR-accepté). **Layer C : 1 MEDIUM** (F4-1) patché :
- **F4-1** (MEDIUM) — test D1 (validation `status` → 400) absent d'AC11 → ajouté.
- **LOW** (3 couches convergentes) : `errorCode` canoniques des rejets `/complete` steps 3/4/6 (`CURRENCY_NOT_SUPPORTED`/`IBAN_REFERENCE_MISMATCH`/`AMOUNT_MISMATCH`, HTTP 400) ; note faux-négatif parser >2 décimales (hors-SPC) ; **sync umbrella** (note de dérogation v0.4 CHF-only + égalité exacte en tête de `12-5-import-repertoire-factures.md` pour le dev 12-5d).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
