# Story 30.2 : Des documents à traiter à la facture fournisseur

## Status

**ready-for-dev** (trois clés) — fiche créée le 2026-10-15 en autonomie (`bmad-create-story`, Opus 5.5), sur
`15686d55` (kickoff de l'Epic 30), worktree `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`. Validation P1
faite (Sonnet, lentilles R et F : 0/0/11/9 et 0/0/7/6) et remédiée le 2026-10-15 ; validation P2 à lancer
(`30-2-validate-prompt-p2.md`, **Opus**, lentilles R et F, passe complète).

⛔ **Trois stories, une fiche, trois PR successives** (C-30-2-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`30-2a-documents-sans-qr`** | la migration (colonnes QR facultatives, « tout ou rien », unicité de `supplier_invoice_id`, `min_required` 0.14.0) ; l'import accepte un document sans QR ou au QR illisible ; la liste « Documents à traiter » avec aperçu (composant et CSP) et « Retirer de la liste » ; la complétion d'un document sans QR par le formulaire existant ; export CSV ; contrôle K5 | AC1 à AC9, AC6-bis, AC9-bis | **`refs #685`** — ⛔ jamais `closes` |
| **`30-2b-creation-cote-a-cote`** | le formulaire extrait en composant ; la page `/supplier-invoices/new`, seule ou à côté du document ; coordonnées QR verrouillées ; corps de complétion = corps de création, projet compris ; fournisseur proposé, ou créé sans quitter la page | AC10 à AC17, AC12-bis | **`refs #685`** |
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
`epic-30-choix-autonomes-302.md` (C-30-2-1 à 21).

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
**toutes** les colonnes bornées que remplit le QR, d'après la migration d'origine : `creditor_iban` 34,
`creditor_name`, `creditor_line1`, `creditor_line2` 70, `creditor_postal_code` 16, `creditor_town` 35,
`creditor_country` 2, `reference_value` 40, `currency` 3, `unstructured_message` et `billing_information` 140 — le
parseur ne borne ni le pays ni la devise, `crates/kesh-qrbill/src/parser.rs:137` et `:150`) ; elle compte en
**caractères** (`chars().count()`, `VARCHAR`/`CHAR` en `utf8mb4`), non en octets — test : un nom accentué de 70
caractères accepté avec ses données, de 71 accepté **sans** données QR (`unreadable`, `qrError: "FIELD_TOO_LONG"`).
Les colonnes du fichier (`storage_path` 512, `original_filename` 255, `mime_type` 100) ne viennent pas du QR : un nom
de fichier de plus de 255 caractères reste refusé en `FIELD_TOO_LONG`, comme aujourd'hui (le contrôle pur le détecte
aussi, avant l'archivage). La branche `DbError::DataLengthOrRange` reste en filet ; avec le contrôle complet, elle
n'est plus atteignable que par un défaut du contrôle lui-même (le montant est borné par le parseur, `amount_max`) —
une mutation sur elle est équivalente (M-a11, déclarée). Un document **avec** QR valide garde exactement son
comportement actuel (`qr: "present"`). Un document **réactivé** (retiré puis redéposé) porte `qr: "present"` ou
`qr: "none"` selon la ligne stockée (un QR illisible n'est pas stocké : il redevient `none`).

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
  `ADD CONSTRAINT uq_imported_supplier_invoice UNIQUE (supplier_invoice_id)` (plusieurs `NULL` permis) **puis**
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
  (`exports/global.rs` appelle seulement `list_all_by_company` ; `kesh-seed` n'a aucun usage). Le dev recompte cet inventaire par `grep -rn
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
worktree) **et** dans Firefox s'il est installé, en thème clair et sombre ; il consigne au Dev Agent Record le
navigateur et sa version, le résultat (affiché / vide / bloqué, avec le message de la console s'il y en a un) et le
chemin des captures (sous `$GATE_LOGS`, jamais au dépôt si elles montrent autre chose qu'une fixture). Si l'aperçu est
bloqué, la CSP se corrige **et** le test de `csp.rs` avec elle, puis le contrôle se refait. Sans ce relevé, la 30-2a
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
Les libellés des erreurs neuves (`QR_FIELDS_FROM_DOCUMENT` en 30-2b, `INVALID_PAYMENT_DETAILS` en 30-2b,
`SOURCE_DOCUMENT_ALREADY_ATTACHED` en 30-2c) suivent le patron `errors.rs:3045-3049` (clé `error-…`, quatre locales).

**AC9 — Documentation de la 30-2a.** Manuel utilisateur FR § *Importer des factures depuis un dossier*
(`docs/manual/fr/user-manual.tex:1712-1741`) réécrit en « Documents à traiter » : tout PDF ou image entre, avec ou
sans QR ; l'aperçu ; « Retirer de la liste » ; ce qui reste refusé et le dossier `failed/` ; le fichier quitte le
dossier dès l'import ; **renvoi à la sauvegarde de `KESH_DOCUMENTS_DIR`** (manuel administrateur) et à #503 ; K5
(l. 801 et 808) ; PDF régénéré et contrôlé aplati. ⛔ **Deux phrases gardées par `textes_coherents.rs` G5**
(`aucun_dossier_surveille`, l. 584-614) restent mot pour mot : « L'import ne se déclenche pas tout seul » au manuel,
« import lancé depuis l'écran » au `README.md:44`. Manuel d'administration § import de factures (`admin-manual.tex:855-880`)
et `README.md:44` mis à jour (documents avec ou sans QR, « documents à traiter »). `docs/api-external.md` n'a **aucune
section** pour ces routes (`grep -n -i "inbox\|imported-supplier" docs/api-external.md` ne rend que des renvois, l. 253,
768, 788) : la 30-2a la **crée** — `POST /inbox-import` (rapport, `qr`/`qrError`), `GET /imported-supplier-invoices`
(champs nullables), `POST …/{id}/discard`, `GET …/{id}/source-document`. CHANGELOG `## [0.14.0] — Non publié` :
`### Ajouté` (documents sans QR, aperçu) et `### Modifié` (libellés, K5, CSP).

**AC9-bis — Passer à la 0.14.0** (migration breaking, `min_required` 0.14.0). Manuel d'administration : sous-section
`\subsubsection{Passer à la 0.14.0}\label{sec:maj-0-14-0}`, avant celle de la 0.13.4 (`admin-manual.tex:2012`), sur
leur patron : sauvegarder **avant** la mise à jour (dump, `.keshbackup` **et** le dossier `KESH_DOCUMENTS_DIR`, que le
`.keshbackup` n'emporte pas — #503) ; une fois la 0.14.0 démarrée, la 0.13.x **ne démarre plus** sur la base ; le retour
arrière passe par la restauration de cette sauvegarde. CHANGELOG : bandeau « ⚠️ **À faire en passant à la 0.14.0** —
le détail est au manuel d'administration, *Passer à la 0.14.0* : … » en tête de la section `[0.14.0]`, sur le patron
des l. 21, 54 et 135. PDF du manuel d'administration régénéré.

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
que s'il vient d'un scan. D'où **AC12-bis**. À l'écran, un IBAN saisi à la main est classé par une fonction pure
(`isQrIban`, Vitest) et routé vers `creditorIban` ou `creditorQrIban`, la référence suit.

**AC12-bis — Les coordonnées de paiement se valident à un seul endroit** (C-30-2-20). La conversion commune de l'AC13
contrôle, pour **les deux routes** (`POST /supplier-invoices` et la complétion sans QR — un seul chemin, c'est voulu) :
`creditorIban` est un IBAN valide (somme de contrôle) et **pas** un QR-IBAN (`kesh_qrbill::validation::validate_iban`,
`validate_qr_iban`, `crates/kesh-qrbill/src/validation.rs:40` et `:72`, dépendance déjà présente de `kesh-api`) ;
`creditorQrIban` est un QR-IBAN valide ; les deux ne sont pas renseignés ensemble ; une référence QRR exige un QR-IBAN et
réciproquement, et une QRR passe `validate_qrr` (l. 141). Les valeurs sont enregistrées **normalisées**
(`normalize_iban`). Refus : `400 INVALID_PAYMENT_DETAILS`, `details.field` ∈ {`creditorIban`, `creditorQrIban`,
`paymentReference`} et `details.reason` (code, jamais une phrase). **Effet sur la création manuelle actuelle** : elle
refuse désormais un IBAN invalide ou un QR-IBAN placé en `creditorIban` qu'elle acceptait — les factures existantes ne
sont pas touchées ; `### Modifié` au CHANGELOG, `api-external.md`. Le dev recense les tests et données qui posent un IBAN
factice (`grep -rn "creditorIban\|creditor_iban" crates/*/tests frontend/tests crates/kesh-seed`) et les corrige avec
des IBAN valides synthétiques. Un document **avec** QR n'est pas concerné (ses valeurs viennent du QR, déjà validées au
décodage, `INVALID_IBAN`).

**AC13 — Un seul corps.** `complete_import` désérialise `CreateSupplierInvoiceRequest`
(`routes/supplier_invoices.rs:238`) ; la conversion en `NewSupplierInvoice` est une fonction commune aux deux routes
(aucune copie du bloc l. 340-362) et porte la validation de l'AC12-bis ; `project_id` est honoré à la complétion
(fin de « extension future », `routes/imported_supplier_invoices.rs:269`). La réponse de la complétion est relue par la
même fonction que le GET (AC18) une fois la transaction validée.
`CompleteImportRequest` et `CompleteImportLineRequest` disparaissent ; un client actuel (sans les champs ajoutés)
reste accepté (test).

**AC14 — Fournisseur proposé.** Avec QR : jusqu'à **trois** fournisseurs actifs, obtenus par
`listContacts({ search: normalizeTerm(creditorName), isSupplier: true, limit: 20 })` puis `rank`, affichés « Proposé :
<nom> » avec un bouton « Retenir » ; **aucun n'est présélectionné**, la liste déroulante reste vide tant que
l'utilisateur n'a ni retenu ni choisi (C-30-2-7). La recherche du serveur est un « OU » de mots
(`crates/kesh-db/src/util/search.rs:87`) et `rank` classe sans filtrer (`duplicate-probe.ts`) : la fonction pure
`proposeSuppliers(candidats, creditorName)` **filtre** donc avant de classer. Un candidat est **éligible** si, après
`fold` et retrait des formes juridiques et mots de liaison (liste fermée dans le module : `sa`, `ag`, `sarl`, `sagl`,
`gmbh`, `srl`, `sas`, `ltd`, `inc`, `cie`, `co`, `et`, `und`, `e`, `and`, `&`, `fils`, `soehne`), **tous** les mots
restants du plus court des deux noms figurent dans l'autre, et il en reste au moins un ; sinon il n'est pas proposé.
Un créancier sans candidat éligible : rien n'est proposé. Sans QR : rien n'est proposé. Vitest : nom exact, variante de
forme juridique (« Dubarde SàRL » / « DUBARDE S.à r.l. ») proposée ; un candidat qui ne partage qu'une forme juridique
ou un mot de liaison non proposé ; tri par `rank` parmi les éligibles. Le manuel le dit : l'absence de proposition ne
prouve pas l'absence du fournisseur, **et sa présence ne prouve pas que c'est le bon**.

**AC15 — Créer le fournisseur sans quitter.** « Nouveau fournisseur » ouvre un panneau de la page (C-30-2-8) : type
(Entreprise par défaut), nom pré-rempli du créancier QR, adresse structurée pré-remplie pour un QR de type `S`,
`isSupplier: true`, signal « contacts proches » (socle 22-2a) avant l'enregistrement. *Créer* appelle `createContact`
(`POST /contacts`, Comptable+), ajoute le contact à la liste et le **sélectionne** ; la saisie de la facture (lignes
comprises) est intacte ; une erreur de création (IDE en double, validation) s'affiche dans le panneau sans rien
perdre ; *Annuler* ne crée rien.

**AC16 — Après création.** Le document passe `completed`, quitte la liste ; l'écran va à la fiche de la facture
créée. Un refus (`AMOUNT_MISMATCH`, `INVALID_PAYMENT_DETAILS` sur son champ, compte non imputable, exercice,
`IMPORT_NOT_PENDING_COMPLETION` si le document a été traité ailleurs entre-temps) s'affiche sans perdre la saisie.

**AC17 — Documentation de la 30-2b.** Manuel FR : § saisie d'une facture fournisseur (`sec:saisie-facture-fournisseur`)
et § documents à traiter — création côte à côte, coordonnées QR verrouillées, fournisseur proposé et « Retenir »,
création en ligne, facture sans document ; PDF régénéré. `api-external.md` : corps de
`POST /imported-supplier-invoices/{id}/complete` = celui de `POST /supplier-invoices`, `QR_FIELDS_FROM_DOCUMENT`,
`INVALID_PAYMENT_DETAILS` sur les deux routes (c'est à la 30-2b, non à la 30-2a, d'écrire ces deux refus).
CHANGELOG complété (`### Modifié` : validation des coordonnées de paiement). i18n quatre locales.

### Partie c — `30-2c-document-joint-et-consulte`

**AC18 — La fiche de la facture sait si elle a un document.** `SupplierInvoiceResponse` gagne `sourceDocument`, à
**trois états** sur la discipline de `settlement_cancellable` (« `None` veut dire non calculé ici, jamais non »,
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
d'intervalle** : deux rattachements simultanés vers **deux factures différentes** ne s'attendent pas. Deux
rattachements vers la **même** facture se sérialisent sur l'étape 2 ; le second reçoit le `409` à l'étape 3. Aucun
chemin existant ne verrouille une facture **puis** un staging (`grep -c "imported_supplier"
crates/kesh-db/src/repositories/supplier_invoices.rs` → 0 ; `complete_import_once` crée sa facture, il n'en verrouille
aucune existante), si bien que l'ordre staging puis facture ne ferme aucun cycle — l'inverser non plus (M-c7, déclarée
équivalente). Le Dev Agent Record écrit le tableau des chemins (`complete_import_once`, `discard_import`, la
réactivation, `pay_in_tx`, `cancel_in_tx`, `cancel_settlement_in_tx`, l'acceptation de la 30-1, le détachement).
**Naissance de la vue** : la transaction ne fait **aucune** lecture ordinaire (deux lectures verrouillantes, un
`UPDATE`, un `INSERT` d'audit) : aucun instantané n'y naît. **Tests** (rouges d'abord, appelant
`attach_source_document_once` **sans rejeu**, pour qu'un interblocage se voie) : deux rattachements simultanés vers
deux factures différentes réussissent tous deux, sans interblocage ; deux vers la même facture : un succès, un
`409 SOURCE_DOCUMENT_ALREADY_ATTACHED` ; rattachement contre complétion du même document : un seul gagne.

**AC20 — Le sélecteur.** « Joindre un document » ouvre la liste des documents à traiter (nom, date, badge QR, montant
QR à côté du **total de la facture**), avec aperçu du document survolé ou choisi ; *Joindre* demande confirmation.
Vide : « Aucun document à traiter — déposez-le dans le dossier d'import puis importez-le. »

**AC21 — L'écriture montre le document.** `JournalEntryDetailResponse` gagne `sourceDocument: { supplierInvoiceId,
supplierInvoiceNumber, originalFilename } | null`, renseigné quand l'écriture est l'**achat** ou le **règlement**
d'une facture fournisseur qui a un document (C-30-2-15), par une lecture ordinaire faite **dans la même lecture** que le
reste du détail (pas de lecture verrouillante, pas de seconde transaction) ; `null` veut dire « aucun document », le
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
avec ses données QR. Après succès, la fiche affiche « Joindre un document ».
**Vérifié au code (`15686d55`)** : rien ne lie `status` et `supplier_invoice_id` en base (seuls `chk_imported_si_status`
et la clé étrangère `ON DELETE SET NULL`, `20260629000001_imported_supplier_invoices.sql:64-67`) ; la réactivation
écrit déjà `supplier_invoice_id = NULL` (`reactivate_to_complete`, `repositories/imported_supplier_invoices.rs:247-258`) ;
l'unicité de l'AC4 admet plusieurs `NULL` ; `find_by_supplier_invoice_id_scoped` (l. 157-171) ne retrouve plus la ligne
détachée (son lien est `NULL` ; la 30-2c y ajoute `AND status = 'completed'`, AC18) ; aucun statut de facture n'intervient dans le lien.
**Verrous** : la transaction ne verrouille **que** la ligne du document, jamais la facture. Avec le rattachement
(staging neuf, puis facture, puis entrée de l'index d'unicité à l'`UPDATE` — AC19), aucun cycle : le détachement ne
demande rien de ce que tient le rattachement ; le contrôle d'unicité de l'`UPDATE` du rattachement attend l'ancienne
ligne que tient le détachement, puis, celui-ci validé (lien remis à `NULL`), aboutit. Un détachement qui verrouillerait la facture **après** le document formerait un cycle avec AC19 :
interdit (M-c10). **Test qui le tient** (déterministe, sans rejeu) : une troisième connexion verrouille la facture
(`SELECT … FOR UPDATE`) et garde le verrou ; `detach_source_document_once` doit **aboutir** pendant ce temps (borne
`tokio::time::timeout` de 5 s) — un détachement qui demanderait la facture attendrait et dépasserait la borne.
**Angle mort assumé** : un document détaché d'une facture **avec QR** peut être « Créer une facture » une seconde fois
— une seconde facture pour la même pièce, la première restant. Geste humain explicite ; le manuel le dit (annuler la
facture fausse d'abord).

## Tasks / Subtasks

### 30-2a

- [ ] **T-a0** Rebase sur `origin/main` après le merge de la 30-1 ; relire la dernière migration et la version des crates.
- [ ] **T-a1** Tests rouges d'abord (AC1-AC3, AC7) dans `crates/kesh-api/tests/inbox_import_e2e.rs` et les tests du
  staging : PNG sans QR, PDF sans QR (accepté `none` **ou** `unreadable` selon pdfium : le test n'exige que
  l'acceptation et l'absence de données QR), QR invalide, champ trop long (nom accentué 70 / 71 caractères, pays de
  trois lettres), PNG corrompu (`UNSUPPORTED_FILE_TYPE`, AC3), doublon sans QR, réactivation sans QR (rapport `qr`),
  complétion sans QR, liste qui rend des `null`. **Deux tests existants changent de sens** et se réécrivent :
  `import_rejects_unsupported_type_and_no_qr` (`inbox_import_e2e.rs:419`, qui attend `NO_QR_CODE_FOUND` : le PNG sans
  QR y devient accepté) et `import_field_too_long_returns_failed_not_500` (l. 616, qui attend `FIELD_TOO_LONG` : le
  champ QR trop long devient accepté sans QR ; **son assertion `archived.exists()`**, qui prouve qu'un fichier partagé
  n'est pas supprimé, est **conservée** sur le cas qui reste refusé — nom de fichier de plus de 255 caractères).
- [ ] **T-a2** Migration, squash, audit P5, P6, P7, bump `min_required` et crates (AC4).
- [ ] **T-a3** Entité et dépôt en `Option`, `without_qr`, contrôle des longueurs, import (AC1-AC3), complétion et retrait (AC7), export CSV.
- [ ] **T-a4** CSP et son test (AC6) ; relevé dans un vrai navigateur (AC6-bis), consigné au Dev Agent Record.
- [ ] **T-a5** `DocumentPreview.svelte` + Vitest (image, PDF, autre type, 410, erreur, révocation) ; liste, badge, rapport, retrait (AC5) ;
  gardes frontend touchées (§ *Gardes existantes*).
- [ ] **T-a6** Textes et i18n (AC8) ; K5.
- [ ] **T-a7** E2E (§ E2E, E1-a, E2-a, E3) ; `inbox-import.spec.ts` adapté (titre, libellés) ; captures clair et sombre de la liste et de l'aperçu.
- [ ] **T-a8** Manuels utilisateur et administrateur, PDF, `README.md:44`, `api-external.md` (section créée), CHANGELOG et son bandeau (AC9, AC9-bis).
- [ ] **T-a9** Mutations M-a1 à M-a12 (sous le verrou de gate ; M-a11 déclarée équivalente) ; gate complet (`kesh-db` touché : gate complet même en boucle) ; E2E complet au dernier commit de code.

### 30-2b

- [ ] **T-b1** Tests rouges : complétion avec corps de création (projet, IBAN sans QR), `QR_FIELDS_FROM_DOCUMENT`,
  client sans les nouveaux champs, `INVALID_PAYMENT_DETAILS` sur les deux routes (IBAN invalide, QR-IBAN en
  `creditorIban`, IBAN en `creditorQrIban`, les deux ensemble, QRR sans QR-IBAN, QR-IBAN sans QRR, QRR invalide), valeur
  enregistrée normalisée, réponse de `complete` qui porte `sourceDocument` (en 30-2c).
- [ ] **T-b2** `CreateSupplierInvoiceRequest` partagé, conversion commune et validation (AC12, AC12-bis, AC13) ; IBAN factices des tests et du seed remplacés par des IBAN valides synthétiques.
- [ ] **T-b3** `SupplierInvoiceForm.svelte` extrait ; Vitest du composant (champs verrouillés avec QR, total, erreurs).
- [ ] **T-b4** Page `/supplier-invoices/new` (AC10) ; liste et page d'import reliées ; formulaires en ligne retirés et
  leurs clés i18n orphelines retirées (AC8) ; `supplier-invoice-scan.spec.ts`, `supplier-invoices.spec.ts`,
  `inbox-import.spec.ts` (complétion en ligne) et `boutons-rouges-inventory.test.ts` adaptés (§ *Gardes existantes*).
- [ ] **T-b5** Propositions de fournisseur (`proposeSuppliers`, pure, éligibilité puis `rank`, Vitest), `isQrIban` (pure, Vitest), panneau de création (AC12, AC14, AC15).
- [ ] **T-b6** i18n, manuel, PDF, `api-external.md`, CHANGELOG (AC17).
- [ ] **T-b7** E2E E1, E2, E4 ; captures clair et sombre de la page côte à côte et du panneau fournisseur ; mutations M-b1 à M-b11 (M-b2 déclarée équivalente) ; gates.

### 30-2c

- [ ] **T-c1** Tests rouges : rattachement (succès, 404 ×2, `409 IMPORT_NOT_PENDING_COMPLETION`,
  `409 SOURCE_DOCUMENT_ALREADY_ATTACHED`, autre société des deux côtés, facture payée) ; concurrence, en appelant les
  fonctions `_once` **sans rejeu** (AC19) : deux rattachements vers **deux factures différentes** (deux succès, aucun
  interblocage), deux vers la même facture (un succès, un `409`), rattachement contre complétion du même document (un
  seul gagne) ; `sourceDocument` des deux réponses et ses trois états ; RBAC de la lecture ; détachement (succès sur
  facture ouverte et payée, 404 sans document ou autre société, document revenu `to_complete` et listé, facture et
  écritures inchangées, audit) ; détachement pendant qu'une autre connexion tient la facture (AC24, il aboutit).
- [ ] **T-c2** Dépôt `kesh-db` : `attach_to_supplier_invoice_in_tx`, `detach_in_tx`, lecture du document d'une facture et d'une écriture.
- [ ] **T-c3** Routes (rattacher, détacher), réponses, déplacement de la lecture (AC18, AC19, AC21, AC22, AC24) ; registre des routes mutantes
  (`audit_route_registry.rs`, compteurs recomptés), registre des libellés d'audit (`crates/kesh-api/tests/audit_label_registry.rs`) pour
  les actions neuves `imported_supplier_invoice.attached` et `.detached`.
- [ ] **T-c4** Fiche de la facture (dont « Détacher le document » et sa confirmation), sélecteur, page de l'écriture
  (AC18, AC20, AC21, AC24) ; `supplier-settlement-page.test.ts:66` (mock du module `imported-supplier-invoices.api`) adapté.
- [ ] **T-c5** i18n, manuel, PDF, `api-external.md`, CHANGELOG, README (AC23).
- [ ] **T-c6** E2E de parcours complets (E1 à E7) ; captures ; mutations M-c1 à M-c10 (M-c7 déclarée équivalente) ; gate complet et E2E complet au dernier commit de code.

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

### Gardes existantes qui changent de sens ou rougiront (à adapter dans la sous-story indiquée)

| Garde | Ce qui change | Sous-story |
|---|---|---|
| `crates/kesh-api/tests/inbox_import_e2e.rs:419` et `:616` | PNG sans QR et champ QR trop long acceptés (T-a1) | a |
| `frontend/tests/e2e/inbox-import.spec.ts` | `h1` `/Importer/i` (l. 98) devient « Documents à traiter » ; la complétion en ligne (`imported-complete-open`, `-submit`, `imported-supplier-select`) disparaît ; le `confirm()` et `imported-discard` suivent le libellé neuf. Ses deux `test.skip(!INBOX_DIR…)` (l. 127, 171) deviennent des échecs bruyants comme ceux du spec neuf | a (titre, retrait), b (complétion) |
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

### E2E de parcours (Guy ne testera qu'en fin d'epic : ils portent la recette)

Un nouveau fichier `frontend/tests/e2e/supplier-documents.spec.ts`. Il dépose dans l'inbox comme
`inbox-import.spec.ts:126-140` (lecture de `KESH_INBOX_DIR`), **mais sans son `test.skip`** : le spec porte la recette,
il ne doit pas pouvoir se taire. En tête du fichier, un `beforeAll` **échoue** (`throw`) si `KESH_INBOX_DIR` ou
`KESH_DOCUMENTS_DIR` manque ou n'est pas inscriptible — un test qui ne tourne pas ne rougit pas, il se tait.

**Fixtures propres à chaque parcours.** Le staging est unique par `(company_id, file_hash)` et un document `completed`
compte comme doublon (`inbox_import.rs:476-479`) : deux parcours qui déposent les mêmes octets s'empêchent l'un
l'autre, et un second run échoue. Chaque parcours dépose donc **sa propre copie**, rendue unique par un suffixe
aléatoire ajouté **après la fin logique** du fichier (après le bloc `IEND` d'un PNG, après `%%EOF` d'un PDF, sous
forme de commentaire), ce qu'ignorent les lecteurs — à vérifier une fois sur le PNG à QR (le décodage doit encore
réussir ; sinon, fixtures distinctes générées). Attribution : E1-a et E1 : une copie de `spc_e2e_invoice.png` **chacun** ;
E2-a et E2 : une copie du PNG sans QR chacun (E2 dépose et importe la sienne) ; E3 : une copie du PDF sans QR ; E5 :
une copie du PNG sans QR ; E7 réutilise le document joint par E5 dans le même test. Le fournisseur proposé d'E1 est
créé par l'API au début du test, au **nom exact** du créancier de la fixture.

| # | Parcours | Sous-story |
|---|---|---|
| E1-a | document **avec** QR déposé → import → badge QR → aperçu visible | a |
| E2-a | document **sans** QR (PNG) déposé → import → badge « Sans QR-code », aperçu `<img>` ; le fichier n'est plus dans le dossier | a |
| E3 | document qui n'est pas une facture → *Retirer de la liste* → disparaît ; le redéposer le fait revenir | a |
| E1 | document avec QR → *Créer une facture* → aperçu présent pendant la saisie, IBAN/référence/montant verrouillés, fournisseur **proposé** (contact seedé au nom du créancier) → *Retenir* → lignes égales au montant → enregistrer → la fiche montre le document, la liste ne l'a plus | b |
| E2 | document sans QR → *Créer une facture* → l'aperçu reste visible pendant toute la saisie (assertion avant **et** après l'ouverture du panneau et l'ajout de lignes) → IBAN et référence saisis à la main (IBAN valide synthétique) → *Nouveau fournisseur* dans le panneau → créé et sélectionné, saisie intacte → enregistrer → la fiche montre le document, et l'IBAN saisi | b |
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
| M-a4 | contrôle de longueur retiré (le champ trop long retombe dans le filet `DataLengthOrRange`, donc en `failed`) | AC2 : le test exige l'**acceptation** sans QR (`unreadable`) |
| M-a5 | contrainte « tout ou rien » amputée d'une colonne | test SQL d'AC4 |
| M-a6 | `iban_present` calculé `true` sans IBAN | test d'audit AC1/AC7 |
| M-a7 | contrôle de devise appliqué sans QR (`currency.as_deref() != Some("CHF")`) | AC7 |
| M-a8 | CSP sans `blob:` dans `frame-src` (ou dans `object-src`) | test `csp.rs` |
| M-a9 | `DocumentPreview` ne révoque pas l'URL | Vitest |
| M-a10 | export CSV écrit `None`/`null` littéral | test `csv_tables.rs` |
| M-a11 | branche `DataLengthOrRange` de l'import mutée | **équivalente, déclarée** : inatteignable une fois le contrôle pur complet (AC2) |
| M-a12 | le contrôle de longueur compte en octets (`len()`) | AC2 (nom accentué de 70 caractères accepté avec ses données) |
| M-b1 | un corps avec IBAN est accepté pour un document avec QR | AC11 |
| M-b2 | le corps l'emporte sur le QR (IBAN du corps enregistré) | **équivalente isolément, déclarée** : la garde de M-b1 refuse tout corps qui porte ces champs ; seule la paire M-b1 + M-b2 est observable, et M-b1 la tue |
| M-b3 | `project_id` ignoré à la complétion | AC13 |
| M-b4 | le premier fournisseur proposé est présélectionné | Vitest AC14 + E1 |
| M-b5 | propositions sans filtre `isSupplier` | Vitest AC14 |
| M-b6 | le panneau vide les lignes à la création du fournisseur | Vitest AC15 + E2 |
| M-b7 | contrôle `AMOUNT_MISMATCH` retiré | test existant de la complétion |
| M-b8 | le scan navigateur proposé avec un document à QR | Vitest AC11 |
| M-b9 | `isQrIban` toujours faux (QR-IBAN routé en `creditorIban`) | Vitest + `INVALID_PAYMENT_DETAILS` côté serveur |
| M-b10 | validation de l'AC12-bis retirée de la conversion commune | T-b1 (IBAN invalide accepté) |
| M-b11 | éligibilité de `proposeSuppliers` retirée (classement seul) | Vitest AC14 (candidat ne partageant que « Sàrl » proposé) |
| M-c1 | traduction de la violation d'unicité retirée (elle sort en 500) | T-c1 (même facture : `409` attendu) |
| M-c2 | unicité retirée de la migration | T-c1 (même facture : deux succès au lieu d'un) |
| M-c3 | rattachement accepté sur un staging `discarded` | AC19 |
| M-c4 | facture d'une autre société acceptée | AC19 (IDOR) |
| M-c5 | `sourceDocument` de l'écriture limité à l'achat | AC21 (écriture de règlement) |
| M-c6 | la lecture du document reste Comptable+ | AC22 |
| M-c7 | ordre des verrous inversé (facture puis staging) | **équivalente, déclarée** : aucun chemin ne verrouille une facture puis un staging (AC19), l'inversion ne ferme aucun cycle |
| M-c8 | « Voir le document » affiché sans document | Vitest de la fiche |
| M-c9 | le détachement laisse `supplier_invoice_id` renseigné (seul le statut change) | AC24 (la fiche montre encore le document ; `find_by_supplier_invoice_id_scoped` le retrouve) + E7 |
| M-c10 | le détachement verrouille la facture après le document | test d'AC24 : une autre connexion tient la facture, le détachement doit aboutir sous 5 s |

### Angles morts assumés (à écrire aux doc-comments et au manuel)

- Détacher un document d'une facture avec QR permet d'en créer une seconde facture (AC24) : la première n'est ni
  annulée ni signalée.
- `.keshbackup` n'emporte pas les documents (#503) ; un document restauré sans son fichier répond 410, l'aperçu le dit.
- Un QR que rien ne décode (image floue) laisse un document « sans QR-code » : la saisie se fait à la main.
- Les propositions de fournisseur héritent de la limite du socle 22-2a (classement d'une fenêtre de 20, sans tri de
  pertinence côté serveur, #315) : **l'absence de proposition ne prouve pas l'absence du fournisseur, et sa présence
  ne prouve pas que c'est le bon** (deux sociétés d'un groupe aux noms presque identiques restent éligibles).
- Inbox unique par instance (L8, #199), inchangé.

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
  C-30-2-1 à 21.
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
