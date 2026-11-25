# Story 55-3a : les justificatifs d'une écriture dans la réponse du journal et du grand livre (serveur)

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 (Q1), D10, D12 (afficher n'est pas figer :
sans aperçu dans l'Epic 55), D14 (fiche de la facture, Q8). Issue : #À-VENIR (`enhancement`, **P4**, Q5 ; point c,
convenu au téléphone le 2026-11-25 entre l'utilisateur et le mainteneur).
Release : la prochaine planifiée après la 0.20.0. **Aucune migration. ⚠️ Backend** (`kesh-report`, `kesh-api`) : un
booléen additif dans la réponse JSON du journal et du grand livre, et **une route de lecture minimale** qui liste les
justificatifs d'une écriture, avec l'état du PDF d'une facture — l'epic était annoncé « frontend seulement » ; **accepté
par le mainteneur le 2026-11-25 (Q1)**. *Réponses de l'utilisateur du 2026-11-25 versées* (D10 : le règlement montre la
facture réglée, client et fournisseur ; D12 : depuis le Journal et les Rapports, **afficher une facture ne la fige
pas**). ⚠️ **Décision du mainteneur du 2026-11-26** : l'aperçu sans figement (`GET /invoices/{id}/pdf/preview`) et la
marque d'un PDF non émis (`kesh-qrbill`) **sortent de l'Epic 55** pour l'epic « PDF marqués » (numéro à confirmer) ; leur
texte est gardé en section *Reporté à l'epic « PDF marqués »*. Dans l'Epic 55, une facture figée s'affiche par son PDF
figé, une facture pas encore émise est signalée « PDF pas encore émis », sans aperçu (*Afficher sans figer, dans
l'Epic 55*).

*Découpée de la 55-3 à la validation P1 (V1-5 : plus de cinq modules pour une seule story) : 55-3a le serveur, 55-3b
l'écran. Le Journal reste le critère principal (demande de l'utilisateur) ; le Grand livre suit, à coût marginal.*

*⚠️ **Récrite le 2026-11-25 sur décision de l'utilisateur** (interface de la 55-3) : **une seule icône PDF par ligne**,
sans compteur ni menu ; au clic, la visionneuse existante avec la liste des documents de l'écriture en haut. Le rapport
ne porte donc qu'un **booléen par écriture** (« a au moins un justificatif »), calculé en une requête ; la **liste** se
charge au clic, par une route de lecture minimale (aucune route existante ne la donne : le détail d'une écriture,
`GET /api/v1/journal-entries/{id}`, ne porte que le document fournisseur, `sourceDocument`). La validation reprend à P1
sur cette version.*

## Story

En tant que comptable qui relit son journal,
je veux que chaque écriture du journal et du grand livre dise quels documents la justifient — PDF de la facture, de
l'avoir, document de la facture fournisseur —,
afin que l'écran puisse me les ouvrir d'un clic (55-3b).

## Vocabulaire

« **Justificatif** » partout (textes, manuel, noms de code `documents`) : « pièce » désigne déjà le **numéro d'écriture**
au grand livre (colonne `reports-ledger-column-piece`, manuel `user-manual.tex:4193`).

## Où un justificatif a un sens

| Rapport | Lignes | Champ `hasDocuments` |
|---|---|---|
| **Journal** | une écriture (`JournalEntryRow`) | **oui — le critère principal** |
| **Grand livre** | une ligne d'écriture d'un compte (`LedgerLine`) | **oui**, même fonction, coût marginal (un appel, un champ) ; *option écrite* : si le T0 montre un coût au-delà, le Grand livre sort de la 55-3a et de la 55-3b et s'ouvre en issue |
| Bilan, Compte de résultat, Balance, TVA | un compte, un taux | **non** : une ligne de compte agrège des écritures ; le chemin est le lien de compte (55-2) puis le grand livre |

## Ce qui se lit au code (`origin/main` `1e23edb55`)

**Aucune table de pièces jointes.** Trois documents se rattachent à une écriture, par **cinq** colonnes (cinq branches
de requête, la branche fournisseur dédoublée pour l'achat et le règlement) :

| Document | Rattachement | Règle du document servi | Route de lecture (droits) |
|---|---|---|---|
| PDF d'une facture client (écriture de vente) | `invoices.journal_entry_id` (`20260417000001_invoice_validation.sql`) | la route sert le PDF **figé** s'il existe, sinon fige une facture **validée**, sinon `400 INVOICE_NOT_VALIDATED` (`crates/kesh-api/src/routes/issued_invoice_pdf.rs:104-109`). Une facture annulée par un avoir **garde** `journal_entry_id` (`crates/kesh-db/src/repositories/credit_notes.rs:843`) ; seule la dévalidation le vide (`crates/kesh-db/src/repositories/invoices.rs:1782`). Manuel § 11.6.1 (`user-manual.tex:1635`) : « une facture annulée par un avoir garde le PDF qu'elle a émis ; … jamais figée … n'a pas de PDF » | `GET /api/v1/invoices/{id}/pdf` (`lib.rs:1262` ; `:1304` sur la PR 46), tout rôle — **fige** au premier appel ; l'écran des Rapports ne l'appelle que pour une facture **figée** (D12,
*Afficher sans figer, dans l'Epic 55*) |
| idem, écriture de **règlement** | `invoice_settlements.journal_entry_id` → `invoice_id` (`20260827000001_invoice_settlements.sql` ; plusieurs factures par écriture depuis `20261118000001_grouped_settlements.sql`, clé `(journal_entry_id, invoice_id)`). Un règlement annulé est **supprimé** (`invoice_settlements_write.rs:1140`, `:972` groupé) | même règle que la ligne précédente | idem |
| PDF d'un **avoir** | `credit_notes.journal_entry_id` (`20260627000001_credit_notes.sql:36` ; `CHECK` : émis ⇒ écriture, `:54`) ; statuts `draft`, `issued`, `cancelled` (`:50`) | rendu à la demande, sans contrôle de statut (`routes/credit_notes.rs:279-330`) | `GET /api/v1/credit-notes/{id}/pdf` (`lib.rs:1275` ; `:1317`), tout rôle |
| **Document** d'une facture fournisseur | `supplier_invoices.purchase_journal_entry_id` ou `settlement_journal_entry_id` (`20260628000001`), ← `imported_supplier_invoices.supplier_invoice_id`, `status = 'completed'` (`20260629000001`). Règlement fournisseur **groupé** : plusieurs factures portent le même `settlement_journal_entry_id` (`20261118000001`, `CHECK` « ligne ⇒ écriture » ; `grouped_settlements.rs:9`) | fichier archivé | `GET /api/v1/supplier-invoices/{id}/source-document` (`lib.rs:1291` ; `:1333`), tout rôle (30-2c AC22) |

- **Précédent** : `kesh-db/src/repositories/imported_supplier_invoices.rs:409-433` `find_source_document_of_entry` — achat
  **ou** règlement, `completed`, pour **une** écriture et **un** document (`ORDER BY si.id LIMIT 1`, `:425`) ; le détail
  d'une écriture l'affiche (`routes/journal_entries.rs:250,512` ; écran `journal-entries/[id]/+page.svelte:589-620`). Un
  règlement détaché ne montre plus le document ; l'achat d'une facture annulée le montre (test
  `cancelled_invoices_take_a_document_and_the_detached_settlement_loses_it`).
- **Aucune voie d'aperçu sans figement** (relevé du 2026-11-25) : `get_or_freeze` (`routes/issued_invoice_pdf.rs:95-112`)
  sert le PDF figé s'il existe (`serve_frozen`, `:350`), sinon fige une facture validée (`render_and_pose`, `:141`, qui
  écrit fichier, colonnes et audit `invoice.pdf_frozen`) ; `Usage` n'a que `Download` et `Send` (`:74-82`). Les autres
  appels de `invoice_pdf_service::render_document` (`invoice_email.rs:566`, `:1317`) rendent un **rappel**
  (`PdfDocument::Reminder`), non la facture. Aucune route d'aperçu de brouillon (`grep -n "preview" lib.rs` : e-mails,
  réévaluation, imports bancaires seulement).
- **Inventaire clos** (validation P1, V1-12) : les autres colonnes vers `journal_entries` (`bank_transactions.matched_entry_id`,
  `journal_entries.reverses_entry_id`) ne portent aucun fichier ; les imports PayPal (`20261120000001`, à `main`) et Stripe
  (`20261120000003`, PR 46) ne rattachent aucun fichier à une écriture.
- **DTO** : `JournalEntryRow` (`kesh-report/src/journal_report.rs:41`), `LedgerLine` (`general_ledger.rs:229`, `entry_id`
  `:231`) : aucun champ de justificatif. Les routes les rendent en JSON telles quelles (`get_journal_report`,
  `get_general_ledger` de `routes/reports.rs`) ; les exports passent par les mêmes `generate` (fonctions `journaux` et
  `grand_livre` de `services/rapports.rs`) et l'outil MCP du grand livre aussi (`mcp/outils/grand_livre.rs:159`), qui
  recopie ses champs dans sa propre structure (`:175`).
