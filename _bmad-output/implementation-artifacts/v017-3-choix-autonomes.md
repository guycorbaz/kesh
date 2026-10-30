# Release 0.17.0 — registre des choix autonomes de la Story v017-3 (v017-3a et v017-3b)

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au merge. Fiches :
`v017-3a-rattacher-ecriture-existante.md` (#671) et `v017-3b-facture-fournisseur-au-centime.md` (#687). Numérotation
`C-v017-3-k`, commune aux deux sous-stories. Issues en `refs` seulement dans les commits. Les numéros de ligne cités
sont ceux de `c5101b07` (tête de la candidate `v0.16.1-rc1`, point de départ de toutes les branches de la 0.17.0).
Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Les choix marqués **(Q)** figurent aussi
parmi les questions à soumettre à Guy (fiches, § *Questions au Project Lead*).

Rédigé le 2026-10-30 en autonomie (Opus 5.5), sous la contrainte du Project Lead du même jour
(`kesh-gate-logs/prompts/v017-donnees-a-garder.md` : les données de la 0.16.1-rc1 se gardent ; aucune réécriture
silencieuse ; un appariement propose, il ne crée jamais).

---

### C-v017-3-1 — Découper : v017-3a (#671) et v017-3b (#687)

- **Contexte** : le mandat demande une story pour les deux issues, et autorise le découpage. #671 est un défaut de
  **réconciliation** (une transaction déjà comptabilisée par un autre chemin reste en attente) ; #687 est un défaut de
  **saisie d'une facture fournisseur** (un TTC sous le centime). Ensemble, elles touchent `kesh-db` (rapprochement **et**
  factures fournisseur), `kesh-api` (routes de réconciliation **et** de factures fournisseur, erreurs, registres
  d'audit), `kesh-core`, `kesh-i18n`, le `frontend` (réconciliation **et** factures fournisseur) et deux sections du
  manuel : plus de cinq modules métier de premier niveau (`CLAUDE.md` § *Règle de splitting préventif*, premier
  critère).
- **Retenu** : deux sous-stories, sans dépendance de code entre elles. **v017-3a** : rattacher une transaction en attente
  à une écriture existante du compte de la banque, et l'avertissement « semble déjà comptabilisée ». **v017-3b** : une
  facture fournisseur ne naît plus avec un total sous le centime ; une facture existante ainsi n'est pas modifiée, elle
  est signalée et son traitement est décrit. Lien fonctionnel, écrit dans les deux fiches : la règle de la 3a exige un
  montant **égal** ; un règlement fournisseur sous le centime (#687) n'y sera donc jamais candidat — c'est la 3b qui
  ferme la source, et le manuel de la 3a renvoie à la conduite de la 3b pour l'existant.
- **Écartées** : (b) une seule story — deux modèles mentaux adversariaux dans une passe, signal de non-convergence
  probable ; (c) traiter #687 en ajustant la règle de la 3a (comparer `ROUND(M, 2)`) — le compte de la banque des
  livres différerait alors du relevé de dixièmes de centime, en silence.
- **Réversible** : oui (organisation du travail).

### C-v017-3-2 — v017-3a : rattacher à une **écriture** existante, par un couple de routes neuf (Q)

- **Contexte** : #671 énumère les chemins qui comptabilisent un mouvement bancaire sans poser de lien : règlement
  fournisseur depuis la fiche (`supplier_invoices::pay_in_tx_as`, `supplier_invoices.rs:761-1018`, qui écrit `D 2000 /
  C banque` sans lire `bank_transactions`), écriture saisie au journal, règlement client par compte interne sur le
  compte du grand livre de la banque, lot pain.001, paiement groupé. La Story 29-8 a posé le seul geste qui lie une
  écriture **existante** (`reconciliation_link::link_in_tx`, `reconciliation_link.rs:329`), restreint aux règlements
  clients par virement. Précision du Project Lead sur #671 (2026-10-23) : avertir et **demander** — garder en attente,
  rejeter, rattacher à l'écriture existante — jamais affecter d'office ; le danger visé est la **seconde** écriture.
- **Retenu** : un rattachement **à une écriture du journal**, quelle qu'elle soit, dont le mouvement sur le compte du
  grand livre **actuel** du compte bancaire égale exactement le montant signé de la transaction : `GET …/transactions/
  {id}/entry-candidates` et `POST …/transactions/{id}/link-entry`. Les routes de la 29-8 (`link-candidates`, `link`)
  ne changent pas de contrat. Rien ne s'écrit au journal ; la transaction passe `reconciled` en pointant l'écriture.
- **Écartées** :
  - (b) élargir la réponse de `link-candidates` à des écritures (candidats hétérogènes, `settlementId` facultatif) :
    change un contrat d'API documenté (`docs/api-external.md:675-677`) ;
  - (c) un statut « écartée » non compté par la clôture du compte (piste (c) de #670, déjà écartée par C-29-8-3) :
    migration breaking, et faux rattachement muet ;
  - (d) un rattachement « fournisseur » seul, symétrique exact de la 29-8 : ne couvre ni l'écriture manuelle ni le
    compte interne, que #671 nomme.
- **Réversible** : oui avant publication ; ensuite, les routes sont un contrat.

- *Amendement du 2026-10-30 (validation P2 de la 3a, R8)* : la citation `supplier_invoices.rs:761-1018` du contexte
  se lit `:761-1013` (la fonction s'arrête là ; `:1018` est le titre de la section suivante).

### C-v017-3-3 — v017-3a : montant **exact**, sans tolérance

- **Contexte** : la 29-8 admet un écart de 0 à 5 centimes (client qui a payé un peu trop, la fiche refusant le
  trop-perçu, C-29-8-5). Ce motif n'existe pas pour une écriture quelconque : l'utilisateur choisit le montant qu'il
  saisit.
- **Retenu** : `M = t.amount` en décimal exact, signe compris (`SUM(debit) − SUM(credit)` des lignes de l'écriture sur
  le compte lié actuel). Un crédit de la banque appelle un débit du compte de banque ; un débit, un crédit.
- **Écartées** : la tolérance de la réconciliation — elle laisserait au compte de la banque un écart que rien ne
  signale à l'utilisateur, alors que la 29-8 le justifie et l'affiche.
- **Réversible** : oui (une condition de la requête).

### C-v017-3-4 — v017-3a : aucune fenêtre de dates ; la proximité trie (Q)

- **Contexte** : la 29-8 n'a pas de fenêtre (C-29-8-6) : un règlement saisi le jour où l'on remarque le paiement peut
  précéder ou suivre la transaction de plusieurs semaines, et Guy importe un relevé par mois. Pour une écriture
  quelconque, une fenêtre réduirait les faux avertissements (loyer du mois précédent non rapproché) sans les supprimer
  (même montant chaque mois).
- **Retenu** : pas de fenêtre ; tri par jours d'écart croissants, puis identifiant ; 50 au plus, filtre et tri dans la
  requête avant `LIMIT`. La date de chaque candidate s'affiche ; rien n'est présélectionné.
- **Écartées** : ±60 jours, ±1 an — une écriture d'un relevé importé en retard (plusieurs mois) ne serait plus
  proposée, et ce serait le cas le plus coûteux à retrouver.
- **Réversible** : oui.

### C-v017-3-5 — v017-3a : le domaine de la 29-8 est exclu de la liste des écritures

- **Contexte** : une écriture de règlement client par virement sur le même compte bancaire est déjà offerte par la 29-8
  (avec sa tolérance, sa trace `reconciliation.linked`, son libellé facture/client).
- **Retenu** : la liste des écritures exclut toute écriture qui est l'écriture d'un `invoice_settlements` de type
  `bank_transfer` dont `settlement_bank_account_id` est le compte de la transaction. Le dialogue montre les deux listes,
  disjointes. Un règlement client par **compte interne** (non couvert par la 29-8) entre dans la liste des écritures.
- **Écartées** : montrer l'écriture deux fois (deux traces différentes pour un même état) ; exclure tout
  `invoice_settlements` (le compte interne resterait sans remède).
- **Réversible** : oui.

### C-v017-3-6 — v017-3a : ni écriture contre-passée, ni contre-passation

- **Contexte** : `journal_entries.reverses_entry_id` (`20260828000002_journal_entries_reversal.sql`, `UNIQUE`) relie une
  contre-passation à son origine. Une écriture contre-passée et sa contre-passation touchent toutes deux le compte de la
  banque ; leur somme est nulle.
- **Retenu** : sont exclues l'écriture qui contre-passe (`reverses_entry_id IS NOT NULL`) et l'écriture contre-passée
  (`EXISTS` d'une écriture qui la désigne). Un rattachement à l'une d'elles marquerait comptabilisé un mouvement que
  les livres ne portent plus.
- **Écartées** : aucune.
- **Réversible** : oui.

### C-v017-3-7 — v017-3a : l'avertissement « semble déjà comptabilisée » est un **avis**, et l'action se confirme (Q)

- **Contexte** : précision du Project Lead sur #671 (2026-10-23) ; règle *Un appariement automatique propose, il ne crée
  jamais*. Un montant égal ne prouve rien (loyer mensuel) : un refus serveur bloquerait des cas légitimes.
- **Retenu** : `GET /reconciliation/proposals` porte, par transaction, un champ additif `alreadyBooked` (`null` ou
  `{ settlementCount, entryCount }`), compté par **les deux mêmes fonctions** que les listes du dialogue (aucune
  seconde règle). L'écran affiche l'avertissement sur la ligne, avec le bouton du dialogue ; *Accepter*, *Affecter
  manuellement* et *Éclater* sur une sélection qui contient une telle ligne passent par une **confirmation** qui la
  nomme. *Rejeter* ne demande rien (c'est la consigne d'attente). Le serveur ne refuse rien : un appel d'API direct
  reste libre, comme aujourd'hui.
- **Écartées** :
  - (b) masquer les propositions d'une telle ligne (comme le `notice` de la 30-1b) : un faux avertissement priverait
    l'utilisateur de la proposition juste ;
  - (c) un refus serveur à l'acceptation : même objection, et une règle d'écran déguisée en garde comptable.
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P2 de la 3a, C-v017-3-25)* : ce choix **passe à la v017-3c**, hors de la release
  0.17.0 ; la v017-3a ne porte plus d'avertissement sur les propositions.

### C-v017-3-8 — v017-3a : un seul dialogue, deux sections ; le bouton change de nom

- **Contexte** : `LinkSettlementModal.svelte` (29-8) n'est offert que sur un crédit en CHF
  (`ReconciliationProposals.svelte:237-239`, `canLink`).
- **Retenu** : le bouton devient « Rattacher à une écriture existante », offert sur toute ligne CHF non nulle ; le
  dialogue montre « Règlements de factures clients » (crédits seulement, liste de la 29-8 inchangée) puis « Écritures
  passées sur le compte de la banque » ; un seul choix parmi les deux ; *Rattacher* appelle la route de la section
  choisie. Le composant est généralisé (DRY), ses `data-testid` existants gardés.
- **Écartées** : deux boutons, deux dialogues — l'utilisateur ne sait pas d'avance par quel chemin le mouvement a été
  comptabilisé.
- **Réversible** : oui.

### C-v017-3-9 — v017-3a : annuler un rattachement suit le classement existant (Q)

- **Contexte** : `reconciliation_cancel` classe l'écriture liée — règlement client, règlement fournisseur, ou écriture
  propre — et défait ce qui la possède (`reconciliation_cancel.rs:57-79`) : le règlement est annulé, l'écriture propre
  contre-passée. #672 (ouverte) : aucun geste ne défait **le seul lien** ; distinguer un rattachement d'une acceptation
  exige de mémoriser l'origine du lien, donc une migration.
- **Retenu** : rien ne change à l'annulation. Annuler le rapprochement d'une transaction rattachée à une écriture
  saisie au journal **contre-passe** cette écriture ; rattachée à un règlement fournisseur, **annule le règlement**
  (la facture redevient ouverte). C'est visible et réparable ; le manuel le dit avant le geste, et le dialogue de
  rattachement le rappelle. #672 garde le besoin.
- **Écartées** : une colonne d'origine du lien (migration ; la 0.17.0 n'en porte qu'une, celle de la v016-2, et la
  contrainte des données demande de ne pas en multiplier) ; un dé-rapprochement « lien seul » sans colonne (il ne
  saurait pas distinguer un lien posé par la 3a d'une écriture créée par la réconciliation).
- **Réversible** : oui (#672).
- *Amendement du 2026-10-30 (validation P1 de la 3a, R6)* : « visible et réparable » est faux quand l'annulation est
  refusée (exercice clôturé, lettrage figé, compte archivé, aucun exercice ouvert aujourd'hui —
  `settlement_cancellation.rs:87-182`) : voir C-v017-3-21.

### C-v017-3-10 — v017-3a : ce qui reste hors périmètre, et son issue

- **Contexte** : `bank_transactions.matched_entry_id` est une seule colonne : une transaction pointe **une** écriture.
- **Retenu** : restent non couverts, écrits au manuel et dans une issue à ouvrir : (1) un paiement qui règle
  **plusieurs** écritures (lot pain.001 de plusieurs factures débité en une ligne, paiement groupé de plusieurs factures
  réglées séparément) ; (2) un règlement passé sur un **ancien** compte du grand livre du compte bancaire ; (3) un
  règlement saisi sur **un autre** compte bancaire ; (4) un paiement qui inclut les frais de rappel (#401). Un lot
  pain.001 d'**une** facture est couvert. Issue demandée : « Une transaction bancaire qui règle plusieurs écritures (lot
  de paiement, paiement groupé) ne se rattache pas ». Proposition à l'orchestrateur : #671 se ferme à la PR de release
  (`closes #671`) une fois l'issue du reste ouverte, avec un commentaire qui dit le partage.
- *Amendement du 2026-10-30 (validation P1 de la 3a)* : l'issue du reste est ouverte, **#823** ; le second côté d'un
  virement entre comptes propres garde un avertissement (C-v017-3-22).
- **Écartées** : une table de liens N:1 dans la 0.17.0 (migration, schéma, annulation, sauvegarde : une story à elle
  seule).
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P2 de la 3a, F7)* : chaque reste a son issue — (1) et le second côté d'un virement :
  #823 ; (2) et (3) : une issue à ouvrir (texte prêt dans la fiche 3a, § *Issues*) ; (4) : #401. La phrase « aucune autre »
  de la remédiation P1 ne vaut plus.

- *Amendement du 2026-10-30 (validation P3 de la 3a, R8)* : l'amendement P1 ci-dessus (« le second côté … garde un
  avertissement (C-v017-3-22) ») ne vaut plus depuis C-v017-3-25 : l'avertissement est passé à la v017-3c (#828) ; dans la
  0.17.0, le second côté n'en porte aucun. Les issues du reste sont ouvertes : #823, #827 ; #401.

### C-v017-3-11 — v017-3a : les données existantes ne sont pas touchées

- **Contexte** : contrainte du Project Lead du 2026-10-30 ; consigne d'usage de #671 (2026-10-23) : une transaction déjà
  comptabilisée « se rejette, elle ne s'affecte pas ». La base du NAS peut donc porter des transactions **rejetées**
  (`auto_match_rejected_at` posé, `status = 'pending'`) qui attendent ce geste.
- **Retenu** : aucune migration, aucune écriture hors du geste de l'utilisateur. Une transaction rejetée se rattache
  par la même voie que dans la 29-8 : *Annuler le rejet* dans le détail de l'import (28-5b), puis le dialogue ; l'API
  admet aussi le rattachement direct d'une transaction rejetée (trace `was_previously_rejected`). Testé sur une
  fixture qui reproduit l'état d'une base 0.16.x.
- **Écartées** : un rattachement automatique des transactions dont une écriture unique correspond (interdit par la
  règle d'appariement) ; une liste dédiée des rejetées « qui semblent comptabilisées » (élargissement : demander
  l'issue si besoin).
- **Réversible** : sans objet.

### C-v017-3-12 — v017-3a : noms de l'action d'audit et du code d'erreur

- **Retenu** : trace `reconciliation.entry_linked` (« Transaction rattachée à une écriture ») ; refus `409
  RECONCILIATION_ENTRY_NOT_LINKABLE` (un seul code pour toutes les conditions de la règle, comme
  `RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, C-29-8-10).
- **Écartées** : réutiliser `reconciliation.linked` — son libellé et ses détails nomment un règlement et une facture.
- **Réversible** : oui avant publication.

### C-v017-3-13 — v017-3b : refuser à la création une ligne sous le centime (Q)

- **Contexte** : `create_in_tx` calcule `line_total = quantity × unit_price` sans arrondi ni borne d'échelle
  (`supplier_invoices.rs:400`, `:426`) ; la TVA est arrondie au centime par ligne (`kesh-core/src/accounting/vat.rs:40-44`) ;
  le TTC est donc sous le centime **exactement** quand une ligne l'est. Le formulaire borne quantité et prix à quatre
  décimales (`frontend/.../journal-entries/balance.ts:23`), le serveur ne borne rien. Pistes de #687 : arrondir ou
  refuser à la création ; régler avec une ligne d'arrondi.
- **Retenu** : refuser, au serveur, toute ligne dont `quantity × unit_price` n'est pas au centime (`400
  SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`, ligne désignée), aux deux routes de création (saisie, complétion d'une pièce
  importée), par une fonction pure unique de `kesh-core`, appelée par `create_in_tx` ; et borner quantité et prix à
  quatre décimales (`MAX_DECIMAL_SCALE`, comme la facture client). Le TTC est alors au centime par construction. Une
  facture fournisseur reprend ce que le fournisseur facture, qui est au centime : la ligne se saisit au montant facturé.
- **Écartées** :
  - (b) arrondir `line_total` au centime en silence : change le HT et la TVA saisis, sans le dire ;
  - (c) régler avec une ligne d'arrondi (#476 côté client) : refonte de `pay_in_tx_as`, partagé par la fiche, la
    confirmation d'un lot et l'acceptation 30-1b ; et le total d'une facture d'achat resterait faux d'un dixième de
    centime au grand livre du fournisseur.
- **Réversible** : oui (une garde).

- *Amendement du 2026-10-30 (validation P2 de la 3b, R5 = F14)* : la phrase du contexte « le TTC est donc sous le
  centime **exactement** quand une ligne l'est » est inexacte — deux lignes sous le centime peuvent se compenser. La
  règle par ligne reste juste, pour une autre raison : chaque ligne écrit au journal une ligne de charge à son
  `line_total` (`supplier_invoices.rs:140-145`) ; une ligne sous le centime écrit donc une ligne de journal sous le
  centime même quand le total est au centime.

### C-v017-3-14 — v017-3b : l'existant n'est pas modifié ; il est signalé, et son traitement décrit (Q)

- **Contexte** : contrainte du Project Lead du 2026-10-30 : une facture fournisseur existante au TTC sous le centime
  n'est pas modifiée d'office.
- **Retenu** : aucune migration, aucun `UPDATE` ; le paiement d'une telle facture **n'est pas refusé** (un lot
  pain.001 déjà généré qui la contient doit pouvoir être confirmé). La fiche de la facture affiche un avertissement qui
  dit la conséquence (son règlement ne pourra jamais être rapproché d'un relevé) et la conduite : **ouverte**, l'annuler
  et la ressaisir au centime ; **payée**, annuler le règlement, puis la facture, la ressaisir, puis la régler par la
  réconciliation ou depuis la fiche suivie du rattachement (v017-3a).
- **Écartées** : (b) refuser le paiement par virement d'une telle facture (casse la confirmation d'un lot déjà parti à
  la banque) ; (c) une migration qui arrondit (réécriture silencieuse, interdite).
- **Réversible** : oui.
- *Amendement du 2026-10-30 (validation P1 de la 3b, R1 = F5)* : la conduite dépend de l'état ; elle devient une
  matrice (ouverte, ouverte dans un lot en cours, payée, annulée) et finit par « si un bouton d'annulation manque, la
  raison est indiquée plus bas ». L'annulation du règlement d'une facture payée par un lot **confirmé** est admise
  (`supplier_invoices.rs:1547-1551`).

- *Amendement du 2026-10-30 (validation P2 de la 3b, C-v017-3-24)* : la matrice de l'amendement précédent est
  **retirée** ; voir C-v017-3-24.

- *Amendement du 2026-10-30 (validation P3 de la 3b, R8 = F4)* : depuis C-v017-3-24, l'avertissement ne porte **plus
  la conduite** à l'écran (ni la matrice, ni la phrase « si un bouton d'annulation manque ») ; elle est au manuel
  (`sec:facture-fournisseur-sous-centime`). Et depuis C-v017-3-26, il ne s'affiche pas pour une facture annulée.

### C-v017-3-15 — v017-3b : ordre des refus de la complétion d'une pièce importée

- **Contexte** : la complétion compare `Σ TTC` des lignes au montant du QR **avant** `create_in_tx`
  (`imported_supplier_invoices.rs:250-275`, `AMOUNT_MISMATCH`) ; son test (b) (`inbox_import_e2e.rs:1375`) utilise
  `3 × 33.3333 = 99.9999`.
- **Retenu** : la garde de ligne vient **avant** `AMOUNT_MISMATCH` (elle désigne la cause : une ligne sous le centime ne
  peut atteindre aucun montant de QR au centime) ; le test (b) attend le nouveau code ; un cas `AMOUNT_MISMATCH` à
  lignes au centime le remplace.
- **Écartées** : garder `AMOUNT_MISMATCH` en premier — son message (« le total ne correspond pas ») cache la cause.
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P2 de la 3b, R2)* : la garde de ligne de la complétion jugeait une ligne
  négative avant toute positivité (rendant le code du centime là où la saisie rend celui de la positivité). La passe de
  forme des lignes (positivité, centime, TVA) devient **une** fonction pure de `kesh-db`, `purchase_lines_form`, appelée
  par `create_in_tx` et par la complétion, avant `AMOUNT_MISMATCH` : un ordre, deux chemins.

- *Amendement du 2026-10-30 (validation P3 de la 3b, R8 = F4)* : « un cas `AMOUNT_MISMATCH` à lignes au centime le
  remplace » ne vaut plus : le cas (a) du même test (`inbox_import_e2e.rs:1365`, `:1373`) le garde déjà (R12 de la P2).

### C-v017-3-16 — Voisin non mécanique : la somme de contrôle du pain.001

- **Contexte** : `kesh-payment/src/pain001/mod.rs:148-150` calcule `CtrlSum` comme l'arrondi de la somme **non
  arrondie**, alors que chaque `InstdAmt` est arrondi un à un (`:124-126`, `round_dp(2)`, arrondi bancaire par défaut de
  `rust_decimal`). Avec des TTC sous le centime, `CtrlSum` peut différer de `Σ InstdAmt` : la banque rejette le fichier.
  La 3b ferme la source pour les factures nouvelles ; les factures existantes restent exposées.
- **Retenu** : hors périmètre (autre crate, autre geste) ; issue demandée, P3 : « pain.001 : `CtrlSum` arrondit la
  somme au lieu de sommer les montants arrondis ».
- *Amendement du 2026-10-30 (validation P1 de la 3b, F8)* : l'issue est ouverte, **#822** (P3) ; la question à Guy est
  retirée.
- **Réversible** : sans objet.

### C-v017-3-17 — Mots-clés et CHANGELOG

- **Retenu** : commits en `refs #671` / `refs #687` seulement. Proposition : la PR de release porte `closes #687` ;
  `closes #671` si l'issue du reste (C-v017-3-10) est ouverte, `refs #671` sinon. CHANGELOG : entrées sous
  `## [0.17.0]`, section *Corrigé*, que l'orchestrateur fusionne ; aucune version bumpée, aucune migration.
- **Réversible** : oui.

### C-v017-3-18 — v017-3b : un montant sous le centime s'affiche aux quatre décimales ; un seul `hasSubCentime`

- **Contexte** (validation P1, R2 = F2) : la fiche d'une facture affiche son total par `formatSupplierInvoiceTotal`, à
  deux décimales (`supplier-invoice-helpers.ts:11-14`, `balance.ts:93-96`) : `100.0040` se lit « 100.00 », à côté d'un
  avertissement qui dit le contraire. `formatExactAmount` existe (`invoices/invoice-helpers.ts:82-89`) ; deux
  `hasSubCentime` homonymes coexistent (`invoices/invoice-helpers.ts:73`, `supplier-invoices/supplier-invoice-form.ts:63`).
- **Retenu** : l'avertissement, la cellule TTC de la fiche et la liste passent par `formatExactAmount` ; les deux
  `hasSubCentime` se réduisent à un (déplacé vers un module partagé si l'import entre domaines l'exige).
- **Écartées** : une troisième fonction de format (DRY).
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P5 de la 3b, C1)* : « et la liste » est remplacé par C-v017-3-27 — seule la
  **fiche** de la facture passe aux décimales exactes (TTC, cellules de lignes, et total de la facture dans le sélecteur de
  document, `AttachDocumentPicker.svelte:164`, qui s'y ajoute) ; la liste garde son format (#831).

### C-v017-3-19 — v017-3b : le formulaire garde son patron de refus au clic

- **Contexte** (validation P1, F1) : le bouton *Enregistrer* n'est désactivé que pendant l'enregistrement
  (`SupplierInvoiceForm.svelte:579`, `disabled={saving}`) ; `linesInvalid` est lu au clic (`:203`), avec un message
  générique ; `inbox-import.spec.ts:102-103` s'appuie sur ce patron.
- **Retenu** : refus au clic, avec un message **dédié** (une ligne n'est pas au centime) et le message sous la ligne ;
  le bouton reste actif.
- **Écartées** : désactiver le bouton (changement de contrat de l'écran, tests existants à reprendre, sans gain pour
  cette story).
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P2 de la 3b, F5 = R7)* : le contrôle de centime est un prédicat distinct de
  `linesInvalid`, lu **après** lui dans `submit()` : le message générique d'abord, le message dédié ensuite.

### C-v017-3-20 — v017-3b : la dépendance à la v017-3a est écrite

- **Contexte** (validation P1, R10 = F3) : la conduite « payée » (« …ou depuis sa fiche avant de rattacher la
  transaction ») suppose le rattachement d'un débit à une écriture, que seule la 3a livre ; à `c5101b07`, le manuel dit
  le contraire (`user-manual.tex:1634-1639`, `:2409`).
- **Retenu** (précision de l'orchestrateur, 2026-10-30) : la 3b est livrée dans la même release que la 3a ; la
  dépendance est écrite dans la fiche, avec les sections du manuel communes aux deux, réconciliées à la réunion des
  branches. Si la 3a n'était pas livrée, la seconde voie de la conduite se retire.
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P3 de la 3b, F4)* : « la conduite “payée” » se lit désormais « la voie 7 du
  manuel » (`sec:facture-fournisseur-sous-centime`) : l'écran ne prescrit plus de geste.

### C-v017-3-21 — v017-3a : un rattachement sans retour est admis, mais annoncé ; la règle de blocage n'est pas dupliquée

- **Contexte** (validation P1, R6 ; orientation de l'orchestrateur : « un rattachement ne doit jamais devenir définitif
  sans que l'utilisateur le sache ») : l'annulation d'un rapprochement passe par `cancel_blocker`
  (`reconciliation_cancel.rs:232-270`) puis, pour une écriture propre, `settlement_entry_cancel_blocker`
  (`settlement_cancellation.rs:87-182` : exercice clôturé, lettrage figé, compte archivé, aucun exercice ouvert
  aujourd'hui). La règle de la 3a n'a pas de filtre d'exercice : rattacher l'écriture d'un exercice clôturé est le cas
  d'usage naturel de #671 (relevé importé après la clôture), et le rattachement ne pourra plus s'annuler.
- **Retenu** : ne **pas** filtrer ces candidates (le filtre écarterait le cas légitime) ; chaque candidate porte
  `cancelBlockedBy`, calculé par **la** fonction de blocage de l'annulation : `cancel_blocker` est scindée en une lecture
  par écriture (`entry_cancel_blocker`), qu'elle appelle et que la liste appelle avec `unlinking = None`. Le dialogue
  dit, pour une telle candidate, que le rattachement ne pourra pas être annulé et pourquoi, exige une case à cocher, et ne
  promet aucune contre-passation. Test : écriture d'exercice clôturé candidate avec `FISCAL_YEAR_CLOSED`, puis annulation
  refusée avec ce même motif.
- **Écartées** : (b) ne proposer que les écritures dont l'annulation serait admise (préférence de l'orchestrateur) —
  écarte le cas d'usage principal, la transaction resterait en attente pour toujours, ce que #671 combat ; (c) un
  prédicteur à part — seconde règle, qui dériverait.
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P2 de la 3a, R2, F3, F4)* : « ne pourra pas être annulé » était faux — chaque
  motif se lève (réouverture d'exercice, réactivation du compte, exercice du jour créé, annulation du solde), et une
  candidate sans motif aujourd'hui peut en recevoir un demain. Le motif devient une **vérité datée** : « Tant que la
  raison suivante subsiste, ce rattachement ne pourra pas être annulé : » suivi du texte existant
  (`reconciliationCancelMessage`, qui dit le moyen de lever) ; pour une candidate sans motif, « Aujourd'hui, annuler …
  contre-passerait cette écriture …, tant que son exercice reste ouvert ». Codes de fil (`SettlementCancelBlocker::code()`),
  `cancelBlockedLabel` ajouté. Les angles morts du prédicteur et les motifs de propriété vont à L9.

- *Amendement du 2026-10-30 (validation P3 de la 3a, R4)* : « chaque motif se lève » est faux pour `INVOICE_CREDITED`
  (« il ne s'annule pas », état hérité d'avant la 0.12.1) ; l'introduction du dialogue devient « Ce rattachement ne pourra
  pas être annulé tant que dure l'obstacle suivant. », vraie pour tous les motifs (fiche, AC-13, L10).

### C-v017-3-22 — v017-3a : l'autre côté d'un virement entre comptes propres garde un avertissement

- **Contexte** (validation P1, R7) : après rattachement du côté A d'une écriture `D banque B / C banque A`, la condition 4
  écarte l'écriture pour la transaction de B : sans autre mesure, B perd tout avertissement et se laisse affecter en
  double — le défaut que la story combat.
- **Retenu** : `alreadyBooked.matchedElsewhereCount`, lu par un second mode du **même** constructeur de requête (condition
  4 retournée : rapprochée d'une transaction d'un **autre** compte bancaire ; condition 6 retirée) ; l'avertissement
  conseille de rejeter ; la confirmation s'applique. Le traitement (rattacher les deux côtés) relève de #823.
- **Écartées** : ne pas compter comme rapprochée une écriture qui touche un autre compte bancaire (rouvre l'invariant
  « une écriture, une transaction » sur lequel reposent l'annulation et la tuile).
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P2 de la 3a, C-v017-3-25)* : ce choix **passe à la v017-3c**, avec un
  correctif (R1 = F6) : le second mode ne compte que les écritures rapprochées d'un compte bancaire dont le compte du
  grand livre **diffère** de celui de `t` ; sans quoi deux comptes bancaires partageant un compte du grand livre feraient
  conseiller de rejeter une transaction non comptabilisée.

### C-v017-3-23 — v017-3a : le coût se mesure sur un historique volumineux

- **Contexte** (validation P1, R5) : la somme par écriture parcourt tout l'historique du compte à chaque appel, jusqu'à
  trois fois par transaction de la page.
- **Retenu** : mesure à 1 000, 5 000 et 20 000 écritures synthétiques, 500 transactions en attente ; seuil : 3 s à
  20 000, ou pente plus que linéaire ; au-delà, section *Performance debt* et issue, remède nommé (« une lecture par
  page ») mais non implémenté ici.
- **Écartées** : implémenter d'emblée la lecture par page (seconde forme de la règle, avant toute mesure).
- **Réversible** : oui.

- *Amendement du 2026-10-30 (validation P2 de la 3a, C-v017-3-25)* : ce choix **passe à la v017-3c**, avec la mesure.

### C-v017-3-24 — v017-3b : la matrice de conduite est retirée ; un avertissement statique, le manuel pour les voies

- **Contexte** (validation P2 de la 3b) : la sévérité stagne (11 → 11 MEDIUM), et la plupart des défauts portent sur la
  matrice de conduite état par état introduite à la remédiation P1 — certains en naissent (F1 à F6, R3, R4 de la P2 :
  séquence qui laisse la facture pire qu'avant, lot déjà transmis, compte interne, état « lot confirmé » non observable,
  rôle Lecteur, facture annulée après paiement, exercice clôturé). C'est le recyclage que vise la règle de découpage ;
  déclaré à Guy par l'orchestrateur.
- **Retenu** (décision de l'orchestrateur, 2026-10-30) : le cœur reste — refus à la saisie et à la complétion (AC-1 à
  AC-7), une fonction de `kesh-core`, restauration (AC-19), aucune donnée réécrite. Pour une facture existante sous le
  centime, un avertissement **unique et statique** sur sa fiche, quel que soit l'état (annulée comprise) et le rôle, qui
  ne promet aucun bouton et renvoie au manuel ; `formatExactAmount` pour le montant. Le manuel porte une sous-section à
  `\label{sec:facture-fournisseur-sous-centime}` qui décrit en prose les voies et leurs limites. Aucune logique d'état à
  l'écran, donc aucune mutation, aucun Vitest, aucune capture par état.
- **Écartées** : corriger la matrice une passe de plus (chaque correction a fait naître la suivante).
- **Réversible** : oui (une matrice pourra revenir dans une story dédiée si l'usage le demande).

- *Amendement du 2026-10-30 (validation P3 de la 3b, F1)* : « quel que soit l'état (annulée comprise) » est remplacé par
  C-v017-3-26.

- *Amendement du 2026-10-30 (validation P4 de la 3b, C4)* : « aucune logique d'état à l'écran, donc aucune mutation,
  aucun Vitest, aucune capture par état » est remplacé par C-v017-3-26, qui réintroduit une condition d'état (annulée) avec
  son Vitest, sa mutation et ses captures.

### C-v017-3-25 — v017-3a : découpage — l'avertissement des propositions part en v017-3c, hors de la 0.17.0

- **Contexte** (validation P2 de la 3a) : la sévérité ne baisse pas (P1 : 1 HIGH, 12 MEDIUM ; P2 : 12 MEDIUM), et une part
  des défauts naît de la remédiation P1 : `cancelBlockedBy` et ses codes (F3, F4, R2, R9), `matchedElsewhereCount` (F5,
  F6, R1). C'est le recyclage que vise la règle de découpage ; déclaré à Guy par l'orchestrateur.
- **Retenu** (décision de l'orchestrateur, 2026-10-30) : la **v017-3a** garde le cœur de #671 et reste dans la release
  0.17.0 — routes `entry-candidates` et `link-entry`, section « écritures » du dialogue, règle de candidature, verrous et
  courses, registres, audit, manuel, `api-external.md` ; la mention d'irréversibilité y devient une vérité datée
  (C-v017-3-21 amendé). La **v017-3c** (`v017-3c-avertissements-propositions.md`, `backlog`, hors de la 0.17.0) reçoit
  `alreadyBooked` et ses deux modes, l'avertissement, les confirmations, la mesure de coût (C-v017-3-7, 22, 23) et les
  findings qui s'y rattachent (R1, F5, F6, F14, F16 en partie). Une issue est à ouvrir pour elle (corps prêt dans sa
  fiche) ; le commentaire posé sur #823 (« l'autre côté garde un avertissement ») se corrige (texte prêt dans la fiche
  3a, § *Issues*).
- **Écartées** : remédier une passe de plus (chaque remédiation de ces deux formes a fait naître la suivante) ; retirer
  aussi `cancelBlockedBy` (le rattachement deviendrait silencieusement irréversible, ce que l'orchestrateur exclut).
- **Réversible** : oui (la 3c se planifie à part).

- *Amendement du 2026-10-30 (validation P4 de la 3a, C3)* : l'issue de la v017-3c est ouverte, **#828** ; celle du
  reste (ancien compte, autre compte bancaire), **#827**.

### C-v017-3-26 — v017-3b : pas d'avertissement pour une facture annulée ; un texte vrai dans tous les autres états

- **Contexte** (validation P3 de la 3b, F1) : l'avertissement statique s'affichait « quel que soit l'état, annulée
  comprise » et disait « son règlement ne pourra pas être rapproché automatiquement d'un relevé bancaire ». Une facture
  annulée n'a plus de règlement (détaché, `[id]/+page.svelte:620-628`) ; une facture réglée par compte interne n'a pas de
  débit au relevé ; « automatiquement » suggère un chemin manuel qui n'existe pas.
- **Retenu** (décision de l'orchestrateur, 2026-10-30) : l'avertissement ne s'affiche pas pour une facture annulée (le
  règlement détaché resté au grand livre est décrit au manuel, voie 3) ; ailleurs, le texte est une condition vraie
  dans tous les cas : « Le total de cette facture ({amount}) n'est pas au centime : un paiement bancaire de ce montant
  ne pourra être rapproché d'aucune transaction de relevé. Voir le manuel, § … ». Vitest, captures et mutations suivent.
- **Écartées** : garder l'annulée avec un texte neutre (l'avertissement inviterait à un traitement inutile).
- **Réversible** : oui.

### C-v017-3-27 — v017-3b : les décimales exactes sur la seule fiche de la facture

- **Contexte** (validation P4 de la 3b, C1, C2) : la remédiation P3 a étendu `formatExactAmount` à la liste et aux
  lignes, laissé le sélecteur de document au format courant sur une justification fausse, et affirmé qu'aucun banc ne
  rendait la page de la fiche — alors que `supplier-settlement-page.test.ts` et `supplier-document-page.test.ts` la
  rendent. Le périmètre d'affichage s'étendait à chaque passe.
- **Retenu** (décision de l'orchestrateur, 2026-10-30) : seule la **fiche** de la facture passe aux décimales exactes —
  TTC, cellules de lignes (`:501`, `:503`) et total de la facture dans le sélecteur de document
  (`AttachDocumentPicker.svelte:164`) ; l'intégration se prouve aux deux bancs existants. La **liste** garde son format :
  limite écrite et issue demandée (P4) — *ouverte depuis : #831 (validation P5, C2)*.
- **Écartées** : passer aussi la liste (aucun banc, le périmètre continuerait de s'étendre).
- **Réversible** : oui (l'issue de la liste).

### C-v017-3-40 — v017-3a : les tests de la règle au dépôt vivent dans le fichier de test HTTP

- **Contexte** (développement, AC-2) : la fiche demande des tests `#[sqlx::test]` de `find_entry_candidates`, une
  condition par test ; leurs montages (facture validée et réglée, facture fournisseur payée, lot pain.001 confirmé,
  écriture saisie, contre-passation) passent par les routes. Aucun fichier de `crates/kesh-db/tests/` ne porte ces
  montages ; la 29-8 a testé sa règle dans `kesh-api/tests/reconciliation_link_e2e.rs`.
- **Retenu** : un seul fichier neuf, `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs` ; les tests AC-2
  appellent `find_entry_candidates` **directement** (fonction du dépôt) sur des états montés par les routes, les
  autres passent par HTTP. Le montage de la 29-8 y est recopié (chaque fichier de test est un binaire). Les deux
  fonctions pures (`transaction_can_be_linked_to_entry`, `entry_candidates_sql`) ont leurs tests unitaires dans
  `reconciliation_link.rs`.
- **Écartées** : un fichier `kesh-db/tests/` qui recopierait par SQL ou par les fonctions de dépôt tout le montage
  des règlements (plus de code, moins de fidélité au chemin réel) ; un module commun de montage (toucherait le
  fichier de la 29-8 pour rien).
- **Réversible** : oui.

### C-v017-3-41 — v017-3a : le dialogue garde son nom de fichier

- **Contexte** (AC-13 : « généralisé, renommé s'il le faut — le dire »).
- **Retenu** : `LinkSettlementModal.svelte` garde son nom et ses `data-testid` (`link-settlement-modal`,
  `link-settlement-button`) ; son en-tête dit qu'il porte les deux sections. Le test des replis le lit par son nom :
  rien à suivre.
- **Écartées** : le renommer (`LinkEntryModal`) — diff plus large, collision accrue avec la v017-3b et la v017-3c,
  qui le reprend.
- **Réversible** : oui.

### C-v017-3-42 — v017-3a : la tentative verrouillée est factorisée pour les deux rattachements

- **Contexte** (AC-6 : « si `link_once` et `link_entry_once` ne diffèrent que par l'appel intérieur et la réponse,
  factoriser et le dire »). Ils ne diffèrent que par cela.
- **Retenu** : `locked_link_attempt` (transaction neuve, `with_account_lock`, `conclude_locked_attempt`) et
  `link_refusal_error` (les refus `NotFound`/`NotPending` communs, `NotLinkable` propre à chacun), appelés par
  `link_once` (29-8, comportement inchangé — ses 24 tests verts) et `link_entry_once`.
- **Écartées** : recopier `link_once` (DRY).
- **Réversible** : oui.

### C-v017-3-43 — v017-3a : la liste des écritures et ses motifs d'annulation dans une seule transaction de lecture

- **Contexte** (AC-5) : `cancelBlockedBy` se calcule par candidate (`entry_cancel_blocker`, plusieurs requêtes).
- **Retenu** : `GET …/entry-candidates` ouvre **une** transaction de lecture (un instantané, aucun verrou), y lit la
  transaction, la liste et chaque motif, puis la referme — patron de `reconciliation_cancel::get_view`. Le pré-vol
  reste sur le pool, comme la 29-8.
- **Écartées** : lectures sur le pool, chacune dans son instantané (une réponse qui pourrait se contredire).
- **Réversible** : oui.

### C-v017-3-44 — v017-3a : le journal de l'écriture s'affiche par son code

- **Contexte** (AC-13, colonne « journal » de la section des écritures). L'écran des écritures traduit le code par une
  fonction locale (`journalLabel`, `routes/(app)/journal-entries/+page.svelte:303`), non partagée.
- **Retenu** : le dialogue affiche le **code** (`Banque`, `OD`, …), tel que l'API le rend.
- **Écartées** : extraire `journalLabel` dans un module partagé (déplacement hors périmètre, gardes i18n à recompter).
- **Réversible** : oui — un défaut cosmétique pour une installation non française.

### C-v017-3-45 — v017-3a : la case de l'obstacle se décoche à chaque changement de choix

- **Contexte** (AC-13 : *Rattacher* exige la case `entry-cancel-blocked-ack` pour une candidate à motif).
- **Retenu** : la case est remise à vide à chaque changement de choix et à chaque relecture des listes : une
  reconnaissance donnée pour une écriture ne vaut pas pour une autre.
- **Écartées** : garder la case cochée (reconnaissance transférée d'office).
- **Réversible** : oui.
