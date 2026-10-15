# Epic 30 — registre des choix autonomes de la Story 30-1

Registre propre à la story (`docs/agents-orchestration.md` § 1) ; l'orchestrateur le verse **par union** dans
`epic-30-choix-autonomes.md`. Fiche : `30-1-reconciliation-factures-fournisseur.md`. Relevés sur `15686d55`.

---

## Story 30-1 — La réconciliation règle les factures fournisseur (#686)

### C-30-1-1 — Établir l'existant au code avant de concevoir

- **Contexte** : #686 décrit le symétrique de la réconciliation client ; cinq stories de l'Epic 29 ont retouché ce code.
- **Retenu** : inventaire cité `fichier:ligne` (fiche, § *Ce qui existe*), verdict par prémisse.
- **Écartée** : partir de la description de l'issue.
- **Réversible** : sans objet.

### C-30-1-2 — Prémisse corrigée : le dé-rapprochement ne sait pas défaire un règlement fournisseur

- **Contexte** : #686 dit « *Annuler le rapprochement* défait le règlement, comme côté client ». `classify`
  (`reconciliation_cancel.rs:75-104`) ne lit que `invoice_settlements` ; un règlement fournisseur rapproché serait classé
  `Entry`, et `reverse_in_tx` refuse une écriture possédée par un règlement fournisseur.
- **Retenu** : la prémisse est **fausse** aujourd'hui ; la 30-1 la rend vraie (30-1a). À commenter sur l'issue par
  l'orchestrateur.
- **Réversible** : sans objet.

### C-30-1-3 — Découpage 30-1a (annuler) puis 30-1b (proposer et accepter)

- **Contexte** : plus de cinq modules touchés (règle de splitting préventif).
- **Retenu** : 30-1a d'abord, pour qu'à aucun commit de `main` une acceptation ne produise un rapprochement que rien ne
  défait ; une fiche, deux clés filles.
- **Écartées** : une story unique (au-delà du seuil) ; l'ordre inverse (état indéfaisable sur `main` entre deux merges) ;
  un découpage backend/frontend (l'E2E de parcours n'existerait qu'à la fin, et la première moitié n'aurait aucun test
  de bout en bout).
- **Réversible** : oui, avant le développement.

### C-30-1-4 — #671 reste hors du périmètre

- **Contexte** : C-30-0-2 ; le symétrique fournisseur du rattachement (29-8).
- **Retenu** : non mécanique — autre table, mouvement de signe opposé, autre ordre de verrous, modale et textes à
  généraliser ; le besoin de Guy est couvert par la 30-1b (facture saisie ouverte puis réglée par la réconciliation).
  #671 reste ouverte ; la 30-1a lui prépare le classement `supplier_settlement`.
- **Écartée** : l'intégrer (élargissement non mécanique, leçon 4.1 de la rétrospective 28).
- **Réversible** : oui.

### C-30-1-5 — Montant exactement égal au TTC

- **Contexte** : #686 dit « montant égal au reste dû » ; le règlement fournisseur est binaire (`pay_in_tx` écrit le TTC
  des deux côtés), il n'y a pas de reste dû ; `total_amount` est en `DECIMAL(19,4)`.
- **Retenu** : candidate seulement si `total_amount = |tx.amount|` exactement ; refus `RECONCILIATION_AMOUNT_MISMATCH`
  sinon.
- **Écartées** : la tolérance client (règlement partiel inexistant ; ligne de banque ≠ relevé) ; comparer le TTC
  arrondi au centime (ligne de banque à quatre décimales ≠ relevé).
- **Réversible** : oui (un règlement partiel fournisseur serait une story).

### C-30-1-6 — Une écriture de règlement détachée reste `Entry`

- **Contexte** : annuler une facture fournisseur payée détache son règlement (25-3-c) ; l'écriture « se contre-passe ».
- **Retenu** : le classement fournisseur exige `status = 'paid'` et la colonne posée ; détachée, l'écriture reste
  `Entry` et le dé-rapprochement la contre-passe par `reverse_in_tx` — comportement d'aujourd'hui, testé.
- **Réversible** : oui.

### C-30-1-7 — Le classement fournisseur est une lecture verrouillante, coût de l'intervalle assumé

- **Contexte** : « aucune lecture non verrouillante avant l'étape 3 » ; l'index de `settlement_journal_entry_id` n'est
  pas unique ; une lecture verrouillante qui ne trouve rien pose un verrou d'intervalle.
- **Retenu** : `FOR UPDATE` à la même place que la lecture de `invoice_settlements` (qui a le même coût sur un `Entry`) ;
  écrit au doc-comment ; cycle cherché par la validation ; interblocage rejoué par la route.
