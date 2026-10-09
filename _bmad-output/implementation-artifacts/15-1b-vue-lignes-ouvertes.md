# Story 15.1b : Les postes ouverts d'un compte, à une date — et les rapprochements que Kesh propose

## Status

ready-for-dev **après la livraison de la 15-1a2-i, de la 15-1a2-ii et de la 15-1b-0** *(réécrite le
2026-10-08 ; validation P1 remédiée le 2026-10-09 ; validation P2 remédiée le 2026-10-09 — la refonte de
la propriété des lignes **extraite** en 15-1b-0, C-15-1b-9 ; **validation P3 à mener avant tout
développement**)*.

⛔ **Ce qui dépend de la 15-1a2, non livrée** (sur `056997b0`, la dernière migration est
`20261009000001_journal_entry_lines_lettering.sql` — la marque et ses gardes seulement), découpée en
**15-1a2-i** (pièces clientes) et **15-1a2-ii** (fournisseurs et rattrapage) : la fixture d'AC2 en tant
qu'elle contient des factures client **soldées** lettrées `document` (15-1a2-i) et des factures
fournisseurs **payées** et **annulées payées** lettrées (15-1a2-ii) ; le test de `documentState =
nothingDue` sur une ligne lettrée après `X` et sur une pièce historique close restée ouverte ; l'origine
`document` dans `letteringOrigin` ; le refus de l'annulation sous verrou qu'AC12 exerce ; le prédicat des
périodes `open_period_rule` (15-1a2-i P7). ⛔ **Ce qui dépend de la 15-1b-0** : `document_owners`, la
propriété des lignes par lot (AC3). **Tout le reste** — requête A, invariant sur groupes `manual` et
`reversal`, moteur de propositions, Grand livre, `letterable` — se développerait sans elles ; l'ordre de
l'epic (C112, C124) les fait passer avant : **15-1a2-i → 15-1a2-ii → 15-1b-0 → 15-1b**.

## Story

**As a** indépendant, PME ou fiduciaire,
**I want** obtenir, pour un compte de créances, de dettes ou de passage, la liste de ce qui y
reste ouvert — aujourd'hui ou à la date de clôture d'un exercice —, et que Kesh me propose les
rapprochements évidents entre ces lignes,
**so that** je justifie le solde du compte ligne à ligne, et que je solde ce qui doit l'être sans
chercher.

Troisième des quatre sous-stories du lettrage (#518). **Suppose 15-1a, 15-1a2 et 15-1b-0 livrées** — la
15-1a étant découpée (C124) en **15-1a-i** (la marque) et **15-1a-ii** (les gardes), la 15-1a2
(C-15-1a2-1) en **15-1a2-i** et **15-1a2-ii**, toutes ; les renvois « 15-1a Rn / ACn » et « 15-1a2 Pn /
ACn » gardent leur numéro (tables de `15-1a-socle-lettrage.md` et `15-1a2-lettrage-des-pieces.md`).
Story **backend** (dépôt, routes, rapport) ; l'écran est la 15-1c.

## Reprise du 2026-10-08 — ce qui change

La fiche d'août portait la vue seule, et se battait avec quatre décisions ouvertes nées d'un monde
où l'encaissement client **n'écrivait rien** : lecteurs de `paid_at` à réconcilier avec la marque
(Décision 1), `paid_at` sans contrepartie comptable (Décision 2), canal `paid_at` sans garde
d'exercice (Décision 3), double écriture fournisseur (Décision 4). **La 24-2 et la 15-1a2 les
dissolvent toutes** :

| décision d'août | état |
|---|---|
| D1 — la balance âgée, les relances et le rapprochement lisent `paid_at`, pas la marque | **sans objet** : ils lisent le reste dû ou `paid_at`, et la 15-1a2-i garantit *lettrée `document`* ⇔ *reste dû nul* **pour toute facture dont une ligne est en période ouverte**, hors deux exceptions nommées — avoir hérité crédité sur un autre compte, compte de créance non lettrable (AC5 de la 15-1a2-i). Hors de ce périmètre, un seul cas atteignable : la facture **soldée non lettrée** d'une pièce historique entièrement close (15-1a2-i P7 point 1) — la vue la montre ouverte, `documentState = nothingDue`, `manuallyLetterable = false` (test 10). Le compte de créance non lettrable n'est **pas** montrable : la vue répond 409 (AC7) ; il se lit au grand livre. Le cas « facture due, ligne de vente lettrée » de la version P1 de la 15-1a2 n'existe plus (refus, 15-1a2-i P7 point 2). Ils n'ont pas à lire la marque |
| D2 — `paid_at` client sans écriture | **close** : l'encaissement écrit (24-2) ; reste l'héritage `paid_at` sans règlement, montré **ouvert** avec son motif (AC4) |
| D3 — `paid_at` sans garde d'exercice | **sans objet pour la vue**, qui ne lit pas `paid_at` ; l'immuabilité du lettrage sur exercice clos est tenue par la 15-1a (AC5) |
| D4 — deux écritures par facture fournisseur | **close** : lettrées ensemble (15-1a2-ii P2) |

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
exercices confondus**. Preuve : un groupe entièrement daté ≤ X se nette à zéro (15-1a R3) et n'est
pas compté ; tout autre groupe a ses lignes ≤ X comptées.

✅ **Ce solde EST celui de la Balance et du bilan** *(corrigé à la validation P1, F-1 — la
rédaction précédente affirmait le contraire)*. Un compte lettrable est un compte de **bilan**
(`Asset` ou `Liability`, 15-1a R4), et pour un compte de bilan les trois rapports partent du cumul
depuis l'origine : la Balance (`kesh-report/src/trial_balance.rs:41-42` — « l'ouverture [filtre]
la date seule » ; SQL `before_debit = SUM(CASE WHEN x.entry_date < ?)` sans filtre d'exercice), le
Grand livre (`general_ledger.rs`, même `opening::opening_from`) et le bilan
(`balance_sheet.rs:8-11`). `opening_from` (`opening.rs:75-86`) prend `before_start` pour tout
compte de bilan. Donc, pour `X` = fin d'un exercice, `balance` = **clôture de la Balance de cet
exercice** pour ce compte, au signe près : la Balance donne le solde du côté naturel (crédit
positif pour un `Liability`), cette route `débit − crédit` — `balance ==
opening::debit_sense(account_type, row.closing_balance)`. AC2 l'assert. Seul un compte de
**résultat** repart de zéro à chaque exercice, et il n'est pas lettrable : la mise en garde
« différera de la Balance » est **sans objet** et quitte la fiche (répercussion sur la 15-1c, voir
« Pour la 15-1c »).

**« Au X » n'est pas un instantané** *(validation P1, F-8 ; réécrit à la validation P2, R-1 = F-1 HIGH,
F-2)*. La vue se calcule toujours sur les **marques d'aujourd'hui** : elle dit ce qui, des lignes datées
≤ X, n'est pas soldé par un groupe entièrement daté ≤ X **selon le lettrage actuel**. Conséquences, à
écrire telles quelles dans `api-external.md` :

