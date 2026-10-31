# Story v017-4 : factures fournisseur — brouillon, validée, payée

Status: split

*Fiche rédigée le 2026-10-31, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0175`, branche
`story/v017-4-facture-fournisseur-brouillon` partie de `8e126cfd` (tête de `release/0.17.0`, candidate
`v0.17.0-rc1`) ; la story se fusionne sur la branche de release. Registre : `v017-4-choix-autonomes.md`
(C-v017-4-1 à C-v017-4-20). **Découpée dès la rédaction** en `v017-4a` (backend) et `v017-4b` (écran) — § *Découpage*,
C-v017-4-1 ; cette fiche porte le contexte, les décisions, le contrat et le texte de **tous** les AC, que les
sous-fiches répartissent par numéro. **Complétée le même jour** sur décision du Project Lead (C-v017-4-20) : #853,
#830, #835 et #831, quatre défauts du même formulaire. Clé courte : `v0174`. Issues : **#849** (CR, P2), #853, #830,
#835, #831 — en `refs` seulement, dans les commits **et** dans la PR de la story ; la fermeture se porte sur la **PR de
release**. Les numéros de ligne cités sont ceux de `8e126cfd`.*

## Story

En tant que Guy, qui tient ses livres réels sur la `0.17.0-rc1` et a dû **annuler puis ressaisir** deux fois une facture
fournisseur pour une faute de compte puis de date — une troisième attend (un total erroné) —, je veux qu'une facture
fournisseur **naisse en brouillon**, sans écriture, **se corrige** et **se supprime** librement, **se valide** quand
elle est juste — l'écriture d'achat est alors passée —, et, validée, puisse **revenir au brouillon** tant qu'elle n'est
pas payée ; payée, elle ne change plus. Afin de corriger une faute de saisie sans laisser dans mes livres une facture
annulée et sa contre-passation.

## Origine

- **#849** (2026-10-31, recette de la rc1 sur les livres de Guy) : « création → validation → paiement. Créée, elle est
  en brouillon et peut être éditée. Validée, elle peut être soit payée, soit revenir au brouillon. Payée, on ne peut
  plus la changer. »
- **Ajout du même jour** (mandat) : « on doit pouvoir supprimer des factures annulées sur la version précédente » —
  traité en **question** (Q1), non prescrit : § *Les factures annulées*.
- **Ajout du Project Lead du même jour** (C-v017-4-20) : #853 (échéance avant la date, date du jour par défaut sur une
  pièce importée), #830 (virgule décimale envoyée brute), #835 (total vivant du formulaire à deux décimales), #831
  (liste arrondie à deux décimales) — quatre défauts du formulaire et de la liste que cette story réécrit.
- Contraintes : `kesh-gate-logs/prompts/v017-donnees-a-garder.md` — montée sans perte, sauvegarde antérieure
  importable, rien de jetable sur la base de Guy. Voisines non traitées : #687 (fermée par la v017-3b ; sa fonction
  `purchase_lines_form` s'applique désormais aussi au `PUT`), #825 et #826 (plafonds, taux de TVA : inchangés).

## Ce que le code dit (relevé le 2026-10-31, `8e126cfd`)

### Prémisses de #849 et du mandat

- **« La création passe l'écriture d'achat immédiatement »** : vrai. `create_in_tx`
  (`crates/kesh-db/src/repositories/supplier_invoices.rs:388-651`) passe l'écriture (`:558-576`) puis insère la facture
  en `'open'` (`:578-601`).
- **« Ni `PUT`, ni brouillon, ni dévalidation »** : vrai. Routes des factures fournisseur (`crates/kesh-api/src/lib.rs:617-674`,
  `:1062-1077`) : création, `pay`, `cancel`, `settlement/cancel`, `scan-qr`, `source-document` (lecture, joindre,
  détacher), liste, détail. `CHECK (status IN ('open', 'paid', 'cancelled'))`
  (`migrations/20260628000001_supplier_invoices.sql:65-66`).
- **« Toutes les factures existantes sont passées au journal »** : vrai par construction —
  `purchase_journal_entry_id BIGINT NOT NULL` (`:42`), clé étrangère `RESTRICT` (`:57`).
- **« `min_required` déjà 0.17.0 dans cette release »** : vrai — `20261030000001_bank_imports_manual_source.sql`,
  dernière instruction `UPDATE _kesh_version SET kesh_version_min_required = '0.17.0' WHERE id = 1;` ; les dix crates en
  `0.17.0` (`crates/kesh-api/Cargo.toml:3`). Migrations : **87** (`ls crates/kesh-db/migrations/*.sql | wc -l`).
- **« La règle déjà retenue pour la facture client » (dévalidation)** : **suppression** de l'écriture, non
  contre-passation. `invoices::unvalidate` (`crates/kesh-db/src/repositories/invoices.rs:1501`), doc-comment
  « Dévalide une facture : elle repasse en brouillon, garde son numéro, et son écriture comptable est supprimée (Story
  25-2-b-1, #440) » ; un seul `UPDATE … SET status = 'draft', journal_entry_id = NULL …` puis
  `journal_entries::delete_in_tx(&mut tx, company_id, je_id, user_id, None, false)` ; le trou de numérotation est
  assumé et décrit (`docs/manual/fr/user-manual.tex:740-755`). C-v017-4-8 la suit.

### Prémisses des quatre défauts ajoutés

Établies au registre, C-v017-4-20 : #853, #835 et #831 **confirmées** ; #830 **confirmée**, avec une correction du
remède — la saisie manuelle des relevés ne « normalise » pas une chaîne, elle **parse** une fois (`parseAmount`,
`frontend/src/lib/features/journal-entries/balance.ts:68-71`) et envoie la valeur lue
(`bank-import/manual-statement.ts:115-119`). Constat voisin dans la même fonction, intégré (mécanique) : `sumTtc` lit
une quantité vide comme `0` (`supplier-invoice-form.ts:35`), l'envoi comme `1` (`SupplierInvoiceForm.svelte:233`).

### Ce sur quoi la conception s'appuie

- Les filtres « facture à payer » lisent tous `status = 'open'` : propositions de réconciliation
  (`repositories/reconciliation.rs:825`, `:885`, `:935`), création d'un lot (`repositories/payment_batches.rs:400`),
  règlement (`supplier_invoices.rs:826`), règlement par une transaction (`:1187`), écran des lots
  (`frontend/src/routes/(app)/payment-batches/+page.svelte:53`, `:91`). Un statut `draft` y est exclu sans y toucher
  (C-v017-4-2).
- Clés étrangères vers la facture : `supplier_invoice_lines` `CASCADE` (`20260628000001…:93`), `payment_batch_items`
  `RESTRICT` (`20260628000002_payment_batches.sql:54`), `imported_supplier_invoices` `SET NULL`
  (`20260629000001_imported_supplier_invoices.sql:65`).
- `delete_in_tx` (`journal_entries.rs:1819-1965`) : verrou de l'écriture et de son exercice ; exercice postérieur clos ;
  exercice clos ; écriture contre-passée ; garde de propriété **si** `enforce_ownership` ; verrou de période ;
  lettrage ; audit `journal_entry.deleted` avant le `DELETE`. Le doc-comment dit `invoices::unvalidate` « seul »
  appelant à `false` (`:1808`) — la story en ajoute un second, à nommer.
- Le détachement d'un document : `imported_supplier_invoices::detach_in_tx` (`repositories/imported_supplier_invoices.rs:321`),
  qui rend le document « à traiter » sans toucher la facture.
- Les règles « document » de la complétion vivent dans `complete_import_once`
  (`crates/kesh-api/src/routes/imported_supplier_invoices.rs:157-345`) : devise (`:189-204`), routage IBAN et
  cohérence QRR (`:209-246`), conversion commune (`:247`), forme des lignes (`:254`), `AMOUNT_MISMATCH` (`:262-288`).
- Le contrôle de clôture des brouillons client : K4 `DRAFT_INVOICES` (`repositories/closing_checks.rs:12`, `:474-481`) ;
  le contrat en annonce **sept** (`docs/api-external.md:255`).
- Le seul lecteur MCP : `mcp/outils/factures_fournisseur.rs` (`ecriture_achat_id: f.purchase_journal_entry_id`, `:233`),
  ensemble fermé des statuts `mcp/outils/commun.rs:147-151`.

## Décisions (détail et alternatives au registre)

| # | Décision | Registre |
|---|---|---|
| D1 | Découpage en `v017-4a` (backend) et `v017-4b` (écran) | C-v017-4-1 |
| D2 | `draft` s'ajoute ; `open` reste la valeur stockée de « validée, à payer », libellé « Ouverte » ; aucune ligne réécrite | C-v017-4-2 **(Q4)** |
| D3 | Migration `20261031000001_supplier_invoices_draft` : écriture d'achat nullable, `CHECK` refaits, `min_required` reposé à `0.17.0` (même valeur) ; aucun bump Cargo | C-v017-4-3 |
| D4 | Créer = brouillon sans écriture ; contrôles de saisie à l'enregistrement, contrôles liés à l'écriture à la validation | C-v017-4-4 |
| D5 | `PUT` d'un brouillon, remplacement complet, règles « document » partagées avec la complétion | C-v017-4-5 |
| D6 | `DELETE` d'un brouillon, Administrateur, document rendu « à traiter », historique de lot opposé | C-v017-4-6 |
| D7 | `POST …/validate` `{version}` | C-v017-4-7 |
| D8 | `POST …/unvalidate` `{version}` : suppression de l'écriture d'achat, règle client | C-v017-4-8 |
| D9 | Payée : inchangée ; un brouillon ne s'annule pas | C-v017-4-9 |
| D10 | La complétion d'un document crée un brouillon ; QR en lecture seule | C-v017-4-10 |
| D11 | Brouillons exclus des propositions et des lots par les filtres existants ; inventaire clos | C-v017-4-11 |
| D12 | #853 : échéance antérieure refusée ; pas de date du jour avec un document ; l'avis de date laissé à une issue | C-v017-4-12 |
| D13 | Contrôle de clôture K8 `DRAFT_SUPPLIER_INVOICES` | C-v017-4-13 |
| D14 | Factures annulées masquées par défaut dans la liste ; aucune suppression d'écriture prescrite | C-v017-4-14 **(Q1)** |
| D15 | Écran : badge, page d'édition, actions par statut ; pas de « Enregistrer et valider » | C-v017-4-15 **(Q3)** |
| D16 | Pas de motif de dévalidation calculé d'avance | C-v017-4-16 |
| D17 | Dépôt : quatre fonctions neuves ; `create_validated` composé, pour seeds et tests | C-v017-4-17 |
| D18 | Candidate `v0.17.0-rc2` | C-v017-4-18 **(Q2)** |
| D19 | MCP : brouillons lus, `statut: draft`, `ecritureAchatId: null` | C-v017-4-19 |
| D20 | #853, #830, #835, #831 intégrés | C-v017-4-20 |

### Le cycle

```
             PUT, DELETE (Admin)
                 ┌──┐
   POST ───► draft ──validate──► open ──pay / lot / réconciliation──► paid
                 ▲                 │ │                                 │
                 └───unvalidate────┘ └──────cancel──► cancelled ◄──cancel
                                        ◄──settlement/cancel───────────┘ (paid → open)
```

### Contrat de l'API (changé)

| Route | Rôle, clés | Corps | Réponse | Effet |
|---|---|---|---|---|
| `POST /api/v1/supplier-invoices` | Comptable+, `read-write` | inchangé | `201`, la facture, `status: "draft"`, `purchaseJournalEntryId: null` | **Aucune écriture** (changement de contrat) |
| `POST /api/v1/imported-supplier-invoices/{id}/complete` | inchangé | inchangé | `200`, la facture relue, `status: "draft"` | Aucune écriture ; document `completed` lié |
| `PUT /api/v1/supplier-invoices/{id}` | Comptable+, `read-write` | celui de la création + `version` | `200`, la facture relue | Brouillon seulement |
| `DELETE /api/v1/supplier-invoices/{id}` | **Administrateur**, fermé aux clés | — | `204` | Brouillon seulement ; document rendu « à traiter » |
| `POST /api/v1/supplier-invoices/{id}/validate` | Comptable+, `read-write`, rejouée | `{ "version": n }` | `200`, la facture relue, `status: "open"` | Écriture d'achat passée |
| `POST /api/v1/supplier-invoices/{id}/unvalidate` | Comptable+, `read-write`, rejouée | `{ "version": n }` | `200`, la facture relue, `status: "draft"` | Écriture d'achat **supprimée** |
| `GET /api/v1/supplier-invoices?status=…` | inchangé | — | la liste filtrée | Filtre optionnel, défaut : tout |

`status` (liste) : une ou plusieurs valeurs parmi `draft`, `open`, `paid`, `cancelled`, séparées par des virgules ;
une valeur inconnue → `400 VALIDATION_ERROR`. Les réponses « relues » sont celles de
`SupplierInvoiceResponse::load_with_settlement_cancellation` (`routes/supplier_invoices.rs:209`).

### Codes d'erreur neufs ou étendus

| Code | Statut | Où | `details` |
|---|---|---|---|
| `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE` | `400` | création, complétion, `PUT` | `{ invoiceDate, dueDate }` |
| `SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST` | `409` | `DELETE` d'une facture ouverte | — |
| `SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY` | `409` | `DELETE` d'un brouillon présent dans un lot | `{ paymentBatchId }` |
| `SUPPLIER_INVOICE_PAID` | `409` | `unvalidate` d'une facture payée | — |
| `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` *(existant)* | `409` | `unvalidate` d'une facture d'un lot `generated` | forme existante |
| `MATCHED_BANK_TRANSACTION` *(existant)* | `409` | `unvalidate`, écriture d'achat rapprochée | `{ documentId, documentNumber }` comme côté client |
| `SUPPLIER_INVOICE_DRAFT` | `409` | `cancel` d'un brouillon (motif `cancelBlockedBy` de la fiche) | — |
| `ILLEGAL_STATE_TRANSITION` *(existant)* | `409` | `PUT`, `validate` d'un non-brouillon ; `unvalidate` d'un brouillon ou d'une annulée ; `DELETE` d'une payée ou annulée ; `pay` d'un brouillon | — |
| `OPTIMISTIC_LOCK_CONFLICT` *(existant)* | `409` | `PUT`, `validate`, `unvalidate` | — |

Après les refus propres de `unvalidate`, ceux de `delete_in_tx`, inchangés : `FISCAL_YEAR_CLOSED`,
`LATER_FISCAL_YEAR_CLOSED`, `ENTRY_IS_REVERSED`, `PERIOD_LOCKED`, `ENTRY_LETTERED`. La validation rend les refus que
rendait la création : `FISCAL_YEAR_INVALID`, `ACCOUNT_NOT_POSTABLE`, `INACTIVE_OR_INVALID_ACCOUNTS`,
`CONFIGURATION_REQUIRED`, `DESIGNATED_ACCOUNTS_NOT_POSTABLE`, les refus du projet analytique. Chaque message neuf a sa
clé dans les quatre catalogues et un repli `errors.rs` égal au catalogue fr-CH.

### Précédence des refus de la dévalidation (testée paire par paire)

`NotFound` → statut (`SUPPLIER_INVOICE_PAID` ; `ILLEGAL_STATE_TRANSITION`) → `OPTIMISTIC_LOCK_CONFLICT` →
`SUPPLIER_INVOICE_IN_PAYMENT_BATCH` → `MATCHED_BANK_TRANSACTION` → `FISCAL_YEAR_CLOSED` → `LATER_FISCAL_YEAR_CLOSED` →
`ENTRY_IS_REVERSED` (inatteignable : une facture ouverte n'a pas d'achat contre-passé ; gardé) → `PERIOD_LOCKED` →
`ENTRY_LETTERED`. *(Le statut avant la version : patron de `invoices::unvalidate`, qui juge « non validée » avant le
verrou optimiste ; l'ordre `exercice clos` puis `postérieur` est celui que `delete_in_tx` rend.)*

### Verrous

- **Validation** : la facture `FOR UPDATE` d'abord (comme `pay_in_tx_as` et `cancel_in_tx`), puis exactement l'ordre
  écrit au doc-comment de l'actuel `create_in_tx` (`supplier_invoices.rs:325-387`), étapes (0) à (5). Cycle **connu**
  avec la confirmation d'un lot et l'acceptation d'un lot de rapprochement (sentinelle `companies` partagée puis facture
  suivante), couvert par le rejeu des routes ; écrit au doc-comment.
- **Dévalidation** : la facture `FOR UPDATE` ; lecture des lots `generated` et du rapprochement **en lecture
  verrouillante** (`LOCK IN SHARE MODE`, patron de `lock_for_bank_settlement_in_tx` étape (c)) ; `UPDATE` ; puis
  `delete_in_tx`. Aucune lecture ordinaire avant `delete_in_tx` qui fige un instantané utile à ses gardes : la
  contre-passation de l'achat ne se fait que par `cancel_in_tx`, qui verrouille d'abord la facture.
- **`PUT` et `DELETE`** : la facture `FOR UPDATE` ; pour `DELETE`, `detach_in_tx` **après** (il ne verrouille que le
  document — ordre facture puis document, l'inverse de `attach_in_tx` qui va document puis facture : cycle possible
  entre un rattachement et une suppression du même brouillon, couvert par le rejeu ; écrit).

### Inventaire clos des lecteurs (AC-4, AC-5)

Sites de production à `8e126cfd`, à recompter par `grep -rn 'purchase_journal_entry_id' crates --include=*.rs | grep -v '/tests/'`
et `grep -rnE "status\s*(=|<>|!=|IN)|\.status\s*(==|!=)|status == \"" ` sur les fichiers listés ; chacun soit adapté,
soit déclaré sans effet au Dev Agent Record, avec la raison :

| Site | Lit | Devant un brouillon |
|---|---|---|
| `entities/supplier_invoice.rs:39` | écriture d'achat | `Option<i64>` |
| `routes/supplier_invoices.rs:81`, `:181` | écriture d'achat | `purchaseJournalEntryId: null` |
| `repositories/supplier_invoices.rs` (`FIND_SCOPED_SQL`, `snapshot_json` `:81`, `purchase_payable_line`, `pay_in_tx_as` `:826`/`:842`, `lock_for_bank_settlement_in_tx` `:1187`, `supplier_invoice_cancel_blocker` `:1278-1306`, `cancel_in_tx` `:1355-1363`, `supplier_settlement_cancel_blocker_unlinking` `:1562`, `cancel_settlement_in_tx`) | statut, écriture | refus (`ILLEGAL_STATE_TRANSITION`, `NotOpen`, `SUPPLIER_INVOICE_DRAFT`, `SUPPLIER_INVOICE_NOT_PAID`) ; jamais d'`Invariant` |
| `repositories/letterings.rs:1847`, `:1886-1888` | statut, écriture | jamais appelé sur un brouillon (paiement seulement) ; le type devient `Option`, un brouillon rend `Invariant` explicite |
| `repositories/payment_batches.rs:376-400` | statut, écriture | `SUPPLIER_INVOICE_NOT_OPEN` (inchangé), lu **avant** l'écriture |
| `repositories/reconciliation.rs:825`, `:885`, `:935` | `status = 'open'` | exclu |
| `repositories/contacts.rs:401` | `status <> 'cancelled'` | **compté** (C-v017-4-11) |
| `repositories/journal_entries.rs:1090` | `status = 'cancelled'` | sans effet |
| `repositories/journal_entries.rs:2259-2260` | `purchase_journal_entry_id IN (…)` | `NULL` ne correspond à rien : sans effet |
| `repositories/journal_entries.rs:2773` | rôle de l'autorité d'achat | sans effet (pas d'écriture) |
| `repositories/imported_supplier_invoices.rs:395` | écriture d'achat ou de règlement | sans effet |
| `routes/reconciliation.rs:3410` | écriture d'achat (audit) | jamais atteint sur un brouillon (refus avant) |
| `exports/csv_tables.rs:1242` | écriture d'achat | cellule vide (`fmt_opt_i64`) |
| `mcp/outils/factures_fournisseur.rs:233`, `mcp/outils/commun.rs:147-151` | écriture, statut | `null`, `draft` (C-v017-4-19) |
| `kesh-seed/src/pieges.rs:585`, `:689` | création, écriture | `create_validated` (AC-12) |
| `kesh-reconciliation/src/matching.rs:799` | champ de test | `Some(999)` |
| `repositories/closing_checks.rs` | — | K8 neuf (AC-16) |
| `frontend/…/payment-batches/+page.svelte:53`, `:91` | `status === 'open'` | exclu |
| `frontend/…/supplier-invoices/[id]/+page.svelte:514`, `:642` | statut | AC-22 |

## Les factures annulées (Q1)

Le mandat demande de pouvoir « supprimer des factures annulées sur la version précédente ». Ce que le code établit :

- une facture annulée porte son **écriture d'achat** et sa **contre-passation** datée du jour de l'annulation, liée
  (`reverses_entry_id`), lettrées ensemble (`cancel_in_tx`, `supplier_invoices.rs:1335-1490`) ; nettes à zéro, elles
  restent deux écritures numérotées du journal ;
- si elle était payée, son règlement **détaché** reste au grand livre, gelé contre la modification par la trace d'audit
  de l'annulation (`modification_guard`, `journal_entries.rs:1066-1100`) — supprimer la facture le dégèlerait ;
- `delete_in_tx` refuse de supprimer une écriture contre-passée, motif écrit au code : « la supprimer effacerait la
  correction, ce que l'art. 958f CO interdit précisément » (étape 3-bis) ; la garde de modification refuse de supprimer
  une contre-passation.

**Options** :

- **(A) Suppression réelle** de la facture et de ses deux écritures (et de leur lettrage), réservée à une facture
  **jamais payée**, dont les deux écritures sont dans un exercice **ouvert**, hors période verrouillée, tracée au journal
  d'audit (instantanés des deux écritures). Conséquences : trois gardes à contourner par un chemin dédié ; **deux trous**
  de numérotation ; la piste de contrôle ne garde la correction que dans le journal d'audit, non plus dans les livres
  (CO 957a et 958f, Olico) ; une restauration antérieure les ferait réapparaître.
- **(B) Masquer** les factures annulées dans la liste, sans toucher au journal (livrée par cette story, AC-15 et
  AC-21) ; le grand livre et le journal gardent les deux écritures, nettes à zéro.
- **(C) Supprimer la facture seule**, écritures gardées : perd le lien pièce ↔ écritures et dégèle un règlement détaché.

**Recommandation** : **(B)** dans cette story, et **ne pas** faire (A). Les factures annulées de la rc1 sont des
corrections **régulières** : une écriture et sa contre-passation, nettes, sont exactement ce que l'art. 958f demande
d'une correction ; elles ne faussent ni solde ni rapport. Le brouillon livré ici supprime la cause pour l'avenir. Si
Guy veut néanmoins (A), une story dédiée, avec ses conditions écrites ci-dessus et sa propre validation, plutôt
qu'un ajout à celle-ci. ⛔ Rien de (A) n'est prescrit sans son accord.

## Découpage

*C-v017-4-1 : plus de cinq modules ; aucun cycle Cargo ; le backend se teste seul en HTTP.*

- **`v017-4a-facture-fournisseur-brouillon-backend.md`** — `kesh-db` (migration, dépôt, contrôle de clôture),
  `kesh-api` (routes, erreurs, audit, MCP, export, registres), `kesh-i18n` (messages d'erreur et libellés d'audit),
  `kesh-seed`, `docs/api-external.md`, tests Rust, textes de la migration (CHANGELOG en-tête et *Modifié*, manuel
  d'administration et son PDF).
- **`v017-4b-facture-fournisseur-brouillon-ecran.md`** — le frontend (liste, fiche, page d'édition, formulaire et ses
  quatre défauts, écran de clôture), les clés de l'écran, le manuel utilisateur et son PDF, l'E2E, les captures, le
  CHANGELOG *Ajouté* et *Corrigé*, le README. **Dépend de la 4a.**

| AC | Sous-fiche |
|---|---|
| AC-1 à AC-20 | `v017-4a` |
| AC-21 à AC-32 | `v017-4b` |

| Mutations | Sous-fiche |
|---|---|
| M1 à M20 | `v017-4a` |
| M21 à M30 | `v017-4b` |

## Critères d'acceptation

*Chaque AC se prouve par un test **écrit rouge d'abord** (commit des tests rouges avant le code, nommés au Dev Agent
Record avec leur sortie rouge), sauf mention. Tests Rust de dépôt et d'API : squash (`#[sqlx::test(migrations =
"../kesh-db/test-schema")]`), sauf ceux de la migration, sur le vrai `MIGRATOR`.*

### Sous-fiche 4a — backend

- **AC-1 — La migration.** `crates/kesh-db/migrations/20261031000001_supplier_invoices_draft.sql` : un en-tête qui dit
  pourquoi (C-v017-4-2, C-v017-4-3) et que la migration est **breaking** au sens de P1 ; puis
  1. `ALTER TABLE supplier_invoices MODIFY COLUMN purchase_journal_entry_id BIGINT NULL;` — même type ; la clé
     étrangère `fk_supplier_invoices_purchase_je` est gardée. Si MariaDB refuse la modification d'une colonne portée par
     une clé étrangère, la retirer et la reposer **identique** dans la même migration ; le choix et l'erreur constatée
     au Dev Agent Record ;
  2. `chk_supplier_invoices_status` refait : `CHECK (status IN ('draft', 'open', 'paid', 'cancelled'))` ;
  3. `chk_supplier_invoices_entry_iff_not_draft` : `CHECK ((status = 'draft') = (purchase_journal_entry_id IS NULL))` ;
  4. `chk_supplier_invoices_draft_without_settlement` : `CHECK (status <> 'draft' OR (settlement_type IS NULL AND
     settlement_bank_account_id IS NULL AND settlement_account_id IS NULL AND settlement_journal_entry_id IS NULL AND
     paid_at IS NULL))` ;
  5. en **dernière** instruction, `UPDATE _kesh_version SET kesh_version_min_required = '0.17.0' WHERE id = 1;` (P2,
     P3 : même valeur ; aucun bump Cargo, P2-bis déjà tenu — `grep -n '^version' crates/*/Cargo.toml` rend dix fois
     `0.17.0`).
  Aucune autre instruction ; aucune écriture d'une table applicative. **P5** : ligne à
  `docs/migrations-idempotence-audit.md` (verdict, justification), les **deux** sites du total passent de 87 à **88**
  (`## Table d'audit (N migrations)`, ligne 17 ; ligne `Total`, ligne 113, avec « + 1 Story v017-4a »), les trois
  compteurs de partition recomptés depuis le tableau ; contrôle `ls crates/kesh-db/migrations/*.sql | wc -l` = `grep -c
  '^| \`20' docs/migrations-idempotence-audit.md` = 88. **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to"
  crates/`, chaque site inspecté, conclusion au Dev Agent Record ; `migrations_upgrade_path.rs:123-127` (`total, 87` →
  **88**, ligne d'historique « + supplier_invoices_draft (Story v017-4a, refs #849) = 88 » sous `:122`) et `:215`, `:218`
  (`total - 53` → **`total - 54`**, ligne d'historique « Story v017-4a : 53 → 54, frontière inchangée (88 - 54 = 34) »
  sous `:208`) ; toutes les copies par `grep -nE '\b(53|54|87|88)\b|total - 5[34]'
  crates/kesh-db/tests/migrations_upgrade_path.rs` (à `8e126cfd` : `:44`, `:46`, `:63-64`, `:88` historique à laisser,
  `:122`, `:125-126`, `:129`, `:137`, `:144`, `:147`, `:208`, `:215`, `:218`, `:287`), chacune triée au Dev Agent
  Record. **P7** : entrée `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`, après `20261030000001`),
  `ExemptionBasis::Durable`, justification « UPDATE sur _kesh_version (bump kesh_version_min_required, valeur
  inchangée 0.17.0), table système jamais restaurée. Les autres instructions sont du DDL (ALTER TABLE). » ; compteur
  `post_restore.rs:1227-1228` 22 → **23**, ligne d'historique « 22 → 23 (Story v017-4a) » au-dessus. **P8** : la
  migration s'inscrit à `crates/kesh-db/migrations.sha384` (`<version> <sha384sum>`) ; squash régénéré par
  `scripts/regen-test-schema.sh` ; `test_schema_guard` vert. `migrations_fresh_install.rs:248` (`"0.17.0"`) reste juste —
  vérifié, non modifié. Tests : un `INSERT` direct d'un brouillon **avec** écriture, d'une facture `open` **sans**
  écriture, d'un brouillon portant `paid_at` — chacun refusé par la base (erreur de contrainte nommée).
- **AC-2 — Montée sans perte, depuis la 0.16.x et depuis la rc1.** Deux tests `#[sqlx::test(migrations = false)]` de
  `crates/kesh-db/tests/migrations_upgrade_path.rs`, sur le patron de `bank_imports_manual_source_preserves_existing_rows`,
  qui montent le schéma jusqu'à la migration exclue par **résolution par version** (P6) — (a)
  `common::migrations_before(20261030000001, …)` (état 0.16.x) et (b) `migrations_before(20261031000001, …)` (état
  rc1) —, puis insèrent dans une société de test (fixtures **synthétiques**) : un exercice ouvert, un fournisseur, des
  comptes, et des factures fournisseur **ouverte**, **payée** (écriture de règlement, lettrage de pièce), **annulée**
  (contre-passation), **payée puis annulée** (règlement détaché, trace d'audit `supplier_invoice.cancelled`), une avec
  un **document** `completed` lié, une présente dans un **lot confirmé** ; puis appliquent le reste et vérifient :
  1. chaque ligne de `supplier_invoices`, `supplier_invoice_lines`, `imported_supplier_invoices`,
     `payment_batch_items`, `journal_entries`, `journal_entry_lines` et des tables de lettrage touchées se relit
     **colonne par colonne à l'identique** (`version` et `updated_at` compris) ;
  2. les mêmes factures se relisent **par le code** : `supplier_invoices::get`, `list`,
     `get_settlement_view` réussissent et rendent les statuts d'origine (`open`, `paid`, `cancelled`) ;
  3. **le cas de Guy** : la facture ouverte migrée se **dévalide** (écriture d'achat supprimée, `draft`), se **modifie**
     (`update_draft`, un total corrigé), se **revalide** (écriture neuve, `open`) ;
  4. `_kesh_version.kesh_version_min_required` = `0.17.0`.
- **AC-3 — Sauvegardes.** (a) Le test de la v016-2a qui importe dans la 0.17.0 une sauvegarde au manifeste de 0.16.x
  (`crates/kesh-api/tests/admin_full_import_e2e.rs`, à nommer au Dev Agent Record) porte en plus des factures
  fournisseur **ouverte**, **payée** et **annulée** : `200`, chacune se relit à l'identique par
  `GET /supplier-invoices/{id}`, et l'ouverte se dévalide puis se revalide. (b) Aller-retour dans la 0.17.0 d'une base
  qui porte un **brouillon** (écriture `null`) et une facture validée : export, import, chaque facture identique, le
  brouillon reste brouillon sans écriture. (c) L'inventaire des tables ne change pas (aucune table neuve) : le test
  existant de la v016-2a qui le compare passe sans modification. (d) **Limite L1, non testable** (binaire rc1) : une
  sauvegarde de la rc2 portant un brouillon ne s'importe pas dans la rc1 — écrite (AC-20).
- **AC-4 — L'écriture d'achat devient facultative, partout.** `SupplierInvoice.purchase_journal_entry_id:
  Option<i64>` ; chaque site de l'inventaire « écriture d'achat » (§ *Inventaire clos*) adapté ou déclaré sans effet au
  Dev Agent Record ; `SupplierInvoiceResponse.purchase_journal_entry_id: Option<i64>` (`null` pour un brouillon) ; export
  CSV : cellule vide ; aucun `unwrap()`/`expect()` neuf sur cette valeur en production — un état incohérent rend
  `DbError::Invariant`. Test : `GET` d'un brouillon rend `"purchaseJournalEntryId": null` (champ **présent**, vérifié
  par `get()` et non `is_null()` seul) ; l'export global d'une base à brouillon s'écrit.
- **AC-5 — Un brouillon n'est jamais payable ni proposé.** Tests, un par chemin, chacun sur un brouillon au montant
  exact d'un débit : (1) `GET` des propositions de réconciliation d'un débit (`reconciliation.rs:825`, `:885`) — absent ;
  (2) proposition par règle d'affectation (`:935`) — absente ; (3) `POST /reconciliation/accept` d'une proposition
  fournisseur visant un brouillon — `200`, `failed[]` `SUPPLIER_INVOICE_NOT_OPEN` (§ *Pattern batch*) ; (4)
  `POST /payment-batches` incluant un brouillon — refus par facture `SUPPLIER_INVOICE_NOT_OPEN` ; (5) `POST …/pay` —
  `409 ILLEGAL_STATE_TRANSITION` ; (6) `POST …/settlement/cancel` — `SUPPLIER_INVOICE_NOT_PAID` (inchangé) ;
  (7) `GET` de la fiche d'un brouillon : `cancellable: false`, `cancelBlockedBy: "SUPPLIER_INVOICE_DRAFT"` ; (8) les
  propositions de fournisseurs par IBAN comptent le brouillon (C-v017-4-11). Rien n'écrit au journal sur ces refus.
- **AC-6 — Créer un brouillon.** `POST /supplier-invoices` → `201`, `status: "draft"`, `purchaseJournalEntryId: null`,
  `version: 1` ; **aucune** ligne neuve dans `journal_entries` (compte avant/après) ; contrôles à l'enregistrement de
  C-v017-4-4, dans l'ordre actuel de `create_in_tx` pour ceux qui y sont (lignes non vides, projet, fournisseur, forme,
  comptes de charge), plus l'échéance (AC-14) dans la conversion commune ; un brouillon daté hors de tout exercice est
  **accepté** ; un compte de charge d'une autre société, inactif, d'un autre type ou non imputable est **refusé** avec
  les codes actuels. Audit `supplier_invoice.created`, instantané sans `journalEntryId`. Route classée au registre
  `audit_route_registry.rs` selon ce qu'elle écrit désormais (plus d'écriture au journal).