- **Écartées** : lecture ordinaire (fige la vue avant le verrou de l'exercice, défaut de la 25-3-b) ; verrouiller
  l'écriture d'abord (ordre inverse de `cancel_settlement_in_tx`) ; une migration d'index unique (hors périmètre, P8).
- **Réversible** : oui.

### C-30-1-8 — Contrat d'API additif

- **Retenu** : champs neufs (`supplierInvoiceId`, `supplierInvoiceNumber`, `contactName`), `kind = supplier_settlement`,
  `candidateType = supplier_invoice`, type d'acceptation `supplier_invoice` ; `invoiceId`/`invoiceNumber` restent
  client ; `accepted[].invoiceId = 0` (sentinelle de `split` et `rule`).
- **Écartée** : réemployer `invoiceId` pour la facture fournisseur (deux tables, un même identifiant : ambigu pour une
  intégration).
- **Réversible** : non une fois publié (contrat externe).

### C-30-1-9 — Où vont les textes

- **Retenu** : `api-external.md` dès la 30-1a (le contrat existe au merge) ; manuel, CHANGELOG et README à la 30-1b
  (rien ne produit l'état avant elle).
- **Réversible** : oui.

### C-30-1-10 — L'acceptation appelle `pay_in_tx`

- **Retenu** : le geste de la fiche, appelé ; la route n'ajoute que ce qui lui est propre (sens, fenêtre, montant, lot
  verrouillé, lien).
- **Écartée** : une écriture de règlement réécrite dans la route (jumeau).
- **Réversible** : oui.

### C-30-1-11 — Texte propre sur la fiche fournisseur, pas de dialogue

- **Contexte** : `settlement-cancel-blocked-bank-match` renvoie à « cette fiche », fausse côté fournisseur.
- **Retenu** : une clé propre au fournisseur, qui renvoie au détail de l'import.
- **Écartée** : porter `CancelReconciliationDialog` sur la fiche fournisseur (symétrie souhaitable, hors périmètre
  minimal).
- **Réversible** : oui.

### C-30-1-12 — Référence de paiement : égalité normalisée

- **Retenu** : `payment_reference` comparée par **égalité** après retrait des blancs et mise en majuscules ; le numéro
  de facture fournisseur garde la règle client ; le maximum des deux.
- **Écartées** : containment ou préfixe (les références QR commencent par des zéros : faux indices) ; changer la
  normalisation client (déplace des scores client établis).
- **Réversible** : oui.

### C-30-1-13 — Fenêtre fournisseur : `F ≤ D + 7`, `max(F, échéance) ≥ D − 365`

- **Contexte** : Guy saisit la facture **après** le paiement, souvent ancienne ; un achat en ligne peut être facturé
  après paiement.
- **Retenu** : 7 jours au-dessus, un an sous l'exigibilité ; jamais `created_at`.
- **Écartées** : la fenêtre client (30 jours : exclut les paiements tardifs fournisseur ; 1 jour : exclut l'achat en
  ligne) ; aucune borne basse (abonnements de même montant proposés indéfiniment).
- **Réversible** : oui (constantes).

### C-30-1-14 — Lot `generated` exclu ; « presque au montant » garde la règle

- **Retenu** : la requête exclut une facture engagée dans un lot `generated` (sinon double paiement à la banque) ; une
  facture à ≤ 0,05 du débit, non égale, portant un indice, masque la règle (leçon de la 29-6).
- **Écartées** : ne pas masquer (charge comptée deux fois, muette) ; masquer sans indice (toute règle disparaîtrait
  devant n'importe quelle facture d'un montant voisin).
- **Réversible** : oui.

### C-30-1-15 — `contactName` rempli pour les deux sortes, affiché pour le fournisseur

- **Retenu** : un champ, une conversion ; l'écran ne l'affiche que pour une facture fournisseur (numéros non uniques
  entre fournisseurs).
- **Écartée** : l'afficher aussi côté client (déplace les captures et E2E de l'Epic 29 ; hors demande).
- **Réversible** : oui.

### C-30-1-16 — Pas de garde de score à l'acceptation fournisseur

- **Retenu** : l'égalité de montant précède ; le score vaut ≥ 0,5 par construction ; une garde serait du code mort
  qu'aucune mutation ne tuerait.
- **Réversible** : oui.

### C-30-1-17 — Les E2E de parcours sont portés par la 30-1b

- **Retenu** : le parcours de bout en bout (débit → acceptation → annulation) n'existe qu'après la 30-1b ; la 30-1a se
  couvre par Rust et Vitest.
- **Écartée** : un E2E de la 30-1a sur un lien posé en SQL (le harnais E2E n'a pas d'accès SQL ; route de test à créer
  pour rien).
- **Réversible** : oui.

### C-30-1-18 — Trois codes neufs

- **Retenu** : `SUPPLIER_INVOICE_NOT_FOUND`, `RECONCILIATION_AMOUNT_MISMATCH`, `RECONCILIATION_TRANSACTION_NOT_DEBIT`.
- **Écartée** : réemployer `INVOICE_NOT_FOUND` / `RECONCILIATION_TRANSACTION_NOT_CREDIT` (le code dirait le contraire
  du cas).
- **Réversible** : non une fois publié.

### C-30-1-19 — `paid_at` hérité de `pay_in_tx`

- **Contexte** : `paid_at = CURRENT_TIMESTAMP(3)` (`supplier_invoices.rs:907`) : l'heure de la saisie, pas la date du
  paiement — déjà vrai depuis la fiche.
- **Retenu** : inchangé dans la 30-1 (défaut voisin non mécanique : `pay` et `confirm_batch` changeraient aussi) ; issue
  demandée.
- **Réversible** : oui.

### C-30-1-20 — `pay_in_tx` prend un `Actor`

- **Retenu** : la clé d'API d'une acceptation figure à l'audit `supplier_invoice.paid` et au lettrage, comme côté client.
- **Réversible** : oui.

### C-30-1-21 — `refuse_wrong_direction`, une fonction pour les deux sens

- **Retenu** : la garde de la 28-6 généralisée, code client inchangé.
- **Réversible** : oui.

### C-30-1-22 — Garde du lot par lecture verrouillante, distincte des vues

- **Contexte** : dans un lot d'acceptation, la vue est née avant le verrou de la facture ; une lecture ordinaire de
  `payment_batch_items` manquerait un lot créé entre-temps.
- **Retenu** : lecture verrouillante après le verrou de la facture ; `in_generated_batch` reste non verrouillant pour
  les vues.
- **Réversible** : oui.

### C-30-1-23 — Généricité par traits

