# Story 15.1b : La vue « ce qui reste ouvert » — et ce qu'elle prétend égaler

## Status

draft

## Story

**As a** indépendant ou fiduciaire qui tient ses comptes dans Kesh,
**I want** voir d'un coup d'œil ce qui reste ouvert sur un compte client ou fournisseur,
**so that** je puisse relancer les bons débiteurs, justifier le solde d'un compte, et clore
un exercice en sachant ce qu'il porte.

Deuxième des trois sous-stories issues du **split de la 15-1**. ⚠️ **Suppose la 15-1a
livrée** — elle lit la marque que celle-ci pose.

## Ce que cette story doit résoudre, et qui n'est pas ce qu'on croit

La 15-1 d'origine tenait « ouvert » pour une définition à écrire. La passe 3 a montré que
c'est une **question de fond non tranchée**, et qu'une implémentation fidèle à la spec
produisait un résultat faux **par trois chemins différents**.

### ⛔ Décision 1 — Lettrer ne calme aujourd'hui NI la balance âgée NI les relances

Le *so that* de cette story dit « relancer les bons débiteurs ». Or les deux dispositifs qui
relancent ne connaissent que `invoices.paid_at` :

```
crates/kesh-report/src/aged_receivables.rs:127      … AND i.paid_at IS NULL
crates/kesh-db/src/repositories/dunning_eligibility.rs:87   AND i.paid_at IS NULL
```

et `grep -rn "lettering" crates/ frontend/` rend **zéro occurrence** : rien ne les fera
changer d'avis.

⚠️ **Le scénario n'est pas théorique.** Facture réglée en espèces, lettrée. L'écran de
lettrage affiche « rien d'ouvert ». La **balance âgée** continue de la porter en 61-90 jours,
et le **moteur de relance envoie un rappel** — puis un deuxième. **Ce défaut-là n'est pas
muet : il est adressé au client.**

C'est le miroir exact de D4 : la 15-1 imposait à la vue de lettrage de lire `paid_at`, et
**rien n'imposait aux autres vues de lire la marque**. Un seul sens de la relation avait été
vu.

**Deux conduites, à arbitrer — c'est une décision de fond, pas d'implémentation :**

**(A)** Lettrer la ligne de créance d'une facture **pose `invoices.paid_at`** — source
unique, les trois vues s'accordent sans rien changer chez elles. ⚠️ **Mais `mark_as_paid`
n'a AUCUNE garde d'exercice** (cf. Décision 3), et ce chemin en hériterait.

**(B)** Le hors-périmètre est **assumé et énoncé**, au même titre que la borne du règlement
groupé : un critère dédié, un test, et une phrase à l'écran. Aujourd'hui, un développeur
fidèle à la spec n'implémentera **ni l'un ni l'autre**.

**(C)** ⚠️ **Faire lire la marque DIRECTEMENT aux deux requêtes**, sans passer par `paid_at`
*(conduite ajoutée en passe 1 — elle manquait, et c'est la seule des trois qui ne porte aucun
des deux risques ci-dessus)*. La ligne de créance d'une facture de vente est identifiable sans
ambiguïté : c'est **la seule ligne à `debit > 0`** de l'écriture de vente, au compte
`default_receivable_account_id` de la société.

⛔ **La forme SQL écrite en passe 1 était FAUSSE, et elle aurait été appliquée sans effet.**
*(P3-5, passe 3.)* Un `LEFT JOIN … AND lettering_id IS NULL` **n'exclut aucune ligne** — un
`LEFT JOIN` ne filtre pas. Il faut un `INNER JOIN … AND jel.lettering_id IS NULL`, ou une
anti-jointure (`LEFT JOIN … AND jel.lettering_id IS NOT NULL … WHERE jel.id IS NULL`). Tel
qu'écrit, un développeur appliquait (C), **rien ne changeait**, et le gate restait vert.

⛔ **Et la jointure DOIT porter ses deux discriminants** — `AND jel.debit > 0 AND
jel.account_id = <compte de créances>` — faute de quoi elle rend **N+1 lignes par facture**
(la créance **plus** toutes les lignes de produit et de TVA). ⚠️ `aged_receivables` est une
requête d'**agrégat** (`SUM()` par contact) : chaque tranche de la balance âgée serait
**multipliée par N+1**. Muet, et faux en argent. Le même oubli sur `dunning_eligibility`
produit des **rappels en double**.

