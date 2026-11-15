# Story 38-6c : rattachement aux écritures existantes — en devise, en lot (2024) ; API des scripts externes

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`, numéros de ligne relevés sur `8b3d548d`. Epic 38
(`_bmad-output/planning-artifacts/epic-38-paypal.md`) : E3, E11, E15, E21, E22, E23, C-38-0, C-38-12, C-38-14 ;
décisions de Beat B17, B21 (`epics-38-40-cadrage.md`). **Dépend de 38-3** (colonnes `provider_tx_id`, `invoice_ref`
de `bank_transactions`, C-38-4 ; comptes prestataires porteurs de transactions) et, à travers elle, de **38-2** (compte
bancaire lié à un compte du plan **en devise**, levée de C-37-9, E11) et de l'**Epic 37** (37-1 : `accounts.currency` ;
37-3a : `journal_entry_lines.amount_currency` toujours positif, sens porté par débit/crédit, `chk_jel_fx_amount` ;
37-3b : conversion entre deux devises = **deux** écritures par le compte de conversion). Fiches sœurs : 38-6a (lève le
saut hors CHF de `get_proposals`, `confirmAlreadyBooked`), 38-9b (écran du rattachement en lot, manuel utilisateur et
voie API des scripts). **Aucune migration** (attendu par l'epic, confirmé : AC-9). Se développe de préférence **avant
38-6a** (epic, § *Stories*).*

## Story

En tant que comptable qui a déjà passé ses livres 2024 par l'API, je veux que Kesh me **suggère**, pour chaque
transaction PayPal téléchargée, l'écriture existante qui lui correspond — dans la devise du compte — et que je les
rattache **en lot**, sans qu'aucune écriture ne soit créée ni doublée ; et, en tant qu'auteur d'un script externe
(exports WHMCS, CB Paid Subscriptions), je veux retrouver la transaction de prestataire d'une facture, créer l'écriture
et la rattacher par une clé d'API, afin que mes encaissements soient comptés une fois et rapprochés.

## Exigences

Reprises de la fiche d'epic (§ 38-6c), complétées là où un AC l'exige.

- **FR**
  1. `FR-38-6c-1` — Une transaction en devise se rattache à une écriture existante sur le montant en devise. → AC-1.
  2. `FR-38-6c-2` — Kesh suggère, par transaction en attente, l'écriture univoque ou la liste ambiguë. → AC-4.
  3. `FR-38-6c-3` — Un lot de rattachements s'envoie en une requête (`POST /reconciliation/link-entries`). → AC-5.
  4. `FR-38-6c-4` — Une transaction de prestataire se cherche par identifiant du prestataire, numéro de facture,
     statut et période. → AC-6.
  5. `FR-38-6c-5` — Un script externe crée l'écriture et la rattache par une clé d'API `read-write`. → AC-7.
  6. `FR-38-6c-6` *(ajoutée par la fiche)* — Une écriture qui mouvemente deux comptes bancaires de devises
     différentes (conversion CHF ↔ devise en une écriture, retrait d'un solde PayPal en devise vers la banque en CHF)
     se rattache **des deux côtés**. → AC-2.
  7. `FR-38-6c-7` *(ajoutée par la validation P1)* — Le rattachement n'écrit aucune écriture : une écriture d'un
     exercice clos ou sous verrou de période se rattache, en lot comme à l'unité. → AC-11.
- **NFR**
  1. `NFR-38-6c-1` — Une seule règle de rattachement : lecture par page égale aux fonctions du dialogue, en devise.
     → AC-3.
  2. `NFR-38-6c-2` — *Pattern batch* ; rien de rattaché d'office. → AC-4, AC-5.
  3. `NFR-38-6c-3` — Une année de transactions se suggère en temps borné (mesure à la fiche). → AC-8.
  4. `NFR-38-6c-4` — Routes des connexions et des secrets toujours fermées aux clés d'API. → AC-7.

## Écarts à l'epic

1. **Une conversion entre deux devises étrangères n'est pas « une écriture qui touche deux comptes en devise ».**
   E21 écrit qu'une conversion « (deux transactions, une écriture qui touche deux comptes en devise) se rattache des
   deux côtés comme un virement entre ses comptes ». Or une écriture ne porte qu'**une** devise étrangère
   (`FX_MULTIPLE_CURRENCIES`, C-37-3 complété par R7) : une conversion EUR → USD est **deux écritures** par le compte
   de conversion en CHF (`POST /journal-entries/conversions`, 37-3b AC-2 : `entries` de longueur 2). Chaque
   transaction se rattache alors à **sa** écriture, sans « autre côté » : la règle actuelle suffit, une fois en devise
   (AC-1). En revanche, une écriture qui mouvemente un compte bancaire **CHF** et un compte bancaire **en devise** —
   conversion CHF ↔ EUR en une écriture, retrait d'un solde PayPal EUR vers la banque CHF — existe (une seule devise
   étrangère), et le second côté de la Story 34-11b la **refuse** : l'« autre côté » y est reconnu à un montant
   **opposé** (`bt.amount = -(M)`, `other_side_predicate`, `reconciliation_link.rs:698-710`), comparaison sans
   objet entre deux devises. D'où `FR-38-6c-6` et l'AC-2.
2. **Corps du lot** : C-38-14 donne `[{bankTransactionId, journalEntryId, version}]`. Le lot porte en plus le
   `bankAccountId` (un compte par lot), comme `POST /reconciliation/accept` (`routes/reconciliation.rs:1593-1600`,
   `body.bank_account_id`) : le pré-vol du compte (archivé, clos, non lié) se fait une fois, et les suggestions sont
   par compte.
3. **Le lot n'est pas une transaction à points de sauvegarde.** L'acceptation en lot tient un verrou et une
   transaction pour tout le lot (`accept_once`, `routes/reconciliation.rs:1822-1860`). Le rattachement **ne le peut
   pas** sans rouvrir le double rattachement : sa sérialisation repose sur une vue de lecture née **après** le
   dernier verrou de ligne (`link_entry_in_tx`, doc `reconciliation_link.rs:1100-1131` ; « toute lecture ordinaire
   déplacée avant l'étape 4.2 rouvre le double rattachement (mutation M10) »). Dans une transaction unique, la vue
   naît à la première lecture du premier élément ; l'élément suivant lirait un instantané antérieur à son propre
   verrou. Chaque élément est donc **une tentative indépendante** du geste unitaire (sa transaction, son verrou nommé,
   son rejeu sur interblocage) — AC-5.
4. **Codes du lot et code du geste unitaire.** Le lot rend les codes de C-38-14 (`ENTRY_NOT_LINKABLE`,
   `ENTRY_ALREADY_LINKED`, `AMOUNT_MISMATCH`, `VERSION_CONFLICT`) ; `POST …/link-entry` **garde** son `409
   RECONCILIATION_ENTRY_NOT_LINKABLE` (`errors.rs:3241`) — changer le code d'une route publiée casserait ses clients.
   Le diagnostic qui distingue les motifs est une lecture de plus, après le refus de la règle, qui **étiquette** et
   n'admet jamais (AC-5).
5. **Écritures 2024 « au brut, frais à part ».** Une transaction se rattache à **une** écriture dont le mouvement sur
   le compte prestataire égale son montant (net, E7). Si Beat a passé en 2024 une vente au brut (débit PayPal 100) et
   ses frais par une **autre** écriture (crédit PayPal 3.20), aucune écriture ne vaut 96.80 : la transaction reste
   « sans correspondance » et se traite à la main. Si les frais sont une ligne de la même écriture (débit PayPal 96.80,
   débit frais 3.20, crédit ventes 100), elle se rattache. **À vérifier sur les écritures de la recette de l'Epic 37
   avant le développement** (question Q-38-6c-1 ci-dessous) ; aucune règle « somme de plusieurs écritures » n'est
   introduite (elle ne se décide pas sans faux rattachement possible : *Un appariement automatique propose, il ne crée
   jamais*).
6. **Écran et manuel** : le miroir de la condition 1 à l'écran (`canLink`,
   `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:262-264`, « CHF seulement ») et le manuel
   (`docs/manual/fr/user-manual.tex:3088`, `:3104`, § `sec:rattacher-reglement`) ne changent qu'en **38-9b** ; tant
   que 38-6a n'a pas levé le saut hors CHF de `get_proposals` (`routes/reconciliation.rs:1101-1104`), aucune
   transaction en devise n'apparaît à cet écran. Cette story liste au Change Log les sites à reprendre.

7. **Montants égaux : le lot ne tranche pas.** Deux transactions de même montant et de même devise (abonnements
   mensuels, ventes d'un même tarif) ont chacune **deux** écritures candidates : elles tombent en `ambiguous`
   (`several_entries`, `entry_shared`) et se traitent une à une au dialogue (38-9b). Aucun appariement « au plus
   proche » n'est ajouté (*Un appariement automatique propose, il ne crée jamais* : un faux rattachement est muet).
   Limite assumée, écrite au doc-comment de `entry_matches_for_page` et au manuel (38-9b).
8. **Le texte du refus unitaire devient faux.** `RECONCILIATION_ENTRY_NOT_LINKABLE` dit « la transaction n'est pas en
   CHF » (`errors.rs:3236`, doc-comment `:1100-1112`, `crates/kesh-i18n/locales/*/messages.ftl`, clé
   `reconciliation-errors-entry-not-linkable`, `fr-CH:1530`) : faux pour un compte en devise. Reformulé en AC-10 (le
   code et le statut ne changent pas).

## Questions ouvertes

- **Q-38-6c-1 (Beat)** — Comment les frais PayPal de 2024 ont-ils été passés : ligne de l'écriture de vente, ou
  écriture à part ? (Écart 5 ; décide si le rattachement en lot couvre l'essentiel de 2024.)

## Critères d'acceptation

1. **AC-1 — La règle en devise** (`FR-38-6c-1`). La règle du rattachement à une écriture (`reconciliation_link.rs`,
   doc `:480-524`, conditions 1 à 6) devient fonction d'une **base de mouvement** lue sur le compte du grand livre
   **actuel** du compte bancaire :
   - `MovementBasis::Chf` si ce compte n'a pas de devise (`accounts.currency IS NULL`, 37-1) — comportement actuel,
     **inchangé** ; `MovementBasis::Currency(code)` sinon (seul un compte prestataire peut être lié à un compte en
     devise, E11 — la règle ne lit pourtant que la devise du compte, non la table des réglages : un seul critère) ;
   - **condition 1** : `transaction_can_be_linked_to_entry(t, &basis)` (`:535-537`) — montant non nul **et**
     `t.currency` = `CHF` (base `Chf`) ou `code` (base `Currency`) ; seule définition, toujours pure ;
   - **condition 3** : le mouvement `M` d'une écriture sur ce compte se somme **en devise** pour la base `Currency` :
     `SUM(CASE WHEN l.debit > 0 THEN l.amount_currency ELSE -l.amount_currency END)` (37-3a : `amount_currency > 0`
     hors réévaluation, `= 0` pour une ligne de réévaluation, qui ne compte donc pas — `chk_jel_fx_amount`) ; base
     `Chf` : `SUM(l.debit) - SUM(l.credit)`, inchangé. `ENTRY_MOVEMENTS` (`:800-804`) devient une fonction
     `entry_movements(basis)` ; `entry_sql(mode)` (`:629-656`) devient `entry_sql(mode, basis)` ; `M = t.amount` reste
     une égalité décimale exacte (`DECIMAL(19,4)` contre le montant de la transaction) ;
   - conditions 2, 4 (sauf AC-2), 5, 6 : inchangées ; les règlements de factures (29-8, `find_link_candidates`,
     `transaction_can_be_linked`, `:90`) restent **CHF seulement** (les factures sont en CHF, D7 de l'Epic 37) — pour
     une base `Currency`, `settlement_count` vaut 0 sans requête ;
   - **qui lit la base** : `link_preflight` (`routes/reconciliation.rs:6228-6245`) rend `(compte du grand livre,
     base)` — une lecture de `accounts.currency` de plus, sur le pool, avec le pré-vol ; `LinkEntryRequest`
     (`reconciliation_link.rs:1060`) gagne `basis` ; `find_entry_candidates` (`:870`), `find_matched_elsewhere`
     (`:896`), `count_already_booked_for_page` (`:959`) et `link_entry_in_tx` (`:1156`) la reçoivent ; tous les
     appelants (`get_entry_candidates` `:6548`, `post_link_entry` `:6656`, `get_proposals` pour 34-4, `:1060-1095`) la
     passent. La devise d'un compte du grand livre qui porte des lignes est verrouillée (37-1, `ACCOUNT_CURRENCY_LOCKED`)
     : la base lue au pré-vol ne change pas sous le geste ; un compte sans ligne n'a aucune candidate.
   Tests (unitaires, `reconciliation_link.rs`) : les égalités de texte existantes
   (`the_restricted_entry_query_is_the_list_query_plus_the_entry`, `the_matched_elsewhere_query_is_the_list_query_plus_the_link`)
   tenues **pour chaque base** ; `transaction_can_be_linked_to_entry` : EUR sur base EUR vrai, CHF sur base EUR faux,
   EUR sur base CHF faux (le test actuel `:1313-1321`, gardé, devient le cas `Chf`). Tests HTTP
   (`reconciliation_link_entry_e2e.rs`, société multi-devises, compte bancaire lié à un compte EUR — montage de 38-2) :
   écriture EUR de mouvement +108.10 EUR (CHF quelconque) → candidate d'une transaction +108.10 EUR, rattachement 200,
   trace `reconciliation.entry_linked` (`amount` en devise) ; mouvement +108.10 **CHF** et 100.00 EUR → non candidate ;
   ligne de réévaluation sur le compte (montant devise 0) → `M` inchangé ; écriture contre-passée → non candidate
   (condition 5) ; compte CHF : tous les tests existants verts sans modification d'assertion.
2. **AC-2 — L'autre côté entre deux devises** (`FR-38-6c-6`, Écart 1). `other_side_predicate`
   (`reconciliation_link.rs:698-710`), **seule définition** de l'autre côté (liste, geste, page, annulation —
   `transfer_other_side` `:773-797`, lue par `reconciliation_cancel::entry_cancel_outlook`), reçoit en plus la devise
   de `t` : une transaction `bt` liée à l'écriture est l'autre côté de `t` si elle est d'un **autre** compte bancaire,
   hors écriture de règlement (inchangé), **et** :
   - même devise que `t` : `bt.amount = -(M)` — **inchangé** (34-11b) ;
   - devise différente : `SIGN(bt.amount) = -SIGN(M)` (sens contraire ; les montants ne se comparent pas).
   Le plafond « une transaction liée au plus » (`linkable_side`, `:739-748`) est inchangé : au plus deux transactions
   par écriture, une de chaque côté. ⚠️ Le commentaire de `linkable_side` dit le plafond `<= 1` **inerte** parce que
   deux liens ont des montants opposés (`:730-737`) : entre deux devises, ce raisonnement ne tient plus — une troisième
   transaction de sens contraire à `M` d'une troisième devise serait « l'autre côté » des deux premières selon le seul
   signe ; le plafond **devient porteur**, ce que le commentaire annonçait (« il deviendrait porteur le jour où le
   prédicat tolérerait un écart ») — un test de comportement le garde désormais (troisième lien refusé, mutation : le
   plafond retiré fait rougir ce test). Paramètres : l'ordre des `?` de `entry_sql`, de la lecture par page et de
   `transfer_other_side` est réécrit au doc-comment de chacun (la devise de `t` est **commune à une page**, toutes ses
   transactions liables étant dans la devise de la base, AC-1). Tests HTTP : écriture « débit 1030 PayPal CHF 100 /
   crédit 1031 PayPal EUR 92 EUR (CHF 100) » (une seule devise étrangère) : la transaction EUR −92 se rattache, puis
   la transaction CHF +100 s'y rattache aussi (second côté) ; même sens — écriture « débit PayPal CHF 100, débit PayPal EUR 92 EUR (CHF 100), crédit ventes 200 », transactions
   +100 CHF et +92 EUR : le premier rattachement passe, le second est refusé (`SIGN` identiques) ;
   troisième transaction → refusée ; annulation d'un côté → retrait du seul lien (`unlink_only` vrai à
   `entry-candidates`) ; écriture de retrait « débit 1020 UBS CHF / crédit PayPal EUR » rattachée des deux côtés
   (UBS en base `Chf`, PayPal en base `EUR`). Les tests de la 34-11b (`s34_11b_*`) restent verts.
3. **AC-3 — Une seule règle, lecture par page comprise** (`NFR-38-6c-1`). La lecture par page de la 34-4
   (`count_already_booked_for_page`, `:959-1056`) et les suggestions (AC-4) dérivent d'**une** fonction de page,
   `entry_matches_for_page(pool, company_id, bank_account_id, ledger_id, &basis, transactions, tolerance) ->
   HashMap<i64, PageMatches>` où `PageMatches { settlement_count, entries: Vec<PageEntry> }` et `PageEntry {
   journal_entry_id, entry_number, entry_date, description, amount, linked }`, triées comme la liste
   (jours d'écart à la date de paiement de `t` — `payment_date`, `repositories/reconciliation.rs:277` —, puis
   identifiant) et plafonnées à `ENTRY_CANDIDATES_LIMIT` (`:526`) : mêmes textes SQL que le dialogue
   (`entry_movements(basis)`, `linkable_side`, `HAS_LINK`, `ENTRY_NOT_REVERSED_NOR_SETTLEMENT`) ; la requête de page
   lit en plus `e.id`, `e.entry_number`, `e.entry_date`, `e.description`. `count_already_booked_for_page` en devient
   une projection (longueurs), de signature inchangée hormis `basis`. Le test d'égalité
   `s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue` (`reconciliation_link_entry_e2e.rs:2443`) est
   étendu : pour chaque transaction, les **identifiants et leur ordre** égalent ceux de `find_entry_candidates`, les
   liés ceux de `find_matched_elsewhere` — sur un compte CHF (cas actuels) **et** sur un compte EUR avec un cas de
   l'AC-2.
4. **AC-4 — Suggestions** (`FR-38-6c-2`, B17). `GET
   /api/v1/reconciliation/link-suggestions?bankAccountId=…&from=…&to=…&limit=…&offset=…` (Comptable+, comme
   `entry-candidates`, `lib.rs:921-925` ; clés d'API admises — une clé `read` lit, la méthode étant `GET`) :
   - **paramètres** : `bankAccountId` requis (> 0) ; `from`, `to` facultatifs (`AAAA-MM-JJ`, bornes **incluses** sur
     `booking_date`, `from ≤ to` sinon 400) ; `limit` 1..=500, défaut 200 (500 = `MAX_IN_CLAUSE_IDS`,
     `repositories/reconciliation.rs:1085`, borne de la clause `IN` des montants) ; `offset` ≥ 0 ;
   - **pré-vol** : `link_preflight` (compte absent ou archivé → 404 `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, clos → 409
     `BANK_ACCOUNT_CLOSED`, non lié → 409 `BANK_ACCOUNT_NOT_CONFIGURED` — codes existants du pré-vol) ;
   - **transactions** : en attente du compte (`status = 'pending'`, rejetées **comprises**, comme le geste unitaire —
     `rejected: true` dans l'élément), dans la fenêtre, triées `booking_date, id` croissants, paginées ; `total` = leur
     nombre ; index `idx_bank_transactions_pending (company_id, bank_account_id, status, booking_date)`
     (`20260504000001_bank_imports.sql:88`) ;
   - **classement**, par `entry_matches_for_page` (AC-3) sur la page :
     - `notLinkable` — condition 1 fausse (montant nul, autre devise que la base) ;
     - `unique` — **une** écriture candidate, **aucun** règlement candidat, l'écriture **non** liée ailleurs, et elle
       n'est candidate d'**aucune autre** transaction de la page ; l'élément porte `entry { journalEntryId,
       entryNumber, entryDate, description, amount }` ;
     - `ambiguous` — sinon, s'il y a au moins une candidate : `reasons` ⊂ {`several_entries`, `settlement_candidate`,
       `matched_elsewhere`, `entry_shared`} et `candidates` (les `PageEntry`, ≤ 50) ;
     - `none` — aucune écriture ni aucun règlement candidat ;
   - **réponse** `ListResponse` (`items`, `total`, `limit`, `offset`, `routes/mod.rs`) ; un élément : `bankTransactionId`,
     `version` (pour le lot), `bookingDate`, `valueDate`, `amount`, `currency`, `rejected`, `kind`, `entry?`,
     `candidates?`, `reasons?`, `settlementCount`, `matchedElsewhereCount` ;
   - **rien n'est écrit, rien n'est coché d'office** : la réponse ne propose aucun lot ; l'écran (38-9b) fait cocher.
   Limite écrite au doc-comment et au Change Log : `entry_shared` n'est vu que **dans la page** ; une écriture
   candidate univoque de deux transactions de deux pages différentes du même compte est refusée au second
   rattachement par la condition 4 (`ENTRY_ALREADY_LINKED`, AC-5) — jamais rattachée deux fois.
   Tests HTTP : chacun des quatre genres et des quatre motifs ; fenêtre et pagination (`total`, deux pages) ;
   transaction rejetée suggérée ; compte en devise ; Consultation → 403 ; clé `read` → 200 ; aucune écriture en base
   après l'appel (compte des lignes de `bank_transactions` à `reconciled` inchangé).
5. **AC-5 — Lot** (`FR-38-6c-3`, `NFR-38-6c-2`, *Pattern batch*). `POST /api/v1/reconciliation/link-entries`
   (Comptable+, clés d'API `read-write` admises — la clé `read` reçoit le 403 `API_KEY_READ_ONLY` du middleware,
   `middleware/auth.rs:141-147`) :
   - **corps** `{ bankAccountId, items: [{ bankTransactionId, journalEntryId, version }] }` ; extracteur
     `impl_validation_extractor` (`extractors.rs`) : tout rejet de corps → 400 `VALIDATION_ERROR` ; `items` vide, plus
     de 500 éléments (`check_batch_size`, `routes/reconciliation.rs:1554`), identifiant ≤ 0, `version` < 1,
     `bankTransactionId` en double → 400 ; un `journalEntryId` répété est admis (la règle refuse le second) ;
   - **erreurs globales** (seules admises, *Pattern batch*) : 400 ci-dessus, 401, 403, 404
     `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (compte absent ou archivé, comme `post_accept`, `:1700-1708`), 500 ;
   - **compte clos** → 200, chaque élément en `failed[]` `BANK_ACCOUNT_CLOSED` (`closed_account_failed`, `:6735`) ;
     **non lié** → chaque élément `BANK_ACCOUNT_NOT_CONFIGURED` (`details.bankAccountId`) ;
   - **pré-vol des transactions** (lecture, sans verrou, `find_by_ids_for_account`) : absente →
     `BANK_TRANSACTION_NOT_FOUND`, rapprochée → `RECONCILIATION_ALREADY_RECONCILED` (codes de
     `classify_batch_transaction`, `:1527-1547`, qui refuse en plus une transaction **rejetée** : le lot, lui,
     l'admet — fonction propre, ou paramètre, au développement, sans copier les codes) ;
   - **chaque élément éligible** = une tentative indépendante (Écart 3) de `link_entry_in_tx` sous
     `locked_link_attempt` (`:6411-6428`), rejouée sur interblocage (`retry_app_on_deadlock`, opération
     `reconciliation::link_entries`), **séquentielle** dans l'ordre reçu ; `LinkEntryRequest` gagne `expected_version:
     Option<i32>` (le geste unitaire passe `None`) et `LinkRefusal` (`reconciliation_link.rs:277-286`) une variante
     `VersionConflict { actual: i32 }`, contrôlée **après** `NotPending` et **avant** la condition 1, sous le verrou de
     la ligne (étape 4.1) — `link_refusal_error` (`routes/reconciliation.rs:6432-6445`) la rend en 409
     `OPTIMISTIC_LOCK_CONFLICT` (inatteignable par les routes unitaires) ;
   - **codes par élément** (constantes, `details` JSON) : `VERSION_CONFLICT` (`{ expected, actual }`) ;
     `RECONCILIATION_TRANSACTION_NOT_PENDING` (rapprochée entre le pré-vol et le verrou) ; `BANK_TRANSACTION_NOT_FOUND` ;
     `BANK_ACCOUNT_CLOSED` (clos entre-temps, étape 4.3) ; `RECONCILIATION_ACCOUNT_LOCKED` (délai du verrou nommé
     dépassé pour cet élément) ; et, sur `NotLinkable`, le **diagnostic** — une fonction de lecture
     `diagnose_entry_link(conn, company_id, &t, ledger_id, &basis, journal_entry_id) -> EntryLinkDiagnosis`, appelée
     dans la même transaction **après** le refus de la règle, qui ne fait qu'étiqueter :
     `ENTRY_NOT_LINKABLE` (`reason` ∈ `transaction_not_linkable` — condition 1 —, `entry_not_found`, `entry_reversed`
     — condition 5 —, `settlement_entry` — condition 6 —, `unclassified`) ; `AMOUNT_MISMATCH` (condition 3 :
     `{ transactionAmount, entryMovement }`, `entryMovement` `null` si l'écriture ne touche pas le compte) ;
     `ENTRY_ALREADY_LINKED` (condition 4 : `{ linkedBankTransactionIds }`) ; tout autre `DbError` →
     `DATABASE_ERROR` (`{ message }`, précédent d'`accept_one_rule`). Test : un refus de chaque classe ; et une
     propriété — pour toute écriture refusée par la règle, le diagnostic ne rend jamais « rattachable » (il n'a pas
     de variante pour cela) et `unclassified` n'apparaît dans aucun cas construit ;
   - **réponse** 200 `{ linked: [{ bankTransactionId, journalEntryId }], failed: [FailedProposal] }` — la struct
     `FailedProposal` existante (`routes/reconciliation.rs:571-576`), `failed` dans l'ordre des éléments reçus
     (`merge_failed_in_order`, `:1568`) ;
   - **audit** : chaque élément rattaché écrit `reconciliation.entry_linked` (dans `link_entry_in_tx`, existant ;
     acteur et clé d'API, `NewAuditLogEntry::for_actor`) ; rien pour un refus ; aucune action neuve.
   Tests HTTP : lot mixte (deux rattachés, un `VERSION_CONFLICT`, un `AMOUNT_MISMATCH`, un `ENTRY_ALREADY_LINKED`,
   un `BANK_TRANSACTION_NOT_FOUND`) → 200, ordre de `failed[]` ; même écriture deux fois dans le lot (même compte) → la
   seconde `ENTRY_ALREADY_LINKED` ; transaction rejetée → rattachée, `was_previously_rejected` vrai à la trace ; compte
   clos → tout en `failed[]` ; 501 éléments → 400 ; clé `read` → 403 ; Consultation → 403 ; concurrence : deux lots
   simultanés sur deux comptes visant la même écriture de virement au même côté (montage de T-race-3,
   `reconciliation_link_entry_e2e.rs`) → un seul lien.
6. **AC-6 — Recherche** (`FR-38-6c-4`, E22). `GET /api/v1/bank-accounts/{id}/transactions?providerTxId=…&invoiceRef=…&status=…&from=…&to=…&limit=…&offset=…`
   (tout rôle authentifié, comme `GET /bank-accounts` — `authenticated_routes`, `lib.rs:1229` — ; clés d'API admises,
   `read` comprise) :
   - compte de la société (archivé compris : lecture) sinon 404 `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` ;
   - filtres facultatifs, cumulés : `providerTxId` (égalité, index unique `(bank_account_id, provider_tx_id)` de 38-3),
     `invoiceRef` (égalité après `trim`, selon la collation que 38-3 donne à la colonne), `status` ∈ {`pending`,
     `reconciled`} (sinon 400), `from`/`to` sur `booking_date`, inclus ; `limit` 1..=500 (défaut 50), `offset` ;
   - tri `booking_date DESC, id DESC` ; réponse `ListResponse<TransactionResponse>` — le DTO du détail d'un import
     (`routes/bank_imports.rs:302-345`, **ne porte aujourd'hui aucun champ de prestataire** : 38-3 (AC-10) n'étend que la liste des imports de
     `providerConnection`, non ce DTO) **réutilisé** et étendu **ici**, champs additifs : `version`, `bankAccountId`,
     et, lus des colonnes de 38-3 (AC-3) — `providerTxId`, `grossAmount`, `feeAmount`, `movementKind`, `eventCode`,
     `counterpartyEmail`, `invoiceRef`, `itemNames`, `relatedProviderTxId`, `reviewReason`, `bookedAt` (`null` pour une
     transaction de fichier) ; sans eux, le script ne peut pas lire ce qu'il cherche. Test de sérialisation ;
   - lecture seule ; aucune donnée d'un autre compte ni d'une autre société (test).
   Pas d'index neuf sur `invoice_ref` : la recherche est bornée au compte (`idx_bank_transactions_company_account_date`,
   `20260504000001_bank_imports.sql:86`) ; si la mesure de l'AC-8 la montre lente, la fiche le dit **avant** toute
   migration (C-38-0, « une story qui découvre une migration non prévue le dit »).
   Tests HTTP : chaque filtre seul et combinés, compte d'une autre société → 404, clé `read` → 200, `status` faux → 400.
7. **AC-7 — L'API des scripts** (`FR-38-6c-5`, `NFR-38-6c-4`, B21, E22). Test de bout en bout avec une **clé d'API
   `read-write`** (patron des tests de clés, `api_keys`) : (1) `GET /bank-accounts/{id}/transactions?invoiceRef=LG-…`
   retrouve la transaction ; (2) `POST /journal-entries` crée l'écriture en devise (37-3a : lignes `foreign`) dont le
   mouvement sur le compte prestataire égale le net ; (3) `POST /reconciliation/transactions/{id}/link-entry` la
   rattache (200) ; (4) le même parcours par le lot `link-entries`. Un rattachement refusé laisse l'écriture (rien
   n'est annulé) ; la transaction porte alors `alreadyBooked` dans `get_proposals` dès que 38-6a l'y fait paraître —
   constaté ici par `count_already_booked_for_page` (≥ 1). **Fermeture aux clés** : `GET /api/v1/provider-connections`
   (C-38-2, 38-1) et `PUT /api/v1/provider-connections/{id}/secrets` avec la clé → 403 `API_KEY_ADMIN_FORBIDDEN`
   (`require_not_pat`, `middleware/rbac.rs:58`) — si 38-1 porte déjà ce test, il est cité au Change Log au lieu d'être
   doublé. Les routes neuves de cette story ne sont **pas** sous `require_not_pat`.
8. **AC-8 — Temps borné** (`NFR-38-6c-3`). Mesure au développement, écrite au Dev Agent Record avec son montage :
   base de 20 000 écritures dont 3 000 sur le compte prestataire (EUR), 3 000 transactions en attente sur l'année ;
   `link-suggestions` sur l'année par pages de 500 : **cible ≤ 2 s par page** (deux lectures par page, comme la 34-4,
   qui avait mesuré ≈ 18,5 s pour 500 transactions à trois lectures **par transaction**, doc `:949-951`) ; lot de 500
   rattachements : durée mesurée, sans cible (séquentiel par construction, Écart 3). Une cible manquée se signale au
   Project Lead avant la revue, avec la mesure — jamais tue.
9. **AC-9 — Aucune migration** (C-38-0). La story n'ajoute ni table, ni colonne, ni index ; `crates/kesh-db/migrations/`
   n'est pas touché (contrôle `git diff --stat` au Change Log). Le gate complet reste obligatoire en cours de boucle
   (exception `kesh-db` : un dépôt est modifié — `reconciliation_link.rs`).
10. **AC-10 — Registres, i18n, MCP, écrans, manuels.**
    - **Routes** (C-38-12) : `LIB_ROUTES` (`kesh-api/tests/audit_route_registry.rs:382`) gagne `("post",
      "reconciliation::post_link_entries", Traced, SansEcritureAuJournal)` — **+1** route mutante ; le total de
      `the_registry_partition_is_what_the_story_declares` (`:1007-1018`) et son message passent de N à N + 1 (N relu à
      la source au moment du développement : 146 sur `8b3d548d`, avant l'Epic 37 et 38-1 à 38-3) ; les deux `GET` ne
      sont pas des routes mutantes (hors registre). Doc-comment du module (`:262-270`, point (vi)) complété pour le lot.
    - **i18n** : aucun message serveur **neuf** (les codes du lot sont des `FailedProposal` sans texte ; les refus
      globaux réutilisent des codes existants). **Un message existant est reformulé** (Écart 8) :
      `reconciliation-errors-entry-not-linkable` aux **quatre** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,
      it-CH,en-CH}/messages.ftl`), repli de `errors.rs:3236` égal mot pour mot au catalogue `fr-CH` (G9), doc-comment
      `errors.rs:1100-1112` — « la transaction n'est pas dans la devise du compte, ou son montant est nul, … ».
      L'écran (38-9b) traduit les codes du lot (`failed-proposal-label.ts`, quatre locales : **ajoutés par 38-9b**,
      cette story ne touche pas `frontend/`) — liste au Change Log.
    - **Documentation de l'API** (`docs/api-external.md`, ni manuel ni frontend ; **38-9b y renvoie comme à un fichier
      « écrit par 38-6c »**) : sections des trois routes neuves (paramètres, réponses, codes par élément du lot, limite
      de 500, clés `read` / `read-write`), règle du rattachement en devise et de l'autre côté entre devises (§ de
      `…/entry-candidates` et `…/link-entry`, `:807-811` : `amount` est dans la devise du compte), tableau des codes
      (`:1076-1077`, texte « hors CHF » corrigé), parcours du script en trois appels (AC-7). Relu contre le code, à
      livrer dans la même PR.
    - **MCP** (E23) : aucun outil neuf ; les écritures rattachées se lisent déjà par `kesh_ecritures`
      (`mcp/outils/ecritures.rs:198`) ; le rattachement est une écriture d'état hors du périmètre en lecture seule du
      serveur MCP.
    - **Écran et manuel** : 38-9b (Écart 6) ; cette story ne touche aucun fichier de `frontend/` ni `docs/manual/` —
      les sites à reprendre (miroir `canLink`, manuel `:3088`, `:3104`, § `sec:rattacher-reglement`, et la voie API
      des scripts : clé `read-write`, trois appels, codes du lot) sont listés au Change Log pour 38-9b.
11. **AC-11 — Exercice clos, période verrouillée** (`FR-38-6c-7` ; risque de l'epic, « à confirmer à la fiche » — **confirmé au
    code**). Le rattachement n'écrit aucune écriture et ne lit ni `fiscal_years` ni le verrou de période
    (`link_entry_in_tx`, doc `reconciliation_link.rs:1089-1093` : « le geste ne prend **jamais** `invoice_settlements`,
    `supplier_invoices`, `invoices`, `bank_accounts` ni `fiscal_years` : il n'écrit rien au journal ») : une écriture de
    2024 dans un exercice clos ou sous verrou de période **se rattache**, en lot comme à l'unité. Test HTTP : exercice
    2024 clos, transaction de 2024 rattachée par le lot (200) ; le comportement est écrit pour 38-9b (manuel).

## Tâches

- [ ] `kesh-db::repositories::reconciliation_link` : `MovementBasis`, condition 1, `entry_movements(basis)`,
      `entry_sql(mode, basis)`, `other_side_predicate` entre devises et paramètres réordonnés (AC-1, AC-2) ;
      `entry_matches_for_page` et `count_already_booked_for_page` projeté (AC-3) ; `expected_version`,
      `LinkRefusal::VersionConflict` (AC-5) ; `diagnose_entry_link` (AC-5) ; tests unitaires de texte et de la
      condition 1.
- [ ] Messages et documentation (AC-10) : message `reconciliation-errors-entry-not-linkable` aux quatre locales,
      `docs/api-external.md`.
- [ ] `kesh-api` : `link_preflight` rend la base ; les gestionnaires `get_link_suggestions` et `post_link_entries`
      restent **déclarés dans `routes/reconciliation.rs`** (l'identité du registre est `reconciliation::…`), minces :
      la logique du lot va dans un module neuf (`routes/reconciliation_link_batch.rs`), le fichier faisant déjà
      7 439 lignes ; appelants mis à jour ; `get_link_suggestions` (AC-4) ;
      `post_link_entries` (AC-5) ; route de recherche dans `routes/bank_accounts.rs` (AC-6) ; enregistrement à
      `lib.rs` (comptable : les deux routes de réconciliation ; authentifiées : la recherche) ; `LIB_ROUTES`.
- [ ] Tests HTTP des AC-1 à AC-7 et AC-11 ; extension de `s34_4_ac8_…` (AC-3) ; mesure (AC-8).
- [ ] Gate complet (base remise à zéro avant), `scripts/gate-ci.sh` au dernier commit de code ; frontend non touché
      (gate frontend tout de même au push, *Test Locally First*) ; E2E complet au dernier commit de code (les specs du
      rattachement existantes, `reconciliation*`, doivent rester vertes : la règle CHF ne change pas).

## Notes de développement

- **L'invariant le plus cher** est celui de la 34-4 / v017-3a : **une seule règle** (C-v017-3-7). Toute condition
  ajoutée (base, autre côté entre devises) se pose **dans les textes partagés** (`entry_movements`, `linkable_side`,
  `other_side_predicate`), jamais dans une copie pour la page ou les suggestions. Le test d'égalité de l'AC-3 est ce
  qui le garde ; il compare désormais des identifiants, non plus des longueurs.
- **Ordre des verrous** du geste unitaire, inchangé : verrou nommé → `bank_transactions` (`t`) `FOR UPDATE` →
  `journal_entries` (`je`) `FOR UPDATE` → première lecture ordinaire (`ensure_open_in_tx`) → requête restreinte à
  `e.id` → `UPDATE`. Le contrôle de version s'insère **entre** deux contrôles de `t` déjà lue sous verrou : aucune
  lecture neuve avant l'étape 4.2. Le diagnostic (AC-5) se fait **après** le refus : il lit, sans rien décider.
- **Pourquoi la base se lit sur le compte du grand livre et non sur `payment_provider_accounts`** : la règle porte
  sur le mouvement d'un compte du plan ; c'est la devise de ce compte qui dit en quelle unité se lit ce mouvement
  (37-3a, T3 : la devise d'une ligne est celle de son compte). Lire la table des réglages ajouterait un second critère
  qui pourrait diverger du premier.
- **Montant de la transaction en devise et `M` à quatre décimales** : `amount_currency` est un `DECIMAL(19,4)` (37-3a
  AC-1) ; une ligne PayPal en EUR a deux décimales ; l'égalité exacte est la bonne (une écriture saisie à quatre
  décimales qui ne tombe pas au centime n'est pas le mouvement de la transaction).
- **`LinkEntryRequest` perd `Copy`** : `MovementBasis::Currency(String)` porte un `String` (le dérivé `Copy` de
  `reconciliation_link.rs:1059` ne tient plus) ; passer la requête par référence, ou une base `Currency([u8; 3])`
  (`Copy`) — au développement, sans changer le texte des requêtes.
- **Réutilisations** : `ListResponse` (`routes/mod.rs`) ; `FailedProposal` et `merge_failed_in_order`
  (`routes/reconciliation.rs:571`, `:1568`) ; `TransactionResponse` (`routes/bank_imports.rs:302`) ;
  `locked_link_attempt` (`:6411`) ; `link_preflight` (`:6228`).
- **Hors périmètre** : saut hors CHF de `get_proposals` et `confirmAlreadyBooked` (38-6a) ; écran (38-9b) ; somme de
  plusieurs écritures pour une transaction (Écart 5).

## Change Log

- 2026-11-15 — rédaction (Opus).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche (FR-38-6c-7, Écarts
  7 et 8, DTO de la recherche étendu par la story, message du refus unitaire reformulé aux quatre locales,
  `docs/api-external.md` livré par la story, emplacement des gestionnaires, perte de `Copy`) ; inter-fiches : 38-9b
  (libellés du lot à y ajouter ; elle renvoie à `docs/api-external.md`, désormais bien écrit ici).