- **stable pour un `X` en période close** — `X` ≤ `books_locked_through`, ou `X` dans un exercice
  clôturé ou suivi d'un exercice clôturé —, **tant qu'aucun administrateur ne déverrouille la période
  (`POST /companies/current/books-lock/release`, `unlock_company_books`), ne rouvre un exercice
  (`POST /fiscal-years/{id}/reopen`, `reopen_fiscal_year`) ni ne restaure une sauvegarde** — trois gestes
  d'administrateur, tracés au journal d'audit. Aucune ligne datée ≤ X n'entre ni ne sort alors d'un groupe
  entièrement daté ≤ X : le lettrage et le délettrage manuels exigent une ligne en période ouverte (15-1a
  R7, C113) ; la synchronisation des pièces s'abstient de lettrer une pièce entièrement close (15-1a2-i P7
  point 1) ; le geste qui devrait délettrer un groupe entièrement clos — annulation d'un règlement, d'un
  solde, d'un rapprochement, d'un paiement ou d'une facture fournisseur — est **refusé** (15-1a2-i P7
  point 2, `409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) ; tout groupe qu'un geste crée contient une ligne
  en période ouverte, donc datée après `X` ; et une écriture ne se crée ni ne disparaît dans une période
  close. ⚠️ Tolérance nommée par la 15-1a2-i (P7 point 2) : un verrou posé **pendant** une annulation ;
- **non stable au-dessus** — `X` après la borne, dans un exercice ouvert sans exercice postérieur
  clôturé : par exemple, le délettrage manuel d'un groupe entièrement daté ≤ X (permis, une de ses
  lignes étant en période ouverte), ou l'annulation d'un règlement dont la créance et la ligne de
  règlement sont datées ≤ X (dissolution du groupe `document`, contre-passation datée du jour, > X), fait
  **réapparaître** ces lignes ouvertes « au X ». L'invariant tient toujours ; c'est la **liste** qui
  change. Et sous un `X` en période close, l'un des trois gestes d'administrateur ci-dessus rend la
  liste de nouveau mobile. Test : AC12.

**Comptes admis** : les comptes **lettrables** (15-1a R4). Un autre compte de la société → **409
`LETTERING_ACCOUNT_NOT_LETTERABLE`** (refus livré par la 15-1a-i : `DbError::LetteringAccountNotLetterable`,
`kesh-api/src/errors.rs:2995`, clé `error-lettering-account-not-letterable` présente dans les quatre
locales — **réutilisé**, aucun second code ; C-15-1b-1). Un compte d'une autre société, ou
inexistant → 404, indiscernables. ⚠️ **Un compte devenu non lettrable** (retypé en charge ou produit,
ou rattaché à un compte bancaire, C104) **reçoit le même 409**, même s'il porte encore des groupes :
ses groupes restent consultables par `GET /letterings/{key}` et dissolubles (C104) ; la vue des
postes ouverts, elle, ne sert que les comptes où l'on peut lettrer (C-15-1b-1).

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
   **manuel**. *(Validation P2 : elle ne change plus non plus par une dissolution **système** — la
   15-1a2-i refuse l'annulation qui l'exigerait, P7 point 2 ; voir point 10.)*
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

*Ajouts de la validation P1 de cette fiche (2026-10-09) — le socle **livré** (15-1a-i #587, 15-1a-ii
#593, base `056997b0`), noms à employer tels quels :*

7. **Lettrabilité** : `letterings::letterable_account(conn, company, account) -> Option<(bool, String)>`
   (`letterings.rs:296`) et son raccourci `is_letterable_account` (`:319`) — une requête **par
   compte**. Refus : `DbError::LetteringAccountNotLetterable` → 409. Index : `idx_jel_account_lettering
   (account_id, lettering_key)` et `idx_jel_lettering (lettering_key)` (migration
   `20261009000001`, l. 33-34). Code affiché : `letterings::code_of(key)` (`:268`). Origine :
   `letterings::Origin` (`document`, `reversal`, `manual`).
8. **R5** (ligne de pièce) : `first_document_owner` (`letterings.rs:511`, privée) appelle
   `journal_entries::reversal_blockers` (`journal_entries.rs:2090`) **une écriture à la fois** et
   retient quatre motifs — `OwnedByInvoice`, `OwnedByCreditNote`, `OwnedBySupplierInvoice`,
   `OwnedBySettlement`. `MatchedBankTransaction` **n'en est pas** : une ligne rapprochée d'une
   transaction bancaire reste lettrable à la main.
9. **R7** (règle des périodes) : `any_line_in_open_period` (`letterings.rs:494`, privée) — exercice
   `Open`, aucun exercice postérieur `Closed`, date > `books_locked_through`.
10. **Le point 2 est tranché** *(réécrit à la validation P2)* : la liste « au X » est stable pour un
    `X` en période close — exercice clos, ou ≤ `books_locked_through` — tant qu'aucun administrateur ne
    déverrouille, ne rouvre ni ne restaure ; non stable au-dessus — écrit dans « Définitions » et testé
    (AC12). La version P1 de ce point disait « non stable sous le verrou » : elle supposait une
    dissolution système que la 15-1a2-i refuse.

*Ajouts de la validation P2 de cette fiche (2026-10-09) — les fiches 15-1a2-i, 15-1a2-ii et 15-1b-0,
remédiées dans la même passe :*

11. **La propriété par lot est la 15-1b-0** (`document_owners(conn: &mut MySqlConnection, company_id,
    &[entry_id]) -> BTreeMap<i64, Vec<DocumentOwner>>`, `DocumentKind`, `DocumentOwner { kind, id, number,
    invoice_id, invoice_number }`) ; `reversal_blockers` et `first_document_owner` y sont réécrits. Cette
    fiche **consomme** ; elle ne touche plus `journal_entries.rs`.
12. **Le prédicat des périodes est celui de la 15-1a2-i** (P7 : `open_period_rule(conn, company_id,
    &fiscal_year_ids) -> Result<OpenPeriodRule, DbError>`, `OpenPeriodRule::line_in_open_period(fiscal_year_id,
    entry_date)`), public, lu sans verrou : il sert `inOpenPeriod` et le filtre des propositions. **Aucune
    seconde factorisation** en `kesh-core` (C-15-1b-4 révisée).
13. **Le refus du rang 2 bis** (15-1a2-i P7 point 2) : une annulation qui délettrerait un groupe `document`
    entièrement clos est refusée. C'est ce qui rend la liste stable sous la borne (point 10).

## Critères d'acceptation

**AC1 — Route des postes ouverts.**
`GET /api/v1/accounts/{id}/open-items?asOf=AAAA-MM-JJ&limit=&offset=` →

```json
{
  "accountId": 7, "accountNumber": "1100", "asOf": "2026-12-31",
  "balance": "1234.5000", "openTotal": "1234.5000",
  "total": 42, "offset": 0, "limit": 50,
  "items": [ {
    "lineId": 1, "entryId": 3, "entryNumber": 12, "fiscalYearName": "2026",
    "date": "2026-03-01", "journal": "Ventes", "description": "…",
    "debit": "100.0000", "credit": "0.0000",
    "document": null | { "type": "invoice", "id": 9, "number": "F-2026-0009",
                         "invoiceId": null, "invoiceNumber": null },
    "letteringCode": null | "AB", "letteringOrigin": null | "document" | "reversal" | "manual",
    "letteredOn": null | "2027-01-15",
    "reason": "unlettered" | "letteredAfterAsOf",
    "documentState": null | "unpaid" | "partiallySettled" | "nothingDue" | "paidWithoutSettlementEntry",
    "amountDue": null | "40.0000",
    "manuallyLetterable": true | false,
    "inOpenPeriod": true | false
  } ]
}
```

- **Entrées** (C-15-1b-5) : `asOf` lu en chaîne et parsé comme `dateFrom` de la liste des écritures
  (`routes/journal_entries.rs:328-332`) — mal formé → **400 `VALIDATION_ERROR`** ; absent → la date
  du jour **selon la convention existante** `Utc::now().naive_utc().date()` (`routes/reports.rs:557`,
  balance âgée — l'écart UTC/heure suisse la nuit est hérité, non corrigé ici) ; **aucune borne** :
  un `X` futur rend aussi les écritures datées dans le futur, un `X` antérieur à toute écriture rend
  une liste vide et `balance = 0`. `limit` : défaut **50**, écrêté `clamp(1, 500)` ; `offset` :
  défaut 0, `max(0)` — patron `routes/journal_entries.rs:40-41, 324-325`. La réponse **renvoie**
  `total`, `offset` et `limit` effectifs, comme `ListResponse` (`routes/mod.rs:60`).
- `balance`, `openTotal` et `total` portent sur **tout** l'ensemble, pas sur la page ; les trois et
  la page sont lus dans **une** transaction (`REPEATABLE READ`, vue unique) pour qu'un écrivain
  concurrent ne les désaccorde pas (F-8, LOW).
- Tri : date, puis numéro d'écriture, puis `lineId`. Rôle : Consultation et plus ; une clé d'API
  en lecture l'atteint (`api-external.md` § 7).
- `letteringCode`, `letteringOrigin`, `letteredOn` décrivent le lettrage **d'aujourd'hui** (nuls si
  la ligne est ouverte aujourd'hui). `letteredOn` = date de la ligne la plus tardive du groupe — la
  date à laquelle le lettrage est acquis ; non nul **et** > `asOf` ⇔ `reason = letteredAfterAsOf`.

**AC2 — L'invariant est testé, contre les rapports.** Base mêlée — factures client (soldée,
partielle, héritée `paid_at`, créditée), fournisseur (payée, annulée payée), écritures manuelles
lettrées et non lettrées, contre-passations, un groupe **à cheval** sur deux exercices, un exercice
**clos** — et pour **trois** dates (`asOf` avant le groupe à cheval, entre ses deux lignes, après) :

1. `openTotal == balance` ;
2. `balance == Σ(debit − credit)` des lignes du compte datées ≤ X, calculé dans le test ;
3. pour `asOf` = **fin de chaque exercice** de la fixture : `balance ==
   opening::debit_sense(account_type, closing_balance)` de la ligne du compte dans
   `trial_balance::generate` sur cet exercice (F-1) — l'accord avec le rapport que l'utilisateur
   ouvre pour clore.

⛔ **La sélection ne lit aucune pièce.** La requête de sélection (requête **A** de T1 : lignes,
`balance`, `openTotal`, `total`, page) ne joint ni `invoices`, ni `credit_notes`, ni
`supplier_invoices`, ni `invoice_settlements`, ni `bank_transactions`, et ne lit pas `paid_at` ;
l'enrichissement (requête **B** : `document`, `documentState`, `amountDue`, `inOpenPeriod`) ne
porte que sur les écritures de la **page**. Garanti par **deux** tests : l'invariant ci-dessus (en
valeur) et un test **lexical** sur la constante SQL de la requête A (aucune des cinq tables, pas de
`paid_at`) — un filtre sur une pièce réintroduit dans A rougirait l'un ou l'autre.

**AC3 — Le document, par une seule source de propriété, en lot.** `document` nomme la pièce qui
possède l'écriture. Il vient de `journal_entries::document_owners(conn, company_id, &[entry_id])`,
**livrée par la 15-1b-0** (C-15-1b-9) — la source unique des motifs de propriété, sur laquelle
`reversal_blockers` et `first_document_owner` sont réécrits, et dont la parité avec le comportement
d'avant est prouvée **là**, contre un oracle indépendant. Cette story l'**appelle** pour les écritures de
la page (requête B) et pour le filtre R5 des propositions ; elle ne la modifie pas.

- **Liste fermée de `document.type`**, dans l'ordre de précédence de `reversal_blockers` (rangs 3 à
  7) : `invoice`, `creditNote`, `supplierInvoice`, `settlement`, `bankTransaction`. Une écriture qui
  en porte plusieurs (règlement encaissé par rapprochement : `settlement` **et** `bankTransaction`)
  montre la **première**.
- `number` : numéro de la pièce quand elle en a un (facture, avoir, facture fournisseur), sinon
  `null`. Pour `settlement`, `invoiceId` et `invoiceNumber` nomment **la facture réglée**
  (`DocumentOwner::invoice_id` / `invoice_number`, 15-1b-0) ; nuls pour les autres types. Sérialisation :
  `DocumentKind` → `invoice`, `creditNote`, `supplierInvoice`, `settlement`, `bankTransaction`.
- `manuallyLetterable` = la ligne est **ouverte aujourd'hui** et son écriture n'a **aucun** des
  quatre motifs de R5 (`invoice`, `creditNote`, `supplierInvoice`, `settlement`) — calculé par le
  même lot que `first_document_owner`, jamais recopié. ⚠️ `bankTransaction` seul **n'ôte pas** la
  lettrabilité (Reçu, point 8) : `document != null` n'équivaut donc pas à « ligne de pièce ».
- `inOpenPeriod` = la ligne est « en période ouverte » au sens de R7 (exercice `Open`, aucun
  postérieur `Closed`, date > `books_locked_through`), lu **sans verrou** — indicatif ; par
  `OpenPeriodRule::line_in_open_period`, chargé **une fois** par `open_period_rule` pour les exercices
  distincts de la page (15-1a2-i P7 — le prédicat même du mode `Manual`, factorisé une seule fois ;
  validation P2, R-3 = L-2).

**AC4 — Pourquoi une ligne est ouverte, et où en est sa pièce** — deux champs, deux dates de
référence, aucune précédence implicite (C-15-1b-3) :

| champ | date de référence | valeurs |
|---|---|---|
| `reason` | **`X`** (le grand livre) | `letteredAfterAsOf` si la ligne est lettrée et `letteredOn > X` ; `unlettered` sinon |
| `documentState` | **aujourd'hui** (la pièce) | pour une ligne dont `document` est une facture client (`invoice`, ou `settlement` → sa facture) ; `null` pour tout autre — **y compris une ligne d'avoir** (`creditNote`), choix écrit (C-15-1b-12) : un avoir émis est total et unique, il n'a pas d'état de règlement propre ; sa facture se lit sur la ligne de vente (validation P2, R L-7 = F L-4) |

Valeurs de `documentState`, **dans cet ordre de précédence** (première qui s'applique) :

1. `paidWithoutSettlementEntry` — `paid_at` posé et **aucune** ligne `invoice_settlements` :
   héritage d'avant la v0.12.0, le grand livre porte réellement la créance ;
2. `nothingDue` — reste dû ≤ 0 (réglée, créditée, ou soldée) ;
3. `partiallySettled` — au moins un règlement et reste dû > 0 ;
4. `unpaid` — aucun règlement, reste dû > 0.

`amountDue` = le reste dû **d'aujourd'hui** de cette facture, par
`invoice_settlements::amount_due_derived_joins()` et `INVOICE_AMOUNT_DUE_DERIVED_SQL`
(`invoice_settlements.rs:108, 120`) — réutilisés, **jamais réécrits** (#416) —, **arrondi au centime par
`invoice_settlements::amount_due_to_centime`** (`:129-140`, « la seule définition » de tout affichage de ce
reste, Story 25-4-c3-b) ; `null` hors facture client. Les seuils de `documentState` (`≤ 0` / `> 0`) se
lisent **sur cette valeur au centime** : un reste brut de 0,004 n'est pas `partiallySettled` (#490 ;
validation P2, F L-1 ; C-15-1b-12). `GET /invoices` expose le brut : écart connu, hors de cette story.

⚠️ Les deux champs sont **descriptifs** : ils ne filtrent rien et n'entrent pas dans `openTotal`.
⚠️ Pour un `X` passé, `documentState` peut décrire un règlement **postérieur** à `X` : c'est voulu
(l'état actuel de la pièce, pour savoir quoi faire), et `api-external.md` le dit. Le motif **à `X`**
est `reason`.

**AC5 — Propositions.** `GET /api/v1/accounts/{id}/lettering-proposals?limit=` → paires de lignes
du compte qui sont, **aujourd'hui** :

- **ouvertes** (non lettrées) ;
- **lettrables à la main** : `manuallyLetterable` (AC3) — même lot, même liste de quatre motifs ;
- de **sens opposés** (une au débit, une au crédit) et de **montants égaux** ;
- **acceptables** : au moins une des deux lignes est en période ouverte (R7, `inOpenPeriod`) — une
  paire tout entière en période close serait refusée au clic (`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`),
  elle n'est donc **pas proposée** (F-5, C-15-1b-4). Lecture sans verrou : une borne posée entre la
  proposition et le clic reste refusée par la 15-1a, et la 15-1c affiche ce refus par son message.

⛔ Kesh **n'écrit rien** : accepter une proposition, c'est `POST /api/v1/letterings` (15-1a), un par un.

- **Classement** : les paires contre-passation/origine d'abord (l'écriture de l'une a pour
  `reverses_entry_id` celle de l'autre — cas d'un groupe `reversal` délettré à la main ; d'une ligne que
  la contre-passation n'a pas lettrée d'office parce qu'elle l'était déjà, **et dont le groupe a été
  dissous depuis** — sans quoi l'origine reste lettrée, hors candidates ; ou d'un compte redevenu
  lettrable après la contre-passation, R6 sautant un compte non lettrable — validation P2, F L-6), puis écart de dates
  croissant, puis `lineId` du débit, puis `lineId` du crédit — départage **stable**.
- **Chaque ligne n'apparaît que dans sa meilleure paire** (appariement glouton dans l'ordre du
  classement).
- **Bornes** : le plafond de **2 000** lignes porte sur les lignes **candidates après** les filtres
  « ouverte » et R5 (F-6/R4 : un compte clients de 3 000 lignes de factures n'a aucune candidate et
  ne rend pas 422) ; au-delà → **422 `LETTERING_PROPOSALS_TOO_MANY_LINES`** (« trop de lignes
  ouvertes sur ce compte » — **jamais** une troncature muette). La variante porte le plafond —
  `DbError::LetteringProposalsTooManyLines { max: usize }`, patron « le plafond voyage dans la variante »
  de `LetteringTooManyLines` (`kesh-db/src/errors.rs:1102`, `kesh-api/src/errors.rs:2975`) —, le message
  l'interpole, et le repli Rust du `t(...)` dit **mot pour mot** le FTL fr-CH (validation P2, R L-4). Le
  filtre R5 passe par le lot d'AC3 découpé par 500 écritures : son coût est linéaire en lignes ouvertes,
  **sans plafond propre** — un compte de 50 000 lignes ouvertes de pièces charge 100 lots avant de rendre
  `candidateCount = 0` ; borné, linéaire, mesuré au test 13 et noté au Dev Agent Record (validation P2,
  F L-10). `limit` : défaut **100**, écrêté `clamp(1, 500)`.
- **Coût du moteur** : groupement par montant puis, dans chaque groupe, toutes les paires débit ×
  crédit classées — `O(Σ n_k·m_k)`, borné par le plafond (≤ 10⁶ paires au pire, 1 000 × 1 000 d'un
  même montant). Pas d'auto-jointure SQL. Test de volume au plafond (T6).
- **Pas de tolérance de montant** (Réserve 1 d'août, tranchée) : un règlement amputé de frais
  bancaires n'est pas une paire ; pour une facture client, le **solde du reste** (`write_off`,
  nature `bank_fees`) existe pour cela ; pour une ligne manuelle, une écriture d'ajustement puis un
  lettrage à trois lignes.
- **Pas de fenêtre de dates en filtre** (Réserve 2 d'août, tranchée) : elle sert au classement.
- **Pas de proposition à plus de deux lignes** dans cette story : le lettrage manuel à N lignes
  reste possible à l'écran (15-1c), sans proposition. Consigné C99.

Réponse (C-15-1b-4) :

```json
{
  "accountId": 7, "candidateCount": 340, "total": 12, "limit": 100,
  "items": [ {
    "amount": "100.0000", "daysApart": 3, "reversalPair": false,
    "debit":  { "lineId": 1, "entryId": 3, "entryNumber": 12, "fiscalYearName": "2026",
                "date": "2026-03-01", "journal": "Ventes", "description": "…", "document": null,
                "inOpenPeriod": true },
    "credit": { "lineId": 8, "entryId": 5, "entryNumber": 15, "fiscalYearName": "2026",
                "date": "2026-03-04", "journal": "Banque", "description": "…", "document": null,
                "inOpenPeriod": true }
  } ]
}
```

`total` = nombre de paires retenues avant `limit` ; `candidateCount` = lignes candidates (≤ 2 000).
Le sous-objet d'une ligne est un **sous-ensemble** de l'item d'AC1, même sérialisation ; il porte
`fiscalYearName`, une paire pouvant chevaucher deux exercices où `entryNumber` est ambigu
(`general_ledger.rs:143-149` ; validation P2, F L-5).

**AC6 — Le code de lettrage dans le Grand livre.** `LedgerLine`
(`kesh-report/src/general_ledger.rs:139-158`) gagne `lettering_code: Option<String>` (`null` si
ouverte), par `letterings::code_of` ; la requête du grand livre lit `lettering_key`. Le **JSON**
seul change (clé `letteringCode`) : l'export **CSV** (`csv.rs:675`, colonnes écrites une à une) et
le **PDF** (`pdf.rs:1579`) restent tels quels (C-15-1b-6) ; leurs constructeurs littéraux de test
(`csv.rs:1303`, `pdf.rs:2447`) reçoivent le champ. Le type TypeScript du Grand livre
(`reports.types.ts`) est laissé à la 15-1c, qui affiche le code.

**AC7 — Anti-IDOR, rôle, compte non lettrable.** Compte d'une autre société → 404 sur les deux
routes ; même indiscernabilité qu'un compte inexistant (patron `accounts::find_by_id_in_company`,
`routes/accounts.rs:265`). Requête scopée par `journal_entries.company_id` (les lignes n'ont pas
de `company_id`). Compte de la société non lettrable — y compris **devenu** non lettrable (C104) —
→ 409 `LETTERING_ACCOUNT_NOT_LETTERABLE` sur les deux routes. **Ordre des refus**, écrit (validation P2,
F L-9 ; C-15-1b-12) : (1) les paramètres — `limit`/`offset` non numériques rejetés par l'extracteur
`Query` d'Axum (sa réponse propre, pas `VALIDATION_ERROR`), puis `asOf` mal formé → 400
`VALIDATION_ERROR`, **avant** toute lecture ; (2) le compte — 404 ; (3) sa lettrabilité — 409. Un `asOf`
mal formé sur le compte d'une autre société rend donc 400 : aucune information ne fuit, le refus ne
dépend que de la requête.

**AC8 — Performances.** La requête A s'appuie sur `idx_jel_account_lettering (account_id,
lettering_key)` ; la condition « groupe ayant une ligne > X » se calcule par **une** table dérivée
`GROUP BY lettering_key` (date max du groupe = `letteredOn`) jointe en `LEFT JOIN`, pas par
sous-requête corrélée ligne à ligne. `EXPLAIN` vérifié et noté au Dev Agent Record, avec la forme
de la condition `g.lettered_on IS NULL OR g.lettered_on > ?` (un `OR` peut dégrader le plan : à
mesurer, non à supposer).

**AC9 — Routes de lecture : rien au registre.** `audit_route_registry.rs` ne balaie que `post`,
`put`, `delete`, `patch` (`crates/kesh-api/tests/audit_route_registry.rs:122`, point (v)) : les deux `GET` n'y entrent pas, et n'ont aucune
enveloppe de rejeu. Aucun test à ajouter.

**AC10 — Documentation.** `docs/api-external.md` : les deux routes (paramètres, défauts, bornes,
réponses), leurs refus **dans l'ordre d'AC7** (`VALIDATION_ERROR` 400, 404, `LETTERING_ACCOUNT_NOT_LETTERABLE`
409, `LETTERING_PROPOSALS_TOO_MANY_LINES` 422), la définition de « ouvert à une date », l'invariant,
**« au X n'est pas un instantané »** (stable en période close **tant qu'aucun administrateur ne
déverrouille, ne rouvre ni ne restaure**), les deux dates de référence d'AC4, une ligne au tableau des
ressources (§ 7, `api-external.md:205-215`), le champ `letterable` de `GET /accounts` et
`letteringCode` du Grand livre. `CHANGELOG.md`, `[0.13.0]` : entrée « Ajouté » pour les deux routes
et, en « Modifié », un **changement de contrat additif** (`letterable`, `letteringCode` du Grand
livre), au même titre que les lignes d'écriture (`CHANGELOG.md:23`). i18n : **une** clé neuve,
`error-lettering-proposals-too-many-lines`, dans les **quatre** locales (le refus 409 est livré).
**Manuel inchangé**, écrit pour que la passe suivante ne le relève pas (validation P2, F L-9) : vérifié
au `.tex` et au PDF aplati, rien n'y devient faux par cette story (glossaire `user-manual.tex:2420-2426` :
« le lettrage manuel et le délettrage se font par l'API ») ; l'écran et son manuel sont la 15-1c.

**AC11 — Le caractère lettrable est exposé, par une seule règle.** `AccountResponse` porte
`letterable: bool` dans les **cinq** réponses qui le construisent (`list_accounts`,
`create_account`, `update_account`, `archive_account`, `reactivate_account` —
`routes/accounts.rs:171-356`), archivés compris. Une seule règle (C-15-1b-7) :

- un prédicat **pur** `letterings::is_letterable(account_type: &str, bank_linked: bool) -> bool` porte R4
  — le type tel que la base le lit (`letterable_account` lit `account_type` en `String`,
  `letterings.rs:301-313`) ; l'entité `Account` passe `account.account_type.as_str()` (validation P2,
  R L-5) ;
- `letterable_account` (une ligne) l'appelle ;
- une fonction **en lot** `letterings::letterable_account_ids(conn, company_id) -> BTreeSet<i64>`
  — **une** requête, même expression SQL `EXISTS (SELECT 1 FROM bank_accounts b WHERE
  b.journal_account_id = a.id)` partagée en constante — l'appelle aussi, et sert `list_accounts`
  (aucun N+1 sur le plan comptable) ;
- les quatre réponses unitaires appellent `is_letterable_account` (une requête).

`From<Account>` ne suffit plus : un constructeur `AccountResponse::new(account, letterable)`.
Le type TypeScript `AccountResponse` (`frontend/src/lib/features/accounts/accounts.types.ts:45` ;
validation P2, F L-8) est laissé à la 15-1c (qui en a l'usage).

**AC12 — « Au X » : ce qui est stable et ce qui ne l'est pas, testé** *(réécrit à la validation P2 :
R-1 = F-1 HIGH, F-2, F L-3)*. La stabilité se prouve par des gestes **qui réussissent** — un geste refusé
laisse la liste inchangée trivialement ; et `openTotal == balance` est asserté avant et après chaque geste.
(a) **`X` dans un exercice clos** : des gestes réussis touchent des lignes ≤ X — lettrage manuel d'une
ligne ≤ X avec une ligne > X (R7 l'admet, l'une étant en période ouverte), règlement en N+1 d'une facture
de l'exercice clos N dont la créance est ≤ X (la synchronisation la lettre avec un règlement > X) — et la
liste « au X » est
**identique** avant et après (lignes, `openTotal`) ; seul le `reason` des lignes lettrées par ces gestes
passe à `letteredAfterAsOf`, sans qu'elles quittent la liste. (b) **`X` ≤ `books_locked_through`,
exercice ouvert** : l'annulation d'un règlement dont toutes les lignes du groupe `document` sont ≤ borne
est **refusée** (`409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, 15-1a2-i P7 point 2) et la liste « au X »
est **identique** ; puis un administrateur **déverrouille** (`POST /companies/current/books-lock/release`,
motif) : la même annulation **réussit**, et la créance et la ligne de règlement **réapparaissent** « au X »
— la stabilité tient tant qu'aucun administrateur ne déverrouille, et cesse quand il le fait. (c) **`X`
au-dessus de la borne, exercice ouvert** : le délettrage manuel d'un groupe entièrement ≤ X fait
réapparaître ses lignes « au X ». *(Le (b) suppose la 15-1a2-i — groupe `document` et refus du rang 2 bis ;
voir Status.)*

