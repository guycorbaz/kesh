# Story 55-3b : ouvrir les justificatifs d'une écriture depuis le Journal (et le Grand livre)

Status: done

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 à D14. Issue : #1607 (point c, comme la
55-3a ; `enhancement`, P4, Q5). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration.** **Suit la 55-3a**
(booléen `hasDocuments`, route `GET /journal-entries/{id}/documents`, `pdfStatus`), **la 55-2** (mêmes vues) **et la 55-1a** (URL, mémoire des rapports).

*⚠️ **Décision du mainteneur du 2026-11-26** : l'aperçu marqué sort de l'Epic 55 (epic « PDF marqués », numéro à
confirmer ; 55-3a, section *Reporté à l'epic « PDF marqués »*). Une facture **figée** s'ouvre par son PDF figé
(`GET /invoices/{id}/pdf`, qui ne fige rien de plus) ; une facture **pas encore émise** est signalée « PDF pas encore
émis », **sans aperçu** et sans aucune lecture de PDF (AC-5). Les anciens textes de l'aperçu (AC-3, AC-5, AC-9, recette
visuelle) sont gardés en section *Reporté à l'epic « PDF marqués »*.*

*Découpée de la 55-3 à la validation P1 (V1-5).*

*⚠️ **Récrite le 2026-11-25 sur décision de l'utilisateur** : **une seule icône PDF par ligne d'écriture**, quel que soit
le nombre de justificatifs — ni compteur, ni menu ; au clic, **la visionneuse existante** (`DocumentPreview`, dont le
cadre affiche le PDF dans le lecteur du navigateur, pages à gauche), avec **la liste des documents de l'écriture en haut** :
un clic dans la liste change le document affiché. **Avec un seul justificatif, aucune liste** : la visionneuse l'ouvre
directement ; la liste en haut ne paraît qu'**à partir de deux** (simplification de l'utilisateur, 2026-11-25). La liste se
charge au clic (route de la 55-3a). Validation reprise à P1 sur cette version.*

## ⚠️ Points renvoyés au développement (validation Pv2p7, et Pv2p3 de la 55-3a)

**État au 2026-11-26** (décisions du mainteneur, aperçu sorti de l'epic) : **R-1 caduc** (le bandeau « Aperçu — PDF non
figé » n'existe plus ; le texte « PDF pas encore émis » de l'AC-5 le remplace) ; **R-2 caduc** (sans aperçu, les deux
phrases du § 11.6.1 restent justes : aucun PDF non émis ne sort ; le point suit l'aperçu, 55-3a section reportée) ;
**R-3 tranché** (resserrement de la dérogation Q6 : § *Gates*) ; **R-4 caduc** (AC-9 récrit) ; **R-5 caduc** (aucun PDF
marqué) ; LOW : Vv2p7-3 caduc (aucun nom `apercu-facture-…`), Vv2p7-4 repris pour le message (l'état « pas encore émis »
suit l'**élément choisi**, AC-5), Vv2p7-5 repris (le message au lieu du bandeau), Vv2p7-6 sans objet. Le texte ci-dessous
est celui du 2026-11-25.

**Précisions d'interface du 2026-11-26** (passe ciblée hors décompte, rapport
`kesh-gate-logs/s55-decisions-2/interface-2026-11-26-validation.md`) — à trancher au **T0**, ligne lue citée :

- **R-8 (UI-1, MEDIUM) — Couleur de l'icône.** `stroke="currentColor"` est vrai de toute icône lucide, et la liste de
  classes interdites est ouverte. Défaut : aucun préfixe `text-`, `fill-`, `stroke-` ni `bg-` de couleur sur le bouton ni
  sur le SVG (Vitest à mutation), et un E2E qui compare `getComputedStyle(svg).color` à celle du `#numéro` de la même
  ligne.
- **R-9 (UI-2, MEDIUM) — Vignettes : un constat qui constate.** Les E2E tournent sans tête (`playwright.config.ts` sans
  `headless` ni `channel`), où Chromium n'a pas de lecteur PDF : « pas de vignettes » y est vrai de toute URL. Défaut :
  capture dans un navigateur **avec tête** (celui où l'utilisateur constate le comportement des factures fournisseur,
  nommé avec sa version), sur un PDF d'une page et un de deux ; repli `#pagemode=none`, essayer aussi `navpanes=0`
  (UI-5).
- **R-10 (UI-3, MEDIUM) — § *Tests à écrire* à compléter** : le Vitest de focus et l'E2E disent encore « Échap,
  Fermer » ; ajouter le clic à côté, la hauteur par `boundingBox`, le `src` sans fragment, la couleur (R-8).
- **LOW** : UI-4 (`DocumentPreview.svelte:92-97`, non `:96-101`) ; UI-5 (voir R-9) ; UI-6 (la ligne d'en-tête,
  `JournalReportView.svelte:59`, n'a pas de `data-testid` : en poser un ; mutation de la mesure de hauteur) ; UI-7 (Échap
  inopérant une fois le focus dans le PDF : le dire au manuel et à la note de version) ; UI-8 (Vitest du clic à côté sans
  précédent dans Kesh — rejet bits-ui par `pointerdown`, `debounce` de 10 ms : l'écrire en premier, l'E2E en repli).

La validation s'est arrêtée à la **septième passe** de la version récrite sur un MEDIUM ; au-delà de la sixième passe,
l'orchestrateur ne remédie plus (`CLAUDE.md` § *Review Iteration Rule*) : les points vont au **T0**, tranchés avant toute
ligne de code, avec le mainteneur si le texte visible change, et le Change Log cite la ligne lue et la décision. La P8,
dernière permise, est gardée en réserve. Le correctif proposé est la résolution par défaut.

- **R-1 (Vv2p7-1, MEDIUM) — Texte du bandeau.** « … n'a encore été ni téléchargée ni envoyée … » est faux pour une facture
  non figée qui l'a été : dévalidée puis revalidée (`user-manual.tex:1645`), restaurée d'une sauvegarde d'avant le gel
  (`:1644`), envoyée avant la v0.12.1 (`:1642-1643`). Défaut : « Aperçu — PDF non figé : ce n'est pas encore le PDF émis
  de cette facture ; il peut encore changer. » — dans l'AC-5, l'AC-8, la phrase du § 11.6.1 de l'AC-9 et D12 du fichier
  d'epic (`grep -n "ni téléchargée ni envoyée"` sur les fiches 55-* et le fichier d'epic : zéro hors historiques).
- **R-2 (Vv2p3-2 de la 55-3a, MEDIUM) — Deux phrases du § 11.6.1 deviennent fausses**, contrairement à ce que dit l'AC-9
  (« aucune n'est réécrite ») : `user-manual.tex:1637` (« Le PDF d'une facture est produit **une seule fois** ») et `:1641`
  (« une pièce sortie de Kesh est figée »). Défaut : « Le PDF **émis** d'une facture est produit une seule fois … ; depuis
  les rapports, une facture non figée s'affiche en **aperçu**, qui n'est pas le PDF émis » ; la phrase de la pièce sortie
  nuancée : l'aperçu affiché dans Kesh n'est pas une pièce émise, et il est fermé aux clés d'API (55-3a) ; l'AC-9 dit
  « les phrases qui attribuent le figement au téléchargement et à l'envoi restent justes ; ces deux-là sont ajustées ».
- **R-11 (D16, 2026-11-26, soir)** — Un rechargement garde la vue montée et grisée (55-1a, AC-24) : au T0, un Vitest
  constate qu'un rechargement arrivé dialogue ouvert ne relit pas le document (lecture unique, AC-5), et que la ligne
  d'écriture du dialogue reste celle du tableau affiché ; ligne lue citée au Change Log.
- **R-3 à R-5 (Pv2p8, LOW, `kesh-gate-logs/s55-decisions/55-3b-validation-pv2p8.md`)** : R-3 (gardes Rust des
  catalogues que la story modifie : la précision de la *Dérogation* proposée au mainteneur, fichier d'epic, question
  *Gardes Rust des textes*) ; R-4 (l'AC-9 renvoie à R-2 : « aucune n'est réécrite » sauf `:1637` et `:1641`) ; R-5 (la
  capture du PDF marqué sous le bandeau : navigateur réel non headless sur une installation jetable, ou deux captures
  composées — dialogue et PDF de la 55-3a —, dit au Dev Agent Record).
