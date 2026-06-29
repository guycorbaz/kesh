# Story 12.5: Import de factures fournisseurs depuis un répertoire surveillé

Status: ready-for-dev

<!-- SPEC UMBRELLA — split figé 12-5a..d (cf. §Découpage). TOUS les DC figés (DC1 pdfium, DC2 bouton manuel, DC3 staging, DC4 lien côté import, DC5 métadonnées seules) — Guy 2026-06-28/29. -->

## Story

As a comptable PME utilisant Kesh sur mon NAS,
I want déposer les factures fournisseurs (PDF/images) dans un dossier que Kesh lit, et que Kesh les importe en lot en lisant leur QR-code, en conservant une copie du fichier associée à chaque facture,
so that je n'ai pas à ressaisir chaque facture ni à classer les fichiers à la main, et je retrouve la pièce justificative en un clic depuis le paiement.

## Contexte & source

- Issue GitHub **#194** (créée 2026-06-28) — priorisée par Guy par rapport au scan manuel (12-4, différé).
- Complément de **#191** (Factures fournisseurs & paiements ; 12-2 + 12-3 mergées). Réutilise l'entité `supplier_invoices` (12-2) et alimente la liste des paiements pain.001 (12-3).
- Exigence Guy 2026-06-28 : **la copie du fichier facture doit être stockée et associée** au paiement/facture (justificatif récupérable rapidement).
- Cible : **v0.4**.

## Objet métier

Un **dossier d'import** (inbox, sur le serveur/NAS) que l'admin configure. L'utilisateur y dépose des fichiers de factures fournisseurs (PDF/image porteurs d'un Swiss QR Code). Kesh **lit le dossier**, **décode le QR côté serveur**, **parse le payload SPC** (coordonnées créancier/montant/référence), **crée des factures importées « à compléter »** (staging), **archive le fichier source** dans Kesh et l'**associe** au staging (téléchargeable depuis le détail après complétion). À l'import réussi, **le fichier original est supprimé de l'inbox** (la copie archivée suffit, décision Guy) ; les échecs sont déplacés en `failed/`.

## Acceptance Criteria

**Parseur SPC (socle — kesh-qrbill) — 12-5a**

