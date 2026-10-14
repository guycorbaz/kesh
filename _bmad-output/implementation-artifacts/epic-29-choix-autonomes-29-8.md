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
  *Annoté le 2026-10-14 (validation P2, R3) : la 29-6, à sa validation P2, a réécrit elle-même la FAQ (« Kesh ne relie
  pas la transaction … ; rejetez-la »), complété `:2050` et rédigé l'indication « ne peut pas être acceptée » ; au
  manuel, ses incises ne portent **aucun** « 670 » (C-29-6-17) — seule son entrée de CHANGELOG le porte. La fiche 29-8
  (AC6) cite désormais ces textes tels qu'ils sont ; le retrait des incises vaut pour l'incise « rejetée, elle reste en
  attente … » de `:2029` et de la FAQ, et pour « ([#670]) » de l'entrée de CHANGELOG de la 29-6.*
  *Annoté le 2026-10-14 (validation P3, R1) : la fiche ne recopie plus les phrases prescrites par les fiches voisines
  (elles bougent : la 29-2 a supprimé la phrase de `:1305`, et réécrit `:2031`, `CHANGELOG.md:15`; la 29-5 prescrit le
  même conseil, `29-5-departage-exigibilite.md:326-327`). L'inventaire des conseils de rejet se fait au rebase, par le
  symptôme — voir C-29-8-29.*
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
  *Annoté le 2026-10-14 (validation P2, R1 = F1 ; C-29-8-25) : T17 n'observe plus l'attente — il n'asserte que le
  résultat d'une course ; l'ordre d'acquisition et la naissance de la vue sont renvoyés à la revue de code, et M14 n'est
  tuée que si les deux requêtes se chevauchent.*
- **Écartées** : garder la lecture en la forçant sur l'index (`FORCE INDEX`) avec un test `EXPLAIN` — le cycle
  d'intervalle demeure ; une contrainte `UNIQUE (matched_entry_id)` — migration, et hors périmètre minimal.
- **Réversible** : oui.

### C-29-8-18 — `M > 0` dans la condition 4

- **Contexte** : validation P1, R3 = F2 : avec `M = 0` (règlement sur un ancien compte lié), tout crédit de 1 à
  5 centimes rendait le règlement candidat à « 0.00 ».
- **Retenu** : `M > 0` (un règlement par virement débite toujours la banque, `settle_invoice` refuse `amount <= 0`) ;
  table dérivée par écriture en jointure interne, `HAVING M > 0`. Test T18 (ancien compte lié, crédit de 0.03 et
  crédit égal), mutations M15 et M16.
  *Annoté le 2026-10-14 (validation P2, R2 = F2) : dans la forme retenue, c'est la **jointure interne** qui écarte
  l'écriture sans ligne sur le compte lié ; le `HAVING M > 0` ne décide que d'une somme ≤ 0, qu'aucun état atteignable
  ne produit — M15 est déclarée **équivalente** (fiche, AC8), non tuée par T18 (a).*
  *Annoté le 2026-10-14 (validation P3, R4 = F1) : la raison est rectifiée. « Aucun état atteignable » était faux à la
  lettre — le refus du lien de la banque à un compte de créance (`repositories/bank_accounts.rs:1273-1288`, et non
  `routes/bank_accounts.rs:426-433`, qui est le contrôle du type de compte) ne compare qu'aux comptes de créance des
  réglages **actuels** : le compte de créance d'une vente antérieure peut être lié, `M < 0`. Le `HAVING` ne le distingue
  que d'un règlement de moins de 5 centimes (`écart > 0,05` sinon) : M15 est une **survivante assumée**, équivalente
  sauf au centime, sans test.*
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

---

*Choix C-29-8-25 à C-29-8-28 : remédiation de la validation P2 (2026-10-14), lentilles R (0/0/5/5) et F (0/0/5/6). Les
trois premiers sont des **décisions de l'orchestrateur** (« retirer plutôt qu'ajouter »), consignées comme telles.*

### C-29-8-25 — L'observation de l'attente est retirée de T16 et T17 *(décision de l'orchestrateur)*

- **Contexte** : validation P2, R1 = F1 (MEDIUM, né de la remédiation P1) : T17 prescrivait d'observer le `POST`
  bloqué sur le verrou de ligne par `information_schema.INNODB_TRX`, que l'utilisateur de dev (`kesh`, sans privilège
  `PROCESS`, `scripts/mariadb-init/01-dev-grants.sql`) ne peut pas lire ; T16 renvoyait à la même observation.
