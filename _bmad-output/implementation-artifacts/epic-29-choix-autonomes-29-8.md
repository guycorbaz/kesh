# Epic 29 — registre des choix autonomes de la Story 29-8

*Registre propre à la story, à verser **par union** au registre de l'epic (`epic-29-choix-autonomes.md`). Fiche :
`29-8-transaction-reglee-ailleurs.md`. Issue : #670. Relevés sur `fcd2ae75`.*

---

### C-29-8-1 — Les prémisses de #670 sont exactes

- **Contexte** : #670 affirme qu'une transaction réglée depuis la fiche, puis rejetée, reste `pending` et bloque à
  jamais la clôture du compte bancaire.
- **Retenu** : établies au code une à une (fiche, § *Les prémisses*) — toutes exactes. Aucune correction à porter à
  l'issue.
- **Réversible** : sans objet.

### C-29-8-2 — Aucune migration : `matched_entry_id` et `reconciled` suffisent

- **Contexte** : rattacher demande un lien transaction → écriture.
- **Retenu** : la colonne `bank_transactions.matched_entry_id` et le statut `reconciled` existent depuis la Story 8-1b ;
  le rattachement les pose comme les cinq autres chemins.
- **Écartée** : une colonne d'origine du lien (« rattachée » contre « créée ») — elle seule permettrait un « détacher
  sans annuler » (L1), au prix d'une migration ; hors périmètre minimal.
- **Réversible** : oui (une migration ADD COLUMN nullable plus tard).

### C-29-8-3 — Piste (a), restreinte aux règlements clients par virement

- **Contexte** : #670 propose (a) rattacher, (b) choisir une facture à la réconciliation (#666), (c) un statut
  « écartée ».
- **Retenu** : (a), restreinte aux lignes `invoice_settlements` de type `bank_transfer` : l'état produit est celui
  d'une proposition « facture » acceptée, que les gardes en aval qui lisent le lien traitent déjà (rang 3 de
  l'annulation, par le socle de la contre-passation ; suppression d'import ; *Annuler le rapprochement*).
  *Amendé en validation P1 (R2/F4)* : la dévalidation ne lit pas ce lien — son motif 5 vise l'écriture de vente ; elle
  est refusée au motif 1 (`Settled`), avant comme après le rattachement.
- **Écartées** : (b) ne couvre pas la 29-6 (trop-perçu refusé par `classify_payment`) ni les règlements déjà saisis,
  et exige une seconde borne (#548) ; (c) migration breaking et faux rattachement **muet** (une transaction jamais
  comptabilisée sortirait des contrôles de clôture) ; (a) généralisée à toute écriture — *Annuler le rapprochement*
  contre-passerait une écriture saisie au journal (`ReconciliationKind::Entry`) : il faudrait d'abord C-29-8-2
  écartée.
