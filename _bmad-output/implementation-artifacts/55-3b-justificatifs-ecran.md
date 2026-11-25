# Story 55-3b : ouvrir les justificatifs d'une écriture depuis le Journal (et le Grand livre)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 à D11. Issue : #À-VENIR (point c, comme la
55-3a). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration.** **Suit la 55-3a**
(booléen `hasDocuments`, route `GET /journal-entries/{id}/documents`), **la 55-2** (mêmes vues) **et la 55-1a** (URL, mémoire des rapports).

*Découpée de la 55-3 à la validation P1 (V1-5).*

*⚠️ **Récrite le 2026-11-25 sur décision de l'utilisateur** : **une seule icône PDF par ligne d'écriture**, quel que soit
le nombre de justificatifs — ni compteur, ni menu ; au clic, **la visionneuse existante** (`DocumentPreview`, dont le
cadre affiche le PDF dans le lecteur du navigateur, pages à gauche), avec **la liste des documents de l'écriture en haut** :
un clic dans la liste change le document affiché ; avec un seul justificatif, la liste se réduit à son nom. La liste se
charge au clic (route de la 55-3a). Validation reprise à P1 sur cette version.*

## Story

En tant que comptable qui relit son journal,
je veux voir d'un clic, depuis la ligne d'une écriture, les PDF qui la justifient,
afin de contrôler une écriture sans ouvrir l'écriture puis la facture.

## Modules touchés (règle de splitting préventif)

Cinq : `frontend/src/lib/features/reports` (composant neuf, deux vues, lecteurs), `frontend/src/lib/features/imported-supplier-invoices`
(une propriété facultative de `DocumentPreview`), `kesh-i18n` (quatre locales), `docs/manual/fr/user-manual.tex`,
`frontend/tests/e2e`. (S'y ajoute le compteur `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts`, recompté
par la garde : un fichier commun, non un module.) Les pages des fiches de facture et d'avoir **ne sont pas touchées** (voir C-55-3b-2).

## Ce qui se lit au code (`origin/main` `1e23edb55`)

- Le contrat de la 55-3a : `hasDocuments?: boolean` sur `JournalEntryRow` et `LedgerLine` ; `GET
  /api/v1/journal-entries/{id}/documents` → `EntryDocument[]`, `EntryDocument { kind: 'invoice' | 'creditNote' |
  'supplierDocument'; id: number; number: string | null; fileName?: string }`.