- **Littéraux** : `JournalEntryRow` et `LedgerLine` n'ont ni `Default` ni constructeur ; à `1e23edb55`, `git grep -nE
  "JournalEntryRow \{|LedgerLine \{" crates/` rend deux sites de production (`journal_report.rs:232`,
  `general_ledger.rs:901`) et des littéraux de test ou de banc (`general_ledger.rs:1022`, `csv.rs:1484,1660,1878`,
  `pdf.rs:2297,2856,2917`, `benches/export.rs:155`) ; la PR 46 et l'Epic 47 déplacent ceux de `csv.rs` et `pdf.rs`
  (relevé au T0 sur la base de départ).
- **Avoirs** : aucun `UPDATE credit_notes` dans `crates/` ; le seul `INSERT` de production écrit `status = 'issued'`
  (`credit_notes.rs:780`, « single-step ») : `draft` et `cancelled` ne sont pas écrits par l'application aujourd'hui.
- **Clés étrangères** vers l'écriture seulement (`fk_invoices_journal_entry`, `fk_credit_notes_journal_entry`,
  `fk_supplier_invoices_purchase_je`, …) : le schéma admet qu'une ligne d'une société pointe une écriture d'une autre ;
  d'où le filtre de société sur chaque table.
- **Documentation de l'API** : `docs/api-external.md` décrit ces réponses champ par champ (grand livre `:310`, journal
  `:1164`, avec les précédents additifs `letteringCode` et champs en devise).
- `fetch_counterparts` (`general_ledger.rs:639-690`) fait **une requête par compte**, appelée dans `for acc in accounts`
  (`:825`, `:861`), avec une liste `IN` de taille variable : **ce n'est pas le patron** de la 55-3a.
- Index `idx_journal_entries_company_date (company_id, entry_date DESC)` (`20260412000001_journal_entries.sql:33`).
- L'Epic 47 ne touche ni `journal_report.rs` ni `general_ledger.rs` (`git diff --stat 1e23edb55 epic/47-comptes-miroirs`).

## Le contrat (D9)

### Dans le rapport : un booléen

`JournalEntryRow` et `LedgerLine` gagnent :

```rust
/// L'écriture a au moins un justificatif — omis quand il est faux (JSON identique à
/// aujourd'hui pour ces écritures). Écran seulement : PDF et CSV inchangés.
#[serde(skip_serializing_if = "std::ops::Not::not")]
pub has_documents: bool,
```

TypeScript (55-3b) : `hasDocuments?: boolean` sur `JournalEntryRow` et `LedgerLine`.

### La liste : une route de lecture minimale

`GET /api/v1/journal-entries/{id}/documents` → `200 [EntryDocument]` (tableau, vide si aucun), **tout rôle authentifié**
(montée parmi les lectures, à côté de `GET /api/v1/journal-entries/{id}`, `lib.rs:1199`), bornée à la société de
l'utilisateur ; écriture inconnue ou d'une autre société → `404`, comme le détail.

```rust
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum EntryDocumentKind { Invoice, CreditNote, SupplierDocument } // ordre = ordre d'affichage

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntryDocument {
    /// `invoice` | `creditNote` | `supplierDocument`.
    pub kind: EntryDocumentKind,
    /// Identifiant pour la route de lecture : facture, avoir, facture fournisseur.
    pub id: i64,
    /// Numéro : facture, avoir ; numéro interne de la facture fournisseur, à défaut le sien.
    pub number: Option<String>,
    /// Nom du fichier importé (document fournisseur seulement).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// État du PDF d'une facture (facture seulement) : `frozen` (PDF émis, figé) ou `notFrozen`
    /// (pas de PDF figé : l'écran affiche « PDF pas encore émis », sans aperçu, 55-3b).
    /// Énumération **ouverte** : l'epic « PDF marqués » (aperçu) et le PDF produit hors de
    /// Kesh (`external`, *Limite connue et évolution*) pourront y ajouter une valeur sans
    /// rien changer d'autre.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_status: Option<PdfStatus>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PdfStatus { Frozen, NotFrozen }
```

TypeScript (55-3b) : `interface EntryDocument { kind: 'invoice' | 'creditNote' | 'supplierDocument'; id: number; number:
string | null; fileName?: string; pdfStatus?: string }` — `pdfStatus` typé `string` et non en union fermée : l'écran ne
lit le PDF d'une facture que pour `'frozen'` ; toute autre valeur (`'notFrozen'`, champ absent, valeur future) ne lit
rien et affiche « PDF pas encore émis » (55-3b, AC-5) — le côté sûr : l'écran ne fige jamais.

`pdf_status` se lit dans la même requête que la liste (`i.pdf_storage_path IS NOT NULL` ⇒ `frozen`, la règle
d'`is_frozen`, `issued_invoice_pdf.rs:325`) ; il vaut **à la lecture de la liste** : une facture figée entre la lecture et
l'affichage (téléchargement depuis la fiche par un autre utilisateur) reste signalée « PDF pas encore émis » jusqu'à la
prochaine ouverture — limite assumée, écrite au manuel par la 55-3b.

### Afficher sans figer, dans l'Epic 55 (D12)

Décision du mainteneur du 2026-11-26 : l'aperçu marqué — la route d'aperçu et la marque dans `kesh-qrbill` — **sort de
l'Epic 55** pour l'epic « PDF marqués » (numéro à confirmer ; section *Reporté à l'epic « PDF marqués »*). La 55-3a
n'ajoute **aucune route de PDF** et ne touche ni `issued_invoice_pdf.rs`, ni `invoice_pdf.rs`, ni
`invoice_pdf_service.rs`, ni `kesh-qrbill`, ni `kesh-i18n`. Ce que l'écran fait d'une facture (55-3b) :

- **facture figée** (`pdfStatus: "frozen"`, validée ou annulée) : l'écran lit `GET /api/v1/invoices/{id}/pdf`, qui sert
  le fichier figé (`serve_frozen`) **sans rien écrire** — ni fichier, ni colonne, ni audit : `get_or_freeze`
  (`issued_invoice_pdf.rs:95-112`) ne rend et ne pose que pour une facture non figée ;
- **facture pas encore émise** (`pdfStatus: "notFrozen"`) : **aucune lecture de PDF** ; l'écran affiche « PDF pas encore
  émis », **sans aperçu**, tant que l'epic « PDF marqués » n'est pas livré — afficher n'est pas figer, et `/pdf`
  figerait ;
- ⚠️ **une validée non figée dont un PDF est pourtant sorti** (`invoices.emailed_at` non nul : envoi d'avant la v0.12.1 ;
  ou une entrée `invoice.pdf_frozen` au journal d'audit : facture téléchargée, dévalidée puis revalidée — la dévalidation
  est refusée à une facture envoyée, `errors.rs:443`) : **pour l'Epic 55, elle n'est pas figée** — `pdfStatus:
  "notFrozen"`, comme les autres, et l'écran la signale « PDF pas encore émis ». La 55-3a ne lit ni `emailed_at` ni le
  journal d'audit. **Aucune nuance de texte** n'est nécessaire dans l'Epic 55 : le texte de la 55-3b (AC-5) dit que la
  facture **n'a pas de PDF figé** et ce qui le figera, non qu'elle n'a jamais été envoyée ni téléchargée ; il reste vrai
  pour ces factures. Le bandeau propre à ce cas est décidé pour l'epic « PDF marqués » (section reportée) ;
- **course** : `pdfStatus` vaut à la lecture de la liste ; une facture figée entre la lecture et le clic reste signalée
  « PDF pas encore émis » jusqu'à la prochaine ouverture (rien n'est lu, rien n'est figé). Le cas inverse — une facture lue
  `frozen`, redevenue non figée avant le clic — ne naît que d'une restauration d'une sauvegarde d'avant le gel (la
  dévalidation vide `journal_entry_id`, `crates/kesh-db/src/repositories/invoices.rs:1782`, et la liste est relue à chaque
  ouverture) : `/pdf` la figerait alors. Limite assumée, écrite à la note de l'API (T3) ;
- **fiche de la facture inchangée** (D14, Q8 refusée) : elle garde son bouton, qui fige au premier téléchargement — le
  **geste d'émission** ;
- **aucune dérogation** à l'arbitrage 8 de la 25-6-b dans l'Epic 55 : aucun PDF non émis ne sort de Kesh.

### Limite connue et évolution : un PDF produit hors de Kesh

Un jour, une facture client créée par l'API pourra porter un **PDF produit hors de Kesh** ; c'est lui qui devra
s'afficher, à la place du PDF que Kesh génère. **Non livré ici** (issue `enhancement`, évolution : texte dans
`kesh-gate-logs/s55-decisions/issues-a-envoyer.md`). Le modèle ne l'empêche pas : un élément de la liste
désigne la **facture** (`kind`, `id`), non un fichier ; c'est la route de lecture qui choisit le fichier servi ; et
`pdfStatus` est une énumération ouverte, à laquelle une valeur (`external`) s'ajoutera, que l'écran d'aujourd'hui traite
déjà (aucun bandeau). Issue `enhancement` à ouvrir (section *Issues à ouvrir*).

**Règles de cette évolution** (avis d'usager du 2026-11-25, **décidées par le mainteneur le même jour**) :

- le PDF fourni **est** la pièce émise : copié dans Kesh, rangé et **sauvegardé** comme les autres documents, **figé**
  avec la facture, **jamais régénéré** ; la raison : il doit rester cohérent avec les lignes de la facture en base, et un
  fichier resté hors de Kesh peut changer ou disparaître sans que la facture le sache ;
- pour une facture arrivée avec son PDF, Kesh ne produit **ni second PDF ni QR-facture à lui** — ni aperçu marqué (epic
  « PDF marqués ») : l'écran le servira comme un PDF figé ;
- « lien seulement » ne viendrait **que sur demande** : un lien ne passe pas dans le `.keshbackup`.

### Règles de rattachement

- **Facture** (vente et règlement) : `i.status = 'validated' OR (i.status = 'cancelled' AND i.pdf_storage_path IS NOT
  NULL)` — les factures pour lesquelles la route **peut** servir un PDF (son rendu peut encore être refusé : `INVOICE_NOT_PDF_READY`, `410` si le fichier figé manque, `409` ; l'écran affiche alors l'erreur, 55-3b AC-4) ; une facture annulée jamais figée n'a pas de justificatif.
  Une facture **validée non figée** reste un justificatif : l'écran la signale « PDF pas encore émis », sans aperçu ni
  figement (D12).
- **Avoir** : `cn.status <> 'draft'` — la route rend le PDF d'un avoir sans contrôle de statut ; un brouillon n'est pas
  une pièce comptable ; l'application n'écrit aujourd'hui que `issued`
  (voir *Ce qui se lit au code*), la règle vaut pour un avoir annulé futur.
- **Document fournisseur** : `isi.status = 'completed'`, achat ou règlement, **tous** les documents d'un règlement groupé.
- **Filtre de société sur chaque table** de chaque branche (`je.company_id`, `i.company_id`, `s.company_id` de
  `invoice_settlements`, `cn.company_id`, `si.company_id`, `isi.company_id`), défense en profondeur (KF-002).
- **Dédoublonnage** par `(kind, id)` dans chaque écriture, à la construction de la `HashMap` (une même facture ne peut
  pas sortir deux fois d'une écriture aujourd'hui, mais rien au schéma ne l'interdit) ; **ordre stable** : genre
  (`invoice`, `creditNote`, `supplierDocument`), puis identifiant.
- **Écart assumé avec le détail d'une écriture** : un règlement fournisseur groupé montre **tous** ses documents au
  journal, **un seul** au détail de l'écriture (`find_source_document_of_entry`, `LIMIT 1`). Issue `enhancement` à ouvrir
  pour aligner le détail (section *Issues à ouvrir*).

### Les requêtes, une seule définition des rattachements (DRY)

- `kesh-report/src/documents.rs` (neuf) porte **un** constructeur des cinq branches (facture de vente, facture réglée,
  avoir, document fournisseur par l'achat, par le règlement — deux branches plutôt qu'un `OR`, pour l'index de chaque
  colonne), avec les règles ci-dessus, paramétré par son filtre d'écritures ; deux fonctions l'emploient :
  - `entries_with_documents(pool, company_id, from, to) -> HashSet<i64>` : `SELECT DISTINCT entry_id FROM (… UNION ALL
    …)`, chaque branche jointe à `journal_entries` et bornée par `je.company_id = ?` et `je.entry_date BETWEEN ? AND ?`
    (index `idx_journal_entries_company_date`) — **une** requête par rapport, aucune liste `IN`, aucune requête par
    écriture ni par compte ;
  - `documents_of_entry(pool, company_id, entry_id) -> Vec<EntryDocument>` : la même union bornée par `je.id = ?` et
    `je.company_id = ?`, dédoublonnée par `(kind, id)`, triée (genre, identifiant) ; la route l'appelle.
- **Site d'appel** du booléen : **un** appel dans `journal_report::generate`, après la requête des lignes ; **un** appel
  dans `general_ledger::generate`, **avant** la boucle `for acc in accounts` (`:825`), l'ensemble étant lu dans la boucle.
  Les exports et l'outil MCP, qui passent par ces `generate`, paient cette requête sans lire le champ : assumé.
- **Borne du grand livre** : la période seule (C-55-3a-4).

## Critères d'acceptation

1. **AC-1 — Journal.** `GET /api/v1/reports/journals` : chaque écriture qui a au moins un justificatif, selon les règles
   ci-dessus, porte `hasDocuments: true` ; les autres n'ont pas le champ.
2. **AC-2 — Grand livre.** `GET /api/v1/reports/general-ledger` : chaque `LedgerLine` porte `hasDocuments` de son écriture ;
   une écriture présente dans plusieurs sections le porte dans chacune. *Option* : § *Où un justificatif a un sens*.
3. **AC-3 — La route.** `GET /api/v1/journal-entries/{id}/documents` rend la liste, ordonnée et dédoublonnée : facture
   validée (un justificatif) ; facture annulée figée (un) ; facture annulée jamais figée (aucun) ; règlement simple (la
   facture) ; **règlement client groupé** de deux factures (deux, ordre par identifiant) ; avoir émis (un) ; facture
   fournisseur à document, écriture d'achat et de règlement (le document) ; **règlement fournisseur groupé** de deux
   factures à document (deux) ; facture fournisseur sans document (aucun) ; facture fournisseur annulée, écriture d'achat
   (le document). Pour chaque cas, `hasDocuments` du journal est vrai **si et seulement si** la liste est non vide (même
   constructeur : test qui les compare sur tous les cas). Rôle *Consultation* : `200` ; clé d'API en lecture : `200` ; écriture d'une
   autre société : `404` ; sans session : `401`. Les règlements de type « solde du reste » et « compte interne » sont
   rattachés à leur facture comme les autres (un cas chacun).
4. **AC-4 — Négatifs qui rougissent.** Chaque filtre est éprouvé par des données **posées en SQL direct** que le schéma
   admet : avoir `draft` portant un `journal_entry_id` ; document importé `discarded` portant un `supplier_invoice_id` dont
   la facture fournisseur porte l'écriture d'achat ; facture `draft` portant un `journal_entry_id` ; **par branche**, une
   ligne de la société B (facture, règlement, avoir, facture fournisseur, document importé) qui pointe une écriture de la
   société A, plus un document importé de B rattaché à une facture fournisseur de A (filtre `isi.company_id`). Les tests
   appellent **`entries_with_documents` et `documents_of_entry` directement** (aucune clé de B ni hors période), en plus du
   journal et de la route. Le T0 vérifie qu'aucune contrainte ne refuse ces lignes ; une ligne refusée rend son filtre
   « défense en profondeur, non éprouvable », écrit tel quel au Change Log. Le règlement annulé (supprimé) : test de
   **non-régression**. Chaque filtre a sa **mutation** (filtre retiré → test rouge), journal `kesh-gate-logs/s55-3a/`.
5. **AC-5 — Une requête.** Le booléen du journal et celui du grand livre viennent d'**un** appel d'`entries_with_documents`
   chacun, hors de toute boucle (`grep -nE "entries_with_documents\(" crates/kesh-report/src/*.rs | grep -vE
   ':[0-9]+:\s*//'` rend une définition et deux appels — les tests d'appel direct de l'AC-4 vivent donc dans
   `crates/kesh-report/tests/documents.rs`, non dans un `mod tests` de `src/`) ; grand livre à plusieurs comptes : le booléen de chaque section est
   juste ; une écriture hors période n'est pas lue.
