# Story 43-3 : un règlement pour plusieurs factures — l'écran de réconciliation, façon bexio (#823)

Status: review

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, code lu à
`8b50fd195` ; révisée le soir même (interface comme bexio, tiers mixtes), remédiée après ses validations P1
(`validation-43-3-p1.md`, 0/3/6/8), P2 (`validation-43-3-p2.md`, 0/0/1/7) et **P3** (`validation-43-3-p3.md`, 0/0/1/6), et
**alignée le 2026-11-18 sur la décision de Guy du 2026-11-17 sur la différence** (`kesh-gate-logs/e43/decision-difference.md`)
et sur la 43-2 remédiée de même. Release **0.19.0**. **Commence tout de suite** : types et Vitest sur le contrat de la fiche
43-2 ; l'**E2E** se joue sur la branche réunie (43-1a et 43-2 fusionnées). Frontend et manuel utilisateur ; aucune
migration. Décisions : C-43-14 à C-43-26 (fichier d'épique), qui font foi. Validation terminée en P5 (0/0/0/5) ;
correction de C-43-25 (arrondi automatique) relue par une passe ciblée.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit. Dette connue : #1235 (gardes de
`LinkSettlementModal`), déjà ouverte — ne pas la rouvrir.

## Story

En tant que comptable, je choisis une ligne du relevé, j'y ajoute des factures une à une en les cherchant, je vois à
chaque ajout le total imputé et l'écart avec le montant du relevé ; s'il reste un écart, je choisis le compte où il se
comptabilise, et j'enregistre — puis, si je me suis trompé, j'annule ce rapprochement en voyant les factures qu'il rouvrira.

## Ce qui se lit au code (`8b50fd195`)

- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - modales `ManualMatchModal`, `TransactionSplitModal`, `LinkSettlementModal` (import :78-80) ;
  - `canLink(p)` : montant non nul **et** `currency === 'CHF'` (:274-276) ;
  - **garde « déjà comptabilisée »** (Stories 34-4, 35-4) : commentaire :296-301, `type GuardedGesture` (:302-306 :
    `accept`, `reject`, `manual`, `split`), `guard()` (:316), `runGesture()` (`switch` fermé par `never`),
    `AlreadyBookedConfirmDialog`, `confirmKind = 'write' | 'reject'` ; l'en-tête du fichier énumère les gestes gardés ;
  - bouton « Rattacher à une écriture existante » (:747, `data-testid="link-settlement-button"`).
- `LinkSettlementModal.svelte` : gardes `loadGen` et `session` (:80-83, revue P1 de la 29-8, C-29-8-37), `RELIRE_SUR`
  (:87-91, commentaire :86), `errorMessageOf` (:117-121), `loadError` (:224-229) ; ni anti-rebond ni `AbortController`.
- `CancelReconciliationDialog.svelte`, `reconciliation-cancel.ts` (textes au singulier : « La facture de ce
  rapprochement… »), `reconciliation.types.ts` (`kind` :204, `candidateType` :27, `AcceptedProposal` :134-141,
  `cancelBlockedDocumentId` :222).
