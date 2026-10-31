# Story v017-4 : factures fournisseur — brouillon, validée, payée

Status: split

*Fiche rédigée le 2026-10-31, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0175`, branche
`story/v017-4-facture-fournisseur-brouillon` partie de `8e126cfd` (tête de `release/0.17.0`, candidate
`v0.17.0-rc1`) ; la story se fusionne sur la branche de release. Registre : `v017-4-choix-autonomes.md`
(C-v017-4-1 à C-v017-4-40). **Remédiée après la validation P3** (2026-10-31, Change Log ; C-v017-4-37 à 40). **Remédiée après la validation P2** (2026-10-31, Change Log ; C-v017-4-34 à 36). **Remédiée après la validation P1** (2026-10-31, Change Log ; C-v017-4-26 à 33). **Amendée le même jour par les décisions de l'orchestrateur sur Q1 à Q5** (C-v017-4-21 à 25, § *Questions au Project Lead*). **Découpée dès la rédaction** en `v017-4a` (backend) et `v017-4b` (écran) — § *Découpage*,
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
  traité en **question** (Q1), non prescrit : § *Les factures annulées*. *(Décision du 2026-10-31, C-v017-4-21 : option (B),
  le masquage, livrée ici ; la suppression réelle n'est pas dans la 0.17.0 — limite L6.)*
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
| D2 | `draft` s'ajoute ; `open` reste la valeur stockée de « validée, à payer », libellé **« Validée »** à l'écran (décision Q4 ; première rédaction : « Ouverte ») ; aucune ligne réécrite | C-v017-4-2, **C-v017-4-24** |
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
| D14 | Factures annulées masquées par défaut dans la liste ; aucune suppression d'écriture | C-v017-4-14, **C-v017-4-21** |
| D15 | Écran : badge, page d'édition, actions par statut ; pas de « Enregistrer et valider » | C-v017-4-15, **C-v017-4-23** |
| D16 | Pas de motif de dévalidation calculé d'avance | C-v017-4-16 |
| D17 | Dépôt : quatre fonctions neuves ; `create_validated` composé, pour seeds et tests | C-v017-4-17 |
| D18 | Candidate `v0.17.0-rc2`, limite L1 aux notes | C-v017-4-18, **C-v017-4-22** |
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

**`sourceDocument` enrichi** *(validation P1, F5 ; C-v017-4-30)* : partout où la réponse porte déjà `sourceDocument`
(`api-external.md:823`), l'objet gagne `importedSupplierInvoiceId`, `linkOrigin` (`"completion"`, `"attachment"` ou
`null` pour un lien antérieur à la 0.17.0, C-v017-4-29) et `qr` — `null` pour un document sans QR, sinon les champs que
lit le mode « document avec QR » du formulaire (`creditorIban`, `isQrIban`, `referenceType`, `referenceValue`, `amount`,
`currency`, nom et adresse du créancier) ; jamais le chemin de stockage ni l'empreinte. Ajout de champs, sans retrait.

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
`ENTRY_LETTERED`. *(Validation P3, R9.)* Le lot `generated` parle **avant** l'exercice clos, à l'inverse de
`supplier_invoice_cancel_blocker` (où le lot vient « en dernier — le lever est le geste le plus lourd, et il serait vain
avant un exercice clos ») : la dévalidation suit le patron de **sa** fonction sœur, `invoices::unvalidate`, dont les
empêchements propres à la pièce (règlement, avoir, rappels, envoi, rapprochement) précèdent tous ceux de `delete_in_tx`
— le lot est ici un empêchement propre à la pièce. Écart voulu, écrit (C-v017-4-40). *(Le statut avant la version : patron de `invoices::unvalidate`, qui juge « non validée » avant le
verrou optimiste ; l'ordre `exercice clos` puis `postérieur` est celui que `delete_in_tx` rend.)*

### Verrous et rejeu *(réécrit à la validation P1, R3 ; C-v017-4-28)*

| Route | Ordre des verrous | Rejeu | Registre `audit_route_registry.rs` |
|---|---|---|---|
| `POST /supplier-invoices` (brouillon) | **aucun verrou exclusif** : la passe d'enregistrement lit fournisseur, projet et comptes **sans verrou** — le projet par une lecture ordinaire (existe, de la société, non archivé), **pas** par `validate_taggable_in_tx` (sentinelle `companies` et projet `FOR UPDATE`, `supplier_invoices.rs:424` — citation corrigée à la validation P3, R7), le jugement d'archivage étant refait sous verrou à la validation *(validation P2, R7)* ; seule l'insertion, qui garde les verrous **partagés** de ses clés étrangères (société, fournisseur, projet, comptes des lignes) — *(la phrase « compatibles entre eux, d'où l'absence d'enveloppe » de la P2 était fausse : validation P3, R2)* | ~~sans enveloppe~~ *(révisé à la validation P3, R2 ; C-v017-4-37)* **enveloppe gardée** : `retry_on_deadlock("supplier_invoices::create", …)` (`routes/supplier_invoices.rs:641`) reste — les verrous partagés de l'insertion ne sont **pas** compatibles avec tout : une ligne posée sur les comptes A puis B tient S(A) et attend S(B) quand une validation aux comptes B puis A tient X(B) et attend X(A) (cycle), et la clé étrangère `company_id` attend tout porteur de la sentinelle exclusive (`bank_accounts.rs:1617`) | `Traced, SansEcritureAuJournal` (reclassée depuis `Traced, Rejouee`, `:425`) |
| `POST …/complete` | inchangé : le document `FOR UPDATE`, puis l'insertion | **gardé** (`retry_app_on_deadlock`, inchangé) | *(validation P2, R6)* reclassée de `Traced, Rejouee` (`:430`) en `Traced, SansEcritureAuJournal` — la colonne dit « écrit au journal » (`audit_route_registry.rs:43-45`), non « a une enveloppe » ; elle reste rejouée, et le décompte du point **(vi)** (« dix-huit routes `SansEcritureAuJournal` sont rejouées quand même », `:158`) passe à dix-neuf ; le point **(ii)** est recompté avec les scénarios de la validation et de la dévalidation |
| `PUT /supplier-invoices/{id}` | la facture `FOR UPDATE` ; puis le document joint en **lecture ordinaire** (ses données de QR et `link_origin` ne changent qu'avec un détachement, bénin) ; lignes `DELETE` puis `INSERT` | **`retry_on_deadlock("supplier_invoices::update", …)`** *(validation P3, R2 : même cycle que la création, par les clés étrangères des lignes)* | `Traced, SansEcritureAuJournal` (rejouée quand même, point (vi)) |
| `DELETE /supplier-invoices/{id}` | *(réécrit à la validation P2, R1 = F1 ; C-v017-4-34)* **la facture d'abord, puis `detach_in_tx`** : la facture `FOR UPDATE`, son statut, les articles de lot en lecture verrouillante, **puis** `detach_in_tx` (qui verrouille l'entrée de `uq_imported_supplier_invoice` du brouillon, ou son intervalle), puis le `DELETE`. C'est l'ordre « facture → entrée d'unicité » de `attach_in_tx` (document neuf, facture, sentinelle, `UPDATE` qui insère la clé) : un rattachement concurrent attend la facture, le `DELETE` atteint l'index sans conflit. L'ordre inverse est **celui que le doc-comment de `detach_in_tx` interdit** (`imported_supplier_invoices.rs:311-314` : « un détachement qui demanderait la facture **après** le document formerait un cycle avec [`attach_in_tx`] (document neuf, puis facture, puis entrée d'unicité) ») | **`retry_on_deadlock("supplier_invoices::delete", …)`** *(validation P3, R2 ; C-v017-4-37)* : l'ordre de `attach_in_tx` évite le cycle avec le rattachement, l'enveloppe couvre le reste (articles de lot, clés étrangères) — aucune limite résiduelle sur une base en production | `Traced, SansEcritureAuJournal` |
| `POST …/validate` | la facture `FOR UPDATE`, puis exactement l'ordre (0) à (5) du doc-comment de l'actuel `create_in_tx` (`supplier_invoices.rs:325-387`) — comptes de charge `FOR UPDATE`, réglages, comptes désignés partagés, exercice, journal. Cycle **connu** avec la confirmation d'un lot et l'acceptation d'un lot de rapprochement (sentinelle `companies` partagée puis facture suivante) | **`retry_on_deadlock("supplier_invoices::validate", …)`** | `Traced, Rejouee` |
| `POST …/unvalidate` | la facture `FOR UPDATE` ; lots `generated` et rapprochement en **lecture verrouillante** (`LOCK IN SHARE MODE`, patron de `lock_for_bank_settlement_in_tx` (c)) ; `UPDATE` ; `delete_in_tx` (écriture et exercice, exercices postérieurs, lignes) | **`retry_on_deadlock("supplier_invoices::unvalidate", …)`** | `Traced, Rejouee` |

`crates/kesh-api/tests/rejeu_interblocage_e2e.rs:865` (`capture.exiger_un_rejeu("supplier_invoices::create")`) devient
`exiger_un_rejeu("supplier_invoices::validate")`, son scénario porté sur la validation d'un brouillon (même montage de
l'interblocage) ; un second scénario exige le rejeu de `supplier_invoices::unvalidate`. Aucune lecture ordinaire ne
précède un verrou dont une garde a besoin : la contre-passation de l'achat ne se fait que par `cancel_in_tx`, qui
verrouille d'abord la facture.

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
| `repositories/payment_batches.rs:376-400` | statut, écriture | *(corrigé à la validation P3, R3 = F4 ; C-v017-4-38)* **change** : le tuple `InvoiceCoordsRow` (`:378-385`) décode `purchase_journal_entry_id` en `i64`, **avant** le contrôle de statut (`:400`) ; pour un brouillon (`NULL`), le décodage échouerait en `500`. Son dernier champ devient `Option<i64>`, et l'écriture n'est extraite qu'**après** `status != "open"` → `failed[]` `SUPPLIER_INVOICE_NOT_OPEN` (§ *Pattern batch*) ; test de l'AC-5 (4) |
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
| `repositories/reconciliation_cancel.rs:116` (`status = 'paid'`), `:387` | statut, numéro | sans effet (un brouillon n'a pas de règlement) *(ajouté à la validation P1, R7)* |
| `repositories/payment_batches.rs:81`, `:588`, `:842`, `:916` | facture d'un lot | sans effet : un brouillon n'entre jamais dans un lot (`:400`) ; un brouillon dévalidé qui en garde l'historique se lit comme une facture (`:916`, lots confirmés) *(R7)* |
| `repositories/journal_entries.rs:2262`, `:2265` (côté règlement), `:2765` | écritures de la facture | sans effet *(R7)* |
| `frontend/…/supplier-invoices/SupplierInvoiceTotal.svelte:23-30` | `status !== 'cancelled'` | un brouillon au total sous le centime (facture dévalidée) affiche l'avertissement : voulu *(R7)* |
| `frontend/…/supplier-invoices/supplier-invoices.types.ts:9`, `:36` | type du statut, `purchaseJournalEntryId: number` | `'draft'`, `number \| null` (AC-21) *(R7)* |

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

**Décision (2026-10-31, orchestrateur, mandat autonome du Project Lead — C-v017-4-21)** : **(B)**, livrée par cette
story (AC-15, AC-21). **(A) n'est pas dans la 0.17.0** : elle sera soumise au Project Lead à part. Limite L6.

## Découpage

*C-v017-4-1 : plus de cinq modules ; aucun cycle Cargo ; le backend se teste seul en HTTP.*

- **`v017-4a-facture-fournisseur-brouillon-backend.md`** — `kesh-db` (migration, dépôt, contrôle de clôture),
  `kesh-api` (routes, erreurs, audit, MCP, export, registres), `kesh-i18n` (messages d'erreur et libellés d'audit),
  `kesh-seed`, `docs/api-external.md`, tests Rust, textes de la migration (CHANGELOG en-tête et *Modifié*, manuel
  d'administration et son PDF).
- **`v017-4b-facture-fournisseur-brouillon-ecran.md`** — le frontend (liste, fiche, page d'édition, formulaire et ses
  quatre défauts, écran de clôture), les clés de l'écran, le manuel utilisateur et son PDF, l'E2E, les captures, le
  CHANGELOG *Ajouté* et *Corrigé*, le README. **Dépend de la 4a.**

**Dérogation écrite (validation P1, R6 ; C-v017-4-31).** Après la 4a seule, l'écran existant crée des brouillons qu'il
ne sait pas valider, et des specs E2E rougissent hors de la liste des échecs attendus. Donc : les deux sous-fiches vivent
sur la **même branche** et ne se **publient qu'ensemble** (une seule PR, fusionnée sur la branche de release après la
4b). La 4a joue au dernier commit de son code le gate complet backend, Vitest et des **E2E ciblés** de ses routes (specs
API seulement : la spec neuve de l'AC-30 réduite à ses étapes par l'API, et `reconciliation-supplier.spec.ts` avec
`createSupplierInvoiceViaApi` qui valide, F3). *(Validation P2, F10.)* Ces artefacts, la 4a les **écrit** : le helper
`createSupplierInvoiceViaApi` qui valide après la création, le helper du relevé saisi par l'API, et la spec
`supplier-invoice-draft-cycle.spec.ts` réduite à ses étapes par l'API ; la 4b y ajoute les étapes par l'écran et adapte
les autres specs (AC-30) ; l'**E2E complet** se joue au dernier commit de code de la **4b**, sur la
branche commune, et couvre les deux. Le Dev Agent Record de la 4a dit « E2E complet au dernier commit de code de la 4b »,
jamais « vert ».

| AC | Sous-fiche |
|---|---|
| AC-1 à AC-20 | `v017-4a` |
| AC-21 à AC-32 | `v017-4b` — sauf le début de l'AC-30 (helpers E2E, étapes API de la spec neuve), livré par la 4a (validation P3, R8) |

| Mutations | Sous-fiche |
|---|---|
| M1 à M20 (M12a et M12c, M20-bis à M20-quater compris ; M12b retirée) | `v017-4a` |
| M21 à M34 (M21-bis compris) | `v017-4b` |

## Critères d'acceptation

*Chaque AC se prouve par un test **écrit rouge d'abord** (commit des tests rouges avant le code, nommés au Dev Agent
Record avec leur sortie rouge), sauf mention. Tests Rust de dépôt et d'API : squash (`#[sqlx::test(migrations =
"../kesh-db/test-schema")]`), sauf ceux de la migration, sur le vrai `MIGRATOR`.*

### Sous-fiche 4a — backend

- **AC-1 — La migration.** `crates/kesh-db/migrations/20261031000001_supplier_invoices_draft.sql` : un en-tête qui dit
  pourquoi (C-v017-4-2, C-v017-4-3) et que la migration est **breaking** au sens de P1 ; puis
  1. `ALTER TABLE supplier_invoices MODIFY COLUMN purchase_journal_entry_id BIGINT NULL;` — même type ; la clé
     étrangère `fk_supplier_invoices_purchase_je` est gardée. *(Précisé à la validation P1, R10.)* Le dépôt n'a aucun
     précédent d'assouplissement d'une colonne de clé étrangère : l'essai se fait sur **MariaDB 10.11**
     (`docker-compose.dev.yml`), base `kesh_v0174`. Si MariaDB refuse, le **repli** est une **instruction unique**, sous
     un **nom neuf** de clé *(validation P3, R5 ; C-v017-4-39 : MySQL refuse le même nom retiré et reposé dans une
     instruction — erreur 1826 —, et MariaDB 10.11 n'est pas établi)* — `ALTER TABLE supplier_invoices DROP FOREIGN KEY
     fk_supplier_invoices_purchase_je, MODIFY COLUMN purchase_journal_entry_id BIGINT NULL, ADD CONSTRAINT
     fk_supplier_invoices_purchase_je_v2 FOREIGN KEY (purchase_journal_entry_id) REFERENCES journal_entries(id) ON DELETE
     RESTRICT;` —, de sorte qu'un échec ne laisse jamais la table sans sa clé. L'essai joue, sur la base jetable, **les
     trois** formes dans l'ordre (le `MODIFY` seul, le repli au même nom, le repli au nom neuf) et inscrit chaque résultat
     au Dev Agent Record ; la migration retient la première qui passe ; le nom effectif de la clé se propage au squash et
     au contrôle P8 ; *(réécrit à la validation P2, R5 ; C-v017-4-35)* les `CHECK` des points 2 à 4 suivent
     le précédent du dépôt — « MariaDB : un CHECK ne se modifie pas en place → DROP CONSTRAINT + ADD CONSTRAINT, chacun
     dans son instruction » (`20261001000004_invoice_settlements_write_off.sql:17-19`) —, avec une précaution de plus :
     la contrainte de statut est posée sous un **nom neuf**, **avant** le retrait de l'ancienne. Instruction A :
     `ALTER TABLE supplier_invoices ADD CONSTRAINT chk_supplier_invoices_status_v2 CHECK (…), ADD CONSTRAINT
     chk_supplier_invoices_entry_iff_not_draft CHECK (…), ADD CONSTRAINT chk_supplier_invoices_draft_without_settlement
     CHECK (…);` ; instruction B : `ALTER TABLE supplier_invoices DROP CONSTRAINT chk_supplier_invoices_status;`. Chaque
     `ALTER` est atomique ; si la migration échoue entre A et B sur la base de production de Guy, `status` reste gardé
     par **les deux** contraintes (l'ancienne, plus stricte, refuse `draft` ; aucune ligne `draft` n'existe), jamais
     par aucune — l'ordre inverse (`DROP` puis `ADD`, le précédent tel quel) laisserait `status` sans garde entre les
     deux. La migration est jouée **deux fois** sur une base de dev peuplée (état 87, factures des quatre statuts) :
     une application, puis, la base remise à l'état 87 par sa sauvegarde, une seconde — résultat identique, relevé au
     Dev Agent Record avec l'erreur éventuelle du premier essai ;
  2. `chk_supplier_invoices_status_v2` : `CHECK (status IN ('draft', 'open', 'paid', 'cancelled'))`, l'ancienne
     `chk_supplier_invoices_status` retirée **après** (instruction B) ;
  3. `chk_supplier_invoices_entry_iff_not_draft` : `CHECK ((status = 'draft') = (purchase_journal_entry_id IS NULL))` ;
  4. `chk_supplier_invoices_draft_without_settlement` : `CHECK (status <> 'draft' OR (settlement_type IS NULL AND
     settlement_bank_account_id IS NULL AND settlement_account_id IS NULL AND settlement_journal_entry_id IS NULL AND
     paid_at IS NULL))` ;
  4 bis. *(validation P1, R5 ; C-v017-4-29)* `ALTER TABLE imported_supplier_invoices ADD COLUMN link_origin
     VARCHAR(16) NULL, ADD CONSTRAINT chk_imported_si_link_origin CHECK (link_origin IS NULL OR link_origin IN
     ('completion', 'attachment'));` — nullable, sans défaut, **non breaking** pour elle-même ; les lignes existantes
     restent `NULL` (« lien antérieur à la 0.17.0 »), aucun backfill (P7 : rien à trier pour cette instruction) ; une
     sauvegarde 0.16.x ou rc1, qui n'a pas la colonne, s'importe (colonne non requise, `admin_backup/import.rs:606-611`) ;
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
  crates/kesh-db/tests/migrations_upgrade_path.rs` (à `8e126cfd` : `:44`, `:46`, `:63-64`, `:88` et `:89` historiques à laisser (validation P3, R7 : `:89` porte « = 54 », qui coexistera avec le `54` neuf),
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
  un **document** `completed` lié par complétion, une autre à **document joint après coup** (non complété) *(validation P1, R5)*,
  une présente dans un **lot confirmé** ; puis appliquent le reste et vérifient :
  1. chaque ligne de `supplier_invoices`, `supplier_invoice_lines`, `imported_supplier_invoices`,
     `payment_batch_items`, `journal_entries`, `journal_entry_lines` et des tables de lettrage touchées se relit
     **colonne par colonne à l'identique** (`version` et `updated_at` compris) ;
  2. les mêmes factures se relisent **par le code** : `supplier_invoices::get`, `list`,
     `get_settlement_view` réussissent et rendent les statuts d'origine (`open`, `paid`, `cancelled`) ;
  3. **le cas de Guy** : la facture ouverte migrée se **dévalide** (écriture d'achat supprimée, `draft`), se **modifie**
     (`update_draft`, un total corrigé), se **revalide** (écriture neuve, `open`) ;
  4. `_kesh_version.kesh_version_min_required` = `0.17.0` ; `imported_supplier_invoices.link_origin` vaut `NULL` sur
     toute ligne existante ;
  5. *(validation P1, R5 — le cas de Guy)* la facture ouverte **à document joint après coup** se dévalide, se modifie
     par `PUT` (total, compte, coordonnées du corps acceptées) et se revalide : aucune règle de la complétion ne
     s'impose (devise, montant du QR, coordonnées du QR).
- **AC-3 — Sauvegardes.** (a) Le test de la v016-2a qui importe dans la 0.17.0 une sauvegarde au manifeste de 0.16.x
  (`crates/kesh-api/tests/admin_full_import_e2e.rs:4681-4841`) porte en plus des factures
  fournisseur **ouverte**, **payée** et **annulée**, et au moins un document lié. *(Validation P2, R2 ; C-v017-4-36.)* La
  sauvegarde forgée **ne porte pas** `link_origin` : le test retire la colonne des `column_names` et de chaque ligne de
  `imported_supplier_invoices` (et recalcule l'empreinte de la table si le manifeste la porte) — *(outils nommés à la
  validation P3, R6)* `ouvrir_tout` (`admin_full_import_e2e.rs:3622`), `recrire_table(…, "imported_supplier_invoices",
  |o| { o.remove("link_origin"); })` (`:3687`, qui récrit les lignes et l'empreinte mais **pas** `columnNames`), le retrait
  de `"link_origin"` de `manifest["tables"]["imported_supplier_invoices"]["columnNames"]` à la main, puis `rezip_tout`
  (`:3653`, qui garde `files/` ; `rezip`, `:263`, ne l'écrit pas et perdrait le document lié) — c'est ce retrait qui
  exerce le chemin « colonne absente de la source » (`admin_backup/import.rs:606-611`) qu'exige la contrainte « données à
  garder » ; après l'import, `link_origin` vaut `NULL` ; le doc-comment `:4689-4695` (« la 0.17.0 n'ajoute … ni colonne »)
  est corrigé ; même exigence pour une sauvegarde de forme rc1. Puis : `200`, chacune se relit à l'identique par
  `GET /supplier-invoices/{id}`, et l'ouverte se dévalide puis se revalide. (b) Aller-retour dans la 0.17.0 d'une base
  qui porte un **brouillon** (écriture `null`) et une facture validée : export, import, chaque facture identique, le
  brouillon reste brouillon sans écriture. (c) L'inventaire des tables ne change pas (aucune table neuve) : le test
  existant de la v016-2a qui le compare passe sans modification. (d) **Limite L1, non testable** (binaire rc1) : une
  sauvegarde de la rc2 ne s'importe pas dans la rc1 — brouillon ou non, elle porte `link_origin`, colonne que la rc1
  ne connaît pas (`IMPORT_SCHEMA_MISMATCH`, `admin_backup/import.rs:597-604` ; aligné sur L1 à la validation P2, R13) —
  écrite aux notes de la candidate.
- **AC-4 — L'écriture d'achat devient facultative, partout.** `SupplierInvoice.purchase_journal_entry_id:
  Option<i64>` ; chaque site de l'inventaire « écriture d'achat » (§ *Inventaire clos*) adapté ou déclaré sans effet au
  Dev Agent Record ; `SupplierInvoiceResponse.purchase_journal_entry_id: Option<i64>` (`null` pour un brouillon) ; `SourceDocumentInfo` enrichi
  (§ *Contrat*, F5) ; export
  CSV : cellule vide ; aucun `unwrap()`/`expect()` neuf sur cette valeur en production — un état incohérent rend
  `DbError::Invariant`. Test : `GET` d'un brouillon rend `"purchaseJournalEntryId": null` (champ **présent**, vérifié
  par `get()` et non `is_null()` seul) ; l'export global d'une base à brouillon s'écrit.
- **AC-5 — Un brouillon n'est jamais payable ni proposé.** Tests, un par chemin, chacun sur un brouillon au montant
  exact d'un débit : (1) `GET` des propositions de réconciliation d'un débit (`reconciliation.rs:825`, `:885`) — absent ;
  (2) *(réécrit à la validation P2, R4)* le **masquage** des règles d'affectation (`:935`, factures « jamais
  proposées », `:916-922`) : une règle qui couvre le débit, un brouillon du même fournisseur (indice) au montant voisin →
  **la règle reste proposée** (un brouillon ne masque rien) ; (3) `POST /reconciliation/accept` d'une proposition
  fournisseur visant un brouillon — `200`, `failed[]` **`RECONCILIATION_INVOICE_NOT_ELIGIBLE`**, `details.reason =
  "supplier_invoice_not_open"` (code réel, `routes/reconciliation.rs:3020-3023`, inchangé ; corrigé à la validation P2,
  R3) (§ *Pattern batch*) ; (4)
  `POST /payment-batches` incluant un brouillon — refus par facture `SUPPLIER_INVOICE_NOT_OPEN` dans `failed[]`, jamais un
  `500` (validation P3, R3 = F4 : le décodage de `payment_batches.rs:378-385` devient tolérant, inventaire) ; (5) `POST …/pay` —
  `409 ILLEGAL_STATE_TRANSITION` ; (6) `POST …/settlement/cancel` — `SUPPLIER_INVOICE_NOT_PAID` (inchangé) ;
  (7) `GET` de la fiche d'un brouillon : `cancellable: false`, `cancelBlockedBy: "SUPPLIER_INVOICE_DRAFT"` ; (8) les
  propositions de fournisseurs par IBAN comptent le brouillon (C-v017-4-11). Rien n'écrit au journal sur ces refus.
- **AC-6 — Créer un brouillon.** `POST /supplier-invoices` → `201`, `status: "draft"`, `purchaseJournalEntryId: null`,
  `version: 1` ; **aucune** ligne neuve dans `journal_entries` (compte avant/après) ; contrôles à l'enregistrement de
  C-v017-4-4, dans l'ordre actuel de `create_in_tx` pour ceux qui y sont (lignes non vides, projet, fournisseur, forme,
  comptes de charge), plus l'échéance (AC-14) dans la conversion commune ; un brouillon daté hors de tout exercice est
  **accepté** ; un compte de charge d'une autre société, inactif, d'un autre type ou non imputable est **refusé** avec
  les codes actuels. Audit `supplier_invoice.created`, instantané sans `journalEntryId`. Route classée au registre
  `audit_route_registry.rs` `Traced, SansEcritureAuJournal` (§ *Verrous et rejeu*). *(Validation P1, R2 ;
  C-v017-4-27.)* **Le total d'un brouillon** — `total_amount` — est calculé par **une fonction pure unique** de
  `kesh-core` (`kesh_core::accounting::purchase_invoice_totals(&[(line_total, vat_rate)]) -> (total_ht, total_vat,
  total_ttc)`, TVA arrondie **par ligne** par `line_vat_amount`), appelée par `create_draft_in_tx`, `update_draft` **et**
  `generate_purchase_journal_lines` (`supplier_invoices.rs:120`), qui ne calcule plus ses sommes lui-même (DRY).
  *(Validation P2, R9.)* Elle s'appuie sur le TTC canonique existant, `kesh_core::accounting::vat::invoice_total_ttc`
  (`vat.rs:46-63`, même formule), pour son TTC — ou le doublon se justifie au doc-comment ; et le contrôle
  `AMOUNT_MISMATCH` de la complétion (`routes/imported_supplier_invoices.rs:262-270`, qui refait la somme) l'appelle aussi. Tests :
  (a) un brouillon à TVA 8.1 %, société **sans** compte de TVA récupérable : il s'enregistre (`201`, `total_amount` =
  HT + TVA) ; sa **validation** rend `CONFIGURATION_REQUIRED` (`default_vat_recoverable_account_id`) ; (b) après `PUT`
  puis validation, `total_amount` = crédit de la ligne créanciers de l'écriture ; (c) tests unitaires de la fonction
  pure (TVA nulle, plusieurs taux, arrondi par ligne).
