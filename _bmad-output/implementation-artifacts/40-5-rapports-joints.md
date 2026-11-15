# Story 40-5 : rapports Stripe joints à la main — CSV admis par les relevés joints, solde de clôture saisi

Status: ready-for-dev

- **Epic** : 40 (`_bmad-output/planning-artifacts/epic-40-stripe.md`, § *40-5*, G13, C-40-0, C-40-2 (d), C-40-6) ;
  décisions de Beat **B10** (Stripe : pas de PDF, rapports joints à la main) et **B20** (le relevé est la somme de
  contrôle), `epics-38-40-cadrage.md`, qui priment.
- **Dépend de** : **38-10** seulement (table `bank_account_documents`, routes `…/documents`, K9, dialogue
  `BankAccountDocumentsDialog.svelte`) — ordre de l'epic : « 40-5 dès 38-10, en parallèle ». Rien de 40-1 à 40-4 n'est
  lu : la story vaut pour tout compte bancaire (C-38-10-1).
- **Suivie de** : **40-6** (manuel utilisateur : rapports Stripe à joindre ; aide de l'écran du compte Stripe).
- **Migration** : **aucune** (C-40-0 ; `kind`, `source`, `mime_type` sont des colonnes à domaine ouvert de 38-10,
  C-38-10-6).
- **Modules** : `kesh-api` (contrôle du fichier, téléchargement), `kesh-db` (genre `ProviderReport`, lecture de
  `mime_type`), `frontend` (dialogue des relevés joints), `kesh-i18n`.

## Story

En tant que **Comptable**,
je veux **joindre au compte bancaire Stripe d'une devise le rapport CSV** que je télécharge du tableau de bord Stripe
(solde, rapprochement des versements) — ou son impression PDF —, pour un exercice ou une période, avec le solde de
clôture qu'il annonce,
afin que ce solde serve de somme de contrôle comme un relevé PDF (écart au solde comptable, contrôle de clôture K9) et
que la pièce soit gardée et sauvegardée dans son format d'origine.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-5-1` | Joindre un rapport CSV (ou PDF) à un compte bancaire, pour un exercice ou une période. | AC-1, AC-2 |
| `FR-40-5-2` | Saisir son solde de clôture et sa devise ; l'écart et le contrôle de clôture de 38-10 s'appliquent. | AC-4 |
| `FR-40-5-3` | Télécharger la pièce dans son type d'origine. | AC-3 |
| `FR-40-5-4` *(ajoutée)* | La liste des pièces dit le type de chaque pièce (PDF, CSV) ; le dialogue accepte les deux. | AC-3, AC-6 |
| `NFR-40-5-1` | Fichier contrôlé (UTF-8, en-tête, borne de taille) ; aucun contenu interprété. | AC-1 |
| `NFR-40-5-2` | Pièce auditée et sauvegardée par 38-10 ; aucune migration. | AC-5 |
| `NFR-40-5-3` *(ajoutée)* | Additif, non régressif : les tests **serveur** de 38-10 (codes, statuts, rangement d'un PDF, téléchargement d'un PDF) restent verts sans retouche de leurs attentes ; les seules attentes de 38-10 que la story change exprès (écran : PDF seul, titre, `accept`, message de `BANK_DOCUMENT_NOT_PDF`) sont inventoriées et modifiées (AC-6) ; quatre locales ; captures. | AC-1, AC-6, AC-7 |

## Ce que le code dit (prémisses vérifiées le 2026-11-15 sur `a2b1da9c`)

- **Rangement** : `document_storage::store_document(dir, bytes, ext, original_filename, mime_type)`
  (`crates/kesh-api/src/document_storage.rs:90-131`) accepte toute extension alphanumérique de 8 caractères au plus
  (`is_safe_ext`, `:66-68`) — `csv` comprise — et nomme le fichier `{sha256}.{ext}` (`:117`) ; `mime_for_ext` ne connaît
  que `pdf`, `png`, `jpg`/`jpeg` (`:54-62`, test `:421-426`). L'inbox filtre ses extensions **avant** d'appeler
  `mime_for_ext` (`ALLOWED_EXT`, `crates/kesh-api/src/inbox_import.rs:98`, `:576`, `:733`) : y ajouter `csv` n'ouvre
  pas l'inbox aux CSV.
- **Téléchargement** : `content_disposition` rend toujours `attachment; filename=…; filename*=UTF-8''…`
  (`crates/kesh-api/src/routes/imported_supplier_invoices.rs:768-784`) ; 38-10 AC-3 la rend partagée et force
  `application/pdf` et `nosniff` au téléchargement d'une pièce de compte.
- **Restauration** : un nom de pièce de sauvegarde est contrôlé par `NOM_DE_PIECE` (`^[0-9a-f]{64}\.[a-z0-9]{1,8}$`) et
  `is_safe_storage_path` (`crates/kesh-api/src/admin_backup/export.rs:177-185`) : `{sha256}.csv` passe, sans changement
  (AC-5) ; aucun filtre d'extension à `pdf` n'existe côté export ni import.
- **Encodage** : `kesh_import::csv::detect_encoding` (`crates/kesh-import/src/csv/encoding.rs:48-100`) rend `Utf8`
  **sans valider le reste du fichier** quand il porte un BOM UTF-8 (`:58-63`), et admet `Iso8859_1` par `chardetng`
  (`:83-90`) : il ne garantit pas un UTF-8 valide et n'est pas réutilisé ici (contrôle direct, AC-1).
- **38-10 (fiche, non encore développée)** : le contrôle du fichier est « non vide qui commence par `%PDF-` » (`400
  BANK_DOCUMENT_NOT_PDF`), le fichier rangé en `pdf`/`application/pdf` (AC-2) ; `kind` (`statement`) et `source`
  (`manual`) lus en enum à variante `Other(String)` (C-38-10-6) ; `mime_type VARCHAR(100) NOT NULL` (AC-1) ; liste sans
  `mimeType` (AC-3) ; K9 retient le dernier document **à solde** du compte, sans filtre de genre (C-38-10-5) ; la
  sauvegarde lit `storage_path`, `sha256` sans regarder le type (AC-7) ; l'export CSV écarte `mime_type`
  (`COLONNES_HORS_EXPORT`, AC-7 d) et exporte `kind`. Ses Vitest d'écran (AC-9) tiennent « PDF seul côté écran » : la
  story change cette attente exprès (NFR-40-5-3).
- **Multipart** : `parse_multipart` (`crates/kesh-api/src/routes/bank_imports.rs:484-485`) tronque le nom d'origine à
  `FILENAME_MAX_BYTES` et pose `upload.xml` quand la partie n'en a pas ; il garde le type de la partie tel quel,
  paramètres compris (`:486`). **38-10 reprend ce parseur** (Dev Notes de 38-10 : « reprendre `parse_multipart` …,
  extraire un utilitaire commun si la forme diffère ») : le nom qu'il rend est donc **déjà tronqué** et, sans nom,
  `upload.xml` ; l'annonce de C-40-5-1 exige le nom d'origine (P3-1, T2).

## Écarts à l'epic

1. **Deux codes de refus, non un** : l'epic nomme `BANK_DOCUMENT_UNSUPPORTED_TYPE` pour « un CSV qui n'en est pas un ».
   Il sert ici au fichier **annoncé CSV** qui échoue au contrôle ; un fichier ni PDF ni annoncé CSV garde
   `BANK_DOCUMENT_NOT_PDF` de 38-10 (code inchangé, message élargi) — sans quoi les tests de 38-10 changeraient
   d'attente (G18).
2. **Le genre se déduit du type** : l'epic pose `kind = 'provider_report'` pour un CSV ; un PDF reste `statement`, même
   s'il est l'impression d'un rapport Stripe — aucun champ « genre » n'est saisi.
3. **Tout compte bancaire** : un CSV se joint à n'importe quel compte (C-38-10-1), non aux seuls comptes Stripe ; le
   genre garde son nom technique `provider_report`, l'écran dit « Rapport (CSV) ».

## Choix

- **C-40-5-1 — Le type se décide au contenu, puis à l'annonce.** (0) fichier **vide** → refus de 38-10, inchangé, **avant**
  toute lecture de l'annonce (un `.csv` vide n'est donc jamais `no_header`) ; (1) commence par `%PDF-` → PDF (38-10
  inchangé) ; (2) sinon, **annoncé CSV** — nom d'origine terminé par `.csv` (casse ignorée) **ou** type de la partie `text/csv`,
  `application/csv` ou `application/vnd.ms-excel` (type qu'envoie un navigateur sous Windows) — → contrôle CSV ; (3)
  sinon → `400 BANK_DOCUMENT_NOT_PDF`. Un CSV n'a pas de signature : l'annonce choisit le contrôle, le contenu le tranche.
  L'annonce se lit sur le nom **d'origine non tronqué** (`parse_multipart` de `bank_imports.rs:484-485` tronque le nom à
  `FILENAME_MAX_BYTES` et pose `upload.xml` quand il manque : ni la troncature ni ce repli ne décident du type ; seul le
  nom rangé est tronqué) et sur l'**essence** du type de la partie (paramètres après `;` retirés, casse ignorée :
  `text/csv; charset=utf-8` → `text/csv`). **L'utilitaire multipart de 38-10 rend donc, en plus du nom rangé (tronqué),
  le nom d'origine non tronqué et si la partie en portait un** (champ additif ; les tests de 38-10 n'en changent pas) :
  `classify_document` reçoit le nom d'origine, jamais le nom tronqué ni le repli `upload.xml`.