- **Retenu** : **retirer** l'observation, ne la remplacer par **aucune** autre. T17 asserte le seul résultat d'une
  course réelle (deux rattachements concurrents du même règlement, `tokio::join!` : exactement un `200`, l'autre
  `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, un seul lien en base) ; T16 de même, entre l'annulation du règlement
  depuis la fiche et le rattachement (exactement l'un réussit, état final cohérent). Ce qu'ils ne prouvent pas —
  l'ordre d'acquisition, l'attente du perdant, le moment où naît la vue — est écrit à la fiche et **renvoyé à la revue
  de code** ; M10 et M14 ne sont tuées que si les requêtes se chevauchent (une survie se déclare au Dev Agent Record).
- **Écartées** : la sonde `attendre_une_requete_en_cours` sur la requête de l'étape 4.2 (a) (proposée par R1 et F1) —
  une autre forme d'observation, que l'orchestrateur a exclue ; garder `INNODB_TRX` (rouge permanent sur le poste de
  dev).
- **Réversible** : oui (une observation peut être ajoutée par une story ultérieure).

### C-29-8-26 — T15 ne clôt pas le compte ; la clôture est assertée par T1 *(décision de l'orchestrateur)*

- **Contexte** : validation P2, R4 = F4 : un compte clos fait rendre `409 BANK_ACCOUNT_CLOSED` à toute annulation qui
  suit, avant ses propres motifs (`reconciliation_cancel.rs:325`, `invoice_settlements_write.rs:896`) ; un compte
  principal ne se clôt pas (`bank_accounts.rs:1016-1019`) ; `…/cancel` supprime le règlement
  (`invoice_settlements_write.rs:946`).
- **Retenu** : T15 opère sur un compte **non principal** qu'il ne clôt pas, assertions dans un ordre écrit (dévalidation
  avant ; rattachement ; refus `MATCHED_BANK_TRANSACTION` puis témoin annulé ; dévalidation après ; K2 ; `…/cancel` en
  dernier) ; le témoin est un acompte **par virement sur le même compte**. La clôture refusée puis réussie — le défaut
  de #670 — passe à **T1**, compte non principal.
- **Écartées** : clore puis rouvrir le compte dans T15 (deux gestes de plus, et la réouverture n'est pas l'objet) ;
  deux décors dans T15.
  *Annoté le 2026-10-14 (validation P3, F5) : l'exigence « compte non principal » de T15 est retirée — sa raison d'être
  (un principal ne se clôt pas) est passée à T1 ; T15 n'a plus à imposer un second compte bancaire.*
- **Réversible** : oui.

### C-29-8-27 — Le `GET` des candidats applique le pré-vol du `POST`

- **Contexte** : validation P2, R9 = F10 (LOW) : l'AC1 promettait « la liste n'offre jamais ce que le `POST`
  refuserait » sans prescrire au `GET` le compte clos ni le compte sans compte du grand livre lié.
- **Retenu** : mêmes fonctions de pré-vol au `GET` (archivé → `BankAccountNotFound`, clos → `409
  BANK_ACCOUNT_CLOSED`, non lié → `412 BANK_ACCOUNT_NOT_CONFIGURED`) ; T13 (a) asserte le `GET` aussi.
- **Écartée** : `200` et `[]` — la modal afficherait « Aucun règlement … » et tairait la cause.
  *Annoté le 2026-10-14 (validation P3, R9 = F6) : « mêmes fonctions » était inexact — aucune fonction existante ne
  rend les trois refus (`ensure_bank_account_open` ne rend que le clos, `_ => Ok(())` sinon). Le dev écrit **une**
  fonction privée `link_preflight`, appelée par les deux handlers, dans l'ordre archivé, clos, non lié, avant toute
  lecture de statut ; l'ordre des refus du `GET` est celui du `POST` (404 inexistante, archivé, clos, 412, puis 404
  rapprochée), faute de quoi un compte clos rendrait `404` au `GET` et `409` au `POST`.*
- **Réversible** : oui.

### C-29-8-28 — Angle mort L7 : règlement par compte interne sur le compte du grand livre de la banque *(décision de l'orchestrateur)*

- **Contexte** : validation P2, F3 : `settle_invoice` admet un règlement `internal_account` dont le compte est celui du
  grand livre de la banque ; son écriture débite la banque, mais il n'est pas un virement.
- **Retenu** : non rattachable (condition 2), angle mort **L7** écrit (Dev Notes, sous-section neuve du manuel,
  CHANGELOG), même remède que L5 (annuler et refaire par virement), versé à #671 par l'orchestrateur ; **aucun test
  neuf**. Dans le même mouvement, M2 est déclarée équivalente (contrainte `chk_invoice_settlements_counterparty`),
  M15 (jointure interne) et M17 (filtre société de l'étape 4.2 (a)) aussi, sans décor pour les tuer ; M1 reste tuée
  par T6, au prix d'une ligne de montage (compte du grand livre partagé).
- **Écartées** : admettre ce règlement au rattachement (élargit la condition 2 hors du périmètre minimal) ; construire
  des décors forgés en SQL pour tuer M2, M15, M17.
- **Réversible** : oui.

### C-29-8-29 — Inventaire des conseils de rejet par le symptôme, au rebase ; retrait du contrôle négatif « tout seul. » *(décision de l'orchestrateur)*

- **Contexte** : validation P3, R1 (MEDIUM) : le tableau de l'AC6 décrivait une 29-2 périmée (sa fiche réelle supprime
  la phrase de `:1305`, réécrit `:2031` en « en rejetant ensuite la transaction », et son entrée de CHANGELOG, ligne 15,
  dit « puis rejetez la transaction ») et ignorait la 29-5 (`29-5-departage-exigibilite.md:326-327`, même conseil).
  R2 (MEDIUM) : le contrôle négatif « tout seul. » = 0 contredisait `:1198`, qui garde la phrase.
- **Retenu** : la fiche cesse de recopier les phrases des fiches voisines. L'AC6 prescrit, à l'étape 0 et **après** rebase
  sur les 29-2, 29-5 et 29-6 mergées, `grep -nE 'rejet[a-z]*( ensuite)?( la transaction|-la)'
  docs/manual/fr/user-manual.tex CHANGELOG.md` et toute forme voisine ; chaque conseil de rejeter une transaction réglée
  depuis la fiche de la facture est réécrit en « rattachez », les autres occurrences sont triées et écrites. Le tableau du
  manuel ne garde que les sites propres à la 29-8 (glossaire, tuile d'écart, introduction, FAQ, `:1198`, `:1308`,
  `:2076`, `:2050`) et renvoie au grep pour `:2029` et `:1305`. La 29-5 entre aux zones partagées. Le contrôle de
  pré-release de la v0.13.4 (fichier d'epic) doit rendre **0 conseil de rejet** après la 29-8. Le contrôle négatif
  « tout seul. » est **retiré** (il rougirait sur une implémentation fidèle) ; il reste au relevé « ce qui doit encore y
  être ».
- **Écartées** : mettre à jour le tableau d'après la 29-2 réelle (il se périmerait au prochain commit d'une fiche
  voisine, ce que la validation P2 avait déjà payé pour la 29-6) ; durcir « tout seul. » en exigeant la suppression de
  la phrase (elle reste vraie).
- **Réversible** : oui.
