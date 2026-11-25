# Story 55-3b : ouvrir les justificatifs d'une écriture depuis le Journal (et le Grand livre)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 à D11. Issue : #À-VENIR (point c, comme la
55-3a). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration.** **Suit la 55-3a**
(champ `documents`), **la 55-2** (mêmes vues) **et la 55-1** (URL, mémoire des rapports).

*Découpée de la 55-3 à la validation P1 (V1-5).*

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

- Le contrat de la 55-3a : `documents?: EntryDocument[]` sur `JournalEntryRow` et `LedgerLine`, `EntryDocument { kind:
  'invoice' | 'creditNote' | 'supplierDocument'; id: number; number: string | null; fileName?: string }`.
- `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` : `Props { load: () => Promise<Blob>;
  fileName: string }` (`:23-28`) ; un `$effect` relit le document **à chaque nouvelle fonction `load`** ; l'erreur de `load`
  est rendue par deux textes fixes — `SOURCE_DOCUMENT_GONE` → « Le document n'a pas été restauré », tout le reste → « Le
  document n'a pas pu être affiché » (`:56-61`, `:81-83`) ; PDF en `<iframe>` d'une URL `blob:`, image en `<img>`, autre type :
  lien « Télécharger » ; CSP `blob:` autorisée (`kesh-api/src/middleware/csp.rs:37-38`).
- `frontend/src/lib/shared/utils/pdf-error.ts` : `pdfErrorMessage` rend le message du serveur pour
  `INVOICE_TOO_MANY_LINES_FOR_PDF` et `INVOICE_PDF_GONE` (`:36`), un générique sinon (`:41`) — propre aux PDF de facture.
- Lectures en `Blob` d'aujourd'hui : `fetchSupplierInvoiceSourceDocument(id)` (`imported-supplier-invoices.api.ts:119`) ;
  facture et avoir, écrites **dans les pages** : `apiClient.getBlob('/api/v1/invoices/${id}/pdf')`
  (`invoices/[id]/+page.svelte:969`, nom `facture-${number ?? id}.pdf`), `…/credit-notes/${id}/pdf`
  (`credit-notes/[id]/+page.svelte:94`).
- Le détail d'une écriture montre déjà le document d'une facture fournisseur (`journal-entries/[id]/+page.svelte:589-620`).
- Manuel : § 11.6.1 « Le PDF émis est figé » (`sec:pdf-fige`, `user-manual.tex:1635-1645`) — produit « au premier
  téléchargement ou au premier envoi par e-mail », « qui que ce soit qui télécharge le premier — y compris un utilisateur en
  consultation » ; § *Grand livre* (`:4184`, « avec sa date, sa pièce, son journal », `:4192`) ; § *Journal des écritures*
  (`:4316-4318`, deux lignes).

- Composants : `frontend/src/lib/components/ui/dialog` (bits-ui, employé par `ManualMatchModal`) ; `dialog-content.svelte`
  porte `showCloseButton = true` par défaut, avec un `<span class="sr-only">Close</span>` **anglais en dur**, et la classe
  `sm:max-w-sm` (`:16`, `:31`, `:37`). `DocumentPreview` : `data-testid` `document-preview`, `-loading`, `-gone`, `-error`,
  `-image`, `-frame`, `-download` (`:71-103`) ; son `<iframe>` fait `h-[75vh]`.
- `GeneralLedgerView.svelte` : colonnes déclarées dans `COLUMNS` (`:41-51`), libellés par un `switch` exhaustif
  (`columnLabel`, `:68-88`), commentaire « Lettrage est la dernière … » (`:35-37`) ; `GeneralLedgerView.test.ts`
  (« 15-1c-ii test 3 ») attend **9** en-têtes ; `piece()` rend le numéro d'écriture préfixé de l'exercice.
- Le numéro d'écriture repart à 1 à chaque exercice ; le journal est rendu pour **un** exercice (`fiscalYearId`), le grand
  livre peut en traverser plusieurs.
- Une écriture à deux justificatifs s'obtient **par l'API** : `POST /api/v1/reconciliation/accept` avec une proposition
  `invoice_group` (règlement client groupé, `routes/reconciliation_group.rs:164`) ; patron et aides dans
  `frontend/tests/e2e/reconciliation-group.spec.ts` et `tests/e2e/helpers/api-fixtures.ts` ; `InvoiceResponse.journalEntryId`
  désigne l'écriture de vente.
- Gardes i18n : `i18n-keys` (`sitesTotal`), `i18n-libelle-en-dur`, `i18n-un-repli-par-cle`, `lint-i18n-ownership` (une clé
  d'un autre domaine, p. ex. `journal-entries-document-preview-close`, est refusée depuis `features/reports`) ; `i18nMsg`
  ne gère pas les pluriels Fluent : variables simples seulement.

## Critères d'acceptation

1. **AC-1 — Journal, un justificatif (critère principal).** Une écriture de vente d'une facture validée porte, sur sa ligne
   d'en-tête (`#numéro`, date), un **bouton** icône (`data-testid="journal-entry-documents-<entryId>"`, `aria-label`
   « Justificatifs de l'écriture n° N : 1 ») ; un clic ouvre un dialogue modal qui affiche le PDF par `DocumentPreview`, lu
   par `GET /api/v1/invoices/{id}/pdf`. Une écriture sans justificatif n'a pas de bouton.
2. **AC-2 — Plusieurs justificatifs.** Une écriture à deux justificatifs : `aria-label` « … : 2 », le nombre visible ; le
   dialogue s'ouvre sur une **liste** (genre, numéro, nom du fichier d'un document fournisseur) ; choisir un justificatif
   l'affiche dans le même dialogue, avec « Retour à la liste ».
