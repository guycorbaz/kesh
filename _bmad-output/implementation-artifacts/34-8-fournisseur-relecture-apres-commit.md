# Story 34-8 : une facture fournisseur n'est plus annoncée en échec après son commit (#938)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** C'est la ligne « Non traité, à tracer » de
la fiche 33-6, qui fermait le même défaut côté client. Registre : `e34-choix-autonomes-e343.md`, C-e343-9 à 12 et 15. Commits en
`refs #938` ; à la PR : `closes #938`.*

## Story

En tant que Comptable, je veux deux choses :
- que la réponse des neuf gestes sur une facture fournisseur (valider, repasser en brouillon, modifier, payer, annuler,
  annuler le règlement, compléter un document, joindre ou détacher un document) dise ce qui a été écrit, sans relecture
  après le commit, pour qu'aucun 404 ni 500 n'annonce en échec un geste réussi ;
- que la fiche relise après une réponse perdue au lieu de se taire, pour ses sept gestes (tous sauf modifier et compléter,
  qui vivent sur d'autres pages, voir « Ce qui ne se traite pas ici »).

*(Phrase scindée à la validation P3, 34-8-F3.)*

## Ce que le code dit (prémisses établies)

- **Le défaut est là : neuf relectures après le commit.** Elles passent toutes par
  `SupplierInvoiceResponse::load_with_settlement_cancellation(&state.pool, …)`
  (`crates/kesh-api/src/routes/supplier_invoices.rs:268-277`), qui ouvre une transaction de lecture neuve
  (`get_settlement_view`, `crates/kesh-db/src/repositories/supplier_invoices.rs:3002-3038`) et rend 404 si la facture manque.
  Numéros relus à `a75835db`, différents de ceux de la fiche 33-6 parce que la 33-1 a récrit le fichier :

  | Route | Handler | Écriture | Relecture |
  |---|---|---|---|
  | `PUT /supplier-invoices/{id}` | `update_supplier_invoice` | `update_supplier_invoice_once` (tx propre, `:805-848`) | `:796` |
  | `POST …/validate` | `validate_supplier_invoice` | `supplier_invoices::validate` (`in_own_tx!`, `:1053-1064`) | `:901` |
  | `POST …/unvalidate` | `unvalidate_supplier_invoice` | `supplier_invoices::unvalidate` (`in_own_tx!`, `:1067-1078`) | `:937` |
  | `POST …/pay` | `pay_supplier_invoice` | `supplier_invoices::pay` (`:1890-1915`) | `:967` |
  | `POST …/cancel` | `cancel_supplier_invoice` | `supplier_invoices::cancel` (`:2678-2694`) | `:991` |
  | `POST …/settlement/cancel` | `cancel_supplier_invoice_settlement` | `supplier_invoices::cancel_settlement` (`:2951-2967`) | `:1023` |
  | `POST /imported-supplier-invoices/{id}/complete` | `complete_import` (`imported_supplier_invoices.rs:128-149`) | `complete_import_once` (`:159-240`) | `:141` |
  | `POST /supplier-invoices/{id}/source-document` | `attach_source_document` (`:577-603`) | `attach_source_document_once` (`:607-670`) | `:595` |
  | `DELETE /supplier-invoices/{id}/source-document` | `detach_source_document` (`:689-708`) | `detach_source_document_once` (`:712-…`) | `:700` |

- **Trois de ces angles morts sont écrits et assumés.** `imported_supplier_invoices.rs:120-123` (C-30-2-42, « patron de
  `pay` »), `:573-578` et `:682-685` (C-30-2-52) : « une erreur de base entre le `COMMIT` et la relecture rend une erreur alors
  que [le geste est fait] ». La story les ferme, et ces doc-comments sont récrits.
- **La lecture est déjà transactionnelle.** `get_settlement_view` lit la facture, ses lignes, les deux motifs de refus, le
  dernier lot confirmé et le document dans **une** transaction (`:3007-3029`), avec `&mut tx` partout. L'extraire en
  `get_settlement_view_in_tx` coûte une signature.
- ⚠️ **Sa règle propre** (doc-comment, `:2985-3001`) : aucune de ces lectures ne doit devenir verrouillante. Dans une
  transaction d'écriture, une lecture ordinaire voit l'instantané de la **première** lecture ordinaire de la transaction, plus
  ses propres écritures. Ce que la transaction a écrit est donc juste. Ce qui vient d'autres lignes peut dater du début de la
  transaction : les drapeaux d'annulation sont indicatifs, le geste suivant les recalcule sous verrou (C-e343-10). À écrire au
  doc-comment de la forme `_in_tx`.
- **Les enveloppes.**
  - `validate` et `unvalidate` passent par la macro `in_own_tx!` (`:411-424`), que la garde `syn` de la 33-6 ne lit pas
    (limite écrite, `invoice_settlement_response_e2e.rs:409-412`).
  - `pay`, `cancel` et `cancel_settlement` sont écrites en clair.
  - Les formes `_in_tx` (`pay_in_tx`, `cancel_in_tx`, `cancel_settlement_in_tx`, `validate_in_tx`, `unvalidate_in_tx`) sont
    réemployées par la confirmation d'un lot et par le rapprochement : elles ne doivent rien lire de plus.
- **La vue garde les noms de champs.** `SupplierInvoiceSettlementView` (`:2973-2990`) a `invoice` et `lines` comme
  `SupplierInvoiceWithLines` (`:64`). Appelants des cinq enveloppes hors des routes : **155** sites, tous en test, dans `kesh-db/tests/`, `kesh-api/tests/` et
  `kesh-report/tests/`. Décompte : `git grep -nE "supplier_invoices::<f>\(" -- crates`, hors `_in_tx` et hors
  `src/routes` ; `validate` 18, `unvalidate` 29, `pay` 40, `cancel` 43, `cancel_settlement` 25. Ceux qui lisent `.invoice` ou `.lines` compilent sans changement.
  `SupplierSettlementCancellation.invoice` (`:2763-2768`) est un `SupplierInvoiceWithLines`.
- **La garde de structure de la 33-6** (`crates/kesh-api/tests/invoice_settlement_response_e2e.rs:395-516`) : `Around`,
  `around`, `expr_calls`. Elle est réutilisable telle quelle, à condition de la sortir du fichier (C-e343-11).
  `tests/common/` existe.
- **Un couplage à prévoir.** `services_motif.rs:2480-2483` compte les fonctions de `routes/supplier_invoices.rs` : 11 `pub async
  fn` en tête de ligne, et 13 `async fn` toutes visibilités et indentations comprises. `load_with_settlement_cancellation` est
  une méthode indentée (`:268`), la seule `async fn` indentée du fichier. Son retrait donne **12 et 11** : 13 passe à 12,
  11 ne bouge pas. Comptage : `grep -cE "^\s*(pub(\([a-z]+\))? )?async fn"` rend 13, et `^pub async fn` rend 11. La garde
  le confirmera (validation P1, 34-8-F6 ; P2, 34-8-F5). Pour `imported_supplier_invoices.rs`
  (`:2391-2394`, 9 et 12), la garde rougit si une signature change de visibilité. Recompter à la source, et mettre à jour
  avec la raison.
- **L'écran** (commentaire de Guy sur #938, 2026-11-07). Fiche `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte` :
  - `pay()` (`:193-221`) n'affiche rien sur une erreur hors `ApiError` (`:215-217`) ;
  - `cancelSettlement()` (`:259-263`), `cancel()` (`:299-300`) et `confirmValidate()` (`:383-386`) affichent `common-error`
    sans relire ;
  - `confirmUnvalidate()` (`:432-444`) relit seulement sur `isStaleState`.

  `reloadAfterRefusal()` (`:477-483`) existe. `isSettlementOutcomeUnknown` (`frontend/src/lib/shared/utils/unknown-outcome.ts`)
  aussi.
- **Pas de double écriture côté fournisseur.**
  - Payer : la garde est `inv.status != "open"`, `kesh-db/src/repositories/supplier_invoices.rs:1980`.
  - Valider et repasser en brouillon : la version.
  - Annuler et annuler le règlement : le statut.
  - Compléter : `IMPORT_NOT_PENDING_COMPLETION`.
  - Joindre : `IMPORT_NOT_PENDING_COMPLETION`. Détacher : `SOURCE_DOCUMENT_NOT_FOUND`.

  Le défaut est un affichage faux, non un geste doublé : P2.

## Acceptance Criteria

1. **AC-1 — kesh-db.**
   - `get_settlement_view_in_tx(tx, company_id, id) -> Result<Option<SupplierInvoiceSettlementView>, DbError>`, réemployée par
     `get_settlement_view` (DRY).
   - **Signature** (validation P1, 34-8-F4 ; P2, 34-8-F2 et F4 ; C-e343-15) :
     `pub async fn commit_with_view<T, E: From<DbError>>(tx: Transaction<'_, MySql>, company_id: i64,
     result: Result<T, E>, view_id: impl FnOnce(&T) -> i64) -> Result<(SupplierInvoiceSettlementView, T), E>`.
     - **L'identifiant vient d'une fermeture `view_id`, appelée seulement sur `Ok`** (validation P3, 34-8-F1). La complétion
       **crée** la facture : son identifiant n'existe qu'au retour de `create_draft_in_tx`
       (`routes/imported_supplier_invoices.rs:194`, puis `created.invoice.id`, `:202`). Un `id: i64` passé avant l'écriture ne
       la servirait pas.
       - La complétion passe `|c| c.invoice.id`.
       - Les autres passent `|_| id`, l'identifiant étant celui du chemin.
       - Sur `Err`, la fermeture n'est pas appelée, et aucun identifiant factice n'est inventé.
     - **Générique sur l'erreur**, pour servir aussi les quatre `_once` de `kesh-api`, qui travaillent en `AppError`
       (`AppError: From<DbError>`). Une seule lecture-puis-commit, et non quatre copies.
     - **`pub`**, parce que `kesh-api` l'appelle et que le test de M5 vit dans `kesh-db/tests/`. Une fonction
       `pub(crate)` serait invisible de l'un et de l'autre : c'est le précédent de la 31-1, P6.
     - C'est une fonction ordinaire, non une macro, pour que la garde `syn` la lise.
     - Sur `Err`, elle annule la transaction et rend l'erreur.
     - Sur `Ok`, elle lit la vue, puis valide la transaction.
     - Une vue absente (`None`) après une écriture réussie est une `DbError::Invariant` qui annule la transaction.
   - `validate`, `unvalidate`, `pay` et `cancel` la terminent, et rendent la vue.
   - **`cancel_settlement` rend un type propre à l'enveloppe** (validation P1, 34-8-F1) :
     `SupplierSettlementCancelled { invoice: SupplierInvoiceSettlementView, reversal_journal_entry_id: i64 }`. Elle est
     construite à partir de la `SupplierSettlementCancellation` que rend `cancel_settlement_in_tx`.
     - **`SupplierSettlementCancellation` ne change pas** : `cancel_settlement_in_tx` la construit (`supplier_invoices.rs:2941-2946`),
       et l'annulation d'un rapprochement fournisseur l'appelle (`reconciliation_cancel.rs:608`). Lui faire porter la vue
       ajouterait des lectures à ce chemin.
     - Le champ garde le nom `invoice` : `done.invoice.invoice.status` se lit de même sur la vue.
   - Les formes `_in_tx` ne changent pas, et ne lisent rien de plus.
   - Un test qui passe le retour de `pay`, `cancel`, `validate` ou `unvalidate` à une fonction typée `SupplierInvoiceWithLines`
     ne compile plus. Leur nombre n'est pas relevé : le compilateur les nomme, et chacun s'adapte (`.invoice`, `.lines`).
2. **AC-2 — routes.**
   - Les six handlers de `supplier_invoices.rs` et les trois de `imported_supplier_invoices.rs` construisent leur réponse par
     `SupplierInvoiceResponse::from_view` sur la vue rendue par l'écriture.
   - Les quatre fonctions `_once` lisent la vue dans leur transaction, avant leur `commit`, et la rendent.
   - `load_with_settlement_cancellation` est **retirée**, et ses mentions dans les doc-comments sont reprises (`git grep`).
3. **AC-3 — garde de structure** (C-e343-11). Le visiteur passe dans `crates/kesh-api/tests/common/`, avec ses limites écrites.
   - Pour les **neuf** handlers : aucun `.await` après l'appel d'écriture.
   - Pour les **cinq** enveloppes et les **quatre** fonctions `_once`, l'assertion qui mord est que la fonction **appelle
     `commit_with_view` et non `commit`** (`tx.commit()`). « Rien d'attendu après » est trivialement vrai de l'appel final.
     - Les enveloppes sont lues dans `crates/kesh-db/src/repositories/supplier_invoices.rs`.
     - Les quatre `_once` sont lus dans `crates/kesh-api/src/routes/` : `update_supplier_invoice_once`
       (`supplier_invoices.rs:805`), `complete_import_once`, `attach_source_document_once` et
       `detach_source_document_once` (`imported_supplier_invoices.rs:159`, `:607`, `:712`).
     - Les sources sont lues par `include_str!` dans le **fichier de test placé à `tests/`**, qui les passe au visiteur
       commun. C'est le patron de `invoice_settlement_response_e2e.rs:519`, chemins `../../kesh-db/src/…` et `../src/routes/…`.
       Un `include_str!` écrit dans `tests/common/` se résoudrait depuis ce dossier, et demanderait `../../../kesh-db/…`
       (validation P4, 34-8-F2).
   - Pour `commit_with_view` elle-même : la vue lue avant le commit, rien d'attendu après.
   - Une **sonde positive** passe par le même découpage :
     - une relecture remise après le commit dans un handler factice rend la garde rouge ;
     - une enveloppe factice qui appelle `tx.commit()` au lieu de `commit_with_view` la rend rouge aussi.

     *(Validation P3, 34-8-F4.)*
   - La garde du client (33-6) passe par le module commun, sans changer d'assertion.
4. **AC-4 — ce qui est rendu est ce qui est écrit.** Pour chacun des neuf gestes, la réponse HTTP égale, champ à champ, un
   `GET /supplier-invoices/{id}` qui la suit : `status`, `version`, `paidAt`, `settlementJournalEntryId`, `internalNumber`,
   lignes, `settlementCancellable`, `cancellable` et leurs motifs, `lastConfirmedBatch`, `sourceDocument`.
5. **AC-5 — pas de commit sans réponse.** Une lecture de vue en échec annule la transaction : c'est écrit au doc-comment de
   `commit_with_view`. Ce n'est pas reproductible sans crochet de production (voir la section suivante).
6. **AC-6 — les refus inchangés.** Les tests existants de ces neuf routes (statut, version, verrou de période, lot généré,
   document déjà traité…) passent sans modification d'assertion. Seules les constructions de type peuvent changer.
7. **AC-7 — le rejeu sur interblocage reste sûr.** `supplier_invoice_is_replayed_when_it_is_the_deadlock_victim` et
   `supplier_invoice_unvalidation_is_replayed_when_it_is_the_deadlock_victim` (`rejeu_interblocage_e2e.rs:821-969`) passent :
   la vue vient de la tentative qui a réussi.
8. **AC-8 — écran** (C-e343-12).
   - Sur les **sept** gestes de la fiche (payer, annuler le règlement, annuler, valider, repasser en brouillon, **joindre et
     détacher un document**), `isSettlementOutcomeUnknown(err)` (réponse perdue, 5xx, erreur hors `ApiError`) mène à une
     relecture.
     - `detach()` (`supplier-invoices/[id]/+page.svelte:522-553`) affiche aujourd'hui « Erreur inattendue » hors
       `ApiError`, sans relire.
     - `attach()` (`:492-520`) relance l'erreur au sélecteur, sans relire.
     - Un second envoi y est refusé (`409 IMPORT_NOT_PENDING_COMPLETION`, `404 SOURCE_DOCUMENT_NOT_FOUND`) : un échec
       affiché pour un geste réussi, le défaut de #938 (validation P2, 34-8-F1).
   - **Deux avis, selon la relecture** (validation P1, 34-8-F2). `reloadAfterRefusal` (`+page.svelte:477-483`) avale
     aujourd'hui l'échec : il rend désormais un booléen.
     - La relecture a réussi : « L'issue n'est pas connue : la facture a été relue, vérifiez-la avant de réessayer. »
     - La relecture a échoué : « L'issue n'est pas connue, et la facture n'a pas pu être relue : rechargez la page avant de
       réessayer. »

     L'écran n'affirme jamais une relecture qui n'a pas eu lieu. C'est le défaut que la revue P1 de la 33-6 a corrigé
     (`invoice-settle-unknown-reread-failed`).
   - Un 4xx garde son traitement actuel.
   - `pay()` n'est plus muet hors `ApiError`.
   - Pas d'état bloquant : un second envoi est refusé par le statut ou la version.
   - **L'état de l'avis** (validation P3, 34-8-F2) :
     - une variable `cycleOutcome: 'known' | 'verify' | 'reread-failed'` ;
     - l'avis est un **encadré persistant** sous les gestes, `data-testid="supplier-unknown-outcome"`, et non une
       notification ;
     - il disparaît au geste suivant (`clearCycleErrors`, qui le remet à `known`) ou à une relecture réussie lancée par la
       page ;
     - pour `attach()` : si la relecture montre le document joint, le sélecteur se ferme et l'encadré `verify` s'affiche. Si
       elle ne le montre pas, l'erreur remonte au sélecteur avec le texte de l'issue inconnue, et le document reste
       sélectionnable. `cycleOutcome` reste alors `known`, et l'encadré ne s'affiche pas : l'erreur est portée par le
       sélecteur (validation P4, 34-8-F4). Si la relecture échoue, `cycleOutcome` vaut `reread-failed`.
9. **AC-9 — textes.**
   - Les doc-comments C-30-2-42 et C-30-2-52 des trois routes de documents ne décrivent plus d'erreur entre le commit et la
     relecture.
   - Celui d'`isSettlementOutcomeUnknown` couvre la fiche fournisseur.
   - `docs/api-external.md:888` (joindre) dit « renvoyée après un succès dont la réponse s'est perdue (coupure réseau, ou
     erreur entre la validation et la relecture de la facture) ». La parenthèse perd sa seconde branche, et le conseil de
     relire reste pour la coupure réseau. `:890` (détacher) ne parle que de la réponse perdue : il est gardé tel quel, après
     vérification.
   - **Le manuel utilisateur gagne une phrase** dans *Régler une facture fournisseur* (`sec:regler-facture-fournisseur`,
     `docs/manual/fr/user-manual.tex:1923`). Elle dit que si la réponse se perd, la fiche relit la facture et le dit, et qu'il
     faut vérifier avant de réessayer. Seul le règlement client (`:1367`) décrit ce cas aujourd'hui. Le PDF est régénéré et
     contrôlé aplati (validation P1, 34-8-F3).
   - **Le périmètre « lecture seule » des services** (34-8-F5). `crates/kesh-api/src/services/mod.rs:162-186` compte la
     fermeture transitive des lectures des services, « cinquante-neuf éléments », en nommant `get_settlement_view`.
     `get_settlement_view_in_tx` y entre : le nombre est recompté à la source et la prose mise à jour. La preuve
     d'exécution sur connexion `READ ONLY` (`services_rollout.rs`) doit rester verte, ce que tient la règle « aucune lecture
     verrouillante ».
   - CHANGELOG *Corrigé* (`refs #938`).

