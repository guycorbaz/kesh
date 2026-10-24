# Epic 30 — registre des choix autonomes

Chaque choix tranché en autonomie : contexte, option retenue, alternatives écartées, réversibilité. Numérotation
`C-<story>-<k>`. Mise à jour **par union** (`docs/agents-orchestration.md` § 1) : chaque branche ajoute ses entrées dans
un registre propre à sa story (`epic-30-choix-autonomes-<clé>.md`), que l'orchestrateur verse ici au merge.

---

## Story 30-0 — Kickoff (doc seulement)

### C-30-0-1 — Une seule release, v0.14.0, en fin d'epic

- **Contexte** : Guy ne testera qu'en fin d'epic (« ces fonctions sont indispensables à la mise en production ») ; la
  29-1b, mergée, attend déjà la v0.14.0.
- **Retenu** : une seule release **v0.14.0**, après le merge de la dernière sous-story de #685, avec la 29-1b.
- **Écartée** : publier la 30-1 seule en v0.13.5 — sans #685, le processus voulu (saisir la facture depuis la pièce
  puis la régler par la réconciliation) reste à moitié praticable, et Guy n'en fera pas la recette.
- **Réversible** : oui.

### C-30-0-2 — #671 reste hors du périmètre, sauf part mécanique

- **Contexte** : #671 (P2, bug) demande de rattacher aux règlements déjà saisis les transactions comptabilisées par
  d'autres chemins — fiche fournisseur, écriture manuelle, paiement unique de plusieurs factures, lots pain.001. Sa
  première puce est le symétrique fournisseur de la 29-8, voisin de la 30-1.
- **Retenu** : la fiche de la 30-1 dit si ce symétrique sort mécaniquement de son travail ; si oui, elle l'intègre
  et l'issue reçoit un commentaire (elle ne se ferme pas tant que ses autres chemins restent). Sinon, rien.
- **Écartée** : faire de #671 une story de l'epic — quatre chemins hétérogènes, dont un (lots pain.001) « à
  vérifier » ; l'epic deviendrait le plus large de l'année pour une issue qui ne bloque aucune écriture.
- **Réversible** : oui.

### C-30-0-3 — La 30-1 avant la 30-2, fiches en parallèle

- **Contexte** : ordre proposé à Guy et non contesté ; les deux stories touchent les factures fournisseur, mais par
  des côtés différents (réconciliation contre import et saisie).
- **Retenu** : les deux fiches s'écrivent en parallèle, chacune dans son worktree ; le développement de la 30-1 passe
  en premier, celui de la 30-2 suit et se reporte sur `main` après le merge de la 30-1.
- **Réversible** : oui.

---

## Versé par union : `epic-30-choix-autonomes-301.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

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
- *(2026-10-15, revue de code P1 — A5, B2 : **remplacé par C-30-1-61**. « plus de 1 jour(s) » était un recul de
  lisibilité du chemin client et rendait inexacte la citation du manuel.)*

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
- *(2026-10-15, revue de code P1 — A1, décision de l'orchestrateur : **dette soldée pour `wiredBankAccountViaApi`**.
  `reconciliation-link.spec.ts` l'importe, sa copie locale `wiredBankAccount` est retirée. `payment-batches.spec.ts` ne
  porte **pas** la même aide : son décor prend le premier compte bancaire déjà câblé, ou en crée un sur un autre IBAN par
  `PUT`, et désigne le compte créanciers — un autre geste, que la lentille A tenait elle-même pour défendable ; il reste
  tel quel.)*

### C-30-1-61 — Le libellé « paiement antérieur à la facture » : deux clés, un jour et plusieurs jours

- **Contexte** : revue de code P1, A5 et B2 — C-30-1-57 faisait dire « plus de 1 jour(s) » à l'écran client et rendait
  inexacte la citation du manuel (« plus d'un jour »). Un sélecteur Fluent (`{ $days -> [one] … *[other] … }`) ne
  fonctionne pas ici : le serveur pré-résout les messages **sans argument** (`i18n.svelte.ts`), le sélecteur n'atteint pas
  l'écran.
- **Retenu** : `reconciliation-failed-payment-before-invoice` reprend **mot pour mot** son texte de `fecc8566` (« plus
  d'un jour ») et sert `tolerance_days` = 1 et tout repli ; `reconciliation-failed-payment-before-invoice-days` (« plus de
  { $days } jours ») sert une tolérance de plus d'un jour (7 pour un fournisseur). Quatre locales ; `sitesTotal`
  2218 → 2219.
- **Écartées** : le sélecteur Fluent (inopérant) ; garder « jour(s) ».
- **Réversible** : oui.

### C-30-1-62 — `RECONCILIATION_RULE_MASKED` : le geste juste, raison par raison

- **Contexte** : revue de code P1, INFO de la lentille A — les quatre libellés conseillaient « Affecter manuellement »
  sans condition. Pour une facture qui correspond vraiment, ce geste compte la charge ou le produit deux fois ; l'encadré
  L1 du manuel le défend déjà pour une dépense déjà réglée.
- **Retenu** : `invoice_candidate` — recharger la page et **accepter la facture proposée** si ce débit la règle ;
  `payment_batch` — si la transaction est le paiement du lot, l'annuler puis accepter la facture proposée ;
  `invoice_slightly_overpaid` — enregistrer le règlement depuis la fiche de la facture, puis y rattacher la transaction
  (le geste du manuel, § *Algorithme*) ; générique — recharger et traiter la facture. Chaque libellé finit par « N'utilisez
  « Affecter manuellement » que si [la transaction] ne règle aucune facture saisie » — cohérent avec L1 : un débit qui
  règle une facture déjà payée « règle une facture saisie », le geste lui reste donc interdit. Repli, quatre locales,
  manuel (*Application des règles*), `api-external.md`, CHANGELOG et commentaire de l'étape 7 bis alignés.
- **Écartées** : garder « Affecter manuellement » en alternative libre.
- **Réversible** : oui.

### C-30-1-63 — La lecture du contact de l'acceptation, partagée et sans erreur avalée

- **Contexte** : revue de code P1, B1 (et I1, E7) — `accept_one_supplier_invoice` recopiait la lecture du contact
  d'`accept_one_invoice`, sans sa garde `contact_id > 0`, et les deux avalaient une erreur de base (`unwrap_or(None)`).
  Côté client, un score de contact ramené à zéro par l'erreur pouvait se changer en refus `score_zero_no_match` trompeur.
- **Retenu** : `contact_for_score` (routes) — garde `contact_id <= 0` puis lecture ; une erreur rend `DATABASE_ERROR` par
  proposition, comme les autres lectures de l'acceptation (un interblocage y est repris par le rejeu du lot). Les deux
  acceptations l'appellent.
- **Écartées** : garder l'avalement avec un `tracing::warn!` (l'erreur resterait sans effet visible) ; factoriser sans
  changer le comportement.
