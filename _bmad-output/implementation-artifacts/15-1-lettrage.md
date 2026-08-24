# Story 15.1 : Lettrage — savoir ce qui reste ouvert

## Status

draft

## Story

**As a** indépendant ou fiduciaire qui tient ses comptes dans Kesh,
**I want** rapprocher une créance de son règlement et voir d'un coup d'œil ce qui reste ouvert sur un compte client ou fournisseur,
**so that** je puisse relancer les bons débiteurs, justifier le solde d'un compte, et clore un exercice en sachant ce qu'il porte.

Couvre **FR85** (lettrer) et **FR86** (délettrer). Première story de l'**Epic 15**, passée
en tête au kickoff sur l'arbitrage de Guy : *« le lettrage est indispensable à la mise en
production »*.

## Contexte — ce qui existe, et ce qui manque vraiment

⚠️ **Trois rapprochements sont déjà en place, et le relevé au sol a changé la question que
cette story doit résoudre.**

| dispositif | ce qu'il lie | où |
|---|---|---|
| Facture ↔ écriture de vente | la pièce et son écriture | `invoices.journal_entry_id` |
| Transaction bancaire ↔ écriture | l'encaissement importé et son écriture | `bank_transactions.matched_entry_id`, `.status` |
| Marqueur de paiement | une facture réputée payée | `invoices.paid_at` |

Et surtout : **`accept_one_invoice` (`reconciliation.rs:1096-1206`) apparie déjà transaction
bancaire ↔ facture, propose, fait valider, puis marque la facture payée.** Le chemin
« facture réglée par virement importé » est donc **entièrement couvert** — et par le geste
même qu'on attendait du lettrage.

**Ce qui reste sans réponse aujourd'hui**, et qui est l'objet de cette story :

- un règlement **hors import bancaire** — espèces, compensation, virement non importé —
  ne rapproche rien : la facture reste `paid_at IS NULL` sauf saisie manuelle ;
- les comptes **fournisseurs** n'ont aucun équivalent de ce mécanisme ;
- deux **écritures manuelles** qui se soldent (acompte puis facture) ne se relient pas ;
- et **aucun écran ne répond à « qu'est-ce qui reste ouvert sur ce compte ? »**.

`lettering_code` est **absent du dépôt** — 0 occurrence sur `crates/`. Tout est à
construire, rien à défaire.

## Décisions

### D1 — Le lettrage porte sur la LIGNE, et l'argument n'est pas le confort futur

`epics.md` prescrit `lettering_code` sur `journal_entry_lines`. La question s'est posée de
le porter sur l'écriture, puisque D2 borne à un appariement 1↔1.

⚠️ **C'est la ligne, et pour une raison de fond : le lettrage porte sur un COMPTE.** Une
écriture de vente touche le compte client, un compte de produit et un compte de TVA. Dire
« cette écriture est lettrée » n'a aucun sens comptable — ce qui est soldé, c'est la
**ligne au compte client**, pas l'écriture entière. Un code posé sur l'écriture rendrait
d'ailleurs la vue « ce qui reste ouvert **sur le compte 1100** » impossible à calculer sans
retrouver la ligne concernée.

Le fait que ce niveau serve aussi le jour où l'on rouvrira le partiel ou le groupé est un
bénéfice, **pas la justification**.

### D2 — Un lettrage, une facture, un règlement

Arbitrage de Guy. Ni paiement **partiel** (une facture réglée en plusieurs fois), ni
règlement **groupé** (un virement pour plusieurs factures).

⚠️ **Ce que cette borne coûte, et il faut que l'utilisateur le sache** : les factures
relevant de ces deux cas resteront affichées **ouvertes**, indéfiniment. Rien ne se perd en
silence — elles sont visibles, c'est même tout le propos de l'écran — mais un utilisateur
qui ne comprend pas *pourquoi* une facture manifestement payée reste ouverte perdra
confiance dans la vue. **Le manuel doit le dire, et l'écran devrait l'expliquer là où le cas
se présente.**

Le règlement groupé est **courant** chez un client qui reçoit plusieurs factures par mois :
c'est le premier candidat à la réouverture du périmètre.

### D3 — Lettrer sur un exercice clôturé : OUI. Délettrer : NON

Le lettrage **ne modifie aucun montant** : il note qu'une créance est soldée. Il n'entre
donc pas en conflit avec l'immuabilité post-clôture, et c'est **ce qui rend le lettrage à
cheval réellement possible** — une facture de décembre payée en février se lettre après la
clôture de l'exercice.

Le délettrage reste refusé sur exercice clôturé (`epics.md`).

⚠️ **La justification par l'ordre des opérations ne tient pas, et il faut l'écrire pour que
personne ne la ressorte** : « ne clôturer que lorsqu'il n'y a plus d'écritures à passer »
ne suffit pas, parce qu'**une créance client ouverte au 31.12 est normale** — elle figure
au bilan. Attendre que tout soit réglé reviendrait à ne jamais clôturer. C'est D3, et elle
seule, qui rend le cas à cheval praticable.

### D4 — « Ouvert » signifie *ni lettré ni marqué payé*

⚠️ **Le piège central de cette story, et il ne se voit qu'en connaissant D1 du plan
d'epic.** Une facture réglée par la réconciliation porte `paid_at` **sans** être lettrée.
Un écran qui ne regarderait que `lettering_code` l'afficherait **ouverte** — il mentirait
précisément là où il doit informer, et sur le chemin le plus fréquent.

La définition est donc : **une ligne est ouverte si elle n'est ni lettrée, ni portée par une
facture marquée payée**. C'est un critère d'acceptation, pas un détail d'implémentation.

