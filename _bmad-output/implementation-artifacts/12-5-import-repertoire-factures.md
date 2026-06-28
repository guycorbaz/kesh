# Story 12.5: Import de factures fournisseurs depuis un répertoire surveillé

Status: ready-for-dev

<!-- SPEC UMBRELLA — candidate split 12-5a..d. DC majeurs OUVERTS (DC1 PDF, DC3 entité) — à figer avec Guy avant dev. -->

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

Un **dossier d'import** (inbox, sur le serveur/NAS) que l'admin configure. L'utilisateur y dépose des fichiers de factures fournisseurs (PDF/image porteurs d'un Swiss QR Code). Kesh **lit le dossier**, **décode le QR côté serveur**, **parse le payload SPC** (coordonnées créancier/montant/référence), **crée les factures fournisseurs** correspondantes, **archive le fichier source** dans Kesh et l'**associe** à la facture (téléchargeable depuis le détail). À l'import réussi, **le fichier original est supprimé de l'inbox** (la copie archivée suffit, décision Guy) ; les échecs sont déplacés en `failed/`.

⚠️ **Deux tensions architecturales à trancher (DC1, DC3) avant le dev** — cf. §DC.

## Acceptance Criteria

**Parseur SPC (socle — kesh-qrbill)**

1. `parse_spc_payload(text) -> Result<ScannedQrBill, QrBillError>` parse un payload SPC (en-tête `SPC`/`0200`, IBAN/QR-IBAN, nom+adresse créancier, montant, devise, type+valeur de référence, message), inverse exact de `generator.rs` (mêmes index de lignes), valide IBAN/QR-IBAN/QRR via `validation.rs`. Robuste aux champs de fin absents. (Ce parseur est **le socle commun** avec le scan manuel 12-4 différé.)

**Décodage serveur & lecture du dossier (kesh-api / nouveau module)**

2. Un **dossier inbox configurable** (`KESH_INBOX_DIR`, pattern env var comme `KESH_ADMIN_BACKUP_DIR`) est lu sur déclenchement (DC2). Chaque fichier est décodé **côté serveur** : **PDF → rendu page via `pdfium-render` → image → `rxing` (QR→texte)** ; image PNG/JPG → `rxing` directement → texte du QR → `parse_spc_payload` (DC1 figé). Multi-page : tenter chaque page jusqu'à trouver un QR.
3. **Pattern batch / rapport per-fichier** (style `FailedProposal`) : chaque fichier réussi → facture créée + fichier associé ; chaque fichier en échec (illisible / pas de QR / SPC invalide / IBAN invalide) → entrée `failed[]` avec le nom du fichier + raison. Aucun fichier ne fait planter l'import global.
4. **Sécurité** : le dossier inbox est maîtrisé par l'admin (chemin via env, pas d'upload réseau arbitraire) ; protection contre path traversal lors du déplacement ; taille de fichier maximale ; types autorisés en liste blanche. Lecture seule des autres parties du FS.

**Stockage & association du fichier (justificatif)**