3. **AC-3 — Les trois genres et leurs lecteurs.** `fetchInvoicePdf(id)` → `GET /api/v1/invoices/{id}/pdf` ;
   `fetchCreditNotePdf(id)` → `GET /api/v1/credit-notes/{id}/pdf` (les deux dans `reports.api.ts`, par `apiClient.getBlob`) ;
   document fournisseur → `fetchSupplierInvoiceSourceDocument(id)`. Noms passés à `DocumentPreview` :
   `facture-<numéro>.pdf`, `avoir-<numéro>.pdf`, à défaut du numéro `facture-<id>.pdf`, `avoir-<id>.pdf` ; le `fileName` du
   document fournisseur, à défaut `justificatif-<id>`. Dans la liste, un numéro nul s'affiche par l'identifiant. Un `Blob`
   vide rend l'état d'erreur avec le message `reports-nav-documents-empty` (« Le document reçu est vide. »).
4. **AC-4 — Erreurs, droits.** Les trois routes d'aujourd'hui, sans route neuve, leurs droits inchangés (un rôle
   *Consultation* voit le bouton et ouvre le justificatif). `DocumentPreview` gagne une propriété **facultative**
   `errorMessage?: (err: unknown) => string` : **présente**, toute erreur de `load` rend l'état `document-preview-error` avec
   ce texte (y compris `SOURCE_DOCUMENT_GONE`) ; **absente**, le comportement d'aujourd'hui. Les tests existants de
   `DocumentPreview.test.ts` passent **sans modification** ; des tests y sont **ajoutés** (propriété présente, absente).
   Facture et avoir : `errorMessage = pdfErrorMessage` (`410 INVOICE_PDF_GONE` nomme le fichier à restaurer) ; document
   fournisseur : sans la propriété. Le journal reste affiché.
5. **AC-5 — Figement et lecture unique.** Ouvrir la facture d'une écriture la **fige** si elle ne l'était pas (route
   d'aujourd'hui, `Usage::Download`), y compris pour un rôle *Consultation* : dit au manuel (AC-9). Le justificatif choisi
   est tenu par **deux primitives** (`kind`, `id`), et la fonction `load` est dérivée de ces seules primitives : remplacer
   `documents` par un tableau d'objets égaux (rapport repris ou redemandé pendant que le dialogue est ouvert) ne relit pas
   le document.
6. **AC-6 — Grand livre.** Une colonne `documents` (`COLUMNS`, après `description`, en-tête `reports-nav-ledger-column-documents`
   « Justificatifs », rendu étroit), **présente seulement si au moins une ligne du rapport a des justificatifs** ; chaque
   ligne qui en a porte le bouton (`data-testid="ledger-entry-documents-<lineId>"`, unique), `aria-label` avec le numéro
   préfixé de l'exercice (`piece()`), même dialogue. Le commentaire `:35-37` est corrigé. Sans justificatif dans le rapport,
   le test « 15-1c-ii test 3 » (9 en-têtes) reste vert sans modification ; un test neuf compte 10 en-têtes avec un
   justificatif. *Option* : celle de la 55-3a.
