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
   - **Fichier en cours d'écriture** : avant traitement, vérifier la **stabilité** (taille + mtime identiques sur 2 lectures espacées d'un court délai) ; si instable → ignorer ce tour (ni `accepted` ni `failed`, retenté au prochain déclenchement) OU `FILE_READ_ERROR` si lecture impossible. Documenter le choix retenu au dev.
   - Lecture seule du reste du FS (jamais d'écriture hors `KESH_DOCUMENTS_DIR` / `processed` / `failed`).

**Stockage & association du fichier (justificatif) — 12-5b (schéma) + 12-5c (I/O)**

5. La **copie du fichier source** est stockée dans `KESH_DOCUMENTS_DIR` (filesystem, hors DB) sous le nom **`{sha256hex}.{ext}`** (pas de collision, pas de traversal ; nom d'origine conservé **en DB uniquement**). Le lien persistant porte : `storage_path` (relatif à `KESH_DOCUMENTS_DIR`), `original_filename`, `sha256` (hex), `mime_type`, `byte_size`. **DC4 figé (lien côté import)** : ces colonnes vivent sur `imported_supplier_invoices` (PAS d'ALTER de `supplier_invoices` 12-2). Récupérable via `GET /api/v1/supplier-invoices/{id}/source-document` qui résout `imported_supplier_invoices WHERE supplier_invoice_id = {id}` (download/consultation) depuis le détail de la facture fournisseur (12-2) **après complétion**, et `GET /api/v1/imported-supplier-invoices/{id}/source-document` avant complétion.
6. À l'import **réussi**, le fichier original est **SUPPRIMÉ de l'inbox** (décision Guy : la copie archivée AC5 est la source de vérité). Les fichiers en **échec** sont déplacés vers `failed/` (conservés, jamais supprimés). **Idempotence** : un fichier dont le hash SHA-256 est **déjà archivé** → `failed[] DUPLICATE`, NON ré-importé, déplacé en `failed/` (le fichier original n'est PAS supprimé puisqu'il finit dans `failed/`). Le hash est calculé **avant** décodage QR (court-circuit doublon).

**Création des factures importées (staging) & complétion — 12-5b (entité) + 12-5d (UI)**

7. Pour chaque fichier valide, une **facture importée « à compléter »** (`imported_supplier_invoices`, statut `to_complete`) est créée à partir des coordonnées QR (créancier nom/IBAN/QR-IBAN, référence, montant TTC, devise) + lien fichier archivé. **DC3 figé.** L'utilisateur la **complète** dans l'UI :
   - **Fournisseur** : **sélection d'un contact existant** avec `is_supplier = true` (la création inline d'un fournisseur est **hors scope 12-5** ; si absent, l'utilisateur le crée via le flux Contacts existant puis revient — documenté UX).
   - **`invoice_date`** : **saisie obligatoire à la complétion** (date-picker). Le payload SPC **ne contient aucune date** → non pré-remplissable (cf. F6). `supplier_invoices::create` exige une `invoice_date` non-optionnelle couverte par un exercice ouvert.
   - **Lignes** : compte de charge (`expense_account_id`) + HT + TVA — **saisis à la complétion** (le QR ne fournit ni compte ni TVA). Le pré-remplissage couvre IBAN/référence/montant TTC/devise (partie error-prone).
   - À la validation → appel `repositories::supplier_invoices::create` (12-2, single-step posté, **avec le fichier source associé** via `supplier_invoice_id` renseigné sur le staging) → staging passe `completed`, facture `open` → entre dans la liste des paiements (12-3).
   - **Échec de `create` à la complétion** (ex. `DbError::FiscalYearInvalid` si `invoice_date` hors exercice ouvert) : le staging **reste `to_complete`**, message d'erreur UX explicite (« Aucun exercice fiscal ouvert couvrant cette date »). Aucune écriture comptable partielle.
   - Possibilité d'**écarter** (`discarded`) une facture importée non pertinente (le fichier archivé est conservé ; pas de suppression du justificatif en v0.4).

**Frontend & doc — 12-5d**

8. Un écran **« Importer le dossier »** (Comptable+) déclenche l'import et affiche le **rapport** (créées / échecs avec `file_name` + `error_code` traduit). Une vue liste des **factures importées** (`to_complete`) avec action **Compléter** (formulaire fournisseur+date+lignes) et **Écarter**. Le détail d'une facture fournisseur (12-2) affiche un lien **« Voir la facture d'origine »** (download du justificatif) **si** un staging `completed` la référence. i18n FR + clés (DE/IT/EN = clés ajoutées, valeurs FR provisoires si pas de traduction — cohérent politique i18n projet).
9. **Tests** : unitaires parseur SPC (round-trip `build_payload`↔`parse_spc_payload` pour type K ; cas type S ; 31/32/34 lignes ; montant vide ; QRR/NON) + décodage image (fixture QR PNG) + rendu PDF (fixture PDF 1 page avec QR ; multi-page) ; intégration import (dossier temp avec fixtures → staging + fichiers associés + rapport `failed`) ; sécurité (path traversal, symlink, type refusé, magic-bytes, doublon hash, taille max) ; complétion (`to_complete`→`completed`, échec exercice fermé, `discarded`) ; E2E selon DC. **Doc** : `.env.example` (`KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR`/`KESH_INBOX_MAX_FILE_BYTES` + défauts + mapping docker-compose volumes), `docker-compose.yml` (volumes `/data/inbox` + `/data/documents`), manuel admin (config dossiers + pdfium), CHANGELOG, README. **Compteurs en dur** : nouvelle table `imported_supplier_invoices` → bumper `admin_full_export_e2e` (`data_count`) + `migrations_upgrade_path` + `TABLES_TO_TRUNCATE` (backup) + manifeste export (17-3) + audit idempotence `docs/migrations-idempotence-audit.md`. Test Locally First **exit code vérifié** (PAS `cargo test | grep`).

## Points de décision (DC) — TOUS FIGÉS

- **DC1 — Décodage des PDF : rendu pdfium** [✅ FIGÉ Guy 2026-06-28] : factures = **PDF** (e-mail ou scans papier). Décodage côté serveur **par rendu de la page PDF → image** via **`pdfium-render`** (binaire natif `pdfium`), puis QR via `rxing`. Couvre tous les cas (vectoriel/raster + scans CCITTFax). Pipeline : PDF → `pdfium-render` (page→`image::DynamicImage`) → `rxing` (QR→texte) → `parse_spc_payload`. Images PNG/JPG directes aussi (rxing sans pdfium). `rxing` passe **dev-dep → runtime** ; `image` à **déclarer explicitement** (cf. F4).
- **DC1-bis — Packaging Docker pdfium** [✅ FIGÉ Guy 2026-06-29] : **`linux/amd64` uniquement** (cohérent `release.yml` actuel ; le NAS de Guy exécute les images amd64). Télécharger le binaire `pdfium` **amd64** (depuis `bblanchon/pdfium-binaries`, version épinglée) dans le stage `runtime` du `Dockerfile` (placement du `.so` + chemin de lib). **Pas de multi-arch arm64** en v0.4 (follow-up si cible ARM un jour). Vérifier la **licence pdfium** (Apache-2.0 / BSD-3, OK) et la mentionner dans la doc.
- **DC2 — Déclenchement : bouton manuel** [✅ FIGÉ Guy 2026-06-29] : bouton « Importer le dossier » (manuel). **PAS de watch auto** (inotify/polling) en v0.4 — follow-up.
- **DC3 — Modèle d'entité : staging « à compléter »** [✅ FIGÉ Guy 2026-06-28] : table `imported_supplier_invoices` (coordonnées QR parsées + lien fichier archivé + `supplier_invoice_id` nullable + statut `to_complete`/`completed`/`discarded`). Découple ingestion et comptabilisation, **préserve l'intégrité comptable** de 12-2.
- **DC4 — Stockage justificatif : lien côté import** [✅ FIGÉ Guy 2026-06-29] : `KESH_DOCUMENTS_DIR` (filesystem, hors DB) + **colonnes document sur `imported_supplier_invoices`** (`storage_path`, `original_filename`, `sha256`, `mime_type`, `byte_size`) + **FK nullable `supplier_invoice_id` → `supplier_invoices.id`** (renseignée à la complétion). **AUCUN ALTER de `supplier_invoices`** (12-2) → migration `CREATE TABLE` **non-breaking** (pas de bump `kesh_version_min_required`). Le détail facture (12-2) résout le justificatif via `imported_supplier_invoices WHERE supplier_invoice_id = {id}`. Base minimale ; Epic 14 « Justificatifs » généralisera.
- **DC5 — Inclusion backup/export : métadonnées seules** [✅ FIGÉ Guy 2026-06-29] : la **métadonnée** (table `imported_supplier_invoices`, incl. `storage_path`/`sha256`) entre dans le `.keshbackup` (17-3, via manifeste export). Le **binaire des fichiers** (`KESH_DOCUMENTS_DIR`) reste **HORS backup v0.4** → limitation **L1** documentée. Le restore ne restaure pas les fichiers physiques.

## Limitations documentées

- **L1** — Les fichiers justificatifs binaires (`KESH_DOCUMENTS_DIR`) ne sont **PAS** inclus dans le `.keshbackup`. Seules les métadonnées (`storage_path`, `original_filename`, `sha256`, `mime_type`) sont exportées. Le restore d'une installation ne restaure pas les fichiers physiques. **Remédiation** : Epic 14 « Justificatifs » généralisera le backup des pièces. (DC5)
- **L2** — Pas de **watch automatique** de l'inbox en v0.4 (déclenchement manuel par bouton). Follow-up. (DC2)
- **L3** — `linux/amd64` uniquement (pas d'image arm64). Follow-up si déploiement sur cible ARM. (DC1-bis)
- **L4** — Création **inline d'un fournisseur** hors scope de la complétion 12-5 (sélection d'un contact `is_supplier` existant ; sinon créer via Contacts puis revenir). (AC7)

## Tasks / Subtasks

- [ ] **T1 (12-5a) — Parseur SPC (kesh-qrbill)** : `src/parser.rs` `parse_spc_payload` + `ScannedQrBill`/`ScannedAddress`/`ScannedReference` exportés `lib.rs` ; gestion type K/S, 31/32/34 lignes, montant vide ; validations IBAN/QR-IBAN/QRR via `validation.rs` ; tests round-trip `build_payload`↔parser.
- [ ] **T2 (12-5b) — Décodage serveur** : ajouter `rxing` + `image` + `pdfium-render` aux `[dependencies]` de kesh-api ; module décodage (PNG/JPG via rxing ; PDF via pdfium-render multi-page ; multi-QR par page). Bundle pdfium dans `Dockerfile` (amd64). Tests fixtures (PNG QR + PDF QR).
- [ ] **T3 (12-5b) — Entité + stockage** : migration `CREATE TABLE imported_supplier_invoices` (colonnes QR + document + `supplier_invoice_id` FK nullable + statut + index) + stockage `KESH_DOCUMENTS_DIR` (`{sha256hex}.{ext}`) + hash SHA-256. Compteurs en dur + audit idempotence + manifeste export 17-3 + TABLES_TO_TRUNCATE.
- [ ] **T4 (12-5c) — Service d'import (kesh-api)** : lecture inbox + court-circuit doublon hash + boucle décodage + création staging + déplacement `failed/` / suppression succès + rapport batch `{accepted, failed}` (HTTP 200) + endpoints `POST .../inbox-import`, `GET .../imported-supplier-invoices/{id}/source-document`, `GET .../supplier-invoices/{id}/source-document`. Sécurité (symlink, traversal, taille, types+magic, stabilité fichier).
- [ ] **T5 (12-5d) — Frontend** : écran « Importer le dossier » + rapport (file_name + error_code i18n) + liste factures importées `to_complete` + formulaire complétion (fournisseur existant + `invoice_date` + lignes → `supplier_invoices::create`) + action Écarter + lien « Voir la facture d'origine » sur le détail 12-2. i18n FR + clés.
- [ ] **T6 (12-5d) — Doc + tests** : `.env.example` (3 env vars + défauts) + `docker-compose.yml` (volumes) + manuel admin (dossiers + pdfium + licence) + CHANGELOG + README. Quality gate Test Locally First exit-code vérifié.

## Découpage

**Split figé** (story large = nouveau sous-système serveur + dép native + migration + UI ; cohérent §"Règle de splitting préventif" CLAUDE.md, scope cross-cutting > 5 modules : kesh-qrbill, kesh-api, kesh-db, frontend, Docker/CI) :

- **12-5a — Parseur SPC (kesh-qrbill)** : story-zéro qui pose le socle (T1). Réutilisable par 12-4 différé. Aucune dép native, testable en isolation (round-trip). **À implémenter en premier.**
- **12-5b — Décodage serveur + entité + stockage** : pdfium/rxing/image + migration `imported_supplier_invoices` + `KESH_DOCUMENTS_DIR` (T2+T3). Dépend de 12-5a.
- **12-5c — Service d'import + endpoints + sécurité** : lecture inbox + rapport batch + sécurité + download (T4). Dépend de 12-5b.
- **12-5d — Frontend + complétion + doc** : UI import/complétion + lien justificatif + doc (T5+T6). Dépend de 12-5c.

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

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story)

### Debug Log References

### Completion Notes List

### File List