## Tasks

- [ ] **T1** (AC1, AC2, AC4, AC8) — `kesh-db`, `repositories/letterings.rs` (tranché : **pas**
      `kesh-report`, qui ne voit ni la lettrabilité ni la propriété — C-15-1b-8) :
      `open_items(conn, company, account, as_of, limit, offset)` en **une transaction** : requête A
      (constante SQL, sans pièce : page, `balance`, `openTotal`, `total`, `letteredOn`), puis
      requête B pour les écritures de la page (lot d'AC3, reste dû, état d'exercice).
- ~~**T2**~~ — **déplacée à la 15-1b-0** (C-15-1b-9) : `document_owners`, `reversal_blockers` et
      `first_document_owner` réécrits dessus, parité par oracle indépendant, mutations éprouvées. Le
      numéro est gardé pour que les renvois restent justes.
- [ ] **T3** (AC5, AC3) — `kesh-core::lettering` : moteur **pur** (entrée : lignes candidates avec
      `line_id`, `entry_id`, `reverses_entry_id`, date, débit, crédit, `in_open_period` ; sortie :
      paires classées) ; `kesh-db` : chargement des candidates (filtres ouverte + R5 par
      `document_owners` de la 15-1b-0, plafond, `in_open_period` par `open_period_rule` de la 15-1a2-i —
      **aucun** prédicat de période neuf : validation P2, R-3, C-15-1b-4 révisée).
