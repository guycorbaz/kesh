# Story 55-3b : ouvrir les justificatifs d'une écriture depuis le Journal (et le Grand livre)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 à D11. Issue : #À-VENIR (point c, comme la
55-3a). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration.** **Suit la 55-3a**
(champ `documents`) **et la 55-2** (mêmes vues).

*Découpée de la 55-3 à la validation P1 (V1-5).*

## Story

En tant que comptable qui relit son journal,
je veux voir d'un clic, depuis la ligne d'une écriture, les PDF qui la justifient,
afin de contrôler une écriture sans ouvrir l'écriture puis la facture.

## Modules touchés (règle de splitting préventif)

Cinq : `frontend/src/lib/features/reports` (composant neuf, deux vues, lecteurs), `frontend/src/lib/features/imported-supplier-invoices`
(une propriété facultative de `DocumentPreview`), `kesh-i18n` (quatre locales), `docs/manual/fr/user-manual.tex`,
`frontend/tests/e2e`. Les pages des fiches de facture et d'avoir **ne sont pas touchées** (voir C-55-3b-2).

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

## Critères d'acceptation

1. **AC-1 — Journal, un justificatif (critère principal).** Une écriture de vente d'une facture validée porte, sur sa ligne
   d'en-tête (`#numéro`, date), un **bouton** icône (`data-testid="journal-entry-documents-<entryId>"`, `aria-label`
   « Justificatifs de l'écriture n° N (1) ») ; un clic ouvre un dialogue modal (étiqueté, focus piégé, *Échap* ferme,
   focus rendu au bouton) qui affiche le PDF par `DocumentPreview`, lu par `GET /api/v1/invoices/{id}/pdf`. Une écriture
   sans justificatif n'a pas de bouton.
