# Story 30.2 : Des documents à traiter à la facture fournisseur

## Status

**30-2a : review** (développée le 2026-10-15 ; reportée sur `fecc8566`, merge de la 30-1a — T-a0 fait ; revue de
code P1 faite — B 0/0/0/5, E 0/0/2/5, A 0/0/0/8 — et remédiée le 2026-10-15 ; **passe ciblée P2** à lancer,
`30-2a-review-prompt-p2-ciblee.md`, Haiku). **30-2b, 30-2c : ready-for-dev.**

*Historique de la validation (paragraphe écrit avant le développement ; la passe ciblée P5 qu'il annonce « à lancer » a
été faite le 2026-10-15 — 0/0/0/5, validation terminée, cf. Change Log) :* **ready-for-dev** (trois clés) — fiche créée le 2026-10-15 en autonomie (`bmad-create-story`, Opus 5.5), sur
`15686d55` (kickoff de l'Epic 30), worktree `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`. Validation P1
faite (Sonnet, lentilles R et F : 0/0/11/9 et 0/0/7/6) et remédiée le 2026-10-15 ; validation P2 faite (Opus, R et F :
0/0/8/13 et 0/0/5/8) et remédiée le même jour ; validation P3 faite (Sonnet, R et F : 0/1/3/12 et 0/0/3/12) et remédiée
le même jour ; validation P4 faite (Opus, R et F : 0/0/2/13 et 0/0/1/7) et remédiée le même jour ; **passe ciblée P5** à
lancer (`30-2-validate-prompt-p5-ciblee.md`, **Haiku**, une lentille, sur le seul diff de la remédiation P4).

⛔ **Trois stories, une fiche, trois PR successives** (C-30-2-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`30-2a-documents-sans-qr`** | la migration (colonnes QR facultatives, « tout ou rien », unicité de `supplier_invoice_id`, `min_required` 0.14.0) ; l'import accepte un document sans QR ou au QR illisible ; la liste « Documents à traiter » avec aperçu (composant et CSP) et « Retirer de la liste » ; la complétion d'un document sans QR par le formulaire existant ; export CSV ; contrôle K5 | AC1 à AC9, AC6-bis, AC9-bis | **`refs #685`** — ⛔ jamais `closes` |
| **`30-2b-creation-cote-a-cote`** | le formulaire extrait en composant ; la page `/supplier-invoices/new`, seule ou à côté du document ; coordonnées QR verrouillées ; corps de complétion = corps de création, projet compris ; fournisseur proposé, ou créé sans quitter la page | AC10 à AC17, AC12-bis | **`refs #685`** |
| **`30-2c-document-joint-et-consulte`** | joindre un document après coup, le **détacher** (décision de l'orchestrateur, C-30-2-13) ; la fiche de la facture et le détail de l'écriture montrent le document ; lecture ouverte à tout rôle authentifié ; E2E de parcours complet ; manuel, CHANGELOG et `api-external.md` complétés | AC18 à AC25 | **`closes #685`** |

**Ordre** : la 30-2a part d'`origin/main` **après le merge de la 30-1** (C-30-0-3) ; la 30-2b après le merge de la
30-2a ; la 30-2c après celui de la 30-2b. Aucune release entre elles : la v0.14.0 attend la 30-2c (C-30-0-1). La clé
mère `30-2-documents-factures-fournisseur` passe au statut `split`.

## Story

**As a** indépendant ou petite entreprise qui saisit chaque mois les factures fournisseur des débits de son relevé —
souvent anciennes, déjà payées, parfois sans QR-code,
**I want** voir tous les documents déposés dans le dossier d'import, avec ou sans QR-code, les prévisualiser, retirer
ceux qui ne sont pas des factures, et créer une facture en gardant le document sous les yeux — fournisseur proposé ou
créé sans quitter la saisie —, ou sans document, et joindre un document plus tard,
**so that** chaque facture fournisseur ait sa pièce, consultable depuis la facture et son écriture, et que la
réconciliation (30-1) puisse ensuite la régler.

Issue : [#685](https://github.com/guycorbaz/kesh/issues/685) (`enhancement`) — **conception de Guy, elle fait foi**.
Epic : `_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md` § 30-2. Cible : **v0.14.0**. Choix :
`epic-30-choix-autonomes-302.md` (C-30-2-1 à 23).

## Prémisses de #685, établies au code (`15686d55`)

| # | Prémisse | Verdict | Preuve |
|---|---|---|---|
| P-a | L'import lit le dossier `KESH_INBOX_DIR` et « refuse un document sans QR-code » | **vraie** | `crates/kesh-api/src/inbox_import.rs:486-512` : décodage puis `Ok(None) => dispose_failed(…, ERR_NO_QR_CODE_FOUND)` (l. 497-505) ; le fichier part dans `<inbox>/failed/`. Libellé « Aucune QR-facture détectée » : `crates/kesh-i18n/locales/fr-CH/messages.ftl:2167`, `frontend/src/lib/features/imported-supplier-invoices/error-label.ts:33` |
| P-a′ | *(au-delà de l'issue)* Cause structurelle du refus | — | la table de staging exige les données du QR : sept colonnes `NOT NULL` (`crates/kesh-db/migrations/20260629000001_imported_supplier_invoices.sql`, `creditor_iban` … `currency`) ; l'entité les porte en `String`/`bool` (`crates/kesh-db/src/entities/imported_supplier_invoice.rs:32-47`). D'où la migration (C-30-2-2) |
| P-a″ | *(au-delà de l'issue)* Quatre autres refus tiennent au QR, non au document | — | `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR` (`map_decode_error`, l. 656-663 ; un PDF de plus de `KESH_INBOX_MAX_PDF_PAGES` pages sans QR rend `PdfRender`, `qr_decode.rs:169-178` ; pdfium absent aussi, l. 154) et `FIELD_TOO_LONG` (l. 556-570) — traités comme l'absence de QR (C-30-2-4) |
| P-b | La liste « à compléter » offre *Compléter* et *Écarter* | **vraie** | `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte` (612 l.) : boutons l. ~455-472 du gabarit (`imported-supplier-invoices-complete`, `-discard`, `-view-doc`) ; *Voir le justificatif* **télécharge** (`downloadImportedSourceDocument`, `imported-supplier-invoices.api.ts:44`), il n'y a **aucune prévisualisation** |
| P-c | La saisie manuelle (*Nouvelle facture*) n'accepte aucun fichier | **vraie, à une nuance près** | formulaire en ligne de `routes/(app)/supplier-invoices/+page.svelte` (bouton l. 229-237, enregistrement l. 192) ; il accepte une **image pour scanner le QR dans le navigateur** (Story 12.4, `supplier-invoice-scan.ts`), sans la conserver |
| P-d | Après création, le document « quitte la liste à traiter **et le dossier d'entrée** » | **à corriger** | le fichier quitte le dossier **à l'import** : archivé sous `KESH_DOCUMENTS_DIR` (`document_storage::store_document`, l. 515) puis supprimé de l'inbox (`remove_inbox_file`, l. 623). À la création, il ne quitte que la **liste** (`mark_completed`). Comportement conservé (C-30-2-17) — à commenter sur l'issue |
| P-e | Le document reste « consultable depuis la facture » | **vraie en partie** | `GET /supplier-invoices/{id}/source-document` (`routes/imported_supplier_invoices.rs:421-436`) et bouton « Voir la facture d'origine » de la fiche (`routes/(app)/supplier-invoices/[id]/+page.svelte:284-290`) — mais le bouton est **toujours affiché**, et rend 404 pour une facture sans document ; la route est **Comptable+** (`lib.rs:506-513`) alors que la fiche se lit par tout rôle (`lib.rs:868-875`) |
| P-f | … « et depuis son écriture » | **fausse aujourd'hui** | `JournalEntryDetailResponse` (`routes/journal_entries.rs:150-182`) ne porte que des **motifs** (`OWNED_BY_SUPPLIER_INVOICE` + numéro en étiquette) ; la page `journal-entries/[id]` n'a aucun lien vers la facture ni le document |
| P-g | « Joindre un document après coup » | **absent** | aucune route n'écrit `supplier_invoice_id` hors de `mark_completed` (`repositories/imported_supplier_invoices.rs:177-205`) |
| P-h | Pièces jointes existantes (#235) | **absentes** | #235 est ouverte ; aucune table de pièces jointes ; seuls existent ce staging et les PDF figés de facture client (Story 25-6-b), dans le même `KESH_DOCUMENTS_DIR` |
| P-i | La sauvegarde doit emporter les pièces (#503) | **fausse aujourd'hui, hors périmètre** | `admin_backup/export.rs:94-97` écrit un `files/` vide ; `admin_backup/import.rs:91` refuse un `files/` non vide. Après restauration, un document répond **410** (`serve_document`, l. 442-468). La 30-2 augmente l'exposition : #503 est à porter à Guy avant la mise en production (C-30-2-16) |
| P-j | Le fournisseur n'est jamais créé d'office | **vraie** | aucun `contacts::create` dans `inbox_import.rs` ni dans `routes/imported_supplier_invoices.rs` (`grep -n "contacts::" `, 0 résultat) ; le formulaire de complétion fait choisir parmi `listContacts({ isSupplier: true })` — **sans proposition** : le nom du créancier est seulement affiché |

### Ce qui existe et se réutilise (DRY — le côté fournisseur reproduit le côté client)

- **Stockage** : `document_storage.rs` — `store_document` (nommage `{sha256}.{ext}`, écriture atomique, anti-traversal)
  et `read_document` (absent → `NotFound` → 410). Inchangé.
- **Lecture d'un document** : `serve_document` (l. 442), `Content-Disposition` RFC 5987 ; côté écran
  `apiClient.getBlob` (gère le 401-refresh). La prévisualisation la réutilise (C-30-2-6).
- **Sécurité de l'import** : liste blanche `pdf`, `png`, `jpg`, `jpeg` + octets magiques (`magic_matches`), taille
  `KESH_INBOX_MAX_FILE_BYTES` (défaut 25 Mio) lue par `stat` avant lecture, liens symboliques refusés (deux fois,
  anti-TOCTOU), anti-traversal par chemin canonique, verrou de run `GET_LOCK` par base, doublon par empreinte
  (`UNIQUE (company_id, file_hash)`), type MIME **dérivé de l'extension** (`mime_for_ext`), jamais du fichier.
  **Rien de cela ne change.**
- **Isolation par société** : toutes les lectures du staging passent par `find_by_id_scoped*` (`company_id`) ; L8 —
  dossier unique par instance, #199 — inchangé.
- **Complétion transactionnelle** : `complete_import_once` (verrou du staging, contrôles, `supplier_invoices::create_in_tx`,
  `mark_completed`, audit, rejeu sur interblocage) — étendue, pas dupliquée.
- **Appariement de contacts** : `features/contacts/duplicate-probe.ts` (`normalizeTerm`, `rank`, `probeTerm`) et le
  patron d'appel de `contacts/+page.svelte:303-320` (`listContacts({ search, limit: 20 })` puis `rank`).
- **Écritures possédées par une facture fournisseur** : `DocumentKind::SupplierInvoice`
  (`kesh-db/src/repositories/journal_entries.rs:2073-2087`) — achat et règlement.

## Périmètre minimal

**Dedans** : les sept points du parcours de #685, et ce qui les rend vrais au code — rien d'autre.

**Dehors** (et pourquoi) :

| Hors périmètre | Raison | Suite |
|---|---|---|
| Inclusion des documents dans `.keshbackup` | #503, non mécanique | signalée (C-30-2-16) ; manuel utilisateur renvoie à la sauvegarde de `KESH_DOCUMENTS_DIR` |
| Plusieurs pages / plusieurs documents par facture | #685, « plus tard » | — |
| Dépôt par le navigateur ; import automatique du dossier | #685 ; #459 | — |
| Pièces jointes sur une écriture quelconque | #235 | — |
| Tickets de caisse, factures client | fichier d'epic | — |
| Factures en devise | factures fournisseur en CHF seulement | inchangé (C-30-2-19) |
| `kesh-reconciliation` | la 30-1 le modifie en parallèle | **ne pas y toucher** |

### Décompte des modules (règle de découpage)

| Sous-story | Modules | Nombre |
|---|---|---|
| 30-2a | `kesh-db` (migration, squash de test, entité, `post_restore`), `kesh-api` (import, routes du staging, CSP, export CSV), `kesh-i18n`, frontend (liste, aperçu, K5), docs | 5 |
| 30-2b | `kesh-db` (lecture des fournisseurs par IBAN, AC14 — ajoutée en P2), `kesh-api` (complétion, validation, route `supplier-candidates`), frontend (formulaire, page, contacts), `kesh-i18n`, docs | 5 |
| 30-2c | `kesh-db` (dépôt : rattachement, lecture du document d'une écriture), `kesh-api` (routes, réponses), frontend (fiche, écriture), `kesh-i18n`, docs | 5 |

## Critères d'acceptation

### Partie a — `30-2a-documents-sans-qr`

**AC1 — Un document sans QR entre dans la liste.** Given un PDF, PNG ou JPEG valide (extension et octets magiques
cohérents, taille sous la borne) **sans** QR-code dans `KESH_INBOX_DIR`, When *Importer le dossier*, Then il est
archivé, supprimé du dossier, inséré en `to_complete` avec les sept colonnes QR à `NULL` (`amount`,
`reference_value`, adresse et messages à `NULL`) ; le rapport le porte dans `accepted[]` avec `qr: "none"` ; l'audit
`imported_supplier_invoice.created` porte `qr_present: false` et ses clés `creditor_name`, `amount`, `currency`,
`reference_type` **présentes à `null`**, et `iban_present: false` (aujourd'hui calculé `!creditor_iban.is_empty()`,
`inbox_import.rs:613`, devient `creditor_iban.is_some()`) — même forme de détails qu'un document avec QR ; le test
l'asserte clé par clé (mutation M-a6 côté retrait, M-a15 côté création).

**AC2 — Un QR inexploitable n'empêche pas l'entrée.** `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR` et un QR
dont une valeur ne tient pas dans sa colonne donnent le même résultat qu'AC1, avec `qr: "unreadable"` et
`qrError: "<code>"`. Le **contrôle pur** est une fonction de la QR-facture décodée (`ScannedQrBill`), appelée **avant
l'archivage** (étape (7), `inbox_import.rs:515`) : elle décide si le document entre avec ou sans ses données QR, puis
l'archivage et `NewImportedSupplierInvoice` (`from_scanned` ou `without_qr`) suivent. Elle couvre **toutes les
colonnes contraintes que remplit le QR** (C-30-2-22) :
- **longueur**, en **caractères** (`chars().count()` — `VARCHAR`/`CHAR` en `utf8mb4` comptent des caractères) :
  `creditor_iban` 34, `creditor_name`, `creditor_line1`, `creditor_line2` 70, `creditor_postal_code` 16,
  `creditor_town` 35, `creditor_country` 2, `reference_value` 40, `currency` 3, `unstructured_message` et
  `billing_information` 140 (le parseur ne borne ni le pays ni la devise, `crates/kesh-qrbill/src/parser.rs:137` et
  `:150`) ;
- **plage du montant** : le parseur ne le borne pas (`Decimal::from_str`, `parser.rs:145` ; `amount_max` n'est appelé
  que par `validate`, côté générateur, `validation.rs:439`) : le contrôle exige `amount_min()..=amount_max()` de
  `kesh_qrbill::validation` (`validation.rs:23-30`) et au plus deux décimales **significatives**
  (`normalize().scale() <= 2` : `20.000` passe, `20.005` non). Seule la **plage** protège la colonne : au-delà de 15
  chiffres entiers, `DECIMAL(19,4)` lève `1264` (→ `DataLengthOrRange`) ; les deux décimales sont une **règle SIX**
  (champ `Amt`), non une contrainte de colonne — trois décimales tiendraient dans `DECIMAL(19,4)`.
- **pas de contrôle de domaine** : le type d'adresse est déjà normalisé par `from_scanned` (tout type ≠ `S` devient `K`,
  `crates/kesh-db/src/entities/imported_supplier_invoice.rs:117-128`, test `from_scanned_normalizes_unexpected_address_type_to_k`,
  l. 256), et `reference_type` est borné par le parseur (`parser.rs:155-164`). *(La remédiation P2 avait ajouté ici un
  contrôle de domaine contre un « défaut d'origine » — 500 sur tout le run — qui n'existe pas ; retiré en P3, voir le
  Change Log.)*
Le contrôle de plage ne porte que sur un montant **présent** : un QR à montant vide (« open amount », `parser.rs:140-141`)
garde ses données, comme aujourd'hui.
**Codes `qrError`** (P4, R2) : `FIELD_TOO_LONG` pour toute longueur dépassée (pays et devise compris) ;
**`QR_AMOUNT_OUT_OF_RANGE`** — code neuf, un seul pour la plage et les décimales — pour le montant ; les codes du
décodage (`INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR`) inchangés. Libellé neuf ×4
`imported-supplier-invoices-error-qr-amount-out-of-range` (« Montant du QR hors des limites admises »), ajouté à la carte
d'`error-label.ts` (sans quoi il tomberait dans le repli « Échec de l'import », l. 45) ; et, pour **tout** document
`unreadable`, l'écran dit que le document est **accepté sans les données du QR**, à saisir à la main (clé
`imported-supplier-invoices-qr-unreadable-hint`, ×4). `api-external.md` documente la liste des codes de `qrError`.
Tests, chacun **asserte `qrError`** : nom accentué de 70 caractères accepté avec ses données et **nom stocké égal** au nom
injecté, de 71 accepté sans (`FIELD_TOO_LONG`) ; pays de trois lettres (`FIELD_TOO_LONG`) ; montant à 16 chiffres entiers,
montant négatif ou nul, montant `20.005` (`QR_AMOUNT_OUT_OF_RANGE`) ; `20.000` et montant vide acceptés avec leurs
données — aucun en `failed[]`. Ces QR se fabriquent en **généralisant** la substitution que fait déjà `qr_invoice_png`
**après** `build_payload` (`crates/kesh-api/tests/inbox_import_e2e.rs:302-305`, jeton `NAMEPLACEHOLDER`) : même procédé
pour le montant, le pays et la devise, que `validate` (`kesh-qrbill/src/generator.rs:14-15`) refuserait avant génération
— pas de second helper (DRY). Le PNG est encodé en **UTF-8 avec ECI** (indication `CharacterSet` de `rxing`, aujourd'hui
`EncodeHints::default()`, l. 259), faute de quoi un accent relu en Latin-1 ou Shift_JIS ferait rougir le test pour
une mauvaise raison (P4, F6).
**Filets** : la branche `DbError::DataLengthOrRange` reste, sans test qui l'atteigne une fois le contrôle complet ; une
branche `DbError::CheckConstraintViolation` est **ajoutée** au même endroit, avec le même traitement (`failed[]`,
`INVALID_SPC_PAYLOAD`, fichier archivé non supprimé) — défense en profondeur, sans test : aucune contrainte CHECK n'est
atteignable depuis `from_scanned`/`without_qr` (type d'adresse normalisé, `reference_type` borné, `byte_size > 0` par les
octets magiques, « tout ou rien » par construction) ; sans elle, une violation sortirait de tout l'import en
`400 CHECK_CONSTRAINT_VIOLATION` (`kesh-api/src/errors.rs:3281-3287`). Une mutation de l'un ou l'autre filet est
équivalente (M-a14, qui couvre les deux). Les colonnes du fichier ne viennent pas du QR et ne se
dépassent pas : `original_filename` est le nom d'une entrée de répertoire, borné à 255 **octets** par le système de
fichiers (`NAME_MAX`), donc à 255 caractères au plus — le cas « nom trop long » est **inatteignable** et n'a pas de test.
Un document **avec** QR valide garde exactement son comportement actuel (`qr: "present"`). Un document **réactivé**
(retiré puis redéposé) porte `qr: "present"` ou `qr: "none"` selon la ligne stockée (un QR inexploitable n'est pas
stocké : il redevient `none`).

**AC3 — Les refus qui ne tiennent pas au QR sont inchangés.** Type hors liste blanche ou octets incohérents, image
non décodable (`ImageDecode` → `UNSUPPORTED_FILE_TYPE` — test neuf : un `.png` aux octets magiques valides et au corps
tronqué, attendu en `failed[]` et dans `failed/` ; aucun test d'import ne l'exerce aujourd'hui,
`grep -n -i "corrupt\|ImageDecode" crates/kesh-api/tests/inbox_import_e2e.rs` → 0), taille, lien symbolique, doublon (y compris un document sans
QR déposé deux fois), erreur de lecture : `failed[]` et `failed/`, comme avant. La réactivation d'un document retiré
(`discarded` → `to_complete`) vaut pour un document sans QR.

**AC4 — Migration** `crates/kesh-db/migrations/2026101?000001_imported_documents_without_qr.sql` (horodatage au-delà de
la dernière migration de `main` au moment du développement — la 30-1 peut en avoir ajouté une) :
- **un seul `ALTER TABLE imported_supplier_invoices`** portant, dans cet ordre : les sept `MODIFY COLUMN … NULL`
  (chacun réécrit la définition complète : type d'origine, `NULL`, sans `DEFAULT`) ; `ADD CONSTRAINT
  chk_imported_si_qr_all_or_nothing CHECK ((creditor_iban IS NULL) = (is_qr_iban IS NULL) AND …)` sur les sept ;
  `ADD CONSTRAINT uq_imported_supplier_invoice UNIQUE (supplier_invoice_id)` (plusieurs `NULL` permis ; elle **ne peut
  pas** échouer en 1062 sur une base peuplée : `mark_completed`, seul écrivain du lien, n'est appelé que par la
  complétion avec la facture qu'elle vient de créer, `repositories/imported_supplier_invoices.rs:183-187`, et la
  réactivation remet le lien à `NULL`, l. 247-258 — argument recopié au doc-comment de la migration) **puis**
  `DROP INDEX idx_imported_supplier_invoice`. L'unicité doit exister avant la suppression : l'ancien index porte la
  clé étrangère `fk_imported_si_supplier_invoice`, et MariaDB refuse de supprimer le dernier index d'une clé étrangère
  (erreur 1553 — précédents `20260714000002_email_templates_reminder.sql:28-34`, qui ajoute puis supprime, et
  `20260814000001_contacts_client_number_canonical.sql:37-40`, un seul `ALTER` à plusieurs clauses). Le squash de
  test ne voit pas ce défaut : vérifier par un `sqlx migrate run` sur une base **fraîche** et sur une base **déjà
  peuplée** (`kesh_302`, seed appliqué, au moins une ligne de staging `completed`) **avant** tout passage sur
  `kesh_e2e_302` — ⛔ **P8** : une fois appliquée, la migration ne se corrige plus, pas même un commentaire ;
- **en dernière instruction** `UPDATE _kesh_version SET kesh_version_min_required = '0.14.0' WHERE id = 1;` (P2) et
  **les dix crates à 0.14.0 dans la même PR** (P2-bis ; si la 30-1 les y a déjà portés, le constater au Dev Agent
  Record) (C-30-2-3) ;
- ligne au tableau de `docs/migrations-idempotence-audit.md` et **compteurs recomptés** (P5) ;
- `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` inspecté site par site (P6) ;
- triage P7 : ajout à `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`, justification standard d'un bump
  de `_kesh_version`, précédent `20261011000001`, l. 578-581) et compteur d'exemptions mis à jour (commentaires
  l. 1196-1204) ;
- squash de test régénéré (`crates/kesh-db/test-schema/`, `scripts/regen-test-schema.sh`) ; `test_schema_guard` vert.
- L'entité `ImportedSupplierInvoice` et `NewImportedSupplierInvoice` passent ces champs en `Option<…>` ;
  `NewImportedSupplierInvoice::without_qr(company_id, doc)` construit la ligne d'AC1 ; **inventaire clos** des
  lecteurs et constructeurs à reprendre (aucun `unwrap` neuf) : `inbox_import.rs` (audit),
  `routes/imported_supplier_invoices.rs` (complétion, retrait), `exports/csv_tables.rs:1559-1631` (cellule vide pour
  `None`), les tests unitaires de l'entité (`imported_supplier_invoice.rs:152-275`), les constructeurs littéraux des
  tests (`crates/kesh-api/tests/inbox_import_e2e.rs:334`, `crates/kesh-db/tests/imported_supplier_invoices_repository.rs:36-37`),
  `crates/kesh-db/tests/closing_checks.rs`, `crates/kesh-api/tests/exports_global_e2e.rs`,
  `crates/kesh-api/tests/admin_full_import_e2e.rs`, le type `imported-supplier-invoices.types.ts` et la page d'import
  (`exports/global.rs` appelle seulement `list_all_by_company` ; `kesh-seed` n'a aucun usage).
  Correction du 2026-10-15 (validation P2, L4) : `crates/kesh-api/tests/admin_full_import_e2e.rs`, cité plus haut, n'a
  aucun usage de cette table — à ne pas compter. Le dev recompte cet inventaire par `grep -rn
  "creditor_iban\|is_qr_iban\|creditor_address_type\|creditor_name\|creditor_country\|reference_type\|\.currency"` sur
  `crates/kesh-api/src` et `frontend/src` et l'écrit au Dev Agent Record.

**AC5 — La liste « Documents à traiter ».** La page `/supplier-invoices/import` s'intitule « Documents à traiter »
(menu, titre, section — C-30-2-18). Chaque ligne : nom du fichier, date d'import, **badge** « QR-facture » (créancier,
montant, référence) ou « Sans QR-code » ; actions **Aperçu**, **Compléter** (30-2a ; devient « Créer une facture » en
30-2b), **Retirer de la liste** (confirmation : « Retirer ce document de la liste ? Le fichier reste conservé. ») —
c'est l'actuel `discard` (C-30-2-5). Le rapport d'import dit, pour un document accepté sans QR, « sans QR-code » ou
« QR illisible ».

**AC6 — Aperçu.** Composant `DocumentPreview.svelte` (`frontend/src/lib/features/imported-supplier-invoices/`) :
prend une fonction de lecture (rend un `Blob`), affiche `<img>` pour `image/png` et `image/jpeg`, `<iframe>` pour
`application/pdf` (titre accessible = nom du fichier) ; **tout autre type** : aucun cadre, un lien « Télécharger »
seulement (un type futur n'ouvre pas un cadre de même origine) ; un état de chargement, un état « Le document n'a pas été
restauré » sur 410 (clé existante `imported-supplier-invoices-doc-gone`), un état d'erreur sinon ; révoque l'URL
`blob:` à la destruction et au changement de document. La CSP (`crates/kesh-api/src/middleware/csp.rs:24-32`) devient
**explicite** pour les trois directives en jeu (C-30-2-6) : `img-src 'self' data: blob:`, `frame-src 'self' blob:`,
`object-src 'self' blob:` — `object-src` n'est plus laissé à l'héritage de `default-src 'self'` : le document `blob:`
d'une `<iframe>` hérite de la CSP de la page, et la visionneuse PDF du navigateur peut y passer par un objet
embarqué. Un test unitaire de `csp.rs` fige la valeur (il n'en existe aucun aujourd'hui :
`grep -n "mod tests" crates/kesh-api/src/middleware/csp.rs` → 0).

**AC6-bis — L'aperçu d'un PDF, vu dans un vrai navigateur.** Chromium sans tête n'a pas de visionneuse PDF : aucun
E2E ne peut voir un PDF s'afficher, alors que la plupart des pièces sont des PDF. Le développeur **ouvre** donc, avec
le binaire construit et la CSP finale servie par lui, un PDF (synthétique, de `frontend/tests/e2e/fixtures/`) et un
PNG dans l'aperçu, dans un Chromium **à fenêtre** (Playwright `--headed` ou navigateur ordinaire sur le port du
worktree) — **obligatoire** — et dans Firefox **seulement s'il est installé sur la station** (son absence s'écrit au
Dev Agent Record, elle ne bloque pas), en thème clair et sombre ; il consigne au Dev Agent Record le
navigateur et sa version, le résultat (affiché / vide / bloqué, avec le message de la console s'il y en a un) et le
chemin des captures (sous `$GATE_LOGS`, jamais au dépôt si elles montrent autre chose qu'une fixture). Si l'aperçu est
bloqué, la CSP se corrige **et** le test de `csp.rs` avec elle, puis le contrôle se refait — **dans ces bornes** : seules
`img-src`, `frame-src` et `object-src` peuvent gagner `blob:` ; `default-src`, `script-src` et `frame-ancestors` ne
changent pas ; aucun attribut `sandbox` sur l'`<iframe>` n'est un remède (il bloque la visionneuse). Hors de ces bornes,
le constat va au rapport et la décision à l'orchestrateur. **Première hypothèse à tester** si l'aperçu est bloqué : le
`frame-ancestors 'none'` (`csp.rs:30`) hérité par le document `blob:` de l'`<iframe>` — hors des bornes, il se rapporte. Sans ce relevé, la 30-2a
n'est pas `done`.

**AC7 — Compléter un document sans QR** (formulaire existant de la page d'import, corps inchangé en 30-2a) : pas de
contrôle de devise (C-30-2-19), pas de routage IBAN (facture sans IBAN ni référence), pas de contrôle de montant
(`staging.amount` est `NULL`) ; `expected_payment_amount` = `NULL`. Un document avec QR garde exactement les contrôles
actuels (`routes/imported_supplier_invoices.rs:183-252`) ; le message de `CURRENCY_NOT_SUPPORTED` (l. 187, « Écartez
la facture et saisissez-la manuellement ») devient « Retirez le document de la liste, ou créez la facture sans
document puis joignez-le » — en 30-2a « retirez le document de la liste » seulement (le rattachement n'existe qu'en
30-2c, qui complète la phrase). ⚠️ **Livrable seule pour le test, non pour l'usage** : après la 30-2a, une facture
complétée depuis un document sans QR naît sans IBAN ni référence, et aucune route ne les ajoute ; sans release entre
les trois PR (C-30-0-1), ce manque ne sort pas. L'audit du retrait n'émet plus `iban_present` à partir d'une
chaîne : `iban_present: creditor_iban.is_some()`.

**AC8 — Textes.** Quatre locales : « Documents à traiter » et variantes (titre, menu `nav-supplier-invoices-import`,
section `-to-complete`, vide `-empty` « Aucun document à traiter. »), libellés du badge, de l'aperçu, de « Retirer de la
liste » et de sa confirmation, des deux mentions du rapport ; K5 de la préparation de clôture : titre « Documents
fournisseur à traiter », message « { $count } document(s) à traiter, toutes dates confondues. », lien « Ouvrir les
documents à traiter » (`messages.ftl:997-1000`, clés `k5-title`, `-help`, `-attention`, `-link` ; `FiscalYearChecks.svelte:224-230`). Retrait de
`imported-supplier-invoices-error-no-qr-code-found` (quatre locales) et de son entrée de `error-label.ts:33` ;
`lint-i18n-ownership`, `i18n-keys.test.ts`, les gardes de parité inter-locales et `textes_coherents.rs` verts.
Inventaire des textes à reprendre — les **jetons** du vocabulaire retiré, non une phrase :
`grep -rnE "compléter|Compléter|Importer des factures|[Éé]cart|justificatif|facture d.origine|facture\(s\) importée"`
sur `crates/kesh-i18n/locales/` (quatre locales), `frontend/src`, `docs/manual/fr`, `README.md` — chaque site traité ou
écrit comme conservé au Dev Agent Record. Sites connus : `messages.ftl:2118-2162` (`discard-confirm`,
`discard-conflict`, `discarded` « Facture écartée. », `err-not-pending`, `report-accepted` « facture(s) importée(s) »,
`subcentime` « écarter cette facture », `view-doc`, `view-source`), `user-manual.tex:1735-1736`,
`admin-manual.tex:855-880`. Les clés du formulaire de complétion en ligne qui deviennent orphelines en 30-2b
(`imported-supplier-invoices-line-*`, `-field-*`, `-lines`, `-sum-ttc`, `-target`, `-mismatch`, `-err-amount`…) sont
**retirées** des quatre locales par la 30-2b, après recherche de chaque clé (`grep -rnF`) dans `frontend/src`.
**Inventaire complété (validation P2, L6)** : la même recherche se fait dans **les quatre locales** (pour `de-CH`,
`it-CH`, `en-CH`, sur les **clés** trouvées en `fr-CH`, leurs valeurs n'ayant pas les mêmes mots), avec aussi les jetons
`complét` et `importée` (formes fléchies : `audit-log-action-imported-supplier-invoice-completed` « Facture importée
complétée », `messages.ftl:2536`) ; texte prescrit pour `fiscal-years-checks-k5-help` : « Les documents fournisseur qui
attendent d'être traités, toutes dates confondues. » ; le repli de `frontend/src/lib/features/fiscal-years/closing-checks-labels.ts:63`
suit. **Codes `qrError`** (L7) : l'écran affiche, pour un document `unreadable`, « QR illisible » suivi du libellé de
`qrError` par les clés existantes `imported-supplier-invoices-error-invalid-spc-payload`, `-invalid-iban`,
`-pdf-render-error`, `-field-too-long` (`error-label.ts`) — elles gardent un usage —, plus la clé **neuve**
`-qr-amount-out-of-range` (AC2 ; liste normative, P5 T1) ; seule `no-qr-code-found` est retirée,
et la constante `ERR_NO_QR_CODE_FOUND` (`inbox_import.rs:54`) est **supprimée** : un document sans QR porte
`qr: "none"` sans `qrError`, et une constante morte ferait rougir `clippy -D warnings`.
Les libellés des sept motifs de `INVALID_PAYMENT_DETAILS` (AC12-bis, quatre locales) relèvent de la 30-2b. Les
libellés des erreurs neuves (`QR_FIELDS_FROM_DOCUMENT` en 30-2b, `INVALID_PAYMENT_DETAILS` en 30-2b,
`SOURCE_DOCUMENT_ALREADY_ATTACHED` en 30-2c) suivent le patron des clés `error-…` présentes dans les quatre locales (`error-check-constraint`, `messages.ftl:25`) —
non `errors.rs:3045-3049`, dont la clé `error-source-document-not-found` n'existe dans aucune locale (repli en dur).

**AC9 — Documentation de la 30-2a.** Manuel utilisateur FR § *Importer des factures depuis un dossier*
(`docs/manual/fr/user-manual.tex:1712-1741`) réécrit en « Documents à traiter » : tout PDF ou image entre, avec ou
sans QR ; l'aperçu ; « Retirer de la liste » ; ce qui reste refusé et le dossier `failed/` ; le fichier quitte le
dossier dès l'import ; **renvoi à la sauvegarde de `KESH_DOCUMENTS_DIR`** (manuel administrateur) et à #503 ; K5
(l. 801 et 808) ; PDF régénéré et contrôlé aplati. ⛔ **Deux phrases gardées par `textes_coherents.rs` G5**
(`aucun_dossier_surveille`, l. 584-614) restent mot pour mot : « L'import ne se déclenche pas tout seul » au manuel,
« import lancé depuis l'écran » au `README.md:44`. Manuel d'administration § import de factures (`admin-manual.tex:855-880`)
et `README.md:44` mis à jour (documents avec ou sans QR, « documents à traiter »). `docs/api-external.md` n'a **aucune
section** pour ces routes (`grep -n -i "inbox\|imported" docs/api-external.md` ne rend que des renvois : l. 253, code
K5, et l. 768, 788) : la 30-2a la **crée** — `POST /inbox-import` (rapport, `qr`/`qrError`), `GET /imported-supplier-invoices`
(champs nullables), `POST …/{id}/discard`, `GET …/{id}/source-document`. CHANGELOG `## [0.14.0] — Non publié` :
`### Ajouté` (documents sans QR, aperçu) et `### Modifié` (libellés, K5, CSP). Le manuel d'administration, l. 1456 et 1599, dit que
`KESH_DOCUMENTS_DIR` porte « les PDF des factures émises » : il porte aussi les **documents fournisseur**, qui en
deviennent la majeure partie — les deux lignes le disent.

**AC9-bis — Passer à la 0.14.0** (migration breaking, `min_required` 0.14.0). Manuel d'administration : sous-section
`\subsubsection{Passer à la 0.14.0}\label{sec:maj-0-14-0}`, avant celle de la 0.13.4 (`admin-manual.tex:2012`), sur
leur patron : sauvegarder **avant** la mise à jour (dump, `.keshbackup` **et** le dossier `KESH_DOCUMENTS_DIR`, que le
`.keshbackup` n'emporte pas — #503) ; une fois la 0.14.0 démarrée, la 0.13.x **ne démarre plus** sur la base ; le retour
arrière passe par la restauration de cette sauvegarde ; et un paragraphe **Intégrations par l'API**, sur le patron de
`admin-manual.tex:2022-2025` : champs QR de `GET /imported-supplier-invoices` désormais **nullables** (un client qui
les lit comme chaînes casse), `NO_QR_CODE_FOUND` et les autres refus liés au QR ne sortent plus en `failed[]` (rapport
`qr`/`qrError`), `POST /supplier-invoices` valide désormais les coordonnées de paiement (`INVALID_PAYMENT_DETAILS`,
AC12-bis — à compléter par la 30-2b), lecture du document d'une facture ouverte aux clés `read` (30-2c). CHANGELOG :
bandeau « ⚠️ **À faire en passant à la 0.14.0** — le détail est au manuel d'administration, *Passer à la 0.14.0* : … »
en tête de la section `[0.14.0]`, sur le patron des l. 21-27, 54 et 135, **avec** l'item « Intégrations par clé
d'API » du patron (`CHANGELOG.md:21-27`). PDF du manuel d'administration régénéré.

### Partie b — `30-2b-creation-cote-a-cote`

**AC10 — Un formulaire, une page.** `SupplierInvoiceForm.svelte` (`frontend/src/lib/features/supplier-invoices/`)
extrait du formulaire de `routes/(app)/supplier-invoices/+page.svelte` (champs, lignes, total TTC vivant, projet,
erreurs) ; la page `routes/(app)/supplier-invoices/new/+page.svelte` l'affiche :
- **sans document** (`/supplier-invoices/new`) — avec le scan navigateur 12.4 ; le bouton « Enregistrer une
  facture » de la liste y mène ; le formulaire en ligne de la liste disparaît (C-30-2-11) ; après enregistrement,
  l'écran va à la fiche de la facture créée ;
- **à côté du document** (`/supplier-invoices/new?document=<id>`) — aperçu (AC6) fixe à gauche, formulaire à droite,
  empilés sous 1024 px ; **le document reste visible pendant toute la saisie** (pas de modal, pas de défilement qui
  l'ôte). Le document se retrouve dans `GET /imported-supplier-invoices?status=to_complete` (aucune lecture unitaire
  n'existe) ; un `document` inconnu, d'une autre société ou qui n'est plus `to_complete` donne **le même** message
  (« Ce document n'est plus à traiter. ») et un lien vers la liste, sans formulaire — ne pas distinguer « inconnu » de
  « autre société » est voulu.
Le formulaire de complétion en ligne de la page d'import disparaît ; *Compléter* devient « Créer une facture » et mène
à `?document=<id>`.

**AC11 — Avec QR.** IBAN ou QR-IBAN, référence et montant attendu pré-remplis et **en lecture seule** ; le total TTC
vivant est comparé au montant du QR (indicateur existant, `import/+page.svelte` `amountMismatch`) ; le serveur garde
le refus `AMOUNT_MISMATCH`. Le corps envoyé n'en porte aucun ; le serveur refuse en `400 QR_FIELDS_FROM_DOCUMENT` un
corps qui en porte pour un document avec QR (C-30-2-9) — décision lue sur la ligne **verrouillée**
(`staging.creditor_iban.is_some()` après `find_by_id_scoped_for_update`, à côté des contrôles de devise et de montant),
jamais sur une lecture antérieure. Le scan navigateur n'est pas proposé.

**AC12 — Sans QR.** Tous les champs se saisissent, y compris IBAN, référence, montant attendu (facultatifs) et projet ;
le serveur les prend dans le corps et les valide par la **conversion commune** de l'AC13 — ⚠️ aujourd'hui,
`POST /supplier-invoices` **ne valide rien** de ces champs : `create_in_tx` les insère tels quels
(`repositories/supplier_invoices.rs:340-343`, `:526-539`), et le formulaire ne route un QR-IBAN vers `creditorQrIban`
que s'il vient d'un scan. D'où **AC12-bis**. À l'écran, un IBAN saisi à la main est classé QR-IBAN ou non par **la
fonction existante** `qrIbanHasValidQrIid` (`routes/(app)/bank-accounts/+page.svelte:276-281`), **extraite** dans un
module partagé (`frontend/src/lib/shared/utils/iban.ts`, avec la condition pays CH/LI qu'elle n'a pas) et employée par
les deux écrans — une seule définition côté frontend (C-30-2-20) ; la saisie est routée vers `creditorIban` ou
`creditorQrIban`.

**AC12-bis — Les coordonnées de paiement se valident à un seul endroit** (C-30-2-20, arbitrages de l'orchestrateur P2 et
P3). La conversion commune de l'AC13 contrôle, pour **les deux routes** (`POST /supplier-invoices` et la complétion sans
QR — un seul chemin, c'est voulu), avec les types existants de `kesh-core` — une seule définition côté serveur :
- **Normalisation d'abord, une seule fois par côté, sur un ensemble de caractères écrit** (P4, F4 : `\s` en JavaScript
  et `char::is_whitespace` en Rust ne coïncident pas — U+FEFF dans l'un, U+0085 dans l'autre) : sont retirés **les
  caractères `White_Space` d'Unicode plus U+200B (espace de largeur nulle) et U+FEFF (BOM)** — espace, tabulation,
  U+00A0, U+202F compris. Côté serveur, `normalize_payment_field` (`c.is_whitespace() || c == '\u{200B}' || c ==
  '\u{FEFF}'`) avant `Iban::new`, qui n'ôte lui-même que l'espace ASCII (`crates/kesh-core/src/types/iban.rs:136`) et
  refuse tout non-ASCII ; côté écran, `normalizeIban` du module partagé `lib/shared/utils/iban.ts`
  (`/[\s\u0085\u200B]/g`, `\s` couvrant déjà U+FEFF), employée par la classification QR-IBAN et par l'envoi. Un test de
  chaque côté porte **la même liste** de caractères (U+0020, U+0009, U+00A0, U+202F, U+0085, U+200B, U+FEFF). Une chaîne
  vide ou faite de ces caractères vaut **absente** (pour AC11 comme ici).
- **`creditorIban`** : `kesh_core::types::Iban::new` (`iban.rs:135`) — format, ASCII seul, **code pays présent dans la
  table intégrée** `IBAN_LENGTHS` (`iban.rs:13-101`) et **longueur propre à ce pays**, somme de contrôle MOD-97 — **et**
  pas un QR-IBAN (`QrIban::new` doit échouer) ;
- **`creditorQrIban`** : `kesh_core::types::QrIban::new` (`qr_iban.rs:38`) — IBAN CH ou LI dont le QR-IID (positions 5
  à 9) est dans 30000-31999 ; une valeur qui n'est **pas un IBAN valide** (`Iban::new` échoue, propagé l. 39) rend
  `INVALID_IBAN` sur `field: creditorQrIban`, un IBAN valide hors plage QR rend `NOT_A_QR_IBAN` (deux tests) ;
- les deux ne sont pas renseignés ensemble ;
- **référence** — une seule règle de type (arbitrage P3, F3-3) : avec un QR-IBAN, elle est **obligatoire** et doit
  passer `kesh_qrbill::validation::validate_qrr` (`validation.rs:141`) une fois ses blancs retirés ; avec un IBAN
  ordinaire ou sans IBAN, elle est un **texte libre** de 64 caractères au plus (colonne `payment_reference VARCHAR(64)`,
  `20260628000001_supplier_invoices.sql:39`), **27 chiffres compris** (ancienne référence BVR/ESR d'une facture
  ancienne — c'est l'usage de Guy) : jamais de refus « forme QRR » ; elle est seulement rognée. En aval, le lot pain.001
  l'émet en référence non structurée (`crates/kesh-db/src/repositories/payment_batches.rs:793`) ;
- valeurs enregistrées **normalisées** (`Iban::as_str`, QRR sans blancs, texte libre rogné).
Refus : `400 INVALID_PAYMENT_DETAILS`, `details.field` ∈ {`creditorIban`, `creditorQrIban`, `paymentReference`} et
`details.reason` ∈ {`INVALID_IBAN`, `QR_IBAN_IN_IBAN_FIELD`, `NOT_A_QR_IBAN`, `BOTH_IBANS`, `QRR_REQUIRED`,
`INVALID_QRR`, `REFERENCE_TOO_LONG`} — **sept** motifs (codes, jamais une phrase). Chacun a son **libellé** dans les
quatre locales (clés `supplier-invoices-payment-error-invalid-iban`, `-qr-iban-in-iban-field`, `-not-a-qr-iban`,
`-both-ibans`, `-qrr-required`, `-invalid-qrr`, `-reference-too-long`), affiché sous le champ nommé (AC16) ; le message
de l'erreur elle-même suit le patron des clés `error-…` présentes dans les quatre locales (`error-check-constraint`,
`messages.ftl:25`).
**Effet sur la création manuelle actuelle**, à écrire **tel quel** au CHANGELOG (`### Modifié`), à `api-external.md`, au
manuel (AC17) et au *Passer à la 0.14.0* (AC9-bis, intégrations) : `POST /supplier-invoices` **refuse désormais** un
IBAN dont le format, le code pays (absent de la table intégrée de Kesh), la longueur pour son pays ou la somme de
contrôle est invalide ; un QR-IBAN dans `creditorIban` ; un IBAN ordinaire dans `creditorQrIban` ; les deux ensemble ;
un QR-IBAN sans référence QRR valide ; une référence de plus de 64 caractères. Il **accepte toujours** un IBAN étranger
valide d'un pays de la table, et toute référence libre de 64 caractères au plus, 27 chiffres compris ; les blancs (y
compris insécables) sont retirés des IBAN, un champ vide vaut absent. Les factures existantes ne changent pas. ⚠️ **Pays
absents de la table** (`iban.rs:13-101`, dont le commentaire, l. 11-12, dit : « Les entrées doivent être vérifiées contre le document officiel "IBAN Registry" … » — par exemple `CI`, `BF`, `BJ`, `ML`,
`NE`, `TG`, `MG`) : un IBAN valide de ces pays, accepté aujourd'hui, sera refusé — angle mort écrit, et dit au CHANGELOG.
⚠️ **Lot de paiement** : un IBAN étranger accepté à la saisie reste refusé à la création d'un lot pain.001, qui
n'accepte que CH/LI (`crates/kesh-db/src/repositories/payment_batches.rs:359`, `kesh_qrbill::validation::validate_iban`) — comportement existant, à dire au manuel.
Tests (T-b1) : **un cas par motif** (sept, dont une référence de 65 caractères pour `REFERENCE_TOO_LONG`) ; IBAN
allemand valide accepté ; IBAN suisse avec espaces insécables accepté et stocké sans blancs ; la référence libre
« Facture E2E » avec un IBAN ordinaire acceptée (corps de `payment-batches.spec.ts:107-108`, qui doit rester vert) ;
**une référence de 27 chiffres avec un IBAN ordinaire acceptée** ; QRR saisie en groupes de cinq avec un QR-IBAN
acceptée et stockée sans blancs ; `""` traité comme absent. Le dev recense les tests et données qui posent un IBAN ou
une référence (`grep -rn "creditorIban\|creditorQrIban\|paymentReference\|creditor_iban\|payment_reference"
crates/*/tests frontend/tests crates/kesh-seed`) et corrige ce qui deviendrait invalide avec des valeurs synthétiques
valides. Un document **avec** QR n'est pas concerné (ses valeurs viennent du QR, déjà validées au décodage,
`INVALID_IBAN`).

**AC13 — Un seul corps.** `complete_import` désérialise `CreateSupplierInvoiceRequest`
(`routes/supplier_invoices.rs:238`) ; la conversion en `NewSupplierInvoice` est une fonction commune aux deux routes
(aucune copie du bloc l. 340-362) et porte la validation de l'AC12-bis ; `project_id` est honoré à la complétion
(fin de « extension future », `routes/imported_supplier_invoices.rs:269`). La réponse de la complétion est relue par la
même fonction que le GET (AC18) une fois la transaction validée.
`CompleteImportRequest` et `CompleteImportLineRequest` disparaissent ; un client actuel (sans les champs ajoutés)
reste accepté (test).

**AC14 — Fournisseur proposé** (arbitrage de l'orchestrateur : le **rappel** avant la précision — rien n'est présélectionné et
trois candidats au plus s'affichent ; C-30-2-7). Avec QR seulement ; sans QR, rien n'est proposé. Deux sources, dans cet
ordre, sans doublon, **trois au plus** au total — *sans QR, aucune donnée d'où proposer (ni nom ni IBAN du créancier) :
d'où « avec QR seulement » ; un document sans QR se rattache par la liste ou par la création en ligne* :
1. **par IBAN** : les fournisseurs actifs qui ont déjà une facture fournisseur portant l'IBAN ou le QR-IBAN du créancier
   (les contacts n'ont pas d'IBAN ; seules les factures en portent : `supplier_invoices.creditor_iban`,
   `creditor_qr_iban`). Lecture neuve `GET /api/v1/imported-supplier-invoices/{id}/supplier-candidates` (Comptable+,
   bornée à la société) : l'IBAN est lu **sur le document**, jamais passé dans l'URL ; comparaison sur la forme
   normalisée des deux côtés (`REPLACE` successifs de l'espace, de U+00A0 et de U+202F sur `UPPER(…)` — les autres blancs
   de l'ensemble d'AC12-bis ne sont pas retirés en SQL : une facture antérieure qui en porterait n'est pas retrouvée,
   perte de rappel écrite en angle mort ; les factures antérieures à l'AC12-bis pouvant porter des
   espaces) ; factures **annulées exclues** ; contacts archivés ou non fournisseurs exclus ; ordre `MAX(invoice_date) DESC`
   puis `contacts.id` ; **`LIMIT 3`** dans la requête ; réponse `200 { candidates: ContactResponse[] }`, liste vide pour un
   document sans QR ; `404 IMPORTED_INVOICE_NOT_FOUND` pour un document inconnu ou d'une autre société (patron de la
   complétion). Le `WHERE REPLACE(UPPER(…))` n'est pas indexable : balayage des factures de la société, acceptable à
   l'échelle visée (écrit). L'IBAN n'est ni dans l'URL ni dans la réponse ni dans les journaux ;
2. **par nom** : `listContacts({ search: searchTermsFor(creditorName), isSupplier: true, limit: 20 })`, puis la
   fonction pure `proposeSuppliers(candidats, creditorName)` (`frontend/src/lib/features/supplier-invoices/`), qui
   filtre puis classe.
**Normalisation d'un nom** — `supplierNameKey(nom)`, pure, dans cet ordre : (a) NFC, minuscules ; (b) `ä`→`ae`,
`ö`→`oe`, `ü`→`ue`, `ß`→`ss` ; (c) repli des autres accents (`fold` de `duplicate-probe.ts` : `é`→`e`, `à`→`a`) ;
(d) tout caractère qui n'est ni lettre ni chiffre devient une espace (« S.à r.l. » → `s a r l`) ; (e) découpage en mots ;
(f) retrait des **formes juridiques**, reconnues comme suites de mots, **de la plus longue à la plus courte**, en un seul
passage de gauche à droite par longueur — liste ordonnée : `s a g l`, `s a r l` (quatre mots) ; `s r l` (trois) ; `s a`
(deux) ; `gmbh`, `sagl`, `sarl`, `srl`, `sas`, `ltd`, `inc`, `cie`, `ag`, `sa`, `kg`, `co`, `ab` (un) — puis des mots de
liaison `et`, `und`, `e`, `and` (sans cet ordre, « S.à r.l. » perdrait `s a` et garderait `r`, `l`) ; (g) l'ensemble des mots restants est la **clé**. La règle (b) s'applique aux deux côtés : « Müller » et
« Mueller » donnent tous deux `mueller` (« Muller », sans tréma ni `e`, reste distinct : angle mort écrit).
**Recherche serveur** — `searchTermsFor(nom)` envoie les mots de la clé et, pour chaque mot contenant `ae`/`oe`/`ue`,
sa variante à tréma et sa variante sans `e` (`mueller müller muller`), le serveur cherchant un « OU » de mots
(`crates/kesh-db/src/util/search.rs:87`) en collation qui confond `ü` et `u`.
**Égalité et éligibilité** — un candidat est éligible si sa clé et celle du créancier sont **égales**, ou si l'une est
**incluse** dans l'autre (inclusion d'ensembles de mots), la plus petite n'étant pas vide ; ordre : IBAN d'abord, puis
clés égales, puis inclusion, puis `rank` (`duplicate-probe.ts`) pour départager.
Chaque proposition s'affiche « Proposé : <nom> » avec son motif (« même IBAN » ou « nom proche ») et un bouton
« Retenir » ; **aucune n'est présélectionnée**, la liste déroulante reste vide tant que l'utilisateur n'a ni retenu ni
choisi. Vitest de `supplierNameKey` et `proposeSuppliers` : « DUBARDE S.à r.l. » contre « Dubarde Sàrl Genève »
(proposé, inclusion) ; « Müller AG » contre « Mueller AG » (proposé, égalité) ; « Dubarde SàRL » contre « DUBARDE
S.à r.l. » (égalité) ; un candidat qui ne partage qu'une forme juridique ou un mot de liaison (« Zurcher Sàrl » contre
« Nicolet Sàrl ») non proposé ; IBAN avant nom ; trois au plus ; sans doublon. Test Rust de `supplier-candidates` : IBAN
avec espaces en base retrouvé, autre société exclue, contact archivé ou non fournisseur exclu, facture annulée
exclue, ordre et plafond de trois. **Test d'intégration de la recherche** (`contacts`, Rust) : contacts « Müller AG »,
« Mueller AG », « Muller SA » ; `search` = `mueller müller muller` ; les trois reviennent — la collation effective de
`contacts.name` (héritée du serveur, `20260814000001_contacts_client_number_canonical.sql:6`) est relevée au Dev Agent
Record. Les mots de moins de trois caractères et les mots vides d'InnoDB ne sont pas cherchés (`MIN_TOKEN_LENGTH`,
`duplicate-probe.ts:64`) : angle mort écrit. Le manuel le dit :
l'absence de proposition ne prouve pas l'absence du fournisseur, **et sa présence ne prouve pas que c'est le bon**
(deux sociétés d'un groupe aux noms proches restent proposées).

**AC15 — Créer le fournisseur sans quitter.** « Nouveau fournisseur » ouvre un panneau de la page (C-30-2-8) : type
(Entreprise par défaut), nom pré-rempli du créancier QR, adresse structurée pré-remplie pour un QR de type `S`,
`isSupplier: true`, signal « contacts proches » (socle 22-2a) avant l'enregistrement. *Créer* appelle `createContact`
(`POST /contacts`, Comptable+), ajoute le contact à la liste et le **sélectionne** ; la saisie de la facture (lignes
comprises) est intacte ; une erreur de création (IDE en double, validation) s'affiche dans le panneau sans rien
perdre ; *Annuler* ne crée rien.

**AC16 — Après création.** Le document passe `completed`, quitte la liste ; l'écran va à la fiche de la facture
créée. Un refus (`AMOUNT_MISMATCH`, `INVALID_PAYMENT_DETAILS` — le libellé de son motif sous le champ nommé par
`details.field`, AC12-bis —, compte non imputable, exercice,
`IMPORT_NOT_PENDING_COMPLETION` si le document a été traité ailleurs entre-temps) s'affiche sans perdre la saisie —
Vitest de la page `new` : un refus simulé laisse champs et lignes intacts. La page `/supplier-invoices/new` a un état de
chargement et un état d'erreur de la lecture du document (patron d'AC6) ; la création est **Comptable+** : pour un
autre rôle, le bouton « Enregistrer une facture » et la page ne proposent rien (patron existant de la liste).

**AC17 — Documentation de la 30-2b.** Manuel FR : § saisie d'une facture fournisseur (`sec:saisie-facture-fournisseur`)
et § documents à traiter — création côte à côte, coordonnées QR verrouillées, fournisseur proposé et « Retenir »,
création en ligne, facture sans document, validation des coordonnées de paiement (AC12-bis) ; le chemin du menu de `user-manual.tex:1570` (« *Nouvelle facture* ») suit le libellé réel du bouton, « Enregistrer une facture » (`messages.ftl:2223`) ; PDF régénéré. `api-external.md` : corps de
`POST /imported-supplier-invoices/{id}/complete` = celui de `POST /supplier-invoices`, `QR_FIELDS_FROM_DOCUMENT`,
`INVALID_PAYMENT_DETAILS` sur les deux routes (c'est à la 30-2b, non à la 30-2a, d'écrire ces deux refus).
CHANGELOG complété (`### Modifié` : validation des coordonnées de paiement). i18n quatre locales.

### Partie c — `30-2c-document-joint-et-consulte`

**AC18 — La fiche de la facture sait si elle a un document.** `SupplierInvoiceResponse` gagne `sourceDocument`, à
**trois états** — la **discipline** est celle de `settlement_cancellable` (« `None` veut dire non calculé ici, jamais non »,
mais celui-ci sort `null`, sans `skip_serializing_if` ; `sourceDocument` distingue en plus l'absence, et le type
TypeScript le dit : `sourceDocument?: SourceDocument | null`,
`routes/supplier_invoices.rs:84-89`) : champ **absent** = non calculé (réponses de `from_parts` : création,
`supplier_invoices.rs:369`) ; `null` = aucun document ; objet `{ originalFilename, mimeType, byteSize }` = un document
(type Rust `Option<Option<SourceDocumentInfo>>`, `skip_serializing_if` sur l'extérieur). Il est calculé dans la vue
d'instantané du GET (`load_with_settlement_cancellation` → `get_settlement_view`, `repositories/supplier_invoices.rs:1519`,
« un seul instantané »), par une lecture **ordinaire** du staging lié dans ce même instantané. La lecture partagée
(`find_by_supplier_invoice_id_scoped`, `repositories/imported_supplier_invoices.rs:157-171`) gagne `AND status =
'completed'`, qui rend l'invariant explicite pour AC18, AC21 et AC22. La complétion (`…/complete`) relit sa réponse par
le GET (AC13). Test : la réponse de `complete` porte l'objet ; celle de `POST /supplier-invoices` n'a pas le champ. La fiche
affiche « Voir le document » (aperçu AC6 dans un panneau) **seulement** s'il existe ; sinon « Joindre un document »
(Comptable+) — plus de bouton qui rend 404. Quand il existe, la fiche offre aussi « Détacher le document » (AC24).

**AC19 — Joindre après coup.** `POST /api/v1/supplier-invoices/{id}/source-document` `{ importedSupplierInvoiceId }`
(Comptable+, ouverte aux clés `read-write` comme `…/complete`). La transaction vit dans `attach_source_document_once`
(une tentative), enveloppée par `retry_app_on_deadlock` :
1. `SELECT … FOR UPDATE` du staging (borné à la société) → `404 IMPORTED_INVOICE_NOT_FOUND` ; statut ≠ `to_complete`
   → `409 IMPORT_NOT_PENDING_COMPLETION` ;
2. `SELECT … FOR UPDATE` de la facture par sa clé primaire, bornée à la société → `404` ;
3. `mark_completed(staging, facture)` — **aucune lecture préalable** d'un document déjà lié (C-30-2-21) : c'est
   l'unicité de `supplier_invoice_id` (AC4) qui refuse, et sa violation (`DbError::UniqueConstraintViolation` sur
   `uq_imported_supplier_invoice`, la seule unicité que touche cet `UPDATE`) se traduit en
   `409 SOURCE_DOCUMENT_ALREADY_ATTACHED` ; toute autre erreur de base reste une erreur ;
4. audit `imported_supplier_invoice.attached` (`supplier_invoice_id`, `original_filename`).
Tout statut de facture accepté (ouverte, payée, annulée) ; aucun champ de la facture modifié (C-30-2-12).
**Ordre des verrous** : staging (enregistrement, clé primaire), facture (enregistrement, clé primaire), puis entrée
de l'index d'unicité à l'`UPDATE`. Aucune lecture verrouillante sur l'index d'unicité, donc **aucun verrou
d'intervalle de notre fait** : deux rattachements simultanés vers **deux factures différentes** ne s'attendent pas —
**sauf** le contrôle de doublon d'InnoDB, qui pose des verrous partagés de clé suivante quand une entrée de **même clé**
subsiste dans l'index, vivante ou marquée supprimée et pas encore purgée (cas d'un détachement récent, AC24) ; un
interblocage est alors possible, rare, et rejoué par `retry_app_on_deadlock`. Le Dev Agent Record ne recopie pas
« aucun verrou d'intervalle » comme une garantie. Deux
rattachements vers la **même** facture se sérialisent sur l'étape 2 ; le second reçoit le `409` à l'étape 3. Aucun
chemin existant ne verrouille une facture **puis** un staging : aucune lecture **verrouillante** du staging n'existe
dans `crates/kesh-db/src/repositories/supplier_invoices.rs` (`grep -c "imported_supplier"` → 0 au `15686d55` ; après
AC18, ce fichier **lira** le staging, mais par une lecture ordinaire — le contrôle à refaire au Dev Agent Record est
`grep -n "FOR UPDATE"` croisé avec `imported_supplier`, même chose pour la lecture de l'écriture, AC21) ;
`complete_import_once` crée sa facture, il n'en verrouille aucune existante, si bien que l'ordre staging puis facture ne ferme aucun cycle — l'inverser non plus (M-c7, déclarée
équivalente). Le Dev Agent Record écrit le tableau des chemins (`complete_import_once`, `discard_import`, la
réactivation, `pay_in_tx`, `cancel_in_tx`, `cancel_settlement_in_tx`, l'acceptation de la 30-1, le détachement).
**Naissance de la vue** : la transaction ne fait **aucune** lecture ordinaire (deux lectures verrouillantes, un
`UPDATE`, un `INSERT` d'audit) : aucun instantané n'y naît. **Réponse** : `200` et la `SupplierInvoiceResponse` relue
par `load_with_settlement_cancellation` une fois la transaction validée (patron de `pay`, `routes/supplier_invoices.rs:398-401`,
et d'AC13), `sourceDocument` calculé — l'écran l'affiche sans seconde lecture ; le détachement (AC24) répond de même.
Le corps transactionnel est exposé sans `COMMIT` (`attach_in_tx`, dépôt) pour les tests. **Tests** (rouges d'abord,
**sans rejeu**, pour qu'un interblocage se voie) :
- **déterministe, sous verrou tenu** (patron de M-c10) : une transaction A rattache D1 à la facture I1 par
  `attach_in_tx` **sans valider** ; pendant ce temps, `attach_source_document_once` de D2 à I2 (I2 = I1 + 1, aucune entrée
  de l'index entre elles ni au-delà) doit **aboutir** sous `tokio::time::timeout` de 5 s ; puis A valide. C'est le test
  qui tient le retrait de la lecture verrouillante (C-30-2-21) : réintroduite, elle pose un verrou d'intervalle qui fait
  attendre B (M-c11) ;
- **relevé au banc, 2026-10-15 (orchestrateur, MariaDB 10.11.16 de dev, `REPEATABLE-READ`)** — les deux validations P2 et
  P3 ont soupçonné que le **contrôle de doublon** d'InnoDB pose à lui seul un verrou partagé sur le successeur
  (supremum), ce qui ferait attendre B même sans lecture verrouillante. **Mesuré : non.** Banc reproductible : deux
  tables jetables `inv(id PK)` et `doc(id PK, inv_id NULL UNIQUE, FOREIGN KEY (inv_id) REFERENCES inv(id))` ; connexion A :
  `BEGIN; SELECT … FROM doc WHERE id = 1 FOR UPDATE; SELECT … FROM inv WHERE id = X FOR UPDATE; UPDATE doc SET inv_id = X
  WHERE id = 1;` sans valider ; connexion B : la même chose sur `doc` 2 et `inv` Y. Couples (X, Y) essayés : (10, 20),
  (20, 10), (30, 10), (10, 30), **avec et sans** liens déjà présents dans l'index (15, 25) : B aboutit **chaque fois en
  moins de 0,35 s**, aucune attente. Le test « B sous 5 s » est donc gardé tel quel.
- Le banc ne reproduit pas tout le schéma réel (l'`UPDATE` réel touche aussi `status`, donc
  `idx_imported_company_status`, et la table porte `fk_imported_si_company`) : c'est le **test déterministe sur le vrai
  schéma** qui fait foi.
- **Bascule écrite, si le test rougit au développement** sur le code non muté (version ou réglage différent du banc) :
  le rattachement se **sérialise par société** par la primitive existante
  `bank_accounts::acquire_company_sentinel_lock` (`repositories/bank_accounts.rs:1613`, `SELECT id FROM companies WHERE id
  = ? FOR UPDATE`), prise **après** le staging et la facture et **avant** l'`UPDATE` — ordre staging → facture →
  `companies` → entrée d'unicité. *(Correction P4, F1 = R1 : la P3 la plaçait en tête et la disait « déjà employée par
  `create_in_tx` » ; `create_in_tx` ne la prend que pour une facture à projet, par `projects::validate_taggable_in_tx`,
  `projects.rs:101`, appelé l. 361-362, et une sentinelle en tête formait deux cycles.)* Ordre de chaque chemin, sous la
  bascule :

  | Chemin | Ordre des verrous | Cycle avec le rattachement ? |
  |---|---|---|
  | rattachement (bascule) | staging X → facture X → `companies` X → entrée d'unicité | — |
  | complétion (`complete_import_once`) | staging X → `companies` X si projet (`validate_taggable_in_tx`) → `companies` S (verrous partagés pris par l'insertion de l'écriture, `journal_entries::create_in_tx` — décrits au doc-comment `supplier_invoices.rs:291-293`) | non : même document → sérialisés sur le staging, avant `companies` ; autre document → aucune ressource commune avant `companies` |
  | `pay_in_tx`, `cancel_in_tx`, `cancel_settlement_in_tx` | facture X (`supplier_invoices.rs:725`, `:1071`, `:1314`) → pour `pay_in_tx`, compte bancaire X (`bank_accounts … FOR UPDATE`, l. 759-761) → `companies` S (insertion de l'écriture, `journal_entries::create_in_tx`) | non : même facture → sérialisés sur la facture, avant `companies` ; le rattachement ne touche pas `bank_accounts` |
  | détachement (AC24) | entrée du document (unicité, puis clé primaire) | non : ne demande ni facture ni `companies` |
  | acceptation de la 30-1 | à relever au Dev Agent Record après le merge de la 30-1 | à établir |

  Le test devient « B aboutit après le `COMMIT` de A, sans interblocage ni `409` » ; M-c11 se redéfinit sur l'ordre
  d'événements ; M-c7 reste équivalente. La bascule et son motif s'écrivent au Dev Agent Record et au registre.
- deux vers la même facture, appelés `attach_source_document_once` : un succès, un `409 SOURCE_DOCUMENT_ALREADY_ATTACHED` ;
- rattachement contre complétion du même document : un seul gagne.

**AC20 — Le sélecteur.** « Joindre un document » ouvre la liste des documents à traiter (nom, date, badge QR, montant
QR à côté du **total de la facture**), avec aperçu du document survolé ou choisi ; *Joindre* demande confirmation.
Vide : « Aucun document à traiter — déposez-le dans le dossier d'import puis importez-le. »

**AC21 — L'écriture montre le document.** `JournalEntryDetailResponse` gagne `sourceDocument: { supplierInvoiceId,
supplierInvoiceNumber, originalFilename } | null`, renseigné quand l'écriture est l'**achat** ou le **règlement**
d'une facture fournisseur qui a un document (C-30-2-15), par une lecture ordinaire faite **dans la transaction de lecture
de `get_journal_entry`** (`routes/journal_entries.rs:479`, l'en-tête restant lu hors transaction par l'écart assumé
existant), jamais sur `&state.pool` ; le lecteur neuf est **ajouté à la garde lexicale**
`get_journal_entry_reads_in_one_transaction` (l. 983-1036) — dont le motif cherche `journal_entries::{lecteur}(`
(l. 1007-1013) : un lecteur d'un autre module ne s'y inscrit pas en ajoutant un nom, le motif est **paramétré par
module** (couples `(module, lecteur)`) — garde qui sinon resterait verte sur
une lecture faite au pool ; `null` veut dire « aucun document », le
détail étant le seul constructeur de cette réponse — le dev le vérifie (`grep -n "JournalEntryDetailResponse {"`) et
l'écrit au Dev Agent Record. La page `journal-entries/[id]` affiche « Facture fournisseur
<numéro> » (lien vers la fiche) et « Voir le document » (aperçu).

**AC22 — Lecture ouverte à tout rôle authentifié.** `GET /supplier-invoices/{id}/source-document` passe parmi les
lectures (`lib.rs`, bloc l. 868-875) ; `GET /imported-supplier-invoices/{id}/source-document` reste Comptable+
(C-30-2-14). Test : le rôle Consultation lit le document d'une facture, reçoit 403 sur celui d'un document à traiter.

**AC23 — E2E de parcours, captures, documentation de clôture.** Voir § *E2E de parcours* ; captures clair et sombre ;
manuel FR (joindre après coup, détacher, consultation depuis l'écriture, renvoi #503 ; le message de
`CURRENCY_NOT_SUPPORTED` complété : « … ou créez la facture sans document puis joignez-le »), PDF régénéré et contrôlé
aplati ; `api-external.md` (routes neuves `POST` et `DELETE …/source-document`, `sourceDocument` des deux réponses et
ses trois états, rôle de la lecture, `SOURCE_DOCUMENT_ALREADY_ATTACHED` ; les deux routes ajoutées au paragraphe des
routes rejouées, l. 788) ; CHANGELOG ; `README.md:44` (ligne existante : « Voir la facture d'origine » devient « Voir le
document », jointure après coup — en gardant « import lancé depuis l'écran », G5).

**AC24 — Détacher le document** (décision de l'orchestrateur, C-30-2-13). Sur la fiche d'une facture fournisseur qui
a un document, « Détacher le document » (Comptable+, confirmation : « Détacher ce document ? Il reviendra dans les
documents à traiter ; la facture et son écriture ne changent pas. ») appelle
`DELETE /api/v1/supplier-invoices/{id}/source-document` (Comptable+, clés `read-write` comme le rattachement), en une
transaction rejouée sur interblocage :
1. (transaction dans `detach_source_document_once`, enveloppée par `retry_app_on_deadlock`)
   `SELECT … FROM imported_supplier_invoices WHERE supplier_invoice_id = ? AND company_id = ? AND status =
   'completed' FOR UPDATE` → `404 SOURCE_DOCUMENT_NOT_FOUND` (code existant) si aucune ligne — facture inconnue,
   d'une autre société ou sans document ;
2. `UPDATE … SET status = 'to_complete', supplier_invoice_id = NULL WHERE id = ? AND company_id = ? AND status =
   'completed'` (fonction `detach_in_tx` du dépôt, garde de statut dans le `WHERE`, 0 ligne = erreur interne) ;
3. audit `imported_supplier_invoice.detached` (`supplier_invoice_id`, `original_filename`).
Tout statut de facture accepté (ouverte, payée, annulée) — comme le rattachement ; la facture, ses lignes, son écriture
d'achat et de règlement **ne sont pas touchées** ; aucun fichier n'est supprimé ; le document réapparaît dans la liste
avec ses données QR. Réponse : `200` et la `SupplierInvoiceResponse` relue (comme AC19) ; la fiche affiche alors
« Joindre un document ». **Effet vérifié par test** : après un détachement, joindre **un autre** document à la même
facture réussit (sans quoi l'unicité tiendrait encore l'ancien lien sur une facture qui n'affiche rien — M-c9).
**Vérifié au code (`15686d55`)** : rien ne lie `status` et `supplier_invoice_id` en base (seuls `chk_imported_si_status`
et la clé étrangère `ON DELETE SET NULL`, `20260629000001_imported_supplier_invoices.sql:64-67`) ; la réactivation
écrit déjà `supplier_invoice_id = NULL` (`reactivate_to_complete`, `repositories/imported_supplier_invoices.rs:247-258`) ;
l'unicité de l'AC4 admet plusieurs `NULL` ; `find_by_supplier_invoice_id_scoped` (l. 157-171) ne retrouve plus la ligne
détachée (son lien est `NULL` ; la 30-2c y ajoute `AND status = 'completed'`, AC18) ; aucun statut de facture n'intervient dans le lien.
**Verrous** : la transaction ne verrouille **que** la ligne du document, jamais la facture (sauf, brièvement, le verrou
d'intervalle que pose la lecture verrouillante sur une clé **absente** — chemin `404`, annulé aussitôt). Avec le rattachement
(staging neuf, puis facture, puis entrée de l'index d'unicité à l'`UPDATE` — AC19), aucun cycle : le détachement ne
demande rien de ce que tient le rattachement ; le contrôle d'unicité de l'`UPDATE` du rattachement attend l'ancienne
ligne que tient le détachement, puis, celui-ci validé (lien remis à `NULL`), aboutit. **Test qui le prouve, avec mesure
de l'attente** (sans rejeu) : une transaction D détache S_old de X par `detach_in_tx` **sans valider** ;
`attach_source_document_once` de S_new à X est lancé dans une tâche ; la sonde existante
`attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:561`, motifs `UPDATE imported_supplier_invoices` et
`supplier_invoice_id`) voit l'`UPDATE` du rattachement **en cours** dans `PROCESSLIST`, puis la tâche est constatée **non
finie** — l'`UPDATE` ne peut être retenu que par le verrou que tient D ; en complément, `sonde_verrou_nowait` (l. 611)
constate depuis une autre connexion que la ligne de S_old est bien verrouillée par D ; D valide ; la tâche aboutit sous
5 s, **en succès**, sans interblocage ni `409`. ⛔ Ni `information_schema.INNODB_TRX` ni `INNODB_LOCK_WAITS` : ils
exigent le privilège `PROCESS`, que l'utilisateur `kesh` n'a pas (`scripts/mariadb-init/01-dev-grants.sql` ; piège déjà
rencontré, `crates/kesh-api/tests/reconciliation_e2e.rs:8139-8141`). La remédiation P3 se fiait à un délai fixe de 300 ms,
qui ne distinguait pas une attente de verrou d'une tâche simplement lente (P4, F3 = R5). Tranché : ce comportement est attendu (l'`UPDATE` du rattachement attend le verrou de l'entrée tenue par D, puis
la trouve marquée supprimée). La mutation **M-c9** (le détachement laisse le lien) le fait rougir (`409` au lieu du
succès) ; le volet « attend » est une propriété du moteur, mesurée, sans mutation (déclaré). Si le test rougit sur le code
non muté (interblocage), arrêt et rapport à l'orchestrateur — l'affirmation ne s'écrit pas au manuel. Un détachement qui verrouillerait la facture **après** le document formerait un cycle avec AC19 :
interdit (M-c10). **Test qui le tient** (déterministe, sans rejeu) : une troisième connexion verrouille la facture
(`SELECT … FOR UPDATE`) et garde le verrou ; `detach_source_document_once` doit **aboutir** pendant ce temps (borne
`tokio::time::timeout` de 5 s) — un détachement qui demanderait la facture attendrait et dépasserait la borne.
**Angle mort assumé** : un document détaché d'une facture **avec QR** peut être « Créer une facture » une seconde fois
— une seconde facture pour la même pièce, la première restant. Geste humain explicite ; le manuel le dit (annuler la
facture fausse d'abord).

**AC25 — La date de règlement s'affiche formatée** (ajout de l'orchestrateur, constat de la 30-1). La fiche d'une facture
fournisseur payée affiche aujourd'hui `paidAt` brut (« Facture réglée. 2026-06-20T00:00:00 »,
`routes/(app)/supplier-invoices/[id]/+page.svelte:393`, `{invoice.paidAt ?? ''}`) ; avec la 30-1, `paid_at` devient la
date du paiement à minuit. L'écran affiche la **date** au format suisse existant, `formatSwissDate`
(`frontend/src/lib/features/reports/reports.api.ts:193`, `AAAA-MM-JJ` → `JJ.MM.AAAA`), appliqué aux dix premiers
caractères — **deux clés**, quatre locales : `supplier-invoices-paid-info` (« Facture réglée. ») sans date, et
`supplier-invoices-paid-info-on` (« Facture réglée le { $date }. ») avec date ; une seule clé à variable rendrait
« Facture réglée le . » sans date (P4, R7). Vitest
de la page (patron `supplier-settlement-page.test.ts`) : `paidAt = "2026-06-20T00:00:00"` rend `20.06.2026` et jamais la
chaîne brute ; `paidAt` absent : la phrase sans date. Mutation **M-c12** (`paidAt` affiché brut) tuée par ce Vitest.

## Tasks / Subtasks

### 30-2a

- [ ] **T-a0** Rebase sur `origin/main` après le merge de la 30-1 ; relire la dernière migration et la version des crates.
- [x] **T-a1** Tests rouges d'abord (AC1-AC3, AC7) dans `crates/kesh-api/tests/inbox_import_e2e.rs` et les tests du
  staging : PNG sans QR, PDF sans QR (accepté `none` **ou** `unreadable` selon pdfium : le test n'exige que
  l'acceptation et l'absence de données QR), QR invalide, champ trop long (nom accentué 70 / 71 caractères, pays de
  trois lettres), montant hors plage (16 chiffres entiers, négatif, `20.005` ; `20.000` accepté avec ses données) —
  fabriqués par le helper « payload brut » (AC2) —, PNG à QR suffixé après `IEND` toujours décodé (prérequis des copies
  E2E), PNG corrompu (`UNSUPPORTED_FILE_TYPE`, AC3), doublon sans QR, réactivation sans QR
  (rapport `qr`), complétion sans QR, liste qui rend des `null`, **retrait d'un document sans QR dont l'audit porte
  `iban_present: false`** (le test existant, `inbox_import_e2e.rs:1124`, ne vérifie que `is_some()`). **Deux tests existants changent de sens** et se réécrivent :
  `import_rejects_unsupported_type_and_no_qr` (`inbox_import_e2e.rs:419`, qui attend `NO_QR_CODE_FOUND` : le PNG sans
  QR y devient accepté) et `import_field_too_long_returns_failed_not_500` (l. 616, qui attend `FIELD_TOO_LONG` : le
  champ QR trop long devient accepté sans QR). L'assertion `archived.exists()` de ce second test (l. 647, garantie de la
  Story 25-6-b : un échec ne supprime pas un fichier archivé partagé) **disparaît avec son sujet** : plus aucun échec
  testable n'emprunte le chemin d'après l'archivage (le nom trop long est inatteignable, AC2 ; les filets
  `DataLengthOrRange` et `CheckConstraintViolation` ne sont plus atteints que par un défaut du contrôle). La garantie
  reste portée par le code des branches d'échec post-archivage (`inbox_import.rs:550-570`, aucune ne supprime le
  fichier) et par la course `UniqueConstraintViolation`, sans test — angle mort écrit au § *Angles morts*.
- [x] **T-a2** Migration, squash, audit P5, P6, P7, bump `min_required` et crates (AC4).
- [x] **T-a3** Entité et dépôt en `Option`, `without_qr`, contrôle des longueurs, import (AC1-AC3), complétion et retrait (AC7), export CSV.
- [x] **T-a4** CSP et son test (AC6) ; relevé dans un vrai navigateur (AC6-bis), consigné au Dev Agent Record.
- [x] **T-a5** `DocumentPreview.svelte` + Vitest (image, PDF, autre type, 410, erreur, révocation) ; liste, badge, rapport, retrait (AC5) ;
  gardes frontend touchées (§ *Gardes existantes*).
- [x] **T-a6** Textes et i18n (AC8) ; K5.
- [x] **T-a7** E2E (§ E2E, E1-a, E2-a, E3) ; module `helpers/inbox.ts` ; `inbox-import.spec.ts` adapté (titre, libellés, plus de `test.skip`) ; `docs/testing.md` mis à jour (répertoires lus par le spec, défauts, échec bruyant ; bloc du `CLAUDE.md` inchangé, constaté) ; captures clair et sombre de la liste et de l'aperçu.
- [x] **T-a8** Manuels utilisateur et administrateur, PDF, `README.md:44`, `api-external.md` (section créée), CHANGELOG et son bandeau (AC9, AC9-bis).
- [x] **T-a9** Mutations M-a1 à M-a15 sauf M-a13, retirée en P3 (sous le verrou de gate ; M-a14 déclarée équivalente ; M-a5 portée sur migration et squash) ; gate complet (`kesh-db` touché : gate complet même en boucle) ; E2E complet au dernier commit de code.

### 30-2b

- [ ] **T-b1** Tests rouges : complétion avec corps de création (projet, IBAN sans QR), `QR_FIELDS_FROM_DOCUMENT`,
  client sans les nouveaux champs, `INVALID_PAYMENT_DETAILS` sur les deux routes (IBAN invalide, QR-IBAN en
  `creditorIban`, IBAN en `creditorQrIban`, les deux ensemble, QR-IBAN sans QRR, QRR invalide, référence de 65
  caractères — les sept motifs d'AC12-bis), référence de 27 chiffres avec un IBAN ordinaire **acceptée**, IBAN à
  espaces insécables accepté, IBAN allemand accepté, valeur enregistrée normalisée, réponse de `complete` qui porte `sourceDocument` (en 30-2c).
- [ ] **T-b2** `CreateSupplierInvoiceRequest` partagé, conversion commune et validation par `kesh_core::types::{Iban, QrIban}` et `validate_qrr` (AC12, AC12-bis, AC13) ; IBAN et références factices des tests et du seed remplacés par des valeurs synthétiques valides ; route `GET /imported-supplier-invoices/{id}/supplier-candidates` et son test Rust (AC14), registre des routes.
- [ ] **T-b3** `SupplierInvoiceForm.svelte` extrait ; Vitest du composant (champs verrouillés avec QR, total, erreurs).
- [ ] **T-b4** Page `/supplier-invoices/new` (AC10 ; Vitest des états « document inconnu, d'une autre société ou plus à
  traiter », « chargement », « erreur de lecture ») ; liste et page d'import reliées ; formulaires en ligne retirés et
  leurs clés i18n orphelines retirées (AC8) ; `supplier-invoice-scan.spec.ts`, `supplier-invoices.spec.ts`,
  `inbox-import.spec.ts` (complétion en ligne) et `boutons-rouges-inventory.test.ts` adaptés (§ *Gardes existantes*).
- [ ] **T-b5** `supplierNameKey`, `searchTermsFor` et `proposeSuppliers` (pures, Vitest des cas d'AC14), extraction de `qrIbanHasValidQrIid` dans `lib/shared/utils/iban.ts` (avec CH/LI, Vitest) employée par `bank-accounts` et le formulaire, panneau de création (AC12, AC14, AC15).
- [ ] **T-b6** i18n, manuel, PDF, `api-external.md`, CHANGELOG (AC17).
- [ ] **T-b7** E2E E1, E2, E4 ; captures clair et sombre de la page côte à côte et du panneau fournisseur ; mutations M-b1 à M-b13 (M-b2 déclarée équivalente) ; gates.

### 30-2c

- [ ] **T-c1** Tests rouges : rattachement (succès, 404 ×2, `409 IMPORT_NOT_PENDING_COMPLETION`,
  `409 SOURCE_DOCUMENT_ALREADY_ATTACHED`, autre société des deux côtés, facture payée) ; concurrence, en appelant les
  fonctions `_once` **sans rejeu** (AC19) : deux rattachements vers **deux factures différentes** (deux succès, aucun
  interblocage), deux vers la même facture (un succès, un `409`), rattachement contre complétion du même document (un
  seul gagne) ; `sourceDocument` des deux réponses et ses trois états ; RBAC de la lecture ; détachement (succès sur
  facture ouverte et payée, 404 sans document ou autre société, document revenu `to_complete` et listé, facture et
  écritures inchangées, audit) ; détachement pendant qu'une autre connexion tient la facture (AC24, il aboutit) ;
  après détachement, joindre un autre document à la même facture (M-c9) ; rattachement pendant un détachement non
  validé (AC24, il attend puis aboutit) ; rattachement déterministe sous verrou tenu (AC19, M-c11) ; détail de
  l'écriture de **règlement** d'une facture à document (M-c5) ; réponses `200` des deux routes avec `sourceDocument`.
- [ ] **T-c2** Dépôt `kesh-db` : `attach_in_tx` et `detach_in_tx` (exposés sans `COMMIT`), lecture du document d'une facture et d'une écriture ; `find_by_supplier_invoice_id_scoped` passe d'un `&MySqlPool` (l. 158) à un exécuteur générique, pour lire dans l'instantané de `get_settlement_view` (AC18), et gagne `AND status = 'completed'`.
- [ ] **T-c3** Routes (rattacher, détacher), réponses, déplacement de la lecture (AC18, AC19, AC21, AC22, AC24) ; registre des routes mutantes
  (`audit_route_registry.rs`, compteurs recomptés), registre des libellés d'audit (`crates/kesh-api/tests/audit_label_registry.rs`) pour
  les actions neuves `imported_supplier_invoice.attached` et `.detached`.
- [ ] **T-c4** Fiche de la facture (dont « Détacher le document » et sa confirmation), sélecteur (Vitest : montant QR à
  côté du total, liste vide), page de l'écriture
  (AC18, AC20, AC21, AC24, AC25) ; `supplier-settlement-page.test.ts:66` (mock du module `imported-supplier-invoices.api`) adapté.
- [ ] **T-c5** i18n, manuel, PDF, `api-external.md`, CHANGELOG, README (AC23).
- [ ] **T-c6** E2E de parcours complets (E1 à E7) ; captures ; mutations M-c1 à M-c12 (M-c7 déclarée équivalente ; M-c2 portée sur migration et squash) ; gate complet et E2E complet au dernier commit de code.

## Dev Notes

### Tests (récapitulatif, rouges d'abord)

- **Rust** : `inbox_import_e2e.rs` (AC1-AC3), tests des routes du staging (AC7, AC11-AC13, AC19, AC22), tests du
  dépôt (`imported_supplier_invoices`, AC4 contrainte « tout ou rien » et unicité directement en SQL), `csp.rs`
  (AC6), `csv_tables.rs` (cellule vide), réponses `SupplierInvoiceResponse` et `JournalEntryDetailResponse`.
- **Vitest** : `DocumentPreview`, `SupplierInvoiceForm`, propositions de fournisseur, panneau de création, pages
  (liste, `new`, fiche, écriture) — patron des `*-page.test.ts` existants.
- **Fixtures synthétiques** (§ 8 des consignes : aucune pièce réelle) : `frontend/tests/e2e/fixtures/` porte déjà
  `spc_e2e_invoice.png` et `spc_e2e_discard.png` (versionnés, à QR) ; le PNG **sans** QR et le PDF **sans** QR sont
  **générés par le code de test**, sans fichier binaire ni interpréteur : en TypeScript, des constantes d'octets dans
  `frontend/tests/e2e/helpers/inbox.ts` (un PNG minimal valide de quelques pixels, un PDF minimal d'une page de texte
  écrit à la main, `%PDF-1.4` … `%%EOF`) ; en Rust, le helper existant **`blank_png()`**
  (`crates/kesh-api/tests/inbox_import_e2e.rs:309`) pour le PNG sans QR, et la même constante PDF. Le PNG corrompu se fabrique en tronquant le PNG minimal.
- **Test SQL de la contrainte « tout ou rien »** : **sept** `INSERT`, chacun laissant **une seule** des sept colonnes à
  `NULL` (les six autres renseignées), tous refusés — sans quoi M-a5 (contrainte amputée d'une colonne) survit.

### Gardes existantes qui changent de sens ou rougiront (à adapter dans la sous-story indiquée)

| Garde | Ce qui change | Sous-story |
|---|---|---|
| `crates/kesh-api/tests/inbox_import_e2e.rs:419` et `:616` | PNG sans QR et champ QR trop long acceptés (T-a1) | a |
| `frontend/tests/e2e/inbox-import.spec.ts` | `h1` `/Importer/i` (l. 118) devient « Documents à traiter » ; la complétion en ligne (`imported-complete-open`, `-submit`, `imported-supplier-select`) disparaît ; le `confirm()` et `imported-discard` suivent le libellé neuf. Ses deux `test.skip(!INBOX_DIR…)` (l. 127, 171) deviennent des échecs bruyants comme ceux du spec neuf | a (titre, retrait), b (complétion) |
| `boutons-rouges-inventory.test.ts` | entrées `supplier-invoices/+page.svelte@button:1` et `supplier-invoices/import/+page.svelte@button:1` (l. 245, 249) déplacées vers `SupplierInvoiceForm.svelte` ; `toHaveLength(14)` (l. 323) recompté | b |
| `textes_coherents.rs` G5 (l. 584-614) | phrases « L'import ne se déclenche pas tout seul » (manuel) et « import lancé depuis l'écran » (README) conservées mot pour mot | a, c |
| `supplier-invoice-scan.spec.ts:41-45` | le formulaire est sur `/supplier-invoices/new`, non plus sur la liste | b |
| `supplier-invoices.spec.ts` | même déplacement ; IBAN factices éventuels → IBAN valides (AC12-bis) | b |
| `supplier-settlement-page.test.ts:66` | mock du module `imported-supplier-invoices.api` (lecture du document) | c |
| `FiscalYearChecks.test.ts:151-160` | lien K5 vers `/supplier-invoices/import` : **conservé** (textes seuls changent) | a |
| `audit_route_registry.rs`, `audit_label_registry.rs` | deux routes mutantes et deux actions d'audit neuves ; compteurs recomptés | c |
| `i18n-keys.test.ts`, `lint-i18n-ownership`, parité inter-locales | clés retirées et ajoutées | a, b, c |
| `csv_tables.rs` | cellule vide pour un champ `None` | a |
| tests de la CSP | aucun aujourd'hui ; créé (AC6) | a |
| `frontend/tests/e2e/payment-batches.spec.ts:107-108` | IBAN ordinaire et référence libre « Facture E2E » : doivent **rester acceptés** par AC12-bis | b |
| `get_journal_entry_reads_in_one_transaction` (`routes/journal_entries.rs:983-1036`) | lecteur neuf du document à ajouter à sa liste (l. 1007) | c |
| `closing-checks-labels.ts:63` (repli du libellé K5) | texte K5 neuf | a |

### E2E de parcours (Guy ne testera qu'en fin d'epic : ils portent la recette)

Un nouveau fichier `frontend/tests/e2e/supplier-documents.spec.ts`, **sans `test.skip`** : le spec porte la recette, il
ne doit pas pouvoir se taire. Il lit les deux répertoires par `process.env.KESH_INBOX_DIR ?? '/tmp/kesh-e2e/inbox'` et
`process.env.KESH_DOCUMENTS_DIR ?? '/tmp/kesh-e2e/documents'` — les valeurs par défaut sont **celles de la recette
documentée** (`docs/testing.md`, terminal 1, l. 168-170 ; même recette au `CLAUDE.md`), si bien que la recette suivie à
la lettre suffit, sans variable côté runner. Son `beforeAll` appelle `seedTestState` (patron d'`inbox-import.spec.ts:37`)
puis **échoue** (`throw`, message qui nomme le répertoire) si l'un des deux n'existe pas ou n'est pas inscriptible, et
fait une **poignée de main** avec le backend : il dépose un fichier sonde, lance l'import, et échoue avec le message « le
runner et le backend n'utilisent pas le même KESH_INBOX_DIR » s'il n'est pas dans `accepted[]` (un backend de worktree
peut avoir ses propres répertoires, `docs/agents-orchestration.md` § 4). Les assertions portent sur le **nom** du fichier
déposé, jamais sur un nombre de lignes (un résidu d'un run interrompu est réimporté au suivant). Un
module d'aide commun (`frontend/tests/e2e/helpers/inbox.ts`) porte ces lectures et le dépôt, et `inbox-import.spec.ts` y
passe aussi (ses deux `test.skip`, l. 127 et 171, disparaissent). **T-a7 met à jour `docs/testing.md`** (le spec lit ces
répertoires, ses défauts, l'échec bruyant) ; le bloc du `CLAUDE.md` **ne change pas**, la recette restant la même —
à constater et écrire au Dev Agent Record.

**Fixtures propres à chaque parcours.** Le staging est unique par `(company_id, file_hash)` et un document `completed`
compte comme doublon (`inbox_import.rs:476-479`) : deux parcours **du même spec** qui déposent les mêmes octets
s'empêchent l'un l'autre (entre deux specs, `seedTestState` vide la table, `backup.rs:56`). Chaque parcours dépose
donc **sa propre copie**, rendue unique par un suffixe aléatoire ajouté **après la fin logique** du fichier (après le
bloc `IEND` d'un PNG, après `%%EOF` d'un PDF, sous forme de commentaire), ce qu'ignorent les lecteurs — à vérifier une
fois sur le PNG à QR — c'est un test Rust de T-a1, préalable. Attribution : E1-a et E1 : une copie de
`spc_e2e_invoice.png` **chacun** ; E2-a et E2 : une copie du PNG sans QR chacun ; E3 : une copie du PDF sans QR (le test
n'asserte pas le badge d'un PDF : `none` ou `unreadable` selon pdfium, les deux acceptés) ; E5 : une copie du PNG sans
QR ; **E7 : une seconde copie du PNG sans QR**, déposée et importée dans le test, pour joindre un **autre** document
après le détachement ; E6 : une copie de `spc_e2e_discard.png` (QR distinct de celui d'E1). Le fournisseur proposé d'E1 est créé
par l'API au début du test, au **nom exact** du créancier de la fixture. E1 et E6 emploient des fixtures à QR
**distinctes** — E6 une copie de `spc_e2e_discard.png` — pour qu'une facture d'E6 au même IBAN n'ajoute pas un
candidat « même IBAN » à E1 ; et E1 clique *Retenir* sur la proposition **désignée par son nom**, jamais sur la
première.

**Indépendance.** Chaque parcours prépare son état **par l'API** (import, complétion, création de facture) et se rejoue
seul — E6 prépare sa propre facture à document au lieu de lire celle d'E1. Seul le bloc E4 → E5 → E7 (une même facture,
joindre puis détacher) est un `test.describe.serial`, déclaré avec son motif (patron `open-items.spec.ts:181`) ;
E4 y est la préparation, rejouée par la 30-2c.

| # | Parcours | Sous-story |
|---|---|---|
| E1-a | document **avec** QR déposé → import → badge QR → aperçu visible | a |
| E2-a | document **sans** QR (PNG) déposé → import → badge « Sans QR-code », aperçu `<img>` ; le fichier n'est plus dans le dossier | a |
| E3 | document qui n'est pas une facture → *Retirer de la liste* → disparaît ; le redéposer le fait revenir | a |
| E1 | document avec QR → *Créer une facture* → aperçu présent pendant la saisie, IBAN/référence/montant verrouillés, fournisseur **proposé** (contact seedé au nom du créancier) → *Retenir* → lignes égales au montant → enregistrer → la fiche montre le document, la liste ne l'a plus | b |
| E2 | document sans QR → *Créer une facture* → l'aperçu reste visible pendant toute la saisie (assertion avant **et** après l'ouverture du panneau et l'ajout de lignes) → IBAN et référence saisis à la main (IBAN valide synthétique) → *Nouveau fournisseur* dans le panneau → créé et sélectionné, saisie intacte → enregistrer → la fiche montre le document, et l'IBAN saisi | b |
| E4 | facture **sans** document depuis *Enregistrer une facture* → la fiche propose « Joindre un document » | b (vérifié en c) |
| E5 | sur la facture de E4 → *Joindre un document* → choisir un document à traiter → la fiche le montre, la liste ne l'a plus | c |
| E6 | facture à document préparée par l'API (copie propre) → depuis son écriture d'achat → lien vers la facture et « Voir le document » | c |
| E7 | sur la facture de E5 → *Détacher le document* → la fiche propose de nouveau « Joindre un document », le document revient dans « Documents à traiter », l'écriture d'achat ne montre plus de document ; joindre un **autre** document à cette facture réussit | c |

⚠️ **Le PDF dans une `<iframe>` ne se rend pas en Chromium sans tête** (pas de visionneuse) : l'E2E d'un PDF vérifie
l'`<iframe>`, son `src` en `blob:` et le type du blob, non le rendu ; les **captures** se font sur un PNG. Le montage
E2E est celui de `docs/testing.md` à la lettre (port 3052 si le worktree en a besoin). Une **régression** se juge contre la liste des échecs attendus de `docs/testing.md`.

### Mutations (motif unique vérifié, restauration `git checkout -- f && touch f`, sous le verrou de gate)

Une mutation à **deux fichiers** (M-a5, M-c2 : migration et squash) vérifie un motif unique **par fichier**, se restaure
par `git checkout -- <migration> <squash> && touch <migration> <squash>`, et ne s'accompagne **jamais** d'un backend ni
d'un `sqlx migrate run` sur une base persistante (`kesh_302`, `kesh_e2e_302`) : la somme de contrôle de la migration
mutée s'y enregistrerait (P8). Les tests de comportement montent le squash sur des bases éphémères.

| # | Mutation | Tuée par |
|---|---|---|
| M-a1 | `Ok(None)` revient à `dispose_failed(… ERR_NO_QR_CODE_FOUND)` | AC1 (PNG sans QR) |
| M-a2 | `INVALID_SPC_PAYLOAD` revient en échec | AC2 |
| M-a3 | `ImageDecode` accepté sans QR | AC3 |
| M-a4 | contrôle de longueur retiré (le champ trop long retombe dans le filet `DataLengthOrRange`, donc en `failed`) | AC2 : le test exige l'**acceptation** sans QR (`unreadable`) |
| M-a5 | contrainte « tout ou rien » amputée d'une colonne — dans la **migration et le squash** (`crates/kesh-db/test-schema/0001_schema_squash.sql`) à l'identique | test SQL d'AC4 (monté sur le squash). La même mutation portée sur la **migration seule** est tuée par `test_schema_guard` (dérive migration ↔ squash), non par un test de comportement — déclaré |
| M-a6 | `iban_present` calculé `true` sans IBAN | T-a1 : retrait d'un document sans QR, audit `iban_present: false` |
| M-a7 | contrôle de devise appliqué sans QR (`currency.as_deref() != Some("CHF")`) | AC7 |
| M-a8 | CSP sans `blob:` dans `frame-src` (ou dans `object-src`) | test `csp.rs` |
| M-a9 | `DocumentPreview` ne révoque pas l'URL | Vitest |
| M-a10 | export CSV écrit `None`/`null` littéral | test `csv_tables.rs` |
| M-a14 | l'un des deux filets de l'import (`DataLengthOrRange`, `CheckConstraintViolation`) retiré | **équivalente, déclarée** : aucun n'est atteignable une fois le contrôle pur juste (AC2) — elle couvre les deux filets |
| M-a11 | le contrôle pur ne borne plus le montant (plage ou décimales) | T-a1 : le montant à 16 chiffres entiers (volet plage) **et** `20.005` (volet décimales) acceptés **sans** données, `qrError: QR_AMOUNT_OUT_OF_RANGE` — sous la mutation, le premier tombe en `failed[]` (`1264`), le second entre avec ses données |
| M-a12 | le contrôle de longueur compte en octets (`len()`) | AC2 (nom accentué de 70 caractères accepté avec ses données, nom stocké égal) |
| M-a15 | `iban_present` calculé `true` à la création d'un document sans QR | T-a1 : audit `imported_supplier_invoice.created` d'un document sans QR, `iban_present: false` |
| M-b1 | un corps avec IBAN est accepté pour un document avec QR | AC11 |
| M-b2 | le corps l'emporte sur le QR (IBAN du corps enregistré) | **équivalente isolément, déclarée** : la garde de M-b1 refuse tout corps qui porte ces champs ; seule la paire M-b1 + M-b2 est observable, et M-b1 la tue |
| M-b3 | `project_id` ignoré à la complétion | AC13 |
| M-b4 | le premier fournisseur proposé est présélectionné | Vitest AC14 + E1 |
| M-b5 | propositions sans filtre `isSupplier` (appel de `listContacts`) | Vitest de la **page** `new` qui observe l'argument passé à `listContacts` (une fonction pure ne le voit pas) |
| M-b6 | le panneau vide les lignes à la création du fournisseur | Vitest AC15 + E2 |
| M-b7 | contrôle `AMOUNT_MISMATCH` retiré | test existant de la complétion |
| M-b8 | le scan navigateur proposé avec un document à QR | Vitest AC11 |
| M-b9 | la fonction partagée `qrIbanHasValidQrIid` toujours fausse (QR-IBAN routé en `creditorIban`) | Vitest du module partagé + `INVALID_PAYMENT_DETAILS` côté serveur |
| M-b10 | validation de l'AC12-bis retirée de la conversion commune | T-b1 (IBAN invalide accepté) |
| M-b11 | éligibilité de `proposeSuppliers` retirée (classement seul) | Vitest AC14 (« Zurcher Sàrl » contre « Nicolet Sàrl » proposé) |
| M-b12 | `supplierNameKey` sans la règle `ü`→`ue` | Vitest AC14 (« Müller AG » / « Mueller AG ») |
| M-b13 | propositions par IBAN ignorées (nom seul) | Vitest de `proposeSuppliers` (IBAN avant nom) + test Rust de `supplier-candidates` |
| M-c1 | traduction de la violation d'unicité retirée (elle sort en 500) | T-c1 (même facture : `409` attendu) |
| M-c2 | unicité de `supplier_invoice_id` retirée — dans la **migration et le squash** à l'identique | T-c1 (même facture : deux succès au lieu d'un). Portée sur la **migration seule** : tuée par `test_schema_guard`, déclaré |
| M-c3 | rattachement accepté sur un staging `discarded` | AC19 |
| M-c4 | facture d'une autre société acceptée | AC19 (IDOR) |
| M-c5 | `sourceDocument` de l'écriture limité à l'achat | T-c1 : test du détail de l'écriture de **règlement** d'une facture à document (`sourceDocument` présent) |
| M-c6 | la lecture du document reste Comptable+ | AC22 |
| M-c7 | ordre des verrous inversé (facture puis staging) | **équivalente, déclarée** : aucun chemin ne verrouille une facture puis un staging (AC19), l'inversion ne ferme aucun cycle |
| M-c8 | « Voir le document » affiché sans document | Vitest de la fiche |
| M-c9 | le détachement laisse `supplier_invoice_id` renseigné (seul le statut change) | T-c1 et E7 : après détachement, joindre **un autre** document à la même facture réussit (sous la mutation : `409`, l'unicité tenant l'ancien lien) — la lecture partagée filtrant `status = 'completed'` (AC18), « la fiche montre encore le document » ne la tue plus |
| M-c10 | le détachement verrouille la facture après le document | test d'AC24 : une autre connexion tient la facture, le détachement doit aboutir sous 5 s |
| M-c12 | `paidAt` affiché brut sur la fiche (`formatSwissDate` retiré) | Vitest d'AC25 |
| M-c11 | lecture verrouillante du lien existant réintroduite avant l'`UPDATE` du rattachement (`SELECT … WHERE supplier_invoice_id = ? FOR UPDATE`) | test déterministe d'AC19 : A rattache D1 à I1 sans valider, B (D2 → I2) doit aboutir sous 5 s |

### Angles morts assumés (à écrire aux doc-comments et au manuel)

- Détacher un document d'une facture avec QR permet d'en créer une seconde facture (AC24) : la première n'est ni
  annulée ni signalée.
- `.keshbackup` n'emporte pas les documents (#503) ; un document restauré sans son fichier répond 410, l'aperçu le dit.
- Un QR que rien ne décode (image floue) laisse un document « sans QR-code » : la saisie se fait à la main.
- Un PDF corrompu à octets magiques valides rend `PdfRender` et **entre** (`unreadable`), alors qu'une image corrompue
  reste refusée (C-30-2-4) : l'asymétrie est inévitable (pdfium absent rend la même erreur) ; l'aperçu d'un tel PDF
  reste vide, et le document se retire de la liste.
- « Muller » (sans tréma ni `e`) et « Müller » ne se rapprochent pas par la clé (AC14) ; les mots de moins de trois
  caractères ne sont pas cherchés par le serveur.
- Un IBAN valide d'un pays absent de la table de Kesh (`iban.rs:13-101`) est refusé à la saisie (AC12-bis) ; un IBAN
  étranger accepté reste refusé par le lot pain.001 (comportement existant).
- La recherche par IBAN de `supplier-candidates` ne retire en SQL que l'espace, U+00A0 et U+202F (AC14).
- Le montant attendu saisi à la main (`expectedPaymentAmount`, sans QR) n'est validé ni en signe, ni en échelle, ni
  contre le total (comportement existant de `create_in_tx`).
- Tout PDF ou image déposé est désormais archivé sous `KESH_DOCUMENTS_DIR` (jusqu'à `KESH_INBOX_MAX_FILE_BYTES` par
  fichier) et compte dans K5 jusqu'à son retrait : coût disque et bruit de K5 assumés, dits au manuel.
- La garantie « un échec ne supprime pas un fichier archivé partagé » n'a plus de test qui la sollicite (T-a1).
- Les propositions de fournisseur héritent de la limite du socle 22-2a (classement d'une fenêtre de 20, sans tri de
  pertinence côté serveur, #315) : **l'absence de proposition ne prouve pas l'absence du fournisseur, et sa présence
  ne prouve pas que c'est le bon** (deux sociétés d'un groupe aux noms presque identiques restent éligibles).
- Inbox unique par instance (L8, #199), inchangé.
- *(Ajouté le 2026-10-15, revue de code P1 de la 30-2a, B1 = E4.)* La **date d'import** affichée est la date **UTC**
  de `created_at` (connexion en `time_zone = '+00:00'`, `crates/kesh-db/src/pool.rs`) : un document importé entre
  minuit et 1 h (hiver) ou 2 h (été) heure suisse s'affiche « Importé le » la veille. Aucun formatage UTC → local
  n'existe côté écran (le journal d'activité lit l'horodatage naïf comme local, autre forme du même écart) ; non
  corrigé ici.
- *(Ajouté le 2026-10-15, revue de code P1 de la 30-2a, B3 = E6.)* Une image que la crate `image` ne décode pas est
  refusée en `UNSUPPORTED_FILE_TYPE` même si un navigateur l'afficherait — JPEG CMJN ou à codage arithmétique, image
  de plus de `MAX_IMAGE_DIMENSION` (10 000) pixels de côté ; le manuel le dit (§ *Ce qui reste refusé*) et C-30-2-4
  est annoté.

### Pièges connus du dépôt, applicables ici

- P8 : une migration appliquée ne se modifie plus, pas même un commentaire ; une correction de la migration de la
  30-2a après son passage sur `kesh_e2e_302` impose de reconstruire cette base. D'où l'essai préalable sur une base
  fraîche et une base peuplée (AC4).
- Le rejeu sur interblocage masque un cycle : les tests de concurrence appellent les fonctions `_once` (AC19, AC24).
- P2-bis : le bump `min_required` sans le bump des crates fait refuser le démarrage **et** l'import d'une sauvegarde —
  invisible avant le gate complet (`admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install`).
- La CSP n'est posée que sur les réponses `text/html` (`csp.rs:34-55`) : le test porte sur la valeur, pas sur une
  réponse d'API.
- `apiClient.getBlob` et le 401-refresh : ne pas pointer une `<iframe>` sur l'URL de l'API (C-30-2-6).
- Le pool de connexions est à cinq et l'import en tient une pour `GET_LOCK` : ne pas élargir la transaction de
  l'import (commentaire `inbox_import.rs:533-538`).
- Le registre des routes mutantes (`audit_route_registry.rs`) et ses compteurs : recompter depuis la source.
- `kesh-reconciliation` : ne pas y toucher (30-1).

### Project Structure Notes

Backend : `crates/kesh-db/migrations/`, `crates/kesh-db/src/entities/imported_supplier_invoice.rs`,
`crates/kesh-db/src/repositories/imported_supplier_invoices.rs`, `crates/kesh-db/src/post_restore.rs`,
`crates/kesh-db/test-schema/`, `crates/kesh-api/src/{inbox_import.rs, routes/imported_supplier_invoices.rs,
routes/supplier_invoices.rs, routes/journal_entries.rs, middleware/csp.rs, exports/csv_tables.rs, lib.rs}`.
Frontend : `frontend/src/lib/features/{imported-supplier-invoices, supplier-invoices, contacts}/`,
`frontend/src/routes/(app)/supplier-invoices/{+page.svelte, import/, new/, [id]/}`,
`frontend/src/routes/(app)/journal-entries/[id]/`, `frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte`.
Docs : `docs/manual/fr/user-manual.tex` (+ PDF), `docs/api-external.md`, `docs/migrations-idempotence-audit.md`,
`CHANGELOG.md`.

### References

- Issue #685 (conception de Guy) ; #503, #235, #459, #199, #315 ; fichier d'epic § 30-2 ; registres C-30-0-1 à 3 et
  C-30-2-1 à 23.
- `CLAUDE.md` : *Migration breaking policy* (P1-P8), *Un appariement automatique propose*, *Règle de splitting
  préventif*, *Review Iteration Rule*, *Test Locally First*.
- `docs/agents-orchestration.md` ; `docs/testing.md` § *Prérequis Playwright local* et § *Les échecs attendus*.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (Claude Code), autonomie, 2026-10-15 — **30-2a seulement** (la 30-2b et la 30-2c ne sont pas commencées).
Worktree `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`, partie de `5b566378` ; bases `kesh_302a` et
`kesh_e2e_302a`, port 3052, `CARGO_TARGET_DIR` du worktree, dossiers E2E `/tmp/kesh-e2e-302a/`.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/` : `302a-rouges.log` (tests rouges), `302a-gate1.log` (premier gate
backend complet), `302a-mutations.log` (campagne), `302a-e2e-cible.log`, `302a-ac6bis.log` et captures
`302a-ac6bis/*.png` (relevé AC6-bis), `302a-make.log` (PDF), `302a-final-*.log` et `302a-e2e-complet.log` (gate final).

### Completion Notes List

**Commits** : `09c0057b` feat(30-2a) — code, tests, i18n, migration (dernier commit de code) ; `af0d2eb4` docs(30-2a) —
manuels et PDF, CHANGELOG, API, README, `.env.example` ; puis le commit de cette fiche, du registre, de
`sprint-status.yaml` et du prompt de revue.

**T-a0 — non fait, différé (C-30-2-27).** *(Fait le 2026-10-15 après la revue P1 : report sur `fecc8566`, cf.
« Report et remédiation de la revue de code P1 » plus bas.)* Développé sur `story/30-2` en parallèle de la 30-1a, à la demande de
l'orchestrateur. Constat : la 30-1a n'ajoute aucune migration (`20261013000001` était la dernière de `main`, d'où
l'horodatage `20261015000001`), et les dix crates étaient à 0.13.4 : la 30-2a les porte à 0.14.0. **Conflits probables
au rebase sur le merge de la 30-1a** : `CHANGELOG.md` (§ `[0.14.0] — Non publié` — la 30-1a y ajoute ses entrées ; le
bandeau *À faire* de la 30-2a doit rester en tête de section, entrées réunies par union), `sprint-status.yaml` (lignes
`30-1*` et `30-2a`, union), `docs/api-external.md` (sections voisines), `docs/manual/fr/*.tex` et **leurs PDF** (à
régénérer après résolution des `.tex`), `crates/kesh-api/src/errors.rs` (**non touché** par la 30-2a — C-30-2-26 —,
donc pas de conflit attendu), `Cargo.toml` des dix crates **et `Cargo.lock`** si la 30-1a a bumpé une version, les
registres de choix (`epic-30-choix-autonomes*.md`, union), les compteurs i18n (`i18n-keys.test.ts` `sitesTotal`,
`i18n-un-repli-par-cle.test.ts`) si la 30-1a a ajouté des sites `i18nMsg` ou des clés du domaine — **à recompter**, non
à additionner. `crates/kesh-reconciliation/Cargo.toml` n'est touché que pour sa version (P2-bis).

**Prérequis d'environnement constaté** : le MariaDB de dev (conteneur redémarré, script d'init monté depuis
`kesh-chore-tmpfs`) n'avait pas les droits du fichier versionné `scripts/mariadb-init/01-dev-grants.sql` (ni
`\_sqlx\_test%`, ni `kesh%`) : les deux `GRANT` du fichier ont été réappliqués par root (`mariadb < 01-dev-grants.sql`),
sans redémarrage du conteneur.

**Tests rouges d'abord (T-a1)** — relevé `302a-rouges.log`, avant tout code, sur `5b566378` + les tests seuls :
`38 tests run: 22 passed, 16 failed`. Extraits copiés :

```
FAIL kesh-api middleware::csp::tests::csp_value_is_pinned_and_allows_blob_previews
  left: Some("img-src 'self' data:")   right: Some("img-src 'self' data: blob:")
FAIL import_accepts_png_without_qr
  {"accepted":[],"failed":[{"errorCode":"NO_QR_CODE_FOUND","fileName":"ticket.png"}],"warnings":[]}
FAIL import_accepts_pdf_without_qr
  {"accepted":[],"failed":[{"errorCode":"PDF_RENDER_ERROR","fileName":"lettre.pdf"}],"warnings":[]}
FAIL import_unreadable_qr_is_accepted_without_qr_data
  {"accepted":[],"failed":[{"errorCode":"INVALID_IBAN","fileName":"iban.png"},{"errorCode":"INVALID_SPC_PAYLOAD","fileName":"spc.png"}],…}
FAIL import_field_too_long_is_accepted_without_qr_data
  {"accepted":[{"fileName":"n70.png",…}],"failed":[{"errorCode":"FIELD_TOO_LONG","fileName":"pays.png"},{"errorCode":"FIELD_TOO_LONG","fileName":"n71.png"}],…}
FAIL import_amount_out_of_range_is_accepted_without_qr_data
  {"accepted":[ouvert, trois-zeros, decimales, nul, negatif],"failed":[{"errorCode":"FIELD_TOO_LONG","fileName":"seize.png"}],…}
FAIL import_with_qr_reports_present / import_png_with_suffix_after_iend_still_decoded / import_reactivation_with_qr_reports_present
  left: Null  right: "present"
FAIL qr_columns_are_all_or_nothing
  Database(… number: 1048, message: "Column 'creditor_iban' cannot be null")
FAIL supplier_invoice_id_is_unique
  un second document lié à la même facture doit être refusé: MySqlQueryResult { rows_affected: 1, … }
```

Les cinq autres rouges *(« sept » jusqu'au 2026-10-15, corrigé à la revue de code P1, A1 : 11 extraits + 5 = 16, le
total du journal)* (`complete_document_without_qr`, `discard_document_without_qr_traces_iban_absent`,
`import_no_qr_duplicate_and_reactivation`, `import_rejects_unsupported_type_and_accepts_no_qr`,
`list_returns_nulls_for_document_without_qr`) tombent sur le même `NO_QR_CODE_FOUND`. Relevé utile au passage : avant
la story, `20.005`, `0.00` et `-5.00` entraient **avec** leurs données, et un montant de 16 chiffres sortait en
`FIELD_TOO_LONG` (1264 → `DataLengthOrRange`). `import_corrupt_png_is_still_rejected` était déjà vert (AC3 tenu avant).
Les tests Vitest du composant d'aperçu sont neufs avec lui (fichier absent avant) ; le test de la CSP n'existait pas
(`grep -n "mod tests" csp.rs` → 0).

**Migration (T-a2)** — essayée **avant toute base persistante** sur une base fraîche et sur une copie peuplée de
`kesh_302a` (seed + une ligne `completed` liée à une facture, une `to_complete`, une `discarded` ; lignes synthétiques),
deux bases jetables supprimées ensuite : `Applied 20261015000001` les deux fois, lignes et lien conservés,
`uq_imported_supplier_invoice` en place, `idx_imported_supplier_invoice` disparu, `min_required` = `0.14.0`. Puis
appliquée à `kesh_302a`, reconstruite à neuf avant chaque gate. Squash régénéré (`scripts/regen-test-schema.sh` :
82 migrations, 40 tables, `_kesh_version = (1, '0.14.0', '0.1.0')`). MariaDB relit la contrainte sans parenthèses
(`creditor_iban is null = (…)`) : même sémantique, prouvée par `qr_columns_are_all_or_nothing` monté sur le squash.
P5 : ligne d'audit, **82** = 82 lignes = `ls … | wc -l`, partition **10 + 72 + 0** recomptée par `grep -cE`. P6 :
`migrations_upgrade_path.rs` 81/47 → **82/48** (frontière 34) ; les autres sites résolvent par version. Test neuf
`imported_documents_without_qr_preserves_existing_rows` (par version, assertion de montage). P7 : `EXEMPT_MIGRATIONS`
18 → **19** (`Durable`). P8 : checksum inscrit à `migrations.sha384`. Tests de version portés à 0.14.0
(`downgrade_protection_*`, `migrations_kesh_version_initial_row`) ; `downgrade_protection_refuses_0_13_0` devient
`…_refuses_0_13_4`. Dix crates à 0.14.0, `Cargo.lock` régénéré par le build.

**Inventaire clos des lecteurs (AC4)** — `grep -rn "creditor_iban\|is_qr_iban\|creditor_address_type\|creditor_name\|creditor_country\|reference_type\|\.currency" crates/kesh-api/src frontend/src`
rend 22 fichiers ; ceux du staging sont `inbox_import.rs`, `routes/imported_supplier_invoices.rs`,
`exports/csv_tables.rs` (partie `imported_supplier_invoices`), `qr_decode.rs` (doc-comment) et la page d'import ; les
autres (`supplier_invoices.rs`, `bank_imports.rs`, `reconciliation.rs`, `invoice_pdf_service.rs`, `credit_notes.rs`,
`bank_profiles.rs`, écrans de banque et de réconciliation) lisent d'autres tables. Les consommateurs des types du
staging (`grep -rln "ImportedSupplierInvoice\b\|imported-supplier-invoices.types"`) sont tous repris. `closing_checks.rs`
et `exports_global_e2e.rs` insèrent en SQL brut : rien à reprendre, ils compilent et passent. `admin_full_import_e2e.rs`
n'a aucun usage (correction L4 de la fiche, confirmée).

**Écarts à la fiche** :

- T-a0 différé (ci-dessus, C-30-2-27).
- Bouton *Télécharger* gardé à côté de l'aperçu (C-30-2-24).
- AC9-bis : le *Passer à la 0.14.0* et le bandeau du CHANGELOG **n'annoncent pas** `INVALID_PAYMENT_DETAILS` (30-2b)
  ni la lecture du document par les clés `read` (30-2c) — la 30-2a n'écrit que ce qu'elle livre (C-30-2-29) ; à
  compléter par les deux sous-stories.
- AC8 : libellés du journal d'activité passés au vocabulaire du document ; messages d'`errors.rs` non touchés
  (C-30-2-26). `-view-doc` devient « Télécharger », `-view-source` « Voir le document », `-no-source-doc` et
  `-source-doc-gone` sans « justificatif ». Conservés, écrits ici : `imported-supplier-invoices-complete`
  « Compléter » (AC5 : 30-2a), `-completed` « Facture créée. », `-save` « Créer une facture », `-mismatch` « écart à
  corriger » (écart de montant, non *Écarter*), les entrées historiques du CHANGELOG (v0.4.0) et les commentaires
  datés du catalogue fr-CH (l. 2277) et du menu (`+layout.svelte:93`, complété).
- Le PDF sans QR des tests est une constante (Rust `PDF_SANS_QR`, TypeScript `helpers/inbox.ts`) ; pour le relevé
  AC6-bis, il est écrit depuis la même constante dans le dossier d'import de test, non versé à `fixtures/`.
- Le pilotage de Firefox (AC6-bis) passe par geckodriver (WebDriver) : Playwright ne pilote pas le Firefox de la
  station.
- Section `.env.example` : commentaire mis au vocabulaire neuf (aucune variable ne change).

**Choix ajoutés au registre** : C-30-2-24 à C-30-2-29.

**Relevé AC6-bis — l'aperçu dans de vrais navigateurs à fenêtre** (binaire construit, CSP servie par lui :
`img-src 'self' data: blob:; frame-src 'self' blob:; object-src 'self' blob:`, autres directives inchangées ;
fixtures synthétiques `spc_e2e_invoice.png` et le PDF « Pas une facture ») — script `302a-ac6bis/releve.mjs` du
scratchpad, journal `302a-ac6bis.log`, captures `/home/gcorbaz/devel/kesh-gate-logs/302a-ac6bis/*.png` :

| Navigateur | Thème | PNG | PDF |
|---|---|---|---|
| Chromium 147.0.7727.15 (Playwright, `headless: false`) | clair, sombre | affiché (`naturalWidth` 512) | **affiché** — visionneuse Chrome, page « Pas une facture » (capture) ; document du cadre `application/pdf`, URL `blob:` |
| Google Chrome 155.0.8059.79 (station) | clair, sombre | affiché | **affiché** (idem) |
| Firefox 157.0.1 (station, geckodriver) | clair, sombre | affiché | **affiché** — pdf.js dans le cadre (capture), document privilégié |

Aucun message de CSP en console ; seul message d'erreur relevé : `401` de la lecture de session **avant** la connexion,
sans rapport. Ni la CSP ni son test n'ont eu à être corrigés. ⚠️ **« Sombre »** : Kesh n'a pas de thème sombre
(C-28-1-7) ; la page reste claire, seule la préférence du navigateur change — même constat qu'à la 29-9. La
visionneuse de Chrome titre le document par l'UUID de l'URL `blob:`, non par son nom : un « Enregistrer » depuis la
visionneuse proposerait ce nom ; le bouton *Télécharger* de la ligne garde le nom d'origine.

**Mutations (T-a9)** — campagne sous le verrou de gate, `302a-mutations.log`, motifs littéraux vérifiés uniques
(`muter.pl`), restauration `git checkout -- f && touch f` (et `touch` de `kesh-db/src/lib.rs` et du binaire de test
pour les mutations de migration et de squash, embarquées à la compilation) ; aucune erreur de compilation dans le
journal (`grep -c "error\[E"` → 0) :

| Mutation | Verdict | Tuée par |
|---|---|---|
| M-a1 `Ok(None)` → `failed` `NO_QR_CODE_FOUND` | tuée | 6 tests dont `import_accepts_png_without_qr` |
| M-a2 erreurs de décodage → `failed` | tuée | `import_unreadable_qr_…`, `import_accepts_pdf_without_qr` |
| M-a3 `ImageDecode` accepté | tuée | `import_corrupt_png_is_still_rejected` |
| M-a4 contrôle de longueur retiré | tuée | `import_field_too_long_…`, test d'entité |
| M-a5 contrainte amputée de `currency`, migration **et** squash | tuée | `qr_columns_are_all_or_nothing` |
| M-a5x la même, migration seule | tuée | `test_schema_guard` (`squash_matches_real_schema_structure`, `published_migrations_keep_their_checksums`) |
| M-a6 `iban_present` vrai au retrait | tuée | `discard_document_without_qr_traces_iban_absent` |
| M-a7 contrôle de devise sans QR (`.or(Some(""))`) | tuée | `complete_document_without_qr` |
| M-a8 / M-a8b CSP sans `blob:` (`frame-src` / `object-src`) | tuées | `csp_value_is_pinned_and_allows_blob_previews` |
| M-a9 URL non révoquée | tuée | Vitest `DocumentPreview` (deux tests) |
| M-a10 CSV `None` littéral | tuée | `serialize_imported_supplier_invoices_csv_cellules_vides_sans_qr` |
| M-a11a plage du montant retirée / M-a11b décimales retirées | tuées | `import_amount_out_of_range_…` et test d'entité, chacune |
| M-a12 longueur en octets | tuée | `import_field_too_long_…` (nom de 70 « é »), test d'entité |
| M-a14a / M-a14b filet `DataLengthOrRange` / `CheckConstraintViolation` retiré | **survivantes — équivalentes, déclarées** par la fiche : aucun des deux filets n'est atteignable une fois le contrôle pur juste |
| M-a15 `iban_present` vrai à la création | tuée | `import_accepts_png_without_qr` |

M-a13 n'existe plus (retirée en P3). Bilan : **16 tuées sur 18 jouées**, les deux survivantes étant M-a14, déclarée
équivalente.

**Tests neufs** — recomptés par `grep -cE '#\[(sqlx::)?test'` / `grep -cE "^\s*it\("` aux deux bornes `5b566378` →
`09c0057b` : `inbox_import_e2e.rs` 17 → 29 (+12, et deux réécrits : `…_accepts_no_qr`, `…_is_accepted_without_qr_data`),
`imported_supplier_invoices_repository.rs` 7 → 10 (+3), entité 3 → 6 (+3), `migrations_upgrade_path.rs` 13 → 14 (+1, et
un renommé), `csp.rs` 0 → 1, `csv_tables.rs` 19 → 20 : **+21 Rust**. Vitest : `DocumentPreview.test.ts` 0 → 8,
`import-page.test.ts` 0 → 5 (**+13**), `FiscalYearChecks.test.ts` 11 → 11 (un test étendu). E2E :
`supplier-documents.spec.ts` 0 → 3 (E1-a, E2-a, E3), `inbox-import.spec.ts` 3 → 3 (adaptés, plus de `test.skip`).

**Gates** — premier gate backend complet, sur le code de `09c0057b` : `scripts/test-fast.sh` vert, **3767 passés,
6 ignorés** (`302a-gate1.log`). E2E ciblé (deux specs du dossier d'import) : **6/6** (`302a-e2e-cible.log`).

**Gate final et E2E complet**, au HEAD `af0d2eb4` (dont le code est celui de `09c0057b`, dernier commit de code), sous
le verrou, base `kesh_302a` remise à zéro avant (`302a-final-*.log`) : `scripts/test-fast.sh` (fmt, clippy
`-D warnings`, nextest) **3767 passés, 6 ignorés** ; `npm run check` **0 erreur** (29 avertissements, aucun dans un
fichier de la story) ; `npm run lint-i18n-ownership` **PASS** ; `npm run test:unit` **1718 passés, 151 fichiers** ;
`npm run build` vert. E2E complet (`302a-e2e-complet.log`, recette de `docs/testing.md`, port 3052, base
`kesh_e2e_302a` remise à zéro, dossiers `/tmp/kesh-e2e-302a/` passés au backend et au runner) : **298 passés,
7 échoués, 17 ignorés** — les sept échecs sont, fichier par fichier, les sept KF-029 (#97) de `docs/testing.md`
§ *Les échecs attendus* (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
`onboarding.spec.ts:57`, `:77`, `:150`) ; aucune régression. Les 17 ignorés au lieu des 19 de la 29-9 : les deux
`test.skip` d'`inbox-import.spec.ts` ont disparu (ses trois tests tournent).

#### Report et remédiation de la revue de code P1 (2026-10-15, Opus 5.5, autonomie)

**Report (T-a0)** — `git rebase --onto origin/main 15686d55` : la branche part désormais de `fecc8566` (PR #688, 30-1a
squashée, qui contient le kickoff). Les hashes cités plus haut sont ceux d'avant le report : `09c0057b` → `6c5ac015`
(dernier commit de code d'origine), `af0d2eb4` → `d385ea6a`, `48bd0cd9` → `adb20154`. Conflits, tous résolus par union :
`sprint-status.yaml` à chaque commit qui le touchait (lignes `30-1*` prises de `main`, lignes `30-2*` de la branche ;
en-tête : les deux lignes de la branche renumérotées 171 → 173 et 172 → 174, au-dessus de la 172 de `main` — commit
`75020c01`) ; `CHANGELOG.md` § `[0.14.0]` (bandeau *À faire* de la 30-2a en tête, *Modifié* réuni, *Corrigé* de la
30-1a gardé). Sans conflit : `docs/api-external.md` (sections distinctes), manuels (la 30-1a n'en touche aucun), i18n
(la 30-1a change une valeur par locale, ni clé ni site `i18nMsg` du domaine ; compteurs vérifiés par les gardes au
gate, verts), `audit_route_registry.rs`, `errors.rs`, `Cargo.toml` (la 30-1a ne bumpe rien). La 30-1a n'ajoute aucune
migration : `20261015000001` reste la dernière ; P5 82 fichiers = 82 lignes d'audit ; P6/P7 verts au gate.

**Remédiation** — commits `75f3bdcd` (code : E1, E2, A6/E3) et `ed796c02` (documentation et PDF) :

- **E1** (MEDIUM) : `-qr-unreadable-hint` dit « … : la facture créée sera sans IBAN ni référence » dans les quatre
  locales et le repli ; `user-manual.tex` (§ *Documents à traiter*) et `admin-manual.tex` (§ `sec:inbox-import`) ne
  promettent plus de saisie de l'IBAN et de la référence ; doc-comments de `inbox_import.rs` et de l'entité alignés.
  Valeur grepée sur le worktree (`saisissez-les|à saisir à la main|se saisissent à la main|von Hand|a mano|by hand`) :
  ne reste que la fiche (AC de spécification des trois sous-stories) ; le CHANGELOG disait déjà « sans IBAN ni
  référence ». La 30-2b réécrira ces textes quand la saisie existera.
- **E2** (MEDIUM) : deux tests Vitest de course dans `DocumentPreview.test.ts` (lecture du document précédent résolue,
  puis rejetée, **après** le changement de document) ; **A6/E3** : un test de `import-page.test.ts` qui recharge la
  liste (import) aperçu ouvert et exige une seule lecture du document. Mutations sous le verrou
  (`302a-mutations-p1.log`), motifs uniques vérifiés, restauration `git checkout -- f && touch f` :
  M-a16 (garde `cancelled` du `.then` retirée) **tuée** (`[blob:doc-1, blob:doc-2]` au lieu de `[blob:doc-1]`) ;
  M-a16b (garde du `.catch`) **tuée** (le « non restauré » du document précédent s'affiche) ; M-a17 (`previewLoader`
  sans cache) **tuée** (`fetchDocMock` appelé 2 fois).
- **A1** : « sept autres rouges » → cinq, annoté. **A3** : « Quatre gestes », *Télécharger* décrit. **A8** : § *Status*
  à jour, le paragraphe de validation daté. **A4/A5/E7/B5** : `admin-manual.tex` (export d'installation) dit
  « documents fournisseur importés » ; la brochure ajoute « depuis la v0.14.0, tout document y entre, avec ou sans
  QR-code » ; le glossaire annote ses trois lignes (`-discard`, `-view-doc`, `-doc-gone`). **Conservés, déclarés ici**
  (AC8 ; consigne de la remédiation : textes seulement) : les doc-comments de `imported-supplier-invoices.api.ts` et ses
  noms de repli de fichier `justificatif-import-${id}` / `justificatif-${id}` (code), les doc-comments de
  `supplier-invoices/[id]/+page.svelte:220-222` et de `import/+page.ts:8`, les messages d'`AppError` d'`errors.rs`
  (C-30-2-26), `closing_checks.rs:13`, l'en-tête de `routes/imported_supplier_invoices.rs`, `admin-manual.tex:1561`
  (commentaire d'un script).
- **B1 = E4** : date d'import UTC — angle mort écrit (§ *Angles morts assumés*) ; non corrigé : aucun formatage
  UTC → local n'existe côté écran. **B3 = E6** : C-30-2-4 annoté au registre, commentaire de `inbox_import.rs`
  corrigé, angle mort écrit, et le manuel dit le refus (§ *Ce qui reste refusé* : JPEG CMJN, plus de 10 000 pixels).
- **A2** : commentaire de la migration (« cinq » colonnes déjà facultatives, elles sont huit) **laissé** : le fichier
  est appliqué à des bases que ce worktree ne voit pas toutes (bases d'autres agents ou de l'orchestrateur ayant tourné
  la branche), où P8 ferait refuser le démarrage ; le gain serait un commentaire. La précision est à la ligne d'audit
  (`docs/migrations-idempotence-audit.md`). **A7** : aucun journal de l'essai manuel de la migration n'a été versé ; ce
  qui fait foi, ce sont `imported_documents_without_qr_preserves_existing_rows` et `migrations_fresh_install`, verts.
- **LOW laissés** : B2 (`previewLoaders` jamais purgé, sans effet mesurable), B4 (`import_accepts_pdf_without_qr`
  accepte deux issues selon la présence de pdfium), E5 (aucun aller-retour de sauvegarde avec une ligne sans QR).

**PDF** : `make fr` sous le verrou (`302a-make-p1.log`), trois PDF régénérés ; phrases relues au PDF aplati (« la
facture qui en naît n'a ni IBAN ni référence », « Quatre gestes par document », « JPEG en couleurs CMJN », « documents
fournisseur importés, PDF », brochure « tout document y entre, avec ou sans QR-code ») ; « se saisissent à la main » :
0 occurrence.

**Gate complet au dernier commit de code** (`ed796c02`, dont le code est celui de `75f3bdcd`), sous le verrou, bases
`kesh_302a` et `kesh_e2e_302a` remises à zéro (`302a-p1rem-*.log`) : `scripts/test-fast.sh` **3780 passés,
6 ignorés** (3767 de la 30-2a, plus ceux de la 30-1a) ; `npm run check` 0 erreur ; `lint-i18n-ownership` PASS ;
`npm run test:unit` **1722 passés, 151 fichiers** (1718 + 1 de la 30-1a + 3 neufs) ; `npm run build` vert. E2E
complet, port 3052 : **297 passés, 8 échoués, 17 ignorés** — les sept KF-029 (#97) (`mode-expert.spec.ts:26`, `:41`,
`onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et `sidebar-navigation.spec.ts:75`
(KF-052, #424, `localStorage` à `null`), tous dans `docs/testing.md` § *Les échecs attendus* ; aucune régression.

**Tests neufs de la remédiation** (de `75020c01` à `75f3bdcd`, `grep -cE "^\s*it\("`) : `DocumentPreview.test.ts`
8 → 10, `import-page.test.ts` 5 → 6 (**+3 Vitest**) ; aucun test Rust.

### File List

30-2a, `git diff --name-status 5b566378 af0d2eb4` (58 fichiers ; A = ajouté) :

- Migration et schéma : `crates/kesh-db/migrations/20261015000001_imported_documents_without_qr.sql` (A),
  `crates/kesh-db/migrations.sha384`, `crates/kesh-db/test-schema/0001_schema_squash.sql`,
  `docs/migrations-idempotence-audit.md`, `crates/kesh-db/src/post_restore.rs`.
- Versions : `crates/{kesh-api,kesh-core,kesh-db,kesh-i18n,kesh-import,kesh-payment,kesh-qrbill,kesh-reconciliation,kesh-report,kesh-seed}/Cargo.toml`, `Cargo.lock`.
- Backend : `crates/kesh-db/src/entities/imported_supplier_invoice.rs`,
  `crates/kesh-db/src/repositories/imported_supplier_invoices.rs`, `crates/kesh-api/src/inbox_import.rs`,
  `crates/kesh-api/src/qr_decode.rs` (doc-comment), `crates/kesh-api/src/routes/imported_supplier_invoices.rs`,
  `crates/kesh-api/src/middleware/csp.rs`, `crates/kesh-api/src/exports/csv_tables.rs`.
- Tests Rust : `crates/kesh-api/tests/inbox_import_e2e.rs`, `crates/kesh-db/tests/imported_supplier_invoices_repository.rs`,
  `crates/kesh-db/tests/migrations_upgrade_path.rs`, `crates/kesh-db/tests/migrations_fresh_install.rs`.
- i18n : `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
- Frontend : `frontend/src/lib/features/imported-supplier-invoices/{DocumentPreview.svelte (A), DocumentPreview.test.ts (A), error-label.ts, imported-supplier-invoices.api.ts, imported-supplier-invoices.types.ts}`,
  `frontend/src/routes/(app)/supplier-invoices/import/{+page.svelte, import-page.test.ts (A)}`,
  `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`, `frontend/src/routes/(app)/+layout.svelte`,
  `frontend/src/lib/features/fiscal-years/{FiscalYearChecks.svelte, FiscalYearChecks.test.ts, closing-checks-labels.ts}`,
  `frontend/src/lib/shared/{i18n-keys.test.ts, i18n-un-repli-par-cle.test.ts}`.
- E2E : `frontend/tests/e2e/helpers/inbox.ts` (A), `frontend/tests/e2e/supplier-documents.spec.ts` (A),
  `frontend/tests/e2e/inbox-import.spec.ts`, `docs/testing.md`.
- Documentation : `docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}`, `docs/api-external.md`, `CHANGELOG.md`,
  `README.md`, `.env.example`.
- BMAD : `_bmad-output/implementation-artifacts/epic-30-choix-autonomes-302.md`, cette fiche,
  `sprint-status.yaml`, `30-2a-review-prompt-p1.md` (A).

## Change Log

### 2026-10-15 — Création de la fiche (Opus 5.5, autonomie)

Inventaire au code sur `15686d55` (dix prémisses, dont P-d à corriger et P-f, P-g fausses aujourd'hui) ; découpage en
trois sous-stories (C-30-2-1) ; migration breaking (C-30-2-2, C-30-2-3) ; 23 AC, 22 tâches (T-a0 à T-a9, T-b1 à T-b7,
T-c1 à T-c6), 26 mutations (M-a1 à M-a10, M-b1 à M-b8, M-c1 à M-c8), huit parcours E2E ; 19 choix au registre.
*(Annotation du 2026-10-15, validation P1 — F13, R12 : « 22 tâches » était faux à cette date ; T-a0 à T-a9, T-b1 à
T-b7 et T-c1 à T-c6 font **23** tâches, `grep -c '^- \[ \] \*\*T-'`.)*

### 2026-10-15 — Amendement : le détachement entre dans la 30-2c (décision de l'orchestrateur)

Une dette sans issue est interdite (rétrospective 28) et l'ouvrir casserait la règle de release (marge d'une issue) :
« Détacher le document » entre dans la 30-2c au plus simple (C-30-2-13 réécrit). Ajouts : **AC24** (route `DELETE
/supplier-invoices/{id}/source-document`, verrou de la seule ligne du document, vérifications au code), T-c1 à T-c6
complétées, **E7**, **M-c9** et **M-c10**, manuel et i18n. Ligne « Détacher » retirée de la table *Dehors* ; angle mort
remplacé. Décomptes vivants : **24 AC**, 22 tâches, **28 mutations** (M-c1 à M-c10), **neuf parcours E2E** (l'entrée
de création ci-dessus, datée, garde les siens).
*(Annotation du 2026-10-15, validation P1 : « 22 tâches » était faux à cette date aussi — **23**.)*

### 2026-10-15 — Validation P1 (Sonnet, lentilles R et F) et sa remédiation

Rapports : `kesh-gate-logs/302-validate-p1-R.md` (**0/0/11/9**) et `302-validate-p1-F.md` (**0/0/7/6**), sur
`3a3390c1`. Chaque finding vérifié au code avant d'être accepté (`grep -nF`, `sed -n`) ; **aucun réfuté**. Deux
devenus sans objet par la remédiation d'un autre (F11 et la moitié de R5 sur M-c1 : l'étape 3 disparaît, C-30-2-21).
Arbitrages de l'orchestrateur suivis (F1/R10, F4, F5/R6, F6, F7/R5, R9, R11).

| Finding | Sév. | Remède |
|---|---|---|
| F1 = R10 | M | AC19 : plus de lecture verrouillante sur l'index d'unicité ; l'unicité rend le `409` ; ordre des verrous et naissance de la vue écrits ; tests `_once` sur deux factures différentes (C-30-2-21) |
| F2 = R3 | M | AC2 : toutes les colonnes bornées du QR (pays 2, devise 3 compris), en caractères ; test 70/71 accentué ; M-a4 réécrite (le test exige l'acceptation) ; M-a11 (filet devenu inatteignable, équivalente déclarée), M-a12 (octets) |
| F3 = R7 | M | AC18 : `sourceDocument` à trois états (absent = non calculé, `null`, objet), calculé dans l'instantané de `get_settlement_view` ; `complete` relit par le GET ; AC21 précisé |
| F4 (+ R13) | M | AC4 : un seul `ALTER TABLE`, unicité ajoutée avant la suppression de l'index de la clé étrangère (1553), précédents cités, essai sur base fraîche et peuplée avant `kesh_e2e_302`, P8 rappelé |
| F5 = R6 | M | AC6 : CSP explicite (`frame-src` et `object-src 'self' blob:`) ; **AC6-bis** : relevé dans un vrai navigateur à fenêtre consigné au Dev Agent Record, condition du `done` |
| F6 (+ R14) | M | **AC12-bis** : validation IBAN / QR-IBAN / QRR dans la conversion commune, pour les deux routes (création manuelle comprise) ; `INVALID_PAYMENT_DETAILS` ; `isQrIban` à l'écran ; M-b9, M-b10 (C-30-2-20) |
| F7 = R5 | M | tests de concurrence sur les fonctions `_once` ; M-c7 et M-b2 déclarées équivalentes ; M-c1, M-c2 redéfinies ; M-c10 tuée par un test déterministe (facture tenue par une autre connexion, le détachement doit aboutir) |
| R1 | M | AC14 : éligibilité calculée par `proposeSuppliers` (filtre avant `rank`) ; M-b11 ; « sa présence ne prouve rien non plus » |
| R2 | M | § *Gardes existantes qui changent de sens ou rougiront* (onze gardes), tâches T-a1, T-a5, T-a7, T-b4, T-c4 |
| R4 | M | AC3 et T-a1 : PNG corrompu (`UNSUPPORTED_FILE_TYPE`), qui tue M-a3 |
| R8 | M | AC8 : grep des jetons du vocabulaire retiré (`écart`, `justificatif`, `facture d'origine`…), sites connus listés, clés orphelines de la 30-2b retirées, libellés des erreurs neuves |
| R9 | M | E2E : `beforeAll` qui échoue sans `KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR` (aucun `test.skip`), copie propre à chaque parcours (suffixe après la fin logique du fichier), attribution écrite ; E2 renforcé (aperçu pendant la saisie, IBAN à la main) |
| R11 | M | **AC9-bis** : *Passer à la 0.14.0* au manuel d'administration (sauvegarde avant, 0.13.x ne démarre plus, retour par la sauvegarde, `KESH_DOCUMENTS_DIR`) et bandeau du CHANGELOG |
| F8, R17 | L | inventaire d'AC4 complété (constructeurs de tests, tests de l'entité) ; rapport `qr` d'un document réactivé ; message `CURRENCY_NOT_SUPPORTED` ; destination après création sans document ; « inconnu » et « autre société » confondus à dessein ; comptage en caractères |
| F9 | L | AC11 : décision lue sur la ligne verrouillée ; libellés des erreurs neuves (AC8) |
| F10 | L | AC6 : autre type → lien de téléchargement, aucun cadre |
| F12 | L | AC18 : `AND status = 'completed'` à la lecture partagée |
| F13 = R12 | L | « 22 tâches » annoté **23** dans les deux entrées datées, non réécrit |
| R15 | L | AC9 : la section de `api-external.md` est **créée** ; AC23 : routes neuves au paragraphe des routes rejouées (l. 788) |
| R16 | L | AC9 : `admin-manual.tex:855-880` et `README.md:44` (G5 conservé) |
| R18 | L | citations corrigées (`lib.rs:506-513`, l. 269, `:64-67`, `inbox_import.rs:476-479`, tests de l'entité l. 152) |
| R19 | L | AC7 : « livrable seule pour le test, non pour l'usage » écrit |
| R20 | L | `QR_FIELDS_FROM_DOCUMENT` documenté à l'AC17 (30-2b), retiré de l'AC9 |

Symptômes grepés sur la fiche, le registre et le prompt (valeurs : `22`, `505-512`, `474-478`, `l. 270`, `65-67`,
`étape 3`, `M-c8`, `E1 à E6`). Décomptes vivants, recomptés depuis la source : **27 AC** (`grep -c '^\*\*AC[0-9]'` :
AC1 à AC24, AC6-bis, AC9-bis, AC12-bis), **23 tâches**, **33 mutations** (M-a1 à M-a12, M-b1 à M-b11, M-c1 à M-c10),
dont **trois déclarées équivalentes** (M-a11, M-b2, M-c7), **neuf parcours E2E**, **21 choix** (C-30-2-1 à 21 ; C-30-2-6
et 7 amendés, C-30-2-10 complété, C-30-2-20 et 21 neufs). Le prompt P1, relevé daté, n'est pas modifié.
*(Annotation du 2026-10-15, validation P2 — F1-1, R1 : M-a11 n'était **pas** équivalente à cette date, le montant
n'étant borné par personne ; voir l'entrée suivante.)*

### 2026-10-15 — Validation P2 (Opus, lentilles R et F) et sa remédiation

Rapports : `kesh-gate-logs/302-validate-p2-R.md` (**0/0/8/13**) et `302-validate-p2-F.md` (**0/0/5/8**), sur `58f65937`.
Douze des treize MEDIUM viennent de la **remédiation P1** (le motif se vérifie) ; R2 est un défaut d'origine. Chaque
finding vérifié au code (`grep -nF`, `sed -n`) ; **aucun réfuté**. Arbitrages de l'orchestrateur suivis (IBAN, montant,
R2, nom > 255, F1-5/R5, R7, R8, R4/F1-8).

| Finding | Sév. | Remède |
|---|---|---|
| F1-1 = R1 | M | AC2 : le montant est borné par le contrôle pur (`amount_min`..`amount_max`, deux décimales) — la prémisse « borné par le parseur » était fausse (`parser.rs:145`) ; M-a11 redéfinie et tuée par le test du montant hors plage |
| R2 | M | défaut d'origine : un type d'adresse hors `K`/`S` rendait 500 à tout le run et empoisonnait l'inbox ; contrôle de domaine dans le contrôle pur, filet `CheckConstraintViolation` ajouté, test, `### Corrigé` ; M-a13, M-a14 (équivalente, déclarée) ; C-30-2-22 *(annotation du 2026-10-15, validation P3 — R1 HIGH, F3-2 : faux à cette date ; `from_scanned` normalisait déjà le type, `imported_supplier_invoice.rs:117-128`, et la violation aurait rendu 400 ; retiré en P3, voir l'entrée suivante)* |
| F1-2 = R3 | M | le nom de fichier de plus de 255 caractères est inatteignable (`NAME_MAX`) : retiré ; l'assertion `archived.exists()` disparaît avec son sujet, la garantie devient un angle mort écrit ; le contrôle pur porte sur `ScannedQrBill` **avant** l'archivage (contradiction relevée par R3) |
| F1-3, F1-4 = R6 | M | AC12-bis réécrit sur `kesh_core::types::{Iban, QrIban}` (IBAN **international**, QR-IBAN CH/LI à QR-IID 30000-31999) ; QR-IBAN ⇒ QRR obligatoire et valide ; IBAN ordinaire ou absent ⇒ référence libre ou SCOR (≤ 64), jamais de forme QRR ; normalisation des espaces, vide = absent ; motifs de refus nommés ; effet sur la création manuelle écrit avec exactitude ; `payment-batches.spec.ts:107-108` gardé ; C-30-2-20 réécrit |
| F1-5 (+ L13) | M | test déterministe du rattachement sous verrou tenu (A sans valider, B sur la facture suivante sous 5 s) ; `attach_in_tx` exposé ; M-c11 |
| R4 = F1-8 | M | M-a5 et M-c2 portées sur la migration **et** le squash ; sur la migration seule, tuées par `test_schema_guard` (déclaré) |
| R5 | M | M-c9 retuée : après détachement, joindre un autre document à la même facture réussit (T-c1, E7) |
| R7 | M | AC14 : rappel avant précision — IBAN connu par les factures passées (route `supplier-candidates`), puis clé de nom (`supplierNameKey` : trémas → `ae`/`oe`/`ue`, accents, ponctuation, formes juridiques en suites de mots), égalité ou inclusion ; cas « DUBARDE S.à r.l. » / « Dubarde Sàrl Genève » et « Müller AG » / « Mueller AG » ; M-b12, M-b13 ; C-30-2-7 amendé |
| R8 | M | le spec lit les répertoires avec les défauts de la recette (`/tmp/kesh-e2e/inbox`, `/tmp/kesh-e2e/documents`), échoue s'ils manquent ; `docs/testing.md` mis à jour par T-a7, recette du `CLAUDE.md` inchangée ; parcours indépendants, seul E4 → E5 → E7 en `describe.serial` |
| F1-6 | L | « aucun verrou d'intervalle » nuancé (contrôle de doublon d'InnoDB sur une entrée subsistante) |
| F1-7 | L | AC21 : lecture dans la transaction de `get_journal_entry` (l. 479), lecteur ajouté à la garde lexicale (l. 983-1036) |
| F1-9 | L | réponses `200` + `SupplierInvoiceResponse` relue, pour le rattachement et le détachement |
| F1-10, L8 | L | AC9 : l. 1456 et 1599 du manuel d'administration ; AC9-bis : paragraphe *Intégrations par l'API* et item du bandeau |
| F1-11 | L | AC6-bis : bornes de la correction de la CSP |
| F1-12 | L | `qrIbanHasValidQrIid` extraite et partagée, pas de seconde définition |
| F1-13 | L | preuve « aucune lecture verrouillante du staging » reformulée, à refaire après AC18 |
| L1 | L | titre de C-30-2-14 corrigé (annotation datée) |
| L2, L3 | L | `inbox-import.spec.ts:118` ; commande de `api-external.md` corrigée |
| L4 | L | `admin_full_import_e2e.rs` retiré de l'inventaire (annotation) ; signature de `find_by_supplier_invoice_id_scoped` en exécuteur générique (T-c2) |
| L5 | L | tueurs prescrits pour M-a6, M-c5, M-b5 |
| L6, L7 | L | inventaire des textes sur quatre locales et formes fléchies, texte K5-help, repli `closing-checks-labels.ts:63` ; libellés `qrError`, constante morte supprimée |
| L9 | L | `user-manual.tex:1570` aligné sur « Enregistrer une facture » (AC17) |
| L10 | L | AC24 : le rattachement pendant un détachement non validé est prouvé par un test, ou l'affirmation se retire |
| L11 | L | AC16 : test du refus sans perte, états de la page `new`, rôle |
| L12 | L | angle mort du PDF corrompu écrit |

Valeurs et symptômes grepés sur la fiche, le registre et le prompt P3 (`amount_max`, `255`, `validate_iban`,
`isQrIban`, `M-a1 à M-a12`, `M-c1 à M-c10`, `l. 98`, `bank-accounts-closed`, `aucun verrou`). Les entrées datées
antérieures gardent leurs chiffres. Décomptes vivants, recomptés : **27 AC**, **23 tâches**, **38 mutations**
(M-a1 à M-a14, M-b1 à M-b13, M-c1 à M-c11), dont **trois déclarées équivalentes** (M-a14, M-b2, M-c7), **neuf parcours
E2E**, **22 choix** (C-30-2-1 à 22 ; C-30-2-7, 14 et 20 amendés, C-30-2-22 neuf).

### 2026-10-15 — Validation P3 (Sonnet, lentilles R et F) et sa remédiation

Rapports : `kesh-gate-logs/302-validate-p3-R.md` (**0/1/3/12**) et `302-validate-p3-F.md` (**0/0/3/12**),
sur `1887aea6`. Chaque finding vérifié au code ; **aucun réfuté**. Arbitrages de l'orchestrateur suivis (R1/F3-2,
R2/F3-1 avec son relevé au banc, F3-3, R3, R4, conditionnels R7, LOW, AC25).

| Finding | Sév. | Remède |
|---|---|---|
| R1 = F3-2 | H | le « défaut d'origine » du type d'adresse **n'existe pas** (`from_scanned` normalise, `imported_supplier_invoice.rs:117-128`, test l. 256 ; `CheckConstraintViolation` = 400) : contrôle de domaine, son test, M-a13 et la ligne `### Corrigé` retirés ; filet `CheckConstraintViolation` gardé en défense en profondeur, sans test, M-a14 couvrant les deux filets ; C-30-2-22 annoté ; entrée P2 annotée |
| R2 = F3-1 | M | relevé au banc de l'orchestrateur écrit à AC19 et C-30-2-21 (B n'attend jamais, < 0,35 s, quatre couples, avec et sans liens) ; test « B sous 5 s » gardé ; bascule écrite (sentinelle `companies … FOR UPDATE`) |
| F3-3 | M | AC12-bis : seul « QR-IBAN ⇒ QRR valide » ; avec un IBAN ordinaire ou sans IBAN, texte libre ≤ 64, 27 chiffres compris ; `QRR_WITHOUT_QR_IBAN` retiré (sept motifs) |
| R3 (+ F3-7, F3-12) | M | effet sur la création manuelle décrit exactement : table des pays et longueur par pays d'`Iban::new` (`iban.rs:13-101`, pays absents refusés), blancs Unicode retirés par une seule fonction par côté, lot pain.001 limité à CH/LI (existant) |
| R4 | M | libellé ×4 pour chacun des sept motifs (clés `supplier-invoices-payment-error-*`), affiché sous le champ ; test de `REFERENCE_TOO_LONG` (65 caractères) |
| R7 (conditionnels) | L | AC24 tranché : « attend puis aboutit », attente **mesurée** (`is_finished` à 300 ms), M-c9 tue le volet « aboutit » ; AC6-bis : Chromium obligatoire, Firefox s'il est installé ; PDF sans QR : `none` ou `unreadable`, badge non asserté ; fixtures sans QR générées par le code de test (constantes d'octets), sans interpréteur |
| R5 = F3-15 | L | formes juridiques retirées de la plus longue à la plus courte, liste ordonnée écrite |
| R6 | L | E7 : seconde copie du PNG sans QR |
| R8 | L | AC1 : clés d'audit présentes à `null` |
| R9 = F3-8 | L | `supplier-candidates` : `LIMIT 3`, ordre, annulées exclues, 200 vide sans QR, 404 |
| R10 | L | patron des libellés : `error-check-constraint` (`messages.ftl:25`) |
| R11 | L | garde lexicale paramétrée par module |
| R12 | L | AC24 : verrou d'intervalle bref du chemin 404 |
| R13 | L | test SQL : sept `INSERT`, une colonne nulle chacun |
| R14 = F3-6 | L | `normalize().scale() <= 2` ; les deux décimales sont une règle SIX, non une contrainte de colonne |
| R15 | L | `expectedPaymentAmount` sans contrôle : angle mort écrit |
| R16 | L | `iban.rs:136` |
| F3-5 | L | helper « payload brut » prescrit (`build_payload` valide avant de générer) |
| F3-9 | L | test d'intégration de la recherche (« Müller AG », « Mueller AG », « Muller SA ») et collation relevée |
| F3-10 | L | M-a14 couvre les deux filets ; aucune équivalence non listée |
| F3-11 | L | procédure de mutation à deux fichiers et interdiction P8 écrites |
| F3-13 | L | coût disque et bruit de K5 : angle mort |
| F3-14 | L | poignée de main runner ↔ backend au `beforeAll` ; assertions sur le nom du fichier |
| F3-16 | L | AC18 : `settlement_cancellable` sort `null` ; type TypeScript `sourceDocument?: … \| null` |
| **Ajout** | — | **AC25** (30-2c) : `paidAt` formaté par `formatSwissDate` ; Vitest ; M-c12 ; C-30-2-23 |

Valeurs et symptômes grepés (`500`, `M-a13`, `Corrigé`, `QRR_WITHOUT_QR_IBAN`, `forme QRR`, `iban.rs:11-96`,
`payment_batches.rs:785`, `AC18 à AC24`, `M-c1 à M-c11`, `C-30-2-1 à 22`). Décomptes vivants, recomptés : **28 AC**
(AC1 à AC25, AC6-bis, AC9-bis, AC12-bis), **23 tâches**, **38 mutations** (M-a1 à M-a14 sauf M-a13, retirée ; M-b1 à
M-b13 ; M-c1 à M-c12), dont **trois déclarées équivalentes** (M-a14, M-b2, M-c7), **neuf parcours E2E**, **23 choix**
(C-30-2-1 à 23 ; C-30-2-20, 21 et 22 amendés, C-30-2-23 neuf).

### 2026-10-15 — Validation P4 (Opus, lentilles R et F) et sa remédiation

Rapports : `kesh-gate-logs/302-validate-p4-R.md` (**0/0/2/13**) et `302-validate-p4-F.md` (**0/0/1/7**), sur `6e8d1266`.
La boucle converge (sévérité maximale MEDIUM, trois MEDIUM dont deux convergents). Chaque finding vérifié au code. **Un
remède refusé** (arbitrage de l'orchestrateur) : la sonde `INNODB_TRX` proposée par F3 et R5 exige le privilège
`PROCESS`, que l'utilisateur `kesh` n'a pas — le finding est accepté, son remède remplacé.

| Finding | Sév. | Remède |
|---|---|---|
| F1 = R1 | M | bascule d'AC19 corrigée : fausse citation de `create_in_tx` retirée ; sentinelle par `bank_accounts::acquire_company_sentinel_lock` prise **après** le staging et la facture, avant l'`UPDATE` ; tableau de l'ordre des verrous de chaque chemin (complétion avec ou sans projet, `pay`/`cancel`/`cancel_settlement`, détachement, 30-1 à relever) ; C-30-2-21 annoté |
| R2 (+ F5) | M | code `qrError` neuf `QR_AMOUNT_OUT_OF_RANGE` (plage et décimales), libellé ×4 et entrée d'`error-label.ts`, mention « accepté sans les données du QR » pour tout `unreadable`, `api-external.md`, chaque test asserte `qrError` ; montant **présent** seulement, montant vide accepté avec ses données ; C-30-2-4 et C-30-2-22 annotés |
| F3 = R5 | L | remède remplacé : `attendre_une_requete_en_cours` et `sonde_verrou_nowait` (`test_fixtures.rs:561`, `:611`) au lieu de 300 ms ; `INNODB_TRX` interdit (privilège `PROCESS`) |
| F2 | L | citation de la sentinelle : `bank_accounts.rs:1613`, `projects.rs:101` |
| F4 | L | ensemble de caractères écrit, commun aux deux côtés (`White_Space` + U+200B + U+FEFF), même liste testée de chaque côté |
| F6 | L | PNG de test encodé en UTF-8 avec ECI ; nom stocké asserté égal |
| F7 | L | AC6-bis : `frame-ancestors` hérité, première hypothèse |
| F8 | L | AC4 : pourquoi l'unicité ne peut pas échouer sur une base peuplée, aussi au doc-comment de la migration |
| R3 | L | M-a11 tuée par ses deux volets (16 chiffres et `20.005`) |
| R4 | L | substitution existante de `qr_invoice_png` généralisée (DRY) ; `blank_png()` nommé |
| R6 | L | registre : titre et contexte de C-30-2-22, écartée de C-30-2-7 annotés |
| R7 | L | AC25 en deux clés (`-paid-info`, `-paid-info-on`) |
| R8 | L | `REPLACE` SQL de l'espace, U+00A0 et U+202F ; le reste en angle mort |
| R9 | L | AC1 : `iban_present: false` à la création ; mutation **M-a15** |
| R10 | L | le test sur le vrai schéma fait foi, le banc étant partiel |
| R11 | L | `creditorQrIban` non IBAN → `INVALID_IBAN`, IBAN hors plage QR → `NOT_A_QR_IBAN`, deux tests |
| R12 | L | AC14 : motif de « avec QR seulement » écrit |
| R13 | L | Vitest des états d'AC10 et d'AC20 (T-b4, T-c4) |
| R14 | L | E6 sur `spc_e2e_discard.png` (QR distinct d'E1) ; *Retenir* désigné par le nom |
| R15 | L | citation littérale d'`iban.rs:11-12` ; `sprint-status.yaml` (en-tête, clés 30-2a et 30-2b) |

Valeurs grepées (`300 ms`, `is_finished`, `:271`, `:356`, `en tête`, `INNODB_TRX`, `M-a1 à M-a14`, `spc_e2e_invoice` d'E6).
Décomptes vivants, recomptés : **28 AC**, **23 tâches**, **39 mutations** (M-a1 à M-a15 sauf M-a13 ; M-b1 à M-b13 ;
M-c1 à M-c12), dont **trois déclarées équivalentes** (M-a14, M-b2, M-c7), **neuf parcours E2E**, **23 choix**
(C-30-2-4, 7, 21 et 22 annotés).

### 2026-10-15 — Passe ciblée P5 (Haiku) et note de synthèse de la boucle de validation

**P5** : `kesh-gate-logs/302-validate-p5-T.md`, une lentille sur le diff `6e8d1266..fd781d07` : **0/0/0/5**. Les cinq LOW
corrigés : T1 (clé `-qr-amount-out-of-range` ajoutée à la liste d'AC8, tenue pour normative), T2 (citation du piège
`PROCESS` : `reconciliation_e2e.rs:8139-8141`, non `:7546` — la même citation fausse figure dans les consignes communes
de l'epic), T3 (tableau de l'AC19 : `:725`, verrou du compte bancaire de `pay_in_tx`, verrou partagé de `companies`
attribué à `journal_entries::create_in_tx`), T4 (`EncodeHints::default()`, l. 259), T5 (registre C-30-2-20 aligné sur
l'ensemble de caractères de la fiche). La remédiation ne touche aucune ligne de code de production (la story n'est pas
développée) : **validation terminée**.

**Synthèse de la boucle** (chiffres repris des rapports, `kesh-gate-logs/302-validate-p{1..5}-*.md`) :

| Passe | Modèle | Lentilles | C/H/M/L |
|---|---|---|---|
| P1 | Sonnet | R 0/0/11/9 · F 0/0/7/6 | 0/0/18/15 |
| P2 | Opus | R 0/0/8/13 · F 0/0/5/8 | 0/0/13/21 |
| P3 | Sonnet | R 0/1/3/12 · F 0/0/3/12 | 0/1/6/24 |
| P4 | Opus | R 0/0/2/13 · F 0/0/1/7 | 0/0/3/20 |
| P5 (ciblée) | Haiku | T 0/0/0/5 | 0/0/0/5 |

Rotation Sonnet ↔ Opus pour les passes complètes, Haiku pour la seule passe ciblée (D6).

- **Signal de découpage** (§ *Règle de splitting préventif*) **levé en P2** (MEDIUM → MEDIUM) **et en P3** (MEDIUM →
  HIGH), **non suivi**, au titre de l'amendement D5 : les défauts n'étaient pas recyclés, mais **nés des remédiations
  précédentes** — douze MEDIUM sur treize en P2, le seul HIGH de P3 — et chacun distinct ; la fiche est déjà découpée en
  trois sous-stories de cinq modules au plus. La sévérité maximale est ensuite redescendue (P4 MEDIUM, P5 LOW).
- **Le faux défaut d'origine de la P2** : « un type d'adresse hors K/S rend 500 sur tout l'import » (R2 de la P2),
  accepté sur arbitrage de l'orchestrateur sans relecture de `from_scanned`, a fait ajouter un contrôle, un test, une
  mutation (M-a13) et une ligne `### Corrigé`. La P3 l'a réfuté (R1 HIGH = F3-2 : `from_scanned` normalise déjà,
  `imported_supplier_invoice.rs:117-128`, et la violation rendrait 400) ; l'orchestrateur a retiré son arbitrage, tout
  a été retiré et les entrées datées annotées. *Une prémisse d'arbitrage se vérifie au code comme un finding.*
- **Le banc InnoDB** : deux validations (P2 F1-6, P3 R2 = F3-1) soupçonnaient le contrôle de doublon d'un index unique
  de faire attendre deux rattachements vers deux factures différentes. L'orchestrateur l'a **mesuré** sur MariaDB
  10.11.16 (`REPEATABLE-READ`, tables jetables, quatre couples, avec et sans liens existants) : aucune attente
  (< 0,35 s). Le relevé est à l'AC19 et à C-30-2-21, avec la bascule (sentinelle `companies` après staging et facture,
  corrigée en P4) si le test rougit au développement.
- **Remède refusé** : la sonde `INNODB_TRX` proposée en P4 (F3, R5) exige le privilège `PROCESS` ; remplacée par
  `attendre_une_requete_en_cours` et `sonde_verrou_nowait`.
- **Motif** : la sévérité s'est déplacée, à chaque passe, vers ce que la remédiation précédente venait d'écrire.

Décomptes finaux, recomptés : **28 AC**, **23 tâches**, **39 mutations** dont **trois déclarées équivalentes**, **neuf
parcours E2E**, **23 choix**. Le développement de la 30-2a attend le merge de la 30-1 (T-a0).

### 2026-10-15 — Développement de la 30-2a (Opus 5.5, autonomie)

`bmad-dev-story` sur la **30-2a seulement**, en parallèle de la 30-1a (C-30-2-27 : T-a0, le rebase, reste à faire après
son merge). Commits `09c0057b` (code, dernier commit de code) et `af0d2eb4` (documentation et PDF). Tests rouges
d'abord (16 rouges sur 38), migration breaking essayée sur une base fraîche et une base peuplée avant toute base
persistante, campagne M-a1 à M-a15 (16 tuées sur 18 jouées, M-a14a et M-a14b survivantes — équivalentes déclarées),
relevé AC6-bis dans Chromium 147, Chrome 155 et Firefox 157 à fenêtre (PNG et PDF affichés partout, CSP inchangée par
le relevé). Gate final au HEAD : nextest 3767/3767 (6 ignorés), Vitest 1718/1718 (151 fichiers), check, lint et build
verts ; E2E 298 passés + les 7 KF-029 (17 ignorés). Choix C-30-2-24 à 29. Le détail est au Dev Agent Record. Statut de
la clé `30-2a-documents-sans-qr` : `review` ; revue de code P1 à lancer (`30-2a-review-prompt-p1.md`, Sonnet, lentilles
B, E, A).

### 2026-10-15 — Revue de code P1 de la 30-2a (Sonnet, lentilles B, E, A), report sur `main` et remédiation (Opus 5.5)

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/302a-review-p1-{B,E,A}.md` — **B 0/0/0/5, E 0/0/2/5, A 0/0/0/8**
(CRITICAL/HIGH/MEDIUM/LOW), aucun finding réfuté. Avant la remédiation, report de la branche sur `fecc8566` (30-1a
mergée, T-a0) : conflits de `sprint-status.yaml` et du CHANGELOG résolus par union.

| Finding | Sév. | Traitement |
|---|---|---|
| E1 | MEDIUM | texte « QR illisible » (4 locales, repli) et deux manuels : plus de saisie promise ; facture « sans IBAN ni référence » |
| E2 | MEDIUM | deux tests de course de l'aperçu ; M-a16, M-a16b tuées |
| E3 = A6 | LOW | test de stabilité de `previewLoader` ; M-a17 tuée |
| A1 | LOW | « sept » → cinq, annoté |
| A2 | LOW | migration laissée (P8), précision à la ligne d'audit |
| A3 | LOW | « Quatre gestes », *Télécharger* au manuel |
| A4, A5, E7, B5 | LOW | admin-manual, brochure, glossaire ; sites conservés déclarés au Record |
| A7 | LOW | le test fait foi, écrit au Record |
| A8 | LOW | § *Status* à jour |
| B1 = E4 | LOW | angle mort « date UTC » écrit, non corrigé |
| B3 = E6 | LOW | C-30-2-4 annoté, commentaire et manuel corrigés, angle mort écrit |
| B2, B4, E5 | LOW | laissés |

Gate complet au dernier commit de code (`ed796c02`) : nextest 3780/3780 (6 ignorés), Vitest 1722/1722 (151 fichiers),
check, lint et build verts ; E2E 297 passés, 8 échecs attendus (7 KF-029 + KF-052), 17 ignorés. Trend : P1 = 2 MEDIUM →
remédiés ; **passe ciblée P2** à lancer (`30-2a-review-prompt-p2-ciblee.md`, Haiku, une lentille, diff
`75020c01..HEAD`), la remédiation touchant du code de production (un texte d'écran et des commentaires).