- `failed-proposal-label.ts` : `failedProposalLabel(code, details)` — lit `role`, `claim`, `reason`, `fiscalYearName`,
  `tolerance_days`, aucun numéro de facture ; 37 `case` (recomptés), dont déjà `ACCOUNT_NOT_POSTABLE`,
  `ACCOUNT_NOT_FOUND` (:162, « Un compte de contrepartie est introuvable ou archivé »), `INACTIVE_OR_INVALID_ACCOUNTS`,
  `VALIDATION_ERROR` (« Erreur de validation », sans lire `reason`, :425-426), `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (branches `role ===
  'rounding'` puis `claim === 'payable'`, :403-425), `RECONCILIATION_CURRENCY_MISMATCH` (:271) ;
  `failed-proposal-label.test.ts` (« couvre les 37 codes », :95-96 ; recette de comptage en tête, :6-16, par `grep` sur
  `reconciliation.rs` seul).
- Sélecteur de compte : `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` (corrigé par la 36-1).
- Comptes désignés : `GET /api/v1/company/invoice-settings`, lisible par tout rôle authentifié (`lib.rs`, « Story 5.2 :
  lecture config facturation »), porte `defaultRoundingAccountId` et `defaultBankFeesAccountId`.
- `frontend/src/lib/shared/i18n-keys.test.ts` : `sitesTotal` = 3086 (:1040).
- E2E : `frontend/tests/e2e/helpers/api-fixtures.ts` (contacts, factures validées, compte bancaire lié ; aucun helper des
  comptes désignés), `helpers/contrast.ts` ; specs voisines `reconciliation-link.spec.ts`, `reconciliation-supplier.spec.ts`
  (nettoyage en `finally`), `reconciliation-cancel.spec.ts`, `reconciliation-already-booked.spec.ts`.
- Manuel : `docs/manual/fr/user-manual.tex`, `\label{sec:reconciliation}` (:3092), `\label{sec:annuler-rapprochement}`
  (:3306).

## Décisions propres à la story

- **D1 — Le bouton et la garde (H1, M1).** Bouton **« Régler plusieurs factures… »** (`data-testid="group-settlement-button"`)
  par transaction, à côté de « Rattacher à une écriture existante », affiché à la **même condition que `canLink`** (montant
  non nul, CHF ; transaction en attente, non rejetée, d'un compte ouvert). Il passe une écriture : il entre dans la garde —
  nouvelle variante `{ kind: 'group'; proposal }` de `GuardedGesture`, `guard({ kind: 'group', proposal: p },
  [p].filter(isAlreadyBooked))`, `confirmKind = 'write'`, branche de `runGesture` qui ouvre `GroupSettlementModal.svelte`
  (neuf). Le commentaire :296-301 et l'en-tête du fichier nomment le geste.
- **D2 — Interface comme bexio (C-43-14).** En tête, la ligne du relevé (date, contrepartie, montant) ; un champ de
  **recherche** (`GET /reconciliation/transactions/{id}/group-candidates?q=…&exclude=…`) dont chaque résultat s'**ajoute**
  d'un clic ; la liste des factures **imputées**, chacune retirable, avec son reste dû ; en pied, **montant du relevé**,
  **total imputé** et **« Écart »** — jamais « différence », mot que l'API réserve à `difference` — chacun avec un **texte**,
  jamais la seule couleur. Aucune présélection : la modale s'ouvre avec **zéro** facture imputée ; le nom de la contrepartie
  préremplit seulement la recherche (un nom sans jeton de 3 caractères ou plus — « BL SA » — est servi par le repli `LIKE`
  de la 43-2). Réponse de recherche à `direction: null` (montant nul, devise non CHF) → état **« aucune facture à proposer
  pour cette transaction »**, distinct de « aucun résultat » pour un `q` (P3, M1).
- **D2 bis — Le compte de l'écart (décision de Guy du 2026-11-17 ; libellés et préremplissage par famille, décision de
  l'orchestrateur du 2026-11-18, C-43-26).** Dès que l'**écart au centime `e`** (D3) est non nul, un
  **sélecteur de compte** apparaît sous « Écart », avec le **sens** en libellé et un **préremplissage** propres à la
  famille :

  | Famille | Écart | Libellé | Prérempli avec |
  |---|---|---|---|
  | Clients (crédit) | crédit inférieur aux factures (`e < 0`) | « Moins-payé de {montant} » | compte de frais bancaires désigné |
  | Clients (crédit) | crédit supérieur (`e > 0`) | « Trop-perçu de {montant} » | compte d'arrondi désigné |
  | Fournisseurs (débit) | débit supérieur aux factures (`e > 0`) | « Débit supérieur aux factures de {montant} » | compte de frais bancaires désigné |
  | Fournisseurs (débit) | débit inférieur (`e < 0`) | « Débit inférieur aux factures de {montant} » | compte d'arrondi désigné |

  **Liste du sélecteur** : `counterpartyAccounts(accounts, bankLedgerAccountId)` (actifs, imputables, hors banque de la
  transaction, `reconciliation-counterparties.ts:25-30`), avec `keepOfferedCounterparty` (:32-50) pour l'identifiant sorti
  de la liste — le patron de la correction du #1000 (35-7) ; `accounts` est celui que `ReconciliationProposals` passe déjà
  aux autres modales (:818, :831). Le compte désigné (`GET /company/invoice-settings`) ne préremplit **que s'il figure dans
  cette liste** (actif, imputable) ; sinon le champ reste **vide** — jamais une valeur invisible envoyée (P4, L1).
  **Choix gardé** (P4, L5) : un drapeau `choisiParUtilisateur` se pose à toute sélection manuelle (même si c'est le compte
  prérempli) et se lève si l'utilisateur vide le champ ; tant qu'il est posé, le compte reste **même si le signe change**
  ou si le sélecteur disparaît puis réapparaît pendant la session de la modale ; seul le **préremplissage** suit le signe.
  Le compte est **obligatoire** dès que `e ≠ 0`. Les deux sens sont admis pour les deux familles ; seul un **écart `e`
  au-delà du plafond** est refusé (`r` n'y est jamais soumis). Textes :
  - **écart** `e` : nul → « Aucun écart » (le sélecteur disparaît) ; dans le plafond → le libellé du tableau, **au
    centime**, avec le sélecteur ; au-delà → « Écart trop grand (au plus {maxDifference}) », *Enregistrer* inactif ;
  - **arrondi** `r` — une **information**, indépendante de l'écart, affichée dès que `r ≠ 0` : « Arrondi de {r} passé au
    compte d'arrondi » (quatre décimales) ; elle peut accompagner un écart (3 × 10.0050 payés 30.06 : « Trop-perçu de 0.05 »
    **et** « Arrondi de −0.0050 ») ou paraître seule (arrondi automatique, `e = 0`) ; si `r ≠ 0` et qu'aucun compte
    d'arrondi n'est désigné (`defaultRoundingAccountId` nul dans `GET /company/invoice-settings`), l'écran le dit **avant**
    l'envoi (« Aucun compte de différences d'arrondi n'est désigné : demandez à l'administrateur de le régler ») et
    *Enregistrer* reste inactif (P5, L2).
- **D3 — Les grandeurs de l'écran, en entiers (H2, C-43-21, C-43-25 définitive).** La modale convertit les montants en
  **entiers de 1/10 000** (aucun `Number` sur un montant, aucun flottant) :
  - *total imputé* = `Σ amountDueRaw` (brut) ; chaque facture imputée affiche son reste **brut** quand il porte une fraction
    de centime (quatre décimales), sinon au centime, pour que la somme affichée soit celle qui est comparée ;
  - *différence* `d = |montant| − total imputé`, signée (la `difference` de l'API) — **non affichée comme telle** ;
  - **écart** `e` = `d` arrondi au centime, demi-centime arrondi en s'éloignant de zéro (en entiers : `e = signe(d) ×
    ⌊(|d| + 50) / 100⌋ × 100`, comme `amount_due_to_centime`) ; **arrondi** `r = d − e` (`|r| ≤ 50`). L'arrondi automatique
    est le cas `e = 0`, soit `|d| < 50` ;
  - *Enregistrer* est actif si au moins **deux** factures sont imputées, si `r = 0` ou un compte d'arrondi est désigné, et,
    si `e ≠ 0`, si `|e| ≤ maxDifference` (valeur **lue de la réponse** : le réglage de la société, C-43-24 ; jamais écrite
    dans le code) **et** un compte est choisi. Avec `e = 0`, le plafond ne joue pas, même réglé à 0.00, comme le serveur
    (P5, L1).
  Après succès, le contrôle de cohérence compare `gap` à `e` et `roundingDifference` à `r` ; s'ils diffèrent (montants
  changés entre la recherche et l'envoi), un **toast dit les montants réellement passés** et sur quels comptes (P4, L3).
  Cohérent avec le tableau de `decision-difference.md` (43-2 AC6) :

  | Cas | `e` affiché | `r` affiché | Compte demandé |
  |---|---|---|---|
  | 2 × 10.0050 payés 20.01 | aucun écart | — | non |
  | 2 × 10.0050 payés 20.02 | trop-perçu de 0.01 | — | oui |
  | 500.00 + 300.00 payés 799.99 | moins-payé de 0.01 | — | oui |
  | 3 × 10.0050 payés 30.06 | trop-perçu de 0.05 | arrondi de −0.0050 | oui |
  | 10.0049 + 10.0000 payés 20.00 | aucun écart | arrondi de −0.0049 | non |
  | 10.0050 + 10.0000 payés 20.00 | moins-payé de 0.01 | arrondi de +0.0050 | oui |
- **D4 — Gardes de concurrence (H3).** La modale tient `session` (incrémentée à l'ouverture, à la fermeture, au
  changement de transaction et avant chaque soumission ; une réponse d'une session précédente n'agit jamais) et `loadGen`
  pour la recherche (délai anti-rebond de 250 ms, requête précédente annulée par `AbortController`, réponse hors d'ordre
  ignorée). Ces gardes vivent dans `reconciliation/modal-guards.ts`, testé seul, **employé par le nouveau composant
  seulement** : `LinkSettlementModal` n'est **pas** touché en 0.19.0 ; la dette existe déjà, **#1235**.
- **D5 — Refus nommés et refus qui ferment.** Un refus `failed[]` s'affiche « Facture {numéro} : {libellé} », le numéro
  venant de `details.invoiceNumber` (43-2) et, à défaut, de la liste imputée par `details.invoiceId` / `supplierInvoiceId`.
  - `RETIRER_SUR` — la facture n'est plus réglable (`RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `INVOICE_NOT_FOUND`,
    `SUPPLIER_INVOICE_NOT_FOUND`, `RECONCILIATION_SUPPLIER_PAYMENT_HOLD`) : retirée des imputées, la phrase la nomme, la
    recherche est relue (elle filtre la date comme le `POST`, une facture retirée ne revient pas).
  - `FERMER_SUR` — le refus porte sur **la transaction** et la sélection n'y peut rien (`BANK_TRANSACTION_NOT_FOUND`,
    `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`, `RECONCILIATION_TRANSACTION_NOT_CREDIT`,
    `RECONCILIATION_TRANSACTION_NOT_DEBIT`, `RECONCILIATION_CURRENCY_MISMATCH`) : la modale **affiche d'abord la phrase du
    refus en toast**, puis se ferme et relit la liste des propositions (P3, L3). De même pour une erreur de **recherche**
    `404 NOT_FOUND` (transaction supprimée avec son import) et `404 RECONCILIATION_TRANSACTION_NOT_PENDING` : toast,
    fermeture, relecture (P4, L4).
  - Refus du **compte de l'écart** (P4, M1) — la phrase s'affiche **sous le sélecteur**, qui reste ouvert, **un libellé par
    cause**, dans l'ordre des contrôles de la 43-2 (D9) :
    - `VALIDATION_ERROR` avec `reason: "counterparty_equals_bank_ledger"` → « Le compte choisi pour l'écart est le compte
      de la banque de cette transaction : choisissez-en un autre » (branche neuve de ce `case`, qui rend sinon « Erreur de
      validation ») ;
    - `ACCOUNT_NOT_FOUND` (`missingAccountIds`) → « Le compte choisi pour l'écart est introuvable ou archivé » ;
    - `ACCOUNT_NOT_POSTABLE` → « Le compte choisi pour l'écart n'est pas imputable » (numéro du compte) ;
    - `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, `role: "counterparty"`, **et** `details.accountId` égal au
      `differenceAccountId` envoyé (sans `bankAccountId`) → « Le compte choisi pour l'écart est le compte de créance (ou de
      dette) d'une facture du groupe » ;
    - `RECONCILIATION_GROUP_INVALID`, `reason: "difference_account_required"` → « Choisissez le compte de l'écart ».
  - Le **même** code `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` avec `role: "counterparty"` **et `bankAccountId`** (le compte
    de créance d'une facture est le compte de la banque — un défaut de **configuration**) garde le libellé existant et
    s'affiche avec les autres refus, **pas** sous le sélecteur (changer de compte n'y ferait rien).
  - Montants périmés (P4, L3) : sur `difference_account_required` ou `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, la
    modale **relit** les factures imputées (montants) avant d'afficher, puis montre l'écart recalculé.
  - Les autres refus laissent la sélection intacte, avec leur libellé existant (`PERIOD_LOCKED`,
    `LATER_FISCAL_YEAR_CLOSED`, `FISCAL_YEAR_INVALID`, `BANK_ACCOUNT_*`, `ROUNDING_ACCOUNT_NOT_CONFIGURED`,
    `RECONCILIATION_GROUP_INVALID` `too_many` / `duplicate`) — classement confirmé à l'étape 0 (P5, L4). Dans un groupe,
    `ACCOUNT_NOT_FOUND` et `ACCOUNT_NOT_POSTABLE` ne viennent **que** du compte de l'écart (l'écriture se crée avec
    `enforce_postable = false` pour le reste, 43-2) : c'est ce qui justifie leur affichage sous le sélecteur.
