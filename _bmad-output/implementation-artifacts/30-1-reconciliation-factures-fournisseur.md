# Story 30.1 : La réconciliation règle les factures fournisseur

## Status

**ready-for-dev** — fiche créée le 2026-10-15 en autonomie (`bmad-create-story`, Opus 5.5), sur `15686d55` (kickoff de
l'Epic 30), branche `story/30-1`, worktree `/home/gcorbaz/devel/kesh-30-1`. **Découpée** (§ *Découpage*) en deux
sous-stories tenues dans cette fiche, comme la 29-1 et la 29-4 : **30-1a** (le dé-rapprochement sait défaire un
règlement fournisseur) **puis** 30-1b (proposer et accepter). Choix consignés au registre propre à la story,
`epic-30-choix-autonomes-301.md`, **C-30-1-1 à C-30-1-24** (à verser par union au registre de l'epic). Validation P1 à
lancer : `30-1-validate-prompt-p1.md`.

PR : **`closes #686`** dans le titre ou le corps de la PR de la **30-1b seulement** — jamais dans un message de commit,
même en prose ou au futur (B4 de la rétrospective 28) ; la PR de la 30-1a porte `refs #686`.

⚠️ **Guy ne testera qu'en fin d'epic** (fichier d'epic, § *Indispensable à la mise en production*) : aucune recette
intermédiaire ne rattrapera ce que les revues manquent. **Les E2E de parcours portent la recette** (AC-b12), avec
captures clair et sombre.

⚠️ **Zones partagées avec la 30-2** (fiche écrite en parallèle, worktree `kesh-30-2`) : `supplier_invoices.rs`
(`kesh-db`), le manuel utilisateur § *Factures fournisseurs et paiements*, le CHANGELOG `## [0.14.0]`, les catalogues
i18n. La 30-1 se développe **en premier** (C-30-0-3) ; la 30-2 se reporte sur `main` après elle.

## Story

**As a** utilisateur qui saisit après coup les factures fournisseur des paiements de son relevé (Comptable ou Admin),
**I want** que la réconciliation me propose, pour chaque **débit**, la facture fournisseur **ouverte** qui lui
correspond, et qu'*Accepter* la règle,
**so that** un seul processus — saisir la facture ouverte, puis laisser la réconciliation la régler — tienne pour mes
fournisseurs comme pour mes clients : facture **payée** et **lettrée**, transaction **rapprochée**, et le tout
défaisable par *Annuler le rapprochement*.

Issue : [#686](https://github.com/guycorbaz/kesh/issues/686) (enhancement). Epic :
`_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md` § *30-1*. Cible : **v0.14.0** (C-30-0-1). Voisine :
[#671](https://github.com/guycorbaz/kesh/issues/671) (C-30-0-2, décision au § *#671*). Garde : [#482] (un débit n'est
jamais rapproché d'une facture **client**).

[#482]: https://github.com/guycorbaz/kesh/issues/482

---

## Ce qui existe, établi au code (`15686d55`) — C-30-1-1

### Côté client : la réconciliation telle qu'elle est

| Fait | Au code |
|---|---|
| **Proposition.** `GET /reconciliation/proposals` charge les transactions `pending` non rejetées du compte (100 par défaut, 500 au plus, sentinelle `limit + 1` → `hasMore`), puis, par transaction, les factures candidates, les contacts en un lot, et note | `routes/reconciliation.rs:674-1007` (`get_proposals`) ; `repositories/reconciliation.rs:646` |
| **Un débit n'a aujourd'hui aucune facture candidate** : `if tx.amount <= Decimal::ZERO { … Vec::new() … continue; }` — mais la **règle d'affectation** reste calculée pour lui (elle n'hérite pas du filtre de signe, P-H7) | `routes/reconciliation.rs:749-752` ; repli de règle `:917-950` (`!has_strong_invoice_match && !rule_masked_by_overpaid_invoice && tx.currency == "CHF"`) |
| Une transaction non CHF n'a ni facture ni règle | `:743-746` |
| **Note** : `propose_matches` (pur, `kesh-reconciliation`) — montant 0,50 (binaire, au centime), référence 0,40 (containment bidirectionnel de `coalesce(reference, end_to_end_id, transaction_id)` contre `invoice_number`, demi-score si préfixe commun ≥ 4), contact 0,10 ; total nul écarté ; tri score décroissant puis `imputation_order` | `kesh-reconciliation/src/matching.rs:72-126`, `:128-195` ; **typé `Invoice`** (`:74`) |
| **Départage** (29-5) : `imputation_order` = date d'exigibilité `max(date, échéance)`, puis date, puis `id` ; même ordre en SQL avant `LIMIT 50` | `repositories/reconciliation.rs:299-344` ; **typé `Invoice`** |
| **Fenêtre de dates** (28-7, 29-2) : une seule définition, `InvoiceDateWindow` — `F ≤ D + 1 jour` et ancre `A ≥ D − 30 jours`, `A` = plus tardive de la date, de l'échéance et du dernier rappel non annulé ; `D` = date de valeur, à défaut de comptabilisation (`payment_date`) ; lue par la proposition **et** l'acceptation | `repositories/reconciliation.rs:119`, `:262`, `:270-272`, `:385-449` |
| **Montant** (29-6) : le reste dû entre `least_due_without_overpayment(tx)` (un demi-centime sous la transaction) et `tx + 0,05` ; les factures dépassées de 1 à 5 centimes ne sont plus candidates mais, si elles portent un indice, **masquent la règle** (`find_invoices_slightly_overpaid_for_window`) | `repositories/reconciliation.rs:532-644` ; `routes/reconciliation.rs:757-799`, `:880-887` |
| **Priorité** (29-3) : `candidate_rank` — facture à signal (montant exact, référence, contact) rang 0, règle rang 1, facture sans signal rang 2 ; `match` **exhaustif sans `_`** sur `CandidateType` ; tri stable ; l'écran n'affiche et n'accepte que `candidates[0]` | `routes/reconciliation.rs:130-136`, `:199-240`, `:968` |
| **Acceptation** : `POST /reconciliation/accept`, enum `type` (`invoice`, `split`, `rule`), pré-vol par lot (27-4 : refus par proposition), compte clos → chaque proposition en `failed[]` (28-4a), un seul `GET_LOCK` par compte (`with_account_lock`), un `SAVEPOINT` par proposition, rejeu sur interblocage (`retry_with`) | `routes/reconciliation.rs:544-601`, `:1087-1300`, `:1305-1444`, `:1446-1512` |
| **`accept_one_invoice`** : recharge la transaction (lecture ordinaire), `pending`, **refus d'un débit** (`refuse_non_credit_settlement`, `RECONCILIATION_TRANSACTION_NOT_CREDIT`, 28-6), facture, éligibilité, fenêtre, score recalculé (garde `> 0`), compte de banque, créance lue sur l'écriture de vente, garde « banque ≠ créance » (15-6b), trop-perçu, arrondi, exercice ouvert, écriture `D banque / C débiteurs`, ligne `invoice_settlements` (`BankTransfer { bank_account_id }` = le compte de l'import), `UPDATE bank_transactions … AND status = 'pending' AND version = ?`, `UPDATE invoices … AND version = ?` (verrou anti-double règlement, 25-4-c2), lettrage, deux audits (`reconciliation.accepted`, `invoice.paid` ou `invoice.partially_settled`) | `routes/reconciliation.rs:1705-1729`, `:1764-2483` |
| `AcceptedProposal.invoice_id` vaut **0** (sentinelle) pour `split` et `rule` | `:2867-2871`, `:3365-3368` |
| **Annulation** (25-3-b) : `reconciliation_cancel::cancel_in_tx` — verrou de la transaction, **classement par lecture verrouillante** de `invoice_settlements` (`ReconciliationKind::InvoiceSettlement` ou `Entry`), verrou de la facture, verrou de l'écriture **et** de son exercice, garde compte clos non verrouillante, motifs (forme **exemptée** du rang 3 pour cette transaction), lien défait **avant** la contre-passation, règlement annulé par `invoice_settlements_write::cancel_settlement_in_tx` (**appelé**, jamais réécrit) ou écriture contre-passée par `reverse_in_tx`, revérification verrouillante du compte (29-4b), audit `reconciliation.cancelled` | `repositories/reconciliation_cancel.rs:46-104`, `:148-182`, `:194-231`, `:293-454` |
| La vue `GET /reconciliation/transactions/{id}` rend `kind` (`invoice_settlement` / `entry` / `null`), `invoiceId`, `invoiceNumber`, `cancellable` et le motif ; `…/cancel` rend `{ bankTransaction, reversalJournalEntryId, invoiceId }` | `routes/reconciliation.rs:4694-4790` |
| **Rattachement** (29-8) : une transaction **au crédit** se rattache à un règlement **client** par virement déjà saisi (requête unique `candidates_sql`, quatre conditions, verrous `t → s → je`) | `repositories/reconciliation_link.rs:1-60`, `:150-176`, `:329-445` ; `transaction_can_be_linked` = crédit CHF (`:77-79`) |

### Côté fournisseur : le règlement depuis la fiche

| Fait | Au code |
|---|---|
| Entité : `status` `open` / `paid` / `cancelled` ; `total_amount` = **TTC**, `DECIMAL(19,4)` ; `supplier_invoice_number` (libre, nullable) ; `payment_reference` (QRR/SCOR, `VARCHAR(64)`) ; `invoice_date`, `due_date` ; colonnes `settlement_*` posées au paiement ; **aucune colonne de devise** (mono-CHF) | `entities/supplier_invoice.rs:17-48` ; migration `20260628000001_supplier_invoices.sql:27-66` |
| **Aucune route de modification** d'une facture fournisseur (création, paiement, annulation, annulation du règlement, pièce jointe) : date, échéance et TTC sont **immuables** une fois la facture saisie | `kesh-api/src/lib.rs:467-511` |
| Le TTC est `Σ HT + Σ TVA` ; la TVA est arrondie au centime par ligne, mais `line_total = quantité × prix unitaire` n'est **pas** arrondi : un TTC à plus de deux décimales est **possible** | `supplier_invoices.rs:109-168` ; `kesh-core/src/accounting/vat.rs:40-44` |
| **Le règlement est binaire** : toujours le TTC entier, jamais partiel ; il n'existe pas de reste dû fournisseur | `supplier_invoices.rs:713-990` |
| `pay` (enveloppe) = `guard_not_in_generated_batch` (verrou `FOR UPDATE` de la facture puis lecture **non verrouillante** de l'appartenance à un lot `generated`) puis `pay_in_tx` | `:603-628`, `:634-650`, `:680-708` |
| `pay_in_tx` (« guard-free », réutilisé par `confirm_batch`) : (1) facture `FOR UPDATE`, statut `open` ; (2) dette **et** montant lus sur la ligne de crédit de l'écriture d'achat (`purchase_payable_line`) ; (3) virement → `bank_accounts` lu `FOR UPDATE` (`closed_on`, `archived`, compte lié : `NotFound`, `BankAccountClosed`, `ConfigurationRequired`) ; (3 bis) contrepartie ≠ créanciers ; (4) exercice ouvert couvrant la date ; (5) écriture `D créanciers / C banque` du **TTC**, journal Banque, sans garde de postabilité ; (6) `status = 'paid'`, colonnes `settlement_*`, **`paid_at = CURRENT_TIMESTAMP(3)`** (le moment de la saisie, **pas** la date du paiement), `… AND version = ? AND status = 'open'` ; (6 bis) lettrage `document` ; (7) audit `supplier_invoice.paid` par **`NewAuditLogEntry::user`** (jamais la clé d'API) | `:713-990`, en particulier `:760`, `:907`, `:931`, `:953` |
| Annuler le règlement : `cancel_settlement_in_tx` (verrou facture → écriture + exercice → garde compte clos → motifs → dissolution du lettrage → contre-passation `reverse_owned_in_tx` autorité `SupplierSettlement` → `open` → revérification verrouillante → audit) ; motifs par `supplier_settlement_cancel_blocker` — queue commune **sans exemption** (`unlinking = None`) | `:1251-1280`, `:1304-1469` |
| Une écriture de règlement fournisseur **rapprochée** d'une transaction ferait refuser l'annulation du règlement depuis la fiche au rang 3 (`MATCHED_BANK_TRANSACTION`) — la queue est commune | `settlement_cancellation.rs:121-140` ; texte affiché `settlement-cancel-blocked-bank-match` (`messages.ftl:851`) par `supplierSettlementCancelMessage` (`frontend/src/lib/features/supplier-invoices/settlement-cancel.ts:19-29`) |
| ⚠️ Ce texte dit « annulez d'abord le rapprochement, **depuis cette fiche** ou le détail de l'import bancaire » : vrai sur la fiche d'une facture **client** (`InvoiceSettlements.svelte` porte `CancelReconciliationDialog`), **faux** sur la fiche fournisseur, qui n'a pas ce dialogue. Inatteignable aujourd'hui, atteignable dès la 30-1b | `grep -rln CancelReconciliationDialog frontend/src` → `invoices/[id]/+page.svelte`, `InvoiceSettlements.svelte`, `bank-import/[id]/+page.svelte` |
| Annuler une facture **payée** (25-3-c) la passe `cancelled` et **détache** son règlement (colonnes à `NULL`, écriture conservée : « une sortie de banque réelle […] se contre-passe ») ; une écriture de règlement détachée et rapprochée est donc classée `Entry` par le dé-rapprochement et contre-passée par `reverse_in_tx` | `supplier_invoices.rs:1036-1218` ; `errors.rs:143-150` (`DetachedSupplierSettlement`) |
| Lots de paiement : `create_batch` verrouille la facture `FOR UPDATE` et insère `payment_batch_items` ; `confirm_batch` règle chaque facture par `pay_in_tx` (exige `open`) | `payment_batches.rs:69-200`, `:432-500` |

### La référence, le fournisseur, la devise

- **Référence** : à l'import d'un QR, `payment_reference` reçoit la valeur QRR/SCOR (`routes/imported_supplier_invoices.rs:211-216`) ;
  à l'import CAMT, `tx.reference` = `CdtrRefInf/Ref` (la référence structurée du paiement), à défaut `NtryRef`
  (`kesh-import/src/camt053/mod.rs:454-458`, `:629`). Un débit qui paie une QR-facture porte donc **la même référence**
  que la facture. La note actuelle ne compare `tx.reference` qu'au **numéro** de facture.
- **Fournisseur** : `contact_id` ; pour un débit, `counterparty_name` est le `Nm` capturé sous `RltdPties/(Cdtr|Dbtr)` —
  le **dernier** dans l'ordre du document, quel que soit le sens (`camt053/mod.rs:459-474`). L'ordre du schéma ISO place
  `Dbtr` avant `Cdtr` : pour un débit, c'est le créancier — le fournisseur — qui est retenu si la banque transmet les
  deux. **Non vérifié sur un relevé réel** (aucune donnée réelle ne se cite ici) ; le score de contact pèse 0,10 et ne
  décide de rien seul.
- **Devise** : aucune colonne côté facture fournisseur ; depuis la 27-2 aucun import n'enregistre de transaction non CHF ;
  `get_proposals` écarte déjà les transactions non CHF.

### Prémisses de #686 — verdicts

| Prémisse | Verdict |
|---|---|
| « la réconciliation ne propose de factures que pour les crédits » | **Exacte** (`routes/reconciliation.rs:749-752`) |
| « un débit n'a ni proposition de facture fournisseur, ni *Rattacher à un règlement* » | **Exacte** (`transaction_can_be_linked`, `reconciliation_link.rs:77-79` ; bouton `canLink`, `ReconciliationProposals.svelte`) |
| « *Accepter* … exactement ce que fait le règlement depuis la fiche par virement » | **Exacte à une réserve près** : `pay_in_tx` pose `paid_at` à l'heure de la saisie, pas à la date du paiement (C-30-1-19) |
| « montant égal au **reste dû** » | **À préciser** : il n'y a **pas** de reste dû fournisseur ; le règlement est binaire, du TTC entier (C-30-1-5) |
| « *Annuler le rapprochement* défait le règlement, comme côté client » | **Fausse aujourd'hui** : `classify` ne connaît que `invoice_settlements` ; une écriture de règlement fournisseur rapprochée serait classée `Entry`, et `reverse_in_tx` refuserait une écriture possédée par un règlement fournisseur — **défait par rien** (C-30-1-2 ; c'est l'objet de la 30-1a) |
| « une seule proposition affichée, l'ordre de priorité écrit (#527) » | **Exacte**, à étendre (`candidate_rank`, AC-b5) |

---

## Découpage (règle de splitting préventif) — C-30-1-3

La story touche **plus de cinq modules** : `kesh-reconciliation/matching`, `kesh-db/repositories/reconciliation`,
`kesh-db/repositories/supplier_invoices` (et son appelant `payment_batches`), `kesh-db/repositories/reconciliation_cancel`,
`kesh-api/routes/reconciliation`, `frontend/src/lib/features/reconciliation` (plus les catalogues i18n, le manuel et
`api-external.md`). Découpée, dans **cet ordre**, sur le patron « story-zéro puis rollout » :

| Sous-story | Clé | Objet | Modules |
|---|---|---|---|
| **30-1a** | `30-1a-annuler-rapprochement-fournisseur` | le dé-rapprochement sait **défaire un règlement fournisseur** (classement, motifs exemptés, annulation appelée), et la vue le dit | `reconciliation_cancel`, `supplier_invoices` (signature d'un lecteur), route de la vue et de `…/cancel`, `CancelReconciliationDialog` + texte de la fiche fournisseur |
| **30-1b** | `30-1b-proposer-accepter-fournisseur` | la proposition et l'acceptation des factures fournisseur pour un débit | `matching`, `repositories/reconciliation`, `supplier_invoices` (chemin de paiement), `routes/reconciliation`, `ReconciliationProposals` |

**Pourquoi cet ordre, et non l'inverse.** Livrée la première, l'acceptation produirait sur `main` des transactions
rapprochées d'un règlement fournisseur que **rien ne défait** (prémisse fausse ci-dessus) — l'état même que la
25-3-b a fermé pour le client (« l'écriture était définitivement incorrigible »). L'annulation d'abord : à aucun
commit de `main` un tel état n'est produisible sans être défaisable. La 30-1a se teste sur un état posé par la
fixture (règlement depuis la fiche, puis lien posé en SQL), que la 30-1b rejouera par l'acceptation.

Chaque sous-story a sa revue de code et son gate complet (**`kesh-db` touché : gate complet même en boucle**).

---

## La voisine #671 — C-30-1-4

#671 (P2) demande d'étendre le rattachement de la 29-8 à quatre chemins ; sa première puce est le **symétrique
fournisseur** (un débit payé depuis la fiche d'une facture fournisseur). **Ce symétrique ne sort pas mécaniquement de
la 30-1** :

- la requête de la 29-8 (`candidates_sql`) lit `invoice_settlements` et un mouvement **positif** sur le compte de la
  banque ; le règlement fournisseur vit dans les colonnes de `supplier_invoices`, son mouvement est **négatif**, et
  l'écart `t − M` change de signe — une seconde requête, une seconde règle d'écart, un second ordre de verrous
  (`supplier_invoices` au lieu de `invoice_settlements`) ;
- la route, la modale (`LinkSettlementModal`, `canLink` : crédit seulement), les textes et le manuel se dupliqueraient
  ou se généraliseraient — un travail de conception, pas un geste ;
- **le besoin de Guy est couvert autrement** : il saisit la facture **ouverte**, et la 30-1b la règle par la
  réconciliation ; il n'a pas à la payer depuis la fiche.

**Retenu** : #671 reste ouverte, inchangée. Ce que la 30-1 lui **apporte** (à dire dans un commentaire de
l'orchestrateur) : la 30-1a classe un règlement fournisseur rapproché (`supplier_settlement`) et sait le défaire — le
rattachement fournisseur, s'il est fait un jour, n'aura plus que le lien à poser. Angle mort L1 ci-dessous.

---

## Périmètre minimal — ce qui est dedans, ce qui est dehors

**Dedans** : proposer pour un **débit CHF** les factures fournisseur **ouvertes** du bon montant ; les accepter par lot
(patron `{ accepted, failed }`) ; défaire l'acceptation par *Annuler le rapprochement* ; la priorité écrite étendue ;
les textes (écran, manuel, `api-external.md`, i18n ×4, CHANGELOG).

**Dehors** (chacun nommé, aucun ne s'intègre en silence) :

- **#671** (C-30-1-4) — rattacher un débit à un règlement fournisseur déjà saisi.
- **Un débit jamais rapproché d'une facture client, un crédit jamais d'une facture fournisseur** : c'est une garde,
  pas un dehors (AC-b4, AC-b7).
- **Avoirs fournisseur, remboursement d'un fournisseur (crédit)** : aucun modèle d'avoir fournisseur n'existe.
- **Paiement partiel d'une facture fournisseur** : le règlement est binaire (C-30-1-5).
- **Un paiement qui règle plusieurs factures fournisseur** (#671, troisième puce).
- **Les lots pain.001** : un lot confirmé règle ses factures par `pay_in_tx` ; son débit unique au relevé relève de
  #671 (quatrième puce). La 30-1b ne propose **jamais** une facture engagée dans un lot `generated` (AC-b3).
- **`paid_at` à la date du paiement** (C-30-1-19) : défaut voisin préexistant, **à ouvrir en issue** (§ *Issues à
  ouvrir*).
- **Le dialogue *Annuler le rapprochement* sur la fiche fournisseur** (symétrique de `InvoiceSettlements.svelte`) :
  non (C-30-1-11) ; la fiche fournisseur dit où le faire.

---

## Critères d'acceptation

### Partie a — `30-1a-annuler-rapprochement-fournisseur`

**AC-a1 — Le classement connaît le règlement fournisseur.**
`ReconciliationKind` gagne `SupplierSettlement { supplier_invoice_id: i64 }`, de code canonique `supplier_settlement`
(`reconciliation_cancel.rs:46-67`). `classify` (`:75-104`) cherche, **après** `invoice_settlements` et **dans la même
forme verrouillante ou non** que lui, `SELECT id FROM supplier_invoices WHERE company_id = ? AND
settlement_journal_entry_id = ? AND status = 'paid'` (avec `FOR UPDATE` si `for_update`). Ordre des bras : règlement
client, puis règlement fournisseur, puis `Entry`. Une écriture de règlement **détachée** (facture annulée, colonnes à
`NULL`) reste `Entry` (C-30-1-6).
*Test* : T-a1 (trois classements, dont la facture annulée après paiement).

**AC-a2 — Le dé-rapprochement défait le règlement, en l'appelant.**
À l'étape (2) de `cancel_in_tx`, le classement verrouillant **verrouille la ligne `supplier_invoices`** (c'est la
« ligne de règlement » du fournisseur : il n'y a pas de seconde ligne à verrouiller ensuite) ; l'étape (3) (écriture +
exercice) est inchangée. À l'étape (6), le bras `SupplierSettlement` appelle
`supplier_invoices::cancel_settlement_in_tx(tx, company_id, supplier_invoice_id, user_id)` — **appelée, jamais
réécrite** — et rend sa `reversal_journal_entry_id`. État final : transaction `pending`, `matched_entry_id = NULL`,
`auto_match_rejected_at = NULL` ; facture `open`, colonnes `settlement_*` et `paid_at` à `NULL` ; contre-passation datée
du jour (`reverses_entry_id` posé) ; lettrage `document` dissous, paiement lettré avec sa contre-passation ; dette
redevenue ouverte au compte créanciers.
*Test* : T-a2 (état final complet, par requêtes en base), T-a3 (le grand livre : solde créanciers et banque revenus à
leur valeur d'avant le règlement).

**AC-a3 — Les motifs, en forme exemptée.**
`supplier_settlement_cancel_blocker` (`supplier_invoices.rs:1251-1280`) prend un paramètre `unlinking: Option<i64>`
transmis à la queue commune — **même** forme que `invoice_settlements_write::settlement_cancel_blocker_unlinking` ; ses
appelants actuels passent `None` (comportement inchangé). `reconciliation_cancel::cancel_blocker` (`:148-182`) appelle
pour le bras fournisseur la forme exemptée pour `bt.id`. Le geste refuse lui-même, comme pour le client,
`FiscalYearClosed` et `DocumentLetteringInClosedPeriods` en `ReconciliationNotCancellable` **avant** de défaire le lien ;
les autres rangs sont refusés par le geste appelé ou par le socle. ⛔ Aucune seconde précédence écrite.
*Tests* : T-a4 (exercice du règlement clos → `409`, rien écrit, lien intact), T-a5 (vue : `cancellable = true` sur un
règlement rapproché ouvert — la forme non exemptée rendrait `MATCHED_BANK_TRANSACTION`, mutation M-a3).

**AC-a4 — Verrous et naissance de la vue.**
Ordre du geste pour le bras fournisseur : verrou nommé (route) → `bank_transactions` (`t`) → `supplier_invoices`
(par l'index de `settlement_journal_entry_id`) → `journal_entries` + `fiscal_years` → garde non verrouillante du compte
clos → … → revérification verrouillante du compte (dernière acquisition). ⛔ **Aucune lecture non verrouillante avant
l'étape 3**, y compris pour le classement fournisseur. ⚠️ **Coût assumé et écrit au doc-comment** : sur une écriture
qui n'est **pas** un règlement fournisseur (`Entry`, règlement client), la lecture verrouillante qui ne trouve rien pose
un **verrou d'intervalle** sur l'index secondaire non unique de `settlement_journal_entry_id` — le même coût que la
lecture de `invoice_settlements` sur un `Entry` aujourd'hui (index unique, intervalle verrouillé en `REPEATABLE READ`
quand rien n'est trouvé) ; un règlement fournisseur concurrent (`pay_in_tx`, étape 6) qui écrit dans cet intervalle
**attend** la fin du dé-rapprochement. Un interblocage éventuel est rejoué par la route (`retry_app_on_deadlock`). La
fiche demande à la validation (lentille F) de chercher un cycle (C-30-1-7).
*Test* : T-a6 (course : `cancel_settlement_in_tx` depuis la fiche contre le dé-rapprochement de la même transaction —
l'un aboutit, l'autre est refusé ou rejoué, jamais d'état mixte ; sonde `sonde_verrou_nowait`, **jamais**
`INNODB_TRX`).

**AC-a5 — La vue et la réponse disent le fournisseur.**
`GET /reconciliation/transactions/{id}` : `kind` peut valoir `supplier_settlement` ; deux champs **ajoutés**,
`supplierInvoiceId` et `supplierInvoiceNumber` (nuls hors de ce cas) ; `invoiceId` / `invoiceNumber` restent ceux d'une
facture **client** (nuls pour un règlement fournisseur). `POST …/cancel` : champ ajouté `supplierInvoiceId`. Audit
`reconciliation.cancelled` : `kind = "supplier_settlement"`, `supplierInvoiceId` ajouté (`invoiceId` et `settlementId`
nuls). Contrat **additif** : aucun champ existant ne change de sens (C-30-1-8).
*Tests* : T-a7 (forme JSON des trois cas), T-a8 (audit).

**AC-a6 — L'écran.**
`CancelReconciliationDialog.svelte` : pour `kind === 'supplier_settlement'`, le texte de confirmation
`reconciliation-cancel-confirm-supplier-invoice` — « Une écriture inverse datée d'aujourd'hui sera passée, le paiement
de la facture fournisseur { $invoice } sera retiré et la facture redeviendra ouverte, à payer ; la transaction
redeviendra à rapprocher. » (quatre locales). `reconciliation.types.ts` : `kind` élargi, champs ajoutés.
Sur la fiche fournisseur, le motif `MATCHED_BANK_TRANSACTION` de l'annulation du règlement reçoit un texte **propre**,
`supplier-invoices-settlement-cancel-blocked-bank-match` — « Ce paiement est rapproché d'une transaction bancaire :
annulez d'abord le rapprochement, depuis le détail de l'import bancaire. » — au lieu du texte commun qui renvoie à
« cette fiche » (C-30-1-11). Le test « la queue est RÉUTILISÉE » (`settlement-cancel.test.ts:22-32`) est **amendé** :
quatre codes réutilisés, celui-ci surchargé, et le test le dit.
*Tests* : Vitest T-a9 (dialogue, trois `kind`), T-a10 (fiche fournisseur, le texte propre).

**AC-a7 — Contrat écrit.**
`docs/api-external.md` § *Annuler un rapprochement* (`:596-612`) : `kind = supplier_settlement`, les champs ajoutés,
« pour une facture fournisseur, retire son paiement » ; la mention « depuis la v0.14.0 ». Le manuel et le CHANGELOG
sont écrits par la 30-1b (rien ne produit cet état avant elle ; C-30-1-9).

### Partie b — `30-1b-proposer-accepter-fournisseur`

**AC-b1 — Une seule notation pour les deux sortes de factures (DRY).**
- `kesh-db/src/repositories/reconciliation.rs` : un trait public `PayableDocument` (`document_id() -> i64`,
  `document_date() -> NaiveDate`, `due_date() -> Option<NaiveDate>`), implémenté pour `Invoice` (`date`, `due_date`) et
  `SupplierInvoice` (`invoice_date`, `due_date`) ; `invoice_exigibility_date` et `imputation_order` deviennent
  génériques sur ce trait, **sans changer leur résultat** pour une facture client (test d'accord SQL/Rust existant
  `exigibility_date_sql_and_rust_agree` inchangé et vert).
- `kesh-reconciliation/src/matching.rs` : `propose_matches` devient générique sur un trait `MatchableDocument:
  PayableDocument` qui fournit le **score de référence** du document contre la transaction ; `Invoice` garde
  **exactement** la règle actuelle (numéro, containment, demi-score de préfixe). `SupplierInvoice` : le **maximum** de
  (a) la règle actuelle appliquée à `supplier_invoice_number`, (b) la **référence de paiement** : `1.0` si
  `payment_reference` **égale** la référence de la transaction après retrait de **tous** les blancs et mise en
  majuscules, sinon `0.0` — ni containment ni préfixe (une référence QR commence souvent par des zéros ; C-30-1-12).
  `MatchProposal.invoice_id` est renommé `document_id` (champ interne ; `grep -rn MatchProposal crates` ne rend que
  `matching.rs` et `lib.rs:33` — à revérifier au développement).
- **Tous** les tests existants de `matching.rs` passent **sans modification de leurs assertions** (les constructeurs
  de fixtures peuvent changer de signature).
*Tests* : T-b1 (référence QR avec et sans blancs, casse ; numéro fournisseur ; maximum des deux), T-b2 (préfixe commun
de zéros sur une référence de paiement → 0), T-b3 (départage fournisseur par exigibilité).

**AC-b2 — La fenêtre de dates fournisseur, écrite et tranchée (C-30-1-13).**
Une seule définition étendue : `InvoiceDateWindow` gagne un constructeur `for_supplier_payment_date(d)` (et
`for_supplier_transaction(tx)`) ; `check` est **réutilisé tel quel**. Bornes, sur `D` = `payment_date(tx)` :

- **`F ≤ D + 7 jours`** (`SUPPLIER_PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS = 7`) — un achat payé en ligne dont la facture
  est émise à l'expédition, quelques jours après ;
- **`X ≥ D − 365 jours`** (`SUPPLIER_INVOICE_WINDOW_DAYS_AFTER_EXIGIBILITY = 365`), `X` = date d'exigibilité
  `max(F, échéance)` — un paiement fait jusqu'à un an après l'échéance est proposé, y compris à cheval sur deux
  exercices ; au-delà, une facture ouverte du même montant n'est plus proposée (abonnements de montant identique).
- ⛔ La fenêtre **ne lit jamais** `created_at` ni la date du jour : la facture peut être **saisie** des mois après le
  paiement — c'est l'usage de Guy, et c'est le cas nominal.

Refus à l'acceptation : `RECONCILIATION_INVOICE_NOT_ELIGIBLE` avec `details.reason` =
`payment_date_before_invoice_date` (facture datée de plus de 7 jours après le paiement) ou `payment_date_outside_window`
avec `details.window_days = 365`. Expression SQL de `X` : `GREATEST(si.invoice_date, COALESCE(si.due_date,
si.invoice_date))`, tenue d'accord avec la fonction Rust par un test de parité (patron
`exigibility_date_sql_and_rust_agree`).
*Tests* : T-b4 (bornes incluses et exclues des deux côtés, `due_date` nulle, échéance antérieure à la date), T-b5
(parité SQL/Rust), T-b6 (facture saisie aujourd'hui, datée et échue un an avant le paiement moins un jour → proposée).

**AC-b3 — Les candidates fournisseur (C-30-1-5, C-30-1-14).**
`repositories/reconciliation.rs` : `find_open_supplier_invoices_for_window(executor, company_id, window, amount)` —
`amount` = valeur absolue du débit :
1. `company_id` ;
2. `status = 'open'` ;
3. **`total_amount = amount`** — égalité **exacte** en décimal (le règlement est binaire et écrit le TTC : un TTC qui
   diffère du débit, fût-ce d'un demi-centime, écrirait une ligne de banque différente du relevé) ;
4. **jamais une facture engagée dans un lot `generated`** : `NOT EXISTS (SELECT 1 FROM payment_batch_items pbi JOIN
   payment_batches pb ON pb.id = pbi.payment_batch_id WHERE pbi.supplier_invoice_id = si.id AND pb.status =
   'generated')` ;
5. fenêtre de l'AC-b2 ;
6. ordre d'imputation (`X`, `invoice_date`, `id` croissants) **avant** `LIMIT 50`.

Une seconde fonction, `find_supplier_invoices_near_amount_for_window`, même requête (un seul texte SQL, paramétré par
le seul prédicat de montant, patron `unpaid_invoices_in_window_sql`), prédicat `ABS(total_amount − amount) <= 0.05 AND
total_amount <> amount` : les factures **presque** au montant, **jamais proposées**, lues pour garder la règle (AC-b5).
*Tests* : T-b7 (les six filtres, chacun par un cas qui le viole seul), T-b8 (lot `generated` → absente ; lot `confirmed`
ou `cancelled` → présente si ouverte), T-b9 (TTC à quatre décimales `100.0040` contre débit `100.00` → absente des
candidates, présente des « presque »), T-b10 (`LIMIT 50` après l'ordre).

**AC-b4 — La proposition : un débit, des factures fournisseur ; jamais de croisement (#482).**
Dans `get_proposals` :
- un **crédit** CHF garde **exactement** le chemin actuel (factures client) ; il ne reçoit **jamais** de facture
  fournisseur ;
- un **débit** CHF reçoit les candidates de l'AC-b3, notées par `propose_matches`, contacts chargés dans le **même** lot
  que les contacts clients ; il ne reçoit **jamais** de facture client ;
- un montant nul ne reçoit aucune facture (ni client ni fournisseur) ;
- `CandidateType` gagne `SupplierInvoice` (sérialisé `supplier_invoice`) ; `ReconciliationCandidate` gagne
  `supplier_invoice_id: Option<i64>` et `contact_name: Option<String>` (rempli pour les **deux** sortes de factures,
  C-30-1-15) ; pour une facture fournisseur, `invoice_number` = `supplier_invoice_number`, `invoice_amount` = TTC au
  centime, `invoice_total_ttc = None`, `invoice_date` = `invoice_date` ; `invoice_id = None`.
*Tests* : T-b11 (débit → fournisseur seulement, même si une facture client a le même montant, le même numéro et le même
contact), T-b12 (crédit → client seulement, symétrique), T-b13 (forme JSON du candidat).

**AC-b5 — La priorité écrite, étendue.**
`candidate_rank` (`routes/reconciliation.rs:199-210`) : bras **explicite** `CandidateType::SupplierInvoice`, même règle
que `Invoice` (rang 0 si signal, 2 sinon) ; le `match` reste **sans `_`**. Le repli de règle : une facture fournisseur
candidate a toujours `amount_score = 1` (AC-b3, filtre 3), donc un score ≥ 0,5 : elle **écarte la règle** (`has_strong_
invoice_match`). Une facture fournisseur **presque** au montant (AC-b3) qui porte un indice (référence ou contact) **garde
la règle** comme le fait la 29-6 (`rule_masked_by_overpaid_invoice`) : sans elle, la règle « Swisscom → 6510 » porterait
en charge un débit dont la facture Swisscom, saisie et ouverte, compte déjà la charge — **une charge comptée deux fois,
muette** (leçon de la 29-6 : *retirer une candidate peut démasquer une autre proposition* ; ici, ne pas l'ajouter
aussi). La ligne reste alors « Aucune correspondance ».
*Tests* : T-b14 (`candidate_order_tests` : fournisseur à signal devant la règle), T-b15 (règle écartée par une facture
fournisseur candidate), T-b16 (règle gardée — ligne vide — par une facture presque au montant désignée par sa référence
ou son fournisseur ; règle proposée si la facture presque au montant ne porte aucun indice).

**AC-b6 — L'acceptation : la proposition `supplier_invoice`.**
`AcceptProposalInput` gagne `#[serde(rename = "supplier_invoice")] SupplierInvoice { bank_transaction_id,
supplier_invoice_id }` ; `bank_transaction_id()` couvre le variant ; pré-vol de surface `supplierInvoiceId > 0`
(`400`) ; message de `AcceptBodyExtractor` complété (`"supplier_invoice"`) ; `accept_one` dispatche vers
`accept_one_supplier_invoice`. `AcceptedProposal` gagne `supplier_invoice_id: Option<i64>` (sérialisé `null` hors de ce
cas) ; pour ce cas, `invoice_id = 0` (la sentinelle de `split` et `rule`, C-30-1-8).

`accept_one_supplier_invoice`, dans le `SAVEPOINT` de la proposition, **dans cet ordre** :
1. recharger la transaction (`find_pending_by_id_for_account`) → `BANK_TRANSACTION_NOT_FOUND` ; non `pending` →
   `RECONCILIATION_ALREADY_RECONCILED` ;
2. **sens** : un crédit ou un montant nul → `RECONCILIATION_TRANSACTION_NOT_DEBIT` (`details.reason` =
   `credit_transaction` | `zero_amount_transaction`, `details.transactionAmount`), **avant** toute lecture de la facture
   (AC-b7) ;
3. **verrou et éligibilité, en `kesh-db`** : `supplier_invoices::lock_for_bank_settlement_in_tx(tx, company_id,
   supplier_invoice_id) -> Result<Result<SupplierInvoice, SupplierSettlementRefusal>, DbError>` — facture `FOR UPDATE`
   (absente ou d'une autre société → `NotFound`), statut ≠ `open` → `NotOpen { status }`, appartenance à un lot
   `generated` lue par une lecture **verrouillante** (`LOCK IN SHARE MODE`) **après** le verrou de la facture →
   `InGeneratedBatch { batch_id }` (AC-b8) ;
4. fenêtre (AC-b2) sur la facture **verrouillée** ;
5. **montant** : `|tx.amount| ≠ total_amount` → `RECONCILIATION_AMOUNT_MISMATCH` (`details.amountDue`,
   `details.transactionAmount`) ; le score est recalculé (`propose_matches`) pour la trace et la réponse — il vaut
   ≥ 0,5 par construction, **aucune** garde de score n'est écrite (elle serait du code mort, C-30-1-16) ;
6. compte de banque lié (`BANK_ACCOUNT_NOT_CONFIGURED` / `BANK_ACCOUNT_NOT_FOUND`, comme l'étape (a) du client) ;
7. `supplier_invoices::pay_in_tx(tx, company_id, id, SettlementChoice::BankTransfer { bank_account_id }, payment_date(tx),
   actor)` — `bank_account_id` = **le compte de l'import**, jamais un choix du client ;
8. `UPDATE bank_transactions SET matched_entry_id = <écriture de règlement>, status = 'reconciled' … WHERE … AND status
   = 'pending' AND version = ?` → zéro ligne : `RECONCILIATION_ALREADY_RECONCILED` (`details.reason =
   "race_during_update"`) ;
9. audit `reconciliation.accepted` (`for_actor`) : `bank_transaction_id`, `supplier_invoice_id`, `score`,
   `batch_size`, `journal_entry_id` (le règlement), `purchase_journal_entry_id`, `paid_amount`.

Effet : facture `paid`, `settlement_type = 'bank_transfer'`, `settlement_bank_account_id` = compte de l'import ;
écriture `D créanciers / C banque` du TTC (= `|tx.amount|`), **datée de la date de paiement**, journal Banque, dans
l'exercice ouvert qui la couvre, projet de la facture ; lettrage `document` ; transaction `reconciled` pointant cette
écriture ; deux audits (`reconciliation.accepted`, `supplier_invoice.paid`).

**Inventaire clos des refus** — tout `DbError` rendu par les étapes 3 et 7 est **classé**, aucun n'escalade en
`AppError` (§ *Pattern batch*) ; une fonction `supplier_payment_error_to_failed_proposal`, `match` sans bras `_` sur les
variantes **nommées** ci-dessous plus un bras final `other → DATABASE_ERROR` :

| Source | Erreur | `errorCode` | `details` |
|---|---|---|---|
| étape 3 | `NotFound` | `SUPPLIER_INVOICE_NOT_FOUND` (neuf) | `{ supplierInvoiceId }` |
| étape 3 | `NotOpen` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `{ reason: "supplier_invoice_not_open" }` |
| étape 3 | `InGeneratedBatch` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `{ reason: "supplier_invoice_in_payment_batch", paymentBatchId }` |
| étape 7 | `BankAccountClosed` | `BANK_ACCOUNT_CLOSED` | `{ bankAccountId }` |
| étape 7 | `NotFound` (compte archivé ou absent) | `BANK_ACCOUNT_NOT_FOUND` | — |
| étape 7 | `ConfigurationRequired` | `BANK_ACCOUNT_NOT_CONFIGURED` | `{ bankAccountId }` |
| étape 7 | `SettlementCounterpartyIsClaimAccount` | (son code) | `claim_account_failed_proposal(…, Some(bank_account_id))` — rôle `counterparty`, côté **dette** |
| étape 7 | `FiscalYearInvalid` | `FISCAL_YEAR_INVALID` | `{ valueDate }` |
| étape 7 | `PeriodLocked` | `PERIOD_LOCKED` | `period_locked_failed_proposal` (projet de la facture) |
| étape 7 | `LaterFiscalYearClosed` | `LATER_FISCAL_YEAR_CLOSED` | `later_fiscal_year_closed_failed_proposal` |
| étape 7 | erreurs de projet (`create_in_tx`) | celles de `project_error_to_failed_proposal` | |
| étape 7 | `OptimisticLockConflict`, `IllegalStateTransition` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `{ reason: "race_during_update" }` |
| étape 7 | `Invariant`, erreurs structurelles du lettrage | `INTERNAL_ERROR` (précédé de `tracing::error!`) | — |
| toutes | autre (`Sqlx`…) | `DATABASE_ERROR` | `{ message }` — un interblocage est rejoué par `accept_batch` |

⚠️ L'inventaire se **refait au développement** depuis le code (`grep -n "Err(DbError::\|DbError::" ` sur `pay_in_tx`,
`journal_entries::create_in_tx_inner`, `letterings::sync_supplier_invoice_in_tx`) et se recopie au Dev Agent Record :
une variante atteignable non classée est un finding.
*Tests* : T-b17 (acceptation nominale : état complet en base — facture, écriture et ses deux lignes, date, exercice,
lettrage, transaction, deux audits), T-b18 (lot mixte client + fournisseur + règle : succès partiel, ordre de
`failed[]`), T-b19 (un test par ligne du tableau atteignable par décor), T-b20 (`200` partiel, jamais `4xx/5xx` pour un
refus par proposition).

**AC-b7 — Le sens de la transaction, une seule fonction (DRY).**
`refuse_non_credit_settlement` (`:1705-1729`) devient `refuse_wrong_direction(bank_transaction_id, amount,
expected: SettlementDirection)` — `Credit` pour une facture client, `Debit` pour une facture fournisseur. Côté client,
**code, raisons et `details` strictement inchangés** (`RECONCILIATION_TRANSACTION_NOT_CREDIT`, `debit_transaction` /
`zero_amount_transaction`) ; côté fournisseur, `RECONCILIATION_TRANSACTION_NOT_DEBIT`, `credit_transaction` /
`zero_amount_transaction`. Comparaison **au zéro**, jamais par `is_sign_negative()` (`-0.00`, C-28-6-12).
*Tests* : `non_credit_settlement_tests` (`:5729-5804`) inchangés et verts ; T-b21 (débit attendu : crédit, zéro,
`-0.00`).

**AC-b8 — La garde du lot de paiement est verrouillante (naissance de la vue).**
Dans la transaction d'un lot d'acceptation, la vue `REPEATABLE READ` est **déjà née** (lecture de la transaction à
l'étape 1, ou proposition précédente du lot) quand la facture est verrouillée. Un `create_batch` validé **entre** la
naissance de la vue et le verrou de la facture serait **invisible** à une lecture ordinaire de `payment_batch_items`, et
la facture serait réglée puis **payée une seconde fois par la banque** sur le fichier pain.001 déjà généré. D'où
l'étape 3 : appartenance lue en **lecture verrouillante**, après le verrou de la facture. `in_generated_batch`
(`supplier_invoices.rs:634-650`), lu aussi par les vues (`supplier_invoice_cancel_blocker`), **reste non
verrouillant** : la lecture verrouillante est une fonction (ou un paramètre) distincte, documentée.
`guard_not_in_generated_batch` (`pay`) peut l'employer aussi (sans effet sur son comportement : sa lecture vient en tête
de transaction).
*Test* : T-b22 (course : vue de la transaction d'acceptation née, `create_batch` validé sur une seconde connexion,
puis acceptation → refus `supplier_invoice_in_payment_batch`, rien écrit ; **rouge** avec une lecture ordinaire —
mutation M-b8).

**AC-b9 — `pay_in_tx` porte l'acteur (DRY).**
`pay_in_tx` remplace `user_id: i64` par `actor: letterings::Actor` (`letterings.rs:202`) : l'audit
`supplier_invoice.paid` passe par `NewAuditLogEntry::for_actor`, le lettrage reçoit la clé. Appelants mis à jour —
`pay` (`Actor { user_id, api_key_id: None }`), `confirm_batch` (`payment_batches.rs:496`, idem), le nouveau chemin
(la clé de la requête). Comportement de `pay` et `confirm_batch` **inchangé**.
*Test* : T-b23 (acceptation par une clé `read-write` : la clé figure dans `supplier_invoice.paid`,
`reconciliation.accepted` et `lettering.created`).

**AC-b10 — Concurrence.**
Ordre des verrous du chemin : verrou nommé → (lecture ordinaire de `t`) → `supplier_invoices` X → `payment_batch_items`
S → (dans `pay_in_tx`) facture X (tenu) → lignes de l'écriture d'achat (lecture) → `bank_accounts` X → exercice
(`create_in_tx`) → écriture → `supplier_invoices` (`UPDATE`) → lettrage → (route) `bank_transactions` X.
- règlement depuis la fiche **pendant** l'acceptation de la même facture : l'un aboutit, l'autre est refusé
  (`supplier_invoice_not_open` ou `IllegalStateTransition` côté fiche) ;
- clôture du compte bancaire concurrente : `pay_in_tx` lit `closed_on` sous `FOR UPDATE` (28-4a, W27) ;
- dé-rapprochement d'une autre transaction du même compte : sérialisé par le verrou nommé.
La validation (lentille F) cherche un cycle avec `close_in_tx`, `create_batch`, `confirm_batch`, `cancel_in_tx`
(fournisseur), `reconciliation_cancel::cancel_in_tx`, `accept_one_invoice`.
*Tests* : T-b24 (fiche contre réconciliation), T-b25 (clôture du compte pendant l'acceptation : refus
`BANK_ACCOUNT_CLOSED` ou clôture refusée `412`, jamais un règlement sur un compte clos).

**AC-b11 — L'écran.**
`ReconciliationProposals.svelte` :
- une candidate `supplier_invoice` s'affiche avec un badge *Facture fournisseur*
  (`reconciliation-labels-supplier-invoice-badge`), le **nom du fournisseur** (`contactName`), le numéro
  (`invoiceNumber`, à défaut `#id`) et le montant ; `data-testid="candidate-top-1"`, `data-candidate-type=
  "supplier_invoice"`, `data-supplier-invoice-id` ;
- une candidate `invoice` s'affiche **comme aujourd'hui** (C-30-1-15 : `contactName` n'y est pas affiché, pour ne pas
  déplacer les captures et les E2E de l'Epic 29) ;
- *Accepter* construit `{ type: 'supplier_invoice', bankTransactionId, supplierInvoiceId }` ;
- l'avis de priorité (`reconciliation-labels-priority-hint`, `messages.ftl:1283`) est réécrit : « Une seule proposition
  est affichée par transaction : pour un crédit, la facture client la mieux notée, pour un débit, la facture fournisseur
  la mieux notée, s'il y en a une, sinon la règle d'affectation qui correspond. Une facture proposée passe toujours
  devant une règle. » (quatre locales).
`failed-proposal-label.ts` : libellés de `RECONCILIATION_TRANSACTION_NOT_DEBIT`, `SUPPLIER_INVOICE_NOT_FOUND`,
`RECONCILIATION_AMOUNT_MISMATCH`, et de la raison `supplier_invoice_in_payment_batch` (le remède : annuler le lot) ;
le décompte du doc-comment (« Les 31 codes ») **recompté depuis la source** par la commande qu'il cite.
`reconciliation.types.ts` : types étendus.
*Tests* : Vitest T-b26 (rendu du candidat fournisseur, du candidat client inchangé), T-b27 (corps d'*Accepter* pour
les quatre types), T-b28 (libellés neufs).

**AC-b12 — E2E de parcours (la recette) et captures.**
`frontend/tests/e2e/reconciliation-supplier.spec.ts`, montants et références **uniques** par exécution (patron
`reconciliation-link.spec.ts`), fixture CAMT **synthétique** dérivée de `camt053_v04_minimal.xml` réécrite en `DBIT`,
helper neuf `createSupplierInvoiceViaApi` (`helpers/api-fixtures.ts`) :
1. fournisseur et facture fournisseur **ouverte** créés par l'API, avec une référence de paiement QR, datée et échue
   **plusieurs mois avant** le débit, **saisie** le jour du test (l'usage de Guy) ;
2. relevé importé, dont un débit du montant de la facture, portant la référence ;
3. *Réconciliation* → la ligne du débit propose la facture fournisseur (badge, fournisseur, numéro, montant) → *Accepter*
   → « 1 opération(s) réussie(s). », la ligne quitte la liste ;
4. fiche de la facture : **payée**, lien vers l'écriture de règlement ; grand livre / fiche d'écriture : lettrée ;
   détail de l'import : transaction rapprochée ;
5. détail de l'import → *Annuler le rapprochement* → le texte de confirmation fournisseur → confirmer → transaction à
   rapprocher, facture **ouverte** à nouveau ; la ligne revient dans la réconciliation.
Un second test : un **crédit** du même montant ne propose **pas** la facture fournisseur.
Captures **clair et sombre** (`page.emulateMedia`) de la ligne proposée et du dialogue d'annulation, dans
`KESH_E2E_CAPTURES_DIR`, hors dépôt ; une capture par schéma va au manuel (`docs/manual/fr/screenshots/`), si la
section en porte une.
Les E2E de la 30-1a (dialogue sur un état posé par l'API de la fiche + lien en SQL) ne sont **pas** requis : le
parcours de la 30-1b les couvre de bout en bout, et le dialogue est couvert par Vitest (C-30-1-17).

**AC-b13 — Les textes.**
- **Manuel utilisateur FR** (`docs/manual/fr/user-manual.tex`) et **son PDF** régénéré : § *Réconciliation bancaire*
  (`:2111-2113` « soit en réglant une facture client » → client **ou fournisseur**), § *Algorithme de matching*
  (`:2119` « créditrice, en CHF » ; un paragraphe pour le débit : facture fournisseur ouverte, montant **exactement**
  égal au total, référence de paiement QR, fenêtre 7 jours / 365 jours, lot de paiement en cours exclu, la facture
  presque au montant qui garde la règle), § *Accepter ou rejeter* (`:2140-2150` la candidate affichée, l'écriture
  `D créanciers / C banque`), § *Annuler un rapprochement* (`:2252`, le paiement fournisseur retiré, la facture
  redevenue ouverte), § *Régler une facture fournisseur* (`:1586`, « ou laisser la réconciliation la régler » ; la fiche
  dit où annuler un rapprochement). Les manuels DE/IT/EN : noter « à traduire » si l'écart existe (règle du dépôt).
- **`docs/api-external.md`** : `POST /reconciliation/accept` (`:572-580`, type `supplier_invoice`, codes et raisons
  neufs, `supplierInvoiceId` de `accepted[]`), `GET /reconciliation/proposals` (candidate `supplier_invoice`,
  `supplierInvoiceId`, `contactName`, la priorité), les **changements** (un débit reçoit désormais des factures).
- **i18n quatre locales** pour chaque clé neuve ou réécrite ; `npm run lint-i18n-ownership` vert.
- **CHANGELOG** `## [0.14.0] — Non publié`, `### Ajouté` : une entrée pour #686 (le processus, l'annulation, les limites
  — montant exact, fenêtre, lot exclu —, la mention API) ; aucun `closes`.
- **README** « Fonctionnalités », ligne de l'import bancaire : « réconciliation des débits avec les factures
  fournisseur » *(à paraître en v0.14.0)*.

**AC-b14 — Les registres du dépôt.**
`crates/kesh-api/tests/bank_account_closed_inventory.rs` : le nouvel écrivain de `bank_transactions`
(`accept_one_supplier_invoice`) au `REGISTRE` avec son statut (garde par `pay_in_tx`, W27), et, s'il porte une garde
non verrouillante, au `FENETRE` ; `audit_label_registry.rs` / `audit_labels.rs` si une action ou un champ d'audit
affiché change ; `idor_multi_tenant_e2e.rs` (facture fournisseur d'une autre société → `SUPPLIER_INVOICE_NOT_FOUND`,
indiscernable d'une facture absente) ; `rbac_e2e.rs` (rôle Consultation → `403`, inchangé) ; `textes_coherents.rs` si
une garde porte sur les textes touchés. Toute liste fermée de codes de `failed[]` (Rust ou TypeScript) recomptée.

**AC-b15 — Aucune migration.**
Ni table, ni colonne, ni index, ni contrainte : `ls crates/kesh-db/migrations/*.sql | wc -l` reste à **81**,
`kesh_version_min_required` à 0.13.1. Si le développement découvre qu'un index manque à la requête de l'AC-b3, il
l'**écrit** comme angle mort mesuré, il n'ajoute pas de migration sans retour à la fiche.

**Clés de `sprint-status.yaml`** (C-30-1-24) : la clé parente `30-1-reconciliation-factures-fournisseur` passe `split`
(patron 29-1, 29-4), deux clés filles `30-1a-annuler-rapprochement-fournisseur` et
`30-1b-proposer-accepter-fournisseur` passent `ready-for-dev`.

---

## Tasks / Subtasks

### 30-1a

- [ ] T0 — Brancher `story/30-1a` depuis `main` à jour ; relire tous les numéros de ligne de cette fiche (relevés sur
  `15686d55`).
- [ ] T1 (AC-a1, a2, a3) — **Tests rouges d'abord** : T-a1 à T-a5 dans `crates/kesh-db/tests/reconciliation_cancel.rs`
  (décor : facture fournisseur payée par `pay`, puis lien posé par un helper de fixture `link_bank_transaction_for_test`
  dans `test_fixtures.rs` — `UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ?`) ; constater le
  rouge (le bras `Entry` refuse : `OWNED_BY_…`).
- [ ] T2 (AC-a1, a4) — `ReconciliationKind::SupplierSettlement`, `classify`, doc-comments (ordre, coût de l'intervalle).
- [ ] T3 (AC-a3) — paramètre `unlinking` de `supplier_settlement_cancel_blocker` ; appelants à `None`.
- [ ] T4 (AC-a2) — bras de l'étape (6) ; audit.
- [ ] T5 (AC-a5) — route de la vue et de `…/cancel` ; T-a7, T-a8 (`crates/kesh-api/tests/reconciliation_e2e.rs` ou
  fichier dédié `reconciliation_supplier_e2e.rs`).
- [ ] T6 (AC-a4) — T-a6, course à deux connexions.
- [ ] T7 (AC-a6) — dialogue, types, texte de la fiche fournisseur, i18n ×4 ; Vitest T-a9, T-a10.
- [ ] T8 (AC-a7) — `api-external.md`.
- [ ] T9 — mutations M-a1 à M-a6 **sous le verrou de gate** ; gate complet et Vitest complet au dernier commit de code.

### 30-1b

- [ ] T10 — Brancher `story/30-1b` depuis `main` après le merge de la 30-1a ; relire les numéros de ligne.
- [ ] T11 (AC-b1) — tests rouges T-b1 à T-b3 ; traits, généricité, renommage ; tous les tests de `matching.rs` verts
  sans toucher leurs assertions.
- [ ] T12 (AC-b2, b3) — tests rouges T-b4 à T-b10 (`crates/kesh-db/tests/reconciliation_repository.rs`) ; fenêtre,
  requêtes.
- [ ] T13 (AC-b4, b5) — tests rouges T-b11 à T-b16 ; `get_proposals`, `candidate_rank`.
- [ ] T14 (AC-b6, b7, b8, b9) — tests rouges T-b17 à T-b23 ; `lock_for_bank_settlement_in_tx`, `pay_in_tx(actor)`,
  `accept_one_supplier_invoice`, `refuse_wrong_direction`, inventaire des refus recopié au Dev Agent Record.
- [ ] T15 (AC-b10) — T-b24, T-b25.
- [ ] T16 (AC-b11) — écran, libellés, types, Vitest T-b26 à T-b28.
- [ ] T17 (AC-b14) — registres.
- [ ] T18 (AC-b12) — E2E et captures ; helper `createSupplierInvoiceViaApi`.
- [ ] T19 (AC-b13) — manuel + PDF (`make fr` dans `docs/manual/`, sous le verrou), `api-external.md`, i18n, CHANGELOG,
  README.
- [ ] T20 — mutations M-b1 à M-b14 sous le verrou ; gate complet, Vitest complet **et E2E complet au dernier commit de
  code** (`flock`, recette de `docs/testing.md` à la lettre, port 3051, bases `kesh_301` / `kesh_e2e_301`).

---

## Dev Notes

### La conception, en une phrase par geste

- **Proposer** : la même boucle que pour un crédit, alimentée par d'autres candidates (factures fournisseur ouvertes au
  montant exact), notée par la **même** fonction générique, ordonnée par la **même** priorité.
- **Accepter** : `pay_in_tx` — le geste de la fiche, **appelé** — précédé d'un verrou et de gardes propres au
  rapprochement, suivi du lien de la transaction : ce que fait `accept_one_invoice` pour le client, sans ses étapes de
  reste dû, de trop-perçu et d'arrondi, qui n'existent pas côté fournisseur.
- **Annuler** : le dé-rapprochement existant, avec un troisième bras qui **appelle** `cancel_settlement_in_tx`.

### Pourquoi réutiliser `pay_in_tx` et non écrire un « accept_one_supplier » complet (C-30-1-10)

Le règlement fournisseur a **une** définition, éprouvée par la fiche et par les lots : dette lue sur l'écriture
d'achat, contrepartie gardée, compte clos lu sous verrou, exercice, lettrage. L'écrire une seconde fois dans la route
créerait le jumeau que le dépôt combat partout (« appelé, jamais réécrit »). Ce que la route ajoute n'existe pas dans
`pay_in_tx` : le sens de la transaction, la fenêtre, l'égalité de montant, la garde verrouillante du lot, le lien.

### Le montant (C-30-1-5)

Pas de reste dû : le règlement est binaire, du TTC lu sur l'écriture d'achat (`purchase_payable_line`), écrit des
**deux** côtés (`D créanciers / C banque`). La ligne de banque doit égaler le relevé — c'est la raison d'être de
l'écriture de règlement (patron 24-2). Donc **égalité exacte** `|tx.amount| = total_amount`. Un TTC à plus de deux
décimales n'est jamais proposé (angle mort L2). ⚠️ `pay_in_tx` lit le montant sur l'écriture d'achat, la proposition
sur `total_amount` : les deux sont égaux par construction (`generate_purchase_journal_lines`, `:155-161`) — un test
(T-b17) l'asserte sur la facture réglée.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `routes/reconciliation.rs` : le chemin crédit de `get_proposals` **inchangé** à l'octet près en comportement (les
  tests des Stories 28-7, 29-2, 29-3, 29-5, 29-6, 29-9 restent verts sans retouche) ; le pré-vol par lot (27-4), le
  compte clos (28-4a), le rejeu (`retry_with`, 25-4-c2) ; `accept_one_invoice` **inchangé** hors l'appel de
  `refuse_wrong_direction`.
- `matching.rs` : le score client, au bit près (l'égalité « 0,40 + 0,10 contre 0,50 » de la 29-5 en dépend).
- `supplier_invoices.rs` : `pay` et `confirm_batch` au comportement inchangé (seule la signature de l'acteur change) ;
  `in_generated_batch` non verrouillant pour les vues.
- `reconciliation_cancel.rs` : l'ordre « aucune lecture non verrouillante avant l'étape 3 » ; la revérification
  verrouillante en dernière acquisition (29-4b).

### Fichiers communs avec la 30-2

`supplier_invoices.rs` (la 30-2 crée des factures depuis un document, elle ne touche pas au paiement — à confirmer au
rebase), manuel § *Factures fournisseurs*, CHANGELOG, catalogues i18n, `helpers/api-fixtures.ts`. Union au rebase.

### Tests (récapitulatif)

30-1a : T-a1 à T-a10 (8 Rust, 2 Vitest). 30-1b : T-b1 à T-b28 (25 Rust, 3 Vitest) et l'E2E de l'AC-b12 (deux tests).
Décors **synthétiques** ; aucune donnée réelle (`docs/agents-orchestration.md` § 8). Concurrence par
`sonde_verrou_nowait` / `attendre_une_requete_en_cours` (`test_fixtures.rs:561`, `:611`), **jamais**
`information_schema.INNODB_TRX` (piège connu des consignes).

### Mutations (à exécuter sous `flock`, restaurées par `git checkout -- f && touch f`)

| # | Mutation | Test qui doit la tuer |
|---|---|---|
| M-a1 | `classify` sans le bras fournisseur | T-a1, T-a2 |
| M-a2 | bras fournisseur classé avant `invoice_settlements` (inversion) | T-a1 (règlement client) — sinon déclarer équivalente et le dire |
| M-a3 | `cancel_blocker` fournisseur en forme **non** exemptée (`None`) | T-a5 |
| M-a4 | étape (6) fournisseur par `reverse_in_tx` au lieu de `cancel_settlement_in_tx` | T-a2 |
| M-a5 | lecture de classement fournisseur **sans** `FOR UPDATE` | T-a6 |
| M-a6 | dialogue : texte client pour `supplier_settlement` | T-a9 |
| M-b1 | `propose_matches` fournisseur sans la référence de paiement | T-b1 |
| M-b2 | référence de paiement par containment au lieu d'égalité | T-b2 |
| M-b3 | blancs non retirés | T-b1 |
| M-b4 | borne haute `+1` au lieu de `+7` (ou inverse) | T-b4 |
| M-b5 | borne basse sur `F` au lieu de `X` | T-b4, T-b6 |
| M-b6 | filtre de lot `generated` retiré de la requête | T-b8 |
| M-b7 | égalité de montant remplacée par la tolérance client | T-b9 |
| M-b8 | appartenance au lot par lecture **ordinaire** | T-b22 |
| M-b9 | débit envoyé aussi aux factures client (garde #482) | T-b11 |
| M-b10 | crédit envoyé aussi aux factures fournisseur | T-b12 |
| M-b11 | `candidate_rank` : fournisseur en rang 2 | T-b14 |
| M-b12 | « presque au montant » ne masque pas la règle | T-b16 |
| M-b13 | `refuse_wrong_direction` : `is_sign_negative()` | T-b21 (`-0.00`) |
| M-b14 | `pay_in_tx` audit par `user` au lieu de `for_actor` | T-b23 |

Une mutation qui survit se tue par un test ajouté, ou se déclare **équivalente** avec la raison, au Dev Agent Record.

### Angles morts assumés (à écrire aux doc-comments et au manuel)

- **L1 — Débit déjà payé depuis la fiche** : la facture est `paid`, non proposée ; la transaction reste en attente, et
  une règle d'affectation peut la proposer — accepter la règle compterait la charge deux fois. Tracé par **#671**
  (première puce) ; le manuel le dit (« réglez la facture par la réconciliation plutôt que depuis la fiche »).
- **L2 — TTC à plus de deux décimales** : jamais proposé ; la facture se règle depuis la fiche. Rare (prix unitaire à
  plus de deux décimales). Issue demandée (§ *Issues à ouvrir*).
- **L3 — Écart de frais bancaires** (débit = TTC + frais) : non proposé, la règle masquée si l'écart ≤ 0,05 et un
  indice, sinon rien. Hors périmètre (pas de règlement partiel fournisseur).
- **L4 — Le nom de contrepartie d'un débit** (`camt053/mod.rs:459-474`) : non vérifié sur relevé réel ; poids 0,10.
- **L5 — `paid_at`** = heure de la saisie (C-30-1-19).

### Pièges connus du dépôt, applicables ici

- `kesh-db` touché : **gate complet même en boucle** de revue ; base de gate remise à zéro avant tout gate complet.
- P8 : aucune migration modifiée (aucune migration du tout).
- Mutations : `touch` après restauration (cargo compare des dates).
- Codes neufs plutôt que réemploi (C-30-1-18) ; `refuse_wrong_direction` (C-30-1-21) ; garde du lot verrouillante
  (C-30-1-22) ; traits génériques (C-30-1-23) ; acteur de `pay_in_tx` (C-30-1-20).
- E2E : `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés, les quatre `KESH_SMTP_*`,
  `KESH_ADMIN_BACKUP_DIR` inscriptible.
- Manuel : contrôler le **PDF** aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, sortie standard seulement).
- Commits : `refs #686`, jamais `closes`/`fixes`, ni « validation close » (écrire « validation terminée »).
- « Recompter ses comptes rendus » : nombres de tests, de mutations, de codes, avec leur périmètre.

### Issues à ouvrir (demandées à l'orchestrateur)

1. **`paid_at` d'une facture fournisseur = heure de la saisie, pas date du paiement** (`supplier_invoices.rs:907`) —
   P3 (la fiche affiche une date fausse pour un paiement antidaté ; aucune écriture fausse). Vu à la fiche 30-1.
2. **TTC fournisseur à plus de deux décimales** : un paiement depuis la fiche écrit une ligne de banque à quatre
   décimales ; la réconciliation ne le propose jamais (L2) — P3.

### Project Structure Notes

Aucun fichier neuf hors tests, fixture E2E et spec E2E ; aucun crate neuf ; aucune dépendance ajoutée.

### References

- Issue #686 ; fichier d'epic § *30-1* ; registre `epic-30-choix-autonomes.md` (C-30-0-1 à 3).
- Fiches : `28-6-*` (#482, sens), `28-7-*` (fenêtre), `29-2-paiement-tardif.md`, `29-3-meilleure-proposition.md`,
  `29-5-departage-exigibilite.md`, `29-6-trop-percu-centimes.md`, `29-8-transaction-reglee-ailleurs.md`, `25-3-*`
  (annulations), `29-4-cloture-compte-concurrente.md` (revérification).
- `CLAUDE.md` § *Pattern batch*, § *Un appariement automatique propose, il ne crée jamais*, § *Règle de splitting
  préventif*, § *Test Locally First*.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche créée par analyse exhaustive du code (`bmad-create-story`, mode création, autonomie) — guide de développement
  complet.

### File List

---

## Change Log

### Création — 2026-10-15

Fiche créée (Opus 5.5) sur `15686d55`. Prémisses de #686 établies : quatre exactes, une à préciser (pas de reste dû
fournisseur), une **fausse** (le dé-rapprochement ne sait pas défaire un règlement fournisseur) — d'où le découpage
30-1a (annulation d'abord) → 30-1b. #671 laissée ouverte (non mécanique). 22 AC (7 + 15), 21 tâches (T0 à T20), 38
tests nommés (T-a1 à T-a10, T-b1 à T-b28) plus un E2E de parcours à deux tests, 20 mutations (6 + 14). Choix C-30-1-1 à
C-30-1-24 (recomptés par `grep` sur cette fiche et le registre).
