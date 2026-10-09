# Story 15.1b : Les postes ouverts d'un compte, à une date — et les rapprochements que Kesh propose

## Status

ready-for-dev *(réécrite le 2026-10-08, à valider — `bmad-create-story validate` avant tout développement)*

## Story

**As a** indépendant, PME ou fiduciaire,
**I want** obtenir, pour un compte de créances, de dettes ou de passage, la liste de ce qui y
reste ouvert — aujourd'hui ou à la date de clôture d'un exercice —, et que Kesh me propose les
rapprochements évidents entre ces lignes,
**so that** je justifie le solde du compte ligne à ligne, et que je solde ce qui doit l'être sans
chercher.

Troisième des quatre sous-stories du lettrage (#518). **Suppose 15-1a et 15-1a2 livrées** — la 15-1a
étant découpée (C124) en **15-1a-i** (la marque) et **15-1a-ii** (les gardes), les deux ; les renvois
« 15-1a Rn / ACn » gardent leur numéro (table de `15-1a-socle-lettrage.md`).
Story **backend** (dépôt, routes, rapport) ; l'écran est la 15-1c.

## Reprise du 2026-10-08 — ce qui change

La fiche d'août portait la vue seule, et se battait avec quatre décisions ouvertes nées d'un monde
où l'encaissement client **n'écrivait rien** : lecteurs de `paid_at` à réconcilier avec la marque
(Décision 1), `paid_at` sans contrepartie comptable (Décision 2), canal `paid_at` sans garde
d'exercice (Décision 3), double écriture fournisseur (Décision 4). **La 24-2 et la 15-1a2 les
dissolvent toutes** :

| décision d'août | état |
|---|---|
| D1 — la balance âgée, les relances et le rapprochement lisent `paid_at`, pas la marque | **sans objet** : ils lisent le reste dû ou `paid_at`, et la 15-1a2 garantit *lettrée `document`* ⇔ *reste dû nul* (AC5 de la 15-1a2). Ils n'ont pas à lire la marque |
| D2 — `paid_at` client sans écriture | **close** : l'encaissement écrit (24-2) ; reste l'héritage `paid_at` sans règlement, montré **ouvert** avec son motif (AC4) |
| D3 — `paid_at` sans garde d'exercice | **sans objet pour la vue**, qui ne lit pas `paid_at` ; l'immuabilité du lettrage sur exercice clos est tenue par la 15-1a (AC5) |
| D4 — deux écritures par facture fournisseur | **close** : lettrées ensemble (15-1a2 P2) |

Et l'arbitrage du 2026-08-26 — **« ouvert » = « non lettré », un point c'est tout** — est
**conservé** : il est ce qui donne l'invariant. La reprise y ajoute la **date** (C94) et fait
entrer ici le **moteur de proposition** de l'ancienne 15-1c (backend), l'écran restant seul en
15-1c (découpage, C99).

## Définitions

**Ligne ouverte à la date `X`** (`asOf`, défaut : aujourd'hui) — une ligne du compte, d'une
écriture datée **≤ X**, telle que :

- elle n'est pas lettrée, **ou**
- son groupe contient une ligne d'une écriture datée **> X** (le lettrage n'était pas acquis à
  `X`).

**Invariant** (AC2) : *la somme `Σ(débit − crédit)` des lignes ouvertes à `X` égale le solde
cumulatif du compte à `X`* — `SUM(debit − credit)` de toutes ses lignes datées ≤ X, **tous
exercices confondus** (convention du bilan, `kesh-report/src/balance_sheet.rs`). Preuve : un
groupe entièrement daté ≤ X se nette à zéro (15-1a R3) et n'est pas compté ; tout autre groupe a
ses lignes ≤ X comptées.

⚠️ **Ce n'est pas le solde de la Balance d'un exercice** (bornée par `fiscal_year_id`,
`trial_balance.rs:82` : elle ne lit que les écritures de l'exercice). Les deux chiffres peuvent
différer — et différeront pour tout compte mouvementé avant l'exercice. La 15-1c le dit à l'écran
(AC3-bis d'août, conservé) ; le développeur **constate** au sol comment l'ouverture d'exercice est
écrite et ajuste la phrase de l'écran, sans changer la définition.

**Comptes admis** : les comptes **lettrables** (15-1a R4). Un autre compte de la société → 400
`ACCOUNT_NOT_LETTERABLE` ; un compte d'une autre société → 404.

## Reçu de la 15-1a — validation P2 du socle (2026-10-09)

*Section ajoutée par la remédiation de la validation P2 de la 15-1a (registre C113, C114). Elle ne
réécrit pas cette fiche : elle liste ce que le socle a changé et que **cette** story doit intégrer
à sa propre validation.*

1. **Prérequis : la 15-12a** (clôture dans l'ordre), non plus « la 15-12 » ; ordre **15-12a → 15-12b →
   15-1a → 15-1a2 → 15-1b → 15-1c** (C112).
2. **La vue « au X » est protégée des gestes manuels sur une période close** (C113) : lettrer et
   délettrer exigent au moins une ligne « en période ouverte » — exercice ouvert, aucun exercice
   postérieur clos, date postérieure à `books_locked_through`. Pour X dans un exercice clos ou ≤ la
   borne du verrou, la liste des postes ouverts « au X » ne change donc plus par un lettrage
   **manuel**. Elle peut encore changer par une dissolution **système** (annulation d'un règlement,
   15-1a2, point 9 de son « Reçu ») : à dire dans la définition, ou à constater.
3. **Vocabulaire** : la 15-1a nomme « en période ouverte » une ligne sur laquelle le lettrage peut
   encore changer — à ne pas confondre avec la « ligne ouverte à X » de cette fiche (non lettrée, ou
   lettrée après X).
4. **État hérité** (« N ouvert, N+1 clos », sauvegarde v0.12.x) : le socle le garde pour le lettrage ;
   la vue, elle, le lit tel qu'il est.

*Ajouts de la remédiation de la validation P3 du socle (2026-10-09, registre C124, C127) :*

5. **Le socle est découpé** (C124) : R4 (comptes lettrables), la primitive et les routes sont dans la
   **15-1a-i** ; ordre **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2 → 15-1b → 15-1c**.
6. **Un compte bancaire archivé laisse son compte non lettrable** (C127, R4 de la 15-1a-i) : la
   condition « aucun `bank_accounts.journal_account_id` ne le désigne » porte sur **tous** les comptes
   bancaires, archivés compris. Le booléen `letterable` de cette fiche, calculé par la même fonction
   (`is_letterable_account`), en hérite — un compte rattaché un jour à un compte bancaire reste hors
   de la vue.

## Critères d'acceptation

**AC1 — Route des postes ouverts.**
`GET /api/v1/accounts/{id}/open-items?asOf=AAAA-MM-JJ&limit=&offset=` →

```json
{
  "accountId": 7, "accountNumber": "1100", "asOf": "2026-12-31",
  "balance": "1234.5000", "openTotal": "1234.5000", "count": 42,
  "items": [ {
    "lineId": 1, "entryId": 3, "entryNumber": 12, "fiscalYearName": "2026",
    "date": "2026-03-01", "journal": "Ventes", "description": "…",
    "debit": "100.0000", "credit": "0.0000",
    "document": { "type": "invoice", "id": 9, "number": "F-2026-0009" } | null,
    "letteringCode": null | "AB", "letteringAfterAsOf": false | true,
    "reason": "unlettered" | "partiallySettled" | "paidWithoutSettlementEntry" | "letteredAfterAsOf"
  } ]
}
```

`balance` et `openTotal` portent sur **tout** l'ensemble, pas sur la page. Pagination sur le
patron du dépôt (`MAX_LIMIT = 500`, `journal_entries.rs:541,673`) ; `limit` hors bornes écrêté.
Tri : date, puis numéro d'écriture, puis `lineId`. Rôle : Consultation et plus.

**AC2 — L'invariant est testé, et il garde toute la définition.** Base mêlée — factures client
(soldée, partielle, héritée `paid_at`, créditée), fournisseur (payée, annulée payée), écritures
manuelles lettrées et non lettrées, contre-passations, un groupe **à cheval** sur deux exercices,
un exercice **clos** — et pour **trois** dates (`asOf` avant le groupe à cheval, entre ses deux
lignes, après) : `openTotal == balance`. ⛔ **Aucun `paid_at` ni statut de facture dans la
requête** : si quelqu'un réintroduisait un filtre sur une pièce, ce test rougirait.

**AC3 — Le document.** `document` nomme la pièce qui possède l'écriture — facture, avoir,
règlement (sa facture), facture fournisseur (achat ou règlement), transaction bancaire —, par
**la même source** que `reversal_blockers` (`journal_entries.rs:1922`) : une **seule** requête de
propriété, factorisée ou réutilisée, pas une seconde liste (C-15-8-5).

**AC4 — Le motif d'une ligne ouverte** (`reason`) — ce que l'écran affichera, pour qu'une ligne
juste ne paraisse jamais fausse :

| motif | quand |
|---|---|
| `letteredAfterAsOf` | lettrée, mais par une ligne postérieure à `X` |
| `partiallySettled` | ligne d'une facture client (créance, règlement, solde) dont le reste dû est **non nul** et **différent du TTC** — partiellement réglée |
| `paidWithoutSettlementEntry` | créance d'une facture `paid_at` **sans** ligne de règlement — héritage d'avant la v0.12.0 ; le grand livre porte réellement la créance |
| `unlettered` | tout le reste |

⚠️ Le motif est **descriptif** : il ne filtre rien, il n'entre pas dans `openTotal`.

**AC5 — Propositions.** `GET /api/v1/accounts/{id}/lettering-proposals?limit=` → paires de
lignes **ouvertes aujourd'hui**, **lettrables à la main** (15-1a R5 : aucune ligne de pièce),
même compte, **sens opposés**, **montants égaux**. ⛔ Kesh **n'écrit rien** : accepter une
proposition, c'est `POST /api/v1/letterings` (15-1a), un par un.

- **Classement** : écart de dates croissant, puis `lineId` du débit, puis `lineId` du crédit —
  départage **stable**. Une paire contre-passation/origine non lettrée (rare : la 15-1a la lettre
  d'office) est rangée en tête.
- **Chaque ligne n'apparaît que dans sa meilleure paire** (appariement glouton dans l'ordre du
  classement) : l'écran ne propose jamais deux paires qui se disputent une ligne.
- **Bornes** (AC7 d'août, conservé) : au plus **2 000** lignes candidates chargées ; au-delà, 422
  `LETTERING_PROPOSALS_TOO_MANY_LINES` (« trop de lignes ouvertes sur ce compte » — **jamais** une
  troncature muette) ; sortie ≤ `MAX_LIMIT` (500), `?limit=` écrêté. L'égalité stricte rend
  l'appariement **groupable par montant** — en mémoire, linéaire, pas d'auto-jointure.
- **Pas de tolérance de montant** (Réserve 1 d'août, tranchée) : un règlement amputé de frais
  bancaires n'est pas une paire ; pour une facture client, le **solde du reste** (`write_off`,
  nature `bank_fees`) existe pour cela ; pour une ligne manuelle, une écriture d'ajustement puis
  un lettrage à trois lignes.
- **Pas de fenêtre de dates en filtre** (Réserve 2 d'août, tranchée) : elle sert au classement.
- **Pas de proposition à plus de deux lignes** dans cette story (règlement groupé manuel) : le
  lettrage manuel à N lignes reste possible à l'écran (15-1c), sans proposition. Consigné C99.

**AC6 — Le code de lettrage dans le Grand livre.** `LedgerLine`
(`kesh-report/src/general_ledger.rs:139-158`) gagne `lettering_code` (nul si ouverte) ; la requête
du grand livre lit `lettering_key`. Rien d'autre ne change au rapport.

**AC7 — Anti-IDOR et rôle.** Compte d'une autre société → 404 sur les deux routes ; même
indiscernabilité qu'un compte inexistant (`routes/products.rs:343-345`). Requête scopée par
`journal_entries.company_id` (les lignes n'ont pas de `company_id`).

**AC8 — Performances.** La requête des postes ouverts s'appuie sur
`idx_jel_account_lettering (account_id, lettering_key)` (15-1a) ; la condition « groupe ayant une
ligne > X » se calcule par **une** agrégation `GROUP BY lettering_key` (date max du groupe), pas
par sous-requête corrélée ligne à ligne. `EXPLAIN` vérifié et noté au Dev Agent Record.

**AC9 — Routes de lecture** : inscrites au registre (`audit_route_registry.rs`) si le registre
couvre les `GET` (il ne couvre que les routes **mutantes** — vérifier) ; aucune enveloppe de rejeu.

**AC10 — Documentation.** `api-external.md` : les deux routes, leurs paramètres, leurs refus
(`ACCOUNT_NOT_LETTERABLE` 400, `LETTERING_PROPOSALS_TOO_MANY_LINES` 422, 404), la définition de
« ouvert à une date » et l'invariant. Messages des deux refus dans les **quatre** locales.

**AC11 — Le caractère lettrable est exposé.** La liste des comptes (`GET /api/v1/accounts`) porte
un booléen `letterable`, calculé par **la même fonction** que la garde de la 15-1a
(`letterings::is_letterable_account`, R4) — l'écran (15-1c) n'en recopie pas la règle.

## Tasks

- [ ] **T1** (AC1–AC4, AC8) — `kesh-db` : `letterings::open_items(company, account, as_of, page)`
      et `open_items_totals` ; motif et document. Ou `kesh-report` si le rapport y a sa place — **un
      seul** endroit.
- [ ] **T2** (AC5) — Moteur de proposition : fonction **pure** en `kesh-core` (entrée : lignes
      ouvertes ; sortie : paires classées) + chargement borné en `kesh-db`.
- [ ] **T3** (AC1, AC5, AC7, AC9) — Routes dans `kesh-api` (`routes/letterings.rs` de la 15-1a).
- [ ] **T4** (AC6, AC11) — `LedgerLine.lettering_code` ; champ `letterable` des comptes.
- [ ] **T5** — Tests : AC2 en premier (l'invariant aux trois dates), AC4 (un test par motif), AC5
      (classement stable, une ligne par paire, ligne de pièce jamais proposée, ligne lettrée jamais
      proposée, plafond 2 000 → 422, `?limit=999999` écrêté), AC7.
- [ ] **T6** (AC10) — `api-external.md`, i18n (2 clés `error-*`, quatre locales).

## Dev Notes

- `asOf` est **bindé**, jamais `UTC_DATE()` en dur (patron `aged_receivables`, testabilité).
- Le moteur de proposition **ne reprend pas** `kesh-reconciliation/src/matching.rs` (score
  montant/référence/contact, sans compte ni sens) : « même compte », « sens opposés », « non
  lettrée » n'y existent pas (relevé d'août, P1-1 de la 15-1c, conservé).
- Modules : `kesh-core`, `kesh-db`, `kesh-api`, `kesh-report`, `kesh-i18n` — cinq, au seuil.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Reçu de la validation P3 du socle — 2026-10-09 (Opus 5.5, remédiation de la 15-1a)

Section « Reçu de la 15-1a » complétée des points 5 et 6 (registre C124, C127) : découpage de la 15-1a
en 15-1a-i et 15-1a-ii ; un compte bancaire archivé laisse son compte non lettrable (même fonction pour
`letterable`). Dépendance de tête mise à jour. Corps non réécrit.

### Reprise du 2026-10-08 — réécriture contre le modèle réel (Opus 5.5, en autonomie)

Corps réécrit (registre C94, C99). Les Décisions 1 à 4 d'août deviennent **sans objet ou closes**
(tableau de tête) ; l'arbitrage « ouvert = non lettré » est **conservé** et gagne une **date**
(`asOf`), ce qui rend vraie la promesse de clôture du *so that* (AC1-bis d'août). Le moteur de
proposition de l'ancienne 15-1c entre ici, avec ses réserves **tranchées** (pas de tolérance, la
date classe, plafond d'entrée explicite) ; la borne par **rôle** de compte (`singleton_role`) est
remplacée par la notion de compte **lettrable** (15-1a R4). **11 critères** (AC1–AC11), **6
tâches** (T1–T6), recomptés depuis ce fichier.

*Entrées antérieures à la reprise — le corps qu'elles décrivent a été remplacé :*


### Passe 4 de `validate` — 2026-08-26 (Sonnet, contexte frais)

**3 HIGH, 2 MEDIUM, 1 LOW.** Deux des trois HIGH sont des **conséquences de la réécriture
d'arbitrage elle-même** — le motif de l'epic, une fois de plus.

⛔ **P4-1 (HIGH) — l'invariant est VRAI, mais pas contre le solde que l'utilisateur consulte
pour clore un exercice.** Le dépôt en calcule **deux** : le **bilan** est *« cumulatif depuis
l'origine, tous exercices confondus »* (`balance_sheet.rs`), tandis que la **Balance des
comptes** et le **Grand Livre** sont bornés par `je.fiscal_year_id = ?`
(`trial_balance.rs:82`).

⚠️ **Et l'écart n'est pas théorique : c'est le prix d'un choix déjà assumé.** 15-1a AC5
autorise **explicitement** le lettrage à cheval. Une créance de 2024 lettrée avec un règlement
de 2025 se nette dans le cumul, **pas** dans la balance de 2025 — or c'est celle-là qu'on ouvre
pour clore. → AC3-bis **nomme** désormais le solde de référence (le cumulatif, convention des
tests existants), et AC4 doit **dire l'écart à l'écran** : le découvrir en production, ce serait
voir deux écrans se contredire sans explication.

⛔ **P4-2 (HIGH) — T4 n'avait PAS été réécrite, et prescrivait encore l'ancienne définition.**
L'arbitrage avait réécrit les critères et T1, **et laissé T4** : elle réclamait toujours « les
deux tables », « les deux écritures », une fixture « une payée et une NON payée », et un
critère `AC2-bis` **qui n'existe plus**. Un développeur la suivant à la lettre aurait écrit des
tests validant **exactement ce qu'AC2 interdit**. → T4 entièrement réécrite autour du test de
l'invariant, **sans un seul `paid_at`**.

⛔ **P4-3 (HIGH) — AC4, déclaré deux fois « le critère le plus important de la fiche », n'avait
ni tâche ni test** — zéro occurrence dans la section Tasks, là où la fiche sœur écrit « Écran
dédié, avec la frontière énoncée (AC5) ». Et son « chemin vers le lettrage » **suppose 15-1c** :
le lettrage manuel est son AC1, et T3 est ici un écran de **consultation**. → T3 porte AC4
nommément, et la question est posée : lien vers 15-1c, ou renvoi assumé.

**Deux MEDIUM** : le piège du `status = 'validated'` tronqué était classé « sans objet », alors
qu'il reste **entier** pour la Décision 1, non résolue ; et les identifiants cités par AC3
désignaient **des findings du SOCLE**, avec un total que leur énumération ne portait pas — la
§ *Recompter ses propres comptes rendus*, encore.

**Un LOW** : la Décision 4 était rendue caduque sans que rien ne le signale, contrairement à la
Décision 2 close explicitement.

**Réfuté** : l'invariant n'est **pas** mathématiquement faux — il est vrai sous la lecture
cumulative, qui est celle du bilan et des tests du dépôt. Le défaut était son **périmètre non
dit**, pas sa vérité.

**Verdict : passe 5 due**, et les deux décisions (1 et 3) restent ouvertes.

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