- **Réversible** : oui (élargir plus tard, après une colonne d'origine).

### C-29-8-4 — Les quatre conditions du rattachement

- **Contexte** : un faux rattachement doit être impossible ou visible.
- **Retenu** : transaction `pending` crédit CHF ; règlement de la société, `bank_transfer`, **même compte bancaire** ;
  écriture rapprochée d'aucune transaction ; `M > 0` et écart `0 ≤ t.amount − M ≤ 0,05`, `M` lu sur le compte du grand
  livre actuel du compte bancaire (*`M > 0` ajouté en validation P1*, C-29-8-18).
- **Écartée** : comparer `invoice_settlements.amount` (montant réglé, qui peut différer du payé d'un demi-centime
  d'arrondi) — la ligne de banque est l'argent.
- **Réversible** : oui.

### C-29-8-5 — Un écart de 0 à 5 centimes, dans un seul sens, affiché

- **Contexte** : le contournement de la 29-6 enregistre le **montant dû** ; la transaction le dépasse de 1 à
  5 centimes. Un rattachement à montant strictement égal laisserait ce cas bloqué.
- **Retenu** : `0 ≤ écart ≤ AMOUNT_TOLERANCE_HUNDREDTHS` (constante existante) ; l'écart s'affiche avant confirmation
  et s'écrit à l'audit. Négatif exclu (argent aux livres que la banque n'a pas reçu).
- **Écartée** : égalité stricte (29-6 non couverte) ; tolérance symétrique (aucun contournement ne la produit).
- **Réversible** : oui.

### C-29-8-6 — Pas de fenêtre de dates : la proximité trie et s'affiche

- **Contexte** : « dates proches — à établir ».
- **Retenu** : le dialogue de règlement préremplit la date au jour (`SettleInvoiceDialog.svelte:76-90`) et Guy importe
  un relevé par mois : une borne refuserait le cas nominal. Tri par écart puis jours d'écart ; colonne « jours ».
- **Écartée** : une borne de ± 30 ou ± 60 jours.
- **Réversible** : oui.

### C-29-8-7 — Un geste humain, sans présélection, absent des propositions

- **Contexte** : règle *Un appariement automatique propose, il ne crée jamais*.
- **Retenu** : liste de candidats, aucun présélectionné, confirmation explicite ; `GET /reconciliation/proposals`
  inchangé.
- **Écartée** : proposer le rattachement parmi les candidates de la liste (nouveau type de candidate, acceptation en
  lot).
- **Réversible** : oui.

### C-29-8-8 — Deux routes unitaires, sous `/reconciliation/transactions/{id}/`

- **Contexte** : forme de l'API.
- **Retenu** : `GET …/link-candidates`, `POST …/link` `{ settlementId }` ; le compte bancaire se lit sur la
  transaction (comme `…/cancel` et `…/unreject`). Pas un lot : le pattern batch ne s'applique pas.
- **Écartée** : un corps `{ bankAccountId, bankTransactionId, … }` comme `/manual` (redondant avec l'identifiant de
  chemin).
- **Réversible** : oui avant publication.

### C-29-8-9 — Une seule définition du « rattachable »

- **Contexte** : la liste et le refus ne doivent pas diverger.
- **Retenu** : condition 1 en fonction pure de `kesh-db` ; conditions 2 à 4 en **une** requête, rejouée par le `POST`
  restreinte à `s.id`. Mutations M1 à M19 (M14 à M19 ajoutées en validation P1).
- **Écartée** : un classifieur Rust des motifs à côté de la requête (deux définitions).
- **Réversible** : oui.

### C-29-8-10 — Un seul code de refus, `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE`

- **Contexte** : motifs multiples (compte, montant, déjà rapproché, transaction non créditrice).
- **Retenu** : un code, `details.bankTransactionId` et `details.settlementId` ; l'écran n'offre que des candidats, le
  refus ne se voit que par une course ou un appel direct — conduite : relire la liste. Les 404 restent distincts
  (anti-énumération).
- **Écartée** : un `details.reason` par condition — il exigerait le classifieur écarté en C-29-8-9.
- **Réversible** : oui (ajouter un `reason` est compatible).

### C-29-8-11 — Rejouée sur interblocage, classée `SansEcritureAuJournal`

- **Contexte** : la route prend des verrous de ligne sur `invoice_settlements` et `journal_entries`.
- **Retenu** : enveloppe `retry_app_on_deadlock`, comme `post_manual` ; registre `Traced, SansEcritureAuJournal` (le
  registre n'interdit pas l'enveloppe, point (vi)).
- **Écartée** : non rejouée comme `…/unreject` (qui ne verrouille que sa transaction).
- **Réversible** : oui.

### C-29-8-12 — Ordre des verrous imposé, vue née après le dernier

- **Contexte** : B5 de la rétrospective 28 (naissance de la vue).
- **Retenu** (*amendé en validation P1, F1/F6*) : verrou nommé → `bank_transactions` (`t`) → `invoice_settlements`
  (`s`) → `journal_entries` (`je`), en **deux instructions** pour `s` puis `je` (l'ordre est écrit, non laissé à
  l'optimiseur d'un `JOIN`), sans l'exercice → `ensure_open_in_tx` (première lecture ordinaire) → requête des
  conditions. Ordre `s → je` aligné sur `cancel_settlement_in_tx` ; les lignes `invoices`, `bank_accounts` et
  `fiscal_years` ne sont jamais prises. Table *Absence de cycle* aux Dev Notes, à recopier au Dev Agent Record.
- **Écartée** : la lecture courante `matched_entry_id = ? FOR UPDATE` de la première rédaction (C-29-8-17).
- **Réversible** : oui.

### C-29-8-13 — *Annuler le rapprochement* d'une transaction rattachée annule le règlement (L1)

- **Contexte** : sans colonne d'origine (C-29-8-2), l'état est indiscernable d'une proposition acceptée.
- **Retenu** : comportement existant gardé et écrit au manuel ; visible (contre-passation, facture rouverte),
  réparable, sans erreur comptable. Évolution tracée par **#672** (« détacher sans annuler »).
- **Écartée** : refuser l'annulation d'une transaction rattachée (indiscernable sans migration).
- **Réversible** : oui.

### C-29-8-14 — Le bouton sur la ligne des propositions, pas au détail de l'import

- **Contexte** : où offrir le geste.
- **Retenu** : bouton de ligne *Rattacher à un règlement* sur les lignes crédit CHF de `ReconciliationProposals` ;
  une transaction rejetée passe d'abord par *Annuler le rejet* (l'API admet une rejetée, comme `/manual`).
- **Écartée** : un second point d'entrée au détail de l'import (deux écrans, deux tests E2E).
- **Réversible** : oui.

### C-29-8-15 — « Rattachez » remplace « rejetez » dans les contournements de la 29-2 et de la 29-6

- **Contexte** : les fiches 29-2 (C-29-2-8) et 29-6 (AC7) prescrivent « régler depuis la fiche, puis rejeter » ;
  même release v0.13.4.
- **Retenu** : la 29-8, développée après elles, réécrit ces phrases (manuel, CHANGELOG, `api-external.md` s'il y a
  lieu) et retire les incises « (#670) » devenues fausses. L'indication de l'écran de la 29-6 reste (elle parle des
  boutons de lot).
- **Écartée** : laisser les deux gestes au manuel (le rejet ne règle pas #670).
- **Réversible** : oui.

### C-29-8-16 — Pas de découpage ; inventaire clos des textes « cinq chemins »

- **Contexte** : règle de découpage (> 5 modules).
- **Retenu** : quatre modules de code (`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend/…/reconciliation`) plus la
  documentation : une seule story. Les cinq textes qui supposent « une écriture créée par le rapprochement » sont
  inventoriés (AC7) et réécrits.
- **Réversible** : oui.

---

*Choix C-29-8-17 à C-29-8-24 : remédiation de la validation P1 (2026-10-14), lentilles R (0/0/4/7) et F (0/1/3/5).*

### C-29-8-17 — Aucune lecture verrouillante sur `matched_entry_id` ; l'invariant tient par le verrou du règlement

- **Contexte** : l'étape 4.3 d'origine (`SELECT … WHERE matched_entry_id = ? FOR UPDATE`) devait lire « au courant »
  la condition 3. Validation P1, F1 (HIGH) : sur un index secondaire non unique, un `FOR UPDATE` sans ligne trouvée
  pose un verrou d'intervalle ; deux rattachements simultanés dont les écritures tombent dans l'intervalle terminal
  s'interbloquent à l'`UPDATE` (verrou d'insertion). Et elle est redondante.
- **Retenu** : étape retirée. L'invariant « une écriture, une transaction » se tient par la lecture ordinaire des
  conditions **sous le verrou de ligne de `s`** : deux rattachements du même règlement visent deux transactions du
  même compte (condition 2) ; le second attend `s` jusqu'au **commit** du premier (le verrou nommé, lui, est relâché
  avant le commit, `mutex.rs:126-132` et `conclude_locked_attempt`) ; sa vue naît après l'attente et voit le lien.
  Les cinq autres chemins ne lient que des écritures qu'ils créent ; le seul autre écrivain de la colonne la remet à
  `NULL` sous le même verrou de `s`. `GET_LOCK` ne fait pas naître la vue (aucune table lue ; fiche 28-4:961). Test
  T17 (course de double rattachement), mutation M14 (requête des conditions avant le verrou de `s`).
- **Écartées** : garder la lecture en la forçant sur l'index (`FORCE INDEX`) avec un test `EXPLAIN` — le cycle
  d'intervalle demeure ; une contrainte `UNIQUE (matched_entry_id)` — migration, et hors périmètre minimal.
- **Réversible** : oui.

### C-29-8-18 — `M > 0` dans la condition 4

- **Contexte** : validation P1, R3 = F2 : avec `M = 0` (règlement sur un ancien compte lié), tout crédit de 1 à
  5 centimes rendait le règlement candidat à « 0.00 ».
- **Retenu** : `M > 0` (un règlement par virement débite toujours la banque, `settle_invoice` refuse `amount <= 0`) ;
  table dérivée par écriture en jointure interne, `HAVING M > 0`. Test T18 (ancien compte lié, crédit de 0.03 et
  crédit égal), mutations M15 et M16.
- **Écartée** : `COALESCE(…, 0)` sans condition (laisse passer `M = 0`).
- **Réversible** : oui.

### C-29-8-19 — `ensure_open_in_tx` dans la transaction, après le dernier verrou

- **Contexte** : validation P1, R9/F8 : un pré-vol seul se classe `Invariant("I1", …)`, non « gardé » ; le
  précédent `unreject_once` lit la garde dans la transaction.
- **Retenu** : pré-vol **et** `bank_accounts::ensure_open_in_tx` à l'étape 4.3, première lecture ordinaire, après le
  verrou de `je` ; statut `Gardee` au `REGISTRE` de `bank_account_closed_inventory.rs`. Test T13 (b) (clôture posée
  pendant l'attente du verrou nommé), mutation M19. Zone partagée avec la 29-4 (`FENETRE`) : la seconde mergée ajoute
  la ligne.
- **Écartée** : pré-vol seul classé `Invariant("I1", …)` — sûr (I1 tient), mais `link_in_tx` serait la seule
  fonction de rapprochement sans garde en transaction parmi celles qui en ont une voisine.
- **Réversible** : oui.

### C-29-8-20 — Compte bancaire archivé refusé ; compte du grand livre inactif admis

- **Contexte** : validation P1, R10.5 : `post_manual` refuse les deux ; la fiche n'en disait rien.
- **Retenu** : archivé → `AppError::BankAccountNotFound` au `GET` et au `POST`, comme `post_manual` ; compte du
  grand livre inactif **admis** (le rattachement n'y écrit rien). Test T20.
- **Écartée** : refuser l'inactif (`412`) — il protège une écriture que le rattachement ne passe pas.
- **Réversible** : oui.

### C-29-8-21 — Exercice clos ou période verrouillée : le rattachement passe

- **Contexte** : validation P1, R10.4.
- **Retenu** : admis (aucune écriture au journal ; le verrou de `je` ne prend pas l'exercice) ; la conséquence —
  *Annuler le rapprochement* ensuite refusé `FISCAL_YEAR_CLOSED` — écrite au manuel. Test T19.
- **Écartée** : refuser (un paiement de décembre rapproché en janvier, une fois décembre clos, resterait bloqué).
- **Réversible** : oui.

### C-29-8-22 — Sérialisation, tolérance, date de paiement : une seule définition chacune

- **Contexte** : validation P1, R10.1 à R10.3 et F9 (types).
- **Retenu** : conditions calculées en décimal exact dans la requête ; sortie JSON et audit `rescale(2)` (patron
  `routes/reconciliation.rs:4274-4283`) ; tolérance liée en `Decimal`, passée en paramètre à `kesh-db` (patron
  `find_unpaid_invoices_for_window`), la constante restant privée à `routes/reconciliation.rs` ; `payment_date(t)`
  calculée en Rust et liée en paramètre, la requête triant sur `ABS(DATEDIFF(s.settled_on, ?))`.
- **Écartées** : rendre la constante publique dans `kesh-db` (deux crates la verraient, une seule la définit
  aujourd'hui) ; réécrire `COALESCE(value_date, booking_date)` en SQL (seconde définition de la date de paiement) ;
  trier en Rust après une requête sans `LIMIT`.
- **Réversible** : oui.

### C-29-8-23 — Angle mort L6 : le paiement qui inclut les frais de rappel

- **Contexte** : validation P1, R4 : le manuel prescrit d'enregistrer le seul montant de la QR-facture quand le client
  paie « reste dû et frais » ; la transaction dépasse alors le règlement de plus de 5 centimes.
- **Retenu** : angle mort écrit (Dev Notes L6, manuel `:1308`, CHANGELOG), renvoyé à #671 (commentaire posé par
  l'orchestrateur) et à #401.
- **Écartée** : élargir la tolérance ou admettre tout dépassement — hors périmètre minimal, et un dépassement franc
  doit se justifier par une écriture que le rattachement ne passe pas.
- **Réversible** : oui.

### C-29-8-24 — Un refus `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` recharge la liste de la modal

- **Contexte** : validation P1, R11 : le message dit « relire la liste », la modal ouverte ne la rechargeait pas.
- **Retenu** : sur ce `409`, la modal recharge `getLinkCandidates` et remet le choix à vide ; l'échec du chargement
  s'affiche. Vitest.
- **Écartée** : fermer la modal (l'utilisateur perdrait le contexte de la transaction).
- **Réversible** : oui.
