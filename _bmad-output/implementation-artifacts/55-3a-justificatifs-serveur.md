# Story 55-3a : les justificatifs d'une écriture dans la réponse du journal et du grand livre (serveur)

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 (Q1), D10, D12 (aperçu sans figement). Issue : #À-VENIR
(`enhancement`, priorité à fixer, Q5 ; point c, convenu au téléphone le 2026-11-25 entre l'utilisateur et le mainteneur).
Release : la prochaine planifiée après la 0.20.0. **Aucune migration. ⚠️ Backend** (`kesh-report`, `kesh-api`) : un
booléen additif dans la réponse JSON du journal et du grand livre, et **une route de lecture minimale** — l'epic était
annoncé « frontend seulement » ;
**à confirmer par le mainteneur (Q1)** avant le développement. *Réponses de l'utilisateur du 2026-11-25 versées* (D10 :
le règlement montre la facture réglée, client et fournisseur ; D12 : depuis le Journal et les Rapports, **afficher une
facture ne la fige pas** — d'où une **route d'aperçu sans figement**, `GET /api/v1/invoices/{id}/pdf/preview`, que la
55-3a crée, aucune n'existant aujourd'hui).

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
| PDF d'une facture client (écriture de vente) | `invoices.journal_entry_id` (`20260417000001_invoice_validation.sql`) | la route sert le PDF **figé** s'il existe, sinon fige une facture **validée**, sinon `400 INVOICE_NOT_VALIDATED` (`crates/kesh-api/src/routes/issued_invoice_pdf.rs:104-109`). Une facture annulée par un avoir **garde** `journal_entry_id` (`crates/kesh-db/src/repositories/credit_notes.rs:843`) ; seule la dévalidation le vide (`crates/kesh-db/src/repositories/invoices.rs:1782`). Manuel § 11.6.1 (`user-manual.tex:1635`) : « une facture annulée par un avoir garde le PDF qu'elle a émis ; … jamais figée … n'a pas de PDF » | `GET /api/v1/invoices/{id}/pdf` (`lib.rs:1262` ; `:1304` sur la PR 46), tout rôle — **fige** au premier appel ; l'écran des Rapports passe par l'aperçu neuf (D12, *L'aperçu sans figement*) |
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
    /// (aucun PDF émis : l'écran montre un aperçu, avec son bandeau, 55-3b). Énumération
    /// **ouverte** : une valeur future (`external`, PDF produit hors de Kesh) s'y ajoute sans
    /// rien changer d'autre (*Limite connue et évolution*).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_status: Option<PdfStatus>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PdfStatus { Frozen, NotFrozen }
```

TypeScript (55-3b) : `interface EntryDocument { kind: 'invoice' | 'creditNote' | 'supplierDocument'; id: number; number:
string | null; fileName?: string; pdfStatus?: string }` — `pdfStatus` typé `string` et non en union fermée : l'écran ne
montre le bandeau que pour `'notFrozen'`, toute autre valeur (y compris une valeur future) n'en montre pas.

`pdf_status` se lit dans la même requête que la liste (`i.pdf_storage_path IS NOT NULL` ⇒ `frozen`, la règle
d'`is_frozen`, `issued_invoice_pdf.rs:325`) ; il vaut **à la lecture de la liste** : une facture figée entre la lecture et
l'affichage (téléchargement depuis la fiche par un autre utilisateur) garde son bandeau jusqu'à la prochaine ouverture —
limite assumée, écrite au manuel par la 55-3b.

### L'aperçu sans figement (D12)

`GET /api/v1/invoices/{id}/pdf/preview` — **neuve**, montée à côté de `GET /api/v1/invoices/{id}/pdf` (`lib.rs:1262`),
tout rôle authentifié (*Consultation* compris), **fermée aux clés d'API**, bornée à la société (`404` sinon, comme `/pdf`) :

- ⚠️ **Dérogation écrite à l'arbitrage 8 de la 25-6-b** (« une pièce sortie de Kesh est figée, qui que ce soit qui l'ait
  fait sortir », `issued_invoice_pdf.rs:37-39`, `invoice_pdf.rs:11-13`, manuel `user-manual.tex:1641`) : l'aperçu fait
  sortir vers l'écran un PDF de facture **non émis**. Motif : la réponse de l'utilisateur (afficher n'est pas figer),
  bornée à l'**affichage dans Kesh** ; le bandeau (55-3b) dit que ce n'est pas le PDF émis. Le lecteur du navigateur
  permet encore de télécharger l'aperçu : Q10 au mainteneur.
- **Clés d'API : `403`** — une clé n'affiche rien ; ouvrir l'aperçu aux clés ferait sortir des PDF non figés vers un
  client d'API, au-delà de la réponse de l'utilisateur. Garde au gestionnaire sur `current_user.api_key_id` (le
  discriminant de `require_not_pat`, `middleware/rbac.rs:58-67`), code neuf `API_KEY_PREVIEW_FORBIDDEN`, message dans les
  quatre locales (`kesh-i18n`). Rouvrir aux clés serait un choix du mainteneur (Q1).

- **facture figée** : le fichier figé, par `serve_frozen` (même vérification d'empreinte ; `410 INVOICE_PDF_GONE` si le
  fichier manque, `500` s'il est altéré) — octet pour octet celui de `/pdf` ;
- **facture validée non figée** : `render_document(…, PdfDocument::Invoice)` dans la langue du client (`client_language`,
  la règle du figement), **sans aucune écriture** : ni fichier dans `KESH_DOCUMENTS_DIR`, ni colonne `pdf_*`, ni
  `version`, ni trace d'audit (`invoice.pdf_frozen` ni aucune autre) ; les erreurs de rendu sont celles de `/pdf`
  (`400 INVOICE_NOT_PDF_READY`, …) ;
- **brouillon, facture annulée jamais figée** : `400 INVOICE_NOT_VALIDATED`, comme `/pdf` ;
- `Content-Disposition: inline; filename="apercu-facture-<numéro>.pdf"` pour un rendu non figé, `facture-<numéro>.pdf`
  pour le fichier figé — en-tête pour un appel direct ; l'écran lit le PDF par `getBlob` et nomme lui-même le document
  (55-3b, AC-3) ;
- **course** : un `GET /pdf` concurrent peut figer la facture entre la lecture et le rendu de l'aperçu ; l'aperçu rend
  alors un rendu neuf, non le PDF émis. Limite assumée (le bandeau dit « aperçu »), sans relecture après le rendu ; écrite
  à la note de l'API (T3) ;
- une fonction `preview(ctx, invoice_id)` dans `issued_invoice_pdf.rs`, qui réemploie `load`, `is_frozen`,
  `serve_frozen` et `client_language` (DRY) ; `get_or_freeze` et `Usage` **inchangés** ; le gestionnaire dans
  `routes/invoice_pdf.rs`, sur le modèle de `get_invoice_pdf` (`:27-67`).
- **Commentaires rendus faux, corrigés au T2ter** : `issued_invoice_pdf.rs:4-5` (« le **seul** chemin vers le PDF d'une
  facture »), `invoice_pdf_service.rs:274-278` (« ⛔ … la facture ne se rend plus directement ») et `invoice_pdf.rs:1`
  (un seul endpoint) : l'aperçu est le seul autre appelant légitime de `render_document(…, PdfDocument::Invoice)`, son
  rendu **n'est pas** le document émis et n'est jamais posé.
- **Fiche de la facture inchangée** : son téléchargement continue de figer (`/pdf`, `Usage::Download`). Faut-il que
  l'**affichage** sur la fiche passe aussi par l'aperçu, avec le même bandeau ? Question au mainteneur (Q8 du fichier
  d'epic), **non tranchée ici**.

### Limite connue et évolution : un PDF produit hors de Kesh

Un jour, une facture client créée par l'API pourra porter un **PDF produit hors de Kesh** ; c'est lui qui devra
s'afficher, à la place du PDF que Kesh génère. **Non livré ici.** Le modèle ne l'empêche pas : un élément de la liste
désigne la **facture** (`kind`, `id`), non un fichier ; c'est la route de lecture qui choisit le fichier servi ; et
`pdfStatus` est une énumération ouverte, à laquelle une valeur (`external`) s'ajoutera, que l'écran d'aujourd'hui traite
déjà (aucun bandeau). Issue `enhancement` à ouvrir (section *Issues à ouvrir*).

### Règles de rattachement

- **Facture** (vente et règlement) : `i.status = 'validated' OR (i.status = 'cancelled' AND i.pdf_storage_path IS NOT
  NULL)` — les factures pour lesquelles la route **peut** servir un PDF (son rendu peut encore être refusé : `INVOICE_NOT_PDF_READY`, `410` si le fichier figé manque, `409` ; l'écran affiche alors l'erreur, 55-3b AC-4) ; une facture annulée jamais figée n'a pas de justificatif.
  Une facture **validée non figée** reste un justificatif : l'écran l'affiche par l'aperçu, sans la figer (D12).
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
10. **AC-10 — État du PDF dans la liste.** `GET /journal-entries/{id}/documents` : une facture validée non figée porte
    `pdfStatus: "notFrozen"`, une facture figée (validée ou annulée) `"frozen"` ; un avoir et un document fournisseur
    n'ont pas le champ. Après un `GET /pdf`, la même écriture rend `"frozen"`. Mutation : `pdf_status` toujours `frozen` →
    test rouge.

## Tâches

- [ ] **T0** — Q1 tranchée par le mainteneur (sinon arrêt) ; réponses de l'utilisateur relues (D10, D12) ; base `epic/55-navigation` (PR 46 reprise) ; relever le sens
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
- [ ] **T2ter** — L'aperçu sans figement (D12, AC-9) : `issued_invoice_pdf::preview`, gestionnaire dans
      `routes/invoice_pdf.rs` (garde des clés d'API), route dans `lib.rs` à côté de `/pdf` ; code
      `API_KEY_PREVIEW_FORBIDDEN` (`errors.rs`, quatre locales) ; les trois commentaires rendus faux ; `pdf_status` dans
      `documents_of_entry` (AC-10) ; tests `invoice_pdf_preview_e2e.rs`.
- [ ] **T3** — `docs/api-external.md` (AC-7, et la route d'aperçu : forme, droits, fermée aux clés, « n'écrit rien »,
      course ; `pdfStatus`) ; la route ajoutée aux lignes de la table des codes d'erreur pour `INVOICE_NOT_PDF_READY`
      (`:1398` à `1e23edb55`), `INVOICE_NOT_VALIDATED` et `INVOICE_PDF_GONE`, et le code neuf `API_KEY_PREVIEW_FORBIDDEN`
      ; lignes relevées sur la base PR 46.
- [ ] **T4** — Gates.

## Gates

Règle de l'epic (fichier d'epic, § *Gates*) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D
warnings` ; `cargo nextest run -E 'package(kesh-report) | package(kesh-api)'` (rayon d'impact : tout `kesh-api`, la liste des binaires
qui lisent le journal ou le grand livre étant longue — `reports_e2e`, `reports_export_e2e`, `fx_recette_api_e2e`,
`open_items_e2e`, `dates_bornes_e2e`, `services_motif`, `mcp_contrat`, `mcp_outils_e2e`, `mcp_enveloppe_e2e`,
`mcp_transport_e2e`, `journal_entry_reversal_e2e`, les tests du lib ; relevé par `git grep -lE
"reports/(journals|general-ledger)|journal_report|general_ledger" crates/kesh-api`) ;
`scripts/gate-ci.sh` en fin d'epic ; E2E selon le § *Gates* du fichier d'epic (Q6 non tranchée ou refusée : E2E complet au dernier commit de code de la story). Aucun fichier de `kesh-db` touché (sinon gate complet, `CLAUDE.md` § *Exception
`kesh-db`*).

## Note de version

API : les écritures du journal et les lignes du grand livre portent `hasDocuments` (vrai quand l'écriture a au moins un
justificatif : facture, avoir, document de facture fournisseur ; absent sinon) ; `GET /journal-entries/{id}/documents` rend
la liste des justificatifs d'une écriture, avec l'état du PDF d'une facture (`pdfStatus`) ;
`GET /invoices/{id}/pdf/preview` rend le PDF d'une facture **sans le figer** (le fichier figé s'il existe, un aperçu
sinon), et n'écrit rien ; `GET /invoices/{id}/pdf` est inchangé ; exports PDF et CSV inchangés. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-3a-1** — Un booléen dans le rapport, la liste au clic (décision de l'utilisateur du 2026-11-25) : le rapport reste
  léger, la liste ne se lit que pour l'écriture ouverte. *Écarté* : la liste dans le rapport (version précédente de la
  fiche), et une route groupée `?ids=…` (inutile : on n'ouvre qu'une écriture à la fois).