5. La **copie du fichier source** est stockée dans un dossier documents configurable (`KESH_DOCUMENTS_DIR`) et **associée** à la facture créée (lien persistant : chemin relatif + nom d'origine + hash SHA-256 + type MIME). Récupérable via `GET …/source-document` (download/consultation) depuis le détail de la facture fournisseur (12-2).
6. À l'import **réussi**, le fichier original est **SUPPRIMÉ de l'inbox** (décision Guy 2026-06-28 : la copie archivée dans Kesh (AC5) est la source de vérité, inutile de le garder dans le dossier). Les fichiers en **échec** sont déplacés vers `failed/` (conservés pour inspection/correction, jamais supprimés). Idempotence : un fichier dont le hash SHA-256 est **déjà archivé** dans Kesh → `failed[] DUPLICATE`, NON ré-importé (déplacé en `failed/`).

**Création des factures & liste des paiements**

7. Pour chaque fichier valide, une **facture importée « à compléter »** (`imported_supplier_invoices`, statut `to_complete`) est créée à partir des coordonnées QR (créancier nom/IBAN/QR-IBAN, référence, montant TTC, devise) + lien fichier archivé (DC3 figé). L'utilisateur la **complète** (fournisseur + lignes compte de charge/HT/TVA) → `supplier_invoices::create` (12-2) avec fichier associé → statut `completed`, facture `open` → entre dans la liste des paiements (12-3). Possibilité d'**écarter** (`discarded`) une facture non pertinente. **Le QR ne fournit ni compte de charge ni TVA** → ces champs sont saisis à la complétion (le pré-remplissage couvre IBAN/référence/montant, partie error-prone).

**Frontend & doc**

8. Un écran **« Importer le dossier »** (Comptable+) déclenche l'import et affiche le **rapport** (créées / échecs avec raison). Le détail d'une facture fournisseur affiche un lien **« Voir la facture d'origine »** (download du justificatif). i18n FR + clés.
9. **Tests** : unitaires parseur SPC (round-trip generator↔parser) + décodage image (fixture QR) ; intégration import (dossier temp avec fixtures → factures + fichiers associés + rapport failed) ; sécurité (path traversal, type refusé, doublon hash) ; E2E selon DC. Doc : `.env.example` (`KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR`), manuel admin (config dossiers), CHANGELOG, README. **Compteurs en dur** : si nouvelle(s) table(s) → bumper `admin_full_export_e2e`/`migrations_upgrade_path` + `TABLES_TO_TRUNCATE` + audit idempotence. Test Locally First **exit code vérifié**.

## Points de décision (DC) — À FIGER AVEC GUY avant le dev

- **DC1 — Décodage des PDF : rendu pdfium** [✅ FIGÉ Guy 2026-06-28] : les factures sont des **PDF** (reçus par e-mail ou scans papier). Décodage **côté serveur par rendu de la page PDF → image** via la crate **`pdfium-render`** (binaire natif `pdfium`), puis localisation/décodage du QR via `rxing`. Couvre TOUS les cas (e-mail vectoriel/raster + scans, y compris CCITTFax). ⚠️ **Impact déploiement** : intégrer le binaire `pdfium` dans l'image Docker (par architecture) — adapter `Dockerfile` + `.github/workflows/release.yml`. Pipeline : fichier PDF → `pdfium-render` (page→`image::DynamicImage`) → `rxing` (QR→texte) → `parse_spc_payload`. Images PNG/JPG directes aussi supportées (rxing sans pdfium). `rxing` passe **dev-dep → runtime**.
- **DC2 — Déclenchement** [proposition] : **bouton « Importer le dossier »** (manuel) en v1, PAS de watch automatique (inotify/polling = tâche de fond + complexité). *Watch auto = follow-up.* À confirmer.
- **DC3 — Modèle d'entité : staging « à compléter »** [✅ FIGÉ Guy 2026-06-28] : l'import crée une **facture importée « à compléter »** (nouvelle table `imported_supplier_invoices` : coordonnées QR parsées — créancier nom/IBAN/QR-IBAN, référence, montant TTC, devise — + lien fichier archivé + statut `to_complete`/`completed`/`discarded`). L'utilisateur la **complète** dans l'UI (sélection/création du fournisseur, ajout des lignes : compte de charge + HT + TVA) → appelle le flux `supplier_invoices::create` de 12-2 (qui poste l'écriture d'achat et crée la facture `open`, **avec le fichier source associé**) → le staging passe `completed`. Découple ingestion et comptabilisation, **préserve l'intégrité comptable** de 12-2 (pas d'écriture approximative). Une facture peut aussi être **écartée** (`discarded`) si non pertinente. Les factures `open` issues de la complétion entrent dans la liste des paiements (12-3).
- **DC4 — Stockage justificatif** [proposition] : `KESH_DOCUMENTS_DIR` (filesystem, hors DB) + table de liaison (ou colonne `source_document_*` sur l'entité créée). Base minimale posée par 12-5 (Epic 14 « Justificatifs » la généralisera). À confirmer.
- **DC5 — Inclusion backup/export** : le fichier justificatif (filesystem) entre-t-il dans le `.keshbackup` (17-3) ? *Proposition : la métadonnée (lien) oui (table) ; le binaire du fichier → hors scope backup v0.4 (documenté).* À confirmer.

## Tasks / Subtasks (provisoire — dépend des DC, à affiner au validate)

- [ ] **T1 — Parseur SPC (kesh-qrbill)** : `src/parser.rs` `parse_spc_payload` + `ScannedQrBill` + tests round-trip (socle).
- [ ] **T2 — Décodage serveur** (selon DC1) : module décodage image (`rxing`+`image`) [+ PDF selon DC1]. Tests fixtures.
- [ ] **T3 — Entité + stockage** (selon DC3/DC4) : migration (staging ou liaison fichier) + stockage `KESH_DOCUMENTS_DIR` + hash + association. Compteurs en dur si table.
- [ ] **T4 — Service d'import (kesh-api)** : lecture inbox + boucle décodage + création + déplacement processed/failed + rapport batch + endpoint download justificatif. Sécurité (traversal, taille, types, doublon hash).
- [ ] **T5 — Frontend** : écran « Importer le dossier » + rapport + lien « Voir la facture d'origine » sur le détail. i18n.
- [ ] **T6 — Doc + tests** : `.env.example` + manuel admin + CHANGELOG + README. Quality gate Test Locally First exit-code vérifié.

## Découpage

**Candidate split 12-5a (parseur SPC) / 12-5b (décodage+entité+stockage) / 12-5c (service import+endpoint) / 12-5d (frontend+doc)** — story large (nouveau sous-système serveur). Le `validate` tranchera après figement des DC1/DC3.

## Dev Notes

### Ground-truth
- `crates/kesh-qrbill/src/generator.rs:18-90` — ordre des lignes SPC (miroir pour le parseur). `validation.rs` — validations IBAN/QR-IBAN/QRR. `rxing` = dev-dep actuellement → **à passer runtime** pour le décodage serveur (DC1). `image` crate dispo (Cargo.lock).
- `lopdf` présent (Cargo.lock) — parse la structure PDF mais **ne rastérise pas** (option DC1-b : extraire image XObject embarquée). `pdfium-render` (non présent) = dép native pour le rendu (DC1-a).
- Pas de mécanisme **justificatif/document** existant (Epic 14 non livré) → 12-5 pose une base minimale (DC4). Pattern env var dossier : `config.rs:897` `KESH_ADMIN_BACKUP_DIR`.
- `supplier_invoices` (12-2) : création **single-step postée** (`repositories::supplier_invoices::create`), exige lignes avec `expense_account_id` + `vat_rate` → incompatible avec import QR-seul (DC3).
- Liste des paiements (12-3) : opère sur factures `open` → l'entité importée doit devenir `open` (après complétion DC3-a) pour entrer dans un lot pain.001.

### Conventions
- **Pattern batch FailedProposal** (CLAUDE.md) pour le rapport per-fichier.
- **Test Locally First** exit code vérifié (PAS `cargo test | grep`).
- **Sécurité filesystem** : chemins via env (admin), validation anti-traversal, liste blanche d'extensions, taille max.
- **Issue** : #194 ; 12-4 différé (parseur SPC livré ici, réutilisable par 12-4 plus tard). #191 reste ouverte (12-4 follow-up).
- **Branche** : `story/12-5-import-repertoire-factures`.

### References
- [Source: issue #194] + commentaire fichier-associé.
- [Source: generator.rs:18] — payload SPC. [validation.rs] — IBAN/QRR.
- [Source: repositories/supplier_invoices.rs] — création 12-2 (tension DC3).
- [Source: config.rs:897] — pattern env var dossier.

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story)

### Debug Log References

### Completion Notes List

### File List