## Ce qui ne se reproduit pas, et ce qui le remplace

Comme à la 33-6 (C-e334-10), le 404 ou le 500 **entre** le commit et la relecture ne se provoque pas sans crochet de
production : la fenêtre est entre deux requêtes SQL. Les tests d'égalité (AC-4) passent **déjà** avant le correctif, et c'est
attendu : le comportement nominal ne change pas. **Le rouge d'avant vient de la garde de structure (AC-3)**, à l'exécution,
et de la compilation des tests `kesh-db` qui lisent un champ de la vue sur le retour de `pay` ou de `validate`
(`settlement_cancel_blocker`, par exemple). Les mutations prouvent que les égalités mordent.

## Tâches

- [ ] **T1 — rouges d'abord**, journalisés dans `kesh-gate-logs/e343/rouges.log` :
  - la garde de l'AC-3, rouge à l'exécution sur les neuf handlers ;
  - les tests `kesh-db` (`supplier_invoices_repository.rs` ou un fichier neuf) qui lisent `view.settlement_cancel_blocker` sur
    le retour de `pay`, rouges à la compilation ;
  - le Vitest de l'AC-8 pour `pay()` hors `ApiError`, rouge.
- [ ] **T2 — `kesh-db`** : `get_settlement_view_in_tx`, `commit_with_view` et les cinq enveloppes.
  - `validate` et `unvalidate` quittent `in_own_tx!`. La macro reste : elle a d'autres usages (`create_draft`,
    `create_validated`, `update_draft`, `delete_draft` ; six appels à `a75835db`, dont ces deux).
  - ⚠️ `kesh-db` est touché : **gate complet même en cours de boucle**.
