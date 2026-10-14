# Epic 29 — registre des choix autonomes, Story 29-6

*Registre propre à la story 29-6 (#665), à verser par **union** dans `epic-29-choix-autonomes.md` par l'orchestrateur.
Fiche : `29-6-trop-percu-centimes.md`.*

---

## Story 29-6 — La réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser ([#665])

### C-29-6-1 — Prémisses exactes ; une précision et un doc-comment faux

- **Contexte** : #665 (2026-10-14) décrit la tolérance de ± 5 centimes à la proposition et le refus du trop-perçu à
  l'acceptation.
- **Retenu** : établi au code sur `fcd2ae75`, les quatre prémisses sont exactes. Précision écrite dans la fiche : la
  tolérance est symétrique, et seul son côté « le paiement dépasse le reste » est refusé ; l'autre côté donne un
  règlement partiel accepté. Le doc-comment de `find_unpaid_invoices_for_window` (« la tolérance couvre l'écart à son
  arrondi au centime, au plus 0.005 ») est faux depuis l'origine (0,05 contre 0,005) : réécrit par l'AC2.
- **Réversible** : sans objet (constat).

### C-29-6-2 — Voie 1 : la proposition écarte le trop-perçu ; voie 2 écartée

- **Contexte** : #665 offre deux voies — la proposition écarte les trop-perçus, ou l'acceptation accepte l'écart toléré
  sur le compte d'arrondi (comme `SettlesWithRounding`). Consigne : établir au code les conséquences comptables, choisir
  le périmètre minimal sûr ; une écriture de produit d'arrondi sans compte d'arrondi configuré est exclue.
- **Retenu** : voie 1. Rien de nouveau ne s'écrit ; la proposition ne charge que ce que `classify_payment` admet.
- **Écartée** : voie 2, pour quatre raisons établies au code : (a) sans compte d'arrondi, `rounding_account_for_write`
  refuse encore (`ROUNDING_ACCOUNT_NOT_CONFIGURED`), donc le défaut subsiste sauf à faire lire les réglages par la
  proposition ; (b) le compte d'arrondi reçoit aujourd'hui des écarts d'un demi-centime au plus — un trop-perçu de
  5 centimes est une dette envers le client, pas une différence d'arrondi, et le passer en produit est une décision
  comptable qui revient à Guy ; (c) `classify_payment` est partagée avec le règlement manuel : l'élargir change la
  fiche de la facture, ne pas l'élargir refait deux règles du trop-perçu (#476 les avait réunies) ; (d) elle contredit
  « le trop-perçu est refusé, jamais écrit » (code, `api-external.md`) et « pas de tolérance de montant » (manuel,
  lettrage).
- **Réversible** : oui — la voie 2 peut venir plus tard comme évolution, sur demande.

### C-29-6-3 — La borne basse : une fonction Rust réciproque de `classify_payment`, liée au SQL, avant le `LIMIT 50`

- **Contexte** : `classify_payment(P, R)` rend `Overpayment` ⇔ `R < P − 0,005` (pour `P > 0` au centime, arrondi
  `MidpointAwayFromZero`). Le dépôt interdit un `ROUND` en SQL (une seule définition de l'arrondi).
- **Retenu** : `invoice_settlements::least_due_without_overpayment(paid) = paid − HALF_CENTIME`, à côté de
  `classify_payment`, liée en paramètre du `HAVING` ; un test unitaire exhaustif (grille au dix-millième) tient
  l'équivalence entre les deux lecteurs.
- **Écartées** : (a) filtrer en Rust après la requête par `classify_payment` — des trop-perçus occuperaient des places
  du `LIMIT 50` et feraient disparaître une facture valable au-delà ; (b) `ROUND(amount_due, 2) = ?` en SQL — seconde
  définition de l'arrondi ; (c) une borne basse égale au montant exact — écarterait le reste à quatre décimales soldé
  au centime (10.01 sur 10.0050, `SettlesWithRounding`), que l'acceptation admet.
- **Réversible** : oui (une fonction, un `HAVING`).

### C-29-6-4 — Le côté du paiement partiel ne change pas

- **Contexte** : un reste supérieur au paiement de 1 à 5 centimes est accepté en règlement partiel (`Ordinary`) ; la
  facture reste ouverte pour la différence.
- **Retenu** : inchangé — ce n'est ni un refus ni une écriture fausse, donc pas le défaut de #665. La tolérance de
  5 centimes ne borne plus que ce côté (constante `AMOUNT_TOLERANCE_HUNDREDTHS`, nom conservé, doc réécrit).
- **Écartées** : retirer la tolérance (les clients qui paient le montant QR arrondi d'une facture partiellement réglée
  ne seraient plus reconnus ; hors périmètre) ; l'élargir.
- **Réversible** : oui.

### C-29-6-5 — Signature : paramètre renommé, position conservée

- **Contexte** : `find_unpaid_invoices_for_window(…, tx_amount, amount_tolerance)` a treize sites d'appel : un en production (`routes/reconciliation.rs:656`), douze en test (`reconciliation_repository.rs` dix, `invoice_write_off.rs` deux).
- **Retenu** : renommer le paramètre `underpayment_tolerance` (le nom dit ce qu'il borne désormais), sans changer sa
  position ni son type : aucun site d'appel ne change.
- **Écartée** : une structure `InvoiceAmountWindow` sur le patron d'`InvoiceDateWindow` — plus de surface pour la même
  règle, et plus de conflits de rebase avec la 29-2, qui modifie la même requête et `InvoiceDateWindow`.
- **Réversible** : oui.

### C-29-6-6 — Aucun texte d'écran, aucune clé i18n, aucune capture

- **Contexte** : le libellé de `RECONCILIATION_OVERPAYMENT` (« Le montant de la transaction dépasse le reste dû de la
  facture. ») reste juste pour un appel direct ou une course.
- **Retenu** : le frontend ne change pas ; la frontière HTTP est exercée par le test `spawn_app` d'AC6 ; l'E2E complet
  tourne au dernier commit de code.
- **Réversible** : sans objet.

### C-29-6-7 — Le contournement écrit au manuel : régler le reste dû, puis rejeter la transaction

- **Contexte** : un client qui a payé quelques centimes de trop n'est plus proposé ; le règlement manuel refuse aussi
  le trop-perçu.
- **Retenu** : le manuel dit d'enregistrer le règlement du reste dû depuis la fiche, de **rejeter** la transaction à la
  réconciliation, et de traiter l'écart (remboursement ou écriture manuelle) — même forme que le contournement de la
  29-2 (C-29-2-8).
- **Réversible** : oui (texte).

### C-29-6-8 — Les angles morts #665 écrits par la 29-3 se retirent au rebase

- **Contexte** : la fiche 29-3 (C-29-3-7) prescrit d'écrire au manuel (`:2049`) et au doc-comment de `candidate_rank`
  qu'une facture que le paiement dépasse est proposée puis refusée (#665). La 29-6 se développe après elle.
- **Retenu** : la 29-6 retire ou réécrit ces phrases au rebase (AC7, § *Zones partagées*), inventaire par
  `grep -n 665`.
- **Écartée** : demander à la 29-3 de ne pas les écrire — elle se merge avant, et ses phrases sont vraies à sa date.
- **Réversible** : sans objet.

### C-29-6-9 — Story ajoutée à l'Epic 29, sortie dans la v0.13.4

- **Contexte** : décision de l'orchestrateur du 2026-10-14 ; #665 relevée pendant la fiche de la 29-3.
- **Retenu** : clé `29-6-trop-percu-centimes` sous `epic-29` ; paragraphe « 29-6 » au fichier d'epic (§ *Les stories*,
  § *Ordre*, § *Releases*) ; ordre de développement 29-2 → 29-3 → 29-5 → 29-6.
- **Réversible** : oui.

[#665]: https://github.com/guycorbaz/kesh/issues/665
