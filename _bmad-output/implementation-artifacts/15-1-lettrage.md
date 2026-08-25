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

**AC1** (porte **D1**) — Une colonne `lettering_code` (nullable) est ajoutée à
`journal_entry_lines` — **la ligne, non l'écriture**. Deux lignes portant le même code sont
lettrées ensemble.

**AC2** — Depuis l'écran dédié, l'utilisateur choisit un compte et voit ses lignes
**ouvertes**, au sens de **D4** : ni lettrées, ni portées par une facture marquée payée.

**AC3** — ⚠️ Une facture réglée par la **réconciliation bancaire** n'apparaît **pas** comme
ouverte, bien qu'elle ne soit pas lettrée. *(Test explicite : c'est le piège de D4.)*

**AC3-bis** — ⚠️ **La règle vaut pour les DEUX tables de factures.** *(Relevé en
relecture.)* `invoices` et `supplier_invoices` sont **deux tables distinctes**, chacune avec
son propre `paid_at` — vérifié au sol. Or **D1 met explicitement les comptes fournisseurs
dans le périmètre** : une définition de « ouvert » qui ne lirait que `invoices.paid_at`
laisserait **la moitié du périmètre annoncé** afficher comme ouvertes des factures
fournisseurs déjà réglées. Un test par table.

**AC4** — Kesh **propose** des rapprochements selon les critères de **D5**, repris de la
réconciliation. Aucun lettrage n'est écrit sans validation explicite (**D5**).

**AC5** — L'utilisateur peut lettrer deux lignes manuellement, sans proposition.

**AC6** — Le lettrage est **refusé** si les deux lignes ne portent pas sur le **même
compte**, ou si leurs sens (débit/crédit) ne s'opposent pas.

