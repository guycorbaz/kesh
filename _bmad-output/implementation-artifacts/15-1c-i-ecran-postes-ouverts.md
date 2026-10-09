# Story 15.1c-i : L'écran « Postes ouverts » — consulter, lettrer, délettrer

## Status

ready-for-dev **après la livraison de la 15-1b** *(et donc de la 15-1b-0, des 15-1a2-0, -i, -ii)* — créée le
2026-10-09 par le découpage de la 15-1c à la remédiation de sa validation P1 (registre **C-15-1c-1**) ;
**validation P2 due** (passe complète, sur les deux sous-fiches).

⛔ **Ordre** : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2-0 → 15-1a2-i → 15-1a2-ii → 15-1b-0 → 15-1b →
15-1c-i → 15-1c-ii**. Sur la base `e892dcfa`, seules la 15-1a-i et la 15-1a-ii sont livrées : **rien de cette
story ne se développe avant la 15-1b mergée** — la vue, les propositions, `letterable` et l'enrichissement
qu'AC15 réemploie n'existent pas avant elle. Le scénario E2E (1) suppose en outre la 15-1a2-i (groupes
`document`). Le développement se fait sur une branche **rebasée sur `main` après le merge de la 15-1b**.

⛔ **Pas de tag v0.13.0 entre la 15-1c-i et la 15-1c-ii** (règle de publication de C124, étendue par C-15-1c-1) :
entre les deux, le manuel dit encore « le délettrage se fait par l'API dans cette version » et le CHANGELOG
« l'écran viendra », alors que l'écran existe.

## Story

**As a** indépendant, PME ou fiduciaire,
**I want** un écran où je choisis un compte et une date, vois ce qui y reste ouvert et pourquoi, lettre à la
main ce que Kesh ne lettre pas seul — en m'appuyant sur ses propositions —, et délettre une erreur,
**so that** je tienne mes comptes de tiers et de passage soldés sans quitter Kesh, et sache, à la clôture, ce
que porte chacun d'eux.