- **AC-7 — Compléter un document crée un brouillon.** `POST /imported-supplier-invoices/{id}/complete` → la facture
  relue, `status: "draft"`, aucune écriture ; document `completed`, lié ; audit `imported_supplier_invoice.completed`
  inchangé. Les règles « document » (devise, routage IBAN et QRR, coordonnées prises du QR, `QR_FIELDS_FROM_DOCUMENT`,
  forme des lignes, `AMOUNT_MISMATCH`) sont **extraites** en une seule fonction de `kesh-api`, appelée par la complétion
  et par le `PUT` (AC-8) ; l'ordre des refus de la complétion est inchangé (les tests existants de
  `inbox_import_e2e.rs` sur ces refus passent sans changement d'assertion de code).
- **AC-8 — Modifier un brouillon (`PUT`).** Corps : celui de la création + `version`. Remplace l'en-tête et **toutes**
  les lignes (positions renumérotées depuis 0) ; `version + 1` ; mêmes contrôles qu'à la création ; si le brouillon a un
  document joint, la fonction de l'AC-7 s'applique (un document avec QR : coordonnées du corps refusées
  `QR_FIELDS_FROM_DOCUMENT`, coordonnées gardées celles du document, `AMOUNT_MISMATCH` si le TTC diffère du montant du
  QR). Refus : non brouillon → `409 ILLEGAL_STATE_TRANSITION` ; version périmée → `409 OPTIMISTIC_LOCK_CONFLICT` ;
  facture d'une autre société → `404`. Audit `supplier_invoice.updated`, instantanés avant et après. Aucune écriture au
  journal. Tests : chaque refus ; un `PUT` qui change fournisseur, dates, lignes, comptes, projet, coordonnées, puis
  relecture complète ; un `PUT` avec trois lignes puis une seule (lignes orphelines absentes).
