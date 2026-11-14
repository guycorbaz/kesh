# Story 35-3b : écritures en devise — soldes d'ouverture, formulaire et écrans

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-4, C-35-5. Suit **35-3a**
(découpe de l'ancienne 35-3 à la rédaction). **Aucune migration.***

## Story

En tant que comptable en multi-devises, je veux saisir à l'écran une écriture en devise avec le cours officiel
proposé, saisir mes propres cours (ou le montant CHF réel) pour une conversion et y ajouter d'un clic la différence de
change, et ouvrir mes comptes en devise avec leur solde en devise, afin de tenir mes livres sans passer par l'API.

## Critères d'acceptation

1. **AC-1 — Soldes d'ouverture** (serveur et écran).
   - **Routes.** `generate_opening_balances` (`routes/opening_balances.rs` :345 → `journal_entries::create_opening_entry`
     :655) et le complément (`create_opening_complement` :460, enveloppe en portée `Screen` de
     `create_opening_complement_in_tx` :551, qui sert aussi le mode Compléter de la 34-2 sur les comptes bancaires)
     acceptent, pour un compte en devise, le **solde en devise** et un **cours**. Un compte CHF reste inchangé.
   - **DTO.** `OpeningBalanceLineRequest` et `OpeningComplementLineRequest` prennent `foreign?: { currency, amount, rate,
     rateSource }` (forme de 35-3a AC-6) ; `debit`/`credit` restent les montants **CHF, fournis par le client et
     vérifiés** par la règle de 35-3a (la route ne calcule rien ; `FX_AMOUNT_MISMATCH` sinon). `rateSource`
     `revaluation` refusé. Le sens est celui de la colonne (`debit` ou `credit`) qui porte le CHF. Mêmes bornes de forme
     que `parse_complement_amount` sur le montant en devise et sur le CHF résultant ; `parse_complement_lines`
     (:535-590, « exactement une colonne non nulle », borne de `Σ(débit − crédit)`) s'applique au CHF.
   - **`ComplementLine`** (`opening_complement.rs`) gagne `foreign: Option<ForeignAmount>` **sans** `Default` ; chaque
     site de construction est revu et inventorié au Change Log : `bank_accounts.rs` :764 (`opening_line` : `None`
     motivé — la 34-2 ne touche jamais un compte en devise, un compte lié ne pouvant porter de devise, 35-1), le
     parseur de la route, les tests de `tests/opening_complement_repository.rs`.
   - **Statut.** `COMPLETABLE_SQL` (`opening_complement.rs` :319), `CompletableAccountDto` et le type TS
     `CompletableAccount` exposent `currency` ; la grille principale la lit sur `AccountResponse`.
   - **Cours proposé** : cours AFC du **mois précédant** la date de l'écriture (mois de clôture de l'exercice antérieur,
     cohérent avec C-35-7 : le solde d'ouverture reprend la clôture), par `lookup` de 35-2 ; pour un complément daté
     `TODAY` (`complementDateKind`), cours du mois de cette date. Modifiable ; `rate_source` `afc` ou `manual`.
   - **Saisie manuelle : cours OU CHF** (ouverture, complément et formulaire d'écriture, AC-2). L'utilisateur saisit
     soit le cours, soit le montant CHF. Si le CHF est saisi : cours = CHF ÷ montant en devise, arrondi à 10 décimales ;
     le client vérifie que `round(montant × cours, 2)` redonne le CHF saisi et, sinon, ajuste le cours au dernier chiffre
     jusqu'à ce que ce soit le cas (borné : si aucun cours à 10 décimales ne convient, message, saisie refusée). Sert à
     reprendre exactement le CHF du bilan de clôture précédent ou le CHF réel d'une conversion ; `rate_source = manual`.
   - **Le serveur ne fait aucun lookup ni lecture de `_kesh_exchange_rates`** (la résolution du cours est faite avant
     l'appel, hors transaction : `create_opening_complement_in_tx` exige que la transaction n'ait fait que des
     lectures verrouillantes avant ses verrous — AC-10 de la 34-2). Le contrôle de devise de 35-3a AC-4 reste dans
     `create_in_tx_inner`, après les verrous.
   - **Date du complément recalculée par le serveur** (sous verrou, « indicative » à l'écran) : l'écran affiche la date
     retenue ; si son mois diffère de celui du cours proposé, le `POST` n'est **pas** refusé — le cours reste celui
     saisi, `rate_source` inchangé.
   - **Écran** (`settings/opening-balances/+page.svelte`) : ligne d'un compte en devise = montant en devise dans la
     colonne Débit ou Crédit, cours, CHF calculé (ou saisi, cf. ci-dessus). Le bandeau de total en direct
     (`computeBalance(rows)`), `computeOpeningTotals` et `retainedEarningsWarning`
     (`features/opening-balances/opening-balances-totals.ts`) reçoivent le **CHF** des lignes en devise. Les deux
     grilles (principale et complément) sont couvertes ; `load()` conserve la saisie du complément (`previous`, :116-123)
     **y compris `foreign`** (cours, source) après un refus.
   - **Tests.** Dépôt : ouverture et complément avec ligne en devise, contrepartie sur le compte de report ; API :
     `FX_AMOUNT_MISMATCH`, devise ≠ devise du compte, mono-devise → `MULTI_CURRENCY_DISABLED`, `revaluation` refusé ;
     aller-retour `.keshbackup` ; **non-régression du mode Compléter bancaire** (34-2, compte CHF inchangé) ; Vitest de
     l'écran et des totaux (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`).
2. **AC-2 — Formulaire d'écriture** (`JournalEntryForm.svelte`), en multi-devises seulement :
   - **Sens.** Une ligne en devise se saisit dans la colonne **Débit ou Crédit en montant en devise** (le sens est
     celui de la colonne) ; le CHF est calculé (`round(montant × cours, 2)`, `Big.roundHalfUp`), affiché en lecture
     seule sous le champ, et alimente `computeBalance` et la ligne d'écart. `debit`/`credit` du payload restent les
     montants CHF, `foreign.amount` le montant en devise. `LineDraft` (`form-helpers.ts` :14) gagne
     `foreign?: { currency, amount, rate, rateSource }` (absent = ligne CHF) ; `classifyLine`, `computeBalance`
     (`balance.ts` :83, :140-162), `isDraftLineNonEmpty` (`vat-purchase.ts` :106) et les fabriques de lignes
     (`JournalEntryForm` :105-106, :187 ; `vat-purchase.ts` :87-96) sont revus, inventaire au Change Log.
     Changer le compte d'une ligne : CHF → devise, champs initialisés ; devise → CHF, `foreign` effacé ;
     USD → EUR, cours réinitialisé.
   - **Champs** : *Montant (USD)*, *Cours* (prérempli par `lookup` du mois de la date ; indicateur « AFC » ou
     « manuel » ; modifié à la main → `manual`), *CHF* ; comme à l'AC-1, **soit le cours soit le CHF** se saisit à la
     main (CHF saisi → cours = CHF ÷ montant à 10 décimales, vérification `round(montant × cours, 2) = CHF`, ajustement
     au dernier chiffre) ; changer la date repropose les cours AFC des lignes dont le cours n'a pas été modifié à la
     main.
   - **Appels `lookup`** : mémoïsation par `(devise, mois)` par formulaire, **résultats négatifs compris** ; appel
     seulement pour une date valide dans `dateBounds` et après débounce (`debounce.ts`) ; jeton de requête pour ignorer
     une réponse périmée ; si le nouveau mois n'a pas de cours, le cours `afc` précédent est **effacé** (jamais gardé
     sous l'étiquette `afc`) ; en édition, aucune reproposition au montage, seulement au changement de date et pour les
     lignes `afc`. `GET /lookup` est **en lecture seule** (35-2) : sur `404 RATE_UNAVAILABLE`, l'écran appelle **une
     fois** `POST /exchange-rates/fetch` pour ce mois (rôle Comptable ou Administrateur ; mois ≤ mois courant), puis
     relit ; `404 RATE_NOT_PUBLISHED`, `503` ou rôle Consultation → message, saisie manuelle exigée. Le cours du mois
     courant **est** publié dès le début du mois (relevé le 2026-11-14 : `xmlavgmonth` sans paramètre rendait
     `2026-11`) ; le cas « cours indisponible » vise surtout un mois futur, une devise non publiée ou une source
     injoignable. *(Annotation de l'orchestrateur, 2026-11-14 : la remédiation P1 avait écrit l'inverse.)*
   - **Formats.** Le cours se valide avec `AmountFormatOptions` `{ maxDecimals: 10, maxIntegerDigits: 10 }`
     (`balance.ts` :22-31), rejette 0 ; le montant en devise : 4 décimales ; message d'erreur propre au cours (le
     « Maximum 4 décimales » existant ne vaut que pour les montants). Le client refuse un CHF calculé au-delà de
     15 chiffres entiers (`DECIMAL(19,4)`). Le payload sérialise par `toFixed`, jamais `toString()`.
   - **Ligne d'écart** : écriture déséquilibrée en CHF avec au moins une ligne en devise → bouton « Ajouter la
     différence de change » : une ligne CHF sur le compte de gains (écart au crédit) ou de pertes (au débit) désigné en
     35-1 ; au-delà de n × 0.005 CHF (n = nombre de lignes en devise, calculé en `Big`), le texte précise qu'il ne s'agit
     pas d'un arrondi ; rien n'est ajouté sans clic (C-35-4). Le formulaire reçoit les props `multiCurrency`,
     `fxGainAccountId`, `fxLossAccountId`, fournies par `journal-entries/+page.svelte` (:163) et
     `journal-entries/[id]/+page.svelte` (:305), qui chargent déjà `fetchCompanyCurrent`. **Contrôle client seulement**
     (la ligne est une ligne CHF ordinaire envoyée au `POST` ; la revalidation serveur de C-35-2 (3) vaut pour la
     réévaluation) : le bouton revalide sur la liste des comptes (existe, actif, imputable, `Expense`/`Revenue`, sans
     devise) ; comptes non désignés ou invalides : il explique quoi régler.
   - **Arrondi** : calcul côté client avec `big.js` dans une fonction pure (`fx.ts`, `computeChf`), `.round(2,
     Big.roundHalfUp)` **explicite** (pas `Big.RM` global), même résultat que le serveur (`MidpointAwayFromZero`) ;
     Vitest lit la fixture de vecteurs **partagée** `crates/kesh-core/tests/fixtures/fx_rounding.json` fournie par 35-3a
     (dont `x.xx5`, cours à 10 décimales, 15 chiffres, hors bornes).
   - **Mono-devise** : formulaire strictement inchangé (test de non-régression).
3. **AC-3 — Affichage et édition** : détail et liste des écritures montrent montant en devise (avec son code), cours
   **sans zéros terminaux** et source (`AFC`, `manuel`, `réévaluation`) à côté du CHF ; `lineResponseToDraft` recharge
   `foreign`. Une écriture portant une ligne `revaluation` **n'est pas modifiable** (`modificationBlockedBy` neuf,
   ajouté par 35-3a/35-5 ; la fiche affiche le motif et le bouton disparaît) ; la contre-passation reste permise.
4. **AC-4 — Sélecteurs** des factures, avoirs, règlements, factures fournisseur, lots, réconciliation, comptes
   bancaires liés, comptes par défaut, produits, assistant TVA achat : les comptes en devise n'y apparaissent pas.
   - Un seul prédicat `isForeignCurrencyAccount(account)` ; `AccountAutocomplete` prend `excludeForeignCurrency`
     (défaut `true` ; `false` seulement dans `JournalEntryForm`) ; les `<select>` natifs passent par
     `withoutForeignCurrency` ; le filtre central `reconciliation-counterparties.ts` :29 l'applique.
   - Inventaire **clos** au Change Log, à partir de l'inventaire serveur de 35-3a et du grep des composants qui
     importent `fetchAccounts`/`AccountAutocomplete` : au moins `InvoiceForm`, `invoices/[id]`, `invoices/due-dates`,
     `credit-notes/[id]`, `SettleInvoiceDialog`, `SupplierInvoiceForm` et les pages fournisseur
     (`new`, `[id]`, `[id]/edit`), `settings/invoicing`, `products/+page.svelte` (:697), `ManualMatchModal`,
     `TransactionSplitModal`, `ReconciliationProposals`, `reconciliation/rules` (`RuleFormModal`),
     `bank-accounts/+page.svelte` (:703, :860, :1116), `BankAccountJournalLinkForm`, `VatPurchaseAssistant`
     (:159, :204 — reçoit les comptes filtrés du formulaire).
   - **À ne pas filtrer** (libellés, consultation) : `reports`, `OpenItemsScreen`, `accounts/+page.svelte`,
     `journal-entries/[id]`, liste des écritures, `RulesList` — le filtre ne se pose donc pas dans `fetchAccounts`.
   - `withCurrentAccount` (`account-options.ts` :48-62) conserve la valeur déjà enregistrée **avec avertissement** ;
     `isAccountUnusable` (`account-validity.ts`) la signale non utilisable (le backend la refusera à l'écriture).
   - Garde Vitest d'inventaire : tout composant important `fetchAccounts` ou `AccountAutocomplete` porte l'une des
     deux marques ou une exemption justifiée.
   - **Hors périmètre, dit** : l'assistant d'onboarding (étape compte bancaire, `onboarding/+page.svelte` :198-204) —
     le mode multi-devises s'active après (D1) ; le mode Compléter bancaire (34-2) ne reçoit jamais `foreign` ; les
     imports bancaires restent hors périmètre (D7).
5. **AC-5 — E2E** : écriture USD au cours AFC (cours servis par le faux serveur en mode test de 35-2 AC-8), conversion
   EUR → USD avec ligne d'écart, saisie du CHF au lieu du cours, soldes d'ouverture d'un compte USD (l'E2E crée sa
   propre société vierge : l'ouverture exige une société sans écriture, `ALREADY_HAS_ENTRIES`), mono-devise inchangé ;
   captures d'écran des états (thème clair) ; liste des échecs attendus de `docs/testing.md` recontrôlée.
6. **AC-6 — Manuel utilisateur** : saisir une écriture en devise, cours proposé et manuel (cours ou CHF), conversion
   EUR → USD pas à pas, soldes d'ouverture en devise (« solde repris au cours de clôture », mois précédent) ; i18n
   quatre locales ; PDF contrôlé.

## Tâches

- [ ] Soldes d'ouverture : serveur (DTO, `ComplementLine`, statut), écran, totaux, tests (AC-1).
- [ ] `fx.ts` (calcul CHF, cours depuis CHF), formulaire, `balance.ts`, `form-helpers.ts`, détail, liste, Vitest sur la
      fixture partagée (AC-2, AC-3).
- [ ] Sélecteurs, prédicat commun, garde d'inventaire (AC-4).
- [ ] E2E, captures, manuel, i18n, CHANGELOG (AC-5, AC-6).

## Notes de développement

- `frontend/src/lib/features/journal-entries/` : `JournalEntryForm.svelte` (575 lignes), `balance.ts` (`parseAmount`
  :68, `computeBalance` :83, `formatSwissAmount` :129), `form-helpers.ts`, `vat-purchase.ts`,
  `journal-entries.types.ts`.
- Écrans des soldes d'ouverture : `frontend/src/routes/(app)/settings/opening-balances/` ;
  `frontend/src/lib/features/opening-balances/` (`opening-balances-totals.ts`, types).
- `kesh-db` : `repositories/opening_complement.rs` (`COMPLETABLE_SQL` :319, `create_opening_complement_in_tx` :551,
  `check_lines`), `routes/bank_accounts.rs` :764.
- Dépendances sur d'autres fiches (portées ailleurs) : 35-1 (devise refusée sur un compte lié, désigné par défaut ou
  portant un rôle ; lier/désigner un compte en devise refusé) ; 35-3a (`checked_mul`, `FX_AMOUNT_OUT_OF_RANGE`,
  fixture `fx_rounding.json`, `modificationBlockedBy` pour `revaluation`) ; C-35-2 (3) amendé.

## Change Log

- 2026-11-14 — fiche rédigée (découpe de 35-3).
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 8 MEDIUM, 3 LOW ; remédiés ; P1-4 (35-1), P1-7 (35-3a) et
  l'amendement de C-35-2 portés par l'orchestrateur.