- [ ] **T4** (AC1, AC5, AC7) — `kesh-api` : deux routes dans `routes/letterings.rs`, montées avec
      les lectures ; `DbError::LetteringProposalsTooManyLines { max }` (`kesh-db/src/errors.rs`, entrée
      dans `error_code()`) → 422 dans `kesh-api/src/errors.rs` (message interpolé, repli mot pour mot),
      ajoutée à la table de test des codes du lettrage (`errors.rs:4294` : `cas.len()` 10 → **11** ; la
      15-1a2-i n'ajoute aucun code de lettrage — elle **réemploie** `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`
      pour son refus, C-15-1a2-11 —, le compte part donc bien de 10).
- [ ] **T5** (AC6, AC11) — `LedgerLine.lettering_code` (+ constructeurs de test `csv.rs:1303`,
      `pdf.rs:2447`) ; `is_letterable`, `letterable_account_ids`, `AccountResponse::new` et les cinq
      handlers.
- [ ] **T6** — Tests (voir la liste ci-dessous).
- [ ] **T7** (AC10) — `api-external.md`, `CHANGELOG.md`, i18n (**une** clé `error-*`, quatre
      locales).

**Tests de T6** — un par ligne, chacun rattaché à son critère :

1. AC2 — invariant aux trois dates (`openTotal == balance == Σ` du test) ;
2. AC2 — accord avec `trial_balance::generate` à la fin de chaque exercice de la fixture ;
3. AC2 — garde lexicale : la constante de la requête A ne nomme aucune des cinq tables ni `paid_at` ;
4. AC1 — pagination : `total`, `offset`, `limit` renvoyés ; `balance`/`openTotal` identiques sur
   deux pages ; tri date / numéro / `lineId` ;