### D5 — Kesh propose, l'utilisateur valide

Arbitrage de Guy, conforme à la règle du dépôt *« un appariement automatique propose, il ne
crée jamais »*. Aucun lettrage ne s'écrit sans validation humaine.

**Les critères de proposition se reprennent de la réconciliation plutôt que de s'inventer**
— ils y sont éprouvés depuis l'Epic 8 :

| critère | valeur chez la réconciliation |
|---|---|
| statut de la facture | `status = 'validated'` |
| non déjà réglée | `paid_at IS NULL` |
| fenêtre de dates | ±`WINDOW_DAYS` = **30 jours** |
| montant | égalité à `amount_tolerance` près |

⚠️ **Ne PAS créer un second jeu de critères.** Deux mécanismes de proposition qui divergent
donneraient des suggestions différentes sur le même cas, et l'utilisateur n'aurait aucun
moyen de savoir lequel croire.

### D6 — Écran dédié, et sa frontière avec la réconciliation doit être lisible

Arbitrage de Guy, modèle Bexio.

⚠️ **Risque réel** : cet écran ressemblera à celui de la réconciliation, qui propose déjà
des appariements. Deux écrans voisins qui font des choses différentes se confondent. La
distinction doit être énoncée **pour l'utilisateur**, pas seulement dans le code :

- **Réconciliation bancaire** : « j'ai importé mon relevé, à quoi correspondent ces
  mouvements ? » — part de la **banque** ;
- **Lettrage** : « qu'est-ce qui reste ouvert sur ce compte, et qu'est-ce qui le solde ? » —
  part du **compte**.

## Critères d'acceptation

**AC1** — Une colonne `lettering_code` (nullable) est ajoutée à `journal_entry_lines`. Deux
lignes portant le même code sont lettrées ensemble.

**AC2** — Depuis l'écran dédié, l'utilisateur choisit un compte et voit ses lignes
**ouvertes**, au sens de **D4** : ni lettrées, ni portées par une facture marquée payée.

**AC3** — ⚠️ Une facture réglée par la **réconciliation bancaire** n'apparaît **pas** comme
ouverte, bien qu'elle ne soit pas lettrée. *(Test explicite : c'est le piège de D4.)*

**AC4** — Kesh **propose** des rapprochements selon les critères de **D5**, repris de la
réconciliation. Aucun lettrage n'est écrit sans validation explicite (**D5**).

**AC5** — L'utilisateur peut lettrer deux lignes manuellement, sans proposition.

**AC6** — Le lettrage est **refusé** si les deux lignes ne portent pas sur le **même
compte**, ou si leurs sens (débit/crédit) ne s'opposent pas.

**AC7** — Le délettrage est possible tant que **les deux** exercices concernés sont ouverts,
et **refusé** dès que l'un est clôturé (**D3**).

**AC8** — Le lettrage est **autorisé** même si l'un des exercices est clôturé (**D3**), y
compris à cheval sur deux exercices.

**AC9** — Le code de lettrage est **visible** sur la ligne, dans le détail d'écriture et
dans l'écran dédié ; les lignes partageant un code sont identifiables entre elles.

**AC10** — ⚠️ L'écran indique **pourquoi** une facture manifestement réglée peut rester
ouverte : paiement partiel ou règlement groupé, hors périmètre (**D2**). Sans cela, la vue
paraît fausse là où elle est seulement bornée.

## Tasks

- [ ] **T1** — Migration : `lettering_code` sur `journal_entry_lines`, nullable, indexée.
      ⚠️ `ADD COLUMN` nullable = **non-breaking**, donc pas de bump `min_required` (P1) ;
      ligne d'audit d'idempotence **obligatoire** (P5) ; triage `POST_RESTORE_BACKFILLS`
      (P7) — cette migration n'écrit **aucune donnée**, donc exemption avec justification.
- [ ] **T2** — Repository : lettrer, délettrer, lister les lignes ouvertes d'un compte.
      ⚠️ La requête « ouvertes » implémente **D4** — la jointure sur `invoices.paid_at` en
      fait partie.
- [ ] **T3** — Moteur de proposition, **réutilisant les critères de la réconciliation**
      (D5). Ne pas dupliquer : extraire si nécessaire.
- [ ] **T4** — Routes : `POST` lettrage, `DELETE` délettrage, `GET` lignes ouvertes.
      Garde de clôture selon **D3** — asymétrique, et c'est délibéré.
- [ ] **T5** — Écran dédié (**D6**), avec la frontière énoncée pour l'utilisateur.
- [ ] **T6** — Tests : AC3 (le piège de D4) et AC7/AC8 (l'asymétrie de clôture) **en
      priorité** — ce sont les deux endroits où une implémentation plausible se trompe.
- [ ] **T7** — i18n : toutes les clés dans les **quatre** locales dès l'écriture. Les six
      gardes de l'Epic 23 le vérifient, et l'allowlist est **vide** — une clé manquante
      rougit au gate.
- [ ] **T8** — Manuel utilisateur : ce que le lettrage fait, et **ce qu'il ne fait pas
      encore** (D2).

## Dev Notes

⚠️ **Le sélecteur E2E ne se fige jamais sur un libellé** — `data-testid` sans exception. La
garde livrée par #326 le vérifie, et son allowlist ne doit pas s'allonger.

⚠️ **Gate `kesh-db` : complet, jamais ciblé.** Cette story touche une migration et un
repository — les garde-fous P6 et P7 imposent le gate entier, et le précédent de la
Story 16-1a (un test devenu muet, passant à vide) dit pourquoi.

⚠️ **La base de gate se remet à zéro AVANT le gate**, sans se demander comment le run
précédent s'est terminé (KF-039, #310).
