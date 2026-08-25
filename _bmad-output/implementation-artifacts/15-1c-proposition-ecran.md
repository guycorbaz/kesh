# Story 15.1c : Proposer un rapprochement, et l'écran qui le porte

## Status

draft

## Story

**As a** indépendant ou fiduciaire qui tient ses comptes dans Kesh,
**I want** que Kesh me propose les rapprochements évidents et me laisse lettrer à la main les
autres, depuis un écran dédié,
**so that** je solde mes comptes sans ressaisir ce que le logiciel voit déjà.

Troisième des trois sous-stories issues du **split de la 15-1**. ⚠️ **Suppose 15-1a livrée**
(elle pose la marque) **et 15-1b spécifiée** (la vue et sa définition d'« ouvert »).

## Décisions héritées, et la correction majeure de la passe 1

### D5 — Kesh propose, l'utilisateur valide

Conforme à la règle du dépôt *« un appariement automatique propose, il ne crée jamais »*.
Aucun lettrage ne s'écrit sans validation humaine.

**Les critères se reprennent de la réconciliation plutôt que de s'inventer** — ils y sont
éprouvés depuis l'Epic 8. ⚠️ **Mais ils ne se transposent PAS tous, et les reprendre en bloc
casserait trois des quatre cas que le lettrage existe pour couvrir** *(relevé en passe 1,
vérifié au sol)*.

**Les critères STRUCTURELS d'éligibilité — dont TROIS sont NEUFS** *(P3-10 : le titre disait « ce qui se reprend », ce que sa propre colonne contredit)* :

| critère | valeur | provenance |
|---|---|---|
| même compte | identité stricte | ⚠️ **NEUF** |
| sens opposés | débit ↔ crédit | ⚠️ **NEUF** |
| **ni l'une ni l'autre déjà lettrée** | `lettering_id IS NULL` **des deux côtés** | ⚠️ **NEUF** |
| ~~fenêtre de dates~~ | ⛔ **retirée de l'éligibilité** — elle sert au **CLASSEMENT** (AC4-bis) | *(cf. Réserve 2)* |
| montant | **égalité STRICTE** — ⚠️ voir la Réserve 1 | adapté (la logique existe, le seuil change) |

⚠️ **La colonne « provenance » n'est pas décorative : trois critères sur cinq sont NEUFS, et
la rédaction précédente laissait croire le contraire** *(relevé en passe 1)*. Le moteur de
l'Epic 8 apparie une **transaction bancaire** à une **facture** — la première n'a pas de
compte du plan comptable, la seconde n'a pas de sens débit/crédit. Son scoring est
`0,50 × montant + 0,40 × référence + 0,10 × contact` (`kesh-reconciliation/src/matching.rs`) :
**aucun terme de compte ni de sens**. Et `rules.rs` (Story 8-5b) a **délibérément retiré** le
seul filtre de sens qui ait existé — *« Sign-agnostic : NE PAS hériter du sign filter 8-4 »*.

⛔ **Conséquence pour T1** : « ne pas dupliquer, extraire si nécessaire » ne vaut que pour la
fenêtre et le montant. **Il n'y a rien à extraire pour « même compte » et « sens opposés »** —
un développeur qui les chercherait perdrait son temps devant du code qui n'existe pas.

⛔ **Et le critère `lettering_id IS NULL` manquait** *(relevé en passe 1)*. AC10 de **15-1a**
refuse de lettrer une ligne déjà marquée — la route est donc protégée —, mais **rien
n'empêchait le MOTEUR de re-proposer** la paire. L'écran aurait présenté en boucle des
« rapprochements évidents » que la validation rejette, ruinant la promesse du *so that* :
*« sans ressaisir ce que le logiciel voit déjà »*.

**Ce qui NE se reprend PAS — les filtres propres à la facture client :**

⚠️ `status = 'validated'` et `paid_at IS NULL` interrogent la table `invoices`. Or **trois
des quatre cas visés n'ont aucune facture `validated` derrière la ligne à apparier** : les
comptes **fournisseurs** (`supplier_invoices`), deux **écritures manuelles** qui se soldent,
et une facture **annulée par un avoir** — qui passe à `status = 'cancelled'`
(`credit_notes.rs:563`, `supplier_invoices.rs:811`), alors que la paire
facture/contre-passation est le cas de lettrage **le plus propre qui soit**.

Et le lien manque : **`journal_entry_lines` ne porte ni `invoice_id` ni
`supplier_invoice_id`** — le seul lien va de `invoices.journal_entry_id` vers l'**entête**
d'écriture. Un moteur ligne-à-ligne ne peut pas s'appuyer sur le statut de facture sans une
jointure que le schéma ne permet pas.

**Ces filtres restent légitimes pour TRIER les suggestions du cas facture client impayée ;
ils ne doivent pas conditionner l'ÉLIGIBILITÉ d'une paire de lignes.**

### D6 — Écran dédié, et sa frontière avec la réconciliation doit être lisible

L'écran ressemblera à celui de la réconciliation. Leur frontière doit être lisible **pour
l'utilisateur**, pas seulement dans le code : deux écrans voisins qui font des choses
différentes se confondent.

## Deux réserves ouvertes, à arbitrer avant le développement

### ⛔ Réserve 1 — La tolérance de montant, et les frais bancaires

La réconciliation tolère **5 centimes** (`AMOUNT_TOLERANCE_HUNDREDTHS = 5`,
`kesh-api/src/routes/reconciliation.rs:60`), précisément pour absorber les frais bancaires.
Mais la borne « un lettrage = une facture = un règlement » impose l'**égalité stricte** :
lettrer 1000 avec 300 prétendrait qu'une créance est soldée.

Reprendre la tolérance produirait un système qui **propose ce qu'il refuse ensuite**. La spec
tranche donc pour l'**égalité stricte des deux côtés**.

⚠️ **Sa conséquence doit être vue avant le développement** : un règlement amputé de frais
bancaires ne sera **jamais lettrable**, et la facture restera affichée ouverte. Conduite
alternative, si l'arbitrage change : garder la tolérance au **classement** des suggestions,
**jamais** au filtre d'éligibilité.

### ⛔ Réserve 2 — La fenêtre de 30 jours tue le cas que AC2 exige nommément

`WINDOW_DAYS = 30` (`reconciliation.rs:55`) a un sens d'origine **borné** : le code note
lui-même que les *« paiements tardifs > 30 j sont reportés Story 8-5 manual »* (l. 1125).

⚠️ Transposée au lettrage, elle **écarte la contre-passation** — le cas que D5 appelle « le
plus propre qui soit ». L'écriture d'annulation fournisseur est **datée du jour**
(`supplier_invoices.rs:786-795`), pas de la facture. Une facture de mars annulée en novembre
forme une paire parfaite qui ne serait **jamais proposée**.

⚠️ **Et le test d'AC2 passerait quand même** — vert, si son auteur date les deux pièces à
moins de 30 jours. Un test qui ne dit rien du cas réel. Même effet sur un règlement client à
45 jours, c'est-à-dire sur le débiteur qu'on veut relancer.

**TRANCHÉ PAR DÉFAUT — la fenêtre s'applique au CLASSEMENT, pas au FILTRE.** *(Explicité en
passe 2 : « tranchée par défaut » ne disait pas laquelle des deux conduites était le défaut, et
**AC2 en dépend**.)*