5. AC1, AC7 — entrées : `asOf` absent = aujourd'hui, `asOf=2026-13-01` → 400 `VALIDATION_ERROR`,
   `limit=999999` → `limit` écrêté à **500** dans la réponse (validation P2, R L-3 : aucun statut HTTP en
   cause), `limit=0` → 1, `offset=-3` → 0 ; ordre des refus : `asOf` mal formé sur un compte d'une autre
   société → 400 ;
6. AC3 — `document` des cinq types (facture, avoir, règlement avec `invoiceNumber`, facture
   fournisseur achat **et** règlement, transaction bancaire), et la précédence règlement +
   transaction → `settlement` ;
7. AC3 — `manuallyLetterable` : faux pour une ligne de pièce, **vrai** pour une ligne seulement
   rapprochée d'une transaction bancaire, faux pour une ligne lettrée ;
8. ~~AC3 — parité~~ — **déplacé à la 15-1b-0**, où la parité se prouve contre un oracle indépendant
   (validation P2, F-4 : la version P1 comparait deux sorties de la même fonction, verte par
   construction). Numéro gardé ;
9. AC4 — `reason` : `letteredAfterAsOf` sur le groupe à cheval à la date intermédiaire,
   `unlettered` ailleurs ;
10. AC4 — `documentState` : un test par valeur (quatre), avec `amountDue` au centime ; un `X` antérieur
    au règlement rend `partiallySettled` (état d'aujourd'hui, assumé) ; `nothingDue` sur une ligne
    **ouverte aujourd'hui** — pièce historique entièrement close, marques effacées en SQL brut, que la
    synchronisation n'a pas lettrée (15-1a2-i P7 point 1, AC14 c) — avec `manuallyLetterable = false` ;
    un reste brut de 0,004 → `nothingDue` ; une ligne d'avoir → `documentState` nul (validation P2, R-2 =
    F-3, F L-1, L-4) ;
