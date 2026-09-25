# Story 25.3-b : Annuler un rapprochement bancaire — par contre-passation

Status: ready-for-dev

**Issue : [#418]** — c'est cette story qui la **ferme** : `closes #418` dans le **titre ET le corps**
de sa PR (le dépôt merge en squash ; un `refs` partout laisserait l'issue ouverte sans signal).

**Grand-mère : `25-3-annuler-reglement-et-rapprochement.md`** (statut `split`) — **source des faits**,
ses corrections valent ici. **Socle** : `25-3-zero-reverse-in-tx.md` (PR #453). **Sœurs mergées** :
25-3-a-1 (règlement client, PR #457) et 25-3-a-2 (règlement fournisseur, PR #462) — cette story
**appelle** le geste de la 25-3-a-1 et **réutilise** la queue commune des motifs. **Cousine** :
25-3-c (annuler une facture fournisseur, #454), sans recouvrement.

## Story

En tant que comptable,
je veux annuler un rapprochement bancaire accepté par erreur,
afin que la transaction redevienne « à rapprocher » et que le grand livre, la facture et le relevé
restent d'accord — sans écriture manuelle, que le gel de la 24-4b interdit de toute façon.

## Pourquoi

⛔ **Aujourd'hui, un rapprochement ne se défait pas, et son écriture est définitivement
incorrigible.** Les routes de réconciliation montées sont `accept`, `reject`, `manual` et `split`
(`crates/kesh-api/src/lib.rs:610-631`) — aucune ne défait. La contre-passation directe refuse une
écriture rapprochée (`ReversalBlocker::MatchedBankTransaction`, dont la doc dit elle-même
« aucune route de dé-rapprochement n'existe (#418) », `errors.rs:78-81`), le gel de la 24-4b en
interdit la modification, et le règlement client né d'un rapprochement est refusé par la 25-3-a-1
au rang 3 (« annulez d'abord le rapprochement ») — **un chemin que l'application ne propose pas**.
Le manuel l'avoue (`user-manual.tex:1846-1848`, « il ne se défait pas encore depuis
l'application »).

## ⛔ Les faits qui structurent la story — établis depuis la source le 2026-09-25

1. **Cinq chemins posent le lien, deux familles d'écritures.** `matched_entry_id` s'écrit en SQL
   brut à **cinq** sites de `routes/reconciliation.rs` :

   | site | chemin | ce qui est créé |
   |---|---|---|
   | `:1467` | `accept_one_invoice` (proposition « facture ») | écriture d'**encaissement** `D banque / C créance` **+ ligne `invoice_settlements`** (`:1416`) + `paid_at` projeté |
   | `:1892` | `accept_one_split` (proposition « éclatement ») | une écriture `banque / contreparties` |
   | `:2256` | `accept_one_rule` (proposition « règle ») | une écriture `banque / contrepartie` |
   | `:2948` | `post_manual` | une écriture `banque / contrepartie` |
   | `:3409` | `post_split` | une écriture `banque / contreparties` |

   ⇒ **Deux cas seulement pour défaire** : l'écriture est un **règlement client** (une ligne
   `invoice_settlements` la porte) — on appelle le geste de la 25-3-a-1 ; sinon l'écriture n'est
   possédée **que** par la transaction — on la contre-passe. Les éclatements, règles et
   rapprochements manuels n'imputent **que** des comptes de contrepartie (`SplitProposalLine`,
   `:372-380`) : aucun ne règle de facture. Aucun chemin ne rapproche un règlement **fournisseur**.

2. **Le lien d'abord, le reste ensuite — c'est ce qui lève le rang 3 sans exemption.** La
   25-3-a-1 l'a préparé en toutes lettres (`invoice_settlements_write.rs:340-342`) : son geste
   `cancel_settlement_in_tx` est `pub`, sans `BEGIN`/`COMMIT`, et le socle refuse une écriture
   rapprochée **tant que le lien existe**. Remettre `matched_entry_id` à `NULL` **dans la même
   transaction, avant** la contre-passation, fait disparaître le motif — aucune variante
   d'autorité nouvelle n'est nécessaire.

3. ⛔ **Jamais par la suppression.** La FK `fk_bank_transactions_matched_entry` est
   `ON DELETE SET NULL` (`20260504000001_bank_imports.sql:83`) : supprimer l'écriture effacerait le
   lien **en silence**. Et **aucune contrainte** ne lie `status` à `matched_entry_id`
   (`chk_bank_transactions_status` ne porte que les deux valeurs) : un `reconciled` sans lien est
   **possible** en base (écriture supprimée avant le gel de la 24-4b) — mais aucune installation
   n'en porte (Q1, ci-dessous).

4. ⚠️ **Les rapprochements antérieurs à la 24-2 pointent sur l'écriture de VENTE.** Avant la
   Story 24-2 (v0.12.0), `accept_one_invoice` liait la transaction à l'écriture de **vente** de la
   facture, sans écriture d'encaissement ni ligne de règlement, et posait `paid_at`. La migration
   `20260827000001_invoice_settlements.sql` est du DDL pur : **rien n'a été repris**. Une
   installation venue de la v0.11.1 ou d'avant, ou une sauvegarde de cette époque restaurée, peut
   donc porter ce lien. ⛔ **Le contre-passer annulerait la VENTE** — le socle le refuse
   (`OWNED_BY_INVOICE` précède `MATCHED_BANK_TRANSACTION`). **Arbitrage Q1** : Kesh n'est pas en
   production, aucune donnée n'est à préserver — **aucun chemin dédié** (AC 5).

5. **Le verrou.** Les quatre routes mutantes de la réconciliation se sérialisent par un verrou
   nommé MariaDB, par compte bancaire (`kesh_reconciliation::mutex::with_account_lock`,
   `GET_LOCK('reconcile:{db}:{company}:{account}')`). Le dé-rapprochement prend **le même**.

6. **Où l'on voit une transaction rapprochée : un seul écran.** La page de réconciliation ne liste
   que les transactions **en attente**. La seule page qui montre une transaction `reconciled` est
   le **détail d'un import** (`frontend/src/routes/(app)/bank-import/[id]/+page.svelte`), qui
   affiche le statut **brut** (`{tx.status}`, `:101`), ne donne aucun lien vers l'écriture, et dont
   tous les libellés sont **en dur** (dette antérieure — cf. Dev Notes).

7. **Le marqueur de rejet.** Quatre des cinq chemins remettent `auto_match_rejected_at` à `NULL`
   en rapprochant ; le chemin « facture » ne part que de transactions non rejetées. Une transaction
   dé-rapprochée dont le marqueur resterait posé **n'apparaîtrait plus** dans les propositions
   (`find_pending_transactions_for_account` filtre `auto_match_rejected_at IS NULL`,
   `repositories/reconciliation.rs:158`).

## Arbitrages de Guy qui s'appliquent ici

- **Ligne `invoice_settlements` RETIRÉE**, non marquée annulée (grand-mère, 2026-09-24).
- **Un paiement détaché de sa facture redevient à lettrer** (2026-09-24) : la transaction revient
  « à rapprocher ».
- ⛔ **Q5 — un règlement dont l'écriture est dans un exercice CLOS ne s'annule pas ; rouvrir
  l'exercice doit rester possible** (2026-09-24). Le dé-rapprochement d'une facture **en hérite**
  par le geste de la 25-3-a-1 (rang 2) — la 25-3-a-1 le disait (`25-3-a-1…md:160-162`).
- **Le lettrage est un prérequis de la mise en service** (décision kesh:D5, 2026-09-25) : défaire
  un rapprochement est le geste inverse du lettrage, et il doit exister avant.

## Arbitrages de Guy sur cette fiche (2026-09-25)

- **Q1 — Aucune donnée à préserver** : « kesh n'est pas encore en production ». Les liens
  antérieurs à la 24-2 (fait 4) et les transactions `reconciled` sans lien (fait 3) n'existent sur
  aucune installation : **pas de chemin dédié**, pas de code pour eux (AC 5).
- **Q2 — L'exercice de la FACTURE n'arrête rien** : « une facture peut être enregistrée à la fin
  d'un exercice et payée au début du suivant : si l'exercice précédent est clos, la facture doit
  pouvoir être rapprochée ». ⇒ Le seul exercice qui compte est celui de l'**écriture de
  rapprochement** (datée du paiement), jamais celui de la vente. C'est déjà le cas à
  l'acceptation (`accept_one_invoice` n'exige qu'un exercice ouvert à la **date de valeur**,
  `reconciliation.rs:1349-1350`) et dans la queue commune (rang 2 lu sur l'écriture de
  **règlement**) ; un test le fixe (AC 12). Le refus du rang 2 porte donc sur un rapprochement
  dont l'écriture **elle-même** est dans un exercice clos — Q5, pour tous les rapprochements.
  ⚠️ **L'EXERCICE de la facture n'arrête rien — sa DATE, si** (précision de Guy) : elle fixe
  l'échéance à défaut de date limite, et borne la fenêtre de l'acceptation automatique. Cette
  story ne touche ni l'une ni l'autre (cf. Dev Notes, faits voisins).
- **Q3 — Les deux boutons** : sur le détail de l'import **et**, sur la fiche facture client, à côté
  du motif « rapproché » d'un règlement (AC 11).

## Acceptance Criteria

### Le geste

1. **`reconciliation_cancel::cancel_in_tx(tx, company_id, bank_transaction_id, user_id,
   actor_api_key_id: Option<i64>)`** — nouveau
   module de dépôt `crates/kesh-db/src/repositories/reconciliation_cancel.rs`, sans `BEGIN` ni
   `COMMIT`, plus son enveloppement. Il rend : la transaction relue, l'écriture inverse
   (`i64`), et la facture touchée
   (`Option<i64>`). **Dans cet ordre** :
   1. verrou `FOR UPDATE` sur la transaction bancaire, scopé par `company_id` — `NotFound` sinon ;
      `status <> 'reconciled'` → refus `BANK_TRANSACTION_NOT_RECONCILED` (AC 3, tête) ;
   2. **classement** du lien (lecture non verrouillante, puis verrous) :
      - une ligne `invoice_settlements` porte `journal_entry_id = matched_entry_id` → **règlement
        client** ; verrou `FOR UPDATE` de la **facture puis** de la ligne — ⛔ **l'ordre de
        `cancel_settlement_in_tx`** (facture → règlement → écriture + exercice), pour ne jamais
        croiser une annulation de règlement lancée depuis la fiche facture ;
      - sinon → **écriture propre** à la transaction ;
   3. verrou de l'écriture **et de son exercice** (`journal_entries JOIN fiscal_years … FOR
      UPDATE`) **avant** de juger — la leçon de la revue de la 25-3-a-1 (course avec
      `fiscal_years::close`) ;
   4. les motifs, par `cancel_blocker` (AC 3) — ⛔ **la forme EXEMPTÉE**, pour la transaction qu'on
      défait : le lien existe encore à cette étape, et la forme non exemptée ferait échouer
      **chaque** dé-rapprochement au rang 3 sur son propre lien. La tête (rang 0) et le rang 2
      refusent ici, par `DbError::ReconciliationNotCancellable` (AC 3) ; le rang 3 ne subsiste que
      si une **autre** transaction pointe l'écriture ; les rangs 4-5 sont laissés au socle ;
   5. `UPDATE bank_transactions SET matched_entry_id = NULL, status = 'pending',
      auto_match_rejected_at = NULL, version = version + 1, updated_at = NOW(3) WHERE id = ? AND
      company_id = ? AND status = 'reconciled' AND version = ?` — `rows_affected() == 1`, sinon
      `OptimisticLockConflict` ; ⛔ **avant** toute contre-passation (fait 2) ;
   6. selon le cas :
      - **règlement client** → `invoice_settlements_write::cancel_settlement_in_tx(tx, company_id,
        invoice_id, settlement_id, user_id)`, **appelé, jamais réécrit** : il contre-passe,
        retire la ligne, projette `paid_at`, bumpe la facture et écrit
        `invoice.settlement_cancelled` ;
      - **écriture propre** → `journal_entries::reverse_in_tx` (sans autorité), dont l'identifiant
        d'écriture inverse est `.entry.id` du `JournalEntryWithLines` rendu (celui du règlement :
        `SettlementCancellation::reversal_journal_entry_id`) ;
   7. audit `reconciliation.cancelled` (AC 4).

   ⛔ **Aucune troisième contre-passation** : le socle et le geste de la 25-3-a-1, rien d'autre.
   ⛔ **Aucune suppression** d'écriture (fait 3).

2. **La date de la contre-passation est celle du JOUR, dans un exercice ouvert — tranchée par le
   socle**, comme dans les deux sœurs. Pas de date fournie par l'appelant. `PERIOD_LOCKED` (400) si
   la période du jour est verrouillée, sans code neuf.

### Ce qui empêche l'annulation

3. **Une précédence, la queue commune réutilisée, une exemption ÉTROITE.** La fonction de lecture
   `reconciliation_cancel::cancel_blocker(conn, company_id, bank_transaction_id)` sert **à la fois**
   la lecture (AC 7) et l'écriture (AC 1). Elle ajoute à `SettlementCancelBlocker`
   (`kesh-db/src/errors.rs`) **une** variante de tête, `BankTransactionNotReconciled` (code
   `BANK_TRANSACTION_NOT_RECONCILED`), et délègue le reste :

   | rang | variante | code | s'applique à | à l'écriture |
   |---|---|---|---|---|
   | 0 | `BankTransactionNotReconciled` | `BANK_TRANSACTION_NOT_RECONCILED` | tous | 409, `ReconciliationNotCancellable` |
   | 1 | `InvoiceCredited` | `INVOICE_CREDITED` | règlement client | 409 — refusé par le geste de la 25-3-a-1 |
   | 2 | `FiscalYearClosed` | `FISCAL_YEAR_CLOSED` | tous — exercice de l'écriture **de rapprochement**, jamais de la facture (Q2) | 409, `ReconciliationNotCancellable` |
   | 3 | `MatchedBankTransaction` | `MATCHED_BANK_TRANSACTION` | seulement si une **autre** transaction pointe la même écriture | laissé au socle : 409 `EntryNotReversable` |
   | 4 | `AccountArchived` | `ACCOUNT_ARCHIVED` | règlement client, écriture propre | laissé au socle : 400 qui **nomme** les comptes |
   | 5 | `NoOpenFiscalYearToday` | `FISCAL_YEAR_INVALID` | règlement client, écriture propre | laissé au socle : 400 `FiscalYearInvalid` |

   - ⛔ **Rangs 2 à 5 : la queue commune** `settlement_cancellation::settlement_entry_cancel_blocker`,
     sur l'écriture liée, **pas un jumeau**. Elle rend aujourd'hui `MatchedBankTransaction` pour
     toute écriture rapprochée — donc pour **chaque** dé-rapprochement. Elle reçoit une
     **exemption étroite** : le rang 3 est levé **pour la transaction qu'on défait**, et seulement
     elle. ⛔ **Pas par l'identifiant de `reversal_blockers`** : sa sous-requête est un `LIMIT 1`
     **sans `ORDER BY`** (`journal_entries.rs:1309`) — elle rend **une** transaction arbitraire,
     et « l'identifiant rendu est celui qu'on défait » ne prouve pas qu'il n'y en a pas d'autre.
     Quand une exemption est fournie, la queue fait sa **propre requête** — `SELECT id FROM
     bank_transactions WHERE company_id = ? AND matched_entry_id = ? AND id <> ? ORDER BY id LIMIT
     1` — et le rang 3 tient, avec **cet** identifiant, si elle rend une ligne ; sans exemption, le
     chemin actuel est inchangé. Les deux sœurs appellent la queue **sans** exemption : le
     **comportement** et leurs **tests** ne changent pas — leur **code** reçoit l'argument
     supplémentaire (`None`), imposé par le compilateur.
   - **Rang 1** : celui de la tête client de la 25-3-a-1 (`settlement_cancel_blocker`), **sans
     copie** — si sa forme actuelle (tête puis queue sans exemption) empêche de le réutiliser,
     extraire le rang 1 en fonction et l'appeler des deux côtés.
   - ⛔ **Une erreur PROPRE au geste** : `DbError::ReconciliationNotCancellable { blocker:
     SettlementCancelBlocker }` (409, `blocker.code()`), mappée dans `crates/kesh-api/src/errors.rs`
     vers la famille **`reconciliation-cancel-blocked-*`** (AC 9). ⛔ **Pas**
     `SettlementNotCancellable`, dont le texte serveur dit « ce règlement »
     (`kesh-api/src/errors.rs:2573-2601`) — faux pour un éclatement, une règle ou un rapprochement
     manuel. Le rang 1, lui, reste refusé par `cancel_settlement_in_tx` en
     `SettlementNotCancellable` : dans ce cas l'écriture **est** un règlement, et le texte est juste.
   - **Qui refuse à l'écriture** : comme dans les sœurs — le geste refuse la tête et le rang 2 ;
     les rangs 3 à 5 sont refusés par le socle avec son erreur canonique (une seule garde par
     motif). Au rang 1, c'est `cancel_settlement_in_tx` qui refuse, **après** que le lien a été
     défait dans la transaction : l'erreur remonte, et le **rollback** rétablit le lien (AC 11,
     composition).
   - ⚠️ **Limite assumée** héritée : le verrou de période du jour n'est pas dans la lecture — il se
     contrôle au clic (`PERIOD_LOCKED`).

### L'audit

4. **`reconciliation.cancelled`**, **littéral** au site d'insertion, écrit par
   `NewAuditLogEntry::for_actor(user_id, actor_api_key_id, …)` — patron des quatre routes de
   réconciliation (`post_manual`, `routes/reconciliation.rs:2986-2995`) —, inscrit à
   `audit_labels.rs::ACTIONS` (liste triée) et libellé dans les **quatre** locales. Entité
   `bank_transaction`. Charge : `kind` (`invoice_settlement` / `entry`), `matchedEntryId`, `reversalJournalEntryId`, `invoiceId`, `settlementId`, montant,
   `wasPreviouslyRejected`. ⚠️ Pour un règlement client, **trois** lignes au total —
   `reconciliation.cancelled`, `invoice.settlement_cancelled` (le geste de la 25-3-a-1) et
   `journal_entry.reversed` (le socle) : c'est voulu, chacune nomme son objet. ⚠️ **Limite
   assumée** : les deux dernières sont écrites par `NewAuditLogEntry::user` (sœur et socle,
   `invoice_settlements_write.rs:466`) et ne portent **pas** la clé API ; seule
   `reconciliation.cancelled` la porte, dans la même transaction. Ne pas modifier la sœur ici —
   le défaut est **antérieur** (sa propre route admet les clés), signalé à Guy.

### Les cas particuliers

5. **Aucun chemin pour les états qui n'existent pas (Q1).** Un lien vers l'écriture de **vente**
   (fait 4) tombe dans le cas « écriture propre » et est **refusé par le socle**
   (`OWNED_BY_INVOICE`, 409), la transaction annulée — rien n'est écrit. Une transaction
   `reconciled` sans lien (fait 3) est refusée en `DbError::Invariant`. Aucun test dédié, aucune
   branche : le doc-comment du geste le dit, avec la raison (arbitrage Q1).

### L'API

6. **`POST /api/v1/reconciliation/transactions/{id}/cancel`** — **Comptable+**, à côté des quatre
   routes de réconciliation (`lib.rs:610-631`) ; clés API d'écriture admises, comme elles
   (`actor_api_key_id` porté à l'audit). Le handler : lecture de la transaction (société) pour
   connaître son compte, **`with_account_lock`** sur ce compte (patron `post_manual` : mapping de
   `LockReleaseFailed`, `AccountLocked` → 409 existant), puis `cancel_in_tx`, puis `COMMIT`.
   Réponse : la transaction relue (même forme que dans le détail d'import, AC 8),
   `reversalJournalEntryId`, `invoiceId`. Erreurs : 404 hors société, 409 avec `code` (AC 3),
   400 comptes archivés / exercice / période. Au registre `audit_route_registry.rs` : `Traced`,
   totaux **recomptés depuis la source** (départ : **108 routes / 90 tracées / 15 exemptées / 3
   sans objet, 111 avec les routes de test**), message de ventilation compris. ⚠️ L'**en-tête** du
   fichier (`audit_route_registry.rs:17-24`) dit encore « 106 routes » et « 109 » — **déjà faux**
   avant cette story : le corriger au passage, en recomptant.
   ⛔ **Rejeu sur interblocage** : le handler enveloppe **toute** l'opération dans
   `kesh_db::retry::retry_with` — patron de `routes/onboarding.rs:596-621` (KF-002-H-002, #43), cf.
   piège 6-bis. **Emboîtement, du dehors vers le dedans** :

   ```text
   retry_with(DEFAULT_MAX_DEADLOCK_ATTEMPTS,
              |e: &AppError| matches!(e, AppError::Database(db) if is_deadlock_error(db)),
              || {                                  // closure Fn : cloner, puis async move
                  let pool = state.pool.clone();    // (patron onboarding.rs:614-620)
                  async move {
                      let mut tx = pool.begin();    // transaction NEUVE à chaque tentative
                      with_account_lock(&mut tx, company, account, timeout,
                          async |tx| Ok(cancel_in_tx(tx, …).await?))  // DbError → ReconciliationError::Db
                      → mapping ReconciliationError → AppError (patron post_manual :3006-3080)
                      → tx.commit()
                  }
              })
   ```

   Pourquoi dans cet ordre : `retry_with` rejoue une closure **`Fn`** qui doit repartir de zéro
   (le 1213 a déjà annulé la transaction côté MariaDB) ; le verrou nommé est relâché par
   `with_account_lock` à chaque sortie **tant que `RELEASE_LOCK` aboutit** — ⚠️ un `GET_LOCK` est
   lié à la **session**, non à la transaction : si le relâchement échoue sur un chemin d'erreur,
   `mutex.rs` ne fait que journaliser, et le verrou fuit jusqu'à la fin de session (L22) ; la
   tentative suivante attendrait alors le délai et rendrait 409 `AccountLocked`, non rejoué.
   Cas pathologique (après un 1213, la connexion est vivante), **antérieur** à cette story :
   limite assumée. ⚠️ **Le
   prédicat porte sur `AppError`**, pas sur `DbError` : `is_deadlock_error` ne reconnaît que
   `DbError::Sqlx(1213)`, et c'est la chaîne de mapping qui doit le **préserver** —
   `ReconciliationError::Db(db)` → `AppError::Database(db)` et `ReconciliationError::Database(e)` →
   `AppError::Database(DbError::Sqlx(e))`, comme `post_manual` le fait déjà (`:3046-3052`). Un
   mapping qui convertirait l'erreur en autre chose (`Internal`, texte) rendrait le rejeu **muet**.

7. **`GET /api/v1/reconciliation/transactions/{id}`** — Comptable+ (comme les propositions) :
   `id`, `status`, `matchedEntryId`, `kind` (AC 4), `invoiceId`, `invoiceNumber`, `amount`,
   **`cancellable`**, **`cancelBlockedBy`**, **`cancelBlockedLabel`** (numéro du compte au rang 4),
   **`cancelBlockedDocumentId`** (l'**autre** transaction au rang 3). Lus **dans une seule
   transaction de lecture** — la leçon de la revue P1 de la 25-3-a-2 (une réponse qui mêle deux
   instantanés peut se contredire). Route GET : hors registre (il ne porte que les mutantes).
   ⚠️ **Pourquoi une route et pas des champs dans le détail d'import** : un import porte des
   centaines de transactions ; calculer la précédence pour chacune coûterait des milliers de
   requêtes. Le calcul se fait **au clic**, pour une transaction.

8. **`GET /api/v1/bank-imports/{id}`** : chaque transaction gagne **`matchedEntryId`** (lien vers
   l'écriture) — lecture de colonne, sans calcul. `api-external.md` le dit.

### Les textes

9. **Propres au rapprochement, écrits une fois.** Famille **`reconciliation-cancel-*`** dans
   `frontend/src/lib/features/reconciliation/` (bouton, confirmation par `kind`, succès, lien vers
   l'écriture inverse) et **`reconciliation-cancel-blocked-*`** pour les six codes de l'AC 3 —
   ⛔ **pas** la famille partagée `settlement-cancel-blocked-*`, qui dit « ce règlement » : une
   écriture d'éclatement n'est pas un règlement (même raison que la 25-3-a-1 pour ne pas détourner
   le mapping des écritures). `switch` exhaustif, garde `never`. **Quatre** locales, repli en dur
   **mot pour mot** le FTL fr-CH. ⛔ **Qui affiche quoi** : le **dialogue** traduit toujours le **`code`**
   reçu — par la lecture de l'AC 7 comme par le refus au clic — avec **sa** famille
   `reconciliation-cancel-blocked-*`, pour les six codes, `INVOICE_CREDITED` compris ; le texte de
   **repli serveur** dépend, lui, de qui refuse : rangs 0 et 2 → `ReconciliationNotCancellable`
   (famille neuve) ; rang 1 → `SettlementNotCancellable` de la 25-3-a-1, dont le texte (« ce
   règlement… ») est **juste** puisque l'écriture est alors un règlement ; rangs 3 à 5 → les
   erreurs du socle. ⛔ **Les codes qui ne sont pas des motifs** — au clic, la route rend aussi
   `PERIOD_LOCKED`, `OPTIMISTIC_LOCK_CONFLICT`, `RECONCILIATION_ACCOUNT_LOCKED`,
   `RECONCILIATION_LOCK_RELEASE_FAILED`, `NOT_FOUND` (`kesh-api/src/errors.rs:1913, 1927, 2257,
   2262, 2698`) — : le dialogue affiche le **`message` du serveur**, déjà traduit et propre à
   chacun. La garde `never` porte sur l'**union typée des six motifs** ; la branche de repli, hors
   de l'union, lit le message serveur — jamais un texte générique (AC 10). Test Vitest avec
   `PERIOD_LOCKED`. Côté serveur, donc, le **bloc de mapping** de
   `ReconciliationNotCancellable` (`crates/kesh-api/src/errors.rs`) — un texte par code qu'il porte
   (rangs 0 et 2), clés `reconciliation-cancel-blocked-*`.

   **Les refus qui orientaient vers un chemin absent nomment le chemin réel** — sites, tous
   ensemble (grep de la **clé**, du **code** et de la **phrase**) :
   - `journal-entries-reverse-blocked-bank-match` ×4 (fr-CH `:344`) + repli serveur
     (`errors.rs:2478`) + repli Svelte (`journal-entries/[id]/+page.svelte:161-165`) : « Cette
     écriture est rapprochée d'une transaction bancaire » → dire **où** l'annuler (détail de
     l'import bancaire) ;
   - `settlement-cancel-blocked-bank-match` ×4 (`:725`) + replis (`errors.rs:2589`,
     `lib/shared/utils/settlement-cancel-blocked.ts:41-45`) ;
   - `error-invoice-unvalidate-blocked-matched` ×4 (`:30`) : **ne bouge pas** — ce refus de la
     dévalidation ne naît que d'un lien vers la **vente** (fait 4), qui n'existe nulle part (Q1) ;
   - la doc de `ReversalBlocker::MatchedBankTransaction` (`kesh-db/src/errors.rs:78-81`, « aucune
     route de dé-rapprochement n'existe ») et les commentaires qui annoncent la 25-3-b
     (`errors.rs:212`, `journal_entries.rs:1454`, `invoice_settlements_write.rs:341`,
     `tests/reconciliation_e2e.rs:2964`) — **les relire et les mettre au présent**.
   ⛔ **Les refus restent** : contre-passer directement une écriture rapprochée reste faux.

### L'écran

10. **Détail d'un import** (`bank-import/[id]/+page.svelte`) : pour une transaction `reconciled`,
    un lien vers l'écriture liée et un bouton **« Annuler le rapprochement »**, masqué pour un rôle
    sans droit d'écriture. Au clic : lecture de l'AC 7, puis un dialogue qui montre **soit le
    motif** (AC 9), **soit la confirmation** — qui dit ce qui va se passer selon le `kind` (« une
    écriture inverse datée d'aujourd'hui sera passée » ; pour une facture, « le règlement de la
    facture N° … sera retiré et la facture redeviendra à régler »). Après succès : le détail est **relu**. Un refus au clic affiche son
    motif — le 409 porte `code` et `details`, le 400 `ACCOUNT_ARCHIVED` porte `details.rejected[]`
    —, jamais un message générique. Les **nouveaux** libellés passent par `i18nMsg` ; les libellés
    en dur **existants** de la page ne sont pas migrés ici (dette antérieure, cf. Dev Notes).

11. **Fiche facture client** (Q3) : un règlement dont le motif est `MATCHED_BANK_TRANSACTION`
    (rang 3 de la 25-3-a-1, `cancelBlockedDocumentId` = la transaction) montre, à côté du motif,
    un bouton **« Annuler le rapprochement »** qui ouvre **le même dialogue** (même composant, même
    lecture AC 7) ; après succès, la liste des règlements et la facture sont relues. ⛔ **Un seul
    composant de dialogue**, `features/reconciliation/CancelReconciliationDialog.svelte` — premier
    dialogue partagé entre deux pages (la 25-3-a-1 confirme **dans** la page, `InvoiceSettlements`
    déléguant au parent) ; gabarits d'extraction : `SettleInvoiceDialog.svelte`,
    `SendEmailDialog.svelte`. **Contrat** : props `bankTransactionId: number`, `open: boolean`,
    `onClose()`, `onSuccess(result)` — le composant fait **lui-même** la lecture de l'AC 7 à
    l'ouverture et l'appel de l'AC 6 ; chaque page ne fait que **relire** dans `onSuccess`, et
    masque le bouton selon le rôle.

### Tests

12. ⛔ Chaque garde **prouvée par mutation**, vue **rouge sur assertion**, mutation décrite au Dev
    Agent Record.
    - **Les cinq chemins, par les VRAIS chemins** (`kesh-api/tests/reconciliation_e2e.rs`, qui
      sait déjà accepter une proposition) : rapprocher puis dé-rapprocher par **facture**,
      **éclatement accepté**, **règle**, **manuel**, **éclatement manuel**. Pour chacun :
      transaction `pending`, `matched_entry_id` NULL, marqueur de rejet NULL, écriture inverse
      avec `reverses_entry_id`, l'origine en `ALREADY_REVERSED`, **la transaction réapparaît dans
      `GET /reconciliation/proposals`**. Facture : ligne de règlement retirée, reste dû = TTC,
      `paid_at` NULL.
    - **Facture partiellement réglée deux fois** (un règlement manuel + un rapprochement) :
      dé-rapprocher ne retire que le règlement rapproché ; `paid_at` reste NULL, reste dû correct.
    - **Re-rapprocher** après dé-rapprochement : la même transaction s'accepte à nouveau.
    - **Précédence** : chaque rang seul, par les vrais chemins (clôture par
      `fiscal_years::close`, avoir, archivage, exercice du jour absent — ⚠️ montage de la 25-3-a-1 :
      écriture dans un exercice ouvert qui ne couvre pas le jour) ; **l'exemption étroite** — deux
      transactions pointant la même écriture (état **forgé**, déclaré tel) ⇒ rang 3 avec
      l'identifiant de l'**autre** ; mutation « l'exemption lève tout rang 3 » ⇒ rouge. ⛔ Le test
      forge les **deux ordres d'insertion** (la transaction défaite créée avant, puis après
      l'autre) : un test à un seul ordre passerait avec la lecture naïve du `LIMIT 1`.
    - **Texte du refus** : un rang 2 sur une écriture d'**éclatement** rend le code
      `FISCAL_YEAR_CLOSED` et un message qui ne dit **pas** « règlement » (réponse HTTP lue) ;
      mutation « `SettlementNotCancellable` au lieu de `ReconciliationNotCancellable` » ⇒ rouge.
    - **Les sœurs inchangées** : les tests de la queue et des deux gestes de règlement passent
      **sans retouche** (preuve que l'exemption est optionnelle).
    - ⛔ **Q2 — facture d'un exercice clos, payée dans le suivant** : facture validée en fin
      d'exercice N, exercice N **clos** (`fiscal_years::close`), paiement en début de N+1 —
      **rapprocher puis dé-rapprocher réussissent**. Mutation « le rang 2 lit l'exercice de la
      **vente** » ⇒ rouge.
    - ⛔ **Composition et rollback** : sur un règlement d'une facture **créditée** (rang 1, refusé
      par `cancel_settlement_in_tx` **après** que le lien a été défait) — la route rend 409 et,
      **après**, le lien est **toujours là** (lecture positive de `matched_entry_id`), rien n'est
      écrit.
    - **Concurrence** : une clôture d'exercice concurrente attend le dé-rapprochement (sonde
      `kesh_db::test_fixtures::attendre_une_requete_en_cours`, déterministe) — mutation « pas de
      verrou de l'exercice » ⇒ rouge ; deux dé-rapprochements simultanés de la même transaction :
      l'un réussit, l'autre rend 409 `BANK_TRANSACTION_NOT_RECONCILED`.
    - **Étanchéité multi-tenant** : société B ne peut ni lire ni annuler (404), et **rien n'est
      écrit** — table par table : `bank_transactions`, `journal_entries`, `journal_entry_lines`,
      `invoice_settlements`, `invoices`, `audit_log`.
    - **API** : 200, **403 pour Consultation**, 404, 409 avec `code`, 400 ; `GET` → 200 Comptable,
      403 Consultation.
    - **Vitest** : le dialogue (motif pour chaque code, confirmation par `kind`, rôle), le bouton
      de la fiche facture. **Gardes i18n recomptées depuis la source** (`i18n-keys.test.ts`,
      `i18n-un-repli-par-cle.test.ts` `CLES_RELEVEES`, `i18n-libelle-en-dur.test.ts`).
    - **Playwright** : importer, rapprocher, annuler depuis le détail de l'import ; la transaction
      revient dans la réconciliation.

### Documentation et gates

13. **Manuel utilisateur FR** (`docs/manual/fr/user-manual.tex`), liste **close** :
    - section « Réconciliation bancaire » (`:1309`) : sous-section **« Annuler un
      rapprochement »** — où (détail de l'import, fiche facture), ce qui est écrit, les motifs de
      refus (dont l'exercice clos : celui du **paiement**, jamais celui de la facture — Q2) ;
    - `:1846-1848` (« il ne se défait pas encore ») → **faux** : réécrire ;
    - `:970-971` (règlement rapproché → « annuler ce rapprochement ») : dire **où** ;
    - `:1102` (dévalidation refusée, écriture rapprochée) : relire ;
    - `:1836-1838` (« passez par le chemin de la pièce ») : relire, sans modifier ;
    - `:1839-1844` (exception de l'exercice clos) : l'étendre aux **rapprochements** (Q5).
    Régénérer le PDF et le **contrôler aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).
    `api-external.md` : les deux routes, le champ `matchedEntryId`, clés API admises. `README.md` :
    la ligne « Import bancaire … » des fonctionnalités (`README.md:39`, qui énumère les
    réconciliations). `CHANGELOG.md`, section **`[0.12.1] — Non
    publié`** (existe, `:11`) — aucune ligne d'une section publiée ne se réécrit, décomptes
    **recomptés**. `website/` : rien (dette antérieure hors périmètre).

14. **Gate complet** — `kesh-db` touché : ciblage interdit, même en boucle de revue ; base remise à
    zéro avant chaque gate complet ; E2E sur `kesh_e2e` **reconstruite** (DROP + migrations).

## Tasks / Subtasks

- [ ] **T1 — Les motifs** (AC 3) : variante `BankTransactionNotReconciled`, exemption étroite de la
      queue, rang 1 réutilisé sans copie, `cancel_blocker`.
- [ ] **T2 — Le geste** (AC 1, 2, 5) et l'audit (AC 4).
- [ ] **T3 — Les routes** (AC 6, 7, 8), registre recompté.
- [ ] **T4 — Les textes** (AC 9) : familles `reconciliation-cancel-*` ×4, refus réorientés ×4 +
      replis, commentaires mis au présent.
- [ ] **T5 — Les écrans** (AC 10, 11) : dialogue unique, détail d'import, fiche facture.
- [ ] **T6 — Tests et mutations** (AC 12).
- [ ] **T7 — Documentation** (AC 13), gates (AC 14), PR `closes #418`.

## Dev Notes

### Ce que cette story ne fait pas

- **Le lettrage** (Epic 15, #459-#461) : une transaction dé-rapprochée revient « à rapprocher » ;
  la rattacher à une autre facture est le geste existant (accepter une proposition, rapprocher
  manuellement).
- **Le dé-rapprochement en lot** : une transaction à la fois. Pas de `FailedProposal` : la route
  n'est pas un batch, les erreurs sont des `AppError` normales.
- **Les liens hérités et les orphelins** (faits 3 et 4) : aucun chemin, aucune reprise (Q1).
- **La traduction des libellés existants** du détail d'import (dette antérieure, cf. ci-dessous).
- #455, #456, #416, #384, 25-3-c (#454).

### Ce qu'il faut savoir du code existant

- **`accept_one_invoice`** (`routes/reconciliation.rs:1056-1603`) : écriture d'encaissement
  `D banque / C créance` du **montant de la transaction**, créance lue sur l'écriture de vente,
  trop-perçu refusé, ligne `invoice_settlements` (`BankTransfer`), `paid_at` projeté, deux audits
  (`reconciliation.accepted`, `invoice.paid` ou `invoice.partially_settled`). C'est exactement ce
  que `cancel_settlement_in_tx` sait défaire.
- **`cancel_settlement_in_tx`** (`invoice_settlements_write.rs:353`) : verrou facture → ligne →
  écriture + exercice → rangs 1-2 → `reverse_owned_in_tx(ClientSettlement)` → retrait → `paid_at`
  → audit. ⚠️ Il **refuse** au rang 1 **après** que le dé-rapprochement a déjà défait le lien : ce
  n'est pas un défaut, c'est la transaction qui le rattrape (AC 12, composition).
- **Socle** : `reverse_in_tx` (`journal_entries.rs:1438`), `reverse_owned_in_tx` (`:1500`),
  `reversal_blockers` (`:1265`, sous-requête `bank_transaction_id` en `LIMIT 1`, `:1309`).
- **Queue commune** : `settlement_cancellation::settlement_entry_cancel_blocker` — rang 2 lu par
  jointure `fiscal_years`, rangs 3-4 par `reversal_blockers`, rang 5 par
  `fiscal_years::has_open_covering_date`.
- **Verrou de compte** : `kesh_reconciliation::mutex::with_account_lock` ; patron d'appel et de
  mapping d'erreurs : `post_manual` (`routes/reconciliation.rs:2874-3010`). ⚠️ Le verrou est
  relâché **avant** le `COMMIT` (comportement existant des quatre routes, non modifié ici) : la
  garde de version et le verrou de ligne de l'AC 1 tiennent la fenêtre.
- **Rechargement de transaction** : `reconciliation::find_pending_by_id_for_account` ne filtre
  **pas** le statut malgré son nom (dette `dette-naming-reconciliation-helpers`) ; le geste écrit
  sa propre lecture `FOR UPDATE`.
- **Détail d'import** : `routes/bank_imports.rs::detail` (`:1278`), `TransactionResponse`.
- **Erreurs** : `DbError::ReconciliationNotCancellable { blocker }` (**neuve**, AC 3) → 409 ;
  `DbError::SettlementNotCancellable { blocker }` → 409 avec `blocker.code()` (rang 1, via
  `cancel_settlement_in_tx`) ;
  `EntryNotReversable` → 409 + `details` ; `ReversalAccountsArchived` → 400 + `details.rejected[]` ;
  `FiscalYearInvalid`, `PeriodLocked` → 400 ; `OptimisticLockConflict` → 409.
- **i18n** : `lint-i18n-ownership` ne balaie que `frontend/src/lib/features/` ; une clé
  `reconciliation-*` y vit dans `features/reconciliation/`.

### Pièges nommés d'avance

1. **Contre-passer avant de défaire le lien** : le socle refuse (`MATCHED_BANK_TRANSACTION`) —
   l'ordre de l'AC 1 est porteur.
2. **Supprimer l'écriture** : le lien disparaît en silence (fait 3).
3. **L'exemption large** : lever le rang 3 pour toute transaction masquerait une seconde
   transaction liée à la même écriture — **muet**, seul un état forgé le révèle.
4. **Oublier `auto_match_rejected_at`** : la transaction ne revient jamais dans les propositions
   (fait 7) — le test de l'AC 12 lit les propositions, pas la colonne.
5. **Un ordre de verrous différent** de `cancel_settlement_in_tx` : interblocage avec une annulation
   de règlement lancée depuis la fiche facture.
6. **Contre-passer un lien hérité** : ce serait annuler la **vente** (fait 4).
6-bis. ⚠️ **Interblocage possible — tranché : REJEU** (passe 1 de validation). `accept_one_invoice` verrouille
   l'**exercice du jour** (`find_open_covering_date … FOR UPDATE`, `:1350`) **avant** la facture
   (`UPDATE invoices`, `:1498`) ; le dé-rapprochement d'un règlement verrouille la **facture** puis,
   par le socle, l'exercice du jour. Sur **deux comptes bancaires différents** (verrous nommés
   distincts), une acceptation sur la facture X et le dé-rapprochement d'un autre règlement de X
   peuvent s'interbloquer : InnoDB en tue un (erreur 1213), rendu aujourd'hui en 500.
   `cancel_settlement_in_tx` (25-3-a-1, route de la fiche facture) porte **déjà** le même ordre :
   le risque est antérieur, cette story l'étend. **Retenu** : le rejeu par
   `kesh_db::retry::retry_with` (AC 6), mécanisme **éprouvé** du dépôt (`onboarding.rs:596-621`,
   son test d'intégration `retry.rs::integration_real_deadlock_recovers_via_retry`), qui absorbe
   un événement transitoire sans le montrer à l'utilisateur. ⛔ **Écartés** : réordonner les
   verrous toucherait `accept_one_invoice` et `cancel_settlement_in_tx`, hors périmètre ; un 409
   « réessayez » exposerait l'accident. ⚠️ **Limite assumée** : pas de test d'interblocage réel
   pour cette route (non déterministe, et aucun point d'injection d'une erreur 1213) — la
   couverture du mécanisme est celle de `retry.rs` ; l'enveloppement se vérifie **en revue**, au
   Dev Agent Record. ⚠️ **Le même risque subsiste** sur la route d'annulation de
   règlement de la 25-3-a-1 (`cancel_settlement_in_tx` appelé sans rejeu) : **à signaler à Guy**,
   hors périmètre.
7. Une réponse qui mêle deux lectures (AC 7) ; des totaux incrémentés au lieu d'être recomptés ; un
   motif corrigé à un site sur quatre.

### Faits voisins, signalés à Guy, hors périmètre

- **Échéance par défaut** (`routes/invoices.rs:667-683`) : sans date limite saisie, l'échéance
  est `date + délai du contact`, **sinon la date de la facture elle-même** — aucun délai d'usage
  au niveau de la société.
- **Fenêtre de l'acceptation automatique** (`reconciliation.rs:55`, `:1180`) : un paiement n'est
  accepté contre une facture que s'il arrive au plus **30 jours après sa DATE** — l'échéance n'y
  entre pas.

### Dette antérieure constatée, hors périmètre

La page de détail d'un import est **entièrement en libellés en dur** (« Détail import bancaire »,
« Transactions », en-têtes de colonnes, statut brut `reconciled`/`pending`). Cette story n'ajoute
que des libellés traduits ; migrer la page relève d'une issue à ouvrir (à proposer à Guy).

### Règle de splitting

Modules : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — **quatre**, sous le seuil. ⚠️ Signal à
surveiller en validation : si la sévérité ne décroît pas d'une passe à l'autre, sortir les écrans
(AC 10-11) en story propre — ils sont la partie la plus séparable.

### References

- [Source: crates/kesh-api/src/routes/reconciliation.rs:1056-1603] — `accept_one_invoice`.
- [Source: crates/kesh-api/src/routes/reconciliation.rs:1892,2256,2948,3409] — les quatre autres sites.
- [Source: crates/kesh-reconciliation/src/mutex.rs:66] — `with_account_lock`.
- [Source: crates/kesh-db/src/repositories/invoice_settlements_write.rs:297-353] — tête client, geste.
- [Source: crates/kesh-db/src/repositories/settlement_cancellation.rs] — la queue commune.
- [Source: crates/kesh-db/src/repositories/journal_entries.rs:1265-1509] — socle.
- [Source: crates/kesh-db/src/errors.rs:57-100,196-240] — `ReversalBlocker`, `SettlementCancelBlocker`.
- [Source: crates/kesh-db/migrations/20260504000001_bank_imports.sql:83] — `ON DELETE SET NULL`.
- [Source: crates/kesh-db/src/repositories/invoices.rs:1384,1496-1512] — dévalidation, lien hérité.
- [Source: frontend/src/routes/(app)/bank-import/[id]/+page.svelte] — le seul écran des transactions rapprochées.
- [Source: docs/manual/fr/user-manual.tex:970,1102,1309,1836-1848] — ce que le manuel dit.
- [Source: _bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md] — patron, leçons.

## Questions pour Guy

Toutes tranchées le 2026-09-25 (cf. « Arbitrages de Guy sur cette fiche »).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-25 | validate P3 | **Passe ciblée** : une lentille **Opus** en contexte frais, sur la seule remédiation de P2 (72 lignes aplaties), prompt versionné `25-3-b-validate-prompt-p3.md`, axes déclarés (non exercés : `cargo check`, frontend inexistant, 1213 sur `GET_LOCK` supposé impossible). **Vérifié juste** : `map_db_error` laisse un 1213 en `DbError::Sqlx` — le rejeu n'est pas muet ; chaîne de mapping exacte ; aucun état partiel entre tentatives ; `.entry.id` exact. **2 MEDIUM, 2 LOW, corrigés** : ① les codes **hors motifs** (`PERIOD_LOCKED`, `OPTIMISTIC_LOCK_CONFLICT`, verrou de compte, `NOT_FOUND`) n'avaient pas de destination dans le dialogue ⇒ message serveur, garde `never` sur l'union des six seulement, test ; ② (hors diff) `actor_api_key_id` absent de la signature du geste alors que l'AC 6 admet les clés ⇒ ajouté, audit par `for_actor` ; limite assumée : les lignes d'audit de la sœur et du socle ne portent pas la clé — **défaut antérieur de la 25-3-a-1, signalé à Guy** ; ③ pseudo-code rendu compilable (`async move` + clones ; `async |tx| Ok(…?)`) ; ④ « relâché à chaque sortie » nuancé — un verrou nommé est de session, fuite possible si `RELEASE_LOCK` échoue, limite assumée antérieure. **Trend** : P1 2 HIGH / 3 MED / 2 LOW → P2 2 MED / 1 LOW → P3 2 MED / 2 LOW. ⚠️ Le correctif ② touche un AC (signature) et non le seul diff de P2 : la passe suivante doit rester ciblée **sur cette remédiation**. |
| 2026-09-25 | validate P2 | **Deux lentilles Haiku 4.5** en contexte frais, diff de la remédiation **aplati**, prompt versionné `25-3-b-validate-prompt-p2.md`, axes déclarés. Lentille A (régressions de P1) : 1 LOW (type rendu par `reverse_in_tx` → `.entry.id`, précisé). Lentille B (à froid) : 1 HIGH **reclassé MEDIUM** — pas une contradiction mais une ambiguïté : le dialogue traduit le **code** par sa propre famille pour les six codes ; seul le repli serveur du rang 1 reste celui du règlement, et il est juste ⇒ « qui affiche quoi » écrit à l'AC 9 ; 1 MEDIUM — l'emboîtement rejeu / transaction / verrou n'était pas écrit ⇒ pseudo-code à l'AC 6. ⚠️ **Axes laissés par la lentille A et repris par l'orchestrateur** : rejouer une closure qui prend un verrou nommé est sûr (relâché à chaque sortie, transaction neuve à chaque tentative) ; ⛔ le prédicat de `retry_with` doit porter sur `AppError`, et la chaîne de mapping **préserver** `DbError::Sqlx` — vérifié sur `post_manual:3046-3052` ; écrit à l'AC 6 (un mapping qui masquerait le 1213 rendrait le rejeu muet). **Trend** : P1 2 HIGH / 3 MED / 2 LOW → P2 2 MED / 1 LOW. |
| 2026-09-25 | validate P1 | **Deux lentilles Sonnet** en contexte frais, prompt versionné `25-3-b-validate-prompt-p1.md`, axes déclarés par chacune. Lentille A : les faits 1-7 et ~25 citations `fichier:ligne` **tous exacts**, registre recompté exact ; **2 MEDIUM, 1 LOW**. Lentille B : **2 HIGH, 1 MEDIUM, 1 LOW**. Corrigés : ① (B, HIGH) le refus propre du geste passait par `SettlementNotCancellable`, dont le texte dit « ce règlement » — faux pour un éclatement ⇒ variante neuve `DbError::ReconciliationNotCancellable`, mappée vers `reconciliation-cancel-blocked-*`, test du texte ; ② (B, HIGH) l'exemption étroite ne pouvait pas s'appuyer sur le `LIMIT 1` sans `ORDER BY` de `reversal_blockers` ⇒ requête dédiée prescrite (`id <> ?`), test aux deux ordres d'insertion ; ③ (A, MEDIUM) l'étape 4 ne disait pas d'appeler la forme **exemptée** ⇒ écrit ; ④ (A, MEDIUM) le piège d'interblocage ignorait le patron du dépôt ⇒ **tranché : rejeu** par `kesh_db::retry::retry_with` (patron `onboarding.rs`, KF-002-H-002) ; ⚠️ un test prescrit par ce correctif (« 1213 forgé ») a été **retiré avant commit** : aucun point d'injection, limite assumée, vérification en revue ; ⑤ (B, MEDIUM) contrat du dialogue partagé écrit (props, qui lit, qui relit) ; ⑥ (A, LOW) en-tête du registre de routes déjà faux (106/109) — à corriger au passage ; ⑦ (B, LOW) ligne du README nommée juste. Reformulé (B, info) : les sœurs gardent comportement et tests, leur code reçoit un argument. ⚠️ **À signaler à Guy** : la route d'annulation de règlement de la 25-3-a-1 porte le même risque d'interblocage, sans rejeu. |
| 2026-09-25 | arbitrages (2) | **Q2 précisée** par Guy : la **date** de la facture compte (échéance à défaut de date limite) — seul son **exercice** n'arrête pas le rapprochement ; deux faits voisins relevés et signalés (échéance par défaut = date de facture sans délai de contact ; fenêtre d'acceptation bornée sur la date, non sur l'échéance), hors périmètre. **Q3 : les deux boutons** (Guy, « ok »). Toutes les questions sont tranchées. |
| 2026-09-25 | arbitrages | **Q1** (Guy) : « kesh n'est pas encore en production : il n'y a aucune donnée à préserver » ⇒ liens hérités (vente) et orphelins sans chemin dédié ; le socle refuse le premier, `Invariant` le second (AC 5 réécrit, un texte de refus laissé tel quel). **Q2** (Guy) : une facture d'un exercice clos, payée dans le suivant, doit pouvoir être rapprochée ⇒ le seul exercice qui compte est celui de l'écriture **de rapprochement** — déjà le cas à l'acceptation (`reconciliation.rs:1349-1350`) et dans la queue commune ; fixé par un test et une mutation (AC 12). **Q3** reformulée, en attente. |
| 2026-09-25 | spec | Spécifiée (Opus 5.5) sur les deux sœurs mergées. **Faits établis depuis la source** : cinq sites posent `matched_entry_id`, deux familles d'écritures seulement (règlement client par `accept_one_invoice`, écriture propre pour les quatre autres) ; défaire le lien **avant** la contre-passation lève le rang 3 sans autorité nouvelle ; aucune contrainte ne lie statut et lien (orphelins possibles) ; ⛔ les rapprochements **antérieurs à la 24-2** pointent sur l'écriture de **vente** et n'ont jamais été repris ; le seul écran des transactions rapprochées est le détail d'import ; le marqueur de rejet doit être remis à NULL. Trois positions soumises à Guy (Q1-Q3). Ultimate context engine analysis completed - comprehensive developer guide created. |
