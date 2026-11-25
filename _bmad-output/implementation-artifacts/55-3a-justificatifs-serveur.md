# Story 55-3a : les justificatifs d'une écriture dans la réponse du journal et du grand livre (serveur)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 (Q1), D10 (Q4), règle des factures (Q7). Issue : #À-VENIR
(`enhancement`, priorité à fixer, Q5 ; point c, convenu au téléphone le 2026-11-25 entre l'utilisateur et le mainteneur).
Release : la prochaine planifiée après la 0.20.0. **Aucune migration. ⚠️ Backend** (`kesh-report`, `kesh-api`) : un
booléen additif dans la réponse JSON du journal et du grand livre, et **une route de lecture minimale** — l'epic était
annoncé « frontend seulement » ;
**à confirmer par le mainteneur (Q1)** avant le développement ; la règle des factures dépend de **Q7** (figement à
l'affichage).

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
au grand livre (colonne `reports-ledger-column-piece`, manuel `user-manual.tex:4192`).

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
| PDF d'une facture client (écriture de vente) | `invoices.journal_entry_id` (`20260417000001_invoice_validation.sql`) | la route sert le PDF **figé** s'il existe, sinon fige une facture **validée**, sinon `400 INVOICE_NOT_VALIDATED` (`crates/kesh-api/src/routes/issued_invoice_pdf.rs:104-109`). Une facture annulée par un avoir **garde** `journal_entry_id` (`crates/kesh-db/src/repositories/credit_notes.rs:843`) ; seule la dévalidation le vide (`crates/kesh-db/src/repositories/invoices.rs:1782`). Manuel § 11.6.1 (`user-manual.tex:1635`) : « une facture annulée par un avoir garde le PDF qu'elle a émis ; … jamais figée … n'a pas de PDF » | `GET /api/v1/invoices/{id}/pdf` (`lib.rs:1262` ; `:1304` sur la PR 46), tout rôle |
| idem, écriture de **règlement** | `invoice_settlements.journal_entry_id` → `invoice_id` (`20260827000001_invoice_settlements.sql` ; plusieurs factures par écriture depuis `20261118000001_grouped_settlements.sql`, clé `(journal_entry_id, invoice_id)`). Un règlement annulé est **supprimé** (`invoice_settlements_write.rs:1140`, `:972` groupé) | même règle que la ligne précédente | idem |
| PDF d'un **avoir** | `credit_notes.journal_entry_id` (`20260627000001_credit_notes.sql:36` ; `CHECK` : émis ⇒ écriture, `:54`) ; statuts `draft`, `issued`, `cancelled` (`:50`) | rendu à la demande, sans contrôle de statut (`routes/credit_notes.rs:279-330`) | `GET /api/v1/credit-notes/{id}/pdf` (`lib.rs:1275` ; `:1317`), tout rôle |
| **Document** d'une facture fournisseur | `supplier_invoices.purchase_journal_entry_id` ou `settlement_journal_entry_id` (`20260628000001`), ← `imported_supplier_invoices.supplier_invoice_id`, `status = 'completed'` (`20260629000001`). Règlement fournisseur **groupé** : plusieurs factures portent le même `settlement_journal_entry_id` (`20261118000001`, `CHECK` « ligne ⇒ écriture » ; `grouped_settlements.rs:9`) | fichier archivé | `GET /api/v1/supplier-invoices/{id}/source-document` (`lib.rs:1291` ; `:1333`), tout rôle (30-2c AC22) |

- **Précédent** : `kesh-db/src/repositories/imported_supplier_invoices.rs:409-433` `find_source_document_of_entry` — achat
  **ou** règlement, `completed`, pour **une** écriture et **un** document (`ORDER BY si.id LIMIT 1`, `:425`) ; le détail
  d'une écriture l'affiche (`routes/journal_entries.rs:250,512` ; écran `journal-entries/[id]/+page.svelte:589-620`). Un
  règlement détaché ne montre plus le document ; l'achat d'une facture annulée le montre (test
  `cancelled_invoices_take_a_document_and_the_detached_settlement_loses_it`).
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
}
```

TypeScript (55-3b) : `interface EntryDocument { kind: 'invoice' | 'creditNote' | 'supplierDocument'; id: number; number:
string | null; fileName?: string }`.

### Règles de rattachement

- **Facture** (vente et règlement) : `i.status = 'validated' OR (i.status = 'cancelled' AND i.pdf_storage_path IS NOT
  NULL)` — exactement les factures dont la route sert un PDF ; une facture annulée jamais figée n'a pas de justificatif.
  **Si Q7 est refusée** (ne pas figer à l'affichage) : `i.pdf_storage_path IS NOT NULL` seulement (facture validée ou
  annulée **déjà figée**) — un filtre serveur, **aucun champ de plus au contrat** ; AC-3 perd « facture validée » non
  figée, gagne « facture validée non figée : aucun justificatif ».
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
   constructeur : test qui les compare sur tous les cas). Rôle *Consultation* : `200` ; écriture d'une autre société :
   `404` ; sans session : `401`.
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
   ':[0-9]+:\s*//'` rend une définition et deux appels) ; grand livre à plusieurs comptes : le booléen de chaque section est
   juste ; une écriture hors période n'est pas lue.