- **C-55-3a-2** — Le règlement montre la facture réglée (D10), comme `find_source_document_of_entry` pour le fournisseur.
- **C-55-3a-3** — Pas de justificatif libre joint à une écriture manuelle : il n'en existe pas dans Kesh ; hors périmètre.
- **C-55-3a-5** — Aperçu sans figement (D12, réponse de l'utilisateur du 2026-11-25) : une route neuve plutôt qu'un
  paramètre de `/pdf` (`?preview=1`) — le figement reste le seul effet de `/pdf`, et l'aperçu ne peut pas figer par un
  paramètre oublié. *Écarté* : n'afficher que les factures déjà figées (ancienne alternative de Q7).
- **C-55-3a-4** — Borne du grand livre par période seule (voir *La requête*), quelle que soit la page (`limit`, `offset`)
  : la requête lit les justificatifs de toute la période, une page n'en montre qu'une partie.

### Modules (règle de splitting préventif)

Deux modules de production : `kesh-report` (`documents.rs`, `journal_report.rs`, `general_ledger.rs`, `lib.rs`, et les
littéraux de test de `csv.rs`, `pdf.rs`, `benches/export.rs`) et `kesh-api` (le service `services/ecritures.rs`, la route
`routes/journal_entries.rs`, l'aperçu `routes/issued_invoice_pdf.rs`, `routes/invoice_pdf.rs` et le commentaire d'`invoice_pdf_service.rs`, `errors.rs`, `lib.rs`), plus `kesh-i18n` (le message du code neuf, quatre locales) ; *réévalué à la validation Pv2p3 (Vv2p3-7)* : trois modules de production, sous le seuil de cinq ; taille portée de M à **L** ; l'aperçu ne dépend pas de la liste et pourrait sortir en story propre, ce qui ferait passer l'epic au-delà de la borne de six : choix laissé au mainteneur (Q1) ; plus `docs/api-external.md` et les tests `kesh-report/tests/`, `kesh-api/tests/`.

### References

- Fiche 30-2c (document d'une facture fournisseur, lecture par tout rôle) ; fiche 25-6-b (PDF figé) ; Story 43-1a/43-1b
  (règlements groupés).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

- (`enhancement`, évolution) Une facture client créée par l'API pourra porter un PDF produit hors de Kesh, à afficher à la
  place du PDF généré (aperçu, justificatifs du Journal, fiche) ; le modèle de la liste ne l'empêche pas (*Limite connue
  et évolution*) (Epic 55, story 55-3a).
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