- **Retenu** : `PayableDocument` (`kesh-db`, à côté d'`imputation_order`), `MatchableDocument` (`kesh-reconciliation`) ;
  `propose_matches` et `imputation_order` génériques.
- **Écartées** : une copie fournisseur de `propose_matches` (jumeau) ; convertir la facture fournisseur en `Invoice`
  (faux type, champs inventés).
- **Réversible** : oui.

### C-30-1-24 — Clés du registre de sprint

- **Contexte** : la consigne demande de passer la clé à `ready-for-dev` ; la story est découpée.
- **Retenu** : clé parente `split`, deux clés filles `ready-for-dev` (patron 29-1, 29-4 ; une clé parente
  `ready-for-dev` puis `done` serait comptée en plus de ses filles).
- **Réversible** : oui.

---

## Amendement de l'orchestrateur — 2026-10-15

### C-30-1-25 — `paid_at` = date du paiement, intégré à la 30-1b (amende C-30-1-19)

- **Contexte** : décision de l'orchestrateur — le défaut est sur le chemin même qu'emprunte *Accepter* (`pay_in_tx`),
  et le `CLAUDE.md` veut qu'un défaut découvert dans le flux d'une story liée s'y corrige.
- **Retenu** : `paid_at = payment_date` à minuit, lié en paramètre, pour tous les appelants (fiche : `paymentDate`
  obligatoire saisi ; acceptation : date de paiement de la transaction ; lot : date de confirmation reçue). Aucune
  migration, factures existantes inchangées. Lecteurs inventoriés (fiche, export CSV, deux audits, sauvegarde) ; aucun
  rapport, échéancier, poste ouvert ni lettrage.
- **Écartée** : la limiter à la réconciliation (deux sens de `paid_at` pour une même colonne).
- **Réversible** : oui.

### C-30-1-26 — TTC à plus de deux décimales : angle mort L2, issue à ouvrir

- **Contexte** : l'orchestrateur demandait d'intégrer la correction si elle était mécanique (« arrondir à la saisie,
  comme côté client »).
- **Constat** : la prémisse est fausse ; le client n'arrondit pas à la saisie (quantité et prix bornés à 4 décimales,
  `line_total` arrondi à 4 décimales), il traite l'écart au **règlement** (reste dû au centime, ligne d'arrondi, #476).
  Côté fournisseur, aucune borne d'échelle à la création.