Première des deux moitiés de la 15-1c (#518, **`refs #518`** — c'est la **15-1c-ii** qui ferme l'issue). Story
**frontend + un enrichissement serveur** (AC15) ; aucune route neuve. La moitié suivante (15-1c-ii) porte le
lettrage dans le reste de Kesh — fiche d'écriture, Grand livre, `ENTRY_LETTERED` —, le manuel, le CHANGELOG, le
README et le site.

## Contrats consommés (lecture seule)

| contrat | fiche validée | ce que l'écran en lit |
|---|---|---|
| `GET /api/v1/accounts/{id}/open-items?asOf=&limit=&offset=` | 15-1b AC1, AC3, AC4, AC7 | items (`reason`, `documentState`, `amountDue`, `manuallyLetterable`, `inOpenPeriod`, `letteringCode`, `letteringOrigin`, `letteredOn`, `document`, `fiscalYearName`), `balance`, `openTotal`, `total`, `offset`, `limit` ; 409 `LETTERING_ACCOUNT_NOT_LETTERABLE`, 404, 400 |
| `GET /api/v1/accounts/{id}/lettering-proposals?limit=` | 15-1b AC5 | paires (`amount`, `daysApart`, `reversalPair`, `debit`, `credit`), `candidateCount`, `total`, `limit` ; **pas d'`offset`** ; 422 `LETTERING_PROPOSALS_TOO_MANY_LINES` |
| `GET /api/v1/accounts` → `letterable` | 15-1b AC11 | le sélecteur |
| `POST /api/v1/letterings`, `DELETE /api/v1/letterings/{key}`, `GET /api/v1/letterings/{key}` | 15-1a-i (livrée, `routes/letterings.rs`) | lettrer, délettrer, lire un groupe — ce dernier **enrichi ici** (AC15) |
| `document_owners`, `DocumentKind::blocks_manual_lettering` | 15-1b-0 | réemployés par AC15, jamais recopiés |
| `open_period_rule` / `OpenPeriodRule::line_in_open_period` | 15-1a2-0 D1 | réemployé par AC15 |
| la requête **B** de la vue (enrichissement de la page) | 15-1b T1 | **réemployée** par AC15 |

Les points 1 à 13 de la section « Pour la 15-1c » de la 15-1b sont intégrés ci-dessous (table de renvoi au
Change Log).

## Critères d'acceptation

*Numérotation de la 15-1c conservée (AC1–AC8, AC10, AC11 part i, AC13 part i) ; AC9, AC12, AC14 et les parts ii
sont à la 15-1c-ii ; AC15 et AC16 sont neufs.*

**AC1 — Choisir un compte et une date.**

- **Sélecteur** des comptes dont `letterable` est vrai, lus par `fetchAccounts(true)` — **archivés compris**
  (un compte archivé reste lettrable, 15-1b AC11 ; C96), un archivé étant marqué comme dans le Grand livre
  (patron `reports-ledger-archived`, mais par une clé `open-items-*`). Aucune règle de lettrabilité recopiée en
  TypeScript : le champ vient du serveur. Le type `AccountResponse`
  (`frontend/src/lib/features/accounts/accounts.types.ts`) gagne `letterable: boolean` (15-1b « Pour la 15-1c »
  point 6).
- **Date** `asOf` : par défaut la **date locale du navigateur** (aujourd'hui), **toujours envoyée explicitement**
  au serveur — jamais omise (C-15-1c-5 : l'omission rendrait la date UTC du serveur, 15-1b AC1, en décalage la
  nuit).
- **État dans l'URL** : `/open-items?accountId=…&asOf=…` (et `&group=…`, AC6). Au premier chargement sans `asOf`,
  la date par défaut est **écrite** dans l'URL (remplacement d'historique, pas d'entrée neuve), si bien qu'un
  rechargement ou un lien copié retombe sur la même vue.
- **Compte non lettrable** atteint par un lien périmé (compte retypé ou rattaché à un compte bancaire depuis,
  C104 ; point 7) : la vue répond **409 `LETTERING_ACCOUNT_NOT_LETTERABLE`** ; l'écran affiche **le message du
  serveur** à la place de la liste, et ne propose pas le compte au sélecteur. Un 404 (compte inexistant ou d'une
  autre société) affiche « compte introuvable ».
- `asOf` mal formé dans l'URL → l'écran retombe sur la date du jour et réécrit l'URL (le 400 du serveur n'est
  jamais provoqué par l'écran).

**AC2 — La liste.** Colonnes : date ; **exercice et n° d'écriture** (`fiscalYearName` avec `entryNumber` — le
numéro repart à 1 à chaque exercice, C127 ; lien vers `/journal-entries/{entryId}`) ; journal ; libellé ;
pièce ; débit ; crédit ; motif (AC3) ; et, pour une ligne lettrée après `X`, son code (lien vers le groupe, AC6).

- **Pièce** — cible du lien selon `document.type` (C-15-1c-6) :

  | `document.type` | texte | lien |
  |---|---|---|
  | `invoice` | numéro | `/invoices/{id}` |
  | `creditNote` | numéro | `/credit-notes/{id}` |
  | `supplierInvoice` | numéro (ou « facture fournisseur » s'il est nul, C-15-1a2-17) | `/supplier-invoices/{id}` |
  | `settlement` | « règlement de » + `invoiceNumber` | `/invoices/{invoiceId}` ; **pas de lien** si `invoiceId` est nul |
  | `bankTransaction` | « transaction bancaire » | **aucun** : aucune route n'ouvre une transaction (`bank-import/[id]` prend un identifiant d'**import**) |
  | `null` | — | — |

- **Pied** : **total des postes ouverts** (`openTotal`) et **solde du compte à la date** (`balance`), et la phrase
  qui dit leur égalité (AC8). Les deux portent sur **tout** l'ensemble, non sur la page (15-1b AC1).
- **Pagination du serveur** (défaut 50 ; `limit` écrêté à 500 par le serveur) ; la page courante n'est pas dans
  l'URL.
- Tri : celui du serveur (le Grand livre), jamais retrié côté client.

**AC3 — Pourquoi une ligne est ouverte, et où en est sa pièce** — **deux** dimensions (15-1b AC4 ; point 2),
chacune un libellé et une aide :

| champ | valeur | libellé (fr-CH, repli Svelte = FTL) |
|---|---|---|
| `reason` | `unlettered` | « non lettrée » |
| `reason` | `letteredAfterAsOf` | « lettrée après cette date » + code (lien AC6) + « le » `letteredOn` |
| `documentState` | `unpaid` | « facture non réglée — reste dû : X » |
| `documentState` | `partiallySettled` | « facture partiellement réglée — reste dû : X » |
| `documentState` | `nothingDue` | « pièce soldée, lettrage non posé : période close ou avoir hérité — rien à faire ici » (point 12) |
| `documentState` | `paidWithoutSettlementEntry` | « marquée payée avant la v0.12.0, sans écriture d'encaissement : le compte porte encore cette créance » |

`X` = `amountDue` formaté (déjà au centime, 15-1b AC4). « Par » qui : **non affiché** — le contrat ne porte aucun
auteur ; le code mène au groupe, et l'auteur est au journal d'audit. `documentState` nul → aucune seconde ligne.
⚠️ Quand `asOf` est antérieur à aujourd'hui, une note de colonne dit que **l'état de la pièce est celui
d'aujourd'hui**, le motif à la date étant `reason` (point 2).

**AC4 — Lettrer à la main.**

- **Case à cocher** sur une ligne **si et seulement si** `manuallyLetterable` est vrai et que l'utilisateur est
  Comptable ou Admin (point 3 ; jamais `document != null` : une ligne seulement rapprochée d'une transaction
  bancaire **a** une case). Sans case, une infobulle dit **pourquoi**, par cause, dans cet ordre :
  `reason = letteredAfterAsOf` → « déjà lettrée (code …) » ; `documentState = nothingDue` → le texte d'AC3 ;
  `document` d'un des quatre types qui bloquent → « son lettrage suit sa pièce et ses règlements ».
- **Sélection** : conservée **d'une page à l'autre** (identifiant de ligne, débit, crédit, `inOpenPeriod`
  retenus) ; **effacée** quand le compte ou la date change, et après un lettrage réussi (C-15-1c-4). Un compteur
  dit combien de lignes sont sélectionnées, dont combien hors de la page.
- **Somme de la sélection** `Σ(débit − crédit)`, affichée en continu, calculée en **décimal** (`big.js`, comme
  `features/journal-entries/balance.ts` — jamais `parseFloat` : « somme nulle » en flottant est faux).
- **« Lettrer »** actif si et seulement si : ≥ 2 lignes, somme **exactement nulle**, ≤ **200** lignes
  (`MAX_LINES_PER_GROUP`, `kesh-core/src/lettering.rs`), et **au moins une** ligne `inOpenPeriod` (R7). Sinon le
  bouton dit ce qui manque, dans cet ordre : « sélectionnez au moins deux lignes » ; « 200 lignes au plus » ;
  « la sélection ne s'équilibre pas : écart X » ; « toutes ces lignes sont dans une période close : le lettrage
  n'y change plus » (point 3 ; prévision **indicative**, lue sans verrou — le refus du serveur reste l'autorité).
