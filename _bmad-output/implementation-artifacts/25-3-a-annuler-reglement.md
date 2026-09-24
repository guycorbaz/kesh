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
`EntryNotReversable`** pour tout motif autre que `AccountArchived` (`:1428-1437`). Une écriture de
règlement client y ressort en `OWNED_BY_SETTLEMENT`, une écriture de règlement fournisseur en
`OWNED_BY_SUPPLIER_INVOICE`. **Appeler `reverse_in_tx` tel quel depuis l'annulation échoue donc
toujours.**

Et la levée naïve est **fausse**, pour une raison de précédence :

- `reversal_blocker` ne rend que le **premier** motif (`:1299-1345`) ; `OwnedBySettlement` (rang 6)
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
   resté ≤ 0 (un avoir couvre le reste, ou un trop-perçu antérieur) **laisse `paid_at` intact**.

6. **Statut exigé : `validated`**, comme pour l'encaissement ; sinon `IllegalStateTransition`.
   ⚠️ Les factures réglées **avant** la table `invoice_settlements` (`20260827000001`, sans
   rattrapage) portent `paid_at` sans aucune ligne : elles n'ont **aucun règlement à annuler**, et
   la story ne leur en invente pas. Le dire au manuel (AC 13).

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

   ⚠️ **Pas de passage direct `paid → cancelled`.** Annuler une facture fournisseur réglée se fait
   en deux gestes : annuler le règlement (cette story), puis annuler la facture (`cancel`, qui
   n'accepte que `open` et ne bouge pas). *Recommandation — cf. Questions ouvertes, Q1.*

9. **Lot de paiement.** Une facture réglée par un lot **confirmé** (`payment_batches::confirm_batch`
   → `pay_in_tx`) reste annulable : le lot confirmé est l'historique d'un ordre transmis à la
   banque, et le cas typique d'annulation est précisément un **rejet bancaire**. Le lot n'est pas
   modifié. ⚠️ **À confirmer par Guy — Q2.** Une facture dans un lot `generated` ne peut pas être
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
      **`cancellable` + `cancelBlockedBy`** — calculés par la **même** fonction que le refus
      (AC 1), pour que l'écran masque le bouton **avant** le clic (patron de la 24-4a, AC 11).
      ⛔ Un bouton affiché qui échoue au clic est le défaut que la 24-4a a déjà payé.
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
    - ⚠️ `invoices_echeancier.spec.ts:173-182` affirme l'**absence** d'un bouton d'annulation
      « tant que #414 n'est pas livrée » : son commentaire devient faux. Le réécrire pour qu'il
      vérifie la **présence** du geste, et non le supprimer.

14. **Tests** (⛔ chaque garde **prouvée par mutation**, vue **rouge sur assertion**, et la mutation
    décrite au Dev Agent Record) :
    - socle : exemption étroite (un autre `settlement_id` ne lève rien) ; précédence poursuivie
      (AC 7) ; écriture d'achat fournisseur refusée malgré l'autorité (AC 2) ; `reverse_in_tx` sans
      autorité **inchangé** — les 27 tests de `journal_entry_reversal_e2e` passent **sans retouche** ;
    - client : règlement unique annulé (résiduel = TTC, `paid_at` NULL, ligne retirée, écriture
      inverse avec `reverses_entry_id`) ; règlement **partiel** parmi deux (AC 5) ; facture couverte
      par un avoir (AC 5, `paid_at` intact) ; double annulation → la seconde rend `NotFound` ;
      facture non `validated` ; exercice du jour absent ou clos → `FISCAL_YEAR_INVALID` ;
    - fournisseur : `paid → open`, colonnes vidées, re-règlement possible ensuite (le cycle
      complet, pas seulement l'annulation) ; facture `open` → refus ; facture réglée par lot
      confirmé (AC 9) ;
    - ⛔ **rollback** : un échec **après** la contre-passation (p. ex. l'`UPDATE` fournisseur en
      conflit de version) ne laisse **aucune** écriture inverse — avec une lecture **positive**
      dans la transaction avant l'échec (leçon de la 25-3-zero : un test de rollback aux assertions
      toutes négatives ne prouve rien) ;
    - **étanchéité multi-tenant** : société B ne peut ni lister ni annuler un règlement de A
      (`NotFound`, rien d'écrit) — le geste écrit dans quatre tables ;
    - API : les trois routes (200, 403 pour Consultation, 404, 409 avec `details`) ;
    - vitest : écran client (liste, bouton masqué avec motif, rafraîchissement) et fournisseur ;
    - Playwright : régler puis annuler, client et fournisseur.

15. **Documentation.** Manuel utilisateur FR (`user-manual.tex`) : le geste, client et fournisseur,
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
- [ ] **T2 — Le règlement client** (AC 4-7) : `cancel_settlement_in_tx` + wrapper, `paid_at`.
- [ ] **T3 — Le règlement fournisseur** (AC 8, 9).
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
  dans la recommandation actuelle — cf. Q3. ⚠️ Ne pas l'imiter pour le règlement fournisseur.
- La propagation du résiduel aux rapports agrégés (#416) ; l'imputation d'un écart (#384).

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

### Questions ouvertes — à trancher par Guy avant le développement

- **Q1** — Facture fournisseur réglée : **deux gestes** (annuler le règlement, puis la facture) ou
  un passage direct `paid → cancelled` ? *Recommandation : deux gestes* — chacun a son écriture et
  son audit, et le second existe déjà.
- **Q2** — Facture réglée par un **lot pain.001 confirmé** : annulation autorisée, lot inchangé ?
  *Recommandation : oui* — le cas d'usage est le rejet bancaire ; le lot reste l'historique de
  l'ordre transmis.
- **Q3** — `supplier_invoices::cancel` réécrit sa contre-passation à la main. Le faire passer par le
  socle **ici** (même famille de geste, pose enfin `reverses_entry_id`) ou ouvrir une **issue**
  distincte ? *Recommandation : issue distincte* — le corriger change la description et les liens
  d'écritures existantes, ce qui n'est pas l'objet de #414.

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