**Le test qui garde la forme** : une facture à trois lignes de produit et deux taux de TVA,
non lettrée, doit apparaître **une seule fois** et pour son TTC.

⛔ **Et l'inventaire des LECTEURS était incomplet — il y en a CINQ, la conduite n'en nommait
DEUX.** *(P3-3, passe 3, recompté au sol.)*

| lecteur de `paid_at IS NULL` | conséquence si le lettrage ne l'atteint pas |
|---|---|
| `kesh-report/src/aged_receivables.rs` | balance âgée fausse — *cosmétique* |
| `kesh-db/src/repositories/dunning_eligibility.rs` | rappel envoyé à tort — *visible du client* |
| ⛔ `kesh-db/src/repositories/reconciliation.rs` | **la facture est proposée à un second règlement** — *écriture fausse* |
| `kesh-db/src/repositories/invoices.rs` (filtres « impayées »/« en retard », `due_dates_summary`) | listes et KPI faux |
| `kesh-api/src/routes/invoices.rs` (champ dérivé `isOverdue`) | affichage faux |

⛔ **Le troisième est d'une autre nature que les autres, et il se traite EN PRIORITÉ** : une
facture réglée en espèces et lettrée, mais laissée `paid_at IS NULL` — c'est-à-dire exactement
ce que (B) et (C) produisent — reste **candidate à la réconciliation**. Un virement du même
montant arrive un mois plus tard, l'utilisateur accepte : **la facture est soldée deux fois**,
une fois en caisse et une fois en banque. Ce n'est plus un rappel de trop, c'est une écriture
fausse.

⚠️ **Toute conduite retenue nomme lequel des cinq elle laisse dehors.** Corriger les deux
premiers en croyant l'écart fermé, c'est reproduire sur les **lecteurs** le défaut que la
passe 1 avait relevé sur les **écrivains** — par son propre correctif.

✅ **Sa prémisse est VÉRIFIÉE au sol** *(contrôle de passe 2)* :
`generate_invoice_journal_lines` (`kesh-db/src/repositories/invoices.rs:1368` et suivantes)
pousse **une seule ligne au débit** — la créance, `total_ht + total_vat` — puis **toutes** les
autres en crédit (produits, TVA). « La seule ligne à `debit > 0` » n'est donc pas une
approximation : c'est la structure engendrée par le code, quel que soit le nombre de lignes de
produit ou de taux de TVA.

⚠️ **Et l'avoir n'est pas un contre-exemple** : il inverse bien les sens, mais il porte **sa
propre écriture** — la jointure passant par `invoices.journal_entry_id` ne l'atteint pas.

⚠️ **Son coût** : une jointure de plus dans deux requêtes existantes. **Sa réserve** : elle ne
couvre que le lettrage, pas `paid_at` — elle est donc **orthogonale** aux deux autres, pas
concurrente. Rien n'interdit de retenir (C) *et* de documenter l'écart résiduel.

⛔ **(C) porte sur `aged_receivables` et `dunning_eligibility`, PAS sur la requête de cette
vue** *(P2-4, passe 2)*. La vue de 15-1b lit la marque de toute façon — c'est sa définition
même. Ce que la Décision 1 arbitre, c'est si les **deux autres dispositifs** la lisent aussi.
Un développeur qui confondrait les deux implémenterait la jointure au mauvais endroit.

### ⛔ Décision 2 — Côté client, `paid_at` n'a aucune contrepartie comptable

`accept_one_invoice` **ne crée aucune écriture** — il met à jour deux tables. Le repository
l'écrit noir sur blanc :

```
crates/kesh-db/src/repositories/invoices.rs:1923
    /// **Ne crée AUCUNE écriture comptable** en v0.1. […] Ici `paid_at`
    /// est un simple marqueur opérationnel.
```

Or la validation de facture **débite la créance** (`invoices.rs:1368`).

⚠️ **Conséquence, et elle touche le *so that* de la story** : dix factures réglées par
virement importé laissent le compte 1100 avec **dix débits jamais crédités** — son solde est
de dix factures. Une vue qui applique « ouvert = ni lettré ni marqué payé » affiche **zéro
ligne ouverte**. « Justifier le solde d'un compte » n'est alors pas atteignable, et c'est le
chemin que la 15-1 qualifiait elle-même de **plus fréquent**.