11. AC5 — classement stable, une ligne par paire, paire contre-passation en tête ;
12. AC5 — exclusions : ligne de pièce, ligne lettrée, paire tout entière en exercice clos ;
    inclusion d'une paire dont une seule ligne est en période ouverte ;
13. AC5 — plafond : 2 001 candidates → 422 ; 3 000 lignes de factures et 10 candidates → 200 ;
    `limit=999999` écrêté ;
14. AC5 — volume : 1 000 débits et 1 000 crédits d'un même montant, sous le plafond — 1 000 paires,
    temps noté au Dev Agent Record ;
15. AC5 — clés JSON de la réponse figées ;
16. AC6 — `letteringCode` au JSON du Grand livre, nul pour une ligne ouverte ;
17. AC7 — 404 (autre société, inexistant) indiscernables sur les deux routes ; 409 pour un compte
    de charge, et pour un compte **retypé** après lettrage (C104) ; rôle Consultation admis ;
18. AC11 — `letterable` : compte de bilan vrai, compte bancaire faux, compte **archivé** dont le
    compte bancaire est archivé faux (C127), compte `Revenue` faux — dans la liste et dans une
    réponse unitaire ;
19. AC12 (a), (b) et (c) — chaque volet asserte la liste « au X » et `openTotal == balance` avant et
    après ;
20. AC1 — les champs et le contrat de la vue (validation P2, R L-6) : `letteringOrigin` des trois
    origines, la valeur de `letteredOn` (date de la ligne la plus tardive du groupe), `fiscalYearName`,
    `inOpenPeriod` **dans la vue** (une ligne sous la borne, une ligne d'un exercice suivi d'un exercice
    clos), accès par une **clé d'API en lecture**, et les **clés JSON de la vue figées** (comme le
    test 15 pour les propositions).

**Tests existants à relire** : `kesh-api/src/errors.rs:4294` (compte des codes) ; tout test de forme
d'`AccountResponse` ; les fixtures du Grand livre dans `csv.rs` et `pdf.rs`. Les tests de
`reversal_blockers` et de R5 relèvent de la 15-1b-0 (son inventaire fermé, dont
`journal_entries_modification.rs:104` et les tests sur pool — validation P2, F L-7).

## Dev Notes

- `asOf` est **bindé**, jamais `UTC_DATE()` en dur (patron `aged_receivables`, testabilité).
- Le moteur de proposition **ne reprend pas** `kesh-reconciliation/src/matching.rs` (score
  montant/référence/contact, sans compte ni sens) : « même compte », « sens opposés », « non
  lettrée » n'y existent pas (relevé d'août, P1-1 de la 15-1c, conservé).
- Modules : `kesh-core` (moteur), `kesh-db` (`letterings.rs` : postes ouverts, candidates,
  `letterable_account_ids`), `kesh-api`, `kesh-report`, `kesh-i18n` — **cinq, au seuil**, plus `docs/` et
  `CHANGELOG.md`. Le frontend n'est pas touché (types laissés à la 15-1c). `journal_entries.rs` n'est plus
  touché : la refonte de la propriété est la 15-1b-0.
- `letterings.rs` est un repository du socle : **gate `kesh-db` complet** au dernier commit et à chaque
  passe qui le touche (§ « Exception `kesh-db` » du `CLAUDE.md`).

## Pour la 15-1c

*Répercussions des validations P1 et P2 de la 15-1b (et de la P2 des fiches 15-1a2-i, 15-1a2-ii) sur ce
que la 15-1c lit. La fiche 15-1c n'est pas modifiée ici ; sa propre validation les intègre.*

1. **AC8 de la 15-1c est fausse** : « la Balance d'un exercice ne lit que ses écritures » ne vaut
   pas pour un compte de bilan (F-1). Le pied de liste peut dire l'inverse : à la fin d'un
   exercice, le total des postes ouverts **égale** la clôture du compte dans la Balance.
2. **AC3 de la 15-1c** : les motifs sont désormais **deux champs** — `reason` (à `X` :
   `unlettered`, `letteredAfterAsOf`) et `documentState` (aujourd'hui : `unpaid`,
   `partiallySettled`, `nothingDue`, `paidWithoutSettlementEntry`), avec `amountDue`. « Lettrée
   après cette date, par … » se lit dans `letteringCode` et `letteredOn`. Pour un `X` passé, dire
   que l'état de la pièce est celui d'aujourd'hui.
3. **AC4 de la 15-1c** : la case à cocher suit `manuallyLetterable` (ne pas recopier R5 ; une ligne
   seulement rapprochée d'une transaction bancaire **a** une case) ; `inOpenPeriod` permet de dire
   d'avance qu'une sélection toute en période close sera refusée.
4. **AC5 de la 15-1c** : le contrat des propositions est celui d'AC5 ci-dessus (`reversalPair`,
   `daysApart`, `candidateCount`) ; aucune paire proposée n'est structurellement refusée (R7
   filtré), seule une proposition **périmée** l'est.
