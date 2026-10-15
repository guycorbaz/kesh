# Story 30.2 : Des documents à traiter à la facture fournisseur

## Status

**ready-for-dev** (trois clés) — fiche créée le 2026-10-15 en autonomie (`bmad-create-story`, Opus 5.5), sur
`15686d55` (kickoff de l'Epic 30), worktree `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`. Validation P1 à
lancer (`30-2-validate-prompt-p1.md`, **Sonnet**, lentilles R et F).

⛔ **Trois stories, une fiche, trois PR successives** (C-30-2-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`30-2a-documents-sans-qr`** | la migration (colonnes QR facultatives, « tout ou rien », unicité de `supplier_invoice_id`, `min_required` 0.14.0) ; l'import accepte un document sans QR ou au QR illisible ; la liste « Documents à traiter » avec aperçu (composant et CSP) et « Retirer de la liste » ; la complétion d'un document sans QR par le formulaire existant ; export CSV ; contrôle K5 | AC1 à AC9 | **`refs #685`** — ⛔ jamais `closes` |
| **`30-2b-creation-cote-a-cote`** | le formulaire extrait en composant ; la page `/supplier-invoices/new`, seule ou à côté du document ; coordonnées QR verrouillées ; corps de complétion = corps de création, projet compris ; fournisseur proposé, ou créé sans quitter la page | AC10 à AC17 | **`refs #685`** |
| **`30-2c-document-joint-et-consulte`** | joindre un document après coup, le **détacher** (décision de l'orchestrateur, C-30-2-13) ; la fiche de la facture et le détail de l'écriture montrent le document ; lecture ouverte à tout rôle authentifié ; E2E de parcours complet ; manuel, CHANGELOG et `api-external.md` complétés | AC18 à AC24 | **`closes #685`** |

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
`epic-30-choix-autonomes-302.md` (C-30-2-1 à 19).

## Prémisses de #685, établies au code (`15686d55`)

| # | Prémisse | Verdict | Preuve |
|---|---|---|---|
| P-a | L'import lit le dossier `KESH_INBOX_DIR` et « refuse un document sans QR-code » | **vraie** | `crates/kesh-api/src/inbox_import.rs:486-512` : décodage puis `Ok(None) => dispose_failed(…, ERR_NO_QR_CODE_FOUND)` (l. 497-505) ; le fichier part dans `<inbox>/failed/`. Libellé « Aucune QR-facture détectée » : `crates/kesh-i18n/locales/fr-CH/messages.ftl:2167`, `frontend/src/lib/features/imported-supplier-invoices/error-label.ts:33` |
| P-a′ | *(au-delà de l'issue)* Cause structurelle du refus | — | la table de staging exige les données du QR : sept colonnes `NOT NULL` (`crates/kesh-db/migrations/20260629000001_imported_supplier_invoices.sql`, `creditor_iban` … `currency`) ; l'entité les porte en `String`/`bool` (`crates/kesh-db/src/entities/imported_supplier_invoice.rs:32-47`). D'où la migration (C-30-2-2) |
| P-a″ | *(au-delà de l'issue)* Quatre autres refus tiennent au QR, non au document | — | `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR` (`map_decode_error`, l. 656-663 ; un PDF de plus de `KESH_INBOX_MAX_PDF_PAGES` pages sans QR rend `PdfRender`, `qr_decode.rs:169-178` ; pdfium absent aussi, l. 154) et `FIELD_TOO_LONG` (l. 556-570) — traités comme l'absence de QR (C-30-2-4) |
| P-b | La liste « à compléter » offre *Compléter* et *Écarter* | **vraie** | `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte` (612 l.) : boutons l. ~455-472 du gabarit (`imported-supplier-invoices-complete`, `-discard`, `-view-doc`) ; *Voir le justificatif* **télécharge** (`downloadImportedSourceDocument`, `imported-supplier-invoices.api.ts:44`), il n'y a **aucune prévisualisation** |
| P-c | La saisie manuelle (*Nouvelle facture*) n'accepte aucun fichier | **vraie, à une nuance près** | formulaire en ligne de `routes/(app)/supplier-invoices/+page.svelte` (bouton l. 229-237, enregistrement l. 192) ; il accepte une **image pour scanner le QR dans le navigateur** (Story 12.4, `supplier-invoice-scan.ts`), sans la conserver |
| P-d | Après création, le document « quitte la liste à traiter **et le dossier d'entrée** » | **à corriger** | le fichier quitte le dossier **à l'import** : archivé sous `KESH_DOCUMENTS_DIR` (`document_storage::store_document`, l. 515) puis supprimé de l'inbox (`remove_inbox_file`, l. 623). À la création, il ne quitte que la **liste** (`mark_completed`). Comportement conservé (C-30-2-17) — à commenter sur l'issue |
| P-e | Le document reste « consultable depuis la facture » | **vraie en partie** | `GET /supplier-invoices/{id}/source-document` (`routes/imported_supplier_invoices.rs:421-436`) et bouton « Voir la facture d'origine » de la fiche (`routes/(app)/supplier-invoices/[id]/+page.svelte:284-290`) — mais le bouton est **toujours affiché**, et rend 404 pour une facture sans document ; la route est **Comptable+** (`lib.rs:505-512`) alors que la fiche se lit par tout rôle (`lib.rs:868-875`) |
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
| 30-2b | `kesh-api` (complétion), frontend (formulaire, page, contacts), `kesh-i18n`, docs | 4 |
| 30-2c | `kesh-db` (dépôt : rattachement, lecture du document d'une écriture), `kesh-api` (routes, réponses), frontend (fiche, écriture), `kesh-i18n`, docs | 5 |

## Critères d'acceptation

### Partie a — `30-2a-documents-sans-qr`

**AC1 — Un document sans QR entre dans la liste.** Given un PDF, PNG ou JPEG valide (extension et octets magiques
cohérents, taille sous la borne) **sans** QR-code dans `KESH_INBOX_DIR`, When *Importer le dossier*, Then il est
archivé, supprimé du dossier, inséré en `to_complete` avec les sept colonnes QR à `NULL` (`amount`,
`reference_value`, adresse et messages à `NULL`) ; le rapport le porte dans `accepted[]` avec `qr: "none"` ; l'audit
`imported_supplier_invoice.created` porte `qr_present: false` (et ni montant ni créancier).

**AC2 — Un QR illisible n'empêche pas l'entrée.** `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR` et un champ
QR plus long que sa colonne donnent le même résultat qu'AC1, avec `qr: "unreadable"` et `qrError: "<code>"`. Le champ
trop long se détecte **avant** l'`INSERT` par une fonction pure de `NewImportedSupplierInvoice` (longueurs des
colonnes de la migration d'origine : 34, 70, 16, 35, 40, 140) ; la branche `DbError::DataLengthOrRange` reste en
filet et garde son comportement actuel. Un document **avec** QR valide garde exactement son comportement actuel
(`qr: "present"`).

**AC3 — Les refus qui ne tiennent pas au QR sont inchangés.** Type hors liste blanche ou octets incohérents, image
non décodable (`ImageDecode` → `UNSUPPORTED_FILE_TYPE`), taille, lien symbolique, doublon (y compris un document sans
QR déposé deux fois), erreur de lecture : `failed[]` et `failed/`, comme avant. La réactivation d'un document retiré
(`discarded` → `to_complete`) vaut pour un document sans QR.

**AC4 — Migration** `crates/kesh-db/migrations/2026101?000001_imported_documents_without_qr.sql` (horodatage au-delà de
la dernière migration de `main` au moment du développement — la 30-1 peut en avoir ajouté une) :
- `MODIFY COLUMN` des sept colonnes en `NULL` (types inchangés) ;
- `ADD CONSTRAINT chk_imported_si_qr_all_or_nothing CHECK ((creditor_iban IS NULL) = (is_qr_iban IS NULL) AND …)`
  sur les sept ;
- `DROP INDEX idx_imported_supplier_invoice` + `ADD CONSTRAINT uq_imported_supplier_invoice UNIQUE
  (supplier_invoice_id)` (plusieurs `NULL` permis) ;
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
  lecteurs de ces champs à reprendre (aucun `unwrap` neuf) : `inbox_import.rs` (audit), `routes/imported_supplier_invoices.rs`
  (complétion, retrait), `exports/csv_tables.rs:1559-1631` (cellule vide pour `None`), `exports/global.rs`, le type
  `imported-supplier-invoices.types.ts` et la page d'import. Le dev recompte cet inventaire par `grep -rn
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
`application/pdf` (titre accessible = nom du fichier), un état de chargement, un état « Le document n'a pas été
restauré » sur 410 (clé existante `imported-supplier-invoices-doc-gone`), un état d'erreur sinon ; révoque l'URL
`blob:` à la destruction et au changement de document. La CSP (`crates/kesh-api/src/middleware/csp.rs:24-32`) devient
`img-src 'self' data: blob:` et gagne `frame-src 'self' blob:` (C-30-2-6) ; un test unitaire de `csp.rs` fige la
valeur (il n'en existe aucun aujourd'hui : `grep -n "mod tests" crates/kesh-api/src/middleware/csp.rs` → 0).

**AC7 — Compléter un document sans QR** (formulaire existant de la page d'import, corps inchangé en 30-2a) : pas de
contrôle de devise (C-30-2-19), pas de routage IBAN (facture sans IBAN ni référence), pas de contrôle de montant
(`staging.amount` est `NULL`) ; `expected_payment_amount` = `NULL`. Un document avec QR garde exactement les contrôles
actuels (`routes/imported_supplier_invoices.rs:183-252`). L'audit du retrait n'émet plus `iban_present` à partir d'une
chaîne : `iban_present: creditor_iban.is_some()`.

**AC8 — Textes.** Quatre locales : « Documents à traiter » et variantes (titre, menu `nav-supplier-invoices-import`,
section `-to-complete`, vide `-empty` « Aucun document à traiter. »), libellés du badge, de l'aperçu, de « Retirer de la
liste » et de sa confirmation, des deux mentions du rapport ; K5 de la préparation de clôture : titre « Documents
fournisseur à traiter », message « { $count } document(s) à traiter, toutes dates confondues. », lien « Ouvrir les
documents à traiter » (`messages.ftl:997-1000`, clés `k5-title`, `-help`, `-attention`, `-link` ; `FiscalYearChecks.svelte:224-230`). Retrait de
`imported-supplier-invoices-error-no-qr-code-found` (quatre locales) et de son entrée de `error-label.ts:33` ;
`lint-i18n-ownership`, `i18n-keys.test.ts`, les gardes de parité inter-locales et `textes_coherents.rs` verts.
Inventaire des textes à reprendre : `grep -rn "à compléter\|Factures à compléter\|Importer des factures\|Compléter"`
sur `crates/kesh-i18n/locales/fr-CH`, `frontend/src`, `docs/manual/fr` — chaque site traité ou écrit comme conservé.

**AC9 — Documentation de la 30-2a.** Manuel utilisateur FR § *Importer des factures depuis un dossier*
(`docs/manual/fr/user-manual.tex:1712-1741`) réécrit en « Documents à traiter » : tout PDF ou image entre, avec ou
sans QR ; l'aperçu ; « Retirer de la liste » ; ce qui reste refusé et le dossier `failed/` ; le fichier quitte le
dossier dès l'import ; **renvoi à la sauvegarde de `KESH_DOCUMENTS_DIR`** (manuel administrateur) et à #503 ; K5
(l. 801 et 808) ; PDF régénéré et contrôlé aplati. `docs/api-external.md` : champs `qr`/`qrError` du rapport, champs
nullables de la liste, refus `QR_FIELDS_FROM_DOCUMENT` en 30-2b. CHANGELOG `## [0.14.0] — Non publié` : `### Ajouté`
(documents sans QR, aperçu) et `### Modifié` (libellés, K5, CSP).

### Partie b — `30-2b-creation-cote-a-cote`

**AC10 — Un formulaire, une page.** `SupplierInvoiceForm.svelte` (`frontend/src/lib/features/supplier-invoices/`)
extrait du formulaire de `routes/(app)/supplier-invoices/+page.svelte` (champs, lignes, total TTC vivant, projet,
erreurs) ; la page `routes/(app)/supplier-invoices/new/+page.svelte` l'affiche :
- **sans document** (`/supplier-invoices/new`) — avec le scan navigateur 12.4 ; le bouton « Enregistrer une
  facture » de la liste y mène ; le formulaire en ligne de la liste disparaît (C-30-2-11) ;
- **à côté du document** (`/supplier-invoices/new?document=<id>`) — aperçu (AC6) fixe à gauche, formulaire à droite,
  empilés sous 1024 px ; **le document reste visible pendant toute la saisie** (pas de modal, pas de défilement qui
  l'ôte). Un `document` inconnu, d'une autre société ou qui n'est plus `to_complete` : message et lien vers la liste,
  aucun formulaire.
Le formulaire de complétion en ligne de la page d'import disparaît ; *Compléter* devient « Créer une facture » et mène
à `?document=<id>`.

**AC11 — Avec QR.** IBAN ou QR-IBAN, référence et montant attendu pré-remplis et **en lecture seule** ; le total TTC
vivant est comparé au montant du QR (indicateur existant, `import/+page.svelte` `amountMismatch`) ; le serveur garde
le refus `AMOUNT_MISMATCH`. Le corps envoyé n'en porte aucun ; le serveur refuse en `400 QR_FIELDS_FROM_DOCUMENT` un
corps qui en porte pour un document avec QR (C-30-2-9). Le scan navigateur n'est pas proposé.

**AC12 — Sans QR.** Tous les champs se saisissent, y compris IBAN, référence, montant attendu (facultatifs) et projet ;
le serveur les prend dans le corps et les valide **par le même chemin** que `POST /supplier-invoices`
(`supplier_invoices::create_in_tx`).

**AC13 — Un seul corps.** `complete_import` désérialise `CreateSupplierInvoiceRequest`
(`routes/supplier_invoices.rs:238`) ; la conversion en `NewSupplierInvoice` est une fonction commune aux deux routes
(aucune copie du bloc l. 340-362) ; `project_id` est honoré à la complétion (fin de « extension future », l. 270).
`CompleteImportRequest` et `CompleteImportLineRequest` disparaissent ; un client actuel (sans les champs ajoutés)
reste accepté (test).

**AC14 — Fournisseur proposé.** Avec QR : jusqu'à **trois** fournisseurs actifs, obtenus par
`listContacts({ search: normalizeTerm(creditorName), isSupplier: true, limit: 20 })` puis `rank`, affichés « Proposé :
<nom> » avec un bouton « Retenir » ; **aucun n'est présélectionné**, la liste déroulante reste vide tant que
l'utilisateur n'a ni retenu ni choisi (C-30-2-7). Un créancier sans correspondance : rien n'est proposé. Sans QR :
rien n'est proposé. La fonction qui calcule les propositions est **pure** et testée (Vitest).

**AC15 — Créer le fournisseur sans quitter.** « Nouveau fournisseur » ouvre un panneau de la page (C-30-2-8) : type
(Entreprise par défaut), nom pré-rempli du créancier QR, adresse structurée pré-remplie pour un QR de type `S`,
`isSupplier: true`, signal « contacts proches » (socle 22-2a) avant l'enregistrement. *Créer* appelle `createContact`
(`POST /contacts`, Comptable+), ajoute le contact à la liste et le **sélectionne** ; la saisie de la facture (lignes
comprises) est intacte ; une erreur de création (IDE en double, validation) s'affiche dans le panneau sans rien
perdre ; *Annuler* ne crée rien.

**AC16 — Après création.** Le document passe `completed`, quitte la liste ; l'écran va à la fiche de la facture
créée. Un refus (`AMOUNT_MISMATCH`, compte non imputable, exercice, `IMPORT_NOT_PENDING_COMPLETION` si le document
a été traité ailleurs entre-temps) s'affiche sans perdre la saisie.

**AC17 — Documentation de la 30-2b.** Manuel FR : § saisie d'une facture fournisseur (`sec:saisie-facture-fournisseur`)
et § documents à traiter — création côte à côte, coordonnées QR verrouillées, fournisseur proposé et « Retenir »,
création en ligne, facture sans document ; PDF régénéré. `api-external.md` : corps de
`POST /imported-supplier-invoices/{id}/complete` = celui de `POST /supplier-invoices`, `QR_FIELDS_FROM_DOCUMENT`.
CHANGELOG complété. i18n quatre locales.

### Partie c — `30-2c-document-joint-et-consulte`

**AC18 — La fiche de la facture sait si elle a un document.** `SupplierInvoiceResponse` gagne `sourceDocument:
{ originalFilename, mimeType, byteSize } | null` (lecture du staging `completed` lié, bornée à la société). La fiche
affiche « Voir le document » (aperçu AC6 dans un panneau) **seulement** s'il existe ; sinon « Joindre un document »
(Comptable+) — plus de bouton qui rend 404. Quand il existe, la fiche offre aussi « Détacher le document » (AC24).

**AC19 — Joindre après coup.** `POST /api/v1/supplier-invoices/{id}/source-document` `{ importedSupplierInvoiceId }`
(Comptable+, ouverte aux clés `read-write` comme `…/complete`), en une transaction rejouée sur interblocage :
1. `SELECT … FOR UPDATE` du staging (borné à la société) → `404 IMPORTED_INVOICE_NOT_FOUND` ; statut ≠ `to_complete`
   → `409 IMPORT_NOT_PENDING_COMPLETION` ;
2. `SELECT … FOR UPDATE` de la facture (bornée à la société) → `404` ;
3. lecture **verrouillante** d'un document déjà lié à la facture → `409 SOURCE_DOCUMENT_ALREADY_ATTACHED` ;
4. `mark_completed(staging, facture)` ; l'unicité de `supplier_invoice_id` (AC4) est le filet : sa violation rend le
   même `409` ;
5. audit `imported_supplier_invoice.attached` (`supplier_invoice_id`, `original_filename`).
Tout statut de facture accepté (ouverte, payée, annulée) ; aucun champ de la facture modifié (C-30-2-12).
**Ordre des verrous** : staging puis facture — à relire contre chaque chemin qui verrouille l'un ou l'autre
(`complete_import_once`, `discard_import`, la réactivation de l'import, `supplier_invoices::pay_in_tx`, l'annulation
de la facture et de son règlement, l'acceptation de la 30-1) ; le Dev Agent Record écrit le tableau. **Naissance de la
vue** : les trois premières lectures sont verrouillantes ; aucune lecture ordinaire avant l'étape 3.

**AC20 — Le sélecteur.** « Joindre un document » ouvre la liste des documents à traiter (nom, date, badge QR, montant
QR à côté du **total de la facture**), avec aperçu du document survolé ou choisi ; *Joindre* demande confirmation.
Vide : « Aucun document à traiter — déposez-le dans le dossier d'import puis importez-le. »

**AC21 — L'écriture montre le document.** `JournalEntryDetailResponse` gagne `sourceDocument: { supplierInvoiceId,
supplierInvoiceNumber, originalFilename } | null`, renseigné quand l'écriture est l'**achat** ou le **règlement**
d'une facture fournisseur qui a un document (C-30-2-15). La page `journal-entries/[id]` affiche « Facture fournisseur
<numéro> » (lien vers la fiche) et « Voir le document » (aperçu).

**AC22 — Lecture ouverte à tout rôle authentifié.** `GET /supplier-invoices/{id}/source-document` passe parmi les
lectures (`lib.rs`, bloc l. 868-875) ; `GET /imported-supplier-invoices/{id}/source-document` reste Comptable+
(C-30-2-14). Test : le rôle Consultation lit le document d'une facture, reçoit 403 sur celui d'un document à traiter.

**AC23 — E2E de parcours, captures, documentation de clôture.** Voir § *E2E de parcours* ; captures clair et sombre ;
manuel FR (joindre après coup, détacher, consultation depuis l'écriture, renvoi #503), PDF
régénéré et contrôlé aplati ; `api-external.md` (route neuve, `sourceDocument` des deux réponses, rôle de la lecture) ;
CHANGELOG ; `README.md` « Fonctionnalités » si la saisie des factures fournisseur y est décrite.

**AC24 — Détacher le document** (décision de l'orchestrateur, C-30-2-13). Sur la fiche d'une facture fournisseur qui
a un document, « Détacher le document » (Comptable+, confirmation : « Détacher ce document ? Il reviendra dans les
documents à traiter ; la facture et son écriture ne changent pas. ») appelle
`DELETE /api/v1/supplier-invoices/{id}/source-document` (Comptable+, clés `read-write` comme le rattachement), en une
transaction rejouée sur interblocage :
1. `SELECT … FROM imported_supplier_invoices WHERE supplier_invoice_id = ? AND company_id = ? AND status =
   'completed' FOR UPDATE` → `404 SOURCE_DOCUMENT_NOT_FOUND` (code existant) si aucune ligne — facture inconnue,
   d'une autre société ou sans document ;
2. `UPDATE … SET status = 'to_complete', supplier_invoice_id = NULL WHERE id = ? AND company_id = ? AND status =
   'completed'` (fonction `detach_in_tx` du dépôt, garde de statut dans le `WHERE`, 0 ligne = erreur interne) ;
3. audit `imported_supplier_invoice.detached` (`supplier_invoice_id`, `original_filename`).
Tout statut de facture accepté (ouverte, payée, annulée) — comme le rattachement ; la facture, ses lignes, son écriture
d'achat et de règlement **ne sont pas touchées** ; aucun fichier n'est supprimé ; le document réapparaît dans la liste
avec ses données QR. Après succès, la fiche affiche « Joindre un document ».
**Vérifié au code (`15686d55`)** : rien ne lie `status` et `supplier_invoice_id` en base (seuls `chk_imported_si_status`
et la clé étrangère `ON DELETE SET NULL`, `20260629000001_imported_supplier_invoices.sql:65-67`) ; la réactivation
écrit déjà `supplier_invoice_id = NULL` (`reactivate_to_complete`, `repositories/imported_supplier_invoices.rs:247-258`) ;
l'unicité de l'AC4 admet plusieurs `NULL` ; `find_by_supplier_invoice_id_scoped` (l. 157-171) ne filtre pas le statut
et ne retrouve plus la ligne détachée ; aucun statut de facture n'intervient dans le lien.
**Verrous** : la transaction ne verrouille **que** la ligne du document, jamais la facture. Avec le rattachement
(staging neuf, puis facture, puis lecture verrouillante de l'ancien document — AC19), aucun cycle : le détachement
ne demande rien de ce que tient le rattachement ; le rattachement attend le détachement à son étape 3, puis voit la
facture libre. Un détachement qui verrouillerait la facture **après** le document formerait un cycle avec AC19 :
interdit (M-c10).
**Angle mort assumé** : un document détaché d'une facture **avec QR** peut être « Créer une facture » une seconde fois
— une seconde facture pour la même pièce, la première restant. Geste humain explicite ; le manuel le dit (annuler la
facture fausse d'abord).

## Tasks / Subtasks

### 30-2a

- [ ] **T-a0** Rebase sur `origin/main` après le merge de la 30-1 ; relire la dernière migration et la version des crates.
- [ ] **T-a1** Tests rouges d'abord (AC1-AC3, AC7) dans `crates/kesh-api/tests/inbox_import_e2e.rs` et les tests du
  staging : PNG sans QR, PDF sans QR (accepté `none` **ou** `unreadable` selon pdfium : le test n'exige que
  l'acceptation et l'absence de données QR), QR invalide, champ trop long, doublon sans QR, réactivation sans QR,
  complétion sans QR, liste qui rend des `null`.
- [ ] **T-a2** Migration, squash, audit P5, P6, P7, bump `min_required` et crates (AC4).
- [ ] **T-a3** Entité et dépôt en `Option`, `without_qr`, contrôle des longueurs, import (AC1-AC3), complétion et retrait (AC7), export CSV.
- [ ] **T-a4** CSP et son test (AC6).
- [ ] **T-a5** `DocumentPreview.svelte` + Vitest (image, PDF, 410, erreur, révocation) ; liste, badge, rapport, retrait (AC5).
- [ ] **T-a6** Textes et i18n (AC8) ; K5.
- [ ] **T-a7** E2E (§ E2E, E1-a, E2-a, E3) ; captures clair et sombre de la liste et de l'aperçu.
- [ ] **T-a8** Manuel, PDF, `api-external.md`, CHANGELOG (AC9).
- [ ] **T-a9** Mutations M-a1 à M-a10 (sous le verrou de gate) ; gate complet (`kesh-db` touché : gate complet même en boucle) ; E2E complet au dernier commit de code.

### 30-2b

- [ ] **T-b1** Tests rouges : complétion avec corps de création (projet, IBAN sans QR), `QR_FIELDS_FROM_DOCUMENT`, client sans les nouveaux champs.
- [ ] **T-b2** `CreateSupplierInvoiceRequest` partagé, conversion commune (AC12, AC13).
- [ ] **T-b3** `SupplierInvoiceForm.svelte` extrait ; Vitest du composant (champs verrouillés avec QR, total, erreurs).
- [ ] **T-b4** Page `/supplier-invoices/new` (AC10) ; liste et page d'import reliées ; formulaires en ligne retirés.
- [ ] **T-b5** Propositions de fournisseur (fonction pure + Vitest), panneau de création (AC14, AC15).
- [ ] **T-b6** i18n, manuel, PDF, `api-external.md`, CHANGELOG (AC17).
- [ ] **T-b7** E2E E1, E2, E4 ; captures clair et sombre de la page côte à côte et du panneau fournisseur ; mutations M-b1 à M-b8 ; gates.

### 30-2c

- [ ] **T-c1** Tests rouges : rattachement (succès, 404, 409 ×2, autre société, facture payée), concurrence (deux
  rattachements sur la même facture : un seul gagne ; rattachement contre complétion du même document : un seul
  gagne), `sourceDocument` des deux réponses, RBAC de la lecture ; détachement (succès sur facture ouverte et payée, 404
  sans document ou autre société, document revenu `to_complete` et listé, facture et écritures inchangées, audit,
  rattachement concurrent du détachement sur la même facture : pas d'interblocage).
- [ ] **T-c2** Dépôt `kesh-db` : `attach_to_supplier_invoice_in_tx`, `detach_in_tx`, lecture du document d'une facture et d'une écriture.
- [ ] **T-c3** Routes (rattacher, détacher), réponses, déplacement de la lecture (AC18, AC19, AC21, AC22, AC24) ; registre des routes mutantes
  (`audit_route_registry.rs`, compteurs recomptés), registre des libellés d'audit (`crates/kesh-api/tests/audit_label_registry.rs`) pour
  les actions neuves `imported_supplier_invoice.attached` et `.detached`.
- [ ] **T-c4** Fiche de la facture (dont « Détacher le document » et sa confirmation), sélecteur, page de l'écriture (AC18, AC20, AC21, AC24).
- [ ] **T-c5** i18n, manuel, PDF, `api-external.md`, CHANGELOG, README (AC23).
- [ ] **T-c6** E2E de parcours complets (E1 à E7) ; captures ; mutations M-c1 à M-c10 ; gate complet et E2E complet au dernier commit de code.

## Dev Notes

### Tests (récapitulatif, rouges d'abord)

- **Rust** : `inbox_import_e2e.rs` (AC1-AC3), tests des routes du staging (AC7, AC11-AC13, AC19, AC22), tests du
  dépôt (`imported_supplier_invoices`, AC4 contrainte « tout ou rien » et unicité directement en SQL), `csp.rs`
  (AC6), `csv_tables.rs` (cellule vide), réponses `SupplierInvoiceResponse` et `JournalEntryDetailResponse`.
- **Vitest** : `DocumentPreview`, `SupplierInvoiceForm`, propositions de fournisseur, panneau de création, pages
  (liste, `new`, fiche, écriture) — patron des `*-page.test.ts` existants.
- **Fixtures synthétiques** (§ 8 des consignes : aucune pièce réelle) : `frontend/tests/e2e/fixtures/` porte déjà
  `spc_e2e_invoice.png` et `spc_e2e_discard.png` ; ajouter un PNG **sans** QR et un PDF **sans** QR générés (une
  page de texte), versionnés, et leurs jumeaux côté `crates/kesh-api/tests/` si les tests Rust en ont besoin.

### E2E de parcours (Guy ne testera qu'en fin d'epic : ils portent la recette)

Un nouveau fichier `frontend/tests/e2e/supplier-documents.spec.ts` (le patron de dépôt dans l'inbox est celui
d'`inbox-import.spec.ts:126-140`, qui lit `KESH_INBOX_DIR`) :

| # | Parcours | Sous-story |
|---|---|---|
| E1-a | document **avec** QR déposé → import → badge QR → aperçu visible | a |
| E2-a | document **sans** QR (PNG) déposé → import → badge « Sans QR-code », aperçu `<img>` ; le fichier n'est plus dans le dossier | a |
| E3 | document qui n'est pas une facture → *Retirer de la liste* → disparaît ; le redéposer le fait revenir | a |
| E1 | document avec QR → *Créer une facture* → aperçu présent pendant la saisie, IBAN/référence/montant verrouillés, fournisseur **proposé** (contact seedé au nom du créancier) → *Retenir* → lignes égales au montant → enregistrer → la fiche montre le document, la liste ne l'a plus | b |
| E2 | document sans QR → *Créer une facture* → *Nouveau fournisseur* dans le panneau → créé et sélectionné, saisie intacte → enregistrer → la fiche montre le document | b |
| E4 | facture **sans** document depuis *Enregistrer une facture* → la fiche propose « Joindre un document » | b (vérifié en c) |
| E5 | sur la facture de E4 → *Joindre un document* → choisir un document à traiter → la fiche le montre, la liste ne l'a plus | c |
| E6 | depuis l'écriture d'achat de E1 → lien vers la facture et « Voir le document » | c |
| E7 | sur la facture de E5 → *Détacher le document* → la fiche propose de nouveau « Joindre un document », le document revient dans « Documents à traiter », l'écriture d'achat ne montre plus de document | c |

⚠️ **Le PDF dans une `<iframe>` ne se rend pas en Chromium sans tête** (pas de visionneuse) : l'E2E d'un PDF vérifie
l'`<iframe>`, son `src` en `blob:` et le type du blob, non le rendu ; les **captures** se font sur un PNG. Le montage
E2E est celui de `docs/testing.md` à la lettre (`KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR` inscriptibles, port 3052 si le
worktree en a besoin). Une **régression** se juge contre la liste des échecs attendus de `docs/testing.md`.

### Mutations (motif unique vérifié, restauration `git checkout -- f && touch f`, sous le verrou de gate)

| # | Mutation | Tuée par |
|---|---|---|
| M-a1 | `Ok(None)` revient à `dispose_failed(… ERR_NO_QR_CODE_FOUND)` | AC1 (PNG sans QR) |
| M-a2 | `INVALID_SPC_PAYLOAD` revient en échec | AC2 |
| M-a3 | `ImageDecode` accepté sans QR | AC3 |
| M-a4 | contrôle de longueur retiré (le champ trop long tombe dans le filet `DataLengthOrRange`) | AC2 (champ trop long → `failed`) |
| M-a5 | contrainte « tout ou rien » amputée d'une colonne | test SQL d'AC4 |
| M-a6 | `iban_present` calculé `true` sans IBAN | test d'audit AC1/AC7 |
| M-a7 | contrôle de devise appliqué sans QR (`currency.as_deref() != Some("CHF")`) | AC7 |
| M-a8 | CSP sans `blob:` dans `frame-src` | test `csp.rs` |
| M-a9 | `DocumentPreview` ne révoque pas l'URL | Vitest |
| M-a10 | export CSV écrit `None`/`null` littéral | test `csv_tables.rs` |
| M-b1 | un corps avec IBAN est accepté pour un document avec QR | AC11 |
| M-b2 | le corps l'emporte sur le QR (IBAN du corps enregistré) | AC11 |
| M-b3 | `project_id` ignoré à la complétion | AC13 |
| M-b4 | le premier fournisseur proposé est présélectionné | Vitest AC14 + E1 |
| M-b5 | propositions sans filtre `isSupplier` | Vitest AC14 |
| M-b6 | le panneau vide les lignes à la création du fournisseur | Vitest AC15 + E2 |
| M-b7 | contrôle `AMOUNT_MISMATCH` retiré | test existant de la complétion |
| M-b8 | le scan navigateur proposé avec un document à QR | Vitest AC11 |
| M-c1 | étape 3 retirée (seule l'unicité garde) | test : le `409` vient de l'étape 3 — vérifier que le test le distingue, sinon le dire |
| M-c2 | unicité retirée de la migration **et** étape 3 retirée | test de concurrence AC19 |
| M-c3 | rattachement accepté sur un staging `discarded` | AC19 |
| M-c4 | facture d'une autre société acceptée | AC19 (IDOR) |
| M-c5 | `sourceDocument` de l'écriture limité à l'achat | AC21 (écriture de règlement) |
| M-c6 | la lecture du document reste Comptable+ | AC22 |
| M-c7 | ordre des verrous inversé (facture puis staging) | test de concurrence rattachement/complétion — sinon équivalente, à dire |
| M-c8 | « Voir le document » affiché sans document | Vitest de la fiche |
| M-c9 | le détachement laisse `supplier_invoice_id` renseigné (seul le statut change) | AC24 (la fiche montre encore le document ; `find_by_supplier_invoice_id_scoped` le retrouve) + E7 |
| M-c10 | le détachement verrouille la facture après le document | test de concurrence détachement/rattachement d'AC24 — sinon équivalente, à dire |

### Angles morts assumés (à écrire aux doc-comments et au manuel)

- Détacher un document d'une facture avec QR permet d'en créer une seconde facture (AC24) : la première n'est ni
  annulée ni signalée.
- `.keshbackup` n'emporte pas les documents (#503) ; un document restauré sans son fichier répond 410, l'aperçu le dit.
- Un QR que rien ne décode (image floue) laisse un document « sans QR-code » : la saisie se fait à la main.
- Les propositions de fournisseur héritent de la limite du socle 22-2a (classement d'une fenêtre de 20, sans tri de
  pertinence côté serveur, #315) : **l'absence de proposition ne prouve pas l'absence du fournisseur**.
- Inbox unique par instance (L8, #199), inchangé.

### Pièges connus du dépôt, applicables ici

- P8 : une migration appliquée ne se modifie plus, pas même un commentaire ; une correction de la migration de la
  30-2a après son passage sur `kesh_e2e_302` impose de reconstruire cette base.
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
  C-30-2-1 à 19.
- `CLAUDE.md` : *Migration breaking policy* (P1-P8), *Un appariement automatique propose*, *Règle de splitting
  préventif*, *Review Iteration Rule*, *Test Locally First*.
- `docs/agents-orchestration.md` ; `docs/testing.md` § *Prérequis Playwright local* et § *Les échecs attendus*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

### 2026-10-15 — Création de la fiche (Opus 5.5, autonomie)

Inventaire au code sur `15686d55` (dix prémisses, dont P-d à corriger et P-f, P-g fausses aujourd'hui) ; découpage en
trois sous-stories (C-30-2-1) ; migration breaking (C-30-2-2, C-30-2-3) ; 23 AC, 22 tâches (T-a0 à T-a9, T-b1 à T-b7,
T-c1 à T-c6), 26 mutations (M-a1 à M-a10, M-b1 à M-b8, M-c1 à M-c8), huit parcours E2E ; 19 choix au registre.

### 2026-10-15 — Amendement : le détachement entre dans la 30-2c (décision de l'orchestrateur)

Une dette sans issue est interdite (rétrospective 28) et l'ouvrir casserait la règle de release (marge d'une issue) :
« Détacher le document » entre dans la 30-2c au plus simple (C-30-2-13 réécrit). Ajouts : **AC24** (route `DELETE
/supplier-invoices/{id}/source-document`, verrou de la seule ligne du document, vérifications au code), T-c1 à T-c6
complétées, **E7**, **M-c9** et **M-c10**, manuel et i18n. Ligne « Détacher » retirée de la table *Dehors* ; angle mort
remplacé. Décomptes vivants : **24 AC**, 22 tâches, **28 mutations** (M-c1 à M-c10), **neuf parcours E2E** (l'entrée
de création ci-dessus, datée, garde les siens).
