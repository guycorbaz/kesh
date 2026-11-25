# Story 55-3 : les pièces d'une écriture depuis le Journal (et le Grand livre)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 (Q1), D10 (Q4), D11. Issue : #À-VENIR
(`enhancement`, priorité à fixer, Q5 ; point c, convenu au téléphone le 2026-11-25 entre l'utilisateur et le mainteneur).
Release : la prochaine planifiée après la 0.20.0. **Aucune migration. ⚠️ Touche le backend** (`kesh-report`) : un champ
additif dans la réponse JSON du journal et du grand livre — l'epic était annoncé « frontend seulement » ; **à confirmer par
le mainteneur (Q1)** avant le développement. **Suit la 55-2** pour les vues (fichiers communs) ; la partie serveur peut
commencer avant.

## Story

En tant que comptable qui relit son journal,
je veux voir d'un clic, depuis la ligne d'une écriture, les PDF qui la justifient — facture, avoir, document de la facture
fournisseur —,
afin de contrôler une écriture sans ouvrir l'écriture puis la facture.

## Où l'icône a un sens

| Rapport | Lignes | Icône |
|---|---|---|
| **Journal** (`JournalReportView.svelte`) | une écriture (`entryId`, numéro, date, libellé, lignes) | **oui — le critère principal et la cible de l'E2E** (demande de l'utilisateur) |
| **Grand livre** (`GeneralLedgerView.svelte`) | une ligne d'écriture d'un compte (`LedgerLine.entryId`) | **oui**, même composant, même champ : coût marginal (une colonne, un appel de la même fonction serveur) ; *option écrite* : si la revue de la fiche ou le T0 montre un coût au-delà (p. ex. un conflit avec l'Epic 47 dans `general_ledger.rs`), le Grand livre sort de la story et s'ouvre en issue |
| Bilan, Compte de résultat, Balance | un **compte** (solde de nombreuses écritures) | **non** : une ligne de compte n'a pas de pièce ; le chemin est le lien de compte (55-2) puis le grand livre |
| TVA | un **taux** ou un total | **non**, même raison |
| Balance âgée, rapports par projet | un contact, un compte analytique | **non** (hors demande) |

## Ce qui se lit au code (`origin/main` `1e23edb55`)

**Aucune table de pièces jointes.** Trois documents se rattachent à une écriture, par quatre colonnes :

| Document | Rattachement à l'écriture | Route de lecture (droits) |
|---|---|---|
| PDF **figé** d'une facture client | `invoices.journal_entry_id` (validation, `migrations/20260417000001_invoice_validation.sql`) ; colonnes `pdf_storage_path`, `pdf_sha256`, `pdf_frozen_at`, `pdf_language` (`20261003000001_invoices_frozen_pdf.sql`) | `GET /api/v1/invoices/{id}/pdf` (`kesh-api/src/lib.rs:1262`, tout rôle authentifié) — **fige au premier rendu** d'une facture validée (`routes/invoice_pdf.rs`, `issued_invoice_pdf::get_or_freeze`, `Usage::Download`) ; `410 INVOICE_PDF_GONE` si le fichier manque |
| idem, écriture de **règlement** | `invoice_settlements.journal_entry_id` → `invoice_id` ; **plusieurs factures par écriture** depuis `20261118000001_grouped_settlements.sql` (clé unique `(journal_entry_id, invoice_id)`) | idem |
| PDF d'un **avoir** | `credit_notes.journal_entry_id` (`20260627000001_credit_notes.sql:36`, renseigné quand `status = 'issued'`) | `GET /api/v1/credit-notes/{id}/pdf` (`lib.rs:1275`, tout rôle) |
| **Document** d'une facture fournisseur (PDF ou image importés) | `supplier_invoices.purchase_journal_entry_id` ou `settlement_journal_entry_id` ← `imported_supplier_invoices.supplier_invoice_id`, `status = 'completed'` (`20260628000001`, `20260629000001`) | `GET /api/v1/supplier-invoices/{id}/source-document` (`lib.rs:1291`, tout rôle, Story 30-2c AC22) |

- **Précédent du même rattachement** : `kesh-db/src/repositories/imported_supplier_invoices.rs:409`
  `find_source_document_of_entry` — achat **ou** règlement, `completed`, une seule écriture ; le détail d'une écriture
  (`routes/journal_entries.rs:250,512`, `sourceDocument`) et son écran (`routes/(app)/journal-entries/[id]/+page.svelte:589-620`,
  `DocumentPreview`) l'affichent déjà pour les factures fournisseur. Un règlement annulé n'est plus rattaché ; l'achat d'une
  facture annulée l'est toujours (test `cancelled_invoices_take_a_document_and_the_detached_settlement_loses_it`). La 55-3
  **suit les mêmes règles**, pour toutes les écritures d'une période et pour les trois documents.
- **Visionneuse existante** : `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` (`load: () =>
  Promise<Blob>`, `fileName`) — `<iframe>` d'une URL `blob:` pour un PDF, `<img>` pour une image, lien « Télécharger » pour
  tout autre type ; CSP `blob:` autorisée (`kesh-api/src/middleware/csp.rs`). Lecteurs en `Blob` : `fetchSupplierInvoiceSourceDocument`
  (`imported-supplier-invoices.api.ts:119`) ; facture et avoir : `apiClient.getBlob('/api/v1/invoices/${id}/pdf')`
  (`invoices/[id]/+page.svelte:969`), `…/credit-notes/${id}/pdf` (`credit-notes/[id]/+page.svelte:94`) ; messages d'erreur
  `pdfErrorMessage` (`shared/utils/pdf-error`).
- **DTO des rapports** : `JournalEntryRow { entry_id, entry_number, entry_date, description, lines }`
  (`kesh-report/src/journal_report.rs:41`) ; `LedgerLine { entry_id, … counterpart … }` (`general_ledger.rs:245`) ; **aucun
  champ de pièce**. Les routes rendent ces structures en JSON telles quelles (`routes/reports.rs:363-366`, `:285-291`). Le
  grand livre lit déjà ses contreparties **en une requête** (`fetch_counterparts`, `general_ledger.rs:636-660`, « surtout pas
  une sous-requête corrélée par ligne ») : le patron à suivre. L'outil MCP du grand livre recopie ses champs dans sa
  propre structure (`mcp/outils/grand_livre.rs:175`) : un champ ajouté à `LedgerLine` n'y passe pas. Le journal n'a pas
  d'outil MCP.

## Le contrat (D9)

`JournalEntryRow` et `LedgerLine` gagnent :

```rust
/// Les pièces de l'écriture — omis quand il n'y en a aucune (JSON identique à
/// aujourd'hui pour ces écritures). Écran seulement : PDF et CSV inchangés.
#[serde(skip_serializing_if = "Vec::is_empty", default)]
pub documents: Vec<EntryDocument>,
```

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntryDocument {
    /// `invoice` | `creditNote` | `supplierDocument`.
    pub kind: EntryDocumentKind,
    /// Identifiant pour la route de lecture : facture, avoir, facture fournisseur.
    pub id: i64,
    /// Numéro affiché : facture, avoir, numéro interne de la facture fournisseur (ou le sien).
    pub number: Option<String>,
    /// Nom du fichier importé (document fournisseur seulement).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}
```

- **Une requête par rapport** : une fonction `kesh-report/src/pieces.rs` (neuve), `documents_by_entry(pool, company_id,
  from, to) -> HashMap<i64, Vec<EntryDocument>>`, un `SELECT … UNION ALL …` des quatre rattachements, joints à
  `journal_entries` et bornés par `je.company_id = ?` et `je.entry_date BETWEEN ? AND ?` — **aucune liste `IN` de taille
  variable, aucune requête par écriture**. Ordre stable : par écriture, puis genre (`invoice`, `creditNote`,
  `supplierDocument`), puis identifiant. Le journal l'appelle sur sa période ; le grand livre sur `from`/`to`.
- Chaque branche porte le filtre de société **sur la table du document aussi** (défense en profondeur, KF-002).
- Avoirs : `status = 'issued'` ; documents fournisseur : `status = 'completed'` ; factures : `journal_entry_id` non nul
  (validées). Règlements : `invoice_settlements` tel qu'il est (un règlement annulé est-il supprimé ou marqué ? **relevé au
  T0**, et la règle de `find_source_document_of_entry` — un règlement détaché ne montre plus la pièce — appliquée de même).
- **Exports PDF et CSV, outil MCP : inchangés** (les rendus ne lisent pas le champ ; test d'égalité des exports existants).

## Critères d'acceptation

1. **AC-1 — Journal, une pièce (critère principal).** Une écriture de vente d'une facture validée montre, sur sa ligne
   d'en-tête (`#numéro`, date), une icône de pièce (`data-testid="entry-documents-<entryId>"`, `aria-label` « Pièces de
   l'écriture n° N (1) ») ; un clic ouvre une fenêtre (dialogue modal, focus piégé, *Échap* ferme) qui affiche le PDF de la
   facture par `DocumentPreview`, chargé par `GET /api/v1/invoices/{id}/pdf`. Une écriture sans pièce n'a pas d'icône.
2. **AC-2 — Journal, plusieurs pièces.** Un règlement groupé de deux factures montre l'icône avec le nombre (2) ; le clic
   ouvre la fenêtre sur une **liste** (genre, numéro, nom du fichier pour un document fournisseur) ; choisir une pièce
   l'affiche dans la même fenêtre, avec un retour à la liste.
3. **AC-3 — Les trois genres.** Avoir émis → son PDF (`/credit-notes/{id}/pdf`) ; facture fournisseur à document, écriture
   d'achat et de règlement → le document (`/supplier-invoices/{id}/source-document`) ; écriture de règlement d'une facture
   client → le PDF de la facture (D10). Une facture fournisseur sans document : aucune pièce.
4. **AC-4 — Droits inchangés.** L'ouverture passe par les trois routes d'aujourd'hui, sans route neuve, avec leurs droits
   (tout rôle authentifié) ; un rôle *Consultation* voit l'icône et ouvre la pièce comme sur la fiche. Une pièce
   introuvable (`404`, `410 INVOICE_PDF_GONE`) affiche le message de `pdfErrorMessage` dans la fenêtre, sans casser le
   journal. ⚠️ Ouvrir la facture d'une écriture la **fige** si elle ne l'était pas encore — comportement de la route
   d'aujourd'hui, le même qu'un téléchargement depuis la fiche : dit au manuel.
5. **AC-5 — Grand livre.** Chaque ligne d'écriture du grand livre porte la même icône (colonne étroite après la pièce),
   le même composant, le même dialogue. *Option* : voir § *Où l'icône a un sens*.
6. **AC-6 — Une requête, bornée à la société.** Test d'intégration (`crates/kesh-api/tests/reports_e2e.rs`, ou un test
   `kesh-report` sur base) : deux sociétés, chacune avec facture, avoir, règlement groupé, facture fournisseur à document ;
   le journal de A ne porte **que** les pièces de A ; une écriture hors période n'est pas lue ; le nombre de requêtes de
   `documents_by_entry` est **un** (structure du code : un seul `query_as`, revu ; aucun appel dans une boucle).
7. **AC-7 — Contrat additif.** Une écriture sans pièce sérialise **exactement** le JSON d'aujourd'hui (test sur la
   réponse d'un journal sans pièce, comparée champ à champ) ; PDF et CSV du journal et du grand livre : octets ou lignes
   de texte identiques à avant (tests existants verts, plus un test d'égalité sur un journal **avec** pièces) ; sortie MCP
   du grand livre inchangée (`mcp_contrat` vert).
8. **AC-8 — Règles de rattachement.** Tests `kesh-report` : avoir brouillon (pas de pièce), avoir émis (pièce) ; document
   importé `to_complete` ou `discarded` (pas de pièce) ; règlement annulé (pas de pièce, règle du T0) ; écriture d'achat
   d'une facture fournisseur annulée (pièce, comme `find_source_document_of_entry`) ; ordre stable.
9. **AC-9 — Textes.** Bloc `reports-nav-*` des quatre locales : libellé de l'icône (avec nombre et numéro d'écriture), titre
   de la fenêtre, genres (« Facture », « Avoir », « Document fournisseur »), « Retour à la liste », « Fermer » (réutiliser
   une clé existante si elle existe, relevé au T0) ; `i18n-keys` (`sitesTotal` par la garde) et `i18n-libelle-en-dur` verts.
10. **AC-10 — Manuel.** § *Journal des écritures* (`user-manual.tex:4316`, aujourd'hui deux lignes) et § *Grand livre*
    (`:4184`) : l'icône, ce qu'elle ouvre (les trois genres, le règlement qui montre la facture), le figement d'une facture
    au premier affichage. `.tex` seul.
11. **AC-11 — Accessibilité.** Icône = bouton (`<button>`, clavier), dialogue étiqueté, focus rendu à l'icône à la
    fermeture ; axe à zéro violation sur la page du journal avec la fenêtre ouverte.

## Tests à écrire, et la preuve du rouge

- **Rust** (`kesh-report`, `#[sqlx::test]` sur le squash `test-schema/`, comme les tests voisins) : AC-6, AC-8 ; sérialisation
  (AC-7). Rouges sans le correctif (le champ n'existe pas : échec de compilation des tests écrits d'abord, puis assertions) ;
  journal `kesh-gate-logs/s55-3/`.
- **Vitest** : `EntryDocuments.test.ts` (neuf : icône absente, une pièce → aperçu direct, plusieurs → liste, erreur `410`),
  `JournalReportView.test.ts` et `GeneralLedgerView.test.ts` (icône présente avec `documents`, absente sans).
- **E2E Playwright** `frontend/tests/e2e/reports-entry-documents.spec.ts` (neuf, **cible : le Journal**) : créer et valider
  une facture par l'API, ouvrir *Rapports → Journaux* sur la période, cliquer l'icône de l'écriture de vente → l'aperçu
  (`iframe` du PDF) s'affiche ; fermer par *Échap*, focus rendu ; puis une écriture à deux pièces (règlement groupé, si le
  seed le permet par l'API, sinon avoir + facture d'une même période en deux écritures) → la liste. Le grand livre : un
  test court (icône présente, aperçu ouvert).

## Recette visuelle

Captures (thème clair, `kesh-gate-logs/s55-3/`) : journal avec icônes (une et plusieurs pièces) ; fenêtre sur un PDF de
facture ; fenêtre en liste ; erreur `410` ; grand livre avec icônes.

## Tâches

- [ ] **T0** — Q1 tranchée par le mainteneur (sinon arrêt) ; base `epic/55-navigation` après la 55-2 ; relever
      l'annulation d'un règlement client (`invoice_settlements`), `general_ledger.rs` sur `epic/47-comptes-miroirs`, les
      clés i18n réutilisables.
- [ ] **T1** — `pieces.rs` et ses tests (rouges d'abord) ; champ `documents` du journal et du grand livre.
- [ ] **T2** — `EntryDocuments.svelte` (icône, dialogue, liste, `DocumentPreview`) ; lecteurs en `Blob` des trois routes
      (réutiliser `fetchSupplierInvoiceSourceDocument` ; factoriser la lecture de la facture et de l'avoir, aujourd'hui
      écrite dans les pages des fiches, sans changer leur téléchargement).
- [ ] **T3** — Vues du journal et du grand livre ; types `reports.types.ts`.
- [ ] **T4** — i18n, manuel, E2E, captures.
- [ ] **T5** — Gates.

## Gates

Backend : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run -E
'package(kesh-report) | binary(reports_e2e) | binary(mcp_contrat)'` (rayon d'impact), suite complète en fin d'epic.
Frontend : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; E2E de la story et des Rapports.

## Note de version

*Rapports → Journaux* et *Grand livre* : une icône sur une écriture ouvre ses pièces — PDF de la facture, de l'avoir, document
de la facture fournisseur ; pour un règlement, la facture réglée. API : les écritures du journal et les lignes du grand livre
portent un champ `documents` (absent quand l'écriture n'a pas de pièce) ; exports PDF et CSV inchangés. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-3-1** — Un champ dans la réponse du rapport plutôt qu'une route groupée (`GET /journal-entries/documents?ids=…`) :
  une requête de moins par affichage, aucune liste d'identifiants à borner, les pièces arrivent avec les écritures
  qu'elles décrivent. *Écarté* : la route groupée — un second aller-retour, une liste `IN` à plafonner.
- **C-55-3-2** — Le règlement montre la facture (D10), comme `find_source_document_of_entry` le fait déjà pour le
  fournisseur.
- **C-55-3-3** — Pas de pièce jointe libre à une écriture manuelle : il n'en existe pas dans Kesh ; hors périmètre (une
  issue `enhancement` s'ouvre si l'utilisateur la demande).
- **C-55-3-4** — Pièces des imports de prestataires (Epics 38 à 40, PayPal, Wise, Stripe) : non relevées à `main` ; si la
  base de départ (PR 46, qui les porte) rattache un fichier à une écriture, le T0 le dit et la fiche est amendée, sinon hors
  périmètre.

### Modules (règle de splitting préventif)

Cinq : `kesh-report`, `frontend/src/lib/features/reports`, `kesh-i18n`, `docs/manual`, `frontend/tests/e2e` (les tests
`kesh-api/tests` n'ajoutent pas de module de production). À la borne, non au-delà : si le T0 ajoute un module (p. ex.
`kesh-db` pour réutiliser une requête), le Grand livre sort (option) ou la story se découpe en serveur et écran, dit au
mainteneur.

### References

- Fiche 30-2c (document d'une facture fournisseur, lecture par tout rôle) ; fiche 25-6-b (PDF figé) ; `CLAUDE.md`
  § *Pattern batch* (sans objet ici : aucune route neuve).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55, story ajoutée le même jour (`drafted`).