1. `parse_spc_payload(text: &str) -> Result<ScannedQrBill, QrBillError>` parse un payload SPC, **inverse exact de `generator.rs::build_payload`** (`crates/kesh-qrbill/src/generator.rs:14-90`, ordre des lignes SPC SIX 2.2 §3), valide IBAN/QR-IBAN/QRR via `validation.rs`. Le parseur est **le socle commun** avec le scan manuel 12-4 différé.

   **Robustesse factures tierces** (le QR provient de logiciels tiers, pas seulement de Kesh) :
   - **En-tête** : rejeter si la 1ʳᵉ ligne ≠ `SPC` ou la 2ᵉ ≠ `0200` → `QrBillError` (mappé `INVALID_SPC_PAYLOAD` en aval).
   - **Bloc adresse créancier (type K ou S)** : lire le 1ᵉʳ champ du bloc (`AdrTp`). Type **K** (Combined) : `name, line1, line2, "" , "", country` (PstCd/TwnNm vides). Type **S** (Structured) : `name, street, building_no, postal_code, town, country`. Le parseur DOIT gérer les **deux** (Kesh n'émet que K, mais les fournisseurs tiers émettent souvent S). Décalage d'index par type.
   - **Nombre de lignes variable** : tolérer **31, 32 ou 34 lignes**. Repérer le trailer `EPD` (champ AddInf). `billing_information` (après `EPD`) peut être **absent** (31 l.) ou présent (32 l.). **AltPmtInf** (2 lignes après billing_information, 34 l.) émis par certains logiciels tiers : lire si présent, sinon ignorer. Robuste aux champs de fin absents (ne pas indexer en dur sur 32).
   - **Bloc débiteur final** (7 lignes) : peut être entièrement vide → `ultimate_debtor = None` (non requis pour l'import : on n'a besoin que du créancier + montant + référence).

   **Définition `ScannedQrBill`** (nouveau type, `kesh-qrbill/src/parser.rs`, exporté dans `lib.rs` ; noms de champs indicatifs) :
   ```rust
   pub struct ScannedQrBill {
       pub creditor_iban: String,          // champ 4 du payload (IBAN ou QR-IBAN, 21 car. sans espaces)
       pub is_qr_iban: bool,               // détecté via IID 30000–31999 (validation.rs)
       pub creditor: ScannedAddress,
       pub amount: Option<Decimal>,        // SPC autorise montant vide (open amount) → Option
       pub currency: String,               // "CHF" | "EUR" (autres tolérées au parse, validées en aval)
       pub reference: ScannedReference,
       pub unstructured_message: Option<String>,
       pub billing_information: Option<String>,
   }
   pub struct ScannedAddress {
       pub address_type: char,             // 'K' ou 'S'
       pub name: String,
       pub street_or_line1: String,
       pub building_or_line2: String,
       pub postal_code: Option<String>,    // type S uniquement
       pub town: Option<String>,           // type S uniquement
       pub country: String,
   }
   pub enum ScannedReference { Qrr(String), Scor(String), None }
   ```
   `amount` non parsable (présent mais malformé) → `QrBillError`. `amount` vide → `None`. La validation QRR (mod-10 récursif) et IBAN/QR-IBAN réutilise `validation.rs` (ne pas dupliquer).

**Décodage serveur & lecture du dossier (kesh-api / nouveau module) — 12-5b + 12-5c**

2. Un **dossier inbox configurable** (`KESH_INBOX_DIR`, pattern env var comme `KESH_ADMIN_BACKUP_DIR` cf. `config.rs`) est lu **sur déclenchement manuel** (DC2 : bouton « Importer le dossier », PAS de watch auto en v0.4). Chaque fichier est décodé **côté serveur** : **PDF → rendu page via `pdfium-render` → `image::DynamicImage` → `rxing` (QR→texte)** ; image PNG/JPG → `rxing` directement → texte → `parse_spc_payload`. **Multi-page** : tenter chaque page jusqu'à trouver un QR SPC valide. **Plusieurs QR sur une page** : tenter chaque QR retourné par `rxing` dans l'ordre jusqu'au premier payload commençant par `SPC\n0200` ; si aucun n'est SPC, la page ne compte pas comme QR trouvé.
3. **Pattern batch / rapport per-fichier** (contrat `FailedProposal` CLAUDE.md) : réponse `{ accepted: [...], failed: [...] }`, **HTTP 200** même en succès partiel. Chaque fichier réussi → staging créé + fichier associé. Chaque fichier en échec → entrée `failed[]` avec **identifiant business** = `file_name: String` (PAS d'index positionnel), `error_code: String` (constante canonique, jamais `format!`), `details: Option<serde_json::Value>`. Aucun fichier ne fait planter l'import global. **Catalogue `error_code`** (constantes) :
   - `UNSUPPORTED_FILE_TYPE` — extension hors liste blanche.
   - `FILE_TOO_LARGE` — dépasse `KESH_INBOX_MAX_FILE_BYTES`.
   - `SYMLINK_REJECTED` — entrée d'inbox qui est un symlink (cf. AC4).
   - `DUPLICATE` — hash SHA-256 déjà archivé (cf. AC6).
   - `NO_QR_CODE_FOUND` — aucun QR SPC détecté (toutes pages/QR essayés).
   - `INVALID_SPC_PAYLOAD` — QR décodé mais pas un payload SPC valide.
   - `INVALID_IBAN` — IBAN/QR-IBAN invalide (validation.rs).
   - `PDF_RENDER_ERROR` — pdfium échoue à rendre le PDF.
   - `FILE_READ_ERROR` — erreur I/O (incl. fichier tronqué/instable, cf. AC4).
4. **Sécurité** (12-5c) :
   - Chemins maîtrisés par l'admin via env (pas d'upload réseau arbitraire).
   - **Symlinks rejetés** : `symlink_metadata().is_symlink()` sur chaque entrée d'inbox → `SYMLINK_REJECTED`, jamais suivi/ouvert.
   - **Anti-traversal** au déplacement (`processed`/`failed`) ET au nommage du fichier archivé : ne jamais réutiliser le nom d'origine pour construire un chemin ; nom archivé = `{sha256hex}.{ext}` (cf. AC5/F19). Vérifier que le chemin canonicalisé reste sous `KESH_INBOX_DIR` / `KESH_DOCUMENTS_DIR`.
   - **Liste blanche d'extensions** (`pdf`, `png`, `jpg`, `jpeg`) **+** sanity sur magic bytes (PDF `%PDF`, PNG/JPEG signatures) ; extension seule insuffisante.
   - **Taille max** `KESH_INBOX_MAX_FILE_BYTES` (défaut documenté, ex. 25 Mo) → `FILE_TOO_LARGE`.
   - **Nombre max de fichiers par déclenchement** `KESH_INBOX_MAX_FILES_PER_RUN` (défaut, ex. 200) : au-delà, les fichiers excédentaires sont **ignorés** ce tour (ni `accepted` ni `failed`, retraités au prochain déclenchement) et un avertissement est ajouté au rapport. Évite un body de réponse `failed[]` non borné (F-NEW-7) et un run trop long.
   - **Fichier en cours d'écriture** : avant traitement, vérifier la **stabilité** (taille + mtime identiques sur 2 lectures espacées d'un court délai) ; si instable → ignorer ce tour (ni `accepted` ni `failed`, retenté au prochain déclenchement) OU `FILE_READ_ERROR` si lecture impossible. Documenter le choix retenu au dev.
   - Lecture seule du reste du FS (jamais d'écriture hors `KESH_DOCUMENTS_DIR` / `processed` / `failed`).
   - **Robustesse rendu pdfium (F4)** — les PDF viennent de tiers (surface d'attaque) : (a) **cap pages rendues** `KESH_INBOX_MAX_PDF_PAGES` (défaut ex. 20 ; au-delà sans QR → `PDF_RENDER_ERROR`) — évite le DoS d'un PDF à 10 000 pages ; (b) **clamp dimensions** de rendu (DPI / pixels max bornés) — évite un MediaBox gigapixel sur un PDF de 1 Mo ; (c) `pdfium` = lib **native in-process** : un segfault sur PDF malformé **tue le process API** (`catch_unwind` ne rattrape pas un SIGSEGV natif) → **L6** (risque accepté v0.4, input semi-contrôlé admin ; follow-up sandbox/sous-process).
   - **Sérialisation des runs (F6)** : DC2 = bouton manuel mais deux déclenchements concurrents (2 onglets, double-clic) sont possibles → **verrou de run** (`GET_LOCK` MariaDB ou flag global) ; si un import est déjà en cours → 409 / avertissement, pas de double traitement. À l'INSERT staging, mapper une **violation `UNIQUE (company_id, file_hash)`** en `failed[] DUPLICATE` (HTTP 200, PAS d'`AppError` 500 — pattern FailedProposal) ; tolérer `ENOENT` à la suppression inbox (idempotent, un fichier déjà traité ne fait pas échouer le run).

**Stockage & association du fichier (justificatif) — 12-5b (schéma) + 12-5c (I/O)**

5. La **copie du fichier source** est stockée dans `KESH_DOCUMENTS_DIR` (filesystem, hors DB) sous le nom **`{sha256hex}.{ext}`** (pas de collision, pas de traversal ; nom d'origine conservé **en DB uniquement**). Le lien persistant porte : `storage_path` (relatif à `KESH_DOCUMENTS_DIR`), `original_filename`, `sha256` (hex), `mime_type`, `byte_size`. **DC4 figé (lien côté import)** : ces colonnes vivent sur `imported_supplier_invoices` (PAS d'ALTER de `supplier_invoices` 12-2). Récupérable via `GET /api/v1/supplier-invoices/{id}/source-document` qui résout `imported_supplier_invoices WHERE supplier_invoice_id = {id}` (download/consultation) depuis le détail de la facture fournisseur (12-2) **après complétion**, et `GET /api/v1/imported-supplier-invoices/{id}/source-document` avant complétion. **RBAC + anti-IDOR** : endpoints réservés Comptable+ et **scopés `company_id` du user courant** (un user ne peut pas télécharger le justificatif d'une autre company — `WHERE company_id = {current}`). **404 si row absente** (facture créée directement via 12-2, cf. L5). **Fichier disque absent alors que la row existe (F7)** — conséquence directe de L1 après un restore métadonnée-seule : `std::fs::read(storage_path)` → `ENOENT` doit retourner **404 (ou 410 Gone)** « justificatif non disponible (fichier non restauré, cf. L1) », **jamais 500**. Test dédié (restore métadonnée → download → assert 404, pas 500).
6. À l'import **réussi**, le fichier original est **SUPPRIMÉ de l'inbox** (décision Guy : la copie archivée AC5 est la source de vérité). Les fichiers en **échec** sont déplacés vers `failed/` (conservés, jamais supprimés). **Idempotence scopée par company (F-NEW-3)** : un fichier dont le hash SHA-256 est **déjà archivé dans la même company** (clé composite `UNIQUE (company_id, file_hash)`) → `failed[] DUPLICATE`, NON ré-importé, déplacé en `failed/`. Deux companies distinctes PEUVENT importer le même fichier (byte-identique) indépendamment. Le hash est calculé **avant** décodage QR (court-circuit doublon). **Re-dépôt après `discarded` (F5)** : si la row matchant `(company_id, file_hash)` est en statut **`discarded`** (écartée par erreur), le ré-import la **réactive en `to_complete`** (au lieu de `failed[] DUPLICATE`) — évite de piéger à vie un fichier écarté par mégarde (MariaDB ne supporte pas d'UNIQUE partiel → géré applicativement). Une row `to_complete` ou `completed` matchant le hash → `DUPLICATE` (vrai doublon).

**Création des factures importées (staging) & complétion — 12-5b (entité) + 12-5d (UI)**

7. Pour chaque fichier valide, une **facture importée « à compléter »** (`imported_supplier_invoices`, statut `to_complete`) est créée à partir des coordonnées QR (créancier nom/IBAN/QR-IBAN, référence, montant TTC, devise) + lien fichier archivé. **DC3 figé.** L'utilisateur la **complète** dans l'UI :
   - **Fournisseur** : **sélection d'un contact existant** avec `is_supplier = true` (la création inline d'un fournisseur est **hors scope 12-5** ; si absent, l'utilisateur le crée via le flux Contacts existant puis revient — documenté UX).
   - **Adresse créancier (type S)** : l'adresse parsée du QR (colonnes `creditor_*` du staging) est affichée **à titre informatif** pour aider à identifier/choisir le contact. Après sélection, **l'adresse du contact existant prime** (source de vérité comptable) ; la facture créée utilise IBAN/référence/montant du QR + les lignes saisies, indépendamment de l'adresse affichée (F-NEW-6).
   - **`invoice_date`** : **saisie obligatoire à la complétion** (date-picker). Le payload SPC **ne contient aucune date** → non pré-remplissable (cf. F6). `supplier_invoices::create` exige une `invoice_date` non-optionnelle couverte par un exercice ouvert.
   - **Lignes** : compte de charge (`expense_account_id`) + HT + TVA — **saisis à la complétion** (le QR ne fournit ni compte ni TVA). Le pré-remplissage couvre IBAN/référence/montant TTC/devise (partie error-prone).
   - **Réconciliation montant (F2)** : pain.001 (12-3) paie le `total_amount` **calculé** des lignes (`payment_batches.rs:298`), PAS le montant du QR. Pour éviter de payer le créancier à un montant ≠ du justificatif/QRR : si `staging.amount` (montant QR) est **présent**, la complétion **exige** `Σ(lignes TTC) == staging.amount` (tolérance arrondi `0.00`) → sinon **bloquer** avec « Le total des lignes (X) ne correspond pas au montant du QR (Y) ». Renseigner aussi `NewSupplierInvoice.expected_payment_amount = staging.amount` (piste d'audit). Si `staging.amount` absent (montant ouvert), pas de contrôle.
   - **Routage IBAN (F9)** : `is_qr_iban=true` → renseigner `creditor_qr_iban` (et **exiger `reference_type='QRR'`** — un QR-IBAN sans QRR est rejeté à la complétion, cohérent `payment_batches.rs:613`) ; `is_qr_iban=false` → `creditor_iban`. (`NewSupplierInvoice` a deux champs distincts `creditor_iban`/`creditor_qr_iban`, `supplier_invoice.rs:77-78`.)
   - À la validation → **complétion atomique** (DC6 : `create_in_tx` + transition staging dans **une seule transaction** sous `FOR UPDATE`) → staging passe `completed`, facture `open` avec **fichier source associé** (`supplier_invoice_id`) → entre dans la liste des paiements (12-3).
   - **Échec de `create` à la complétion** : le staging **reste `to_complete`**, message d'erreur UX explicite, **aucune écriture comptable partielle**. Messages (non-exhaustif, F-NEW-9) : `DbError::FiscalYearInvalid` (`invoice_date` hors exercice ouvert) → « Aucun exercice fiscal ouvert couvrant cette date » ; devise ≠ CHF/EUR → « Devise non supportée (CHF ou EUR) » ; montant ≤ 0 → « Le montant doit être positif ».
   - Possibilité d'**écarter** (`discarded`) une facture importée non pertinente (le fichier archivé est conservé ; pas de suppression du justificatif en v0.4).

**Frontend & doc — 12-5d**

8. Un écran **« Importer le dossier »** (Comptable+) déclenche l'import et affiche le **rapport** (créées / échecs avec `file_name` + `error_code` traduit). Une vue liste des **factures importées** (`to_complete`) avec action **Compléter** (formulaire fournisseur+date+lignes) et **Écarter**. Le détail d'une facture fournisseur (12-2) affiche un lien **« Voir la facture d'origine »** (download du justificatif) **si** un staging `completed` la référence. i18n FR + clés (DE/IT/EN = clés ajoutées, valeurs FR provisoires si pas de traduction — cohérent politique i18n projet).
9. **Tests** : unitaires parseur SPC — **round-trip `build_payload`↔`parse_spc_payload` pour type K uniquement** (Kesh n'émet que K) ; **type S testé via fixture SPC construite à la main** (string SPC type S valide → parser → vérifier `address_type='S'`, `postal_code`/`town` remplis ; pas de round-trip possible sans generator type S, F-NEW-4) ; 31/32/34 lignes ; montant vide ; QRR/NON + décodage image (fixture QR PNG) + rendu PDF (fixture PDF 1 page avec QR ; multi-page) ; intégration import (dossier temp avec fixtures → staging + fichiers associés + rapport `failed`) ; sécurité (path traversal, symlink, type refusé, magic-bytes, doublon hash, taille max) ; complétion (`to_complete`→`completed`, échec exercice fermé, `discarded`) ; E2E selon DC. **Doc** : `.env.example` (`KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR`/`KESH_INBOX_MAX_FILE_BYTES` + défauts + mapping docker-compose volumes), `docker-compose.yml` (volumes `/data/inbox` + `/data/documents`), manuel admin (config dossiers + pdfium), CHANGELOG, README. **Compteurs en dur** : nouvelle table `imported_supplier_invoices` → bumper `admin_full_export_e2e` (`data_count`) + `migrations_upgrade_path` + `TABLES_TO_TRUNCATE` + manifeste export (17-3) + audit idempotence `docs/migrations-idempotence-audit.md`. **Ordre `TABLES_TO_TRUNCATE` (F10)** : la liste est enfants→parents, le restore insère en ordre inverse (`backup.rs:432` `.rev()`). Placer `imported_supplier_invoices` **avant** `supplier_invoices` ET `companies` (zone enfants, en tête) pour qu'au restore elle soit insérée **après** ses parents (FK `company_id` RESTRICT NOT NULL satisfaite ; `supplier_invoice_id` SET NULL nullable moins contraignant). `backup_inventory_matches_schema` attrape l'oubli de table mais PAS un mauvais ordre FK. Test Locally First **exit code vérifié** (PAS `cargo test | grep`).

**Endpoints backend complétion (12-5c — F3)**

10. La logique de complétion est **backend (12-5c), pas frontend**. Endpoints (Comptable+, company-scopés) — `routes/supplier_invoices.rs` n'a aujourd'hui que `list`/`create`/`pay`/`cancel`, **aucun handler de complétion** :
    - `GET /api/v1/imported-supplier-invoices?status=to_complete` — liste des factures importées à compléter (AC8).
    - `POST /api/v1/imported-supplier-invoices/{id}/complete` — corps = `contact_id` + `invoice_date` + lignes. Siège de **DC6 (transaction atomique)** + **réconciliation montant (AC7/F2)** + **routage IBAN (AC7/F9)**. Retourne la facture `open` créée.
    - `POST /api/v1/imported-supplier-invoices/{id}/discard` — transition `discarded` (AC7).
    Le frontend 12-5d **consomme** ces endpoints (ne réimplémente PAS la logique transactionnelle). 12-5c = « Service d'import + endpoints import **et complétion** + sécurité ».

## Points de décision (DC) — TOUS FIGÉS

- **DC1 — Décodage des PDF : rendu pdfium** [✅ FIGÉ Guy 2026-06-28] : factures = **PDF** (e-mail ou scans papier). Décodage côté serveur **par rendu de la page PDF → image** via **`pdfium-render`** (binaire natif `pdfium`), puis QR via `rxing`. Couvre tous les cas (vectoriel/raster + scans CCITTFax). Pipeline : PDF → `pdfium-render` (page→`image::DynamicImage`) → `rxing` (QR→texte) → `parse_spc_payload`. Images PNG/JPG directes aussi (rxing sans pdfium). `rxing` passe **dev-dep → runtime** ; `image` à **déclarer explicitement** (cf. F4).
- **DC1-bis — Packaging Docker pdfium** [✅ FIGÉ Guy 2026-06-29] : **`linux/amd64` uniquement** (cohérent `release.yml` actuel ; le NAS de Guy exécute les images amd64). **Pas de multi-arch arm64** en v0.4 (follow-up, L3). Vérifier la **licence pdfium** (Apache-2.0 / BSD-3, OK) et la mentionner dans la doc. **Détails packaging (F-NEW-5)** dans le stage `runtime` (`debian:bookworm-slim`, Dockerfile:18) :
  - Télécharger le `.so` depuis une **release épinglée** de `bblanchon/pdfium-binaries` (tag `chromium/NNNN` figé, pas `latest`) — fichier `pdfium-linux-x64.tgz` → `libpdfium.so`.
  - **Vérifier le checksum** (SHA-256 du tgz épinglé) avant extraction.
  - Placer dans `/usr/local/lib/libpdfium.so` et `ldconfig` (chemin standard, pas besoin de `LD_LIBRARY_PATH`) — OU `/app/lib/libpdfium.so` + `ENV LD_LIBRARY_PATH=/app/lib:$LD_LIBRARY_PATH`. `pdfium-render` résout via le loader système ; le dev confirmera le nom attendu (`libpdfium.so`).
  - Impact taille image (~plusieurs Mo) acceptable. Documenter dans le manuel admin.
- **DC2 — Déclenchement : bouton manuel** [✅ FIGÉ Guy 2026-06-29] : bouton « Importer le dossier » (manuel). **PAS de watch auto** (inotify/polling) en v0.4 — follow-up.
- **DC3 — Modèle d'entité : staging « à compléter »** [✅ FIGÉ Guy 2026-06-28] : table `imported_supplier_invoices` (coordonnées QR parsées + lien fichier archivé + `supplier_invoice_id` nullable + statut `to_complete`/`completed`/`discarded`). Découple ingestion et comptabilisation, **préserve l'intégrité comptable** de 12-2.
- **DC4 — Stockage justificatif : lien côté import** [✅ FIGÉ Guy 2026-06-29] : `KESH_DOCUMENTS_DIR` (filesystem, hors DB) + **colonnes document sur `imported_supplier_invoices`** (`storage_path`, `original_filename`, `sha256`, `mime_type`, `byte_size`) + **FK nullable `supplier_invoice_id` → `supplier_invoices.id`** (renseignée à la complétion). **AUCUN ALTER de `supplier_invoices`** (12-2) → migration `CREATE TABLE` **non-breaking** (pas de bump `kesh_version_min_required`). Le détail facture (12-2) résout le justificatif via `imported_supplier_invoices WHERE supplier_invoice_id = {id}`. Base minimale ; Epic 14 « Justificatifs » généralisera.
- **DC5 — Inclusion backup/export : métadonnées seules** [✅ FIGÉ Guy 2026-06-29] : la **métadonnée** (table `imported_supplier_invoices`, incl. `storage_path`/`sha256`) entre dans le `.keshbackup` (17-3, via manifeste export). Le **binaire des fichiers** (`KESH_DOCUMENTS_DIR`) reste **HORS backup v0.4** → limitation **L1** documentée. Le restore ne restaure pas les fichiers physiques.
- **DC6 — Complétion atomique staging ↔ `create` (Option A)** [✅ FIGÉ orchestrateur 2026-06-29, précédent `pay_in_tx`] : **PROBLÈME** (F1 CRITICAL) — `repositories::supplier_invoices::create` (`supplier_invoices.rs:212`) possède le pool (`pool.begin()` l.239 / `tx.commit()` l.406) et **commit sa propre transaction**. Une complétion naïve `create()` (COMMIT) **puis** `UPDATE staging SET completed` laisse une **fenêtre non-atomique** : crash entre les deux → facture + écriture d'achat postées mais staging encore `to_complete` → l'utilisateur **re-complète** → **double facture + double écriture comptable** (corruption du grand livre : double 1171/2000/charge). **SOLUTION figée** : extraire **`supplier_invoices::create_in_tx(&mut tx, …)`** (refactor du cœur de `create`, **exactement le pattern `pay`/`pay_in_tx` l.457/490 déjà présent**) et exécuter la complétion dans **UNE seule transaction** : `SELECT … FROM imported_supplier_invoices WHERE id=? FOR UPDATE` (garde anti-double) → si `status != 'to_complete'` abandon → `create_in_tx` → `UPDATE staging SET status='completed', supplier_invoice_id=?` → `commit`. Atomique : pas de double facture possible. `create` (pool-owning) reste pour les appels directs 12-2.

## Schéma `imported_supplier_invoices` (T3 — 12-5b)

Migration `CREATE TABLE` **non-breaking** (pas de bump `kesh_version_min_required`). Conventions calquées sur `20260628000001_supplier_invoices.sql` : multi-tenant `company_id`, statut via **`CHECK` texte** (PAS d'enum SQLx — cf. `feedback_sqlx_mysql_gotchas`), `version INT`, `DATETIME(3)`.

| Colonne | Type | Null | Notes |
|---|---|---|---|
| `id` | BIGINT PK AUTO_INCREMENT | non | |
| `company_id` | BIGINT | non | **FK `companies(id)` ON DELETE RESTRICT — multi-tenant (F-NEW-1)** |
| `status` | VARCHAR(16) DEFAULT `'to_complete'` | non | CHECK IN (`to_complete`,`completed`,`discarded`) |
| `supplier_invoice_id` | BIGINT | **oui** | FK `supplier_invoices(id)` ON DELETE SET NULL — renseigné à la complétion (DC4) |
| `file_hash` | CHAR(64) | non | SHA-256 hex (idempotence) |
| `storage_path` | VARCHAR(512) | non | relatif à `KESH_DOCUMENTS_DIR`, = `{sha256hex}.{ext}` |
| `original_filename` | VARCHAR(255) | non | nom d'origine (affichage seul) |
| `mime_type` | VARCHAR(100) | non | |
| `byte_size` | BIGINT | non | |
| `creditor_iban` | VARCHAR(34) | non | IBAN ou QR-IBAN |
| `is_qr_iban` | BOOLEAN | non | IID 30000–31999 |
| `creditor_address_type` | CHAR(1) | non | `K` ou `S` |
| `creditor_name` | VARCHAR(70) | non | |
| `creditor_line1` | VARCHAR(70) | oui | line1 (K) / street (S) |
| `creditor_line2` | VARCHAR(70) | oui | line2 (K) / building_no (S) |
| `creditor_postal_code` | VARCHAR(16) | oui | type S |
| `creditor_town` | VARCHAR(35) | oui | type S |
| `creditor_country` | CHAR(2) | non | |
| `reference_type` | VARCHAR(8) | non | CHECK IN (`QRR`,`SCOR`,`NON`) |
| `reference_value` | VARCHAR(40) | oui | vide si `NON` |
| `amount` | DECIMAL(19,4) | **oui** | SPC autorise montant vide (F2/AC1) |
| `currency` | VARCHAR(3) | non | validée CHF/EUR à la complétion |
| `unstructured_message` | VARCHAR(140) | oui | |
| `billing_information` | VARCHAR(140) | oui | |
| `version` | INT DEFAULT 1 | non | |
| `created_at` | DATETIME(3) DEFAULT CURRENT_TIMESTAMP(3) | non | |
| `updated_at` | DATETIME(3) … ON UPDATE | non | |

**Index / contraintes** :
- `UNIQUE uq_imported_company_hash (company_id, file_hash)` — **idempotence scopée par company (F-NEW-3)**.
- `INDEX idx_imported_company_status (company_id, status)` — liste des factures à compléter.
- `INDEX idx_imported_supplier_invoice (supplier_invoice_id)` — lookup justificatif (AC5).

## Limitations documentées

- **L1** — Les fichiers justificatifs binaires (`KESH_DOCUMENTS_DIR`) ne sont **PAS** inclus dans le `.keshbackup`. Seules les métadonnées (`storage_path`, `original_filename`, `sha256`, `mime_type`) sont exportées. Le restore d'une installation ne restaure pas les fichiers physiques. **Remédiation** : Epic 14 « Justificatifs » généralisera le backup des pièces. (DC5)
- **L2** — Pas de **watch automatique** de l'inbox en v0.4 (déclenchement manuel par bouton). Follow-up. (DC2)
- **L3** — `linux/amd64` uniquement (pas d'image arm64). Follow-up si déploiement sur cible ARM. (DC1-bis)
- **L4** — Création **inline d'un fournisseur** hors scope de la complétion 12-5 (sélection d'un contact `is_supplier` existant ; sinon créer via Contacts puis revenir). (AC7)
- **L5** — En v0.4, **seules les factures importées via 12-5** ont un justificatif stocké/téléchargeable (lien porté par `imported_supplier_invoices`). Une facture créée **directement** via 12-2 (`supplier_invoices::create` hors import) n'a pas de lien justificatif → `GET .../source-document` renvoie **404**. **Remédiation** : Epic 14 « Justificatifs » généralisera le stockage à toutes les factures. (DC4, F-NEW-8)
- **L6** — `pdfium` est une lib **native in-process** (non sandboxée en v0.4). Un PDF malformé provoquant un segfault natif **tue le process API** (non rattrapable par `catch_unwind`). Risque **accepté v0.4** (inbox semi-contrôlée par l'admin, caps pages/dimensions en place). **Remédiation** : follow-up rendu en sous-process/sandbox isolé. (DC1, F4)

## Tasks / Subtasks

- [ ] **T1 (12-5a) — Parseur SPC (kesh-qrbill)** : `src/parser.rs` `parse_spc_payload` + `ScannedQrBill`/`ScannedAddress`/`ScannedReference` exportés `lib.rs` ; gestion type K/S, 31/32/34 lignes, montant vide ; validations IBAN/QR-IBAN/QRR via `validation.rs` ; tests round-trip `build_payload`↔parser.
- [ ] **T2 (12-5b) — Décodage serveur** : ajouter `rxing` + `image` + `pdfium-render` aux `[dependencies]` de kesh-api ; module décodage (PNG/JPG via rxing ; PDF via pdfium-render multi-page ; multi-QR par page). Bundle pdfium dans `Dockerfile` (amd64). Tests fixtures (PNG QR + PDF QR).
- [ ] **T3 (12-5b) — Entité + stockage** : migration `CREATE TABLE imported_supplier_invoices` (colonnes QR + document + `supplier_invoice_id` FK nullable + statut + index) + stockage `KESH_DOCUMENTS_DIR` (`{sha256hex}.{ext}`) + hash SHA-256. Compteurs en dur + audit idempotence + manifeste export 17-3 + TABLES_TO_TRUNCATE.
- [ ] **T4 (12-5c) — Service d'import + complétion (kesh-api)** : (import) lecture inbox + verrou run + court-circuit doublon hash (réactive `discarded`) + boucle décodage + création staging + déplacement `failed/` / suppression succès + rapport batch `{accepted, failed}` (HTTP 200, UNIQUE→DUPLICATE). (complétion) **`supplier_invoices::create_in_tx`** (refactor DC6) + endpoints `POST .../inbox-import`, `GET/POST .../imported-supplier-invoices` (liste + `{id}/complete` atomique + réconciliation F2 + routage IBAN F9 + `{id}/discard`), `GET .../imported-supplier-invoices/{id}/source-document`, `GET .../supplier-invoices/{id}/source-document` (404 si fichier absent). Sécurité (symlink, traversal, taille, types+magic, stabilité fichier, caps pdfium pages/dimensions, RBAC+company-scope).
- [ ] **T5 (12-5d) — Frontend** : écran « Importer le dossier » + rapport (file_name + error_code i18n) + liste factures importées `to_complete` + formulaire complétion (fournisseur existant + `invoice_date` + lignes → `supplier_invoices::create`) + action Écarter + lien « Voir la facture d'origine » sur le détail 12-2. i18n FR + clés.
- [ ] **T6 (12-5d) — Doc + tests** : `.env.example` (3 env vars + défauts) + `docker-compose.yml` (volumes) + manuel admin (dossiers + pdfium + licence) + CHANGELOG + README. Quality gate Test Locally First exit-code vérifié.

## Découpage

**Split figé** (story large = nouveau sous-système serveur + dép native + migration + UI ; cohérent §"Règle de splitting préventif" CLAUDE.md, scope cross-cutting > 5 modules : kesh-qrbill, kesh-api, kesh-db, frontend, Docker/CI) :

- **12-5a — Parseur SPC (kesh-qrbill)** : story-zéro qui pose le socle (T1). Réutilisable par 12-4 différé. Aucune dép native, testable en isolation (round-trip). **À implémenter en premier.**
- **12-5b — Décodage serveur + entité + stockage** : pdfium/rxing/image + migration `imported_supplier_invoices` + `KESH_DOCUMENTS_DIR` (T2+T3). Dépend de 12-5a. **Note (F8)** : T2 (dép native pdfium + Docker) et T3 (migration/entité/stockage) ont des modes d'échec disjoints et aucune dépendance mutuelle → **split optionnel 12-5b1 (migration+entité+stockage) / 12-5b2 (pdfium+rxing+decode+Docker)** si la passe de review devient hétérogène ; sinon garder ensemble (dérogation assumée, < 5 modules).
- **12-5c — Service d'import + endpoints import ET complétion + sécurité** : lecture inbox + rapport batch + sécurité + verrou run + download (T4) **ET endpoints liste/complétion/écarter backend** (AC10, siège de DC6/F2/F9 — transaction atomique + réconciliation). Dépend de 12-5b.
- **12-5d — Frontend + doc** : UI import (rapport) + UI complétion (formulaire) + UI écarter + lien justificatif — **consomme les endpoints 12-5c** (pas de logique transactionnelle en frontend) + doc (T5+T6). Dépend de 12-5c.

Série a → b → c → d (chaque sous-story testable/mergéable). Les sous-stories seront créées individuellement (`bmad-create-story 12-5a` …) après convergence du présent umbrella.

## Dev Notes

### Ground-truth (vérifié 2026-06-29)
- `crates/kesh-qrbill/src/generator.rs:14-90` — `build_payload`, ordre des lignes SPC (miroir pour le parseur). Payload Kesh = **32 lignes** type K (en-tête 3 l. + IBAN + créancier 7 l. + ultimate creditor 7 l. vides + CcyAmt 2 l. + ultimate debtor 7 l. + RmtInf 2 l. + AddInf 3 l. dont `EPD`). AltPmtInf (l. 33-34) **omis** par Kesh mais possible chez des tiers → parseur tolère 34 l.
- `crates/kesh-qrbill/src/types.rs` — `QrBillData`, `Address` (type K seul en v0.1), `Currency` (Chf/Eur), `Reference` (Qrr/None ; **SCOR non émis** par Kesh mais à tolérer au parse). `ScannedQrBill` **n'existe pas** → à créer (parser.rs). `kesh-qrbill/src/` contient `generator.rs`, `lib.rs`, `pdf.rs`, `types.rs`, `validation.rs` — **pas de `parser.rs`**.
- `crates/kesh-qrbill/src/validation.rs` — validations IBAN/QR-IBAN (IID 30000–31999)/QRR (mod-10 récursif) à **réutiliser** (ne pas dupliquer).
- `crates/kesh-qrbill/Cargo.toml:17` — `rxing = "0.7"` en **`[dev-dependencies]`** → à passer **runtime** (et dans **kesh-api**). `image` = dép **transitive** (Cargo.lock ~0.25.9) **NON déclarée** → à **ajouter explicitement** à `kesh-api/Cargo.toml`. `pdfium-render` **absent** → à ajouter. `lopdf` présent mais **ne rastérise pas** (écarté au profit de pdfium, DC1).
- `crates/kesh-db/src/entities/supplier_invoice.rs` — `NewSupplierInvoice` : `company_id, contact_id, supplier_invoice_number, invoice_date: NaiveDate (NON optional), due_date, creditor_iban, creditor_qr_iban, payment_reference, expected_payment_amount, lines: Vec<NewSupplierInvoiceLine>`. **AUCUN champ document** → confirme DC4 (lien porté par `imported_supplier_invoices`). `invoice_date` non fourni par le QR → saisi à la complétion (AC7).
- `crates/kesh-db/src/repositories/supplier_invoices.rs` — `create` single-step posté ; exige `contact_id` d'un contact **existant** `is_supplier=true` + lignes `expense_account_id`+`vat_rate` ; appelle `fiscal_years::find_open_covering_date(invoice_date)` → `DbError::FiscalYearInvalid` si hors exercice (gérer à la complétion, AC7).
- `crates/kesh-api/src/config.rs` — pattern `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp` — **NE PAS** réutiliser ce défaut pour documents : perte au redémarrage conteneur). Défauts Docker-friendly à poser : `KESH_INBOX_DIR=/data/inbox`, `KESH_DOCUMENTS_DIR=/data/documents`, volumes docker-compose.
- `.github/workflows/release.yml:53` — `platforms: linux/amd64` (single-arch confirmé). `Dockerfile` runtime = `debian:bookworm-slim` (l. 18) sans lib native → ajouter pdfium.
- Liste des paiements (12-3) : opère sur factures `open` → le staging doit produire une facture `open` (après complétion) pour entrer dans un lot pain.001.

### Conventions
- **Pattern batch FailedProposal** (CLAUDE.md) : `{accepted, failed}`, HTTP 200, `file_name` identifiant business, `error_code` constante, `details` JSON. Catalogue en AC3.
- **Migration policy** : `CREATE TABLE imported_supplier_invoices` = **non-breaking** (pas de bump min_required) ; ajouter ligne audit idempotence + compteurs en dur (AC9).
- **Sécurité filesystem** : env (admin), anti-traversal (canonicalize sous racine), symlink rejeté, liste blanche + magic bytes, taille max, stabilité fichier.
- **Test Locally First** exit code vérifié (PAS `cargo test | grep` — masque l'exit code).
- **Issue** : #194 ; 12-4 différé (parseur SPC livré ici, réutilisable). #191 reste ouverte (12-4 follow-up).
- **Branche** : `story/12-5-import-repertoire-factures` (umbrella) ; sous-stories sur la même branche ou branches dédiées 12-5a..d selon flux.

### References
- [Source: issue #194] + commentaire fichier-associé (Guy 2026-06-28).
- [Source: generator.rs:14-90] — payload SPC. [validation.rs] — IBAN/QRR.
- [Source: entities/supplier_invoice.rs] — `NewSupplierInvoice` (pas de champ document → DC4). [repositories/supplier_invoices.rs] — `create` 12-2 + `find_open_covering_date`.
- [Source: config.rs] — pattern env var dossier (`KESH_ADMIN_BACKUP_DIR`). [release.yml:53] — single-arch amd64.
- [pdfium-binaries] — https://github.com/bblanchon/pdfium-binaries (binaire natif, licence).

## Change Log

### Pass 1 — validate (Sonnet 4.6, 2026-06-29)
Reviewer adversarial contexte frais. **19 findings** : 3 CRITICAL, 4 HIGH, 10 MEDIUM, 2 LOW. Trend > LOW : **17**.
- **DC figés par Guy** (AskUserQuestion 2026-06-29) suite F1/F3/F5/F13 : DC4 = lien côté import (FK `supplier_invoice_id` sur `imported_supplier_invoices`, pas d'ALTER 12-2) ; DC2 = bouton manuel seul ; DC5 = métadonnées seules (L1) ; DC1-bis = amd64 + bundle pdfium.
- **F1** (CRITICAL) résolu : tous les DC figés → `ready-for-dev` légitime.
- **F2** (CRITICAL) résolu : `ScannedQrBill`/`ScannedAddress`/`ScannedReference` définis (AC1).
- **F3/F13** (CRITICAL/MEDIUM) résolus : DC4 figé, FK + colonnes document sur `imported_supplier_invoices`, endpoints download spécifiés (AC5).
- **F4** (HIGH) résolu : `rxing`+`image`+`pdfium-render` à déclarer explicitement dans `kesh-api/Cargo.toml` (DC1, Dev Notes).
- **F5** (HIGH) requalifié : prémisse arm64 **réfutée** (NAS amd64, images amd64 OK) ; cœur réel (bundle pdfium natif Docker) figé DC1-bis amd64-only + L3.
- **F6** (HIGH) résolu : `invoice_date` saisi obligatoire à la complétion (AC7), gestion exercice fermé.
- **F7** (HIGH) résolu : symlinks rejetés (`SYMLINK_REJECTED`, AC4).
- **F8/F9** (MEDIUM) résolus : parseur gère type S + 31/32/34 lignes + AltPmtInf (AC1).
- **F10** (MEDIUM) résolu : catalogue `error_code` (AC3).
- **F11** (MEDIUM) résolu : défauts `/data/inbox`/`/data/documents` + volumes (AC9, Dev Notes ; PAS le défaut `/tmp`).
- **F12** (MEDIUM) résolu : comportement exercice fermé à la complétion (AC7).
- **F14** (MEDIUM) résolu : création fournisseur hors scope (L4, sélection contact existant, AC7).
- **F15** (MEDIUM) résolu : multi-QR par page (AC2).
- **F16** (MEDIUM) résolu : détection fichier en cours d'écriture / stabilité (AC4).
- **F17** (MEDIUM) résolu : L1 identifiée (DC5).
- **F18** (LOW) résolu : `generator.rs:14-90` (était 18-90).
- **F19** (LOW) résolu : nommage archivé `{sha256hex}.{ext}` (AC5).

### Pass 2 — validate (Haiku 4.5, 2026-06-29)
Reviewer adversarial contexte frais, discipline grep ground-truth. **9 findings** : 2 CRITICAL, 2 HIGH, 3 MEDIUM, 2 LOW. Trend > LOW : **7**. Aucune hallucination (F-NEW-1/F-NEW-3 multi-tenant vérifiés grep `company_id` sur `supplier_invoices.sql:29` + entity).
- **F-NEW-1** (CRITICAL) résolu : `company_id BIGINT NOT NULL` FK `companies` ajouté → section Schéma explicite.
- **F-NEW-2** (CRITICAL) résolu : tableau complet des colonnes `imported_supplier_invoices` (QR + document + statut + timestamps + version).
- **F-NEW-3** (HIGH) résolu : idempotence scopée `UNIQUE (company_id, file_hash)` (AC6 + schéma) ; 2 companies peuvent importer le même fichier.
- **F-NEW-4** (HIGH) résolu : round-trip type K seul ; type S via fixture manuelle (AC9, generator n'émet que K).
- **F-NEW-5** (MEDIUM) résolu : packaging pdfium détaillé (tag épinglé + checksum + `/usr/local/lib/libpdfium.so` + ldconfig, DC1-bis).
- **F-NEW-6** (MEDIUM) résolu : adresse QR informative, adresse contact prime à la complétion (AC7).
- **F-NEW-7** (MEDIUM) résolu : cap `KESH_INBOX_MAX_FILES_PER_RUN` (AC4) borne le rapport `failed[]`.
- **F-NEW-8** (LOW) résolu : L5 (factures directes 12-2 sans justificatif, 404).
- **F-NEW-9** (LOW) résolu : messages d'erreur complétion (devise/montant/exercice, AC7).
- **Hardening complémentaire** : RBAC Comptable+ + anti-IDOR company-scope sur endpoints download (AC5).

### Pass 3 — validate (Opus 4.8, 2026-06-29)
Reviewer adversarial senior contexte frais, axe **architecture cross-story** (atomicité FS↔DB, intégrité comptable, backup 17-3, split). **10 findings** : 1 CRITICAL, 2 HIGH, 4 MEDIUM, 3 LOW. Trend > LOW : **7**. Pattern « Opus catch architectural » confirmé — les passes 1-2 avaient raté **tout l'axe complétion** (pivot comptable). Ground-truth vérifié (`create` owns-tx `:212/239/406` sans `create_in_tx` ; pain.001 paie `total_amount` `:298` ; routes sans handler complétion `:216/247`).
- **F1** (CRITICAL) résolu : **DC6 figé** — complétion atomique via `create_in_tx` (précédent `pay_in_tx`) + `FOR UPDATE` dans une seule transaction → élimine double-facture/double-écriture.
- **F2** (HIGH) résolu : réconciliation `Σ lignes TTC == staging.amount` à la complétion + `expected_payment_amount` (AC7) → pain.001 ne paie plus un montant ≠ du QR.
- **F3** (HIGH) résolu : **AC10** endpoints liste/complétion/écarter en **backend 12-5c** (étaient implicites/non assignés) ; 12-5d consomme seulement.
- **F4** (MEDIUM) résolu : caps pdfium pages/dimensions + L6 (crash natif in-process) (AC4).
- **F5** (MEDIUM) résolu : re-dépôt après `discarded` réactive `to_complete` au lieu de `DUPLICATE` (AC6).
- **F6** (MEDIUM) résolu : verrou de run + UNIQUE→DUPLICATE + tolérance ENOENT (AC4).
- **F7** (MEDIUM) résolu : fichier disque absent post-restore → 404/410, jamais 500 (AC5).
- **F8** (LOW) résolu : note split optionnel 12-5b1/b2 (Découpage).
- **F9** (LOW) résolu : routage `is_qr_iban`→colonne + appariement QR-IBAN⇒QRR (AC7).
- **F10** (LOW) résolu : position `imported_supplier_invoices` avant `supplier_invoices`/`companies` dans `TABLES_TO_TRUNCATE` (AC9).

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story)

### Debug Log References

### Completion Notes List

### File List
