# Story 34-8 : une facture fournisseur n'est plus annoncée en échec après son commit (#938)

Status: done

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

- [x] **T1 — rouges d'abord**, journalisés dans `kesh-gate-logs/e343/rouges.log` :
  - la garde de l'AC-3, rouge à l'exécution sur les neuf handlers ;
  - les tests `kesh-db` (`supplier_invoices_repository.rs` ou un fichier neuf) qui lisent `view.settlement_cancel_blocker` sur
    le retour de `pay`, rouges à la compilation ;
  - le Vitest de l'AC-8 pour `pay()` hors `ApiError`, rouge.
- [x] **T2 — `kesh-db`** : `get_settlement_view_in_tx`, `commit_with_view` et les cinq enveloppes.
  - `validate` et `unvalidate` quittent `in_own_tx!`. La macro reste : elle a d'autres usages (`create_draft`,
    `create_validated`, `update_draft`, `delete_draft` ; six appels à `a75835db`, dont ces deux).
  - ⚠️ `kesh-db` est touché : **gate complet même en cours de boucle**.
- [x] **T3 — routes** : les neuf handlers et les quatre `_once`. `load_with_settlement_cancellation` est retirée. Les comptes
  de `services_motif.rs` sont recomptés.
- [x] **T4 — garde** : module commun, garde fournisseur, sonde positive, et la garde client rebranchée.
- [x] **T5 — égalités AC-4** dans `crates/kesh-api/tests/supplier_invoice_response_e2e.rs` (neuf), un test par geste. Les
  montages de `supplier_invoice_draft_e2e.rs` et de `supplier_settlement_cancel_e2e.rs` sont à réemployer, sans copie
  (règle DRY : aide commune si deux fichiers en ont besoin).
- [x] **T6 — écran** : AC-8, avec deux clés i18n, `supplier-invoices-unknown-outcome` et
  `supplier-invoices-unknown-outcome-reread-failed`, aux quatre locales, chacune avec un repli identique. Vitest :
  - un cas par geste, `attach` et `detach` compris : réponse perdue → relecture et avis ; 400 → message, sans relecture ;
  - un cas où la relecture échoue → second avis.
- [x] **T7 — E2E** : un paiement dont la réponse est coupée. Le patron est `cutResponseAfterWrite`
  (`invoices-settle-unknown-outcome.spec.ts:72-80`) : `route.fetch()`, **puis** `route.abort("connectionreset")`. Un
  `abort()` seul n'enverrait pas la requête, et rien ne serait écrit (validation P1, 34-8-F7). La fiche relue montre `payée` et l'avis. Un second clic sur *Payer* n'est pas
  offert, puisque le statut a changé.
- [x] **T8 — textes** (AC-9) et **captures** de l'avis, en thème clair, dans `kesh-gate-logs/e343/captures/`.

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
  se voit dans la liste, mais l'opération est enregistrée et annoncée en échec. **Issue demandée** (P2 proposé) : ouverte,
  **#975**. Elle n'est
  pas traitée ici : autre nature, la même question que #951.
- **La complétion d'un document** (`POST /imported-supplier-invoices/{id}/complete`, page `supplier-invoices/new`,
  `save`, `:145-149`) : l'erreur remonte au formulaire, sans relecture. Après une réponse perdue, un second clic rend
  `409 IMPORT_NOT_PENDING_COMPLETION`, affiché comme un échec alors que la facture est créée. C'est le symptôme de #938, sur
  un autre écran. **Issue demandée** (validation P2, 34-8-F1) : ouverte, **#985**.
- **La page d'édition** (`[id]/edit`) : son `PUT` porte la version. Après une réponse perdue, un second clic rend 409, et
  `describeSubmitError` invite à recharger (`edit/+page.svelte:22`). Pas de double écriture, mais un échec annoncé pour
  une modification enregistrée, sans relecture. **Issue demandée** : ouverte, **#986**. *(La fiche disait « rien à faire » : c'était juger la
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
- 2026-11-10/11 : **développement** (Opus 5.5, worktree `/home/gcorbaz/devel/kesh-e348`, base `665de151`). Détail au Dev
  Agent Record ci-dessous.
  - **Étape 0.** Validation P5 : F1 porte sur la 34-6, hors de cette fiche ; **F2** (lieu de l'état `reread-failed` de
    `attach`) tranché par C-e348-2 : le sélecteur se ferme, l'encadré dit de recharger (`+page.svelte`, `attach`, branche
    `if (!(await reloadAfterRefusal()))`). Points à établir : comptes de `services_motif.rs` relevés à la source,
    `grep -cE '^\s*(pub(\([a-z]+\))? )?async fn'` : `routes/supplier_invoices.rs` **12**, `^pub async fn` **11** (13 → 12,
    `services_motif.rs` mis à jour) ; `routes/imported_supplier_invoices.rs` **12** et **9**, inchangés.
  - **Écarts à la consigne « Edit seulement »** : (1) le bloc de trois tests ajouté en fin de
    `crates/kesh-db/tests/supplier_invoices_repository.rs` l'a été par un `cat >>` (ajout, sans remplacement), par habitude ;
    le reste du code par l'outil Edit ou Write ; (2) `cargo fmt --all`, le formateur, a reformaté les fichiers touchés ; (3)
    les mutations par un script de remplacement à motif unique (`kesh-gate-logs/e348/mut/appliquer.py`), restaurées par
    `git checkout --` puis `touch`, comme la consigne le permet ; (4) pour le rouge Vitest, la fiche corrigée a été mise de
    côté par `cp`, la fiche de base restaurée par `git checkout --`, puis la copie remise par `cp`. L'outil Edit n'a manqué
    de rien : la suppression du bloc de 117 lignes déplacé vers `common/autour_appel.rs` s'y est faite d'un tenant.
