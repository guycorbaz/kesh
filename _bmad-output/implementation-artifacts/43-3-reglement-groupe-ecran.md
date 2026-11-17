# Story 43-3 : un règlement pour plusieurs factures — l'écran de réconciliation, façon bexio (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`, code lu à
`8b50fd195` ; révisée le soir même (décisions de Guy : interface comme bexio, tiers mixtes, écart), puis **remédiée dans la
nuit** après sa validation P1 (`kesh-gate-logs/e43/validation-43-3-p1.md`, 0/3/6/8 : H1-H3, M1-M6 appliqués ; LOW d'une
ligne appliqués, les autres en Dev Notes, étape 0) et alignée sur la 43-2 remédiée, la remédiation P2 de la 43-1a et Q4
(plafond réglable). Release **0.19.0**. **Commence tout de suite** : types et Vitest sur le contrat de la fiche 43-2 ;
l'**E2E** se joue sur la branche réunie (43-1a et 43-2 fusionnées). Frontend et manuel utilisateur ; aucune migration.
Décisions : C-43-14 à C-43-24 (fichier d'épique), qui font foi ; la règle de différence (C-43-16) est une proposition en
attente de Guy. Validation : passe 2 à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable, je choisis une ligne du relevé, j'y ajoute des factures une à une en les cherchant, je vois à
chaque ajout le total imputé et la différence avec le montant du relevé, et j'enregistre — puis, si je me suis trompé,
j'annule ce rapprochement en voyant les factures qu'il rouvrira.

## Ce qui se lit au code (`8b50fd195`)

- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - modales `ManualMatchModal`, `TransactionSplitModal`, `LinkSettlementModal` (import :78-80) ;
  - `canLink(p)` : montant non nul **et** `currency === 'CHF'` (:274-276) ;
  - **garde « déjà comptabilisée »** (Stories 34-4, 35-4) : commentaire :296-301, `type GuardedGesture` (:302-306 :
    `accept`, `reject`, `manual`, `split`), `guard()` (:316), `runGesture()` (`switch` fermé par `never`),
    `AlreadyBookedConfirmDialog`, `confirmKind = 'write' | 'reject'` ; l'en-tête du fichier énumère les gestes gardés ;
  - bouton « Rattacher à une écriture existante » (:747, `data-testid="link-settlement-button"`).
- `LinkSettlementModal.svelte` : gardes `loadGen` et `session` (:73-82, revue P1 de la 29-8, C-29-8-37), `RELIRE_SUR`
  (:84-90) — le patron de concurrence à reprendre.
- `CancelReconciliationDialog.svelte`, `reconciliation-cancel.ts` (textes au singulier : « La facture de ce
  rapprochement… »), `reconciliation.types.ts` (`kind` :204, `candidateType` :24, `AcceptedProposal` :134-141,
  `cancelBlockedDocumentId` :233).
- `failed-proposal-label.ts` (lit `role`, `claim`, `reason`, `fiscalYearName`, `tolerance_days` ; aucun numéro de facture)
  et `failed-proposal-label.test.ts` (« couvre les 37 codes », :95-96 ; recette de comptage en tête, :6-16, par `grep` sur
  `reconciliation.rs` seul).
- `frontend/src/lib/shared/i18n-keys.test.ts` : `sitesTotal` = 3086 (:1040).
- E2E : `frontend/tests/e2e/helpers/api-fixtures.ts` (contacts, factures validées, compte bancaire lié ; **aucun** helper
  des comptes désignés d'arrondi ou de frais), `helpers/contrast.ts` ; specs voisines `reconciliation-link.spec.ts`,
  `reconciliation-supplier.spec.ts` (nettoyage en `finally`), `reconciliation-cancel.spec.ts`,
  `reconciliation-already-booked.spec.ts`.
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
  **total imputé**, **différence**, chacun avec un **texte**, jamais la seule couleur. Textes de la différence (C-43-16,
  proposition en attente de Guy ; la décomposition suit la 43-2) :
  - nulle → « Différence nulle » ;
  - frais admis (client : moins reçu ; fournisseur : plus payé), dans le plafond → « {montant} de frais bancaires, passés au
    compte de frais bancaires ; les factures seront soldées en entier » ;
  - au-delà du plafond → « Différence trop grande (au plus {maxDifference}) » ;
  - sens refusé (client : trop reçu ; fournisseur : moins payé) → « {montant} de plus que les factures : non admis » /
    « {montant} de moins que les factures : non admis » ;
  - fractions de centime → rien d'affiché à part (elles passent au compte d'arrondi, comme une facture seule).
  Aucune présélection : la modale s'ouvre avec **zéro** facture imputée ; le nom de la contrepartie préremplit seulement la
  recherche (un nom sans jeton de 3 caractères est servi par le repli `LIKE` de la 43-2).
- **D3 — Montants bruts, calcul en entiers de 1/10 000 (H2, C-43-21).** La modale reçoit `amountDueRaw` (chaîne à quatre
  décimales) et le montant du relevé ; elle convertit chaque montant en **entier de 1/10 000** (aucun `Number` sur un
  montant, aucun flottant), calcule les apports au centime (`MidpointAwayFromZero`, comme `amount_due_to_centime`), les
  frais `e = |montant| − Σ apports au centime` et affiche selon D2. *Enregistrer* est actif si au moins **deux** factures
  sont imputées, si le sens de `e` est admis et si `|e| ≤ maxDifference` (valeur **lue de la réponse** : le réglage de la
  société, C-43-24 ; jamais écrite dans le code). Après succès, `difference`, `roundingDifference` et `bankFees` rendus par
  le serveur sont comparés à ceux de l'écran ; un écart est journalisé en console (contrôle de cohérence), non montré.
- **D4 — Gardes de concurrence (H3).** La modale reprend le patron de `LinkSettlementModal` : `session` (incrémentée à
  l'ouverture, à la fermeture, au changement de transaction et avant chaque soumission ; une réponse d'une session
  précédente n'agit jamais), `loadGen` pour la recherche (délai anti-rebond de 250 ms, requête précédente annulée par
  `AbortController`, réponse hors d'ordre ignorée). Le code commun aux deux modales s'**extrait** dans
  `reconciliation/modal-guards.ts`, testé seul, plutôt que d'être recopié.
- **D5 — Refus nommés (M2).** Un refus `failed[]` s'affiche « Facture {numéro} : {libellé} », le numéro venant de
  `details.invoiceNumber` (43-2) et, à défaut, de la liste imputée par `details.invoiceId` / `supplierInvoiceId`. Une
  facture refusée parce qu'elle n'est plus réglable (`RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `INVOICE_NOT_FOUND`,
  `SUPPLIER_INVOICE_NOT_FOUND`, `RECONCILIATION_SUPPLIER_PAYMENT_HOLD`) est **retirée des imputées**, la phrase la nomme, la
  recherche est relue ; les autres refus laissent la sélection intacte. Ensemble de ces codes nommé `RETIRER_SUR`, à côté
  de `RELIRE_SUR`.
- **D6 — Côté fournisseur retirable (C-43-18).** Le débit (`supplier_invoice_group`) et ses tests vivent dans des commits à
  part, retirables si la 43-1b sort : sans eux, le bouton n'apparaît que sur un crédit.

## Critères d'acceptation

- **AC1** — Bouton selon D1 : présent sur un crédit CHF en attente ; absent sur une transaction rapprochée, rejetée, de
  montant nul, en EUR ; sur une ligne signalée « déjà comptabilisée », le dialogue de confirmation s'ouvre **avant** la
  modale, et l'annuler n'ouvre rien ; sur une ligne non signalée, la modale s'ouvre directement.
- **AC2** — D2 et D3 : ajout, retrait, recherche, total, différence et ses textes, bouton ; une facture imputée est passée à
  `exclude` (elle ne revient pas, et `hasMore` reste juste) ; `hasMore` → phrase qui invite à préciser la recherche ; aucun
  résultat → phrase qui le dit ; chargement → état visible.
- **AC3** — D4 : réponse de recherche hors d'ordre ignorée ; fermeture pendant l'envoi sans effet sur une modale rouverte ;
  rouverture sur une autre transaction propre.
- **AC4** — *Enregistrer* envoie `POST /reconciliation/accept` (`invoice_group` ou `supplier_invoice_group`) ; succès →
  toast, modale fermée, transaction retirée ; refus → D5 ; `failed-proposal-label.ts` gagne les codes neufs de la 43-2
  (`RECONCILIATION_GROUP_INVALID` par `reason`, `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, `RECONCILIATION_UNDERPAYMENT`,
  et la phrase du `409` de la recherche) et vérifie que `RECONCILIATION_OVERPAYMENT`, `ROUNDING_ACCOUNT_NOT_CONFIGURED` et
  `WRITE_OFF_ACCOUNT_NOT_CONFIGURED` (`nature: "bank_fees"`) ont un libellé.
- **AC5** — Annulation (M3) : `CancelReconciliationDialog`, `kind = "grouped_settlement"` → les **N** factures listées
  (`groupedInvoices`) redeviendront ouvertes, l'écriture sera contre-passée ; un refus d'annulation par facture nomme la
  facture (`cancelBlockedInvoice`) avec les textes existants mis au pluriel du groupe (« Une des factures de ce
  rapprochement, {numéro}, a été créditée… ») ; si la 43-2 ne rend pas `cancelBlockedInvoice` à temps, la limite est écrite
  au manuel et une issue s'ouvre.
- **AC6** — i18n : quatre locales (`fr-CH` rédigé ; `de-CH`, `it-CH`, `en-CH` traduits, glossaire
  `docs/i18n-glossaire.md`) ; les clés s'ajoutent **avant** le composant qui les lit ; `lint-i18n-ownership` ; `sitesTotal`
  recompté depuis la source (3086 à la tête) ; `failed-proposal-label.test.ts` : liste et recette de comptage étendues à
  `reconciliation_group.rs`.
- **AC7** — Manuel utilisateur (`docs/manual/fr/user-manual.tex`, `.tex` seulement ; § `sec:reconciliation` et
  `sec:annuler-rapprochement`) : « Régler plusieurs factures par une transaction » — gestes ; différence : frais bancaires
  admis jusqu'au plafond réglé par l'administrateur (5.00 par défaut), sens refusé, fractions de centime ; comptes de
  différences d'arrondi et de frais bancaires à désigner ; 50 factures au plus ; confirmation sur une ligne signalée « déjà
  comptabilisée » comme les autres gestes ; un règlement et un lettrage par facture ; annulation en entier seulement ;
  limites : la ligne de banque d'un règlement groupé ne porte pas de compte analytique ; le changement de compte analytique
  d'une facture réglée en groupe est refusé en 0.19.0 (43-6) ; *(avec la 43-5)* le lot pain.001 confirmé. **Contrôle
  (M6)** : `make fr` (ou `latexmk`) compile le manuel sans erreur ni `\ref` indéfini, **sans commiter le PDF** ; le
  contrôle aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`) porte sur ce PDF jetable ; le prompt de revue P1 nomme l'axe
  « manuel contre code, PDF compris ».
- **AC8** — Captures (thème clair, base de démonstration jetable), une par état d'AC2 à AC5 : modale vide, chargement,
  aucun résultat, `hasMore`, une facture, deux factures de tiers différents avec frais, différence nulle, différence trop
  grande, sens refusé, facture refusée nommée et retirée, confirmation « déjà comptabilisée », côté fournisseur (D6),
  dialogue d'annulation d'un groupe et son refus nommé — dans `/home/gcorbaz/devel/kesh-gate-logs/e433/captures/`.

## Tests

- **Vitest** :
  - `GroupSettlementModal.test.ts` : ajout, retrait, `exclude`, bouton (une facture ; frais trop grands ; sens refusé ;
    admis) ; calcul en entiers (`0.10 + 0.20` ; **10.0050 + 10.0050 contre 20.02 → différence nulle affichée, contre 20.01
    → 0.01 de frais**) ; textes de la différence ; `maxDifference` lu de la réponse (réglage à 1.00 dans la maquette) ;
    corps envoyé pour chaque famille ; refus nommé et retiré (une facture parmi trois) ; zéro facture imputée à
    l'ouverture ; gardes D4 (réponse hors d'ordre, fermeture pendant l'envoi) ;
  - `modal-guards.test.ts` (module extrait) ; `LinkSettlementModal.test.ts` reste vert sans modification de ses cas ;
  - `ReconciliationProposals.test.ts` : bouton selon `canLink` (EUR, montant nul) ; ligne signalée → dialogue avant la
    modale ; ligne non signalée → modale ; sonde négative ;
  - `CancelReconciliationDialog.test.ts` : `grouped_settlement`, refus nommé (deux motifs) ;
  - `failed-proposal-label.test.ts` (codes neufs) ; `reconciliation.api.test.ts` (recherche, `exclude`) ; désérialisation de
    l'élément d'`accepted[]` d'un groupe (43-2, M1).
- **E2E** (`frontend/tests/e2e/reconciliation-group.spec.ts`, neuf) — préconditions et nettoyage (M4) : helper
  `setDesignatedAccountsViaApi` (comptes d'arrondi **et** de frais bancaires) ajouté à `helpers/api-fixtures.ts`, réglages
  **restaurés en `finally`** ; crédit synthétique de montant et de référence **uniques à chaque exécution** (patron de
  `reconciliation-link.spec.ts`) ; en `finally` : groupe annulé, factures annulées, import supprimé. Cas : un crédit réglant
  deux factures de **deux clients** avec 0.40 de frais, puis son annulation (factures rouvertes) ; deux restes sous le
  centime ; des frais trop grands laissent *Enregistrer* inactif ; sans compte de frais, le refus s'affiche et le dit ; une
  ligne signalée « déjà comptabilisée » demande confirmation (cas ajouté ici ou à `reconciliation-already-booked.spec.ts`) ;
  *(commit D6)* un débit réglant deux factures fournisseur. Sonde négative : la transaction rapprochée n'offre plus le
  bouton.
- **Rouges d'abord** : `kesh-gate-logs/e433/rouges.log`. **Mutations** (`kesh-gate-logs/e433/mut/`) : retirer « au moins
  deux » ; retirer le contrôle du plafond ; remplacer les entiers par `Number` (le Vitest 10.0050 rougit) ; retirer la
  branche `group` de `guard` (AC1 rougit) ; retirer la garde de session (AC3 rougit).

## Planning et ordre de retrait (M5, C-43-23)

- Types (`reconciliation.types.ts`) **dès T1**, sur le contrat de la 43-2 ; Vitest tenus sur ces types ; relus à l'étape 0
  et à chaque commit de la 43-2.
- Le spec E2E s'écrit et se joue **mercredi soir** sur une branche de travail réunie (43-1a jusqu'à son T6 et 43-2 jusqu'à
  son T2 au moins), puis jeudi sur la branche réunie définitive. Tout commit de code après jeudi 12 h relance l'E2E complet
  (prévoir 1 h 30).
- Si le temps manque : commits fournisseur (D6) d'abord, puis les captures non essentielles ; **jamais l'E2E** ; les
  traductions de/it/en ne se retirent pas (la garde `i18n-keys.test.ts` exige les quatre locales) : elles s'écrivent avec
  les clés, avant le composant.

## Tâches

- [ ] **T0** — Étape 0 : LOW restants (Dev Notes) ; contrat de la 43-2 relu (fiche, puis code s'il existe) ; réponse de Guy
  sur M-A (textes de D2).
- [ ] **T1** — Types et appels (`reconciliation.types.ts`, `reconciliation.api.ts`).
- [ ] **T2** — Clés i18n des quatre locales (AC6), avant les composants.
- [ ] **T3** — `modal-guards.ts` extrait (D4), `LinkSettlementModal` repris dessus sans changement de comportement.
- [ ] **T4** — `GroupSettlementModal.svelte`, bouton, garde `group` (AC1 à AC4), côté client.
- [ ] **T5** — Dialogue d'annulation (AC5). — [ ] **T6** — Manuel et compilation jetable (AC7).
- [ ] **T7** — Commits D6 (côté fournisseur).
- [ ] **T8** — E2E (mercredi soir sur la branche de travail réunie, jeudi sur la branche définitive) ; captures (AC8).
- [ ] **T9** — Gates frontend complets ; E2E complet à la clôture ; Dev Agent Record ; prompt de revue P1.

## Note de version

- **Ajouté** — Réconciliation : « Régler plusieurs factures… » — sur une ligne du relevé, ajoutez les factures une à une ;
  le total et la différence s'affichent ; Kesh règle et lettre chacune, des frais bancaires retenus passant au compte de
  frais bancaires dans la limite réglée par l'administrateur (refs #823).

## Dev Notes

### Modules touchés

`frontend/src/lib/features/reconciliation` (modale, garde, dialogue, types, appels, `modal-guards.ts`), catalogues i18n,
`frontend/tests/e2e` (spec et helper), manuel utilisateur. `ContactPicker.svelte` n'est pas employé.

### Garde CHF et multi-devise

Le bouton suit `canLink` (CHF) et le serveur refuse une transaction non CHF (43-2, D6). Quand le multi-devise arrivera,
cette condition est le **seul** point à élargir côté écran ; la phrase est au fichier d'épique (§ travaux parallèles).

### LOW à l'étape 0

- **L-5** : `exclude` envoyé au serveur (appliqué à AC2).
- **L-6** : nom sans jeton de 3 caractères : servi par le repli `LIKE` (43-2) ; un test de la modale l'exerce par maquette.
- **L-7 (c)** et **L-8** : appliqués (mutation de la garde ; libellé exact « Rattacher à une écriture existante » et
  `data-testid`).
- **L-4** : contraste du texte de la différence vérifié par `helpers/contrast.ts` dans l'E2E.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43).
- 2026-11-17 (soir) — Révisée : interface comme bexio, tiers mixtes, écart borné, côté fournisseur retirable.
- 2026-11-17 (nuit) — Remédiée après la validation P1 (0/3/6/8) : H1 (garde « déjà comptabilisée », D1), H2 (montants
  bruts, entiers de 1/10 000, D3), H3 (gardes de session et de génération, module extrait, D4), M1 (`canLink`, CHF), M2
  (refus nommés et retirés, D5), M3 (annulation d'un groupe, `cancelBlockedInvoice`), M4 (préconditions et nettoyage de
  l'E2E), M5 (planning et ordre de retrait), M6 (manuel compilé, PDF jetable) ; LOW L-1, L-2, L-3, L-4, L-5, L-7, L-8
  appliqués, L-6 en Dev Notes ; alignée sur la remédiation P2 de la 43-1a (différences, proposition M-A en attente) et sur
  Q4 (plafond lu du réglage).
