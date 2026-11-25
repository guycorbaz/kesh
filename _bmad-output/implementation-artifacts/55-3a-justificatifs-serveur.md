# Story 55-3a : les justificatifs d'une écriture dans la réponse du journal et du grand livre (serveur)

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D9 (Q1), D10 (Q4). Issue : #À-VENIR
(`enhancement`, priorité à fixer, Q5 ; point c, convenu au téléphone le 2026-11-25 entre l'utilisateur et le mainteneur).
Release : la prochaine planifiée après la 0.20.0. **Aucune migration, aucune route neuve. ⚠️ Backend** (`kesh-report`) :
un champ additif dans la réponse JSON du journal et du grand livre — l'epic était annoncé « frontend seulement » ;
**à confirmer par le mainteneur (Q1)** avant le développement.

*Découpée de la 55-3 à la validation P1 (V1-5 : plus de cinq modules pour une seule story) : 55-3a le serveur, 55-3b
l'écran. Le Journal reste le critère principal (demande de l'utilisateur) ; le Grand livre suit, à coût marginal.*

## Story

En tant que comptable qui relit son journal,
je veux que chaque écriture du journal et du grand livre dise quels documents la justifient — PDF de la facture, de
l'avoir, document de la facture fournisseur —,
afin que l'écran puisse me les ouvrir d'un clic (55-3b).

## Vocabulaire

« **Justificatif** » partout (textes, manuel, noms de code `documents`) : « pièce » désigne déjà le **numéro d'écriture**
au grand livre (colonne `reports-ledger-column-piece`, manuel `user-manual.tex:4192`).

## Où un justificatif a un sens

| Rapport | Lignes | Champ `documents` |
|---|---|---|
| **Journal** | une écriture (`JournalEntryRow`) | **oui — le critère principal** |
| **Grand livre** | une ligne d'écriture d'un compte (`LedgerLine`) | **oui**, même fonction, coût marginal (un appel, un champ) ; *option écrite* : si le T0 montre un coût au-delà, le Grand livre sort de la 55-3a et de la 55-3b et s'ouvre en issue |
| Bilan, Compte de résultat, Balance, TVA | un compte, un taux | **non** : une ligne de compte agrège des écritures ; le chemin est le lien de compte (55-2) puis le grand livre |

## Ce qui se lit au code (`origin/main` `1e23edb55`)

**Aucune table de pièces jointes.** Trois documents se rattachent à une écriture, par **cinq** colonnes (quatre branches
de requête) :

| Document | Rattachement | Règle du document servi | Route de lecture (droits) |
|---|---|---|---|
| PDF d'une facture client (écriture de vente) | `invoices.journal_entry_id` (`20260417000001_invoice_validation.sql`) | la route sert le PDF **figé** s'il existe, sinon fige une facture **validée**, sinon `400 INVOICE_NOT_VALIDATED` (`kesh-api/src/routes/issued_invoice_pdf.rs:104-109`). Une facture annulée par un avoir **garde** `journal_entry_id` (`kesh-db/src/repositories/credit_notes.rs:843`) ; seule la dévalidation le vide (`invoices.rs:1782`). Manuel § 11.6.1 (`user-manual.tex:1635`) : « une facture annulée par un avoir garde le PDF qu'elle a émis ; … jamais figée … n'a pas de PDF » | `GET /api/v1/invoices/{id}/pdf` (`lib.rs:1262` ; `:1304` sur la PR 46), tout rôle |
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
  `:231`) : aucun champ de justificatif. Les routes les rendent en JSON telles quelles (`routes/reports.rs:363-366`,
  `:285-291`) ; les exports passent par les mêmes `generate` (`services/rapports.rs:664`, `:718`) et l'outil MCP du grand
  livre aussi (`mcp/outils/grand_livre.rs:159`), qui recopie ses champs dans sa propre structure (`:175`).
- `fetch_counterparts` (`general_ledger.rs:639-690`) fait **une requête par compte**, appelée dans `for acc in accounts`
  (`:825`, `:861`), avec une liste `IN` de taille variable : **ce n'est pas le patron** de la 55-3a.
- Index `idx_journal_entries_company_date (company_id, entry_date DESC)` (`20260412000001_journal_entries.sql:33`).
- L'Epic 47 ne touche ni `journal_report.rs` ni `general_ledger.rs` (`git diff --stat 1e23edb55 epic/47-comptes-miroirs`).

## Le contrat (D9)