- **C-40-5-2 — Contrôle CSV, sans interprétation** : BOM UTF-8 (`EF BB BF`) retiré pour le contrôle seul (le fichier est
  rangé tel quel) ; BOM UTF-16/32 ou octets non UTF-8 (`std::str::from_utf8` strict sur le reste) → motif `not_utf8` ;
  octet nul ou caractère de contrôle autre que tabulation, `\r`, `\n` (`char::is_control` : C0, DEL et C1) → `binary` ;
  première ligne (jusqu'au premier `\n`, `\r` final retiré, mesurée après retrait du BOM) vide, de plus de 4 096 octets,
  ou sans séparateur (`,`, `;` ou tabulation) → `no_header`.
  Rien d'autre n'est lu (ni colonnes, ni solde : le solde est **saisi**, G13).
- **C-40-5-3 — Rangement d'un CSV** : extension `csv`, `mime_type = 'text/csv'` (via `mime_for_ext`, qui gagne `csv`),
  `kind = 'provider_report'`, `source = 'manual'`. Une partie **sans nom de fichier** (type `text/csv` seul) est rangée
  sous le nom d'origine `rapport.csv`, non sous le repli `upload.xml` du parseur : le téléchargement ne porterait pas
  une extension `.xml` sur un CSV (`Content-Disposition` lit le nom d'origine).
- **C-40-5-4 — Téléchargement au type stocké, par une table fermée** : `application/pdf` → `application/pdf` ;
  `text/csv` → `text/csv; charset=utf-8` ; toute autre valeur (version ultérieure, Wise 39) →
  `application/octet-stream`. Toujours `attachment` et `X-Content-Type-Options: nosniff` (un CSV n'est jamais rendu dans
  la page).

## Critères d'acceptation

1. **AC-1 — Joindre un CSV (FR-40-5-1, NFR-40-5-1, NFR-40-5-3).** `POST /api/v1/bank-accounts/{id}/documents` (38-10
   AC-2 : rôles, clés d'API, borne de corps de 25 Mio, champs, ordre des contrôles inchangés) : le contrôle du fichier
   (la place de « commence par `%PDF-` » dans l'ordre de 38-10) suit C-40-5-1 et C-40-5-2. Refus : fichier annoncé CSV
   qui échoue → `400 BANK_DOCUMENT_UNSUPPORTED_TYPE`, `details: { reason }` ∈ `not_utf8`, `binary`, `no_header`
   (constantes ; jamais un extrait du fichier) ; fichier ni PDF ni annoncé CSV → `400 BANK_DOCUMENT_NOT_PDF` (38-10) ;
   vide → refus de 38-10 inchangé. Code neuf `BANK_DOCUMENT_UNSUPPORTED_TYPE` à `crates/kesh-api/src/errors.rs`
   (variante et `build_response`, patron des codes `BANK_DOCUMENT_*` de 38-10) ; contrôle dans une fonction pure
   `classify_document(bytes, filename, content_type) -> Result<DocumentType, DocumentTypeError>` (module de 38-10 ou
   `document_storage.rs`), testée unitairement : rapport de solde Stripe (en-tête `balance_transaction_id,created_utc,…`,
   fixture **synthétique**, aucune donnée réelle) admis avec et sans BOM, fin de ligne `\r\n` ; Latin-1 (`é` en `0xE9`)
   → `not_utf8` ; BOM UTF-16 → `not_utf8` ; octet nul → `binary` ; première ligne sans séparateur ou de 4 097 octets →
   `no_header` ; fichier `.csv` commençant par `%PDF-` → PDF ; fichier `.txt` annoncé `text/plain` → `NOT_PDF` ;
   `application/vnd.ms-excel` + `.csv` → CSV ; nom `.CSV` (majuscules) → CSV ; type `text/csv` **seul** (nom sans extension)
  → CSV ; type `application/vnd.ms-excel` seul → CSV ; nom sans extension et type `application/octet-stream` → `NOT_PDF` ;
  fichier vide nommé `.csv` → refus de 38-10 (pas `no_header`) ; en-tête d'une seule colonne (sans séparateur) →
  `no_header` (limite assumée, dite au manuel) ; type `text/csv; charset=utf-8` seul → CSV ; nom de plus de 255 octets
  terminé par `.csv` → CSV (annonce lue avant troncature : test **de route**, qui exerce l'utilitaire multipart, car
  `classify_document` seul reçoit déjà le nom entier) ; partie `text/csv` sans nom → `201`, `originalFilename`
  `rapport.csv`. Tests de route : CSV joint → `201`, `kind: "provider_report"`,
   `mimeType: "text/csv"` ; doublon du même CSV → `409 BANK_DOCUMENT_DUPLICATE` (38-10) ; même CSV sur deux comptes →
   admis ; les tests **serveur** de 38-10 passent **sans retouche** de leurs attentes (NFR-40-5-3). `mime_for_ext("csv") == "text/csv"` (test
   `:421-426` étendu) ; l'inbox refuse toujours un `.csv` (`ALLOWED_EXT` inchangé ; test existant cité au Change Log, ou ajouté s'il n'y en a
   pas).
2. **AC-2 — Genre `provider_report` (FR-40-5-1).** L'enum du genre de 38-10 (`kesh-db`, entité `BankAccountDocument`,
   lecture tolérante `Other(String)`) gagne la variante `ProviderReport` ↔ `"provider_report"` (aller-retour testé ;
   `statement` et une valeur inconnue inchangés). Aucune valeur neuve dans une colonne à domaine fermé (C-40-0 : `kind`
   est `VARCHAR(32)` sans `CHECK`, 38-10 AC-1).
3. **AC-3 — Lister et télécharger (FR-40-5-3, FR-40-5-4).** `GET /api/v1/bank-accounts/{id}/documents` : chaque élément
   gagne `mimeType` (lu de la colonne ; additif). `GET …/documents/{docId}` : `Content-Type` par C-40-5-4,
   `Content-Disposition` par la fonction partagée de 38-10 (nom d'origine, `.csv` compris), `nosniff` ; refus `404
   BANK_DOCUMENT_NOT_FOUND`, `410 BANK_DOCUMENT_GONE` inchangés. Tests : le CSV se télécharge octet pour octet (BOM
   compris), `Content-Type: text/csv; charset=utf-8` ; le PDF garde `application/pdf` ; une ligne posée par SQL avec
   `mime_type = 'text/html'` → `application/octet-stream`, `attachment`.
4. **AC-4 — Solde de clôture et contrôles (FR-40-5-2).** `closingBalance`/`closingCurrency` saisis comme pour un PDF
   (38-10 AC-2, C-38-10-3 : devise du compte) ; correction et retrait (38-10 AC-4) identiques. Tests : rapport CSV d'un
   compte EUR au 31.12 avec solde → `gap` de la liste calculé comme pour un PDF (C-38-10-4) ; K9 retient un CSV comme
   tout document à solde (C-38-10-5) : CSV au 31.12 en écart → K9 `attention` avec `documentId` du CSV ; un PDF au 30.11
   et un CSV au 31.12 → le CSV compte (dernier `periodTo`).
5. **AC-5 — Audit, sauvegarde, export (NFR-40-5-2, C-40-6).** Trace `bank_account_document.attached` (38-10) : son
   `details` porte `{ kind, mimeType }` en plus de ce que 38-10 y met (C-40-6 ; aucun libellé d'action neuf, aucune
   route neuve : `LIB_ROUTES` delta 0) — test de route : après l'ajout d'un CSV puis d'un PDF, la ligne d'audit porte
   `kind: "provider_report"`, `mimeType: "text/csv"`, puis `statement`, `application/pdf`, et aucun extrait du fichier. Sauvegarde : rien à changer (38-10 lit `storage_path`, `sha256`, quel que soit le
   type) — test : export complet puis import dans une instance neuve (`admin_full_import_e2e` de 38-10 étendu d'un CSV)
   → le CSV se télécharge, empreinte identique. Export CSV de souveraineté : colonnes inchangées (`kind` exporté dit le
   genre ; `mime_type` reste à `COLONNES_HORS_EXPORT`, motif de 38-10) — un test le constate. **MCP** : aucun outil
   (E23, G17 ; 38-10 AC-10 : les relevés joints ne sont pas exposés).
6. **AC-6 — Écran (FR-40-5-4, NFR-40-5-3).** `BankAccountDocumentsDialog.svelte` (38-10 AC-9) : le titre du formulaire
   devient « Joindre un relevé ou un rapport » ; le champ fichier accepte `.pdf,.csv,application/pdf,text/csv` ; contrôle
   côté écran : PDF ou `.csv` (le serveur décide) ; la liste montre le type (« PDF » / « Rapport CSV », lu de `mimeType`
   et `kind` ; valeur inconnue → « Pièce ») ; aide sous le champ, valable pour tout compte (Écart 3) : « Un relevé PDF,
   ou le rapport CSV d'un prestataire qui n'en fournit pas (Stripe : le rapport de solde, *Balance*, téléchargé du
   tableau de bord ; son solde de fin de période est le solde de clôture à saisir). Un rapport sans solde de fin, comme
   le rapprochement des versements, se joint sans solde. » — le rapport de rapprochement des versements ne porte pas le
   solde Stripe de fin de période (<https://docs.stripe.com/reports/report-types/payout-reconciliation> : résumés par
   `reporting_category`, `count, currency, fee, gross, net` ; le solde de fin est l'objet d'`ending_balance_reconciliation`
   et du rapport de solde) : y saisir un solde ferait un écart K9 fictif ; les refus `BANK_DOCUMENT_UNSUPPORTED_TYPE` (un libellé par `reason`) et `BANK_DOCUMENT_NOT_PDF`
   (message élargi : « Le fichier n'est ni un PDF ni un CSV ») traduits ; API cliente et types (`bank-accounts.api.ts`,
   champ `mimeType`) ; `+page.svelte` des comptes bancaires **non touché**. **Préfixes** (`lint-i18n-ownership.js:177-186`) :
   les clés neuves du dialogue (titre, aide, type affiché, un libellé par `reason`) sont au préfixe **`bank-accounts-`**
   (`features/bank-accounts/`, comme 38-10 AC-9) ; le message serveur de `BANK_DOCUMENT_UNSUPPORTED_TYPE` suit le patron
   des messages d'erreur du serveur (`t("bank-import-errors-…", …)`, `errors.rs:2553-2558`), et celui de
   `BANK_DOCUMENT_NOT_PDF` garde la clé que 38-10 lui donne ; jamais une entrée à `KNOWN_VIOLATIONS`. Clés aux **quatre** locales
   (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), en toutes lettres ; message de `BANK_DOCUMENT_NOT_PDF`
   mis à jour aux quatre (le code ne change pas). **Attentes de 38-10 changées exprès** (NFR-40-5-3) : les Vitest de
   38-10 AC-9 qui tiennent « PDF seul côté écran », le titre « Joindre un relevé », l'attribut `accept` et le texte de
   `BANK_DOCUMENT_NOT_PDF` (et `i18n-keys.test.ts` s'il cite la valeur) sont modifiés, chacun cité au Change Log avec
   l'attente d'avant et d'après ; aucune autre attente de 38-10 ne bouge. Vitest : formulaire avec CSV, libellés des trois motifs, type affiché,
   valeur inconnue. **E2E** (`frontend/tests/e2e/bank-account-documents.spec.ts` de 38-10, cas ajouté) : joindre un CSV
   synthétique avec un solde qui diffère des livres → écart affiché, type « Rapport CSV » → télécharger (contenu
   identique) → retirer ; joindre un `.csv` Latin-1 → refus libellé.
7. **AC-7 — Recette visuelle (NFR-40-5-3).** Captures (thème clair, base de démonstration, `kesh-gate-logs/`) : formulaire
   avec un CSV choisi, liste avec un PDF et un CSV, refus `not_utf8` ; jointes au Dev Agent Record et au prompt de revue.
8. **AC-8 — Documentation.** `docs/api-external.md` : la route `POST …/documents` admet un CSV (C-40-5-1, C-40-5-2), le
   code `BANK_DOCUMENT_UNSUPPORTED_TYPE` et ses motifs, `mimeType` de la liste, `Content-Type` du téléchargement. Manuel
   utilisateur : à 40-6 (rapports Stripe à joindre), avec le texte de l'aide de l'AC-6 ; si 40-6 n'est pas livrée dans la
   même PR, la sous-section *Joindre un relevé* de 38-10 gagne ici une phrase (« un rapport CSV s'y joint de même »), PDF
   régénéré.

## Tâches

- [ ] T1 — `classify_document`, code d'erreur, `mime_for_ext("csv")`, tests unitaires (AC-1).
- [ ] T2 — Route d'ajout : branche CSV, genre, rangement ; utilitaire multipart de 38-10 étendu du nom d'origine non
  tronqué ; tests de route (AC-1, AC-2).
- [ ] T3 — Liste (`mimeType`) et téléchargement par table fermée (AC-3) ; K9 et écart avec un CSV (AC-4).
- [ ] T4 — Audit `details`, sauvegarde et export constatés (AC-5).
- [ ] T5 — Dialogue, i18n, Vitest, E2E, captures (AC-6, AC-7) ; `api-external.md` (AC-8).
- [ ] T6 — Gate complet (dépôt `kesh-db` touché) ; E2E complet au dernier commit de code.

## Dev Notes

- **Sécurité** : le CSV n'est jamais servi `inline` ni en `text/html` ; le nom d'origine n'entre jamais dans un chemin
  (`store_document`) ; aucun extrait du fichier dans une erreur, un journal ou l'audit.
- **Pas de parseur** : `kesh-import` (CSV bancaire, profils) ne lit pas ces rapports ; le solde est saisi (G13, B20).
  Le rapport par l'API Stripe (`balance.summary`) est tracé pour plus tard (C-40-7).
- **Rapports Stripe** : <https://docs.stripe.com/reports/report-types/balance>,
  <https://docs.stripe.com/reports/report-types/payout-reconciliation> (CSV, téléchargeables du tableau de bord) ; le
  solde de clôture d'un rapport de solde arrête le solde Stripe à la fin de la période, versements partis compris — d'où
  le détail « versements en route » de K9 (40-4 AC-4).
- **Références** : epic G13, C-40-2 (d), C-40-6, C-40-7 ; 38-10 AC-1 à AC-9, C-38-10-1 à C-38-10-7 ; 40-4 AC-4.

## Questions ouvertes

- Aucune propre à la story. (Le CSV admis sur tout compte bancaire, Écart 3, se contesterait par une garde « compte
  prestataire seulement » sans migration.)

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — rédaction (Opus) : 8 AC ; 4 FR (3 de l'epic + 1 ajoutée), 3 NFR (2 + 1) ; aucune migration ; un code
  d'erreur neuf (`BANK_DOCUMENT_UNSUPPORTED_TYPE`) ; écarts : `BANK_DOCUMENT_NOT_PDF` gardé pour un fichier ni PDF ni
  annoncé CSV, genre déduit du type, CSV admis sur tout compte. Relu au code : `document_storage.rs:54-131`, `:421-426`,
  `inbox_import.rs:98`, `:576`, `:733`, `:897-905`, `imported_supplier_invoices.rs:733-784`,
  `kesh-import/src/csv/encoding.rs:48-100` ; fiche 38-10 (AC-1 à AC-9, C-38-10-*).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : P1-1 fichier vide annoncé CSV, ordre des contrôles non fixé (C-40-5-1 étape 0 ajoutée) ; P1-2
  annonce par le seul type de partie ou par un nom en majuscules sans test (AC-1 étendu). LOW : P1-3 compatibilité de la
  restauration avec une pièce `.csv` non vérifiée (relu : `export.rs:177-185`, ajouté aux prémisses) ; P1-4 en-tête à une
  seule colonne refusé, limite non dite (test et manuel). Prémisses relues et justes : `document_storage.rs:54-131`,
  `:421-426`, `inbox_import.rs:98`, `imported_supplier_invoices.rs:768-784`, `encoding.rs:48-100`, 38-10 AC-2/AC-3/AC-7,
  40-4 AC-4.
- 2026-11-15 — validation P2 (Opus) : 4 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW) ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : P2-1 NFR-40-5-3 et AC-1 promettaient les tests de 38-10 « sans retouche » (G18) alors que l'AC-6
  change exprès des attentes que 38-10 AC-9 fixe (Vitest « PDF seul côté écran », titre, `accept`, texte de
  `BANK_DOCUMENT_NOT_PDF`) ; G18 vise PayPal et Wise — portée réduite aux tests serveur, attentes changées inventoriées
  au Change Log ; P2-2 l'aide disait de saisir le solde de clôture du rapport de rapprochement des versements, qui ne
  porte pas de solde Stripe de fin de période (docs.stripe.com, payout-reconciliation : résumés par
  `reporting_category`) — K9 fictif ; aide réécrite (rapport de solde, *Balance* ; rapprochement joint sans solde),
  valable pour tout compte (Écart 3). LOW : P2-3 annonce lue sur le nom tronqué par `parse_multipart`
  (`bank_imports.rs:484-485`, repli `upload.xml`) et type de partie avec paramètres (`text/csv; charset=utf-8`) —
  C-40-5-1 précisé, deux tests ; P2-4 rédaction : `store_document` nomme le fichier à `:117` (non `:116`) ; « caractère
  de contrôle » = `char::is_control` ; longueur de la première ligne mesurée après retrait du BOM. Relu au code :
  `document_storage.rs:54-131`, `:421-426`, `inbox_import.rs:98`, `:576`, `:733`, `bank_imports.rs:448-486`, `:614`,
  `imported_supplier_invoices.rs:768-784`, `admin_backup/export.rs:175-185`, `encoding.rs:48-91`,
  `tests/support/pieces.rs:113` (seul autre appelant de `mime_for_ext` : extension passée par le test, aucun `csv`) ;
  fiches 38-10 (AC-2, AC-3, AC-7, AC-9), 40-4 AC-4, 40-6 AC-8 ; epic G13, C-40-2 (d), C-40-6. Trend : P1 = 4 (2 MEDIUM)
  → P2 = 4 (2 MEDIUM, distincts, d'origine et non de remédiation : pas de signal de découpage) ; une passe de plus est
  due. Axes non exercés : code de 38-10 (non écrit) ; PDF du manuel (non régénéré) ; encodage réel d'un CSV du tableau
  de bord Stripe (non documenté à la page lue, à constater à la recette 40-0).
- 2026-11-15 — validation P3 (Sonnet) : 4 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW) ; remédiés dans la fiche, aucun
  inter-fiches (le point touchant 38-10 est un champ additif de l'utilitaire multipart, porté par T2). MEDIUM : P3-1 la
  fiche exigeait le nom d'origine non tronqué alors que 38-10 reprend `parse_multipart`, qui tronque (`bank_imports.rs:485`)
  — l'utilitaire rend désormais aussi le nom d'origine, test de route pour le nom > 255 octets ; P3-2 AC-6 ne nommait aucun
  préfixe i18n (même défaut que 38-9a/38-10, `lint-i18n-ownership.js:177-186`) — `bank-accounts-` et patron serveur. LOW :
  P3-3 partie `text/csv` sans nom rangée sous `upload.xml` (téléchargement en `.xml`) — `rapport.csv`, test ; P3-4 le détail
  d'audit `{ kind, mimeType }` n'avait aucun test. Relu au code : `document_storage.rs:54-131`, `:421-426`,
  `inbox_import.rs:98`, `:576`, `:733`, `bank_imports.rs:415-486`, `admin_backup/export.rs:175-185`, `errors.rs:2553`,
  `encoding.rs:48-64` ; 38-10 (AC-2, AC-3, AC-8, AC-9, Dev Notes), 40-6 AC-8, epic G13, C-40-6 — concordants. Axes non
  exercés : code de 38-10 (non écrit) ; PDF du manuel ; encodage réel d'un CSV du tableau de bord Stripe (recette 40-0).
  Trend : P2 = 4 (2 MEDIUM) → P3 = 4 (2 MEDIUM, d'origine) ; une passe de plus est due.