- **D6 — Côté fournisseur retirable (C-43-18).** Le débit (`supplier_invoice_group`) et ses tests vivent dans des commits à
  part, retirables si la 43-1b sort : sans eux, le bouton n'apparaît que sur un crédit.

## Critères d'acceptation

- **AC1** — Bouton selon D1 : présent sur un crédit CHF en attente ; absent sur une transaction rapprochée, rejetée, de
  montant nul, en EUR ; sur une ligne signalée « déjà comptabilisée », le dialogue de confirmation s'ouvre **avant** la
  modale, et l'annuler n'ouvre rien ; sur une ligne non signalée, la modale s'ouvre directement.
- **AC2** — D2, D2 bis et D3 : ajout, retrait, recherche, total imputé, écart et son sens, sélecteur du compte (apparition,
  préremplissage selon le sens et les réglages, choix gardé, obligation), bouton ; une facture imputée est passée à
  `exclude` (elle ne revient pas, et `hasMore` reste juste) ; `hasMore` → phrase qui invite à préciser la recherche ; aucun
  résultat → phrase qui le dit ; `direction: null` → « aucune facture à proposer pour cette transaction » ; chargement →
  état visible.
- **AC3** — D4 : réponse de recherche hors d'ordre ignorée ; fermeture pendant l'envoi sans effet sur une modale rouverte ;
  rouverture sur une autre transaction propre.