6. **AC-6 — Contrat additif.** Une écriture sans justificatif sérialise **exactement** le JSON d'aujourd'hui (clés
   comparées) ; les exports PDF et CSV du journal et du grand livre rendent le même résultat avec `hasDocuments` vrai ou
   faux (CSV : chaînes égales ; PDF : texte lu par `kesh_report::texte_pdf::lignes` de la PR 46, après avoir vérifié qu'il
   contient un numéro d'écriture connu) ; garde de non-régression, déclarée telle ; `kesh-report/tests/rapports_mono_devise.rs`
   (références figées du JSON, du CSV et du PDF) reste vert sans modification et tient le cas « sans justificatif ». L'outil MCP du grand livre recopie ses
   champs : sortie inchangée par construction, `mcp_contrat` vert comme filet.
7. **AC-7 — Documentation de l'API.** `docs/api-external.md` : le champ `hasDocuments` aux paragraphes du grand livre
   (`:310` à `1e23edb55`) et du journal (`:1164`), et la route `GET /journal-entries/{id}/documents` (forme, genres, droits,
   `404`), avec sa ligne dans la table des routes ouvertes aux clés (`:232`) ; lignes relevées sur la base PR 46.
8. **AC-8 — Note pour l'écran.** Les deux contrats TypeScript ci-dessus sont recopiés dans la fiche 55-3b à l'identique.
9. **AC-9 — Aucune route de PDF neuve** (l'aperçu sans figement est reporté à l'epic « PDF marqués », section
   reportée). `grep -n "pdf/preview" crates/kesh-api/src/lib.rs` ne rend rien ; `issued_invoice_pdf.rs`,
   `invoice_pdf.rs`, `invoice_pdf_service.rs` et `crates/kesh-qrbill/` sont absents du `git diff --stat` de la story ;
   constat au Change Log.
10. **AC-10 — État du PDF dans la liste.** `GET /journal-entries/{id}/documents` : une facture validée non figée porte
    `pdfStatus: "notFrozen"`, une facture figée (validée ou annulée) `"frozen"` ; une validée non figée à `emailed_at` non
    nul `"notFrozen"` aussi (*Afficher sans figer, dans l'Epic 55*) ; un avoir et un document fournisseur n'ont pas le
    champ. Après un `GET /pdf`, la même écriture rend `"frozen"`. Mutation : `pdf_status` toujours `frozen` → test rouge.
11. **AC-11, AC-12 — reportés** à l'epic « PDF marqués » (la marque d'un PDF non émis ; le PDF émis inchangé ; section
    reportée). L'Epic 55 ne touche pas `kesh-qrbill` : le rendu du PDF émis est inchangé par construction (AC-9).

## Tâches

- [ ] **T0** — Q1 acceptée par le mainteneur le 2026-11-25 ; aperçu et marque sortis de l'epic le 2026-11-26 (section
      reportée) ; réponses de l'utilisateur relues (D10, D12) ; base `epic/55-navigation` (PR 46 reprise) ; relever le sens
      d'un avoir `cancelled` et de son écriture ; vérifier les lignes SQL directes de l'AC-4 contre les contraintes.