- **AC-9 — Supprimer un brouillon (`DELETE`).** Administrateur seulement (un Comptable → `403`, une clé → `403
  API_KEY_ADMIN_FORBIDDEN`, montée avec les routes Administrateur comme `DELETE /invoices/{id}`). `204` ; facture et
  lignes supprimées ; document joint repassé `to_complete`, lien `NULL`, audit `imported_supplier_invoice.detached` puis
  `supplier_invoice.deleted` (instantané) dans la même transaction. Refus (C-v017-4-6) : ouverte →
  `409 SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST` ; payée, annulée → `409 ILLEGAL_STATE_TRANSITION` ; présente dans un
  lot (tout statut de lot) → `409 SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY` — jamais une erreur de clé étrangère.
- **AC-10 — Valider.** `POST …/validate` `{version}` → `200`, `status: "open"`, `purchaseJournalEntryId` posé,
  `version + 1`. L'écriture d'achat est **la même** qu'aurait passée la création de la 0.17.0-rc1 pour les mêmes
  données : test d'égalité ligne par ligne (comptes, débits, crédits, projet, journal `Achats`, date = date de facture,
  libellé) entre `validate_in_tx` sur un brouillon et `create_validated_in_tx` — et contre une écriture attendue écrite
  en dur. Refus : ceux de la création actuelle (exercice ouvert absent → `FISCAL_YEAR_INVALID` ; compte non imputable,
  inactif ; réglages ; comptes désignés ; projet archivé depuis l'enregistrement), plus `ILLEGAL_STATE_TRANSITION`
  (non brouillon) et `OPTIMISTIC_LOCK_CONFLICT`. Un refus n'écrit rien (facture toujours `draft`, aucune écriture). Audit
  `supplier_invoice.validated` (`journalEntryId`). Rejouée sur interblocage ; ordre des verrous au doc-comment (§
  *Verrous*).
- **AC-11 — Dévalider.** `POST …/unvalidate` `{version}` → `200`, `status: "draft"`, `purchaseJournalEntryId: null`,
  `version + 1` ; l'écriture d'achat et ses lignes n'existent plus ; **aucune** contre-passation n'est passée (compte des
  écritures avant/après : −1) ; audits `journal_entry.deleted` et `supplier_invoice.unvalidated` (instantané, identifiant
  de l'écriture supprimée). Empêchements et précédence du § *Précédence*, chacun testé, paire par paire pour les
  voisins ; en particulier : payée → `SUPPLIER_INVOICE_PAID` ; lot `generated` → `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` ;
  écriture d'achat rapprochée (montée par `UPDATE bank_transactions SET matched_entry_id`) → `MATCHED_BANK_TRANSACTION`,
  et la transaction garde son lien ; exercice clos ; exercice postérieur clos ; période verrouillée. Le doc-comment de
  `delete_in_tx` nomme le second appelant du drapeau `false`. Une facture payée dont on annule le règlement puis qu'on
  dévalide : test de bout en bout (dépôt).
- **AC-12 — `create_validated` pour les seeds et les tests.** `create`/`create_in_tx` renommés
  `create_validated`/`create_validated_in_tx`, composés de `create_draft_in_tx` puis `validate_in_tx` dans une même
  transaction (aucun corps dupliqué) ; aucune route ne l'appelle (`grep -rn "create_validated" crates/kesh-api/src`
  ne rend rien). Appelants mis à jour : `kesh-seed/src/pieges.rs:585`, les tests (`grep -rln
  'supplier_invoices::create\b'` à `8e126cfd`, dont `supplier_invoices_repository.rs` — 45 appels), `kesh-db/src/retry.rs`
  si c'en est un. Les tests d'API qui créent par HTTP une facture qu'ils paient ensuite (`grep -rln
  '"/api/v1/supplier-invoices"' crates/kesh-api/tests` : `inbox_import_e2e.rs`, `mcp_enveloppe_e2e.rs`,
  `mcp_transport_e2e.rs`, `opening_balances_e2e.rs`, `rejeu_interblocage_e2e.rs`, `services_rollout.rs`,
  `supplier_settlement_cancel_e2e.rs`) appellent `…/validate` après la création ; leur liste triée au Dev Agent Record.
  `rejeu_interblocage_e2e.rs` : le rejeu de la **validation** remplace celui de la création.
- **AC-13 — Un brouillon ne s'annule pas.** Variante `SettlementCancelBlocker::SupplierInvoiceDraft`, code
  `SUPPLIER_INVOICE_DRAFT`, rang 1 de `supplier_invoice_cancel_blocker` avant `SupplierInvoiceCancelled` ; `POST
  …/cancel` d'un brouillon → `409`, rien n'est écrit ; message aux quatre catalogues.
- **AC-14 — #853, au serveur.** `to_new_supplier_invoice` refuse `dueDate < invoiceDate` :
  `400 SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`, `details { invoiceDate, dueDate }` au format `AAAA-MM-JJ`, message qui
  dit les deux dates ; une échéance égale ou absente passe. Testé sur la création, la complétion (document sans QR et
  avec QR) et le `PUT`. Le refus passe **après** l'échelle des lignes et **avant** les coordonnées de paiement (ordre
  écrit au doc-comment). Une facture existante dont l'échéance précède la date reste lisible et payable (test : une
  ligne posée en SQL se relit et se paie).
- **AC-15 — Filtre de statut de la liste.** `GET /supplier-invoices?status=draft,open,paid` ne rend que ces statuts ;
  `total` compte le filtre ; sans paramètre, tout (inchangé) ; valeur inconnue ou vide → `400 VALIDATION_ERROR`. Le
  service `factures_fournisseur::lister` reçoit le filtre en option ; l'outil MCP de liste n'en change pas.
- **AC-16 — K8 `DRAFT_SUPPLIER_INVOICES`.** Dans `closing_checks.rs` : sévérité `info`, compte des brouillons
  fournisseur de la société **datés dans l'exercice**, liste de 20 au plus `{id, label, date}` par date puis `id`
  (`label` : « <numéro> — <fournisseur> », ou le seul fournisseur sans numéro), rendu **après** K4. Doc du module et
  tableau d'en-tête mis à jour ; `closing_checks_e2e.rs` : un brouillon dans Y → `attention`, `count` exact ; dans Y+1 →
  ignoré ; une facture ouverte → ignorée ; le total des contrôles rendus passe de 7 à **8** (assertion figée mise à jour).
- **AC-17 — MCP.** `STATUTS_FACTURE_FOURNISSEUR` gagne `("draft", StatutFactureFournisseur::Draft)` ;
  `ecriture_achat_id: Option<i64>`, `null` pour un brouillon ; les tests MCP qui figent ces sorties (`mcp_outils_e2e.rs`)
  couvrent un brouillon ; le contrat MCP (K23 et voisins) reste vert.
- **AC-18 — Audit, registres, messages.** Quatre actions neuves — `supplier_invoice.updated`, `.validated`,
  `.unvalidated`, `.deleted` — à `audit_labels.rs` (`:194-197`), au registre `audit_label_registry.rs`, et leurs libellés
  `audit-log-action-supplier-invoice-*` aux **quatre** catalogues (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}`) ;
  les quatre routes neuves au registre `audit_route_registry.rs` (`:425-455`), classées (validation, dévalidation :
  `Traced, Rejouee` ; `PUT`, `DELETE` : `Traced, SansEcritureAuJournal`), et la création reclassée ; le compteur des
  routes tracées de `lib.rs` suit. Chaque code d'erreur neuf : variante `DbError` ou `AppError`, réponse dans
  `errors.rs`, clé aux quatre catalogues, repli égal au fr-CH ; `textes_coherents.rs` et `i18n_e2e.rs` verts, leurs
  décomptes figés recomptés.