2. **AC-2 — Plusieurs justificatifs.** Une écriture à deux justificatifs montre le nombre (2) ; le clic ouvre le dialogue
   sur une **liste** (genre, numéro, nom du fichier d'un document fournisseur) ; choisir un justificatif l'affiche dans le
   même dialogue, avec « Retour à la liste ».
3. **AC-3 — Les trois genres.** Avoir → `GET /api/v1/credit-notes/{id}/pdf` ; document fournisseur →
   `fetchSupplierInvoiceSourceDocument` ; facture → `GET /api/v1/invoices/{id}/pdf`. Noms passés à `DocumentPreview` :
   `facture-<numéro>.pdf`, `avoir-<numéro>.pdf` (l'identifiant si le numéro manque), le `fileName` du document fournisseur.
4. **AC-4 — Erreurs, droits.** Les trois routes d'aujourd'hui, sans route neuve, leurs droits inchangés (tout rôle
   authentifié : un rôle *Consultation* voit le bouton et ouvre le justificatif). `DocumentPreview` gagne une propriété
   **facultative** `errorMessage?: (err: unknown) => string` : présente, elle rend le texte de l'erreur ; absente, le
   comportement d'aujourd'hui (aucun changement pour ses appelants actuels, `DocumentPreview.test.ts` vert sans
   modification). Facture et avoir : `pdfErrorMessage` (`410 INVOICE_PDF_GONE` nomme le fichier à restaurer) ; document
   fournisseur : sans la propriété (« Le document n'a pas été restauré » pour `SOURCE_DOCUMENT_GONE`). Le journal reste
   affiché.
5. **AC-5 — Figement.** Ouvrir la facture d'une écriture la **fige** si elle ne l'était pas (route d'aujourd'hui,
   `Usage::Download`), y compris pour un rôle *Consultation* : dit au manuel (AC-9). La fonction `load` passée à
   `DocumentPreview` est **créée une fois par justificatif choisi** (mémoïsée), pour qu'un nouveau rendu ne relise pas le
   document (ni ne relance la route qui fige) : test Vitest qui compte les lectures.
6. **AC-6 — Grand livre.** Chaque ligne d'écriture porte le même bouton, dans une colonne étroite après « Libellé »
   (`data-testid="ledger-entry-documents-<accountNumber>-<entryId>"` : une écriture paraît dans plusieurs sections), même
   dialogue. *Option* : celle de la 55-3a.
7. **AC-7 — URL et navigation.** Ouvrir le dialogue n'écrit rien dans l'URL (55-1, D2) ; une navigation (Retour) le ferme.
   Un rapport repris de la mémoire (55-1, D3) peut porter un justificatif devenu introuvable : l'erreur s'affiche dans le
   dialogue (AC-4).
8. **AC-8 — Textes.** Bloc `reports-nav-*` des quatre locales : libellé du bouton (numéro d'écriture et nombre, message
   Fluent à arguments), titre du dialogue (« Justificatifs de l'écriture n° N »), genres (« Facture », « Avoir », « Document
   fournisseur »), « Retour à la liste », « Fermer » (une clé existante réutilisée si elle a ce texte, relevé au T0) ;
   `i18n-keys` (`sitesTotal` par la garde) et `i18n-libelle-en-dur` verts.
9. **AC-9 — Manuel.** `docs/manual/fr/user-manual.tex` : § *Journal des écritures* (`:4316`) — le bouton, les trois genres,
   le règlement qui montre la facture réglée, la liste ; § *Grand livre* (`:4184`) — une phrase ; § 11.6.1
   (`sec:pdf-fige`, `:1635-1637`) — « au premier téléchargement, au premier envoi, ou au premier affichage depuis les
   rapports », avec renvoi `\S\ref{sec:pdf-fige}` depuis les deux paragraphes des rapports ; le chemin « Mensuel →
   Rapports → Journal » (`:4318`) vérifié contre l'onglet réel (« Journaux »). `.tex` seul, PDF aplati contrôlé avant.
10. **AC-10 — Accessibilité.** Bouton au clavier ; dialogue `role="dialog"`, `aria-modal`, étiqueté ; axe à zéro violation
    sur le Journal avec le dialogue ouvert (E2E, `AxeBuilder`, précédent `tests/e2e/auth.spec.ts:2`).

## Tests à écrire, et la preuve du rouge

- **Vitest** : `EntryDocuments.test.ts` (neuf : aucun bouton sans `documents` ; un justificatif → aperçu direct ; deux →
  liste puis aperçu puis retour ; erreur `410` → message de `pdfErrorMessage` ; une lecture par justificatif choisi malgré
  deux rendus) ; `DocumentPreview.test.ts` (propriété `errorMessage` présente et absente) ; `JournalReportView.test.ts` et
  `GeneralLedgerView.test.ts` (bouton présent avec `documents`, absent sans). Rouges sans le correctif (journal
  `kesh-gate-logs/s55-3b/`).
- **E2E** `frontend/tests/e2e/reports-entry-documents.spec.ts` (neuf ; fichier propre, déclaré au tableau des fichiers
  communs), **cible : le Journal** : créer et valider une facture par l'API, ouvrir *Rapports → Journaux* sur la période,
  cliquer le bouton de l'écriture de vente → l'`iframe` de l'aperçu apparaît ; *Échap* ferme, focus rendu ; axe à zéro
  violation dialogue ouvert. **Deux justificatifs** : une écriture à deux justificatifs obtenue **par l'API** — le chemin
  est fixé au T0 parmi le règlement client groupé (réconciliation, Epic 43) et le règlement fournisseur groupé ; si aucun
  n'est atteignable par l'API dans un E2E, l'AC-2 est couvert par Vitest et par le test Rust de la 55-3a, **écrit comme
  angle mort assumé** au Change Log (pas de repli à deux écritures, qui n'ouvrirait jamais la liste). Grand livre : un test
  court (bouton, aperçu).

## Recette visuelle

Captures (thème clair, `kesh-gate-logs/s55-3b/`) : journal avec boutons (un et plusieurs justificatifs) ; dialogue sur un
PDF de facture ; dialogue en liste ; erreur `410` ; grand livre avec boutons.

## Tâches

- [ ] **T0** — Base `epic/55-navigation` avec la 55-2 et la 55-3a ; chemin de l'E2E à deux justificatifs ; clés i18n
      réutilisables ; lignes du manuel sur la base PR 46.
- [ ] **T1** — Vitest rouges.
- [ ] **T2** — `DocumentPreview` (`errorMessage`) ; lecteurs `fetchInvoicePdf(id)`, `fetchCreditNotePdf(id)` dans
      `reports.api.ts` ; `EntryDocuments.svelte` (bouton, dialogue, liste, `load` mémoïsée par justificatif).
- [ ] **T3** — Vues du journal et du grand livre ; types `reports.types.ts`.
- [ ] **T4** — i18n, manuel, E2E, captures.
- [ ] **T5** — Gates (fichier d'epic, § *Gates*).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E de la story et
des Rapports ; suite complète en fin d'epic.

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