- **Test** : aucun test ne provoque une erreur de base sur cette lecture (pas d'injection de faute dans le dépôt) ; la
  propagation est vérifiée à la lecture. La garde `<= 0` est équivalente à son absence (aucun contact d'id 0 ou négatif :
  `AUTO_INCREMENT` dès 1) — elle épargne une lecture, elle ne change aucun résultat.
- **Réversible** : oui.

### C-30-1-64 — Constats LOW de la revue de code P1 laissés, et pourquoi

- **E1** (la garde (d) peut nommer la facture acceptée elle-même, vue périmée) : laissé. L'écarter demande de rendre à
  `find_open_supplier_invoice_in_account_batch` le paramètre `excluding` retiré en P5 (C-30-1-41) — trois lecteurs, et
  avec `LIMIT 1` un filtre posé après coup masquerait une autre facture retenue : ce n'est pas une ligne. Refus spurieux,
  sans écriture, rejouable ; écrit en L11 de la fiche.
- **E3** (montant comparé au `total_amount`, règlement posté pour la ligne de crédit de l'achat) : laissé ; aucun chemin
  ne les fait diverger (la création pose la ligne de crédit au total) — garde défensive sans cas.
- **B4** (`MatchProposal.invoice_id` → `document_id`, clé JSON `documentId`) : laissé ; aucun consommateur JSON de cette
  structure (les routes reconstruisent leur réponse) et `kesh-reconciliation` n'est pas publié (seul `kesh-import` l'est).
- **B5** (`NotOpen { status }` non lu) : laissé ; le libellé générique « n'est pas éligible » suffit au geste, un statut
  dans `details` serait un ajout de contrat sans demande.
- **Réversible** : oui.

---

## Versé par union : `epic-30-choix-autonomes-302.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

Fiche : `30-2-documents-factures-fournisseur.md`. Issue [#685]. Choix tranchés à la rédaction de la fiche, le
2026-10-15, en autonomie (Opus 5.5), sur `15686d55`. Format : contexte, retenu, écartées, réversibilité.

[#685]: https://github.com/guycorbaz/kesh/issues/685

---

### C-30-2-1 — Trois sous-stories, le socle d'abord

- **Contexte** : la story touche `kesh-db` (migration, entité, dépôt), `kesh-api` (import, routes des pièces, factures
  fournisseur, écritures, CSP, export CSV), `kesh-i18n`, le frontend (quatre écrans, contacts) et la documentation —
  plus de cinq modules : règle de découpage préventif du `CLAUDE.md`.
- **Retenu** : **30-2a** (tout document entre dans la liste, prévisualisation, retrait — la migration et le socle de
  la prévisualisation), **30-2b** (créer la facture à côté du document, fournisseur proposé ou créé sans quitter),
  **30-2c** (joindre après coup, consulter depuis la facture et l'écriture). Chacune livrable seule : après la 30-2a,
  un document sans QR se complète déjà par le formulaire existant. Une fiche, trois clés, trois PR successives ;
  `refs #685` sur les deux premières, `closes #685` sur la PR de la 30-2c seulement. Le découpage indicatif du
  fichier d'epic est confirmé, à un déplacement près : la prévisualisation va dans la 30-2a (elle sert la liste),
  la consultation depuis l'écriture dans la 30-2c.
- **Écartées** : une story unique (au-delà du seuil, et sept écrans à revoir d'une seule lentille) ; quatre
  sous-stories (le fournisseur en ligne séparé de la création côte à côte : la création n'a pas de sens livrée sans
  lui, la 30-2b tient en quatre modules).
- **Réversible** : oui, avant le développement.

### C-30-2-2 — Garder la table `imported_supplier_invoices`, rendre ses colonnes QR facultatives

- **Contexte** : la table de staging exige les colonnes du QR (`creditor_iban`, `is_qr_iban`,
  `creditor_address_type`, `creditor_name`, `creditor_country`, `reference_type`, `currency` : `NOT NULL`,
  `20260629000001_imported_supplier_invoices.sql`). Un document sans QR n'y entre pas.
- **Retenu** : une migration rend ces sept colonnes `NULL`, pose une contrainte « tout ou rien » (les sept nulles
  ensemble ou renseignées ensemble) et remplace l'index `idx_imported_supplier_invoice` par une contrainte d'unicité
  sur `supplier_invoice_id` (« une facture = un document », filet de la 30-2c). « Le document porte un QR » se lit
  `creditor_iban IS NOT NULL` ; aucune colonne neuve. Le nom de la table ne change pas (un `RENAME TABLE` serait
  breaking pour rien) ; l'écran dit « documents ».
- **Écartées** : (1) une table générique `documents` reliée aux factures — le bon modèle pour #235, mais une
  reprise des lignes existantes (backfill, triage P7, inventaire de sauvegarde, export CSV) hors de proportion ; (2)
  des valeurs sentinelles (`''`, `'CHF'`, `'CH'`, `'NON'`) sans migration — elles inventeraient une devise et un pays
  à un document qui n'en a pas, et `complete_import` routerait un IBAN vide (`Some("")`) : un faux muet.
- **Réversible** : difficilement après publication (une migration publiée ne se défait pas, P8).

### C-30-2-3 — La migration est breaking : `min_required` 0.14.0 et crates 0.14.0 dans la même PR

- **Contexte** : un binaire 0.13.x lit ces colonnes dans des `String` non optionnelles : une ligne sans QR fait
  échouer son décodage (liste, export CSV). `MODIFY COLUMN` entre dans la liste de P3.
- **Retenu** : dernière instruction de la migration `UPDATE _kesh_version SET kesh_version_min_required = '0.14.0'
  WHERE id = 1;` ; **les dix crates passent à 0.14.0 dans la même PR** (P2-bis) ; exemption au registre de rejeu
  (`EXEMPT_MIGRATIONS`, justification standard des bumps, précédents `20261009000001` et `20261011000001`) et
  compteur d'exemptions mis à jour. Si la 30-1 a déjà porté les crates à 0.14.0, la 30-2a le constate et ne bumpe
  que `min_required`.
- **Écartées** : `-- breaking-skip-bump` (P4) — la justification serait fausse : l'ancien binaire lit ces colonnes.
- **Réversible** : non après publication.

### C-30-2-4 — Un QR illisible n'empêche plus l'entrée du document

- **Contexte** : aujourd'hui, `INVALID_SPC_PAYLOAD`, `INVALID_IBAN`, `PDF_RENDER_ERROR` et `FIELD_TOO_LONG`
  envoient le fichier dans `failed/`, comme `NO_QR_CODE_FOUND`. Le parcours voulu fait entrer tout document PDF ou
  image ; la saisie à la main couvre un QR inexploitable.
- **Retenu** : ces cinq cas donnent un document **accepté sans données QR**, avec au rapport `qr: "none"` (aucun QR)
  ou `qr: "unreadable"` et le code d'origine en `qrError` *(précisé le 2026-10-15, validation P4, R2 : « le code d'origine
  du décodage, ou le code du contrôle pur » — `FIELD_TOO_LONG`, `QR_AMOUNT_OUT_OF_RANGE`)*. Le champ trop long se détecte **avant** l'insertion
  (longueurs des colonnes contrôlées sur la valeur à insérer) ; la branche `DataLengthOrRange` reste en filet. Restent
  refusés comme aujourd'hui : type hors liste blanche ou octets incohérents, image non décodable
  (`ImageDecode` → `UNSUPPORTED_FILE_TYPE`), taille, lien symbolique, doublon, erreur de lecture.
- **Écartées** : garder `failed/` pour le QR invalide (le document serait perdu pour la saisie, contre le parcours) ;
  accepter aussi l'image non décodable (le navigateur ne l'afficherait pas davantage).
  *(Corrigé le 2026-10-15, revue de code P1 de la 30-2a, B3 = E6 : la justification est fausse en partie. La crate
  `image` refuse des images que Chromium et Firefox affichent — JPEG CMYK ou à codage arithmétique, certaines variantes
  12 bits *(annoté le 2026-10-15, passe ciblée P2, T1 : le refus du JPEG CMYK n'est pas établi — `image` 0.25 et `zune-jpeg` le convertissent en RVB ; le manuel ne le cite plus)* — et toute image de plus de 10 000 pixels de côté (`MAX_IMAGE_DIMENSION`, `qr_decode.rs`) ; elles partent
  dans `failed/` sous `UNSUPPORTED_FILE_TYPE`, code qui ne dit pas pourquoi. Le refus est maintenu — il couvre le cas
  courant, l'image réellement corrompue ou tronquée —, mais comme **angle mort assumé**, écrit à la fiche, et non plus
  comme un cas sans perte.)*
- **Réversible** : oui.

### C-30-2-5 — « Supprimer de la liste » est l'actuel *Écarter*, sous un libellé neuf

- **Contexte** : Guy demande de « supprimer de la liste » un document qui n'est pas une facture ou un doublon.
  `discard_import` (`routes/imported_supplier_invoices.rs:336`) passe la ligne `discarded`, garde le fichier, et un
  nouveau dépôt du même fichier la réactive (`inbox_import.rs`, réactivation, Story 25-1b).
- **Retenu** : le geste reste `discard`, libellé « Retirer de la liste », avec confirmation ; fichier conservé,
  trace d'audit inchangée.
- **Écartées** : supprimer la ligne et le fichier — le stockage est adressé par contenu et peut être partagé (avec
  une autre société, ou un PDF figé de facture au même octet près : commentaire `inbox_import.rs:556-567`) ; une
  suppression à tort ferait répondre 410 à une pièce émise.
- **Réversible** : oui.

### C-30-2-6 — Prévisualiser par un blob, et élargir la CSP à `blob:`

- **Contexte** : la CSP des pages (`middleware/csp.rs:24-32`) est `default-src 'self'; img-src 'self' data:` — un
  `<img>` ou un `<iframe>` sur une URL `blob:` y est bloqué. Le téléchargement actuel passe par
  `apiClient.getBlob`, qui gère le rafraîchissement du jeton (cookie httpOnly) ; une `<iframe>` pointée sur l'URL de
  l'API ne le gère pas et afficherait un 401 brut à l'expiration du jeton.
- **Retenu** : le composant de prévisualisation lit le document par `apiClient.getBlob`, en fait une URL `blob:`
  (révoquée à la destruction), l'affiche en `<img>` (PNG, JPEG) ou en `<iframe>` (PDF), et pour tout autre type un
  simple lien de téléchargement ; la CSP gagne `blob:` dans `img-src` et deux directives **explicites**,
  `frame-src 'self' blob:` et `object-src 'self' blob:` (validation P1, F5/R6 : `object-src` ne reste pas hérité de
  `default-src`). Aucun changement du `Content-Disposition` de l'API. **Amendé le 2026-10-15 (validation P1)** : aucun
  E2E ne voit un PDF s'afficher (Chromium sans tête n'a pas de visionneuse) ; un relevé dans un vrai navigateur à
  fenêtre est exigé au Dev Agent Record (AC6-bis), et la CSP se corrige s'il échoue.
- **Écartées** : `<iframe src="/api/v1/…/source-document">` avec un `Content-Disposition: inline` (401 brut à
  l'expiration, pas d'état d'erreur maîtrisé) ; un rendu pdf.js embarqué (dépendance lourde pour un aperçu).
- **Réversible** : oui.

### C-30-2-7 — Le fournisseur est proposé, jamais présélectionné

- **Contexte** : règle « un appariement propose, il ne crée jamais » ; le faux rattachement est le danger muet.
- **Retenu** : à partir du nom du créancier du QR, jusqu'à trois fournisseurs actifs proposés, chacun avec un bouton
  « Retenir » ; la liste déroulante reste vide tant que l'utilisateur n'a pas retenu ou choisi. Recherche et
  classement réutilisent le socle des contacts (`listContacts({ search, isSupplier: true })` puis
  `normalizeTerm`/`rank`, `features/contacts/duplicate-probe.ts`). Sans QR, aucune proposition. **Amendé le 2026-10-15
  (validation P1, R1)** : la recherche serveur est un « OU » de mots et `rank` ne filtre pas ; une fonction pure
  `proposeSuppliers` filtre d'abord (tous les mots du plus court des deux noms, formes juridiques et mots de liaison
  retirés, figurent dans l'autre), puis classe. Un fournisseur qui ne partage que « Sàrl » n'est plus proposé.
  **Amendé de nouveau le 2026-10-15 (validation P2, R7, arbitrage de l'orchestrateur : le rappel avant la précision)** :
  deux sources dans l'ordre — les fournisseurs dont une facture passée porte l'IBAN ou le QR-IBAN du créancier (lecture
  neuve `GET /imported-supplier-invoices/{id}/supplier-candidates`, l'IBAN lu sur le document), puis le nom par une clé
  normalisée (`supplierNameKey` : trémas développés `ae`/`oe`/`ue`, accents repliés, ponctuation en espaces, formes
  juridiques retirées comme suites de mots, mots de liaison retirés) avec égalité ou inclusion d'ensembles de mots ;
  trois au plus, motif affiché, aucun présélectionné. La règle « tous les mots du plus court » de la P1 n'était pas
  décidable (tokenisation, mesure) et écartait « Dubarde Sàrl Genève » et « Mueller AG ».
- **Écartées** : présélectionner le premier (un enregistrement sans regarder rattacherait au mauvais fournisseur) ;
  un appariement par IBAN (les contacts ne portent pas d'IBAN) *(écartée levée le 2026-10-15 par l'amendement P2 ci-dessus :
  l'IBAN se lit sur les factures passées, non sur les contacts)*.
- **Réversible** : oui.

### C-30-2-8 — Le fournisseur se crée dans un panneau de la page, au minimum

- **Contexte** : « le créer sans quitter la création de la facture, puis continuer ». Le formulaire de contact de
  `routes/(app)/contacts/+page.svelte` (1161 lignes) n'est pas un composant.
- **Retenu** : un panneau de la page de création : type (Entreprise par défaut), nom (pré-rempli du créancier QR),
  adresse structurée pré-remplie quand le QR est de type `S` (rue, numéro, NPA, localité, pays), `isSupplier` posé ;
  le signal « contacts proches » du socle 22-2a s'affiche avant l'enregistrement (signaler, jamais bloquer). À
  l'enregistrement, le contact créé est ajouté à la liste et **sélectionné** — c'est le geste humain qui le désigne.
  La saisie de la facture (lignes comprises) n'est pas perdue ; *Annuler* ferme le panneau sans rien créer.
- **Écartées** : extraire le formulaire complet des contacts (refonte d'un écran hors périmètre) ; ouvrir la page des
  contacts dans un onglet (on quitte la saisie) ; pré-remplir l'adresse d'un QR de type `K` (lignes combinées dont le
  découpage serait deviné).
- **Réversible** : oui.

### C-30-2-9 — Avec QR, les coordonnées de paiement viennent du document et ne se modifient pas

- **Contexte** : `complete_import` prend aujourd'hui IBAN, référence et montant dans le staging et ignore le corps.
- **Retenu** : à l'écran, IBAN ou QR-IBAN, référence et montant attendu sont affichés pré-remplis et en lecture
  seule ; côté serveur, un corps qui porte l'un de ces champs pour un document **avec** QR est refusé en `400`
  `QR_FIELDS_FROM_DOCUMENT` (un écart silencieux entre ce qu'on a saisi et ce qui est enregistré serait muet).
  Sans QR, ils viennent du corps, validés comme `POST /supplier-invoices`.
- **Écartées** : laisser modifier (le document dit autre chose que la facture) ; ignorer silencieusement le corps
  (comportement actuel, acceptable tant que l'écran ne montrait pas ces champs).
- **Réversible** : oui.

### C-30-2-10 — La complétion reçoit le corps de `POST /supplier-invoices`

- **Contexte** : `CompleteImportRequest` (`routes/imported_supplier_invoices.rs`) double
  `CreateSupplierInvoiceRequest` (`routes/supplier_invoices.rs:238`) moins les champs de paiement et le projet
  (« extension future → None », l. 269).
- **Retenu** : `complete_import` désérialise `CreateSupplierInvoiceRequest` ; la construction de `NewSupplierInvoice`
  est extraite en une fonction commune aux deux routes ; `project_id` est honoré. Compatible avec les clients
  actuels (champs ajoutés optionnels). Cette fonction porte aussi la validation des coordonnées de paiement
  (C-30-2-20).
- **Écartées** : garder deux types (règle DRY).
- **Réversible** : oui.

### C-30-2-11 — Une page de création, un composant de formulaire

- **Contexte** : le formulaire de saisie vit en ligne dans `routes/(app)/supplier-invoices/+page.svelte`, et un
  second, plus pauvre, dans `routes/(app)/supplier-invoices/import/+page.svelte` (complétion).
- **Retenu** : `SupplierInvoiceForm.svelte` (dans `features/supplier-invoices/`) extrait du premier ; page
  `/supplier-invoices/new` qui l'affiche seul (« sans document », avec le scan navigateur 12.4) ou à côté du document
  (`?document=<id>`, aperçu fixe à gauche, formulaire à droite ; empilés sous 1024 px). Le bouton de la liste mène à
  `/supplier-invoices/new` ; *Compléter* devient « Créer une facture » et mène à `?document=<id>` ; le formulaire de
  complétion en ligne disparaît.
- **Écartées** : garder les deux formulaires (DRY) ; un panneau modal (le document doit rester visible pendant toute
  la saisie et la répartition des montants).
- **Réversible** : oui.

### C-30-2-12 — Joindre après coup depuis la fiche de la facture, sans contrôle de montant

- **Contexte** : point 7 du parcours ; l'entrée reste le dossier, donc le document à joindre est un document à
  traiter.
- **Retenu** : « Joindre un document » sur la fiche d'une facture fournisseur **sans** document, quel que soit son
  statut (ouverte, payée, annulée) : sélecteur des documents à traiter, avec aperçu, montant QR affiché à côté du total
  de la facture ; route `POST /supplier-invoices/{id}/source-document` `{ importedSupplierInvoiceId }`. Aucun champ de
  la facture n'est modifié, aucun refus sur un écart de montant : l'humain voit les deux et décide.
- **Écartées** : refuser un écart de montant (une facture peut avoir été saisie à un montant corrigé) ; joindre
  depuis la liste des documents (deux entrées pour un même geste ; l'utilisateur part de la facture qui lui manque).
- **Réversible** : oui.

### C-30-2-13 — Le détachement entre dans la 30-2c, au plus simple (décision de l'orchestrateur, 2026-10-15)

- **Contexte** : un document joint par erreur ne se détachait pas. La fiche proposait d'en faire une limite avec une
  issue ; l'orchestrateur l'a refusé : une dette sans issue est interdite (rétrospective 28), et ouvrir l'issue
  casserait la règle de release (marge d'une issue au tag v0.14.0).
- **Retenu** : AC24 — « Détacher le document » sur la fiche de la facture (Comptable+, confirmation),
  `DELETE /supplier-invoices/{id}/source-document` : le document repasse `to_complete` avec `supplier_invoice_id =
  NULL` et revient dans « Documents à traiter » ; la facture, ses lignes et ses écritures sont inchangées ; audit
  `imported_supplier_invoice.detached` ; tout statut de facture. La transaction ne verrouille que la ligne du document
  (pas de cycle avec le rattachement, AC19). Vérifié au code : aucune contrainte ne lie `status` et
  `supplier_invoice_id`, la réactivation écrit déjà ce `NULL`, l'unicité de l'AC4 admet plusieurs `NULL`.
- **Écartées** : limite écrite avec issue (refusée, ci-dessus) ; supprimer le document au détachement (contre
  C-30-2-5) ; refuser le détachement d'une facture payée (le lien n'a aucun effet comptable).
- **Angle mort** : un document avec QR détaché peut servir à créer une seconde facture ; geste humain, dit au manuel.
- **Réversible** : oui.

### C-30-2-14 — La lecture du document d'une facture s'ouvre à tout rôle authentifié *(titre corrigé le 2026-10-15, validation P2, L1 : il disait « les deux lectures », contre son propre Retenu)*

- **Contexte** : `GET /imported-supplier-invoices/{id}/source-document` et `GET /supplier-invoices/{id}/source-document`
  sont montées parmi les routes Comptable+ (`lib.rs:506-513`), alors que la facture et l'écriture se lisent par tout
  rôle authentifié (`lib.rs:868-875`). « Consultable depuis la facture et depuis son écriture » échouerait en 403 pour
  le rôle Consultation.
- **Retenu** : `GET /supplier-invoices/{id}/source-document` rejoint les lectures (tout rôle authentifié, toujours
  bornée à la société) ; la lecture d'un document **à traiter** reste Comptable+ (la liste l'est).
- **Écartées** : masquer les boutons pour le rôle Consultation (la pièce resterait invisible à qui lit les livres).
- **Réversible** : oui.

### C-30-2-15 — L'écriture d'achat et l'écriture de règlement montrent le document

- **Contexte** : une facture fournisseur possède deux écritures (`purchase_journal_entry_id`,
  `settlement_journal_entry_id`, `DocumentKind::SupplierInvoice`, `journal_entries.rs:2073-2087`).
- **Retenu** : le détail d'une écriture expose `sourceDocument` (`supplierInvoiceId`, numéro de facture fournisseur,
  nom du fichier) si l'écriture est l'achat **ou** le règlement d'une facture fournisseur qui a un document ; la page
  affiche le lien vers la facture et « Voir le document ».
- **Écartées** : l'achat seul (le règlement par la réconciliation de la 30-1 est l'écriture que Guy ouvre depuis le
  relevé).
- **Réversible** : oui.

### C-30-2-16 — #503 reste hors périmètre, signalé comme préalable à la mise en production

- **Contexte** : `.keshbackup` n'emporte aucun fichier (`admin_backup/export.rs:94-97`, `files/` vide ;
  `admin_backup/import.rs:91` refuse un `files/` non vide). La 30-2 fera entrer **davantage** de pièces dans
  `KESH_DOCUMENTS_DIR`.
- **Retenu** : aucune modification de la sauvegarde ; le manuel utilisateur, à la section des documents, renvoie à la
  sauvegarde de `KESH_DOCUMENTS_DIR` décrite au manuel administrateur (l. 793, 1456, 1599) ; le rapport demande à
  l'orchestrateur de porter #503 à l'attention de Guy avant la mise en production.
- **Écartées** : intégrer #503 (changement du format de sauvegarde, version du manifeste, vérification d'empreintes :
  une story à elle seule, non mécanique).
- **Réversible** : oui.

### C-30-2-17 — Le dossier se vide à l'import, comme aujourd'hui

- **Contexte** : #685 dit qu'« après création » le document « quitte la liste à traiter et le dossier d'entrée ». Au
  code, le fichier quitte le dossier **à l'import** : archivé sous `KESH_DOCUMENTS_DIR`, supprimé de l'inbox
  (`inbox_import.rs:623`) ; seul un échec le laisse, dans `failed/`.
- **Retenu** : comportement conservé ; la création fait quitter la **liste** (statut `completed`). Prémisse corrigée
  dans la fiche, à commenter sur l'issue.
- **Écartées** : laisser le fichier dans le dossier jusqu'à la création (chaque import le relirait et le
  signalerait en doublon).
- **Réversible** : oui.

### C-30-2-18 — « Documents à traiter », adresse de page inchangée

- **Contexte** : « Importer des factures », « Factures à compléter », « Compléter » disent une facture ; la liste
  contiendra des documents qui n'en sont pas.
- **Retenu** : titre et menu « Documents à traiter », section « Documents à traiter », bouton « Créer une facture »,
  contrôle K5 de la préparation de clôture « Documents fournisseur à traiter » ; l'URL `/supplier-invoices/import`
  reste (liens de `FiscalYearChecks.svelte`). Les clés i18n gardent leur nom, seuls les textes changent dans les quatre
  locales ; la clé `imported-supplier-invoices-error-no-qr-code-found`, qui n'a plus d'émetteur, est retirée avec son
  entrée de `error-label.ts`.
- **Écartées** : renommer la route (liens et E2E à reprendre pour rien).
- **Réversible** : oui.

### C-30-2-19 — Un document sans QR n'a pas de devise : la facture est en CHF

- **Contexte** : `complete_import` refuse une devise autre que CHF (`routes/imported_supplier_invoices.rs:183`), lue
  dans le staging.
- **Retenu** : le contrôle ne s'applique qu'à un document avec QR ; sans QR, la facture est en CHF comme toute
  facture fournisseur saisie.
- **Écartées** : demander une devise (les factures fournisseur sont en CHF seulement).
- **Réversible** : oui.

### C-30-2-20 — Les coordonnées de paiement se valident dans la conversion commune, pour les deux routes

- **Contexte** : validation P1 (F6, R14). `POST /supplier-invoices` ne valide ni l'IBAN, ni l'exclusivité IBAN /
  QR-IBAN, ni la cohérence QRR : `create_in_tx` insère tel quel (`repositories/supplier_invoices.rs:340-343`,
  `:526-539`). Arbitrage de l'orchestrateur : prescrire la validation.
- **Retenu** (réécrit le 2026-10-15 après la validation P2 — F1-3, F1-4, R6, arbitrage de l'orchestrateur ; la première
  rédaction s'appuyait sur `kesh_qrbill::validation::validate_iban`, qui n'accepte que CH/LI à 21 caractères et aurait
  refusé tout IBAN étranger) : AC12-bis — un seul type par côté. Serveur : `kesh_core::types::Iban` (international,
  MOD-97) pour `creditorIban`, qui ne doit pas être un QR-IBAN ; `kesh_core::types::QrIban` (CH/LI, QR-IID 30000-31999)
  pour `creditorQrIban` ; pas les deux. QR-IBAN ⇒ référence QRR obligatoire et valide (`validate_qrr`) ; IBAN ordinaire
  ou absent ⇒ référence en texte libre (≤ 64), **27 chiffres compris** *(amendé le 2026-10-15, validation P3, F3-3 : la
  P2 refusait une « forme QRR » sans QR-IBAN, ce qui rejetait l'ancienne référence BVR/ESR d'une facture ancienne sans
  bénéfice en aval)*. Normalisation par **une** fonction par côté, qui retire tous les blancs Unicode (insécables
  compris) des IBAN *(amendé en P3, R3 = F3-7 : `Iban::new` n'ôte que l'espace ASCII ; précisé le 2026-10-15, P4 F4 et P5
  T5 : l'ensemble retiré est `White_Space` d'Unicode plus U+200B et U+FEFF, la même liste des deux côtés — AC12-bis)* ; chaîne vide ou blanche =
  absente ; valeurs stockées normalisées. Sept motifs de refus, chacun avec son libellé dans les quatre locales (P3, R4).
  L'effet sur la création manuelle se décrit exactement : table des pays et longueur par pays d'`Iban::new` (des pays
  absents, refusés), lot pain.001 limité à CH/LI (existant). Refus `400 INVALID_PAYMENT_DETAILS`
  (`details.field`, `details.reason`). Frontend : la fonction existante `qrIbanHasValidQrIid`
  (`bank-accounts/+page.svelte:276`) extraite et partagée, complétée de la condition CH/LI. Effet sur la création
  manuelle écrit tel quel au CHANGELOG : refus d'un IBAN faux, d'un QR-IBAN mal placé, des deux ensemble, d'un QR-IBAN
  sans QRR valide ; un IBAN étranger valide (pays de la table) et une référence libre restent acceptés.
- **Écartées** : `kesh_qrbill::validation::validate_iban` (CH/LI seulement : régression silencieuse pour les
  fournisseurs étrangers) ; « aucun contrôle, comme aujourd'hui » en angle mort (dette sans issue interdite) ; valider
  la seule complétion (deux chemins pour une même saisie).
- **Réversible** : oui.

### C-30-2-21 — Le rattachement ne lit pas le lien existant : l'unicité refuse

- **Contexte** : validation P1 (F1, R10). L'étape 3 prévue (lecture verrouillante d'un document déjà lié, sur l'index
  d'unicité) pose un verrou d'intervalle quand aucune ligne n'existe — le cas nominal — et fait s'interbloquer deux
  rattachements vers deux factures voisines ; le rejeu le masquerait. Arbitrage de l'orchestrateur : la voie la plus
  simple.
- **Retenu** : plus de lecture préalable ; après les verrous du staging et de la facture (clés primaires), l'`UPDATE`
  de `mark_completed` rencontre l'unicité de `supplier_invoice_id`, dont la violation se traduit en
  `409 SOURCE_DOCUMENT_ALREADY_ATTACHED`. La transaction ne fait aucune lecture ordinaire (aucun instantané n'y naît).
  Tests sans rejeu (fonctions `_once`) : deux factures différentes, deux succès ; même facture, un `409`.
- **Écartées** : lecture ordinaire après les deux verrous (correcte aussi, mais un second chemin pour le même refus et
  une mutation équivalente de plus) ; garder la lecture verrouillante (verrou d'intervalle).
- **Réversible** : oui.
- **Amendé le 2026-10-15 (validation P3, R2 = F3-1, relevé de l'orchestrateur)** : les deux validations soupçonnaient
  le contrôle de doublon d'InnoDB de poser seul un verrou sur le successeur. **Mesuré au banc** (MariaDB 10.11.16 de dev,
  `REPEATABLE-READ`, tables jetables `inv` / `doc` à `inv_id NULL UNIQUE` et clé étrangère ; A verrouille doc 1 puis inv X
  et écrit le lien sans valider, B fait de même sur doc 2 et inv Y ; couples (10, 20), (20, 10), (30, 10), (10, 30), avec
  et sans liens existants 15 et 25) : B aboutit chaque fois en moins de 0,35 s. Test « B sous 5 s » gardé. **Bascule
  écrite** si le test rougit au développement : sérialisation par la sentinelle `companies … FOR UPDATE` en tête de
  transaction (patron de `supplier_invoices::create_in_tx`), test réécrit sur l'ordre d'événements. *(Corrigé le
  2026-10-15, validation P4, F1 = R1 : `create_in_tx` ne prend la sentinelle que pour une facture à projet, et une
  sentinelle en tête formait des cycles avec la complétion et le règlement ; la bascule prend
  `bank_accounts::acquire_company_sentinel_lock` **après** le staging et la facture, avant l'`UPDATE` — ordre staging →
  facture → `companies`, tableau des chemins à l'AC19.)*

### C-30-2-22 — Le contrôle pur couvre longueur, plage et domaine, avant l'archivage ; filet des contraintes CHECK *(titre à lire « longueur et plage » depuis le 2026-10-15, validation P3 : le domaine est retiré ; le contexte ci-dessous, « 500 sur tout le run … défaut d'origine », était faux à sa date — voir le Retenu)*

- **Contexte** : validation P2 (F1-1, R1, R2, R3). Le montant du QR n'est borné par personne (`parser.rs:145`) ; le type
  d'adresse est pris sans contrôle (`parser.rs:120`) et une valeur hors `K`/`S` viole `chk_imported_si_address_type`,
  que l'import ne capte pas : 500 sur tout le run, fichier resté dans l'inbox, chaque import suivant en échec (défaut
  d'origine) ; le contrôle de la P1 était décrit comme une fonction de `NewImportedSupplierInvoice`, construite **après**
  l'archivage, tout en se disant « avant l'archivage ».
- **Retenu** : une fonction pure de `ScannedQrBill`, appelée avant l'archivage, vérifie longueur (en caractères) et plage
  du montant (`amount_min`..`amount_max`, deux décimales significatives) ; *(le contrôle de domaine de
  `creditor_address_type` d'abord retenu est **retiré le 2026-10-15**, validation P3, R1 HIGH = F3-2 : `from_scanned`
  normalise déjà tout type ≠ `S` en `K`, `imported_supplier_invoice.rs:117-128`, test l. 256 — le « défaut d'origine »
  n'existait pas, et un `CheckConstraintViolation` rend 400, non 500 ; arbitrage P2 de l'orchestrateur pris pour acquis
  sans vérification, reconnu par lui)* ; un QR qui échoue entre
  sans ses données (`unreadable`). Un filet `CheckConstraintViolation` rejoint `DataLengthOrRange` à l'`INSERT`
  (`failed[]`) — défense en profondeur, sans test, aucune contrainte CHECK n'étant atteignable.
  *(Précisé le 2026-10-15, validation P4, R2 = F5 : un code `qrError` neuf, `QR_AMOUNT_OUT_OF_RANGE`, un seul pour la
  plage et les décimales, libellé ×4 ; le contrôle ne porte que sur un montant présent ; tout document `unreadable` dit
  à l'écran qu'il est accepté sans les données du QR.)* Le cas « nom de fichier trop long » est retiré : inatteignable (`NAME_MAX` = 255 octets).
- **Écartées** : laisser le filet seul décider (un QR exploitable en partie perdrait le document) ; garder un test de
  nom trop long (impossible à monter).
- **Réversible** : oui.

### C-30-2-23 — La date de règlement s'affiche formatée (décision de l'orchestrateur, 2026-10-15)

- **Contexte** : constat de la 30-1 — la fiche d'une facture fournisseur payée affiche `paidAt` brut
  (« 2026-06-20T00:00:00 », `supplier-invoices/[id]/+page.svelte:393`) ; avec la 30-1, `paid_at` devient la date du
  paiement à minuit.
- **Retenu** : AC25, dans la 30-2c — `formatSwissDate` existant (`reports.api.ts:193`) sur les dix premiers caractères,
  libellé adapté dans les quatre locales, Vitest, mutation M-c12.
- **Écartées** : une issue séparée (la marge d'issues au tag est d'une seule) ; un formateur neuf (DRY).
- **Réversible** : oui.

---

*Choix du développement de la 30-2a (2026-10-15, Opus 5.5, autonomie), sur `5b566378`, branche `story/30-2`.*

### C-30-2-24 — Le téléchargement reste à côté de l'aperçu

- **Contexte** : AC5 énumère trois gestes par ligne (Aperçu, Compléter, Retirer de la liste) ; la page portait aussi
  « Voir le justificatif », qui **télécharge** le document, et `inbox-import.spec.ts` l'exerce. Un PDF s'affiche dans la
  visionneuse du navigateur, qui sait l'enregistrer ; une image, non.
- **Retenu** : le bouton reste, libellé « Télécharger » (clé `imported-supplier-invoices-view-doc`, texte changé dans les
  quatre locales — le mot « justificatif » sort du vocabulaire, AC8). `DocumentPreview` garde son lien « Télécharger »
  pour les seuls types qu'il n'affiche pas (AC6).
- **Écartées** : retirer le bouton (une image ne s'enregistrerait plus qu'au clic droit, et la couverture E2E du
  téléchargement disparaîtrait) ; un lien de téléchargement dans tout aperçu (AC6 le réserve aux autres types).
- **Réversible** : oui.

### C-30-2-25 — Le contrôle pur du QR vit dans l'entité, et rend un type, non une chaîne

- **Contexte** : AC2 prescrit une fonction pure de `ScannedQrBill`, appelée avant l'archivage ; les largeurs de colonnes
  sont un savoir de `kesh-db`, les codes `qrError` un savoir du rapport d'import.
- **Retenu** : `check_scanned_fits_columns(&ScannedQrBill) -> Result<(), QrColumnsMismatch>` dans
  `kesh-db/src/entities/imported_supplier_invoice.rs`, à côté de `from_scanned` et de `without_qr` ;
  `QrColumnsMismatch::{FieldTooLong { column }, AmountOutOfRange}` et sa méthode `code()` (`FIELD_TOO_LONG`,
  `QR_AMOUNT_OUT_OF_RANGE`). Les deux filets de l'`INSERT` gardent les codes prescrits (`DataLengthOrRange` →
  `FIELD_TOO_LONG`, `CheckConstraintViolation` → `INVALID_SPC_PAYLOAD`, AC2).
- **Écartées** : la fonction dans `inbox_import.rs` (les largeurs y seraient recopiées loin du schéma) ; un `&'static str`
  nu (la colonne en défaut se perdrait, et un test ne pourrait pas la nommer).
- **Réversible** : oui.

### C-30-2-26 — Le vocabulaire du journal d'activité suit l'écran ; les messages d'erreur du serveur, non

- **Contexte** : AC8 fait inventorier les jetons « importée », « complét », « écart » dans les quatre locales ; les
  libellés d'audit (`audit-log-action-imported-supplier-invoice-*`, `audit-log-entity-*`) disaient « Facture importée
  créée / complétée / écartée / réactivée ». Les messages d'`AppError` du serveur (`errors.rs`, « Facture importée
  introuvable ») ne sont pas affichés par la page, qui traduit le code ; et la 30-1a modifie `errors.rs` en parallèle.
- **Retenu** : les cinq libellés d'audit passent au vocabulaire du document (« Document importé », « Document traité
  (facture créée) », « Document retiré de la liste », « Document remis dans la liste », entité « Document fournisseur »),
  dans les quatre locales. `errors.rs` n'est pas touché ; le message de `CURRENCY_NOT_SUPPORTED`, construit dans la route,
  devient « … Retirez le document de la liste. » (AC7).
- **Écartées** : garder « Facture importée » au journal (le journal nommerait « facture » un document qui n'en est pas
  une) ; réécrire les messages de `errors.rs` (conflit avec la 30-1a, sans effet à l'écran).
- **Réversible** : oui.

### C-30-2-27 — Le développement part de `story/30-2`, avant le merge de la 30-1a

- **Contexte** : T-a0 et C-30-0-3 font partir la 30-2a d'`origin/main` **après** le merge de la 30-1 ; l'orchestrateur a
  lancé le développement en parallèle de la 30-1a, et demandera un rebase après son merge.
- **Retenu** : développement sur `story/30-2` (`5b566378`) ; T-a0 reste à faire au rebase. La 30-1a n'ajoute aucune
  migration : l'horodatage `20261015000001` reste le plus récent, et les dix crates n'étaient pas à 0.14.0 — la 30-2a les
  y porte. Conflits probables écrits au Dev Agent Record.
- **Écartées** : attendre le merge (consigne de l'orchestrateur).
- **Réversible** : oui (rebase).

### C-30-2-28 — Les dossiers E2E du worktree sont nommés par la clé, et passés au runner

- **Contexte** : le spec lit `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` avec pour défauts ceux de la recette
  (`/tmp/kesh-e2e/…`) ; d'autres agents peuvent monter la même recette au même moment, dans les mêmes dossiers.
- **Retenu** : le montage du worktree emploie `/tmp/kesh-e2e-302a/{inbox,documents,backup}`, passés au backend **et** au
  runner (consigne « fichiers temporaires nommés par ta clé ») ; les défauts du spec restent ceux de la recette.
- **Écartées** : les dossiers par défaut (un import d'un autre agent prendrait mes fichiers, et inversement).
- **Réversible** : oui.

### C-30-2-29 — Ce qui revient à la 30-2b et à la 30-2c n'est pas écrit d'avance

- **Contexte** : AC9-bis demande, au *Passer à la 0.14.0* et au bandeau du CHANGELOG, la validation des coordonnées de
  paiement (`INVALID_PAYMENT_DETAILS`, « à compléter par la 30-2b ») et la lecture du document ouverte aux clés `read`
  (30-2c) ; rien de cela n'est livré par la 30-2a.
- **Retenu** : la 30-2a n'écrit que ce qu'elle livre (champs nullables, `qr`/`qrError`, disparition de
  `NO_QR_CODE_FOUND`) ; les deux points s'ajouteront avec leur code. Écart à la fiche déclaré au Dev Agent Record.
- **Écartées** : annoncer au manuel un comportement absent du code (le manuel dirait faux jusqu'au merge de la 30-2b).
- **Réversible** : oui.
- *(Suite donnée le 2026-10-15 par la 30-2b : `INVALID_PAYMENT_DETAILS` et `QR_FIELDS_FROM_DOCUMENT` sont écrits au
  *Passer à la 0.14.0*, au bandeau du CHANGELOG et à `api-external.md` ; la lecture par les clés `read` reste à la 30-2c.)*

---

*Choix de la 30-2b — développement du 2026-10-15 (Opus 5.5, autonomie), sur `f1508653` (30-1a et 30-2a mergées).*

### C-30-2-30 — La conversion commune vit sur `CreateSupplierInvoiceRequest`, la source des coordonnées est un type

- **Contexte** : AC12-bis et AC13 veulent une seule conversion vers `NewSupplierInvoice`, portant la validation des
  coordonnées, pour `POST /supplier-invoices` et la complétion ; avec QR, les coordonnées viennent de la ligne verrouillée
  et le corps ne doit pas en porter (AC11).
- **Retenu** : `CreateSupplierInvoiceRequest::to_new_supplier_invoice(company_id, PaymentSource)` dans
  `routes/supplier_invoices.rs`, avec `PaymentSource::{Body, Document { payment, expected_payment_amount }}` ; `Body`
  passe par `validate_payment_details` (seule définition serveur), `Document` refuse `QR_FIELDS_FROM_DOCUMENT` si le corps
  porte l'une des quatre coordonnées (une chaîne blanche vaut absente). La complétion construit la source après le
  verrou et le routage IBAN existant. `normalize_payment_field` est `pub(crate)` (aussi employée par `supplier-candidates`).
  Deux variantes neuves d'`AppError` : `InvalidPaymentDetails { field, reason }` et `QrFieldsFromDocument`, messages par
  les clés `error-invalid-payment-details` et `error-qr-fields-from-document` (quatre locales).
- **Écartées** : la validation dans `kesh-core` (le type `Iban` y est déjà ; c'est la politique de la route — quel champ,
  quel motif — qui est neuve) ; réutiliser `ImportCompletionRejected` pour `QR_FIELDS_FROM_DOCUMENT` (message en dur, non
  traduit, et le refus vaut au-delà de la complétion).
- **Réversible** : oui.

### C-30-2-31 — Ordre et champ des motifs de refus

- **Contexte** : AC12-bis nomme sept motifs et trois champs, sans ordre ni champ pour `BOTH_IBANS`.
- **Retenu** : `BOTH_IBANS` d'abord, sur `creditorIban` ; puis l'IBAN (`INVALID_IBAN`, `QR_IBAN_IN_IBAN_FIELD`), puis le
  QR-IBAN (`INVALID_IBAN` si `CoreError::InvalidIban`, `NOT_A_QR_IBAN` sinon), puis la référence. Avec un QR-IBAN, la
  référence est normalisée comme un IBAN (blancs de l'ensemble écrit) avant `validate_qrr` ; sans QR-IBAN, elle est
  rognée (`trim`) et comptée en caractères (64 au plus).
- **Écartées** : signaler `BOTH_IBANS` sur `creditorQrIban` (l'écran n'a qu'un champ IBAN : le choix est sans effet
  visible).
- **Réversible** : oui.

### C-30-2-32 — La lecture `supplier-candidates` vit dans le dépôt des contacts

- **Contexte** : AC14 demande les fournisseurs dont une facture passée porte l'IBAN ; la 30-1b, développée en parallèle,
  modifie `repositories/supplier_invoices.rs`.
- **Retenu** : `contacts::find_suppliers_by_invoice_iban` (`kesh-db/src/repositories/contacts.rs`) — elle rend des
  contacts, réutilise `COLUMNS`, et n'entre pas en conflit avec la 30-1b. Sous-requête `GROUP BY contact_id` avec
  `MAX(invoice_date)`, `status <> 'cancelled'`, `REPLACE` de l'espace, U+00A0, U+202F sur `UPPER(…)` des deux colonnes,
  `LIMIT` en SQL ; l'IBAN du document est normalisé côté route (`normalize_payment_field` puis majuscules).
- **Écartées** : le dépôt des factures fournisseur (conflit certain avec la 30-1b) ; un dépôt neuf (un module pour une
  fonction).
- **Réversible** : oui.

### C-30-2-33 — La réponse de la complétion est relue par la fonction du GET dès la 30-2b

- **Contexte** : AC13 (30-2b) dit « la réponse de la complétion est relue par la même fonction que le GET (AC18) une fois
  la transaction validée » ; AC18 (`sourceDocument`) est en 30-2c.
- **Retenu** : `complete_import` répond par `SupplierInvoiceResponse::load_with_settlement_cancellation` après le
  `COMMIT` (patron de `pay`) ; la 30-2c n'aura qu'à y ajouter `sourceDocument`. Test : `cancellable` vaut `true` dans la
  réponse (il vaut `null` par `from_parts`).
- **Écartées** : attendre la 30-2c (AC13 est en 30-2b).
- **Réversible** : oui.

### C-30-2-34 — Le formulaire porte les propositions et le panneau ; la page les calcule

- **Contexte** : AC14 (propositions), AC15 (panneau « Nouveau fournisseur ») et M-b5 (« Vitest de la page qui observe
  `listContacts` »).
- **Retenu** : la page `new` appelle `getSupplierCandidates` et `listContacts({ search: searchTermsFor(nom), isSupplier:
  true, limit: 20 })`, compose par `proposeSuppliers` et passe la liste au formulaire, qui l'affiche (« Proposé : … »,
  motif, *Retenir*). Le panneau `SupplierCreatePanel` vit dans le formulaire (un `<div role="group">`, jamais un `<form>`
  imbriqué) ; il crée par `createContact`, le formulaire ajoute le contact à sa liste (`bind:suppliers`) et le
  sélectionne. *Retenir* ajoute aussi le contact à la liste s'il n'y est pas (la liste déroulante est bornée à 200).
  La sonde « contacts proches » du panneau suit le patron de la page des contacts (`probeTerm`, `rank`,
  `describeProches`, cinq au plus), avec un délai de 300 ms.
- **Écartées** : le panneau dans la page (il devrait alors piloter l'état du formulaire de l'extérieur) ; un modal
  (C-30-2-8).
- **Réversible** : oui.

### C-30-2-35 — Les clés du formulaire passent au préfixe `supplier-invoices-` ; 24 clés retirées

- **Contexte** : `lint-i18n-ownership` n'admet, dans `lib/features/supplier-invoices/`, que des clés
  `supplier-invoices-*` ; le formulaire de complétion de la page d'import employait des clés
  `imported-supplier-invoices-*`.
- **Retenu** : les libellés repris (total TTC, écart, sous-centimes, refus de la complétion) renaissent sous le préfixe
  du composant ; les 23 clés `imported-supplier-invoices-*` devenues orphelines et `supplier-invoices-form-close` sont
  retirées des quatre locales (recherche de chaque clé dans `frontend/src` et `crates`). `imported-supplier-invoices-completed`
  (« Facture créée. ») est **gardée** et sert la notification de la page `new` : le glossaire i18n l'a pour référence du
  verbe *créer*. Les clés d'erreur d'import (`-error-*`) restent : la carte `IMPORT_ERROR_LABELS` les lit par préfixe.
- **Écartées** : ajouter les clés à la liste des violations connues du lint (dette #30, à ne pas grossir).
- **Réversible** : oui.

### C-30-2-36 — La création est réservée aux rôles qui écrivent, à l'écran aussi

- **Contexte** : AC16 — « pour un autre rôle, le bouton « Enregistrer une facture » et la page ne proposent rien
  (patron existant de la liste) » ; or la liste montrait le bouton à tout rôle.
- **Retenu** : le patron `canWrite` (Admin ou Comptable) de la fiche de la facture : le bouton de la liste est masqué, la
  page `new` affiche « Votre rôle ne permet pas de saisir une facture fournisseur. » sans rien charger.
- **Écartées** : garder le bouton et laisser le serveur refuser (403 après saisie).
- **Réversible** : oui.

### C-30-2-37 — Un paramètre `document` illisible vaut « plus à traiter »

- **Contexte** : AC10 : inconnu, autre société, plus `to_complete` → même message.
- **Retenu** : `?document=abc` (non entier) est traité comme un document inconnu — même message, même lien, aucun
  formulaire. Le document se cherche dans `GET /imported-supplier-invoices?status=to_complete`.
- **Écartées** : ignorer le paramètre et ouvrir la saisie sans document (on croirait avoir lié la pièce).
- **Réversible** : oui.

### C-30-2-38 — `normalizeIban` passe aussi en majuscules ; le total ne se compare qu'au montant du QR

- **Contexte** : AC12-bis prescrit l'ensemble de blancs de `normalizeIban` ; AC11 compare le total au montant du QR.
- **Retenu** : `normalizeIban` retire les blancs **et** passe en majuscules (comme `Iban::new` côté serveur) — sans effet
  sur la validation, utile à la classification (`ch44…` reconnu QR-IBAN). Le total vivant n'est comparé qu'au montant
  du QR (document avec QR) ; le montant attendu saisi à la main n'est pas une cible (angle mort écrit de la fiche :
  `expectedPaymentAmount` n'est validé ni contre le total).
- **Écartées** : comparer aussi au montant attendu saisi (un indicateur rouge sans refus serveur derrière tromperait).
- **Réversible** : oui.

### C-30-2-39 — Les captures clair et sombre se prennent dans les E2E E1 et E2

- **Contexte** : T-b7 demande des captures de la page côte à côte et du panneau, clair et sombre.
- **Retenu** : `supplier-documents.spec.ts` (E1, E2) capture la page en clair puis en sombre (`page.emulateMedia`) dans
  `KESH_E2E_CAPTURES_DIR` s'il est posé (hors dépôt), sinon dans la sortie du test — patron de
  `reconciliation-reject-unmatched.spec.ts`. ⚠️ Kesh n'a pas de thème sombre (C-28-1-7) : seule la préférence du
  navigateur change.
- **Écartées** : un script à part (non rejoué par la suite).
- **Réversible** : oui.

### C-30-2-40 — Le README marque l'ensemble « Documents à traiter » comme à paraître

- **Contexte** : `README.md:44` cochait ✓ les documents sans QR et l'aperçu (30-2a), non publiés ; la 30-2b y ajoute la
  création côte à côte.
- **Retenu** : la ligne décrit la création côte à côte et le fournisseur proposé, et porte *(documents sans QR-code,
  aperçu et création côte à côte à paraître en v0.14.0)*, sur le patron de la ligne des transitoires ; « import lancé
  depuis l'écran » (garde G5) est conservé mot pour mot.
- **Écartées** : n'annoncer que la 30-2b (la ligne mentirait par omission sur la 30-2a).
- **Réversible** : oui.

### C-30-2-41 — Entrée dans le panneau « Nouveau fournisseur » crée le fournisseur, jamais la facture (revue P1, B1 = E1 = A1)

- **Contexte** : le panneau (C-30-2-34) vit dans le `<form>` de la facture ; Entrée dans l'un de ses `<input>`
  déclenchait la soumission implicite de ce formulaire — avec un fournisseur déjà sélectionné et des lignes valides, la
  facture s'enregistrait sur ce fournisseur-là pendant que l'on en tapait un autre : faux rattachement muet.
- **Retenu** : un `onkeydown` sur le conteneur du panneau (`onPanelKeydown`) : Entrée dont la cible est un `<input>`
  (hors composition IME) → `preventDefault()` — ce qui annule la soumission implicite — puis `create()` si aucune
  création n'est en cours. Entrée sur un bouton garde son clic ; un `<select>` ne soumet pas. Tests : Vitest qui modèle
  la soumission implicite (keydown annulable, puis `requestSubmit()` s'il n'a pas été annulé — jsdom ne l'implémente
  pas), sur chacun des champs en mode Entreprise (6) et Personne (7), plus un **témoin** qui montre que le modèle soumet
  bien depuis un champ de la facture ; E2E E2 : Entrée dans le nom au lieu du clic. Mutation M-b20.
- **Écartées** : sortir le panneau du `<form>` (il faut alors le replacer par CSS à côté du sélecteur, ou un portail :
  plus de code pour le même effet) ; un `<form>` distinct relié par l'attribut `form` (chaque champ du panneau devrait
  le porter, et un oubli rouvrirait le défaut en silence) ; Entrée qui ne fait rien (moins attendu qu'une création).
- **Réversible** : oui.

### C-30-2-42 — Relecture après COMMIT de la complétion : l'échec reste une erreur, écrit en angle mort (revue P1, B2 = A9)

- **Contexte** : `complete_import` relit la facture après le `COMMIT` (C-30-2-33) ; une erreur de base entre le
  `COMMIT` et la relecture rend une erreur sur une facture bel et bien créée, et un nouvel essai donne
  `409 IMPORT_NOT_PENDING_COMPLETION`.
- **Retenu** : **angle mort écrit** (fiche, doc-comment de `complete_import`), non corrigé. Le même patron vit dans
  `pay`, `cancel` et l'annulation du règlement ; la fenêtre est un incident de pool entre deux requêtes ; aucune
  donnée n'est perdue ni faussée, et le 409 dit « déjà traité » — l'écran l'affiche en tête du formulaire, et la
  facture créée est dans la liste des factures fournisseurs.
- **Écartées** : repli sur `from_parts` avec `tracing::warn!` — une branche qu'aucun test n'exerce sans injection de
  panne, propre à une seule des quatre routes du patron ; à traiter, si on le traite, pour les quatre ensemble.
- **Réversible** : oui.

### C-30-2-43 — Page `/supplier-invoices/new` : `document` lu une fois, `goto` rejeté — angles morts écrits (revue P1, B4)

- **Contexte** : `documentId` est lu à l'initialisation de la page ; un passage `?document=5` → `?document=6` sans
  quitter la page la laisserait sur le 5. Si `goto` rejette après la création, le formulaire se réactive et un second
  clic peut créer un doublon (sans document ; avec document, `409`).
- **Retenu** : **angles morts écrits** à la fiche, non corrigés : aucun lien de l'application ne fait passer d'un
  document à l'autre sans quitter la page (liste → page → retour détruit le composant), et un `goto` qui rejette après
  une création réussie suppose une navigation annulée entre-temps.
- **Écartées** : `$derived` sur `page.url` avec `{#key}` (rechargement complet des données à justifier par un parcours
  qui n'existe pas) ; `try/catch` autour de `goto` (ne ferme pas la fenêtre du doublon : le bouton se réactive quand
  même).
- **Réversible** : oui.

### C-30-2-44 — Un seul ensemble de blancs côté serveur, et des points de code explicites côté écran (revue P1, B3, A6 = E7)

- **Contexte** : la référence libre était rognée par `trim()` (White_Space seul) alors que `present()` ignore aussi
  U+200B et U+FEFF : un U+200B final collé depuis un PDF était stocké et compté. `iban.ts` et `iban.test.ts` portaient
  des caractères invisibles littéraux.
- **Retenu** : `is_payment_blank` est la seule définition de l'ensemble côté serveur, partagée par
  `normalize_payment_field` et par le rognage de la référence libre (`trim_matches`), la longueur comptée après ce
  rognage, en caractères. Côté écran, l'expression régulière se construit par `String.fromCodePoint(0x85, 0x200b)` et
  le test liste ses sept blancs par points de code : la source ne porte plus aucun caractère invisible (l'outil
  d'édition convertit en caractère brut une séquence `​` écrite dans un littéral).
- **Écartées** : garder `trim()` et écrire l'angle mort (un caractère invisible stocké fait manquer un rapprochement).
- **Réversible** : oui.

---

*Choix du développement de la 30-2c (2026-10-16, puis reprise du 2026-10-20, Opus 5.5, autonomie), sur `1aca02d2`,
branche `story/30-2c`. C-30-2-45 et C-30-2-46 sont cités par le code écrit le 2026-10-16 par l'agent interrompu ; leur
entrée est écrite par la reprise.*

### C-30-2-45 — Les tests de concurrence appellent `attach_in_tx` / `detach_in_tx`, non les fonctions `_once` de la route

- **Contexte** : AC19 et AC24 demandent des tests « en appelant les fonctions `_once` sans rejeu ». Les `_once` vivent
  dans `kesh-api` (elles ajoutent l'audit à la transaction du dépôt) ; les tests qui doivent **tenir** une transaction
  ouverte pendant qu'une autre attend (sondes `attendre_une_requete_en_cours`, `sonde_verrou_nowait`) vivent dans
  `kesh-db`, qui ne voit pas `kesh-api`.
- **Retenu** : les tests de concurrence (`crates/kesh-db/tests/supplier_document_attachment.rs`) ouvrent et valident
  eux-mêmes leurs transactions autour de `attach_in_tx` / `detach_in_tx`, **sans rejeu**. La `_once` de la route n'y
  ajoute qu'un `INSERT` dans `audit_log`, qui ne touche aucune des lignes en jeu ; les routes sont exercées à part, par
  les tests HTTP de `inbox_import_e2e.rs` (succès, refus, RBAC, audit).
- **Écartées** : exposer les `_once` de `kesh-api` aux tests de `kesh-db` (dépendance inverse) ; écrire les tests de
  concurrence dans `kesh-api` (les sondes et le décor `fournisseur_rapproche` sont dans `kesh-db`).
- **Réversible** : oui.

### C-30-2-46 — Bascule écrite de l'AC19 : le rattachement prend la sentinelle de la société après le document et la facture

- **Contexte** : le test déterministe de l'AC19 (A rattache D1 → I1 sans valider ; B rattache D2 → I2 doit aboutir sous
  5 s) **a rougi sur le code non muté**, sur le vrai schéma (2026-10-16, `kesh-gate-logs/302c-db-1.log` :
  `B (D2 → I2) doit aboutir pendant que A tient D1 → I1 : Err(Elapsed(()))`) — contrairement au banc de la fiche (tables
  jetables, où B aboutissait). **Mesuré** : B attend dans son `UPDATE` tant que A n'a pas validé. **Raisonné, non
  mesuré** : le contrôle de doublon de l'index d'unicité poserait un verrou partagé sur l'intervalle jusqu'au supremum
  quand aucune clé n'existe au-delà, et l'insertion de l'autre rattachement y attendrait ; deux contrôles accordés
  avant les deux insertions s'interbloqueraient (aucun test ne force cet interblocage).
- **Retenu** : la **bascule écrite par la fiche** (AC19, P4) — `bank_accounts::acquire_company_sentinel_lock` prise
  **après** le staging et la facture, **avant** l'`UPDATE` : ordre staging → facture → `companies` → entrée d'unicité.
  Le test devient « B attend la sentinelle (vu dans `PROCESSLIST`), puis aboutit après le `COMMIT` de A, sans
  interblocage ni refus » (`two_attachments_to_two_invoices_are_serialized_by_company`) ; mutation ajoutée **M-c13**
  (sentinelle retirée). **Cycle relevé avec l'acceptation de la 30-1b** (ligne « à établir » du tableau de l'AC19) : un
  lot de plusieurs propositions tient `companies` en partagé dès l'insertion de la première écriture, puis verrouille
  la facture fournisseur d'une proposition suivante (`pay_in_tx_as`, `FOR UPDATE` de la facture) ; un rattachement
  qui tient cette facture et demande `companies` en exclusif ferme le cycle. Rare (même facture, au même instant) ; les
  deux routes sont rejouées sur interblocage (`retry_app_on_deadlock` pour le rattachement, l'acceptation est
  `Rejouee`) ; écrit au point (vi) de `audit_route_registry.rs` et à `api-external.md`.
- **Écartées** : garder le code sans sentinelle (le test rouge dit que deux rattachements simultanés vers deux
  factures s'attendent et peuvent s'interbloquer — rejoués, mais l'AC19 promettait qu'ils ne s'attendent pas) ;
  sentinelle en tête (deux cycles, P4 de la validation).
- **Réversible** : oui.
- **Annotation du 2026-10-20 (revue de code P1, B3 = E2, E7, A1, A6)** — l'entrée ci-dessus disait juste à sa date
  pour ce qu'elle mesurait, mais **incomplète** sur le cycle :
  - **Second partenaire du cycle** : la confirmation d'un lot de paiement (`payment_batches::confirm_batch`, boucle
    de `pay_in_tx` par article) tient `companies` en partagé après le règlement du premier article, puis pose
    `FOR UPDATE` sur la facture du suivant — même forme que l'acceptation de la 30-1b. Rejouée
    (`retry_on_deadlock("payment_batches::confirm")`) : pas de défaut de comportement. Écrit au doc-comment
    d'`attach_in_tx`, à la doc de la route, au point (vi) d'`audit_route_registry.rs` et à `api-external.md`.
    **Réfuté** : la suppression d'un import (`bank_imports::delete`), citée par la lentille B, ne verrouille aucune
    facture et n'écrit aucune écriture (`delete_unreconciled_in_tx` : entête, transactions de l'import, compte
    bancaire) — elle n'est pas partenaire du cycle.
  - **Attente sous la sentinelle (E7)** : un rattachement dont l'`UPDATE` attend l'entrée d'unicité d'un
    détachement non validé tient la sentinelle `companies` en exclusif pendant cette attente ; toute écriture de
    la société (verrou partagé par clé étrangère) attend alors aussi. Sans cycle — le détachement ne demande rien
    d'autre — et bref (la durée d'une transaction de détachement). Conséquence assumée de la bascule.
  - **Mesuré / raisonné (A6)** : le doc-comment d'`attach_in_tx` écrivait au présent l'explication par le verrou
    d'intervalle ; il la dit désormais « retenue, raisonnée et non mesurée », comme le fait cette entrée.
  - **Preuve de M-c11 redéfinie (A1)** : sous la bascule, la preuve porte sur l'**ordre des événements** —
    l'attente du rattachement doit être dans son `UPDATE`, après la sentinelle (sonde de `PROCESSLIST` sur
    `UPDATE imported_supplier_invoices` et `supplier_invoice_id`, test
    `attach_waits_for_an_uncommitted_detach_then_succeeds`). M-c11 meurt parce qu'elle déplace l'attente dans un
    `SELECT … FOR UPDATE`, non par un comportement faux ; la propriété « aucun verrou d'intervalle de notre fait »
    n'a **pas de preuve déterministe** sous la sentinelle, et une lecture non verrouillante du lien ne serait pas
    vue. Écrit au doc-comment du test.

### C-30-2-47 — La fiche montre le QR-IBAN (ajout de l'orchestrateur à la 30-2c)

- **Contexte** : écart relevé par la 30-2b (« la fiche n'affiche que `creditorIban` ») ; l'orchestrateur l'a versé dans
  la 30-2c, avec AC25 (déjà à la fiche, C-30-2-23).
- **Retenu** : une ligne « QR-IBAN » sous l'IBAN, quand `creditorQrIban` existe (libellé littéral, comme « IBAN ») ;
  Vitest. AC25 : deux clés (`-paid-info`, `-paid-info-on`), `formatSwissDate` sur les dix premiers caractères.
- **Écartées** : une clé i18n pour « QR-IBAN » (terme identique dans les quatre langues, comme « IBAN » à côté).
- **Réversible** : oui.

### C-30-2-48 — Le sélecteur « Joindre un document » vit dans `imported-supplier-invoices/`, la date d'import est partagée

- **Contexte** : le sélecteur (AC20) affiche ce qu'affiche la liste « Documents à traiter » (badge, date, aperçu) ; la
  garde `lint-i18n-ownership` interdit à un composant de `supplier-invoices/` d'employer les clés
  `imported-supplier-invoices-*`.
- **Retenu** : `AttachDocumentPicker.svelte` dans `lib/features/imported-supplier-invoices/`, qui réemploie les clés
  du badge et de l'aperçu ; six clés neuves `imported-supplier-invoices-attach-*`. La date d'import passe dans
  `importDateOf` (`imported-supplier-invoices.types.ts`), employée par la liste et le sélecteur (DRY). Le montant du QR
  et le total de la facture se formatent par `formatSupplierInvoiceTotal`, le même formateur des deux côtés.
- **Écartées** : le composant dans `supplier-invoices/` avec des clés dupliquées (badge et aperçu deux fois) ; une
  exemption au linter.
- **Réversible** : oui.

### C-30-2-49 — « Voir le document » ouvre l'aperçu sous la fiche ; « Télécharger » reste ; `undefined` n'affiche rien

- **Contexte** : AC18 — l'aperçu dans un panneau, et plus de bouton qui rend 404.
- **Retenu** : sur une facture **avec** document : son nom, *Voir le document* (aperçu `DocumentPreview` sous les
  boutons, qui dit lui-même 410 et erreur), *Télécharger* (patron de C-30-2-24 : une image ne s'enregistre pas depuis
  l'aperçu), *Détacher le document* (Comptable+). **Sans** document (`null`) : *Joindre un document* (Comptable+). Champ
  **absent** (`undefined`, non calculé) : rien — non calculé n'est pas « sans ». Le téléchargement garde son traitement
  du 404 (document détaché ailleurs entre-temps). Sur la fiche d'une écriture, *Voir le document* ouvre le même aperçu,
  refermé quand on passe à une autre écriture.
- **Écartées** : un panneau modal (masque la fiche) ; retirer *Télécharger*.
- **Réversible** : oui.

### C-30-2-50 — Le dépôt rend un refus métier (`AttachRefusal`), la route le traduit ; une facture inconnue rend `404 NOT_FOUND`

- **Contexte** : AC19 nomme les refus `404` (document, facture), `409 IMPORT_NOT_PENDING_COMPLETION`,
  `409 SOURCE_DOCUMENT_ALREADY_ATTACHED`, sans code pour la facture.
- **Retenu** : `attach_in_tx` rend `Result<Result<ImportedSupplierInvoice, AttachRefusal>, DbError>` — refus métier
  distinct d'une erreur de base, testé au dépôt sans HTTP. Facture inconnue ou d'une autre société :
  `404 NOT_FOUND` (code des autres routes de la facture). La violation d'unicité n'est traduite que si son message
  nomme `uq_imported_supplier_invoice` ; toute autre erreur de base reste une erreur (mutation M-c1).
- **Écartées** : `DbError` pour tout (le refus métier se confondrait avec une panne) ; `404 SUPPLIER_INVOICE_NOT_FOUND`,
  code neuf sans précédent.
- **Réversible** : oui.

### C-30-2-51 — Le document d'une écriture se lit par une jointure ; un règlement annulé ne le montre plus

- **Contexte** : AC21 — achat **ou** règlement (C-30-2-15).
- **Retenu** : `find_source_document_of_entry`, une seule requête (`supplier_invoices` joint au document `completed`
  de la même société, `purchase_journal_entry_id = ? OR settlement_journal_entry_id = ?`), exécuteur générique, lue
  dans la transaction de lecture de `get_journal_entry` et inscrite à sa garde lexicale, paramétrée par couples
  `(module, lecteur)`. Un règlement annulé vide `settlement_journal_entry_id` : son écriture ne montre plus le
  document (écrit au manuel et à `api-external.md`).
- **Écartées** : deux lectures (facture, puis document) — deux appels à inscrire à la garde pour le même fait.
- **Réversible** : oui.
- **Annotation du 2026-10-20 (revue de code P1, A7)** : l'annulation d'une facture **payée** vide aussi les
  colonnes de règlement (`supplier_invoices.rs`, « règlement DÉTACHÉ ») — son écriture de règlement ne montre plus
  le document, l'écriture d'achat le montre toujours. Écrit au manuel utilisateur, à `api-external.md` et au
  doc-comment ; test `cancelled_invoices_take_a_document_and_the_detached_settlement_loses_it`.

### C-30-2-52 — Rattachement et détachement : relecture après COMMIT et non-idempotence, écrites en angles morts (revue P1, A8, B4)

- **Contexte** : les deux routes relisent la facture après le `COMMIT` (patron de `pay` et de `complete_import`,
  C-30-2-42) ; une erreur de base entre les deux rend une erreur sur un geste réussi. Et le rattachement n'est pas
  idempotent : renvoyé après un succès (réponse perdue), il rend `409 IMPORT_NOT_PENDING_COMPLETION`, dont le
  message (« n'est plus en attente de complétion ») ne dit pas que l'état voulu est atteint ; le détachement
  renvoyé rend `404 SOURCE_DOCUMENT_NOT_FOUND`.
- **Retenu** : **angles morts écrits** — doc-comments des deux routes, `api-external.md` (« relisez la facture
  avant de conclure à un échec »). Pas d'idempotence.
- **Écartées** : rendre le rattachement idempotent (`200` si le document est déjà lié à **cette** facture) — il
  faudrait relire le lien existant avant l'`UPDATE`, ce que C-30-2-21 écarte (lecture verrouillante : verrou
  d'intervalle, c'est la forme que M-c11 réintroduit ; lecture ordinaire : un second chemin pour le même refus) ;
  un message dédié pour ce cas — même lecture préalable. Repli sur
  `from_parts` en cas d'échec de relecture : écarté pour les quatre routes du patron ensemble (C-30-2-42).
- **Réversible** : oui.

### C-30-2-53 — Après un refus 404/409, l'écran relit ce qu'il montre ; gardes explicites du double clic (revue P1, B1 = E1 = A2, B2 = E5)

- **Contexte** : le sélecteur ne relisait pas sa liste après un refus (la ligne refusée restait proposée, chaque
  essai redonnait le même refus) ; la fiche ne relisait pas la facture après un détachement refusé en `404`
  (« Détacher » restait proposé, « Joindre » absent) ni après un `409 SOURCE_DOCUMENT_ALREADY_ATTACHED` ; le refus
  du détachement n'était exercé par aucun test.
- **Retenu** : `isStaleDocumentRefusal` (`imported-supplier-invoices/stale-refusal.ts`, un seul lieu) reconnaît un
  `404`/`409` ; sur un tel refus, le sélecteur relit sa liste, la fiche relit la facture (un échec de cette relecture
  laisse l'écran tel quel). Si la facture relue a un document, le sélecteur se ferme sur lui et le refus se dit par
  une notification (le message du sélecteur disparaîtrait avec lui). Le message du détachement est rendu **hors**
  du bloc du document, pour rester lisible quand la facture relue n'en a plus. Un autre refus (`5xx`, réseau, `403`)
  ne relit rien. Gardes `if (detaching) return;` et `if (attaching) return;` : sous jsdom, un second clic est parti
  malgré `disabled` (test rouge constaté avant la garde). Sept Vitest ; mutations M-c19 à M-c25 tuées.
- **Écartées** : écrire seulement l'angle mort au manuel (« fermez et rouvrez ») — la relecture est locale et
  testable ; relire sur tout refus — un `500` ne dit rien des données.
- **Réversible** : oui.

### C-30-2-54 — AC20 : l'aperçu du sélecteur s'ouvre au bouton, non au survol (écart déclaré, revue P1, A4)

- **Contexte** : AC20 demande un « aperçu du document survolé ou choisi » ; le sélecteur l'ouvre par le bouton
  *Aperçu* seulement.
- **Retenu** : **écart déclaré**, non comblé. Un aperçu au survol chargerait un document (image ou PDF) à chaque
  passage de souris sur une ligne, déplacerait la liste sous le pointeur (le panneau s'ouvre sous la ligne) et
  n'existe pas au clavier ni au toucher ; le bouton porte `aria-expanded` et sert les trois.
- **Écartées** : `onmouseenter` ouvrant le panneau (coût et instabilité ci-dessus) ; une vignette au survol — un
  composant de plus, hors du « si trivial » de la décision de l'orchestrateur.
- **Réversible** : oui (un gestionnaire de survol sur la ligne).

> **Annotation du 2026-10-20 (revue P2 ciblée, T1 et T2 — Sonnet 5.5)** — C-30-2-53 est complété, non réécrit.
> **T1 (MEDIUM, corrigé)** : le message de détachement ayant été sorti du bloc du document, le chemin « refus
> `404`/`409` du rattachement, facture relue avec un document » (qui sort par `return` avant la remise à zéro du
> chemin de succès) laissait un message périmé (« Cette facture n'a pas de document. ») sous le document joint
> ailleurs. Retenu : `detachError = ''` dans cette branche (`+page.svelte`, `attach`), au plus près du `return` ;
> l'autre variable d'erreur soumise à la relecture, `attachError`, vit dans le sélecteur, démonté avec lui, et
> `detach()` remet `detachError` à vide en entrée. Écartée : la remise à zéro dans `reloadAfterRefusal` — elle
> effacerait le message du `404` du détachement qu'on veut précisément garder. Test Vitest rouge constaté d'abord ;
> mutation M-c27 (retrait de la ligne) tuée. **T2 (LOW, écart assumé)** : une relecture en échec après un
> détachement `404` laisse l'ancienne facture et son message sous le document ; le comportement est celui d'avant la
> P1, la relecture échouée signifie que le réseau ou le serveur est en défaut, et le message reste lisible et vrai
> de l'instant du refus. Corriger obligerait à inventer un état « facture périmée » à l'écran pour un cas sans effet
> sur les comptes : non fait.
>
> **Annotation du 2026-10-20 (revue P3 ciblée, Opus, lentille T — orchestrateur)** — C-30-2-53 est complété, non
> réécrit. (1) L'en-tête de l'annotation précédente nomme **Sonnet 5.5 pour la remédiation** ; la revue P2 elle-même
> a été conduite par **Haiku**. (2) L'argument qui y écarte la remise à zéro dans `reloadAfterRefusal` ne vaut que
> pour une remise à zéro **sans condition** ; une variante conditionnée à la présence d'un document dans la facture
> relue n'aurait pas eu ce défaut — le correctif retenu (dans la branche de `attach`) reste juste, mais le motif
> d'exclusion était trop large. (3) **Écart assumé (P3, T1)** : le chemin jumeau existe dans `detach()` — un
> détachement refusé en `404`, puis un autre onglet qui joint un document juste avant la relecture, laisse « Cette
> facture n'a pas de document. » sous ce document. Fenêtre de course très courte, comportement antérieur à la P1,
> sans effet sur les comptes, disparu au rechargement : non corrigé.

---

## Versé par union : `epic-30-choix-autonomes-303.md`

*(Versé le 2026-10-24 par la Story 32-0 — action A2 de la rétrospective de l'Epic 31. Contenu recopié sans retouche depuis la deuxième ligne du registre de la story, qui reste en place et auquel renvoient la fiche et `sprint-status.yaml`.)*

Fiche : `30-3-chemin-facturation.md`. Issue [#585]. Choix tranchés au développement, le 2026-10-16, en autonomie.

[#585]: https://github.com/guycorbaz/kesh/issues/585

---

### C-30-3-1 — Corriger le texte, ne pas ajouter de lien

- **Contexte** : l'issue offre deux remèdes — écrire le vrai chemin, ou ajouter sur `/settings` un lien vers la facturation.
- **Retenu** : écrire le vrai chemin (« Administration → Facturation »). Le lien est un changement de navigation, hors
  d'une correction de texte.
- **Écartées** : le lien sur `/settings` (à décider à part, avec le titre d'écran de C-30-3-2).
- *Annotation du 2026-10-16 (revue P1, F4)* : la rédaction d'origine ajoutait que les entrées Exercices, Taux de TVA,
  Utilisateurs… « ont le même défaut ». C'était faux : `/settings` porte une section pour chacune ; seule
  « Facturation » n'en a pas. Retiré.
- **Réversible** : oui.

### C-30-3-2 — Le titre de l'écran reste « Paramètres — Facturation »

- **Contexte** : `settings-invoicing-title` est le titre affiché par l'écran, avec un tiret cadratin : ce n'est pas un chemin.
- **Retenu** : ne pas le toucher (changement d'écran, hors correction de texte de chemin).
- **Écartées** : le renommer « Facturation » (cohérent avec la barre latérale, mais touche les quatre catalogues, des
  tests et des captures ; à prendre avec le lien éventuel de C-30-3-1).
- **Réversible** : oui.

### C-30-3-3 — La garde G3 suit la barre latérale

- **Contexte** : `le_message_du_compte_de_produit_renvoie_a_l_ecran_reel` (`kesh-i18n/src/loader.rs`) dérivait le chemin
  attendu du titre d'écran (` — ` → ` → `), ce qui entérinait « Paramètres → Facturation ».
- **Retenu** : la garde construit le chemin depuis `nav-administration` et `nav-invoicing` de chaque locale ; une
  clé de navigation renommée fait rougir le message.
- **Écartées** : un chemin écrit en dur par locale (ne suit pas le renommage d'une entrée de menu).
- **Réversible** : oui.