- **AC-19 — `docs/api-external.md`.** Une section neuve « Le cycle d'une facture fournisseur » (tableau du contrat, codes,
  précédence de la dévalidation, `details`) ; ⚠️ *depuis la v0.17.0*, `POST /supplier-invoices` et la complétion
  créent un **brouillon** sans écriture — **changement de contrat**, avec la conduite d'une intégration (valider après
  créer) ; `purchaseJournalEntryId` nullable ; § *Documents à traiter* (`:788-792`) ; contrôles de clôture : « sept »
  → « huit » et K8 (`:255-256`) ; tableau des codes d'erreur (`:907` et suivantes) ; § *Interblocages* (`:953`) ;
  filtre `status` de la liste ; `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`.
- **AC-20 — Textes de la migration et de la version.** CHANGELOG `[0.17.0] — Non publié` : paragraphe d'en-tête —
  « six stories » → sept, « **une migration** » → **deux** (nommer `20261031000001_supplier_invoices_draft`, breaking,
  `min_required` inchangé à 0.17.0), la limite L1 rc2 → rc1 ; section *Modifié* : le changement de contrat de
  `POST /supplier-invoices` et de la complétion, `purchaseJournalEntryId` nullable, contrôles de clôture à huit. Manuel
  d'administration, § *Passer à la 0.17.0* (`docs/manual/fr/admin-manual.tex:2486`) : deux migrations, aucune ligne
  existante modifiée, la limite L1 ; et là où il décrit les sorties MCP des factures fournisseur, `draft` et
  `ecritureAchatId` nul (`grep -n 'kesh_facture_fournisseur\|ecritureAchat' docs/manual/fr/admin-manual.tex`). PDF
  régénéré au commit de son `.tex`, contrôlé aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '`). Macros de version
  **inchangées** (release).