- **AC-7 — Compléter un document crée un brouillon.** `POST /imported-supplier-invoices/{id}/complete` → la facture
  relue, `status: "draft"`, aucune écriture ; document `completed`, lié ; audit `imported_supplier_invoice.completed`
  inchangé ; `link_origin` posé dans le même `UPDATE` que `mark_completed`, qui **reçoit l'origine en paramètre**
  *(validation P2, R8)* — `mark_completed` a deux appelants : la complétion (`routes/imported_supplier_invoices.rs:289`,
  `'completion'`) et `attach_in_tx` (`repositories/imported_supplier_invoices.rs:289`, `'attachment'`) ; `detach_in_tx`
  remet `NULL` ; le seed (`kesh-seed/src/pieges.rs:641`, par `attach_in_tx`) produit donc `'attachment'`. *(Validation
  P3, R7.)* Citation corrigée : l'appel de `mark_completed` dans `attach_in_tx` est `repositories/imported_supplier_invoices.rs:294`
  (non `:289`, qui est celui de la route) ; un troisième appelant, de test, suit la signature
  (`kesh-db/tests/supplier_document_attachment.rs:437`) ; `services/pieces.rs:88` destructure `ImportedSupplierInvoice`
  champ à champ : `link_origin` y est lu et **non exposé** à l'outil MCP des pièces (`PiecePublique` inchangé ;
  C-v017-4-40).
  *(Validation P2, R12.)* `link_origin` entre à l'export de souveraineté des documents
  (`exports/csv_tables.rs:1565-1594`, colonnes énumérées en dur), avec son test d'export. Les règles « document » (devise, routage IBAN et QRR, coordonnées prises du QR, `QR_FIELDS_FROM_DOCUMENT`,
  forme des lignes, `AMOUNT_MISMATCH`) sont **extraites** en une seule fonction de `kesh-api`, appelée par la complétion
  et par le `PUT` (AC-8) ; l'ordre des refus de la complétion est inchangé (les tests existants de
  `inbox_import_e2e.rs` sur ces refus passent sans changement d'assertion de code).
