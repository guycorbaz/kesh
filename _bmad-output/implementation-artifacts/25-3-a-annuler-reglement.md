# Story 25.3-a : Annuler un règlement — client et fournisseur, par contre-passation

Status: ready-for-dev

**Issue : [#414]**, qu'elle **ferme** : `closes #414` dans le **titre ET le corps** de la PR (le
dépôt merge en squash — cf. CLAUDE.md § « Commits qui adressent une issue »). Commits intermédiaires
en `refs #414`.

**Mère : `25-3-annuler-reglement-et-rapprochement.md`** (statut `split`), qui **reste la source des
faits** — son tableau de corrections vaut ici. **Socle : `25-3-zero-reverse-in-tx.md`**, mergée
(PR #453, `ac1f2829`). **Sœur : 25-3-b** ([#418], dé-rapprochement), qui **appellera** le geste
écrit ici.

## Story

En tant que comptable,
je veux annuler un règlement enregistré par erreur — sur une facture client comme sur une facture
fournisseur —,
afin de corriger une imputation sans écriture manuelle au journal, qui laisserait le résiduel et
`paid_at` en désaccord avec les livres.

## Pourquoi, en une phrase

⛔ **Aujourd'hui, une écriture de règlement est définitivement incorrigible** : la route de
contre-passation la refuse (`OWNED_BY_SETTLEMENT`, `OWNED_BY_SUPPLIER_INVOICE`), le gel de la 24-4b
interdit de la modifier ou de la supprimer, et la seule sortie — une écriture manuelle — laisse
`invoice_settlements` et `paid_at` dire le contraire du grand livre. *C'est l'écart muet que la
vague 1 existe pour supprimer.* Et le manuel **promet déjà** le geste
(`user-manual.tex:1048` : « annulez d'abord le règlement ») alors qu'il n'existe pas.

## ⛔ Le fait qui structure toute la story — le socle refuse ces écritures

`reverse_in_tx` (`journal_entries.rs:1401`) commence par `reversal_blocker` et **rend
`EntryNotReversable`** pour tout motif autre que `AccountArchived` (`:1427-1436`). Une écriture de
règlement client y ressort en `OWNED_BY_SETTLEMENT`, une écriture de règlement fournisseur en
`OWNED_BY_SUPPLIER_INVOICE`. **Appeler `reverse_in_tx` tel quel depuis l'annulation échoue donc
toujours.**

Et la levée naïve est **fausse**, pour une raison de précédence :

- `reversal_blocker` ne rend que le **premier** motif (`:1299-1346`) ; `OwnedBySettlement` (rang 6)
  précède `MatchedBankTransaction` (rang 7).
- Or un règlement créé par **rapprochement bancaire** (`reconciliation.rs:1416` puis `:1465-1470`)
  porte **les deux** : une ligne `invoice_settlements` **et** un `matched_entry_id`.
- ⛔ **« Ignorer `OWNED_BY_SETTLEMENT` » ferait donc passer en silence un règlement rapproché** : la
  contre-passation serait écrite, le lien bancaire resterait sur l'écriture d'origine, et la
  transaction bancaire se dirait rapprochée d'un paiement annulé. *C'est exactement le mode
  d'échec de #418, reproduit par le correctif de #414.*

⇒ **L'exemption doit être ÉTROITE et laisser la précédence se poursuivre** : elle lève **un** motif,
**pour une pièce nommée**, et les motifs suivants restent évalués. C'est l'AC 1.

## Acceptance Criteria

### Le socle

1. **Une contre-passation « au titre de son propriétaire ».** `journal_entries` expose une variante
   de `reverse_in_tx` qui reçoit l'**autorité** de l'appelant — le motif qu'il a qualité pour lever
   **et** l'identifiant de la pièce :
   - règlement client : `(OwnedBySettlement, invoice_settlements.id)` ;
   - règlement fournisseur : `(OwnedBySupplierInvoice, supplier_invoices.id)`.

   Exigences, toutes testées :
   - **une seule logique** : `reverse_in_tx` (sans autorité) et la variante délèguent à une même
     fonction interne ; ⛔ **aucune quatrième contre-passation** — c'est la raison d'être de la
     25-3-zero, et son doc-comment le dit ;
   - l'exemption ne vaut que si le motif **ET** l'identifiant de pièce rendus correspondent à
     l'autorité déclarée — un règlement client ne lève pas le motif d'un autre règlement ;
   - ⛔ **la précédence se poursuit après le motif levé** : `IsAReversal`, `AlreadyReversed`,
     `MatchedBankTransaction` et les autres restent opposables. Concrètement, le calcul des motifs
     rend leur **liste ordonnée** (ou accepte le motif exempté) ; `reversal_blocker` continue de
     rendre le **premier**, sans aucun changement de comportement pour ses appelants (GET de
     l'écriture, route de contre-passation) ;
   - `AccountArchived` garde son traitement actuel : ignoré au recensement, refusé à l'étape 3 par un
     **400 qui nomme les comptes** (`ReversalAccountsArchived`).

2. **Côté fournisseur, l'autorité ne couvre QUE l'écriture de règlement.** `reversal_blocker`
   attribue `OwnedBySupplierInvoice` à l'écriture d'achat **comme** à celle de règlement (`:1272-1277`,
   `purchase_journal_entry_id = je.id OR settlement_journal_entry_id = je.id`). ⛔ **L'autorité
   « facture fournisseur n° N » ne doit donc JAMAIS permettre de contre-passer l'écriture d'achat** :
   la variante vérifie que l'écriture est bien `settlement_journal_entry_id` de cette facture, et
   refuse sinon. Test dédié, prouvé par mutation.

3. **La date est celle du jour, dans un exercice ouvert — tranché par le socle.** La mère laissait
   la question ouverte (sa AC 3) ; `reverse_in_tx` y répond déjà : `entry_date = today`, exercice
   **ouvert** couvrant le jour, sinon `FiscalYearInvalid` (400). ⚠️ **Ne PAS ajouter de date
   fournie par l'appelant** : ce serait une seconde règle de datation pour la même contre-passation.
   Le verrou de période (24-4c) s'applique par `create_in_tx_inner` : une contre-passation datée
   d'un jour verrouillé rend `PERIOD_LOCKED` (400), sans code neuf.

3-bis. **Un règlement dont l'écriture est dans un exercice CLOS ne s'annule pas** — *position de
   la fiche (Q5), recommandation non encore confirmée par Guy*, par cohérence avec sa règle sur la
   facture fournisseur (« sauf exercice clos »). ⚠️ C'est **plus strict** que `reverse_in_tx`, qui
   accepte une origine en exercice clos (contre-passation datée du jour) : la borne appartient donc
   au **geste d'annulation**, pas au socle, dont le comportement ne change pas. Erreur existante :
   `DbError::FiscalYearClosed` → 400 `FISCAL_YEAR_CLOSED` (`crates/kesh-api/src/errors.rs:2682`,
   le mapping de la variante **`DbError`** — à ne pas confondre avec `AppError::FiscalYearClosed`,
   `:304`, autre type), aucune variante neuve.

   **Comment lire « exercice clos »** — ne pas l'inventer : `fiscal_years.status = 'Closed'`,
   joint par `journal_entries.fiscal_year_id` de l'écriture de **règlement** (non de l'exercice du
   jour). Le patron existe : `delete_in_tx` (`journal_entries.rs:1014-1035`) lit
   `fy.status` dans la même requête que le verrou de l'écriture. ⚠️ Le texte générique du repli
   serveur (« aucune écriture ne peut y être ajoutée ou modifiée ») est **approximatif** ici — la
   contre-passation serait datée d'un exercice ouvert ; ce n'est pas un défaut, parce que l'écran
   masque le bouton avant le clic et affiche le texte **propre** de la famille
   `invoice-settlement-cancel-blocked-*` (AC 13).

   **Ce qui empêche une annulation, et dans quel ordre.** Le calcul qui sert à la fois le refus et
   le `cancellable` de la lecture (AC 11) rend, par précédence **figée et testée** :
   1. `FISCAL_YEAR_CLOSED` — l'exercice de l'écriture de règlement est clos : **en premier**, car
      c'est le seul motif **absolu** ; annoncer d'abord un rapprochement ferait défaire un
      rapprochement pour rien ;
   2. le statut de la facture (client : non `validated`, cf. AC 6 ; fournisseur : non `paid`) ;
   3. `MATCHED_BANK_TRANSACTION` (AC 7) ;
   4. `ACCOUNT_ARCHIVED` — **en dernier**, seul motif que l'utilisateur lève lui-même (même raison
      que dans `reversal_blocker`).

### Le règlement client

4. **Le geste, écrit UNE fois, composable.** Une fonction de dépôt `cancel_settlement_in_tx(tx,
   company_id, invoice_id, settlement_id, user_id)` — sans `BEGIN` ni `COMMIT` —, plus son
   enveloppement transactionnel. Dans cet ordre :
   1. verrou `FOR UPDATE` sur la facture (même ordre de verrous que `settle_invoice`, qui verrouille
      la facture en premier) — `NotFound` si elle n'est pas de la société ;
   2. verrou sur la ligne `invoice_settlements` **scopée** par `company_id` **et** `invoice_id` —
      `NotFound` sinon (un règlement d'une autre facture ou d'une autre société n'existe pas) ;
   3. contre-passation de son `journal_entry_id` **au titre** de `(OwnedBySettlement, settlement_id)`
      (AC 1) ;
   4. **retrait** de la ligne `invoice_settlements` (arbitrage de Guy du 2026-09-24, cf. mère :
      retirée, non marquée annulée) ;
   5. recalcul du résiduel par `invoice_settlements::amount_due`, et **projection** de `paid_at`
      (AC 5) ; `version + 1` et `updated_at` **toujours** — la facture a changé d'état, comme dans
      `settle_invoice` ;
   6. audit (AC 9), dans la même transaction.

   ⛔ **La 25-3-b l'appellera** pour le dé-rapprochement d'un encaissement de facture, **après avoir
   défait le lien bancaire dans la même transaction** — c'est ce qui lève `MatchedBankTransaction`
   sans exemption. D'où la forme `_in_tx` publique, et d'où l'interdiction d'y écrire quoi que ce
   soit qui présuppose l'appelant HTTP.

5. **`paid_at` retombe à `NULL` si, et seulement si, le résiduel redevient positif.** Une facture
   réglée en deux fois dont on annule **un** règlement reste partiellement réglée ; une facture
   partiellement réglée dont on annule l'**unique** règlement redevient « à régler ». Même règle
   que l'encaissement (`invoice_settlements_write.rs:228-241`), dans l'autre sens. ⚠️ Un résiduel
   resté ≤ 0 **laisse `paid_at` intact** — mais c'est une branche **défensive** : l'application ne
   produit pas cet état (le trop-perçu est refusé par l'encaissement **et** par le rapprochement, et
   une facture créditée n'est plus `validated`, cf. AC 6). Son test forge l'état par SQL et **le dit**
   dans son doc-comment ; il ne prétend pas reproduire un parcours utilisateur.

6. **Statut exigé : `validated`**, comme pour l'encaissement ; sinon `IllegalStateTransition`.
   ⚠️ Les factures réglées **avant** la table `invoice_settlements` (`20260827000001`, sans
   rattrapage) portent `paid_at` sans aucune ligne : elles n'ont **aucun règlement à annuler**, et
   la story ne leur en invente pas. Le dire au manuel (AC 15).

   ⛔ **Le cas que cette garde laisse en plan, et il est atteignable** (passe 1, lentille A) : un
   avoir **peut** viser une facture **partiellement** réglée — `create_credit_note` ne refuse que
   `paid_at IS NOT NULL` (`credit_notes.rs:291-304`), et `paid_at` reste `NULL` tant que le solde
   n'est pas atteint. L'avoir bascule la facture en `cancelled` (`:561-564`) sans toucher
   `invoice_settlements` : le règlement antérieur survit sur une facture annulée, et cette garde
   **le refuse**. ⚠️ **Ce n'est pas cette story qui crée l'état, et ce n'est pas elle qui doit le
   réparer** : l'avoir y rend déjà la créance **créditrice** du montant encaissé — défaut
   antérieur, tracé par **[#456]**, dont la correction naturelle est de refuser l'avoir en amont.
   *Position de la fiche, conforme aux arbitrages du 2026-09-24* (le paiement est **à lettrer**,
   cf. § Arbitrages) : garder le refus, le **nommer** à l'écran
   (`ILLEGAL_STATE_TRANSITION` ne dit rien — cf. AC 13), et renvoyer à #456. ⚠️ Et le reste dû
   affiché pour une telle facture est lui-même faux : `amount_due` retranche l'avoir **HT** d'un
   total **TTC** — **[#455]**. Ni l'un ni l'autre n'est dans le périmètre.

7. **Un règlement rapproché est REFUSÉ, avec le bon motif.** Par la précédence de l'AC 1, un
   règlement dont l'écriture est `matched_entry_id` d'une transaction bancaire rend
   `MATCHED_BANK_TRANSACTION` (409), **pas** une contre-passation. Le chemin est le dé-rapprochement
   (25-3-b, #418). ⛔ **C'est le test le plus important de la story**, et il doit être vu **rouge
   sur assertion** sous la mutation « l'exemption court-circuite toute la précédence ».

### Le règlement fournisseur

8. **« Annuler le règlement » ramène la facture à `open`.** Fonction de dépôt sur le modèle de
   `pay` / `pay_in_tx` (wrapper + cœur `_in_tx`), dans cet ordre :
   1. verrou facture `FOR UPDATE`, statut exigé `paid` (sinon `IllegalStateTransition`) ;
   2. contre-passation de `settlement_journal_entry_id` **au titre** de
      `(OwnedBySupplierInvoice, supplier_invoice_id)` (AC 1, AC 2) ;
   3. `UPDATE` → `status = 'open'`, `settlement_type`, `settlement_bank_account_id`,
      `settlement_account_id`, `settlement_journal_entry_id` et `paid_at` **à `NULL`**,
      `version + 1`, sous verrou optimiste `AND version = ? AND status = 'paid'` —
      `OptimisticLockConflict` si 0 ligne ;
   4. audit (AC 9) — qui **conserve** l'identifiant de l'écriture de règlement annulée et celui de
      sa contre-passation : c'est la seule trace du lien, la colonne étant remise à `NULL`.

   ⚠️ **Pourquoi les colonnes `settlement_*` sont vidées et non conservées** : il n'y en a qu'un jeu,
   et un nouveau règlement l'écraserait de toute façon. `chk_supplier_invoices_paid_has_settlement`
   n'impose rien à une facture `open`. Le lien historique survit dans l'audit **et** au grand livre,
   où l'écriture d'origine porte désormais `reversed_by`.

   ⚠️ **Le passage direct `paid → cancelled` n'est PAS dans cette story — il existe, ailleurs.**
   Guy a corrigé son arbitrage Q1 le 2026-09-24 : une facture fournisseur **s'annule dans tous les
   cas, sauf si l'exercice de son écriture d'achat est clos** ; payée, son règlement **reste** au
   grand livre et **redevient à lettrer**. C'est l'objet de la **25-3-c**. Ici, « annuler le
   règlement » ramène la facture à `open` et ne touche pas à l'écriture d'achat ; `cancel` n'est
   pas modifiée par cette story.

9. **Lot de paiement.** Une facture réglée par un lot **confirmé** (`payment_batches::confirm_batch`
   → `pay_in_tx`) reste annulable : le lot confirmé est l'historique d'un ordre transmis à la
   banque, et le cas typique d'annulation est précisément un **rejet bancaire**. Le lot n'est pas
   modifié. **Arbitrage de Guy du 2026-09-24 (Q2) : le lot est laissé tel quel.** Une facture dans un lot `generated` ne peut pas être
   `paid`, le cas ne se pose donc pas.

### Traçabilité, refus, écrans

10. **L'audit nomme chaque geste** : `invoice.settlement_cancelled` et
    `supplier_invoice.settlement_cancelled` — **littéraux au site d'insertion** (donc hors
    `SITES_INDIRECTS`), inscrits à `audit_labels.rs::ACTIONS` (liste **triée**) et libellés dans les
    **quatre** locales (`audit-log-action-invoice-settlement-cancelled`, …). Charge utile : montant,
    date du règlement, `settlementJournalEntryId`, `reversalJournalEntryId`, et côté client
    `amountDueAfter`. ⚠️ `reverse_in_tx` écrit **en plus** son propre `journal_entry.reversed` sur
    l'écriture : **deux lignes d'audit, c'est voulu** — l'une dit le fait comptable, l'autre le geste
    métier. Ne pas « dédoublonner ».

11. **Les routes.**
    - `POST /api/v1/invoices/{id}/settlements/{settlement_id}/cancel` et
      `POST /api/v1/supplier-invoices/{id}/settlement/cancel` — **Comptable+**, dans
      `comptable_routes` (`lib.rs`), à côté de leurs jumelles d'écriture. Au registre
      `audit_route_registry.rs` : `Traced`, et **les totaux RECOMPTÉS depuis la source**, message de
      ventilation compris (état de départ : 106 / 88 tracées / 109 avec les routes de test).
    - `GET /api/v1/invoices/{id}/settlements` — **n'existe pas aujourd'hui** (`list_for_invoice`
      n'a que des appelants de test) : sans elle, l'écran n'a aucun moyen de **désigner** le
      règlement à annuler. Tout rôle (`authenticated_routes`, comme `GET /invoices/{id}`). Chaque
      élément : `id`, `journalEntryId`, `amount`, `settledOn`, `settlementType`, et
      **`cancellable` + `cancelBlockedBy`** — calculés par la **même** fonction que le refus du
      geste d'annulation, **celle de l'AC 3-bis** (sa précédence propre, exercice clos en tête — et
      non `reversal_blocker` du socle, AC 1, qui ignore l'exercice de l'origine), pour que l'écran masque le bouton **avant** le clic (patron de la 24-4a, AC 11).
      ⛔ Un bouton affiché qui échoue au clic est le défaut que la 24-4a a déjà payé.
      **Valeurs atteignables** de `cancelBlockedBy` pour une ligne présente : celles de l'AC 3-bis —
      `FISCAL_YEAR_CLOSED`, le statut de facture, `MATCHED_BANK_TRANSACTION` et `ACCOUNT_ARCHIVED` (avec le numéro du compte en libellé, comme `reversalBlockedLabel`) ;
      `IS_A_REVERSAL` / `ALREADY_REVERSED` ne le sont pas — la ligne disparaît dans la transaction
      même qui contre-passe — mais le type les admet, et l'écran les traite (AC 13).
    - **Côté fournisseur**, même besoin : `SupplierInvoiceResponse` gagne
      `settlementCancellable` / `settlementCancelBlockedBy`, calculés par la même fonction, pour une
      facture `paid`. ⚠️ **Où les calculer** : `SupplierInvoiceResponse::from_parts`
      (`routes/supplier_invoices.rs:86`) est **synchrone** et partagée par quatre handlers (GET,
      création, `pay`, `cancel`) ; le calcul demande une lecture en base. Suivre le patron
      `InvoiceResponse::from_parts(..).with_settlement(settled, due)` (`routes/invoices.rs:640`) :
      un **constructeur additionnel** appelé par le **GET** et par la réponse de l'annulation du
      règlement ; ailleurs les champs sont **absents** (`Option` + `skip_serializing_if`), jamais une
      valeur par défaut qui mentirait (`settlementCancellable: false` sur une facture annulable).
      La liste (`SupplierInvoiceListItem`) n'est pas touchée. Seuls `FISCAL_YEAR_CLOSED` et `ACCOUNT_ARCHIVED` y sont atteignables aujourd'hui — aucun des cinq sites de
      `reconciliation.rs` qui posent `matched_entry_id` ne rapproche une écriture fournisseur
      (vérifié en passe 1) —, et c'est précisément pourquoi le calcul ne doit pas le **supposer**.
    - Réponse des deux `cancel` : la facture relue (`InvoiceResponse` avec `amountSettled` /
      `amountDue` à jour ; `SupplierInvoiceResponse`) et `reversalJournalEntryId`.

12. **Les motifs de refus changent de sens.** `OWNED_BY_SETTLEMENT` dit aujourd'hui « son
    annulation viendra avec la contre-passation des règlements » — un chemin qui va exister.
    `OWNED_BY_SUPPLIER_INVOICE` dit « annulez la facture », ce qui est **faux** pour une écriture
    de règlement (une facture `paid` ne s'annule pas). Nouveaux textes : ils **nomment le chemin
    réel** — la fiche de la facture, bouton « Annuler le règlement ». Sites, **tous** à modifier
    ensemble (grep du **code** et de la **clé**, pas de la phrase) :
    - FTL `journal-entries-reverse-blocked-settlement` / `-supplier-invoice`, **quatre** locales
      (fr-CH `:342-343`, autres `:348-349`) ;
    - repli serveur `crates/kesh-api/src/errors.rs:2466-2478` ;
    - repli Svelte `journal-entries/[id]/+page.svelte:151-165` ;
    - manuel (`user-manual.tex:1769`).

    ⚠️ **Les refus RESTENT** : contre-passer une écriture de règlement **directement** par
    `POST /journal-entries/{id}/reverse` reste faux — c'est l'annulation qui la contre-passe, avec
    le reste. `MATCHED_BANK_TRANSACTION` **ne change pas** ici : c'est la 25-3-b.

13. **Les écrans.**
    - **Fiche facture client** (`invoices/[id]/+page.svelte`) : la liste des règlements (date,
      montant, mode, lien vers l'écriture), avec un bouton « Annuler le règlement » par ligne
      **`cancellable`**, et le **motif** affiché à la place du bouton sinon. Confirmation avant
      l'envoi, qui dit ce qui va se passer (« une écriture inverse datée d'aujourd'hui sera
      passée »). Après succès : totaux, statut et bouton « Enregistrer un règlement » rafraîchis.
    - **Fiche facture fournisseur** (`supplier-invoices/[id]/+page.svelte`) : bouton « Annuler le
      règlement » quand `status === 'paid'` (le bloc `:280-284` n'a aujourd'hui ni bouton ni lien
      vers l'écriture).
    - ⛔ **Chaque refus s'affiche en nommant son motif**, et le repli en dur dit **mot pour mot** ce
      que dit le FTL fr-CH. Le 409 `EntryNotReversable` porte `details.documentId` ; le 400
      `ACCOUNT_ARCHIVED` porte `details.rejected[]` — l'afficher, pas « Transition interdite ».
    - ⛔ **Les motifs de l'annulation ont leurs PROPRES textes, écrits UNE fois.** Le seul mapping
      existant des codes (`journal-entries/[id]/+page.svelte:129-186`) parle de « cette écriture »
      et sert un autre écran ; le recopier violerait la règle DRY, le détourner ferait dire à la
      fiche de facture une phrase fausse (« rapprochée d'une transaction bancaire » est juste,
      « appartient à une facture » ne l'est pas). ⇒ une famille de clés
      `invoice-settlement-cancel-blocked-*` (quatre locales) et **un seul** module partagé
      (`frontend/src/lib/features/…`) qui mappe `cancelBlockedBy` → texte, en `switch` exhaustif
      avec garde `never`, appelé par les **deux** fiches. `MATCHED_BANK_TRANSACTION` y dit le
      chemin : annuler le rapprochement (25-3-b) — tant que la 25-3-b n'est pas livrée, le texte
      dit qu'il faut d'abord annuler le rapprochement, **sans** promettre de bouton.
    - Le refus de l'AC 6 (facture créditée) : le 409 `ILLEGAL_STATE_TRANSITION` n'a qu'un message
      générique à l'écran. La liste ne proposant pas le bouton sur une facture non `validated`
      (le calcul de `cancellable` l'intègre), le cas ne s'atteint que par l'API ; l'écran affiche
      alors la raison **à la place** du bouton, par la même famille de clés.
    - ⚠️ `invoices_echeancier.spec.ts:173-182` affirme l'**absence** d'un bouton d'annulation
      « tant que #414 n'est pas livrée » : son commentaire devient faux. Le réécrire pour qu'il
      vérifie la **présence** du geste, et non le supprimer. **Même affirmation** dans
      `invoices.spec.ts:322-324` (« rien ne le remplace … issue #414 ») — commentaire à corriger,
      l'assertion voisine (`settle-open` absent) reste juste. Inventaire fait par
      `grep -rn "#414" frontend/tests/e2e/` : ces deux fichiers, et eux seuls.

14. **Tests** (⛔ chaque garde **prouvée par mutation**, vue **rouge sur assertion**, et la mutation
    décrite au Dev Agent Record) :
    - socle : exemption étroite (un autre `settlement_id` ne lève rien) ; précédence poursuivie
      (AC 7) ; écriture d'achat fournisseur refusée malgré l'autorité (AC 2) ; `reverse_in_tx` sans
      autorité **inchangé** — les 27 tests de `journal_entry_reversal_e2e` passent **sans retouche** ;
    - client : règlement unique annulé (résiduel = TTC, `paid_at` NULL, ligne retirée, écriture
      inverse avec `reverses_entry_id`) ; règlement **partiel** parmi deux (AC 5) ; branche
      défensive « résiduel ≤ 0 » sur **état forgé et déclaré tel** (AC 5) ; double annulation → la
      seconde rend `NotFound` ; facture **créditée après un règlement partiel**, produite par le
      **vrai** chemin (`settle_invoice` puis `create_credit_note`) → refus nommé (AC 6) ; exercice
      du jour absent ou clos → `FISCAL_YEAR_INVALID` ; **écriture de règlement dans un exercice
      clos** → `FISCAL_YEAR_CLOSED`, alors que l'exercice du jour est ouvert (AC 3-bis — c'est ce
      qui distingue les deux refus) ; **précédence** de l'AC 3-bis, chaque paire de motifs
      cumulés ;
    - fournisseur : `paid → open`, colonnes vidées, re-règlement possible ensuite (le cycle
      complet, pas seulement l'annulation) ; facture `open` → refus ; facture réglée par lot
      confirmé (AC 9) ;
    - ⛔ **rollback** : un échec **après** la contre-passation (p. ex. l'`UPDATE` fournisseur en
      conflit de version) ne laisse **aucune** écriture inverse — avec une lecture **positive**
      dans la transaction avant l'échec (leçon de la 25-3-zero : un test de rollback aux assertions
      toutes négatives ne prouve rien) ;
    - **étanchéité multi-tenant** : société B ne peut ni lister ni annuler un règlement de A
      (`NotFound`), et **rien n'est écrit** — vérifié table par table : `journal_entries`,
      `journal_entry_lines`, `invoice_settlements`, `invoices` (ou `supplier_invoices`),
      `audit_log`. *Énumérer, pas compter* ;
    - API : les deux `POST …/cancel` → 200, **403 pour Consultation**, 404, 409 avec `details` ;
      `GET …/settlements` → **200 pour tout rôle, Consultation compris** (AC 11), 404 hors
      société ;
    - vitest : écran client (liste, bouton masqué avec motif, rafraîchissement) et fournisseur ;
    - Playwright : régler puis annuler, client et fournisseur.

15. **Documentation.** ⚠️ **Le manuel n'a AUCUNE section sur l'enregistrement d'un règlement
    client** (Story 24-3) — seulement deux mentions incidentes (`:338`, `:348`) ; la seule section
    de règlement est « Régler une facture fournisseur » (`:1098-1108`). L'annulation n'a donc nulle
    part où s'accrocher : créer, à côté de « Échéancier des factures » (`:925`), une sous-section
    **« Enregistrer et annuler un règlement »** qui couvre les deux gestes côté client (le premier
    est un **manque antérieur** que cette story comble parce qu'elle en a besoin). Puis manuel
    utilisateur FR (`user-manual.tex`) : le geste, client et fournisseur,
    ce qu'il écrit au grand livre, ce qu'il refuse et pourquoi (règlement rapproché → 25-3-b ;
    facture réglée avant l'Epic 24 → rien à annuler) ; relire `:1048` (dévalidation), `:1098-1108`
    (règlement fournisseur), `:1769` (contre-passation). `api-external.md` : les trois routes.
    `CHANGELOG.md`, section **`[0.12.1] — Non publié`** — ⛔ **aucune ligne d'une section publiée ne
    se réécrit**, et le décompte « 94 actions » de cette section **se recompte**. Régénérer le PDF
    et le **contrôler aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).

16. **Gate complet** — la story touche `kesh-db` (dépôts) : ciblage interdit, même en boucle de
    revue. Base remise à zéro avant chaque gate complet.

## Tasks / Subtasks

- [ ] **T1 — Le socle** (AC 1, 2, 3) : motifs en liste ordonnée, contre-passation « au titre de »,
      `reverse_in_tx` inchangé. Tests de précédence et d'exemption **avant** tout appelant.
- [ ] **T2 — Le règlement client** (AC 3-bis, 4-7) — la précédence d'annulation de l'AC 3-bis, écrite **une** fois et partagée avec T3 : `cancel_settlement_in_tx` + wrapper, `paid_at`.
- [ ] **T3 — Le règlement fournisseur** (AC 3-bis, 8, 9).
- [ ] **T4 — Audit et routes** (AC 10, 11) : codes, libellés ×4, `ACTIONS`, registre de routes
      recompté, `GET /invoices/{id}/settlements`.
- [ ] **T5 — Motifs de refus** (AC 12) : FTL ×4, replis serveur et Svelte, grep de la clé.
- [ ] **T6 — Écrans** (AC 13) et gardes i18n du frontend recomptées.
- [ ] **T7 — Tests et mutations** (AC 14).
- [ ] **T8 — Documentation** (AC 15), gates (AC 16), PR `closes #414`.

## Dev Notes

### Ce que cette story ne fait pas

- **Le dé-rapprochement** : 25-3-b (#418). Un règlement rapproché est **refusé** ici (AC 7).
- **L'avoir** : annuler une *facture* reste le chemin de l'avoir (`OwnedByInvoice`,
  `OwnedByCreditNote`) — inchangé.
- **`supplier_invoices::cancel`** et sa contre-passation écrite à la main (ne pose pas
  `reverses_entry_id`, ne consulte pas `reversal_blocker`) : **défaut antérieur, non corrigé ici**
  — **[#454]**, absorbée par la **25-3-c** (annulation d'une facture fournisseur dans tous les cas). ⚠️ Ne pas l'imiter pour le règlement fournisseur.
- La propagation du résiduel aux rapports agrégés (#416) ; l'imputation d'un écart (#384).
- L'avoir émis sur une facture partiellement réglée (**#456**) et l'avoir HT retranché d'un TTC
  dans `amount_due` (**#455**) — deux défauts antérieurs trouvés par la passe 1, cf. AC 6.

### Ce qu'il faut savoir du code existant

- **Encaissement client** : `invoice_settlements_write.rs::settle_invoice` — verrou facture, compte
  de créance lu **sur l'écriture de vente**, trop-perçu refusé, exercice ouvert de `settled_on`,
  écriture `D contrepartie / C créance`, ligne `invoice_settlements`, `paid_at` si résiduel ≤ 0,
  audit `invoice.paid` / `invoice.partially_settled` (ternaire → `SITES_INDIRECTS`). ⚠️ Il ne
  bumpe `version` **qu'au solde** ; l'annulation le bumpe **toujours** (AC 4) — la mention
  explicite évite qu'une relecture y voie une incohérence.
- **Encaissement par rapprochement** : `reconciliation.rs::accept_one_invoice` (`:1056`), même
  ligne `invoice_settlements` (`:1416`) + `matched_entry_id` (`:1465-1470`). D'où l'AC 7.
- **Règlement fournisseur** : `supplier_invoices.rs::pay_in_tx` (`:542`), un seul règlement, pour
  le TTC, porté par la **ligne de facture** (`settlement_*`, `paid_at`, `status`) — aucune ligne
  `invoice_settlements`. Le projet est posé au niveau document et **recopié sur chaque ligne** à
  l'insertion (`journal_entries.rs:426`, `line.project_id.or(new.project_id)`) : la contre-passation
  par `reverse_in_tx`, qui reprend `project_id` **par ligne**, **préserve donc l'analytique** — vérifié,
  pas supposé.
- **Contre-passation** : `reverse_in_tx` (`journal_entries.rs:1401-1530`) — verrou sur l'origine,
  motifs, lignes par `line_order` inversées, comptes archivés (400 qui nomme), exercice ouvert **du
  jour**, `create_in_tx_inner(..., Some(id))` qui pose `reverses_entry_id`, audit
  `journal_entry.reversed`. **Ne pas en modifier l'ordre** (contrat de la 25-3-zero).
- **Après annulation**, l'écriture d'origine ressort en `ALREADY_REVERSED` (rang 2) et sa
  contre-passation en `IS_A_REVERSAL` (rang 1) : **aucune** des deux ne peut plus être
  contre-passée ni ré-annulée. C'est voulu, et un test le vérifie.
- **Conséquences voulues, à ne pas « corriger »** : une facture client dont tous les règlements sont
  annulés redevient **relançable** (`paid_at IS NULL`) et, faute d'autre motif, **dévalidable**
  (`UnvalidationBlocker::Settled` lit l'existence d'une ligne **ou** `paid_at`) — les deux
  conséquences suivent de l'état, sans code.
- **Correspondance des erreurs** (`crates/kesh-api/src/errors.rs`) : `EntryNotReversable` → 409 avec
  `blocker.code()` et `details{documentId, documentNumber}` ; `ReversalAccountsArchived` → 400
  `ACCOUNT_ARCHIVED` + `details.rejected[]` ; `IllegalStateTransition` → 409 au message
  **générique** (le texte Rust n'est que journalisé) ; `FiscalYearInvalid` → 400 ; `PeriodLocked`
  → 400 ; `OptimisticLockConflict` → 409 ; `NotFound` → 404. Aucune variante neuve n'est
  nécessaire.
- **Frontend** : pas de fichiers de langue propres — `i18nMsg(key, fallback)` lit les FTL du
  serveur. ⚠️ Les gardes vitest comptent les sites : `i18n-keys.test.ts` (`ATTENDU.sitesTotal`,
  `sitesNonResolus`…), `i18n-un-repli-par-cle.test.ts` (`CLES_RELEVEES`),
  `i18n-libelle-en-dur.test.ts` (`CANDIDATES_ATTENDUES`). **Les recompter, ne pas les incrémenter
  de confiance.**

### Pièges nommés d'avance

1. **L'exemption large** (AC 1, AC 7) — le défaut le plus probable, et il est **muet** : tous les
   tests d'annulation passent, seul le règlement rapproché trahit.
2. **L'autorité fournisseur qui couvre l'écriture d'achat** (AC 2) — même motif, même
   identifiant de pièce ; seule la vérification du rôle de l'écriture les distingue.
3. **Le test de rollback aux assertions négatives** (AC 14) — satisfait aussi par « jamais écrit ».
4. **Le motif de refus corrigé à un site sur quatre** (AC 12) — greper la **clé** et le **code**,
   jamais la phrase.
5. **Les totaux des registres incrémentés au lieu d'être recomptés** (AC 10, 11).

### Règle de splitting

Modules : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — **quatre**, sous le seuil de cinq.
⚠️ **Signal à surveiller** : si la validation fait stagner la sévérité, le premier découpage naturel
est **client / fournisseur** (deux appelants du même socle, sans code partagé au-delà de T1).

### Project Structure Notes

- Dépôts : `crates/kesh-db/src/repositories/journal_entries.rs` (socle),
  `invoice_settlements_write.rs` (client — c'est le fichier du geste d'écriture, l'annulation y a
  sa place), `supplier_invoices.rs` (fournisseur). Aucune migration : le schéma porte déjà tout.
- Routes : `crates/kesh-api/src/routes/invoices.rs`, `routes/supplier_invoices.rs`, enregistrement
  dans `crates/kesh-api/src/lib.rs` (`comptable_routes` / `authenticated_routes`).
- Registres : `crates/kesh-api/src/audit_labels.rs`, `crates/kesh-api/tests/audit_route_registry.rs`,
  `crates/kesh-api/tests/audit_label_registry.rs`.
- Frontend : `frontend/src/routes/(app)/invoices/[id]/+page.svelte`,
  `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`,
  `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`, `frontend/src/lib/features/invoices/`,
  `frontend/src/lib/features/supplier-invoices/`.

### Arbitrages rendus par Guy le 2026-09-24

- **Q1 — corrigé par Guy le même jour.** Première réponse : « deux gestes ». Correction : **une
  facture fournisseur s'annule dans tous les cas, sauf si l'exercice est clos** (l'exercice, non le
  verrou de période de la 24-4c) ; si elle est payée, le règlement **reste** et **redevient à
  lettrer**. ⇒ story **25-3-c**, hors de celle-ci. Cette story garde l'annulation du **règlement**
  (`paid → open`), qui reste un geste propre.
- **Factures clients** : *« une fois envoyée, on ne devrait plus la modifier »* — le chemin reste
  l'**avoir**, rien ne change ici.
- **Un paiement doit correspondre à une facture** : un règlement détaché de sa facture n'a pas
  vocation à rester seul, il attend d'être **lettré** à une facture — au besoin créée pour lui
  (*« c'est la manière dont bexio travaille »*). Le lettrage est l'**Epic 15**.
- **Q2 — Lot pain.001 confirmé : annulation du règlement autorisée, lot laissé tel quel.**
- **Q3 — `supplier_invoices::cancel`** : d'abord renvoyée à une issue distincte (**#454**) ; la
  25-3-c, qui réécrit `cancel`, l'absorbe naturellement.

⚠️ **Ne pas contester ces arbitrages en revue** : en contester la mise en œuvre.

**Ce que ces arbitrages disent du cas de l'AC 6** (ancienne Q4) : le règlement antérieur d'une
facture client **créditée** est un paiement **à lettrer**, pas une anomalie à effacer. Cette story
**n'y touche pas** : elle refuse d'annuler ce règlement, en le **nommant** (AC 6, AC 13), et le
chemin est le lettrage à une facture (Epic 15). ⚠️ **#456 change donc de nature** : la correction
qu'elle suggère (refuser l'avoir) contredit ce modèle — à reprendre avec Guy.

**En attente** :

- **Q5** — La borne « exercice clos » vaut-elle aussi pour l'annulation d'un **règlement** ? Guy a
  répondu « continue » sans trancher : la fiche **applique la recommandation** (AC 3-bis) et la
  marque comme telle — **ce n'est pas un arbitrage**, une revue peut la discuter. *Recommandation :
  oui, par cohérence avec la règle de Guy sur la facture fournisseur* — un règlement dont l'écriture
  est dans un exercice clos ne s'annulerait plus.

### References

- [Source: crates/kesh-db/src/repositories/journal_entries.rs:1234-1345] — `reversal_blocker`, précédence.
- [Source: crates/kesh-db/src/repositories/journal_entries.rs:1401-1555] — `reverse_in_tx`, `reverse`.
- [Source: crates/kesh-db/src/errors.rs:57-106] — `ReversalBlocker`, huit variantes.
- [Source: crates/kesh-db/src/repositories/invoice_settlements_write.rs] — `settle_invoice`.
- [Source: crates/kesh-db/src/repositories/invoice_settlements.rs] — `amount_due`, `list_for_invoice`.
- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs:470-898] — garde de lot, `pay`, `pay_in_tx`, `cancel`.
- [Source: crates/kesh-api/src/routes/reconciliation.rs:1416,1465-1470] — règlement rapproché.
- [Source: crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:65-73] — contraintes de statut.
- [Source: docs/manual/fr/user-manual.tex:1048,1098-1108,1769] — ce que le manuel promet.
- [Source: _bmad-output/implementation-artifacts/25-3-annuler-reglement-et-rapprochement.md] — source des faits.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-24 | spec | Fille de la 25-3 (split), sur le socle `reverse_in_tx` mergé (PR #453). ⛔ **Fait établi à la lecture, et il structure la story** : `reverse_in_tx` **refuse lui-même** les écritures de règlement (`OWNED_BY_SETTLEMENT`, `OWNED_BY_SUPPLIER_INVOICE`) — le socle ne suffit pas tel quel. Et la levée naïve est **fausse** : `reversal_blocker` ne rend que le premier motif, et un règlement **rapproché** porte `OWNED_BY_SETTLEMENT` (rang 6) **avant** `MATCHED_BANK_TRANSACTION` (rang 7) — l'ignorer contre-passerait en silence un paiement que la banque dit rapproché. D'où une exemption **étroite** (un motif, une pièce nommée) qui **laisse la précédence se poursuivre** (AC 1, AC 7). Deux autres faits établis : **aucune route ne liste les règlements d'une facture** (`list_for_invoice` n'a que des appelants de test), donc l'écran ne pouvait pas désigner le règlement à annuler — route ajoutée (AC 11) ; et le manuel **promet déjà** le geste (`:1048`). La question de date laissée ouverte par la mère est **tranchée par le socle** : jour, exercice ouvert (AC 3). Trois questions laissées à Guy (Q1-Q3), chacune avec une recommandation. |
| 2026-09-24 | validate P3 | **Deux lentilles Opus** en contexte frais, prompt versionné `25-3-a-validate-prompt-p3.md`, axes déclarés (non exercés : contenu des locales non françaises, recompte des gardes i18n, interblocage avec la 25-3-b, concurrence avec `confirm_batch`). Lentille A (régressions) : **1 HIGH, 5 MEDIUM, 4 LOW** ; lentille B (à froid) : **8 MEDIUM, 8 LOW** — recoupés, une dizaine de défauts distincts. ⛔ **HIGH** : la borne « exercice clos » de l'AC 3-bis — **ma** recommandation Q5, appliquée faute de réponse — **contredit l'issue #414** (« une date dans un exercice ouvert — jamais à la date d'origine si son exercice est clos ») **et le manuel** (`:1771-1774`, « elle reste contre-passable […] aucune raison de rouvrir l'exercice »), et sa justification est **fausse** (« seul motif absolu » : `fiscal_years::reopen` existe) — vérifié. MEDIUM, tous nés de l'AC 3-bis ou du côté fournisseur : ordre des motifs contraire à son propre critère (le statut « créditée » est définitif, l'exercice clos ne l'est pas) ; aucun **code** ni **type** pour le motif de statut, et un pré-contrôle `ACCOUNT_ARCHIVED` qui avalerait le 400 qui **nomme** les comptes ; aucun champ pour le **numéro** du compte archivé ; bouton fournisseur conditionné au statut au lieu de `settlementCancellable`, et la réponse de `pay` qui efface ces champs ; test « exercice du jour clos » **masqué** par la borne dans son montage naturel, et exercice du jour absent (janvier) non couvert par `cancellable` ; clés `invoice-settlement-cancel-blocked-*` qui font **échouer `lint-i18n-ownership`** ; nouveau texte d'`OWNED_BY_SUPPLIER_INVOICE` **faux** pour l'écriture d'achat ; mutation de l'AC 7 qui **passe à vide** quand deux gardes se recouvrent ; test de rollback par conflit de version sur un **état impossible** (version lue sous verrou). ⛔ **Règle de découpage déclenchée** : sévérité P2 → P3 **égale ou supérieure**. **Rien n'est corrigé** : Q5 et le découpage sont soumis à Guy avant tout patch. |
| 2026-09-24 | validate P2 | **Deux lentilles Haiku 4.5** en contexte frais, prompt versionné `25-3-a-validate-prompt-p2.md`. Lentille A : **1 MEDIUM, 2 LOW** ; lentille B : **0 finding**, axes déclarés tous exercés — mais deux d'entre eux effleurés, **repris par l'orchestrateur**, qui y trouve **1 MEDIUM et 1 LOW**. ✅ **MEDIUM (A)** : l'AC 11 faisait calculer `cancelBlockedBy` par la précédence du **socle** (AC 1), qui ignore l'exercice de l'origine, au lieu de celle du **geste** (AC 3-bis) — corrigé. ✅ **MEDIUM (orchestrateur)** : `SupplierInvoiceResponse::from_parts` est synchrone et partagée par quatre handlers, le calcul demandé ne pouvait pas y vivre — patron `with_settlement` prescrit, champs **absents** ailleurs plutôt que faux. ✅ LOW : l'AC 3-bis n'était rattaché à aucune tâche ; la lecture de « exercice clos » n'était pas dite (patron `delete_in_tx`). ❌ **LOW (A) réfuté** : « `errors.rs:2682` devrait être `:304` » — la 2682 est bien le mapping de `DbError::FiscalYearClosed`, la 304 est `AppError::FiscalYearClosed`, un autre type ; faux positif Haiku, et la citation précise désormais les deux. |
| 2026-09-24 | Q5 | Guy répond « continue » sans trancher Q5 : la fiche **applique la recommandation** — un règlement dont l'écriture est en exercice **clos** ne s'annule pas (AC 3-bis, `FISCAL_YEAR_CLOSED`, erreur existante), marquée comme **position de la fiche**, non comme arbitrage. La précédence des motifs d'annulation est écrite (exercice clos, statut, rapprochement, compte archivé). |
| 2026-09-24 | arbitrages (2) | ⛔ **Guy corrige Q1** : une facture fournisseur s'annule **dans tous les cas, sauf exercice clos** ; payée, son règlement **reste** et **redevient à lettrer** ; un paiement doit correspondre à une facture, au besoin créée pour lui (« comme bexio ») ; une facture client envoyée ne se modifie plus. ⇒ **25-3-c** créée au registre, qui absorbe #454 ; cette story garde l'annulation du **règlement**. L'ancienne Q4 est tranchée par ce modèle (le règlement d'une facture créditée est **à lettrer**, refus nommé ici) ; **#456 change de nature**. Q5 posée (borne « exercice clos » pour l'annulation d'un règlement). |
| 2026-09-24 | validate P1 | **Deux lentilles Sonnet** en contexte frais, prompt versionné `25-3-a-validate-prompt-p1.md`, **tous les axes déclarés** (non exercés : recalcul des compteurs i18n du frontend, contenu linguistique des trois locales non françaises, concurrence avec `confirm_batch`). **1 HIGH, 3 MEDIUM, 3 LOW**, tous vérifiés dans le code avant correction. ⛔ **HIGH (A)** : un avoir **peut** viser une facture partiellement réglée (`credit_notes.rs:291-304` ne lit que `paid_at`), la bascule en `cancelled` et laisse le règlement en place — que la garde `validated` de l'AC 6 rend alors inannulable ; et le test « facture couverte par un avoir » de l'AC 14 décrivait un état que l'application **ne produit pas**. Corrigé : le cas est nommé, refusé et renvoyé à **#456** (défaut antérieur, ouvert), la branche « résiduel ≤ 0 » est déclarée **défensive** et testée sur état forgé **dit tel**. ⚠️ **Deux défauts ANTÉRIEURS trouvés au passage, et ouverts** : **#456** (l'avoir sur facture partiellement réglée rend la créance créditrice) et **#455** (`amount_due` retranche un avoir **HT** d'un total **TTC**). **MEDIUM (B)** : aucun texte pour les motifs sur les **nouveaux** écrans — famille de clés `invoice-settlement-cancel-blocked-*` et **un seul** module de mapping ; côté fournisseur, `SupplierInvoiceResponse` gagne `settlementCancellable` / `settlementCancelBlockedBy` ; bullet API qui prescrivait un 403 à Consultation sur une route ouverte à tout rôle ; **le manuel n'a aucune section sur l'enregistrement d'un règlement client** — sous-section « Enregistrer et annuler un règlement » à créer. **LOW** : tables du test multi-tenant **énumérées** au lieu d'être comptées (il y en a cinq, pas quatre) ; second commentaire E2E périmé (`invoices.spec.ts:322-324`) ; deux plages de lignes décalées d'une unité. Symptômes grepés sur la fiche après correction : aucun résidu. Q4 posée à Guy. |
| 2026-09-24 | arbitrages | Guy tranche les trois questions dans le sens recommandé : **deux gestes** côté fournisseur (Q1), **lot confirmé laissé tel quel** (Q2), **`cancel` en issue distincte [#454]** (Q3). |