### Sous-fiche 4b — écran

- **AC-21 — Liste.** `SupplierInvoiceStatus` gagne `'draft'` ; `supplierInvoiceStatusLabel('draft')` → clé
  `supplier-invoices-status-draft` (« Brouillon », glossaire partie A, `docs/i18n-glossaire.md:92`) ; badge dans la liste.
  **#831** : la colonne TTC affiche `formatExactAmount(inv.totalAmount)` (`invoice-helpers.ts:92`) au lieu de
  `formatSupplierInvoiceTotal` (`supplier-invoices/+page.svelte:93`) — `100.0040` s'affiche `100.0040`, `100.00`
  s'affiche `100.00`. **C-v017-4-14** : case « Afficher les factures annulées », décochée par défaut ; la liste appelle
  `?status=draft,open,paid`, puis sans filtre une fois cochée. **Banc de la page de liste** neuf
  (`frontend/src/routes/(app)/supplier-invoices/supplier-invoices-list-page.test.ts`) : rendu d'un brouillon (badge),
  d'un total sous le centime, appel filtré par défaut, appel non filtré après la case.
- **AC-22 — Fiche, actions par statut.** Brouillon : *Modifier* (lien vers l'édition), *Valider* (confirmation : « L'écriture
  d'achat sera passée au journal »), *Supprimer le brouillon* (Administrateur seul, confirmation) ; **ni** bloc *Payer*
  (`[id]/+page.svelte:514`) **ni** bloc *Annuler la facture* (`:642`) ; les gestes du document (joindre, détacher)
  restent. Ouverte : *Dévalider* (Administrateur et Comptable, confirmation : l'écriture d'achat sera **supprimée**, la
  numérotation gardera un trou, la facture redeviendra modifiable), plus *Payer* et *Annuler* comme aujourd'hui. Payée,
  annulée : inchangées. Après chaque geste, la fiche affiche la facture rendue par le serveur et une notification de
  succès. Banc de la fiche : chaque statut × chaque rôle (Administrateur, Comptable, Consultation) — boutons présents ou
  absents.
- **AC-23 — Page d'édition.** `/supplier-invoices/[id]/edit` : charge la facture ; si elle n'est pas brouillon, redirige
  vers la fiche avec un message. Rend `SupplierInvoiceForm` **pré-rempli** (fournisseur, numéro, dates, coordonnées,
  projet, lignes) — nouvelle propriété `initial` ; `onSubmit` appelle `PUT` avec la `version` lue ; avec un document
  joint, le même mode que la complétion (QR en lecture seule, total comparé au QR). Succès → fiche ; refus → saisie
  intacte, message à sa place (patron AC16 de la 30-2b) ; `OPTIMISTIC_LOCK_CONFLICT` → message qui invite à recharger.
- **AC-24 — Le formulaire.** (a) bouton « Enregistrer le brouillon » à la création, « Enregistrer » à l'édition ;
  (b) **#853** : avec un `document`, `fInvoiceDate` démarre **vide**, le champ est obligatoire et une aide dit « Date de
  la facture, telle qu'imprimée sur le document » ; sans document, la date du jour reste proposée ; une échéance
  antérieure à la date est signalée **sous le champ** et bloque l'envoi (message qui dit les deux dates) ; le refus
  serveur `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE` s'affiche au même endroit (`describeSubmitError`) ; (c) **#830** :
  quantité, prix et taux sont **lus une fois** par `parseAmount` et envoyés avec un point décimal (« 12,50 » → `"12.50"`) ;
  `sumTtc`, `amountMismatch` et `lineNotInCentimes` lisent les mêmes valeurs ; une quantité vide vaut `1` partout (le
  total vivant comme l'envoi) ; le commentaire « ⚠️ La virgule n'est pas normalisée à l'envoi : #830 »
  (`supplier-invoice-form.ts`) est retiré ; (d) **#835** : le total vivant affiche `formatExactAmount`
  (`SupplierInvoiceForm.svelte:591`), si bien qu'une ligne à 99.9999 se lit `99.9999` sous son message de refus, et le
  montant du QR (`fmt`, `:189-196`) de même. Bancs : `SupplierInvoiceForm.test.ts`, `supplier-invoice-form.test.ts`.
- **AC-25 — Messages des refus.** Une fonction pure `supplierInvoiceUnvalidateMessage(code, details)` (patron
  `supplierInvoiceCancelMessage`, `invoice-cancel.ts`) rend un message par code de la dévalidation (§ *Précédence*) et un
  repli générique ; `SUPPLIER_INVOICE_DRAFT` ajouté à `supplierInvoiceCancelMessage` ; refus de la suppression et de la
  validation affichés sous leur bouton. Tests unitaires : chaque code.
- **AC-26 — Écran de clôture.** `closing-checks-labels.ts`, `FiscalYearChecks.svelte` et leurs types : K8, libellé,
  liste avec liens vers les fiches ; fixtures et bancs (`FiscalYearChecks.test.ts`, `closing-checks.test-fixtures.ts`)
  à huit contrôles.
- **AC-27 — Lots.** L'écran des lots ne propose jamais un brouillon (`payment-batches/+page.svelte:53`, `:91`) : banc qui
  le prouve.
- **AC-28 — i18n.** Toutes les clés neuves de l'écran aux quatre catalogues ; `npm run lint-i18n-ownership` vert ; les
  gardes i18n (moissonneur, allowlist, décomptes figés) recomptées. Libellés : glossaire (brouillon, valider, dévalider,
  ouverte) ; tout terme neuf ajouté au glossaire ou justifié.
- **AC-29 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, et son PDF régénéré au même commit, contrôlé
  aplati). Sites, à compléter par `grep -n -i 'enregistr\|ouverte\|annuler-facture-fournisseur\|ressaisi' ` sur la §
  `sec:factures-fournisseurs` (`:1565`) et les § qui la citent : (1) § *Saisir une facture fournisseur* (`:1573-1641`) —
  l'enregistrement crée un **brouillon**, sans écriture ; (2) une § neuve « Le brouillon : modifier, valider,
  supprimer » et « Dévalider une facture ouverte » (empêchements et levée de chacun ; trou de numérotation ; payée : annuler
  le règlement d'abord) ; (3) § *Une facture fournisseur au total sous le centime* (`:1714-1759`) : « ouverte :
  annulez-la puis ressaisissez » devient « dévalidez-la, corrigez la ligne, revalidez » ; payée : annuler le règlement,
  dévalider ; (4) § *Annuler une facture fournisseur* (`:1761-1795`) : quand annuler (la facture n'était pas due) et
  quand dévalider (une faute de saisie) ; « Une facture annulée ne se désannule pas : saisissez-la de nouveau » relu ;
  les factures annulées masquées dans la liste ; (5) § *Numérotation* (`:740-755`) : « Deux chemins en creusent » →
  trois, la dévalidation d'une facture fournisseur ; (6) réconciliation d'un débit (`:2480`) et lots
  (`sec:paiements-fournisseurs`) : seules les factures **validées** (ouvertes) ; (7) documents à traiter
  (`sec:import-factures`) : la complétion crée un brouillon ; date de facture à saisir ; (8) contrôles de clôture : K8 ;
  (9) #853 : l'échéance ne peut précéder la date ; (10) la virgule décimale admise. Tout passage qui affirme que
  l'enregistrement **passe l'écriture** est réécrit (`:1583-1584` ; la phrase est coupée en fin de ligne : greper « Kesh passe l'écriture d'achat »).
- **AC-30 — E2E.** Une spec neuve `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts` : fournisseur et comptes
  synthétiques ; **brouillon** par le formulaire avec un compte de charge faux → fiche « Brouillon », aucune écriture
  (lien d'écriture absent) ; *Modifier* → bon compte → *Enregistrer* ; *Valider* → « Ouverte », lien vers l'écriture ;
  *Dévalider* → « Brouillon » ; *Modifier* le prix (synthétique, 100.20 → 100.00) ; *Valider* ; un relevé **saisi à la
  main** par l'API (`POST /bank-imports/manual`) porte un débit égal ; à la réconciliation, la facture est proposée pour
  ce débit, et *Accepter* la rend « Payée ». Et : un brouillon au montant d'un débit **n'est pas** proposé ; un brouillon
  se supprime (Administrateur). Specs existantes adaptées pour valider après créer : `supplier-invoices.spec.ts:108`,
  `payment-batches.spec.ts:102`, `supplier-documents.spec.ts:491`, `helpers/api-fixtures.ts:289` et toute spec qui
  enregistre par l'interface puis paie (liste au Dev Agent Record). La suite complète tourne au dernier commit de code
  de la 4b (règle « Test Locally First »), échecs jugés fichier par fichier contre `docs/testing.md`.
- **AC-31 — Captures** (thème clair, base de démonstration, hors dépôt, `kesh-gate-logs/v0174/captures/`) : liste avec
  un brouillon et la case des annulées (cochée, décochée) ; fiche d'un brouillon (Administrateur, Comptable) ; page
  d'édition, avec et sans document à QR ; confirmations *Valider*, *Dévalider*, *Supprimer* ; fiche ouverte avec
  *Dévalider* ; un refus de dévalidation affiché ; le formulaire avec l'échéance refusée, la date vide sur un document, le
  total vivant à quatre décimales ; la liste avec un total sous le centime ; l'écran de clôture avec K8.
- **AC-32 — CHANGELOG et README.** `[0.17.0] — Non publié` : *Ajouté* — le cycle brouillon des factures fournisseur
  (#849), les factures annulées masquées dans la liste, le contrôle de clôture des brouillons fournisseur ; *Corrigé* —
  #853, #830, #835, #831, chacun en une phrase. `README.md:42` (*Factures fournisseurs & règlement*) : brouillon modifiable,
  validation, dévalidation.

## Mutations prévues

Chacune posée sur une base éphémère, tuée par le test nommé, restaurée par `git checkout -- <fichier>` puis `touch`
(`docs/agents-orchestration.md` § 7) ; résultat au Dev Agent Record.

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | `create_draft_in_tx` passe l'écriture d'achat | AC-6 (aucune écriture neuve) |
| M2 | `validate_in_tx` laisse `purchase_journal_entry_id` nul | AC-10 / contrainte AC-1 |
| M3 | dévalidation : garde du rapprochement retirée | AC-11 `MATCHED_BANK_TRANSACTION` |
| M4 | dévalidation : garde du lot `generated` retirée | AC-11 |
| M5 | dévalidation : contre-passation au lieu de la suppression (`reverse_owned_in_tx`) | AC-11 (−1 écriture, aucune contre-passation) |
| M6 | dévalidation : statut `paid` admis | AC-11 `SUPPLIER_INVOICE_PAID` |
| M7 | dévalidation : verrou optimiste ignoré | AC-11 `OPTIMISTIC_LOCK_CONFLICT` |
| M8 | `PUT` admis sur une facture ouverte | AC-8 |
| M9 | `DELETE` sans `detach_in_tx` | AC-9 (document `to_complete`) |
| M10 | `DELETE` sans la garde d'historique de lot | AC-9 (code au lieu de l'erreur de clé) |
| M11 | la complétion appelle `create_validated_in_tx` | AC-7 |
| M12 | propositions : `status IN ('draft','open')` | AC-5 (1) |
| M13 | `PUT` : règles « document » sautées | AC-8 (`QR_FIELDS_FROM_DOCUMENT`, `AMOUNT_MISMATCH`) |
| M14 | échéance antérieure admise | AC-14 |
| M15 | `CHECK` « écriture si et seulement si non brouillon » retiré de la migration | AC-1 |
| M16 | K8 compte les factures ouvertes | AC-16 |
| M17 | filtre `status` de la liste ignoré | AC-15 |
| M18 | `cancel` d'un brouillon admis (tête retirée) | AC-13 |
| M19 | audit `supplier_invoice.unvalidated` retiré | AC-11 |
| M20 | `validate_in_tx` sans le verrou de la facture en premier | AC-10 (test de course : deux validations concurrentes, une seule écriture) |
| M21 | fiche : bloc *Payer* affiché pour un brouillon | AC-22 |
| M22 | fiche : *Supprimer* offert au Comptable | AC-22 |
| M23 | formulaire : date du jour avec un document | AC-24 (b) |
| M24 | formulaire : échéance antérieure envoyée | AC-24 (b) |
| M25 | formulaire : `unitPrice` brut envoyé | AC-24 (c) |
| M26 | total vivant en `formatSwissAmount` | AC-24 (d) |
| M27 | liste : `formatSupplierInvoiceTotal` | AC-21 |
| M28 | liste : annulées affichées par défaut | AC-21 |
| M29 | `supplierInvoiceUnvalidateMessage` : un code rend le repli | AC-25 |
| M30 | lots : filtre `open` retiré de l'écran | AC-27 |

## Ce que les gardes ne voient pas (limites)

- **L1** — rc2 → rc1 (C-v017-4-3) : même version Cargo ; retour par la sauvegarde d'avant la mise à jour seulement.
- **L2** — une écriture d'achat supprimée par la dévalidation creuse un trou de numérotation (comme le client) ; le
  journal d'audit la garde.
- **L3** — un brouillon n'est pas compté aux rapports (aucune écriture) : seule la clôture le signale (K8), et la
  liste par son badge.
- **L4** — la dévalidation d'une facture qui a l'historique d'un lot **confirmé** n'est pas refusée (le règlement a été
  annulé explicitement ; la fiche montre ce lot) : la banque a pu exécuter l'ordre — si le montant change, le manuel dit
  de vérifier le relevé.
- **L5** — l'avis « date de facture postérieure au dépôt de plus de quelques jours » (#853, point 3) n'est pas fait :
  issue demandée.
- **L6** — les factures annulées restent dans les livres (Q1).

## Ce que la release doit contenir (orchestrateur ; ne pas prescrire au développeur)

1. Candidate `v0.17.0-rc2` (Q2) ; notes : deux migrations, la limite L1, le changement de contrat de
   `POST /supplier-invoices`.
2. **Avant la mise à jour du NAS** : sauvegarde complète (base, `.keshbackup`, documents) ; aucune recette ne s'écrit
   dans la base de Guy (`v017-donnees-a-garder.md`, point 3) — la recette du cycle se joue sur une installation jetable.
3. PR de release : `closes #849`, `closes #830`, `closes #835`, `closes #831` ; **#853** : `closes` si Guy accepte le
   renvoi du point 3 à une issue neuve (L5), sinon `refs`.
4. Issue à ouvrir : #853 point 3 (avis de date). Commenter #849 (prémisse : la règle client est la suppression ;
   statut `open` gardé) et #830 (remède : parser une fois, non une substitution de chaîne).

## Dev Notes (communes aux deux sous-fiches)

### Le manuel (axe de revue)

Toute passe de revue de cette story vérifie les manuels FR **au PDF aplati** contre le code : « enregistrer passe
l'écriture » ne doit plus se lire nulle part ; « ouverte » ne doit jamais désigner un brouillon.

### Propagation

Après chaque correctif, greper la **valeur** : `\bopen\b` et `'open'` dans les fichiers de l'inventaire ; `sept`/`7`
près de `checks` ; `87`/`88` et `53`/`54` (AC-1) ; `22`/`23` (`post_restore.rs`) ; « une migration » (CHANGELOG, admin
manual) ; « six stories ».

### Références

- Patron client : `repositories/invoices.rs` (`update` `:1080`, `delete` `:1343`, `unvalidate` `:1501`,
  `validate_invoice` `:2038`) ; `routes/invoices.rs` (`update_invoice` `:736`, `delete_invoice` `:797`,
  `validate_invoice_handler` `:815`, `unvalidate_invoice_handler` `:862`) ; `docs/api-external.md:460-498` ;
  `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (dévalidation `:341-370`, `:1089-1093`, `:1453`) et `edit/`.
- Factures fournisseur : `repositories/supplier_invoices.rs`, `routes/supplier_invoices.rs`,
  `routes/imported_supplier_invoices.rs`, `repositories/imported_supplier_invoices.rs`,
  `frontend/src/lib/features/supplier-invoices/`, `frontend/src/routes/(app)/supplier-invoices/`.
- Migrations : `CLAUDE.md` § *Migration breaking policy* ; patron récent `v016-2a-releve-manuel-backend.md` AC-2,
  AC-2-bis, AC-3-bis.

## Questions au Project Lead

1. **Q1 — Supprimer les factures annulées (C-v017-4-14).** Options (A) suppression réelle de la facture et de ses deux
   écritures, (B) masquage dans la liste, (C) suppression de la seule facture — détaillées au § *Les factures
   annulées*. **Recommandation : (B), livré par cette story ; pas de (A)** : vos factures annulées sont des corrections
   régulières, nettes à zéro, que l'art. 958f CO demande de garder ; le brouillon en supprime la cause. Si vous voulez
   néanmoins (A), une story dédiée, avec ses conditions (jamais payée, exercice ouvert, période non verrouillée, audit des
   deux écritures, deux trous de numérotation).
2. **Q2 — La candidate (C-v017-4-18).** `v0.17.0-rc2`, même version Cargo `0.17.0` : une sauvegarde de la rc2 qui porte
   un brouillon ne se restaure pas dans la rc1 ; le retour passe par la sauvegarde d'avant la mise à jour. D'accord ?
3. **Q3 — « Enregistrer et valider » (C-v017-4-15).** La fiche suit la facture client : on enregistre le brouillon, puis
   on valide depuis sa fiche. Un second bouton au formulaire ferait les deux d'un geste. Le voulez-vous ?
4. **Q4 — Le libellé de l'état validé (C-v017-4-2).** Il reste « Ouverte » (« validée, elle est ouverte, à payer »),
   comme partout dans les manuels. Préférez-vous « Validée » à l'écran ?
5. **Q5 — #853, point 3.** L'avis « date de facture bien postérieure à la date de dépôt du document » n'est pas fait ;
   une issue le garde. D'accord pour fermer #853 avec la release, cet avis renvoyé à l'issue neuve ?

## Change Log

- 2026-10-31 — Fiche créée (Opus 5.5, autonomie), découpée dès la rédaction en `v017-4a` et `v017-4b` (C-v017-4-1) ;
  complétée le même jour de #853, #830, #835, #831 sur décision du Project Lead (C-v017-4-20). Registre
  `v017-4-choix-autonomes.md` (C-v017-4-1 à 20). Questions Q1 à Q5. Prochaine étape : validation P1
  (`v017-4-validate-prompt-p1.md`, Sonnet, lentilles R et F).