```rust
/// Les justificatifs de l'écriture — omis quand il n'y en a aucun (JSON identique
/// à aujourd'hui pour ces écritures). Écran seulement : PDF et CSV inchangés.
#[serde(skip_serializing_if = "Vec::is_empty", default)]
pub documents: Vec<EntryDocument>,

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
string | null; fileName?: string }`, `documents?: EntryDocument[]` sur `JournalEntryRow` et `LedgerLine`.

### Règles de rattachement

- **Facture** (vente et règlement) : `i.status = 'validated' OR (i.status = 'cancelled' AND i.pdf_storage_path IS NOT
  NULL)` — exactement les factures dont la route sert un PDF ; une facture annulée jamais figée n'a pas de justificatif.
- **Avoir** : `cn.status <> 'draft'` (un avoir émis ou annulé a un PDF ; le sens d'un avoir `cancelled` et de son écriture
  se relève au T0, et la règle se corrige au Change Log si le relevé la contredit).
- **Document fournisseur** : `isi.status = 'completed'`, achat ou règlement, **tous** les documents d'un règlement groupé.
- **Filtre de société sur chaque table** de chaque branche (`je.company_id`, `i.company_id`, `cn.company_id`,
  `si.company_id`, `isi.company_id`), défense en profondeur (KF-002).
- **Dédoublonnage** par `(kind, id)` dans chaque écriture, à la construction de la `HashMap` (une même facture ne peut
  pas sortir deux fois d'une écriture aujourd'hui, mais rien au schéma ne l'interdit) ; **ordre stable** : genre
  (`invoice`, `creditNote`, `supplierDocument`), puis identifiant.
- **Écart assumé avec le détail d'une écriture** : un règlement fournisseur groupé montre **tous** ses documents au
  journal, **un seul** au détail de l'écriture (`find_source_document_of_entry`, `LIMIT 1`). Issue `enhancement` à ouvrir
  pour aligner le détail (section *Issues à ouvrir*).

### La requête et son site d'appel

- `kesh-report/src/documents.rs` (neuf), `documents_by_entry(pool, company_id, from, to) -> HashMap<i64,
  Vec<EntryDocument>>` : **une** requête, `SELECT … UNION ALL …` des quatre branches, chacune jointe à `journal_entries` et
  bornée par `je.company_id = ?` et `je.entry_date BETWEEN ? AND ?` (index `idx_journal_entries_company_date`) — aucune
  liste `IN`, aucune requête par écriture ni par compte.
- **Site d'appel** : **un** appel dans `journal_report::generate`, après la requête des lignes ; **un** appel dans
  `general_ledger::generate`, **avant** la boucle `for acc in accounts` (`:825`), la `HashMap` étant lue dans la boucle.
  Les exports et l'outil MCP, qui passent par ces `generate`, paient cette requête sans lire le champ : coût d'**une**
  requête par rapport, assumé (*écarté* : n'appeler la fonction que pour l'affichage, dans `services/rapports.rs` — un
  module de production de plus, `kesh-api`).
- **Borne du grand livre** : la période seule, non les comptes demandés — pour un compte sur plusieurs exercices, la
  requête lit les justificatifs de toutes les écritures de la période (bornés par le nombre de factures, d'avoirs et de
  documents de la société) ; assumé et écrit au Dev Notes.

## Critères d'acceptation

1. **AC-1 — Journal.** `GET /api/v1/reports/journals` (route d'aujourd'hui) : chaque écriture qui a des justificatifs porte
   `documents`, selon les règles ci-dessus ; les autres n'ont pas le champ.
2. **AC-2 — Grand livre.** `GET /api/v1/reports/general-ledger` : chaque `LedgerLine` porte les justificatifs de son
   écriture ; une écriture présente dans plusieurs sections les porte dans chacune. *Option* : § *Où un justificatif a un
   sens*.
3. **AC-3 — Les genres et les cas.** Tests `kesh-report` sur base (`#[sqlx::test]` sur le squash `test-schema/`, comme les
   tests voisins) : facture validée (justificatif) ; facture annulée figée (justificatif) ; facture annulée jamais figée
   (aucun) ; règlement simple d'une facture (la facture) ; **règlement client groupé** de deux factures (deux justificatifs,
   ordre par identifiant) ; avoir émis (justificatif) ; facture fournisseur à document, écriture d'achat et de règlement
   (le document) ; **règlement fournisseur groupé** de deux factures à document (deux) ; facture fournisseur sans document
   (aucun) ; facture fournisseur annulée, écriture d'achat (le document, comme le précédent).
