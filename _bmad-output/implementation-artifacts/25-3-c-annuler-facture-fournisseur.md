# Story 25.3-c : Annuler une facture fournisseur — dans tous les cas, par le socle

Status: ready-for-dev

**Issue : [#454]**, qu'elle **ferme** : `closes #454` dans le **titre ET le corps** de la PR (squash ;
un `refs` partout laisserait l'issue ouverte sans signal).

**Grand-mère : `25-3-annuler-reglement-et-rapprochement.md`** (`split`) ; **mère des arbitrages :
`25-3-a-annuler-reglement.md`** (`split`), § Arbitrages et Change Log du 2026-09-24. **Socle** :
`25-3-zero-reverse-in-tx.md` (PR #453). **Sœurs mergées** : 25-3-a-1 (PR #457), 25-3-a-2 (PR #462),
25-3-b (PR #464) — cette story **réutilise** l'enum d'autorité, la queue commune des motifs, la
lecture en un instantané et la sonde d'entrelacement qu'elles ont posées.

## Story

En tant que comptable,
je veux annuler une facture fournisseur saisie par erreur **même si elle est déjà payée**,
afin que l'achat disparaisse de mes charges et de ma TVA préalable, tandis que le paiement réellement
sorti de la banque reste au grand livre, en attente d'être rattaché à la bonne facture.

## Arbitrages de Guy qui s'appliquent ici (2026-09-24)

- **Une facture fournisseur s'annule DANS TOUS LES CAS, sauf si l'exercice de son écriture d'ACHAT
  est clos** — l'exercice, **pas** le verrou de période (la contre-passation est datée du jour).
- **Payée, son règlement RESTE au grand livre et REDEVIENT À LETTRER.** Un paiement doit
  correspondre à une facture, au besoin créée pour lui (« comme bexio ») ; le lettrage est
  l'**Epic 15**.
- **Q3 de la 25-3-a** : `cancel` est réécrite par cette story, qui absorbe **#454**.

⚠️ **Ne pas contester ces arbitrages en revue** : en contester la mise en œuvre.

### ✅ Arbitrage du 2026-09-26 (Q1) — le règlement est DÉTACHÉ de la facture

*« Quand on annule une facture fournisseur payée, son règlement repasse dans l'état “à
réconcilier”, comme si aucune facture n'y était liée. »* (Guy)

Donc, à l'annulation d'une facture payée :

- les colonnes `settlement_type`, `settlement_bank_account_id`, `settlement_account_id`,
  `settlement_journal_entry_id` et `paid_at` sont **remises à `NULL`** — exactement comme
  l'annulation du règlement le fait (25-3-a-2, `supplier_invoices.rs:1031-1035`) ;
  `chk_supplier_invoices_paid_has_settlement` n'impose rien à une facture `cancelled`
  (`20260628000001_supplier_invoices.sql:70-72`) ;
- l'écriture de règlement **reste au grand livre, non contre-passée**, et n'est plus **possédée**
  par aucune pièce : `reversal_blockers` ne lui oppose plus `OWNED_BY_SUPPLIER_INVOICE`. C'est un
  paiement **sans facture**, en attente d'être rattaché (Epic 15) — ou contre-passé à la main
  depuis sa fiche d'écriture s'il était lui-même erroné ;
- le lien historique survit dans l'**audit** du geste (AC 3.5), qui garde l'identifiant de
  l'écriture de règlement, son type et sa date.

⚠️ **Vocabulaire — à ne pas confondre.** Dans Kesh, « à rapprocher » est l'état d'une
**transaction bancaire** importée (`reconciliation-cancel-confirm-*`, `messages.ftl:738-739`). Le
règlement fournisseur n'est **jamais** lié à une transaction bancaire (vérifié par la 25-3-a-2 :
les cinq sites qui posent `matched_entry_id` lient une écriture qu'ils viennent de créer). Cette
story ne touche donc **aucune** transaction bancaire : « à réconcilier » s'y lit **« sans facture,
à rattacher »**. Les textes (AC 7, 8, 10) disent « paiement sans facture », jamais « à rapprocher ».

⚠️ **Limite assumée** : Kesh n'a pas encore d'écran qui liste les paiements sans facture — le
compte 2000 n'a pas de sous-compte par fournisseur. Le paiement détaché se retrouve au **grand
livre** (compte 2000, solde débiteur) et dans le **journal d'audit**. Le lettrage est l'Epic 15.

## Ce que fait `cancel` aujourd'hui (#454), vérifié dans le code

`supplier_invoices::cancel` (`crates/kesh-db/src/repositories/supplier_invoices.rs:752-893`) :

1. verrouille la facture, exige `status = 'open'` — sinon `IllegalStateTransition` (**409
   `ILLEGAL_STATE_TRANSITION`**, message générique « Transition d'état interdite »,
   `kesh-api/src/errors.rs:2726-2733`) : une facture **payée** est refusée, **sans motif lisible** ;
2. appelle `guard_not_in_generated_batch` (`:474-505`) — même 409 générique ;
3. **réécrit la contre-passation à la main** : relit les lignes (`ORDER BY jel.id`), inverse D/C,
   `project_id: None` par ligne et le projet **document** en tête ; `create_in_tx(…, false)` — donc
   **jamais** `reverses_entry_id`, **jamais** `reversal_blockers`, description « Annulation facture
   fournisseur N » ;
4. `UPDATE … status = 'cancelled' … AND status = 'open'` ; audit `supplier_invoice.cancelled`
   `{ reversalJournalEntryId }`.

**Aucun contrôle d'exercice clos** : une facture ouverte dont l'achat est dans un exercice clos
s'annule aujourd'hui. **L'arbitrage le refuse désormais** — changement de comportement voulu, à
écrire au CHANGELOG.

Conséquences au grand livre, **visibles aujourd'hui** : la fiche de l'écriture d'achat d'une facture
annulée affiche `OWNED_BY_SUPPLIER_INVOICE` au lieu d'`ALREADY_REVERSED`, et l'écriture
d'annulation ne se présente pas comme une contre-passation (limite relevée par la 25-3-a-2, AC 8).

## Acceptance Criteria

1. **Une autorité ACHAT, contrôlée par le SOCLE.** Nouvelle variante
   `ReversalAuthority::SupplierPurchase { supplier_invoice_id }`
   (`journal_entries.rs:1458-1492`), qui lève `OwnedBySupplierInvoice` pour la facture nommée — **et
   pour son écriture d'ACHAT seulement**.
   - ⛔ Symétrique exact de `SupplierSettlement` : `reversal_blockers` attribue le même motif, avec
     le même identifiant de pièce, à l'écriture d'achat **et** à celle de règlement
     (`:1302-1307`). Le contrôle vit dans l'étape **1-bis** de `reverse_in_tx_inner`
     (`:1532-1553`) : `SELECT id FROM supplier_invoices WHERE id = ? AND company_id = ? AND
     purchase_journal_entry_id = ?` — faute de ligne, l'autorité est **retirée**.
   - **DRY** : une seule branche 1-bis pour les deux variantes, qui ne diffèrent que par la colonne
     (`settlement_journal_entry_id` / `purchase_journal_entry_id`). ⚠️ La colonne est choisie par un
     `match` sur la variante, **jamais** interpolée depuis une donnée — deux littéraux SQL
     complets, ou un `&'static str` rendu par le `match`.
   - Tests, par **appel direct du socle** : `SupplierPurchase` sur l'écriture de **règlement** d'une
     facture payée → `EntryNotReversable { OwnedBySupplierInvoice }` ; mutation « supprimer le
     contrôle pour `SupplierPurchase` » ⇒ rouge. Le test existant
     `supplier_authority_never_covers_the_purchase_entry` doit **rester vert** (la factorisation ne
     doit pas l'affaiblir).

2. **Les motifs — une tête facture, la queue COMMUNE sur l'écriture d'ACHAT.**

   | rang | motif | code | qui refuse, au clic |
   |---|---|---|---|
   | 1 | facture déjà `cancelled` | **`SUPPLIER_INVOICE_CANCELLED`** (variante `SupplierInvoiceCancelled`, **neuve**) | le geste, 409 |
   | 2 | exercice de l'écriture **d'achat** clos | `FISCAL_YEAR_CLOSED` (queue) | le geste, 409 |
   | 3 | écriture d'achat rapprochée | `MATCHED_BANK_TRANSACTION` (queue) — **inatteignable** (§ ci-dessous) | le socle |
   | 4 | compte de l'achat archivé | `ACCOUNT_ARCHIVED` (queue), `label` = numéro du compte | le socle, **400 qui nomme** |
   | 5 | aucun exercice ouvert le jour | `FISCAL_YEAR_INVALID` (queue) | le socle, 400 |
   | 6 | facture engagée dans un lot de paiement `generated` | **`SUPPLIER_INVOICE_IN_PAYMENT_BATCH`** (variante `SupplierInvoiceInPaymentBatch`, **neuve**) | le geste, 409 |

   - **Fonction de tête** `supplier_invoice_cancel_blocker(conn, company_id, id)` : rang 1, puis
     **appelle** `settlement_cancellation::settlement_entry_cancel_blocker(conn, company_id,
     purchase_journal_entry_id, None)` (rangs 2 à 5, **aucun jumeau** — la queue porte sur une
     **écriture**, pas sur un règlement ; son doc-comment le dit déjà, `:1-11`), puis le rang 6.
   - ⚠️ **Pourquoi le lot en DERNIER** : c'est le seul motif dont la levée est un geste lourd
     (annuler un lot de paiement). L'annoncer avant un exercice clos ferait annuler le lot pour
     buter ensuite sur la clôture. *Le définitif d'abord, puis ce qui se lève — et ce qui coûte le
     plus à lever en dernier.*
   - ⚠️ **Rang 3 inatteignable, et le calcul ne le suppose pas** : aucun des cinq sites qui posent
     `matched_entry_id` ne vise une écriture d'achat fournisseur (vérifié par la 25-3-a-2 pour le
     règlement ; **re-vérifier pour l'achat** : `grep -n "matched_entry_id" crates/kesh-api/src/routes/reconciliation.rs`).
     Si le recensement le rend, le socle refuse, comme ailleurs.
   - **Nouvelle erreur** `DbError::SupplierInvoiceNotCancellable { blocker: SettlementCancelBlocker }`
     → **409** avec `blocker.code()`. ⛔ **Distincte de `SettlementNotCancellable`**, dont le texte
     dit « ce règlement » — précédent exact : `ReconciliationNotCancellable` (25-3-b,
     `errors.rs:451-462`). Textes : famille propre, AC 8.
   - **La tête du RÈGLEMENT ne change pas** (`supplier_settlement_cancel_blocker`,
     `supplier_invoices.rs:909-938`) : une facture annulée n'a plus d'écriture de règlement
     (arbitrage Q1), son rang 1 rend `SUPPLIER_INVOICE_NOT_PAID` — vrai : il n'y a plus de
     règlement **de cette facture** à annuler. Le commentaire de `:924-926` (« l'absence ne peut
     venir que d'une facture non `paid` ») reste juste ; ne rien y ajouter.
   - ⚠️ Codes neufs : `grep -rn "SUPPLIER_INVOICE_CANCELLED\|SUPPLIER_INVOICE_IN_PAYMENT_BATCH" crates frontend/src`
     doit être **vide** avant ajout (collision). `SUPPLIER_INVOICE_NOT_OPEN`
     (`payment_batches.rs:265`) a un autre sens et **ne se réemploie pas**.

3. **Le geste.** `cancel_in_tx` (cœur, sans `BEGIN`/`COMMIT`) + `cancel` (wrapper, **signature
   inchangée** — la route et les tests existants l'appellent), sur le modèle de
   `cancel_settlement_in_tx` (`:962-1080`) :
   1. **Les verrous d'abord, tous, avant toute lecture non verrouillante** : facture `FOR UPDATE`,
      **puis** l'écriture d'achat **et son exercice** (`SELECT fy.id FROM journal_entries je JOIN
      fiscal_years fy ON fy.id = je.fiscal_year_id WHERE je.id = ? AND je.company_id = ? FOR
      UPDATE`).
      ⛔ **Leçon de la 25-3-b, à ne pas réapprendre** : sous `REPEATABLE READ`, une lecture **non
      verrouillante** fige l'instantané ; placée **avant** un verrou, elle fait relire après
      l'attente un exercice « ouvert » que la clôture concurrente vient de fermer. La lecture du
      lot (`payment_batch_items`) et celles de la queue sont non verrouillantes : elles viennent
      **après** les deux verrous. Le doc-comment de `cancel_in_tx` l'écrit, et avertit qu'un
      appelant qui compose ne doit avoir fait **aucune** lecture non verrouillante avant.
   2. `supplier_invoice_cancel_blocker` : rangs **1, 2 et 6** → `SupplierInvoiceNotCancellable`
      (409) ; les autres → on continue, le socle refuse avec son erreur canonique (même partage que
      la 25-3-a-2, c'est ce qui garde le 400 qui **nomme** les comptes archivés).
      ⚠️ `guard_not_in_generated_batch` **n'est plus appelée** par `cancel` : son contrôle devient
      le rang 6, **même requête**. Elle reste pour `pay` (`:520`). Ne pas écrire la requête du lot
      deux fois : l'extraire en une fonction `in_generated_batch(conn, id) -> Option<i64>` appelée
      par la garde **et** par le rang 6.
   3. **Contre-passation** de `purchase_journal_entry_id` par
      `journal_entries::reverse_owned_in_tx(…, ReversalAuthority::SupplierPurchase { supplier_invoice_id: id })`.
      **Plus aucune réécriture à la main** : les lignes lues par `jel.id`, le `project_id: None` par
      ligne et la description « Annulation facture fournisseur N » **disparaissent**.
   4. `UPDATE supplier_invoices SET status = 'cancelled', settlement_type = NULL,
      settlement_bank_account_id = NULL, settlement_account_id = NULL,
      settlement_journal_entry_id = NULL, paid_at = NULL, version = version + 1 WHERE id = ? AND
      company_id = ? AND version = ? AND status IN ('open', 'paid')` — 0 ligne ⇒
      `OptimisticLockConflict`. Pour une facture `open`, les colonnes sont déjà `NULL` : une seule
      requête pour les deux cas. ⛔ **L'écriture de règlement n'est PAS contre-passée** — elle est
      **détachée** (arbitrage Q1).
      ⚠️ **Ordre imposé** : la contre-passation (étape 3) **précède** l'`UPDATE`. L'inverse ne
      casserait pas l'autorité d'achat (elle lit `purchase_journal_entry_id`, non vidé), mais le
      patron de la 25-3-a-2 est celui-là ; ne pas en créer un second.
   5. Audit `supplier_invoice.cancelled` (action **existante**, libellés ×4 inchangés) — payload
      enrichi : `previousStatus` (`open` | `paid`), `purchaseJournalEntryId`,
      `reversalJournalEntryId`, et, si la facture était payée, `settlementJournalEntryId`,
      `settlementType` et `paidAt` — ⛔ **seule trace du lien**, les colonnes étant remises à
      `NULL`. Le socle écrit en plus `journal_entry.reversed` — voulu.

   ⚠️ **Ce qui change au grand livre, et c'est voulu** (#454) : l'écriture d'annulation porte
   `reverses_entry_id` et la description du socle (« Contre-passation écriture n° … », suffixée de
   l'exercice d'origine s'il diffère) ; la fiche de l'écriture d'achat affiche désormais
   `ALREADY_REVERSED` (rang 2 de `reversal_blockers`, avant la propriété). Les projets sont repris
   **ligne par ligne** — identiques à l'ancien comportement, puisque l'achat recopie le projet
   document sur chaque ligne à l'insertion (`journal_entries.rs:426`). Les écritures d'annulation
   **déjà passées** ne sont **pas** reliées rétroactivement (dit par #454).

   ⚠️ **Écriture comptable d'une facture payée annulée** : achat D charges + TVA préalable / C 2000 ;
   règlement D 2000 / C banque (reste) ; annulation D 2000 / C charges + TVA préalable. Charges et
   TVA reviennent à zéro, **2000 porte un solde DÉBITEUR** du TTC (créance sur le fournisseur :
   le paiement sans facture), la banque reste créditée. Test d'équilibre à l'AC 9.

4. **La route** — `POST /api/v1/supplier-invoices/{id}/cancel`, **inchangée** (Comptable+, clés API
   d'écriture admises). Registre de routes **inchangé** : ni route ni action d'audit neuves — le
   vérifier (`audit_route_registry.rs` vert sans modification). La réponse passe de
   `SupplierInvoiceResponse::from_parts` (champs de lecture `null`) au constructeur en un
   instantané (AC 5) — l'écran remplace son état par cette réponse (`+page.svelte:159`).

5. **Les champs de lecture.** `SupplierInvoiceResponse` gagne `cancellable: Option<bool>`,
   `cancelBlockedBy: Option<&'static str>`, `cancelBlockedLabel: Option<String>` — même discipline
   que les champs du règlement (`routes/supplier_invoices.rs:84-104`) : **`null` = non calculé**,
   jamais une valeur par défaut qui mentirait.
   - Calculés par **la fonction même qui refuse** (AC 2), **dans le même instantané** que la facture
     et le motif du règlement : `get_settlement_view` (`supplier_invoices.rs:1127-1153`) gagne le
     champ `cancel_blocker`, lu dans **sa** transaction de lecture — ⛔ aucune de ces lectures ne
     devient verrouillante (doc-comment `:1117-1126`). Renommer la vue et son constructeur pour
     ce qu'ils sont désormais (p. ex. `get_view` / `load_with_cancellations`) est **permis**, pas
     exigé ; si renommés, grep des appelants.
   - Servis par le **GET**, `pay`, l'annulation du règlement **et l'annulation de la facture**.
     `SupplierInvoiceListItemResponse` n'est pas touché.

6. **Les textes du grand livre — `OWNED_BY_SUPPLIER_INVOICE`.** Le texte actuel (« annulez la
   facture ou son règlement depuis sa fiche ») est **faux** pour l'écriture d'achat d'une facture
   annulée **avant** cette story (annulation non reliée, § Dev Notes « Legacy ») : la facture est
   déjà annulée, et elle n'a pas de règlement. ⚠️ Après cette story, le cas ne naît plus — l'achat
   annulé ressort en `ALREADY_REVERSED`, le règlement détaché n'a plus de propriétaire. Nouveau
   texte, **vrai dans tous les cas** :
   « Cette écriture appartient à une facture fournisseur : elle se corrige depuis la fiche de la
   facture, qui indique ce qui est possible. » (patron : le texte d'`OWNED_BY_SETTLEMENT`,
   `kesh-api/src/errors.rs:2472-2474`). **Sites, ensemble** : FTL
   `journal-entries-reverse-blocked-supplier-invoice` ×4 (fr-CH `:342`, autres `:348`), repli
   serveur `kesh-api/src/errors.rs:2468-2471`, repli Svelte
   `journal-entries/[id]/+page.svelte:151-155`, doc-comment `kesh-db/src/errors.rs:66-69` (qui
   renvoie aussi à `cancel`), manuel `:504-507`.

7. **L'écran** (`supplier-invoices/[id]/+page.svelte`).
   - **Bouton « Annuler la facture »** si `cancellable === true` **et** `canManage` (`:57-59`) —
     aujourd'hui il est affiché **sans condition de rôle** et **seulement** sur une facture `open`
     (`:324-331`) : un Consultant le voit, clique, et reçoit un 403. Il apparaît désormais **aussi**
     sur une facture `paid`, à côté d'« Annuler le règlement ».
   - Le **motif** à la place du bouton si `cancellable !== true` et `cancelBlockedBy` est posé,
     **sauf** `SUPPLIER_INVOICE_CANCELLED` (une facture annulée n'annonce pas qu'on ne peut pas
     l'annuler).
   - **Confirmation** avant l'envoi, qui dit ce qui va s'écrire :
     - `open` : « Annuler cette facture ? Une écriture inverse de l'achat, datée d'aujourd'hui, sera
       passée au grand livre. »
     - `paid` : la même, **plus** « Elle est payée : son règlement reste au grand livre, détaché
       de la facture — un paiement sans facture, à rattacher. Si c'est le paiement lui-même qui est
       erroné, annulez plutôt le règlement d'abord. » ⚠️ Ce dernier conseil n'est pas une
       obligation (le paiement détaché reste contre-passable depuis sa fiche d'écriture), mais
       c'est le chemin qui garde le lien au grand livre (`reverses_entry_id` posé au titre de la
       facture).
   - ⛔ **Refus au clic affiché LOCALEMENT** (409 `code`, 400 `details.rejected[]`), **jamais** dans
     `errorMsg`, qui remplace **toute la fiche** (`:200-201`) — c'est le défaut actuel de `cancel()`
     (`:161`). Patron : `settlementCancelError` (`:60-63`, `:146-149`).
   - **Facture `cancelled`** — aujourd'hui la fiche n'affiche **rien** sous les lignes. Afficher
     « Facture annulée. » — et **rien d'autre** : ses colonnes de règlement sont vides (arbitrage
     Q1), la fiche ne sait plus qu'elle a été payée. ⚠️ Ne pas aller chercher l'ancien règlement
     dans l'audit pour l'afficher : ce serait ré-attacher à l'écran ce que l'arbitrage détache.
   - Après l'annulation : notification de succès, état remplacé par la réponse.

8. **Les textes.**
   - Famille **propre** au geste, dans `features/supplier-invoices/` (préfixe
     `supplier-invoices-` : `lint-i18n-ownership` l'accepte, cf. 25-3-a-2 AC 8) :
     `supplier-invoices-cancel-blocked-cancelled`, `-fiscal-year-closed`, `-bank-match`,
     `-account-archived`, `-no-fiscal-year`, `-in-payment-batch`. ⛔ **Ne pas réemployer** la queue
     `settlement-cancel-blocked-*` : elle dit « ce règlement » (précédent : la 25-3-b a créé
     `reconciliation-cancel-blocked-*` pour la même raison). Le rang 3 a son texte bien
     qu'inatteignable : le type TypeScript est l'union de ce que le serveur **peut** rendre, et le
     serveur ne le suppose pas.
   - Textes : exercice clos → « Cette facture appartient à un exercice clôturé : un administrateur
     doit rouvrir l'exercice pour pouvoir l'annuler. » ; lot → « Cette facture figure dans un lot de
     paiement en cours : annulez d'abord le lot. » ; les autres calqués sur la famille
     `reconciliation-cancel-blocked-*` (`messages.ftl:730-735`).
   - **Tête du règlement** : **inchangée** (AC 2) ; aucune clé neuve côté règlement.
   - Textes de confirmation et de succès (AC 7), « Facture annulée. ».
   - **Quatre** locales ; repli serveur de `SupplierInvoiceNotCancellable` (fonction
     `supplier_invoice_cancel_blocked_text`, patron `reconciliation_cancel_blocked_text`) et de
     `SettlementNotCancellable` pour le nouveau code ; replis en dur **mot pour mot** le FTL fr-CH.
   - Gardes i18n (`i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`) **recomptées depuis la
     source**, jamais incrémentées de confiance.

9. **Tests** (mutations vues **rouges sur assertion**, restaurées ensuite) :
   - **Socle** : AC 1 (autorité achat refusée sur le règlement ; test fournisseur existant vert).
   - **Facture ouverte** : annulation → `cancelled` ; la contre-passation porte
     `reverses_entry_id = purchase_journal_entry_id` ; `reversal_blocker(achat)` rend
     `AlreadyReversed` ; charges, TVA préalable et 2000 à zéro (le test
     `cancel_reverses_purchase_entry` est **complété**, pas remplacé).
   - **Facture payée** : ⛔ `cancel_paid_invoice_rejected` (`supplier_invoices_repository.rs:452`)
     pose l'**ancien** comportement — il est **réécrit** en `cancel_paid_invoice_detaches_its_settlement` :
     `cancelled` ; colonnes `settlement_*` et `paid_at` **à `NULL`** ; l'écriture de règlement
     existe toujours et **n'a pas** de contre-passation ; soldes : charges et TVA à 0, **2000
     débiteur du TTC**, banque (ou compte interne) crédité du TTC ; audit portant
     `settlementJournalEntryId`. ⛔ **La preuve du détachement** : `reversal_blockers(écriture de
     règlement)` ne contient **plus** `OwnedBySupplierInvoice`, et `journal_entries::reverse` sur
     elle **réussit** — mutation « ne pas vider `settlement_journal_entry_id` » ⇒ rouge. Puis
     `cancel_settlement` → 409 `SUPPLIER_INVOICE_NOT_PAID`.
   - **Précédence**, paire par paire atteignable : déjà annulée (seconde annulation → rang 1) ;
     exercice d'achat clos (facture ouverte **et** facture payée) ; compte d'achat archivé → **400
     qui nomme** au clic, `ACCOUNT_ARCHIVED` + numéro à la lecture ; aucun exercice ouvert le jour ;
     lot `generated` ; **paires** « exercice clos + lot » → `FISCAL_YEAR_CLOSED` et « compte
     archivé + lot » → `ACCOUNT_ARCHIVED`, **lecture et clic** d'accord. Mutation « lot avant la
     queue » ⇒ rouge.
   - ⛔ **Concurrence** : une clôture d'exercice concurrente **attend** l'annulation (patron
     `a_concurrent_close_waits_for_the_cancellation`, sonde
     `kesh_db::test_fixtures::attendre_une_requete_en_cours`) ; mutation « verrou écriture + exercice
     retiré » ⇒ rouge. Et la variante 25-3-b : clôture **validée pendant** l'attente du verrou
     facture → le geste refuse `FISCAL_YEAR_CLOSED` (mutation « lecture non verrouillante avant les
     verrous » ⇒ rouge).
   - **Projet** : `cancel_nets_project_to_zero` reste vert ; ajouter une facture **à deux lignes**
     de comptes différents — la contre-passation reprend le projet **sur chaque ligne**.
   - **Composition et rollback** (`cancel_in_tx`, lecture **positive** dans la transaction,
     abandon, rien n'a changé — ni facture, ni écriture, ni audit).
   - **Étanchéité multi-tenant** table par table (`journal_entries`, `journal_entry_lines`,
     `supplier_invoices`, `audit_log`) : annuler depuis une autre société → `NotFound`, rien d'écrit.
   - **API** (fichier `supplier_settlement_cancel_e2e.rs` étendu, ou jumeau `supplier_invoice_cancel_e2e.rs`) :
     200 (facture payée, champs de lecture présents), 403 Consultation, 404 autre société, 409 avec
     `code` pour chaque motif refusé par le geste, 400 `ACCOUNT_ARCHIVED` nommé.
   - **Vitest** (`supplier-settlement-page.test.ts` étendu ou fichier voisin) : bouton selon
     `cancellable` **et** le rôle, présent sur `paid` ; motif affiché, **pas** pour
     `SUPPLIER_INVOICE_CANCELLED` ; confirmation `paid` qui dit « paiement sans facture » ; refus au
     clic affiché **sans effacer la fiche** ; fiche `cancelled` qui dit « Facture annulée. » sans
     lien de règlement.
   - **Playwright** (`supplier-invoices.spec.ts`) : payer, annuler la facture, voir « Facture
     annulée », puis ouvrir l'écriture de règlement depuis le grand livre et constater qu'elle est
     contre-passable.

10. **Documentation.**
    - Manuel utilisateur, § « Factures fournisseurs et paiements » (`:1125-1166`) : **sous-section
      « Annuler une facture fournisseur »** — ouverte ou payée ; ce qui s'écrit ; le règlement qui
      reste au grand livre, **détaché**, paiement sans facture (où le retrouver : compte 2000,
      journal d'audit ; le rattachement est à venir) ; le conseil « paiement erroné → annuler
      plutôt le règlement d'abord » ; les
      refus (exercice clos, lot en cours, compte archivé, exercice du jour). La phrase « l'annuler
      elle-même » de `:1159` s'y rattache.
    - `:504-507` (le chemin par pièce) et la liste `:1873-1885` : facture fournisseur « annulable
      même payée » ; ⛔ l'exception d'exercice clôturé (`:1877-1882`) **s'étend** à l'annulation
      d'une facture fournisseur.
    - PDF régénéré et **contrôlé aplati** (`pdftotext … | tr '\n' ' ' | tr -s ' '`).
    - `docs/api-external.md:271` : « qui annule la **facture** (seulement `open`) » devient faux —
      réécrire ; décrire les codes 409 et les trois champs de lecture (`:273`).
    - `README.md:40` : « annulation de la facture » → « annulation de la facture, **même payée** ».
    - `CHANGELOG` `[0.12.1] — Non publié` : *Changed* (annulation d'une facture payée ; refus si
      l'exercice de l'achat est clos — **changement de comportement** ; description de l'écriture
      d'annulation) ; *Fixed* (#454 : `reverses_entry_id` posé, `ALREADY_REVERSED` au grand livre,
      bouton masqué au Consultant, refus qui n'efface plus la fiche).
    - ⛔ **C'est cette story qui ferme #454** : relancer
      `grep -rn "#454\|réécrit sa contre-passation\|supplier_invoices::cancel" crates frontend/src docs`
      et réécrire ce qui décrit encore l'ancien `cancel` : `journal_entries.rs:1427-1433`,
      `supplier_invoices.rs:752-753` et `:954-955`, `kesh-db/src/errors.rs:66-69`.
    - **Gate complet** (`kesh-db` touché) ; E2E complète avant push, base `kesh_e2e` reconstruite.

## Tasks / Subtasks

- [ ] **T1 — Socle** : `ReversalAuthority::SupplierPurchase`, 1-bis factorisée (AC 1).
- [ ] **T2 — Motifs** : variantes `SupplierInvoiceCancelled`, `SupplierInvoiceInPaymentBatch` ;
      `supplier_invoice_cancel_blocker` ; tête du règlement dédoublée ; `in_generated_batch`
      extraite ; `DbError::SupplierInvoiceNotCancellable` et son mapping 409 (AC 2).
- [ ] **T3 — Le geste** : `cancel_in_tx` + `cancel` (AC 3), audit enrichi.
- [ ] **T4 — Lecture** : `get_settlement_view` étendue, trois champs, réponse de `cancel` (AC 4, 5).
- [ ] **T5 — Textes** : `OWNED_BY_SUPPLIER_INVOICE` (AC 6), familles et replis (AC 8).
- [ ] **T6 — Écran** (AC 7).
- [ ] **T7 — Tests** (AC 9), dont les mutations.
- [ ] **T8 — Documentation et gates** (AC 10), PR `closes #454`.

## Dev Notes

- **Aucune migration.** Aucun des garde-fous P1-P8 ne s'applique ; ne pas en créer une « pour
  lier » les anciennes annulations (#454 l'exclut).
- **Ce qui reste de `cancel` après la story** : verrous, motifs, un appel au socle, un `UPDATE`,
  un audit. S'il reste une ligne qui lit `journal_entry_lines`, c'est un résidu.
- `guard_not_in_generated_batch` garde son `SELECT … FOR UPDATE` de la facture pour `pay` ; dans
  `cancel_in_tx`, la facture est déjà verrouillée à l'étape 1 — ne pas la reverrouiller par la
  garde, appeler `in_generated_batch` (AC 3.2).
- `create_batch` exige `open` et verrouille la facture (`payment_batches.rs:265-283`) : il se
  sérialise avec `cancel` sur le verrou facture. Une facture annulée ne peut plus entrer dans un lot.
- La **période verrouillée du jour** (24-4c) n'est pas un motif de lecture — refusée au clic par
  le socle (`PERIOD_LOCKED`, 400). Limite assumée, identique à la queue (`settlement_cancellation.rs:30-32`).
- **Legacy** : les factures annulées **avant** cette story ont une annulation non reliée ; leur
  écriture d'achat affiche encore `OWNED_BY_SUPPLIER_INVOICE`, avec le nouveau texte (AC 6), qui
  reste vrai. Aucune donnée réelle à préserver (CLAUDE.md, « Deployed is not the same as keeping
  the books »).
- L'export de souveraineté (25-5-a) exporte `supplier_invoices` tel quel : une facture `cancelled`
  y a des colonnes de règlement vides ; le lien reste retrouvable par `audit_log` (exportée) et au
  grand livre (`journal_entries.csv`).
- ⚠️ **Hors périmètre** : le lettrage (Epic 15) ; « désannuler » une facture ; la dette #455/#456
  côté client.

### References

- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs:474-505,752-893,909-1153] — garde de lot, `cancel`, tête et geste du règlement, vue.
- [Source: crates/kesh-db/src/repositories/journal_entries.rs:1239-1385,1438-1672] — recensement, autorité, socle.
- [Source: crates/kesh-db/src/repositories/settlement_cancellation.rs] — queue commune.
- [Source: crates/kesh-db/src/errors.rs:57-110,186-250,440-462] — `ReversalBlocker`, `SettlementCancelBlocker`, erreurs d'annulation.
- [Source: crates/kesh-api/src/errors.rs:2462-2474,2613-2616,2726-2733] — replis et mappings.
- [Source: crates/kesh-api/src/routes/supplier_invoices.rs:59-170,375-390] — réponse, `cancel`.
- [Source: frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte] — la fiche.
- [Source: frontend/src/lib/features/supplier-invoices/settlement-cancel.ts] — tête des textes.
- [Source: crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:65-72] — contraintes de statut.
- [Source: _bmad-output/implementation-artifacts/25-3-a-annuler-reglement.md] — arbitrages du 2026-09-24.
- [Source: _bmad-output/implementation-artifacts/25-3-a-2-annuler-reglement-fournisseur.md] — autorité fournisseur, verrou d'exercice, vue en un instantané.
- [Source: _bmad-output/implementation-artifacts/25-3-b-annuler-rapprochement.md] — famille de textes propre, leçon `REPEATABLE READ`.

## Questions pour Guy

- ✅ **Q1 — tranchée le 2026-09-26** : le règlement est **détaché** (§ Arbitrage du 2026-09-26).
  Aucune question ouverte.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-26 | arbitrage Q1 | Guy : *« son règlement repasse dans l'état “à réconcilier”, comme si aucune facture n'y était liée »* ⇒ le règlement est **détaché** : colonnes `settlement_*` et `paid_at` remises à `NULL`, écriture de règlement libre (contre-passable depuis sa fiche), lien gardé par l'audit. Retirés : le dédoublement de la tête du règlement et sa clé ; la fiche `cancelled` n'affiche plus de lien. « À réconcilier » lu « sans facture, à rattacher » — aucune transaction bancaire n'est touchée (dit en § Arbitrage). |
| 2026-09-26 | create | Spécifiée (Opus 5.5) depuis l'arbitrage du 2026-09-24, #454 et le code de `main` à `0e4c2682`. Modèle « à lettrer » repris du côté client (`INVOICE_CREDITED`) — **Q1 posée**. Aucune migration. Prochaine étape : `bmad-create-story validate`. |