7. **AC-7 — Dialogue.** `components/ui/dialog` avec `showCloseButton={false}`, un bouton « Fermer » propre
   (`reports-nav-documents-close`), une largeur qui tient l'aperçu (`sm:max-w-4xl`) et un contenu qui défile ; titre
   `reports-nav-documents-title` (« Justificatifs de l'écriture n° { $number } ») ; *Échap* et « Fermer » ferment, focus
   rendu au bouton d'origine. Ouvrir le dialogue n'appelle pas `goto` (55-1, D2 : l'URL ne change pas). Une navigation qui
   change le rapport affiché (Retour vers un autre onglet) démonte la vue et ferme le dialogue. Un rapport repris de la
   mémoire (55-1, D3) peut porter un justificatif devenu introuvable : l'erreur s'affiche dans le dialogue (AC-4).
8. **AC-8 — Textes.** Clés neuves du bloc `reports-nav-*`, quatre locales : `reports-nav-documents-button` (« Justificatifs
   de l'écriture n° { $number } : { $count } »), `reports-nav-documents-title`, `reports-nav-documents-kind-invoice`
   (« Facture »), `-kind-credit-note` (« Avoir »), `-kind-supplier-document` (« Document fournisseur »),
   `reports-nav-documents-back` (« Retour à la liste »), `reports-nav-documents-close` (« Fermer »),
   `reports-nav-documents-empty`, `reports-nav-ledger-column-documents` ; constat : `i18n-keys` (`sitesTotal` recompté par
   la garde, parité des quatre locales), `i18n-libelle-en-dur`, `i18n-un-repli-par-cle`, `npm run lint-i18n-ownership`
   verts.
9. **AC-9 — Manuel.** `docs/manual/fr/user-manual.tex` (lignes à `1e23edb55`) : § *Journal des écritures* (`:4316`) — le
   bouton, les trois genres, le règlement qui montre la facture réglée, la liste ; § *Grand livre*, l'énumération des
   colonnes (`:4193-4194`) gagne « ses justificatifs » ; § 11.6.1 (`sec:pdf-fige`, `:1635-1637`) — « au premier
   téléchargement, au premier envoi, ou au premier affichage depuis les rapports », renvoi `\S\ref{sec:pdf-fige}` depuis
   les deux paragraphes des rapports ; le chemin « Mensuel → Rapports → Journal » (`:4318`) contrôlé contre l'onglet réel
   (« Journaux »). `.tex` seul. Constat : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o
   'premier téléchargement.\{0,120\}'` avant (journal) ; après la régénération à la release, la phrase neuve. Captures du
   manuel montrant le grand livre (p. ex. `rd-grand-livre-usd.png`) : périmées si la colonne y paraît, à refaire à la
   release (dit au Change Log).
10. **AC-10 — Accessibilité.** Bouton au clavier ; dialogue `role="dialog"`, `aria-modal`, étiqueté par son titre ; axe
    à zéro violation, dialogue ouvert : `new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).include('[role="dialog"]')`
    (modèle : les specs axe existantes de `tests/e2e/`).

## Tests à écrire, et la preuve du rouge

- **Vitest** `EntryDocuments.test.ts` (neuf) : aucun bouton sans `documents` ; un justificatif → aperçu direct ; deux →
  liste, aperçu, retour ; les trois genres (lecteur appelé, nom de fichier, replis de l'AC-3) ; erreur `410` → texte de
  `pdfErrorMessage` ; `Blob` vide ; **lecture unique** : dialogue ouvert, `documents` remplacé par un tableau d'objets
  égaux → toujours **une** lecture (mutation déclarée au journal : sélection tenue par l'objet → deux lectures, test
  rouge) ; ouverture sans appel de `goto` (mock de `$app/navigation`).
- **Vitest** `reports.api.test.ts` : `fetchInvoicePdf`, `fetchCreditNotePdf` (URL exactes).
- **Vitest** `DocumentPreview.test.ts` : tests ajoutés (propriété présente, y compris `SOURCE_DOCUMENT_GONE` ; absente).
- **Vitest** `JournalReportView.test.ts` et `GeneralLedgerView.test.ts` : bouton présent avec `documents`, absent sans ;
  colonne présente (10 en-têtes) ou absente (9).
- Rouges sans le correctif, journal `kesh-gate-logs/s55-3b/`.
- **E2E** `frontend/tests/e2e/reports-entry-documents.spec.ts` (neuf, déclaré au tableau des fichiers communs), **cible :
  le Journal** : créer et valider une facture par l'API, ouvrir *Rapports → Journaux* sur la période, cliquer le bouton de
  l'écriture de vente (`journalEntryId` de la facture) → `document-preview-frame` visible ; *Échap* ferme, focus rendu ;
  axe à zéro violation dialogue ouvert. **Deux justificatifs** : deux factures validées, puis un règlement groupé par
  `POST /api/v1/reconciliation/accept` (`invoice_group`, patron `reconciliation-group.spec.ts`) → le bouton de l'écriture
  de règlement ouvre la liste de deux. Retour : dialogue ouvert, onglet changé puis `page.goBack()` → dialogue absent.
  Grand livre : un test court (colonne, bouton, aperçu). Le test du Journal rougit sans le correctif (aucun bouton).

## Recette visuelle

Captures (thème clair, `kesh-gate-logs/s55-3b/`) : journal avec boutons (un et plusieurs justificatifs) ; dialogue sur un
PDF de facture ; dialogue en liste ; erreur `410` ; grand livre avec la colonne.

## Tâches

- [ ] **T0** — Base `epic/55-navigation` avec la 55-1, la 55-2 et la 55-3a ; lignes du manuel sur la base PR 46.
- [ ] **T1** — Vitest rouges.
- [ ] **T2** — `DocumentPreview` (`errorMessage`) ; lecteurs dans `reports.api.ts` ; `EntryDocuments.svelte` (bouton,
      dialogue `ui/dialog`, liste, sélection par primitives).
- [ ] **T3** — Vues du journal et du grand livre (colonne conditionnelle) ; types `reports.types.ts`.
- [ ] **T4** — i18n, manuel, E2E, captures.
- [ ] **T5** — Gates (fichier d'epic, § *Gates*).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E de la story et
des Rapports ; suite complète selon le § *Gates* du fichier d'epic (Q6).

## Note de version

*Rapports → Journaux* et *Grand livre* : un bouton sur une écriture ouvre ses justificatifs — PDF de la facture, de
l'avoir, document de la facture fournisseur ; pour un règlement, la facture réglée. Ouvrir ainsi une facture dont le PDF
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