- [ ] **T1** — Tests rouges d'abord (AC-3 à AC-6) ; journal.
- [ ] **T2** — `documents.rs` (constructeur commun, deux fonctions) ; `has_documents` du journal et du grand livre, appels
      hors boucle ; les deux littéraux de **production** (`journal_report.rs:232`, `general_ledger.rs:901`) le lisent dans
      l'ensemble (`contains(&entry_id)`) ; les littéraux de **test et de banc** reçoivent `has_documents: false` et rien
      d'autre (règle
      de fichier commun pour `csv.rs`, `pdf.rs`, `benches/export.rs`, réécrits par les Epics 46 et 47).
- [ ] **T2bis** — Un service `services::ecritures::justificatifs` (`crates/kesh-api/src/services/ecritures.rs`, placé après
      `detail`, avec sa constante `EXIGENCE_*` comme `EXIGENCE_DETAIL` `:46`) : `find_by_id` borné à la société (le `404`),
      puis `documents_of_entry(lecture.pool(), …)` — la couche de services est le seul lieu des lectures des livres
      (`services/mod.rs`, R1 à R4) ; la route `GET /api/v1/journal-entries/{id}/documents` l'appelle, sur le modèle du détail
      (`routes/journal_entries.rs:494-499`, `ecritures::detail(&ContexteAcces::depuis_api(..), &state.lecture(), id)`) ;
      tests d'intégration `crates/kesh-api/tests/journal_entry_documents_e2e.rs` (neuf).