- **AC-8 — Modifier un brouillon (`PUT`).** Corps : celui de la création + `version`. Remplace l'en-tête et **toutes**
  les lignes (positions renumérotées depuis 0) ; `version + 1` ; mêmes contrôles qu'à la création. *(Réécrit à la
  validation P1, R5 ; C-v017-4-29.)* La fonction de l'AC-7 s'applique **seulement si la facture est née de la
  complétion de ce document** (`link_origin = 'completion'`) : coordonnées du corps refusées `QR_FIELDS_FROM_DOCUMENT`,
  coordonnées gardées celles du QR, devise, `AMOUNT_MISMATCH` si le TTC diffère du montant du QR. Un document **joint
  après coup** (`'attachment'`) ou lié avant la 0.17.0 (`NULL`) n'impose **rien** : le `PUT` se juge comme une saisie
  (coordonnées du corps, validées par `validate_payment_details`). Tests : document né de la complétion (les trois
  refus) ; document joint après coup, en devise ≠ CHF, au montant de QR ≠ total, aux coordonnées différentes de la
  facture — `PUT` accepté, coordonnées de la facture **inchangées** si le corps les reprend ; **le cas de Guy** :
  facture validée, document joint, dévalidée, éditée, revalidée. Refus : non brouillon → `409 ILLEGAL_STATE_TRANSITION` ; version périmée → `409 OPTIMISTIC_LOCK_CONFLICT` ;
  facture d'une autre société → `404`. Audit `supplier_invoice.updated`, instantanés avant et après. Aucune écriture au
  journal. Tests : chaque refus ; un `PUT` qui change fournisseur, dates, lignes, comptes, projet, coordonnées, puis
  relecture complète ; un `PUT` avec trois lignes puis une seule (lignes orphelines absentes). Un `PUT` sur une facture
  ouverte rend **le code** `ILLEGAL_STATE_TRANSITION` (assertion sur le code, non sur le seul `409` — R4).
- **AC-9 — Supprimer un brouillon (`DELETE`).** Administrateur seulement (un Comptable → `403`, une clé → `403
  API_KEY_ADMIN_FORBIDDEN`, montée avec les routes Administrateur comme `DELETE /invoices/{id}`). `204` ; facture et
  lignes supprimées ; document joint repassé `to_complete`, lien `NULL`, audit `imported_supplier_invoice.detached` puis
  `supplier_invoice.deleted` (instantané) dans la même transaction. Refus (C-v017-4-6) : ouverte →
  `409 SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST` ; payée, annulée → `409 ILLEGAL_STATE_TRANSITION` ; présente dans un
  lot (tout statut de lot) → `409 SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY` — jamais une erreur de clé étrangère.
  Ordre des verrous : **la facture puis le document** (§ *Verrous et rejeu*, réécrit à la validation P2, R1).
  *(Test déterministe, validation P3, R1 = F3 ; C-v017-4-38.)* `attach_in_tx` étant atomique, deux appels libres ne
  produisent l'entrelacement fautif qu'au hasard. Le test le pilote : A = une transaction qui rejoue **à la main** les
  étapes (1) à (3) de `attach_in_tx` (document neuf `FOR UPDATE`, facture `FOR UPDATE`, sentinelle) **sans** `mark_completed` ;
  B = `delete_draft` lancé dans une tâche ; attente que B soit bloqué par `attendre_une_requete_en_cours(&pool,
  &["FROM supplier_invoices WHERE id", "FOR UPDATE"], …)` (`test_fixtures.rs`, patron de
  `supplier_document_attachment.rs:330`, sonde par `PROCESSLIST`, sans privilège `PROCESS`) ; A appelle alors
  `mark_completed` et commite. Assertions : A et B aboutissent, aucune erreur 1213 ni `500` ; le document est rendu « à
  traiter » et le brouillon supprimé. Sous M20-quater, B a posé le verrou d'intervalle de `uq_imported_supplier_invoice`
  avant d'attendre la facture, et `mark_completed` de A l'attend : interblocage — le test rougit **à coup sûr**. Retenu
  plutôt que le protocole « intervalle tenu puis `FOR UPDATE NOWAIT` sur la facture » de F3, qui exigeait une sonde
  `INNODB_TRX`/`INNODB_LOCK_WAITS` (privilège `PROCESS`, interdit par `docs/agents-orchestration.md` § 6.1) et ne prouve
  que l'ordre, non l'absence d'interblocage ; un refus annule aussi le détachement (test :
  `DELETE` d'une facture ouverte à document — le document reste `completed`, lié).
