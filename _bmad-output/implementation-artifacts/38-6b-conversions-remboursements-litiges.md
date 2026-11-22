# Story 38-6b : Conversions, retraits, remboursements, litiges ; file « à examiner »

Status: deferred

*(2026-11-20 — reportée en phase 2 par la décision du responsable de l'epic (phase 1 KISS, § « Phase 1 KISS » du fichier d'epic) ; validée le 2026-11-15, gardée telle quelle comme base de la phase 2. Ce qu'en reprend la phase 1 est dit dans les fiches 38-11a à 38-11d.)*

- **Epic** : 38 (`_bmad-output/planning-artifacts/epic-38-paypal.md`, § *38-6b*, E7, E16, C-38-8, C-38-9) ; décisions du responsable de l'epic
  B4, B7, B20 (`epics-38-40-cadrage.md`), qui priment.
- **Dépend de** : **38-6a** (chemin prestataire de `get_proposals`, variante `provider`, constructeur `build_provider_entry`,
  codes de C-38-8), **38-6c1** (rattachement en devise, `link_entry_in_tx` en devise, diagnostic du lot, C-38-14),
  **38-6c2** (groupe d'écritures B28, paires, `pairCount` ; D-P2-3), **38-3a** (colonnes
  `movement_kind`, `related_provider_tx_id`, `review_reason`, `provider_tx_id`, C-38-4), **38-2** (connexion d'un compte,
  compte de frais, C-38-3), **37-3b** (conversion par le compte de transit, `completeDifference`, AC-2 bis ; frais au
  débit d'un compte de charge de la devise dans l'écriture 2, **A33**, qu'exige B27).
- **Prépare** : 38-9b (file « à examiner », aperçu d'une conversion, annulation d'une paire à l'écran).
- **Migration** : aucune (C-38-0). « Examiné » réutilise `bank_transactions.auto_match_rejected_at` (C-38-6b-6).

## Story

En tant que **Comptable** d'une société qui tient ses soldes PayPal dans Kesh,
je veux que **les conversions entre soldes, les retraits vers la banque, les remboursements se proposent justes**, et
que **les litiges et mouvements inconnus attendent dans une file « à examiner »**,
afin que **tout mouvement qui change un solde finisse comptabilisé, sans qu'aucun ne le soit sans moi**.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-6b-1` | Une paire de conversion se propose comme une conversion de 37-3b, `completeDifference: true`. | AC-2, AC-3 |
| `FR-38-6b-2` | Un retrait vers la banque ou un dépôt depuis elle se rattache des deux côtés comme un virement entre ses comptes (34-11b). | AC-4 |
| `FR-38-6b-3` | Un remboursement se propose comme l'inverse de l'écriture d'origine, aux montants rapportés. | AC-5 |
| `FR-38-6b-4` | Litige, rétrofacturation, mouvement inconnu ou origine absente vont à la file « à examiner », sans proposition ; « examiné » se marque (`review-done`). | AC-6, AC-7 |
| `FR-38-6b-5` *(ajoutée)* | La file se lit par l'API (`GET /reconciliation/review`), avec le motif de chaque transaction. | AC-6 |
| `NFR-38-6b-1` | Les deux moitiés d'une conversion se réconcilient et s'annulent ensemble. | AC-3, AC-8 |
| `NFR-38-6b-2` | Aucune écriture d'office ; *Pattern batch*. | AC-1, AC-3, AC-4, AC-5 |
| `NFR-38-6b-3` *(ajoutée)* | Une seule définition de la paire d'une conversion, de l'origine d'un remboursement et de l'appartenance à la file, partagée par proposition, acceptation, file et annulation. | AC-1, AC-6 |
| `NFR-38-6b-4` *(ajoutée)* | La file se calcule à la lecture : rien n'est écrit pour y entrer ou en sortir, hors « examiné ». | AC-6 |

## Ce que le code dit (prémisses vérifiées le 2026-11-15)

- **Annuler un rapprochement** : `post_cancel_reconciliation` (`crates/kesh-api/src/routes/reconciliation.rs:5891`) prend le
  verrou nommé du **seul** compte de la transaction (`cancel_reconciliation_once`, `:5917-5997`, `with_account_lock`) et
  appelle `reconciliation_cancel::cancel_in_tx` (`crates/kesh-db/src/repositories/reconciliation_cancel.rs:564`) : une
  écriture qu'aucune pièce ne possède (`ReconciliationKind::Entry`, `:77-92`) est **contre-passée** (`:28-37`) ; le
  retrait d'un seul lien quand l'autre côté d'un virement pointe la même écriture (34-11b, `unlink_only`, `:616-647`) ; trace
  `reconciliation.cancelled` (`:775-782`). La vue (`get_view`, `:424` ; `ReconciliationView`, `:384-402`) dit ce que fera
  l'annulation. Rien ne lie deux transactions entre elles : annuler une moitié d'une conversion contre-passerait **son**
  écriture seule, et le compte de transit ne reviendrait pas à 0.
- **Le verrou nommé** est par compte (`crates/kesh-reconciliation/src/mutex.rs:1-5`, `GET_LOCK` à délai) ; il est de
  session et rendu au retour au pool (`:34-41`).
- **Le virement entre ses comptes** (34-11b) : une écriture admet une transaction **par compte du grand livre**
  (`reconciliation_link.rs`, `linkable_side` `:739`, `transfer_other_side` `:773-795`) ; le rattachement
  `link_entry_in_tx(tx, LinkEntryRequest)` prend une transaction ouverte (`:1156-1159`) et rend `Result<Result<EntryLinked,
  LinkRefusal>, DbError>` ; la route `post_link_entry` (`reconciliation.rs:6656`) l'appelle sous le verrou du compte. Le
  rattachement n'admet aujourd'hui que le CHF (`transaction_can_be_linked_to_entry`, `:535-537`) : 38-6c1 l'étend en
  devise (C-38-14).
- **Rejeter** pose `auto_match_rejected_at = NOW(3)` sur une transaction `pending` non rapprochée (`reject_batch`,
  `reconciliation.rs:4668-4700`) ; la transaction quitte alors les propositions (`find_pending_transactions_for_account`,
  `kesh-db/src/repositories/reconciliation.rs:973-978`) ; l'annulation du rejet existe (`post_unreject_transaction`,
  `reconciliation.rs:6086`, route `…/unreject`, `crates/kesh-api/src/lib.rs:909-910`), refus
  `RECONCILIATION_TRANSACTION_NOT_REJECTED` (`crates/kesh-api/src/errors.rs:1075`). `post_reject` est
  `Traced, SansEcritureAuJournal` (`crates/kesh-api/tests/audit_route_registry.rs:550`).
- **Audit** : `reconciliation.rejected`, `reconciliation.unrejected` existent (`crates/kesh-api/src/audit_labels.rs:189`,
  `:191`) ; aucune action « examinée ».
- **La conversion** (37-3b AC-2 bis, *non livrée*, contrat) : `POST /journal-entries/conversions { entries: [e1, e2],
  completeDifference }`, deux écritures dans une transaction, l'écriture « à compléter » est celle où le compte de
  transit est au **crédit net** ; **forme exigée** pour la compléter : **un seul** compte en devise, au débit net
  (« deux comptes en devise distincts » → `ENTRY_UNBALANCED`, ou 400 si une ligne de différence est portée).
- **Le chemin prestataire** de `get_proposals`, la variante `provider` et le constructeur pur sont ceux de 38-6a
  (C-38-6a-1 à C-38-6a-11) ; 38-6a laisse sans candidat les genres `conversion`, `refund`, `withdrawal`, `deposit`,
  `dispute`, `other` (C-38-6a-11).

## Écarts à l'epic

1. **Citation du virement** : E16 renvoie à `reconciliation.rs:6373`, qui est `post_link_transaction` (rattachement à un
   **règlement**, 29-8) ; le virement entre ses comptes passe par `post_link_entry` (`:6656`) et `link_entry_in_tx`
   (`reconciliation_link.rs:1156`). C'est celui-ci que la fiche emploie.
2. **Frais d'une conversion du côté crédité — tranché par B27 et A33** (B4 : « coût au compte de frais quand le
   prestataire le donne à part »). Un frais donné à part sur la moitié **créditée** va au **compte de frais du
   prestataire et de la devise** (B3 ; `payment_provider_accounts.fee_account_id` du compte de la moitié créditée,
   gardé par 38-2 : compte de charge, même devise — raisons `not_expense`, `currency_mismatch`), **jamais** dans la
   différence de change. L'écriture 2 de 37-3b l'admet depuis A33 (37-3b AC-2 bis, FR-37-3b-8) : une ligne **au débit**
   d'un compte de charge de la même devise n'entre pas dans la règle « un seul compte en devise » (hors comptes de
   charge), et la ligne de différence se calcule **après** elle. Les frais de la moitié **débitée** vont, comme avant,
   au compte de frais de son solde (écriture 1, jamais complétée). *(La limite écrite à la rédaction — frais de la
   moitié créditée laissés dans la différence, Q-6b-1 — est retirée : elle aurait mis un frais dans un gain ou une
   perte de change, et le sous-compte de frais ne se rapprocherait plus du relevé du prestataire, B3/B27.)*