- 2026-11-11 : **revue de code P1 (Sonnet, trois lentilles) : 2 MEDIUM, 7 LOW — tous traités** (Opus 5.5, commits `c7251f56`
  et `69855016`). Rapports `kesh-gate-logs/e348-review-p1-{B,E,A}.md`. B : 1 M, 3 L ; E : 2 L ; A : 1 M, 2 L. Le cœur
  Rust (`commit_with_view`, les enveloppes, les handlers) n'a rien rendu au-dessus de LOW. Prémisses relues au code :
  - **B-1 (M), recoupé par E-1 et A-2** — « la facture relue » dans `docs/api-external.md` (le tableau des routes de la
    0.17.0, le paragraphe des champs de lecture, `cancel`, la complétion, `sourceDocument`, le rattachement, le
    détachement) et au doc-comment de `update_supplier_invoice`. Le paragraphe des champs de lecture promettait « l'état
    courant — celui d'une écriture concurrente éventuelle » : récrit (la facture telle que le geste l'a écrite ; drapeaux
    d'annulation et dernier lot indicatifs ; seul `GET` décrit l'état courant), et la liste des gestes qui portent
    `settlementCancellable` passe aux neuf. Symptôme grepé (`git grep -n -iE "\brelue|relecture"`) : les doc-comments des
    clients `supplier-invoices.api.ts` (quatre), `imported-supplier-invoices.api.ts` (deux),
    `imported-supplier-invoices.types.ts` et deux de la fiche Svelte, récrits. Laissés tels quels : la relecture **de
    l'écran** (la fiche relit vraiment par `GET`), les relectures **dans** la transaction (`cancel_settlement_in_tx`,
    `SupplierSettlementCancellation`), les tests et textes de la 25-3-c (historiques), les deux avis de l'écran.
  - **A-1 (M)** — deux captures neuves par la spec E2E : `e348-relecture-en-echec.png` (paiement coupé, puis relecture en
    `500` : encadré « rechargez la page ») et `e348-selecteur-issue-inconnue.png` (rattachement coupé avant le serveur :
    le sélecteur garde l'issue inconnue, sans encadré). Premier jet : la relecture était coupée par `abort`, que le client
    rejoue (`NETWORK_ERROR`, réessai d'environ 14 s) ; l'E2E a rougi, d'où le `500`.
  - **B-2 (L)** — le repli `common-error` de `pay()`, inatteignable, retiré ; `sitesTotal` **2925 → 2924**, annoté.
  - **B-3 (L)** — test `commit_with_view_rolls_back_when_the_view_cannot_be_read` et mutation **M7** (le rollback du bras
    « lecture en échec » remplacé par un commit). ⚠️ **Le premier montage était faux, et la première mesure de M7 vide** :
    retirer l'écriture d'achat est refusé par `chk_supplier_invoices_entry_iff_not_draft`, si bien que le test échouait au
    montage, mutation ou non (`mut/M7-montage-invalide.log`). C'est le gate ciblé suivant qui l'a montré. Second montage :
    l'écriture d'achat passe à une autre société dans la transaction (`NotFound` au rang 2 de
    `settlement_entry_cancel_blocker`). M7 rejouée : rouge sur le témoin (`left: 1`, `right: 0`).
  - **B-4 (L)** — l'hypothèse du cache des requêtes préparées (`SELECT *`) est écrite au doc-comment de
    `find_by_supplier_invoice_id_scoped`, avec ce qui la tient (`main.rs` migre avant de servir ;
    `git grep` : ni `pre_upgrade_backup`, ni `admin_backup`, ni `auto_backup.rs` n'appellent cette lecture ni la vue).
    Aucune issue demandée.
  - **A-3 (L)** — `attach_returns_what_was_written` et `detach_returns_what_was_written`, séparés (aide `invoice_and_document`).
  - **E-2 (L)** — observation (un 5xx certain se lit « issue inconnue »), choix C-e343-12 : rien à faire.
  - **Tendance** : P1 2 M / 7 L → remédiation. Les deux défauts nés de la remédiation (montage de B-3, coupure de A-1) ont été
    vus par le gate, non par une revue : la **P2 complète (Opus)** relit d'abord `c7251f56` et `69855016`, prompt
    `34-8-fournisseur-relecture-apres-commit-review-prompt-p2.md`.
  - Gates au commit `69855016`, voir *Gates de la remédiation P1* au Dev Agent Record.
- 2026-11-11 : **revue de code P2 (Opus, trois lentilles) : 0 C, 0 H, 2 MEDIUM distincts, 9 LOW — tous traités** (Opus 5.5,
  commit de code `3f5d36c4`). Rapports `kesh-gate-logs/e348-review-p2-{B,E,A}.md`. Prémisses relues au code :
  - **E-1 (M)** — après une réponse perdue puis une relecture en échec, le second clic sur *Payer*, *Annuler le règlement*
    ou *Annuler* recevait un refus de statut affiché en rouge sur la fiche d'avant, sans relecture : un geste écrit annoncé
    en échec. Codes relus (`crates/kesh-db/src/errors.rs` : `SUPPLIER_INVOICE_NOT_PAID`, `SUPPLIER_INVOICE_CANCELLED`,
    `SUPPLIER_INVOICE_DRAFT` ; `ILLEGAL_STATE_TRANSITION` pour `pay_in_tx`). Retenu C-e348-7 : `reloadAfterStaleRefusal`,
    `isStaleState` étendu, encadré `reread-failed` si la relecture échoue encore ; employé aussi par valider et repasser en
    brouillon. Vitest : les trois gestes, la relecture encore en échec ; mutations **M8** (payer sans relecture : 2 rouges)
    et **M9** (l'aide sans effet : 4 rouges, les trois gestes).
  - **B-1 = A-1 = E-3 (M)** — l'historique écrit à `docs/api-external.md:609` par la remédiation P1 était faux : à `665de151`,
    les neuf handlers passaient déjà par `load_with_settlement_cancellation` (`git show 665de151:…`, relu). L'incise
    est retirée ; `:623` passe aux neuf gestes ; le doc-comment de `supplier-invoices.types.ts` aligné ; `:884` vérifié
    (exact : `PUT`, `validate`, `unvalidate` depuis la v0.17.0). Symptôme grepé
    (`git grep -n -iE "pay. et l'annulation|jusque-là .pay"`) : plus rien.
  - **LOW** :
    - **E-2** — relecture sans rejeu (C-e348-8) : `apiClient.get` et `getSupplierInvoice` prennent les options d'appel ;
      Vitest `retry: false`.
    - **B-2 = A-2** — les quatre autres replis `common-error` inatteignables retirés (`cancelSettlement`, `cancel`,
      `confirmValidate`, `detach`) et le `else` mort de `confirmUnvalidate` ; `sitesTotal` **2924 → 2921** (−4, puis +1
      pour B-4), recompté par la garde. `confirmDelete` garde le sien : il ne passe pas par l'issue inconnue.
    - **B-3** — la règle du cache des requêtes préparées passe au doc-comment du module `imported_supplier_invoices.rs`
      (« aucune requête `SELECT *` sur le pool avant `MIGRATOR.run` »), la fonction y renvoie.
    - **B-4** — « gardé par le statut, la version ou l'état du document » (`unknown-outcome.ts`, la fiche). Le faux « déjà
      traité » sur son propre rattachement était court : corrigé ici (C-e348-9), Vitest dédié ; pas d'issue.
    - **A-3** — File List et décomptes au Dev Agent Record (ci-dessous).
    - **A-4** — #975, #985 et #986 cités à « Ce qui ne se traite pas ici ».
    - **A-5** — le bandeau jugé au Dev Agent Record ; issue demandée.
    - **E-2 de la P1** (observation) : rien.
  - Manuel : la phrase du second envoi dit que la fiche relit ; PDF régénéré (`make user`) et contrôlé aplati.
  - ⚠️ **Incident pendant la remédiation** : la première mesure de M8 a été jouée sur la remédiation **non commitée** ; le
    script de mutation restaure par `git checkout --`, qui a effacé les changements de `+page.svelte`. Ils ont été refaits
    à l'identique, vérifiés (Vitest 464/464, `check` 0 erreur), commités (`3f5d36c4`), puis M8 rejouée sur le code commité
    (`mut/M8.log` ; la première mesure est gardée dans `mut/M8-avant-commit.log`). Leçon : commiter avant toute mutation.
  - **Tendance** : P1 2 M / 7 L → P2 2 M / 9 L (distincts : E-1 vient de la story, B-1 de la remédiation P1). La P3
    complète (Sonnet) relit d'abord `3f5d36c4` : `34-8-fournisseur-relecture-apres-commit-review-prompt-p3.md`.
- 2026-11-11 : **revue de code P3 (Sonnet, trois lentilles) : 0 C, 0 H, 1 MEDIUM convergent (A-1 = B-1 = E-1, coté LOW par
  E), 9 LOW — traités** (Opus 5.5, commit de code `7840aab2`, tests E2E au commit suivant). Rapports
  `kesh-gate-logs/e348-review-p3-{B,E,A}.md`. Prémisses relues au code :
  - **A-1 = B-1 = E-1 (M)** — né de la remédiation P2 : `payError`, `settlementCancelError`, `cancelError` survivaient à la
    relecture, sous un bloc masqué, et reparaissaient au geste inverse. Relu : `grep -n "payError = ''"` ne rendait que le
    début de `pay()`. Retenu C-e348-10 : `clearBlockErrors`, appelée par `clearCycleErrors` (dont le commentaire devient
    vrai) et par toute relecture réussie. Vitest **en DOM visible** : second clic refusé, relecture « Payée », *Annuler le
    règlement*, le formulaire de paiement rendu sans refus. Mutation **M10** (`clearBlockErrors` sans effet) : rouge sur
    l'assertion du refus visible. **M10b** (le seul appel de la relecture retiré) **survit** : chaque geste qui rouvre un
    bloc passe par `clearCycleErrors` ; seul un bloc rouvert par une relecture, statut changé ailleurs, le distinguerait.
  - **LOW** :
    - **B-2** — l'encadré n'est plus effacé avant la confirmation de *Détacher* ni avant le contrôle des champs de *Payer*
      (`clearRefusals` au clic, l'encadré après). Le Vitest existant « Dévalider refusé, puis Payer : le refus disparaît dès
      le geste » (`supplier-invoice-draft-page.test.ts`) tient : il a rougi au premier jet, qui reportait tout l'effacement,
      d'où la séparation `clearRefusals` / encadré. Deux Vitest neufs.
    - **B-3 = E-2** — un troisième avis, `refused-reread-failed` (C-e348-11), clé `supplier-invoices-reread-failed` aux
      quatre locales.
    - **E-3** — pas de garde d'ordre ici, issue demandée (C-e348-12) *(ouverte : #1018)*.
    - **A-2** — Vitest de valider et repasser en brouillon (refus après relecture en échec, relecture, encadré) ; mutations
      **M11** et **M12** rouges.
    - **A-3** — CHANGELOG : la fiche relit sur le refus d'un second envoi ; la page de modification n'est pas couverte (#986).
    - **B-4** — même précision au manuel (phrase neuve, PDF régénéré) et au commentaire d'`isSettlementOutcomeUnknown`
      (#986, #985, #975).
    - **A-4** — l'échec instable de `backup-restore-new-install.spec.ts:140` : issue KF demandée, à l'orchestrateur
      *(déjà tracé : #931, KF-056, où l'occurrence a été versée)*.
    - **A-5** — deux tests E2E neufs, avec leurs captures (`e348-refus-relecture-en-echec.png`,
      `e348-propre-document-joint.png`).
  - **Gardes i18n** : `sitesTotal` 2921 → **2922**, clés du domaine 409 → **410**.
  - Commit fait **avant** les mutations (leçon de la P2) ; aucune perte.
  - **Tendance** : P1 2 M / 7 L → P2 2 M / 9 L → P3 1 M / 9 L. Le MEDIUM de la P3 est né de la remédiation P2 : la P4 est
    une **passe ciblée** (Haiku) sur le commit `7840aab2` et les tests E2E qui suivent,
    `34-8-fournisseur-relecture-apres-commit-review-prompt-p4-ciblee.md`.
- 2026-11-12 : **M10b tuée** (commit de test `b01229f6`) : un refus de paiement certain, puis deux gestes de document dont
  la réponse se perd ; la première relecture montre la facture payée ailleurs, la seconde de nouveau validée, et le bloc
  de paiement rouvert par la relecture ne montre plus le refus. M10b rejouée : rouge sur le refus visible.
- 2026-11-12 : **passe ciblée P4 (Haiku) : 1 MEDIUM, 2 LOW — traités** (Opus 5.5, commit de code `c0ec8440`). Rapport
  `kesh-gate-logs/e348-review-p4-ciblee.md`. Le MEDIUM est confirmé au code par l'orchestrateur ; les prémisses relues :
  - **C-1 (M), régression de la remédiation P3** — `clearBlockErrors` sans condition dans `reloadAfterRefusal` effaçait le
    refus que `pay()`, `cancelSettlement()` ou `cancel()` venait de poser, dès que la relecture qui suit un refus d'écran
    périmé réussissait. Quand elle rendait le **même** statut (`OPTIMISTIC_LOCK_CONFLICT`, facture toujours validée), le
    bloc restait affiché sans message. Corrigé : les refus des blocs ne sont vidés que si le statut a changé
    (`statusBefore`). `validateError` et `unvalidateError`, rendus hors des blocs, n'étaient pas touchés et ne le sont
    pas : cohérent. Vitest « refus périmé relu au même statut » ; mutation **M13** (sans condition) rouge ; **M10c**
    (condition retirée) rouge sur le test de M10b.
  - **C-2 (L)** — `detachError` n'est plus vidé avant la confirmation. Vitest ; **M14** rouge sous sa forme juste (vider
    avant la confirmation, le défaut d'origine). Sa première forme, la ligne simplement retirée, survivait, et ne pouvait
    que survivre : sans vidage du tout, le test d'une boîte annulée ne rougit pas.
  - **C-3 (L)** — le commentaire de `clearCycleErrors` ne range plus *Payer* parmi ses gestes.
  - La P4 avait vu `+page.svelte` modifié pendant sa lecture : c'était la mutation M10b en cours. Le worktree était
    restauré (`git status` vide) avant la remédiation.
  - **Tendance** : P3 1 M / 9 L → P4 ciblée 1 M / 2 L (né de la remédiation P3). Passe ciblée **P5** (Sonnet) sur
    `c0ec8440` : `34-8-fournisseur-relecture-apres-commit-review-prompt-p5-ciblee.md`.
- 2026-11-12 : **passe ciblée P5 (Sonnet) : 0 au-dessus de LOW** (rapport `kesh-gate-logs/e348-review-p5-ciblee.md`) — la
  remédiation `c0ec8440` n'a rien cassé. La boucle se clôt, **sans toucher aucune ligne de production** :
  - **S-1 (L)** — après un refus périmé, une relecture qui change le statut fait basculer l'écran sans aucun avis : ouvert
    par l'orchestrateur, **#1019**.
  - **S-2 (L)** — la survie de la forme « suppression » de M14 n'était justifiée que pour un test. Ajouté (fichier de test
    seul, commit `67ffa45f`) : « refus de détachement, puis détachement réussi ⇒ le refus d'avant disparaît ». **M14s**
    (forme « suppression ») rejouée sur ce commit : rouge (`expected <p> to be null`), restaurée. `mut/SYNTHESE.log` corrigé
    (« survivait faute de test, non à juste titre »).
  - Vitest du fichier **36/36** ; Vitest complet et `npm run check` au commit `67ffa45f`, voir le Dev Agent Record.
- 2026-11-12 : **clôture de la boucle de revue de code.** Tendance P1 → P5 :

  | Passe | Modèle | Portée | Résultat | Remédiation |
  |---|---|---|---|---|
  | P1 | Sonnet | complète, trois lentilles | 0 C, 0 H, 2 M, 7 L | `c7251f56`, `69855016` |
  | P2 | Opus | complète, trois lentilles | 0 C, 0 H, 2 M, 9 L | `3f5d36c4` |
  | P3 | Sonnet | complète, trois lentilles | 0 C, 0 H, 1 M (convergent), 9 L | `7840aab2`, `cf5883e1`, `610f72c4` |
  | P4 | Haiku | ciblée, une lentille (`7840aab2`) | 0 C, 0 H, 1 M, 2 L | `b01229f6`, `c0ec8440` |
  | P5 | Sonnet | ciblée, une lentille (`c0ec8440`) | 0 au-dessus de LOW, 2 L | `67ffa45f` (test seul) |

  Le motif connu s'est vérifié : chaque MEDIUM d'après la P1 est né de la remédiation précédente (P2 B-1 de la P1 ; P3 A-1
  de la P2 ; P4 C-1 de la P3), sauf P2 E-1, défaut de la story. La boucle se clôt sur une passe sans rien au-dessus de LOW,
  dont la remédiation ne touche que des tests. Mutations M1 à M14s : toutes rouges, sauf la forme « suppression » de M14,
  tuée à la P5 (`mut/SYNTHESE.log`). Issues ouvertes pendant la story : #975, #985, #986 (gestes hors de la fiche), #1017
  (bandeau de réessai), #1018 (ordre des relectures), #1019 (S-1) ; #931 (KF-056) a reçu une occurrence.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (claude-opus-5-5), en autonomie, sous le prompt `kesh-gate-logs/prompts/e34-dev-commun.md`.

### Completion Notes List

- **Rouges d'abord** (`kesh-gate-logs/e348/rouges.log`) :
  - (a) la garde AC-3 rouge à l'exécution : les **neuf** handlers nommés (« attendent encore après l'écriture »), les **neuf**
    fonctions qui écrivent (« n'appelle pas commit_with_view »), `commit_with_view` introuvable ; la sonde positive verte ;
  - (b) `kesh-db`, `supplier_invoices_repository` : **8** erreurs de compilation (`commit_with_view` absente ;
    `settlement_cancel_blocker` et `invoice_cancel_blocker` absents de `SupplierInvoiceWithLines`) ;
  - (c) Vitest AC-8 sur la fiche d'avant : **15** rouges sur 21. L'un d'eux (« repasser en brouillon : refus 400 ») venait du
    test, le repassage traduisant le code ; corrigé dans le test (`refusalText`), noté au journal.
- **Mutations** (`kesh-gate-logs/e348/mut/`, synthèse `SYNTHESE.log`), toutes rouges, toutes restaurées (0 ligne de diff) :
  - **M1** (`pay` lit la vue avant d'écrire, puis `tx.commit()`) : rouge AC-4, `pay_returns_what_was_written` (`left:
    "open"`, `right: "paid"`), `the_wrappers_return_the_view_of_what_they_wrote`, la garde (commit direct), et 21 autres
    tests `kesh-db` (le mutant `unwrap()` la vue d'une facture absente) ; 24 rouges sur 76 ;
  - **M2** (`validate`, même forme) : `validate_returns_what_was_written`, `unvalidate_returns_what_was_written` (version
    d'avant), `the_wrappers…`, la garde ; 4 sur 76 ;
  - **M3** (relecture remise après l'écriture dans `cancel_supplier_invoice`) : `the_nine_handlers_wait_for_nothing_after_the_write` ;
  - **M4** (`attach_source_document_once` : `commit` puis relecture) : `the_nine_writers_end_with_commit_with_view` ;
  - **M5** (`commit_with_view` valide malgré une vue `None`) : `commit_with_view_rolls_back_when_the_view_is_missing` ;
  - **M6** (la fiche ne relit que sur `ApiError`) : le Vitest « payer : une erreur hors `ApiError` n'est plus muette ».
- **Un rouge du gate complet, hors des fichiers de la story** : `migrations_upgrade_path`,
  `draft_migration_preserves_a_populated_{016,rc1}_base`, `ColumnNotFound("link_origin")`. Le test semait par `pay` et
  `cancel` sur le schéma d'avant la montée ; ces enveloppes lisent désormais la vue (`SELECT *` sur
  `imported_supplier_invoices`), dont la requête préparée restait en cache sur la connexion du pool, sans la colonne que
  la montée ajoute. Le semis passe désormais par `pay_in_tx` / `cancel_in_tx`, qui écrivent ce qu'écrivaient les
  enveloppes de la 0.16 sans rien lire de plus. **En production**, les migrations tournent au démarrage, et la sauvegarde
  d'avant migration ne lit pas `imported_supplier_invoices` par `query_as` (`git grep "SELECT \*"` : aucun site dans
  `admin_backup/` ni `pre_upgrade_backup.rs`) : le défaut est propre au test.
- **Gardes i18n recomptées** (Vitest complet) : `i18n-keys` 2922 → **2925** sites (les deux avis, le repli `common-error`
  de `pay()`), `i18n-un-repli-par-cle` 407 → **409** clés du domaine, `i18n-libelle-en-dur` 85 → **86** candidates
  (`unknownOutcomeText`, `conforme` 71 → 72).
- **Deux tests Vitest de la 30-2c** passent de 500 à 403 (C-e348-4) : un 5xx est désormais une issue inconnue.
- **Tests ajoutés**, comptés aux deux bornes (`665de151` → tête, `grep -cE '#\[(test|sqlx::test)'`) :
  `kesh-api/tests/supplier_invoice_response_e2e.rs` 0 → **12** (8 égalités AC-4, dont joindre et détacher dans un même
  test, et 4 gardes) ; `kesh-db/tests/supplier_invoices_repository.rs` 61 → **64** ;
  `invoice_settlement_response_e2e.rs` 10 → 10 (garde déplacée, assertions inchangées) ; Vitest
  `supplier-unknown-outcome-page.test.ts` neuf, **21** tests (7 gestes × 2, plus 7).
- **Manuel** : `user-manual.tex`, *Régler une facture fournisseur*, paragraphe « Si la réponse se perd » ; PDF régénéré
  (`make fr`, `user-manual.pdf` seul gardé : `admin-manual.pdf` et `marketing-brochure.pdf`, sans changement de source,
  restaurés), contrôlé aplati : `pdftotext … | tr '\n' ' ' | tr -s ' ' | grep -o "Si la réponse se perd…"` rend le
  paragraphe entier.
- **Gates** — voir la section *Gates* ci-dessous, tels qu'ils ont tourné.

### Gates

- Gate ciblé (pendant la story) : 11 binaires (`supplier_invoices_repository`, `supplier_invoices_draft`,
  `supplier_invoices_internal_number`, `supplier_invoice_response_e2e`, `invoice_settlement_response_e2e`, `services_motif`,
  `supplier_invoice_draft_e2e`, `supplier_settlement_cancel_e2e`, `rejeu_interblocage_e2e`, `services_rollout`,
  `supplier_document_attachment`) : **283/283**, dont les deux rejeux sur interblocage fournisseur (AC-7).
- `cargo fmt --check` vert ; `cargo clippy --workspace --all-targets -D warnings` vert (au commit `2b5d7b2d`).
- `scripts/test-fast.sh --ci`, bases remises à neuf : au commit `2b5d7b2d`, 5151/5153, 2 rouges (`migrations_upgrade_path`,
  ci-dessus) ; **au commit `afb47235` (dernier commit de code : `1ec10969`) : 5153 exécutés, 5153 passés, 12 ignorés**.
- Frontend au commit `2b5d7b2d` : `npm run check` 0 erreur, `lint-i18n-ownership` vert, `test:unit` 2898/2902 (les 4 rouges
  des gardes i18n, recomptées au commit `1ec10969` : les trois fichiers 56/56), `npm run build` vert.
- **Différés par la pause des commandes lourdes** (`kesh-gate-logs/PAUSE-HEAVY`, décision de Guy du 2026-11-11) : `cargo
  build --workspace --all-targets`, `scripts/gate-ci.sh kesh_ci_e348`, le gate frontend complet et l'E2E complet au dernier
  commit de code. Le build en attente du verrou a été arrêté avant d'avoir démarré ; tous ont tourné à la levée de la
  pause, au commit `59d64250` (dernier commit de code : `1ec10969`) :
  - `cargo build --workspace --all-targets` : vert ;
  - `scripts/gate-ci.sh kesh_ci_e348` (base de CI recréée, un processus par binaire) : **5160 passés, 0 échoué, 16 ignorés**
    (`kesh-gate-logs/e348/final-gate-ci.log`) ;
  - frontend : `npm run check` 0 erreur (29 avertissements, aucun sous `supplier-invoices`), `lint-i18n-ownership` vert,
    `test:unit` **2902/2902** (223 fichiers), `npm run build` vert ;
  - **E2E complet**, base `kesh_e2e_e348` recréée vide, backend sur 3348, `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` passés
    aussi au runner : **387 passés, 7 échecs, 17 ignorés** (17,7 min). Les sept échecs, nommés un par un contre
    `docs/testing.md` § « Les échecs attendus » : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
    `onboarding.spec.ts:57`, `:77`, `:150` — les sept de la KF-029 (#97). Aucun autre. La spec neuve
    `supplier-invoice-unknown-outcome.spec.ts` passe (`kesh-gate-logs/e348/e2e-r1/playwright.log`). Aucun backend ni port
    3348 restant (`ss-avant.log`, `ss-apres.log` vides).
- **Capture** (thème clair) : `kesh-gate-logs/e348/captures/e348-paiement-issue-inconnue.png`, l'encadré `verify` après un
  paiement coupé, la fiche relue « Payée », sans *Payer*. ⚠️ L'état `reread-failed` n'a **pas** de capture : il n'est
  exercé que par Vitest. Le dossier reçoit aussi les captures des autres specs, la variable `KESH_E2E_CAPTURES_DIR` valant
  pour toute la suite. *(Relevé de la livraison du 2026-11-11 ; complété par la remédiation P1, ci-dessous.)*

### Gates de la remédiation P1

- Au commit `c7251f56` (`gates-p1.out`) : gate ciblé 135/136, `test-fast` 5154/5155, `gate-ci` 4572 passés et 1 échec — le
  même test, `commit_with_view_rolls_back_when_the_view_cannot_be_read`, refusé au montage par
  `chk_supplier_invoices_entry_iff_not_draft` ; frontend vert (2902/2902) ; E2E 387 passés, **9 échecs** : les sept de la
  KF-029, la spec neuve `supplier-invoice-unknown-outcome.spec.ts:145` (relecture coupée, rejouée par le client) et
  `date-filtre-frappe.spec.ts:86` (`locator.pressSequentially: Target crashed`, plantage du navigateur ; repasse au run
  suivant, ci-dessous).
- **Au commit `69855016`, dernier commit de code** (`gates-p1b.out`, puis la voie frontend) :
  - gate ciblé (5 binaires) **136/136** ; `scripts/test-fast.sh --ci`, bases remises à neuf : **5155/5155**, 12 ignorés ;
  - `cargo build --workspace --all-targets` vert ; `scripts/gate-ci.sh kesh_ci_e348` : **5162 passés, 0 échoué, 16 ignorés** ;
  - frontend, par `.gate-front.lock` : `check` 0 erreur, `lint-i18n-ownership` vert, `test:unit` **2902/2902** (223
    fichiers), `build` vert. Ma commande `npm run check` avait d'abord été mise en file dans la voie base ; je l'ai arrêtée
    avant son démarrage pour la relancer dans la voie frontend ;
  - **E2E complet**, base `kesh_e2e_e348` recréée vide : **389 passés, 7 échecs, 17 ignorés**. Les sept échecs, nommés un par
    un : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`,
    ceux de la KF-029 (#97). Les trois tests de `supplier-invoice-unknown-outcome.spec.ts` passent. `ss` avant et après :
    vides.
- **Mutation M7** : rouge sur le témoin (`left: 1`, `right: 0`), restaurée (`mut/M7.log`).
- **Captures** (thème clair), `kesh-gate-logs/e348/captures/` :
  - `e348-paiement-issue-inconnue.png` : encadré « vérifiez », fiche relue « Payée » ;
  - `e348-relecture-en-echec.png` : encadré « rechargez la page ». La fiche reste celle d'avant (« Validée », *Payer*
    offert), et le bandeau « Base de données temporairement indisponible » vient de la coupure du `POST`. Un second clic
    serait refusé (statut changé) : pas d'état bloquant, C-e343-12 ;
  - `e348-selecteur-issue-inconnue.png` : le sélecteur reste ouvert sur l'issue inconnue, sans encadré.

### Gates de la remédiation P2

- **Au commit `3f5d36c4`, dernier commit de code** (`gates-p2.out`) :
  - frontend, par `.gate-front.lock` : `check` 0 erreur, `lint-i18n-ownership` vert, `test:unit` **2908/2908** (223
    fichiers), `build` vert ;
  - `scripts/test-fast.sh --ci`, bases remises à neuf : **5155/5155**, 12 ignorés ; `cargo build --workspace --all-targets`
    vert ; `scripts/gate-ci.sh kesh_ci_e348` : **5162 passés, 0 échoué, 16 ignorés** ;
  - **E2E complet**, base recréée vide : **388 passés, 8 échecs, 17 ignorés** (20,8 min). Sept échecs sont ceux de la
    KF-029 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
    `:150`. Le huitième, `backup-restore-new-install.spec.ts:140` (`locator.click: Test timeout of 180000ms exceeded`, page
    de restauration de l'assistant), est hors de la story ; **rejouée seule** sur une base recréée (`e2e-r5/`), la spec
    passe entière (3/3) : un échec qui change d'identité d'un run à l'autre, au sens de `docs/testing.md`, non une
    régression. Les trois tests de
    `supplier-invoice-unknown-outcome.spec.ts` passent ;
  - mutations **M8** et **M9** rouges, restaurées (`mut/SYNTHESE.log`). Le manuel régénéré (`make user`), contrôlé aplati.
- **Décomptes des tests**, aux deux bornes (`665de151` → `3f5d36c4`, `grep -cE '#\[(test|sqlx::test)'`) :
  `kesh-api/tests/supplier_invoice_response_e2e.rs` 0 → **13** (9 égalités AC-4, une par geste, et 4 gardes) ;
  `kesh-db/tests/supplier_invoices_repository.rs` 61 → **65** ; Vitest `supplier-unknown-outcome-page.test.ts` **27**
  (21 de la livraison, 6 de la P2) ; E2E `supplier-invoice-unknown-outcome.spec.ts` **3**. Totaux : nextest 5153 → 5155,
  Vitest 2902 → 2908.
- **A-5, le bandeau de la capture `e348-relecture-en-echec.png`** : « Base de données temporairement indisponible —
  réessai automatique en cours » vient du client commun, qui pose l'état dégradé au premier échec réseau, même d'un `POST`
  qu'il ne rejoue pas (`api-client.ts`, `setDegraded()`), et ne le lève qu'au premier succès. Sur cette capture, il dit
  « réessai automatique » alors que rien n'est réessayé, à côté de l'encadré qui dit de recharger : l'écran se contredit.
  Antérieur à la story et commun à tout l'écran : **issue demandée** (une, P4 proposé), sans correction ici *(ouverte :
  #1017)*.

### Gates de la remédiation P3

- Le backend n'est pas touché par cette remédiation (aucun fichier Rust ; la clé neuve est dans les `.ftl`) : pas de gate
  backend, les derniers restant ceux de `3f5d36c4` (5155/5155 nextest, 5162 `gate-ci`).
- Au commit `cf5883e1` (`gates-p3.out`) : `check` 0 erreur, `lint-i18n-ownership` vert, `build` vert, `make user` vert ;
  `test:unit` 2912/2913 — la garde `e2e-selecteurs-traduits.test.ts` (KF-043) a relevé un `getByText('Document joint à la
  facture.')` dans la spec neuve : remplacé au commit `610f72c4` par `[data-sonner-toast][data-type="success"]`. E2E
  **inexploitable** : 132 échecs, `seedTestState … 500`, le backend journalisant 128 `pool timed out while waiting for an
  open connection` dès 23 h 08, sur des specs sans rapport (la suite entière), alors que la station tournait plusieurs
  gates. Non retenu comme mesure.
- **Au commit `610f72c4`, dernier commit de code** : `test:unit` **2913/2913** (223 fichiers) ; **E2E complet**, base
  recréée vide : **391 passés, 7 échecs, 17 ignorés** (17,7 min), aucun `pool timed out`. Les sept échecs sont ceux de la
  KF-029 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`.
  Les cinq tests de `supplier-invoice-unknown-outcome.spec.ts` passent. `ss` avant et après : vides.
- Mutations **M10**, **M11**, **M12** rouges, **M10b** survit (justifiée au Change Log et à `mut/SYNTHESE.log`).
- **Captures neuves** (thème clair) : `e348-refus-relecture-en-echec.png` — l'encadré « La facture n'a pas pu être relue :
  rechargez la page avant de réessayer. » au-dessus du refus « Transition d'état interdite » du second paiement, fiche
  restée « Validée » (le bandeau de réessai, voir A-5 ci-dessus) ; `e348-propre-document-joint.png` — le document joint
  affiché, notification « Document joint à la facture. ».
- Décomptes : Vitest `supplier-unknown-outcome-page.test.ts` **32** (27 + 5 de la P3) ; E2E de la story **5** ; Vitest total
  2908 → 2913.

### Gates de la remédiation P4 ciblée

- **Au commit `c0ec8440`, dernier commit de code** (`gates-p4.out`), demandés aussi parce que les `.ftl` changés à la P3
  sont lus par des tests Rust d'autres crates :
  - frontend, par `.gate-front.lock` : `check` 0 erreur, `lint-i18n-ownership` vert, `test:unit` **2916/2916** (223
    fichiers), `build` vert ;
  - `scripts/test-fast.sh --ci`, bases remises à neuf : **5155/5155**, 12 ignorés ; `scripts/gate-ci.sh kesh_ci_e348` :
    **5162 passés, 0 échoué, 16 ignorés** ;
  - **E2E complet**, base recréée vide : **391 passés, 7 échecs, 17 ignorés** (17,9 min), aucun `pool timed out`. Les
    sept échecs sont ceux de la KF-029 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
    `onboarding.spec.ts:57`, `:77`, `:150`. `ss` avant et après : vides.
- Le gate backend lancé d'abord au commit `610f72c4`, encore en attente du verrou, a été arrêté avant son démarrage et
  rejoué ici.
- Mutations **M10b** (rejouée), **M10c**, **M13**, **M14** rouges ; tout le relevé dans `mut/SYNTHESE.log`.
- Décomptes : Vitest `supplier-unknown-outcome-page.test.ts` **35** (32 + M10b + C-1 + C-2) ; Vitest total 2913 → 2916.

### Gates de la clôture (revue P5 ciblée)

- Au commit `67ffa45f` (test seul : aucune ligne de production depuis `c0ec8440`, dont les gates backend et E2E
  valent) : `npm run check` 0 erreur ; Vitest du fichier **36/36** ; Vitest complet **2917/2917** (223 fichiers), par
  `.gate-front.lock`. Mutation **M14s** rouge, restaurée.
- Nettoyage : bases `kesh_e348`, `kesh_e2e_e348`, `kesh_ci_e348` supprimées (story `done`) ; aucun backend, aucun verrou ;
  le worktree est gardé.

### File List

- `crates/kesh-db/src/repositories/supplier_invoices.rs` — `get_settlement_view_in_tx`, `commit_with_view`,
  `SupplierSettlementCancelled`, les cinq enveloppes.
- `crates/kesh-api/src/routes/supplier_invoices.rs`, `crates/kesh-api/src/routes/imported_supplier_invoices.rs` — les neuf
  handlers, les quatre `_once`, `load_with_settlement_cancellation` retirée, doc-comments C-30-2-42 et C-30-2-52 récrits.
- `crates/kesh-api/src/services/mod.rs` — périmètre « lecture seule », soixante éléments.
- `crates/kesh-api/tests/common/autour_appel.rs` (neuf), `crates/kesh-api/tests/supplier_invoice_response_e2e.rs` (neuf),
  `crates/kesh-api/tests/invoice_settlement_response_e2e.rs`, `crates/kesh-api/tests/services_motif.rs`,
  `crates/kesh-api/tests/supplier_invoice_draft_e2e.rs`.
- `crates/kesh-db/tests/supplier_invoices_repository.rs`, `supplier_invoices_draft.rs`,
  `supplier_invoices_internal_number.rs`, `migrations_upgrade_path.rs`, `support/fournisseur_rapproche.rs`.
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` — deux clés.
- `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`, `supplier-unknown-outcome-page.test.ts` (neuf),
  `supplier-document-page.test.ts` ; `frontend/src/lib/shared/utils/unknown-outcome.ts` ;
  `frontend/src/lib/shared/{i18n-keys,i18n-un-repli-par-cle,i18n-libelle-en-dur}.test.ts` ;
  `frontend/tests/e2e/supplier-invoice-unknown-outcome.spec.ts` (neuf).
- `docs/manual/fr/user-manual.tex`, `user-manual.pdf` ; `docs/api-external.md` ; `CHANGELOG.md`.
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e348.md` (neuf),
  `34-8-fournisseur-relecture-apres-commit-review-prompt-p1.md` (neuf).
- *Ajoutés par les remédiations P1 et P2 (revue P2, A-3)* : `crates/kesh-db/src/repositories/imported_supplier_invoices.rs`
  (règle du cache, au module) ; `frontend/src/lib/features/supplier-invoices/supplier-invoices.api.ts` et
  `supplier-invoices.types.ts` ; `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.api.ts`
  et `.types.ts` ; `frontend/src/lib/shared/utils/api-client.ts` (`get` et ses options d'appel) ;
  `_bmad-output/implementation-artifacts/sprint-status.yaml` ;
  `34-8-fournisseur-relecture-apres-commit-review-prompt-p2.md` et `…-review-prompt-p3.md` (neufs).