**Deux conduites** : afficher côte à côte le solde du compte et le total des lignes ouvertes,
en **assumant l'écart et en le nommant** ; ou restreindre la vue aux comptes qui **ont** une
contrepartie comptable — les fournisseurs, cf. Décision 4 — et déclarer le cas client comme
angle mort. ⛔ **Ce qui n'est pas tenable, c'est de laisser croire au développeur que la vue
justifie le solde.**

### ⛔ Décision 3 — Le canal `paid_at` n'a aucune garde d'exercice

⛔ **`paid_at` a TROIS écrivains en production, pas un — et la rédaction précédente n'en
nommait qu'un** *(relevé en passe 1, recompté par l'orchestrateur qui en a trouvé un de plus
que la passe)* :

| site | fonction | garde d'exercice ? |
|---|---|---|
| `kesh-db/src/repositories/invoices.rs:1989` | `mark_as_paid` (et le **dé-marquage**) | **non** |
| `kesh-api/src/routes/reconciliation.rs:1231` | `accept_one_invoice` — **le chemin le plus fréquent** selon AC2 | **non** |
| `kesh-db/src/repositories/supplier_invoices.rs:674` | `pay_in_tx` — côté fournisseur | **non** |

`mark_as_paid` vérifie `status = 'validated'` et la version optimiste. **Rien d'autre** — pas
de `fiscal_years`, pas de `FiscalYearClosed`. Il sert aussi au **dé-marquage**, exposé par
`POST /api/v1/invoices/:id/mark-paid` (`kesh-api/src/routes/invoices.rs:1033`). Les deux
autres écrivent la colonne **par SQL brut**, en court-circuitant complètement `mark_as_paid`.

⛔ **Pourquoi ce recomptage change la décision** : un développeur fidèle à l'ancienne
rédaction ouvre `mark_as_paid`, y pose la garde, et **croit le trou fermé**. Les deux autres
canaux continuent d'écrire sans garde — dont celui de la réconciliation, que AC2 désigne comme
le plus fréquent. **Toute conduite retenue doit couvrir les TROIS**, ou nommer explicitement
lequel elle laisse dehors.

⚠️ Exercice 2025 clos : le refus de délettrage de la 15-1a interdit de rouvrir une ligne.
Mais un **dé-marquage** rend cette même ligne « ouverte » au sens de la définition — le
résultat que le refus existe pour empêcher, obtenu par l'autre canal.

**Soit la garde d'exercice est étendue au marquage/dé-marquage** — changement hors périmètre,
à ouvrir en CR —, **soit la story écrit que sa garantie d'immuabilité est partielle**. La
laisser implicite fait croire à une protection qui n'existe pas.

### ⛔ Décision 4 — Le fournisseur n'est pas le symétrique du client, il est en avance

La 15-1 affirmait que « les comptes fournisseurs n'ont aucun équivalent de ce mécanisme ».
**C'est faux**, vérifié en passe 3 :

```
crates/kesh-db/src/repositories/supplier_invoices.rs:495   pub async fn pay(
crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:44,47,67-68
    settlement_type VARCHAR(20) NULL,
    settlement_journal_entry_id BIGINT NULL,
    CHECK (settlement_type IS NULL OR settlement_type IN ('bank_transfer', 'internal_account'))
```

Le fournisseur a **davantage** que le client : un règlement **hors import bancaire compris**
(`internal_account`), qui pose `paid_at`, `status='paid'`, **et une vraie écriture de
règlement** référencée par une colonne dédiée.

Deux conséquences :

1. Sur le compte fournisseur, **les deux lignes à lettrer existent déjà** — `C 2000` à
   l'achat, `D 2000` au règlement, même compte, sens opposés, même TTC. Il n'y a rien à
   construire, seulement à afficher.
2. ⛔ **Une facture fournisseur payée est rattachée à DEUX écritures.** Une vue qui joint sur
   `purchase_journal_entry_id` seul masque la ligne d'achat et **laisse la ligne de règlement
   ouverte à jamais** : le compte 2000 afficherait un débit fantôme **par facture payée**,
   alors que son solde est zéro.

## Critères d'acceptation

⚠️ **Les critères ci-dessous sont écrits pour rester vrais quel que soit l'arbitrage des
quatre décisions**, sauf là où c'est indiqué. Ils ne peuvent pas tous être figés avant.

**AC1** — Depuis l'écran, l'utilisateur choisit un compte et voit ses lignes **ouvertes**,
avec le total.