- **AC4** — *Enregistrer* envoie `POST /reconciliation/accept` (`invoice_group` ou `supplier_invoice_group`, avec
  `differenceAccountId` si l'écart au centime `e` est non nul) ; succès → toast, modale fermée, transaction
  retirée ; refus → D5. `failed-proposal-label.ts` **ajoute** les deux codes neufs de la 43-2 (`RECONCILIATION_GROUP_INVALID`
  par `reason`, dont `difference_account_required` ; `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`) : le test passe de **37 à
  39** codes, compte recompté depuis la source ; sa recette de comptage s'étend à `reconciliation_group.rs`. **Forme du
  groupe** : la fonction reçoit un discriminant, `failedProposalLabel(code, details, { group: true, differenceAccountId })`,
  fourni par la modale (seul appelant du cas groupe) ; il choisit la forme propre au groupe de
  `ROUNDING_ACCOUNT_NOT_CONFIGURED`, des libellés du compte de l'écart (D5 : `VALIDATION_ERROR` /
  `counterparty_equals_bank_ledger`, `ACCOUNT_NOT_FOUND`, `ACCOUNT_NOT_POSTABLE`, `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`
  avec `details.accountId === differenceAccountId`), et laisse le cas configuration (`bankAccountId` présent) au libellé
  existant ; test de **chaque** forme et de **chaque** cause, avec et sans discriminant (P3 L2, P4 M1). Les erreurs de la
  recherche (`404 NOT_FOUND`, `404 RECONCILIATION_TRANSACTION_NOT_PENDING`, `409 BANK_ACCOUNT_CLOSED`, `412
  BANK_ACCOUNT_NOT_CONFIGURED`) s'affichent selon D5 (les deux `404` ferment) ou comme `loadError`, patron de
  `LinkSettlementModal`.
- **AC5** — Annulation : `CancelReconciliationDialog`, `kind = "grouped_settlement"` → les **N** factures listées
  (`groupedInvoices`) redeviendront ouvertes, l'écriture sera contre-passée (écart compris, compte nommé : `gap`,
  `differenceAccount`, dont la 43-2 écrit la source — « Vue et annulation » : `gap` recalculé par `group_difference`,
  compte relu par `grouped_difference_account_id` puis numéro et nom lus du compte ; l'écran les affiche tels quels, sans
  recalcul) ; pour un groupe **fournisseur**, ces champs dépendent de l'AC6 bis (vue) et de l'AC6 (dé-rapprochement) de la
  **43-1b** — l'étape 0 le vérifie ; un refus d'annulation par facture nomme la facture (`cancelBlockedInvoice`) avec les textes existants
  mis au pluriel du groupe (« Une des factures de ce rapprochement, {numéro}, a été créditée… ») ; si la 43-2 retire
  `cancelBlockedInvoice` (son ordre de retrait), la limite est écrite au manuel et une issue s'ouvre.
- **AC6** — i18n : quatre locales (`fr-CH` rédigé ; `de-CH`, `it-CH`, `en-CH` traduits, glossaire
  `docs/i18n-glossaire.md`) ; les clés s'ajoutent **avant** le composant qui les lit ; `lint-i18n-ownership` ; `sitesTotal`
  recompté depuis la source (3086 à la tête).
