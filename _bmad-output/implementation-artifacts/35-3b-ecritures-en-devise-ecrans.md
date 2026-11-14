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
     :655) et le complément (`complete_opening_balances` :593 → `opening_complement.rs` `create_opening_complement` :460, enveloppe en portée `Screen` de
     `create_opening_complement_in_tx` :551, qui sert aussi le mode Compléter de la 34-2 sur les comptes bancaires)
     acceptent, pour un compte en devise, le **solde en devise** et un **cours**. Un compte CHF reste inchangé.
   - **DTO.** `OpeningBalanceLineRequest` et `OpeningComplementLineRequest` prennent `foreign?: { currency, amount, rate,
     rateSource }` (forme de 35-3a AC-6 — **même type** que le `foreign` de `CreateJournalEntryLineRequest` : quatre
     `String` analysées par la même fonction, d'où un **400** et non le 422 d'axum pour `rateSource: "revaluation"`,
     statut testé sur les deux routes) ; `debit`/`credit` restent les montants **CHF, fournis par le client et
     vérifiés** par la règle de 35-3a (la route ne calcule rien ; `FX_AMOUNT_MISMATCH` sinon). `rateSource`
     `revaluation` refusé ; l'écran envoie toujours `manual` (cf. *Cours proposé*) ; l'API admet `afc`, alors contrôlé
     par `FX_RATE_NOT_AFC` de 35-3a contre le mois de la date retenue. Le sens est celui de la colonne (`debit` ou
     `credit`) qui porte le CHF. Complément : mêmes bornes de forme que `parse_complement_amount` (`opening_balances.rs:515`) sur le CHF résultant ;
     génération : ses montants se lisent par `Decimal::from_str` (`:362-368`), sans ce parseur — les bornes y viennent du
     contrôle FX de 35-3a et de `validate`. Dans les deux cas, les bornes de 35-3a AC-3 (montant ≤ 10¹³ à 4 décimales, cours ≤ 10⁶ à 10 décimales) sur la devise ; `parse_complement_lines`
     (:535-580, « exactement une colonne non nulle », borne de `Σ(débit − crédit)`) s'applique au CHF.
   - **Contrôle FX du complément avant la transaction.** `parse_complement_lines` applique à chaque ligne `foreign` le
     contrôle FX pur de 35-3a AC-3 **avant** la règle « exactement une colonne non nulle » (comme 35-3a AC-3 pour
     l'exclusivité : sans cet ordre, une ligne dont le CHF s'arrondit à 0 ou manque rendrait l'erreur de montant
     générique au lieu de `FX_LINE_ROUNDS_TO_ZERO` / `FX_AMOUNT_MISMATCH`) (bornes, signe, `FX_LINE_ROUNDS_TO_ZERO`, `FX_AMOUNT_MISMATCH` ; fonction publique de
     `kesh-core`), sans lecture de la base. Dans le dépôt, `accounting::validate` s'exécute sous verrou et son erreur
     devient `DbError::Invariant("opening-complement:unbalanced:…")` (`opening_complement.rs:699-700`), rendu en 500
     `INTERNAL_ERROR` (`errors.rs:4808-4813`) : ce `map_err` reste le filet d'un défaut de calcul, jamais le refus d'une
     saisie. Test d'API : chaque refus FX du complément rend son code en 400, jamais 500. Le refus est converti par `map_core_error` (déjà importé par la route) et son `details.line` est le rang **1-based** de la ligne dans `req.lines` (celui du corps envoyé, 35-3a AC-3). (La génération appelle
     `validate` dans la route, `:448`, par `map_core_error` : rien à déplacer.)
   - **`ComplementLine`** (`opening_complement.rs`) gagne `foreign: Option<ForeignAmount>` **sans** `Default` ; chaque
     site de construction est revu et inventorié au Change Log : `bank_accounts.rs` :764 (`opening_line` : `None`
     motivé — la 34-2 ne touche jamais un compte en devise, un compte lié ne pouvant porter de devise, 35-1), le
     parseur de la route (:566), l'aide de test `line()` de la route (`opening_balances.rs:634-645`, littéral
     `OpeningComplementLineRequest`, qui gagne `foreign`), les tests de `tests/opening_complement_repository.rs` et de
     `tests/opening_complement_bank_scope.rs`. **Propagation jusqu'à la ligne écrite** (sites où 35-3a pose `None`,
     à étendre ici) : génération — `JournalEntryLineDraft` `routes/opening_balances.rs:369`, `NewJournalEntryLine`
     `:461` ; complément — `JournalEntryLineDraft` depuis `ComplementLine` `opening_complement.rs:669`, contrepartie
     sur le report `:682` (`None` motivé : compte CHF), `NewJournalEntryLine` `:711` (recopie le `foreign` du brouillon
     validé).
   - **Statut.** `COMPLETABLE_SQL` (`opening_complement.rs` :319), la structure `CompletableAccount` de `kesh-db`
     (`opening_complement.rs:203`, lue par `query_as` :379), sa recopie dans la route (`opening_balances.rs:294`),
     `CompletableAccountDto` et le type TS
     `CompletableAccount` exposent `currency` ; la grille principale la lit sur `AccountResponse`.
   - **Cours proposé** : cours AFC du **mois précédant** la date de l'écriture (mois de clôture de l'exercice antérieur,
     cohérent avec C-35-7 : le solde d'ouverture reprend la clôture), par `lookup` de 35-2, **appelé avec le 1er jour du mois précédent** (le `lookup` ne lit que le mois de sa `date` ; hors des bornes de 35-2, mois < 2000-01, il rend 400 : l'écran passe alors en saisie manuelle, sans `POST /fetch`) ; pour un complément daté
     `TODAY` (`complementDateKind`), cours du mois de cette date. Modifiable. **Envoyé `rate_source = manual`** sur
     l'ouverture et le complément, proposé ou saisi : le contrôle `FX_RATE_NOT_AFC` de 35-3a (AC-4, 4) compare une
     ligne `afc` au cours du mois de `entry_date`, à la création **et** au `PUT` — un cours du mois précédent, ou d'un
     mois que la date recalculée du complément a quitté, y serait refusé, et l'écriture d'ouverture ne se modifierait
     plus. L'écran affiche la provenance (« proposé : AFC <mois> ») ; le Change Log de la story le dit. Sur `404 RATE_UNAVAILABLE`
     du `lookup`, même repli qu'à l'AC-2 (un `POST /exchange-rates/fetch`, puis relecture ; sinon saisie manuelle).
   - **Saisie manuelle : cours OU CHF** (ouverture, complément et formulaire d'écriture, AC-2). L'utilisateur saisit
     soit le cours, soit le montant CHF. Si le CHF est saisi : cours = CHF ÷ montant en devise, arrondi à 10 décimales ;
     le client vérifie que `round(montant × cours, 2)` redonne le CHF saisi et, sinon, ajuste le cours au dernier chiffre
     jusqu'à ce que ce soit le cas (borné : si aucun cours à 10 décimales ne convient, message, saisie refusée). Sert à
     reprendre exactement le CHF du bilan de clôture précédent ou le CHF réel d'une conversion ; `rate_source = manual`.
   - **Le serveur ne fait aucun `lookup`** (la résolution du cours est faite à l'écran, avant l'appel) et ne lit
     `_kesh_exchange_rates` **avant aucun verrou** : `create_opening_complement_in_tx` exige que la transaction n'ait
     fait que des lectures verrouillantes avant ses verrous — AC-10 de la 34-2. Le seul accès à la table est celui du
     contrôle `FX_RATE_NOT_AFC` de 35-3a, pour une ligne `afc` venue de l'API, dans `create_in_tx_inner`, après les
     verrous ; le contrôle de devise de 35-3a AC-4 y reste aussi.
   - **Date du complément recalculée par le serveur** (sous verrou, « indicative » à l'écran) : l'écran affiche la date
     retenue ; si son mois diffère de celui du cours proposé, le `POST` n'est **pas** refusé — le cours reste celui
     saisi, `rate_source = manual` (ce qui rend le non-refus vrai : une ligne `afc` serait contrôlée).
   - **Écran** (`settings/opening-balances/+page.svelte`) : ligne d'un compte en devise = montant en devise dans la
     colonne Débit ou Crédit, cours, CHF calculé (ou saisi, cf. ci-dessus). Le bandeau de total en direct
     (`computeBalance(rows)`), `computeOpeningTotals` et `retainedEarningsWarning`
     (`features/opening-balances/opening-balances-totals.ts`), ainsi que `complementCounterpart` (:102, contrepartie
     affichée du complément, page :173), reçoivent le **CHF** des lignes en devise. Les deux
     grilles (principale et complément) sont couvertes ; `load()` conserve la saisie du complément (`previous`, :116-123)
     **y compris `foreign`** (cours, source) après un refus.
   - **Contrat des lignes de grille** (`GridRow` :44, `ComplementRow` :56) : celui du `LineDraft` de l'AC-2 (montant et
     sens dans `foreign`, `debit`/`credit` = CHF dérivé seul). Une ligne en devise dont le montant est saisi sans CHF
     calculable (cours vide ou invalide) est **incomplète** : elle n'est **jamais retirée en silence** du corps —
     `nonEmptyRows` (:147-153) et `complementLines` (:163-169) la filtreraient aujourd'hui comme vide, et l'écriture
     partirait sans ce compte —, elle désactive `canGenerate` / `canSubmitComplement` (comme `complementInvalid` :170)
     avec un message sur la ligne ; les corps envoyés (`handleGenerate` :196-201, `handleComplete` :240-245) portent
     `foreign`. Vitest : montant en devise sans cours → bouton désactivé, aucun appel.
   - **Tests.** Dépôt : ouverture et complément avec ligne en devise, contrepartie sur le compte de report ; API :
     `FX_AMOUNT_MISMATCH`, devise ≠ devise du compte, mono-devise → `MULTI_CURRENCY_DISABLED`, `revaluation` refusé ;
     aller-retour `.keshbackup` ; **non-régression du mode Compléter bancaire** (34-2, compte CHF inchangé) ; Vitest de
     l'écran et des totaux (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`).
2. **AC-2 — Formulaire d'écriture** (`JournalEntryForm.svelte`), en multi-devises seulement :
   - **Sens.** Une ligne en devise se saisit dans la colonne **Débit ou Crédit en montant en devise** (le sens est
     celui de la colonne) ; le CHF est calculé (`round(montant × cours, 2)`, `Big.roundHalfUp`), affiché en lecture
     seule sous le champ, et alimente `computeBalance` et la ligne d'écart. `debit`/`credit` du payload restent les
     montants CHF, `foreign.amount` le montant en devise. `LineDraft` (`form-helpers.ts` :13) gagne
     `foreign?: { currency, side: 'debit' | 'credit', amount, rate, rateSource }` (absent = ligne CHF) — **contrat de
     stockage** : le montant saisi en devise vit dans `foreign.amount` et le sens dans `foreign.side` (il doit survivre
     à un cours ou un CHF encore vides) ; `debit`/`credit` de la ligne ne portent **que le CHF dérivé** (l'un des deux
     vide, selon `side`), recalculé à chaque changement et seul lu par `computeBalance`, `classifyLine` et
     `isDraftLineNonEmpty` — aucun d'eux ne lit jamais `foreign.amount` comme du CHF ; une ligne en devise sans CHF
     calculable (cours ou montant vide ou invalide) est `partial` (le message de la ligne est celui du code rendu par `computeChf` — `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO` — traduit, non un « montant invalide » générique) ; `classifyLine` (`balance.ts` :143-162),
     `computeBalance` (`balance.ts` :83), `isDraftLineNonEmpty` (`vat-purchase.ts` :106) et les fabriques de lignes
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
     fois** `POST /exchange-rates/fetch` pour ce mois — par la fonction d'API de 35-2 (seule entrée : sans bandeau
     dégradé sur 503, `timeoutMs` de 90 s, 35-2 AC-3 et AC-5), le mois tiré de `details.month` (`AAAA-MM-01`) ; les specs
     E2E pilotent la source factice et s'appuient sur le vidage des tables par `/_test/seed` (35-2 AC-7) — (rôle Comptable ou Administrateur ; mois ≤ mois courant), puis
     relit ; `404 RATE_NOT_PUBLISHED`, `503` ou rôle Consultation → message, saisie manuelle exigée. Pour un mois
     **postérieur** au mois courant, **aucun** `POST /fetch` (il rendrait 400, 35-2 AC-5) : saisie manuelle
     directement ; un 400 du `POST /fetch` se traite comme `RATE_NOT_PUBLISHED`. Le cours du mois
     courant **est** publié dès le début du mois (relevé le 2026-11-14 : `xmlavgmonth` sans paramètre rendait
     `2026-11`) ; le cas « cours indisponible » vise surtout un mois futur, une devise non publiée ou une source
     injoignable. *(Annotation de l'orchestrateur, 2026-11-14 : la remédiation P1 avait écrit l'inverse.)*
   - **Formats et bornes** — celles de 35-3a AC-3, à l'identique : le cours se valide avec `AmountFormatOptions`
     `{ maxDecimals: 10, maxIntegerDigits: 7 }` (`balance.ts` :23-31) puis cours ≤ 10⁶, rejette 0 ; le montant en
     devise : 4 décimales, ≤ 10¹³ ; message d'erreur propre au cours (le « Maximum 4 décimales » existant ne vaut que
     pour les montants). `computeChf` rend l’erreur `FX_VALUE_OUT_OF_RANGE` sur ces bornes **et sur un montant ou un cours ≤ 0** (35-3a AC-3 étape 2 : même code) (les vecteurs `error` de la
     fixture partagée l'exigent), **et aussi pour un CHF arrondi > 10¹³** (borne
     de 35-3a AC-3, vecteur « CHF > 10¹³ » de la fixture) — une seule borne, au client comme au serveur, et non les
     quinze chiffres entiers de `DECIMAL(19,4)` ; un cours déduit d'un CHF saisi passe par les mêmes bornes. Le payload sérialise par `toFixed`, jamais `toString()`.
   - **Refus du serveur propres aux devises** (`POST` et `PUT`) : `RATE_UNAVAILABLE {currency, month}` — p. ex. le
     `PUT` d'une écriture ancienne à lignes `afc` sur une installation restaurée, dont la table des cours, système, est
     vide (35-2 AC-7) — → l'écran propose « Récupérer le mois » (`POST /exchange-rates/fetch`, mêmes rôles et même
     règle que ci-dessus) puis le renvoi, ou le passage de la ligne en `manual` ; `FX_RATE_NOT_AFC` → le cours attendu
     (`details.expectedRate`, 35-3a AC-4) est affiché et proposé à la reprise ; `FX_AMOUNT_MISMATCH` porte
     `details.expected` / `details.provided` (35-3a AC-3) ; `FX_CURRENCY_MISMATCH`, `ACCOUNT_REQUIRES_CURRENCY`,
     `FX_AMOUNT_MISMATCH` → message traduit sur la ligne fautive (numéro en `details` : le rang **1-based** de la ligne dans `lines`,
     lu dans `details.line` (35-3a AC-3), y compris pour `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO` et
     `FX_RATE_NOT_AFC`). ⚠️ Ce rang est celui du **corps envoyé**, qui omet les lignes `empty` (`nonEmptyLines`,
     `JournalEntryForm.svelte:171-174`, payload :206) : l'écran le traduit en ligne du formulaire par l'`index` de
     `nonEmptyLines[line − 1]`, jamais par `lines[line − 1]` ; même règle sur les grilles d'ouverture (`nonEmptyRows`,
     `complementLines`). Vitest de chacun, dont un refus sur une ligne précédée d'une ligne vide. `editRefusalOutcome`
     (`form-helpers.ts` :108) classe ces refus en édition : `ENTRY_IS_FX_REVALUATION` → `stale` (la fiche se recharge,
     le motif s'affiche) ; `RATE_UNAVAILABLE`, `FX_*`, `ACCOUNT_REQUIRES_CURRENCY`, `MULTI_CURRENCY_DISABLED` (réglage retiré entre le chargement et l'envoi) → `stay` (le formulaire reste
     ouvert) ; sinon ils tomberaient en `other`. Test par code.
   - **Ligne d'écart** : écriture déséquilibrée en CHF avec au moins une ligne en devise → bouton « Ajouter la
     différence de change » : une ligne CHF sur le compte de gains (écart au crédit) ou de pertes (au débit) désigné en
     35-1 ; au-delà de n × 0.005 CHF (n = nombre de lignes en devise, calculé en `Big`), le texte précise qu'il ne s'agit
     pas d'un arrondi ; rien n'est ajouté sans clic (C-35-4). Le formulaire reçoit les props `multiCurrency`,
     `fxGainAccountId`, `fxLossAccountId`, fournies par `journal-entries/+page.svelte` (:163) et
     `journal-entries/[id]/+page.svelte` (:305), qui chargent déjà `fetchCompanyCurrent`. Si cet appel échoue
     (les deux pages le tolèrent), `multiCurrency` vaut vrai dès qu'un compte de `accounts` porte une devise (35-1 : un
     compte en devise n'existe qu'en multi-devises ; `AccountResponse.currency` et `currencyLocked` sont **omis** pour
     un compte CHF, 35-1 AC-4 et C-35-9 : type TS `currency?: string`, tout test de devise lit `account.currency ?? null`,
     jamais `=== null` — alignement de l'orchestrateur sur 35-1-P5-IF1) — sans quoi l'édition d'une écriture en devise perdrait `foreign`
     au `PUT` ; les comptes de différence restent alors inconnus et le bouton d'écart l'explique. **Contrôle client seulement**
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
   `foreign`. Une écriture portant une ligne `revaluation` **n'est pas modifiable** : le champ existant
   `modificationBlockedBy` (`journal-entries.types.ts:84`, union `ModificationBlocker` :117 ; serveur
   `ModificationBlocker`/`ModificationGuard`, `kesh-db/src/errors.rs:134,210`) reçoit la **valeur** neuve
   `ENTRY_IS_FX_REVALUATION`, rendue par le serveur (35-3a AC-7) ; 35-3b l'ajoute à l'union TS et à
   `modificationBlockerLabel` (quatre locales ; `blocker-messages.test.ts`, « couvre exactement douze codes »
   `:51-54`, passe à treize), et la fiche affiche le motif. Les types TS `JournalEntryLineResponse` et
   `CreateJournalEntryLineRequest` (`journal-entries.types.ts`) gagnent `foreign`. « Modifier » disparaît ; **« Supprimer »
   reste** pour ce motif — la suppression et la contre-passation sont les corrections prévues (35-5 AC-3) — alors
   qu'aujourd'hui les deux boutons partagent la condition `entry.modifiable` (`journal-entries/[id]/+page.svelte`
   :418-432) : la condition de « Supprimer » devient `modifiable || (modificationBlockedBy === 'ENTRY_IS_FX_REVALUATION' &&
   !periodLocked && !lettered)`. ⚠️ Les deux termes sont nécessaires : le motif `ENTRY_IS_FX_REVALUATION` se rend
   **avant** `PERIOD_LOCKED` et la marque de lettrage (35-3a AC-7 ; `modification_blocker`, `journal_entries.rs:1243`),
   que `delete_in_tx` contrôle (étapes 3-quater et 3-quinquies) ; sans eux, le bouton serait offert pour un refus
   certain, contre la règle de la page (« les boutons sont ABSENTS, pas désactivés », `:410-414`). `lettered` se lit
   sur les lignes (`letteringKey`), `periodLocked` compare `entryDate` à `booksLockedThrough` (`fetchCompanyCurrent`,
   chargé au montage de la fiche — il ne l'est aujourd'hui qu'à l'édition, `:305` ; inconnu → bouton absent). Vitest
   des boutons : réévaluation libre, réévaluation en période verrouillée, réévaluation lettrée.
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
     `bank-accounts/+page.svelte` (`<select>` :860, :1116) et `BankAccountJournalLinkForm` (:67) — tous trois par
     `linkableLedgerAccounts` (`features/bank-accounts/format.ts:180`), où le filtre se pose une fois —,
     `VatPurchaseAssistant` (:159, :204 — reçoit les comptes **non filtrés** du formulaire,
     `JournalEntryForm.svelte:350-355` ; ses deux `AccountAutocomplete` gardent le défaut `excludeForeignCurrency =
     true`, qui suffit — ne pas filtrer dans le formulaire, qui a besoin des comptes en devise).
   - **À ne pas filtrer** (libellés, consultation, ou saisie en devise) : `reports`, `OpenItemsScreen`,
     `accounts/+page.svelte`, `journal-entries/[id]`, `journal-entries/+page.svelte` (liste, et comptes passés au
     formulaire), `RulesList`, et **`settings/opening-balances/+page.svelte`** (`fetchAccounts` :8, :77 : les comptes en
     devise y figurent, exception C-35-5) — le filtre ne se pose donc pas dans `fetchAccounts`. `onboarding/+page.svelte`
     (:198-204) passe par `linkableLedgerAccounts` et reçoit le filtre sans effet (aucun compte en devise avant D1).
   - `withCurrentAccount` (`account-options.ts` :58) conserve la valeur déjà enregistrée **avec avertissement** ;
     `isAccountUnusable` (`account-validity.ts:54`) la signale non utilisable (le backend la refusera à l'écriture)
     **par une option** `excludeForeignCurrency` d'`AccountValidityOptions` (défaut `true`) : la fonction est aussi
     appelée par `JournalEntryForm` (:135) et par `AccountAutocomplete` (:219), qui passent `false` dans le formulaire
     d'écriture — sans quoi chaque compte en devise y serait marqué inutilisable. Vitest des deux cas.
   - ⚠️ `SettleInvoiceDialog`, `SupplierInvoiceForm` (`expenseAccounts`), `RuleFormModal` et `RulesList` **n'importent ni
     `fetchAccounts` ni `AccountAutocomplete`** : ils reçoivent leurs comptes en props (`accounts: AccountResponse[]`).
     Le filtre se pose donc, pour eux, soit à l'entrée du composant (`withoutForeignCurrency`), soit dans la page
     parente qui appelle `fetchAccounts` — au choix, écrit à l'inventaire, mais un seul des deux par composant.
   - Garde Vitest d'inventaire : tout fichier `.svelte` ou `.ts` (hors tests) qui importe `fetchAccounts`,
     `AccountAutocomplete` ou `linkableLedgerAccounts`, **ou** qui déclare une prop de type `AccountResponse[]`, porte
     l'une des deux marques ou une exemption justifiée (les composants de la liste « à ne pas filtrer » et
     `onboarding` compris) ; le test lit le source, il ne se fie pas à la liste du Change Log. La détection de la
     prop admet `readonly AccountResponse[]` (`CounterpartyClaimNotice.svelte:18`, libellés : exemption) ; les
     fichiers `*.test.svelte` (`SettleInvoiceDialogHost.test.svelte`) comptent comme tests ; les aides `.ts` qui
     prennent un `AccountResponse[]` en **paramètre** (`account-label.ts`, `account-options.ts`,
     `account-validity.ts`, `accounts.api.ts`, `opening-balances-totals.ts`, `bank-accounts/format.ts`,
     `reconciliation-counterparties.ts`) ne déclarent pas de prop : hors garde, sauf si elles importent l'un des trois
     symboles (règle de détection écrite au test, cas de chacune vérifié par un Vitest de la garde).
   - **Hors périmètre, dit** : l'assistant d'onboarding (étape compte bancaire, `onboarding/+page.svelte` :198-204) —
     le mode multi-devises s'active après (D1) ; le mode Compléter bancaire (34-2) ne reçoit jamais `foreign` ; les
     imports bancaires restent hors périmètre (D7).