⛔ **La vue est BORNÉE aux comptes de créances et de dettes** — `role IN ('Receivable',
'Payable')` — et le mécanisme **existe déjà, il n'est pas à construire** *(relevé en passe 1 :
rien ne bornait la vue)*. `AccountRole::Receivable` / `::Payable`
(`kesh-db/src/entities/account.rs:92,96`) est déjà consommé par le bilan.

⚠️ **Trois rectifications, toutes vérifiées au sol** *(P3-6, passe 3)* : le singleton n'est
**pas** tenu par `chk_accounts_role` — qui ne contrôle que le **domaine de valeurs** — mais par
`uq_accounts_company_singleton_role`. Il ne vaut que **parmi les comptes ACTIFS**, la colonne
générée valant `NULL` dès qu'un compte est archivé : un compte de créances archivé **conserve**
`role = 'Receivable'` et passerait une borne écrite `role IN (…)`. Et le dépôt interroge
**toujours `singleton_role`**, jamais `role` — avec sa raison écrite : *« `role = ?` scannait
l'index, `singleton_role = ?` restaure l'accès `const` »* (revue de la Story 14-3b). Écrire
`role IN (…)` réintroduirait le défaut qu'une passe avait fermé.

⛔ **Et une question de fond reste ouverte** : `accounts.role` n'est **pas** la source de vérité
des écritures. Celles-ci visent `company_invoice_settings.default_receivable_account_id`, et
`accounts::update` change `role` **sans jamais toucher ces réglages**. Déplacer le rôle de 1100
vers 1101 laisserait les factures s'imputer sur 1100 tandis que la vue n'accepterait plus que
1101, **vide** : « rien d'ouvert » sur le compte qui porte tout le solde.

⚠️ **`account_type` ne suffit PAS** : un compte débiteur et un compte de caisse sont tous deux
`Asset`. Sans cette borne, ouvrir la vue sur un compte de produit ou sur le compte bancaire
ledger exclurait des lignes *« parce que la facture derrière est payée »* — un critère qui
**n'a aucun sens comptable** sur ces comptes-là. Et côté compte bancaire, cela chevaucherait
en silence ce que la réconciliation gère par un tout autre mécanisme
(`bank_transactions.status`) : exactement la confusion que 15-1c s'efforce de nommer **à
l'écran**, et qu'il faut d'abord fermer **dans la requête**.

**AC1-bis** — ⚠️ **La vue est un instantané « aujourd'hui », et elle le dit** *(relevé en
passe 1)*. Elle ne répond **pas** à « qu'est-ce qui était ouvert au 31.12 ». Le rapport voisin
le plus proche, `aged_receivables`, porte un paramètre `as_of` **bindé et non `UTC_DATE()` en
dur, pour la testabilité** — la vue de lettrage n'en a pas.