- **La visionneuse** : la seule de Kesh est `DocumentPreview` (`grep -rln "iframe\|<object\|<embed"` sous
  `frontend/src` ne rend qu'elle ; aucune bibliothèque PDF — ni `pdfjs` ni vignettes — dans `package.json`) : un `<iframe>`
  d'une URL `blob:` où le **lecteur PDF du navigateur** affiche le document, avec ses pages à gauche. Elle sert déjà à
  cinq endroits : `AttachDocumentPicker.svelte:198`, `supplier-invoices/new/+page.svelte:236`,
  `supplier-invoices/import/+page.svelte:605`, `supplier-invoices/[id]/+page.svelte:1086`,
  `journal-entries/[id]/+page.svelte:618`. Pour recevoir une liste, il ne lui
  faut rien : elle affiche **un** document, et une nouvelle fonction `load` le remplace (son `$effect` relit, et révoque
  l'URL `blob:` précédente). La liste vit donc **au-dessus d'elle**, dans un composant neuf qui choisit le document et lui
  passe `load` et `fileName` — `DocumentPreview` ne change pas pour ses cinq appelants (une seule propriété facultative,
  `errorMessage`, AC-4).
- `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` : `Props { load: () => Promise<Blob>;
  fileName: string }` (`:23-28`) ; un `$effect` relit le document **à chaque nouvelle fonction `load`** ; l'erreur de `load`
  est rendue par deux textes fixes — `SOURCE_DOCUMENT_GONE` → « Le document n'a pas été restauré », tout le reste → « Le
  document n'a pas pu être affiché » (`:56-61`, `:81-83`) ; PDF en `<iframe>` d'une URL `blob:`, image en `<img>`, autre type :
  lien « Télécharger » ; CSP `blob:` autorisée (`kesh-api/src/middleware/csp.rs:37-38`).
- `frontend/src/lib/shared/utils/pdf-error.ts` : `pdfErrorMessage` rend le message du serveur pour
  `INVOICE_TOO_MANY_LINES_FOR_PDF` et `INVOICE_PDF_GONE` (`:36`), un générique sinon (`:41`) — propre aux PDF de facture.
- Lectures en `Blob` d'aujourd'hui : `fetchSupplierInvoiceSourceDocument(id)` (`imported-supplier-invoices.api.ts:120`) ;
  facture et avoir, écrites **dans les pages** : `apiClient.getBlob('/api/v1/invoices/${id}/pdf')`
  (`invoices/[id]/+page.svelte:969`, nom `facture-${number ?? id}.pdf`), `…/credit-notes/${id}/pdf`
  (`credit-notes/[id]/+page.svelte:94`).
- Le détail d'une écriture montre déjà le document d'une facture fournisseur (`journal-entries/[id]/+page.svelte:589-620`).
- Manuel : § 11.6.1 « Le PDF émis est figé » (`sec:pdf-fige`, `user-manual.tex:1635-1645`) — produit « au premier
  téléchargement ou au premier envoi par e-mail », « qui que ce soit qui télécharge le premier — y compris un utilisateur en
  consultation » ; § *Grand livre* (`:4184`, « avec sa date, sa pièce, son journal », `:4193`) ; § *Journal des écritures*
  (`:4316-4318`, deux lignes).

- Composants : `frontend/src/lib/components/ui/dialog` (bits-ui, employé par `ManualMatchModal`) ; `dialog-content.svelte`
  porte `showCloseButton = true` par défaut (`:16`, test `:37`), avec un `<span class="sr-only">Close</span>` **anglais en
  dur** (`:42`), et la classe `sm:max-w-sm` (`:31`) ; précédent de dialogue large qui défile : `max-h-[85vh] overflow-y-auto`
  (`LinkSettlementModal.svelte:201`). `DocumentPreview` : `data-testid` `document-preview`, `-loading`, `-gone`, `-error`,
  `-image`, `-frame`, `-download` (`:71-103`) ; son `<iframe>` fait `h-[75vh]`.
- `GeneralLedgerView.svelte` : colonnes déclarées dans `COLUMNS` (`:44-54`), constantes de module qui en découlent
  (`COLUMN_COUNT` `:58`, `BALANCE_LABEL_SPAN`, `AFTER_BALANCE`, `MOVEMENTS_LABEL_SPAN`, `AFTER_MOVEMENTS` `:61-67`, `colspan` par
  `indexOf` `:300-301`), libellés par un `switch` exhaustif (`columnLabel`, `:70-91`), commentaire « Lettrage est la dernière … »
  (`:36-39`) ; `GeneralLedgerView.test.ts` : « 15-1c-ii test 3 — « Lettrage » est la DERNIÈRE colonne, après « Solde
  progressif » » (`:241-248`) attend **9** en-têtes, et « chaque rangée du corps et du pied couvre exactement les en-têtes »
  (`:251-262`), « le solde tombe sous Solde progressif » (`:274`), « le total des mouvements tombe sous Débit et Crédit »
  (`:293`) ; `piece()` (`:151-155`) rend le numéro d'écriture, préfixé de l'exercice **quand la période en traverse un**.
- Le numéro d'écriture repart à 1 à chaque exercice ; le journal est rendu pour **un** exercice (`fiscalYearId`), le grand
  livre peut en traverser plusieurs.
- Une écriture à deux justificatifs s'obtient **par l'API** : `POST /api/v1/reconciliation/accept` avec une proposition
  `invoice_group` (règlement client groupé, `accept_one_invoice_group`, `routes/reconciliation_group.rs:169` ; l'écriture se lit dans
  `accepted[0].journalEntryId`, `routes/reconciliation.rs:566`) ; patron et aides dans
  `frontend/tests/e2e/reconciliation-group.spec.ts` et `tests/e2e/helpers/api-fixtures.ts` ; `InvoiceResponse.journalEntryId`
  désigne l'écriture de vente.
- Gardes i18n : `i18n-keys` (`sitesTotal`), `i18n-libelle-en-dur`, `i18n-un-repli-par-cle`, `lint-i18n-ownership` (une clé
  d'un autre domaine, p. ex. `journal-entries-document-preview-close`, est refusée depuis `features/reports`) ; `i18nMsg`
  ne gère pas les pluriels Fluent : variables simples seulement.

## Critères d'acceptation

1. **AC-1 — Journal : une icône par écriture (critère principal).** Une écriture qui porte `hasDocuments` montre, sur sa
   ligne d'en-tête (`#numéro`, date), **une** icône PDF — un `<button>` (`data-testid="journal-entry-documents-<entryId>"`,
   `aria-label` « Justificatifs de l'écriture n° N »), **sans compteur ni menu**, qu'elle ait un justificatif ou plusieurs.
   Une écriture sans `hasDocuments` n'en a pas. Aucune requête n'est faite avant le clic.
2. **AC-2 — La visionneuse et sa liste.** Au clic, un dialogue s'ouvre ; il lit `GET /api/v1/journal-entries/{id}/documents`
   (une requête, un indicateur de chargement), puis montre **en haut** la liste des justificatifs (genre, numéro, nom du
   fichier d'un document fournisseur ; l'élément affiché marqué `aria-current`) et **dessous** `DocumentPreview` sur le
   **premier** de la liste. Un clic sur un autre élément change le document affiché, sans fermer le dialogue. Un seul
   justificatif : la liste se réduit à son nom (aucun choix). Liste vide (justificatif disparu depuis le rapport) : le
   message `reports-nav-documents-none` ; erreur de la route : `errorMessageOf`. La liste est **relue à chaque ouverture** ;
   une réponse d'une ouverture antérieure (fermé puis rouvert en vol) est ignorée (compteur d'ouverture), test dédié.
3. **AC-3 — Les trois genres et leurs lecteurs.** `fetchInvoicePdf(id)` → `GET /api/v1/invoices/{id}/pdf` ;
   `fetchCreditNotePdf(id)` → `GET /api/v1/credit-notes/{id}/pdf` (les deux dans `reports.api.ts`, par `apiClient.getBlob`) ;
   document fournisseur → `fetchSupplierInvoiceSourceDocument(id)`. Noms passés à `DocumentPreview` :
   `facture-<numéro>.pdf`, `avoir-<numéro>.pdf`, à défaut du numéro `facture-<id>.pdf`, `avoir-<id>.pdf` ; le `fileName` du
   document fournisseur, à défaut `justificatif-<id>`. Dans la liste, un numéro nul s'affiche par l'identifiant. Un `Blob`
   vide : le lecteur d'`EntryDocuments` **rejette** par une erreur « vide » dédiée (`DocumentPreview` ne contrôle pas la
   taille, `:50-61`), rendue par `reports-nav-documents-empty` (« Le document reçu est vide. »), pour les trois genres —
   clé neuve parce que `lint-i18n-ownership` refuse `invoice-pdf-error-empty` depuis `features/reports`.
4. **AC-4 — Erreurs, droits.** Les trois routes d'aujourd'hui, sans route neuve, leurs droits inchangés (un rôle
   *Consultation* voit le bouton et ouvre le justificatif). `DocumentPreview` gagne une propriété **facultative**
   `errorMessage?: (err: unknown) => string | undefined`, lue **dans le `catch` seulement** (pas dans l'`$effect` qui lit
   `load` : une fonction neuve ne relance pas la lecture) : une chaîne rend l'état `document-preview-error` avec ce texte ;
   `undefined`, ou la propriété absente, le comportement d'aujourd'hui (`gone` pour `SOURCE_DOCUMENT_GONE`, texte fixe
   sinon). Les tests existants de
   `DocumentPreview.test.ts` passent **sans modification** ; des tests y sont **ajoutés** (propriété présente, absente).
   `EntryDocuments` la passe pour les trois genres : erreur « vide » → `reports-nav-documents-empty` ; sinon facture et
   avoir → `pdfErrorMessage(err)` (`410 INVOICE_PDF_GONE` nomme le fichier à restaurer), document fournisseur → `undefined`.
   Le journal reste affiché. Rôle *Consultation* : constat par construction, écrit au Change Log — aucun contrôle de rôle
   dans `EntryDocuments` (`grep -n "role" EntryDocuments.svelte` vide) et les trois routes sous « tout rôle authentifié »
   (`lib.rs:1262-1293`).
5. **AC-5 — Figement et lecture unique.** Ouvrir la facture d'une écriture la **fige** si elle ne l'était pas (route
   d'aujourd'hui, `Usage::Download`), y compris pour un rôle *Consultation* : dit au manuel (AC-9). Le justificatif choisi
   est tenu par **deux primitives** (`kind`, `id`), et la fonction `load` est dérivée de ces seules primitives : remplacer
   le rapport ou la liste relue par des objets égaux ne relit pas le document. Changer d'élément dans la liste ne relit que le document
   choisi. Le cas réel du tableau remplacé : Retour ou Suivant vers le **même
   onglet**, dont le rapport est repris de la mémoire (55-1a, C-55-1-2) ou redemandé, l'écriture y étant encore ; *Générer*
   n'est pas atteignable dialogue ouvert (modal) et démonte la vue (`{#if loading}`). La décision de figer à l'affichage est
   soumise au mainteneur (Q7 du fichier d'epic) ; refusée, la 55-3a ne rend que les factures déjà figées (un filtre serveur,
   aucun champ de plus au contrat).
6. **AC-6 — Grand livre.** Une colonne `documents` (`COLUMNS`, après `description`, en-tête `reports-nav-ledger-column-documents`
   « Justificatifs », rendu étroit), **présente seulement si au moins une ligne du rapport porte `hasDocuments`** (condition
   sur le rapport, non sur la section : toutes les sections ont alors le même nombre d'en-têtes) ; les constantes qui
   découlent de `COLUMNS` (`COLUMN_COUNT`, `BALANCE_LABEL_SPAN`, `AFTER_BALANCE`, `MOVEMENTS_LABEL_SPAN`, `AFTER_MOVEMENTS`,
   `indexOf`) deviennent dérivées de la liste effective. Chaque ligne qui porte `hasDocuments` porte la même icône
   (`data-testid="ledger-entry-documents-<lineId>"`, unique), `aria-label` avec le numéro de `piece()` (préfixé quand la
   période traverse un exercice), même dialogue. Le commentaire `:36-39` est corrigé. Sans justificatif, les tests existants
   restent verts sans modification ; avec une ligne à justificatif, 10 en-têtes, et les trois tests « chaque rangée … couvre
   exactement les en-têtes » (section en devise comprise, seconde ligne `ledger-fx-*`), « le solde tombe sous Solde
   progressif », « le total des mouvements tombe sous Débit et Crédit » sont **rejoués** ; deux sections dont une seule a un
   justificatif : même nombre d'en-têtes. *Option* : celle de la 55-3a.
7. **AC-7 — Dialogue.** `components/ui/dialog` avec `showCloseButton={false}`, un bouton « Fermer » propre
   (`reports-nav-documents-close`), une largeur qui tient l'aperçu (`sm:max-w-4xl`) et un contenu qui défile ; titre
   `reports-nav-documents-title` (« Justificatifs de l'écriture n° { $number } ») ; *Échap* (focus dans le dialogue, hors du cadre
   du PDF, qui est un autre document) et « Fermer » — le chemin sûr — ferment, focus rendu au bouton d'origine. Ouvrir le dialogue n'appelle pas `goto` (55-1a, D2 : l'URL ne change pas). Une navigation qui
   change le rapport affiché (Retour vers un autre onglet) démonte la vue et ferme le dialogue. Un rapport repris de la
   mémoire (55-1a, D3) peut porter un justificatif devenu introuvable : l'erreur s'affiche dans le dialogue (AC-4).
8. **AC-8 — Textes.** Clés neuves du bloc `reports-nav-*`, quatre locales : `reports-nav-documents-button` (« Justificatifs
   de l'écriture n° { $number } »), `reports-nav-documents-title`, `reports-nav-documents-kind-invoice` (« Facture »),
   `-kind-credit-note` (« Avoir »), `-kind-supplier-document` (« Document fournisseur »), `reports-nav-documents-list`
   (libellé de la liste), `reports-nav-documents-loading`, `reports-nav-documents-none`, `reports-nav-documents-close`
   (« Fermer »), `reports-nav-documents-empty`, `reports-nav-ledger-column-documents` ; constat : `i18n-keys` (`sitesTotal` recompté par
   la garde, parité des quatre locales), `i18n-libelle-en-dur`, `i18n-un-repli-par-cle`, `npm run lint-i18n-ownership`
   verts.
9. **AC-9 — Manuel.** `docs/manual/fr/user-manual.tex` (lignes à `1e23edb55`) : § *Journal des écritures* (`:4316`) — le
   bouton, les trois genres, le règlement qui montre la facture réglée, la liste ; § *Grand livre*, l'énumération des
   colonnes (`:4193-4194`) gagne « ses justificatifs » ; § 11.6.1 (`sec:pdf-fige`, `:1635-1637`) — « au premier
   téléchargement, au premier envoi, ou au premier affichage depuis les rapports », renvoi `\S\ref{sec:pdf-fige}` depuis
   les deux paragraphes des rapports ; le chemin « Mensuel → Rapports → Journal » (`:4318`) contrôlé contre l'onglet réel
   (« Journaux »). **Phrases du figement** — par une **règle**, non par une liste de lignes (validation P4 de la version
   récrite : chaque liste se révélait incomplète) : **toute phrase du manuel qui attribue le figement d'un PDF de facture au
   seul téléchargement ou au seul envoi**, ou qui dit qu'une facture « jamais téléchargée ni envoyée » n'est pas figée, est
   corrigée pour y ajouter l'affichage depuis les rapports, avec renvoi au § 11.6.1. Méthode, au T4 et sur la base de
   développement (les numéros glissent de −6 entre `1e23edb55` et la PR 46) : `grep -nE
   "t[ée]l[ée]charg|envoy|envoi|ouvert|rendu|fig[ée]|refig" docs/manual/fr/user-manual.tex` (balises `\textbf{}` comprises),
   chaque occurrence **jugée** dans une table du Change Log — corrigée, ou laissée avec sa raison —, ligne lue citée.
   Repères à `1e23edb55`, non limitatifs : `:1502`, `:1513`, `:1553`, `:1633`, `:1637`, `:1641`-`:1645`, `:1650`, `:1674`,
   `:1725`, `:2058` (laissée : audit), `:4909`. Formulation : § 11.6.1 (`:1637`) « au premier téléchargement, au premier
   envoi, ou au premier affichage depuis les rapports » ; ailleurs, l'ajout de « ou affichage » à la tournure existante.
   `.tex` seul ; le PDF se régénère à la release. Captures du
   manuel montrant le grand livre (p. ex. `rd-grand-livre-usd.png`) : périmées si la colonne y paraît, à refaire à la
   release (dit au Change Log).
10. **AC-10 — Accessibilité.** Bouton au clavier ; dialogue `role="dialog"`, `aria-modal`, étiqueté par son titre ; axe
    à zéro violation, dialogue ouvert : `new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).include('[role="dialog"]')`
    (modèle : les specs axe existantes de `tests/e2e/`).

## Tests à écrire, et la preuve du rouge

- **Vitest** `EntryDocuments.test.ts` (neuf, l'icône et le dialogue) et `EntryDocumentsViewer.test.ts` (neuf, la liste
  au-dessus de `DocumentPreview`) : aucune icône sans `hasDocuments`, une seule avec (que la liste compte 1 ou 3
  éléments) ; aucune requête avant le clic, **une** au clic ; premier élément affiché ; clic sur un autre élément → son
  lecteur appelé, une lecture ; un seul justificatif → son nom, sans choix ; liste vide ; erreur de la route ; les trois
  genres (lecteur appelé, nom de fichier, replis de l'AC-3) ; erreur `410` → texte de `pdfErrorMessage` ; `Blob` vide
  **pour chacun des trois genres** ; **lecture unique** : dialogue ouvert, rapport remplacé par un objet égal → toujours
  **une** lecture (mutation déclarée au journal : sélection tenue par l'objet → deux lectures, test rouge) ; ouverture sans
  appel de `goto` (mock de `$app/navigation`) ; **fermé puis rouvert pendant la lecture de la liste** — deux promesses
  contrôlées, la première résolue **après** la seconde, avec des listes de noms **différents** ; l'assertion porte sur la
  liste affichée : celle de la seconde ouverture — (mutation : compteur d'ouverture retiré → la liste ancienne s'affiche, test rouge).
- **Vitest** (focus) : fermeture (Échap, « Fermer ») → focus sur l'icône qui a ouvert (deux icônes, la seconde ouvre ;
  mutation : focus rendu à la première → test rouge).
- **Vitest** `reports.api.test.ts` : `fetchEntryDocuments(entryId)`, `fetchInvoicePdf`, `fetchCreditNotePdf` (URL exactes).
- **Vitest** `DocumentPreview.test.ts` : tests ajoutés (propriété présente rendant une chaîne, y compris pour
  `SOURCE_DOCUMENT_GONE` ; rendant `undefined` ; absente ; nouveau rendu avec une autre fonction `errorMessage` → **une**
  lecture).
- **Vitest** `JournalReportView.test.ts` et `GeneralLedgerView.test.ts` : icône présente avec `hasDocuments`, absente sans ;
  colonne présente ou absente, et les tests de l'AC-6 rejoués avec la colonne.
- Rouges sans le correctif, journal `kesh-gate-logs/s55-3b/`.
- **E2E** `frontend/tests/e2e/reports-entry-documents.spec.ts` (neuf, déclaré au tableau des fichiers communs), **cible :
  le Journal** : créer et valider une facture par l'API, ouvrir *Rapports → Journaux* sur la période, cliquer le bouton de
  l'écriture de vente (`journalEntryId` de la facture) → la liste à un élément (le numéro de la facture) et
  `document-preview-frame` visible ; *Échap* ferme, focus rendu ;
  axe à zéro violation dialogue ouvert. **Deux justificatifs** : deux factures validées, puis un règlement groupé par
  `POST /api/v1/reconciliation/accept` (`invoice_group`, patron `reconciliation-group.spec.ts:796-800`, écriture lue dans
  `accepted[0].journalEntryId`) → **une** icône sur l'écriture de règlement ; le dialogue montre la liste de deux, la première facture
  affichée ; un clic sur la seconde change le document (la requête de son PDF part) ; les aides
  (`createAndValidateInvoiceViaApi`, `createManualCreditStatementViaApi` d'`api-fixtures.ts`, et des aides neuves **dans le
  spec neuf** pour le règlement groupé — le spec `reconciliation-group.spec.ts` de l'Epic 43 n'est pas touché, ni
  `api-fixtures.ts` modifié) ; nettoyage en `finally` comme le patron (groupe annulé, import supprimé).
  **Retour** : partir du *Bilan*, cliquer *Journaux* (entrée d'historique, 55-1a D2), générer, ouvrir le dialogue
  (`[role="dialog"]` visible), puis `page.goBack()` **dialogue ouvert** → URL `tab=balance-sheet`, `[role="dialog"]` en
  nombre 0, et un clic sur un onglet réussit (aucun overlay ni verrou de défilement resté).
  Grand livre : un test court (colonne, bouton, aperçu). Le test du Journal rougit sans le correctif (aucun bouton).

## Recette visuelle

Captures (thème clair, `kesh-gate-logs/s55-3b/`) : journal avec ses icônes ; visionneuse à un justificatif (liste réduite
au nom) ; visionneuse à deux justificatifs, chacun affiché ; erreur `410` (obtenue sur une installation jetable en retirant de `KESH_DOCUMENTS_DIR` le fichier `<empreinte>.pdf`
d'une facture figée, ou par `page.route` rendant `410 INVOICE_PDF_GONE`) ; grand livre avec la colonne.

## Tâches

- [ ] **T0** — Base `epic/55-navigation` avec les 55-1a, 55-1b, 55-2 et 55-3a ; lignes du manuel sur la base PR 46 ; et les
      points renvoyés par la validation, chacun tranché au Change Log, ligne lue citée :
      - hauteur de l'aperçu dans le dialogue (titre, liste, cadre et « Fermer » tiennent dans `85vh`, sans double
        défilement) ;
      - fichier d'epic : `helpers/api-fixtures.ts` non touché ;
      - (tranché : **une** instance du dialogue par vue — `EntryDocuments.svelte` est rendu une fois par vue, avec l'écriture
        ouverte en état ; l'icône par ligne n'est qu'un bouton qui l'ouvre ; T2 en conséquence) ;
      - nature de l'erreur « vide » (classe dédiée reconnue par `errorMessage` avant `pdfErrorMessage`) ; une clé pour le
        libellé de l'icône et le titre s'ils portent le même texte ;
      - E2E *Consultation* exigé (patron `reports.spec.ts:441`) ;
      - LOW de la version précédente (V3-1 à V3-7).
- [ ] **T1** — Vitest rouges.
- [ ] **T2** — `DocumentPreview` (`errorMessage`, rien d'autre) ; lecteurs dans `reports.api.ts`
      (`fetchEntryDocuments`, `fetchInvoicePdf`, `fetchCreditNotePdf`) ; `EntryDocumentsViewer.svelte` (la liste en haut,
      `DocumentPreview` dessous, sélection par primitives) ; `EntryDocuments.svelte` (le dialogue `ui/dialog`, **une
      instance par vue**, et la lecture de la liste au clic) ; l'icône de chaque ligne est un bouton qui lui passe
      l'écriture **et son élément** ; l'instance rend le focus à cet élément à la fermeture.
- [ ] **T3** — Vues du journal et du grand livre (colonne conditionnelle) ; types `reports.types.ts`.
- [ ] **T4** — i18n, manuel, E2E, captures.
- [ ] **T5** — Gates (fichier d'epic, § *Gates*).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E de la story et
des Rapports ; suite complète selon le § *Gates* du fichier d'epic (Q6).

## Note de version

*Rapports → Journaux* et *Grand livre* : une icône PDF sur une écriture ouvre ses justificatifs dans la visionneuse — PDF
de la facture, de l'avoir, document de la facture fournisseur ; pour un règlement, la ou les factures réglées, à choisir
dans la liste en haut. Ouvrir ainsi une facture dont le PDF
n'était pas encore produit le fige, comme un téléchargement. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-3b-1** — `DocumentPreview` réutilisé (une visionneuse dans Kesh), étendu d'une propriété facultative, plutôt
  qu'une seconde visionneuse.
- **C-55-3b-2** — Lecteurs de la facture et de l'avoir posés dans `reports.api.ts`, les pages des fiches **inchangées** :
  la factorisation de leurs téléchargements toucherait deux modules de plus. Duplication d'une ligne (l'URL de la route)
  assumée, tracée en issue `technical-debt` (section *Issues à ouvrir*).
- **C-55-3b-3** — « Justificatif » et non « pièce » (vocabulaire de la 55-3a).

### References

- Fiche 55-3a ; fiche 30-2c (aperçu `DocumentPreview`) ; fiche 25-6-b (PDF figé).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

- (LOW, `bug` P4) `frontend/src/lib/components/ui/dialog/dialog-content.svelte:42` porte un `sr-only` « Close » anglais en
  dur pour tout dialogue à bouton par défaut (Epic 55, story 55-3b).

- (LOW, `technical-debt`) Lecture en `Blob` du PDF d'une facture et d'un avoir écrite deux fois : pages des fiches et
  `reports.api.ts` (C-55-3b-2).

### File List

## Change Log

- 2026-11-25 — Fiche écrite à la remédiation de la validation P1 de la 55-3 (découpage, V1-5) ; reprend V1-4 (erreurs de
  `DocumentPreview`), V1-6 (manuel, PDF figé), V1-9 (E2E à deux justificatifs), V1-14 à V1-17 (`data-testid`, URL,
  fichier E2E, `load` stable) (`drafted`).
- 2026-11-25 — Validation P1 (Sonnet) : 0/0/4/9 ; remédiée : colonne du grand livre (id, en-tête, conditionnelle, test des
  9 en-têtes, commentaire) (V1-1) ; dialogue sur `ui/dialog` sans le bouton anglais, clés propres (V1-2) ; tests de chaque
  AC, `Retour` précisé, `DocumentPreview.test.ts` complété sans modification des tests existants (V1-3) ; sélection par
  primitives et test de lecture unique qui rougit par mutation (V1-4) ; E2E à deux justificatifs par
  `reconciliation/accept` (A9, l'angle mort retiré) ; LOW V1-5 à V1-13.
- 2026-11-25 — Validation P2 (Opus) : 0/0/4/10 ; remédiée : `Blob` vide par une erreur dédiée et `errorMessage` à repli
  `undefined` (V2-1) ; inventaire des cinq phrases du figement au manuel (V2-2) ; E2E du Retour par l'historique, dialogue
  ouvert (V2-3) ; colonne conditionnelle et constantes dérivées, tests de couverture rejoués (V2-4) ; figement soumis au
  mainteneur (Q7, V2-5) ; LOW V2-6 à V2-14.
- 2026-11-25 — Validation P3 (Sonnet) : 0/0/0/7 — **validation terminée**. Tendance : P1 Sonnet 0/0/4/9 → P2 Opus 0/0/4/10 →
  P3 Sonnet 0/0/0/7. Les sept LOW ne sont pas remédiés dans la fiche : ils se reprennent au **T0 du développement**, ligne
  lue citée au Change Log — V3-1 (inventaire du figement par `grep -n "téléchargement\|télécharge"` sur le `.tex`, `:1553`,
  `:1642`, `:1672-1673`, `:2058` jugées ; constat sur le `.tex`, le PDF n'étant régénéré qu'à la release), V3-2 (décor E2E
  à montant exact sans compte désigné, aides neuves sans toucher au spec de l'Epic 43), V3-3 (issue tracée ci-dessus),
  V3-4 (assertion `goto` : garde, et `page.url()` avant et après à l'E2E), V3-5 (`lib.rs:1170`, ou E2E *Consultation*,
  patron `reports.spec.ts:448`), V3-6 (hauteur de l'aperçu bornée dans le dialogue ; texte générique de
  `pdfErrorMessage`), V3-7 (« condition `:37` »). Seule modification après la P3 : la phrase de l'AC-5 sur Q7 refusée,
  alignée sur la 55-3a (V4-1 de sa P4), relue par la passe P5 ciblée de la 55-3a.
- 2026-11-25 — **Récrite sur décision de l'utilisateur** : une icône par ligne, sans compteur ni menu ; la visionneuse
  existante (`DocumentPreview`, lecteur PDF du navigateur) avec la liste des documents en haut, chargée au clic par la route
  de la 55-3a ; un composant neuf `EntryDocumentsViewer` au-dessus de `DocumentPreview`, inchangé pour ses quatre
  appelants. Statut repassé à `drafted`, validation reprise à P1 sur cette version.
- 2026-11-25 — Validation P1 de la version récrite (Sonnet) : 0/0/1/8 ; remédiée : aides de l'E2E dans le spec neuf, le spec de
  l'Epic 43 intact (Vv2p1-1) ; cinq appelants (Vv2p1-2) ; liste relue à chaque ouverture, garde d'ouverture (Vv2p1-3) ;
  Vv2p1-4 à Vv2p1-9 **renvoyés au T0** (hauteur du dialogue, `:1643-1644` et `:1650` du manuel, fichier d'epic, une instance
  de dialogue par vue, nature de l'erreur « vide » et clés en double, E2E *Consultation* exigé).
- 2026-11-25 — Validation P2 ciblée (version récrite, Haiku) : 0/0/2/0 ; remédiée : `:1643`, `:1644`, `:1650` à l'AC-9
  (Vv2p2-1) ; renvois au T0 écrits en tâches (Vv2p2-1) ; test de la garde d'ouverture et sa mutation (Vv2p2-2).
- 2026-11-25 — Validation P3 ciblée (version récrite, Sonnet) : 0/0/1/3 ; remédiée : constat de l'AC-9 sur le `.tex`, huit
  lignes nommées, `:2058` jugée (Vv2p3-1) ; une formulation par ligne (Vv2p3-2) ; test de la garde d'ouverture spécifié
  (Vv2p3-3) ; un dialogue par vue, tranché (Vv2p3-4).
- 2026-11-25 — Validation P4 ciblée (version récrite, Haiku) : 0/0/2/2 ; remédiée **par simplification** : l'inventaire de
  lignes du manuel est remplacé par une règle, une méthode de recherche large et une table de jugement au Change Log, sur
  la base de développement (Vv2p4-1, Vv2p4-2, Vv2p4-3) ; focus rendu à l'icône d'origine, test et mutation (Vv2p4-4).
  Prochaine passe : P5 ; dès la P6, retrait ou renvoi.