⛔ **C'est la seule lecture cohérente avec AC2**, qui exige un test datant les deux pièces à
**plus de 30 jours d'écart**. Si la fenêtre filtrait, ce test **échouerait par construction** —
et deux développeurs auraient raison en même temps : celui qui écrit le moteur avec un filtre à
30 jours, et celui qui écrit le test tel qu'AC2 le prescrit. C'est exactement la contradiction
« proposer ce qu'on refuse » de la Réserve 1, transposée à la date.

**La conduite alternative**, si l'arbitrage change : une fenêtre **distincte et justifiée** —
l'exercice comptable est le candidat naturel —, appliquée cette fois en filtre. ⚠️ Elle
**oblige alors à réécrire AC2**, dont l'écart de dates devrait rentrer dans la nouvelle
fenêtre.

## Critères d'acceptation

**AC1** — L'utilisateur peut lettrer deux lignes **manuellement**, sans proposition.

**AC2** (porte **D5**) — Kesh **propose** des rapprochements selon les critères
**structurels** de D5. Aucun lettrage n'est écrit sans validation explicite.

⚠️ **La proposition ne filtre PAS sur le statut de la facture.** **Un test nommé par cas** :
une paire **facture/avoir** (`status = 'cancelled'`), une paire sur un **compte
fournisseur**, et une paire d'**écritures manuelles** sans facture doivent chacune être
proposées. ⚠️ **Le test de la contre-passation date les deux pièces à plus de 30 jours
d'écart** — sinon il passe sans rien prouver (cf. Réserve 2).