4. **AC-4 — Négatifs qui rougissent.** Chaque filtre est éprouvé par des données **posées en SQL direct** que le schéma
   admet : avoir `draft` portant un `journal_entry_id` (aucun justificatif) ; document importé `discarded` portant un
   `supplier_invoice_id` (aucun) ; facture `draft` portant un `journal_entry_id` (aucune). Le T0 vérifie qu'aucune
   contrainte ne refuse ces lignes ; une ligne refusée par le schéma rend son filtre « défense en profondeur, non
   éprouvable », écrit tel quel au Change Log, et son test est retiré. Le règlement annulé (supprimé) : test de
   **non-régression** (annulation par le dépôt, puis aucun justificatif sur l'écriture d'origine ni sur sa contre-passation),
   nommé comme tel. Chaque filtre a sa **mutation** (filtre retiré → test rouge), journal `kesh-gate-logs/s55-3a/`.
5. **AC-5 — Bornes.** Deux sociétés, chacune avec ses justificatifs : le journal et le grand livre de A ne portent que
   ceux de A ; une écriture hors période n'est pas lue ; grand livre **à plusieurs comptes** : les justificatifs de chaque
   section sont justes. **Une requête** : la ligne d'appel de `documents_by_entry` dans chaque `generate` est citée au
   Change Log et se trouve **hors** de toute boucle (`grep -n "documents_by_entry" crates/kesh-report/src/*.rs` rend une
   définition et deux appels).
6. **AC-6 — Contrat additif.** Une écriture sans justificatif sérialise **exactement** le JSON d'aujourd'hui (test de
   sérialisation, clés comparées) ; les exports PDF et CSV du journal et du grand livre rendent **le même résultat avec
   `documents` rempli et vidé** (même rapport, champ vidé à la main) : CSV, chaînes égales ; PDF, texte extrait par
   `lopdf` (`kesh-report/Cargo.toml`, dépendance de développement) égal. L'outil MCP du grand livre recopie ses champs : sa
   sortie est inchangée **par construction** ; `mcp_contrat` reste vert comme filet, non comme preuve.
7. **AC-7 — Note pour l'écran.** Le contrat TypeScript ci-dessus est recopié dans la fiche 55-3b à l'identique (revu à la
   passe de validation suivante).

## Tâches

- [ ] **T0** — Q1 tranchée par le mainteneur (sinon arrêt) ; base `epic/55-navigation` (PR 46 reprise) ; relever le sens
      d'un avoir `cancelled` et de son écriture ; vérifier les lignes SQL directes de l'AC-4 contre les contraintes.
- [ ] **T1** — Tests rouges d'abord (AC-3 à AC-6) ; journal.
- [ ] **T2** — `documents.rs`, champ `documents` du journal et du grand livre, appels hors boucle.
- [ ] **T3** — Gates.

## Gates

Règle de l'epic (fichier d'epic, § *Gates*) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D
warnings` ; `cargo nextest run -E 'package(kesh-report) | binary(reports_e2e) | binary(mcp_contrat)'` (rayon d'impact) ;
`scripts/gate-ci.sh` en fin d'epic. Aucun fichier de `kesh-db` touché (sinon gate complet, `CLAUDE.md` § *Exception
`kesh-db`*).

## Note de version

API : les écritures du journal et les lignes du grand livre portent un champ `documents` — les justificatifs de
l'écriture (facture, avoir, document de facture fournisseur) —, absent quand il n'y en a aucun ; exports PDF et CSV
inchangés. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-3a-1** — Un champ dans la réponse du rapport plutôt qu'une route groupée (`GET /journal-entries/documents?ids=…`) :
  les justificatifs arrivent avec les écritures qu'ils décrivent, une requête de moins par affichage, aucune liste
  d'identifiants à plafonner.
- **C-55-3a-2** — Le règlement montre la facture réglée (D10), comme `find_source_document_of_entry` pour le fournisseur.
- **C-55-3a-3** — Pas de justificatif libre joint à une écriture manuelle : il n'en existe pas dans Kesh ; hors périmètre.
- **C-55-3a-4** — Borne du grand livre par période seule (voir *La requête*).

### Modules (règle de splitting préventif)

Un : `kesh-report` (et ses tests ; `kesh-api/tests/reports_e2e.rs` pour la réponse HTTP, sans code de production).

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