- **Lettrage à N lignes permis** (règlement groupé, acompte imputé sur plusieurs factures, hors pièces) ; seules
  les **propositions** sont des paires (15-1b AC5).
- **Refus du serveur** : affiché **par son message** (`ApiError.message`, déjà traduit par le serveur), jamais en
  erreur générique — codes du `POST` : `LETTERING_TOO_FEW_LINES`, `LETTERING_TOO_MANY_LINES`,
  `LETTERING_ACCOUNTS_DIFFER`, `LETTERING_ACCOUNT_NOT_LETTERABLE`, `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`,
  `LETTERING_LINE_OWNED_BY_DOCUMENT`, `LETTERING_LINE_ALREADY_LETTERED`, `LETTERING_UNBALANCED`,
  `LETTERING_CONCURRENT_CHANGE` (`errors.rs`, table des codes du lettrage). Les refus qui disent que la liste est
  **périmée** — `LETTERING_LINE_ALREADY_LETTERED`, `LETTERING_CONCURRENT_CHANGE` (« réessayez », un refus
  **métier** 409, jamais un 500 — `errors.rs` R7 point 4 ; il n'est pas per-proposal, 15-1a2-i C-15-1a2-14, et
  l'écran n'a pas de lot), `LETTERING_ACCOUNT_NOT_LETTERABLE` — **rechargent** la liste et les propositions après
  l'affichage du message ; la sélection est alors effacée.
- **Après succès** (201) : la liste et les propositions se rechargent ; un message annonce le **code** du groupe,
  en lien vers lui (AC6).

**AC5 — Les propositions.** Panneau **« Rapprochements proposés »** (15-1b AC5 ; points 4, 11, 13 ; C-15-1c-7).

- **Chargé séparément** de la liste, avec son propre état de chargement et **son propre échec** : un 422
  `LETTERING_PROPOSALS_TOO_MANY_LINES` affiche son message dans le panneau ; toute autre erreur affiche un message
  d'échec dans le panneau ; **dans les deux cas la liste et le lettrage manuel restent utilisables**.