5. **AC-5 — E2E** : écriture USD au cours AFC (cours servis par la source factice du mode test de 35-2 AC-3, pilotée
   par `/api/v1/_test/exchange-rates/source`), conversion EUR → USD avec ligne d'écart, saisie du CHF au lieu du
   cours, soldes d'ouverture d'un compte USD (l'E2E repart d'une société vierge par `seedTestState('with-company')`,
   précédent `opening-balances.spec.ts:21, :80` : l'ouverture exige une société sans écriture, `ALREADY_HAS_ENTRIES`),
   mono-devise inchangé ;
   préalables posés par l'API (Administrateur) : mode multi-devises activé **avec les comptes de gain et de perte de
      change désignés** (35-1 AC-3, sans quoi le bouton d'écart n'ajoute rien), compte USD (et EUR) créé, cours des
      mois utiles servis par la source factice — dont le mois **précédant** l'ouverture ; un scénario au moins part
      d'une table des cours vide et exerce le repli de l'écran (`404 RATE_UNAVAILABLE` → `POST /exchange-rates/fetch`
      → relecture), les autres récupèrent leurs mois par `POST /exchange-rates/fetch` en préalable (`lookup` ne lit que
      la table) ;
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
  portant un rôle ; lier/désigner un compte en devise refusé) ; 35-3a (produit en entiers `i128`, jamais `Decimal::checked_mul`, `FX_VALUE_OUT_OF_RANGE`,
  fixture `fx_rounding.json`, valeur `ENTRY_IS_FX_REVALUATION` de `modificationBlockedBy` rendue par le serveur,
  contrôle FX par ligne exposé en fonction publique de `kesh-core`) ; C-35-2 (3) amendé.