- [ ] **T3 — routes** : les neuf handlers et les quatre `_once`. `load_with_settlement_cancellation` est retirée. Les comptes
  de `services_motif.rs` sont recomptés.
- [ ] **T4 — garde** : module commun, garde fournisseur, sonde positive, et la garde client rebranchée.
- [ ] **T5 — égalités AC-4** dans `crates/kesh-api/tests/supplier_invoice_response_e2e.rs` (neuf), un test par geste. Les
  montages de `supplier_invoice_draft_e2e.rs` et de `supplier_settlement_cancel_e2e.rs` sont à réemployer, sans copie
  (règle DRY : aide commune si deux fichiers en ont besoin).
- [ ] **T6 — écran** : AC-8, avec deux clés i18n, `supplier-invoices-unknown-outcome` et
  `supplier-invoices-unknown-outcome-reread-failed`, aux quatre locales, chacune avec un repli identique. Vitest :
  - un cas par geste, `attach` et `detach` compris : réponse perdue → relecture et avis ; 400 → message, sans relecture ;
  - un cas où la relecture échoue → second avis.
- [ ] **T7 — E2E** : un paiement dont la réponse est coupée. Le patron est `cutResponseAfterWrite`
  (`invoices-settle-unknown-outcome.spec.ts:72-80`) : `route.fetch()`, **puis** `route.abort("connectionreset")`. Un
  `abort()` seul n'enverrait pas la requête, et rien ne serait écrit (validation P1, 34-8-F7). La fiche relue montre `payée` et l'avis. Un second clic sur *Payer* n'est pas
  offert, puisque le statut a changé.