3. **« Origine absente ou non réconciliée → file »** (E16) : tenu tel quel, mais **calculé à la lecture** (NFR-38-6b-4) ;
   la valeur `refund_origin_missing` que le passage (38-3b, classement de 38-8) peut poser dans `review_reason` est une indication, non l'autorité
   (C-38-6b-4). La fiche 38-8 (AC-4, AC-5) écrit que `refund_origin_missing` « est posé par 38-6b » : ici il n'est
   **posé nulle part**, il se **calcule** (rien n'est écrit pour entrer dans la file) ; tranché par **O11** (calculé, jamais écrit) :
   38-8 est alignée depuis la P1 (`38-8-codes-evenement-paypal.md:102`, `:140`).

## Choix

- **C-38-6b-1 — Une seule résolution (NFR-38-6b-3).** Trois fonctions de lecture dans `kesh-db` (dépôt
  `bank_transactions` ou module neuf `provider_movements`), appelées par la proposition, l'acceptation, la file et
  l'annulation, jamais recopiées :
  - `conversion_pair_of(conn, company_id, t) -> PairLookup { Found(BankTransaction) | Missing | Ambiguous }` : la
    transaction `u ≠ t`, `movement_kind = conversion`, d'un compte bancaire de **la même connexion**
    (`payment_provider_accounts.connection_id`), telle que `u.provider_tx_id = t.related_provider_tx_id` **ou**
    `u.related_provider_tx_id = t.provider_tx_id` ; plus d'une → `Ambiguous` ;
  - `refund_origin_of(conn, company_id, t) -> OriginLookup { Found(BankTransaction) | Missing }` : la transaction de la même
    connexion dont `provider_tx_id = t.related_provider_tx_id`, de genre `sale` ou `purchase` ;
  - `review_reason_of(…)` (pure, sur les deux précédentes **et sur le fait « correspond à une facture »** rendu par
    `invoice_match_for_provider_tx` de 38-6a, C-38-6a-7, jamais recopié ; D-P2-5) : le motif de file (C-38-6b-4).
- **C-38-6b-2 — Conversion : deux écritures par 37-3b, jamais une copie.** La moitié **débitée** (montant < 0) donne
  l'écriture 1 (sa date, son cours), la moitié **créditée** l'écriture 2 ; elles se passent par la fonction de service de
  `POST /journal-entries/conversions` **appelée dans la transaction du lot** (si 37-3b ne l'a pas extraite en
  `…_in_tx`, cette story l'extrait, sans recopie), `completeDifference: true` (A31). Écriture 1 : crédit compte
  prestataire X1 `|N₁|` (devise de X1), débit compte de frais X1 `|F₁|` si `F₁ ≠ 0`, débit compte de transit
  `round(|N₁| × c₁, 2) − round(|F₁| × c₁, 2)` (CHF₁, équilibre exact). Écriture 2 : débit compte prestataire X2 `|N₂|`,
  débit compte de frais X2 `|F₂|` si `F₂ ≠ 0` (compte de charge en X2, montant en devise `|F₂|`, CHF
  `round(|F₂| × c₂, 2)` ; ligne de frais admise par A33), crédit transit CHF₁, ligne de différence ajoutée par le serveur
  **après** les frais (comptes de X2, `fx_accounts_for` ; Σ débit, frais compris, contre Σ crédit — A33) : la différence
  ne contient que l'écart de change, jamais `F₂` (Écart 2, B27). Le constructeur pur ne pose pas la différence (37-3b la
  calcule) ; il rend la différence **attendue**, frais déduits, pour l'aperçu. Compte de frais X2 absent ou invalide au
  moment de l'acceptation → `FEE_ACCOUNT_INVALID` (raison de 38-2/38-6a, `missing` comprise) en `failed[]`, rien de
  créé (comme pour `F₁`). Cours : cours retenu du mois de chaque date (C-38-6a-10, AC-6 de 38-6a).
- **C-38-6b-2 bis — Conversion dont un côté est en CHF (ajoutée à la passe P1).** B1 donne trois soldes par compte
  (CHF, EUR, USD) : la conversion CHF ↔ devise est la plus courante, et **37-3b AC-2 bis ne la couvre pas** (l'assistant
  et `POST /journal-entries/conversions` valent entre **deux devises étrangères** — « sinon une écriture ordinaire
  suffit » —, et `completeDifference` exige un compte en devise au débit net). 38-6c1 (AC-2) rattache déjà des deux côtés
  l'écriture « débit 1030 PayPal CHF 100 / crédit 1031 PayPal EUR 92 EUR (CHF 100) ». Quand l'une des deux moitiés est
  en CHF (compte du grand livre sans devise, 37-1), la paire se passe donc en **une seule écriture**, sans compte de
  transit ni appel à la fonction de 37-3b : débit du compte qui reçoit, crédit du compte qui paie, **une seule devise
  étrangère** (celle de l'autre moitié, 37-3a), frais de **chaque** moitié au compte de frais de **son** solde (B27 : la
  moitié créditée comprise ; CHF et la devise étrangère seulement, l'écriture garde une seule devise étrangère), et une
  **ligne de différence** (Σ crédits − Σ débits en CHF) aux comptes de différence de change du compte en devise
  (`fx_accounts_for`, S2 ; gain au crédit, perte au débit), calculée par le constructeur, montrée à l'aperçu (D6),
  jamais silencieuse. Cours : cours retenu du mois de la date de la moitié **en devise** (C-38-6a-10). Date de
  l'écriture : celle de la moitié débitée. Les **deux** transactions sont réconciliées sur cette écriture (virement
  entre ses comptes, 34-11b, 38-6c1 AC-2 : `matched_entry_id` des deux), `pairedJournalEntryId` = l'écriture elle-même.
  Seules les paires **sans CHF** (deux devises étrangères) passent par les deux écritures de C-38-6b-2. Annulation :
  celle de 34-11b, inchangée (le lien de la moitié annulée est retiré, la dernière contre-passe l'écriture) — la route
  à deux verrous (AC-8) ne vise que les paires à deux écritures. La date unique ne heurte aucune règle : le
  rattachement ne regarde aucune date (conditions 1 à 6 de `reconciliation_link.rs` ; `docs/api-external.md:704`,
  « aucun ne regarde de date »), et l'acceptation pose elle-même `matched_entry_id` des deux transactions sur
  l'écriture qu'elle crée *(réserve de la rédaction levée à la passe P2)*.
- **C-38-6b-3 — Retrait et dépôt : virement entre ses comptes.** Deux voies, toutes deux par la variante `provider` :
  (a) **rattacher** : si l'écriture du virement existe déjà (le côté banque a été passé — 34-4 : `matchedElsewhereCount
  = entryCount = 1`, l'écriture candidate étant celle-là, **et aucune paire** : `pairCount = 0`, 38-6c2), la proposition
  porte `linkJournalEntryId` et l'acceptation appelle `link_entry_in_tx` (38-6c1, en devise) — aucune écriture ; un
  refus du rattachement rend en `failed[]` le diagnostic du lot de 38-6c1 (`diagnose_entry_link` : `ENTRY_NOT_LINKABLE`,
  `ENTRY_ALREADY_LINKED`, `AMOUNT_MISMATCH`, C-38-14), non le code d'`AppError` du geste unitaire ; (b) **créer** : sinon, la proposition porte
  `transferBankAccountId` (le compte bancaire de l'autre côté, suggéré : **un seul** compte bancaire non archivé, non
  clos, lié à un compte du grand livre, ayant une transaction `pending` de montant `−G` dans la même devise, datée de
  `booking_date` à `booking_date + 5 jours` pour un retrait, de `booking_date − 5 jours` à `booking_date` pour un dépôt ;
  sinon `null`, à choisir) ; l'acceptation crée l'écriture « compte du grand livre de l'autre compte ↔ compte du
  prestataire », frais au compte de frais, et réconcilie la seule transaction du prestataire ; le côté banque se
  rattache ensuite à cette écriture (34-11b, 38-6c1, 38-9b). Devise du compte du grand livre de l'autre compte = devise de
  la transaction, sinon `PROVIDER_TRANSFER_CURRENCY_MISMATCH` (un retrait avec change passe chez PayPal par une
  conversion, puis un retrait dans la devise reçue).
- **C-38-6b-4 — La file « à examiner » se calcule (NFR-38-6b-4).** Une transaction de prestataire `pending`, non
  rejetée (`auto_match_rejected_at IS NULL`), y entre si : `review_reason` est posé (motif `stored`), **ou** son genre est
  `dispute`, `other` ou inconnu (`Other`) (motif `unknown_event` si `review_reason` est nul), **ou** c'est une moitié de
  conversion dont la paire est `Missing` (`conversion_half_missing`) ou `Ambiguous` (`conversion_pair_ambiguous`), **ou**
  un remboursement dont l'origine est `Missing` (`refund_origin_missing`), **non réconciliée** (`refund_origin_not_reconciled`)
  ou **non inversible** (`refund_origin_not_invertible`, C-38-6b-5), **ou** une moitié dont l'autre est à la file
  (`conversion_pair_under_review`, ci-dessous), **ou** une vente ou un achat de compte CHF dont le **brut** correspond
  à une **facture** de Kesh (lecture de 38-6a C-38-6a-7) et qui porte `fee_amount ≠ 0` (`invoice_with_fee`, motif **calculé**, `reasonSource: "computed"`, D-P2-5 :
  la proposition `provider` passerait en produit le paiement d'une facture existante ; le règlement au brut avec frais
  se passe à la main, issue C-38-13 ; test : facture réglée ou annulée → la transaction sort de la file et le candidat
  `provider` de 38-6a revient). Elle en sort d'elle-même quand la cause disparaît
  (moitié téléchargée, origine réconciliée) ; une transaction de la file n'a jamais de candidat. **Moitié de
  conversion et motif stocké** (passe P2) : le classement de 38-8 (AC-5) pose `review_reason = 'other'` sur une moitié
  restée sans lien (aucun candidat ou plusieurs, paire coupée comprise) ; or l'autre moitié peut arriver au passage
  suivant et s'y relier (`u.related_provider_tx_id = t.provider_tx_id`, 38-8). Pour `movement_kind = conversion`, la
  file se décide donc sur la **paire seule** : `'other'` stocké est lu comme une indication tant que la paire est
  `Missing` (motif rendu `conversion_half_missing`, `reasonSource: "stored"`), et **ignoré** dès qu'elle est `Found` —
  sans quoi la première moitié resterait à la file pour toujours ; tout autre motif stocké (`dispute`, valeur inconnue)
  garde la transaction à la file. **Paire et file** : une moitié dont l'**autre** est à la file (motif stocké autre
  que ci-dessus, ou rejetée/examinée) n'a pas de candidat non plus — motif `conversion_pair_under_review`
  (`relatedBankTransactionId` = l'autre) ; jamais une paire acceptée dont une moitié attend un humain. Tests : moitié
  posée avec `'other'` sans lien → file ; l'autre moitié insérée, reliée → les deux sortent, un candidat ; moitié dont
  l'autre porte `dispute` → file, `conversion_pair_under_review`.
- **C-38-6b-5 — Inverse d'une écriture d'origine.** L'origine `o` est réconciliée sur l'écriture `E`, non contre-passée
  (`reverses_entry_id` / `reversed` lus comme le fait la condition 5 du rattachement). Lecture des lignes de `E` :
  compte du prestataire (compte du grand livre du compte de `o`), compte de frais (réglages du compte de `o`), comptes de
  TVA (`company_invoice_settings`), **la ligne d'arrondi** que 38-6a pose en devise (C-38-6a-6 : comptes de différence
  `fx_accounts_for` du compte du prestataire, ligne CHF sans montant en devise ; passe P2 — sans cette exception,
  toute vente en devise dont l'écriture porte un arrondi serait « non inversible » et irait à la file), et **exactement
  une** autre ligne : la contrepartie. `E` est l'écriture **principale** de `o` (`matched_entry_id`) : si `o` est
  rattachée à un groupe (B28, 38-6c2 — vente de 2024 au brut, frais à part), les membres en plus ne sont pas lus,
  la principale (plus grand mouvement) étant la vente ; test : remboursement d'une vente EUR dont l'écriture porte une
  ligne d'arrondi → inversible ; d'une vente rattachée à un groupe → inversible, sur la principale. Traitement de TVA déduit des
  lignes (ligne de TVA due seule : `SaleIncluded` ; préalable seul : `PurchaseIncluded` ; préalable et TVA sur
  acquisitions égales : `Acquisition` ; aucune : sans TVA), **contrôlé** contre `default_vat_at(conn, company_id,
  contrepartie, date de E)` (38-5 AC-5) ; taux = son `rate_percent` (catégorie en vigueur à la date de `E`, 38-5 Écart 1) ;
  `VatKindUnknown`/`VatRateUnavailable` → non inversible. Tout écart (contrepartie absente ou multiple, genre déduit ≠ genre du
  compte, taux introuvable, `E` contre-passée) → non inversible (file). L'écriture du remboursement est construite par
  `build_provider_entry` avec un champ neuf `reversal: true` (le genre de TVA s'applique au sens de l'origine : un
  remboursement de vente débite la TVA due), aux **montants du remboursement** (brut, frais rendus, net) et au cours du
  mois de **sa** date (B20 ; l'écart de cours avec l'origine est réalisé, il passe dans les lignes CHF).
- **C-38-6b-6 — « Examiné » = rejet nommé.** `POST /reconciliation/transactions/{id}/review-done` pose
  `auto_match_rejected_at` comme `reject_batch` (même `UPDATE`, mêmes gardes) et trace `reconciliation.review_done` ; la
  transaction quitte la file et les propositions ; elle **reste `pending`** (rien n'est comptabilisé) et compte toujours
  dans le contrôle de complétude (E20) ; on revient en arrière par `…/unreject` (28-5a, inchangé). Aucune colonne neuve.
- **C-38-6b-7 — Paire et verrous.** L'acceptation d'une moitié, depuis l'un ou l'autre compte, prend le verrou du lot
  (compte du corps) puis, dans la proposition, celui de l'autre compte (`with_account_lock` **imbriqué**,
  `LOCK_TIMEOUT_SECS` = 5, `routes/reconciliation.rs:90`) ; `AccountLocked` → `FailedProposal` `PROVIDER_PAIR_ACCOUNT_LOCKED`
  (`{ bankAccountId }`), rien d'écrit. ⚠️ `with_account_lock` rend `Result<T, ReconciliationError>` et sa fermeture doit
  en rendre un : la proposition est exécutée dans la fermeture et son `FailedProposal` en ressort par la valeur `T`
  (non par l'erreur) ; **`LockReleaseFailed` n'est jamais un `FailedProposal`** — il remonte, le lot entier est refusé et
  la transaction abandonnée, comme `post_accept` le fait pour le verrou du lot (en-tête de `mutex.rs`, « caller
  pattern ») ; un `GET_LOCK` rendu en interblocage (1213) suit le rejeu existant de `post_accept` (`retry_with`). L'annulation lit la paire hors
  verrou, prend les **deux** verrous dans l'ordre croissant des identifiants de compte, relit la paire sous verrou
  (changée → 409 `PROVIDER_PAIR_CHANGED`), puis appelle `cancel_in_tx` **deux fois** dans la même transaction.
- **C-38-6b-8 — Les étapes de 38-6a AC-8 pour les genres neufs (ajoutée à la passe P2).** `accept_one_provider` garde
  l'ordre de 38-6a AC-8 ; pour chaque genre de cette fiche :
  - **(4)** la liste admise s'élargit à `conversion`, `refund`, `withdrawal`, `deposit` ; `dispute`, `other` et un
    genre inconnu restent refusés, et une transaction de la file (C-38-6b-4, motif **calculé** compris) rend
    `PROVIDER_TX_UNDER_REVIEW` (`{ reason }`) ; pour une conversion, l'étape vaut pour **les deux** moitiés ; le motif
    `invoice_with_fee` est déjà rendu par l'étape (4) de 38-6a AC-8 (vente ou achat à frais qui correspond à une
    facture) : la file de cette fiche et l'acceptation lisent la même fonction ;
  - **(5)** sans objet (aucune règle) : `ruleId` présent → `VALIDATION_ERROR` (`reason: rule_not_expected`), comme
    pour un frais seul ;
  - **(6)/(7)** comptes de frais (des **deux** moitiés d'une conversion, B27) et, pour un remboursement, comptes de TVA
    de l'inverse : lus et contrôlés par **un** appel à `lock_designated_accounts_in_tx`, comme 38-6a AC-8 (7) — ils
    sont écrits par `create_in_tx(…, false)` ; pour la paire sans CHF, la fonction de 37-3b prend ses propres verrous
    (`companies`, compte de conversion) **avant** cet appel, dans l'ordre de 37-3b AC-2 bis ;
  - **(8)** « déjà comptabilisée » : vaut pour une écriture **créée** — remboursement, conversion (lue pour **chacune**
    des deux moitiés, `TRANSACTION_ALREADY_BOOKED` avec `details.bankTransactionId` de la moitié en cause), virement
    voie (b) ; **ne vaut pas** pour la voie (a) d'un retrait ou d'un dépôt, qui **rattache** l'écriture même que
    signale `alreadyBooked` (aucune seconde écriture) : `confirmAlreadyBooked` y est ignoré ;
  - **(9)–(12)** inchangées ; la voie (a) ne lit ni exercice ni cours (`link_entry_in_tx` n'en lit pas, 38-6c1 AC-11).

## Critères d'acceptation

1. **AC-1 — Propositions des quatre genres (NFR-38-6b-2, NFR-38-6b-3).** Sur un compte prestataire, `GET
   /api/v1/reconciliation/proposals` (forme de 38-6a) donne, pour une transaction hors de la file (C-38-6b-4) :
   `conversion` → un candidat `provider` portant `pairedBankTransactionId`, `pairedTransaction { bookingDate, amount,
   currency, bankAccountName }` (l'autre moitié, lue par la même requête de page que la paire), `providerLines` (écriture de **cette** moitié)
   et `pairedLines` (écriture de l'autre), différence calculée pour l'aperçu par la même règle que 37-3b (gain au crédit
   si les débits l'emportent, perte au débit ; comptes de X2) ; conversion à moitié **CHF** (C-38-6b-2 bis) → `providerLines` = l'écriture unique,
   `pairedLines: null` ; `withdrawal`/`deposit` → un candidat portant
   `linkJournalEntryId` (voie a) **ou** `transferBankAccountId` (voie b, `null` si aucun ou plusieurs comptes suggérés) et
   ses lignes ; `refund` → un candidat portant `originBankTransactionId` et les lignes de l'inverse. `blockedBy` (38-6a
   C-38-6a-4) pour un cours absent, un compte de frais invalide, `MULTI_CURRENCY_DISABLED`, `FX_ACCOUNT_INVALID` (comptes de
   différence), `PROVIDER_TRANSFER_CURRENCY_MISMATCH`. Lectures par page, nombre de requêtes non proportionnel au nombre
   de transactions (paires, origines et suggestions de compte lues en une requête chacune pour la page) : test du
   compteur — hormis les lectures de factures CHF par transaction de 38-6a (C-38-6a-7, au brut).
2. **AC-2 — Conversion : les deux écritures (FR-38-6b-1).** Test d'intégration HTTP : soldes EUR (X1) et USD (X2) d'une
   même connexion, liés à des comptes du plan EUR et USD, compte de transit et comptes de différence de la société
   (37-1), cours retenus de mars ; moitiés −100.00 EUR (frais −1.00) et +107.84 USD reliées par
   `related_provider_tx_id`. La proposition (sur X1 comme sur X2) montre les deux écritures (C-38-6b-2), chiffres écrits
   dans le test ; acceptée depuis X1, deux écritures créées par la fonction de 37-3b avec `completeDifference: true`, la
   ligne de différence en dernière ligne de l'écriture 2 ; compte de transit à **0** ; **les deux** transactions
   `reconciled`, chacune sur son écriture ; `AcceptedProposal` porte les champs additifs `pairedBankTransactionId` et
   `pairedJournalEntryId`. **Variante frais sur la moitié créditée** (B27, A33, Écart 2) : moitié +107.84 USD avec frais
   −0.50 USD (compte de frais USD 6911, compte de charge en USD) → l'écriture 2 porte la ligne **débit 6911 `USD 0.50`**
   (CHF au cours du mois), créée par la fonction de 37-3b avec `completeDifference: true` ; la ligne de différence vient
   **après** et vaut l'écart de change seul — chiffres écrits dans le test, égaux à la différence attendue de l'aperçu,
   et différents de ceux du même cas sans la ligne de frais (le frais n'est pas dans la différence) ; solde de 6911
   augmenté de `USD 0.50` ; compte de transit à 0. Compte de frais USD non désigné → `FEE_ACCOUNT_INVALID`
   (`reason: missing`) en `failed[]`, aucune écriture. Frais sur les **deux** moitiés → une ligne de frais dans chaque
   écriture, chacune à son compte. **Variante CHF** (C-38-6b-2 bis) : solde CHF (compte 1030) et
   solde EUR, moitiés −100.00 CHF (frais −1.00) et +92.00 EUR → **une** écriture (crédit 1030 100.00 ; débit compte de
   frais CHF 1.00 ; débit compte EUR `EUR 92.00` × cours du mois ; ligne de différence aux comptes de change du compte
   EUR), les deux transactions `reconciled` sur elle, aucun passage par le compte de transit ; même paire avec frais
   −0.40 EUR sur la moitié créditée → ligne de débit au compte de frais EUR (`EUR 0.40`), différence calculée après elle
   (B27) ; paire EUR/USD du test
   ci-dessus inchangée ; annulation d'une moitié de la paire CHF → retrait de son seul lien (34-11b).
3. **AC-3 — Conversion : refus (NFR-38-6b-1, NFR-38-6b-2).** Corps `provider` d'une moitié : `pairedBankTransactionId`
   **exigé** et égal à la paire résolue sous verrou, sinon `PROVIDER_CONVERSION_PAIR_INVALID` (`{ expected, actual }`) ;
   paire `Missing`/`Ambiguous` → `PROVIDER_TX_UNDER_REVIEW` (`{ reason }`) ; autre moitié non `pending` →
   `RECONCILIATION_TRANSACTION_NOT_PENDING` (`{ bankTransactionId: <autre> }`) ; deux moitiés de même devise ou de même
   signe → `PROVIDER_CONVERSION_PAIR_INVALID` ; verrou de l'autre compte non obtenu → `PROVIDER_PAIR_ACCOUNT_LOCKED` ;
   refus de 37-3b (`FX_CONVERSION_TRANSIT_UNBALANCED`, `FX_ACCOUNT_INVALID`, `ENTRY_UNBALANCED` avec `details.entry`,
   `MULTI_CURRENCY_DISABLED`) → leur code en `failed[]`, jamais `DATABASE_ERROR` ; **autre** moitié « déjà
   comptabilisée » (écriture de 2024 passée par l'API) sans `confirmAlreadyBooked` → `TRANSACTION_ALREADY_BOOKED`
   (`details.bankTransactionId` = l'autre moitié, C-38-6b-8) ; compte de frais de l'autre moitié archivé en concurrence
   → `FEE_ACCOUNT_INVALID`, rien d'écrit (verrou de C-38-6b-8). Chaque refus : rien d'écrit, **ni l'une
   ni l'autre** moitié rapprochée (point de sauvegarde de la proposition). Test à deux connexions : deux lots qui
   acceptent la même paire depuis X1 et X2 en même temps → une acceptée, l'autre `RECONCILIATION_TRANSACTION_NOT_PENDING`
   ou `PROVIDER_PAIR_ACCOUNT_LOCKED`, jamais deux paires d'écritures.
4. **AC-4 — Retrait et dépôt (FR-38-6b-2).** Test d'intégration, deux comptes (PayPal CHF lié à 1030, UBS lié à 1020) :
   (a) **voie créer** — retrait PayPal −500.00 CHF (frais 0) daté J, transaction UBS +500.00 datée J + 2 (dans la
   fenêtre de C-38-6b-3) ; une seconde à J + 6 n'est pas suggérée ;
   `transferBankAccountId` suggéré = UBS ; acceptation avec `transferBankAccountId` → écriture `D 1020 / C 1030` 500.00,
   PayPal `reconciled` ; la transaction UBS liste cette écriture parmi ses candidates (`…/entry-candidates`) et s'y
   rattache (34-11b) ; une écriture, deux transactions rapprochées. (b) **voie rattacher** — ordre inverse : l'UBS d'abord
   passée par écriture manuelle `D 1020 / C 1030` (34-3) ; la proposition PayPal porte `linkJournalEntryId` ;
   l'acceptation rattache (`reconciliation.entry_linked` tracée par `link_entry_in_tx`, plus `reconciliation.accepted`
   `type: provider`, `movement_kind: withdrawal`, `linked: true`) sans créer d'écriture. Refus : `transferBankAccountId` et
   `linkJournalEntryId` tous deux ou aucun → `VALIDATION_ERROR` (`reason: transfer_target_required`) ; compte cible
   inexistant, archivé, clos, d'une autre société, sans grand livre, égal au compte du prestataire →
   `PROVIDER_TRANSFER_ACCOUNT_INVALID` (`{ bankAccountId, reason }`) ; devise →
   `PROVIDER_TRANSFER_CURRENCY_MISMATCH` ; refus du rattachement → le code du lot de 38-6c1 (`ENTRY_NOT_LINKABLE`
   `{ journalEntryId, reason }`, `ENTRY_ALREADY_LINKED`, `AMOUNT_MISMATCH` ; C-38-14, libellés de 38-6c1 réutilisés)
   — passe P2 : `RECONCILIATION_ENTRY_NOT_LINKABLE` est le 409 du geste unitaire, non un code de `failed[]` ; voie (a)
   sur une transaction dont `alreadyBooked` est non nul, sans `confirmAlreadyBooked` → acceptée (C-38-6b-8). Frais d'un retrait
   (`F ≠ 0`) : ligne au compte de frais. Dépôt depuis la banque : symétrique, test.
5. **AC-5 — Remboursement (FR-38-6b-3, B7, B20).** Test d'intégration : vente EUR 100.00 (TVA 8.1 %, frais −3.70)
   acceptée par 38-6a ; remboursement partiel −40.00 EUR, frais rendus +1.20 : la proposition porte
   `originBankTransactionId` et les lignes inverses — crédit compte prestataire 38.80, crédit compte de frais 1.20, débit
   contrepartie `40.00 − T`, débit TVA due `round(T × cours du mois du remboursement, 2)`, `T = vat_from_gross(40.00,
   8.1)` (38-5) — chiffres dans le test ; acceptée, une écriture, transaction `reconciled`. Frais non rendus (0) : pas de
   ligne de frais. Remboursement d'un **achat** (montant > 0) avec impôt préalable : inverse symétrique. Acquisitions :
   les deux lignes égales inversées. Corps : `originBankTransactionId` **exigé**, égal à l'origine résolue sous verrou,
   sinon `PROVIDER_REFUND_ORIGIN_INVALID` (`{ expected, actual }`) ; origine devenue non réconciliée, contre-passée ou non
   inversible entre-temps → `PROVIDER_TX_UNDER_REVIEW` (`{ reason }`).
6. **AC-6 — La file « à examiner » (FR-38-6b-4, FR-38-6b-5, NFR-38-6b-4).** `GET /api/v1/reconciliation/review?bankAccountId&limit`
   (C-38-9 ; même groupe de routes, mêmes rôles et mêmes clés d'API que `GET /reconciliation/proposals` ; `bankAccountId`
   facultatif : sans lui, tous les comptes prestataires de la société ; `limit` défaut 100, plafond 500 ;
   `bankAccountId` d'un compte non prestataire, archivé ou d'une autre société → 404 `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`
   (`AppError::BankAccountNotFound`, `errors.rs:2553`, que `get_proposals` rend `:1020-1028` ; `BANK_ACCOUNT_NOT_FOUND`
   est le seul code de `failed[]`, `reconciliation.rs:2626`, passe P1)) rend `{ items: [{ bankTransactionId, bankAccountId, bankAccountName, bookingDate, amount, currency,
   movementKind, eventCode, counterpartyName, invoiceRef, reason, reasonSource: "stored"|"computed",
   relatedBankTransactionId }], hasMore }`, triés par `bookingDate` décroissante puis `id`. Motifs : ceux de C-38-6b-4
   (`dispute`, `refund_origin_missing`, `unknown_event`, `other` — C-38-4 — et `conversion_half_missing`,
   `conversion_pair_ambiguous`, `conversion_pair_under_review` — ajouté à la passe P2 —, `refund_origin_not_reconciled`,
   `refund_origin_not_invertible`, `invoice_with_fee` — ajouté à la passe P3, D-P2-5 ; les factures se lisent par
   `invoice_match_for_provider_tx` de 38-6a (C-38-6a-7), **au brut** et **par transaction** (fonctions de dépôt des
   factures CHF, une à deux lectures par vente ou achat à frais d'un compte CHF), seule exception au compteur de
   requêtes de la route, comme dans `get_proposals`) ; une valeur stockée
   inconnue (`Other`) est rendue telle quelle. Lecture seule, aucun verrou, rien d'écrit. Tests : chaque motif ; sortie
   automatique de la file quand l'autre moitié est insérée ou l'origine réconciliée ; vente à frais d'un compte CHF
   dont le **brut** (100.00, frais −3.20, net 96.80) égale une facture client ouverte de la fenêtre → motif `invoice_with_fee`, `reasonSource: "computed"`,
   aucun candidat dans `GET …/proposals` (38-6a AC-1), `PROVIDER_TX_UNDER_REVIEW` à l'acceptation `provider` ; même
   vente à `fee_amount = 0` → pas dans la file (la facture se règle, 38-6a) ; une transaction de la file n'a
   aucun candidat dans `GET …/proposals` ; une transaction rejetée n'est pas dans la file.
7. **AC-7 — « Examiné » (FR-38-6b-4, C-38-6b-6).** `POST /api/v1/reconciliation/transactions/{id}/review-done` (Comptable+,
   clés `read-write` admises comme `…/unreject` ; corps vide) → `204` ; transaction d'un autre compte ou société → 404
   `NOT_FOUND` ; compte non prestataire → 409 `PROVIDER_ACCOUNT_REQUIRED` ; compte clos → 409 `BANK_ACCOUNT_CLOSED` ;
   rapprochée → 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` (`errors.rs:1069`) ; déjà rejetée ou examinée → 409
   `RECONCILIATION_ALREADY_REJECTED` (variante d'`AppError` neuve, même code que la `FailedProposal` existante, message
   aux quatre locales). Sous le verrou nommé du compte ; `UPDATE` de `reject_batch` (mêmes gardes) ; trace
   `reconciliation.review_done` (`entity: bank_transaction`, `details: { bank_transaction_id, bank_account_id, reason,
   reason_source }`), la clé d'API nommée. Retour par `…/unreject` (inchangé) : la transaction revient dans la file si sa
   cause demeure. Registre : `("post", "reconciliation::post_review_done", Traced, SansEcritureAuJournal)` — **+1 route
   mutante, +1 tracée** (la lecture `GET …/review` n'entre pas à `LIB_ROUTES`, réservé aux routes mutantes) ; phrase et
   décomptes du manuel d'administration qui citent le nombre de routes recomptés.
8. **AC-8 — Annulation d'une paire (NFR-38-6b-1).** `POST /api/v1/reconciliation/transactions/{id}/cancel` d'une moitié
   de conversion dont l'autre moitié est rapprochée sur **une autre** écriture : les **deux** écritures contre-passées,
   les **deux** transactions `pending`, compte de transit revenu à son solde d'avant ; deux traces
   `reconciliation.cancelled`, chacune avec `pairedBankTransactionId` (paramètre neuf de `cancel_in_tx`,
   `Option<i64>`, écrit en camelCase avec les autres détails) ; réponse `CancelReconciliationResponse` +
   `pairedBankTransactionId`, `pairedReversalJournalEntryId` (additifs, `null` hors paire) ; la vue `GET
   …/transactions/{id}` porte `pairedBankTransactionId` et `pairedTransaction { bookingDate, amount, currency,
   bankAccountName }` (`null` hors paire ; `ReconciliationView`, Rust, `reconciliation.types.ts`,
   `api-external.md`). Verrous et relecture : C-38-6b-7 ; un refus de l'une des deux annulations (exercice clos, verrou
   de période, compte clos) annule le tout, rien n'est écrit. Une moitié rapprochée seule (l'autre `pending`) s'annule
   seule, comme aujourd'hui. Test à deux connexions : annulations simultanées depuis les deux moitiés → une réussit,
   l'autre 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` ou 409 `RECONCILIATION_ACCOUNT_LOCKED`, jamais une seule
   écriture contre-passée.
9. **AC-9 — Constructeur et paire, purs (NFR-38-6b-3).** `kesh-reconciliation::provider` : `build_provider_conversion`
   (deux `NewJournalEntry`, sans ligne de différence, plus la différence attendue pour l'aperçu), `build_provider_transfer`,
   et `build_provider_entry` étendu de `reversal` (C-38-6b-5) ; `review_reason_of` pur. Tests unitaires sans base :
   conversion avec et sans frais de chaque côté, équilibre exact de l'écriture 1, ligne de frais de la moitié créditée
   au **débit** du compte de frais de sa devise dans l'écriture 2 et différence attendue calculée **après** elle (B27,
   A33 : à brut `|N₂| + |F₂|` et cours fixes, la différence attendue ne dépend pas de la part des frais, au centime
   d'arrondi près — propriété testée) ; virement avec
   frais ; inverse d'une
   vente, d'un achat, d'acquisitions ; chaque motif de la file.
10. **AC-10 — Textes, i18n, documentation.** Codes neufs (`failed[]`, `blockedBy` ou `AppError`) :
    `PROVIDER_CONVERSION_PAIR_INVALID`, `PROVIDER_PAIR_ACCOUNT_LOCKED`, `PROVIDER_PAIR_CHANGED`,
    `PROVIDER_TRANSFER_ACCOUNT_INVALID`, `PROVIDER_TRANSFER_CURRENCY_MISMATCH`, `PROVIDER_REFUND_ORIGIN_INVALID`,
    `RECONCILIATION_ALREADY_REJECTED` (en `AppError`) — **et `PROVIDER_ACCOUNT_REQUIRED` rendu en `AppError` 409 par
    `review-done`** (code de 38-6a, créé comme `FailedProposal` : sa variante d'`AppError` et son message d'erreur aux
    quatre locales sont à ajouter ici, passe P1) — libellés dans `failed-proposal-label.ts` (clés écrites en toutes
    lettres, quatre locales ; `details.reason` de `PROVIDER_TX_UNDER_REVIEW` lu pour dire le motif), messages d'`AppError`
    aux quatre locales, décompte des codes du doc-comment recompté au `grep` ; motifs de la file : clés
    `reconciliation-review-reason-*` (une par motif, `invoice_with_fee` compris — clé
    `reconciliation-review-reason-invoice-with-fee`, **posée par 38-6a AC-12** (développée avant) et reprise ici sans
    être recréée —, quatre locales) ; action d'audit `reconciliation.review_done` à
    `audit_labels.rs` et sa clé `audit-log-action-reconciliation-review-done` aux quatre locales
    (`tests/audit_label_registry.rs` vert). `docs/api-external.md` : file, « examiné », variante `provider` des genres
    neufs (corps, refus), annulation d'une paire. Manuel utilisateur et écrans : **38-9b** (aucun contrôle visible ici,
    aucune capture). **MCP** : aucun outil (E23 ; la file et les connexions ne sont pas exposées, les écritures passées se
    lisent par `kesh_ecritures`).
11. **AC-11 — Non-régression.** Annulation d'un virement 34-11b, d'un rapprochement ordinaire et d'un règlement :
    tests existants verts sans modification ; `post_reject`/`…/unreject` inchangés ; comptes non prestataires inchangés.

## Tâches

- [ ] T1 — `kesh-db` : `conversion_pair_of`, `refund_origin_of` (une requête par page aussi : variante par lot) ;
      lecture des lignes d'une écriture d'origine pour l'inverse (AC-1, C-38-6b-1, C-38-6b-5).
- [ ] T2 — `kesh-reconciliation::provider` : `build_provider_conversion`, `build_provider_transfer`, `reversal`,
      `review_reason_of` ; tests (AC-9).
- [ ] T3 — `reconciliation_provider.rs` : candidats des quatre genres, suggestion du compte de l'autre côté, file
      (AC-1, AC-6) ; route `GET /reconciliation/review` (`lib.rs`, même groupe que `…/proposals`).
- [ ] T4 — Acceptation : genres neufs dans `accept_one_provider` (verrou de l'autre compte, fonction de service de
      37-3b dans la transaction — extraction si besoin —, `link_entry_in_tx`), champs du corps
      (`pairedBankTransactionId`, `transferBankAccountId`, `linkJournalEntryId`, `originBankTransactionId`) et leur
      pré-validation de surface dans `post_accept` (AC-2 à AC-5).
- [ ] T5 — `review-done` : route, handler, trace, `AppError::ReconciliationAlreadyRejected`, registre (AC-7).
- [ ] T6 — Annulation d'une paire : route (deux verrous ordonnés), `cancel_in_tx` + `paired_with`, vue, réponse
      (AC-8).
- [ ] T7 — Tests d'intégration et de concurrence (AC-2 à AC-8, AC-11) ; i18n, libellés, `api-external.md` (AC-10).
- [ ] T8 — Gate complet (backend, frontend ; E2E de la suite existante au dernier commit de code — 38-9b porte l'E2E
      de ces écrans). Gate complet **même en cours de boucle** : `cancel_in_tx` et des dépôts sont touchés (règle
      `kesh-db` de `CLAUDE.md`).

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release.)*