5. **AC6 de la 15-1c** : `letteringOrigin` est sur chaque item.
6. **AC1 de la 15-1c** : `letterable` est sur `GET /accounts` (et les quatre réponses unitaires) ;
   le type TypeScript `AccountResponse` (`accounts.types.ts:45`) et celui du Grand livre
   (`letteringCode`) sont à compléter par la 15-1c.
7. **Un compte devenu non lettrable** rend 409 sur la vue : l'écran ne le propose pas au sélecteur
   (il suit `letterable`), et un lien périmé affiche le message du refus.
8. **« Au X » n'est pas un instantané** *(réécrit à la validation P2)* : une impression de justification
   est reproductible pour un `X` **en période close** — dans un exercice clôturé, ou ≤ la borne du verrou
   de période — **tant qu'aucun administrateur ne déverrouille, ne rouvre un exercice ni ne restaure une
   sauvegarde** ; au-dessus de la borne, elle ne l'est pas — à dire à l'écran ou au manuel (AC12 de la
   15-1c).
9. **Le refus du rang 2 bis** (15-1a2-i P7 point 2) : l'annulation d'un règlement, d'un solde, d'un
   rapprochement ou d'un paiement fournisseur dont le lettrage est figé par une période close est refusée
   (`409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) ; les écrans d'annulation le disent déjà (15-1a2-i
   AC17). Le manuel de la 15-1c, qui explique le lettrage figé, y renvoie plutôt que de le redire.
10. **Les lignes sans pièce d'une annulation** : après l'annulation d'un règlement client, sa ligne et son
    miroir sont lettrés `reversal` ; si l'utilisateur délettre cette paire (permis : le règlement n'est
    plus possédé), les deux lignes reparaissent avec `document = null` et `manuallyLetterable = true` — la
    facture n'est plus joignable, sa ligne `invoice_settlements` ayant été retirée. Les propositions les
    remettent en tête (`reversalPair`). Ce n'est pas un angle mort de la vue : c'est ce que dit le grand
    livre.
11. **Le sous-objet de ligne des propositions** porte `fiscalYearName` (AC5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des trois fiches, en autonomie)

**Rapports** : `kesh-gate-logs/15-1b-validate-p2-R.md` (**1 HIGH, 3 MEDIUM, 7 LOW**) et `…-F.md` (**1 HIGH,
4 MEDIUM, 10 LOW**) — 0 CRITICAL. Doublons : R-1 = F-1, R-2 = F-3, R-4 = F-5, R-3 = F L-2, R L-2 = F-2,
R L-7 = F L-4 → **1 HIGH et 5 MEDIUM distincts**, **tous nés des remédiations P1** — trois de la
**collision** des remédiations P1 de la 15-1b (`e1fa4b6c`) et de la 15-1a2 (`7fca24f9`), écrites à quinze
minutes d'écart chacune contre l'ancien état de l'autre ; deux du choix C-15-1b-2. **Trend** : P1 **0 HIGH /
11 MEDIUM distincts** → P2 **1 HIGH / 5 MEDIUM distincts**. ⚠️ **Signal D5 levé** (sévérité qui remonte,
HIGH né d'une remédiation) — **suivi** : décision de l'orchestrateur, la refonte du socle est **extraite**
en **15-1b-0** (C-15-1b-9), la cause de la collision traitée en remédiant les trois fiches d'une seule
main. Le découpage proposé au Change Log P1 (postes ouverts / propositions) n'isolait aucun finding :
abandonné. Chaque finding relu au code (`056997b0`).

| finding | sévérité | verdict | où |
|---|---|---|---|
| R-1 = F-1 — AC12 (b) et la définition prescrivent la dissolution sous verrou que la 15-1a2-i écartait | HIGH | **corrigé** : la 15-1a2-i **refuse** l'annulation (C-15-1a2-10) ; définition réécrite (stable en période close tant qu'aucun administrateur n'agit, non stable au-dessus) ; AC12 en trois volets ; Reçu 2 et 10, « Pour la 15-1c » 8 | Définitions, Reçu, AC12, Pour la 15-1c ; C-15-1b-11 |
| R-2 = F-3 — « Pour la 15-1b » de la 15-1a2-i non intégrée | MEDIUM | **corrigé** : D1 avec sa portée et ses exceptions ; cas historique clos testé (test 10) ; compte non lettrable non montrable (409) écrit dans les deux fiches ; le cas « facture due, ligne de vente lettrée » n'existe plus ; lignes sans pièce d'une annulation délettrée (Pour la 15-1c 10) | D1, test 10 |
| R-3 (= F L-2) — deux factorisations du prédicat des périodes | MEDIUM | **corrigé** : celle de la 15-1a2-i, employée telle quelle (C-15-1a2-18, C-15-1b-4 révisée) | AC3, T3, Reçu 12 |
| R-4 = F-5 — `document_owners` / `reversal_blockers` : signature inapplicable, appelants non inventoriés | MEDIUM | **corrigé par extraction** en 15-1b-0 : `&mut MySqlConnection`, inventaire fermé (six sites de production, six appels de test), route en transaction de lecture (C-15-1b-10) | AC3, T2 → 15-1b-0 |
| F-2 — « stable dans un exercice clos » ignore réouverture et déverrouillage | MEDIUM | **corrigé** (et la restauration) | Définitions, AC10, AC12 |
| F-4 — test 8 de parité vert par construction | MEDIUM | **corrigé par extraction** : oracle **indépendant** en 15-1b-0 (requête gelée de `056997b0` en module de test, valeurs écrites), mutations éprouvées | test 8 → 15-1b-0 |
| R L-1 — ordre et dépendances (15-1a2-i / ii) | LOW | **corrigé** | Status, Story |
| R L-3 — test 5 ambigu (« → 500 ») | LOW | **corrigé** | test 5 |
| R L-4 — message du 422, plafond | LOW | **corrigé** (variante `{ max }`, repli mot pour mot) | AC5, T4 |
| R L-5 — type du prédicat `is_letterable` | LOW | **corrigé** (`&str`) | AC11 |
| R L-6 — champs d'AC1 sans test, clés JSON de la vue | LOW | **corrigé** (test 20) | T6 |
| R L-7 = F L-4 — `documentState` nul pour un avoir | LOW | **corrigé** : choix écrit (C-15-1b-12) | AC4 |
| F L-1 — `amountDue` brut | LOW | **corrigé** : au centime, seuils sur cette valeur | AC4, test 10 |
| F L-3 — AC12 (a) teste un geste refusé | LOW | **corrigé** (gestes réussis) | AC12 |
| F L-5 — `fiscalYearName` absent des propositions | LOW | **corrigé** | AC5 |
| F L-6 — « non lettrée d'office parce qu'elle l'était déjà » | LOW | **corrigé** | AC5 |
| F L-7 — tests à relire | LOW | **corrigé** (relèvent de la 15-1b-0) | Tests existants |
| F L-8 — type TS `AccountResponse` | LOW | **corrigé** | AC11, Pour la 15-1c 6 |
| F L-9 — ordre des refus ; manuel non dit | LOW | **corrigé** (C-15-1b-12) | AC7, AC10 |
| F L-10 — filtre R5 avant le plafond, sans plafond propre | LOW | **corrigé** (écrit, mesuré au test 13) | AC5 |

**Propagation** : « seulement verrouillée », « non stable sous », « suppose la 15-1a2 — groupe `document`
dissous », « `document_owners(executor` », prédicat en `kesh-core`, « `Account` » (type TS) grepés sur les
trois fiches, l'index et le registre. Choix consignés : **C-15-1b-9 à 12** ; C-15-1b-2 et C-15-1b-4
annotés. **Recompte** (depuis ce fichier) : **12 critères** (AC1–AC12 ; AC3 garde la lecture, la refonte
part en 15-1b-0), **6 tâches** (T1, T3–T7 ; T2 déplacée, numéro gardé), **19 tests** (1–7, 9–20 ; le 8
déplacé, le 20 neuf). Prochaine passe : **P3, Sonnet**, complète ; la 15-1b-0 commence à sa **P1**.

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5)

**Rapports** : `kesh-gate-logs/15-1b-validate-p1-R.md` (**9 MEDIUM, 8 LOW**) et
`15-1b-validate-p1-F.md` (**8 MEDIUM, 5 LOW**) — 0 CRITICAL, 0 HIGH ; 17 MEDIUM et 13 LOW bruts,
**11 MEDIUM distincts** après recoupement (R1=F-2, R2≈F-3, R3≈F-6, R4≈F-4, R5=F-5, R6⊂F-3, R7=F-7 ;
propres : R8, R9, F-1, F-8). Chaque finding vérifié au code sur `056997b0` avant correction.

| finding | sévérité | verdict | où |
|---|---|---|---|
| R1 / F-2 — `ACCOUNT_NOT_LETTERABLE` 400 n'existe pas | MEDIUM | **corrigé** : 409 `LETTERING_ACCOUNT_NOT_LETTERABLE` réutilisé, une seule clé neuve, `cas.len()` 10 → 11 | Définitions, AC7, AC10, T4, T7 ; C-15-1b-1 |
| R2 / F-3 (iii) — pièce interdite « dans la requête » mais exigée par AC3/AC4 | MEDIUM | **corrigé** : requête A sans pièce, requête B sur la page ; garde en valeur **et** lexicale | AC2, T1, tests 1-3 |
| R6 / F-3 — `reason` à quelle date, précédence, données manquantes | MEDIUM | **corrigé** : `reason` (à X) et `documentState` (aujourd'hui) séparés, précédence écrite, `amountDue`, `letteredOn`, `letteringAfterAsOf` retiré (redondant) ; la question du « TTC arrondi » (F-3 ii) tombe avec l'ancienne définition | AC1, AC4 ; C-15-1b-3 |
| R3 / F-6 — contrat des propositions absent | MEDIUM | **corrigé** : JSON, défaut `limit` 100, `reversalPair`, plafond **après** R5 | AC5 ; C-15-1b-4 |
| R4 / F-4 — `reversal_blockers` unitaire, N+1, types non fermés, règlement sans facture | MEDIUM | **corrigé** : `document_owners` par lot, source unique, liste fermée et précédence, facture du règlement ; test de parité | AC3, T2, test 8 ; C-15-1b-2 |
| R5 / F-5 — R7 absente du moteur | MEDIUM | **corrigé** : paire tout entière en période close non proposée, même prédicat pur | AC5, T3, test 12 ; C-15-1b-4 |
| R7 / F-7 — `letterable` : N+1, handlers non dits | MEDIUM | **corrigé** : prédicat pur + lot + cinq handlers. *Précision* : R7 cite un `get_account` qui n'existe pas (`grep -n "pub async fn" routes/accounts.rs` → cinq handlers, ceux de F-7) | AC11, T5, test 18 ; C-15-1b-7 |
| R8 — critères sans test | MEDIUM | **corrigé** : 19 tests nommés, un critère chacun | T6 |
| R9 — pas de booléen « lettrable à la main », ni d'origine | MEDIUM | **corrigé** : `manuallyLetterable`, `inOpenPeriod`, `letteringOrigin` | AC1, AC3 |
| F-1 — « différera de la Balance » faux | MEDIUM | **corrigé** : prémisse réfutée au code (`trial_balance.rs:41-42`, `opening.rs:75`, `balance_sheet.rs:9`) ; paragraphe réécrit, assertion contre `trial_balance::generate` | Définitions, AC2, test 2 ; « Pour la 15-1c » 1 |
| F-8 — « au X » n'est pas un instantané | MEDIUM | **corrigé** : stable sur exercice clos, non stable ailleurs, écrit et testé ; lecture en une transaction (LOW joint) | Définitions, AC1, AC12, test 19 ; Reçu 10 |
| L1 / F-10 — références périmées | LOW | **corrigé** : `journal_entries.rs:2090`, `routes/journal_entries.rs:40-41`, `accounts.rs:265`, `trial_balance.rs:82` retiré | AC1, AC3, AC7 |
| L2 — 15-1a2 non livrée, noms du socle | LOW | **corrigé** : Status (ce qui en dépend), Reçu 7-9 | Status, Reçu |
| L3 — paire contre-passation « rare », détection non écrite | LOW | **corrigé** : par `reverses_entry_id`, « rare » retiré | AC5 |
| L4 — « le développeur constate » | LOW | **corrigé** par F-1 | Définitions |
| L5 — compte devenu non lettrable | LOW | **corrigé** : 409, groupes consultables par `GET /letterings/{key}` | Définitions, AC7, test 17 ; C-15-1b-1 |
| L6 / F-13 — entrées non spécifiées | LOW | **corrigé** : parse, défaut, bornes, écho | AC1, test 5 ; C-15-1b-5 |
| L7 / F-10 — T1 `kesh-db` ou `kesh-report` | LOW | **corrigé** : `kesh-db` | T1 ; C-15-1b-8 |
| L8 / F-11 / F-12 — exports du Grand livre, CHANGELOG, tests à relire | LOW | **corrigé** : JSON seul, CSV/PDF inchangés, CHANGELOG et tableau des ressources | AC6, AC10, T6 ; C-15-1b-6 |
| F-9 — moteur « linéaire » | LOW | **corrigé** : `O(Σ n_k·m_k)` borné, test de volume | AC5, test 14 |

**Réfuté** : aucun finding en entier ; une précision de R7 (`get_account`).

**Relevé hors fiche** : la table des décisions de la 15-1a-i (l. 63) dit que « la vue (15-1b) les
regroupe par pièce avec leur reste » ; la vue liste des **lignes**, chacune avec sa pièce et le
reste dû de celle-ci, sans regroupement. Fiche livrée, non modifiée ; à corriger si elle est
rééditée.

**Recompte** (depuis ce fichier) : **12 critères** (AC1–AC12 ; AC12 neuf), **7 tâches** (T1–T7 ;
l'ancienne T2 est scindée en T2 — propriété par lot — et T3 — moteur), **19 tests** nommés en T6.
Choix consignés **C-15-1b-1 à C-15-1b-8** au registre.

**Découpage (D5)** : non levé. C'est une passe 1 (aucune sévérité antérieure à comparer) ; les
défauts sont d'origine, aucun ne vient d'une remédiation ; les modules restent **cinq**
(`kesh-core`, `kesh-db`, `kesh-api`, `kesh-report`, `kesh-i18n`), au seuil sans le dépasser. ⚠️ La
remédiation **élargit** pourtant la story : elle réécrit deux fonctions du socle
(`reversal_blockers`, `letterable_account`). Si la passe 2 trouve un recyclage, le découpage
naturel est **15-1b-i** (postes ouverts : AC1–AC4, AC6–AC12, T1, T2, T5) et **15-1b-ii**
(propositions : AC5, T3), la seconde consommant le lot d'AC3 — proposé, non fait.

**Verdict** : passe 2 due (Opus, contexte frais), complète — la remédiation change des règles et
touche plusieurs modules.

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