- **Retenu** : non mécanique (arrondir à la création change HT/TVA saisis et la complétion des pièces importées ;
  régler avec une ligne d'arrondi refond `pay_in_tx`) ; L2 écrit, issue à ouvrir. Une correction côté serveur
  (création) resterait compatible avec la 30-2b.
- **Réversible** : oui.

### C-30-1-20 (amendé) — l'acteur par `pay_in_tx_as`, `pay_in_tx` garde sa signature

- **Retenu** : le corps passe dans `pay_in_tx_as(…, actor)` ; `pay_in_tx` devient son enveloppe ; `payment_batches.rs`
  n'est pas touché.
- **Réversible** : oui.

### C-30-1-27 — Recompte des modules de la 30-1b : 5, pas de nouveau découpage

- **Retenu** : `matching`, `repositories/reconciliation`, `repositories/supplier_invoices`, `routes/reconciliation`,
  `features/reconciliation` = 5 ; la règle découpe au-delà de 5.
- **Réversible** : oui.

---

## Remédiation de la validation P1 — 2026-10-15 (arbitrages de l'orchestrateur)

### C-30-1-12 (amendé) — Référence fournisseur : deux égalités normalisées

- **Retenu** : `payment_reference` **et** `supplier_invoice_number` comparés par **égalité** après retrait des blancs et
  mise en majuscules, des deux côtés, contre `coalesce(reference, end_to_end_id, transaction_id)` ; le numéro ne compte
  que s'il a au moins 6 caractères alphanumériques. Aucun containment, aucun préfixe : aucun faux indice ne départage
  contre l'ordre d'imputation ; seule une désignation entière (art. 86 CO) le fait.
- **Écartée** : la règle client (containment, préfixe) sur un numéro libre et court (P1, R8).

### C-30-1-14 (amendé) — Le critère de masquage de la règle

- **Retenu** : une facture fournisseur ouverte à 0,05 près du débit, non candidate, masque la règle si elle porte un
  indice (référence ou fournisseur) ; au montant **exact** et engagée dans un lot `generated`, elle la masque
  **toujours**, avec une raison visible (`notice`) ; sans borne de dates.
- **Écartées** : ne pas masquer (double comptage muet, et double paiement pour le lot — P1 R2, F3) ; masquer sans
  indice hors lot (sur-blocage : tout débit d'un montant égal à une facture d'un autre fournisseur).
- **Réversible** : oui.

### C-30-1-11 (amendé) — Un texte commun vrai des deux côtés, plutôt qu'une clé propre

- **Retenu** : le texte `settlement-cancel-blocked-bank-match` (catalogues, repli serveur, repli de l'écran) dit « depuis
  le détail de l'import bancaire — ou, pour une facture client, depuis sa fiche » ; il est ainsi vrai pour l'écran **et**
  pour un client d'API (P1, R-L1 : le repli serveur disait aussi « cette fiche »).
- **Écartée** : une clé propre au fournisseur (l'écran seul corrigé, le serveur resté faux).

### C-30-1-16 (maintenu) — Pas de garde de score

- **Contexte** : son énoncé (« ≥ 0,5 par construction ») était faux sur un montant signé (P1, R1/F2).
- **Retenu** : la note compare la valeur absolue (AC-b1) ; l'énoncé, corrigé, tient — l'égalité est vérifiée juste
  avant ; T-b17 l'asserte, M-b16 la viole.

### C-30-1-28 — La création d'un lot de paiement est rejouée sur interblocage, dans la 30-1a

- **Contexte** : un lot d'acceptation de plusieurs propositions prend le compte (par `pay_in_tx`) avant les factures
  suivantes ; `create_batch` prend une facture avant le compte : cycle inhérent (C1). `create_batch` n'était pas rejoué.
- **Retenu** : `retry_on_deadlock("payment_batches::create", …)`, comme `confirm` ; dans la 30-1a, pour précéder le
  chemin qui ouvre le cycle.
- **Écartées** : supprimer le cycle (impossible : `pay_in_tx` verrouille le compte, et les propositions sont
  séquentielles) ; le laisser (`500` sans rapport visible sur la génération d'un fichier pain.001).
- **Réversible** : oui.

### C-30-1-29 — Ordre des verrous de l'acceptation fournisseur : facture → compte → articles du lot

- **Contexte** : arbitrage de l'orchestrateur sur F1/R7.
- **Retenu** : le compte bancaire `FOR UPDATE` avant la lecture verrouillante des articles, dans l'ordre de `pay_in_tx`
  et de `create_batch` : une proposition seule ne ferme aucun cycle avec `create_batch` (C3) ; la table complète des
  chemins et des cycles est écrite à l'AC-b10 ; l'intervalle sur `idx_payment_batch_items_invoice` est dit (AC-b8).
- **Réversible** : oui.

### C-30-1-27 (recompté) — Modules après la P1

- 30-1a : `reconciliation_cancel`, `supplier_invoices`, `routes/reconciliation`, `routes/payment_batches`,
  `features/reconciliation` = 5 ; 30-1b : `matching`, `repositories/reconciliation`, `supplier_invoices`,
  `routes/reconciliation`, `features/reconciliation` = 5. Au seuil, sans nouveau découpage ; signal déclaré.

---

## Remédiation de la validation P2 — 2026-10-15 (arbitrages de l'orchestrateur)

### C-30-1-14 (amendé une seconde fois) — Lot `generated` du compte : rien n'est proposé

- **Retenu** : une facture fournisseur ouverte au montant exact, engagée dans un lot `generated` **du compte de la
  transaction**, efface toute proposition de la ligne (facture et règle) et pose la `notice` ; un lot d'un autre compte
  ne masque que la règle, et seulement avec un indice. La sûreté prime sur la commodité (P2, F1, F2).
- **Écartée** : ne masquer que la règle (le paiement du lot proposé contre la mensualité suivante, puis la confirmation
  du lot : un débit, deux règlements).

### C-30-1-12 (amendé une seconde fois) — La référence de paiement est un texte libre

- **Retenu** : même normalisation et même longueur minimale (6 alphanumériques) que le numéro (P2, R3).

### C-30-1-30 — Les conventions de `details` du refus fournisseur

- **Retenu** : montant de la transaction tel quel (négatif), TTC positif ; casse de chaque famille existante (P2, R-L4).

### C-30-1-31 — `create_batch` verrouille ses factures avant toute lecture (défaut préexistant intégré)

- **Contexte** : deux créations de lot concurrentes mettaient la même facture dans deux lots `generated` (vue née à la
  lecture ordinaire du compte, avant le verrou de la facture) — double paiement (P2, F-L11, R-L7) ; arbitrage : intégrer
  à la 30-1a, qui touche déjà `create_batch`, si mécanique.
- **Retenu** : `SELECT … FOR UPDATE` des factures demandées par identifiant croissant, en tête ; la lecture du compte et
  les lectures d'appartenance après. Jugé mécanique.
- **Réversible** : oui.

### C-30-1-32 — Requête masquante sans troncature qui coupe la facture qui bloque

- **Retenu** : tri lot du compte, montant exact, imputation ; plafond 200 ; liste pleine = masquante ; exécutée pour
  tout débit CHF, règle active ou non (P2, F6, R4, R-L5).
- *Annotation du 2026-10-15 (P5)* : **amendé par C-30-1-39** — la requête masquante n'a plus d'indicateur de lot ; tri :
  montant exact d'abord, puis l'ordre d'imputation ; le lot du compte est une fonction à part.

### C-30-1-33 — Le dialogue d'annulation passe à la 30-1b

- **Contexte** : l'AC-a9 portait la 30-1a à six modules.
- **Retenu** : le dialogue (et ses tests) dans la 30-1b, qui touche déjà `features/reconciliation` et produit le premier
  rapprochement fournisseur ; 30-1a et 30-1b à cinq modules.
- **Écartée** : une dérogation à la règle de découpage, ou une troisième sous-story pour une ligne de rejeu.

---

## Remédiation de la validation P3 — 2026-10-15 (arbitrages de l'orchestrateur)

### C-30-1-34 — RETIRÉ (P4) — ~~Le règlement fournisseur en attente de rapprochement bloque la ligne~~

- **Ce qu'il disait (P3)** : une facture payée par virement depuis le compte, au montant exact, dont l'écriture de
  règlement n'est rapprochée de rien, à ±30 jours du débit, effaçait toute proposition de la ligne (notice
  « paiement en attente de rapprochement ») et faisait refuser l'acceptation.
- **Retiré le 2026-10-15, décision de l'orchestrateur après la P4** : il sur-bloquait (une mensualité payée depuis la
  fiche bloquait le débit de la suivante), ne laissait aucun geste sûr à l'écran, ajoutait des lectures verrouillantes
  aux cycles neufs, et chaque passe y trouvait un double comptage nouveau. Le risque **préexiste** à la 30-1 (v0.13.4 :
  une règle peut déjà affecter ce débit) et relève de **#671** : il redevient l'angle mort **L1**, écrit au manuel avec
  le geste sûr (annuler le paiement depuis la fiche, puis accepter la facture dans la réconciliation ; sinon ne pas
  utiliser la règle).

### C-30-1-35 — L'acceptation fournisseur rejoue la garde de la proposition

- **Retenu** : `supplier_payment_hold(…)`, une fonction, lue par `get_proposals` et par l'étape 3 (d) de l'acceptation,
  en lectures verrouillantes après le verrou du compte ; refus `RECONCILIATION_SUPPLIER_PAYMENT_HOLD`.
- **Écartée** : la garde à la proposition seulement (appel direct, écran périmé).
- **Hors** : l'acceptation d'une **règle** (L7, issue demandée).
- *Annotation du 2026-10-15* : l'acceptation d'une règle est **intégrée** par C-30-1-38 (aucune issue) ; la garde
  ne porte plus, depuis la P4, que sur le lot du compte, en lecture ordinaire (C-30-1-39).
- *Annotation du 2026-10-15 (P5, reformulée en P6)* : le corps ci-dessus (« lectures verrouillantes »,
  `supplier_payment_hold`) est **périmé** — la garde (d) est la fonction du lot du compte en lecture ordinaire
  (C-30-1-39) ; la garde ajoutée en P5 à côté d'elle est **retirée** en P6 (C-30-1-40) : (d) est la seule garde
  rejouée.

### C-30-1-36 — `POST /payment-batches` : liste vide court-circuitée, 500 identifiants au plus

- **Retenu** : aucun `SELECT … IN ()` sur une liste vide (sortie `200` d'aujourd'hui) ; borne de 500 à la route (`400`).
- **Réversible** : oui.

### C-30-1-37 — Les `notice` ne conseillent jamais de confirmer un lot

- **Retenu** : « si cette transaction est le paiement de ce lot, annulez le lot ; sinon, affectez à la main » ; pour un
  paiement déjà au grand livre, « n'affectez pas cette transaction » (#671).
- **Écartée** : « confirmez le lot » (mène à L1, une charge comptée deux fois).
- *Annotation du 2026-10-15 (P5, reformulée en P6)* : la seconde `notice` (« paiement déjà au grand livre ») est
  **retirée** avec C-30-1-34 ; celle que la P5 ajoutait (C-30-1-40) est retirée en P6 avec sa garde. Reste une seule
  `notice`, `supplier_invoice_in_payment_batch`, qui ne cite aucune issue.

### C-30-1-38 — L'acceptation d'une règle rejoue le masquage (ex-L7, décision de l'orchestrateur, 2026-10-15)

- **Retenu** : `accept_one_rule` rejoue, par les fonctions de la proposition, le masquage de la 29-6 (crédit) et celui
  de la 30-1b (débit : `supplier_payment_hold`, masquantes à indice, liste pleine) ; refus `RECONCILIATION_RULE_MASKED`.
  Le masquage par une facture candidate n'est pas rejoué (priorité, non interdiction). Aucun module ajouté.
- **Écartée** : une issue séparée (même route, même fichier).
- **Changement de contrat client**, écrit à `api-external.md` et au CHANGELOG.
- *Annotation du 2026-10-15 (P5, déplacée sous son entrée et reformulée en P6 — R10)* : **amendé par C-30-1-39** —
  pour un débit, le rejeu porte sur le lot du compte et `has_strong_invoice_match` (facture candidate, donc rejouée) ;
  ni `supplier_payment_hold` ni les masquantes à indice ne sont rejoués. Le troisième critère que la P5 y ajoutait
  (C-30-1-40) est retiré en P6.

---

## Simplification après la validation P4 — 2026-10-15 (décision de l'orchestrateur)

### C-30-1-39 — On ne découpe pas, on simplifie : une fonction, des lectures ordinaires, une garde de moins

- **Contexte** : signal de recyclage — la classe « double comptage » revenait à chaque passe sous une forme née de nos
  propres gardes (P3, amendement AC-b17).
- **Retenu** : (1) retrait de C-30-1-34 ; (2) aucune lecture verrouillante neuve pour le masquage, ni à la proposition,
  ni à l'acceptation (d), ni dans `accept_one_rule` — la tolérance « ce qui est commis après la vue n'est pas vu » est
  écrite ; (3) une seule fonction source du critère « lot `generated` du compte au montant exact »
  (`find_open_supplier_invoice_in_account_batch`), trois lecteurs ; (4) `accept_one_rule` rejoue, en lectures
  ordinaires, le lot du compte et `has_strong_invoice_match` pour un débit, la condition de la 29-6 pour un crédit
  (`RECONCILIATION_RULE_MASKED`) ; l'imputation délibérée passe par *Affecter manuellement*.
- **Écartées** : un nouveau découpage (le défaut n'était pas la largeur mais la prolifération des gardes) ; garder la
  garde de C-30-1-34 en la resserrant (aurait encore demandé un geste sûr à l'écran et une borne arbitraire).
- **Réversible** : oui.

---

## Remédiation de la validation P5 — 2026-10-15 (arbitrages de l'orchestrateur)

### C-30-1-40 — RETIRÉ (P6) — ~~La référence d'une facture déjà payée bloque la ligne~~

- **Ce qu'il disait (P5, remède de F1 = R1-6)** : quand la référence d'un débit égalait, après normalisation, la
  référence de paiement ou le numéro d'une facture fournisseur **payée** de la société, la ligne ne proposait rien (une
  `notice` le disait), et l'acceptation d'une facture comme celle d'une règle étaient refusées pour ce débit.
- **Retiré le 2026-10-15, décision de l'orchestrateur après la P6** (F1 = R1, HIGH ; signal de recyclage) : il
  sur-bloquait le cas nominal des créanciers à **référence constante** (loyer, abonnement : la même référence QR chaque
  mois) — dès le deuxième mois, la facture ouverte ne pouvait plus être réglée par la réconciliation, même quand le
  mois précédent avait été réglé **et rapproché** par elle — et son remède affiché (annuler le paiement depuis la
  fiche) était refusé ou bouclait ; c'est le défaut même qui avait fait retirer C-30-1-34 en P4. Coût en sus : une
  comparaison normalisée par `REGEXP` qu'aucun index ne sert, sur toutes les factures payées de la société, pour
  presque tout débit (F2). Toute garde « dépense déjà réglée par un autre chemin » recrée ce sur-blocage ; le remède
  est le **rattachement** (#671).
- **Ce qui le remplace** : l'angle mort **L1**, élargi à toutes ses formes — paiement depuis la fiche (avec ou sans
  référence), lot confirmé, facture payée puis annulée dont le règlement reste au grand livre (R4) —, avec le risque
  nommé (la règle, ou une autre facture ouverte du même montant, peuvent être proposées) et le geste sûr vérifié au
  code : ne rien accepter pour ce débit, ni *Affecter manuellement* ni *Éclater* (R2) ; la transaction reste en attente
  (I1, K2) jusqu'à #671 ; exercice ouvert : annuler le paiement depuis la fiche (ou contre-passer le paiement détaché),
  puis accepter la facture. Écrit au manuel, à `api-external.md` et au CHANGELOG ; l'écran ne cite aucune issue.
- **Écartées** : restreindre la garde aux factures payées non rapprochées et sans facture ouverte de même référence
  (F1 a/b, R1) — une garde de plus, à nouveau à prouver passe après passe, qui laisse les formes sans référence et
  annulée ; garder la garde telle quelle.
- **Réversible** : oui.

### C-30-1-41 — `excluding` retiré ; contacts des masquantes chargés, par tranches de 500

- **Contexte** : P5, F4 = R1-3 (le paramètre `excluding` de la fonction du lot n'avait pas d'autre usage) et F5 / R1-7
  (l'indice « fournisseur » des factures masquantes se lit sur leurs contacts, que la proposition ne chargeait pas ;
  `find_contacts_by_ids` refuse au-delà de `MAX_IN_CLAUSE_IDS`).
- **Retenu** : la fonction du lot n'a plus de paramètre `excluding` (aucun autre usage ; (c) couvre la facture acceptée) ;
  les contacts des factures masquantes entrent dans le lot de chargement, découpé par tranches de 500
  (`MAX_IN_CLAUSE_IDS`) ; `accept_one_rule` note avec la même carte de contacts.
- **Écartées** : garder `excluding` sans lecteur ; charger les contacts des masquantes à part, sans découpage (plafond
  de 500 atteignable avec 500 débits) ; noter le rejeu de `accept_one_rule` sans contacts (l'indice par le nom serait
  perdu, M-b33).
- **Réversible** : oui.
- *Complété le 2026-10-15 (P6, R10)* : *Contexte*, *Écartées* et *Réversible* ajoutés, au gabarit des entrées voisines.

---

## Développement de la 30-1a — 2026-10-15 (agent de développement, Opus 5.5)

### C-30-1-42 — Un interblocage est une troisième issue de la course T-a6 ; le test joue les gestes dans l'enveloppe de rejeu de leurs routes

- **Contexte** : l'AC-a4 prévoyait, pour la course dé-rapprochement ↔ annulation de la même facture, que « l'un attend
  l'autre au verrou de la facture ». Constaté au développement (`1213`) : la lecture verrouillante du classement passe
  par l'index `fk_supplier_invoices_settlement_je` et en tient l'enregistrement **avant** la ligne ; l'annulation de la
  facture tient la ligne par sa clé primaire, puis réécrit `settlement_journal_entry_id` et demande l'enregistrement
  d'index. Cycle, victime le dé-rapprochement (la plus légère), qui n'a rien écrit.
- **Retenu** : le cycle est le prix déjà assumé côté client (étape 2 de `cancel_in_tx` : la ligne de règlement avant la
  facture), avec le même remède — les deux routes **rejouent** (`retry_app_on_deadlock` pour le dé-rapprochement,
  `retry_on_deadlock` pour `POST /supplier-invoices/{id}/cancel`, vérifié au code). T-a6 joue chaque geste dans
  l'enveloppe `DbError` du dépôt, comme ses routes, et asserte le même état final des deux issues rejouées. Le cycle est
  écrit au doc-comment de `classify`.
- **Écartées** : verrouiller la facture par sa clé primaire avant le classement (l'identifiant n'est pas connu avant la
  lecture par l'index) ; une lecture ordinaire pour trouver l'identifiant (interdite avant l'étape 3 : elle ferait
  naître la vue avant le verrou de l'exercice — M-a5) ; un test sans rejeu (il rougissait sur un cycle que les routes
  rattrapent).
- **Réversible** : oui.

### C-30-1-43 — `link_bank_transaction_for_test` vit dans `tests/support/`, non dans `test_fixtures.rs`

- **Contexte** : la fiche (T1) plaçait l'aide dans `crates/kesh-db/src/test_fixtures.rs`. L'inventaire des écrivains d'une
  ligne rattachée à un compte bancaire (`kesh-api/tests/bank_account_closed_inventory.rs`, Story 28-4a) balaie `src/`,
  fixtures comprises : le gate a rougi (`every_bank_account_writer_is_classified`, écrivain non classé).
- **Retenu** : l'aide passe dans `crates/kesh-db/tests/support/fournisseur_rapproche.rs` (inclus par `#[path]` dans
  `reconciliation_cancel.rs` et `reconciliation_e2e.rs`), avec le montage du règlement fournisseur rapproché. Le compteur
  de sites de l'inventaire (32) ne bouge pas : la 30-1b, qui le porte à 33 avec `pay_in_tx_as` (AC-b9), reste juste.
- **Écartées** : classer une aide de test au registre des écrivains de production (statut sans objet, compteur décalé
  pour la 30-1b).
- **Réversible** : oui.

### C-30-1-44 — `FORCE INDEX (PRIMARY)` au verrouillage de tête du lot ; le classement fournisseur sans indice

- **Contexte** : l'AC-a9 demandait un plan `range` sur `PRIMARY`, ou à défaut `FORCE INDEX (PRIMARY)` écrit et justifié.
  Constaté (T-a12 étendu, quatre factures) : l'optimiseur de MariaDB 10.11 choisit l'index **couvrant**
  `idx_supplier_invoices_company_date` (il porte `id`), qui verrouillerait toutes les factures de la société dans un
  autre ordre.
- **Retenu** : `FORCE INDEX (PRIMARY)` dans `lock_invoices_sql`, justifié au doc-comment ; T-a12 asserte `PRIMARY` et
  `range`. Le classement fournisseur (AC-a4) garde un plan libre : T-a12 constate `fk_supplier_invoices_settlement_je`
  sur trois factures payées, trois runs ; l'égalité sur l'écriture de règlement est la plus sélective dès qu'il y a plus
  d'une facture payée, et sur une seule le mauvais plan ne verrouillerait qu'elle.
- **Écartées** : un indice aussi sur le classement (une seconde dépendance au nom d'un index, sans défaut constaté).
- **Réversible** : oui.

### C-30-1-45 — La 30-1a est développée sur la branche `story/30-1` du worktree assigné

- **Contexte** : T0 prescrivait une branche `story/30-1a` depuis `main` ; l'orchestrateur a assigné le worktree
  `kesh-30-1`, branche `story/30-1` (qui porte la fiche et sa validation).
- **Retenu** : la consigne de l'orchestrateur ; les commits portent le préfixe `(30-1a)`. La 30-1b suivra dans une story
  distincte, après la revue et le merge de la 30-1a.
- **Réversible** : sans objet.

### C-30-1-46 — Placement et forme de trois tests

- **T-a17** va dans `crates/kesh-api/tests/supplier_settlement_cancel_e2e.rs`, le fichier qui teste déjà
  `POST /payment-batches` par la route (aucun fichier propre aux lots n'existe), dont il réutilise le montage.
- **T-a11** joue la clôture par son effet (`status = 'Closed'` dans une transaction tenue ouverte), patron de
  `a_concurrent_close_waits_for_the_unreconciliation` : la clôture réelle exige un exercice suivant et des écritures de
  bouclement que le montage n'a pas, et c'est le verrou de la ligne `fiscal_years` qui est en jeu.
- **T-a1** : ses trois classements sont le règlement fournisseur, l'écriture propre et le règlement détaché ; le
  classement client est tenu par les tests de route existants (`cancelling_an_invoice_reconciliation_undoes_everything`).
- **Écartées** : un fichier de tests neuf pour T-a17 (harnais recopié une fois de plus).
- **Réversible** : oui.

---

## Développement de la 30-1b — 2026-10-15 (agent de développement, Opus 5.5)

### C-30-1-47 — `MatchProposal.document_id` : les accès des tests renommés, une annotation de type ajoutée

- **Contexte** : l'AC-b1 renomme le champ `invoice_id` en `document_id` et exige que les tests de `matching.rs` passent
  « sans modification de leurs assertions ». Neuf assertions lisent `proposals[i].invoice_id` ou `p.invoice_id`, et
  `propose_matches(&tx, &[])` ne s'infère plus une fois la fonction générique.
- **Retenu** : l'accès au champ est renommé (`.document_id`) — la valeur comparée et l'attendu ne bougent pas — et le
  test de la liste vide écrit `propose_matches::<Invoice>(&tx, &[])`. Aucun attendu n'est touché.
- **Écartées** : garder le nom `invoice_id` pour une facture fournisseur (le champ mentirait) ; un alias.
- **Réversible** : oui.

### C-30-1-48 — La fenêtre porte ses nombres de jours

- **Contexte** : `date_refusal_failed_proposal(bank_transaction_id, refusal, window)` (AC-b2) doit écrire
  `tolerance_days` (1 / 7) et `window_days` (30 / 365), que la fenêtre ne portait pas (seulement ses deux dates).
- **Retenu** : `InvoiceDateWindow` gagne `tolerance_days` et `window_days`, posés par un constructeur unique
  (`with_days`) que partagent `for_payment_date` et `for_supplier_payment_date`. `check` est inchangé. Aucun site
  hors du fichier ne construit la structure par littéral (vérifié par `grep`).
- **Écartées** : passer les deux nombres en paramètres du refus (deux sources pour une même règle) ; deux types de
  fenêtre.
- **Réversible** : oui.

### C-30-1-49 — `find_contacts_by_ids_chunked` au dépôt, lu par la proposition et par le rejeu

- **Contexte** : P5 R1-7 — le lot de contacts doit être découpé par tranches de 500 ; le rejeu de `accept_one_rule`
  doit noter « avec la même carte de contacts ».
- **Retenu** : une fonction de dépôt, `find_contacts_by_ids_chunked(conn, company_id, ids)` (doublons et `0` retirés,
  tranches de `MAX_IN_CLAUSE_IDS`), appelée par `get_proposals` (sur une connexion du pool) et par `load_rule_mask`
  (dans la transaction du lot). Elle découpe aussi le lot des contacts **clients** — sans effet sous 500.
- **Écartées** : un découpage écrit dans la route (deux copies).
- **Réversible** : oui.

### C-30-1-50 — Le masquage partagé : `rule_mask_for` (pur) et `load_rule_mask` (lecture)

- **Contexte** : AC-b17 — « les mêmes fonctions que `get_proposals` (DRY, une fonction `rule_mask_for(tx, …)` lue
  par les deux) ». La proposition masque plus de cas que l'acceptation n'en rejoue (inventaire clos, P5 F6).
- **Retenu** : `rule_mask_for(lot du compte, notes des candidates fournisseur, notes des factures client trop-perçues)
  -> Option<RuleMask>`, pure, qui décide des **trois** cas rejoués ; `get_proposals` l'appelle (et ajoute ses cas
  propres : facture client candidate, masquantes fournisseur non candidates, liste pleine) ; `load_rule_mask` relit,
  en lectures ordinaires et par les mêmes fonctions de dépôt, ce que `rule_mask_for` juge, pour `accept_one_rule`.
  `has_strong_invoice_match`, `client_triples`, `supplier_triples` et `bears_identity_hint` sont des fonctions
  communes.
- **Écartées** : une seule fonction asynchrone qui charge et décide pour les deux (la proposition charge par lot, pas
  transaction par transaction) ; dupliquer les critères dans `accept_one_rule`.
- **Réversible** : oui.

### C-30-1-51 — `SUPPLIER_INVOICE_COLUMNS_SI`, une seconde liste des colonnes

- **Contexte** : les trois requêtes de l'AC-b3 lisent `SupplierInvoice` sur l'alias `si` ; la seule liste existante,
  `FIND_SCOPED_SQL`, est une requête entière, sans alias.
- **Retenu** : une constante publique `SUPPLIER_INVOICE_COLUMNS_SI` dans `supplier_invoices.rs`, à côté de
  `FIND_SCOPED_SQL`, avec la consigne « qui ajoute un champ à l'entité l'ajoute aux deux ». `FIND_SCOPED_SQL` n'est
  pas réécrite (fichier partagé avec la 30-2, diff minimal).
- **Écartées** : `si.*` (colonnes ajoutées plus tard lues sans le dire) ; réécrire `FIND_SCOPED_SQL` sur la constante.
- **Réversible** : oui.

### C-30-1-52 — `amountDue` d'un refus de montant : le TTC tel que la base le porte

- **Contexte** : `RECONCILIATION_AMOUNT_MISMATCH` porte `details.amountDue` (AC-b6) ; le TTC est un `DECIMAL(19,4)`.
- **Retenu** : la valeur brute (`"10.0000"`), comme le `amountDue` brut de `RECONCILIATION_OVERPAYMENT` côté client ;
  un TTC à plus de deux décimales (L2) reste lisible tel quel.
- **Écartées** : arrondir au centime (un TTC sous le centime y paraîtrait égal au débit).
- **Réversible** : oui.

### C-30-1-53 — La sonde de verrou du compte : un fragment de requête qui porte son `FOR UPDATE`

- **Contexte** : T-b34 doit voir l'acceptation bloquée sur le compte (`attendre_une_requete_en_cours`) ; la lecture
  verrouillante de `lock_for_bank_settlement_in_tx` avait le même texte que celle de `pay_in_tx`. Un fragment
  distinctif sans `FOR UPDATE`, défini hors fonction, a fait rougir l'inventaire des gardes du compte clos
  (`every_unlocked_closed_account_guard_is_rechecked_or_justified` : « lecture non verrouillante de `closed_on` »).
- **Retenu** : une macro `bank_settlement_account_lock_marker!` (`… archived AS bank_settlement_lock FROM
  bank_accounts WHERE id = ? AND company_id = ? FOR UPDATE`), source unique de la requête et de la constante de sonde
  `BANK_SETTLEMENT_ACCOUNT_LOCK_MARKER`.
- **Écartées** : exempter le fragment au registre `FENETRE` (une exemption pour une chaîne de test) ; une sonde sur le
  texte de `pay_in_tx` (indiscernable).
- **Réversible** : oui.

### C-30-1-54 — Le classement des erreurs du règlement : deux bras défensifs de plus

- **Contexte** : l'inventaire de l'AC-b6 classe les erreurs de `pay_in_tx_as` ; `BankAccountClosed` et
  `ConfigurationRequired` y sont inatteignables (l'étape 3 tient le compte en exclusif et les a refusés avant).
- **Retenu** : `supplier_payment_error_to_failed_proposal` les classe quand même (`BANK_ACCOUNT_CLOSED`,
  `BANK_ACCOUNT_NOT_CONFIGURED`), plutôt que de les laisser tomber dans `DATABASE_ERROR` ; inventaire recopié au Dev
  Agent Record. Les codes de refus sont écrits **au site de construction** (`error_code: "…"`), forme que recompte
  `failed-proposal-label.ts` — un premier jet en tuple les cachait au décompte (et `RECONCILIATION_TRANSACTION_NOT_CREDIT`
  avec eux), corrigé avant le commit.
- **Réversible** : oui.

### C-30-1-55 — Placement et regroupement des tests

- **Retenu** : les tests de verrouillage au dépôt (T-b22, T-b34, refus de l'étape 3, T-b31) dans
  `payment_batches_repository.rs`, qui a les aides de lot ; T-b29 dans `supplier_invoices_repository.rs` ; les tests de
  route dans `reconciliation_e2e.rs` (T-b32 et T-b33 y sont prescrits), avec des aides de montage dans
  `tests/support/fournisseur_rapproche.rs` (`facture_ouverte`, `transaction_en_attente`, `lot_genere`,
  `autre_compte`). Regroupés : T-b11 et T-b13 ; T-b17 et le premier cas de T-b30 ; T-b18 et T-b20 ; T-b4 a un volet pur
  (`reconciliation.rs`) et un volet de requête (T-b7). Deux tests **ajoutés** (C-30-1-59).
- **Réversible** : oui.

### C-30-1-56 — Captures : schéma clair seulement, hors dépôt ; le manuel garde ses emplacements

- **Contexte** : AC-b12 demande des captures clair et sombre, et une capture au manuel « si la section en porte une ».
  Kesh n'a pas de thème sombre (C-28-1-7, rappelé par l'orchestrateur) ; toutes les captures du manuel sont des
  emplacements (`_placeholder.png`, `% TODO capture`).
- **Retenu** : deux captures en clair (la ligne proposée, le dialogue d'annulation) dans `KESH_E2E_CAPTURES_DIR` ;
  aucune image versée au manuel.
- **Réversible** : oui.

### C-30-1-57 — Le libellé « paiement antérieur à la facture » lit ses jours, client compris

- **Contexte** : AC-b11 — « le libellé lit `details.tolerance_days` (repli 1) ». Le texte d'une seule clé sert les deux
  sortes de factures.
- **Retenu** : « Le paiement est daté de plus de { $days } jour(s) avant la facture… » dans les quatre locales ; le test
  Vitest existant, qui cherchait « plus d’un jour », lit désormais « plus de 1 jour(s) ».
- **Écartées** : deux clés (client, fournisseur) ; garder « un jour » en dur (faux pour 7).
- **Réversible** : oui.

### C-30-1-58 — `RECONCILIATION_RULE_MASKED` : un libellé de repli pour une raison inconnue

- **Retenu** : trois libellés par raison (`payment_batch`, `invoice_candidate`, `invoice_slightly_overpaid`) et une clé
  générique `reconciliation-failed-rule-masked` pour toute autre raison, plutôt que d'attribuer d'office le texte d'une
  des trois.
- **Réversible** : oui.

### C-30-1-59 — Deux tests ajoutés à la fiche, pour deux mutations qui auraient survécu

- **Contexte** : la mutation « la liste pleine ne masque pas » (extension de M-b12 que la fiche prévoit tuée par T-b42)
  n'était éprouvée que par le dépôt (T-b42 vérifie que 200 lignes sont rendues, pas que la route en tire le masquage) ;
  la fenêtre de l'**acceptation** sur l'exigibilité (`window.check(F, X)`) n'était éprouvée par aucun décor où `F` sort
  de la fenêtre et `X` y reste (M-b5 ne visait que la requête).
- **Retenu** : `a_full_masking_list_masks_the_rule` (199 factures : règle proposée ; 200 : masquée) et
  `the_acceptance_window_reads_the_exigibility` (facture datée de D − 400 et échue à D − 300, facture datée de D + 7 :
  proposées **et** acceptées) ; mutations nommées M-b12e et M-b5r au Dev Agent Record.
- **Réversible** : oui.

### C-30-1-60 — Les aides E2E neuves ne remplacent pas les copies des autres specs

- **Contexte** : `createSupplierInvoiceViaApi` reprend le patron de `payment-batches.spec.ts` et
  `wiredBankAccountViaApi` celui de `reconciliation-link.spec.ts`, qui gardent leurs copies locales.
- **Retenu** : les deux aides vont dans `helpers/api-fixtures.ts` pour la spec neuve ; les deux specs existantes ne sont
  pas réécrites dans cette story (une spec qui passe, hors du périmètre de la 30-1b). Dette de duplication écrite ici ;
  la factorisation se fera au premier geste qui touche l'une des deux.
- **Écartées** : réécrire deux specs vertes sans autre raison (risque sans gain pour la recette).
- **Réversible** : oui.