- **Indépendant de `asOf`** : les propositions sont calculées **aujourd'hui** (15-1b AC5) ; le panneau le dit
  (« lignes ouvertes aujourd'hui ») et **ne se recharge pas** quand seule la date change. Une ligne d'une paire peut
  donc être absente de la liste affichée (datée après `X`) : la paire montre ses deux lignes en entier (date,
  exercice et n° d'écriture, journal, libellé, pièce), lues dans la paire elle-même.
- Chaque paire : montant, écart de dates (`daysApart`), le repère **« contre-passation »** quand `reversalPair`
  est vrai (point 10 : ces paires viennent en tête), et un bouton **« Lettrer »** (Comptable, Admin).
- **Aucun lettrage sans clic** (`CLAUDE.md`, « Un appariement automatique propose, il ne crée jamais ») : le
  panneau n'appelle jamais `POST /letterings` de lui-même ; « Lettrer » envoie **les deux** `lineId` de la paire, un
  `POST` par clic.
- **`total` > nombre de paires affichées** : le panneau dit « N autres rapprochements apparaîtront quand ceux-ci
  seront lettrés » (pas d'`offset`, point 13).
- **Proposition périmée** (une de ses lignes lettrée entre-temps, une borne posée) : le refus s'affiche par son
  message, puis la liste et les propositions se **rechargent** (comme AC4). Aucune paire proposée n'est
  structurellement refusée (R7 filtré par le moteur, point 4).
- **Après une acceptation réussie** : liste et propositions rechargées (le moteur recalcule ; coût borné par le
  plafond de 2 000 candidates, 15-1b AC5).

**AC6 — Voir et défaire un groupe.**

- **État d'URL** : `/open-items?group=<code>` — avec ou sans `accountId`/`asOf` (C-15-1c-3). Un groupe s'ouvre par
  son code depuis : un code de la liste (AC2, AC3), le message d'un lettrage réussi (AC4), un champ « Code » de
  l'écran, et — 15-1c-ii — la fiche d'écriture, le Grand livre et le motif `ENTRY_LETTERED`. Le panneau du groupe
  s'affiche **indépendamment de la liste** : sans `accountId`, l'écran lit le groupe (`GET /letterings/{code}`),
  affiche son compte, et ne charge la liste que si l'utilisateur le demande ; si le compte du groupe n'est plus
  lettrable, la liste répond 409 (AC1) **mais le groupe reste affiché et délettrable** (C104 ; 15-1b
  « Définitions », comptes admis).
- **Contenu** (réponse enrichie, AC15) : le code, l'origine **en clair**, le compte, et chaque ligne — date,
  exercice et n° d'écriture (lien vers la fiche), journal, libellé, pièce (table d'AC2), débit, crédit. Origine :
  `manual` → « lettrage manuel » ; `reversal` → « contre-passation » ; `document` → « règlement de la pièce » suivi
  du numéro de la **pièce** portée par les lignes (la facture, la facture fournisseur ; à défaut d'une pièce
  numérotée, « lettrage d'une pièce »).
- **« Délettrer »** affiché si et seulement si `manualDissolutionBlockedBy` est nul (AC15) **et** l'utilisateur est
  Comptable ou Admin. Sinon, la phrase du motif, **par son code** et avec le texte même du refus que le `DELETE`
  rendrait — la clé i18n du serveur, lue par `i18nMsg('error-lettering-…')`, aucun texte parallèle :
  `LETTERING_IS_DOCUMENT` (« … annulez le règlement plutôt que de délettrer »), `LETTERING_LINE_OWNED_BY_DOCUMENT`,
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`. Pour un groupe `document`, la phrase ajoute le lien vers la pièce. Ces trois clés (`fr-CH/messages.ftl`, plusieurs sur deux lignes) sont lues avec un repli égal à leur valeur
  fr-CH **entière**, continuations jointes (G13), et un seul repli par clé dans tout le frontend
  (`i18n-un-repli-par-cle`).
- **Refus au clic** (la prévision est indicative, lue sans verrou) : affiché par son message — dont
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (**jamais** `LETTERING_FISCAL_YEARS_CLOSED`, code qui n'existe pas :
  `grep -rn "LETTERING_FISCAL_YEARS_CLOSED" crates frontend/src docs` ne rend rien sur `e892dcfa`) et
  `LETTERING_CONCURRENT_CHANGE` — puis le groupe et la liste se rechargent.
- **Après succès** (204) : le panneau dit que le groupe est délettré ; la liste et les propositions se rechargent ;
  le paramètre `group` quitte l'URL.
- **Code inconnu** (404 — inexistant ou d'une autre société, indiscernables) : « aucun groupe ne porte ce code ».

**AC7 — La frontière avec la réconciliation, pour l'utilisateur** (D6 d'août, C95) : un bandeau visible dit que
cet écran **solde des lignes de comptes de tiers et de passage entre elles**, et que **le rapprochement des
relevés bancaires** se fait dans *Mensuel → Réconciliation* ; les comptes bancaires n'apparaissent pas au
sélecteur (`letterable` faux, 15-1a R4). Atteint par un `data-testid`, jamais par son libellé.

**AC8 — Le solde, et la Balance**, dit en pied de liste (15-1b « Définitions », F-1 de sa validation P1 ; point
1). La phrase de la version précédente (« la Balance d'un exercice ne lit que ses écritures ») est **fausse** pour
un compte lettrable — un compte de bilan, dont la Balance part du cumul depuis l'origine — et **retirée**. La
phrase affichée : « Le total des postes ouverts au *X* égale le solde du compte au *X* — celui que la Balance
montre pour ce compte à cette date (solde cumulé depuis l'ouverture des livres). » L'écran ne compare rien
lui-même : l'égalité `openTotal == balance` est l'invariant du serveur (15-1b AC2) ; s'ils différaient, l'écran
affiche les deux sans phrase d'égalité (défaut à signaler, jamais masqué). La stabilité de la liste « au X »
(point 8) est dite au **manuel** (15-1c-ii AC12, C-15-1c-12).

**AC10 — Rôles.** **Consultation** : voit la liste, les motifs, les propositions et les groupes ; n'a **ni** cases,
**ni** « Lettrer » (manuel ou proposition), **ni** « Délettrer », sans motif affiché (patron C-15-8-14 de la fiche
d'écriture). Comptable et Admin : tout. Le 403 du serveur reste le refus qui fait autorité.

**AC11 (part i) — i18n.**

- Clés `open-items-*` (dossier `features/open-items/`, propriété vérifiée par `lint-i18n-ownership`) et
  `nav-open-items` (menu, `routes/(app)/+layout.svelte`, groupe **Mensuel**, entre `nav-reconciliation` et
  `nav-reports`), dans les **quatre** locales.
- **Vocabulaire** : celui que la 15-1a-i a retenu pour le lettrage — de-CH *Ausgleich / ausgleichen / Ausgleich
  aufheben*, en-CH *matching / match / unmatch*, it-CH *abbinamento / abbinare / disabbinare* (vouvoiement
  pluriel) — **C-15-1a-i-4**, **C-15-1a-ii-2** ; jamais *Abgleich* / *reconciliation* / *riconciliazione*,
  réservés au rapprochement bancaire. *(Le rapport F citait « C132 » : C132 porte sur la dévalidation ; le
  vocabulaire est à C-15-1a-i-4.)*
- Les messages de refus **ne sont pas** dupliqués : l'écran affiche `ApiError.message` (traduit par le serveur) ou,
  pour la prévision d'AC6, les clés `error-lettering-*` existantes.
- **Gardes à faire passer, nommées** : `frontend/src/lib/shared/i18n-keys.test.ts` (bornes `sitesTotal`,
  `sitesNonResolus`, `relais`, `sitesGabarit`, **relevées aux deux bornes** au T0 et au dernier commit, justifiées
  en commentaire — jamais citées par numéro de ligne), `i18n-un-repli-par-cle.test.ts` (un même repli par clé),
  `i18n-libelle-en-dur.test.ts` (toute fonction `*Label` passe par `i18nMsg`), `i18n-repli-divergent-actif.test.ts`
  (**G13** : repli Svelte = valeur FTL fr-CH), `e2e-selecteurs-traduits.test.ts` (#326) ; côté Rust,
  `parity_between_locales` (`kesh-i18n/src/loader.rs`), **G8 / G8-bis** (marqueurs d'ordre et bornes de réouverture,
  `loader.rs`, si une clé neuve prescrit une réouverture), **G9** `les_replis_rust_suivent_le_catalogue` et **G4-bis**
  `les_comptes_cites_en_exemple_existent_dans_les_plans_livres` / `la_forme_libre_nnnn_nom_est_juste_partout`
  (`kesh-api/tests/textes_coherents.rs` — tout numéro de compte cité en exemple dans un catalogue). ⛔ Les FTL vivent
  dans `crates/kesh-i18n` : **le gate backend complet** (fmt, clippy, nextest) tourne au dernier commit de code,
  en plus du gate frontend.

**AC13 (part i) — E2E** (`frontend/tests/e2e/open-items.spec.ts`), sélecteurs `data-testid` seuls (garde #326) :

- **Montage** (C-15-1c-13) : le spec **crée son propre compte** lettrable par l'API des comptes (`Asset`, numéro
  unique dérivé de `Date.now()`, jamais rattaché à un compte bancaire) et des **montants uniques** ; il n'emploie
  aucun compte du seed (1100 est ou devient un compte de journal bancaire, donc non lettrable). Il ne lit que les
  lignes qu'il a créées (par leur `lineId`, en `data-testid`). Les écritures lettrées par le test restent en base
  (figées, `ENTRY_LETTERED`) : le scénario (3) les délettre avant la fin.
- **Scénarios** : (1) une facture soldée par un règlement n'apparaît pas ouverte sur le compte de créance, et son
  groupe s'ouvre par son code avec l'origine « règlement de la pièce » et sans « Délettrer » — **suppose la
  15-1a2-i** ; compte de créance du preset vérifié lettrable au T0, sinon configuré par l'API des réglages ;
  (2) deux écritures manuelles opposées sur le compte créé → **proposées** → « Lettrer » sur la proposition → plus
  ouvertes, code annoncé ; **rechargement** de la page : même vue (AC1) ; (3) le groupe ouvert par son code
  (`?group=`) → « Délettrer » → les deux lignes de nouveau ouvertes ; (4) une sélection déséquilibrée garde
  « Lettrer » inactif et dit l'écart ; (5) le bandeau de frontière est présent ; (6) **rôle Consultation** (patron
  `journal-entries.spec.ts`, utilisateur créé par l'API) : liste visible, aucune case, aucun « Lettrer », aucun
  « Délettrer ».
- **Lancée au dernier commit de code** (D7), suite complète, jugée fichier par fichier contre `docs/testing.md`
  § « Les échecs attendus ».

**AC15 — `GET /api/v1/letterings/{key}` porte de quoi afficher et défaire un groupe** (R-2 = F-2 de la validation
P1 ; C-15-1c-2). La réponse livrée (`routes/letterings.rs`, `LetteringResponse` : `key`, `code`, `origin`,
`accountId`, et par ligne `id`, `entryId`, `entryNumber`, `fiscalYearId`, `fiscalYearName`, `date`, `debit`,
`credit`) ne permet ni d'écrire « règlement de la facture F-… », ni de savoir si « Délettrer » aboutira. Elle
gagne, **pour le seul `GET`** :

```json
{
  "key": 27, "code": "AA", "origin": "reversal", "accountId": 7,
  "accountNumber": "1100", "accountName": "…",
  "manualDissolutionBlockedBy": null | "LETTERING_IS_DOCUMENT" | "LETTERING_LINE_OWNED_BY_DOCUMENT" | "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
  "lines": [ {
    "id": 27, "entryId": 3, "entryNumber": 12, "fiscalYearId": 1, "fiscalYearName": "2026",
    "date": "2026-03-01", "debit": "100.0000", "credit": "0.0000",
    "journal": "Ventes", "description": "…",
    "document": null | { "type": "settlement", "id": 9, "number": null, "invoiceId": 4, "invoiceNumber": "F-2026-0004" },
    "ownedByDocument": true | false,
    "inOpenPeriod": true | false
  } ]
}
```

- **Mêmes champs, même source que les items de la vue** : `document` (même sous-objet, sérialisé par
  `DocumentKind::as_str()`), `inOpenPeriod` (par `open_period_rule` / `line_in_open_period`), `journal`,
  `description` sont produits par **l'enrichissement de la requête B de la 15-1b**, factorisé en une fonction de
  `kesh-db/src/repositories/letterings.rs` qu'appellent `open_items` **et** la lecture d'un groupe — **jamais** une
  seconde implémentation de la propriété, du reste dû ou des périodes. Si la 15-1b ne l'a pas déjà exposée ainsi,
  cette story l'extrait (refactor sans changement de comportement, prouvé par les tests de la 15-1b inchangés). La
  forme exacte (une fonction prenant `&[(line_id, entry_id, fiscal_year_id, date)]`, ou les en-têtes lus avec les
  lignes) se tranche au T0, à deux contraintes : aucun N+1, et `journal`/`description` lus dans la **même** requête
  que les lignes ou que l'enrichissement — noté au Dev Agent Record.
- `ownedByDocument` = un propriétaire de l'écriture a `DocumentKind::blocks_manual_lettering()` (15-1b-0) — la
  « possession » au sens de R5 ; `bankTransaction` seul ne la donne pas.
- `manualDissolutionBlockedBy` = le **premier** refus que `dissolve_group_in_tx` en mode `Manual` rendrait, **dans
  son ordre** (`letterings.rs`, refus 1, 2, 3) : `LETTERING_IS_DOCUMENT` si l'origine est `document` ; sinon
  `LETTERING_LINE_OWNED_BY_DOCUMENT` si l'origine est `reversal` et qu'une ligne a `ownedByDocument` ; sinon
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` si aucune ligne n'est `inOpenPeriod` ; sinon `null`. ⛔ **L'ordre vit une
  fois** : une fonction **pure** `manual_dissolution_blocker(origin, any_owned, any_in_open_period) ->
  Option<DbError>` que `dissolve_group_in_tx` appelle désormais pour ses refus 1 à 3, et que la lecture appelle ; la
  dissolution garde ses lectures **verrouillantes** (le verrou d'exercice reste pris avant), la lecture les fait
  **sans verrou** — la prévision est **indicative**, le `DELETE` fait autorité. La lettrabilité du compte **n'entre
  pas** dans la prévision (C104 : la dissolution ne l'exige pas).
- `accountNumber`, `accountName` : le compte du groupe (le panneau l'affiche sans charger la liste, AC6) —
  `group_account_number` existe ; le nom se lit dans la même requête.
- ⛔ **Inchangés** : `LetteringGroup` / `LetteringLine` (le type de la primitive), la réponse **201** du `POST`
  (elle garde la forme livrée par la 15-1a-i — l'enrichissement n'y est pas calculé, AC16), les `details`
  d'audit `lettering.created` / `lettering.removed` (`audit_details`, construits champ par champ), le 404
  indiscernable (autre société, clé ou code inexistant), le rôle (Consultation et plus, clé d'API en lecture).
- Aucune migration, aucun code d'erreur neuf, aucune clé i18n côté serveur.

**AC16 — Documentation de l'enrichissement** : `docs/api-external.md`, section « Lettrer des lignes », le
paragraphe de `GET /api/v1/letterings/{key}` décrit les champs d'AC15, dit que la prévision
`manualDissolutionBlockedBy` est **indicative** (lue sans verrou ; le `DELETE` fait autorité) et que le `POST`
garde sa réponse. C'est un **changement de contrat additif** ; son entrée au CHANGELOG est écrite par la
**15-1c-ii** dans l'entrée unique du lettrage (AC14, C-15-1c-10). Tout compte cité en exemple existe dans un plan
livré (G4-bis).

## Tasks

- [ ] **T0** — Rebaser sur `main` après le merge de la 15-1b ; relever au code livré les noms réels (la fonction
      de la requête B, les types TypeScript de la vue s'ils ont été écrits, `fetchAccounts`), les bornes de
      `i18n-keys.test.ts`, le compte de créance du preset E2E ; écrire au Dev Agent Record ce qui diffère de la
      fiche.
- [ ] **T1** (AC15) — `kesh-db` `repositories/letterings.rs` : enrichissement factorisé (réemploi de la requête
      B), lecture détaillée d'un groupe, `manual_dissolution_blocker` pur appelé par `dissolve_group_in_tx` ;
      `kesh-api` `routes/letterings.rs` : DTO du `GET` (`LetteringDetailResponse`), le `POST` inchangé.
- [ ] **T2** (AC16) — `docs/api-external.md`, paragraphe du `GET`.
- [ ] **T3** (AC1, AC2, AC3, AC8) — `frontend/src/lib/features/open-items/` : `open-items.api.ts`,
      `open-items.types.ts` (types de la vue, des propositions et du groupe enrichi, s'ils ne sont pas écrits),
      composants de liste et de motifs ; route `frontend/src/routes/(app)/open-items/+page.svelte` (état d'URL) ;
      `accounts.types.ts` (`letterable`) ; entrée de menu.
- [ ] **T4** (AC4) — sélection multi-pages, somme `big.js`, état du bouton, `POST`, refus et rechargements.
- [ ] **T5** (AC5) — panneau des propositions.
- [ ] **T6** (AC6) — panneau du groupe, `DELETE`, champ « Code ».
- [ ] **T7** (AC7, AC10) — bandeau de frontière ; rôles.
- [ ] **T8** (AC11 part i) — clés ×4 locales, bornes de `i18n-keys.test.ts`, gardes nommées.
- [ ] **T9** — Tests (liste ci-dessous) ; E2E `open-items.spec.ts` (AC13 part i).

**Tests de T9** — un par ligne, rattaché à son critère :

*Rust (`kesh-db`, binaire `letterings.rs` des tests de dépôt ; `kesh-api`, binaire des routes du lettrage —
noms relevés au T0) :*

1. AC15 — groupe `manual` : chaque ligne porte `journal`, `description`, `document = null`,
   `ownedByDocument = false`, `inOpenPeriod` ; `manualDissolutionBlockedBy = null` ; **clés JSON figées**.
2. AC15 — groupe `document` (suppose la 15-1a2-i) : `document` de la créance (`invoice`) et du règlement
   (`settlement`, `invoiceNumber`) ; `manualDissolutionBlockedBy = LETTERING_IS_DOCUMENT`.
3. AC15 — groupe `reversal` dont une ligne reste **possédée** par sa pièce — par exemple la paire posée par
   l'annulation d'une facture fournisseur, la facture annulée possédant toujours son écriture (cas relevé au T0
   contre `document_owners` ; à défaut d'un geste qui le produise, montage en SQL brut avec assertion de
   montage). ⚠️ **Pas** la paire d'un règlement client annulé : sa ligne `invoice_settlements` est retirée, la
   paire n'est plus possédée et se délettre (15-1b « Pour la 15-1c » point 10) — ce cas-là est asserté dans le même test :
   `ownedByDocument` faux, prévision nulle. Attendu du cas possédé : `ownedByDocument` vrai sur cette ligne,
   `LETTERING_LINE_OWNED_BY_DOCUMENT`.
4. AC15 — groupe dont toutes les lignes sont en période close → `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` ; groupe à
   cheval → `null`.
5. AC15 — **la prévision égale la dissolution** : pour chacun des cas 1 à 4, `DELETE` rend exactement le code prévu
   (ou 204 quand la prévision est nulle) — test de table ; et `manual_dissolution_blocker` en test unitaire sur les
   huit combinaisons (origine × possession × période).
6. AC15 — **même source que la vue** : pour une ligne lettrée après `X` (présente dans la vue **et** dans son
   groupe), `document`, `inOpenPeriod`, `journal`, `description` sont **égaux** dans les deux réponses.
7. AC15 — compte du groupe **devenu non lettrable** (retypé, C104) : `GET` 200, `manualDissolutionBlockedBy` nul
   (la lettrabilité n'y entre pas), `DELETE` 204.
8. AC15 — inchangés : réponse 201 du `POST` (clés figées, celles de la 15-1a-i) ; `details` d'audit (tests
   existants de la 15-1a-i verts sans modification) ; 404 d'une autre société et d'un code inexistant,
   indiscernables ; clé d'API en lecture et rôle Consultation admis.

*Vitest (`features/open-items/*.test.ts`) :*

9. AC1 — état d'URL : lecture de `accountId`, `asOf`, `group` ; `asOf` absent → date **locale** écrite dans l'URL
   par remplacement ; `asOf` mal formé → date du jour ; changement de compte → page 1, sélection vide.
10. AC1 — sélecteur : seuls les comptes `letterable`, archivés compris et marqués ; 409 de la vue → message du
    serveur à la place de la liste ; 404 → « compte introuvable ».
11. AC2 — colonnes, exercice avec le numéro, table des liens de pièce (cinq types, `settlement` à `invoiceId` nul
    sans lien, `bankTransaction` sans lien), pagination (page suivante demandée avec `offset`).
12. AC3 — les six libellés, `amountDue` formaté, note « état d'aujourd'hui » quand `asOf` < aujourd'hui et pas
    sinon ; `documentState` nul → aucune seconde ligne.
13. AC4 — case présente **ssi** `manuallyLetterable` (dont une ligne `bankTransaction` lettrable) ; infobulle par
    cause (trois causes).
14. AC4 — somme en décimal : `0.1 + 0.2 − 0.3` à quatre décimales donne **zéro exact** ; états du bouton (moins de
    deux lignes, 201 lignes, écart, toutes en période close, actif) ; sélection conservée d'une page à l'autre et
    effacée au changement de compte ou de date.
15. AC4 — refus : chaque code affiché par son message ; `LETTERING_LINE_ALREADY_LETTERED`,
    `LETTERING_CONCURRENT_CHANGE`, `LETTERING_ACCOUNT_NOT_LETTERABLE` → rechargement et sélection vidée ; succès →
    rechargement et code annoncé en lien.
16. AC5 — panneau : **aucun `POST` au montage ni au rendu** ; échec des propositions (422, autre) sans effet sur la
    liste ; repère « contre-passation » ; phrase « N autres » quand `total` dépasse le nombre de paires ; pas de
    rechargement au seul changement de date ; « Lettrer » envoie les deux `lineId` ; refus périmé → rechargement.
17. AC6 — panneau du groupe : libellés des trois origines (dont « règlement de la pièce » + numéro) ;
    « Délettrer » **ssi** `manualDissolutionBlockedBy` nul et rôle d'écriture ; chacun des trois motifs par sa clé
    `error-lettering-*` ; refus au clic par son message puis rechargement ; 204 → `group` retiré de l'URL ; 404 →
    « aucun groupe ne porte ce code » ; vue en 409 et groupe affiché ensemble.
18. AC7, AC8 — bandeau par `data-testid` ; phrase du pied ; `openTotal ≠ balance` → les deux montants sans phrase
    d'égalité.
19. AC10 — rôle Consultation : aucune case, aucun « Lettrer » (liste et propositions), aucun « Délettrer ».

*E2E* : AC13 part i, scénarios (1) à (6).

*Tests existants à relire* : les tests de forme d'`AccountResponse` côté frontend (le champ neuf) ;
`sidebar-navigation.spec.ts` (l'entrée de menu neuve) ; les tests du `GET /letterings/{key}` de la 15-1a-i (clés
figées : ils changent **de sens** — ils figent désormais la réponse enrichie ; les reprendre explicitement, jamais
les laisser rougir puis « corriger » la valeur attendue sans relire).

## Dev Notes

- **Modules — aux deux grains** (C-15-1a2-21) : crates et paquets — `kesh-db`, `kesh-api`, `kesh-i18n`,
  `frontend` = **4**, sous le seuil ; modules de premier niveau — `kesh-db/repositories/letterings`,
  `kesh-api/routes/letterings`, `features/open-items` et sa route `routes/(app)/open-items`, l'E2E
  `open-items.spec.ts` (**cinq de logique**), plus `kesh-i18n` (catalogues), `features/accounts` (un champ de type),
  `routes/(app)/+layout.svelte` (une entrée de menu), `shared/i18n-keys.test.ts` (bornes) — **quatre mécaniques** —,
  et `docs/api-external.md` (un paragraphe) : **9 au grain fin**. Signal déclaré à l'orchestrateur ; dérogation
  écrite plus bas (C-15-1c-11).
- `letterings.rs` est un **repository** : gate `kesh-db` **complet** à chaque passe qui le touche (§ « Exception
  `kesh-db` »). `dissolve_group_in_tx` change de forme (ses refus passent par la fonction pure) : les tests de la
  15-1a-i et de la 15-1a-ii sur la dissolution doivent rester verts **sans modification** — c'est leur preuve
  d'absence de régression.
- Aucune migration (P5–P8 sans objet).
- L'écran ne lit aucune table de pièce : `documentState`, `amountDue`, `document` viennent du serveur.
- Montage E2E local : `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, `KESH_TEST_MODE=true`,
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `KESH_BACKEND_URL` sur le port attribué (`CLAUDE.md`).
- Composant de lien de code de lettrage : s'il est partagé avec la 15-1c-ii (fiche d'écriture, Grand livre), il vit
  dans `features/open-items/` et n'emploie **aucune** clé i18n (le code est son propre texte) — sans quoi
  `lint-i18n-ownership` refuserait une clé `open-items-*` dans `features/reports`.

## Dérogation règle de splitting

Au grain des crates et paquets — celui que la règle a toujours appliqué dans cet epic —, la story est à 4, sous le
seuil. Au grain fin, 9, dont **cinq** modules de logique (le seuil) et quatre éditions **mécaniques** — catalogues
×4, un champ `letterable` de type, une entrée de menu, les bornes d'un test de comptage — plus un paragraphe de
documentation, qui ne portent aucune règle. Décision de l'orchestrateur (registre **C-15-1c-11**, sur le patron de
**C-15-1a2-23**, qu'elle élargit des textes aux éditions d'une ligne) : pas de découpage supplémentaire.
L'alternative — extraire l'enrichissement serveur (AC15) en une story préalable — laisserait une route enrichie
sans écran qui la lise, et un écran qui ne peut pas montrer un groupe : deux moitiés non livrables seules. Accepted
risk : une passe de revue relit la propagation mécanique comme un axe à part entière, et le changement de forme de
`dissolve_group_in_tx` comme un axe de sécurité.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Création — 2026-10-09 (Opus 5.5, remédiation de la validation P1 de la 15-1c, en autonomie)

Fiche créée par le découpage de la 15-1c (registre C-15-1c-1 ; R-10 = F-10). Corps réécrit contre les contrats
validés de la 15-1b, de la 15-1b-0, des 15-1a2-* et contre le code de `e892dcfa`. Bilan par finding de la
validation P1 : voir le Change Log de l'index `15-1c-proposition-ecran.md`. Recompté depuis ce fichier
(`grep -c '^\*\*AC[0-9]'`, `grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) : **13 critères** (AC1–AC8, AC10,
AC11 part i, AC13 part i, AC15, AC16), **10 tâches** (T0–T9), **19 tests** numérotés (8 Rust, 11 Vitest) et
6 scénarios E2E.