- [ ] **T2ter** — `pdf_status` dans `documents_of_entry` (AC-10), lu dans la même requête ; aucune route de PDF neuve
      (AC-9). *(La route d'aperçu et la marque — anciens T2ter et T2quater : section reportée.)*
- [ ] **T3** — `docs/api-external.md` (AC-7 ; `pdfStatus` : ses deux valeurs, énumération ouverte, valeur à la lecture
      de la liste, et la course de *Afficher sans figer, dans l'Epic 55*) ; lignes relevées sur la base PR 46. *(La
      documentation de la route d'aperçu et de son code d'erreur : section reportée.)*
- [ ] **T4** — Gates.

## Gates

Règle de l'epic (fichier d'epic, § *Gates*) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D
warnings` ; `cargo nextest run -E 'package(kesh-report) | package(kesh-api)'` (rayon d'impact : tout `kesh-api`, la liste des binaires
qui lisent le journal ou le grand livre étant longue — `reports_e2e`, `reports_export_e2e`, `fx_recette_api_e2e`,
`open_items_e2e`, `dates_bornes_e2e`, `services_motif`, `mcp_contrat`, `mcp_outils_e2e`, `mcp_enveloppe_e2e`,
`mcp_transport_e2e`, `journal_entry_reversal_e2e`, les tests du lib ; relevé par `git grep -lE
"reports/(journals|general-ledger)|journal_report|general_ledger" crates/kesh-api`). **À la clôture de la story** (fichier d'epic, § *Gates*, *Dérogation* : la
story qui touche le serveur suit la règle) : **gate complet** (`cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo nextest` du workspace) **et E2E complet** au dernier commit de code ;
`scripts/gate-ci.sh` en fin d'epic. Aucun fichier de `kesh-db` touché (sinon gate complet en cours de boucle aussi,
`CLAUDE.md` § *Exception `kesh-db`*).

## Recette visuelle

Aucune : la story ne change ni un écran ni un PDF (la recette du PDF marqué suit l'aperçu : section reportée).

## Note de version

API : les écritures du journal et les lignes du grand livre portent `hasDocuments` (vrai quand l'écriture a au moins un
justificatif : facture, avoir, document de facture fournisseur ; absent sinon) ; `GET /journal-entries/{id}/documents` rend
la liste des justificatifs d'une écriture, avec l'état du PDF d'une facture (`pdfStatus` : `frozen` ou `notFrozen`) ;
`GET /invoices/{id}/pdf` est inchangé ; exports PDF et CSV inchangés. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-3a-1** — Un booléen dans le rapport, la liste au clic (décision de l'utilisateur du 2026-11-25) : le rapport reste
  léger, la liste ne se lit que pour l'écriture ouverte. *Écarté* : la liste dans le rapport (version précédente de la
  fiche), et une route groupée `?ids=…` (inutile : on n'ouvre qu'une écriture à la fois).
- **C-55-3a-2** — Le règlement montre la facture réglée (D10), comme `find_source_document_of_entry` pour le fournisseur.
- **C-55-3a-3** — Pas de justificatif libre joint à une écriture manuelle : il n'en existe pas dans Kesh ; hors périmètre.
- **C-55-3a-5** — Reporté avec l'aperçu (section *Reporté à l'epic « PDF marqués »*).
- **C-55-3a-4** — Borne du grand livre par période seule (voir *La requête*), quelle que soit la page (`limit`, `offset`)
  : la requête lit les justificatifs de toute la période, une page n'en montre qu'une partie.

### Modules (règle de splitting préventif)

Deux modules de production : `kesh-report` (`documents.rs`, `journal_report.rs`, `general_ledger.rs`, `lib.rs`, et les
littéraux de test de `csv.rs`, `pdf.rs`, `benches/export.rs`) et `kesh-api` (le service `services/ecritures.rs`, la route
`routes/journal_entries.rs`, `lib.rs`) ; plus `docs/api-external.md` et les tests `kesh-report/tests/`, `kesh-api/tests/`.
*Réévalué après la décision du mainteneur du 2026-11-26* (aperçu et marque sortis de l'epic) : `kesh-qrbill`,
`kesh-i18n`, `errors.rs` et les fichiers du PDF de la facture (`issued_invoice_pdf.rs`, `invoice_pdf.rs`,
`invoice_pdf_service.rs`) sortent du périmètre ; au grain des « modules métier de premier niveau », un seul chantier
(les justificatifs des écritures) dans deux crates, loin du seuil de cinq ; taille ramenée de **XL** à **M** — celle de
la fiche avant l'aperçu (validation P2 de la 55-3a, V2-11) : cinq AC de comportement (AC-1 à AC-5), l'AC-10 de
`pdfStatus` et trois AC de contrat et de documentation (AC-6 à AC-8, AC-9 constat), une requête et une route de lecture.

### References

- Fiche 30-2c (document d'une facture fournisseur, lecture par tout rôle) ; fiche 25-6-b (PDF figé) ; Story 43-1a/43-1b
  (règlements groupés).

## Reporté à l'epic « PDF marqués »

Décision du mainteneur du 2026-11-26 : les PDF marqués sortent de l'Epic 55 et forment un petit epic à part, « PDF
marqués » (numéro à confirmer), qui porte le filigrane, l'en-tête et la QR-facture barrée dans `kesh-qrbill`, l'aperçu
« APERÇU — non émis » et le PDF du brouillon. Le texte ci-dessous est celui de la fiche au 2026-11-25, après sa passe
Pv2p6, **laissé tel quel** pour servir à la fiche de cet epic ; ses renvois (D12, D13, AC-9, AC-11, AC-12, T2ter,
T2quater, *Recette visuelle*, 55-3b) désignent la numérotation de ce jour-là. Les LOW renvoyés au T0 qui portent sur ce
contenu le suivent : Vv2p4-1 à Vv2p4-4 (Pv2p4), Pv2p6-1 à Pv2p6-4 (Pv2p6) (Change Log).

**Un PDF déjà sorti sans être figé** (décision du mainteneur du 2026-11-26, pour l'epic « PDF marqués » ; tranche la
question *Aperçu d'un PDF déjà sorti* du fichier d'epic) : une facture validée non figée dont un PDF est sorti —
`invoices.emailed_at` non nul (envoi d'avant la v0.12.1), ou une entrée `invoice.pdf_frozen` au journal d'audit (facture
téléchargée, dévalidée puis revalidée) — **recevra l'aperçu marqué**, avec un bandeau « Un PDF de cette facture est
sorti le … sans être gardé ; ceci est un aperçu. » (date : celle de l'envoi ou du gel). Dans l'Epic 55, une telle facture
n'est pas figée : « PDF pas encore émis » (*Afficher sans figer, dans l'Epic 55*).

**Ancien § *L'aperçu sans figement (D12)* et § *La marque d'un PDF non émis (D13, Q10)*** :

### L'aperçu sans figement (D12)

`GET /api/v1/invoices/{id}/pdf/preview` — **neuve**, montée à côté de `GET /api/v1/invoices/{id}/pdf` (`lib.rs:1262`),
tout rôle authentifié (*Consultation* compris), **fermée aux clés d'API**, bornée à la société (`404` sinon, comme `/pdf`) :

- ⚠️ **Dérogation écrite à l'arbitrage 8 de la 25-6-b** (« une pièce sortie de Kesh est figée, qui que ce soit qui l'ait
  fait sortir », `issued_invoice_pdf.rs:37-39`, `invoice_pdf.rs:11-13`, manuel `user-manual.tex:1641`) : l'aperçu fait
  sortir vers l'écran un PDF de facture **non émis**. Motif : la réponse de l'utilisateur (afficher n'est pas figer),
  bornée à l'**affichage dans Kesh** ; **acceptée par le mainteneur (Q1) à condition que le PDF porte lui-même sa marque**
  (*La marque d'un PDF non émis*, D13) : téléchargé par le lecteur du navigateur, imprimé ou transmis, l'aperçu dit de
  lui-même qu'il n'est pas émis, et sa QR-facture barrée ne se paie pas. Le bandeau (55-3b) le dit en plus dans Kesh.
- **Clés d'API : `403`** — une clé n'affiche rien ; ouvrir l'aperçu aux clés ferait sortir des PDF non figés vers un
  client d'API, au-delà de la réponse de l'utilisateur. Garde au gestionnaire sur `current_user.api_key_id` (le
  discriminant de `require_not_pat`, `middleware/rbac.rs:58-67`), code neuf `API_KEY_PREVIEW_FORBIDDEN`, message dans les
  quatre locales (`kesh-i18n`). Q1 acceptée par le mainteneur avec cette fermeture ; la rouvrir aux clés serait
  un choix du mainteneur.

- **facture figée** (validée ou annulée) : le fichier figé, par `serve_frozen` (même vérification d'empreinte ; `410
  INVOICE_PDF_GONE` si le fichier manque, `500` s'il est altéré) — octet pour octet celui de `/pdf`, **tel quel, sans
  marque** (décision du mainteneur, Q1) ;
- **facture validée non figée** — la seule à laquelle sert le rendu d'aperçu (décision du mainteneur, Q1 : « les factures
  validées dont le PDF n'est jamais sorti ») : `render_document(…, PdfDocument::Invoice)`
  dans la langue du client (`client_language`, la règle du figement), **avec la marque d'aperçu** (D13), **sans aucune
  écriture** : ni fichier dans `KESH_DOCUMENTS_DIR`, ni colonne `pdf_*`, ni `version`, ni trace d'audit
  (`invoice.pdf_frozen` ni aucune autre) ; les erreurs de rendu sont celles de `/pdf` (`400 INVOICE_NOT_PDF_READY`, …) ;
- ⚠️ **une validée non figée dont le PDF est pourtant sorti** : deux des cas du § 11.6.1 laissent une trace au code — un
  envoi d'avant la v0.12.1 (`invoices.emailed_at` non nul, posé à chaque envoi, `invoices.rs:2843`) et une facture
  dévalidée puis revalidée (la dévalidation vide les `pdf_*`, `invoices.rs:1782-1784`, mais une entrée `invoice.pdf_frozen`
  antérieure reste au journal d'audit) ; seule la restauration d'une sauvegarde d'avant le gel n'en laisse aucune. La fiche
  les rend, comme toute validée non figée, en **aperçu marqué** (la marque dit d'elle-même que le document n'est pas
  émis) : c'est une **lecture** de la condition de Q1, soumise au mainteneur (question *Aperçu d'un PDF déjà sorti* du
  fichier d'epic) ; s'il la refuse, ces deux signes ferment l'aperçu à ces factures (un cas chacun à l'AC-9, une mutation),
  avec un code d'erreur et un affichage à fixer avec la 55-3b ;
- **brouillon, facture annulée jamais figée** : `400 INVOICE_NOT_VALIDATED`, comme `/pdf` ;
- `Content-Disposition: inline; filename="apercu-facture-<numéro>.pdf"` pour un rendu non figé, `facture-<numéro>.pdf`
  pour le fichier figé — en-tête pour un appel direct ; l'écran lit le PDF par `getBlob` et nomme lui-même le document
  (55-3b, AC-3) ;
- **course** : un `GET /pdf` concurrent peut figer la facture entre la lecture et le rendu de l'aperçu ; l'aperçu rend
  alors un rendu neuf **marqué**, non le PDF émis. Limite assumée (le PDF porte « APERÇU » et sa QR-facture est barrée),
  sans relecture après le rendu ; écrite à la note de l'API (T3) ;
- une fonction `preview(ctx, invoice_id)` dans `issued_invoice_pdf.rs`, qui réemploie `load`, `is_frozen`,
  `serve_frozen` et `client_language` (DRY) ; `get_or_freeze` et `Usage` **inchangés** ; le gestionnaire dans
  `routes/invoice_pdf.rs`, sur le modèle de `get_invoice_pdf` (`:27-67`).
- **Commentaires rendus faux, corrigés au T2ter** : `issued_invoice_pdf.rs:4-5` (« le **seul** chemin vers le PDF d'une
  facture »), `invoice_pdf_service.rs:274-278` (« ⛔ … la facture ne se rend plus directement ») et `invoice_pdf.rs:1`
  (un seul endpoint) : l'aperçu est le seul autre appelant légitime de `render_document(…, PdfDocument::Invoice)`, son
  rendu **n'est pas** le document émis et n'est jamais posé.
- **Fiche de la facture inchangée** (D14, décision du mainteneur du 2026-11-25, Q8 refusée) : elle garde son bouton
  actuel, qui fige au premier téléchargement (`/pdf`, `Usage::Download`) — c'est le **geste d'émission** ; elle n'a pas
  d'aperçu.

### La marque d'un PDF non émis (D13, Q10)

Décision du mainteneur du 2026-11-25. Le rendu de l'aperçu non figé — et lui seul dans cette story — porte, **sur chaque
page** :

- **un filigrane « APERÇU »** en diagonale, assez gros pour traverser la page, qui peut passer sur le texte mais reste
  **nettement moins foncé** que lui : gris clair (niveau fixé au T0 par la recette visuelle, point de départ `0.85` en RVB
  égal, à comparer au noir `0.0` des textes), **dessiné avant le contenu** — donc sous lui : une lettre de texte noir posée
  après reste noire là où elle croise le filigrane ; la facture reste bien lisible, **même imprimée en noir et blanc** ;
- **en en-tête**, « Aperçu — non émis », à un endroit que le contenu d'aujourd'hui laisse libre (relevé au T0 : le bloc du
  créancier commence à `PAGE_H - 20.0`, `pdf.rs:430`) ;
- **la QR-facture barrée et non payable** : la section de paiement (`dessiner_section_de_paiement`, `pdf.rs:1802`) n'est
  **pas** dessinée ; à sa place, sous le séparateur (`draw_separator`), un **cadre barré** (deux diagonales) portant une
  mention courte (« Aperçu — non payable » : **proposition de la fiche**, Q10 ne demandant qu'un cadre barré ; confirmée
  ou retirée à la recette visuelle). Aucune matrice QR, aucune croix suisse, aucun IBAN ni référence de paiement
  dans le PDF. La charge utile reste **construite et validée** comme pour l'émission (`build_payload`, contrôle du module ≥
  0,4 mm, `pdf.rs:73-86`) : l'aperçu refuse ce que l'émission refuserait (`400 INVOICE_NOT_PDF_READY`, …), il ne dessine
  seulement pas la QR.

**Un seul mécanisme de rendu**, dans `kesh-qrbill` (vérifié au code : le PDF de la facture se dessine dans
`crates/kesh-qrbill/src/pdf.rs` : `render_document` appelle `generate_qr_bill_pdf`,
`crates/kesh-api/src/routes/invoice_pdf_service.rs:335`, qui délègue à `generate_qr_bill_pdf_with_date`, `pdf.rs:47`,
`:52`, `:58`) : une structure publique de marque, **paramétrée par ses textes** (filigrane, en-tête, mention du cadre
barré), passée en option au rendu de la facture par une **variante neuve** (ou un paramètre qui traverse les deux
fonctions) **sans changer les signatures** que lisent les tests figés (`rendu_avant_apres.rs:39`, `golden_test.rs:12`) ;
**sans marque, le rendu est celui d'aujourd'hui** : même **empreinte** (`rendu_avant_apres.rs`,
`neuf_pieces_chiffrees_se_rendent_comme_en_0_19_0`, `:411` — les octets bruts ne sont pas stables, `printpdf` tirant au
hasard une part du `/ID` et de la métadonnée XMP, `empreinte.rs:8-10`), golden `golden_test.rs` inchangés et verts. Le même mécanisme doit pouvoir servir **« BROUILLON »** (PDF d'un brouillon : filigrane « BROUILLON »,
sans numéro, QR-facture barrée), avec d'autres textes et sans rien d'autre de neuf : aucun texte ni booléen « aperçu » en
dur dans `kesh-qrbill`. **Le PDF d'un brouillon n'est pas livré ici** (question au mainteneur du fichier d'epic) :
`render_document` refuse aujourd'hui un brouillon (`invoice.status != "validated"` → `InvoiceNotValidated`,
`invoice_pdf_service.rs:293-295`) et la 55-3a ne change pas ce refus.

- **Textes** : trois clés neuves de `kesh-i18n` (préfixe `invoice-pdf-preview-`, quatre locales), dans la langue du
  client comme le reste du document. **Qui remplit la structure** : `kesh-api`. Deux voies, tranchées au T0 : (a) les clés
  ajoutées à `I18N_KEYS` **et à la même position** dans `DEFAULT_EN` (`types.rs:312`, test d'appariement positionnel
  `pdf.rs:3759`), lues par `build_i18n` (`invoice_pdf_service.rs:547-549`), et `render_document` construit la marque par
  `qr_i18n.get("invoice-pdf-preview-…")` — les replis anglais de `DEFAULT_EN` sont alors l'exception admise à « aucun
  texte en dur » ; (b) les trois textes lus au bundle dans `kesh-api` (`i18n.format`), `I18N_KEYS` inchangé. Dans les deux
  cas, le dessin de `kesh-qrbill` ne lit **que** la structure. Un test du patron de
  `client_number_labels_are_translated_in_all_four_locales` (`crates/kesh-i18n/src/loader.rs:260-276`) garde les trois
  clés traduites dans les quatre locales (le repli silencieux sur le français, `loader.rs:249-253`, ne rougit pas sans
  lui). Les caractères (`Ç`, `é`, `—`) sont ceux de WinAnsi : le T0 vérifie par extraction de texte qu'ils sortent tels
  quels ; ⚠️ `texte_imprime` remplace U+2014 par `-` (règle `texte.rs:32`) : la marque ne passe pas par lui, ou la mention
  s'écrit sans tiret long — tranché au T0, constat au Change Log.
- **Qui la porte** : la seule fonction `preview` (via `render_document`, un paramètre de marque). `get_or_freeze`
  (téléchargement et envoi : le PDF **émis**), `serve_frozen` (le PDF figé) et les rappels (`PdfDocument::Reminder`) ne la
  portent **jamais** (AC-12).
- **Chaque page** : le PDF d'une facture tient aujourd'hui sur **une** page (`draw_invoice_section` refuse au-delà de sa
  capacité, `INVOICE_TOO_MANY_LINES_FOR_PDF`, `pdf.rs:640`) ; la marque se dessine par page, pour qu'un rendu à plusieurs
  pages futur la garde sur chacune.
- **Gardes existantes** : `chaque_texte_s_imprime_en_noir` (`pdf.rs:2777`) tient le rendu **sans marque** et reste
  inchangé ; le rendu marqué a son propre test (tous les textes en noir **sauf** le filigrane, remplissage remis au noir
  après lui).

**Anciens AC-9, AC-11 et AC-12** :

9. **AC-9 — L'aperçu ne fige pas.** `GET /api/v1/invoices/{id}/pdf/preview` sur une facture validée non figée : `200`,
   `application/pdf`, et **rien n'est écrit** — la ligne de la facture (colonnes `pdf_*`, `version`, `updated_at`) est
   égale avant et après, le journal d'audit n'a aucune ligne de plus, `KESH_DOCUMENTS_DIR` aucun fichier de plus ; deux
   appels de suite rendent `200` tous deux, la facture toujours non figée ; **puis** `GET /pdf` fige comme aujourd'hui
   (non-régression : colonnes posées, audit `invoice.pdf_frozen`). Facture figée : l'aperçu rend **les mêmes octets** que
   `/pdf`, sans écriture ; fichier figé retiré : `410 INVOICE_PDF_GONE`. Brouillon : `400 INVOICE_NOT_VALIDATED`. Rôle
   *Consultation* : `200`, sans figement ; clé d'API en lecture : `403 API_KEY_PREVIEW_FORBIDDEN`, rien d'écrit ; autre
   société : `404` **avec le corps JSON d'`AppError`** (code d'erreur vérifié : le `404` du service statique, `lib.rs:280`,
   `:1619`, n'en a pas) ; sans session : `401`.
   Tests dans `crates/kesh-api/tests/invoice_pdf_preview_e2e.rs` (neuf). *Preuve du rouge* : avant le correctif, la route
   tombe dans le service statique (`404` sans corps d'`AppError`) : tous les tests rouges, « autre société » compris par son
   corps ; **mutation** : `preview` qui appelle `get_or_freeze(…, Usage::Download)` → le test
   « rien n'est écrit » rouge.
11. **AC-11 — La marque d'aperçu (D13).** Tests dans `kesh-qrbill` (rendu avec marque, fixture d'une facture de
    **plusieurs lignes**, à la capacité de la page — huit lignes chiffrées sans TVA, la fixture `S3` de
    `rendu_avant_apres.rs:239-246`, ou six avec un taux ; au-delà, le rendu refuse en `INVOICE_TOO_MANY_LINES_FOR_PDF`,
    `pdf.rs:787-820`, et la marque ne libère aucune place) : le texte extrait du PDF (l'extracteur des tests de `pdf.rs`, `couleurs_des_textes`
    / `operations_du_pdf`, ou `kesh_report::texte_pdf` de la PR 46) contient le filigrane et l'en-tête de la marque, et la
    mention du cadre barré ; il ne contient **aucun** des textes de la section de paiement (titres du récépissé et de la
    section de paiement, IBAN formaté, référence) ; aucune matrice QR n'est dessinée (nombre de rectangles pleins de la
    zone QR nul : seuls `draw_qr_matrix`, `pdf.rs:1829`, et la croix, `:1862`, y posent des rectangles remplis ; le
    cadre barré n'est fait que de traits) ; le filigrane est rempli en gris de niveau ≥ celui fixé au T0
    (jamais `0 0 0 rg`) et dessiné **avant** le premier texte du contenu (ordre des opérations du flux) ; tous les autres
    textes sont en noir. Le **même** mécanisme, appelé avec les textes « BROUILLON », rend « BROUILLON » et aucun
    « APERÇU » (test du paramétrage ; aucun PDF de brouillon livré). Une charge utile que l'émission refuse est refusée
    avec marque aussi. Mutations, chacune rouge : filigrane dessiné après le contenu ; filigrane en noir ; section de
    paiement dessinée sous la marque ; texte de marque en dur. Bout en bout (`invoice_pdf_preview_e2e.rs`) : l'aperçu d'une
    facture validée non figée porte la marque dans la langue du client (deux locales), celui d'une facture figée **n'en
    porte pas** (mêmes octets que `/pdf`, AC-9).
12. **AC-12 — Le PDF émis ne change pas.** Sans marque, le rendu garde l'**empreinte** d'aujourd'hui
    (`rendu_avant_apres.rs`, inchangé et vert ; golden de `kesh-qrbill` inchangés et verts, `chaque_texte_s_imprime_en_noir`
    inchangé) ; `GET /pdf`
    après un aperçu fige un PDF **sans** marque (texte extrait : ni filigrane ni en-tête ; section de paiement présente) ;
    un rappel n'en porte pas. Mutation : marque passée par `get_or_freeze` → test rouge.

**Anciens T2ter et T2quater** :

- *(reporté)* **T2ter** — L'aperçu sans figement (D12, AC-9) : `issued_invoice_pdf::preview`, gestionnaire dans
      `routes/invoice_pdf.rs` (garde des clés d'API), route dans `lib.rs` à côté de `/pdf` ; code
      `API_KEY_PREVIEW_FORBIDDEN` (`errors.rs`, quatre locales) ; les trois commentaires rendus faux ; `pdf_status` dans
      `documents_of_entry` (AC-10) ; tests `invoice_pdf_preview_e2e.rs`.
- *(reporté)* **T2quater** — La marque d'un PDF non émis (D13, AC-11, AC-12) : structure de marque et rendu dans
      `kesh-qrbill/src/pdf.rs` (filigrane sous le contenu, en-tête, cadre barré à la place de la section de paiement),
      clés dans `types.rs` (`I18N_KEYS`, `DEFAULT_EN`) et dans les quatre `messages.ftl` de `kesh-i18n` ; paramètre de
      marque de `render_document`, passé par `preview` seulement ; tests et mutations ; recette visuelle.

**Ancienne *Recette visuelle*** :

#### Recette visuelle (ancienne)

Captures (thème clair, `kesh-gate-logs/s55-3a/`) du PDF d'aperçu d'une facture validée non figée **de plusieurs lignes**
(à la capacité de la page, voir l'AC-11), en couleur **et en niveaux de gris** (`pdftoppm -gray`, pour juger l'impression en noir et blanc) : le
filigrane lisible comme marque, le texte de la facture lisible au travers ; l'en-tête ; le cadre barré à la place de la
QR-facture. En regard, le PDF figé de la même facture, sans marque. Jointes au Dev Agent Record et au prompt de la revue
de code.

**Ancien C-55-3a-5** :

- **C-55-3a-5** — Aperçu sans figement (D12, réponse de l'utilisateur du 2026-11-25) : une route neuve plutôt qu'un
  paramètre de `/pdf` (`?preview=1`) — le figement reste le seul effet de `/pdf`, et l'aperçu ne peut pas figer par un
  paramètre oublié. *Écarté* : n'afficher que les factures déjà figées (ancienne alternative de Q7).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

- (`enhancement`, évolution) Une facture client créée par l'API pourra porter un PDF produit hors de Kesh, qui **est** la
  pièce émise (copié, sauvegardé, figé, jamais régénéré ; ni second PDF ni QR-facture de Kesh ; « lien seulement » sur
  demande) ; le modèle de la liste ne l'empêche pas (*Limite connue et évolution*) (Epic 55, story 55-3a). Texte prêt :
  `kesh-gate-logs/s55-decisions/issues-a-envoyer.md`.
- (LOW, `enhancement`) Le détail d'une écriture de règlement fournisseur groupé ne montre qu'un document
  (`find_source_document_of_entry`, `LIMIT 1`) quand le journal les montre tous.

### File List

## Change Log

- 2026-11-25 — Fiche 55-3 écrite au kickoff de l'Epic 55, story ajoutée le même jour (`drafted`).
- 2026-11-25 — Validation P1 (Opus) de la 55-3 : 0/0/10/8. Remédiée par le **découpage** en 55-3a (serveur, cette fiche) et
  55-3b (écran) (V1-5) ; règles des factures annulées (V1-1), règlements groupés client et fournisseur et écart avec le
  détail d'écriture (V1-2), site d'appel hors boucle et borne du grand livre, `fetch_counterparts` n'est plus le patron
  (V1-3), négatifs posés en SQL direct et mutations (V1-7), constats de l'AC-5 et de l'AC-6 (V1-8), contrat aligné sur D9
  (V1-10), LOW V1-11, V1-12, V1-13, V1-18 ; V1-4, V1-6, V1-9, V1-14 à V1-17 vont à la 55-3b.
- 2026-11-25 — Validation P2 (Sonnet) : 0/0/3/8 ; remédiée : littéraux à compléter et fichiers communs (V2-1) ; négatifs
  inter-sociétés par branche (V2-2) ; `docs/api-external.md` (V2-3, AC-7) ; LOW V2-4 à V2-11 (garde de l'extraction PDF,
  fonctions citées au lieu des lignes de `kesh-api`, cinq branches, avoirs relevés au code, rayon d'impact, contrat
  compilable, commande de contrôle, taille M).
- 2026-11-25 — Validation P3 ciblée (Haiku) : 0/0/5/6 ; remédiée : littéraux de production alimentés par la `HashMap`
  (V3-1) ; D9 à cinq branches (V3-2) ; fichiers communs `lib.rs`, `csv.rs`, `docs/api-external.md` corrigés pour les
  Epics 46 et 47 (V3-3, V3-4, V3-7) ; figement par la lecture depuis le Journal soumis au mainteneur (Q7, V3-5) ; LOW V3-6,
  V3-8 à V3-11.
- 2026-11-25 — Validation P4 ciblée (Sonnet) : 0/0/3/5 ; remédiée : Q7 au T0 et sa conséquence (filtre serveur, aucun champ)
  (V4-1, 55-3b et fichier d'epic alignés) ; tests de filtres par appel direct, `s.company_id` et `isi.company_id` (V4-2) ;
  extracteur `texte_pdf` de la PR 46 au lieu de `lopdf` (V4-3) ; LOW V4-4 à V4-8.
- 2026-11-25 — Validation P5 ciblée (Haiku) : 0/0/0/3 — **validation terminée**. Tendance (55-3 puis 55-3a) : P1 Opus
  0/0/10/8 (découpage) → P2 Sonnet 0/0/3/8 → P3 ciblée Haiku 0/0/5/6 → P4 ciblée Sonnet 0/0/3/5 → P5 ciblée Haiku 0/0/0/3.
  Les trois LOW se reprennent au **T0 du développement**, ligne lue citée au Change Log : V5-1 (règle « Q7 refusée » avec
  `i.status IN ('validated','cancelled')` et un négatif « brouillon portant un PDF figé »), V5-2 (AC-3 récrit selon la
  réponse à Q7), V5-3 (statut de la 55-3b confirmé par ce verdict).
- 2026-11-25 — **Récrite sur décision de l'utilisateur** (interface de la 55-3 : une icône par ligne, la liste au clic) :
  booléen `hasDocuments` dans le rapport, route `GET /journal-entries/{id}/documents`, un constructeur commun des
  rattachements (DRY) ; règles de rattachement, négatifs, bornes et contrat additif repris ; statut repassé à `drafted`,
  validation reprise à P1 sur cette version (la numérotation des passes repart : P1 de la version récrite).
- 2026-11-25 — Validation P1 de la version récrite (Opus) : 0/0/4/6 ; remédiée : service `ecritures::justificatifs` (couche
  de services) (Vv2p1-1) ; fichier d'epic mis à jour (en-tête, § *Backend*, fichiers communs) (Vv2p1-2) ; tests d'appel
  direct hors de `src/` (Vv2p1-3) ; gate ciblé `package(kesh-api)` (Vv2p1-4) ; LOW Vv2p1-5 à Vv2p1-9 (Vv2p1-10, lignes à
  relever au T0).
- 2026-11-25 — Validation P2 ciblée de la version récrite (Sonnet) : 0/0/0/5 — **validation terminée**. Tendance de la
  version récrite : P1 Opus 0/0/4/6 → P2 ciblée Sonnet 0/0/0/5. Les cinq LOW se reprennent au **T0 du développement**,
  ligne lue citée au Change Log : Vv2p2-1 (la liste des binaires du gate est indicative ; le gate est `package(kesh-api)`),
  Vv2p2-2 (manuel : titre `:1635`, phrase `:1642`, base des numéros nommée), Vv2p2-3 (`docs/api-external.md` : `:232` à
  `1e23edb55`, `:234` sur la PR 46 ; paragraphe neuf à côté de `:310` et `:1164`), Vv2p2-4 (55-3b AC-4 : « sans route neuve
  **pour les PDF** »), Vv2p2-5 (service au motif R1 à R3 : `(&ContexteAcces, &Lecture, id)`, `acces.exiger(…)?` d'abord,
  `company_for_ids` avant `find_by_id`, portée MCP `None` par défaut ; types de règlement `write_off`, `internal_account`,
  `bank_transfer` ; `INVOICE_NOT_PDF_READY` est un `400`).
- 2026-11-25 — **Réponses de l'utilisateur versées** (partie A, points 3 et 4) : D10 confirmée (règlement client et
  fournisseur → la facture réglée ; écriture d'une facture client → son PDF) ; **D12** : l'affichage depuis le Journal et
  les Rapports ne fige pas — aucune voie d'aperçu sans figement n'existant au code, la 55-3a crée
  `GET /invoices/{id}/pdf/preview` (AC-9) et `pdfStatus` dans la liste (AC-10) ; l'alternative « Q7 refusée » est retirée ;
  fiche de la facture inchangée, question de son affichage au mainteneur (Q8) ; limite connue et évolution du PDF produit
  hors de Kesh, issue à ouvrir. Validation P3 ciblée sur ces changements.
- 2026-11-25 — Validation Pv2p3 ciblée sur les réponses de l'utilisateur (Opus) : 0/0/2/7 ; remédiée : dérogation à
  l'arbitrage 8 écrite, aperçu **fermé aux clés d'API** (`403 API_KEY_PREVIEW_FORBIDDEN`) (Vv2p3-1) ; Vv2p3-2 (deux
  phrases du § 11.6.1 rendues fausses) porte sur le manuel, qui est à la 55-3b : renvoyé à son R-2 (la 55-3b est au-delà
  de la sixième passe) ; LOW : commentaires rendus faux (Vv2p3-3, et Vv2p7-2 de la 55-3b), `404` vérifié par son corps
  (Vv2p3-4), nom du fichier à l'usage des appels directs (Vv2p3-5), objectif 3 du fichier d'epic (Vv2p3-6), taille L et
  modules réévalués (Vv2p3-7), course écrite (Vv2p3-8), table des codes au T3 (Vv2p3-9). Passe suivante : Pv2p4 ciblée.
- 2026-11-25 — Validation Pv2p4 ciblée (Sonnet) : 0/0/0/4 — **validation terminée**. Tendance de la version récrite :
  P1 Opus 0/0/4/6 → P2 ciblée Sonnet 0/0/0/5 → (réponses de l'utilisateur) Pv2p3 ciblée Opus 0/0/2/7 → Pv2p4 ciblée
  Sonnet 0/0/0/4. Les quatre LOW se reprennent au **T0 du développement**, ligne lue citée au Change Log : Vv2p4-1
  (réemployer `ApiKeyOperationForbidden`, `403 API_KEY_OPERATION_FORBIDDEN`, `errors.rs:261-277`, au lieu d'un code neuf —
  `kesh-i18n` et `errors.rs` sortiraient alors du périmètre), Vv2p4-2 (ajouter au T2ter les deux sites de l'« arbitrage 8 »,
  `issued_invoice_pdf.rs:37-39`, `invoice_pdf.rs:8-13`), Vv2p4-3 (« `404` sans corps » vrai sous le harnais
  `"nonexistent-static-dir"` seulement), Vv2p4-4 (clé `read-write` aussi `403` ; garde avant `get_company_for` et `load`,
  cas « clé sur une facture d'une autre société ou à fichier figé retiré : `403`, non `404`/`410` », mutation).
- 2026-11-25 — **Décisions du mainteneur versées** (fichier d'epic, § *Décisions du mainteneur*) : Q1 acceptée sous
  conditions (en-tête ; figée : PDF figé tel quel, sans marque ; rendu d'aperçu pour les seules validées non figées) ;
  **Q10** : la marque d'un PDF non émis, section neuve (filigrane « APERÇU » sous le contenu, en gris clair ; en-tête
  « Aperçu — non émis » ; QR-facture barrée, cadre barré à la place de la section de paiement, charge utile validée ;
  un mécanisme paramétré par ses textes dans `kesh-qrbill`, prêt pour « BROUILLON », non livré), AC-11, AC-12, T2quater,
  *Recette visuelle* (facture de plusieurs lignes) ; dérogation à l'arbitrage 8 réglée par la marque, course réécrite ;
  **Q8** refusée (D14) ; Q5 (P4) ; Q6 (gate complet et E2E complet à la clôture de cette story) ; règles du PDF produit hors
  de Kesh décidées ; modules réévalués, taille **XL**, signalée sans découpage. Validation Pv2p5
  ciblée sur ces changements.
- 2026-11-25 — Validation Pv2p5 ciblée (Opus) : 0/0/2/8 (`kesh-gate-logs/s55-decisions/55-3a-validation-pv2p5.md`) ;
  remédiée : fixture à la capacité de la page (huit lignes sans TVA, six avec un taux) au lieu de « au moins dix »
  (V5p-1) ; prémisse « aucun autre signe que l'absence de figement » retirée — `emailed_at` et l'empreinte au journal
  d'audit en sont deux ; aperçu marqué gardé comme lecture de Q1, **soumise au mainteneur** (V5p-2, voie b) ; LOW : taille
  écrite comme signalement décidé, non comme question (V5p-3) ; chaîne d'appel, empreinte au lieu des octets, règle de
  `texte.rs:32` (V5p-4) ; qui remplit la marque, deux voies au T0 (V5p-5) ; test des quatre locales et `package(kesh-i18n)`
  (V5p-6) ; preuve du grep du *Hors périmètre* (V5p-7) ; `sprint-status.yaml`, ligne de l'epic (V5p-8) ; mention du cadre
  écrite comme proposition, modules « au seuil » (V5p-9) ; branche vague de l'AC-11 retirée (V5p-10). Fichier d'epic aussi
  remédié pour les passes des 55-2 (V7-2) et 55-4 (V8-1 : argument de `gate-ci.sh`). Passe suivante : Pv2p6 ciblée,
  dernière remédiable.
- 2026-11-25 — Validation Pv2p6 ciblée (Sonnet) sur la remédiation de la Pv2p5 : 0/0/0/4 — **validation terminée**.
  Tendance depuis les décisions du mainteneur : Pv2p5 ciblée Opus 0/0/2/8 → Pv2p6 ciblée Sonnet 0/0/0/4. Les quatre LOW
  (`kesh-gate-logs/s55-decisions/55-3a-validation-pv2p6.md`) ne sont pas remédiés (une remédiation après la dernière passe
  ne serait relue par personne) : ils se reprennent au **T0 du développement**, ligne lue citée au Change Log — Pv2p6-1
  (fichier d'epic, fichiers communs : « à l'octet près » → « même empreinte, `rendu_avant_apres.rs` »), Pv2p6-2 (citer
  `crates/kesh-db/src/repositories/invoices.rs` ; la dévalidation est refusée à une facture envoyée, `errors.rs:443` : le
  cas « dévalidée puis revalidée » ne vise qu'une facture téléchargée), Pv2p6-3 (test des quatre locales écrit à l'AC-11 ;
  T2quater cite les deux voies), Pv2p6-4 (AC-9 : une validée non figée à `emailed_at` non nul rend `200`, marquée, rien
  d'écrit — selon la réponse du mainteneur à la question *Aperçu d'un PDF déjà sorti*).
- 2026-11-26 — **Décisions du mainteneur du 2026-11-26 versées** (fichier d'epic, § *Décisions du mainteneur
  (2026-11-26)*) : les **PDF marqués sortent de l'Epic 55** pour un epic à part, « PDF marqués » (numéro à confirmer) — la
  route `GET /invoices/{id}/pdf/preview`, la marque (D13), AC-9, AC-11, AC-12, T2ter (sa partie aperçu), T2quater, la
  recette visuelle et C-55-3a-5 sont rangés, sans être supprimés, en section *Reporté à l'epic « PDF marqués »* ; un PDF
  déjà sorti sans être figé y est décidé (aperçu marqué, bandeau « sorti le … ») ; dans l'Epic 55, une telle facture n'est
  pas figée (`notFrozen`, « PDF pas encore émis », aucune nuance de texte) ; § neuf *Afficher sans figer, dans l'Epic 55*
  (facture figée : `/pdf` sert le PDF figé sans rien écrire ; pas encore émise : aucune lecture) ; AC-9 récrit en constat
  « aucune route de PDF neuve », AC-10 complété (`emailed_at`) ; `pdfStatus` inchangé (`frozen`, `notFrozen`), énumération
  ouverte à l'aperçu à venir et au PDF externe ; `kesh-qrbill`, `kesh-i18n`, `errors.rs` et les fichiers du PDF de la
  facture hors périmètre ; taille ramenée de **XL** à **M**. Passe suivante : Pv2p7 ciblée (septième de la version
  récrite : au-delà de la sixième, ses points vont au T0).