- **AC7** — Manuel utilisateur (`docs/manual/fr/user-manual.tex`, `.tex` seulement ; § `sec:reconciliation` et
  `sec:annuler-rapprochement`) : « Régler plusieurs factures par une transaction » — gestes ; **écart** : dans les deux
  sens (clients : moins-payé, trop-perçu ; fournisseurs : débit supérieur ou inférieur aux factures), dans la limite du
  plafond réglé par l'administrateur (5.00 par défaut), comptabilisé sur le compte que l'on choisit (prérempli par le
  compte de frais bancaires ou d'arrondi désigné, s'il est utilisable) ; un écart minime, dû aux fractions de centime des
  factures, part seul au compte d'arrondi ; 50
  factures au plus ; confirmation sur une ligne signalée « déjà comptabilisée » comme les autres gestes ; un règlement et un
  lettrage par facture ; annulation en entier seulement ; limites : la ligne de banque et l'écart d'un règlement groupé ne
  portent pas de compte analytique ; le changement de compte analytique d'une facture réglée en groupe est refusé en 0.19.0
  (43-6). **Minimum vital si le temps manque** (P3, L5) : la limite de la 43-6 et l'annulation « en entier seulement ».
  **Contrôle** : `make fr` (ou `latexmk`) compile le manuel sans erreur ni `\ref` indéfini, **sans commiter le PDF** ; le
  contrôle aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`) porte sur ce PDF jetable ; le prompt de revue P1 nomme l'axe
  « manuel contre code, PDF compris ».
- **AC8** — Captures (thème clair, base de démonstration jetable), une par état d'AC2 à AC5 : modale vide, chargement,
  aucun résultat, aucune facture à proposer, `hasMore`, une facture, deux factures de tiers différents avec moins-payé et
  compte prérempli, trop-perçu, débit supérieur aux factures (D6), écart nul, arrondi seul passé au compte d'arrondi, écart
  et arrondi ensemble (30.06), arrondi
  sans compte d'arrondi désigné, écart
  trop grand, compte désigné inutilisable (champ vide), compte refusé (une capture par cause), facture refusée nommée et retirée, confirmation
  « déjà comptabilisée », côté fournisseur (D6), dialogue d'annulation d'un groupe et son refus nommé — dans
  `/home/gcorbaz/devel/kesh-gate-logs/e433/captures/`.

## Tests

- **Vitest** :
  - `GroupSettlementModal.test.ts` : ajout, retrait, `exclude`, bouton (une facture ; écart trop grand ; écart sans compte ;
    admis) ; calcul en entiers : **`0.10 + 0.20`** (c'est lui qui rougit sous la mutation `Number`) ; **décomposition de
    l'écart (C-43-25 définitive)** : **les six lignes du tableau de D3**, chacune avec `e`, `r` et « compte demandé » — dont
    20.02 (rougit si l'on somme `amountDue` au lieu de `amountDueRaw`), 30.06 (écart **et** arrondi affichés ensemble),
    10.0049 (arrondi seul, sans compte) et 10.0050 (écart de 0.01 **et** arrondi de +0.0050 : borne du demi-centime) ;
    arrondi automatique avec un plafond réglé à **0.00** → *Enregistrer* actif (P5, L1) ; `r ≠ 0` sans compte d'arrondi
    désigné → phrase avant l'envoi, *Enregistrer* inactif, **même si `e = 0`** (P5, L2) ; libellés et
    préremplissage **des quatre cas** du tableau de
    D2 bis ; compte désigné archivé ou non imputable → champ vide ; choix gardé : sélection manuelle conservée quand le signe
    change, quand le sélecteur disparaît puis réapparaît, levée quand le champ est vidé ; obligation ; contrôle de cohérence
    de `gap` contre `e` et de `roundingDifference` contre `r` (aucun toast s'ils concordent), toast s'ils diffèrent ; `maxDifference` lu de la réponse (réglage à 1.00 dans la maquette) ; corps envoyé
    pour chaque famille, `differenceAccountId` compris ; refus nommé et retiré (une facture parmi trois) ; refus de
    transaction, de sens, de devise et `404` de recherche → toast puis fermeture ; **refus du compte, une cause par test**
    (banque, introuvable, non imputable, compte de tiers du groupe) sous le sélecteur, et le cas configuration
    (`bankAccountId` présent) **hors** du sélecteur ; relecture des montants sur `difference_account_required` ;
    `direction: null` → état dédié ; zéro facture imputée à l'ouverture ; gardes D4 (réponse hors d'ordre, fermeture
    pendant l'envoi) ;
  - `modal-guards.test.ts` (module neuf) ; `LinkSettlementModal` n'est pas touché ;
  - `ReconciliationProposals.test.ts` : bouton selon `canLink` (EUR, montant nul) ; ligne signalée → dialogue avant la
    modale ; ligne non signalée → modale ; sonde négative ;
  - `CancelReconciliationDialog.test.ts` : `grouped_settlement`, écart et compte, refus nommé (deux motifs) ;
  - `failed-proposal-label.test.ts` (39 codes, discriminant `group`) ; `reconciliation.api.test.ts` (recherche, `exclude`,
    `direction: null`) ; désérialisation de l'élément d'`accepted[]` d'un groupe (43-2).
- **E2E** (`frontend/tests/e2e/reconciliation-group.spec.ts`, neuf) — préconditions et nettoyage : helper
  `setDesignatedAccountsViaApi` (comptes d'arrondi **et** de frais bancaires) ajouté à `helpers/api-fixtures.ts`, réglages
  **restaurés en `finally`** ; crédit synthétique de montant et de référence **uniques à chaque exécution** (patron de
  `reconciliation-link.spec.ts`) ; en `finally` : groupe annulé, factures annulées, import supprimé. Cas : un crédit réglant
  deux factures de **deux clients** avec un moins-payé de 0.40 sur le compte prérempli, puis son annulation (factures
  rouvertes) ; un trop-perçu sur un compte choisi ; deux restes sous le centime payés 20.01 (aucun écart, aucun compte
  demandé) ; un écart trop grand laisse *Enregistrer*
  inactif ; un compte de créance choisi pour l'écart est refusé et la phrase le dit ; une ligne signalée « déjà
  comptabilisée » demande confirmation (cas ajouté ici ou à `reconciliation-already-booked.spec.ts`) ; *(commit D6)* un
  débit réglant deux factures fournisseur. Sonde négative : la transaction rapprochée n'offre plus le bouton.
- **Rouges d'abord** : `kesh-gate-logs/e433/rouges.log`. **Mutations** (`kesh-gate-logs/e433/mut/`) : retirer « au moins
  deux » ; retirer le contrôle du plafond ; remplacer les entiers par `Number` (**le Vitest `0.10 + 0.20` rougit**) ;
  **sommer `amountDue` au lieu de `amountDueRaw`** (le Vitest 20.02 ne demande plus de compte : il rougit) ; **tronquer au
  lieu d'arrondir au centime** (`e` du cas 10.0050 vaut 0 au lieu de −0.01 : il rougit) ; **arrondir le demi-centime vers
  zéro** (même cas : il rougit) ; **envoyer tout `d` comme écart** (le cas 10.0049 demande un compte : il rougit ; le cas
  30.06 affiche 0.0450) ; **retirer l'affichage de `r` quand `e ≠ 0`** (le cas 30.06 rougit) ; préremplir sans vérifier que le compte figure dans
  la liste (le Vitest du compte désigné archivé rougit) ; retirer l'obligation du compte quand l'écart est non nul ; retirer la branche `group` de `guard`
  (AC1 rougit) ; retirer la garde de session (AC3 rougit).

## Planning et ordre de retrait (C-43-23)

- Types (`reconciliation.types.ts`) **dès T1**, sur le contrat de la 43-2 ; Vitest tenus sur ces types ; relus à l'étape 0
  et à chaque commit de la 43-2.
- Le spec E2E s'écrit et se joue **mercredi soir** sur une branche de travail réunie (43-1a jusqu'à son T6 et 43-2 jusqu'à
  son T2 au moins), puis jeudi sur la branche réunie définitive. Tout commit de code après jeudi 12 h relance l'E2E complet
  (prévoir 1 h 30).
- Si le temps manque, dans cet ordre : commits fournisseur (D6) ; captures non essentielles ; le refus nommé de
  `cancelBlockedInvoice` dans le dialogue d'annulation (AC5, limite écrite au manuel) ; le manuel réduit à son minimum vital
  (AC7). **Jamais l'E2E**, ni le sélecteur du compte de l'écart, ni les deux limites du manuel ; les traductions de/it/en ne
  se retirent pas (la garde `i18n-keys.test.ts` exige les quatre locales) : elles s'écrivent avec les clés, avant le
  composant.

## Tâches

- [x] **T0** — Étape 0 : LOW restants (Dev Notes) ; contrat de la 43-2 relu (fiche, puis code s'il existe).
- [x] **T1** — Types et appels (`reconciliation.types.ts`, `reconciliation.api.ts`, lecture des réglages de facturation).
- [x] **T2** — Clés i18n des quatre locales (AC6), avant les composants.
- [x] **T3** — `modal-guards.ts` neuf (D4), employé par le nouveau composant seulement ; `LinkSettlementModal` non touché.
- [x] **T4** — `GroupSettlementModal.svelte` (avec le sélecteur de compte, D2 bis), bouton, garde `group` (AC1 à AC4),
  côté client.
- [x] **T5** — Dialogue d'annulation (AC5). — [x] **T6** — Manuel et compilation jetable (AC7).
- [x] **T7** — Commits D6 (côté fournisseur).
- [x] **T8** — E2E (mercredi soir sur la branche de travail réunie, jeudi sur la branche définitive) ; captures (AC8).
  *Joué sur la branche réunie (43-2 `d20095425`, épique `d54bad428`) ; à rejouer sur la tête définitive de la 43-2.*
- [x] **T9** — Gates frontend complets ; E2E complet à la clôture ; Dev Agent Record (#1235 citée) ; prompt de revue P1.

## Note de version

- **Ajouté** — Réconciliation : « Régler plusieurs factures… » — sur une ligne du relevé, ajoutez les factures une à une ;
  le total et l'écart s'affichent ; un petit moins-payé ou trop-perçu se comptabilise sur le compte que vous choisissez
  (prérempli par le compte de frais bancaires ou d'arrondi), dans la limite réglée par l'administrateur ; Kesh règle et
  lettre chaque facture (refs #823).

## Dev Notes

### Modules touchés

`frontend/src/lib/features/reconciliation` (modale, garde, dialogue, types, appels, `modal-guards.ts`), catalogues i18n,
`frontend/tests/e2e` (spec et helper), manuel utilisateur. `AccountAutocomplete.svelte` est employé sans être modifié ;
`ContactPicker.svelte` n'est pas employé.

### Garde CHF et multi-devise

Le bouton suit `canLink` (CHF) et le serveur refuse une transaction non CHF (43-2, D6). Quand le multi-devise arrivera,
cette condition est le **seul** point à élargir côté écran ; la phrase est au fichier d'épique (§ travaux parallèles).

### LOW à l'étape 0

- **L-6 (P2)** : nom sans jeton de 3 caractères ou plus : servi par le repli `LIKE` (43-2) ; le test réel est celui de la
  43-2 (AC5, « BL SA »).
- **L-4 (P2)** : contraste du texte de l'écart vérifié par `helpers/contrast.ts` dans l'E2E.
- **L4 (P3)** : citations corrigées (`RELIRE_SUR` :87-91 ; en-tête).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement de la 43-3, clé `e433`, voie 2), 2026-11-17.

### Completion Notes List

*Relevé daté du 2026-11-17, fin de l'étape 1 (tout ce qui ne dépend pas de l'API réelle). L'étape 2 — E2E et
captures — attend la branche de la 43-2 ; ce relevé ne déclare rien qui n'ait tourné.*

- **Étape 0** (lignes lues citées) :
  - L-6 (P2) — repli `LIKE` d'un nom sans jeton de trois caractères : présent dans la 43-2,
    `crates/kesh-db/src/repositories/reconciliation_group_candidates.rs:21` (« repli `LIKE` quand le terme n'a aucun
    jeton de trois caractères ») et `:152` (`AND (c.name LIKE `) ; le test réel reste celui de la 43-2.
  - L-4 (P2) — contraste du texte de l'écart : vérifié dans l'E2E (`reconciliation-group.spec.ts`, couleurs calculées
    passées par un canevas, `helpers/contrast.ts`), qui attend l'API.
  - L4 (P3) — `RELIRE_SUR` lu à `LinkSettlementModal.svelte:87` (`const RELIRE_SUR = new Set([`) : citation juste.
  - P5 L4 — les refus non classés (`PERIOD_LOCKED`, `BANK_ACCOUNT_*`, `ROUNDING_ACCOUNT_NOT_CONFIGURED`,
    `too_many` / `duplicate`…) laissent la sélection intacte, libellé existant : `handleRefusal` (modale), testé sur
    `PERIOD_LOCKED`.
  - LOW-8 (passe contre `t3.md`) — sources de la vue dans le code de la 43-2 (`70f74f964`) :
    `routes/reconciliation_group.rs:527` (`gap` ← recalculé par `group_difference`), `:530` (`differenceAccount` ←
    `view.grouped_difference_account_id`), `:533` (`cancelBlockedInvoice`) ; l'écran les affiche tels quels.
  - AC5, groupe fournisseur : la vue et le dé-rapprochement fournisseur dépendent de la 43-1b (`ready-for-dev` au
    `sprint-status.yaml:1113`) ; l'écran les traite déjà (D6), sans test réel avant elle.
  - Contrat de la 43-2 relu au code (`70f74f964`) : structures `GroupCandidateResponse`, `GroupCandidatesResponse`,
    `GroupAcceptedFields`, refus de `group_refusal_to_failed_proposal`, `CancelReconciliationResponse` (`kind`,
    `cancelledInvoices`) ; types alignés. 39 codes : union des littéraux de `reconciliation.rs` (34) et des codes de la
    fonction de traduction de `reconciliation_group.rs`, plus les trois de `DbError` ; deux neufs.
- **Fait** : types et appel (`getGroupCandidates`) ; grandeurs en `bigint` de 1/10 000 (`group-settlement.ts`) ; gardes
  (`modal-guards.ts`, fabriques — C-e433-6) ; libellés des refus (39 codes, discriminant `group`) ; modale, bouton,
  geste `group` dans la garde « déjà comptabilisée » ; dialogue d'annulation d'un groupe ; i18n des quatre locales ;
  manuel (`.tex`, compilé hors du dépôt) ; côté fournisseur en commits à part (C-e433-7) ; E2E écrit, en attente.
  `LinkSettlementModal` n'est pas touché (dette #1235).
- **Compteurs relevés par les gardes** : `sitesTotal` 3086 → 3146 (« expected 3146 to be 3086 »,
  `kesh-gate-logs/e433/vitest-i18n-releve-1.log`), dont 3142 sans le côté fournisseur ; candidates de
  `i18n-libelle-en-dur.test.ts` 95 → 104, `ecartee` 14 → 18, `conforme` 81 → 86 (`vitest-i18n-releve-2.log`).
- **Gates tels qu'ils ont tourné** (étape 1) : frontend complet au commit `81e4b4a2b` — `npm run check` 0 erreur,
  `lint-i18n-ownership` PASS, `npm run test:unit` **4071 tests, 249 fichiers**, `npm run build` vert
  (`kesh-gate-logs/e433/front-complet-etape1.log`) ; `cargo test -p kesh-i18n` 41 passés (parité des locales,
  `cargo-i18n-1.log`). **Non lancés** : `test-fast --ci`, `gate-ci`, E2E (au passage en revue et à la clôture).
- **Mutations** (`kesh-gate-logs/e433/mut/`, script `mutations.py`) : M1 à M11 et M12b tuées ; M12 (un seul site de la
  garde de session) survit, le même contrôle se répétant dans `handleRefusal` — équivalente sur ce chemin ; M12b
  (garde entière neutralisée) est tuée par deux tests (`mutations-1.log`, `mutations-2.log`).
- **Rouges** : tests écrits après le code, dans la même séance — écart déclaré (`kesh-gate-logs/e433/rouges.log`).
- **Écarts à la consigne Edit** : aucun sur le code, les tests ni les fiches ; le script de mutations écrit et restaure
  (`git restore --worktree` puis `touch`), comme la consigne le permet. `git add` pour les fichiers neufs.
- **Limite connue** : l'abandon de la recherche est local (C-e433-1) — `apiClient` ne prend pas de signal.

*Relevé daté du 2026-11-18, étape 2 et passage en revue.*

- **Fusions** : la 43-2 (`d20095425`, avec la 43-1a, la 43-1b et l'Epic 37 ; 95 migrations) par `a179b687d`, puis
  l'épique (`d54bad428`, 43-7 comprise) par `7ac22c02d`. Conflits résolus en gardant les deux côtés ; compteurs relevés
  par la garde, jamais additionnés : `sitesTotal` 3450 puis 3456, puis **3457** avec l'avis du lot
  (`fusion-432-releve-1.log`, `fusion-epic-releve-1.log`, `front-complet-review.log`) ; candidates de la garde des libellés
  en dur **128** (22 écartées, 106 conformes) ; test des libellés de refus à **48** codes. Squash sans conflit.
- **Raccords Epic 37** (C-e433-10) : refus `ACCOUNT_IN_FOREIGN_CURRENCY` libellé (clé globale ; forme du groupe dans les
  quatre locales, sous le sélecteur) ; sélecteur `excludeForeignCurrency={true}` ; Vitest : compte en devise ni offert
  ni prérempli ; garde `fx-account-filter-inventory` verte.
- **Revue P4 de la 43-2, P4-1** (`8664d51fd`) : `heldByPaymentBatch` de la recherche d'un débit → avis qui nomme le lot et
  dit de l'annuler d'abord, distinct de « aucun résultat » ; quatre locales ; Vitest ; état intercepté et capture
  `43-3-retenu-par-un-lot.png` ; manuel (cas de liste vide). Le champ n'arrive dans la branche qu'avec la tête définitive
  de la 43-2 : avant elle, il est absent (facultatif au type) et l'écran n'affiche rien de plus.
- **E2E** : `fixme` retirés (client et D6) ; spec `reconciliation-group.spec.ts` **12 tests verts** contre l'API réelle
  (`e2e-group-4.log`, voie 2, port 3433). Points vérifiés au code : `totalAmount` est le HT (reste dû lu sur la fiche) ;
  le seed pose l'arrondi des pièces à 5 centimes (levé pendant la spec, restauré) ; `line_total` garde quatre décimales
  (0.5 × 20.01 = 10.0050) ; `/reverse` sans corps. États fugaces par une réponse interceptée sur un décor réel
  (C-e433-11). **26 captures** dans `kesh-gate-logs/e433/captures/`, thème clair.
- **Manuel** : `sec:reglement-groupe` ; le lot pain.001 débité en une ligne reste à rejeter (textes rétablis tels
  quels, P3-1 de la revue de la 43-2) et un paragraphe le distingue des factures ouvertes hors lot ; liste vide. PDF
  compilé hors du dépôt (`kesh-gate-logs/e433/manuel/user-manual.pdf`), aucune référence indéfinie, contrôle aplati.
- **Gates de passage en revue**, bases remises à neuf (`kesh_e433`, `kesh_e2e_e433` : 95 migrations) :
  - `test-fast --ci` (voie 2) au commit `7ac22c02d` : **6069 passés, 13 ignorés** (`test-fast-review.log`) ; un premier
    passage avait échoué sur un disque plein, sans rapport avec le code (`alerte.md`) ;
  - `gate-ci kesh_ci_e433` (voie 1) au commit `7ac22c02d` : **6086 passés, 0 échoué, 17 ignorés** (`gate-ci-review.log`)
    — les quatre catalogues portaient déjà, non commitée, la clé de l'avis du lot (une ligne de plus dans chacun) ;
  - frontend complet au commit `8664d51fd` : `check` 0 erreur, `lint-i18n-ownership` PASS, **4519 Vitest (275 fichiers)**,
    `build` vert (`front-complet-review.log`) ;
  - E2E complet au commit `8664d51fd`, base E2E recréée : **471 passés, 7 échoués, 17 ignorés** (`e2e-complet-review.log`) ;
    les sept échecs sont exactement ceux de la KF-029 (#97) de `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`,
    `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; aucun autre.
- **À la clôture** : reprendre la tête définitive de la 43-2 (champ `heldByPaymentBatch`), rejouer `gate-ci` et l'E2E
  complet.

### File List

- `frontend/src/lib/features/reconciliation/GroupSettlementModal.svelte` (neuf) et `.test.ts` (neuf)
- `frontend/src/lib/features/reconciliation/group-settlement.ts` (neuf) et `.test.ts` (neuf)
- `frontend/src/lib/features/reconciliation/modal-guards.ts` (neuf) et `.test.ts` (neuf)
- `frontend/src/lib/features/reconciliation/ModalSuccessStubGroup.test.svelte` (neuf)
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts`, `reconciliation.api.ts`, `reconciliation.api.test.ts`
- `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`, `failed-proposal-label.test.ts`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `ReconciliationProposals.test.ts`
- `frontend/src/lib/features/reconciliation/CancelReconciliationDialog.svelte`, `CancelReconciliationDialog.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts`, `reconciliation-cancel.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`, `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`
- `frontend/tests/e2e/reconciliation-group.spec.ts` (neuf), `frontend/tests/e2e/helpers/api-fixtures.ts`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`
- `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e433.md` (neuf)

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43).
- 2026-11-17 (soir) — Révisée : interface comme bexio, tiers mixtes, écart borné, côté fournisseur retirable.
- 2026-11-17 (nuit) — Remédiée après la validation P1 (0/3/6/8) : garde « déjà comptabilisée », montants bruts, gardes
  de session et de génération, `canLink`, refus nommés, annulation d'un groupe, E2E, planning, manuel.
- 2026-11-18 — Remédiée après la validation P2 (0/0/1/7) : grandeurs nommées (total imputé = Σ `amountDue`, « Écart »),
  libellés, refus de transaction, borne de date, `loadError`, `LinkSettlementModal` non touché, citations, critère du repli.
- 2026-11-18 (P3 et décision de Guy) — Remédiée après la validation P3 (0/0/1/6) et sur la décision de Guy du 2026-11-17 :
  sélecteur du compte de l'écart sous « Écart », prérempli (frais bancaires pour un moins-payé, arrondi pour un
  trop-perçu), obligatoire, libellé « Moins-payé » / « Trop-perçu », deux sens admis, refus au-delà du plafond seulement ;
  contrôle sur `gap` ; M1 (rôles réels, `role: "counterparty"` pour le compte choisi, plus de `409` de recherche, état
  « aucune facture à proposer ») ; L1 (mutation `Number` attribuée à `0.10 + 0.20`, mutation « sommer `amountDueRaw` »), L2
  (discriminant `{ group: true }`), L3 (refus de sens et de devise : toast puis fermeture ; recherche `NOT_PENDING` de
  même), L4 (citations, en-tête), L5 (minimum vital du manuel), L6 (#1235) ; ordre de retrait complété ; 39 codes.
- 2026-11-18 (P4) — Remédiée après la validation P4 (0/0/1/6) et sur deux décisions de l'orchestrateur (arrondi automatique
  C-43-25 ; libellés et préremplissage par famille C-43-26) : M1 (un libellé par cause sous le sélecteur, dont
  `counterparty_equals_bank_ledger` et `ACCOUNT_NOT_FOUND` ; discriminant `details.accountId === differenceAccountId`, cas
  configuration hors du sélecteur) ; L1 (liste `counterpartyAccounts` et `keepOfferedCounterparty`, préremplissage seulement
  d'un compte utilisable), L2 (libellés fournisseur « Débit supérieur / inférieur aux factures », préremplissage par
  famille), L3 (montants relus, toast si `gap` diffère), L4 (`404 NOT_FOUND` ferme), L5 (choix gardé défini), L6 (réglé par
  l'arrondi automatique) ; tests et mutations recalculés (trois restes à 10.0050 pour la mutation `amountDueRaw`).
- 2026-11-18 (P5) — **Clôture de la validation.** Tendance : P1 Sonnet 0/3/6/8 → P2 Opus 0/0/1/7 → P3 Sonnet 0/0/1/6 → P4
  Opus 0/0/1/6 → **P5 Sonnet 0/0/0/5** : aucune passe au-dessus de LOW à la cinquième, la boucle se clôt. Modèles : Sonnet
  5.5 (P1, P3, P5), Opus 5.5 (P2, P4) ; remédiations par l'agent de rédaction (Opus 5.5). LOW de la P5 appliqués : L1
  (*Enregistrer* actif dans la zone d'arrondi quel que soit le plafond), L2 (« Arrondi de … passé au compte d'arrondi » au
  lieu de « Aucun écart » ; compte d'arrondi non désigné signalé avant l'envoi), L3 (seuil en entiers `|d| < 50`, bornes
  0.0049 et 0.0050 testées), L4 (autres refus classés, `ACCOUNT_NOT_FOUND` / `ACCOUNT_NOT_POSTABLE` justifiés sous le
  sélecteur) ; L5 (calendrier) sans objet. **Correction de l'orchestrateur sur C-43-25**, appliquée dans le même commit :
  la différence se calcule **au brut** (`d = |montant| − Σ amountDueRaw`, total imputé au brut) ; arrondi automatique
  seulement si `|d| < 0.005`, sinon `d` entier sur le compte choisi ; tests recalculés (20.01 aucun écart ; 20.02 et 799.99
  compte demandé ; 30.06 → 0.0450) ; mutations : « sommer `amountDue` », « seuil retiré », « `≤` au lieu de `<` »,
  « seuil en centimes ». Cette correction change une règle : une passe ciblée la relit. Statut : `ready-for-dev`.
- 2026-11-18 (C-43-25 définitive) — Décomposition de l'écart selon `decision-difference.md`, § « Décomposition de l'écart » :
  l'écran calcule `d` au brut, puis `e` (au centime, demi-centime en s'éloignant de zéro) et `r = d − e` ; il affiche
  l'**écart `e` au centime** (sélecteur, libellé du sens, plafond sur `|e|` seulement) et l'**arrondi `r` comme une
  information**, seul ou avec l'écart ; compte d'arrondi désigné requis si `r ≠ 0` ; tableau des six cas en D3, tests et
  mutations refaits (troncature, demi-centime vers zéro, tout `d` comme écart, `r` masqué). Remplace l'entrée précédente
  sur ce point (« `d` entier sur le compte choisi »). La passe ciblée relit ce commit et le précédent.
- 2026-11-18 (fiches contre le contrat `t3.md`) — LOW-8 : AC5 renvoie à la source des champs de la vue écrite par la 43-2
  (MEDIUM-3) et à la dépendance du groupe fournisseur envers l'AC6 bis et l'AC6 de la 43-1b (HIGH-1).
- 2026-11-17 (développement, étape 1) — Clôture de la validation reprise telle quelle (P1 Sonnet 0/3/6/8 → P5 Sonnet
  0/0/0/5, puis passes ciblées sur C-43-25 et contre `t3.md`). Étape 0 : LOW de la P5 déjà intégrés à la fiche ; L-6,
  L-4, L4, P5 L4 et LOW-8 contrôlés, lignes lues au Dev Agent Record. Développement de tout ce qui ne dépend pas de l'API
  réelle (T0 à T7) : commits `be7c0c2eb` (types, appel), `388d3bd65` (i18n), `cf908bfd8` (grandeurs, gardes),
  `c11469b2d` (libellés, 39 codes), `b2bd9078e` (modale, bouton, garde), `4053741c7` (annulation), `fefaed04f` (D6),
  `951263aeb` (manuel), `ed68aea4d` et `81e4b4a2b` (E2E en attente, dont D6). Choix C-e433-1 à 9. Statut :
  `in-progress` — E2E et captures attendent l'API de la 43-2.