- [ ] **T8 — textes** (AC-9) et **captures** de l'avis, en thème clair, dans `kesh-gate-logs/e343/captures/`.

### Mutations nommées

- **M1** : dans `pay`, la vue lue **avant** `pay_in_tx` → rouge AC-4 (`status: open` rendu pour une facture payée).
- **M2** : dans `validate`, la vue lue avant `validate_in_tx` → rouge AC-4 (`internalNumber` absent, `version` d'avant).
- *Forme de M1 et M2* (validation P3, 34-8-F5) : `commit_with_view` lit toujours après le résultat. Le mutant appelle donc
  `get_settlement_view_in_tx` **avant** l'écriture, puis `tx.commit()` directement, en contournant `commit_with_view`. Ce
  mutant est exprimable. Il rougit aussi la garde de l'AC-3 (commit direct).
- **M3** : `load_with_settlement_cancellation` réintroduite dans `cancel_supplier_invoice` → rouge AC-3.
- **M4** : dans `attach_source_document_once`, la vue lue après le `commit` → rouge AC-3.
- **M5** : `commit_with_view` valide malgré une vue `None` → rouge du test `kesh-db` de `commit_with_view`. Le test écrit une
  ligne témoin dans la transaction et passe `view_id` rendant un identifiant de facture absent. Il attend `DbError::Invariant`, et le témoin
  absent après coup : la transaction est annulée.
- **M6** : la fiche relit seulement sur `ApiError` → rouge Vitest AC-8 (cas hors `ApiError`).

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db`, frontend entier, E2E complet au dernier commit de code. Pas de `sqlx` :
aucune migration, aucun schéma.

## Ce qui ne se traite pas ici

- **La création d'une facture fournisseur** (`POST /supplier-invoices`, page `supplier-invoices/new`) n'a ni version ni clé.
  Une réponse perdue suivie d'un second clic crée un **second brouillon**. Il n'y a pas d'écriture au journal, et le doublon
  se voit dans la liste, mais l'opération est enregistrée et annoncée en échec. **Issue demandée** (P2 proposé). Elle n'est
  pas traitée ici : autre nature, la même question que #951.
- **La complétion d'un document** (`POST /imported-supplier-invoices/{id}/complete`, page `supplier-invoices/new`,
  `save`, `:145-149`) : l'erreur remonte au formulaire, sans relecture. Après une réponse perdue, un second clic rend
  `409 IMPORT_NOT_PENDING_COMPLETION`, affiché comme un échec alors que la facture est créée. C'est le symptôme de #938, sur
  un autre écran. **Issue demandée** (validation P2, 34-8-F1).
- **La page d'édition** (`[id]/edit`) : son `PUT` porte la version. Après une réponse perdue, un second clic rend 409, et
  `describeSubmitError` invite à recharger (`edit/+page.svelte:22`). Pas de double écriture, mais un échec annoncé pour
  une modification enregistrée, sans relecture. **Issue demandée.** *(La fiche disait « rien à faire » : c'était juger la
  double écriture, non le symptôme.)*
- **La confirmation d'un lot de paiement** et **l'acceptation d'un rapprochement** : elles passent par les formes `_in_tx` et
  ne relisent pas par `load_with_settlement_cancellation`. Hors de #938.

## Recoupements

- `unknown-outcome.ts` : **34-5** en récrit aussi le doc-comment (C-e343-5). Fusion par union.
- `rejeu_interblocage_e2e.rs` : 34-5 et 34-7 y touchent d'autres tests.
- `routes/supplier_invoices.rs` et `imported_supplier_invoices.rs` : **34-9** (#969, l'import du dossier) touche
  `post_inbox_import`, dans le même fichier mais une autre fonction. Union.
- **34-11a1 et 34-11a2** (34-11a découpée) ne touchent pas `pay` côté serveur. La 34-11a1 range `:2010`, `:2268` et
  `invoice_settlements_write.rs:170` en classe C (`34-11a1-compte-sans-iban.md:76`). Le vrai recoupement est la fiche
  fournisseur, `supplier-invoices/[id]/+page.svelte:888`, que la 34-8 modifie à l'AC-8 :
  - la 34-11a1 y change le libellé du compte bancaire (S13) ;
  - la 34-11a2 y propose la carte au paiement (`34-11a2-carte-credit-creation.md:39`).

  Fusion par union. *(Rectifié à la validation P2, 34-8-F3 : le recoupement écrit à la P1 était faux.)*

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : **validation P1 (Sonnet) : 3 M, 5 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p1-e343.md`. Chaque prémisse a été relue au code :
  - **F1 (M)** : `supplier_invoices.rs:2941-2946` et `reconciliation_cancel.rs:608` lus. Un type propre à l'enveloppe,
    `SupplierSettlementCancelled` ; `SupplierSettlementCancellation` inchangée. C-e343-9 corrigé de même.
  - **F2 (M)** : `reloadAfterRefusal` (`:477-483`) avale l'échec. Deux avis, et un booléen rendu ; C-e343-12 corrigé.
  - **F3 (M)** : seul `user-manual.tex:1367` décrit le cas, côté client. La phrase est obligatoire dans
    `sec:regler-facture-fournisseur` (`:1923`).
  - **F4 (L)** : signature de `commit_with_view` écrite.
  - **F5 (L)** : `services/mod.rs` lu (« cinquante-neuf éléments », `get_settlement_view` nommée), recompte à l'AC-9.
  - **F6 (L)** : les comptes de `services_motif.rs` sont précisés, au conditionnel.
  - **F7 (L)** : `invoices-settle-unknown-outcome.spec.ts:72-80` lu, fetch puis abort.
  - **F8 (L)** : recoupement avec 34-11a.
  - **Hors du rapport** : la fiche disait que `in_own_tx!` pouvait n'avoir plus d'usage. Six appels sont relus, la macro
    reste.
- 2026-11-10 : **validation P2 (Opus) : 1 M, 4 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p2-e343.md`. Prémisses relues :
  - `attach` et `detach` de la fiche (`:492-553`) ;
  - `supplier-invoices/new/+page.svelte:145-149` et `edit/+page.svelte:22` ;
  - `34-11a1-compte-sans-iban.md:76` et `34-11a2-carte-credit-creation.md:39`.
  - **F1 (M), neuf** : `attach` et `detach` sont ajoutés à l'AC-8. L'inventaire part du symptôme (relecture absente après une
    réponse perdue) : la complétion et l'édition vont à « Ce qui ne se traite pas ici », une issue demandée chacune, en plus
    de la création.
  - **LOW** :
    - **F2** : `commit_with_view` générique sur l'erreur (C-e343-15).
    - **F3** : recoupement 34-11a récrit (34-11a1, 34-11a2, page `:888`).
    - **F4** : `pub`.
    - **F5** : 12 et 11, comptés.
- 2026-11-10 : **validation P3 (Sonnet) : 1 M, 4 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p3-e343.md`. Prémisses relues : `imported_supplier_invoices.rs:194,202` et
  `invoice_settlement_response_e2e.rs:519`.
  - **F1 (M), né de la remédiation P2** (C-e343-15) : `commit_with_view` prend une fermeture `view_id`, appelée sur `Ok`. Les
    sites corrigés : AC-1, AC-3, M5, C-e343-15. C-e343-9 est annoté « remplacée par C-e343-15 ».
  - **LOW** :
    - **F2** : état, encadré et `data-testid` de l'avis, cas d'`attach`.
    - **F3** : phrase *Story* scindée.
    - **F4** : la garde lit `kesh-db` par `include_str!`, et l'assertion qui mord est « `commit_with_view`, non `commit` »,
      avec sa sonde. Prémisse du rapport nuancée : la garde de la 33-6 lit déjà un fichier de `kesh-db`.
    - **F5** : forme des mutants M1 et M2.
- 2026-11-10 : **validation P4 ciblée (Haiku) : 2 L, remédiés** (Opus 5.5).
  - **F2** : le lieu des `include_str!` et l'emplacement des quatre `_once` sont écrits (`supplier_invoices.rs:805`,
    `imported_supplier_invoices.rs:159,607,712`).
  - **F4** : la valeur de `cycleOutcome` est donnée pour la branche « relecture sans document ».

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