- **LOW** : Vv2p7-3 et V8-4 de la 55-1a (nom `apercu-facture-<numéro>.pdf` passé à `DocumentPreview` quand `pdfStatus`
  vaut `'notFrozen'`, test Vitest du nom) ; Vv2p7-4 (le bandeau suit l'**élément choisi** : deux éléments `notFrozen` puis
  `frozen`, un clic sur le second retire le bandeau ; mutation « bandeau tenu par `list[0]` ») ; Vv2p7-5 (contrôle de
  hauteur du T0 : titre, liste, **bandeau**, cadre et « Fermer ») ; Vv2p7-6 (objectif 3 du fichier d'epic : corrigé par la
  remédiation de la Pv2p3 de la 55-3a).

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
  'supplierDocument'; id: number; number: string | null; fileName?: string; pdfStatus?: string }` (`pdfStatus` : facture
  seulement, `'frozen'` ou `'notFrozen'`, énumération ouverte) ; **aucune route de PDF neuve** (55-3a, *Afficher sans
  figer, dans l'Epic 55*) : `GET /api/v1/invoices/{id}/pdf` sert le PDF figé d'une facture figée sans rien écrire, et
  figerait une facture qui ne l'est pas.
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
   Une écriture sans `hasDocuments` n'en a pas. Aucune requête n'est faite avant le clic. **Aspect** (précision de
   l'utilisateur du 2026-11-26) : l'icône est **un peu plus grande qu'une icône de texte** (p. ex. `FileText` de
   `@lucide/svelte`, déjà employé par Kesh, à `1.15em` environ, réglé à la capture), **sans augmenter la hauteur de la
   ligne** (bouton sans remplissage vertical, `leading-none`, aligné sur le texte), et **monochrome, de la couleur du
   texte** (`currentColor`, aucune classe de couleur : ni rouge, ni couleur d'alerte ou d'accent), pour ne pas attirer
   l'œil ; elle garde son libellé accessible (`aria-label` ci-dessus, l'icône `aria-hidden`). Constat : Vitest — le bouton
   ne porte aucune classe de couleur (`text-destructive`, `text-warning`, `text-red-*`, `text-primary` : absentes) et son
   SVG `stroke="currentColor"` ; E2E — dans un même journal, la hauteur (`boundingBox`) de la ligne d'en-tête d'une
   écriture **avec** icône égale celle d'une écriture **sans** icône, à un pixel près ; capture.
2. **AC-2 — La visionneuse et sa liste.** Au clic, un dialogue s'ouvre ; il lit `GET /api/v1/journal-entries/{id}/documents`
   (une requête, un indicateur de chargement), puis, s'il y en a deux ou plus, montre **en haut** la liste des justificatifs (genre, numéro, nom du
   fichier d'un document fournisseur ; l'élément affiché marqué `aria-current`) et **dessous** `DocumentPreview` sur le
   **premier** de la liste — ou, pour une facture pas encore émise, le message de l'AC-5 à la place de
   `DocumentPreview`. Un clic sur un autre élément change le document affiché, sans fermer le dialogue. Un seul
   justificatif : **aucune liste** — ni élément, ni libellé (`data-testid="entry-documents-list"` absent) — et
   `DocumentPreview` l'ouvre directement (facture pas encore émise : le message de l'AC-5) ; la liste ne paraît qu'à partir de **deux** justificatifs. Liste vide (justificatif disparu depuis le rapport) : le
   message `reports-nav-documents-none` ; erreur de la route : `errorMessageOf`. La liste est **relue à chaque ouverture** ;
   une réponse d'une ouverture antérieure (fermé puis rouvert en vol) est ignorée (compteur d'ouverture), test dédié.
3. **AC-3 — Les trois genres et leurs lecteurs.** `fetchInvoicePdf(id)` → `GET /api/v1/invoices/{id}/pdf`, appelé
   **seulement** pour une facture dont l'élément porte `pdfStatus: 'frozen'` (le PDF figé, servi sans rien écrire) ;
   `fetchCreditNotePdf(id)` → `GET /api/v1/credit-notes/{id}/pdf` (les deux dans `reports.api.ts`, par `apiClient.getBlob`) ;
   document fournisseur → `fetchSupplierInvoiceSourceDocument(id)`. Noms passés à `DocumentPreview` :
   `facture-<numéro>.pdf`, `avoir-<numéro>.pdf`, à défaut du numéro `facture-<id>.pdf`, `avoir-<id>.pdf` ; le `fileName` du
   document fournisseur, à défaut `justificatif-<id>`. Dans la liste, un numéro nul s'affiche par l'identifiant. Un `Blob`
   vide : le lecteur d'`EntryDocuments` **rejette** par une erreur « vide » dédiée (`DocumentPreview` ne contrôle pas la
   taille, `:50-61`), rendue par `reports-nav-documents-empty` (« Le document reçu est vide. »), pour les trois genres —
   clé neuve parce que `lint-i18n-ownership` refuse `invoice-pdf-error-empty` depuis `features/reports`.
4. **AC-4 — Erreurs, droits.** Les trois routes d'aujourd'hui (`/pdf` pour une facture figée, l'avoir, le document
   fournisseur), toutes sous « tout rôle authentifié » (un rôle
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
5. **AC-5 — Aucun figement, « PDF pas encore émis », lecture unique.** Ouvrir la facture d'une écriture **ne la fige
   pas** (réponse de l'utilisateur du 2026-11-25, D12) : l'écran ne lit `/pdf` que pour `pdfStatus: 'frozen'`, où il sert
   le PDF figé **tel quel**, sans marque ni bandeau (Q1). Une facture dont l'élément porte toute **autre** valeur —
   `'notFrozen'`, champ absent ou valeur future (55-3a, énumération ouverte) — **ne lit rien** : à la place de
   `DocumentPreview`, un message (`data-testid="entry-documents-not-issued"`, `role="status"`), texte
   `reports-nav-documents-not-issued` : « PDF pas encore émis : cette facture n'a pas encore de PDF figé ; son prochain
   téléchargement depuis sa fiche, ou son prochain envoi par e-mail, le figera. » — **sans aperçu**, tant que l'epic « PDF
   marqués » n'est pas livré (décision du mainteneur du 2026-11-26). Les termes sont ceux de Kesh : « PDF émis »
   (`error-invoice-pdf-gone`, « Le PDF émis de cette facture … »), « PDF figé » et « son prochain téléchargement le
   figera » (`error-invoice-pdf-not-frozen`) ; texte définitif au T4, quatre locales (AC-8). Il vaut aussi pour une
   facture dont un PDF est sorti sans être figé (`emailed_at`, empreinte d'audit : 55-3a, *Afficher sans figer*) — il ne
   dit pas qu'elle n'a jamais été envoyée ni téléchargée : **aucune nuance de texte**. Le message suit l'**élément
   choisi** (deux factures, `notFrozen` puis `frozen` : un clic sur la seconde retire le message et ouvre son PDF ;
   mutation « état tenu par `list[0]` » → test rouge). Aucun message pour `'frozen'`, un avoir ou un document
   fournisseur. Il suit la liste lue à l'ouverture : une facture figée entre-temps garde le message jusqu'à la prochaine
   ouverture (limite dite au manuel, AC-9). La **fiche de la facture ne change pas** : elle garde son bouton, qui fige au
   premier téléchargement — le geste d'émission (D14, décision du mainteneur du 2026-11-25, Q8 refusée).
   **Lecture unique** : le justificatif choisi
   est tenu par **deux primitives** (`kind`, `id`), et la fonction `load` est dérivée de ces seules primitives : remplacer
   le rapport ou la liste relue par des objets égaux ne relit pas le document. Changer d'élément dans la liste ne relit que le document
   choisi. Le cas réel du tableau remplacé : Retour ou Suivant vers le **même
   onglet**, dont le rapport est repris de la mémoire (55-1a, C-55-1-2) ou redemandé, l'écriture y étant encore ; *Générer*
   n'est pas atteignable dialogue ouvert (modal) et démonte la vue (`{#if loading}`). *(D16, 2026-11-26 : *Générer*
   disparaît des rapports à période, et un **rechargement** garde la vue montée, grisée, 55-1a AC-24 ; aucun déclencheur
   n'est atteignable dialogue ouvert, mais un rechargement en vol à l'ouverture peut remplacer le tableau : c'est le cas
   de la « lecture unique » ci-dessus, à constater par un Vitest au T0, R-11.)*
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
   (`reports-nav-documents-close`), une largeur qui tient la visionneuse (`sm:max-w-4xl`) et un contenu qui défile ; titre
   `reports-nav-documents-title` (« Justificatifs de l'écriture n° { $number } ») ; **trois façons de fermer**, comme
   les autres dialogues de Kesh (précision de l'utilisateur du 2026-11-26) : le bouton **« Fermer »**, visible ; un **clic
   à côté** (sur le fond) ; la touche **Échap** (focus dans le dialogue, hors du cadre du PDF, qui est un autre document)
   — ce que le dialogue de Kesh offre déjà : `interactOutsideBehavior` et `escapeKeydownBehavior` valent `"close"` par
   défaut (bits-ui 2.16.5, `utilities/dismissible-layer/dismissible-layer.svelte:8`,
   `utilities/escape-layer/escape-layer.svelte:8`), et le dialogue ne les surcharge pas (les dialogues qui les surchargent,
   p. ex. `WriteOffDialog.svelte:124-125`, ne le font que pendant un envoi en vol, ce que la visionneuse n'a pas) ; un clic
   **dans** le PDF n'est pas un clic à côté. Chaque fermeture rend le focus au bouton d'origine. Constat : Vitest, une
   fermeture par chacune des trois voies, focus rendu (mutation : `interactOutsideBehavior="ignore"` → test du clic à
   côté rouge) ; E2E, clic sur le fond → `[role="dialog"]` en nombre 0. Ouvrir le dialogue n'appelle pas `goto` (55-1a, D2 : l'URL ne change pas). Une navigation qui
   change le rapport affiché (Retour vers un autre onglet) démonte la vue et ferme le dialogue. Un rapport repris de la
   mémoire (55-1a, D3) peut porter un justificatif devenu introuvable : l'erreur s'affiche dans le dialogue (AC-4).
8. **AC-8 — Textes.** Clés neuves du bloc `reports-nav-*`, quatre locales : `reports-nav-documents-button` (« Justificatifs
   de l'écriture n° { $number } »), `reports-nav-documents-title`, `reports-nav-documents-kind-invoice` (« Facture »),
   `-kind-credit-note` (« Avoir »), `-kind-supplier-document` (« Document fournisseur »), `reports-nav-documents-list`
   (libellé de la liste), `reports-nav-documents-loading`, `reports-nav-documents-none`, `reports-nav-documents-close`
   (« Fermer »), `reports-nav-documents-empty`, `reports-nav-documents-not-issued` (« PDF pas encore émis », AC-5 ;
   proposition pour les trois autres locales, reprise des termes de leurs catalogues : de « PDF noch nicht ausgestellt:
   Diese Rechnung hat noch kein festgeschriebenes PDF; der nächste Download aus ihrer Ansicht oder der nächste Versand per
   E-Mail schreibt es fest. », it « PDF non ancora emesso: questa fattura non ha ancora un PDF fissato; il prossimo download
   dalla sua scheda, o il prossimo invio per e-mail, lo fisserà. », en « PDF not yet issued: this invoice has no frozen PDF
   yet; its next download from the invoice page, or its next sending by e-mail, will freeze it. »),
   `reports-nav-ledger-column-documents` ; constat : `i18n-keys` (`sitesTotal` recompté par
   la garde, parité des quatre locales), `i18n-libelle-en-dur`, `i18n-un-repli-par-cle`, `npm run lint-i18n-ownership`
   verts.
9. **AC-9 — Manuel.** `docs/manual/fr/user-manual.tex` (lignes à `1e23edb55`) : § *Journal des écritures* (`:4316`) — le
   bouton, les trois genres, le règlement qui montre la facture réglée, la liste, « PDF pas encore émis » ; § *Grand
   livre*, l'énumération des colonnes (`:4193-4194`) gagne « ses justificatifs » ; le chemin « Mensuel → Rapports →
   Journal » (`:4318`) contrôlé contre l'onglet réel (« Journaux »). **§ 11.6.1** (`sec:pdf-fige`, `:1635-1645`) gagne
   **une** phrase : afficher une facture depuis les rapports (Journal, Grand livre) **ne la fige pas** — une facture
   figée s'y montre par son PDF figé ; une facture qui ne l'est pas encore y est signalée « PDF pas encore émis », sans
   aperçu ; le PDF émis reste produit au premier téléchargement depuis la fiche ou au premier envoi ; l'état affiché vaut
   à l'ouverture de la visionneuse. Renvoi `\S\ref{sec:pdf-fige}` depuis le paragraphe du Journal. **Les phrases du
   figement restent justes** (téléchargement et envoi demeurent les seuls gestes qui figent, et aucun PDF non émis ne
   sort de Kesh) : aucune n'est réécrite ; le T4 le constate par `grep -nE "fig[ée]|refig" docs/manual/fr/user-manual.tex
   docs/manual/fr/admin-manual.tex` sur la base de développement, nombre de lignes lues déclaré au Change Log, et toute
   phrase qui dirait le contraire (« toute ouverture fige ») corrigée. **Le refus d'un PDF figé disparu**
   (`user-manual.tex:1650`) : l'affichage depuis les rapports est refusé de même, avec le même message ; le bouton
   *Refiger* n'apparaît qu'après un téléchargement refusé **depuis la fiche** (`invoices/[id]/+page.svelte:926`, `:994`,
   `:1037`) — une demi-phrase. `.tex` seul ; le PDF se régénère à la release. Captures du manuel montrant le grand livre
   (p. ex. `rd-grand-livre-usd.png`) : périmées si la colonne y paraît, à refaire à la release (dit au Change Log).
10. **AC-10 — Accessibilité.** Bouton au clavier ; dialogue `role="dialog"`, `aria-modal`, étiqueté par son titre ; axe
    à zéro violation, dialogue ouvert : `new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).include('[role="dialog"]')`
    (modèle : les specs axe existantes de `tests/e2e/`).
11. **AC-11 — Pas de vignettes de pages par défaut** (précision de l'utilisateur du 2026-11-26). Une facture ne fait le
    plus souvent qu'une ou deux pages : la visionneuse l'ouvre **sans le panneau des vignettes** à gauche, comme
    aujourd'hui pour les factures fournisseur. Relevé au code : `DocumentPreview` ne passe **aucun paramètre** au lecteur
    PDF du navigateur (`DocumentPreview.svelte:96-101`, `<iframe src={view.url}>`, l'URL `blob:` nue ; `grep -rn
    "pagemode\|navpanes" frontend/src` ne rend rien) ; les PDF que Kesh produit portent `/PageMode /UseNone` (printpdf 0.7,
    `pdf_document.rs:498-505`, sans signets). Le comportement des factures fournisseur vient donc du lecteur du navigateur
    et du PDF, non d'un réglage de Kesh, et la 55-3b le **reprend tel quel** : même `DocumentPreview`, même URL nue.
    Constat : Vitest — le `src` du cadre est l'URL `blob:` sans fragment (mutation : fragment ajouté → test rouge, garde de
    non-régression déclarée telle) ; **recette visuelle** dans Chromium (le navigateur des E2E) : une facture figée d'une
    page s'ouvre sans panneau de vignettes. **Si le lecteur les montre** malgré tout, le T0 ajoute à `DocumentPreview` une
    propriété facultative qui suffixe l'URL de `#pagemode=none` (paramètre d'ouverture des PDF d'Adobe, lu par pdf.js de
    Firefox), absente pour les cinq appelants d'aujourd'hui, et vérifie à la capture que Chromium l'honore ; décision et
    capture au Change Log.

## Tests à écrire, et la preuve du rouge

- **Vitest** `EntryDocuments.test.ts` (neuf, l'icône et le dialogue) et `EntryDocumentsViewer.test.ts` (neuf, la liste
  au-dessus de `DocumentPreview`) : aucune icône sans `hasDocuments`, une seule avec (que la liste compte 1 ou 3
  éléments) ; aucune requête avant le clic, **une** au clic ; premier élément affiché ; clic sur un autre élément → son
  lecteur appelé, une lecture ; un seul justificatif → **aucune liste** (`entry-documents-list` absent), la visionneuse ouverte ;
  deux → la liste ; message « pas encore émis » présent, **sans aucun appel de lecteur ni `DocumentPreview`**, pour
  `'notFrozen'`, un champ absent et une valeur inconnue ; absent pour `'frozen'`, un avoir, un document fournisseur
  (mutations : message toujours affiché, liste rendue à un élément, lecteur de facture appelé pour `'notFrozen'` → tests
  rouges) ; message qui suit l'élément choisi (AC-5, mutation « état tenu par `list[0]` ») ; lecteur de facture qui vise
  `/pdf` ; liste vide ; erreur de la route ; les trois
  genres (lecteur appelé, nom de fichier, replis de l'AC-3) ; erreur `410` → texte de `pdfErrorMessage` ; `Blob` vide
  **pour chacun des trois genres** ; **lecture unique** : dialogue ouvert, rapport remplacé par un objet égal → toujours
  **une** lecture (mutation déclarée au journal : sélection tenue par l'objet → deux lectures, test rouge) ; ouverture sans
  appel de `goto` (mock de `$app/navigation`) ; **fermé puis rouvert pendant la lecture de la liste** — deux promesses
  contrôlées, la première résolue **après** la seconde, avec des listes de noms **différents** ; l'assertion porte sur la
  liste affichée : celle de la seconde ouverture — (mutation : compteur d'ouverture retiré → la liste ancienne s'affiche, test rouge).
- **Vitest** (focus, dans `EntryDocuments.test.ts`) : fermeture (Échap, « Fermer ») → focus sur l'icône qui a ouvert (deux
  icônes, la seconde ouvre, par `fireEvent.click`, qui ne donne pas le focus au bouton). Mécanisme : `onCloseAutoFocus` du
  dialogue avec `preventDefault()` puis le focus rendu à l'élément retenu (bits-ui 2.16.5 rend le focus par défaut,
  `focus-scope.svelte.js:79-90`, ce qui laisserait l'E2E vert sans le code). Mutation : retrait du rendu explicite → test
  rouge.
- **Vitest** `reports.api.test.ts` : `fetchEntryDocuments(entryId)`, `fetchInvoicePdf`, `fetchCreditNotePdf` (URL exactes).
- **Vitest** `DocumentPreview.test.ts` : tests ajoutés (propriété présente rendant une chaîne, y compris pour
  `SOURCE_DOCUMENT_GONE` ; rendant `undefined` ; absente ; nouveau rendu avec une autre fonction `errorMessage` → **une**
  lecture).
- **Vitest** `JournalReportView.test.ts` et `GeneralLedgerView.test.ts` : icône présente avec `hasDocuments`, absente sans ;
  colonne présente ou absente, et les tests de l'AC-6 rejoués avec la colonne.
- Rouges sans le correctif, journal `kesh-gate-logs/s55-3b/`.
- **E2E** `frontend/tests/e2e/reports-entry-documents.spec.ts` (neuf, déclaré au tableau des fichiers communs), **cible :
  le Journal** : créer et valider une facture par l'API, ouvrir *Rapports → Journaux* sur la période, cliquer le bouton de
  l'écriture de vente (`journalEntryId` de la facture) → **aucune liste**, message `entry-documents-not-issued`
  visible, `document-preview` absent, **aucune requête** vers `/api/v1/invoices/{id}/pdf` (écoute `page.on('request')`) ;
  puis `GET /api/v1/invoices/{id}` rend `pdfFrozenAt: null` (la facture n'a pas été figée) ; la facture figée ensuite
  (`GET /pdf` par l'API, le geste de la fiche), une seconde ouverture montre `document-preview-frame`, sans message ;
  *Échap* ferme, focus rendu ;
  axe à zéro violation dialogue ouvert. **Deux justificatifs** : deux factures validées, puis un règlement groupé par
  `POST /api/v1/reconciliation/accept` (`invoice_group`, patron `reconciliation-group.spec.ts:796-800`, écriture lue dans
  `accepted[0].journalEntryId`) → **une** icône sur l'écriture de règlement ; les deux factures figées d'abord par l'API
  (`GET /pdf`) ; le dialogue montre la liste de deux, la première facture affichée ; un clic sur la seconde change le
  document (la requête de son PDF part) ; les aides
  (`createAndValidateInvoiceViaApi`, `createManualCreditStatementViaApi` d'`api-fixtures.ts`, et des aides neuves **dans le
  spec neuf** pour le règlement groupé — le spec `reconciliation-group.spec.ts` de l'Epic 43 n'est pas touché, ni
  `api-fixtures.ts` modifié) ; nettoyage en `finally` comme le patron (groupe annulé, import supprimé).
  **Retour** : partir du *Bilan*, cliquer *Journaux* (entrée d'historique, 55-1a D2), générer, ouvrir le dialogue
  (`[role="dialog"]` visible), puis `page.goBack()` **dialogue ouvert** → URL `tab=balance-sheet`, `[role="dialog"]` en
  nombre 0, et un clic sur un onglet réussit (aucun overlay ni verrou de défilement resté).
  Grand livre : un test court (colonne, bouton, visionneuse). Le test du Journal rougit sans le correctif (aucun bouton).

## Recette visuelle

Captures (thème clair, `kesh-gate-logs/s55-3b/`) : journal avec ses icônes (monochromes, de la couleur du texte,
lignes à hauteur inchangée, AC-1) ; visionneuse sans panneau de vignettes (AC-11) ; visionneuse à un justificatif (ouverte
directement, sans liste) ; visionneuse à deux justificatifs, chacun affiché ; facture pas encore émise : le message « PDF
pas encore émis » à la place de l'aperçu ; erreur `410` (obtenue sur une installation jetable en retirant de
`KESH_DOCUMENTS_DIR` le fichier `<empreinte>.pdf` d'une facture figée, ou par `page.route` rendant `410 INVOICE_PDF_GONE`) ;
grand livre avec la colonne.

## Tâches

- [x] **T0** — Base `epic/55-navigation` avec les 55-1a, 55-1b, 55-2 et 55-3a ; lignes du manuel sur la base de développement ; et les
      points renvoyés par la validation, chacun tranché au Change Log, ligne lue citée :
      - hauteur de la visionneuse dans le dialogue (titre, liste, cadre et « Fermer » tiennent dans `85vh`, sans double
        défilement) ;
      - fichier d'epic : `helpers/api-fixtures.ts` non touché ;
      - (tranché : **une** instance du dialogue par vue — `EntryDocuments.svelte` est rendu une fois par vue, avec l'écriture
        ouverte en état ; l'icône par ligne n'est qu'un bouton qui l'ouvre ; T2 en conséquence) ;
      - nature de l'erreur « vide » (classe dédiée reconnue par `errorMessage` avant `pdfErrorMessage`) ; une clé pour le
        libellé de l'icône et le titre s'ils portent le même texte ;
      - E2E *Consultation* exigé (patron `reports.spec.ts:441`) ;
      - LOW de la version précédente (V3-1 à V3-7).
      - **caducs depuis D12** (l'affichage ne fige plus) : V3-1, Vv2p6-1 et Vv2p6-2, qui portaient sur la réécriture des
        phrases du figement ; à écrire tels au Change Log ;
      - hauteur (Vv2p7-5) : titre, liste, message ou cadre, et « Fermer » ;
      - points de la passe du 2026-11-26 (Change Log), la fiche étant au plafond de huit passes.
- [x] **T1** — Vitest rouges.
- [x] **T2** — `DocumentPreview` (`errorMessage`, rien d'autre) ; lecteurs dans `reports.api.ts`
      (`fetchEntryDocuments`, `fetchInvoicePdf`, `fetchCreditNotePdf`) ; l'état « pas encore émis » (AC-5) ; `EntryDocumentsViewer.svelte` (la liste en haut,
      `DocumentPreview` dessous, sélection par primitives) ; `EntryDocuments.svelte` (le dialogue `ui/dialog`, **une
      instance par vue**, et la lecture de la liste au clic) ; l'icône de chaque ligne est un bouton qui lui passe
      l'écriture **et son élément** ; l'instance rend le focus à cet élément à la fermeture.
- [x] **T3** — Vues du journal et du grand livre (colonne conditionnelle) ; types `reports.types.ts`.
- [x] **T4** — i18n, manuel, E2E, captures.
- [x] **T5** — Gates (fichier d'epic, § *Gates*).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E de la story et
des specs des Rapports et de la navigation, au dernier commit de code ; la story modifiant `messages.ftl` et le manuel,
**à sa clôture** aussi `cargo nextest run -p kesh-i18n` et `cargo nextest run -p kesh-api -E 'binary(textes_coherents)'`
(resserrement de la dérogation, décision du mainteneur du 2026-11-26) ; story **purement écran** : E2E complet et
`scripts/gate-ci.sh` en fin d'epic (fichier d'epic, § *Gates*, *Dérogation*, Q6).

## Note de version

*Rapports → Journaux* et *Grand livre* : une icône PDF sur une écriture ouvre ses justificatifs dans la visionneuse — PDF
de la facture, de l'avoir, document de la facture fournisseur ; pour un règlement, la ou les factures réglées. Avec un
seul justificatif, il s'ouvre directement ; avec plusieurs, une liste en haut permet de choisir. Afficher ainsi une
facture **ne la fige pas** : une facture figée s'ouvre par son PDF figé ; une facture dont le PDF n'est pas encore émis
est signalée « PDF pas encore émis », sans aperçu. (refs #1607)

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

## Reporté à l'epic « PDF marqués »

Décision du mainteneur du 2026-11-26 (55-3a, même section). Textes de la fiche au 2026-11-25, après la Pv2p8, **laissés
tels quels** pour l'écran de l'epic « PDF marqués » (la visionneuse affichera alors l'aperçu marqué d'une facture pas
encore émise, sous un bandeau ; et, pour un PDF sorti sans être figé, le bandeau « Un PDF de cette facture est sorti le …
sans être gardé ; ceci est un aperçu. ») ; s'y rattachent R-1, R-2, R-5 et Vv2p7-3 (points renvoyés, en tête).

- *(ancien)* **AC-3 — Les trois genres et leurs lecteurs.** `fetchInvoicePdfPreview(id)` → `GET /api/v1/invoices/{id}/pdf/preview`
   (l'aperçu sans figement de la 55-3a ; **jamais** `/pdf`, qui fige) ;
   `fetchCreditNotePdf(id)` → `GET /api/v1/credit-notes/{id}/pdf` (les deux dans `reports.api.ts`, par `apiClient.getBlob`) ;
   document fournisseur → `fetchSupplierInvoiceSourceDocument(id)`. Noms passés à `DocumentPreview` :
   `facture-<numéro>.pdf`, `avoir-<numéro>.pdf`, à défaut du numéro `facture-<id>.pdf`, `avoir-<id>.pdf` ; le `fileName` du
   document fournisseur, à défaut `justificatif-<id>`. Dans la liste, un numéro nul s'affiche par l'identifiant. Un `Blob`
   vide : le lecteur d'`EntryDocuments` **rejette** par une erreur « vide » dédiée (`DocumentPreview` ne contrôle pas la
   taille, `:50-61`), rendue par `reports-nav-documents-empty` (« Le document reçu est vide. »), pour les trois genres —
   clé neuve parce que `lint-i18n-ownership` refuse `invoice-pdf-error-empty` depuis `features/reports`.
- *(ancien)* **AC-5 — Aucun figement, bandeau, lecture unique.** Ouvrir la facture d'une écriture **ne la fige pas** (réponse de
   l'utilisateur du 2026-11-25, D12) : l'écran lit l'aperçu (AC-3), jamais `/pdf`. Le PDF de l'aperçu d'une facture non
   figée **porte lui-même sa marque** (filigrane « APERÇU », en-tête « Aperçu — non émis », QR-facture barrée, 55-3a, D13) ;
   celui d'une facture figée est le PDF figé tel quel, sans marque ni bandeau (décision du mainteneur, Q1). L'écran ne
   dessine rien sur le PDF : la marque vient du serveur. Une facture dont l'élément porte
   `pdfStatus: 'notFrozen'` s'affiche avec un **bandeau** au-dessus de `DocumentPreview`
   (`data-testid="entry-documents-not-frozen"`, `role="status"`), texte `reports-nav-documents-not-frozen` : « Aperçu — PDF
   non figé : cette facture n'a encore été ni téléchargée ni envoyée, et son PDF peut encore changer. » (« figé », « PDF
   émis » : les termes de Kesh, manuel § 11.6.1, `invoice-detail-pdf-frozen-at-label` « Document figé le »,
   `error-invoice-pdf-not-frozen`). Aucun bandeau pour `'frozen'`, pour un avoir, un document fournisseur, ni pour une
   valeur inconnue de `pdfStatus` (une valeur future, 55-3a *Limite connue et évolution*). Le bandeau suit la liste lue à
   l'ouverture : une facture figée entre-temps le garde jusqu'à la prochaine ouverture (limite dite au manuel, AC-9). La
   **fiche de la facture ne change pas** : elle garde son bouton, qui fige au premier téléchargement — le geste
   d'émission (D14, décision du mainteneur du 2026-11-25, Q8 refusée).
   **Lecture unique** : le justificatif choisi
   est tenu par **deux primitives** (`kind`, `id`), et la fonction `load` est dérivée de ces seules primitives : remplacer
   le rapport ou la liste relue par des objets égaux ne relit pas le document. Changer d'élément dans la liste ne relit que le document
   choisi. Le cas réel du tableau remplacé : Retour ou Suivant vers le **même
   onglet**, dont le rapport est repris de la mémoire (55-1a, C-55-1-2) ou redemandé, l'écriture y étant encore ; *Générer*
   n'est pas atteignable dialogue ouvert (modal) et démonte la vue (`{#if loading}`).
- *(ancien)* **AC-9 — Manuel.** `docs/manual/fr/user-manual.tex` (lignes à `1e23edb55`) : § *Journal des écritures* (`:4316`) — le
   bouton, les trois genres, le règlement qui montre la facture réglée, la liste ; § *Grand livre*, l'énumération des
   colonnes (`:4193-4194`) gagne « ses justificatifs » ; le chemin « Mensuel → Rapports → Journal » (`:4318`) contrôlé
   contre l'onglet réel (« Journaux »). **§ 11.6.1** (`sec:pdf-fige`, `:1635-1645`) gagne **une** phrase : afficher une
   facture depuis les rapports (Journal, Grand livre) **ne la fige pas** — une facture non figée s'y montre en aperçu, sous
   un bandeau « PDF non figé », et ce PDF d'aperçu porte lui-même la mention « APERÇU » et une QR-facture barrée, non
   payable (il ne peut pas passer pour la facture émise, même téléchargé ou imprimé) ; une facture figée s'y montre par son
   PDF figé ; le PDF émis reste produit au premier téléchargement depuis la fiche ou au premier envoi ; le bandeau vaut
   à l'ouverture de la visionneuse. Renvoi `\S\ref{sec:pdf-fige}` depuis le paragraphe du Journal. **Les phrases du
   figement restent justes** (téléchargement et envoi demeurent les seuls gestes qui figent) : aucune n'est réécrite ; le
   T4 le constate par `grep -nE "fig[ée]|refig" docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex` sur la base
   de développement, nombre de lignes lues déclaré au Change Log, et toute phrase qui dirait le contraire (« toute
   ouverture fige ») corrigée. **Le refus d'un PDF figé disparu** (`user-manual.tex:1650`) : l'affichage depuis les
   rapports est refusé de même, avec le même message ; le bouton *Refiger* n'apparaît qu'après un téléchargement refusé
   **depuis la fiche** (`invoices/[id]/+page.svelte:926`, `:994`, `:1037`) — une demi-phrase.
   `.tex` seul ; le PDF se régénère à la release. Captures du
   manuel montrant le grand livre (p. ex. `rd-grand-livre-usd.png`) : périmées si la colonne y paraît, à refaire à la
   release (dit au Change Log).

**Ancienne recette visuelle** :

Captures (thème clair, `kesh-gate-logs/s55-3b/`) : journal avec ses icônes ; visionneuse à un justificatif (ouverte
directement, sans liste) ; visionneuse à deux justificatifs, chacun affiché ; bandeau « PDF non figé », avec le PDF
d'aperçu marqué visible dessous (filigrane, QR-facture barrée) ; erreur `410` (obtenue sur une installation jetable en retirant de `KESH_DOCUMENTS_DIR` le fichier `<empreinte>.pdf`
d'une facture figée, ou par `page.route` rendant `410 INVOICE_PDF_GONE`) ; grand livre avec la colonne.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement de la story, sans revue de code).

### Completion Notes List

- **Livré** : `EntryDocuments.svelte` (le dialogue `ui/dialog`, **une instance par vue**, `openFor(entryId, titre,
  élément)` ; l'icône est un snippet exporté du même fichier, `entryDocumentsButton`), `EntryDocumentsViewer.svelte`
  (liste en haut à partir de deux, `DocumentPreview` dessous, sélection par deux primitives, « PDF pas encore émis »
  sans lecture), `entry-documents.ts` (lecteur par genre qui refuse un `Blob` vide, noms de fichier et replis,
  libellés, texte d'erreur, titre ; module pur ajouté pour la testabilité, non nommé par la fiche) ; `hasDocuments` sur
  `JournalEntryRow`/`LedgerLine`, `EntryDocumentKind`/`EntryDocument` en fin de `reports.types.ts` ; `fetchEntryDocuments`,
  `fetchInvoicePdf`, `fetchCreditNotePdf` dans `reports.api.ts` ; `DocumentPreview` gagne **deux** propriétés
  facultatives, `errorMessage` (AC-4, lue dans le `catch` seulement) et `hideThumbnails` (AC-11, repli prévu par
  l'AC-11 : voir R-9) — comportement inchangé sans elles ; Journal : icône sur la ligne d'en-tête
  (`journal-entry-header-<id>`, `journal-entry-number-<id>`, `journal-entry-documents-<id>`) ; Grand livre : colonne
  `documents` après « Libellé », présente si une ligne du **rapport** porte `hasDocuments`, `colspan` dérivés de la liste
  effective (`columns`), icône `ledger-entry-documents-<lineId>` au numéro de `piece()` ; onze clés `reports-nav-*`
  ×4 locales ; manuel (§ 11.6.1, refus d'un PDF disparu, § Grand livre, § Journal des écritures et son paragraphe
  `sec:journal-justificatifs`, « Journaux »).
- **R-9 (vignettes), constaté** : Chromium de Playwright **avec tête** (Google Chrome for Testing 147.0.7727.15, sous
  `xvfb-run`) **montre** le panneau des vignettes sur l'URL `blob:` nue (capture
  `captures-avec-tete/55-3b-3-visionneuse-un-justificatif.png`) : le repli de l'AC-11 est donc pris —
  `hideThumbnails` ajoute `#pagemode=none&navpanes=0`, et Chromium l'honore (aucune vignette,
  `captures-avec-tete-sans-vignettes/55-3b-3-…`, `-4-…`). Sans tête, le cadre reste vide (pas de lecteur PDF), d'où
  les captures avec tête. Les cinq appelants d'avant gardent l'URL nue (Vitest).
- **UI-8** : le Vitest du clic à côté fonctionne sous jsdom (`pointerdown`/`pointerup` avec coordonnées positives, le
  rectangle du contenu étant nul ; rappel bits-ui de 10 ms) — gardé, et l'E2E le constate aussi (`page.mouse.click(5, 5)`).
- **R-6, défaut retenu, en attente de confirmation du mainteneur** : « son prochain téléchargement, ou son prochain envoi
  par e-mail, le figera » (sans « depuis sa fiche ») dans les quatre locales ; manuel « (depuis la fiche ou par l'API) ».
  **Une clé** pour le libellé de l'icône et le titre (`reports-nav-documents-title`, pas de `-button`) : défaut retenu,
  en attente. **Signalé au mainteneur (R-7)** : une facture envoyée avant la v0.12.1 et jamais figée est dite « PDF pas
  encore émis ».
- **Écarts au T0, en cours de développement** : l'erreur « vide » n'est pas une classe `EmptyDocumentError` comme écrit
  au T0, mais une `Error` nommée (`emptyDocumentError()`, reconnue par son nom) : la garde
  `i18n-libelle-en-dur.test.ts` refuse toute classe dans `src/`, et `erreurs-en-texte.test.ts` refuse `instanceof
  Error` (relevés par les gardes au front complet).
- **Rôle Consultation** (AC-4) : `grep -n "role" EntryDocuments.svelte` vide ; routes sous « tout rôle authentifié »
  (`lib.rs:1296`, `:1309`, `:1325`) ; E2E Consultation vert.
- **AC-9, constat** : `grep -cE "fig[ée]|refig"` 56 lignes (`user-manual.tex`) et 21 (`admin-manual.tex`) à la base,
  aucune ne disant qu'une ouverture fige (`admin-manual.tex:1688`, « le premier téléchargement … le fige », reste juste).
  Captures du manuel montrant le grand livre (p. ex. `rd-grand-livre-usd.png`) : périmées si la colonne y paraît, à
  refaire à la release.
- **Tests ajoutés** (recomptés, de `4c19bd76d` à la tête ; déclarations `it(`/`test(`, et tests exécutés) :
  `EntryDocuments.test.ts` 11 (neuf), `EntryDocumentsViewer.test.ts` 12 déclarations, 16 exécutés (deux boucles de
  trois), `GeneralLedgerView.test.ts` 27 → 32 (+5), `reports.api.test.ts` +2, `DocumentPreview.test.ts` 10 → 16 (+6,
  dont l'hôte `DocumentPreviewHost.test.svelte` : sans lui, `rerender` de testing-library renotifie toutes les
  propriétés et relit le document, ce qui n'arrive pas dans l'application) — **40 Vitest exécutés de plus** ; E2E
  `reports-entry-documents.spec.ts` 5 (neuf). `JournalReportView.test.ts` inchangé : l'icône du Journal est testée dans
  `EntryDocuments.test.ts`, monté sur `JournalReportView`.
- **Mutations** (`kesh-gate-logs/s55-dev/55-3b/mutations/`, une à la fois, témoins non mutés verts, « tuée » = test
  nommé en échec sur une assertion) : **17 sur 17 tuées** au passage complet à `48b92e713`
  (`resultats-final-48b92e713.txt`) ; m12 rejouée à `9efde054f` (tuée). Premier passage : m09 et m12 refusées par le
  script (échec par `TestingLibraryElementError`, non par une assertion) → tests réécrits, rejouées tuées. ⚠️ Le
  premier script restaurait par `git checkout`, qui a effacé un travail non commité au second passage (refait à
  l'identique, commité avant tout nouveau passage ; le script restaure désormais depuis une copie). **UI-6** : mutation
  de hauteur (`py-2` sur l'icône) jouée sur l'E2E — l'assertion de hauteur rougit (« Received: 12.09375 », ≤ 1 attendu),
  `mutation-hauteur-e2e.sh`.
- **Gates** (dernier commit de code `9efde054f`), tous par `heavy.sh` : `clippy.sh 55-3b/final` fmt rc=0, clippy rc=0
  (aucun Rust changé) ; `nextest.sh 55-3b/i18n -p kesh-i18n` 42/42 ; `nextest.sh 55-3b/textes -p kesh-api -E
  'binary(textes_coherents)'` 34/34 ; `make user` réussi (`55-3b/manuel-make.log`, 2 passes, 0 référence indéfinie),
  PDF contrôlé par `pdftotext` (copie `55-3b/user-manual-controle.pdf`) puis remis par `git checkout` ;
  `front.sh 55-3b/final` — check 0 erreur (30 avertissements préexistants, aucun dans les fichiers de la story),
  lint-i18n-ownership, Vitest **5585 tests (318 fichiers)**, build : tous rc=0 ; E2E partiel (16 specs : `reports*.spec.ts`
  dont `reports-entry-documents` et `reports-navigation`, `homepage-charts`, `open-items`, `dunning-roundtrip`, `smoke`,
  et les specs des écrans de `DocumentPreview` : `supplier-documents`, `backup-restore-new-install`,
  `session-idle-warning`, `supplier-invoice-unknown-outcome`) — **84 passés, 0 échec** (7,6 min, `55-3b/e2e/`) ; à `48b92e713` (avant le correctif de la garde)
  84/84 (`55-3b/e2e-48b92e713/`). Garde KF-043 (`e2e-selecteurs-traduits.test.ts`) verte. E2E complet et gate-ci en fin
  d'epic.
- **Recette visuelle** (`kesh-gate-logs/s55-dev/55-3b/captures/`, thème clair, sans tête) : `1-journal-icone` (icône
  monochrome à côté de « #1 », ligne à hauteur des autres), `2-pdf-pas-encore-emis` (le message à la place de la
  visionneuse, sans liste), `3-visionneuse-un-justificatif` et `4`/`5` (liste de deux, élément marqué ; cadre vide
  sans tête), `6-erreur-410` (message du serveur en rouge dans le dialogue), `7-grand-livre-colonne` (colonne
  « Justificatifs » après « Libellé »), `8-grand-livre-visionneuse` ; avec tête : `captures-avec-tete/` (vignettes
  visibles, avant le repli) et `captures-avec-tete-sans-vignettes/` (PDF rendu sans vignettes, une et deux factures).

### Issues à ouvrir

- (LOW, `documentation`) Les captures du manuel qui montrent le grand livre (`rd-grand-livre-usd.png` et voisines)
  sont à refaire à la release si la colonne « Justificatifs » y paraît (Epic 55, story 55-3b).
- (LOW, `bug` P4) `frontend/src/lib/components/ui/dialog/dialog-content.svelte:42` porte un `sr-only` « Close » anglais en
  dur pour tout dialogue à bouton par défaut (Epic 55, story 55-3b).

- (LOW, `technical-debt`) Lecture en `Blob` du PDF d'une facture et d'un avoir écrite deux fois : pages des fiches et
  `reports.api.ts` (C-55-3b-2).

### File List

- `frontend/src/lib/features/reports/EntryDocuments.svelte`, `EntryDocumentsViewer.svelte`, `entry-documents.ts` (neufs)
- `frontend/src/lib/features/reports/EntryDocuments.test.ts`, `EntryDocumentsViewer.test.ts` (neufs)
- `frontend/src/lib/features/reports/JournalReportView.svelte`, `GeneralLedgerView.svelte`, `GeneralLedgerView.test.ts`
- `frontend/src/lib/features/reports/reports.types.ts`, `reports.api.ts`, `reports.api.test.ts`
- `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte`, `DocumentPreview.test.ts`,
  `DocumentPreviewHost.test.svelte` (neuf)
- `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal` 3785 → 3796), `i18n-libelle-en-dur.test.ts` (152 → 154)
- `frontend/tests/e2e/reports-entry-documents.spec.ts` (neuf)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`
- `_bmad-output/implementation-artifacts/55-3b-justificatifs-ecran.md`, `sprint-status.yaml`

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
- 2026-11-25 — Validation P5 ciblée (version récrite, Opus) : 0/0/2/4 ; remédiée (dernière remédiation permise à
  l'orchestrateur) : manuel d'administration dans la règle (`:949`, `:1607`, `:1644`) et au fichier d'epic (Vv2p5-1) ;
  phrase propre pour le refus d'un PDF disparu, *Refiger* depuis la fiche seulement (Vv2p5-2) ; LOW Vv2p5-3 à Vv2p5-6
  (décalage inégal, recherche en deux étapes, renvoi hors du § 11.6.1, test du focus et `onCloseAutoFocus`).
- 2026-11-25 — Validation P6 ciblée (version récrite, Sonnet) : 0/0/0/3 — **validation terminée**. Tendance de la version
  récrite : P1 Sonnet 0/0/1/8 → P2 ciblée Haiku 0/0/2/0 → P3 ciblée Sonnet 0/0/1/3 → P4 ciblée Haiku 0/0/2/2 → P5 ciblée
  Opus 0/0/2/4 → P6 ciblée Sonnet 0/0/0/3. Les trois LOW se reprennent au **T0 du développement**, ligne lue citée :
  Vv2p6-1 (`README.md:33` et le commentaire `invoices.types.ts:109` dans la règle du figement), Vv2p6-2 (le manuel dit
  comment l'administrateur refige : depuis la fiche, téléchargement refusé puis *Refiger le document* ;
  `admin-manual.tex:1648`, `:1653` aux repères), Vv2p6-3 (`admin-manual.tex` nommé au § *Modules*, même module ;
  `focus-scope.svelte.js:78-90` ; raison de laisser `:3426` écrite en toutes lettres).
- 2026-11-25 — **Réponses de l'utilisateur versées** (partie A, points 3 à 5) : l'affichage d'une facture depuis les
  Rapports **ne la fige pas** — lecture par l'aperçu de la 55-3a (`/pdf/preview`), bandeau « PDF non figé » sur
  `pdfStatus: 'notFrozen'` (AC-3, AC-5, AC-8) ; la règle de réécriture des phrases du figement au manuel est **retirée**
  (elles restent justes), remplacée par une phrase au § 11.6.1 (AC-9) ; fiche de la facture inchangée (Q8 au mainteneur) ;
  **un seul justificatif : aucune liste**, la visionneuse l'ouvre directement, la liste ne paraît qu'à partir de deux
  (AC-2) ; tests, E2E, captures et note de version alignés. Validation P7 ciblée sur ces changements (dès la P6, plus de
  remédiation : points en R-n).
- 2026-11-25 — Validation Pv2p7 ciblée sur les réponses de l'utilisateur (Opus) : 0/0/1/5. **Arrêt par renvoi au
  développement** (au-delà de la sixième passe) : R-1 (texte du bandeau), R-2 (deux phrases du § 11.6.1, venu de la
  Pv2p3 de la 55-3a) ; LOW listés en tête. Tendance de la version récrite : P1 Sonnet 0/0/1/8 → P2 Haiku 0/0/2/0 → P3
  Sonnet 0/0/1/3 → P4 Haiku 0/0/2/2 → P5 Opus 0/0/2/4 → P6 Sonnet 0/0/0/3 → (réponses de l'utilisateur) P7 Opus 0/0/1/5.
  `ready-for-dev` **sous réserve** du T0 (R-1, R-2) ; la P8 reste en réserve.
- 2026-11-25 — **Décisions du mainteneur versées** (fichier d'epic, § *Décisions du mainteneur*) : Q1 (facture figée :
  PDF figé tel quel, sans marque ni bandeau ; aperçu marqué pour une validée non figée) et Q10 (la marque vient du serveur,
  55-3a D13) à l'AC-5, à l'AC-9 (phrase du § 11.6.1) et à la recette visuelle ; **Q8 refusée** (D14) à l'AC-5 ; Q5 (P4) ;
  Q6 au § *Gates* (story purement écran). Aucune remédiation de passe : la passe suivante, **Pv2p8**, est la dernière
  permise ; au-delà de la sixième, ses points vont au T0 (R-n).
- 2026-11-25 — Validation Pv2p8 ciblée (Sonnet) sur les décisions du mainteneur : 0/0/0/3 — **validation terminée**,
  dernière passe permise ; les trois LOW vont au T0 (R-3 à R-5). `ready-for-dev` sous réserve du T0 (R-1 à R-5).
- 2026-11-26 — **Décisions du mainteneur du 2026-11-26 versées** (fichier d'epic) : l'aperçu marqué sort de l'Epic 55
  (epic « PDF marqués ») ; facture figée → `fetchInvoicePdf` (`GET /pdf`, PDF figé, rien d'écrit) ; toute autre valeur de
  `pdfStatus` → **aucune lecture**, message « PDF pas encore émis » (`reports-nav-documents-not-issued`, quatre locales
  proposées), sans aperçu, qui suit l'élément choisi ; aucune nuance de texte pour un PDF sorti sans être figé (AC-3,
  AC-4, AC-5, AC-8, AC-9, tests, E2E, recette visuelle, T0, T2, note de version) ; anciens textes en section *Reporté à
  l'epic « PDF marqués »* ; R-1, R-2, R-4, R-5 caducs, **R-3 tranché** (§ *Gates* : `nextest -p kesh-i18n` et
  `binary(textes_coherents)` à la clôture). La fiche est au **plafond de huit passes** (version récrite, Pv2p8) : la
  passe ciblée du 2026-11-26 la relit comme contexte de cohérence, et ses points sur cette fiche vont au **T0** (R-n).
- 2026-11-26 — Passe Pv2p7 de la 55-3a (Opus, `kesh-gate-logs/s55-decisions-2/55-3a-validation-pv2p7.md`), qui relit cette fiche pour la
  cohérence (fiche au plafond, la passe ne compte pas pour elle) : 0/0/0/2 sur cette fiche. Au **T0**, avec le mainteneur
  pour le texte visible : **R-6** (Pv2p7-8) — « depuis sa fiche » est plus étroit que le manuel (`user-manual.tex:1641` : une
  clé d'API en lecture fige aussi) ; défaut : « son prochain téléchargement, ou son prochain envoi par e-mail, le figera »,
  et au manuel « au premier téléchargement (depuis la fiche ou par l'API) ou au premier envoi », traductions alignées ;
  **R-7** (Pv2p7-9) — la raison de « aucune nuance » est le vocabulaire de Kesh (« émis » veut dire « figé », manuel
  `:1642`), à écrire telle ; signaler au mainteneur qu'une facture envoyée avant la v0.12.1 se verra promettre un gel dont
  `:1643` dit qu'il peut différer du document reçu (le bandeau daté de l'epic « PDF marqués » le dira).
- 2026-11-26 — **Précisions d'interface de l'utilisateur versées** (transmises par le coordinateur) : AC-1, aspect de
  l'icône (un peu plus grande qu'une icône de texte, hauteur de ligne inchangée, monochrome de la couleur du texte, libellé
  accessible ; constats Vitest et E2E) ; AC-7, trois façons de fermer (« Fermer », clic à côté, Échap ; comportement par
  défaut du dialogue de Kesh, relevé dans bits-ui 2.16.5 ; mutation) ; **AC-11 neuf**, pas de vignettes par défaut (aucun
  paramètre au code aujourd'hui ; repris tel quel, recette dans Chromium, `#pagemode=none` en repli au T0). Maquettes
  `m5a`, `m5b`, `m5c` corrigées et `m5d` (« PDF pas encore émis ») ajoutée hors du dépôt (`kesh-gate-logs/s55-maquettes/`).
  Fiche au plafond de huit passes : la passe ciblée du 2026-11-26 (précisions d'interface) la lit, ses points vont au
  **T0**.
- 2026-11-26 — Passe ciblée sur les précisions d'interface (Opus, `kesh-gate-logs/s55-decisions-2/interface-2026-11-26-validation.md`), hors décompte (fiche au plafond) :
  0/0/3/5. Renvoi au **T0** (R-8 à R-10, en tête) ; LOW UI-4 à UI-8 listés en tête.
- 2026-11-26 (soir) — **D16 versée** (le rapport se charge seul ; un rechargement garde la vue montée) : note à l'AC-5
  et R-11 au T0 ; sans passe de remédiation (fiche au plafond).
- 2026-11-27 — **T0 du développement** (base `epic/55-navigation` à `4c19bd76d`, 55-1a, 55-1b, 55-2 et 55-3a
  développées ; lignes citées à cette base). Chaque point renvoyé, la ligne lue et la décision :
  - **R-1, R-2, R-4, R-5** — caducs (décision du mainteneur du 2026-11-26 : aucun aperçu, aucun bandeau, aucun PDF
    marqué), comme l'écrit la tête de la fiche. **R-3** — tranché (§ *Gates* : `nextest -p kesh-i18n` et
    `binary(textes_coherents)` à la clôture, la story touchant `messages.ftl` et le manuel).
  - **R-6** — `user-manual.tex:1641` « Qui que ce soit qui télécharge le premier --- y compris un utilisateur en
    consultation ou une clé d'API en lecture --- fige le document » : « depuis sa fiche » est trop étroit. **Défaut
    retenu, en attente de confirmation du mainteneur** : `reports-nav-documents-not-issued` = « PDF pas encore émis :
    cette facture n'a pas encore de PDF figé ; son prochain téléchargement, ou son prochain envoi par e-mail, le
    figera. » (traductions alignées : sans « depuis sa fiche ») ; au manuel, « au premier téléchargement (depuis la
    fiche ou par l'API) ou au premier envoi ».
  - **R-7** — `user-manual.tex:1642-1643` (« Une facture validée avant la version v0.12.1 est figée à son prochain
    téléchargement … le document figé n'est pas forcément celui que le client a reçu ») : « aucune nuance » tient au
    vocabulaire de Kesh, où « émis » veut dire « figé » ; **signalé au mainteneur** : une facture envoyée avant la
    v0.12.1 se verra dite « pas encore émise » et promettre un gel qui peut différer du document reçu (le bandeau daté
    de l'epic « PDF marqués » le dira). Aucun texte de plus.
  - **R-8** — `@lucide/svelte` rend `stroke="currentColor"` pour toute icône : le constat porte donc sur l'absence
    de **toute** classe à préfixe de couleur (`text-`, `fill-`, `stroke-`, `bg-`, variantes `hover:`/`focus:`
    comprises) sur le bouton et sur le SVG (Vitest, motif ouvert et non liste), et l'E2E compare
    `getComputedStyle(svg).color` à celle du `#numéro` de la même ligne. Défaut retenu.
  - **R-9, UI-5** — `frontend/playwright.config.ts` sans `headless` ni `channel` : sans tête, Chromium n'a pas de
    lecteur PDF. Décision : la capture des vignettes se fait **avec tête**, Chromium de Playwright (révision 1217)
    sous `xvfb-run`, sur un PDF de facture d'une page ; repli `#pagemode=none` / `navpanes=0` seulement si le
    lecteur montre les vignettes (résultat au Dev Agent Record).
  - **R-10** — tests complétés : Vitest des trois fermetures (Échap, « Fermer », clic à côté) avec focus rendu ;
    E2E : clic sur le fond, hauteur par `boundingBox`, `src` sans fragment, couleur (R-8).
  - **R-11** — `routes/(app)/reports/+page.svelte:1616` `{#if loading && inFlightTab === activeTab}` (premier
    chargement : la vue est démontée) et `:1625` `<div class={reloadingActive ? 'opacity-60' : ''}
    data-testid="report-content">` (rechargement : la vue reste montée, le `dto` est remplacé). Décision : la
    sélection est tenue par deux primitives et l'écriture ouverte est un état d'`EntryDocuments` (identifiant et
    libellé copiés au clic) ; un Vitest remplace le `dto` par un objet égal, dialogue ouvert, et constate une seule
    lecture du document et le titre inchangé.
  - **UI-4** — `DocumentPreview.svelte:92-97` (le `<iframe src={view.url}>`) : référence corrigée.
  - **UI-6** — `JournalReportView.svelte:63` (`<div class="flex justify-between text-sm">`, ligne d'en-tête sans
    `data-testid`) : `data-testid="journal-entry-header-<entryId>"` posé ; la mutation de la mesure (remplissage
    vertical sur le bouton) est jouée sur l'E2E de hauteur (résultat au Dev Agent Record).
  - **UI-7** — Échap ne ferme pas une fois le focus dans le cadre du PDF (autre document) : dit au manuel et à la
    note de version.
  - **UI-8** — bits-ui 2.16.5, `dismissible-layer.svelte:8` `interactOutsideBehavior = "close"` et
    `escape-layer.svelte:8` `escapeKeydownBehavior = "close"` : le Vitest du clic à côté s'écrit en premier
    (`pointerdown` puis `pointerup` sur le fond) ; s'il ne peut pas constater la fermeture sous jsdom, l'E2E est le
    constat et le Vitest est retiré (dit au Dev Agent Record).
  - **Hauteur (Vv2p6, Vv2p7-5)** — `DocumentPreview.svelte:92-97` : `class="h-[75vh] w-full"`, et le précédent
    `LinkSettlementModal.svelte:201` `max-h-[85vh] overflow-y-auto` : titre + liste + cadre de 75vh + « Fermer »
    dépasseraient 85vh. Décision : `DocumentPreview` inchangé (seule `errorMessage` s'ajoute) ; le dialogue borne la
    hauteur du cadre par une variante d'arbre (`[&_iframe]:h-[…]`, plus basse avec la liste), la liste défile seule
    au-delà de quelques éléments ; constat E2E : hauteur du dialogue ≤ 85 % de la fenêtre et aucun défilement
    interne (`scrollHeight ≤ clientHeight`).
  - **Fichier d'epic** — `helpers/api-fixtures.ts` non touché : les aides du règlement groupé vivent dans le spec neuf.
  - **Une instance du dialogue par vue** — tranché (fiche) : `EntryDocuments.svelte` rendu une fois par vue, ouvert
    par `openFor(entryId, label, élément)` ; l'icône de chaque ligne est un snippet exporté du même fichier.
  - **Erreur « vide »** — classe `EmptyDocumentError` (module `entry-documents.ts`), reconnue par `errorMessage`
    **avant** `pdfErrorMessage`. **Une clé** pour le libellé de l'icône et le titre, qui portent le même texte :
    `reports-nav-documents-title` (« Justificatifs de l'écriture n° { $number } ») ; `reports-nav-documents-button`
    n'est pas créée. Défaut retenu, en attente de confirmation du mainteneur.
  - **`pdfErrorMessage` (V3-6)** — `pdf-error.ts:39-41` : hors `ApiError`, son texte générique dit
    « téléchargement » ; `errorMessage` ne l'appelle donc que pour une `ApiError` (le reste retombe sur le texte de
    `DocumentPreview`, « Le document n'a pas pu être affiché »).
  - **E2E Consultation** — exigé (patron `reports.spec.ts:437-460`) : un utilisateur *Consultation* ouvre le PDF figé
    d'une facture depuis le Journal. Constat par construction : routes sous « tout rôle authentifié »,
    `lib.rs:1296` (`/invoices/{id}/pdf`), `:1309` (`/credit-notes/{id}/pdf`), `:1325`
    (`/supplier-invoices/{id}/source-document`).
  - **LOW V3-1 à V3-7** — V3-1 caduc (D12 : les phrases du figement restent justes) ; V3-2 repris (décor à montant
    exact, sans compte désigné, aides neuves dans le spec) ; V3-3 tracé (*Issues à ouvrir*) ; V3-4 repris (mock de
    `$app/navigation`, `page.url()` avant et après à l'E2E) ; V3-5 repris (E2E Consultation) ; V3-6 ci-dessus ; V3-7
    repris (le commentaire « Lettrage est la dernière » de `GeneralLedgerView.svelte:36-39` corrigé).
  - **Vv2p6-1, Vv2p6-2** — caducs depuis D12 (règle de réécriture des phrases du figement retirée) ; **Vv2p6-3** —
    `admin-manual.tex` non touché (aucune phrase n'y devient fausse), `focus-scope.svelte.js:72-90` lu (focus rendu par
    défaut, d'où `onCloseAutoFocus` avec `preventDefault()` et le focus rendu explicitement).
  - **Vv2p7-3** caduc ; **Vv2p7-4** repris (le message suit l'élément choisi) ; **Vv2p7-5** repris (hauteur) ;
    **Vv2p7-6** sans objet.
  - **Manuel, constat de l'AC-9** — `grep -cE "fig[ée]|refig"` : 56 lignes dans `user-manual.tex`, 21 dans
    `admin-manual.tex`, lues à cette base ; aucune ne dit qu'une ouverture fige.
- 2026-11-27 — **Développée** (sans revue de code) : T1 à T5 ; R-9 constaté avec tête, repli de l'AC-11 pris
  (`hideThumbnails`) ; erreur « vide » sans classe (gardes) — annotation du T0 ci-dessus, qui disait « classe
  `EmptyDocumentError` » ; mutations 17/17 et mutation de hauteur E2E tuées ; gates de la story verts (Dev Agent
  Record). Statut `review`.
- **2026-11-26 — revue de code, close (une passe).** Tendance : P1 complète Sonnet **0/0/0/7**. Rien au-dessus de LOW ;
  aucun correctif dans la story. Aucun reclassement.
  Prompts versionnés à côté de la fiche (`<clé>-code-review-prompt-pN.md`, consignes communes `55-code-review-brief-commun.md`) ; rapports hors dépôt, `kesh-gate-logs/s55-revue/<clé>/revue-pN-rapport.md`. Base : `epic/55-navigation` après la fusion de la PR de l'Epic 46 (`c8fe4d0ac`, revue d'intégration P1 Opus 0/0/0/2, LOW corrigés `f16463f91`). LOW non corrigés : issues à ouvrir, `kesh-gate-logs/s55-revue/issues-a-ouvrir.md` (R22 à R26). Gates de fin d'epic (gate-ci et E2E complet) au dernier commit de code de l'epic : voir le fichier d'epic.