## Change Log

- 2026-11-14 — fiche rédigée (découpe de 35-3).
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 8 MEDIUM, 3 LOW ; remédiés ; P1-4 (35-1), P1-7 (35-3a) et
  l'amendement de C-35-2 portés par l'orchestrateur.
- 2026-11-14 — validation P2 (Opus) : 0 CRITICAL, 3 HIGH, 6 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  35-3b-P2-4 et 35-3b-P2-2 pour leur part dans 35-3a (inter-fiches, portés par l'orchestrateur).
- 2026-11-14 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 3 LOW ; remédiés dans la fiche sauf aucun
  inter-fiches (P3-1 stockage `LineDraft`, P3-2 ordre des contrôles du complément, P3-3 garde d'inventaire des
  sélecteurs, P3-4 `editRefusalOutcome` ; LOW : repli `lookup` de l'ouverture, bornes de la génération, préalables E2E).
- 2026-11-14 — validation P4 (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 7 LOW ; remédiés dans la fiche sauf aucun
  inter-fiches (P4-1 borne CHF 10¹³ de `computeChf`, P4-2 rang `details.line` traduit par `nonEmptyLines`, P4-3 ligne
  en devise incomplète jamais retirée en silence des grilles d'ouverture, P4-4 « Supprimer » d'une réévaluation
  conditionné au verrou de période et au lettrage ; LOW : DTO `foreign` en `String`, mois futur sans `POST /fetch`,
  repli de `multiCurrency`, sites oubliés (`CompletableAccount`, aide de test, `blocker-messages.test.ts`, types TS),
  garde d'inventaire et prémisse de `VatPurchaseAssistant`, numéros de ligne, préalables E2E).
- 2026-11-14 — validation P5 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW ; remédiés dans la fiche sauf aucun inter-fiches (numéros de ligne `JournalEntryForm` 171-174/:206 ; date du `lookup` du mois précédent et ses bornes ; `map_core_error` et rang `details.line` du complément ; `MULTI_CURRENCY_DISABLED` classé `stay` et message des lignes `partial`).
