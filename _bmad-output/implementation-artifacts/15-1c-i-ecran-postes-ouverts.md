# Story 15.1c-i : L'écran « Postes ouverts » — consulter, lettrer, délettrer

## Status

review — développée le 2026-10-10 (Opus 5.5, en autonomie). *Historique :* ready-for-dev **après la livraison de la 15-1c-0** *(et donc de la 15-1b, de la 15-1b-0, des 15-1a2-0, -i, -ii)*
— créée le 2026-10-09 par le découpage de la 15-1c à la remédiation de sa validation P1 (registre **C-15-1c-1**) ;
sa partie serveur (AC15, AC16 et leurs tests) extraite en **15-1c-0** à la remédiation de la validation P2
(**C-15-1c-14**) ; **VALIDATION CLOSE** (P3 Sonnet ×2 : 0 au-dessus de LOW ; P4 ciblée Haiku : 0 au-dessus de LOW, LOW appliqués).

⛔ **Ordre** : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2-0 → 15-1a2-i → 15-1a2-ii → 15-1b-0 → 15-1b →
15-1c-0 → 15-1c-i → 15-1c-ii**. Sur la base `f9b6b199`, seules la 15-1a-i et la 15-1a-ii sont livrées : **rien de
cette story ne se développe avant la 15-1c-0 mergée** — la vue, les propositions, `letterable` (15-1b) et la lecture
enrichie d'un groupe (15-1c-0) n'existent pas avant elles. Le scénario E2E (1) suppose en outre la 15-1a2-i (groupes
`document`). Le développement se fait sur une branche **rebasée sur `main` après le merge de la 15-1c-0**.

⛔ **Pas de tag v0.13.0 entre la 15-1c-i et la 15-1c-ii** (règle de publication de C124, étendue par C-15-1c-1) :
entre les deux, le manuel dit encore « le délettrage se fait par l'API dans cette version » et le CHANGELOG
« l'écran viendra », alors que l'écran existe.

## Story

**As a** indépendant, PME ou fiduciaire,
**I want** un écran où je choisis un compte et une date, vois ce qui y reste ouvert et pourquoi, lettre à la
main ce que Kesh ne lettre pas seul — en m'appuyant sur ses propositions —, et délettre une erreur,
**so that** je tienne mes comptes de tiers et de passage soldés sans quitter Kesh, et sache, à la clôture, ce
que porte chacun d'eux.

