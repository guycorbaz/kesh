# Story 30.1 : La réconciliation règle les factures fournisseur

## Status

**ready-for-dev** — fiche créée le 2026-10-15 en autonomie (`bmad-create-story`, Opus 5.5), sur `15686d55` (kickoff de
l'Epic 30), branche `story/30-1`, worktree `/home/gcorbaz/devel/kesh-30-1`. **Découpée** (§ *Découpage*) en deux
sous-stories tenues dans cette fiche, comme la 29-1 et la 29-4 : **30-1a** (le dé-rapprochement sait défaire un
règlement fournisseur) **puis** 30-1b (proposer et accepter). Choix consignés au registre propre à la story,
`epic-30-choix-autonomes-301.md`, **C-30-1-1 à C-30-1-41** (à verser par union au registre de l'epic). Validation P1
faite (Sonnet : F 0/2/4/5, R 0/2/7/9), remédiée le 2026-10-15 ; validation P2 faite (Opus : F 0/1/5/11, R 0/0/5/16),
remédiée le 2026-10-15 ; validation P3 faite (Sonnet : F 0/0/5/6, R 0/0/6/6), remédiée le 2026-10-15 ; validation P4 faite (Opus : F 0/0/7/6, R 0/1/5/7),
**simplifiée** le 2026-10-15 (signal de recyclage déclaré) ; validation P5 faite (Sonnet : F 0/1/4/4, R 0/0/2/6),
remédiée le 2026-10-15 ; validation P6 **ciblée** (Haiku) à lancer : `30-1-validate-prompt-p6-ciblee.md`.

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
| Entité : `status` `open` / `paid` / `cancelled` ; `total_amount` = **TTC**, `DECIMAL(19,4)` ; `supplier_invoice_number` (libre, nullable) ; `payment_reference` (`VARCHAR(64)` : QRR/SCOR à l'import d'un QR, **texte libre** à la saisie manuelle — corrigé en P2, R3) ; `invoice_date`, `due_date` ; colonnes `settlement_*` posées au paiement ; **aucune colonne de devise** (mono-CHF) | `entities/supplier_invoice.rs:17-48` ; migration `20260628000001_supplier_invoices.sql:27-66` |
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
| « *Accepter* … exactement ce que fait le règlement depuis la fiche par virement » | **Exacte à une réserve près** : `pay_in_tx` pose `paid_at` à l'heure de la saisie, pas à la date du paiement — défaut du chemin même qu'emprunte *Accepter*, **corrigé par la 30-1b** (AC-b16, C-30-1-25) |
| « montant égal au **reste dû** » | **À préciser** : il n'y a **pas** de reste dû fournisseur ; le règlement est binaire, du TTC entier (C-30-1-5) |
| « *Annuler le rapprochement* défait le règlement, comme côté client » | **Fausse aujourd'hui** : `classify` ne connaît que `invoice_settlements` ; une écriture de règlement fournisseur rapprochée serait classée `Entry`, et `reverse_in_tx` refuserait une écriture possédée par un règlement fournisseur — **défait par rien** (C-30-1-2 ; c'est l'objet de la 30-1a) |
| « une seule proposition affichée, l'ordre de priorité écrit (#527) » | **Exacte**, à étendre (`candidate_rank`, AC-b5) |

---

## Découpage (règle de splitting préventif) — C-30-1-3, recompté en P1 (C-30-1-27)

La story touche **plus de cinq modules** : `kesh-reconciliation/matching`, `kesh-db/repositories/reconciliation`,
`kesh-db/repositories/supplier_invoices`, `kesh-db/repositories/reconciliation_cancel`,
`kesh-api/routes/reconciliation`, `kesh-db/repositories/payment_batches` et `kesh-api/routes/payment_batches` (la
création d'un lot : verrous et rejeu, AC-a8, AC-a9),
`frontend/src/lib/features/reconciliation` (plus les catalogues i18n, le manuel et `api-external.md`). Découpée, dans
**cet ordre**, sur le patron « story-zéro puis rollout » :

| Sous-story | Clé | Objet | Modules de code |
|---|---|---|---|
| **30-1a** | `30-1a-annuler-rapprochement-fournisseur` | le dé-rapprochement sait **défaire un règlement fournisseur** (classement, motifs exemptés, annulation appelée), la vue le dit ; préalables de la 30-1b : le texte commun « rapproché » rendu vrai des deux côtés, la création d'un lot rejouée sur interblocage **et** verrouillant ses factures avant toute lecture (AC-a9, défaut préexistant intégré) | `repositories/reconciliation_cancel`, `repositories/supplier_invoices` (une fonction de lecture ajoutée), `repositories/payment_batches` (`create_batch`), `routes/reconciliation` (vue et `…/cancel`), `routes/payment_batches` (une enveloppe de rejeu) — **5** |
| **30-1b** | `30-1b-proposer-accepter-fournisseur` | la proposition et l'acceptation des factures fournisseur pour un débit, `paid_at` à la date du paiement ; le dialogue *Annuler le rapprochement* pour un règlement fournisseur | `kesh-reconciliation/matching`, `repositories/reconciliation`, `repositories/supplier_invoices` (chemin de paiement), `routes/reconciliation`, `features/reconciliation` (`ReconciliationProposals`, `CancelReconciliationDialog`, libellés) — **5** |

Le dialogue *Annuler le rapprochement* passe de la 30-1a à la 30-1b (C-30-1-33) : rien ne produit un rapprochement
fournisseur avant la 30-1b, et sans ce déplacement l'intégration de l'AC-a9 portait la 30-1a à six modules. Hors
décompte, dans les deux : `kesh-i18n` (catalogues), `kesh-api/src/errors.rs` (un texte de repli),
`frontend/src/lib/shared/utils/settlement-cancel-blocked.ts` (un texte de repli, P2 R-L8), les tests et
`helpers/api-fixtures.ts`. Chaque sous-story est **au seuil** (5), non au-delà : signal déclaré au Project Lead, sans
nouveau découpage (C-30-1-27).

**Pourquoi cet ordre, et non l'inverse.** Livrée la première, l'acceptation produirait sur `main` des transactions
rapprochées d'un règlement fournisseur que **rien ne défait** (prémisse fausse ci-dessus) — l'état même que la
25-3-b a fermé pour le client (« l'écriture était définitivement incorrigible »). L'annulation d'abord : à aucun
commit de `main` un tel état n'est produisible sans être défaisable. La 30-1a se teste sur un état posé par la
fixture (règlement depuis la fiche, puis lien posé en SQL), que la 30-1b rejouera par l'acceptation. De même, le rejeu
de `create_batch` précède le chemin (30-1b) qui ouvre le cycle qu'il rattrape (AC-b10).

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
(patron `{ accepted, failed }`) ; défaire l'acceptation par *Annuler le rapprochement* ; la priorité écrite étendue,
avec la règle d'affectation **masquée** quand une facture fournisseur saisie la rendrait fausse ; `paid_at` à la date
du paiement ; les textes (écran, manuel, `api-external.md`, i18n ×4, CHANGELOG).

**Dehors** (chacun nommé, aucun ne s'intègre en silence) :

- **#671** (C-30-1-4) — rattacher un débit à un règlement fournisseur déjà saisi.
- **Un débit jamais rapproché d'une facture client, un crédit jamais d'une facture fournisseur** : c'est une garde,
  pas un dehors (AC-b4, AC-b7).
- **Avoirs fournisseur, remboursement d'un fournisseur (crédit)** : aucun modèle d'avoir fournisseur n'existe.
- **Paiement partiel d'une facture fournisseur** : le règlement est binaire (C-30-1-5).
- **Un paiement qui règle plusieurs factures fournisseur** (#671, troisième puce).
- **Les lots pain.001** : un lot confirmé règle ses factures par `pay_in_tx` ; son débit au relevé relève de #671
  (quatrième puce). La 30-1b ne propose **jamais** une facture engagée dans un lot `generated` (AC-b3), et une telle
  facture au montant exact **bloque** la règle avec une raison visible (AC-b5).
- **TTC fournisseur à plus de deux décimales** — angle mort L2, tracé par **#687** (C-30-1-26).
- **Le dialogue *Annuler le rapprochement* sur la fiche fournisseur** (symétrique de `InvoiceSettlements.svelte`) :
  non (C-30-1-11) ; le texte commun dit où le faire (AC-a6).
- **Attribution du dé-rapprochement à la clé d'API** : `cancel_settlement_in_tx` (fournisseur) comme
  `invoice_settlements_write::cancel_settlement_in_tx` (client) n'écrivent que l'utilisateur ; seule l'acceptation porte
  la clé (AC-b9). Asymétrie **préexistante et identique des deux côtés**, nommée (`api-external.md:366` le dit pour le
  client).
- **Garde de devise à l'acceptation fournisseur** : aucune, comme `accept_one_invoice` (parité) ; les transactions non
  CHF n'ont aucune candidate (`routes/reconciliation.rs:743-746`) et aucun import n'en enregistre depuis la 27-2.

---

## Critères d'acceptation

### Partie a — `30-1a-annuler-rapprochement-fournisseur`

**AC-a1 — Le classement connaît le règlement fournisseur.**
`ReconciliationKind` gagne `SupplierSettlement { supplier_invoice_id: i64 }`, de code canonique `supplier_settlement`
(`reconciliation_cancel.rs:46-67`). `classify` (`:75-104`) cherche, **après** `invoice_settlements` et **dans la même
forme verrouillante ou non** que lui, `SELECT id FROM supplier_invoices WHERE company_id = ? AND
settlement_journal_entry_id = ? AND status = 'paid'` (avec `FOR UPDATE` si `for_update`). Ordre des bras : règlement
client, puis règlement fournisseur, puis `Entry`. Une écriture de règlement **détachée** (facture annulée, colonnes à
`NULL`) reste `Entry` (C-30-1-6) ; `ModificationGuard::DetachedSupplierSettlement` ne couvre que la modification et la
suppression, pas la contre-passation (vérifié en P1, F).
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

**AC-a3 — Les motifs, en forme exemptée, par une fonction ajoutée.**
Une fonction **ajoutée**, `supplier_settlement_cancel_blocker_unlinking(conn, company_id, id, unlinking: Option<i64>)`,
patron exact de `invoice_settlements_write::settlement_cancel_blocker_unlinking` (`invoice_settlements_write.rs:752`) ;
`supplier_settlement_cancel_blocker` (`supplier_invoices.rs:1251-1280`) **garde sa signature** et devient son
enveloppe (`None`) — ses six appelants (`supplier_invoices.rs:1365`, `:1536` et quatre tests) ne changent pas.
`reconciliation_cancel::cancel_blocker` (`:148-182`) appelle pour le bras fournisseur la forme exemptée pour `bt.id`.
Le geste refuse lui-même, comme pour le client, `FiscalYearClosed` et `DocumentLetteringInClosedPeriods` en
`ReconciliationNotCancellable` **avant** de défaire le lien ; les autres rangs sont refusés par le geste appelé ou par le
socle. ⛔ Aucune seconde précédence écrite.
*Tests* : T-a4 (exercice du règlement clos → `409`, rien écrit, lien intact), T-a5 (vue : `cancellable = true` sur un
règlement rapproché ouvert — la forme non exemptée rendrait `MATCHED_BANK_TRANSACTION`, mutation M-a3).

**AC-a4 — Verrous et naissance de la vue.**
Ordre du geste pour le bras fournisseur : verrou nommé (route) → `bank_transactions` (`t`) X → `supplier_invoices` X
(par l'index de `settlement_journal_entry_id`, `fk_supplier_invoices_settlement_je`) → `journal_entries` + `fiscal_years`
X → garde non verrouillante du compte clos → … → revérification verrouillante du compte (dernière acquisition). ⛔
**Aucune lecture non verrouillante avant l'étape 3**, y compris pour le classement fournisseur. Le bras fournisseur
verrouille la facture **avant** l'écriture, comme `cancel_settlement_in_tx` (facture → écriture) : même ordre que le
geste qu'il appelle.

- **Plan épinglé** : la lecture verrouillante doit passer par l'index de `settlement_journal_entry_id` ; un plan par
  `idx_supplier_invoices_company_status` verrouillerait **toutes** les factures payées de la société. Le test T-a12
  asserte le plan (`EXPLAIN`, patron des tests de plan du dépôt) ; à défaut d'un plan stable, `FORCE INDEX` écrit et
  justifié.
- ⚠️ **Coût assumé et écrit au doc-comment** : le bras fournisseur n'est lu qu'après un échec du bras client, donc
  seulement pour un rapprochement `Entry` (ou fournisseur) ; sur un `Entry`, la lecture qui ne trouve rien pose un
  **verrou d'intervalle** sur `fk_supplier_invoices_settlement_je`. Un `Entry` récent (identifiant supérieur à tous les
  `settlement_journal_entry_id`) tient l'intervalle de queue : un `pay_in_tx` concurrent (étape 6, `UPDATE … SET
  settlement_journal_entry_id = <écriture neuve>`) **attend** la fin du dé-rapprochement. Même coût que la lecture de
  `invoice_settlements` sur un `Entry` aujourd'hui. Cycle possible avec le verrou de l'exercice (`pay_in_tx` tient
  l'exercice par `find_open_covering_date … FOR UPDATE`, `fiscal_years.rs:758`, avant son `UPDATE` ; le
  dé-rapprochement tient l'intervalle avant l'exercice) : **rejoué des deux côtés** (`retry_app_on_deadlock` pour le
  dé-rapprochement, `retry_on_deadlock` pour `pay`, `retry_with` pour l'acceptation, `retry_on_deadlock` pour `confirm`).
*Tests* : T-a6 (course : dé-rapprochement d'un règlement fournisseur rapproché contre `supplier_invoices::cancel_in_tx`
de la **même** facture, qui **détache** le règlement — l'un attend l'autre au verrou de la facture ; issues admises :
dé-rapprochement puis annulation de la facture `open`, ou annulation (règlement détaché, transaction toujours
rapprochée) puis dé-rapprochement classé `Entry` qui contre-passe le règlement détaché ; jamais d'état mixte, rien
d'écrit par le perdant s'il est refusé. ⚠️ Réorienté en P2 (F-L9) : la course contre `cancel_settlement_in_tx` était
verte par construction, la fiche refusant toujours au rang 3 ou au rang 1), **T-a11** (dé-rapprochement d'un règlement
fournisseur rapproché **pendant** `fiscal_years::close` de l'exercice du règlement, sonde
`attendre_une_requete_en_cours` : refus `ReconciliationNotCancellable` (exercice clos), rien écrit — c'est lui qui tue
M-a5, la lecture de classement sans `FOR UPDATE` qui ferait naître la vue avant le verrou de l'exercice), T-a12 (plan).
Sondes `sonde_verrou_nowait` / `attendre_une_requete_en_cours`, **jamais** `INNODB_TRX`.

**AC-a5 — La vue et la réponse disent le fournisseur.**
`GET /reconciliation/transactions/{id}` : `kind` peut valoir `supplier_settlement` ; deux champs **ajoutés**,
`supplierInvoiceId` et `supplierInvoiceNumber` (nuls hors de ce cas) ; `invoiceId` / `invoiceNumber` restent ceux d'une
facture **client** (nuls pour un règlement fournisseur). `POST …/cancel` : champ ajouté `supplierInvoiceId`. Audit
`reconciliation.cancelled` : `kind = "supplier_settlement"`, `supplierInvoiceId` ajouté (`invoiceId` et `settlementId`
nuls). Contrat **additif** : aucun champ existant ne change de sens (C-30-1-8).
*Tests* : T-a7 (forme JSON des trois cas), T-a8 (audit).

**AC-a6 — Le texte « rapproché », vrai des deux côtés.**
Le motif `MATCHED_BANK_TRANSACTION` de l'annulation d'un règlement a **un** texte, commun au client et au fournisseur,
lu par l'écran (`frontend/src/lib/shared/utils/settlement-cancel-blocked.ts:51`, `messages.ftl:851` et ses trois
jumeaux `:803`) — c'est par la **vue** de la fiche (`settlementCancelBlockedBy`) que l'utilisateur le voit. Il dit
aujourd'hui « depuis **cette fiche** ou le détail de l'import », faux sur la fiche fournisseur, qui n'a pas le dialogue.
**Réécrit** (C-30-1-11 amendé) : « Ce règlement est rapproché d'une transaction bancaire : annulez d'abord le
rapprochement, depuis le détail de l'import bancaire — ou, pour une facture client, depuis sa fiche. » — quatre
locales, repli de l'écran, et repli serveur `kesh-api/src/errors.rs:3736-3739` aligné. ⚠️ Ce bras serveur de
`SettlementNotCancellable` est **inatteignable** (`errors.rs:3703` : « Seuls les rangs que le GESTE refuse lui-même
arrivent ici » ; les deux `cancel_settlement_in_tx` ne refusent eux-mêmes que les rangs 1, 2 et 2 bis) : au clic, le
rang 3 vient du socle, texte `journal-entries-reverse-blocked-bank-match`, déjà vrai. Le repli serveur est aligné
pour que les deux replis du même code ne divergent pas (P2, R-L2) — non parce qu'un client d'API le lirait. La clé est
**ajoutée** à `les_replis_rust_suivent_le_catalogue` (`textes_coherents.rs:679-700`), qui compare les replis de
`errors.rs` au catalogue mais ne la porte pas aujourd'hui. Aucune clé propre au fournisseur : le test « la queue est
RÉUTILISÉE » (`settlement-cancel.test.ts:22-32`) reste vrai tel quel. Le commentaire `errors.rs:4237-4238` (« Aucun
rapprochement ne règle une facture fournisseur : le motif ne peut pas naître ici ») devient faux par sa raison :
**réécrit** — le motif reste inatteignable parce que le classement fournisseur exige `status = 'paid'` (P2, R-L1).
Le **dialogue** *Annuler le rapprochement* et son texte fournisseur relèvent de la 30-1b (AC-b11, C-30-1-33).
*Tests* : Vitest T-a10 (le texte commun, sur la fiche fournisseur et la fiche client), T-a13
(`textes_coherents.rs::les_replis_rust_suivent_le_catalogue`, clé ajoutée : repli serveur = catalogue fr-CH).

**AC-a7 — Contrat écrit.**
`docs/api-external.md` § *Annuler un rapprochement* (`:596-612`) : `kind = supplier_settlement`, les champs ajoutés,
« pour une facture fournisseur, retire son paiement » ; la mention « depuis la v0.14.0 » ; `:788` (AC-a8) ; `POST /payment-batches` : la borne de 500 identifiants
(`400`, compte sur la longueur **brute** de `supplierInvoiceIds`, avant dédoublonnage — P4 R13) et l'absence de lot pour
une liste vide (AC-a9), écrites au paragraphe qui mentionne déjà `POST /api/v1/payment-batches` (`api-external.md:554`,
P4 R8). **CHANGELOG** `## [0.14.0] — Non publié`, écrit par la 30-1a (P4 R4) : `### Corrigé`, deux créations de lot
simultanées ne mettent plus la même facture fournisseur dans deux lots en cours (deux fichiers pain.001, un double
paiement) ; la création d'un lot est rejouée sur interblocage ; `### Modifié`, 500 factures au plus par lot (`400`). Le
manuel et l'entrée #686 sont écrits par la 30-1b (rien ne produit un rapprochement fournisseur avant elle ; C-30-1-9).

**AC-a8 — La création d'un lot de paiement est rejouée sur interblocage (préalable de la 30-1b, C-30-1-28).**
`POST /payment-batches` (`kesh-api/src/routes/payment_batches.rs:159`) passe par
`kesh_db::retry::retry_on_deadlock("payment_batches::create", …)`, comme `confirm` (`:251`). Motif : la 30-1b ouvre un
cycle acceptation ↔ `create_batch` inhérent à un lot d'acceptation de plusieurs propositions (AC-b10, cycle C1) ; sans
rejeu, la victime `create_batch` finirait en `500` sans rapport visible. Rejeu **sûr**, vérifié en P2 (F5, R) : tout est
dans une transaction (`pool.begin()` … `commit()`), le `msg_id` dérive de l'identifiant dans la transaction, aucun
fichier n'est écrit (le pain.001 est produit au `GET`), `NewPaymentBatch` est `Clone` — une tentative ne rend `Err`
qu'annulée, aucun double lot.
**Textes et registres à mettre en accord** (P2, R2 et F5) :
- `docs/api-external.md:788` (§ *Interblocages*) dit « L'import d'un relevé (`POST /bank-imports`) et la création d'un
  lot de paiement **ne sont pas rejoués** » : retirer la création d'un lot de cette phrase et ajouter `POST
  /payment-batches` aux routes qui rejouent (« depuis la v0.14.0 ») ; il n'existe **pas** de « § lots » ;
- `crates/kesh-api/tests/audit_route_registry.rs`, **quatre sites** du doc-comment (P3, F7) : la route (`:360`,
  `SansEcritureAuJournal`, classification inchangée — `the_replay_partition_is_what_the_story_declares` ne bouge pas)
  entre au point **(vi)** (`:152`, « **quatorze** routes `SansEcritureAuJournal` sont rejouées » → **quinze**) et à sa
  phrase finale (`:228`, « Quatorze avec elle. » → quinze) ; prouvée par T-a14, elle entre aux preuves dynamiques
  **(ii)** (`:57-60`, « pour sept routes de la famille `DbError` » → **huit**) et **(iii bis)** (`:85`, « les sept routes
  des tests 2 à 5, 7, 9 et 10 » → huit, avec le test neuf).
*Test* : **T-a14**, sur le patron `bank_account_creation_is_replayed_when_it_is_the_deadlock_victim`
(`crates/kesh-api/tests/rejeu_interblocage_e2e.rs:2089`, aide `victime` `:347`) : la création d'un lot victime d'un
interblocage est rejouée, et **un seul** lot existe à la fin. Mutation M-a8 (enveloppe retirée).

**AC-a9 — Deux créations de lot ne mettent plus la même facture dans deux lots `generated` (défaut préexistant,
intégré sur décision de l'orchestrateur, C-30-1-31).**
Au sol, `create_batch` (`payment_batches.rs:69-200`) fait d'abord une lecture **ordinaire** du compte (étape (0),
`:92` — la vue naît), puis, par facture, `FOR UPDATE` (`:321`) et l'appartenance à un lot `generated` par une lecture
**ordinaire** (`:340-348`). Deux `create_batch` concurrents sur la même facture : le second attend le verrou de la
facture, puis lit l'appartenance dans **sa** vue, née avant le commit du premier — il ne voit pas l'article, et la
facture entre dans **deux** lots `generated`, deux fichiers pain.001 : **double paiement** (P2, F-L11 = R-L7).
**Correction** : avant l'étape (0), `create_batch` verrouille **toutes** les factures demandées, dédoublonnées, **par
identifiant croissant** (`SELECT id FROM supplier_invoices WHERE id IN (…) AND company_id = ? ORDER BY id FOR UPDATE`) ;
- **liste vide** (après dédoublonnage) : **aucun** `SELECT` — `IN ()` est une erreur de syntaxe en MariaDB et rendrait
  `500` là où la route rend aujourd'hui `200` `{ batch: null, failed: [] }` (`payment_batches.rs:120`, boucle vide ; P3,
  F3 = R1) ; le court-circuit garde la sortie d'aujourd'hui. La taille de la liste n'a pas de borne à la route
  (`routes/payment_batches.rs:133`) : au-delà de 65 535 identifiants, la limite des paramètres liés rendrait `500` —
  borne de **500** identifiants ajoutée à la route (`400 VALIDATION_ERROR`, patron `MAX_IN_CLAUSE_IDS`,
  `repositories/reconciliation.rs:747`), écrite à `api-external.md` ;
- **plan épinglé** (P3, F4 = R7) : les verrous InnoDB suivent l'ordre du **parcours**, pas l'`ORDER BY` ; un plan par
  `idx_supplier_invoices_company_status` verrouillerait toutes les factures de la société, dans un autre ordre. Le plan
  attendu est un parcours de la clé primaire (`range` sur `PRIMARY`) ; T-a12 est **étendu** à cette requête
  (`EXPLAIN` asserté) ; à défaut d'un plan stable, `FORCE INDEX (PRIMARY)` écrit et justifié ;
l'étape (0) et les lectures d'appartenance viennent **après** : la vue naît après le dernier verrou de facture, et voit
tout lot commité par un concurrent qui tenait l'une d'elles. L'ordre croissant supprime aussi l'interblocage de deux
lots aux factures croisées. L'ordre des verrous reste facture → compte (clé étrangère de l'`INSERT payment_batches`) →
articles, celui de la table de l'AC-b10. Sorties inchangées : `failed[]` garde l'ordre de la requête et ses codes
(`ALREADY_IN_GENERATED_BATCH` pour le second). ⚠️ Prix, écrit exactement au doc-comment (P4, F-L3) : un identifiant
**absent** pose un verrou d'intervalle sur la clé primaire ; un identifiant **existant d'une autre société** pose un verrou
d'**enregistrement** sur sa ligne (parcours de `PRIMARY` en `REPEATABLE READ`) — préexistant, la boucle actuelle
(`payment_batches.rs:318-321`) faisant de même. **Mécanique** : un `SELECT` ajouté en tête, aucune
autre ligne déplacée que la première lecture du compte, qui passe après lui ; le doc-comment de l'étape (0) (« c'est
déjà sa première lecture ») est réécrit.
⚠️ Comme l'AC-b8, la garde repose sur « une lecture postérieure au dernier verrou voit le dernier état commité » :
vrai en MariaDB 10.11 (`innodb_snapshot_isolation = OFF`) ; l'angle mort 11.6+ est tracé par **#669** (P3, R9).
*Tests* (motifs de sonde **communs** au sol et après la correction — `FROM supplier_invoices` et `FOR UPDATE` —, faute de
quoi la sonde paniquerait au sol pour une autre raison que le défaut, P4 F-L2) : **T-a15** (course au niveau du dépôt, trois connexions : (1) une connexion tient la facture `FOR UPDATE` ;
(2) `create_batch` A démarre et se bloque sur la facture ; (3) `create_batch` B démarre et se bloque à son tour — la
sonde attend **deux** requêtes en attente sur la facture, et non une (P3, F2 : A et B attendent sur le **même** texte
préparé, et `attendre_une_requete_en_cours`, `test_fixtures.rs:561`, rend vrai dès la première) : aide de test
**ajoutée**, `attendre_des_requetes_en_cours(pool, motifs, au_moins, interrompre)`, ou paramètre `au_moins` ; **au
sol**, B a fait sa lecture ordinaire du compte **avant** de se bloquer, sa vue est née ; (4) la connexion (1) relâche, A
crée son lot et commite, B obtient la facture. Attendu : B refuse la facture (`ALREADY_IN_GENERATED_BATCH`, `batch:
None` si c'est la seule) ; **rouge au sol**, où B crée un second lot ; cas ajouté au niveau du dépôt : liste vide →
`batch: None`, `failed` vide, aucune requête de verrouillage), **T-a16** (deux lots aux factures **croisées**, au niveau
du dépôt **sans rejeu**, chorégraphie réalisable (P4, R5) : (1) une connexion C tient la facture 1 `FOR UPDATE` ; (2) A =
{1, 2} démarre et se bloque sur 1 **sans rien tenir** ; (3) B = {2, 1} démarre — **au sol**, B tient 2 et se bloque sur
1 —, sonde « deux requêtes en attente » ; (4) C relâche : A, premier en file, obtient 1, puis demande 2 tenu par B →
**1213 au sol**, rouge ; après correction, B trie {1, 2} et attend 1 sans rien tenir : **aucun 1213**. Hypothèse écrite :
l'octroi des verrous en file dans l'ordre d'arrivée en MariaDB 10.11. Mutation M-a9), **T-a17** (test de **route**,
`crates/kesh-api/tests/` — fichier des lots de paiement s'il existe, sinon neuf, P4 R7 : `supplierInvoiceIds: []` →
`200`, `batch: null`, `failed: []` ; 501 identifiants, comptés avant dédoublonnage → `400 VALIDATION_ERROR` ; 500 → la
route traite). Mutation M-a7 (le
verrouillage en tête retiré).

### Partie b — `30-1b-proposer-accepter-fournisseur`

**AC-b1 — Une seule notation pour les deux sortes de factures (DRY), montant en valeur absolue.**
- `kesh-db/src/repositories/reconciliation.rs` : un trait public `PayableDocument` (`document_id() -> i64`,
  `document_date() -> NaiveDate`, `due_date() -> Option<NaiveDate>`), implémenté pour `Invoice` (`date`, `due_date`) et
  `SupplierInvoice` (`invoice_date`, `due_date`) ; `invoice_exigibility_date` et `imputation_order` deviennent
  génériques sur ce trait, **sans changer leur résultat** pour une facture client (test d'accord SQL/Rust existant
  `exigibility_date_sql_and_rust_agree` inchangé et vert).
- `kesh-reconciliation/src/matching.rs` : `propose_matches` devient générique sur un trait `MatchableDocument:
  PayableDocument` qui fournit le **score de référence** du document contre la transaction.
- ⛔ **Le montant de la transaction est comparé en valeur absolue** dans la fonction générique (`amount_score(tx.amount.abs(),
  …)`, ou équivalent écrit au contrat du trait) : un débit est stocké **négatif** (`kesh-import/src/camt053/mod.rs:627`,
  `sign.apply`) et le TTC est positif ; comparé signé, `amount_score` vaudrait 0 pour tout débit (P1 : R1, F2). Le chemin
  crédit ne change pas (montants déjà positifs ; montant nul : aucune candidate, garde amont) : score client **au bit
  près**. ⚠️ Conséquence écrite (P2, F-L4) : la note ne distingue plus le sens ; la garde #482 repose sur les filtres
  amont (`routes/reconciliation.rs:749-752` et son jumeau fournisseur) et sur `refuse_wrong_direction` à l'acceptation —
  testés par T-b11, T-b12, T-b21. Un montant orienté par sorte de document a été considéré et écarté par l'arbitrage de
  la P1 (valeur absolue dans la fonction générique).
- **Référence, client** : `Invoice` garde **exactement** la règle actuelle (`coalesce(reference, end_to_end_id,
  transaction_id)` contre `invoice_number`, containment, demi-score de préfixe).
- **Référence, fournisseur** (C-30-1-12, amendé deux fois) : le **maximum** de deux **égalités normalisées**, jamais de
  containment ni de préfixe, contre le **même** `coalesce(reference, end_to_end_id, transaction_id)` de la transaction ;
  normalisation appliquée **aux deux côtés** : retrait de tous les blancs, majuscules ; chaque valeur ne compte que si
  elle a **au moins 6 caractères alphanumériques** après normalisation :
  (a) `payment_reference` égale → `1.0`. ⚠️ Ce n'est **pas** toujours une référence QRR/SCOR : c'est un **texte libre**
  à la saisie manuelle (`supplier-invoices/+page.svelte:331`, aucun contrôle à `POST /supplier-invoices`,
  `routes/supplier_invoices.rs:245`, `:348`) ; seul l'import d'un QR la garantit structurée
  (`imported_supplier_invoices.rs:211-216`). D'où la même borne que le numéro (P2, R3) ;
  (b) `supplier_invoice_number` égal → `1.0`.
  Une valeur plus courte (« 1 », « 12 », « Loyer ») → `0.0`.
  **Ce que cela garantit** — pour les indices de **référence** seulement : aucun ne naît d'une coïncidence partielle ou
  d'une valeur courte, donc aucun ne départage **contre l'ordre d'imputation** (art. 87 CO, 29-5) ; seule une désignation
  **entière** par le payeur (référence ou numéro complet d'au moins 6 caractères) fait passer une facture devant une
  autre de même montant : la déclaration du débiteur (art. 86 CO). ⚠️ L'indice de **contact** (0,10, containment
  bidirectionnel, `matching.rs:167-182`) départage encore deux factures de même montant de **fournisseurs différents**,
  comme côté client (P2, R-L12) ; entre deux factures du **même** fournisseur, il est égal et ne départage pas.
- `MatchProposal.invoice_id` est renommé `document_id` (champ interne) : le **type** n'est lu que par `matching.rs` et
  `lib.rs:33`, le **champ** aussi par `routes/reconciliation.rs:895` (`mp.invoice_id`, P2 F-L1), déjà dans le périmètre.
- **Tous** les tests existants de `matching.rs` passent **sans modification de leurs assertions** (les constructeurs
  de fixtures peuvent changer de signature).
*Tests* : T-b1 (sur une transaction **négative** : `amount_score = 1.0` ; référence de paiement de 27 chiffres avec et
sans blancs, en minuscules ; maximum des deux), T-b2 (préfixe commun de zéros, référence de paiement contenue → 0),
T-b3 (départage fournisseur par exigibilité), T-b37 (numéro **et** référence de paiement : égal et ≥ 6 → 1 ; égal et
< 6 → 0 ; contenu → 0 ; deux factures de même montant, l'une au numéro court contenu dans la référence : l'ordre
d'imputation tient).

**AC-b2 — La fenêtre de dates fournisseur, écrite et tranchée (C-30-1-13).**
Une seule définition étendue : `InvoiceDateWindow` gagne un constructeur `for_supplier_payment_date(d)` (et
`for_supplier_transaction(tx)`) ; `check` est **réutilisé tel quel**. Bornes, sur `D` = `payment_date(tx)` :

- **`F ≤ D + 7 jours`** (`SUPPLIER_PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS = 7`) — un achat payé en ligne dont la facture
  est émise à l'expédition, quelques jours après ;
- **`X ≥ D − 365 jours`** (`SUPPLIER_INVOICE_WINDOW_DAYS_AFTER_EXIGIBILITY = 365`), `X` = date d'exigibilité
  `max(F, échéance)` — un paiement fait jusqu'à un an après l'échéance est proposé, y compris à cheval sur deux
  exercices.
- ⛔ La fenêtre **ne lit jamais** `created_at` ni la date du jour : la facture peut être **saisie** des mois après le
  paiement — c'est l'usage de Guy, et c'est le cas nominal.
- Vérifié en P1 (F) : aucune contrainte fournisseur n'oppose un règlement daté avant la facture (aucun
  `chk_*paid_at*` sur `supplier_invoices`), `create_in_tx` ne compare pas les dates de deux écritures, le lettrage
  s'abstient en période close sans comparer l'ordre achat/règlement.

**Refus, une seule traduction (DRY, P1 F11)** : la conversion `InvoiceDateRefusal → FailedProposal` d'`accept_one_invoice`
(`routes/reconciliation.rs:1918-1937`) devient une fonction commune `date_refusal_failed_proposal(bank_transaction_id,
refusal, window)` ; `RECONCILIATION_INVOICE_NOT_ELIGIBLE` avec `details.reason` = `payment_date_before_invoice_date`
(+ `details.tolerance_days` : 1 client, 7 fournisseur — champ **ajouté**, additif) ou `payment_date_outside_window`
(+ `details.window_days` : 30 client, 365 fournisseur). Côté client, `reason` et `window_days` **inchangés**. Expression
SQL de `X` : `CAST(GREATEST(si.invoice_date, COALESCE(si.due_date, si.invoice_date)) AS DATE)` — le `CAST` du patron
client (`repositories/reconciliation.rs:180`, type rendu garanti `NaiveDate`, P2 R-L13) —, tenue d'accord avec la fonction Rust
par un test de parité (patron `exigibility_date_sql_and_rust_agree`).
*Tests* : T-b4 (bornes incluses et exclues des deux côtés, `due_date` nulle, échéance antérieure à la date), T-b5
(parité SQL/Rust), T-b6 (facture saisie aujourd'hui, datée et échue un an avant le paiement moins un jour → proposée).

**AC-b3 — Les candidates fournisseur, les factures qui masquent, et le lot du compte (C-30-1-5, C-30-1-14, C-30-1-32, simplifiés en P4 : C-30-1-39).**
`repositories/reconciliation.rs` : `find_open_supplier_invoices_for_window(executor, company_id, window, amount)` —
`amount` = valeur absolue du débit :
1. `company_id` ;
2. `status = 'open'` ;
3. **`total_amount = amount`** — égalité **exacte** en décimal (`DECIMAL(19,4)` contre `DECIMAL(18,2)` : comparaison
   exacte en MariaDB) ;
4. **jamais une facture engagée dans un lot `generated`** : `NOT EXISTS (SELECT 1 FROM payment_batch_items pbi JOIN
   payment_batches pb ON pb.id = pbi.payment_batch_id WHERE pbi.supplier_invoice_id = si.id AND pb.status =
   'generated')` ;
5. fenêtre de l'AC-b2 ;
6. ordre d'imputation (`X`, `invoice_date`, `id` croissants) **avant** `LIMIT 50`.

**Le lot du compte — une seule fonction source** (P4, F6, arbitrage de l'orchestrateur) :
`find_open_supplier_invoice_in_account_batch(executor, company_id, bank_account_id, amount) -> Option<(supplier_invoice_id,
supplier_invoice_number, payment_batch_id)>` — la facture fournisseur **ouverte**, au montant **exact**, engagée dans un
lot `generated` **dont `pb.bank_account_id` est le compte de la transaction**, la première de l'ordre d'imputation
(`LIMIT 1` : un test d'existence, rien n'est coupé). **Lecture ordinaire.** Ses trois lecteurs : la proposition (AC-b5,
première ligne), l'acceptation d'une facture (AC-b6, étape 3 (d)) et l'acceptation d'une règle (AC-b17). Aucun autre
texte ne décide de ce critère. *(Le paramètre `excluding` de la P4 est **retiré** en P5 — F4 = R1-3 : il n'avait pas
d'autre usage ; la facture acceptée, si elle est dans un lot, est déjà refusée par (c), en lecture verrouillante. Seul
effet de son retrait : sur une vue périmée — lot **annulé** après sa naissance —, (d) pourrait refuser la facture
acceptée en se nommant elle-même ; refus sûr, la proposition suivante se relit.)*

**La référence d'une facture déjà payée — une seule fonction source** (P5, F1 = R1-6, arbitrage de l'orchestrateur) :
`find_paid_supplier_invoice_by_reference(executor, company_id, tx_reference_normalized) -> Option<(supplier_invoice_id,
supplier_invoice_number)>` — une facture fournisseur **payée** (`status = 'paid'`) de la société dont la référence de
paiement **ou** le numéro est **égal** à la référence de la transaction, selon les règles d'égalité normalisée de
l'AC-b1 : `coalesce(reference, end_to_end_id, transaction_id)` de la transaction, retrait de tous les blancs, majuscules,
**au moins 6 caractères alphanumériques** (en deçà, la fonction n'est pas appelée) ; côté facture, la même
normalisation en SQL (`UPPER(REGEXP_REPLACE(…, '[[:space:]]', ''))`, MariaDB 10.11), sur `payment_reference` et
`supplier_invoice_number`. **Lecture ordinaire**, `LIMIT 1` (s'il y en a plusieurs, la plus récente) ; aucun compte ni
montant n'entre au critère : la référence, entière, **désigne** la facture (art. 86 CO) — c'est le débit de **cette**
facture, déjà payée. Ses trois lecteurs sont ceux de la fonction du lot (AC-b5, AC-b6 (e), AC-b17). Coût : une requête
de plus par débit **qui porte une référence** d'au moins 6 caractères ; sans index sur l'expression normalisée, par
`idx_supplier_invoices_company_status` (factures `paid` de la société) — mesuré au développement, écrit au doc-comment.
⚠️ **Ce qu'elle ne couvre pas** : un débit **sans** référence (ou à référence courte) d'une facture déjà payée — la
mensualité voisine ouverte de même montant reste alors proposée (L1, forme sans référence, écrite).

**Les factures masquantes** : `find_supplier_invoices_masking(executor, company_id, amount)` — les factures
fournisseur **ouvertes** de la société dont le TTC est à **0,05 au plus** du débit, **sans borne de dates**, tri
`total_amount = amount` d'abord, puis l'ordre d'imputation, plafond **200** ; l'appelant en retire les candidates (par
identifiant). Une liste **pleine** (200 rendues) est traitée comme **masquante** par prudence (la règle n'est pas
proposée), et le doc-comment le dit. Jamais proposées ; lues pour l'AC-b5, **pour tout débit CHF**, qu'une règle active
existe ou non (la 29-6 ne les lit que s'il y a une règle, `routes/reconciliation.rs:773` ; ici, le coût est le même et
le cas reste simple). Aucune jointure aux lots : une ligne par facture. **Lecture ordinaire.**
**Coût, écrit au doc-comment** (P3 F8 = R10, P4 F-L5) : pour chaque débit CHF de la page (100 à 500), **trois**
requêtes — candidates, masquantes, lot du compte — par `idx_supplier_invoices_company_status` (factures `open` de la
société), prédicat de montant non indexé : négligeable à l'échelle d'une personne ou d'une PME ; mesuré au développement
(AC-b15). **Ce qui est partagé** entre les requêtes (P3, F9) : le `FROM` et l'expression d'exigibilité `X` (constantes
communes).
⛔ **Aucune lecture verrouillante neuve** pour le masquage, ni ici, ni à l'acceptation (C-30-1-39) : la proposition comme
l'acceptation lisent dans la vue de leur transaction, et ne voient pas ce qui est commis après — tolérance déjà admise
par la fiche (AC-b17) et par la 29-6.
*Tests* : T-b7 (les six filtres, chacun par un cas qui le viole seul), T-b8 (lot `generated` → absente des candidates,
rendue par la fonction du lot si le lot est **du compte** et le montant exact ; lot d'un autre compte → absente de la
fonction du lot ; lot `confirmed` ou `cancelled` → candidate si ouverte), T-b9 (TTC `100.0040` contre débit `100.00` →
absente des candidates, présente des masquantes), T-b10 (`LIMIT 50` après l'ordre), T-b42 (201 factures ouvertes sans
indice à 0,05 près : liste pleine → règle masquée), **T-b48** (fonction de la référence : facture payée dont la référence
de paiement, puis le numéro, égalent la référence de la transaction aux blancs et à la casse près → rendue ; facture
ouverte ou annulée, référence de 5 caractères, égalité partielle → absente).

**AC-b4 — La proposition : un débit, des factures fournisseur ; jamais de croisement (#482).**
Dans `get_proposals` :
- un **crédit** CHF garde **exactement** le chemin actuel (factures client) ; il ne reçoit **jamais** de facture
  fournisseur ;
- un **débit** CHF reçoit les candidates de l'AC-b3, notées par `propose_matches`, contacts chargés dans le **même** lot
  que les contacts clients — ceux des candidates **et des factures masquantes** (P5, F5), puisque l'indice « fournisseur »
  se lit sur eux ; le lot de contacts est **découpé par tranches de 500** identifiants (`find_contacts_by_ids` refuse
  au-delà de `MAX_IN_CLAUSE_IDS`, `repositories/reconciliation.rs:692`, `:747` — P5, R1-7) ; il ne reçoit **jamais** de
  facture client ;
- un montant nul ne reçoit aucune facture (ni client ni fournisseur) ;
- `CandidateType` gagne `SupplierInvoice` (sérialisé `supplier_invoice`) ; `ReconciliationCandidate` gagne
  `supplier_invoice_id: Option<i64>` et `contact_name: Option<String>` (rempli pour les **deux** sortes de factures,
  C-30-1-15) ; pour une facture fournisseur, `invoice_number` = `supplier_invoice_number`, `invoice_amount` = TTC au
  centime, `invoice_total_ttc = None`, `invoice_date` = `invoice_date` ; `invoice_id = None`.
*Tests* : T-b11 (débit → fournisseur seulement, même si une facture client a le même montant, le même numéro et le même
contact ; `amount_score = 1.0`), T-b12 (crédit → client seulement, symétrique), T-b13 (forme JSON du candidat).

**AC-b5 — La priorité écrite, étendue ; aucun double comptage muet que la story crée (principe de la 29-6).**
`candidate_rank` (`routes/reconciliation.rs:199-210`) : bras **explicite** `CandidateType::SupplierInvoice`, même règle
que `Invoice` (rang 0 si signal, 2 sinon) ; le `match` reste **sans `_`**. Une facture fournisseur candidate a
`amount_score = 1` (filtre 3 et valeur absolue de l'AC-b1), donc un score ≥ 0,5 : elle **écarte la règle**
(`has_strong_invoice_match`, `:878`, `:932`).

Ce que la ligne propose, selon ce critère écrit, **dans cet ordre** :

| Situation | Effet sur la ligne | Ce que montre la ligne |
|---|---|---|
| une facture ouverte au montant **exact** est engagée dans un lot `generated` **du compte de la transaction** (fonction du lot, AC-b3) | **aucune proposition** : ni facture candidate, ni règle — la sûreté prime sur la commodité (arbitrage de l'orchestrateur, P2 F1) | la **raison** : `notice` `supplier_invoice_in_payment_batch` ; *Accepter* grisé |
| la référence de la transaction désigne une facture fournisseur **déjà payée** (fonction de la référence, AC-b3) | **aucune proposition** : ni facture, ni règle — ce débit est celui de cette facture, et toute proposition le compterait deux fois (P5, F1 = R1-6, arbitrage de l'orchestrateur) | la **raison** : `notice` `reference_of_paid_supplier_invoice` ; *Accepter* grisé |
| une facture masquante (AC-b3) au montant **exact**, non candidate (hors fenêtre, ou dans un lot d'un **autre** compte) | masque la **règle** si elle porte un indice | « Aucune correspondance » s'il n'y a pas de candidate (patron 29-6) |
| une facture masquante à **0,05 au plus**, non exacte | masque la **règle** si elle porte un indice | idem |
| liste masquante **pleine** (200) | masque la **règle** | idem |
| toute autre, sans indice | aucun : la règle est proposée s'il n'y a pas de candidate | la règle |

**Pourquoi la première ligne efface aussi les candidates** (P2, F1, cas nominal des charges mensuelles) : le loyer de
janvier et celui de février, même fournisseur, même montant ; janvier part dans un lot pain.001 ; la banque débite
janvier. Sans cette ligne, février serait candidat et *Accepter* le réglerait avec le débit de janvier ; la
confirmation du lot réglerait ensuite janvier — un seul débit, deux règlements. Kesh ne propose rien et le dit.

**Pourquoi l'indice ailleurs** : un débit du même montant qu'une facture ouverte d'un **autre** fournisseur, sans
référence ni nom commun, garde sa règle — pas de sur-blocage. L'indice se lit sur les **sous-scores**
(`reference_score > 0 || contact_score > 0`), **jamais** par `!propose_matches(…).is_empty()` comme la 29-6
(`routes/reconciliation.rs:887`) : une facture au montant **exact** a `amount_score = 1`, un total ≥ 0,5, et
masquerait la règle **sans** indice (P3, R11 ; T-b36 « proposée sans indice » le tue).

**Ce que la ligne ne fait pas** (C-30-1-34, **retiré** en P4) : une facture fournisseur **déjà payée** — depuis sa fiche
ou par la confirmation d'un lot — dont le paiement n'est rapproché d'aucune transaction ne bloque rien ; la règle peut
être proposée pour son débit, comme en v0.13.4. C'est l'angle mort **L1**, préexistant, territoire de #671 ; le manuel
dit le geste sûr (AC-b13).

`ReconciliationProposal` gagne `notice: Option<ProposalNotice>` (additif), `ProposalNotice { code,
supplier_invoice_id, supplier_invoice_number (nullable), payment_batch_id (nullable) }`, `code` ∈ {
`supplier_invoice_in_payment_batch`, `reference_of_paid_supplier_invoice` } — posé dès que l'une des deux fonctions rend
une facture, **candidates ou non** (elles sont alors retirées) ; si les deux, la référence prime (elle désigne).
*Tests* : T-b14 (`candidate_order_tests` : fournisseur à signal devant la règle), T-b15 (règle écartée par une facture
fournisseur candidate), T-b16 (règle gardée — ligne vide — par une facture presque au montant désignée par sa référence
ou son fournisseur ; règle proposée sans indice), T-b35 (facture exacte en lot `generated` du compte : aucune
proposition, `notice` posée avec facture et lot, même sans indice et **sans règle active**), T-b36 (facture exacte hors
fenêtre : règle masquée avec indice, proposée sans indice), **T-b41** (deux mensualités du même fournisseur et du même
montant, la première en lot `generated` du compte, la seconde ouverte dans la fenêtre : **aucune** proposition, `notice`
sur la première ; même décor avec le lot sur un **autre** compte : la seconde est proposée), **T-b49** (janvier payé
depuis la fiche, février ouvert au même montant : un débit portant la référence QR de janvier → **aucune** proposition,
`notice` `reference_of_paid_supplier_invoice` ; le même débit **sans** référence → février **proposé** — l'angle mort L1,
forme sans référence, asserté tel quel ; une référence de 5 caractères égale au numéro → pas de garde).

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
3. **verrous et éligibilité, en `kesh-db`** : `supplier_invoices::lock_for_bank_settlement_in_tx(tx, company_id,
   supplier_invoice_id, bank_account_id) -> Result<Result<LockedForSettlement, SupplierSettlementRefusal>, DbError>`,
   dans **cet ordre de verrous** (celui de `pay_in_tx` et de `create_batch` : facture, puis compte, puis articles —
   AC-b10) :
   (a) facture `FOR UPDATE` → absente ou d'une autre société : `NotFound` ; statut ≠ `open` : `NotOpen { status }` ;
   (b) compte bancaire de l'import `SELECT journal_account_id, closed_on, archived … FOR UPDATE` → absent ou archivé :
   `BankAccountNotFound` ; clos : `BankAccountClosed` ; sans compte lié : `BankAccountNotConfigured` ;
   (c) appartenance à un lot `generated`, lecture **verrouillante** (`LOCK IN SHARE MODE`) → `InGeneratedBatch {
   payment_batch_id }` (AC-b8) ;
   (d) **la même garde que la proposition** (P3, F6 = R5 ; simplifiée en P4, C-30-1-39) : la fonction du lot du compte
   (AC-b3), en **lecture ordinaire** — une **autre** facture ouverte au montant
   exact engagée dans un lot `generated` de ce compte → `HeldByPaymentBatch { supplier_invoice_id, payment_batch_id }`.
   Sans elle, un appel d'API direct, ou un écran chargé avant la création du lot (deux onglets), réglerait février avec
   le débit de janvier — le faux rattachement que la proposition efface. ⚠️ **Tolérance écrite** : en lecture
   ordinaire, (d) voit la vue née à l'étape 1 ; un lot commis **après** n'est pas vu — la même tolérance que la
   proposition et que l'AC-b17. (d) n'ajoute **aucun** verrou : l'ordre des verrous de l'acceptation reste celui de
   (a), (b), (c) et de `pay_in_tx_as` ;
   (e) **la référence d'une facture déjà payée** (P5, arbitrage) : la fonction de l'AC-b3, lecture ordinaire →
   `HeldByPaidReference { supplier_invoice_id }` ;
4. fenêtre (AC-b2) sur la facture **verrouillée** ;
5. **montant** : `|tx.amount| ≠ total_amount` → `RECONCILIATION_AMOUNT_MISMATCH` (`details.amountDue`,
   `details.transactionAmount`) ; le score est recalculé (`propose_matches`, valeur absolue) pour la trace et la
   réponse — il vaut ≥ 0,5 **puisque** l'égalité vient d'être vérifiée et que la note compare la valeur absolue (AC-b1) ;
   **aucune garde de score** (C-30-1-16, maintenu en P1 sur l'énoncé corrigé ; T-b17 asserte `score.total ≥ 0.5` sur la
   réponse, M-b16 rétablit la comparaison signée et doit rougir) ;
6. `supplier_invoices::pay_in_tx_as(tx, company_id, id, SettlementChoice::BankTransfer { bank_account_id },
   payment_date(tx), actor)` — `bank_account_id` = **le compte de l'import**, jamais un choix du client ; ses lectures
   verrouillantes de la facture et du compte redemandent des verrous **déjà tenus** ;
7. `UPDATE bank_transactions SET matched_entry_id = <écriture de règlement>, status = 'reconciled' … WHERE … AND status
   = 'pending' AND version = ?` → zéro ligne : `RECONCILIATION_ALREADY_RECONCILED` (`details.reason =
   "race_during_update"`) ;
8. audit `reconciliation.accepted` (`for_actor`) : `bank_transaction_id`, `supplier_invoice_id`, `score`,
   `batch_size`, `journal_entry_id` (le règlement), `purchase_journal_entry_id`, `paid_amount`.

Effet : facture `paid`, `settlement_type = 'bank_transfer'`, `settlement_bank_account_id` = compte de l'import,
`paid_at` = date du paiement (AC-b16) ; écriture `D créanciers / C banque` du TTC (= `|tx.amount|`), **datée de la date
de paiement**, journal Banque, dans l'exercice ouvert qui la couvre, projet de la facture ; lettrage `document` ;
transaction `reconciled` pointant cette écriture ; deux audits (`reconciliation.accepted`, `supplier_invoice.paid`).

**Inventaire clos des refus** (refait en P1 sur `pay_in_tx`, `create_in_tx_inner`, `validate_lines_accounts_in_tx`,
`letterings::sync_supplier_invoice_in_tx`) — aucun n'escalade en `AppError` (§ *Pattern batch*). L'étape 3 lève
**toute** ambiguïté de `NotFound` et d'`IllegalStateTransition` : facture et compte y sont lus et verrouillés, et
l'exercice est trouvé par `find_open_covering_date … FOR UPDATE` avant `create_in_tx` ; et le règlement ne valide
**aucun** projet (lignes à `project_id: None`, `supplier_invoices.rs:875`, `:881` ; projet de document non revalidé,
`journal_entries.rs:300-303`, testé par `pay_succeeds_when_project_archived_after_tagging`). À l'étape 6, ces deux
variantes sont donc **inatteignables** : des invariants rompus, classés `INTERNAL_ERROR` — et non par
`project_error_to_failed_proposal`, qui les rendrait `PROJECT_NOT_FOUND` / `PROJECT_ARCHIVED` à tort (corrigé en P2,
F3 = R1). Une fonction `supplier_payment_error_to_failed_proposal`,
`match` sur les variantes **nommées** ci-dessous plus un bras final `other → DATABASE_ERROR` :

| Source | Erreur | `errorCode` | `details` |
|---|---|---|---|
| étape 3 | `NotFound` | `SUPPLIER_INVOICE_NOT_FOUND` (neuf pour la réconciliation ; existe pour les lots) — indiscernable d'une facture d'une autre société | `{ supplierInvoiceId }` |
| étape 3 | `NotOpen` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `{ reason: "supplier_invoice_not_open" }` |
| étape 3 | `BankAccountNotFound` | `BANK_ACCOUNT_NOT_FOUND` | — |
| étape 3 | `BankAccountClosed` | `BANK_ACCOUNT_CLOSED` | `{ bankAccountId }` |
| étape 3 | `BankAccountNotConfigured` | `BANK_ACCOUNT_NOT_CONFIGURED` | `{ bankAccountId }` |
| étape 3 | `InGeneratedBatch` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `{ reason: "supplier_invoice_in_payment_batch", paymentBatchId }` |
| étape 3 (d) | `HeldByPaymentBatch` | `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` (neuf, libellé ×4) | `{ reason: "other_invoice_in_payment_batch", supplierInvoiceId, paymentBatchId }` |
| étape 3 (e) | `HeldByPaidReference` | `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` | `{ reason: "reference_of_paid_invoice", supplierInvoiceId }` (libellé ×4) |
| étape 6 | `SettlementCounterpartyIsClaimAccount` | (son code) | `claim_account_failed_proposal(…, Some(bank_account_id))` — rôle `counterparty`, `claim = "payable"` |
| étape 6 | `InactiveOrInvalidAccounts` (compte créanciers de l'écriture d'achat, ou compte du grand livre de la banque, désactivé depuis l'achat : `journal_entries.rs:166`) | `INACTIVE_OR_INVALID_ACCOUNTS` (`err.error_code()`, code canonique existant) | — |
| étape 6 | `FiscalYearInvalid` | `FISCAL_YEAR_INVALID` | `{ valueDate }` |
| étape 6 | `FiscalYearClosed`, `DateOutsideFiscalYear` (`create_in_tx_inner`) — inatteignables après `find_open_covering_date` sous verrou, **classés quand même** | `FISCAL_YEAR_INVALID` | `{ valueDate }` |
| étape 6 | `PeriodLocked` | `PERIOD_LOCKED` | `period_locked_failed_proposal` (projet de la facture) |
| étape 6 | `LaterFiscalYearClosed` | `LATER_FISCAL_YEAR_CLOSED` | `later_fiscal_year_closed_failed_proposal` |
| étape 6 | `NotFound`, `IllegalStateTransition` — inatteignables (cf. ci-dessus) | `INTERNAL_ERROR` (précédé de `tracing::error!`) | — |
| étape 6 | `OptimisticLockConflict` | `RECONCILIATION_INVOICE_NOT_ELIGIBLE` | `{ reason: "race_during_update" }` |
| étape 6 | `Invariant`, erreurs structurelles du lettrage | `INTERNAL_ERROR` (précédé de `tracing::error!`) | — |
| toutes | autre (`Sqlx`…) | `DATABASE_ERROR` | `{ message }` — un interblocage est rejoué par `accept_batch` |

⚠️ L'inventaire se **refait au développement** depuis le code et se recopie au Dev Agent Record : une variante
atteignable non classée est un finding.

**Conventions de `details`** (P2, R-L4) : `transactionAmount` = le montant de la transaction **tel quel** (négatif pour
un débit), comme les refus client ; `amountDue` = le TTC (positif) ; l'audit `reconciliation.accepted` porte
`paid_amount` = le TTC réglé (positif) et `transaction_amount` = le montant tel quel. Casse : celle de chaque famille
existante — `snake_case` pour les clés de raison de dates (`window_days`, `tolerance_days`) et pour l'audit,
`camelCase` pour les identifiants (`supplierInvoiceId`, `paymentBatchId`, `bankAccountId`), comme `details.invoiceId`
d'aujourd'hui.
*Tests* : T-b17 (acceptation nominale : état complet en base — facture, écriture et ses deux lignes, date, exercice,
lettrage, transaction, deux audits, `score.total ≥ 0.5` ; TTC de la facture = crédit de l'écriture d'achat), T-b18 (lot
mixte client + fournisseur + règle : succès partiel, ordre de `failed[]`), T-b19 (un test par ligne du tableau
atteignable par décor, dont `INACTIVE_OR_INVALID_ACCOUNTS`), T-b20 (`200` partiel, jamais `4xx/5xx` pour un refus par
proposition), **T-b45** (appel d'API direct : `supplier_invoice` février accepté pour le débit de janvier, janvier en lot
`generated` du compte → `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` / `other_invoice_in_payment_batch`, rien écrit ; lot sur
un **autre** compte → acceptée), **T-b50** (API directe : février accepté pour un débit qui porte la référence de janvier,
payé → `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` / `reference_of_paid_invoice`, rien écrit).

**AC-b7 — Le sens de la transaction, une seule fonction (DRY).**
`refuse_non_credit_settlement` (`:1705-1729`) devient `refuse_wrong_direction(bank_transaction_id, amount,
expected: SettlementDirection)` — `Credit` pour une facture client, `Debit` pour une facture fournisseur. Côté client,
**code, raisons et `details` strictement inchangés** (`RECONCILIATION_TRANSACTION_NOT_CREDIT`, `debit_transaction` /
`zero_amount_transaction`) ; côté fournisseur, `RECONCILIATION_TRANSACTION_NOT_DEBIT`, `credit_transaction` /
`zero_amount_transaction`. Comparaison **au zéro**, jamais par `is_sign_negative()` (`-0.00`, C-28-6-12).
*Tests* : `non_credit_settlement_tests` (`:5729-5804`) : `refuse_non_credit_settlement` est **gardée** comme enveloppe
d'une ligne de `refuse_wrong_direction(…, Credit)`, si bien que leurs appels directs (`:5738`, `:5784`, `:5789`,
`:5794`) et leurs assertions restent inchangés (P2, R-L3) ; T-b21 (débit attendu : crédit, zéro,
`-0.00`).

**AC-b8 — La garde du lot de paiement est verrouillante (naissance de la vue).**
Dans la transaction d'un lot d'acceptation, la vue `REPEATABLE READ` est **déjà née** (lecture de la transaction à
l'étape 1, ou proposition précédente du lot) quand la facture est verrouillée. Un `create_batch` validé **entre** la
naissance de la vue et le verrou de la facture serait **invisible** à une lecture ordinaire de `payment_batch_items` :
`create_batch` verrouille la facture mais ne la modifie pas (elle reste `open`), et la facture serait réglée puis
**payée une seconde fois par la banque** sur le fichier pain.001 déjà généré (scénario confirmé réel en P1, F). D'où
l'étape 3 (c) : appartenance lue en **lecture verrouillante**, **après** le verrou de la facture **et** celui du compte.
`in_generated_batch` (`supplier_invoices.rs:634-650`), lu aussi par les vues (`supplier_invoice_cancel_blocker`), **reste
non verrouillant** : la lecture verrouillante est une fonction distincte, documentée.
⚠️ Garde valable en MariaDB 10.11 (`innodb_snapshot_isolation = OFF`) ; l'angle mort 11.6+ est tracé par **#669** (P3,
R9).
⚠️ **Verrou d'intervalle, écrit** : la lecture verrouillante qui ne trouve aucun article pose un intervalle partagé sur
`idx_payment_batch_items_invoice` (`20260628000002_payment_batches.sql:59`) au point `supplier_invoice_id = <facture>`,
tenu **jusqu'au commit du lot d'acceptation**. Un `create_batch` d'**autres** factures dont un article tombe dans cet
intervalle (le plus souvent l'intervalle de queue : facture récente) **attend** la fin du lot d'acceptation ; sur le
**même** compte, il attendait déjà au verrou du compte (AC-b10, C3). Le seul cycle qui passe par cet intervalle est **C1 / C7**, rejoué des deux
côtés : la proposition 1 tient l'intervalle, un `create_batch` (de ce compte ou d'un autre) qui a déjà pris la facture
`b` y attend l'insertion de son article, et la proposition 2 demande `b` (P5, R1-4).
*Test* : T-b22 (course : vue de la transaction d'acceptation née, `create_batch` validé sur une seconde connexion,
puis acceptation → refus `supplier_invoice_in_payment_batch`, rien écrit ; **rouge** avec une lecture ordinaire —
mutation M-b8).

**AC-b9 — Le règlement porte l'acteur (DRY), sans toucher `payment_batches.rs`.**
Le corps de `pay_in_tx` passe dans `pay_in_tx_as(tx, company_id, id, choice, payment_date, actor: letterings::Actor)`
(`letterings.rs:202`) : l'audit `supplier_invoice.paid` passe par `NewAuditLogEntry::for_actor`, le lettrage reçoit la
clé. `pay_in_tx` garde **sa signature** et devient une enveloppe d'une ligne (`Actor { user_id, api_key_id: None }`) :
`pay` et `confirm_batch` (`payment_batches.rs:496`) ne changent pas. Le nouveau chemin appelle `pay_in_tx_as` avec la
clé de la requête. Un seul corps : aucun jumeau.
⚠️ **Deux registres nomment l'écrivain `pay_in_tx`** et rougiront au déplacement du corps (P1 : F6, R4) :
`crates/kesh-api/tests/bank_account_closed_inventory.rs:274-277` (`REGISTRE`, entrée `pay_in_tx` → `pay_in_tx_as`, et les
textes `:243`, `:867`, qui restent vrais par l'enveloppe) et `crates/kesh-db/tests/letterings_lexical.rs:596-668`
(`supplier_settlement_writers_sync_or_dissolve` : nom codé en dur `:604` et liste fermée `:631` → `pay_in_tx_as`).
*Test* : T-b23 (acceptation par une clé `read-write` : la clé figure dans `supplier_invoice.paid`,
`reconciliation.accepted` et `lettering.created`).

**AC-b10 — Concurrence : l'ordre des verrous de chaque chemin, écrit, et les cycles qu'il ferme ou laisse (C-30-1-29).**

| Chemin | Ordre des verrous (→ = puis) |
|---|---|
| **Acceptation fournisseur, une proposition** (30-1b) | `GET_LOCK(compte)` → lecture ordinaire de `t` (**naissance de la vue**) → `supplier_invoices` X → `bank_accounts` X → `payment_batch_items`/`payment_batches` S (intervalle si vide) → [`pay_in_tx_as` : facture et compte tenus, lignes d'achat lues, comptes lus, exercice verrouillé par `find_open_covering_date`/`create_in_tx`, écriture, `UPDATE supplier_invoices`, lettrage] → `bank_transactions` X (`UPDATE`) ; la garde (d) et la lecture du lot du compte : **lectures ordinaires**, aucun verrou (C-30-1-39) |
| **Acceptation d'une règle** (`accept_one_rule`, AC-b17) | inchangé au sol : `GET_LOCK(compte)` → lectures ordinaires (compte, règle, transaction) → exercice et écriture (`create_in_tx`) → `bank_transactions` X ; le rejeu du masquage (AC-b17) n'ajoute que des **lectures ordinaires** : aucun verrou, aucun cycle neuf (P4, F4) |
| **Lot d'acceptation de N propositions** | chaque proposition dans l'ordre reçu ; les verrous d'une proposition — `bank_accounts` X dès la **première** fournisseur, S dès la première **client** (clé étrangère de `invoice_settlements.settlement_bank_account_id`) — sont **tenus jusqu'au commit du lot**, y compris pour une proposition refusée (`ROLLBACK TO SAVEPOINT` ne relâche pas les verrous de lignes) ; les factures des propositions suivantes sont donc verrouillées **après** le compte |
| `pay` (fiche fournisseur) | facture X → `payment_batch_items` (lecture ordinaire, première lecture : vue née après le verrou) → compte X → exercice → écriture → `UPDATE` → lettrage ; rejoué (`routes/supplier_invoices.rs:388`) |
| `create_batch` (après la 30-1a) | **toutes** les factures demandées X, par identifiant croissant, parcours de la clé primaire (AC-a9 ; aucune si la liste est vide) → compte (lecture ordinaire : **la vue naît ici**, après le dernier verrou de facture) → par facture `validate_invoice_for_batch` (verrou redemandé, déjà tenu ; appartenance lue dans la vue née après) → `INSERT payment_batches` (compte S par clé étrangère) → revérification du compte S → `INSERT payment_batch_items` (intention d'insertion) ; **rejoué** (AC-a8). *Au sol, avant la 30-1a : compte lu d'abord, facture par facture dans l'ordre de la requête — l'ordre que l'AC-a9 supprime (P3, F1 = R4).* |
| `confirm_batch` | `payment_batches` X (`:444`) → par facture : `pay_in_tx` (facture X → compte X → exercice …) → `UPDATE payment_batches` ; rejoué (`:251`) |
| Dé-rapprochement (30-1a) | `GET_LOCK(compte)` → `bank_transactions` X → `invoice_settlements` X ou `supplier_invoices` X (intervalle si rien) → (client : `invoices` X) → écriture + exercice X → compte (lecture) → … → compte S (dernière acquisition) ; rejoué |
| Clôture du compte (`bank_accounts::close_in_tx`) | compte X → décomptes de transactions `pending` et de lots `generated` (`bank_accounts.rs:1289`, `:1304`) ; rejouée (`routes/bank_accounts.rs:1163`) |
| Règlement client depuis la fiche (`settle_invoice`) et son annulation | facture X → compte `FOR UPDATE` (`invoice_settlements_write.rs:138`) → exercice → … ; rejoués (`routes/invoices.rs:1282`, `:1497`) |
| `supplier_invoices::cancel_in_tx`, `cancel_settlement_in_tx` (fiche fournisseur) | facture X → écriture + exercice X → compte (lecture, puis revérification S) ; rejoués (`routes/supplier_invoices.rs:419`, `:451`) |
| `POST /bank-imports` | `INSERT` d'entête (clé étrangère S sur le compte), sans verrou antérieur ; **non rejoué** — attend le compte X pendant un lot d'acceptation qui contient une proposition fournisseur ; aucun cycle établi (P2, R-L6) |
| Clôture d'exercice (`fiscal_years::close`) | lectures des exercices, puis verrous des **exercices** seulement — ni facture, ni compte, ni transaction (à revérifier au développement, `fiscal_years.rs:1115`) |

Cycles, chacun avec sa victime et son rejeu :
- **C1 — lot d'acceptation ↔ `create_batch`** : la proposition 1 tient le compte X ; `create_batch` tient la facture `b`
  X et attend le compte (S, clé étrangère) ; la proposition 2 demande la facture `b`. **Cycle inhérent** à tout lot qui
  prend le compte avant une facture suivante (le compte est pris par `pay_in_tx`, inévitablement). Rejoué des deux côtés
  : `retry_with` (acceptation), `retry_on_deadlock` (création, AC-a8).
- **C2 — acceptation ↔ `confirm_batch`** : quand la facture `a` est dans le lot que l'on confirme, l'acceptation tient
  `a` X et lit la ligne `payment_batches` (S, jointure) que `confirm_batch` tient X ; `confirm_batch` demande `a` X.
  Rejoué des deux côtés ; l'acceptation, rejouée, voit le lot `confirmed` et la facture `paid` : refus
  `supplier_invoice_not_open`.
- **C3 — une seule proposition ↔ `create_batch`** : **aucun cycle** — même ordre (facture → compte → articles) ;
  `create_batch` attend au verrou du compte la fin du lot d'acceptation (jusqu'à sa durée ; au-delà de
  `innodb_lock_wait_timeout`, 50 s par défaut, un `1205` non rejoué — prix écrit). C'est l'arbitrage de la P1 (F1) :
  le compte est verrouillé **avant** la lecture des articles.
- **C4 — lot mixte, promotion** : une proposition client tient le compte S (clé étrangère), une proposition fournisseur
  suivante le demande X ; une clôture du compte en file (X) ferme le cycle de promotion. Rejoué des deux côtés
  (acceptation ; clôture, `routes/bank_accounts.rs:1163`, vérifié en P2).
- **C7 — entre comptes** : deux lots d'acceptation sur deux comptes, ou un dé-rapprochement sur A contre un lot sur B (le
  verrou nommé est par compte), peuvent se croiser sur une facture ou un exercice ; tous les participants sont rejoués
  (P2, F-L2). La table ci-dessus ne prétend pas énumérer les cycles **préexistants** sans participant neuf.
- **C5 — `pay` (fiche) ↔ lot d'acceptation** : même forme que C1 (la fiche prend la facture puis le compte) ; rejoué
  des deux côtés.
- **C6 — dé-rapprochement `Entry` ↔ `pay_in_tx`** : AC-a4 (intervalle de `fk_supplier_invoices_settlement_je` contre
  verrou de l'exercice) ; rejoué des deux côtés.
- La 29-4b (« revérification verrouillante en dernière acquisition ») ne s'applique pas à l'acceptation : celle-ci ne
  revérifie pas un compte lu sans verrou, elle le **verrouille** en exclusif à l'étape 3 (b) et le garde ; la clôture
  attend donc, et aucune acceptation ne passe sur un compte clos (la clôture la voit ensuite `reconciled`).

*Tests* : T-b24 (fiche contre réconciliation, même facture : l'un aboutit, l'autre est refusé), T-b25 (clôture du compte
pendant l'acceptation : ⚠️ la clôture refuse tant qu'une transaction est `pending` (I1), donc elle échoue en `412` ou
attend — le test établit qu'aucun règlement ne passe sur un compte clos, **pas** que la garde de `pay_in_tx` s'exécute :
c'est I1 qui protège ; la branche `BANK_ACCOUNT_CLOSED` n'est atteinte que par le pré-vol R1), T-b34 (course
**au niveau du dépôt, sans enveloppe de rejeu** — le rejeu des deux côtés rendrait la mutation verte —, décor où
l'identifiant `b` de la facture du lot est **supérieur** à `a` (celle de l'acceptation), et `a` **comme** `b` sont
supérieurs à tout `supplier_invoice_id` déjà présent dans `payment_batch_items` (P3, R8 : sinon l'intervalle de `a` est
borné par un article existant et la mutation survit), si bien que l'article de `b` tombe dans l'intervalle de queue
verrouillé pour `a` ; chorégraphie forcée par sondes (P2, F4 = R5) : (1) une troisième connexion tient le compte `FOR UPDATE` ; (2)
`create_batch(b)` démarre, tient `b` et se bloque sur la clé étrangère du compte (`attendre_une_requete_en_cours`) ; (3)
l'acceptation de `a` démarre et se bloque sur le compte (sonde) ; (4) la troisième connexion relâche. Code prescrit :
l'acceptation n'a pas lu les articles au point (3), `create_batch` (premier en file) obtient le compte en S, insère son
article, commite ; l'acceptation passe ensuite, voit le lot de `b` sans rapport avec `a` — le TTC de `b` **diffère**
du débit, sinon la garde (d) refuserait l'acceptation (P4, F1) —, et aboutit ; **aucun 1213**.
Mutant M-b23 : l'acceptation tient déjà l'intervalle au point (3) ; `create_batch` bute sur l'intervalle → **1213**,
rouge).

**AC-b11 — L'écran.**
`ReconciliationProposals.svelte` :
- une candidate `supplier_invoice` s'affiche avec un badge *Facture fournisseur*
  (`reconciliation-labels-supplier-invoice-badge`), le **nom du fournisseur** (`contactName`), le numéro
  (`invoiceNumber`, à défaut `#id`) et le montant ; `data-testid="candidate-top-1"`, `data-candidate-type=
  "supplier_invoice"`, `data-supplier-invoice-id` ;
- une candidate `invoice` s'affiche **comme aujourd'hui** (C-30-1-15) ;
- une ligne qui porte un `notice` l'affiche **à la place** de toute proposition (`data-testid="proposal-notice"`,
  `data-notice-code`) ; *Accepter* reste grisé pour elle, comme pour toute ligne sans candidate. Son texte ne conseille
  **jamais** de confirmer le lot ni d'accepter une règle (P3, F5 = R6 : confirmer mène à L1, une charge comptée deux
  fois), et ne cite aucun numéro d'issue (P4, R9) :
  `reconciliation-labels-notice-supplier-invoice-in-batch` : « La facture fournisseur { $invoice } de ce montant figure
  dans le lot de paiement n° { $batch } en cours sur ce compte. Si cette transaction est le paiement de ce lot, annulez
  le lot : la facture vous sera alors proposée ici. Sinon, affectez la transaction à la main. » ; `{ $invoice }` = le
  numéro, à défaut `#id` — numéro nullable, P2 F-L7 ; et
  `reconciliation-labels-notice-reference-of-paid-supplier-invoice` (P5) : « La référence de cette transaction est celle
  de la facture fournisseur { $invoice }, déjà payée. Rien n'est proposé : toute affectation compterait ce paiement deux
  fois. Si cette facture doit être réglée par la réconciliation, annulez d'abord son paiement depuis sa fiche. » — sans
  numéro d'issue. *(La `notice` « paiement en attente de rapprochement » de la P3 est retirée en P4 avec son critère,
  C-30-1-34.)*
- *Accepter* construit `{ type: 'supplier_invoice', bankTransactionId, supplierInvoiceId }` ;
- l'avis de priorité (`reconciliation-labels-priority-hint`, `messages.ftl:1283`) est réécrit : « Une seule proposition
  est affichée par transaction : pour un crédit, la facture client la mieux notée, pour un débit, la facture fournisseur
  la mieux notée, s'il y en a une, sinon la règle d'affectation qui correspond. Une facture proposée passe toujours
  devant une règle. » (quatre locales).
`CancelReconciliationDialog.svelte` (déplacé de la 30-1a, C-30-1-33) : pour `kind === 'supplier_settlement'`, le texte
`reconciliation-cancel-confirm-supplier-invoice` — « Une écriture inverse datée d'aujourd'hui sera passée, le paiement
de la facture fournisseur { $invoice } sera retiré et la facture redeviendra ouverte, à payer ; la transaction
redeviendra à rapprocher. » (quatre locales ; `{ $invoice }` = `supplierInvoiceNumber`, à défaut `#supplierInvoiceId`).
`failed-proposal-label.ts` :
- libellés neufs : `RECONCILIATION_TRANSACTION_NOT_DEBIT`, `SUPPLIER_INVOICE_NOT_FOUND` (neuf **pour la réconciliation** :
  le code existe déjà comme refus d'un lot, `payment_batches.rs:330`, libellé « Facture introuvable »,
  `payment-batch-helpers.ts:88` — le libellé de la réconciliation peut le reprendre, P2 R-L16),
  `RECONCILIATION_AMOUNT_MISMATCH`, `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` (AC-b6 (d) et (e), raisons
  `other_invoice_in_payment_batch` et `reference_of_paid_invoice`), `RECONCILIATION_RULE_MASKED` (AC-b17, quatre
  raisons : `payment_batch`, `invoice_candidate`, `paid_invoice_reference`, `invoice_slightly_overpaid` — le libellé lit
  la raison et le dit ; le remède : *Affecter manuellement*, ou traiter la facture), et la raison
  `supplier_invoice_in_payment_batch` (le remède : annuler le lot, ou affecter à la main — jamais « confirmer ») ;
- `INACTIVE_OR_INVALID_ACCOUNTS` : la clé globale `error-inactive-accounts` existe (`messages.ftl:307`), mais
  `failed-proposal-label.ts` n'a **aucun** bras pour ce code (P3, R2 : `grep -nF INACTIVE_OR_INVALID_ACCOUNTS
  failed-proposal-label.ts` vide ; il tomberait dans « Refus non reconnu ») — un `case` est **ajouté**, qui lit la clé
  globale (admise par `lint-i18n-ownership`) ;
- `payment_date_before_invoice_date` : le libellé lit `details.tolerance_days` (repli 1) — « Le paiement est daté de
  plus de { $days } jour(s) avant la facture… » ; le texte client reste vrai (P1 R6) ;
- `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` : le libellé lit aussi `details.claim` — `payable` → « … lié au compte
  **créanciers** de la facture », `receivable` → le texte actuel (P1 R6) ;
- le doc-comment (« Les 31 codes », « Cinq codes seulement lisent leur `details` ») **et** le test
  `failed-proposal-label.test.ts:11`, `:84-85` (`toHaveLength(31)`, `EXPECTED_KEYS`) recomptés depuis la source par la
  commande qu'ils citent : **37** attendus — 31 + cinq littéraux neufs (`RECONCILIATION_TRANSACTION_NOT_DEBIT`,
  `SUPPLIER_INVOICE_NOT_FOUND`, `RECONCILIATION_AMOUNT_MISMATCH`, `RECONCILIATION_SUPPLIER_PAYMENT_HOLD`,
  `RECONCILIATION_RULE_MASKED`) + un code par `error_code()` (`INACTIVE_OR_INVALID_ACCOUNTS`). *(La P3 comptait 35, avant le code de l'AC-b6 (d) ; à recompter au
  développement.)*
`reconciliation.types.ts` : types étendus (`notice`, `supplierInvoiceId`, `contactName`, le type d'acceptation, `kind`
élargi à `supplier_settlement`, `supplierInvoiceId` et `supplierInvoiceNumber` de la vue).
*Tests* : Vitest T-b26 (rendu du candidat fournisseur, du candidat client inchangé), T-b27 (corps d'*Accepter* pour
les quatre types), T-b28 (libellés neufs), T-b38 (libellés lus sur `details` : `claim`, `tolerance_days`, repli),
T-b39 (chacune des deux `notice` affichée à la place d'une candidate, *Accepter* grisé, aucun texte ne contient
« confirmez » ni un numéro d'issue), **T-b40** (dialogue d'annulation, trois `kind`,
repli `#id` ; ex-T-a9, déplacé avec le dialogue).

**AC-b12 — E2E de parcours (la recette) et captures.**
`frontend/tests/e2e/reconciliation-supplier.spec.ts`, montants et références **uniques** par exécution (patron
`reconciliation-link.spec.ts:143-170`), fixture CAMT **synthétique** dérivée de `camt053_v04_minimal.xml` réécrite en
`DBIT`, création des factures par l'API (patron `payment-batches.spec.ts:100-114`, factorisé en
`createSupplierInvoiceViaApi` dans `helpers/api-fixtures.ts`) :
1. fournisseur et facture fournisseur **ouverte** créés par l'API, avec une référence de paiement QR, datée et échue
   **plusieurs mois avant** le débit, **saisie** le jour du test (l'usage de Guy) ;
2. relevé importé, dont un débit du montant de la facture, portant la référence ;
3. *Réconciliation* → la ligne du débit propose la facture fournisseur (badge, fournisseur, numéro, montant) → *Accepter*
   → « 1 opération(s) réussie(s). », la ligne quitte la liste ;
4. fiche de la facture : **payée**, à la **date du paiement** (`paidAt`), lien vers l'écriture de règlement ; fiche
   d'écriture : lettrée ; détail de l'import : transaction rapprochée ;
5. détail de l'import → *Annuler le rapprochement* → le texte de confirmation fournisseur → confirmer → transaction à
   rapprocher, facture **ouverte** à nouveau ; la ligne revient dans la réconciliation.
Un deuxième test : un **crédit** du même montant ne propose **pas** la facture fournisseur. Un troisième, **sans QR**
(fichier d'epic, § *Indispensable* : « avec QR, sans QR ») : facture saisie sans référence de paiement, débit sans
référence, désigné par le montant et le nom du fournisseur → proposée, acceptée, payée (P2, R-L10). La variante « sans
document » relève de la 30-2 (création de la facture) : pour la réconciliation, une facture saisie sans document est
une facture fournisseur ouverte comme une autre.
Captures **clair et sombre** (`page.emulateMedia`) de la ligne proposée et du dialogue d'annulation, dans
`KESH_E2E_CAPTURES_DIR`, hors dépôt ; une capture par schéma va au manuel (`docs/manual/fr/screenshots/`), si la
section en porte une.
Les E2E de la 30-1a (dialogue sur un état posé par l'API de la fiche + lien en SQL) ne sont **pas** requis : le
parcours de la 30-1b les couvre de bout en bout, et le dialogue est couvert par Vitest (C-30-1-17).

**AC-b13 — Les textes.**
- **Manuel utilisateur FR** (`docs/manual/fr/user-manual.tex`) et **son PDF** régénéré :
  - § *Réconciliation bancaire* (`:2111-2113` « soit en réglant une facture client » → client **ou fournisseur**) ;
  - § *Algorithme de matching* (`:2121` « créditrice, en CHF » ; un paragraphe pour le débit : facture fournisseur
    ouverte, montant **exactement** égal au total, référence de paiement QR ou numéro complet, fenêtre 7 jours / 365
    jours, lot de paiement en cours exclu et signalé sur la ligne, les factures qui gardent la règle — critère de
    l'AC-b5) ;
  - § *Accepter ou rejeter* (`:2140-2150` la candidate affichée ; `:2156`, l'*Accepter (N)* qui décrit « Débit banque /
    Crédit débiteurs » — à compléter par `D créanciers / C banque` pour une facture fournisseur ; la ligne qui dit
    pourquoi rien n'est proposé : lot en cours, ou référence d'une facture déjà payée ; et qu'une **règle** peut désormais être refusée à l'acceptation —
    `RECONCILIATION_RULE_MASKED` —, l'imputation délibérée passant par *Affecter manuellement*, P4 F7) ;
  - **FAQ** *Aucune correspondance* (`sec:faq-aucun-match`) : `:3034` « La transaction est un débit, ou n'est pas en
    CHF : seules les transactions créditrices en CHF reçoivent une facture candidate », fausse dès la 30-1b — réécrite
    (un débit reçoit des factures fournisseur ; il n'en reçoit pas d'une facture client) ; « Que faire ensuite »
    (`:3040-3045`) gagne la branche « paiement de fournisseur » (saisir la facture ouverte, puis l'accepter ici ; ne pas
    affecter par une règle le débit d'une facture déjà payée — L1) (P3, R3) ;
  - **L1, le geste sûr**, au § *Régler une facture fournisseur* et à la FAQ (P5, F1 b, F10 ; acceptée par
    l'orchestrateur, portée au Project Lead) : « Si cette dépense est déjà réglée depuis la fiche de la facture, ou par
    la confirmation d'un lot de paiement, n'acceptez **rien** de ce que la réconciliation propose pour son débit — ni la
    règle d'affectation, ni une **autre facture du même montant** (la mensualité suivante, typiquement) : la charge ou le
    paiement seraient comptés deux fois. Quand le débit porte la référence de la facture payée, Kesh le reconnaît et ne
    propose rien ; sans référence, il ne le peut pas. Kesh ne sait pas encore rapprocher ce débit du paiement enregistré
    (#671) : la transaction reste **en attente**, et empêche la clôture du compte bancaire et figure parmi les
    transactions à rapprocher de la préparation de clôture, tant que ce rapprochement n'existe pas. Pour la régler par la
    réconciliation, annulez le paiement depuis la fiche de la facture : elle vous sera alors proposée ici — sauf si
    l'exercice du paiement est **clos** : l'annulation est alors refusée, et il n'y a pas d'autre geste. » (le numéro
    d'issue au manuel seulement, jamais à l'écran) ;
  - **`:3089`** (« Comment annuler quoi ») : « Rapprochement bancaire : il s'annule depuis le détail de l'import
    bancaire, ou depuis la fiche de la facture qu'il a réglée » — faux pour une facture **fournisseur**, dont la fiche n'a
    pas le dialogue : « … ou depuis la fiche de la facture **client** qu'il a réglée » (P5, R1-2) ;
  - **`:2244`** (§ *Règles d'affectation*) : « une règle ne s'applique que quand aucune facture n'est candidate… » —
    complété pour un débit (facture fournisseur à 0,05 près, lot du compte, référence d'une facture payée : rien n'est
    proposé) et pour l'acceptation (`RECONCILIATION_RULE_MASKED`) (P5, R1-2) ;
  - § *Annuler un rapprochement* (`:2252`, le paiement fournisseur retiré, la facture redevenue ouverte) ;
  - § *Régler une facture fournisseur* (`:1586`) : « ou laisser la réconciliation la régler » ; **`paidAt`** : la fiche
    affiche la date du paiement (la date saisie), et la confirmation d'un lot sa date de paiement ; la liste des motifs
    qui empêchent d'annuler le règlement (`:1627-1640`) gagne « le paiement est **rapproché** d'une transaction
    bancaire : annulez d'abord le rapprochement, depuis le détail de l'import » (P1 R-L6).
  Les manuels DE/IT/EN : noter « à traduire » si l'écart existe (règle du dépôt).
- **`docs/api-external.md`** : `POST /reconciliation/accept` (`:572-580`, type `supplier_invoice`, codes et raisons
  neufs, `details.tolerance_days`, `supplierInvoiceId` de `accepted[]`), `GET /reconciliation/proposals` (candidate
  `supplier_invoice`, `supplierInvoiceId`, `contactName`, `notice` et ses deux codes (`supplier_invoice_in_payment_batch`, `reference_of_paid_supplier_invoice`), la priorité, les factures qui
  masquent la règle, les lignes qui ne proposent rien), les codes `RECONCILIATION_SUPPLIER_PAYMENT_HOLD` et
  `RECONCILIATION_RULE_MASKED` (raisons, détails, et le **changement de contrat** des règles, **des deux côtés** — crédit
  et débit —, avec le renvoi vers `POST /reconciliation/manual`, P5 R1-5), **L1 et sa forme sans référence**, les
  **changements** (un débit reçoit désormais des factures fournisseur ; « un débit ne règle jamais une facture » devient
  « jamais une facture **client** ») ; **`paidAt`** de `POST /supplier-invoices/{id}/pay` et de la confirmation d'un lot
  (date du paiement à minuit, au lieu de l'heure de l'enregistrement — P1 R9) ; `:366` (événements de lettrage : la
  clé figure aussi au lettrage d'une facture **fournisseur** réglée par la réconciliation, `documentType` du fournisseur
  — P2, F-L6 = R-L11). La phrase de `:788` est traitée par la 30-1a (AC-a8).
- **i18n quatre locales** pour chaque clé neuve ou réécrite ; `npm run lint-i18n-ownership` vert.
- **CHANGELOG** `## [0.14.0] — Non publié` (P4, R4 = F7) : `### Ajouté`, une entrée pour #686 (le processus,
  l'annulation, les limites — montant exact, fenêtre, lot exclu et signalé —, L1 et son geste sûr, la mention API) ;
  `### Corrigé`, la date du paiement d'une facture fournisseur (`paid_at`) ; `### Modifié`, l'acceptation d'une règle
  désormais refusée quand une facture la défend (`RECONCILIATION_RULE_MASKED`, client compris) ; aucun `closes`. Les
  entrées de la 30-1a sont écrites par la 30-1a (AC-a7).
- **README** « Fonctionnalités », ligne de l'import bancaire : « réconciliation des débits avec les factures
  fournisseur » *(à paraître en v0.14.0)*.

**AC-b14 — Les registres du dépôt.**
- `crates/kesh-api/tests/bank_account_closed_inventory.rs` : le nouvel écrivain de `bank_transactions`
  (`accept_one_supplier_invoice`) au `REGISTRE` avec son statut (garde par l'étape 3 (b), compte sous `FOR UPDATE`) ;
  le compteur anti-muet `assert_eq!(sites.len(), 32, …)` (`:508`) **recompté** (33 attendus) ; l'entrée `pay_in_tx` →
  `pay_in_tx_as` (AC-b9) ; le `FENETRE` si une garde non verrouillante apparaît (aucune n'est prescrite).
- `crates/kesh-db/tests/letterings_lexical.rs` : `pay_in_tx_as` (AC-b9).
- **IDOR et rôle** : dans `crates/kesh-api/tests/reconciliation_e2e.rs` (où vivent les gardes d'isolation de la
  réconciliation, patron `:2401-2475`), **pas** dans `idor_multi_tenant_e2e.rs` ni `rbac_e2e.rs`, qui n'ont pas de
  harnais de réconciliation : T-b32 (facture fournisseur d'une autre société → `SUPPLIER_INVOICE_NOT_FOUND`, `details`
  identiques à ceux d'une facture absente, rien écrit), T-b33 (rôle Consultation → `403`).
- `audit_labels.rs` / `audit_label_registry.rs` si une action ou un champ d'audit affiché change ; `textes_coherents.rs`
  (AC-a6) ; toute liste fermée de codes de `failed[]` (Rust ou TypeScript) recomptée.
- **Doc-comments qui comptent « six chemins » qui rapprochent** — sept après la 30-1b (l'acceptation fournisseur crée
  son écriture, comme les cinq premiers) : `reconciliation_cancel.rs:11`, `bank_imports.rs:380`, `bank_accounts.rs:1701`
  (ce dernier énonce l'invariant « une transaction, une écriture », dont le chemin neuf hérite) ; `grep -rnF "six
  chemins" crates` refait (P2, F-L3).

**AC-b15 — Aucune migration.**
Ni table, ni colonne, ni index, ni contrainte : `ls crates/kesh-db/migrations/*.sql | wc -l` reste à **81**,
`kesh_version_min_required` à 0.13.1 (vérifié en P1 : `fk_supplier_invoices_settlement_je`,
`idx_payment_batch_items_invoice`, `idx_supplier_invoices_company_status` existent). Si le développement découvre
qu'un index manque à la requête de l'AC-b3, il l'**écrit** comme angle mort mesuré, il n'ajoute pas de migration sans
retour à la fiche.

**AC-b17 — L'acceptation d'une règle rejoue le masquage (ex-L7, C-30-1-38, simplifié en P4 : C-30-1-39).**
La proposition n'offre pas la règle quand une facture la défend (AC-b5 ; côté client, la 29-6 :
`rule_masked_by_overpaid_invoice`, `routes/reconciliation.rs:887`), mais `accept_one_rule` (`:2912`) ne revérifie aucun
de ces critères : un appel d'API direct, ou un écran **périmé** — l'usage même de la story : l'écran de réconciliation
ouvert **avant** la saisie de la facture dans un autre onglet (P4, R1 = F5) — accepte la règle et compte la charge (ou le
produit) deux fois. **Correction**, dans le même fichier : après la revérification `rule_matches` de l'étape 7
(`:3119-3126`) et avant toute écriture, `accept_one_rule` rejoue, **en lectures ordinaires**, par **les mêmes
fonctions** que `get_proposals` (DRY, une fonction `rule_mask_for(tx, …)` lue par les deux) :
- **débit** : (a) la fonction du lot du compte (AC-b3) → raison `payment_batch` ; (b) `has_strong_invoice_match` sur les
  candidates fournisseur de la fenêtre (même requête, même `propose_matches`, même seuil 0,5 que la proposition, et
  **la même carte de contacts**, chargée pour les factures qu'elle note — P5, F5) → raison `invoice_candidate` ; (c) la
  fonction de la référence d'une facture payée (AC-b3) → raison `paid_invoice_reference` ;
- **crédit** : la condition de la 29-6 — une facture que le paiement dépasse de 1 à 5 centimes
  (`find_invoices_slightly_overpaid_for_window`) et qui porte un indice (`propose_matches` non vide sur elle, comme à la
  proposition ; P4 F : sur ces factures, `amount_score` vaut 0, « non vide » vaut donc « indice ») → raison
  `invoice_slightly_overpaid`.
Refus **par proposition** : `RECONCILIATION_RULE_MASKED` (neuf), `details.reason` ∈ { `payment_batch`,
`invoice_candidate`, `paid_invoice_reference`, `invoice_slightly_overpaid` }, avec `invoiceId` ou `supplierInvoiceId`, et `paymentBatchId` pour un
lot. **L'imputation délibérée** d'une transaction malgré une facture reste possible par *Affecter manuellement*
(`POST /reconciliation/manual`) ou *Éclater* — non par la règle. ⚠️ **Non rejoués — inventaire clos** (P5, F6), écrits au doc-comment et au manuel : côté **client**, le masquage par une
facture **candidate** — comportement d'aujourd'hui, gardé pour ne rien retirer à un client existant, alors que le côté
débit, neuf, n'a pas d'acceptation antérieure à préserver (P5, R1-8) ; côté **fournisseur**, quatre formes que la
proposition masque sans les refuser à l'acceptation : la facture presque au montant portant un indice, la facture
exacte hors fenêtre portant un indice, la facture exacte dans un lot d'un **autre** compte portant un indice, et la
**liste pleine** de 200. Une règle ne se refuse donc à l'acceptation que dans **quatre** cas : `payment_batch`,
`invoice_candidate`, `paid_invoice_reference` (débit), `invoice_slightly_overpaid` (crédit).
Lectures ordinaires : un lot ou une facture commis après la naissance de la vue de la transaction du lot ne sont pas vus
— même tolérance que la proposition. ⚠️ **Changement de contrat, des deux côtés** (P5, R1-5) : une acceptation de
règle autrefois réussie est désormais refusée — crédit : le cas de la 29-6 ; débit : les trois cas ci-dessus (un débit
n'avait aucune facture en v0.13.4) ; l'intégration garde *Affecter manuellement*, unitaire — écrit à `api-external.md` (§ `POST /reconciliation/accept`) et
au CHANGELOG (*Modifié*). Modules : `routes/reconciliation` et `repositories/reconciliation`, déjà dans la 30-1b —
**aucun module ajouté** (toujours 5).
*Tests* : **T-b46** (API directe, crédit : règle acceptée alors qu'une facture client dépassée de 3 centimes porte la
référence → `RECONCILIATION_RULE_MASKED` / `invoice_slightly_overpaid`, rien écrit ; sans indice → la règle passe),
**T-b47** (API directe, débit : facture fournisseur en lot `generated` du compte → `payment_batch` ; facture fournisseur
ouverte **candidate**, saisie après le chargement de la proposition → `invoice_candidate` ; référence d'une facture
payée → `paid_invoice_reference` ; rien écrit ; sans facture → la règle passe). T-b46 gagne un second décor : facture
client dépassée de 3 centimes désignée **par le seul nom** du client → refus (la carte de contacts du rejeu, P5 F5 ;
mutation M-b33). Mutation **M-b30** (rejeu du masquage retiré de `accept_one_rule`).

**AC-b16 — Un règlement fournisseur porte la date du paiement (décision de l'orchestrateur, C-30-1-25).**
`pay_in_tx_as`, étape (6) (`supplier_invoices.rs:905-907`) : `paid_at = <payment_date> 00:00:00` (la même forme que
le client, `routes/reconciliation.rs:1995-1996` : `and_time(NaiveTime::from_hms_opt(0,0,0))`), **lié en paramètre**,
au lieu de `CURRENT_TIMESTAMP(3)`. Pour **tous** les appelants, une seule définition :
- *Accepter* (30-1b) : la date de paiement de la transaction (`payment_date(tx)` : valeur, à défaut comptabilisation) ;
- la fiche (`POST /supplier-invoices/{id}/pay`) : le `paymentDate` **obligatoire** du corps
  (`PaySupplierInvoiceRequest.payment_date: NaiveDate`, `routes/supplier_invoices.rs:256-284`), saisi par
  l'utilisateur (champ date, aujourd'hui par défaut, `supplier-invoices/[id]/+page.svelte:59`, `:371`) — la date même
  de l'écriture de règlement ;
- la confirmation d'un lot (`confirm_batch`, `payment_batches.rs:432-503`) : le `payment_date` qu'elle reçoit — la date
  de l'écriture de règlement du lot. ⚠️ **Changement visible** : jusqu'ici, l'heure de la confirmation.
Aucune contrainte ne s'y oppose : `chk_supplier_invoices_paid_has_settlement` (migration
`20260628000001_supplier_invoices.sql:73`) n'exige que `paid_at IS NOT NULL` ; aucun `chk_*paid_at_after_date`
fournisseur (vérifié en P1, F et R). **Aucune migration** ; les factures déjà payées gardent leur `paid_at` (rien n'est
réécrit ; l'écart est dit au CHANGELOG, au manuel et à `api-external.md`, AC-b13).

**Lecteurs de `supplier_invoices.paid_at`, inventaire clos** (`grep -rln supplier_invoices crates/*/src | xargs grep -n
paid_at`, `grep -rn paidAt frontend/src`, relevé sur `15686d55`, refait en P1 par R sur `crates/*/src`, `kesh-report`,
`kesh-seed`, `frontend/src`, `open_items.rs`, `aged_receivables.rs`) :

| Lecteur | Site | Effet |
|---|---|---|
| réponse de la fiche (`SupplierInvoiceResponse.paid_at`) et son affichage « Facture réglée. <paidAt> » | `routes/supplier_invoices.rs:80`, `:147` ; `supplier-invoices/[id]/+page.svelte:393` | affiche la date du paiement (minuit) au lieu de l'heure de la saisie — c'est le but |
| export CSV `supplier_invoices.csv`, colonne `paid_at` | `kesh-api/src/exports/csv_tables.rs:1215`, `:1241` | idem ; format inchangé |
| audits `supplier_invoice.cancelled` (`paidAt`) et `supplier_invoice.settlement_cancelled` | `supplier_invoices.rs:1190`, `:1449` | portent la date du paiement |
| sauvegarde / restauration, listes `list_all_by_company` | `supplier_invoices.rs:1566` | colonne recopiée telle quelle |
| contrainte `chk_supplier_invoices_paid_has_settlement` | migration `:73` | non nul : tenu |

**Aucun** rapport, échéancier, poste ouvert ni lettrage ne lit `supplier_invoices.paid_at` : ils lisent les **dates des
écritures** (`entry_date`, déjà la date du paiement) ; l'échéancier (`dunning`, `open-items`, `aged_receivables`) ne vise
que les factures **client**.
*Tests* : T-b29 (fiche : `paymentDate` antérieur de 40 jours → `paid_at` = cette date à minuit, rouge au sol), T-b30
(acceptation : `paid_at` = date de valeur de la transaction ; date de comptabilisation si la date de valeur manque),
T-b31 (lot confirmé avec un `payment_date` passé → `paid_at` = cette date). Mutation M-b15.

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
  rouge (le bras `Entry` refuse : `OWNED_BY_SUPPLIER_INVOICE`).
- [ ] T2 (AC-a1, a4) — `ReconciliationKind::SupplierSettlement`, `classify`, doc-comments (ordre, coût de l'intervalle).
- [ ] T3 (AC-a3) — `supplier_settlement_cancel_blocker_unlinking` ajoutée ; l'existante en devient l'enveloppe.
- [ ] T4 (AC-a2, a6) — bras de l'étape (6) ; audit ; commentaire `errors.rs:4237-4238` réécrit.
- [ ] T5 (AC-a5) — route de la vue et de `…/cancel` ; T-a7, T-a8 (`crates/kesh-api/tests/reconciliation_e2e.rs`).
- [ ] T6 (AC-a4) — T-a6, T-a11 (courses), T-a12 (plan).
- [ ] T7 (AC-a6) — texte commun « rapproché » : quatre locales, repli serveur (`errors.rs:3736-3739`), repli de l'écran
  (`settlement-cancel-blocked.ts:51`) ; T-a10 (Vitest), T-a13 (`textes_coherents.rs`).
- [ ] T8 (AC-a8) — rejeu de la création d'un lot ; T-a14 ; `audit_route_registry.rs` (vi) et (iii bis).
- [ ] T9 (AC-a9) — `create_batch` verrouille ses factures par identifiant croissant avant toute lecture (liste vide
  court-circuitée, borne de 500 à la route, plan épinglé) ; aide `attendre_des_requetes_en_cours` ; T-a12 étendu, T-a15,
  T-a16, T-a17 (route).
- [ ] T10 (AC-a7, a8) — `api-external.md` (§ *Annuler un rapprochement*, `:788` *Interblocages*).
- [ ] T11 — mutations M-a1 à M-a9 (M-a2 équivalente déclarée, M-a6 retirée) **sous le verrou de gate** ; gate complet
  et Vitest complet au dernier commit de code.

### 30-1b

- [ ] T12 — Brancher `story/30-1b` depuis `main` après le merge de la 30-1a ; relire les numéros de ligne.
- [ ] T13 (AC-b1) — tests rouges T-b1 à T-b3, T-b37 ; traits, généricité, valeur absolue, renommage ; tous les tests de
  `matching.rs` verts sans toucher leurs assertions.
- [ ] T14 (AC-b2, b3) — tests rouges T-b4 à T-b10, T-b42, T-b48 (`crates/kesh-db/tests/reconciliation_repository.rs`) ;
  fenêtre, traduction commune des refus de dates, requêtes.
- [ ] T15 (AC-b4, b5) — tests rouges T-b11 à T-b16, T-b35, T-b36, T-b41, T-b49 ; `get_proposals`, `candidate_rank`, masquage,
  `notice`.
- [ ] T16 (AC-b6, b7, b8, b9, b17) — tests rouges T-b17 à T-b23, T-b45, T-b50 (acceptation directe refusée par les gardes
  (d) et (e)), T-b46, T-b47 (règle acceptée par l'API, masquage rejoué) ; `rule_mask_for` ; `lock_for_bank_settlement_in_tx`, `pay_in_tx_as`,
  `accept_one_supplier_invoice`, `refuse_wrong_direction`, inventaire des refus recopié au Dev Agent Record.
- [ ] T17 (AC-b10) — T-b24, T-b25, T-b34 ; table des verrous recopiée au Dev Agent Record avec les lignes réelles.
- [ ] T18 (AC-b11) — écran, dialogue, libellés, types ; Vitest T-b26 à T-b28, T-b38 à T-b40.
- [ ] T19 (AC-b16) — tests rouges T-b29 à T-b31 ; `paid_at` lié ; inventaire des lecteurs refait et recopié au Dev Agent
  Record.
- [ ] T20 (AC-b14) — registres, doc-comments « six chemins », T-b32, T-b33.
- [ ] T21 (AC-b12) — E2E (trois tests) et captures ; helper `createSupplierInvoiceViaApi`.
- [ ] T22 (AC-b13) — manuel + PDF (`make fr` dans `docs/manual/`, sous le verrou), `api-external.md`, i18n, CHANGELOG,
  README.
- [ ] T23 — mutations M-b1 à M-b30 (M-b28 retirée) sous le verrou ; gate complet, Vitest complet **et E2E complet au dernier commit de
  code** (`flock`, recette de `docs/testing.md` à la lettre, port 3051, bases `kesh_301` / `kesh_e2e_301`).

---

## Dev Notes

### La conception, en une phrase par geste

- **Proposer** : la même boucle que pour un crédit, alimentée par d'autres candidates (factures fournisseur ouvertes au
  montant exact), notée par la **même** fonction générique (montant en valeur absolue), ordonnée par la **même**
  priorité, et la règle masquée par la même famille de garde que la 29-6.
- **Accepter** : `pay_in_tx_as` — le geste de la fiche, **appelé** — précédé de verrous (facture, compte, lot) et de
  gardes propres au rapprochement, suivi du lien de la transaction : ce que fait `accept_one_invoice` pour le client,
  sans ses étapes de reste dû, de trop-perçu et d'arrondi, qui n'existent pas côté fournisseur.
- **Annuler** : le dé-rapprochement existant, avec un troisième bras qui **appelle** `cancel_settlement_in_tx`.

### Pourquoi réutiliser `pay_in_tx` et non écrire un « accept_one_supplier » complet (C-30-1-10)

Le règlement fournisseur a **une** définition, éprouvée par la fiche et par les lots : dette lue sur l'écriture
d'achat, contrepartie gardée, compte clos lu sous verrou, exercice, lettrage. L'écrire une seconde fois dans la route
créerait le jumeau que le dépôt combat partout (« appelé, jamais réécrit »). Ce que la route ajoute n'existe pas dans
`pay_in_tx` : le sens de la transaction, la fenêtre, l'égalité de montant, la garde verrouillante du lot, le lien.

### Le montant (C-30-1-5)

Pas de reste dû : le règlement est binaire, du TTC lu sur l'écriture d'achat (`purchase_payable_line`), écrit des
**deux** côtés (`D créanciers / C banque`). La ligne de banque doit égaler le relevé — c'est la raison d'être de
l'écriture de règlement (patron 24-2). Donc **égalité exacte** `|tx.amount| = total_amount`, et la note compare la
**valeur absolue** (AC-b1). Un TTC à plus de deux décimales n'est jamais proposé (angle mort L2, #687). ⚠️ `pay_in_tx`
lit le montant sur l'écriture d'achat, la proposition sur `total_amount` : les deux sont égaux par construction
(`generate_purchase_journal_lines`, `:155-161`) — T-b17 l'asserte sur la facture réglée.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `routes/reconciliation.rs` : le chemin crédit de `get_proposals` **inchangé** en comportement (les tests des Stories
  28-7, 29-2, 29-3, 29-5, 29-6, 29-9 restent verts sans retouche) ; le pré-vol par lot (27-4), le compte clos (28-4a),
  le rejeu (`retry_with`, 25-4-c2) ; `accept_one_invoice` **inchangé** hors l'appel de `refuse_wrong_direction` et de la
  traduction commune des refus de dates (dont `details` gagne seulement `tolerance_days`).
- `matching.rs` : le score client, au bit près (l'égalité « 0,40 + 0,10 contre 0,50 » de la 29-5 en dépend).
- `supplier_invoices.rs` : `pay`, `pay_in_tx` (signature) et `confirm_batch` au comportement inchangé hors `paid_at`
  (AC-b16) ; `in_generated_batch` non verrouillant pour les vues ; `supplier_settlement_cancel_blocker` à signature
  inchangée.
- `reconciliation_cancel.rs` : l'ordre « aucune lecture non verrouillante avant l'étape 3 » ; la revérification
  verrouillante en dernière acquisition (29-4b).

### Fichiers communs avec la 30-2

`supplier_invoices.rs` (la 30-2 crée des factures depuis un document, elle ne touche pas au paiement — à confirmer au
rebase), manuel § *Factures fournisseurs*, CHANGELOG, catalogues i18n, `helpers/api-fixtures.ts`. Union au rebase.

### Tests (récapitulatif)

30-1a : T-a1 à T-a17 sauf T-a9 (déplacé en T-b40) = **16** (15 Rust, 1 Vitest : T-a10). 30-1b : T-b1 à T-b50 sauf T-b43 et T-b44 (retirés en P4) = **48**
(42 Rust, 6 Vitest : T-b26, T-b27, T-b28, T-b38, T-b39, T-b40) et l'E2E de l'AC-b12 (**trois** tests). Décors
**synthétiques** ; aucune donnée réelle (`docs/agents-orchestration.md` § 8). Concurrence par `sonde_verrou_nowait` /
`attendre_une_requete_en_cours` (`test_fixtures.rs:561`, `:611`), **jamais** `information_schema.INNODB_TRX` (piège
connu des consignes).

### Mutations (à exécuter sous `flock`, restaurées par `git checkout -- f && touch f`)

| # | Mutation | Test qui doit la tuer |
|---|---|---|
| M-a1 | `classify` sans le bras fournisseur | T-a1, T-a2 |
| M-a2 | bras fournisseur classé avant `invoice_settlements` (inversion) | **équivalente déclarée** (P1, F10 et R-L2 ; confirmée en P2) : une écriture ne peut être à la fois `invoice_settlements.journal_entry_id` et `settlement_journal_entry_id` ; seule l'empreinte de verrou d'intervalle diffère. Non exécutée |
| M-a3 | `cancel_blocker` fournisseur en forme **non** exemptée (`None`) | T-a5 |
| M-a4 | étape (6) fournisseur par `reverse_in_tx` au lieu de `cancel_settlement_in_tx` | T-a2 |
| M-a5 | lecture de classement fournisseur **sans** `FOR UPDATE` | T-a11 (et non T-a6 — P1, F4) |
| ~~M-a6~~ | ~~dialogue : texte client pour `supplier_settlement`~~ — **retirée** avec le dialogue, devenue M-b26 | — |
| M-a7 | `create_batch` sans le verrouillage des factures en tête (AC-a9) | T-a15 |
| M-a8 | rejeu de `create_batch` retiré (AC-a8) | T-a14 |
| M-b1 | `propose_matches` fournisseur sans la référence de paiement | T-b1 |
| M-b2 | référence de paiement par containment au lieu d'égalité | T-b2 |
| M-b3 | blancs non retirés | T-b1 |
| M-b4 | borne haute `+1` au lieu de `+7` (ou inverse) | T-b4 |
| M-b5 | borne basse sur `F` au lieu de `X` | T-b4, cas **échéance postérieure à la date**, `F` hors fenêtre et `X` dedans (P2 F-L5 : T-b6 a `X = F` et ne la tue pas) |
| M-b6 | filtre de lot `generated` retiré de la requête des candidates | T-b8 |
| M-b7 | égalité de montant remplacée par la tolérance client | T-b9 |
| M-b8 | appartenance au lot par lecture **ordinaire** | T-b22 |
| M-b9 | `|amount|` d'un débit envoyé à `find_unpaid_invoices_for_window` (garde #482) — ⚠️ le simple retrait de la garde `:749` est **équivalent** (la requête client reçoit un montant négatif et ne rend rien, P2 R-L14) | T-b11 |
| M-b10 | crédit envoyé aussi aux factures fournisseur | T-b12 |
| M-b11 | `candidate_rank` : fournisseur en rang 2 | T-b14 |
| M-b12 | facture presque au montant avec indice ne masque pas la règle | T-b16 |
| M-b13 | `refuse_wrong_direction` : `is_sign_negative()` | T-b21 (`-0.00`) |
| M-b14 | `pay_in_tx_as` audit par `user` au lieu de `for_actor` | T-b23 |
| M-b15 | `paid_at = CURRENT_TIMESTAMP(3)` rétabli | T-b29, T-b30 |
| M-b16 | montant comparé **signé** (sans `abs`) | T-b1, T-b11, T-b17 |
| M-b17 | majuscules non appliquées à la normalisation | T-b1 |
| M-b18 | filtre `status = 'open'` retiré (AC-b3) | T-b7 |
| M-b19 | `BankTransfer` avec un compte autre que celui de l'import | T-b17 |
| M-b20 | facture exacte en lot `generated` du compte ne bloque pas la ligne | T-b35 |
| M-b21 | facture exacte hors fenêtre, avec indice, ne masque pas la règle | T-b36 |
| M-b22 | numéro ou référence de paiement par containment, ou sans longueur minimale | T-b37 |
| M-b23 | articles du lot lus **avant** le verrou du compte | T-b34 (chorégraphie de l'AC-b10) |
| M-b24 | lot `generated` du compte : la règle masquée mais les candidates **gardées** | T-b41 |
| M-b25 | lot `generated` d'un **autre** compte traité comme du compte (sur-blocage) | T-b41 (second décor) |
| M-b26 | dialogue : texte client pour `supplier_settlement` | T-b40 |
| M-a9 | verrouillage de tête de `create_batch` dans l'ordre de la requête (non croissant) | T-a16 |
| M-b27 | étape 3 (b) sans `FOR UPDATE` (lecture ordinaire du compte) | T-b34 — réécrite en P4 : sans le verrou de (b), les articles sont lus en S **avant** que le compte soit pris (par `pay_in_tx_as`), l'ordre même de M-b23 ; la garde (d), en lecture ordinaire, ne dépend plus de ce verrou (C-30-1-39) |
| ~~M-b28~~ | ~~règlement en attente de rapprochement : la ligne garde ses propositions~~ — **retirée** avec le critère (C-30-1-34, P4) | — |
| M-b29 | garde (d) de l'acceptation retirée (`excluding` retiré en P5) | T-b45 |
| M-b30 | rejeu du masquage retiré de `accept_one_rule` (AC-b17) | T-b46, T-b47 |
| M-b31 | garde de la référence d'une facture payée absente de la proposition | T-b49 |
| M-b32 | garde (e) de l'acceptation d'une facture retirée | T-b50 |
| M-b33 | rejeu de `accept_one_rule` sans la carte de contacts (indice par le nom perdu) | T-b46, second décor |
| M-b34 | référence d'une facture payée comparée sans la longueur minimale de 6 | T-b49 (référence de 5 caractères) |

Le `status = 'paid'` du classement (AC-a1) n'est pas muté : équivalent (tout écrivain qui quitte `paid` remet la
colonne à `NULL`, P1 F10). Une mutation qui survit se tue par un test ajouté, ou se déclare **équivalente** avec la
raison, au Dev Agent Record.

**Tests de non-régression, sans mutation dédiée** (P5, F8 — écrit, pour que « tuée » reste vérifiable) : T-a6 (course
dé-rapprochement ↔ annulation de la facture : établit l'absence d'état mixte, aucune ligne neuve ne la décide), T-b5
(parité SQL/Rust de l'exigibilité : garde de dérive), T-b10 (`LIMIT 50` après l'ordre : patron hérité), T-b24 (le refus
vient de `pay_in_tx_as`, inchangé), T-b25 (I1 protège, la fiche le dit), T-b38 (libellés lus sur `details`) ; T-b42
(liste pleine) est tué par M-b12 étendue à « la liste pleine ne masque pas ».

### Angles morts assumés (à écrire aux doc-comments et au manuel)

- **L1 — Débit d'une facture déjà payée, depuis sa fiche ou par la confirmation d'un lot** : la facture est `paid`,
  non proposée ; la transaction reste en attente, et deux propositions peuvent la compter deux fois : (i) la **règle**
  d'affectation — **préexistant** (v0.13.4) ; (ii) **créé par la 30-1b** (P5, F1 = R1-6) : une **autre facture ouverte
  du même montant**, typiquement la mensualité suivante, dans la fenêtre. **Réduit en P5** : quand le débit porte la
  référence (≥ 6 caractères) de la facture payée, la ligne ne propose rien et le dit (AC-b5, `notice`
  `reference_of_paid_supplier_invoice`) ; **reste** la forme **sans référence**, acceptée par l'orchestrateur et portée
  au Project Lead, écrite au manuel, à `api-external.md` et au CHANGELOG avec le geste sûr complet (AC-b13 : la
  transaction reste en attente et gêne la clôture du compte jusqu'à #671 ; exercice clos : aucun geste). Territoire de
  **#671** (première et quatrième puces). La garde de la P3 (C-30-1-34) est **retirée** en P4 : elle sur-bloquait (une
  mensualité payée depuis la fiche bloquait le débit de la suivante), sans geste sûr à l'écran, et chaque passe y
  trouvait un double comptage neuf. Le manuel dit le geste sûr (AC-b13) ; l'écran ne dit rien.
- **L9 — Débit collectif d'un lot** (P4, F-L4) : le pain.001 n'émet pas `BtchBookg` ; la banque débite souvent le total
  d'un lot de plusieurs factures, qui n'égale aucune facture : aucune candidate, la règle peut être proposée — même
  remède que L1 (#671, quatrième puce).
- **L10 — Affecter manuellement et Éclater ne rejouent aucune garde** (P4, R12) : délibéré — ce sont les gestes
  d'imputation volontaire que l'AC-b17 désigne ; le manuel le dit.
- **L2 — TTC à plus de deux décimales** (C-30-1-26) — tracé par **#687** : jamais proposé ; la facture se règle depuis la
  fiche, qui écrit alors une ligne de banque à plus de deux décimales. Origine : `line_total = quantité × prix
  unitaire` sans arrondi (`supplier_invoices.rs:389`, `:415`) — et **aucune** limite d'échelle à la création
  fournisseur (ni `scale_within` ni `MAX_DECIMAL_SCALE` dans `routes/supplier_invoices.rs` ni
  `routes/imported_supplier_invoices.rs`), alors que le client borne quantité et prix à 4 décimales
  (`routes/invoices.rs:482-501`, `limits.rs:28`) et arrondit `line_total` à 4 décimales
  (`repositories/invoices.rs:456-459`). ⚠️ **Le client n'évite pas un TTC sous le centime** : il le **traite au
  règlement**, par le reste dû arrondi au centime et une troisième ligne sur le compte de différences d'arrondi (#476).
  Correction non mécanique (création ou règlement à refondre). Une correction côté serveur (création) reste compatible
  avec la 30-2b, qui refait le **formulaire** de création.
- **L3 — Écart de frais bancaires** (débit = TTC + frais) : non proposé ; la règle masquée si l'écart ≤ 0,05 et un
  indice (AC-b5), sinon rien. Hors périmètre (pas de règlement partiel fournisseur).
- **L4 — Le nom de contrepartie d'un débit** (`camt053/mod.rs:459-474`) : non vérifié sur relevé réel ; poids 0,10.
- **L5 — Facture masquante au-delà de 0,05** (frais bancaires plus élevés, escompte) : la règle reste proposée ; même
  borne que la 29-6, écrite au manuel.
- **L6 — `paidAt` affiché brut** sur la fiche fournisseur (`supplier-invoices/[id]/+page.svelte:393` : après la 30-1b,
  « 2026-06-20T00:00:00 ») : cosmétique, préexistant (avant : l'heure de la saisie, brute aussi) ; hors des modules de la
  30-1b — **déjà intégré à la 30-2c (AC25)**, sur décision de l'orchestrateur : pas d'issue.
- ~~L7 — Acceptation d'une règle par l'API, sans garde de masquage~~ : **intégré** à la 30-1b (AC-b17, C-30-1-38) ;
  reste seulement l'acceptation délibérée d'une règle contre une facture **candidate**, inchangée (AC-b17).
- **L8 — La `notice` ne cite qu'une facture** : quand une candidate à signal est effacée par un lot du compte (AC-b5,
  première ligne), la `notice` ne dit pas qu'une autre facture de ce montant existait (P3, F11) — pur confort.

### Pièges connus du dépôt, applicables ici

- `kesh-db` touché : **gate complet même en boucle** de revue ; base de gate remise à zéro avant tout gate complet.
- P8 : aucune migration modifiée (aucune migration du tout).
- Mutations : `touch` après restauration (cargo compare des dates).
- Codes neufs plutôt que réemploi (C-30-1-18) ; `refuse_wrong_direction` (C-30-1-21) ; garde du lot verrouillante
  (C-30-1-22) ; traits génériques (C-30-1-23) ; acteur de `pay_in_tx_as` (C-30-1-20).
- E2E : `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés, les quatre `KESH_SMTP_*`,
  `KESH_ADMIN_BACKUP_DIR` inscriptible.
- Manuel : contrôler le **PDF** aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, sortie standard seulement).
- Commits : `refs #686`, jamais `closes`/`fixes`, ni « validation close » (écrire « validation terminée »).
- « Recompter ses comptes rendus » : nombres de tests, de mutations, de codes, avec leur périmètre.

### Issues

- `paid_at` : intégré à la 30-1b (AC-b16), pas d'issue.
- TTC fournisseur à plus de deux décimales : **#687** (ouverte par l'orchestrateur).
- #671 : commentée par l'orchestrateur (apport de la 30-1a).
- Même facture dans deux lots `generated` (`create_batch`) : **intégré** à la 30-1a (AC-a9), pas d'issue.
- `paidAt` affiché brut sur la fiche fournisseur (L6) : intégré à la **30-2c (AC25)**, pas d'issue.
- Acceptation d'une règle par l'API sans revérifier le masquage (ex-L7) : intégré à la 30-1b (AC-b17), pas d'issue.

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
tests nommés (T-a1 à T-a10, T-b1 à T-b28) plus un E2E de parcours à deux tests, 20 mutations (6 + 14).
*(Valeurs à la création ; amendées le même jour, voir l'entrée suivante.)* Choix C-30-1-1 à
C-30-1-24 (recomptés par `grep` sur cette fiche et le registre).

### Amendement de l'orchestrateur — 2026-10-15

Deux défauts du chemin de *Accepter* (`pay_in_tx`), signalés par la fiche :
- **`paid_at`** : intégré à la 30-1b (AC-b16, T16-bis, T-b29 à T-b31, M-b15 ; C-30-1-25). Lecteurs inventoriés : réponse
  et affichage de la fiche, export CSV, deux audits, sauvegarde ; aucun rapport, échéancier, poste ouvert ni lettrage.
  Changement visible pour la confirmation d'un lot (date du paiement au lieu de l'heure de confirmation).
- **TTC à plus de deux décimales** : origine établie (aucune limite d'échelle à la création fournisseur, `line_total`
  non arrondi) ; la prémisse « le client arrondit à la saisie » est **fausse** — le client traite l'écart au règlement
  (#476). Correction non mécanique : angle mort L2, **issue à ouvrir** (C-30-1-26).
- `pay_in_tx` garde sa signature (enveloppe de `pay_in_tx_as`) : `payment_batches.rs` n'est plus touché (AC-b9 ;
  C-30-1-20 amendé).
- **Recompte des modules de la 30-1b** (C-30-1-27) : `kesh-reconciliation/matching`, `kesh-db/repositories/
  reconciliation`, `kesh-db/repositories/supplier_invoices`, `kesh-api/routes/reconciliation`,
  `frontend/src/lib/features/reconciliation` = **5** — au seuil, non au-delà (`> 5`) ; aucun nouveau découpage.
- Décomptes : **23** AC (7 + 16), **22** tâches (T0 à T20 + T16-bis), **41** tests nommés (T-a1 à a10, T-b1 à b31 ;
  5 Vitest), **21** mutations (6 + 15), choix C-30-1-1 à **27**.

### Validation P1 — 2026-10-15

Sonnet, deux lentilles en contexte frais (rapports `kesh-gate-logs/301-validate-p1-{F,R}.md`, sur `4862053a`) :
**F 0/2/4/5**, **R 0/2/7/9**. Chaque finding vérifié au code (`grep -nF`, `sed -n`) avant patch ; **aucun réfuté**.
Remédiation (arbitrages de l'orchestrateur compris) :
- **R1 = F2 (HIGH)** — montant comparé signé : la note compare la **valeur absolue** (AC-b1) ; T-b1 sur transaction
  négative, M-b16. C-30-1-16 (pas de garde de score) **maintenu** : son énoncé, corrigé, tient (« ≥ 0,5 puisque
  l'égalité est vérifiée et la note en valeur absolue »), et T-b17 l'asserte.
- **R2 = F3 (HIGH/MEDIUM)** — démasquage de la règle : critère écrit à l'AC-b5 (facture exacte en lot `generated` : bloque
  toujours, raison visible `notice` ; exacte hors fenêtre ou à 0,05 près : bloque si indice) ; requête
  `find_supplier_invoices_masking_rule` (AC-b3) ; T-b35, T-b36 ; M-b20, M-b21 ; écran (AC-b11, T-b39) ; L5 ajouté.
- **F1 = R7 (HIGH/MEDIUM)** — verrous : compte bancaire `FOR UPDATE` **avant** la lecture des articles de lot, ordre
  facture → compte → articles (celui de `pay_in_tx` et `create_batch`) ; table complète des chemins et des cycles
  (AC-b10, C1 à C6) ; C1 inhérent à un lot de plusieurs propositions → la **création d'un lot est rejouée** (AC-a8, dans
  la 30-1a) ; intervalle sur `idx_payment_batch_items_invoice` écrit (AC-b8) ; T-b34, M-b23.
- **F4 (MEDIUM)** — M-a5 tuée par T-a11 (course contre la clôture d'exercice), non par T-a6.
- **F5 = R3 (MEDIUM)** — inventaire des refus : `InactiveOrInvalidAccounts` classé ; `FiscalYearClosed` /
  `DateOutsideFiscalYear` classés (inatteignables) ; ambiguïté de `NotFound` / `IllegalStateTransition` levée par
  l'étape 3 (facture et compte lus et verrouillés : à l'étape 6, projet seulement).
- **F6 = R4 (MEDIUM)** — registres `bank_account_closed_inventory.rs` (`pay_in_tx_as`, compteur 32 → 33) et
  `letterings_lexical.rs` nommés (AC-b9, AC-b14).
- **R5 (MEDIUM)** — IDOR et rôle dans `reconciliation_e2e.rs` : T-b32, T-b33.
- **R6 (MEDIUM)** — libellés lus sur `details.tolerance_days` et `details.claim` (AC-b11, T-b38) ; traduction commune
  des refus de dates (AC-b2, F11).
- **R8 (MEDIUM)** — numéro fournisseur : égalité normalisée, 6 caractères au moins, jamais de containment ; ce qui
  garantit qu'aucun faux indice ne départage contre l'ordre d'imputation (AC-b1, T-b37, M-b22).
- **R9 (MEDIUM)** — `paidAt` dit à `api-external.md` et au manuel (AC-b13, AC-b16).
- LOW traités : F7 (plan épinglé, T-a12 ; cycle de l'intervalle du classement écrit), F8 (cycles écrits), F9 (T-b25 dit ce
  qu'il n'établit pas), F10 (M-a2 équivalente déclarée ; M-b17 à M-b19 ajoutées), F11 (traduction commune), R-L1 (texte
  commun « rapproché » réécrit, vrai des deux côtés ; la clé propre au fournisseur abandonnée — C-30-1-11 amendé ;
  T-a13), R-L2 (= F10), R-L3 (`…_unlinking` ajoutée, signature existante intacte), R-L4 (compteur du test Vitest), R-L5
  (devise : parité, écrit au § *Dehors*), R-L6 (manuel : motif « rapproché »), R-L7 (modules déclarés au seuil), R-L8
  (attribution de l'annulation : asymétrie nommée), R-L9 (#687 cité).
- Symptômes grepés sur toute la fiche (`pay_in_tx(`, `supplier_invoices_near_amount`, `texte propre`, `issue à
  ouvrir`, `M-b14`, `T-b31`, `C-30-1-27`) ; décomptes recomptés : **24** AC (8 + 16), **24** tâches (T0 à T23), **52**
  tests nommés (T-a1 à a13, T-b1 à b39 ; 7 Vitest) plus l'E2E à deux tests, **29** mutations (6 + 23, dont une
  équivalente déclarée), choix C-30-1-1 à **29** (C-30-1-28, 29 neufs ; 11, 12, 14 amendés). Modules recomptés : 30-1a
  **5**, 30-1b **5** (C-30-1-27, au seuil).

### Validation P2 — 2026-10-15

Opus, deux lentilles en contexte frais (rapports `kesh-gate-logs/301-validate-p2-{F,R}.md`, sur `f53fa0f2`) :
**F 0/1/5/11**, **R 0/0/5/16** — presque tout né de la remédiation P1. Chaque finding vérifié au code (`grep -nF`,
`sed -n`) avant patch ; **aucun réfuté**. Remédiation, arbitrages de l'orchestrateur compris :
- **F1 (HIGH)** — un lot `generated` du compte de la transaction, au montant exact : la ligne ne propose **rien** (ni
  facture, ni règle) et affiche la `notice` (AC-b5) ; T-b41 (deux mensualités), M-b24.
- **F2** — masquage par lot restreint au compte du lot (`in_generated_batch_of_account`) ; un lot d'un autre compte
  retombe dans la ligne « si indice » ; M-b25.
- **F6 = R4** — requête masquante triée lot du compte, puis montant exact, puis imputation ; plafond 200, liste pleine
  traitée comme masquante ; exécutée sans règle active (R-L5) ; T-b42.
- **F3 = R1** — étape 6 : `NotFound` / `IllegalStateTransition` inatteignables (le règlement ne valide aucun projet) →
  `INTERNAL_ERROR`, et non `PROJECT_*`.
- **F4 = R5** — T-b34 au niveau du dépôt sans rejeu, ordre des identifiants dans l'intervalle de queue, chorégraphie
  forcée par sondes ; M-b23 tuable.
- **F5 = R2** — T-a14 (rejeu de `create_batch`, patron `rejeu_interblocage_e2e.rs:2089`) ; `audit_route_registry.rs` (vi)
  quatorze → quinze et (iii bis) ; `api-external.md:788` corrigé (il n'y a pas de « § lots ») ; M-a8.
- **F-L11 = R-L7** — même facture dans deux lots `generated` : **intégré** à la 30-1a (AC-a9) — `create_batch` verrouille
  ses factures par identifiant croissant avant toute lecture ; jugé **mécanique** (un `SELECT … FOR UPDATE` en tête, la
  première lecture du compte passe après) ; T-a15, M-a7.
- **R3** — `payment_reference` est un texte libre à la saisie : prémisse corrigée ; même normalisation et même longueur
  minimale (6) que le numéro ; T-b37 étendu.
- **Module** : l'AC-a9 portait la 30-1a à six modules ; le dialogue *Annuler le rapprochement* passe à la 30-1b
  (C-30-1-33 ; T-a9 → T-b40, M-a6 → M-b26). Recompte : 30-1a **5**, 30-1b **5**, au seuil.
- LOW traités : F-L1 (champ `mp.invoice_id`), F-L2 = R-L6 (table complétée, C4 levé, C7), F-L3 (« six chemins »,
  AC-b14), F-L4 (conséquence de la valeur absolue écrite), F-L5 (M-b5 et son cas), F-L6 = R-L11 (`api-external.md:366`),
  F-L7 (repli `#id`), F-L9 (T-a6 réorienté), F-L10 (L6, issue demandée), R-L1 (`errors.rs:4237`), R-L2 (bras serveur
  inatteignable, justification réécrite), R-L3 (enveloppe `refuse_non_credit_settlement`), R-L4 (conventions de
  `details`), R-L8 (repli de l'écran hors décompte), R-L9 (recomptes ci-dessous et au `sprint-status`), R-L10 (E2E sans
  QR), R-L12 (garantie restreinte à la référence), R-L13 (`CAST … AS DATE`), R-L14 (M-b9 redéfinie), R-L15 (lot
  confirmé dans L1), R-L16 (`SUPPLIER_INVOICE_NOT_FOUND` existe pour les lots). Non retenu : F-L8 (le bras serveur est
  inatteignable, R-L2).
- *Annotation datée, entrée P1* : son décompte « 11, 12, 14 amendés » omettait C-30-1-16 (maintenu) et C-30-1-27
  (recompté) ; l'entrée n'est pas réécrite.
- Décomptes, recomptés par `grep` sur la fiche : **25** AC (9 + 16), **24** tâches (T0 à T23), **56** tests nommés
  (T-a1 à a15 sauf a9 = 14 ; T-b1 à b42 = 42 ; 7 Vitest) plus l'E2E à trois tests, **34** lignes de mutation (M-a1 à a8,
  M-b1 à b26 : 32 à exécuter, M-a2 équivalente, M-a6 retirée), choix C-30-1-1 à **33** (30 à 33 neufs ; 12 et 14
  amendés). Modules : 5 et 5.

### Validation P3 — 2026-10-15

Sonnet, deux lentilles en contexte frais (rapports `kesh-gate-logs/301-validate-p3-{F,R}.md`, sur `da6d6ecc`) :
**F 0/0/5/6**, **R 0/0/6/6**, convergentes, presque tout né de la remédiation P2. Chaque finding vérifié au code avant
patch ; **aucun réfuté**. Prémisses de l'orchestrateur vérifiées : une seule est **inexacte au code** — « reconnaître
une facture payée **par** la confirmation d'un lot » : `payment_batch_items` ne relie aucune ligne à une écriture de
règlement (doc-comment de `last_confirmed_batch_for_invoice`, `payment_batches.rs:825-836`) ; le critère retenu porte sur
le règlement par virement de ce compte, au montant exact, non rapproché, à ±30 jours, **quelle qu'en soit l'origine**
(lot ou fiche), le lot confirmé n'étant qu'un renseignement (C-30-1-34). Remédiation :
- **F5 = R6** — les `notice` ne conseillent plus de confirmer le lot ; seconde `notice`
  `supplier_payment_awaiting_reconciliation` : la ligne ne propose **rien** (ni facture, ni règle) ; troisième fonction
  de requête (AC-b3), T-b43, T-b44, M-b28.
- **F6 = R5** — garde (d) de l'acceptation, même critère (une fonction, deux lecteurs), lectures verrouillantes après le
  compte ; code neuf `RECONCILIATION_SUPPLIER_PAYMENT_HOLD`, deux raisons, libellé ×4 ; T-b45, M-b29.
- **F3 = R1** — liste vide court-circuitée (contrat `200` inchangé) ; borne de 500 identifiants à la route ; cas dans
  T-a15.
- **F1 = R4** — ligne `create_batch` de la table des verrous réécrite (factures en tête, puis compte) ; symptôme
  « compte (lecture ordinaire) → » grepé : plus aucun résidu hors de la note « au sol ».
- **F2** — sonde qui compte (`attendre_des_requetes_en_cours`, `au_moins`) ; T-a15 rouge au sol garanti.
- **F4 = R7** — plan épinglé (`WHERE id IN (…) AND company_id = ?`, parcours de `PRIMARY`, T-a12 étendu) ; T-a16 (lots
  aux factures croisées, sans rejeu) ; M-a9.
- **R2** — bras `INACTIVE_OR_INVALID_ACCOUNTS` ajouté à `failed-proposal-label.ts` ; décompte **36** (35 de la P3 + le
  code neuf de la garde (d)).
- **R3** — FAQ `user-manual.tex:3034` et « Que faire ensuite » `:3040-3045`, *Accepter* `:2156`, dans l'AC-b13.
- LOW : F7 (quatre sites d'`audit_route_registry.rs`), F8 = R10 (coût écrit, `GROUP BY si.id`), F9 (ce qui est
  partagé, dit exactement), F10 (M-b27 déclarée non tuable, raison écrite), F11 (L8), R8 (`a` et `b` au-delà des
  articles existants), R9 (#669 cité à l'AC-a9 et à l'AC-b8), R11 (indice lu sur les sous-scores), R12 (ci-dessous).
- **Angle mort neuf, issue demandée** : L7 — l'acceptation d'une **règle** par l'API ne revérifie pas le masquage
  (préexistant côté client, 29-6).
- **Modules** (R12) : 30-1a = 5, 30-1b = 5 au sens des modules de code nommés ; s'y ajoutent, déclarés hors décompte,
  `kesh-i18n`, `kesh-api/src/errors.rs` (un texte de repli), `lib/shared/utils/settlement-cancel-blocked.ts` (un texte
  de repli) et `kesh-db/src/test_fixtures.rs` (une aide de test). **Signal « au seuil » déclaré au Project Lead**, sans
  nouveau découpage.
- Décomptes, recomptés par `grep` sur la fiche : **25** AC (9 + 16), **24** tâches, **60** tests nommés (T-a1 à a16 sauf
  a9 = 15 ; T-b1 à b45 = 45 ; 7 Vitest) plus l'E2E à trois tests, **38** lignes de mutation (M-a1 à a9, M-b1 à b29 : 35
  à exécuter, M-a2 équivalente, M-a6 retirée, M-b27 non tuable déclarée), choix C-30-1-1 à **37**.

### Amendement de l'orchestrateur après la P3 — 2026-10-15

- **L6** (`paidAt` brut) : déjà intégré à la 30-2c (AC25) — cité, pas d'issue.
- **L7** : intégré à la 30-1b (AC-b17, C-30-1-38) — `accept_one_rule` rejoue le masquage (client : 29-6 ; fournisseur :
  `supplier_payment_hold` et les masquantes) et refuse en `RECONCILIATION_RULE_MASKED` ; T-b46, T-b47, M-b30 ;
  changement de contrat client écrit. Même fichier et même dépôt de requêtes : **30-1b toujours à 5 modules**.
- Décomptes : **26** AC (9 + 17), **24** tâches, **62** tests nommés (15 + 47 ; 7 Vitest) plus l'E2E à trois tests,
  **39** lignes de mutation (36 à exécuter), choix C-30-1-1 à **38** ; libellés : 37 codes.

### Validation P4 — 2026-10-15 — et simplification

Opus, deux lentilles (rapports `kesh-gate-logs/301-validate-p4-{F,R}.md`, sur `3e900779`) : **F 0/0/7/6**, **R
0/1/5/7**. ⚠️ **Signal de recyclage, déclaré au Project Lead par l'orchestrateur** : la classe « double comptage » revient
à chaque passe sous une forme neuve, **née de nos gardes** (P3 : paiement en attente de rapprochement ; P3 : lectures
verrouillantes de (d) ; amendement : AC-b17). **Décision de l'orchestrateur : on ne découpe pas, on SIMPLIFIE**
(C-30-1-39) :
1. **Retrait** de la garde « payée par virement depuis ce compte, non rapprochée, ±30 jours » (C-30-1-34, sa requête, sa
   `notice`, sa raison d'acceptation, T-b43, T-b44, M-b28) : le risque préexiste à la 30-1 (v0.13.4) et relève de #671 ;
   il redevient **L1**, écrit au manuel avec le geste sûr (aucun numéro d'issue à l'écran). Lève R2, F3, R9.
2. **Aucune lecture verrouillante neuve** pour le masquage (proposition, acceptation (d), `accept_one_rule`) : lectures
   ordinaires, tolérance écrite. Les verrous de l'acceptation restent (a), (b), (c) et `pay_in_tx_as` ; la table de
   l'AC-b10 le dit, avec une ligne pour `accept_one_rule` ; aucun cycle neuf (C3 reste vrai). M-b27 réécrite (tuée par
   T-b34, ordre de M-b23). Lève R3, F1, F2, F4.
3. **Une seule fonction source** du critère « lot `generated` du compte au montant exact »
   (`find_open_supplier_invoice_in_account_batch`), lue par la proposition, l'acceptation d'une facture et l'acceptation
   d'une règle ; l'indicateur de la requête masquante est retiré. Lève F6, R11.
4. **AC-b17** : pour un débit, `accept_one_rule` rejoue (a) le lot du compte et (b) `has_strong_invoice_match` →
   `payment_batch` / `invoice_candidate` ; pour un crédit, la condition de la 29-6 → `invoice_slightly_overpaid`.
   L'imputation délibérée passe par *Affecter manuellement*. Lève R1 (HIGH), F5.
5. Comme proposé : R4 = F7 (CHANGELOG de la 30-1a à l'AC-a7, *Modifié* et `RECONCILIATION_RULE_MASKED` à l'AC-b13 et à
   l'AC-b11, T23 jusqu'à M-b30, C-30-1-35 annoté), R5 (chorégraphie C/A/B de T-a16, octroi en file écrit), R6, R7
   (T-a17, test de route), R8 (`api-external.md:554`), R10 (`:2121`), R12 (L10), R13 (longueur brute), F-L1 (L'écriture
   de règlement porte déjà la date du paiement au sol, `supplier_invoices.rs:890` ; seul `paid_at` change — noté),
   F-L2 (motifs de sonde communs), F-L3 (verrou d'enregistrement d'une autre société), F-L4 (L9), F-L5 (coût : trois
   requêtes), F-L6 (`:887` harmonisé), F1 résiduel (TTC de `b` ≠ débit dans T-b34).
- Résidus grepés sur la fiche (`supplier_payment_hold`, `awaiting`, `±30`, `HeldByAwaiting`, « verrouillante » de (d),
  T-b43, T-b44, deux `notice`) : n'en restent que les mentions **datées** des entrées passées et les lignes barrées.
- Décomptes, recomptés par `grep` sur la fiche : **26** AC (9 + 17), **24** tâches, **61** tests nommés (T-a1 à a17 sauf
  a9 = 16 ; T-b1 à b47 sauf b43, b44 = 45 ; 7 Vitest) plus l'E2E à trois tests, **39** lignes de mutation (36 à exécuter :
  sans M-a2 équivalente, M-a6 et M-b28 retirées), choix C-30-1-1 à **39** ; libellés : **37** codes (31 + cinq
  littéraux neufs + `INACTIVE_OR_INVALID_ACCOUNTS`). Modules : 30-1a **5**, 30-1b **5** (au seuil, quatre fichiers hors
  décompte déclarés).

### Validation P5 — 2026-10-15

Sonnet, deux lentilles (rapports `kesh-gate-logs/301-validate-p5-{F,R}.md`, sur `300e1bb8`) : **F 0/1/4/4**, **R
0/0/2/6**. La simplification tient : aucun cycle neuf, aucun défaut rouvert. Chaque finding vérifié au code ; **aucun
réfuté**. Remédiation, arbitrages de l'orchestrateur :
- **F1 (HIGH) = R1-6** — la 30-1b proposait la mensualité voisine ouverte pour le débit d'une facture déjà payée.
  (a) Garde précise, **lecture ordinaire** : la référence de la transaction (égalité normalisée, ≥ 6 caractères) désigne
  une facture fournisseur **payée** → la ligne ne propose rien (`notice` `reference_of_paid_supplier_invoice`) ; la même
  fonction (`find_paid_supplier_invoice_by_reference`) est rejouée à l'acceptation d'une facture (étape 3 (e), raison
  `reference_of_paid_invoice`) et d'une règle (raison `paid_invoice_reference`) ; libellés ×4 ; T-b48, T-b49, T-b50,
  T-b47 étendu ; M-b31, M-b32, M-b34. (b) La forme **sans référence** est écrite à L1 (manuel, `api-external.md`,
  CHANGELOG) avec le geste sûr complet (F10 : en attente, clôture du compte gênée jusqu'à #671 ; exercice clos : aucun
  geste) — acceptée par l'orchestrateur, portée au Project Lead. T-b49 asserte la forme sans référence telle quelle.
- **F2, F3, F7, R1-1** — résidus purgés : `:837` (« la ligne qui dit… »), `:859` (deux codes de `notice`, nommés) ;
  registre : C-30-1-32, C-30-1-35, C-30-1-37, C-30-1-38 annotés et datés.
- **F4 = R1-3** — `excluding` retiré (aucun autre usage) ; M-b29 réduite ; l'effet de bord (vue périmée) écrit.
- **F5 / R1-7** — contacts des masquantes chargés dans le lot, par tranches de 500 ; `accept_one_rule` note avec la même
  carte ; T-b46 second décor (indice par le nom) ; M-b33.
- **R1-2** — `user-manual.tex:3089` et `:2244` à l'AC-b13.
- **R1-4** (AC-b8 : le seul cycle par l'intervalle est C1/C7, rejoué), **R1-5** (changement de contrat des deux côtés,
  renvoi à `POST /reconciliation/manual`), **R1-8** (asymétrie crédit/débit justifiée), **F6** (inventaire clos des
  formes non rejouées : quatre refus seulement), **F8** (tests de non-régression nommés, sans mutation dédiée).
- ⚠️ **Portée de la remédiation, dite à l'orchestrateur** : la garde (a) ajoute du **comportement** au-delà de quelques
  lignes — une requête neuve (normalisation SQL par `REGEXP_REPLACE`), une `notice` neuve, deux raisons de refus, trois
  points d'appel, des libellés en quatre locales et un passage de manuel. Une passe **complète** est recommandée plutôt
  que la passe ciblée demandée ; le prompt ciblé est écrit quand même (`30-1-validate-prompt-p6-ciblee.md`).
- Décomptes, recomptés par `grep` sur la fiche : **26** AC (9 + 17), **24** tâches, **64** tests nommés (T-a1 à a17 sauf
  a9 = 16 ; T-b1 à b50 sauf b43, b44 = 48 ; 7 Vitest) plus l'E2E à trois tests, **43** lignes de mutation (40 à
  exécuter : sans M-a2 équivalente, M-a6 et M-b28 retirées), choix C-30-1-1 à **41** ; libellés : **37** codes (les
  raisons neuves n'ajoutent aucun code). Modules : 30-1a **5**, 30-1b **5** (la garde vit dans `repositories/reconciliation`
  et `routes/reconciliation`).