⚠️ **Le *so that* de cette story invoque pourtant la clôture** (« clore un exercice en sachant
ce qu'il porte ») : une facture de 2024 réglée en janvier 2026 porte `paid_at` **avant** la
consultation, et n'apparaîtra donc **jamais** comme ouverte au 31.12.2024 — alors qu'elle
l'était. **Soit un `as_of` est ajouté, soit la limite est écrite** ; la laisser implicite fait
promettre à la vue ce qu'elle ne tient pas.

**AC2** — ⚠️ Une facture réglée par la **réconciliation bancaire** n'apparaît **pas** comme
ouverte, bien qu'elle ne porte aucune marque de lettrage. « Ouvert » = **ni lettré, ni marqué
payé**. C'est le chemin le plus fréquent, et une vue qui ne regarderait que la marque
mentirait là où elle doit informer.

**AC3** — ⚠️ **La règle vaut pour les DEUX tables de factures**, `invoices` et
`supplier_invoices`, chacune avec son propre `paid_at`. **Un test par table.**

**AC2-bis** — ⛔ **« Ni lettré, ni marqué payé » ne s'applique PAS uniformément : une écriture
manuelle n'a AUCUN `paid_at`.** *(Relevé en passe 2.)* La définition doit donc se décliner en
**trois cas**, et la requête les distingue :

| la ligne appartient à… | « ouverte » signifie |
|---|---|
| une facture **client** | `invoices.paid_at IS NULL` **et** pas de marque |
| une facture **fournisseur** | `supplier_invoices.paid_at IS NULL` **et** pas de marque |
| **aucune facture** — écriture manuelle | **pas de marque**, un point c'est tout |

⚠️ **Le troisième cas est celui qui casse en silence** : un filtre `paid_at IS NULL` écrit
naïvement sur une jointure externe **exclut** les lignes sans facture — or les écritures
manuelles qui se soldent sont **l'un des quatre cas d'usage** que le lettrage existe pour
couvrir. Le test qui l'attrape : deux écritures manuelles au compte de créances, sens opposés,
aucune facture derrière — **les deux doivent apparaître ouvertes**.

**AC3-bis** *(rectifié en passe 3 — la passe 1 avait corrigé la fixture dans T4 et laissé
le critère, si bien que l'assertion prescrite était **fausse par construction** avec la
fixture imposée)* : **les deux lignes de la facture PAYÉE disparaissent des ouverts ; celles
d'une facture NON payée sur le même compte y restent.** ⚠️ **Et pour les DEUX écritures d'une
facture fournisseur payée** *(relevé en
passe 3)* : l'achat (`purchase_journal_entry_id`) **et** le règlement
(`settlement_journal_entry_id`). Une jointure qui n'en voit qu'une laisse l'autre ouverte à
jamais. **Un test qui paie une facture fournisseur et vérifie que le compte 2000 n'affiche
plus AUCUNE ligne ouverte.**

**AC4** — L'écran indique **pourquoi** une facture manifestement réglée peut rester ouverte :
paiement partiel ou règlement groupé, hors périmètre. Sans cela, la vue paraît fausse là où
elle est seulement bornée.

**AC5** — *(dépend de la Décision 1)* La relation entre la marque de lettrage, la **balance
âgée** et le **moteur de relance** est **explicite** : soit lettrer pose `paid_at` et les
trois vues s'accordent, soit l'écart est énoncé à l'écran et couvert par un test qui
**documente** le comportement retenu. ⛔ **Le silence n'est pas une option** : il produit un
rappel envoyé à un débiteur soldé.

**AC6** — *(dépend de la Décision 2)* Ce que la vue prétend égaler est **écrit**. Si elle ne
justifie pas le solde du compte, elle ne le laisse pas croire.

**AC7** — *(dépend de la Décision 3)* La portée de la garantie d'immuabilité sur exercice
clos est **énoncée** : ce que le refus de délettrage protège, et ce que le canal `paid_at`
laisse passer.

## Tasks

- [ ] **T1** — Repository : lister les lignes ouvertes d'un compte. ⚠️ La requête implémente
      la définition d'« ouvert » — les jointures sur les **deux** tables de factures, les
      **deux** écritures fournisseur **et les trois cas d'AC2-bis** en font partie.
      ⛔ **La borne de rôle d'AC1 s'applique DANS LA REQUÊTE, pas seulement à l'écran**
      *(P2-2, passe 2 : AC1 disait « la vue est bornée » sans dire où)*. Un compte hors
      `role IN ('Receivable','Payable')` ne rend **aucune ligne**. Une borne posée seulement au
      frontend laisserait la route la contourner — et le développeur qui lit T1 seul ne
      l'implémenterait jamais.
      ⚠️ **La requête est un instantané « aujourd'hui »** (AC1-bis) : elle lit `paid_at` tel
      qu'il est **au moment de l'appel**, sans date de référence. C'est la limite énoncée par
      AC1-bis, pas un oubli — mais elle se code explicitement, pas par omission.
- [ ] **T2** — Route `GET` lignes ouvertes d'un compte. ⛔ **Paginée**, sur le patron
      systématique du dépôt — `const MAX_LIMIT: i64 = 500` et `list_by_company_paginated`
      (`kesh-db/src/repositories/journal_entries.rs:541,673`), repris à l'identique par
      `credit_notes`, `payment_batches`, `supplier_invoices` et `users` *(relevé en passe 1 :
      aucune pagination n'était prévue)*. Un compte de créances actif depuis plusieurs
      exercices accumule des centaines de lignes ouvertes — d'autant plus tant que la
      Décision 1 n'est pas tranchée.
- [ ] **T3** — Écran de consultation.
- [ ] **T4** — Tests : **AC2, AC3 et AC3-bis en priorité** — le piège de la définition, sur
      les deux tables **et** les deux écritures. Puis le test qui matérialise l'arbitrage
      d'AC5, et **AC1** (un compte hors `Receivable`/`Payable` n'ouvre pas la vue).
      ⛔ Plus **AC2-bis** : deux écritures manuelles au compte de créances, sens opposés,
      **aucune facture derrière** — les deux doivent apparaître **ouvertes**. C'est le cas que
      la définition « ni lettré ni marqué payé » exclut en silence si on l'applique sans le
      décliner.
      ⛔ **La fixture d'AC3-bis porte DEUX factures fournisseur sur le compte 2000, une payée
      et une NON payée** *(relevé en passe 1)*. Avec une seule facture, le test ne valide que
      la **sous**-exclusion — il ne distingue pas une implémentation correcte d'une
      implémentation qui **masque tout le compte** dès qu'une facture y est payée. La
      non-payée doit **rester** dans les lignes ouvertes après le règlement de l'autre.
- [ ] **T5** — i18n : quatre locales, allowlist vide.
- [ ] **T6** — Manuel utilisateur : ce que la vue montre, et **ce qu'elle ne montre pas
      encore**.

## Dev Notes

⚠️ **Le sélecteur E2E ne se fige jamais sur un libellé traduit** — `data-testid` sans
exception (garde #326, son allowlist ne doit pas s'allonger).

⚠️ **La base de gate se remet à zéro AVANT le gate**, inconditionnellement (KF-039, #310).

## Change Log

### Passe 3 de `validate` — 2026-08-25 (Opus, contexte frais)

⛔ **3 HIGH, 6 MEDIUM, 3 LOW. LA SÉVÉRITÉ REMONTE** (`0 HIGH` en passe 2 → `3 HIGH`). Critère
de non-convergence franchi — **et 15-1c a franchi le sien dans la même heure**.

⚠️ **Les deux rapports, indépendants, concluent à la MÊME chose : les fiches ne sont pas trop
larges, elles sont trop COUPLÉES.** Aucun des trois HIGH n'est visible en relisant la fiche
contre elle-même ; les trois le sont en la relisant contre **le code** et contre **ses sœurs**.

⛔ **P3-2 (HIGH) — 15-1b et 15-1c donnent DEUX définitions différentes d'« ouvert ».** Ici,
`paid_at IS NULL` est **constitutif** ; là-bas, il est **interdit** à l'éligibilité. Le cas
d'usage nº 1 de l'epic le révèle : facture réglée en espèces, écriture de caisse passée à la
main **et** facture marquée payée. La vue **cache** le débit de créance et **affiche** le
crédit de caisse ; le moteur, lui, **propose la paire**. L'écran montre un rapprochement dont
la vue ne rend qu'une moitié, et **le total des lignes ouvertes devient négatif**. → **c'est
une CINQUIÈME décision, portée par aucune des deux fiches.**

⛔ **P3-3 (HIGH) — l'inventaire des LECTEURS de `paid_at` était incomplet : CINQ, la conduite
(C) en nommait DEUX.** Le troisième est d'une autre nature — `reconciliation.rs` propose une
facture lettrée mais non marquée payée à un **second règlement** : la facture est **soldée deux
fois**, une fois en caisse et une fois en banque. ⚠️ **C'est le mode d'échec que la passe 1
avait relevé sur les ÉCRIVAINS, reproduit sur les LECTEURS par son propre correctif.**

⛔ **P3-1 (HIGH) — le quatrième cas d'AC2-bis existe, et la fiche sœur le nomme depuis sa
passe 1** : la facture **annulée par un avoir** passe à `cancelled`. Et les deux SQL cités en
Décision 1 étaient **tronqués** — ils portent aussi `AND i.status = 'validated'`. Un
développeur calquant la jointure sur les voisins que la fiche lui désigne ferait disparaître le
débit de vente et laisserait le crédit d'avoir **ouvert à jamais** : la paire la plus propre du
lettrage deviendrait la seule qu'on ne peut pas lettrer.

**Trois erreurs factuelles de mes patches, corrigées ici :**

| | ce qui était écrit | ce qui est vrai |
|---|---|---|
| **P3-5** | *« un `LEFT JOIN … AND lettering_id IS NULL` s'ajoute au filtre »* | **un `LEFT JOIN` n'exclut RIEN** — (C) aurait été appliquée **sans effet**, gate vert. Et sans ses deux discriminants, elle rend **N+1 lignes par facture** : `aged_receivables` étant un **agrégat**, chaque tranche serait multipliée |
| **P3-6** | *« singleton tenu par `chk_accounts_role` »* | c'est `uq_accounts_company_singleton_role` ; le singleton ne vaut que **parmi les actifs** ; et le dépôt interroge **toujours `singleton_role`**, jamais `role` |
| **P3-8** | AC3-bis exigeait « aucune ligne ouverte » | la fixture imposée par T4 en passe 1 rend cette assertion **fausse par construction** — la passe 1 avait corrigé la tâche et laissé le critère |

**Six MEDIUM restants**, dont : une **troisième conduite** manque à la Décision 2 — ne pas
filtrer sur `paid_at` du tout, seule qui rende vrai l'invariant *« somme des lignes ouvertes =
solde du compte »*, testable en une assertion ; une société **sans rôle configuré** voit une
vue vide sans que rien ne le lui dise ; « le total » d'AC1 ne dit pas s'il porte sur la page ou
sur l'ensemble — **700 lignes, une page de 500, un total amputé de 29 %** sans le moindre
signe ; le troisième cas d'AC2-bis est **mal nommé** (« aucune facture » recouvre l'avoir et la
contre-passation, pas seulement le manuel) ; et une facture fournisseur peut porter **TROIS**
écritures, la troisième n'étant **référencée par aucune colonne**.

**Réfuté** : une facture payée **ne peut pas** être créditée (`credit_notes.rs:300` le refuse),
donc pas de crédit d'avoir orphelin par ce chemin ; les trois FK nécessaires sont **indexées**,
la requête de T1 n'a aucun problème de plan ; et la prémisse de (C) est confirmée **une
troisième fois**, au niveau du schéma cette fois. ⚠️ **Mais elle ne se transpose pas au
fournisseur** : à l'achat la ligne du compte 2000 est la seule à `credit > 0`, **au règlement
la seule à `debit > 0`** — le discriminant **change de sens selon l'écriture**.

**Verdict : ni une passe 4, ni un split — une DÉCISION et une relecture croisée des trois
fiches.**

### Passe 2 de `validate` — 2026-08-25 (Haiku, contexte frais)

**0 HIGH, 3 MEDIUM, 2 LOW.** Sévérité décroissante (`HIGH → MEDIUM`) : convergence monotone.
Le recompte des **trois** écrivains de `paid_at` est confirmé exact au sol.

⛔ **P2-1 (MEDIUM) — « ni lettré, ni marqué payé » ne s'applique PAS uniformément : une
écriture manuelle n'a AUCUN `paid_at`.** La définition se décline en **trois** cas — facture
client, facture fournisseur, et **aucune facture**. ⚠️ **Le troisième casse en silence** : un
filtre `paid_at IS NULL` écrit naïvement sur une jointure externe **exclut** les lignes sans
facture — or les écritures manuelles qui se soldent sont **l'un des quatre cas d'usage** que le
lettrage existe pour couvrir. → **AC2-bis**, avec son test.

**P2-2 (MEDIUM)** — AC1 disait « la vue est bornée aux comptes `Receivable`/`Payable` » **sans
dire où**. Une borne posée au seul frontend laisserait la route la contourner, et un
développeur lisant T1 seul ne l'implémenterait jamais. → T1 dit **dans la requête**.

**P2-3 (MEDIUM)** — la limite temporelle d'AC1-bis se **code explicitement**, elle ne
s'obtient pas par omission. → précisé à T1.

**P2-4 (LOW)** — **la conduite (C) porte sur `aged_receivables` et `dunning_eligibility`, PAS
sur la requête de cette vue** : celle-ci lit la marque de toute façon, c'est sa définition
même. Ce que la Décision 1 arbitre, c'est si les **deux autres dispositifs** la lisent aussi.
Confondre les deux ferait implémenter la jointure au mauvais endroit.

✅ **Et un contrôle que la passe n'avait PAS fait, mené par l'orchestrateur parce qu'il décide
de la conduite (C)** : la prémisse *« la ligne de créance est la seule à `debit > 0` »* est
**vraie**. `generate_invoice_journal_lines` (`invoices.rs:1368` sq.) pousse **une seule ligne
au débit** — la créance TTC — puis **toutes** les autres en crédit, quel que soit le nombre de
lignes de produit ou de taux de TVA. Et l'avoir n'est pas un contre-exemple : il inverse les
sens mais porte **sa propre écriture**, hors d'atteinte de la jointure par
`invoices.journal_entry_id`. **La conduite (C) est donc réalisable telle qu'écrite.**

La spec passe de **9 à 10 critères**. **Verdict : passe 3 due.**

### Passe 1 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**3 HIGH, 3 MEDIUM.** Tous vérifiés au sol par l'orchestrateur avant application — et **l'un
d'eux recompté à la hausse**.

⚠️ **Le problème n'était pas le CONTENU des quatre décisions ouvertes, mais leur
EXHAUSTIVITÉ.** Trois sur quatre étaient correctement posées et vérifiées ; la quatrième
(Décision 4, les deux écritures fournisseur) est **exacte à la lettre**. Mais deux d'entre
elles reposaient sur un relevé incomplet — et le Project Lead aurait tranché sur un tableau
d'options faux.

⛔ **P1-1 (HIGH) — `paid_at` a TROIS écrivains, la spec n'en nommait qu'un.** La passe en avait
trouvé deux ; le recompte de l'orchestrateur en établit **trois** :
`invoices.rs:1989` (`mark_as_paid`, et le dé-marquage), `reconciliation.rs:1231`
(`accept_one_invoice` — **le chemin le plus fréquent** selon AC2) et `supplier_invoices.rs:674`
(`pay_in_tx`). **Aucun des trois n'a de garde d'exercice**, et les deux derniers écrivent par
SQL brut en court-circuitant `mark_as_paid`. Un développeur fidèle à l'ancienne rédaction
aurait posé la garde à un seul endroit et **cru le trou fermé**.

⛔ **P1-2 (HIGH) — une conduite manquait à la Décision 1, et c'est la moins risquée.**
**(C)** faire lire la marque **directement** aux deux requêtes de relance, sans passer par
`paid_at` : la ligne de créance est la seule à `debit > 0` de l'écriture de vente, donc
identifiable sans ambiguïté. Un `LEFT JOIN … AND lettering_id IS NULL` s'ajoute au filtre
existant **sans toucher `mark_as_paid`** — donc **sans hériter du trou d'exercice** que (A)
traîne. Elle est **orthogonale** aux deux autres, pas concurrente.

⛔ **P1-3 (HIGH) — la vue n'était bornée à aucun type de compte**, alors que le mécanisme
existe déjà : `AccountRole::Receivable` / `::Payable`, singleton par société, tenu par
`chk_accounts_role` et déjà consommé par le bilan. ⚠️ **`account_type` ne suffit pas** — un
compte débiteur et une caisse sont tous deux `Asset`. Sans borne, la vue ouverte sur un compte
de produit exclurait des lignes *« parce que la facture derrière est payée »*, critère qui n'a
**aucun sens comptable** là ; et sur le compte bancaire, elle chevaucherait en silence la
réconciliation — la confusion même que 15-1c nomme à l'écran, à fermer d'abord dans la requête.

**Trois MEDIUM** : **P1-4** aucun paramètre temporel, alors que le *so that* invoque la clôture
— une facture de 2024 réglée en 2026 n'apparaîtra **jamais** comme ouverte au 31.12.2024, et
`aged_receivables` porte un `as_of` bindé pour cette raison exacte → **AC1-bis** ; **P1-5**
aucune pagination, alors que le dépôt a un patron systématique (`MAX_LIMIT = 500`) ; **P1-6**
le test d'AC3-bis ne validait que la **sous**-exclusion — avec une seule facture en fixture, il
ne distingue pas une implémentation correcte d'une qui **masque tout le compte** dès qu'une
facture y est payée.

**Pistes réfutées au sol**, dont trois qui rassurent : `accept_one_invoice` **ne crée
effectivement aucune écriture** ; un lot de paiement fournisseur n'appelle **pas** une écriture
partagée — `confirm_batch` boucle facture par facture, chacune avec sa propre paire ; et
**aucun canal de démarquage n'existe côté réconciliation**, le seul restant `mark_as_paid`. Le
risque d'IDOR sur le choix du compte a été écarté : le dépôt applique `find_by_id_in_company`
**sans un seul contre-exemple**.

La spec passe de **7 à 9 critères**. **Verdict : passe 2 due** — et elle doit précéder
l'arbitrage, faute de quoi le Project Lead tranchera sur des options incomplètes.

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**. Recueille les deux HIGH de définition (**P3-3** relance
et balance âgée, **P3-4** les deux écritures fournisseur) et deux MEDIUM (**P3-6** `paid_at`
sans contrepartie comptable, **P3-8** le canal sans garde d'exercice).

⛔ **Quatre décisions de fond restent ouvertes et bloquent le développement.** Elles sont
posées en tête, chacune avec ses conduites possibles et son coût. Ce ne sont pas des
précisions manquantes : chacune, laissée au développeur, produit un résultat faux — et trois
d'entre elles le produisent **en silence**.