Partie « écran » de la 15-1c (#518, **`refs #518`** — c'est la **15-1c-ii** qui ferme l'issue). Story
**frontend + i18n** : aucune route, aucune ligne de code serveur ; le serveur qu'elle lit est livré par la 15-1b et
la 15-1c-0. La suivante (15-1c-ii) porte le lettrage dans le reste de Kesh — fiche d'écriture, Grand livre,
`ENTRY_LETTERED` —, le manuel, l'entrée *Ajouté* du CHANGELOG et le site.

## Contrats consommés (lecture seule)

| contrat | fiche validée | ce que l'écran en lit |
|---|---|---|
| `GET /api/v1/accounts/{id}/open-items?asOf=&limit=&offset=` | 15-1b AC1, AC3, AC4, AC7 | items (`reason`, `documentState`, `amountDue`, `manuallyLetterable`, `inOpenPeriod`, `letteringCode`, `letteredOn`, `document`, `fiscalYearName` — `letteringOrigin` n'est pas lu : l'origine s'affiche au panneau du groupe, depuis le `GET`), `balance`, `openTotal`, `total`, `offset`, `limit` ; 409 `LETTERING_ACCOUNT_NOT_LETTERABLE`, 404, 400 |
| `GET /api/v1/accounts/{id}/lettering-proposals?limit=` | 15-1b AC5 | paires (`amount`, `daysApart`, `reversalPair`, `debit`, `credit`), `candidateCount`, `total`, `limit` ; **pas d'`offset`** ; 422 `LETTERING_PROPOSALS_TOO_MANY_LINES` |
| `GET /api/v1/accounts` → `letterable` | 15-1b AC11 | le sélecteur |
| `POST /api/v1/letterings`, `DELETE /api/v1/letterings/{key}` | 15-1a-i (livrée, `routes/letterings.rs`) | lettrer (201 ; refus 400, 404, 409), délettrer (204 ; refus 404, 409) |
| `GET /api/v1/letterings/{key}` **enrichi** | **15-1c-0** AC15 | le groupe : `accountNumber`, `accountName`, `manualDissolutionBlockedBy`, et par ligne `journal`, `description`, `document`, `ownedByDocument`, `inOpenPeriod` |

Les points 1 à 13 de la section « Pour la 15-1c » de la 15-1b sont intégrés ci-dessous (table de renvoi au
Change Log).

## Critères d'acceptation

*Numérotation de la 15-1c conservée (AC1–AC8, AC10, AC11 part i, AC13 part i) ; AC9, AC12, AC14, AC17 et les parts
ii sont à la 15-1c-ii ; AC15, AC16 et AC18 à la 15-1c-0 ; AC19 est neuf.*

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

- **Pied** : **total des postes ouverts** (`openTotal`) et **solde du compte à la date** (`balance`), affichés
  **avec leur sens** (AC8), et la phrase qui dit leur égalité. Les deux portent sur **tout** l'ensemble, non sur la
  page (15-1b AC1).
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
  erreur générique — codes du `POST` (`kesh-api/src/errors.rs`, table des codes du lettrage) : en **400**,
  `LETTERING_TOO_FEW_LINES`, `LETTERING_TOO_MANY_LINES` (de forme : l'écran ne les provoque pas, son bouton les
  prévient) ; en **409**, `LETTERING_ACCOUNTS_DIFFER`, `LETTERING_ACCOUNT_NOT_LETTERABLE`,
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, `LETTERING_LINE_OWNED_BY_DOCUMENT`, `LETTERING_LINE_ALREADY_LETTERED`,
  `LETTERING_UNBALANCED`, `LETTERING_CONCURRENT_CHANGE` (« réessayez », un refus **métier**, jamais un 500 —
  R7 point 4 ; il n'est pas per-proposal, 15-1a2-i C-15-1a2-14, et l'écran n'a pas de lot) ; en **404**
  `NOT_FOUND`, une ligne inconnue.
- ⛔ **Tout refus 404 ou 409 dit que la liste est périmée** (validation P2, R M-3 = F2-4 ; **C-15-1c-17**). Depuis
  #532, une écriture ouverte se **modifie** — ses lignes sont supprimées puis réinsérées sous d'autres identifiants
  (`journal_entries.rs`, étape 9 de la modification) — et se **supprime** : une ligne retenue dans la sélection peut
  avoir disparu (404), changé de compte (`LETTERING_ACCOUNTS_DIFFER`), de montant (`LETTERING_UNBALANCED` alors que
  la somme affichée, faite sur les montants **retenus**, est nulle), ou être passée sous un verrou posé entre-temps.
  Donc, après l'affichage du message : la liste **et** les propositions se **rechargent**, et la sélection est
  **effacée** — pour **tout** 404 ou 409, sans liste de codes à tenir ; seuls les 400 de forme gardent la
  sélection. ⚠️ **Pas d'exception** pour `LETTERING_CONCURRENT_CHANGE`, bien que son message dise « réessayez »
  (validation P3, F-3 ; validation P4, F-4) : la sélection est effacée comme pour tout 409 — le refus dit que les lignes ont changé entre la lecture et l'écriture, sans dire
  lesquelles ; « réessayer » sur des montants retenus avant le changement pourrait lettrer un état que l'écran n'a
  pas montré. L'utilisateur reconstruit sa sélection sur la liste rechargée. Le 404 ne porte pas de message
  utile : l'écran affiche le sien (clé `open-items-*`) — « une ligne sélectionnée n'existe plus : son écriture a été modifiée ou supprimée ».
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
- **Proposition périmée** (une de ses lignes lettrée entre-temps, son écriture modifiée ou supprimée, une borne
  posée) : **tout** refus 404 ou 409 s'affiche par son message (le 404 par celui d'AC4), puis la liste et les
  propositions se **rechargent** — la règle même d'AC4 (C-15-1c-17). Aucune paire proposée n'est structurellement
  refusée (R7 filtré par le moteur, point 4).
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
- **Contenu** (réponse enrichie, 15-1c-0 AC15) : le code, l'origine **en clair**, le compte (`accountNumber`,
  `accountName`), et chaque ligne — date, exercice et n° d'écriture (lien vers la fiche), journal, libellé, pièce
  (table d'AC2), débit, crédit. Origine : `manual` → « lettrage manuel » ; `reversal` → « contre-passation » ;
  `document` → « lettrage de la pièce » suivi du numéro de la **pièce** portée par les lignes, choisi ainsi
  (validation P3, F-4) : le `number` de la **première** ligne (ordre des lignes du `GET`) dont `document.type` est
  `invoice` ou `supplierInvoice` ; sinon l'`invoiceNumber` de la première ligne `settlement` ; sinon « lettrage d'une
  pièce ». Un groupe facture + avoir montre donc le numéro de la facture, jamais celui de l'avoir. ⚠️ Jamais « règlement de la
  pièce » (validation P2, R M-4 = F2-L5 ; **C-15-1c-18**) : un groupe `document` naît aussi d'un **avoir**
  (15-1a2-i, `create_credit_note` → `sync_invoice_in_tx`), où aucun règlement n'existe.
- **« Délettrer »** affiché si et seulement si `manualDissolutionBlockedBy` est nul (15-1c-0 AC15) **et**
  l'utilisateur est Comptable ou Admin. Sinon, la phrase du motif, **par son code**, au **texte de la clé** que le
  serveur emploie pour ce refus — lue par `i18nMsg('error-lettering-…')`, aucun texte parallèle :
  `error-lettering-is-document` (texte **réécrit par la 15-1a2-0**, D5 : il ne dit plus « annulez le règlement »,
  faux pour un groupe facture + avoir — cette fiche ne le cite pas), `error-lettering-line-owned-by-document`,
  `error-lettering-all-lines-in-closed-periods`. Le `DELETE` suffixe le refus 2 du numéro de la pièce
  (`refusal_409`) : l'écran ne recompose pas ce suffixe — la pièce de la ligne possédée est dans sa colonne
  *Pièce*. Pour un groupe `document`, la phrase ajoute le lien vers la pièce. Les replis de ces trois clés valent
  leur valeur fr-CH **entière**, relevée au T0 dans `fr-CH/messages.ftl` (une ligne chacune sur `f9b6b199` ; la
  15-1a2-0 réécrit la première) — G13 —, et un seul repli par clé dans tout le frontend
  (`i18n-un-repli-par-cle`).
- **Refus au clic** (la prévision est indicative, lue sans verrou) : **tout** 404 ou 409 du `DELETE` est affiché
  par son message — dont `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (**jamais** `LETTERING_FISCAL_YEARS_CLOSED`, code
  qui n'existe pas : `grep -rn "LETTERING_FISCAL_YEARS_CLOSED" crates frontend/src docs` ne rend rien) et
  `LETTERING_CONCURRENT_CHANGE` — puis le groupe, la liste et les propositions se rechargent (C-15-1c-17).
- **Après succès** (204) : le panneau dit que le groupe est délettré ; la liste et les propositions se rechargent ;
  le paramètre `group` quitte l'URL.
- **Code inconnu** (404 — inexistant ou d'une autre société, indiscernables) : « aucun groupe ne porte ce code ».

**AC7 — La frontière avec la réconciliation, pour l'utilisateur** (D6 d'août, C95) : un bandeau visible dit que
cet écran **solde des lignes de comptes de tiers et de passage entre elles**, et que **le rapprochement des
relevés bancaires** se fait dans *Mensuel → Réconciliation* ; les comptes bancaires n'apparaissent pas au
sélecteur (`letterable` faux, 15-1a R4). Atteint par un `data-testid`, jamais par son libellé.

**AC8 — Le solde, et la Balance**, dit en pied de liste (15-1b « Définitions », F-1 de sa validation P1 ; point
1). La phrase de la version précédente (« la Balance d'un exercice ne lit que ses écritures ») est **fausse** pour
un compte lettrable — un compte de bilan, dont la Balance part du cumul depuis l'origine — et **retirée**.

- **Le sens** (validation P2, R M-2 = F2-2 ; **C-15-1c-16**). `openTotal` et `balance` sont en **sens débit**
  (`Σ(débit − crédit)`, 15-1b AC2) ; la Balance et le Grand livre montrent le solde **du côté naturel du compte**
  (`kesh-report/src/opening.rs`, `signed` : crédit positif pour un passif). Sur le compte fournisseurs, `-500.0000`
  à l'écran et `500.00` à la Balance désignent le même solde. L'écran affiche donc chaque montant du pied en
  **valeur absolue suivie de son sens** — « débiteur » si `Σ(débit − crédit) > 0`, « créditeur » si `< 0`, rien
  à zéro — par une fonction pure de `features/open-items/` qui ne lit **que le signe** : aucune règle de type de
  compte recopiée en TypeScript (le type n'est pas lu).
- **La phrase** : « Le total des postes ouverts au *X* égale le solde du compte au *X* — celui que la Balance
  montre pour ce compte à cette date, du côté naturel du compte (solde cumulé depuis l'ouverture des livres). »
- L'écran ne compare rien lui-même : l'égalité `openTotal == balance` est l'invariant du serveur (15-1b AC2) ; s'ils
  différaient, l'écran affiche les deux sans phrase d'égalité (défaut à signaler, jamais masqué). La stabilité de
  la liste « au X » (point 8) est dite au **manuel** (15-1c-ii AC12, C-15-1c-12).
- Les colonnes débit et crédit des lignes, et la somme de la sélection (AC4), restent **brutes**.

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
  pour la prévision d'AC6, les clés `error-lettering-*` existantes (espace `error-*` global du lint d'appartenance,
  `lint-i18n-ownership.js`). Clés neuves de cette story : `open-items-*` seulement (dont le texte du 404 d'AC4 et
  les mots « débiteur » / « créditeur » d'AC8, s'ils n'existent pas déjà sous une clé réemployable — relevé au T0).
- **Composant de lien de code** (validation P2, F2-L10 ; C-15-1c-22) : cette story le **crée**, dans
  `features/open-items/`, **sans aucune clé i18n** (le code est son propre texte), qu'il ait ou non un second
  consommateur ici — la 15-1c-ii l'emploie sur la fiche d'écriture et au Grand livre.
- **Gardes à faire passer, nommées** : `frontend/src/lib/shared/i18n-keys.test.ts` (bornes `sitesTotal`,
  `sitesNonResolus`, `relais`, `sitesGabarit`, **relevées aux deux bornes** au T0 et au dernier commit, justifiées
  en commentaire — jamais citées par numéro de ligne), `i18n-un-repli-par-cle.test.ts` (un même repli par clé),
  `i18n-libelle-en-dur.test.ts` (toute fonction `*Label` passe par `i18nMsg`), `i18n-repli-divergent-actif.test.ts`
  (**G13** : repli Svelte = valeur FTL fr-CH), `i18n-entrees-a-variables.test.ts` (variables Fluent ↔ arguments du
  site d'appel — « écart X », « reste dû : X », « N autres », le numéro de la pièce ; validation P3, F-7),
  `e2e-selecteurs-traduits.test.ts` (#326) ; côté Rust,
  `parity_between_locales` (`kesh-i18n/src/loader.rs`), **G8 / G8-bis** (marqueurs d'ordre et bornes de réouverture,
  `loader.rs`, si une clé neuve prescrit une réouverture), **G9** `les_replis_rust_suivent_le_catalogue` et **G4-bis**
  `les_comptes_cites_en_exemple_existent_dans_les_plans_livres` / `la_forme_libre_nnnn_nom_est_juste_partout`
  (`kesh-api/tests/textes_coherents.rs` — tout numéro de compte cité en exemple dans un catalogue). ⛔ Les FTL vivent
  dans `crates/kesh-i18n` : **le gate backend complet** (fmt, clippy, nextest) tourne au dernier commit de code,
  en plus du gate frontend — bien que cette story n'écrive aucun code serveur.

**AC13 (part i) — E2E** (`frontend/tests/e2e/open-items.spec.ts`), sélecteurs `data-testid` seuls (garde #326) :

- **Montage** (C-15-1c-13) : le spec **crée son propre compte** lettrable par l'API des comptes (`Asset`, numéro
  unique dérivé de `Date.now()` **tronqué à dix caractères au plus** — `routes/accounts.rs` refuse au-delà ; patron
  `accounts.spec.ts`, `T${Date.now().toString().slice(-5)}` —, jamais rattaché à un compte bancaire) et des **montants uniques** ; il n'emploie
  aucun compte du seed (1100 est ou devient un compte de journal bancaire, donc non lettrable). Il ne lit que les
  lignes qu'il a créées (par leur `lineId`, en `data-testid`). Les écritures lettrées par le test restent en base
  (figées, `ENTRY_LETTERED`) : le scénario (3) les délettre avant la fin.
- **Scénarios** : (1) une facture soldée par un règlement n'apparaît pas ouverte sur le compte de créance, et son
  groupe s'ouvre par son code avec l'origine « lettrage de la pièce » et sans « Délettrer » — **suppose la
  15-1a2-i** ; compte de créance du preset vérifié lettrable au T0, sinon configuré par l'API des réglages ;
  (2) deux écritures manuelles opposées sur le compte créé → **proposées** → « Lettrer » sur la proposition → plus
  ouvertes, code annoncé ; **rechargement** de la page : même vue (AC1) ; (3) le groupe ouvert par son code
  (`?group=`) → « Délettrer » → les deux lignes de nouveau ouvertes ; (4) une sélection déséquilibrée garde
  « Lettrer » inactif et dit l'écart ; (5) le bandeau de frontière est présent ; (6) **rôle Consultation** (patron
  `journal-entries.spec.ts`, utilisateur créé par l'API) : liste visible, aucune case, aucun « Lettrer », aucun
  « Délettrer ».
- **Lancée au dernier commit de code** (D7), suite complète, jugée fichier par fichier contre `docs/testing.md`
  § « Les échecs attendus ».

**AC19 — README** (validation P2, R L-6 ; C-15-1c-21) : `README.md`, *Feuille de route*, ligne v0.13.0 — l'écran des
postes ouverts (15-1c-i) passe de « À venir » à « Livré sur `main` », dans la PR de cette story (`CLAUDE.md`,
« Synchroniser le planning du README à chaque commit »). Le CHANGELOG n'est pas touché ici : l'entrée *Ajouté* du
lettrage, écran compris, est écrite par la 15-1c-ii (AC14) — pas de tag entre les deux (Status).

## Tasks

- [x] **T0** — Rebaser sur `main` après le merge de la 15-1c-0 ; relever au code livré les noms réels (types
      TypeScript de la vue s'ils ont été écrits, `fetchAccounts`, la forme de la réponse du `GET` enrichi), les
      valeurs fr-CH des trois clés `error-lettering-*` d'AC6 (replis), les clés réemployables d'AC8 et d'AC4, les
      bornes de `i18n-keys.test.ts`, le compte de créance du preset E2E ; écrire au Dev Agent Record ce qui diffère
      de la fiche.
- [x] **T1** (AC1, AC2, AC3, AC8) — `frontend/src/lib/features/open-items/` : `open-items.api.ts`,
      `open-items.types.ts` (types de la vue, des propositions et du groupe enrichi, s'ils ne sont pas écrits),
      composants de liste, de motifs et de pied (montant et sens) ; route
      `frontend/src/routes/(app)/open-items/+page.svelte` (état d'URL) ; `accounts.types.ts` (`letterable`) ;
      entrée de menu.
- [x] **T2** (AC4) — sélection multi-pages, somme `big.js`, état du bouton, `POST`, refus (tout 404/409 :
      rechargement, sélection effacée) et succès.
- [x] **T3** (AC5) — panneau des propositions.
- [x] **T4** (AC6, AC11) — panneau du groupe, `DELETE`, champ « Code » ; composant de lien de code.
- [x] **T5** (AC7, AC10) — bandeau de frontière ; rôles.
- [x] **T6** (AC11 part i) — clés ×4 locales, bornes de `i18n-keys.test.ts`, gardes nommées.
- [x] **T7** (AC19) — `README.md`, ligne v0.13.0.
- [x] **T8** — Tests (liste ci-dessous) ; E2E `open-items.spec.ts` (AC13 part i) ; gate backend complet et gate
      frontend au dernier commit de code.

**Tests de T8** — un par ligne, rattaché à son critère *(Vitest, `features/open-items/*.test.ts` ; les tests
serveur de l'ancienne numérotation 1 à 8 sont à la 15-1c-0)* :

1. AC1 — état d'URL : lecture de `accountId`, `asOf`, `group` ; `asOf` absent → date **locale** écrite dans l'URL
   par remplacement ; `asOf` mal formé → date du jour ; changement de compte → page 1, sélection vide.
2. AC1 — sélecteur : seuls les comptes `letterable`, archivés compris et marqués ; 409 de la vue → message du
   serveur à la place de la liste ; 404 → « compte introuvable ».
3. AC2 — colonnes, exercice avec le numéro, table des liens de pièce (cinq types, `settlement` à `invoiceId` nul
   sans lien, `bankTransaction` sans lien), pagination (page suivante demandée avec `offset`), ordre du serveur
   conservé (une réponse dont l'ordre n'est pas celui des dates s'affiche telle quelle).
4. AC3 — les six libellés, `amountDue` formaté, note « état d'aujourd'hui » quand `asOf` < aujourd'hui et pas
   sinon ; `documentState` nul → aucune seconde ligne.
5. AC4 — case présente **ssi** `manuallyLetterable` (dont une ligne `bankTransaction` lettrable) ; infobulle par
   cause (trois causes).
6. AC4 — somme en décimal : `0.1 + 0.2 − 0.3` à quatre décimales donne **zéro exact** ; états du bouton (moins de
   deux lignes, 201 lignes, écart, toutes en période close, actif) ; sélection conservée d'une page à l'autre et
   effacée au changement de compte ou de date ; le **compteur** dit le nombre de lignes sélectionnées et combien
   hors de la page (validation P3, R L-8).
7. AC4 — refus : chaque code affiché par son message ; **un 409 de chaque code** et **un 404** → rechargement de la
   liste et des propositions, sélection vidée — dont `LETTERING_UNBALANCED` sur une sélection dont la somme
   affichée est nulle, `LETTERING_CONCURRENT_CHANGE` (sélection effacée, sans exception), et le 404 par le texte
   d'écran ; un 400 (`LETTERING_TOO_FEW_LINES`) → sélection gardée ;
   succès → rechargement et code annoncé en lien.
8. AC5 — panneau : **aucun `POST` au montage ni au rendu** ; échec des propositions (422, autre) sans effet sur la
   liste ; repère « contre-passation » ; phrase « N autres » quand `total` dépasse le nombre de paires ; pas de
   rechargement au seul changement de date ; « Lettrer » envoie les deux `lineId` ; refus périmé — un 409 et un
   404 (paire dont une ligne a disparu) → rechargement.
9. AC6 — panneau du groupe : libellés des trois origines (dont « lettrage de la pièce » + numéro, et jamais
   « règlement » ; groupe facture + avoir → le numéro de la **facture**) ; le champ « Code » ouvre le groupe saisi
   (`?group=` écrit dans l'URL) ; « Délettrer » **ssi** `manualDissolutionBlockedBy` nul et rôle d'écriture ; chacun des trois
   motifs par sa clé `error-lettering-*` ; refus au clic (409, 404) par son message puis rechargement ; 204 →
   `group` retiré de l'URL ; 404 à l'ouverture → « aucun groupe ne porte ce code » ; vue en 409 et groupe affiché
   ensemble.
10. AC7, AC8 — bandeau par `data-testid` ; phrase du pied ; **sens** : `openTotal = "-500.0000"` (compte
    fournisseurs, `Liability`) → « 500.00 créditeur », `"20.0000"` → « 20.00 débiteur », `"0.0000"` → « 0.00 »
    sans sens ; `openTotal ≠ balance` → les deux montants sans phrase d'égalité.
11. AC10 — rôle Consultation : aucune case, aucun « Lettrer » (liste et propositions), aucun « Délettrer ».
12. AC11 — gardes nommées d'AC11 vertes (tests existants), bornes de `i18n-keys.test.ts` relevées aux deux bornes
    et justifiées en commentaire ; le composant de lien de code ne lit aucune clé.

*E2E* : AC13 part i, scénarios (1) à (6). *Contrôle documentaire* (AC19) : la ligne v0.13.0 du README relue, sortie
au Dev Agent Record.

*Tests existants à relire* : les tests de forme d'`AccountResponse` côté frontend (le champ neuf) ;
`sidebar-navigation.spec.ts` (l'entrée de menu neuve).

## Dev Notes

- **Modules — aux deux grains** (C-15-1a2-21) : crates et paquets — `frontend`, `kesh-i18n` = **2**, sous le seuil ;
  modules de premier niveau — `features/open-items`, sa route `routes/(app)/open-items`, l'E2E `open-items.spec.ts`
  (**trois de logique**), plus `kesh-i18n` (catalogues), `features/accounts` (un champ de type),
  `routes/(app)/+layout.svelte` (une entrée de menu), `shared/i18n-keys.test.ts` (bornes) — **quatre
  mécaniques** —, et `README.md` (une ligne) : **8 au grain fin**. Dérogation écrite plus bas (C-15-1c-11, motif
  réécrit par C-15-1c-14).
- Aucun code serveur, aucune migration (P5–P8 sans objet) ; aucun repository touché : l'exception `kesh-db` du
  gate ne s'applique pas. Gate backend complet au dernier commit de code quand même (FTL de `kesh-i18n`).
- L'écran ne lit aucune table de pièce : `documentState`, `amountDue`, `document`, `ownedByDocument`,
  `manualDissolutionBlockedBy` viennent du serveur. Il ne recopie ni R5, ni R7, ni l'ordre des refus du délettrage.
- Montage E2E local : `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, `KESH_TEST_MODE=true`,
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `KESH_BACKEND_URL` sur le port attribué (`CLAUDE.md`).
- Le composant de lien de code vit dans `features/open-items/` et n'emploie **aucune** clé i18n (AC11) — sans quoi
  `lint-i18n-ownership` refuserait une clé `open-items-*` dans `features/reports` quand la 15-1c-ii l'y emploiera.

## Dérogation règle de splitting

Au grain des crates et paquets — celui que la règle a toujours appliqué dans cet epic —, la story est à 2, sous le
seuil. Au grain fin, 8 : **trois** modules de logique (l'écran, sa route, son E2E), quatre éditions **mécaniques** —
catalogues ×4, un champ `letterable` de type, une entrée de menu, les bornes d'un test de comptage — et une ligne de
README, qui ne portent aucune règle. Décision de l'orchestrateur (registre **C-15-1c-11**, sur le patron de
**C-15-1a2-23**, qu'elle élargit des textes aux éditions d'une ligne) : pas de découpage supplémentaire — le compte
de logique est sous le seuil, et le reste est mécanique. *(Le motif d'avant — « extraire le serveur laisserait deux
moitiés non livrables » — était faux, la 15-1b étant elle-même une story serveur sans écran ; le serveur a été
extrait en 15-1c-0, C-15-1c-14.)* Accepted risk : une passe de revue relit la propagation mécanique comme un axe à
part entière.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (Claude Code), en autonomie (consignes de l'Epic 15).

### Completion Notes List

- **T0 — relevé du code livré** (tête de la 15-1c-0, `230a635d`) :
  - contrats lus dans `crates/kesh-api/src/routes/letterings.rs` : `OpenItemsResponse` porte en plus `accountId`,
    `accountNumber`, `asOf` (non lus, sauf `accountId`/`asOf` pour les liens de code) ; les lignes des propositions
    n'ont **ni débit ni crédit** (le montant est sur la paire) ; `GET /letterings/{key}` accepte la clé ou le code,
    minuscules comprises (`parse_group_reference`) ; le `POST` rend `LetteringResponse` (forme de la 15-1a-i) dont
    l'écran ne lit que `code`. **Aucun type TypeScript** de la vue n'existait : tous écrits ici
    (`open-items.types.ts`). `fetchAccounts(includeArchived)` existe ; `AccountResponse` n'avait pas `letterable` :
    ajouté, et les sept fabriques de test qui construisent un `AccountResponse` complet le gagnent
    (`letterable: false`).
  - replis des trois clés d'AC6, valeurs `fr-CH` entières (une ligne chacune) : `error-lettering-is-document`
    (« Ce lettrage est celui d'une pièce : il suit la pièce et ses règlements, et ne se défait pas à la main. »),
    `error-lettering-line-owned-by-document`, `error-lettering-all-lines-in-closed-periods` — repris au caractère près.
  - clés réemployables (AC4, AC8) : aucune « débiteur / créditeur » ni texte du 404 dans un espace global ; les clés
    de `features/` voisines sont interdites par `lint-i18n-ownership` → `open-items-*` neuves. Réemployées :
    `common-loading`, `common-previous`, `common-next` (valeurs du catalogue).
  - bornes de `i18n-keys.test.ts` à la base : `sitesTotal` 1923, `sitesNonResolus` 31, `relais` 6, `sitesGabarit` 10.
  - preset E2E `with-company` (relevé sur l'API d'un backend de test) : comptes `1000 Caisse CI`, `1100 Banque CI`,
    `2000 Capital CI` lettrables (aucun compte bancaire) ; `defaultReceivableAccountId` = `1100`. Le scénario (1) le
    vérifie à l'exécution (`letterable` vrai) au lieu de le configurer.
- **T1–T5** — `features/open-items/` : `open-items.types.ts`, `open-items.api.ts`, `open-items.ts` (logique pure, sans
  clé), `open-items-labels.ts` (tous les textes, `switch` exhaustifs), `OpenItemsScreen.svelte` (l'écran ; l'URL est
  la source de vérité, C-15-1c-i-1), `OpenItemsTable.svelte`, `ProposalsPanel.svelte`, `LetteringGroupPanel.svelte`,
  `DocumentCell.svelte`, `LetteringCodeLink.svelte` (aucune clé i18n) ; route `routes/(app)/open-items/+page.svelte`
  (passe-plat) ; entrée `nav-open-items` au groupe Mensuel, entre Réconciliation et Rapports.
- **T6** — 73 clés neuves ×4 locales (72 `open-items-*` + `nav-open-items`), vocabulaire C-15-1a-i-4 (de *Ausgleich*,
  en *matching*, it *abbinamento*, vouvoiement pluriel) ; *Abstimmung / Reconciliation / Riconciliazione* n'apparaissent
  que pour nommer le menu de la réconciliation (bandeau d'AC7). Aucune expression de sélection Fluent (pluriels évités
  par la tournure). Bornes recomptées aux deux bornes (`grep -o "i18nMsg("` fichier par fichier ; aucun fichier
  n'existait à `230a635d`) : `sitesTotal` **1923 → 2015** (+92 : labels 26, écran 24, liste 19, groupe 12,
  propositions 10, route 1) ; `sitesNonResolus`, `relais`, `sitesGabarit` inchangés ; garde « libellé en dur »
  **47 → 60** candidates (`ecartee` 7 → 14, `conforme` 40 → 46, les treize nommées au commentaire) ; garde « un
  repli par clé » étendue à `open-items-` et `error-lettering-` (C-15-1c-i-5), **214 → 289** clés relevées.
- **T7** — README, ligne v0.13.0 : l'écran (15-1c-i) passe sous « Livré sur `main` » ; seule la 15-1c-ii reste « À
  venir ». Relu : `sed -n 223p README.md` → « … (15-1c-0) ; l'écran des postes ouverts — consulter un compte à une
  date, lettrer à la main ou sur proposition, ouvrir et délettrer un groupe (15-1c-i). **À venir** : le lettrage
  montré sur la fiche d'écriture, au Grand livre et au manuel (15-1c-ii) … ».
- **T8 — tests** : **86** tests Vitest neufs (recompté : 116 → 121 fichiers, 1167 → 1253 tests au `test:unit`
  complet, base `230a635d` → commit de développement), en cinq fichiers : `open-items.test.ts` (16), 
  `open-items-labels.test.ts` (11), `OpenItemsTable.test.ts` (13), `panels.test.ts` (13), `OpenItemsScreen.test.ts`
  (33, dont sept engendrés par la boucle des codes 409) — tests 1 à 12 de la fiche couverts (renvois dans les titres). E2E `open-items.spec.ts` : **6 scénarios**, 6/6
  verts sur la branche (backend port 3027, base `kesh_e2e_1ci`) ; `sidebar-navigation.spec.ts` rejoué, 4/4.
  Gardes Rust des catalogues (`kesh-i18n` + `textes_coherents`) : 51/51.
- **Mutations** (fichier restauré puis `touch`é après chacune ; filtre `src/lib/features/open-items`) — **toutes
  rouges** :
  | # | mutation | rougit |
  |---|---|---|
  | M1 | prévision « toutes closes » retirée | état du bouton, période close |
  | M2 | `CONCURRENT_CHANGE` exclu des refus périmés | `isStaleRefusal` ; 409 `CONCURRENT_CHANGE` au `POST` ; refus au clic 409 |
  | M3 | somme en `parseFloat` | zéro exact `0.1 + 0.2 − 0.3` |
  | M4 | lien vers une transaction bancaire | table des cinq liens (pure et rendue) |
  | M5 | sens ignorant le signe | sens pur ; pied « créditeur » ; libellé du pied |
  | M6 | propositions rechargées au changement de date | « changer de date … ne recharge pas les propositions » |
  | M7 | case sur `document == null` au lieu de `manuallyLetterable` | case ssi `manuallyLetterable` |
  | M8 | « Délettrer » toujours offert | groupe `document` ; deux autres motifs |
  | M9 | numéro de pièce = premier document quelconque | facture + avoir (pur et rendu) |
  | M10 | sélection gardée sur un refus périmé | les sept 409 et le 404 au `POST` |
  | M11 | date UTC (mal formée) écrite dans l'URL | `asOf` absent ; `asOf` mal formé |
  | M12 | `group` laissé dans l'URL après un délettrage | 204 → `group` retiré |
- **Modules** (signal D5, déclaré) : 2 crates/paquets (`frontend`, `kesh-i18n`) ; au grain fin les 8 annoncés —
  pas de découpage.

### File List

- `frontend/src/lib/features/open-items/` (neuf) : `open-items.types.ts`, `open-items.api.ts`, `open-items.ts`,
  `open-items-labels.ts`, `OpenItemsScreen.svelte`, `OpenItemsTable.svelte`, `ProposalsPanel.svelte`,
  `LetteringGroupPanel.svelte`, `DocumentCell.svelte`, `LetteringCodeLink.svelte`, `open-items.test.fixtures.ts`,
  `open-items.test.ts`, `open-items-labels.test.ts`, `OpenItemsTable.test.ts`, `panels.test.ts`,
  `OpenItemsScreen.test.ts`
- `frontend/src/routes/(app)/open-items/+page.svelte` (neuf)
- `frontend/src/routes/(app)/+layout.svelte`
- `frontend/src/lib/features/accounts/accounts.types.ts`
- fabriques de test (`letterable`) : `features/bank-accounts/BankAccountJournalLinkForm.test.ts`,
  `features/opening-balances/opening-balances-totals.test.ts`, `features/reconciliation/ManualMatchModal.test.ts`,
  `features/reconciliation/TransactionSplitModal.test.ts`, `routes/(app)/accounts/accounts-page.test.ts`,
  `routes/(app)/bank-accounts/bank-accounts-page.test.ts`,
  `routes/(app)/settings/opening-balances/opening-balances-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts`, `i18n-un-repli-par-cle.test.ts`
- `frontend/tests/e2e/open-items.spec.ts` (neuf)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `README.md`
- `_bmad-output/implementation-artifacts/15-1c-i-ecran-postes-ouverts.md`,
  `sprint-status.yaml`, `epic-15-choix-autonomes.md` (C-15-1c-i-1 à 6)

## Change Log

### Développement — 2026-10-10 (Opus 5.5, en autonomie)

T0 à T8 faits sur la tête de la 15-1c-0 (`230a635d`). 86 tests Vitest neufs, 6 scénarios E2E verts sur la branche,
douze mutations toutes rouges. Choix **C-15-1c-i-1 à 6**. Tests ciblés exécutés (Vitest complet 1253/1253, gardes
Rust des catalogues 51/51, `npm run check` 0 erreur, `lint-i18n-ownership` vert) ; gate complet, frontend et E2E
complet au dernier commit de code, après le rebase sur `main` (15-1c-0 fusionnée).

### Création — 2026-10-09 (Opus 5.5, remédiation de la validation P1 de la 15-1c, en autonomie)

Fiche créée par le découpage de la 15-1c (registre C-15-1c-1 ; R-10 = F-10). Corps réécrit contre les contrats
validés de la 15-1b, de la 15-1b-0, des 15-1a2-* et contre le code de `e892dcfa`. Bilan par finding de la
validation P1 : voir le Change Log de l'index `15-1c-proposition-ecran.md`. Recompté depuis ce fichier
(`grep -c '^\*\*AC[0-9]'`, `grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) : **13 critères** (AC1–AC8, AC10,
AC11 part i, AC13 part i, AC15, AC16), **10 tâches** (T0–T9), **19 tests** numérotés (8 Rust, 11 Vitest) et
6 scénarios E2E.

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p2-{R,F}.md` ; prompt `15-1c-validate-prompt-p2.md`.
**R : 0 HIGH / 5 MEDIUM / 11 LOW ; F : 0 HIGH / 6 MEDIUM / 10 LOW**, tous les MEDIUM nés de la remédiation P1 —
signal D5 levé ; décision de l'orchestrateur : **extraire la partie serveur** (F2-3) en **15-1c-0** (C-15-1c-14).
Cette fiche perd AC15, AC16 et les tests 1 à 8 (serveur), gagne AC19 (README) ; les tests Vitest sont renumérotés
1 à 11 (anciens 9 à 19) et le test 12 (AC11, R L-11) est ajouté. Remédiés ici : R M-2 = F2-2 (sens au pied, AC8 —
C-15-1c-16), R M-3 = F2-4 (tout 404/409 recharge et vide la sélection, AC4, AC5, AC6 — C-15-1c-17), R M-4 = F2-L5
(« lettrage de la pièce », texte de la 15-1a2-0 non cité, AC6 — C-15-1c-18), R L-4 = F2-L9 (clés relevées au T0,
pas de suffixe recomposé), R L-6 (README, AC19 — C-15-1c-21), R L-9 = F2-3 (motif de la dérogation réécrit),
R L-11 (test 12), F2-L10 (composant de lien créé ici — C-15-1c-22). Bilan complet par finding : Change Log de
l'index `15-1c-proposition-ecran.md`. Recompté depuis ce fichier (`grep -c '^\*\*AC[0-9]'`,
`grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) : **12 critères** (AC1–AC8, AC10, AC11 part i,
AC13 part i, AC19), **9 tâches** (T0–T8), **12 tests** numérotés (Vitest) et 6 scénarios E2E.

### Validation P3 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Prompt `15-1c-validate-prompt-p3.md` ; rapports `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p3-R.md` et `-F.md`.
**R : 0 MEDIUM / 10 LOW ; F : 0 MEDIUM / 9 LOW** — aucun MEDIUM+ ; les « 0 » vérifiés par l'orchestrateur (axes
déclarés exercés par les deux lentilles, recoupés au code : séquence de `dissolve_group_in_tx`, statuts des refus,
`colspan`, sites du manuel). LOW appliqués ici : F-3 (`LETTERING_CONCURRENT_CHANGE` vide aussi la sélection, exception assumée — C-15-1c-24), F-4 (numéro de la pièce d'un groupe `document` — C-15-1c-25), F-7 (garde `i18n-entrees-a-variables`), R L-8 (compteur, champ « Code », ordre du serveur testés ; `letteringOrigin` non lu), observation F (numéro de compte E2E ≤ 10 caractères). Bilan complet : Change Log de l'index. Recompté : **12** critères, **9** tâches, **12** tests.

### Validation P4 ciblée — 2026-10-09 (Haiku 4.5, une lentille, commit `99280a24`) — VALIDATION CLOSE

Rapport `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p4-F.md`. **0 MEDIUM / 4 LOW.** Appliqué ici : F-4 (« exception
assumée » se lisait à contresens : `LETTERING_CONCURRENT_CHANGE` n'est **pas** une exception à « tout 409 vide la
sélection » — reformulé à AC4, au test 7 et au registre C-15-1c-24). Le comportement prescrit ne change pas.
