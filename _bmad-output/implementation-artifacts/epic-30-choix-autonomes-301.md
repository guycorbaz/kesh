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