6. **AC-6 — Contrat additif.** Une écriture sans justificatif sérialise **exactement** le JSON d'aujourd'hui (clés
   comparées) ; les exports PDF et CSV du journal et du grand livre rendent le même résultat avec `hasDocuments` vrai ou
   faux (CSV : chaînes égales ; PDF : texte lu par `kesh_report::texte_pdf::lignes` de la PR 46, après avoir vérifié qu'il
   contient un numéro d'écriture connu) ; garde de non-régression, déclarée telle. L'outil MCP du grand livre recopie ses
   champs : sortie inchangée par construction, `mcp_contrat` vert comme filet.
7. **AC-7 — Documentation de l'API.** `docs/api-external.md` : le champ `hasDocuments` aux paragraphes du grand livre
   (`:310` à `1e23edb55`) et du journal (`:1164`), et la route `GET /journal-entries/{id}/documents` (forme, genres, droits,
   `404`), lignes relevées sur la base PR 46.
8. **AC-8 — Note pour l'écran.** Les deux contrats TypeScript ci-dessus sont recopiés dans la fiche 55-3b à l'identique.

## Tâches

- [ ] **T0** — Q1 et Q7 tranchées par le mainteneur (sinon arrêt ; Q7 fixe la règle des factures) ; base `epic/55-navigation` (PR 46 reprise) ; relever le sens
      d'un avoir `cancelled` et de son écriture ; vérifier les lignes SQL directes de l'AC-4 contre les contraintes.
- [ ] **T1** — Tests rouges d'abord (AC-3 à AC-6) ; journal.
- [ ] **T2** — `documents.rs` (constructeur commun, deux fonctions) ; `has_documents` du journal et du grand livre, appels
      hors boucle ; les deux littéraux de **production** (`journal_report.rs:232`, `general_ledger.rs:901`) le lisent dans
      l'ensemble (`contains(&entry_id)`) ; les littéraux de **test et de banc** reçoivent `has_documents: false` et rien
      d'autre (règle
      de fichier commun pour `csv.rs`, `pdf.rs`, `benches/export.rs`, réécrits par les Epics 46 et 47).
- [ ] **T2bis** — Route `GET /api/v1/journal-entries/{id}/documents` (`kesh-api`, lectures « tout rôle », vérification de
      la société comme le détail) et ses tests d'intégration (`crates/kesh-api/tests/journal_entry_documents_e2e.rs`, neuf).
- [ ] **T3** — `docs/api-external.md` (AC-7).
- [ ] **T4** — Gates.

## Gates

Règle de l'epic (fichier d'epic, § *Gates*) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D
warnings` ; `cargo nextest run -E 'package(kesh-report) | binary(reports_e2e) | binary(reports_export_e2e) | binary(journal_entry_documents_e2e) | binary(fx_recette_api_e2e)
| binary(open_items_e2e) | binary(dates_bornes_e2e) | binary(mcp_contrat)'` (rayon d'impact : binaires qui lisent le
journal ou le grand livre) ;
`scripts/gate-ci.sh` en fin d'epic ; E2E selon le § *Gates* du fichier d'epic (Q6 non tranchée ou refusée : E2E complet au dernier commit de code de la story). Aucun fichier de `kesh-db` touché (sinon gate complet, `CLAUDE.md` § *Exception
`kesh-db`*).

## Note de version

API : les écritures du journal et les lignes du grand livre portent `hasDocuments` (vrai quand l'écriture a au moins un
justificatif : facture, avoir, document de facture fournisseur ; absent sinon) ; `GET /journal-entries/{id}/documents` rend
la liste des justificatifs d'une écriture ; exports PDF et CSV inchangés. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-3a-1** — Un booléen dans le rapport, la liste au clic (décision de l'utilisateur du 2026-11-25) : le rapport reste
  léger, la liste ne se lit que pour l'écriture ouverte. *Écarté* : la liste dans le rapport (version précédente de la
  fiche), et une route groupée `?ids=…` (inutile : on n'ouvre qu'une écriture à la fois).
- **C-55-3a-2** — Le règlement montre la facture réglée (D10), comme `find_source_document_of_entry` pour le fournisseur.
- **C-55-3a-3** — Pas de justificatif libre joint à une écriture manuelle : il n'en existe pas dans Kesh ; hors périmètre.
- **C-55-3a-4** — Borne du grand livre par période seule (voir *La requête*), quelle que soit la page (`limit`, `offset`)
  : la requête lit les justificatifs de toute la période, une page n'en montre qu'une partie.

### Modules (règle de splitting préventif)

Deux modules de production : `kesh-report` (`documents.rs`, `journal_report.rs`, `general_ledger.rs`, `lib.rs`, et les
littéraux de test de `csv.rs`, `pdf.rs`, `benches/export.rs`) et `kesh-api` (la route, `lib.rs` et `routes/journal_entries.rs`)
; plus `docs/api-external.md` et les tests `kesh-api/tests/`.

### References

- Fiche 30-2c (document d'une facture fournisseur, lecture par tout rôle) ; fiche 25-6-b (PDF figé) ; Story 43-1a/43-1b
  (règlements groupés).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

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