- **AC-10 — Valider.** `POST …/validate` `{version}` → `200`, `status: "open"`, `purchaseJournalEntryId` posé,
  `version + 1`. L'écriture d'achat est **la même** qu'aurait passée la création de la 0.17.0-rc1 pour les mêmes
  données : test d'égalité ligne par ligne (comptes, débits, crédits, projet, journal `Achats`, date = date de facture,
  libellé) contre une **écriture attendue écrite en dur** — *(validation P1, R8)* et non contre `create_validated_in_tx`,
  qui en est la composition (comparaison tautologique, retirée) ; les assertions existantes de
  `supplier_invoices_repository.rs` sur les lignes de l'écriture d'achat, relues **avant** la modification et gardées
  telles quelles (golden), passent par `create_validated`. Refus : ceux de la création actuelle (exercice ouvert absent → `FISCAL_YEAR_INVALID` ; compte non imputable,
  inactif ; réglages ; comptes désignés ; projet archivé depuis l'enregistrement), plus `ILLEGAL_STATE_TRANSITION`
  (non brouillon) et `OPTIMISTIC_LOCK_CONFLICT`. Un refus n'écrit rien (facture toujours `draft`, aucune écriture). Audit
  `supplier_invoice.validated` (`journalEntryId`). Rejouée sur interblocage ; ordre des verrous au doc-comment (§
  *Verrous et rejeu*). *(Validation P1, R4.)* Test de course : deux validations concurrentes du même brouillon — une
  seule écriture **et** le perdant reçoit le **code** `ILLEGAL_STATE_TRANSITION` (il relit `open` sous le verrou) ; sans
  le verrou initial, il recevrait `OPTIMISTIC_LOCK_CONFLICT` : c'est le code qui tue M20. *(Pilotage, validation P3,
  R1 ; C-v017-4-38.)* A = `validate_in_tx` du brouillon, **non commitée** ; B = seconde validation dans une tâche ;
  attente que B soit bloqué (`attendre_une_requete_en_cours`, motif `FOR UPDATE` — sous l'ordre prescrit, B attend la
  facture ; sous M20, B a déjà lu `draft` par une lecture ordinaire et attend un compte de charge) ; A commite ; B rend
  `ILLEGAL_STATE_TRANSITION` (prescrit) ou `OPTIMISTIC_LOCK_CONFLICT` (mutant).
- **AC-11 — Dévalider.** `POST …/unvalidate` `{version}` → `200`, `status: "draft"`, `purchaseJournalEntryId: null`,
  `version + 1` ; l'écriture d'achat et ses lignes n'existent plus ; **aucune** contre-passation n'est passée (compte des
  écritures avant/après : −1) ; audits `journal_entry.deleted` et `supplier_invoice.unvalidated` (instantané, identifiant
  de l'écriture supprimée). Empêchements et précédence du § *Précédence*, chacun testé, paire par paire pour les
  voisins ; en particulier : payée → `SUPPLIER_INVOICE_PAID` ; lot `generated` → `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` ;
  écriture d'achat rapprochée (montée par `UPDATE bank_transactions SET matched_entry_id`) → `MATCHED_BANK_TRANSACTION`,
  et la transaction garde son lien ; exercice clos ; exercice postérieur clos ; période verrouillée. Le doc-comment de
  `delete_in_tx` (`journal_entries.rs:1808`) **et** le commentaire de `invoices.rs:1448` (« `unvalidate` est désormais le
  **seul** appelant de production à passer `false` ») nomment le second appelant du drapeau `false` *(validation P2, R11)*. Une facture payée dont on annule le règlement puis qu'on
  dévalide : test de bout en bout (dépôt). Chaque refus s'asserte sur son **code** (R4) : version périmée →
  `OPTIMISTIC_LOCK_CONFLICT`, distinct de `ILLEGAL_STATE_TRANSITION`.
- **AC-12 — `create_validated` pour les seeds et les tests.** `create`/`create_in_tx` renommés
  `create_validated`/`create_validated_in_tx`, composés de `create_draft_in_tx` puis `validate_in_tx` dans une même
  transaction (aucun corps dupliqué) ; aucune route ne l'appelle (`grep -rn "create_validated" crates/kesh-api/src`
  ne rend rien). Appelants mis à jour : `kesh-seed/src/pieges.rs:585`, les tests (`grep -rln
  'supplier_invoices::create\b'` à `8e126cfd`, dont `supplier_invoices_repository.rs` — 45 appels), `kesh-db/src/retry.rs`
  si c'en est un. Les tests d'API qui créent par HTTP une facture qu'ils paient ensuite — *(commandes corrigées à la
  validation P2, R13)* par la création (`grep -rln '"/api/v1/supplier-invoices"' crates/kesh-api/tests` :
  `inbox_import_e2e.rs`, `rejeu_interblocage_e2e.rs`, `services_rollout.rs`, `supplier_settlement_cancel_e2e.rs`) et par
  la complétion (`grep -rln '/complete' crates/kesh-api/tests` : `inbox_import_e2e.rs`, `mcp_enveloppe_e2e.rs`,
  `mcp_transport_e2e.rs`, `opening_balances_e2e.rs`) — appellent `…/validate` après la création ; leur liste triée au Dev Agent Record.
  `rejeu_interblocage_e2e.rs:865` : le rejeu de la **validation** remplace celui de la création (§ *Verrous et rejeu*).
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
  service `factures_fournisseur::lister` reçoit le filtre en option ; l'outil MCP de liste n'en change pas. L'écran des
  lots l'emploie (`status=open`, AC-27, F14).
- **AC-16 — K8 `DRAFT_SUPPLIER_INVOICES`.** Dans `closing_checks.rs` : sévérité `info`, compte des brouillons
  fournisseur de la société **datés dans l'exercice**, liste de 20 au plus `{id, label, date}` par date puis `id`
  (`label` : « <numéro> — <fournisseur> », ou le seul fournisseur sans numéro). *(Réécrit à la validation P1, R1 ;
  C-v017-4-26.)* K8 est rendu **en dernier**, après K7 : les index de K1 à K7 ne bougent pas
  (`closing_checks_e2e.rs:258`, `kesh-db/tests/closing_checks.rs:322` restent justes). Ses deux requêtes sont des
  constantes `…_SQL` inscrites à `ALL_SQL` (`closing_checks.rs:522`), sans mot de verrou, si bien que la garde **T-a2**
  (`:30-36`, tests du module) reste verte avec son compte de constantes recompté ; doc du module (`:3`, « exactement
  sept » → huit) et tableau d'en-tête mis à jour. `closing_checks_e2e.rs` : un brouillon dans Y → `attention`, `count`
  exact ; dans Y+1 → ignoré ; une facture validée → ignorée ; le total des contrôles rendus passe de **7 à 8**, `checks[7]`
  est K8.
- **AC-17 — MCP.** `STATUTS_FACTURE_FOURNISSEUR` gagne `("draft", StatutFactureFournisseur::Draft)` ;
  `ecriture_achat_id: Option<i64>`, `null` pour un brouillon ; les tests MCP qui figent ces sorties (`mcp_outils_e2e.rs`)
  couvrent un brouillon ; le contrat MCP (K23 et voisins) reste vert.
- **AC-18 — Audit, registres, messages.** Quatre actions neuves — `supplier_invoice.updated`, `.validated`,
  `.unvalidated`, `.deleted` — à `audit_labels.rs` (`:194-197`), au registre `audit_label_registry.rs`, et leurs libellés
  `audit-log-action-supplier-invoice-*` aux **quatre** catalogues (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}`) ;
  les quatre routes neuves au registre `audit_route_registry.rs` (`:425-455`), classées (validation, dévalidation :
  `Traced, Rejouee` ; `PUT`, `DELETE` : `Traced, SansEcritureAuJournal`), et la création reclassée de `Traced, Rejouee`
  (`:425`) en `Traced, SansEcritureAuJournal`, la complétion de même (`:430`, gardée rejouée), et les décomptes des
  points (ii) et (vi) de l'en-tête du registre recomptés (§ *Verrous et rejeu*, validation P2, R6) ; le compteur des
  routes tracées de `lib.rs` suit. *(Validation P3, R4.)* Assertions figées du registre à recompter :
  `the_registry_partition_is_what_the_story_declares` (`LIB_ROUTES.len(), 138` → 142 ; `traced, 131` → 135) et
  `the_replay_partition_is_what_the_story_declares` (`rejouees, 25` ; `sans_ecriture, 113` → 117 ; `tout.len(), 141` →
  145) — valeurs à reconfirmer au développement —, le point (vi) de l'en-tête (« dix-huit routes `SansEcritureAuJournal`
  sont rejouées quand même ») recompté avec la création, le `PUT`, le `DELETE` et la complétion rejoués (C-v017-4-37), et
  `le_manuel_dit_le_compte_du_registre` (`:978`), qui garde la phrase du manuel d'administration (AC-20).
  *(Validation P3, R11.)* Le message de `pay` d'un non-validé, `supplier_invoices.rs:828` (« seule une facture ouverte
  peut être réglée »), devient « seule une facture validée peut être réglée » (rendu à l'AC-5 (5)). Chaque code d'erreur neuf : variante `DbError` ou `AppError`, réponse dans
  `errors.rs`, clé aux quatre catalogues, repli égal au fr-CH ; `textes_coherents.rs` et `i18n_e2e.rs` verts, leurs
  décomptes figés recomptés. **Aucun message neuf ne nomme l'état `open` « ouverte »** (règle Q4, C-v017-4-24 ;
  validation P1, F2) : « une facture validée », « non validée ».
- **AC-19 — `docs/api-external.md`.** Une section neuve « Le cycle d'une facture fournisseur » (tableau du contrat, codes,
  précédence de la dévalidation, `details`) ; ⚠️ *depuis la v0.17.0*, `POST /supplier-invoices` et la complétion
  créent un **brouillon** sans écriture — **changement de contrat**, avec la conduite d'une intégration (valider après
  créer) ; `purchaseJournalEntryId` nullable ; § *Documents à traiter* (`:788-792`) ; contrôles de clôture : « sept »
  → « huit » et K8 (`:255-256`) ; tableau des codes d'erreur (`:907` et suivantes) ; § *Interblocages* (`:953`) ;
  filtre `status` de la liste ; `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`. *(Complété à la validation P1, F16, R9.)*
  `:823` (`sourceDocument`, à trois états : `PUT`, `validate`, `unvalidate` s'ajoutent à la liste des réponses, et les
  champs neufs de F5) ; `:827` (« quel que soit son statut (ouverte… ») : brouillon compris ; *(validation P3, F1)* `:837` (« **Deux** opérations
  sur les factures sont réservées à l'interface web … Tout le reste du cycle de facturation reste ouvert ») → **trois**,
  `DELETE /supplier-invoices/{id}` nommé ; `:839` (« `DELETE /invoices/{id}` ne supprime plus que des BROUILLONS … »)
  : le brouillon fournisseur se supprime de même, la dévalidation `POST /supplier-invoices/{id}/unvalidate` restant
  ouverte aux clés ; et la règle Q4 : **aucun
  texte de `api-external.md` ne nomme plus l'état `open` « ouverte »** — tri **par la valeur** (`grep -n -i 'ouvert'
  docs/api-external.md`), « ouverte ou payée » → « validée ou payée », le statut `open` et ses codes inchangés ; le
  « ouvert » du poste ouvert d'un compte reste.
- **AC-20 — Textes de la migration et de la version.** CHANGELOG `[0.17.0] — Non publié` : paragraphe d'en-tête —
  « six stories » → sept, « **une migration** » → **deux**, et *(validation P3, F9)* les deux phrases qui suivent
  (« qui relève `kesh_version_min_required` à 0.17.0 … » ; « La migration ne modifie aucune ligne existante ») réécrites
  migration par migration — la seconde ne relève rien (même valeur), pose la colonne `link_origin` nulle et ne réécrit
  aucune ligne (nommer `20261031000001_supplier_invoices_draft`, breaking,
  `min_required` inchangé à 0.17.0 ; la colonne `link_origin` neuve) — *(validation P1, F17 ; C-v017-4-33)* la limite
  L1 rc2 → rc1 **ne va pas** au CHANGELOG ni au manuel, qui décrivent la 0.17.0 publiée : elle va aux **notes de la
  candidate** seulement ; section *Modifié* : le changement de contrat de
  `POST /supplier-invoices` et de la complétion, `purchaseJournalEntryId` nullable, contrôles de clôture à huit. Manuel
  d'administration, § *Passer à la 0.17.0* (`docs/manual/fr/admin-manual.tex:2486`) : deux migrations, aucune ligne
  existante modifiée. *(Validation P1, F15.)* Le manuel d'administration ne décrit pas les sorties des outils MCP
  des factures fournisseur (`grep -n 'facture\\_fournisseur' docs/manual/fr/admin-manual.tex` ne rend que la liste des
  outils, `:3222-3223`) : **rien n'y est créé** ; le CHANGELOG *Modifié* dit `statut: draft` et `ecritureAchatId` nul.
  *(Validation P1, F1, F10, R9.)* Le manuel d'administration dit encore que le compte créanciers ou de TVA récupérable
  non imputable fait refuser « l'enregistrement » d'une facture fournisseur (`admin-manual.tex`, § *Compte créanciers*,
  `:3580`, et le paragraphe qui précède, `grep -n "enregistrement d" docs/manual/fr/admin-manual.tex`) : c'est désormais
  la **validation** ; et « les sept contrôles s'affichent » (`:1826`) → huit ; *(validation P3, R4)* `:2979` (« 131 des
  138 routes à verbe mutant … 131 + 4 + 3 = 138 ») suit le recompte de l'AC-18 (« 135 des 142 … 135 + 4 + 3 = 142 »,
  valeurs à reconfirmer), PDF régénéré. *(Validation P2, F5.)* Et
  `admin-manual.tex:2996` (« En usage courant, deux voies font disparaître une écriture … la dévalidation d'une facture
  … refusée sous neuf motifs ») : la dévalidation d'une facture fournisseur est une troisième forme de la même voie, avec
  ses propres motifs (§ *Précédence*) et son audit (`supplier_invoice.unvalidated`) ; `:2949` (« Ce qu'une clé
  `read-write` peut faire … dévalider une facture, donc supprimer l'écriture comptable … Les neuf refus ») : une clé
  `read-write` peut aussi **dévalider une facture fournisseur**, donc supprimer son écriture d'achat, sous les refus de
  la § *Précédence* ; `docs/api-external.md:81` (opérations réservées à l'interface) : le `DELETE` d'un brouillon
  fournisseur y est nommé. Règle Q4 : aucun texte neuf ou modifié
  ne nomme l'état `open` « ouverte ». PDF
  régénéré au commit de son `.tex`, contrôlé aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '`). Macros de version
  **inchangées** (release).