**AC6-bis** — ⚠️ Le lettrage est **refusé si les deux montants ne sont pas égaux**.
*(Relevé en relecture — la spec ne l'imposait nulle part.)* **D2 exclut le paiement
partiel** : lettrer une créance de 1000 avec un règlement de 300 prétendrait qu'elle est
soldée, et **ferait mentir la vue qui est tout l'objet de cette story**. Tant que le partiel
est hors périmètre, l'égalité est la condition qui rend le lettrage véridique — ce n'est pas
une restriction technique mais la garantie du sens.

⚠️ **Le message de refus doit nommer la cause** — *« les montants diffèrent ; le lettrage
partiel n'est pas encore géré »* — sinon l'utilisateur conclut à un défaut.

**AC7** — Le délettrage est possible tant que **les deux** exercices concernés sont ouverts,
et **refusé** dès que l'un est clôturé (**D3**).

**AC8** — Le lettrage est **autorisé** même si l'un des exercices est clôturé (**D3**), y
compris à cheval sur deux exercices.

**AC9** — Le code de lettrage est **visible** sur la ligne, dans le détail d'écriture et
dans l'écran dédié ; les lignes partageant un code sont identifiables entre elles.

**AC10** — ⚠️ L'écran indique **pourquoi** une facture manifestement réglée peut rester
ouverte : paiement partiel ou règlement groupé, hors périmètre (**D2**). Sans cela, la vue
paraît fausse là où elle est seulement bornée.

**AC11** — ⚠️ **Le code de lettrage est engendré par le serveur, et sa portée est la
société.** *(Relevé en relecture : la spec disait « deux lignes portant le même code sont
lettrées ensemble » sans dire d'où vient le code.)* Deux sociétés doivent pouvoir porter le
même code sans se voir ni se percuter. **Un compteur non scopé serait un défaut de
multi-tenant** — le dépôt en a déjà payé un (KF-002), et c'est exactement la classe d'erreur
qu'une spécification muette invite à commettre. Le format est libre ; sa **portée** ne l'est
pas.

**AC12** — ⚠️ **Une écriture dont une ligne est lettrée ne se supprime pas en laissant un
code orphelin.** *(Relevé en relecture : `delete_journal_entry` existe —
`journal_entries.rs:631`.)* Sans garde, supprimer l'écriture d'encaissement laisserait la
ligne de créance **lettrée donc réputée soldée**, alors que sa contrepartie n'existe plus :
la facture disparaîtrait de la vue des ouverts **en restant impayée**. C'est un défaut muet,
et du genre le plus coûteux — il fausse le solde sans rien signaler.

Deux conduites possibles, à trancher à l'implémentation : **refuser** la suppression d'une
écriture lettrée (cohérent avec le refus de délettrage après clôture), ou **délettrer
automatiquement** la contrepartie. ⚠️ La seconde ne vaut que si l'exercice est ouvert —
sinon elle contournerait AC7 par un chemin détourné.

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
- [ ] **T6** — Tests : **AC3 et AC3-bis** (le piège de D4, sur les DEUX tables),
      **AC6-bis** (égalité des montants) et **AC7/AC8** (l'asymétrie de clôture) **en
      priorité** — ce sont les endroits où une implémentation plausible se trompe.
- [ ] **T9** — Portée du code de lettrage (**AC11**) et garde sur la suppression d'une
      écriture lettrée (**AC12**). ⚠️ Les deux sont nés d'une relecture, pas de la spec
      initiale : ils manquaient tous les deux.
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


## Change Log

### Relecture critique — 2026-08-25

⚠️ **Ce n'est PAS une passe de `bmad-create-story validate` au sens de la § *Review
Iteration Rule***, et le dire importe : elle a été menée **par l'auteur de la spec**, sans
contexte frais ni modèle distinct. Elle n'offre donc **aucune protection contre le biais
d'auteur** — c'est précisément ce que la rotation des modèles existe pour couvrir. **Une
passe adversariale reste due avant tout développement.**

Elle a néanmoins trouvé **quatre défauts, tous vérifiés au sol** :

| | défaut | gravité |
|---|---|---|
| **F1** | Le **code de lettrage** n'était engendré par rien : ni format, ni portée. ⚠️ Un compteur non scopé serait un défaut de multi-tenant — le dépôt en a déjà payé un (KF-002) → **AC11** | MEDIUM |
| **F2** | **Rien n'imposait l'égalité des montants.** D2 exclut le partiel : lettrer 1000 avec 300 prétendrait qu'une créance est soldée et **ferait mentir la vue qui est tout l'objet de la story** → **AC6-bis** | **HIGH** |
| **F3** | D4 définissait « ouvert » via `invoices.paid_at` seul, alors que **D1 met les fournisseurs dans le périmètre** et que `supplier_invoices` est une **table distincte** avec son propre `paid_at` (vérifié au sol). **La moitié du périmètre annoncé** aurait affiché des factures réglées comme ouvertes → **AC3-bis** | **HIGH** |
| **F4** | `delete_journal_entry` **existe** (`journal_entries.rs:631`). Supprimer une écriture lettrée laissait sa contrepartie **réputée soldée** : la facture quittait la vue des ouverts **en restant impayée** — défaut muet qui fausse le solde sans rien signaler → **AC12** | MEDIUM |

**Et un défaut de traçabilité** : **D1 n'était cité par aucun critère**. AC1 le portait sans
le nommer — or *un développeur lit le critère et la case à cocher, pas la décision*. C'est
la cinquième récidive de ce geste dans le dépôt ; AC1 porte désormais la mention.

⚠️ **Ce que cette récolte dit de la spec initiale** : ses six décisions étaient justes, et
les deux pièges qu'elle nommait le sont toujours. Ce qui manquait n'était pas du
raisonnement mais des **conditions de véracité** — l'égalité des montants, la seconde table
de factures, la portée du code, le sort d'une contrepartie supprimée. **Trois des quatre
défauts font mentir la vue**, c'est-à-dire attaquent exactement ce que la story existe pour
produire.