- **Ajouté** : les propositions pour les conversions de devises, les remboursements et les litiges des prestataires de
  paiement (PayPal), avec annulation d'une paire ; rien n'est écrit d'office.

**Compteurs** (`LIB_ROUTES`, décomptes du manuel d'administration qui citent le nombre de routes, codes du doc-comment,
`Cargo.lock`) : recomptés depuis la source après chaque reprise de `main`.

## Dev Notes

- **Pourquoi la file ne s'écrit pas** : la moitié manquante ou l'origine arrive au passage suivant (recouvrement `overlapDays`,
  trois jours par défaut, 38-3b) ; écrire un état « à examiner » demanderait de le défaire à chaque passage. Calculée, la file dit toujours
  vrai au moment de la lecture.
- **Pourquoi « examiné » = rejet** : un mouvement qui change le solde doit finir comptabilisé (E20 : la complétude et
  l'écart comptable le signalent tant qu'il ne l'est pas) ; « examiné » ne comptabilise rien, il range. Le rejet a déjà
  ce sens (« manuellement revue », `reconciliation.rs:13`) et son annulation existe.
- **Litige résolu en faveur du vendeur** (deux mouvements qui s'annulent) : les deux vont à la file ; l'utilisateur les
  marque examinés, ou passe deux écritures manuelles (34-3). Écrit au manuel (38-9b).
- **Verrous d'une paire** : le verrou nommé est à délai (`GET_LOCK`), un croisement de deux lots ne bloque pas
  indéfiniment : l'un rend `PROVIDER_PAIR_ACCOUNT_LOCKED` après `LOCK_TIMEOUT_SECS`. L'annulation, elle, ordonne ses deux
  verrous (identifiant croissant), comme le recommande `mutex.rs`.
- **Cycle de verrous avec 37-3b** : la fonction de conversion prend `companies` puis le compte de transit en partage,
  avant les exercices (37-3b AC-2 bis, *Verrous*). Appelée dans la proposition, sous les deux verrous nommés et **sans**
  avoir pris d'exercice avant elle (l'acceptation d'une conversion ne lit pas l'exercice elle-même : la fonction de
  37-3b le fait), elle garde son ordre. Tout cycle résiduel (bascule de 37-1) est défendu par le rejeu de `post_accept`
  (`retry_with`, 1213 et 1305). Argument à écrire au Change Log.
- **Un invariant écrit à mettre à jour** (passe P2) : l'argument « absence de cycle `t → je` » de `link_entry_in_tx`
  (`kesh-db/src/repositories/reconciliation_link.rs:1127-1135`) dit que les acceptations « n'écrivent aucune
  transaction d'un autre compte ». Les paires de cette fiche en écrivent une (l'autre moitié), sous le verrou nommé de
  **son** compte, pris avant (C-38-6b-7) : l'argument tient, mais sa phrase change, dans le même commit. Les poses
  neuves de `matched_entry_id` (paire, paire à moitié CHF, virement) portent sur une écriture **neuve** : elles
  s'ajoutent au relevé (f) de l'inventaire de 38-6c2 (AC-1 (f) de 38-6c2, déjà écrit par elle), « par construction » (une écriture neuve n'est membre
  d'aucun groupe).
- **Références** : epic E7, E16, E20, C-38-4, C-38-8, C-38-9, C-38-14 ; 37-3b AC-2 bis (forme, ordre des refus) ; 34-11b
  (C-e341-22, C-e341-23) ; v017-3a ; 28-5a (unreject) ; 38-6a (C-38-6a-1 à 11).

## Questions ouvertes

- *Close* — **Q-6b-1** (frais donnés à part sur la moitié créditée d'une conversion) : tranchée par **B27** (le responsable de l'epic,
  2026-11-15) — au compte de frais de la devise, jamais dans la différence —, rendue possible par **A33** (37-3b). Voir
  Écart 2.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — validation P5 ciblée (Sonnet) : 0 finding. Remédiation P4 relue : lecture au brut et par transaction
  (`reconciliation.rs:646-663`, `:813-832`, lues) cohérente avec 38-6a C-38-6a-7, AC-1 et AC-12 ; chiffres du test de l'AC-6
  (100.00, −3.20, 96.80) justes ; clé `…-invoice-with-fee` posée par 38-6a AC-12 (textes aux quatre locales y sont
  écrits) et reprise ici sans être redéfinie. Axes non exercés : le reste de la fiche.
- 2026-11-15 — validation P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW ; remédiés dans la fiche. P4-1
  (MEDIUM) le test de l'AC-6 décidait `invoice_with_fee` sur le « numéro de facture », critère que la fonction de 38-6a
  n'emploie pas (montant et fenêtre) ; et au net, la lecture ne retrouvait jamais la facture (38-6a P4-1) → brut, chiffres
  du test (AC-6, C-38-6b-4). P4-2 (MEDIUM) « lecture des factures par page, jamais par transaction » : irréalisable avec
  la fonction unique de 38-6a, bâtie sur les fonctions de dépôt par transaction (`kesh-db/src/repositories/
  reconciliation.rs:646-663`, `:813-832`, lues) → par transaction, seule exception au compteur (AC-6, AC-1). P4-3
  (LOW) AC-10 disait la clé `…-invoice-with-fee` posée ici et reprise par 38-6a, développée avant → posée par 38-6a
  AC-12, reprise ici. P4-4 (LOW) AC-1 : exception du compteur pour ces lectures. Inter-fiches : 38-9b (la clé de
  `invoice_with_fee` vient de 38-6a, non de 38-6b).
- 2026-11-15 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW ; remédiés dans la fiche sauf inter-fiches
  (38-9b : motif `invoice_with_fee` à l'écran de la file) : P3-1 (MEDIUM) D-P2-5 — la file ne connaissait pas
  `invoice_with_fee` (vente à frais qui correspond à une facture : aucune proposition, motif calculé) → C-38-6b-1
  (`review_reason_of` reçoit le fait « facture correspondante » de 38-6a), C-38-6b-4, C-38-6b-8 (4), AC-6 (motif, test,
  lecture par page), AC-10 (clé de libellé) ; P3-2 (LOW) renvoi « AC-12 de 38-6c » (Dev Notes) → AC-1 (f) de 38-6c2.
  Renvois relus contre 38-6c1/38-6c2 : AC-2 et AC-11 de 38-6c1, `diagnose_entry_link` et codes `ENTRY_NOT_LINKABLE`,
  `ENTRY_ALREADY_LINKED`, `AMOUNT_MISMATCH` (AC-5), `pairCount` (38-6c2 AC-3), relevé (f) (38-6c2 AC-1) : exacts ;
  `reversalJournalEntryIds` (38-6c2 AC-4, groupe) et `pairedReversalJournalEntryId` (AC-8, paire) : deux champs de
  sens distinct, sans collision. Prémisses relues au code : `LOCK_TIMEOUT_SECS` (`routes/reconciliation.rs:90`),
  `reconciliation_link.rs:535-537` et `:1127-1135`, exacts. Taille : cinq modules, inchangée ; défauts de P2 non
  recyclés.
- 2026-11-15 — alignement inter-fiches P2 (Opus) : D-P2-3 — dépendance et renvois à 38-6c répartis entre 38-6c1
  (rattachement en devise, `link_entry_in_tx`, diagnostic du lot, exercice clos) et 38-6c2 (groupe, relevé (f),
  paires) ; voie (a) du retrait/dépôt : « aucune paire » écrit `pairCount = 0`. Numéros d'AC cités : ceux de 38-6c
  avant découpage, à relire contre les deux fiches.
- 2026-11-15 — validation P2 (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 3 LOW ; remédiés dans la fiche sauf les
  inter-fiches P2-3 (38-8) et P2-2/P2-1 (38-6c, nom du décompte des paires) : P2-1 (MEDIUM) les étapes de 38-6a AC-8
  n'étaient pas dites pour les genres neufs — liste admise de l'étape 4, verrou des comptes de frais des **deux**
  moitiés (B27) et de TVA d'un remboursement, « déjà comptabilisée » (le rattachement voie (a) est par définition
  « déjà comptabilisé » : `confirmAlreadyBooked` y aurait bloqué le geste juste) → C-38-6b-8 neuve, AC-3, AC-4, voie (a)
  sans paire B28 ; P2-2 (MEDIUM) l'inverse d'une origine refusait toute écriture portant la ligne d'arrondi de 38-6a
  (C-38-6a-6), et ignorait le groupe de B28 → C-38-6b-5 ; P2-3 (MEDIUM) une moitié de conversion posée `'other'` par
  38-8 (AC-5) restait à la file pour toujours, l'autre moitié arrivée — et une moitié pouvait être proposée quand
  l'autre attend un humain → C-38-6b-4 (`conversion_pair_under_review` neuf, AC-6) ; P2-4 (MEDIUM) refus du
  rattachement voie (a) en `failed[]` sous le code du 409 unitaire, contre les codes du lot de C-38-14 → C-38-6b-3,
  AC-4 ; P2-5 (LOW) `LOCK_TIMEOUT_SECS` est à `routes/reconciliation.rs:90`, non `mutex.rs:70` ; P2-6 (LOW) Écart 3 :
  38-8 alignée depuis la P1 ; réserve de date de C-38-6b-2 bis levée (aucune condition de date au rattachement) ;
  P2-7 (LOW) invariant « aucune transaction d'un autre compte » de `reconciliation_link.rs:1135` et relevé (f) de
  38-6c AC-12 (Dev Notes) ; date du test de l'AC-4 (« la veille + 2 jours »). Prémisses relues au code : exactes hors
  P2-5. Taille : cinq modules, sévérité en baisse (P1 : 1 HIGH ; P2 : 0 HIGH), défauts distincts, non recyclés — pas de
  découpage ; 38-6b1/38-6b2 reste la voie si la P3 stagne.
- 2026-11-15 — B27/B28 (Opus) : **B27** — le frais donné à part sur la moitié créditée d'une conversion va au compte de
  frais du prestataire et de sa devise, ligne au débit de l'écriture 2 admise par A33 (37-3b), différence calculée
  après lui ; la limite « frais dans la différence » (Écart 2, Q-6b-1) est retirée ; C-38-6b-2 (écriture 2, refus
  `FEE_ACCOUNT_INVALID`), C-38-6b-2 bis (frais de chaque moitié à son compte), AC-2 (variantes frais sur la moitié
  créditée, deux écritures et écriture unique), AC-9 (constructeur : ligne de frais, différence attendue indépendante
  de la part des frais). B28 ne touche pas cette fiche (rattachement à plusieurs écritures : 38-6c) ; le rattachement
  des deux moitiés d'une conversion reste celui d'une transaction à **une** écriture. Aucun FR/NFR ajouté.
- 2026-11-15 — alignement inter-fiches P1 (Opus) : AC-1 — le candidat `conversion` porte `pairedTransaction
  { bookingDate, amount, currency, bankAccountName }` ; AC-8 — la vue porte de même `pairedTransaction` à côté de
  `pairedBankTransactionId` (liste inter-fiches, point 38-6b) ; découpage D-P1-1 : dépendance portée sur 38-3a
  (colonnes), `review_reason` posé par le passage de 38-3b, recouvrement `overlapDays` de 38-3b (Dev Notes).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 1 HIGH, 2 MEDIUM, 2 LOW ; remédiés dans la fiche sauf l'inter-fiches
  P1-5 : P1-1 (HIGH) conversion dont un côté est en CHF non couverte (37-3b AC-2 bis ne vaut qu'entre deux
  devises étrangères ; 38-6c AC-2 la rattache déjà en une écriture) → C-38-6b-2 bis, AC-1, AC-2 ; P1-2 (MEDIUM)
  verrou imbriqué de l'autre compte : type de retour de `with_account_lock`, `LockReleaseFailed` (C-38-6b-7) ; P1-3
  (MEDIUM) `PROVIDER_ACCOUNT_REQUIRED` rendu en `AppError` par `review-done` sans variante ni message (AC-10) ; P1-4
  (LOW) 404 de `GET …/review` : le code est `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, non `BANK_ACCOUNT_NOT_FOUND` (AC-6) ;
  P1-5 (LOW, inter-fiches) 38-8 AC-4/AC-5 :
  `refund_origin_missing` « posé par 38-6b » contre O11, table de l'epic (38-6b dépend de 38-6c, totaux FR/NFR, 38-6b
  ajoute un code d'audit et une route). Taille : cinq modules (`kesh-db`, `kesh-reconciliation`, `kesh-api`, `kesh-i18n`,
  frontend), 11 AC dont l'annulation de paire qui touche `cancel_in_tx` : limite — à découper en 38-6b1 (conversions,
  retraits, annulation de paire) / 38-6b2 (remboursements, file, « examiné ») si la validation stagne.
- 2026-11-15 — rédaction (Opus) : 11 AC, 5 FR (4 de l'epic + 1 ajoutée), 4 NFR (2 + 2) ; aucune migration
  (« examiné » réutilise `auto_match_rejected_at`) ; +1 route mutante (`review-done`), +1 lecture (`GET …/review`) ;
  action d'audit neuve `reconciliation.review_done` ; écarts : citation du virement (`:6373` → `post_link_entry`), frais
  de la moitié créditée d'une conversion (Q-6b-1), file calculée ; alignée le même jour sur les fiches sœurs : 38-5
  (`default_vat_at`, traitements), 38-8 (liens mutuels des moitiés, `review_reason = 'other'` d'une paire ambiguë,
  `refund_origin_missing` calculé et non posé — écart signalé).
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : section *Note de version* ajoutée ;
  compteurs « recomptés après chaque reprise de `main` ». Aucun PDF ni CHANGELOG commité dans cette fiche.