### Sous-fiche 4b — écran

- **AC-21 — Liste.** `SupplierInvoiceStatus` gagne `'draft'` ; `supplierInvoiceStatusLabel('draft')` → clé
  `supplier-invoices-status-draft` (« Brouillon », glossaire partie A, `docs/i18n-glossaire.md:92`) ; badge dans la liste.
  **Décision Q4 (C-v017-4-24)** : l'état stocké `open` s'affiche **« Validée »** — la clé
  `supplier-invoices-status-open` prend la valeur de `invoice-status-validated` de chaque catalogue : « Validée »
  (fr-CH `:613`), « Validiert » (de-CH `:619`), « Convalidata » (it-CH `:619`), « Validated » (en-CH `:619`) ; le repli de
  `supplier-invoice-helpers.ts:37`, son commentaire (`:31`) et son banc (`supplier-invoice-helpers.test.ts:36`, `:48`,
  `:54`) suivent ; le glossaire (`docs/i18n-glossaire.md:93`) dit que, pour une facture fournisseur, l'état se nomme
  « validée ». **Aucun texte de l'écran ne nomme plus cet état « ouverte »** — inventaire clos par `grep -n -i
  'ouvert\|offen\|apert\|\bopen\b'`, *(élargi à la validation P1, F2 ; C-v017-4-32)* **par la valeur, sur tout le
  dépôt** — *(restreint à la validation P2, F6, comme C-v017-4-32 : `open` ne se cherche qu'**en prose**, jamais dans les
  identifiants, codes et clés, sans quoi le `grep` rend plus de 4 600 lignes, dont 126 `open` du seul catalogue en-CH ;
  « ouvert », « offen », « apert » se cherchent partout)* : les quatre catalogues **entiers**, `frontend/src` **et** `frontend/tests/e2e` entiers, `crates/` (messages et
  replis), `docs/`, `README.md`, `website/`. À `8e126cfd`, en plus des clés fr-CH `reconciliation-failed-rule-masked-invoice-candidate`
  (`:1440`), `supplier-invoices-settlement-cancel-confirm` (`:2571`), `supplier-invoices-settlement-cancelled` (`:2573`),
  `payment-batches-failed-supplier-invoice-not-open` (`:2753`), `payment-batches-no-open` (`:2842`) et de leurs replis
  (`[id]/+page.svelte:182`, `:202`, `payment-batches/+page.svelte:150`) : la clé `reconciliation-cancel-confirm-supplier-invoice`
  (fr-CH `:881`, « la facture redeviendra ouverte, à payer » ; de-CH, it-CH, en-CH `:833`), son repli
  `features/reconciliation/CancelReconciliationDialog.svelte:91` (commentaire `:87`) et ses tests
  (`CancelReconciliationDialog.test.ts:161`, `tests/e2e/reconciliation-supplier.spec.ts:295`) ; le repli
  `features/reconciliation/failed-proposal-label.ts:340` et son test `:345` (commentaire `reconciliation.types.ts:52`) ; le
  repli `features/payment-batches/payment-batch-helpers.ts:91` (« Facture non ouverte ») et son test
  `payment-batch-helpers.test.ts:64` ; les E2E `tests/e2e/supplier-invoices.spec.ts:151`, `:209` (`/Ouverte/i`) ;
  *(validation P2, F6)* les commentaires et titres `payment-batches/+page.svelte:89`, `payment-batch-helpers.ts:62`,
  `[id]/supplier-settlement-page.test.ts:152`, `:305`, `:463`, `tests/e2e/supplier-invoices.spec.ts:135`,
  `tests/e2e/reconciliation-supplier.spec.ts:10`, `tests/e2e/helpers/api-fixtures.ts:243`, et `README.md:43`. Chacun
  passe à « validée » (ou « non validée ») aux quatre catalogues, replis et tests compris ; les commentaires de code
  (`supplier-invoices.api.ts:35`, `:52`, `:62` ; `[id]/+page.svelte:215`, `:639`) suivent ; chaque occurrence triée au Dev
  Agent Record. « Ouvert » au sens du **poste ouvert** d'un compte (la dette), d'un exercice ouvert ou d'une ligne ouverte
  reste. Les valeurs et codes (`open`,
  `SUPPLIER_INVOICE_NOT_OPEN`) ne changent pas. Mutation **M31** : le repli redevient « Ouverte » → le banc des
  helpers rougit.
  **#831** : la colonne TTC affiche `formatExactAmount(inv.totalAmount)` (`invoice-helpers.ts:92`) au lieu de
  `formatSupplierInvoiceTotal` (`supplier-invoices/+page.svelte:93`) — `100.0040` s'affiche `100.0040`, `100.00`
  s'affiche `100.00`. **C-v017-4-14** : case « Afficher les factures annulées », décochée par défaut ; la liste appelle
  `?status=draft,open,paid`, puis sans filtre une fois cochée. **Banc de la page de liste** neuf
  (`frontend/src/routes/(app)/supplier-invoices/supplier-invoices-list-page.test.ts`) : rendu d'un brouillon (badge),
  d'un total sous le centime, appel filtré par défaut, appel non filtré après la case.
- **AC-22 — Fiche, actions par statut.** Brouillon : *Modifier* (lien vers l'édition), *Valider* (confirmation : « L'écriture
  d'achat sera passée au journal »), *Supprimer le brouillon* (Administrateur seul, confirmation) ; **ni** bloc *Payer*
  (`[id]/+page.svelte:514`) **ni** bloc *Annuler la facture* (`:642`) ; les gestes du document (joindre, détacher)
  restent. Validée (`open`) : *Dévalider* (Administrateur et Comptable, confirmation : l'écriture d'achat sera **supprimée**, la
  numérotation gardera un trou, la facture redeviendra modifiable), plus *Payer* et *Annuler* comme aujourd'hui. Payée,
  annulée : inchangées. Après chaque geste, la fiche affiche la facture rendue par le serveur et une notification de
  succès. Banc de la fiche : chaque statut × chaque rôle (Administrateur, Comptable, Consultation) — boutons présents ou
  absents ; le bloc *Annuler la facture* est absent d'un brouillon (mutation M21-bis, validation P1, F12).
  *(Validation P2, R14.)* Un lien « Voir l'écriture d'achat » (`/journal-entries/{purchaseJournalEntryId}`) s'affiche
  pour une facture validée, payée ou annulée, et jamais pour un brouillon (la fiche n'a aujourd'hui que le lien de
  règlement, `[id]/+page.svelte:585-588`). Les refus de
  la validation et de la suppression s'affichent sous leur bouton par le `message` du serveur (`err.message`), patron de
  la fiche client (`invoices/[id]/+page.svelte:403-413`) : aucun second jeu de textes.
- **AC-23 — Page d'édition.** `/supplier-invoices/[id]/edit` : charge la facture ; si elle n'est pas brouillon, redirige
  vers la fiche avec un message. Rend `SupplierInvoiceForm` **pré-rempli** (fournisseur, numéro, dates, coordonnées,
  projet, lignes) — nouvelle propriété `initial` ; `onSubmit` appelle `PUT` avec la `version` lue. *(Réécrit à la
  validation P1, F5, R5.)* Le mode « document avec QR » (coordonnées en lecture seule, total comparé au montant du QR)
  ne s'applique que si `sourceDocument.linkOrigin === 'completion'` et `sourceDocument.qr !== null` ; le formulaire lit
  les données du QR dans `sourceDocument.qr` de la facture (4a, § *Contrat*) — aucune lecture de la liste des documents.
  Document joint après coup ou lien antérieur (`'attachment'`, `null`) : saisie libre. Bancs : document né de la
  complétion avec QR, sans QR, joint après coup. *(Validation P2, F8.)* `sourceDocument.qr` porte exactement les champs
  que le formulaire lit d'un `ImportedSupplierInvoice` (`SupplierInvoiceForm.svelte:66-67`, `:118-124`, `:409`) :
  `creditorIban`, `isQrIban`, `referenceType`, `referenceValue`, `amount`, `currency`, `creditorName`,
  `creditorAddressType`, `creditorLine1`, `creditorLine2`, `creditorPostalCode`, `creditorTown`, `creditorCountry` ; la
  page d'édition les passe au formulaire, dont la prop `document` devient *(validation P3, F8)* un
  `Pick<ImportedSupplierInvoice, …>` de **ces treize champs** — les seuls qu'il lit (`SupplierInvoiceForm.svelte:85`,
  `:87`, `:118-124`, `:318-343`, `:409`) — : la page de complétion lui passe le document entier (qui les contient),
  l'édition l'objet `qr` ; aucun champ inventé (`storagePath`, `fileHash`). À l'édition, `initial.invoiceDate`
  **prime** sur la règle « date vide avec un document » de l'AC-24 (b). Succès → fiche ; refus → saisie
  intacte, message à sa place (patron AC16 de la 30-2b) ; `OPTIMISTIC_LOCK_CONFLICT` → message qui invite à recharger.
- **AC-24 — Le formulaire.** (a) bouton « Enregistrer le brouillon » à la création, « Enregistrer » à l'édition ;
  (b) **#853** : avec un `document`, `fInvoiceDate` démarre **vide**, le champ est obligatoire et une aide dit « Date de
  la facture, telle qu'imprimée sur le document » ; sans document, la date du jour reste proposée ; une échéance
  antérieure à la date est signalée **sous le champ** et bloque l'envoi (message qui dit les deux dates) ; le refus
  serveur `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE` s'affiche au même endroit (`describeSubmitError`) ; *(validation P1,
  F8)* une date vide a son **propre** message (clé neuve `supplier-invoices-err-invoice-date`, « Saisissez la date de la
  facture, telle qu'imprimée sur le document. », quatre catalogues), et non celui des lignes (`SupplierInvoiceForm.svelte:208-213`) ;
  (c) **#830** : la quantité, le prix **et le montant attendu** (`paymentFields`, `expectedPaymentAmount`, validation P1,
  F4) sont **lus une fois** par `parseAmount` et envoyés avec un point décimal (« 12,50 » → `"12.50"`) ; le taux, choisi
  dans une liste, n'a pas de virgule ; `sumTtc`, `amountMismatch` et `lineNotInCentimes` lisent les mêmes valeurs ; une
  quantité vide vaut `1` partout (le total vivant comme l'envoi) ; *(corrigé à la validation P2, F2 ; C-v017-4-36)*
  `linesInvalid` **admet** aujourd'hui un prix vide (`isValidAmount` à `allowEmpty: true` par défaut, `balance.ts`) et
  seul le serveur le refuse : le formulaire le **refusera** désormais, par `isValidAmount(l.unitPrice, { allowEmpty:
  false })` au site de `linesInvalid` (`supplier-invoice-form.ts:98`) — mécanique ; banc « prix vide → message des
  lignes, aucun envoi », qui tue M33 ; un prix vide n'est donc jamais lu `0` (`parseAmount('')` rend `0`) ; un montant
  attendu vide reste `null` ; le commentaire « ⚠️ La virgule n'est pas normalisée à l'envoi : #830 »
  (`supplier-invoice-form.ts`) est retiré ; (d) **#835** : le total vivant affiche `formatExactAmount`
  (`SupplierInvoiceForm.svelte:591`), si bien qu'une ligne à 99.9999 se lit `99.9999` sous son message de refus, et le
  montant du QR (`fmt`, `:189-196`) de même ; (e) *(validation P1, F13)* après la complétion d'un document, la
  notification dit « Brouillon enregistré. » (`supplier-invoices/new/+page.svelte:143`, clé
  `imported-supplier-invoices-completed`, quatre catalogues ; banc de la page). Bancs : `SupplierInvoiceForm.test.ts`,
  `supplier-invoice-form.test.ts`.
- **AC-25 — Messages des refus.** Une fonction pure `supplierInvoiceUnvalidateMessage(code, label)` — la signature du
  patron `supplierInvoiceCancelMessage(code, label)` (`invoice-cancel.ts:32-35`, corrigé à la validation P1, F12) — rend
  un message par code de la dévalidation (§ *Précédence*) et un repli générique. `SUPPLIER_INVOICE_DRAFT` **n'est pas**
  ajouté à `supplierInvoiceCancelMessage` : la fiche n'affiche pas le bloc d'annulation d'un brouillon (AC-22), le
  message serait mort (F12) ; *(validation P2, F9)* mais la valeur entre à l'union `SupplierInvoiceCancelCode`
  (`invoice-cancel.ts`, « l'union ne porte que ce qu'il **peut** rendre ») — le serveur la rend (AC-5 (7)). *(Tranché au
  code à la validation P3, F2 ; C-v017-4-40 — remplace « rendue par le repli générique », qui n'existe pas : le
  `default` de `supplierInvoiceCancelMessage` est une garde d'exhaustivité `never`, `invoice-cancel.ts:74-77`.)* Le
  serveur rend bien la valeur sur ce chemin (`GET` de la fiche d'un brouillon) : l'union la porte, **avec** un `case
  "SUPPLIER_INVOICE_DRAFT"` et son texte (clé neuve `supplier-invoices-cancel-blocked-draft`, « Un brouillon ne s'annule
  pas : supprimez-le, ou validez-le d'abord. », quatre catalogues) ; l'AC-22 ne l'affiche pas, le type reste exhaustif
  et vrai. Tests unitaires : chaque code.
- **AC-26 — Écran de clôture.** `closing-checks-labels.ts`, `FiscalYearChecks.svelte` et leurs types : K8, libellé,
  liste avec liens vers les fiches ; fixtures et bancs (`FiscalYearChecks.test.ts`, `closing-checks.test-fixtures.ts`,
  et *(validation P3, R10 = F6)* `routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts:460`, `toHaveLength(7)` → 8)
  à huit contrôles ; la page de clôture (`+page.svelte:675-705`) passe par `checkTitle`, sans code en dur — vérifié.
- **AC-27 — Lots.** L'écran des lots ne propose jamais un brouillon (`payment-batches/+page.svelte:53`, `:91`) : banc qui
  le prouve. *(Validation P1, F14.)* Il demande `listSupplierInvoices({ limit: 200, status: 'open' })` (filtre serveur de
  l'AC-15) au lieu de filtrer côté client une fenêtre où brouillons et annulées prendraient la place de factures validées ;
  le filtre client reste en défense.
- **AC-28 — i18n.** Toutes les clés neuves de l'écran aux quatre catalogues ; `npm run lint-i18n-ownership` vert ; les
  gardes i18n (moissonneur, allowlist, décomptes figés) recomptées. Libellés : glossaire. *(Validation P1, F11.)*
  `docs/i18n-glossaire.md` n'a d'entrée que pour « brouillon » (`:92`) et « ouverte (facture) » (`:93`) : y **ajouter**
  « valider », « dévalider » et « validée (facture fournisseur) », aux quatre langues, sur les clés client existantes
  (`invoice-validate-button`, `invoice-unvalidate-button`, `invoice-status-validated`) ; l'entrée « ouverte » dit qu'elle
  ne nomme plus l'état d'une facture fournisseur.
- **AC-29 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, et son PDF régénéré au même commit, contrôlé
  aplati). Sites, à compléter par `grep -n -i 'enregistr\|ouverte\|annuler-facture-fournisseur\|ressaisi' ` sur la §
  `sec:factures-fournisseurs` (`:1565`) et les § qui la citent : (1) § *Saisir une facture fournisseur* (`:1573-1641`) —
  l'enregistrement crée un **brouillon**, sans écriture ; (2) une § neuve « Le brouillon : modifier, valider,
  supprimer » et « Dévalider une facture validée » (limite L8 : un brouillon à historique de lot ne se supprime pas, et
  son chemin ; empêchements et levée de chacun ; trou de numérotation ; payée : annuler
  le règlement d'abord) ; (3) § *Une facture fournisseur au total sous le centime* (`:1714-1759`) : « ouverte :
  annulez-la puis ressaisissez » devient « dévalidez-la, corrigez la ligne, revalidez » ; payée : annuler le règlement,
  dévalider ; (4) § *Annuler une facture fournisseur* (`:1761-1795`) : quand annuler (la facture n'était pas due) et
  quand dévalider (une faute de saisie) ; « Une facture annulée ne se désannule pas : saisissez-la de nouveau » relu ;
  les factures annulées masquées dans la liste, la case « Afficher les factures annulées », et *(validation P1, F9)* la
  phrase qui dit que leur écriture d'achat et sa contre-passation **restent au journal** — la 0.17.0 ne les supprime pas
  (limite L6) ; (5) § *Numérotation* (`:740-755`) : « Deux chemins en creusent » →
  trois, la dévalidation d'une facture fournisseur ; (6) réconciliation d'un débit (`:2480`) et lots
  (`sec:paiements-fournisseurs`) : seules les factures **validées** ; (7) documents à traiter
  (`sec:import-factures`) : la complétion crée un brouillon ; date de facture à saisir ; (8) contrôles de clôture : K8 ;
  (9) #853 : l'échéance ne peut précéder la date ; (10) la virgule décimale admise ; (11) **décision Q4
  (C-v017-4-24)** : l'état s'appelle **« validée »** — chaque « ouverte » qui désigne l'état d'une facture fournisseur
  passe à « validée » (à `8e126cfd` : `:1645`, `:1646`, `:1660`, `:1695`, `:1727`, `:1734`, `:1765`, `:1828`, `:2480`,
  et ce que rend `grep -n -i 'ouverte' docs/manual/fr/user-manual.tex` dans les § fournisseur, lots, réconciliation et
  règles d'affectation) ; « la dette redevient ouverte » (`:1686`, le poste ouvert) reste. *(Remplacé à la validation P1,
  F6.)* Le contrôle du PDF aplati est un **tri de chaque** « ouverte » / « ouvertes » des § fournisseur, lots,
  réconciliation, règles d'affectation, clôture et dépannage, et non deux motifs : formes relevées au PDF à `8e126cfd`
  — « facture fournisseur ouverte » (6), « factures fournisseur ouvertes » (1), « facture ouverte » (3), « redevient
  ouverte » (5), « ouverte, à payer » (2), « elle est ouverte » (2), *(complété à la validation P2, F7)* « factures
  ouvertes » (2), « n’est plus ouverte » (1, `:2505`) —, chacune recomptée après la correction, et la liste
  des occurrences **gardées** (poste ouvert `:1686`, `:1780`, exercice ouvert, ligne ouverte) écrite au Dev Agent Record.
  *(Validation P1, F1 ; C-v017-4-32.)* Phrases dont le **sens** change, à réécrire et non à renommer :
  (12) le parcours de Guy — « saisir après coup les factures des paiements du relevé » — s'écrit « **saisissez, puis
  validez** la facture » : `:1646` (« saisissez la facture ouverte, puis, dans Réconciliation, acceptez-la »), `:2480`
  (« saisir la facture ouverte, puis laisser la réconciliation la régler »), `:3541` (« saisissez la facture fournisseur
  ouverte, sans la payer depuis sa fiche ») ; (13) `:1721` « La liste des factures, elle, arrondit encore ce total à deux
  décimales » — faux après #831 : la liste l'affiche aux décimales exactes ; (14) `:1639` (« la saisie est refusée avec
  un message qui nomme le compte ») et `:1914` (refus de la complétion « compte créanciers ou TVA récupérable non
  imputable, exercice fermé ») : ces refus passent à la **validation** ; (15) `:1945` « Tout statut de facture est
  accepté --- ouverte, payée ou annulée » (joindre un document) : brouillon compris ; (16) `:1957-1958` « Si c'est la
  facture elle-même qui est fausse, annulez-la d'abord, puis traitez le document » : un brouillon se **supprime** (le
  document redevient « à traiter »), une facture validée se dévalide puis se supprime ; (17) `:3586` « Facture
  fournisseur : elle s'annule depuis sa fiche, ouverte ou payée » : brouillon (supprimer) et dévalidation ajoutés ;
  (18) *(F10)* « sept contrôles préalables » (`:798`) et leur liste (`:803-811`) : huit, K8 ajouté ;
  (19) *(validation P2, F4)* dans la § neuve du brouillon : quelles coordonnées restent **verrouillées** à l'édition —
  celles d'une facture **née** d'un document avec QR, dont le total reste comparé au montant du QR —, et lesquelles se
  modifient — document joint après coup, document lié avant la 0.17.0 (limite L7, en une phrase) ; les phrases
  « affichés en lecture seule … ne se modifient pas » (`:1885`) et « Kesh ne compare pas les montants et ne modifie
  rien de la facture » (§ *Joindre un document*) relues en ce sens ; (20) *(F5)* `:670-672` (« Une écriture disparaît
  donc par deux voies … la dévalidation d'une facture … voir « Supprimer ou dévalider une facture » ») : la
  dévalidation d'une facture fournisseur s'y ajoute, avec son renvoi ; (21) *(F7)* le tri `.tex` couvre les **mêmes**
  § que le contrôle du PDF (fournisseur, lots, réconciliation, règles d'affectation, clôture, dépannage), dont `:3532`
  (« une facture fournisseur ouverte dont le total… ») et `:3592` (« qui redevient ouverte »). Même tri dans la prose de
  `docs/api-external.md` (« ouverte ou payée » → « validée ou payée » ; statut `open` et codes inchangés), au manuel
  d'administration et dans le texte **neuf** du CHANGELOG `[0.17.0]` (les entrées publiées ne se réécrivent pas). Tout passage qui affirme que
  l'enregistrement **passe l'écriture** est réécrit (`:1583-1584` ; la phrase est coupée en fin de ligne). *(Validation P1,
  F7.)* Au PDF aplati, l'apostrophe est **typographique** (’) : greper `passe l’écriture d’achat` **et** `passe
  l'écriture d'achat`, ou le motif sans apostrophe `Kesh passe l` — un motif à apostrophe ASCII seule rend 0 avant comme
  après la correction. *(F10.)* La brochure (`docs/manual/fr/marketing-brochure.tex:437`, « Sept contrôles », et `:448`,
  « la facture fournisseur ouverte ») suit, PDF régénéré.
- **AC-30 — E2E.** *(Écrit en deux temps, validation P3, R8 = F7 ; C-v017-4-31.)* **La 4a livre** le helper
  `createSupplierInvoiceViaApi` qui valide, le helper du relevé saisi par l'API et la spec neuve réduite à ses étapes
  par l'API (tâche de la 4a) ; **la 4b** y ajoute les étapes par l'écran et adapte les autres specs. Une spec neuve `frontend/tests/e2e/supplier-invoice-draft-cycle.spec.ts` : fournisseur et comptes
  synthétiques ; **brouillon** par le formulaire avec un compte de charge faux → fiche « Brouillon », aucune écriture
  (lien d'écriture absent) ; *Modifier* → bon compte → *Enregistrer* ; *Valider* → « Validée », lien vers l'écriture ;
  *Dévalider* → « Brouillon » ; *Modifier* le prix (synthétique, 100.20 → 100.00) ; *Valider* ; un relevé **saisi à la
  main** par l'API (`POST /bank-imports/manual`) porte un débit égal ; à la réconciliation, la facture est proposée pour
  ce débit, et *Accepter* la rend « Payée ». *(Validation P1, F3.)* Aucun helper n'existe pour `POST /bank-imports/manual`
  (`grep -n "bank-imports/manual" frontend/tests/e2e/helpers/*.ts` : rien) : en écrire un (`helpers/api-fixtures.ts`,
  relevé **équilibré au centime** : solde de départ, un débit, solde d'arrivée), ou réutiliser `decor()` de
  `reconciliation-supplier.spec.ts:93-130` (relevé CAMT.053) — le choix au Dev Agent Record. Et : un brouillon au montant d'un débit **n'est pas** proposé ; un brouillon
  se supprime (Administrateur). Specs existantes adaptées pour valider après créer : `supplier-invoices.spec.ts:108`,
  `:151`, `:209` (`/Ouverte/i` → « Validée »), `payment-batches.spec.ts:102`, `supplier-documents.spec.ts:360` (lit
  `purchaseJournalEntryId` après un enregistrement par l'interface, puis l'ouvre `:406`, `:423`) et `:491`,
  `reconciliation-supplier.spec.ts:295` (« redeviendra ouverte, à payer »), et le helper
  **`createSupplierInvoiceViaApi`** (`helpers/api-fixtures.ts:249`, création `:289`), qui **valide** après la création —
  dont dépend toute `reconciliation-supplier.spec.ts` (`:259`, `:301`) ; `supplier-documents.spec.ts:463`, `:488`,
  `:513` (lisent `purchaseJournalEntryId`, validation P2, R14). *(Validation P2, F3.)* La date de facture, vide par
  défaut avec un document (AC-24 (b)), casse les specs qui créent la facture d'un document par le formulaire :
  `inbox-import.spec.ts:101-105`, `supplier-documents.spec.ts:213`, `:249`, `:302`, `:353` — la date se saisit dans le
  helper `fillFirstInvoiceLine` (`helpers/inbox.ts:228-238`) ou un helper voisin, non spec par spec. Critère de
  recherche : toute spec qui enregistre par l'interface ou l'API puis **paie, annule ou lit l'écriture d'achat**, **et
  toute spec qui crée la facture d'un document par le formulaire** (liste au Dev Agent Record). La suite complète tourne au dernier commit de code
  de la 4b (règle « Test Locally First »), échecs jugés fichier par fichier contre `docs/testing.md`.
- **AC-31 — Captures** (thème clair, base de démonstration, hors dépôt, `kesh-gate-logs/v0174/captures/`) : liste avec
  un brouillon et la case des annulées (cochée, décochée) ; fiche d'un brouillon (Administrateur, Comptable) ; page
  d'édition, avec et sans document à QR ; confirmations *Valider*, *Dévalider*, *Supprimer* ; fiche ouverte avec
  *Dévalider* ; un refus de dévalidation affiché ; le formulaire avec l'échéance refusée, la date vide sur un document, le
  total vivant à quatre décimales ; la liste avec un total sous le centime ; l'écran de clôture avec K8.
- **AC-32 — CHANGELOG et README.** `[0.17.0] — Non publié` : *Ajouté* — le cycle brouillon des factures fournisseur
  (#849), les factures annulées masquées dans la liste — leur écriture d'achat et sa contre-passation restent au journal
  (L6) —, le contrôle de clôture des brouillons fournisseur ; *Corrigé* —
  #853, #830, #835, #831, chacun en une phrase. `README.md:42` (*Factures fournisseurs & règlement*) : brouillon modifiable,
  validation, dévalidation ; *(validation P3, F5)* `README.md:47` (« sept contrôles en lecture seule ») → huit, `:231`
  (historique) laissé.

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
| M7 | dévalidation : comparaison précoce de la version retirée | AC-11 — *(validation P2, R10)* la paire « version périmée **et** lot `generated` » rend `OPTIMISTIC_LOCK_CONFLICT` (précédence) ; le mutant rend `SUPPLIER_INVOICE_IN_PAYMENT_BATCH` (l'`UPDATE … version = ?` seul rendrait le même code sans la paire) |
| M8 | `PUT` admis sur une facture validée | AC-8 — assertion sur le **code** `ILLEGAL_STATE_TRANSITION` (R4) |
| M9 | `DELETE` sans `detach_in_tx` | AC-9 (document `to_complete`) |
| M10 | `DELETE` sans la garde d'historique de lot | AC-9 (code au lieu de l'erreur de clé) |
| M11 | la complétion appelle `create_validated_in_tx` | AC-7 |
| M12a | `reconciliation.rs:825` : `status IN ('draft','open')` | AC-5 (1) *(scindée à la validation P1, R11)* |
| ~~M12b~~ | *retirée à la validation P2, R4* : `reconciliation.rs:885` ne lit que les factures d'un lot `generated`, où un brouillon n'entre jamais (création de lot et dévalidation refusées) — **mutant équivalent** | — |
| M12c | `reconciliation.rs:935` (masquage des règles) : `status IN ('draft','open')` | AC-5 (2) réécrit (validation P2, R4) : un brouillon du même fournisseur au montant voisin ne masque pas la règle — avec le mutant, la règle cesse d'être proposée |
| M13 | `PUT` : règles « document » sautées | AC-8 (`QR_FIELDS_FROM_DOCUMENT`, `AMOUNT_MISMATCH`) |
| M14 | échéance antérieure admise | AC-14 |
| M15 | `CHECK` « écriture si et seulement si non brouillon » retiré de la migration | AC-1 |
| M16 | K8 compte les factures ouvertes | AC-16 |
| M17 | filtre `status` de la liste ignoré | AC-15 |
| M18 | `cancel` d'un brouillon admis (tête retirée) | AC-13 |
| M19 | audit `supplier_invoice.unvalidated` retiré | AC-11 |
| M20 | `validate_in_tx` sans le verrou de la facture en premier | AC-10 — test **piloté** (validation P3, R1) : le perdant reçoit le **code** `ILLEGAL_STATE_TRANSITION` (le mutant rend `OPTIMISTIC_LOCK_CONFLICT`) ; « une seule écriture » ne suffit pas (R4) |
| M20-bis | `total_amount` du brouillon calculé hors de `purchase_invoice_totals` (somme HT seule) | AC-6 (a) *(validation P1, R2)* |
| M20-ter | `PUT` : règles de la complétion appliquées à un document joint après coup | AC-8, cas de Guy *(R5)* |
| M20-quater | *(inversée à la validation P2, R1 = F1)* `DELETE` : `detach_in_tx` (le document) **avant** la facture | AC-9 — test **piloté** (validation P3, R1 = F3) : attache rejouée à la main jusqu'à la sentinelle, `DELETE` bloqué sur la facture, puis `mark_completed` — le mutant interbloque à coup sûr, l'ordre prescrit non |
| M21 | fiche : bloc *Payer* affiché pour un brouillon | AC-22 |
| M21-bis | fiche : bloc *Annuler la facture* affiché pour un brouillon | AC-22 *(F12)* |
| M22 | fiche : *Supprimer* offert au Comptable | AC-22 |
| M23 | formulaire : date du jour avec un document | AC-24 (b) |
| M24 | formulaire : échéance antérieure envoyée | AC-24 (b) |
| M25 | formulaire : `unitPrice` brut envoyé | AC-24 (c) |
| M26 | total vivant en `formatSwissAmount` | AC-24 (d) |
| M27 | liste : `formatSupplierInvoiceTotal` | AC-21 |
| M28 | liste : annulées affichées par défaut | AC-21 |
| M29 | `supplierInvoiceUnvalidateMessage` : un code rend le repli | AC-25 |
| M30 | lots : filtre `open` retiré de l'écran | AC-27 |
| M31 | repli du libellé `open` redevenu « Ouverte » | AC-21 (ajoutée par la décision Q4) |
| M32 | formulaire : montant attendu envoyé brut | AC-24 (c) *(F4)* |
| M33 | formulaire : prix vide lu `0` | AC-24 (c) *(F4)* |
| M34 | édition : mode QR appliqué à un document joint après coup | AC-23 *(F5)* |

## Ce que les gardes ne voient pas (limites)

- **L1** — rc2 → rc1 (C-v017-4-3) : même version Cargo ; retour par la sauvegarde d'avant la mise à jour seulement.
  Écrite aux **notes de la candidate** seulement (C-v017-4-33). La colonne `link_origin` rend désormais le refus net : une
  sauvegarde de la rc2 porte une colonne que la rc1 ne connaît pas (`IMPORT_SCHEMA_MISMATCH`).
- **L2** — une écriture d'achat supprimée par la dévalidation creuse un trou de numérotation (comme le client) ; le
  journal d'audit la garde.
- **L3** — un brouillon n'est pas compté aux rapports (aucune écriture) : seule la clôture le signale (K8), et la
  liste par son badge.
- **L4** — la dévalidation d'une facture qui a l'historique d'un lot **confirmé** n'est pas refusée (le règlement a été
  annulé explicitement ; la fiche montre ce lot) : la banque a pu exécuter l'ordre — si le montant change, le manuel dit
  de vérifier le relevé.
- **L5** — l'avis « date de facture postérieure au dépôt de plus de quelques jours » (#853, point 3) n'est pas fait :
  il part dans une issue neuve, ouverte par l'orchestrateur (C-v017-4-25).
- **L8** — *(validation P3, R9 ; C-v017-4-40)* un brouillon qui garde l'**historique d'un lot** (validé, mis en lot,
  réglé, règlement annulé, dévalidé) **n'est pas supprimable** (`SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY` ;
  `payment_batch_items` en `RESTRICT`, `20260628000002_payment_batches.sql:54`) : l'historique de l'ordre transmis à la
  banque se garde. Son chemin : le corriger et le valider, ou, s'il n'est pas dû, le valider puis l'**annuler** ; K8 le
  compte tant qu'il reste brouillon. Le manuel le dit (§ neuve du brouillon, AC-29 (2)).
- **L7** — *(validation P1, R5)* un document lié avant la 0.17.0 (`link_origin` nul) n'impose rien au `PUT` : les
  coordonnées d'une facture née d'un QR avant la 0.17.0 redeviennent modifiables une fois dévalidée. Aucun backfill (on ne
  sait pas, en base, si le lien vient d'une complétion ou d'un rattachement).
- **L6** — les factures annulées restent dans les livres, avec leur écriture d'achat et sa contre-passation : la 0.17.0
  les masque dans la liste, elle ne les supprime pas (C-v017-4-21). La suppression réelle sera soumise au Project Lead à
  part ; le manuel utilisateur le dit (§ *Annuler une facture fournisseur*).

## Ce que la release doit contenir (orchestrateur ; ne pas prescrire au développeur)

1. Candidate `v0.17.0-rc2` (décidée, C-v017-4-22) ; notes : deux migrations, la limite L1 **écrite en toutes lettres**, le changement de contrat de
   `POST /supplier-invoices`.
2. **Avant la mise à jour du NAS** : sauvegarde complète (base, `.keshbackup`, documents) ; aucune recette ne s'écrit
   dans la base de Guy (`v017-donnees-a-garder.md`, point 3) — la recette du cycle se joue sur une installation jetable.
3. PR de release : `closes #849`, `closes #830`, `closes #835`, `closes #831` ; **`closes #853`** (décision Q5,
   C-v017-4-25 : son point 3 part dans une issue neuve).
4. Issue à ouvrir : #853 point 3 (avis de date). Commenter #849 (prémisse : la règle client est la suppression ;
   statut `open` gardé) et #830 (remède : parser une fois, non une substitution de chaîne).

## Dev Notes (communes aux deux sous-fiches)

### Le manuel (axe de revue)

Toute passe de revue de cette story vérifie les manuels FR **au PDF aplati** contre le code : « enregistrer passe
l'écriture » ne doit plus se lire nulle part ; « ouverte » ne désigne plus l'état validé d'une facture fournisseur
(décision Q4) — *(corrigé à la validation P1, F6 ; première rédaction : « ne doit jamais désigner un brouillon »)* ; au
PDF, l'apostrophe est typographique (F7).

### Propagation

Après chaque correctif, greper la **valeur** : `\bopen\b` et `'open'` dans les fichiers de l'inventaire ; `sept`/`7`
près de `checks` ; `87`/`88` et `53`/`54` (AC-1) ; `22`/`23` (`post_restore.rs`) ; « une migration » (CHANGELOG, admin
manual) ; « six stories » ; *(validation P1)* « ouverte »/« ouvertes »/« offen »/« aperta » (par la valeur, tout le
dépôt, F2) ; « Facture créée » (F13) ; *(validation P3, F5)* « sept contrôles » en prose (`grep -rn "sept contr"`).

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

*Tranchées le 2026-10-31 par l'orchestrateur, dans le mandat autonome du Project Lead (C-v017-4-21 à 25) : Q1 (B) ;
Q2 oui, L1 aux notes de la candidate ; Q3 non ; Q4 « Validée » à l'écran, quatre locales, valeur stockée `open` ; Q5
#853 fermée avec la release, point 3 en issue neuve. Le texte des questions reste tel que posé.*

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
- 2026-10-31 — **Décisions sur Q1 à Q5** (orchestrateur, mandat autonome du Project Lead), consignées en C-v017-4-21 à
  25 : (B) livrée, (A) hors 0.17.0 (L6) ; candidate rc2, L1 aux notes ; pas de « Enregistrer et valider » ; libellé
  « Validée » aux quatre locales, inventaire des « ouverte » de l'écran et des textes (AC-21, AC-29), M31 ajoutée ;
  `closes #853`, point 3 en issue neuve (L5). Touchés : en-tête, § *Origine*, table des décisions, § *Les factures
  annulées*, AC-21, AC-22, AC-29, AC-30, table des mutations et du découpage, limites L5 et L6, § *Ce que la release doit
  contenir*, § *Questions*. Prochaine étape inchangée : validation P1.
- 2026-10-31 — **Validation P1** (Sonnet, lentilles R et F, contexte frais ; rapports `kesh-gate-logs/v0174-validate-p1-{R,F}.md`) :
  **0 CRITICAL, 0 HIGH, 12 MEDIUM, 16 LOW** (R : 6 MEDIUM, 5 LOW ; F : 6 MEDIUM, 11 LOW). Prémisses vérifiées au code avant
  chaque remède (lignes lues : `closing_checks.rs:3`, `:30-36`, `:522` ; `closing_checks_e2e.rs:258` ;
  `kesh-db/tests/closing_checks.rs:322` ; `rejeu_interblocage_e2e.rs:865` ; `admin_backup/import.rs:579-619` ;
  `supplier-documents.spec.ts:360` ; `helpers/api-fixtures.ts:249` ; `failed-proposal-label.ts:340` ;
  `payment-batch-helpers.ts:91` ; `CancelReconciliationDialog.svelte:91` ; `supplier-invoices.spec.ts:151` ;
  `reconciliation-supplier.spec.ts:295` ; `supplier-invoices/new/+page.svelte:143` ; `user-manual.tex:1639`, `:1721`, `:1914`,
  `:1945`, `:1957-1958`, `:3541`, `:3586` ; `admin-manual.tex:1826`, `:3580` ; `marketing-brochure.tex:437`). Écarts de
  numérotation relevés et corrigés dans les citations (F cite `:1722`, `:1608-1612`, `:1962-1966`, `:1973`, `:3578` ;
  lignes réelles `:1721`, `:1639`, `:1914`, `:1957`, `:3580`). Aucun finding rejeté. Décisions de l'orchestrateur appliquées
  (C-v017-4-26 à 33) :
  - **MEDIUM** — R1 (K8 en dernier, `ALL_SQL`, T-a2, 7 → 8) ; R2 (`purchase_invoice_totals` en `kesh-core`, réutilisée
    par la génération ; test sans compte de TVA récupérable) ; R3 (§ *Verrous et rejeu* réécrit : régime par route,
    `DELETE` dans l'ordre de `attach_in_tx`, création reclassée, `rejeu_interblocage_e2e.rs:865`) ; R4 (M7, M8, M20 sur le
    code) ; R5 (règles de la complétion au `PUT` seulement pour un document né de la complétion — colonne
    `link_origin` ; cas de Guy testé ; L7) ; R6 (dérogation écrite : publication commune, E2E complet au dernier commit
    de la 4b) ; F1, F2, F6 (inventaire et phrases du manuel par la valeur sur tout le dépôt, « saisissez, puis
    validez », tri de chaque « ouverte » du PDF) ; F3 (specs et helper nommés) ; F4 (montant attendu, prix vide refusé) ;
    F5 (`sourceDocument` enrichi par la 4a, consommé par la 4b).
  - **LOW, tous appliqués** — R7 (inventaire complété), R8 (comparaison tautologique retirée), R9 (règle Q4 aux AC-18,
    AC-19, AC-20), R10 (repli du `MODIFY COLUMN` dans l'AC, deux applications), R11 (M12a-c), F7 (apostrophe), F8 (message
    de la date), F9 (L6 au manuel et au CHANGELOG), F10 (« sept » : manuels et brochure), F11 (glossaire), F12 (signature,
    chemin mort, `err.message`), F13 (« Brouillon enregistré »), F14 (lots filtrés au serveur), F15 (rien à créer au manuel
    d'administration pour MCP), F16 (`api-external.md:823`, `:827`), F17 (L1 aux notes de la candidate seulement).
  Mutations : M12 scindée en M12a-c ; ajoutées M20-bis, M20-ter, M20-quater, M21-bis, M32, M33, M34. Prochaine étape :
  validation P2 (`v017-4-validate-prompt-p2.md`, Opus, passe complète, lentilles R et F).
- 2026-10-31 — **Validation P2** (Opus, lentilles R et F, contexte frais ; rapports `kesh-gate-logs/v0174-validate-p2-{R,F}.md`) :
  **0 CRITICAL, 1 HIGH, 9 MEDIUM, 14 LOW** (R : 1 HIGH, 4 MEDIUM, 9 LOW ; F : 5 MEDIUM, 5 LOW ; R1 = F1). Prémisses lues au
  code avant chaque remède : `imported_supplier_invoices.rs:305-314` (doc-comment de `detach_in_tx`), `:289` ;
  `routes/reconciliation.rs:3019-3024` ; `20261001000004_invoice_settlements_write_off.sql:17-19` ;
  `admin_backup/import.rs:579-619` ; `invoice-cancel.ts:5-12` ; `admin-manual.tex:2949`, `:2996` ; `user-manual.tex:1885`,
  `:3532`, `:3592` ; `README.md:43` ; `frontend/tests/e2e/inbox-import.spec.ts:101`, `supplier-documents.spec.ts:213-353`.
  Aucun finding rejeté. Arbitrages de l'orchestrateur appliqués (C-v017-4-34 à 36) :
  - **HIGH / MEDIUM** — R1 = F1 (`DELETE` : la facture puis `detach_in_tx`, l'ordre que le doc-comment déclare sûr ;
    M20-quater inversée ; C-v017-4-28 annoté) ; R2 (la sauvegarde 0.16.x forgée sans `link_origin`) ; R3 (code réel
    `RECONCILIATION_INVOICE_NOT_ELIGIBLE` / `supplier_invoice_not_open`) ; R4 (M12b retirée, mutant équivalent ; M12c et
    AC-5 (2) réécrits) ; R5 (contrainte de statut sous un nom neuf, ajoutée avant le retrait de l'ancienne) ; F2 (prix
    vide refusé par le formulaire, `allowEmpty: false`) ; F3 (specs de complétion et helper `fillFirstInvoiceLine`) ; F4
    (règle du QR verrouillé selon `link_origin` et L7 au manuel) ; F5 (« deux voies », « neuf motifs », capacité d'une
    clé `read-write`).
  - **LOW, tous appliqués** — R6 (complétion reclassée, (ii) et (vi) recomptés), R7 (projet lu sans verrou, verrous
    partagés de l'insertion), R8 (`mark_completed` reçoit l'origine), R9 (`invoice_total_ttc`, `AMOUNT_MISMATCH`), R10
    (M7 par la paire), R11 (inventaire, `invoices.rs:1448`), R12 (`link_origin` à l'export), R13 (AC-3 (d), en-tête de la
    4a, commandes de l'AC-12), R14 (lien d'écriture d'achat, specs `:463`, `:488`, `:513`), F6 (restriction de
    C-v017-4-32 reprise à l'AC-21, sites nommés), F7 (périmètres alignés, formes complétées), F8 (champs de `qr`,
    précédence de la date), F9 (union du type), F10 (la 4a n'est plus « livrable seule » ; helper et étapes API de la
    spec neuve écrits par la 4a).
  R1 touchant une règle de verrous, la P3 est **complète** (`v017-4-validate-prompt-p3.md`, Sonnet, lentilles R et F, axe
  obligatoire sur cette remédiation).
- 2026-10-31 — **Validation P3** (Sonnet, lentilles R et F, contexte frais ; rapports `kesh-gate-logs/v0174-validate-p3-{R,F}.md`) :
  **0 CRITICAL, 0 HIGH, 6 MEDIUM, 14 LOW** (R : 3 MEDIUM, 8 LOW ; F : 3 MEDIUM, 6 LOW ; R1 = F3, R3 = F4, R10 = F6, R8 = F7).
  Prémisses lues au code avant chaque remède : `supplier_document_attachment.rs:17`, `:330`, `:437` ;
  `api-external.md:837`, `:839` ; `supplier_invoices.rs:828` ; `imported_supplier_invoices.rs:294`, `:410` ;
  `services/pieces.rs:85-92` ; `README.md:47` ; `invoice-cancel.ts:74-77`. Aucun finding rejeté. Arbitrages de
  l'orchestrateur appliqués (C-v017-4-37 à 40) :
  - **MEDIUM** — R1 = F3 (tests de M20-quater et M20 **pilotés** par `attendre_une_requete_en_cours`, attache rejouée à la
    main ; le protocole `NOWAIT` de F écarté : il exigeait `INNODB_TRX`, privilège interdit, § 6.1) ; R2 (`retry_on_deadlock`
    gardé sur la création, le `PUT` et aussi le `DELETE` ; le *Retenu* de C-v017-4-28 annoté) ; R3 = F4 (décodage tolérant
    de `payment_batches.rs:378-385`, `failed[]` `SUPPLIER_INVOICE_NOT_OPEN`) ; F1 (`api-external.md:837-839`) ; F2 (un
    `case "SUPPLIER_INVOICE_DRAFT"` et son texte : le serveur rend la valeur sur ce chemin).
  - **LOW, tous appliqués** — R4 (assertions du registre et `admin-manual.tex:2979`), R5 (repli au nom neuf, trois essais),
    R6 (outils de l'AC-3 (a)), R7 (citations `:424`, `:294`, `:89` ; appelant de test ; `PiecePublique` non exposé), R8 =
    F7 (artefacts E2E de la 4a en tâche et dans l'AC-30), R9 (limite L8, précédence justifiée), R10 = F6
    (`fiscal-years-page.test.ts:460`), R11 (`supplier_invoices.rs:828`), F5 (`README.md:47`), F8 (`Pick` de treize champs),
    F9 (phrases du CHANGELOG par migration).
  R2 et R3 changeant des règles d'exécution, la P4 est **complète** (`v017-4-validate-prompt-p4.md`, Opus, lentilles R et
  F, axe obligatoire sur cette remédiation).