**AC3** — Le lettrage est **refusé si les deux montants ne sont pas égaux**, et **le message
de refus nomme la cause** — *« les montants diffèrent ; le lettrage partiel n'est pas encore
géré »* —, sinon l'utilisateur conclut à un défaut.

**AC4** — La marque est **visible** sur la ligne, dans le détail d'écriture et dans l'écran
dédié ; les lignes partageant une marque sont identifiables entre elles.

**AC4-bis** — ⛔ **L'ordre des propositions est DÉFINI, et le plafond s'y adosse.**
*(P3-3, passe 3 : « la fenêtre s'applique au classement » — mais aucun critère, aucune tâche,
aucun test ne disait ce qu'était ce classement.)* Les propositions sont rendues par
**proximité de date décroissante**, avec un départage stable.

⚠️ **Sans cet ordre, le plafond tronque un ensemble non ordonné — et retourne AC2 contre
elle-même** : la contre-passation, **datée du jour**, est celle que la proximité de date place
**en dernier**, donc la première évincée. Le test d'AC2 passerait quand même sur une fixture à
trois paires : c'est mot pour mot le « test qui ne dit rien du cas réel » que la Réserve 2
existe pour empêcher. **Le test vérifie que la paire hors fenêtre est présente ET rangée.**

**AC5** (porte **D6**) — ⚠️ **L'écran énonce sa frontière avec la réconciliation bancaire,
pour l'UTILISATEUR.** Deux exigences **distinctes** : **(1)** un texte **visible** — bandeau
ou aide contextuelle — dit ce que cet écran fait et ce qu'il ne fait pas ; **(2)** le test
E2E l'atteint par un `data-testid` stable et **jamais** par son libellé traduit. Un
`data-testid` ne satisfait pas (1) : il est invisible.

**AC6** — ⛔ **Le moteur ne voit QUE les lignes de la société de l'appelant, et c'est
vérifié par jointure.** *(Relevé en passe 1 : 15-1c n'avait **aucune** mention de scoping —
zéro occurrence de `company_id`, « multi-tenant » ou « IDOR ».)*

⚠️ **15-1c ouvre une surface d'API NEUVE** — le moteur et ses routes de proposition — par un
chemin **distinct** des routes de lettrage que 15-1a a pris soin de scoper (son AC11). La
discipline ne s'hérite pas : elle se réécrit ici. `journal_entry_lines` n'ayant **aucun**
`company_id`, la vérification passe par jointure sur `journal_entries.company_id`.

⚠️ **Si une route accepte un identifiant externe** — un compte, une ligne —, le refus est un
**404 indiscernable** de « inconnu », jamais un message qui nomme la cause : c'est la
convention anti-IDOR du dépôt (`kesh-api/src/routes/products.rs:343`), et **le dépôt a déjà
payé un défaut de cette classe (KF-002)**.

⛔ **RECTIFIÉ — l'inscription au Pattern 5 ne s'applique PAS ici, et l'affirmation qui la
motivait était fausse** *(P3-6, passe 3)*. `MULTI-TENANT-SCOPING-PATTERNS.md` n'exige rien de
« tout nouvel endpoint » : il impose un **ordre de verrous** à ceux qui en prennent **plus
d'un**, et une inscription **seulement en cas de divergence délibérée** (« deny list »). Or
**les routes de 15-1c sont des routes de LECTURE** — aucun `FOR UPDATE`, donc **aucune
séquence de verrous à inscrire**. L'exigence avait été recopiée de 15-1a T4, où elle est
légitime (le lettrage prend le sentinel `companies` **et** `fiscal_years FOR UPDATE`).

⚠️ Et le raffinement de la passe 2 se contredisait lui-même : le tableau « Where This Applies »
a trois colonnes — `Endpoint | Lock sequence | File` — et **pas de colonne de scoping**.

**Ce qui s'applique réellement à une route de lecture**, et qui est déjà écrit plus haut : la
**jointure de scoping** sur `journal_entries.company_id` et le **404 indiscernable**.

**AC7** — ⛔ **Le moteur est BORNÉ, en portée et en volume.** *(Relevé en passe 1 : T1 et T2
tenaient en une phrase, sans limite ni portée.)*

- **Portée** : le moteur travaille sur **un compte choisi**, jamais sur la société entière.
- ⛔ **Jeu candidat borné — c'est LA borne, et elle agit AVANT le calcul** *(P3-2, passe 3 :
  les deux bornes précédentes agissaient **après**, l'appariement ayant déjà eu lieu)* : le
  nombre de lignes ouvertes **chargées** pour un compte est plafonné. Au-delà, **refus
  explicite** — *« trop de lignes ouvertes sur ce compte, affinez »* — et **jamais** une
  troncature silencieuse.
- **Volume de sortie** : plafonné à **500** (`MAX_LIMIT`, patron du dépôt —
  `journal_entries.rs:541`, repris par `credit_notes`, `payment_batches`, `supplier_invoices`,
  `users`, et déjà cité par 15-1b T2), un `?limit=` reçu étant **écrêté**. *(Le chiffre
  manquait : « le plafond tient » n'est pas un test tant qu'aucune valeur n'est nommée.)*

⚠️ **Pourquoi la borne de sortie ne suffisait pas, et pourquoi l'analogie était fausse.** Le
`LIMIT 50` de la réconciliation borne le **jeu candidat**, dans un `WHERE` que la fenêtre de
dates **et** la tolérance de montant réduisent déjà. Ici, la Réserve 2 a retiré la fenêtre du
filtre et AC2 interdit `status`/`paid_at` : **il ne reste aucun prédicat réducteur**. Et la
réconciliation est un problème **1 → N** — une transaction, ses factures voisines — quand le
lettrage est **N → N** : toutes les lignes ouvertes d'un compte appariées entre elles. Le
premier se borne par un `LIMIT`, le second non.

✅ **Ce que la Réserve 1 offre gratuitement, et que la fiche ne relevait pas** : l'égalité
**stricte** des montants rend l'appariement **groupable par montant** — un regroupement en
mémoire, **linéaire**, plutôt qu'une auto-jointure quadratique.

⚠️ **La réconciliation, dont cette story dit reprendre les critères, porte TROIS garde-fous
pour le même genre de calcul** : `LIMIT 50` dans le SQL candidat, `MAX_PROPOSALS_LIMIT = 500`
côté route — dont le commentaire dit en toutes lettres *« défense anti-DoS contre
`?limit=999999` »* — et un **index dédié** créé pour l'occasion.

⛔ **Deux facteurs aggravent le cas ici, et ils sont propres à cette story** : la **Réserve 2**
envisage de retirer la fenêtre de dates du filtre — ce qui ferait comparer **toutes** les
lignes ouvertes d'un compte entre elles, sans borne temporelle ; et **AC2 interdit de filtrer
sur le statut de facture**, donc l'ensemble candidat ne peut plus être réduit comme le fait la
réconciliation. Sur un compte fournisseur actif depuis plusieurs exercices — que **D3 autorise
explicitement** à lettrer à cheval —, un appariement **quadratique non plafonné** est un
calcul lourd et bloquant.

⚠️ **L'index nécessaire est créé par 15-1a, pas ici** *(tranché en passe 2 : AC7 exigeait un
composite que **personne** ne créait — 15-1a ne posait qu'un `idx_jel_lettering` simple)*. La
migration vit dans le socle, donc **`idx_jel_account_lettering (account_id, lettering_id)` y a
été ajouté** ; 15-1c le **consomme**, elle ne le crée pas.

⚠️ **À vérifier au démarrage de 15-1c** : que le socle l'a bien livré. Sans lui, la requête
`WHERE account_id = ? AND lettering_id IS NULL` balaie le compte entier — les deux index
simples préexistants — `idx_jel_entry`, `idx_jel_account` **et `idx_jel_project`**
*(P3-9 : le décompte disait deux, il y en a **trois**)* — ne la servent pas.

## ⛔ Décisions ouvertes au terme de la passe 3

**Elles ne se tranchent ni par l'orchestrateur ni par le développeur** — quatre sur cinq sont
des **coutures entre les trois fiches**, pas des défauts internes à celle-ci.

✅ **(1) RÉSOLUE — l'arbitrage du 2026-08-26 a supprimé la contradiction.** *(Était le HIGH
P3-1 : 15-1c exigeait qu'une paire fournisseur payée soit **proposée**, 15-1b qu'elle soit
**invisible** — le même écran aurait dit « 0 ligne ouverte » et « 1 rapprochement proposé » sur
les mêmes deux lignes.)*

**« Ouvert » signifie désormais « non lettré », un point c'est tout** : la vue de 15-1b ne
joint plus aucune table de factures et ne regarde plus `paid_at`. **Les deux fiches
s'accordent** — l'ensemble affiché et l'ensemble appariable sont **le même**, et la règle de
D5 (« ne jamais filtrer sur la facture ») devient la règle **commune**, non plus une divergence
à arbitrer.

⚠️ **Ce que 15-1c y gagne au passage** : sa règle n'a plus à être justifiée contre sa sœur, et
le test de la paire fournisseur d'AC2 n'a plus de test miroir contradictoire à honorer.

**(2) MEDIUM — le moteur n'a aucune borne de rôle de compte** *(P3-7)*, là où 15-1b a tranché
`role IN ('Receivable','Payable')` **dans la requête**. Rien n'empêche de pointer le moteur sur
le **compte bancaire ledger**, où il apparierait des lignes que la réconciliation gère par un
tout autre mécanisme. 15-1c y répond par **un bandeau** ; la fiche sœur ferme la porte dans le
SQL.

**(3) MEDIUM — l'acceptation d'une proposition n'a ni contrat, ni convention de lot, ni
traitement de la proposition périmée** *(P3-8)*. ⚠️ Si l'écran permet d'accepter **plusieurs**
propositions — ce que « l'écran ressemblera à celui de la réconciliation » laisse attendre —,
le **pattern `FailedProposal` du `CLAUDE.md` s'applique** et rien ne le nomme : identifiant
métier, `error_code` canonique, HTTP 200 sur succès partiel. Et entre l'affichage et
l'acceptation, la paire peut avoir été lettrée ou son écriture modifiée.

**(4) MEDIUM — AC4 n'a ni tâche ni test** *(P3-5)*, et il exige une exposition d'API que
personne ne porte : `JournalEntryLineResponse` (`kesh-api/src/routes/journal_entries.rs:101`)
n'a **aucun champ de lettrage**, et la passe 8 de 15-1a cite ce DTO comme **preuve d'absence
d'effet de bord** — donc comme raison de **ne pas** le toucher. ⚠️ **Et AC4 a besoin du CODE,
pas de l'identifiant** : `lettering_id` est un entier opaque, le code `A`, `B` vit dans
`letterings.code`. C'est le défaut que 15-1a T5-bis nomme pour l'export, transposé à l'API.

## Tasks

- [ ] **T1** — Moteur de proposition (D5). ⚠️ **Ce qui s'extrait, ce sont la fenêtre et le
      montant** — « même compte », « sens opposés » et `lettering_id IS NULL` sont **neufs**,
      sans précédent dans `kesh-reconciliation`. Et **ne pas reprendre** les filtres de
      facture (`status`, `paid_at`), propres à `invoices`.
      ⛔ **Borne de volume et index (AC7)** avant d'écrire la requête : portée par compte,
      plafond serveur, composite `(account_id, lettering_id)`.
- [ ] **T2** — Routes de proposition. ⛔ **Scoping par jointure (AC6)**, écrêtage du `?limit=`
      reçu (AC7), et inscription au tableau du Pattern 5.
- [ ] **T3** — Écran dédié, avec la frontière énoncée (AC5).
- [ ] **T4** — Tests : **AC2 en priorité**, ses trois cas nommés, avec l'écart de dates
      réaliste sur la contre-passation. Puis AC3 et AC5.
      ⛔ Plus **AC6** (une ligne d'une autre société n'est **jamais** candidate — test d'IDOR),
      **AC7** (le plafond tient, et un `?limit=` démesuré est écrêté), et le cas de **la ligne
      déjà lettrée** : elle ne doit **jamais** apparaître en proposition. Ce dernier manquait
      à la fois du tableau des critères **et** de la liste des tests.
- [ ] **T5** — i18n : quatre locales dès l'écriture, allowlist vide.
- [ ] **T6** — Manuel utilisateur : ce que le lettrage fait, et **ce qu'il ne fait pas
      encore** — le partiel et le groupé.

## Dev Notes

⚠️ **Le sélecteur E2E ne se fige jamais sur un libellé traduit** — `data-testid` sans
exception (garde #326).

⚠️ **Un E2E n'est pas un test comme un autre** : c'est le seul qui vérifie qu'une valeur
traverse réellement la frontière HTTP.

## Change Log

### Arbitrage du Project Lead — 2026-08-26 : « ouvert » = « non lettré »

⛔ **La CINQUIÈME décision — celle que ni 15-1b ni 15-1c ne portait — est tranchée.** Les deux
passes 3, indépendantes, avaient conclu qu'elle décidait de la forme de la requête que les
deux stories allaient écrire.

> **« Ouvert » signifie « non lettré ». La vue ne regarde ni `paid_at`, ni aucun statut de
> facture, et ne joint aucune table de factures.**

✅ **Ce que l'arbitrage achète — un INVARIANT, pas une commodité.** Toute paire lettrée se
nettant exactement à zéro (15-1a AC4 et AC12), **la somme algébrique des lignes ouvertes d'un
compte égale son solde**, sans exception. C'est **testable en une assertion**, et c'est ce qui
rend enfin vraie la promesse du *so that* : *« justifier le solde d'un compte »*. Aucune des
deux définitions concurrentes ne le permettait.

⛔ **Ce qu'il coûte, et qui doit être assumé à l'écran** : une facture réglée par virement
importé réapparaît **ouverte** tant qu'elle n'est pas lettrée. C'est **comptablement vrai** —
la réconciliation ne crée aucune écriture, le compte porte toujours son débit — mais
contre-intuitif. **AC4 de 15-1b devient de ce fait le critère le plus important de la fiche**,
et il doit offrir un chemin vers le lettrage, pas seulement une explication.

✅ **Ce qu'il SUPPRIME, et c'est le plus notable** : **trois HIGH et trois MEDIUM des passes 1
à 3 tombent avec lui** — la contradiction entre fiches sœurs, la déclinaison en trois puis
quatre cas, les deux tables de factures, les deux écritures fournisseur, la troisième écriture
d'annulation non référencée. **Ce n'est pas une simplification cosmétique : c'est la
disparition de la classe entière de défauts que ces passes trouvaient**, tous nés de ce que la
vue tentait de concilier deux mécanismes que rien n'oblige à concilier.

⚠️ **Ce qu'il NE tranche PAS.** La Décision 1 (relance) reste ouverte et son enjeu se
**déplace** : la vue ne lit plus `paid_at`, mais les **cinq lecteurs** recensés continuent de
le lire — et le plus grave est comptable, pas cosmétique. `reconciliation.rs` proposera une
facture lettrée mais non marquée payée à un **second règlement** : **soldée deux fois**, une
fois en caisse et une fois en banque. La Décision 3 reste ouverte pour la même raison.

### Passe 3 de `validate` — 2026-08-25 (Opus, contexte frais)

⛔ **2 HIGH, 6 MEDIUM, 4 LOW. LA SÉVÉRITÉ REMONTE** — `2 HIGH+2 MED` → `2 MED+1 LOW` →
`2 HIGH+6 MED`. **Le critère de non-convergence de la § *Règle de splitting préventif* est
déclenché**, pour la seconde fois dans cet epic.

⚠️ **Mais le diagnostic n'est PAS « la fiche est trop large » — elle est trop COUPLÉE à ses
sœurs.** Cinq des huit findings > LOW sont des **coutures** entre 15-1a, 15-1b et 15-1c, et
deux n'existent que depuis la passe 2 de **15-1b**, tombée pendant cette revue. **Une passe 4
sur 15-1c seule ne verrait pas la suivante.** Ce qu'il faut n'est pas une passe de plus, mais
**une relecture des trois fiches ENSEMBLE sur la seule question « qu'est-ce qui est ouvert, et
qui le dit ».**

⛔ **P3-1 (HIGH) — 15-1c et 15-1b prescrivent, chacune par un test nommé, deux comportements
INCOMPATIBLES.** Sur le compte fournisseur, AC2 exige que la paire soit **proposée** ;
AC3-bis de 15-1b exige que le compte n'affiche **aucune ligne ouverte**. Le même écran dirait
« 0 ligne ouverte » et « 1 rapprochement proposé » sur **les mêmes deux lignes**. → **décision
ouverte**, c'est un arbitrage de produit.

⛔ **P3-2 (HIGH) — AC7 ne bornait pas ce qu'il disait borner.** Ses deux bornes agissaient
**après** l'appariement ; le `LIMIT 50` qu'il citait en modèle borne le **jeu candidat**, dans
un `WHERE` que la fenêtre **et** la tolérance réduisent. Or la passe 2 a retiré la fenêtre du
filtre et AC2 interdit `status`/`paid_at` : **il ne restait aucun prédicat réducteur**. Et
l'analogie était fausse — la réconciliation est **1 → N**, le lettrage **N → N**. → borne sur
le **jeu candidat**, plafond **chiffré à 500**, et la remarque que l'égalité stricte rend
l'appariement **groupable par montant**, donc linéaire.

**Quatre MEDIUM corrigés ici** : **P3-3** le « classement » tranché en passe 2 n'existait dans
aucun critère — le plafond tronquait un ensemble non ordonné, évinçant **en premier** la
contre-passation que la Réserve 2 protège → **AC4-bis** ; **P3-4** la fenêtre figurait
**toujours** comme critère d'éligibilité dans le tableau, la passe 2 ayant corrigé les deux
sites qui en *parlent* et laissé les deux qui la *prescrivent* — le geste même que la
§ *Propagation post-patch* codifie ; **P3-6** ⚠️ **mon affirmation « ce document exige que tout
nouvel endpoint y figure » était FAUSSE** — le Pattern 5 impose un ordre à ceux qui prennent
**plus d'un verrou**, et les routes de 15-1c sont des routes de **lecture** ; **P3-9/P3-10** le
décompte d'index (trois, pas deux) et le titre du tableau, que sa propre colonne contredisait.

**Quatre MEDIUM laissés en décisions ouvertes** — voir le § dédié : la borne de rôle du moteur,
le contrat d'acceptation et le pattern de lot, AC4 sans tâche ni test.

**Réfuté** : le déplacement de l'index vers 15-1a est **correct des deux côtés**, `CREATE INDEX`
n'impose aucun bump `min_required`, et la colonne « provenance » est exacte sur ses trois lignes
NEUF.

**Verdict : relecture croisée des trois fiches due, pas une passe 4 sur celle-ci.**

### Passe 2 de `validate` — 2026-08-25 (Haiku, contexte frais)

**0 HIGH, 2 MEDIUM, 1 LOW.** Sévérité décroissante (`HIGH → MEDIUM`) : convergence monotone.

**Aucune régression des patches de la passe 1** : AC6, AC7, la colonne « provenance » et le
critère `lettering_id IS NULL` sont vérifiés exacts au sol et jugés bien posés.

⚠️ **Les deux MEDIUM ne sont pas des défauts de raisonnement mais des ambiguïtés de
COORDINATION entre fiches** — la classe d'erreur que le split fabrique, et la troisième fois
qu'elle se manifeste dans cet epic.

⛔ **P2-1 — AC7 exigeait un index composite que PERSONNE ne créait.** 15-1a ne posait qu'un
`idx_jel_lettering (lettering_id)` simple ; le composite `(account_id, lettering_id)` n'était
la tâche d'aucune des deux fiches. → **Tranché : la migration vit dans le socle**, donc
`idx_jel_account_lettering` a été **ajouté au DDL de 15-1a** ; 15-1c le **consomme**.

⚠️ **Et les deux index sont nécessaires**, le préfixe gauche ne permettant pas de les
confondre : `(lettering_id)` sert « retrouver la contrepartie d'une marque »,
`(account_id, lettering_id)` sert « les lignes ouvertes du compte A » — la requête du moteur
**et** celle de la vue de 15-1b.

⛔ **P2-2 — « tranchée par défaut » ne disait pas LAQUELLE des deux conduites était le défaut,
et AC2 en dépendait.** → **Tranché : la fenêtre s'applique au CLASSEMENT, pas au filtre.**
C'est la seule lecture cohérente avec AC2, qui exige un test datant les deux pièces à **plus de
30 jours d'écart** : si la fenêtre filtrait, ce test **échouerait par construction**, et deux
développeurs auraient raison en même temps — celui qui écrit le moteur avec un filtre, celui
qui écrit le test tel qu'AC2 le prescrit. **C'est la contradiction « proposer ce qu'on refuse »
de la Réserve 1, transposée à la date.**

**P2-3 (LOW)** : « inscrire au tableau du Pattern 5 » ne disait ni quoi ni sous quelle forme —
le format des lignes voisines est désormais nommé.

**Vérifié et réfuté** : multi-devise et arrondis `DECIMAL(19,4)` sans objet ; AC1 et AC2
complémentaires, pas antagonistes ; la Réserve 1 correctement posée, sa conséquence énoncée.

**Verdict : passe 3 due** — deux MEDIUM au rapport. ⚠️ Aucun ne relève d'un arbitrage produit :
les deux sont tranchés ici sur la cohérence interne, et **restent réversibles d'un mot**.

### Passe 1 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**2 HIGH, 2 MEDIUM.** Tous vérifiés au sol par l'orchestrateur avant application.

⚠️ **Les quatre findings répètent le motif des huit passes de 15-1a** : une lecture de la spec
contre elle-même les aurait tous laissés passer. **Seule la lecture des chemins de code de
l'Epic 8 et du schéma de `journal_entry_lines` les révèle.**

| | défaut | gravité | remède |
|---|---|---|---|
| **P1-2** | **Aucune mention de scoping multi-tenant** — zéro occurrence de `company_id`, « multi-tenant » ou « IDOR ». Or 15-1c ouvre une **surface d'API neuve** par un chemin **distinct** des routes que 15-1a a scopées, et `journal_entry_lines` n'a **aucun** `company_id` | **HIGH** | **AC6** — jointure, 404 indiscernable, inscription au Pattern 5 |
| **P1-4** | **Aucune borne de performance ni anti-DoS** — T1 et T2 tenaient en une phrase. La réconciliation en porte **trois** pour le même calcul : `LIMIT 50`, `MAX_PROPOSALS_LIMIT = 500` (*« défense anti-DoS contre `?limit=999999` »*) et un index dédié | **HIGH** | **AC7** — portée par compte, plafond serveur, composite `(account_id, lettering_id)` |
| **P1-1** | **La provenance de deux critères sur quatre était FAUSSE** : « même compte » et « sens opposés » **n'existent nulle part** dans l'Epic 8, dont le scoring est `0,50 montant + 0,40 référence + 0,10 contact`. Pire, `rules.rs` a **délibérément retiré** le seul filtre de sens qui ait existé | MEDIUM | colonne « provenance » au tableau ; T1 dit ce qui s'extrait et ce qui est neuf |
| **P1-3** | **Le critère `lettering_id IS NULL` manquait** au tableau **et** aux tests. AC10 de 15-1a protège la route, mais **rien n'empêchait le MOTEUR de re-proposer** une paire déjà lettrée | MEDIUM | critère ajouté, test nommé |

⛔ **P1-4 est aggravé par deux traits propres à cette story**, et c'est ce qui le rend HIGH
plutôt que MEDIUM : la **Réserve 2** envisage de retirer la fenêtre de dates du filtre — donc
de comparer toutes les lignes d'un compte sans borne temporelle — et **AC2 interdit de filtrer
sur le statut de facture**, si bien que l'ensemble candidat ne peut plus être réduit comme le
fait la réconciliation. Sur un compte fournisseur actif depuis plusieurs exercices — que **D3
autorise explicitement** —, l'appariement devient **quadratique et non plafonné**.

⚠️ **P1-3 dit quelque chose du découpage** : le socle protège la **route**, la story de l'écran
alimente le **moteur**, et le critère qui les relie n'était écrit ni dans l'une ni dans l'autre.
C'est la même classe de trou que l'égalité des montants, tombée entre 15-1a et 15-1c et
rapatriée en passe 3.

**Six pistes réfutées au sol**, dont : la tolérance de 5 centimes n'est **pas** justifiée par
les frais bancaires *dans le code* — ce narratif vient de la story mère, le commentaire réel
dit seulement *« réduit le candidate set sans accepter le mismatch »* ; le montant TTC/HT ne
pose **pas** ici le problème qu'il a posé à la réconciliation (#246), les lignes de grand livre
portant déjà le TTC ; la paire facture/avoir **est** structurellement exacte, l'avoir créditant
`total_ht + total_vat` au même compte ; et le règlement fournisseur crée bien une écriture au
même compte, sens opposé, même TTC, **pour les deux modes de règlement**.

La spec passe de **5 à 7 critères**. **Verdict : passe 2 due.**

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**. Recueille la correction majeure de la passe 1 sur D5
(les filtres de facture qui excluaient trois cas sur quatre) et le MEDIUM **P3-7** de la
passe 3 (la fenêtre de 30 jours qui tue la contre-passation).

⛔ **Deux réserves restent ouvertes** : la tolérance de montant face aux frais bancaires, et
la fenêtre de dates. Toutes deux sont tranchées **par défaut** dans la spec, avec leur
conduite alternative nommée — elles se changent d'un mot tant que le développement n'a pas
commencé.
